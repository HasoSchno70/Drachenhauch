"""Tastatur-Bedienung der `gui`-Widgets -- Tab-Fokus und Auslösen ohne Maus.

Bis 2026-08-30 lief der Tab-Zyklus nur über Textfelder; Knopf, Kästchen,
Klappliste und Baum waren ohne Maus nicht erreichbar. Diese Datei sichert
die Erweiterung ab.

Die Tasten werden ECHT eingespeist (Automation-Wiedergabe, gleiches Muster
wie `test_gebundene_methoden_gui.py` und `test_automation.py`). Ein Test,
der nur `GUI_FOCUS` aufruft und dann `GUI_FOCUSED` liest, würde die
Tab-Logik gar nicht anfassen -- er belegte nur, dass zwei Setter/Getter
zueinander passen.

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

# raylibs AutomationEventType-Nummern (rcore.c), wie in test_automation.py.
KEY_UP, KEY_DOWN = 1, 2

# raylib-Tastencodes (KeyboardKey), NICHT die pygame-Codes der GB-Ebene.
RL_TAB, RL_SPACE, RL_ENTER = 258, 32, 257
RL_RIGHT, RL_LEFT, RL_ARR_DOWN, RL_ARR_UP = 262, 263, 264, 265
RL_LSHIFT = 340

# Fenster aus dem Bild schieben: sonst überschreibt die ECHTE Mausposition
# die eingespeiste und ein Hover verstellt den Zustand (ausführlich begründet
# in test_automation.py).
_KOPF = ('IMPORT "gui"\n'
         'SCREEN(360, 240, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 10, 10, 300, 200)\n')


def _lauf(tmp_path, src, frames=16):
    """Programm laufen lassen und die Ausgabezeilen liefern.

    `assert returncode == 0, r.stderr` ist Pflicht: ohne Bildschirm bricht
    raylib beim Fenster ab, und nur wenn seine Meldung IM FEHLERTEXT steht,
    macht `conftest.pytest_runtest_makereport` daraus einen Skip statt eines
    Fehlschlags.
    """
    (tmp_path / "a.dh").write_text(src, encoding="utf-8")
    env = dict(os.environ, DHRT_FRAMES=str(frames))
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "a.dh")], capture_output=True,
                       text=True, encoding="utf-8", env=env, timeout=90, cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return [ln for ln in (r.stdout or "").splitlines()
            if not ln.startswith(("WARNING:", "INFO:", "TRACE:"))]


def _ereignisse(tmp_path, name, events):
    """Aufnahmedatei im raylib-Textformat schreiben (wie test_automation.py)."""
    zeilen = ["# Test-Aufnahme", f"c {len(events)}"]
    for frame, typ, *params in events:
        p = (list(params) + [0, 0, 0, 0])[:4]
        zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
    (tmp_path / name).write_text("\n".join(zeilen) + "\n", encoding="utf-8")


def _tipp(frame, code, halten=None):
    """Ein Tastendruck als (down, up)-Paar ab `frame`.

    Ein eingespeistes KEY_DOWN bleibt gedrückt, bis ein eigenes KEY_UP kommt
    -- ohne das feuert die Taste im Folgeframe erneut. `halten` legt einen
    Modifikator (z.B. Umschalt) um den Druck herum.
    """
    ev = []
    if halten is not None:
        ev.append((frame, KEY_DOWN, halten))
    ev.append((frame, KEY_DOWN, code))
    ev.append((frame + 1, KEY_UP, code))
    if halten is not None:
        ev.append((frame + 1, KEY_UP, halten))
    return ev


# Schleife, die die Wiedergabe abfährt. Die Bilder müssen zu den
# Ereignis-Frames passen, sonst läuft die Aufnahme ins Leere.
_SCHLEIFE = """
AUTOMATION_PLAY("ev.txt")
DIM f AS INTEGER
FOR f = 0 TO 11
    GUI_UPDATE()
    GUI_DRAW()
    FLIP()
NEXT
"""


def test_tab_springt_auf_den_knopf_leertaste_loest_aus(tmp_path):
    """Der Kern: ohne einen einzigen Mausklick einen Knopf auslösen.

    Vorher unmöglich -- der Tab-Zyklus enthielt nur Textfelder, ein Fenster
    ohne Textfeld war per Tastatur überhaupt nicht bedienbar.
    """
    _ereignisse(tmp_path, "ev.txt", _tipp(1, RL_TAB) + _tipp(4, RL_SPACE))
    zeilen = _lauf(tmp_path, _KOPF + """
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 10, 10, 80, 24)
DIM traf AS INTEGER
""" + _SCHLEIFE.replace("    FLIP()", "    FLIP()\n    IF GUI_CLICKED(b) THEN traf = traf + 1") + """
PRINT traf
""")
    assert zeilen and zeilen[-1] == "1", zeilen


def test_tab_ueberspringt_deko(tmp_path):
    """Label, Panel und Trennlinie fangen keinen Fokus.

    Sonst müsste man sich durch Beschriftungen hindurchtabben -- die
    häufigste Art, eine Tastaturnavigation unbrauchbar zu machen.
    """
    _ereignisse(tmp_path, "ev.txt", _tipp(1, RL_TAB))
    zeilen = _lauf(tmp_path, _KOPF + """
