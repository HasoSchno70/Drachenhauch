# Plugins of the IDE

A plugin extends the IDE with its own commands, its own panel, work on
events (saving, opening, running …) and code helpers (checks, snippets,
colouring of further file types). It is a **Drachenhauch program of its
own**: the IDE starts it at startup with `WINDOW_OPEN` and talks to it over
its channel, one JSON line per message. A plugin that crashes or hangs does
not take the IDE down with it -- its state becomes "crashed" ("abgestürzt"),
its panels close, its commands are disabled. As in VS Code, the extensions
run in a process of their own.

## Where plugins live

Each plugin is a folder under `plugins/` next to the `ide.json`
(Windows: `%APPDATA%\Drachenhauch\plugins\<name>\`) with a `plugin.dh`
in it. **Plugins → Open plugin folder** creates the folder and opens it.
**Plugins → Manage plugins …** shows all of them with their state; the tick
says whether it is loaded at startup (`plugins_aus` in the `ide.json`).
**Plugins → Reload plugins** stops all of them and starts them again --
after changing a plugin that is enough, the IDE does not have to restart.

An example lives in `examples/plugins/todo/`: TODO and FIXME of the file in
front as a panel on the right, a command with a shortcut, a warning per
FIXME, a snippet and a colouring for `.todo` files. To try it, copy the
folder `todo` into the plugin folder and reload.

A third one in `examples/plugins/schreibweise/` shows the code helps in the
code field: typical German misspellings in comments ("Standart",
"seperat") are underlined, Ctrl+. replaces them, the tooltip names the
right spelling, and the context menu has "Schreibweise: Wort erlauben"
(allow word) for words meant that way (`erlaubt.txt` next to the plugin).

A second one in `examples/plugins/klassen/`: all classes and structs of the
project as a tree (from `CODE_CLASSES$`, without compiling), below them
fields, constants, properties and methods with their type; a filter field,
"only the file in front", and a click jumps to the place. It reads again
after saving and leaves out folders starting with `.` or `_`, as well as
`pakete/` and `target/`.

## Writing a plugin

The library `dh_plugin.dh` lives next to the IDE; the IDE puts its folder
into `DH_PATH`, so a plugin simply writes `IMPORT "dh_plugin.dh"`. The
program registers what it can do and then waits for events:

```basic
IMPORT "dh_plugin.dh"
PLUGIN_BEFEHL("hallo", "Hallo einfügen", "Strg+Alt+Umschalt+H")
PLUGIN_LEISTE("notizen", "Notizen", "leiste.dhform", "rechts")
PLUGIN_HOERE("gesichert")
WHILE PLUGIN_LAEUFT()
    SELECT CASE PLUGIN_EREIGNIS$()
        CASE "befehl"
            IF PLUGIN_ID$() = "hallo" THEN PLUGIN_EINFUEGEN("' Hallo" + CHR$(10))
        CASE "klick"
            PLUGIN_MELDUNG("geklickt: " + PLUGIN_WIDGET$())
        CASE "ereignis"
            PLUGIN_AUSGABE("gesichert: " + PLUGIN_DATEI$())
        CASE "ende"
            EXIT(0)
    END SELECT
WEND
```

`PLUGIN_EREIGNIS$` waits up to 50 ms and returns "" without an event -- so
the loop needs no `SLEEP` of its own. Without an IDE (the plugin started on
its own) `PLUGIN_LAEUFT()` is FALSE and the program ends right away.

## What a plugin can do

| Command | What it does |
|---|---|
| `PLUGIN_BEFEHL(id$, titel$ [, kuerzel$])` | register a command: it appears in the **Plugins** menu, in the command palette and on the shortcut. When triggered, the event `befehl` arrives with `PLUGIN_ID$()` |
| `PLUGIN_LEISTE(id$, titel$, formdatei$ [, bereich$])` | a panel of its own in the dock: a `.dhform` (from the form designer, say) as a window on the left, right (default), bottom or floating. The user can move it like any panel of the IDE |
| `PLUGIN_HOERE(ereignis$)` | listen to an event: `gesichert` (saved), `geoeffnet` (opened), `reiter` (another tab in front), `gestartet` (started), `beendet` (a program has run), `geprueft` (checked) -- each with `PLUGIN_DATEI$()`. Plus `hover`: when the mouse rests on a word, as an event of its own (see `PLUGIN_HOVER_ANTWORT`) |
| `PLUGIN_SETZE(leiste$, widget$, was$, wert$)` | change a widget of the own panel: `text`, `checked`, `value`, `enabled`, `visible` or `selected` (row of a list, entry of a dropdown, node of a tree) |
| `PLUGIN_LISTE(leiste$, widget$, eintraege)` | the entries of a list or dropdown of the panel |
| `PLUGIN_BAUM(leiste$, widget$, ebenen, texte [, auf])` | fill a tree of the panel: per entry a level (0 = top) and a text, in order from top to bottom; an entry hangs on the last one before it with a smaller level. The number of a node is its place in the list -- the same number arrives as the value of `geaendert`. `auf` (default TRUE) expands the branches |
| `PLUGIN_TEXT$()` | the text in the tab in front |
| `PLUGIN_DATEINAME$()` | its path ("" = not saved yet) |
| `PLUGIN_AUSWAHL$()` | the selected text |
| `PLUGIN_MARKE$()` | line and column of the cursor as `line,column` |
| `PLUGIN_PROJEKT$()` | the project folder |
| `PLUGIN_WORT$()` | the word at the cursor |
| `PLUGIN_EINFUEGEN(text$)` | insert text at the cursor (it replaces a selection) |
| `PLUGIN_MELDUNG(text$)` | a message in the status bar |
| `PLUGIN_AUSGABE(text$)` | a line in the output |
| `PLUGIN_OEFFNEN(pfad$ [, zeile])` | open a file, on request at a line |
| `PLUGIN_BEFUND(datei$, zeile, meldung$ [, schwere$ [, spalte, laenge]])` | collect a finding (`fehler` error, `warnung` warning or `hinweis` hint). It is underlined in the code like a message of the runtime, with column (from 1) and length exactly that place, otherwise the line without indentation, and shows as text at the end of the line and in the tooltip |
| `PLUGIN_KORREKTUR(titel$, text$ [, zeile, spalte, bis_zeile, bis_spalte])` | a quick fix (Ctrl+.) for the finding collected last: `text$` replaces the finding's place, or another one if given (columns from 1, `bis_spalte` exclusive). Several calls = several offers |
| `PLUGIN_BEFUNDE_SENDEN(datei$)` | send the collected findings of the file -- they appear in the problems list while the file is in front, with the plugin's name in front; a new send replaces the old ones, an empty send removes them |
| `PLUGIN_SCHNIPSEL(name$, kuerzel$, text$)` | a snippet (Ctrl+J, or the abbreviation + Tab); `\|` marks where the cursor goes |
| `PLUGIN_KONTEXT(id$, titel$)` | an entry in the context menu of the code field. When clicked, `befehl` arrives with this `id`, and `PLUGIN_WERT$()` is the word that was clicked (the right click moves the cursor there) |
| `PLUGIN_HOVER_ANTWORT(text$)` | answer to `hover` (after `PLUGIN_HOERE("hover")`): the text is added to the tooltip as long as the mouse still rests there; empty = nothing to add. Belonging to the event are `PLUGIN_WERT$()` (the word), `PLUGIN_ZEILE()`, `PLUGIN_SPALTE()` (from 1) and `PLUGIN_DATEI$()`. The IDE does not wait -- a plugin that does not answer does not hold up the tooltip |
| `PLUGIN_FAERBUNG(endung$, muster$, farbe)` | colouring for a further file type (`".md"`): each rule a regular expression and a colour; the IDE then colours such files with it instead of with the Drachenhauch colouring |

Belonging to the event are `PLUGIN_ID$()` (the command or the name of the
event), `PLUGIN_LEISTE$()` and `PLUGIN_WIDGET$()` (for `klick` and
`geaendert` -- the widget's name from the `.dhform`, field `name`),
`PLUGIN_WERT$()` (for `geaendert`: the text, a tick as `TRUE`/`FALSE`, the
value of a slider, the number of the selected row or tree node; for
`befehl` the word at the cursor) and `PLUGIN_DATEI$()`
(for `ereignis` and `hover`). The questions (`PLUGIN_TEXT$` and the others) wait up to
two seconds for the answer; whatever arrives in between is delivered by the
next `PLUGIN_EREIGNIS$`.

## The channel

Whoever does not want to use `dh_plugin.dh` (or writes a plugin that needs
nothing beyond `PARENT_SEND`/`PARENT_RECV$`) sends the lines directly. Each
one is a JSON object with `art` (kind).

From the plugin to the IDE:

| `art` | Fields |
|---|---|
| `befehl` | `id`, `titel`, `kuerzel` |
| `kontext` | `id`, `titel` |
| `leiste` | `id`, `titel`, `bereich`, `form` (the `.dhform` as an object) |
| `hoere` | `ereignis` |
| `setze` | `leiste`, `widget` (number in the form), `was`, `wert` |
| `liste` | `leiste`, `widget`, `eintraege` (array) |
| `baum` | `leiste`, `widget`, `ebenen` and `texte` (arrays of the same length), `auf` |
| `frage` | `nr`, `was` (`text`, `datei`, `auswahl`, `marke`, `wort`, `projekt`) |
| `tue` | `was` (`einfuegen`, `meldung`, `ausgabe`, `oeffnen`), `text`, `datei`, `zeile` |
| `befunde` | `datei`, `liste` (objects with `zeile`, `meldung`, `schwere`, optionally `spalte`, `laenge` and `korrekturen` -- each with `titel` and `aenderungen` holding `zeile`, `spalte`, `bis_zeile`, `bis_spalte`, `text`, as in `CODE_CHECK$`) |
| `hover` | `nr` (from the event), `text` |
| `schnipsel` | `name`, `kuerzel`, `text` |
| `faerbung` | `endung`, `muster`, `farbe` |

From the IDE to the plugin: `befehl` (`id`, `wert`), `klick` and `geaendert`
(`leiste`, `widget`, `wert`), `ereignis` (`name`, `datei`), `hover` (`nr`,
`wert`, `zeile`, `spalte`, `datei`), `antwort`
(`nr`, `wert`) and `ende`, shortly before the IDE closes the channel.

Per frame the IDE reads at most 50 messages of a plugin -- a chatty plugin
cannot stall it that way. An unreadable line goes into the log, an error
while carrying it out (an unknown shortcut, say) also into the output with
the plugin's name in front.

## Plugins in Glutfaden

Glutfaden (`examples/glutfaden`, from stage 26 on) loads plugins the same
way: a folder `plugins/<name>/plugin.dh` in Glutfaden's settings folder
(logo menu → Plugins → Open the plugin folder), started with
`WINDOW_OPEN`, the same line and the same library `dh_plugin.dh`. The
commands are in the logo menu under **Plugins**, their shortcuts work
anywhere in the window. Glutfaden rebuilds the panel from GTK widgets
(position and size as in the form): label, button, text field, check box,
slider, list and drop-down list; anything else shows as a note. It sits
among the dockable panels and can be dragged like Overview or Pattern.
The events are `gesichert`, `geoeffnet` and `geaendert` (after every change
to the pattern).

| Command | What it does |
|---|---|
| `PLUGIN_MUSTER$()` | the pattern as JSON: `name`, `breite`, `hoehe`, `stiche`, `farbe` (the selected one) and `farben` (each `name`, `nummer`, `hersteller`, `rgb`, `stiche`); "" without a pattern |
| `PLUGIN_STICHE(punkte, farben [, text$])` | stitches on the active layer: `punkte` = x, y, x, y ..., `farben` = for each point the number of the color in the pattern (-1 clears). One step for undo, labelled with `text$` (otherwise the plugin's name) |

`PLUGIN_DATEINAME$()` is the path of the pattern, `PLUGIN_PROJEKT$()` the
folder used last, `PLUGIN_TEXT$()` the same as `PLUGIN_MUSTER$()`. Whatever
belongs to a code editor (findings, snippets, highlighting, inserting) is
skipped by Glutfaden. On the line, `PLUGIN_STICHE` is a `tue` with `was` =
`stiche` and the fields `punkte` and `farben`.
