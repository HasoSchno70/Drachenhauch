# Befehlssatz-Roadmap (Audit 2026-06-05)

Ergebnis eines Audits des GameBasic-Befehlssatzes (~710 Builtins): Lücken,
Inkonsistenzen und echte Editor-↔-Export-Fallstricke. **Leitsatz: jeder neue/
geänderte Befehl muss nativ in `gbrt` laufen** — also immer in BEIDEN Pfaden
umsetzen und per Parity-Test absichern.

## Umsetzungs-Checkliste pro Befehl
- [ ] Tree-Walker: `@builtin`/`@graphics_builtin` in `gamebasic/interpreter.py`
      (bzw. passendes `gamebasic/modules/*.py`).
- [ ] Native Runtime: `rust/gb_runtime/src/builtins.rs` (+ `vm.rs`-Dispatch,
      ggf. `graphics.rs`/`audio.rs`), danach `rust\build_runtime.py`.
- [ ] Parity-Snippet in `tests/test_gbrt_parity.py` (TW == gbrt). PRNG-/Uhr-
      basierte Befehle als „erwartet unterschiedlich" behandeln.
- [ ] Doku: `builtin_docs.py` (Hover) + `vscode-gamebasic/build_grammar.py` neu
      generieren; README/CLAUDE bei Bedarf.

---

## WP0 — Native Fallstricke (zuerst — Bugs, kein Komfort)
Läuft im Editor (Tree-Walker), crasht/divergiert im exportierten Spiel (gbrt):

- [x] **`PHYSICS_BROAD_*` nativ** (NEW/ADD/CLEAR/COUNT/QUERY/PAIR_A/PAIR_B/
      PAIR_COUNT). Uniform-Grid-Broadphase in `rust/gb_runtime/src/physics.rs`
      (portiert aus `gb_native/src/broadphase.rs` — selbe Paare/Reihenfolge),
      `Value::PhysicsBroad`, Dispatch + Validierung (Radius>=0, Paar-Index-
      Bounds) in `builtins.rs`. Parity-Snippet `physics_broad` (bit-identische
      Paare TW==gbrt).
- [x] **`TIME$`/`DATE$` nativ** — in `builtins.rs` (`local_datetime()`: Windows
      `GetLocalTime`, sonst UTC-Fallback via civil-from-days). Format-Test in
      `test_gbrt_parity.py::test_time_date_format_tw_and_gbrt` (kein Exakt-
      Vergleich, da Wert variiert).
- [x] **`DRAWTILEMAP` nativ** — `graphics.rs::draw_tilemap` (jedes Tile via
      `draw_image_part`, Camera/Zoom korrekt) + Dispatch in `vm.rs`. Rendert jetzt
      tatsaechlich nativ (CLAUDE.md-Aussage damit korrekt). Manuell verifiziert
      (kein stdout fuer Parity).
- [x] **`INKEY$`/`WAITKEY` + Core-`JOYSTICK_*` nativ; `SCROLL` TW-only.**
      - `INKEY$` (raylib `get_char_pressed`), `WAITKEY` (blockt via
        `window_should_close`-Pump → raylib-Keycode, -1 bei Fensterschluss).
      - `JOYSTICK_COUNT/NAME/AXIS` exakt auf raylib-Gamepad; `BUTTON/HAT` als
        Best-Effort (raylib-Standard-Layout; Roh-Index weicht von pygame ab →
        fuer praezise Bindings `IMPORT "input"`). Ungueltiger Joystick-INDEX
        wirft wie der TW (Sub-Index liefert 0/false).
      - `SCROLL`: gbrt zeichnet jeden Frame neu aus dem Command-Buffer (kein
        persistenter Framebuffer) → graceful No-Op, **Tree-Walker-only**
        (Kommentar in `vm.rs`).
- [x] **Float-Koordinaten angleichen.** Befund: schon konsistent. Im
      Tree-Walker ist `_check_int` ein Alias auf `_check_intish`
      (`interpreter.py:3541`, seit dem Initial-Commit) → die Zeichenprimitive
      (LINE/BOX/RECT/CIRCLE/PLOT/GRADIENT*) akzeptieren Floats und trunkieren
      sie, genau wie gbrts `gi()`. Die Audit-Annahme „TW lehnt `LINE(10.5,…)`
      ab" war falsch. Empirisch verifiziert (beide Pfade laufen identisch);
      `vm.rs`-Kommentar praezisiert (verweist auf den Alias als Quelle).

## WP1 — Array-Power (✅ ERLEDIGT 2026-06-05, **gbrt-only**)
> Ab dieser WP gilt die neue Direktive: neue Builtins NUR noch in gbrt (Rust),
> der Python-Tree-Walker (`interpreter.py`) wird nicht mehr erweitert. Tests =
> gbrt-Golden (`tests/test_gbrt_builtins.py`, läuft via `gbrt --runsrc`).
> Editor-Metadaten (Completion/Grammar/Hover) via `BUILTIN_DOCS`.
- [x] **Dynamische Arrays:** `ARRAY_PUSH`/`POP`/`INSERT`/`REMOVE_AT`/`REDIM`
      (1D, mutieren IN PLACE: `values` + `dims=[len]` + `strides=[1]`).
      `Value::Array` ist von Natur aus growable — die Cython-Hürde entfiel mit
      der Ent-Cythonisierung des TW.
