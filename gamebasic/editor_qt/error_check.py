"""Async-Parser-Check fuer Live-Error-Marker.

Pipeline: Preprocess -> Lex -> Parse -> Compile, alles in einem
`threading.Thread` (Daemon). Der Compile-Pass fuegt den semantischen
Check (Scope, Type, undefinierte Variable, doppelte Deklaration) hinzu,
den Lex+Parse alleine nicht haetten -- ein Quick-Win fuer den Editor.

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
    # Severity: "error" (default) oder "warning". Aktuell liefert die
    # Pipeline nur Errors; Warnings sind ein nahe liegender naechster
    # Schritt (unbenutzte Variable, geschattete Deklaration).
    severity: str = "error"
    # Phase, in der das Problem erkannt wurde. Hilft bei Debugging und
    # erlaubt der UI, Compile-Errors anders zu rendern als Parse-Errors
    # (z.B. anderes Icon).
    phase: str = "parse"


def _check_source(source: str, base_path: Path | None) -> Optional[ParseProblem]:
    """Laeuft die volle Editor-Diagnostik-Pipeline und liefert das erste
    gefundene Problem zurueck (oder None, wenn alles ok ist).

    Phasen, in Reihenfolge, kurzes Stop beim ersten Fehler:
      1. Preprocess (IMPORT-Aufloesung)
      2. Lex
      3. Parse
      4. Compile (semantischer Check via Bytecode-Compiler)
    """
    try:
        from ..lexer import Lexer
        from ..parser import Parser
        from ..compiler import Compiler, CompileError
        from ..preprocess import process as _preprocess
        from ..errors import LexerError, ParseError
    except Exception as exc:
        return ParseProblem(line=1, message=f"Import-Fehler: {exc}")

    # 1) Preprocessor -- IMPORTs werden aufgeloest, dabei kann selbst
    #    schon ein LexerError fliegen (z.B. fehlende Datei).
    try:
        merged, origins = _preprocess(
            source,
            base_path or Path.cwd(),
            file_label="<editor>",
        )
    except LexerError as exc:
        return ParseProblem(line=getattr(exc, "line", 1) or 1,
                            message=f"IMPORT: {exc}",
                            phase="preprocess")
    except Exception as exc:
        return ParseProblem(line=1,
                            message=f"Preprocess: {type(exc).__name__}: {exc}",
                            phase="preprocess")

    # 2) Lex + Parse.
    try:
        tokens = Lexer(merged).tokenize()
        ast = Parser(tokens).parse()
    except (LexerError, ParseError) as exc:
        merged_line = getattr(exc, "line", 1) or 1
        line, msg = _map_back(origins, merged_line, str(exc))
        return ParseProblem(line=line, message=msg, phase="parse")
    except Exception as exc:
        return ParseProblem(line=1,
                            message=f"{type(exc).__name__}: {exc}",
                            phase="parse")

    # 3) Compile -- semantischer Check. Faengt undeclared variables,
    #    Type-Mismatch, doppelte Deklarationen, falsche AS-Annotations etc.
    #    Compile-Errors haben in der Regel keine line-Info; im Editor
    #    landet der Marker dann auf Zeile 1, was OK ist (User sieht das
    #    Problem im Status, nicht zwingend im Gutter).
    try:
        Compiler().compile(ast)
    except CompileError as exc:
        merged_line = getattr(exc, "line", 0) or 1
        line, msg = _map_back(origins, merged_line, str(exc))
        return ParseProblem(line=line, message=msg, phase="compile")
    except Exception as exc:
        # Defensiv: ein Bug im Compiler soll den Editor nicht crashen.
        return ParseProblem(line=1,
                            message=f"Compile {type(exc).__name__}: {exc}",
                            phase="compile")
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

    problem_changed = Signal(object)   # ParseProblem | None

    def __init__(self, parent: QObject | None = None):
        super().__init__(parent)
        self._gen = 0
        self._lock = threading.Lock()

    def check(self, source: str, base_path: Path | None = None) -> None:
        with self._lock:
            self._gen += 1
            gen = self._gen
        t = threading.Thread(
            target=self._run, args=(gen, source, base_path),
            name="LiveErrorCheck", daemon=True,
        )
        t.start()

    def _run(self, gen: int, source: str, base_path: Path | None) -> None:
        problem = _check_source(source, base_path)
        # Ist dieses Resultat noch das aktuelle? Sonst verwerfen.
        with self._lock:
            if gen != self._gen:
                return
        self.problem_changed.emit(problem)
