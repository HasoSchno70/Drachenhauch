"""Golden-Tests fuer drift-anfaellige Kern-Semantik (gegen `gbrt`).

Stufe B: Es gibt nur noch EINE Runtime (gbrt) -- die `run_all`-Fixture ist ein
Alias auf `gbrt run`. Diese Tests prueften historisch die Aequivalenz von
Tree-Walker/Python-VM/Cython-VM; jetzt sind es schlicht gbrt-Golden-Tests fuer
die frueher drift-anfaelligen Stellen (Operatoren, Coerce, Typ-Inferenz, _fmt,
IN, Slicing). Jeder Test asserted die erwartete Ausgabe.
"""
import pytest


# --- Arithmetik (inkl. der frueher divergenten MOD/POW) -----------------

def test_arithmetic(run_all):
    assert run_all("PRINT 7 - 3") == "4\n"
    assert run_all("PRINT 7 / 2") == "3.5\n"
    assert run_all("PRINT 8 / 2") == "4\n"          # int/int ohne Rest -> int
    assert run_all("PRINT 7 MOD 3") == "1\n"
    assert run_all("PRINT 2 ^ 10") == "1024\n"
    assert run_all("PRINT 7 \\ 2") == "3\n"          # INTDIV
    assert run_all("PRINT -7 \\ 2") == "-3\n"        # Truncation gegen 0


def test_arithmetic_type_errors(run_all):
    # Bool ist keine Zahl -- muss in allen Pfaden fangbar werfen (K2/K4).
    for op in ("t - t", "t / 2", "t MOD 2", "t ^ 2", "t + t", "t * 2"):
        src = ('DIM t AS BOOLEAN\nt = TRUE\n'
               f'TRY\n  PRINT {op}\nCATCH e\n  PRINT "caught"\nEND TRY\n')
        assert run_all(src) == "caught\n", op


def test_bitwise(run_all):
    assert run_all("PRINT 12 BAND 10") == "8\n"
    assert run_all("PRINT 12 BOR 10") == "14\n"
    assert run_all("PRINT 12 BXOR 10") == "6\n"
    assert run_all("PRINT 1 SHL 4") == "16\n"
    assert run_all("PRINT 256 SHR 2") == "64\n"
    assert run_all("PRINT BNOT 0") == "-1\n"


def test_string_mult_and_concat(run_all):
    assert run_all('PRINT "ab" * 3') == "ababab\n"
    assert run_all('PRINT 3 * "ab"') == "ababab\n"
    assert run_all('PRINT "x" + STR$(5) + "y"') == "x5y\n"


# --- _fmt: PRINT-Darstellung aller Werttypen (dreifach dupliziert) ------

def test_fmt_scalars(run_all):
    assert run_all("PRINT 1.0") == "1.0\n"
    assert run_all("PRINT 1.5") == "1.5\n"
    assert run_all("PRINT TRUE") == "TRUE\n"
    assert run_all("PRINT FALSE") == "FALSE\n"


def test_fmt_tuple(run_all):
    assert run_all("PRINT (1, 2, 3)") == "(1, 2, 3)\n"


def test_fmt_array_and_map(run_all):
    arr = "DIM a[3] AS INTEGER\nPRINT a\n"
    assert run_all(arr) == "<ARRAY[3] OF INTEGER>\n"
    mp = "DIM m AS MAP OF INTEGER\nMAPPUT(m, \"k\", 1)\nPRINT m\n"
    assert run_all(mp) == "<MAP[1] OF INTEGER>\n"


def test_fmt_funcref(run_all):
    src = ("FUNCTION sq(x AS INTEGER) AS INTEGER\n  RETURN x * x\n"
           "END FUNCTION\nDIM f AS FUNCREF\nf = sq\nPRINT f\n")
    assert run_all(src) == "<FUNCREF sq>\n"


# --- KA: untypisiertes CONST leitet in allen Pfaden denselben Typ ab ----

