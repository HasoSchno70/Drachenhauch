"""Modul `chart` -- Diagramme (Kuchen/Balken/Linie/Tacho).

Golden-Tests gegen gbrt. Gezeichnet wird hier nichts (das braucht ein
Fenster) -- geprueft werden Aufbau, Daten, Kennzahlen und vor allem die
Fehlermeldungen der vier Stil-Setter, denn deren Schluessel sind Strings
und darum die einzige Stelle, an der ein Tippfehler erst zur Laufzeit
auffaellt.
"""
import pytest


def _lines(out):
    return [line.strip() for line in out.split("\n") if line.strip()]


HEAD = 'IMPORT "chart"\nDIM c AS CHART\n'


def test_chart_new_liefert_handle(run_gb):
    src = HEAD + 'c = CHART_NEW("kuchen", 0, 0, 100, 100)\nPRINT CHART_COUNT(c)\n'
    assert _lines(run_gb(src)) == ["0"]


def test_chart_new_kennt_deutsche_und_englische_arten(run_gb):
    src = HEAD
    for art in ("kuchen", "pie", "donut", "balken", "bar", "linie", "line",
                "flaeche", "area", "tacho", "gauge"):
        src += f'c = CHART_NEW("{art}", 0, 0, 10, 10)\n'
    src += 'PRINT "ok"\n'
    assert _lines(run_gb(src)) == ["ok"]


def test_chart_new_meldet_unbekannte_art(run_gb):
    src = HEAD + (
        'TRY\n'
        '    c = CHART_NEW("torte", 0, 0, 10, 10)\n'
        'CATCH e\n'
        '    PRINT e\n'
        'END TRY\n'
    )
    out = run_gb(src)
    assert "torte" in out and "kuchen" in out


def test_chart_add_und_get(run_gb):
    src = HEAD + (
        'c = CHART_NEW("kuchen", 0, 0, 100, 100)\n'
        'CHART_ADD(c, "Holz", 45.0, 100)\n'
        'CHART_ADD(c, "Stein", 30.0, 200)\n'
        'PRINT CHART_COUNT(c)\n'
        'PRINT CHART_GET(c, 0, 1)\n'
    )
    assert _lines(run_gb(src)) == ["2", "30.0"]


def test_chart_data_aus_array(run_gb):
    src = HEAD + (
        'DIM s AS INTEGER\n'
        'DIM v[3] AS FLOAT\n'
        'c = CHART_NEW("balken", 0, 0, 100, 100)\n'
        's = CHART_SERIES(c, "a", 0)\n'
        'v[0] = 1.5\nv[1] = 2.5\nv[2] = 3.5\n'
        'CHART_DATA(c, s, v)\n'
        'PRINT CHART_COUNT(c)\n'
        'PRINT CHART_GET(c, s, 2)\n'
    )
    assert _lines(run_gb(src)) == ["3", "3.5"]


def test_chart_push_mit_fenster_laesst_vorne_herausfallen(run_gb):
    """Gleitendes Fenster fuer Live-Kurven -- der aelteste Wert faellt raus."""
    src = HEAD + (
        'DIM i AS INTEGER\n'
        'c = CHART_NEW("linie", 0, 0, 100, 100)\n'
        'CHART_SERIES(c, "s", 0)\n'
        'CHART_SET_NUM(c, "fenster", 3)\n'
        'FOR i = 0 TO 5\n'
        '    CHART_PUSH(c, 0, i)\n'
        'NEXT\n'
        'PRINT CHART_COUNT(c)\n'
        'PRINT CHART_GET(c, 0, 0)\n'
    )
    assert _lines(run_gb(src)) == ["3", "3.0"]


