# Modul `db`

SQLite-Datenbank: erstellen, einfügen, abfragen, Transaktionen. Nutzt Pythons `sqlite3` (eingebaut, keine Extra-Installation).

```basic
IMPORT "db"
```

## Übersicht

| Funktion | Rückgabe |
|---|---|
| `DB_OPEN(pfad$)` | DB_CONN |
| `DB_CLOSE(conn)` | — |
| `DB_EXEC(conn, sql$, ...)` | INTEGER (rowcount) |
| `DB_QUERY(conn, sql$, ...)` | DB_RESULT |
| `DB_NEXT(result)` | BOOLEAN |
| `DB_GET_STRING(r, idx)` | STRING |
| `DB_GET_INT(r, idx)` | INTEGER |
| `DB_GET_FLOAT(r, idx)` | FLOAT |
| `DB_GET_BOOL(r, idx)` | BOOLEAN |
| `DB_IS_NULL(r, idx)` | BOOLEAN |
| `DB_COL_COUNT(r)` | INTEGER |
| `DB_COL_NAME(r, idx)` | STRING |
| `DB_CLOSE_RESULT(r)` | — |
| `DB_LAST_ROWID(conn)` | INTEGER |
| `DB_BEGIN(conn)`, `DB_COMMIT(conn)`, `DB_ROLLBACK(conn)` | — |

## Verbindung

`":memory:"` für eine flüchtige In-Memory-Datenbank, sonst Dateipfad:

```basic
IMPORT "db"

DIM con AS DB_CONN
con = DB_OPEN("highscores.db")        ' oder ":memory:"

DB_EXEC(con, "CREATE TABLE IF NOT EXISTS scores (name TEXT, score INTEGER)")

' ... arbeiten ...

DB_CLOSE(con)
```

## Schreiben (Insert / Update / Delete)

`DB_EXEC` ist für DDL und alle DML außer SELECT. Parameter werden mit `?` gebunden — **immer Parameter-Binding nutzen**, niemals SQL stringly konstruieren (SQL-Injection):

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

Unterstützte Parameter-Typen: INTEGER, FLOAT, STRING, BOOLEAN (wird als 0/1 gespeichert), NIL (wird zu NULL).

## Lesen (SELECT)

`DB_QUERY` gibt einen `DB_RESULT` zurück. Iteriere mit `DB_NEXT`, lies Spalten mit `DB_GET_*` (typisiert):

```basic
DIM r AS DB_RESULT
r = DB_QUERY(con, "SELECT name, score FROM scores WHERE score > ? ORDER BY score DESC", 80)

WHILE DB_NEXT(r)
    PRINT DB_GET_STRING(r, 0), ": ", DB_GET_INT(r, 1)
WEND

DB_CLOSE_RESULT(r)
```

Spalten werden über 0-basierten Index gelesen (`0` = erste Spalte im SELECT).

## NULL-Behandlung

`DB_GET_*` geben Defaults zurück bei NULL: `0` für INT, `0.0` für FLOAT, `""` für STRING, `FALSE` für BOOL. Wer NULL erkennen will, nutzt `DB_IS_NULL`:

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

## Schema-Inspektion

`DB_COL_COUNT(r)` und `DB_COL_NAME(r, idx)` liefern Spalten-Anzahl und -Namen:

```basic
r = DB_QUERY(con, "SELECT * FROM scores")
DIM i AS INTEGER
FOR i = 0 TO DB_COL_COUNT(r) - 1
    PRINT "Spalte ", i, ": ", DB_COL_NAME(r, i)
NEXT
DB_CLOSE_RESULT(r)
```

## Transaktionen

Standardmäßig läuft Auto-Commit — jedes `DB_EXEC` ist atomar. Für mehrere Statements in einer Transaktion:

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

SQLite kennt keinen nativen BOOLEAN-Typ, daher werden Werte als `0` oder `1` (INTEGER) gespeichert. `DB_GET_BOOL` liest sie korrekt zurück: alles ungleich 0 → TRUE.

```basic
DB_EXEC(con, "INSERT INTO users (name, aktiv) VALUES (?, ?)", "Anna", TRUE)
r = DB_QUERY(con, "SELECT name, aktiv FROM users")
WHILE DB_NEXT(r)
    PRINT DB_GET_STRING(r, 0), " aktiv? ", DB_GET_BOOL(r, 1)
WEND
DB_CLOSE_RESULT(r)
```

## Komplettes Beispiel

Siehe [examples/25_db.gb](../examples/25_db.gb) — zeigt CREATE, INSERT mit Binding, mehrere Queries, NULL, Transaktionen mit ROLLBACK/COMMIT, Aggregate.

## Best Practices

- **Immer `?` für Parameter** — niemals Strings interpolieren.
- **Result schließen** mit `DB_CLOSE_RESULT(r)` wenn fertig (sonst bleiben die eager geladenen Zeilen bis Programmende im Speicher).
- **Verbindung schließen** mit `DB_CLOSE(conn)` am Programm-Ende.
- **Indizes** anlegen für häufige WHERE/ORDER BY-Spalten:
  ```basic
  DB_EXEC(con, "CREATE INDEX IF NOT EXISTS idx_score ON scores(score)")
  ```
- **Schema-Init** mit `IF NOT EXISTS` — dann ist mehrfaches Programm-Starten ungefährlich.

## In der nativen Runtime (dhrt)

`db` laeuft nativ mit dem Cargo-Feature `db` (SQLite via `rusqlite`, gebuendelt — kein System-SQLite noetig). Bit-identisch zu den Python-Pfaden fuer Standard-SQL (CRUD, `?`-Binding, Transaktionen, typisierte Getter). `DB_QUERY` laedt die Zeilen eager in den Speicher; `DB_CLOSE_RESULT` gibt sie wieder frei. Bauen: `python rust/build_runtime.py` (Feature `db` ist im Standard-Dev-Build bereits dabei). Fehlt das Feature, meldet der Builtin „nicht verfuegbar“.
