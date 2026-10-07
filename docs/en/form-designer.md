# Form designer (WYSIWYG, Xojo style)

Click together user interfaces for the `gui` module: drop controls, set them
up in the inspector, save them as `.dhform` and load them in your own program
with `GUI_LOAD` — or run them straight away with F5. The designer is itself a
Drachenhauch program:
[`examples/197_form_designer.dh`](../../examples/197_form_designer.dh)
(1,342 lines). The former Qt version (PySide6) was removed together with the
Python part of the project.

## Starting

- **From the IDE:** menu *Tools → Form designer* (or "Tool: form designer"
  in the command palette). Clicking a `.dhform` in the project tree opens the
  designer **with that file**; if you want to see it as text, use "Open form
  as text" in the command palette.
- **From the command line:**

```
dhrt run examples/197_form_designer.dh [-- formular.dhform]
```

A relative file name is taken from the folder in which the call was made
(`DHRT_START_DIR`). If the file does not exist yet, an empty form begins,
which is written there on the first save.

## Layout

- **Left — menu and palette.** The palette lists **all 32 widget kinds of
  the runtime** plus the **grid** (a table in cell mode with every column
  editable), all with German names (Knopf, Beschriftung, Kästchen, Textfeld,
  Klappliste ... — button, label, checkbox, text input, dropdown; the name is
  also the default text). Click an entry ("armed"), then click on the form =
  drop it, snapped to the 8 px grid. The status line sits below the palette.
- **Centre — the form.** It is a **real `GUI_WINDOW` in design mode**
  (`GUI_WINDOW_DESIGN(win, TRUE)`): the runtime draws the controls exactly as
  the program will get them later, but takes away all their input; the
  designer handles the mouse itself (`GUI_HIT_TEST` keeps working). There is
  no repainted preview that could differ from the runtime. Click = select,
  drag = move, drag one of the **eight handles** = resize — all snapped to the
  grid. **Zoom** (Ansicht → Vergrößern, Verkleinern, Originalgröße — View →
  Zoom in, Zoom out, Original size — or `Strg`+wheel (Ctrl+wheel) over the
  form): 50 to 300 % in steps, drawn via `GUI_WINDOW_ZOOM`. It only zooms in
  as far as the form fits between palette and inspector (the status line says
  so); the measurements in the form stay unzoomed, and dropping, dragging and
  handles convert the mouse back into the form. Zoomed text is slightly soft
  — the surface is scaled up as an image.
- **Right — inspector.** With a control selected: name, X, Y, width, height,
  text, tooltip, anchor (`lrtb`), `on_click`, `on_change`, `on_enter`,
  **font style** (`fett`, `kursiv`, `unterstrichen`, `durchgestrichen` —
  bold, italic, underlined, struck through — joined with `+`), *Aktiviert*
  (Enabled) and the **tab page** (see below), plus the fields per kind
  (below). With nothing selected it shows the **form itself**: title, width,
  height, *Größenveränderbar* (Resizable) and the **theme** (`glas_dunkel`,
  `glas_hell`, `dark`, `light` or none). Enter in a field or [Übernehmen]
  (Apply) writes the values.

## Keys

| Key | Effect |
|---|---|
| `Strg+N` / `Strg+O` | new form / open form |
| `Strg+S` / `Strg+Umschalt+S` | save / save as |
| `F5` | run the form (see below) |
| `Strg+G` | write DH code (see below) |
| `Strg+Z` / `Strg+Y` | undo / redo |
| `Strg+D` | duplicate controls (offset by one grid step) |
| `Entf` | delete controls |
| Arrows | move by one grid step, with `Umschalt` (Shift) by one point |
| `Umschalt`/`Strg`+click | add a control to the selection or take it away again |
| Drag over empty space | selection rectangle: selected is what lies completely inside (with `Umschalt` (Shift) added) |
| `Strg+A` | select all controls |
| `Strg+Plus` / `Strg+Minus` / `Strg+0`, `Strg`+wheel | zoom the design surface in / out / 100 % |
| `Esc` | disarm the palette, clear the selection |
| `Strg+Q` | quit |

(`Strg` = Ctrl, `Umschalt` = Shift, `Entf` = Del.)

*Nach vorn* and *Nach hinten* (Bring forward / Send backward) are in the
*Bearbeiten* (Edit) menu: the order of the controls is their drawing order,
the last one lies in front. Del, the arrows and `Strg+A` (Ctrl+A) only work
while no input field of the inspector has the focus (there `Strg+A` selects
the text).

