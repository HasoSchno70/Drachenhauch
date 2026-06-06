# Form-Designer (WYSIWYG, Xojo-Stil)

Visueller GUI-Designer für GameBasic — Controls per Klick platzieren, im
Inspector konfigurieren, als `.gbform` speichern und mit den `gui`-Builtins zur
Laufzeit laden. Sprache der Logik bleibt GameBasic.

**Start:** `gbform [datei.gbform]` (bzw. `gbrun.py --form`). Benötigt PySide6.
Alternativ `gb` (oder `gbrun.py`) **ohne Argument** → Auswahl-Dialog
*Code-Editor* / *Form-Designer*. `gbedit` öffnet direkt den Code-Editor.

## Aufbau (wie Xojo)

- **Links — Controls:** Palette aller Widget-Arten (Button, Label, Checkbox,
  Radio, Slider, TextInput, Dropdown, ListBox, ProgressBar, Image, Canvas,
  Panel). Eintrag anklicken → „scharf", dann auf die Fläche klicken zum Platzieren.
- **Mitte — Design-Fläche:** das Formular. Control anklicken = auswählen, ziehen =
  verschieben, an den 8 **Resize-Griffen** ziehen = Größe ändern, `Entf` = löschen.
  Bewegen/Platzieren/Resizen rasten am **8-px-Raster** ein (Punkt-Raster sichtbar);
  Toggle über `Ansicht → Am Raster ausrichten` (`Strg+G`). **Undo/Redo** über
  `Strg+Z` / `Strg+Y` (auch `Strg+Umschalt+Z`) — eine Geste (Platzieren, Ziehen,
  Resizen, Löschen, eine Inspector-Edit-Sitzung) = ein Schritt.
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
3. **Nutzen** — zwei Wege:
   - **Im eigenen Code:** `GUI_LOAD("meinform.gbform")` und die Handler-`SUB`s
     schreiben; `GUI_UPDATE` ruft sie automatisch per Name auf.
   - **Direkt testen:** `F5` (Ausführen) — der Designer schreibt das Layout +
     ein generiertes Programm-Gerüst (Handler-Stubs + GUI-Schleife) in einen
     Temp-Ordner und startet `gbrt`.

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
Speichern/Laden, Ausführen (F5). Geplant: Multi-Form-Projekte, GB-Code-Export
(explizite `GUI_*`-Konstruktion statt `GUI_LOAD`).

**Undo/Redo-Mechanik:** Snapshot-basiert — die Qt-freie `History` (in
`formdesigner/document.py`) hält komplette `FormDoc`-Snapshots auf einem
Undo-/Redo-Stack; die Canvas legt vor jeder Mutation einen Checkpoint an und
fasst kontinuierliche Gesten (Drag/Resize) bzw. eine Inspector-Edit-Sitzung zu
je einem Schritt zusammen.
