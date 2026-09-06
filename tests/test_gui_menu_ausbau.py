"""gui-Ausbau Punkt 2: Menues -- Tastenkuerzel, Untermenues, Haekchen,
Sperren, Symbole, Text aendern.

Die Kuerzel werden ECHT gedrueckt (Automation-Wiedergabe): ein Test, der
nur GUI_MENU_SHORTCUT setzt und die JSON liest, belegte nichts ueber das
Ausloesen. Untermenues und Haekchen laufen ueber den Rundweg durch die Datei
und ueber ihr Kuerzel -- ein Menue per Maus aufzuklappen braeuchte die
Geometrie der Leiste, die kein Befehl liefert.

Braucht ein Fenster, steht darum in `conftest._BRAUCHT_GRAFIK`.
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


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
RL_S, RL_G, RL_Z, RL_F5, RL_DELETE = 83, 71, 90, 294, 261
RL_LCTRL, RL_LSHIFT, RL_LALT = 341, 340, 342

_KOPF = ('IMPORT "gui"\n'
         'SCREEN(400, 300, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 10, 10, 360, 260)\n'
         'DIM m AS INTEGER : m = GUI_MENU(w, "Datei")\n')


def _lauf(tmp_path, src, frames=12, events=None):
    if events is not None:
        ev = sorted(events, key=lambda e: e[0])
        zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
        for frame, typ, *params in ev:
            p = (list(params) + [0, 0, 0, 0])[:4]
            zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
        (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
        src = src.replace('SET_WINDOW_POS(-3000, -3000)\n',
                          'SET_WINDOW_POS(-3000, -3000)\nAUTOMATION_PLAY("ev.txt")\n', 1)
    f = tmp_path / "t.dh"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    # raylib meldet beim Lesen der Aufnahme gelegentlich "Issue reading line
    # to buffer" auf stdout -- die Ereignisse kommen trotzdem an.
    return [ln.rstrip() for ln in r.stdout.splitlines()
            if ln.strip() and not ln.startswith("WARNING:")]


def _schleife(bilder, probe):
    return (f"DIM f AS INTEGER\nFOR f = 1 TO {bilder}\n    GUI_UPDATE()\n"
            f"{probe}    GUI_DRAW()\n    FLIP()\nNEXT\n")


def _tipp(frame, code, *halten):
    ev = [(frame, KEY_DOWN, h) for h in halten]
    ev += [(frame, KEY_DOWN, code), (frame + 1, KEY_UP, code)]
    ev += [(frame + 1, KEY_UP, h) for h in halten]
    return ev


# ---------------------------------------------------------------- Kuerzel
def test_strg_s_loest_den_eintrag_aus(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM a AS INTEGER : a = GUI_MENU_ITEM(m, "Sichern", "Strg+S")\n'
                + _schleife(10, '    IF GUI_CLICKED(a) THEN PRINT "sichern " + STR$(f)\n'),
                events=_tipp(4, RL_S, RL_LCTRL))
    assert len([ln for ln in out if ln.startswith("sichern ")]) == 1, out


def test_ohne_strg_loest_es_nicht_aus(tmp_path):
    """Die Modifier muessen GENAU passen -- ein blosses S ist kein Strg+S,
    und Strg+Umschalt+S auch nicht."""
    out = _lauf(tmp_path, _KOPF + 'DIM a AS INTEGER : a = GUI_MENU_ITEM(m, "Sichern", "Strg+S")\n'
                + _schleife(14, '    IF GUI_CLICKED(a) THEN PRINT "sichern"\n'),
                frames=16, events=_tipp(3, RL_S) + _tipp(8, RL_S, RL_LCTRL, RL_LSHIFT))
    assert "sichern" not in out


def test_funktionstaste_trifft_auch_aus_dem_textfeld(tmp_path):
    """Ohne Strg/Alt gehoert eine Taste dem Textfeld mit Fokus -- F1..F12 aber
    nicht: sie erzeugen nie Text. Gefunden an der IDE in Drachenhauch, deren
    F5 aus dem Code-Feld heraus nie startete. Gegenprobe: Entf bleibt dort
    dem Feld (loescht das Zeichen), der Eintrag schweigt."""
    out = _lauf(tmp_path, _KOPF + 'DIM a AS INTEGER : a = GUI_MENU_ITEM(m, "Start", "F5")\n'
                'DIM b AS INTEGER : b = GUI_MENU_ITEM(m, "Loeschen", "Entf")\n'
                'DIM tf AS GUI_WIDGET : tf = GUI_TEXTINPUT(w, 10, 40, 200, 24, "abc")\n'
                'GUI_FOCUS(tf)\n'
                + _schleife(12, '    IF GUI_CLICKED(a) THEN PRINT "start"\n'
                                '    IF GUI_CLICKED(b) THEN PRINT "loeschen"\n'
                                '    IF f = 12 THEN PRINT "text=" + GUI_TEXT(tf)\n'),
                frames=14, events=_tipp(3, RL_F5) + _tipp(8, RL_DELETE))
    # Entf ging ans Feld (der Text ist nicht mehr "abc"), nicht ans Menue.
    assert out[0] == "start" and "loeschen" not in out, out
    assert out[-1].startswith("text=") and out[-1] != "text=abc", out


def test_funktionstaste_und_alt(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM a AS INTEGER : a = GUI_MENU_ITEM(m, "Start", "F5")\n'
                'DIM b AS INTEGER : b = GUI_MENU_ITEM(m, "Eigenschaften", "Alt+Enter")\n'
                + _schleife(12, '    IF GUI_CLICKED(a) THEN PRINT "start"\n'
                                '    IF GUI_CLICKED(b) THEN PRINT "eig"\n'),
                frames=14, events=_tipp(3, RL_F5) + _tipp(8, 257, RL_LALT))
    assert out == ["start", "eig"]


def test_ein_gesperrter_eintrag_hat_kein_kuerzel(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM a AS INTEGER : a = GUI_MENU_ITEM(m, "Rueckgaengig", "Strg+Z")\n'
                'GUI_MENU_ENABLE(a, FALSE)\n'
                + _schleife(18, '    IF GUI_CLICKED(a) THEN PRINT "zurueck"\n'
                                '    IF f = 8 THEN GUI_MENU_ENABLE(a, TRUE)\n'),
                frames=20, events=_tipp(3, RL_Z, RL_LCTRL) + _tipp(12, RL_Z, RL_LCTRL))
    assert out.count("zurueck") == 1, "gesperrt: nichts; nach dem Freigeben: einmal"


def test_ohne_modifier_gehoert_die_taste_dem_textfeld(tmp_path):
    """'Entf' als Kuerzel darf im Textfeld nicht den Menuepunkt ausloesen --
    dort loescht es ein Zeichen. Ohne Textfokus loest es aus."""
    out = _lauf(tmp_path, _KOPF + 'DIM a AS INTEGER : a = GUI_MENU_ITEM(m, "Loeschen", "Entf")\n'
                'DIM tf AS GUI_WIDGET : tf = GUI_TEXTINPUT(w, 10, 40, 200, 26)\n'
                'GUI_FOCUS(tf)\n'
                + _schleife(16, '    IF GUI_CLICKED(a) THEN PRINT "loeschen " + STR$(f)\n'
                                '    IF f = 8 THEN GUI_DESTROY(tf)\n'),
                frames=18, events=_tipp(3, RL_DELETE) + _tipp(12, RL_DELETE))
    treffer = [ln for ln in out if ln.startswith("loeschen ")]
    assert len(treffer) == 1 and int(treffer[0].split()[1]) > 8, out


def test_kuerzel_nachtraeglich_und_wieder_weg(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM a AS INTEGER : a = GUI_MENU_ITEM(m, "Gitter")\n'
                'GUI_MENU_SHORTCUT(a, "Strg+G")\n'
                + _schleife(16, '    IF GUI_CLICKED(a) THEN PRINT "gitter"\n'
                                '    IF f = 8 THEN GUI_MENU_SHORTCUT(a, "")\n'),
                frames=18, events=_tipp(3, RL_G, RL_LCTRL) + _tipp(12, RL_G, RL_LCTRL))
    assert out.count("gitter") == 1


def test_unbekannte_taste_ist_ein_fehler(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'TRY\n    GUI_MENU_ITEM(m, "x", "Hyper+Q")\nCATCH e\n    PRINT e\nEND TRY\n'
                'TRY\n    GUI_MENU_ITEM(m, "y", "Strg+")\nCATCH e2\n    PRINT e2\nEND TRY\n', frames=1)
    assert "unbekannt" in out[0] and "GUI_MENU_ITEM" in out[0]
    assert "keine Taste" in out[1]


# ---------------------------------------------------------------- Haekchen, Text, Sperren
def test_haekchen_kippt_mit_dem_kuerzel(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM a AS INTEGER : a = GUI_MENU_ITEM(m, "Statusleiste", "Strg+G")\n'
                'GUI_MENU_CHECK(a, TRUE)\nPRINT GUI_MENU_CHECKED(a)\n'
                + _schleife(16, '    IF GUI_CLICKED(a) THEN PRINT GUI_MENU_CHECKED(a)\n'),
                frames=18, events=_tipp(3, RL_G, RL_LCTRL) + _tipp(10, RL_G, RL_LCTRL))
    assert out == ["TRUE", "FALSE", "TRUE"]


def test_text_aendern_und_falsche_handles(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM a AS INTEGER : a = GUI_MENU_ITEM(m, "Pause")\n'
                'GUI_MENU_TEXT(a, "Weiter")\n'
                'PRINT INSTR(GUI_TO_JSON(w), "Weiter") >= 0\n'
                'TRY\n    GUI_MENU_ENABLE(m, FALSE)\nCATCH e\n    PRINT e\nEND TRY\n'
                'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, "k", 10, 40, 60, 24)\n'
                'TRY\n    GUI_MENU_CHECK(b, TRUE)\nCATCH e2\n    PRINT e2\nEND TRY\n'
                'GUI_MENU_SEPARATOR(m)\n', frames=1)
    assert out[0] == "TRUE"
    assert "Menue-Eintrag" in out[1]
    assert "Menue-Eintrag" in out[2]


# ---------------------------------------------------------------- Untermenues + Datei
def test_untermenue_ueberlebt_die_datei_verschachtelt(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM s AS INTEGER : s = GUI_SUBMENU(m, "Zuletzt")\n'
                'DIM z1 AS INTEGER : z1 = GUI_MENU_ITEM(s, "eins.dh")\n'
                'DIM s2 AS INTEGER : s2 = GUI_SUBMENU(s, "Aeltere")\n'
                'DIM z2 AS INTEGER : z2 = GUI_MENU_ITEM(s2, "alt.dh", "Strg+2")\n'
                'DIM h AS INTEGER : h = GUI_MENU_ITEM(m, "Statusleiste")\n'
                'GUI_MENU_CHECK(h, TRUE)\n'
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)\n'
                'PRINT GUI_TO_JSON(w2) = j\n'
                'PRINT INSTR(j, "alt.dh") >= 0\n'
                'PRINT INSTR(j, "Strg+2") >= 0\n'
                'PRINT INSTR(j, "checkable") >= 0\n', frames=1)
    assert out == ["TRUE", "TRUE", "TRUE", "TRUE"]


def test_kuerzel_im_untermenue_loest_aus(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM s AS INTEGER : s = GUI_SUBMENU(m, "Zuletzt")\n'
                'DIM z AS INTEGER : z = GUI_MENU_ITEM(s, "alt.dh", "Strg+G")\n'
                + _schleife(10, '    IF GUI_CLICKED(z) THEN PRINT "alt"\n'),
                events=_tipp(4, RL_G, RL_LCTRL))
    assert out == ["alt"]


def test_ein_untermenue_ist_kein_kontextmenue(tmp_path):
    """Das erste Menue ohne Leiste galt als Kontextmenue -- ein Untermenue
    darf diese Rolle nicht uebernehmen. Rechtsklick ohne GUI_CONTEXT oeffnet
    nichts; mit GUI_CONTEXT das Kontextmenue, nicht das Untermenue."""
    MOUSE_POSITION, MOUSE_BUTTON_DOWN, MOUSE_BUTTON_UP = 7, 6, 5
    # Der Linksklick sitzt 15 px unter und rechts der Ecke: dort liegt die
    # erste Zeile des Popups (2 px Rand oben).
    ev = [(3, MOUSE_POSITION, 150, 150), (4, MOUSE_POSITION, 150, 150),
          (4, MOUSE_BUTTON_DOWN, 1), (5, MOUSE_BUTTON_UP, 1),
          (7, MOUSE_POSITION, 165, 165), (7, MOUSE_BUTTON_DOWN, 0), (8, MOUSE_BUTTON_UP, 0)]
    out = _lauf(tmp_path, _KOPF +
                'DIM s AS INTEGER : s = GUI_SUBMENU(m, "Zuletzt")\n'
                'DIM z AS INTEGER : z = GUI_MENU_ITEM(s, "alt.dh")\n'
                'DIM c AS INTEGER : c = GUI_CONTEXT(w)\n'
                'DIM ci AS INTEGER : ci = GUI_MENU_ITEM(c, "Kontext")\n'
                # Der Linksklick an derselben Stelle trifft den ersten Eintrag
                # des Popups, das der Rechtsklick dort geoeffnet hat.
                + _schleife(10, '    IF GUI_CLICKED(ci) THEN PRINT "kontext"\n'
                                '    IF GUI_CLICKED(z) THEN PRINT "unter"\n'),
                events=ev)
    assert out == ["kontext"]
