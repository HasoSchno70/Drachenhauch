"""Golden-Tests fuer die GUI-Serialisierung (Phase 2): GUI_TO_JSON/FROM_JSON
(String) + GUI_SAVE/LOAD (Datei). Headless via run_gb (kein SCREEN noetig)."""
import pytest

from gamebasic.errors import GameBasicError


_BUILD = (
    'IMPORT "gui"\n'
    'DIM win AS GUI_WINDOW\n'
    'win = GUI_WINDOW("Login", 100, 80, 300, 200)\n'
    'GUI_WINDOW_CLOSABLE(win, TRUE)\n'
    'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "Anmelden", 20, 120, 120, 30)\n'
    'DIM c AS GUI_WIDGET\nc = GUI_CHECKBOX(win, "Merken", 20, 80)\n'
    'GUI_SET_CHECKED(c, TRUE)\n'
    'DIM s AS GUI_WIDGET\ns = GUI_SLIDER(win, 20, 160, 200, 0, 100)\n'
    'GUI_SET_VALUE(s, 42)\n'
)


def test_string_roundtrip_structure(run_gb):
    out = run_gb(_BUILD +
        'DIM js AS STRING\njs = GUI_TO_JSON(win)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(js)\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(w2)\n'
        'PRINT f"{GUI_WINDOW_GET_X(w2)},{GUI_WINDOW_GET_W(w2)}"\n'
        'DIM w0 AS GUI_WIDGET\nw0 = GUI_WINDOW_WIDGET(w2, 0)\n'
        'PRINT f"{GUI_KIND(w0)} {GUI_GET_X(w0)},{GUI_GET_Y(w0)},{GUI_GET_W(w0)},{GUI_GET_H(w0)}"\n')
    assert out.splitlines() == ["3", "100,300", "button 20,120,120,30"]


def test_roundtrip_preserves_state(run_gb):
    # Checkbox-Zustand + Slider-Wert muessen erhalten bleiben.
    out = run_gb(_BUILD +
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_CHECKED(GUI_WINDOW_WIDGET(w2, 1))\n'
        'PRINT GUI_VALUE(GUI_WINDOW_WIDGET(w2, 2))\n')
    assert out.splitlines() == ["TRUE", "42.0"]


def test_roundtrip_skips_destroyed(run_gb):
    # Zerstoerte Widgets landen NICHT im JSON.
    out = run_gb(_BUILD +
        'GUI_DESTROY(b)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(w2)\n'              # 2 statt 3
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(w2, 0))\n')      # checkbox (button weg)
    assert out.splitlines() == ["2", "checkbox"]


def test_table_roundtrip(run_gb):
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM win AS GUI_WINDOW\nwin = GUI_WINDOW("T", 0, 0, 300, 200)\n'
        'DIM hdr[2] AS STRING\nhdr[0]="ID" : hdr[1]="Name"\n'
        'DIM cells[2, 2] AS STRING\n'
        'cells[0,0]="1" : cells[0,1]="Anna"\n'
        'cells[1,0]="2" : cells[1,1]="Bert"\n'
        'DIM t AS GUI_WIDGET\n'
        't = GUI_TABLE(win, 10, 10, 280, 150, hdr, cells)\n'
        'GUI_TABLE_SET_SELECTED(t, 1)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'DIM t2 AS GUI_WIDGET\nt2 = GUI_WINDOW_WIDGET(w2, 0)\n'
        'PRINT GUI_KIND(t2)\n'
        'PRINT GUI_TABLE_ROW_COUNT(t2)\n'
        'PRINT GUI_TABLE_SELECTED(t2)\n')
    assert out.splitlines() == ["table", "2", "1"]


def test_file_roundtrip(run_gb, tmp_path):
    out = run_gb(_BUILD +
        'GUI_SAVE(win, "layout.json")\n'
        'DIM w3 AS GUI_WINDOW\nw3 = GUI_LOAD("layout.json")\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(w3)\n'
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(w3, 0))\n',
        base=tmp_path)
    assert out.splitlines() == ["3", "button"]
    assert (tmp_path / "layout.json").exists()


