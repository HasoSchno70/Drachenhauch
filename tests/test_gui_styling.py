"""Golden-Tests fuer das GUI-Styling (Phase 4): enabled-Zustand, per-Widget-
Font/-Groesse, corner_radius-Metrik. Headless (State/Serialisierung); die
visuelle Wirkung (Ausgrauen, runde Ecken, Font-Rendering) braucht SCREEN ->
manuell verifiziert.
"""
import pytest

from drachenhauch.errors import DrachenhauchError

_W = ('IMPORT "gui"\n'
      'DIM win AS GUI_WINDOW\nwin = GUI_WINDOW("S", 0, 0, 300, 200)\n'
      'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "OK", 20, 40, 120, 30)\n')


def test_enabled_toggle(run_gb):
    out = run_gb(_W +
        'PRINT GUI_ENABLED(b)\n'
        'GUI_SET_ENABLED(b, FALSE)\n'
        'PRINT GUI_ENABLED(b)\n')
    assert out.splitlines() == ["TRUE", "FALSE"]


def test_disabled_still_hit_testable(run_gb):
    # Fuer den Editor: deaktivierte Widgets bleiben selektierbar (GUI_HIT_TEST),
    # nur die Interaktion (Klick/Hover) ist unterbunden.
    out = run_gb(_W +
        'GUI_SET_ENABLED(b, FALSE)\n'
        'PRINT GUI_HIT_TEST(60, 77) = b\n')   # Mitte des Buttons (abs: y=40+22+15)
    assert out.strip() == "TRUE"


def test_font_setters_and_roundtrip(run_gb):
    out = run_gb(_W +
        'GUI_SET_ENABLED(b, FALSE)\n'
        'GUI_SET_FONT_SIZE(b, 24)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_ENABLED(GUI_WINDOW_WIDGET(w2, 0))\n')
    assert out.strip() == "FALSE"


def test_font_size_negative_raises(run_gb):
    with pytest.raises(DrachenhauchError, match="GUI_SET_FONT_SIZE"):
        run_gb(_W + 'GUI_SET_FONT_SIZE(b, -5)\n')


def test_corner_radius_metric(run_gb):
    out = run_gb('IMPORT "gui"\n'
        'PRINT GUI_METRIC_GET("corner_radius")\n'   # Default 0
        'GUI_METRIC_SET("corner_radius", 8)\n'
        'PRINT GUI_METRIC_GET("corner_radius")\n')
    assert out.splitlines() == ["0", "8"]


def test_set_font_accepts_handle(run_gb):
    # GUI_SET_FONT akzeptiert ein (auch -1=Default) Handle ohne Fehler.
    out = run_gb(_W + 'GUI_SET_FONT(b, -1)\nPRINT GUI_KIND(b)\n')
    assert out.strip() == "button"


# --- Plastik-Metriken + Glas-Themen ---------------------------------------
#
# Verlauf, Glanzkante und Fase sind Metriken, keine Farben -- so bleibt ein
# Thema ein KOMPLETTER Look (Farben + Plastik) statt beides von Hand
# kombinieren zu muessen.

def test_glas_themen_schalten_die_plastik_ein(run_gb):
    src = ('IMPORT "gui"\n'
           'GUI_THEME_PRESET("glas_dunkel")\n'
           'PRINT GUI_METRIC_GET("gradient")\n'
           'PRINT GUI_METRIC_GET("gloss")\n'
           'PRINT GUI_METRIC_GET("bevel")\n'
           'PRINT GUI_METRIC_GET("corner_radius")\n')
    assert [l.strip() for l in run_gb(src).split("\n") if l.strip()] == ["16", "26", "1", "5"]


def test_flache_themen_bleiben_flach(run_gb):
    """Bestehende Themen duerfen sich nicht veraendern -- sonst saehen alle
    schon geschriebenen Programme ploetzlich anders aus."""
    src = ('IMPORT "gui"\n'
           'GUI_THEME_PRESET("dark")\n'
           'PRINT GUI_METRIC_GET("gradient") + GUI_METRIC_GET("gloss") + GUI_METRIC_GET("bevel")\n'
           'GUI_THEME_PRESET("modern_dark")\n'
           'PRINT GUI_METRIC_GET("gradient") + GUI_METRIC_GET("gloss") + GUI_METRIC_GET("bevel")\n')
    assert [l.strip() for l in run_gb(src).split("\n") if l.strip()] == ["0", "0"]


