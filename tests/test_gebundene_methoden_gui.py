"""Gebundene Methoden am `gui`-Modul -- der Teil, der ein Fenster braucht.

Getrennt von `test_gebundene_methoden.py`, weil `dhrt` sich ohne raylib bauen
laesst (`default = []`) und die posix-CI genau so baut. Diese Datei steht
darum in `conftest._BRAUCHT_GRAFIK` und wird unter `DHRT_OHNE_GRAFIK`
uebersprungen; die 11 rein sprachlichen Tests der Schwesterdatei laufen
weiter ueberall.

Der Klick wird ECHT eingespeist (Automation-Wiedergabe). Ohne das waere nur
belegt, dass ein Rueckruf angenommen wird, nicht dass er auch feuert.
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

pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

# raylibs AutomationEventType-Nummern (rcore.c), wie in test_automation.py.
_MOUSE_BUTTON_UP, _MOUSE_BUTTON_DOWN, _MOUSE_POSITION = 5, 6, 7

# Fenster aus dem Bild schieben, bevor irgendetwas abgespielt wird -- sonst
# ueberschreibt die ECHTE Mausposition die eingespeiste (ausfuehrlich
# begruendet in test_automation.py).
_KOPF = ('IMPORT "gui"\n'
         'SCREEN(320, 200, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n')

_AUFBAU = ('DIM w AS GUI_WINDOW\n'
           'w = GUI_WINDOW("T", 10, 10, 200, 100)\n'
           'DIM b AS GUI_WIDGET\n'
           'b = GUI_BUTTON(w, "ok", 10, 10, 80, 24)\n')


def _lauf(tmp_path, src, frames=12):
    """Programm laufen lassen und die Ausgabezeilen liefern.

    `assert returncode == 0, r.stderr` ist hier Pflicht und keine Kosmetik:
    auf einer Maschine ohne Bildschirm bricht raylib beim Fenster ab, und nur
    wenn seine Meldung IM FEHLERTEXT steht, macht der Haken in
    `conftest.pytest_runtest_makereport` daraus einen Skip statt eines
    Fehlschlags. Wer die Ausgabe stumm weiterverarbeitet, bekommt stattdessen
    ein nichtssagendes IndexError -- und die CI wird rot, wo sie
    ueberspringen sollte.
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
    (tmp_path / name).write_text(chr(10).join(zeilen) + chr(10), encoding="utf-8")


def _knopf_mitte(tmp_path):
    """Bildschirm-Mitte des Knopfes SUCHEN statt sie auszurechnen.

    Wo ein Widget landet, haengt an der Titelleisten-Hoehe und den
    Innenabstaenden des Themas -- interne Masse, die sich aendern duerfen,
    ohne dass dieser Test etwas damit zu tun haette. `GUI_HIT_TEST` fragt
    dieselbe Geometrie, die auch der Klick benutzt.
    """
    zeilen = _lauf(tmp_path, _KOPF + _AUFBAU + """
DIM x AS INTEGER
DIM y AS INTEGER
DIM fertig AS BOOLEAN
FOR y = 0 TO 199
  FOR x = 0 TO 319
    IF GUI_HIT_TEST(x, y) = b AND NOT fertig THEN
      PRINT STR$(x + GUI_GET_W(b) / 2); " "; STR$(y + GUI_GET_H(b) / 2)
      fertig = TRUE
    END IF
  NEXT
NEXT
""", frames=2)
    assert zeilen, "GUI_HIT_TEST fand den Knopf nirgends"
    mx, my = zeilen[0].split()
    return int(float(mx)), int(float(my))


def test_gui_speichert_gebundenen_handler_nicht(tmp_path):
    """Eine gebundene Methode kann in keiner `.dhform` stehen: beim Laden gibt
    es das Objekt nicht. Ihren blossen Namen zu schreiben waere eine Luege --
    er wuerde als freie Funktion gedeutet."""
    zeilen = _lauf(tmp_path, """
IMPORT "gui"
CLASS S
    SUB klick()
    END SUB
END CLASS
SUB frei()
END SUB
SCREEN(320, 200)
DIM s AS S
s = NEW S()
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 10, 10, 200, 100)
DIM a AS GUI_WIDGET
a = GUI_BUTTON(w, "gebunden", 10, 10, 80, 24)
GUI_ON_CLICK(a, s.klick)
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "frei", 10, 40, 80, 24)
GUI_ON_CLICK(b, frei)
PRINT GUI_TO_JSON(w)
""", frames=1)
    # GUI_TO_JSON gibt mehrzeilig aus -- ab der ersten Klammer alles nehmen.
    text = chr(10).join(zeilen)
    assert "{" in text, f"kein JSON in der Ausgabe: {zeilen!r}"
    roh = json.loads(text[text.index("{"):])
    nach_text = {wd["text"]: wd.get("on_click") for wd in roh["widgets"]}
    assert nach_text["gebunden"] is None
    assert nach_text["frei"] == "frei"


def test_gui_klick_ruft_gebundene_methode(tmp_path):
    """Der Kern: ein Knopf ruft eine Methode AUF EINER INSTANZ."""
    mx, my = _knopf_mitte(tmp_path)
    _ereignisse(tmp_path, "klick.txt", [
        (0, _MOUSE_POSITION, mx, my),
        (1, _MOUSE_BUTTON_DOWN, 0),
        (2, _MOUSE_BUTTON_UP, 0),
    ])
    zeilen = _lauf(tmp_path, _KOPF + """
CLASS Steuerung
    DIM zahl AS INTEGER
    SUB klick()
        Self.zahl = Self.zahl + 1
    END SUB
END CLASS
DIM s AS Steuerung
s = NEW Steuerung()
""" + _AUFBAU + """
GUI_ON_CLICK(b, s.klick)
AUTOMATION_PLAY("klick.txt")
DIM f AS INTEGER
FOR f = 0 TO 7
    GUI_UPDATE()
    GUI_DRAW()
    FLIP()
NEXT
PRINT s.zahl
""", frames=10)
    assert zeilen and zeilen[-1] == "1", zeilen
