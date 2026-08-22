"""Faehrt die schweren Qt-Testdateien -- je Datei ein eigener pytest-Prozess.

WARUM UEBERHAUPT:

Die Qt-Testdateien lassen ihre Fenster stehen (sie schliessen sie nie, und der
Abbau ist nachweislich nicht sicher -- siehe `tests/conftest.py`). In EINEM
gemeinsamen pytest-Prozess sammeln sich diese Altlasten ueber ALLE Dateien
hinweg an, die derselbe xdist-Arbeiter abbekommt. Gemessen am 2026-08-22:

    2284 uebrig gebliebene Top-Level-Fenster ueber alle 81 Qt-Dateien
    davon 2078 (91 %) in 16 Dateien

Jede Operation, die dann ueber ALLE Fenster des Prozesses laeuft, fasst diese
Altlasten an: `app.processEvents()` genauso wie `QApplication.setStyleSheet()`
(globales Repolish, das der Editor beim Theme-Wechsel im Konstruktor ausloest).
Genau dort starb der CI-Arbeiter sporadisch mit "Windows fatal exception:
access violation" -- zwei belegte Stellen, beide in fremden Altlasten:

    app.processEvents()          test_editor_qt_swatch / _particle_editor
    QApplication.setStyleSheet() test_editor_qt_window_close_cleanup

Einzeln laeuft jede dieser Dateien sauber durch. Ein eigener Prozess je Datei
nimmt dem Problem also die Grundlage, ohne eine einzige Zusicherung zu
streichen -- das Betriebssystem raeumt auf, wo Qts eigener Abbau abstuerzt.

WARUM NICHT ANDERS GELOEST:

* Fenster am Testende zerstoeren: gemessen, stuerzt ab (`sendPostedEvents` mit
  DeferredDelete) -- die Editor-Widgets haben echte Zerstoerungs-Reihenfolge-
  Fehler. Deshalb entschaerft `tests/conftest.py` sie nur.

Dieselbe Schwaeche zeigt sich beim PROZESSENDE: einzelne Dateien beenden sich
mit STATUS_HEAP_CORRUPTION, nachdem alle Tests gruen waren (gemessen an
test_sfxeditor.py, 4 von 4). Bisher fiel das niemandem auf, weil xdist den
Rueckgabewert eines Arbeiters nie ansieht -- dieses Werkzeug tut es. Deshalb
setzt es `DH_TEST_HARTES_ENDE=1`; conftest beendet den Prozess dann per
`os._exit()`, sobald das Ergebnis feststeht. Uebersprungen wird nur der Abbau
eines Prozesses, der ohnehin endet.
* Mehr xdist-Arbeiter: verduennt bloss die Wahrscheinlichkeit.
* `pytest --forked`: gibt es auf Windows nicht.

Aufruf (dasselbe Kommando lokal wie in der CI):

    python tools/qt_tests_einzeln.py [-j N] [weitere pytest-Argumente]
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

_ROOT = Path(__file__).resolve().parent.parent


def _dateien() -> list[str]:
    """Alle Qt-Testdateien -- erkannt von derselben Funktion, mit der conftest
    den `qt`-Marker setzt.

    Genau diese Menge waehlt der gemeinsame Lauf ueber `-m "not qt"` ab. Eine
    zweite, gepflegte Liste waere die sicherste Art, beide auseinander laufen
    zu lassen: eine neue Qt-Datei liefe dann entweder gar nicht oder zweimal.

    Die Dateien mit `seriell`-Marker bleiben aussen vor -- die holt der
    serielle Durchgang, sonst liefen sie doppelt.
    """
    sys.path.insert(0, str(_ROOT / "tests"))
    import conftest                                  # type: ignore[import-not-found]
    treffer = []
    for p in sorted((_ROOT / "tests").glob("test_*.py")):
        if p.name in conftest._SERIELL:
            continue
        if conftest._module_uses_qt(str(p)):
            treffer.append(p.name)
    return treffer


def _lauf(datei: str, extra: list[str]) -> tuple[str, int, str]:
    t0 = time.perf_counter()
    umgebung = dict(os.environ, DH_TEST_HARTES_ENDE="1")
    p = subprocess.run(
        [sys.executable, "-m", "pytest", str(_ROOT / "tests" / datei), "-q",
         "-p", "no:cacheprovider", *extra],
        cwd=_ROOT, capture_output=True, text=True, env=umgebung,
    )
    dauer = time.perf_counter() - t0
    letzte = [z for z in p.stdout.strip().splitlines() if z.strip()]
    kurz = letzte[-1] if letzte else "(keine Ausgabe)"
    return datei, p.returncode, f"{dauer:5.1f}s  {kurz}" if p.returncode == 0 else (
        f"{dauer:5.1f}s  FEHLER\n{p.stdout}\n{p.stderr}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("-j", type=int, default=min(4, os.cpu_count() or 4),
                    help="wieviele Dateien gleichzeitig (Vorgabe: 4)")
    args, extra = ap.parse_known_args()

    dateien = _dateien()
    print(f"{len(dateien)} Qt-Dateien, je ein eigener Prozess, {args.j} gleichzeitig")
    t0 = time.perf_counter()
    fehler = []
    with ThreadPoolExecutor(max_workers=args.j) as pool:
        for datei, rc, text in pool.map(lambda d: _lauf(d, extra), dateien):
            print(f"  {'ok ' if rc == 0 else 'FEHL'} {datei:52s} {text}")
            if rc != 0:
                fehler.append(datei)
    print(f"\n{len(dateien) - len(fehler)}/{len(dateien)} gruen "
          f"in {time.perf_counter() - t0:.1f}s")
    if fehler:
        print("Fehlgeschlagen: " + ", ".join(fehler))
    return 1 if fehler else 0


if __name__ == "__main__":
    raise SystemExit(main())
