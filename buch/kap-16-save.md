# Kapitel 16 — Save: Highscore, der bleibt

Bisher ist Star Pilot **vergesslich**. Jedes Mal wenn du das Programm beendest und neu startest, ist der Highscore wieder bei null. Das ist okay für einen Demo, aber unbefriedigend für ein echtes Spiel — *die Hetze* nach einem höheren Endstand lebt davon, dass der vorherige Bestwert sichtbar bleibt.

In diesem Kapitel ändern wir das mit dem `save`-Modul. Eine winzige JSON-Datei auf der Festplatte hält den Highscore zwischen den Sitzungen. Beim Programmstart wird sie geladen, beim neuen Bestwert geschrieben.

## Lernziele

Nach diesem Kapitel:

- aktivierst du das `save`-Modul mit `IMPORT "save"`
- nutzt du `SAVE_NEW`, `SAVE_LOAD`, `SAVE_LOAD_OR_NEW`, `SAVE_WRITE` korrekt
- unterscheidest du strikte Getter (`SAVE_GET_INT`) und tolerante (`SAVE_GET_INT_OR`)
- baust du eine Top-5-Tabelle mit parallelen Save-Keys
- hast in Star Pilot einen persistenten Highscore + „NEW HIGHSCORE!"-Anzeige

## Warum nicht direkt Datei-I/O?

GameBasic hat auch Low-Level-Datei-Funktionen (`OPEN`, `WRITE`, `READ`, `CLOSE`). Du könntest deinen Highscore direkt als Text in eine Datei schreiben:

```basic
DIM f AS FILE
f = OPEN("highscore.txt", "w")
WRITE(f, STR$(score))
CLOSE(f)
```

Funktioniert. Aber drei Probleme stoßen dir bei einem echten Spiel auf:

1. **Format**: Wenn du später zwei Werte speichern willst (Score + Spielername), wird die Parsing-Logik schnell hässlich — Zeilenumbrüche, Trennzeichen, Escaping.
2. **Robustheit**: Wenn die Datei kaputt ist (Hardware-Fehler, abgebrochener Schreibvorgang), liest dein Code Müll. Du brauchst überall Validierung.
3. **Versionierung**: Wenn du später ein zweites Feld dazupackst, brechen alte Save-Dateien.

Das `save`-Modul löst das mit JSON als Backend, typsicheren Settern/Gettern und einem Versionsfeld.

## Schritt 1: Lifecycle

Ein Save-Lebenszyklus hat vier Phasen:

| Funktion | Wann |
|---|---|
| `SAVE_NEW()` | Komplett leerer Save (für initial-state) |
| `SAVE_LOAD(path$)` | Existierende Datei laden — wirft, wenn die Datei fehlt |
| `SAVE_LOAD_OR_NEW(path$)` | **Empfohlen für Programmstart**: leerer Save wenn Datei fehlt |
| `SAVE_WRITE(s, path$)` | Save zur Datei schreiben |

Der Standard-Flow für ein Spiel:

```basic
DIM s AS SAVE_HANDLE
s = SAVE_LOAD_OR_NEW("starpilot.save")    ' beim Programmstart

' ... irgendwann waehrend des Spiels ...
SAVE_SET_INT(s, "highscore", 4200)
SAVE_WRITE(s, "starpilot.save")
```

`SAVE_LOAD_OR_NEW` ist der **richtige** Weg, weil's robust ist: bei der ersten Programmausführung gibt's keine Datei, der Spieler hat aber ein einsatzbereites Save-Objekt.

> **Wann `SAVE_LOAD` (mit Wurf) sinnvoll ist**: für *erwartete* Saves, etwa nach „Spielstand laden" im Menü. Da soll's einen Fehler geben, wenn die Datei fehlt — der User soll wissen, dass er noch nichts gespeichert hat.

## Schritt 2: Strikte vs. tolerante Getter

Pro Wert-Typ gibt's zwei Getter:

| Strikt | Tolerant |
|---|---|
| `SAVE_GET_INT(s, key$)` | `SAVE_GET_INT_OR(s, key$, default)` |
| `SAVE_GET_FLOAT(s, key$)` | `SAVE_GET_FLOAT_OR(s, key$, default)` |
| `SAVE_GET_STRING(s, key$)` | `SAVE_GET_STRING_OR(s, key$, default)` |
| `SAVE_GET_BOOL(s, key$)` | `SAVE_GET_BOOL_OR(s, key$, default)` |