DIM l AS GUI_WIDGET
l = GUI_LABEL(w, "Name", 10, 10)
DIM p AS GUI_WIDGET
p = GUI_PANEL(w, 10, 30, 100, 40)
DIM s AS GUI_WIDGET
s = GUI_SEPARATOR(w, 10, 75, 100)
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 10, 90, 80, 24)
""" + _SCHLEIFE + """
PRINT IIF(GUI_FOCUSED() = b, "knopf", "falsch")
""")
    assert zeilen and zeilen[-1] == "knopf", zeilen


def test_tab_reihenfolge_und_shift_tab_rueckwaerts(tmp_path):
    """Vorwärts in Anlege-Reihenfolge, Umschalt+Tab zurück."""
    _ereignisse(tmp_path, "ev.txt",
                _tipp(1, RL_TAB) + _tipp(3, RL_TAB) + _tipp(5, RL_TAB, halten=RL_LSHIFT))
    zeilen = _lauf(tmp_path, _KOPF + """
DIM a AS GUI_WIDGET
a = GUI_BUTTON(w, "a", 10, 10, 60, 24)
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "b", 10, 40, 60, 24)
""" + _SCHLEIFE + """
' 1. Tab -> a, 2. Tab -> b, Umschalt+Tab -> zurueck auf a
PRINT IIF(GUI_FOCUSED() = a, "a", IIF(GUI_FOCUSED() = b, "b", "keins"))
""")
    assert zeilen and zeilen[-1] == "a", zeilen


def test_leertaste_schaltet_kaestchen_um(tmp_path):
    _ereignisse(tmp_path, "ev.txt", _tipp(1, RL_TAB) + _tipp(4, RL_SPACE))
    zeilen = _lauf(tmp_path, _KOPF + """
DIM c AS GUI_WIDGET
c = GUI_CHECKBOX(w, "aktiv", 10, 10)
""" + _SCHLEIFE + """
PRINT GUI_CHECKED(c)
""")
    assert zeilen and zeilen[-1] == "TRUE", zeilen


def test_pfeiltaste_verstellt_den_regler(tmp_path):
    """Ein Zwanzigstel des Bereichs je Druck -- 0..100 also +5."""
    _ereignisse(tmp_path, "ev.txt", _tipp(1, RL_TAB) + _tipp(4, RL_RIGHT))
    zeilen = _lauf(tmp_path, _KOPF + """
DIM s AS GUI_WIDGET
s = GUI_SLIDER(w, 10, 10, 120, 0, 100, 50)
""" + _SCHLEIFE + """
PRINT INT(GUI_VALUE(s))
""")
    assert zeilen and zeilen[-1] == "55", zeilen


def test_klappliste_mit_tastatur_aufklappen_und_waehlen(tmp_path):
    """Enter klappt auf, Pfeil-ab wählt weiter, Enter übernimmt."""
    _ereignisse(tmp_path, "ev.txt",
                _tipp(1, RL_TAB) + _tipp(3, RL_ENTER) + _tipp(5, RL_ARR_DOWN) + _tipp(7, RL_ENTER))
    zeilen = _lauf(tmp_path, _KOPF + """
DIM items AS ARRAY OF STRING
items = SPLIT$("rot|gruen|blau", "|")
DIM d AS GUI_WIDGET
d = GUI_DROPDOWN(w, 10, 10, 120, 24, items)
""" + _SCHLEIFE + """
PRINT GUI_DROPDOWN_TEXT(d)
""")
    assert zeilen and zeilen[-1] == "gruen", zeilen


def test_baum_pfeiltasten_klappen_und_waehlen(tmp_path):
    """Rechts klappt auf, Ab geht ins Kind -- wie in jedem Dateibaum."""
    _ereignisse(tmp_path, "ev.txt",
                _tipp(1, RL_TAB) + _tipp(3, RL_ARR_DOWN) + _tipp(5, RL_RIGHT) + _tipp(7, RL_ARR_DOWN))
    zeilen = _lauf(tmp_path, _KOPF + """
DIM t AS GUI_WIDGET
t = GUI_TREE(w, 10, 10, 200, 120)
DIM wurzel AS INTEGER
wurzel = GUI_TREE_ADD(t, -1, "Ordner")
DIM kind AS INTEGER
kind = GUI_TREE_ADD(t, wurzel, "Datei")
""" + _SCHLEIFE + """
PRINT GUI_TREE_LABEL(t, GUI_TREE_SELECTED(t))
""")
    assert zeilen and zeilen[-1] == "Datei", zeilen


def test_gui_focused_ohne_fokus_ist_minus_eins(tmp_path):
    zeilen = _lauf(tmp_path, _KOPF + """
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 10, 10, 80, 24)
PRINT GUI_FOCUSED()
""", frames=2)
    assert zeilen and zeilen[-1] == "-1", zeilen
