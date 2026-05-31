# GUI-Module — Design & Plan (für die nächste Session)

> Status: **Phase 1+2 implementiert** (Modul `gui`, Retained-Mode) -- siehe
> [module-gui.md](module-gui.md), `gamebasic/modules/gui.py`,
> Tests `tests/test_modules_gui.py`, Demo `examples/45_gui.gb`.
> Verifiziert: TW == Python-VM == Native-VM (bit-identisch).
> Phase 3 (FUNCREF-Callbacks `GUI_ON_CLICK`) implementiert -- Builtin->Engine-
> Bruecke `call_funcref` + `gb_call_function` in allen 3 Pfaden, 3-Pfad-
> verifiziert. Phase 4 (`ui`-Immediate-Mode-Fenster `UI_WINDOW_BEGIN/END`)
> implementiert (Offset fuer Child-Widgets, Titel-Drag, Einklappen, Z-Order
> via Vorframe-Hit-Test). **Alle Phasen 1-4 fertig.**
> Entscheidungen mit dem Nutzer abgestimmt (siehe unten).

## Ziel

Fenster, Buttons & weitere Widgets als **nachladbare Module** — passend zur
bestehenden GameBasic-Modul-Architektur (`IMPORT "x"` lädt
`gamebasic/modules/x.py` beim ersten Aufruf, registriert Builtins +
externe Typen, **kein** Eingriff in Lexer/Parser/VM). Rein additiv,
drei-Pfade-neutral, bit-identisch-unkritisch (Grafik/Interaktion).

## Abgestimmte Richtung

- **Paradigma: BEIDES**
  1. `ui`-Modul (Immediate-Mode) um **Fenster** erweitern (`UI_WINDOW_BEGIN/END`,
     ImGui-Stil: verschiebbar, Z-Order/Fokus über den vorhandenen id-State).
  2. **Neues `gui`-Modul** (Retained-Mode): Fenster/Widgets als persistente
     Objekte (`GUI_WINDOW`/`GUI_WIDGET`), ein `_GuiManager` verwaltet
     Drag/Z-Order/Fokus, `GUI_UPDATE()` + `GUI_DRAW()` pro Frame.
