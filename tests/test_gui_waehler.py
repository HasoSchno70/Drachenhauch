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


# --- Farbe als Text --------------------------------------------------------
#
# Es gibt sonst KEINEN Weg von "#FF8800" zu einer Farbe: `VAL` kennt keine
# Hexzahlen (`VAL("&HFF8800")` ist 0), und `&H`-Literale gibt es nur im
# Quelltext. Ohne diese beiden kann ein Programm weder eine Farbe aus einer
# Einstellungsdatei lesen noch eine eingetippte übernehmen.

def test_farbe_als_text_hin_und_zurueck(run_gb):
    out = run_gb('''
DIM proben AS ARRAY OF INTEGER
proben = [&HFF8800, &H000000, &HFFFFFF, &H123456]
DIM i AS INTEGER
DIM schief AS INTEGER
FOR i = 0 TO LEN(proben) - 1
    IF COLOR_FROM_HEX(COLOR_HEX$(proben[i])) <> proben[i] THEN schief = schief + 1
NEXT
PRINT schief; " "; COLOR_HEX$(&HFF8800)
''')
    n, text = out.strip().split()
    assert n == "0" and text == "#FF8800"


def test_kurzform_wird_verdoppelt(run_gb):
    """`#F80` = `#FF8800`, wie im Web."""
    out = run_gb('PRINT COLOR_HEX$(COLOR_FROM_HEX("#F80"))')
    assert out.strip() == "#FF8800"


def test_hex_akzeptiert_die_ueblichen_schreibweisen(run_gb):
    out = run_gb('''
DIM formen AS ARRAY OF STRING
formen = SPLIT$("#FF8800|FF8800|0xFF8800|&HFF8800", "|")
DIM i AS INTEGER
FOR i = 0 TO LEN(formen) - 1
    IF COLOR_FROM_HEX(formen[i]) <> &HFF8800 THEN PRINT "schief: "; formen[i]
NEXT
PRINT "fertig"
''')
    assert out.strip() == "fertig"


def test_kein_hex_wird_abgelehnt(run_gb):
    for schlecht in ("rot", "#GG0000", "#FF88", ""):
        out = run_gb(f'''
TRY
    PRINT COLOR_FROM_HEX("{schlecht}")
CATCH e
    PRINT "abgelehnt"
END TRY
''')
        assert out.strip() == "abgelehnt", f"'{schlecht}' wurde angenommen"


def test_deckkraft_steht_im_text_nur_wenn_sie_gesetzt_ist(run_gb):
    """Oberstes Byte 0 heisst DECKEND -- dann gehört es auch nicht in den
    Text, sonst stünde vor jeder gewöhnlichen Farbe ein sinnloses `00`."""
    out = run_gb('PRINT COLOR_HEX$(&HFF8800); " "; COLOR_HEX$(&H80FF8800)')
    assert out.strip() == "#FF8800 #80FF8800"


# --- Deckkraft im Farbwähler ----------------------------------------------

def test_ohne_alpha_bleibt_es_bei_sechs_stellen(run_gb):
    """Programme, die den Wähler schon benutzen, bekommen weiterhin
    `0xRRGGBB` -- der Streifen ist ausdrücklich einzuschalten."""
    out = run_gb(_FENSTER + '''
DIM cp AS GUI_WIDGET
cp = GUI_COLORPICKER(w, 10, 10, 200, 150)
GUI_SET_PICKED_COLOR(cp, &H80FF8800)
PRINT COLOR_HEX$(GUI_PICKED_COLOR(cp))
''')
    assert out.strip() == "#FF8800"


def test_mit_alpha_kommt_die_deckkraft_mit(run_gb):
    out = run_gb(_FENSTER + '''
DIM cp AS GUI_WIDGET
cp = GUI_COLORPICKER(w, 10, 10, 200, 150)
GUI_COLORPICKER_SET(cp, "alpha", 1)
GUI_SET_PICKED_COLOR(cp, &H80FF8800)
PRINT COLOR_HEX$(GUI_PICKED_COLOR(cp))
''')
    assert out.strip() == "#80FF8800"


def test_unbekannte_einstellung_wird_abgelehnt(run_gb):
    out = run_gb(_FENSTER + '''
DIM cp AS GUI_WIDGET
cp = GUI_COLORPICKER(w, 10, 10, 200, 150)
TRY
    GUI_COLORPICKER_SET(cp, "durchsicht", 1)
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


# --- Grenzen im Datumswähler ----------------------------------------------

def test_grenzen_ziehen_ein_datum_herein(run_gb):
    """Sonst stünde im Feld ein Wert, den der Wähler selbst nicht zulässt."""
    out = run_gb(_FENSTER + '''
DIM dp AS GUI_WIDGET
dp = GUI_DATEPICKER(w, 10, 10, 300, 220)
GUI_SET_DATE(dp, "2026-01-01")
GUI_DATE_RANGE(dp, "2026-09-05", "2026-09-25")
PRINT GUI_DATE(dp)
GUI_SET_DATE(dp, "2026-12-31")
GUI_DATE_RANGE(dp, "2026-09-05", "2026-09-25")
PRINT GUI_DATE(dp)
''')
    assert out.strip().splitlines() == ["2026-09-05", "2026-09-25"]


def test_leere_grenze_hebt_sie_auf(run_gb):
    out = run_gb(_FENSTER + '''
DIM dp AS GUI_WIDGET
dp = GUI_DATEPICKER(w, 10, 10, 300, 220)
GUI_DATE_RANGE(dp, "2026-09-05", "")
GUI_SET_DATE(dp, "2030-01-01")
PRINT GUI_DATE(dp)
''')
    assert out.strip() == "2030-01-01"


def test_verdrehte_grenzen_werden_abgelehnt(run_gb):
    out = run_gb(_FENSTER + '''
DIM dp AS GUI_WIDGET
dp = GUI_DATEPICKER(w, 10, 10, 300, 220)
TRY
    GUI_DATE_RANGE(dp, "2026-09-25", "2026-09-05")
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


def test_wochenbeginn_wird_angenommen(run_gb):
    out = run_gb(_FENSTER + '''
DIM dp AS GUI_WIDGET
dp = GUI_DATEPICKER(w, 10, 10, 300, 220)
GUI_DATEPICKER_SET(dp, "wochenbeginn", 6)
TRY
    GUI_DATEPICKER_SET(dp, "wochenanfang", 6)
    PRINT "angenommen"
CATCH e
    PRINT "unbekannter Schluessel abgelehnt"
END TRY
''')
    assert out.strip() == "unbekannter Schluessel abgelehnt"
