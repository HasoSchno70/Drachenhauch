"""Das `TEXTAREA` als Code-Feld: einfärben und einstellen.

`SYNTAX_SPANS` zerlegt Drachenhauch-Quelltext in Abschnitte
`(start, länge, art)`; `GUI_TEXTAREA_SPANS` färbt ein Textfeld danach ein.
Getrennt, weil welche Farbe ein Kommentar hat eine Frage des Themas ist und
nicht der Sprache.

**Ein Hervorheber ist kein Lexer** -- er bekommt halb getippten Text zu sehen
und muss ihn trotzdem darstellen. Die Zerlegung selbst prüfen zwölf
Rust-`#[test]`s in `syntax.rs` (dort ohne Fenster und ohne Prozessstart);
hier steht, was über die GB-Grenze geht -- dazu die Einstellungen
(`GUI_TEXTAREA_SET`). Was nur mit echten Tastendrücken zu prüfen ist
(Tabulator), steht in `test_gui_tastatur.py`.
"""


def test_liefert_drei_gleich_lange_listen(run_gb):
    out = run_gb('''
DIM s AS ARRAY OF INTEGER
DIM l AS ARRAY OF INTEGER
DIM a AS ARRAY OF STRING
(s, l, a) = SYNTAX_SPANS("DIM x AS INTEGER")
PRINT LEN(s); " "; LEN(l); " "; LEN(a)
''')
    n = out.strip().split()
    assert n[0] == n[1] == n[2] == "4", out


def test_arten_stimmen(run_gb):
    """Die Zerlegung kommt aus `syntax.rs`; hier zählt, dass sie unverfälscht
    in GB ankommt -- Reihenfolge, Namen, Positionen."""
    out = run_gb('''
DIM s AS ARRAY OF INTEGER
DIM l AS ARRAY OF INTEGER
DIM a AS ARRAY OF STRING
DIM q AS STRING
q = "DIM n AS INTEGER"
(s, l, a) = SYNTAX_SPANS(q)
DIM i AS INTEGER
FOR i = 0 TO LEN(s) - 1
    PRINT a[i]; "=" ; MID$(q, s[i], l[i])
NEXT
''')
    zeilen = [z.strip() for z in out.strip().splitlines()]
    assert zeilen == ["schluessel=DIM", "name=n", "schluessel=AS",
                      "schluessel=INTEGER"], zeilen


def test_schluesselwort_und_name_werden_unterschieden(run_gb):
    out = run_gb('''
DIM s AS ARRAY OF INTEGER
DIM l AS ARRAY OF INTEGER
DIM a AS ARRAY OF STRING
(s, l, a) = SYNTAX_SPANS("WHILE zaehler")
PRINT a[0]; " "; a[1]
''')
    assert out.strip() == "schluessel name"


def test_kommentar_und_zahl(run_gb):
    out = run_gb('''
DIM s AS ARRAY OF INTEGER
DIM l AS ARRAY OF INTEGER
DIM a AS ARRAY OF STRING
(s, l, a) = SYNTAX_SPANS("x = 42 " + CHR$(39) + " Notiz")
DIM i AS INTEGER
FOR i = 0 TO LEN(a) - 1
    PRINT a[i]
NEXT
''')
    assert out.strip().splitlines() == ["name", "operator", "zahl", "kommentar"]


def test_leerer_text_gibt_leere_listen(run_gb):
    out = run_gb('''
DIM s AS ARRAY OF INTEGER
DIM l AS ARRAY OF INTEGER
DIM a AS ARRAY OF STRING
(s, l, a) = SYNTAX_SPANS("")
PRINT LEN(s)
''')
    assert out.strip() == "0"


def test_spans_brauchen_gleich_lange_listen(run_gb):
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 0, 0, 200, 200)
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 4, 4, 180, 100)
DIM s AS ARRAY OF INTEGER
s = [0, 4]
DIM l AS ARRAY OF INTEGER
l = [3]
DIM f AS ARRAY OF INTEGER
f = [255]
TRY
    GUI_TEXTAREA_SPANS(ta, s, l, f)
    PRINT "kein Fehler"
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


