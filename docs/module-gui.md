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
> benutzt werden. Widgets werden auf den **Fenster-Innenbereich geclippt** —
> wird ein Fenster kleiner gezogen, ragt nichts über Rand/Titelleiste hinaus.

## Übersicht

| Funktion | Rückgabe | Zweck |
|---|---|---|
| `GUI_WINDOW(titel$, x, y, w, h)` | GUI_WINDOW | Fenster anlegen |
| `GUI_WINDOW_MOVABLE(win, an)` | — | per Titelleiste verschiebbar (Default: an) |
| `GUI_WINDOW_CLOSABLE(win, an)` | — | Schließen-Button anzeigen (Default: aus) |
| `GUI_WINDOW_VISIBLE(win, an)` | — | Sichtbarkeit setzen |
| `GUI_WINDOW_RESIZABLE(win, an)` | — | am unteren-rechten Griff größenveränderbar (Default: aus) |
| `GUI_WINDOW_SCROLLABLE(win, an)` | — | Inhalt scrollt, wenn er höher als das Fenster ist (Mausrad + Scrollbalken). Inhaltshöhe automatisch aus den Widgets. Default: aus |
| `GUI_WINDOW_CHROME(win, an)` | — | Titelleiste/Rahmen/Buttons zeichnen? Aus = randlos, Inhalt ab oben (damit eine Form das OS-Fenster ausfüllen kann). Default: an |
| `GUI_WINDOW_SET_MIN_SIZE(win, w, h)` | — | Mindestgröße beim Resizen (0 = keine) |
| `GUI_WINDOW_SET_MAX_SIZE(win, w, h)` | — | Maximalgröße beim Resizen (0 = keine) |
| `GUI_SEPARATOR(win, x, y, w)` | GUI_WIDGET | dekorative Trennlinie (horizontal) |
| `GUI_GROUPBOX(win, x, y, w, h, title$)` | GUI_WIDGET | gerahmte Gruppe mit eingelassenem Titel |
| `GUI_SET_ANCHOR(wdg, edges$)` | — | Anchoring: an welchen Kanten das Widget klebt (Teilmenge von `"lrtb"`, Default `"lt"` = oben-links). Beim Fenster-Resize fließen die Widgets mit: links+rechts → dehnen, nur rechts → mitwandern, keiner → zentrieren (analog oben/unten). |
| `GUI_WINDOW_CLOSED(win)` | BOOLEAN | wurde das Fenster geschlossen? |
| `GUI_BUTTON(win, text$, x, y, w, h)` | GUI_WIDGET | Knopf |
| `GUI_ICON_BUTTON(win, x, y, w, h, tex[, text$])` | GUI_WIDGET | Knopf mit Icon (Textur-Handle); ohne Text = flacher Toolbar-Button |
| `GUI_SET_ICON(button, tex)` | — | Icon eines Buttons setzen/ersetzen (-1 entfernt) |
| `GUI_TOOLBAR(win, x, y, w, h)` | GUI_WIDGET | flacher Werkzeugleisten-Streifen (Deko, für Icon-Button-Reihe) |
| `GUI_LABEL(win, text$, x, y[, farbe])` | GUI_WIDGET | Text |
| `GUI_CHECKBOX(win, label$, x, y[, default])` | GUI_WIDGET | Toggle |
| `GUI_SLIDER(win, x, y, w, min, max[, default])` | GUI_WIDGET | Wert-Schieber |
| `GUI_SPINNER(win, x, y, w, min, max[, default[, step]])` | GUI_WIDGET | Zahlenfeld mit +/- (Klick/Mausrad/Pfeiltasten; Wert via `GUI_VALUE`) |
| `GUI_SPLITTER(win, x, y, length, orient$, min, max)` | GUI_WIDGET | verschiebbare Trennlinie (`"v"`/`"h"`); Position via `GUI_VALUE` |
| `GUI_PANEL(win, x, y, w, h[, titel$])` | GUI_WIDGET | Container (Deko) |
| `GUI_TEXTINPUT(win, x, y, w, h[, platzhalter$])` | GUI_WIDGET | einzeiliges Eingabefeld (Caret + Selektion) |
| `GUI_TEXTAREA(win, x, y, w, h[, platzhalter$])` | GUI_WIDGET | **mehrzeiliges** Textfeld (ENTER = neue Zeile, vertikal scrollend, Pfeile hoch/runter, Selektion via Maus-Drag/Shift+Pfeil, Strg+A/C/X/V) |
| `GUI_TABLE(win, x, y, w, h[, headers, cells])` | GUI_WIDGET | scrollbare Tabelle (Header + Body) |
| `GUI_TREE(win, x, y, w, h)` | GUI_WIDGET | Baum-Ansicht (auf-/zuklappbar, scrollbar) |
| `GUI_TREE_ADD(tree, parent, label$)` | INT | Knoten anhängen (parent = -1 = Wurzel), liefert Knoten-id |
| `GUI_TREE_CLEAR(tree)` | — | alle Knoten löschen |
| `GUI_TREE_SELECTED(tree)` / `GUI_TREE_SET_SELECTED(tree, node)` | INT / — | gewählten Knoten lesen/setzen (-1 = keiner) |
| `GUI_TREE_LABEL(tree, node)` | STRING | Text eines Knotens |
| `GUI_TREE_EXPAND(tree, node, flag)` | — | Knoten auf-/zuklappen |
| `GUI_TABLE_HEADERS(tbl, headers)` | — | Spaltentitel setzen (1D ARRAY OF STRING) |
| `GUI_TABLE_ROWS(tbl, cells)` | — | Datenzeilen setzen (2D ARRAY OF STRING) |
| `GUI_TABLE_COL_WIDTHS(tbl, widths)` | — | Spaltenbreiten (1D ARRAY OF INTEGER; NIL = Auto) |
| `GUI_TABLE_SELECTED(tbl)` | INTEGER | selektierte Zeile (-1 wenn keine) |
| `GUI_TABLE_SET_SELECTED(tbl, row)` | — | Selektion setzen (-1 = keine) |
| `GUI_TABLE_CLICKED(tbl)` | INTEGER | in diesem Frame geklickte Zeile (-1) |
| `GUI_TABLE_ROW_COUNT(tbl)` | INTEGER | Anzahl Datenzeilen |
| `GUI_UPDATE()` | — | **Pflicht** pro Frame: Maus/Tasten verarbeiten |
| `GUI_DRAW()` | — | alle Fenster zeichnen (hinten→vorne) |
| `GUI_CLICKED(widget)` | BOOLEAN | Button in diesem Frame geklickt? |
| `GUI_CHECKED(widget)` | BOOLEAN | Checkbox-Zustand |
| `GUI_VALUE(widget)` | FLOAT | Slider-Wert |
| `GUI_TEXT(widget)` | STRING | Text (Label/Button/TextInput) |
| `GUI_HOVERED(widget)` | BOOLEAN | Maus über dem Widget? |
| `GUI_SET_TEXT(widget, text$)` | — | Text setzen |
| `GUI_TOOLTIP(widget, text$)` | — | Hover-Hilfetext (mehrzeilig via `\n`; "" entfernt ihn) |
| `GUI_SET_CHECKED(widget, an)` | — | Checkbox setzen |
| `GUI_SET_VALUE(widget, wert)` | — | Slider-Wert setzen (wird geclamped) |
| `GUI_ON_CLICK(widget, funcref)` | — | FUNCREF-Callback bei Klick (Button/Checkbox) |
| `GUI_ON_CHANGE(widget, funcref)` | — | FUNCREF-Callback bei Wertänderung (Slider/TextInput/Checkbox/Table-Selektion) |
| `GUI_THEME(accent)` | — | Akzentfarbe (RGB) umstellen (Kurzform) |
| `GUI_THEME_SET(key$, farbe)` / `GUI_THEME_GET(key$)` | — / INT | einzelne Theme-Farbe setzen/lesen |
| `GUI_THEME_PRESET(name$)` | — | Look: dark/light/retro/contrast + **modern_dark/modern_light** (rund + Schatten) |
| `GUI_METRIC_SET(key$, wert)` / `GUI_METRIC_GET(key$)` | — / INT | Layout-Größe setzen/lesen |
| `GUI_SET_COLOR(widget, rolle$, farbe)` | — | eine Farbe pro Widget (bg/fg/border/accent; -1 entfernt) |
| `GUI_RESET()` | — | Fenster/Widgets löschen + Theme/Metriken zurücksetzen |

