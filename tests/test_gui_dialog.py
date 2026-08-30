"""`GUI_DIALOG` -- modales Meldungs-/Rückfragefenster im eigenen Fenster.

Abzugrenzen von `GUI_MESSAGE`/`GUI_CONFIRM`: das sind die **nativen,
blockierenden** OS-Kästen (rfd). `GUI_DIALOG` ist der Dialog im Stil der
eigenen Oberfläche — er blockiert nicht, sondern wird wie alles andere in
`gui` gepollt (`GUI_ANSWER`).

Der Kern, den diese Datei absichert, ist die **Modalität**: dass ein Klick
neben den Dialog wirklich verschluckt wird. Ein Test, der nur prüft, ob der
Dialog erscheint, würde genau das nicht bemerken — und ein Dialog, neben dem
man weiterklicken kann, ist keiner.
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
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7
RL_SPACE = 32

# Fenster aus dem Bild schieben, damit die ECHTE Maus die eingespeiste nicht
# überschreibt (ausführlich begründet in test_automation.py).
_KOPF = ('IMPORT "gui"\n'
         'SCREEN(520, 320, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n')

_SCHLEIFE = """
DIM f AS INTEGER
FOR f = 0 TO 11
    GUI_UPDATE()
    GUI_DRAW()
    FLIP()
NEXT
"""


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


def _ereignisse(tmp_path, events):
    zeilen = ["# Test-Aufnahme", f"c {len(events)}"]
    for frame, typ, *params in events:
        p = (list(params) + [0, 0, 0, 0])[:4]
        zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
    (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")


def test_dialog_steht_und_sperrt(tmp_path):
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM d AS GUI_WINDOW
d = GUI_DIALOG("Frage", "Wirklich?", "janein")
GUI_UPDATE()
PRINT GUI_MODAL(); " "; GUI_ANSWER(d)
''', frames=2)
    assert zeilen and zeilen[-1].split() == ["TRUE", "0"], zeilen


def test_leertaste_beantwortet_mit_ja(tmp_path):
    """Der Fokus liegt beim Öffnen auf dem ersten Knopf -- ein Dialog soll
    sofort mit der Tastatur zu beantworten sein, ohne erst Tab zu drücken."""
    _ereignisse(tmp_path, [(1, KEY_DOWN, RL_SPACE), (2, KEY_UP, RL_SPACE)])
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM d AS GUI_WINDOW
d = GUI_DIALOG("Frage", "Wirklich?", "janein")
AUTOMATION_PLAY("ev.txt")
DIM antwort AS INTEGER
DIM f AS INTEGER
FOR f = 0 TO 11
    GUI_UPDATE()
    IF GUI_ANSWER(d) <> 0 THEN antwort = GUI_ANSWER(d)
    GUI_DRAW()
    FLIP()
NEXT
PRINT antwort; " "; GUI_MODAL()
''')
    assert zeilen and zeilen[-1].split() == ["1", "FALSE"], zeilen


def test_antwort_gilt_nur_ein_bild(tmp_path):
    """Wie `GUI_CLICKED`: eine Antwort ist ein Ereignis, kein Zustand. Sonst
    liefe die Verzweigung, die daran hängt, in jedem folgenden Bild erneut."""
    _ereignisse(tmp_path, [(1, KEY_DOWN, RL_SPACE), (2, KEY_UP, RL_SPACE)])
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM d AS GUI_WINDOW
d = GUI_DIALOG("Frage", "Wirklich?", "janein")
AUTOMATION_PLAY("ev.txt")
DIM wie_oft AS INTEGER
DIM f AS INTEGER
FOR f = 0 TO 11
    GUI_UPDATE()
    IF GUI_ANSWER(d) = 1 THEN wie_oft = wie_oft + 1
    GUI_DRAW()
    FLIP()
NEXT
PRINT wie_oft
''')
    assert zeilen and zeilen[-1].strip() == "1", zeilen


# Ein Punkt weit links oben -- im Hintergrundfenster, sicher ausserhalb des
# mittig zentrierten Dialogs (520x320-Schirm).
_HINTERGRUND = '''
DIM w AS GUI_WINDOW
w = GUI_WINDOW("Hinten", 4, 4, 200, 120)
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "los", 8, 8, 90, 26)
'''
_KLICK = [(0, MOUSE_POSITION, 60, 50), (1, MOUSE_BUTTON_DOWN, 0), (2, MOUSE_BUTTON_UP, 0)]

_ZAEHLEN = '''
AUTOMATION_PLAY("ev.txt")
DIM traf AS INTEGER
DIM f AS INTEGER
FOR f = 0 TO 11
    GUI_UPDATE()
    IF GUI_CLICKED(b) THEN traf = traf + 1
    GUI_DRAW()
    FLIP()
NEXT
PRINT traf
'''


def test_klick_trifft_den_hintergrund_ohne_dialog(tmp_path):
    """Gegenprobe zum nächsten Test: OHNE Dialog löst genau dieser Klick den
    Knopf aus. Ohne diesen Nachweis könnte der Test unten auch dann grün
    sein, wenn die Klickstelle einfach danebenliegt."""
    _ereignisse(tmp_path, _KLICK)
    zeilen = _lauf(tmp_path, _KOPF + _HINTERGRUND + _ZAEHLEN)
    assert zeilen and zeilen[-1].strip() == "1", zeilen


def test_dialog_verschluckt_klicks_daneben(tmp_path):
    """Der Kern der Modalität."""
    _ereignisse(tmp_path, _KLICK)
    zeilen = _lauf(tmp_path, _KOPF + _HINTERGRUND + '''
DIM d AS GUI_WINDOW
d = GUI_DIALOG("Frage", "Wirklich?", "janein")
''' + _ZAEHLEN)
    assert zeilen and zeilen[-1].strip() == "0", zeilen


def test_answer_auf_normales_fenster_ist_fehler(tmp_path):
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM w AS GUI_WINDOW
w = GUI_WINDOW("Normal", 10, 10, 100, 80)
TRY
    PRINT GUI_ANSWER(w)
CATCH e
    PRINT "abgelehnt"
END TRY
''', frames=2)
    assert zeilen and zeilen[-1].strip() == "abgelehnt", zeilen


def test_unbekannter_stil_ist_fehler(tmp_path):
    zeilen = _lauf(tmp_path, _KOPF + '''
TRY
    DIM d AS GUI_WINDOW
    d = GUI_DIALOG("T", "x", "vielleicht")
    PRINT "kein Fehler"
CATCH e
    PRINT "abgelehnt"
END TRY
''', frames=2)
    assert zeilen and zeilen[-1].strip() == "abgelehnt", zeilen
