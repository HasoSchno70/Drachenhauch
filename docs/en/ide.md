# The IDE in Drachenhauch (`ide/ide.dh`)

A development environment written in the language it serves.
It is path C from [Design: phasing out Python](../entwurf-python-abbau.md) (German): the
Qt IDE (32,000 lines of Python) is to be replaced by a Drachenhauch program.
Until it catches up, the Qt IDE stays in the installer.

```
dhrt run ide/ide.dh                 # leer starten
dhrt run ide/ide.dh -- spiel.dh     # mit einer Datei
```

The project tree on the left shows the folder you were in at startup, and a
relative file name means a file there. That is not a matter of course:
`dhrt run` changes into the directory of the source before running, i.e. into `ide/`.
So that a program still knows where it was called from, `dhrt` stores
that location beforehand in the environment variable `DHRT_START_DIR`.

Above the tree, next to the project name, there is a **filter**: part of the
file name (case-insensitive) keeps only the matching files; the
folders stay and are filtered the same way when expanded.

## The stages at a glance

It grew in stages, one per round. The table says what came in
which one — for the question "when did … actually arrive?"; what it can do **today**
is described in full below, without stages.

| Stage | What was added |
|---|---|
| 1 | Tabs, project tree, syntax colouring, error list, help for the word, completion, find/replace, run with output |
| 2 | Debugger, profile, find in project, command palette, markers in the line number column, real fonts |
| 3 | Manual in a window, printing and PDF, Tools menu, expressions in the debugger, installer without Python |
| 4 | Welcome page, session and settings in one file, open file in project, the line operations, bookmarks, outline, conditional breakpoints, export |
| 5 | Folding, multiple cursors, split view, minimap, git blame, rename, snippets, signature help, session per project |
| 6 | Indentation on line break, occurrences and bracket pair coloured, suggestion list while typing, git diff/log, search with regular expressions |
| 7 | Settings dialog, command reference, peek definition, symbol trail, autosave, colour swatches with colour picker |
| 8 | Snippets by abbreviation and Tab, symbol index across the project, definition across file boundaries, multiple selection in the project tree |
| 9 | Indent guides, search output, replace in the whole project, close other tabs, About box |
| 10 | Column selection, line tools, rename in the whole project, compare two files |
| 11 | Toolbar with icons, cards with preview images of the examples |
| 12 | Expand/shrink selection, extract selection into a subroutine, compare two tabs side by side |
| 13 | Preview before refactoring, reorder parameters including calls, scaffold while typing (`END IF` comes along) |
| 14 | Reorder parameters **in the whole project**, list the callers of a subroutine, signature placeholders in completion |
| 15 | Objection when the new name is already taken; add and remove parameters; the callers as a **tree** |
| 16 | Move a subroutine into another file (including IMPORT); refactorings also for methods; undo a written refactoring in one go |
| 17 | Move classes; moving creates the target file; objection when a rename tears an override apart; arrow keys in the picker |
| 18 | More matte surface (the gradient fades out on large areas); what is moved takes along what it needs; undo several times; callers know FUNCREF |
| 19 | Deselect individual files in the preview; objection when the name already exists in the target; move constants and global variables |
| 20 | Skip individual **blocks** in the preview; renaming updates the handlers in `.dhform`; the callers show the measured run counts |
| 21 | **Revise** the last refactoring; the refactorings see **subfolders**; a constant that is used counts for the `IMPORT` |
| 22 | The project tree shows the subfolders; a refactoring can be **checked beforehand**; a newly created file gets a tab |
| 23 | The tree also shows forms, data and images; a running refactoring can be **cancelled**; checking names the **errors** |
| 24 | The project tree is a **runtime widget** (`GUI_FILETREE`): it reads the folder itself, shows a new file by itself and expands on click; plus checkmarks in the tree and **tabs inside a window** |
| 25 | The manual is **rendered by the widget** (`GUI_RICHTEXT`, with clickable links); checkmarks in the tree restrict the project; without preview, checking still happens; moving and parameters also collect in steps |
| 26 | The **toolbar is a runtime widget** (`GUI_TOOLBAR` with items, drawn icons in toolbar size, separators, spacer, toggle buttons); menus and toolbar carry the same icons, the hand-painted 16-pixel images are gone |
| 27 | **Button variants** and **comfortable list boxes** in the runtime; the command palette shows shortcuts and folders as extra text on the right, the main buttons of the dialogs are highlighted, the debugger stop is red |
| 28 | **Status bar with fields** (message, cursor position, wrap — the two on the right clickable; a message expires by itself after twelve seconds), the symbol trail is a **breadcrumb bar** (a click jumps to the block), and the toolbar puts what does not fit into a **» menu** |
| 29 | Select in the **rendered manual** (drag, double-click = word, Ctrl+A) and copy (Ctrl+C); the search goes **forward and back** with wrap-around, selects the match and highlights **all matches** |
| 30 | The runtime's table can be a **grid** (`GUI_GRID`, cell mode: current cell, range, typing edits, Ctrl+C/V as tab-separated text, number and choice columns); the **profile** is a grid, its rows can be selected and copied |
| 31 | A **`.dhform` opens the form designer in Drachenhauch** (as text via the command palette); the designer knows all 30 kinds and the grid, shows fields per kind (items, columns, cell mode, editable columns, column kinds, min/max/value) and writes **DH code without `GUI_LOAD`** (Ctrl+G) |
| 32 | The test collections can test **existing programs** (`--- programm pfad nach MARKE`, `--- inhalt`/`--- ohne datei`) — the form designer's tests run **without Python** |
| 33 | The **animation FSM editor and the score editor** are also tested without Python: `--- streichen` removes `WINDOW_MAXIMIZE()` from the copy, `--- nachher` lets a Drachenhauch program read what was saved, and placeholders also work in fixture files |
| 34 | **The IDE itself** is largely tested without Python: the building blocks (`PROCESS_*`, `CODE_*`, text area) and the cases of stages 1 to 5 are in test collections; a `--- nachher` program reads the log line by line |
| 35 | **Stages 6 to 12** of the IDE are also tested without Python (indentation, completion, bookmarks, symbol trail, settings, snippets, project tree, replace and rename in the project, toolbar, cards including preview image, expand selection, extract) |
| 36 | **Stages 13 to 25** are also tested without Python (scaffold, preview before refactoring, reorder, add and remove parameters, placeholders, caller tree, close tab, move including IMPORT, undo and revise a refactoring, skip blocks, form handlers, file tree, rendered manual, checkmarks in the tree) |
| 37 | The **special cases** run without Python too: the test collections can run a helper program before the run (`--- vorher`, e.g. a git repository), one between runs (`--- zwischen`, e.g. measuring a click position from the screenshot) and start the program again (`--- nochmal [in ORDNER]`, session across a restart). The highlighting is counted in the image; only the PDF listing remains in pytest |

## What it can do today (stage 37, 13.09.2026)

