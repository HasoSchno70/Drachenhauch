# Editor

Der GameBasic-Editor (`gbrun.py --editor` oder einfach `gbrun.py` ohne Argument) ist eine eigenständige IDE: Syntax-Highlighting, Auto-Vervollständigung, Snippets, Built-in-Sidebar, Run/Bench, Find in Project, klickbare Fehler.

## Starten

```
.venv\Scripts\python.exe gbrun.py --editor
```

Oder ohne Argumente:

```
.venv\Scripts\python.exe gbrun.py
```

(Dann öffnet sich der Editor wenn `PySide6` installiert ist; sonst Hilfe-Anzeige.)

## Layout

```
+--+----------------------------------------------------------------+
|  | Toolbar: ➕ Neu | 📂 Öffnen | 💾 Speichern | ▶ Run | ■ Stop  ☾ |
|  +-+--------------------------------------------------------------+
|AB| Sidebar (240 px, umschaltbar via AB)                          |
|  | - Dateien                                                      |
|📂| - Struktur (Outline)        | Tabs                              |
|🧭| - Built-ins (mit Filter)    | Code-Editor       | Minimap |▮▯▮ |
|🔧|                              | - Zeilennummern   | rechts  |    |
|  |                              | - Syntax-HL       | + Scroll|    |
|  |                              | - Indent-Guides   |bar      |    |
|  |                              | - Bracket-Matching|         |    |
|  |                              | - Folding         |         |    |
|  |                              +----------------------------------+
|  |                              | Konsole (Output, klickbare Fehler)|
+--+------------------------------+----------------------------------+
| Statuszeile: Bereit | Zeile/Spalte | ● OK / ● Fehler  |  ☾ Dark   |
+--------------------------------------------------------------------+
```

**Activity Bar** (links, 48 px): drei Icon-Buttons schalten den Inhalt der Sidebar um (📂 Dateien / 🧭 Struktur / 🔧 Built-ins). **Klick auf den aktiven Button** klappt die Sidebar komplett zu — gibt dem Editor die volle Breite.

**Sidebar-Breite** ist per Maus über den Sash zwischen Sidebar und Editor ziehbar. Die zuletzt eingestellte Breite wird in `~/.gamebasic-editor/settings.json` (`sidebar_width`) gespeichert.

**Statusleiste** zeigt rechts: `Zeile / Spalte / Zeichen-Anzahl`, Encoding (`UTF-8`), `N Tabs`, OK/Fehler-Status, Theme-Indikator. Links: aktive optionale Editor-Deps (`✓ Pillow  ✓ DnD`).

**Sticky Tabs**: Optional aktivierbar über `Ansicht → Sticky Tabs umschalten` oder die Befehlspalette. Wenn an, wandert der zuletzt aktivierte Tab automatisch nach Position 0 (Most-Recently-Used-Order). Default aus, persistiert in den Settings.

**Statusleiste**: Nachrichten links, Position/Fehler/Theme rechts. Der Theme-Indikator (☾ Dark / ☀ Light) ist klickbar — toggelt das Theme.

## Tastenkürzel

### Datei

| Kürzel | Aktion |
|---|---|
| `Strg+N` | Neue Datei |
| `Strg+O` | Datei öffnen |
| `Strg+S` | Speichern |
| `Strg+Shift+S` | Speichern unter |
| `Strg+Alt+S` | **Alle Tabs speichern** (Skip für unbenannte) |
| `Strg+W` | Tab schließen |
| `Strg+Shift+T` | **Zuletzt geschlossenen Tab wiederherstellen** (LIFO-Stack) |

### Bearbeiten

| Kürzel | Aktion |
|---|---|
| `Strg+Z` | Rückgängig |
| `Strg+Y` | Wiederholen |
| `Strg+X` / `Strg+C` / `Strg+V` | Ausschneiden / Kopieren / Einfügen |
| `Strg+A` | Alles markieren |
| `Strg+#` (DE) / `Strg+/` (US) | **Kommentar umschalten** (Zeile oder Selektion mit `'` ein/aus) |
| `(`, `[`, `"` | **Auto-Pair**: schließendes Pendant wird mit eingefügt; bei Selektion wird die Selektion gewrappt; Skip-Over wenn das nächste Zeichen schon das Pair ist |
| `Backspace` zwischen Pair | **Smart-Backspace**: löscht beide Zeichen (`(|)` → ``) |
| `Tab` mit Multi-Line-Selektion | **Selektion einrücken** (alle Zeilen +4 Spaces) |
| `Shift+Tab` | **Zeile ausrücken** — mit Selektion alle Zeilen, sonst nur die aktuelle (jeweils −4 Spaces) |
| `Alt+Klick` | **Multi-Cursor**: Sekundär-Cursor an der Klick-Position hinzufügen (siehe unten) |
| `Esc` | Multi-Cursor verlassen / Autocomplete-Popup schließen |
| `Strg+=` | **Schrift größer** |
| `Strg+-` | **Schrift kleiner** |
| `Strg+0` | **Schriftgröße zurücksetzen** |
| `Strg+Shift+[` | **Block falten** (vorher `Strg+-`, kollidierte mit Zoom-Out) |
| `Strg+Shift+]` | **Alle Falten öffnen** |

