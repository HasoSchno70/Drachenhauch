# Befehlssatz-Roadmap (Audit 2026-06-05)

> **⚠️ Lesehinweis (Stufe B).** Diese Roadmap entstand, als es noch zwei Pfade
> gab (Python-Tree-Walker + `dhrt`). Der Tree-Walker und die Python-Toolchain
> sind inzwischen **entfernt** — neue/geänderte Befehle kommen **nur** in `dhrt`
> (`builtins.rs`/`vm.rs`) + ein run_gb-Golden-Test. „BEIDE Pfade"/`interpreter.py`
> unten sind historisch.

Ergebnis eines Audits des Drachenhauch-Befehlssatzes (~710 Builtins): Lücken,
Inkonsistenzen und echte Editor-↔-Export-Fallstricke. **Leitsatz: jeder neue/
geänderte Befehl muss nativ in `dhrt` laufen** — per run_gb-Golden-Test
absichern.

## Umsetzungs-Checkliste pro Befehl
- [ ] Tree-Walker: `@builtin`/`@graphics_builtin` in `drachenhauch/interpreter.py`
      (bzw. passendes `drachenhauch/modules/*.py`).
- [ ] Native Runtime: `rust/drachenhauch_runtime/src/builtins.rs` (+ `vm.rs`-Dispatch,
      ggf. `graphics.rs`/`audio.rs`), danach `rust\build_runtime.py`.
- [ ] Parity-Snippet in `tests/test_dhrt_parity.py` (TW == dhrt). PRNG-/Uhr-
      basierte Befehle als „erwartet unterschiedlich" behandeln.
- [ ] Doku: `builtin_docs.py` (Hover) + `vscode-drachenhauch/build_grammar.py` neu
      generieren; README/CLAUDE bei Bedarf.

---

## WP0 — Native Fallstricke (zuerst — Bugs, kein Komfort)
Läuft im Editor (Tree-Walker), crasht/divergiert im exportierten Spiel (dhrt):

- [x] **`PHYSICS_BROAD_*` nativ** (NEW/ADD/CLEAR/COUNT/QUERY/PAIR_A/PAIR_B/
      PAIR_COUNT). Uniform-Grid-Broadphase in `rust/drachenhauch_runtime/src/physics.rs`
      (portiert aus `gb_native/src/broadphase.rs` — selbe Paare/Reihenfolge),
      `Value::PhysicsBroad`, Dispatch + Validierung (Radius>=0, Paar-Index-
      Bounds) in `builtins.rs`. Parity-Snippet `physics_broad` (bit-identische
      Paare TW==dhrt).
- [x] **`TIME$`/`DATE$` nativ** — in `builtins.rs` (`local_datetime()`: Windows
      `GetLocalTime`, sonst UTC-Fallback via civil-from-days). Format-Test in
      `test_dhrt_parity.py::test_time_date_format_tw_and_dhrt` (kein Exakt-
      Vergleich, da Wert variiert).
- [x] **`DRAWTILEMAP` nativ** — `graphics.rs::draw_tilemap` (jedes Tile via
      `draw_image_part`, Camera/Zoom korrekt) + Dispatch in `vm.rs`. Rendert jetzt
      tatsaechlich nativ (CLAUDE.md-Aussage damit korrekt). Manuell verifiziert
      (kein stdout fuer Parity).
- [x] **`INKEY$`/`WAITKEY` + Core-`JOYSTICK_*` nativ; `SCROLL` TW-only.**
      - `INKEY$` (raylib `get_char_pressed`), `WAITKEY` (blockt via
        `window_should_close`-Pump → raylib-Keycode, -1 bei Fensterschluss).
      - `JOYSTICK_COUNT/NAME/AXIS` exakt auf raylib-Gamepad; `BUTTON/HAT` als
        Best-Effort (raylib-Standard-Layout; der Roh-Button-Index ist pad-
        abhaengig → fuer praezise Bindings `IMPORT "input"`). Ungueltiger Joystick-INDEX
        wirft wie der TW (Sub-Index liefert 0/false).
      - `SCROLL`: dhrt zeichnet jeden Frame neu aus dem Command-Buffer (kein
        persistenter Framebuffer) → graceful No-Op, **Tree-Walker-only**
        (Kommentar in `vm.rs`).
- [x] **Float-Koordinaten angleichen.** Befund: schon konsistent. Im
      Tree-Walker ist `_check_int` ein Alias auf `_check_intish`
      (`interpreter.py:3541`, seit dem Initial-Commit) → die Zeichenprimitive
      (LINE/BOX/RECT/CIRCLE/PLOT/GRADIENT*) akzeptieren Floats und trunkieren
      sie, genau wie dhrts `gi()`. Die Audit-Annahme „TW lehnt `LINE(10.5,…)`
      ab" war falsch. Empirisch verifiziert (beide Pfade laufen identisch);
      `vm.rs`-Kommentar praezisiert (verweist auf den Alias als Quelle).

