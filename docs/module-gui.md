# Modul `gui`

Retained-Mode-GUI: Fenster und Widgets sind **persistente Objekte**. Einmal
angelegt, leben sie weiter — du baust die Oberfläche einmal auf und rufst pro
Frame nur noch `GUI_UPDATE()` (Maus/Tasten verarbeiten) und `GUI_DRAW()`
(zeichnen). Events fragst du per **Polling** ab (`IF GUI_CLICKED(btn) THEN ...`).

```basic
IMPORT "gui"
```

> Unterschied zum [`ui`-Modul](module-ui.md): `ui` ist **Immediate-Mode** —
> jedes Widget wird jeden Frame neu aufgerufen, kein Objekt bleibt bestehen.
> `gui` ist **Retained-Mode** mit echten Fenster-/Widget-Objekten, inklusive
> Verschieben (Drag an der Titelleiste), Z-Order (Klick bringt ein Fenster
> nach vorne), Fokus und Schließen-Button. Beide Module dürfen parallel
> benutzt werden.

## Übersicht

| Funktion | Rückgabe | Zweck |
|---|---|---|
| `GUI_WINDOW(titel$, x, y, w, h)` | GUI_WINDOW | Fenster anlegen |
| `GUI_WINDOW_MOVABLE(win, an)` | — | per Titelleiste verschiebbar (Default: an) |
| `GUI_WINDOW_CLOSABLE(win, an)` | — | Schließen-Button anzeigen (Default: aus) |
| `GUI_WINDOW_VISIBLE(win, an)` | — | Sichtbarkeit setzen |
| `GUI_WINDOW_CLOSED(win)` | BOOLEAN | wurde das Fenster geschlossen? |
| `GUI_BUTTON(win, text$, x, y, w, h)` | GUI_WIDGET | Knopf |
| `GUI_LABEL(win, text$, x, y[, farbe])` | GUI_WIDGET | Text |
| `GUI_CHECKBOX(win, label$, x, y[, default])` | GUI_WIDGET | Toggle |
| `GUI_SLIDER(win, x, y, w, min, max[, default])` | GUI_WIDGET | Wert-Schieber |
| `GUI_PANEL(win, x, y, w, h[, titel$])` | GUI_WIDGET | Container (Deko) |
| `GUI_TEXTINPUT(win, x, y, w, h[, platzhalter$])` | GUI_WIDGET | Eingabefeld |
| `GUI_UPDATE()` | — | **Pflicht** pro Frame: Maus/Tasten verarbeiten |
| `GUI_DRAW()` | — | alle Fenster zeichnen (hinten→vorne) |
| `GUI_CLICKED(widget)` | BOOLEAN | Button in diesem Frame geklickt? |
| `GUI_CHECKED(widget)` | BOOLEAN | Checkbox-Zustand |
| `GUI_VALUE(widget)` | FLOAT | Slider-Wert |
| `GUI_TEXT(widget)` | STRING | Text (Label/Button/TextInput) |
| `GUI_HOVERED(widget)` | BOOLEAN | Maus über dem Widget? |
| `GUI_SET_TEXT(widget, text$)` | — | Text setzen |
| `GUI_SET_CHECKED(widget, an)` | — | Checkbox setzen |
| `GUI_SET_VALUE(widget, wert)` | — | Slider-Wert setzen (wird geclamped) |
| `GUI_ON_CLICK(widget, funcref)` | — | FUNCREF-Callback bei Klick (Button/Checkbox) |
| `GUI_ON_CHANGE(widget, funcref)` | — | FUNCREF-Callback bei Wertänderung (Slider/TextInput/Checkbox) |
| `GUI_THEME(accent)` | — | Akzentfarbe (RGB) umstellen |
| `GUI_RESET()` | — | alle Fenster/Widgets löschen |

## Externe Typen

`gui` registriert zwei Typen, die du mit `DIM` deklarieren kannst:

```basic
DIM win AS GUI_WINDOW
DIM btn AS GUI_WIDGET
```

Alle Widgets (Button, Label, Checkbox, …) haben denselben Typ `GUI_WIDGET` —
welcher Art ein Widget ist, ergibt sich aus der Konstruktor-Funktion.

## Konzept: ein Aufbau, dann Polling pro Frame

