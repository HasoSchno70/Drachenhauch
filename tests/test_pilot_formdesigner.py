"""Der Form-Designer in Drachenhauch (`examples/197_form_designer.dh`, Weg B).

Die Entwurfsflaeche ist ein echtes GUI_WINDOW im Entwurfsmodus
(`GUI_WINDOW_DESIGN`); der Designer verwaltet die Maus selbst. Geprueft wird
mit ECHTEN Klicks ueber die Eingabe-Wiedergabe (Palette anklicken, auf die
Form klicken, Strg+S) und der Datei, die dabei entsteht -- gelesen vom
FREMDEN Leser `drachenhauch.formdesigner.document.FormDoc`, dem Modell des
Qt-Designers. Ein Format, das nur sein Schreiber liest, waere nicht geprueft.

Braucht ein Fenster und speist Eingaben ein -- `_BRAUCHT_GRAFIK` + `_SERIELL`.
"""
from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

from drachenhauch.formdesigner.document import FormDoc

_ROOT = Path(__file__).resolve().parent.parent
DESIGNER = _ROOT / "examples" / "197_form_designer.dh"


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
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7
RL_F5, RL_LCTRL, RL_S, RL_Z, RL_DELETE = 294, 341, 83, 90, 261

# Lagen aus dem Quelltext des Designers: Palette links (Menueleiste 26 px,
# Liste ab Inhalts-y 56, Zeilen 22 px), die Form ab (PAL_B + 40, 60).
PAL_LISTE_Y = 26 + 56
ZEILE_H = 22
FORM_X, FORM_Y = 230 + 40, 60


def _klick(frame, x, y, knopf=0):
    return [(frame, MOUSE_POSITION, x, y), (frame + 1, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_BUTTON_DOWN, knopf), (frame + 3, MOUSE_BUTTON_UP, knopf)]


def _zug(frame, x0, y0, x1, y1, schritte=6):
    """Druecken bei (x0,y0), in Schritten nach (x1,y1) ziehen, loslassen."""
    ev = [(frame, MOUSE_POSITION, x0, y0), (frame + 1, MOUSE_POSITION, x0, y0),
          (frame + 1, MOUSE_BUTTON_DOWN, 0)]
    for k in range(1, schritte + 1):
        ev.append((frame + 1 + k, MOUSE_POSITION, x0 + (x1 - x0) * k // schritte, y0 + (y1 - y0) * k // schritte))
    ev.append((frame + 3 + schritte, MOUSE_BUTTON_UP, 0))
    return ev


def _taste(frame, code, *halten):
    ev = [(frame, KEY_DOWN, h) for h in halten]
    ev += [(frame, KEY_DOWN, code), (frame + 2, KEY_UP, code)]
    ev += [(frame + 2, KEY_UP, h) for h in halten]
    return ev


def _designer(tmp_path, form, frames, events):
    log = tmp_path / "fd.log"
    ev = sorted(events, key=lambda e: e[0])
    zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
    for frame, typ, *params in ev:
        p = (list(params) + [0, 0, 0, 0])[:4]
        zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
    (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
    text = DESIGNER.read_text(encoding="utf-8").replace(
        'SETFPS(60)\n', 'SETFPS(60)\nAUTOMATION_PLAY("' + (tmp_path / "ev.txt").as_posix() + '")\n', 1)
    (tmp_path / "_fd").mkdir(exist_ok=True)
    quelle = tmp_path / "_fd" / "designer_test.dh"
    quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle), "--", str(form)], capture_output=True,
                       text=True, encoding="utf-8", errors="replace", timeout=180,
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DH_FORM_LOG=str(log)),
                       cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    return log.read_text(encoding="utf-8").splitlines() if log.exists() else []


def test_designer_uebersetzt_ohne_befund():
    r = subprocess.run([str(_DHRT), "--check", str(DESIGNER)], capture_output=True, text=True,
                       encoding="utf-8", timeout=120)
    assert r.returncode == 0 and r.stdout.strip() == "[]", r.stdout


