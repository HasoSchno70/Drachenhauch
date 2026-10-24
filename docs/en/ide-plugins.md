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
| `PLUGIN_HOERE(ereignis$)` | listen to an event: `gesichert` (saved), `geoeffnet` (opened), `reiter` (another tab in front), `gestartet` (started), `beendet` (a program has run), `geprueft` (checked) -- each with `PLUGIN_DATEI$()` |
| `PLUGIN_SETZE(leiste$, widget$, was$, wert$)` | change a widget of the own panel: `text`, `checked`, `value`, `enabled`, `visible` or `selected` |
| `PLUGIN_LISTE(leiste$, widget$, eintraege)` | the entries of a list or dropdown of the panel |
| `PLUGIN_TEXT$()` | the text in the tab in front |
| `PLUGIN_DATEINAME$()` | its path ("" = not saved yet) |
| `PLUGIN_AUSWAHL$()` | the selected text |
| `PLUGIN_MARKE$()` | line and column of the cursor as `line,column` |
| `PLUGIN_PROJEKT$()` | the project folder |
| `PLUGIN_EINFUEGEN(text$)` | insert text at the cursor (it replaces a selection) |
| `PLUGIN_MELDUNG(text$)` | a message in the status bar |
| `PLUGIN_AUSGABE(text$)` | a line in the output |
| `PLUGIN_OEFFNEN(pfad$ [, zeile])` | open a file, on request at a line |
| `PLUGIN_BEFUND(datei$, zeile, meldung$ [, schwere$])` | collect a finding (`fehler` error, `warnung` warning or `hinweis` hint) |
| `PLUGIN_BEFUNDE_SENDEN(datei$)` | send the collected findings of the file -- they appear in the problems list while the file is in front, with the plugin's name in front; a new send replaces the old ones, an empty send removes them |
| `PLUGIN_SCHNIPSEL(name$, kuerzel$, text$)` | a snippet (Ctrl+J, or the abbreviation + Tab); `\|` marks where the cursor goes |
| `PLUGIN_FAERBUNG(endung$, muster$, farbe)` | colouring for a further file type (`".md"`): each rule a regular expression and a colour; the IDE then colours such files with it instead of with the Drachenhauch colouring |

Belonging to the event are `PLUGIN_ID$()` (the command or the name of the
event), `PLUGIN_LEISTE$()` and `PLUGIN_WIDGET$()` (for `klick` and
`geaendert` -- the widget's name from the `.dhform`, field `name`),
`PLUGIN_WERT$()` (for `geaendert`: the text, a tick as `TRUE`/`FALSE`, the
value of a slider, the number of the selected row) and `PLUGIN_DATEI$()`
(for `ereignis`). The questions (`PLUGIN_TEXT$` and the others) wait up to
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
| `leiste` | `id`, `titel`, `bereich`, `form` (the `.dhform` as an object) |
| `hoere` | `ereignis` |
| `setze` | `leiste`, `widget` (number in the form), `was`, `wert` |
| `liste` | `leiste`, `widget`, `eintraege` (array) |
| `frage` | `nr`, `was` (`text`, `datei`, `auswahl`, `marke`, `projekt`) |
| `tue` | `was` (`einfuegen`, `meldung`, `ausgabe`, `oeffnen`), `text`, `datei`, `zeile` |
| `befunde` | `datei`, `liste` (objects with `zeile`, `meldung`, `schwere`) |
| `schnipsel` | `name`, `kuerzel`, `text` |
| `faerbung` | `endung`, `muster`, `farbe` |

From the IDE to the plugin: `befehl` (`id`), `klick` and `geaendert`
(`leiste`, `widget`, `wert`), `ereignis` (`name`, `datei`), `antwort`
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