```basic
IMPORT "gui"
SCREEN(480, 360, "GUI", 2)

' --- einmaliger Aufbau ---
DIM win AS GUI_WINDOW
win = GUI_WINDOW("Einstellungen", 80, 50, 300, 220)
GUI_WINDOW_MOVABLE(win, TRUE)
GUI_WINDOW_CLOSABLE(win, TRUE)

DIM ok AS GUI_WIDGET
ok = GUI_BUTTON(win, "Start", 20, 150, 110, 32)
DIM snd AS GUI_WIDGET
snd = GUI_CHECKBOX(win, "Sound an", 20, 60, TRUE)

' --- Frame-Schleife ---
WHILE NOT QUITREQUESTED()
    CLS(&H101828)
    GUI_UPDATE()                 ' Maus/Tasten: Hover, Klick, Drag, Fokus, Z-Order
    GUI_DRAW()                   ' alle Fenster hinten→vorne

    IF GUI_CLICKED(ok) THEN
        DIM an AS BOOLEAN
        an = GUI_CHECKED(snd)
        PRINT "Start, Sound=" + STR$(an)
    END IF
    IF GUI_WINDOW_CLOSED(win) THEN BREAK

    FLIP()
    SLEEP(16)
WEND
```

`GUI_UPDATE()` muss **vor** `GUI_DRAW()` und vor den Polling-Abfragen stehen —
es verarbeitet die Maus-/Tasten-Events dieses Frames und setzt die internen
Flags (geklickt, gehovert, Slider-Wert, Fokus, Fenster-Z-Order).

## Koordinaten

Widget-Koordinaten `(x, y)` sind **relativ zum Client-Bereich** ihres
Fensters (unterhalb der 22 px hohen Titelleiste). Verschiebt sich das Fenster,
wandern die Widgets automatisch mit.

## Fenster

```basic
GUI_WINDOW(titel$, x, y, w, h) -> GUI_WINDOW
```

Legt ein Fenster an und gibt es zurück. Neue Fenster erscheinen **oben**
(vorderste Z-Order). Ein Klick in ein Fenster bringt es nach vorne.

- `GUI_WINDOW_MOVABLE(win, TRUE)` — Drag an der Titelleiste (Default: TRUE)
- `GUI_WINDOW_CLOSABLE(win, TRUE)` — Schließen-Button (×) oben rechts
- `GUI_WINDOW_VISIBLE(win, FALSE)` — aus-/einblenden (Einblenden setzt das
  Geschlossen-Flag zurück)
- `GUI_WINDOW_CLOSED(win)` — TRUE sobald der ×-Button gedrückt wurde (bleibt
  TRUE bis das Fenster wieder sichtbar gesetzt wird)

## Button

```basic
GUI_BUTTON(win, text$, x, y, w, h) -> GUI_WIDGET
```

`GUI_CLICKED(btn)` liefert in genau dem Frame **TRUE**, in dem die Maus über
dem Knopf **losgelassen** wurde (nachdem sie vorher darauf gedrückt wurde —
press-and-release über demselben Knopf, wie ein klassischer OK-Knopf).

## Label

```basic
GUI_LABEL(win, text$, x, y[, farbe]) -> GUI_WIDGET
```

Statischer Text. Default-Farbe weiß. Mit `GUI_SET_TEXT(lbl, ...)` jederzeit
änderbar (z. B. um einen Slider-Wert live anzuzeigen).

## Checkbox

```basic
GUI_CHECKBOX(win, label$, x, y[, default]) -> GUI_WIDGET
```

Klick toggelt. `GUI_CHECKED(chk)` liefert den Zustand, `GUI_SET_CHECKED` setzt
ihn programmatisch.

## Slider

```basic
GUI_SLIDER(win, x, y, w, min, max[, default]) -> GUI_WIDGET
```

Horizontaler Wert-Schieber. `GUI_VALUE(s)` liefert den FLOAT-Wert,
`GUI_SET_VALUE(s, v)` setzt ihn (geclamped auf `[min, max]`). `max > min` ist
Pflicht.

## TextInput

```basic
GUI_TEXTINPUT(win, x, y, w, h[, platzhalter$]) -> GUI_WIDGET
```

Einzeiliges Eingabefeld. Klick fokussiert, getippte Zeichen werden angehängt,
Backspace löscht. `GUI_TEXT(tf)` liefert den Inhalt, `GUI_SET_TEXT(tf, ...)`
belegt ihn vor. Ein blinkender Cursor erscheint am fokussierten Feld.

## Panel

```basic
GUI_PANEL(win, x, y, w, h[, titel$]) -> GUI_WIDGET
```

Rein dekorativer Container (Rahmen + optionale Titelzeile). Nicht interaktiv.

## Theme

Default ist die Logo-Cyan-Palette (konsistent zum Editor). `GUI_THEME(accent)`
stellt die Akzentfarbe (RGB-INTEGER) um, z. B. `GUI_THEME(RGB(255, 160, 60))`.

## Callbacks: GUI_ON_CLICK

Neben dem Polling (`IF GUI_CLICKED(b) THEN ...`) kannst du eine GameBasic-
FUNCTION/SUB als **Callback** registrieren, die automatisch beim Klick
aufgerufen wird:

```basic
SUB on_start()
    PRINT "Start gedrueckt!"
END SUB

DIM ok AS GUI_WIDGET
ok = GUI_BUTTON(win, "Start", 20, 150, 110, 32)
GUI_ON_CLICK(ok, on_start)        ' on_start ist eine FUNCREF
```

- Der Handler ist **parameterlos** (Zustand fragst du drin per `GUI_CHECKED`/
  `GUI_TEXT`/… ab). Er läuft wie eine normale Funktion — sieht Parameter und
  Globals, aber keine Locals des umgebenden Scopes (FUNCREF-Regel).
- Funktioniert für **Buttons** (Klick = Press+Release auf dem Knopf) und
  **Checkboxen** (bei jedem Toggle).
- Die Callbacks werden **am Ende von `GUI_UPDATE()`** aufgerufen (nachdem alle
  Events des Frames verarbeitet sind). Während eines Callbacks ausgelöste
  weitere Events feuern erst im nächsten Frame — keine Re-Entrancy-Schleife.
- Polling und Callback schließen sich nicht aus; beides ist gleichzeitig nutzbar.
- `GUI_ON_CLICK(widget, NIL)` entfernt den Callback wieder.

Intern überbrückt `GUI_ON_CLICK` die Builtin→Interpreter/VM-Grenze (eine
FUNCREF aus einem Built-in heraus aufrufen) — bit-identisch im Tree-Walker, in
der Python-VM und in der nativen VM.

### GUI_ON_CHANGE (Wertänderung)

`GUI_ON_CHANGE(widget, funcref)` ruft den Handler auf, wenn sich der **Wert**
eines Widgets ändert:

- **Slider** — während des Ziehens, bei jeder tatsächlichen Wertänderung
- **TextInput** — beim Tippen/Löschen (Textänderung)
- **Checkbox** — beim Toggle (zusätzlich zu `GUI_ON_CLICK`)

```basic
SUB vol_changed()
    PRINT "Lautstaerke: " + STR$(INT(GUI_VALUE(vol) * 100.0))
END SUB

vol = GUI_SLIDER(win, 20, 44, 240, 0.0, 1.0, 0.5)
GUI_ON_CHANGE(vol, vol_changed)
```

Gleiche Regeln wie `GUI_ON_CLICK`: parameterloser Handler, am Ende von
`GUI_UPDATE()` aufgerufen, `NIL` entfernt ihn. Nur für `slider`, `textinput`
und `checkbox` zulässig (sonst Fehler).

## Booleans drucken

`GUI_CLICKED`/`GUI_CHECKED`/`GUI_HOVERED` liefern echte BOOLEANs und
funktionieren direkt in Bedingungen (`IF GUI_CLICKED(b) THEN ...`). Möchtest du
einen solchen Wert mit `PRINT` als `TRUE`/`FALSE` ausgeben, weise ihn vorher
einer `BOOLEAN`-Variablen zu — wie bei jedem Modul-Built-in:

```basic
DIM r AS BOOLEAN
r = GUI_CHECKED(snd)
PRINT r            ' TRUE / FALSE
```

(`PRINT GUI_CHECKED(snd)` direkt gibt den Python-Wahrheitswert aus, da der
Rückgabetyp eines Built-ins beim PRINT nicht statisch bekannt ist.)

## GUI_RESET

```basic
GUI_RESET()
```

Löscht alle Fenster und Widgets — sinnvoll beim Wechsel des Menü-Screens.
Achtung: vorher geholte `GUI_WINDOW`/`GUI_WIDGET`-Referenzen zeigen danach auf
nicht mehr verwaltete Objekte.

## Vollständiges Beispiel

Siehe [examples/45_gui.gb](../examples/45_gui.gb): Fenster mit Slider (live ins
Label gespiegelt), Checkbox, Textfeld und Start-Button — verschiebbar und
schließbar.

## Limitationen (Stand jetzt)

- **Events**: Polling (`GUI_CLICKED` …) **und** FUNCREF-Callbacks
  (`GUI_ON_CLICK` für Buttons/Checkboxen, `GUI_ON_CHANGE` für Slider/TextInput/
  Checkbox) werden unterstützt.
- **Immediate-Mode-Fenster** (`UI_WINDOW_BEGIN/END` im `ui`-Modul) sind die
  geplante Alternative (Phase 4).
- **Absolute Koordinaten**, kein Auto-Layout.
- **Headless/grafisch**: `GUI_UPDATE`/`GUI_DRAW` brauchen einen aktiven
  `SCREEN`. State und Hit-Test sind headless getestet
  (`tests/test_modules_gui.py`).
