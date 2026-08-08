# Form-Designer (WYSIWYG, Xojo-Stil)

Visueller GUI-Designer für GameBasic — Controls per Klick platzieren, im
Inspector konfigurieren, als `.gbform` speichern und mit den `gui`-Builtins zur
Laufzeit laden. Sprache der Logik bleibt GameBasic.

**Start:** `gbform [datei.gbform | projekt.gbproj]` (bzw. `gbrun.py --form`).
Benötigt PySide6. Alternativ `gb` (oder `gbrun.py`) **ohne Argument** →
Auswahl-Dialog *Code-Editor* / *Form-Designer*. `gbedit` öffnet direkt den
Code-Editor.

## Multi-Form-Projekte

Der Designer hält **mehrere Formulare** gleichzeitig offen — jedes mit eigenem
Pfad, eigener Undo-Historie und eigenem Dirty-Status (Undo läuft nie über
Formulare hinweg). Der Navigator links wechselt zwischen ihnen.

- **Neues Formular** (`Strg+N`), **Formular öffnen…** (`Strg+O`, fügt ein
  bestehendes `.gbform` zum Projekt hinzu), **Formular schließen** (`Strg+W`).
- **Speichern** (`Strg+S`) / **Speichern unter…** (`Strg+Umschalt+S`) betrifft das
  aktive Formular; **Alle speichern** (`Strg+Alt+S`) alle offenen.
- **Projekt** = eine `.gbproj`-Manifestdatei, die die zugehörigen `.gbform`-Pfade
  (relativ) und das **Startformular** auflistet. *Projekt speichern…* sichert alle
  Formulare + das Manifest; *Projekt öffnen…* lädt den ganzen Satz. *Als
  Startformular setzen* markiert das aktive Formular als `main`.

## Aufbau (wie Xojo)

