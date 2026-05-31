# Variablen-Scope

GameBasic ist Pascal-strikt: Variablen müssen mit `DIM` deklariert werden, behalten ihren Typ, und die Sichtbarkeit folgt einem klaren Drei-Ebenen-Modell ohne Block-Scoping.

## Drei Scope-Ebenen

```
Globals (Top-Level)
    │
    ├─ Klassen-Felder (nur in Methoden)
    │       │
    │       └─ SUB / FUNCTION lokal
    │
    └─ SUB / FUNCTION lokal
```

Implementierungs-Referenz: [environment.py](../gamebasic/environment.py), [interpreter.py:923](../gamebasic/interpreter.py).

| Ebene | Wann | Persistenz |
|---|---|---|
| **Global** | Top-Level außerhalb SUB/FUNCTION/CLASS | gesamtes Programm |
| **Klassen-Felder** | innerhalb einer Methode automatisch sichtbar | so lange das Objekt lebt |
| **Lokal** | innerhalb SUB/FUNCTION/Methode | bis Funktion endet |

## Block-Statements machen *kein* neues Scope

`IF` / `WHILE` / `FOR` / `SELECT` / `TRY` legen kein eigenes Scope an. Was du dort `DIM`'st, lebt im umschließenden Scope weiter.

```basic
IF x > 0 THEN
    DIM hilf AS INTEGER
    hilf = 42
END IF
PRINT hilf                ' 42 - hilf existiert weiter
```

## Lookup-Regel: lokal → Felder → global

`get_slot()` läuft die Eltern-Kette hoch — beim Lesen *und* beim Schreiben. Das ist der entscheidende Unterschied zu Python.

```basic
DIM zaehler AS INTEGER
zaehler = 0

SUB inkrement()
    zaehler = zaehler + 1     ' schreibt das GLOBALE zaehler
END SUB

inkrement()
PRINT zaehler                 ' 1
```

In Python würde die Zuweisung `zaehler = zaehler + 1` eine *neue lokale* Variable anlegen (oder auf `UnboundLocalError` laufen). In GameBasic löst der Schreibzugriff durch die Eltern-Kette auf wie der Lesezugriff — ohne `global`-Schlüsselwort.

## Lokales Shadowing

Sobald innerhalb der SUB/FUNCTION ein `DIM x` steht, wird `x` lokal — Reads und Writes greifen auf die lokale Kopie:

```basic
DIM x AS INTEGER
x = 100

SUB demo()
    DIM x AS INTEGER          ' lokale x - schattet das globale ab
    x = 5
    PRINT x                   ' 5
END SUB

demo()
PRINT x                       ' 100 - global unberührt
```

## Klassen-Felder ohne Präfix

In Methoden liegen die Instanz-Felder als zwischengeschalteter Scope zwischen Locals und Globals. Kein `self.` / `this.` nötig.

```basic
CLASS Spieler
    DIM hp AS INTEGER
    DIM name AS STRING

    SUB Init(n AS STRING, start_hp AS INTEGER)
        name = n               ' direkt - kein Praefix
        hp = start_hp
    END SUB

    SUB heile(menge AS INTEGER)
        hp = hp + menge        ' liest und schreibt das Feld
        IF hp > 100 THEN hp = 100
    END SUB
END CLASS
```

Wenn ein lokaler Parameter denselben Namen wie ein Feld hätte, würde der Parameter abschatten — typisch löst man das mit unterschiedlichen Namen (oben `n` vs `name`).

## Parameter sind lokal, Argumente werden by-value/by-reference übergeben

Jeder SUB/FUNCTION-Call kreiert eine eigene `local_env`. Parameter werden dort deklariert und mit den Argument-Werten initialisiert.

| Parameter-Typ | Übergabe |
|---|---|
| `INTEGER`, `FLOAT`, `STRING`, `BOOLEAN` | by-value (Kopie) |
| Arrays | by-reference (Mutationen sichtbar beim Aufrufer) |
| Objekt-Instanzen (CLASS) | by-reference |

```basic
SUB modify_int(n AS INTEGER)
    n = n + 1                 ' verändert nur die lokale Kopie
END SUB

SUB modify_arr(a AS ARRAY OF INTEGER)
    a[0] = 99                 ' verändert das Original
END SUB
```

## FOR-Variable überlebt den Loop

`FOR i` deklariert `i` im *umschließenden* Scope (wenn nicht schon vorhanden). Nach `NEXT` behält `i` den Wert, mit dem der Abbruch ausgelöst wurde.

```basic
DIM i AS INTEGER
FOR i = 1 TO 5
    PRINT i
NEXT
PRINT "Nach Loop: ", i        ' 6
```

## CATCH-Variable ist *nicht* block-lokal

Anders als in vielen Sprachen ist die `CATCH e`-Variable nicht auf den CATCH-Block beschränkt — sie wird im umschließenden Scope angelegt und bleibt nach `END TRY` erreichbar.

```basic
TRY
    PRINT 1 / 0
CATCH msg
    PRINT "Fehler: ", msg
END TRY
PRINT "Letzte Fehler-Message: ", msg     ' funktioniert
```

Die Variable muss vom Typ `STRING` sein (oder noch nicht deklariert — dann wird sie als STRING angelegt).

## DIM ist idempotent für gleichen Typ

Innerhalb desselben Scopes ist `DIM x AS INTEGER` mehrfach erlaubt, solange der Typ gleich bleibt. Bei Typkonflikt: Fehler.

```basic
DIM x AS INTEGER
DIM x AS INTEGER         ' OK - idempotent, Wert bleibt
DIM x AS STRING          ' Fehler: Typkonflikt
```

Das ist der Grund, warum `DIM` in Schleifen-Bodies sicher ist — bei jeder Iteration wird neu deklariert, aber ohne den Wert zu verlieren.

## CONST sperrt den Slot

```basic
CONST PI AS FLOAT = 3.14159
PI = 3.0                 ' Fehler: PI ist CONST
DIM PI AS FLOAT          ' Fehler: kann nicht erneut deklariert werden
```

Auch `FOR i = 1 TO 10` schmeißt, wenn `i` als CONST deklariert wurde.

## Keine Closures, keine verschachtelten SUBs

GameBasic erlaubt keine SUB/FUNCTION innerhalb einer SUB/FUNCTION. Jede Routine sieht ihren eigenen lokalen Scope + (bei Methoden) Felder + Globals — fertig. Es gibt keinen "äußeren" lokalen Scope den man closen könnte.

## Quick Reference

| Situation | Was passiert |
|---|---|
| `x = 5` in SUB ohne lokales `DIM x` | schreibt das globale `x` (oder Feld bei Methode) |
| `DIM x AS INT` in SUB | erzeugt lokale `x`, schattet ab |
| `DIM x` in IF/WHILE/FOR | landet im umschließenden SUB/Top-Level-Scope |
| FOR `i` ohne vorheriges DIM | wird automatisch deklariert (INTEGER, oder FLOAT bei FLOAT-Bounds) |
| FOR `i` nach Schleife lesen | erlaubt, hat den Endwert |
| `CATCH e` | `e` bleibt im umschließenden Scope erreichbar |
| Parameter `p` | by-value für Skalare, by-reference für Arrays/Objekte |
| `DIM x AS INTEGER` zweimal mit gleichem Typ | idempotent (kein Fehler, kein Wert-Reset) |
| `DIM x AS INTEGER` dann `DIM x AS STRING` | Typkonflikt-Fehler |
