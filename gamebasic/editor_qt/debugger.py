"""Tree-Walking-Debugger fuer GameBasic (Editor).

Fuehrt das Programm im **Tree-Walker** (Python-Interpreter) in einem Worker-
Thread aus und installiert einen Debug-Hook (`Interpreter._debug_hook`), der
vor jedem Statement aufgerufen wird. Der Hook pausiert den Worker an
Breakpoints bzw. beim Schrittweisen Ausfuehren (Step Over/Into/Out), schickt
einen Variablen-Snapshot per Qt-Signal an den UI-Thread und blockiert auf
einem Event, bis der User fortsetzt.

Nur Tree-Walker -- die native Runtime `gbrt` hat keinen Debug-Kanal. Am
besten fuer Konsolen-/Logik-Programme; bei `INPUT` liefert der Debugger EOF
(kein Hang), Grafik-Programme laufen best effort.

Zeilen-Mapping: der Preprocessor merged IMPORT-Quelldateien -- `origins`
bildet merged-Zeile -> (datei, original-Zeile) ab. Breakpoints/Highlights
beziehen sich auf die Editor-Datei (`<editor>`).
"""
from __future__ import annotations

import threading

from PySide6.QtCore import QObject, Signal


_EDITOR_LABEL = "<editor>"


class _DebugStop(Exception):
    """Intern -- bricht den Worker beim Stop sauber ab."""


class _EmitWriter:
    """Datei-artiges stdout-Surrogat: Writes aus dem **Worker-Thread** gehen
    ans Qt-Signal (Programm-Ausgabe), alle anderen (z.B. UI-Thread-Prints
    waehrend einer Pause) durch zum echten stdout -- der Redirect ist global,
    soll aber nur die Programm-Ausgabe abfangen."""

    def __init__(self, signal, passthrough, worker_ident):
        self._signal = signal
        self._passthrough = passthrough
        self._worker_ident = worker_ident

    def write(self, text):
        if not text:
            return 0
        if threading.get_ident() == self._worker_ident:
            self._signal.emit(text)
        elif self._passthrough is not None:
            self._passthrough.write(text)
        return len(text)

    def flush(self):
        if self._passthrough is not None and \
                threading.get_ident() != self._worker_ident:
            self._passthrough.flush()


class _EofReader:
    """stdin-Ersatz: INPUT bekommt EOF statt zu blockieren."""

    def readline(self, *a):
        return ""

    def read(self, *a):
        return ""


def _fmt_value(v) -> str:
    """Kurze, sichere Darstellung eines Variablenwerts fuer das Panel."""
    try:
        from ..interpreter import _Instance, _GBArray  # type: ignore
    except Exception:                                   # pragma: no cover
        _Instance = _GBArray = ()                       # type: ignore
    if isinstance(v, bool):
        return "TRUE" if v else "FALSE"
    if isinstance(v, str):
        s = v if len(v) <= 120 else v[:117] + "..."
        return '"' + s + '"'
    if v is None:
        return "NIL"
    if _Instance and isinstance(v, _Instance):
        cls = getattr(getattr(v, "cls", None), "name", "?")
        return f"<{cls}>"
    try:
        s = str(v)
    except Exception:
        s = f"<{type(v).__name__}>"
    return s if len(s) <= 200 else s[:197] + "..."


