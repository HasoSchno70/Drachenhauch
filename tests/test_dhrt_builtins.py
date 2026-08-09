"""Golden-Tests fuer **dhrt-only**-Builtins.

Seit 2026-06-05 werden neue Builtins nur noch in der nativen Runtime `dhrt`
(Rust) implementiert -- der Python-Tree-Walker (`interpreter.py`) wird nicht
mehr erweitert. Diese Builtins lassen sich daher nicht per TW==dhrt-Paritaet
testen; stattdessen laeuft das Programm durch dhrt und der Output wird gegen
erwartete Literale geprueft (Golden-Test).

Die Quelle wird ueber dhrts EIGENEN Rust-Compiler (`dhrt --runsrc`) kompiliert
und ausgefuehrt -- NICHT ueber die Python-Toolchain. Wichtig: die Python-
`compiler.py` erkennt dhrt-only-Builtins nicht (sie stehen nicht mehr im Python-
`BUILTINS`-Registry) und wuerde sie als User-Calls fehlkompilieren. dhrts Rust-
Compiler emittiert CALL_BUILTIN fuer jeden unbekannten Call -> der dhrt-VM-Pfad
loest sie nativ auf.
"""
import os
import subprocess
import tempfile
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_dhrt():
    base = _ROOT / "rust" / "drachenhauch_runtime" / "target"
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(
    _DHRT is None,
    reason="native Runtime 'dhrt' nicht gebaut (rust/build_runtime.py)")


def _dhrt(source: str) -> str:
    """Schreibt `source` in eine temp .dh und fuehrt sie via `dhrt --runsrc`
    (Rust-Frontend: preprocess->lex->parse->compile->VM); stdout mit LF."""
    fd, tmp = tempfile.mkstemp(suffix=".dh")
    os.close(fd)
    Path(tmp).write_text(source, encoding="utf-8")
    try:
        res = subprocess.run([str(_DHRT), "--runsrc", tmp],
                             capture_output=True, text=True, timeout=60)
        return res.stdout.replace("\r\n", "\n")
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass


def _run(source: str) -> str:
    return _dhrt(source).strip()


# --- WP1: Array-Aggregate ---------------------------------------------------

def test_array_sum_avg_min_max_int():
    src = '''DIM a[3] AS INTEGER
a[0]=5 : a[1]=9 : a[2]=1
PRINT ARRAY_SUM(a), ARRAY_AVG(a), ARRAY_MIN(a), ARRAY_MAX(a)'''
    assert _run(src) == "15 5.0 1 9"


def test_array_aggregate_float():
    src = '''DIM f[3] AS FLOAT
f[0]=1.5 : f[1]=2.5 : f[2]=4.0
PRINT ARRAY_SUM(f), ARRAY_MIN(f), ARRAY_MAX(f)
PRINT ARRAY_AVG(f)'''
    assert _run(src) == "8.0 1.5 4.0\n2.6666666666666665"


def test_array_fill_coerces_int_to_float():
    src = '''DIM b[3] AS INTEGER
ARRAY_FILL(b, 7)
PRINT b[0], b[1], b[2]
DIM f[2] AS FLOAT
ARRAY_FILL(f, 9)
PRINT f[0], f[1]'''
    assert _run(src) == "7 7 7\n9.0 9.0"


def test_array_copy_is_independent():
    src = '''DIM a[3] AS INTEGER
a[0]=5 : a[1]=9 : a[2]=1
DIM c AS ARRAY OF INTEGER
c = ARRAY_COPY(a)
c[0] = 100
PRINT a[0], c[0]'''
    assert _run(src) == "5 100"


def test_array_avg_empty_errors():
    # 0-grosses Array -> ARRAY_AVG wirft; "before" kommt noch, "after" nicht.
    src = '''PRINT "before"
DIM a[0] AS INTEGER
PRINT ARRAY_AVG(a)
PRINT "after"'''
    assert _run(src) == "before"


# --- WP1: Dynamische Arrays -------------------------------------------------

def test_array_push_pop():
    src = '''DIM a[3] AS INTEGER
a[0]=10 : a[1]=20 : a[2]=30
PRINT ARRAY_PUSH(a, 40), a[3], LEN(a)
PRINT ARRAY_POP(a), LEN(a)'''
    assert _run(src) == "4 40 4\n40 3"


