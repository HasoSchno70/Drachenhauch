"""Tests fuer STATIC CONST Klassen-Members.

Statics werden im globalen Scope unter dem Klassen-Namen als Namespace
abgelegt. Zugriff: `Player.MAX_HP`. Cython-VM braucht einen Recompile,
damit `_ClassStaticNamespace` im LOAD_MEMBER erkannt wird.
"""
import pytest


def test_static_int_const(run_gb, run_vm):
    src = '''
CLASS Player
    STATIC CONST MAX_HP AS INTEGER = 100
    DIM hp AS INTEGER
END CLASS

PRINT Player.MAX_HP
'''
    expected = "100\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_static_string_const(run_gb, run_vm):
    src = '''
CLASS Game
    STATIC CONST TITLE AS STRING = "Awesome"
END CLASS

PRINT Game.TITLE
'''
    expected = "Awesome\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_static_in_method(run_gb, run_vm):
    """Methoden koennen ihre eigenen Statics nutzen."""
    src = '''
CLASS Player
    STATIC CONST MAX_HP AS INTEGER = 100
    DIM hp AS INTEGER
    SUB Init()
        Self.hp = Player.MAX_HP
    END SUB
END CLASS

DIM p AS Player
p = NEW Player()
PRINT p.hp
'''
    expected = "100\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_multiple_statics(run_gb, run_vm):
    src = '''
CLASS Cfg
    STATIC CONST WIDTH AS INTEGER = 800
    STATIC CONST HEIGHT AS INTEGER = 600
    STATIC CONST FPS AS INTEGER = 60
    STATIC CONST TITLE AS STRING = "Demo"
END CLASS

PRINT Cfg.WIDTH, Cfg.HEIGHT, Cfg.FPS
PRINT Cfg.TITLE
'''
    expected = "800 600 60\nDemo\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_static_negative_number(run_gb, run_vm):
    src = '''
CLASS Limits
    STATIC CONST MIN_X AS INTEGER = -100
END CLASS

PRINT Limits.MIN_X
'''
    expected = "-100\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_static_float(run_gb, run_vm):
    src = '''
CLASS Phys
    STATIC CONST GRAVITY AS FLOAT = 9.81
END CLASS

PRINT Phys.GRAVITY
'''
    expected = "9.81\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_static_bool(run_gb, run_vm):
    src = '''
CLASS Cfg
    STATIC CONST DEBUG AS BOOLEAN = TRUE
END CLASS

IF Cfg.DEBUG THEN PRINT "debug" ELSE PRINT "release"
'''
    expected = "debug\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_unknown_static_throws(run_gb, run_vm):
    from gamebasic.errors import GBRuntimeError
    src = '''
CLASS C
    STATIC CONST X AS INTEGER = 1
END CLASS

PRINT C.NONEXISTENT
'''
    with pytest.raises(GBRuntimeError):
        run_gb(src)
    with pytest.raises(GBRuntimeError):
        run_vm(src)


def _compile(src):
    """Kompiliert Quelltext zu Bytecode (der Pfad, den gbrt konsumiert)."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler
    return Compiler().compile(Parser(Lexer(src).tokenize()).parse())


def test_duplicate_static_rejected_at_compile():
    """Doppelte STATIC CONST werden beim Compile (fuer gbrt) abgelehnt."""
    from gamebasic.compiler import CompileError
    src = '''
CLASS C
    STATIC CONST X AS INTEGER = 1
    STATIC CONST X AS INTEGER = 2
END CLASS
'''
    with pytest.raises(CompileError):
        _compile(src)


def test_static_non_literal_rejected():
    """STATIC CONST mit Ausdruck statt Literal -> CompileError beim Compile."""
    from gamebasic.compiler import CompileError
    src = '''
CLASS C
    STATIC CONST X AS INTEGER = 1 + 1
END CLASS
'''
    with pytest.raises(CompileError):
        _compile(src)


def test_class_with_no_statics_unaffected(run_gb, run_vm):
    """Eine Klasse ohne Statics darf weiter normal verwendet werden -- der
    Klassen-Name ist nicht im globalen Scope reserviert."""
    src = '''
CLASS Foo
    DIM v AS INTEGER
END CLASS

DIM f AS Foo
f = NEW Foo()
f.v = 42
PRINT f.v
'''
    expected = "42\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