class DebugController(QObject):
    """Steuert eine Debug-Sitzung. Lebt im UI-Thread, fuehrt den Interpreter
    in einem Worker-Thread aus."""

    paused = Signal(int, object)   # editor_line (-1 = nicht in Editor-Datei), frames
    finished = Signal(str)         # Grund: "fertig" / "abgebrochen" / "fehler"
    output = Signal(str)           # PRINT-Ausgabe
    failed = Signal(str, int)      # Fehlermeldung, editor_line (-1 unbekannt)

    def __init__(self, parent=None):
        super().__init__(parent)
        self._breakpoints: set[int] = set()    # Editor-Zeilen (1-basiert)
        self._merged_bps: set[int] = set()     # merged-Zeilen
        self._origins = None
        self._mode = "idle"                    # idle | running | paused
        self._step: str | None = None          # None | over | into | out
        self._step_depth = 0
        self._paused_depth = 0
        self._resume = threading.Event()
        self._stop = False
        self._thread: threading.Thread | None = None
        self._interp = None
        self._baseline_globals: set = set()

    # ----------------------------------------------------- Status
    def is_active(self) -> bool:
        return self._mode != "idle"

    def is_paused(self) -> bool:
        return self._mode == "paused"

    def set_breakpoints(self, lines) -> None:
        self._breakpoints = set(int(x) for x in lines)
        self._recompute_merged_bps()

    # ----------------------------------------------------- Start
    def start(self, source: str, base_path) -> bool:
        """Parst die Quelle und startet die Debug-Sitzung. Liefert False bei
        einem Parse-Fehler (per `failed` gemeldet)."""
        from pathlib import Path
        from ..preprocess import process
        from ..lexer import Lexer
        from ..parser import Parser
        from ..interpreter import Interpreter
        from ..errors import GameBasicError
        try:
            merged, origins = process(source, Path(base_path), file_label=_EDITOR_LABEL)
            tokens = Lexer(merged).tokenize()
            ast = Parser(tokens).parse()
        except GameBasicError as exc:
            self.failed.emit(str(exc), getattr(exc, "line", -1) or -1)
            return False
        except Exception as exc:  # noqa: BLE001
            self.failed.emit(f"{type(exc).__name__}: {exc}", -1)
            return False

        self._origins = origins
        self._recompute_merged_bps()
        interp = Interpreter()
        interp._debug_hook = self._hook
        self._interp = interp
        # Vorregistrierte Built-in-Konstanten (BLACK, KEY_*, PI, ...) merken,
        # um sie aus der Globals-Anzeige rauszufiltern -- der User will nur
        # seine eigenen Globals sehen.
        self._baseline_globals = set(interp.global_env.vars.keys())
        self._stop = False
        self._mode = "running"
        # Mit Breakpoints: bis zum ersten BP laufen. Ohne: am ersten
        # Statement anhalten (wie "Start & Halt").
        self._step = None if self._merged_bps else "into"
        self._thread = threading.Thread(
            target=self._run, args=(ast,), daemon=True)
        self._thread.start()
        return True

    def _run(self, ast) -> None:
        import sys
        from ..errors import GameBasicError
        old_out, old_in = sys.stdout, sys.stdin
        sys.stdout = _EmitWriter(self.output, old_out, threading.get_ident())
        sys.stdin = _EofReader()
        reason = "fertig"
        try:
            self._interp.run(ast)
        except _DebugStop:
            reason = "abgebrochen"
        except GameBasicError as exc:
            reason = "fehler"
            self.failed.emit(str(exc), self._to_editor_line(getattr(exc, "line", 0)))
        except Exception as exc:  # noqa: BLE001
            reason = "fehler"
            self.failed.emit(f"{type(exc).__name__}: {exc}", -1)
        finally:
            sys.stdout, sys.stdin = old_out, old_in
            self._mode = "idle"
            self._interp = None
            self.finished.emit(reason)

    # ----------------------------------------------------- Hook (Worker)
    def _hook(self, line, interp) -> None:
        if self._stop:
            raise _DebugStop()
        depth = interp.call_depth
        pause = (
            line in self._merged_bps
            or self._step == "into"
            or (self._step == "over" and depth <= self._step_depth)
            or (self._step == "out" and depth < self._step_depth)
        )
        if not pause:
            return
        self._paused_depth = depth
        frames = self._snapshot(interp)
        editor_line = self._to_editor_line(line)
        self._mode = "paused"
        self._step = None
        self._resume.clear()
        self.paused.emit(editor_line, frames)
        self._resume.wait()
        if self._stop:
            raise _DebugStop()
        self._mode = "running"

    def _snapshot(self, interp) -> dict:
        def collect(env, skip=None):
            rows = []
            for name, slot in env.vars.items():
                if name.startswith("__"):
                    continue
                if skip is not None and name in skip:
                    continue
                rows.append((name, slot.get("type", "?"),
                             _fmt_value(slot.get("value"))))
            return rows

        glob = collect(interp.global_env, skip=self._baseline_globals)
        locs: list = []
        env = interp.env
        while env is not None and env is not interp.global_env:
            locs.extend(collect(env))
            env = env.parent
        return {"locals": locs, "globals": glob, "depth": interp.call_depth}

    # ----------------------------------------------------- Steuerung (UI)
    def cont(self) -> None:
        self._step = None
        self._resume.set()

    def step_over(self) -> None:
        self._step = "over"
        self._step_depth = self._paused_depth
        self._resume.set()

    def step_into(self) -> None:
        self._step = "into"
        self._resume.set()

    def step_out(self) -> None:
        self._step = "out"
        self._step_depth = self._paused_depth
        self._resume.set()

    def stop(self) -> None:
        self._stop = True
        self._resume.set()

    # ----------------------------------------------------- Zeilen-Mapping
    def _recompute_merged_bps(self) -> None:
        if not self._origins:
            self._merged_bps = set(self._breakpoints)
            return
        s = set()
        for m in range(1, len(self._origins)):
            o = self._origins[m]
            if o and o[0] == _EDITOR_LABEL and o[1] in self._breakpoints:
                s.add(m)
        self._merged_bps = s

    def _to_editor_line(self, merged_line: int) -> int:
        if not merged_line:
            return -1
        if self._origins and 0 < merged_line < len(self._origins):
            o = self._origins[merged_line]
            if o and o[0] == _EDITOR_LABEL:
                return o[1]
            return -1
        return merged_line
