# Drachenhauch Studio

Drachenhauch Studio is an IDE for programs with a user interface, in the
style of Xojo: put windows together with the mouse, set properties in the
inspector, later write the code of the events and start the program. It runs
on the `dhrt` runtime and is itself a Drachenhauch program. The design with
all stages is in [entwurf-studio.md](../entwurf-studio.md) (German).

```text
dhrt run studio/studio.dh
dhrt run studio/studio.dh -- meinprojekt
dhrt run studio/studio.dh -- fenster.dhform
```

A folder with `projekt.dhstudio` opens as a project, a folder without one
gets a new project, a `.dhform` opens as a single window.

When installed, "Drachenhauch Studio" is in the start menu or application
menu, and a `.dhstudio` opens in the studio with a double-click (on Windows,
if the association was chosen during installation). On macOS and Linux
`drachenhauch --studio [project]` starts it from the terminal. In the IDE:
Werkzeuge → Drachenhauch Studio (with the project folder if it is a studio
project), and a `.dhstudio` in the project tree opens in the studio. The form
designer opens its form with Datei → Im Drachenhauch Studio öffnen
(Ctrl+Shift+O).

**Status: stage 4.** Projects with several windows, modules and classes,
events and a code field per method; starting, building and errors shown at
the method they belong to; timers, menus, dialogs that wait, images and a
debugger. On top of that custom controls and databases in the project.

## The user interface

| Area | what it does |
|---|---|
| Toolbar | New, Open, Save; add window, method, event; Undo, Redo, Align, Zoom; Start, Stop, Check, Build |
| Navigator (top left) | the project as a tree: App, windows with their controls and events, modules, classes, their methods and properties; a click opens |
| Library (bottom left) | all building blocks of the gui in groups, with a preview; search field above |
| Middle | tabs with a cross: per window the layout, per method its code, per element its properties |
| Inspector (right) | the properties of the selected control, without a selection those of the window; when the tab of another element is in front (database, module), only its name and kind |
| Status bar | message, name, position and size of the selected control, zoom (click: 100 %) |

The panels can be moved, stacked as tabs and detached, as in the IDE. The View
menu offers a light and a dark appearance.

**The interface comes in German and in English.** View -> "English
interface" switches; the choice is stored in `studio.json` next to the
`ide.json` and takes effect at the next start. Without a choice of its own
the Studio follows the language of the IDE. In English the new names are
English too (`MainWindow`, `Button1`), "New project" offers English
templates (`studio/vorlagen/en`: the same projects with English names, texts
and comments), and the messages of the runtime (Problems, Output) come in
English. **The names of the events stay as they are** (`Klick`,
`Geaendert`, `Aktion` ...): they are part of the method names
(`Knopf1_Klick`), and a project has to stay the same in both languages. So
do the commands the Studio gives a program (`Meldung`, `Frage`,
`Zumachen`).

## The project

A project is a folder with readable files:

```text
meinprojekt/
  projekt.dhstudio      Name, Startfenster, die Elemente (JSON)
  App.dh                CLASS App EXTENDS StudioApp
  Hauptfenster.dhform   das Layout, wie GUI_SAVE es schreibt
  Hauptfenster.dh       CLASS Hauptfenster EXTENDS Hauptfenster_Basis
  Werkzeuge.dh          ein Modul: SUBs und FUNCTIONs
  Kunde.dh              eine Klasse
  .gitignore            _studio/ (dort erzeugt Stufe 3 den Startcode)
```

A new project has the App and a window `Hauptfenster`. New project
(Ctrl+Shift+N) asks what to start with: **Leer** (empty), **Formular mit
Eingaben** (form: fields in a column, OK and Cancel at the bottom right,
validation rules), **Adressbuch mit Datenbank** (address book: SQLite, table
with filter row, fields bound to the columns) or **Zeichenfläche mit Timer**
(canvas with a bouncing ball). Every template is a finished project that runs
straight away with F5; the templates are project folders under
`studio/vorlagen/` and are never overwritten. A project nobody has touched yet
does not ask again on the next New or Open. Save (Ctrl+S) writes all files; a
project never saved asks for a folder. A renamed element takes
its old files along, a deleted one disappears with the next save.

| Action | effect |
|---|---|
| Ctrl+Shift+N | new project |
| Ctrl+O | open a project (`projekt.dhstudio`) or a single window |
| Ctrl+N | add a window (asks for the name) |
| Projekt → Modul / Klasse hinzufügen | a module or a class, its properties open |
| Projekt → Als Startfenster | the window the program starts with ("(Start)" in the navigator) |
| Projekt → Element löschen | takes a window, module or class with its code out of the project (the App stays) |

A name is an identifier and unique in the project, because it becomes the
name of the class or module. The name of a window is changed in the
inspector (with nothing selected).

## Code and events

