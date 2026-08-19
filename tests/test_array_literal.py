"""A1: Array-Literale `[a, b, c]`.

Frueher kollidierte `[...]` ausschliesslich mit der List-Comprehension
(`a = [1,2,3]` -> „Erwartet FOR in List-Comprehension"). Jetzt disambiguiert
der Parser: FOR nach dem ersten Ausdruck = Comprehension, sonst Array-Literal.
"""
import pytest
from drachenhauch.errors import DrachenhauchError


def test_int_array_literal(run_gb):
    assert run_gb("DIM a AS ARRAY OF INTEGER\n"
                  "a = [10, 20, 30]\n"
                  "PRINT a[0], a[2], LEN(a)\n") == "10 30 3\n"


def test_float_promotion(run_gb):
    # gemischte Zahlen -> FLOAT-Array (Ints hochgezogen)
    assert run_gb("DIM f AS ARRAY OF FLOAT\n"
                  "f = [1.5, 2, 3.5]\n"
                  "PRINT f[1]\n") == "2.0\n"


def test_string_array_literal(run_gb):
    assert run_gb('DIM s AS ARRAY OF STRING\n'
                  's = ["hallo", "welt"]\n'
                  'PRINT s[0]; "-"; s[1]\n') == "hallo-welt\n"


def test_array_literal_as_iterable(run_gb):
    assert run_gb("DIM sum AS INTEGER\n"
                  "sum = 0\n"
                  "FOR EACH n IN [1, 2, 3, 4]\n"
                  "    sum = sum + n\n"
                  "NEXT\n"
                  "PRINT sum\n") == "10\n"


def test_single_element_array(run_gb):
    assert run_gb("DIM a AS ARRAY OF INTEGER\n"
                  "a = [5]\n"
                  "PRINT LEN(a), a[0]\n") == "1 5\n"


def test_trailing_comma(run_gb):
    assert run_gb("PRINT [1, 2, 3,][2]\n") == "3\n"


def test_sort_on_literal_array(run_gb):
    assert run_gb("DIM a AS ARRAY OF INTEGER\n"
                  "a = [3, 1, 2]\n"
                  "SORT(a)\n"
                  "PRINT a[0], a[1], a[2]\n") == "1 2 3\n"


def test_comprehension_still_works(run_gb):
    # `[expr FOR ...]` bleibt eine Comprehension -- inkl. ueber ein Array-Literal.
    assert run_gb("DIM sq AS TUPLE\n"
                  "sq = [x * x FOR x IN [1, 2, 3]]\n"
                  "PRINT sq\n") == "(1, 4, 9)\n"


# --- Leeres Array-Literal ---------------------------------------------
#
# Frueherer Stolperstein: `[]` brach mit "Leeres Array-Literal -- Typ
# unbekannt" ab. Ein Widget, das seine Daten erst spaeter bekommt, brauchte
# deshalb einen Platzhalter-Eintrag. Jetzt gibt der Ort den Elementtyp vor,
# an dem das Literal landet.

def test_leeres_array_literal_ist_erlaubt(run_gb):
    assert run_gb("DIM a AS ARRAY OF INTEGER\na = []\nPRINT LEN(a)\n") == "0\n"


def test_leeres_literal_uebernimmt_den_typ_der_variablen(run_gb):
    """Sonst waere `a = []` ein stilles Loch in der Typpruefung."""
    with pytest.raises(DrachenhauchError, match=r"Erwartet STRING"):
        run_gb('DIM a AS ARRAY OF STRING\n'
               'a = []\n'
               'ARRAY_PUSH(a, 42)\n')


def test_leeres_literal_als_parameter(run_gb):
    assert run_gb('SUB zeige(liste AS ARRAY OF STRING)\n'
                  '    PRINT LEN(liste)\n'
                  'END SUB\n'
                  'zeige([])\n') == "0\n"


def test_leeres_literal_als_rueckgabewert(run_gb):
    with pytest.raises(DrachenhauchError, match=r"Erwartet STRING"):
        run_gb('FUNCTION leer() AS ARRAY OF STRING\n'
               '    RETURN []\n'
               'END FUNCTION\n'
               'DIM x AS ARRAY OF STRING\n'
               'x = leer()\n'
               'ARRAY_PUSH(x, 42)\n')


def test_ohne_kontext_bleibt_es_any(run_gb):
    """Ohne typisiertes Ziel nimmt das Array alles auf -- ARRAY OF ANY."""
    assert run_gb('DIM frei AS ARRAY OF ANY\n'
                  'frei = []\n'
                  'ARRAY_PUSH(frei, "Text")\n'
                  'ARRAY_PUSH(frei, 42)\n'
                  'PRINT LEN(frei)\n') == "2\n"


