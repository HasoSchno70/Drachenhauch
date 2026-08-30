"""`GUI_SCALE` -- Anzeige-Massstab für hochauflösende Bildschirme.

Der Massstab multipliziert jede Länge, die in die GUI hineingeht: Fenster-
und Widget-Geometrie beim Anlegen, die Chrome-Metriken und die Schrift.
Nach aussen bleibt alles **logisch** -- `GUI_GET_X` liefert die Zahl zurück,
die das Programm hineingegeben hat, und ein gespeichertes `.dhform`
beschreibt weiterhin das Layout, nicht die Anzeige. Einzige Ausnahme:
`GUI_HIT_TEST` spricht Bildschirm-Pixel, weil es eine Frage über den
Bildschirm beantwortet (die Maus liefert nichts anderes).

Genau diese Trennung prüfen die Tests hier -- sie ist der ganze Kniff, und
sie ist die Stelle, an der ein Fehler nicht auffiele, bis jemand eine Form
zweimal speichert und sie doppelt so gross wiederfindet.
"""
import json

import pytest


def test_massstab_wirkt_erst_auf_spaeter_angelegtes(run_gb):
    """Ein Wechsel bei bestehenden Fenstern ist ein FEHLER, keine Halbheit.

    Schon angelegte Widgets nachträglich umzurechnen ginge nur näherungsweise
    (jede Runde neue Rundungsfehler); eine halb skalierte Oberfläche wäre
    schlimmer als eine klare Absage.
    """
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 0, 0, 100, 100)
TRY
    GUI_SCALE(2.0)
    PRINT "kein Fehler"
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


def test_massstab_ausserhalb_der_grenzen(run_gb):
    out = run_gb('''
IMPORT "gui"
TRY
    GUI_SCALE(9.0)
    PRINT "kein Fehler"
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


def test_massstab_lesbar(run_gb):
    out = run_gb('''
IMPORT "gui"
PRINT GUI_SCALE_GET()
GUI_SCALE(1.5)
PRINT GUI_SCALE_GET()
''')
    assert out.split() == ["1.0", "1.5"]


def test_getter_bleiben_logisch(run_gb):
    """`GUI_GET_*` liefert, was hineingegeben wurde -- sonst wäre
    `GUI_SET_BOUNDS(w, GUI_GET_X(w) + 10, ...)` bei Massstab 2 ein Sprung um
    das Doppelte."""
    out = run_gb('''
IMPORT "gui"
GUI_SCALE(2.0)
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 10, 20, 300, 200)
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 15, 25, 80, 24)
PRINT GUI_GET_X(b); " "; GUI_GET_Y(b); " "; GUI_GET_W(b); " "; GUI_GET_H(b)
PRINT GUI_WINDOW_GET_X(w); " "; GUI_WINDOW_GET_W(w)
GUI_SET_BOUNDS(b, 15, 25, 90, 24)
PRINT GUI_GET_W(b)
''')
    zeilen = out.strip().splitlines()
    assert zeilen[0].split() == ["15", "25", "80", "24"]
    assert zeilen[1].split() == ["10", "300"]
    assert zeilen[2].strip() == "90"


def test_hit_test_spricht_bildschirm_pixel(run_gb):
    """Der Beweis, dass INNEN wirklich skaliert wird.

    `GUI_HIT_TEST` fragt dieselbe Geometrie wie ein Mausklick. Bei Massstab 2
    muss ein Punkt getroffen werden, der beim doppelten Abstand vom
    Fensterursprung liegt -- am einfachen nicht.
    """
    src = '''
IMPORT "gui"
%s
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 0, 0, 400, 300)
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 40, 40, 60, 20)
' Mitte des Knopfes in LOGISCHEN Koordinaten, plus Titelleiste
DIM x AS INTEGER
DIM y AS INTEGER
DIM getroffen AS INTEGER
FOR y = 0 TO 299
  FOR x = 0 TO 399
    IF GUI_HIT_TEST(x, y) = b AND getroffen = 0 THEN
      PRINT x; " "; y
      getroffen = 1
    END IF
  NEXT
NEXT
'''
    eins = run_gb(src % "").strip().split()
    zwei = run_gb(src % "GUI_SCALE(2.0)").strip().split()
    assert eins and zwei, (eins, zwei)
    # Linke obere Ecke des Knopfes: bei Massstab 2 doppelt so weit vom
    # Fensterursprung entfernt (Titelleiste skaliert mit).
    assert int(zwei[0]) == 2 * int(eins[0]), (eins, zwei)
    assert int(zwei[1]) == 2 * int(eins[1]), (eins, zwei)


def test_gespeicherte_form_ist_massstabsfrei(run_gb):
    """Die wichtigste Zusage: `GUI_TO_JSON` schreibt dasselbe, egal bei
    welchem Massstab. Ohne das wüchse eine Form bei jedem Öffnen und
    Speichern um den Faktor weiter."""
    src = '''
IMPORT "gui"
%s
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 10, 20, 300, 200)
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 15, 25, 80, 24)
DIM t AS GUI_WIDGET
t = GUI_TABLE(w, 10, 60, 200, 100)
PRINT GUI_TO_JSON(w)
'''

    def json_von(vorspann):
        text = run_gb(src % vorspann)
        return json.loads(text[text.index("{"):])

    assert json_von("") == json_von("GUI_SCALE(2.0)")


def test_geladene_form_waechst_mit(run_gb):
    """Umgekehrt: eine bei Massstab 1 gebaute Form wird bei Massstab 2
    einfach grösser gezeichnet -- ihre logischen Zahlen bleiben gleich."""
    out = run_gb('''
IMPORT "gui"
DIM roh AS STRING
roh = "{""title"":""T"",""x"":10,""y"":20,""w"":300,""h"":200,""widgets"":" + _
      "[{""kind"":""button"",""x"":15,""y"":25,""w"":80,""h"":24,""text"":""ok""}]}"
GUI_SCALE(2.0)
DIM w AS GUI_WINDOW
w = GUI_FROM_JSON(roh)
DIM b AS GUI_WIDGET
b = GUI_WINDOW_WIDGET(w, 0)
PRINT GUI_GET_X(b); " "; GUI_GET_W(b)
''')
    assert out.strip().split() == ["15", "80"]
