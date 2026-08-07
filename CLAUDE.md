# GameBasic

BASIC-Dialekt mit Pascal-strikter Typisierung und OOP, ausgelegt für Spiele.
**Eine Runtime: `gbrt`** (Rust/raylib) — sie ist Lexer → Parser → Compiler → VM
in einem (eigenes Rust-Frontend) und übernimmt Ausführung, Konsole, Grafik/Audio
und Standalone-Export. Python ist nur noch **Editor-/Tooling-Schicht** (eigener
Lexer/Parser für Highlighting/LSP, die Qt-Editoren, preprocess für IMPORT-Merge).

> ## ⚠️ STUFE B — Tree-Walker + Python-Toolchain ENTFERNT (2026-06-06)
> Früher gab es zusätzlich einen Python-**Tree-Walker** (`interpreter.py`, Referenz
> + Built-in-Host), einen Python-**Compiler** (`compiler.py`/`bytecode.py`/
> `serialize.py` → `.gbc`) und zwei Bytecode-VMs (`vm.py`, `vm_native.pyx`). **Alle
> entfernt** — `gbrt` hat sie abgelöst und ist die EINZIGE Runtime. Ebenfalls weg:
> `builtins_registry.py`, alle `modules/*.py`-Implementierungen (gbrt
> reimplementiert die Module nativ in Rust), `export.py`, `environment.py`, pygame.
>
> **Folgen für die Arbeit:** Neue Builtins/Sprach-Features kommen NUR in
> `rust/gb_runtime/` (+ ein run_gb-Golden-Test). Es gibt KEIN „beide Pfade" /
> „drei Pfade" / „Parität gegen Tree-Walker" mehr — Korrektheit sichern
> run_gb-Golden-Tests (`assert run_gb(src) == expected`) + Rust-`#[test]`s.
>
> **WICHTIG:** Viele Feature-Abschnitte WEITER UNTEN erwähnen noch
> `interpreter.py` / `compiler.py` / `vm.py` / `vm_native.pyx` / „Tree-Walker" /
> „alle drei Pfade" / „in BEIDEN Pfaden umsetzen". Das sind **historische
> Implementierungsnotizen** — die Dateien existieren nicht mehr; gültig ist heute
> ausschließlich `gbrt` (`rust/gb_runtime/src/`: `lexer.rs`/`parser.rs`/
> `compiler.rs`/`vm.rs`/`builtins.rs`/Modul-`.rs`).

## Verzeichnisstruktur

```
gamebasic/             # Python = nur noch Editor-Tooling + Front-End
  __main__.py            # py -m gamebasic <file> -> ruft `gbrt run`
  lexer.py / tokens.py   # Tokenisierung (Highlighting/LSP/Dev)
  parser.py / ast_nodes.py
  preprocess.py          # IMPORT-Merge (.gb-Source) + Built-in-Modul-Namen erkennen
  graphics.py            # nur COLORS/KEYS + Kamera-Mathematik (kein Render; pygame raus)
  synth.py               # Synth-Mathematik (von SFX-/Tracker-Editor + gbsfx genutzt)
  particle_sim.py        # pure numpy-Partikel-Sim (für den Partikel-Editor)
  errors.py              # Fehlertypen
  modules/__init__.py    # NUR Modul-NAMENSLISTE (KNOWN_MODULES) — keine Impls mehr
  editor_qt/             # Qt-Editor + LSP-Bausteine; gbrt_meta.py = Builtin-Index
  spriteeditor*/tilemap*/tracker*/sfxeditor*/particleeditor*  # Begleit-Editoren
rust/gb_runtime/         # >>> die Runtime: gbrt (Rust/raylib)
  src/lexer.rs parser.rs compiler.rs vm.rs builtins.rs + <modul>.rs  # alles in Rust
gbrun.py                 # CLI: Editor-Launcher + run/--native/--export/--tokens/--ast (run -> gbrt)
examples/*.gb            # Demos
tests/                   # pytest (1561+): run_gb-Golden gegen gbrt + Rust-#[test]
```

## Architektur-Pipeline

```
Source.gb  →  preprocess.process()  →  Lexer  →  Parser  →  AST
                                                              │
              ┌───────────────────────────────────────────────┤
              ▼                                               ▼
        Interpreter.run(ast)                         Compiler.compile(ast)
        (Tree-Walking, Python)                               │
                                                             ▼
                                                   Module mit Bytecode
                                                             │
                                                  serialize.py → .gbc
                                                             │
                                                             ▼
                                                   rust/gb_runtime  (gbrt)
                                                   native Ausführung (raylib)
```

`gbrun.py` ist Default-Einstiegspunkt (macht `os.chdir(file.parent)` für relative
Asset-Pfade). `py -m gamebasic` funktioniert auch, wechselt aber nicht ins
Datei-Verzeichnis — Programme mit `LOADIMAGE("assets/...")` brauchen `gbrun.py`.

> **Run/Export laufen über `gbrt` (Rust-Frontend).** `gbrun.py` (Default-Run +
> `--native`) und der Editor-Run rufen `gbrt run datei.gb`; `gbrun.py --export` /
> Editor-Export rufen `gbrt --export` (hängt den `.gbc`-Payload an eine Kopie der
> Exe, kopiert `assets/`). gbrt chdirt selbst ins Datei-Verzeichnis (relative
> Asset-/IMPORT-Pfade). Es gibt keinen Python-Run-/Export-Pfad mehr.

## Built-ins erweitern (in gbrt / Rust)

Builtins leben in `rust/gb_runtime/src/builtins.rs` (pure) bzw. `vm.rs`
(`try_graphics`/`try_*` — brauchen VM-/Fenster-State). Der Dispatch läuft über
`CALL_BUILTIN` (vm.rs) → der große Match in `builtins.rs`. Neuer Builtin:

1. In `builtins.rs` (oder dem passenden `try_*` in `vm.rs`) einen Match-Arm
   ergänzen: Arity + Typen selbst prüfen (Validierung gehört in den Wrapper, nicht
   ins Backend), Fehlermeldung im gewohnten Wortlaut (`"NAME: erwartet …"`).
2. Für den Editor: `editor_qt/builtin_index.json` ergänzen (Name/kind/Signatur/
   Modul) — Completion/Highlighting/LSP ziehen daraus (`editor_qt/gbrt_meta.py`).
   Optional Prosa-Doku in `editor_qt/builtin_docs.py`.
3. Einen `tests/`-Golden-Test schreiben (`assert run_gb('PRINT NAME(...)') == ...`).
   **Die Signatur in `builtin_index.json` muss stimmen** — der Compiler leitet
   daraus die erlaubte Argumentzahl ab und warnt bei Abweichung (`gbrt --check`).
   Formen, die er versteht: `NAME(a, b [, c])`, `NAME(a, b = "")` (Vorgabewert =
   optional), `NAME(a, b, ...)` (beliebig viele), `NAME(6..8 Argumente)`.
   `NAME(*args)` schaltet die Pruefung ab — nur nehmen, wenn es wirklich offen
   ist. Eine zu enge Signatur erzeugt Falsch-Alarme in fremdem Code.
4. Bei neuem Keyword: `vscode-gamebasic/build_grammar.py` neu generieren.

(Es gibt KEINE Python-`@builtin`-Registry / kein `interpreter.py` mehr.)

## Built-in-Module schreiben

### Modul `gui` (Retained-Mode-GUI)

Persistente Fenster/Widgets (externe Typen `GUI_WINDOW`/`GUI_WIDGET` via `register_type`). Aufbau einmalig, pro Frame `GUI_UPDATE()` + `GUI_DRAW()`; Events per Polling (`GUI_CLICKED`/`CHECKED`/`VALUE`/`TEXT`/`HOVERED`) **oder** FUNCREF-Callbacks (`GUI_ON_CLICK`/`GUI_ON_CHANGE`). Widgets: Button, Label, Checkbox, Slider, TextInput, Panel, **Separator** (`GUI_SEPARATOR` — Trennlinie), **GroupBox** (`GUI_GROUPBOX` — gerahmte Gruppe mit eingelassenem Titel), **Table** (`GUI_TABLE` — professionelle Tabelle: fixierte Kopfzeile, V/H-Scroll, persistente Zeilen-Selektion. **Zellmodell** (`Cell` in gui.rs): jede Zelle mit eigener Vorder-/Hintergrundfarbe, Ausrichtung und ART (`text`/`bild`/`haken`/`balken`/`knopf`) — `GUI_TABLE_SET_CELL/CELL_COLOR/CELL_ALIGN/CELL_KIND/CELL_IMAGE/CELL_VALUE`, dazu `GUI_TABLE_ROW_COLOR` (ganze Zeile) + `GUI_TABLE_COL_ALIGN`. Farben in drei Stufen Zelle>Zeile>Zebra, -1 = nicht gesetzt; Auswahl/Hover liegen HALBDURCHSICHTIG darüber, sonst deckte eine Zellfarbe sie zu. **Spaltenzahl frei** = breiteste Angabe (`TableState::n_cols`) — kürzere Zeilen sind leer, keine Reihenfolge-Pflicht mehr (vorher war jede Abweichung ein Fehler, und Zeilen-vor-Kopf umging die Prüfung und crashte das Zeichnen). **Sortieren** per Kopfklick (2. Klick dreht um, Pfeil zeigt es an) — ZAHLENWEISE wenn beide Zellen Zahlen sind, sonst Text; `GUI_TABLE_SORT/SORT_COL/SORT_DESC`. **Filterzeile** im Kopf (`GUI_TABLE_SET(t,"filterzeile",1)`) — Teiltext je Spalte, case-insensitiv, UND-verknüpft; `GUI_TABLE_FILTER/GET_FILTER`. **Spaltenbreiten ziehbar** (Kopfkante, ±4px Fangbereich). **WICHTIG — Datenzeile vs. Ansicht:** Sortieren/Filtern stellen die Daten NIE um, sie bauen nur `view: Vec<usize>`; alle Zeilenangaben nach außen sind DATENzeilen (eine gemerkte Nummer bleibt gültig), sichtbare Reihenfolge über `GUI_TABLE_VIEW_COUNT/VIEW_ROW`. Einstellungen über EINEN Setter `GUI_TABLE_SET(key$,wert)` wie bei `chart`. Layout aus einer Quelle `table_geom` für Hit-Test + Zeichnen. Doku `docs/module-gui.md`, Demos `examples/157_gui_tabelle.gb` + `examples/158_gui_tabelle_sqlite.gb` (an einer echten SQLite-DB: lesen/sortieren/filtern/UPDATE beim Bearbeiten/DELETE in einer Transaktion; **arbeitet auf einer Kopie via `VACUUM INTO`**, die Originaldatei wird nur gelesen -- Test `tests/test_beispiel_sqlite_tabelle.py` sichert genau diese Zusage ab). Muster fuer DB-Anbindung: DB ist die Wahrheit, Tabelle die Ansicht; je Zeile die id merken (Sortieren/Filtern stellen Datenzeilen nicht um); beim Mehrfach-Loeschen ERST alle ids einsammeln, dann loeschen. **Zellen bearbeiten:** Doppelklick auf eine Textzelle einer per `GUI_TABLE_COL_EDIT` freigegebenen Spalte oeffnet ein Eingabefeld IN der Zelle (Enter uebernimmt, ESC nimmt zurueck, Klick woanders uebernimmt); `GUI_TABLE_EDITING_ROW/COL` fragen den Zustand ab. Arbeitskopie `edit_text` -- die Zelle wird ERST beim Bestaetigen geschrieben, sonst koennte ESC nichts zuruecknehmen und jede Taste wuerde Sortierung/Filter neu anwerfen (die Zeile spraenge beim Tippen weg). Doppelklick-Erkennung im Gui (`last_click`, zeitbasiert via `g.get_time()`, 0.4 s + max 4 px Versatz); `editing_table` merkt die Tabelle, damit ein Klick woanders in O(1) uebernimmt statt einen verwaisten Editor stehen zu lassen. **ESC-Falle:** benutzt das Programm ESC zum Beenden, muss es `GUI_TABLE_EDITING_ROW < 0` abfragen. **Feste Spalten** (`GUI_TABLE_SET(t,"feste_spalten",n)`): die ersten n scrollen waagerecht nicht mit. ALLES was Spalten verortet geht ueber EINE Quelle -- `col_x` (Lage) + `col_clip` (sichtbarer Bereich), von Treffertest UND Zeichnen benutzt; ein fester Block waere sonst der sicherste Weg, beide auseinander laufen zu lassen. **Mehrfachauswahl** (`mehrfachauswahl`-Schalter, aus per Vorgabe): Strg+Klick schaltet eine Zeile um, Umschalt+Klick waehlt den Bereich ab dem Anker -- in der SICHTBAREN Reihenfolge (ueber Datenzeilen traefe es bei sortierter Tabelle etwas anderes); `GUI_TABLE_SEL_COUNT/SEL_ROW/IS_SELECTED/SELECT/CLEAR_SELECTION`. `GUI_TABLE_SELECTED` bleibt die zuletzt angeklickte Zeile (Rueckwaertskompatibilitaet), beim Zeilen-Loeschen ruecken alle Auswahl-Indizes dahinter auf. **Spalten umsortieren** (`spalten_verschiebbar`, per Vorgabe AUS): Kopfzelle seitwaerts ziehen, Tausch sobald sie ueber die Mitte des Nachbarn kommt. Klick und Zug sind DIESELBE Geste -- erst beim Loslassen entscheidet sich, ob sortiert (keine Bewegung) oder verschoben wurde (>= 5 px); ohne das wuerde jedes Verschieben nebenbei sortieren. Wie bei den Zeilen werden die DATEN nicht umgestellt: `col_order` bildet nur Anzeige-Position -> Datenspalte ab, `TGeom.col_widths` steht in ANZEIGE-Reihenfolge, `pos_at` liefert die Position und `col_at` die Datenspalte. `TableState::order()` bereinigt+ergaenzt die Liste bei JEDEM Abruf, damit eine spaeter dazugekommene Spalte nicht unter den Tisch faellt. `GUI_TABLE_MOVE_COL/COL_AT/COL_POS/RESET_COLS`; Reihenfolge + feste Spalten werden im .gbform gespeichert. **Textauswahl in der Zelle:** der Zell-Editor benutzt DIESELBE Routine wie das TextInput-Widget (`Gui::einzeiler_tasten`, aus `edit_textinput` herausgeloest) -- Markieren per Umschalt+Navigation oder Maus-Ziehen, Strg+A/C/V/X, Pos1/Ende. Beim Oeffnen ist alles markiert (erstes Tippen ersetzt). **Gotcha:** der oeffnende Doppelklick darf NICHT als Marke-setzen zaehlen -- `edit_maus_sperre` sperrt die Maus, bis die Taste einmal los war; ohne das hob derselbe Klick die Markierung sofort wieder auf (Ruecktaste loeschte dann ein Zeichen statt des Inhalts). Klicks IM Eingabefeld sind sowohl in `handle_press` als auch in `table_press` ausgenommen, sonst schlossen sie den Editor. Das Feld-Rechteck wird aus `table_geom` GERECHNET (`edit_cell_rect`), nicht beim Zeichnen gemerkt -- `draw_cell` ist `&self`, und eine zweite nachgefuehrte Quelle liefe auseinander. **Nicht umgesetzt:** Zeilengruppen), **Radio** (`GUI_RADIO(win,group$,text,x,y)` — Gruppen mit gegenseitigem Ausschluss, `GUI_RADIO_SELECTED`), **Dropdown/ComboBox** (`GUI_DROPDOWN(win,x,y,w,h,items)` — aufklappendes Popup über allen Widgets, `GUI_DROPDOWN_SELECTED/TEXT/SET_SELECTED`, `GUI_SET_DROPDOWN`), **ProgressBar** (`GUI_PROGRESS`, Wert via `GUI_SET_VALUE` 0..1). **Laufzeit-Manipulation** (`GUI_SET_BOUNDS`/`GUI_GET_X/Y/W/H`, `GUI_DESTROY`, `GUI_SET_VISIBLE`/`GUI_VISIBLE`, `GUI_KIND`, `GUI_FOCUS`, `GUI_HIT_TEST`, Window-Pendants + `GUI_WINDOW_WIDGET_COUNT/WIDGET`-Enumeration; Tombstone-Handles bleiben stabil) + **Serialisierung** (`GUI_SAVE/LOAD` Datei, `GUI_TO_JSON/FROM_JSON` String) — Basis für dynamische UIs + WYSIWYG-Editor. Window-Drag an der Titelleiste, Z-Order (Klick bringt nach vorne), Fokus, Schliessen-Button, **resizeable Fenster** (`GUI_WINDOW_RESIZABLE` — am unteren-rechten Griff ziehbar, `GUI_WINDOW_SET_MIN_SIZE`/`MAX_SIZE` als Grenzen; in `.gbform`-JSON als `resizable`/`min_w`/`min_h`/`max_w`/`max_h`) + **Control-Anchoring** (`GUI_SET_ANCHOR(wdg, "lrtb")` — Reflow beim Resize: Widgets kleben an Kanten/dehnen sich; in JSON als `anchor`-Edge-String) + **randlos** (`GUI_WINDOW_CHROME(win, an)` — ohne Titelleiste/Rahmen, damit eine Form das OS-Fenster füllt; der Form-Designer-Run koppelt die Form so ans native OS-Fenster). **Widgets werden auf den Fenster-Innenbereich geclippt.** Cyan-Theme (programmierbar). Konstruktoren/Getter sind `@builtin`, nur `GUI_UPDATE`/`GUI_DRAW` sind `graphics_builtin`. Komplement zum Immediate-Mode-`ui`-Modul (dort `UI_WINDOW_BEGIN/END` + `UI_TABLE` mit `UI_TABLE_SELECTED`/`SET_SELECTED`/`HEADER_CLICK`). Doku `docs/module-gui.md`, Demos `examples/45_gui.gb` + `examples/81_table_select.gb` + **`examples/156_gui_alle_widgets.gb`** (alle 22 Widget-Arten in EINER Vollbild-Anwendung, jedes mit echter Aufgabe: Baum filtert Tabelle, Tabellenzeile fuellt Editor, Regler formen die Kurve auf der Zeichenflaeche -- der schnellste Weg, eine Widget-Art in Aktion zu sehen), Tests `tests/test_gui_*.py`.


**Module sind in gbrt/Rust implementiert** (`rust/gb_runtime/src/<modul>.rs` +
Dispatch in `vm.rs` `try_<modul>`; externe Typen + ihr Default in `vm.rs`/
`value.rs`). Neues Modul: `.rs` schreiben, im `vm.rs`-`CALL_BUILTIN`-Dispatch
einhängen, den Modul-Namen in `rust/gb_runtime/src/preprocess.rs` MODULES **und**
in `gamebasic/modules/__init__.py` `KNOWN_MODULES` ergänzen (synchron halten —
sonst erkennt der Preprocessor `IMPORT "modul"` nicht). Dann Golden-Test +
`builtin_index.json`.

**IMPORT-Auflösung in [preprocess.py](gamebasic/preprocess.py):**
Der Pfad wird **wörtlich** aufgelöst — es wird KEINE `.gb`-Endung angehängt.
1. Existiert der geschriebene Pfad als Datei? → textuelles Inkludieren
   (Quellcode-Modul). Dafür muss die Endung mitgeschrieben werden:
   `IMPORT "helfer.gb"`.
2. Sonst: ist der Name ein bekanntes Built-in-Modul (`KNOWN_MODULES`)? → die
   `IMPORT`-Zeile wird zu einem Kommentar (gbrt kennt das Modul nativ).
3. Sonst: Fehler.

Daraus folgt: **`IMPORT "json"` nimmt IMMER das eingebaute Modul**, auch wenn
ein `json.gb` daneben liegt — die Endung fehlt, also greift Regel 1 gar nicht.
Wer ein Built-in mit eigenem Code überschreiben will (z.B. für Tests), muss die
Endung schreiben: `IMPORT "json.gb"`. Beide Engines verhalten sich identisch
(in beiden verifiziert).

## Verfügbare Built-in-Module

