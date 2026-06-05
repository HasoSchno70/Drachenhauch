"""Tree-Walking-Profiler fuer GameBasic (Editor).

Nutzt denselben Hook-Mechanismus wie der Debugger (`Interpreter._debug_hook`,
wird vor jedem Statement aufgerufen), misst aber statt zu pausieren pro Zeile
Trefferzahl und Zeit. Aggregiert auf Editor-Zeilen (via Preprocessor-`origins`)
und auf Funktionen/Subs (via `symbols.scan_scopes`).

Nur Tree-Walker -- am besten fuer Konsolen-/Logik-Programme. Grafik-Programme
mit Endlos-Loop laufen nicht durch (wie Bench/Debugger); die UI bietet Stop.

**Naeherung:** Die gemessene Zeit pro Zeile enthaelt die in aufgerufenem Code
(inkl. Built-ins/IMPORT) verbrachte Zeit, die der aufrufenden Editor-Zeile
zugeschlagen wird, sowie den Hook-Overhead selbst. Fuer die *relative*
Hotpath-Einordnung ist das aussagekraeftig, nicht als absolute Profiling-Zeit.
"""
from __future__ import annotations

import contextlib
import io
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path

from PySide6.QtCore import QObject, Signal

_EDITOR_LABEL = "<editor>"


class _ProfileStop(Exception):
    """Intern -- bricht den Worker beim Stop sauber ab."""


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


def run_profile(source: str, base_path, should_stop=None) -> ProfileResult:
    """Fuehrt `source` im Tree-Walker mit Profiling-Hook aus.

    `should_stop` ist ein optionales Callable -> bool, mit dem ein UI-Thread
    den Lauf abbrechen kann. stdin liefert EOF (INPUT haengt nicht)."""
    from ..preprocess import process
    from ..lexer import Lexer
    from ..parser import Parser
    from ..interpreter import Interpreter
    from . import symbols as gb_symbols

    merged, origins = process(source, Path(base_path), file_label=_EDITOR_LABEL)
    ast = Parser(Lexer(merged).tokenize()).parse()

    def to_editor(merged_line: int) -> int:
        if origins and 0 < merged_line < len(origins):
            o = origins[merged_line]
            if o and o[0] == _EDITOR_LABEL:
                return o[1]
            return -1
        return merged_line

    counts: dict[int, int] = {}
    times: dict[int, float] = {}
    state = {"last_el": None, "last_t": None}
    perf = time.perf_counter

    def hook(line, _interp):
        if should_stop is not None and should_stop():
            raise _ProfileStop()
        now = perf()
        last_el = state["last_el"]
        if last_el is not None and state["last_t"] is not None:
            times[last_el] = times.get(last_el, 0.0) + (now - state["last_t"])
        state["last_t"] = now
        el = to_editor(line)
        if el > 0:
            counts[el] = counts.get(el, 0) + 1
            state["last_el"] = el

    interp = Interpreter()
    interp._debug_hook = hook

    result = ProfileResult()
    buf = io.StringIO()

    class _Eof:
        def readline(self, *a): return ""
        def read(self, *a): return ""

    import os
    import sys
    old_in = sys.stdin
    sys.stdin = _Eof()
    # Ins Datei-Verzeichnis wechseln (wie gbrun.py os.chdir(file.parent)),
    # damit relative Laufzeit-Pfade wie LOADIMAGE("assets/...") funktionieren.
    old_cwd = os.getcwd()
    try:
        os.chdir(str(base_path))
    except OSError:
        pass
    t0 = perf()
    try:
        with contextlib.redirect_stdout(buf):
            interp.run(ast)
    except _ProfileStop:
        result.stopped = True
    finally:
        sys.stdin = old_in
        os.chdir(old_cwd)
        # Letzte Zeit-Delta der zuletzt aktiven Zeile zuschlagen.
        if state["last_el"] is not None and state["last_t"] is not None:
            times[state["last_el"]] = times.get(state["last_el"], 0.0) + (
                perf() - state["last_t"])
    result.total_time = perf() - t0
    result.output = buf.getvalue()

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
            # Aufrufzahl ~ Treffer der ersten ausgefuehrten Body-Zeile (die
            # FUNCTION/SUB-Zeile selbst ist kein Laufzeit-Statement).
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
