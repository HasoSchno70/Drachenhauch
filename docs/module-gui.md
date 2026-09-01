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
| `GUI_TEXTAREA(win, x, y, w, h[, platzhalter$])` | GUI_WIDGET | **mehrzeiliges** Textfeld (ENTER = neue Zeile, scrollt senkrecht und waagerecht, Pfeile, Selektion via Maus-Drag/Shift+Pfeil, Strg+A/C/X/V). Als **Code-Feld** einfärbbar — siehe [Code-Feld](#das-textarea-als-code-feld) |
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
| `GUI_DRAW([obenauf])` | — | alle Fenster zeichnen (hinten→vorne); `obenauf`=FALSE lässt Kontextmenü und Tooltip weg |
| `GUI_DRAW_TOP()` | — | nur was über allen Fenstern liegt: offenes Kontextmenü und Tooltip |
| `GUI_DRAW_WINDOW(win)` | — | EIN Fenster noch einmal zeichnen, über allem, was seit `GUI_DRAW` dazugekommen ist |
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

Klick-Auswertung wie bei Buttons über `GUI_CLICKED(item)`. Die Menüleiste schiebt den Fensterinhalt automatisch nach unten; Klick auf ein Menü öffnet das Dropdown, Klick daneben schließt es. Komplettes Beispiel: [`examples/129_gui_menu.dh`](../examples/129_gui_menu.dh).

### Reiter (Tabs) + Tastatur-Navigation

| Funktion | Rückgabe | Zweck |
|---|---|---|
| `GUI_TABS(win, labels)` | — | Reiter-Leiste anlegen (`labels` = ARRAY/TUPLE von Strings); aktiver Reiter → 0 |
| `GUI_SET_TAB(widget, seite)` | — | Widget einem Reiter zuordnen (`-1` = auf allen Reitern sichtbar) |
| `GUI_ACTIVE_TAB(win)` | INTEGER | aktiver Reiter-Index |
| `GUI_SET_ACTIVE_TAB(win, i)` | — | Reiter umschalten |

Nur die Widgets des aktiven Reiters (plus die mit `tab_page = -1`) werden gezeichnet und sind bedienbar. **Tastatur:** `TAB` / `SHIFT+TAB` wechselt den Fokus zwischen **allen bedienbaren Widgets** des aktiven Fensters — siehe [Bedienung ohne Maus](#bedienung-ohne-maus). Beispiel: [`examples/131_gui_tabs.dh`](../examples/131_gui_tabs.dh).

### Modale Dialoge

Native, blockierende Standarddialoge (kein IMPORT nötig — wie die Datei-Dialoge):

| Funktion | Rückgabe | Zweck |
|---|---|---|
| `GUI_MESSAGE(titel$, text$)` | — | Info-Box mit OK |
| `GUI_CONFIRM(titel$, text$[, stil$])` | BOOLEAN | Rückfrage → `TRUE` bei Zustimmung |
| `GUI_RADIO(win, group$, text$, x, y)` | GUI_WIDGET | Auswahlknopf; alle mit derselben `group$` schliessen sich gegenseitig aus |
| `GUI_RADIO_SELECTED(radio)` | INTEGER | welcher der Gruppe ist gewaehlt? (Erstellungsreihenfolge ab 0, `-1` = keiner) |
| `GUI_DROPDOWN(win, x, y, w, h, items)` | GUI_WIDGET | aufklappende Auswahlliste; das Popup wird ueber allen anderen Widgets gezeichnet |
| `GUI_DROPDOWN_SELECTED(dd)` | INTEGER | Index der Auswahl (`-1` = keine) |
| `GUI_DROPDOWN_TEXT(dd)` | STRING | Text der Auswahl |
| `GUI_DROPDOWN_SET_SELECTED(dd, i)` | — | Auswahl vom Programm aus setzen |
| `GUI_SET_DROPDOWN(dd, items)` | — | Eintraege ersetzen |
| `GUI_LISTBOX(win, x, y, w, h, items)` | GUI_WIDGET | scrollbare Auswahlliste (Mausrad scrollt) |
| `GUI_LISTBOX_SELECTED(lb)` | INTEGER | Index der Auswahl (`-1` = keine) |
| `GUI_LISTBOX_TEXT(lb)` | STRING | Text der Auswahl |
| `GUI_LISTBOX_SET_SELECTED(lb, i)` | — | Auswahl vom Programm aus setzen |
| `GUI_SET_LISTBOX(lb, items)` | — | Eintraege ersetzen |
| `GUI_IMAGE(win, x, y, w, h, image)` | GUI_WIDGET | Bild oder Symbol im Fenster |
| `GUI_SET_IMAGE(widget, image)` | — | Bild austauschen |
| `GUI_CANVAS(win, x, y, w, h)` | GUI_WIDGET | freie Zeichenflaeche -- hinein malt man mit den normalen Zeichenbefehlen, **nach** `GUI_DRAW` |
| `GUI_CANVAS_X(canvas)` / `GUI_CANVAS_Y(canvas)` | INTEGER | **absolute** Bildschirmposition der Flaeche (wandert mit dem Fenster) |
| `GUI_CANVAS_W(canvas)` / `GUI_CANVAS_H(canvas)` | INTEGER | Groesse der Flaeche |
| `GUI_SET_ENABLED(wdg, on)` | — | Widget bedienbar machen oder sperren (gesperrt = ausgegraut, nimmt keine Klicks) |
| `GUI_ENABLED(wdg)` | BOOLEAN | ist es bedienbar? |
| `GUI_SET_FONT(wdg, font)` | — | eigene Schrift fuer dieses Widget |
| `GUI_SET_FONT_SIZE(wdg, px)` | — | eigene Schriftgroesse fuer dieses Widget |
| `GUI_STYLE_SET(name$, prop$, wert)` | — | benannten Stil festlegen (`bg`, `fg`, `border`, `accent`, `font`, `font_size`) |
| `GUI_APPLY_STYLE(widget, name$)` | — | einen benannten Stil auf ein Widget uebertragen -- spart, ihn Widget fuer Widget zu wiederholen |
| `GUI_GET_Y(wdg)` / `GUI_GET_W(wdg)` / `GUI_GET_H(wdg)` | INTEGER | Lage und Groesse des Widgets im Fenster (Gegenstueck zu `GUI_SET_BOUNDS`) |
| `GUI_WINDOW_GET_Y(win)` / `GUI_WINDOW_GET_W(win)` / `GUI_WINDOW_GET_H(win)` | INTEGER | Lage und Groesse des Fensters auf dem Bildschirm |
| `GUI_TABLE_SET(tbl, schluessel$, wert)` | — | Einstellung der Tabelle setzen (`zebra`, `gitter`, `zeilenhoehe`, `filterzeile`, `feste_spalten`, ...) |
| `GUI_TABLE_GET(tbl, schluessel$)` | FLOAT | eine dieser Einstellungen zurueckreisen |
| `GUI_TABLE_CLICKED_COL(tbl)` | INTEGER | welche Spalte wurde angeklickt? -- fuer Zellen der Art `knopf` |
| `GUI_TABLE_VIEW_COUNT(tbl)` | INTEGER | wie viele Zeilen sind gerade SICHTBAR? (nach Filtern) |
| `GUI_TABLE_VIEW_ROW(tbl, i)` | INTEGER | welche **Datenzeile** steht an sichtbarer Stelle `i`? -- Sortieren und Filtern stellen die Daten nicht um |

`stil$` beschriftet die Knöpfe: `"ok"` (Vorgabe) zeigt **OK/Abbrechen**,
`"janein"` zeigt **Ja/Nein**. Der Unterschied ist nicht kosmetisch — bei einer
Frage liest sich „Abbrechen" als „Dialog schließen", „Nein" als Antwort.
Faustregel: Anweisung („Löschen") → `"ok"`, Frage („Wirklich löschen?") → `"janein"`.

```basic
IF GUI_CONFIRM("Löschen", "Alle Einträge werden entfernt.") THEN GUI_SET_TEXT(ta, "")

IF GUI_CONFIRM("Sicherung einspielen?", "Alles seitdem geht verloren.", "janein") THEN
    GUI_MESSAGE("Fertig", "Sicherung eingespielt.")
END IF
```

Beide Dialoge **blockieren**, bis geantwortet wurde — das Fenster dahinter
zeichnet solange nicht. Für eine Rückfrage ist das richtig: niemand soll
weiterklicken können, während die Frage offen ist.

Beispiel mit TextArea + Dialogen: [`examples/132_gui_textarea.dh`](../examples/132_gui_textarea.dh).

#### `GUI_DIALOG` — derselbe Dialog, aber im eigenen Fenster

`GUI_MESSAGE`/`GUI_CONFIRM` öffnen einen **Kasten des Betriebssystems**. Das
ist die richtige Wahl für ein Werkzeug — er sieht aus wie alles andere auf
dem Rechner. Für ein Spiel im Vollbild ist er es oft nicht: er sprengt den
Look, erscheint als eigenes OS-Fenster, und im Web-Build gibt es ihn gar
nicht (`dialogs`-Feature).

`GUI_DIALOG` ist die Alternative **innerhalb** deines Fensters: dein Thema,
dein Maßstab, alles hinter ihm wird abgedunkelt und nimmt keine Klicks mehr
an.

| Funktion | Rückgabe | Zweck |
|---|---|---|
| `GUI_DIALOG(titel$, text$[, stil$])` | GUI_WINDOW | modalen Dialog öffnen (`"ok"` = Vorgabe, `"janein"`) |
| `GUI_ANSWER(dialog)` | INTEGER | `0` = noch offen, `1` = OK/Ja, `2` = Abbrechen/Nein |
| `GUI_MODAL()` | BOOLEAN | steht gerade ein Dialog? |

`\n` im Text (`CHR$(10)`) trennt Zeilen; das Fenster passt sich Text und
Zeilenzahl an und wird auf dem Bildschirm zentriert.

```basic
DIM frage AS GUI_WINDOW
frage = -1                  ' -1 = gerade keiner offen

' ... im Spielablauf:
IF GUI_CLICKED(loeschen) AND frage < 0 THEN
    frage = GUI_DIALOG("Löschen", "Eintrag wirklich löschen?", "janein")
END IF

GUI_UPDATE()
IF frage >= 0 THEN
    IF GUI_ANSWER(frage) = 1 THEN eintrag_loeschen() : frage = -1
    IF GUI_ANSWER(frage) = 2 THEN frage = -1
END IF
```

> **Merk dir den offenen Dialog mit `-1` als „keiner".** Nicht mit `0` —
> Fenster-Handles zählen ab 0, und ausgerechnet die `0` ist das **erste
> Fenster deines Programms**. `GUI_ANSWER` auf einen negativen Handle
> liefert darum `0` („keine Antwort"), damit die Abfrage auch dann
> durchläuft, wenn gar kein Dialog steht.

**`GUI_DIALOG` blockiert nicht.** Es gibt dir ein Fenster zurück, und die
Antwort holst du dir mit `GUI_ANSWER` — genau wie einen Klick mit
`GUI_CLICKED`. Das ist kein Kompromiss, sondern die Bauweise des Moduls: ein
blockierender Dialog müsste mitten in deinem Bild eine eigene Zeichenschleife
drehen und dabei deinen Layer- und Render-Ziel-Zustand übernehmen.

> **Die Antwort gilt genau ein Bild** — wie `GUI_CLICKED`. Eine Antwort ist
> ein Ereignis, kein Zustand. Wer sie länger braucht, schreibt sie sich in
> eine Variable. Danach ist das Dialogfenster weg; sein Handle bleibt gültig,
> `GUI_ANSWER` liefert dann `0`.

**Wann welchen?**

| | `GUI_MESSAGE` / `GUI_CONFIRM` | `GUI_DIALOG` |
|---|---|---|
| Aussehen | Kasten des Betriebssystems | dein Thema, dein Maßstab |
| Ablauf | blockiert, liefert die Antwort direkt | läuft weiter, Antwort per `GUI_ANSWER` |
| Web-Build | nein | ja |
| passt zu | Werkzeug, Editor | Spiel, Vollbild-Anwendung |

Demo für alle drei Neuerungen (Tastatur, Maßstab, Dialog):
[`examples/182_gui_tastatur_massstab.dh`](../examples/182_gui_tastatur_massstab.dh).

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
GUI_ON_CHANGE(sp, wert_geaendert)                     ' FUNCREF = der nackte Name
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
Bereich.

### Ein Fenster über dem Gezeichneten: `GUI_DRAW_WINDOW`

Daraus folgt ein Fallstrick, den man erst sieht, wenn man ihn hat: **ein
Fenster, das über einer Zeichenfläche liegt, verschwindet hinter deren
Inhalt.** `GUI_DRAW` zeichnet Fenster und Zeichenflächen in einem Durchgang,
und der Inhalt einer Zeichenfläche entsteht per Bauart danach. Ein Dialog, der
mitten auf der Fläche aufgeht, ist damit unsichtbar — er sperrt die Eingabe und
ist nicht zu sehen, das Programm wirkt eingefroren.

Ein zweites `GUI_DRAW` hilft nicht: es zeichnet auch die Fenster-Hintergründe
und die Zeichenflächen neu, also genau das, was gerade gemalt wurde. Stattdessen
das eine Fenster am Ende des Bildes noch einmal:

```basic
GUI_UPDATE()
GUI_DRAW()
' ... hier malt das Programm in seine Zeichenflächen ...
IF dialogOffen THEN GUI_DRAW_WINDOW(dlg)    ' obenauf, zuletzt
FLIP()
```

Ein unsichtbares oder zerstörtes Fenster zeichnet nichts (kein Fehler) — der
Aufruf darf also unbedingt in der Bildschleife stehen. Ist das Fenster das
modale, kommt sein Schleier mit.

**Kontextmenü und Tooltip** liegen über *allen* Fenstern und sind deshalb vom
selben Problem betroffen — ein Tooltip folgt der Maus und landet also
regelmäßig über einer Zeichenfläche. Für sie gibt es `GUI_DRAW_TOP()`, und
`GUI_DRAW` lässt sie mit `GUI_DRAW(FALSE)` weg:

```basic
GUI_UPDATE()
GUI_DRAW(FALSE)                             ' Fenster, ohne die obenauf-Schicht
' ... hier malt das Programm in seine Zeichenflächen ...
IF dialogOffen THEN GUI_DRAW_WINDOW(dlg)
GUI_DRAW_TOP()                              ' Kontextmenü + Tooltip, zuletzt
FLIP()
```

Warum nicht einfach zweimal zeichnen: Tooltip und Kontextmenü haben einen
halbdurchsichtigen Schlagschatten. Zweimal übereinander ist er dunkler — und
zwar **nur dort, wo das eigene Zeichnen die erste Fassung nicht zugedeckt
hat**, also genau an einer Kante. Bei Fenstern (`GUI_DRAW_WINDOW`) bleibt das
ein bekannter Rest: liegt ein Fenster teilweise außerhalb des selbst
gezeichneten Bereichs, wird sein Glanz dort zweimal aufgetragen. Ein Dialog
mitten auf der Fläche ist davon nicht betroffen.

(Für echte Fenster-Überlappung/Occlusion ein Render-Target nutzen.)

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

### Feste Spalten

`GUI_TABLE_SET(tbl, "feste_spalten", 2)` lässt die ersten zwei Spalten beim
waagerechten Scrollen stehen — die Kennung bleibt sichtbar, während man nach
rechts durch die Daten wandert. **Sobald tatsächlich seitwärts gescrollt ist**,
markiert eine etwas kräftigere Trennlinie das Ende des festen Blocks; bei
stehender Tabelle bleibt sie weg, weil sie dort nichts erklären würde. Bewusst
nicht in der Akzentfarbe: die bedeutet überall sonst „ausgewählt/aktiv".

Alles, was Spalten verortet, geht über **eine** Quelle (`col_x` für die Lage,
`col_clip` für den sichtbaren Bereich) — Treffertest *und* Zeichnen benutzen
sie. Ein fester Block wäre sonst der sicherste Weg, beide auseinander laufen zu
lassen: man klickt auf „Name" und trifft die Spalte, die darunter
durchgescrollt ist.

Mehr feste Spalten anzugeben als vorhanden sind wird gedeckelt.

### Spalten umsortieren

Aus (ein versehentlicher Zug am Kopf würde sonst die Anordnung zerwürfeln, und
zurück geht es nur von Hand). Mit
`GUI_TABLE_SET(tbl, "spalten_verschiebbar", 1)` lässt sich eine **Kopfzelle
seitwärts ziehen**; die Spalte tauscht live den Platz, sobald sie über die
Mitte des Nachbarn kommt.

**Klick und Zug sind dieselbe Geste.** Erst beim Loslassen steht fest, was
gemeint war: ohne Bewegung wird sortiert, ab 5 px Bewegung wurde verschoben.
Ohne diese Unterscheidung würde jedes Verschieben nebenbei auch noch sortieren.

Auch hier gilt: **die Daten werden nicht umgestellt.** `col_order` bildet nur
Anzeige-Position → Datenspalte ab. `GUI_TABLE_GET_CELL(tbl, r, 0)` liefert nach
dem Verschieben immer noch dieselbe Spalte — eine Spaltennummer, die dein
Programm sich gemerkt hat, bleibt gültig.

| Built-in | Wirkung |
|---|---|
| `GUI_TABLE_MOVE_COL(tbl, von_pos, nach_pos)` | Spalte verschieben (Anzeige-Positionen) |
| `GUI_TABLE_COL_AT(tbl, pos)` | welche **Datenspalte** steht an dieser Position? |
| `GUI_TABLE_COL_POS(tbl, spalte)` | an welcher Position steht diese Datenspalte? |
| `GUI_TABLE_RESET_COLS(tbl)` | ursprüngliche Reihenfolge |

Feste Spalten zählen nach **Position**: schiebt man eine Spalte nach vorn, wird
sie mit fest. Die Reihenfolge wird im `.dhform` mitgespeichert.

### Mehrfach-Auswahl

Aus (dann verhält sich die Tabelle wie bisher: eine Zeile). Mit
`GUI_TABLE_SET(tbl, "mehrfachauswahl", 1)`:

- **Strg+Klick** nimmt eine Zeile dazu oder heraus
- **Umschalt+Klick** wählt den Bereich ab der zuletzt angeklickten — in der
  **sichtbaren** Reihenfolge, denn das ist es, was man zwischen beiden Klicks
  sieht (über die Datenzeilen zu gehen träfe bei sortierter Tabelle etwas ganz
  anderes)
- normaler Klick ersetzt die Auswahl

| Built-in | Wirkung |
|---|---|
| `GUI_TABLE_SEL_COUNT(tbl)` | Anzahl gewählter Zeilen |
| `GUI_TABLE_SEL_ROW(tbl, i)` | i-te gewählte **Datenzeile**, in Klick-Reihenfolge |
| `GUI_TABLE_IS_SELECTED(tbl, zeile)` | ist diese Datenzeile gewählt? |
| `GUI_TABLE_SELECT(tbl, zeile, an)` | Zeile dazunehmen / herausnehmen |
| `GUI_TABLE_CLEAR_SELECTION(tbl)` | Auswahl leeren |

`GUI_TABLE_SELECTED` liefert weiterhin die **zuletzt angeklickte** Zeile —
vorhandener Code, der nur diese kennt, läuft unverändert. Beim Einschalten wird
eine bestehende Einzelauswahl übernommen; beim Löschen einer Zeile rücken alle
Indizes dahinter nach (sonst zeigte die Auswahl danach auf andere Zeilen).

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

**Der Editor kann dasselbe wie ein `TextInput`** — beide benutzen dieselbe
Routine, es gibt die Logik also nur einmal: Schreibmarke, Pfeiltasten,
`Pos1`/`Ende`, `Rücktaste`, `Entf`, `Strg+A/C/V/X`, **Text markieren** (mit
Umschalt+Navigation oder durch Ziehen mit der Maus im Feld), und er schiebt den
Text mit, wenn er länger ist als die Zelle.

Beim Öffnen ist der **ganze Inhalt markiert** — das erste Tippen ersetzt ihn
also, statt ihn zu verlängern. Genau das erwartet man, wenn man eine Zelle zum
Überschreiben aufmacht. (Der öffnende Doppelklick selbst setzt die Marke
deshalb *nicht*; erst nach dem Loslassen hört der Editor auf die Maus. Sonst
hätte derselbe Klick die Markierung sofort wieder aufgehoben.)

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

`.dhform` bleibt lesbar: eine schlichte Textzelle wird weiterhin als **String**
geschrieben, nur eine mit Farbe/Art/Bild als Objekt. Beide Formen werden
gelesen, ältere Dateien laufen unverändert.

> **Zum Anschauen:** [examples/157_gui_tabelle.dh](../examples/157_gui_tabelle.dh)
> — Serverliste mit Ampel-Bildern, Auslastungsbalken, Favoriten-Haken und
> Aktionsknopf; sortierbar, filterbar, Spalten ziehbar.

### An eine Datenbank hängen

Das Muster ist immer dasselbe: **die Datenbank ist die Wahrheit, die Tabelle
ihre Ansicht.** Jede Zeile merkt sich ihren Schlüssel; weil Sortieren und
Filtern die Datenzeilen nicht umstellen, bleibt diese Zuordnung gültig.

```basic
IMPORT "db"
DIM r AS DB_RESULT
r = DB_QUERY(db, "SELECT id, name, punkte FROM spieler ORDER BY id")
WHILE DB_NEXT(r)
    DIM z AS ARRAY OF STRING
    z = SPLIT$(DB_GET_STRING(r, 1) + "|" + STR$(DB_GET_INT(r, 2)), "|")
    GUI_TABLE_ADD_ROW(tbl, z)
    zeilenId[n] = DB_GET_INT(r, 0)      ' Datenzeile -> Datenbank-id
    n = n + 1
WEND
```

Beim Bearbeiten schreibt man zurück, sobald die Bearbeitung *endet* — den alten
Wert merkt man sich beim Öffnen:

```basic
DIM jetzt AS INTEGER : jetzt = GUI_TABLE_EDITING_ROW(tbl)
IF jetzt >= 0 AND warZeile < 0 THEN
    warZeile = jetzt : altText = GUI_TABLE_GET_CELL(tbl, jetzt, 0)
ELIF jetzt < 0 AND warZeile >= 0 THEN
    IF GUI_TABLE_GET_CELL(tbl, warZeile, 0) <> altText THEN
        DB_EXEC(db, "UPDATE spieler SET name = ? WHERE id = ?", _
                GUI_TABLE_GET_CELL(tbl, warZeile, 0), zeilenId[warZeile])
    END IF
    warZeile = -1
END IF
```

Beim Löschen mehrerer Zeilen: **erst alle Schlüssel einsammeln, dann löschen** —
während des Löschens verschieben sich die Zeilennummern der Tabelle.

> **Zum Anschauen:** [examples/158_gui_tabelle_sqlite.dh](../examples/158_gui_tabelle_sqlite.dh)
> — die gelösten Level aus `pyramid_pusher.db` in der Tabelle: sortieren,
> filtern, umbenennen (`UPDATE`), löschen (`DELETE` in einer Transaktion).
> Die Demo arbeitet auf einer **Kopie** (`VACUUM INTO`); die Originaldatei wird
> nur gelesen. Eine Demo, die den Spielstand des Nutzers ändert, wäre eine
> schlechte Demo.

**Nicht umgesetzt** (bewusst): Zeilengruppen/Baum in der Tabelle.
- Optionaler `GUI_ON_CHANGE(tbl, funcref)`-Callback feuert bei Selektionswechsel.
- Farben folgen dem Theme (`GUI_SET_COLOR(tbl, ...)` überschreibt pro Widget: bg/fg/border/accent).

```basic
GUI_UPDATE()
GUI_DRAW()
IF GUI_TABLE_CLICKED(tbl) >= 0 THEN
    PRINT "Zeile " + STR$(GUI_TABLE_SELECTED(tbl)) + " gewaehlt"
END IF
```

**Beispiel:** [examples/81_table_select.dh](../examples/81_table_select.dh) zeigt
beide Tabellen (Retained `gui` + Immediate `ui`) nebeneinander.

## Baum (Tree-View)

```basic
DIM tree AS GUI_WIDGET : tree = GUI_TREE(win, 20, 20, 300, 380)
DIM proj AS INTEGER : proj = GUI_TREE_ADD(tree, -1, "Mein Spiel")   ' Wurzel
DIM src  AS INTEGER : src  = GUI_TREE_ADD(tree, proj, "src")        ' Kind von proj
GUI_TREE_ADD(tree, src, "main.dh")
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
lange Bäume. Beispiel: [examples/137_gui_tree.dh](../examples/137_gui_tree.dh).

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
Live-Umschalter: [`examples/128_gui_modern.dh`](../examples/128_gui_modern.dh).

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

Neben dem Polling (`IF GUI_CLICKED(b) THEN ...`) kannst du eine Drachenhauch-
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
- **Eine Methode geht auch** — `GUI_ON_CLICK(ok, spieler.start)` bindet den
  Handler an die Instanz `spieler`, und `Self` zeigt darin auf sie. Das ist der
  Weg, wenn der Handler Zustand braucht: statt einer globalen Variablen plus
  freier `SUB` trägt der Rückruf sein Objekt selbst mit.

  ```basic
  CLASS Spiel
      DIM punkte AS INTEGER
      SUB start()
          Self.punkte = 0
      END SUB
  END CLASS

  DIM spiel AS Spiel
  spiel = NEW Spiel()
  GUI_ON_CLICK(ok, spiel.start)
  ```

  **Grenze:** ein so gebundener Handler wird von `GUI_SAVE`/`GUI_TO_JSON`
  **nicht** mitgeschrieben. Beim Laden gäbe es die Instanz nicht, und nur den
  Methodennamen zu speichern wäre irreführend — er würde als freie Funktion
  gedeutet. Formulare, die als `.dhform` überleben sollen (Form-Designer),
  brauchen also weiterhin freie Handler-Funktionen.
- Funktioniert für **Buttons** (Klick = Press+Release auf dem Knopf) und
  **Checkboxen** (bei jedem Toggle).
- Die Callbacks werden **am Ende von `GUI_UPDATE()`** aufgerufen (nachdem alle
  Events des Frames verarbeitet sind). Während eines Callbacks ausgelöste
  weitere Events feuern erst im nächsten Frame — keine Re-Entrancy-Schleife.
- Polling und Callback schließen sich nicht aus; beides ist gleichzeitig nutzbar.
- `GUI_ON_CLICK(widget, NIL)` entfernt den Callback wieder.

Intern überbrückt `GUI_ON_CLICK` die Builtin→VM-Grenze (eine FUNCREF aus einem
Built-in heraus aufrufen) — nativ in `dhrt`.

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

## Booleans

`GUI_CLICKED`/`GUI_CHECKED`/`GUI_HOVERED` liefern echte BOOLEANs — sie
funktionieren in Bedingungen (`IF GUI_CLICKED(b) THEN ...`) und lassen sich
direkt drucken:

```basic
DIM r AS BOOLEAN
r = GUI_CHECKED(snd)
PRINT r                   ' TRUE / FALSE
PRINT GUI_CHECKED(snd)    ' dasselbe
```

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
| `GUI_FOCUSED()` → GUI_WIDGET | welches Widget hat den Tastatur-Fokus? (`-1` = keins) |
| `GUI_SCALE(faktor)` | Anzeige-Maßstab (0.5–4.0), **vor** dem ersten Fenster |
| `GUI_SCALE_GET()` → FLOAT | aktueller Maßstab |
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

Weil das `.dhform` pro Control den **Namen seines Event-Handlers** mitspeichert
(`on_click` / `on_change`) und `GUI_UPDATE` ausgelöste Handler **automatisch per
Name aufruft**, ergibt sich der Xojo-Ablauf von selbst: Formular laden, nur die
Handler ausfüllen — kein manuelles Verdrahten.

```basic
IMPORT "gui"
SCREEN(800, 480, "App", 1)

DIM frm AS GUI_WINDOW
frm = GUI_LOAD("forms/settings.dhform")   ' Controls + Handler-Namen

SUB on_save()                              ' du schreibst NUR die Handler ...
    PRINT "Speichern geklickt"
END SUB

WHILE NOT QUITREQUESTED()
    GUI_UPDATE()                           ' ... GUI_UPDATE ruft on_save automatisch
    CLS(0) : GUI_DRAW() : FLIP()
WEND
```

Vollständiges Beispiel (Formular `examples/forms/settings.dhform` +
[examples/105_form_runner.dh](../examples/105_form_runner.dh)): ein
Einstellungs-Dialog mit TextInput, Checkbox, Slider, Dropdown und zwei Buttons,
dessen Handler die Control-Werte auslesen. Das `.dhform` lässt sich von Hand
schreiben **oder** (künftig) von einem visuellen Designer erzeugen — beides
ergibt dieselbe JSON.

## Vollständiges Beispiel

Siehe [examples/45_gui.dh](../examples/45_gui.dh): Fenster mit Slider (live ins
Label gespiegelt), Checkbox, Textfeld und Start-Button — verschiebbar und
schließbar.

**Alle 22 Widget-Arten in einer Anwendung:**
[examples/156_gui_alle_widgets.dh](../examples/156_gui_alle_widgets.dh) —
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

> Zum Anschauen: [examples/155_gui_glas.dh](../examples/155_gui_glas.dh) —
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

**Fokus bekommt jedes bedienbare Widget** (seit 2026-08-30 — vorher nur
Textfeld, Textbereich und Zahlenfeld). `on_focus`/`on_blur` feuern damit auch
an Knopf, Kästchen, Klappliste und Baum, sobald `TAB` oder ein Klick dort
landet. Reine Deko (Beschriftung, Panel, Trennlinie, Gruppe, Werkzeugleiste,
Fortschritt, Bild, Zeichenfläche) bekommt weiterhin keinen Fokus und feuert
die beiden Flanken nie.

Alle sechs überleben `GUI_SAVE`/`GUI_LOAD` bzw. `GUI_TO_JSON`/`GUI_FROM_JSON`
— das ist der Weg, über den ein im Form-Designer gebautes Formular seine
Handler bekommt.

## Bedienung ohne Maus

`TAB` / `SHIFT+TAB` wandert durch **alle bedienbaren Widgets** des aktiven
Fensters, in Anlege-Reihenfolge und nur über sichtbare, eingeschaltete. Reine
Deko wird übersprungen — sonst müsste man sich durch Beschriftungen
hindurchtabben. Das fokussierte Widget bekommt einen **Ring in der
Akzentfarbe**; ohne sichtbaren Fokus wäre die Navigation wertlos.

| Taste | Wirkung |
|---|---|
| `TAB` / `SHIFT+TAB` | nächstes / vorheriges Widget |
| `LEERTASTE`, `ENTER` | Knopf auslösen, Kästchen/Schalter umschalten, Radio wählen, Klappliste auf/zu, Baumknoten auf/zuklappen |
| `←` `→` `↑` `↓` | Regler/Drehknopf verstellen (ein Zwanzigstel des Bereichs je Druck), Trenner verschieben (8 px), Auswahl in Liste/Klappliste/Baum bewegen |
| `POS1` / `ENDE` | Regler/Drehknopf auf Minimum / Maximum |
| `→` / `←` im Baum | aufklappen bzw. ins Kind / zuklappen bzw. zum Elternknoten |
| `ESC` | offene Klappliste schließen |

Welches Widget gerade dran ist, liefert `GUI_FOCUSED()` (`-1` = keins) —
gedacht für eine Statuszeile oder einen Hilfetext zum aktiven Feld. Setzen
lässt sich der Fokus mit `GUI_FOCUS(wdg)`.

```basic
' Hilfetext zum Widget unter dem Fokus
DIM aktiv AS GUI_WIDGET
aktiv = GUI_FOCUSED()
IF aktiv = feldName THEN TEXT(10, 220, "Vor- und Nachname eingeben")
IF aktiv = knopfOk THEN TEXT(10, 220, "Uebernimmt die Aenderung")
```

> **Stolperstein:** Benutzt dein Programm `ESC` zum Beenden, prüfe vorher, ob
> gerade eine Klappliste offen ist oder eine Tabellenzelle bearbeitet wird
> (`GUI_TABLE_EDITING_ROW < 0`) — sonst beendet die Taste das Programm,
> während der Benutzer nur ein Popup schließen wollte.

## Das `TEXTAREA` als Code-Feld

Ein mehrzeiliges Textfeld wird mit vier Einstellungen und einer Einfärbung zu
einem brauchbaren Code-Feld.

| Funktion | Wirkung |
|---|---|
| `GUI_TEXTAREA_SET(ta, schluessel$, wert)` | `zeilennummern`, `aktive_zeile`, `tab_fuegt_ein`, `tabbreite` |
| `GUI_TEXTAREA_SPANS(ta, starts, laengen, farben)` | Zeichen `start … start+laenge` in `farbe` zeichnen |
| `SYNTAX_SPANS(quelltext$)` → (starts, laengen, arten) | Drachenhauch-Quelltext zerlegen |
| `GUI_TEXTAREA_VIEW(ta)` → (erste_zeile, zeilen, start_zeichen, laenge_zeichen) | welcher Ausschnitt ist gerade zu sehen? |

```basic
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(win, 12, 12, 650, 300)
GUI_TEXTAREA_SET(ta, "zeilennummern", 1)
GUI_TEXTAREA_SET(ta, "aktive_zeile", 1)
GUI_TEXTAREA_SET(ta, "tab_fuegt_ein", 1)

' Einfärben -- nach jeder Änderung neu
DIM starts AS ARRAY OF INTEGER
DIM laengen AS ARRAY OF INTEGER
DIM arten AS ARRAY OF STRING
(starts, laengen, arten) = SYNTAX_SPANS(GUI_TEXT(ta))
DIM farben[LEN(starts)] AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO LEN(starts) - 1
    farben[i] = &HD8E4F0
    IF arten[i] = "kommentar" THEN farben[i] = &H6A8A5A
    IF arten[i] = "text" THEN farben[i] = &HE0A060
    IF arten[i] = "zahl" THEN farben[i] = &HD07070
    IF arten[i] = "schluessel" THEN farben[i] = &H2BC4E8
NEXT
GUI_TEXTAREA_SPANS(ta, starts, laengen, farben)
```

**Warum getrennt?** `SYNTAX_SPANS` sagt, *was* ein Stück Text ist —
`GUI_TEXTAREA_SPANS` sagt, welche *Farbe* es bekommt. Welche Farbe ein
Kommentar hat, ist eine Frage deines Themas und nicht der Sprache. Und weil
`GUI_TEXTAREA_SPANS` nur Zahlen entgegennimmt, kannst du damit auch etwas ganz
anderes einfärben: Suchtreffer, Fehlerstellen, ein Diff.

Die Arten aus `SYNTAX_SPANS`: `kommentar`, `text`, `zahl`, `schluessel`,
`name`, `operator`. Leerraum bekommt keinen Abschnitt und bleibt in der
Grundfarbe.

> **Der Hervorheber ist kein Lexer.** Er sieht halb getippten Text (`"abc`
> ohne schließendes Anführungszeichen, `IF x THE`) und stellt ihn trotzdem
> dar, statt abzubrechen. Eine offene Zeichenkette endet für ihn am
> Zeilenende — sonst färbte ein einzelnes Anführungszeichen den Rest der
> Datei ein. Die Wortliste teilt er sich mit dem echten Lexer, ein neues
> Schlüsselwort ist also sofort auch hier bekannt.

**`tab_fuegt_ein` ist per Vorgabe AUS.** Sonst käme man in einem Formular aus
dem Textfeld nicht mehr heraus — `TAB` ist dort die Weiter-Taste (siehe
[Bedienung ohne Maus](#bedienung-ohne-maus)). Ist der Schalter an, rückt der
Tabulator bis zur **nächsten Spalte** ein, nicht stur um `tabbreite`
Zeichen; mitten in der Zeile getippt stünde die Einrückung sonst schief.

> **`GUI_SET_TEXT` löscht die Abschnitte.** Sie gehörten zum alten Text; sie
> stehen zu lassen färbte den neuen nach den Positionen des alten. Nach jedem
> `GUI_SET_TEXT` also neu einfärben. Beim **Tippen** hinkt die Einfärbung
> naturgemäß hinterher, bis dein Programm sie erneuert — Abschnitte, die über
> das Textende hinausragen, werden beim Zeichnen abgeschnitten und sind
> unschädlich.

Vollständiges Beispiel: [`examples/184_codefeld.dh`](../examples/184_codefeld.dh).

### Große Dateien: nur einfärben, was man sieht

Den **ganzen** Text bei jedem Tastendruck neu zu zerlegen kostet mit der
Dateigröße. Gemessen auf einem Arbeitsrechner, je Anschlag:

| Zeilen | Abschnitte | ganze Datei | nur der sichtbare Ausschnitt |
|---|---|---|---|
| 3.000 | 16.500 | ~26 ms | **0,2 ms** |
| 10.000 | 55.000 | ~88 ms | **0,8 ms** |
| 30.000 | 165.000 | ~272 ms | **2,1 ms** |

Bis etwa 3.000 Zeilen ist der einfache Weg (ganze Datei) völlig in Ordnung —
ein Anschlag kostet dann rund ein Bild. Darüber wird das Tippen zäh.

Der Grund ist nicht das Zeichnen (das kostet konstant ~0,4 ms, weil nur die
sichtbaren Zeilen gezeichnet werden) und auch nicht `SYNTAX_SPANS` (2,4 ms bei
3.000 Zeilen). Es ist die **Schleife im eigenen Programm**, die jede Art auf
eine Farbe abbildet — bei 16.500 Abschnitten. Schon eine `MAP` statt vier
`IF`-Vergleichen halbiert sie.

Ganz weg bekommt man sie mit `GUI_TEXTAREA_VIEW`:

```basic
DIM z0 AS INTEGER
DIM anz AS INTEGER
DIM von AS INTEGER
DIM laenge AS INTEGER
(z0, anz, von, laenge) = GUI_TEXTAREA_VIEW(ta)

DIM teil AS STRING
teil = MID$(GUI_TEXT(ta), von, laenge)      ' nur der sichtbare Ausschnitt
DIM st AS ARRAY OF INTEGER
DIM ln AS ARRAY OF INTEGER
DIM ar AS ARRAY OF STRING
(st, ln, ar) = SYNTAX_SPANS(teil)

DIM fb[LEN(st)] AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO LEN(st) - 1
    st[i] = st[i] + von                     ' Ausschnitt -> ganzer Text
    fb[i] = MAPGETOR(tabelle, ar[i], &HD8E4F0)
NEXT
GUI_TEXTAREA_SPANS(ta, st, ln, fb)
```

Dann hängen die Kosten nur noch an der **Fenstergröße**, nicht an der Datei.
Neu einfärben musst du dafür nicht nur beim Tippen, sondern auch beim
**Scrollen** — `z0` sagt dir, ob sich der Ausschnitt verschoben hat.

> **Warum das nicht nur schneller, sondern auch richtig ist:** Kommentare und
> Zeichenketten enden in Drachenhauch an der **Zeile**. Ein an Zeilengrenzen
> geschnittener Ausschnitt kann darum nicht mitten in einem Gebilde anfangen.
> Bei einer Sprache mit Blockkommentaren wäre genau das die Falle — dort
> müsste man wissen, in welchem Zustand die erste sichtbare Zeile beginnt.

Der Rest an Kosten (0,2 → 2,1 ms über die drei Größen) ist das Kopieren des
Textes durch `GUI_TEXT` und `MID$`. Das wächst weiter mit der Datei, ist aber
billig genug, um nicht aufzufallen.

## Farbwähler und Datumswähler

Zwei Widget-Arten, die eine Oberfläche oft braucht und die man sonst von Hand
nachbauen müsste.

| Funktion | Wirkung |
|---|---|
| `GUI_COLORPICKER(win, x, y, w, h)` → GUI_WIDGET | Sättigungs-/Helligkeitsfeld plus Ton-Streifen |
| `GUI_PICKED_COLOR(picker)` → INTEGER | gewählte Farbe (`0xRRGGBB`) |
| `GUI_SET_PICKED_COLOR(picker, farbe)` | Farbe setzen |
| `GUI_DATEPICKER(win, x, y, w, h)` → GUI_WIDGET | Monatsgitter mit Blätterpfeilen |
| `GUI_DATE(picker)` → STRING | gewähltes Datum als `JJJJ-MM-TT` |
| `GUI_SET_DATE(picker, datum$)` | Datum setzen |
| `GUI_COLORPICKER_SET(picker, schluessel$, wert)` | `alpha` (0/1) — Deckkraft-Streifen zeigen |
| `GUI_DATEPICKER_SET(picker, schluessel$, wert)` | `wochenbeginn` (0 = Montag … 6 = Sonntag) |
| `GUI_DATE_RANGE(picker, von$, bis$)` | erlaubter Bereich (leerer Text = keine Grenze) |

Beide melden Änderungen über `GUI_ON_CHANGE`.

```basic
DIM waehler AS GUI_WIDGET
waehler = GUI_COLORPICKER(win, 16, 36, 320, 230)
GUI_SET_PICKED_COLOR(waehler, &H3FA9F5)

DIM kalender AS GUI_WIDGET
kalender = GUI_DATEPICKER(win, 366, 36, 410, 286)
GUI_SET_DATE(kalender, "2026-08-30")

' ... pro Bild:
BOX(20, 300, 200, 340, GUI_PICKED_COLOR(waehler))
TEXT(20, 350, "Termin am " + GUI_DATE(kalender), WEISS)
```

**Das Datumsformat ist `JJJJ-MM-TT`** — dasselbe, das `DATE$()` liefert. Zwei
Datumsformate im selben System wären eine Stolperfalle. Ein neuer Wähler zeigt
**heute**; ein leerer Kalender wäre eine unnötige Frage an den Benutzer.
Unsinn wird abgelehnt, auch der 29. Februar in einem Jahr, das keines ist.

**Ohne Maus:**

| Taste | Farbwähler | Datumswähler |
|---|---|---|
| `←` `→` | Sättigung | ein Tag |
| `↑` `↓` | Helligkeit | eine Woche |
| `BILD ↑` `BILD ↓` | Farbton | ein Monat |
| `POS1` `ENDE` | Deckkraft (nur mit `alpha`) | — |
| `UMSCHALT` dazu | feinere Schritte | ein **Jahr** statt ein Monat |

> **Der Farbton wird mitgeführt, nicht zurückgerechnet.** Bei Schwarz ist er
> unbestimmt (jeder Ton ergibt Schwarz), bei Grau die Sättigung. Ein Wähler,
> der nur die RGB-Farbe behält, verliert ihn genau dort — der Zeiger springt
> beim Herunterziehen auf Schwarz nach links, und beim Aufhellen kommt Rot
> zurück statt der Farbe, die man gewählt hatte. Deshalb speichert das Widget
> HSV; `GUI_PICKED_COLOR` rechnet erst beim Abruf um.

> **Beim Blättern klemmt der Tag.** Der 31. Januar plus ein Monat ist der
> letzte Februartag, nicht der 3. März — sonst liefe der Wähler beim
> Durchblättern langsam nach vorne davon.

Ein Klick auf einen Tag des Nachbarmonats (die matten Zahlen am Rand)
blättert dorthin. Farbe und Datum überleben `GUI_SAVE`/`GUI_LOAD` — in der
`.dhform` stehen sie als `"#RRGGBB"` und `"JJJJ-MM-TT"`, also lesbar.

### Deckkraft

`GUI_COLORPICKER_SET(picker, "alpha", 1)` blendet einen zweiten Streifen ein.
`GUI_PICKED_COLOR` liefert dann `0xAARRGGBB` statt `0xRRGGBB`:

```basic
GUI_COLORPICKER_SET(waehler, "alpha", 1)
GUI_SET_PICKED_COLOR(waehler, &HC03FA9F5)     ' halb durchscheinendes Blau
```

Ohne den Schalter bleibt es bei sechs Stellen — Programme, die den Wähler
schon benutzen, bekommen unverändert das, was sie erwarten.

> **Die Deckkraft geht von 1 bis 255, nicht von 0.** Ein oberstes Byte von `0`
> liest die Laufzeit als **deckend** — so bleiben die alten 24-Bit-Farben
> (`&Hrrggbb`) undurchsichtig. Für „praktisch unsichtbar" ist `1` der kleinste
> Wert; was ganz weg soll, zeichnet man einfach nicht.

Der Streifen liegt auf einem **Schachbrett**: ohne das sähe man dem Verlauf
nicht an, wo er durchsichtig wird.

### Grenzen und Wochenbeginn

```basic
' Keine Termine in der Vergangenheit
GUI_DATE_RANGE(kalender, DATE$(), "")          ' leer = keine Obergrenze
GUI_DATEPICKER_SET(kalender, "wochenbeginn", 6) ' Woche ab Sonntag
```

Gesperrte Tage werden **matter** gezeichnet als die des Nachbarmonats (der
eine ist erreichbar, der andere nicht) und nehmen weder Klick noch Taste an.
Ein bereits gesetztes Datum außerhalb wird beim Setzen der Grenzen sofort
hereingezogen — sonst stünde im Feld ein Wert, den der Wähler selbst nicht
mehr zulässt.

Im Kopf blättern **vier** Bereiche: `‹‹` ein Jahr zurück, `‹` einen Monat
zurück, `›` und `››` entsprechend vorwärts. Mit der Tastatur ist
`UMSCHALT`+`BILD ↑`/`BILD ↓` der Jahressprung — ohne ihn bräuchte man bis
1985 rund fünfhundert Klicks.

### Farbe als Text

Zwei Kern-Builtins (kein `IMPORT` nötig), die es vorher nicht gab:

| Funktion | Wirkung |
|---|---|
| `COLOR_HEX$(farbe)` → STRING | `"#RRGGBB"`, mit Deckkraft `"#AARRGGBB"` |
| `COLOR_FROM_HEX(text$)` → INTEGER | `#RGB`, `#RRGGBB`, `#AARRGGBB` — mit oder ohne `#`, auch `0x`/`&H` |

Ohne sie **kann ein Programm Hex-Text gar nicht in eine Farbe wandeln**:
`VAL("&HFF8800")` liefert `0`, und `&H`-Literale gibt es nur im Quelltext,
nicht zur Laufzeit. Damit ließ sich weder eine Farbe aus einer
Einstellungsdatei lesen noch eine eingetippte übernehmen.

Die Kurzform verdoppelt jede Ziffer wie im Web: `#F80` = `#FF8800`. Ist die
Deckkraft `0` (also deckend), lässt `COLOR_HEX$` sie weg — sonst stünde vor
jeder gewöhnlichen Farbe ein sinnloses `00`.

Beispiel: [`examples/186_farbe_und_datum.dh`](../examples/186_farbe_und_datum.dh).

## Eigene Schrift

Die eingebaute raylib-Schrift muss nicht bleiben. Zwei Zeilen genügen, und
sie gilt für **alles** — Fenstertitel, Beschriftungen, Knöpfe, Eingabefelder,
Code-Feld, Zeilennummern:

```basic
DIM mono AS INTEGER
mono = LOADFONT("C:/Windows/Fonts/consola.ttf", 18)
SETFONT(mono)                  ' ab hier zeichnet alles damit
```

`SETFONT` setzt die **aktive** Schrift, und jedes Widget ohne eigene Schrift
folgt ihr. Für ein Code-Feld ist eine echte Monospace-Schrift ein spürbarer
Unterschied.

Feiner steuerbar:

| Funktion | Wirkung |
|---|---|
| `SETFONT(font)` | die aktive Schrift — gilt für die ganze Oberfläche |
| `GUI_SET_FONT(wdg, font)` | eigene Schrift nur für dieses Widget |
| `GUI_SET_FONT_SIZE(wdg, px)` | eigene Größe nur für dieses Widget |
| `GUI_STYLE_SET(name$, "font", font)` + `GUI_APPLY_STYLE(wdg, name$)` | eine Schrift für eine ganze Gruppe |

Für **Pixel-Schrift** aus einem PNG gibt es `LOADFONT_IMAGE(bild, trennfarbe,
erstes_zeichen)`. Die bleibt bewusst ungefiltert (nearest), damit Pixel-Schrift
pixelig bleibt — anders als `LOADFONT` (TTF), das glättet.

> **Die Größe steckt im Handle, und `SETFONT` übernimmt sie.** `LOADFONT(pfad,
> 18)` baut den Zeichensatz *für 18 px*; `SETFONT` setzt damit nicht nur die
> Schriftart, sondern auch die aktive Textgröße auf 18. Wundere dich also
> nicht, wenn nach `SETFONT` alles größer oder kleiner ist als vorher.
> Dieselbe Schrift in zwei Größen scharf? Zweimal laden — stark hochskaliert
> wird eine TTF sonst weich. `GUI_SET_FONT_SIZE` ändert nur, wie groß
> gezeichnet wird, nicht, wofür der Zeichensatz gebaut wurde.

> **Mit `GUI_SCALE`** (unten) skaliert auch die Schrift mit — du lädst sie also
> in der logischen Größe und bekommst sie auf einem HiDPI-Schirm größer
> gezeichnet.

## Maßstab (hochauflösende Bildschirme)

Alle Maße im `gui`-Modul sind feste Pixel. Auf einem 4K-Bildschirm mit 200 %
Skalierung wird eine Oberfläche, die für 1920×1080 gebaut wurde, damit
halb so groß wie gedacht — lesbar, aber winzig. `GUI_SCALE` löst das:

```basic
IMPORT "gui"
GUI_SCALE(WINDOW_DPI_X())      ' 1.0 normal, 2.0 auf HiDPI/Retina
SCREEN(1600, 1000, "Mein Werkzeug", 1)
' ... ab hier wie immer, in den gewohnten Zahlen
DIM w AS GUI_WINDOW
w = GUI_WINDOW("Formular", 10, 10, 400, 300)
```

Der Faktor (0.5 bis 4.0) multipliziert **jede Länge, die in die GUI
hineingeht**: Fenster- und Widget-Geometrie, Titelleisten- und Zeilenhöhen,
Innenabstände, Spaltenbreiten und die Schriftgröße. Du rechnest also nichts
um — dein Layout bleibt in den Zahlen, in denen du es entworfen hast.

**Nach außen bleibt alles logisch.** `GUI_GET_X`, `GUI_WINDOW_GET_W`,
`GUI_TABLE_GET("zeilenhoehe")` und `GUI_TO_JSON` liefern die Zahlen zurück,
die du hineingegeben hast; `GUI_SET_BOUNDS` nimmt sie genauso entgegen. Ein
gespeichertes `.dhform` beschreibt damit weiterhin das **Layout**, nicht die
Anzeige — sonst würde eine Form bei jedem Öffnen und Speichern um den Faktor
weiterwachsen.

**Die eine Ausnahme ist `GUI_HIT_TEST(x, y)`** — es beantwortet eine Frage
über den Bildschirm und nimmt deshalb Bildschirm-Pixel, so wie `MOUSEX()`
sie liefert. Wer eigene Zeichenbefehle (`TEXT`, `BOX`, …) neben ein Widget
setzen will, rechnet mit `GUI_SCALE_GET()` um:

```basic
TEXT(GUI_GET_X(feld) * GUI_SCALE_GET(), y, "Pflichtfeld", ROT)
```

> **`GUI_SCALE` muss vor dem ersten Fenster kommen.** Danach ist es ein
> Fehler. Schon angelegte Widgets nachträglich umzurechnen ginge nur
> näherungsweise — jede Runde brächte neue Rundungsfehler, und eine halb
> skalierte Oberfläche wäre schlimmer als eine klare Absage. Wer den Maßstab
> zur Laufzeit umstellen will (Einstellungsdialog), baut die Oberfläche nach
> `GUI_RESET` neu auf.


