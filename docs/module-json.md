# Modul `json`

JSON parsen, lesen, **bauen** und ausgeben. Lädt Datei oder String, lässt dich über Pfad-Notation auf Felder zugreifen — und seit 2026-08 auch welche setzen.

```basic
IMPORT "json"
```

## Übersicht

| Funktion | Rückgabe |
|---|---|
| `JSON_PARSE(s$)` | JSON_HANDLE |
| `JSON_LOAD(path$)` | JSON_HANDLE |
| `JSON_STRINGIFY(h)` | STRING (kompakt) |
| `JSON_PRETTY(h)` | STRING (eingerückt) |
| `JSON_GET_STRING(h, path$)` | STRING |
| `JSON_GET_INT(h, path$)` | INTEGER |
| `JSON_GET_FLOAT(h, path$)` | FLOAT |
| `JSON_GET_BOOL(h, path$)` | BOOLEAN |
| `JSON_HAS(h, path$)` | BOOLEAN |
| `JSON_LEN(h, path$)` | INTEGER |
| `JSON_TYPE(h, path$)` | STRING |
| `JSON_KEYS(h, path$)` | ARRAY OF STRING |

**Schreiben:**

| Funktion | Wirkung |
|---|---|
| `JSON_NEW_OBJECT()` | leeres `{}` → JSON_HANDLE |
| `JSON_NEW_ARRAY()` | leeres `[]` → JSON_HANDLE |
| `JSON_SET_STRING(h, pfad$, wert$)` | Feld setzen (anlegen oder ersetzen) |
| `JSON_SET_INT(h, pfad$, wert)` | " |
| `JSON_SET_FLOAT(h, pfad$, wert)` | " |
| `JSON_SET_BOOL(h, pfad$, wert)` | " |
| `JSON_SET_NULL(h, pfad$)` | " |
| `JSON_SET_JSON(h, pfad$, andere)` | ganzes Dokument einhängen (**Kopie**) |
| `JSON_APPEND_STRING/INT/FLOAT/BOOL/JSON(h, pfad$, wert)` | an das Array am Pfad anhängen |
| `JSON_REMOVE(h, pfad$)` | entfernen → BOOLEAN (war etwas da?) |

## Pfad-Notation

Pfade nutzen Punkt-Notation: `"user.name"` greift in einem Objekt auf Feld `name` im Sub-Objekt `user` zu. Array-Indizes werden auch mit Punkt geschrieben: `"items.0.title"` adressiert das `title`-Feld des ersten Array-Eintrags.

Leerer Pfad `""` zeigt auf das Wurzel-Element.

## Beispiel

```basic
IMPORT "json"

DIM doc AS STRING
doc = "{""user"":{""name"":""Anna"",""alter"":30,""aktiv"":true},""hobbies"":[""Lesen"",""Code""],""score"":98.5}"

DIM j AS JSON_HANDLE
j = JSON_PARSE(doc)

PRINT JSON_GET_STRING(j, "user.name")       ' "Anna"
PRINT JSON_GET_INT(j, "user.alter")         ' 30
PRINT JSON_GET_BOOL(j, "user.aktiv")        ' TRUE
PRINT JSON_GET_FLOAT(j, "score")            ' 98.5

' Array iterieren
DIM i AS INTEGER
FOR i = 0 TO JSON_LEN(j, "hobbies") - 1
    PRINT JSON_GET_STRING(j, "hobbies." + STR$(i))
NEXT

' Existenz prüfen
IF JSON_HAS(j, "user.email") THEN
    PRINT JSON_GET_STRING(j, "user.email")
ELSE
    PRINT "Keine Email"
END IF
```

## Datei lesen

```basic
IMPORT "json"

IF FILEEXISTS("settings.json") THEN
    DIM cfg AS JSON_HANDLE
    cfg = JSON_LOAD("settings.json")
    DIM max AS INTEGER
    max = JSON_GET_INT(cfg, "max_lives")
ELSE
    PRINT "settings.json nicht gefunden, nutze Defaults"
END IF
```

## Typen prüfen

`JSON_TYPE(h, path)` gibt einen STRING zurück: `"object"`, `"array"`, `"string"`, `"number"`, `"boolean"`, `"null"` oder `"missing"` (wenn der Pfad nicht existiert).

```basic
SELECT CASE JSON_TYPE(j, "user.alter")
    CASE "number"
        PRINT "Alter ist eine Zahl"
    CASE "string"
        PRINT "Alter ist ein String!?"
    CASE "missing"
        PRINT "Alter fehlt"
END SELECT
```

## JSON bauen

Bis 2026-08 ließ sich JSON nur lesen. Wer eines schreiben wollte, klebte
Zeichenketten zusammen — und brach am ersten Anführungszeichen in einem Namen.
Ein Handle ist deshalb **veränderbar**:

