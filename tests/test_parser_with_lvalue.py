"""WITH-Block und Lvalue-Kette im Python-Parser.

Aus dem Clean-Code-Review des Frontends. Kernbefund: `parser.py` ist der
stale twin von `parser.rs` -- mehrere `// Review-Fund`-Fixes auf der
Rust-Seite wurden nie zurueckportiert. Der Python-Parser bedient nur noch
die Editor-Schicht (gbrt ist die Laufzeit), aber `_check_syntax_only`
liefert nur das ERSTE Problem: ein Fehlalarm verdeckt damit alle echten
Fehler der Datei.

Leitprinzip der Tests: der Editor muss genau das akzeptieren, was gbrt
auch ausfuehrt -- nicht mehr (sonst schweigt er bei kaputtem Code) und
nicht weniger (sonst streicht er gueltigen Code rot an).
"""
import pytest

from gamebasic.lexer import Lexer
from gamebasic.parser import Parser
from gamebasic.errors import ParseError


CLS = ("CLASS P\n"
       "    DIM hp AS INTEGER\n"
       "    SUB go()\n"
       "    END SUB\n"
       "END CLASS\n"
       "DIM p AS P\n"
       "p = NEW P()\n")


def parse(src):
    return Parser(Lexer(src).tokenize()).parse()


def _first(prog, type_name):
    return [s for s in prog.statements if type(s).__name__ == type_name][0]


# --- H3: Methodenaufruf im WITH-Block (gbrt fuehrt das aus) ----------

@pytest.mark.parametrize("body", [".go()", ".hp.go()", ".go().x"])
def test_method_call_in_with_is_not_an_error(body):
    """`.go()` lief in ein hartes "Erwartet '=' oder Compound-Operator",
    weil die Lvalue-Kette `(` nicht kennt -- ein Aufruf ist ja kein
    Zuweisungsziel. Der Editor strich damit voellig gueltigen Code rot an,
    den gbrt korrekt ausfuehrt."""
    prog = parse(CLS + f"WITH p\n{body}\nEND WITH\n")
    stmt = _first(prog, "With").body[0]
    assert type(stmt).__name__ == "ExprStmt"


def test_member_assign_in_with_still_works():
    """Regression: der Rewind darf die normale Zuweisung nicht brechen."""
    prog = parse(CLS + "WITH p\n.hp = 1\nEND WITH\n")
    assert type(_first(prog, "With").body[0]).__name__ == "MemberAssign"


def test_compound_assign_in_with_still_works():
    prog = parse(CLS + "WITH p\n.hp += 1\nEND WITH\n")
    assert type(_first(prog, "With").body[0]).__name__ == "MemberAssign"


# --- M4: Membernamen, die wie Keywords lexen ------------------------

@pytest.mark.parametrize("src", [
    "spr.image = 5",     # IMAGE ist ein Typ-Keyword
    "obj.data = 1",      # DATA ist ein Statement-Keyword
    "n.next = 2",        # NEXT ist ein Schleifen-Keyword
    "o.sound = 3",       # SOUND ist ein Typ-Keyword
    "o.file = 4",        # FILE ebenso
])
def test_keyword_member_assignment_is_recognised(src):
    """Die drei Zuweisungs-Pfade verlangten IDENT nach '.', waehrend der
    Lese-Pfad `_postfix` jedes Token mit String-Wert akzeptierte. Folge:
    `spr.image = 5` wurde nicht einmal als Zuweisung ERKANNT und landete
    still als verworfener `=`-Vergleich im AST -- ohne Fehlermeldung.
    parser.rs hatte dafuer laengst `member_name_after_dot`."""
    stmt = parse(src).statements[0]
    assert type(stmt).__name__ == "MemberAssign", (
        f"{src!r} wurde zu {type(stmt).__name__} statt MemberAssign")


def test_plain_member_assignment_still_works():
    assert type(parse("obj.hp = 1").statements[0]).__name__ == "MemberAssign"


# --- L4: Slice-Zuweisung auf JEDEM Index erkannt --------------------

@pytest.mark.parametrize("src", ["x[1:2] = 5", "x[1, 2:3] = 5", "x[0, 1, 2:9] = 5"])
def test_slice_assignment_reports_the_friendly_message(src):
    """Die Slice-Pruefung deckte nur den ERSTEN Index ab -- `x[1, 2:3] = 5`
    fiel darum in ein generisches "Erwartet ']'" statt in die erklaerende
    Meldung."""
    with pytest.raises(ParseError, match="Slice-Zuweisung"):
        parse(src)


def test_multidim_index_assignment_still_works():
    assert type(parse("x[1, 2] = 5").statements[0]).__name__ == "IndexAssign"


# --- M3: einzeiliges IF -- Grenze klar benennen ---------------------
# gbrt lehnt beide Formen ab (verifiziert per `gbrt --check`:
# "Erwartet Zeilenende"). Frueher verschluckte der Python-Parser sie
# still als `=`-Vergleich; sie zu AKZEPTIEREN waere die falsche Richtung
# gewesen (Editor stumm bei Code, der nicht laeuft).

def test_inline_if_member_assign_is_rejected_like_gbrt():
    with pytest.raises(ParseError, match="einzeiligen IF"):
        parse(CLS + "WITH p\nIF TRUE THEN .hp = 1\nEND WITH\n")


def test_inline_if_tuple_assign_is_rejected_like_gbrt():
    with pytest.raises(ParseError, match="einzeiligen IF"):
        parse("DIM a AS INTEGER\nDIM b AS INTEGER\n"
              "IF TRUE THEN (a, b) = MINMAX(1, 2)\n")