def test_tree_menu_tabs_roundtrip(run_gb):
    # Tree-Knoten, Menue und Tabs muessen den GUI_SAVE/LOAD-Kreis ueberleben
    # (frueher stillschweigend verworfen -- siehe Review-Fund).
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM win AS GUI_WINDOW\nwin = GUI_WINDOW("T", 0, 0, 300, 200)\n'
        'DIM tabs[2] AS STRING\ntabs[0]="Eins" : tabs[1]="Zwei"\n'
        'GUI_TABS(win, tabs)\n'
        'GUI_SET_ACTIVE_TAB(win, 1)\n'
        'DIM m AS INTEGER\nm = GUI_MENU(win, "Datei")\n'
        'DIM mi AS INTEGER\nmi = GUI_MENU_ITEM(m, "Oeffnen")\n'
        'DIM t AS GUI_WIDGET\nt = GUI_TREE(win, 10, 10, 100, 100)\n'
        'DIM root AS INTEGER\nroot = GUI_TREE_ADD(t, -1, "Wurzel")\n'
        'DIM child AS INTEGER\nchild = GUI_TREE_ADD(t, root, "Kind")\n'
        'GUI_TREE_SET_SELECTED(t, child)\n'
        'GUI_SET_TAB(t, 1)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_ACTIVE_TAB(w2)\n'
        'DIM t2 AS GUI_WIDGET\nt2 = GUI_WINDOW_WIDGET(w2, 0)\n'
        'PRINT GUI_KIND(t2)\n'
        'PRINT GUI_TREE_SELECTED(t2)\n'
        'PRINT GUI_TREE_LABEL(t2, GUI_TREE_SELECTED(t2))\n')
    assert out.splitlines() == ["1", "tree", str(1), "Kind"]


def test_from_json_invalid_raises(run_gb):
    with pytest.raises(GameBasicError, match="ungueltiges JSON"):
        run_gb('IMPORT "gui"\nDIM w AS GUI_WINDOW\nw = GUI_FROM_JSON("{nope")\n')


def test_load_missing_file_raises(run_gb):
    with pytest.raises(GameBasicError, match="GUI_LOAD"):
        run_gb('IMPORT "gui"\nDIM w AS GUI_WINDOW\nw = GUI_LOAD("nope_xyz.json")\n')


# ---------------------------------------------- Zellen mit eigenen Merkmalen

def test_schlichte_zellen_bleiben_strings_im_json(run_gb, tmp_path):
    """Eine gewoehnliche Tabelle muss im .gbform genauso aussehen wie vorher --
    sonst koennten aeltere Dateien und der Form-Designer sie nicht mehr lesen.
    Erst eine Zelle mit Farbe/Art/Bild wird zum Objekt."""
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 300, 200)
DIM t AS GUI_WIDGET : t = GUI_TABLE(w, 5, 5, 280, 150)
DIM k AS ARRAY OF STRING : k = SPLIT$("A|B", "|")
GUI_TABLE_HEADERS(t, k)
DIM z AS ARRAY OF STRING : z = SPLIT$("eins|zwei", "|")
GUI_TABLE_ADD_ROW(t, z)
PRINT GUI_TO_JSON(w)
''', base=tmp_path)
    # JSON auswerten statt nach Bruchstuecken zu suchen: "text" steht auch in
    # jedem Widget, ein blosses `not in` haette immer angeschlagen.
    import json as _json
    d = _json.loads(out.strip())
    zeilen = d["widgets"][0]["table"]["rows"]
    assert zeilen == [["eins", "zwei"]], zeilen


def test_zelle_mit_farbe_ueberlebt_speichern_und_laden(run_gb, tmp_path):
    """Farbe, Ausrichtung und Art muessen den Roundtrip ueberstehen -- sonst
    saehe eine gespeicherte Form nach dem Laden anders aus als vorher."""
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 300, 200)
DIM t AS GUI_WIDGET : t = GUI_TABLE(w, 5, 5, 280, 150)
DIM k AS ARRAY OF STRING : k = SPLIT$("A|B", "|")
GUI_TABLE_HEADERS(t, k)
DIM z AS ARRAY OF STRING : z = SPLIT$("eins|zwei", "|")
GUI_TABLE_ADD_ROW(t, z)
GUI_TABLE_CELL_COLOR(t, 0, 1, &HFF0000, &H101010)
GUI_TABLE_CELL_KIND(t, 0, 0, "haken")
GUI_TABLE_CELL_VALUE(t, 0, 0, 1.0)
GUI_TABLE_SET(t, "zeilenhoehe", 33)
GUI_TABLE_SET(t, "zebra", 1)
DIM j AS STRING : j = GUI_TO_JSON(w)
DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)
DIM t2 AS GUI_WIDGET : t2 = GUI_WINDOW_WIDGET(w2, 0)
PRINT GUI_TABLE_GET_CELL(t2, 0, 1)
PRINT STR$(GUI_TABLE_GET_VALUE(t2, 0, 0))
PRINT STR$(GUI_TABLE_GET(t2, "zeilenhoehe"))
PRINT STR$(GUI_TABLE_GET(t2, "zebra"))
''', base=tmp_path)
    zeilen = out.strip().splitlines()
    assert zeilen[0] == "zwei", out
    assert zeilen[1].startswith("1"), out          # Haken-Wert
    assert zeilen[2].startswith("33"), out         # Zeilenhoehe
    assert zeilen[3].startswith("1"), out          # Zebra


