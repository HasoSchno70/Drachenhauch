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

- [ ] **`PHYSICS_BROAD_*` nativ** (NEW/ADD/CLEAR/COUNT/QUERY/PAIR_A/PAIR_B/
      PAIR_COUNT). `DIM b AS PHYSICS_BROAD` kompiliert in gbrt (external type in
      `preprocess.rs:47`), aber `PHYSICS_BROAD_NEW()` crasht zur Laufzeit.
      → Uniform-Grid-Broadphase in Rust (Pendant zu `modules/physics.py`) oder
      external type entfernen (Fail beim Kompilieren statt still zur Laufzeit).
- [x] **`TIME$`/`DATE$` nativ** — in `builtins.rs` (`local_datetime()`: Windows
      `GetLocalTime`, sonst UTC-Fallback via civil-from-days). Format-Test in
      `test_gbrt_parity.py::test_time_date_format_tw_and_gbrt` (kein Exakt-
      Vergleich, da Wert variiert).
- [x] **`DRAWTILEMAP` nativ** — `graphics.rs::draw_tilemap` (jedes Tile via
      `draw_image_part`, Camera/Zoom korrekt) + Dispatch in `vm.rs`. Rendert jetzt
      tatsaechlich nativ (CLAUDE.md-Aussage damit korrekt). Manuell verifiziert
      (kein stdout fuer Parity).
- [ ] **`INKEY$`/`WAITKEY`/`SCROLL` + Core-`JOYSTICK_*` nativ** (oder sauber als
      Tree-Walker-only dokumentieren). Gamepad geht nativ nur via `IMPORT
      "input"`.
- [ ] **Float-Koordinaten angleichen.** TW lehnt `LINE(10.5,…)` ab
      (`_check_int`), gbrt akzeptiert + truncated. → TW-Zeichenprimitive
      (LINE/BOX/RECT/CIRCLE/PLOT/GRADIENT*) auf `_check_intish` umstellen
      (= dokumentierte „intish"-Konvention, matcht gbrt); falschen Kommentar in
      `vm.rs` korrigieren.

## WP1 — Array-Power (größte praktische Lücke)
- [ ] **Dynamische Arrays:** `ARRAY_PUSH`/`POP`/`INSERT`/`REMOVE_AT`/`REDIM`
      (heute fix dimensioniert; `_GBArray` + Rust `Value::Array` growable machen).
      Größerer Brocken — eigener Schritt.
- [ ] **Aggregate:** `ARRAY_SUM`/`ARRAY_AVG`/`ARRAY_MIN`/`ARRAY_MAX`/
      `ARRAY_FILL`/`ARRAY_COPY` (variadic MIN/MAX nehmen nur Skalare).
- [ ] **`SORT(arr, comparator)`** mit FUNCREF + Descending-Flag.

## WP2 — Spiel-Quickwins (billig, hoher Wert)
- [ ] **`MOUSEWHEEL` exponieren** — Backend sammelt es schon
      (`graphics.py:1380 pop_mouse_wheel`), nur kein Builtin. Nativ: raylib
      `GetMouseWheelMove`.
- [ ] **`SCREENWIDTH`/`SCREENHEIGHT`** (Setzen geht, Zurücklesen fehlt).
- [ ] **Ranged Random:** `RANDINT(lo,hi)`, `RANDF(lo,hi)`, `CHOICE(arr)`,
      `SHUFFLE(arr)` (PRNG ≠ Python → parity „erwartet unterschiedlich").
- [ ] **Farb-Helfer** (kein Farb-Modul): `RGBA`, `HSV`→RGB, `COLOR_LERP`,
      `RED`/`GREEN`/`BLUE`-Extraktion.
- [ ] **Math:** `ASIN`/`ACOS`, `HYPOT`, `DEG`/`RAD`, `LERP`, `REMAP`, `FRAC`;
      Konstanten `E`, `TAU`.
- [ ] **`ROUND(x, decimals)`** (heute nur ganzzahlig).

## WP3 — String + Datei
- [ ] **String:** `LTRIM$`/`RTRIM$`, `REVERSE$`, `STARTSWITH`/`ENDSWITH`/
      `CONTAINS`, `BIN$`/`OCT$`, robustes Parsen (`ISNUMERIC`/`TRYVAL`; `VAL`
      gibt still 0).
- [ ] **Datei/Verzeichnis:** `DIRLIST`, `DIREXISTS`, `MKDIR`, `DELETEFILE`,
      `RENAME`, `WRITEALL(path,text)`, `READLINES`, `FILESIZE`, `PATHJOIN`.

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
