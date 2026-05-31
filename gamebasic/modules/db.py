"""SQLite-Datenbank-Modul fuer GameBasic.

Built-ins:
    DB_OPEN(pfad$)             -> DB_CONN
    DB_CLOSE(conn)
    DB_EXEC(conn, sql$, ...)   -> INTEGER (rowcount)   ' DDL/DML
    DB_QUERY(conn, sql$, ...)  -> DB_RESULT             ' SELECT
    DB_NEXT(result)            -> BOOLEAN               ' naechste Zeile
    DB_COL_COUNT(result)       -> INTEGER
    DB_COL_NAME(result, idx)   -> STRING
    DB_GET_STRING(result, idx) -> STRING                ' NULL -> ""
    DB_GET_INT(result, idx)    -> INTEGER               ' NULL -> 0
    DB_GET_FLOAT(result, idx)  -> FLOAT                 ' NULL -> 0.0
    DB_GET_BOOL(result, idx)   -> BOOLEAN               ' NULL -> FALSE
    DB_IS_NULL(result, idx)    -> BOOLEAN
    DB_CLOSE_RESULT(result)
    DB_LAST_ROWID(conn)        -> INTEGER
    DB_BEGIN(conn)                                       ' Transaktionen
    DB_COMMIT(conn)
    DB_ROLLBACK(conn)

Parameter-Binding mit `?`-Platzhaltern in SQL ist sicher gegen SQL-Injection:

    DB_EXEC(conn, "INSERT INTO users (name, age) VALUES (?, ?)", "Anna", 30)

Auto-Commit ist aktiv. Fuer Atomaritaet ueber mehrere Statements
DB_BEGIN/DB_COMMIT/DB_ROLLBACK explizit nutzen.
"""
from __future__ import annotations

import sqlite3 as _sql

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type


class _DBConn:
    __slots__ = ("conn", "path")

    def __init__(self, conn, path: str):
        self.conn = conn
        self.path = path

    def __repr__(self):
        state = "open" if self.conn is not None else "closed"
        return f"<DB '{self.path}' {state}>"


class _DBResult:
    __slots__ = ("cursor", "current_row", "_closed")

    def __init__(self, cursor):
        self.cursor = cursor
        self.current_row = None
        self._closed = False

    def __repr__(self):
        n = len(self.cursor.description or []) if not self._closed else 0
        return f"<DB_RESULT cols={n}>"


register_type("db_conn", _DBConn)
register_type("db_result", _DBResult)


# --- Helfer ----------------------------------------------------------

def _check_conn(v, fn: str) -> _DBConn:
    if not isinstance(v, _DBConn):
        raise TypeMismatchError(f"{fn} erwartet DB_CONN")
    if v.conn is None:
        raise GBRuntimeError(f"{fn}: Verbindung wurde bereits geschlossen")
    return v


def _check_result(v, fn: str) -> _DBResult:
    if not isinstance(v, _DBResult):
        raise TypeMismatchError(f"{fn} erwartet DB_RESULT")
    if v._closed:
        raise GBRuntimeError(f"{fn}: Result wurde bereits geschlossen")
    return v


def _coerce_param(v, fn: str):
    """GB-Wert in einen SQLite-akzeptierten Wert konvertieren."""
    if v is None:
        return None
    if isinstance(v, bool):
        # SQLite hat kein Boolean - speichern als 0/1
        return 1 if v else 0
    if isinstance(v, (int, float, str)):
        return v
    raise TypeMismatchError(
        f"{fn}: Parameter-Typ {type(v).__name__} nicht unterstuetzt "
        f"(erlaubt: INTEGER, FLOAT, STRING, BOOLEAN, NIL)"
    )


def _check_str(v, fn: str) -> str:
    if not isinstance(v, str):
        raise TypeMismatchError(f"{fn}: SQL muss STRING sein, ist {type(v).__name__}")
    return v


def _check_int(v, fn: str) -> int:
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn}: Spalten-Index muss INTEGER sein")
    return v


def _get_value(r: _DBResult, idx: int, fn: str):
    if r.current_row is None:
        raise GBRuntimeError(
            f"{fn}: Keine aktuelle Zeile - DB_NEXT vor dem Lesen aufrufen"
        )
    if idx < 0 or idx >= len(r.current_row):
        raise GBRuntimeError(
            f"{fn}: Spalten-Index {idx} ausserhalb [0..{len(r.current_row) - 1}]"
        )
    return r.current_row[idx]


# --- Verbindung ------------------------------------------------------

@builtin("DB_OPEN", arity=1, types=("str",))
def _open(path):
    try:
        conn = _sql.connect(path, isolation_level=None)  # Auto-Commit
        return _DBConn(conn, path)
    except _sql.Error as exc:
        raise GBRuntimeError(f"DB_OPEN: {exc}")


@builtin("DB_CLOSE", arity=1)
def _close(c):
    c = _check_conn(c, "DB_CLOSE")
    c.conn.close()
    c.conn = None
    return None


@builtin("DB_LAST_ROWID", arity=1)
def _last_rowid(c):
    c = _check_conn(c, "DB_LAST_ROWID")
    cur = c.conn.execute("SELECT last_insert_rowid()")
    rid = cur.fetchone()[0]
    cur.close()
    return rid


# --- Statements ------------------------------------------------------

@builtin("DB_EXEC", arity=(2, None))
def _exec(*args):
    c = _check_conn(args[0], "DB_EXEC")
    sql = _check_str(args[1], "DB_EXEC")
    params = [_coerce_param(p, "DB_EXEC") for p in args[2:]]
    try:
        cur = c.conn.execute(sql, params)
        rowcount = cur.rowcount if cur.rowcount is not None else 0
        cur.close()
        return max(0, rowcount)
    except _sql.Error as exc:
        raise GBRuntimeError(f"DB_EXEC: {exc}")


