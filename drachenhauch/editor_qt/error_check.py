"""Async-Diagnostik fuer Live-Error-Marker.

Diagnostik laeuft ueber die native Runtime: `dhrt --check datei.dh` liefert die
gefundenen Probleme (preprocess/lex/parse/compile) als JSON mit Zeile. Damit
haengt der Editor nicht mehr am Python-Compiler (Stufe B). Faellt dhrt (nicht
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

WICHTIG (Use-after-free-Fix): `LiveErrorChecker` haengt am `CodeEditor`
(als Qt-Parent) -- wird der Editor waehrend ein Check-Thread noch laeuft
zerstoert (z.B. ein Test-Fenster, das nie explizit geschlossen wird und
per Python-Refcount weggeraeumt wird), ist das C++-Objekt bereits
geloescht, wenn der Hintergrund-Thread danach `.emit()` aufruft --
ein Cross-Thread-Use-after-free auf dem geloeschten QObject (fuehrte zu
sporadischen Access-Violations, sichtbar erst beim naechsten
`processEvents()` irgendwo im Prozess). Deshalb vor jedem `.emit()`
`shiboken6.isValid(self)` pruefen.
"""
from __future__ import annotations

import threading
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

import shiboken6
from PySide6.QtCore import QObject, Signal

# Review-Fund: 4 Module (error_check/debugger/profiler/output_console)
# definierten je eine eigene, fast wortgleiche `_find_dhrt()`-Wrapper-
# Funktion (inkl. dupliziertem Docstring) um dieselbe `dhrt_locate.find_dhrt`
# -- als Alias importiert bleibt der Modul-Name `_find_dhrt` fuer bestehende
# Tests patchbar (`monkeypatch.setattr(modul, "_find_dhrt", ...)`), ohne die
# Wrapper-Funktion + ihren Docstring viermal zu duplizieren.
from .dhrt_locate import find_dhrt as _find_dhrt
from . import tempdateien


@dataclass
class ParseProblem:
    line: int
    message: str
    # Severity: "error" (default) oder "warning". dhrt --check liefert beide
    # (z.B. Hardware-Modul-IMPORT ohne Build = Warning); der Editor rendert
    # Warnungen mit gelber statt roter Wellenlinie.
    severity: str = "error"
    # Phase, in der das Problem erkannt wurde. Hilft bei Debugging und
    # erlaubt der UI, Compile-Errors anders zu rendern als Parse-Errors
    # (z.B. anderes Icon).
    phase: str = "parse"


def _check_source(source: str, base_path: Path | None,
                   checker: "LiveErrorChecker | None" = None) -> list[ParseProblem]:
    """Liefert ALLE Diagnostik-Probleme (leer = sauber). Bevorzugt `dhrt --check`
    (volle preprocess/lex/parse/compile-Diagnostik mit Zeile, inkl. Warnungen);
    fehlt dhrt, gibt es
    eine Warnung mit Bau-Hinweis statt halber Diagnostik (siehe
    `_keine_diagnose`). `checker` (optional) registriert
    den laufenden dhrt-Subprozess, damit ein neuerer `check()`-Aufruf ihn
    abbrechen kann (siehe LiveErrorChecker.check)."""
    from .dhrt_locate import dhrt_spawn_semaphore
    # Das Semaphor umschliesst ABSICHTLICH auch `_find_dhrt()` (nicht nur den
    # Popen()-Call) -- dessen `Path.resolve()` ist unter Buendel-Last (viele
    # gleichzeitig feuernde Checker-Threads) selbst abgestuerzt (siehe
    # dhrt_locate.py-Kommentar).
    with dhrt_spawn_semaphore:
        dhrt = _find_dhrt()
        if dhrt is not None:
            return _check_via_dhrt(source, base_path, dhrt, checker)
    return [_keine_diagnose()]


