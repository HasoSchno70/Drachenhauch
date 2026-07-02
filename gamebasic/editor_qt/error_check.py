"""Async-Diagnostik fuer Live-Error-Marker.

Diagnostik laeuft ueber die native Runtime: `gbrt --check datei.gb` liefert die
gefundenen Probleme (preprocess/lex/parse/compile) als JSON mit Zeile. Damit
haengt der Editor nicht mehr am Python-Compiler (Stufe B). Faellt gbrt (nicht
gebaut), greift ein Lexer/Parser-Fallback (nur Syntax) -- der Editor bleibt
nutzbar, ohne den Python-Compiler zu importieren.

Alles in einem `threading.Thread` (Daemon).
Ein Generation-Counter verwirft veraltete Resultate, sodass bei
schnellem Tippen nur das neueste Ergebnis das Signal feuert. Bei
typischen Datei-Groessen (<100 KB) dauert ein Check ~2ms; der Thread
ist also fast immer schnell fertig. Bei groesseren Files vermeiden
wir UI-Hangs.

Implementierungs-Hinweis: wir benutzen `threading.Thread` statt
`QThread`, weil ein frueherer Versuch mit QThread auf manchen Pfaden
silent-hangs erzeugt hat (vermutlich Reparenting-Issues mit dem
Worker beim moveToThread). Cross-Thread-Signal-Emission funktioniert
in Qt6/PySide6 zuverlaessig auch von Python-Native-Threads -- Qt
queue't die Slot-Calls automatisch in die Empfaenger-Event-Loop.
"""
from __future__ import annotations

import threading
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from PySide6.QtCore import QObject, Signal


@dataclass
class ParseProblem:
    line: int
    message: str
    # Severity: "error" (default) oder "warning". gbrt --check liefert beide
    # (z.B. Hardware-Modul-IMPORT ohne Build = Warning); der Editor rendert
    # Warnungen mit gelber statt roter Wellenlinie.
    severity: str = "error"
    # Phase, in der das Problem erkannt wurde. Hilft bei Debugging und
    # erlaubt der UI, Compile-Errors anders zu rendern als Parse-Errors
    # (z.B. anderes Icon).
    phase: str = "parse"


def _find_gbrt():
    """Pfad zur gbrt-Binary (frozen-aware: installiert neben GameBasic.exe,
    sonst Dev-Baum) oder None."""
    from .gbrt_locate import find_gbrt
    return find_gbrt()


def _check_source(source: str, base_path: Path | None,
                   checker: "LiveErrorChecker | None" = None) -> list[ParseProblem]:
    """Liefert ALLE Diagnostik-Probleme (leer = sauber). Bevorzugt `gbrt --check`
    (volle preprocess/lex/parse/compile-Diagnostik mit Zeile, inkl. Warnungen);
    faellt gbrt, nur Syntax (Lexer/Parser) ohne den Python-Compiler -- dieser
    Fallback findet nur das erste Syntaxproblem. `checker` (optional) registriert
    den laufenden gbrt-Subprozess, damit ein neuerer `check()`-Aufruf ihn
    abbrechen kann (siehe LiveErrorChecker.check)."""
    gbrt = _find_gbrt()
    if gbrt is not None:
        return _check_via_gbrt(source, base_path, gbrt, checker)
    one = _check_syntax_only(source, base_path)
    return [one] if one is not None else []


_DIAG_FAILED_MSG = "Diagnose fehlgeschlagen -- moeglicherweise veraltete Fehleranzeige"