def test_chart_stat_kennzahlen(run_gb):
    src = HEAD + (
        'DIM v[4] AS FLOAT\n'
        'c = CHART_NEW("balken", 0, 0, 100, 100)\n'
        'CHART_SERIES(c, "s", 0)\n'
        'v[0] = 2.0\nv[1] = 4.0\nv[2] = 6.0\nv[3] = 8.0\n'
        'CHART_DATA(c, 0, v)\n'
        'PRINT CHART_STAT(c, 0, "summe")\n'
        'PRINT CHART_STAT(c, 0, "mittel")\n'
        'PRINT CHART_STAT(c, 0, "min")\n'
        'PRINT CHART_STAT(c, 0, "max")\n'
        'PRINT CHART_STAT(c, 0, "anzahl")\n'
    )
    assert _lines(run_gb(src)) == ["20.0", "5.0", "2.0", "8.0", "4.0"]


def test_chart_value_geht_ohne_series(run_gb):
    """Beim Tacho legt CHART_NEW die Reihe schon an."""
    src = HEAD + (
        'c = CHART_NEW("tacho", 0, 0, 100, 100)\n'
        'CHART_VALUE(c, 42.0)\n'
        'PRINT CHART_GET(c, 0, 0)\n'
    )
    assert _lines(run_gb(src)) == ["42.0"]


def test_chart_get_liefert_den_echten_wert_trotz_animation(run_gb):
    """CHART_GET meldet immer den gesetzten Wert -- die Animation betrifft
    nur die Anzeige. Sonst koennte ein Programm seinen eigenen Wert nicht
    zuverlaessig zuruecklesen."""
    src = HEAD + (
        'c = CHART_NEW("tacho", 0, 0, 100, 100)\n'
        'CHART_SET_NUM(c, "animation", 0.5)\n'
        'CHART_VALUE(c, 100.0)\n'
        'CHART_UPDATE(c, 0.016)\n'
        'PRINT CHART_GET(c, 0, 0)\n'
    )
    assert _lines(run_gb(src)) == ["100.0"]


@pytest.mark.parametrize("setter,key,wert", [
    ("CHART_SET", "titel", '"Hallo"'),
    ("CHART_SET", "legende", '"oben"'),
    ("CHART_SET", "werte", '"innen"'),
    ("CHART_SET", "ausrichtung", '"waagerecht"'),
    ("CHART_SET", "zeigerform", '"pfeil"'),
    ("CHART_SET_NUM", "innenradius", "0.5"),
    ("CHART_SET_NUM", "deckkraft", "0.5"),
    ("CHART_SET_NUM", "schatten_weich", "6"),
    ("CHART_SET_NUM", "flaeche_deckkraft", "0.2"),
    ("CHART_SET_COLOR", "hintergrund", "255"),
    ("CHART_SET_COLOR", "schatten", "128"),
    ("CHART_SET_COLOR", "verlauf_ende", "255"),
    ("CHART_SET_FLAG", "stapel", "TRUE"),
    ("CHART_SET_FLAG", "verlauf_daten", "TRUE"),
    ("CHART_SET_FLAG", "schatten_daten", "TRUE"),
])
def test_stil_setter_akzeptieren_ihre_schluessel(run_gb, setter, key, wert):
    src = HEAD + (
        'c = CHART_NEW("balken", 0, 0, 100, 100)\n'
        f'{setter}(c, "{key}", {wert})\n'
        'PRINT "ok"\n'
    )
    assert _lines(run_gb(src)) == ["ok"]


@pytest.mark.parametrize("setter,wert", [
    ("CHART_SET", '"x"'),
    ("CHART_SET_NUM", "1.0"),
    ("CHART_SET_COLOR", "255"),
    ("CHART_SET_FLAG", "TRUE"),
])
def test_unbekannter_schluessel_nennt_die_gueltigen(run_gb, setter, wert):
    """Der Preis der String-Schluessel: der Tippfehler faellt erst zur
    Laufzeit auf. Dann muss die Meldung aber sagen, was gueltig waere."""
    src = HEAD + (
        'c = CHART_NEW("balken", 0, 0, 100, 100)\n'
        'TRY\n'
        f'    {setter}(c, "gibtsnicht", {wert})\n'
        'CATCH e\n'
        '    PRINT e\n'
        'END TRY\n'
    )
    out = run_gb(src)
    assert "gibtsnicht" in out
    assert "gueltig" in out