A window is a class, its events are methods. A method is called
`<Control>_<Event>` (`knopf1_Klick`), an event of the window or the App just
`<Event>` (`Geoeffnet`). The base class the studio generates when starting
wires them up.

| Action | effect |
|---|---|
| double-click a control in the design | "add event" for this control; on empty space: for the window |
| Ctrl+Shift+E | the same for the selection (or for the element of the front tab) |
| Ctrl+Shift+M | add a method: name, parameters (`a AS INTEGER, b AS STRING`), return type (empty = SUB) |
| Ctrl+Space | suggestions at the cursor; after `Self.` or `objekt.` they open by themselves. In the list typing jumps, Enter or Tab inserts, Esc closes |
| click a method in the navigator | opens its code in a tab |
| click "Eigenschaften" | opens DIM, CONST and whatever else is outside the methods |
| Projekt → Methode entfernen | the method in the front tab |
| cross on a tab, middle mouse button | closes it (nothing is lost) |

The events per kind:

| Kind | events |
|---|---|
| button, icon button, card | Klick |
| checkbox, toggle, radio button | Klick, Geaendert |
| text field | Geaendert, Enter, FokusRein, FokusRaus |
| text area | Geaendert, FokusRein, FokusRaus |
| list, tree | Geaendert, Doppelklick (for the list also Enter) |
| table | Geaendert, Doppelklick (also Enter), Zelle(zeile, spalte) |
| slider, number field, dropdown, pickers, tab control | Geaendert |
| canvas, chart | Zeichnen, Maus, Ziehen(x, y), Rad(schritte) |
| toolbar | Knopf(nr) |
| status bar | Feld(nr) |
| every control in addition | MausRein, MausRaus, Abgelegt(zeile, ziel) |
| timer, menu entry | Aktion |
| window | Geoeffnet, Schliessen, Groesse, Taste(taste), Takt |
| own control | Geoeffnet, Groesse, Takt |
| App | Geoeffnet, Beenden |

`Zeichnen` (draw) runs in every frame in which the canvas is visible (not
on a tab page in the background), clipped to the canvas and in the font of
the interface: the gui's own (`GUI_FONT`), otherwise the built-in Sans at
the program's text size -- not the pixel font `TEXT` uses otherwise. Whatever
`Zeichnen` changes in font, size and style stays there (like
`GFX_PUSH`/`GFX_POP`). Coordinates are screen points from
`GUI_CANVAS_X/Y`. Maus, Rad and Ziehen do not come while a modal window
stands in front of the canvas.

A **chart** (library, group Drawing) is a canvas with a chart from the
[`chart`](module-chart.md) module: the inspector sets its kind (pie, donut,
bar, line, area, gauge, bar gauge, LEDs) and title, the design view shows a
sample. In code the control is a `StudioDiagramm` that draws itself in every
frame, as large as the canvas and in the theme of the interface (light or
dark, also after a switch); your own `Zeichnen` comes on top.

```basic
Self.vorrat.Leeren()
Self.vorrat.Hinzufuegen("Obst", 5)
Self.vorrat.Hinzufuegen("Milch", 2, RGB(200, 60, 60))   ' own colour
Self.vorrat.Setzen("legende", "rechts")                 ' CHART_SET
Self.vorrat.SetzenSchalter("prozent", TRUE)             ' CHART_SET_FLAG
```

Also `Reihe(name, werte)` for lines and grouped bars, `Beschriften`, `Wert`
for gauge, bar gauge and LEDs, `Titel`, `SetzenZahl`, `Anzahl()`; `Flaeche`
is the canvas, `Diagramm` the CHART handle for everything else.

An event with parentheses calls its method with values: `Ziehen` (drag)
with the position on the canvas (while the button is held after a press on
it), `Rad` (wheel) with the steps of the mouse wheel, `Taste` (key) with the
key code (`KEY_A`, `KEY_SPACE` ...), as long as the window is in front and
no input field takes the key, `Knopf` (button) with the number of the
toolbar button pressed and `Feld` (field) with the number of the status bar
field clicked (from 0, separators and gaps count too), `Zelle` (cell) with
the data row and column of every click into a table -- also on a button in
the cell (`GUI_TABLE_CELL_KIND "knopf"`) and also in the row already
selected, where `Geaendert` no longer comes; `Abgelegt` (dropped) with the
dragged row of the source and the row it landed on, when something was
dragged onto a control that is "Ablage" (drop target) in the inspector (the
source is "Ziehbar" (draggable): for list, tree and table the row, otherwise
the control; `GUI_DROP_SOURCE()` says where it came from, `GUI_DROP_TEXT()`
what). `Takt` (tick) runs in every
frame after the events of the controls (those of an own control before
those of the window) -- that is where code polls what has no event, such as
a context menu from `GUI_CONTEXT`. The inspector holds
the buttons in one line, `text|symbol|tip` per button, separated by
semicolons, `-` is a separator, `~` a gap (`New|plus|New product; -;
|rueckgaengig|Undo`); the fields as `text|width`, optionally followed by
`links`/`mitte`/`rechts` and `klick` for a field that reports `Feld`
(width 0 shares the rest). The suggestions know the whole text of the
program including the generated base class -- so after `Self.` the
controls, timers and menu entries of the window appear too.