**Strikt** wirft bei fehlendem Key oder Typ-Mismatch. Nutzt du, wenn du **sicher** bist, dass der Wert da sein muss — z.B. nach einem expliziten Save.

**Tolerant** liefert immer einen Wert. Bei fehlendem Key: Default. Bei Typ-Mismatch: ebenfalls Default (kein Crash, kein Cast). Nutzt du für **optionale** Werte — z.B. Highscore beim ersten Programmstart (existiert noch nicht).

```basic
DIM hi AS INTEGER
hi = SAVE_GET_INT_OR(s, "highscore", 0)    ' beim ersten Lauf: 0
```

Faustregel: bei „kann fehlen" → `_OR`. Bei „muss da sein" → strikt (der Crash hilft dir, Bugs früh zu erwischen).

## Schritt 3: Single-Highscore in der Konsole

Bevor wir das Spiel anpacken, ein Mini-Programm zum Verstehen:

```basic
IMPORT "save"

CONST SAVE_PATH AS STRING = "kap16_demo.save"

DIM s AS SAVE_HANDLE
s = SAVE_LOAD_OR_NEW(SAVE_PATH)

DIM hi AS INTEGER
hi = SAVE_GET_INT_OR(s, "highscore", 0)
PRINT f"Aktueller Highscore: {hi}"

SAVE_SET_INT(s, "highscore", hi + 1000)
SAVE_WRITE(s, SAVE_PATH)
PRINT f"Neuer Wert: {hi + 1000} - gespeichert."

DIM s2 AS SAVE_HANDLE
s2 = SAVE_LOAD(SAVE_PATH)
PRINT f"Frisch geladen: {SAVE_GET_INT(s2, 'highscore')}"

SAVE_DELETE_FILE(SAVE_PATH)
```

Output:

```
Aktueller Highscore: 0
Neuer Wert: 1000 - gespeichert.
Frisch geladen: 1000
Datei geloescht.
```

Drei Dinge:

1. Beim ersten Lauf: Datei nicht da → `_OR(0)` liefert 0.
2. Wir setzen `0 + 1000 = 1000`, schreiben.
3. Frischer `SAVE_LOAD` (strikt) liest den Wert.

`SAVE_DELETE_FILE` am Ende räumt die Datei weg — sonst würde der zweite Programmlauf den Wert „1000" als Anfangshighscore lesen. Praktisch fürs Debugging.

## Schritt 4: Save-Datei direkt anschauen

Eine der schönen Eigenschaften des `save`-Moduls: die Dateien sind **menschen-lesbares JSON**. Öffne `kap16_demo.save` (vor `SAVE_DELETE_FILE`) mit einem Texteditor:

```json
{
  "_version": 1,
  "data": {
    "highscore": 1000
  }
}
```

Drei Felder:
- **`_version`** — fürs Schema-Versioning (default 1)
- **`data`** — das Dict deiner Werte
- Innen: deine Keys mit ihren Werten

Vorteil: bei Problemen siehst du sofort, was los ist. „Save geht nicht?" — Datei aufmachen, schauen, fertig. Bei Binärformaten wäre das ein Albtraum.

## Schritt 5: Top-5-Highscore-Tabelle

Single-Highscore ist Anfänger-Niveau. Top-5 oder Top-10 wirken schon „richtig" wie ein klassisches Arcade-Spiel. Idee:

- 10 Save-Keys: `score_0`..`score_4` und `name_0`..`name_4`
- `score_0` ist der höchste, `score_4` der niedrigste in der Top 5
- Beim Hinzufügen: Position finden, alle darunter liegenden um 1 nach hinten schieben, neuen Eintrag einfügen

