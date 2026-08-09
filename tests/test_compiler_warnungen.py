"""Compiler-Warnungen, die Fehler melden, bevor sie wehtun.

Beide Warnungen hier stammen aus echten Stolpersteinen beim Bauen der Demo:

* **Argumentzahl** -- die Grafik-Builtins greifen ihre Argumente per Index ab
  und ignorieren ueberzaehlige STILL. Ein mitgegebenes Argument tat damit
  einfach nichts, ohne ein Wort. (Kostete in Szene 2 der Demo einen halben
  Nachmittag: die Stueckzahl von PLOTS wurde verschluckt.)
* **Verdeckte Konstante** -- GameBasic ignoriert Gross-/Kleinschreibung, eine
  lokale `hoehe` verdeckt also die Konstante `HOEHE`. Der Fehler taucht dann
  weit weg von der Ursache auf.

Beides sind WARNUNGEN, keine Fehler: das Programm laeuft weiter. Geprueft wird
ueber `dhrt --check`, das die Warnungen als JSON ausgibt.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")


def _warnungen(tmp_path, quelle: str):
    """`dhrt --check` laufen lassen und die Warnungstexte liefern."""
    f = tmp_path / "w.gb"
    f.write_text(quelle, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "--check", str(f)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    return [w.get("message", "") for w in json.loads(r.stdout or "[]")]


# --------------------------------------------------------- Argumentzahl
def test_zu_viele_argumente_werden_gemeldet(tmp_path):
    w = _warnungen(tmp_path, """
SCREEN(64, 64, "a", 1)
DIM xs[3] AS INTEGER
DIM ys[3] AS INTEGER
PLOTS(xs, ys, &HFF0000, 2, 9)
""")
    assert any("PLOTS" in m and "5 Argumente" in m for m in w), w


def test_zu_wenige_argumente_werden_gemeldet(tmp_path):
    w = _warnungen(tmp_path, 'PRINT MID$("abc")\n')
    assert any("MID$" in m and "Argument" in m for m in w), w


def test_richtige_aufrufe_schweigen(tmp_path):
    # Pflicht-, Optional- und Vorgabewert-Parameter in allen Kombinationen.
    w = _warnungen(tmp_path, """
SCREEN(64, 64, "a", 1)
DIM xs[3] AS INTEGER
DIM ys[3] AS INTEGER
PLOTS(xs, ys, &HFF0000)
PLOTS(xs, ys, &HFF0000, 2)
LINE(0, 0, 10, 10)
LINE(0, 0, 10, 10, &HFFFFFF)
PRINT MID$("abc", 1)
PRINT MID$("abc", 1, 2)
""")
    assert not any("Argument" in m for m in w), w


def test_variadische_builtins_werden_nicht_gemeldet(tmp_path):
    # PATHJOIN nimmt beliebig viele Teile ("..."-Signatur) -- da darf die
    # Pruefung keine Obergrenze erfinden.
    w = _warnungen(tmp_path, 'PRINT PATHJOIN("a", "b")\nPRINT PATHJOIN("a", "b", "c", "d")\n')
    assert not any("Argument" in m for m in w), w


# ---------------------------------------------------- verdeckte Konstante
def test_lokale_variable_verdeckt_konstante(tmp_path):
    w = _warnungen(tmp_path, """
CONST HOEHE AS INTEGER = 720

SUB stolpert()
    DIM hoehe AS FLOAT
    hoehe = 0.6
END SUB

stolpert()
""")
    assert any("verdeckt" in m and "hoehe" in m for m in w), w


def test_anderer_name_wird_nicht_gemeldet(tmp_path):
    w = _warnungen(tmp_path, """
CONST HOEHE AS INTEGER = 720

SUB sauber()
    DIM hoch AS FLOAT
    hoch = 0.6
END SUB

sauber()
""")
    assert not any("verdeckt" in m for m in w), w


def test_gleicher_name_im_hauptprogramm_ist_kein_verdecken(tmp_path):
    # Auf oberster Ebene waere es eine Namens-Kollision (eigener Fehler),
    # kein Verdecken -- die Warnung darf hier nicht zusaetzlich feuern.
    w = _warnungen(tmp_path, """
CONST GRENZE AS INTEGER = 10

SUB nutzt()
    PRINT GRENZE
END SUB

nutzt()
""")
    assert not any("verdeckt" in m for m in w), w