Above the code stands the header of the method (`SUB knopf1_Klick()`), which
you do not type; to its right, what it is for. The code field colours,
closes brackets, indents on a line break and adds the end of a block
(`IF` → `END IF`). In a code tab Ctrl+Z/Y belong to the code.

When a control is renamed in the inspector, its event methods follow, and in
the code of the window the old name is replaced as a whole word.

## Starting and building

| Action | effect |
|---|---|
| F5, Start | saves, checks, generates `_studio/` and starts the program with `dhrt run` |
| Shift+F5, Stop | ends the running program |
| Shift+F7, Check | checks the whole project without starting it |
| Ctrl+B, Build | generates `_studio/` and writes a standalone executable to `bau/` with `dhrt --export` |

What the program prints appears in the **Output** panel at the bottom; next
to it are the **Problems**. Checking also happens on its own, a moment after
every change. A finding is shown at the method it belongs to
(`Hauptfenster.knopf1_Klick, Zeile 3`), there as a squiggle, a mark and text
at the end of the line; a click in the list opens the method at that spot.
What the compiler only notices at the end of a method (an open `IF`) is shown
"am Ende" (at the end). An error stops the start. A runtime error is also
placed at its method.

In `_studio/` (not for the repository, see `.gitignore`) the studio generates:

| File | what it contains |
|---|---|
| `main.dh` | the entry point: includes everything and calls `studioLauf` with the App and the start window |
| `studio_laufzeit.dh` | `StudioApp`, `StudioFenster` and the frame loop |
| `<Fenster>_basis.dh` | per window the class `<Fenster>_Basis`: controls as fields, construction, wiring of the events, empty event methods |
| `projekt/<Element>.dh` | the code of the project as it stands in the editor |

The user class `<Fenster> EXTENDS <Fenster>_Basis` overrides the event
methods. A control is a field there (`Self.knopf1`), `Self.Fenster()` returns
the gui window; `Self.Zeigen()` and `Self.Zumachen()` open and close a
window, and closing the start window ends the program. The base class builds
the window from the layout as JSON (`GUI_FROM_JSON`) instead of call by
call -- so it looks exactly as in the designer. Without the studio, start the
program with `dhrt run _studio/main.dh`.

## What a program needs

**Timers.** In the library under "Unsichtbar" (invisible). A timer does not
sit in the window but in a strip below it (as in Xojo); a click selects it,
the inspector shows name, interval, "runs at start" and "repeat", Del
deletes it. Its event is `<Name>_Aktion`. In code it is a field of the window
class with `Intervall`, `Aktiv`, `Wiederholen`, `Starten()` and `Stoppen()`.

**Menus.** Projekt → Menüs des Fensters (Ctrl+Shift+U) opens the menu
editor: the tree on the left, text, name and shortcut (`Strg+S`, `F5` ...)
on the right, plus "with check mark". A new entry gets a name from its text
(`miOeffnen`) that follows the text until you change it yourself. Only OK
writes the menus into the layout -- one undo step; renamed entries take
their event methods with them. Named entries appear in the navigator under
the window, their event is `<Name>_Aktion`.

**Dialogs that wait.** The runtime of the project offers:

| Call | what it does |
|---|---|
| `Meldung(text$ [, titel$])` | a message with OK |
| `Frage(text$ [, knoepfe$ [, titel$]])` | buttons separated by `\|` (default `Ja\|Nein`), returns the number of the button; Enter = the first, Esc = the last |
| `Eingabe(text$ [, vorgabe$ [, titel$]])` | asks for a text, `""` on cancel |
| `fenster.ZeigenModal()` | shows a window modally and returns only when it is closed |
| `Bild(name$)` | an image from the project's `bilder/` as IMAGE |

All four return only with the answer, as in Xojo or Visual Basic. This works
because the generated base class polls the events **after** `GUI_UPDATE`
instead of running them as gui callbacks -- so an event method may run
frames itself. If the program ends meanwhile, `Frage` returns 0. Timers keep
running during a dialog.

**Images.** Projekt → Bild hinzufügen (Ctrl+Shift+B) copies a file into `bilder/` in the
project folder; an image control picks its image in the inspector under
"Bild" and shows it in the designer already. When starting and building,
the images go to `_studio/assets/bilder/`, and `dhrt --export` takes
`assets/` next to the executable.

**Debugger.**

| Action | effect |
|---|---|
| Click into the line numbers, F9 | breakpoint on the line on/off (red dot) |
| F7, Debug | generates `_studio/` and starts the program under `dhrt debug` |
| F5 | continue while it is halted |
| F10, F11, Shift+F11 | step, step into, step out |
| Shift+F5 | ends the program |