```basic
SUB AddScore(player AS STRING, score AS INTEGER)
    DIM pos AS INTEGER
    pos = -1
    DIM i AS INTEGER
    FOR i = 0 TO TABLE_LEN - 1
        IF score > SAVE_GET_INT_OR(s, f"score_{i}", 0) THEN
            pos = i
            BREAK
        END IF
    NEXT i

    IF pos = -1 THEN RETURN     ' Score nicht gut genug

    ' Eintraege nach hinten schieben (von hinten her, sonst ueberschreiben wir uns)
    FOR i = TABLE_LEN - 1 TO pos + 1 STEP -1
        SAVE_SET_INT(s,    f"score_{i}", SAVE_GET_INT_OR(s,    f"score_{i - 1}", 0))
        SAVE_SET_STRING(s, f"name_{i}",  SAVE_GET_STRING_OR(s, f"name_{i - 1}",  ""))
    NEXT i

    SAVE_SET_INT(s,    f"score_{pos}", score)
    SAVE_SET_STRING(s, f"name_{pos}",  player)
END SUB
```

> **Aha-Moment**: das **rückwärts-Schieben** (`STEP -1`) ist kein Zufall. Würden wir vorwärts gehen (`pos+1`, `pos+2`, ...), würden wir Werte überschreiben, bevor wir sie weiterschieben. Klassischer Array-Insert-Algorithmus.

Die `f"score_{i}"`-Konstruktion ist clever: pro Schleifen-Iteration ein anderer Key — dynamische Schlüssel-Erzeugung. Funktioniert weil `SAVE_SET_INT` einen `STRING` als Key nimmt, und f-Strings beliebige Strings produzieren.

## Schritt 6: Star Pilot mit persistentem Highscore

Im Spiel-Code (single-Highscore-Variante, nicht Top-5 — siehe Übung 1):

**Globale Variablen:**

```basic
DIM save_data AS SAVE_HANDLE
DIM highscore AS INTEGER
```

**Setup() lädt:**

```basic
save_data = SAVE_LOAD_OR_NEW(SAVE_PATH)
highscore = SAVE_GET_INT_OR(save_data, "highscore", 0)
```

**Beim Player-Tod prüfen:**

```basic
IF NOT player.alive THEN
    DIM is_new_hi AS BOOLEAN
    is_new_hi = FALSE
    IF score > highscore THEN
        highscore = score
        SaveHighscore()
        is_new_hi = TRUE
    END IF
    DIM final AS INTEGER
    final = score
    SCENE_SWITCH("gameover")
    SCENE_SET_INT("final_score", final)
    SCENE_SET_BOOL("new_highscore", is_new_hi)
    RETURN
END IF
```

**Save-Helper:**

```basic
SUB SaveHighscore()
    SAVE_SET_INT(save_data, "highscore", highscore)
    SAVE_WRITE(save_data, SAVE_PATH)
END SUB
```

**Im Menu-Draw:**

```basic
TEXT(WIDTH / 2 - 60, 100, f"Highscore: {highscore}", &HFFDC00)
```

**Im GameOver: feiern wenn neuer Highscore:**

```basic
IF SCENE_GET_BOOL_OR("new_highscore", FALSE) THEN
    TEXT(WIDTH / 2 - 60, ..., "NEUER HIGHSCORE!", PLAYER_C)
ELSE
    TEXT(WIDTH / 2 - 70, ..., f"Highscore: {highscore}", &HCCCCCC)
END IF
```

> **Reihenfolge bei SCENE_SET nach SWITCH**: `SCENE_SET_INT` schreibt auf die *aktuelle* Scene. Daher muss der `SWITCH` *vor* den `SET`-Aufrufen kommen — sonst landen die Werte in der `"playing"`-Scene und sind nach dem Switch weg. Das ist im Code oben so gemacht: erst `SCENE_SWITCH("gameover")`, dann `SCENE_SET_*(...)`.

## Schritt 7: Versionierung

Wenn dein Spiel länger lebt, ändert sich das Schema. Heute: nur `highscore`. Morgen vielleicht: `highscore`, `total_kills`, `prefer_keyboard_layout`. Wenn du beim nächsten Release einfach loslegst, bekommt der Spieler beim ersten Start einen halb-zugemüllten Save — alte Felder fehlen, neue sind nicht da.

Das **Versionsfeld** löst das. `_version` wird beim Schreiben mitgespeichert. Du kannst beim Laden prüfen:

```basic
DIM ver AS INTEGER
ver = SAVE_VERSION(save_data)

IF ver < 2 THEN
    ' Migration v1 -> v2: alte Felder umbenennen, neue Defaults setzen
    SAVE_SET_INT(save_data, "total_kills", 0)
    SAVE_SET_VERSION(save_data, 2)
    SAVE_WRITE(save_data, SAVE_PATH)
END IF
```