def test_ungueltiger_stil_wert_wird_abgelehnt(run_gb):
    src = HEAD + (
        'c = CHART_NEW("balken", 0, 0, 100, 100)\n'
        'TRY\n'
        '    CHART_SET(c, "legende", "schraeg")\n'
        'CATCH e\n'
        '    PRINT e\n'
        'END TRY\n'
    )
    out = run_gb(src)
    assert "schraeg" in out and "oben" in out


def test_theme_nennt_die_verfuegbaren(run_gb):
    src = HEAD + (
        'c = CHART_NEW("balken", 0, 0, 100, 100)\n'
        'CHART_THEME(c, "neon")\n'
        'TRY\n'
        '    CHART_THEME(c, "quietschbunt")\n'
        'CATCH e\n'
        '    PRINT e\n'
        'END TRY\n'
    )
    out = run_gb(src)
    assert "quietschbunt" in out and "dunkel" in out and "pastell" in out


def test_falscher_handle_typ_meldet_klartext(run_gb):
    src = (
        'IMPORT "chart"\n'
        'TRY\n'
        '    PRINT CHART_COUNT(42)\n'
        'CATCH e\n'
        '    PRINT e\n'
        'END TRY\n'
    )
    assert "CHART" in run_gb(src)


def test_reihe_ausserhalb_meldet_klartext(run_gb):
    src = HEAD + (
        'c = CHART_NEW("balken", 0, 0, 100, 100)\n'
        'CHART_SERIES(c, "a", 0)\n'
        'TRY\n'
        '    CHART_PUSH(c, 7, 1.0)\n'
        'CATCH e\n'
        '    PRINT e\n'
        'END TRY\n'
    )
    assert "7" in run_gb(src)


def test_clear_leert_die_daten(run_gb):
    src = HEAD + (
        'c = CHART_NEW("kuchen", 0, 0, 100, 100)\n'
        'CHART_ADD(c, "a", 1.0, 0)\n'
        'CHART_ADD(c, "b", 2.0, 0)\n'
        'CHART_CLEAR(c)\n'
        'PRINT CHART_COUNT(c)\n'
    )
    assert _lines(run_gb(src)) == ["0"]


def test_zonen_sammeln_und_leeren(run_gb):
    src = HEAD + (
        'c = CHART_NEW("tacho", 0, 0, 100, 100)\n'
        'CHART_ZONE(c, 50, 80, 16776960)\n'
        'CHART_ZONE(c, 80, 100, 16711680)\n'
        'CHART_ZONE_CLEAR(c)\n'
        'PRINT "ok"\n'
    )
    assert _lines(run_gb(src)) == ["ok"]


def test_null_groesse_wird_abgelehnt(run_gb):
    src = HEAD + (
        'TRY\n'
        '    c = CHART_NEW("balken", 0, 0, 0, 100)\n'
        'CATCH e\n'
        '    PRINT e\n'
        'END TRY\n'
    )
    assert "Breite" in run_gb(src)


def test_tacho_bringt_seine_reihe_schon_mit(run_gb):
    """IMPORT "chart" ohne .gb-Endung nimmt immer das eingebaute Modul."""
    src = HEAD + (
        'c = CHART_NEW("tacho", 0, 0, 10, 10)\n'
        'PRINT CHART_SERIES_COUNT(c)\n'
    )
    assert _lines(run_gb(src)) == ["1"]


# --- Interaktion (Hover/Klick) -------------------------------------------
#
# Der eigentliche Treffertest braucht Fenster und Maus und laeuft darum nicht
# hier, sondern als Rust-#[test] auf der Winkel-/Bereichsmathematik. Golden
# geprueft wird, dass die Abfragen ohne Zeichnen einen sauberen Leerwert
# liefern statt zu stolpern -- genau das fragt ein Programm im ersten Bild ab.