### Suche / Navigation

| Kürzel | Aktion |
|---|---|
| `Strg+F` | Suchen (in aktueller Datei) — mit Toggles für Gross/Klein, Ganzes Wort, Regex |
| `Strg+H` | Ersetzen (in aktueller Datei) — Regex-aware Replace-All |
| `F2` | **Symbol umbenennen** — alle Whole-Word-Vorkommen des Identifiers am Cursor |
| `F11` | **Zen-Mode** — Toolbar/Sidebar/Konsole/Statusbar verstecken, nochmal F11 zurück |
| `Strg+Shift+W` | **Word-Wrap umschalten** |
| `Strg+Shift+F` | **Im Projekt suchen** (alle .gb-Dateien) |
| `Strg+G` | Gehe zu Zeile |
| `Strg+Tab` / `Strg+PageDown` | **Nächster Tab** (mit Wrap-Around) |
| `Strg+Shift+Tab` / `Strg+PageUp` | **Vorheriger Tab** (mit Wrap-Around) |
| `F8` | **Nächster Fehler** (springt zum nächsten roten Gutter-Marker, mit Wrap) |
| `Shift+F8` | **Voriger Fehler** |
| `F12` | **Gehe zur Definition** — sucht erst im aktuellen Buffer, dann in `IMPORT`-Dateien |
| `Strg+Klick` | Wie F12, aber per Maus |
| `Alt+F12` | **Peek-Definition** — zeigt Signatur, Doc-Kommentar und den Anfang des Bodys in einem Popup am Cursor, ohne wegzuspringen. `Enter`/Button springt doch hin, `Esc` schließt |
| `Shift+F12` | **Find-References** — listet alle Vorkommen in der Konsole (klickbar) |
| `Strg+Leertaste` | Auto-Vervollständigung manuell auslösen. Beim Tippen öffnet sich das Popup auch automatisch (siehe unten); `↑`/`↓` navigiert, `Enter`/`Tab` committed, `Esc` oder Klick außerhalb schließt |
| `Strg+Shift+P` | **Befehlspalette** (siehe unten) |
| `Strg+P` | **Quick-Open** — Fuzzy-Search durch alle `.gb`-Dateien im Projekt |
| `Strg+D` | **Nächstes Vorkommen** — selektiert das Wort am Cursor + highlightet alle Vorkommen, weiteres `Strg+D` springt zum nächsten Treffer (mit Wrap) |
| `Strg+-` | Block falten |
| `Strg++` | Alles entfalten |

### Ausführen

| Kürzel | Aktion |
|---|---|
| `F5` | Programm starten — **native Runtime `gbrt`**, mit automatischem Fallback auf den Tree-Walker, wenn `gbrt` nicht ausführen kann (nicht gebaut / Compile- oder Start-Fehler) |
| Toolbar `Bench` | Vergleicht Tree-Walker-Output mit gbrt-Output |
| Toolbar `Stop` | Laufendes Programm abbrechen (auch den nativen `gbrt`-Prozess) |

## Befehlspalette (`Strg+Shift+P`)

Wie in VSCode/Cursor: ein Toplevel mit Eingabefeld + scrollbarer Liste aller verfügbaren Aktionen. Live-Filter mit Fuzzy-Match — du tippst „spei" und siehst sofort `Datei: Speichern`, `Datei: Speichern unter ...`. Pfeil-↑/↓ navigieren, **Enter** führt aus, **Escape** schließt.

In der Palette sind alle Menü-Aktionen plus **alle Doku-Files** zum direkten Aufruf — tippe `doku json` und du landest in einem Klick im JSON-Modul-Doc.

## Auto-Vervollständigung beim Tippen

Sobald du **mindestens 2 Buchstaben** eines Identifiers tippst, öffnet sich das Vorschlags-Popup automatisch — ohne `Strg+Leer`. Es filtert live während du weitertippst.

**Vorgeschlagen werden:**
- Eigene Variablen aus `DIM` und `CONST`-Deklarationen (markiert mit `[Var]` / `[Const]`)
- Parameter von `SUB` und `FUNCTION` (`[Param]`) — auch innerhalb der Funktion sichtbar
- Eigene Funktionen, Subs, Klassen, Structs (`[Fn]` / `[Sub]` / `[Class]` / `[Struct]`)
- Alle Built-ins (Standard, Grafik, Module — ohne Suffix, gold gefärbt)
- Konstanten (Farben, Tasten — grün) und Keywords (blau)

User-Identifier ranken vor gleichnamigen Built-ins. Pro Kategorie eine eigene Farbe — auf einen Blick erkennbar, ob ein Vorschlag deine Variable, ein Built-in oder ein Keyword ist.

**Member-Vorschläge nach `.`:** Wenn du `objekt.` tippst und `objekt` per `DIM x AS Klasse` deklariert wurde, listet das Popup automatisch die **Felder und Methoden** der Klasse auf. Auch für `STRUCT` und Multi-DIM (`DIM a, b AS Player`).