## WP1 — Array-Power (✅ ERLEDIGT 2026-06-05, **dhrt-only**)
> Ab dieser WP gilt die neue Direktive: neue Builtins NUR noch in dhrt (Rust),
> der Python-Tree-Walker (`interpreter.py`) wird nicht mehr erweitert. Tests =
> dhrt-Golden (`tests/test_dhrt_builtins.py`, läuft via `dhrt --runsrc`).
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

> **✅ Folge-Schritt erledigt (2026-06-05):** Run-/Export-Pfad auf dhrts Rust-
> Frontend umgestellt. `dhrun.py --native` → `dhrt run`, `dhrun.py --export` →
> `dhrt --export`, Editor-Run (`output_console._start_native`) → `dhrt run`,
> Editor-Export (`main_window._export_active`) → `dhrt --export`. Damit laufen die
> dhrt-only-Builtins überall (verifiziert: `dhrun.py --native`/`--export` + die
> exportierte .exe). dhrts Compile-Fehler bekamen das Format `datei.dh:Zeile:`
> (Editor-klickbar). (Hinweis: der Python-Compiler ist seit Stufe B entfernt —
> dhrt kompiliert selbst; „beide Pfade" unten sind historisch.)

## WP2 — Spiel-Quickwins (✅ ERLEDIGT 2026-06-05, nativ in beiden Pfaden)
- [x] **`MOUSEWHEEL` exponieren** — Builtin in beiden Pfaden (Backend war da:
      `pop_mouse_wheel` / raylib `GetMouseWheelMove`). Graceful 0 ohne SCREEN.
- [x] **`SCREENWIDTH`/`SCREENHEIGHT`** — Zurücklesen der logischen Fenstergröße
      (0 vor SCREEN, wie TW `_buf_size`).
- [x] **Ranged Random:** `RANDINT(lo,hi)`, `RANDF(lo,hi)`, `CHOICE(arr)`,
      `SHUFFLE(arr)`. PRNG ≠ Python → Parity „erwartet unterschiedlich"
      (Strukturtest prüft Bereich/Multiset-Invariante).
- [x] **Farb-Helfer:** `HSV`→RGB, `COLOR_LERP`, `RED`/`GREEN`/`BLUE`-Extraktion.
- [x] **Alpha-Kanal (2026-06-08, dhrt):** `RGBA(r,g,b,a)` packt Alpha ins obere
      Byte (`&Haarrggbb`); `ALPHA(farbe)` liest es. `col()` (graphics.rs) wertet
      das obere Byte als Alpha aus — **0 = deckend** (Rückwärts-Kompatibilität:
      alte `&Hrrggbb`/`RGB(...)` bleiben voll deckend). Damit zeichnen
      `BOX/RECT/CIRCLE/LINE/TEXT/PLOT/…` sowie der Bild-/Sprite-Tint halb-
      transparent (Standard-Blendmodus „alpha"). `RGBA(_,_,_,0)` → Alpha 1
      (ganz transparent = einfach nicht zeichnen). Tests in
      `tests/test_builtins_extra.py`.
- [x] **Math:** `ASIN`/`ACOS` (Domain-Check), `HYPOT`, `DEG`/`RAD`, `LERP`,
      `REMAP`, `FRAC`; Konstante `TAU`. **`E` bewusst weggelassen** — `e` ist
      ein häufiger `CATCH e`-Variablenname; eine gleichnamige Konstante würde
      das brechen (nutze `EXP(1)`).
- [x] **`ROUND(x, decimals)`** → FLOAT (Half-to-even via Decimal-Formatierung,
      bit-identisch: Python `f"{x:.nf}"` == Rust `format!("{:.n}")`).

## WP3 — String + Datei (✅ ERLEDIGT 2026-06-05, **dhrt-only**)
- [x] **String:** `LTRIM$`/`RTRIM$`, `REVERSE$`, `STARTSWITH`/`ENDSWITH`/
      `CONTAINS`, `BIN$`/`OCT$` (mit Vorzeichen), `ISNUMERIC`/`TRYVAL` (robustes
      Parsen via `parse_number`; `VAL` bleibt unverändert).
- [x] **Datei/Verzeichnis:** `DIRLIST` (sortiert), `DIREXISTS`, `MKDIR`
      (rekursiv), `DELETEFILE`, `RENAME`, `WRITEALL`, `READLINES`, `FILESIZE`,
      `PATHJOIN` (mit `/`). Alle pfadbasiert via `std::fs`.

## WP4 — Konsistenz / Aliase (✅ ERLEDIGT 2026-06-05, **dhrt-only**)
- [x] BASIC-Aliase: `SGN`→`SIGN`, `SQRT`→`SQR` (Match-Arm erweitert). `LTRIM$`/
      `RTRIM$`/`REVERSE$`/`STARTSWITH`/`ENDSWITH` existieren bereits als
      Primärnamen (WP3) — kein Alias nötig.
- [x] `AUDIO_SET_VOLUME`/`AUDIO_MUSIC_SET_VOLUME` als Alias (alte Namen bleiben).
- [x] Container-Methode `arr.join(trenner)` → `JOIN$` (`container_method` in vm.rs).
- [x] Doku-Korrekturen: CLAUDE.md (DRAWTILEMAP) + `vm.rs`-Kommentar (intish) waren
      schon in WP0 erledigt.
- [x] Nur dokumentiert (Rename zu riskant): `$`-Suffix nur Core, doppelte
      Sound-API, Suffix-`2`-Mehrdeutigkeit, `SPRITE_COLLIDE`/`SPRITE_COLLIDES`,
      `CAMERA_X`/`CAMERA3D_X` — Abschnitt „Aliase & Namenskonventionen" in
      `docs/builtins-core.md`.

---

**Empfohlene Reihenfolge:** WP0 → WP2 → WP1 → WP3 → WP4. **Alle WPs erledigt
(2026-06-05).** WP1–WP4 wurden dhrt-only umgesetzt (neue Builtins nur in der
nativen Runtime; Run-/Export-Pfad läuft über dhrts Rust-Frontend).

---

## Nachtrag „rund machen" (2026-06-06, dhrt-only)

Zweiter Lücken-Sweep gegen die echte Builtin-Liste; alles pur in
`rust/drachenhauch_runtime/src/builtins.rs` (+ Golden-Tests `tests/test_builtins_extra.py`,
`builtin_index.json`). Vorher fehlten u.a. `vec2`-Verwandte → siehe Modul `m3d`.

- [x] **Game-Math:** `WRAP(v,lo,hi)`, `PINGPONG(t,len)`, `MOVETOWARD(cur,ziel,maxd)`,
      `SMOOTHSTEP(e0,e1,x)`, `CLAMP01(v)`, `APPROX(a,b[,eps])`, `LOG10(x)`.
      (`CLAMP`/`LERP`/`REMAP`/`HYPOT`/`DEG`/`RAD` gab es schon.)
- [x] **Perlin-Noise** (deterministisch, ~[-1,1]): `NOISE(x)`, `NOISE2(x,y)`,
      `NOISE3(x,y,z)`, `FBM(x,y,octaves)`, `FBM3(x,y,z,octaves)` — prozedurale
      Generierung. (Ergänzt das texturbasierte `GENTEX_PERLIN`.)
- [x] **Laufzeit-Typen:** `TYPEOF(x)`, `ISNUM/ISINT/ISSTR/ISBOOL(x)`.
- [x] **Encoding/Hash:** `BASE64_ENCODE/DECODE`, `CRC32(s$)`, `HASH(s$)` (FNV-1a 64).
- [x] **Datei/OS:** NEU `COPYFILE`, `APPENDFILE`, `BASENAME`, `DIRNAME`. Die
      Verzeichnis-/Pfad-Basics gab es bereits aus WP3 (`DIRLIST`, `PATHJOIN`,
      `RENAME`, `MKDIR`, `DIREXISTS`, `READLINES`, `WRITEALL`, `FILESIZE`) — daher
      KEINE konkurrierenden `LISTDIR`/`JOINPATH`/`RENAMEFILE` einführen.

Abgedeckt durch Bestehendes (nicht neu): `ARRAY_FIND`→`ARRAY_INDEXOF`,
`ARRAY_CONTAINS`→`IN`, `ARRAY_MAP/FILTER`→Comprehensions, `ARRAY_SLICE`→`a[i:j]`,
`ASC`=`ORD`, `SLEEP`, `DELETEFILE`, `URL_ENCODE/DECODE` (Modul `html`),
`LTRIM$/RTRIM$/STARTSWITH/ENDSWITH/CONTAINS/REVERSE$/BIN$/OCT$`.

## Nachtrag 2 — PRINT-Trenner + Komfort-Aliase (2026-06-06)

- [x] **PRINT mit `,` und `;`**: `,` → Leerzeichen, `;` → kein Zwischenraum,
      abschließender Trenner → kein Newline. Sprach-Feature in BEIDEN Parsern
      (`parser.rs` + `parser.py` `print_items`), AST `Print{items, seps, newline}`
      (Parität gewahrt), Compiler emittiert `PRINT [count, newline, seps...]`, VM
      rendert mit Trennern. Vorher: nur `,` (immer Leerzeichen), trailing `;`
      wurde geparst aber ignoriert.
- [x] **String:** `COUNT(s$, teil$)` (nicht-überlappende Vorkommen), `TITLE$(s$)`.
- [x] **Random:** `WEIGHTED_CHOICE(werte, gewichte)` (Loot-Tabellen).
- [n] `RANDRANGE` NICHT nötig — `RANDINT(lo,hi)` (inkl.) + `RANDF(lo,hi)` decken es ab.

Tests: `tests/test_print_and_aliases.py`. Beleg, wie wichtig der Abgleich gegen
`builtins.rs` (nicht den eingefrorenen `builtin_index.json`) ist — Nachtrag 1 hatte
sonst beinahe `LISTDIR`/`JOINPATH`/`RENAMEFILE` als Dubletten zu `DIRLIST`/
`PATHJOIN`/`RENAME` eingeführt (korrigiert).