**Several controls.** Dragging, the arrows, Del and `Strg+D` (Ctrl+D) act on
all selected controls; a click into a multiple selection keeps it. The **last
one clicked** is the reference: the inspector shows it (with "+n weitere",
+n more), and the **Anordnen** (Arrange) menu aligns to it — left, right,
top, bottom, horizontally or vertically centred —, gives the others its
width, height or size, or **distributes** all of them with equal gaps (at
least three; the first and the last stay where they are). The eight handles
only exist when a single control is selected.

**Tab pages.** A control belongs to a page of a **tab control** (or to a
step of a **wizard**) if its field *Reiterseite* (tab page) says
`NAME:SEITE`, the page counted from 1 — `reiterwerk1:2`. The pages
themselves are the *Einträge* (entries) of the tab control; *Gezeigte Seite*
(Shown page) decides which one the form currently shows (the controls of the
others are hidden there, as they will be in the program later). The runtime
keeps the children as widget **numbers** (`tabctl.kinder`), which shift with
every deletion and reordering; the designer therefore keeps them by **name**
and recomputes the numbers before every view. A renamed tab control takes its
children with it. A file that only knows numbers (from `GUI_SAVE`) gets the
names when opened — a control without a name gets one.

**Undo** remembers the whole form as JSON text per step (up to 200 states).
A mouse drag is **one** step, recorded on release.

**Quitting** — via the menu, the close button or Alt+F4 — asks
(*Sichern|Verwerfen|Abbrechen* — Save|Discard|Cancel) if the form is not
saved; otherwise the designer ends at once. **New** and **Open** ask the
same way (until 2026-09-23 they threw unsaved work away without a word); the
file dialog only comes after the answer.

## Fields per kind

Visible only for the matching kind; lists are separated with a **semicolon**,
because a comma too often appears inside an entry itself.

| Kind | Fields |
|---|---|
| Dropdown, list box | entries (`Rot; Grün; Blau`); for the dropdown the placeholder (`Bitte wählen`) and the checkbox *Tippen erlaubt* (typing allowed — combobox, `GUI_DROPDOWN_SET … "bearbeitbar"`), for the list box the checkbox *Umbenennen mit F2* (rename with F2, `GUI_LISTBOX_SET … "bearbeitbar"`) |
| Tab control, wizard | entries (the pages or steps), *Gezeigte Seite* (shown page, from 1) |
| Table, grid | columns, widths, editable (`0; 2` or `alle`), column types (`text; ganz; zahl; auswahl`), choices (`2 = Rot\|Grün`), empty hint (`Keine Treffer`, shown by the table when there is no row), checkbox *Zellmodus* (cell mode) |
| Tree | checkbox *Umbenennen mit F2*, empty hint |
| Slider, progress bar, number field, knob | min, max, value |

A column with a choice list thereby **becomes a choice column**, even if the
column types do not say so — otherwise the list would have had no effect on
loading.

The **data rows of a table** are deliberately not entered in the designer. A
table is normally filled at run time — from a file, a database, the save
game. The designer fixes the skeleton; the rows come from the program
(`GUI_TABLE_ADD_ROW`).

## Using it: three ways

1. **In your own code:** `GUI_LOAD("meinform.dhform")` and write the handler
   `SUB`s. The `.dhform` stores the **name** of its handler for each control
   (`on_click`, `on_change`, …), and `GUI_UPDATE` calls triggered handlers
   automatically by name — no wiring by hand (form workflow in
   [module-gui.md](module-gui.md)).

```basic
' How to use a saved form in your own program:
IMPORT "gui"
SCREEN(800, 480, "App", 1)
DIM frm AS GUI_WINDOW
frm = GUI_LOAD("forms/settings.dhform")

SUB on_save()            ' name = the handler entered in the inspector
    PRINT "gespeichert"
END SUB

WHILE NOT QUITREQUESTED()
    GUI_UPDATE() : CLS(0) : GUI_DRAW() : FLIP()
WEND
```

2. **Run directly (F5):** the designer saves (a new form asks for the name),
   writes `<name>_lauf.dh` **next to the `.dhform`** and starts it with
   `dhrt run`. The skeleton sets the theme, loads the form via `GUI_LOAD`
   borderless onto the program window (window size = form size) and contains
   a `SUB` per handler — with the body from the `code` field of the
   `.dhform`, otherwise with a `' TODO`. Pressing F5 again ends the previous
   run; it is also ended when the designer ends. The status line reports how
   it finished.
