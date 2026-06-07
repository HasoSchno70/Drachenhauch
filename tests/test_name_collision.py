"""Namens-Kollisionen (case-insensitiv) zwischen Variablen und aufrufbaren
Namen liefern eine KLARE Meldung statt stillem Fehlverhalten / kryptischem
"nicht aufrufbar".

Im Galaga-Prototyp bissen genau zwei Faelle: eine Variable `shoot` neben einer
SUB `Shoot`, und ein Array `box` neben dem Builtin `BOX`."""

import pytest
from gamebasic.errors import GBRuntimeError


def test_var_shadows_user_sub_is_clear_compile_error(run_gb):
    """Variable verdeckt gleichnamige SUB -> klare Meldung beim Kompilieren."""
    src = '''
DIM shoot AS INTEGER
shoot = 1
SUB Shoot()
    PRINT "bang"
END SUB
Shoot()
'''
    with pytest.raises(GBRuntimeError, match="Namens-Kollision.*[Vv]ariable.*SUB"):
        run_gb(src)


def test_var_shadows_builtin_call_names_the_builtin(run_gb):
    """Variable verdeckt einen Builtin -> Laufzeitmeldung nennt Variable + Befehl."""
    src = '''
DIM box AS INTEGER
box = 5
BOX(0, 0, 10, 10, 255)
'''
    with pytest.raises(GBRuntimeError, match="'box'.*BOX"):
        run_gb(src)


def test_funcref_variable_call_still_works(run_gb):
    """Regression: eine echte FUNCREF-Variable (kein Namens-Konflikt) bleibt aufrufbar."""
    src = '''
FUNCTION square(x AS INTEGER) AS INTEGER
    RETURN x * x
END FUNCTION
DIM f AS FUNCREF
f = square
PRINT f(5)
'''
    assert run_gb(src) == "25\n"


def test_harmless_variable_named_like_builtin_still_works(run_gb):
    """Eine Variable, die nur als Wert benutzt wird (nie als Befehl aufgerufen),
    darf weiter so heissen wie ein Builtin -- kein Fehler."""
    src = '''
DIM text AS STRING
text = "hallo"
PRINT text
'''
    assert run_gb(src) == "hallo\n"


def test_var_vs_enum_is_clear_compile_error(run_gb):
    """Variable kollidiert case-insensitiv mit einem ENUM -> klare Meldung beim
    Kompilieren statt kryptischem "CONST kann nicht ueberschrieben werden"."""
    src = '''
DIM mode AS INTEGER
ENUM Mode = A, B
mode = 1
'''
    with pytest.raises(GBRuntimeError, match=r"Namens-Kollision.*mode.*ENUM"):
        run_gb(src)


def test_enum_then_var_is_clear_compile_error(run_gb):
    """Gleiche Kollision in umgekehrter Reihenfolge (ENUM zuerst)."""
    src = '''
ENUM Mode = A, B
DIM mode AS INTEGER
'''
    with pytest.raises(GBRuntimeError, match=r"Namens-Kollision.*mode"):
        run_gb(src)


def test_var_vs_const_is_clear_compile_error(run_gb):
    src = '''
CONST X AS INTEGER = 5
DIM x AS INTEGER
'''
    with pytest.raises(GBRuntimeError, match=r"Namens-Kollision.*'x'.*CONST"):
        run_gb(src)


def test_dim_and_for_same_name_still_allowed(run_gb):
    """Variable-vs-Variable (DIM + FOR derselbe Name) ist erlaubt -- kein Fehler."""
    src = '''
DIM i AS INTEGER
FOR i = 1 TO 3
    PRINT i
NEXT i
'''
    assert run_gb(src) == "1\n2\n3\n"


def test_enum_with_distinct_variable_name_works(run_gb):
    """Ein ENUM und eine ANDERS benannte Variable funktionieren normal."""
    src = '''
ENUM St = X, Y
DIM s AS INTEGER
s = St.Y
PRINT s
'''
    assert run_gb(src) == "1\n"