- [x] **Aggregate:** `ARRAY_SUM`/`ARRAY_AVG`/`ARRAY_MIN`/`ARRAY_MAX`/
      `ARRAY_FILL`/`ARRAY_COPY` (1D numerisch; FILL via `coerce_elem`, COPY
      unabhängig).
- [x] **`SORT(arr, …)`** mit Descending-`BOOL` (builtins.rs) + FUNCREF-Comparator
      (vm.rs `sort_with_comparator`, stabil, cmp(a,b)→INT).

> **✅ Folge-Schritt erledigt (2026-06-05):** Run-/Export-Pfad auf gbrts Rust-
> Frontend umgestellt. `gbrun.py --native` → `gbrt run`, `gbrun.py --export` →
> `gbrt --export`, Editor-Run (`output_console._start_native`) → `gbrt run`,
> Editor-Export (`main_window._export_active`) → `gbrt --export`. Damit laufen die
> gbrt-only-Builtins überall (verifiziert: `gbrun.py --native`/`--export` + die
> exportierte .exe). gbrts Compile-Fehler bekamen das Format `datei.gb:Zeile:`
> (Editor-klickbar). Python-Compiler nur noch für Bench/Tests.

## WP2 — Spiel-Quickwins (✅ ERLEDIGT 2026-06-05, nativ in beiden Pfaden)
- [x] **`MOUSEWHEEL` exponieren** — Builtin in beiden Pfaden (Backend war da:
      `pop_mouse_wheel` / raylib `GetMouseWheelMove`). Graceful 0 ohne SCREEN.
- [x] **`SCREENWIDTH`/`SCREENHEIGHT`** — Zurücklesen der logischen Fenstergröße
      (0 vor SCREEN, wie TW `_buf_size`).
- [x] **Ranged Random:** `RANDINT(lo,hi)`, `RANDF(lo,hi)`, `CHOICE(arr)`,
      `SHUFFLE(arr)`. PRNG ≠ Python → Parity „erwartet unterschiedlich"
      (Strukturtest prüft Bereich/Multiset-Invariante).
- [x] **Farb-Helfer:** `HSV`→RGB, `COLOR_LERP`, `RED`/`GREEN`/`BLUE`-Extraktion.
      `RGBA` **bewusst weggelassen** — der Draw-Pfad ist 24-Bit-RGB ohne
      Alpha-Kanal (`col()` forciert 255, TW maskiert `&0xFFFFFF`), Alpha wäre
      irreführend.
- [x] **Math:** `ASIN`/`ACOS` (Domain-Check), `HYPOT`, `DEG`/`RAD`, `LERP`,
      `REMAP`, `FRAC`; Konstante `TAU`. **`E` bewusst weggelassen** — `e` ist
      ein häufiger `CATCH e`-Variablenname; eine gleichnamige Konstante würde
      das brechen (nutze `EXP(1)`).
- [x] **`ROUND(x, decimals)`** → FLOAT (Half-to-even via Decimal-Formatierung,
      bit-identisch: Python `f"{x:.nf}"` == Rust `format!("{:.n}")`).

## WP3 — String + Datei (✅ ERLEDIGT 2026-06-05, **gbrt-only**)
- [x] **String:** `LTRIM$`/`RTRIM$`, `REVERSE$`, `STARTSWITH`/`ENDSWITH`/
      `CONTAINS`, `BIN$`/`OCT$` (mit Vorzeichen), `ISNUMERIC`/`TRYVAL` (robustes
      Parsen via `parse_number`; `VAL` bleibt unverändert).
- [x] **Datei/Verzeichnis:** `DIRLIST` (sortiert), `DIREXISTS`, `MKDIR`
      (rekursiv), `DELETEFILE`, `RENAME`, `WRITEALL`, `READLINES`, `FILESIZE`,
      `PATHJOIN` (mit `/`). Alle pfadbasiert via `std::fs`.

## WP4 — Konsistenz / Aliase (geringes Risiko; NICHT umbenennen, nur Aliase + Doku)
- [ ] BASIC-Aliase: `SGN`→`SIGN`, `SQRT`→`SQR`, `LTRIM$`/`RTRIM$`, `REVERSE$`,
      `STARTSWITH`/`ENDSWITH`.
- [ ] `AUDIO_SET_VOLUME`/`AUDIO_MUSIC_SET_VOLUME` als Alias (Asymmetrie zu
      `AUDIO_GET_VOLUME`); alte Namen behalten.
- [ ] Container-Methode `arr.join(sep)` ergänzen (`CONTAINER_METHODS`).
- [ ] Doku-Korrekturen: CLAUDE.md (DRAWTILEMAP), `vm.rs`-Kommentar (intish).
- [ ] Nur dokumentieren (Rename zu riskant): `$`-Suffix gilt nur im Core; doppelte
      Audio-API (`PLAYSOUND` vs `AUDIO_PLAY`); Suffix-`2`-Mehrdeutigkeit
      (squared/2D/vec2-Arität); `SPRITE_COLLIDE` vs `SPRITE_COLLIDES`; `CAMERA_X`
      (2D) vs `CAMERA3D_X`.

---

**Empfohlene Reihenfolge:** WP0 → WP2 → WP1 → WP3 → WP4.
