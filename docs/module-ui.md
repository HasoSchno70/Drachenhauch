# Modul `ui`

Immediate-Mode-UI: Label, Button, Checkbox, Slider, Progressbar, Panel, Textfeld, Radio-Group. Komponenten werden jeden Frame neu aufgerufen — kein separates Setup, keine Komponenten-Objekte. State (Checkbox-Zustand, Slider-Wert, Textfeld-Inhalt, Radio-Auswahl) wird intern über String-IDs verwaltet.

```basic
IMPORT "ui"
```

## Übersicht

| Funktion | Rückgabe | Zweck |
|---|---|---|
| `UI_LABEL(x, y, text$[, color])` | — | Text zeichnen |
| `UI_BUTTON(id$, x, y, w, h, text$[, bg, fg])` | BOOLEAN | TRUE wenn in diesem Frame geklickt |
| `UI_CHECKBOX(id$, x, y, label$[, default])` | BOOLEAN | aktueller Toggle-Zustand |
| `UI_SLIDER(id$, x, y, w, min, max[, default])` | FLOAT | aktueller Wert |
| `UI_PROGRESS(x, y, w, h, value, max[, fg, bg])` | — | read-only Fortschrittsbalken |
| `UI_PANEL(x, y, w, h[, title$[, bg]])` | — | Container mit optionalem Titel |
| `UI_TEXTFIELD(id$, x, y, w, h[, placeholder$])` | STRING | aktueller Eingabe-Text |
| `UI_TEXTFIELD_SET(id$, value$)` | — | Wert programmatisch setzen |
| `UI_RADIO(id$, x, y, options[, default_idx])` | INTEGER | gewählter Index |
| `UI_TABLE(id$, x, y, w, h, headers, cells[, cell_colors[, col_widths[, cell_bg_colors]]])` | INTEGER | Index der geklickten Zeile (-1 wenn keine) |
| `UI_TABLE_SELECTED(id$)` | INTEGER | persistent selektierte Zeile (-1 wenn keine) |
| `UI_TABLE_SET_SELECTED(id$, row)` | — | Selektion programmatisch setzen (-1 = keine) |
| `UI_TABLE_HEADER_CLICK(id$)` | INTEGER | in diesem Frame geklickte Header-Spalte (-1) — Sortier-Hook |
| `UI_WINDOW_BEGIN(id$, titel$, x, y, w, h)` | BOOLEAN | verschiebbares Fenster; FALSE wenn eingeklappt |
| `UI_WINDOW_END()` | — | Fenster abschließen (immer paaren) |
| `UI_END_FRAME()` | — | **Pflicht** am Ende jedes Frames vor `FLIP()` |
| `UI_RESET()` | — | allen UI-State löschen + Theme/Metriken zurücksetzen |
| `UI_THEME_SET(key$, farbe)` / `UI_THEME_GET(key$)` | — / INT | einzelne Theme-Farbe setzen/lesen |
| `UI_THEME_PRESET(name$)` | — | Farbschema: dark/light/retro/contrast |
| `UI_METRIC_SET(key$, wert)` / `UI_METRIC_GET(key$)` | — / INT | Layout-Größe setzen/lesen |

## Konzept: Immediate-Mode

Komponenten werden jeden Frame neu aufgerufen. Das ist BASIC-typisch und sehr lesbar — kein "Komponenten registrieren / Events binden / Komponenten zeichnen"-Pattern.

```basic
IMPORT "ui"

SCREEN(320, 240, "UI-Demo", 2)

WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 25, 40))

    UI_LABEL(10, 10, "Mein Spiel", RGB(255, 220, 80))

    IF UI_BUTTON("start", 10, 40, 100, 30, "Start") THEN
        PRINT "Spiel startet!"
    END IF

    DIM sound AS BOOLEAN
    sound = UI_CHECKBOX("snd", 10, 80, "Sound an", TRUE)

    DIM lautstaerke AS FLOAT
    lautstaerke = UI_SLIDER("vol", 10, 110, 200, 0.0, 1.0, 0.7)

    UI_END_FRAME()              ' wichtig!
    FLIP()
    SLEEP(16)
WEND
```

