"""Gebundene Methoden als FUNCREF (`f = spieler.tick`).

Eine Methode OHNE Klammern ist ein FUNCREF-Wert, der seine Instanz
mittraegt. Damit koennen die Rueckruf-Schnittstellen der Laufzeit
(GUI_ON_*, TIMER_*, SORT) auf ein Objekt zeigen, statt eine freie SUB plus
globale Variable zu verlangen.

Der GUI-Teil unten spielt einen echten Klick ueber die Automation-Wiedergabe
ein -- ohne das waere nur belegt, dass der Rueckruf ANGENOMMEN wird, nicht
dass er auch feuert.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

from drachenhauch.errors import DHRuntimeError

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()

ZAEHLER = """
CLASS Zaehler
    DIM stand AS INTEGER
    SUB tick()
        Self.stand = Self.stand + 1
    END SUB
    FUNCTION mal(x AS INTEGER) AS INTEGER
        RETURN x * Self.stand
    END FUNCTION
END CLASS
"""


def test_gebundene_sub_wirkt_auf_ihre_instanz(run_gb):
    out = run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
DIM f AS FUNCREF
f = z.tick
f()
f()
PRINT z.stand
""")
    assert out == "2\n"


def test_gebundene_funktion_mit_argument_und_rueckgabe(run_gb):
    out = run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
z.stand = 3
DIM f AS FUNCREF
f = z.mal
PRINT f(7)
""")
    assert out == "21\n"


def test_zwei_instanzen_bleiben_getrennt(run_gb):
    out = run_gb(ZAEHLER + """
DIM a AS Zaehler
DIM b AS Zaehler
a = NEW Zaehler()
b = NEW Zaehler()
DIM fa AS FUNCREF
DIM fb AS FUNCREF
fa = a.tick
fb = b.tick
fa()
fa()
fb()
PRINT a.stand; " "; b.stand
""")
    assert out == "2 1\n"


def test_ueberschriebene_methode_gewinnt(run_gb):
    """Aufgeloest wird beim AUFRUF, nicht beim Binden -- eine als Elternklasse
    gehaltene Instanz ruft trotzdem die Ueberschreibung des Kindes."""
    out = run_gb("""
CLASS Tier
    SUB laut()
        PRINT "..."
    END SUB
END CLASS
CLASS Hund EXTENDS Tier
    SUB laut()
        PRINT "Wuff"
    END SUB
END CLASS
DIM t AS Tier
t = NEW Hund()
DIM f AS FUNCREF
f = t.laut
f()
""")
    assert out == "Wuff\n"


def test_ist_nach_aussen_ein_funcref(run_gb):
    out = run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
DIM f AS FUNCREF
f = z.tick
PRINT TYPEOF(f)
PRINT f
""")
    assert out == "FUNCREF\n<FUNCREF zaehler.tick>\n"


def test_gleichheit_vergleicht_instanz_und_methode(run_gb):
    out = run_gb(ZAEHLER + """
DIM a AS Zaehler
DIM b AS Zaehler
a = NEW Zaehler()
b = NEW Zaehler()
PRINT a.tick = a.tick
PRINT a.tick = b.tick
PRINT a.tick = a.mal
""")
    assert out == "TRUE\nFALSE\nFALSE\n"


def test_unbekannter_name_bleibt_ein_fehler(run_gb):
    with pytest.raises(DHRuntimeError, match="existiert nicht"):
        run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
DIM f AS FUNCREF
f = z.gibtsnicht
""")


def test_feld_gewinnt_vor_methode(run_gb):
    """Heisst ein Feld wie eine Methode, liefert der Zugriff weiterhin das
    Feld -- sonst haette diese Erweiterung bestehenden Code umgedeutet."""
    out = run_gb("""
CLASS K
    DIM wert AS INTEGER
    SUB wert2()
    END SUB