def _check_via_gbrt(source: str, base_path, gbrt,
                     checker: "LiveErrorChecker | None" = None) -> list[ParseProblem]:
    """`gbrt --check` auf einer temporaeren .gb-Datei. JSON-Diagnose -> Liste
    aller ParseProblems (Errors UND Warnungen). Zeilen sind quell-relativ (bei
    `IMPORT "datei.gb"`-Inlining moeglw. verschoben).

    Frueher gab jeder gbrt-Fehler (Crash/Timeout/kaputtes JSON) eine leere
    Liste zurueck -- der Editor zeigte dann "keine Fehler", obwohl der
    Checker selbst kaputt war (irrefuehrender als gar keine Diagnose). Jetzt
    wird ein einzelnes Warning-ParseProblem geliefert, das das transparent
    macht, statt es zu verschweigen."""
    import json
    import os
    import subprocess
    import tempfile
    base = base_path or Path.cwd()
    tmp_dir = str(base) if Path(base).is_dir() else None
    fd, tmp = tempfile.mkstemp(suffix=".gb", dir=tmp_dir)
    os.close(fd)
    proc = None
    try:
        Path(tmp).write_text(source, encoding="utf-8")
        proc = subprocess.Popen(
            [str(gbrt), "--check", tmp],
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
        if checker is not None:
            checker._set_active_proc(proc)
        stdout, _ = proc.communicate(timeout=15)
        diags = json.loads(stdout or "[]")
    except subprocess.TimeoutExpired:
        return [ParseProblem(line=1, message="Diagnose-Timeout (gbrt --check antwortete nicht)",
                             severity="warning", phase="diagnostic")]
    except (OSError, ValueError):
        # ValueError deckt json.JSONDecodeError ab (kaputtes/leeres stdout,
        # z.B. bei einem gbrt-Absturz oder abgebrochenem Prozess).
        return [ParseProblem(line=1, message=_DIAG_FAILED_MSG,
                             severity="warning", phase="diagnostic")]
    finally:
        if checker is not None:
            checker._set_active_proc(None)
        try:
            os.unlink(tmp)
        except OSError:
            pass
    out: list[ParseProblem] = []
    for d in diags:
        out.append(ParseProblem(
            line=int(d.get("line", 0)) or 1,
            message=str(d.get("message", "")),
            severity=str(d.get("severity", "error")),
            phase=str(d.get("phase", "compile"))))
    return out


def _check_syntax_only(source: str, base_path: Path | None) -> Optional[ParseProblem]:
    """Fallback ohne gbrt: Preprocess + Lex + Parse (nur Syntax, kein Compiler)."""
    try:
        from ..lexer import Lexer
        from ..parser import Parser
        from ..preprocess import process as _preprocess
        from ..errors import LexerError, ParseError
    except Exception as exc:
        return ParseProblem(line=1, message=f"Import-Fehler: {exc}")

    try:
        merged, origins = _preprocess(
            source, base_path or Path.cwd(), file_label="<editor>")
    except LexerError as exc:
        return ParseProblem(line=getattr(exc, "line", 1) or 1,
                            message=f"IMPORT: {exc}", phase="preprocess")
    except Exception as exc:
        return ParseProblem(line=1,
                            message=f"Preprocess: {type(exc).__name__}: {exc}",
                            phase="preprocess")
    try:
        Parser(Lexer(merged).tokenize()).parse()
    except (LexerError, ParseError) as exc:
        merged_line = getattr(exc, "line", 1) or 1
        line, msg = _map_back(origins, merged_line, str(exc))
        return ParseProblem(line=line, message=msg, phase="parse")
    except Exception as exc:
        return ParseProblem(line=1,
                            message=f"{type(exc).__name__}: {exc}", phase="parse")
    return None


def _map_back(origins, merged_line: int, message: str) -> tuple[int, str]:
    """Konvertiert merged-Line -> User-Buffer-Line ueber `origins`.

    - Fehler in `<editor>` (= unser Buffer): liefere die Original-Zeile.
    - Fehler in einem IMPORT-File: bleibe bei Zeile 1, prefixe das
      File ins Message-Feld.
    """
    if 1 <= merged_line < len(origins) and origins[merged_line] is not None:
        file_label, orig_line = origins[merged_line]
        if file_label == "<editor>":
            return orig_line, message
        return 1, f"in {file_label}:{orig_line} -> {message}"
    return merged_line, message


class LiveErrorChecker(QObject):
    """Async-Live-Check mit Generation-Counter.

    Aufrufer ruft `check(source, base_path)` jedes Mal mit dem aktuellen
    Buffer. Wir spawnen einen Worker-Thread; bevor er das Resultat
    emittiert, prueft er, ob inzwischen eine neuere Anfrage gekommen ist
    (Generation-Counter) -- in dem Fall wird das Ergebnis verworfen.

    `problem_changed` feuert exklusiv im Main-Thread (Qt's Cross-Thread-
    Signaling marshalt automatisch in die Receiver-Event-Loop).
    """

    problem_changed = Signal(object)    # ParseProblem | None (erstes Problem)
    problems_changed = Signal(object)   # list[ParseProblem] (alle, evtl. leer)

    def __init__(self, parent: QObject | None = None):
        super().__init__(parent)
        self._gen = 0
        self._lock = threading.Lock()
        self._active_proc = None   # laufender `gbrt --check`-Subprozess (falls einer)

    def check(self, source: str, base_path: Path | None = None) -> None:
        with self._lock:
            self._gen += 1
            gen = self._gen
            proc = self._active_proc
        # Einen noch laufenden Check-Subprozess abbrechen -- frueher wurde
        # bei schnellem Tippen nur das ERGEBNIS verworfen (Generation-
        # Counter), der `gbrt --check`-Prozess lief aber unbeaufsichtigt
        # bis zum Ende weiter. Bei sehr schnellem Tippen konnten sich so
        # mehrere Prozesse gleichzeitig ansammeln.
        if proc is not None:
            try:
                proc.terminate()
            except OSError:
                pass
        t = threading.Thread(
            target=self._run, args=(gen, source, base_path),
            name="LiveErrorCheck", daemon=True,
        )
        t.start()

    def _set_active_proc(self, proc) -> None:
        with self._lock:
            self._active_proc = proc

    def _run(self, gen: int, source: str, base_path: Path | None) -> None:
        problems = _check_source(source, base_path, self)
        # Ist dieses Resultat noch das aktuelle? Sonst verwerfen.
        with self._lock:
            if gen != self._gen:
                return
        # Erst die Voll-Liste (Editor-Underlines + Problems-Panel), dann das
        # erste Problem (Statusbar) -- so ist `current_error()` schon frisch,
        # wenn der Statusbar-Slot laeuft.
        self.problems_changed.emit(problems)
        self.problem_changed.emit(problems[0] if problems else None)