**Tooltips:** `GUI_TOOLTIP(widget, text$)` hängt einem beliebigen Widget einen Hilfetext an. Er erscheint automatisch, sobald die Maus ~0,5 s ruhig über dem Widget verweilt (nur im obersten Fenster), und folgt dem Cursor am Bildschirmrand abgeklemmt. `\n` macht mehrere Zeilen; `""` entfernt den Tooltip wieder. Bewegung oder ein Mausklick setzt die Verweilzeit zurück.

### Menüs

| Funktion | Rückgabe | Zweck |
|---|---|---|
| `GUI_MENU(win, label$)` | Menü-Handle | Top-Level-Menü in der **Menüleiste** (z. B. „Datei") |
| `GUI_CONTEXT(win)` | Menü-Handle | **Kontextmenü** (per Rechtsklick im Fenster) |
| `GUI_MENU_ITEM(menu, label$)` | Item-Handle | Eintrag anhängen — Handle für `GUI_CLICKED` |
| `GUI_MENU_SEPARATOR(menu)` | — | Trennlinie anhängen |

Klick-Auswertung wie bei Buttons über `GUI_CLICKED(item)`. Die Menüleiste schiebt den Fensterinhalt automatisch nach unten; Klick auf ein Menü öffnet das Dropdown, Klick daneben schließt es. Komplettes Beispiel: [`examples/129_gui_menu.gb`](../examples/129_gui_menu.gb).

### Reiter (Tabs) + Tastatur-Navigation

| Funktion | Rückgabe | Zweck |
|---|---|---|
| `GUI_TABS(win, labels)` | — | Reiter-Leiste anlegen (`labels` = ARRAY/TUPLE von Strings); aktiver Reiter → 0 |
| `GUI_SET_TAB(widget, seite)` | — | Widget einem Reiter zuordnen (`-1` = auf allen Reitern sichtbar) |
| `GUI_ACTIVE_TAB(win)` | INTEGER | aktiver Reiter-Index |
| `GUI_SET_ACTIVE_TAB(win, i)` | — | Reiter umschalten |

Nur die Widgets des aktiven Reiters (plus die mit `tab_page = -1`) werden gezeichnet und sind bedienbar. **Tastatur:** `TAB` / `SHIFT+TAB` wechselt den Fokus zwischen den Eingabefeldern (TextInput **und** TextArea) des aktiven Fensters. Beispiel: [`examples/131_gui_tabs.gb`](../examples/131_gui_tabs.gb).

### Modale Dialoge

Native, blockierende Standarddialoge (kein IMPORT nötig — wie die Datei-Dialoge):

| Funktion | Rückgabe | Zweck |
|---|---|---|
| `GUI_MESSAGE(titel$, text$)` | — | Info-Box mit OK |
| `GUI_CONFIRM(titel$, text$)` | BOOLEAN | OK/Abbrechen → `TRUE` bei OK |

```basic
IF GUI_CONFIRM("Löschen?", "Wirklich alles löschen?") THEN GUI_SET_TEXT(ta, "")
GUI_MESSAGE("Fertig", "Gespeichert.")
```

Beispiel mit TextArea + Dialogen: [`examples/132_gui_textarea.gb`](../examples/132_gui_textarea.gb).

`labels` ist ein `ARRAY OF STRING` — am einfachsten via `SPLIT$`:

```basic
DIM tabs AS ARRAY OF STRING : tabs = SPLIT$("Allgemein|Konto", "|")
GUI_TABS(win, tabs)
DIM nameI AS GUI_WIDGET : nameI = GUI_TEXTINPUT(win, 24, 44, 400, 32, "Name ...")
GUI_SET_TAB(nameI, 0)                 ' nur auf Reiter "Allgemein"
DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(win, "OK", 24, 300, 200, 38)   ' ohne SET_TAB -> immer sichtbar
```

```basic
DIM mFile AS INTEGER : mFile = GUI_MENU(win, "Datei")
DIM miOpen AS INTEGER : miOpen = GUI_MENU_ITEM(mFile, "Öffnen ...")
GUI_MENU_SEPARATOR(mFile)
DIM miQuit AS INTEGER : miQuit = GUI_MENU_ITEM(mFile, "Beenden")
' ... in der Schleife:
IF GUI_CLICKED(miOpen) THEN ...
```

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

## Formular-Widgets: Radio, Dropdown, ProgressBar

### RadioButton (Gruppen, gegenseitiger Ausschluss)

```basic
DIM easy AS GUI_WIDGET
easy = GUI_RADIO(win, "diff", "Einfach", 20, 60)   ' Gruppe "diff"
DIM hard AS GUI_WIDGET
hard = GUI_RADIO(win, "diff", "Schwer", 20, 90)    ' selbe Gruppe
GUI_SET_CHECKED(easy, TRUE)                          ' waehlt easy, deselektiert die Gruppe
```

Radios mit derselben `group$` im selben Fenster schliessen sich gegenseitig aus.
- `GUI_CHECKED(radio)` → ist dieser gewählt?
- `GUI_RADIO_SELECTED(radio)` → Index (0-basiert, Erstellungsreihenfolge) des
  gewählten Radios der Gruppe, oder `-1`. (`radio` darf ein beliebiges der Gruppe sein.)
- `GUI_ON_CHANGE(radio, handler)` feuert bei Auswahl.

### Dropdown / ComboBox

```basic
DIM farben[3] AS STRING
farben[0]="Rot" : farben[1]="Grün" : farben[2]="Blau"
DIM dd AS GUI_WIDGET
dd = GUI_DROPDOWN(win, 20, 130, 160, 24, farben)   ' Klick klappt die Liste auf
```

- `GUI_DROPDOWN_SELECTED(dd)` → Index (oder `-1`), `GUI_DROPDOWN_TEXT(dd)` → gewählter Text.
- `GUI_DROPDOWN_SET_SELECTED(dd, i)` setzt die Auswahl, `GUI_SET_DROPDOWN(dd, items)` ersetzt die Liste.
- `GUI_ON_CHANGE(dd, handler)` feuert bei Auswahländerung. Das aufgeklappte Popup
  wird über allen anderen Widgets gezeichnet; Klick daneben schliesst es.

### ProgressBar

```basic
DIM bar AS GUI_WIDGET
bar = GUI_PROGRESS(win, 20, 170, 200, 18)
GUI_SET_VALUE(bar, 0.65)        ' 0.0 .. 1.0 (zeigt "65%")
PRINT GUI_VALUE(bar)
```

Nicht interaktiv; Fortschritt via `GUI_SET_VALUE` (0..1, geklemmt).

### Spinner — Zahlenfeld mit +/-

```basic
DIM sp AS GUI_WIDGET
sp = GUI_SPINNER(win, 20, 200, 120, 0, 100, 50, 5)   ' min 0, max 100, Start 50, Schritt 5
GUI_ON_CHANGE(sp, FUNCREF(wert_geaendert))
PRINT GUI_VALUE(sp)                                   ' aktueller Wert (FLOAT)
```

Eingabefeld mit zwei Schaltflächen (▲/▼) rechts. Der Wert ändert sich um `step`
(Default 1) per Klick auf die Buttons, **Mausrad** über dem Feld oder **Pfeil
hoch/runter**, wenn das Feld den Fokus hat (Klick hinein). Immer auf `[min, max]`
geklemmt. `GUI_VALUE`/`GUI_SET_VALUE` lesen/setzen den Wert, `GUI_ON_CHANGE`
feuert bei jeder Änderung. Ganze Zahlen werden ohne Nachkommastellen angezeigt.

### Splitter — verschiebbare Trennlinie

```basic
DIM spl AS GUI_WIDGET
spl = GUI_SPLITTER(win, 300, 70, 360, "v", 140, 440)   ' senkrechter Balken, x 140..440
' ... jeden Frame: Position lesen und zwei Bereiche layouten:
DIM sx AS INTEGER : sx = GUI_VALUE(spl)
GUI_SET_BOUNDS(linksPanel,  20,   70, sx - 20,        360)
GUI_SET_BOUNDS(rechtsPanel, sx+6, 70, 580 - (sx+6),   360)
```

Ein dünner Greif-Balken, den man mit der Maus zieht. `orient$` = `"v"`/`"vertical"`
(senkrechter Balken, zieht waagerecht) oder `"h"`/`"horizontal"` (waagerechter
Balken, zieht senkrecht). Bei `"v"` ist `x` die Startposition (geklemmt auf
`[min, max]`) und der Balken läuft von `y` über `length` Pixel; bei `"h"` sind
`x`/`y` getauscht. Das GUI hat **kein** Layout-Parenting — die Trennlinie liefert
nur ihre Position (`GUI_VALUE`, fenster-relativ); damit legst du selbst die zwei
Bereiche per `GUI_SET_BOUNDS` fest. `GUI_ON_CHANGE` feuert beim Ziehen.

### Icon-Buttons & Toolbar

```basic
DIM tb AS GUI_WIDGET : tb = GUI_TOOLBAR(win, 0, 0, 600, 46)        ' Streifen
DIM icSave AS INTEGER : icSave = GENTEX_COLOR(28, 28, &H4BE87A)    ' Icon = Textur-Handle
DIM bSave AS GUI_WIDGET : bSave = GUI_ICON_BUTTON(win, 8, 7, 34, 32, icSave)        ' nur Icon
DIM bRun  AS GUI_WIDGET : bRun  = GUI_ICON_BUTTON(win, 120, 7, 110, 32, icSave, "Start")  ' Icon + Text
IF GUI_CLICKED(bSave) THEN ...
GUI_SET_ICON(bRun, icOther)                                        ' Icon wechseln
```

`GUI_ICON_BUTTON` ist ein **ganz normaler Button** (`GUI_CLICKED`/`GUI_ON_CLICK`)
mit einem Bild. `tex` ist ein Textur-Handle aus `LOADIMAGE` oder `GENTEX_*`.
**Ohne** Text wird der Button flach gezeichnet (Fläche nur bei Hover/Klick) und
das Icon mittig — der klassische Toolbar-Look; **mit** Text steht das Icon links,
der Text rechts (normale Button-Optik). `GUI_TOOLBAR` ist nur ein dekorativer
Streifen als Hintergrund — die Icon-Buttons legst du selbst darauf.

## ListBox, Image, Canvas

### ListBox — scrollbare Auswahlliste

```basic
DIM obst[3] AS STRING
obst[0]="Apfel" : obst[1]="Birne" : obst[2]="Kirsche"
DIM lb AS GUI_WIDGET
lb = GUI_LISTBOX(win, 20, 40, 160, 100, obst)   ' Klick wählt, Mausrad scrollt
```

- `GUI_LISTBOX_SELECTED(lb)` → Index (oder `-1`), `GUI_LISTBOX_TEXT(lb)` → Text.
- `GUI_LISTBOX_SET_SELECTED(lb, i)`, `GUI_SET_LISTBOX(lb, items)`.
- `GUI_ON_CHANGE(lb, handler)` feuert bei Auswahl. (Teilt die Item-Logik mit
  Dropdown — die `GUI_DROPDOWN_*`-Getter funktionieren auch auf ListBoxen.)

### Image — Bild/Icon im UI

```basic
DIM logo AS INTEGER
logo = LOADIMAGE("assets/logo.png")
DIM iw AS GUI_WIDGET
iw = GUI_IMAGE(win, 20, 20, 96, 96, logo)   ' skaliert ins Rechteck
GUI_SET_IMAGE(iw, anderesBild)              ' Bild wechseln
```

Zeigt eine via `LOADIMAGE` geladene Textur, auf das Widget-Rechteck skaliert.

### Canvas — freie Zeichenfläche („Mini-Screen" im Fenster)

Ein Canvas reserviert einen Bereich, in den du mit den **normalen
Zeichenbefehlen** (`PLOT`/`LINE`/`BOX`/`CIRCLE`/`DRAWIMAGE`/3D …) malst — ideal
für ein eingebettetes Spiel, einen Diagramm- oder Vorschau-Bereich.

```basic
DIM cv AS GUI_WIDGET
cv = GUI_CANVAS(win, 10, 30, 280, 180)

' --- pro Frame ---
GUI_UPDATE()
GUI_DRAW()                       ' zeichnet das Fenster + den Canvas-Rahmen
' Danach in den Canvas-Bereich malen (absolute Bildschirm-Koordinaten):
DIM cx AS INTEGER : DIM cy AS INTEGER : DIM cw AS INTEGER : DIM ch AS INTEGER
cx = GUI_CANVAS_X(cv) : cy = GUI_CANVAS_Y(cv)
cw = GUI_CANVAS_W(cv) : ch = GUI_CANVAS_H(cv)
BOX(cx, cy, cx + cw, cy + ch, &H101820)
CIRCLE(cx + cw/2, cy + ch/2, 30, &H30FFA0)
FLIP()
```

`GUI_CANVAS_X/Y/W/H` liefern den **absoluten** Inhaltsbereich (folgt dem Fenster
beim Verschieben). Du zeichnest **nach** `GUI_DRAW` und clippst selbst auf den
Bereich. (Für echte Fenster-Überlappung/Occlusion ein Render-Target nutzen.)

## Tabelle

```basic
GUI_TABLE(win, x, y, w, h[, headers, cells]) -> GUI_WIDGET
```

Persistente Tabelle mit **fixierter Kopfzeile** und **scrollbarem Body**
(vertikal + horizontal). Komplement zum Immediate-Mode-`UI_TABLE`: die Daten
leben am Widget, gesetzt einmal beim Aufbau (oder jederzeit per Setter) statt
jeden Frame neu übergeben.

Daten gleich beim Anlegen mitgeben (beide Arrays oder keins) — oder leer
anlegen und später setzen:

```basic
DIM headers AS ARRAY OF STRING
headers = SPLIT$("Name|HP|Level", "|")

DIM cells[3, 3] AS STRING
' ... cells füllen ...

DIM tbl AS GUI_WIDGET
tbl = GUI_TABLE(win, 10, 40, 280, 160, headers, cells)
' alternativ:
'   tbl = GUI_TABLE(win, 10, 40, 280, 160)
'   GUI_TABLE_HEADERS(tbl, headers)
'   GUI_TABLE_ROWS(tbl, cells)
```

**Setter** (jederzeit, z. B. bei Daten-Updates):

| Built-in | Wirkung |
|---|---|
| `GUI_TABLE_HEADERS(tbl, headers)` | Spaltentitel (1D `ARRAY OF STRING`) |
| `GUI_TABLE_ROWS(tbl, cells)` | Datenzeilen (2D `ARRAY OF STRING`); Zeilen dürfen kürzer oder länger sein als der Kopf |
| `GUI_TABLE_COL_WIDTHS(tbl, widths)` | Pixelbreite pro Spalte (1D `ARRAY OF INTEGER`); `NIL` = gleichmäßig verteilen |

**Bedienung & Polling:**

| Built-in | Wirkung |
|---|---|
| `GUI_TABLE_SELECTED(tbl)` | persistent selektierte Zeile (-1 wenn keine) |
| `GUI_TABLE_SET_SELECTED(tbl, row)` | Selektion programmatisch setzen (-1 = keine; out-of-range → -1) |
| `GUI_TABLE_CLICKED(tbl)` | nur im Frame des Klicks die geklickte Zeile, sonst -1 |
| `GUI_TABLE_ROW_COUNT(tbl)` | Anzahl Datenzeilen |

**Verhalten:**
- Kopfzeile ist fixiert; der Body scrollt je nach Inhalt.
- **Mausrad** scrollt vertikal (über dem Body), **Scrollbalken-Drag** auf beiden Achsen.
- Klick auf eine Zeile **selektiert** sie persistent (zweites Highlight unter dem Hover).
- Schrumpfen die Daten unter den selektierten Index, fällt die Selektion auf -1.

### Die Spaltenzahl ist frei

Sie ergibt sich aus der **breitesten Angabe**: kürzere Zeilen als der Kopf sind
einfach leer, längere erweitern die Tabelle (der Kopf bekommt dann leere Titel).
Es gibt keine Reihenfolge, die man einhalten müsste — Zeilen vor dem Kopf zu
setzen ist genauso in Ordnung wie umgekehrt.

### Zellen: mehr als Text

Jede Zelle trägt Text, eigene **Vorder- und Hintergrundfarbe**, eine
**Ausrichtung** und eine **Art**:

| Art | Was gezeichnet wird |
|---|---|
| `text` | der Text (Vorgabe) |
| `bild` | ein IMAGE, in die Zelle eingepasst (Seitenverhältnis bleibt) |
| `haken` | Ankreuzfeld — **Klick schaltet um**, die Tabelle macht das selbst |
| `balken` | Fortschrittsbalken (`wert` 0..1), der Text liegt mittig darauf |
| `knopf` | Schaltfläche; welche Spalte getroffen wurde, sagt `GUI_TABLE_CLICKED_COL` |

| Built-in | Wirkung |
|---|---|
| `GUI_TABLE_SET_CELL(tbl, zeile, spalte, text$)` | Zellinhalt |
| `GUI_TABLE_GET_CELL(tbl, zeile, spalte)` | Zellinhalt lesen |
| `GUI_TABLE_CELL_COLOR(tbl, z, s, vordergrund, hintergrund)` | Farben; `-1` = nicht gesetzt |
| `GUI_TABLE_CELL_ALIGN(tbl, z, s, "links"/"mitte"/"rechts")` | Ausrichtung dieser Zelle |
| `GUI_TABLE_CELL_KIND(tbl, z, s, art$)` | Art (siehe Tabelle oben) |
| `GUI_TABLE_CELL_IMAGE(tbl, z, s, bild)` | Bild setzen (setzt die Art gleich mit) |
| `GUI_TABLE_CELL_VALUE(tbl, z, s, wert)` / `GUI_TABLE_GET_VALUE(...)` | Haken (0/1) bzw. Balken (0..1) |
| `GUI_TABLE_ROW_COLOR(tbl, zeile, vg, hg)` | ganze Zeile färben — **ein** Aufruf statt einer je Spalte |
| `GUI_TABLE_COL_ALIGN(tbl, spalte, wie$)` | Ausrichtung der ganzen Spalte |

**Farben in drei Stufen:** Zelle → Zeile → Zebra. `-1` heißt „nicht gesetzt"
und reicht an die nächste Stufe weiter — so färbt man mit einem Aufruf eine
ganze Zeile und lässt trotzdem einzelne Zellen ausscheren. Auswahl und Hover
liegen **halbdurchsichtig darüber**: eine eigene Zellfarbe deckt sie nicht zu.

### Zeilen einzeln pflegen

| Built-in | Wirkung |
|---|---|
| `GUI_TABLE_ADD_ROW(tbl, zellen)` | Zeile anhängen → neuer Zeilenindex |
| `GUI_TABLE_REMOVE_ROW(tbl, zeile)` | Zeile entfernen (die Auswahl rutscht mit) |
| `GUI_TABLE_CLEAR(tbl)` | alle Zeilen weg |

### Sortieren

Ein **Klick auf die Kopfzelle** sortiert, ein zweiter dreht die Richtung um; ein
kleiner Pfeil zeigt an, wonach gerade sortiert ist. Sortiert wird
**zahlenweise**, wenn beide Zellen als Zahl lesbar sind, sonst textweise — ohne
das stünde `100` vor `9`.

| Built-in | Wirkung |
|---|---|
| `GUI_TABLE_SORT(tbl, spalte, absteigend)` | programmgesteuert; Spalte `< 0` hebt die Sortierung auf |
| `GUI_TABLE_SORT_COL(tbl)` / `GUI_TABLE_SORT_DESC(tbl)` | aktuelle Sortierung lesen |

### Filtern

`GUI_TABLE_SET(tbl, "filterzeile", 1)` blendet unter dem Kopf eine Zeile mit
einem kleinen Eingabefeld je Spalte ein. Hineinklicken und tippen filtert die
Spalte nach **Teiltext**, Groß-/Kleinschreibung egal; `ESC` oder `Enter`
beendet die Eingabe. Mehrere Spalten wirken zusammen (UND).

| Built-in | Wirkung |
|---|---|
| `GUI_TABLE_FILTER(tbl, spalte, text$)` | Filter setzen (leerer Text = kein Filter) |
| `GUI_TABLE_GET_FILTER(tbl, spalte)` | gesetzten Filter lesen |

### Zellen direkt bearbeiten

**Doppelklick** auf eine Textzelle einer freigegebenen Spalte öffnet ein
Eingabefeld genau in dieser Zelle. `Enter` übernimmt, `ESC` nimmt zurück, ein
Klick woanders übernimmt ebenfalls (wie überall sonst auch — zurücknehmen geht
mit `ESC`).

```basic
GUI_TABLE_COL_EDIT(tbl, 1, TRUE)     ' Spalte 1 freigeben
```

Ohne Freigabe passiert beim Doppelklick nichts: eine Tabelle, in die man
versehentlich überall hineinschreiben kann, wäre schlimmer als eine ohne
Bearbeitung. Nur `text`-Zellen sind betroffen — ein Haken schaltet ohnehin per
Klick um, ein Balken oder Bild hat keinen Text zum Tippen.

| Built-in | Wirkung |
|---|---|
| `GUI_TABLE_COL_EDIT(tbl, spalte, an)` | Spalte für die Bearbeitung freigeben |
| `GUI_TABLE_EDITING_ROW(tbl)` / `GUI_TABLE_EDITING_COL(tbl)` | welche Zelle gerade bearbeitet wird (`-1` = keine) |

Beim Übernehmen feuert `on_change` und die Ansicht (Sortierung + Filter) wird
neu gebaut — **erst dann**, nicht bei jeder Taste: sonst spränge die Zeile
einem beim Tippen unter dem Finger weg.

**Der Editor kann:** Schreibmarke, Pfeiltasten, `Pos1`/`Ende`, `Rücktaste`,
`Entf`, Einfügen mit `Strg+V`, und er schiebt den Text nach links wenn er
länger ist als die Zelle. **Er kann nicht:** Text markieren. Die volle
Textfeld-Logik hier zu wiederholen wäre ein zweiter Ort, an dem dieselben
Fehler entstehen.

> **Achtung bei `ESC`:** wenn dein Programm `ESC` zum Beenden benutzt, frag
> vorher `GUI_TABLE_EDITING_ROW(tbl) < 0` ab — sonst beendet dieselbe Taste
> das Programm, mit der man eine Eingabe zurücknehmen will (so macht es die
> Demo).

### Zeilennummern: Daten oder Ansicht?

**Alle Zeilenangaben nach außen sind DATENzeilen.** Sortieren und Filtern
stellen die Daten nicht um, sie bauen nur eine Ansicht darüber — eine
Zeilennummer, die dein Programm sich gemerkt hat, zeigt also weiter auf
denselben Eintrag. (Würde stattdessen sortiert werden, zeigte jede gemerkte
Nummer nach dem ersten Kopfklick auf etwas anderes.)

Um die Tabelle in der **sichtbaren** Reihenfolge durchzugehen:

```basic
DIM i AS INTEGER
FOR i = 0 TO GUI_TABLE_VIEW_COUNT(tbl) - 1
    DIM r AS INTEGER : r = GUI_TABLE_VIEW_ROW(tbl, i)   ' -> Datenzeile
    PRINT GUI_TABLE_GET_CELL(tbl, r, 0)
NEXT
```

### Aussehen und Verhalten einstellen

Ein Setter mit Schlüsselwort statt einem Built-in je Schalter (wie bei
`chart`); ein unbekannter Schlüssel zählt die gültigen auf.

```basic
GUI_TABLE_SET(tbl, "zeilenhoehe", 38)    ' Bilder brauchen Platz
GUI_TABLE_SET(tbl, "kopfhoehe", 26)
GUI_TABLE_SET(tbl, "zebra", 1)
GUI_TABLE_SET(tbl, "gitter", 0)
GUI_TABLE_SET(tbl, "filterzeile", 1)
GUI_TABLE_SET(tbl, "sortierbar", 0)         ' Kopfklick sortiert nicht mehr
GUI_TABLE_SET(tbl, "spalten_ziehbar", 0)    ' Spaltenkante nicht mehr ziehbar
PRINT GUI_TABLE_GET(tbl, "spalten")         ' Spaltenzahl
```

**Spaltenbreiten mit der Maus:** die Kante zwischen zwei Kopfzellen lässt sich
ziehen (Fangbereich ±4 px). Beim ersten Zug übernimmt die Tabelle die bis dahin
errechneten Breiten als eigene Werte — sonst spränge sie beim Loslassen zurück.

### Speichern

`.gbform` bleibt lesbar: eine schlichte Textzelle wird weiterhin als **String**
geschrieben, nur eine mit Farbe/Art/Bild als Objekt. Beide Formen werden
gelesen, ältere Dateien laufen unverändert.

> **Zum Anschauen:** [examples/157_gui_tabelle.gb](../examples/157_gui_tabelle.gb)
> — Serverliste mit Ampel-Bildern, Auslastungsbalken, Favoriten-Haken und
> Aktionsknopf; sortierbar, filterbar, Spalten ziehbar.

**Nicht umgesetzt** (bewusst): Text in einer Zelle markieren,
Mehrfach-Auswahl, feste (mitscrollende) Spalten, Spalten umsortieren,
Zeilengruppen/Baum in der Tabelle.
- Optionaler `GUI_ON_CHANGE(tbl, funcref)`-Callback feuert bei Selektionswechsel.
- Farben folgen dem Theme (`GUI_SET_COLOR(tbl, ...)` überschreibt pro Widget: bg/fg/border/accent).

```basic
GUI_UPDATE()
GUI_DRAW()
IF GUI_TABLE_CLICKED(tbl) >= 0 THEN
    PRINT "Zeile " + STR$(GUI_TABLE_SELECTED(tbl)) + " gewaehlt"
END IF
```

**Beispiel:** [examples/81_table_select.gb](../examples/81_table_select.gb) zeigt
beide Tabellen (Retained `gui` + Immediate `ui`) nebeneinander.

## Baum (Tree-View)

```basic
DIM tree AS GUI_WIDGET : tree = GUI_TREE(win, 20, 20, 300, 380)
DIM proj AS INTEGER : proj = GUI_TREE_ADD(tree, -1, "Mein Spiel")   ' Wurzel
DIM src  AS INTEGER : src  = GUI_TREE_ADD(tree, proj, "src")        ' Kind von proj
GUI_TREE_ADD(tree, src, "main.gb")
GUI_TREE_EXPAND(tree, proj, TRUE)                                   ' aufgeklappt starten

' jeden Frame:
DIM sel AS INTEGER : sel = GUI_TREE_SELECTED(tree)
IF sel >= 0 THEN PRINT GUI_TREE_LABEL(tree, sel)
```

`GUI_TREE_ADD(tree, parent, label$)` hängt einen Knoten an und liefert seine
**id** (eine ganze Zahl, stabil bis `GUI_TREE_CLEAR`). `parent` ist `-1` für einen
Wurzelknoten, sonst die id eines vorhandenen Knotens — so entsteht die Hierarchie.
Ein Klick auf das Dreieck links klappt einen Knoten auf/zu (oder per
`GUI_TREE_EXPAND`), ein Klick auf die Zeile wählt ihn aus. `GUI_TREE_SELECTED`
liefert die id des gewählten Knotens (`-1` = keiner), `GUI_TREE_LABEL` dessen
Text. `GUI_ON_CHANGE(tree, …)` feuert bei Auswahländerung. Das Mausrad scrollt
lange Bäume. Beispiel: [examples/137_gui_tree.gb](../examples/137_gui_tree.gb).

## Aussehen ändern (Theme, Metriken, Per-Widget)

Das Aussehen lässt sich auf drei Ebenen steuern:

### 1. Fertige Farbschemata

```basic
GUI_THEME_PRESET("dark")          ' Default (Logo-Cyan), flacher Look
GUI_THEME_PRESET("light")         ' helles Schema, flach
GUI_THEME_PRESET("retro")         ' grün auf schwarz (Terminal)
GUI_THEME_PRESET("contrast")      ' schwarz/gelb, maximaler Kontrast
GUI_THEME_PRESET("modern_dark")   ' PROFESSIONELL: Anthrazit + Cyan, runde Ecken + Schatten
GUI_THEME_PRESET("modern_light")  ' PROFESSIONELL: hell, Windows-11-nah, runde Ecken + Schatten
```

Die beiden **`modern_*`**-Presets sind ein *kompletter* Look: sie setzen nicht nur
die Farben, sondern auch die Metriken (`corner_radius`, `title_h`, `pad`, `shadow`)
— so bekommst du **runde Ecken, einen weichen Fenster-Schatten, eine höhere
Titelleiste und Häkchen-Checkboxen**. Die klassischen Presets (`dark`/`light`/…)
setzen die Metriken auf den flachen Default zurück. Komplettes Beispiel mit
Live-Umschalter: [`examples/128_gui_modern.gb`](../examples/128_gui_modern.gb).

### 2. Einzelne Theme-Farben (global)

```basic
GUI_THEME_SET(schluessel$, farbe)   ' eine Palette-Farbe setzen
GUI_THEME_GET(schluessel$)          ' -> aktuelle Farbe (INTEGER)
GUI_THEME(accent)                   ' Kurzform für GUI_THEME_SET("accent", ...)
```

Gültige Schlüssel: `win_bg`, `win_border`, `title_bg`, `title_bg_focus`,
`title_fg`, `widget_bg`, `widget_border`, `text_fg`, `muted_fg`, `accent`,
`close_hover`.

```basic
GUI_THEME_SET("win_bg", RGB(20, 20, 30))
GUI_THEME_SET("accent", RGB(255, 160, 60))
```

### 3. Layout-Metriken (Größen)

```basic
GUI_METRIC_SET(schluessel$, wert)   ' eine Metrik setzen (INTEGER-Pixel)
GUI_METRIC_GET(schluessel$)         ' -> aktueller Wert
```

Schlüssel: `title_h` (Titelleisten-Höhe), `slider_h`, `check_size`,
`slider_handle_w`, `caret_period` (Cursor-Blink), `pad` (Text-Innenabstand),
`corner_radius` (runde Ecken für Fenster/Titelleiste/Buttons/TextInput/Dropdown/
Progress/Panel + Häkchen-Checkbox; 0 = eckig/flach), `shadow` (weicher
Fenster-Schatten in Pixeln; 0 = aus). **Hinweis:** Größen, die in die Widget-Maße
einfließen (`check_size`, `slider_h`), wirken nur auf **neu angelegte** Widgets —
am besten vor dem UI-Aufbau setzen. `title_h`/`pad`/`caret_period`/`corner_radius`/
`shadow` wirken sofort.

### 4. Einzelnes Widget einfärben (überschreibt das Theme)

```basic
GUI_SET_COLOR(widget, rolle$, farbe)   ' rolle: "bg" / "fg" / "border" / "accent"
GUI_SET_COLOR(widget, rolle$, -1)      ' Override entfernen (zurück zum Theme)
```

```basic
DIM warn AS GUI_WIDGET
warn = GUI_BUTTON(win, "Löschen", 20, 100, 100, 30)
GUI_SET_COLOR(warn, "bg", RGB(160, 40, 40))   ' nur dieser Button ist rot
```

`GUI_RESET()` setzt Theme **und** Metriken wieder auf die Defaults zurück
(und löscht alle Fenster/Widgets).

### 5. Zustand & Schrift pro Widget

```basic
GUI_SET_ENABLED(widget, FALSE)   ' deaktivieren: ausgegraut + nicht interaktiv
GUI_ENABLED(widget)              ' -> BOOLEAN
GUI_SET_FONT(widget, font)       ' eigener TTF-Font (Handle aus LOADFONT, -1 = Default)
GUI_SET_FONT_SIZE(widget, px)    ' eigene Textgröße (0 = Standard)
```

Ein **deaktiviertes** Widget wird in `muted_fg` gezeichnet und nimmt keine
Maus-/Tastatur-Eingaben an; im Editor bleibt es per `GUI_HIT_TEST` aber
selektierbar. Eigene Fonts:

```basic
DIM fnt AS INTEGER
fnt = LOADFONT("assets/Inter.ttf", 32)
DIM titel AS GUI_WIDGET
titel = GUI_LABEL(win, "Einstellungen", 20, 16)
GUI_SET_FONT(titel, fnt)
GUI_SET_FONT_SIZE(titel, 28)
```

### 6. Benannte Styles (Stylesheet)

Einen Style einmal definieren und auf viele Widgets anwenden — spart das
wiederholte `GUI_SET_COLOR`/`GUI_SET_FONT`:

```basic
GUI_STYLE_SET("primary", "bg", RGB(32, 80, 192))   ' props: bg/fg/border/accent
GUI_STYLE_SET("primary", "fg", &HFFFFFF)            '        + font / font_size
GUI_STYLE_SET("primary", "font_size", 18)

GUI_APPLY_STYLE(okBtn, "primary")
GUI_APPLY_STYLE(saveBtn, "primary")
```

`GUI_APPLY_STYLE` überträgt die Style-Properties als Per-Widget-Overrides
(Farben) bzw. Font/Größe. Inkrementell erweiterbar; `GUI_RESET()` löscht auch
die Styles.

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

Intern überbrückt `GUI_ON_CLICK` die Builtin→VM-Grenze (eine FUNCREF aus einem
Built-in heraus aufrufen) — nativ in `gbrt`.

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

## Laufzeit-Manipulation (Geometrie / Lifecycle / Hit-Test)

Widgets und Fenster lassen sich nach dem Anlegen zur Laufzeit verändern — die
Basis für **dynamische UIs** und einen **WYSIWYG-Editor**. Handles bleiben dabei
stabil (Löschen markiert nur als „tot", verschiebt keine Indizes).

| Builtin | Wirkung |
|---|---|
| `GUI_SET_BOUNDS(wdg, x, y, w, h)` | Widget verschieben/skalieren (fenster-relativ) |
| `GUI_GET_X/Y/W/H(wdg)` → INTEGER | aktuelle Maße lesen |
| `GUI_SET_VISIBLE(wdg, an)` / `GUI_VISIBLE(wdg)` → BOOLEAN | ein-/ausblenden (unsichtbar = nicht gezeichnet, nicht interaktiv) |
| `GUI_DESTROY(wdg)` | Widget entfernen (Handle wird ungültig, andere bleiben gültig) |
| `GUI_KIND(wdg)` → STRING | `"button"`/`"label"`/`"checkbox"`/`"slider"`/`"textinput"`/`"panel"`/`"table"` |
| `GUI_FOCUS(wdg)` | Tastatur-Fokus setzen (z.B. auf ein TextInput) |
| `GUI_HIT_TEST(x, y)` → GUI_WIDGET | oberstes Widget am Bildschirmpunkt, oder `-1` (Selektion im Editor) |
| `GUI_WINDOW_SET_BOUNDS(win, x, y, w, h)` / `GUI_WINDOW_GET_X/Y/W/H(win)` | Fenster bewegen/skalieren/lesen |
| `GUI_WINDOW_DESTROY(win)` | Fenster + Inhalt entfernen |
| `GUI_WINDOW_WIDGET_COUNT(win)` → INTEGER | Anzahl lebender Widgets |
| `GUI_WINDOW_WIDGET(win, n)` → GUI_WIDGET | n-tes lebendes Widget (Enumeration/Serialisierung), oder `-1` |

```basic
' Editor-Idee: Klick wählt das Widget unter der Maus aus, Ziehen verschiebt es.
DIM sel AS GUI_WIDGET
sel = GUI_HIT_TEST(MOUSEX(), MOUSEY())
IF sel <> -1 THEN
    GUI_SET_BOUNDS(sel, MOUSEX() - win_x, MOUSEY() - win_y - 22, GUI_GET_W(sel), GUI_GET_H(sel))
END IF

' Alle Widgets eines Fensters durchgehen (z.B. zum Speichern):
DIM i AS INTEGER
FOR i = 0 TO GUI_WINDOW_WIDGET_COUNT(win) - 1
    DIM wdg AS GUI_WIDGET
    wdg = GUI_WINDOW_WIDGET(win, i)
    PRINT GUI_KIND(wdg), GUI_GET_X(wdg), GUI_GET_Y(wdg)
NEXT
```

## Serialisierung (Layout als JSON)

Ein Fenster lässt sich mitsamt seiner Widgets als JSON speichern und wieder
laden — das **schließt den Kreis zwischen einem (WYSIWYG-)Editor und der
Laufzeit**: der Editor schreibt JSON, die App lädt es (wie `ATLAS_LOAD`/
`TILED_LOAD`). Erhalten bleiben Maße, Texte, Zustände (checked/value), Farb-
Overrides, Fenster-Flags und Tabellen-Daten. Zerstörte Widgets werden nicht
mitgespeichert.

| Builtin | Wirkung |
|---|---|
| `GUI_SAVE(win, pfad$)` | Fenster als JSON-Datei speichern |
| `GUI_LOAD(pfad$)` → GUI_WINDOW | JSON-Datei laden, neues Fenster, Handle zurück |
| `GUI_TO_JSON(win)` → STRING | Fenster als JSON-String (für Netzwerk/Embedding/`json`-Modul) |
| `GUI_FROM_JSON(json$)` → GUI_WINDOW | aus JSON-String ein Fenster bauen |

```basic
' Design speichern ...
GUI_SAVE(win, "forms/login.json")

' ... und in der App wieder aufbauen:
DIM win AS GUI_WINDOW
win = GUI_LOAD("forms/login.json")
' Callbacks/Handler werden NICHT mitgeladen -> hier neu verdrahten
' (FUNCREF = blanker Funktionsname):
GUI_ON_CLICK(GUI_WINDOW_WIDGET(win, 0), on_login)
```

> Hinweis: `GUI_ON_CLICK`/`ON_CHANGE` speichern den Funktions**namen** mit; beim
> Laden in einem anderen Programm muss die Funktion existieren. Für portable
> Designs die Handler nach dem Laden per Enumeration (`GUI_WINDOW_WIDGET`) setzen.

### Formular-Workflow (Xojo-Stil)

Weil das `.gbform` pro Control den **Namen seines Event-Handlers** mitspeichert
(`on_click` / `on_change`) und `GUI_UPDATE` ausgelöste Handler **automatisch per
Name aufruft**, ergibt sich der Xojo-Ablauf von selbst: Formular laden, nur die
Handler ausfüllen — kein manuelles Verdrahten.

```basic
IMPORT "gui"
SCREEN(800, 480, "App", 1)

DIM frm AS GUI_WINDOW
frm = GUI_LOAD("forms/settings.gbform")   ' Controls + Handler-Namen

SUB on_save()                              ' du schreibst NUR die Handler ...
    PRINT "Speichern geklickt"
END SUB

WHILE NOT QUITREQUESTED()
    GUI_UPDATE()                           ' ... GUI_UPDATE ruft on_save automatisch
    CLS(0) : GUI_DRAW() : FLIP()
WEND
```

Vollständiges Beispiel (Formular `examples/forms/settings.gbform` +
[examples/105_form_runner.gb](../examples/105_form_runner.gb)): ein
Einstellungs-Dialog mit TextInput, Checkbox, Slider, Dropdown und zwei Buttons,
dessen Handler die Control-Werte auslesen. Das `.gbform` lässt sich von Hand
schreiben **oder** (künftig) von einem visuellen Designer erzeugen — beides
ergibt dieselbe JSON.

## Vollständiges Beispiel

Siehe [examples/45_gui.gb](../examples/45_gui.gb): Fenster mit Slider (live ins
Label gespiegelt), Checkbox, Textfeld und Start-Button — verschiebbar und
schließbar.

**Alle 22 Widget-Arten in einer Anwendung:**
[examples/156_gui_alle_widgets.gb](../examples/156_gui_alle_widgets.gb) —
Vollbild, randloses Fenster (die Form *ist* der Bildschirm), Menüleiste,
Werkzeugleiste, Kontextmenü und drei Reiter. Kein Schaukasten: jedes Widget
hat eine Aufgabe. Der Baum filtert die Tabelle, eine Tabellenzeile füllt den
Editor, „Übernehmen" schreibt zurück; auf dem zweiten Reiter formen
Drehknöpfe, Schieber, Radios und Schalter live die Kurve auf der
Zeichenfläche, die Liste lädt Voreinstellungen; der dritte Reiter führt
Kennzahlen aus den echten Daten mit. Über das Menü lassen sich alle vier
Themen umschalten.

## Limitationen (Stand jetzt)

- **Events**: Polling (`GUI_CLICKED` …) **und** FUNCREF-Callbacks
  (`GUI_ON_CLICK` für Buttons/Checkboxen, `GUI_ON_CHANGE` für Slider/TextInput/
  Checkbox) werden unterstützt.
- **Immediate-Mode-Fenster** (`UI_WINDOW_BEGIN/END` im `ui`-Modul) sind die
  geplante Alternative (Phase 4).
- **Absolute Koordinaten**, kein Auto-Layout.
- **Laufzeit-Manipulation** (Verschieben/Skalieren/Löschen/Ein-/Ausblenden/
  Hit-Test/Enumeration) wird unterstützt — siehe Abschnitt oben.
- **Headless/grafisch**: `GUI_UPDATE`/`GUI_DRAW` brauchen einen aktiven
  `SCREEN`. Konstruktion, State, Geometrie und Hit-Test sind headless getestet
  (`tests/test_gui_runtime.py`).


## Plastischer Look: die Glas-Themen

> Zum Anschauen: [examples/155_gui_glas.gb](../examples/155_gui_glas.gb) —
> `f` schaltet dort zwischen flach und plastisch um, `g` legt eine eigene
> Grafik auf die Knöpfe.

`GUI_THEME_PRESET("glas_dunkel")` bzw. `"glas_hell"` schalten einen
gewölbten Look ein: senkrechter Verlauf auf jeder Fläche, Glanzkante über der
oberen Hälfte, feine Fase oben und unten.

Das steckt in drei **Metriken**, nicht in Farben — so ist ein Thema ein
kompletter Look statt zweier Dinge, die man von Hand kombinieren muss:

| Metrik | Bedeutung |
|---|---|
| `gradient` | Helligkeitsabstand oben/unten; 0 = flach |
| `gloss` | Stärke der Glanzkante, 0…100 |
| `bevel` | 1 = helle Linie oben, dunkle unten |

Einzeln setzbar über `GUI_METRIC_SET`. Alle **bestehenden** Themen stehen
weiterhin auf 0 — schon geschriebene Programme sehen unverändert aus.

**Erhaben und versenkt.** Knöpfe, Panels und Auswahlfelder sind erhaben
(hell oben → dunkel unten, mit Glanz). Eingabefelder, Listen und der
Fortschritts-Trog sind versenkt: der Verlauf läuft andersherum und statt der
Glanzkante liegt ein Schatten unter dem oberen Rand. Ohne diesen Unterschied
sieht ein Eingabefeld aus wie ein Knopf, und die Oberfläche verliert ihre
Aussage darüber, was man anklickt und was man ausfüllt.


## Kippschalter, Drehregler, runde Knöpfe

Drei Bedienelemente, die den Glas-Themen ihre Wirkung geben:

```basic
DIM t AS GUI_WIDGET
DIM k AS GUI_WIDGET
DIM b AS GUI_WIDGET

t = GUI_TOGGLE(w, "Musik", 24, 20, TRUE)          ' An/Aus-Pille
k = GUI_KNOB(w, 220, 20, 90, 0.0, 100.0, 72.0)    ' Drehregler
b = GUI_BUTTON(w, ">", 24, 285, 42, 42)
GUI_SET_ROUND(b, TRUE)                            ' runder Transport-Knopf
```

**`GUI_TOGGLE(win, text$, x, y [, an])`** — der Zustand liegt wie beim
Kästchen in `checked`, also lesbar mit `GUI_CHECKED` und setzbar mit
`GUI_SET_CHECKED`. Der Knopf gleitet beim Umlegen hinüber und die Rinne färbt
sich mit; beides läuft über einen inneren Wert, nicht sprunghaft.

**`GUI_KNOB(win, x, y, groesse, min, max [, wert])`** — verstellt durch
**senkrechtes Ziehen** (nach oben = mehr). Das ist die Bedienung von
Mischpult-Oberflächen und kommt ohne Kreisbewegung der Maus aus; 140 Pixel
entsprechen dem ganzen Bereich. Wert über `GUI_VALUE` / `GUI_SET_VALUE`.
Der Wertbogen umläuft die Metallkappe über 270°, die Kerbe zeigt die Stellung.

**`GUI_SET_ROUND(widget, an)`** — zeichnet einen Knopf rund statt eckig. Für
Knöpfe, die nur ein Sinnbild tragen: ein runder Knopf mit Dreieck darin liest
sich sofort als Abspieltaste.

Kästchen, Auswahlknöpfe und Schieber sind ebenfalls plastisch: das leere
Kästchen ist eine versenkte Mulde, das gesetzte eine gewölbte Fläche in der
Akzentfarbe; der Schieber hat eine versenkte Rinne, deren zurückgelegter Teil
eingefärbt wird, und einen metallischen Griff.


## Eigene Grafiken: 9-Slice-Skins

Wo der gezeichnete Look nicht reicht, ersetzt eine Grafik die Fläche eines
Widget-Typs:

```basic
DIM haut AS IMAGE
haut = LOADIMAGE("assets/knopf.png")
GUI_SKIN("button", haut, 12)      ' 12 px Rand
GUI_SKIN("button", -1)            ' wieder wegnehmen
```

`GUI_SKIN(art$, bild, rand)` gilt für **alle** Widgets dieser Art. Der Rest —
Beschriftung, Häkchen, Schieberegler-Griff — wird weiterhin gezeichnet; nur
der Untergrund kommt aus dem Bild.

**Warum 9-Slice und nicht einfach skalieren?** Das Bild wird in neun Stücke
geteilt: die vier Ecken bleiben unverändert, die Kanten dehnen sich nur
entlang ihrer Achse, die Mitte in beide Richtungen. Ein schlicht skaliertes
Bild würde seine runden Ecken zu Ellipsen ziehen, sobald der Knopf breiter
wird als die Vorlage.

Der Rand wird auf die halbe Bild- **und** Zielseite gedeckelt. Ein Widget, das
kleiner ist als seine Skin-Ränder, schrumpft dadurch sauber zusammen, statt
sich zu überlappen.

Zulässige Arten sind alle Widget-Namen, die `GUI_KIND` liefert (`button`,
`panel`, `textinput`, `listbox`, `progress`, …).


## Schrift je Widget

Global gilt die per `SETFONT` gesetzte Schrift. Einzelne Widgets können davon
abweichen:

```basic
DIM k AS GUI_WIDGET
k = GUI_BUTTON(w, "Gross", 20, 20, 180, 40)
GUI_SET_FONT_SIZE(k, 24)          ' nur die Groesse
GUI_SET_FONT(k, andere_schrift)   ' andere Schriftart (Handle aus LOADFONT)
```

`GUI_SET_FONT_SIZE` ändert **nur** die Größe — die Schriftart bleibt die
global gesetzte. `GUI_SET_FONT(wdg, -1)` nimmt eine eigene Schrift wieder
zurück.

Über `GUI_STYLE_SET` / `GUI_APPLY_STYLE` lassen sich Schrift und Größe auch
als benannter Stil auf mehrere Widgets übertragen.

**Was daran wichtig ist:** Die Beschriftung wird in der Schrift *gemessen*, in
der sie auch gezeichnet wird. Andernfalls säße zentrierter Text schief und der
Beschnitt griffe an der falschen Stelle, sobald ein Widget eine eigene Schrift
oder Größe hat.


## Der Glas-Look im `ui`-Modul

Das Immediate-Mode-Modul `ui` kennt dieselben zwei Themen:

```basic
IMPORT "ui"
UI_THEME_PRESET("glas_dunkel")     ' oder "glas_hell"
```

Es bringt dieselben Metriken mit (`gradient`, `gloss`, `bevel`,
`corner_radius`, einzeln über `UI_METRIC_SET`), und ein Preset setzt Farben
**und** Plastik — wer von einem Glas-Thema auf ein flaches wechselt, behält
also keine Wölbung zurück.

Umgestellt sind Knopf, Kästchen, Schieber, Fortschritt, Eingabefeld, Panel
und Fenster. Die Knopf-Beschriftung sitzt hier ebenfalls mittig und wird im
Knopf abgeschnitten statt überzulaufen.

Alle bisherigen `ui`-Themen (`dark`, `light`, `retro`, `contrast`) stehen
weiterhin auf 0 und sehen unverändert flach aus.


## Ereignisse

Sechs Rückrufe je Widget, alle über FUNCREF (parameterlos):

| Befehl | Wann |
|---|---|
| `GUI_ON_CLICK(w, fn)` | angeklickt |
| `GUI_ON_CHANGE(w, fn)` | Wert/Text geändert |
| `GUI_ON_HOVER(w, fn)` | Maus **betritt** das Widget |
| `GUI_ON_LEAVE(w, fn)` | Maus verlässt es |
| `GUI_ON_FOCUS(w, fn)` | bekommt die Eingabe |
| `GUI_ON_BLUR(w, fn)` | verliert sie — der Punkt, an dem man eine Eingabe prüft |

Die letzten vier sind **Flanken**: sie feuern beim Übergang, nicht in jedem
Bild, solange der Zustand anhält. Ausgelöst werden sie in `GUI_UPDATE`;
aufgerufen wird der Handler danach, damit er die Oberfläche nicht mitten im
Zustands-Update umbauen kann.

**Fokus bekommen nur Widgets, die ihn auch führen** — Textfeld, Textbereich
und Zahlenfeld (oder programmatisch über `GUI_FOCUS`). Bei einem Knopf würde
`on_focus` nie von selbst feuern; der Form-Designer bietet es dort deshalb
gar nicht erst an.

Alle sechs überleben `GUI_SAVE`/`GUI_LOAD` bzw. `GUI_TO_JSON`/`GUI_FROM_JSON`
— das ist der Weg, über den ein im Form-Designer gebautes Formular seine
Handler bekommt.