def test_ablegen_sichern_und_der_qt_leser_liest_es(tmp_path):
    """Palette 'Button' anklicken, auf die Form klicken (250,180 -> Raster 248,184),
    Strg+S. Die Datei liest das Modell des Qt-Designers: ein Button, benannt,
    an der gerasterten Stelle, mit der Vorgabegroesse der Palette."""
    form = tmp_path / "neu.dhform"
    ev = _klick(20, 60, PAL_LISTE_Y + 2 + ZEILE_H // 2)          # Zeile 0 = Button
    ev += _klick(40, FORM_X + 250, FORM_Y + 180)
    ev += _taste(70, RL_S, RL_LCTRL)
    log = _designer(tmp_path, form, 120, ev)
    assert "palette button" in log, log
    assert "neu button 248 184" in log, log
    assert any(z.startswith("gesichert ") for z in log), log
    doc = FormDoc.load(str(form))
    assert len(doc.controls) == 1
    c = doc.controls[0]
    assert (c.kind, c.name, c.x, c.y, c.w, c.h) == ("button", "button1", 248, 184, 100, 28)
    assert c.text == "Button"


def test_verschieben_und_rueckgaengig(tmp_path):
    """Ein abgelegter Button wird um 80 px nach rechts gezogen (gerastert),
    Strg+Z nimmt den Zug zurueck, ein zweites Strg+Z das Ablegen."""
    form = tmp_path / "neu.dhform"
    ev = _klick(20, 60, PAL_LISTE_Y + 2 + ZEILE_H // 2)
    ev += _klick(40, FORM_X + 100, FORM_Y + 100)                   # Raster: 100 -> 104; Button 104,104 .. 204,132
    ev += _zug(70, FORM_X + 140, FORM_Y + 110, FORM_X + 220, FORM_Y + 110)
    ev += _taste(110, RL_Z, RL_LCTRL) + _taste(130, RL_Z, RL_LCTRL)
    ev += _taste(150, RL_S, RL_LCTRL)
    log = _designer(tmp_path, form, 200, ev)
    assert "neu button 104 104" in log, log
    assert "bewegt 0 184 104 100 28" in log, log       # +80 px, gerastert
    assert "rueckgaengig 1" in log and "rueckgaengig 0" in log, log
    assert len(FormDoc.load(str(form)).controls) == 0


def test_f5_schreibt_das_laufprogramm_und_startet_es(tmp_path):
    form = tmp_path / "neu.dhform"
    ev = _klick(20, 60, PAL_LISTE_Y + 2 + ZEILE_H // 2)
    ev += _klick(40, FORM_X + 100, FORM_Y + 100)
    ev += _taste(70, RL_S, RL_LCTRL) + _taste(90, RL_F5)
    log = _designer(tmp_path, form, 200, ev)
    assert any(z.startswith("gestartet ") for z in log), log
    lauf = tmp_path / "neu_lauf.dh"
    assert lauf.exists()
    text = lauf.read_text(encoding="utf-8")
    assert 'GUI_LOAD("neu.dhform")' in text and "GUI_WINDOW_CHROME(frm, FALSE)" in text
    r = subprocess.run([str(_DHRT), "--check", str(lauf)], capture_output=True, text=True,
                       encoding="utf-8", timeout=120)
    assert r.stdout.strip() == "[]", r.stdout


def test_bestehende_form_bleibt_vollstaendig(tmp_path):
    """Oeffnen + Sichern ohne Aenderung: alles, was die Laufzeit schreibt
    (Menues, Reiter, Tabellendaten) und die Designer-Felder (`code`) bleiben
    erhalten -- das Modell ist das JSON selbst."""
    quelle = _ROOT / "examples" / "forms" / "settings.dhform"
    form = tmp_path / "settings.dhform"
    import json
    d = json.loads(quelle.read_text(encoding="utf-8"))
    d["code"] = {"on_save": 'PRINT "gesichert"'}
    d["menus"] = [{"label": "Datei", "in_bar": True, "items": [{"label": "Ende"}]}]
    form.write_text(json.dumps(d), encoding="utf-8")
    log = _designer(tmp_path, form, 80, _taste(20, RL_S, RL_LCTRL))
    assert "geladen 9" in log, log
    nachher = json.loads(form.read_text(encoding="utf-8"))
    assert nachher["code"] == {"on_save": 'PRINT "gesichert"'}
    assert nachher["menus"][0]["label"] == "Datei"
    assert [w["kind"] for w in nachher["widgets"]] == [w["kind"] for w in d["widgets"]]


# ---------------------------------------------------------- Laufzeit-Baustein

def test_entwurfsmodus_nimmt_den_widgets_die_eingabe(tmp_path):
    """GUI_WINDOW_DESIGN: ein Klick auf den Knopf loest NICHTS aus, GUI_HIT_TEST
    findet ihn trotzdem. Gegenprobe: ohne Entwurfsmodus klickt derselbe Klick."""
    src = ('IMPORT "gui"\nSCREEN(400, 300, "T", 1)\nSET_WINDOW_POS(-3000, -3000)\n'
           'DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 300, 200)\n'
           'GUI_WINDOW_CHROME(w, FALSE)\n'
           'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, "Ok", 20, 20, 100, 30)\n'
           'GUI_WINDOW_DESIGN(w, {design})\n'
           'DIM f AS INTEGER\nDIM traf AS BOOLEAN : traf = FALSE\n'
           'FOR f = 1 TO 8\n    GUI_UPDATE()\n    IF GUI_CLICKED(b) THEN traf = TRUE\n'
           '    CLS(0) : GUI_DRAW() : FLIP()\nNEXT\n'
           'PRINT traf ; " " ; GUI_HIT_TEST(60, 35) = b\n')
    ev = _klick(2, 60, 35)
    zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
    for frame, typ, *params in ev:
        p = (list(params) + [0, 0, 0, 0])[:4]
        zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
    (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
    out = {}
    for design in ("TRUE", "FALSE"):
        f = tmp_path / f"t_{design}.dh"
        f.write_text(src.replace("{design}", design).replace(
            'SET_WINDOW_POS(-3000, -3000)\n', 'SET_WINDOW_POS(-3000, -3000)\nAUTOMATION_PLAY("ev.txt")\n'),
            encoding="utf-8")
        r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                           encoding="utf-8", errors="replace", timeout=120,
                           env=dict(os.environ, DHRT_FRAMES="10"), cwd=str(tmp_path))
        assert r.returncode == 0, r.stderr
        out[design] = [ln for ln in r.stdout.splitlines() if ln.strip() and not ln.startswith(("WARNING:", "INFO:"))][-1]
    assert out["TRUE"] == "FALSE TRUE", out
    assert out["FALSE"] == "TRUE TRUE", out