## ID-System

Jede stateful Komponente (`UI_BUTTON`, `UI_CHECKBOX`, `UI_SLIDER`) braucht eine **eindeutige String-ID**. Das UI-Modul merkt sich pro ID intern den Zustand zwischen Frames.

```basic
UI_CHECKBOX("musik", 10, 50, "Musik")     ' ID = "musik"
UI_CHECKBOX("sound", 10, 70, "Sound")     ' ID = "sound" - andere Checkbox
```

Wenn du dieselbe ID zweimal pro Frame verwendest, gibt es **eine** Komponente mit dem zuletzt-gewonnen-Wert (sinnlos). IDs müssen einmalig sein.

`UI_LABEL` braucht keine ID, weil es zustandslos ist.

## UI_LABEL

```basic
UI_LABEL(x, y, text$[, color])
```

Zeichnet Text bei `(x, y)`. Default-Farbe ist Weiß. Funktional dasselbe wie `TEXT(x, y, text, color)`, aber im UI-Stil.

```basic
UI_LABEL(10, 10, "Score: " + STR$(score))
UI_LABEL(10, 30, "HP: " + STR$(hp), RGB(255, 100, 100))
```

## UI_BUTTON

```basic
UI_BUTTON(id$, x, y, w, h, text$[, bg, fg]) -> BOOLEAN
```

Zeichnet einen klickbaren Knopf. Gibt **TRUE** zurück in dem Frame, in dem der User die Maus über dem Knopf **losgelassen** hat (nachdem er ihn vorher gedrückt hatte). Press-and-drag-away-and-release zählt nicht — typisches OK-Knopf-Verhalten.

```basic
IF UI_BUTTON("ok", 10, 100, 80, 28, "OK") THEN
    PRINT "OK geklickt"
END IF

' Mit eigenen Farben:
IF UI_BUTTON("danger", 100, 100, 80, 28, "Loeschen", RGB(160, 40, 40)) THEN
    daten_loeschen()
END IF
```

**Visuell:**
- Normal: bg-Farbe
- Hover (Maus drüber): aufgehellt
- Press (Maus drüber + gedrückt): abgedunkelt

## UI_CHECKBOX

```basic
UI_CHECKBOX(id$, x, y, label$[, default]) -> BOOLEAN
```

Toggle-Checkbox mit Label rechts daneben. Der `default`-Wert wird **nur beim ersten Aufruf** dieser ID gesetzt — danach bleibt der vom User getoggelte State erhalten.

```basic
DIM sound AS BOOLEAN
sound = UI_CHECKBOX("snd", 10, 50, "Sound", TRUE)        ' Default beim 1. Frame: TRUE

' Pro Frame: aktueller Wert
IF sound THEN
    musik_starten()
END IF
```

Klick auf die Checkbox toggelt sie (Press, nicht Release — kein Drag-Trick wie beim Button).

## UI_SLIDER

```basic
UI_SLIDER(id$, x, y, w, min, max[, default]) -> FLOAT
```

Horizontaler Wert-Slider. `min` und `max` definieren den Wertebereich (FLOAT). Default landet im Bereich (wird geclamped).

```basic
DIM lautstaerke AS FLOAT
lautstaerke = UI_SLIDER("vol", 10, 100, 200, 0.0, 1.0, 0.7)

DIM bg_helligkeit AS INTEGER
bg_helligkeit = INT(UI_SLIDER("bg", 10, 130, 200, 0, 100, 50))
```

Der User klickt/zieht im Slider-Bereich. Der Wert ändert sich solange die Maus gedrückt ist UND über dem Slider steht.

**Constraints:** `max > min` (sonst Fehler beim Aufruf).

## UI_PROGRESS

```basic
UI_PROGRESS(x, y, w, h, value, max[, fg, bg])
```

Read-only Fortschrittsbalken — keine ID, kein State. Gut für HP-Bars, Loading-Anzeigen, XP-Fortschritt.