END CLASS
DIM k AS K
k = NEW K()
k.wert = 42
PRINT k.wert
""")
    assert out == "42\n"


def test_timer_ruft_gebundene_methode(run_gb):
    out = run_gb(ZAEHLER + """
IMPORT "timer"
DIM z AS Zaehler
z = NEW Zaehler()
TIMER_AFTER(0, z.tick)
TIMER_UPDATE()
PRINT z.stand
""")
    assert out == "1\n"


def test_sort_mit_gebundenem_vergleicher(run_gb):
    out = run_gb("""
CLASS Regel
    DIM absteigend AS BOOLEAN
    FUNCTION cmp(a AS INTEGER, b AS INTEGER) AS INTEGER
        IF Self.absteigend THEN RETURN b - a
        RETURN a - b
    END FUNCTION
END CLASS
DIM r AS Regel
r = NEW Regel()
r.absteigend = TRUE
DIM a AS ARRAY OF INTEGER
a = [3, 1, 2]
SORT(a, r.cmp)
PRINT a[0]; a[1]; a[2]
""")
    assert out == "321\n"


def test_task_start_lehnt_gebundene_methode_ab(run_gb):
    """Ein Auftrag laeuft in einem eigenen Prozess -- die Instanz gibt es dort
    nicht. Das muss klar gesagt werden statt still ins Leere zu laufen."""
    with pytest.raises(DHRuntimeError, match="an ein Objekt gebunden"):
        run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
TASK_START(z.tick)
""")


# --------------------------------------------------------------- GUI

pytestmark_gui = pytest.mark.skipif(_DHRT is None, reason="dhrt nicht gebaut")


def _lauf(tmp_path, src, frames=12, aufnahme=None):
    (tmp_path / "a.dh").write_text(src, encoding="utf-8")
    env = dict(os.environ, DHRT_FRAMES=str(frames))
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "a.dh")], capture_output=True,
                       text=True, encoding="utf-8", env=env, timeout=90, cwd=str(tmp_path))
    return [ln for ln in (r.stdout or "").splitlines()
            if not ln.startswith(("WARNING:", "INFO:", "TRACE:"))]


@pytest.mark.skipif(_DHRT is None, reason="dhrt nicht gebaut")
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
    roh = json.loads(text[text.index("{"):])
    nach_text = {wd["text"]: wd.get("on_click") for wd in roh["widgets"]}
    assert nach_text["gebunden"] is None
    assert nach_text["frei"] == "frei"


# raylibs AutomationEventType-Nummern (rcore.c), wie in test_automation.py.
_MOUSE_BUTTON_UP, _MOUSE_BUTTON_DOWN, _MOUSE_POSITION = 5, 6, 7

# Fenster aus dem Bild schieben, bevor irgendetwas abgespielt wird -- sonst
# ueberschreibt die ECHTE Mausposition die eingespeiste (siehe die
# ausfuehrliche Begruendung in test_automation.py).
_KOPF = ('IMPORT "gui"\n'
         'SCREEN(320, 200, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n')

_AUFBAU = ('DIM w AS GUI_WINDOW\n'
           'w = GUI_WINDOW("T", 10, 10, 200, 100)\n'
           'DIM b AS GUI_WIDGET\n'
           'b = GUI_BUTTON(w, "ok", 10, 10, 80, 24)\n')


def _ereignisse(tmp_path, name, events):
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
    mx, my = zeilen[0].split()
    return int(float(mx)), int(float(my))


@pytest.mark.skipif(_DHRT is None, reason="dhrt nicht gebaut")
def test_gui_klick_ruft_gebundene_methode(tmp_path):
    """Der Kern von Punkt 1: ein Knopf ruft eine Methode AUF EINER INSTANZ.

    Der Klick wird echt eingespeist (Automation-Wiedergabe) -- damit ist
    belegt, dass der Rueckruf feuert, nicht nur dass er angenommen wird.
    """
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
    assert zeilen[-1] == "1", zeilen