```basic
IMPORT "json"

DIM h AS JSON_HANDLE
h = JSON_NEW_OBJECT()
JSON_SET_STRING(h, "name", "Anna")
JSON_SET_INT(h, "alter", 30)
JSON_SET_BOOL(h, "aktiv", TRUE)

' Zwischenstufen entstehen von selbst:
JSON_SET_STRING(h, "adresse.ort", "Koeln")

PRINT JSON_STRINGIFY(h)
' {"name":"Anna","alter":30,"aktiv":true,"adresse":{"ort":"Koeln"}}
```

Listen wachsen mit `JSON_APPEND_*`. Der leere Pfad `""` meint dabei das
Dokument selbst:

```basic
DIM posten AS JSON_HANDLE
posten = JSON_NEW_ARRAY()
JSON_APPEND_STRING(posten, "", "Schraube")
JSON_APPEND_STRING(posten, "", "Mutter")
JSON_SET_JSON(h, "posten", posten)

' danach direkt am Ziel weiterfuellen:
JSON_APPEND_STRING(h, "posten", "Unterlegscheibe")
```

Und ein Objekt lässt sich jetzt auch durchlaufen — `JSON_LEN` lieferte für ein
Objekt schon immer die Anzahl seiner Schlüssel, an die Schlüssel selbst kam man
nicht heran:

```basic
DIM schluessel AS ARRAY OF STRING
DIM i AS INTEGER
schluessel = JSON_KEYS(h, "adresse")
FOR i = 0 TO LEN(schluessel) - 1
    PRINT schluessel[i], JSON_GET_STRING(h, "adresse." + schluessel[i])
NEXT
```

### Sechs Regeln, die man einmal gelesen haben sollte

1. **Ein Handle ist eine Referenz**, wie MAP und ARRAY. `b = a` legt keine
   Kopie an — wer `b` ändert, ändert `a`.
2. **`JSON_SET_JSON` hängt eine KOPIE ein.** Ein JSON-Baum kann sich keinen
   Teilbaum mit einem anderen teilen; spätere Änderungen an der Quelle
   schlagen nicht durch.
3. **Fehlende Zwischenstufen entstehen als Objekt.** Genau deshalb ist
   `JSON_SET_STRING(h, "kunde.adresse.ort", "Koeln")` ein Aufruf und nicht drei.
4. **Ein Zahl-Segment legt nichts an.** `"posten.0"` auf einem frischen
   Dokument könnte ein Array meinen oder ein Objekt mit dem Schlüssel `"0"` —
   beides ist gültiges JSON, und die falsche Wahl fällt erst dem Empfänger auf.
   Statt zu raten sagt die Meldung, wie ein Array entsteht. Arrays legt man
   also mit `JSON_NEW_ARRAY` an und füllt sie mit `JSON_APPEND_*`.
5. **Der leere Pfad `""` meint beim Schreiben NICHT die Wurzel.** Beim Lesen
   tut er das; beim Setzen hieße er „das ganze Dokument wegwerfen", und eine
   versehentlich leere Variable darf das nicht — `JSON_SET_*` lehnt ihn ab.
   Bei `JSON_APPEND_*` ist er erlaubt (dort geht nichts verloren) und meint das
   Dokument selbst.
6. **Die Reihenfolge der Schlüssel bleibt die Einfüge-Reihenfolge**, auch nach
   `JSON_REMOVE`. Für JSON ist sie bedeutungslos, aber wer einen Rumpf
   signiert oder zwei Ausgaben vergleicht, sieht den Unterschied.

**Ein Punkt im Schlüsselnamen ist nicht adressierbar** — er trennt die
Pfad-Segmente. Das gilt beim Lesen wie beim Schreiben.

## Roundtrip

```basic
PRINT JSON_STRINGIFY(j)              ' kompakt: {"user":{"name":"Anna",...}}
PRINT JSON_PRETTY(j)                 ' formatiert mit Einrückung
```

## Fehlerbehandlung

Strikte Type-Getter werfen, wenn der Wert nicht zum gewünschten Typ passt — fangen mit `TRY/CATCH`:

```basic
TRY
    DIM x AS INTEGER
    x = JSON_GET_INT(j, "user.name")     ' name ist STRING -> Fehler
CATCH e
    PRINT "Fehler: ", e
END TRY
```

`JSON_GET_INT` akzeptiert auch ganzzahlige Floats (`5.0` → `5`), aber nicht `3.14` → das wirft.

`JSON_PARSE` und `JSON_LOAD` werfen bei ungültigem JSON (unbalancierte Klammern, Trailing Komma, …).

## Komplettes Beispiel

Lesen: [examples/24_json.dh](../examples/24_json.dh).
Bauen (ein REST-Rumpf und eine Konfigurationsdatei):
[examples/171_json_bauen.dh](../examples/171_json_bauen.dh).