def test_spans_nur_auf_einem_textfeld(run_gb):
    """Ein Knopf hat keine Zeilen -- die Angabe waere sinnlos, und still
    nichts zu tun liesse den Programmierer die Ursache woanders suchen."""
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 0, 0, 200, 200)
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 4, 4, 60, 20)
DIM s AS ARRAY OF INTEGER
s = [0]
DIM l AS ARRAY OF INTEGER
l = [3]
DIM f AS ARRAY OF INTEGER
f = [255]
TRY
    GUI_TEXTAREA_SPANS(b, s, l, f)
    PRINT "kein Fehler"
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


def test_abschnitt_hinter_dem_textende_ist_harmlos(run_gb):
    """Beim Tippen hinkt die Einfärbung dem Text immer ein Stück hinterher.
    Ein Abschnitt, der über das Ende hinausragt, darf darum nicht abstürzen --
    er wird beim Zeichnen abgeschnitten."""
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 0, 0, 200, 200)
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 4, 4, 180, 100)
GUI_SET_TEXT(ta, "kurz")
DIM s AS ARRAY OF INTEGER
s = [0, 900]
DIM l AS ARRAY OF INTEGER
l = [9999, 5]
DIM f AS ARRAY OF INTEGER
f = [255, 255]
GUI_TEXTAREA_SPANS(ta, s, l, f)
PRINT "ok"
''')
    assert out.strip() == "ok"


def test_der_ganze_weg(run_gb):
    """Vom Quelltext zur Einfärbung, so wie ein Editor es tut."""
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 0, 0, 300, 200)
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 4, 4, 280, 150)
DIM q AS STRING
q = "FOR i = 0 TO 9" + CHR$(10) + "NEXT"
GUI_SET_TEXT(ta, q)
DIM s AS ARRAY OF INTEGER
DIM l AS ARRAY OF INTEGER
DIM a AS ARRAY OF STRING
(s, l, a) = SYNTAX_SPANS(q)
DIM f[LEN(s)] AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO LEN(s) - 1
    f[i] = IIF(a[i] = "schluessel", &H2BC4E8, &HFFFFFF)
NEXT
GUI_TEXTAREA_SPANS(ta, s, l, f)
PRINT LEN(s)
''')
    # FOR i = 0 TO 9 -> 6 Abschnitte, NEXT -> 1
    assert out.strip() == "7"


def test_einstellung_muss_bekannt_sein(run_gb):
    """Ein unbekannter Schlüssel zählt die gültigen auf, statt still nichts
    zu tun -- sonst sucht man den Fehler im eigenen Programm."""
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 0, 0, 200, 200)
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 4, 4, 180, 100)
TRY
    GUI_TEXTAREA_SET(ta, "zeilennumern", 1)
    PRINT "kein Fehler"
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


def test_einstellung_nur_auf_einem_textbereich(run_gb):
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 0, 0, 200, 200)
DIM e AS GUI_WIDGET
e = GUI_TEXTINPUT(w, 4, 4, 120, 24)
TRY
    GUI_TEXTAREA_SET(e, "zeilennummern", 1)
    PRINT "kein Fehler"
CATCH ex
    PRINT "abgelehnt"
END TRY
''')
    assert out.strip() == "abgelehnt"


def test_die_vier_einstellungen_werden_angenommen(run_gb):
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW
w = GUI_WINDOW("T", 0, 0, 300, 200)
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 4, 4, 280, 150)
GUI_TEXTAREA_SET(ta, "zeilennummern", 1)
GUI_TEXTAREA_SET(ta, "aktive_zeile", 1)
GUI_TEXTAREA_SET(ta, "tab_fuegt_ein", 1)
GUI_TEXTAREA_SET(ta, "tabbreite", 8)
PRINT "ok"
''')
    assert out.strip() == "ok"
