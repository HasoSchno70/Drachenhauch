# Module `db`

Databases: create, insert, query, transactions. Either an SQLite file (native in dhrt via the Rust crate `rusqlite`, with SQLite built in -- nothing extra to install) or a **PostgreSQL or MySQL server** -- with the same commands; `DB_OPEN` decides by the target (see [Server](#server-postgresql-und-mysql)).

```basic
IMPORT "db"
```

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `DB_OPEN(ziel$)` | DB_CONN | open a database: a file (created if it does not exist; `":memory:"` = volatile), `postgres://...` or `mysql://...` |
| `DB_CLOSE(conn)` | — | close the connection |
| `DB_EXEC(conn, sql$, ...)` | INTEGER (rowcount) | SQL without a result set (INSERT/UPDATE/DELETE/CREATE); bind values with `?`, returns the number of affected rows |
| `DB_QUERY(conn, sql$, ...)` | DB_RESULT | run a SELECT and return a cursor on the result |
| `DB_NEXT(result)` | BOOLEAN | advance by one row; FALSE when there are no more |
| `DB_GET_STRING(r, idx)` | STRING | read a column of the current row as text (index from 0) |
| `DB_GET_INT(r, idx)` | INTEGER | read a column of the current row as an integer |
| `DB_GET_FLOAT(r, idx)` | FLOAT | read a column of the current row as a floating-point number |
| `DB_GET_BOOL(r, idx)` | BOOLEAN | read a column of the current row as a truth value (0/1 in SQLite) |
| `DB_IS_NULL(r, idx)` | BOOLEAN | is this column NULL? |
| `DB_COL_COUNT(r)` | INTEGER | number of columns in the result |
| `DB_COL_NAME(r, idx)` | STRING | name of the column (index from 0) |
| `DB_CLOSE_RESULT(r)` | — | release a result if you do not read it to the end |
| `DB_LAST_ROWID(conn)` | INTEGER | ID of the most recently inserted row (SQLite rowid, MySQL `AUTO_INCREMENT`, PostgreSQL `lastval()`) |
| `DB_KIND$(conn)` | STRING | which database: `sqlite`, `postgres` or `mysql` |
| `DB_PING(conn)` | BOOLEAN | is the connection still up? (a file always is; a server answers within 5 s or not) |
| `DB_BEGIN(conn)`, `DB_COMMIT(conn)`, `DB_ROLLBACK(conn)` | — | begin, commit, roll back a transaction |

## Server: PostgreSQL and MySQL

As soon as several workstations work with the same data, a file is no longer
enough. The same program then talks to a server -- only the target in
`DB_OPEN` changes:

```basic
IMPORT "db"
DIM c AS DB_CONN
c = DB_OPEN("postgres://hans:geheim@server:5432/laden")
' or: c = DB_OPEN("mysql://hans:geheim@server:3306/laden")

DB_EXEC(c, "INSERT INTO kunde (name, ort) VALUES (?, ?)", "Meier", "Köln")
DIM r AS DB_RESULT
r = DB_QUERY(c, "SELECT name, umsatz FROM kunde WHERE ort = ?", "Köln")
WHILE DB_NEXT(r)
    PRINT DB_GET_STRING(r, 0), DB_GET_FLOAT(r, 1)
WEND
DB_CLOSE(c)
```

* **The target:** `postgres://nutzer:kennwort@host:port/datenbank` (also
  `postgresql://`), `mysql://nutzer:kennwort@host:port/datenbank` (also
  `mariadb://`) -- user, password, host, port, database. Special characters
  in the password are encoded as in any address (`@` as `%40`).
* **Placeholders are `?` -- for all three.** PostgreSQL itself writes
  `$1`, `$2`; Drachenhauch translates `?` into those, except inside strings,
  identifiers and comments. If you write `$1` yourself (say, to use a value
  twice, or next to the `jsonb` operators `?`/`?|`), your SQL is passed on
  unchanged.
* **Encryption** is used by default if the server supports it
  (`sslmode=prefer`). `?sslmode=require` demands it, `?sslmode=disable`
  does without it, `?sslmode=verify-full` additionally checks the certificate
  against the known certificate authorities -- for servers on the network
  whose certificate comes from a real authority (cloud databases). Without
  `verify-full` every server is trusted, as with `psql`; your own server with
  a self-issued certificate thus works without any detour. With MySQL,
  `verify-full` checks the name from the target -- an IP address instead of
  the name does not work there.
* **Exact decimal numbers** (`numeric`, `DECIMAL`) arrive as text:
  `DB_GET_STRING` returns `19.99` unchanged, `DB_GET_FLOAT` converts it,
  `DB_GET_INT` only accepts one without decimal places. For money this is the
  right way (see [Money](builtins-core.md#calculating-with-money)).
* **Other types:** `BOOLEAN` arrives as 1/0 (for `DB_GET_BOOL`, as with
  SQLite), date and time as text (`2024-02-29`, `2026-10-06 13:05:09`;
  PostgreSQL's `timestamptz` in UTC with `+00`); bytes (`BYTEA`, `BLOB`) are
  read by `DB_GET_STRING` as text and bound as a `BUFFER`. With PostgreSQL,
  `uuid`, `json`/`jsonb`, `interval` and `money` also come as text.
  Whatever Drachenhauch cannot read (a field `int[]`, for example) is named in
  the message, together with the way out: `::text` in the query.
* **With PostgreSQL, values are bound as text**, and the server converts:
  `"19.99"` into a `numeric` column, `42` into `int4`, `TRUE` (as 1) into
  `boolean`.
* **`DB_LAST_ROWID`** returns, with PostgreSQL, the last value of a sequence
  in this connection (`lastval()`), i.e. what a `SERIAL` or `IDENTITY`
  column has assigned. Completely safe -- even with several tables -- is
  `INSERT ... RETURNING id` with `DB_QUERY`.
* **The connection can drop.** A file does not do that, a server does: every
  call then reports it as an error, `DB_PING` asks.
* **Errors from the server** come with its error number: `DB_EXEC: relation
  "kunde" does not exist (PostgreSQL 42P01)`, `Table 'laden.kunde' doesn't
  exist (MySQL 1146)`.
* **`DB_QUERY_START`** takes the same target and opens its own connection to
  the server for the job; **`GUI_FORM_LOAD`/`GUI_FORM_SAVE`** work the same
  way (with PostgreSQL, saving fetches the new ID via `RETURNING id`).
* **Whatever differs**, you write differently yourself: creating tables
  (`SERIAL` versus `AUTO_INCREMENT`, `BYTEA` versus `BLOB`). `DB_KIND$` tells
  you whom the program is talking to.

The drivers are pure Rust (`postgres`, `mysql`); encryption goes through the
same `rustls` as for `smtp` and `html`. They are part of the full build and
of the console runtime, not of the game runtime of a slim export. Without
them, `DB_OPEN` with a server target reports that the build does not know
it.

## Connection

`":memory:"` for a volatile in-memory database, otherwise a file path:

```basic
IMPORT "db"

DIM con AS DB_CONN
con = DB_OPEN("highscores.db")        ' or ":memory:"

DB_EXEC(con, "CREATE TABLE IF NOT EXISTS scores (name TEXT, score INTEGER)")

' ... work ...

DB_CLOSE(con)
```

## Writing (Insert / Update / Delete)

`DB_EXEC` is for DDL and all DML except SELECT. Parameters are bound with `?` — **always use parameter binding**, never build SQL out of strings (SQL injection):

```basic
DIM rc AS INTEGER
rc = DB_EXEC(con, "INSERT INTO scores (name, score) VALUES (?, ?)", "Anna", 95)
PRINT "Eingefuegt, rowid =", DB_LAST_ROWID(con)

' UPDATE
rc = DB_EXEC(con, "UPDATE scores SET score = score + 10 WHERE name = ?", "Anna")
PRINT rc, " Zeile(n) geupdated"

' DELETE
DB_EXEC(con, "DELETE FROM scores WHERE score < ?", 50)
```

Supported parameter types: INTEGER, FLOAT, STRING, BOOLEAN (stored as 0/1), NIL (becomes NULL).

## Reading (SELECT)

`DB_QUERY` returns a `DB_RESULT`. Iterate with `DB_NEXT`, read columns with `DB_GET_*` (typed):

```basic
DIM r AS DB_RESULT
r = DB_QUERY(con, "SELECT name, score FROM scores WHERE score > ? ORDER BY score DESC", 80)

WHILE DB_NEXT(r)
    PRINT DB_GET_STRING(r, 0), ": ", DB_GET_INT(r, 1)
WEND

DB_CLOSE_RESULT(r)
```

Columns are read by a 0-based index (`0` = first column in the SELECT).

## Handling NULL

`DB_GET_*` return defaults for NULL: `0` for INT, `0.0` for FLOAT, `""` for STRING, `FALSE` for BOOL. To detect NULL, use `DB_IS_NULL`:

```basic
r = DB_QUERY(con, "SELECT name, email FROM users WHERE id = ?", 42)
IF DB_NEXT(r) THEN
    PRINT DB_GET_STRING(r, 0)
    IF DB_IS_NULL(r, 1) THEN
        PRINT "  (keine Email)"
    ELSE
        PRINT "  Email: ", DB_GET_STRING(r, 1)
    END IF
END IF
DB_CLOSE_RESULT(r)
```

## Schema inspection

`DB_COL_COUNT(r)` and `DB_COL_NAME(r, idx)` return the number and names of the columns:

```basic
r = DB_QUERY(con, "SELECT * FROM scores")
DIM i AS INTEGER
FOR i = 0 TO DB_COL_COUNT(r) - 1
    PRINT "Spalte ", i, ": ", DB_COL_NAME(r, i)
NEXT
DB_CLOSE_RESULT(r)
```

## Transactions

By default auto-commit is on — every `DB_EXEC` is atomic. For several statements in one transaction:

```basic
DB_BEGIN(con)
TRY
    DB_EXEC(con, "INSERT INTO scores VALUES (?, ?)", "Anna", 100)
    DB_EXEC(con, "INSERT INTO scores VALUES (?, ?)", "Bert", 95)
    DB_EXEC(con, "UPDATE meta SET total = total + 2")
    DB_COMMIT(con)
CATCH e
    DB_ROLLBACK(con)
    PRINT "Fehler, zurueckgerollt: ", e
END TRY
```

## Booleans

SQLite has no native BOOLEAN type, so values are stored as `0` or `1` (INTEGER). `DB_GET_BOOL` reads them back correctly: anything other than 0 → TRUE.

```basic
DB_EXEC(con, "INSERT INTO users (name, aktiv) VALUES (?, ?)", "Anna", TRUE)
r = DB_QUERY(con, "SELECT name, aktiv FROM users")
WHILE DB_NEXT(r)
    PRINT DB_GET_STRING(r, 0), " aktiv? ", DB_GET_BOOL(r, 1)
WEND
DB_CLOSE_RESULT(r)
```

## Complete example

See [examples/25_db.dh](../../examples/25_db.dh) — shows CREATE, INSERT with binding, several queries, NULL, transactions with ROLLBACK/COMMIT, aggregates.

## Best practices

- **Always use `?` for parameters** — never interpolate strings.
- **Close the result** with `DB_CLOSE_RESULT(r)` when you are done (otherwise the eagerly loaded rows stay in memory until the program ends).
- **Close the connection** with `DB_CLOSE(conn)` at the end of the program.
- **Create indexes** for columns frequently used in WHERE/ORDER BY:
  ```basic
  DB_EXEC(con, "CREATE INDEX IF NOT EXISTS idx_score ON scores(score)")
  ```
- **Initialise the schema** with `IF NOT EXISTS` — then starting the program several times is harmless.

## In the native runtime (dhrt)

`db` runs natively with the Cargo feature `db` (SQLite via `rusqlite`, bundled — no system SQLite needed). Bit-identical to the Python paths for standard SQL (CRUD, `?` binding, transactions, typed getters). `DB_QUERY` loads the rows eagerly into memory; `DB_CLOSE_RESULT` releases them again. Build: `python rust/build_runtime.py` (the `db` feature is already part of the standard dev build). If the feature is missing, the built-in reports “not available”.


## Queries in the background

`DB_QUERY` halts the whole program until the answer is there. With a large
table, in a window that means: no mouse, no keys, no redrawing.

For anything that runs in a loop, there is the same query to check on instead
of waiting for — the same pattern as `HTTP_GET_START`:

| Function | Effect |
|---|---|
| `DB_QUERY_START(ziel$, sql$ [, params…])` → INTEGER | starts in the background, immediately returns the job number (a file or a server, as with `DB_OPEN`) |
| `DB_QUERY_READY(auftrag)` → BOOLEAN | is the result there? (checks, does not wait) |
| `DB_QUERY_RESULT(auftrag)` → DB_RESULT | fetch the result and free the slot |
| `DB_QUERY_CANCEL(auftrag)` | discard (unknown number = no effect) |
| `DB_QUERY_PENDING()` → INTEGER | how many queries are still open |

```basic
DIM auftrag AS INTEGER
DIM erg AS INTEGER
auftrag = DB_QUERY_START("spiel.db", "SELECT * FROM bestenliste ORDER BY punkte DESC")

WHILE NOT QUITREQUESTED()
    CLS()
    IF auftrag >= 0 AND DB_QUERY_READY(auftrag) THEN
        erg = DB_QUERY_RESULT(auftrag)
        auftrag = -1
    END IF
    ' ... draw ...
    FLIP()
WEND
```

> **The job opens its own connection to the file** — the program's own
> connection can be used meanwhile. The price: the job only sees what has
> already been **committed**, not the program's open transaction. For reading
> in the background that is exactly right; if you write and read in the same
> breath, use the usual `DB_QUERY`.
>
> The reason is technical: an SQLite connection may change threads, but must
> not be used on two at the same time. Handing it to the job would mean
> taking it away from the main thread.

**Errors arrive when you fetch.** Broken SQL or a missing table gets through
`DB_QUERY_START` and only throws at `DB_QUERY_RESULT` — where the program can
deal with it (`TRY`/`CATCH` around the fetch).