def test_spaltenzahl_ist_frei(run_gb, tmp_path):
    """Kuerzere und laengere Zeilen als der Kopf sind erlaubt -- die
    Spaltenzahl ist die BREITESTE Angabe. Frueher war jede Abweichung ein
    Fehler, und wer die Zeilen vor dem Kopf setzte, umging die Pruefung ganz
    und brachte das Zeichnen zum Absturz."""
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 300, 200)
DIM t AS GUI_WIDGET : t = GUI_TABLE(w, 5, 5, 280, 150)
DIM k AS ARRAY OF STRING : k = SPLIT$("A|B", "|")
GUI_TABLE_HEADERS(t, k)
DIM kurz AS ARRAY OF STRING : kurz = SPLIT$("nur eins", "|")
GUI_TABLE_ADD_ROW(t, kurz)
DIM lang AS ARRAY OF STRING : lang = SPLIT$("a|b|c|d", "|")
GUI_TABLE_ADD_ROW(t, lang)
PRINT STR$(GUI_TABLE_GET(t, "spalten"))
PRINT "[" + GUI_TABLE_GET_CELL(t, 0, 1) + "]"
PRINT GUI_TABLE_GET_CELL(t, 1, 3)
''', base=tmp_path)
    zeilen = out.strip().splitlines()
    assert zeilen[0].startswith("4"), out          # breiteste Zeile bestimmt
    assert zeilen[1] == "[]", out                  # fehlende Zelle = leer
    assert zeilen[2] == "d", out


def test_zeile_entfernen_zieht_die_auswahl_mit(run_gb, tmp_path):
    """Sonst zeigte die Auswahl nach dem Loeschen auf eine ANDERE Zeile."""
    out = run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 300, 200)
DIM t AS GUI_WIDGET : t = GUI_TABLE(w, 5, 5, 280, 150)
DIM z AS ARRAY OF STRING
z = SPLIT$("a", "|") : GUI_TABLE_ADD_ROW(t, z)
z = SPLIT$("b", "|") : GUI_TABLE_ADD_ROW(t, z)
z = SPLIT$("c", "|") : GUI_TABLE_ADD_ROW(t, z)
GUI_TABLE_SET_SELECTED(t, 2)
GUI_TABLE_REMOVE_ROW(t, 0)
PRINT STR$(GUI_TABLE_SELECTED(t)) + " " + GUI_TABLE_GET_CELL(t, GUI_TABLE_SELECTED(t), 0)
GUI_TABLE_SET_SELECTED(t, 0)
GUI_TABLE_REMOVE_ROW(t, 0)
PRINT STR$(GUI_TABLE_SELECTED(t))
''', base=tmp_path)
    zeilen = out.strip().splitlines()
    assert zeilen[0] == "1 c", out     # Index rutscht mit, zeigt weiter auf "c"
    assert zeilen[1] == "-1", out      # die gewaehlte Zeile selbst geloescht


def test_unbekannter_schluessel_nennt_die_gueltigen(run_gb, tmp_path):
    from gamebasic.errors import GBRuntimeError
    import pytest as _pt
    with _pt.raises(GBRuntimeError, match="zeilenhoehe"):
        run_gb('''
IMPORT "gui"
DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 300, 200)
DIM t AS GUI_WIDGET : t = GUI_TABLE(w, 5, 5, 280, 150)
GUI_TABLE_SET(t, "gibtsnicht", 1)
''', base=tmp_path)