When it halts, the code tab of the method opens, the line is marked, and the
**Debugger** panel at the bottom shows the variables (for `Self` the fields
of the window) and the call stack. The position is translated by the same
line map that brings errors to their method.

## Designing a window

| Action | effect |
|---|---|
| click a building block in the library, then click into the window | places it there (its centre at the click) |
| drag a building block into the window | places it where you let go |
| double-click a building block | places it in the middle of the window |
| click a control | selects it; Shift or Ctrl click adds or removes |
| drag a frame over empty space | selects everything entirely inside |
| drag a control | moves the whole selection |
| drag one of the eight handles | changes the size |
| drag the right or bottom edge or the corner of the window | changes the size of the window; anchored controls move along (likewise when you type the width or height in the inspector) |
| arrow keys | move by one pixel, with Shift by the grid (8) |
| Del | deletes the selection |
| Ctrl+D | duplicates |
| Ctrl+A | selects everything (not while an input field has the focus) |
| Ctrl+Z / Ctrl+Y | undo / redo |
| Ctrl+mouse wheel, Ctrl+Plus/Minus, Ctrl+0 | zoom |
| Esc | cancels placing, otherwise clears the selection |

A window that does not fit into the middle at 100 % opens at 75 or 50 %;
whatever reaches beyond the middle is clipped instead of lying over the
inspector and the output. The height in the inspector is that of the window
including its title bar (a control at the bottom has to end above it; only
the start window runs without a title bar in the program). If the window has
menus, the menu bar sits above the content: in the program the window grows
by the bar.

**Alignment guides:** while dragging and resizing, edges and centres snap to
those of the other controls. They also snap to the edges and the centre of
the window, 20 pixels from the edge and 12 pixels next to a neighbour. A pink
line shows what they snapped to. Where no guide applies, the grid does.
Holding Alt snaps to nothing. Both can be switched off in the View menu.

**Arrange** (menu and toolbar) aligns the selection with the control clicked
last: edges, centres, same width, height or size. Distribute sets equal gaps
from three controls on. Bring to front and send to back change the drawing
order.

**Layouts** (library, group containers: row, column, grid) arrange their
controls themselves: side by side, one below the other or in rows and
columns, with spacing and margin. In the design a layout has a dashed
outline; an empty one says "Controls hierher ziehen". A building block that
lands on a layout goes into it, and so do the controls dragged into it --
several selected ones together, left to right or top to bottom; a blue
marker shows the place. A child is not moved but repositioned: it stays
where it is, its outline follows the mouse, and when released it goes to the
place of the marker -- released next to every layout it is free afterwards.
That also works for several selected children at once. The arrow keys move a
child one place on. A **spacer** (also in the containers group) is free space
in a layout: with weight 1 it takes the rest and pushes what follows to the
end. In the design it is a spring between its neighbours; a click selects
it, the inspector shows its weight, the arrow keys move it, Del deletes it. Duplicating a layout
takes its children along, deleting leaves them standing free. In the
navigator the children are listed under their layout. Anchoring the layout in
the inspector (say left, right, top) lets it grow with the window, and it
spreads its children anew -- in the generated program the runtime places
them just as in the design.

## The inspector

