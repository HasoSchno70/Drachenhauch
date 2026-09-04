"""gui-Ausbau Punkt 1: Text und Formular.

Umbruch und Ausrichtung bei Beschriftungen, Passwort/Nur-Lesen/Hoechst-
laenge/Zahlenfilter im Textfeld, Enter als Ereignis, Strg+Z beim Tippen,
Standard- und Abbrechen-Knopf je Fenster.

Getippt wird ueber die ZWISCHENABLAGE (Strg+V): raylibs Wiedergabe legt
Tasten in die Tastenwarteschlange, aber keine Zeichen in die
Zeichenwarteschlange -- ein KEY_DOWN erzeugt kein Zeichen. Das Einfuegen
laeuft durch dieselben Filter wie das Tippen, das ist hier der Punkt.

Braucht ein Fenster, steht darum in `conftest._BRAUCHT_GRAFIK`.
"""
import json
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
pytestmark = [pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut"),
              # EIN Betriebsmittel: die Systemzwischenablage, siehe conftest._SERIELL.
              pytest.mark.seriell]

KEY_UP, KEY_DOWN = 1, 2
RL_ENTER, RL_ESC, RL_SPACE = 257, 256, 32
RL_V, RL_Z, RL_Y = 86, 90, 89
RL_LCTRL, RL_LSHIFT = 341, 340

_KOPF = ('IMPORT "gui"\n'
         'SCREEN(400, 300, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 10, 10, 360, 260)\n')


def _lauf(tmp_path, src, frames=16, events=None):
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
    return [ln.rstrip() for ln in r.stdout.splitlines() if ln.strip()]


def _schleife(bilder, probe):
    return (f"DIM f AS INTEGER\nFOR f = 1 TO {bilder}\n    GUI_UPDATE()\n"
            f"    GUI_DRAW()\n{probe}    FLIP()\nNEXT\n")


def _tipp(frame, code, halten=None):
    ev = []
    if halten is not None:
        ev.append((frame, KEY_DOWN, halten))
    ev += [(frame, KEY_DOWN, code), (frame + 1, KEY_UP, code)]
    if halten is not None:
        ev.append((frame + 1, KEY_UP, halten))
    return ev


def _einfuegen(frame):
    return _tipp(frame, RL_V, RL_LCTRL)


# ---------------------------------------------------------------- Beschriftung
def test_umbruch_laesst_die_hoehe_wachsen(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM l AS GUI_WIDGET\n'
                'l = GUI_LABEL(w, "Ein Satz mit vielen Woertern, der nicht in eine Zeile passt", 10, 10)\n'
                'PRINT GUI_GET_H(l)\n'
                'GUI_SET_WRAP(l, 150)\n'
                'GUI_UPDATE()\n'
                'PRINT GUI_GET_W(l) : PRINT GUI_GET_H(l)\n'
                'GUI_SET_WRAP(l, 0)\n'
                'GUI_UPDATE()\n'
                'PRINT GUI_GET_H(l)\n', frames=1)
    h0, breite, h1, h2 = [int(x) for x in out]
    assert breite == 150, "die Umbruchbreite ist die Widget-Breite"
    assert h1 >= 3 * h0, f"mehrere Zeilen erwartet, Hoehe {h0} -> {h1}"
    assert h2 == h1, "ohne Umbruch bleibt die zuletzt gemessene Hoehe stehen"


def test_umbruch_nur_fuer_beschriftungen(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(w, "x", 10, 10, 80, 26)\n'
                'TRY\n    GUI_SET_WRAP(b, 100)\nCATCH e\n    PRINT e\nEND TRY\n', frames=1)
    assert "GUI_LABEL" in out[0]


def test_ausrichtung_wird_gespeichert_und_geprueft(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM l AS GUI_WIDGET\nl = GUI_LABEL(w, "rechts", 10, 10)\n'
                'GUI_SET_BOUNDS(l, 10, 10, 200, 16)\n'
                'GUI_SET_ALIGN(l, "rechts")\n'
                'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(w, "OK", 10, 40, 100, 26)\n'
                'GUI_SET_ALIGN(b, "links")\n'
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)\n'
                'PRINT GUI_TO_JSON(w2) = j\n'
                'TRY\n    GUI_SET_ALIGN(l, "oben")\nCATCH e\n    PRINT e\nEND TRY\n'
                'DIM s AS GUI_WIDGET : s = GUI_SLIDER(w, 10, 80, 100, 0, 1)\n'
                'TRY\n    GUI_SET_ALIGN(s, "mitte")\nCATCH e2\n    PRINT e2\nEND TRY\n', frames=1)
    assert out[0] == "TRUE", "Ausrichtung ueberlebt den Rundweg durch die Datei"
    assert "links, mitte oder rechts" in out[1]
    assert "Beschriftung, Knopf und Textfeld" in out[2]