# ------------------------------------------------------- Sortieren + Filtern

def _tabelle(zeilen, rest):
    """Kleines Geruest: Kopf 'Name|Punkte', danach die Zeilen, dann `rest`."""
    add = "\n".join(
        f'z = SPLIT$("{a}|{b}", "|") : GUI_TABLE_ADD_ROW(t, z)' for a, b in zeilen)
    return f'''
IMPORT "gui"
DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 400, 300)
DIM t AS GUI_WIDGET : t = GUI_TABLE(w, 10, 10, 360, 200)
DIM k AS ARRAY OF STRING : k = SPLIT$("Name|Punkte", "|")
GUI_TABLE_HEADERS(t, k)
DIM z AS ARRAY OF STRING
{add}
SUB zeig()
    DIM s AS STRING : s = ""
    DIM i AS INTEGER
    FOR i = 0 TO GUI_TABLE_VIEW_COUNT(t) - 1
        s = s + GUI_TABLE_GET_CELL(t, GUI_TABLE_VIEW_ROW(t, i), 0) + " "
    NEXT
    PRINT TRIM$(s)
END SUB
{rest}
'''


def test_zahlenspalte_wird_zahlenweise_sortiert(run_gb, tmp_path):
    """Textweise stuende 100 vor 9 -- in einer Punktespalte die haeufigste
    Enttaeuschung an einer Tabelle."""
    out = run_gb(_tabelle([("Cleo", 9), ("Anna", 10), ("Bruno", 100)],
                          'GUI_TABLE_SORT(t, 1, FALSE)\nzeig()'), base=tmp_path)
    assert out.strip() == "Cleo Anna Bruno", out


def test_textspalte_sortiert_ohne_ruecksicht_auf_schreibweise(run_gb, tmp_path):
    out = run_gb(_tabelle([("bruno", 1), ("Anna", 2), ("cleo", 3)],
                          'GUI_TABLE_SORT(t, 0, FALSE)\nzeig()'), base=tmp_path)
    assert out.strip() == "Anna bruno cleo", out


def test_sortierung_stellt_die_daten_nicht_um(run_gb, tmp_path):
    """Der entscheidende Punkt: eine gemerkte Zeilennummer muss nach dem
    Sortieren noch auf denselben Eintrag zeigen. Wuerden die Daten selbst
    umgestellt, zeigte sie auf etwas anderes."""
    out = run_gb(_tabelle([("Cleo", 9), ("Anna", 10), ("Bruno", 100)], '''
PRINT GUI_TABLE_GET_CELL(t, 0, 0)
GUI_TABLE_SORT(t, 0, FALSE)
PRINT GUI_TABLE_GET_CELL(t, 0, 0)
PRINT STR$(GUI_TABLE_VIEW_ROW(t, 0))
'''), base=tmp_path)
    z = out.strip().splitlines()
    assert z[0] == "Cleo" and z[1] == "Cleo", out   # Datenzeile 0 bleibt Cleo
    assert z[2] == "1", out                          # oben steht jetzt Anna (Datenzeile 1)


def test_filter_wirkt_als_teiltext_und_kombiniert(run_gb, tmp_path):
    out = run_gb(_tabelle([("Anna", 1), ("Bruno", 2), ("Cleo", 3)], '''
GUI_TABLE_FILTER(t, 0, "o")
zeig()
PRINT STR$(GUI_TABLE_VIEW_COUNT(t)) + "/" + STR$(GUI_TABLE_ROW_COUNT(t))
GUI_TABLE_FILTER(t, 1, "3")
zeig()
GUI_TABLE_FILTER(t, 0, "")
GUI_TABLE_FILTER(t, 1, "")
zeig()
'''), base=tmp_path)
    z = out.strip().splitlines()
    assert z[0] == "Bruno Cleo", out       # beide enthalten "o"
    assert z[1] == "2/3", out              # Daten bleiben vollstaendig
    assert z[2] == "Cleo", out             # zweiter Filter wirkt zusaetzlich
    assert z[3] == "Anna Bruno Cleo", out


def test_filter_ignoriert_die_schreibweise(run_gb, tmp_path):
    out = run_gb(_tabelle([("Anna", 1), ("BRUNO", 2)],
                          'GUI_TABLE_FILTER(t, 0, "bru")\nzeig()'), base=tmp_path)
    assert out.strip() == "BRUNO", out