def test_untyped_const_scalar(run_all):
    assert run_all("CONST C = 42\nPRINT C") == "42\n"
    assert run_all('CONST C = "hi"\nPRINT C') == "hi\n"
    assert run_all("CONST C = 3.5\nPRINT C") == "3.5\n"


def test_untyped_const_array(run_all):
    # Frueher: lief im Tree-Walker, warf in beiden VMs.
    src = "DIM a[3] AS INTEGER\na[1] = 9\nCONST C = a\nPRINT C[1]\n"
    assert run_all(src) == "9\n"


def test_untyped_const_funcref(run_all):
    # Frueher: lief in den VMs, warf im Tree-Walker.
    src = ("FUNCTION dbl(x AS INTEGER) AS INTEGER\n  RETURN x + x\n"
           "END FUNCTION\nCONST F = dbl\nPRINT F(21)\n")
    assert run_all(src) == "42\n"


# --- Coercion / Type-Mismatch (dreifaches _coerce) ----------------------

def test_coerce_int_float(run_all):
    assert run_all("DIM x AS FLOAT\nx = 3\nPRINT x") == "3.0\n"
    # FLOAT -> INTEGER ohne Verlust verboten -> fangbar in allen Pfaden
    src = ('DIM x AS INTEGER\nTRY\n  x = 3.5\nCATCH e\n'
           '  PRINT "caught"\nEND TRY\n')
    assert run_all(src) == "caught\n"


# --- IN-Operator (_eval_in dreifach) ------------------------------------

def test_in_operator(run_all):
    assert run_all('IF "lo" IN "Hello" THEN\n  PRINT 1\nEND IF') == "1\n"
    assert run_all("IF 5 IN (1, 5, 9) THEN\n  PRINT 1\nEND IF") == "1\n"
    assert run_all("DIM a[3] AS INTEGER\na[0]=7\nIF 7 IN a THEN\n  PRINT 1\nEND IF") == "1\n"


# --- Slicing (_apply_slice dreifach) ------------------------------------

def test_slicing(run_all):
    assert run_all('PRINT "Hello World"[6:11]') == "World\n"
    assert run_all('PRINT "Hello"[:3]') == "Hel\n"
    assert run_all('PRINT "Hello"[2:]') == "llo\n"


# --- Comprehensions -----------------------------------------------------

def test_comprehension(run_all):
    src = ("DIM nums AS TUPLE\nnums = (1, 2, 3, 4, 5, 6)\n"
           "DIM evens AS TUPLE\nevens = [n FOR n IN nums WHERE n MOD 2 = 0]\n"
           "PRINT evens\n")
    assert run_all(src) == "(2, 4, 6)\n"


# --- OOP: Klassen, Vererbung, Properties, Static, Self ------------------

def test_oop_inheritance_and_self(run_all):
    src = (
        "CLASS Animal\n"
        "  DIM name AS STRING\n"
        "  FUNCTION speak() AS STRING\n    RETURN \"...\"\n  END FUNCTION\n"
        "  SUB describe()\n    PRINT Self.name + \": \" + Self.speak()\n  END SUB\n"
        "END CLASS\n"
        "CLASS Dog EXTENDS Animal\n"
        "  FUNCTION speak() AS STRING\n    RETURN \"Wuff\"\n  END FUNCTION\n"
        "END CLASS\n"
        "DIM d AS Dog\nd = NEW Dog()\nd.name = \"Rex\"\nd.describe()\n"
    )
    assert run_all(src) == "Rex: Wuff\n"


def test_property_clamp(run_all):
    src = (
        "CLASS P\n  DIM _hp AS INTEGER\n"
        "  PROPERTY GET hp() AS INTEGER\n    RETURN Self._hp\n  END PROPERTY\n"
        "  PROPERTY SET hp(v AS INTEGER)\n"
        "    IF v > 100 THEN v = 100\n    Self._hp = v\n  END PROPERTY\n"
        "END CLASS\n"
        "DIM p AS P\np = NEW P()\np.hp = 250\nPRINT p.hp\n"
    )
    assert run_all(src) == "100\n"


