"""Tests fuer Variadic-Functions: `SUB log(label AS STRING, ...args)`.

Variadic-Args werden als TUPLE der restlichen Positional-Args gesammelt.
Muss letzter Parameter sein. Named-Args fuer Variadic-Slot werden
abgelehnt.
"""
import pytest


def test_variadic_basic(run_gb, run_vm):
    src = '''
SUB show(...args)
    PRINT args.length()
END SUB

show()
show(1)
show(1, 2, 3)
show("a", "b", "c", "d")
'''
    expected = "0\n1\n3\n4\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_variadic_with_required(run_gb, run_vm):
    """Pflicht-Param + Variadic: erste Args bedienen Pflicht, Rest -> Tuple."""
    src = '''
SUB log(label AS STRING, ...args)
    PRINT label, "=>", args.length()
END SUB

log("a")
log("b", 1)
log("c", 1, 2, 3)
'''
    expected = "a => 0\nb => 1\nc => 3\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_variadic_index_access(run_gb, run_vm):
    """`args[0]`, `args[1]` etc. funktioniert auf dem Tupel."""
    src = '''
FUNCTION sum_all(...nums) AS INTEGER
    DIM total AS INTEGER
    DIM i AS INTEGER
    total = 0
    FOR i = 0 TO nums.length() - 1
        total = total + nums[i]
    NEXT
    RETURN total
END FUNCTION

PRINT sum_all()
PRINT sum_all(1, 2, 3, 4, 5)
PRINT sum_all(100)
'''
    expected = "0\n15\n100\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_variadic_too_few_args_throws(run_gb, run_vm):
    """Wenn Pflicht-Args vor Variadic nicht erfuellt sind."""
    from gamebasic.errors import GBRuntimeError
    src = '''
SUB log(label AS STRING, ...args)
    PRINT label
END SUB

log()
'''
    with pytest.raises(GBRuntimeError):
        run_gb(src)
    with pytest.raises(GBRuntimeError):
        run_vm(src)


def test_variadic_must_be_last(run_gb, run_vm):
    """`SUB foo(...args, x)` -> ParseError."""
    from gamebasic.errors import ParseError
    src = '''
SUB foo(...args, x AS INTEGER)
END SUB
'''
    with pytest.raises(ParseError):
        run_gb(src)
    with pytest.raises(ParseError):
        run_vm(src)


def test_variadic_two_variadic_rejected(run_gb, run_vm):
    """Zwei Variadic-Params -> ParseError."""
    from gamebasic.errors import ParseError
    src = '''
SUB foo(...a, ...b)
END SUB
'''
    with pytest.raises(ParseError):
        run_gb(src)
    with pytest.raises(ParseError):
        run_vm(src)


def test_variadic_passed_to_helper(run_gb, run_vm):
    """Die gesammelten Args koennen als TUPLE an andere Funktionen
    weitergereicht werden."""
    src = '''
FUNCTION first_or_default(t AS TUPLE, dflt AS INTEGER) AS INTEGER
    IF t.length() = 0 THEN RETURN dflt
    RETURN t[0]
END FUNCTION

FUNCTION head(...nums) AS INTEGER
    RETURN first_or_default(nums, -1)
END FUNCTION

PRINT head()
PRINT head(42, 99)
'''
    expected = "-1\n42\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_variadic_mixed_types(run_gb, run_vm):
    """TUPLE kann Werte verschiedener Typen halten."""
    src = '''
SUB describe(...items)
    DIM i AS INTEGER
    FOR i = 0 TO items.length() - 1
        PRINT items[i]
    NEXT
END SUB

describe(1, "zwei", 3.5, TRUE)
'''
    expected = "1\nzwei\n3.5\nTRUE\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
