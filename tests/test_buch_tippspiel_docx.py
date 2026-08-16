"""Der Buch-Build des Tippspiel-Bands bleibt heil.

Geprueft wird das, was still kaputtgehen kann: ein Bild, das im Skript steht,
aber nicht mehr existiert (der Build laesst die Stelle dann einfach aus), ein
Kapitel, das aus dem Dokument gefallen ist, oder ein Verweis auf einen
Kapitelstand, den es nicht mehr gibt.

Das Dokument selbst wird nur gebaut, wenn Node und das docx-Paket da sind --
sonst waere der Test eine Aufforderung, npm install auszufuehren, und das
gehoert nicht in einen Testlauf.
"""
from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_BUCH = _ROOT / "buch-tippspiel" / "buch"
_SKRIPT = _BUCH / "build_book.js"
_IMG = _BUCH / "images"


def _bildnamen():
    """Alle in build_book.js referenzierten PNG-Dateien."""
    text = _SKRIPT.read_text(encoding="utf-8")
    return sorted(set(re.findall(r'figure\("([^"]+\.png)"', text)))


def test_skript_vorhanden():
    assert _SKRIPT.exists(), "build_book.js fehlt"


def test_alle_referenzierten_bilder_existieren():
    """Fehlt ein Bild, laesst der Build die Stelle stillschweigend aus -- im
    fertigen Buch klafft dann eine Luecke, die niemandem auffaellt."""
    fehlend = [n for n in _bildnamen() if not (_IMG / n).exists()]
    assert not fehlend, f"Bilder fehlen in {_IMG}: {fehlend}"


def test_bilder_sind_nicht_leer():
    for name in _bildnamen():
        p = _IMG / name
        assert p.stat().st_size > 2000, f"{name} ist verdaechtig klein"


def test_alle_dreizehn_kapitel_im_buch():
    text = _SKRIPT.read_text(encoding="utf-8")
    for nr in range(1, 14):
        assert f'chapter("Kapitel {nr}:' in text, f"Kapitel {nr} fehlt im Buch"


def test_kein_kapitel_ohne_ueberschrift_im_verzeichnis():
    """toc_titles.json entsteht beim Build und speist den Zwei-Pass."""
    datei = _BUCH / "toc_titles.json"
    if not datei.exists():
        pytest.skip("toc_titles.json entsteht erst beim Build")
    titel = json.loads(datei.read_text(encoding="utf-8"))
    for nr in range(1, 14):
        assert any(t.startswith(f"Kapitel {nr}:") for t in titel), \
            f"Kapitel {nr} steht nicht im Inhaltsverzeichnis"


@pytest.mark.skipif(shutil.which("node") is None, reason="Node nicht installiert")
@pytest.mark.skipif(not (_BUCH / "node_modules" / "docx").exists(),
                    reason="docx nicht installiert (npm install in buch-tippspiel/buch)")
def test_build_laeuft_durch(tmp_path):
    # BUCH_ZIEL_DIR lenkt die Ausgabe ins Temp-Verzeichnis. Ohne das schreibt
    # der Build das eingecheckte .docx neu -- inhaltlich gleich, aber mit
    # frischen ZIP-Zeitstempeln, und git meldet es nach jedem Testlauf als
    # geaendert, obwohl niemand das Buch angefasst hat.
    umgebung = {**os.environ, "BUCH_ZIEL_DIR": str(tmp_path)}
    r = subprocess.run(["node", "build_book.js"], cwd=_BUCH, env=umgebung,
                       capture_output=True, timeout=300)
    aus = (r.stdout + r.stderr).decode("utf-8", "replace")
    assert r.returncode == 0, aus
    assert "Geschrieben:" in aus, aus
    # Der Build meldet fehlende Bilder als Warnung, statt abzubrechen --
    # hier ist sie ein Fehler.
    assert "Bild fehlt" not in aus, aus
    assert (tmp_path / "Drachenhauch-Tippspiel.docx").exists()
