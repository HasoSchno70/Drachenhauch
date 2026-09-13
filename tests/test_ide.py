"""Die IDE in Drachenhauch (`ide/ide.dh`, Weg C aus docs/entwurf-python-abbau.md).

Die Faelle der IDE stehen seit Stufe 34 bis 37 in Pruefsammlungen ohne Python
(`tests/pruef/ide_bausteine.dhtest`, `werkzeug_ide.dhtest`,
`werkzeug_ide_6_12.dhtest`, `werkzeug_ide_13_25.dhtest`,
`werkzeug_ide_sonderfaelle.dhtest`). Hier bleibt nur das PDF-Listing: das
pdf-Modul packt seine Seiten (FlateDecode), ein Leser in Drachenhauch saehe
den Text darin nicht -- geprueft wird mit PyMuPDF.

Braucht ein Fenster und speist Tasten ein -- `_BRAUCHT_GRAFIK` und `_SERIELL`.
"""
from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
IDE = _ROOT / "ide" / "ide.dh"


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

KEY_UP, KEY_DOWN = 1, 2
# raylib-Tastencodes (US-Belegung, physisch)
RL_ENTER, RL_LSHIFT, RL_LCTRL, RL_P, RL_V = 257, 340, 341, 80, 86


def _ide(tmp_path, datei, frames, events, zwischenablage):
    """Die IDE mit `datei` starten, eine Aufnahme abspielen, Protokoll liefern.

    Protokoll, Sitzung, Aufnahme und die IDE-Kopie liegen NEBEN dem
    Projektordner -- der Projektbaum zeigt auch .json und .txt, und die Kopie
    enthaelt den eingeschobenen Text."""
    aussen = tmp_path.parent / (tmp_path.name + "_idekopie")
    aussen.mkdir(exist_ok=True)
    log = aussen / "ide.log"
    ev = sorted(events, key=lambda e: e[0])
    zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
    for frame, typ, *params in ev:
        p = (list(params) + [0, 0, 0, 0])[:4]
        zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
    (aussen / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
    einschub = ('SETFPS(60)\nAUTOMATION_PLAY("' + (aussen / "ev.txt").as_posix() + '")\n'
                'CLIPBOARD_SET("' + zwischenablage + '")\n')
    quelle = aussen / "ide_test.dh"
    quelle.write_text(IDE.read_text(encoding="utf-8").replace('SETFPS(60)\n', einschub, 1), encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle), "--", str(datei)], capture_output=True,
                       text=True, encoding="utf-8", errors="replace", timeout=180,
                       # DH_IDE_KONFIG: die Sitzung des Tests bleibt im Testordner --
                       # sonst schriebe jeder Lauf in die echte ide.json des Nutzers.
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DH_IDE_LOG=str(log),
                                DH_IDE_WURZEL=str(_ROOT), DH_IDE_KONFIG=str(aussen / "ide.json")),
                       cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    return log.read_text(encoding="utf-8").splitlines() if log.exists() else []


def _taste(frame, code, *halten):
    """Eine Taste (mit gehaltenen Modifiern) druecken und wieder loslassen --
    eine eingespeiste Taste bleibt sonst bis zum Ende gedrueckt."""
    ev = [(frame, KEY_DOWN, h) for h in halten]
    ev += [(frame, KEY_DOWN, code), (frame + 2, KEY_UP, code)]
    ev += [(frame + 2, KEY_UP, h) for h in halten]
    return ev


def test_listing_als_pdf_neben_der_quelle(tmp_path):
    fitz = pytest.importorskip("fitz")
    quelle = tmp_path / "spiel.dh"
    quelle.write_text("".join(f'PRINT "Zeile {i} mit Umlaut ae oe ue"\n' for i in range(1, 141)),
                      encoding="utf-8")
    ev = _taste(20, RL_P, RL_LCTRL, RL_LSHIFT) + _taste(50, RL_V, RL_LCTRL) + _taste(70, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=150, events=ev, zwischenablage="pdf")
    assert any(z.startswith("pdf ") for z in log), log
    pdf = tmp_path / "spiel.pdf"
    assert pdf.exists()
    doc = fitz.open(str(pdf))
    assert doc.page_count == 3, doc.page_count            # 140 Zeilen, 66 je Seite
    text = doc[0].get_text()
    assert "spiel.dh" in text and "Seite 1" in text and "Zeile 1 mit" in text, text[:300]
    assert "Zeile 140" in doc[2].get_text()