```basic
UI_PROGRESS(10, 50, 150, 12, hp, 100, RGB(220, 60, 60), RGB(60, 30, 30))
```

`value` wird auf `[0, max]` geclamped. `max` muss `> 0` sein (sonst Fehler). Default-Farben sind grün-auf-dunkelgrau.

## UI_PANEL

```basic
UI_PANEL(x, y, w, h[, title$[, bg]])
```

Visueller Container mit Rahmen. Wenn `title` gesetzt ist, kommt eine 18 px hohe Titel-Bar oben drauf. Komplett zustandslos — das Spiel zeichnet seine Komponenten selbst über den Panel.

```basic
UI_PANEL(10, 10, 200, 130, "Held")
UI_LABEL(20, 35, "HP", RGB(220, 100, 100))
UI_PROGRESS(50, 36, 150, 12, hp, 100)
```

## UI_TEXTFIELD

```basic
UI_TEXTFIELD(id$, x, y, w, h[, placeholder$]) -> STRING
```

Text-Input. **Klick** auf das Feld setzt den Fokus; **getippte Zeichen** werden direkt angehängt; **Backspace** löscht das letzte Zeichen; **Klick außerhalb** entfernt den Fokus. Der zurückgegebene STRING ist der aktuelle Inhalt.

```basic
DIM name AS STRING
name = UI_TEXTFIELD("name", 10, 50, 200, 26, "Hier tippen ...")
UI_LABEL(10, 90, "Hallo, " + name + "!")
```

`placeholder$` wird in gedämpfter Farbe angezeigt solange das Feld leer ist UND nicht den Fokus hat.

Ein blinkender Cursor erscheint nur am fokussierten Feld. Layout/Shift/Dead-Keys werden vom OS aufgelöst — Umlaute (`ä`, `ü`, …) und Sonderzeichen funktionieren wie in jedem normalen Eingabefeld.

```basic
' Wert vorbelegen (z.B. beim Bearbeiten eines Eintrags):
UI_TEXTFIELD_SET("name", "Anna")
```

Tab/Enter werden vom Feld nicht behandelt — `KEYPRESSED(13)` kannst du selbst pollen wenn du Enter als „Submit" willst.

## UI_RADIO

```basic
UI_RADIO(id$, x, y, options[, default_idx]) -> INTEGER
```

Vertikale Radio-Gruppe — genau eine Option ausgewählt. `options` muss ein `ARRAY OF STRING` sein. Klick auf eine Zeile wählt sie. Liefert den Index (0-basiert) der gewählten Option, oder `-1` bei leerem Array.

```basic
DIM diff_options AS ARRAY OF STRING
diff_options = SPLIT$("Leicht|Mittel|Schwer", "|")

DIM diff AS INTEGER
diff = UI_RADIO("diff", 10, 50, diff_options, 1)    ' Default: "Mittel"

UI_LABEL(10, 130, "Gewaehlt: " + diff_options[diff])
```

Reihenhöhe ist 18 px, Klickfläche umfasst die ganze Reihe (200 px breit) damit auch lange Labels klickbar sind.

## UI_TABLE

```basic
UI_TABLE(id$, x, y, w, h, headers, cells[, cell_colors[, col_widths[, cell_bg_colors]]]) -> INTEGER
```

Tabelle mit fixiertem Header, scrollbarem Body und optionaler Per-Zelle-Farbgebung (Vorder- und Hintergrund). Unterstützt vertikales Mausrad-Scrolling und drag-bare Scrollbalken in beiden Achsen.

**Parameter:**

