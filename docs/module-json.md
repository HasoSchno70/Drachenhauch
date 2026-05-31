# Modul `json`

JSON parsen, lesen, ausgeben. Lädt Datei oder String, lässt dich über Pfad-Notation auf Felder zugreifen.

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

Siehe [examples/24_json.gb](../examples/24_json.gb).