**Wann es nicht aufpoppt:**
- Du bist gerade in einem `"…"`-String oder `'…`-Kommentar
- Das Wort fängt mit einer Ziffer an (Zahlen sind keine Identifier)
- Du hast es gerade per `Esc` weggeklickt — bleibt zu, bis du einen Nicht-Identifier-Char tippst (Space, Symbol, Zeilenwechsel)

**Bedienen:**
- `↑`/`↓` navigiert in der Liste
- `Enter` oder `Tab` committed (ersetzt den Prefix durch den gewählten Eintrag)
- `Esc` oder Klick irgendwo im Editor schließt
- Doppelklick auf einen Eintrag committed

**Abschalten:** `Bearbeiten → Auto-Vervollst. beim Tippen umschalten` (oder Befehlspalette). Das Setting wird in `~/.gamebasic-editor/settings.json` gespeichert. `Strg+Leer` funktioniert weiterhin, auch wenn der Auto-Trigger aus ist.

## Signature-Help / Parameter-Hints

Sobald du einen Funktionsaufruf öffnest — `LINE(` — erscheint über der Cursor-Zeile ein dezentes Popup mit der **Signatur**. Der gerade aktive Parameter ist **fett + in Akzentfarbe** hervorgehoben und wandert mit, während du Argumente und Kommas tippst (`LINE(10, 20, ▮` → `x2` ist aktiv).

- Funktioniert für **Built-ins** (benannte Signatur, z. B. `LINE(x1, y1, x2, y2[, farbe])`) **und** für eigene `SUB`/`FUNCTION` im Buffer (volle Signatur mit Param-Namen und Typen).
- Optionale Parameter werden als `[, farbe]` gezeigt; verschachtelte Aufrufe lösen auf den **innersten** Aufruf auf.
- Blendet sich automatisch aus: außerhalb einer Argumentliste, in `"…"`-Strings / `'…`-Kommentaren, bei `Esc`, beim Scrollen oder Fokuswechsel. Das Vervollständigungs-Popup hat Vorrang.

## Color-Picker (Swatch-Klick)

Color-Literale (`&HRRGGBB` oder `RGB(r, g, b)`) bekommen rechts daneben ein kleines Farbquadrat (Swatch). **Klick auf den Swatch** öffnet den Farbwähler; die gewählte Farbe ersetzt das Literal im selben Format. So tunst du Farben visuell, ohne Hex-Werte im Kopf zu rechnen.

## Bookmarks

Schnell-Navigation in langen Dateien:

| Aktion | Kürzel |
|---|---|
| Bookmark setzen / entfernen | `Ctrl+F2` |
| Nächstes Bookmark | `F9` |
| Vorheriges Bookmark | `Shift+F9` |

