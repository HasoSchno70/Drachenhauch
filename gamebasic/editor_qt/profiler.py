"""Profiler fuer GameBasic (Editor) -- ueber die native Runtime `gbrt profile`.

gbrt fuehrt das Programm instrumentiert aus (per-Zeile Count+Zeit) und liefert
das Ergebnis als JSON; hier nur noch Aggregation pro Editor-Zeile und auf
Funktionen/Subs (via `symbols.scan_scopes`). Der Qt-Wrapper (`Profiler`) laeuft
in einem Worker-Thread und kann den Subprozess via Stop abbrechen.

Am besten fuer Konsolen-/Logik-Programme. **Naeherung:** Die Zeit pro Zeile
enthaelt die in aufgerufenem Code (Built-ins/Funktionen) verbrachte Zeit, die
der aufrufenden Zeile zugeschlagen wird -- aussagekraeftig fuer die *relative*
Hotpath-Einordnung, nicht als absolute Profiling-Zeit. Zeilen beziehen sich auf
die Quelle (bei `IMPORT "datei.gb"`-Inlining koennen sie verschoben sein).
"""
from __future__ import annotations

import threading
from dataclasses import dataclass, field
from pathlib import Path

from PySide6.QtCore import QObject, Signal


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
    """Den JSON-Blob aus gbrt's stdout herausziehen -- tolerant gegen
    raylib-Logging.

    `gbrt profile` gibt das Ergebnis als EINE JSON-Zeile via `println!` aus, aber
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


def _find_gbrt():
    """Pfad zur gebauten gbrt-Binary (release vor debug) oder None."""
    import os
    root = Path(__file__).resolve().parents[2]
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = root / "rust" / "gb_runtime" / "target" / variant / exe
        if p.exists():
            return p
    return None


def run_profile(source: str, base_path, should_stop=None) -> ProfileResult:
    """Profiliert `source` ueber die native Runtime (`gbrt profile`).

    gbrt fuehrt das Programm instrumentiert aus und liefert pro Zeile Count+Zeit
    als JSON; hier nur noch Aggregation pro Editor-Zeile + Scope. `should_stop`
    bricht den Subprozess ab (fuer Endlos-Loops). Zeilen beziehen sich auf die
    Quelle; bei `IMPORT "datei.gb"` (Inlining) koennen sie verschoben sein.
    """
    import os
    import subprocess
    import tempfile
    from . import symbols as gb_symbols
    from ..errors import GameBasicError

    gbrt = _find_gbrt()
    if gbrt is None:
        raise GameBasicError(
            "Profiler benoetigt die native Runtime gbrt "
            "(bauen: python rust/build_runtime.py).")

    base = Path(base_path)
    tmp_dir = str(base) if base.is_dir() else None
    fd, tmp = tempfile.mkstemp(suffix=".gb", dir=tmp_dir)
    os.close(fd)
    result = ProfileResult()
    out = ""
    try:
        Path(tmp).write_text(source, encoding="utf-8")
        proc = subprocess.Popen(
            [str(gbrt), "profile", tmp],
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            stdin=subprocess.DEVNULL, text=True)
        while True:
            try:
                out, _ = proc.communicate(timeout=0.1)
                break
            except subprocess.TimeoutExpired:
                if should_stop is not None and should_stop():
                    proc.terminate()
                    try:
                        out, _ = proc.communicate(timeout=2)
                    except subprocess.TimeoutExpired:
                        proc.kill()
                    result.stopped = True
                    break
    finally:
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

    # Laufzeitfehler (gbrt): als GameBasicError mit Zeile melden (wie bisher).
    if "error" in data:
        raise GameBasicError(str(data["error"]), line=int(data.get("error_line", -1) or -1))

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
    scopes = [sc for sc in gb_symbols.scan_scopes(source)
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
        from ..errors import GameBasicError
        try:
            result = run_profile(source, base_path, should_stop=lambda: self._stop)
        except GameBasicError as exc:
            self.failed.emit(str(exc), getattr(exc, "line", -1) or -1)
            return
        except Exception as exc:  # noqa: BLE001
            self.failed.emit(f"{type(exc).__name__}: {exc}", -1)
            return
        self.finished.emit(result)
