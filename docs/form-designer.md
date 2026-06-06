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
  Image, Canvas, Panel) — jeder Eintrag mit Mini-Vorschau-Icon. Platzieren auf
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
- **Bearbeiten-Menü:** **Undo/Redo** (`Strg+Z` / `Strg+Y`, auch `Strg+Umschalt+Z`)
  — eine Geste (Platzieren, Ziehen, Resizen, Pfeil-Burst, Inspector-/Code-Sitzung)
  = ein Schritt. **Duplizieren** (`Strg+D`), **Kopieren/Einfügen** (`Strg+C` /
  `Strg+V`), **Nach vorne/hinten** (`Strg+]` / `Strg+[`). Diese Kürzel wirken nur,
  wenn die Design-Fläche fokussiert ist (kapern also nicht die Textbearbeitung im
  Code-/Inspector-Panel).
- **Rechts — Inspector:** Eigenschaften des gewählten Controls (Name, Text,
  Position/Größe, `on_click`/`on_change`-Handler, Items, Min/Max/Wert, aktiviert …).
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
     ein generiertes Programm-Gerüst (Handler-Stubs + GUI-Schleife) in einen
     Temp-Ordner und startet `gbrt`.
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

## Dateiformat

`.gbform` ist exakt das JSON, das `GUI_SAVE`/`GUI_LOAD` lesen/schreiben (siehe
[module-gui.md](module-gui.md)) — plus zwei Designer-Felder, die die Runtime
ignoriert: `name` pro Control und ein Top-Level-`code` (`{handler_name:
gb-code}`) mit den Event-Handler-Körpern. Der Designer und ein handgeschriebenes
`GUI_SAVE` erzeugen dieselbe Datei; beides ist austauschbar. Beim **Ausführen
(F5)** webt der Designer die `code`-Körper als `SUB`-Rümpfe in das generierte
Programm-Gerüst (Handler ohne Body werden zu `' TODO`-Stubs).

## Architektur / Erweiterung

- Datenmodell Qt-frei in [`gamebasic/formdesigner/document.py`](../gamebasic/formdesigner/document.py)
  (`FormDoc`/`Control`, `.gbform`-IO, `PALETTE`, Code-Generierung) — headless
  getestet (`tests/test_formdesigner_document.py`).
- UI in [`gamebasic/formdesigner_qt.py`](../gamebasic/formdesigner_qt.py)
  (Palette/Canvas/Inspector/Code-Panel). Neue Control-Arten: Eintrag in `PALETTE`
  ergänzen — Inspector/Canvas/Serialisierung ziehen daraus.
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
run_gb-Golden-Test führt die erzeugte Konstruktion real in gbrt aus.

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
