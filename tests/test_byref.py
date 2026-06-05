"""Tests fuer BYREF-Parameter (Multi-Return via Copy-In-Copy-Out)."""
import pytest


# --- Tree-Walker-Pfad: BYREF funktioniert vollstaendig ---------------

@pytest.mark.xfail(reason="gbrt unterstuetzt BYREF noch nicht (VM-Pfad) -- Phase-8-Blocker", strict=False)
def test_byref_swap(run_gb):
    out = run_gb('''
SUB swap(BYREF a AS INTEGER, BYREF b AS INTEGER)
    DIM tmp AS INTEGER
    tmp = a
    a = b
    b = tmp
END SUB

DIM x AS INTEGER
DIM y AS INTEGER
x = 1
y = 2
swap(x, y)
PRINT x
PRINT y
''')
    assert out == "2\n1\n"


@pytest.mark.xfail(reason="gbrt unterstuetzt BYREF noch nicht (VM-Pfad) -- Phase-8-Blocker", strict=False)
def test_byref_with_string(run_gb):
    out = run_gb('''
SUB make_upper(BYREF s AS STRING)
    s = UPPER$(s)
END SUB

DIM name AS STRING
name = "anna"
make_upper(name)
PRINT name
''')
    assert out == "ANNA\n"


@pytest.mark.xfail(reason="gbrt unterstuetzt BYREF noch nicht (VM-Pfad) -- Phase-8-Blocker", strict=False)
def test_byref_function_returns_plus_modifies(run_gb):
    """FUNCTION mit BYREF: regulaerer return PLUS Side-Effect.

    `rem` waere eigentlich praktisch als Param-Name, ist aber GB-Keyword
    (REM = Kommentar) - wir nehmen `mod_out`.
    """
    out = run_gb('''
FUNCTION divmod(a AS INTEGER, b AS INTEGER, BYREF mod_out AS INTEGER) AS INTEGER
    mod_out = a MOD b
    RETURN a \\ b
END FUNCTION

DIM r AS INTEGER
DIM q AS INTEGER
q = divmod(17, 5, r)
PRINT q
PRINT r
''')
    assert out == "3\n2\n"


@pytest.mark.xfail(reason="gbrt unterstuetzt BYREF noch nicht (VM-Pfad) -- Phase-8-Blocker", strict=False)
def test_byref_mixed_with_regular_params(run_gb):
    out = run_gb('''
SUB add_to(amount AS INTEGER, BYREF total AS INTEGER)
    total = total + amount
END SUB

DIM sum AS INTEGER
sum = 0
add_to(5, sum)
add_to(10, sum)
add_to(3, sum)
PRINT sum
''')
    assert out == "18\n"


@pytest.mark.xfail(reason="gbrt unterstuetzt BYREF noch nicht (VM-Pfad) -- Phase-8-Blocker", strict=False)
def test_byref_with_array_index(run_gb):
    """BYREF kann auch auf einen Array-Slot zeigen."""
    out = run_gb('''
SUB increment(BYREF v AS INTEGER)
    v = v + 1
END SUB

DIM arr[3] AS INTEGER
arr[0] = 10
arr[1] = 20
arr[2] = 30
increment(arr[1])
PRINT arr[0]
PRINT arr[1]
PRINT arr[2]
''')
    assert out == "10\n21\n30\n"


@pytest.mark.xfail(reason="gbrt unterstuetzt BYREF noch nicht (VM-Pfad) -- Phase-8-Blocker", strict=False)
def test_byref_with_class_field(run_gb):
    out = run_gb('''
CLASS Point
    DIM x AS INTEGER
    DIM y AS INTEGER
END CLASS

SUB inc(BYREF v AS INTEGER)
    v = v + 100
END SUB

DIM p AS Point
p = NEW Point
p.x = 1
p.y = 2
inc(p.x)
inc(p.y)
PRINT p.x
PRINT p.y
''')
    assert out == "101\n102\n"


def test_byref_literal_argument_raises(run_gb):
    """BYREF-Argument muss zuweisbar sein - Literal -> Fehler."""
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError) as exc:
        run_gb('''
SUB foo(BYREF x AS INTEGER)
    x = 5
END SUB
foo(42)
''')
    assert "BYREF" in str(exc.value)


def test_byref_expression_argument_raises(run_gb):
    """BYREF-Argument muss eine LValue sein - Expression nicht erlaubt."""
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError):
        run_gb('''
SUB foo(BYREF x AS INTEGER)
    x = 1
END SUB
DIM a AS INTEGER
DIM b AS INTEGER
a = 1
b = 2
foo(a + b)
''')


# --- Parser-Validierung ---------------------------------------------

def test_byref_with_default_is_parser_error():
    """BYREF und Default-Wert schliessen sich aus."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.errors import ParseError
    src = '''
SUB foo(BYREF x AS INTEGER = 5)
END SUB
'''
    with pytest.raises(ParseError):
        Parser(Lexer(src).tokenize()).parse()


# --- VM-Pfad: BYREF wirft CompileError -------------------------------

def test_byref_compile_error_on_vm_path():
    """Im VM-Pfad ist BYREF noch nicht unterstuetzt - klare Fehlermeldung."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler, CompileError
    src = '''
SUB swap(BYREF a AS INTEGER, BYREF b AS INTEGER)
    DIM tmp AS INTEGER
    tmp = a
    a = b
    b = tmp
END SUB
'''
    ast = Parser(Lexer(src).tokenize()).parse()
    with pytest.raises(CompileError) as exc:
        Compiler().compile(ast)
    msg = str(exc.value).lower()
    assert "byref" in msg
    assert "vm" in msg or "tree-walker" in msg


# --- Default-Parameter (Sanity-Tests, da der Cython-Bug gefixt wurde) -

def test_default_param_treewalker(run_gb):
    out = run_gb('''
FUNCTION greet(name AS STRING, prefix AS STRING = "Hi, ") AS STRING
    RETURN prefix + name + "!"
END FUNCTION
PRINT greet("Anna")
PRINT greet("Bob", "Hello, ")
''')
    assert out == "Hi, Anna!\nHello, Bob!\n"


def test_default_param_vm(run_vm):
    out = run_vm('''
FUNCTION greet(name AS STRING, prefix AS STRING = "Hi, ") AS STRING
    RETURN prefix + name + "!"
END FUNCTION
PRINT greet("Anna")
PRINT greet("Bob", "Hello, ")
''')
    assert out == "Hi, Anna!\nHello, Bob!\n"


def test_default_param_multiple(run_gb):
    out = run_gb('''
FUNCTION mult(a AS INTEGER, b AS INTEGER = 2, c AS INTEGER = 3) AS INTEGER
    RETURN a * b * c
END FUNCTION
PRINT mult(10)
PRINT mult(10, 5)
PRINT mult(10, 5, 4)
''')
    assert out == "60\n150\n200\n"