# ---------------------------------------------------------------- Textfeld
_FELD = ('DIM tf AS GUI_WIDGET\ntf = GUI_TEXTINPUT(w, 10, 10, 200, 26)\n'
         'GUI_FOCUS(tf)\n')


def test_einfuegen_ueber_die_zwischenablage_geht_ueberhaupt(tmp_path):
    """Die Gegenprobe fuer alles Weitere: ohne Filter kommt an, was in der
    Zwischenablage liegt."""
    out = _lauf(tmp_path, _KOPF + _FELD + 'CLIPBOARD_SET("hallo")\n'
                + _schleife(10, "") + 'PRINT GUI_TEXT(tf)\n',
                frames=12, events=_einfuegen(3))
    assert out[-1] == "hallo"


def test_nur_lesen_nimmt_nichts_an(tmp_path):
    out = _lauf(tmp_path, _KOPF + _FELD + 'GUI_SET_TEXT(tf, "fest")\n'
                'GUI_TEXTINPUT_SET(tf, "nur_lesen", 1)\nCLIPBOARD_SET("hallo")\n'
                + _schleife(10, "") + 'PRINT GUI_TEXT(tf)\n',
                frames=12, events=_einfuegen(3))
    assert out[-1] == "fest"


def test_hoechstlaenge_schneidet_ab(tmp_path):
    out = _lauf(tmp_path, _KOPF + _FELD +
                'GUI_TEXTINPUT_SET(tf, "maxlaenge", 3)\nCLIPBOARD_SET("abcdef")\n'
                + _schleife(10, "") + 'PRINT GUI_TEXT(tf)\n',
                frames=12, events=_einfuegen(3))
    assert out[-1] == "abc", "beim Einfuegen wird abgeschnitten, nicht abgelehnt"


@pytest.mark.parametrize("modus,eingabe,erwartet", [
    (1, "42", "42"),
    (1, "-7", "-7"),
    (1, "12a3", ""),
    (1, "3.5", ""),
    (2, "3.14", "3.14"),
    (2, "3,14", "3,14"),
    (2, "3.1.4", ""),
])
def test_zahlenfilter(tmp_path, modus, eingabe, erwartet):
    out = _lauf(tmp_path, _KOPF + _FELD +
                f'GUI_TEXTINPUT_SET(tf, "zahlen", {modus})\nCLIPBOARD_SET("{eingabe}")\n'
                + _schleife(10, "") + 'PRINT "[" + GUI_TEXT(tf) + "]"\n',
                frames=12, events=_einfuegen(3))
    assert out[-1] == f"[{erwartet}]"


def test_passwort_verbirgt_nur_die_anzeige(tmp_path):
    """GUI_TEXT liefert den echten Text; in der Datei steht der Modus."""
    out = _lauf(tmp_path, _KOPF + _FELD +
                'GUI_TEXTINPUT_SET(tf, "passwort", 1)\nCLIPBOARD_SET("geheim")\n'
                + _schleife(10, "") + 'PRINT GUI_TEXT(tf)\n'
                'PRINT INSTR(GUI_TO_JSON(w), CHR$(34) + "passwort" + CHR$(34) + ": true") >= 0\n',
                frames=12, events=_einfuegen(3))
    assert out[-2] == "geheim"
    assert out[-1] == "TRUE"


def test_unbekannter_schluessel_nennt_die_gueltigen(tmp_path):
    out = _lauf(tmp_path, _KOPF + _FELD +
                'TRY\n    GUI_TEXTINPUT_SET(tf, "farbe", 1)\nCATCH e\n    PRINT e\nEND TRY\n', frames=1)
    assert "passwort, nur_lesen, maxlaenge, zahlen" in out[0]


def test_enter_meldet_sich_ein_bild_lang_und_ruft_den_handler(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'SUB fertig()\n    PRINT "handler"\nEND SUB\n'
                + _FELD + 'GUI_ON_ENTER(tf, fertig)\n'
                + _schleife(10, '    IF GUI_ENTERED(tf) THEN PRINT "enter " + STR$(f)\n'),
                frames=12, events=_tipp(4, RL_ENTER))
    treffer = [ln for ln in out if ln.startswith("enter ")]
    assert len(treffer) == 1, out
    assert out.count("handler") == 1


def test_strg_z_nimmt_einen_schritt_zurueck_und_strg_y_holt_ihn(tmp_path):
    """Zwei Einfuegungen mehr als 0,8 s auseinander sind zwei Schritte."""
    ev = (_einfuegen(3)
          + _einfuegen(70)                              # > 0,8 s spaeter
          + _tipp(90, RL_Z, RL_LCTRL)
          + _tipp(100, RL_Z, RL_LCTRL)
          + _tipp(110, RL_Y, RL_LCTRL))
    out = _lauf(tmp_path, _KOPF + _FELD + 'CLIPBOARD_SET("ab")\n'
                + _schleife(120, '    IF f = 85 OR f = 95 OR f = 105 OR f = 115 THEN PRINT "[" + GUI_TEXT(tf) + "]"\n'),
                frames=122, events=ev)
    assert out[-4:] == ["[abab]", "[ab]", "[]", "[ab]"]