3. **DH code (Ctrl+G):** `<name>_code.dh` builds the form **call by call** —
   a constructor per control, for the table the header, widths, cell mode,
   editable columns, column types, choice lists and empty hint, for list box
   and tree the renaming, for the tree the empty hint, plus date and time,
   disabled state, anchor, tooltip, font style and the handlers with their
   bodies from `code`; without `GUI_LOAD` and without the `.dhform` at run
   time, readable and ready to be continued by hand. The constructors that
   measure themselves (label, checkbox, slider …) get a `GUI_SET_BOUNDS`
   afterwards, otherwise the size from the designer would be lost. Only the
   image is skipped (the `.dhform` knows no image source, and `GUI_IMAGE`
   would need a `LOADIMAGE`) — with a comment at that place. The code does
   not rebuild menus and says so in a comment as well.

## File format

`.dhform` is exactly the JSON that `GUI_SAVE`/`GUI_LOAD` and
`GUI_TO_JSON`/`GUI_FROM_JSON` write and read (see
[module-gui.md](module-gui.md)) — plus two designer fields that the runtime
skips: `name` per control and a top-level `code`
(`{handler_name: rumpf}`) with the handler bodies.

**The designer's model IS this JSON** (json module), the form on screen only
the view: every change writes into the JSON and rebuilds the view
(`GUI_FROM_JSON`). Saving is `JSON_PRETTY`, loading `JSON_LOAD`. It follows
that **everything the inspector does not show passes through unchanged** —
menus, tabs, table data, tree nodes, `code`, rules, bindings. A form built in
a program and saved with `GUI_SAVE` can therefore be opened in the designer
and readjusted without losing any of that; it cannot be edited there,
though.

Example form: `examples/forms/settings.dhform` with
[examples/105_form_runner.dh](../../examples/105_form_runner.dh).

## What the Qt version had and this one does not

The designer in Drachenhauch has a quarter of the lines of the Qt version
(1,342 against 5,055, factor 0.27) — and the factor mostly measures what is
left out. Not (or no longer) available:

- **Alignment guides** while dragging, **zoom**, a **context menu**.
- **Copy/paste** (only duplicate) and **dropping by drag and drop** from the
  palette (only click, then click on the form).
- A **code editor for handlers** (double-clicking a control created a handler
  there). Here the body lives in the `code` field of the `.dhform` and is
  only passed through; it is written in your own program or by hand in the
  file.
- **Multi-form projects** (`.dhproj`): here exactly one form is always open.
- In the inspector: **layout/panel assignment**, **rules** and **binding**,
  tab order, radio **group**, **placeholder**, the **selection** of a
  dropdown, min/max size and *movable/closable/visible* of the form, the
  table switches (zebra, filter row, sorting …) and the values of tree,
  colour and date pickers. Whatever of this is in a file is kept on opening
  and saving (see File format).
- **Menus in DH code** — the Qt version wrote them out; this one only says in
  a comment that `GUI_LOAD` builds them. The Qt version had no menu *editor*
  either.
- F5 does not check the run program with `dhrt --check` beforehand; an error
  shows up in the started program.
- The form on screen appears in the designer's theme, not in the form's
  chosen theme — only the generated program gets that.

## Three pitfalls during construction

- **A list box reports no `GUI_CLICKED`** — the palette becomes armed through
  its SELECTION; the selection is the event.
- **A newly built window takes the focus**, and menu shortcuts only applied in
  the window with the focus. `ansichtBauen` therefore remembers
  `GUI_FOCUSED()` and hands it back — without that, Ctrl+S and F5 would have
  been dead after the first drop. (Since 2026-09-07 shortcuts apply in all
  visible windows; handing the focus back stayed.)
- **Whole numbers stay whole:** `JSON_TYPE` only says `number`. When copying a
  subtree (duplicating, reordering), a whole number is therefore written as a
  whole number — `GUI_FROM_JSON` reads no x any more from `48.0`.

## Testing

[`tests/pruef/werkzeug_formdesigner.dhtest`](../../tests/pruef/werkzeug_formdesigner.dhtest)
(`dhrt test`): one case drops a button with a real click, saves with Ctrl+S
and reads the file with the json module; one drags a control and undoes
twice; one checks the F5 run program; one that foreign fields of an existing
form are kept; one creates **every** kind and runs the DH code through
`dhrt --check` and a run; two check the inspector on table and grid; one
measures the palette against `Kind::from_str` in `gui.rs` — a kind the
runtime can do and the designer cannot would otherwise go unnoticed; two
prove design mode (with a counter-check). `DH_FORM_LOG=<datei>` makes the
designer log its events line by line.