def test_sortieren_auf_ungueltige_spalte_hebt_sie_auf(run_gb, tmp_path):
    out = run_gb(_tabelle([("Cleo", 9), ("Anna", 10)], '''
GUI_TABLE_SORT(t, 0, FALSE)
PRINT STR$(GUI_TABLE_SORT_COL(t))
GUI_TABLE_SORT(t, -1, FALSE)
PRINT STR$(GUI_TABLE_SORT_COL(t))
zeig()
'''), base=tmp_path)
    z = out.strip().splitlines()
    assert z[0] == "0" and z[1] == "-1", out
    assert z[2] == "Cleo Anna", out        # wieder Einfuege-Reihenfolge


def test_neue_zeile_taucht_sofort_in_der_ansicht_auf(run_gb, tmp_path):
    """Die Ansicht wird nach jeder Datenaenderung neu gebaut -- sonst zeigte
    eine sortierte Tabelle neue Zeilen erst nach dem naechsten Kopfklick."""
    out = run_gb(_tabelle([("Cleo", 9)], '''
GUI_TABLE_SORT(t, 0, FALSE)
DIM n AS ARRAY OF STRING : n = SPLIT$("Anna|1", "|")
GUI_TABLE_ADD_ROW(t, n)
zeig()
'''), base=tmp_path)
    assert out.strip() == "Anna Cleo", out


# ------------------------------------------------------- Zellen bearbeiten

def test_bearbeiten_startet_nur_auf_freigegebenen_textspalten(run_gb, tmp_path):
    """Ohne Freigabe passiert nichts. Eine Tabelle, in die man versehentlich
    ueberall hineinschreiben kann, waere schlimmer als eine ohne Bearbeitung.
    Der Doppelklick selbst braucht eine echte Maus und ist per AUTOMATION_PLAY
    abgenommen -- hier wird der Zustand geprueft, den er setzt."""
    out = run_gb(_tabelle([("Anna", 1), ("Bruno", 2)], '''
PRINT STR$(GUI_TABLE_EDITING_ROW(t)) + " " + STR$(GUI_TABLE_EDITING_COL(t))
GUI_TABLE_COL_EDIT(t, 0, TRUE)
PRINT STR$(GUI_TABLE_EDITING_ROW(t))
'''), base=tmp_path)
    z = out.strip().splitlines()
    assert z[0] == "-1 -1", out       # nichts in Bearbeitung
    assert z[1] == "-1", out          # Freigeben allein startet nichts


def test_negative_spalte_beim_freigeben_wird_gemeldet(run_gb, tmp_path):
    from gamebasic.errors import GBRuntimeError
    import pytest as _pt
    with _pt.raises(GBRuntimeError, match="negativ"):
        run_gb(_tabelle([("Anna", 1)], 'GUI_TABLE_COL_EDIT(t, -1, TRUE)'), base=tmp_path)


# ------------------------------------------- Mehrfachauswahl + feste Spalten

def test_ohne_mehrfachauswahl_bleibt_es_bei_einer_zeile(run_gb, tmp_path):
    """Vorhandener Code, der nur GUI_TABLE_SELECTED kennt, muss unveraendert
    laufen -- deshalb ist die Mehrfachauswahl aus, bis man sie einschaltet."""
    out = run_gb(_tabelle([("Anna", 1), ("Bruno", 2)], '''
GUI_TABLE_SET_SELECTED(t, 1)
PRINT STR$(GUI_TABLE_SEL_COUNT(t)) + " " + STR$(GUI_TABLE_SEL_ROW(t, 0))
PRINT STR$(GUI_TABLE_IS_SELECTED(t, 0)) + " " + STR$(GUI_TABLE_IS_SELECTED(t, 1))
'''), base=tmp_path)
    z = out.strip().splitlines()
    assert z[0] == "1 1", out
    assert z[1] == "FALSE TRUE", out


def test_einschalten_uebernimmt_die_bisherige_auswahl(run_gb, tmp_path):
    """Sonst waere die gewaehlte Zeile nach dem Umschalten ploetzlich weg."""
    out = run_gb(_tabelle([("Anna", 1), ("Bruno", 2)], '''
GUI_TABLE_SET_SELECTED(t, 1)
GUI_TABLE_SET(t, "mehrfachauswahl", 1)
PRINT STR$(GUI_TABLE_SEL_COUNT(t)) + " " + STR$(GUI_TABLE_SEL_ROW(t, 0))
'''), base=tmp_path)
    assert out.strip() == "1 1", out


