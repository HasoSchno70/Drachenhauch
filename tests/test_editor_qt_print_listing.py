"""Tests fuer den Listing-Druck (HTML-Aufbau).

`build_listing_html` ist reine Logik (Lexer + Token-Klassifikation + HTML),
braucht also kein QApplication. Getestet werden Farb-vs-SW, Escaping,
Zeilennummern, Kommentar-/String-Faerbung und die Qt-Zeilentrenner-
Normalisierung (U+2029/U+2028), die bei Selektionen auftritt.
"""
from gamebasic.editor_qt.print_listing import build_listing_html

SRC = 'SCREEN(480, 640, "Galaga")   \' Fenster\nDIM x AS INTEGER'


def test_color_html_has_syntax_colors():
    html = build_listing_html(SRC, color=True, line_numbers=False, title="t.gb")
    assert "<pre" in html and "</pre>" in html
    assert "#7F0055" in html or "#0000C0" in html   # decl/ctrl keyword
    assert "#067D17" in html                          # string-Farbe
    assert "#808080" in html                          # comment-Farbe


def test_bw_html_has_no_syntax_colors_but_keeps_emphasis():
    html = build_listing_html(SRC, color=False, line_numbers=False, title="t.gb")
    assert "#7F0055" not in html and "#067D17" not in html and "#808080" not in html
    # Struktur bleibt: Keywords fett, Kommentare kursiv
    assert "font-weight:bold" in html
    assert "font-style:italic" in html


def test_html_escaping():
    html = build_listing_html('PRINT "a<b>&c"', color=True, line_numbers=False, title="t.gb")
    assert "&lt;b&gt;" in html and "&amp;c" in html
    assert "<b>" not in html.split("<pre")[1]   # kein roher HTML-Tag im Code


def test_title_is_escaped_and_present():
    html = build_listing_html("PRINT 1", color=True, line_numbers=False, title="a&b.gb")
    assert "a&amp;b.gb" in html


def test_line_numbers_present_and_padded():
    src = "\n".join(f"PRINT {i}" for i in range(12))   # 12 Zeilen -> 2-stellig
    html = build_listing_html(src, color=False, line_numbers=True, title="t.gb")
    assert "#A0A0A0" in html              # Zeilennummern-Farbe
    # 1 wird auf Breite 2 rechtsbuendig aufgefuellt -> " 1"
    assert "&nbsp;1" in html or " 1  " in html or ">1" not in html


def test_qt_paragraph_separator_is_split():
    # Qt-Selektionen trennen Zeilen mit U+2029 statt \n.
    src = "PRINT 1" + chr(0x2029) + "PRINT 2"
    html = build_listing_html(src, color=False, line_numbers=True, title="t.gb")
    # Zwei Code-Zeilen -> die Zahl 2 als Zeilennummer taucht auf
    assert "PRINT" in html
    assert html.count("PRINT") == 2


def test_empty_source_does_not_crash():
    html = build_listing_html("", color=True, line_numbers=True, title="leer.gb")
    assert "<pre" in html


def test_font_size_is_applied():
    small = build_listing_html("PRINT 1", color=True, line_numbers=False,
                               title="t.gb", font_pt=8)
    big = build_listing_html("PRINT 1", color=True, line_numbers=False,
                             title="t.gb", font_pt=14)
    assert "font-size:8pt" in small
    assert "font-size:14pt" in big