def test_enum_and_static(run_all):
    assert run_all("ENUM St = MENU, PLAY, OVER\nPRINT St.PLAY") == "1\n"
    src = ("CLASS C\n  STATIC CONST MAX AS INTEGER = 7\nEND CLASS\n"
           "PRINT C.MAX\n")
    assert run_all(src) == "7\n"


# --- Tupel + Destructuring ----------------------------------------------

def test_tuple_destructuring(run_all):
    src = ("FUNCTION mm(a AS INTEGER, b AS INTEGER) AS TUPLE\n"
           "  IF a < b THEN RETURN (a, b)\n  RETURN (b, a)\nEND FUNCTION\n"
           "DIM lo AS INTEGER\nDIM hi AS INTEGER\n(lo, hi) = mm(9, 2)\n"
           "PRINT lo\nPRINT hi\n")
    assert run_all(src) == "2\n9\n"


# --- DATA / READ / RESTORE (K1: fehlte frueher in der Native-VM) --------

def test_data_read_restore(run_all):
    src = ("DATA 10, 20, 30\nDIM a AS INTEGER\nDIM b AS INTEGER\n"
           "READ a\nREAD b\nPRINT a + b\nRESTORE\nREAD a\nPRINT a\n")
    assert run_all(src) == "30\n10\n"


# --- f-Strings ----------------------------------------------------------

def test_fstring(run_all):
    src = ('DIM n AS STRING\nn = "Anna"\nDIM hp AS INTEGER\nhp = 75\n'
           'PRINT f"{n} hat {hp} HP"\n')
    assert run_all(src) == "Anna hat 75 HP\n"


# --- f-String-Format-Specs (L1) -----------------------------------------

def test_fstring_format_specs(run_all):
    src = ("DIM fps AS FLOAT\nfps = 59.731\nDIM sc AS INTEGER\nsc = 42\n"
           'PRINT f"{fps:.1f}|{sc:05d}"\n')
    assert run_all(src) == "59.7|00042\n"


def test_fstring_slice_not_spec(run_all):
    # `:` in einem Slice darf NICHT als Format-Spec interpretiert werden.
    src = 'DIM s AS STRING\ns = "Hello"\nPRINT f"{s[0:3]}"\n'
    assert run_all(src) == "Hel\n"


# --- Array-/Map-Helfer (L2) ---------------------------------------------

def test_array_sort_reverse_indexof(run_all):
    src = ("DIM a[5] AS INTEGER\n"
           "a[0]=5 : a[1]=2 : a[2]=8 : a[3]=1 : a[4]=2\n"
           "SORT(a)\nPRINT a[0]\nPRINT a[4]\n"
           "PRINT ARRAY_INDEXOF(a, 8)\n"
           "REVERSE(a)\nPRINT a[0]\n")
    assert run_all(src) == "1\n8\n4\n8\n"


def test_array_method_syntax(run_all):
    src = ("DIM a[3] AS INTEGER\na[0]=3 : a[1]=1 : a[2]=2\n"
           "a.sort()\nPRINT a[0]\nPRINT a.indexof(3)\n")
    assert run_all(src) == "1\n2\n"


def test_map_values_items(run_all):
    src = ("DIM m AS MAP OF INTEGER\nMAPPUT(m, \"x\", 10)\nMAPPUT(m, \"y\", 20)\n"
           "DIM vs AS ARRAY OF INTEGER\nvs = MAPVALUES(m)\n"
           "PRINT vs[0] + vs[1]\n"
           "DIM it AS ARRAY OF TUPLE\nit = m.items()\n"
           "DIM k AS STRING\nDIM v AS INTEGER\n(k, v) = it[0]\nPRINT k\nPRINT v\n")
    assert run_all(src) == "30\nx\n10\n"


# --- Array-Index-Fast-Path (P4): Edge-Cases muessen identisch werfen -----