_DIAG_FAILED_MSG = "Diagnose fehlgeschlagen -- moeglicherweise veraltete Fehleranzeige"


def _check_via_dhrt(source: str, base_path, dhrt,
                     checker: "LiveErrorChecker | None" = None) -> list[ParseProblem]:
    """`dhrt --check` auf einer temporaeren .dh-Datei. JSON-Diagnose -> Liste
    aller ParseProblems (Errors UND Warnungen). Zeilen sind quell-relativ (bei
    `IMPORT "datei.dh"`-Inlining moeglw. verschoben).

    Frueher gab jeder dhrt-Fehler (Crash/Timeout/kaputtes JSON) eine leere
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
    # Erkennbarer Name (Praefix + Prozessnummer), damit ein beim
    # Abbruch liegengebliebener Rest beim naechsten Start sicher
    # entfernt werden kann -- siehe tempdateien.py.
    fd, tmp = tempdateien.neu(tmp_dir)
    os.close(fd)
    proc = None
    try:
        # Laeuft bereits unter `dhrt_spawn_semaphore` (siehe `_check_source`).
        Path(tmp).write_text(source, encoding="utf-8")
        from .dhrt_locate import NO_WINDOW
        proc = subprocess.Popen(
            [str(dhrt), "--check", tmp],
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True,
            creationflags=NO_WINDOW)
        if checker is not None:
            checker._set_active_proc(proc)
        stdout, _ = proc.communicate(timeout=15)
        diags = json.loads(stdout or "[]")
    except subprocess.TimeoutExpired:
        return [ParseProblem(line=1, message="Diagnose-Timeout (dhrt --check antwortete nicht)",
                             severity="warning", phase="diagnostic")]
    except (OSError, ValueError):
        # ValueError deckt json.JSONDecodeError ab (kaputtes/leeres stdout,
        # z.B. bei einem dhrt-Absturz oder abgebrochenem Prozess).
        return [ParseProblem(line=1, message=_DIAG_FAILED_MSG,
                             severity="warning", phase="diagnostic")]
    finally:
        if checker is not None:
            checker._set_active_proc(None)
        try:
            os.unlink(tmp)
        except OSError:
            pass
    origins = _origins_for(source, base)
    out: list[ParseProblem] = []
    for d in diags:
        phase = str(d.get("phase", "compile"))
        line = int(d.get("line", 0)) or 1
        message = str(d.get("message", ""))
        # NUR die Phasen mappen, die auf der GEMERGTEN Quelle laufen.
        # dhrt liefert in EINEM Array drei Koordinatensysteme (siehe
        # main.rs::check_source): lex/parse/compile laufen auf dem
        # gemergten Text, die Hardware-Import-Warnungen werden dagegen
        # ausdruecklich aus `raw_source` berechnet und sind damit schon
        # Buffer-Zeilen. Alles blind zu mappen wuerde die korrekten
        # Warnungen kaputtmachen.
        if origins is not None and phase in ("lex", "parse", "compile"):
            line, message = _map_back(origins, line, message)
        out.append(ParseProblem(
            line=line,
            message=message,
            severity=str(d.get("severity", "error")),
            phase=phase))
    return out


def _origins_for(source: str, base_path) -> list | None:
    """`origins`-Tabelle der gemergten Quelle (oder None, wenn nicht ermittelbar).

    dhrt preprocesst intern selbst und meldet Diagnosen deshalb in
    GEMERGTEN Zeilen -- `main.rs::check_main` sagt das ausdruecklich
    ("der Editor mappt via origins zurueck"). Genau diese Haelfte fehlte
    hier: die Zeilen wurden woertlich uebernommen, wodurch in JEDER Datei
    mit `IMPORT "x.dh"` saemtliche Marker um die Laenge des inlinierten
    Codes verrutschten -- bis weit hinter das Dateiende. Der Fallback
    `_check_syntax_only` machte es die ganze Zeit richtig.

    Wir preprocessen dafuer ein zweites Mal in Python; die Paritaet der
    beiden Implementierungen ist durch `tests/test_rust_preprocess_parity.py`
    abgesichert. Schlaegt es fehl (z.B. kaputter IMPORT), liefern wir None
    und lassen die Zeilen unveraendert -- lieber unverschoben-ungenau als
    falsch verschoben.
    """
    try:
        from ..preprocess import process as _preprocess
        _merged, origins = _preprocess(
            source, base_path or Path.cwd(), file_label="<editor>")
        return origins
    except Exception:
        return None


def _keine_diagnose() -> ParseProblem:
    """Ohne `dhrt` gibt es keine Diagnostik mehr -- und das wird gesagt.

    Frueher stand hier ein Ersatz-Pruefer ueber den Python-Parser. Er lief nur,
    wenn `dhrt` gar nicht gefunden wurde, lieferte genau EIN Problem (das erste
    Syntaxproblem) und kannte keinen Compiler -- also keine unbekannten
    Builtins, keine Arity-Pruefung, keine Namensraum-Meldungen. In genau dieser
    Lage kann der Nutzer sein Programm ohnehin nicht starten; der Run-Knopf
    sagt dort dasselbe. Fuer diese halbe Auskunft hing ein zweiter Parser im
    Repo, der bei jeder Sprachaenderung mitgepflegt werden musste.

    Siehe docs/entwurf-python-parser-entfernen.md.
    """
    return ParseProblem(
        line=1,
        message=("Diagnose nicht verfuegbar: native Runtime 'dhrt' nicht gefunden. "
                 "Einmalig bauen mit: rust\\build_runtime.py"),
        severity="warning",
        phase="setup")

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
        self._active_proc = None   # laufender `dhrt --check`-Subprozess (falls einer)

    def check(self, source: str, base_path: Path | None = None) -> None:
        with self._lock:
            self._gen += 1
            gen = self._gen
            proc = self._active_proc
        # Einen noch laufenden Check-Subprozess abbrechen -- frueher wurde
        # bei schnellem Tippen nur das ERGEBNIS verworfen (Generation-
        # Counter), der `dhrt --check`-Prozess lief aber unbeaufsichtigt
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

    def cancel(self) -> None:
        """Bricht einen laufenden Check ab (Generation-Bump + Subprozess
        terminieren), OHNE einen neuen zu starten. Fuer Tab-Close gedacht:
        ohne das haelt der Worker-Thread (der eine Referenz auf `self`,
        also indirekt den Editor + sein Dokument haelt) den geschlossenen
        Editor bis zu 15s (Subprozess-Timeout) laenger am Leben als noetig
        (Review-Fund)."""
        with self._lock:
            self._gen += 1
            proc = self._active_proc
        if proc is not None:
            try:
                proc.terminate()
            except OSError:
                pass

    def _set_active_proc(self, proc) -> None:
        with self._lock:
            self._active_proc = proc

    def _run(self, gen: int, source: str, base_path: Path | None) -> None:
        problems = _check_source(source, base_path, self)
        # Ist dieses Resultat noch das aktuelle? Sonst verwerfen.
        with self._lock:
            if gen != self._gen:
                return
        # Der Empfaenger (dieses QObject selbst, per Qt-Parent am Editor)
        # kann inzwischen zerstoert worden sein (z.B. Testfenster, das nie
        # geschlossen wurde und per Refcount weggeraeumt wurde) -- ein
        # `.emit()` auf einem geloeschten C++-Objekt ist Use-after-free.
        if not shiboken6.isValid(self):
            return
        # Erst die Voll-Liste (Editor-Underlines + Problems-Panel), dann das
        # erste Problem (Statusbar) -- so ist `current_error()` schon frisch,
        # wenn der Statusbar-Slot laeuft.
        self.problems_changed.emit(problems)
        self.problem_changed.emit(problems[0] if problems else None)
