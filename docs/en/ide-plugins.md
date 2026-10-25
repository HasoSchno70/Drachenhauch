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

Each plugin is a folder with a `plugin.dh` in it, in one of three places:

* **Your own plugins** under `plugins/` next to the `ide.json` (Windows:
  `%APPDATA%\Drachenhauch\plugins\<name>\`). They run until you switch
  them off (`plugins_aus` in the `ide.json`).
* **Bundled plugins** in `examples/plugins/`, shipped with the installer.
  They are off until you tick them (`plugins_an`). A plugin of your own
  with the same name hides the bundled one.
* **Fetched plugins** under `plugins/pakete/<name>/` -- **Fetch …** in the
  manager takes a source as for `dhrt paket` (`github:user/repo@ref`, an
  https address of a ZIP, a folder or a ZIP on disk) and fetches the plugin
  with `dhrt paket -C <plugin-folder> hole`; the checksum goes into the
  paket.lock.json of the plugin folder, so fetching again gets exactly the
  same content. **Remove** takes a fetched one away again. They run like your
  own; the manager shows "(geholt)" (fetched) after the name. A plugin of
  your own with the same name comes first.

**Plugins → New plugin …** creates one of your own from the template (a
command, an entry in the context menu, a panel), opens its `plugin.dh` and
starts it. **Plugins → Manage plugins …** shows all of them with title,
version and state; the tooltip gives description, author and path; the tick
says whether it is loaded at startup. **Plugins → Reload plugins** stops all
of them and starts them again -- after changing a plugin that is enough, the
IDE does not have to restart. **Plugins → Open plugin folder** opens the
folder of your own plugins.

Three are bundled:

* `todo`: TODO and FIXME of the file in front as a panel on the right, a
  command with a shortcut, a warning per FIXME, a snippet and a colouring
  for `.todo` files.
* `klassen`: all classes and structs of the project as a tree (from
  `CODE_CLASSES$`, without compiling), below them fields, constants,
  properties and methods with their type; a filter field, "only the file in
  front", and a click jumps to the place. It reads again after saving and
  leaves out folders starting with `.` or `_`, as well as `pakete/` and
  `target/`.
* `schreibweise`: typical German misspellings in comments ("Standart",
  "seperat") are underlined, Ctrl+. replaces them, the tooltip names the
  right spelling, and the context menu has "Schreibweise: Wort erlauben"
  (allow word) for words meant that way (`erlaubt.txt` in the data folder).

### The profile

Next to the `plugin.dh` there may be a `plugin.json`:

```json
{"name": "Mein Plugin", "fassung": "1.0", "beschreibung": "Was es tut.", "autor": "Ich", "ide": "2026.36"}
```

The manager shows name and version instead of the folder name. `ide` is the
smallest IDE version the plugin needs -- with an older one it does not
start, and its state says so. Without a profile the folder name counts.

Each plugin has a **data folder** next to the `ide.json`
(`plugin-daten/<name>/`, `PLUGIN_DATENORDNER$()`). Its settings and lists
go there -- a bundled plugin may live in a folder it must not write to.

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
| `PLUGIN_HOERE(ereignis$)` | listen to an event: `gesichert` (saved), `geoeffnet` (opened), `reiter` (another tab in front), `gestartet` (started), `beendet` (a program has run), `geprueft` (checked) -- each with `PLUGIN_DATEI$()`. Plus `hover`: when the mouse rests on a word, as an event of its own (see `PLUGIN_HOVER_ANTWORT`), and `vervollstaendigen` (complete): when the completion list opens (see `PLUGIN_VORSCHLAEGE`) |
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
| `PLUGIN_ERSETZEN(zeile, spalte, bis_zeile, bis_spalte, text$ [, datei$])` | replace a range (lines and columns from 1, `bis_spalte` exclusive) -- in the tab in front or in `datei$`, which the IDE opens for it. One step for Ctrl+Z |
| `PLUGIN_AUSWAHL_SETZEN(zeile, spalte [, bis_zeile, bis_spalte [, datei$]])` | set the cursor, with an end a selection |
| `PLUGIN_DATENORDNER$()` | the folder the plugin may write to |
| `PLUGIN_MELDUNG(text$)` | a message in the status bar |
| `PLUGIN_AUSGABE(text$)` | a line in the output |
| `PLUGIN_OEFFNEN(pfad$ [, zeile])` | open a file, on request at a line |
| `PLUGIN_BEFUND(datei$, zeile, meldung$ [, schwere$ [, spalte, laenge]])` | collect a finding (`fehler` error, `warnung` warning or `hinweis` hint). It is underlined in the code like a message of the runtime, with column (from 1) and length exactly that place, otherwise the line without indentation, and shows as text at the end of the line and in the tooltip |
| `PLUGIN_KORREKTUR(titel$, text$ [, zeile, spalte, bis_zeile, bis_spalte])` | a quick fix (Ctrl+.) for the finding collected last: `text$` replaces the finding's place, or another one if given (columns from 1, `bis_spalte` exclusive). Several calls = several offers |
| `PLUGIN_BEFUNDE_SENDEN(datei$)` | send the collected findings of the file -- they appear in the problems list while the file is in front, with the plugin's name in front; a new send replaces the old ones, an empty send removes them |
| `PLUGIN_SCHNIPSEL(name$, kuerzel$, text$)` | a snippet (Ctrl+J, or the abbreviation + Tab); `\|` marks where the cursor goes |
| `PLUGIN_KONTEXT(id$, titel$)` | an entry in the context menu of the code field. When clicked, `befehl` arrives with this `id`, and `PLUGIN_WERT$()` is the word that was clicked (the right click moves the cursor there) |
| `PLUGIN_HOVER_ANTWORT(text$)` | answer to `hover` (after `PLUGIN_HOERE("hover")`): the text is added to the tooltip as long as the mouse still rests there; empty = nothing to add. Belonging to the event are `PLUGIN_WERT$()` (the word), `PLUGIN_ZEILE()`, `PLUGIN_SPALTE()` (from 1) and `PLUGIN_DATEI$()`. The IDE does not wait -- a plugin that does not answer does not hold up the tooltip |
| `PLUGIN_VORSCHLAEGE(eintraege)` | answer to `vervollstaendigen` (after `PLUGIN_HOERE("vervollstaendigen")`): the words (`ARRAY OF STRING`) are added to the completion list, after those of the IDE; if the IDE itself knows nothing, the answer opens the list. `PLUGIN_WERT$()` is the start of the word at the cursor, plus `PLUGIN_ZEILE()`, `PLUGIN_SPALTE()`, `PLUGIN_DATEI$()`. An answer that only arrives after more typing is dropped by the IDE |
| `PLUGIN_IDE_FASSUNG$()` | the version of the IDE that started the plugin (`"2026.36"`), from its greeting; an older IDE without a greeting gives `""` |
| `PLUGIN_PROTOKOLL()` | the version of the channel: 1 = 2026.35, 2 = 2026.36, 3 = with `PLUGIN_ERSETZEN`, `PLUGIN_AUSWAHL_SETZEN` and `PLUGIN_VORSCHLAEGE`. That way a plugin does without a newer message instead of sending one the IDE only writes to its log |
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

## Testing a plugin

`ide/dh_plugin_probe.dh` is a stand-in IDE: it starts the plugin as the IDE
does (process of its own, greeting, data folder `probe-daten/` in the
working folder), answers its questions from a state the test sets, applies
replacing and inserting to it and keeps everything the plugin sends. A
test collection (`dhrt test`) thus needs no window and no IDE:

```basic
IMPORT "dh_plugin_probe.dh"
PRINT PROBE_START("plugins/mein/plugin.dh")
PROBE_SETZE_TEXT("PRINT 1" + CHR$(10), CWD$() + "/haupt.dh", 1, 3)
PROBE_BEFEHL("zaehlen")
PRINT PROBE_WARTEN("setze")
PRINT PROBE_WIDGET$("leiste", "kopf", "text")
PROBE_ENDE()
```

| Command | What it does |
|---|---|
| `PROBE_START(plugin$ [, ide_ordner$])` | start the plugin (`ide_ordner$` = the folder with `dh_plugin.dh`, otherwise `ide/` next to the runtime); TRUE if it runs |
| `PROBE_SETZE_TEXT(text$ [, datei$, zeile, spalte])`, `PROBE_SETZE_AUSWAHL(text$)`, `PROBE_SETZE_PROJEKT(ordner$)` | the state the questions are answered from |
| `PROBE_BEFEHL(id$)`, `PROBE_EREIGNIS(name$ [, datei$])`, `PROBE_KLICK(leiste$, widget$)`, `PROBE_GEAENDERT(leiste$, widget$, wert$)` | send an event to the plugin |
| `PROBE_HOVER$(wort$, zeile, spalte)`, `PROBE_VORSCHLAEGE(wort$, zeile, spalte)` | send `hover` or `vervollstaendigen` and wait for the answer (at most 2 s) |
| `PROBE_LAUFEN(ms)`, `PROBE_WARTEN(art$ [, ms])` | process messages -- for a while, or until a message of that kind came (TRUE) |
| `PROBE_TEXT$()`, `PROBE_AUSWAHL$()`, `PROBE_MELDUNG$()`, `PROBE_AUSGABE$()`, `PROBE_GEOEFFNET$()` | the state afterwards |
| `PROBE_BEFEHLE$()`, `PROBE_KONTEXT$()`, `PROBE_LEISTEN$()`, `PROBE_HOERT$()` | what the plugin registered, separated by commas |
| `PROBE_WIDGET$(leiste$, widget$, was$)` | what it set on a widget of its panel (`text`, `checked` …, `eintraege`, `texte` as JSON) |
| `PROBE_BEFUNDE$(datei$)`, `PROBE_LETZTE$(art$)`, `PROBE_ANZAHL()` | findings of a file, the last message of a kind, all messages |
| `PROBE_ENDE()`, `PROBE_LAEUFT()` | send `ende` and wait until the plugin has quit |

The bundled plugins are tested this way (`tests/pruef/plugin_probe.dhtest`).

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
| `tue` | `was` (`einfuegen`, `meldung`, `ausgabe`, `oeffnen`, `ersetzen`, `waehlen`), `text`, `datei`, `zeile`, plus `spalte`, `bis_zeile`, `bis_spalte` |
| `befunde` | `datei`, `liste` (objects with `zeile`, `meldung`, `schwere`, optionally `spalte`, `laenge` and `korrekturen` -- each with `titel` and `aenderungen` holding `zeile`, `spalte`, `bis_zeile`, `bis_spalte`, `text`, as in `CODE_CHECK$`) |
| `hover` | `nr` (from the event), `text` |
| `vorschlaege` | `nr` (from the event), `eintraege` (array) |
| `schnipsel` | `name`, `kuerzel`, `text` |
| `faerbung` | `endung`, `muster`, `farbe` |

From the IDE to the plugin: first `hallo` (`protokoll` -- 3 today --,
`ide` = the version), then `befehl` (`id`, `wert`), `klick` and `geaendert`
(`leiste`, `widget`, `wert`), `ereignis` (`name`, `datei`), `hover` and
`vervollstaendigen` (each `nr`, `wert`, `zeile`, `spalte`, `datei`), `antwort`
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
