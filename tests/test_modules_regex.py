"""Tests fuer das regex-Modul."""
import pytest


# --- Boolean-Tests --------------------------------------------------

def test_regex_match_full(run_gb, run_vm):
    src = '''
IMPORT "regex"
PRINT REGEX_MATCH("abc123", "[a-z]+[0-9]+")
PRINT REGEX_MATCH("abc123x", "[a-z]+[0-9]+")
'''
    expected = "TRUE\nFALSE\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_regex_test_searches_anywhere(run_gb, run_vm):
    src = '''
IMPORT "regex"
PRINT REGEX_TEST("Hallo Welt", "Welt")
PRINT REGEX_TEST("Hallo Welt", "X")
PRINT REGEX_TEST("hello123world", "[0-9]+")
'''
    expected = "TRUE\nFALSE\nTRUE\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Find -----------------------------------------------------------

def test_regex_find_first(run_gb, run_vm):
    src = '''
IMPORT "regex"
PRINT REGEX_FIND("foo123bar456", "[0-9]+")
PRINT REGEX_FIND("nothing here", "[0-9]+")
'''
    expected = "123\n\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_regex_find_all(run_gb, run_vm):
    src = '''
IMPORT "regex"
DIM nums AS ARRAY OF STRING
nums = REGEX_FIND_ALL("a1 b22 c333", "[0-9]+")
PRINT nums.length()
PRINT nums[0]
PRINT nums[1]
PRINT nums[2]
'''
    expected = "3\n1\n22\n333\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_regex_find_all_no_match_empty(run_gb, run_vm):
    src = '''
IMPORT "regex"
DIM r AS ARRAY OF STRING
r = REGEX_FIND_ALL("nothing", "[0-9]+")
PRINT r.length()
'''
    expected = "0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Replace --------------------------------------------------------

def test_regex_replace_all(run_gb, run_vm):
    src = '''
IMPORT "regex"
PRINT REGEX_REPLACE("Hallo Welt", "[aeiou]", "*")
PRINT REGEX_REPLACE("foo123bar456", "[0-9]+", "X")
'''
    expected = "H*ll* W*lt\nfooXbarX\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_regex_replace_once_only_first(run_gb, run_vm):
    src = '''
IMPORT "regex"
PRINT REGEX_REPLACE_ONCE("aaa", "a", "X")
'''
    expected = "Xaa\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_regex_replace_with_backref(run_gb, run_vm):
    """Capture-Gruppen via `\\1` im Replacement."""
    src = r'''
IMPORT "regex"
PRINT REGEX_REPLACE("hello world", "(\w+) (\w+)", "\2 \1")
'''
    expected = "world hello\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Split ----------------------------------------------------------

def test_regex_split_multi_separator(run_gb, run_vm):
    """SPLIT$ kann nur Single-Char-Sep -- regex_split kann jedes Pattern."""
    src = r'''
IMPORT "regex"
DIM parts AS ARRAY OF STRING
parts = REGEX_SPLIT("a,b;c, d", "[,;]\s*")
PRINT parts.length()
PRINT parts[0]
PRINT parts[1]
PRINT parts[2]
PRINT parts[3]
'''
    expected = "4\na\nb\nc\nd\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_regex_split_whitespace(run_gb, run_vm):
    src = r'''
IMPORT "regex"
DIM words AS ARRAY OF STRING
words = REGEX_SPLIT("hello   world  foo", "\s+")
PRINT words.length()
PRINT words[0]
PRINT words[1]
PRINT words[2]
'''
    expected = "3\nhello\nworld\nfoo\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Fehlerfaelle ---------------------------------------------------

def test_invalid_regex_throws_gb_error(run_gb, run_vm):
    """Ungueltige Regex (z.B. unbalanced paren) wirft DHRuntimeError,
    nicht Python's re.error."""
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError):
        run_gb('IMPORT "regex"\nPRINT REGEX_TEST("abc", "(unbalanced")')


def test_regex_pattern_cache(run_gb, run_vm):
    """Wiederholte Aufrufe mit gleichem Pattern muessen funktionieren --
    Test, dass der Cache nicht stoert."""
    src = '''
IMPORT "regex"
DIM i AS INTEGER
DIM count AS INTEGER
count = 0
FOR i = 1 TO 5
    IF REGEX_TEST("hello123", "[0-9]+") THEN count = count + 1
NEXT
PRINT count
'''
    expected = "5\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