| Area | What works | Shortcut |
|---|---|---|
| Start | Welcome page, as long as no tab is open: at the top the Drachenhauch logotype (from `daten/bilder/` next to `ide/`; the logo there also becomes the window icon, without the images the title stands there as text), New, Open, Examples, the recently opened files (double-click), the most important shortcuts. Before anything else the **intro** plays (any `.mp4` in `daten/video/`, fitted with black borders): any key and any click skips it, the close button quits the IDE; switch it off with "Intro at start" in the settings (`vorspann` in the ide.json). **Which** one plays is chosen by the "Which intro" dropdown next to it (`vorspann_datei`): every video there, named after the file (`vorspann_2.mp4` is called "Vorspann 2"), or "Alternating" -- then each start plays the next one (`vorspann_zuletzt`). [Watch] plays the chosen one right away. Putting a new video into `daten/video/` is enough for it to appear in the list. The **logotype** of the welcome page and of the About window (`daten/bilder/schriftzug.png`) is the last frame of `vorspann_2.mp4`, cut out by `tools/schriftzug.dh`; the IDE fits it into its box without distortion. The session comes back at the next start (open files, active tab, **breakpoints including condition, and bookmarks** -- `haltepunkte`/`lesezeichen` in the ide.json), as do theme and font size | |
| Files | New, Open, Open file in project (picker with fuzzy filter: `spred` finds `189_sprite_editor.dh`; plus the recently opened ones), Recently opened (submenu), Save, Save as, close a tab and reopen it (Ctrl+Shift+T); up to 12 tabs; confirmation for unsaved changes; print the listing via a print dialog (printer, copies, **font size, margin, line numbers, selection only** -- remembered, and the PDF uses the same) or as a PDF next to the source (default: Courier 9 pt, 20 mm margin, 66 lines per page, line numbers, header with page number; the line width is measured on the font) | Ctrl+N, Ctrl+O, Ctrl+Shift+O, Ctrl+S, Ctrl+W, Ctrl+Q, Ctrl+P |
| Edit | **Search bar** (stays open: search field with next/previous and hit count "3 of 12", switches **Match case**, **Whole word** and **Regex**, all hits coloured in the code; typing jumps to the first hit, Enter/Shift+Enter next/previous, ESC closes; Tab goes from the search field straight into the replacement field), replace **one at a time** (replaces the selected hit and moves on — so you see every place beforehand) or **all** (one step for Ctrl+Z; with Regex, `\1` back-references apply, without it a backslash stays a backslash), Go to line, Find in project (all `.dh` in the project folder, hits bottom right, double-click opens), with the same three switches as the search bar (they apply equally to find, replace and project search), TODO/FIXME list (the same search), command palette (typing filters, **arrows** select, Enter executes) | Ctrl+F, F3, Shift+F3, Ctrl+H, Ctrl+Shift+1, Ctrl+Alt+Enter, Alt+C, Alt+W, Ctrl+G, Ctrl+Shift+F, Ctrl+Shift+M, Ctrl+Shift+P |
| Git | Who wrote this (`git blame`, date and person per line), what did I change (`git diff` coloured in a window), history of this file (`git log`); the changed lines carry a marker in the margin (asked again after saving, not every frame) | Ctrl+Shift+B, Ctrl+Shift+D |
| Writing | Renaming a symbol across the whole file (`CODE_RENAME$`: whole words, comments and strings stay; a malformed name changes nothing) — and the **forms follow along**: a `.dhform` names its callbacks by name, and whoever renames the subroutine and leaves the file as it is has a button that no longer does anything; insert snippets (13 scaffolds, `\|` says where the cursor belongs, the line's indentation is taken over); a cursor on every occurrence of the word — afterwards typing changes all of them; Alt+click adds a cursor, ESC clears them. Signature help: if the cursor is in an argument list, the status line shows the signature of the call and the number of the argument | Shift+F6, Ctrl+J, Ctrl+Shift+L |
| Tabs | Every tab has a **close button**, the middle mouse button closes it too. If it carries unsaved changes, you are asked (Save / Discard / Cancel) — previously that only applied when quitting, Ctrl+W took them away without a word. **Close other tabs** leaves only the front one | Ctrl+W, Ctrl+Shift+W |
| File list | **Checkmarks restrict the project** (only visible on request: `File → Show checkboxes in project tree`, in the settings or in the palette; `baum_haken` in the ide.json -- when hidden, the checks are dropped, an invisible restriction would be a trap): if something is checked, every refactoring (rename, replace in project, move), the project search and the symbol index work only on that set -- a checked **folder** takes everything below it along (until 2026-09-27 it made the project empty); without checks the whole project applies. The tooltip above the tree says so — the label above the tree says so, `File → Clear checks in project tree` removes them. Since stage 24 the project tree is a `GUI_FILETREE`: it reads the folder itself and only the branches that are open. Folders are on top and expand on click; a file that another program creates appears after at most two seconds. Ctrl+click collects, Shift+click spans a range; **Open selected files** turns them all into tabs. While collecting nothing opens yet — otherwise every click would add a tab nobody wanted | Ctrl+Shift+E |
| Settings | Ctrl+U shows all switches in one place (theme, wrap, minimap, changed lines, regular expressions, suggestion list, indent guides, scaffold while typing, preview before refactoring, **close brackets**, **format on save** -- off by default, only `.dh`, never on autosave, and what cannot be parsed is saved as it stands --, code font size, **list font** -- outline, output, problems, default 18 -- and **manual font**, default 18, **interface font** (menu, tabs, tree, labels), default 18, **faint logo behind the code** (on by default; `GUI_TEXTAREA_BACKGROUND`, it fills the code field and does not scroll along), autosave). They take effect **immediately**, without Apply — a theme you only see after closing is one you choose blindly; in the menu they remain where you look for them | Ctrl+U |
| Lines | Everything on whole lines, of the selection or of the cursor line, each operation its own undo step: toggle comment, duplicate line, delete line, move lines up/down, indent/outdent (Tab belongs to the field itself), format document (`CODE_FORMAT$`, the same formatter as `dhrt fmt`; on a syntax error everything stays as it is), insert snippet (Ctrl+J fills the filter with the word start to the left of the cursor: whoever has typed `for` has the FOR loop in front of them; **or simply type `for` and Tab** — every snippet has a short word that expands it), autosave after an adjustable idle time. A new line takes over the **indentation** of the old one and indents one level further after `SUB`, `FOR`, `THEN` and the other block words; an `END` or `NEXT` alone on a line moves it back. And the **scaffold grows along**: after `IF x > 0 THEN` the line break puts the `END IF` right below, the cursor stays in between (not `CASE`, `ELSE` and `ELSEIF` — they indent but close nothing; can be switched off). Set and jump to bookmarks (blue marker) — forward and backward **across files**, plus the list of all of them | Ctrl+K, Ctrl+D, Ctrl+Shift+K, Alt+Up/Down, Alt+Right/Left, Shift+Alt+F, Ctrl+F2, F2, Shift+F2, Alt+F2 |
| Language | Colouring of the visible section, plus every occurrence of the word under the cursor, the matching bracket pair and a **colour swatch** next to every `&H` literal (a click on it opens the colour picker and writes the new colour back to that place); a **symbol trail** above the code says which class and which subroutine you are in; **Peek definition** (Alt+F12) shows ten lines around the definition without leaving the current place; help for the word (bottom right below the problems, with its own header; without a word a note says what appears here), completion — it opens by itself while typing from three characters on, **below the cursor** (above it if there is no room below), focus stays in the code field, Ctrl+Space brings it in, and if the name has a **signature** it comes along as a scaffold: `CIRC` becomes `CIRCLE(x, y, r)` with `x` selected, **Tab** goes to the next argument (not after `AS`, a type stands there, and not if a parenthesis already follows); Go to definition, also via **Ctrl+click** (while Ctrl is held, the pointer shows the hand over a word; the jump happens on release); **tooltip on hover**: if the mouse rests on a word, its help appears next to it -- the same as for the word under the cursor, wrapped and shortened, none past the end of the line; outline bottom left (SUB/FUNCTION/CLASS with methods, **a click jumps** -- on release, otherwise the field would select up to the mouse pointer --, arrows in the list make the code field follow, Enter jumps into it; without any, "No subroutines" is shown; the kind is in the keyword colour, the name in the name colour). **The code colours depend on the theme**: the light theme has its own darker tones, otherwise light-yellow strings would stand on white | Ctrl+Space, F12, Ctrl+click |
| Checking | Error list bottom right, by itself 0.6 s after the last change; a click jumps to the line; error lines carry an orange marker, and the **place itself is underlined** (wavy line, red for errors, yellow for warnings) -- if the mouse rests on it, the message is in the tooltip. **Quick fix** (Ctrl+.): if the cursor is on a message with a suggestion, a picker offers the fixes -- replace a typo with the intended name, create a missing `DIM` (in a subroutine below its header -- there is additionally the **global** variant at the top of the file, for a value that should outlive the call --, otherwise before the block at top level, the type guessed from the assigned value; a suggested name appears as it is written in the program), put parentheses around a command's values, `!=` by `<>`, a name from another BASIC by the one used here; the tooltip says when there is one. In addition: put a missing block end (`END IF`, `NEXT`, `END SUB`, `WEND` ...) after the body of the open block, insert a missing `IMPORT` at the top, and a variable that is created in a subroutine and never used is a **hint** (grey, does not count as a problem, Ctrl+. removes the line). The suggestions come from the runtime (`CODE_CHECK$`, field `korrekturen`); VS Code gets the same ones. **Check whole project** (Ctrl+Shift+F7): every `.dh` of the project including subfolders (the checkmarks in the tree apply), open tabs with their state in the editor, in steps with a counter in the status line (ESC cancels); the list shows `datei:zeile: Meldung` (file:line: message), a click opens it there. An error in an imported file appears once, at its own place; hints do not count | Shift+F7, Ctrl+., Ctrl+Shift+F7 |
| Debugger | Breakpoints (red marker) at the cursor line or by **clicking in the line number column** (left toggles, right with condition; the cursor stays where it is), conditional breakpoints (violet; `i = 3`, `hp < 10` — the expression is evaluated in the program, it only pauses when it is true); Debug runs to the first breakpoint, without breakpoints it stops at line 1; the paused line is marked yellow, variables (local and global) appear bottom right in place of the problem list, plus the step buttons; while the program is paused, the input line evaluates expressions (`? 2 * 21` → `= 42 (INTEGER)`). To the right of the variables the **call stack** (innermost on top, each outer line is the one in which it called the inner one; double-click jumps there) and **Watch**: expressions that are recomputed at every pause (Enter in the field adds one, [Remove] removes the selected one; they persist across several runs). **Double-clicking a variable** sets a new value -- an expression, converted as in an assignment (no text goes into an INTEGER variable; a `CONST` is not a variable, the compiler has already substituted it). **Run to cursor** pauses at the line with the cursor; without a running debugger it starts one. A breakpoint before it still pauses, and "run to here" lapses in that case. **Breakpoints in imported files** pause (the debugger names file and line, the IDE opens the file), a breakpoint on a line **without code** (comment, empty line, `SUB` header, `END IF`) slides to the next line with code, and the red dot moves along; if one never pauses, the status line says so. Breakpoints set **while the program is running** apply immediately, **Pause** (F6) stops it at the next line. If the program is waiting in an `INPUT`, the debugger field says so and the **input line** sends the answer. While typing, **breakpoints and bookmarks move with their line**. **All breakpoints** (Ctrl+Shift+F9) shows every breakpoint of every file with condition and source text: the checkbox switches one **on and off** (switched off it stays in grey and does not pause), double-click or [Jump] goes there, plus change condition, remove, all on/off/remove; whether on or off is stored in the ide.json. **Logpoints** ([Logpoint ...] in the breakpoint list, blue marker): instead of pausing, the breakpoint writes a line to the output (`» spiel.dh:4  i = 3`), expressions go in `{}` (`{{`/`}}` for the braces themselves, an error appears as `{? meldung}`). **Hit count** ([Hit count ...]): `5` pauses from the 5th time on, `=5` exactly at the 5th, `%5` every 5th time; counting only happens when the condition holds. **Restart** (Ctrl+Shift+F5) stops the running program and restarts it in the same mode (run, debugger, profile). Whoever changes a file while debugging is told once that the program still knows the old lines. **Variables as a tree:** arrays, tuples, MAPs and objects expand (at most 100 entries per level, three levels deep, the rest as "... n more"); what was expanded is expanded again at the next pause; classes are named as they were written (`<Held>`, not `<held>`). **Pause on error:** a runtime error that no `CATCH` takes pauses at its place -- variables, call stack and watches as they were at the error, expressions can still be evaluated; F8 (or a step) then ends it. A **graphics program** stays responsive while paused (the debugger fetches the window messages). **Window to the front:** on a pause the IDE comes to the front (the program releases the foreground for that), on Continue and Run to cursor the program window does (`PROCESS_FRONT`); a single step leaves the IDE in front. **Values in the code:** while paused, grey text after each line from the subroutine's header to the paused line shows which variables occur in it and what they currently hold (`GUI_TEXTAREA_HINTS`); they disappear when execution continues. **Values on hover:** if the debugger is paused in the file, the tooltip over a name shows its value -- including a member chain (`held.hp`), never with a call (`eval` with `id`); `eval`, Watch and the input line now also know fields and indices for that (`held.hp`, `feld[3]`, `karte["a"]`); the debugger does not evaluate a PROPERTY, because it would execute code | F9, Shift+F9, Ctrl+Shift+F9 all breakpoints, Ctrl+Shift+F5 restart, click/right-click in the line number column, F7, F6 pause, F8 continue, F10 over, F11 into, Shift+F11 out, Ctrl+F10 run to cursor, Shift+F5 stop, Ctrl+E expression |
| Profile | Run under `dhrt profile`; at the end a window with the lines by time (count, ms, share, source), a click jumps to the line. **By function** (checkbox top right) sums the same measurement per SUB, FUNCTION and method (`Klasse.methode`) — a line belongs to the innermost one enclosing it, the rest to the main program; a click jumps to the function's header. **Clear output** is under Run | Ctrl+Shift+Y |
| Run | Start with live output bottom left, input line for `INPUT`, stop; export as a standalone program (`dhrt --export --schlank`: with the smallest runtime that has all of the program's commands, into `<name>_dist/` next to the source; which runtime is shown in the output bottom left) | F5, Shift+F5, Ctrl+F6 |
| Tools | The companion editors in Drachenhauch (SFX generator, particle editor, tilemap editor, sprite editor, tracker, form designer, animation FSM editor, score editor) as separate programs; open the examples as a project | |
| View | Light/dark theme (every area carries a hint of its own colour: project and outline blue, output green, problems orange, debugger violet -- mixed from the theme's base colour), full screen, font larger/smaller/normal (10 to 32 px, remembered; **Ctrl+Plus/Minus/0 and Ctrl+wheel** -- the wheel anywhere in the window, one point per step, Plus and Minus on German as well as US layout and on the numeric keypad); word wrap (applies to all tabs); fold blocks (**remembered per file**: what was folded when the tab or the IDE was closed folds again at the next opening as soon as the outline knows the blocks; the blocks come from `CODE_SYMBOLS$` **and from the indentation** -- everything under which something more deeply indented stands, so a `FOR` loop too; a click on the triangle in the line number column does the same); show and hide the **side bar** and the **bottom panel** (Alt+1, Alt+2; the code gets the space, remembered in the ide.json; the bottom one comes back by itself as soon as a program is started, debugged or profiled); split view (two tabs side by side); **same file side by side** (Alt+Shift+G: on the left a second view of the same file with its own cursor and its own section, one text and one undo for both; it follows the active tab, the fold arrows and breakpoints are only on the right); minimap at the right edge (drawn word by word, a light box for the visible section, on the right errors red and warnings yellow, on the left the cursor line; a **click scrolls the place into the middle**, **dragging** makes the section follow -- the cursor stays where it is, as with the mouse wheel); changed lines in the margin; the IDE starts maximised | Alt+Enter, Ctrl+Plus, Ctrl+Minus, Ctrl+0, Ctrl+wheel, Alt+Z, F4, Ctrl+F4, Shift+F4, Alt+G, Alt+Shift+G, Alt+1, Alt+2 |
| Toolbar | Below the menu a row of **icons** for what you need all the time: new, open, save, start, stop, debug, check, find, word wrap (toggle, kept in sync with the menu and Alt+Z), on the right manual and settings. Since stage 26 the toolbar is a runtime widget (`GUI_TOOLBAR` with items), the icons are the same as in the menus. Each button calls the same command as its menu item and names its shortcut in the tooltip; can be switched off under View | — |
| Cards | On the welcome page the **showcase**: the examples from `examples/showcase.json` as cards (`GUI_CARD`: the whole card is the button) with image, title and short description in normal font size (the same list as in the Qt editor). The images come from `examples/screenshots/` (generated by `tools/showcase_bilder.dh`); if one is missing, the example really runs (`dhrt bild`), one after the other in the background and invisibly, and the image is afterwards stored next to the session. If not all rows fit, the **mouse wheel** scrolls over the cards -- but only if no other window (manual, palette ...) lies on top there. Without `showcase.json` eight fixed examples are shown. A click opens the example | Mouse wheel |
| Examples by topic | All examples in 16 topics (first steps, games, 2D graphics, 3D, sound, user interfaces, data and network, hardware, tools ...): *Tools* -> *Examples by topic*, the button on the welcome page or the command palette. Per topic a group header with the count, per example the first line of its header comment (without decorative lines, file names and internal notes; if it breaks mid-sentence, it fetches the next line), on the right the file name. The filter field searches both, Enter opens the first hit, [Open and run] starts it right away. The assignment is in `examples/kategorien.json` (maintained by hand; a test requires every example to appear in it exactly once) | |
| Refactoring | **Extract selection into a subroutine** (Ctrl+Shift+R): the selected lines move into a new `SUB` at the end of the file, the call stands in their place. What has to go along as parameters is not guesswork — a SUB sees global names anyway, so it is exactly the **local** names of the enclosing subroutine that occur in the lines; whatever is also assigned in them goes **BYREF**. Afterwards the IDE recounts the errors and says so if the selection was not balanced | Ctrl+Shift+R |
| Refactoring | **Reorder, add, remove parameters** (Ctrl+Shift+U): rearrange the parameters of the subroutine under the cursor, add a new one (`hp AS INTEGER = 0` — the part after the `=` goes to every call site) or take one away — and **the calls follow along**, in **all** `.dh` of the project folder. The order of the arguments is their meaning; a forgotten call silently passes the wrong thing. A call that passes a different number of arguments (omitted default value) or **names** them is skipped and counted — there the order means something else | Ctrl+Shift+U |
| Refactoring | A refactoring across the whole project collects **in steps**, a small chunk per frame: the status line counts along, **ESC cancels**. Done in one frame, the IDE would stand still that long — with four hundred files long enough that you take it for hung, and nothing could be cancelled that never even gets to draw | ESC |
| Refactoring | **Deselect in the preview**: on the left are the affected files with checkboxes — what you deselect stays as it is. A refactoring across twelve files is rarely meant for all twelve, and "all or nothing" would then mean: rework by hand. Finer still is **skip block**: the cursor on a changed line, and the contiguous block stays as it was (it is then shown in grey with `~`). Behind this there is no longer a text output but a sequence of steps — the text of the file is built from the same steps you see | — |
| Refactoring | **Preview before refactoring**: rename, rename in project, replace in project, extract and the parameter refactorings first show the difference and ask (Enter applies, ESC discards). Can be switched off under Ctrl+U — **except when there is an objection**: if the new name already exists in the project, or if the rename tears an **override** apart (the same method exists in the superclass or in an inheriting class — from then on the other version would be called, without an error message), the preview opens and says so at the top. Whoever writes an objection only into the status line has not really raised it | — |
| Refactoring | **Move** (Ctrl+Shift+V): a subroutine or an entire **class** moves into another file of the project, together with the comment lines above it — and every file that uses it gets the `IMPORT` of the target file added. In Drachenhauch `IMPORT` inserts the text; without this part, moving would be a refactoring that breaks compilation. If the cursor is in a class, the **class** is meant — a method alone would no longer be a subroutine without it. The first entry in the picker creates a **new file**. And it works in both directions: if what is moved needs something that stays behind, the **target file** imports the source. If the cursor is on a `CONST` or `DIM` line, **the line** is the unit. If the target already has the name, the preview says so — two of the same name in one file are not a compile error, the second simply wins | Ctrl+Shift+V |
| Refactoring | **Check** in the preview: `dhrt --check` runs over the texts that **would** be written, with the current block selection. Whoever skips a block and breaks something with it sees it before writing — and **which** error, not only how many: the messages appear at the top of the diff. Only on request — a refactoring across twelve files would otherwise need twelve compile runs before you even get to see the difference | — |
| Refactoring | **Revise last refactoring** (Ctrl+Shift+G): the same steps once more, with the same selection. A skipped block could otherwise only be brought back by undoing the whole refactoring and doing it again from scratch; the computation uses the steps, not what is currently in the file | Ctrl+Shift+G |
| Refactoring | **Undo refactoring** (Ctrl+Shift+Z): Ctrl+Z in the code field only undoes the tab you are in — a refactoring across six files would have to be undone six times, in six tabs you would first have to open for that. The last ten lie on a **stack**, each press undoes one more. **If one of the files has been changed since the refactoring, it undoes nothing at all** and names it — otherwise the later work would be gone without a word (half undone would be worse than not at all). An undone refactoring can be **redone** (Edit, palette), with the same check in the opposite direction; a new refactoring empties this store | Ctrl+Shift+Z |
| Calls | **Who calls this?** (Shift+F12): the opposite direction of F12 — all places at which the subroutine under the cursor is called, across all files of the project. As a **tree**: below each call are the callers of the subroutine it is in, three levels deep; a click jumps there. Places where the name is passed on **without parentheses** (`f = malen`) are counted too — that is where it is decided that it runs later. If there was a profile run, every place shows **how often** it ran; without a run nothing is shown there, a zero would be a statement nobody has measured. The definition is not listed, it is not a call | Shift+F12 |
| Occurrences | **All occurrences in project** (Ctrl+Shift+F12): every line in which the name under the cursor appears as a whole word — a variable, a constant, a field, in any spelling, not in strings and comments, open tabs with their state in the editor. The list appears bottom right as with the project search, a double-click jumps there. This is text, not name resolution: two local `i` in two subroutines appear in the same list | Ctrl+Shift+F12 |
| Selection | **Expand** takes the next larger bracket: word, line, block, parent block, whole file. **Shrink** goes the same way back — a stack remembers each level instead of guessing it again | Ctrl+Shift+Up / Down |
| Rename | **Rename in whole project** (Ctrl+Shift+F6): the name under the cursor, in all `.dh` of the project folder. `CODE_RENAME$` leaves out comments and strings, as when renaming within one file | Ctrl+Shift+F6 |
| Compare | **Two tabs side by side**: the split view turns on, and in both fields every differing line gets a marker. The comparison is done without git via the longest common subsequence on lines — the tabs do not need to be saved for this, and it is exactly while typing that you want to know | — |
| Compare | **Compare with another file**: `git diff --no-index` in the same window and with the same colouring as git diff. Meant is the file selected in the project tree if it is a different one — only otherwise does the file dialog ask | — |
| Line tools | **Sort**, **Remove duplicates**, **Remove trailing whitespace** — on the selection, without a selection on the whole file | — |
| Replace | **Replace in whole project** (Ctrl+Shift+H): first counts the places and asks, then writes all `.dh` of the project folder. A tab with un**saved** changes cancels it — those would overwrite the file again at the next save | Ctrl+Shift+H |
| Output | **Search output** (Ctrl+Shift+A): the lines of the running program. The same text again means find next. **To the place of a message:** a line with `datei.dh:zeile` (runtime and parse errors, warnings) is coloured, double-click or Enter opens the file there -- the name is looked for next to the program that ran, then in the project, and may contain spaces | Ctrl+Shift+A, double-click/Enter |
| Symbols | **Find symbol in project** (Ctrl+Shift+S): all SUB, FUNCTION, CLASS and methods across ALL `.dh` of the project folder, filterable; the filter is pre-filled with the word under the cursor. The same index backs **Go to definition** and **Peek definition** across file boundaries: what `CODE_DEFINITION` does not find in its own text may be right next door | Ctrl+Shift+S |
| Help | **Look up built-in commands** (Ctrl+F3): all names the completion knows, with signature and description, filterable, with a button for inserting. Manual in a window, **rendered instead of raw** — since stage 25 by the runtime widget (`GUI_RICHTEXT`): headings in three sizes, paragraphs wrapped, bullet lists with dots, code blocks monospaced, tables with **columns**, bold and italic, and **clickable links** (a link to another document opens it, an anchor scrolls within the same one); the button at the top right switches to the source. Since 2026-09-27 it is **coloured and navigable**: on the left a **table of contents** from the headings (a click jumps, while reading the highlight moves along), **Back/Forward** as in a browser (buttons top left or Alt+Left/Right over the window; a step is another document, an anchor or a jump in the contents), code blocks in the editor's colours, code in the text on its own background, headings in three colours, tables with a tinted header and alternating rows, quotes as one block with a background. **Every code block has buttons**: *Copy*, *Into new tab* (an unsaved tab with the code) and *Start* (runs immediately without asking for a name -- stored as `handbuch_beispiel.dh` next to the `ide.json`). **In all** searches all documents: on the left the hits per document are then listed with line number, a click opens the place and highlights the search text, *Contents* brings back the table of contents. The **source** is in the editor's font and is coloured (headings, `Code`, **bold**, links, table pipes, code blocks as in the editor). The window grows with the screen and can be dragged at the corner; messages appear below the text. F1 looks up the word under the cursor in `docs/` and opens the document with the most occurrences in code font; dropdown of all documents, search in the document; keyboard shortcut overview; **About Drachenhauch** shows icon, logotype, version and folders (Enter or ESC closes). The interface writes umlauts; the search in the command palette and pickers therefore accepts both spellings (`oeffnen` finds "Öffnen") | F1, Ctrl+F1 |

Settings and session live in ONE JSON file in the user profile
(`%APPDATA%\Drachenhauch\ide.json`, otherwise `~/.config/Drachenhauch/ide.json`):
`zuletzt`, `sitzung`, `aktiv`, `hell`, `schrift`, `umbruch`, `karte`,
`git_rand`, `regex`, `vorschlag`, `geruest`, `umbau_vorschau`, `autosichern`, `wiederherstellung_s` and `projekte` (session per folder, the last twelve — whoever works
on two things does not want to find the other one's tabs when switching).
The environment variable
`DH_IDE_KONFIG` sets a different location — the tests need this, otherwise
every test run would write a foreign session into the user's file.

**Crash recovery** (since 18.09.2026). Next to the `ide.json`
there is a folder `wiederherstellung/`. Every running IDE writes there every
30 seconds (`wiederherstellung_s`) into its **own** subfolder
`<kennung>/stand.json`: path and text of every tab with unsaved
changes, including unnamed ones, plus the time. It is written into a
side file and then renamed, so that a crash in the middle of writing does not leave
half a state behind. The time is also the sign of life: at
startup the IDE only offers what is **orphaned** — marked as finished
or without a sign of life for three ticks. It leaves alone the state of a second IDE that
is currently running. (The Qt version had a shared
folder for all: there a second IDE took the first one's backups for
crash remnants, and whoever quit cleanly first deleted the other's.) The
question at startup has three answers: **Recover** opens the tabs
with their rescued text, *unsaved* — the file on disk stays
as it was until you save yourself. **Discard** deletes the states,
**Later** (also ESC) leaves them, and the next start asks again.
An orphaned state without tabs disappears silently. If you quit cleanly
(nothing unsaved, or via the confirmation with Save or Discard),
your own state goes away. **If the IDE ends without a confirmation** with
unsaved tabs, it stays behind marked as finished.

**The window's close button** (also Alt+F4) has asked since 18.09 like
File → Quit: if anything is unsaved, the confirmation comes (Save all /
Discard / Cancel), otherwise the IDE ends immediately. Before, the
loop ended at `QUITREQUESTED()`, and the work was gone without a word -- found
while building the recovery. This needed two things in the runtime:
`WINDOW_CLOSE_REQUESTED()` reports the close button without counting the frame limit of a
test run, and **ESC no longer ends a program with gui**
(see `docs/builtins-grafik.md`) -- before, an ESC in a dialog closed the
whole IDE, including unsaved work. Tests
`tests/pruef/werkzeug_ide_schliessen.dhtest` (real `WM_CLOSE` via
PowerShell).

## What it is built from

The IDE needs nothing from `dhrt` that another program would not get
as well. These building blocks came with it:

- **Processes with live output** (`PROCESS_START/READ$/ERR$/WRITE/
  RUNNING/CODE/KILL/CLOSE`, [builtins-core.md](builtins-core.md)). The
  output of the started program arrives line by line while it
  runs; `PROCESS_WRITE` passes input through to its `INPUT`. A `dhrt` started by
  `PROCESS_START` writes every `PRINT` line out immediately
  (environment variable `DHRT_LIVE=1`), otherwise on a pipe
  everything would only arrive at the end.
- **Language services as built-ins** (`CODE_CHECK$`, `CODE_HOVER$`,
  `CODE_COMPLETE`, `CODE_DEFINITION`, `CODE_REFERENCES`, `CODE_SYMBOLS$`).
  The same core as `dhrt lsp` and `dhrt --check`, without a process and without
  JSON-RPC.
- **Text area commands** (`GUI_TEXTAREA_CURSOR/GOTO/SELECTION$/SELECT/
  INSERT/FIND`, [module-gui.md](module-gui.md)): read and set the cursor,
  read and set the selection, insert at the cursor, search from a position.
  Without them a program could only read a text area as a whole and write it as a
  whole.
- **Markers in the text area** (`GUI_TEXTAREA_MARKS(ta, zeilen, farben)`): a
  dot in the line number column and a tint of colour over the line, for
  breakpoints, the paused line and error lines. They are tied to the
  line number; the IDE sets them anew as soon as something changes.
- **The selection range** (`GUI_TEXTAREA_SELECTION_RANGE(ta)` → line and
  column of start and end): the selected text alone
  (`GUI_TEXTAREA_SELECTION$`) does not say WHICH lines are meant — and
  toggle comment, indent or move lines work on
  lines. Writing then goes via `GUI_TEXTAREA_SELECT` +
  `GUI_TEXTAREA_INSERT`, so that every operation stays its own undo step
  (`GUI_SET_TEXT` would empty the history).
- **The formatter as a built-in** (`CODE_FORMAT$(quelltext$)`): the same
  core as `dhrt fmt`, without a process and without a file. Empty if the
  source cannot be parsed — nobody shuffles broken code around.
- **Window title afterwards** (`GUI_WINDOW_TITLE(win, titel$)`): the
  command palette and the file picker are the same window.
- **Folding** (`GUI_TEXTAREA_FOLDABLE/FOLD/FOLD_ALL/FOLDED/FOLDS`): which
  lines form a block is said by the program — the runtime knows no
  language here and counts no indentation. It sits in `ta_rows`, the
  single source of visible lines, and thereby works for drawing, clicks,
  arrows, cursor and scrolling at once.
- **Multiple cursors** (`GUI_TEXTAREA_ADD_CARET/CARETS/CLEAR_CARETS`,
  Alt+click, ESC): every change runs through **one** place, which applies it at
  every cursor from back to front. With one cursor this is exactly
  the path from before.
- **Renaming** (`CODE_RENAME$`): the same occurrences as
  `CODE_REFERENCES`, but with columns — the reference command throws those away.
- **Reading visibility** (`GUI_WINDOW_SHOWN(win)`): without the getter a
  program has to remember what it set itself, and is wrong
  as soon as the user closes the window via its close button.
- **Indentation on line break** (`GUI_TEXTAREA_SET(ta, "auto_einzug", 1)`
  + `GUI_TEXTAREA_INDENT_WORDS`): the runtime takes over the indentation of the
  current line -- that is language-independent; which words mean one level more or
  less is said by the IDE. The same principle as with folding.
- **Column selection** (`GUI_TEXTAREA_SELECT_COLUMNS`, with the mouse **hold
  Alt and drag**): a rectangle instead of a run. Every line
  gets its own cursor including selection — the machinery for this has been there since
  the multiple cursors. A line that is too short gets its
  cursor at the end instead of silently dropping out.
- **Indent guides** (`GUI_TEXTAREA_SET(ta, "einzugslinien", 1)`): a
  fine stroke per level, below the text. The width of a level is measured on
  `tabbreite` spaces, and an empty line takes the smaller
  depth of its neighbours — otherwise the line would break off in every paragraph.
- **New project from template** (File → New project from template ..., Ctrl+Alt+N):
  game (window with game loop, figure from `assets/figur.png`, moved with
  the arrow keys and `DELTA()`), form application (gui with text input,
  button and list box), console program (`main.dh` with INPUT, `rechnen.dh`
  and a test collection `tests.dhtest`) or empty; each with LIESMICH.md.
  Name and location (default: next to the current project, [Choose ...] opens
  the folder dialog); an occupied folder is rejected. Afterwards the
  new folder is the project and `main.dh` is open.
- **Images and sounds:** if the mouse in the code rests on a string that
  names an existing image (`LOADIMAGE("assets/figur.png")`), it appears
  after a short pause next to the pointer together with its dimensions -- looked for next to the
  file, then in the project. An image or sound from the project tree opens
  a **preview** in the IDE (image with dimensions and size, sound with
  [Play]/[Stop]); [Open with the system] is still there.
- **Packages** (File → Project packages ..., Ctrl+Alt+P): the libraries from
  `paket.json` as a list, per row name and source, the checksum in the
  tooltip; a package that is missing in the folder `pakete/` is shown in red.
  **Fetch package ...** asks for the source (`github:nutzer/repo@stand`,
  an https address or a folder, optionally with `--als name`), **Fetch
  all** fetches everything against `paket.lock.json`, **Renew** accepts new
  content, **Remove** asks first. Behind it runs `dhrt paket -C
  <projekt>` as a separate process, with its output below; afterwards
  the project tree, the symbol index and the check of the front file are fresh -- an
  `IMPORT` into a package just fetched is then no longer an error. The packages
  appear in the project tree under `pakete/`. **Refactorings across the whole
  project, project search and Check project leave out `pakete/`** (foreign
  code that the next fetch overwrites); Go to definition, Open file in
  project and the inheritance check see the packages. Whoever opens a
  file from a package reads in the status line that changes
  will be lost at the next fetch.
- **Source control** (Edit → Source control (git) ..., Ctrl+Alt+G): the
  changed files of the git repository as a list; the checkbox stages a
  file for the next commit (`git add`) or takes it back;
  on the right is its diff (a new file in full). Below that the message
  and **Commit** (Enter in the field does the same), **Discard change**
  (with confirmation; not while the file stands unsaved in a tab,
  and not for a new file -- that would mean deleting), **Pull**/**Push**
  (`git pull`/`push` as a separate process, the output runs along below).
  At the top is the branch with ahead/behind count, next to it the selection of
  branches with **Switch** and **New ...** (creates a branch at the current
  state, the changes come along). **Stash**/**Unstash**
  (`git stash push -u` / `pop`). Switch and Stash refuse
  as long as a tab is unsaved, and afterwards reload open tabs from
  disk. A file with a conflict is called "Konflikt" (conflict) in the list;
  **Edit → Conflict at the cursor** replaces the block
  `<<<<<<< … ======= … >>>>>>>` around the cursor with mine, theirs or both
  versions (one step, Ctrl+Z undoes it). **View → Who wrote
  this line (at line end)** shows, dimmed at the end of the cursor
  line, who changed it last, when and with which message
  (`git blame`, only for saved tabs).
- **Introduce / inline variable, SUB to FUNCTION** (Edit, Ctrl+Alt+V /
  Ctrl+Alt+I / Ctrl+Alt+U): a selected expression becomes a variable
  (`DIM name AS TYP : name = …` before the statement, the type guessed and
  changeable in the box); a variable that is assigned exactly once
  disappears, and at every reading place the expression stands -- refused
  if it is assigned a second time, passed to a subroutine with a BYREF parameter,
  if the expression calls something and would be needed several times,
  or if one of its variables changes in between. A SUB with exactly one
  BYREF parameter becomes a FUNCTION: the parameter becomes local, `EXIT SUB`
  becomes `RETURN r`, and every call `name(a, x)` as a statement becomes,
  throughout the project, `x = name(a)` -- refused if the SUB reads the
  value passed in before setting it. All three go through the
  refactoring preview.
- **Live preview** (View → Live preview, Ctrl+Alt+L): after every pause
  in typing, the program in the tab runs for 30 frames via `dhrt bild`,
  the last frame appears in a window of its own. What is computed is a
  hidden copy next to the file (`.dh_live_vorschau.dh`, so that relative
  images and IMPORTs are found), which is gone again afterwards; the
  program's input is closed, after 10 s it is aborted. A program
  without a window or with an error shows the message instead of an image.
  The window stays on top of the code and can be moved by its
  title bar, even after you have clicked into the code.
- **Call counts** (View → Call counts above subroutines, on by
  default): after every SUB/FUNCTION it says, dimmed, how often its name is used in the
  project ("3 Aufrufe" = 3 calls, "nicht aufgerufen" = not called) -- also in other
  files, as FUNCREF and as a method, not in a comment, not in a
  string. A click on it shows "Who calls this?". What is counted is
  text (`CODE_NAMES`), not name resolution: two methods of the same name
  count together. While the debugger is paused, the values stand there, afterwards the
  counts again.
- **Test collections** (Run → Run test collection, Ctrl+Alt+F5): `dhrt
  test` over the `.dhtest` in the tab (otherwise over the project folder), the
  output runs along; every failed case appears with file and line in
  the list bottom right, a click jumps to it. Ctrl+Alt+Shift+F5
  takes only the case under the cursor (`dhrt test --fall Name`).
  **Overview** (Ctrl+Alt+T): all `.dhtest` of the project as a tree, one
  dot per case -- green passed, red failed, grey not run yet
  or skipped; a collection is red as soon as one case in it is
  red, and is then expanded. [Run all], [Run selection]
  (a whole collection or one case alone), [Failed ones] (only the
  red ones, each with `--fall`), double-click opens the case. The state
  holds until the next run, also across closing the window.
- **Own snippets** (Edit → Edit own snippets): a
  text file `schnipsel.txt` next to the `ide.json`. Every snippet begins with
  `=== Name | kuerzel` and reaches to the next one; `|` in the text is the
  position of the cursor. After saving, the file applies immediately; an own
  abbreviation that already exists **replaces** the built-in snippet (in Ctrl+J
  it then says "eigen, ersetzt" — own, replaces). An abbreviation that is not a word is left out —
  the snippet is then only available via Ctrl+J. **Save selection as snippet
  ...** appends the selected code with "Name | abbreviation" to the file.
- **Own key bindings** (Help → Change keyboard shortcuts ...): choose a command,
  type the new shortcut (empty = none). It is stored in `tasten.json`
  next to the `ide.json`, and only what differs from the default
  (`{"neu": "Strg+Alt+N"}`); the file can also be edited by hand
  and applies after saving. If a shortcut takes away another command's
  key, that one loses it — two commands on one key would both fire.
  Unknown commands and shortcuts the IDE does not know appear in the
  output. Reset deletes the file. Behind this is **one table** of all
  menu items with command and default shortcut (`menueBefehl`); the clicks
  also fire from it (previously 136 separate lines in the frame loop).
  Anything that has a menu item can be rebound.
- **Terminal** (View → Terminal, Alt+3, or the "Terminal" tab above
  the output): the input line below sends commands to `cmd` (Windows)
  or `sh`, the output runs along, errors in red, a return value other than 0
  is shown after it. Every command is a separate process in the terminal's
  folder (initially the project folder); **`cd` works**
  nevertheless, because the script reports its folder at the end. A variable set with `set`,
  on the other hand, only applies to that one command. Loops are written as in the
  command prompt (`for %f in (*.dh) do echo %f`, also `%~nf`) -- the
  terminal doubles the `%` for the script file. While a command is running, the
  input line goes to it (`set /p`, an `INPUT`); **Cancel** ends it
  together with everything it started. Arrow up/down scrolls through the
  commands, `cls`/`clear` clears. If you start a program (F5), the
  area switches back to the output. Under Windows umlauts arrive correctly
  (`chcp 65001`).
- **Parameter names at calls** (View, on by default): after a
  line with a call with at least two arguments, one of them a
  fixed one (number, text, TRUE/FALSE), the names of the parameters
  appear dimmed -- `kreis(100, 50, 20)` → `x, y, r`, also for built-in
  commands (`MID$("abc", 1, 1)` → `s, start, n`). As many names
  appear as there are arguments, without type; a single argument
  (`SETFPS(60)`) gets no hint. One hint per line (the first
  such call). The IDE remembers the signatures until the next
  save.
- **Indent on paste** (View, on by default): a multi-line
  block, pasted with Ctrl+V or via the context menu, gets the
  indentation of the place where it lands; its inner structure stays (the
  reference is the smallest indentation of the further lines -- so a
  block copied from its first word on fits too). In the middle of a line
  the further lines follow that line's indentation. The adjustment is
  an undo step of its own: Ctrl+Z first brings back the block as it was
  copied. Nothing is touched with tabs, and not in **column 0**
  either -- there nothing says where the block belongs, it stays as it was
  copied (previously it moved to 0 and lost its indentation).
- **Next / previous problem** (Edit, Alt+F8 or Shift+Alt+F8
  -- F8 alone belongs to the debugger): the cursor jumps to the next error
  or warning of the file (not hints), at the end starting again
  from the top; the message appears in the status line.
- **Go to last edit** (Edit, Ctrl+Shift+Backspace): back
  to the place where you last typed, also in another file;
  Ctrl+Alt+Left leads back to where you were before.
- **Compare with saved version** (Edit, also in the
  context menu of a tab): shows the difference between the tab and the
  file on disk as a coloured diff (three lines of context); from there
  **Save** or **Discard** (one undo step -- Ctrl+Z brings back the
  changes).
- **Case and join lines** (Edit → Lines, palette):
  UPPERCASE, lowercase, Title Case on the selection
  (without a selection on the word under the cursor); the selection stays on the
  changed text. **Join lines** turns the lines of the selection
  (without a selection: this one and the next) into one: the indentation of the first
  stays, a space in between (not after `(`/`[`, not before
  `)`/`]`/`,`), empty lines are dropped, a ` _` at the end of a line
  disappears along with it. Each one undo step.
- **Coloured bracket pairs** (View, on by default): every bracket depth has
  its own colour (three in rotation, in `farben.json` as `klammer1`..
  `klammer3`). Brackets in strings and comments do not count; the
  depth starts anew on each line, unless the previous one ends with ` _`.
- **Replace search hits individually:** the hits of Find in project
  (Ctrl+Shift+F) carry checkmarks, all on. Right-click on the list:
  Go to hit, Replace this hit ..., Replace checked hits
  ..., Check all/Check none; also Edit → Replace checked
  search hits .... Replacement happens only in the chosen lines, with
  the same switches as the search (match case, whole word, regex),
  through the refactoring preview and with undo. An open file
  gets the text in its tab (unsaved), the others on
  disk. If the list shows problems again, the checks are gone.
- **Right-click in the code field:** Cut, Copy, Paste, Go to
  definition, Who calls this?, Rename, Quick fix, Toggle
  breakpoint, Reveal in project tree. The right-click first puts the
  cursor at that place (inside a selection it stays), so the commands refer to
  the clicked word or the clicked line. In the line number column
  the right-click remains the one for a breakpoint's condition.
- **Right-click on a tab:** Close, Close others, Close to the
  right, Copy path, Reveal in project tree -- each for the
  clicked tab, not the front one. When closing several, a
  tab with unsaved changes stays open (the status line says how
  many).
- **Reveal in project tree** (Edit, Ctrl+Alt+B): selects the file of the
  front tab in the project tree, expands the folders above it and scrolls
  the row into view; a hidden side bar comes back. If
  the file is not in the project folder, the status line says so.
- **Type hints** (View → Type hints, on by default): after a
  line with `FOR EACH` there is, dimmed blue, what the variable holds
  (`p: INTEGER`, for a MAP in the pair form `k: STRING, v: FLOAT`) --
  its type is otherwise written nowhere. What the compiler does not know (a tuple,
  the result of `SPLIT$`) is not shown. The hints come with the
  check (`CODE_TYPES$`).
- **Custom colours** (View → Edit custom colours): opens
  `farben.json` next to the `ide.json`, the first time with the values of both
  themes as a template. Per theme (`"dunkel"`, `"hell"`) there are the
  code colours (`grund`, `kommentar`, `text`, `zahl`, `schluessel`,
  `operator`, `fundstelle`, `klammer`, `suchtreffer`, `typ`), the tones of the
  areas (`projekt`, `ausgabe`, `probleme`, `debugger`) and under
  `"oberflaeche"` every key of `GUI_THEME_SET` (`accent`,
  `window_bg`, `widget_bg`, `text_fg` ...), colours as `"#RRGGBB"`. Only
  what should be different needs to be in it; after saving (Ctrl+S) the
  file applies immediately. The output reports unknown names and wrong colours.
- **Files in the project tree** (right-click on a row): New file,
  New folder (in the selected folder or in the folder of the selected file),
  Rename, Delete and Copy path; the same commands are in the
  command palette. The right-click selects the row but does not open it.
  **Rename** carries along open tabs, breakpoints, bookmarks and
  start arguments, and every `IMPORT "..."` in the project that refers to the file
  gets the new name (in an open tab as an undo step of its own,
  otherwise on disk). **Delete** asks first,
  deletes permanently (a folder including its contents) and closes the tabs of the
  deleted files; the IDE does not delete the project folder itself.
- **Back to previous location / Forward** (Edit, Ctrl+Alt+Left /
  Ctrl+Alt+Right, also the mouse's side buttons): every
  jump into another file or by at least eight lines without the
  text having changed is remembered -- Go to definition, problem list, search, outline,
  a click far away. Page Up/Down and Home/End do not count. At most 50
  places; a new jump takes away the forward list.
- **Filter and save output:** the field "Filter ..." above the output
  shows only lines that contain the text (case-insensitive, empty = all);
  double-clicking an error line still jumps to its place. Right-click
  in the output: Copy line, Copy shown lines, Save
  output ... (the shown lines as a text file), Clear output.
- **Error text at line end** (View → Error text at line end, on by
  default): the message from Check appears after its line, red for an
  error, yellow for a warning -- readable without moving the mouse there.
  Several messages of one line appear in one hint, long ones are
  shortened (the whole sentence is in the problem list and in the tooltip);
  call counts and profile numbers of the same line follow after it.
- **Start arguments and environment per file** (Run → Start arguments ...
  or Environment for start ...): applies to the file in the front tab,
  for Start, Debug and Profile, and is kept in the `ide.json` (`start`).
  Arguments are split at spaces, `"in Anführungszeichen"` ("in quotation marks") stays
  one (`ARG$(0)` ...). The environment is written as `NAME=wert NAME2="mit
  Leerzeichen"`; it only applies to the started program, not to the
  IDE and not to the next start of another file. Empty = none.
- **Restart on save** (Run, off by default): if a
  program is running (F5), Ctrl+S restarts it -- also when an imported file
  was saved, and without switching the tab. Debugger and profile
  stay untouched; autosave never restarts.
- **Profile in code** (View → Profile in code, on by default): after
  Run → Record profile (Ctrl+Shift+Y) there is a colour band behind every measured
  line, pale yellow for little time up to strong red for the
  most expensive line, and at the 30 most expensive ones the numbers are at the end of the line
  (`0.42 ms  12.3 %  30000x`). The bands move with their line while typing
  (`GUI_TEXTAREA_LINE_COLORS`), also in a tab of a measured file
  opened later. What is measured is what ran: after larger changes
  measure again, or Command palette → "Remove profile from code" (the table
  in the profile window stays).
- **English interface** (View → "Language: English (at next start)",
  or back "Sprache: Deutsch"): menus, buttons, command palette, tooltips,
  dialogs and status line are English, shortcuts are then called `Ctrl+Shift+O`
  (the runtime understands both spellings, also in `tasten.json`).
  The switch happens at the next start (`sprache` in the `ide.json`,
  `DH_IDE_SPRACHE=en` overrides). Every display text goes through `tr$("...")`,
  the table is in `ide/sprache/en.txt` (per line German, tab,
  English); `tests/pruef/werkzeug_ide_englisch.dhtest` reports every text without
  a translation. The runtime's messages follow: the IDE sets
  `DHRT_LANG=en`, so check messages, hints and the errors of
  started programs come in English, as far as the catalogue knows them (see
  [rust-runtime.md](rust-runtime.md), "Meldungen auf Englisch"). German
  remain: the manual, the descriptions of the examples, the IDE's own messages
  in the output (for instance about key bindings) and the tools
  183–199.
- **Sticky block header** (`GUI_TEXTAREA_SET(ta, "kopfzeilen", 3)`, View
  → Sticky block header, on by default): if the header of a SUB, a
  loop or an IF has scrolled out at the top but the block is still in view,
  the header line stays pinned at the top — at most three, outermost first, on
  its own background with an edge. A click on it jumps there. The blocks are
  the same as for folding; the cursor never slips under a header.
- **Abbreviations at Tab** (`GUI_TEXTAREA_ABBREV` +
  `GUI_TEXTAREA_ABBREV_HIT`): if one of the given words stands to the left of the
  cursor, Tab reports it instead of indenting. What goes in its place
  is set by the IDE — the runtime knows no snippets.
- **Multiple selection in the tree** (`GUI_TREE_SET "mehrfachauswahl"` +
  `GUI_TREE_SEL_COUNT/SEL_NODE/IS_SELECTED/SELECT/CLEAR_SELECTION`):
  the same queries as for list box and table. The tree was the last
  kind of selection without them.
- **Colour swatches** (`GUI_TEXTAREA_SWATCHES` + `GUI_TEXTAREA_SWATCH_CLICKED`):
  a small square for a piece of text, at the **end** of its line — directly
  after it, it would lie on the next character. Which place in the text means a
  colour, again only the caller knows.
- **The growing scaffold** (`GUI_TEXTAREA_CLOSE_WORDS`): if the line opens
  a block, the line break puts the closing line right
  below it. The pairs (`IF` → `END IF`) are in the IDE — the runtime
  knows no language. What is already closed gets no second
  closing.
- **Close brackets** (`GUI_TEXTAREA_PAIRS`): `(`, `[`, `{` and `"`
  get their counterpart right after them; whoever types it anyway just steps
  over it, and Backspace between an empty pair removes both. The
  single quote is missing on purpose — it starts a comment.
  Can be switched off in the settings (second column) and in the palette.

The debugger is a client of `dhrt debug`: the child writes events
as JSON lines to stdout (`paused` with `line`, `file`, depth, `locals`,
`globals`, `stack` -- innermost first, each with `name`, `line` and `file` -- and
`watches` -- each with `expr` and `value` or `error`; `watches` as a separate
event after `set-watches`; `breakpoints` as the answer to
`set-breakpoints`, each with `file`, `line`, `actual` (where it really pauses) and
`verified`, and for one that never pauses additionally `in_program` (FALSE: the file
does not belong to the program at all -- the IDE sends the breakpoints of all
files and reports only those with TRUE); `run-to-error` when `run-to` cannot pause anywhere (foreign
file, no code below) -- the program then keeps standing; `input` with
`line` when the program is waiting in an `INPUT`;
`set-result`/`set-error`; `eval-result`/`eval-error`; `output`;
`finished`; `error` with `line` and `file`) and takes commands on stdin
(`continue`, `step-over`, `step-into`, `step-out`, `set-breakpoints`,
`set-watches` with `exprs`, `run-to` with `line` and optionally `file`, `set`
with `name` and `value` (an expression), `eval`, `input` with `text`,
`pause`, `stop`). Every variable in `locals`/`globals` can carry `children`
(built the same way: `name`, `type`, `value`, `children`); an instance has its
class name as `type`. Names appear as they are written in the program
(`btnAdd`, not `btnadd`), and a handle -- at runtime an integer
or still NIL -- carries the declared type as `type` (`GUI_WIDGET`, `DB_CONN`,
a class) instead of `INTEGER`. An uncaught error first reports itself as
`paused` with `reason: "error"` and `message` (at the error location, `eval` and
`set-watches` still work), then after the next command as `error`.

**Lines are lines of the file, not of the merged source.** Until
2026-09-27 the debugger counted after the `IMPORT`s were inserted: with an
`IMPORT` at the top it paused "in line 7" although it was at line 3, and
a breakpoint at line 4 never hit. An entry in `breakpoints` can also carry `hits` (hit count as above) and
`log` (log text; then the breakpoint reports itself as event `log` with
`text`, `line`, `file` instead of pausing). Conditions and expressions
compute `MOD`, `\` and `^` like the VM (until 2026-09-27 the debugger did not know them,
and `i MOD 2 = 0` paused on every iteration). `set-breakpoints` takes two forms,
also mixed: `lines` (+ `conditions`, line → expression) means the
started file, `breakpoints` is a list of `{file, line,
condition}` for every file of the program. A breakpoint on a line
without code slides to the next line with code of the **same** file.

stdin is read by a thread of its own. **What arrives while running
only applies at the next pause** -- except `pause` and a
`set-breakpoints`/`set-watches` with `"now": true`: those take effect at the
next line. Without the switch, a script that sends all commands
in advance stays correct: its `set-breakpoints` after a `continue`
means the pause after that. `input` commands wait until an `INPUT`
collects them; a `stop` before that wins.

At the first pause the IDE sends breakpoints (all files, `now`) and
watched expressions and waits for the `breakpoints` answer: if there is
a breakpoint exactly here or none pauses, it stays paused, otherwise
the program continues (or runs to the cursor if it was started that way).
While typing, the markers in the code field move with their line
(`GUI_TEXTAREA_MARKS_GET`); the IDE updates breakpoints and bookmarks
accordingly. It compares paths with `SAMEFILE` -- `helfer.dh` can arrive as
`C:\...\helfer.dh` or `C:/.../helfer.dh`.

**Light sweep** (`GUI_WINDOW_GLOW`): at startup and when a program
starts, a warm light sweeps diagonally from right to left across the
window, and the frames flash where it crosses them (kind `rahmen`).
Can be switched off in the settings (second column, `glanz` in the ide.json);
a test run with `DHRT_FRAMES` only shows it with `DH_IDE_GLANZ=1`. The profiler
(`dhrt profile`) delivers at the end a JSON line with `total_time`, `lines`
(line, `file`, count, time), the program output and, on an error,
`error`, `error_line` and `error_file`. As with the debugger these are lines of the
FILE (until 2026-09-27 of the merged source -- after an `IMPORT`
none was right); the profile table names lines of other files as
`helfer.dh:3`, the function view computes the ranges per file, and a
click jumps into the right file.

Before starting, debugging and profiling, the IDE saves **all** changed
tabs with a name, not only the front one -- the program reads an imported file
from disk. A folder as start argument
(`dhrt run ide/ide.dh -- projekt`) becomes the project. git is only asked for the margin with the
changed lines if the file lies in a repository
(`.git` searched upwards, without a process).

Where manual and examples live: `docs/` and `examples/` next to `ide/`, in the
repo as in the installation; the environment variable `DH_IDE_WURZEL`
overrides that (the tests need it because their IDE copy lives
elsewhere). If the examples are missing next to `ide/`, the IDE looks in
`%PUBLIC%\Documents\Drachenhauch\examples`, where the installer
puts them.

Fonts: a text font for the interface (Segoe UI, otherwise Arial or
DejaVu Sans) and a monospaced one for the code (Consolas, otherwise Menlo
or DejaVu Sans Mono), both loaded at 32 px and drawn at 16. If none is
found, raylib's bitmap font remains.

Every **refactoring** goes through one place: register the new texts, then
`umbauStarten`. If the preview is on, it shows the difference and asks;
if it is off, it writes right away — one path, not two that can
drift apart. Writing happens in two ways: into a
tab via selection + `GUI_TEXTAREA_INSERT` (so **one** undo step;
`GUI_SET_TEXT` would empty the history) or to disk, in which case an open
tab is updated along with it. The difference is computed by the same longest
common subsequence as the side-by-side comparison, coloured like `git diff`.

**Reordering the parameters** works on the whole text, not line
by line: a call may span several lines, and the arguments
are swapped as pieces of text — what stands between them stays.
The definition needs no special case: `SUB name(` looks to the searcher
like a call, and its "arguments" are the parameters. It skips strings
and comments; a comma in `PRINT "a, b"` is not an
argument boundary.

## Testing without looking

If you set `DH_IDE_LOG=<datei>`, the IDE writes its events line by line:
`bereit`, `geoeffnet <pfad>`, `geprueft <anzahl>`, `gestartet <pfad>`,
`beendet <code>`, `gesichert`, `geschlossen <nummer>`, `projekt <ordner>`, `haltepunkt <zeile> an|aus`,
`debug gestartet`, `debug pause <zeile>`, `debug beendet`, `profil <zeilen>`,
`suche <treffer>`, `palette <befehl>`, `handbuch <datei> <zeile>`, `pdf <pfad>`,
`gedruckt <drucker> <kopien>`, `werkzeug <datei>`, `eval <wert>`,
`haltepunkt <zeile> bedingt <ausdruck>`, `lesezeichen <zeile> an|aus`,
`lesezeichen sprung <zeile>`, `kommentar <von>-<bis>`, `dupliziert <von>-<bis>`,
`geloescht <von>-<bis>`, `verschoben <von>-<bis> <richtung>`, `formatiert`,
`gliederung <anzahl>`, `schnell <pfad>`, `export <pfad>`,
`exportiert <code> <ordner>`, `kuerzel <anzahl>`, `falte <zeile> zu|auf`,
`falten alle <anzahl>`, `geteilt <reiter> x <lage>+<breite> <lage>+<breite>`,
`geteilt aus`, `karte an|aus`, `karte sprung <zeile>`,
`umbenannt <anzahl> <name>`, `schnipsel <name>`, `signatur <text>`,
`marken <anzahl>`, `blame <zeilen>`, `hbansicht gesetzt|quelltext`,
`hbsuche <y> <anzahl> <markierter text>`,
`git diff|log <zeilen>`, `git rand <zeilen>`, `lesezeichen liste <anzahl>`,
`wieder auf <pfad>`, `spur <pfad>`, `peek <zeile> [<datei>]`,
`symbolindex <anzahl>`, `symbol <datei> <zeile>`, `baum offen <anzahl>`,
`befehle <anzahl>`, `linien an|aus`, `andere zu <anzahl>`,
`ausgabe treffer <zeile>`, `marke naechste <anzahl>`,
`projekt ersetzt <anzahl>` (-1 = cancelled, a tab was unsaved),
`ueber <fassung>`, `marken enden <anzahl>`, `zeilen <art> <anzahl>`,
`projekt umbenannt <anzahl>` (-1 = cancelled), `vergleich <zeilen>`,
`leiste an|aus`, `kachel <datei>`, `vorschau <datei>`,
`vorschau fertig <nummer>`, `vorschau neu`, `erweitern <was>`,
`verkleinern <tiefe>`, `herausgeloest <name> <parameter> <mehr fehler>`,
`reiter vergleich <links> <rechts>`,
`befehl eingefuegt <name>`, `einstellungen auf`, `autosichern <s>`,
`geruest an|aus`, `umbau schalter an|aus`,
`umbau vorschau <dateien> <weniger> <mehr>`, `umbau verworfen`,
`parameter <anzahl>` (0 = none found),
`parameter umgestellt <stellen> <uebergangen>` (-1 = cancelled, another
tab was unsaved), `aufrufer <anzahl>`,
`platzhalter <nummer> von <anzahl>`, `umbau einwand`,
`parameter neu <deklaration>`, `parameter weg <nummer>`,
`aufrufer baum <knoten>`, `aufrufer sprung <zeile>`,
`verschieben <name> <zeilen> <imports>` (0 = did not work),
`umbau zurueck <dateien>`, `umbau abgewaehlt alles`,
`umbau block <nummer> aus|an` (-1 = no changed line),
`umbau nachbessern <dateien>` (0 = none there), `umbau nachgebessert <dateien>`,
`umbau geprueft <fehler>`, `umbau neue datei <anzahl>`,
`umbau abgebrochen`, `extern <datei>`,
`aufrufer gemessen <anzahl>`,
`auto gesichert`, `sicherung <anzahl> [beendet]`, `sicherung bleibt|entfernt`,
`wiederherstellung angeboten <anzahl>|verworfen|spaeter`,
`wiederhergestellt <pfad>|(neu)`, `farbfeld <stelle>`, `farbe <wert>`, `leistenknopf <befehl>`, `statusfeld <nr>`, `pfad sprung <zeile>`, `ende`. This way the test collections see what it
has done; keys come in via `AUTOMATION_PLAY` (F5 starts, F7
checks). Stages 1 to 5 are tested by `tests/pruef/werkzeug_ide.dhtest` without
Python, stages 6 to 12 by `tests/pruef/werkzeug_ide_6_12.dhtest`,
stages 13 to 25 by `tests/pruef/werkzeug_ide_13_25.dhtest`, whatever needs several runs,
a git repository or an image measurement by
`tests/pruef/werkzeug_ide_sonderfaelle.dhtest`, the crash recovery by
`tests/pruef/werkzeug_ide_wiederherstellung.dhtest`, the building blocks individually by
`tests/pruef/ide_bausteine.dhtest`. Since stage 38 none of the cases is
in pytest any more: the PDF listing too is in the special cases, its packed
pages are unpacked by `BUFFER_INFLATE`.

## What is still missing

The list from stage 4 is done: rendered Markdown view,
code folding, minimap, split editor, multiple cursors,
snippets, signature help, rename, git blame, session per project,
word wrap and icons in the menus are all there. Two runtime building blocks
were added for that (folding and multiple cursors in the
text area), plus `CODE_RENAME$` and `GUI_WINDOW_SHOWN`.

The list from stage 5 is done too: folding by indentation,
git diff and history, search with regular expressions, bookmarks across
files and a minimap that shows words. In addition came the
indentation on line break, the suggestion list while typing, the
highlighting of occurrences and bracket pair, and reopening a
closed tab.

The list from stage 6 is done too: settings dialog,
command reference, peek, symbol trail, autosave, colour swatches
with colour picker and the pre-filled snippet filter.

The list from stage 7 is done too: snippets by typing and
Tab, multiple selection in the file list, a symbol index across
the whole project, and definition and preview across file boundaries.

The list from stage 12 is done too: the preview before refactoring
(instead of counting the errors afterwards), reordering the parameters including
calls, and the keyword scaffold that grows along while typing.

The list from stage 13 is done too: refactoring across
file boundaries, the list of callers and the signature placeholders.

The list from stage 14 is done too: the objection when renaming
to a name already taken, adding and removing parameters,
and the callers as a tree.

The list from stage 15 is done too: moving into another
file including IMPORT, the refactorings for methods (the cursor may for this
also stand **inside** the argument list) and undoing in one go.

The list from stage 16 is done too: moving classes, the
creation of the target file and the objection to a torn-apart override.

The list from stage 17 is done too: what is moved takes along
what it needs itself, the refactoring can be undone several times, and the
caller list knows FUNCREF. In addition came a **more matte surface**: the
gradient of the glass theme is made for buttons, and on the large areas
of a development environment it looked like a shadow across half the
height. The new metric `verlauf_hoehe` lets it fade out above 120 pixels.

The list from stage 18 is done too: the objection for a hidden
name, moving constants and global variables, and
deselecting in the preview.

The list from stage 19 is done too: skipping individual blocks,
the form handlers follow along, and the callers show the measured
run counts.

The list from stage 20 is done too: revising, the
subfolders and the constant that is used.

**Where the refactorings search.** Everything that says "in the whole project" has
also gone through the subfolders since stage 21, and since stage 22 the
project tree shows them as branches. Since stage 23 it also lists what lies next to the
source (`.dhform`, `.dhsprite`, `.json`, `.md`, `.csv`, `.png`
and so on). What the editor can edit goes into a tab; an
image or a sound goes to the system's program. Only `.dh` is checked —
sending a `.json` through the compiler fills the list with
messages about something that is not a program at all. Folders that start with `.`
or `_` are skipped, as are `target` and `__pycache__`: generated things live there,
not source code, and a build folder can have ten thousand files.

The list from stage 21 is done too: the subfolders in the tree,
checking before writing and the tab for a newly created file.

The list from stage 22 is done too: the other files in the tree,
cancelling and the named errors.

**Since stage 24 the project tree is no longer home-made.** It was
handwork: fetch all files per pattern, sort them, build the folder nodes
from them and drag along bookkeeping that maps node numbers to
file names. That is now a runtime widget
(`GUI_FILETREE`, see [module-gui.md](module-gui.md)), and two things
come with it that were missing before: a **freshly created file** appears
in the tree by itself (checked every two seconds; until stage 23 only after
the next opening), and a **click on a folder** toggles it
instead of only the narrow triangle. In exchange, folders are now **closed** until someone
looks inside — otherwise the tree reads a folder nobody looks into.

The list from stage 24 is done too: all four refactorings collect
in steps, a refactoring is also checked **without preview** (and opens it
if it introduces errors), and the **checkmarks** in the tree are the
means to restrict a refactoring to a self-chosen set of files.

Compared with the Qt IDE nothing has been open at menu level since stage 9; the smaller gaps in keyboard, mouse and panels are listed in `docs/entwurf-python-abbau.md`, section 7.3. Since
stage 29 you can **select and copy** in the rendered manual, and
the search goes **forward and back** (Enter / Shift+Enter, two buttons),
selects the match and highlights all others. What would be up
next: when moving there is still **no undo across
file boundaries** except for the steps that `umbauZurueck` holds. An
installer without Python has existed since stage 3:
`installer/Drachenhauch-IDE.iss` packs `dhrt.exe`, `ide/`, `docs/` and the
examples -- 33 MB instead of 92; the Qt IDE stays installable alongside until
this one catches up. **Since 2026-09-16 it is built in Drachenhauch**
(`dhrt run installer/bauen.dh`: version from `VERSION$()`, licences via
`installer/lizenzen.dh`, then ISCC); only the runtime itself is still built by
`rust/build_runtime.py` -- a running `.exe` cannot be
overwritten. **Since 2026-09-19 the same `bauen.dh` also packages for macOS
(`.dmg`) and Linux (`.tar.gz` with `install.sh`)**; the launcher copies the
examples at startup into `Dokumente/Drachenhauch/examples` and tells the IDE
via `DH_IDE_BEISPIELE` where they are (details in
[installer/README.md](../../installer/README.md)). The list is in
[entwurf-python-abbau.md](../entwurf-python-abbau.md) (German), section C.
