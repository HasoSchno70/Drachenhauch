# Module `gui`

Retained-mode GUI: windows and widgets are **persistent objects**. Once
created, they live on — you build the interface once and then, per frame,
only call `GUI_UPDATE()` (process mouse/keys) and `GUI_DRAW()` (draw). You
query events by **polling** (`IF GUI_CLICKED(btn) THEN ...`).

```basic
IMPORT "gui"
```

> Difference from the [`ui` module](module-ui.md): `ui` is **immediate mode** —
> every widget is called anew every frame, no object persists.
> `gui` is **retained mode** with real window/widget objects, including
> moving (drag on the title bar), z-order (a click brings a window to the
> front), focus and a close button. Both modules may be used side by side.
> Widgets are **clipped to the window's inner area** — if a window is
> dragged smaller, nothing sticks out past the border/title bar.

## Overview

| Function | Returns | Purpose |
|---|---|---|
| `GUI_WINDOW(titel$, x, y, w, h)` | GUI_WINDOW | create a window |
| `GUI_WINDOW_MOVABLE(win, an)` | — | movable by its title bar (default: on) |
| `GUI_WINDOW_TITLE(win, titel$)` | — | change the title bar text later (name of the document, "Commands" / "Open file") |
| `GUI_WINDOW_GLOW(win [, farbe [, dauer_ms]])` | — | a streak of light sweeps across the window (default warm, 1200 ms, left to right) -- see [Streak of light](#light-sweep) |
| `GUI_WINDOW_GLOW_SET(win, schluessel$, wert)` | — | configure the streak of light: `farbe` (colour), `dauer` (duration), `pause` (ms until it returns, 0 = once), `richtung` (direction: rechts/links/unten/oben), `breite` (width), `staerke` (strength, 0..1), `rand` (edge), `art` (kind: `streif`, `lampe` or `rahmen`) |
| `GUI_WINDOW_GLOW_STOP(win)` | — | stop the streak of light |
| `GUI_WINDOW_GLOWING(win)` | BOOLEAN | is a streak of light running (also during the pause between two)? |
| `GUI_WINDOW_CLOSABLE(win, an)` | — | show a close button (default: off) |
| `GUI_WINDOW_VISIBLE(win, an)` | — | set visibility |
| `GUI_WINDOW_FRONT(win)` | — | bring the window to the front without giving it focus -- for a suggestion list or a tooltip that should lie over the text while typing. Shown via `GUI_WINDOW_VISIBLE`, a window otherwise lies behind the one clicked last |
| `GUI_WINDOW_SHOWN(win)` | BOOLEAN | is the window currently visible? The counterpart to the setter -- without the getter a program would have to remember what it set itself, and would be wrong as soon as the user closes the window with its cross |
| `GUI_WINDOW_RESIZABLE(win, an)` | — | resizable by the lower-right grip (default: off) |
| `GUI_WINDOW_SCROLLABLE(win, an)` | — | content scrolls when it is taller than the window (mouse wheel + scroll bar). Content height determined automatically from the widgets. Default: off |
| `GUI_WINDOW_CHROME(win, an)` | — | draw title bar/border/buttons? Off = borderless, content starts at the top (so that a form can fill the OS window). Default: on |
| `GUI_WINDOW_DESIGN(win, an)` | — | design mode: the window is drawn, but its widgets receive no input (no hover, click, focus, callback; no dragging the border either). A form designer places real widgets on the surface this way and handles the mouse itself — `GUI_HIT_TEST` and all setters keep working. Default: off |
| `GUI_WINDOW_ZOOM(win, faktor)` | — | draw a window in design mode magnified (0.25 to 4): the position stays, the area grows. Geometry seen from outside stays unmagnified, `GUI_HIT_TEST` and `GUI_WINDOW_AT` convert the mouse back. Drawing goes through an intermediate surface that is magnified smoothly — text is somewhat soft at 150–200 %. Outside design mode it is an error (the window would accept input, and the mouse would miss) |
| `GUI_WINDOW_GET_ZOOM(win)` | FLOAT | the factor of `GUI_WINDOW_ZOOM` |
| `GUI_WINDOW_SET_MIN_SIZE(win, w, h)` | — | minimum size when resizing (0 = none) |
| `GUI_WINDOW_SET_MAX_SIZE(win, w, h)` | — | maximum size when resizing (0 = none) |
| `GUI_SEPARATOR(win, x, y, w)` | GUI_WIDGET | decorative divider line (horizontal) |
| `GUI_GROUPBOX(win, x, y, w, h, title$)` | GUI_WIDGET | framed group with an inset title |
| `GUI_BIND(wdg, schluessel$[, formular$])` | — | **data binding**: the widget carries a key (column name) — see [Data binding](#data-binding) |
| `GUI_FORM_GET(win[, formular$])` / `GUI_FORM_SET(win, map[, formular$])` | MAP OF STRING / — | read all bound values / set them from a MAP |
| `GUI_FORM_CLEAR` / `GUI_FORM_CLEAN` / `GUI_FORM_CHANGED(win[, formular$])` | — / — / BOOL | clear / mark as saved / changed since the last set? |
| `GUI_FORM_LOAD(win, db, tabelle$, id[, formular$])` / `GUI_FORM_SAVE(…)` | BOOL / INTEGER | load the row `WHERE id = ?` into the widgets / INSERT (id < 0) or UPDATE, returns the id |
| `GUI_TEXTAREA_SET(ta, "umbruch", 1)` | — | text area wraps at word boundaries (notes, letters) |
| `GUI_SET_MIN_SIZE(wdg, min_w, min_h)` | — | minimum size: a weighted child in a container and an anchored widget do not shrink below it |
| `GUI_LAYOUT_MIN_W(layout)` / `GUI_LAYOUT_MIN_H(layout)` | INTEGER | what a container needs at least — the value for `WINDOW_MIN_SIZE` |
| `GUI_RULE(feld, art$[, a, b][, muster$][, meldung$])` | — | attach a **validation rule**: `pflicht` (required), `zahl` (number), `ganz` (integer), `bereich a b` (range), `laenge min max` (length), `email`, `datum` (date), `muster regex$` (pattern) — see [Form validation](#form-validation) |
| `GUI_VALIDATE(win)` / `GUI_VALIDATE_WIDGET(feld)` | INTEGER / STRING | check all rules of the window (number of errors, the first field gets focus) / one field (message or "") |
| `GUI_ERROR(feld)` / `GUI_SET_ERROR(feld, meldung$)` / `GUI_CLEAR_ERRORS(win)` | STRING / — / — | read the message, set it from outside (e.g. from the database), clear all |
| `GUI_ERROR_LABEL(feld, label)` / `GUI_VALIDATE_LIVE(win, an)` / `GUI_RULES_CLEAR(feld)` | — | a label shows the message / check when leaving a field / remove the rules |
| `GUI_VSLIDER(win, x, y, h, min, max, default)` | GUI_WIDGET | **vertical slider** — `h` is the length, the value grows upwards |
| `GUI_PROGRESS_SET(progress, "unbestimmt", 1)` | — | a running band instead of a value, for anything whose duration is unknown |
| `GUI_IMAGE_MODE(image, modus$)` / `GUI_IMAGE_MODE_GET(image)` | — / STRING | `strecken` (stretch, default), `einpassen` (fit), `fuellen` (fill), `mitte` (centre), `kacheln` (tile) |
| `GUI_TREE_ICON(tree, node, bild)` / `GUI_TREE_COLOR(tree, node, farbe)` | — | icon and text colour per tree node |
| `GUI_PANEL_ADD(panel, wdg)` / `GUI_PANEL_REMOVE` / `GUI_PANEL_SCROLL(panel, y)` / `GUI_PANEL_SCROLL_GET(panel)` | — | a **panel that scrolls its children** (mouse wheel, scroll bar) — see [Finishing touches](#finishing-touches-slider-progress-image-tree-panel-dragging) |
| `GUI_DRAGGABLE(wdg, an)` / `GUI_DROP_TARGET(wdg, an)` | — | **dragging between widgets**: source and drop target |
| `GUI_DRAGGING()` / `GUI_DROPPED(wdg)` / `GUI_DROP_TEXT()` / `GUI_DROP_SOURCE()` / `GUI_DRAG_INDEX()` / `GUI_DROP_INDEX()` | GUI_WIDGET / BOOL / STRING / GUI_WIDGET / INTEGER / INTEGER | current drag, drop in this frame, what was dragged, from where, which row, onto which row |
| `GUI_CURSORS(an)` | — | mouse pointer shapes depending on the spot in the widget (default on): text beam over text, hand over anything clickable, double arrow at dividers, column edges and the window grip |
| `GUI_SET_CURSOR(wdg, form$)` / `GUI_GET_CURSOR$(wdg)` | — / STRING | a custom pointer shape over a widget (say, a canvas); `auto` or `""` = the gui decides |
| `GUI_LAYOUT(win, art$, x, y, w, h)` | GUI_WIDGET | invisible **layout container**: `zeile` (row), `spalte` (column) or `raster:N` (grid) — distributes its children in every `GUI_UPDATE` (see [Layout](#layout-size-by-content-and-containers)) |
| `GUI_LAYOUT_ADD(layout, wdg[, gewicht])` / `GUI_LAYOUT_SPACER(layout[, gewicht])` / `GUI_LAYOUT_REMOVE(layout, wdg)` | — | append a child (weight 0 = its own size, from 1 = share of the remaining space), empty space, detach |
| `GUI_LAYOUT_SET(layout, key$, wert)` | — | `abstand` (spacing), `rand` (margin), `ausrichtung` (alignment, 0/1/2 across), `dehnen` (stretch across, default on), `rahmen` (frame, visible while developing) |
| `GUI_AUTOSIZE(wdg)` | — | width and height from the content; a width or height of `0` when creating means the same |
| `GUI_SET_ANCHOR(wdg, edges$)` | — | anchoring: which edges the widget sticks to (subset of `"lrtb"`, default `"lt"` = top-left). When the window is resized, the widgets flow along: left+right → stretch, only right → move along, none → centre (likewise top/bottom). |
| `GUI_WINDOW_CLOSED(win)` | BOOLEAN | was the window closed? |
| `GUI_BUTTON(win, text$, x, y, w, h)` | GUI_WIDGET | button |
| `GUI_BUTTON_VARIANT(knopf, art$)` | — | button kind: `standard`, `primaer` (primary), `erfolg` (success), `warnung` (warning), `gefahr` (danger), `umriss` (outline), `flach` (flat), `link` |
| `GUI_BUTTON_GET_VARIANT$(knopf)` | STRING | the button kind as a name |
| `GUI_ICON_BUTTON(win, x, y, w, h, tex[, text$])` | GUI_WIDGET | button with an icon (texture handle or name of a built-in icon); without text = flat toolbar button |
| `GUI_SET_ICON(button, tex)` | — | set/replace a button's icon (-1 removes it) |
| `GUI_TOOLBAR(win, x, y, w, h)` | GUI_WIDGET | toolbar: with entries (`GUI_TOOLBAR_ADD`) it lays out, draws and reports its buttons itself; without entries a flat strip |
| `GUI_TOOLBAR_ADD(tb, sinnbild[, tip$[, text$]])` | INTEGER | append a button — `sinnbild` is the name of a built-in icon or an image; returns the number of the entry |
| `GUI_TOOLBAR_SEPARATOR(tb)` | INTEGER | vertical separator as an entry |
| `GUI_TOOLBAR_SPACER(tb)` | INTEGER | gap that pushes the rest of the bar to the right |
| `GUI_TOOLBAR_CLICKED(tb)` | INTEGER | number of the button clicked in this frame, otherwise -1 |
| `GUI_TOOLBAR_COUNT(tb)` | INTEGER | number of entries (separators and gaps count) |
| `GUI_TOOLBAR_CLEAR(tb)` | — | remove all entries |
| `GUI_TOOLBAR_ENABLE(tb, eintrag, an)` | — | disable or enable a button |
| `GUI_TOOLBAR_ENABLED(tb, eintrag)` | BOOLEAN | whether a button can be used |
| `GUI_TOOLBAR_CHECKABLE(tb, eintrag, an)` | — | make a button toggleable: a click switches it on and off |
| `GUI_TOOLBAR_SET_CHECKED(tb, eintrag, an)` | — | switch a toggleable button on or off |
| `GUI_TOOLBAR_CHECKED(tb, eintrag)` | BOOLEAN | whether a toggleable button is switched on |
| `GUI_TOOLBAR_SET_ICON(tb, eintrag, sinnbild)` | — | change the icon of an entry (name or image) |
| `GUI_TOOLBAR_SET_TIP(tb, eintrag, tip$)` | — | tooltip of an entry |
| `GUI_TOOLBAR_SET_TEXT(tb, eintrag, text$)` | — | label of an entry |
| `GUI_TOOLBAR_SET(tb, schluessel$, wert)` | — | `beschriftung` (0/1: text next to the icon), `symbolgroesse` (icon size in points, 0 = from the height) |
| `GUI_TOOLBAR_ITEM_X(tb, eintrag)` | INTEGER | left edge of an entry in the window (like `GUI_GET_X`) |
| `GUI_TOOLBAR_ITEM_W(tb, eintrag)` | INTEGER | width of an entry |
| `GUI_TOOLBAR_OVERFLOW(tb)` | INTEGER | how many buttons are currently in the » menu (0 = everything fits) |
| `GUI_STATUSBAR(win, x, y, w, h[, text$])` | GUI_WIDGET | status bar; `text$` is the first field |
| `GUI_STATUSBAR_ADD(sb, text$, breite[, ausrichtung$])` | INTEGER | append a field: `breite` (width) in points, 0 = shares the rest; `links`/`mitte`/`rechts` (left/centre/right) |
| `GUI_STATUSBAR_SET(sb, feld, text$)` | — | text of a field (`GUI_SET_TEXT` means field 0) |
| `GUI_STATUSBAR_TEXT$(sb, feld)` | STRING | text of a field |
| `GUI_STATUSBAR_TIP(sb, feld, tip$)` | — | tooltip of a field |
| `GUI_STATUSBAR_CLICKABLE(sb, feld, an)` | — | make a field clickable |
| `GUI_STATUSBAR_CLICKED(sb)` | INTEGER | number of the field clicked in this frame, otherwise -1 |
| `GUI_STATUSBAR_COUNT(sb)` | INTEGER | number of fields |
| `GUI_STATUSBAR_FIELD_X(sb, feld)` | INTEGER | left edge of a field in the window |
| `GUI_STATUSBAR_FIELD_W(sb, feld)` | INTEGER | width of a field |
| `GUI_BREADCRUMB(win, x, y, w, h)` | GUI_WIDGET | breadcrumb bar |
| `GUI_BREADCRUMB_ADD(bc, text$[, wert$])` | INTEGER | append a part, with an invisible value |
| `GUI_BREADCRUMB_SET(bc, teile)` | — | all parts at once (ARRAY OF STRING); the same path again changes nothing |
| `GUI_BREADCRUMB_CLEAR(bc)` | — | remove all parts |
| `GUI_BREADCRUMB_COUNT(bc)` | INTEGER | number of parts |
| `GUI_BREADCRUMB_CLICKED(bc)` | INTEGER | number of the part clicked in this frame, otherwise -1 |
| `GUI_BREADCRUMB_TEXT$(bc, teil)` | STRING | text of a part |
| `GUI_BREADCRUMB_DATA$(bc, teil)` | STRING | invisible value of a part |
| `GUI_BREADCRUMB_FIRST_VISIBLE(bc)` | INTEGER | first visible part; the ones before it sit under "…" |
| `GUI_LABEL(win, text$, x, y[, farbe])` | GUI_WIDGET | text |
| `GUI_SET_ALIGN(wdg, wie$)` | — | align text left, `mitte` (centre) or `rechts` (right) (label, button, text input) |
| `GUI_SET_WRAP(label, breite)` | — | wrap a label at word boundaries at `breite` pixels (0 = off); the height follows the text |
| `GUI_WINDOW_DEFAULT(win, knopf)` | — | default button: Enter in the window triggers it (-1 = none) |
| `GUI_WINDOW_CANCEL(win, knopf)` | — | cancel button: ESC triggers it (-1 = none) |
| `GUI_CHECKBOX(win, label$, x, y[, default])` | GUI_WIDGET | toggle |
| `GUI_SLIDER(win, x, y, w, min, max[, default])` | GUI_WIDGET | value slider |
| `GUI_SPINNER(win, x, y, w, min, max[, default[, step]])` | GUI_WIDGET | number field with +/- (click/mouse wheel/arrow keys; value via `GUI_VALUE`) |
| `GUI_SPLITTER(win, x, y, length, orient$, min, max)` | GUI_WIDGET | movable divider (`"v"`/`"h"`); position via `GUI_VALUE` |
| `GUI_PANEL(win, x, y, w, h[, titel$])` | GUI_WIDGET | container (decoration) |
| `GUI_TEXTINPUT(win, x, y, w, h[, platzhalter$])` | GUI_WIDGET | single-line input field (cursor + selection, Ctrl+Z/Y) |
| `GUI_TEXTINPUT_SET(tf, key$, wert)` | — | `passwort` (password), `nur_lesen` (read-only), `maxlaenge` (max length), `zahlen` (numbers: 0 free, 1 integer, 2 decimal) |
| `GUI_ENTERED(tf)` | BOOLEAN | was Enter pressed in the text input in this frame? |
| `GUI_ON_ENTER(tf, handler)` | — | callback for it |
| `GUI_TEXTAREA(win, x, y, w, h[, platzhalter$])` | GUI_WIDGET | **multi-line** text field (ENTER = new line, scrolls vertically and horizontally, arrows, selection via mouse drag/Shift+arrow, Ctrl+A/C/X/V). Can be coloured as a **code field** — see [Code field](#the-textarea-as-a-code-field) |
| `GUI_TABLE(win, x, y, w, h[, headers, cells])` | GUI_WIDGET | scrollable table (header + body) |
| `GUI_TREE(win, x, y, w, h)` | GUI_WIDGET | tree view (expandable/collapsible, scrollable) |
| `GUI_TREE_ADD(tree, parent, label$)` | INT | append a node (parent = -1 = root), returns the node id |
| `GUI_TREE_CLEAR(tree)` | — | delete all nodes |
| `GUI_TREE_SELECTED(tree)` / `GUI_TREE_SET_SELECTED(tree, node)` | INT / — | read/set the selected node (-1 = none) |
| `GUI_TREE_LABEL(tree, node)` | STRING | text of a node |
| `GUI_TREE_EXPAND(tree, node, flag)` | — | expand/collapse a node |
| `GUI_TREE_SET(tree, key$, wert)` | — | setting on the tree: `mehrfachauswahl` (multi-select), `kaestchen` (checkboxes), `bearbeitbar` (editable: F2 renames) |
| `GUI_TREE_EDIT(tree, node)` | — | rename a node: input field over the name, everything selected (not for the file tree) |
| `GUI_TREE_EDITED(tree)` | INTEGER | node renamed in this frame, -1 = none |
| `GUI_TREE_EDITING(tree)` | INTEGER | node currently being renamed, -1 = none |
| `GUI_TREE_PLACEHOLDER(tree, text$)` | — | hint shown when the tree is empty |
| `GUI_TREE_SEL_COUNT(tree)` | INTEGER | how many nodes are selected |
| `GUI_TREE_SEL_NODE(tree, i)` | INTEGER | the i-th selected node id (-1 = no more) |
| `GUI_TREE_IS_SELECTED(tree, node)` | BOOLEAN | is this node selected? |
| `GUI_TREE_SELECT(tree, node, an)` | — | add a node to the selection or take it out |
| `GUI_TREE_CLEAR_SELECTION(tree)` | — | clear the selection |
| `GUI_TREE_CHECKED(tree, node)` | BOOLEAN | is the check mark set on this node (only with `kaestchen`)? |
| `GUI_TREE_SET_CHECKED(tree, node, an)` | — | set or remove the check mark |
| `GUI_FILETREE(win, x, y, w, h, wurzel$ = ".")` | GUI_WIDGET | file tree: fetches its nodes from disk itself, each folder only when it is expanded |
| `GUI_FILETREE_SET(ft, key$, wert)` | — | `ordner_zuerst` (folders first), `verborgene` (hidden files), `nur_ordner` (folders only), `mehrfachauswahl` (multi-select), `kaestchen` (checkboxes), `klick_klappt` (click expands), `auffrischen` (refresh, milliseconds, 0 = never) |
| `GUI_FILETREE_FILTER(ft, muster$)` | — | which files it shows (`*.dh;*.md`, empty = all) |
| `GUI_FILETREE_SKIP(ft, namen$)` | — | names or patterns it skips — folders and files alike (`target;__pycache__;_*`) |
| `GUI_FILETREE_SET_ROOT(ft, pfad$)` / `GUI_FILETREE_ROOT$(ft)` | — / STRING | set and read the root folder |
| `GUI_FILETREE_REFRESH(ft)` | — | read from disk again now |
| `GUI_FILETREE_SELECTED$(ft)` | STRING | full path of the selection (empty = none) |
| `GUI_FILETREE_SELECT(ft, pfad$)` | — | select a path; the folders above it expand for that, and the row is scrolled into view |
| `GUI_FILETREE_ACTIVATED$(ft)` | STRING | what was opened in this frame by double click or Enter (empty = nothing) |
| `GUI_FILETREE_IS_DIR(ft, pfad$ = "")` | BOOLEAN | is it a folder? Without a path the selection counts |
| `GUI_FILETREE_EXPAND(ft, pfad$, an)` | — | expand or collapse a folder |
| `GUI_FILETREE_COUNT(ft)` / `GUI_FILETREE_PATH$(ft, i)` | INTEGER / STRING | count and read the visible rows |
| `GUI_FILETREE_SEL_COUNT(ft)` / `GUI_FILETREE_SEL_PATH$(ft, i)` | INTEGER / STRING | count and read the multi-selection |
| `GUI_FILETREE_CHECKED(ft, pfad$)` / `GUI_FILETREE_SET_CHECKED(ft, pfad$, an)` | BOOLEAN / — | read and set the check mark on a path |
| `GUI_FILETREE_CHECKED_COUNT(ft)` / `GUI_FILETREE_CHECKED_PATH$(ft, i)` | INTEGER / STRING | all checked paths, including those currently collapsed |
| `GUI_FILETREE_ICONS(ft, ordnerbild, dateibild)` | — | icons for folders and files (-1 = none) |
| `GUI_TABCONTROL(win, x, y, w, h)` | GUI_WIDGET | tabs **inside** a window (card index) |
| `GUI_TABCONTROL_ADD(tc, titel$)` | INTEGER | append a page, returns its number |
| `GUI_TABCONTROL_ADD_WIDGET(tc, wdg, seite)` | — | put a widget on a page; it keeps its position in the window |
| `GUI_TABCONTROL_PAGE(tc)` / `GUI_TABCONTROL_SET_PAGE(tc, seite)` | INTEGER / — | read and set the front page |
| `GUI_TABCONTROL_COUNT(tc)` | INTEGER | how many pages there are |
| `GUI_TABCONTROL_TITLE$(tc, i)` / `GUI_TABCONTROL_SET_TITLE(tc, i, titel$)` | STRING / — | read and set the label of a page |
| `GUI_TABCONTROL_REMOVE(tc, i)` | — | remove a page; its children remain as widgets |
| `GUI_RICHTEXT(win, x, y, w, h, text$ = "")` | GUI_WIDGET | **typeset text**: Markdown is typeset instead of merely displayed |
| `GUI_RICHTEXT_SET_TEXT(rt, text$)` | — | set new Markdown source (`GUI_SET_TEXT` does the same) |
| `GUI_RICHTEXT_SET(rt, key$, wert)` | — | `groesse` (base font size), `codeschrift` (FONT handle for code blocks) and the colours of the code blocks `farbe_kommentar`/`farbe_text`/`farbe_zahl`/`farbe_schluessel`/`farbe_name`/`farbe_operator` (-1 = default matching the background) |
| `GUI_RICHTEXT_HEADINGS(rt)` | ARRAY OF STRING | the headings (`#` to `###`) without markup, from the source -- correct right after `SET_TEXT` |
| `GUI_RICHTEXT_HEADING_LEVELS(rt)` | ARRAY OF INTEGER | their levels 1..3, in the same order |
| `GUI_RICHTEXT_CODE_BUTTONS(rt, knoepfe$)` | — | buttons on every code block, separated with `\|` (e.g. `"Kopieren\|Starten"`); empty = none. The block gets a strip at the top for them |
| `GUI_RICHTEXT_CODE_ACTION$(rt)` | STRING | label of the code button pressed in this frame (empty = none); also fires `GUI_ON_CLICK` |
| `GUI_RICHTEXT_CODE$(rt)` | STRING | the code of the block on which a button was last pressed (without the fences) |
| `GUI_RICHTEXT_GOTO_HEADING(rt, nr)` | — | scroll to heading number `nr`; before typesetting the jump is queued |
| `GUI_RICHTEXT_HEADING_AT(rt)` | INTEGER | which section the top edge is in (-1 = before the first) -- for a table of contents that follows along |
| `GUI_RICHTEXT_LINK$(rt)` | STRING | which link was clicked in this frame (empty = none) |
| `GUI_RICHTEXT_FIND(rt, text$, ab = -1)` | INTEGER | scroll to the first match; returns its y or -1 (0 = queued: right after `SET_TEXT` the search only runs once the new text is typeset) |
| `GUI_RICHTEXT_SCROLL(rt, y)` / `GUI_RICHTEXT_SCROLL_GET(rt)` | — / INTEGER | set and read the view offset |
| `GUI_RICHTEXT_HEIGHT(rt)` | INTEGER | how tall the typeset text has become |
| `GUI_RICHTEXT_SELECTION$(rt)` | STRING | the selected text, lines separated by line breaks (empty = no selection) |
| `GUI_RICHTEXT_SELECT_ALL(rt)` | — | select everything (like Ctrl+A) |
| `GUI_RICHTEXT_CLEAR_SELECTION(rt)` | — | clear the selection |
| `GUI_RICHTEXT_FIND_NEXT(rt, text$)` | INTEGER | select the next match from the selection on and scroll it into view, wrapping around; y of the line or -1 |
| `GUI_RICHTEXT_FIND_PREV(rt, text$)` | INTEGER | previous match, wrapping around; y of the line or -1 |
| `GUI_RICHTEXT_MARK_ALL(rt, text$)` | INTEGER | highlight all matches faintly (empty = off); returns their number |
| `GUI_TIMEPICKER(win, x, y, w = 0, h = 0)` | GUI_WIDGET | time of day: fields with arrows for hour, minute and (optionally) second |
| `GUI_TIME$(tp)` | STRING | the time as `HH:MM:SS` — like `TIME$()`, even without a seconds field |
| `GUI_SET_TIME(tp, zeit$)` | — | set the time (`HH:MM` or `HH:MM:SS`; invalid values are an error) |
| `GUI_TIMEPICKER_SET(tp, key$, wert)` | — | `sekunden` (third field, seconds) and `schritt` (step size of the minute) |
| `GUI_ACCORDION(win, x, y, w, h)` | GUI_WIDGET | **accordion**: sections with a header that expand and collapse; whatever lies below moves down when expanding |
| `GUI_ACCORDION_ADD(acc, titel$)` | INTEGER | append a section (collapsed), returns its number |
| `GUI_ACCORDION_ADD_WIDGET(acc, wdg, abschnitt)` | — | put a widget in a section; its position counts from the section (below the header) |
| `GUI_ACCORDION_OPEN(acc, abschnitt, an)` | — | expand or collapse a section; without `mehrere`, expanding closes the others |
| `GUI_ACCORDION_IS_OPEN(acc, abschnitt)` | BOOLEAN | is the section open? |
| `GUI_ACCORDION_COUNT(acc)` | INTEGER | how many sections there are |
| `GUI_ACCORDION_TITLE$(acc, abschnitt)` / `GUI_ACCORDION_SET_TITLE(acc, abschnitt, titel$)` | STRING / — | read and set the header of a section |
| `GUI_ACCORDION_SET_HEIGHT(acc, abschnitt, px)` | — | fixed height of an open section (0 = by content) |
| `GUI_ACCORDION_SET(acc, key$, wert)` | — | `mehrere` (several): several sections open at once (off by default) |
| `GUI_ACCORDION_TOGGLED(acc)` | INTEGER | which section was toggled in this frame (-1 = none) |
| `GUI_WIZARD(win, x, y, w, h)` | GUI_WIDGET | **wizard**: steps with a step indicator and Back/Next/Finish/Cancel |
| `GUI_WIZARD_ADD(wz, titel$)` | INTEGER | append a step, returns its number |
| `GUI_WIZARD_ADD_WIDGET(wz, wdg, schritt)` | — | put a widget on a step; it keeps its position in the window |
| `GUI_WIZARD_STEP(wz)` / `GUI_WIZARD_SET_STEP(wz, schritt)` | INTEGER / — | read and set the current step |
| `GUI_WIZARD_COUNT(wz)` | INTEGER | how many steps there are |
| `GUI_WIZARD_NEXT(wz)` | BOOLEAN | like the Next button (on the last step: Finish); FALSE if disabled or the validation fails |
| `GUI_WIZARD_BACK(wz)` | BOOLEAN | like the Back button; FALSE on the first step |
| `GUI_WIZARD_ENABLE_NEXT(wz, an)` | — | disable or enable Next/Finish |
| `GUI_WIZARD_CHANGED(wz)` | BOOLEAN | step changed in this frame? |
| `GUI_WIZARD_FINISHED(wz)` | BOOLEAN | Finish pressed in this frame? |
| `GUI_WIZARD_CANCELLED(wz)` | BOOLEAN | Cancel pressed in this frame? |
| `GUI_WIZARD_LABELS(wz, zurueck$, weiter$, fertig$, abbrechen$)` | — | labels of the four buttons (say, for an English interface) |
| `GUI_WIZARD_SET(wz, key$, wert)` | — | `pruefen` (validate): Next only once the fields of this step satisfy their `GUI_RULE`s |
| `GUI_TREETABLE(win, x, y, w, h [, kopf])` | GUI_WIDGET | **tree table**: a table whose rows form a tree (indentation and triangle in the first column) |
| `GUI_TREETABLE_ADD(tt, eltern, zellen)` | INTEGER | append a row under `eltern` (parent, -1 = top level); returns the data row |
| `GUI_TREETABLE_EXPAND(tt, zeile, an)` | — | expand or collapse a row |
| `GUI_TREETABLE_EXPAND_ALL(tt, an)` | — | expand or collapse everything |
| `GUI_TREETABLE_EXPANDED(tt, zeile)` | BOOLEAN | is the row expanded? |
| `GUI_TREETABLE_PARENT(tt, zeile)` | INTEGER | parent row (-1 = top level) |
| `GUI_TREETABLE_LEVEL(tt, zeile)` | INTEGER | depth in the tree (0 = top level) |
| `GUI_TREETABLE_SET_PARENT(tt, zeile, eltern)` | — | move a row to another parent; under itself or one of its descendants is an error |
| `GUI_TABLE_HEADERS(tbl, headers)` | — | set the column titles (1D ARRAY OF STRING) |
| `GUI_TABLE_ROWS(tbl, cells)` | — | set the data rows (2D ARRAY OF STRING) |
| `GUI_TABLE_COL_WIDTHS(tbl, widths)` | — | column widths (1D ARRAY OF INTEGER; NIL = auto) |
| `GUI_TABLE_SELECTED(tbl)` | INTEGER | selected row (-1 if none) |
| `GUI_TABLE_SET_SELECTED(tbl, row)` | — | set the selection (-1 = none) |
| `GUI_TABLE_CLICKED(tbl)` | INTEGER | row clicked in this frame (-1) |
| `GUI_TABLE_ROW_COUNT(tbl)` | INTEGER | number of data rows |
| `GUI_UPDATE()` | — | **required** every frame: process mouse/keys |
| `GUI_DRAW([obenauf])` | — | draw all windows (back→front); `obenauf`=FALSE leaves out the context menu and tooltip |
| `GUI_DRAW_TOP()` | — | only what lies above all windows: open context menu and tooltip |
| `GUI_DRAW_WINDOW(win)` | — | draw ONE window again, above everything added since `GUI_DRAW` |
| `GUI_CLICKED(widget)` | BOOLEAN | clicked in this frame? Applies to button, menu entry, **checkbox, toggle and radio** and to a click on a list entry -- also the one already selected |
| `GUI_CHECKED(widget)` | BOOLEAN | checkbox state |
| `GUI_VALUE(widget)` | FLOAT | slider value |
| `GUI_TEXT(widget)` | STRING | text (label/button/TextInput) |
| `GUI_HOVERED(widget)` | BOOLEAN | mouse over the widget? |
| `GUI_SET_TEXT(widget, text$)` | — | set the text |
| `GUI_TOOLTIP(widget, text$)` | — | hover help text (multi-line via `\n`; "" removes it) |
| `GUI_SET_CHECKED(widget, an)` | — | set a checkbox |
| `GUI_SET_VALUE(widget, wert)` | — | set the slider value (gets clamped) |
| `GUI_ON_CLICK(widget, funcref)` | — | FUNCREF callback on click (button/checkbox) |
| `GUI_ON_CHANGE(widget, funcref)` | — | FUNCREF callback on value change (slider/TextInput/checkbox/table selection) |
| `GUI_THEME(accent)` | — | change the accent colour (RGB) (short form) |
| `GUI_THEME_SET(key$, farbe)` / `GUI_THEME_GET(key$)` | — / INT | set/read a single theme colour |
| `GUI_THEME_PRESET(name$)` | — | look: dark/light/retro/contrast + **modern_dark/modern_light** (rounded + shadows) |
| `GUI_METRIC_SET(key$, wert)` / `GUI_METRIC_GET(key$)` | — / INT | set/read a layout size |
| `GUI_SET_COLOR(widget, rolle$, farbe)` | — | one colour per widget (bg/fg/border/accent, for buttons also hover/pressed; -1 removes it) |
| `GUI_RESET()` | — | delete windows/widgets + reset theme/metrics |

**A press belongs to the window on top:** a text area with focus only reacts to a press that begins inside it and whose window is on top there — a list over the code can be clicked and dragged without the cursor jumping or text getting selected underneath.

**Tooltips:** `GUI_TOOLTIP(widget, text$)` attaches a help text to any widget. It appears automatically as soon as the mouse rests still over the widget for ~0.5 s (only in the topmost window; if a modal window is in front, only that one shows tips), and follows the cursor, clamped at the screen edge. `\n` makes several lines; `""` removes the tooltip again. Movement or a mouse click resets the resting time.

### Menus

| Function | Returns | Purpose |
|---|---|---|
| `GUI_MENU(win, label$)` | menu handle | top-level menu in the **menu bar** (e.g. "File") |
| `GUI_CONTEXT(win)` | menu handle | **context menu** (by right-click in the window) |
| `GUI_CONTEXT_TABS(menu)` | — | bind a context menu to the window's tab bar: it opens on a right-click on a tab |
| `GUI_TAB_CONTEXT(win)` | INTEGER | on which tab the tab bar's context menu last opened (-1 = never yet) |
| `GUI_CONTEXT_WIDGET(menu, wdg)` | — | bind a context menu to a widget: it only opens on a right-click on it (list box, tree, table select their row while doing so, a text area sets the cursor; not in its line-number column, which belongs to `GUI_TEXTAREA_GUTTER_CLICKED`); `-1` binds it back to the window |
| `GUI_SUBMENU(menu, label$)` | menu handle | **submenu** — opens to the right on hover, any depth |
| `GUI_MENU_ITEM(menu, label$[, kuerzel$])` | item handle | append an entry — handle for `GUI_CLICKED`; `kuerzel$` (shortcut) e.g. `"Strg+S"` |
| `GUI_MENU_SEPARATOR(menu)` | — | append a separator line |
| `GUI_MENU_SHORTCUT(item, kuerzel$)` | — | set a shortcut or remove it with `""` |
| `GUI_MENU_ENABLE(item, an)` | — | disable an entry (grey, no click, no shortcut) or enable it |
| `GUI_MENU_CHECK(item, an)` | — | makes the entry a **check-mark entry** and sets it; afterwards a click toggles it by itself |
| `GUI_MENU_CHECKED(item)` | BOOLEAN | state of the check mark |
| `GUI_MENU_ICON(item, bild)` | — | icon to the left of the text (name of a built-in icon or texture handle as with `GUI_ICON_BUTTON`, -1 removes it) |
| `GUI_MENU_TEXT(item, label$)` | — | change the label ("Pause" / "Continue") |

Clicks are evaluated as with buttons via `GUI_CLICKED(item)`. The menu bar pushes the window content down automatically; a click on a menu opens the dropdown, a click elsewhere closes it. Complete example: [`examples/129_gui_menu.dh`](../../examples/129_gui_menu.dh).

**Operation like the list box** (since 2026-09-23):

* **Context menus** can be operated by keyboard like the menu bar:
  `↑`/`↓` move, `→` opens a submenu, `←` closes it, `ENTER`
  selects, `ESC` closes — without pressing the window's cancel button.
  While a menu is open, the keys belong to it.
* **`POS1`/`ENDE`** (Home/End) jump to the first or last usable entry.
* **Typing jumps**: the initial letter marks the next entry that starts
  with it. If it is **unique**, the entry is triggered right away
  (or its submenu opened) — as in any menu without underlined
  letters.
* **At the screen edge** a popup moves out of the way: a context menu then
  sits to the left of or above the mouse instead of running off the screen, a
  submenu opens to the left of its parent menu, a bar menu moves to the left.

**Keyboard shortcuts** appear on the right of the entry and are checked every frame — even with the menu closed, and **in all visible windows of the program**: first in the window with focus, then in the others from top to bottom. A program with a toolbar on the left and an inspector on the right has one menu, and `Strg+S` saves even when the last click went into the inspector (until 2026-09-07 a shortcut only applied in the window with focus — three editors had to take the focus back after every button). If the focus window has the same shortcut itself, it wins. A modal window only allows its own shortcuts; a window in design mode (`GUI_WINDOW_DESIGN`) does not count, its menus are just a view. They are written the way you read them: `Strg+S`, `Strg+Umschalt+O`, `Alt+Enter`, `F5`, `Entf`; English names (`Ctrl`, `Shift`, `Delete`, `PageDown`) work too. **`Plus` and `Minus`** (also `Strg++` and `Strg+-`) are not a single key: raylib names keys by their position in the US layout, and the "+" of a German keyboard sits where the US keyboard has "]", its "-" on "/" -- such a shortcut therefore hits the key of both layouts and the numeric keypad (`Strg+Plus`, `Strg+Minus`, `Strg+0` for zoom). **Letters follow the layout**: `Strg+Z` is the key labelled Z — on a German keyboard it sits where the US keyboard has the Y (until 2026-10-01 Ctrl+Y therefore undid and Ctrl+Z redid, in every text field). This applies to menu shortcuts and the keys of the text fields (Ctrl+A/C/V/X/Z/Y/B/I/U); `KEYHIT`/`KEYPRESSED` in programs stay with the position, a game wants WASD where the fingers are. While a recording is playing, the position applies here too. The modifiers must match **exactly**: a plain S is not Ctrl+S. **Without Ctrl or Alt a key belongs to the text field with focus** — a `Entf` shortcut deletes a character there instead of triggering the menu item; without text focus it triggers. **F1 to F12 are exempt**: they never produce text and trigger even from inside the text field (F5 starts in an IDE from the code field). A disabled entry has no shortcut. An unknown key name is an error when creating, not a shortcut that silently never fires. **A shortcut that has fired takes the key away from the text area** — otherwise `Strg+Umschalt+Hoch` would do two things: trigger the entry and push the selection up one line, and a command that reads the selection afterwards would see a different one than the one marked.

### Ctrl+wheel belongs to the program

The mouse wheel scrolls list boxes, tables, trees, text areas, typeset text,
scrolling panels and scrollable windows -- but **not with the Ctrl key
held down**: Ctrl+wheel means "zoom" in every editor, and if the content
under the mouse scrolled at the same time, it would jump away at every step.
The program then asks itself: `IF KEYPRESSED(KEY_LCTRL) AND MOUSEWHEEL_Y() <> 0 THEN ...`.
A **text area** has scrolled three lines per step since 2026-09-18 (before
that not at all); the cursor does not pull the view back as long as neither
it nor the text changes -- a key that moves it pulls the view back to it.

**Submenus** are created with `GUI_SUBMENU` and receive their entries like any menu; they open on hover and stay open while you move diagonally across (only another entry on the same level closes them). In the `.dhform` they are stored **nested** on the entry (`items`), shortcuts as `shortcut`, check marks as `checkable`/`checked`. Icons are texture handles and, as with `GUI_IMAGE`, are not saved. The form designer does not edit menus, but passes them through unchanged and writes them into the DH code.

### Tabs + keyboard navigation

| Function | Returns | Purpose |
|---|---|---|
| `GUI_TABS(win, labels)` | — | create a tab bar (`labels` = ARRAY/TUPLE of strings); active tab → 0 |
| `GUI_SET_TAB(widget, seite)` | — | assign a widget to a tab (`-1` = visible on all tabs) |
| `GUI_ACTIVE_TAB(win)` | INTEGER | index of the active tab |
| `GUI_SET_ACTIVE_TAB(win, i)` | — | switch tabs |

Only the widgets of the active tab (plus those with `tab_page = -1`) are drawn and usable. **Keyboard:** `TAB` / `SHIFT+TAB` moves the focus between **all usable widgets** of the active window — see [Operation without a mouse](#operation-without-a-mouse). Example: [`examples/131_gui_tabs.dh`](../../examples/131_gui_tabs.dh).

### Modal dialogs

Native, blocking standard dialogs (no IMPORT needed — like the file dialogs):

| Function | Returns | Purpose |
|---|---|---|
| `GUI_MESSAGE(titel$, text$)` | — | info box with OK |
| `GUI_CONFIRM(titel$, text$[, stil$])` | BOOLEAN | confirmation question → `TRUE` on agreement |
| `GUI_RADIO(win, group$, text$, x, y)` | GUI_WIDGET | radio button; all with the same `group$` exclude each other |
| `GUI_RADIO_SELECTED(radio)` | INTEGER | which one of the group is selected? (creation order from 0, `-1` = none) |
| `GUI_DROPDOWN(win, x, y, w, h, items)` | GUI_WIDGET | drop-down selection list; the popup is drawn above all other widgets |
| `GUI_DROPDOWN_SELECTED(dd)` | INTEGER | index of the selection (`-1` = none) |
| `GUI_DROPDOWN_TEXT(dd)` | STRING | text of the selection |
| `GUI_DROPDOWN_SET_SELECTED(dd, i)` | — | set the selection from the program |
| `GUI_SET_DROPDOWN(dd, items)` | — | replace the entries |
| `GUI_DROPDOWN_PLACEHOLDER(dd, text$)` | — | hint in the box while nothing is selected ("Please choose") |
| `GUI_DROPDOWN_SET(dd, key$, wert)` | — | setting of the dropdown; `bearbeitbar` (editable) = you may type into the field, including what is not in the list (combo box) |
| `GUI_LISTBOX(win, x, y, w, h, items)` | GUI_WIDGET | scrollable selection list (the mouse wheel scrolls; with the Ctrl key held down no widget scrolls, see the section "Ctrl+wheel belongs to the program") |
| `GUI_LISTBOX_SELECTED(lb)` | INTEGER | index of the selection (`-1` = none) |
| `GUI_LISTBOX_TEXT(lb)` | STRING | text of the selection |
| `GUI_LISTBOX_SET_SELECTED(lb, i)` | — | set the selection from the program |
| `GUI_SET_LISTBOX(lb, items)` | — | replace the entries |
| `GUI_LISTBOX_ADD(lb, text$[, pos])` / `GUI_LISTBOX_REMOVE(lb, i)` / `GUI_LISTBOX_CLEAR(lb)` | INTEGER / — / — | entries one by one (also for dropdowns); omit `pos` = append at the end |
| `GUI_LISTBOX_COUNT(lb)` / `GUI_LISTBOX_ITEM(lb, i)` / `GUI_LISTBOX_SET_ITEM(lb, i, text$)` / `GUI_LISTBOX_MOVE(lb, von, nach)` | — | count, read, relabel, move |
| `GUI_LISTBOX_SET(lb, key$, wert)` | — | `mehrfachauswahl` (multi-select), `kaestchen` (checkboxes), `bearbeitbar` (editable: F2 renames) |
| `GUI_LISTBOX_ICON(lb, i, bild)` / `GUI_LISTBOX_COLOR(lb, i, farbe)` | — | icon and text colour per entry (-1 = none / theme) |
| `GUI_LISTBOX_CHECKED(lb, i)` / `GUI_LISTBOX_SET_CHECKED(lb, i, an)` | BOOLEAN / — | check mark per entry (with `kaestchen`) |
| `GUI_LISTBOX_IS_SELECTED(lb, i)` / `GUI_LISTBOX_SELECT(lb, i, an)` / `GUI_LISTBOX_SEL_COUNT(lb)` / `GUI_LISTBOX_SEL_ROW(lb, k)` / `GUI_LISTBOX_CLEAR_SELECTION(lb)` | — | multi-selection as in the table |
| `GUI_LISTBOX_SET_DATA(lb, i, wert$)` | — | invisible value per entry (an ID, a path) |
| `GUI_LISTBOX_DATA$(lb, i)` | STRING | read the invisible value of an entry |
| `GUI_LISTBOX_FIND_DATA(lb, wert$)` | INTEGER | first entry with this value, -1 = none |
| `GUI_LISTBOX_DETAIL(lb, i, text$)` | — | extra text on the right of the entry, dimmed (shortcut, size, line number) |
| `GUI_LISTBOX_TIP(lb, i, text$)` | — | tooltip per entry |
| `GUI_LISTBOX_ENABLE(lb, i, an)` | — | disable an entry: visible, but neither clickable nor reachable with the arrow keys |
| `GUI_LISTBOX_ENABLED(lb, i)` | BOOLEAN | whether an entry is selectable |
| `GUI_LISTBOX_HEADER(lb, i, an)` | — | make an entry a group header: structures the list, is not selectable |
| `GUI_LISTBOX_FILTER(lb, text$)` | — | only show entries whose text or extra text contains the substring (empty = all) |
| `GUI_LISTBOX_GET_FILTER$(lb)` | STRING | the current filter |
| `GUI_LISTBOX_VIEW_COUNT(lb)` | INTEGER | number of visible rows |
| `GUI_LISTBOX_VIEW_ROW(lb, k)` | INTEGER | the k-th visible row as an entry number, -1 = none |
| `GUI_LISTBOX_PLACEHOLDER(lb, text$)` | — | hint that appears when there is nothing to see |
| `GUI_LISTBOX_SPANS(lb, eintrag, starts, laengen, farben)` | — | colour sections of an entry (characters from 0, three equally long ARRAY OF INTEGER; empty removes them) -- say, the keyword in one colour, the name in another. On the selection and when disabled one colour applies. The row height of a list box follows its font (`GUI_SET_FONT_SIZE`) |
| `GUI_LISTBOX_SORT(lb[, absteigend])` | — | sort naturally ("File 9" before "File 10"), within the groups |
| `GUI_LISTBOX_FIND(lb, text$[, ab])` | INTEGER | next entry from `ab` on that contains the substring, -1 = none |
| `GUI_LISTBOX_SCROLL_TO(lb, i)` | — | scroll so that the entry is visible |
| `GUI_LISTBOX_EDIT(lb, i)` | — | rename entry i: input field over the row, everything selected |
| `GUI_LISTBOX_EDITED(lb)` | INTEGER | entry renamed in this frame, -1 = none |
| `GUI_LISTBOX_EDITING(lb)` | INTEGER | entry currently being renamed, -1 = none |
| `GUI_DOUBLE_CLICKED(lb)` | BOOLEAN | double click (or Enter) on an entry, for one frame; also for tables and trees |
| `GUI_IMAGE(win, x, y, w, h, image)` | GUI_WIDGET | image or icon in the window |
| `GUI_SET_IMAGE(widget, image)` | — | replace the image (also that of a card) |
| `GUI_CARD(win, x, y, w, h, bild[, titel$[, text$]])` | GUI_WIDGET | card: image at the top, title, description -- the whole thing is a button (`GUI_CLICKED`, Enter/Space); `bild` -1 = without |
| `GUI_CARD_SET_TEXT(card, text$)` | — | set the description of a card (the title is set by `GUI_SET_TEXT`) |
| `GUI_CARD_TEXT$(card)` | STRING | description of a card |
| `GUI_CANVAS(win, x, y, w, h)` | GUI_WIDGET | free drawing surface -- you paint into it with the normal drawing commands, **after** `GUI_DRAW` |
| `GUI_CANVAS_X(canvas)` / `GUI_CANVAS_Y(canvas)` | INTEGER | **absolute** screen position of the surface (moves with the window) |
| `GUI_CANVAS_W(canvas)` / `GUI_CANVAS_H(canvas)` | INTEGER | size of the surface |
| `GUI_SET_ENABLED(wdg, on)` | — | make a widget usable or disable it (disabled = greyed out, takes no clicks) |
| `GUI_ENABLED(wdg)` | BOOLEAN | is it usable? |
| `GUI_SET_FONT(wdg, font)` | — | own font for this widget |
| `GUI_SET_FONT_SIZE(wdg, px)` | — | own font size for this widget |
| `GUI_SET_FONT_STYLE(wdg, stil$)` | — | font style for this widget: `fett` (bold), `kursiv` (italic), `unterstrichen` (underlined), `durchgestrichen` (strikethrough), joined with `+`; `"normal"` removes it. Real font faces where they exist as files, otherwise emulated |
| `GUI_GET_FONT_STYLE$(wdg)` | STRING | the widget's style, e.g. `"fett+unterstrichen"` |
| `GUI_STYLE_SET(name$, prop$, wert)` | — | define a named style (`bg`, `fg`, `border`, `accent`, `font`, `font_size`) |
| `GUI_APPLY_STYLE(widget, name$)` | — | apply a named style to a widget -- saves repeating it widget by widget |
| `GUI_GET_Y(wdg)` / `GUI_GET_W(wdg)` / `GUI_GET_H(wdg)` | INTEGER | position and size of the widget in the window (counterpart to `GUI_SET_BOUNDS`) |
| `GUI_WINDOW_GET_Y(win)` / `GUI_WINDOW_GET_W(win)` / `GUI_WINDOW_GET_H(win)` | INTEGER | position and size of the window on the screen |
| `GUI_WINDOW_CONTENT_W(win)` / `GUI_WINDOW_CONTENT_H(win)` | INTEGER | width and height of the content area below the title, menu and tab bar -- widget coordinates count from there; a status bar sits at the bottom without guessing the height of the menu bar |
| `GUI_TABLE_SET(tbl, schluessel$, wert)` | — | set a table setting (`zebra`, `gitter`, `zeilenhoehe`, `filterzeile`, `feste_spalten`, ...) |
| `GUI_TABLE_GET(tbl, schluessel$)` | FLOAT | read one of these settings back |
| `GUI_TABLE_CLICKED_COL(tbl)` | INTEGER | which column was clicked? -- for cells of the kind `knopf` (button) |
| `GUI_TABLE_VIEW_COUNT(tbl)` | INTEGER | how many rows are currently VISIBLE? (after filtering) |
| `GUI_TABLE_VIEW_ROW(tbl, i)` | INTEGER | which **data row** is at visible position `i`? -- sorting and filtering do not rearrange the data |
| `GUI_GRID(win, x, y, w, h, headers, zeilen = 0)` | GUI_WIDGET | **grid**: table in cell mode -- current cell, range, typing edits, Ctrl+C/V as tab-separated text |

`stil$` labels the buttons: `"ok"` (default) shows **OK/Cancel**,
`"janein"` shows **Yes/No**. The difference is not cosmetic — with a
question, "Cancel" reads as "close the dialog", "No" as an answer.
Rule of thumb: instruction ("Delete") → `"ok"`, question ("Really delete?") → `"janein"`.

```basic
IF GUI_CONFIRM("Löschen", "Alle Einträge werden entfernt.") THEN GUI_SET_TEXT(ta, "")

IF GUI_CONFIRM("Sicherung einspielen?", "Alles seitdem geht verloren.", "janein") THEN
    GUI_MESSAGE("Fertig", "Sicherung eingespielt.")
END IF
```

Both dialogs **block** until answered — the window behind them does not
draw in the meantime. For a confirmation question that is right: nobody
should be able to keep clicking while the question is open.

Example with TextArea + dialogs: [`examples/132_gui_textarea.dh`](../../examples/132_gui_textarea.dh).

#### `GUI_DIALOG` — the same dialog, but in your own window

`GUI_MESSAGE`/`GUI_CONFIRM` open a **box of the operating system**. That is
the right choice for a tool — it looks like everything else on the
computer. For a fullscreen game it often is not: it breaks the look, appears
as a separate OS window, and in the web build it does not exist at all
(`dialogs` feature).
`GUI_DIALOG` is the alternative **inside** your window: your theme,
your scale, everything behind it is dimmed and no longer accepts
clicks.

| Function | Returns | Purpose |
|---|---|---|
| `GUI_DIALOG(titel$, text$[, stil$])` | GUI_WINDOW | open a modal dialog (`"ok"` = default, `"janein"`, or **your own buttons** separated with `\|`: `"Speichern\|Verwerfen\|Abbrechen"`) |
| `GUI_PROMPT(titel$, text$[, vorgabe$[, knoepfe$]])` | GUI_WINDOW | question **with an input field** (default value selected; without `knoepfe$` OK/Cancel) |
| `GUI_ANSWER(dialog)` | INTEGER | `0` = still open, otherwise the **number of the button** (1-based): `1` = OK/Yes, `2` = Cancel/No |
| `GUI_DIALOG_TEXT(dialog)` | STRING | what was typed into the input field of a `GUI_PROMPT` — also after the answer |
| `GUI_MODAL()` | BOOLEAN | is a dialog or a modal window currently up? |
| `GUI_WINDOW_MODAL(win, an)` | — | make one of **your own** windows modal: everything else lies behind the overlay and accepts no input |

**Your own buttons** (since 2026-09-04): one to five labels, separated with
`|`. The answer is the number of the button. **Enter presses the first,
ESC and the close cross the last** — the convention of every system dialog,
Cancel sits on the right. The button width follows the longest label.

**`GUI_PROMPT`** is the same dialog with an input field between text and
buttons; the focus is in the field, the default value is selected (the first
keystroke replaces it), Enter is OK. `GUI_DIALOG_TEXT(dialog)` fetches the
text — also in the frame in which `GUI_ANSWER` answers, and afterwards: the
handle stays valid.

**`GUI_WINDOW_MODAL`** turns any window of your own into a dialog with
everything you build into it (checkboxes, sliders, lists): clicks elsewhere
are swallowed, menu shortcuts of other windows fall silent, the overlay lies
over everything else. Unlike with `GUI_DIALOG`, **a button in the window
ends nothing** — the program releases it when it is ready (`GUI_WINDOW_MODAL(win,
FALSE)`), and hiding or destroying the window releases it by itself.
`GUI_WINDOW_DEFAULT`/`GUI_WINDOW_CANCEL` supply Enter and ESC for it.
Example: [`examples/193_gui_dialoge.dh`](../../examples/193_gui_dialoge.dh).

`\n` in the text (`CHR$(10)`) separates lines; the window adapts to the text
and the number of lines and is centred on the screen.

```basic
DIM frage AS GUI_WINDOW
frage = -1                  ' -1 = none open right now

' ... during the game:
IF GUI_CLICKED(loeschen) AND frage < 0 THEN
    frage = GUI_DIALOG("Löschen", "Eintrag wirklich löschen?", "janein")
END IF

GUI_UPDATE()
IF frage >= 0 THEN
    IF GUI_ANSWER(frage) = 1 THEN eintrag_loeschen() : frage = -1
    IF GUI_ANSWER(frage) = 2 THEN frage = -1
END IF
```

> **Remember the open dialog with `-1` as "none".** Not with `0` —
> window handles count from 0, and of all numbers the `0` is the **first
> window of your program**. `GUI_ANSWER` on a negative handle therefore
> returns `0` ("no answer"), so that the query also runs through when no
> dialog is up at all.

**`GUI_DIALOG` does not block.** It gives you back a window, and you fetch
the answer with `GUI_ANSWER` — just like a click with `GUI_CLICKED`. That is
not a compromise but the way the module is built: a blocking dialog would
have to spin its own drawing loop in the middle of your frame and take over
your layer and render-target state while doing so.

> **The answer is valid for exactly one frame** — like `GUI_CLICKED`. An
> answer is an event, not a state. Whoever needs it longer writes it into a
> variable. After that the dialog window is gone; its handle stays valid,
> `GUI_ANSWER` then returns `0`.

**When to use which?**

| | `GUI_MESSAGE` / `GUI_CONFIRM` | `GUI_DIALOG` |
|---|---|---|
| Appearance | box of the operating system | your theme, your scale |
| Flow | blocks, returns the answer directly | keeps running, answer via `GUI_ANSWER` |
| Web build | no | yes |
| suits | tool, editor | game, fullscreen application |

Demo for all three new features (keyboard, scale, dialog):
[`examples/182_gui_tastatur_massstab.dh`](../../examples/182_gui_tastatur_massstab.dh).

`labels` is an `ARRAY OF STRING` — easiest via `SPLIT$`:

```basic
DIM tabs AS ARRAY OF STRING : tabs = SPLIT$("Allgemein|Konto", "|")
GUI_TABS(win, tabs)
DIM nameI AS GUI_WIDGET : nameI = GUI_TEXTINPUT(win, 24, 44, 400, 32, "Name ...")
GUI_SET_TAB(nameI, 0)                 ' only on tab "Allgemein"
DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(win, "OK", 24, 300, 200, 38)   ' without SET_TAB -> always visible
```

```basic
DIM mFile AS INTEGER : mFile = GUI_MENU(win, "Datei")
DIM miOpen AS INTEGER : miOpen = GUI_MENU_ITEM(mFile, "Öffnen ...")
GUI_MENU_SEPARATOR(mFile)
DIM miQuit AS INTEGER : miQuit = GUI_MENU_ITEM(mFile, "Beenden")
' ... in the loop:
IF GUI_CLICKED(miOpen) THEN ...
```

## External types

`gui` registers two types that you can declare with `DIM`:

```basic
DIM win AS GUI_WINDOW
DIM btn AS GUI_WIDGET
```

All widgets (button, label, checkbox, …) have the same type `GUI_WIDGET` —
what kind of widget it is follows from the constructor function.

## Concept: build once, then poll every frame

```basic
IMPORT "gui"
SCREEN(480, 360, "GUI", 2)

' --- one-time setup ---
DIM win AS GUI_WINDOW
win = GUI_WINDOW("Einstellungen", 80, 50, 300, 220)
GUI_WINDOW_MOVABLE(win, TRUE)
GUI_WINDOW_CLOSABLE(win, TRUE)

DIM ok AS GUI_WIDGET
ok = GUI_BUTTON(win, "Start", 20, 150, 110, 32)
DIM snd AS GUI_WIDGET
snd = GUI_CHECKBOX(win, "Sound an", 20, 60, TRUE)

' --- frame loop ---
WHILE NOT QUITREQUESTED()
    CLS(&H101828)
    GUI_UPDATE()                 ' mouse/keys: hover, click, drag, focus, z-order
    GUI_DRAW()                   ' all windows back→front

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

`GUI_UPDATE()` must come **before** `GUI_DRAW()` and before the polling
queries — it processes this frame's mouse/key events and sets the internal
flags (clicked, hovered, slider value, focus, window z-order).

## Coordinates

Widget coordinates `(x, y)` are **relative to the client area** of their
window (below the 22 px high title bar). If the window moves, the widgets
move along automatically.

## Window

```basic
GUI_WINDOW(titel$, x, y, w, h) -> GUI_WINDOW
```

Creates a window and returns it. New windows appear **on top**
(frontmost z-order). A click into a window brings it to the front.

- `GUI_WINDOW_MOVABLE(win, TRUE)` — drag on the title bar (default: TRUE)
- `GUI_WINDOW_CLOSABLE(win, TRUE)` — close button (×) at the top right
- `GUI_WINDOW_VISIBLE(win, FALSE)` — hide/show (showing resets the
  closed flag)
- `GUI_WINDOW_CLOSED(win)` — TRUE as soon as the × button was pressed (stays
  TRUE until the window is set visible again)

## Button

```basic
GUI_BUTTON(win, text$, x, y, w, h) -> GUI_WIDGET
```

`GUI_CLICKED(btn)` returns **TRUE** in exactly the frame in which the mouse
was **released** over the button (after it was pressed on it before —
press-and-release over the same button, like a classic OK button).

The label is centred; `GUI_SET_ALIGN(btn, "links")` or `"rechts"`
moves it (with an icon, its space on the left stays reserved).

### Default and cancel button

```basic
GUI_WINDOW_DEFAULT(win, okKnopf)      ' Enter in the window presses it
GUI_WINDOW_CANCEL(win, abbruchKnopf)  ' ESC presses it
```

What every form needs and what every program otherwise rebuilt with `KEYHIT`,
without paying attention to the focus. The default button carries the accent
as a border, so that you can see what Enter will do. The key belongs first,
however, to the widget with focus: a button or checkbox with focus takes
Enter itself, a text area turns it into a line break, a table cell being
edited into its end. **From inside a text input**, by contrast, Enter is
exactly that: submitting the form — the field reports `GUI_ENTERED` **and**
the default button clicks. ESC has no effect while a dropdown is open.
Both are written into the `.dhform` (`default_button`/`cancel_button`,
as a widget index).

## Label

```basic
GUI_LABEL(win, text$, x, y[, farbe]) -> GUI_WIDGET
```

Static text. Default colour white. Can be changed at any time with
`GUI_SET_TEXT(lbl, ...)` (e.g. to show a slider value live).

```basic
GUI_SET_WRAP(lbl, 220)              ' wrap at word boundaries at 220 px
GUI_SET_ALIGN(lbl, "rechts")        ' within the widget width
```

**Wrapping:** `GUI_SET_WRAP(lbl, breite)` sets the width and lets the text
wrap at word boundaries; a word that does not fit into the line on its own is
split at a character boundary, `\n` stays a line break. The **height follows
the text** — `GUI_GET_H(lbl)` returns it from the next `GUI_UPDATE` on, because
it is measured with the widget's font, and only the runtime knows that.
`0` switches wrapping off. **Alignment:** `GUI_SET_ALIGN` applies per line
within the widget width; without wrapping the width is the estimated
text width, so give `GUI_SET_BOUNDS` first.

## Checkbox

```basic
GUI_CHECKBOX(win, label$, x, y[, default]) -> GUI_WIDGET
```

A click toggles. `GUI_CHECKED(chk)` returns the state, `GUI_SET_CHECKED` sets
it programmatically.

## Slider

```basic
GUI_SLIDER(win, x, y, w, min, max[, default]) -> GUI_WIDGET
```

Horizontal value slider. `GUI_VALUE(s)` returns the FLOAT value,
`GUI_SET_VALUE(s, v)` sets it (clamped to `[min, max]`). `max > min` is
required.

## TextInput

```basic
GUI_TEXTINPUT(win, x, y, w, h[, platzhalter$]) -> GUI_WIDGET
```

Single-line input field. A click focuses it, typed characters are appended,
Backspace deletes. `GUI_TEXT(tf)` returns the content, `GUI_SET_TEXT(tf, ...)`
prefills it. A blinking cursor appears in the focused field.
**Ctrl+Z / Ctrl+Y** undo and redo while typing — keystrokes within 0.8 s are
one step, otherwise Ctrl+Z would undo a character instead of a word. A
`GUI_SET_TEXT` clears the history: what the program sets is not a step of the
user.
```basic
GUI_TEXTINPUT_SET(tf, "passwort", 1)     ' dots instead of characters; GUI_TEXT returns the real text
GUI_TEXTINPUT_SET(tf, "nur_lesen", 1)    ' display, select, copy -- but not change
GUI_TEXTINPUT_SET(tf, "maxlaenge", 8)    ' cuts off, also when pasting (0 = unlimited)
GUI_TEXTINPUT_SET(tf, "zahlen", 2)       ' 1 = integer (with sign), 2 = decimal (point or comma)
GUI_SET_ALIGN(tf, "rechts")              ' numbers sit on the right -- as long as the text fits
```

The number filter rejects, as a whole, an input that would make the field
invalid, but lets intermediate states like `-` through — otherwise a
negative number could not be typed at all. **Enter** reports the field via
`GUI_ENTERED(tf)` (for exactly one frame, like `GUI_CLICKED`) or the callback
`GUI_ON_ENTER(tf, handler)`; if the window has a default button, it clicks
in the same frame (see [Button](#button)).

## Panel

```basic
GUI_PANEL(win, x, y, w, h[, titel$]) -> GUI_WIDGET
```

Purely decorative container (border + optional title row). Not interactive.

## Form widgets: Radio, Dropdown, ProgressBar

### RadioButton (groups, mutual exclusion)

```basic
DIM easy AS GUI_WIDGET
easy = GUI_RADIO(win, "diff", "Einfach", 20, 60)   ' group "diff"
DIM hard AS GUI_WIDGET
hard = GUI_RADIO(win, "diff", "Schwer", 20, 90)    ' same group
GUI_SET_CHECKED(easy, TRUE)                          ' selects easy, deselects the group
```

Radios with the same `group$` in the same window exclude each other.
- `GUI_CHECKED(radio)` → is this one selected?
- `GUI_RADIO_SELECTED(radio)` → index (0-based, creation order) of the
  selected radio of the group, or `-1`. (`radio` may be any one of the group.)
- `GUI_ON_CHANGE(radio, handler)` fires on selection.

### Dropdown / ComboBox

```basic
DIM farben[3] AS STRING
farben[0]="Rot" : farben[1]="Grün" : farben[2]="Blau"
DIM dd AS GUI_WIDGET
dd = GUI_DROPDOWN(win, 20, 130, 160, 24, farben)   ' a click opens the list
```

- `GUI_DROPDOWN_SELECTED(dd)` → index (or `-1`), `GUI_DROPDOWN_TEXT(dd)` → selected text.
- `GUI_DROPDOWN_SET_SELECTED(dd, i)` sets the selection, `GUI_SET_DROPDOWN(dd, items)` replaces the list.
- `GUI_ON_CHANGE(dd, handler)` fires when the selection changes. The opened popup
  is drawn above all other widgets; a click elsewhere closes it.
- **Long lists**: the popup shows at most ten entries at a time, the rest
  scrolls (mouse wheel over the popup, scroll bar on the right, a click into
  the bar scrolls there). When it opens, the selection sits in the middle. If
  there is no room below the box but there is above, the list opens
  **upwards**.
- **Keyboard**: open with Space, Enter or Down arrow. While open, the arrow
  keys, Page Up/Down, Home/End and typing move only a **marker**; Enter or
  Space takes it over, **ESC closes without a change**. So browsing does not
  fire `on_change`, and a mistake can be taken back.
- **Typing jumps** as in the list box: with the dropdown closed it selects
  the next matching entry right away, with it open it marks it.
- **Placeholder**: `GUI_DROPDOWN_PLACEHOLDER(dd, "Bitte wählen")` sits dimmed
  in the box while nothing is selected (`GUI_DROPDOWN_SET_SELECTED(dd, -1)`);
  it is stored in the `.dhform` (`placeholder`).
- **Editable** (combo box): `GUI_DROPDOWN_SET(dd, "bearbeitbar", 1)` turns
  the box into a text input with an arrow. You type as in a text input
  (selecting, Ctrl+Z, pasting); the list **suggests** -- it opens and marks
  the first entry that starts the same way. **The suggestion is only taken
  over with the arrow keys**: Enter takes the entry chosen with Down/Up arrow,
  otherwise what was typed stays (a suggestion that won by itself would
  replace "Rotwein" with "Rot"). A click into the field sets the cursor, only
  the arrow opens the list; Down arrow opens, ESC closes.
  `GUI_DROPDOWN_TEXT` and `GUI_TEXT` return the **text**, `GUI_DROPDOWN_SELECTED`
  the entry that is called exactly that (case does not matter) -- otherwise `-1`.
  `GUI_SET_TEXT` sets the text, `GUI_DROPDOWN_SET_SELECTED` writes the
  entry into the field. Enter from the field submits the form as from a
  text input (`GUI_ENTERED`, default button). In the `.dhform` it is stored as
  `bearbeitbar`.

```basic
DIM stadt AS GUI_WIDGET
stadt = GUI_DROPDOWN(win, 20, 20, 220, 28, ["Berlin", "Bremen", "Hamburg", "München"])
GUI_DROPDOWN_SET(stadt, "bearbeitbar", 1)
' ... after GUI_UPDATE:
IF GUI_ENTERED(stadt) THEN PRINT GUI_DROPDOWN_TEXT(stadt); GUI_DROPDOWN_SELECTED(stadt)
```

### ProgressBar

```basic
DIM bar AS GUI_WIDGET
bar = GUI_PROGRESS(win, 20, 170, 200, 18)
GUI_SET_VALUE(bar, 0.65)        ' 0.0 .. 1.0 (shows "65%")
PRINT GUI_VALUE(bar)
```

Not interactive; progress via `GUI_SET_VALUE` (0..1, clamped).

### Spinner — number field with +/-

```basic
DIM sp AS GUI_WIDGET
sp = GUI_SPINNER(win, 20, 200, 120, 0, 100, 50, 5)   ' min 0, max 100, start 50, step 5
GUI_ON_CHANGE(sp, wert_geaendert)                     ' FUNCREF = the bare name
PRINT GUI_VALUE(sp)                                   ' current value (FLOAT)
```

Input field with two buttons (▲/▼) on the right. The value changes by `step`
(default 1) by clicking the buttons, the **mouse wheel** over the field or
**Up/Down arrow** when the field has focus (click into it). Always clamped to
`[min, max]`. `GUI_VALUE`/`GUI_SET_VALUE` read/set the value, `GUI_ON_CHANGE`
fires on every change. Whole numbers are shown without decimal places.

### Splitter — movable divider

```basic
DIM spl AS GUI_WIDGET
spl = GUI_SPLITTER(win, 300, 70, 360, "v", 140, 440)   ' vertical bar, x 140..440
' ... every frame: read the position and lay out two areas:
DIM sx AS INTEGER : sx = GUI_VALUE(spl)
GUI_SET_BOUNDS(linksPanel,  20,   70, sx - 20,        360)
GUI_SET_BOUNDS(rechtsPanel, sx+6, 70, 580 - (sx+6),   360)
```

A thin grip bar that you drag with the mouse. `orient$` = `"v"`/`"vertical"`
(vertical bar, drags horizontally) or `"h"`/`"horizontal"` (horizontal bar,
drags vertically). With `"v"`, `x` is the start position (clamped to
`[min, max]`) and the bar runs from `y` over `length` pixels; with `"h"`,
`x`/`y` are swapped. The GUI has **no** layout parenting — the divider only
supplies its position (`GUI_VALUE`, window-relative); with it you define the
two areas yourself via `GUI_SET_BOUNDS`. `GUI_ON_CHANGE` fires while dragging.

### Icon buttons & toolbar

```basic
DIM tb AS GUI_WIDGET : tb = GUI_TOOLBAR(win, 0, 0, 600, 46)        ' strip
DIM icSave AS INTEGER : icSave = GENTEX_COLOR(28, 28, &H4BE87A)    ' icon = texture handle
DIM bSave AS GUI_WIDGET : bSave = GUI_ICON_BUTTON(win, 8, 7, 34, 32, icSave)        ' icon only
DIM bRun  AS GUI_WIDGET : bRun  = GUI_ICON_BUTTON(win, 120, 7, 110, 32, icSave, "Start")  ' icon + text
IF GUI_CLICKED(bSave) THEN ...
GUI_SET_ICON(bRun, icOther)                                        ' change the icon
```

`GUI_ICON_BUTTON` is a **perfectly normal button** (`GUI_CLICKED`/`GUI_ON_CLICK`)
with an image. `tex` is a texture handle from `LOADIMAGE` or `GENTEX_*` —
or the **name of a built-in icon** (see below).
**Without** text the button is drawn flat (surface only on hover/click) and
the icon centred — the classic toolbar look; **with** text the icon sits on
the left, the text on the right (normal button look).

### Toolbar with entries

```basic
DIM tb AS GUI_WIDGET : tb = GUI_TOOLBAR(win, 0, 0, 800, 36)
GUI_TOOLBAR_ADD(tb, "neu", "Neue Datei (Strg+N)")         ' 0
GUI_TOOLBAR_ADD(tb, "sichern", "Sichern (Strg+S)")        ' 1
GUI_TOOLBAR_SEPARATOR(tb)                                 ' 2
GUI_TOOLBAR_ADD(tb, "start", "Starten (F5)")              ' 3
GUI_TOOLBAR_ADD(tb, "umbruch", "Zeilenumbruch")           ' 4
GUI_TOOLBAR_CHECKABLE(tb, 4, TRUE)
GUI_TOOLBAR_SPACER(tb)                                    ' 5 -- pushes the rest to the right
GUI_TOOLBAR_ADD(tb, "einstellungen", "Einstellungen")     ' 6

' every frame, after GUI_UPDATE:
SELECT CASE GUI_TOOLBAR_CLICKED(tb)
    CASE 0 : PRINT "neue Datei"
    CASE 3 : PRINT "starten"
    CASE 4 : PRINT "Umbruch "; GUI_TOOLBAR_CHECKED(tb, 4)
END SELECT
```

Up to stage 25, `GUI_TOOLBAR` was only a strip on which a program placed
icon buttons and moved them by hand at every resize — with images that were
painted for one size and stretched for the bar. Now **the bar lays out its
entries itself**: buttons are as wide as the bar is high (minus 6 points),
separators 11, a gap takes the remaining space. The number of an entry is
the order of creation, separators and gaps count.

**Built-in icons** are drawn at the size at which they are needed — as
strokes on a 16 grid, not as a stretched pixel image —, all with the same
stroke width and in the theme's text colour. Start, Stop, Check and
Breakpoint carry their own colour. `GUI_ICON_BUTTON`, `GUI_SET_ICON` and
`GUI_MENU_ICON` take the same names, so that menu and bar look alike:

`neu`, `datei`, `oeffnen`, `sichern`, `drucken`, `export`, `import`,
`rueckgaengig`, `wiederholen`, `ausschneiden`, `kopieren`, `einfuegen`,
`suchen`, `start`, `stopp`, `pause`, `debug`, `haltepunkt`, `pruefen`,
`profil`, `handbuch`, `einstellungen`, `werkzeug`, `umbruch`, `info`,
`warnung`, `plus`, `minus`, `schliessen`, `links`, `rechts`, `hoch`,
`runter`, `menue`, `aktualisieren`.

An unknown name is an **error** that lists the known ones — a silently empty
button would only be noticed when someone looks for it. Whoever wants their
own image passes a texture handle instead of the name.

A button only shows its surface when it has something to say: under the
mouse, pressed or switched on (a touch of accent, so that two switched-on
buttons next to each other still look like buttons). A click counts when
the key goes up again on the **same** button; dragging away takes the press
back. `GUI_TOOLBAR_CLICKED` reports it for exactly one frame,
`GUI_CLICKED(tb)` and `GUI_ON_CLICK` fire as well. Separators, gaps and
disabled buttons accept no click. Every entry has its own tooltip, and a
screen reader sees the buttons as buttons.

**Without entries the bar stays the strip it was**, and accepts no clicks —
buttons that an older program places on it keep working. In the `.dhform`
the entries are stored under `leiste` (images not — texture handles are only
valid in this run).

**If not everything fits**, the rear buttons move into a **» menu** at the
right end: as many as fit next to the » button stay, gaps fall away, and a
separator at the break point disappears. A click on an entry in the menu is
the same click as on the button (`GUI_TOOLBAR_CLICKED`), disabled ones are
shown dimmed, switched-on ones in the accent colour. `GUI_TOOLBAR_OVERFLOW`
tells how many are currently in the menu.

### Status bar and breadcrumb bar

```basic
DIM sb AS GUI_WIDGET : sb = GUI_STATUSBAR(win, 0, 560, 800, 24, "Bereit.")
DIM fPos AS INTEGER : fPos = GUI_STATUSBAR_ADD(sb, "Zeile 1", 140, "mitte")
GUI_STATUSBAR_CLICKABLE(sb, fPos, TRUE)
GUI_STATUSBAR_TIP(sb, fPos, "Gehe zu Zeile")

DIM pf AS GUI_WIDGET : pf = GUI_BREADCRUMB(win, 0, 0, 800, 22)
GUI_BREADCRUMB_ADD(pf, "spiel.dh", "1")
GUI_BREADCRUMB_ADD(pf, "class Held", "12")

' every frame:
GUI_SET_TEXT(sb, "Gesichert.")                     ' field 0
IF GUI_STATUSBAR_CLICKED(sb) = fPos THEN PRINT "Gehe zu Zeile"
IF GUI_BREADCRUMB_CLICKED(pf) >= 0 THEN PRINT GUI_BREADCRUMB_DATA$(pf, GUI_BREADCRUMB_CLICKED(pf))
```

A **status bar** has fields: fixed widths first, the rest goes to the fields
with width 0. Until then a program wrote everything into one label — and the
cursor position overwrote the last message at every arrow key. `GUI_SET_TEXT`
and `GUI_TEXT` mean the first field, so a label used as a status line can be
replaced without rewriting. Clickable fields stand out under the mouse and
report themselves via `GUI_STATUSBAR_CLICKED`; a screen reader hears the
fields as status and the clickable ones as buttons.

A **breadcrumb bar** shows parts with an angle bracket between them; the last
one is "where you are" and is shown in bolder type. Each part carries an
invisible value (a path, a line number). **If not everything fits, the front
parts disappear under a "…"** — the end of a path is what you need; the
tooltip over the "…" names the whole path, and a screen reader gets it in
full anyway. The widths are measured in `GUI_UPDATE`, the click reads the
same number.

Both are stored in the `.dhform` (`status` with fields, `pfad` with parts and
values); in the form designer they are in the palette.

## ListBox, Image, Canvas

### ListBox — scrollable selection list

```basic
DIM obst[3] AS STRING
obst[0]="Apfel" : obst[1]="Birne" : obst[2]="Kirsche"
DIM lb AS GUI_WIDGET
lb = GUI_LISTBOX(win, 20, 40, 160, 100, obst)   ' click selects, mouse wheel scrolls
```

- `GUI_LISTBOX_SELECTED(lb)` → index (or `-1`), `GUI_LISTBOX_TEXT(lb)` → text.
- `GUI_LISTBOX_SET_SELECTED(lb, i)`, `GUI_SET_LISTBOX(lb, items)`.
- `GUI_ON_CHANGE(lb, handler)` fires on selection. (Shares the item logic with
  the dropdown — the `GUI_DROPDOWN_*` getters work on list boxes too.)

**Items one at a time** (since 2026-09-04, also for dropdowns): `GUI_LISTBOX_ADD(lb, text$[, pos])`
returns the index, `GUI_LISTBOX_REMOVE`, `GUI_LISTBOX_CLEAR`, `GUI_LISTBOX_COUNT`,
`GUI_LISTBOX_ITEM`, `GUI_LISTBOX_SET_ITEM`, `GUI_LISTBOX_MOVE(lb, von, nach)`
(take out and insert, not swap). The selection **keeps meaning the same
item**: if something is inserted or deleted before it, it moves along; if the
selected item is deleted, nothing is selected any more. Before, the list could
only be replaced as a whole, and `GUI_SET_LISTBOX` resets the scroll position.

```basic
GUI_LISTBOX_SET(lb, "kaestchen", 1)          ' a check mark per item; a click on the checkbox toggles ONLY the check mark
GUI_LISTBOX_SET(lb, "mehrfachauswahl", 1)    ' Ctrl+click collects, Shift+click spans from the anchor
GUI_LISTBOX_ICON(lb, 0, bild)                ' icon to the left of the text
GUI_LISTBOX_COLOR(lb, 2, RED)                ' text colour of one item
IF GUI_DOUBLE_CLICKED(lb) THEN oeffnen(GUI_LISTBOX_SELECTED(lb))
```

With **checkboxes**, a click on the checkbox only toggles the check mark (the
selection stays, otherwise you would change the selection every time you tick
something off), and the space bar toggles the check mark of the selected row;
both fire `on_change`. **Multi-selection** has the same queries as the table:
`GUI_LISTBOX_IS_SELECTED`, `GUI_LISTBOX_SELECT`, `GUI_LISTBOX_SEL_COUNT`,
`GUI_LISTBOX_SEL_ROW(lb, k)` (the k-th selected one, in list order),
`GUI_LISTBOX_CLEAR_SELECTION`; `GUI_LISTBOX_SELECTED` stays the row clicked
last, and an arrow key sets the set to a single row. Without multi-selection
the queries return the one selected row. `GUI_DOUBLE_CLICKED(lb)` holds for
one frame, like `GUI_CLICKED`. Check marks, colours, selection and the two
switches are stored in the `.dhform` (`list`), icons are not, since they are
texture handles.

### Comfortable lists

```basic
DIM lb AS GUI_WIDGET
lb = GUI_LISTBOX(win, 20, 40, 260, 220, ["Obst", "Apfel", "Birne", "Gemüse", "Mais"])
GUI_LISTBOX_HEADER(lb, 0, TRUE)                ' group headers
GUI_LISTBOX_HEADER(lb, 3, TRUE)
GUI_LISTBOX_SET_DATA(lb, 1, "artikel-17")      ' invisible: the ID of the database row
GUI_LISTBOX_DETAIL(lb, 1, "1,20 €")            ' additional text on the right
GUI_LISTBOX_TIP(lb, 1, "aus der Region")
GUI_LISTBOX_ENABLE(lb, 2, FALSE)               ' sold out: visible, not selectable
GUI_LISTBOX_PLACEHOLDER(lb, "Keine Treffer")
GUI_LISTBOX_SORT(lb)                           ' natural, within the groups

GUI_LISTBOX_FILTER(lb, "ap")                   ' e.g. every frame from a search field
IF GUI_DOUBLE_CLICKED(lb) THEN PRINT GUI_LISTBOX_DATA$(lb, GUI_LISTBOX_SELECTED(lb))
```

What a list needs in everyday use, it can do by itself since stage 27:

* **An invisible value per item** (`SET_DATA`/`DATA$`/`FIND_DATA`). The
  text is for humans and changes; a remembered number is wrong after the
  first insertion. The value travels along when the item is moved, sorted
  in, or something is inserted before it.
* **Additional text on the right** (`DETAIL`), muted and right-aligned; the
  main text ends before it instead of running on underneath it.
* **Disabled items** and **group headers** are visible, but neither
  clickable nor reachable with the arrow keys — an arrow key jumps over
  them. If a selected item is disabled, the selection is dropped.
* **Filter**: only what contains the partial text in its text or additional
  text, ignoring upper/lower case. A group header stays as long as something
  in its group matches. **The items stay where they are**: all numbers
  passed to the outside are still item numbers; the visible order comes from
  `VIEW_COUNT`/`VIEW_ROW` (as with the table). If nothing is visible, the
  hint from `PLACEHOLDER` is shown instead of an empty area.
* **Sorting** is natural — “Datei 9” before “Datei 10”, case does not
  matter — and stays within the groups; otherwise it would tear the headers
  away from their items. Afterwards the selection means the same item.
* **Keyboard**: Page Up/Down page through, **Enter** reports `GUI_DOUBLE_CLICKED`
  (open without the mouse), and **typing jumps**: letters select the next
  item that starts with them; what is typed within one second counts as one
  word (“ki” → “Kirsche”), and the same key pressed several times cycles
  through all items with that start.
* **Searching** (`FIND`) and **scrolling into view** (`SCROLL_TO`) from the program.
* **Scroll bar**: if not all items fit, a bar appears on the right. You drag
  the thumb, a click into the track puts it under the mouse (and you can go
  on dragging right away). The rows end before the bar.
* **Smooth scrolling**: the mouse wheel scrolls over the duration of the
  transitions (`uebergang`, see [Transitions](#transitions)) instead of jumping;
  several wheel steps in a row add up. If something else scrolls (arrow key,
  `SCROLL_TO`), the wheel gives way.
* **Right click selects**: a context menu refers to the row you point at.
  In a multi-selection the selection stays if the row belongs to it — so
  “Delete” acts on all selected ones.
* **Renaming** with `GUI_LISTBOX_SET(lb, "bearbeitbar", 1)`: **F2** opens an
  input field over the selected item, with the text selected. Enter or a
  click elsewhere accepts, ESC reverts; Tab or a change of focus accepts as
  well. Selecting text and Ctrl+A/C/V/X work as in the text input.
  `GUI_LISTBOX_EDIT(lb, i)` starts it from the program (for instance from a
  context menu “Rename” — that works without the switch too),
  `GUI_LISTBOX_EDITED(lb)` reports for one frame which item has a new name
  (an unchanged text does not count), and `on_change` fires.
  **ESC trap** as with the table: if you query ESC yourself (for instance to
  quit), check `GUI_LISTBOX_EDITING(lb) < 0` first.

A screen reader sees only the visible items, disabled ones as disabled, and
the additional text as part of the name. Values, additional texts, tooltips,
headers, disabled items and the hint are stored in the `.dhform`; the filter
is not — it is a view, not content.

### Image — picture/icon in the UI

```basic
DIM logo AS INTEGER
logo = LOADIMAGE("assets/logo.png")
DIM iw AS GUI_WIDGET
iw = GUI_IMAGE(win, 20, 20, 96, 96, logo)   ' scaled into the rectangle
GUI_SET_IMAGE(iw, anderesBild)              ' change the image
```

Shows a texture loaded via `LOADIMAGE`, scaled to the widget rectangle.

### Canvas — free drawing area (“mini screen” inside the window)

A canvas reserves an area in which you paint with the **normal drawing
commands** (`PLOT`/`LINE`/`BOX`/`CIRCLE`/`DRAWIMAGE`/3D …) — ideal for an
embedded game, a chart or a preview area.

```basic
DIM cv AS GUI_WIDGET
cv = GUI_CANVAS(win, 10, 30, 280, 180)

' --- per frame ---
GUI_UPDATE()
GUI_DRAW()                       ' draws the window + the canvas frame
' Then paint into the canvas area (absolute screen coordinates):
DIM cx AS INTEGER : DIM cy AS INTEGER : DIM cw AS INTEGER : DIM ch AS INTEGER
cx = GUI_CANVAS_X(cv) : cy = GUI_CANVAS_Y(cv)
cw = GUI_CANVAS_W(cv) : ch = GUI_CANVAS_H(cv)
BOX(cx, cy, cx + cw, cy + ch, &H101820)
CIRCLE(cx + cw/2, cy + ch/2, 30, &H30FFA0)
FLIP()
```

`GUI_CANVAS_X/Y/W/H` return the **absolute** content area (it follows the
window when it is moved). You draw **after** `GUI_DRAW` and clip to the area
yourself.

### A window above what was drawn: `GUI_DRAW_WINDOW`

This leads to a pitfall you only see once you have it: **a window that lies
above a canvas disappears behind its content.** `GUI_DRAW` draws windows and
canvases in one pass, and the content of a canvas is by design created
afterwards. A dialog that opens in the middle of the area is therefore
invisible — it blocks input and cannot be seen; the program seems frozen.

A second `GUI_DRAW` does not help: it also redraws the window backgrounds and
the canvases, that is, exactly what was just painted. Instead, draw that one
window once more at the end of the frame:

```basic
GUI_UPDATE()
GUI_DRAW()
' ... here the program paints into its canvases ...
IF dialogOffen THEN GUI_DRAW_WINDOW(dlg)    ' on top, last
FLIP()
```

An invisible or destroyed window draws nothing (no error) — so the call may
stand unconditionally in the frame loop. If the window is the modal one, its
overlay comes along.

The window also comes to the front in the **order**, without taking the
focus: what is visible on top also gets the clicks in the next frame.
Before, after a click into the window underneath it lay behind again -- you
saw a button, and the click went to the window you could not see.

**Context menu and tooltip** lie above *all* windows and are therefore
affected by the same problem — a tooltip follows the mouse and so regularly
ends up above a canvas. For them there is `GUI_DRAW_TOP()`, and `GUI_DRAW`
leaves them out with `GUI_DRAW(FALSE)`:

```basic
GUI_UPDATE()
GUI_DRAW(FALSE)                             ' windows, without the on-top layer
' ... here the program paints into its canvases ...
IF dialogOffen THEN GUI_DRAW_WINDOW(dlg)
GUI_DRAW_TOP()                              ' context menu + tooltip, last
FLIP()
```

Why not simply draw twice: tooltip and context menu have a semi-transparent
drop shadow. Drawn twice on top of each other it is darker — and **only where
your own drawing did not cover the first version**, that is, exactly at an
edge. With windows (`GUI_DRAW_WINDOW`) this remains a known leftover: if a
window lies partly outside the area you drew yourself, its gloss is applied
twice there. A dialog in the middle of the area is not affected.

(For real window overlap/occlusion, use a render target.)

### When `GUI_CLICKED` reports for a switch

A button sets its flag on **release**, a checkbox (and toggle, radio)
toggles already on **press** — for the caller both mean “clicked in this
frame”, and in both cases it is TRUE for exactly one frame. The state itself
still comes from `GUI_CHECKED`:

```basic
IF GUI_CLICKED(cbSichtbar) THEN
    ebene_sichtbar = GUI_CHECKED(cbSichtbar)
END IF
```

Until 2026-09-01, `GUI_CLICKED` stayed **silent** on a switch — always
FALSE, without an error. Two editor pilots (tilemap and sprite) had the same
dead switch because of it: the check mark toggled, the layer stayed visible.
If you test with an older runtime, check every frame instead whether
`GUI_CHECKED` has changed compared with your own state.

## Table

```basic
GUI_TABLE(win, x, y, w, h[, headers, cells]) -> GUI_WIDGET
```

Persistent table with a **fixed header row** and a **scrollable body**
(vertical + horizontal). Complement to the immediate-mode `UI_TABLE`: the data
lives in the widget, set once during setup (or at any time via a setter)
instead of being passed anew every frame.

Pass the data right when creating it (both arrays or neither) — or create it
empty and set it later:

```basic
DIM headers AS ARRAY OF STRING
headers = SPLIT$("Name|HP|Level", "|")

DIM cells[3, 3] AS STRING
' ... fill cells ...

DIM tbl AS GUI_WIDGET
tbl = GUI_TABLE(win, 10, 40, 280, 160, headers, cells)
' alternatively:
'   tbl = GUI_TABLE(win, 10, 40, 280, 160)
'   GUI_TABLE_HEADERS(tbl, headers)
'   GUI_TABLE_ROWS(tbl, cells)
```

**Setters** (at any time, e.g. on data updates):

| Built-in | Effect |
|---|---|
| `GUI_TABLE_HEADERS(tbl, headers)` | column titles (1D `ARRAY OF STRING`) |
| `GUI_TABLE_ROWS(tbl, cells)` | data rows (2D `ARRAY OF STRING`); rows may be shorter or longer than the header |
| `GUI_TABLE_COL_WIDTHS(tbl, widths)` | pixel width per column (1D `ARRAY OF INTEGER`); `NIL` = distribute evenly |

**Operation & polling:**

| Built-in | Effect |
|---|---|
| `GUI_TABLE_SELECTED(tbl)` | persistently selected row (-1 if none) |
| `GUI_TABLE_SET_SELECTED(tbl, row)` | set the selection from the program (-1 = none; out of range → -1) |
| `GUI_TABLE_CLICKED(tbl)` | the clicked row only in the frame of the click, otherwise -1 |
| `GUI_TABLE_ROW_COUNT(tbl)` | number of data rows |

**Behaviour:**
- The header row is fixed; the body scrolls depending on its content.
- The **mouse wheel** scrolls vertically (over the body), **dragging the scroll bar** works on both axes.
- A click on a row **selects** it persistently (a second highlight under the hover).
- If the data shrinks below the selected index, the selection falls back to -1.

### The number of columns is free

It results from the **widest specification**: rows shorter than the header are
simply empty, longer ones extend the table (the header then gets empty titles).
There is no order you have to keep — setting the rows before the header is
just as fine as the other way round.

### Cells: more than text

Every cell carries text, its own **foreground and background colour**, an
**alignment** and a **kind**:

| Kind | What is drawn |
|---|---|
| `text` | the text (default) |
| `bild` | an IMAGE, fitted into the cell (aspect ratio is kept) |
| `haken` | check box — **a click toggles it**, the table does that by itself |
| `balken` | progress bar (`wert` 0..1), the text sits centred on it |
| `knopf` | button; which column was hit is told by `GUI_TABLE_CLICKED_COL` |

| Built-in | Effect |
|---|---|
| `GUI_TABLE_SET_CELL(tbl, zeile, spalte, text$)` | cell content |
| `GUI_TABLE_GET_CELL(tbl, zeile, spalte)` | read cell content |
| `GUI_TABLE_CELL_COLOR(tbl, z, s, vordergrund, hintergrund)` | colours; `-1` = not set |
| `GUI_TABLE_CELL_ALIGN(tbl, z, s, "links"/"mitte"/"rechts")` | alignment of this cell |
| `GUI_TABLE_CELL_KIND(tbl, z, s, art$)` | kind (see the table above) |
| `GUI_TABLE_CELL_IMAGE(tbl, z, s, bild)` | set an image (sets the kind along with it) |
| `GUI_TABLE_CELL_VALUE(tbl, z, s, wert)` / `GUI_TABLE_GET_VALUE(...)` | check mark (0/1) or bar (0..1) |
| `GUI_TABLE_ROW_COLOR(tbl, zeile, vg, hg)` | colour a whole row — **one** call instead of one per column |
| `GUI_TABLE_COL_ALIGN(tbl, spalte, wie$)` | alignment of the whole column |

**Colours in three levels:** cell → row → zebra. `-1` means “not set” and
passes on to the next level — so you colour a whole row with one call and can
still let individual cells stand out. Selection and hover lie
**semi-transparently on top**: a cell colour of your own does not cover them.

### Maintaining rows one at a time

| Built-in | Effect |
|---|---|
| `GUI_TABLE_ADD_ROW(tbl, zellen)` | append a row → new row index |
| `GUI_TABLE_REMOVE_ROW(tbl, zeile)` | remove a row (the selection moves along) |
| `GUI_TABLE_CLEAR(tbl)` | all rows gone |

### Sorting

A **click on the header cell** sorts, a second one reverses the direction; a
small arrow shows what the table is currently sorted by. Sorting is
**numeric** if both cells can be read as numbers, otherwise textual — without
that, `100` would come before `9`.

| Built-in | Effect |
|---|---|
| `GUI_TABLE_SORT(tbl, spalte, absteigend)` | from the program; column `< 0` removes the sorting |
| `GUI_TABLE_SORT_COL(tbl)` / `GUI_TABLE_SORT_DESC(tbl)` | read the current sorting |

### Filtering

`GUI_TABLE_SET(tbl, "filterzeile", 1)` shows a row below the header with a
small input field per column. Clicking into it and typing filters the column
by **partial text**, ignoring upper/lower case; `ESC` or `Enter` ends the
input. Several columns work together (AND).

| Built-in | Effect |
|---|---|
| `GUI_TABLE_FILTER(tbl, spalte, text$)` | set a filter (empty text = no filter) |
| `GUI_TABLE_GET_FILTER(tbl, spalte)` | read the filter that is set |

### Operation as with the list box

What makes a list box comfortable, the table can do just as well in **row mode**:

* **Double click or Enter** on a row reports `GUI_DOUBLE_CLICKED(tbl)` for
  one frame — “open” without your own double-click detection. A double click
  on an editable text cell still edits it instead of reporting.
* **F2** edits the first enabled text column of the selected row
  (`GUI_TABLE_COL_EDIT`, in display order). In cell mode F2 edits the current
  cell as before.
* **Typing jumps** to the next row whose text in the sort column (without
  sorting: in the first visible column) starts that way; what is typed within
  one second is one word, the same key pressed several times cycles through
  all matches.
* **Right click selects** the row under the mouse, so that a context menu
  refers to it; a multi-selection stays if the row belongs to it.
* The **mouse wheel** scrolls smoothly (see [Transitions](#transitions)), the
  **row under the mouse** fades in.
* **Empty hint**: if no row is shown (no data or no match for the filter),
  the table shows the text from `GUI_TABLE_PLACEHOLDER`.

| Built-in | Effect |
|---|---|
| `GUI_TABLE_PLACEHOLDER(tbl, text$)` | hint when no row is visible (“No matches”); stored in the `.dhform` |

### Fixed columns

`GUI_TABLE_SET(tbl, "feste_spalten", 2)` keeps the first two columns in place
during horizontal scrolling — the identifier stays visible while you move
right through the data. **As soon as the table is actually scrolled
sideways**, a somewhat stronger dividing line marks the end of the fixed
block; while the table stands still it stays away, because there it would
explain nothing. Deliberately not in the accent colour: everywhere else that
means “selected/active”.

Everything that locates columns goes through **one** source (`col_x` for the
position, `col_clip` for the visible area) — hit testing *and* drawing use
it. A fixed block would otherwise be the surest way to let the two drift
apart: you click on “Name” and hit the column that has scrolled through
underneath it.

Specifying more fixed columns than exist is capped.

### Reordering columns

Off (an accidental drag on the header would otherwise jumble the arrangement,
and getting back is only possible by hand). With
`GUI_TABLE_SET(tbl, "spalten_verschiebbar", 1)` a **header cell can be dragged
sideways**; the column swaps places live as soon as it passes the middle of
its neighbour.

**Click and drag are the same gesture.** Only on release is it clear what was
meant: without movement the table sorts, from 5 px of movement on the column
was moved. Without this distinction every move would also sort as a side
effect.

Here too: **the data is not rearranged.** `col_order` only maps display
position → data column. `GUI_TABLE_GET_CELL(tbl, r, 0)` still returns the same
column after moving — a column number your program has remembered stays
valid.

| Built-in | Effect |
|---|---|
| `GUI_TABLE_MOVE_COL(tbl, von_pos, nach_pos)` | move a column (display positions) |
| `GUI_TABLE_COL_AT(tbl, pos)` | which **data column** is at this position? |
| `GUI_TABLE_COL_POS(tbl, spalte)` | at which position is this data column? |
| `GUI_TABLE_RESET_COLS(tbl)` | original order |

Fixed columns count by **position**: if you move a column to the front, it
becomes fixed too. The order is saved in the `.dhform` as well.

### Multi-selection

Off (then the table behaves as before: one row). With
`GUI_TABLE_SET(tbl, "mehrfachauswahl", 1)`:

- **Ctrl+click** adds a row or removes it
- **Shift+click** selects the range from the row clicked last — in the
  **visible** order, because that is what you see between the two clicks
  (going over the data rows would hit something entirely different in a
  sorted table)
- a normal click replaces the selection

| Built-in | Effect |
|---|---|
| `GUI_TABLE_SEL_COUNT(tbl)` | number of selected rows |
| `GUI_TABLE_SEL_ROW(tbl, i)` | i-th selected **data row**, in click order |
| `GUI_TABLE_IS_SELECTED(tbl, zeile)` | is this data row selected? |
| `GUI_TABLE_SELECT(tbl, zeile, an)` | add / remove a row |
| `GUI_TABLE_CLEAR_SELECTION(tbl)` | clear the selection |

`GUI_TABLE_SELECTED` still returns the row **clicked last** — existing code
that only knows that one runs unchanged. When switching it on, an existing
single selection is taken over; when a row is deleted, all indices behind it
move up (otherwise the selection would afterwards point to other rows).

### Editing cells directly

A **double click** on a text cell of an enabled column opens an input field
exactly in that cell. `Enter` accepts, `ESC` reverts, a click elsewhere
accepts as well (as everywhere else — reverting is done with `ESC`).

```basic
GUI_TABLE_COL_EDIT(tbl, 1, TRUE)     ' enable column 1
```

Without enabling, nothing happens on a double click: a table you can
accidentally type into anywhere would be worse than one without editing. Only
`text` cells are affected — a check mark toggles by click anyway, a bar or an
image has no text to type.

| Built-in | Effect |
|---|---|
| `GUI_TABLE_COL_EDIT(tbl, spalte, an)` | enable a column for editing |
| `GUI_TABLE_EDITING_ROW(tbl)` / `GUI_TABLE_EDITING_COL(tbl)` | which cell is being edited right now (`-1` = none) |

On accepting, `on_change` fires and the view (sorting + filter) is rebuilt —
**only then**, not on every key: otherwise the row would jump away from under
your finger while typing.

**The editor can do the same as a `TextInput`** — both use the same routine,
so the logic exists only once: cursor, arrow keys, `Pos1`/`Ende`,
`Rücktaste`, `Entf`, `Strg+A/C/V/X`, **selecting text** (with Shift+navigation
or by dragging with the mouse in the field), and it scrolls the text along
when it is longer than the cell.

On opening, the **whole content is selected** — so the first keystroke
replaces it instead of extending it. That is exactly what you expect when you
open a cell to overwrite it. (That is why the opening double click itself does
*not* set the mark; the editor only listens to the mouse after the button has
been released. Otherwise the same click would have cancelled the selection
again right away.)

> **Careful with `ESC`:** if your program uses `ESC` to quit, check
> `GUI_TABLE_EDITING_ROW(tbl) < 0` first — otherwise the same key you use to
> revert an input quits the program (that is how the demo does it).

### Grid: cell mode

A table shows rows; a **grid** edits cells. Both are the same widget.
`GUI_GRID` creates a table that starts out as a grid (cell mode, every column
editable, empty rows), and `GUI_TABLE_SET(tbl, "zellmodus", 1)` turns any
existing table into one — sorting, filter, fixed columns and cell kinds stay
as they are.

```basic
DIM kopf[3] AS STRING
kopf[0] = "Artikel" : kopf[1] = "Menge" : kopf[2] = "Farbe"
DIM gt AS GUI_WIDGET : gt = GUI_GRID(win, 10, 10, 400, 240, kopf, 5)
GUI_TABLE_COL_TYPE(gt, 1, "ganz")              ' integers only
GUI_TABLE_COL_CHOICES(gt, 2, farben)           ' choice list
GUI_TABLE_SET(gt, "zeilen_anhaengen", 1)       ' Enter below the last row creates one
```

Operation as in a spreadsheet:

| Key | Effect |
|---|---|
| Arrow keys, `Bild`, `Pos1`/`Ende` | move the current cell (`Strg+Pos1/Ende` = first/last cell) |
| Shift + movement, mouse drag, Shift+click | span a range |
| `Tab` / `Umschalt+Tab` | one cell forward / back, at the end of a row into the next row; **`Strg+Tab`** leaves the grid |
| Typing | edits the cell and **replaces** the content |
| `Enter` / `F2` | edit the cell; while editing, `Enter` accepts and moves one row down |
| `Entf` / `Rücktaste` | clear the range (editable cells only) |
| `Leertaste` | toggle a check-mark cell |
| `Strg+A` / `Strg+C` / `Strg+X` / `Strg+V` | select all / copy the range / cut / paste from the cell on |

What is copied is **tab-separated text** (cells separated by tabs, rows by
line breaks) — exactly what every spreadsheet understands on pasting and
delivers on copying. On pasting, the column decides: a locked column or a
value that does not fit (letters in a number column, an entry that is not in
the choice list) is **skipped**, not half written; `GUI_TABLE_PASTE` says how
many cells it really came to.

**The range is a rectangle in the visible order** — after sorting, that is,
what you see, not what would lie side by side in the data rows. To the
outside, all values remain data rows and data columns, as everywhere with the
table.

| Built-in | Returns | Effect |
|---|---|---|
| `GUI_GRID(win, x, y, w, h, headers, zeilen = 0)` | GUI_WIDGET | table in cell mode, all columns editable |
| `GUI_TABLE_CURRENT_ROW(tbl)` / `GUI_TABLE_CURRENT_COL(tbl)` | INTEGER | current cell (-1 = none) |
| `GUI_TABLE_SET_CURRENT(tbl, zeile, spalte)` | — | set the current cell and scroll it into view (`-1, -1` = none) |
| `GUI_TABLE_RANGE(tbl)` | TUPLE | `(anker_zeile, anker_spalte, zeile, spalte)` of the range |
| `GUI_TABLE_SELECT_RANGE(tbl, anker_zeile, anker_spalte, zeile, spalte)` | — | set the range |
| `GUI_TABLE_COPY$(tbl)` | STRING | the range as tab-separated text |
| `GUI_TABLE_PASTE(tbl, text$)` | INTEGER | paste tab-separated text from the current cell on; number of cells written |
| `GUI_TABLE_COL_TYPE(tbl, spalte, art$)` | — | `text`, `ganz`, `zahl` or `auswahl` |
| `GUI_TABLE_COL_CHOICES(tbl, spalte, eintraege)` | — | entries of a choice column (makes it a choice column) |

A **number column** accepts while typing only what can become a number (`-`
as an intermediate state, with `zahl` a comma or a point); only a real number
is accepted. A **choice column** opens its list below the cell instead of the
input field: arrow keys select, a letter jumps, `Enter` or a click accepts.
The list lies in the top layer — above a canvas it needs `GUI_DRAW_TOP()`,
like a tooltip.

**Even without cell mode** the arrow keys, `Bild` and `Pos1`/`Ende` now move
the selected row as soon as the table has the focus. Until stage 30 a table
with focus accepted no key at all.

### Row numbers: data or view?

**All row values passed to the outside are DATA rows.** Sorting and filtering
do not rearrange the data, they only build a view on top of it — a row number
your program has remembered therefore keeps pointing to the same entry. (If
the data were sorted instead, every remembered number would point to
something else after the first header click.)

To go through the table in the **visible** order:

```basic
DIM i AS INTEGER
FOR i = 0 TO GUI_TABLE_VIEW_COUNT(tbl) - 1
    DIM r AS INTEGER : r = GUI_TABLE_VIEW_ROW(tbl, i)   ' -> data row
    PRINT GUI_TABLE_GET_CELL(tbl, r, 0)
NEXT
```

### Setting appearance and behaviour

One setter with a keyword instead of one built-in per switch (as with
`chart`); an unknown key lists the valid ones.

```basic
GUI_TABLE_SET(tbl, "zeilenhoehe", 38)    ' images need room
GUI_TABLE_SET(tbl, "kopfhoehe", 26)
GUI_TABLE_SET(tbl, "zebra", 1)
GUI_TABLE_SET(tbl, "gitter", 0)
GUI_TABLE_SET(tbl, "filterzeile", 1)
GUI_TABLE_SET(tbl, "sortierbar", 0)         ' header click no longer sorts
GUI_TABLE_SET(tbl, "spalten_ziehbar", 0)    ' column edge no longer draggable
PRINT GUI_TABLE_GET(tbl, "spalten")         ' number of columns
```

**Column widths with the mouse:** the edge between two header cells can be
dragged (catch range ±4 px). On the first drag the table takes over the
widths computed until then as its own values — otherwise it would jump back
on release.

### Saving

`.dhform` stays readable: a plain text cell is still written as a **string**,
only one with colour/kind/image as an object. Both forms are read, older
files run unchanged.

> **To look at:** [examples/157_gui_tabelle.dh](../../examples/157_gui_tabelle.dh)
> — server list with traffic-light images, load bars, favourite check marks
> and an action button; sortable, filterable, columns draggable.

### Attaching to a database

The pattern is always the same: **the database is the truth, the table its
view.** Every row remembers its key; because sorting and filtering do not
rearrange the data rows, this mapping stays valid.

```basic
IMPORT "db"
DIM r AS DB_RESULT
r = DB_QUERY(db, "SELECT id, name, punkte FROM spieler ORDER BY id")
WHILE DB_NEXT(r)
    DIM z AS ARRAY OF STRING
    z = SPLIT$(DB_GET_STRING(r, 1) + "|" + STR$(DB_GET_INT(r, 2)), "|")
    GUI_TABLE_ADD_ROW(tbl, z)
    zeilenId[n] = DB_GET_INT(r, 0)      ' data row -> database id
    n = n + 1
WEND
```

When editing, you write back as soon as the editing *ends* — you remember the
old value when it opens:

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

When deleting several rows: **first collect all keys, then delete** — while
deleting, the row numbers of the table shift.

> **To look at:** [examples/158_gui_tabelle_sqlite.dh](../../examples/158_gui_tabelle_sqlite.dh)
> — the solved levels from `pyramid_pusher.db` in the table: sort, filter,
> rename (`UPDATE`), delete (`DELETE` in a transaction).
> The demo works on a **copy** (`VACUUM INTO`); the original file is only
> read. A demo that changes the user's saved game would be a bad demo.

Row groups are in the **tree table** (`GUI_TREETABLE`, further below) -- it is the same table with the switch `baum`.
- An optional `GUI_ON_CHANGE(tbl, funcref)` callback fires when the selection changes.
- Colours follow the theme (`GUI_SET_COLOR(tbl, ...)` overrides per widget: bg/fg/border/accent).

```basic
GUI_UPDATE()
GUI_DRAW()
IF GUI_TABLE_CLICKED(tbl) >= 0 THEN
    PRINT "Zeile " + STR$(GUI_TABLE_SELECTED(tbl)) + " gewaehlt"
END IF
```

**Example:** [examples/81_table_select.dh](../../examples/81_table_select.dh) shows
both tables (retained `gui` + immediate `ui`) side by side.

## Tree (tree view)

```basic
DIM tree AS GUI_WIDGET : tree = GUI_TREE(win, 20, 20, 300, 380)
DIM proj AS INTEGER : proj = GUI_TREE_ADD(tree, -1, "Mein Spiel")   ' root
DIM src  AS INTEGER : src  = GUI_TREE_ADD(tree, proj, "src")        ' child of proj
GUI_TREE_ADD(tree, src, "main.dh")
GUI_TREE_EXPAND(tree, proj, TRUE)                                   ' start expanded

' every frame:
DIM sel AS INTEGER : sel = GUI_TREE_SELECTED(tree)
IF sel >= 0 THEN PRINT GUI_TREE_LABEL(tree, sel)
```

`GUI_TREE_ADD(tree, parent, label$)` appends a node and returns its **id** (an
integer, stable until `GUI_TREE_CLEAR`). `parent` is `-1` for a root node,
otherwise the id of an existing node — this is how the hierarchy is built. A
click on the triangle on the left expands/collapses a node (or via
`GUI_TREE_EXPAND`), a click on the row selects it. `GUI_TREE_SELECTED` returns
the id of the selected node (`-1` = none), `GUI_TREE_LABEL` its text.
`GUI_ON_CHANGE(tree, …)` fires when the selection changes. The mouse wheel
scrolls long trees. Example: [examples/137_gui_tree.dh](../../examples/137_gui_tree.dh).

## The file tree

Showing a folder was manual work until stage 24: fetch all files, sort them,
build the folder nodes from them, and then drag along a bookkeeping that maps
node numbers to file names. `GUI_FILETREE` does that by itself.

```basic
DIM ft AS GUI_WIDGET : ft = GUI_FILETREE(win, 10, 10, 260, 500, "C:/Projekt")
GUI_FILETREE_FILTER(ft, "*.dh;*.json")        ' empty = all files
GUI_FILETREE_SKIP(ft, "target;__pycache__")   ' folders like files
GUI_FILETREE_SET(ft, "klick_klappt", 1)       ' a click on a folder opens it
GUI_FILETREE_SET(ft, "auffrischen", 2000)     ' check every two seconds

' every frame:
IF GUI_FILETREE_ACTIVATED$(ft) <> "" THEN oeffne(GUI_FILETREE_ACTIVATED$(ft))
```

**Only what is visible is read** — the root and every expanded folder. A
project with a `target` folder in it therefore costs nothing as long as nobody
looks inside; and the fact that every folder gets its triangle, even an empty
one, follows from this: you only know what is inside once you look, and
looking inside just in case is exactly what the tree avoids.

**To the outside this widget speaks in PATHS, not in node numbers.** The node
list is rebuilt on every expansion, so a number would only be valid until the
next movement. The selection comes back via its path; what is collapsed drops
out of the selection (as in every file manager). A **check mark**, on the
other hand, stays: it was a decision, and `GUI_FILETREE_CHECKED_COUNT` also
counts the paths that currently lie behind a collapsed folder.

A path may come from outside as a full path or relative to the root; what
comes out is always the full one, in the notation of the system. `verborgene`
is off, so everything whose name starts with `.` is missing.

**`auffrischen` is the difference between a tree that is right and one you
have to nudge by hand.** A file that another program creates otherwise only
shows up when the program happens to call `GUI_FILETREE_REFRESH`. The
interval counts in milliseconds; 0 means “never by itself”.

## Check marks in the tree

`GUI_TREE_SET(tree, "kaestchen", 1)` puts a checkbox in front of every row —
the same operation as with the list box: **a click on the checkbox only
toggles the check mark**, the selection stays, otherwise you would change the
selection every time you tick something off. With the keyboard the **space
bar** belongs to the check mark and Enter to expanding and collapsing; one key
for both would toggle the check mark along the way with every movement.

A check mark on a folder applies **only to it** — it does not colour its
children along with it. If you need “everything below”, walk through the
children yourself: whether a collapsed folder takes its unseen files along is
a question the program must answer, not the runtime.

## Operating the tree like a list box

What makes the list box comfortable, the tree can do just as well:

* A **double click** on a node reports `GUI_DOUBLE_CLICKED(tree)` for one
  frame (a branch additionally toggles open or closed), and so does **Enter**
  on a leaf; Enter on a branch toggles it as before.
* **F2** renames the selected node if `GUI_TREE_SET(tree,
  "bearbeitbar", 1)` is set — the same operation as with the list box (Enter
  or a click elsewhere accepts, ESC reverts, `GUI_TREE_EDITED` reports it,
  `GUI_TREE_EDIT` starts it from the program). Not in the **file tree**: its
  name is the name of a file, the program has to rename it itself (`RENAME`),
  and the tree shows it afterwards.
* **Typing jumps** to the next visible node that starts that way;
  **Home/End** and **Page Up/Down** jump as in the list box.
* **Right click selects** the node under the mouse; a multi-selection stays
  if the node belongs to it.
* A **scroll bar** on the right as soon as not everything fits (drag the
  thumb, click into the track), and the **mouse wheel** scrolls smoothly.
* **Empty hint** via `GUI_TREE_PLACEHOLDER`.

`bearbeitbar` and the hint are stored in the `.dhform`.

## Tabs inside a window

Tabs existed only **on the window** (`GUI_TABS`). A card box in a corner —
settings with three cards, a tool column with two views — could not be built
with that.

```basic
DIM tc AS GUI_WIDGET : tc = GUI_TABCONTROL(win, 20, 20, 300, 220)
GUI_TABCONTROL_ADD(tc, "Allgemein")
GUI_TABCONTROL_ADD(tc, "Farben")
DIM feld AS GUI_WIDGET : feld = GUI_TEXTINPUT(win, 40, 70, 200, 26)
GUI_TABCONTROL_ADD_WIDGET(tc, feld, 0)        ' only shows on page 0
```

The children **keep their position in the window** — a page only shows or
hides them, exactly as the window tabs do. Nothing is moved; a layout
container on a page goes on computing with the same coordinates. A container
takes its children along onto the page.

The **width of the tab headers** is estimated from the number of characters,
not measured: the hit test runs at a place where there are no graphics, and a
second, more precise calculation during drawing would be the surest way to
let click and label drift apart. With very long labels the text therefore
sits somewhat tighter in its header than it should.

## Rich text

The `gui` could always DISPLAY text -- a label wraps, a text area scrolls.
What was missing was text with SHAPE: headings, bold passages, lists, code
blocks, tables, links. Anyone who wanted to show a manual painted it onto a
canvas themselves (the IDE did that with 180 lines).

```basic
DIM rt AS GUI_WIDGET : rt = GUI_RICHTEXT(win, 10, 10, 600, 400, READALL$("docs/ide.md"))
GUI_RICHTEXT_SET(rt, "codeschrift", monoFont)

' every frame:
IF GUI_RICHTEXT_LINK$(rt) <> "" THEN oeffne(GUI_RICHTEXT_LINK$(rt))
```

It understands a subset of Markdown: `#`, `##`, `###`, paragraphs,
`- ` and `1. ` (also nested, two spaces per level),
` ``` ` blocks, `| Tabellen |` (tables), `> Zitate` (quotes), `---`, plus `**fett**` (bold),
`*kursiv*` (italic), `***beides***` (both), `~~durchgestrichen~~` (strikethrough), `<u>unterstrichen</u>` (underlined),
`` `code` `` and links in square brackets with the target in
round brackets behind them. **What it does not know is shown as
text** -- a document must not lose a line it does not understand.

**Consecutive lines are ONE paragraph**, as in Markdown. That is not a
formality: documents are often wrapped by hand at 76 columns, and set line by
line they would result in a ragged block that looks equally bad at every
window width.

**Typesetting happens in `GUI_UPDATE`, not while drawing.** Only there do the
graphics (for measuring) and write access come together; that way a document
with two thousand lines pays for its typesetting **once** instead of every
frame. It is set anew when the source, width, font size or scale changes --
so resizing the window rewraps, and a frame without a change costs nothing.

**Bold and italic are real font faces where they exist.** If the font is
loaded via `LOADFONT` and its bold or italic face lies next to it (Segoe UI,
Arial, Consolas, Noto, DejaVu ...), the text is set with it. Otherwise the
runtime draws bold as a second stroke and italic slanted -- emphasis you
cannot see would be worse than emphasis that looks different than expected.
A link is underlined like `<u>`.

A **table column** is never squeezed below its widest WORD: a word does not
wrap, otherwise it would run into the neighbouring column and stick to its
text. If the table still does not fit, the columns overflow -- cutting them
off would hide the last one, and that one often carries the explanation.

**Selecting and searching.** Dragging with the mouse selects, a double click
takes the word, Ctrl+A everything, Ctrl+C copies. A position here is (line,
character in the **line text**) -- the words of a set line, with a space
where there is room between them. That is why the search also finds
`Zwei Hunde` across the word boundary, and copied text is readable text
instead of words glued together; the blank lines that typesetting inserts
below a heading do not come along. `FIND_NEXT` searches from the START of the
selection plus one -- that way you get from one selected match to the next,
and without a selection to the first.

```basic
anzahl = GUI_RICHTEXT_MARK_ALL(rt, suche$)     ' highlight all faintly
IF GUI_RICHTEXT_FIND_NEXT(rt, suche$) < 0 THEN PRINT "nicht gefunden"
PRINT GUI_RICHTEXT_SELECTION$(rt)              ' the selected match
```

The position under the mouse is only measured in the next `GUI_UPDATE`: the
press arrives at a place where there are no graphics for measuring, and an
estimated position would be visibly off with a proportional font. A new text
(or new typesetting after a resize) cancels the selection -- its positions
would point into the old typesetting.

**Not included:** images, lists with their own numbering, nested tables,
HTML, Shift+click to extend a selection.

## Time of day

The counterpart to the date picker:

```basic
DIM tp AS GUI_WIDGET : tp = GUI_TIMEPICKER(win, 20, 20)
GUI_SET_TIME(tp, "07:30")
GUI_TIMEPICKER_SET(tp, "schritt", 5)      ' the minute in steps of five
PRINT GUI_TIME$(tp)                        ' "07:30:00"
```

**To the outside ONE format applies, `HH:MM:SS` like `TIME$()`** -- even if
the seconds field is not shown at all; `HH:MM` may go in. Two formats would be
the same trap that was avoided with the date. A malformed value is an
**error**, not a silent 00:00: otherwise you would only notice it from what
the program makes of it.

At the limit the value **wraps around** instead of stopping -- whoever turns
backwards from 00 wants to see 23. The upper half of a field counts up, the
lower half down (the same gesture as on the number field); with the keyboard,
Left/Right select the field and Up/Down the value.

## What does not exist (yet)

So that you do not go looking for them — these controls are missing from the
`gui`, and deliberately as a list, not by oversight. (The time picker and rich
text were listed here until stage 25 and are built now — the list gets
shorter, not longer.)

At the moment nothing is listed here. Accordion, wizard and tree with columns
were the last three items and have been built since 2026-09-21 (section
below). Breadcrumb bar, status bar with fields and the toolbar overflow were
listed here until stage 27.

## Accordion, wizard, tree table

None of the three **creates children of its own**: you build the fields in
the window as always and assign them afterwards to a section, a step or (in
the tree) a parent row. A child must be created AFTER its container. That is
the same rule as with the tab control, and the reason is the order of hits: a
click goes to the first widget that it hits.

**Accordion.** Every section has a header (30 points high); an open section is
as high as its content plus margin, or as high as
`GUI_ACCORDION_SET_HEIGHT` says. A child's position counts from the top edge
of its section below the header. If a section above it expands, **the
children below move along**; without that they would lie on top of the
expanded content. Without `mehrere`, at most one is ever open. Keyboard: arrow
keys select a header, Enter and the space bar toggle it. If not everything
fits, the mouse wheel scrolls.

```basic
DIM acc AS GUI_WIDGET : acc = GUI_ACCORDION(win, 10, 10, 300, 360)
GUI_ACCORDION_ADD(acc, "Allgemein")
GUI_ACCORDION_ADD(acc, "Grafik")
DIM vollbild AS GUI_WIDGET : vollbild = GUI_CHECKBOX(win, "Vollbild", 12, 8, FALSE)
GUI_ACCORDION_ADD_WIDGET(acc, vollbild, 1)     ' 12/8 INSIDE the section "Grafik"
GUI_ACCORDION_OPEN(acc, 1, TRUE)
```

**Wizard.** At the top the step indicator, at the bottom the row of buttons;
in between, the wizard shows only the children of the current step. On the
last step, Next is called “Finish”. The events hold **for one frame**, like
`GUI_CLICKED`. With `pruefen`, Next only works once the fields of THIS step
fulfil their rules (`GUI_RULE`). The first wrong field gets the focus. An
error on a later step does not count yet, otherwise there would be no way
out. Enter presses Next.

```basic
DIM wz AS GUI_WIDGET : wz = GUI_WIZARD(win, 10, 10, 420, 300)
GUI_WIZARD_ADD(wz, "Konto") : GUI_WIZARD_ADD(wz, "Adresse")
DIM name AS GUI_WIDGET : name = GUI_TEXTINPUT(win, 30, 90, 200, 26, "Name")
GUI_WIZARD_ADD_WIDGET(wz, name, 0)
GUI_RULE(name, "pflicht")
GUI_WIZARD_SET(wz, "pruefen", 1)
' in the frame loop:
IF GUI_WIZARD_FINISHED(wz) THEN speichern()
```

**Tree table.** An ordinary table with the switch `baum`
(`GUI_TABLE_SET(t, "baum", 1)`; `GUI_TREETABLE` creates it that way). Sorting,
filtering, column widths, cell kinds and selection remain those of the table.
Only the **view** is different: children stand below their parents, and only
below expanded ones. Sorting happens among siblings, otherwise a child would
leave its parent. A filter shows a row if it or one of its descendants
matches, and opens the way there. All numbers passed to the outside are still
DATA rows; the order on screen comes from
`GUI_TABLE_VIEW_COUNT`/`GUI_TABLE_VIEW_ROW`. If a row is removed, its
children move up to its parent. A click on the triangle toggles without
selecting. Right expands or jumps to the first child, Left collapses or jumps
to the parent.

```basic
DIM tt AS GUI_WIDGET : tt = GUI_TREETABLE(win, 10, 10, 400, 250, ["Name", "Größe"])
DIM ex AS INTEGER : ex = GUI_TREETABLE_ADD(tt, -1, ["examples", ""])
GUI_TREETABLE_ADD(tt, ex, ["hallo.dh", "1 KB"])
GUI_TREETABLE_EXPAND(tt, ex, TRUE)
```

All three are stored completely in the `.dhform`: open sections, heights and
child positions under `akkordeon`, the check switch and button texts under
`assistent`, parent entries and expanded rows under `table` (`baum`,
`eltern`, `offen`). The form designer offers them in the palette; their
sections and steps go into the field “Einträge” (entries).

The wizard divides up its width: the three buttons are 112 points wide, and
narrower in a narrow wizard, so that they do not lie on top of each other.

Example: [`examples/200_gui_akkordeon_assistent.dh`](../../examples/200_gui_akkordeon_assistent.dh)
— a small project filing system with all three: settings in the accordion,
folders and files in the tree table, new entries via the wizard.

## Changing the look (theme, metrics, per widget)

The look can be controlled on three levels:

### 1. Ready-made colour schemes

```basic
GUI_THEME_PRESET("dark")          ' default (logo cyan), flat look
GUI_THEME_PRESET("light")         ' light scheme, flat
GUI_THEME_PRESET("retro")         ' green on black (terminal)
GUI_THEME_PRESET("contrast")      ' black/yellow, maximum contrast
GUI_THEME_PRESET("modern_dark")   ' PROFESSIONAL: anthracite + cyan, rounded corners + shadow
GUI_THEME_PRESET("modern_light")  ' PROFESSIONAL: light, close to Windows 11, rounded corners + shadow
```

The two **`modern_*`** presets are a *complete* look: they set not only the
colours but also the metrics (`corner_radius`, `title_h`, `pad`, `shadow`)
— this gives you **rounded corners, a soft window shadow, a taller title bar
and tick-mark checkboxes**. The classic presets (`dark`/`light`/…) reset the
metrics to the flat default. Complete example with a live switcher:
[`examples/128_gui_modern.dh`](../../examples/128_gui_modern.dh).

### 2. Individual theme colours (global)

```basic
GUI_THEME_SET(schluessel$, farbe)   ' set one palette colour
GUI_THEME_GET(schluessel$)          ' -> current colour (INTEGER)
GUI_THEME(accent)                   ' short form of GUI_THEME_SET("accent", ...)
```

Valid keys: `win_bg`, `win_border`, `title_bg`, `title_bg_focus`,
`title_fg`, `widget_bg`, `widget_border`, `text_fg`, `muted_fg`, `accent`,
`close_hover`.

```basic
GUI_THEME_SET("win_bg", RGB(20, 20, 30))
GUI_THEME_SET("accent", RGB(255, 160, 60))
```

### 3. Layout metrics (sizes)

```basic
GUI_METRIC_SET(schluessel$, wert)   ' set one metric (INTEGER pixels)
GUI_METRIC_GET(schluessel$)         ' -> current value
```

Keys: `title_h` (title bar height), `slider_h`, `check_size`,
`slider_handle_w`, `caret_period` (cursor blink), `pad` (text padding),
`corner_radius` (rounded corners for window/title bar/buttons/TextInput/Dropdown/
Progress/Panel + tick-mark checkbox; 0 = square/flat), `shadow` (soft
window shadow in pixels; 0 = off), `uebergang` (duration of the transitions in
milliseconds, default 120; 0 = jump, see [Transitions](#transitions)). **Note:** sizes that feed into the widget dimensions
(`check_size`, `slider_h`) only affect **newly created** widgets —
best set them before building the UI. `title_h`/`pad`/`caret_period`/`corner_radius`/
`shadow` take effect immediately.

### 4. Colouring a single widget (overrides the theme)

```basic
GUI_SET_COLOR(widget, rolle$, farbe)   ' role: "bg" / "fg" / "border" / "accent"
GUI_SET_COLOR(widget, rolle$, -1)      ' remove the override (back to the theme)
```

```basic
DIM warn AS GUI_WIDGET
warn = GUI_BUTTON(win, "Löschen", 20, 100, 100, 30)
GUI_SET_COLOR(warn, "bg", RGB(160, 40, 40))   ' only this button is red
```

Buttons have two more roles: `"hover"` (under the mouse) and
`"pressed"` (held down); without them the background becomes lighter or darker. And the
**text colour follows the background** as long as `"fg"` is not set: dark on a
light button, white on a dark one — previously it stayed white, and a
yellow button was unreadable.

**Button variants** instead of colours by hand:

```basic
DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(win, "OK", 20, 200, 100, 30)
DIM loeschen AS GUI_WIDGET : loeschen = GUI_BUTTON(win, "Löschen", 130, 200, 100, 30)
DIM mehr AS GUI_WIDGET : mehr = GUI_BUTTON(win, "Mehr ...", 240, 200, 100, 30)
GUI_BUTTON_VARIANT(ok, "primaer")       ' the theme's accent colour
GUI_BUTTON_VARIANT(loeschen, "gefahr")  ' red
GUI_BUTTON_VARIANT(mehr, "umriss")      ' border only, surface only under the mouse
PRINT GUI_BUTTON_GET_VARIANT$(ok)       ' primaer
```

| Variant | Appearance |
|---|---|
| `standard` | as before: the theme's colour |
| `primaer` | the theme's accent colour — for the one main action of a window |
| `erfolg` | green |
| `warnung` | orange |
| `gefahr` | red — for deleting, discarding, cancelling a run |
| `umriss` | no background, double border in the accent colour |
| `flach` | no surface until the mouse is over it (tool button with text) |
| `link` | only text in the accent colour, underlined under the mouse |

A colour of your own (`GUI_SET_COLOR(.., "bg", ..)`) wins over the variant. The
default button of a window (`GUI_WINDOW_DEFAULT`) still carries its
accent border — unless it is already `primaer`, then the surface says so. The
variant is stored in the `.dhform` as `variant`.

`GUI_RESET()` resets the theme **and** the metrics to the defaults
(and deletes all windows/widgets).

### 5. State & font per widget

```basic
GUI_SET_ENABLED(widget, FALSE)   ' disable: greyed out + not interactive
GUI_ENABLED(widget)              ' -> BOOLEAN
GUI_SET_FONT(widget, font)       ' own TTF font (handle from LOADFONT, -1 = default)
GUI_SET_FONT_SIZE(widget, px)    ' own text size (0 = standard)
```

A **disabled** widget is drawn in `muted_fg` and accepts no
mouse/keyboard input; in the editor it can still be selected via
`GUI_HIT_TEST`. Custom fonts:

```basic
DIM fnt AS INTEGER
fnt = LOADFONT("assets/Inter.ttf", 32)
DIM titel AS GUI_WIDGET
titel = GUI_LABEL(win, "Einstellungen", 20, 16)
GUI_SET_FONT(titel, fnt)
GUI_SET_FONT_SIZE(titel, 28)
```

### 6. Named styles (stylesheet)

Define a style once and apply it to many widgets — saves the
repeated `GUI_SET_COLOR`/`GUI_SET_FONT`:

```basic
GUI_STYLE_SET("primary", "bg", RGB(32, 80, 192))   ' props: bg/fg/border/accent
GUI_STYLE_SET("primary", "fg", &HFFFFFF)            '        + font / font_size
GUI_STYLE_SET("primary", "font_size", 18)

GUI_APPLY_STYLE(okBtn, "primary")
GUI_APPLY_STYLE(saveBtn, "primary")
```

`GUI_APPLY_STYLE` transfers the style properties as per-widget overrides
(colours) or font/size. Can be extended incrementally; `GUI_RESET()` also deletes
the styles.

## Callbacks: GUI_ON_CLICK

Besides polling (`IF GUI_CLICKED(b) THEN ...`) you can register a Drachenhauch
FUNCTION/SUB as a **callback** that is called automatically on a click:

```basic
SUB on_start()
    PRINT "Start gedrueckt!"
END SUB

DIM ok AS GUI_WIDGET
ok = GUI_BUTTON(win, "Start", 20, 150, 110, 32)
GUI_ON_CLICK(ok, on_start)        ' on_start is a FUNCREF
```

- The handler takes **no parameters** (you query the state inside it with `GUI_CHECKED`/
  `GUI_TEXT`/…). It runs like a normal function — it sees parameters and
  globals, but no locals of the surrounding scope (FUNCREF rule).
- **A method works too** — `GUI_ON_CLICK(ok, spieler.start)` binds the
  handler to the instance `spieler`, and `Self` inside it points to that instance. This is the
  way to go when the handler needs state: instead of a global variable plus a
  free `SUB`, the callback carries its object with it.

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

  **Limit:** a handler bound this way is **not** written by `GUI_SAVE`/`GUI_TO_JSON`.
  When loading, the instance would not exist, and saving only the
  method name would be misleading — it would be interpreted as a free function.
  Forms that are meant to survive as a `.dhform` (form designer)
  therefore still need free handler functions.
- Works for **buttons** (click = press+release on the button) and
  **checkboxes** (on every toggle).
- The callbacks are called **at the end of `GUI_UPDATE()`** (after all
  events of the frame have been processed). Further events triggered during a callback
  only fire in the next frame — no re-entrancy loop.
- Polling and callbacks are not mutually exclusive; both can be used at the same time.
- `GUI_ON_CLICK(widget, NIL)` removes the callback again.

Internally `GUI_ON_CLICK` bridges the built-in→VM boundary (calling a FUNCREF from
inside a built-in) — natively in `dhrt`.

### GUI_ON_CHANGE (value change)

`GUI_ON_CHANGE(widget, funcref)` calls the handler when the **value**
of a widget changes:

- **Slider** — while dragging, on every actual change of value
- **TextInput** — when typing/deleting (text change)
- **Checkbox** — on toggle (in addition to `GUI_ON_CLICK`)

```basic
SUB vol_changed()
    PRINT "Lautstaerke: " + STR$(INT(GUI_VALUE(vol) * 100.0))
END SUB

vol = GUI_SLIDER(win, 20, 44, 240, 0.0, 1.0, 0.5)
GUI_ON_CHANGE(vol, vol_changed)
```

Same rules as `GUI_ON_CLICK`: parameterless handler, called at the end of
`GUI_UPDATE()`, `NIL` removes it. Only allowed for `slider`, `textinput`
and `checkbox` (otherwise an error).

## Booleans

`GUI_CLICKED`/`GUI_CHECKED`/`GUI_HOVERED` return real BOOLEANs — they
work in conditions (`IF GUI_CLICKED(b) THEN ...`) and can be
printed directly:

```basic
DIM r AS BOOLEAN
r = GUI_CHECKED(snd)
PRINT r                   ' TRUE / FALSE
PRINT GUI_CHECKED(snd)    ' the same
```

## GUI_RESET

```basic
GUI_RESET()
```

Deletes all windows and widgets — useful when switching menu screens.
Caution: `GUI_WINDOW`/`GUI_WIDGET` references obtained beforehand then point to
objects that are no longer managed.

## Runtime manipulation (geometry / lifecycle / hit test)

Widgets and windows can be changed at runtime after they have been created — the
basis for **dynamic UIs** and a **WYSIWYG editor**. Handles stay
stable while doing so (deleting only marks as "dead", it does not shift any indices).

| Built-in | Effect |
|---|---|
| `GUI_SET_BOUNDS(wdg, x, y, w, h)` | move/resize a widget (relative to the window) |
| `GUI_GET_X/Y/W/H(wdg)` → INTEGER | read the current dimensions |
| `GUI_SET_VISIBLE(wdg, an)` / `GUI_VISIBLE(wdg)` → BOOLEAN | show/hide (invisible = not drawn, not interactive) |
| `GUI_DESTROY(wdg)` | remove a widget (the handle becomes invalid, others stay valid) |
| `GUI_KIND(wdg)` → STRING | `"button"`/`"label"`/`"checkbox"`/`"slider"`/`"textinput"`/`"panel"`/`"table"` |
| `GUI_FOCUS(wdg)` | set the keyboard focus (e.g. on a TextInput) |
| `GUI_FOCUSED()` → GUI_WIDGET | which widget has the keyboard focus? (`-1` = none) |
| `GUI_SCALE(faktor)` | display scale (0.5–4.0), **before** the first window |
| `GUI_SCALE_GET()` → FLOAT | current scale |
| `GUI_HIT_TEST(x, y)` → GUI_WIDGET | topmost widget at the screen point, or `-1` (selection in the editor) |
| `GUI_WINDOW_AT(x, y)` → GUI_WINDOW | topmost visible window at the screen point, or `-1` -- whoever evaluates the mouse wheel themselves asks here whether their area is currently on top |
| `GUI_WINDOW_SET_BOUNDS(win, x, y, w, h)` / `GUI_WINDOW_GET_X/Y/W/H(win)` | move/resize/read a window |
| `GUI_WINDOW_DESTROY(win)` | remove a window + its contents |
| `GUI_WINDOW_WIDGET_COUNT(win)` → INTEGER | number of living widgets |
| `GUI_WINDOW_WIDGET(win, n)` → GUI_WIDGET | n-th living widget (enumeration/serialisation), or `-1` |

```basic
' Editor idea: a click selects the widget under the mouse, dragging moves it.
DIM sel AS GUI_WIDGET
sel = GUI_HIT_TEST(MOUSEX(), MOUSEY())
IF sel <> -1 THEN
    GUI_SET_BOUNDS(sel, MOUSEX() - win_x, MOUSEY() - win_y - 22, GUI_GET_W(sel), GUI_GET_H(sel))
END IF

' Go through all widgets of a window (e.g. for saving):
DIM i AS INTEGER
FOR i = 0 TO GUI_WINDOW_WIDGET_COUNT(win) - 1
    DIM wdg AS GUI_WIDGET
    wdg = GUI_WINDOW_WIDGET(win, i)
    PRINT GUI_KIND(wdg), GUI_GET_X(wdg), GUI_GET_Y(wdg)
NEXT
```

## Serialisation (layout as JSON)

A window together with its widgets can be saved as JSON and loaded
again — this **closes the loop between a (WYSIWYG) editor and the
runtime**: the editor writes JSON, the app loads it (like `ATLAS_LOAD`/
`TILED_LOAD`). Preserved are dimensions, texts, states (checked/value), colour
overrides, window flags and table data. Destroyed widgets are not
saved.

| Built-in | Effect |
|---|---|
| `GUI_SAVE(win, pfad$)` | save a window as a JSON file |
| `GUI_LOAD(pfad$)` → GUI_WINDOW | load a JSON file, new window, returns the handle |
| `GUI_TO_JSON(win)` → STRING | window as a JSON string (for network/embedding/the `json` module) |
| `GUI_FROM_JSON(json$)` → GUI_WINDOW | build a window from a JSON string |

```basic
' Save the design ...
GUI_SAVE(win, "forms/login.json")

' ... and rebuild it in the app:
DIM win AS GUI_WINDOW
win = GUI_LOAD("forms/login.json")
' Callbacks/handlers are NOT loaded along -> wire them up again here
' (FUNCREF = bare function name):
GUI_ON_CLICK(GUI_WINDOW_WIDGET(win, 0), on_login)
```

> Note: `GUI_ON_CLICK`/`ON_CHANGE` store the function **name**; when
> loading in a different program, the function must exist. For portable
> designs, set the handlers after loading via enumeration (`GUI_WINDOW_WIDGET`).

### Form workflow (Xojo style)

Because the `.dhform` stores the **name of its event handler** for each control
(`on_click` / `on_change`) and `GUI_UPDATE` **calls triggered handlers automatically by
name**, the Xojo workflow comes naturally: load the form, just fill in the
handlers — no manual wiring.

```basic
IMPORT "gui"
SCREEN(800, 480, "App", 1)

DIM frm AS GUI_WINDOW
frm = GUI_LOAD("forms/settings.dhform")   ' controls + handler names

SUB on_save()                              ' you write ONLY the handlers ...
    PRINT "Speichern geklickt"
END SUB

WHILE NOT QUITREQUESTED()
    GUI_UPDATE()                           ' ... GUI_UPDATE calls on_save automatically
    CLS(0) : GUI_DRAW() : FLIP()
WEND
```

Complete example (form `examples/forms/settings.dhform` +
[examples/105_form_runner.dh](../../examples/105_form_runner.dh)): a
settings dialog with TextInput, Checkbox, Slider, Dropdown and two buttons,
whose handlers read out the control values. The `.dhform` can be written by hand
**or** (in future) generated by a visual designer — both
result in the same JSON.

## Complete example

See [examples/45_gui.dh](../../examples/45_gui.dh): a window with a slider (mirrored live into the
label), checkbox, text input and start button — movable and
closable.

**All 22 widget kinds in one application:**
[examples/156_gui_alle_widgets.dh](../../examples/156_gui_alle_widgets.dh) —
fullscreen, borderless window (the form *is* the screen), menu bar,
toolbar, context menu and three tabs. Not a showcase: every widget
has a job. The tree filters the table, a table row fills the
editor, "Apply" writes back; on the second tab
knobs, sliders, radios and switches shape the curve on the
canvas live, the list loads presets; the third tab keeps
key figures from the real data. All four
themes can be switched via the menu.

## Limitations (current state)

- **Events**: polling (`GUI_CLICKED` …) **and** FUNCREF callbacks
  (`GUI_ON_CLICK` for buttons/checkboxes, `GUI_ON_CHANGE` for slider/TextInput/
  checkbox) are supported.
- **Immediate-mode windows** (`UI_WINDOW_BEGIN/END` in the `ui` module) are the
  planned alternative (phase 4).
- **Coordinates plus anchors**, plus auto-size and row/column/grid containers
  (`GUI_LAYOUT`); no constraint system.
- **One program, one OS window**; gui windows live inside it — not beyond the
  edge, not on the second monitor, without their own entry in the taskbar.
  Why, and which paths there would be: [entwurf-native-fenster.md](../entwurf-native-fenster.md) (German).
- **Runtime manipulation** (moving/resizing/deleting/showing/hiding/
  hit test/enumeration) is supported — see the section above.
- **Headless/graphical**: `GUI_UPDATE`/`GUI_DRAW` need an active
  `SCREEN`. Construction, state, geometry and hit test are tested headless
  (`tests/pruef/gui_runtime.dhtest`).


## A sculpted look: the glass themes

> To look at: [examples/155_gui_glas.dh](../../examples/155_gui_glas.dh) —
> there `f` switches between flat and sculpted, `g` puts a graphic of your own
> on the buttons.

`GUI_THEME_PRESET("glas_dunkel")` or `"glas_hell"` switch on a
domed look: a vertical gradient on every surface, a gloss edge over the
upper half, a fine bevel at the top and bottom.

This lives in four **metrics**, not in colours — that way a theme is a
complete look instead of several things you have to combine by hand:

| Metric | Meaning |
|---|---|
| `gradient` | brightness difference top/bottom; 0 = flat |
| `gloss` | strength of the gloss edge, 0…100 |
| `bevel` | 1 = light line at the top, dark one at the bottom |
| `verlauf_hoehe` | from this height on, both fade out; 0 = the same everywhere |

Settable individually via `GUI_METRIC_SET`. All **existing** themes are still
at 0 — programs that have already been written look unchanged.

**Why the fourth.** `gradient` is a fixed brightness difference, no matter how
tall the surface is. A button of 28 pixels may dome with it; a
list of 400 gets the same difference over its whole height and looks
as if a shadow lay over it. `verlauf_hoehe` says from when on it becomes
less: above it the strength decreases proportionally, one fifth remains,
so that a tall surface does not look completely like paper.
`GUI_METRIC_SET("verlauf_hoehe", 120)` is a good start for an
interface with large lists and text inputs.

**Raised and sunken.** Buttons, panels and selection fields are raised
(light at the top → dark at the bottom, with gloss). Input fields, lists and the
progress trough are sunken: the gradient runs the other way and instead of the
gloss edge there is a shadow below the upper edge. Without this difference
an input field looks like a button, and the interface loses its
statement about what you click and what you fill in.

## Transitions

Under the mouse a button blends into its hover colour instead of
jumping, and back again when the mouse leaves. The same applies to cards,
checkboxes, radio buttons, dropdowns, the toolbar entries, the
focus ring and the toggle switch, whose knob accelerates and slows down.

An **accordion** opens and closes over the same duration: the section
grows, the headers below move along, the triangle rotates, and the
contents slide in (they are clipped to the part that is already visible).
**A child can only be clicked once it is fully visible.**
A section that is opened while building (before the first `GUI_UPDATE`)
is open immediately. Whoever needs the final position of a child with
`GUI_GET_Y` directly after `GUI_ACCORDION_OPEN` sets `uebergang` to 0 or
waits out the duration.

A **tree** opens the same way: the child rows grow out of their parent row
and shrink back into it when closing, the angle turns from `>`
to `v`. Hit test, arrow keys and scrolling already calculate with the
new state, only the picture catches up. The **file tree** opens
smoothly; it closes immediately, because it no longer reads the children of a closed
folder at all.

A **dropdown** rolls down from the top when opening and fades in at the same time.
It closes immediately — whoever has chosen no longer wants to see the list. A
click during the roll-out still hits the entry it
points at.

**Menus** roll open the same way, submenus too. The **row under the
mouse** fades in smoothly in lists, tables, trees, the open dropdown and in
menus, while the one it left fades out at the same time. Keyboard cursor and
open submenus are fully there immediately. The **mouse wheel** scrolls a list
smoothly (see *Lists made comfortable*).

| Metric | Meaning |
|---|---|
| `uebergang` | duration in milliseconds, default 120; 0 = jump as before |

```basic
GUI_METRIC_SET("uebergang", 200)   ' more leisurely
GUI_METRIC_SET("uebergang", 0)     ' off
```

**Only hovering fades in.** Pressing and focus appear in the
same frame and only fade *out*: whoever clicks or moves on with Tab wants to
see the response immediately, not three frames later. The final picture is in every
case the same as without a transition, only the way there changes. What a
program queries (`GUI_HOVERED`, `GUI_CLICKED`, `GUI_FOCUSED`) still applies
from the first frame. The transitions only affect what you see.

The calculation uses the real frame time (`DELTA()`), not frames: at
144 Hz a transition is just as long as at 60. Without a window (`DHRT_FRAMES`)
`DELTA()` is fixed at 1/60 s, so every recording shows the same stage.

## Cards

A card (`GUI_CARD`) shows an image, below it a title and a
description -- and accepts the click **everywhere**, not just on a
button below it. Under the mouse it lifts and gets an accent border,
pressed it sinks; with focus, Enter and the space bar trigger it.

```basic
DIM k AS GUI_WIDGET
k = GUI_CARD(win, 20, 60, 200, 190, LOADIMAGE("vorschau.png"), _
             "Schatten", "Tiefenpuffer mit weichen Kanten")
GUI_TOOLTIP(k, "examples/93_shadows.dh")
' ...
IF GUI_CLICKED(k) THEN starteBeispiel()
```

The image takes up the inner width at a ratio of 16:10 and fills it
(`GUI_IMAGE_MODE` changes that); without an image (-1) only the text is there. The
description is wrapped in the widget's font -- not smaller;
if it does not fit, the last line ends with "...", and a tooltip can say the
rest. The title is set and read as with any widget (`GUI_SET_TEXT`,
`GUI_TEXT`), the description with `GUI_CARD_SET_TEXT`/`GUI_CARD_TEXT$`. In
the `.dhform` the kind is `card`, the title is under `text` and the
description under `placeholder`; the image is a handle and, as with the
image widget, is not saved.

## Toggle switches, rotary knobs, round buttons

Three controls that give the glass themes their effect:

```basic
DIM t AS GUI_WIDGET
DIM k AS GUI_WIDGET
DIM b AS GUI_WIDGET

t = GUI_TOGGLE(w, "Musik", 24, 20, TRUE)          ' on/off pill
k = GUI_KNOB(w, 220, 20, 90, 0.0, 100.0, 72.0)    ' rotary knob
b = GUI_BUTTON(w, ">", 24, 285, 42, 42)
GUI_SET_ROUND(b, TRUE)                            ' round transport button
```

**`GUI_TOGGLE(win, text$, x, y [, an])`** — as with the
checkbox, the state is in `checked`, so it can be read with `GUI_CHECKED` and set with
`GUI_SET_CHECKED`. The knob glides across when switched and the track takes
on the colour; both run over an internal value, not abruptly.

**`GUI_KNOB(win, x, y, groesse, min, max [, wert])`** — adjusted by
**dragging vertically** (up = more). That is how mixing-desk
interfaces are operated and it works without moving the mouse in a circle; 140 pixels
correspond to the whole range. Value via `GUI_VALUE` / `GUI_SET_VALUE`.
The value arc runs around the metal cap over 270°, the notch shows the position.

**`GUI_SET_ROUND(widget, an)`** — draws a button round instead of square. For
buttons that only carry a symbol: a round button with a triangle inside reads
immediately as a play button.

Checkboxes, radio buttons and sliders are sculpted as well: the empty
checkbox is a sunken hollow, the set one a domed surface in the
accent colour; the slider has a sunken track whose covered part
is coloured, and a metallic handle.


## Graphics of your own: 9-slice skins

Where the drawn look is not enough, a graphic replaces the surface of a
widget type:

```basic
DIM haut AS IMAGE
haut = LOADIMAGE("assets/knopf.png")
GUI_SKIN("button", haut, 12)      ' 12 px border
GUI_SKIN("button", -1)            ' take it away again
```

`GUI_SKIN(art$, bild, rand)` applies to **all** widgets of this kind. The rest —
label, tick mark, slider handle — is still drawn; only
the background comes from the image.

**Why 9-slice and not simply scaling?** The image is divided into nine pieces:
the four corners stay unchanged, the edges only stretch
along their axis, the middle in both directions. A plainly scaled
image would pull its rounded corners into ellipses as soon as the button becomes wider
than the template.

The border is capped at half the image side **and** half the target side. A widget that
is smaller than its skin borders thus shrinks cleanly instead of
overlapping itself.

Permitted kinds are all widget names that `GUI_KIND` returns (`button`,
`panel`, `textinput`, `listbox`, `progress`, …).


## Font per widget

Globally the font set via `SETFONT` applies. Individual widgets can
deviate from it:

```basic
DIM k AS GUI_WIDGET
k = GUI_BUTTON(w, "Gross", 20, 20, 180, 40)
GUI_SET_FONT_SIZE(k, 24)          ' only the size
GUI_SET_FONT(k, andere_schrift)   ' another typeface (handle from LOADFONT)
```

`GUI_SET_FONT_SIZE` changes **only** the size — the typeface remains the
globally set one. `GUI_SET_FONT(wdg, -1)` takes back a font of the widget's own.

Via `GUI_STYLE_SET` / `GUI_APPLY_STYLE` font and size can also be transferred
to several widgets as a named style.

**What matters here:** the label is *measured* in the font in which
it is also drawn. Otherwise centred text would sit askew and the
clipping would take effect at the wrong place as soon as a widget has a font
or size of its own.


## The glass look in the `ui` module

The immediate-mode module `ui` knows the same two themes:

```basic
IMPORT "ui"
UI_THEME_PRESET("glas_dunkel")     ' or "glas_hell"
```

It brings the same metrics (`gradient`, `gloss`, `bevel`,
`corner_radius`, individually via `UI_METRIC_SET`), and a preset sets colours
**and** sculpting — whoever switches from a glass theme to a flat one does
not keep any doming.

Converted are button, checkbox, slider, progress, input field, panel
and window. Here too the button label sits in the centre and is
clipped inside the button instead of overflowing.

All previous `ui` themes (`dark`, `light`, `retro`, `contrast`) are still
at 0 and look flat and unchanged.


## Layout: size by content and containers

Up to here every widget sits where you write it, and anchors
let it flow along when the window is enlarged. For a form you do
not want to calculate pixel by pixel, two things have been added since 2026-09-04.

**Size by content.** `GUI_AUTOSIZE(wdg)` measures button, label,
checkbox, radio and toggle switch by their text (for text inputs only the height).
A `0` as width or height when creating achieves the same — measuring happens
in the next `GUI_UPDATE`, because only there is the font available. `GUI_SET_BOUNDS`
cancels the auto-size again.

**Layout containers.** `GUI_LAYOUT(win, art$, x, y, w, h)` creates an
invisible container, `art$` is `"zeile"` (row), `"spalte"` (column) or
`"raster:3"` (grid). Whatever you put into it with `GUI_LAYOUT_ADD` it redistributes in every
`GUI_UPDATE`:

```basic
DIM form AS GUI_WIDGET : form = GUI_LAYOUT(win, "spalte", 12, 12, 376, 236)
GUI_SET_ANCHOR(form, "lrtb")                  ' the container flows with the window
GUI_LAYOUT_SET(form, "abstand", 8)

DIM lbl AS GUI_WIDGET : lbl = GUI_LABEL(win, "Nachricht", 0, 0)
DIM txt AS GUI_WIDGET : txt = GUI_TEXTAREA(win, 0, 0, 10, 10)
GUI_LAYOUT_ADD(form, lbl)                     ' weight 0: own height
GUI_LAYOUT_ADD(form, txt, 1)                  ' weight 1: gets the rest

DIM knoepfe AS GUI_WIDGET : knoepfe = GUI_LAYOUT(win, "zeile", 0, 0, 10, 30)
GUI_LAYOUT_SPACER(knoepfe)                    ' pushes the buttons to the right
GUI_LAYOUT_ADD(knoepfe, GUI_BUTTON(win, "Senden", 0, 0, 0, 0))
GUI_LAYOUT_ADD(knoepfe, GUI_BUTTON(win, "Abbrechen", 0, 0, 0, 0))
GUI_LAYOUT_ADD(form, knoepfe)                 ' container inside a container
```

The rules, briefly:

- **Weights** share the space that is left after the fixed children,
  proportionally; weight 0 keeps its own size. `GUI_LAYOUT_SPACER` is an
  empty space with a weight — an empty space at the start of a row pushes everything to
  the right.
- **Across the direction** (in a row: the height) children are stretched
  (`dehnen`, default on) or placed according to `ausrichtung`: 0 start, 1 centre,
  2 end. `abstand` lies between the children, `rand` inside the container.
- A **grid** has columns of equal width; the row height is that of the
  tallest child in it.
- Labels in a container are always measured exactly (not with the
  estimate from `GUI_LABEL`), otherwise the rows would not line up.
- A widget sits in at most **one** container; a container may not take in
  itself or one of its ancestors. The container is **air for
  clicks** — it swallows nothing meant for its children. `rahmen` draws
  it dashed, for development.
- Anchors apply to the **container**; its children get their position from it.

**Minimum sizes:** `GUI_SET_MIN_SIZE(wdg, min_w, min_h)` — a weighted child
never falls below it (then the row would rather overflow than let a button become
a line), nor does an anchored widget when the window shrinks.
A container brings along the size of its content (fixed children at their
natural size, weighted ones at their minimum size, plus spacing and border),
also across several levels. `GUI_LAYOUT_MIN_W/H(layout)` return it — the
right value for `WINDOW_MIN_SIZE`.

No constraint system — for a form this is enough, and you still understand it. In the `.dhform` the container is stored with `layout`
(kind, dimensions, `kinder` as `[Index, Gewicht]`); the form designer knows it as a
palette entry with a "Layout" field per control. Example:
[`examples/194_gui_layout.dh`](../../examples/194_gui_layout.dh).

## Indentation on line break

`GUI_TEXTAREA_SET(ta, "auto_einzug", 1)` makes a new line start with the
indentation of the old one. That alone is language-independent and already half the
benefit; the words that mean one level more or less come from
outside:

```basic
GUI_TEXTAREA_SET(ta, "auto_einzug", 1)
GUI_TEXTAREA_INDENT_WORDS(ta, ["SUB", "FUNCTION", "FOR", "WHILE", "IF"], _
                              ["THEN"], _
                              ["END", "NEXT", "WEND", "ELSE"])
```

- **`anfang`** (start) — the line starts with the word, the next one is indented.
- **`ende`** (end) — the line ends with it. There is a separate list for this:
  `SUB` opens at the **start**, `THEN` at the **end** — with a single list
  an `END SUB` would also indent the next line. And `IF x THEN y = 1` does not end
  with `THEN`, so it correctly does not indent.
- **`aus`** (out) — if this word stands **alone** in a line, the line itself moves
  back one level. The condition is "the line **is** the word", not
  "starts with it": that way it dedents exactly once, and whatever is
  added afterwards (`END IF`) changes nothing any more. For the same reason a pasted block
  does not trigger it.

The runtime knows no language here — the same principle as with
folding. A text area without `auto_einzug` behaves unchanged.

## The skeleton that grows along

`GUI_TEXTAREA_CLOSE_WORDS(ta, oeffner, schluesse)` also inserts the line that closes the block
right away on a line break:

```basic
GUI_TEXTAREA_CLOSE_WORDS(ta, ["IF", "SUB", "FOR", "WHILE"], _
                             ["END IF", "END SUB", "NEXT", "WEND"])
```

With it, `IF x > 0 THEN` plus Enter becomes

```
IF x > 0 THEN
    |
END IF
```

with the cursor in the empty line. Two lists of equal length — word and
closing belong together in pairs; the **longest** matching word wins,
so that `DO WHILE` comes before `DO`. The spelling of the closing stays as
it comes in: it is written, not compared.

Three conditions, each with its reason:

- The line must **open a block** — the same question that already decides the
  indentation. Without it `IF x THEN y = 1` would get an `END IF`,
  although it does not even indent.
- **Nothing may follow** the cursor, and nothing may be selected:
  otherwise the rest of the line would end up behind the closing.
- The block must **not already be closed**. For this the runtime goes
  downwards: more deeply indented lines are the body, a line from the
  `aus` list at the same level still belongs to it (`ELSE`, `CASE`), anything
  else ends the search. Without this every Enter at the end of an
  existing `SUB` line would sow a second `END SUB`. With crookedly indented
  files the answer falls on "not closed" — one closing too many
  is easier to see than one too few.

With **several cursors** it stays a plain line break: nobody wants five closings
at once. Two empty lists switch it off.

## Closing tabs

`GUI_TABS_CLOSABLE(win, TRUE)` gives every tab a cross; `GUI_TAB_CLOSED(win)`
says which one should go (-1 = none), transient like `GUI_CLICKED`. It is hit
via the cross or — as in a browser — via the **middle** mouse button
anywhere on the tab.

**Nothing is closed.** What a closed tab means, only
the program knows: its widgets hang on it, and maybe it first wants to
ask whether unsaved work may be lost. A click next to the cross
switches as before; without the switch nothing changes at all, the tabs
are just narrower then.

## The Tab key, when it means more than one thing

`GUI_TEXTAREA_SET(ta, "tab_meldet", 1)` gives the Tab key entirely to the
caller: the field does not indent, does not look for abbreviations, but
**reports** it via `GUI_TEXTAREA_TAB_HIT(ta)` — transient like
`GUI_CLICKED`, a key press is an event.

The reason is the order. Whoever has three meanings for the Tab key in the same field
— jump to the next placeholder, expand a snippet,
otherwise indent — can only query them one after another themselves; the runtime
would decide the first two beforehand. But then the
indenting belongs to them too: `GUI_TEXTAREA_CURSOR` says the column, `GUI_TEXTAREA_INSERT`
inserts the spaces up to the next one. Without the switch everything stays as
it was.

The field thereby **keeps** the key: the Tab key then does not additionally
move the focus on, otherwise one key would do two things. A field
you are supposed to leave with the Tab key therefore does not set the switch.

## Several cursors in the text area

**Alt+click** places another cursor, **Esc** removes them, as does
an ordinary click, a right-click, Ctrl+A and `GUI_TEXTAREA_GOTO`/
`GUI_TEXTAREA_SELECT` -- they all mean **one** position; a
program sets them with `GUI_TEXTAREA_ADD_CARET`. Typing, Enter, Backspace,
Del, Tab and pasting then act at **every** cursor, and the arrows
move all of them — if the others stayed put, they would drift apart at the first
key press.

Everything runs through **one** place (`an_marken`): for each cursor it says
which range gives way and what goes in, and it works from **back to
front**. Then the positions of the cursors still open stay valid, and no
bookkeeping about shifts is needed. With a single cursor this is
exactly the path from before — a text area without Alt+click therefore behaves
unchanged.

Two things stay with the **leading** cursor: copying and cutting
(what several pieces in the clipboard are supposed to mean is not agreed upon outside the
program), and Ctrl+A removes the others — a
selection over everything with three cursors next to it would not give a picture
anyone has in mind.

## Folding in the text area

A collapsed block hides its inner lines; the header line stays
in place and carries a speech bubble behind it ("… 12 Zeilen"). In the
line-number column there is a triangle at every header line — pointing down when open,
pointing right when collapsed —, and a click on it toggles without taking the cursor
along.

**Which lines form a block is said by the program**, not by the runtime:

```basic
' The blocks from CODE_SYMBOLS$ -- from/to are already line numbers
GUI_TEXTAREA_FOLDABLE(ta, [1, 14, 30], [12, 22, 44])
GUI_TEXTAREA_FOLD(ta, 14)          ' toggle
GUI_TEXTAREA_FOLD_ALL(ta, TRUE)    ' collapse all (only the outer ones)
```

The runtime does not count indentation and knows no language here — a
text area with YAML, Markdown or sections of its own folds with the same
two fields. Without foldable blocks folding costs no space: the
line-number column only becomes wider when there are triangles to show.

Three things are not obvious:

- **The narrowest block wins.** `GUI_TEXTAREA_FOLD(ta, zeile)` folds the
  smallest foldable block around the line — otherwise a method would take its
  whole class along when collapsing.
- **A cursor in the hidden part unfolds.** Clicks and arrows run over the
  visible lines and do not get in at all; `GUI_TEXTAREA_GOTO`,
  Left/Right and Backspace can. The block then opens instead of
  the cursor dodging: with a search hit you want to see the match.
  Conversely the cursor moves to the header line when collapsing.
- **Folding hangs on line numbers.** A change above it shifts it
  along; whoever types in the block itself unfolds it; `GUI_SET_TEXT` clears it
  away (the new text has different lines). What is currently collapsed is read by
  `GUI_TEXTAREA_FOLDS` — enough to restore it the next time the file is
  opened.

## Word wrap in the text area

The text area was a code field: long lines scroll horizontally. For
notes and letters `GUI_TEXTAREA_SET(ta, "umbruch", 1)` switches on wrapping
at word boundaries (a word that does not fit on its own breaks at a character).
Home, End and the arrows then move in **visible** lines, as in
any editor (`Strg+Pos1`/`Strg+Ende` always go to the start or the
end of the whole text); line numbers only appear at the first line of a paragraph;
`GUI_TEXTAREA_VIEW` still counts logical lines. With wrapping there is no
horizontal offset. In the `.dhform`: `wrap_text`.

## Data binding

The sixth pilot filled three forms from the database by hand and wrote them
back by hand: one `GUI_SET_TEXT` per field, one parameter in the `UPDATE`,
and a glued-together text for the question "changed?". Since 2026-09-05 a
widget carries a **key**, and a single call takes care of the form:

```basic
GUI_BIND(tfName, "name", "kunde")
GUI_BIND(tfPlz, "plz", "kunde")
GUI_BIND(cbAktiv, "aktiv", "kunde")
GUI_BIND(ddArt, "art", "kunde")

' load, save, ask -- without a single column list in the program
IF NOT GUI_FORM_LOAD(win, db, "kunden", id, "kunde") THEN status("weg")
IF GUI_VALIDATE(win) = 0 THEN id = GUI_FORM_SAVE(win, db, "kunden", id, "kunde")
IF GUI_FORM_CHANGED(win, "kunde") THEN nachfragen()
```

- **Form name** (third parameter, empty by default): several forms in one
  window — on tabs, for example — keep apart this way, and the same key
  `name` may occur in two of them.
- **What can be bound:** text input, text area, label, checkbox, toggle
  switch, dropdown, slider, spin box, knob, date and colour picker. The key
  may contain only letters, digits and `_` — it becomes a column name.
- **Values:** `GUI_FORM_GET` returns everything as text (checkbox `1`/`0`,
  dropdown its index, numbers with a dot). `GUI_FORM_SET` takes any MAP and
  is tolerant: `"ja"`, `1` or `TRUE` set a check mark, an entry text or an
  index selects in the dropdown, a number as text lands in the slider. Keys
  that are not named leave their widget alone.
- **Database:** `GUI_FORM_LOAD` reads `SELECT schlüssel… FROM tabelle WHERE
  id = ?` and returns FALSE if the row does not exist; `GUI_FORM_SAVE`
  writes `INSERT` (for `id < 0`, returns the new id) or `UPDATE`. The
  column is called `id` — SQLite's convention. Values are stored **typed**:
  a numeric field as a number (unless it starts with a zero like a postcode
  — that stays text), a checkbox as 0/1, a dropdown as its index, a slider
  as a floating-point number, everything else as text. Whatever the program
  converts (amounts in cents, a dropdown into a tax rate) it does not bind
  and writes itself.
- **Changed?** On binding, after `GUI_FORM_SET/LOAD/SAVE/CLEAR` and after
  `GUI_FORM_CLEAN` the state counts as clean; `GUI_FORM_CHANGED` compares
  against it. That makes the prompt "Save | Discard | Cancel" a single
  line.
- In the `.dhform`: `bind` and `form`; the form designer has the fields
  "Binding" and "Form" for them.

Deliberately not included: binding table rows to lists, resolving foreign
keys, joins. That is the border between a form and an ORM, and beyond it a
`DB_QUERY` is more honest.

## Form validation

The sixth pilot (the invoice manager) wrote the same validation by hand
three times: required field, postcode, e-mail, number, date, each time with
an error text, focus and a red label. Since 2026-09-05 **rules hang on the
field**, and one call validates the window:

```basic
GUI_RULE(tfName, "pflicht", "Der Name fehlt.")
GUI_RULE(tfName, "laenge", 2, 60)
GUI_RULE(tfPlz, "muster", "[0-9]{5}", "Eine Postleitzahl hat fünf Ziffern.")
GUI_RULE(tfMail, "email")
GUI_RULE(tfPreis, "zahl") : GUI_RULE(tfPreis, "bereich", 0, 100000)
GUI_RULE(tfDatum, "datum")
GUI_RULE(cbAgb, "pflicht", "Bitte zustimmen.")
GUI_ERROR_LABEL(tfName, lblFehler)         ' the message appears here

IF GUI_CLICKED(bSpeichern) THEN
    IF GUI_VALIDATE(win) = 0 THEN speichern()
END IF
```

- **Kinds:** `pflicht` (required: text not empty; dropdown selected;
  checkbox on), `zahl` (number, comma or dot), `ganz` (integer),
  `bereich a b` (range), `laenge min max` (length, in characters), `email`,
  `datum` (date, `JJJJ-MM-TT`, a real calendar day), `muster` (pattern: a
  regular expression that applies to the **whole** input). The last text is
  your own message; without it a German default applies.
- **Empty passes every rule except `pflicht`.** Whether a field may be
  empty is decided by `pflicht` alone — otherwise "range 1..10" would also
  mean "must be filled in", and you cannot see that in the rule.
- `GUI_VALIDATE(win)` checks all visible, enabled fields that have rules,
  returns the number of errors and puts the focus on the first wrong field.
  Every field with an error gets a red frame; its message appears in the
  tooltip (before any help text of your own) and in the label from
  `GUI_ERROR_LABEL`. Fields on another tab or hidden ones do not count — an
  error there would be one with no way out.
- `GUI_SET_ERROR(feld, meldung$)` sets a message from outside, such as
  "number already taken" from the database; an empty text removes it.
  `GUI_CLEAR_ERRORS(win)` clears them all.
- `GUI_VALIDATE_LIVE(win, TRUE)` checks a field as soon as the focus leaves
  it — not on every keystroke: a half-typed address is not a wrong one.
- Rules are stored in the `.dhform` (`rules`, `error_label`); the form
  designer has a field "Rules" for them (`pflicht; laenge 2 60; email`).

Deliberately not included: a custom rule as a FUNCREF. Whatever a rule
cannot do, the program checks itself after `GUI_VALIDATE` and reports with
`GUI_SET_ERROR` — then it goes through the same display.

## Finishing touches: slider, progress, image, tree, panel, dragging

The small things that tell an application from a demo (since 2026-09-05):

- **Vertical slider** `GUI_VSLIDER(win, x, y, h, min, max, default)`: like
  `GUI_SLIDER`, except that `h` stands in place of `w`, and the value grows
  upwards — as on any mixing desk. Arrow up/right increases, Home/End jump.
- **Indeterminate progress** `GUI_PROGRESS_SET(p, "unbestimmt", 1)`: a band
  runs, driven by time, not by the value. For anything whose duration you do
  not know (a search, a connection). `0` switches back.
- **Image modes** `GUI_IMAGE_MODE(img, modus$)`: `strecken` (stretch, as
  before), `einpassen` (fit: fully visible, aspect ratio kept, centred),
  `fuellen` (fill: area covered, overhang cut off), `mitte` (centre,
  original size), `kacheln` (tile). `GUI_IMAGE_MODE_GET` reads it.
- **Tree icons** `GUI_TREE_ICON(tree, node, bild)` and
  `GUI_TREE_COLOR(tree, node, farbe)`: as soon as one node has an icon,
  every row gets the room for it, otherwise the names would be misaligned.
- **Scrolling panel** `GUI_PANEL_ADD(panel, wdg)`: the children keep their
  position in the window (`GUI_GET_Y` does not change while scrolling), the
  panel only lays a view offset over them. Whatever is scrolled out can
  neither be seen nor hit. The mouse wheel over the panel scrolls (a list
  inside gets the wheel first), a scroll bar appears as soon as the content
  is taller; `GUI_PANEL_SCROLL`/`_GET` set and read it. A layout container
  in the panel takes its children along. Create the children after the
  panel, otherwise they lie under its area. A scrolling panel inside a
  scrolling panel does not exist.
- **Dragging between widgets**: `GUI_DRAGGABLE(wdg, TRUE)` makes a widget a
  source (for list and tree the row that was hit, otherwise the widget
  itself with its text), `GUI_DROP_TARGET(wdg, TRUE)` a drop target. A press
  becomes a drag after 5 px of movement — a click stays a click. During the
  drag the text hangs on the mouse and the drop target underneath gets a
  frame; `GUI_DRAGGING()` returns the source. When released over a drop
  target, `GUI_DROPPED(ziel)` reports **for one frame** (like
  `GUI_CLICKED`), plus `GUI_DROP_TEXT()`, `GUI_DROP_SOURCE()`,
  `GUI_DRAG_INDEX()` (row in the source) and `GUI_DROP_INDEX()` (row in the
  target, -1 = none). Released next to a drop target, the drag fizzles out
  silently. **Within the same list the list reorders itself** — that is the
  one meaning a drag can have there; what else happens to the entry is up
  to the program:

  ```basic
  GUI_DRAGGABLE(vorrat, TRUE) : GUI_DROP_TARGET(einkauf, TRUE)
  ...
  IF GUI_DROPPED(einkauf) THEN
      GUI_LISTBOX_ADD(einkauf, GUI_DROP_TEXT())
      GUI_LISTBOX_REMOVE(vorrat, GUI_DRAG_INDEX())
  END IF
  ```

- **Checkbox, radio and toggle switch** extend over their label since
  2026-09-05: a click on the text hits, and a layout container reserves the
  whole space. The box itself keeps the size of the metric `check_size`,
  however wide the rectangle is.
- **Pointer shapes** come by themselves, and according to the SPOT in the
  widget, not just its kind:

  | Where | Pointer |
  |---|---|
  | Text input, text in the text area, editable dropdown (left of the arrow), filter row and cell being edited in a table, renaming in list and tree, rich text | I-beam |
  | Line-number column and scroll bar of the text area | Arrow |
  | Button, card, checkbox, radio, toggle switch, colour swatch in the text area, link in the rich text, header of a sortable table, enabled toolbar button (and its »), clickable status bar field, part of the breadcrumb bar (not the last one) | Hand |
  | Column edge in the table header (also while dragging), splitter, window grip | Double arrow |
  | Field and strip of the colour picker | Crosshair |
  | Dragging: over a drop target or its own source Hand, otherwise »not allowed« | |
  | Disabled widget, everything else | Arrow |

  The gui reports a wish every frame; it is applied at `FLIP` — only when
  it changes. **The program's shape takes precedence:**
  `MOUSE_CURSOR(form$)` stays until the program names another one, and
  `MOUSE_CURSOR("auto")` hands the pointer back to the gui (until
  2026-09-27 both set raylib directly, and anyone wanting to get rid of
  their shape had to guess what the gui would have shown there).
  `GUI_SET_CURSOR(wdg, form$)` gives a widget a shape of its own — for
  canvases on which the program itself offers something; besides the
  standard shapes, `warten`, `arbeitet`, `hilfe`, `kopieren`, `stift`,
  `pipette` and pointers from `MOUSE_CURSOR_NEW(bild, bx, by)` work too.
  `MOUSE_CURSOR_GET$()` tells which shape applies. `GUI_CURSORS(FALSE)`
  switches the gui's shapes off.

In the `.dhform`: `vertical`, `indeterminate`, `mode`, `draggable`,
`drop_target`, `panel` (with `kinder` as indices and `scroll`). Tree icons
and colours are not stored in the file (icons are texture handles).
Example: [`examples/195_gui_feinschliff.dh`](../../examples/195_gui_feinschliff.dh).

## Events

Seven callbacks per widget, all via FUNCREF (without parameters):

| Command | When |
|---|---|
| `GUI_ON_CLICK(w, fn)` | clicked |
| `GUI_ON_CHANGE(w, fn)` | value/text changed |
| `GUI_ON_HOVER(w, fn)` | mouse **enters** the widget |
| `GUI_ON_LEAVE(w, fn)` | mouse leaves it |
| `GUI_ON_FOCUS(w, fn)` | receives the input |
| `GUI_ON_BLUR(w, fn)` | loses it — the point at which you validate an input |
| `GUI_ON_ENTER(tf, fn)` | Enter in the text input (only `GUI_TEXTINPUT`) |

The last four are **edges**: they fire on the transition, not in every
frame while the state lasts. They are triggered in `GUI_UPDATE`; the
handler is called afterwards, so that it cannot rebuild the interface in the
middle of the state update.

**Every enabled widget can receive focus** (since 2026-08-30 — before, only
text input, text area and number field). `on_focus`/`on_blur` therefore
also fire on button, checkbox, dropdown and tree as soon as `TAB` or a click
lands there. Pure decoration (label, panel, separator, group, toolbar,
progress, image, canvas) still receives no focus and never fires the two
edges.

All seven survive `GUI_SAVE`/`GUI_LOAD` and `GUI_TO_JSON`/`GUI_FROM_JSON`
— that is the way a form built in the form designer gets its handlers.

## Operation without a mouse

`TAB` / `SHIFT+TAB` moves through **all enabled widgets** of the active
window, in creation order and only over visible, enabled ones. Pure
decoration is skipped — otherwise you would have to tab your way through
labels. The focused widget gets a **ring in the accent colour**; without
visible focus, navigation would be worthless.

| Key | Effect |
|---|---|
| `TAB` / `SHIFT+TAB` | next / previous widget |
| `LEERTASTE`, `ENTER` | trigger a button, toggle a checkbox/switch, select a radio, open/close a dropdown, expand/collapse a tree node |
| `←` `→` `↑` `↓` | adjust a slider/knob (one twentieth of the range per press), move a splitter (8 px), move the selection in list/dropdown/tree |
| `POS1` / `ENDE` | slider/knob to minimum / maximum |
| `→` / `←` in the tree | expand or go to the child / collapse or go to the parent node |
| `ESC` | close an open dropdown |
| `F10` or `ALT` alone | open the menu bar (first entry highlighted); `←`/`→` switch menus or open a submenu, `↑`/`↓` move, `POS1`/`ENDE` jump, a letter jumps to the matching entry, `ENTER` selects, `ESC` closes one level — the same in an open context menu |

The **tab order** is the order of creation. Anyone who wants it different —
say, because a field was added later — sets `GUI_SET_TAB_INDEX(wdg, n)`:
widgets with an index above 0 come first, in ascending order, the rest
follows as before. A widget that receives the focus by keyboard shows its
**tooltip** below itself after a short pause — otherwise the help text was
available only to the mouse.

Which widget currently has its turn is returned by `GUI_FOCUSED()` (`-1` =
none) — intended for a status line or a help text for the active field. The
focus can be set with `GUI_FOCUS(wdg)`.

```basic
' help text for the widget under the focus
DIM aktiv AS GUI_WIDGET
aktiv = GUI_FOCUSED()
IF aktiv = feldName THEN TEXT(10, 220, "Vor- und Nachname eingeben")
IF aktiv = knopfOk THEN TEXT(10, 220, "Uebernimmt die Aenderung")
```

> **Pitfall:** If your program uses `ESC` to quit, first check whether a
> dropdown is open or a table cell is being edited
> (`GUI_TABLE_EDITING_ROW < 0`) — otherwise the key quits the program while
> the user only wanted to close a popup.

## Accessibility

A screen reader (on Windows: NVDA, JAWS, Narrator), the magnifier and
speech control query a window through the system's interface — on Windows
*UI Automation*. raylib paints pixels and reports no controls to the
system; until September 2026 every Drachenhauch window was, for these
programs, a title and nothing else (measured: zero descendants in the UIA
tree, see
[entwurf-barrierefreiheit.md](../entwurf-barrierefreiheit.md) (German)).

Since then **the `gui` module reports its tree by itself**, through the
Rust library [AccessKit](https://accesskit.dev/): every window, every
widget with role, label, value, position and state, the menu bar with its
entries and shortcuts, the tabs, list entries, table rows and tree nodes.
And the other direction works too: when an assistive program clicks a
button, `GUI_CLICKED` and `GUI_ON_CLICK` fire as for a mouse click; when it
sets the focus or a text, the same paths are used as for the keyboard.
**A program does not have to do anything for this.** The tree is only built
when an assistive program asks for it — a game without a screen reader pays
nothing.

What the program can contribute:

| Command | Description |
|---|---|
| `GUI_ANNOUNCE(text$[, dringend])` | hand a sentence to the user's screen reader — it speaks it with its voice (polite: after the current sentence; urgent: immediately). Without an assistive program nothing happens. For everything that would otherwise only appear in a status line: "Invoice saved", "3 rows deleted" |
| `GUI_SCREENREADER()` | `TRUE` as soon as an assistive program has requested the tree — for example to announce more or to switch off motion |
| `GUI_SET_TAB_INDEX(wdg, index)` | set the tab order: widgets with an index above 0 first, ascending; 0 (default) = order of creation |

**Labels.** An input field has no text of its own; the screen reader takes
the **label to its left** (same row) or **directly above it** (same left
edge) — the way the form designer places them. If there is none, the
tooltip serves as the name. An error message from form validation is read
out as a description. So anyone building a field without a neighbouring
label should at least give it a `GUI_TOOLTIP`.

**What the toolkit brings along anyway:** operation without a mouse
(above), `GUI_SCALE` for large text, the theme `contrast` (white on black,
yellow frames, contrast 21:1) via `GUI_THEME_PRESET("contrast")`; the
default `dark` and the light theme are above the WCAG value of 4.5:1 with
all their texts.

**What the program must do — and only it can:** never convey information
through colour alone (that is why the red field also has a message), offer
every mouse gesture as a key as well (the `gui` does, the canvas does not),
never use sound as the only feedback, make motion switchable. And a canvas
is an image to the reader: what happens on it, you tell them with
`GUI_ANNOUNCE` — or with `SPEAK` (see
[module-audio.md](module-audio.md)), which uses the system voice without a
reader and the reader's voice with one; that also works in a program
without a gui.

**Input methods (IME):** anyone writing Japanese, Chinese or Korean through
an input method sees the **conversion in the field** — the unfinished
preview stands underlined at the cursor of the text input or text area that
has the focus, the system's candidate list next to it; on confirmation the
result is inserted, through the same limits as typed text (maximum length,
number filter, undo, `on_change`). During conversion, arrow keys and Enter
belong to the input method. This only applies while a text input has the
focus; without one (a game with `INKEY$`) the input method works as before
through its own window. Windows; built according to the Windows
documentation and not measured without an installed input method — anyone
who has one is the acceptance test. What the field can display is described
in
[builtins-grafik.md](builtins-grafik.md#umlauts-the-euro-sign-and-foreign-scripts)
— Kanji, Hangul and emoji are loaded on demand.

**Limits (as of 2026-09-06):** This has been measured on Windows. The
adapters for macOS (VoiceOver, NSAccessibility) and Linux (Orca, AT-SPI via
D-Bus) are hooked in and are compiled in the CI on the real systems —
**nothing has run there yet**, because there is neither a Mac nor Linux
here; under Wayland a program does not know the position of its window, so
the tree lacks it. Anyone who has VoiceOver or Orca is the acceptance test.
The cursor in a text input is not yet reported character by character (the
content is). The `ui` module (immediate mode) and the web build report
nothing.

```basic
IMPORT "gui"
SCREEN(480, 320, "Ansage", 1)
DIM w AS GUI_WINDOW : w = GUI_WINDOW("Formular", 10, 10, 440, 280)
DIM name AS GUI_WIDGET : name = GUI_LABEL(w, "Name", 10, 12)
DIM feld AS GUI_WIDGET : feld = GUI_TEXTINPUT(w, 70, 8, 220, 26)
DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(w, "Speichern", 10, 50, 120, 32)
WHILE NOT QUITREQUESTED()
    GUI_UPDATE()
    IF GUI_CLICKED(ok) THEN GUI_ANNOUNCE("Gespeichert: " + GUI_TEXT(feld))
    CLS(0) : GUI_DRAW() : FLIP()
WEND
```

## Operating the text area like a list

Since 2026-09-23 the text area can do what you expect from any editor:

* **Double-click** selects the word (letters, digits, `_`, `$` —
  Drachenhauch names like `name$` stay whole), **triple-click** the line
  including its line break. Dragging afterwards does not shrink the
  selection again.
* **Right-click** puts the cursor at that spot, so that a context menu
  ("Cut", "Go to definition") refers to it. If it hits the existing
  selection, the selection stays. The line-number column still belongs to
  the breakpoint (`GUI_TEXTAREA_GUTTER_CLICKED`).
* **Scroll bar** on the right in the inner padding as soon as the text does
  not fit: drag the thumb, a click in the track jumps there. It lies over no
  character, and the cursor does not pull the view back afterwards.
* The **mouse wheel** scrolls smoothly (three lines per step, see
  [Transitions](#transitions)).

The single-line **text input** (`GUI_TEXTINPUT`) can do the same with the
mouse: double-click selects the word, triple-click everything, a
right-click sets the cursor and leaves a selection it hits in place. In the
password field even the double-click selects everything — word boundaries
would give away where a space is.

## Bold, italic, underlined in the text area

A text area can shape its characters individually — for notes, letters,
descriptions. Headings and lists are not part of it; anyone who wants those
displays the text with `GUI_RICHTEXT`.

| Command | Effect |
|---|---|
| `GUI_TEXTAREA_SET(ta, "formatiert", 1)` | switches formatting on (0 = off, the formatting is dropped) |
| `GUI_TEXTAREA_STYLE(ta, stil$[, an])` | apply a style to the selection (`an` = TRUE, the default) or remove it; without a selection it applies to the next thing typed |
| `GUI_TEXTAREA_GET_STYLE$(ta)` | what applies to the whole selection, without a selection the style of the next thing typed (`"normal"`, `"fett+kursiv"` ...) |
| `GUI_TEXTAREA_MARKDOWN$(ta)` | the text including formatting as Markdown |
| `GUI_TEXTAREA_SET_MARKDOWN(ta, md$)` | set the text from Markdown (switches formatting on) |

The style words are those of `TEXT_STYLE`: `fett` (bold), `kursiv`
(italic), `unterstrichen` (underlined), `durchgestrichen`
(strikethrough), joined with `+`.

With the keyboard: **Ctrl+B** bold, **Ctrl+I** italic, **Ctrl+U**
underlined — on the selection (if all characters already carry the style,
it is removed), without a selection for what you type next, until the
cursor wanders elsewhere. Otherwise typed text continues the style of the
character before it. A formatting change is a step of its own for
Ctrl+Z/Ctrl+Y. If a menu has the same key as a shortcut, the menu wins.

The Markdown: `**fett**`, `*kursiv*`, `***beides***`, `~~durch~~`,
`<u>unter</u>`; a backslash takes the next character literally (`\*`).
`GUI_TEXT` still returns the plain text, `GUI_SET_TEXT` sets it without
formatting. In the `.dhform` a formatted text area is stored with
`formatiert` and `markdown`.

```basic
GUI_TEXTAREA_SET(notiz, "formatiert", 1)
GUI_TEXTAREA_SET_MARKDOWN(notiz, "Ein **wichtiger** Punkt")
' ... the user writes and formats ...
WRITEALL("notiz.md", GUI_TEXTAREA_MARKDOWN$(notiz))
```

The formatting is drawn **imitated** on the field's font (a second stroke
for bold, sheared for italic) — a real bold face would be wider, and
cursor, selection and click measure against the unshaped text.

## The `TEXTAREA` as a code field

With four settings and some colouring, a multi-line text field becomes a
usable code field.

| Function | Effect |
|---|---|
| `GUI_TEXTAREA_SET(ta, schluessel$, wert)` | `zeilennummern` (line numbers), `aktive_zeile` (active line), `tab_fuegt_ein` (Tab inserts), `tabbreite` (tab width), `umbruch` (wrap), `auto_einzug` (auto-indent), `einzugslinien` (a fine line per indentation level, beneath the text; the first level stays free, it would lie on the left edge), `kopfzeilen` (sticky block headers, at most n: if the header line of a block from `GUI_TEXTAREA_FOLDABLE` has scrolled out at the top, it stays pinned; a click jumps there, 0 = off), `hinweise_klickbar` (hints accept clicks, see `GUI_TEXTAREA_HINT_CLICKED`) |
| `GUI_TEXTAREA_SPANS(ta, starts, laengen, farben)` | draw characters `start … start+laenge` in `farbe` |
| `SYNTAX_SPANS(quelltext$)` → (starts, laengen, arten) | split Drachenhauch source code |
| `GUI_TEXTAREA_VIEW(ta)` → (erste_zeile, zeilen, start_zeichen, laenge_zeichen) | which part is currently visible? |
| `GUI_TEXTAREA_CURSOR(ta)` → (zeile, spalte) | where the cursor is (from 1, in characters) |
| `GUI_TEXTAREA_CARET_XY(ta)` → (x, y, hoehe) | where the cursor is on the screen, in logical points like `GUI_WINDOW_SET_BOUNDS` (also without focus) -- for a suggestion list at the cursor; (-1, -1, 0) if the field is not visible |
| `GUI_TEXTAREA_POS_AT(ta, x, y)` → (zeile, spalte) | which character lies under the screen point (from 1; the same calculation as the click). (0, 0) next to the text: outside, in the line-number column, below the last line, behind the end of the line -- for tooltips on hover and Ctrl+click |
| `GUI_TEXTAREA_GOTO(ta, zeile[, spalte])` | set the cursor, clear the selection, scroll the view so that the line is in the middle |
| `GUI_TEXTAREA_HINTS(ta, zeilen, texte [, farbe])` | put dimmed text behind the end of lines (from 1), for example the values of the variables when the debugger stops; follows its line like a marker, empty lists clear, `GUI_SET_TEXT` too. `farbe` may be an array, one per hint (`-1` = dimmed) |
| `GUI_TEXTAREA_SCROLL(ta, zeile)` | scroll the view so that the line is at the top -- without moving the cursor, like the mouse wheel (for an overview map) |
| `GUI_TEXTAREA_SELECT(ta, z1, s1, z2, s2)` | select a range, cursor at the end |
| `GUI_TEXTAREA_SELECTION$(ta)` → STRING | the selected text |
| `GUI_TEXTAREA_SELECTION_RANGE(ta)` → (z1, s1, z2, s2) | start and end of the selection (from 1, ordered); without a selection the cursor is at both ends — this way an editor knows WHICH lines to indent or comment out |
| `GUI_TEXTAREA_INSERT(ta, text$)` | replaces the selection or inserts at the cursor — an undo step of its own, `GUI_ON_CHANGE` fires as when typing |
| `GUI_TEXTAREA_MARKS(ta, zeilen, farben)` | markers per line: a dot in the line-number column and a tint of colour over the line — breakpoints, the stopped line, error lines. Replaces all previous ones, two empty arrays clear; when the field is edited, they move with their line |
| `GUI_TEXTAREA_LINE_COLORS(ta, zeilen, farben)` | a colour band behind whole lines (lines from 1, colours with opacity in the top byte, without it a quarter) -- for example a profile as heat in the code; separate from the markers, moves with its line, empty lists clear |
| `GUI_TEXTAREA_LINE_COLORS_GET(ta)` | the lines of the line colours as they stand now (in the order they were set) |
| `GUI_TEXTAREA_MARKS_GET(ta)` | the lines of the markers as they stand now (ARRAY OF INTEGER, in the order they were set) -- this is how a program learns where its breakpoint has slipped to while typing |
| `GUI_TEXTAREA_BACKGROUND(ta, bild, deckkraft = 24)` | lay an image faintly behind the text: fitted into the inner area, centred, does not scroll along. Opacity 0..255, `bild` < 0 removes it -- this is how the IDE puts the logotype behind the code |
| `GUI_TEXTAREA_SQUIGGLES(ta, starts, laengen, farben)` | wavy lines under character ranges (start from 0, length in characters, 0 = a short piece at that spot) -- errors and warnings where they are. Replaces all previous ones, empty arrays clear; a change to the text removes them like the colour spans |
| `GUI_TEXTAREA_SWATCHES(ta, starts, laengen, farben)` | colour swatches: a small square behind the piece. Which spot in the text MEANS a colour only the caller knows -- the IDE looks for `&H` literals |
| `GUI_TEXTAREA_GUTTER_CLICKED(ta [, taste])` → INTEGER | which line (from 1) was clicked in the line-number column in this frame, 0 = none; `taste` 0 = left (default), 1 = right. Applies for one frame like `GUI_CLICKED`; a left click there does NOT set the cursor, the fold arrow still toggles. For breakpoints and bookmarks |
| `GUI_TEXTAREA_HINT_CLICKED(ta)` → INTEGER | which hint (`GUI_TEXTAREA_HINTS`) was clicked in this frame: its line from 1, 0 = none; only with `GUI_TEXTAREA_SET(ta, "hinweise_klickbar", 1)` (then a hand as the pointer, the cursor stays put); applies for one frame |
| `GUI_TEXTAREA_SWATCH_CLICKED(ta)` → INTEGER | which colour swatch was clicked in this frame (-1 = none); applies for one frame like `GUI_CLICKED`, the cursor stays put |
| `GUI_TEXTAREA_INDENT_WORDS(ta, anfang, ende, aus)` | three word lists for indentation: the line starts with it, it ends with it, or the word alone in a line moves it back (while typing; a pasted block with several lines stays as it is) |
| `GUI_TABS_CLOSABLE(win, an)` | every tab gets a cross. Nothing is closed — the runtime only reports which one was hit |
| `GUI_TAB_CLOSED(win)` | which tab should be closed in this frame (-1 = none). Hit by the cross or the middle mouse button |
| `GUI_TEXTAREA_TAB_HIT(ta)` | was Tab pressed in this frame? Only with `GUI_TEXTAREA_SET(ta, "tab_meldet", 1)` — then the field does not indent and does not look for abbreviations, it only reports |
| `GUI_TEXTAREA_CLOSE_WORDS(ta, oeffner, schluesse)` | the scaffold that grows along: if the line opens a block, the line break puts the closing line right underneath |
| `GUI_TEXTAREA_SHARE(ansicht, von)` | a second view of the same text: `ansicht` shows what is in `von` (both in the same window), and what is changed in one of the two is in the other after `GUI_UPDATE`. Each keeps its cursor, view and folding; a cursor behind a change moves along. The history lies with the owner — Ctrl+Z in the view undoes there. `-1` removes the link |
| `GUI_TEXTAREA_PAIRS(ta, paare$)` | close brackets while typing: every two characters are a pair (`"()[]{}"` plus two quotation marks), empty = off. The opening one puts the closing one right behind it, before an identical closing one you only step over it, a selection is wrapped, Backspace between an empty pair takes both. Quotation marks do not pair behind a word and not in an open string; nothing is paired before a word. Pasting from the clipboard bypasses this |
| `GUI_TEXTAREA_ABBREV(ta, woerter)` | abbreviations: if one of these words stands to the left of the cursor, Tab reports it instead of indenting. What goes in its place is set by the caller — the runtime knows no snippets |
| `GUI_TEXTAREA_ABBREV_HIT(ta)` → INTEGER | which abbreviation Tab hit in this frame (-1 = none); applies for one frame like `GUI_CLICKED` |
| `GUI_TEXTAREA_SELECT_COLUMNS(ta, z1, s1, z2, s2)` → INTEGER | column selection: a RECTANGLE instead of a run. Each line gets its own cursor with its own selection, typing goes into all of them at once; a line that is too short gets its cursor at the end instead of dropping out. With the mouse: **hold Alt and drag**. Returns the number of cursors |
| `GUI_TEXTAREA_ADD_CARET(ta, zeile[, spalte])` → INTEGER | set an additional cursor; returns how many there are afterwards. Two at the same spot become one |
| `GUI_TEXTAREA_CARETS(ta)` → INTEGER | how many cursors the field currently has (at least 1) |
| `GUI_TEXTAREA_CLEAR_CARETS(ta)` | back to a single cursor |
| `GUI_TEXTAREA_FOLDABLE(ta, von_zeilen, bis_zeilen)` | which blocks can be folded: two arrays of equal length with header line and last line. What a block is only the program knows — the runtime knows no language here |
| `GUI_TEXTAREA_FOLD(ta, zeile[, an])` → BOOLEAN | collapse or expand the innermost foldable block around `zeile`; without `an`, toggle. Returns whether it is collapsed afterwards |
| `GUI_TEXTAREA_FOLD_ALL(ta[, zu])` → INTEGER | collapse everything (only the outer blocks) or expand everything; returns the number of collapsed blocks |
| `GUI_TEXTAREA_FOLDED(ta, zeile)` → BOOLEAN | is the line hidden in a collapsed block? |
| `GUI_TEXTAREA_FOLDS(ta)` → ARRAY OF INTEGER | the header lines of the collapsed blocks, ascending |
| `GUI_TEXTAREA_FIND(ta, text$[, ab_zeile[, ab_spalte[, genau]]])` → (zeile, spalte) | next match from the given spot, `(-1, -1)` if there is none; without `genau` regardless of upper/lower case. No wrap-around — at the end search again from `1, 1` |

```basic
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(win, 12, 12, 650, 300)
GUI_TEXTAREA_SET(ta, "zeilennummern", 1)
GUI_TEXTAREA_SET(ta, "aktive_zeile", 1)
GUI_TEXTAREA_SET(ta, "tab_fuegt_ein", 1)

' colouring -- anew after every change
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

**Why separate?** `SYNTAX_SPANS` says *what* a piece of text is —
`GUI_TEXTAREA_SPANS` says which *colour* it gets. Which colour a comment has
is a question of your theme, not of the language. And because
`GUI_TEXTAREA_SPANS` only takes numbers, you can also use it to colour
something entirely different: search matches, error locations, a diff.

The kinds from `SYNTAX_SPANS`: `kommentar` (comment), `text` (string),
`zahl` (number), `schluessel` (keyword), `name`, `operator`. Whitespace gets
no span and stays in the base colour.

> **The highlighter is not a lexer.** It sees half-typed text (`"abc`
> without a closing quotation mark, `IF x THE`) and still displays it
> instead of giving up. For it, an open string ends at the end of the line
> — otherwise a single quotation mark would colour the rest of the file. It
> shares its word list with the real lexer, so a new keyword is immediately
> known here too.

**`tab_fuegt_ein` is OFF by default.** Otherwise you could no longer get out
of the text field in a form — `TAB` is the move-on key there (see
[Operation without a mouse](#operation-without-a-mouse)). If the switch is on,
Tab indents to the **next column**, not stubbornly by `tabbreite`
characters; typed in the middle of a line the indentation would otherwise
end up crooked.

> **`GUI_SET_TEXT` clears the spans.** They belonged to the old text;
> leaving them in place would colour the new text by the positions of the
> old. So colour again after every `GUI_SET_TEXT`. While **typing** the
> colouring naturally lags behind until your program renews it — spans that
> reach beyond the end of the text are cut off when drawing and do no harm.

Complete example: [`examples/184_codefeld.dh`](../../examples/184_codefeld.dh).

### Cursor, selection, search — the commands of a development environment

With the six commands above a program builds search/replace, "Go to line",
jumping to an error line and a completion that replaces the typed beginning
of a word. The IDE written in Drachenhauch (`ide/ide.dh`) uses exactly
these:

```basic
' replace: all matches from the start
GUI_TEXTAREA_GOTO(ta, 1, 1)
DIM z AS INTEGER
DIM s AS INTEGER
(z, s) = GUI_TEXTAREA_FIND(ta, "alt", 1, 1)
WHILE z >= 0
    GUI_TEXTAREA_SELECT(ta, z, s, z, s + LEN("alt"))
    GUI_TEXTAREA_INSERT(ta, "neu")
    (z, s) = GUI_TEXTAREA_FIND(ta, "alt", z, s + LEN("neu"))
WEND
```

### Large files: colour only what you see

Splitting the **whole** text anew on every keystroke costs in proportion to
the file size. Measured on a workstation, per keystroke:

| Lines | Spans | whole file | only the visible part |
|---|---|---|---|
| 3,000 | 16,500 | ~26 ms | **0.2 ms** |
| 10,000 | 55,000 | ~88 ms | **0.8 ms** |
| 30,000 | 165,000 | ~272 ms | **2.1 ms** |

Up to about 3,000 lines the simple way (whole file) is perfectly fine — a
keystroke then costs about one frame. Beyond that, typing becomes sluggish.

The reason is not the drawing (that costs a constant ~0.4 ms, because only
the visible lines are drawn) and not `SYNTAX_SPANS` either (2.4 ms at
3,000 lines). It is the **loop in your own program** that maps every kind to
a colour — across 16,500 spans. A `MAP` instead of four `IF` comparisons
alone halves it.

You get rid of it entirely with `GUI_TEXTAREA_VIEW`:

```basic
DIM z0 AS INTEGER
DIM anz AS INTEGER
DIM von AS INTEGER
DIM laenge AS INTEGER
(z0, anz, von, laenge) = GUI_TEXTAREA_VIEW(ta)

DIM teil AS STRING
teil = MID$(GUI_TEXT(ta), von, laenge)      ' only the visible part
DIM st AS ARRAY OF INTEGER
DIM ln AS ARRAY OF INTEGER
DIM ar AS ARRAY OF STRING
(st, ln, ar) = SYNTAX_SPANS(teil)

DIM fb[LEN(st)] AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO LEN(st) - 1
    st[i] = st[i] + von                     ' visible part -> whole text
    fb[i] = MAPGETOR(tabelle, ar[i], &HD8E4F0)
NEXT
GUI_TEXTAREA_SPANS(ta, st, ln, fb)
```

Then the cost depends only on the **window size**, not on the file. For
this you have to recolour not only when typing but also when **scrolling** —
`z0` tells you whether the visible part has moved.

> **Why this is not only faster but also correct:** comments and strings
> end at the **line** in Drachenhauch. A visible part cut at line boundaries
> therefore cannot begin in the middle of a construct. In a language with
> block comments exactly that would be the trap — there you would have to
> know in which state the first visible line begins.

The remaining cost (0.2 → 2.1 ms across the three sizes) is the copying of
the text by `GUI_TEXT` and `MID$`. That still grows with the file, but is
cheap enough not to be noticed.

## Colour picker and date picker

Two widget kinds that an interface often needs and that you would otherwise
have to rebuild by hand.

| Function | Effect |
|---|---|
| `GUI_COLORPICKER(win, x, y, w, h)` → GUI_WIDGET | saturation/brightness field plus hue strip |
| `GUI_PICKED_COLOR(picker)` → INTEGER | chosen colour (`0xRRGGBB`) |
| `GUI_SET_PICKED_COLOR(picker, farbe)` | set the colour |
| `GUI_DATEPICKER(win, x, y, w, h)` → GUI_WIDGET | month grid with paging arrows |
| `GUI_DATE(picker)` → STRING | chosen date as `JJJJ-MM-TT` |
| `GUI_SET_DATE(picker, datum$)` | set the date |
| `GUI_COLORPICKER_SET(picker, schluessel$, wert)` | `alpha` (0/1) — show an opacity strip |
| `GUI_DATEPICKER_SET(picker, schluessel$, wert)` | `wochenbeginn` (start of week: 0 = Monday … 6 = Sunday) |
| `GUI_DATE_RANGE(picker, von$, bis$)` | allowed range (empty text = no limit) |

Both report changes via `GUI_ON_CHANGE`.

```basic
DIM waehler AS GUI_WIDGET
waehler = GUI_COLORPICKER(win, 16, 36, 320, 230)
GUI_SET_PICKED_COLOR(waehler, &H3FA9F5)

DIM kalender AS GUI_WIDGET
kalender = GUI_DATEPICKER(win, 366, 36, 410, 286)
GUI_SET_DATE(kalender, "2026-08-30")

' ... per frame:
BOX(20, 300, 200, 340, GUI_PICKED_COLOR(waehler))
TEXT(20, 350, "Termin am " + GUI_DATE(kalender), WEISS)
```

**The date format is `JJJJ-MM-TT`** (YYYY-MM-DD) — the same one `DATE$()`
returns. Two date formats in the same system would be a pitfall. A new
picker shows **today**; an empty calendar would be an unnecessary question
to the user. Nonsense is rejected, including 29 February in a year that is
not a leap year.

**Without a mouse:**

| Key | Colour picker | Date picker |
|---|---|---|
| `←` `→` | saturation | one day |
| `↑` `↓` | brightness | one week |
| `BILD ↑` `BILD ↓` | hue | one month |
| `POS1` `ENDE` | opacity (only with `alpha`) | — |
| plus `UMSCHALT` | finer steps | one **year** instead of one month |

> **The hue is carried along, not recalculated.** For black it is
> undefined (every hue gives black), for grey the saturation is. A picker
> that keeps only the RGB colour loses it exactly there — the pointer jumps
> to the left when you pull down to black, and when brightening, red comes
> back instead of the colour you had chosen. That is why the widget stores
> HSV; `GUI_PICKED_COLOR` converts only when asked.

> **When paging, the day is clamped.** 31 January plus one month is the last
> day of February, not 3 March — otherwise the picker would slowly run away
> forward while paging through.

A click on a day of the neighbouring month (the dim numbers at the edge)
pages there. Colour and date survive `GUI_SAVE`/`GUI_LOAD` — in the
`.dhform` they are stored as `"#RRGGBB"` and `"JJJJ-MM-TT"`, i.e. readable.

### Opacity

`GUI_COLORPICKER_SET(picker, "alpha", 1)` shows a second strip.
`GUI_PICKED_COLOR` then returns `0xAARRGGBB` instead of `0xRRGGBB`:

```basic
GUI_COLORPICKER_SET(waehler, "alpha", 1)
GUI_SET_PICKED_COLOR(waehler, &HC03FA9F5)     ' half-transparent blue
```

Without the switch it stays at six digits — programs that already use the
picker get exactly what they expect, unchanged.

> **Opacity goes from 1 to 255, not from 0.** The runtime reads a top byte
> of `0` as **opaque** — that way the old 24-bit colours (`&Hrrggbb`) stay
> opaque. For "practically invisible", `1` is the smallest value; whatever
> should be gone entirely you simply do not draw.

The strip lies on a **checkerboard**: without it you could not tell from
the gradient where it becomes transparent.

### Limits and start of the week

```basic
' no appointments in the past
GUI_DATE_RANGE(kalender, DATE$(), "")          ' empty = no upper limit
GUI_DATEPICKER_SET(kalender, "wochenbeginn", 6) ' week starts on Sunday
```

Disabled days are drawn **dimmer** than those of the neighbouring month
(the one is reachable, the other is not) and accept neither click nor key.
A date already set outside the range is pulled in immediately when the
limits are set — otherwise the field would hold a value the picker itself
no longer allows.

In the header, **four** areas page: `‹‹` one year back, `‹` one month back,
`›` and `››` correspondingly forward. With the keyboard, Shift+Page Up/Page
Down (`UMSCHALT`+`BILD ↑`/`BILD ↓`) is the year jump — without it you would
need about five hundred clicks to reach 1985.

### Colour as text

Two core built-ins (no `IMPORT` needed) that did not exist before:

| Function | Effect |
|---|---|
| `COLOR_HEX$(farbe)` → STRING | `"#RRGGBB"`, with opacity `"#AARRGGBB"` |
| `COLOR_FROM_HEX(text$)` → INTEGER | `#RGB`, `#RRGGBB`, `#AARRGGBB` — with or without `#`, also `0x`/`&H` |

Without them **a program cannot turn hex text into a colour at all**:
`VAL("&HFF8800")` returns `0`, and `&H` literals only exist in the source
code, not at run time. So a colour could neither be read from a settings
file nor taken over from typed input.

The short form doubles every digit as on the web: `#F80` = `#FF8800`. If
the opacity is `0` (i.e. opaque), `COLOR_HEX$` leaves it out — otherwise
every ordinary colour would be preceded by a pointless `00`.

Example: [`examples/186_farbe_und_datum.dh`](../../examples/186_farbe_und_datum.dh).

## Custom font

The built-in raylib font does not have to stay. Two lines are enough, and it
applies to **everything** — window titles, labels, buttons, input fields,
code field, line numbers:

```basic
DIM mono AS INTEGER
mono = LOADFONT("C:/Windows/Fonts/consola.ttf", 18)
SETFONT(mono)                  ' from here on everything draws with it
```

`SETFONT` sets the **active** font, and every widget without a font of its
own follows it. For a code field, a real monospace font makes a noticeable
difference.

More finely controllable:

| Function | Effect |
|---|---|
| `SETFONT(font)` | the active font — applies to the whole interface |
| `GUI_SET_FONT(wdg, font)` | a font of its own for this widget only |
| `GUI_SET_FONT_SIZE(wdg, px)` | a size of its own for this widget only |
| `GUI_SET_FONT_STYLE(wdg, stil$)` | bold, italic, underlined, strikethrough for this widget only (also stored in the `.dhform` as `font_style`) |
| `GUI_STYLE_SET(name$, "font", font)` + `GUI_APPLY_STYLE(wdg, name$)` | one font for a whole group |

For a **pixel font** from a PNG there is `LOADFONT_IMAGE(bild, trennfarbe,
erstes_zeichen)`. It deliberately stays unfiltered (nearest), so that a pixel
font stays pixelated — unlike `LOADFONT` (TTF), which smooths.

> **The size is part of the handle, and `SETFONT` takes it over.**
> `LOADFONT(pfad, 18)` builds the character set *for 18 px*; `SETFONT`
> thereby sets not only the typeface but also the active text size to 18.
> So don't be surprised if after `SETFONT` everything is larger or smaller
> than before. The same font sharp in two sizes? Load it twice — otherwise a
> TTF becomes soft when scaled up a lot. `GUI_SET_FONT_SIZE` only changes how
> large it is drawn, not what the character set was built for.

> **With `GUI_SCALE`** (below) the font scales too — so you load it in the
> logical size and get it drawn larger on a HiDPI screen.

## Scale (high-resolution screens)

All sizes in the `gui` module are fixed pixels. On a 4K screen with 200 %
scaling, an interface built for 1920×1080 thus becomes half as large as
intended — readable, but tiny. `GUI_SCALE` solves this:

```basic
IMPORT "gui"
GUI_SCALE(WINDOW_DPI_X())      ' 1.0 normal, 2.0 on HiDPI/Retina
SCREEN(1600, 1000, "Mein Werkzeug", 1)
' ... from here on as always, in the usual numbers
DIM w AS GUI_WINDOW
w = GUI_WINDOW("Formular", 10, 10, 400, 300)
```

The factor (0.5 to 4.0) multiplies **every length that goes into the GUI**:
window and widget geometry, title bar and row heights, inner padding, column
widths and the font size. So you convert nothing — your layout stays in the
numbers in which you designed it.

**Towards the outside everything stays logical.** `GUI_GET_X`,
`GUI_WINDOW_GET_W`, `GUI_TABLE_GET("zeilenhoehe")` and `GUI_TO_JSON` return
the numbers you put in; `GUI_SET_BOUNDS` accepts them the same way. A saved
`.dhform` therefore still describes the **layout**, not the display —
otherwise a form would grow by the factor every time it is opened and
saved.

**The one exception is `GUI_HIT_TEST(x, y)`** — it answers a question about
the screen and therefore takes screen pixels, as `MOUSEX()` returns them.
Anyone who wants to place their own drawing commands (`TEXT`, `BOX`, …) next
to a widget converts with `GUI_SCALE_GET()`:

```basic
TEXT(GUI_GET_X(feld) * GUI_SCALE_GET(), y, "Pflichtfeld", ROT)
```

> **`GUI_SCALE` must come before the first window.** After that it is an
> error. Converting widgets that have already been created would only be
> approximate — every round would bring new rounding errors, and a
> half-scaled interface would be worse than a clear refusal. Anyone who
> wants to change the scale at run time (settings dialog) rebuilds the
> interface after `GUI_RESET`.



## Light sweep

`GUI_WINDOW_GLOW(win)` lets a light sweep pass over a window -- a soft, warm
glow with a bright line in the middle, additive over the widgets (but below
opened menus and dropdowns), and the frame lights up where the sweep
currently is. It starts and stops smoothly and fades in and out at the ends,
so that it does not pop up at the edge.

```basic
GUI_WINDOW_GLOW(win)                          ' once, warm, 1.2 s
GUI_WINDOW_GLOW(win, RGB(120, 200, 255), 800) ' cool and faster
GUI_WINDOW_GLOW_SET(win, "richtung", "oben")  ' from bottom to top
GUI_WINDOW_GLOW_SET(win, "pause", 3000)       ' again every three seconds
GUI_WINDOW_GLOW(win)
```

Three **kinds** (`GUI_WINDOW_GLOW_SET(win, "art", ...)`):

| Kind | Effect |
|---|---|
| `streif` (default) | a band across the window, as described above |
| `lampe` | like a torch: the window darkens slightly, a warm cone of light with a soft edge passes over it in a flat arc, the text inside becomes brighter |
| `rahmen` | grazing light: a slanted edge of light passes flatly over it, and every edge it crosses -- window frame, frames of the widgets -- flashes briefly; the content only gets a faint shimmer, nothing is darkened |

The soft edge of lamp and frame is a drawing command of the runtime in its
own right (a fan of triangles with a colour per corner), hence without
visible steps.

Colour `-1` is the theme's accent colour. `breite` (width) is a fraction of
the window width (or height for `oben`/`unten`, default 0.35), `staerke`
(strength) 0..1 (default 0.55). Time runs with `DELTA()` -- without a real
clock a fixed 1/60 s --, so a frame always shows the same spot. A second call
of `GUI_WINDOW_GLOW` starts from the beginning. The IDE shows it at startup
and when a program starts.