def test_array_insert_remove_at():
    src = '''DIM a[3] AS INTEGER
a[0]=10 : a[1]=20 : a[2]=30
PRINT ARRAY_INSERT(a, 1, 99)
PRINT a[0], a[1], a[2], a[3]
PRINT ARRAY_REMOVE_AT(a, 1)
PRINT a[0], a[1], a[2], LEN(a)'''
    assert _run(src) == "4\n10 99 20 30\n99\n10 20 30 3"


def test_redim_grow_and_shrink():
    src = '''DIM a[3] AS INTEGER
a[0]=10 : a[1]=20 : a[2]=30
REDIM(a, 5)
PRINT LEN(a), a[2], a[3], a[4]
REDIM(a, 2)
PRINT LEN(a), a[0], a[1]'''
    # grow: Bestand bleibt, neue Slots = 0; shrink: abschneiden, Rest bleibt.
    assert _run(src) == "5 30 0 0\n2 10 20"


def test_array_push_string_and_index_in_new_range():
    src = '''DIM s[1] AS STRING
s[0] = "hi"
ARRAY_PUSH(s, "there")
PRINT s[0], s[1], LEN(s)'''
    assert _run(src) == "hi there 2"


def test_array_pop_empty_errors():
    src = '''PRINT "before"
DIM a[1] AS INTEGER
PRINT ARRAY_POP(a)
PRINT ARRAY_POP(a)
PRINT "after"'''
    # zweites POP auf leerem Array wirft -> "after" fehlt.
    assert _run(src) == "before\n0"


# --- WP3: String-Erweiterungen ----------------------------------------------

def test_string_trim_reverse():
    src = '''PRINT "[" + LTRIM$("  hi  ") + "]"
PRINT "[" + RTRIM$("  hi  ") + "]"
PRINT REVERSE$("hello")'''
    assert _run(src) == "[hi  ]\n[  hi]\nolleh"


def test_string_predicates():
    src = '''PRINT STARTSWITH("hello world", "hello"), STARTSWITH("hello", "x")
PRINT ENDSWITH("hello world", "world"), ENDSWITH("hello", "x")
PRINT CONTAINS("hello world", "o w"), CONTAINS("abc", "z")'''
    assert _run(src) == "TRUE FALSE\nTRUE FALSE\nTRUE FALSE"


def test_string_bin_oct():
    src = 'PRINT BIN$(10), BIN$(-10), OCT$(64), OCT$(-64)'
    assert _run(src) == "1010 -1010 100 -100"


def test_string_isnumeric_tryval():
    src = '''PRINT ISNUMERIC("42"), ISNUMERIC("3.14"), ISNUMERIC("1e5"), ISNUMERIC("abc"), ISNUMERIC("")
PRINT TRYVAL("42", -1), TRYVAL("3.5", -1), TRYVAL("oops", -1), TRYVAL("  7  ", 0)'''
    assert _run(src) == "TRUE TRUE TRUE FALSE FALSE\n42 3.5 -1 7"


# --- Compiler-Regression: funktions-lokale Slots (kein globaler Name-Leak) ---

def test_function_local_scope_no_global_leak():
    """Frueher kompilierte dhrts Rust-Compiler skalare DIMs und FOR-Variablen in
    Funktionen als GLOBALE Namen statt funktions-lokale Slots. Dann korrumpierte
    ein gleichnamiges Local einer aufgerufenen Funktion die FOR-Schleifenvariable
    -> Endlosschleife (z.B. die Cybermatic-3D-Demo hing). Hier: `fb` hat ein
    lokales `k`, `scn` eine FOR-`k`-Schleife, die `fb` aufruft. Muss terminieren."""
    src = '''FUNCTION fb(i AS INTEGER) AS INTEGER
DIM k AS INTEGER
k = i * 10
RETURN k
END FUNCTION
SUB scn()
DIM seg AS INTEGER
FOR seg = 0 TO 2
DIM k AS INTEGER
FOR k = 0 TO 2
DIM c AS INTEGER
c = fb(seg)
NEXT
PRINT "seg=" + STR$(seg) + " k=" + STR$(k)
NEXT
END SUB
scn()
PRINT "done"'''
    assert _run(src) == "seg=0 k=3\nseg=1 k=3\nseg=2 k=3\ndone"


def test_function_local_shadows_global_loop_var():
    """Ein funktions-lokales `i` (DIM/FOR) muss einen gleichnamigen Top-Level-
    Global SHADOWEN. Frueher schrieb das Local faelschlich das GLOBAL -> die
    aufrufende FOR-Schleife brach ab/hing (so hing 64_showcase.dh / Variadic-
    Logger in einer FOR i-Schleife)."""
    src = '''SUB inner()
DIM i AS INTEGER
FOR i = 0 TO 1
NEXT
END SUB
DIM i AS INTEGER
DIM tot AS INTEGER
tot = 0
FOR i = 0 TO 2
inner()
tot = tot + i
NEXT
PRINT tot'''
    assert _run(src) == "3"