| Param | Typ | Bemerkung |
|---|---|---|
| `headers` | `ARRAY OF STRING` | Spaltentitel; `LEN(headers)` definiert die Spaltenanzahl |
| `cells` | 2D `ARRAY OF STRING` `[rows, cols]` | Zell-Inhalte; Spaltenzahl muss zu `headers` passen |
| `cell_colors` | 2D `ARRAY OF INTEGER` `[rows, cols]` *optional* | Text-RGB pro Zelle; default weiß |
| `col_widths` | `ARRAY OF INTEGER` *optional* | Pixelbreite pro Spalte; default = gleichmäßig verteilt |
| `cell_bg_colors` | 2D `ARRAY OF INTEGER` `[rows, cols]` *optional* | Hintergrund-RGB pro Zelle. **Wert -1 = kein Hintergrund** zeichnen (Standard-Verhalten ohne Background-Override) |

**Rückgabe:** Index der Zeile, die in **diesem Frame** geklickt wurde (Press + Release auf derselben Zeile, wie `UI_BUTTON`-Logik). `-1` wenn keine Zeile geklickt.

**Verhalten:**
- Header-Zeile (22 px) ist fixiert oben — scrollt nicht mit
- Body scrollt je nach Inhalt vertikal und/oder horizontal
- Scrollbar erscheint automatisch wenn Inhalt größer als sichtbar
- Mausrad scrollt vertikal (immer)
- Scrollbar-Drag funktioniert auf beiden Achsen
- Hover-Highlight auf der Zeile unter dem Mauszeiger
- Zell-Text wird automatisch auf die Spaltenbreite zugeschnitten (sauberes Pixel-Clipping, kein „…")

**Beispiel — Highscore-Liste mit farbigen HP-Werten:**

```basic
IMPORT "ui"
SCREEN(560, 380, "Highscore", 2)

CONST ROWS AS INTEGER = 10
CONST COLS AS INTEGER = 3

DIM headers AS ARRAY OF STRING
headers = SPLIT$("Name|HP|Punkte", "|")

DIM cells[ROWS, COLS] AS STRING
DIM colors[ROWS, COLS] AS INTEGER
DIM widths[COLS] AS INTEGER
widths[0] = 140
widths[1] = 80
widths[2] = 100

' Daten + Farben aufbauen
DIM r AS INTEGER
FOR r = 0 TO ROWS - 1
    cells[r, 0] = "Spieler_" + STR$(r + 1)
    DIM hp AS INTEGER
    hp = 30 + (r * 17) MOD 70
    cells[r, 1] = STR$(hp) + "/100"
    cells[r, 2] = STR$(((r + 1) * 313) MOD 9999)

    colors[r, 0] = RGB(220, 220, 230)
    IF hp >= 70 THEN colors[r, 1] = RGB(100, 220, 100)
    IF hp >= 40 AND hp < 70 THEN colors[r, 1] = RGB(255, 220, 80)
    IF hp < 40 THEN colors[r, 1] = RGB(240, 80, 80)
    colors[r, 2] = RGB(220, 220, 230)
NEXT

DIM selected AS INTEGER
selected = -1

WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(27) THEN BREAK
    CLS(RGB(15, 18, 32))

    DIM clicked AS INTEGER
    clicked = UI_TABLE("scores", 10, 10, 340, 300, headers, cells, colors, widths)
    IF clicked >= 0 THEN selected = clicked

    IF selected >= 0 THEN
        UI_LABEL(10, 320, "Ausgewaehlt: " + cells[selected, 0])
    END IF

    UI_END_FRAME()
    FLIP()
    SLEEP(16)
WEND
```

**Tipps:**
- **Per-Zelle-Hintergrund** ist ideal für Statistik-Ampeln (HP grün/gelb/rot mit volltöniger Bg-Farbe), Zebra-Streifen (gerade Zeilen `RGB(30, 34, 52)`, ungerade `-1`) oder Heatmaps. Auf der hovered-Zeile überlagert das Hover-Highlight die Cell-Bgs — der User sieht trotzdem klar wo der Mauszeiger ist.
- Für **dynamische Updates** (Liste wächst): die Arrays mit Maximalgröße deklarieren und nicht-existente Zeilen mit Leerstrings füllen.
- **Klick auf Scrollbalken** scrollt nur — die Zeile darunter wird nicht versehentlich „geklickt" weil die Scrollbar als Klickziel die Klick-Erkennung blockiert.
- **Komplettes Beispiel** mit Per-Zelle-Farben (Gold/Silber/Bronze, Klassen-Farben, HP-Ampel) und Detail-Panel: [examples/43_ui_table.gb](../examples/43_ui_table.gb).

### Persistente Selektion

```basic
UI_TABLE_SELECTED(id$)         -> INTEGER   ' selektierte Zeile, -1 wenn keine
UI_TABLE_SET_SELECTED(id$, row)             ' programmatisch setzen, -1 = keine
```

`UI_TABLE` setzt die Selektion automatisch auf die Zeile, die in einem Frame geklickt wird — anders als der Rückgabewert (nur im Klick-Frame `>= 0`) **bleibt** sie über Frames erhalten und wird als zweites Highlight (unter dem Hover) gezeichnet. So muss das Spiel die Auswahl nicht mehr selbst in einer Variable mitführen:

```basic
UI_TABLE("scores", 10, 10, 340, 300, headers, cells)
DIM sel AS INTEGER
sel = UI_TABLE_SELECTED("scores")
IF sel >= 0 THEN UI_LABEL(10, 320, "Ausgewaehlt: " + cells[sel, 0])
```

Schrumpfen die Daten (weniger Zeilen als der selektierte Index), fällt die Selektion automatisch auf `-1` zurück. `UI_TABLE_SET_SELECTED` greift erst, nachdem die Tabelle einmal mit `UI_TABLE` gezeichnet wurde (die `id$` muss bekannt sein).

### Klickbare Header / Sortierung

```basic
UI_TABLE_HEADER_CLICK(id$)     -> INTEGER   ' geklickte Header-Spalte, -1 wenn keine
```

Liefert die Spalte, deren **Kopfzeile** in diesem Frame angeklickt wurde (Press + Release auf derselben Spalte). Da `UI_TABLE` Immediate-Mode ist und die Daten dem Spiel gehören, sortiert die Tabelle **nicht** selbst — sie meldet nur den Klick, das Spiel sortiert seine Arrays und übergibt sie im nächsten Frame neu:

```basic
DIM col AS INTEGER
col = UI_TABLE_HEADER_CLICK("scores")
IF col >= 0 THEN
    sort_dir = IIF(sort_col = col, -sort_dir, 1)   ' Toggle auf/ab
    sort_col = col
    sortiere_daten(cells, col, sort_dir)            ' eigene Sortier-Routine
END IF
```

Direkt **nach** dem `UI_TABLE`-Aufruf im selben Frame abfragen.

**Vollständiges Beispiel** mit Selektion, klickbaren Headern (auf-/absteigend) und Detail-Panel: [examples/81_table_select.gb](../examples/81_table_select.gb).

## Immediate-Mode-Fenster (UI_WINDOW_BEGIN / UI_WINDOW_END)

Verschiebbare, einklappbare Fenster im Immediate-Mode. Zwischen
`UI_WINDOW_BEGIN` und `UI_WINDOW_END` gezeichnete Widgets sind **fenster-
relativ** (ihre `x`/`y` zählen ab der linken oberen Ecke des Fenster-Inhalts,
unterhalb der Titelleiste) und wandern beim Verschieben automatisch mit.

```basic
IMPORT "ui"
SCREEN(480, 360, "Fenster", 2)

WHILE NOT QUITREQUESTED()
    CLS(RGB(16, 20, 36))

    IF UI_WINDOW_BEGIN("settings", "Einstellungen", 120, 80, 220, 140) THEN
        IF UI_BUTTON("ok", 20, 90, 90, 28, "OK") THEN PRINT "OK"
        DIM snd AS BOOLEAN
        snd = UI_CHECKBOX("snd", 20, 30, "Sound", TRUE)
        DIM vol AS FLOAT
        vol = UI_SLIDER("vol", 20, 55, 170, 0.0, 1.0, 0.7)
    END IF
    UI_WINDOW_END()

    UI_END_FRAME()
    FLIP()
    SLEEP(16)
WEND
```

- **`UI_WINDOW_BEGIN(id$, titel$, x, y, w, h)`** → BOOLEAN. Liefert **TRUE**
  wenn das Fenster offen ist (Body zeichnen), **FALSE** wenn eingeklappt — dann
  den Body via `IF ... THEN ... END IF` überspringen. `x, y` sind die
  Startposition; ab dem ersten Frame merkt sich das Fenster seine (per Drag
  geänderte) Position über die `id$`.
- **`UI_WINDOW_END()`** schließt das aktuelle Fenster. **Immer** aufrufen —
  auch wenn `UI_WINDOW_BEGIN` FALSE lieferte (also außerhalb des `IF`).
- **Verschieben**: Titelleiste ziehen. **Einklappen**: auf den `-`/`+`-Pfeil
  links in der Titelleiste klicken.
- **Z-Order / Input**: Fenster werden in Aufrufreihenfolge gezeichnet (später
  aufgerufen = optisch oben); Maus-Input geht an das oberste Fenster unter dem
  Cursor (Hit-Test aus dem Vorframe, wie in Dear ImGui). Ein echtes
  „Klick bringt nach vorne" (Umsortieren) gibt es im Immediate-Mode nicht —
  dafür das Retained-Mode-Modul [`gui`](module-gui.md).
- Widgets außerhalb von Fenstern verhalten sich unverändert (Offset 0).

## UI_END_FRAME

```basic
UI_END_FRAME()
```

**Pflicht** am Ende jedes Frames, **vor** `FLIP()`. Speichert:
- Mausstatus (für Klick-Edge-Detection in Buttons/Checkboxen/Radios/Tabellen)
- Tastatur-Snapshot (für Backspace-Edge im Textfeld)
- Frame-Counter (für blinkenden Cursor)

Ohne diesen Aufruf zählen gehaltene Mausklicks als kontinuierliches Klicken (Buttons feuern jeden Frame, Checkboxen toggelten unendlich) und gehaltene Backspace löscht alle Zeichen sofort.

## UI_RESET

```basic
UI_RESET()
```

Setzt allen UI-State zurück: Checkbox-Werte, Slider-Werte, Klick-State. Sinnvoll bei:
- Spiel-Restart (Settings auf Default zurück)
- Wechsel des Menü-Screens (alte UI-IDs sollen vergessen werden)

```basic
SUB neues_spiel()
    UI_RESET()
    state = "playing"
END SUB
```

## Vollständiges Beispiel

Ein Settings-Dialog mit Lautstärke-Slider, Sound/Musik-Checkboxen, drei Schwierigkeits-Buttons und einem Beenden-Knopf:

```basic
IMPORT "ui"

CONST W AS INTEGER = 320
CONST H AS INTEGER = 240
SCREEN(W, H, "UI-Demo", 2)

DIM beendet AS BOOLEAN
beendet = FALSE

WHILE NOT QUITREQUESTED() AND NOT beendet
    IF KEYPRESSED(KEY_ESCAPE) THEN
        BREAK
    END IF

    CLS(RGB(20, 25, 40))

    UI_LABEL(10, 10, "Einstellungen", RGB(255, 220, 80))

    UI_LABEL(10, 50, "Lautstaerke:")
    DIM vol AS FLOAT
    vol = UI_SLIDER("vol", 110, 50, 180, 0.0, 1.0, 0.7)

    DIM sound AS BOOLEAN
    sound = UI_CHECKBOX("snd", 10, 100, "Sound an", TRUE)
    DIM musik AS BOOLEAN
    musik = UI_CHECKBOX("mus", 10, 120, "Musik an", FALSE)

    UI_LABEL(10, 150, "Schwierigkeit:")
    IF UI_BUTTON("easy", 110, 145, 60, 24, "Leicht") THEN
        PRINT "Leicht gewaehlt"
    END IF
    IF UI_BUTTON("hard", 175, 145, 60, 24, "Hart") THEN
        PRINT "Hart gewaehlt"
    END IF

    IF UI_BUTTON("quit", 240, 200, 70, 24, "Beenden", RGB(120, 40, 40)) THEN
        beendet = TRUE
    END IF

    UI_END_FRAME()
    FLIP()
    SLEEP(16)
WEND
```

Siehe auch [examples/33_ui.gb](../examples/33_ui.gb) — komplettes lauffähiges Demo.

## Tipps

- **`UI_END_FRAME()` immer aufrufen** — sonst geht Klick-Edge-Detection kaputt.
- **IDs sprechend wählen** — nicht `"a"`, `"b"`, sondern `"musik"`, `"vol"`, `"start"`. Hilft beim Debugging.
- **Bedingung am Knopf-Aufruf**: `IF UI_BUTTON(...) THEN ... END IF` ist die idiomatische Form. Der Knopf wird **immer** gezeichnet — die `IF`-Bedingung greift nur die Klick-Auswertung.
- **Layout per Variablen**: bei vielen Komponenten sammelt man `y_offset` und arbeitet mit Konstanten:
  ```basic
  DIM y AS INTEGER
  y = 50
  UI_LABEL(10, y, "...")
  y = y + 30
  UI_BUTTON("a", 10, y, 80, 24, "A")
  y = y + 30
  UI_BUTTON("b", 10, y, 80, 24, "B")
  ```
- **Scroll-Trick**: für Listen mit vielen Buttons kann man eine `scroll_y`-Variable an alle `y`-Werte addieren — billige Scroll-Liste.

## Aussehen ändern (Theme & Metriken)

Die Default-Farben aller Komponenten lassen sich global umstellen — entweder per fertigem Schema oder pro Einzelfarbe. (Farben, die du direkt als Argument übergibst, z. B. `UI_BUTTON(..., bg, fg)`, haben weiterhin Vorrang.)

```basic
UI_THEME_PRESET("dark")       ' Default
UI_THEME_PRESET("light")      ' helles Schema
UI_THEME_PRESET("retro")      ' grün auf schwarz
UI_THEME_PRESET("contrast")   ' schwarz/gelb

UI_THEME_SET("button_bg", RGB(60, 40, 90))   ' eine Farbe global
DIM c AS INTEGER
c = UI_THEME_GET("accent")
```

Theme-Schlüssel: `accent`, `text_fg`, `muted_fg`, `button_bg`, `panel_bg`, `panel_border`, `panel_title_bg`, `field_bg`, `field_border`, `slider_track`, `progress_fg`, `progress_bg`, `win_bg`, `win_border`, `win_title_bg`, `win_title_bg_focus`.

Größen über Metriken:

```basic
UI_METRIC_SET("checkbox_size", 20)   ' größere Checkbox
UI_METRIC_SET("slider_h", 18)
UI_METRIC_SET("win_title_h", 24)     ' Titelleiste der UI_WINDOW_BEGIN-Fenster
```

`UI_RESET()` setzt auch Theme + Metriken auf die Defaults zurück.

## Limitationen (Stand jetzt)

- **Kein Dropdown / ListBox** — bisher mit Radio-Group + Panel oder einer Reihe Buttons gut substituierbar.
- **Layout ist absolut** — keine automatischen Layouts à la "horizontal/vertical stack". Du legst Pixel selbst.
- **Schriftart fix** — die Standard-SansSerif der nativen Runtime. Du kannst die Größe global mit `TEXT_SIZE(n)` umstellen — das wirkt aber auf alle nachfolgenden TEXT/UI-Aufrufe.
- **Kein Multi-Line-Textfeld** — `UI_TEXTFIELD` ist single-line. Enter wird nicht behandelt; du kannst's selbst per `KEYPRESSED(13)` als Submit-Trigger nutzen.

Wenn du eines dieser Features brauchst, in einer späteren Iteration leicht zu ergänzen.

Siehe auch [examples/42_ui_extended.gb](../examples/42_ui_extended.gb) für eine Demo aller neuen Komponenten kombiniert (HP-Bars, Settings-Panel mit Textfeld + Radio + Slider).
