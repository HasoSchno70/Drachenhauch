# Modul `db`

SQLite-Datenbank: erstellen, einfügen, abfragen, Transaktionen. Nativ in dhrt über die Rust-Crate `rusqlite` (mit eingebautem SQLite -- kein System-SQLite nötig, keine Extra-Installation).

```basic
IMPORT "db"
```

## Übersicht

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `DB_OPEN(pfad$)` | DB_CONN | Datenbank oeffnen oder anlegen (`":memory:"` = fluechtig, nur im Speicher) |
| `DB_CLOSE(conn)` | — | Verbindung schliessen |
| `DB_EXEC(conn, sql$, ...)` | INTEGER (rowcount) | SQL ohne Ergebnismenge (INSERT/UPDATE/DELETE/CREATE); Werte mit `?` binden, liefert die Zahl betroffener Zeilen |
| `DB_QUERY(conn, sql$, ...)` | DB_RESULT | SELECT ausfuehren und einen Cursor auf das Ergebnis liefern |
| `DB_NEXT(result)` | BOOLEAN | eine Zeile weiterruecken; FALSE, wenn keine mehr kommt |
| `DB_GET_STRING(r, idx)` | STRING | Spalte der aktuellen Zeile als Text lesen (Index ab 0) |
| `DB_GET_INT(r, idx)` | INTEGER | Spalte der aktuellen Zeile als ganze Zahl lesen |
| `DB_GET_FLOAT(r, idx)` | FLOAT | Spalte der aktuellen Zeile als Kommazahl lesen |
| `DB_GET_BOOL(r, idx)` | BOOLEAN | Spalte der aktuellen Zeile als Wahrheitswert lesen (0/1 in SQLite) |
| `DB_IS_NULL(r, idx)` | BOOLEAN | steht in dieser Spalte NULL? |
| `DB_COL_COUNT(r)` | INTEGER | Anzahl der Spalten im Ergebnis |
| `DB_COL_NAME(r, idx)` | STRING | Name der Spalte (Index ab 0) |
| `DB_CLOSE_RESULT(r)` | — | Ergebnis freigeben, wenn man es nicht zu Ende liest |
| `DB_LAST_ROWID(conn)` | INTEGER | rowid der zuletzt eingefuegten Zeile |
| `DB_BEGIN(conn)`, `DB_COMMIT(conn)`, `DB_ROLLBACK(conn)` | — | Transaktion beginnen, festschreiben, verwerfen |

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

Siehe [examples/25_db.dh](../examples/25_db.dh) — zeigt CREATE, INSERT mit Binding, mehrere Queries, NULL, Transaktionen mit ROLLBACK/COMMIT, Aggregate.

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


## Abfragen im Hintergrund

`DB_QUERY` hält das ganze Programm an, bis die Antwort da ist. Bei einer großen
Tabelle heißt das in einem Fenster: keine Maus, keine Taste, kein Neuzeichnen.

Für alles, was in einer Schleife läuft, gibt es dieselbe Abfrage zum Nachsehen
statt zum Warten — dasselbe Muster wie `HTTP_GET_START`:

| Funktion | Wirkung |
|---|---|
| `DB_QUERY_START(datei$, sql$ [, params…])` → INTEGER | startet im Hintergrund, liefert sofort die Auftragsnummer |
| `DB_QUERY_READY(auftrag)` → BOOLEAN | ist das Ergebnis da? (fragt nach, wartet nicht) |
| `DB_QUERY_RESULT(auftrag)` → DB_RESULT | Ergebnis abholen und den Platz freigeben |
| `DB_QUERY_CANCEL(auftrag)` | verwerfen (unbekannte Nummer = wirkungslos) |
| `DB_QUERY_PENDING()` → INTEGER | wie viele Abfragen noch offen sind |

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
    ' ... zeichnen ...
    FLIP()
WEND
```

> **Der Auftrag öffnet eine eigene Verbindung zur Datei** — die des Programms
> kann währenddessen weiterbenutzt werden. Der Preis dafür: der Auftrag sieht
> nur, was schon **festgeschrieben** ist, nicht die offene Transaktion des
> Programms. Für Lesen im Hintergrund ist das genau richtig; wer im selben
> Atemzug schreibt und liest, nimmt das gewohnte `DB_QUERY`.
>
> Der Grund ist technisch: eine SQLite-Verbindung darf zwar den Thread
> wechseln, aber nicht auf zweien gleichzeitig benutzt werden. Sie dem Auftrag
> mitzugeben hieße, sie dem Hauptthread wegzunehmen.

**Fehler kommen beim Abholen.** Kaputtes SQL oder eine fehlende Tabelle lässt
`DB_QUERY_START` durchgehen und wirft erst bei `DB_QUERY_RESULT` — dort, wo das
Programm damit umgehen kann (`TRY`/`CATCH` um das Abholen).