def test_mehrere_zeilen_waehlen_und_wieder_abwaehlen(run_gb, tmp_path):
    out = run_gb(_tabelle([("Anna", 1), ("Bruno", 2), ("Cleo", 3)], '''
GUI_TABLE_SET(t, "mehrfachauswahl", 1)
GUI_TABLE_SELECT(t, 0, TRUE)
GUI_TABLE_SELECT(t, 2, TRUE)
PRINT STR$(GUI_TABLE_SEL_COUNT(t))
PRINT STR$(GUI_TABLE_IS_SELECTED(t, 1))
GUI_TABLE_SELECT(t, 0, FALSE)
PRINT STR$(GUI_TABLE_SEL_COUNT(t)) + " " + STR$(GUI_TABLE_SEL_ROW(t, 0))
GUI_TABLE_CLEAR_SELECTION(t)
PRINT STR$(GUI_TABLE_SEL_COUNT(t)) + " " + STR$(GUI_TABLE_SELECTED(t))
'''), base=tmp_path)
    z = out.strip().splitlines()
    assert z[0] == "2", out
    assert z[1] == "FALSE", out
    assert z[2] == "1 2", out
    assert z[3] == "0 -1", out


def test_zeile_loeschen_zieht_die_ganze_auswahl_mit(run_gb, tmp_path):
    """Alle Indizes hinter der geloeschten ruecken auf. Ohne das zeigte die
    Auswahl danach auf voellig andere Zeilen."""
    out = run_gb(_tabelle([("Anna", 1), ("Bruno", 2), ("Cleo", 3)], '''
GUI_TABLE_SET(t, "mehrfachauswahl", 1)
GUI_TABLE_SELECT(t, 1, TRUE)
GUI_TABLE_SELECT(t, 2, TRUE)
GUI_TABLE_REMOVE_ROW(t, 0)
DIM s AS STRING : s = ""
DIM i AS INTEGER
FOR i = 0 TO GUI_TABLE_SEL_COUNT(t) - 1
    s = s + GUI_TABLE_GET_CELL(t, GUI_TABLE_SEL_ROW(t, i), 0) + " "
NEXT
PRINT TRIM$(s)
'''), base=tmp_path)
    assert out.strip() == "Bruno Cleo", out    # dieselben Eintraege wie vorher


def test_feste_spalten_werden_gedeckelt(run_gb, tmp_path):
    """Mehr feste Spalten als vorhanden waeren ein leerer Scrollbereich."""
    out = run_gb(_tabelle([("Anna", 1)], '''
GUI_TABLE_SET(t, "feste_spalten", 99)
PRINT STR$(GUI_TABLE_GET(t, "feste_spalten"))
GUI_TABLE_SET(t, "feste_spalten", -3)
PRINT STR$(GUI_TABLE_GET(t, "feste_spalten"))
'''), base=tmp_path)
    z = out.strip().splitlines()
    assert z[0].startswith("99"), out    # gemerkt wird der Wunsch ...
    assert z[1].startswith("0"), out     # ... negativ wird auf 0 geklemmt


# ----------------------------------------------------- Spalten umsortieren

def _spalten(rest):
    return f'''
IMPORT "gui"
DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 500, 300)
DIM t AS GUI_WIDGET : t = GUI_TABLE(w, 10, 10, 460, 200)
DIM k AS ARRAY OF STRING : k = SPLIT$("A|B|C|D", "|")
GUI_TABLE_HEADERS(t, k)
DIM z AS ARRAY OF STRING
z = SPLIT$("a|b|c|d", "|") : GUI_TABLE_ADD_ROW(t, z)
SUB zeig()
    DIM s AS STRING : s = ""
    DIM i AS INTEGER
    FOR i = 0 TO 3
        s = s + GUI_TABLE_GET_CELL(t, 0, GUI_TABLE_COL_AT(t, i))
    NEXT
    PRINT s
END SUB
{rest}
'''