def test_gui_set_text_leert_den_verlauf(tmp_path):
    """Ein vom Programm gesetzter Text ist kein Schritt des Nutzers -- Strg+Z
    danach darf nichts Aelteres zurueckholen."""
    ev = _einfuegen(3) + _tipp(60, RL_Z, RL_LCTRL)
    out = _lauf(tmp_path, _KOPF + _FELD + 'CLIPBOARD_SET("ab")\n'
                + _schleife(70, '    IF f = 30 THEN GUI_SET_TEXT(tf, "neu")\n'
                                '    IF f = 65 THEN PRINT "[" + GUI_TEXT(tf) + "]"\n'),
                frames=72, events=ev)
    assert out[-1] == "[neu]"


# ---------------------------------------------------------------- Standard-/Abbrechen-Knopf
_FORM = ('DIM tf AS GUI_WIDGET\ntf = GUI_TEXTINPUT(w, 10, 10, 200, 26)\n'
         'DIM ok AS GUI_WIDGET\nok = GUI_BUTTON(w, "OK", 10, 50, 80, 26)\n'
         'DIM ab AS GUI_WIDGET\nab = GUI_BUTTON(w, "Abbrechen", 100, 50, 100, 26)\n'
         'GUI_WINDOW_DEFAULT(w, ok)\nGUI_WINDOW_CANCEL(w, ab)\n')
_PROBE = ('    IF GUI_CLICKED(ok) THEN PRINT "ok " + STR$(f)\n'
          '    IF GUI_CLICKED(ab) THEN PRINT "ab " + STR$(f)\n')


def test_enter_im_textfeld_loest_den_standardknopf_aus(tmp_path):
    out = _lauf(tmp_path, _KOPF + _FORM + 'GUI_FOCUS(tf)\n' + _schleife(10, _PROBE),
                frames=12, events=_tipp(4, RL_ENTER))
    assert len([ln for ln in out if ln.startswith("ok ")]) == 1, out
    assert not [ln for ln in out if ln.startswith("ab ")]


def test_esc_loest_den_abbrechen_knopf_aus(tmp_path):
    out = _lauf(tmp_path, _KOPF + _FORM + 'GUI_FOCUS(tf)\n' + _schleife(10, _PROBE),
                frames=12, events=_tipp(4, RL_ESC))
    assert len([ln for ln in out if ln.startswith("ab ")]) == 1, out
    assert not [ln for ln in out if ln.startswith("ok ")]


def test_ein_knopf_mit_fokus_nimmt_enter_selbst(tmp_path):
    """Gegenprobe: Enter auf dem Abbrechen-Knopf drueckt DIESEN, nicht den
    Standardknopf -- sonst taeten zwei Knoepfe auf eine Taste."""
    out = _lauf(tmp_path, _KOPF + _FORM + 'GUI_FOCUS(ab)\n' + _schleife(10, _PROBE),
                frames=12, events=_tipp(4, RL_ENTER))
    assert len([ln for ln in out if ln.startswith("ab ")]) == 1, out
    assert not [ln for ln in out if ln.startswith("ok ")]


def test_ohne_fokus_gilt_der_standardknopf(tmp_path):
    out = _lauf(tmp_path, _KOPF + _FORM + _schleife(10, _PROBE),
                frames=12, events=_tipp(4, RL_ENTER))
    assert len([ln for ln in out if ln.startswith("ok ")]) == 1, out


def test_standardknopf_ueberlebt_die_datei(tmp_path):
    out = _lauf(tmp_path, _KOPF + _FORM +
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'DIM q AS STRING : q = CHR$(34)\n'
                'PRINT INSTR(j, q + "default_button" + q + ": 1") >= 0\n'
                'PRINT INSTR(j, q + "cancel_button" + q + ": 2") >= 0\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)\n'
                'PRINT INSTR(GUI_TO_JSON(w2), q + "cancel_button" + q + ": 2") >= 0\n', frames=1)
    assert out == ["TRUE", "TRUE", "TRUE"]


def test_nur_ein_knopf_desselben_fensters(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM tf AS GUI_WIDGET\ntf = GUI_TEXTINPUT(w, 10, 10, 200, 26)\n'
                'TRY\n    GUI_WINDOW_DEFAULT(w, tf)\nCATCH e\n    PRINT e\nEND TRY\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_WINDOW("B", 20, 20, 100, 100)\n'
                'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w2, "x", 5, 5, 40, 20)\n'
                'TRY\n    GUI_WINDOW_DEFAULT(w, b)\nCATCH e2\n    PRINT e2\nEND TRY\n'
                'GUI_WINDOW_DEFAULT(w, -1)\nPRINT "ok"\n', frames=1)
    assert "GUI_BUTTON" in out[0]
    assert "anderen Fenster" in out[1]
    assert out[2] == "ok"
