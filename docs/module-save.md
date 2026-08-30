# Modul `save`

High-Level Save/Load mit JSON-Backend. Typsichere Setter/Getter, Versionsfeld, tolerantes Laden alter Save-Dateien.

```basic
IMPORT "save"
```

## Übersicht

| Funktion | Rückgabe / Wirkung |
|---|---|
| `SAVE_NEW()` | SAVE_HANDLE — leerer Save |
| `SAVE_LOAD(path$)` | SAVE_HANDLE — wirft wenn Datei fehlt |
| `SAVE_LOAD_OR_NEW(path$)` | SAVE_HANDLE — leer wenn Datei fehlt |
| `SAVE_EXISTS(path$)` | BOOLEAN — gibt es die Datei? |
| `SAVE_WRITE(s, path$)` | nach Datei schreiben |
| `SAVE_DELETE_FILE(path$)` | idempotent |
| `SAVE_VERSION(s)` / `SAVE_SET_VERSION(s, n)` | Versionsfeld lesen/schreiben |
| `SAVE_SET_INT/FLOAT/STRING/BOOL(s, key$, value)` | Setter — Wert unter einem Schluessel ablegen |
| `SAVE_GET_INT/FLOAT/STRING/BOOL(s, key$)` | strikt — fehlender Schluessel ist ein Fehler |
| `SAVE_GET_INT_OR/FLOAT_OR/STRING_OR/BOOL_OR(s, key$, default)` | mit Fallback — liefert `default`, wenn der Schluessel fehlt oder den falschen Typ hat |
| `SAVE_HAS(s, key$)` | BOOLEAN — ist der Schluessel belegt? |
| `SAVE_DELETE(s, key$)` | idempotent |
| `SAVE_CLEAR(s)` | alle Keys weg, Version bleibt |
| `SAVE_KEYS(s)` | STRING — sortierte Liste |

## Konzept

Ein Save-File ist eine flache Map von Key → Value. Werte sind primitive Typen (Integer, Float, String, Bool). Optional ein Versionsfeld zur Migration.

```basic
IMPORT "save"

DIM s AS SAVE_HANDLE
s = SAVE_LOAD_OR_NEW("highscore.save")

DIM hi AS INTEGER
hi = SAVE_GET_INT_OR(s, "highscore", 0)

' Spielerwert mit aktueller Runde vergleichen ...
IF score > hi THEN
    SAVE_SET_INT(s, "highscore", score)
    SAVE_WRITE(s, "highscore.save")
END IF
```

## Lifecycle

Drei Wege, einen Save-Handle zu bekommen:

```basic
DIM s AS SAVE_HANDLE

' Leer starten
s = SAVE_NEW()

' Datei muss existieren - sonst Fehler
s = SAVE_LOAD("save.dat")

' Datei laden, oder leer wenn nicht vorhanden (typischer Game-Start)
s = SAVE_LOAD_OR_NEW("save.dat")
```

`SAVE_LOAD_OR_NEW` ist der "richtige" Weg für den Programmstart: existiert die Datei, wird sie geladen; existiert sie nicht, kriegst du einen leeren Save. Bei kaputtem JSON wirft es trotzdem — sonst würde der nächste `SAVE_WRITE` den verkorksten Save überschreiben.

## Strikte vs. tolerante Getter

**Strikt** (`SAVE_GET_*`): wirft wenn Key fehlt oder Typ nicht passt. Gut wenn du sicher bist, dass der Wert da sein muss:

```basic
DIM name AS STRING
name = SAVE_GET_STRING(s, "player_name")     ' wirft wenn nicht da
```

**Mit Default** (`SAVE_GET_*_OR`): liefert immer einen Wert. Bei Typ-Mismatch wird der Default zurückgegeben (kein Cast):

```basic
DIM hi AS INTEGER
hi = SAVE_GET_INT_OR(s, "highscore", 0)      ' Default 0 wenn fehlend oder kein Int
```

Tipp: nutze `_OR` für alles "Optional"-artige (Highscore, gewählte Schwierigkeit), strikte Getter wenn du beim Laden ohnehin auf konsistenten State angewiesen bist.

## Datei-Format

Save-Dateien sind menschen-lesbares, eingerücktes JSON:

```json
{
  "_version": 1,
  "data": {
    "highscore": 4200,
    "player_name": "Anna",
    "completed_tutorial": true,
    "music_volume": 0.7
  }
}
```

Beim Debugging kannst du sie also ganz normal mit dem Texteditor öffnen.

## Versionierung

Wenn sich dein Save-Schema ändert, nutze das Versionsfeld:

```basic
DIM s AS SAVE_HANDLE
s = SAVE_LOAD_OR_NEW("save.dat")

IF SAVE_VERSION(s) < 2 THEN
    ' Migration v1 -> v2: alte Felder umbenennen, neue Defaults setzen ...
    DIM oldname AS STRING
    oldname = SAVE_GET_STRING_OR(s, "name", "")
    SAVE_SET_STRING(s, "player_name", oldname)
    SAVE_DELETE(s, "name")
    SAVE_SET_VERSION(s, 2)
    SAVE_WRITE(s, "save.dat")
END IF
```

Default-Version ist `1`. Save-Files ohne `_version`-Feld werden als `1` interpretiert (rückwärts-tolerant).

## Toleranz beim Laden

Damit ein altes Save-File nicht den ganzen Spielstart torpediert:

- **Fehlende `_version`**: → 1
- **Fehlendes `data`**: → leer (keine Keys)
- **Top-Level kein Objekt** (z.B. JSON-Array): → wirft (irreparabel)
- **Keys mit unerwarteten Typen**: strikte Getter werfen, `_OR`-Getter liefern Default

JSON unterscheidet keinen `1` von `1.0` zuverlässig — `SAVE_GET_INT` akzeptiert daher auch ganzzahlige Floats (`5.0` → `5`), wirft aber bei `5.5`.

## Externer Typ

`SAVE_HANDLE` — opake Hülle um Versionsfeld + Daten-Map.

## Siehe auch

- [`scene`](module-scene.md) — Pro-Scene-Daten haben dieselbe API-Form (typsichere Setter/Getter mit `_OR`-Variante), aber leben nur für die Lebensdauer der Scene
- [`json`](module-json.md) — Low-Level JSON wenn du komplexere verschachtelte Strukturen brauchst
- Vollständiges Beispiel: [`examples/49_pong_scene.dh`](../examples/49_pong_scene.dh) — Pong mit `pong.save` für persistierten Highscore
