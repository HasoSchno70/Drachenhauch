"""Profiler fuer Drachenhauch (Editor) -- ueber die native Runtime `dhrt profile`.

dhrt fuehrt das Programm instrumentiert aus (per-Zeile Count+Zeit) und liefert
das Ergebnis als JSON; hier nur noch Aggregation pro Editor-Zeile und auf
Funktionen/Subs (via `symbols.scan_scopes`). Der Qt-Wrapper (`Profiler`) laeuft
in einem Worker-Thread und kann den Subprozess via Stop abbrechen.

Am besten fuer Konsolen-/Logik-Programme. **Naeherung:** Die Zeit pro Zeile
enthaelt die in aufgerufenem Code (Built-ins/Funktionen) verbrachte Zeit, die
der aufrufenden Zeile zugeschlagen wird -- aussagekraeftig fuer die *relative*
Hotpath-Einordnung, nicht als absolute Profiling-Zeit. Zeilen beziehen sich auf
die Quelle (bei `IMPORT "datei.dh"`-Inlining koennen sie verschoben sein).
"""
from __future__ import annotations

import threading
from dataclasses import dataclass, field
from pathlib import Path

import shiboken6
from PySide6.QtCore import QObject, Signal

# Review-Fund: siehe error_check.py -- geteilter Alias statt vierfach
# duplizierter Wrapper-Funktion, bleibt fuer bestehende Tests patchbar.
from .dhrt_locate import find_dhrt as _find_dhrt


@dataclass
class LineStat:
    line: int
    count: int
    time: float       # Sekunden
    text: str = ""


@dataclass
class FuncStat:
    name: str
    kind: str
    line: int
    count: int        # Aufrufe der Eroeffnungszeile (~ Aufrufzahl)
    time: float


@dataclass
class ProfileResult:
    output: str = ""
    total_time: float = 0.0
    lines: list = field(default_factory=list)   # list[LineStat], zeit-absteigend
    funcs: list = field(default_factory=list)    # list[FuncStat], zeit-absteigend
    stopped: bool = False


def _extract_profile_json(out: str) -> dict:
    """Den JSON-Blob aus dhrt's stdout herausziehen -- tolerant gegen
    raylib-Logging.

    `dhrt profile` gibt das Ergebnis als EINE JSON-Zeile via `println!` aus, aber
    raylib schreibt seine `WARNING`-TraceLogs (z.B. bei `MESH_SPHERE`:
    "WARNING: MESH: vertexCount ...") ebenfalls auf stdout -- und beim
    Fenster-Shutdown (Drop nach dem println) koennen weitere Zeilen folgen. Ein
    naives `json.loads(out)` scheitert dann an den Nicht-JSON-Zeilen und die
    Auswertung bleibt leer. Programm-Output (PRINT) liegt im `output`-Feld, NICHT
    auf rohem stdout -- der einzige `{...}`-JSON ist daher unser Blob.
    """
    import json
    for line in out.splitlines():
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return {}