def test_beide_glas_themen_gibt_es_hell_und_dunkel(run_gb):
    src = ('IMPORT "gui"\n'
           'GUI_THEME_PRESET("glas_dunkel")\n'
           'PRINT GUI_THEME_GET("win_bg")\n'
           'GUI_THEME_PRESET("glas_hell")\n'
           'PRINT GUI_THEME_GET("win_bg")\n')
    zeilen = [l.strip() for l in run_gb(src).split("\n") if l.strip()]
    assert zeilen[0] != zeilen[1], "hell und dunkel haben denselben Hintergrund"
    assert int(zeilen[0]) < int(zeilen[1]), "dunkel muss dunkler sein als hell"


def test_plastik_metriken_sind_einzeln_setzbar(run_gb):
    src = ('IMPORT "gui"\n'
           'GUI_METRIC_SET("gradient", 30)\n'
           'GUI_METRIC_SET("gloss", 50)\n'
           'GUI_METRIC_SET("bevel", 1)\n'
           'PRINT GUI_METRIC_GET("gradient")\n')
    assert [l.strip() for l in run_gb(src).split("\n") if l.strip()] == ["30"]


# --- Neue Bedienelemente: Kippschalter, Drehregler, runde Knoepfe ---------

def _z(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def test_kippschalter_verhaelt_sich_wie_ein_kaestchen(run_gb):
    """Der Zustand liegt in `checked` -- damit gelten GUI_CHECKED und
    GUI_SET_CHECKED unveraendert, ohne eigene Abfragen."""
    src = ('IMPORT "gui"\n'
           'DIM w AS GUI_WINDOW\n'
           'DIM t AS GUI_WIDGET\n'
           'w = GUI_WINDOW("W", 0, 0, 200, 120)\n'
           't = GUI_TOGGLE(w, "Musik", 10, 10, TRUE)\n'
           'PRINT GUI_CHECKED(t)\n'
           'GUI_SET_CHECKED(t, FALSE)\n'
           'PRINT GUI_CHECKED(t)\n'
           'PRINT GUI_KIND(t)\n')
    assert _z(run_gb(src)) == ["TRUE", "FALSE", "toggle"]


def test_drehregler_haelt_seinen_wert(run_gb):
    src = ('IMPORT "gui"\n'
           'DIM w AS GUI_WINDOW\n'
           'DIM k AS GUI_WIDGET\n'
           'w = GUI_WINDOW("W", 0, 0, 200, 200)\n'
           'k = GUI_KNOB(w, 10, 10, 80, 0.0, 100.0, 72.0)\n'
           'PRINT GUI_VALUE(k)\n'
           'GUI_SET_VALUE(k, 30.0)\n'
           'PRINT GUI_VALUE(k)\n'
           'PRINT GUI_KIND(k)\n')
    assert _z(run_gb(src)) == ["72.0", "30.0", "knob"]


def test_drehregler_lehnt_leeren_bereich_ab(run_gb):
    src = ('IMPORT "gui"\n'
           'DIM w AS GUI_WINDOW\n'
           'w = GUI_WINDOW("W", 0, 0, 200, 200)\n'
           'TRY\n'
           '    GUI_KNOB(w, 10, 10, 80, 5.0, 5.0, 5.0)\n'
           'CATCH e\n'
           '    PRINT e\n'
           'END TRY\n')
    out = run_gb(src)
    assert "max" in out and "min" in out


def test_runde_knoepfe_sind_schaltbar(run_gb):
    src = ('IMPORT "gui"\n'
           'DIM w AS GUI_WINDOW\n'
           'DIM b AS GUI_WIDGET\n'
           'w = GUI_WINDOW("W", 0, 0, 200, 120)\n'
           'b = GUI_BUTTON(w, ">", 10, 10, 40, 40)\n'
           'GUI_SET_ROUND(b, TRUE)\n'
           'GUI_SET_ROUND(b, FALSE)\n'
           'PRINT "ok"\n')
    assert _z(run_gb(src)) == ["ok"]


def test_kippschalter_startet_ohne_vorgabe_aus(run_gb):
    src = ('IMPORT "gui"\n'
           'DIM w AS GUI_WINDOW\n'
           'DIM t AS GUI_WIDGET\n'
           'w = GUI_WINDOW("W", 0, 0, 200, 120)\n'
           't = GUI_TOGGLE(w, "Aus", 10, 10)\n'
           'PRINT GUI_CHECKED(t)\n')
    assert _z(run_gb(src)) == ["FALSE"]


# --- 9-Slice-Skins --------------------------------------------------------

def test_skin_setzen_und_wieder_wegnehmen(run_gb):
    src = ('IMPORT "gui"\n'
           'SCREEN(200, 120, "S")\n'
           'DIM b AS IMAGE\n'
           'b = GENTEX_COLOR(32, 32, 16750848)\n'
           'GUI_SKIN("button", b, 8)\n'
           'GUI_SKIN("button", -1)\n'
           'PRINT "ok"\n')
    assert _z(run_gb(src)) == ["ok"]


def test_skin_meldet_unbekannte_widget_art(run_gb):
    src = ('IMPORT "gui"\n'
           'SCREEN(200, 120, "S")\n'
           'DIM b AS IMAGE\n'
           'b = GENTEX_COLOR(32, 32, 16750848)\n'
           'TRY\n'
           '    GUI_SKIN("knopfdruck", b, 8)\n'
           'CATCH e\n'
           '    PRINT e\n'
           'END TRY\n')
    out = run_gb(src)
    assert "knopfdruck" in out


def test_skin_ueberlebt_winzige_widgets(run_gb):
    """Ein Widget kleiner als seine Skin-Raender darf zusammenschrumpfen,
    nicht kaputtgehen -- der Rand wird dafuer gestutzt."""
    src = ('IMPORT "gui"\n'
           'SCREEN(200, 120, "S")\n'
           'DIM w AS GUI_WINDOW\n'
           'DIM b AS IMAGE\n'
           'DIM f AS INTEGER\n'
           'b = GENTEX_COLOR(48, 48, 16750848)\n'
           'w = GUI_WINDOW("W", 5, 5, 180, 100)\n'
           'GUI_BUTTON(w, "x", 5, 5, 6, 4)\n'
           'GUI_SKIN("button", b, 20)\n'
           'FOR f = 1 TO 2\n'
           '    GUI_UPDATE()\n'
           '    GUI_DRAW()\n'
           '    FLIP()\n'
           'NEXT\n'
           'PRINT "ok"\n')
    assert _z(run_gb(src)) == ["ok"]


# --- ui-Modul (Immediate-Mode): dieselbe Plastik ---------------------------

def test_ui_glas_themen_bringen_plastik_mit(run_gb):
    src = ('IMPORT "ui"\n'
           'UI_THEME_PRESET("glas_dunkel")\n'
           'PRINT UI_METRIC_GET("gradient")\n'
           'PRINT UI_METRIC_GET("gloss")\n'
           'PRINT UI_METRIC_GET("bevel")\n'
           'PRINT UI_METRIC_GET("corner_radius")\n')
    assert _z(run_gb(src)) == ["16", "26", "1", "5"]


def test_ui_flache_themen_bleiben_flach(run_gb):
    """Bestehende ui-Themen duerfen sich nicht veraendern."""
    src = ('IMPORT "ui"\n'
           'UI_THEME_PRESET("dark")\n'
           'PRINT UI_METRIC_GET("gradient") + UI_METRIC_GET("gloss") + UI_METRIC_GET("bevel")\n'
           'UI_THEME_PRESET("light")\n'
           'PRINT UI_METRIC_GET("gradient") + UI_METRIC_GET("gloss") + UI_METRIC_GET("bevel")\n')
    assert _z(run_gb(src)) == ["0", "0"]


def test_ui_theme_wechsel_setzt_die_plastik_zurueck(run_gb):
    """Ein Preset ist ein KOMPLETTER Look: wer von glas auf flach wechselt,
    darf keine Woelbung behalten."""
    src = ('IMPORT "ui"\n'
           'UI_THEME_PRESET("glas_hell")\n'
           'PRINT UI_METRIC_GET("gradient")\n'
           'UI_THEME_PRESET("retro")\n'
           'PRINT UI_METRIC_GET("gradient")\n')
    assert _z(run_gb(src)) == ["16", "0"]


def test_ui_beide_glas_themen_gibt_es(run_gb):
    src = ('IMPORT "ui"\n'
           'UI_THEME_PRESET("glas_dunkel")\n'
           'PRINT UI_THEME_GET("win_bg")\n'
           'UI_THEME_PRESET("glas_hell")\n'
           'PRINT UI_THEME_GET("win_bg")\n')
    zeilen = _z(run_gb(src))
    assert int(zeilen[0]) < int(zeilen[1]), "dunkel muss dunkler sein als hell"


def test_ui_plastik_metriken_einzeln_setzbar(run_gb):
    src = ('IMPORT "ui"\n'
           'UI_METRIC_SET("gradient", 24)\n'
           'UI_METRIC_SET("gloss", 40)\n'
           'PRINT UI_METRIC_GET("gradient")\n')
    assert _z(run_gb(src)) == ["24"]


# --- Neue Ereignisse: betreten/verlassen, Fokus/Blur -----------------------
#
# Alle vier sind FLANKEN: sie feuern beim Uebergang, nicht in jedem Bild,
# solange der Zustand anhaelt. Genau das ist die Stelle, die leicht schiefgeht.

def test_hover_und_leave_feuern_je_einmal(run_gb):
    src = ('IMPORT "gui"\n'
           'SCREEN(300, 200, "E")\n'
           'DIM w AS GUI_WINDOW\n'
           'DIM b AS GUI_WIDGET\n'
           'DIM f AS INTEGER\n'
           'w = GUI_WINDOW("W", 10, 10, 280, 160)\n'
           'b = GUI_BUTTON(w, "K", 20, 20, 100, 30)\n'
           'SUB rein() : PRINT "rein" : END SUB\n'
           'SUB raus() : PRINT "raus" : END SUB\n'
           'GUI_ON_HOVER(b, rein)\n'
           'GUI_ON_LEAVE(b, raus)\n'
           'FOR f = 1 TO 8\n'
           '    IF f = 2 THEN MOUSE_SET_POS(60, 55)\n'
           '    IF f = 5 THEN MOUSE_SET_POS(290, 195)\n'
           '    GUI_UPDATE()\n'
           '    CLS(0)\n'
           '    GUI_DRAW()\n'
           '    FLIP()\n'
           'NEXT\n')
    # Genau EIN "rein" und EIN "raus" -- bliebe die Maus stehen und wuerde
    # jedes Bild gefeuert, staende "rein" hier dreimal.
    assert _z(run_gb(src)) == ["rein", "raus"]


def test_fokus_und_blur_folgen_dem_wechsel(run_gb):
    src = ('IMPORT "gui"\n'
           'SCREEN(300, 200, "F")\n'
           'DIM w AS GUI_WINDOW\n'
           'DIM t AS GUI_WIDGET\n'
           'DIM u AS GUI_WIDGET\n'
           'DIM f AS INTEGER\n'
           'w = GUI_WINDOW("W", 10, 10, 280, 160)\n'
           't = GUI_TEXTINPUT(w, 20, 30, 200, 26)\n'
           'u = GUI_TEXTINPUT(w, 20, 70, 200, 26)\n'
           'SUB fa() : PRINT "A auf" : END SUB\n'
           'SUB ba() : PRINT "A zu" : END SUB\n'
           'SUB fb() : PRINT "B auf" : END SUB\n'
           'GUI_ON_FOCUS(t, fa)\n'
           'GUI_ON_BLUR(t, ba)\n'
           'GUI_ON_FOCUS(u, fb)\n'
           'FOR f = 1 TO 8\n'
           '    IF f = 2 THEN GUI_FOCUS(t)\n'
           '    IF f = 5 THEN GUI_FOCUS(u)\n'
           '    GUI_UPDATE()\n'
           '    CLS(0)\n'
           '    GUI_DRAW()\n'
           '    FLIP()\n'
           'NEXT\n')
    # Der Wechsel meldet erst das Verlieren, dann das Bekommen.
    assert _z(run_gb(src)) == ["A auf", "A zu", "B auf"]


def test_ereignisse_ueberleben_speichern_und_laden(run_gb):
    """Der Form-Designer speichert Handler-NAMEN im .dhform; GUI_LOAD stellt
    sie wieder her. Geprueft wird deshalb der ganze Weg -- nicht nur, dass
    der Name im JSON steht, sondern dass der Handler danach auch feuert."""
    src = ('IMPORT "gui"\n'
           'SCREEN(300, 200, "S")\n'
           'DIM w AS GUI_WINDOW\n'
           'DIM w2 AS GUI_WINDOW\n'
           'DIM b AS GUI_WIDGET\n'
           'DIM f AS INTEGER\n'
           'SUB rein() : PRINT "rein" : END SUB\n'
           'w = GUI_WINDOW("W", 10, 10, 280, 160)\n'
           'b = GUI_BUTTON(w, "K", 20, 20, 100, 30)\n'
           'GUI_ON_HOVER(b, rein)\n'
           'w2 = GUI_FROM_JSON(GUI_TO_JSON(w))\n'
           'GUI_WINDOW_DESTROY(w)\n'
           'FOR f = 1 TO 6\n'
           '    IF f = 2 THEN MOUSE_SET_POS(60, 55)\n'
           '    GUI_UPDATE()\n'
           '    CLS(0)\n'
           '    GUI_DRAW()\n'
           '    FLIP()\n'
           'NEXT\n')
    assert _z(run_gb(src)) == ["rein"]