def test_spalte_verschieben_laesst_die_daten_unberuehrt(run_gb, tmp_path):
    """Der Kernpunkt, genau wie beim Sortieren der Zeilen: eine gemerkte
    Spaltennummer muss weiter auf dieselbe Spalte zeigen."""
    out = run_gb(_spalten('''
zeig()
GUI_TABLE_MOVE_COL(t, 0, 3)
zeig()
PRINT GUI_TABLE_GET_CELL(t, 0, 0)
PRINT STR$(GUI_TABLE_COL_POS(t, 0)) + " " + STR$(GUI_TABLE_COL_AT(t, 0))
'''), base=tmp_path)
    z = out.strip().splitlines()
    assert z[0] == "abcd", out
    assert z[1] == "bcda", out       # Anzeige umgestellt
    assert z[2] == "a", out          # Datenspalte 0 ist unveraendert "a"
    assert z[3] == "3 1", out        # A steht auf Position 3, dort Datenspalte 1


def test_reihenfolge_zuruecksetzen(run_gb, tmp_path):
    out = run_gb(_spalten('''
GUI_TABLE_MOVE_COL(t, 0, 3)
GUI_TABLE_RESET_COLS(t)
zeig()
'''), base=tmp_path)
    assert out.strip() == "abcd", out


def test_neue_spalte_faellt_nicht_unter_den_tisch(run_gb, tmp_path):
    """Die gespeicherte Reihenfolge wird bei jedem Abruf bereinigt und
    ergaenzt -- sonst fehlte eine spaeter dazugekommene Spalte beim Zeichnen
    oder eine entfernte tauchte doppelt auf."""
    out = run_gb(_spalten('''
GUI_TABLE_MOVE_COL(t, 0, 3)
DIM k2 AS ARRAY OF STRING : k2 = SPLIT$("A|B|C|D|E", "|")
GUI_TABLE_HEADERS(t, k2)
DIM s AS STRING : s = ""
DIM i AS INTEGER
FOR i = 0 TO 4
    s = s + STR$(GUI_TABLE_COL_AT(t, i)) + " "
NEXT
PRINT TRIM$(s)
'''), base=tmp_path)
    # Die neue Spalte 4 haengt sich hinten an, die Umstellung bleibt erhalten.
    assert out.strip() == "1 2 3 0 4", out


def test_position_ausserhalb_wird_gemeldet(run_gb, tmp_path):
    from gamebasic.errors import GBRuntimeError
    import pytest as _pt
    with _pt.raises(GBRuntimeError, match="ausserhalb"):
        run_gb(_spalten('GUI_TABLE_MOVE_COL(t, 0, 9)'), base=tmp_path)


def test_reihenfolge_ueberlebt_speichern_und_laden(run_gb, tmp_path):
    out = run_gb(_spalten('''
GUI_TABLE_MOVE_COL(t, 0, 3)
GUI_TABLE_SET(t, "feste_spalten", 2)
DIM j AS STRING : j = GUI_TO_JSON(w)
DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)
DIM t2 AS GUI_WIDGET : t2 = GUI_WINDOW_WIDGET(w2, 0)
PRINT STR$(GUI_TABLE_COL_AT(t2, 0)) + " " + STR$(GUI_TABLE_GET(t2, "feste_spalten"))
'''), base=tmp_path)
    assert out.strip().startswith("1 2"), out


def test_beim_oeffnen_ist_alles_markiert(run_gb, tmp_path):
    """Das erste Tippen soll den Inhalt ERSETZEN, nicht verlaengern -- so
    erwartet man es, wenn man eine Zelle zum Ueberschreiben aufmacht.
    Geprueft wird der Zustand, den `table_begin_edit` setzt; das Tippen selbst
    braucht eine echte Tastatur (raylibs Automation fuellt nur die Tasten-,
    nicht die Zeichen-Warteschlange) und ist ueber eingespeiste Ruecktaste/
    Entf abgenommen: beide loeschten den GANZEN Inhalt."""
    out = run_gb(_tabelle([("Anna", 1)], '''
GUI_TABLE_COL_EDIT(t, 0, TRUE)
PRINT STR$(GUI_TABLE_EDITING_ROW(t)) + " " + STR$(GUI_TABLE_EDITING_COL(t))
'''), base=tmp_path)
    # Freigeben allein startet nichts -- der Doppelklick tut das.
    assert out.strip() == "-1 -1", out