def test_leeres_array_wird_von_builtins_angenommen(run_gb):
    """Ein leeres Array kann keinen falschen Wert enthalten -- Builtins, die
    ARRAY OF STRING erwarten, duerfen es nicht wegen seines Typs ablehnen.
    Geprueft an JOIN$, das dieselbe Pruefung nutzt wie GUI_DROPDOWN."""
    assert run_gb('PRINT "[" + JOIN$([], ",") + "]"\n') == "[]\n"


# --- Elementtyp bei der Zuweisung ---------------------------------------
#
# Frueherer Stolperstein: `DIM texte AS ARRAY OF STRING : texte = zahlen`
# ging still durch, auch wenn `zahlen` ein ARRAY OF INTEGER war -- coerce()
# reichte alle Referenz-Typen einfach durch. Bei einfachen Werten meldet die
# Runtime so etwas seit jeher; der Fehler fiel deshalb erst weit entfernt auf,
# beim Lesen eines Elements mit dem falschen Typ.

def test_falscher_elementtyp_wird_gemeldet(run_gb):
    with pytest.raises(DrachenhauchError, match=r"Erwartet ARRAY OF STRING, erhalten ARRAY OF INTEGER"):
        run_gb('DIM zahlen AS ARRAY OF INTEGER\n'
               'zahlen = [1, 2, 3]\n'
               'DIM texte AS ARRAY OF STRING\n'
               'texte = zahlen\n')


def test_falscher_elementtyp_als_parameter(run_gb):
    with pytest.raises(DrachenhauchError, match=r"Parameter: Erwartet ARRAY OF STRING"):
        run_gb('SUB zeige(liste AS ARRAY OF STRING)\n'
               '    PRINT LEN(liste)\n'
               'END SUB\n'
               'DIM zahlen AS ARRAY OF INTEGER\n'
               'zahlen = [1, 2]\n'
               'zeige(zahlen)\n')


def test_einfacher_wert_an_array_ziel(run_gb):
    with pytest.raises(DrachenhauchError, match=r"Erwartet ARRAY OF INTEGER, erhalten INTEGER"):
        run_gb('DIM a AS ARRAY OF INTEGER\n'
               'a = 42\n')


def test_gleicher_elementtyp_geht_weiterhin(run_gb):
    assert run_gb('DIM a AS ARRAY OF STRING\n'
                  'a = SPLIT$("x|y|z", "|")\n'
                  'DIM b AS ARRAY OF STRING\n'
                  'b = a\n'
                  'PRINT LEN(b); b[2]\n') == "3z\n"


def test_array_of_any_bleibt_zuweisbar(run_gb):
    """ARRAY OF ANY kann jeden Wert enthalten -- es einem engeren Ziel zu
    geben ist eine bewusste Entscheidung, so wie das Auspacken einer Map."""
    assert run_gb('DIM frei AS ARRAY OF ANY\n'
                  'frei = []\n'
                  'ARRAY_PUSH(frei, "Text")\n'
                  'DIM texte AS ARRAY OF STRING\n'
                  'texte = frei\n'
                  'PRINT texte[0]\n') == "Text\n"


def test_ganzzahl_literal_an_float_ziel(run_gb):
    """`[1, 2, 3]` an einem FLOAT-Ziel: die Zellen werden umgebaut, statt die
    Zuweisung abzulehnen -- sonst waere ein FLOAT-Array ohne Komma-Werte im
    Literal nicht mehr zu schreiben."""
    assert run_gb('DIM werte AS ARRAY OF FLOAT\n'
                  'werte = [1, 2, 3]\n'
                  'werte[0] = 1.5\n'
                  'PRINT werte[0]; " "; werte[1]\n') == "1.5 2.0\n"


def test_vorhandenes_int_array_wird_nicht_zu_float(run_gb):
    """Umbauen darf nur ein frisches Literal. Ein vorhandenes ARRAY OF INTEGER
    wuerde sonst unter der Hand zu FLOAT -- sein bisheriger Name zeigt ja
    weiter auf dieselben Zellen."""
    with pytest.raises(DrachenhauchError, match=r"Erwartet ARRAY OF FLOAT, erhalten ARRAY OF INTEGER"):
        run_gb('DIM ganz AS ARRAY OF INTEGER\n'
               'ganz = [7, 8]\n'
               'DIM komma AS ARRAY OF FLOAT\n'
               'komma = ganz\n')


def test_unzugewiesenes_array_bleibt_erlaubt(run_gb):
    """Ein sizeless ARRAY ist bis zur ersten Zuweisung NIL."""
    assert run_gb('DIM leer AS ARRAY OF STRING\n'
                  'PRINT "ok"\n') == "ok\n"


def test_rueckgabewert_prueft_den_elementtyp(run_gb):
    with pytest.raises(DrachenhauchError, match=r"RETURN: Erwartet ARRAY OF STRING"):
        run_gb('FUNCTION namen() AS ARRAY OF STRING\n'
               '    DIM zahlen AS ARRAY OF INTEGER\n'
               '    zahlen = [1, 2]\n'
               '    RETURN zahlen\n'
               'END FUNCTION\n'
               'PRINT LEN(namen())\n')