Bookmarks werden als schmaler Mint-Balken am linken Gutter-Rand markiert; die Navigation läuft zyklisch (nach dem letzten geht's wieder zum ersten).

## TODO / FIXME-Liste

`Ctrl+Shift+M` (oder *Ansicht → TODO / FIXME-Liste*) blendet ein Panel rechts ein, das die aktuelle Datei nach Markern in Kommentaren durchsucht — `TODO`, `FIXME`, `HACK`, `XXX`, `BUG` (hinter `'` oder `REM`; Treffer in Strings werden ignoriert). Farbcodiert nach Typ; **Klick springt zur Zeile**. Aktualisiert sich beim Tippen und Tab-Wechsel.

## Zeilenumbruch

`Alt+Z` (oder *Ansicht → Zeilenumbruch*) schaltet weichen Zeilenumbruch um — lange Zeilen brechen am Fensterrand statt horizontal zu scrollen. Die Einstellung gilt für alle Tabs und bleibt erhalten.

## Smart-Outdent

Sobald du eine Zeile schreibst, die genau einem block-schließenden oder
block-fortsetzenden Schlüsselwort entspricht, korrigiert der Editor das Indent
automatisch zur passenden Block-Eröffnung:

```
IF x THEN
    PRINT 1
    END IF       ← wird beim Tippen automatisch zu 'END IF' (kein Indent)
```

Erkannte Trigger: `END IF`, `END SUB`, `END FUNCTION`, `END CLASS`, `END STRUCT`,
`END SELECT`, `END TRY`, `WEND`, `NEXT`, `UNTIL`, `ELSE`, `ELSEIF …`, `CASE …`,
`CASE ELSE`, `CATCH …`.

Die zugehörige Block-Eröffnung wird über einen Tiefen-Counter gefunden — bei
verschachtelten Blöcken landet das Token also korrekt auf der äußeren oder
inneren Ebene. `CASE` im `SELECT CASE` rückt konventionsgemäß +4 vom `SELECT`
ein, alle anderen sind auf der gleichen Ebene wie ihr Opener.

Wenn der Indent bereits korrekt ist, passiert nichts — die Funktion ist
idempotent und stört keine bewusste manuelle Einrückung.

## Snippets (Tab-Trigger)

Tippe einen Trigger und drück `Tab` — der Editor expandiert ihn zu einem Skelett. Trigger müssen am Zeilen-Anfang (nach evtl. Indent) stehen.

| Trigger | Expandiert zu |
|---|---|
| `if`  | `IF \| THEN ... END IF` |
| `ife` | `IF/THEN/ELSE/END IF` |
| `for` | `FOR i = 1 TO ... NEXT` |
| `wh`  | `WHILE ... WEND` |
| `sc`  | `SELECT CASE ... END SELECT` |
| `sub` | `SUB ... END SUB` |
| `fn`  | `FUNCTION ... END FUNCTION` |
| `cls` | `CLASS ... END CLASS` |
| `try` | `TRY ... CATCH ... END TRY` |
| `gl`  | Game-Loop-Skelett (`WHILE NOT QUITREQUESTED ... CLS ... FLIP ... WEND`) |
| `scr` | `SCREEN(320, 240, "...", 2)` |
| `imp` | `IMPORT "..."` |

`|` im Template markiert die Cursor-Position nach Insertion.

`Hilfe → Snippets ...` zeigt die Liste im Editor.

## Built-ins-Sidebar

Wird über die **Activity Bar** (🔧) eingeblendet. Listet alle Built-ins aus Standard, Grafik und allen Modulen, gruppiert nach Modul:

```
STANDARD     (Math, Strings, Maps, ...)
GRAFIK       (SCREEN, BOX, CIRCLE, ...)
BT           (Bluetooth LE)
CAMERA
DB           (SQLite)
IMGFX
JSON
PARTICLES
SERIAL       (RS-232 / USB-COM)
SPRITE
TWEEN
UI
USB          (HID)
WIFI         (Windows-only)
```

- **Aufklappbar pro Modul**: Klick auf den Pfeil ▶ vor einem Modulnamen klappt die Liste auf/zu. Hinter dem Modulnamen steht in Klammern die Anzahl Built-ins. Default sind alle Module zugeklappt — Übersicht über die verfügbaren Module zuerst.
- **Suchfeld oben** filtert live: `json` zeigt nur JSON-Befehle, `_get_` alle Getter quer durch alle Module. Während der Filter aktiv ist, werden alle matchenden Module automatisch aufgeklappt; der manuell gesetzte Auf-/Zu-Zustand wird zurückgespielt sobald der Filter geleert wird.
- **Doppelklick** auf einen Eintrag fügt den Namen am aktuellen Cursor in die Datei ein.
- **Signatur in Klammern** — z.B. `JSON_GET_STRING(h, path: STRING)` — zeigt direkt, was die Funktion erwartet. Lange Signaturen sind über die horizontale Scrollbar erreichbar.

## Breadcrumbs (Scope-Pfad)

Schmale Leiste direkt über dem Editor: zeigt, wo der Cursor gerade steckt — z. B. `game.gb  ›  C Player  ›  ▸ Init`, wenn er in der Methode `Init` der Klasse `Player` steht. Jedes Segment ist anklickbar und springt zur jeweiligen Definitionszeile (Dateiname → Zeile 1). Aktualisiert live beim Tippen und Cursor-Bewegen; auf Top-Level zeigt sie nur den Dateinamen. Gespeist aus demselben Scope-Scanner wie die Outline.

## Outline (Struktur-Sidebar)

Activity Bar 🧭. Zeigt alle `SUB`/`FUNCTION`/`CLASS`/`STRUCT`-Deklarationen der aktiven Datei, Klick springt zur Zeile. Hat die Datei keine Strukturen, erscheint ein Hinweis statt einer leeren Liste.

**Filter-Eingabefeld** oben — bei vielen Deklarationen schnell Substring-suchbar (`init` → alle `Init`-Subs/Methoden).

## Minimap

Rechts vom Code, vor der Scrollbar, blendet eine Vorschau der gesamten Datei ein. Pro Zeile eine kurze farbige Linie:

- **Akzentfarbe** für Deklarations-Zeilen (`SUB`, `FUNCTION`, `CLASS`, `STRUCT`, `ENUM`, `IMPORT`, `CONST`)
- **Comment-Farbe** für Kommentar-Zeilen (`'…` oder `REM`)
- **Vordergrund-Farbe** für normalen Code

Ein durchscheinendes Rechteck markiert den aktuellen Sichtbereich. Klick oder Drag in der Minimap scrollt den Editor zur entsprechenden Zeile (geklickte Zeile wird mittig ausgerichtet). Mausrad scrollt synchron zum Editor.

Bei sehr langen Dateien (> 8000 Zeilen) wird die Auflösung automatisch reduziert (jede n-te Zeile gerendert), damit der Render unter ~50 ms bleibt.

Per `CodeEditor.set_minimap_visible(False)` lässt sich die Minimap programmatisch ausblenden — sie wird komplett aus dem Layout entfernt, der Editor bekommt dann den freigewordenen Platz.

## Multi-Cursor

Mehrere Caret-Positionen gleichzeitig — für parallele Edits:

- **`Alt+Klick`** fügt einen Sekundär-Cursor an der Klick-Position hinzu. Der primäre Cursor bleibt unverändert.
- **Tippen** fügt das Zeichen an *allen* aktiven Cursorn ein.
- **`Backspace`** löscht je ein Zeichen rückwärts an *allen* Cursorn.
- **`Esc`** oder **normaler Klick** verlässt den Multi-Cursor-Modus und kehrt zu einem einzelnen Cursor zurück.

Die zusätzlichen Cursorn werden als kleine farbige Blöcke (Akzentfarbe) auf dem Zeichen rechts der Cursor-Position dargestellt — Tk hat nur einen echten Caret, alles weitere ist eine Tag-basierte Simulation.

**Bewusst ausgespart** (zu fragil bei Tk-Constraints):

- Undo/Redo: Nutzt Tks normalen Undo-Stack — nach Multi-Edits ist das Verhalten nicht garantiert konsistent. Bei Bedarf einfach `Esc` und mit Single-Cursor weiterarbeiten.
- Pfeiltasten / Selektions-Erweiterung über alle Cursorn — `Esc` und neu starten ist robuster.
- Multi-Cursor + Find/Replace.

## Sticky Sub/Function-Header

Beim Scrollen durch eine lange `SUB`/`FUNCTION`/`CLASS` bleibt die Header-Zeile als Sticky-Label oben am Editor sichtbar — du verlierst nie den Kontext, in welcher Funktion du gerade bist. Klick auf das Sticky-Label springt zurück zum Header.

## Diff-Indicator im Gutter

Zeilen, die seit dem letzten Save geändert oder neu hinzugefügt wurden, werden links neben der Zeilennummer mit einem dünnen blauen `▎` markiert. Nach `Strg+S` verschwinden die Marker. Diff wird über `difflib.SequenceMatcher` berechnet (200 ms Debounce nach jedem Tippen) — Einfügungen verschieben nicht alle nachfolgenden Zeilen als „geändert".

## Hover-Tooltips

Maus über jeden Identifier:
- **Standard-Built-ins**: kuratierte deutsche Doku (`SIN(zahl) - Sinus von zahl`).
- **Modul-Built-ins**: automatisch generierte Signatur (`JSON_PARSE(s: STRING)`).
- **Eigene Variablen/Funktionen**: zeigt Typ und Deklarationszeile (`held: SPRITE (deklariert in Zeile 73)`).
- **Schlüsselwörter**: `WHILE - Keyword`.
- **Fehler-Stellen**: `[X]` + Fehlermeldung.

## Konsole — PRINT, INPUT, Kontextmenü

`PRINT`-Ausgaben des laufenden Programms erscheinen direkt in der Konsole.

`INPUT`-Statements lesen Zeilen aus dem **Eingabefeld** unter der Konsole. Das Feld ist nur aktiv, solange ein Programm läuft. Tippe deine Antwort, drücke `Enter` — die Zeile geht an `stdin` des laufenden Programms und wird in der Konsole als Echo (in Akzentfarbe) angezeigt.

Beispiel:

```basic
DIM name AS STRING
INPUT "Wie heisst du? ", name
PRINT "Hallo, " + name + "!"
```

Auch wenn dein Programm parallel ein Pygame-Fenster (`SCREEN(...)`) öffnet, funktioniert INPUT weiter über die Editor-Konsole — nützlich für Debugging und einfache textuelle Programme ohne Grafik.

**Rechtsklick** in der Konsole öffnet ein Kontextmenü:
- Kopieren (markierten Text in die Zwischenablage)
- Alles auswählen
- Konsole leeren

**Rechtsklick im Editor** öffnet ein Kontextmenü mit Ausschneiden / Kopieren / Einfügen / Alles markieren / Gehe zur Definition / Symbol umbenennen — Cursor wird vorher an die Klick-Position gesetzt, damit z. B. „Gehe zur Definition" auf das angeklickte Wort wirkt.

## Klickbare Fehler in der Konsole

Wenn ein Programm einen Fehler wirft, erkennt die Konsole automatisch:

| Pattern | Sprung |
|---|---|
| `[Zeile NN]` | aktuelle Run-Datei, Zeile NN |
| `Fehler in DATEI.gb:` | DATEI.gb (Zeile 1) |
| `DATEI.gb:NN` | DATEI.gb, Zeile NN (z.B. nach IMPORT) |

Hover über einen Link zeigt Hand-Cursor; **Klick** öffnet die Datei und springt zur Zeile.

## Run / Bench / Stop

- **`Run` (F5)**: führt das Programm über **einen** Button aus — **primär die native Runtime `gbrt`** (kompiliert die Datei in eine temporäre `.gbc` und startet `gbrt` direkt als Prozess, sodass `Stop` ihn beendet; Arbeitsverzeichnis = Ordner der Quelldatei, relative Asset-Pfade; Laufzeitfehler als klickbares `datei.gb:Zeile`). Kann `gbrt` nicht ausführen — **nicht gebaut**, **Compile-Fehler** oder **Start-Fehler** —, fällt der Run **automatisch auf den Tree-Walker** zurück (ein Hinweis erscheint in der Konsole). So genügt ein Knopf, ohne dass man manuell zwischen Runtimes wählt. (Hintergrund: [docs/rust-runtime.md](rust-runtime.md).)
- **`Bench`** (Ctrl+F5): führt `gbrun.py --bench <datei>` aus — vergleicht den stdout-Output von Tree-Walker und `gbrt` (nur deterministische Konsolen-Programme).
- **`Stop`** (Shift+F5): bricht den laufenden Prozess ab (für Game-Loops, die nicht von selbst enden).

**Run-Indikator:** Bei mehreren offenen Tabs markiert der Editor den Tab der gerade laufenden Datei — Präfix **⚙** (native Runtime `gbrt`) bzw. **▶** (Tree-Walker-Fallback) plus Akzent-Textfarbe. Die Statusleiste zeigt zusätzlich Datei + Modus. Beim Run-Ende wird die Markierung zurückgesetzt.

## Debugger

Der Editor hat einen **Tree-Walking-Debugger** mit Breakpoints, Einzelschritt und Variablen-Inspektion. Er läuft auf dem Python-Interpreter (Referenz-Pfad) — die native Runtime `gbrt` hat keinen Debug-Kanal. Am besten für Konsolen-/Logik-Programme geeignet.

**Bedienung:**

| Aktion | Kürzel |
|---|---|
| Breakpoint setzen/entfernen | Klick im **linken Gutter-Band** (roter Punkt) |
| Debuggen starten | `F7` (Toolbar-Käfer / Menü *Debug*) |
| Fortsetzen (bis nächster Breakpoint) | `F8` |
| Step Over (Zeile, ohne in Aufrufe zu springen) | `F10` |
| Step Into (in `SUB`/`FUNCTION` hinein) | `F11` |
| Step Out (aus der aktuellen Funktion heraus) | `Shift+F11` |
| Debug stoppen | Menü *Debug → Debug stoppen* |

**Ablauf:** Setz einen oder mehrere Breakpoints, drück `F7`. Das Programm läuft bis zum ersten Breakpoint (oder hält am ersten Statement, wenn keiner gesetzt ist). Die **aktuelle Zeile** wird mit einem ▶-Pfeil im Gutter und einem Zeilen-Highlight markiert; das Panel **Variablen** (rechts) zeigt die lokalen und globalen Variablen (Name / Wert / Typ) — nur deine eigenen, die eingebauten Konstanten (`BLACK`, `KEY_*`, `PI` …) sind ausgeblendet. `PRINT`-Ausgabe landet in der Konsole.

**Grenzen:** `INPUT` liefert im Debugger EOF (kein Hängen). Grafik-Programme laufen, aber das Schrittweise durch eine 60-fps-Schleife ist unpraktisch — Breakpoints in Init-/Logik-Code funktionieren trotzdem. Während einer Debug-Sitzung sind Run/Bench deaktiviert.

## Profiler

`Strg+Shift+Y` (oder Toolbar-Button / `Debug → Profiler`) führt das Programm im Tree-Walker durch und misst pro Statement **Trefferzahl** und **Zeit**. Das Ergebnis erscheint im **Profiler**-Panel (rechts):

- **Zeilen** — jede ausgeführte Zeile mit Treffern, Zeit (ms), Anteil (Balken + %) und Quelltext, nach Zeit sortiert (Hotspots oben). Doppelklick springt zur Zeile.
- **Funktionen** — pro `SUB`/`FUNCTION` aggregiert: Aufrufzahl + Gesamtzeit.

**Naeherung:** Die Zeit einer Zeile enthält die in aufgerufenem Code (Built-ins, `IMPORT`) verbrachte Zeit sowie den Mess-Overhead — gut für die *relative* Hotpath-Einordnung, nicht als absolute Zeit. Wie der Debugger nur Tree-Walker, am besten für Konsolen-/Logik-Programme; ein laufender Profiler-Lauf lässt sich mit erneutem `Strg+Shift+Y` abbrechen (Endlos-Loops). Kern + Aggregation liegen in `gamebasic/editor_qt/profiler.py` (headless getestet: `tests/test_profiler.py`).

## Git-Blame

`Strg+Shift+A` (oder `Ansicht → Git-Blame`) öffnet das **Blame**-Panel: pro Zeile der Datei, wer sie zuletzt geändert hat — Zeile, Commit (Kurz-Hash), Autor, Datum, Commit-Zusammenfassung. Die Tabelle **folgt dem Cursor** (die aktuelle Zeile wird markiert), **Doppelklick** springt im Editor zur Zeile. `Aktualisieren` lädt neu.

Lokale, noch nicht committete Zeilen sind als `•••` / *(uncommitted)* ausgegraut. Bei ungespeicherten Änderungen zeigt das Panel den zuletzt gespeicherten Stand (Hinweis im Status). Ist die Datei nicht in einem Git-Repository oder `git` nicht installiert, erscheint eine entsprechende Meldung statt einer leeren Tabelle. Implementierung: `gamebasic/editor_qt/gitinfo.py` (Porcelain-Parser von der I/O getrennt, headless getestet: `tests/test_gitinfo.py`).

## Finden & Ersetzen im Projekt

`Strg+Shift+F` öffnet den Dialog. Tippe ein Pattern, `Enter` startet die Suche über alle `.gb`-Dateien rekursiv. Die Suche läuft im Hintergrund-Thread und liefert Treffer **inkrementell** in die Liste — der Editor friert auch bei großen Projekten nicht ein. Mit dem `Stop`-Button lässt sich die Suche jederzeit abbrechen.

Optionen:
- **Gross/Klein** — case-sensitive matchen
- **Ganzes Wort** — Treffer nur an Wortgrenzen (`\bfoo\b`)
- **Regex** — Query als Python-Regex interpretieren; bei Syntaxfehler erscheint ein Hinweis im Status

Treffer-Liste zeigt `pfad/datei.gb:42  code-snippet`. **Doppelklick** öffnet den Treffer.

**Ersetzen:** Im Feld **Ersetzen durch** den Ersatztext eingeben (leer = löschen) und **Alle ersetzen** klicken. Ein Bestätigungsdialog nennt die Anzahl Vorkommen und betroffener Dateien, bevor geschrieben wird. Die Optionen (Gross/Klein, Ganzes Wort, Regex) gelten auch fürs Ersetzen; im Regex-Modus sind Rückverweise (`\1`, `\g<1>`) im Ersatztext erlaubt, sonst wird er literal eingesetzt. **Offene Tabs werden mit ihren ungespeicherten Änderungen berücksichtigt** — der Ersatz wird auf den Live-Buffer angewendet, auf die Platte geschrieben und der Tab synchron gehalten (kein Datenverlust). Implementierung/Tests: `gamebasic/editor_qt/find_in_project.py`, `tests/test_find_in_project.py`.

`.venv/`, `build/`, `dist/`, `__pycache__/` werden ausgespart.

## Code-Folding

`Strg+Shift+[` faltet den Block, in dem der Cursor steht (oder den der Cursor beginnt: `IF`, `FOR`, `WHILE`, `SUB`, `FUNCTION`, `CLASS`, `STRUCT`).

`Strg+Shift+]` entfaltet alles. (Vorher `Strg+-` / `Strg++` — die Tasten gehören jetzt zum Font-Zoom.)

In der Zeilennummern-Spalte zeigt:
- `▾` = aufklappbarer Block-Anfang (klickbar)
- `▶` = gefalteter Block

## Theme-Wechsel

Drei Wege:
- `Ansicht → Theme → Dark / Hell` im Menü
- **Klick auf den Theme-Indikator** in der Toolbar (oben rechts) oder in der Statusleiste (unten rechts) — toggelt
- Befehlspalette: „Theme umschalten", „Theme: Dark", „Theme: Hell"

Beide Themes sind VSCode-orientiert (Dark+ und Light+), Akzentfarbe ist ein modernes Blau-500. Wahl wird in `~/.gamebasic-editor/settings.json` gespeichert.

## Dokumentation aufrufen

`Hilfe → Dokumentation` zeigt alle `docs/*.md`-Files als Untermenü — Klick öffnet das File mit dem System-Default (Markdown-Viewer / Browser / Editor je nachdem was registriert ist).

Schneller geht's per Befehlspalette: `Strg+Shift+P`, dann „doku" tippen — alle Doc-Files erscheinen sofort gefiltert.

Der **Markdown-Viewer** rendert die Doku theme-treu (helle Schrift auf dunklem Grund) und bietet zwei Navigationshilfen:

- **Inhaltsverzeichnis** (links, Toggle `☰ Inhalt`): aus den Überschriften des Dokuments aufgebaut und nach Ebene eingerückt — Klick springt zur Stelle. Bei Dokumenten ohne Überschriften ausgeblendet.
- **Suche** (`Strg+F`): Suchleiste mit `Weiter`/`Zurück`, Wrap-Around und Treffer-Zähler; **alle** Treffer werden hervorgehoben. `Esc` schließt die Leiste.

## Branding (Window-Icon + About-Dialog)

Das GameBasic-Logo aus `gamebasic/assets/logo.png` wird automatisch:
- als **Fenster-/Taskleisten-Icon** verwendet (quadratischer Smart-Crop, 64 × 64)
- im **About-Dialog** angezeigt — `Hilfe → Ueber GameBasic` oder Befehlspalette `Ueber GameBasic`

Voraussetzung ist das optionale Paket `Pillow`:

```
.venv\Scripts\python.exe -m pip install Pillow
```

Wenn Pillow oder die Logo-Datei fehlt, startet der Editor unverändert mit dem Default-Tk-Icon.

## Drag-and-Drop

`.gb`-Dateien aus dem Datei-Explorer lassen sich direkt ins Editor-Fenster
ziehen — pro Datei wird ein Tab geöffnet. Andere Endungen werden ignoriert.
Funktioniert ohne zusätzliches Paket (Qt-natives `dragEnterEvent`/`dropEvent`).

## Quick-Open (`Strg+P`)

VSCode-Pattern: `Strg+P` öffnet einen Fuzzy-Finder über alle `.gb`-Dateien im Projekt. Tippe Teile des Datei-Namens oder Pfads — die Liste filtert live (gleiche Fuzzy-Logik wie die Befehlspalette). Pfeil-Up/Down navigiert, Enter öffnet den gewählten Tab.

`.venv/`, `build/`, `dist/`, `__pycache__/` werden ausgespart.

## Welcome-Panel & Showcase

Startet der Editor ohne offene Datei, zeigt er ein **Welcome-Panel** (kein leerer Tab): Logo, Action-Buttons (Neu / Öffnen / Beispiele / Doku) und eine **Showcase-Galerie** — kuratierte Demos als anklickbare Karten mit echtem Screenshot-Thumbnail, Titel und Kurzbeschreibung (3D/PBR/IBL, Demoscene, Partikel, Platformer …). Klick auf eine Karte öffnet die Demo. Darunter die Liste der zuletzt geöffneten Dateien. Sobald eine Datei geöffnet oder ein neuer Tab angelegt wird, verschwindet das Welcome.

Die Galerie-Liste ist die Single-Source-of-Truth in `gamebasic/editor_qt/showcase.py`. Die Thumbnails liegen unter `examples/screenshots/` und werden per `tools/gen_showcase_thumbs.py` erzeugt (kompiliert jede Demo und zieht über `gbrt` headless — `GBRT_FRAMES` + `GBRT_SCREENSHOT` — einen Screenshot). Fehlt ein Thumbnail, zeigt die Karte einen Play-Glyph-Platzhalter.

## Explorer: Beispiele nach Kategorie

Der Datei-Explorer gruppiert die flache `examples/`-Sammlung (100+ Dateien) virtuell nach Kategorie (Benchmarks, 3D & Rendering, Spiele, Module, Grafik & Demos, Sprache & Grundlagen, Weitere) — rein für die Anzeige, es werden keine Dateien verschoben. Andere Verzeichnisse behalten den normalen Ordner-Baum. Das Filterfeld blendet passende Kategorien auf.

## Workspace-Persistenz

Beim Schließen des Editors werden alle offenen Tabs (mit Pfad) und der aktive Tab in `~/.gamebasic-editor/settings.json` gespeichert. Beim nächsten Start öffnet der Editor sie automatisch wieder.

Reihenfolge beim Start:

1. Wenn `gbrun.py --editor <datei.gb>` mit Pfad-Argument: nur diese Datei wird geöffnet (überschreibt Workspace-Restore).
2. Sonst: Auto-Save-Recovery — wenn ungesicherte Inhalte aus der vorigen Session vorhanden sind, fragt der Editor, ob sie wiederhergestellt werden sollen (siehe „Auto-Save").
3. Sonst: Workspace-Restore — alle Tabs aus dem letzten Lauf werden wieder geöffnet.
4. Sonst: ein leerer Default-Tab.

Verschollene Pfade (Datei wurde gelöscht / verschoben) werden beim Restore still übersprungen.

## Settings & Recent Files

- **Recent Files** unter `Datei → Zuletzt geoeffnet` (Liste der letzten 10).
- **Theme** wird persistent gespeichert.
- Settings-Datei: `~/.gamebasic-editor/settings.json`.

## Performance

Der Editor verarbeitet auch große Dateien (mehrere tausend Zeilen) flüssig:

- **Inkrementelles Syntax-Highlighting**: bei jedem Tastendruck wird nur die geänderte Zeilenrange neu gelext und retaggt — bei einem 2000-Zeilen-Buffer sind das ca. 0,04 ms pro Tastendruck statt ~34 ms für einen vollen Re-Highlight (~750 ×). Ein Full-Highlight läuft nur bei `Datei laden`, `Theme-Wechsel`, `Snippet-Expansion`, `Find/Replace-Replace-All`, `Symbol umbenennen` und nach `Cut`/`Paste`.
- **Asynchroner Live-Errorcheck**: der Parser läuft in einem Worker-Thread, ein Generation-Counter verwirft veraltete Resultate.
- **Tag-Batching für Indent-Guides**: alle Spalten-Marker werden in einem einzigen `tag_add`-Call übergeben statt N einzelner Calls.

GameBasic-Tokens enden am Zeilenende (keine Multi-Line-Strings/-Kommentare), darum ist jede Zeile tokenmäßig unabhängig — das macht das inkrementelle Highlighting korrekt ohne Span-Tracking.

## Bekannte Eigenheiten

- **`step` ist Schlüsselwort** (FOR…STEP) — Variablen anders benennen (`i`, `tick`, `iter`).
- **`gbrun.py --editor` braucht `PySide6`**: bei fehlender Installation kommt eine klare Fehlermeldung. Installation: `pip install PySide6`.
- **Pygame-Beispiele blockieren** den Editor während sie laufen — das ist normal, der Run-Prozess ist getrennt vom Editor-Prozess. ESC oder Fenster-X im Pygame-Fenster beendet das Programm.