def test_function_local_same_name_as_other_function():
    """Zwei Funktionen mit gleichnamigem Local duerfen sich nicht beeinflussen."""
    src = '''FUNCTION a() AS INTEGER
DIM x AS INTEGER
x = 111
RETURN x
END FUNCTION
FUNCTION b() AS INTEGER
DIM x AS INTEGER
x = a()
x = x + 1
RETURN x
END FUNCTION
PRINT b()'''
    assert _run(src) == "112"


def test_function_local_array_isolated():
    """Lokale Arrays gleichen Namens in zwei Funktionen duerfen sich nicht
    teilen (frueher: DECLARE_ARRAY_NAME global statt DECLARE_ARRAY_LOCAL)."""
    src = '''FUNCTION makeit() AS INTEGER
DIM tmp[3] AS INTEGER
tmp[0] = 99
RETURN tmp[0]
END FUNCTION
SUB use()
DIM tmp[3] AS INTEGER
tmp[0] = 1 : tmp[1] = 2 : tmp[2] = 3
DIM x AS INTEGER
x = makeit()
PRINT tmp[0], tmp[1], tmp[2]
END SUB
use()
PRINT "done"'''
    assert _run(src) == "1 2 3\ndone"


def test_function_local_struct_isolated():
    """Lokale STRUCT-Instanzen gleichen Namens in zwei Funktionen isoliert."""
    src = '''STRUCT Pt
DIM x AS INTEGER
DIM y AS INTEGER
END STRUCT
FUNCTION other() AS INTEGER
DIM p AS Pt
p.x = 99
RETURN p.x
END FUNCTION
SUB use()
DIM p AS Pt
p.x = 5 : p.y = 7
DIM z AS INTEGER
z = other()
PRINT p.x, p.y
END SUB
use()
PRINT "done"'''
    assert _run(src) == "5 7\ndone"


# --- WP3: Datei / Verzeichnis -----------------------------------------------

def test_pathjoin():
    src = '''PRINT PATHJOIN("assets", "img", "x.png")
PRINT PATHJOIN("a/", "b")
PRINT PATHJOIN("", "sub", "data.txt")'''
    assert _run(src) == "assets/img/x.png\na/b\nsub/data.txt"


def test_file_and_dir_ops(tmp_path):
    base = str(tmp_path).replace("\\", "/")
    src = f'''DIM base AS STRING
base = "{base}"
MKDIR(PATHJOIN(base, "sub"))
PRINT DIREXISTS(PATHJOIN(base, "sub"))
WRITEALL(PATHJOIN(base, "sub/data.txt"), "line1" + CHR$(10) + "line2" + CHR$(10) + "line3")
PRINT FILESIZE(PATHJOIN(base, "sub/data.txt"))
DIM lines AS ARRAY OF STRING
lines = READLINES(PATHJOIN(base, "sub/data.txt"))
PRINT LEN(lines), lines[0], lines[2]
WRITEALL(PATHJOIN(base, "sub/b.txt"), "x")
DIM entries AS ARRAY OF STRING
entries = DIRLIST(PATHJOIN(base, "sub"))
PRINT LEN(entries), entries[0], entries[1]
RENAME(PATHJOIN(base, "sub/b.txt"), PATHJOIN(base, "sub/c.txt"))
PRINT FILEEXISTS(PATHJOIN(base, "sub/b.txt")), FILEEXISTS(PATHJOIN(base, "sub/c.txt"))
DELETEFILE(PATHJOIN(base, "sub/c.txt"))
PRINT FILEEXISTS(PATHJOIN(base, "sub/c.txt"))'''
    assert _run(src) == (
        "TRUE\n"
        "17\n"
        "3 line1 line3\n"
        "2 b.txt data.txt\n"
        "FALSE TRUE\n"
        "FALSE"
    )


# --- WP4: BASIC-Aliase + Container-Methode ----------------------------------

def test_aliases_sgn_sqrt():
    src = '''PRINT SGN(-5), SGN(0), SGN(3)
PRINT SQRT(16.0), SQRT(2.0)'''
    assert _run(src) == "-1 0 1\n4.0 1.4142135623730951"


def test_array_join_method():
    src = '''DIM names[3] AS STRING
names[0]="Anna" : names[1]="Bert" : names[2]="Cilly"
PRINT names.join(", ")'''
    assert _run(src) == "Anna, Bert, Cilly"