@pytest.mark.parametrize("src", [
    "IF TRUE THEN PRINT 1",
    "IF TRUE THEN x = 1",
    "IF TRUE THEN BREAK",
])
def test_supported_inline_if_forms_still_work(src):
    """Regression: die tatsaechlich unterstuetzten Formen bleiben gueltig."""
    parse("DIM x AS INTEGER\nWHILE TRUE\n" + src + "\nWEND\n")


# --- M2: COLON als Terminator nach PRINT / RETURN -------------------
# gbrt akzeptiert alle vier Formen (per `gbrt --check` verifiziert); der
# Python-Parser warf "Unerwartetes Token COLON" -- ein Fehlalarm auf Code,
# der laeuft. Ein solcher Fehlalarm verdeckt zudem alle echten Fehler der
# Datei, weil `_check_syntax_only` nur das erste Problem liefert.

@pytest.mark.parametrize("src", [
    'PRINT : PRINT "x"',
    'IF TRUE THEN PRINT : PRINT "x"',
])
def test_colon_after_bare_print_is_accepted(src):
    parse(src)


@pytest.mark.parametrize("src", [
    'SUB f()\n    DIM x AS INTEGER\n    x = 1 : RETURN : x = 2\nEND SUB\nf()\n',
    'SUB f()\n    IF TRUE THEN RETURN : PRINT "x"\nEND SUB\nf()\n',
])
def test_colon_after_bare_return_is_accepted(src):
    parse(src)


# --- M8: Sicherheitsnetz gegen still verworfenes '=' ----------------

@pytest.mark.parametrize("src", [
    'DIM a AS INTEGER\n(a) = MINMAX(1, 2)\n',
    'DIM a AS INTEGER\nDIM b AS INTEGER\na + b = 3\n',
])
def test_discarded_toplevel_equality_is_reported(src):
    """Ein Top-Level `=` ist fast immer eine gemeinte ZUWEISUNG, deren Ziel
    der Lookahead nicht erkannt hat. Ohne Netz wird der Ausdruck als
    Vergleich geparst, sein Ergebnis verworfen -- die Zuweisung verschwindet
    spurlos. Genau diese Stille liess die Keyword-Member-Faelle so lange
    unentdeckt. Wortlaut wie gbrt (parser.rs)."""
    with pytest.raises(ParseError, match="meintest du eine Zuweisung"):
        parse(src)


@pytest.mark.parametrize("src", [
    'DIM a AS INTEGER\na = 3\n',                      # echte Zuweisung
    'DIM a AS INTEGER\nIF a = 3 THEN PRINT 1\n',      # echter Vergleich
    'DIM a AS INTEGER\nWHILE a = 3\nWEND\n',
])
def test_real_assignments_and_comparisons_still_parse(src):
    parse(src)


# --- L3: nicht geschlossenes IF meldet an der IF-Zeile --------------

def test_unterminated_if_reports_at_the_if_line():
    """Ohne `_at_end()`-Wache (die jedes andere Blockkonstrukt hat) meldete
    ein offenes IF "Unerwartetes Token EOF" auf der LETZTEN Zeile der Datei
    statt "END IF erwartet" an der oeffnenden Zeile."""
    src = 'PRINT 1\nIF TRUE THEN\n    PRINT 2\nPRINT 3\nPRINT 4\n'
    with pytest.raises(ParseError, match="END IF erwartet") as ei:
        parse(src)
    assert ei.value.line == 2, f"erwartet Zeile 2 (das IF), bekam {ei.value.line}"


# --- L5: Aufruf-Ziele wie in gbrt -----------------------------------

@pytest.mark.parametrize("src", ['PRINT 1(2)', 'PRINT "s"(2)'])
def test_calling_a_literal_is_rejected_like_gbrt(src):
    """`1(2)` parste klaglos als Call durch, waehrend gbrt es ablehnt --
    der Editor blieb stumm bei Code, der zur Laufzeit scheitert."""
    with pytest.raises(ParseError, match="nicht aufrufbar"):
        parse(src)


@pytest.mark.parametrize("src", [
    'SUB f(n AS INTEGER)\nEND SUB\nf(1)\n',                 # Funktion
    'DIM arr[2] AS FUNCREF\narr[0](1)\n',                   # FUNCREF im Array
    'CLASS C\n SUB m(n AS INTEGER)\n END SUB\nEND CLASS\n'
    'DIM o AS C\no = NEW C()\no.m(1)\n',                    # Methode
])
def test_legitimate_call_targets_still_work(src):
    parse(src)


# --- L6: tiefe Verschachtelung ergibt einen positionierten Fehler ---

def test_deeply_nested_expression_gives_a_positioned_parse_error():
    """Jede Ebene kostet ~12 Stack-Frames durch die feste Praezedenz-Kette.
    Ohne eigene Grenze knallte es in Pythons RecursionError -- den faengt
    `_check_syntax_only` zwar ab, meldet dem Nutzer dann aber "maximum
    recursion depth exceeded" auf Zeile 1 statt zu sagen, wo und was."""
    src = "PRINT " + "(" * 120 + "1" + ")" * 120
    with pytest.raises(ParseError, match="zu tief verschachtelt"):
        parse(src)


@pytest.mark.parametrize("n", [1, 5, 20])
def test_normal_nesting_depth_still_parses(n):
    parse("PRINT " + "(" * n + "1" + ")" * n)