| Modul | Funktionen (Auswahl) | Externer Typ |
|---|---|---|
| `json` | `JSON_PARSE/LOAD/STRINGIFY`, `JSON_GET_STRING/INT/FLOAT/BOOL`, Pfad-Notation `"user.name"` / `"items.0"` | `JSON_HANDLE` |
| `db` | SQLite. `DB_OPEN/CLOSE`, `DB_EXEC/QUERY` mit `?`-Binding, `DB_NEXT`, `DB_GET_*`, `DB_BEGIN/COMMIT/ROLLBACK` | `DB_CONN`, `DB_RESULT` |
| `tween` | Werteinterpolation. 13 Easings (`linear`, `out_bounce`, `out_elastic`, …), Pause/Resume/Reverse | `TWEEN` |
| `timer` | Geplante Aktionen ohne MILLIS-Buchführung: `TIMER_AFTER/EVERY(ms, fnref)` → ID (FUNCREF-Callbacks, parameterlos), `TIMER_UPDATE()` pro Frame feuert die fälligen (Muster wie INPUT_UPDATE/GUI_UPDATE; EVERY max. 1×/Update, kein Aufhol-Burst), `TIMER_CANCEL/ACTIVE/COUNT/CLEAR` (Tombstone-stabile IDs). Plus `COOLDOWN(id$, ms)` — String-ID-Ratenbegrenzer (TRUE wenn frei, startet dann die Sperre; braucht kein UPDATE). Konsolen-tauglich (kein Grafik-Bezug; `rust/gb_runtime/src/timer.rs` + `try_timer` in vm.rs). Doku `docs/module-timer.md`, Demo `examples/113_timer.gb`, Tests `tests/test_modules_timer.py`. | — |
| `imgfx` | `IMAGE_SCALE/ROTATE/FLIP/TINT/COPY` — immutable, geben neues IMAGE zurück. **`IMAGE_SCALE` glaettet bilinear** (raylib `ImageResize`); fuer Pixelgrafik `IMAGE_SCALE_NN` (Nearest-Neighbour, `ImageResizeNN`) — Demo `examples/152_pixelart_skalierung.gb` | — |
| `particles` | Emitter mit Velocity/Lifetime/Gravity/Color/Size/Fade. `PARTICLE_EMIT/UPDATE/DRAW`. NumPy-vektorisiert. **Render-Modi** `PARTICLE_SET_MODE` (`circle`/`pixel`/`square`/`streak`/`glow` — `glow` wird in `PARTICLE_DRAW` (vm.rs) aktuell identisch zu `circle` gerendert, kein additives Blending im Recording-Modell; fuer echtes additives Leuchten `BLEND_MODE("add")` um die `PARTICLE_DRAW`-Aufrufe legen) + **Farbverlauf** `PARTICLE_SET_COLOR_END` (Start→End ueber die Lebenszeit, z.B. Feuer gelb→rot). | `PARTICLE_SYSTEM` |
| `physics` | Pure Functions: AABB-/Circle-Collision, Distance, Reflect, Normalize, Ray-Cast (Box+Circle). Kein State. Auch **3D-Mathematik ohne Physik-Welt**: `PHYSICS_SPHERE_SPHERE(x1,y1,z1,r1, x2,y2,z2,r2)` (Kugel-Naeherung), `PHYSICS_DISTANCE3`; dazu `PHYSICS_POINT_TRI(px,py, ax,ay, bx,by, cx,cy)` (Punkt im Dreieck, baryzentrisch — unabhaengig vom Umlaufsinn). Plus **Broadphase** (`PHYSICS_BROAD_NEW/ADD/QUERY/PAIR_A/PAIR_B`): O(n)-Kollisionspaare fuer viele Kreis-Entities (Uniform-Grid, nativ via `gb_native`). | `PHYSICS_BROAD` |
| `physics3d` | **Echte 3D-Starrkoerper-Physik via Rapier3D** (voller Solver: Schwerkraft, Integration, Kollisionsaufloesung, Restitution/Reibung — kein blosses Kollisions-Toolkit wie `physics`). `PHYS3D_NEW`, `PHYS3D_SET_GRAVITY`, `PHYS3D_ADD_BOX`/`ADD_SPHERE(... , dynamic, bounce)`, `PHYS3D_STEP(w, dt)`, `PHYS3D_BODY_X/Y/Z` + `BODY_QX/QY/QZ/QW` (Quaternion -> `MAT4_TRS`/`MODEL_MATRIX`), `PHYS3D_SET_VEL`/`APPLY_IMPULSE`/`SET_POS`/`REMOVE`/`COUNT`. Koerper-Index stabil (Tombstones). Rapier3D ist pure-Rust (nalgebra) -> ungated in gbrt. Demo `examples/107_physics3d.gb`, Tests `tests/test_physics3d.py`. | `PHYS_WORLD` |
| `physics2d` | **Echte 2D-Starrkoerper-Physik via Rapier2D** (voller Solver wie `physics3d`, nur 2D — fuer Stapeln/Werfen/Rollen/Sandbox; nicht zu verwechseln mit `physics` = nur Kollisions-Mathe). `PHYS2D_NEW`, `PHYS2D_SET_GRAVITY(w,gx,gy)`, `PHYS2D_ADD_BOX(w,x,y,hw,hh,dynamic,bounce)`/`ADD_CIRCLE(w,x,y,r,...)`, `PHYS2D_STEP(w,dt)`, `PHYS2D_BODY_X/Y/ANGLE/VX/VY`, `PHYS2D_SET_VEL`/`APPLY_IMPULSE`/`SET_POS`/`LOCK_ROTATION`/**`SET_DYNAMIC`**/`IS_DYNAMIC`/`REMOVE`/`COUNT` (`SET_DYNAMIC` schaltet statisch<->dynamisch um -- fuer Aufbauten, die erst stehen und dann zusammenfallen). **Bildschirm-Konvention** (Y unten, Default-Gravitation 0/980), `length_unit=100` fuer Pixel-Stabilitaet; Box-Maße = Halb-Extents; `dynamic`-Flag akzeptiert TRUE/FALSE oder 1/0 (Helfer `need_flag`). Koerper-Index stabil (Tombstones). Rapier2D pure-Rust -> ungated. Doku `docs/module-physics2d.md`, Demo `examples/112_physics2d.gb`, Tests `tests/test_physics2d.py`. | `PHYS2D_WORLD` |
| `camera` | World-Translation+Zoom+**Rotation** für **alle** Drawing-Befehle. `CAMERA_SET/RESET/FOLLOW`, `CAMERA_SET_ROTATION`/`CAMERA_ROTATION`, `CAMERA_S2W_X/Y`. Rotation dreht nur Positionen (um die Bildschirm-Mitte), keine automatische Kontur-Rotation von Formen/Sprites — siehe `docs/module-camera.md`. | — |
| `sprite` | Animiertes Sheet-basiertes Sprite. Position+Velocity, benannte Animationen mit FPS, `PLAY`/`PLAY_ONCE`, Flip, AABB-Kollision | `SPRITE` |
| `animfsm` | **Animations-State-Machine** (Unity-Mecanim-Stil), datengetrieben aus `.gbanim`-JSON (Editor `gbanim`): States (an Sprite-Anim gebunden) + Parameter (`bool`/`float`/`int`/`trigger`) + Transitions mit Bedingungen (`gt`/`lt`/`eq`/…, Any-State `*`, `wait_finished` für one-shot). `ANIM_FSM_LOAD/SETUP/UPDATE(fsm,sprite,dt)/SET_*/TRIGGER/STATE/FORCE`. Doku `docs/module-animfsm.md`, Demo `examples/111_anim_fsm.gb`, Tests `tests/test_animfsm.py`. | `ANIM_FSM` |
| `ui` | Immediate-Mode-UI. `UI_LABEL`, `UI_BUTTON`, `UI_CHECKBOX`, `UI_SLIDER` mit String-IDs für State. Pflicht: `UI_END_FRAME()` vor `FLIP()`. **Plastischer Look wie `gui`:** Themen `glas_dunkel`/`glas_hell` + Metriken `gradient`/`gloss`/`bevel`/`corner_radius` (0 = flach, alle alten Themen unveraendert); ein Preset setzt Farben UND Plastik. Gemeinsame Flaechen-Routine `ui_flaeche` in vm.rs (erhaben/versenkt). **Gotcha:** die Plastik-Werte muessen VOR `self.gfx.as_mut()` gelesen werden -- der Aufruf leiht `self` veraenderlich aus, danach ist `self.ui_state` nicht mehr lesbar (daher `UiPlastik` als Buendel). | — |
| `scene` | Stack-basierter Scene-Manager. `SCENE_PUSH/POP/SWITCH/CURRENT`, pro-Scene-Daten via `SCENE_SET_INT/FLOAT/STRING/BOOL` + `_OR`-Variante. | — |
| `save` | Persistente Save-Slots, JSON-Backend, Versionsfeld. `SAVE_NEW/LOAD/LOAD_OR_NEW/WRITE`, `SAVE_SET/GET_INT/FLOAT/STRING/BOOL`. | `SAVE_HANDLE` |
| `astar` | A*-Pathfinding auf Tile-Grid. `ASTAR_NEW/SET_WALL/FIND/PATH_X/PATH_Y`. Manhattan/Euclid/Chebyshev, Diagonal-Toggle, Anti-Cornercutting. | `ASTAR_GRID` |
| `vec2` | 2D-Vektor mit Operator-Overloading (`+`, `-`, `*`, `/`, `=`, `<>`). `VEC2_NEW/X/Y/LENGTH/NORMALIZE/DOT/CROSS/DISTANCE/LERP/PERP/REFLECT/ANGLE/FROM_ANGLE`. Immutable. | `VEC2` |
| `m3d` | 3D-Mathe: **VEC3/VEC4/QUAT/MAT4** (immutable, Operator-Overloading `+ - * / = <>`, inkl. `mat*mat`/`mat*vec`/`quat*quat`). Quaternionen (`QUAT_FROM_AXIS_ANGLE/EULER/SLERP/ROTATE_VEC3`), Matrizen (`MAT4_TRS/MUL/INVERT/LOOKAT/PERSPECTIVE/ORTHO/...`, column-major). Rendering via **`MODEL_MATRIX(handle, mat[, tint])`** (hierarchische Transforms/Bones/Gizmos) + **`MODEL_INSTANCED(handle, mats[, tint[, anzahl]])`** -- `tint` darf eine Farbe ODER ein `ARRAY OF INTEGER` sein (eine Farbe je Matrix); die Laufzeit gruppiert dann nach Farben und zeichnet **einen Draw-Call je VERSCHIEDENER Farbe**, nicht je Instanz (raylibs `DrawMeshInstanced` uebertraegt nur Matrizen, keine Farb-Attribute -- bei sehr vielen verschiedenen Farben ist ein Verlauf im Shader die bessere Antwort). Echtes GPU-Instancing: dasselbe Mesh mit N MAT4-Welt-Matrizen aus einem `ARRAY OF MAT4`/`TUPLE` in EINEM Draw-Call via raylib `DrawMeshInstanced`; eigener schlanker Instancing-Shader mit Ambient+bis-4-Lichtern, kein PBR/IBL/Schatten/Normal-Maps) + **`CAMERA3D_VIEW/PROJECTION(mat)`** (Ortho/Custom-Frustum) — native-only (gbrt). Doku `docs/module-m3d.md`, Demos `examples/103_m3d.gb` + `examples/104_instancing.gb`, Tests `tests/test_m3d.py`. | `VEC3`/`VEC4`/`QUAT`/`MAT4` |
| `input` | Action-basiertes Input-Mapping mit Edge-Detection. `INPUT_BIND/UNBIND/UPDATE`, `INPUT_HELD/PRESSED/RELEASED/AXIS/BOUND`. Multi-Key-Bindings. **Gamepad-Support**: `JOY_BUTTON_A..Y`, `JOY_DPAD_*` als Bind-Codes, `INPUT_JOY_AXIS(slot, "left_x")` mit Deadzone. | — |
| `regex` | Python-kompatible Pattern-Matching. `REGEX_MATCH/TEST/FIND/FIND_ALL/REPLACE/REPLACE_ONCE/SPLIT`. Pattern-Cache fuer wiederholte Aufrufe. | — |
| `audio` | Erweiterte Audio-API (nativ in gbrt ueber **Kira**/cpal -- eigener Audio-Thread, vom Game-Loop entkoppelt; loeste 2026-06-13 raylib-Audio ab, `rust/gb_runtime/src/audio.rs`). Channels, Pause/Resume/Fade (native Kira-Tweens), Stereo-Pan, Music-Position. Tone-Generation (`AUDIO_TONE`/`AUDIO_NOISE`) mit Sine/Square/Saw/Triangle/Noise. **`AUDIO_SFX`** -- prozeduraler sfxr-Stil-Synth (Waveform + Pitch-Slide + ADSR + Vibrato + optionale `stereo_width` fuer breiten Stereo-Sound; geteilte Mathematik in `gamebasic/synth.py`; der SFX-Generator `gbsfx` exportiert solche Aufrufe, Pan via `AUDIO_PAN`). Liefert kompatible `SOUND`-Objekte (auch fuer `PLAYSOUND` nutzbar). **Tracker-Module** `.mod`/`.xm` laufen ueber `PLAYMUSIC`/`AUDIO_MUSIC_LOAD` in **Echtzeit gestreamt** (Kira-Custom-`Sound` `ModuleSound` pollt den reinen Rust-Player `xmrs`/`xmrsplayer` auf dem Audio-Thread; sofort geladen, exaktes Endlos-Loopen, Pitch-Resampler + Volume-Ramp im Sound, Steuerung via `Arc<ModShared>`-Atomics, Modul geleakt + im Drop freigegeben) -- echter 4-Kanal-Amiga-Sound, Demo `examples/115_modplayer.gb`. **Sampler `SAMPLE_*`** (Amiga/Paula-Prinzip): `SAMPLE_LOAD(pfad$)->SAMPLE`, `SAMPLE_PLAY(sample, halbtoene, vol[, dur_ms])->AUDIO_CHANNEL` (Resampling per linearer Interpolation = Tonhoehe wie Geschwindigkeit; resampelte Noten gecacht), `SAMPLE_SET_LOOP`/`SAMPLE_LEN`. One-Shot (dur<=0) fuer Drums/Hits, dur>0 + Loop-Region fuer gehaltene Noten. Reine Resampling-Mathematik = freie `resample()` in audio.rs (Rust-`#[test]`); Demo `examples/116_sampler.gb`. **Paula-Lo-Fi** `AUDIO_LOFI(an[, bits[, cutoff_hz]])` -- Bit-Crush (Default 8-bit) + LED-Tiefpass (Default 3300 Hz) fuer NEU synthetisierte Sounds (TONE/NOISE/SFX/SAMPLE_PLAY; Cache wird geleert); pure `lofi_chain()` mit Rust-`#[test]`. **Mixer-Busse** `AUDIO_BUS_VOLUME(bus$, vol)`/`AUDIO_BUS_GET_VOLUME(bus$)` mit `bus$` = `sfx`/`music`/`master` -- SFX-/Musik-Master getrennt (Kira-Sub-Tracks: SFX/Sampler/Synth -> sfx_track, Musik -> music_track, beide -> Main mit dem FFT-Tap; Bus×Sound-Volume multiplizieren). **Echtzeit-Effekte je Bus** (Kira-Effektkette am Track, live steuerbar, kein Buffer-Bake): `AUDIO_FILTER(bus$, cutoff_hz[, resonance])` (Tiefpass, SID/Acid-Sweep), `AUDIO_REVERB(bus$, mix[, feedback[, damping]])` (Hall), `AUDIO_DELAY(bus$, mix[, feedback[, time_ms]])` (Echo, eigener Ringpuffer-Effekt -> Zeit zur Laufzeit aenderbar, 1..4000 ms), `AUDIO_DISTORTION(bus$, amount[, mix])` (Overdrive/Fuzz), `AUDIO_COMPRESSOR(bus$, threshold_db, ratio[, makeup_db])` (Dynamik, ratio<=1=aus), `AUDIO_EQ(bus$, freq_hz, gain_db[, q])` (Glocken-EQ, gain 0=transparent); Signalfluss EQ->Filter->Distortion->Compressor->Reverb->Delay, neutral bis aktiviert, Demo `examples/117_audiofx.gb`. **Clock** `AUDIO_CLOCK_NEW(ticks_per_second)->AUDIO_CLOCK` (Kira-Uhr fuer sample-genaues Musik-/Rhythmus-Timing; startet pausiert) + `AUDIO_CLOCK_START/PAUSE/STOP/REMOVE`, `AUDIO_CLOCK_TICKING`/`AUDIO_CLOCK_TICKS`, `AUDIO_CLOCK_SET_SPEED` -- und **`AUDIO_PLAY_AT(sound, clock, ticks[, volume[, loops]])`**: Sound-Start exakt auf einen Clock-Tick geplant, getrieben vom Kira-Audio-Thread selbst (KEIN Polling/Update-Call noetig -- anders als das frame-getriebene `timer`-Modul). BPM->ticks_per_second rechnet der Aufrufer selbst um (`bpm / 60.0 * subdivisions`). Ticking-Status wird im Wrapper selbst mitgefuehrt (nicht direkt Kiras `ClockHandle::ticking()`), weil Kira das nur asynchron per Audio-Thread-Kommando spiegelt -- eine Abfrage direkt nach START/STOP koennte sonst kurz den alten Wert zeigen. **Nicht-lineare Easings** fuer Fades/Slides: optionaler trailing `easing$`-Parameter (`"linear"` Default/`"in"`/`"out"`/`"inout"`, quadratisch) bei `AUDIO_PLAY` (fade_in_ms), `AUDIO_STOP` (fade_out_ms), `AUDIO_PAN_SLIDE` (dauer_ms), `AUDIO_MUSIC_PLAY`/`AUDIO_MUSIC_STOP` (fade_in/out_ms) -- vorher liefen alle Tweens linear, obwohl Kira `Easing::{In,Out,InOut}Powi` eingebaut hat. Interner Helfer `FadeCurve` (audio.rs) konvertiert zu `kira::Easing` fuer den Kira-Tween-Pfad (Stream/Static-Sounds) UND dupliziert dieselbe Kurven-Mathematik als reine `apply()`-Funktion fuer den MOD/XM-Modul-Fade (eigener Atomics-Ramp in `ModShared`, kein Kira-Tween beteiligt) -- beide Pfade klingen dadurch identisch. Rust-`#[test]`s verifizieren die Kurven-Mathematik gegen Kiras eigene Formel. **Raeumliches Audio (Listener/Emitter):** `AUDIO_LISTENER_NEW(x,y,z)->AUDIO_LISTENER` ("Ohr" der Szene, z.B. Kamera-/Spielerposition; unrotiert blickt -Z), `AUDIO_LISTENER_SET_POSITION`/`AUDIO_LISTENER_SET_ORIENTATION(listener, yaw_grad)` (nur Y-Achsen-Yaw -- deckt die typische Top-Down-/3rd-Person-Kamera ab, ohne BASIC-Nutzern volle Quaternionen zuzumuten) + `AUDIO_LISTENER_REMOVE`; `AUDIO_EMITTER_NEW(listener,x,y,z[,min_dist[,max_dist]])->AUDIO_EMITTER` (ein raeumlicher Kira-Sub-Track, an einen Listener + Position gebunden; Kira berechnet Panning + lineare Lautstaerke-Abnahme zwischen min_dist=laut/max_dist=lautlos komplett selbst -- keine eigene DSP) + `AUDIO_EMITTER_SET_POSITION`/`AUDIO_EMITTER_REMOVE`; **`AUDIO_PLAY_ON(sound,emitter[,loops[,volume[,fade_in_ms[,easing$]]]])->AUDIO_CHANNEL`** startet einen Sound auf dem Emitter-Track statt dem flachen SFX-Bus -- der zurueckgegebene `AUDIO_CHANNEL` ist danach identisch mit `AUDIO_PAUSE`/`STOP`/`VOLUME`/... steuerbar (`StaticSoundHandle` unterscheidet nicht, von welchem Track-Typ es kommt). Listener/Emitter im selben Tombstone-Vec-Pattern wie Clocks (Kira kennt weder `remove_listener()` noch `remove_spatial_sub_track()` -- nur Handle-Drop). `mint`-Crate (winzige, abhaengigkeitsfrei Interop-Structs) baut die Position/Rotation-Werte fuer Kiras API, ohne `glam` direkt einzubinden. Rust-`#[test]`s verifizieren `yaw_quat()` (Einheits-Quaternion, korrekte Komponenten). Demo `examples/139_audio_spatial.gb`. **Modulatoren (LFO + Tweener):** `AUDIO_LFO_NEW(wellenform$, hz [, amplitude [, mitte]])` -> `AUDIO_MOD` (`sine`/`triangle`/`saw`/`pulse`), `AUDIO_LFO_SET`, `AUDIO_LFO_WAVEFORM`; `AUDIO_TWEENER_NEW([start])` + `AUDIO_TWEENER_TO(mod, ziel, dauer_ms [, easing$])`; gebunden per `AUDIO_MODULATE(bus$, ziel$, mod, min, max)` mit ziel$ = **`volume`** (Tremolo) / **`pan`** (Auto-Pan) / `filter` / `resonance` / `reverb` / `distortion`; dazu `AUDIO_BUS_PAN(bus$, pos)` fuer eine feste Bus-Balance (-1..+1) -- vorher liess sich nur ein EINZELNER Kanal pannen, entfernt per `AUDIO_MOD_REMOVE`. Der Wertebereich des Modulators (LFO: -1..+1 bei Standard-Amplitude) wird auf `min..max` abgebildet. **Der Punkt daran:** Kira faehrt sie auf dem AUDIO-Thread -- Tremolo, Vibrato, Wobble-Bass, Auto-Pan und Filter-Sweeps laufen sample-genau weiter, auch wenn die Bildrate einbricht, und das GB-Programm rechnet pro Frame NICHTS nach. LFO und Tweener teilen sich den Handle-Typ `AUDIO_MOD`; ein LFO-Aufruf auf einem Tweener (und umgekehrt) meldet das im Klartext. Doku `docs/module-audio-modulatoren.md`, Demo `examples/150_audio_modulatoren.gb`. | `AUDIO_CHANNEL`, `SAMPLE`, `AUDIO_CLOCK`, `AUDIO_LISTENER`, `AUDIO_EMITTER`, `AUDIO_MOD` |
| `chart` | **Diagramme.** `CHART_NEW(art$,x,y,b,h)` -> `CHART` mit art$ = `kuchen`/`donut` (Kuchen/Ring), `balken` (senkrecht/waagerecht, gruppiert/gestapelt), `linie`/`flaeche` (Verlaufskurven, gleitendes Fenster fuer Live-Werte), `tacho` (Rundskala mit Zeiger `nadel`/`balken`/`pfeil`, Farbzonen via `CHART_ZONE`). Daten kurz (`CHART_ADD(c,name$,wert[,farbe])`) oder voll (`CHART_SERIES` + `CHART_DATA`/`CHART_PUSH`/`CHART_SET_POINT`); dazu `CHART_GET/COUNT/SERIES_COUNT/LABEL/CLEAR/BOUNDS/STAT`. **Stil ueber vier String-Setter statt ~40 Builtins:** `CHART_SET` (Text), `CHART_SET_NUM` (Zahlen), `CHART_SET_COLOR` (Farben), `CHART_SET_FLAG` (Schalter) -- Schluessel-Tabellen `KEYS_STR/NUM/COLOR/FLAG` in `chart.rs`, unbekannter Schluessel = Fehler, der die gueltigen auflistet. `CHART_THEME` (dunkel/hell/neon/pastell) + `CHART_PALETTE`. **Alpha/Schatten/Verlaeufe:** alle Farben nehmen `RGBA()` (0xAARRGGBB, Alpha 0 = DECKEND -- Helfer `with_alpha`/`scale_rgb` heben das vorher an); `deckkraft`/`flaeche_deckkraft` als globale Regler, `schatten`+`schatten_weich` (gestaffelte Kopien, raylib hat keinen Formen-Weichzeichner) mit `schatten_daten` auch fuer Balken/Segmente/Zeiger, `verlauf` (Hintergrund) und `verlauf_daten` (Balken/Flaeche senkrecht, Kuchen als abgedunkeltes Innenband = Naeherung, kein Radialverlauf). `animation` + `CHART_UPDATE(c, DELTA())` laesst Werte nachziehen -- **ohne Animation zeichnet `draw` direkt die echten Werte** (`anzeige()`), sonst waere CHART_UPDATE auch ohne Animationswunsch Pflicht. Nur `CHART_DRAW` braucht ein Fenster (in `vm.rs`), alles andere ist pure. Neues Zeichen-Primitiv dafuer: `Cmd::Ring` (raylib `draw_ring`) deckt Kuchenstueck/Donut/Tacho-Bogen ab, plus `text_width_at` (Breite bei expliziter Groesse). **Farbe `0` ist SCHWARZ, nicht "Palette"** -- dafuer `-1` bzw. Argument weglassen. **Sechs Arten** (nicht vier): dazu `leiste`/`bar_gauge` (liegende oder stehende Leiste mit wanderndem Marker) und `led`/`lampen` (diskrete Zellen, leuchten bis zum Wert) -- beide einwertig wie der Tacho, teilen sich dessen Farbzonen. Sie setzen `ausrichtung` selbst auf `waagerecht`, weil die Vorgabe `senkrecht` nur fuer Balkendiagramme richtig ist. **Skalen-Farbverlauf** ist eine EIGENE Farbrolle (`skala_von`/`skala_mitte`/`skala_bis`, rot->gelb->gruen je Thema) -- NICHT die Palette: die ist kategorial und ergibt interpoliert einen Regenbogen ohne Richtung. Farbzonen schlagen den Verlauf. **Tacho-Gestaltung:** `zifferblatt` = `ring`/`segmente`/`striche`/`baender`, `blatt_teile`/`blatt_luecke`/`blatt_dicke`, `fassung` (metallischer Ring aus gestaffelten Ringen -- ein Verlauf ENTLANG eines Kreises geht mit `ring` nicht), `CHART_ZONE(..., name$)` beschriftet die Zone entlang des Bogens (untere Haelfte wird gedreht), `wertanzeige` = `aus`/`innen`/`pille`/`blase`/`am_zeiger` (Pille nimmt die Farbe der getroffenen Zone). Der Tacho haengt allein an `wertanzeige` -- ihn zusaetzlich an `werte` zu koppeln liess ihn stumm, weil das per Vorgabe `aus` ist. **Maus:** `CHART_DRAW` wertet sie selbst aus (kein Zusatzaufruf) -> `CHART_HOVER`/`_SERIES`/`_LABEL$`/`_VALUE`, `CHART_CLICKED`/`_SERIES`; Schalter `hover`/`tooltip`, Zahlen `hover_tempo`/`hover_weite`/`hover_glanz`. Damit die Maus nicht neben dem trifft, was zu sehen ist, liegt die Geometrie an EINER Stelle (`kuchen_geom`/`kuchen_stuecke`/`achsen_geom`/`balken_geom`/`legende_abzug`), die Treffertest UND Zeichnen benutzen. Die Hervorhebung mischt gegen WEISS statt RGB zu skalieren -- beim Skalieren klemmt der groesste Kanal bei 255 und hervorgehobenes Orange wurde gelb. **Linien:** `punktform` (kreis/quadrat/raute/dreieck), `treppe`, `strich` (Strichlaenge; Phase laeuft ueber den GANZEN Zug weiter, sonst verdichtet sich das Muster bei engen Stuetzpunkten), `fadenkreuz`. `glatt`+`treppe` schliessen sich aus, die Treppe gewinnt. Doku `docs/module-chart.md`, Demo `examples/154_chart.gb`, Tests `tests/test_modules_chart.py` + Rust-`#[test]`s. | `CHART` |
| `curves` | Animation-Kurven (komplementaer zu `tween`'s Easings): `CURVE_BEZIER/BEZIER2`, `CURVE_CATMULL/CATMULL2`, `CURVE_HERMITE`, `CURVE_LERP`, `CURVE_SMOOTHSTEP`, `CURVE_SMOOTHERSTEP`. Pure Functions, kein State. | — |
| `net` | TCP + UDP via stdlib-Sockets (cross-platform). Default non-blocking fuer Game-Loops. `NET_TCP_LISTEN/ACCEPT/CONNECT`, `NET_SEND/RECV`, `NET_UDP_BIND/SEND/RECV`. Encoding: UTF-8. | `NET_LISTENER`, `NET_SOCKET`, `NET_UDP` |
| `mqtt` | **MQTT-3.1.1-Client** (das im Maker-/IoT-Bereich dominante Pub/Sub-Protokoll fuer ESP32/IoT-Steuerung) -- direkt gegen die OASIS-Spec via `std::net` implementiert, Feature `net` (bereits im Standard-Build, kein neues Crate). Nur **QoS 0** (kein Packet-ID-Ack-Handshake noetig), kein UNSUBSCRIBE/Will/TLS. `MQTT_CONNECT(host,port,client_id[,keepalive_s[,user[,pass]]])`, `MQTT_PUBLISH(h,topic,payload[,retain])`, `MQTT_SUBSCRIBE`, `MQTT_UPDATE` (Pro-Frame-Polling + automatisches Keepalive-PINGREQ), eingehende Nachrichten ueber Cursor-Muster wie `db` (`MQTT_NEXT_MESSAGE`/`MQTT_MESSAGE_TOPIC`/`MQTT_MESSAGE_PAYLOAD`, analog `DB_NEXT`+`DB_GET_*`). Doku `docs/module-mqtt.md`, Demo `examples/148_mqtt.gb`. | `MQTT_HANDLE` |
| `ecs` | Entity-Component-System. World mit Entity-IDs (INTEGER) und benannten typed Components (INT/FLOAT/STRING/BOOL/OBJ). Query 1/2/3-fach via Component-Intersection. `ECS_NEW_ENTITY`, `ECS_ADD_INT`, `ECS_QUERY2`, etc. Plus **Bulk-System-Ops** (`ECS_INTEGRATE_FLOAT`, `ECS_SCALE_FLOAT`, `ECS_FILL_*`, `ECS_CLAMP_FLOAT`, `ECS_REMOVE_DEAD`, `ECS_COUNT_WITH`) — siehe eigener Abschnitt unten. Reine Python-Implementation in `modules/ecs_py.py` (Cython entfernt); Produktions-Performance via `gbrt`. | `ECS_WORLD` |
| `html` | HTTP-GET/POST/DOWNLOAD + HTML-Parsing (pure stdlib). `HTTP_GET/POST/DOWNLOAD`, `HTTP_STATUS/HEADER`, `URL_ENCODE/DECODE`, `HTML_TEXT`, `HTML_FIND_ALL`. | — |
| `bt` | Bluetooth Low Energy (BLE) via `bleak`. Scan, Connect, Service/Characteristic-Listing, Read/Write/Notify auf Characteristics. Externer Dep, IoT/Sensor-Targets. | `BT_HANDLE` |
| `serial` | RS-232 / USB-COM nativ ueber die Rust-Crate `serialport` (kein `pyserial` noetig). `SERIAL_OPEN/READ/WRITE/READLINE/AVAILABLE/FLUSH/TIMEOUT`. | `SERIAL_HANDLE` |
| `firmata` | Direkte Arduino/ESP32-**Pin-Steuerung** ueber StandardFirmata (kein eigener Sketch/Text-Protokoll noetig -- einmalig StandardFirmata hochladen). Baut auf derselben `serialport`-Crate wie `serial` auf (Feature `serial`, keine neue Abhaengigkeit). `FIRMATA_PORTS/OPEN/CLOSE/IS_OPEN`, `FIRMATA_PIN_MODE`, `FIRMATA_DIGITAL_WRITE/READ`, `FIRMATA_ANALOG_WRITE/READ`, `FIRMATA_UPDATE` (Pro-Frame-Polling wie `INPUT_UPDATE`/`TIMER_UPDATE`). Nur Pin-I/O -- kein I2C/Servo/OneWire/Stepper/Encoder. **Zwei Nummerierungen** (echte Protokoll-Eigenheit): Schreiben nimmt die rohe digitale Pin-Nummer, `FIRMATA_ANALOG_READ` nimmt den Analog-**Kanal** (A0=0, A1=1, ...) -- nicht dieselbe Zahl fuer denselben physischen Pin. Doku `docs/module-firmata.md`, Demo `examples/147_firmata.gb`. | `FIRMATA_HANDLE` |
| `usb` | USB-HID via `hidapi`. Maker-Boards, Programmer, Custom-Controller. `USB_LIST/OPEN/READ/WRITE/PRODUCT`. | `USB_HANDLE` |
| `wifi` | WiFi-Management (Windows-only via `netsh wlan`). `WIFI_SCAN/CONNECT/DISCONNECT/CURRENT/SIGNAL/PROFILES`. | — |
| `tiled` | Tiled-Map-Loader (JSON-Format, kein TMX). `TILED_LOAD`, Layer-/Tile-/Object-Access, Per-Tile/Per-Object-Custom-Properties (`solid`, `damage`, ...). Industriestandard fuer 2D-Level-Design. Plus **Bulk-Ops** fuer Generierung/Editor: `TILED_FILL_RECT`, `TILED_REPLACE`, `TILED_COUNT_GID`, `TILED_FLOOD_FILL` (Bucket-Fill, nativ via `gb_native`). | `TILED_MAP` |
| `tile_collide` | Box-vs-Tilemap-Kollision. `TILE_SWEEP_X/Y` mit separat-Achsen-Sweep-Pattern. Solid-Detection via `solid`-Property (mit Convention-Fallback). Klassische Platformer-Physik. Sweep nativ via `gb_native.TileCollider` (Solid-Maske einmal gespiegelt+gecacht), sonst Python-`_sweep_axis`. | — |
| `controller` | Character-Controller mit Coyote-Time, Jump-Buffer, Variable-Jump-Height. `CHAR_NEW/SET_INPUT/UPDATE`, `CHAR_X/Y/VX/VY`, `CHAR_ON_GROUND/WALL_LEFT/RIGHT`. Konfigurable Move-Speed, Jump-Velocity, Gravity, Coyote/Buffer-Frames, Variable-Jump-Cut. | `CHAR_CONTROLLER` |
| `g3d` | **3D-Grafik, NUR native Runtime** (`gbrt` — Editor-Run F5 nutzt gbrt; der Tree-Walker-Fallback kann kein 3D und wirft eine klare Meldung). Immediate-Primitive: `CAMERA3D`, `CUBE`/`CUBE_WIRES`, `SPHERE`/`SPHERE_WIRES`, `CYLINDER` (Kegel via r_oben=0), `PLANE`, `LINE3D`, `POINT3D`, `GRID3D`. **3D-Modelle** (wiederverwendbare MODEL-Handles): `LOADMODEL` (OBJ/GLTF), prozedural `MESH_CUBE/SPHERE/CYLINDER/TORUS/KNOT/PLANE` + `MESH_HEIGHTMAP` (Terrain aus Graustufen-Image), zeichnen via `MODEL`/`MODEL_EX` (Achsen-Rotation)/`MODEL_WIRES`, `MODEL_TEXTURE` (Diffuse-Map aus LOADIMAGE). **Skelett-Animation** (geriggte GLTF/IQM): `MODEL_LOAD_ANIMS(pfad$)` -> ANIM_SET (Integer-Handle), `MODEL_ANIM_COUNT/NAME/FRAMES`, `MODEL_ANIMATE(modell, set, anim_idx, frame)` setzt die Pose (frame loopt). Nutzt seit raylib-rs 6.0 dessen RAII-`ModelAnimations`-Collection (`load_model_animations`/`update_model_animation`; Unload automatisch im `Drop` -- loeste den fruehreren rohen-FFI-Workaround ab, der noetig war weil der 5.x-Wrapper die Structs flach kopierte und dann `UnloadModelAnimations` rief -> Use-after-free). **`MODEL_ANIMATE_BLEND(modell, set, anim_a, frame_a, anim_b, frame_b, blend)`** (neu in raylib 6.0 via `UpdateModelAnimationEx`): blendet weich zwischen zwei Animationen desselben Sets (`blend` 0.0=ganz A .. 1.0=ganz B), z.B. fuer Walk->Run-Uebergaenge statt hartem Anim-Wechsel. Demo `examples/108_skeletal_anim.gb` (CC0-Modell via `examples/assets/download_robot.py`). **Billboards** `BILLBOARD` (Textur zeigt zur Kamera) + **Ray-Kollision/Picking** `RAY_HIT_BOX`/`RAY_HIT_SPHERE` (Distanz oder -1) und `PICK_BOX`/`PICK_SPHERE` (Mausstrahl, Klick-Selektion). **Picking auf echter Flaeche** (nicht nur Huellkoerper): `RAY_HIT_TRI(ursprung, richtung, 3 Punkte)`/`RAY_HIT_QUAD(ursprung, richtung, 4 Punkte)` + `PICK_TRI`/`PICK_QUAD` — Bodenkacheln, Wandstuecke, frei schwebende Panels. Ohne Backface-Culling (eine Flaeche trifft auch von hinten); die Vierecks-Punkte muessen **reihum** liegen; die Richtung wird vor dem Test normalisiert (sonst waere die Distanz in Vielfachen der Richtungslaenge, raylibs Rohverhalten). Demo `examples/151_picking_flaechen.gb`. **Beleuchtung** (PBR/Cook-Torrance, bis 4 Lichter): `LIGHT_ENABLE`/`LIGHT_AMBIENT`/`LIGHT_DIRECTIONAL`/`LIGHT_POINT`/`LIGHT_SET_POS/COLOR/ENABLED` + `MODEL_LIT(modell)` + `MODEL_PBR(modell, metalness, roughness)` (eingebetteter GGX-Shader) + `MODEL_EMISSIVE(modell, farbe, staerke)` (Eigenleuchten pro Modell — durchschlaegt den Fog; mit Bloom-`POSTFX` echter Neon-Glow, Demo `examples/110_emissive_glow.gb`) + `LIGHT_FOG(farbe, dichte)` (Tiefen-Fog) + `LIGHT_ENV(himmel, boden, intensitaet)` (analytisches IBL — Metalle spiegeln die Umgebung) + `LIGHT_ENV_HDR(pfad$ [, intensitaet])` (**echtes HDR-Cubemap-IBL**: laedt ein equirect-.hdr, berechnet Irradiance/Prefilter/BRDF-LUT-Maps, `useIBLMaps`-Gate; analytischer `LIGHT_ENV`-Pfad bleibt Fallback) + `SKYBOX(an)` (zeichnet die HDR-Umgebung als sichtbaren 3D-Hintergrund — env-Cubemap auf einen kamerazentrierten Wuerfel, ohne Depth-Write). **Schatten** `SHADOW_ENABLE([res])`/`SHADOW_AREA(groesse,dist)`/`SHADOW_TARGET(x,y,z)` (Shadow-Mapping via Depth-FBO + PCF; erstes directional Light wirft Schatten, MODEL_LIT-Modelle werfen+empfangen). **Normal-Mapping** `MODEL_TEXTURE_NORMAL(modell,bild)` (TBN-basiert, MODEL_LIT erzeugt Tangenten; useNormalMap-Gate -> lit Modelle ohne Map unveraendert). **Kamera-Modi** `CAMERA3D_UPDATE(mode)` (1=free/2=orbital/3=first_person/4=third_person, raylib UpdateCamera) + Getter `CAMERA3D_X/Y/Z`/`CAMERA3D_TARGET_X/Y/Z`. Render via raylib `begin_mode3D` beim FLIP (3D zuerst, 2D-HUD obenauf). Doku `docs/rust-runtime.md` (Schritt 6), Demos `examples/82_3d_intro.gb`, `88_3d_models.gb`, `90_billboards_picking.gb`, `91_lighting.gb`, `92_fog.gb`, `93_shadows.gb`, `94_normalmap.gb`, `95_pbr.gb`, `96_ibl.gb`, `99_ibl_hdr.gb`. | — |

**Zusätzlich als Core-Graphics-Built-ins** (kein IMPORT noetig, in gbrt
`vm.rs`/`try_graphics`):

| Bereich | Funktionen | Externer Typ |
|---|---|---|
| Asset-Cache | `LOAD_ASSETS(manifest.json)` — bulk-Preload mit Alias-Cache. `LOADIMAGE` / `LOADSOUND` cachen automatisch (rohem + abs Pfad). | — |
| Z-Layer | `LAYER_DEFINE(name, z)`, `LAYER(name)`, `LAYER_END()`, `LAYER_CLEAR(name)`. Layer-Surfaces mit SRCALPHA, FLIP composiert in z-Order und cleart. | — |
| Sprite-Atlas | `ATLAS_LOAD(manifest.json)` -> `SPRITE_ATLAS`. `ATLAS_DRAW(atlas, name, x, y)` zeichnet einzeln, Camera-aware. `BATCH_DRAW(...)`/`BATCH_FLUSH()` existieren aus Kompatibilitaet zur alten Python-Engine, sind in gbrt aber **kein echtes Batching**: `BATCH_DRAW` ist derselbe Dispatch-Arm wie `ATLAS_DRAW` (sofortiges Emit in den Layer-Command-Puffer), `BATCH_FLUSH()` ist ein No-Op (`vm.rs`: "Recording-Modell: alles flusht beim FLIP"). Kein separater Batch-Queue-Zustand, keine Draw-Call-Ersparnis gegenueber `ATLAS_DRAW`. | `SPRITE_ATLAS` |
| Bulk-Plot | `PLOTS(xs, ys, color [, anzahl])` — viele Pixel in EINEM Aufruf (vektorisiert), `color` = INT (alle gleich) oder ARRAY OF INT (pro Pixel). Groessenordnungen schneller als `PLOT` in einer Schleife (Starfields, Punktwolken). **`anzahl`** zeichnet nur die ersten n Eintraege — ohne sie wird IMMER das ganze Array gezeichnet, ein fest dimensionierter Puffer schleppt also seine ungenutzten Plaetze mit ins Bild. Ein Argument zu viel ist ein Fehler (frueher still ignoriert). | — |
| Bulk-Shapes | `BOXES(x1s,y1s,x2s,y2s,color[,anzahl])`, `CIRCLES(xs,ys,rs,color[,anzahl])`, `LINES(x1s,y1s,x2s,y2s,color[,anzahl])` — viele Shapes in EINEM Builtin-Call (spart den Dispatch pro Shape; gezeichnet wird pro Shape). `color` = INT oder ARRAY (darf laenger als `anzahl` sein). | — |
| Bulk-Tilemap | `TILED_FILL_RECT`, `TILED_REPLACE`, `TILED_COUNT_GID`, `TILED_FLOOD_FILL` (Bucket-Fill, nativ via `gb_native`) — siehe `tiled`-Modul. `DRAWTILEMAP` rendert intern via `blits()`-Batch (1 Call statt rows×cols). | — |
| 2D-Extras | **Nativ in gbrt:** `LINEW(x1,y1,x2,y2,breite[,c])` (dicke Linie), `BOXROUND`/`RECTROUND(x1,y1,x2,y2,radius[,c])` (runde Rechtecke gefuellt/Umriss), `GRADIENTV`/`GRADIENTH(x1,y1,x2,y2,c1,c2)` (Farbverlauf-Blocks), `SPLINE(xs,ys[,c[,breite]])` (Catmull-Rom durch Punkte). Demo `examples/100_2d_extras.gb`. | — |
| Blend-Modes | `BLEND_MODE(modus$)` — `"alpha"`/`"add"`/`"mult"`/`"subtract"` fuer folgende Draws (Glow via additiv). **Nur native** (raylib `BeginBlendMode`); Tree-Walker konsolen-only -> wirft "nur gbrt". | — |
| Prozedurale Texturen | **Nur native** (raylib `GenImage*`): `GENTEX_PERLIN(w,h,skala)`, `GENTEX_GRADIENT(w,h,c1,c2,vertikal)`, `GENTEX_CHECKED(w,h,fx,fy,c1,c2)`, `GENTEX_COLOR(w,h,c)`, **`GENTEX_RADIAL(w,h,inner,outer[,density])`** (radialer Verlauf Mitte→Rand — weiche Glows/Lichter/Vignetten, additiv gezeichnet) -> IMAGE-Handle (mit `DRAWIMAGE`/`DRAWIMAGEROT` nutzbar). Demo `examples/101_blend_gentex.gb`. | — |
| Clipboard / Drag&Drop | **Nur native**: `CLIPBOARD_GET()->STRING` / `CLIPBOARD_SET(text$)` (System-Zwischenablage), `FILES_DROPPED()->INTEGER` (Anzahl gedroppter Dateien dieses Frame) + `FILE_DROPPED(i)->STRING` (Pfad). Tree-Walker konsolen-only -> wirft "nur gbrt". | — |
| Render-Targets | **Nur native:** `RENDERTARGET_NEW(w,h[,behalten])->INTEGER` (Off-Screen-Render-Ziel; `behalten`=TRUE laesst den Inhalt ueber das Bild hinaus stehen -> **echte Rueckkopplung/Schweife**, `RENDERTARGET_CLEAR(rt[,farbe])` raeumt es von Hand), `RENDERTARGET_BEGIN(rt)` / `RENDERTARGET_END()` (folgende Draws ins Ziel — pro Frame transparent gecleart), `RENDERTARGET_DRAW(rt,x,y[,skala[,tint]])` (Ziel als Bild stempeln). gbrt: eigener Command-Buffer pro Target, beim FLIP vor der Hauptszene auf die RenderTexture gerendert (y-flip); Tree-Walker konsolen-only -> wirft "nur gbrt". Demo `examples/102_render_target.gb`. *Grenze:* RtDraw innerhalb eines anderen Targets = No-Op -- ein Target kann sich also auch NICHT selbst zeichnen. Schweife entstehen ueber `behalten`=TRUE plus Verblassen mit `BLEND_MODE("mult")` + Vollbild-`BOX` in dunklem Grau (Rezept + Tests: `tests/test_rendertarget_persistenz.py`). | — |
| Zustand sichern | `GFX_PUSH()` / `GFX_POP()` — Zeichenzustand auf einen Stapel legen und zurueckholen: 2D-Kamera+Ruetteln, aktive Layer, Hintergrundfarbe, Licht (Ambient/Nebel/alle Lichtquellen), Umgebung (`LIGHT_ENV`, IBL-Schalter, `SKYBOX`), Schatten (an/Bereich/Ziel), 3D-Kamera samt View-/Projektions-Ueberschreibung, Schrift und `POSTFX`. **Nicht** enthalten: geladene Ressourcen (bleiben geladen — POP schaltet nur ihre Benutzung zurueck), die Schatten-AUFLOESUNG (haengt am allozierten Tiefenpuffer) und der Blend-Modus (ohnehin nur ein Bild lang gueltig). Analog `AUDIO_PUSH()` / `AUDIO_POP()` fuer alle Bus-Einstellungen (Lautstaerke, Balance, Filter, Hall, Echo, Verzerrer, Kompressor, EQ) — eine laufende `AUDIO_MODULATE`-Bindung wird dabei abgeloest, weil das Zurueckschreiben denselben Kira-Parameter beschreibt (empirisch belegt in `tests/test_gfx_push_pop.py`). `GFX_DEPTH`/`AUDIO_DEPTH` liefern die Stapeltiefe, ein POP ohne PUSH ist ein Fehler. **Der Grund:** dieser Zustand ist global, und eine vergessene Ruecknahme faellt erst Szenen spaeter auf. | — |
| Fenster-Zustand | `WINDOW_FOCUSED()` (Spiel pausieren, wenn der Nutzer wegklickt), `WINDOW_MINIMIZED/MAXIMIZED/HIDDEN()`, `WINDOW_IS_FULLSCREEN()`, `WINDOW_FOCUS()` (nach vorne holen), `WINDOW_OPACITY(0..1)` (ganzes Fenster durchscheinend), **`WINDOW_ICON(bild)`** — ohne das trug jedes exportierte Spiel das raylib-Standardsymbol. `WINDOW_DPI_X/Y()` = Bildschirm-Skalierung (1.0 normal, 2.0 HiDPI/Retina — ohne sie weiss ein Programm nicht, ob seine Pixelgroessen auf dem Zielgeraet winzig herauskommen). `GET_TIME()` = monotone Sekunden seit Programmstart. `OPENURL(adresse$)` oeffnet den Standardbrowser — **bewusst auf http/https begrenzt**, weil raylib die Zeichenkette an die Shell weiterreicht und ein `file:`-Schema sonst ein Weg waere, aus einem GB-Programm Beliebiges zu starten. | — |
| Kompression | `COMPRESS$(text$)` / `DECOMPRESS$(gepackt$)` — DEFLATE, Ergebnis Base64 (GB-Strings sind UTF-8, roher Deflate-Output waere keins). Typisch ~9x kleiner bei Savegame-artigem Text; passt ueberall dorthin, wo heute schon `BASE64_ENCODE`-Ausgaben stehen. **Ungated** (miniz_oxide statt raylibs CompressData) — laeuft also auch in Konsolen-Programmen ohne Fenster. | — |
| Bild-Verarbeitung (Ausbau) | `IMAGE_CONVOLVE(bild, kern)` — freie Faltung mit quadratischem, ungerade-seitigem Kern als flachem `ARRAY OF FLOAT` (Schaerfen, Kanten, Praegen; `IMAGE_BLUR` kann nur Gauss). `IMAGE_ALPHA_MASK/CROP/PREMULTIPLY` (weiche Raender, eng zuschneiden, dunkle Saeume beim Skalieren vermeiden). `IMAGE_DITHER(bild, r,g,b,a)` — **nur 5,6,5,0 / 5,5,5,1 / 4,4,4,4**; raylib warnt bei allem anderen bloss und liefert ein Bild mit ungueltigem Format (Textur wird schwarz), deshalb hier hart abgelehnt. `IMAGE_PALETTE(bild, max)` -> `ARRAY OF INTEGER` der haeufigsten Farben. | — |
| Textur-Generatoren (Ausbau) | `GENTEX_CELLULAR(w,h,kachel)` (Voronoi/Zellrauschen — Steinboden, Risse), `GENTEX_NOISE(w,h,anteil)` (Weissrauschen — Sternenfelder, Korn), `GENTEX_GRADIENT_BOX(w,h,dichte,c1,c2)` (rechteckiger Verlauf von innen nach aussen — Vignetten; das eckige Gegenstueck zu `GENTEX_RADIAL`). | — |
| Bitmap-Fonts | `LOADFONT_IMAGE(bild, trennfarbe, erstes_zeichen)` — Pixel-Schrift aus einem PNG, dessen Zeichen durch die Trennfarbe getrennt sind. Bleibt bewusst ungefiltert (nearest), damit Pixel-Schrift pixelig bleibt — anders als `LOADFONT` (TTF), das bilinear glaettet. `TEXT_LINE_SPACING(px)` fuer mehrzeiligen Text. **Nicht umgesetzt:** animierte GIFs (`LoadImageAnim` liefert nur Bild 0 nutzbar, raylib-rs macht `Image` readonly) und `GetClipboardImage` (Windows-only) — Begruendungen stehen im Quelltext. | — |
| Shader-Uniforms (Ausbau) | `SHADER_SET_ARRAY(sh, name$, werte)` fuellt ein `uniform float[]` aus einem `ARRAY OF FLOAT` (Lichtpositionen, Verlaufsstufen — vorher liess sich pro Aufruf nur EIN Wert setzen). `SHADER_SET_TEXTURE(sh, name$, bild)` belegt einen **zweiten Sampler** (Masken, Paletten-LUTs, Ueberblendungen). `SHADER_SET_MATRIX(sh, name$, mat)` nimmt eine `MAT4` aus `m3d`. **Wichtig:** raylibs `SetShaderValueTexture` ruft intern `glUniform1i` und wirkt damit auf das GERADE AKTIVE Programm — ausserhalb von `BeginShaderMode` landet die Zuweisung am falschen Shader und der Sampler bleibt schwarz. gbrt merkt sie deshalb vor und setzt sie beim Zeichnen (`shader_textures` in graphics.rs). | — |
| Linien-/Polygon-Geometrie | Im `physics`-Modul, pure Functions ohne Fenster: `PHYSICS_LINES_HIT` (schneiden sich zwei **Strecken**?) mit `PHYSICS_LINES_X/Y` fuer den Schnittpunkt (**NAN** wenn es keinen gibt — erst HIT fragen), `PHYSICS_POINT_LINE(px,py, ax,ay, bx,by, dicke)`, `PHYSICS_CIRCLE_LINE`, und `PHYSICS_POINT_POLY(px, py, xs, ys)` (Strahl-Verfahren, funktioniert auch bei konkaven Polygonen). | — |
| Eingabe-Flanken | **"genau in DIESEM Frame"** statt "wird gehalten": `MOUSE_HIT(n)`/`MOUSE_RELEASED(n)`, `KEYHIT(c)`/`KEYRELEASED(c)`, `KEYREPEAT(c)` (+ System-Auto-Repeat), `JOYSTICK_HIT/RELEASED(idx,btn)`. **Achtung:** `MOUSEBUTTON` und `KEYPRESSED` melden weiterhin *gehalten* — die Namen sind historisch und behalten ihre Bedeutung. Dazu `JOYSTICK_ANY_BUTTON()` (zuletzt gedrueckter Knopf, -1 = keiner — fuer Belegungsdialoge) und `JOYSTICK_AXIS_COUNT(idx)`. **Belegungsdialoge auch fuer die Tastatur:** `KEY_ANY_HIT()` (Code der zuletzt gedrueckten Taste, -1 = keine) + `KEY_NAME$(code)` (Anzeigename; GLFW kennt nur die druckbaren und die layout-abhaengig, fuer Sondertasten hat gbrt eine eigene Tabelle: `LEER`/`LINKS`/`UMSCHALT`/`F5`/…). `JOYSTICK_MAPPINGS(sdl_db$)` laedt SDL-GameControllerDB-Zeilen nach. **Neue Tastencodes** (vorher gab es dafuer GAR KEINE Konstante, „Sprint mit Umschalt" war nicht abfragbar): `KEY_LSHIFT/RSHIFT`, `KEY_LCTRL/RCTRL`, `KEY_LALT/RALT`, `KEY_LSUPER/RSUPER`, `KEY_CAPSLOCK`, `KEY_INSERT/DELETE/HOME/END/PAGEUP/PAGEDOWN`, Ziffernblock `KEY_KP0..KEY_KP9` + `KEY_KP_ENTER/PLUS/MINUS/MULTIPLY/DIVIDE/PERIOD` (in `vm.rs` DEFAULT_KEYS **und** `gamebasic/graphics.py` KEYS — Drift-Schutz `tests/test_constants_sync.py`). | — |
| Eingabe aufzeichnen/abspielen | `AUTOMATION_RECORD(datei$)` / `AUTOMATION_STOP()` (schreibt die Datei, liefert die Anzahl) / `AUTOMATION_PLAY(datei$)` + `AUTOMATION_RECORDING/PLAYING/FRAME/COUNT` — raylibs Automation-Events (Tasten/Maus/Rad/Gamepad/Touch je Frame). Fuer Demo-/Attract-Modus, nachspielbare Fehlerberichte, automatische Spieltests. Eingespeist wird in `automation_tick()` am **Ende jedes FLIP** (direkt nach dem Einlesen der echten Eingabe -> aufgezeichnete Werte gewinnen; ein Ereignis aus Aufnahme-Frame N wirkt im Durchlauf N+1). Die Liste liegt in einer **Box**, weil `SetAutomationEventList` sich einen rohen Zeiger merkt. Aufnahme und Wiedergabe schliessen sich aus (raylib spielt waehrend einer Aufnahme nichts ab -> klare Fehlermeldung). Aufgezeichnet wird die EINGABE, nicht der Ablauf: Startzustand zuruecksetzen, `RANDOMIZE` festnageln, pro Frame statt pro Sekunde rechnen. **`KEY_ANY_HIT` blendet aus, was die laufende Wiedergabe selbst einspeist** (`auto_injected_keys` in graphics.rs) -- raylib legt eingespeiste Tasten auch in seine "zuletzt gedrueckt"-Warteschlange, ohne den Filter braeche ein Attract-Modus ("Demo endet bei Tastendruck") an seiner eigenen Demo ab; `KEYHIT`/`KEYPRESSED` sehen sie weiterhin, `JOYSTICK_ANY_BUTTON` ist nicht betroffen. Doku `docs/automation.md`, Demo `examples/153_automation.gb`, Tests `tests/test_automation.py` (schreiben die Aufnahmedatei selbst — raylibs Textformat). | — |
| Maus-Blick + Cursor | `MOUSE_DELTA_X/Y()` (relative Bewegung — bei `MOUSE_LOCK` stehen MOUSEX/MOUSEY still, nur das Delta bewegt sich noch), `MOUSE_SET_POS(x,y)`, `MOUSE_ON_SCREEN()`, `MOUSEWHEEL_X/Y()` (Rad in **beiden** Achsen und als Kommazahl — `MOUSEWHEEL` liefert nur vertikal + ganzzahlig, feine Touchpad-Schritte fielen darin auf 0), `MOUSE_CURSOR(form$)` mit `default`/`ibeam`/`crosshair`/`hand`/`resize_ew`/`resize_ns`/`resize_nwse`/`resize_nesw`/`resize_all`/`not_allowed`. | — |
| Touch + Gesten | `TOUCH_COUNT()`, `TOUCH_X/Y(i)`, `TOUCH_ID(i)` (stabile Finger-Kennung ueber Frames). `GESTURE$()` liefert einen **Namen** statt einer Zahl: `tap`/`doubletap`/`hold`/`drag`/`swipe_left|right|up|down`/`pinch_in`/`pinch_out` (`""` = keine). Dazu `GESTURE_DRAG_X/Y/ANGLE`, `GESTURE_PINCH_X/Y/ANGLE`, `GESTURE_HOLD_TIME()`. Demo `examples/149_input_edges.gb`. | — |
| Game-Loop | `DELTA()` — Sekunden seit letztem `FLIP` (framerate-unabhaengige Bewegung: `x = x + speed * DELTA()`). `FPS()` / `SETFPS(n)` (Ziel-Framerate, 0 = ungedrosselt). `SET_FULLSCREEN(an)`, `SETWINDOWTITLE(s$)`, `SAVESCREENSHOT(pfad$)`. **Natives OS-Fenster** (das SCREEN-Fenster selbst): `WINDOW_RESIZABLE(an)` (vom OS aus groessenveraenderbar), `WINDOW_MIN_SIZE(w,h)`/`WINDOW_MAX_SIZE(w,h)`, `WINDOW_MAXIMIZE/MINIMIZE/RESTORE()`, `WINDOW_RESIZED()->BOOL`; `SCREENWIDTH()`/`SCREENHEIGHT()` liefern die **live**-Groesse (waechst mit dem Fenster). Demo `examples/106_resizable_window.gb`. Nativ in gbrt (raylib). | — |
| Shader / Post-FX | **Nur native Runtime** (raylib/GPU): `SHADER_LOAD(pfad$_oder_glsl$)` -> SHADER-Handle (oder -1), `SHADER_SET(h, uniform$, f)` / `SHADER_SET2` (vec2) / `SHADER_SET3` (vec3), `POSTFX(h)` (Frame durch Fragment-Shader; -1 = aus). Szene -> RenderTexture -> Shader -> Screen. Tree-Walker konsolen-only -> wirft "nur gbrt". Beispiel-Shader `examples/assets/shaders/` (CRT/Bloom/Vignette), Demo `examples/86_postfx_shaders.gb`. | — |

Module mit eigenem Typ registrieren ihn lowercase (`register_type("json_handle", _JSONHandle)`),
GB-Code schreibt ihn in jeder Casing-Form (`DIM j AS JSON_HANDLE`).

## Convention: Wert-Typen in GB

| GB-Typ | Python-Typ | type-Spec |
|---|---|---|
| INTEGER | `int` (kein bool) | `"int"` |
| FLOAT | `float` | `"num"` (akzeptiert auch int) |
| STRING | `str` | `"str"` |
| BOOLEAN | `bool` | `"bool"` |
| Klasse / Externer Typ | Instanz / Handle | `"any"` (selbst prüfen) |
| ARRAY OF T | `_GBArray` | — (Parser-Form `array:T`) |
| MAP OF T | `_GBMap` | — (Parser-Form `map:T`) |
| FILE / IMAGE / SOUND | `_GBFile` / `_Image` / `_Sound` | — (eigene target-Strings) |
| SPRITE_ATLAS | `_SpriteAtlas` (image + frames-Dict) | — (eigener target-String `"sprite_atlas"`) |

**Mathe-Typen starten neutral:** `DIM m AS MAT4` ist die Einheitsmatrix,
`QUAT` das Einheits-Quaternion, `VEC2/3/4` der Nullvektor — nicht NIL. Genauso
wie INTEGER mit 0 und STRING mit `""` anfaengt; ein `DIM mats[N] AS MAT4` laesst
sich damit schrittweise fuellen. Eine Quelle fuer alle drei Wege (global, lokal,
Array-Element): `model::neutrales_element` (die Compiler-Konstanten koennen kein
MAT4 tragen, deshalb wird es beim Laden bzw. beim DECLARE nachgetragen).

**Bool ist KEINE Zahl** — `_check_num(True)` wirft, weil `isinstance(True, int)` zwar `True`
ist, aber `True` semantisch keine Zahl in GB ist.

## Camera-Wirkung auf Drawing

Wenn `CAMERA_SET` aufgerufen wurde, sind ab da alle Koordinaten in den
core-Grafik-Built-ins (`PLOT`, `LINE`, `BOX`, `RECT`, `CIRCLE`, `TEXT`,
`DRAWIMAGE*`, `DRAWTILEMAP`) **World-Koordinaten**. `TEXT` wird nur translatiert
(nicht gezoomt) — für scharfen HUD-Text vorher `CAMERA_RESET()`.

`PARTICLE_DRAW` ruft intern `g.circle()` und folgt der Camera automatisch.

## Build und Test

**Die Runtime `gbrt` bauen** (raylib, der einzige Ausführungspfad):
```
.venv\Scripts\python.exe rust\build_runtime.py
```
Baut `rust/gb_runtime/` → `gbrt`. Nötig für Run/Export/Editor-Run + die Tests
(run_gb spawnt `gbrt run`; skippen, wenn nicht gebaut). Details: docs/rust-runtime.md.

> Historisch: Es gab Python-Beschleuniger (Cython `array_native`/`ecs_native`,
> PyO3 `gb_native`) für die alten Python-Modul-Impls. Mit dem Entfernen des
> Tree-Walkers + der Module sind sie **obsolet** — die gesamte Performance liegt in
> `gbrt` (Rust). Kein `rust/build.py`/Cython-Schritt mehr nötig.

**Tests laufen lassen:**
```
.venv\Scripts\python.exe -m pytest tests/ -v
```

**Headless prüfen:** `GBRT_FRAMES=n GBRT_SCREENSHOT=p.png gbrt run x.gb` liefert
EIN Bild (ein Augenblick). Für alles, was sich über die ZEIT falsch verhält
(zu früh umkippen, stehenbleibende Ränder, ruckelnde Bewegung) stattdessen den
**Kontaktbogen**: `GBRT_FRAMES=480 GBRT_CONTACT=bogen.png gbrt run x.gb` setzt
mehrere Bilder beschriftet als Raster in eine PNG (`GBRT_CONTACT_MAX`,
`_COLS`, `_EVERY`). Details: docs/rust-runtime.md.

**Programm ausführen:** `.venv\Scripts\python.exe gbrun.py examples/<file>.gb`
(läuft über `gbrt run`). Direkt: `gbrt run datei.gb`. (Der frühere `--bench`-
Tree-Walker-Vergleich ist entfernt — es gibt nur noch gbrt.)

## Häufige Fallstricke

- **Grafik/Audio NUR in gbrt:** Konsolen-Programme (PRINT/INPUT/Logik) laufen voll;
  Grafik/Audio rendert raylib (Fenster). pygame ist raus; `graphics.py` (Python)
  hält nur noch `COLORS`/`KEYS` + Kamera-Mathematik fürs Editor-Tooling.
- **`step` ist Schlüsselwort** (FOR…STEP). Variablen entsprechend benennen
  (`i`, `iter`, `tick` statt `step`).
- **Neue Builtins/Sprach-Features NUR in gbrt** (`rust/gb_runtime/src/`):
  Builtin → `builtins.rs`/`vm.rs`; Sprach-Feature → `lexer.rs`/`parser.rs`/
  `ast.rs`/`compiler.rs`/`vm.rs`. Es gibt KEINE „beide Pfade"/Tree-Walker-Parität
  mehr — Korrektheit per **run_gb-Golden-Test** (`assert run_gb(src) == expected`)
  + ggf. Rust-`#[test]`. Bei neuem Keyword die VSCode-Grammatik regenerieren.
- **`run_gb`/`run_vm`/`run_native`/`run_all`-Fixtures** sind alle Aliase auf
  `gbrt run` (conftest); `run_gb(src, base=tmp_path)` legt die .gb in ein
  Verzeichnis, damit relative Fixture-Pfade (TILED_LOAD etc.) gefunden werden.
- **`IS NIL`/`IS NOT NIL` gibt es NICHT** als Parser-Konstrukt (Doku-Altlast) —
  nil-Check via `IS_NIL(x)`-Builtin.
- **Qt-Tests: nie ungebremst `app.processEvents()` aufrufen.** Die Qt-Testdateien
  lassen ihre Fenster stehen; in EINEM gemeinsamen `pytest tests/`-Prozess
  sammeln sich so tausende QObjects mit hunderten scharfen Timern (u.a.
  wiederholende 16-ms-Vorschau-Timer). Ein nacktes `processEvents()` laeuft
  dann NIE zurueck — der ganze Lauf haengt mit 100 % CPU. Wer wirklich pumpen
  muss: die `quiet_qt_process`-Fixture aus `tests/conftest.py` anfordern (stellt
  den Prozess vorher ruhig) UND mit Zeitgrenze pumpen
  (`processEvents(flags, ms)` in einer Schleife mit `QDeadlineTimer`) — Muster
  in `tests/test_spriteeditor_qt_canvas.py::_event_loop_tick`. Die autouse-
  Fixture `_qt_widget_cleanup` entschaerft Altlasten nach jedem Test; sie
  ZERSTOERT sie bewusst nicht (`deleteLater()` auf die Editor-Fenster crasht —
  echte Zerstoerungs-Reihenfolge-Fehler, noch offen).

## Coroutines / YIELD

Eine `FUNCTION`/`SUB`, deren Body ein `YIELD` enthaelt, ist eine **Coroutine**.
Ihr Aufruf fuehrt den Body NICHT aus, sondern liefert ein `COROUTINE`-Handle.

```basic
FUNCTION zaehler() AS INTEGER
    YIELD 1
    YIELD 2
    RETURN 99            ' Endwert (optional), via CORO_RESULT abrufbar
END FUNCTION

DIM c AS COROUTINE
c = zaehler()
PRINT CORO_RESUME(c)     ' 1
PRINT CORO_RESUME(c)     ' 2
PRINT CORO_RESUME(c)     ' 99 (beendet) -- CORO_DONE(c) ist jetzt TRUE
```

**API** (Builtins, kein neuer Opcode ausser `YIELD_VALUE`):
- `CORO_RESUME(c)` -- fortsetzen bis zum naechsten YIELD, liefert den YIELD-Wert
  (bzw. den RETURN-Wert, wenn die Coroutine in diesem Schritt endet).
- `CORO_SEND(c, v)` -- wie RESUME, aber der `YIELD`-**Ausdruck** im Body
  evaluiert zu `v`: `DIM x AS INTEGER : x = YIELD 5`. Der Sende-Wert des
  ERSTEN Resume ist immer NIL (wie Python -- nicht lesen).
- `CORO_DONE(c)` -- BOOLEAN, ob beendet (RETURN/Ende/CLOSE).
- `CORO_RESULT(c)` -- finaler RETURN-Wert (wirft, wenn noch nicht beendet).
- `CORO_CLOSE(c)` -- suspendierte Coroutine abbauen (raeumt den Worker-Thread).
- `FOR EACH v IN coro` / Comprehensions -- treiben die Coroutine **eager** bis
  zum Ende (RETURN-Wert nicht enthalten). Vorsicht bei unendlichen Generatoren
  -- dort `CORO_RESUME`/`CORO_DONE` manuell verwenden.

**`YIELD` ist ein Ausdruck** (niedrige Praezedenz): `YIELD v` als Statement
verwirft den Sende-Wert; `x = YIELD v` liest ihn. Operand optional (`YIELD`).

**Mechanismus:** Jede Coroutine laeuft auf einem eigenen **Daemon-Thread** mit
striktem Ping-Pong (Queues; immer nur ein Thread laeuft gleichzeitig). Dadurch
bleibt die Ausgabe deterministisch und **bit-identisch** ueber Tree-Walker,
Python-VM und Cython-VM. Folgen davon:
- **Kein Cross-Frame-Yield:** ein Helfer mit `YIELD` ist selbst eine Coroutine
  (sein Aufruf liefert ein Handle), `YIELD` laeuft also nie ueber einen
  normalen Call hinweg.
- **Typ-Coercion:** in `FUNCTION ... AS T` werden YIELD- UND RETURN-Werte auf
  `T` gecoerct (ein Typ fuer beide Kanaele). SUB-Coroutinen yielden "any".
- **Idiom:** ein manueller `WHILE NOT CORO_DONE(c)`-Resume-Loop bekommt beim
  letzten (beendenden) Aufruf den RETURN-Wert; gibt der Generator einen
  typisierten `RETURN` zurueck, klappt die Zuweisung an eine typisierte
  Variable -- sonst `FOR EACH` nutzen.

**Implementierung:** `_Coroutine` + `function_has_yield` in
[interpreter.py](gamebasic/interpreter.py); Erzeugung in den `CALL_*`-Pfaden
aller Engines (is_coroutine-Branch), Treiben ueber die `CORO_*`-Builtins.
Pro-Thread-State (`env`/`call_depth`/`_method_stack`) im Tree-Walker liegt in
`threading.local`.

**Auch nativ (gbrt/Rust, `--native` + Standalone-`.exe`).** Statt Threads nutzt
die Rust-VM einen **Frame-Snapshot**: `dispatch` liefert `Step::Return | Yield`;
bei YIELD wird der Frame (ip/locals/stack/try_handlers) in einem
`Value::Coroutine` (`CoroState`) abgelegt und beim Resume restauriert. Moeglich
ist das, weil **kein Cross-Frame-YIELD** existiert -- nur der oberste
Coroutine-Frame muss fortsetzbar sein (verschachtelte Calls laufen normal
rekursiv). Kein OS-Thread -> raylib-Main-Thread bleibt sicher, deterministisch
per Konstruktion. Tree-Walker (thread-basiert) und gbrt (Frame-Snapshot) liefern
bit-identisch.

Use-Cases: Cutscene-DSL, prozedurale Generation, Boss-Patterns, NPC-Dialoge.
Doku-Demo [examples/98_coroutines.gb](examples/98_coroutines.gb), Tests
[tests/test_coroutines.py](tests/test_coroutines.py).

## Input-Mapping (Modul `input`)

Statt hardcoded Keycodes ueberall (`KEYPRESSED(1073741904)`), bindet das
input-Modul Tastenkombinationen an benannte Actions:

```basic
IMPORT "input"
INPUT_BIND("move_left",  KEY_LEFT,  KEY_A)
INPUT_BIND("jump",       KEY_SPACE, KEY_W)

' --- Pro Frame ---
INPUT_UPDATE()                          ' Snapshot
IF INPUT_PRESSED("jump") THEN ...       ' Edge: gerade JETZT
IF INPUT_HELD("move_left") THEN ...     ' Held: dauerhaft
PRINT INPUT_AXIS("move_left", "move_right")   ' -1, 0, +1
```

**Edge-Detection** funktioniert ueber den `INPUT_UPDATE()`-Call am
Frame-Start: das Modul vergleicht den aktuellen Snapshot mit dem
vorigen. Ohne UPDATE bleiben PRESSED/RELEASED bei FALSE haengen.

**Multi-Key-Bindings:** Eine Action kann an N Tasten gebunden sein --
trifft sobald irgendeine davon gedrueckt ist.

**Action-Namen** sind case-insensitive (lower-case-Vergleich). Re-BIND
ueberschreibt die alte Liste.

**Beispiel:** [examples/59_input.gb](examples/59_input.gb).

## VEC2 + Operator-Overloading

Im `vec2`-Modul liefert `VEC2_NEW(x, y)` einen immutable 2D-Vektor.
Die arithmetischen Operatoren `+`, `-`, `*` (Skalar), `/`, `=` und `<>`
sind fuer VEC2 ueberladen:

```basic
IMPORT "vec2"
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(3.0, 4.0)
w = VEC2_NEW(1.0, 2.0)
PRINT v + w           ' Vec2(4.0, 6.0)
PRINT v * 2.0         ' Vec2(6.0, 8.0)
PRINT VEC2_LENGTH(v)  ' 5.0
```

**Operator-Hooks:** Vec2 registriert seine Operatoren ueber die
**Operator-Registry** (siehe Abschnitt unten) -- der Dispatch ist NICHT mehr
hardcoded. Generisches User-Operator-Overloading auf beliebigen Klassen gibt
es ebenfalls (`OPERATOR + (...) END OPERATOR`, siehe „Operator-Overloading auf
User-Klassen"). Wer einen weiteren mathematischen Wert-Typ als Modul einbaut,
ruft `register_operators(Typ, {...})` in seinem Modul -- ohne Eingriff in
interpreter.py / vm.py / vm_native.pyx.

**Werte sind immutable** -- `w = v` aliased nicht; jede Operation erzeugt
ein neues VEC2.

**Beispiel:** [examples/58_vec2.gb](examples/58_vec2.gb).

## Function References (FUNCREF)

User-Functions als first-class Werte fuer Higher-Order-Patterns:

```basic
FUNCTION square(x AS INTEGER) AS INTEGER
    RETURN x * x
END FUNCTION

DIM f AS FUNCREF
f = square            ' bare Identifier wird zur FUNCREF
PRINT f(5)            ' 25 -- Aufruf via Variable
```

Use-Cases: Sort-Comparator, Tween-Easing-Callbacks, Event-Handler.

```basic
FUNCTION twice(g AS FUNCREF, x AS INTEGER) AS INTEGER
    RETURN g(g(x))
END FUNCTION
```

**Closures werden NICHT unterstuetzt** -- der Body sieht nur Parameter und
Globals/CONST. Wer Closure-Verhalten braucht, uebergibt Werte explizit als
Parameter. Das gilt konsistent in allen drei Pfaden -- damit bleibt die
"ALLE IDENTISCH"-Garantie erhalten.

**Implementierung:**
- Type-Token `FUNCREF`. AST braucht keinen neuen Node -- bare Identifier
  in Expression-Position wird kontextabhaengig aufgeloest.
- Tree-Walker: `_eval_Identifier` liefert `_FuncRef(name)` wenn der Name
  keine Variable ist und in `self.functions` existiert. `_eval_Call`
  dispatched FuncRef-callees direkt.
- Compiler: `_global_vars`-Set wird vor Phase 5 gefuellt (alle Top-Level
  DIM/CONST/MultiDim). `_expr_Identifier` und `_expr_Call` checken Locals,
  Felder, Globals zuerst -- nur wenn keiner trifft, fallen sie auf
  Function-Lookup zurueck (Tree-Walker-konsistente Vorrang-Reihenfolge).
  Eine User-Variable mit gleichem Namen wie eine Function verschattet
  diese.
- Bytecode: `LOAD_FUNCREF` (53) und `CALL_VALUE` (54). LOAD_FUNCREF nimmt
  den Function-Namen aus dem const-Pool und pusht eine `_FuncRef`-Instanz.
  CALL_VALUE pop n args, pop callee (FuncRef), dispatch via `_exec`.
- VM-Pfade: gleicher Flow. `_FuncRef` aus `gamebasic.interpreter` importiert.

**Einschraenkung Reihenfolge:** Im VM-Pfad wird `_global_vars` static aus
allen Top-Level-Statements gefuellt. Wer eine Function `foo` erst aufruft
und DANACH eine Variable `foo` deklariert, kriegt im VM einen
"Variable nicht deklariert"-Fehler bei der Function-Verwendung -- der
Tree-Walker ist hier dynamischer. Praxisrelevant ist das selten;
empfohlen: User-Variablen oben deklarieren oder anders benennen.

**Beispiel:** [examples/57_funcref.gb](examples/57_funcref.gb).

## Static Class Members

`STATIC CONST` innerhalb einer Klasse erzeugt klassen-bezogene Konstanten,
zugreifbar via `<ClassName>.<MEMBER>` -- analog zum ENUM-Pattern.

```basic
CLASS Player
    STATIC CONST MAX_HP AS INTEGER = 100
    STATIC CONST DEFAULT_NAME AS STRING = "Hero"
    DIM hp AS INTEGER
    SUB Init()
        Self.hp = Player.MAX_HP        ' Static aus Methode
    END SUB
END CLASS

PRINT Player.MAX_HP                     ' 100
```

**Werte muessen Compile-Zeit-Literale sein:** Number, String, Bool oder
negierte Number. Keine Ausdruecke -- gleiche Strenge wie ENUM-Member,
damit alle drei Pfade konsistent sind.

**Implementierung:** Klassen-Statics werden zur Klassen-Hoisting-Phase als
`_ClassStaticNamespace` gebaut und unter dem Klassen-Namen als globale
CONST registriert. MemberAccess auf das Namespace-Objekt liefert den
Member-Wert -- LOAD_MEMBER in beiden VMs erkennt den Typ explizit.

`_infer_type` bekommt `"class_static"` als neuen Type-String. `Self.hp =
Player.MAX_HP` funktioniert, weil `Player` als CONST-Variable im globalen
Scope existiert (mit dem Namespace als Wert), nicht als Klassen-Konstruktor
-- der NEW-Pfad geht ueber `self.classes[name]`, nicht ueber Identifier-
Lookup.

**Einschraenkungen:**
- Klassen mit Statics duerfen nicht den gleichen Namen wie eine andere
  globale Variable haben. (Klassen ohne Statics sind unbeeinflusst.)
- Static-Werte sind immutable nach Compile-Zeit -- es gibt kein
  `Player.MAX_HP = 200`. Wer Mutable-Class-State will, schreibt eine globale
  Variable mit "DIM ... AS Klasse".

**Beispiel:** [examples/56_static.gb](examples/56_static.gb).

## Properties (PROPERTY GET/SET)

Klassen koennen Property-Accessors deklarieren -- Member-Read und -Write
laufen dann durch User-Code statt direkt aufs Feld:

```basic
CLASS Player
    DIM _hp AS INTEGER

    PROPERTY GET hp() AS INTEGER
        RETURN Self._hp
    END PROPERTY

    PROPERTY SET hp(value AS INTEGER)
        IF value < 0 THEN value = 0
        IF value > 100 THEN value = 100
        Self._hp = value
    END PROPERTY
END CLASS

DIM p AS Player
p = NEW Player()
p.hp = 200      ' Setter laeuft -> clamped zu 100
PRINT p.hp      ' Getter laeuft -> 100
```

**Implementation:** Properties werden intern als Methoden mit den Namen
`__get_<name>` und `__set_<name>` registriert. Die Klasse merkt sich die
Property-Namen in einem `set` (`_ClassInfo.properties` /
`VMClassInfo.properties`). Im MemberAccess- und MemberAssign-Pfad wird
zuerst gegen das Property-Set geprueft, bei Treffer dispatcht der Code
zur Internal-Methode.

**Read-only / Write-only:** Wenn nur GET deklariert ist, wirft `obj.x = v`
einen Fehler. Wenn nur SET, wirft `PRINT obj.x`.

**Inheritance:** Properties werden automatisch vererbt (gleicher MRO-
Lookup wie Methoden).

**`GET`/`SET` sind keine Keywords:** Sie sind kontext-abhaengig nach
`PROPERTY`. So bleiben User-Methoden wie `FUNCTION Get() AS T` unbeeinflusst.

**Beispiel:** [examples/63_props_comp.gb](examples/63_props_comp.gb).

## List-Comprehensions

`[expr FOR var IN container]` und `[expr FOR var IN container WHERE filter]`.
Liefert ein TUPLE der transformierten Werte:

```basic
DIM evens AS TUPLE
evens = [n FOR n IN nums WHERE n MOD 2 = 0]

' Auch mit Method-Calls und Properties
DIM names AS TUPLE
names = [it.name FOR it IN cart WHERE it.price > 5]
```

**Iterable:** STRING (chars), TUPLE, 1D-ARRAY, MAP (Keys).

**Implementation:**
- AST `ListComp(var, iterable, filter, transform)`. Im Parser an LBRACKET
  in Primary-Position.
- Tree-Walker iteriert direkt ueber Python-Iterable.
- Compiler nutzt einen Marker-Singleton (`bytecode.COMP_MARKER`) und einen
  index-basierten Loop. Vor dem Loop wird der iterable durch das Built-in
  `__COMP_ITER` in ein TUPLE umgewandelt -- so funktioniert der gleiche
  LEN+Index-Mechanismus fuer alle Container-Typen.
- Neuer Op `BUILD_TUPLE_DYN=57`: sammelt alle Werte oberhalb des Markers
  zu einem Tupel.
- Iter-Variable wird als anonymer Local-Slot reserviert -- ueberlappt
  nicht mit existierenden globalen Variablen gleichen Namens.

**Bonus:** Strings haben jetzt auch normalen Index-Access (`s[0]`) -- nicht
nur Slicing. Das war fuer die Comprehension noetig und ist eine sinnvolle
generelle Erweiterung.

**Beispiel:** [examples/63_props_comp.gb](examples/63_props_comp.gb).

## SELECT CASE mit Guards

`CASE ... WHERE expr` -- der naechste Case wird probiert, wenn entweder das
Match-Pattern nicht trifft ODER der Guard-Ausdruck falsy ist:

```basic
SELECT CASE hp
    CASE IS <= 0
        ' tot
    CASE IS <= 30 WHERE has_potion
        ' low aber Trank verfuegbar -> heilen
    CASE IS <= 30
        ' low ohne Trank -> fliehen
    CASE ELSE
        ' OK
END SELECT
```

Guard-Expressions koennen auf normale Variablen zugreifen, inkl. der
Subject-Variable. Klassische Use-Cases:
- Permission-Checks: `CASE "delete" WHERE user IN ("admin", "moderator")`
- Numerische Conditions: `CASE 1 TO 100 WHERE n MOD 2 = 0`
- State-Combinationen: `CASE "save" WHERE dirty`

**Implementierung:** Compile-Zeit-Erweiterung von `_stmt_Select`. Nach dem
Match-Erfolg wird die Guard-Expression evaluiert (Subject bleibt am Stack
fuer den Body), und ein zusaetzlicher `JUMP_IF_FALSE` springt zum
naechsten Case wenn der Guard falsch ist. Kein neuer Bytecode -- die
existierenden Ops reichen.

**Kompatibilitaet:** Existierende SELECT-Statements ohne `WHERE` laufen
unveraendert -- der Parser erzeugt `(matches, guard=None, block)`-Tupel,
sowohl 2-Tuple- als auch 3-Tuple-Cases werden in den Pfaden akzeptiert
(Backward-Compat).

## IN-Operator

`x IN container` testet Mitgliedschaft auf String, Tupel, Array oder Map:

```basic
IF "World" IN "Hello World" THEN ...      ' Substring
IF 5 IN (1, 5, 9) THEN ...                ' Tupel
IF "name" IN m THEN ...                   ' Map-Key
IF 20 IN nums THEN ...                    ' Array-Element
```

Maps haben STRING-Keys -- `5 IN map` wirft TypeMismatch. Negation klassisch
mit `NOT (x IN c)`. Praezedenz wie `=`/`<>` (Comparison-Ebene), Bytecode:
neuer Op `IN_OP=56`.

## Variadic-Functions

`...args` als letzter Parameter sammelt alle restlichen Positional-Args
in ein TUPLE:

```basic
SUB log(level AS STRING, ...rest)
    DIM msg AS STRING
    msg = "[" + level + "]"
    DIM j AS INTEGER
    FOR j = 0 TO rest.length() - 1
        msg = msg + " " + STR$(rest[j])
    NEXT
    PRINT msg
END SUB

log("WARN")                          ' rest = ()
log("INFO", "App", "started")        ' rest = ("App", "started")
```

**Einschraenkungen:**
- Variadic muss letzter Parameter sein (Parser-Fehler sonst).
- Variadic-Slot kann nicht mit Named-Arg uebergeben werden (semantisch
  unklar -- werfen).
- TUPLE als Type stuetzt `length()`, `len()` und `[i]`-Index-Access.
- Keine Default-Werte fuer Variadic (immer `()` wenn leer).

**Implementation:** AST `Param.is_variadic`, `CompiledFunction.is_variadic`.
Im Tree-Walker `_resolve_args` und in den VMs `_exec` werden ueberzaehlige
Positional-Args in ein Tupel gesammelt; Default-Resolution wird fuer
variadic-Funktionen umgangen.

**Beispiel:** [examples/62_qol_sprint.gb](examples/62_qol_sprint.gb).

## Method-Syntax auf Built-in-Containern

Strings, Arrays und Maps haben Convenience-Methoden, die zu BUILTINs
delegieren:

```basic
PRINT "hello".upper()           ' "HELLO" (= UPPER$("hello"))
PRINT "  hi  ".trim().upper()   ' Method-Chain: "HI"
PRINT a.length()                ' = LEN(a)
m.put("k", 1)                   ' = MAPPUT(m, "k", 1)
PRINT m.has("k")                ' = MAPHAS(m, "k")
```

**Dispatch-Tabelle:** `interpreter.CONTAINER_METHODS` mappt
`(target_kind, method_name)` zu BUILTIN-Namen. Tree-Walker und beide
VMs konsumieren dieselbe Tabelle (Single-Source-of-Truth).

**Verfuegbare Methoden:**
- String: `upper`, `lower`, `length`/`len`, `trim`, `left`, `right`,
  `mid`, `indexof`, `replace`, `split`, `padl`, `padr`.
- Array: `length`/`len`.
- Map: `put`, `get`, `getor`, `has`, `keys`, `size`/`length`/`len`,
  `remove`, `clear`.

Wer einen weiteren Method-Alias will, fuegt einen Eintrag in
`CONTAINER_METHODS` hinzu -- kein VM/Bytecode-Change noetig.

**User-Klassen-Methoden gewinnen:** Nur wenn der Receiver kein User-
Instanz ist (sondern String/Array/Map), wird die Container-Methoden-
Tabelle konsultiert. So kollidiert `Foo.upper()` mit User-Klasse `Foo`
nicht.

**Beispiel:** [examples/61_method_syntax.gb](examples/61_method_syntax.gb).

## Slicing

`s[a:b]`, `s[a:]`, `s[:b]`, `s[:]` -- liefert Substring (String) oder
neues 1D-Array (Array, echte Kopie). Negative Indices und Step werden
NICHT unterstuetzt -- konsistent mit der existierenden strikten Index-
Validierung. Out-of-bounds (`s[0:1000]` bei `len(s)=11`) wird auf den
gueltigen Bereich geclampt.

```basic
PRINT "Hello World"[6:11]     ' "World"
DIM b AS ARRAY OF INTEGER
b = a[1:4]                    ' echte Kopie, kein Alias
```

**Multi-Dim-Slicing** (`g[0:2, 1:3]`) wird nicht unterstuetzt --
NumPy-Semantik. Slice-Assign (`s[a:b] = ...`) wird ebenfalls bewusst
abgelehnt (Laenge-Match unklar).

**Implementierung:** AST `SliceAccess(target, lo, hi)`. Parser
disambiguiert in `_index_or_slice` anhand des Top-Level-`:`. Bytecode
`SLICE` mit Flag-Tupel `(has_lo, has_hi)` -- die VM popt entsprechend
viele Werte.

**Beispiel:** [examples/60_slicing.gb](examples/60_slicing.gb).

## ELIF / String-Multiplikation

Kleine Quality-of-Life-Erweiterungen:

- **`ELIF`** ist Alias fuer `ELSEIF` -- gleiches Token, kein AST-Change.
- **`"-" * 40`** liefert einen String aus 40 Bindestrichen. Auch
  `40 * "-"`. Negative Counts liefern leeren String. Strikt INTEGER --
  kein Float, kein Bool. Im VM-Pfad wurde gleichzeitig `OP.MUL` strenger
  gemacht (war vorher zu lax bei Bool-Operanden).

## WITH ... END WITH

Klassisches BASIC-Konstrukt fuer kompakte Member-Bursts:

```basic
WITH player
    .x = 100
    .y = 50
    .hp = 100
    .name = "Alice"
END WITH
```

**Semantik:**
- WITH-Ziel wird **einmal** evaluiert (wichtig bei Side-Effects).
- Innerhalb des Body ist `.member` Shortcut fuer `<target>.member`.
- Auch in Read-Position: `len = SQR(.x * .x + .y * .y)`.
- Compound-Assigns funktionieren: `.points += 5`.
- Verschachtelte WITHs erlaubt; innerstes gewinnt (Stack-Semantik).

**Implementierung:** Reines Compile-Zeit-Desugar ohne neuen Bytecode.
- Parser haelt einen `_with_stack: list[str]` mit Compiler-generierten
  Variablen-Namen (`__with_<n>`).
- `_with_stmt` parst, generiert frischen Namen, pusht auf Stack, parst Body,
  popt, gibt `With(var_name, target, body)` zurueck.
- Im `_primary` und `_statement_inner`: wenn aktueller Stack nicht leer und
  Token = DOT, desugar zu `MemberAccess(Identifier(top), name)`.
- Tree-Walker: `_exec_With` setzt `env.vars[var_name] = {"type":"any","value":val}`,
  fuehrt body aus, entfernt den Slot wieder.
- Compiler: `_stmt_With` allokiert anonymen Local-Slot (`_alloc_anon_slot`),
  bindet `var_name -> slot` in `local_slots` waehrend Body-Compile, entfernt
  ihn danach. So wird `Identifier(__with_<n>)` zu `LOAD_LOCAL slot`.
- "any"-Type-Coerce ist passthrough (Tree-Walker `interpreter.py:_coerce`,
  VM `vm.py:_coerce_any`, Cython `vm_native.pyx`).

**Beispiel:** [examples/55_with.gb](examples/55_with.gb).

## Tupel + Destructuring

Mehrfach-Rueckgabewerte ohne BYREF-Krampf.

```basic
FUNCTION minmax(a AS INTEGER, b AS INTEGER) AS TUPLE
    IF a < b THEN RETURN (a, b)
    RETURN (b, a)
END FUNCTION

DIM lo AS INTEGER
DIM hi AS INTEGER
(lo, hi) = minmax(7, 3)        ' Destructuring
```

**Tupel-Literal:** `(a, b, c)` -- mindestens 2 Elemente. Eine einzelne
geklammerte Expression `(expr)` bleibt Klammer-Gruppierung. `(1,)`-Single-
Tupel wie in Python wird NICHT unterstuetzt (kein Use-Case).

**Destructuring-Assignment:** `(t1, t2, ..., tn) = expr`. Die `expr` muss zur
Laufzeit ein Tupel mit exakt n Elementen ergeben -- sonst `GBRuntimeError`.
Targets duerfen Identifier, MemberAccess oder IndexAccess sein
(`(p.x, p.y) = polar_to_cart(r, a)` funktioniert).

**Type-Annotation:** `DIM t AS TUPLE` -- generisch, akzeptiert beliebige
Tupel. Keine Element-Type-Annotation an der Sprachebene; wer striktere
Garantien braucht, prueft selbst beim Destructuring.

**Implementierung:**
- AST: `TupleLit(elements)`, `TupleAssign(targets, value)`.
- Bytecode: `BUILD_TUPLE n` und `UNPACK_TUPLE n` (Ops 68, 69).
- Im Compiler-`_stmt_TupleAssign` werden Member/Index-Targets ueber einen
  anonymen Local-Slot zwischengepuffert (per `_alloc_anon_slot`), weil
  STORE_MEMBER/STORE_INDEX die Receiver-Position vor dem Wert braucht.
- Wertsemantik = Python-`tuple` (immutable). `_fmt` erzeugt `(a, b, c)`
  fuer PRINT.
- Cython-VM muss nach Aenderungen am Tupel-Pfad neu kompiliert werden.

**Praktisch:** Beispiel [examples/54_tuple.gb](examples/54_tuple.gb) zeigt
Min/Max, Vektor-Reflexion, Polar-Konvertierung, Player-State als Tupel.

## Bitwise-Operatoren

Strikt INTEGER (kein FLOAT, kein BOOL). Sechs Operatoren als Keywords:

```basic
a BAND b      ' bit-and
a BOR  b      ' bit-or
a BXOR b      ' bit-xor
a SHL  n      ' shift-left  (n >= 0)
a SHR  n      ' shift-right (n >= 0)
BNOT a        ' unaer, bitweises NICHT (= ~a in Python)
```

**Praezedenz:** Alle binaeren Bitwise auf EINER Ebene, links-assoziativ.
Position zwischen `Vergleich` und `+,-`. Heisst:
- `a BAND b = c` parst als `(a BAND b) = c`.
- `a + b BAND c` parst als `(a + b) BAND c`.
- `1 BOR 2 BAND 3` parst als `((1 BOR 2) BAND 3) = 3` — wer C-Stil-Praezedenz
  will, klammert (`1 BOR (2 BAND 3)`).

`BNOT` liegt im `_unary` neben `-` und unaerem `+` — d.h. tighter binding als
`*`/`/`. `BNOT a BAND b` ist `(BNOT a) BAND b`.

**Type-Strictness:** Bool wird abgelehnt (gleiche Linie wie `_check_num`).
Negativer Shift-Count wirft `GBRuntimeError` statt nichtssagendem Python-Fehler.

**Keine alten Built-ins mehr:** Frueher gab's `BITAND/BITOR/BITXOR/BITNOT/SHL/SHR`
als Funktions-Built-ins. Mit den Operatoren ueberfluessig — entfernt.
`BITAND(a, b)` -> `a BAND b`. Im Tree-Walker (interpreter.py:1009-1027) und
in beiden VMs implementiert (Ops 62-67 in bytecode.py).

## SELECT CASE

Mehrweg-Verzweigung statt verschachtelter `IF/ELSEIF`-Ketten. Drei Match-Formen
pro CASE, beliebig kombinierbar:

```basic
SELECT CASE x
    CASE 1                       ' exakter Wert
        ...
    CASE 2, 3, 4                 ' Liste von Werten
        ...
    CASE 10 TO 20                ' Bereich (inklusiv)
        ...
    CASE IS > 100                ' Vergleich (=, <>, <, >, <=, >=)
        ...
    CASE 1, 5 TO 8, IS = 13      ' alle Formen mischbar
        ...
    CASE ELSE                    ' Fallback (optional, max. einmal, muss letzter sein)
        ...
END SELECT
```

**Garantie:** Subject-Ausdruck wird **einmal** evaluiert (auch bei
Side-Effects in Function-Calls). Der erste passende Case gewinnt.

**Implementierung (lehrreich):** Im Parser zu `Select(subject, cases, else_block)`,
Cases sind `(list[CaseMatch], list[Stmt])`-Tupel mit `kind ∈ {"value", "range"}`.
Im Compiler **kein neuer Bytecode** — der Subject bleibt während aller Match-Tests
auf dem Stack (per `DUP` geklont), Range-Tests werden zu `subj >= lo` (mit
`JUMP_IF_FALSE`) gefolgt von `subj <= hi` (mit `JUMP_IF_TRUE` zum Block) verkettet.
Cython-VM hat es ohne Neukompilation übernommen.

## ENUM

Typsichere Konstanten mit Namespace-Zugriff (`State.PLAYING`):

```basic
ENUM State = MENU, PLAYING, PAUSED       ' compact
ENUM Permission                          ' block
    NONE = 0
    READ = 1
    WRITE = 2
END ENUM
```

Auto-Nummerierung (0, 1, 2, …) oder explizit. Mixed: nach explicit zählt's
weiter (`A, B = 5, C` → A=0, B=5, C=6). Member-Namen dürfen Keywords sein
(`READ`, `FILE`, `DATA`, `NONE`) — der qualifizierte Zugriff ist eindeutig.

**Implementierung:** `EnumDecl(name, members)` AST-Node. Im Tree-Walker
und Compiler zur Compile-Zeit zu einem `_EnumNamespace`-Objekt aufgelöst,
als globale CONST abgelegt. `MemberAccess` erkennt `_EnumNamespace` (in
`interpreter.py`, `vm.py`, `vm_native.pyx`) und liefert den Member-Wert.
Member-Werte müssen Compile-Time-Integer-Literale sein (auch im
Tree-Walker — Konsistenz). `DIM x AS State` löst der Parser zu `INTEGER`
auf, indem er bekannte Enum-Namen in `self._enum_names` trackt.

Keywords als Member-Namen: Parser-Helfer `_consume_member_name` und
DOT-Zugriff in `_postfix` akzeptieren jedes Token mit string-`value`,
nicht nur `IDENT`.

## Named Arguments

`func(name: "Anna", age: 30)` mit Defaults. Lexer-Token `COLON`,
AST-Node `NamedArg(name, value)` als Element von `Call.args`.

**Tree-Walker** (`interpreter.py`): `_resolve_args(decl, raw_args, fn_name)`
mappt positional + named auf Param-Reihenfolge, liefert eine voll-lange
Liste mit `_DEFAULT_SENTINEL` für Slots, die der User nicht belegt hat.
`_invoke` evaluiert Sentinels via Default-Ausdruck im local_env. Funktioniert
mit BYREF (Sentinel-Slots können kein BYREF sein) und mit Param-
referenzierenden Defaults.

**Compiler** (`compiler.py`): `_resolve_named_args(fn, raw_args, fn_name)`
löst zur Compile-Zeit auf — Slots ohne Wert kriegen den evaluierten
Default-Literalwert direkt als `LOAD_CONST` emittiert. `param_names` ist
ein neues Feld auf `CompiledFunction` (Compile-Zeit-Info, VM nutzt es nicht).
Auch `NEW Klasse(...)` wird so resolved (Init-Methode lookup zur Compile-Zeit).

**Einschränkungen im VM-Pfad:**
- Built-ins haben keine deklarierten Param-Namen → werfen.
- Method-Calls (`obj.method(name: ...)`): Klasse erst zur Laufzeit
  bekannt → Compiler wirft. Tree-Walker kann's.

## Self + implizite Methoden-Aufrufe

Innerhalb einer Klassen-Methode:

```basic
CLASS Wave
    SUB Init()
        StartCurrent()         ' impliziter Methoden-Aufruf
    END SUB
    SUB StartCurrent()
        ...
    END SUB
END CLASS
```

`Self` als Identifier liefert die aktuelle Instanz; bare `MethodName(...)`
ohne `Self.`-Präfix dispatcht zuerst gegen die Methoden der eigenen Klasse
(und Superklassen), erst dann gegen globale Funktionen.

**Tree-Walker:** `Interpreter._method_stack: list[(_Instance, _ClassInfo)]`
wird in `_invoke` gepusht/gepoppt. `_eval_Identifier` erkennt `"self"` und
liefert die aktuelle Instanz. `_eval_Call` mit Identifier-callee prüft
zuerst `_resolve_method(current_cls, name)`.

**Compiler:** `_load_var` emittiert für `name == "self"` (innerhalb
`current_class != None`) den neuen Op `LOAD_SELF`. `_expr_Call` bei
Identifier-callee resolved Methoden via `_resolve_method_compile` und
emittiert `LOAD_SELF` + Args + `CALL_METHOD`. Damit Methode A in derselben
Klasse die Methode B sehen kann, registriert Phase 4a vor dem Body-
Kompilieren leere Stub-`CompiledFunction`s in `ci.methods`.

**Bytecode-Op:** `LOAD_SELF = 88` — push `self_obj` (im VM-`_exec` als
Parameter). Implementiert in `vm.py` und `vm_native.pyx`. Wer Self-Code
schreibt, muss daher `vm_native.pyx` neu kompilieren.

## Statement-Trenner Doppelpunkt

`x = 1 : y = 2` — Doppelpunkt trennt Statements wie Newline.
`Parser._consume_terminator` und `_skip_newlines` akzeptieren beide Token.
Funktioniert mit Named-Args nicht in Konflikt, weil dort der `IDENT COLON`-
Lookahead in `_call_arg` läuft (innerhalb von `(` ... `)`), wo der
Terminator gar nicht erst geprüft wird.

## f-Strings (String-Interpolation)

`f"text {expr} text..."` -- der Lexer expandiert das zur Token-Sequenz
`("text" + STR$(expr) + "text" + ...)`. Damit funktionieren f-Strings ohne
einen einzigen Eingriff in Parser, Interpreter, Compiler oder VMs:

```basic
DIM name AS STRING
DIM hp AS INTEGER
name = "Anna"
hp = 75
PRINT f"{name} hat {hp} HP"          ' "Anna hat 75 HP"
PRINT f"max: {MAX(a, b)}"            ' Methodenaufrufe in {} sind ok
PRINT f"literal {{nicht interpoliert}}, aber {hp}"
```

**Eigenschaften:**
- `{{` und `}}` sind Escapes fuer literale geschweifte Klammern.
- Verschachtelte f-Strings sind nicht erlaubt (`f"{f"..."}"`).
- Ausdruecke duerfen `(`, `)`, Methoden-Aufrufe, MemberAccess etc.
  enthalten -- der Tokenizer matched balanced braces.
- Ohne `f`-Prefix bleibt `"hi {name}"` ein wortlich enthaltener String mit
  geschweiften Klammern -- Opt-in.
- Editor-Highlighter erkennt f-Strings als Block und faerbt den ganzen
  Range einheitlich als String (siehe `editor_qt/highlighter.py`).

**Format-Specs** (`{expr:spec}`): ein Top-Level-`:` im Platzhalter trennt
einen printf-Spec ab -- der Lexer emittiert dann `FORMAT$(expr, "%spec")`
statt `STR$(expr)`:
```basic
PRINT f"FPS {fps:.1f}  Score {score:05d}"   ' "FPS 59.7  Score 00042"
```
Ein `:` innerhalb von `()`/`[]`/`{}` (z.B. Slice `s[0:3]`) oder String-Literalen
zaehlt NICHT als Spec-Trenner -- `_split_fstring_spec` in `lexer.py` trackt
Klammer-/String-Tiefe. Rein Lexer-basiert, daher in allen drei Pfaden gleich.

**Implementierung:** `lexer._scan_fstring` wird beim ersten `f"`-Lookahead
aufgerufen ([lexer.py:114-115](gamebasic/lexer.py:114)) und emittiert die
expandierte Token-Sequenz selbst -- mit Sub-Lexer fuer den Ausdrucks-Teil.

**Beispiel:** [examples/69_fstring.gb](examples/69_fstring.gb).

## Kontrollfluss: BREAK / CONTINUE / REPEAT / TRY

Diese sind implementiert (waren in aelteren Doku-Staenden nicht aufgefuehrt):

- **`BREAK`** / **`CONTINUE`** in `FOR`, `FOR EACH`, `WHILE`, `REPEAT`. Auch in
  Single-Line-IF (`IF v = 40 THEN BREAK`). Tree-Walker via `_BreakSignal`/
  `_ContinueSignal`-Exceptions; Compiler via `break_patches`/`continue_patches`-
  Stack (mit `try_depth` fuer korrektes `TRY_END`-Unwinding).
- **`REPEAT ... UNTIL cond`** -- Post-Test-Loop (laeuft mindestens einmal).
- **`WHILE cond ... WEND`** -- Pre-Test-Loop.
- **`TRY ... CATCH [e] ... END TRY`** + **`THROW value`**. Die Catch-Variable
  ist optional und faengt den (String-)Wert. Kein typed Catch -- `THROW` wirft
  beliebige Werte; Module-/Runtime-Fehler kommen als `GBRuntimeError`-Message.

```basic
FOR EACH e IN enemies
    IF e.dead THEN CONTINUE
    IF boss_killed THEN BREAK
    e.update()
NEXT

TRY
    riskante_op()
CATCH msg
    PRINT "Fehler: " + msg
END TRY
```

## FOR EACH

`FOR EACH var IN container ... NEXT` -- iteriert ueber STRING (Zeichen),
TUPLE, 1D-ARRAY oder MAP (Keys):

```basic
FOR EACH b IN bullets
    b.update()
NEXT
FOR EACH k IN scores        ' Map -> Keys
    PRINT k, MAPGET(scores, k)
NEXT
```

`each` ist **kontextuell**, kein Keyword: `FOR each = 1 TO 3` mit einer
Variable namens „each" bleibt ein regulaerer FOR (Disambiguierung im Parser:
FOR EACH nur wenn nach `each` ein IDENT statt `=` folgt).

**Implementierung:** AST-Node `ForEach(var, iterable, body)`. Tree-Walker
iteriert direkt (`_iter_for_comp`, wie Comprehensions). Compiler desugart zu
einem Vorwaerts-Index-Loop ueber `__comp_iter(iterable)` (-> TUPLE) +
`LOAD_INDEX` und nutzt den vorhandenen break/continue-Patch-Stack -- **kein
neuer Bytecode**, beide VMs unveraendert. Loop-Var wird als `"any"` deklariert.

## IIF (Inline-Ternary)

`IIF(cond, then, else)` -- echter **lazy** Ternary, nur EIN Zweig wird
ausgewertet (Short-Circuit):

```basic
dx = IIF(moving_left, -speed, speed)
PRINT IIF(x <> 0, 100 \ x, -1)    ' bei x=0: kein Division-Crash
```

`iif` ohne `(` bleibt ein normaler Bezeichner (kontextuell im Parser).
**Implementierung:** AST-Node `TernaryExpr(cond, then, else)`. Compiler
desugart zu `JUMP_IF_FALSE` (poppt die Bedingung) -- **kein neuer Bytecode**.
Tree-Walker: `_eval_TernaryExpr` evaluiert nur den gewaehlten Zweig.

## Array- & Map-Helfer

Reine Builtins (alle drei Pfade automatisch), auch als Container-Methoden
(`CONTAINER_METHODS`-Tabelle):

- `SORT(arr)` / `arr.sort()` -- 1D-Array IN PLACE aufsteigend (INTEGER/FLOAT/STRING).
- `REVERSE(arr)` / `arr.reverse()` -- 1D-Array IN PLACE umkehren.
- `ARRAY_INDEXOF(arr, v)` / `arr.indexof(v)` -- erster Index oder -1.
- `MAPVALUES(m)` / `m.values()` -- ARRAY aller Werte (Einfuege-Reihenfolge).
- `MAPITEMS(m)` / `m.items()` -- ARRAY von `(key, value)`-TUPELn (gut mit
  `FOR EACH` + Destructuring).

## Module-Imports mit Alias

`IMPORT "modul" AS alias` -- aliased die Built-ins / externen Typen unter
einem ersetzten Praefix:

```basic
IMPORT "json" AS j
DIM h AS J_HANDLE
h = J_PARSE("[1, 2, 3]")
PRINT J_GET_INT(h, "0")     ' 1
```

**Aliasing-Strategie:** GameBasic-Module teilen einen flachen Built-in-
Namespace -- es gibt kein echtes Namespacing. Der Alias dupliziert alle
Built-ins / Typen, deren Name mit `<modul>_` anfaengt, unter `<alias>_`.
Single-word-Namen (z.B. der externe Typ `vec2`) werden komplett ersetzt
(`vec2` -> `v` bei `IMPORT "vec2" AS v`).

**Konvention-basiert:** funktioniert fuer Module, deren Built-in-Praefix
dem Modul-Namen entspricht (json, db, tween, vec2, sprite, ecs, ...).
Module mit abweichendem Praefix (z.B. `imgfx` registriert `IMAGE_*`,
nicht `IMGFX_*`) sind nicht via Alias adressierbar -- der Praefix-Match
liefert dann leer.

**Idempotent + sticky:** zweimal mit unterschiedlichen Aliasen ist OK
(beide werden zusaetzlich registriert), aber doppelt mit demselben Alias
ist no-op.

## Dict/Set-Comprehensions

`{key: val FOR var IN iterable [WHERE filter]}` -- Dict-Comprehension,
liefert eine MAP. `{expr FOR var IN iterable [WHERE filter]}` -- Set-
Comprehension, liefert ein TUPLE mit deduplizierten Werten in der
Reihenfolge des ersten Auftretens.

```basic
' Dict-Comp: Quadrate als Map
DIM squares AS MAP OF INTEGER
squares = {STR$(x) + "sq": x * x FOR x IN (1, 2, 3, 4)}
PRINT MAPGET(squares, "3sq")     ' 9

' Set-Comp: eindeutige Mod-Werte
DIM distinct AS TUPLE
distinct = {x MOD 3 FOR x IN (0, 1, 2, 3, 4, 5, 6, 7, 8)}
PRINT distinct                    ' (0, 1, 2)
```

**Dict-Keys MUESSEN STRING sein** (GameBasic-MAP-Konvention). Der MAP-
Wert-Typ wird beim ersten Eintrag inferiert. Set-Comp ist eine pragmatische
Naeherung -- GameBasic hat keinen echten SET-Typ; das deduplizierte TUPLE
ist die nahe liegende Alternative.

**Implementierung:** Lexer kennt jetzt `LBRACE`/`RBRACE` (nur fuer Comp-
Position). Parser disambiguiert per `:`-Lookahead (Dict) oder direkt
`FOR` (Set). Compiler nutzt das existierende `BUILD_TUPLE_DYN`-Pattern
plus zwei interne Built-ins (`__SET_DEDUP`, `__DICT_FROM_PAIRS`) als
Final-Schritt -- keine neuen Bytecode-Ops noetig, Cython-VM ohne
Aenderung kompatibel.

**Beispiel:** [examples/71_dictcomp.gb](examples/71_dictcomp.gb).

## Operator-Overloading auf User-Klassen

Klassen koennen Operatoren ueberladen, indem sie `OPERATOR <op>`-Methoden
definieren -- analog zu `SUB`/`FUNCTION` im Class-Body:

```basic
CLASS Money
    DIM cents AS INTEGER

    OPERATOR + (other AS Money) AS Money
        DIM r AS Money
        r = NEW Money()
        r.cents = Self.cents + other.cents
        RETURN r
    END OPERATOR

    OPERATOR = (other AS Money) AS BOOLEAN
        RETURN Self.cents = other.cents
    END OPERATOR
END CLASS

DIM a AS Money
a = NEW Money()
a.cents = 100
DIM b AS Money
b = NEW Money()
b.cents = 200
PRINT (a + b).cents     ' 300
PRINT a = b             ' FALSE
```

**Erlaubte Operatoren:** `+`, `-`, `*`, `/`, `MOD`, `=`, `<>`, `<`, `>`,
`<=`, `>=`. Genau ein Parameter (`other`), Rueckgabetyp ist Pflicht.
BYREF und variadic sind nicht erlaubt.

**Implementierung:** Parser konvertiert `OPERATOR + (...)` zu einer Methode
mit reserviertem Namen `__op_add__` (siehe `parser._OPERATOR_NAMES`).
Tree-Walker (`_eval_BinaryOp`) und beide VMs (`OP.ADD/SUB/MUL/DIV/MOD/EQ/...`)
konsultieren via `_user_op(...)` die Methode auf LHS, dann auf RHS
(Reverse-Dispatch). Fallback ist der Standard-Pfad.

**Vererbung:** Operator-Methoden werden ueber die normale MRO gesucht --
Child-Klassen erben sie automatisch.

**Einschraenkungen:**
- Keine reflektierten Operatoren a la Python (`__radd__`). Wer `5 + money`
  unterstuetzen will, definiert `OPERATOR + (other AS INTEGER) AS Money`
  auf `Money` -- der Reverse-Dispatch greift dann.
- Kein Method-Overloading: pro Operator gibt's genau eine Methode.
  `Money + Money` und `Money + INTEGER` koennen nicht gleichzeitig
  definiert werden (man muesste type-switchen im Body).
- Operatoren auf Modul-Typen (Vec2 etc.) gewinnen vor User-Klassen --
  die Modul-Registry wird zuerst konsultiert.

**Beispiel:** [examples/70_operator.gb](examples/70_operator.gb).

## Operator-Registry

Modul-eigene Typen koennen arithmetische Operatoren (`+`, `-`, `*`, `/`)
ueberladen, ohne dass interpreter.py / vm.py / vm_native.pyx angefasst werden:

```python
# In gamebasic/modules/<name>.py
from . import register_operators

def _op_add(a, b):
    if isinstance(a, _MyType) and isinstance(b, _MyType):
        return _MyType(...)
    raise TypeMismatchError("...")

register_operators(_MyType, {"+": _op_add, "-": _op_sub, "*": _op_mul, "/": _op_div})
```

**Dispatch:** Vor dem Standard-Pfad ruft Tree-Walker (`_eval_BinaryOp`) und
beide VMs (`OP.ADD/SUB/MUL/DIV`) `modules.dispatch_binary_op(op, a, b)`.
Wenn `type(a)` oder `type(b)` registriert ist, dispatcht zur Handler-Tabelle;
sonst liefert die Registry `NO_OP_MATCH` und der Standard-Pfad uebernimmt.

**Konvention:** Bei asymmetrischen Operatoren (z.B. `Skalar * Vec2`) muss
der Handler beide Reihenfolgen selbst akzeptieren, weil die Registry nur
einen Handler-Eintrag pro Typ kennt -- siehe `vec2._op_mul` als Pattern.

**Equality:** `=` und `<>` sind nicht in der Registry -- die Standard-
Python-Equality (`__eq__`/`__ne__` auf der Klasse) reicht. Die VMs nutzen
`a == b` direkt.

Wer einen neuen Math-Typ wie `_Mat3x3`, `_Complex` oder `_Quat` einbauen
will, schreibt nur sein Modul -- keine Aenderung an Interpreter oder VMs.

## Asset-Cache + `LOAD_ASSETS`

`Graphics` haelt zwei Caches: `_image_cache` und `_sound_cache`. Sowohl
`LOADIMAGE` als auch `LOADSOUND` pruefen zuerst den Cache und cachen
das Ergebnis unter zwei Schluesseln: dem rohen Pfad UND dem
normalisierten Absolut-Pfad. So treffen verschiedene Pfad-Schreibweisen
(`"sprites/x.png"` vs. `"./sprites/x.png"` vs. absolut) denselben Eintrag.

`LOAD_ASSETS(manifest.json)` praefuellt den Cache aus einem JSON-Manifest:

```json
{
  "images": { "player": "sprites/player.png", "enemy": "sprites/enemy.png" },
  "sounds": [ "sfx/jump.wav", "music/level1.ogg" ]
}
```

Beide Sektionen sind optional, jede kann **Dict** (Alias → Pfad) oder
**Liste** (nur Pfade) sein. Bei Dict-Form ist `LOADIMAGE("player")`
ein Cache-Hit (Alias) **und** `LOADIMAGE("sprites/player.png")` auch
(Pfad-Hit unter Absolut-Pfad). Pfade im Manifest sind relativ zum
Manifest-Verzeichnis.

`LOAD_ASSETS` liefert die Anzahl geladener Assets. Idiomatisch in der
Init-Phase nach `SCREEN(...)` aufrufen, damit Bilder direkt
`convert_alpha`-optimiert werden. Beispiel: [examples/75_preloader.gb](examples/75_preloader.gb).

## Z-Layer-Rendering

Layer sind named Compose-Surfaces mit explizitem z-Wert. Alle
draw-Methoden zeichnen auf `Graphics._buffer`; `LAYER("name")` lenkt
`_buffer` auf die Layer-Surface um. `FLIP` composiert alle Layer in
z-Order auf den `_main_buffer`, blittet zum Screen, und cleart die
Layer (transparent) fuer den naechsten Frame.

```basic
LAYER_DEFINE("bg", 0)
LAYER_DEFINE("sprites", 10)
LAYER_DEFINE("ui", 100)

LAYER("bg");      CLS(SKY); DRAWIMAGE(parallax, 0, 0)
LAYER("sprites"); DRAWIMAGE(player, x, y)
LAYER("ui");      TEXT(10, 10, "Score: 100")
FLIP()   ' composiert in z-Order, cleart fuer naechsten Frame
```

**Builtins:**
- `LAYER_DEFINE(name, z)` — registrieren mit z; redefine aktualisiert z
- `LAYER(name)` — switchen (auto-Define mit auto-z wenn neu)
- `LAYER_END()` — zurueck zum Main-Buffer (optional, FLIP macht's auch)
- `LAYER_CLEAR(name)` — manuell leeren (selten gebraucht)

**Implementation** ([rust/gb_runtime/src/graphics.rs](rust/gb_runtime/src/graphics.rs)):
kein Surface-Compositing (das war das alte, entfernte Python-`graphics.py`
via pygame) — gbrt ist ein **Command-Recording-Modell**. Jede Layer ist ein
`Layer { z, cmds: Vec<Cmd> }`; Draw-Aufrufe haengen sofort einen `Cmd` an
die *aktive* Layer (`self.active`, per `LAYER(name)` umgeschaltet) an, statt
etwas zu rendern. `FLIP()` sortiert alle Layer-Indizes nach `z` aufsteigend
(niedrigstes z = hinten, hoechstes z = vorne) und spielt deren `cmds` in
dieser Reihenfolge in EINEM `begin_drawing`-Block ab (`render_scene`),
danach werden alle `cmds`-Vecs geleert (Immediate-Mode: ein Frame lang
gueltig) und `self.active`/`self.active_rt` auf den Main-Buffer
zurueckgesetzt.

**Backwards-Compat:** Code ohne `LAYER_*`-Calls hat `_layer_order = []`,
der Compose-Pfad in FLIP ist ein No-Op und `_buffer` zeigt direkt auf
`_main_buffer`. Existierende Programme laufen unveraendert.

**Beispiel:** [examples/76_layers_atlas.gb](examples/76_layers_atlas.gb).

## Sprite-Atlas + Batch-Draw

Sprite-Atlas: EIN grosses Image + Dict von `name -> (x, y, w, h)`-Rects.
Mehrere Sub-Sprites teilen sich eine Textur -- Game-Engine-Pattern fuer
Tilemaps, Bullet-Hell, Tile-Drawing. Nativ in gbrt; Tree-Walker
konsolen-only -> wirft "nur gbrt".

> **`BATCH_DRAW`/`BATCH_FLUSH` sind heute reine API-Kompatibilitaet, KEIN
> echtes Batching.** Der Name + die Kommentare unten stammen aus der alten
> Python-Engine (pygame-Surface-Blits, die sich tatsaechlich zu einem
> gebatchten Blit sammeln liessen). gbrts Immediate-Mode ist ein
> Command-Recording-Modell: jeder Draw-Aufruf haengt sofort einen `Cmd` an
> die aktive Layer, `FLIP()` sortiert die Layer nach z und spielt sie ab.
> In diesem Modell IST `BATCH_DRAW` identisch zu `ATLAS_DRAW` (derselbe
> Match-Arm in `vm.rs`), und `BATCH_FLUSH()` ist ein No-Op -- es gibt
> keinen separaten Batch-Queue-Zustand, der etwas zu flushen haette. Die
> "Auto-Flush"/"Zoom-Caveat"-Absaetze unten beschreiben daher ein
> Verhalten, das in gbrt nicht (mehr) existiert; sie sind als historische
> Notiz stehen gelassen, nicht als aktuelle Verhaltensdokumentation.

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/tiles_atlas.json")

' Einzel-Draw (Camera-aware):
ATLAS_DRAW(atlas, "tile_grass", 0, 0)

' Batch-Pattern (schneller bei vielen Sprites):
FOR i = 0 TO 99
    BATCH_DRAW(atlas, "tile_grass", i * 16, 0)
NEXT
BATCH_FLUSH()   ' EIN gebatchter Draw-Call fuer 100 Sprites
```

**Manifest-Format:**
```json
{
  "image": "tiles.png",
  "sprites": {
    "tile_grass": [0,  0, 16, 16],
    "player":     [16, 0, 24, 32]
  }
}
```
Rects sind `[x, y, w, h]`. Bild-Pfad relativ zum Manifest.

**Builtins:**
- `ATLAS_LOAD(json)` -> `SPRITE_ATLAS`
- `ATLAS_DRAW(atlas, name, x, y)` — einzeln, Camera-aware
- `BATCH_DRAW(atlas, name, x, y)` — an Batch-Queue anhaengen
- `BATCH_FLUSH()` — Queue jetzt rendern (gebatchter Draw-Call)

**Auto-Flush** an den richtigen Punkten:
- vor FLIP (sonst geht die Queue verloren)
- vor LAYER-Switch (damit Batch zum richtigen Target geht)
- vor ATLAS_DRAW (Direct-Call) -- bewahrt Reihenfolge

**Zoom-Caveat:** Bei `CAMERA_SET`-Zoom ≠ 1 faellt jeder `BATCH_DRAW`
auf einen Einzel-Draw zurueck (kein Batch-Vorteil, weil der Batch nicht
skaliert zeichnen kann). Translation ist OK.

**Externer Typ:** `SPRITE_ATLAS` ist in TYPE_DEFAULTS / _coerce in
allen drei Pfaden (Tree-Walker, vm.py, vm_native.pyx) registriert.
`DIM x AS SPRITE_ATLAS` funktioniert direkt ohne IMPORT.

## ECS Bulk-System-Ops

Klassische ECS-Performance-Falle: pro-Entity-Loop in BASIC mit 6
Builtin-Calls/Entity. Beispiel-Bench: 500 Entities × 100 Frames mit
`ECS_GET_FLOAT`/`ECS_ADD_FLOAT` → 215 ms auf der Native-VM. Mit
`ECS_INTEGRATE_FLOAT(world, "px", "vx")` → **5 ms (43× schneller)**.

Die Bulk-Ops verarbeiten eine ganze Component-Schicht in einer cdef-
Loop, ohne Python-Dispatch-Overhead pro Entity:

| Builtin | Wirkung |
|---|---|
| `ECS_INTEGRATE_FLOAT(w, target, delta)` | `target += delta` fuer alle Entities mit beiden Components |
| `ECS_INTEGRATE_INT(w, target, delta)` | INT-Variante |
| `ECS_SCALE_FLOAT(w, target, factor)` | `target *= factor` (z.B. Friction) |
| `ECS_FILL_FLOAT(w, target, value)` | alle Werte = value (Reset) |
| `ECS_FILL_INT(w, target, value)` | INT-Variante |
| `ECS_CLAMP_FLOAT(w, target, lo, hi)` | Bounds-Clamp |
| `ECS_REMOVE_DEAD(w, name, threshold)` | Entities mit `value <= threshold` zerstoeren |
| `ECS_COUNT_WITH(w, name)` | O(1) Halter-Zaehlung |

**Implementation:** [gamebasic/modules/ecs_py.py](gamebasic/modules/ecs_py.py)
(reine Python; die frühere Cython-Variante `ecs_native.pyx` wurde entfernt).
`_World` und `_Component` sind normale Python-Klassen. Sparse-Set-Ops als
Methoden. Fast-Path-Methoden auf `_World` (`get_float`, `add_float`, ...)
wickeln `_check_*` + `_get_value` + die Bulk-Loops in einem Call ab. Performance
für ECS-Hot-Paths liefert die native Runtime `gbrt` (Rust, `src/ecs.rs`).

**Beispiele:** [examples/bench_ecs_movement_v2.gb](examples/bench_ecs_movement_v2.gb)
(Integrate-only), [examples/bench_ecs_systems.gb](examples/bench_ecs_systems.gb)
(volles Bullet-Hell-Pattern mit 8 Systemen pro Frame).

**Game-Pattern-Lesson:** Wer ein Spiel-Hot-Path-System hat, das ueber
viele Entities laeuft, sollte es als Bulk-Op-Builtin schreiben statt
als pro-Entity-BASIC-Loop. Boilerplate fuer einen neuen Bulk-Builtin:
Methode auf `_World` in `modules/ecs_py.py` + `@builtin`-Wrapper in `ecs.py` +
(für Produktion) die entsprechende Logik in `rust/gb_runtime/src/ecs.rs`.

## Tree-Walker-Kernklassen (`_GBArray`, ECS) — reine Python, kein Cython mehr

Früher gab es zwei Cython-Module (`array_native.pyx` = `_GBArray`,
`ecs_native.pyx` = ECS-`_World`/`_Component`), die Hot-Path-Code im Tree-Walker
nach C verschoben. **Beide wurden entfernt.** Der Tree-Walker ist nur noch
Editor-/Referenzpfad — die Performance liegt vollständig in der nativen Runtime
`gbrt` (Rust). Kein `setup.py build_ext` mehr, kein neuer Cython-Code.

| Klasse | wohnt jetzt in | Hinweis |
|---|---|---|
| `_GBArray` | inline in `gamebasic/interpreter.py` | reine Python-Klasse (`array.array`-Backing für INTEGER/FLOAT, sonst Liste); Public-API unverändert (`element_type`/`dims`/`strides`/`values`/`total_size`/`flat_index`/`get_at`/`set_at`). |
| `_World`, `_Component` | `gamebasic/modules/ecs_py.py` | reine Python (Sparse-Set + Bulk-System-Ops); `ecs.py` importiert von dort. |

**pygame ist ENTFERNT (Stufe A)** — Grafik/Audio laufen nur in `gbrt`, der
Tree-Walker ist konsolen-only. Python bedient damit primär Editoren + Compiler/
Toolchain; der Rest läuft in `gbrt`. (Tree-Walker + Python-Compiler selbst zu
entfernen wäre eine separate „Stufe B".)

## Performance-Optimierungen im Compiler/VM

Mehrere Stufen, die zusammen die Native-VM auf 3–14× ggue Tree-Walker
bringen. Vollstaendige Bench-Tabelle in [docs/PERFORMANCE.md](docs/PERFORMANCE.md).

### Spezialisierte Numeric-Opcodes

11 Opcodes (`ADD_NN`, `SUB_NN`, `MUL_NN`, `DIV_NN`, `LT_NN`, `GT_NN`,
`LEQ_NN`, `GEQ_NN`, `EQ_NN`, `NEQ_NN`, `NEG_N`). Der Compiler emittiert
sie ueber `_expr_type(e)`-Inference (best-effort, konservativ), wenn
beide Operanden statisch als numerisch bekannt sind. Liest aus:
- Locals via `local_types[slot]`
- Globals via `_global_types[name]`
- Function-Return-Types
- BinaryOp/UnaryOp rekursiv
- **IndexAccess auf typisierte Arrays** (`buf[i]` bei `ARRAY OF INTEGER/FLOAT`
  -> Element-Typ) und **`Self.feld`** mit statisch bekanntem Skalartyp.
  Sicher, weil typisierte Arrays/Felder homogen sind. Damit greift der
  `_NN`-Pfad auch bei `buf[i] + i + j` und `Self.total + n`.

Bei `bench_loop` (tighter Numeric-Loop): **1.46×** auf Native-VM.

FOR-Loop-Bookkeeping (Increment + Bound-Check) wird ebenfalls ueber
spec-ops emittiert.

### 1D-Array-Index-Fast-Path

`LOAD_INDEX`/`STORE_INDEX` in beiden VMs haben einen Fast-Path fuer den
haeufigsten Fall: 1D-`_GBArray`, ein int-Index in Bounds. Ueberspringt die
`isinstance`-Cascade (str/tuple/array) + Index-Validierungs-Loop. Alle
Edge-Cases (String/Tupel/Multidim/OOB/bool) fallen unveraendert in den
generischen Pfad -> identische Fehler, kein Bit-identisch-Risiko. Kein neuer
Opcode, kein Compiler-Eingriff.

### Tree-Walker-Dispatch-Cache

`Interpreter._eval`/`_exec` memoisieren die Handler-Methode pro Node-Typ
(`_eval_cache`/`_exec_cache`: `type(node) -> gebundene Methode`) statt pro
Node `getattr(self, f"_eval_{name}")` zu bauen. Wirkt auf ALLEM Tree-Walker-
Code (~13 % bei expression-dichten Frames) -- relevant, weil der Editor-Run
und die Bench-Equivalenz den Tree-Walker nutzen.

### Inline-Cache fuer OOP-Dispatch

`CompiledFunction.caches`-Liste parallel zu `code`. Monomorphic IC fuer
`CALL_METHOD`, `LOAD_MEMBER`, `STORE_MEMBER` auf `_Instance`-Receivern.
Hit-Check: `obj.cls is cache[0]`. Spart `_resolve_method`-Call und
Dict-Lookup. Bei `bench_method_dispatch`: **1.34×**.

Cache wird auch im stub-Pfad (Phase 4a) konsistent kopiert
([compiler.py: stub.caches = compiled.caches](gamebasic/compiler.py)).

### Globals-as-Slots

Compile-Zeit-Aufloesung von Top-Level-DIM/CONST/Enum/For-var/Class-
Static-Namen zu Slot-Indizes (`_global_slots: dict[str, int]`).
Neue Opcodes:
- `LOAD_GLOBAL_SLOT idx`
- `STORE_GLOBAL_SLOT idx`
- `DECLARE_GLOBAL_SLOT (idx, name_idx, type, default)`
- `DECLARE_GLOBAL_CONST_SLOT (idx, name_idx, type)`

Die VM-Pfade fuehren `global_slots: list[_Slot]` parallel zum
`globals_`-Dict. `DECLARE_*_SLOT` schreibt den `_Slot` in BEIDE
Strukturen (gleiches Object), so bleiben name-basierte Ops
(`INPUT_NAME`, `LOAD_NAME`) konsistent.

Pre-registrierte Globals (`KEY_*`, `BLACK`, `WHITE`, `PI`, ...) leben
weiter nur im Dict, weil der Compiler sie nicht statisch erkennt --
sie gehen ueber den Fallback `LOAD_NAME`.

`Struct`/`Array`/`Map`-DIMs werden **nicht** slot-allokiert (ihre Init-
Pfade `DECLARE_STRUCT_NAME` / `DECLARE_ARRAY_NAME` haben spezielle
Allokation -- der Performance-Vorteil waere klein, der Kosten gross).

Bei `bench_loop`: **1.30×** zusaetzlich.

### Constant Folding

`_try_fold(e)` im Compiler — BinaryOp/UnaryOp mit konstanten Operanden
wird zu einem einzelnen `LOAD_CONST`. Konservativ:
- kein Folding bei Bool-in-Arithmetik
- kein Folding bei Division durch 0 (Runtime-Error besser)
- kein Folding bei extrem grossen POW-Werten (Safety-Cap)
- `and`/`or` werden NICHT gefoldet (Short-Circuit-Semantik bewahren)

Hilft Patterns wie `FOR i = 0 TO 100 - 1`, `width / 2`, `2 * 3.14`.

### Typed Array Backing + cdef `_GBArray`

`ARRAY OF INTEGER` nutzt `array.array('q')` (8-Byte signed int) statt
Python-Liste. `ARRAY OF FLOAT` analog `array.array('d')`. Spart
Box/Unbox bei jedem Zugriff. **64-bit-Limit fuer INTEGER-Arrays**
(-9.2e18..9.2e18) — Skalar-`DIM x AS INTEGER` bleibt arbitrary-
precision.

`_GBArray` ist eine reine Python-Klasse (inline in `interpreter.py`; die frühere
Cython-Variante `array_native.pyx` mit typed memoryviews wurde entfernt).
`get_at(indices)` / `set_at(indices, value)` bleiben die Zugriffs-API.

### Convert/Coerce-Fast-Path in der Python-VM

`vm.py` hat ein `_FAST_COERCE`-Dict mit pro-Typ-Funktionen statt einer
generischen `if`/`elif`-Cascade. Trifft den heissesten Pfad jeder
STORE-Op (Local/Global/Field/Index/Parameter-Binding).

## Sprite-Editor (`gbsprites`)

PySide6-basierter Pixel-Art-Editor in [`gamebasic/spriteeditor_qt.py`](gamebasic/spriteeditor_qt.py)
(UI-Schicht, 4200 LOC) plus Submodul [`gamebasic/spriteeditor/`](gamebasic/spriteeditor/)
mit `document.py` (Datenmodell), `tools.py` (Pixel-Tools), `tool_context.py`
(Tool-Host-Protocol), `icons.py` (programmatische Toolbar-Icons).

**Start:** `gbsprites` (leer) oder `gbsprites datei.png`. Aufruf-Trampoline in
`gbsprites.cmd` → `gbrun.py --sprites`. User-Doku: [docs/sprite-editor.md](docs/sprite-editor.md).

**Tools:** Pencil, Eraser, Bucket, Line, Rect, Ellipse, Eyedropper, Select,
**Lasso** (Freiform-Auswahl mit echter Pixel-Maske — Cut/Copy/Fuellen/Spiegeln/
Move wirken nur auf maskierte Pixel), Move, Magic Wand, Spray. Plus
Multi-Frame-Animation, **Ebenen pro Frame** (Layer-Stapel mit Sichtbarkeit/
Deckkraft/Merge-Down; Tools zeichnen auf die aktive Ebene, Anzeige/Export =
Composite; `frame.pixels` = aktive Ebene, `frame.composite()` = geflattet),
Onion-Skin (Deckkraft + Reichweite 1–3 einstellbar), Symmetrie
X/Y, Tile-Preview-3×3, Palette-Im-/Export (.gpl), Sheet-Import, Crop, Resize,
Farbe-Ersetzen, Flip/Rotate, Paste-as-new-Frame (`Ctrl+Shift+V`, intern oder
System-Clipboard-Bild).

**Export-Formate** (alle in `SpriteDoc.save_*`-Methoden; alle Bild-Exporte mit
optionalem `scale`-Parameter = Integer-Hochskalierung via Nearest-Neighbor,
UI fragt 1x/2x/4x/8x ab):
- `save_native(path)` — .gbsprite (JSON + base64-RGBA pro Frame, mit Frame-Dauern; **v5: Ebenen** als `layers`-Liste, `data` bleibt das geflattete Composite fuer aeltere Leser)
- `save_png_single(path, scale)` — einzelnes Frame (Composite)
- `save_sheet_png(path, layout, scale)` — horizontaler oder vertikaler Sheet
- `save_animated_gif(path, fps, loop, scale)` — GIF mit Transparenz
- `save_sheet_atlas(png_path, json_path, name_prefix, layout, scale)` — **PNG + JSON-Manifest** im Format, das `ATLAS_LOAD(...)` direkt versteht (siehe Sprite-Atlas-Section; Manifest-Rects werden bei scale>1 mitskaliert). Closed-Loop-Workflow: Editor schreibt, Engine liest.

**Atlas-Export-Detail:** Sprite-Namen sind standardmaessig `<png_basename>_<idx>`
(z.B. PNG `tiles.png` → Sprites `tiles_0`, `tiles_1`, ...). **Per-Frame-Namen:**
`Frame` hat ein `name`-Feld (Rechtsklick → „Umbenennen..." in der Frame-Liste);
benannte Frames nutzen ihren Namen direkt als Sprite-ID im Atlas, doppelte Namen
werden beim Export per `_<idx>`-Suffix eindeutig gemacht. Der Name persistiert in
`.gbsprite` (Format-Version 3, abwaerts-kompatibel — aeltere Dateien laden mit
leerem Namen).

**Tests:** `tests/test_spriteeditor_document.py` (Datenmodell, alle Export-Pfade,
inkl. Atlas-Roundtrip durch `ATLAS_LOAD`), `tests/test_spriteeditor_tools.py` (Pixel-Ops,
Bresenham, Brush-Offsets, Symmetrie), `tests/test_spriteeditor_tool_context.py`
(ToolHost-Protocol). 50+ Tests.

**Erweiterung:** neue Tools subclassen `Tool` in `tools.py`, implementieren
`begin/move/end`, registrieren sich in `SpriteEditorWindow._setup_tools()`.
Tool-Konvention im `tools.py`-Header dokumentiert.

## Tilemap-/Level-Editor (`gbtilemap`)

PySide6-Tool [`gamebasic/tilemapeditor_qt.py`](gamebasic/tilemapeditor_qt.py)
(UI) + Qt-freies Datenmodell [`gamebasic/tilemap/document.py`](gamebasic/tilemap/document.py)
(`TileMapDoc`/`TileLayer`/`ObjectLayer`/`MapObject` + Tiled-JSON-Serialisierung,
headless testbar). Tiles aus einem Tileset-PNG aufs Gitter malen (Stift/Radierer/
Füllen/Rechteck/Pipette/**Auswahl** `S` mit Copy/Cut/Paste rechteckiger Tile-
Regionen via Strg+C/X/V, `get_region`/`stamp_region`/`clear_region` im Modell),
mehrere Layer (Sichtbarkeit/Sortierung/umbenennen),
Per-Tile-Properties (`solid`/`damage`/...), Undo/Redo. **Object-Layer** (`+◇`):
Spawn-Punkte/Trigger/Zonen als Objekte mit Name/Typ/Properties (Klick = Punkt,
Ziehen = Rechteck, Doppelklick = bearbeiten, Entf/Rechtsklick = löschen) — der
Layer-Typ steuert die Canvas-Interaktion; Undo umfasst Tile- UND Objekt-Ops
(getaggte Stack-Einträge). **Multi-Tileset:** `doc.tilesets` ist eine Liste von
`Tileset`-Objekten mit fortlaufenden `firstgid`-Werten; `gid_to_tileset(gid)` /
`local_to_gid(ts,lid)` lösen GIDs auf, die Facade-Properties (`columns`/
`tile_count`/`tileset_image*`/`tile_src_rect`/`tile_properties` + `set_property`)
zeigen aufs **aktive** Tileset (Palette/Canvas-Code unverändert). Tileset-Combo
über der Palette wechselt/+/− Tilesets; Pipette schaltet aufs gid-Tileset um.
**Speichern/Laden = Tiled-JSON** (genau das Format, das `gamebasic/modules/tiled.py`
via `TILED_LOAD` liest: **N eingebettete Tilesets** mit eigenen `firstgid`s,
CSV-Tile-Daten, Per-Tile-Props + `objectgroup` mit Objekten als
`{name,type,value}`-Props; `TILED_OBJECT_*`/`TILED_TILESET_*` lesen sie). `GB-Code`
exportiert einen selbstständigen Renderer (`LOADIMAGE` pro Tileset + `TILED_LOAD` +
`DRAWIMAGEPART`, gid→Tileset per `firstgid`-Kette; Object-Layer werden nicht
gezeichnet, nur ein Auslese-Hinweis kommentiert). Schließt den Kreis mit dem
Sprite-Atlas-Export (Atlas-PNG als Tileset).

**Start:** `gbtilemap [datei.json]` / `gbrun.py --tilemap` / im Editor Toolbar +
`Datei`-Menü + `Strg+Shift+G` (in-process via `_open_tilemap_editor`, Icon
`"tilemap"` in `editor_qt/icons.py`). User-Doku: [docs/tilemap-editor.md](docs/tilemap-editor.md).

**Tests:** [`tests/test_tilemapeditor.py`](tests/test_tilemapeditor.py) — Datenmodell
(set/get/flood-fill/resize/Layer-Ops) + **Roundtrip-Garantie**: Editor-Export →
`TILED_LOAD` → identische Werte; eigener Save/Load-Roundtrip; der GB-Code-Export
lext+parst+kompiliert. **Stolperstein:** `MAP` ist ein Keyword (MAP OF T) → im
GB-Code-Export keine Variable `map` (heißt `lvl`).

## Form-Designer / WYSIWYG (`gbform`)

Eigenständiger PySide6-GUI-Designer im **Xojo-Stil** für das `gui`-Modul. Qt-frei
das Datenmodell [`gamebasic/formdesigner/document.py`](gamebasic/formdesigner/document.py)
(`FormDoc`/`Control`, `PALETTE`, `.gbform`-IO **exakt im Runtime-`gui`-JSON-Format**
+ Designer-Feld `name`, `generate_runner()`-Code-Gen), UI in
[`gamebasic/formdesigner_qt.py`](gamebasic/formdesigner_qt.py) (Palette links /
Canvas Mitte / Inspector rechts; Platzieren/Selektieren/Verschieben/Löschen,
Speichern/Laden, F5 = Run via `gbrt`). Start: `gbform [datei.gbform]` /
`gbrun.py --form`. **Xojo-Prinzip:** das `.gbform` speichert pro Control den
Event-Handler-**Namen** (`on_click`/`on_change`); `GUI_LOAD` stellt sie wieder
her und `GUI_UPDATE` ruft sie automatisch per Name auf — kein manuelles
Verdrahten. Doku [docs/form-designer.md](docs/form-designer.md), Tests
`tests/test_formdesigner_document.py` (Modell/Roundtrip/Codegen, headless) +
`tests/test_formdesigner_qt.py` (Konstruktion offscreen). Neue Control-Art:
Eintrag in `PALETTE` + ggf. gui-Runtime-Widget. **Geplanter Funktionsumfang
komplett** (siehe docs/form-designer.md „Status/geplant"): Resize-Handles +
Snap-Grid, Undo/Redo, integrierter Code-Editor (Doppelklick-Control →
Handler), Multi-Form-Projekte (`.gbproj`) sind alle vorhanden -- diese
CLAUDE.md-Zeile listete sie faelschlich noch als offen.

## Notenblatt-Editor (`gbscore`)

Eigenständiges PySide6-Tool für echte Notensatz-Darstellung (5-Linien-System,
Violin-/Bassschlüssel, Hilfslinien, Vorzeichen) statt des Zeilen-Rasters des
Trackers. Qt-frei das Datenmodell [`gamebasic/score/document.py`](gamebasic/score/document.py)
(`ScoreDoc`/`Track`/`NoteEvent`, Zeiten in Viertel-Beats) + Konvertierung
[`gamebasic/score/convert.py`](gamebasic/score/convert.py)
(`to_tracker_song(doc) -> (Song, warnings)`, mappt Beats auf Tracker-Zeilen
— 4 Zeilen/Beat), UI in [`gamebasic/scoreeditor_qt.py`](gamebasic/scoreeditor_qt.py)
(`_StaffView` pro Spur: Klick setzt/entfernt Noten via diatonischer
Tonhöhe↔Y- und Zeit↔X-Zuordnung, Dauer-Auswahl inkl. Punktierung ist
gleichzeitig das Snap-Raster, Vorzeichen-Toggle ♮/♯/♭, Pause-Toggle,
**Balken-Gruppierung** für zusammenhängende Achtel-/Sechzehntel-Läufe
gleicher Dauer im selben Beat via `_beam_groups()` — Läufe mit gemischten
Dauern bekommen weiterhin Einzel-Fähnchen statt Partial-Balken, siehe
Limitationen. **Noten per Ziehen verschieben** statt Löschen+Neu-Setzen:
`mousePressEvent` auf einer bestehenden Note startet einen Drag (die
`NoteEvent`-Instanz wird in `mouseMoveEvent` live mutiert -- kein Ghost-
Overlay nötig, `paintEvent` zeichnet sie einfach an ihrer aktuellen
Position), `mouseReleaseEvent` unterscheidet Klick-ohne-Bewegung (=
entfernen, wie bisher) von echtem Drag (= Kollision am Zielort auflösen +
Liste neu sortieren); eine Pause bleibt beim Ziehen eine Pause (nur
`start_beat` ändert sich, `pitch` bleibt `None`)). Jede Spur hat GENAU EIN
Instrument (Presets aus
`tracker.presets`) und beim Schlüsselwechsel (Violin-/Bassschlüssel) einen
optionalen Oktav-Transpose-Dialog (`_octave_shift_for_clef` rückt den
Notendurchschnitt der Spur ans neue System, wenn er sonst weit ab läge —
volle Oktaven, Melodie/Intervalle bleiben exakt erhalten), Wiedergabe über
den geteilten additiven Mixer [`gamebasic/audio_preview.py`](gamebasic/audio_preview.py)
(`Mixer` — derselbe, den auch der Tracker nutzt; ein einziger dauerhafter
`sounddevice.OutputStream` mischt alle gleichzeitig klingenden Stimmen
additiv, weil `sd.play()` selbst keine Überlappung kann). Statusleiste
(Info-Panel) zeigt live den aktuellen Eingabe-Modus (Dauer/Vorzeichen/
Pause), Stück-Überblick (Spuren/Beats/BPM) und Kurzhinweise. **Undo/Redo**
über `SnapshotUndo` (`editor_qt/undo_history.py`, gleiches Muster wie
Tracker/SFX/Partikel-Editor) — snapshotted das ganze `ScoreDoc.to_dict()`,
`_mark_dirty()` ist der einzige Aufrufpunkt für `undo.mark()` (jede
Doc-Mutation läuft schon durch diese eine Methode, kein Streuen über
einzelne Handler nötig). **Ungespeicherte-Änderungen-Schutz**: Fenstertitel
zeigt `*` bei `_dirty`, `closeEvent`/`_new_doc`/`_open` fragen über
`_confirm_dirty()` nach (Speichern/Verwerfen/Abbrechen), gleiches Muster wie
`spriteeditor_qt.py`s `_confirm_dirty()`. Start:
Code-Editor-Toolbar/Menü (`Strg+Shift+N`,
`editor_qt/main_window.py:_open_score_editor`) oder
`gbscore [datei.json]` / `gbrun.py --score`. Fenster startet maximiert
(`F11` = echtes Vollbild, wie Audio Studio). Eigenes `*.json`-Format
(`"format": "gbscore-song"`, permissiv wie `Song.from_dict`) via
`ScoreDoc.save_json/load_json` **UND** direkte Übernahme in den Tracker
("In Tracker öffnen": `to_tracker_song` konvertiert, Warnungen werden
angezeigt, das Ergebnis wird als Tracker-Projekt gespeichert und `gbtracker`
per Subprozess mit der Datei gestartet).

**Notationszusätze über einen exklusiven Eingabe-Modus** (`entry_mode`:
`note`/`rest`/`slur`/`fingering`/`staccato`, 5er-`QButtonGroup` in der
Toolbar): `NoteEvent.staccato`/`NoteEvent.fingering` + `Track.slurs` (Liste
von Beat-Positions-Paaren, JSON-serialisierbar statt Objekt-Referenzen,
damit Undo/Redo-Snapshots sie automatisch mitnehmen). Im **Bindebogen**-
Modus verbindet ein zweiter Klick auf eine andere Note die Anker-Note mit
ihr (`Track.add_slur`, gerendert als quadratische Bézierkurve in
`_draw_slurs`); Rechtsklick entfernt dort gezielt einen Bogen, ohne die
Note zu löschen (`Track.remove_slurs_at`); ein Ziehen der Note verschiebt
ihren Bogen-Anker automatisch mit (`Track.relocate_slurs`, in
`mouseReleaseEvent`s Drag-Finalisierung). **Fingersatz**-Modus weist die
per Spinbox gewählte Zahl (1..5) zu (erneuter Klick mit derselben Zahl
löscht sie). **Staccato**-Modus schaltet `NoteEvent.staccato` um -- wirkt
NICHT nur optisch: `to_tracker_song` platziert dafür ein früheres
`NOTE_OFF` (`STACCATO_FACTOR=0.5` der notierten Dauer, mind. 1 Zeile,
siehe `gamebasic/score/convert.py`) und `ScoreEditor._trigger_note`
rendert für die Editor-eigene Wiedergabe entsprechend kürzer. Bindebögen/
Fingersätze sind rein informativ, keine Tracker-Entsprechung. Ein
Moduswechsel bricht eine offene Bindebogen-Anker-Auswahl ab
(`_on_mode_changed`).

**V1-Limitationen** (bewusst, dokumentiert statt stillschweigend verschluckt):
festes 4/4-Metrum (UI zeigt/ändert `time_sig` nicht), ein Instrument pro
Spur (kein Pattern-Zell-Override wie im Tracker), Akkorde (mehrere Noten
gleichen Start-Beats auf einer Spur) werden beim Tracker-Export auf die
höchste Note reduziert (ein Tracker-Kanal ist einstimmig), Balken-Gruppierung
nur innerhalb gleichlanger Achtel-/Sechzehntel-Läufe (keine Partial-Balken bei
gemischten Dauern), Noten die über eine 64-Zeilen-Tracker-
Pattern-Grenze hinaus klingen würden werden dort gekappt, kein
Schlagzeug-Spurtyp (der Pflicht-Drum-Kanal bleibt beim Export unbelegt),
kein optisches Notenlinien-Layout (keine automatische Kollisionsvermeidung
zwischen Vorzeichen/Fingersätzen/Bindebögen/Hilfslinien). Bindebögen und
Fingersätze bleiben rein informativ (keine Tracker-/Wiedergabe-Wirkung) --
echtes Legato/Phrasing waere machbar (Kira `set_playback_rate`+Tween als
Glide-Primitiv existiert bereits fuer AUDIO_PITCH), aber bewusst nicht gebaut
(bräuchte neuen Tracker-Tie-Befehl + Player-Logik in audio.rs).
**Vorzeichen** (♮/♯/♭) werden seit 2026-07-06 korrekt als Kreuz ODER B
notiert (`NoteEvent.accidental` haelt fest, welches Vorzeichen beim Setzen
aktiv war) -- die B-Notation ist NICHT mehr auf "immer Kreuz" beschraenkt.

Doku [docs/score-editor.md](docs/score-editor.md), Tests
`tests/test_score_document.py` + `tests/test_score_convert.py` (Datenmodell +
Konvertierung, headless), `tests/test_scoreeditor_qt.py` (Offscreen-UI),
`tests/test_audio_preview_mixer.py` (geteilter Mixer).

## Language Server (`gamebasic.lsp`) + VSCode-Extension

Externe Editor-Unterstützung via **LSP**, mit derselben Sprach-Intelligenz wie
der Qt-Editor. Start: `py -m gamebasic.lsp` (stdio, JSON-RPC). Aufbau bewusst
zweigeteilt: [`gamebasic/lsp/features.py`](gamebasic/lsp/features.py) =
**transport-freie** Feature-Logik (Text+Position → LSP-Daten: diagnostics/
completions/hover/definition/references/document_symbols), headless testbar;
[`gamebasic/lsp/server.py`](gamebasic/lsp/server.py) (`LspServer`) = nur
JSON-RPC + Dokument-Store + Position/URI-Mapping. Beide bauen auf den schon
vorhandenen Editor-Bausteinen auf (`editor_qt/symbols.py`, `error_check.py`,
`builtin_docs.py`, `completer.py`) — **keine Logik-Duplizierung**. Tests:
[`tests/test_lsp_features.py`](tests/test_lsp_features.py) (Feature-Logik) +
[`tests/test_lsp_server.py`](tests/test_lsp_server.py) (Protokoll + echter
stdio-Subprozess). VSCode-Extension in [`vscode-gamebasic/`](vscode-gamebasic/):
`extension.js` startet den Server, die TextMate-Grammatik wird aus den echten
Lexer-Keywords + Built-ins **generiert** (`build_grammar.py`). Doku
[docs/lsp.md](docs/lsp.md). **Bei neuen Keywords/Built-ins:** Grammatik neu
generieren (`python vscode-gamebasic/build_grammar.py`).

## Front-End-Portierung nach Rust (Lexer → Parser → Compiler)

**Laufendes Ziel:** Python langfristig nur noch in den Editoren — `gbrt` soll
selbst aus Quelltext Bytecode erzeugen (heute macht das die Python-Toolchain).
Die Front-End-Stufen werden **inkrementell** nach Rust portiert, **jede gegen
Python verifiziert** (cargo+rustc vorhanden → hier beweisbar). **ALLE 5 STUFEN
FERTIG:** Lexer (1) + Parser (2) + Compiler 3a–3e (3) + Preprocess/IMPORT (4) +
Verdrahtung (5). **`gbrt run datei.gb` ist ein eigenständiger End-to-End-Lauf
ohne Python** (preprocess→lex→parse→compile→VM, chdir ins Datei-Verzeichnis für
relative Asset-/IMPORT-Pfade; `gbrt datei.gb` ohne `run` per `.gb`-Auto-Detect
genauso, `.gbc` läuft weiter den direkten VM-Pfad). Debug-Einstiege
`gbrt --tokens` / `--ast` / `--preprocess` / `--runsrc` geben Token-Strom bzw.
AST bzw. gemergte Quelle aus bzw. führen ohne chdir aus (Dev/Parity).
Parity gegen Python: [`tests/test_rust_lexer_parity.py`](tests/test_rust_lexer_parity.py)
(137) + [`tests/test_rust_parser_parity.py`](tests/test_rust_parser_parity.py) (96)
+ [`tests/test_rust_compiler_parity.py`](tests/test_rust_compiler_parity.py) (71)
+ [`tests/test_rust_preprocess_parity.py`](tests/test_rust_preprocess_parity.py) (7)
+ [`tests/test_rust_run_parity.py`](tests/test_rust_run_parity.py) (2).
Dateien: [`src/lexer.rs`](rust/gb_runtime/src/lexer.rs),
[`src/ast.rs`](rust/gb_runtime/src/ast.rs),
[`src/parser.rs`](rust/gb_runtime/src/parser.rs),
[`src/preprocess.rs`](rust/gb_runtime/src/preprocess.rs),
[`src/compiler.rs`](rust/gb_runtime/src/compiler.rs). Plan/Stufen/Gotchas:
[docs/rust-frontend-port.md](docs/rust-frontend-port.md).
**Compiler-Gate = Output-Parität** (`gbrt --runsrc` stdout == Python-TW), NICHT
byte-exakter Bytecode: gbrt's VM kann beide Opcode-Formen, der Rust-Compiler
emittiert die generischen (kein Folding/`_NN`/IC) → identisches Verhalten, viel
weniger Code. **3a–3e fertig:** Skalar/Arithmetik/IF/WHILE/Builtins (3a) + FOR/Arrays/Index/
INPUT/DATA-READ/Locals (3b) + User-SUB/FUNCTION (Rekursion/CALL_USER/Named-Args/
Defaults/Variadic/FUNCREF, 3c) + Klassen/Structs (NEW/Member/Self/Methoden-
Calls/Vererbung/Properties/Operatoren/STATIC/ENUM, 3d) + SELECT/FOR EACH/REPEAT/
Tupel+Destructuring/WITH/TRY-CATCH-THROW/Slicing/List-Set-Dict-Comprehensions/
IIF/Coroutinen-YIELD + TUPLE/COROUTINE/FUNCREF/IMAGE-DIM-Typen (3e). Nicht-
unterstützt → `Err("Stufe 3e: ...")`. **Stufe 4 (Preprocess) fertig:**
`src/preprocess.rs` portiert `preprocess.process()` — `IMPORT "datei.gb"`
rekursiv inlinen (mit `' === IMPORT … ===`-Markern + `seen`-Dedup),
`IMPORT "modul"[ AS x]` → Kommentar (gbrt hat Modul-Builtins nativ). Importierte
Module liefern ihre externen Typen (`MODULE_TYPES`) an `compile_to_gbc(ast,
external_types)` → `DIM v AS VEC2` kompiliert nach `IMPORT "vec2"`. `--runsrc`
schaltet Preprocess vor. **Stufe 5 (Verdrahtung) fertig:** `gbrt run datei.gb`
(+ `gbrt datei.gb` per `.gb`-Auto-Detect) — eigenständiger End-to-End-Lauf ohne
Python, chdir ins Datei-Verzeichnis (relative IMPORT-/Laufzeit-Pfade); gemeinsame
Kette `compile_and_run_source` (geteilt mit `--runsrc`, das ohne chdir läuft).
Tree-Walker bleibt Referenz + `@builtin`-Host. AST-Parity-Gotchas: `.line` kein
Feld, `Param.by_ref` ist ein Token. 3e-Gotcha: `_collect_data` rekursiert in
SELECT/TRY, aber NICHT FOR EACH/WITH. 4-Gotcha: `MODULES`/`MODULE_TYPES` hardcoded,
mit `modules.discover_modules()` synchron halten. **Anschluss-Features fertig:**
(a) **Selbst-Export** `gbrt --export datei.gb [out]` — kompiliert Quelle → `.gbc`,
hängt Payload an Kopie der eigenen Exe (wie `export.py`, aber ohne Python),
kopiert `assets/`. (b) **Aliasierte Modul-IMPORTs** `IMPORT "json" AS j` —
`compile_env` liefert `(alias, modul)`-Paare, der Compiler bildet `j_parse`→
`json_parse` / `v`→`vec2` zurück (gbrt findet sie nativ), aliasierte externe
Typen (`j_handle`/`v`) sind gültige DIM-Typen. (c) **WASM (gebaut + verifiziert)** — emscripten-Einstieg
kompiliert `/program.gb` selbst (kein Pyodide), `build_wasm.py` bettet die Quelle
ein + verdrahtet das Windows-emscripten-Env automatisch (`setup_emscripten_env`:
CC/CXX/AR/Linker→`.exe`, bindgen-Includes, Ninja). `node web/gbrt.js` ==
Tree-Walker verifiziert. Toolchain (emscripten 6.0.0 + wasm-Target) installiert.

## Web-Playground (gbrt → WASM) — experimentell/Gerüst

`gbrt` als WebAssembly (emscripten) im Browser. **Status: Gerüst, nicht
gebaut/verifiziert** (kein emscripten/raylib-web in der Dev-Umgebung). Teile:
[`rust/build_wasm.py`](rust/build_wasm.py) (`.gb`→`web/program.gbc`, dann
`cargo`+emscripten-Build, tolerant wenn Toolchain fehlt), cfg-gegateter
WASM-Einstieg in [`rust/gb_runtime/src/main.rs`](rust/gb_runtime/src/main.rs)
(`#[cfg(target_os = "emscripten")]` liest `/program.gbc`), Web-Harness
[`web/`](web/) (`index.html` + `playground.js`, emscripten-`Module`-Konfig).
**Kernhürde:** der VM-Render-Loop in `vm.run()` blockiert → Web braucht ASYNCIFY
(gesetzt) oder Umbau auf `emscripten_set_main_loop`. **Compiler bleibt Python**
→ Playground führt vorab kompilierte `.gbc` aus; Live-Kompilierung bräuchte
Pyodide dazu. Doku/Grenzen: [docs/web-playground.md](docs/web-playground.md).
Tests [`tests/test_build_wasm.py`](tests/test_build_wasm.py) (Geruest/Harness,
nicht der emscripten-Build).

## Build und Test

```
.venv\Scripts\python.exe rust\build_runtime.py        # Runtime gbrt (Rust)
.venv\Scripts\python.exe -m pytest tests/ -v          # run_gb-Golden gegen gbrt
.venv\Scripts\python.exe gbrun.py examples/<file>.gb  # ausführen (-> gbrt run)
```

Nur `gbrt` wird gebaut (kein Cython/PyO3 mehr). Builtins/Module sind in Rust
(`rust/gb_runtime/src/`); Korrektheit über run_gb-Golden-Tests + Rust-`#[test]`.