def run_profile(source: str, base_path, should_stop=None) -> ProfileResult:
    """Profiliert `source` ueber die native Runtime (`dhrt profile`).

    dhrt fuehrt das Programm instrumentiert aus und liefert pro Zeile Count+Zeit
    als JSON; hier nur noch Aggregation pro Editor-Zeile + Scope. `should_stop`
    bricht den Subprozess ab (fuer Endlos-Loops). Zeilen beziehen sich auf die
    Quelle; bei `IMPORT "datei.dh"` (Inlining) koennen sie verschoben sein.
    """
    import os
    import subprocess
    import tempfile
    from . import symbols as dh_symbols
    from ..errors import DrachenhauchError

    dhrt = _find_dhrt()
    if dhrt is None:
        raise DrachenhauchError(
            "Profiler benoetigt die native Runtime dhrt "
            "(bauen: python rust/build_runtime.py).")

    base = Path(base_path)
    tmp_dir = str(base) if base.is_dir() else None
    fd, tmp = tempfile.mkstemp(suffix=".dh", dir=tmp_dir)
    os.close(fd)
    result = ProfileResult()
    out = ""
    proc = None
    try:
        Path(tmp).write_text(source, encoding="utf-8")
        # `--stoppable`: dhrt liest stdin -> wir koennen einen Endlos-Loop
        # (Grafik-Render-Loop, WHILE TRUE) per "stop"-Zeile SAUBER abbrechen,
        # sodass dhrt die bis dahin gesammelten Profile-Daten noch ausgibt
        # (ein harter terminate() wuerde den finalen JSON-println verschlucken).
        from .dhrt_locate import NO_WINDOW, dhrt_spawn_semaphore
        # Semaphor rund um die Prozess-ERSTELLUNG: `subprocess.Popen` laeuft
        # hier in einem Hintergrund-Thread (`Profiler._run`); auf Windows ist
        # die Handle-Vererbung in `subprocess.Popen.__init__` nicht dafuer
        # ausgelegt, aus mehreren Threads gleichzeitig aufgerufen zu werden
        # (siehe dhrt_locate.dhrt_spawn_semaphore-Kommentar fuer den
        # verifizierten Crash).
        with dhrt_spawn_semaphore:
            proc = subprocess.Popen(
                [str(dhrt), "profile", "--stoppable", tmp],
                stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                stdin=subprocess.PIPE, text=True, creationflags=NO_WINDOW)

        # stdout in einem Thread leeren: der JSON-Blob kommt am Ende in einem
        # Rutsch, waehrend des Laufs schreibt dhrt (fast) nichts -> der Pipe-
        # Puffer kann nicht blockieren und wir behalten stdin fuer das Stop-
        # Signal in der Hand (communicate() wuerde stdin sofort schliessen ->
        # dhrt saehe sofort EOF und braeche gleich am Anfang ab).
        chunks: list[str] = []

        def _drain():
            try:
                chunks.append(proc.stdout.read())
            except (OSError, ValueError):
                pass

        drain = threading.Thread(target=_drain, daemon=True)
        drain.start()

        requested_stop = False
        while True:
            try:
                proc.wait(timeout=0.1)
                break                                   # Programm regulaer fertig
            except subprocess.TimeoutExpired:
                if not requested_stop and should_stop is not None and should_stop():
                    requested_stop = True
                    result.stopped = True
                    try:
                        proc.stdin.write("stop\n")      # sauberer Abbruch
                        proc.stdin.flush()
                    except (OSError, ValueError):
                        pass
                    try:
                        proc.wait(timeout=5)            # JSON noch ausgeben lassen
                    except subprocess.TimeoutExpired:
                        proc.terminate()               # haengt doch -> hart beenden
                        try:
                            proc.wait(timeout=2)
                        except subprocess.TimeoutExpired:
                            proc.kill()
                    break
        drain.join(timeout=5)
        out = chunks[0] if chunks else ""
    finally:
        if proc is not None:
            for stream in (proc.stdin, proc.stdout):
                try:
                    if stream is not None:
                        stream.close()
                except OSError:
                    pass
        try:
            os.unlink(tmp)
        except OSError:
            pass

    data = _extract_profile_json(out)
    result.output = data.get("output", "")
    result.total_time = float(data.get("total_time", 0.0))
    result.stopped = result.stopped or bool(data.get("stopped", False))

    counts: dict[int, int] = {}
    times: dict[int, float] = {}
    for entry in data.get("lines", []):
        ln = int(entry.get("line", 0))
        if ln > 0:
            counts[ln] = counts.get(ln, 0) + int(entry.get("count", 0))
            times[ln] = times.get(ln, 0.0) + float(entry.get("time", 0.0))

    # Laufzeitfehler (dhrt): als DrachenhauchError mit Zeile melden (wie bisher).
    if "error" in data:
        raise DrachenhauchError(str(data["error"]), line=int(data.get("error_line", -1) or -1))

    # Zeilen-Stats + Quelltext.
    src_lines = source.split("\n")
    line_stats = []
    for ln in sorted(set(counts) | set(times)):
        text = src_lines[ln - 1].strip() if 1 <= ln <= len(src_lines) else ""
        line_stats.append(LineStat(ln, counts.get(ln, 0),
                                   times.get(ln, 0.0), text))
    line_stats.sort(key=lambda s: s.time, reverse=True)
    result.lines = line_stats

    # Funktions-Aggregation ueber die Scope-Bereiche.
    scopes = [sc for sc in dh_symbols.scan_scopes(source)
              if sc.kind in ("sub", "function")]
    func_stats = []
    for sc in scopes:
        c = sum(counts.get(ln, 0) for ln in range(sc.line, sc.end + 1))
        t = sum(times.get(ln, 0.0) for ln in range(sc.line, sc.end + 1))
        if c or t:
            calls = 0
            for ln in range(sc.line, sc.end + 1):
                if counts.get(ln, 0) > 0:
                    calls = counts[ln]
                    break
            func_stats.append(FuncStat(sc.name, sc.kind, sc.line, calls, t))
    func_stats.sort(key=lambda s: s.time, reverse=True)
    result.funcs = func_stats
    return result


class Profiler(QObject):
    """Qt-Wrapper: fuehrt `run_profile` in einem Worker-Thread aus."""

    finished = Signal(object)        # ProfileResult
    failed = Signal(str, int)        # message, editor-line (-1 unbekannt)

    def __init__(self, parent=None):
        super().__init__(parent)
        self._thread: threading.Thread | None = None
        self._stop = False

    def is_running(self) -> bool:
        return self._thread is not None and self._thread.is_alive()

    def start(self, source: str, base_path) -> None:
        self._stop = False
        self._thread = threading.Thread(
            target=self._run, args=(source, base_path), daemon=True)
        self._thread.start()

    def stop(self) -> None:
        self._stop = True

    def _run(self, source: str, base_path) -> None:
        from ..errors import DrachenhauchError
        try:
            result = run_profile(source, base_path, should_stop=lambda: self._stop)
        except DrachenhauchError as exc:
            if shiboken6.isValid(self):
                self.failed.emit(str(exc), getattr(exc, "line", -1) or -1)
            return
        except Exception as exc:  # noqa: BLE001
            if shiboken6.isValid(self):
                self.failed.emit(f"{type(exc).__name__}: {exc}", -1)
            return
        # `self` (Qt-Parent = das Fenster, das den Profiler besitzt) kann
        # inzwischen zerstoert worden sein (siehe error_check.py-Kommentar) --
        # ein `.emit()` auf einem geloeschten C++-Objekt ist Use-after-free.
        if shiboken6.isValid(self):
            self.finished.emit(result)
