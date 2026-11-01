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
debugger.

## The user interface

| Area | what it does |
|---|---|
| Toolbar | New, Open, Save; add window, method, event; Undo, Redo, Align, Zoom; Start, Stop, Check, Build |
| Navigator (top left) | the project as a tree: App, windows with their controls and events, modules, classes, their methods and properties; a click opens |
| Library (bottom left) | all building blocks of the gui in groups, with a preview; search field above |
| Middle | tabs with a cross: per window the layout, per method its code, per element its properties |
| Inspector (right) | the properties of the selected control, without a selection those of the window |
| Status bar | message, name, position and size of the selected control, zoom (click: 100 %) |

The panels can be moved, stacked as tabs and detached, as in the IDE. The View
menu offers a light and a dark appearance.

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

A new project has the App and a window `Hauptfenster`. Save (Ctrl+S) writes
all files; a project never saved asks for a folder. A renamed element takes
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
| list, tree | Geaendert, Doppelklick |
| slider, number field, dropdown, pickers, tab control, table | Geaendert |
| canvas | Zeichnen, Maus, Ziehen(x, y), Rad(schritte) |
| every control in addition | MausRein, MausRaus |
| timer, menu entry | Aktion |
| window | Geoeffnet, Schliessen, Groesse, Taste(taste) |
| App | Geoeffnet, Beenden |

An event with parentheses calls its method with values: `Ziehen` (drag)
with the position on the canvas (while the button is held after a press on
it), `Rad` (wheel) with the steps of the mouse wheel, `Taste` (key) with the
key code (`KEY_A`, `KEY_SPACE` ...), as long as the window is in front and
no input field takes the key. The suggestions know the whole text of the
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
| arrow keys | move by one pixel, with Shift by the grid (8) |
| Del | deletes the selection |
| Ctrl+D | duplicates |
| Ctrl+A | selects everything (not while an input field has the focus) |
| Ctrl+Z / Ctrl+Y | undo / redo |
| Ctrl+mouse wheel, Ctrl+Plus/Minus, Ctrl+0 | zoom |
| Esc | cancels placing, otherwise clears the selection |

**Alignment guides:** while dragging and resizing, edges and centres snap to
those of the other controls. They also snap to the edges and the centre of
the window, 20 pixels from the edge and 12 pixels next to a neighbour. A pink
line shows what they snapped to. Where no guide applies, the grid does.
Holding Alt snaps to nothing. Both can be switched off in the View menu.

**Arrange** (menu and toolbar) aligns the selection with the control clicked
last: edges, centres, same width, height or size. Distribute sets equal gaps
from three controls on. Bring to front and send to back change the drawing
order.

## The inspector

The properties are in groups that fold open and closed: identity, position,
content, appearance, behaviour. Only what fits the kind of control is shown.
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

## A single window

A `.dhform` (the same format that `GUI_SAVE` writes and `GUI_LOAD` reads)
also opens without a project, from the form designer or from a program;
controls without a name get one when opened. As long as only the window is
edited, Save writes exactly this file. When code or another element is
added, it becomes a project and Save asks for a folder. New, Open and Quit
ask when there are unsaved changes.

## For tests

| Environment | effect |
|---|---|
| `DH_STUDIO_LOG=file` | writes what the studio does, line by line, plus the positions of library rows, inspector fields, toolbar buttons, navigator nodes and tabs |
| `DH_STUDIO_GROESSE=WxH` | a fixed window size instead of maximised |
| `DH_STUDIO_AUFNAHME=file` | plays a recording (`AUTOMATION_PLAY`) |
| `DH_STUDIO_ABLAGE=text` | puts text on the clipboard, for typing with Ctrl+V |
| `DH_STUDIO_LAUF_BILDER=n` | Start runs the program for only n frames (`dhrt run --bilder n`) |
| `DH_STUDIO_DATEI=path` | Bild hinzufügen takes this file instead of a dialog |

The test collections `tests/pruef/werkzeug_studio.dhtest` (stage 1) and
`tests/pruef/werkzeug_studio2.dhtest` (stage 2) start the studio twice: once
to measure, once with real clicks and keys. They also test the project model
(`studio/projekt.dh`) without a window.
`tests/pruef/werkzeug_studio3.dhtest` (stage 3) presses F5 and Ctrl+B, runs
the generated program with a click and checks at which method and line an
error lands. `tests/pruef/werkzeug_studio4.dhtest` (stage 4) creates timers
and menus, runs the generated program through questions, a modal window,
timers, a menu shortcut and an image, and halts in the debugger.
