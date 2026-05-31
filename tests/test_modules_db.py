"""Tests fuer das db-Modul (SQLite, In-Memory)."""
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_db():
    assert load_module("db")


@pytest.fixture
def conn(call_builtin):
    """Frische In-Memory-DB mit kleinem Schema fuer jeden Test."""
    c = call_builtin("db_open", [":memory:"])
    call_builtin("db_exec", [c,
        "CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT, score INTEGER, dabei INTEGER)"
    ])
    yield c
    call_builtin("db_close", [c])


def test_open_close(call_builtin):
    c = call_builtin("db_open", [":memory:"])
    call_builtin("db_close", [c])
    # Doppeltes Close oder Operationen danach -> Fehler
    with pytest.raises(GBRuntimeError, match="bereits geschlossen"):
        call_builtin("db_exec", [c, "SELECT 1"])


def test_exec_insert_returns_rowcount(conn, call_builtin):
    rc = call_builtin("db_exec", [conn,
        "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", "Anna", 100, True
    ])
    assert rc == 1


def test_query_iterate(conn, call_builtin):
    call_builtin("db_exec", [conn, "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", "Anna", 100, True])
    call_builtin("db_exec", [conn, "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", "Bert", 50, True])

    r = call_builtin("db_query", [conn, "SELECT name, score FROM t ORDER BY score DESC"])
    rows = []
    while call_builtin("db_next", [r]):
        rows.append((call_builtin("db_get_string", [r, 0]),
                     call_builtin("db_get_int", [r, 1])))
    call_builtin("db_close_result", [r])
    assert rows == [("Anna", 100), ("Bert", 50)]


def test_get_bool(conn, call_builtin):
    call_builtin("db_exec", [conn, "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", "Anna", 1, True])
    call_builtin("db_exec", [conn, "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", "Bert", 0, False])
    r = call_builtin("db_query", [conn, "SELECT name, dabei FROM t ORDER BY name"])
    results = {}
    while call_builtin("db_next", [r]):
        results[call_builtin("db_get_string", [r, 0])] = call_builtin("db_get_bool", [r, 1])
    call_builtin("db_close_result", [r])
    assert results == {"Anna": True, "Bert": False}


def test_null_handling(conn, call_builtin):
    call_builtin("db_exec", [conn, "INSERT INTO t (name, score) VALUES (?, NULL)", "Emil"])
    r = call_builtin("db_query", [conn, "SELECT score FROM t WHERE name = ?", "Emil"])
    assert call_builtin("db_next", [r]) is True
    assert call_builtin("db_is_null", [r, 0]) is True
    assert call_builtin("db_get_int", [r, 0]) == 0       # default
    assert call_builtin("db_get_string", [r, 0]) == ""   # default
    call_builtin("db_close_result", [r])


def test_last_rowid(conn, call_builtin):
    call_builtin("db_exec", [conn, "INSERT INTO t (name) VALUES (?)", "X"])
    rid = call_builtin("db_last_rowid", [conn])
    assert rid == 1
    call_builtin("db_exec", [conn, "INSERT INTO t (name) VALUES (?)", "Y"])
    assert call_builtin("db_last_rowid", [conn]) == 2


def test_col_count_and_name(conn, call_builtin):
    r = call_builtin("db_query", [conn, "SELECT id, name, score FROM t"])
    assert call_builtin("db_col_count", [r]) == 3
    assert call_builtin("db_col_name", [r, 0]) == "id"
    assert call_builtin("db_col_name", [r, 1]) == "name"
    call_builtin("db_close_result", [r])


def test_transaction_rollback(conn, call_builtin):
    call_builtin("db_begin", [conn])
    call_builtin("db_exec", [conn, "INSERT INTO t (name) VALUES (?)", "ToRollback"])
    call_builtin("db_rollback", [conn])
    r = call_builtin("db_query", [conn, "SELECT COUNT(*) FROM t WHERE name = ?", "ToRollback"])
    call_builtin("db_next", [r])
    assert call_builtin("db_get_int", [r, 0]) == 0
    call_builtin("db_close_result", [r])


def test_transaction_commit(conn, call_builtin):
    call_builtin("db_begin", [conn])
    call_builtin("db_exec", [conn, "INSERT INTO t (name) VALUES (?)", "Committed"])
    call_builtin("db_commit", [conn])
    r = call_builtin("db_query", [conn, "SELECT COUNT(*) FROM t WHERE name = ?", "Committed"])
    call_builtin("db_next", [r])
    assert call_builtin("db_get_int", [r, 0]) == 1
    call_builtin("db_close_result", [r])


def test_invalid_sql_raises(conn, call_builtin):
    with pytest.raises(GBRuntimeError, match="DB_EXEC"):
        call_builtin("db_exec", [conn, "DAS IST KEIN SQL"])


def test_get_before_next_raises(conn, call_builtin):
    r = call_builtin("db_query", [conn, "SELECT 1"])
    with pytest.raises(GBRuntimeError, match="Keine aktuelle Zeile"):
        call_builtin("db_get_int", [r, 0])
    call_builtin("db_close_result", [r])
