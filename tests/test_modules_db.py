"""Tests fuer das db-Modul (SQLite, In-Memory).

Golden-Tests gegen `gbrt` (Stufe B): IMPORT "db" + `:memory:`-DB + PRINT.
Frueher via `call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""
import pytest

from gamebasic.errors import GBRuntimeError

_SCHEMA = ('IMPORT "db"\nDIM c AS DB_CONN\nc = DB_OPEN(":memory:")\n'
           'DB_EXEC(c, "CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT, '
           'score INTEGER, dabei INTEGER)")\n')


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def test_open_close(run_gb):
    with pytest.raises(GBRuntimeError, match="geschlossen"):
        run_gb('IMPORT "db"\nDIM c AS DB_CONN\nc = DB_OPEN(":memory:")\n'
               'DB_CLOSE(c)\nDB_EXEC(c, "SELECT 1")\n')


def test_exec_insert_returns_rowcount(run_gb):
    out = _lines(run_gb(_SCHEMA +
        'PRINT DB_EXEC(c, "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", '
        '"Anna", 100, TRUE)\n'))
    assert out == ["1"]


def test_query_iterate(run_gb):
    out = _lines(run_gb(_SCHEMA +
        'DB_EXEC(c, "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", "Anna", 100, TRUE)\n'
        'DB_EXEC(c, "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", "Bert", 50, TRUE)\n'
        'DIM r AS DB_RESULT\n'
        'r = DB_QUERY(c, "SELECT name, score FROM t ORDER BY score DESC")\n'
        'WHILE DB_NEXT(r)\n'
        '    PRINT DB_GET_STRING(r, 0) + "=" + STR$(DB_GET_INT(r, 1))\n'
        'WEND\nDB_CLOSE_RESULT(r)\n'))
    assert out == ["Anna=100", "Bert=50"]


def test_get_bool(run_gb):
    out = _lines(run_gb(_SCHEMA +
        'DB_EXEC(c, "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", "Anna", 1, TRUE)\n'
        'DB_EXEC(c, "INSERT INTO t (name, score, dabei) VALUES (?, ?, ?)", "Bert", 0, FALSE)\n'
        'DIM r AS DB_RESULT\n'
        'r = DB_QUERY(c, "SELECT name, dabei FROM t ORDER BY name")\n'
        'WHILE DB_NEXT(r)\n'
        '    PRINT DB_GET_STRING(r, 0) + "=" + STR$(DB_GET_BOOL(r, 1))\n'
        'WEND\nDB_CLOSE_RESULT(r)\n'))
    assert out == ["Anna=TRUE", "Bert=FALSE"]


def test_null_handling(run_gb):
    out = _lines(run_gb(_SCHEMA +
        'DB_EXEC(c, "INSERT INTO t (name, score) VALUES (?, NULL)", "Emil")\n'
        'DIM r AS DB_RESULT\n'
        'r = DB_QUERY(c, "SELECT score FROM t WHERE name = ?", "Emil")\n'
        'PRINT DB_NEXT(r)\n'
        'PRINT DB_IS_NULL(r, 0)\n'
        'PRINT DB_GET_INT(r, 0)\n'
        'PRINT "[" + DB_GET_STRING(r, 0) + "]"\n'
        'DB_CLOSE_RESULT(r)\n'))
    assert out == ["TRUE", "TRUE", "0", "[]"]


def test_last_rowid(run_gb):
    out = _lines(run_gb(_SCHEMA +
        'DB_EXEC(c, "INSERT INTO t (name) VALUES (?)", "X")\n'
        'PRINT DB_LAST_ROWID(c)\n'
        'DB_EXEC(c, "INSERT INTO t (name) VALUES (?)", "Y")\n'
        'PRINT DB_LAST_ROWID(c)\n'))
    assert out == ["1", "2"]


def test_col_count_and_name(run_gb):
    out = _lines(run_gb(_SCHEMA +
        'DIM r AS DB_RESULT\nr = DB_QUERY(c, "SELECT id, name, score FROM t")\n'
        'PRINT DB_COL_COUNT(r)\nPRINT DB_COL_NAME(r, 0)\nPRINT DB_COL_NAME(r, 1)\n'
        'DB_CLOSE_RESULT(r)\n'))
    assert out == ["3", "id", "name"]


def test_transaction_rollback(run_gb):
    out = _lines(run_gb(_SCHEMA +
        'DB_BEGIN(c)\n'
        'DB_EXEC(c, "INSERT INTO t (name) VALUES (?)", "ToRollback")\n'
        'DB_ROLLBACK(c)\n'
        'DIM r AS DB_RESULT\n'
        'r = DB_QUERY(c, "SELECT COUNT(*) FROM t WHERE name = ?", "ToRollback")\n'
        'DB_NEXT(r)\nPRINT DB_GET_INT(r, 0)\nDB_CLOSE_RESULT(r)\n'))
    assert out == ["0"]


def test_transaction_commit(run_gb):
    out = _lines(run_gb(_SCHEMA +
        'DB_BEGIN(c)\n'
        'DB_EXEC(c, "INSERT INTO t (name) VALUES (?)", "Committed")\n'
        'DB_COMMIT(c)\n'
        'DIM r AS DB_RESULT\n'
        'r = DB_QUERY(c, "SELECT COUNT(*) FROM t WHERE name = ?", "Committed")\n'
        'DB_NEXT(r)\nPRINT DB_GET_INT(r, 0)\nDB_CLOSE_RESULT(r)\n'))
    assert out == ["1"]


def test_invalid_sql_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="DB_EXEC"):
        run_gb(_SCHEMA + 'DB_EXEC(c, "DAS IST KEIN SQL")\n')


def test_get_before_next_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="Keine aktuelle Zeile"):
        run_gb(_SCHEMA +
               'DIM r AS DB_RESULT\nr = DB_QUERY(c, "SELECT 1")\n'
               'PRINT DB_GET_INT(r, 0)\n')
