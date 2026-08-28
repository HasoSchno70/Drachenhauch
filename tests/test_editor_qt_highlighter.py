"""Tests fuer den Highlighter -- speziell die f-String-Erkennung.

Das eigentliche Token-Coloring laeuft via Qt und ist nicht trivial unit-
testbar; die `_find_fstring_ranges`-Hilfsmethode hingegen ist eine reine
Funktion und deckt den fehleranfaelligen Teil ab (Lookahead, Escape,
Wort-Anfang, mehrere f-Strings in derselben Zeile)."""
from drachenhauch.editor_qt.highlighter import DHHighlighter

F = DHHighlighter._find_fstring_ranges


def test_no_fstring_returns_empty():
    assert F('PRINT "normal string"') == []


def test_no_fstring_in_plain_text():
    assert F("foo() + foobar") == []


def test_simple_fstring():
    # 'PRINT f"hi {x}!"' -> f at col 6, f-string spans 10 chars
    assert F('PRINT f"hi {x}!"') == [(6, 10)]


def test_two_fstrings_in_one_line():
    assert F('PRINT f"a" + f"b"') == [(6, 4), (13, 4)]


def test_uppercase_F_recognized():
    assert F('PRINT F"x"') == [(6, 4)]


def test_lowercase_f_at_word_start():
    # 'foo + f"x"' -> f at col 6
    assert F('foo + f"x"') == [(6, 4)]


def test_f_inside_normal_string_ignored():
    # Anfuehrungszeichen vor f bedeutet: f ist Inhalt, kein f-String
    assert F('PRINT "f"') == []


def test_dim_f_is_not_fstring():
    # 'DIM f AS STRING' -- f ist eine Variable, kein f-String-Praefix
    assert F("DIM f AS STRING") == []


def test_f_after_ident_char_is_not_fstring():
    # 'foofbar' -- f ist mitten in einem Wort
    assert F('foofbar"x"') == []


def test_doubled_quotes_inside_fstring():
    # f"hi ""there""!" -- doppelte Quotes sind Escape, nicht Stringende
    src = 'PRINT f"hi ""there""!"'
    assert F(src) == [(6, len(src) - 6)]


def test_unterminated_fstring_extends_to_eol():
    # 'PRINT f"unterminiert' -- 6+14 = bis zum Zeilenende
    src = 'PRINT f"unterminiert'
    assert F(src) == [(6, 14)]


# ------------------------------------------------ Keyword-Klassifikation
# Review-Fund: OPERATOR/YIELD/COROUTINE/NIL sind fertige Sprach-Features,
# wurden aber im Highlighter (und im Completer) wie normale Identifier
# behandelt, weil sie bei ihrer Einfuehrung nicht in die Klassifikations-
# Sets uebernommen wurden. Diese Tests sichern die Klassen ab, damit
# zukuenftige neue Keywords nicht denselben Drift erzeugen.
from drachenhauch.editor_qt.highlighter import line_color_spans


def test_coroutine_type_is_classified():
    spans = line_color_spans("DIM c AS COROUTINE")
    assert ("type" in [s[2] for s in spans])


def test_yield_is_classified_as_ctrl():
    spans = line_color_spans("YIELD 1")
    assert spans[0][2] == "ctrl"


def test_operator_is_classified_as_decl():
    spans = line_color_spans("OPERATOR + (other AS Money) AS Money")
    assert spans[0][2] == "decl"


def test_nil_is_classified_as_bool():
    spans = line_color_spans("PRINT x = NIL")
    assert spans[-1][2] == "bool"


def test_completer_keywords_include_new_features():
    from drachenhauch.editor_qt.completer import KEYWORDS
    for kw in ("OPERATOR", "YIELD", "COROUTINE", "NIL"):
        assert kw in KEYWORDS, kw


# --------------------------------------------------- Builtin-Einfaerbung
# `classify_token` zog seine Builtin-Namen frueher aus einer handgepflegten
# Aufzaehlung im Highlighter (72 Namen, "aus dem CTk-Editor uebernommen").
# Der Befehlssatz ist seither auf ueber 1500 gewachsen -- alles aus `gui`,
# `chart`, `g3d`, `audio` und `m3d` sah im Editor aus wie eine gewoehnliche
# Variable. Quelle ist jetzt `dhrt_meta` (builtin_index.json).

from drachenhauch.lexer import Lexer
from drachenhauch.editor_qt.highlighter import builtin_names, classify_token


def _klasse(quelltext: str, wort: str) -> str | None:
    """Highlight-Klasse des IDENT-Tokens `wort` in `quelltext`."""
    for tok in Lexer(quelltext).tokenize():
        if isinstance(tok.value, str) and tok.value.lower() == wort.lower():
            return classify_token(tok)
    raise AssertionError(f"{wort!r} nicht im Token-Strom von {quelltext!r}")


def test_builtins_der_grossen_module_sind_builtin():
    """Je ein Vertreter der Module, die die alte Liste komplett uebersah."""
    for quelle, wort in [
        ("GUI_DRAW()", "gui_draw"),
        ("CHART_DRAW(c)", "chart_draw"),
        ("CUBE(0, 0, 0, 1, 1, 1)", "cube"),
        ("AUDIO_PLAY(s)", "audio_play"),
        ("VEC2_NEW(1.0, 2.0)", "vec2_new"),
        ("PRINT JSON_GET_INT(h, \"a\")", "json_get_int"),
    ]:
        assert _klasse(quelle, wort) == "builtin", wort


def test_dollar_form_und_kurzform_beide_builtin():
    """Der Lexer liefert `STR$(1)` als `str$` und `STR(1)` als `str` --
    beide rufen dasselbe Builtin, beide muessen gefaerbt werden."""
    assert _klasse("PRINT STR$(1)", "str$") == "builtin"
    assert _klasse("PRINT STR(1)", "str") == "builtin"


def test_eigene_namen_bleiben_ident():
    assert _klasse("meine_variable = 1", "meine_variable") == "ident"
    assert _klasse("SUB spieler_zeichnen()", "spieler_zeichnen") == "ident"


def test_compiler_interna_sind_keine_builtins():
    """`__COMP_ITER` & Co. stehen im Index, schreibt aber niemand von Hand."""
    assert not any(n.startswith("__") for n in builtin_names())


def test_liste_deckt_den_ganzen_index_ab():
    from drachenhauch.editor_qt.dhrt_meta import builtin_names_lower
    fehlend = {n for n in builtin_names_lower() if not n.startswith("__")} - builtin_names()
    assert not fehlend, f"nicht eingefaerbt: {sorted(fehlend)[:10]}"
