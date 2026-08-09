"""Debugger fuer GameBasic (Editor) -- ueber die native Runtime `dhrt debug`.

`DebugController` spawnt `dhrt debug datei.gb` und spricht dessen newline-JSON-
Protokoll: ein Reader-Thread liest stdout-Events (paused/output/finished/error)
und uebersetzt sie in Qt-Signale; Steuer-Kommandos (continue/step/stop/
set-breakpoints) gehen als JSON-Zeilen an stdin. Die Pause-Logik (Breakpoints
inkl. Bedingungen, Step over/into/out, Variablen-Snapshot) liegt in dhrt
(vm.rs); hier nur noch das Protokoll <-> Qt-Glue.

Die oeffentliche API (Signale + Methoden) ist unveraendert, damit main_window
nicht angefasst werden muss. Zeilen sind quell-relativ; bei `IMPORT "datei.gb"`-
Inlining koennen Breakpoint-/Pause-Zeilen verschoben sein.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
import threading
import time
from pathlib import Path

import shiboken6
from PySide6.QtCore import QObject, Signal

# Review-Fund: siehe error_check.py -- geteilter Alias statt vierfach
# duplizierter Wrapper-Funktion, bleibt fuer bestehende Tests patchbar.
from .dhrt_locate import find_dhrt as _find_dhrt


class DebugController(QObject):
    """Steuert eine Debug-Sitzung ueber einen `dhrt debug`-Subprozess.
    Lebt im UI-Thread; ein Reader-Thread marshalt Events als Qt-Signale."""

    paused = Signal(int, object)   # editor_line, frames {locals,globals,depth}
    finished = Signal(str)         # "fertig" / "abgebrochen" / "fehler"
    output = Signal(str)           # PRINT-Ausgabe
    failed = Signal(str, int)      # Fehlermeldung, editor_line (-1 unbekannt)

    def __init__(self, parent=None):
        super().__init__(parent)
        self._breakpoints: set[int] = set()
        self._conditions: dict[int, str] = {}
        self._proc: subprocess.Popen | None = None
        self._reader: threading.Thread | None = None
        self._tmp: str | None = None
        self._mode = "idle"            # idle | running | paused
        self._started = False          # erstes (Initial-)paused gesehen?
        self._got_terminal = False     # finished/error-Event gesehen?

    # ----------------------------------------------------- Status
    def is_active(self) -> bool:
        return self._mode != "idle"

    def is_paused(self) -> bool:
        return self._mode == "paused"

    def set_breakpoints(self, lines, conditions=None) -> None:
        self._breakpoints = set(int(x) for x in lines)
        if conditions:
            self._conditions = {int(ln): str(c) for ln, c in conditions.items()
                                if int(ln) in self._breakpoints and c}
        else:
            self._conditions = {}
        # Laufende Sitzung: Breakpoints live aktualisieren.
        if self._proc is not None and self._mode != "idle":
            self._send_breakpoints()

    # ----------------------------------------------------- Start
    def start(self, source: str, base_path) -> bool:
        """Startet die Debug-Sitzung (dhrt-Subprozess). Liefert True, wenn der
        Prozess gestartet wurde; Parse-/Compile-Fehler kommen asynchron via
        `failed`. False nur, wenn dhrt fehlt/nicht startbar."""
        dhrt = _find_dhrt()
        if dhrt is None:
            self.failed.emit(
                "Debugger benoetigt die native Runtime dhrt "
                "(bauen: python rust/build_runtime.py).", -1)
            return False
        base = Path(base_path)
        tmp_dir = str(base) if base.is_dir() else None
        fd, tmp = tempfile.mkstemp(suffix=".gb", dir=tmp_dir)
        os.close(fd)
        self._tmp = tmp
        try:
            from .dhrt_locate import dhrt_spawn_semaphore
            Path(tmp).write_text(source, encoding="utf-8")
            # Semaphor rund um die Prozess-ERSTELLUNG: schuetzt gegen
            # gleichzeitig startende `dhrt`-Subprozesse aus anderen Editor-
            # Threads (siehe dhrt_locate.dhrt_spawn_semaphore-Kommentar fuer
            # den verifizierten Crash).
            with dhrt_spawn_semaphore:
                self._proc = subprocess.Popen(
                    [str(dhrt), "debug", tmp],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE, text=True,
                    cwd=str(base) if base.is_dir() else None, bufsize=1)
        except Exception as exc:  # noqa: BLE001
            self._cleanup_tmp()
            self.failed.emit(f"dhrt-Start: {exc}", -1)
            return False
        self._mode = "running"
        self._started = False
        self._got_terminal = False
        self._reader = threading.Thread(target=self._read_loop, daemon=True)
        self._reader.start()
        return True

    def _safe_emit(self, signal, *args) -> None:
        """Emit von Signalen, die auch vom Reader-Thread (`_read_loop`) aus
        aufgerufen werden. `self` haengt am Editor-Fenster (Qt-Parent) --
        wird das Fenster zerstoert, waehrend der Reader-Thread noch laeuft
        (z.B. ein Testfenster, das nie geschlossen wird), ist das C++-Objekt
        bereits weg; `.emit()` darauf ist Use-after-free (siehe error_check.py)."""
        if shiboken6.isValid(self):
            signal.emit(*args)

    # ----------------------------------------------------- Protokoll
    def _send(self, obj: dict) -> None:
        if self._proc is not None and self._proc.stdin is not None:
            try:
                self._proc.stdin.write(json.dumps(obj) + "\n")
                self._proc.stdin.flush()
            except (OSError, ValueError) as exc:
                # Frueher hier still verschluckt: stirbt dhrt zwischen zwei
                # Kommandos, wurde z.B. "Continue" zu einem No-Op ohne jede
                # Rueckmeldung. Nur melden, wenn wir nicht ohnehin schon ein
                # finished/error-Event gesehen haben (dann ist das Ende
                # bereits erklaert, keine zusaetzliche Fehlermeldung noetig).
                if not self._got_terminal:
                    self._mode = "idle"
                    self._safe_emit(self.failed, f"Debugger-Verbindung verloren: {exc}", -1)

    def _send_breakpoints(self) -> None:
        self._send({
            "cmd": "set-breakpoints",
            "lines": sorted(self._breakpoints),
            "conditions": {str(ln): c for ln, c in self._conditions.items()},
        })

    def _read_loop(self) -> None:
        proc = self._proc
        assert proc is not None and proc.stdout is not None
        for line in proc.stdout:
            line = line.strip()
            if not line:
                continue
            try:
                ev = json.loads(line)
            except json.JSONDecodeError:
                continue
            self._handle_event(ev)
        proc.wait()
        # stdout zu, aber kein finished/error gesehen -> Compile-/Parse-Fehler
        # (dhrt debug bricht vor dem Lauf ab und schreibt nur stderr).
        if not self._got_terminal:
            err = ""
            stderr_read_failed = False
            try:
                err = (proc.stderr.read() or "").strip() if proc.stderr else ""
            except (OSError, ValueError):
                # Vorher wurde ein Lesefehler hier zu "fertig" verwaschen --
                # ein echter Absturz sah dann wie ein Erfolg aus. Jetzt als
                # Fehler melden statt stillschweigend auf "fertig" zu fallen.
                stderr_read_failed = True
            self._mode = "idle"
            if err:
                self._safe_emit(self.failed, err, self._err_line(err))
                self._safe_emit(self.finished, "fehler")
            elif stderr_read_failed:
                self._safe_emit(
                    self.failed,
                    "dhrt-Prozess beendet, stderr nicht lesbar (moeglicher Absturz)", -1)
                self._safe_emit(self.finished, "fehler")
            else:
                self._safe_emit(self.finished, "fertig")
        self._cleanup_tmp()

    def _handle_event(self, ev: dict) -> None:
        kind = ev.get("event")
        if kind == "output":
            self._safe_emit(self.output, ev.get("text", ""))
        elif kind == "paused":
            if not self._started:
                # Initial-Pause an Zeile 1: Breakpoints setzen; mit Breakpoints
                # bis zum ersten BP weiterlaufen (nicht an Zeile 1 halten),
                # ohne: an Zeile 1 anhalten ("Start & Halt").
                self._started = True
                self._send_breakpoints()
                if self._breakpoints:
                    self._mode = "running"
                    self._send({"cmd": "continue"})
                    return
            self._mode = "paused"
            self._safe_emit(self.paused, int(ev.get("line", -1)), self._frames(ev))
        elif kind == "finished":
            self._got_terminal = True
            self._mode = "idle"
            reason = {"done": "fertig", "stopped": "abgebrochen"}.get(
                ev.get("reason"), "fertig")
            self._safe_emit(self.finished, reason)
        elif kind == "error":
            self._got_terminal = True
            self._mode = "idle"
            self._safe_emit(self.failed, str(ev.get("message", "")),
                            int(ev.get("line", -1) or -1))
            self._safe_emit(self.finished, "fehler")
        # eval-result/eval-error: vom Editor aktuell ungenutzt.

    @staticmethod
    def _frames(ev: dict) -> dict:
        def conv(arr):
            return [(d.get("name", ""), d.get("type", "?"), str(d.get("value", "")))
                    for d in (arr or [])]
        return {"locals": conv(ev.get("locals")),
                "globals": conv(ev.get("globals")),
                "depth": int(ev.get("depth", 0))}

    @staticmethod
    def _err_line(err: str) -> int:
        m = re.search(r":(\d+):", err)
        return int(m.group(1)) if m else -1

    def _cleanup_tmp(self) -> None:
        if self._tmp:
            try:
                os.unlink(self._tmp)
            except OSError:
                pass
            self._tmp = None

    # ----------------------------------------------------- Steuerung (UI)
    def cont(self) -> None:
        self._mode = "running"
        self._send({"cmd": "continue"})

    def step_over(self) -> None:
        self._mode = "running"
        self._send({"cmd": "step-over"})

    def step_into(self) -> None:
        self._mode = "running"
        self._send({"cmd": "step-into"})

    def step_out(self) -> None:
        self._mode = "running"
        self._send({"cmd": "step-out"})

    def stop(self) -> None:
        """Sendet das Stop-Kommando; reagiert dhrt binnen kurzer Frist nicht
        (haengt/abgestuerzt), wird der Prozess hart beendet. Frueher gab es
        hier gar keinen Fallback (anders als profiler.py/output_console.py),
        wodurch Subprozess + Reader-Thread bei einem haengenden dhrt
        unbegrenzt weiterliefen."""
        self._send({"cmd": "stop"})
        proc = self._proc
        if proc is not None:
            threading.Thread(target=self._stop_watchdog, args=(proc,), daemon=True).start()

    @staticmethod
    def _stop_watchdog(proc: subprocess.Popen) -> None:
        # Gestaffelt wie profiler.run_profile: erst Zeit zum sauberen Beenden
        # geben, dann terminate(), zuletzt kill() -- nutzt proc.poll() statt
        # proc.wait(), damit es sich nicht mit dem proc.wait() im Reader-
        # Thread (_read_loop) ueberschneidet.
        for _ in range(20):                       # ~2s in 0.1s-Schritten
            if proc.poll() is not None:
                return
            time.sleep(0.1)
        try:
            proc.terminate()
        except OSError:
            pass
        for _ in range(10):                       # ~1s
            if proc.poll() is not None:
                return
            time.sleep(0.1)
        try:
            proc.kill()
        except OSError:
            pass