@builtin("DB_QUERY", arity=(2, None))
def _query(*args):
    c = _check_conn(args[0], "DB_QUERY")
    sql = _check_str(args[1], "DB_QUERY")
    params = [_coerce_param(p, "DB_QUERY") for p in args[2:]]
    try:
        cur = c.conn.execute(sql, params)
        return _DBResult(cur)
    except _sql.Error as exc:
        raise GBRuntimeError(f"DB_QUERY: {exc}")


# --- Result-Navigation ----------------------------------------------

@builtin("DB_NEXT", arity=1)
def _next(r):
    r = _check_result(r, "DB_NEXT")
    row = r.cursor.fetchone()
    r.current_row = row
    return row is not None


@builtin("DB_COL_COUNT", arity=1)
def _col_count(r):
    r = _check_result(r, "DB_COL_COUNT")
    return len(r.cursor.description or [])


@builtin("DB_COL_NAME", arity=2)
def _col_name(r, idx):
    r = _check_result(r, "DB_COL_NAME")
    idx = _check_int(idx, "DB_COL_NAME")
    desc = r.cursor.description
    if desc is None:
        raise GBRuntimeError("DB_COL_NAME: Result hat keine Spalten")
    if idx < 0 or idx >= len(desc):
        raise GBRuntimeError(
            f"DB_COL_NAME: Index {idx} ausserhalb [0..{len(desc) - 1}]"
        )
    return desc[idx][0]


@builtin("DB_CLOSE_RESULT", arity=1)
def _close_result(r):
    r = _check_result(r, "DB_CLOSE_RESULT")
    r.cursor.close()
    r._closed = True
    return None


# --- Werte aus aktueller Zeile lesen --------------------------------

@builtin("DB_IS_NULL", arity=2)
def _is_null(r, idx):
    r = _check_result(r, "DB_IS_NULL")
    idx = _check_int(idx, "DB_IS_NULL")
    return _get_value(r, idx, "DB_IS_NULL") is None


@builtin("DB_GET_STRING", arity=2)
def _get_string(r, idx):
    r = _check_result(r, "DB_GET_STRING")
    idx = _check_int(idx, "DB_GET_STRING")
    v = _get_value(r, idx, "DB_GET_STRING")
    if v is None:
        return ""
    if isinstance(v, str):
        return v
    if isinstance(v, bool):
        return "TRUE" if v else "FALSE"
    if isinstance(v, (int, float)):
        return str(v)
    if isinstance(v, bytes):
        return v.decode("utf-8", errors="replace")
    raise TypeMismatchError(
        f"DB_GET_STRING: Spalte {idx} ist {type(v).__name__}, nicht STRING-konvertierbar"
    )


@builtin("DB_GET_INT", arity=2)
def _get_int(r, idx):
    r = _check_result(r, "DB_GET_INT")
    idx = _check_int(idx, "DB_GET_INT")
    v = _get_value(r, idx, "DB_GET_INT")
    if v is None:
        return 0
    if isinstance(v, bool):
        return 1 if v else 0
    if isinstance(v, int):
        return v
    if isinstance(v, float) and v.is_integer():
        return int(v)
    raise TypeMismatchError(
        f"DB_GET_INT: Spalte {idx} ist {type(v).__name__}, nicht INTEGER"
    )


@builtin("DB_GET_FLOAT", arity=2)
def _get_float(r, idx):
    r = _check_result(r, "DB_GET_FLOAT")
    idx = _check_int(idx, "DB_GET_FLOAT")
    v = _get_value(r, idx, "DB_GET_FLOAT")
    if v is None:
        return 0.0
    if isinstance(v, bool):
        return 1.0 if v else 0.0
    if isinstance(v, (int, float)):
        return float(v)
    raise TypeMismatchError(
        f"DB_GET_FLOAT: Spalte {idx} ist {type(v).__name__}, nicht Zahl"
    )


@builtin("DB_GET_BOOL", arity=2)
def _get_bool(r, idx):
    r = _check_result(r, "DB_GET_BOOL")
    idx = _check_int(idx, "DB_GET_BOOL")
    v = _get_value(r, idx, "DB_GET_BOOL")
    if v is None:
        return False
    if isinstance(v, bool):
        return v
    if isinstance(v, (int, float)):
        return v != 0
    raise TypeMismatchError(
        f"DB_GET_BOOL: Spalte {idx} ist {type(v).__name__}, nicht 0/1"
    )


# --- Transaktionen --------------------------------------------------

@builtin("DB_BEGIN", arity=1)
def _begin(c):
    c = _check_conn(c, "DB_BEGIN")
    try:
        c.conn.execute("BEGIN")
    except _sql.Error as exc:
        raise GBRuntimeError(f"DB_BEGIN: {exc}")
    return None


@builtin("DB_COMMIT", arity=1)
def _commit(c):
    c = _check_conn(c, "DB_COMMIT")
    try:
        c.conn.execute("COMMIT")
    except _sql.Error as exc:
        raise GBRuntimeError(f"DB_COMMIT: {exc}")
    return None


@builtin("DB_ROLLBACK", arity=1)
def _rollback(c):
    c = _check_conn(c, "DB_ROLLBACK")
    try:
        c.conn.execute("ROLLBACK")
    except _sql.Error as exc:
        raise GBRuntimeError(f"DB_ROLLBACK: {exc}")
    return None