def test_hover_abfragen_sind_ohne_zeichnen_leer(run_gb):
    src = HEAD + (
        'c = CHART_NEW("kuchen", 0, 0, 100, 100)\n'
        'CHART_ADD(c, "a", 1.0, 0)\n'
        'PRINT CHART_HOVER(c)\n'
        'PRINT CHART_HOVER_SERIES(c)\n'
        'PRINT CHART_CLICKED(c)\n'
        'PRINT "[" + CHART_HOVER_LABEL$(c) + "]"\n'
        'PRINT CHART_HOVER_VALUE(c)\n'
    )
    assert _lines(run_gb(src)) == ["-1", "-1", "-1", "[]", "0.0"]


@pytest.mark.parametrize("key", ["hover", "tooltip"])
def test_interaktions_schalter(run_gb, key):
    src = HEAD + (
        'c = CHART_NEW("kuchen", 0, 0, 100, 100)\n'
        f'CHART_SET_FLAG(c, "{key}", FALSE)\n'
        'PRINT "ok"\n'
    )
    assert _lines(run_gb(src)) == ["ok"]


@pytest.mark.parametrize("key", ["hover_tempo", "hover_weite", "hover_glanz"])
def test_interaktions_zahlen(run_gb, key):
    src = HEAD + (
        'c = CHART_NEW("kuchen", 0, 0, 100, 100)\n'
        f'CHART_SET_NUM(c, "{key}", 0.5)\n'
        'PRINT "ok"\n'
    )
    assert _lines(run_gb(src)) == ["ok"]


# --- Tacho-Gestaltung -----------------------------------------------------

@pytest.mark.parametrize("blatt", ["ring", "segmente", "striche", "baender"])
def test_zifferblatt_bauarten(run_gb, blatt):
    src = HEAD + (
        'c = CHART_NEW("tacho", 0, 0, 200, 200)\n'
        f'CHART_SET(c, "zifferblatt", "{blatt}")\n'
        'PRINT "ok"\n'
    )
    assert _lines(run_gb(src)) == ["ok"]


@pytest.mark.parametrize("form", ["aus", "innen", "pille", "blase", "am_zeiger"])
def test_wertanzeige_formen(run_gb, form):
    src = HEAD + (
        'c = CHART_NEW("tacho", 0, 0, 200, 200)\n'
        f'CHART_SET(c, "wertanzeige", "{form}")\n'
        'PRINT "ok"\n'
    )
    assert _lines(run_gb(src)) == ["ok"]


def test_zifferblatt_meldet_unbekannte_bauart(run_gb):
    src = HEAD + (
        'c = CHART_NEW("tacho", 0, 0, 200, 200)\n'
        'TRY\n'
        '    CHART_SET(c, "zifferblatt", "kringel")\n'
        'CATCH e\n'
        '    PRINT e\n'
        'END TRY\n'
    )
    out = run_gb(src)
    assert "kringel" in out and "segmente" in out


def test_zone_mit_namen(run_gb):
    """Der Name wird entlang des Bogens mitgedreht -- hier nur der Aufruf."""
    src = HEAD + (
        'c = CHART_NEW("tacho", 0, 0, 200, 200)\n'
        'CHART_ZONE(c, 0, 50, 16711680, "SCHLECHT")\n'
        'CHART_ZONE(c, 50, 100, 65280)\n'
        'PRINT "ok"\n'
    )
    assert _lines(run_gb(src)) == ["ok"]


@pytest.mark.parametrize("key,wert", [
    ("blatt_teile", "24"), ("blatt_luecke", "3"), ("blatt_dicke", "0.25"), ("fassung", "8"),
])
def test_zifferblatt_zahlen(run_gb, key, wert):
    src = HEAD + (
        'c = CHART_NEW("tacho", 0, 0, 200, 200)\n'
        f'CHART_SET_NUM(c, "{key}", {wert})\n'
        'PRINT "ok"\n'
    )
    assert _lines(run_gb(src)) == ["ok"]
