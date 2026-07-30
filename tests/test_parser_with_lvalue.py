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
