"""Namens-Kollisionen (case-insensitiv) zwischen Variablen und aufrufbaren
Namen liefern eine KLARE Meldung statt stillem Fehlverhalten / kryptischem
"nicht aufrufbar".

Im Galaga-Prototyp bissen genau zwei Faelle: eine Variable `shoot` neben einer
SUB `Shoot`, und ein Array `box` neben dem Builtin `BOX`."""

import pytest
from drachenhauch.errors import DHRuntimeError


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
    with pytest.raises(DHRuntimeError, match="Namens-Kollision.*[Vv]ariable.*SUB"):
        run_gb(src)


def test_var_shadows_builtin_call_names_the_builtin(run_gb):
    """Eine FUNCREF-Variable verdeckt einen Builtin -> die Laufzeitmeldung
    nennt Variable + Befehl und sagt, welche Variablen ueberhaupt verdecken.

    Bis 2026-09-04 galt das fuer JEDE Variable, auch `DIM len AS INTEGER`
    neben `LEN(s)` -- siehe test_variable_wie_builtin.py fuer die neue
    Regel (bekannter Typ, kein FUNCREF -> der Builtin ist gemeint)."""
    src = '''
DIM box AS FUNCREF
BOX(0, 0, 10, 10, 255)
'''
    with pytest.raises(DHRuntimeError, match="'box'.*BOX"):
        run_gb(src)


def test_typed_variable_named_like_builtin_calls_the_builtin(run_gb):
    src = '''
DIM len AS INTEGER
len = 5
PRINT LEN("abc") + len
'''
    assert run_gb(src).strip() == "8"


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
    with pytest.raises(DHRuntimeError, match=r"Namens-Kollision.*mode.*ENUM"):
        run_gb(src)


def test_enum_then_var_is_clear_compile_error(run_gb):
    """Gleiche Kollision in umgekehrter Reihenfolge (ENUM zuerst)."""
    src = '''
ENUM Mode = A, B
DIM mode AS INTEGER
'''
    with pytest.raises(DHRuntimeError, match=r"Namens-Kollision.*mode"):
        run_gb(src)


def test_var_vs_const_is_clear_compile_error(run_gb):
    src = '''
CONST X AS INTEGER = 5
DIM x AS INTEGER
'''
    with pytest.raises(DHRuntimeError, match=r"Namens-Kollision.*'x'.*CONST"):
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


# --------------------------------------------------------------------------
# `DIM` in einem Block ist genauso global wie eines daneben
#
# `collect_globals` lief bis 2026-08-31 nur ueber die OBERSTE Anweisungsliste.
# Ein `DIM` in einem IF/WHILE/FOR bekam deshalb keinen globalen Platz und
# wurde ueber seinen NAMEN angelegt -- und `DECLARE_NAME` laesst einen schon
# vorhandenen Eintrag stehen. Bei den vorbelegten Konstanten (18 Farbnamen,
# alle KEY_*, `pi`, `tau`) war das die Konstante selbst, und die naechste
# Zuweisung scheiterte mit "CONST kann nicht ueberschrieben werden".
#
# Das Tueckische daran: dieselbe Zeile lief oben im Programm einwandfrei,
# `dhrt --check` schwieg, und die Meldung zeigte auf die Konstante statt auf
# den Variablennamen. Gefunden im Sprite-Editor (`DIM pi` in der Hauptschleife).

@pytest.mark.parametrize("name", ["pi", "tau", "red", "green", "blue", "key_space"])
def test_dim_darf_eine_eingebaute_konstante_verschatten(run_gb, name):
    """Oben ging es immer -- in einem Block jetzt auch."""
    assert run_gb(f'''
IF TRUE THEN
    DIM {name} AS INTEGER
    {name} = 5
    PRINT {name}
END IF
''').strip() == "5"


@pytest.mark.parametrize("kopf,fuss", [
    ("IF TRUE THEN", "END IF"),
    ("WHILE n < 1", "WEND"),
    ("FOR n = 1 TO 1", "NEXT"),
    ("SELECT CASE 1\n    CASE 1", "END SELECT"),
    ("TRY", "CATCH e\n    PRINT \"x\"\nEND TRY"),
])
def test_in_jeder_blockart(run_gb, kopf, fuss):
    assert run_gb(f'''
DIM n AS INTEGER : n = 0
{kopf}
    DIM pi AS INTEGER
    pi = 5
    PRINT pi
    n = 1
{fuss}
''').strip() == "5"


def test_unverschattet_gilt_die_konstante_weiter(run_gb):
    """Gegenprobe -- verschattet wird nur, wo wirklich ein DIM steht."""
    assert run_gb('PRINT INT(PI * 100)\nPRINT RED\nPRINT KEY_SPACE\n'
                  ).split() == ["314", "16711680", "32"]


def test_wiederholtes_dim_setzt_den_wert_nicht_zurueck(run_gb):
    """Ein `DIM` in einer Schleife laeuft bei jedem Durchlauf -- es darf den
    Wert NICHT zuruecksetzen. Galt vorher fuer den Namens-Weg und muss nach
    der Umstellung auf Plaetze genauso gelten."""
    assert run_gb('''
DIM n AS INTEGER
FOR n = 1 TO 3
    DIM z AS INTEGER
    z = z + 1
NEXT
PRINT z
''').strip() == "3"


def test_kollision_wird_auch_ueber_blockgrenzen_erkannt(run_gb):
    """Nebenertrag derselben Ursache: die Kollisions-Erkennung sah bis dahin
    nur Geschwister, ein `DIM` im Block gegen eine CONST weiter oben also
    nicht."""
    with pytest.raises(DHRuntimeError, match="Namens-Kollision.*CONST"):
        run_gb('CONST Modus = 3\nIF TRUE THEN\n    DIM modus AS INTEGER\nEND IF\n')