- **Links — Formulare + Controls:** oben der **Formular-Navigator** (alle im
  Projekt geöffneten Formulare; Klick wechselt, `*` = ungespeichert, `★` =
  Startformular), darunter die **grafische Palette** aller Widget-Arten (Button,
  Label, Checkbox, Radio, Slider, TextInput, Dropdown, ListBox, ProgressBar,
  Image, **Tabelle**, Canvas, Panel, GroupBox, Separator) — jeder Eintrag mit
  Mini-Vorschau-Icon. Platzieren auf
  zwei Wegen: **per Drag&Drop** auf die Fläche ziehen, **oder** Eintrag anklicken
  („scharf") und auf die Fläche klicken.
- **Mitte — Design-Fläche:** das Formular, mit **realistisch gerenderten Controls**
  (Cyan-Theme, wie zur Laufzeit). Control anklicken = auswählen, ziehen =
  verschieben, an den 8 **Resize-Griffen** ziehen = Größe ändern, `Entf` = löschen.
  **Mehrfach-Auswahl:** `Strg`+Klick togglet einzelne Controls, ein **Auswahlrahmen**
  (Ziehen im leeren Bereich) fasst alle berührten zusammen; eine Gruppe lässt sich
  gemeinsam ziehen, nudgen und löschen. **Pfeiltasten** verschieben pixelweise
  (`Umschalt`+Pfeil = ein Rasterschritt), **Rechtsklick** öffnet ein Kontextmenü.
  Beim Ziehen erscheinen **Ausrichtungs-Hilfslinien** (Snap an Kanten/Mitten anderer
  Controls + Formularränder). Bewegen/Platzieren/Resizen rasten am **8-px-Raster**
  ein (Toggle `Ansicht → Am Raster ausrichten`, `Strg+G`). **Zoom** über `Strg`+`=`/
  `-`/`0` oder `Strg`+Mausrad (0,25×–4×). Die **Statusleiste** zeigt Position + Größe
  (bzw. Anzahl) der Selektion sowie die Zoom-Stufe.
- **Die Fläche um das Formular** ist eine eigene Arbeitsfläche: leichter Verlauf,
  bewusst **heller** als die Panels ringsum und als beide Formular-Themen, damit
  ein **weicher Schlagschatten** das Formular sichtbar darauflegt. (Auf einem
  fast schwarzen Grund hätte ein schwarzer Schatten keinen Spielraum — er war
  dort messbar unsichtbar.) Das **Raster** mischt sich aus Fenster- und
  Schriftfarbe des gewählten Themas, ist also in hellen wie dunklen Formularen
  eine gleich dezente Andeutung; unter 0,5× Zoom entfällt es, weil die Punkte
  dann als Rauschen lesen.
- **Bearbeiten-Menü:** **Undo/Redo** (`Strg+Z` / `Strg+Y`, auch `Strg+Umschalt+Z`)
  — eine Geste (Platzieren, Ziehen, Resizen, Pfeil-Burst, Inspector-/Code-Sitzung)
  = ein Schritt. **Duplizieren** (`Strg+D`), **Kopieren/Einfügen** (`Strg+C` /
  `Strg+V`), **Nach vorne/hinten** (`Strg+]` / `Strg+[`). Diese Kürzel wirken nur,
  wenn die Design-Fläche fokussiert ist (kapern also nicht die Textbearbeitung im
  Code-/Inspector-Panel).
- **Anordnen-Menü** (für die Mehrfach-Auswahl): **Ausrichten** (links/rechts/oben/
  unten/zentriert), **Gleiche Breite/Höhe/Größe** (an das zuletzt geklickte
  „primäre" Control), **Horizontal/Vertikal verteilen** (gleiche Lücken, erstes +
  letztes bleiben fix) — alles undobar.
- **Werkzeugleisten** (zwei Zeilen, Symbole programmatisch gezeichnet):
  oben die ständig gebrauchten Befehle — **Neu / Öffnen / Speichern**,
  **Rückgängig / Wiederholen** (dieselben Aktionen wie im Menü, grauen also
  gemeinsam aus), **Code-Fenster**, **Ausführen** (grün, `F5`). Darunter die
  Anordnen-Befehle in vier Gruppen: waagerecht ausrichten, senkrecht
  ausrichten, gleiche Größe, verteilen. Sie sind **grau, solange zu wenig
  ausgewählt ist** (Ausrichten ab 2, Verteilen ab 3 Controls) — die Absage
  steht damit am Knopf statt erst nach dem Klick in der Statuszeile.
- **Rechts — Inspector:** bei gewähltem **Control** dessen Eigenschaften (Name,
  Text, **Gruppe** (nur RadioButton — Radios derselben Gruppe schließen sich
  gegenseitig aus), **Platzhalter** (nur TextInput), Position/Größe, **Anker**
  L/R/O/U, `on_click`/`on_change`-Handler, Items, **Auswahl** (Dropdown/ListBox,
  `-1` = keine), Min/Max/Wert, aktiviert, sichtbar …). Wird ein Handler hier
  **umbenannt**, wandert sein Code-Rumpf mit (außer ein zweites Control nutzt
  ihn noch). **Anker** = an welchen Fensterkanten das Control
  klebt; beim Vergrößern des (resizeable) Formulars wandern/wachsen die Controls
  entsprechend mit (Xojo-Reflow, `GUI_SET_ANCHOR` in der Runtime). Ist **kein**
  Control gewählt, zeigt der Inspector das **Formular selbst** (Xojo-Stil):
  Titel, Breite/Höhe, Min/Max-Größe, beweglich/schließbar/**größenveränderbar**/
  sichtbar. Das Formular hat dann Resize-Griffe (rechts/unten/Ecke). Ist
  „größenveränderbar" gesetzt, ist das **gebaute Fenster zur Laufzeit** am
  unteren-rechten Griff ziehbar (geklemmt an Min/Max) — `GUI_WINDOW_RESIZABLE` /
  `GUI_WINDOW_SET_MIN_SIZE`/`MAX_SIZE` in der `gui`-Runtime.
- **Unten — Code:** integrierter GameBasic-Editor (syntax-gehighlightet). Eine
  Combo listet die Event-Handler des Formulars, der Editor zeigt/ändert den Body
  des gewählten. **Doppelklick auf ein Control** legt für sein Haupt-Event einen
  Handler an (Name `<control>Click`/`Changed`) bzw. springt zu einem vorhandenen
  und fokussiert den Editor.

## Workflow

1. **Entwerfen:** Controls platzieren, im Inspector benennen und Events
   (Handler-Namen) eintragen, z.B. `on_save` für einen Button.
2. **Speichern** (`Strg+S`) als `.gbform` — JSON im Runtime-Format.
3. **Nutzen** — drei Wege:
   - **Im eigenen Code:** `GUI_LOAD("meinform.gbform")` und die Handler-`SUB`s
     schreiben; `GUI_UPDATE` ruft sie automatisch per Name auf.
   - **Direkt testen:** `F5` (Ausführen) — der Designer schreibt das Layout +
     ein generiertes Programm-Gerüst in einen Temp-Ordner, prüft es mit
     `gbrt --check` (Fehler landen in einem Dialog statt im Nichts) und startet
     dann `gbrt`. Ein erneutes F5 beendet den vorigen Lauf und räumt dessen
     Temp-Ordner; das Schließen des Designers ebenso.
     Die Form läuft **randlos auf dem echten OS-Fenster** (Fenstergröße =
     Formgröße, Titel = Formtitel); ist sie „größenveränderbar", ist das
     **Programmfenster nativ resizebar** und die Form füllt es jeden Frame —
     die verankerten Controls fließen dabei mit (Reflow).
   - **GB-Code exportieren:** *Datei → GB-Code exportieren…* schreibt ein
     **eigenständiges** `.gb`, das das Formular mit den `GUI_*`-Konstruktoren
     **explizit aufbaut** (`GUI_WINDOW`/`GUI_BUTTON`/… + Setter + `GUI_ON_CLICK`/
     `GUI_ON_CHANGE`) statt `GUI_LOAD` — frei lesbar und weiter editierbar; die
     im Code-Editor hinterlegten Handler-Körper sind als `SUB`s eingewebt.

```basic
' So nutzt du ein gespeichertes Formular im eigenen Programm:
IMPORT "gui"
SCREEN(800, 480, "App", 1)
DIM frm AS GUI_WINDOW
frm = GUI_LOAD("forms/settings.gbform")

SUB on_save()            ' Name = der im Inspector eingetragene Handler
    PRINT "gespeichert"
END SUB

WHILE NOT QUITREQUESTED()
    GUI_UPDATE() : CLS(0) : GUI_DRAW() : FLIP()
WEND
```

## Die Tabelle im Designer

Die Tabelle lässt sich wie jedes andere Control platzieren; im Inspector gibt
es dafür einen eigenen Abschnitt:

| Feld | Wirkung |
|---|---|
| **Spalten (1/Zeile)** | Spaltentitel, ein Titel je Zeile |
| **Breiten (px)** | z. B. `120, 80, 60` — leer heißt gleichmäßig verteilen |
| **Zeilenhöhe / Kopfhöhe** | Bilder in Zellen brauchen mehr Höhe |
| **Feste Spalten** | die ersten *n* scrollen waagerecht nicht mit |
| **Bearbeitbar** | Spaltennummern, z. B. `1, 2` — nur diese lassen sich per Doppelklick ändern |
| Schalter | Zebra, Gitter, Filterzeile, Sortieren, Breiten ziehbar, Spalten verschiebbar, Mehrfachauswahl |

Die **Datenzeilen fehlen bewusst.** Eine Tabelle wird im Normalfall zur
Laufzeit gefüllt — aus einer Datei, einer Datenbank, dem Spielstand — und nicht
im Designer abgetippt. Der Designer legt das Gerüst fest, die Zeilen kommen aus
dem Programm:

```basic
DIM z AS ARRAY OF STRING
z = SPLIT$("Anna|Hamburg|420", "|")
GUI_TABLE_ADD_ROW(tbl, z)
```

Auf der Design-Fläche zeigt die Vorschau Kopfzeile, Filterzeile, Zebra, Gitter
und die Kante des festen Blocks — aber **keine erfundenen Inhalte**. Die Kante
wird hier immer gezeigt (in der Laufzeit erst beim Scrollen): im Entwurf gibt
es kein Scrollen, an dem man sie sonst erkennen könnte.

Ein Formular, das mit `GUI_SAVE` aus einem laufenden Programm entstanden ist,
bringt auch **Zeilen** mit. Der Designer stellt sie nicht dar, wirft sie aber
auch nicht weg — Öffnen und Speichern verliert nichts.

## Dateiformat

`.gbform` ist exakt das JSON, das `GUI_SAVE`/`GUI_LOAD` lesen/schreiben (siehe
[module-gui.md](module-gui.md)) — plus zwei Designer-Felder, die die Runtime
ignoriert: `name` pro Control und ein Top-Level-`code` (`{handler_name:
gb-code}`) mit den Event-Handler-Körpern. Der Designer und ein handgeschriebenes
`GUI_SAVE` erzeugen dieselbe Datei; beides ist austauschbar. Beim **Ausführen
(F5)** webt der Designer die `code`-Körper als `SUB`-Rümpfe in das generierte
Programm-Gerüst (Handler ohne Body werden zu `' TODO`-Stubs).

**Felder, die der Designer nicht darstellt, reicht er unverändert durch.** Die
`gui`-Laufzeit kennt mehr als die Palette anbietet — auf Fenster-Ebene `chrome`,
`menus`, `tabs`/`active_tab`, pro Widget `table`, `tree`, `tab_page`, `font`,
und die Widget-Arten `table`/`tree`/`textarea`/`spinner`/`splitter`/`toolbar`.
Eine im Programm gebaute und mit `GUI_SAVE` gesicherte Form lässt sich also im
Designer öffnen und nachjustieren, ohne dass Menüs, Reiter oder Tabellendaten
verloren gehen; bearbeiten lassen sie sich dort aber nicht (sie werden auf der
Design-Fläche auch nicht gezeichnet). Ein Golden-Test führt diesen Roundtrip
real durch gbrt.

**Robustheit beim Laden:** Beschädigte oder von Hand geschriebene Dateien
(fehlende Felder, falsche Typen, `null`) fallen feldweise auf den Default
zurück — genau wie `gui.rs` es tut — statt einen Fehler zu werfen. Ein
`.gbproj`-Manifest wird beim Laden als Formular **abgelehnt** (sonst hätte ein
anschließendes Speichern die Projektdatei überschrieben).

## Architektur / Erweiterung

- Datenmodell Qt-frei in [`gamebasic/formdesigner/document.py`](../gamebasic/formdesigner/document.py)
  (`FormDoc`/`Control`, `.gbform`-IO, `PALETTE`, Code-Generierung) — headless
  getestet (`tests/test_formdesigner_document.py`).
- UI in [`gamebasic/formdesigner_qt.py`](../gamebasic/formdesigner_qt.py)
  (Palette/Canvas/Inspector/Code-Panel). Neue Control-Arten: Eintrag in `PALETTE`
  ergänzen — Inspector/Canvas/Serialisierung ziehen daraus.
- **Ungespeichert-Schutz:** `FormDesigner._confirm_dirty()` fragt für **alle**
  offenen Formulare (nicht nur das aktive) und wird von `closeEvent`,
  „Projekt öffnen…" und `close_form` benutzt. Qt-Tests dieser Datei müssen
  modale Dialoge abfangen — dafür gibt es in `tests/test_formdesigner_qt.py`
  eine Autouse-Fixture, die `QMessageBox.question/warning/critical` ersetzt.
  Ohne sie hält der erste Dialog den ganzen pytest-Lauf an.
- **Gotcha:** Das Code-Panel hängt einen `GBHighlighter` an sein Editor-Dokument.
  Ein lebender `QSyntaxHighlighter` segfaultet beim Interpreter-Shutdown, wenn er
  die Teardown-Race von Dokument + `QApplication` überlebt (im Test sichtbar als
  Exit-Code 116, sobald vorher ein `gbrt`-Subprozess lief). Deshalb löst
  `FormDesigner.closeEvent` ihn via `code_panel.detach_highlighter()`
  (`setDocument(None)`); Qt-Tests müssen das Fenster mit `win.close()` schließen.

## Status / geplant

Vorhanden: Platzieren, Auswählen, Verschieben, **Resize-Handles + Snap-Grid**,
Löschen, **Undo/Redo**, Inspector (Kerneigenschaften + Events), **integrierter
Code-Editor** (Doppelklick-auf-Control → Handler anlegen/anspringen),
**Multi-Form-Projekte** (`.gbproj`), **GB-Code-Export** (explizite
`GUI_*`-Konstruktion statt `GUI_LOAD`), Speichern/Laden, Ausführen (F5). Damit
ist der geplante Funktionsumfang komplett.

**GB-Code-Export-Detail:** `FormDoc.generate_gb_code()` (Qt-frei) emittiert pro
Control den passenden Konstruktor (`GUI_LABEL` ohne w/h, `GUI_SLIDER` mit
min/max/value, Items als sized `DIM x[n] AS STRING` …) plus nur die abweichenden
Setter (`GUI_SET_ENABLED/VISIBLE/VALUE/FONT_SIZE/COLOR`, `*_SET_SELECTED`).
Handler werden per `GUI_ON_CLICK/CHANGE`-FUNCREF verdrahtet. **Grenze:**
`image`-Controls werden übersprungen (das `.gbform` speichert keine Bildquelle —
`GUI_IMAGE` bräuchte ein `LOADIMAGE`). Strings escapen `"`→`""`. Ein
run_gb-Golden-Test führt die erzeugte Konstruktion real in gbrt aus **und
vergleicht sie gegen `GUI_LOAD` desselben `.gbform`** — beide Wege müssen
dasselbe Formular bauen.

**Zwei Eigenheiten der Konstruktoren**, die der Export ausgleicht:
`GUI_LABEL`/`CHECKBOX`/`RADIO`/`SLIDER`/`SEPARATOR` berechnen ihre Größe selbst,
deshalb wird `GUI_SET_BOUNDS` nachgereicht. `GUI_PROGRESS` liegt fest auf
`min=0/max=1` und hat keinen Range-Setter, deshalb wird der Wert auf den Anteil
normiert (optisch identisch — die Laufzeit zeichnet den Balken als
`(value-min)/(max-min)`). *Restgrenze:* `GUI_SET_BOUNDS` aktualisiert die
Anchor-Basis der Laufzeit nicht mit — wird ein exportiertes, resizebares Fenster
gezogen, springen diese fünf Control-Arten auf ihre Konstruktor-Größe zurück.

**Multi-Form-Architektur:** Qt-freies `FormProject` (in
`formdesigner/document.py`) ist nur ein Manifest (`forms`-Liste + `main`); jede
Form bleibt ihr eigenes `.gbform`. Im Fenster bündelt `_OpenForm` pro Formular
Dokument + Pfad + `History` + Dirty; `FormDesigner.history`/`.path` sind
Properties auf die aktive Form, `_switch_to` tauscht Canvas/Inspector/Code-Panel
um.

**Undo/Redo-Mechanik:** Snapshot-basiert — die Qt-freie `History` (in
`formdesigner/document.py`) hält komplette `FormDoc`-Snapshots auf einem
Undo-/Redo-Stack; die Canvas legt vor jeder Mutation einen Checkpoint an und
fasst kontinuierliche Gesten (Drag/Resize) bzw. eine Inspector-Edit-Sitzung zu
je einem Schritt zusammen.


## Thema des Formulars

In den Formular-Eigenschaften (Inspector, wenn kein Control gewählt ist) gibt
es die Auswahl **Thema**. Sie bestimmt zweierlei:

- Das erzeugte Programm setzt `GUI_THEME_PRESET(...)` **vor** `GUI_LOAD` — das
  Preset legt auch Metriken wie den Eckenradius fest, und die gehen in die
  Darstellung der Widgets ein.
- Die Entwurfsfläche malt das Thema nach, damit der Entwurf zeigt, was das
  Formular später wirklich tut.

`(Vorgabe)` heißt: kein Preset-Aufruf, also das eingebaute Cyan-Thema der
Laufzeit. Ein Formular ohne Eintrag bekommt beim Speichern **kein** neues
Feld — bestehende `.gbform`-Dateien ändern sich also nicht.

> **Achtung bei Änderungen:** `FORM_THEME_COLORS` in
> `gamebasic/formdesigner/document.py` ist ein **Nachbau** der Presets aus
> `rust/gb_runtime/src/gui.rs` — der Designer zeichnet mit Qt und kann die
> Laufzeit nicht fragen. `tests/test_formdesigner_theme.py` vergleicht beide
> gegeneinander; wer ein Preset in gui.rs ändert oder hinzufügt, muss die
> Tabelle nachziehen (der Test sagt genau, welcher Wert abweicht).