- **Events: BEIDES**
  - **Polling als Standard** (`IF GUI_CLICKED(btn) THEN ...`) — einfach, passt
    zum Builtin-Modell, kein Interpreter-Rückruf nötig.
  - **FUNCREF-Callbacks optional** (`GUI_ON_CLICK(btn, on_ok)`) — braucht eine
    kleine **Builtin→Interpreter/VM-Brücke**, damit ein Builtin eine GB-Funktion
    aufrufen kann (s. „Offene technische Punkte").

## Was es bereits gibt (Basis)

`gamebasic/modules/ui.py` (Immediate-Mode) deckt schon ab: `UI_LABEL`,
`UI_BUTTON`, `UI_CHECKBOX`, `UI_SLIDER`, `UI_PROGRESS`, `UI_PANEL` (Container
mit Titel), `UI_TEXTFIELD` (Fokus + Tastatur), `UI_RADIO`, `UI_TABLE`,
`UI_END_FRAME`, `UI_RESET`. State per id-String. **Route 1 baut darauf auf.**

Maus ist scale-korrekt (`MOUSEX`/`MOUSEY` teilen durch den SCREEN-Scale).
Zeichen-Primitive vorhanden: `BOX`/`RECT`/`LINE`/`CIRCLE`/`TEXT` + neu
`BOXES`/`CIRCLES`/`LINES` (Bulk, für viele Widgets effizient).

## API-Skizze

### Route 1 — `ui`-Fenster (Immediate-Mode)
```basic
IMPORT "ui"
IF UI_WINDOW_BEGIN("settings", "Einstellungen", 120, 80, 320, 220) THEN
    IF UI_BUTTON("ok", 20, 160, 90, 30, "OK") THEN spiel_start()
    snd = UI_CHECKBOX("snd", 20, 60, "Sound", TRUE)
END IF
UI_WINDOW_END()
UI_END_FRAME() : FLIP()
```
`UI_WINDOW_BEGIN` pusht einen Koordinaten-Offset (Widgets darin werden
fenster-relativ gezeichnet) und verwaltet Titelleisten-Drag + Z-Order über
id-State. Rückgabe FALSE wenn eingeklappt/zu → Body überspringen.

### Route 2 — `gui` (Retained-Mode)
```basic
IMPORT "gui"
DIM win AS GUI_WINDOW
win = GUI_WINDOW("Einstellungen", 120, 80, 320, 220)
GUI_WINDOW_MOVABLE(win, TRUE) : GUI_WINDOW_CLOSABLE(win, TRUE)
DIM b_ok AS GUI_WIDGET
b_ok = GUI_BUTTON(win, "OK", 20, 160, 90, 30)
DIM chk AS GUI_WIDGET
chk = GUI_CHECKBOX(win, "Vollbild", 20, 60, FALSE)

WHILE NOT QUITREQUESTED()
    CLS(&H101828)
    GUI_UPDATE()          ' Maus/Tasten: Hover, Klick, Drag, Fokus, Z-Order
    GUI_DRAW()            ' alle Fenster hinten→vorne
    IF GUI_CLICKED(b_ok) THEN PRINT GUI_CHECKED(chk)
    FLIP() : SLEEP(16)
WEND
```
Optionale Callbacks: `GUI_ON_CLICK(b_ok, on_ok_funcref)`.

## Widgets v1
Window, Button, Label, Checkbox, Slider, TextInput, Panel.
Später: Dropdown, ListBox, Tabs, RadioGroup, modale Dialoge.

## Implementierungs-Skizze (Python-Seite, `gui`)
- Externe Typen: `register_type("gui_window", _Window)`, `("gui_widget", _Widget)`.
- `_GuiManager` (Modul-Singleton): Liste der Fenster (Z-geordnet), Fokus,
  Drag-State, vorige Maustaste (Edge-Detection).
- `_Window`: Titel, Rect, Widget-Liste, Flags (movable/closable/visible).
- `_Widget`-Basis + Subklassen: Rect (fenster-relativ), State (hover/pressed/value), id.
- `GUI_UPDATE(g)` (`@graphics_builtin`): Maus aus `g` lesen, Hit-Test top-down
  (oberstes Fenster zuerst), Hover/Active setzen, Press/Release → `clicked`-Flag,
  Titelleisten-Drag, Klick-auf-Fenster → nach vorne.
- `GUI_DRAW(g)`: Fenster hinten→vorne; Fenster-BG/Border/Titelleiste, dann Widgets.
  Viele gleichartige Widgets → `BOXES` (Bulk) nutzen.
- Theme: Logo-Cyan-Palette als Default (konsistent zum Editor), `GUI_THEME(...)`.

## Offene technische Punkte
1. **FUNCREF-Brücke**: Builtins sind reine Python-Funktionen ohne Handle auf den
   laufenden Interpreter/VM. Für Callbacks aus `GUI_UPDATE` heraus braucht es
   einen Weg, eine `_FuncRef` aufzurufen. Optionen:
   - `GUI_UPDATE` gibt die ausgelösten Callbacks NICHT selbst auf, sondern
     sammelt „pending"-FuncRefs; ein dünner Sprach-Hook ruft sie auf — oder
   - kleine Bridge: Interpreter/VM hinterlegen beim Lauf eine `call_funcref`-
     Funktion im `gui`-Modul (z.B. via `graphics`-Instanz oder einem Modul-
     globalen Slot), die `GUI_UPDATE` nutzen kann.
   Sauberste Variante zuerst evaluieren; Polling braucht das nicht.
2. **Layout**: v1 absolute Koordinaten (fenster-relativ). Auto-Layout später.
3. **Tastatur/Textinput**: `INKEY$`/Fokus wie im `ui`-Textfield wiederverwenden.
4. **Modul-Split**: ggf. `gui` (Fenster/Widgets) + `gui_dialogs` (MessageBox/
   File-Picker) getrennt ladbar.

## Phasen-Plan (nächste Session)
1. `gui`-Modul-Gerüst: `_GuiManager`, `_Window`, `_Widget`-Basis, `GUI_WINDOW`,
   `GUI_UPDATE`, `GUI_DRAW` + Theme. Button + Label. **Polling.**
2. Checkbox, Slider, Panel, TextInput. Drag/Z-Order/Fokus robust.
3. FUNCREF-Brücke + `GUI_ON_*`-Callbacks (optional obendrauf).
4. `ui`-Fenster (`UI_WINDOW_BEGIN/END`) als Immediate-Mode-Variante.
5. Beispiel-Demo `examples/NN_gui.gb` + Tests (Hit-Test/State headless,
   wo möglich `run_all`-tauglich).

## Test-/Doku-Hinweise
- Builtins laufen in allen drei Pfaden identisch → wo nicht-grafisch testbar
  (State/Hit-Test), `run_all` in `tests/test_three_paths.py` nutzen.
- Neue Module in der CLAUDE.md-Modul-Tabelle + eine eigene Sektion ergänzen.