The properties are in groups that fold open and closed: identity, position,
layout, content, appearance, behaviour. Only what fits the kind of control
is shown: a layout has its arrangement, columns (for the grid), spacing,
margin, alignment, "Kinder dehnen" (stretch children) and a frame; a child
of a layout has its weight instead of left, top and anchor (0 = its own
size, otherwise its share of the free space).
Every property has its editor:
- text and numbers in fields;
- switches as toggles;
- choices as dropdowns;
- colours in the colour picker, which opens as a popover ("Vorgabe des
  Themas" removes the colour again);
- the anchor as four checkboxes (which edge the control sticks to when the
  window grows).

A change shows in the window at once and applies to all selected controls
that have the property. An undo step is made when the field loses the focus
or Enter comes, not per keystroke. A name must be an identifier and unique in
the window, because it later becomes a field of the window class; an invalid
name is marked red and not applied.

Without a selection the inspector shows the window, including **Default
button (Enter)** and **Cancel button (Esc)**: a dropdown with the window's
buttons. In the program Enter then presses the one and Esc the other -- its
`_Klick` runs as for a mouse click (Enter also from a single-line text
field; a text area keeps its Enter). The button is remembered by name; renaming it carries the choice
along, deleting it drops it.

## Custom controls

Several controls become a building block of their own, like a
ContainerControl in Xojo: Project -> "Add custom control …" creates an element
that is designed like a window -- only without a title and menus (timers
work). It has its own code and events for its parts (`knopf_Klick`), plus the
events `Geoeffnet` and `Groesse` of the control itself.

The library lists all custom controls under **Project**. You place one into
a window like a button and get an instance (`zaehler1`). It can be moved,
resized, anchored and put into rows, columns and grid layouts; in the design
it shows its real content, outlined with a dashed line, and a click on one
of its parts selects the whole instance.

```basic
' in the code of the control Zaehler
SUB knopf_Klick()
    Self.n = Self.n + 1
    GUI_SET_TEXT(Self.anzeige, STR$(Self.n))
END SUB

FUNCTION Wert() AS INTEGER
    RETURN Self.n
END FUNCTION

' in the code of the window
SUB knopfSumme_Klick()
    Meldung(STR$(Self.zaehler1.Wert() + Self.zaehler2.Wert()))
END SUB
```

In the code of the window an instance is an object of the control's class:
its methods and fields can be reached (`Self.zaehler1.Wert()`,
`Self.zaehler1.anzeige`). Every control also brings `Fenster()`,
`Behaelter()`, `Breite()`, `Hoehe()` and `Sichtbar(an)`.

**How it runs:** in the program an instance is an empty layout container --
it takes the anchors, the weight in a layout and the position --, and the
parts of the control stand behind it in the same window. Every frame the
runtime puts them where the container currently is, following their anchors
inside the control: a part anchored left and right grows with the instance.

**Own events and properties.** The inspector of the control has two fields
under "Custom control": **Events** (names separated by semicolons,
`Erreicht; Geleert`) and **Properties** (`Ziel AS INTEGER = 3; Vorsatz =
"Anzahl: "` -- without `AS` a STRING, also FLOAT and BOOLEAN; quotes keep
spaces at the edges). Every property is a field of the control's class with
this default. An instance has the field **Values** in the inspector (`Ziel =
2; Vorsatz = "Erste: "`); a name the control doesn't know, or a value that
doesn't fit the type, is dropped.

```basic
' in the code of the control Zaehler
SUB knopf_Klick()
    Self.n = Self.n + 1
    IF Self.n = Self.Ziel THEN Self.Ausloesen("Erreicht")
END SUB

' in the code of the window
SUB zaehler1_Erreicht()
    Meldung("Ziel erreicht")
END SUB
```

`Self.Ausloesen("Name")` reports one of the events; the window then calls
its method `<instance>_<event>` -- after the control's tick, in the same
frame. A double click on the instance offers its events like on any other
control.

**Controls inside controls.** A control may contain instances of other
controls, just like a window. Their events arrive in the outer control
(`z1_Erreicht` in the code of `Zwei`), which can pass them on to the window
with `Ausloesen`; in the window it is then `Self.zwei1.z1.Wert()`. The
library offers the other controls inside a control, but not the control
itself and none that already contains it -- that would be a cycle. If one
is in the file anyway (written by hand), the instance stays an empty
placeholder and the design view says "… contains itself". When a control is
renamed, every instance in the other windows and controls takes the new
name along, in their undo states too. Deleting only works once no instance
points at it any more; otherwise the status bar says where one still is.

## Databases

Project → Add database adds an SQLite database as an element. Its structure
is kept in `<Name>.dhdb` (JSON) in the project, the data in an SQLite file
in the project folder (default `<name>.sqlite`, changeable at the top right
of the tab). The navigator lists its tables under the database; a click
opens the database tab: the tables on the left with New, Rename and Delete,
on the right either their **columns** or their **data**.

| Type | in SQLite | typed as |
|---|---|---|
| Text | TEXT | as it is |
| Whole number | INTEGER | `-4` |
| Decimal | REAL | `2,5` or `2.5` |
| Money | INTEGER in cents | `19,99` (stored as 1999) |
| Date | TEXT | `2026-11-03` |
| Yes/No | INTEGER 0/1 | `yes`/`no` |
| Image | BLOB (the bytes of the image file) | with **Image …** from a file |

Every table has the column `id INTEGER PRIMARY KEY` by itself -- data
binding (`GUI_FORM_LOAD`/`GUI_FORM_SAVE`) needs exactly that. In the column
grid you rename a column in its cell, pick the type from the list and tick
**Required** (NOT NULL; when empty, the type's empty value or the default
applies) and **Unique** (an index of its own). In the data grid you type
values into the cells (several at once with Ctrl+V too); empty is NULL,
except in a required column. The first 1000 rows are shown.

**Every change applies to the file in the project folder at once**, in one
transaction: if it fails (Unique on duplicate values, a value that does not
fit the type), the status bar says why, and file and structure stay as they
were. Changing a column rebuilds the table and copies its rows (SQLite
cannot change a column); columns and tables the program created by itself
stay. Deleting and renaming cannot be undone.

**A file that already exists** (from another program) is taken over with
Project → Database from file: the file goes into the project folder (if it
is already there, it stays), its structure becomes the schema -- types,
required, defaults, unique (UNIQUE over one column) and references (FOREIGN
KEY to an `id`). What no Studio type matches exactly (`DATE`, `TIMESTAMP`,
`DEFAULT CURRENT_TIMESTAMP`, a required column without a
default) is kept verbatim next to it and applies until you change the type
or the default in the grid; so the sync afterwards rebuilds no table, and
the file stays exactly as it was. What the schema cannot hold (UNIQUE over
two columns, a table without `id`) stays in the file and is listed in a
message; the tab still shows and edits a table without `id`, but it cannot
be bound.

In the program the database is a global connection under its name, open
before the app starts:

```text
DIM r AS DB_RESULT : r = DB_QUERY(Daten, "SELECT name FROM kunden ORDER BY name")
```

Started from the studio (F5), the program works on the file in the project
folder -- what it writes shows up in the data grid afterwards. Building puts
the file next to the built program if there is none yet; one that has
already been worked in stays. The built program creates missing tables,
columns and indexes itself and renames what was renamed in the studio --
in order, as long as the old name still exists there and the new one does
not (a rename that only changes case does not count, SQLite sees the same
name). It leaves deleted things in place. Deleting the element only removes
the structure from the project; the file with the data stays.

**Binding a window to a table.** The navigator lists the columns under each
table. Dragging a column or the whole table into the open window adds a
label and a matching field per column, bound to the column: text, numbers
and money as a text field (money with a decimal comma), a date as a text
field for `YYYY-MM-DD`, yes/no as a check box. Double-clicking a column puts
its field below the last control. If the window has no table yet, it gets
this one; the inspector shows it in the **Data** group (Table on the window,
Column on each field).

| Method | does |
|---|---|
| `Laden(id)` | loads the row into the fields, `DatensatzId` is then `id` |
| `Sichern()` | checks the fields' rules and writes (a new row when `DatensatzId` is 0) |
| `Neu()` | empties the fields and sets the columns' defaults |
| `Loeschen()` | deletes the row that is shown |
| `Ungesichert()` | has anything changed since loading or saving? |

A list, a table or a drop-down list shows the rows of a table when its
**Data source** is chosen in the inspector; **Shown columns** says which
(separated by semicolons; empty: all for a table, otherwise the first). The
window fills it when it opens; `Neuladen()` fetches it again, and
`IdVon(list)` returns the id of the selected row. If a list shows the
window's own table, clicking a row loads it into the fields, and after
saving and deleting the list is already up to date.

```text
SUB knopfNeu_Klick()
    Self.Neu()
END SUB

SUB knopfSichern_Klick()
    IF NOT Self.Sichern() THEN Meldung("Bitte die Eingaben prüfen.")
END SUB
```

**References.** In the column grid, **Refers to** names another table of the
same database: the column then holds the id of one of its rows (it becomes a
whole number). Dragged into a window, such a column becomes a drop-down list
that shows the first text column of the target table and binds the id. The
**Shown columns** of a list may also name a column of the target table:
`nr;kunde.name` shows the customer's name next to the number, for the
customer whose id is in `kunde`. The studio puts a LEFT JOIN around the table
for this (a row without a customer stays, with an empty name); in the result
the column is called `kunde.name`, and under that name **Sort by**, the
**Condition** (`"kunde.ort" = 'Köln'`) and the search field know it as well.

**Images.** A column of the type **Image** holds the bytes of an image file
(PNG, JPG, BMP or GIF). The data grid shows its size; **Image …** sets the
image of the selected cell from a file, Del empties it. Dragged into a
window, an image column becomes an image control on that column (inspector:
**Image column**; it is not part of the form binding). It shows the image
with `Laden(id)`; `BildSetzen(control, path)` and `BildLeeren(control)` change
it, and `Sichern()` writes it to the file. A table with a data source shows
an image column as a picture in the cell.

**Condition, sorting, search.** A list with a data source also has **Sort
by** (a column of the source) and **Descending** in the inspector, a
**Condition** (SQL as after WHERE, for example `ort <> 'Bonn'`; the studio
tries it on the file in the project folder and shows an error at the field --
also for a name in double quotes that the source does not have: SQLite would
otherwise silently take it as text, and the list would stay empty)
and a **Search field** (a text field of the window: as soon as something is
typed into it, the list shows only rows whose shown columns contain the
text).

**Queries.** [Query …] in the database tab adds a named SELECT; the queries
are listed below the tables on the left, their SQL and the result (the first
1000 rows) on the right. An SQL is only saved once it runs on the file --
otherwise the previous version stays. A query is a data source like a table
(with an `id` column, `IdVon` returns it, otherwise 0); dragged from the
navigator into a window it becomes a table. In code it runs by its name, also
with the database's name in front:

```text
DIM r AS DB_RESULT : r = Abfrage("koelner")
WHILE DB_NEXT(r)
    PRINT DB_GET_STRING(r, 1)
WEND
```

## Reports

A **report** is a printed page from the project's data, for example an
invoice or a list (Project → Add report …). Its description is in
`<Name>.dhreport` (JSON); how it is built and what is still to come is in the
[design](../entwurf-berichte.md) (German). A report has **bands** one below the
other: report header, page header, group header, detail (per row of the
source), group footer, page footer, report footer. They hold elements in mm:
fixed text, fields (a column of the source, also `kunde.name` through a
reference), sums (sum, count, average, min, max per group or report), page
number ("Seite {n} von {m}"), date, lines, rectangles and images.

The **source** is a table or query of a database of the project; the
**condition** may have parameters (`rechnung = :id`). If a band no longer fits
on the page, it goes onto the next one, with the page footer before it and the
page header after it.

The tab of a report is its **design**: the page in millimetres, every band
with a bar carrying its name, the elements inside.

* **Placing:** at the top the bar of elements (text, field, sum, page number,
  date, line, rectangle, image) -- choose a kind, then click into a band
  (default size) or drag a rectangle. Esc cancels.
* **Columns of the source** are at the top right; dragging a column into a
  band makes a sum in a group or report footer, a heading in the page header,
  otherwise a field (money and numbers right-aligned); an image column becomes
  an image element. A double-click puts it into the detail.
* **Dragging** moves what is selected (grid 1 mm, Alt without); edges and
  centres snap to other elements and to the margins, a pink line shows it. A
  single element may change its band. Eight handles change the size, the
  bottom edge of a band its height.
* **Selecting:** click, with Shift or Ctrl to add, a frame over empty space
  selects everything it touches; a click on the bar of a band selects the
  band, a click beside all bands the report. Arrow keys move by 1 mm (with
  Shift 5 mm), Del deletes, Ctrl+D duplicates, Ctrl+A selects all elements,
  Ctrl+Z/Ctrl+Y undo and redo.
* **Properties** below belong to what is selected: for an element position
  and size in mm, text, column, format, function of a sum and what it sums
  over (group or report), "Only rows with" (below), font,
  size, bold, italic, colour (`#RRGGBB`), alignment, stroke, fill, image; for
  a band its height and for a group its column, plus "Remove band" or
  "Remove group"; for the report title, source, condition, sorting (`a; b`),
  paper, portrait and margins (`t; r; b; l` or one number for all), plus "Add
  band" (what is missing) and "Add group" (inside, with header and footer).
  Text is taken over by Enter or leaving the field, a choice applies at once;
  with several elements selected the value applies to all it fits.

**Only rows with** limits an element to rows with a value: `mwst = 19`
(also `<>`, `<`, `>`, `<=`, `>=`; text may be in quotes, numbers are compared
as numbers). A sum then only adds up those rows, every other element only
appears when its band has at least one such row. That is how an invoice shows
the tax per rate: in the report footer "MwSt 19 %" and a sum of the tax, both
with `mwst = 19`, below it the same for 7 % -- an invoice without items at
7 % does not show that line. What cannot be read like this, the field does not
accept.

Ctrl+wheel zooms, the wheel scrolls (with Shift sideways). **Preview …**
typesets the report with the data of the development file, asks for the
parameters and pages through the result.

An **image element** shows a file from `bilder/` (property Bild) or, when it
names a **column**, the image from that image column for each row -- a photo
per article in the detail, for example. An image is fitted into the rectangle
without distorting it; an empty cell or bytes that are not an image leave it
empty.

In the program a report is an object under its name:

```text
Rechnungsbericht.Pdf("rechnung.pdf", {"id": 5})
Rechnungsbericht.Drucken({"id": 5})
Rechnungsbericht.Vorschau({"id": 5})
DIM p AS PDF : p = Rechnungsbericht.Erzeugen({"id": 5})
```

`Vorschau` shows a window with the pages, Print, "Als PDF …" and Close and
returns when it is closed. A missing parameter is an error naming it.

## A single window

A `.dhform` (the same format that `GUI_SAVE` writes and `GUI_LOAD` reads)
also opens without a project, from the form designer or from a program;
controls without a name get one when opened. As long as only the window is
edited, Save writes exactly this file. When code or another element is
added, it becomes a project and Save asks for a folder. New, Open and Quit
ask when there are unsaved changes.

## The example project

`examples/studio/rechnungen/` is a small invoicing program (after the pilot
`examples/196_rechnungen.dh`) as a studio project: Datei → Öffnen, choose the
folder, F5.

| Element | Kind | What it does |
|---|---|---|
| App | App | fills the database on the first start, before the first window appears |
| Daten | database | the tables in `rechnungen.db` with references, plus the query `Belegzeilen` (item number, net, tax and gross per row, address) |
| Datenbank | module | sample data on the first start; next invoice number |
| Ausgabe | module | date and amounts for display, CSV export |
| Rechnung | class | an invoice with its items: load, calculate, save (one transaction), as PDF and in the preview through the report |
| Hauptfenster | window | all invoices with a filter row, menu with shortcuts, new/edit/PDF/preview/delete |
| RechnungFenster | window | create or change an invoice, modal |
| KundenFenster, ArtikelFenster | window | master data through data binding (inspector: Bindung) |
| Rechnungsbeleg | report | the invoice from `Belegzeilen` with `rechnung = :id`: sender and address in the report header, column header on every page, net, tax per rate and total in the report footer |

What it shows: money is an INTEGER in cents, the tax is rounded per item; the
master data fields are bound to the columns, so loading and saving are one
call each (`GUI_FORM_LOAD`/`GUI_FORM_SAVE`), and `GUI_VALIDATE` checks the
rules (required, postcode, e-mail); a double-click on an invoice opens it
(`tabelle_Doppelklick`); unsaved input is asked about with
`Frage(..., "Sichern|Verwerfen|Abbrechen")`, and `ZeigenModal` waits until
the window is closed. The invoice is printed by the report: `Rechnung.alsPdf`
is one call, `Rechnungsbeleg.Pdf(pfad, {"id": Self.id})`. What has to be
calculated, the query calculates. The database is in the project folder (the
element Daten creates it, its tab shows the data); when started from the
studio, PDFs and the CSV end up in `_studio/`.

**A pitfall when naming:** names are not case-sensitive. A method
`zeigen(id)` in a window overrides the inherited `Zeigen()` -- `dhrt --check`
(and so the problem list) warns when a method overrides an inherited one with
a different number of parameters.

## For tests

| Environment | effect |
|---|---|
| `DH_STUDIO_LOG=file` | writes what the studio does, line by line, plus the positions of library rows, inspector fields, toolbar buttons, navigator nodes and tabs |
| `DH_STUDIO_GROESSE=WxH` | a fixed window size instead of maximised |
| `DH_STUDIO_AUFNAHME=file` | plays a recording (`AUTOMATION_PLAY`) |
| `DH_STUDIO_ABLAGE=text` | puts text on the clipboard, for typing with Ctrl+V |
| `DH_STUDIO_LAUF_BILDER=n` | Start runs the program for only n frames (`dhrt run --bilder n`) |
| `DH_STUDIO_DATEI=path` | Bild hinzufügen takes this file instead of a dialog |
| `DH_STUDIO_SPRACHE=en` | the language of the interface, before any setting (`de` or `en`) |
| `DH_STUDIO_KONFIG=file` | `studio.json` in this place; the `ide.json` is then not consulted |
| `DH_STUDIO_SPRACHPROBE=1` | opens every dialog once, writes its texts and the number of missing translations to the log, and quits |
| `DH_STUDIO_LOESCHEN=element` | asks at start like "Delete element …" for this element |
| `DH_STUDIO_TEMPO=1` | writes (with `DH_STUDIO_LOG`) `tempo <ms>` per frame: how long the studio computed, without waiting for the next frame |
| `DH_STUDIO_OEFFNEN=element[.table]` | opens an element at start like a click in the navigator (for a database optionally a table) |

The test collections `tests/pruef/studio/werkzeug_studio.dhtest` (stage 1) and
`tests/pruef/studio/werkzeug_studio2.dhtest` (stage 2) start the studio twice: once
to measure, once with real clicks and keys. They also test the project model
(`studio/projekt.dh`) without a window.
`tests/pruef/studio/werkzeug_studio3.dhtest` (stage 3) presses F5 and Ctrl+B, runs
the generated program with a click and checks at which method and line an
error lands. `tests/pruef/studio/werkzeug_studio4.dhtest` (stage 4) creates timers
and menus, runs the generated program through questions, a modal window,
timers, a menu shortcut and an image, and halts in the debugger.
`tests/pruef/studio/werkzeug_studio_datenbank.dhtest` tests the databases: the core
without a window (on the file, as SQLite sees it), the generated program on
an older file, and the studio with real clicks into both grids, F5 and
building; `tests/pruef/studio/werkzeug_studio_datenbank2.dhtest` tests binding:
methods and lists in the generated window including a click on a row,
dragging and double-clicking columns into the window, and the choices in the
inspector; `tests/pruef/studio/werkzeug_studio_datenbank3.dhtest` tests references,
condition, sorting, search field and queries: the core, the generated window
and the studio with real clicks in the tab, the navigator and the inspector;
`tests/pruef/studio/werkzeug_studio_datenbank4.dhtest` tests image columns (core,
generated window including the picture in the table cell, "Image …" and
dragging in the studio) and `tests/pruef/studio/werkzeug_studio_verbunde.dhtest`
joins through references; `tests/pruef/studio/werkzeug_studio_berichte.dhtest` tests
reports (layout with sums and pages, F5 with parameters, tab and preview),
`tests/pruef/studio/werkzeug_studio_berichte2.dhtest` their design (placing,
dragging, handles, columns, properties, bands and groups, undo).