def test_array_index_fastpath_edges(run_all):
    assert run_all("DIM a[3] AS INTEGER\na[2] = 9\nPRINT a[2]") == "9\n"
    # Out-of-bounds: Fast-Path faellt in den generischen Pfad -> fangbar
    oob = ('DIM a[3] AS INTEGER\nTRY\n  PRINT a[5]\nCATCH e\n'
           '  PRINT "oob"\nEND TRY\n')
    assert run_all(oob) == "oob\n"
    neg = ('DIM a[3] AS INTEGER\nTRY\n  PRINT a[0 - 1]\nCATCH e\n'
           '  PRINT "neg"\nEND TRY\n')
    assert run_all(neg) == "neg\n"
    # 2D-Array mit nur einem Index -> Dim-Mismatch (kein Fast-Path)
    dim2 = ('DIM g[3, 3] AS INTEGER\nTRY\n  PRINT g[1]\nCATCH e\n'
            '  PRINT "dim"\nEND TRY\n')
    assert run_all(dim2) == "dim\n"


# --- IIF / Ternary (lazy) -----------------------------------------------

def test_iif_basic(run_all):
    assert run_all('PRINT IIF(5 > 3, "a", "b")') == "a\n"
    assert run_all('PRINT IIF(1 > 9, 100, 7)') == "7\n"


def test_iif_lazy(run_all):
    # Toter Zweig (Division durch 0) darf NICHT ausgewertet werden.
    src = ("DIM x AS INTEGER\nx = 0\nPRINT IIF(x <> 0, 100 \\ x, -1)\n")
    assert run_all(src) == "-1\n"


def test_iif_as_variable(run_all):
    # `iif` ohne '(' bleibt eine Variable (kein Keyword).
    assert run_all("DIM iif AS INTEGER\niif = 5\nPRINT iif") == "5\n"


# --- FOR EACH -----------------------------------------------------------

def test_foreach_array(run_all):
    src = ("DIM a[4] AS INTEGER\na[0]=10 : a[1]=20 : a[2]=30 : a[3]=40\n"
           "DIM s AS INTEGER\ns = 0\nFOR EACH v IN a\n  s = s + v\nNEXT\nPRINT s\n")
    assert run_all(src) == "100\n"


def test_foreach_break_continue(run_all):
    src = ("DIM a[4] AS INTEGER\na[0]=10 : a[1]=20 : a[2]=30 : a[3]=40\n"
           "DIM c AS INTEGER\nc = 0\nFOR EACH v IN a\n"
           "  IF v = 20 THEN CONTINUE\n  IF v = 40 THEN BREAK\n"
           "  c = c + 1\nNEXT\nPRINT c\n")
    assert run_all(src) == "2\n"


def test_foreach_string_tuple_map(run_all):
    assert run_all('FOR EACH ch IN "ab"\n  PRINT ch\nNEXT') == "a\nb\n"
    assert run_all("FOR EACH x IN (7, 8)\n  PRINT x\nNEXT") == "7\n8\n"
    src = ('DIM m AS MAP OF INTEGER\nMAPPUT(m, "k1", 1)\nMAPPUT(m, "k2", 2)\n'
           "FOR EACH k IN m\n  PRINT k\nNEXT\n")
    assert run_all(src) == "k1\nk2\n"


def test_foreach_var_named_each_still_regular(run_all):
    # `FOR each = ...` (Variable namens "each") bleibt ein normaler FOR.
    src = ("DIM s AS INTEGER\ns = 0\nFOR each = 1 TO 3\n  s = s + each\nNEXT\nPRINT s\n")
    assert run_all(src) == "6\n"


# --- SELECT CASE mit Guards ---------------------------------------------

def test_select_guards(run_all):
    src = ("DIM hp AS INTEGER\nhp = 20\nDIM trank AS BOOLEAN\ntrank = TRUE\n"
           "SELECT CASE hp\n"
           "  CASE IS <= 0\n    PRINT \"tot\"\n"
           "  CASE IS <= 30 WHERE trank\n    PRINT \"heilen\"\n"
           "  CASE IS <= 30\n    PRINT \"fliehen\"\n"
           "  CASE ELSE\n    PRINT \"ok\"\n"
           "END SELECT\n")
    assert run_all(src) == "heilen\n"
