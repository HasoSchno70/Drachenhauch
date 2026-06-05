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
    # Severity: "error" (default) oder "warning". Aktuell liefert die
    # Pipeline nur Errors; Warnings sind ein nahe liegender naechster
    # Schritt (unbenutzte Variable, geschattete Deklaration).
    severity: str = "error"
    # Phase, in der das Problem erkannt wurde. Hilft bei Debugging und
    # erlaubt der UI, Compile-Errors anders zu rendern als Parse-Errors
    # (z.B. anderes Icon).
    phase: str = "parse"


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


def _check_source(source: str, base_path: Path | None) -> Optional[ParseProblem]:
    """Liefert das erste Diagnostik-Problem (oder None). Bevorzugt `gbrt --check`
    (volle preprocess/lex/parse/compile-Diagnostik mit Zeile); faellt gbrt, nur
    Syntax (Lexer/Parser) ohne den Python-Compiler."""
    gbrt = _find_gbrt()
    if gbrt is not None:
        return _check_via_gbrt(source, base_path, gbrt)
    return _check_syntax_only(source, base_path)


def _check_via_gbrt(source: str, base_path, gbrt) -> Optional[ParseProblem]:
    """`gbrt --check` auf einer temporaeren .gb-Datei. JSON-Diagnose -> erstes
    ParseProblem. Zeilen sind quell-relativ (bei `IMPORT "datei.gb"`-Inlining
    moeglw. verschoben). Bei jedem gbrt-Fehler defensiv None (kein Editor-Crash)."""
    import json
    import os
    import subprocess
    import tempfile
    base = base_path or Path.cwd()
    tmp_dir = str(base) if Path(base).is_dir() else None
    fd, tmp = tempfile.mkstemp(suffix=".gb", dir=tmp_dir)
    os.close(fd)
    try:
        Path(tmp).write_text(source, encoding="utf-8")
        r = subprocess.run([str(gbrt), "--check", tmp],
                           capture_output=True, text=True, timeout=15)
        diags = json.loads(r.stdout or "[]")
    except Exception:
        return None
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass
    if not diags:
        return None
    d = diags[0]
    return ParseProblem(
        line=int(d.get("line", 0)) or 1,
        message=str(d.get("message", "")),
        severity=str(d.get("severity", "error")),
        phase=str(d.get("phase", "compile")))


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
