"""gui-Ausbau Punkt 4: Dialoge -- freie Knopfsaetze, GUI_PROMPT mit
Eingabefeld, GUI_DIALOG_TEXT, GUI_WINDOW_MODAL fuer eigene Fenster.

Wie in test_gui_dialog.py wird die MODALITAET am Klick geprueft, nicht am
Erscheinen. Die Lage der Knoepfe im zentrierten Dialog kennt kein Befehl --
der Test legt deshalb eine 1x1-Zeichenflaeche bei (0, 0) in den Dialog, die
Fenster- und Bildschirmkoordinaten zugleich liefert (dasselbe Muster wie in
den Piloten-Tests).

Braucht ein Fenster (`_BRAUCHT_GRAFIK`), speist Eingabe ein (seriell).
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
pytestmark = [pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut"),
              pytest.mark.seriell]

KEY_UP, KEY_DOWN = 1, 2
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7
RL_ENTER, RL_ESC, RL_V, RL_LCTRL = 257, 256, 86, 341

_KOPF = ('IMPORT "gui"\n'
         'SCREEN(520, 320, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n')
# Knoepfe sind die letzten Widgets des Dialogs (nach Beschriftungen und ggf.
# dem Eingabefeld); die Zeichenflaeche kommt danach dazu und stoert die
# Zaehlung nicht, weil ueber GUI_WINDOW_WIDGET vorher gelesen wird.
_GEO = ('DIM np AS GUI_WIDGET : np = GUI_CANVAS(d, 0, 0, 1, 1)\n'
        'GUI_UPDATE()\n'
        'DIM ox AS INTEGER : ox = GUI_CANVAS_X(np)\n'
        'DIM oy AS INTEGER : oy = GUI_CANVAS_Y(np)\n')


def _lauf(tmp_path, src, frames=16, events=None):
    if events is not None:
        ev = sorted(events, key=lambda e: e[0])
        zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
        for frame, typ, *params in ev:
            p = (list(params) + [0, 0, 0, 0])[:4]
            zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
        (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
    (tmp_path / "a.dh").write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "a.dh")], capture_output=True,
                       text=True, encoding="utf-8", errors="replace",
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), timeout=90, cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return [ln for ln in (r.stdout or "").splitlines()
            if ln.strip() and not ln.startswith(("WARNING:", "INFO:", "TRACE:"))]


def _schleife(bilder, probe):
    return (f"DIM f AS INTEGER\nFOR f = 1 TO {bilder}\n    GUI_UPDATE()\n"
            f"{probe}    GUI_DRAW()\n    FLIP()\nNEXT\n")


def _tipp(frame, code, *halten):
    ev = [(frame, KEY_DOWN, h) for h in halten]
    ev += [(frame, KEY_DOWN, code), (frame + 1, KEY_UP, code)]
    ev += [(frame + 1, KEY_UP, h) for h in halten]
    return ev


def _knopf_geo(tmp_path, stil, n_knopf):
    """Bildschirm-Mitte des n-ten Knopfs (0-basiert) eines Dialogs mit `stil`."""
    out = _lauf(tmp_path, _KOPF + f'DIM d AS GUI_WINDOW : d = GUI_DIALOG("F", "Text?", "{stil}")\n'
                + 'DIM k AS GUI_WIDGET : k = GUI_WINDOW_WIDGET(d, GUI_WINDOW_WIDGET_COUNT(d) - '
                + f'{n_knopf})\n' + _GEO
                + 'PRINT ox + GUI_GET_X(k) + GUI_GET_W(k) \\ 2 ; " " ; oy + GUI_GET_Y(k) + GUI_GET_H(k) \\ 2\n',
                frames=2)
    x, y = [int(v) for v in out[-1].split()]
    return x, y


# ---------------------------------------------------------------- freie Knopfsaetze
def test_eigene_knoepfe_antwort_ist_die_nummer(tmp_path):
    """Klick auf den zweiten von drei Knoepfen -> Antwort 2."""
    x, y = _knopf_geo(tmp_path, "Speichern|Verwerfen|Abbrechen", 2)   # zweiter = vorletzter -> Index -2
    ev = [(4, MOUSE_POSITION, x, y), (5, MOUSE_POSITION, x, y), (5, MOUSE_BUTTON_DOWN, 0), (6, MOUSE_BUTTON_UP, 0)]
    out = _lauf(tmp_path, _KOPF + 'DIM d AS GUI_WINDOW : d = GUI_DIALOG("F", "Text?", "Speichern|Verwerfen|Abbrechen")\n'
                'AUTOMATION_PLAY("ev.txt")\n'
                + _schleife(12, '    IF GUI_ANSWER(d) <> 0 THEN PRINT "antwort " + STR$(GUI_ANSWER(d))\n'),
                events=ev)
    assert out == ["antwort 2"]


def test_enter_ist_der_erste_esc_der_letzte(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM d AS GUI_WINDOW : d = GUI_DIALOG("F", "Text?", "Speichern|Verwerfen|Abbrechen")\n'
                'AUTOMATION_PLAY("ev.txt")\n'
                + _schleife(12, '    IF GUI_ANSWER(d) <> 0 THEN PRINT "antwort " + STR$(GUI_ANSWER(d))\n'),
                events=_tipp(3, RL_ESC))
    assert out == ["antwort 3"]
    out = _lauf(tmp_path, _KOPF + 'DIM d AS GUI_WINDOW : d = GUI_DIALOG("F", "Text?", "Speichern|Verwerfen|Abbrechen")\n'
                'AUTOMATION_PLAY("ev.txt")\n'
                + _schleife(12, '    IF GUI_ANSWER(d) <> 0 THEN PRINT "antwort " + STR$(GUI_ANSWER(d))\n'),
                events=_tipp(3, RL_ENTER))
    assert out == ["antwort 1"]


def test_janein_bleibt_wie_es_war(tmp_path):
    """Die alten Stile liefern weiter 1 und 2 -- und ESC ist Nein."""
    out = _lauf(tmp_path, _KOPF + 'DIM d AS GUI_WINDOW : d = GUI_DIALOG("F", "Text?", "janein")\n'
                'AUTOMATION_PLAY("ev.txt")\n'
                + _schleife(12, '    IF GUI_ANSWER(d) <> 0 THEN PRINT "antwort " + STR$(GUI_ANSWER(d))\n'),
                events=_tipp(3, RL_ESC))
    assert out == ["antwort 2"]


def test_zu_viele_oder_leere_knoepfe_sind_ein_fehler(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'TRY\n    GUI_DIALOG("F", "x", "a|b|c|d|e|f")\nCATCH e\n    PRINT e\nEND TRY\n'
                'TRY\n    GUI_DIALOG("F", "x", "a||b")\nCATCH e2\n    PRINT e2\nEND TRY\n'
                'TRY\n    GUI_DIALOG("F", "x", "vielleicht")\nCATCH e3\n    PRINT e3\nEND TRY\n', frames=1)
    assert "fuenf" in out[0]
    assert "ohne Beschriftung" in out[1]
    assert "Speichern|Verwerfen|Abbrechen" in out[2], "die Meldung zeigt die neue Form"


# ---------------------------------------------------------------- GUI_PROMPT
def test_prompt_liefert_den_text_mit_enter(tmp_path):
    """Tippen ueber die Zwischenablage (Strg+V); die Vorgabe ist markiert und
    wird ersetzt. Enter = OK, und der Text bleibt nach der Antwort lesbar."""
    out = _lauf(tmp_path, _KOPF + 'CLIPBOARD_SET("Hans")\n'
                'DIM d AS GUI_WINDOW : d = GUI_PROMPT("Name", "Wie heisst du?", "Anonym")\n'
                'PRINT GUI_DIALOG_TEXT(d)\n'
                'AUTOMATION_PLAY("ev.txt")\n'
                + _schleife(16, '    IF GUI_ANSWER(d) <> 0 THEN PRINT "antwort " + STR$(GUI_ANSWER(d)) + " " + GUI_DIALOG_TEXT(d)\n')
                + 'PRINT GUI_DIALOG_TEXT(d) ; " " ; GUI_MODAL()\n',
                frames=20, events=_tipp(3, RL_V, RL_LCTRL) + _tipp(9, RL_ENTER))
    assert out == ["Anonym", "antwort 1 Hans", "Hans FALSE"]


def test_prompt_esc_ist_abbrechen(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM d AS GUI_WINDOW : d = GUI_PROMPT("Name", "Wie heisst du?")\n'
                'AUTOMATION_PLAY("ev.txt")\n'
                + _schleife(12, '    IF GUI_ANSWER(d) <> 0 THEN PRINT "antwort " + STR$(GUI_ANSWER(d))\n'),
                events=_tipp(3, RL_ESC))
    assert out == ["antwort 2"]


def test_prompt_mit_eigenen_knoepfen(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM d AS GUI_WINDOW : d = GUI_PROMPT("Name", "Wie?", "", "Weiter|Ueberspringen|Abbruch")\n'
                'AUTOMATION_PLAY("ev.txt")\n'
                + _schleife(12, '    IF GUI_ANSWER(d) <> 0 THEN PRINT "antwort " + STR$(GUI_ANSWER(d))\n'),
                events=_tipp(3, RL_ESC))
    assert out == ["antwort 3"]


def test_dialog_text_nur_bei_prompt(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM d AS GUI_WINDOW : d = GUI_DIALOG("F", "x")\n'
                'TRY\n    PRINT GUI_DIALOG_TEXT(d)\nCATCH e\n    PRINT e\nEND TRY\n'
                'DIM w AS GUI_WINDOW : w = GUI_WINDOW("W", 10, 10, 100, 80)\n'
                'TRY\n    PRINT GUI_DIALOG_TEXT(w)\nCATCH e2\n    PRINT e2\nEND TRY\n', frames=1)
    assert "kein Eingabefeld" in out[0]
    assert "kein Dialog" in out[1]


# ---------------------------------------------------------------- GUI_WINDOW_MODAL
_HINTEN = ('DIM w AS GUI_WINDOW : w = GUI_WINDOW("Hinten", 4, 4, 200, 120)\n'
           'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, "los", 8, 8, 90, 26)\n'
           'DIM m AS GUI_WINDOW : m = GUI_WINDOW("Modal", 250, 150, 200, 120)\n'
           'DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(m, "zu", 8, 8, 90, 26)\n')
_KLICK_HINTEN = [(2, MOUSE_POSITION, 60, 50), (3, MOUSE_POSITION, 60, 50), (3, MOUSE_BUTTON_DOWN, 0), (4, MOUSE_BUTTON_UP, 0)]


def test_eigenes_fenster_modal_verschluckt_klicks_daneben(tmp_path):
    out = _lauf(tmp_path, _KOPF + _HINTEN + 'GUI_WINDOW_MODAL(m, TRUE)\n'
                'PRINT GUI_MODAL()\nAUTOMATION_PLAY("ev.txt")\n'
                + _schleife(10, '    IF GUI_CLICKED(b) THEN PRINT "traf"\n'),
                events=_KLICK_HINTEN)
    assert out == ["TRUE"], "der Klick auf den Hintergrund darf nicht ankommen"


def test_gegenprobe_ohne_modal_trifft_der_klick(tmp_path):
    out = _lauf(tmp_path, _KOPF + _HINTEN + 'AUTOMATION_PLAY("ev.txt")\n'
                + _schleife(10, '    IF GUI_CLICKED(b) THEN PRINT "traf"\n'),
                events=_KLICK_HINTEN)
    assert out == ["traf"]


def test_modal_aus_gibt_frei_und_ein_knopf_im_fenster_beendet_es_nicht(tmp_path):
    """Anders als ein Dialog: ein Klick auf einen Knopf im modalen EIGENEN
    Fenster ist ein Klick, kein Schliessen. Erst GUI_WINDOW_MODAL(m, FALSE)
    gibt frei."""
    out = _lauf(tmp_path, _KOPF + _HINTEN + 'GUI_WINDOW_MODAL(m, TRUE)\n'
                'GUI_FOCUS(ok)\nAUTOMATION_PLAY("ev.txt")\n'
                + _schleife(14, '    IF GUI_CLICKED(ok) THEN PRINT "zu " + STR$(GUI_MODAL())\n'
                                '    IF f = 8 THEN GUI_WINDOW_MODAL(m, FALSE)\n'
                                '    IF f = 12 THEN PRINT GUI_MODAL()\n'),
                events=_tipp(3, RL_ENTER))
    assert out == ["zu TRUE", "FALSE"]


def test_ausblenden_gibt_die_modalitaet_frei(tmp_path):
    out = _lauf(tmp_path, _KOPF + _HINTEN + 'GUI_WINDOW_MODAL(m, TRUE)\n'
                + _schleife(6, '    IF f = 2 THEN GUI_WINDOW_VISIBLE(m, FALSE)\n'
                               '    IF f = 5 THEN PRINT GUI_MODAL()\n'),
                frames=8)
    assert out == ["FALSE"]
