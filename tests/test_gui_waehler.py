"""Farbwähler (`GUI_COLORPICKER`) und Datumswähler (`GUI_DATEPICKER`).

Die Rechenkerne prüfen Rust-`#[test]`s ohne Fenster: `farbraum.rs` (der
RGB→HSV→RGB-Rundweg über 4096 Farben) und `kalender.rs` (Schaltjahre,
Monatslängen, Wochentag nach Zeller). Hier steht, was über die GB-Grenze
geht -- und die eine Zusage, die man leicht bricht: dass ein gesetzter Wert
unverändert wieder herauskommt.

Die Bedienung mit Maus und Tastatur steht in `test_gui_tastatur.py`, weil
sie echte Eingaben braucht.
"""

_FENSTER = ('IMPORT "gui"\n'
            'DIM w AS GUI_WINDOW\n'
            'w = GUI_WINDOW("T", 0, 0, 400, 300)\n')


# --- Farbwähler ------------------------------------------------------------

def test_farbe_kommt_unveraendert_zurueck(run_gb):
    """Der Rundweg geht über HSV. Rundet er, kommt eine andere Farbe heraus
    als die gesetzte -- und ein Werkzeug, das die eigene Farbe verfälscht,
    ist unbrauchbar."""
    out = run_gb(_FENSTER + '''
DIM cp AS GUI_WIDGET
cp = GUI_COLORPICKER(w, 10, 10, 200, 150)
DIM proben AS ARRAY OF INTEGER
proben = [&HFF0000, &H00FF00, &H0000FF, &H3FA9F5, &H808080, &HFFFFFF, &H000000, &H123456]
DIM i AS INTEGER
DIM schief AS INTEGER
FOR i = 0 TO LEN(proben) - 1
    GUI_SET_PICKED_COLOR(cp, proben[i])
    IF GUI_PICKED_COLOR(cp) <> proben[i] THEN schief = schief + 1
NEXT
PRINT schief
''')
    assert out.strip() == "0", f"{out.strip()} von 8 Farben kamen verändert zurück"


def test_grau_und_schwarz_behalten_den_farbton(run_gb):
    """Bei Grau ist der Farbton unbestimmt, bei Schwarz auch.

    Würde er dabei auf 0 (Rot) zurückfallen, spränge der Zeiger im Feld --
    und aus Schwarz käme beim Aufhellen Rot statt der Farbe, die man vorher
    gewählt hatte. Sichtbar wird das erst, wenn man danach wieder aufhellt:
    darum hier über den Umweg 'blau setzen, schwarz setzen, blau setzen'.
    """
    out = run_gb(_FENSTER + '''
DIM cp AS GUI_WIDGET
cp = GUI_COLORPICKER(w, 10, 10, 200, 150)
GUI_SET_PICKED_COLOR(cp, &H0000FF)
GUI_SET_PICKED_COLOR(cp, &H000000)
PRINT FORMAT$(GUI_PICKED_COLOR(cp), "%06X")
GUI_SET_PICKED_COLOR(cp, &H0000FF)
PRINT FORMAT$(GUI_PICKED_COLOR(cp), "%06X")
''')
    zeilen = out.strip().splitlines()
    assert zeilen[0] == "000000", "Schwarz muss Schwarz bleiben"
    assert zeilen[1] == "0000FF"


def test_farbwaehler_zugriffe_nur_auf_einem_farbwaehler(run_gb):
    out = run_gb(_FENSTER + '''
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 10, 10, 60, 24)
TRY
    PRINT GUI_PICKED_COLOR(b)
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


# --- Datumswähler ----------------------------------------------------------

def test_datum_kommt_unveraendert_zurueck(run_gb):
    out = run_gb(_FENSTER + '''
DIM dp AS GUI_WIDGET
dp = GUI_DATEPICKER(w, 10, 10, 300, 220)
DIM proben AS ARRAY OF STRING
proben = SPLIT$("2026-08-30|2000-01-01|2024-02-29|1999-12-31", "|")
DIM i AS INTEGER
FOR i = 0 TO LEN(proben) - 1
    GUI_SET_DATE(dp, proben[i])
    IF GUI_DATE(dp) <> proben[i] THEN PRINT "schief: "; proben[i]
NEXT
PRINT "fertig"
''')
    assert out.strip() == "fertig", out


def test_neuer_waehler_zeigt_heute(run_gb):
    """Ein leerer Kalender wäre eine unnötige Frage an den Benutzer."""
    out = run_gb(_FENSTER + '''
DIM dp AS GUI_WIDGET
dp = GUI_DATEPICKER(w, 10, 10, 300, 220)
PRINT IIF(GUI_DATE(dp) = DATE$(), "heute", GUI_DATE(dp) + " statt " + DATE$())
''')
    assert out.strip() == "heute"


def test_unsinniges_datum_wird_abgelehnt(run_gb):
    """Ein halb erkanntes Datum wäre schlimmer als eine klare Absage --
    besonders der 29. Februar in einem Jahr, das keines ist."""
    for schlecht in ("2026-13-01", "2023-02-29", "2026-8-30", "30.08.2026", ""):
        out = run_gb(_FENSTER + f'''
DIM dp AS GUI_WIDGET
dp = GUI_DATEPICKER(w, 10, 10, 300, 220)
TRY
    GUI_SET_DATE(dp, "{schlecht}")
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''')
        assert out.strip() == "abgelehnt", f"'{schlecht}' wurde angenommen"


def test_schalttag_wird_angenommen(run_gb):
    """Gegenprobe zum Test darüber: die Prüfung darf nicht einfach alles
    ablehnen, was ungewöhnlich aussieht."""
    out = run_gb(_FENSTER + '''
DIM dp AS GUI_WIDGET
dp = GUI_DATEPICKER(w, 10, 10, 300, 220)
GUI_SET_DATE(dp, "2024-02-29")
PRINT GUI_DATE(dp)
''')
    assert out.strip() == "2024-02-29"


def test_datum_zugriffe_nur_auf_einem_datumswaehler(run_gb):
    out = run_gb(_FENSTER + '''
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 10, 10, 60, 24)
TRY
    PRINT GUI_DATE(b)
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


# --- Beides überlebt Speichern und Laden -----------------------------------

def test_farbe_und_datum_ueberleben_die_form(run_gb):
    """Ohne das wäre ein im Form-Designer gesetzter Wert beim Laden weg --
    dieselbe Falle, in die die Tabellen-Schalter schon einmal gelaufen sind.
    """
    out = run_gb(_FENSTER + '''
DIM cp AS GUI_WIDGET
cp = GUI_COLORPICKER(w, 10, 10, 200, 150)
GUI_SET_PICKED_COLOR(cp, &H3FA9F5)
DIM dp AS GUI_WIDGET
dp = GUI_DATEPICKER(w, 10, 170, 300, 120)
GUI_SET_DATE(dp, "2024-02-29")

DIM roh AS STRING
roh = GUI_TO_JSON(w)
DIM w2 AS GUI_WINDOW
w2 = GUI_FROM_JSON(roh)
DIM cp2 AS GUI_WIDGET
cp2 = GUI_WINDOW_WIDGET(w2, 0)
DIM dp2 AS GUI_WIDGET
dp2 = GUI_WINDOW_WIDGET(w2, 1)
PRINT FORMAT$(GUI_PICKED_COLOR(cp2), "%06X"); " "; GUI_DATE(dp2)
''')
    assert out.strip() == "3FA9F5 2024-02-29"
