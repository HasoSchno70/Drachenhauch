"""A2: NIL ist jetzt ein echtes Literal.

Frueher schlug jede Erwaehnung von NIL fehl („Variable 'nil' nicht
deklariert"), obwohl Doku/Beispiele `<> NIL` und `IS_NIL(NIL)` nutzten.
Jetzt ist NIL ein Keyword-Literal (Wert = der leere/uninitialisierte Wert).
"""
import pytest


# --- Laufzeit-Semantik (dhrt) -------------------------------------------

def test_is_nil_of_nil_literal(run_gb):
    assert run_gb("PRINT IS_NIL(NIL)\n") == "TRUE\n"


def test_uninitialized_object_equals_nil(run_gb):
    src = ("CLASS Foo\n  DIM x AS INTEGER\nEND CLASS\n"
           "DIM o AS Foo\n"
           "PRINT o = NIL\n"
           "PRINT o <> NIL\n")
    assert run_gb(src) == "TRUE\nFALSE\n"


def test_object_after_new_is_not_nil(run_gb):
    src = ("CLASS Foo\n  DIM x AS INTEGER\nEND CLASS\n"
           "DIM o AS Foo\n"
           "o = NEW Foo()\n"
           "PRINT o = NIL\n"
           "PRINT IS_NIL(o)\n")
    assert run_gb(src) == "FALSE\nFALSE\n"


def test_assign_nil_clears_object(run_gb):
    src = ("CLASS Foo\n  DIM x AS INTEGER\nEND CLASS\n"
           "DIM o AS Foo\n"
           "o = NEW Foo()\n"
           "o = NIL\n"
           "PRINT IS_NIL(o)\n")
    assert run_gb(src) == "TRUE\n"


def test_db_bind_nil_stores_null(run_gb):
    """Das in docs/module-db.md versprochene NIL->NULL-Binding funktioniert jetzt."""
    src = ('IMPORT "db"\n'
           'DIM con AS DB_CONN\n'
           'con = DB_OPEN(":memory:")\n'
           'DB_EXEC(con, "CREATE TABLE u (name TEXT, email TEXT)")\n'
           'DB_EXEC(con, "INSERT INTO u VALUES (?, ?)", "Anna", NIL)\n'
           'DIM r AS DB_RESULT\n'
           'r = DB_QUERY(con, "SELECT name, email FROM u")\n'
           'IF DB_NEXT(r) THEN PRINT DB_IS_NULL(r, 1)\n'
           'DB_CLOSE_RESULT(r)\n'
           'DB_CLOSE(con)\n')
    assert run_gb(src) == "TRUE\n"