Für Star Pilot brauchen wir das nicht — wir bleiben bei `_version = 1`. Aber gut zu wissen, dass der Mechanismus da ist.

## Schritt 8: Toleranz beim Laden

Was passiert, wenn die Save-Datei **kaputt** ist (durch Crash, Festplatten-Fehler, manueller Mist im Texteditor)?

- **Datei fehlt**: `SAVE_LOAD_OR_NEW` → leerer Save (kein Drama)
- **Kaputtes JSON** (Syntax-Fehler): wirft `GBRuntimeError`
- **Top-Level kein Objekt** (z.B. `[1, 2, 3]`): wirft
- **Fehlende `_version`**: → wird als 1 interpretiert (rückwärts-tolerant)
- **Fehlendes `data`**: → leerer Save (kein Crash)
- **Falscher Typ** für einen Key: strikt wirft, `_OR` liefert Default

Die Faustregel: **strikt nur wenn du sicher bist; sonst `_OR` mit sinnvollem Default**. Dann übersteht dein Programm auch verkorkste Save-Dateien gracefully.

## Übungen

**1. Top-5 ins Spiel.** Übertrage die Logik aus `02_highscore_table.gb` in `main.gb`. Beim Player-Tod: `AddScore("PILOT", score)` aufrufen. Im Menu: die Top-5 anzeigen statt nur dem Single-Highscore. Hinweis: für den Player-Namen reicht erstmal hardcoded „PILOT" — Texteingabe lernen wir in Kap 17 mit dem UI-Modul.

**2. Statistik-Tracking.** Erweitere die Save-Datei um drei Felder: `total_kills` (Gesamtzahl getöteter Gegner über alle Spiele), `total_deaths`, `total_waves`. Beim Treffer eines Gegners: `total_kills += 1`. Im Menu: Statistik-Zeile anzeigen.

**3. Reset-Funktion.** Im Menu eine versteckte Taste (z.B. R für „Reset"), die `SAVE_DELETE_FILE(SAVE_PATH)` aufruft und `highscore = 0` setzt. Handlich beim Demonstrieren — der Buch-Leser kann seinen Highscore jederzeit zurücksetzen.

**4. Stretch — Migration üben.** Schreibe Code, der beim Programmstart prüft: wenn `_version < 2`, dann setze `total_kills = 0` (neuer Default), erhöhe Version auf 2, speichere. Der praktische Test: Datei manuell mit `_version: 1` und nur `highscore` editieren, Programm starten, schauen ob das `total_kills`-Feld dazu kommt.

## Zusammenfassung

Du hast in diesem Kapitel:

- das `save`-Modul mit `IMPORT "save"` aktiviert,
- den Lebenszyklus `SAVE_NEW` / `LOAD_OR_NEW` / `WRITE` verstanden,
- den Unterschied zwischen strikten und toleranten Gettern verinnerlicht,
- eine Top-5-Tabelle mit parallelen Keys gebaut (in der Konsole),
- den persistenten Highscore in Star Pilot integriert mit „NEUER HIGHSCORE!"-Feier,
- die Versionierung als Vorbereitung für zukünftige Schema-Änderungen kennengelernt,
- gesehen, wie das Modul mit kaputten / fehlenden Save-Dateien umgeht.

Im **nächsten Kapitel** kommt das letzte „Drumherum"-Thema: ein **Pause-Modus** mit dem `ui`-Modul, plus ein einfaches Optionen-Menü mit Lautstärke-Slider. Das war's dann mit Boilerplate — danach gibt's Kap 18 für den Boss-Fight und JSON-Wellen.

## Code-Stand am Ende des Kapitels

- [`code/kap-16/01_save_basics.gb`](code/kap-16/01_save_basics.gb) — eine Zahl speichern und laden, Konsolen-Demo
- [`code/kap-16/02_highscore_table.gb`](code/kap-16/02_highscore_table.gb) — Top-5-Logik mit dynamischen Keys
- [`code/kap-16/main.gb`](code/kap-16/main.gb) — Star Pilot mit persistentem Highscore und „NEW HIGHSCORE!"-Anzeige