# --- WP1: SORT mit Descending-Flag + FUNCREF-Comparator ---------------------

def test_sort_ascending_and_descending_flag():
    src = '''DIM x[5] AS INTEGER
x[0]=3 : x[1]=1 : x[2]=9 : x[3]=2 : x[4]=7
SORT(x)
PRINT x[0], x[1], x[2], x[3], x[4]
SORT(x, TRUE)
PRINT x[0], x[1], x[2], x[3], x[4]'''
    assert _run(src) == "1 2 3 7 9\n9 7 3 2 1"


def test_sort_string_descending():
    src = '''DIM s[3] AS STRING
s[0]="banana" : s[1]="apple" : s[2]="cherry"
SORT(s, TRUE)
PRINT s[0], s[1], s[2]'''
    assert _run(src) == "cherry banana apple"


def test_sort_with_comparator_funcref():
    # Comparator b-a -> absteigend; sowohl bare Name als auch via FUNCREF-Variable.
    src = '''FUNCTION desc(a AS INTEGER, b AS INTEGER) AS INTEGER
RETURN b - a
END FUNCTION
DIM x[5] AS INTEGER
x[0]=3 : x[1]=1 : x[2]=9 : x[3]=2 : x[4]=7
SORT(x, desc)
PRINT x[0], x[1], x[2], x[3], x[4]
DIM f AS FUNCREF
f = desc
DIM y[3] AS INTEGER
y[0]=5 : y[1]=8 : y[2]=2
SORT(y, f)
PRINT y[0], y[1], y[2]'''
    assert _run(src) == "9 7 3 2 1\n8 5 2"


def test_sort_comparator_stable_by_distance():
    # Stabilitaet: gleiche Schluessel behalten Eingabereihenfolge.
    src = '''FUNCTION bydist(a AS INTEGER, b AS INTEGER) AS INTEGER
RETURN ABS(a - 5) - ABS(b - 5)
END FUNCTION
DIM y[5] AS INTEGER
y[0]=3 : y[1]=1 : y[2]=9 : y[3]=2 : y[4]=7
SORT(y, bydist)
PRINT y[0], y[1], y[2], y[3], y[4]'''
    assert _run(src) == "3 7 2 1 9"


# --- G1: FLT (Int->Float-Cast) nativ in dhrt -------------------------------

def test_flt_int_to_float():
    """FLT lief frueher nur im Tree-Walker; jetzt auch nativ (Stolperstein G1)."""
    assert _run("PRINT FLT(7)") == "7.0"
    assert _run("PRINT FLT(7) / 2") == "3.5"
    assert _run("PRINT FLT(2.5)") == "2.5"


# --- NUMFMT$: Big-Number-Formatierung (Idle-/Incremental-Game-Stil) --------

def test_numfmt_below_1000_no_suffix():
    assert _run("PRINT NUMFMT$(42)") == "42.00"
    assert _run("PRINT NUMFMT$(999)") == "999.00"
    assert _run("PRINT NUMFMT$(0)") == "0.00"


def test_numfmt_suffix_tiers():
    assert _run("PRINT NUMFMT$(1000)") == "1.00K"
    assert _run("PRINT NUMFMT$(1234)") == "1.23K"
    assert _run("PRINT NUMFMT$(1234567)") == "1.23M"
    assert _run("PRINT NUMFMT$(1500000000)") == "1.50B"
    assert _run("PRINT NUMFMT$(1000000000000000000.0)") == "1.00Qi"  # 10^18


def test_numfmt_rounding_overflow_bumps_tier():
    # 999999 = 999.999K -- bei 2 Nachkommastellen rundet das auf 1000.00K,
    # was ueber die Grenze rutscht; muss auf "1.00M" hochspringen statt
    # das falsche "1000.00K" auszugeben.
    assert _run("PRINT NUMFMT$(999999)") == "1.00M"
    assert _run("PRINT NUMFMT$(999995)") == "1.00M"


def test_numfmt_custom_decimals():
    assert _run("PRINT NUMFMT$(1234, 0)") == "1K"
    assert _run("PRINT NUMFMT$(1234567, 1)") == "1.2M"


def test_numfmt_negative():
    assert _run("PRINT NUMFMT$(-2500)") == "-2.50K"


def test_numfmt_scientific_fallback_beyond_decillion():
    # Jenseits von Dc (10^33) gibt es keinen benannten Suffix mehr.
    assert _run("PRINT NUMFMT$(POW(10.0, 40))") == "1.00e40"
