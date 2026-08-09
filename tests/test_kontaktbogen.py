"""Kontaktbogen: mehrere Bilder eines Laufs als Raster in EINE PNG.

Ein einzelner Screenshot zeigt einen AUGENBLICK. Vieles geht aber erst ueber
die Zeit schief -- etwas kippt zu frueh um, ein Rand bleibt stehen, eine
Bewegung ruckelt. Der Kontaktbogen nimmt in festen Abstaenden Bilder auf und
setzt sie beschriftet nebeneinander, damit ein ABLAUF pruefbar wird.

Gesteuert wird er ueber Umgebungsvariablen wie die vorhandene
Headless-Verifizierung:

    GBRT_FRAMES=480 GBRT_CONTACT=bogen.png gbrt run demo.gb

Ohne weitere Angaben verteilt er GBRT_CONTACT_MAX (Standard 12) Bilder
gleichmaessig ueber GBRT_FRAMES.
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_GBRT = _find_gbrt()
pytestmark = pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")
pytest.importorskip("PIL", reason="Pillow noetig zum Pixel-Pruefen")

# Ein Punkt wandert nach rechts -- so laesst sich am Bogen ABLESEN, dass die
# Bilder aus verschiedenen Zeitpunkten stammen und in der richtigen Reihenfolge
# stehen.
QUELLE = """
SCREEN(160, 100, "bogen", 1)
DIM f AS INTEGER
FOR f = 0 TO 200
    CLS(0)
    BOX(f, 40, f + 8, 48, &HFF0000)
    FLIP()
NEXT
"""


def _lauf(tmp_path, **env):
    (tmp_path / "a.gb").write_text(QUELLE, encoding="utf-8")
    umg = dict(os.environ, **{k: str(v) for k, v in env.items()})
    r = subprocess.run([str(_GBRT), "run", str(tmp_path / "a.gb")], capture_output=True,
                       text=True, encoding="utf-8", env=umg, timeout=120, cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return tmp_path / "bogen.png"


def test_bogen_entsteht_mit_der_erwarteten_kachelzahl(tmp_path):
    from PIL import Image

    p = _lauf(tmp_path, GBRT_FRAMES=60, GBRT_CONTACT="bogen.png",
              GBRT_CONTACT_MAX=6, GBRT_CONTACT_COLS=3)
    assert p.exists(), "Kontaktbogen wurde nicht geschrieben"
    with Image.open(p) as im:
        b, h = im.size
    # 3 Spalten x 2 Zeilen bei 160 Pixel breiten Kacheln (kleiner als die
    # Verkleinerungsgrenze, also unveraendert) + Raender.
    assert b == 3 * 160 + 4 * 8, f"Breite {b} passt nicht zu 3 Spalten"
    assert h == 2 * (100 + 18) + 3 * 8, f"Hoehe {h} passt nicht zu 2 Zeilen"


def test_spaltenzahl_wirkt(tmp_path):
    from PIL import Image

    p = _lauf(tmp_path, GBRT_FRAMES=60, GBRT_CONTACT="bogen.png",
              GBRT_CONTACT_MAX=6, GBRT_CONTACT_COLS=6)
    with Image.open(p) as im:
        b, h = im.size
    assert b == 6 * 160 + 7 * 8      # eine einzige Zeile
    assert h == 1 * (100 + 18) + 2 * 8


def test_kacheln_zeigen_verschiedene_zeitpunkte(tmp_path):
    # Der wandernde Punkt muss in jeder Kachel weiter rechts stehen -- sonst
    # waeren alle Bilder aus demselben Augenblick.
    from PIL import Image

    p = _lauf(tmp_path, GBRT_FRAMES=90, GBRT_CONTACT="bogen.png",
              GBRT_CONTACT_MAX=3, GBRT_CONTACT_COLS=3)
    with Image.open(p) as im:
        rgb = im.convert("RGB")
        positionen = []
        for sp in range(3):
            x0 = 8 + sp * (160 + 8)
            # Zeile 44 innerhalb der Kachel (Kachel beginnt bei y=8)
            xs = [x for x in range(x0, x0 + 160)
                  if rgb.getpixel((x, 8 + 44))[0] > 120]
            positionen.append(min(xs) - x0 if xs else -1)
    assert all(p0 >= 0 for p0 in positionen), f"Punkt fehlt in einer Kachel: {positionen}"
    assert positionen[0] < positionen[1] < positionen[2], \
        f"Kacheln nicht in zeitlicher Reihenfolge: {positionen}"


def test_ohne_umgebungsvariable_entsteht_nichts(tmp_path):
    # Rueckwaertskompatibilitaet: wer GBRT_CONTACT nicht setzt, merkt nichts.
    (tmp_path / "a.gb").write_text(QUELLE, encoding="utf-8")
    umg = dict(os.environ, GBRT_FRAMES="20")
    umg.pop("GBRT_CONTACT", None)
    r = subprocess.run([str(_GBRT), "run", str(tmp_path / "a.gb")], capture_output=True,
                       text=True, encoding="utf-8", env=umg, timeout=120, cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    assert not (tmp_path / "bogen.png").exists()
