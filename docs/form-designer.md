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
  verschieben, `Entf` = löschen.
- **Rechts — Inspector:** Eigenschaften des gewählten Controls (Name, Text,
  Position/Größe, `on_click`/`on_change`-Handler, Items, Min/Max/Wert, aktiviert …).

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
[module-gui.md](module-gui.md)) — plus ein Designer-Feld `name` pro Control (von
der Runtime ignoriert). Der Designer und ein handgeschriebenes `GUI_SAVE`
erzeugen dieselbe Datei; beides ist austauschbar.

## Architektur / Erweiterung

- Datenmodell Qt-frei in [`gamebasic/formdesigner/document.py`](../gamebasic/formdesigner/document.py)
  (`FormDoc`/`Control`, `.gbform`-IO, `PALETTE`, Code-Generierung) — headless
  getestet (`tests/test_formdesigner_document.py`).
- UI in [`gamebasic/formdesigner_qt.py`](../gamebasic/formdesigner_qt.py)
  (Palette/Canvas/Inspector). Neue Control-Arten: Eintrag in `PALETTE`
  ergänzen — Inspector/Canvas/Serialisierung ziehen daraus.

## Status / geplant

Vorhanden: Platzieren, Auswählen, Verschieben, Löschen, Inspector
(Kerneigenschaften + Events), Speichern/Laden, Ausführen (F5). Geplant:
Resize-Handles + Snap-Grid, Undo/Redo, integrierter Code-Editor mit
Doppelklick-auf-Control → Handler anlegen/anspringen, Multi-Form-Projekte,
GB-Code-Export (explizite `GUI_*`-Konstruktion statt `GUI_LOAD`).
