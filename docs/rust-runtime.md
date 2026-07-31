# Native Rust-Runtime (raylib) — Migration

> **⚠️ Lesehinweis (Stufe B).** Dieses Dokument beschreibt die Migration *und*
> verweist vielfach auf die damalige Verifikation „bit-identisch zu den
> Python-Pfaden" (Tree-Walker / Python-VM / Cython-VM) und auf
> `interpreter.py`/`vm.py`/`serialize.py`. Diese Python-Pfade und -Dateien sind
> **alle entfernt** — `gbrt` ist heute die **einzige** Runtime und kompiliert den
> Quelltext selbst. Korrektheit sichern jetzt **run_gb-Golden-Tests** + Rust-
> `#[test]`s. Die „bit-identisch"-Stellen unten sind also historische
> Port-Verifikations-Notizen, kein aktueller Mehr-Pfad-Zustand.

Ein vierter Ausführungspfad neben Tree-Walker, Python-VM und Cython-VM: eine
**native Rust-Runtime**, die denselben Bytecode ausführt und (später) Grafik
über **raylib** rendert. Die Python-Toolchain (Lexer → Parser → Compiler)
bleibt unverändert — Rust übernimmt nur die Ausführung des kompilierten
`Module`s.

## Migrationsplan (inkrementell, nichts wegwerfen)

1. **Bytecode-Format einfrieren/serialisieren** — `.gbc` schreiben (Python) +
   lesen (Rust). ✅ *erledigt (Spike)*
2. **Rust-VM-Kern** — Dispatch-Schleife + Skalar-Ops + Control-Flow. Test:
   Konsolen-Programme, `stdout == Python-VM`. ✅ *erledigt (Spike)*
3. Strings/Arrays/Maps/Structs in Rust nachziehen (echte Rust-Typen statt
   geboxt — die Typ-Strenge von GB hilft). ✅ *erledigt*
4. raylib einbinden, Grafik-Builtins nach Rust. ✅ *erledigt (Core-2D)*
5. Module portieren (gui/ui/physics …). ✅ *erledigt — ALLE Module nativ inkl.
   ui (komplett, mit UI_TABLE) + gui (komplett, mit GUI_TABLE + Callbacks)*
6. 3D-Builtins auf raylibs Mesh/Kamera-API. ✅ *Core-Primitive erledigt (`g3d`)*
7. Editor: „Export → native Exe bundeln". ✅ *erledigt (Bytecode + Assets in
   eine standalone `.exe` gebündelt — siehe unten)*

**Dev-Run-Loop** (quer zu den Schritten): `gbrun.py --native <datei.gb>` —
ein Befehl kompiliert (Python) → `.gbc` → startet `gbrt`. ✅ *erledigt* (siehe
unten).

## Offen / nächste Schritte (Stand 2026-06-01)

Schritte 1–6 fertig; zusätzlich nativ: **Audio inkl. echter FFT** (`AUDIO_FFT`),
**Game-Loop** (`DELTA`/`FPS`/`SETFPS`/`SET_FULLSCREEN`/`SETWINDOWTITLE`/
`SAVESCREENSHOT`), **Shader/Post-Processing** (`SHADER_LOAD`/`SET`/`SET2`/`SET3`/
`POSTFX`, CRT/Bloom/Vignette), **TTF-Fonts** (`LOADFONT`/`SETFONT`/
`TEXT_SPACING`). Die native Runtime deckt damit ein komplettes
2D/3D-Spiel mit Sound, Menüs/Tabellen und GPU-Effekten ab.

**Lohnende nächste Hebel (raylib bietet noch mehr):**
- **3D-Stack vollständig:** Modelle, GenMesh inkl. Heightmap, Texturen,
  **Normal-Maps**, **PBR + analytisches IBL** (`LIGHT_ENV`) **+ echtes
  HDR-Cubemap-IBL** (`LIGHT_ENV_HDR`), Billboards, Ray-Kollision/Picking,
  Beleuchtung, **Fog**, **Schatten**, **Kamera-Modi** — siehe unten.
- 2D-Politur **erledigt (2026-06-04):** 2D-Extras (`LINEW`/`BOXROUND`/`RECTROUND`/
  `GRADIENTV`/`GRADIENTH`/`SPLINE`, dual-path), Blend-Modes (`BLEND_MODE`, nativ),
  prozedurale Texturen (`GENTEX_PERLIN`/`GRADIENT`/`CHECKED`/`COLOR`, nativ),
  Clipboard + Drag&Drop (`CLIPBOARD_GET/SET`, `FILES_DROPPED`/`FILE_DROPPED`,
  nativ), **Render-Targets** (`RENDERTARGET_NEW`/`BEGIN`/`END`/`DRAW`, dual-path —
  gbrt: eigener Command-Buffer pro Target, beim FLIP vor der Hauptszene auf die
  RenderTexture gerendert). Sound-Pan existiert bereits (`AUDIO_PAN`). Demos
  `examples/100_2d_extras.gb`, `101_blend_gentex.gb`, `102_render_target.gb`.
- **Noch offen (klein):** Sound-Aliase (`LoadSoundAlias`, ueberlappende Wiedergabe
  desselben Sounds); Render-Target-Trails (aktuell pro Frame transparent gecleart).

**Voll-Native-Portierung (laufend):** Ziel ist, ALLE Module nativ zu machen,
sodass nur die Editoren Python brauchen. Schwere/optionale Module sind dabei
feature-gated (Standard-`.exe` bleibt schlank).

- **Phase 1 — ERLEDIGT (Spiel-Logik, always-on):** `regex` (Crate `regex`,
  Pattern-Cache, `\1`→`${1}`-Replacement-Übersetzung), `tiled` (TILED_*, 36 —
  JSON-Loader via serde_json, Properties, Objekte, Bulk-Ops inkl. Flood-Fill),
  `tile_collide` (TILE_*, 4 — Box-Sweep-Port), `controller` (CHAR_*, 24 —
  Platformer-Physik). Alle bit-identisch zu den Python-Pfaden verifiziert
  (`tiled`/`tile_collide`/`controller` gegen `examples/levels/level1.json`,
  `examples/77_tiled_platformer.gb` rendert nativ).
- **Phase 2 — ERLEDIGT (erweitertes `audio`, `graphics`-Gate):** AUDIO_* (27 —
  Mixer-Lifecycle, Channel-Playback mit Volume/Pan/Pause/Resume/Stop,
  Music-Streaming mit Volume/Position/Queue, Ton-Generierung AUDIO_TONE/
  AUDIO_NOISE via In-RAM-WAV → `new_wave_from_memory`/`new_sound_from_wave`).
  Funktional (Audio gehoert nicht zur bit-identischen Garantie). SOUND/
  AUDIO_CHANNEL = INTEGER-Handles; raylib hat keine eigenstaendigen Mixer-Channels, daher
  steuert ein „Channel" die Wiedergabe genau seines Sounds (Volume per Handle
  getrackt, da raylib keinen Getter hat). Fade/loops=N werden vereinfacht
  (raylib kann das nicht direkt).

### Audio-Backend: Kira (cpal) — loeste 2026-06-13 raylib-Audio ab

raylib-Audio hatte eine strukturelle Schwaeche: **Musik-Streaming wurde per
`UpdateMusicStream` aus dem Game-Loop (FLIP) nachgefuellt** — bei schweren
Frames konnte der Puffer unterlaufen → Knacken.

Das Audio-Backend laeuft daher auf **[Kira](https://crates.io/crates/kira)**
(cpal) in [`src/audio.rs`](../rust/gb_runtime/src/audio.rs): eigener Audio-
Thread, vollstaendig vom Game-Loop entkoppelt (kein Stottern), native Tweens
fuer Fades/Pan, FFT-Tap als Effect am Main-Track (ersetzt raylibs
`AttachAudioMixedProcessor`), MOD/XM via reinem Rust-Player (`xmrs`/
`xmrsplayer` — kein C). Eingebunden mit `--features graphics` (raylib bleibt
fuer Fenster/GL/Input). Volume ist bei Kira in Dezibel (`db()`-Helfer), Pan
−1..1; `vm.rs` ruft die Audio-API unveraendert.

**Tracker-Module** werden in **Echtzeit gestreamt**: ein Kira-Custom-`Sound`
(`ModuleSound`) pollt den reinen Rust-Player `xmrs` pro Audio-Block auf dem
Audio-Thread. Sofort geladen (kein Vorab-Render), exaktes Endlos-Loopen (der
Player zaehlt die Loops via `set_max_loop_count`), wenig RAM. Steuerung
(Volume/Fade/Pitch/Pause/Stop/Position) ueber `Arc<ModShared>`-Atomics; ein
Pitch-Resampler (fraktionale Leseposition + lineare Interpolation) und eine
Volume-Ramp (klickfreie Fades) sitzen im Sound. Das Modul wird beim Start
geleakt (`Box::into_raw` → `'static`-Borrow fuer den Player) und vom Sound im
`Drop` wieder freigegeben (Player zuerst fallen lassen, dann `Box::from_raw`).
Stream-Formate (ogg/mp3/wav/flac) streamen via Kira von Platte.

Naheliegende Ausbauten dank Kira (Mixer-Tracks/Effekte): **Busse**
(SFX-/Musik-Master getrennt) und **Echtzeit-Effekte** (Filter/Reverb/Delay als
neue Builtins, z.B. fuer SID-Charakter ohne Buffer-Bake).
- **Phase 3 — ERLEDIGT (Daten/Netz, feature-gated):**
  - `db` (DB_*, 17, Feature `db` → `rusqlite` bundled): SQLite, `?`-Binding,
    DB_QUERY laedt Zeilen eager (vermeidet self-referenzielle Cursor). DB_CONN/
    DB_RESULT = INTEGER-Handles. **Bit-identisch** verifiziert (CRUD, rowid,
    typed Getter, rowcount).
  - `net` (NET_*, 19, Feature `net`, nur stdlib `std::net`): TCP-Listener/
    -Sockets + UDP, non-blocking default, INTEGER-Handles. Loopback verifiziert.
  - `html` (HTTP_*/HTML_*/URL_*, 10, Feature `http` → `ureq`+rustls): HTTP
    GET/POST/DOWNLOAD (https/TLS), URL-Encode/Decode, HTML-Text/Tag-Find/Attr
    als handgeschriebener Scanner. HTTP_GET verifiziert (Status 200, Body), HTML-
    Parsing bit-identisch (ausser Non-ASCII-Konsolen-Encoding -- wie CRLF ein
    OS-Artefakt).
  - **Cargo-Features:** `db`/`net`/`http` + Aggregat `full`. `build_runtime.py`
    baut Standard-Dev `graphics db net http` (`--no-data` laesst sie weg).
    Wenn ein Feature fehlt, liefert der Builtin den „nicht verfuegbar"-Fehler.
- **Phase 4 — ERLEDIGT (Hardware/IoT, feature-gated):**
  - `serial` (SERIAL_*, 10, Feature `serial` → `serialport`): RS-232/USB-COM.
    Ports/Open/Read/Write/Readline/Available/Flush/Timeout. Nativ COM1 erkannt.
  - `usb` (USB_*, 9, Feature `usb` → `hidapi`): HID. List/Open/Open_Path/Read/
    Write/Product/Manufacturer/Serial. latin-1-Byte<->STRING. Geraete gelistet.
  - `wifi` (WIFI_*, 8, Feature `wifi`, Windows via `netsh wlan`, Linux via
    `nmcli`, macOS via `networksetup`/`airport`): Available/Current/Signal/
    Scan/Connect/Disconnect/Profiles/Delete. Windows-Zweig verifiziert;
    Linux/macOS neu (2026-07, Cross-Platform-Migration Phase 3) und NICHT
    auf echter Hardware getestet, siehe docs/module-wifi.md.
  - `bt` (BT_*, 8, Feature `bt` → `btleplug`+`tokio`): BLE, async->sync ueber
    globale tokio-Runtime (block_on pro Aufruf). Scan/Connect/Services/
    Characteristics/Read/Write, latin-1. **BT_SCAN fand reale BLE-Geraete** (mit
    RSSI). Adresse->Peripheral aus letztem Scan fuer BT_CONNECT.
  - Handles = INTEGER-Index in cfg-gated VM-Vecs. `build_runtime.py --hardware`
    nimmt sie dazu. Default-Dev-Build laesst Hardware weg (haelt die schweren
    Deps tokio/btleplug/windows aus dem Normal-Build).
  - **Frueh-Warnung beim IMPORT:** Importiert ein Programm ein Hardware-Modul,
    das dem aktuellen Build fehlt, warnt gbrt schon beim IMPORT — `gbrt run`
    auf stderr vor dem Lauf, `gbrt --check` als `severity:"warning"` auf der
    IMPORT-Zeile (Editor-Marker). Der Laufzeitfehler beim ersten Aufruf bleibt
    zusaetzlich (`vm.rs::unknown_builtin_msg`). Logik in `preprocess.rs`
    (`missing_hardware_modules` / `missing_hardware_imports_with_lines`).

**Voll-Native-Portierung KOMPLETT (2026-06-03):** alle 12 zuvor Python-only-
Module laufen jetzt nativ in gbrt. Nur die Editoren brauchen noch Python.

### Echtes HDR-Cubemap-IBL (`LIGHT_ENV_HDR`) — erledigt

Ersetzt die analytische `LIGHT_ENV`-Näherung durch echte Environment-Maps aus
einem `.hdr`. Port von raylibs `shaders_basic_pbr` / learnopengl-IBL, rein über
raylib-rs `ffi`/`rlgl` (wie das Shadow-Mapping). Builtin
**`LIGHT_ENV_HDR(pfad$ [, intensität])`** (native-only). Pipeline einmalig in
[`graphics.rs`](../rust/gb_runtime/src/graphics.rs) (`light_env_hdr()`):

1. **`.hdr` laden** — eigener Radiance-RGBE-Dekoder (`load_hdr_rgbe`) → RGBA32F
   2D-Float-Textur via `rlLoadTexture`. (Nötig, weil raylib-sys **ohne**
   `SUPPORT_FILEFORMAT_HDR` gebaut ist — `LoadImage("*.hdr")` schlägt fehl.)
2. **equirect → Cubemap** (512², `ibl_render_cube` + `EQUIRECT_FS`, 6 Faces über
   `rlLoadDrawCube`).
3. **Irradiance-Cubemap** (32², `IRRADIANCE_FS`-Convolution) — diffuse IBL.
4. **Prefilter-Cubemap** (128² + Roughness-Mips, `PREFILTER_FS` GGX-Importance) —
   specular IBL.
5. **BRDF-LUT** (512², 2D, `BRDF_FS` + `rlLoadDrawQuad`) — einmalig.
6. **PBR-`LIGHT_FS`** um `samplerCube irradianceMap/prefilterMap` +
   `sampler2D brdfLUT` + **`useIBLMaps`-Gate** erweitert. Gesetzt → echte Maps
   (`texture(irradianceMap,N)`, `textureLod(prefilterMap,R,rough·4)`,
   `texture(brdfLUT,vec2(NoV,rough))`); sonst der **bestehende analytische
   `LIGHT_ENV`-Pfad** (Default `useIBLMaps=0` → kein `.hdr` nötig, alle Demos
   unverändert). `envIntensity` bleibt das gemeinsame An/Aus-Gate.

Maps liegen als GL-Texturen in `Graphics` (`ibl_irradiance/_prefilter/_brdf`),
werden in `render_scene` im Draw-Kontext an die Slots 11/12/13 gebunden (Cubemaps
via `rlEnableTextureCubemap`, BRDF-LUT 2D via `rlEnableTexture`; Material-Maps
nutzen 0..2, Shadow 10 → kein Clash). Dispatch `"light_env_hdr"` in
[`vm.rs`](../rust/gb_runtime/src/vm.rs), native-only-Stub in
[`g3d.py`](../gamebasic/modules/g3d.py).

**Drei rlgl-Stolpersteine** (für Nachbauten):
- `rlFramebufferAttach` endet mit `glBindFramebuffer(0)` → **nach jedem Attach das
  FBO neu `rlEnableFramebuffer`** (sonst landet der Cube auf dem Screen, Cubemap
  bleibt schwarz).
- Leere **Float-Cubemaps** (R32/R16) werden von `rlLoadTextureCubemap` abgelehnt →
  **R8G8B8A8** (LDR; sehr helle Werte clampen, für Reflexionen ausreichend).
- Prefilter-Cubemap braucht den **vollen Mip-Chain** (128→1 = 8 Level), sonst ist
  sie mit `LINEAR_MIPMAP_LINEAR` nicht *mipmap-complete* und sampelt komplett
  schwarz; prefiltert werden nur die ersten 5 Roughness-Level
  (`MAX_REFLECTION_LOD=4`).

**Skybox** `SKYBOX(an)` zeichnet die env-Cubemap zusätzlich als sichtbaren 3D-
Hintergrund (eigener `SKYBOX_VS/FS`; `mat3(matView)` entfernt die Translation →
kamerazentriert/unendlich; als erstes im 3D-Pass mit `rlDisableDepthMask` +
`rlDisableBackfaceCulling`, Modelle zeichnen darüber). Die env-Cubemap wird dafür
nach der IBL-Generierung aufbewahrt (`ibl_env`). Ohne `LIGHT_ENV_HDR` ein No-Op.

Demo [examples/99_ibl_hdr.gb](../examples/99_ibl_hdr.gb): Reihe Chrom-Metallkugeln
(`MODEL_PBR` metalness 1, Roughness-Verlauf) spiegeln das HDRI **vor der als
Skybox sichtbaren Umgebung**; `FILEEXISTS`-Guard fällt ohne `.hdr` auf
analytisches `LIGHT_ENV` zurück. **Asset:**
`py examples/assets/download_hdri.py` holt ein CC0-1k-HDRI (Poly Haven,
kloofendal_43d_clear) als `examples/assets/ibl_env.hdr` (gitignored). Per
Screenshot verifiziert (Spiegel→diffus über die Roughness-Reihe). Bit-Identität
entfällt (GPU/3D); analytisches `96_ibl` bleibt unverändert.

## Dev-Run-Loop: `gbrun.py --native`

Für schnelles Iterieren beim Coden gibt es einen One-Command-Pfad:

```
.venv\Scripts\python.exe gbrun.py --native examples\30_shapes.gb
```

Das kompiliert die `.gb`-Datei (Lexer/Parser/Compiler bleiben in Python),
serialisiert sie in eine **temporäre `.gbc`** und startet `gbrt` im Verzeichnis
der Quelldatei (damit relative Asset-Pfade wie `LOADIMAGE("assets/…")`
stimmen). stdout/stderr und ein etwaiges Grafik-Fenster werden direkt
durchgereicht; der Exit-Code von `gbrt` wird weitergegeben. Fehlt das Binary,
verweist die Meldung auf `rust\build_runtime.py`.

So bleibt Python die Toolchain (und später nur noch der Editor), während die
Ausführung nativ läuft — kein manuelles `serialize` + `gbrt` mehr.

**Im Editor:** Der **Run**-Button (F5) ist der einzige Run-Knopf und nutzt
**primär `gbrt`** (fällt bei nicht gebauter Runtime / Compile- oder Start-Fehler
automatisch auf den Tree-Walker zurück). Der Editor kompiliert die Datei
in-process in eine temporäre `.gbc` und startet `gbrt` **direkt** als `QProcess`
(nicht über `gbrun.py`) — so beendet der `Stop`-Button auch den nativen Prozess
(kein verwaister gbrt). Output und Laufzeitfehler
(`datei.gb:Zeile`, klickbar) landen in derselben Konsole wie der Python-Run.

### Laufzeitfehler mit Zeilennummer

Der Compiler stempelt pro Bytecode-Instruktion die **Quell-Zeile** (`stmt.line`
vom Parser) in ein zu `code` paralleles `lines`-Array (`CompiledFunction.lines`,
serialisiert als `"lines"` in der `.gbc`). Die Rust-VM merkt sich die Zeile der
zuletzt ausgeführten Instruktion (`Vm.cur_line`); bei einem propagierenden
Fehler bleibt die **innerste** fehlschlagende Zeile stehen. `gbrun.py --native`
reicht den Quell-Dateinamen als 2. Arg an `gbrt` durch, sodass die Meldung lautet:

```
Laufzeitfehler in spiel.gb:42: Index 10 ausserhalb [0..2] in Dimension 0
```

Die Python/Cython-VMs ignorieren `lines` (additives Feld, kein Recompile nötig);
`gbrt <datei.gbc>` ohne Label nutzt den `.gbc`-Pfad. Zeile `0` (untracked) →
Meldung ohne Zeilenangabe.

**Compile-Fehler** (vor der Ausführung, in Python) tragen ebenfalls eine Zeile:
Parser-Fehler ohnehin, und `CompileError` wird zentral mit der Statement- bzw.
Deklarations-Zeile angereichert (`Compiler._at` + `_stmt`, via
`GameBasicError.set_line`). So zeigt der `--native`/F6-Pfad z. B.
`[Zeile 4] CompileError: SUB 'foo' bereits deklariert` — im Editor als
klickbarer Link in die Quelldatei.

## Schritt 1: `.gbc`-Serialisierung

[`gamebasic/serialize.py`](../gamebasic/serialize.py) wandelt ein vom
`Compiler` erzeugtes `Module` ([bytecode.py](../gamebasic/bytecode.py)) in eine
selbstbeschreibende JSON-Datei. JSON ist für den Spike bewusst gewählt
(debuggbar); ein kompaktes Binärformat ist später drop-in möglich.

**CLI:**
```
.venv\Scripts\python.exe -m gamebasic.serialize [--pretty] <datei.gb> [out.gbc]
```

> **Wichtig:** `serialize.py` importiert `interpreter.py`, bevor es kompiliert.
> Der Compiler entscheidet anhand der `BUILTINS`-Registry (von den `@builtin`-
> Decorators in `interpreter.py` gefüllt), ob ein Identifier-Call wie `LEN(a)`
> zu `CALL_BUILTIN` wird. Ohne diesen Import sähe der Compiler die Registry
> nicht und erzeugte für `LEN`/`INT`/… kaputten `LOAD_NAME + CALL_VALUE`-
> Bytecode. `gbrun.py` importiert `interpreter` am Modulanfang — dieselbe
> Umgebung wird hier hergestellt.

**Wert-Encoding** (eindeutig dekodierbar — INT/FLOAT/BOOL müssen unterscheidbar
bleiben, da `bool` in Python ein `int`-Subtyp ist und `1` ≠ `1.0` in GB):

| GB-Wert | JSON |
|---|---|
| `None` | `null` |
| `bool` | `{"b": true}` |
| `int` | `123` (Plain-Zahl) |
| `float` | `{"f": 1.5}` |
| `str` | `"text"` |
| `tuple`/`list` | `[…]` |
| `COMP_MARKER` | `{"comp": true}` |
| `_FuncRef` | `{"funcref": "name"}` |

Code-Instruktionen: `[op:int, arg]`, `arg` mit demselben Encoding. Die Rust-VM
weiß pro Opcode, welche Struktur `arg` hat (Index, Slot, Name, Tupel).

**Noch nicht serialisierbar:** Laufzeit-Handles im const-Pool
(IMAGE/SOUND/MAP/ARRAY/Instanzen). Diese kommen mit Schritt 3.

## Schritt 2: Rust-VM-Kern

Standalone-Crate [`rust/gb_runtime/`](../rust/gb_runtime) (getrennt vom
PyO3-Helper-Crate `rust/gb_native/`). Binary `gbrt`.

```
cd rust/gb_runtime
cargo build --release
target/release/gbrt <datei.gbc>
```

**Implementiert:** LOAD_CONST/POP/DUP, Locals (LOAD/STORE/DECLARE), Global-Slots
(LOAD/STORE/DECLARE/DECLARE_CONST), Name-Globals (Fallback), volle Skalar-
Arithmetik (ADD/SUB/MUL/DIV/MOD/POW/NEG/INT_DIV) inkl. der spezialisierten
`_NN`-Opcodes, Vergleiche, Bitwise, Control-Flow (JUMP/JUMP_IF_*), User-Calls
(CALL_USER/RETURN/RETURN_VOID), PRINT, HALT.

**Bit-Identitäts-kritische Semantik** (1:1 aus `gamebasic/vm.py`):
- `_fmt`: `NIL`/`TRUE`/`FALSE`; float-ganzzahlig → `{:.1f}` (immer Fixpunkt,
  auch `1e16` → `10000000000000000.0`), nicht-ganzzahlig → Python-`repr` mit
  E-Notation-Schwelle (`decpt <= -4` oder `> 16`), nachgebaut über Rusts `{:e}`.
  Bit-identisch verifiziert für kleine (`1e-09`), normale und große Beträge.
- DIV: int/int mit Rest 0 → int, sonst float; `b==0` wirft.
- INT_DIV (`\`): Trunkierung Richtung 0 (= Rust `i64`-Division).
- MOD: Ergebnis-Vorzeichen = Vorzeichen des Divisors (Python-Semantik).
- integer-Coercion: float-ohne-Nachkomma OK, `bool` wirft.

**Seit Schritt 3 zusätzlich:** Name-Globals (`DECLARE_NAME`/`DECLARE_CONST`),
Tupel (`BUILD_TUPLE`/`UNPACK_TUPLE`/`BUILD_TUPLE_DYN`), `IN`, Slicing, Arrays
(multidim, `LOAD/STORE_INDEX`, `DECLARE_ARRAY_*`), Maps, String-Index, OOP
(`NEW_INSTANCE`, Felder, Member, Methoden, `LOAD_SELF`, Properties, Operator-
Overloading, Structs), FUNCREF (`LOAD_FUNCREF`/`CALL_VALUE`), Container-Methoden
(`"x".upper()`), DATA/READ (`PUSH_DATA`/`RESET_DATA_PTR`), `TRY`/`THROW`/`CATCH`,
und eine **Registry pure/deterministischer Builtins** (`builtins.rs`): `STR$`,
`VAL`, `INT`, `ABS`, `LEN`, `CHR$`/`ASC`, `SQR`, `SIN`/`COS`/`TAN`/`ATAN`/
`ATAN2`, `FLOOR`/`CEIL`/`ROUND` (Banker's), `LOG`/`EXP`/`POW`, `MIN`/`MAX`/
`CLAMP`/`SIGN`, `UPPER$`/`LOWER$`, `LEFT$`/`RIGHT$`/`MID$`/`INSTR`/`REPLACE$`/
`TRIM$`/`SPLIT$`/`JOIN$`, `PADL$`/`PADR$`/`SPACE$`/`REPEAT$`/`HEX$`/`FORMAT$`,
`RGB`, `MAP*`, `SORT`/`REVERSE`/`ARRAY_INDEXOF`, `RANGE`, Comprehension-Helfer.

**Seit Schritt 4 zusätzlich:** `RND`/`RANDOMIZE`/`MILLIS`/`TIMER` (PRNG bzw.
SystemTime — NICHT bit-identisch, per Definition).

**Noch nicht im Kern:** Datei-I/O, ENUM/STATIC-Namespaces (deren const-Pool-
Handles noch nicht serialisierbar sind), Module (`IMPORT` → Schritt 5),
Bulk-Draws (`PLOTS`/`BOXES`/…), Layer/Atlas. *(Diese sind inzwischen alle
implementiert — siehe die jeweiligen Schritte unten.)*

**Coroutines/`YIELD` — nativ unterstützt (Frame-Snapshot):** Die Python-VMs
treiben Coroutinen thread-basiert; die native Rust-VM nutzt stattdessen einen
**Frame-Snapshot** (keine OS-Threads → raylib-Main-Thread bleibt sicher,
deterministisch per Konstruktion). `dispatch` liefert `Step::Return | Yield`;
bei `YIELD_VALUE` (Opcode 115) wird der Frame (`ip`/`locals`/`stack`/
`try_handlers`) in einem `Value::Coroutine` (`CoroState`) abgelegt und beim
`CORO_RESUME`/`SEND` restauriert. Der `CoroState` hält einen rohen `*const Func`
auf die — über die ganze Programmlaufzeit unveränderliche — `Func`, sodass
`Value` keinen Lifetime-Parameter braucht. Möglich ist die Single-Frame-Lösung,
weil GameBasic **kein Cross-Frame-`YIELD`** erlaubt (ein Helfer mit `YIELD` ist
selbst eine Coroutine): nur der oberste Coroutine-Frame muss fortsetzbar sein,
verschachtelte normale Calls laufen weiter rekursiv auf dem nativen Stack.
`CORO_*`-Builtins laufen über `try_coro` (brauchen VM-State); `FOR EACH`/
Comprehension über eine Coroutine drainen via `__comp_iter`. Verifiziert
bit-identisch zu allen drei Python-Pfaden inkl. Standalone-`.exe`
([examples/98_coroutines.gb](../examples/98_coroutines.gb)).

### Validierung

`stdout` der Rust-VM ist bit-identisch zur Python-VM (modulo OS-Newline:
Python schreibt auf Windows `\r\n`, Rust `\n` — semantisch identisch).
Verifiziert per Vollsweep: **30 Beispiele bit-identisch** (inkl. OOP, Arrays,
Maps, Tupel, Strings, alle Benchmarks), 0 echte Mismatches.

## Schritt 4: raylib-Grafik

Grafik ist **feature-gated** (`graphics`, default aus): der pure VM-Kern baut
ohne C-Toolchain. Mit Grafik wird [`raylib`](https://crates.io/crates/raylib)
(raylib-rs 6.0, [raylib-rs/raylib-rs](https://github.com/raylib-rs/raylib-rs))
eingebunden.

**Bit-Identität gilt NICHT für Pixel** (Grafik rendert nur die native Runtime) —
nur `PRINT`/stdout bleibt bit-identisch. Grafik wird per Screenshot verifiziert.

### Build (mit Grafik)

raylib kompiliert seine C-Quellen via **cmake** und braucht **libclang** für
die FFI-Bindings (bindgen). Der Helfer setzt die Umgebung:

```
.venv\Scripts\python.exe rust\build_runtime.py            # release, mit Grafik
.venv\Scripts\python.exe rust\build_runtime.py --no-graphics
```

Voraussetzungen (Windows): VS C++ Build Tools (liefern `cl.exe` + gebündeltes
cmake), LLVM für `libclang.dll` (`winget install LLVM.LLVM`). `cl.exe` findet die
`cc`-Crate automatisch; cmake-PATH und `LIBCLANG_PATH` setzt `build_runtime.py`.

**Cross-Platform (Linux/macOS): experimentell, noch nicht auf echter Hardware
verifiziert** — Entwicklung/CI laufen bisher ausschließlich unter Windows.
`build_runtime.py` erkennt seit Kurzem das Betriebssystem und sucht auf
Linux/macOS `cmake`/`clang` nur über `PATH` (Paketmanager-Installation
vorausgesetzt: `apt install cmake clang libclang-dev` bzw.
`brew install cmake llvm`), statt wie unter Windows feste Pfade abzusuchen.
Das `wifi`-Feature bleibt vorerst Windows-only (`netsh`-basiert) — auf
anderen Systemen baut es zwar mit, `WIFI_*`-Builtins scheitern aber zur
Laufzeit mit einer klaren Fehlermeldung.

### Builtins ([`graphics.rs`](../rust/gb_runtime/src/graphics.rs))

`SCREEN`, `CLS`, `FLIP`, `PLOT`, `LINE`, `BOX` (gefüllt), `RECT` (Umriss),
`CIRCLE`, `TRIANGLE`/`TRIANGLEOUTLINE`, `ELLIPSE`/`ELLIPSEOUTLINE`, `ARC`,
`POLYGON`/`POLYGONOUTLINE`, `TEXT`/`TEXT_SIZE`/`TEXT_WIDTH`/`TEXT_HEIGHT`,
`TEXT_BOLD`/`TEXT_ITALIC` (No-Op — Default-Font), `LOADIMAGE`/`DRAWIMAGE`/
`IMAGEWIDTH`/`IMAGEHEIGHT`, `KEYPRESSED`, `MOUSEX`/`MOUSEY`/`MOUSEBUTTON`,
`QUITREQUESTED`, `SLEEP`.

**Game-Loop-Grundlagen** (nativ in gbrt/raylib): `DELTA()`
(Sekunden seit letztem FLIP, framerate-unabhaengige Bewegung), `FPS()`,
`SETFPS(n)` (Ziel-Framerate, 0 = ungedrosselt), `SET_FULLSCREEN(an)` (nativ
echtes `ToggleFullscreen`, nicht mehr No-Op), `SETWINDOWTITLE(s)`,
`SAVESCREENSHOT(pfad)`. Nativ ueber raylibs `GetFrameTime`/`GetFPS`/
`SetTargetFPS`/`ToggleFullscreen`/`SetWindowTitle`/`TakeScreenshot`.

**Bulk-Draws:** `PLOTS`/`BOXES`/`CIRCLES`/`LINES` (Koordinaten-Arrays + Farbe als
INT oder ARRAY). **Bilder erweitert:** `DRAWIMAGEPART`, `DRAWIMAGEFLIPPED`,
`LOAD_ASSETS` (Manifest-Vorladen + Alias/Pfad-Cache, `LOADIMAGE("alias")` trifft).
**Z-Layer:** `LAYER_DEFINE`/`LAYER`/`LAYER_END`/`LAYER_CLEAR` — FLIP komponiert
alle Layer aufsteigend nach z. **Sprite-Atlas:** `ATLAS_LOAD` (JSON-Manifest:
`{"image":..., "sprites":{name:[x,y,w,h]}}`), `ATLAS_DRAW`/`ATLAS_DRAW_FLIPPED`,
`BATCH_DRAW`/`BATCH_FLUSH` (im Recording-Modell flusht alles beim FLIP).

**Modell:** Draw-Builtins hängen `Cmd`s an eine Liste; `CLS` leert sie + merkt
die Clear-Farbe; `FLIP` rendert alle Cmds in einem `begin_drawing`-Block und
präsentiert. So muss kein raylib-Draw-Handle über Builtin-Aufrufe gehalten
werden. Farben sind `&HRRGGBB`-INTEGER → raylib `Color`.

**Vordefinierte Globals:** Farben (`BLACK`/`WHITE`/`RED`/…), Tasten (`KEY_*` als
SDL2-Keycodes, in `KEYPRESSED` auf raylib-Keys gemappt) und `PI` werden
mit Python-identischen Werten vorregistriert (`register_default_globals`).

### Headless-Verifizierung

`gbrt` rendert headless und schreibt einen Screenshot, gesteuert per ENV:

```
GBRT_FRAMES=3 GBRT_SCREENSHOT=out.png gbrt programm.gbc
```

Nach `GBRT_FRAMES` Frames liefert `QUITREQUESTED()` `true` (Loop endet sauber);
beim Erreichen der Grenze wird das PNG gespeichert (auch wenn das Programm eine
feste `FOR`-Schleife statt `QUITREQUESTED` nutzt). **Hinweis:** raylibs
`TakeScreenshot` legt die Datei relativ zum Arbeitsverzeichnis ab.

Verifiziert (visuell per Screenshot) an allen 7 IMPORT-freien Grafik-Beispielen:
`30_shapes` (alle Formen), `44_language_showcase`, `34_schneefall`,
`39_textscroll`, `40_parallax`, `75_preloader` (LOAD_ASSETS + Alias-Cache),
`76_layers_atlas` (ATLAS_LOAD + BATCH_DRAW + Z-Layer-Compositing).

## Schritt 5: Module (IMPORT)

Modul-Builtins erreichen den Compiler automatisch: `IMPORT "x"` lädt im
Preprocessor das Python-Modul `gamebasic.modules.x`, dessen `@builtin`-
Decorators die `BUILTINS`-Registry füllen → der Compiler emittiert
`CALL_BUILTIN`. Die Rust-VM muss nur die jeweiligen Builtins implementieren —
**keine Serializer-Änderung nötig**.

**Portiert (pur, bit-identisch verifiziert):**
- `vec2` — `Value::Vec2(f64,f64)` (immutabler Wert-Typ) + Operator-Overloading
  (`+`/`-`/`*`/`/` via `module_op`, vor User-Operatoren). `VEC2_NEW/ZERO/X/Y/
  LENGTH/LENGTH_SQ/NORMALIZE/DOT/CROSS/DISTANCE/LERP/PERP/REFLECT/ANGLE/FROM_ANGLE`.
- `curves` — `CURVE_LERP/SMOOTHSTEP/SMOOTHERSTEP/BEZIER/BEZIER2/CATMULL/CATMULL2/HERMITE`.
- `physics` (pure) — `PHYSICS_BOX_BOX/CIRCLE_CIRCLE/BOX_CIRCLE/POINT_BOX/POINT_CIRCLE/
  DISTANCE/DISTANCE2/LENGTH/NORM_X/NORM_Y/REFLECT_X/REFLECT_Y/RAY_BOX/RAY_CIRCLE`.
  (Broadphase mit externem Typ noch nicht.)
- `input` — `INPUT_BIND/UNBIND/RESET/UPDATE/HELD/PRESSED/RELEASED/AXIS/BOUND`.
  Edge-Detection über prev/cur-Snapshots; Tastenstatus via raylib (`INPUT_UPDATE`
  ohne Fenster = keine Tasten → Konsolen-Demos bit-identisch). **Gamepad** ist
  nun nativ: die `JOY_BUTTON_*`/`JOY_DPAD_*`-
  Bind-Codes (negativ) sind als Globals registriert und `key_down(negativer Code)`
  pollt über `IsGamepadButtonDown` **alle** verbundenen Pads (wie Pythons
  `_poll_joysticks_into`) — `INPUT_BIND("jump", JOY_BUTTON_A)` + `INPUT_HELD/
  PRESSED` funktionieren. `INPUT_JOY_COUNT` (zusammenhängend ab Slot 0),
  `INPUT_JOY_NAME(idx)`, `INPUT_JOY_AXIS(pad, "left_x"|…|"rt")` (Deadzone 0.15
  für Sticks) über raylibs Gamepad-API. Achsen-Namen → `GamepadAxis`-Indizes
  (Xbox-Layout). Ohne Pad: Count 0, Achsen 0.0, Buttons false (kein Crash).
- `camera` — `CAMERA_SET/RESET/X/Y/ZOOM/FOLLOW/S2W_X/S2W_Y`. World→Screen-Transform
  (`w2s`/`ssize`) wird in allen Draw-Methoden angewandt; TEXT-Position transformiert,
  Font-Größe bleibt. 141_camera_visual rendert korrekt.
- `sprite` — `Value::Sprite` (Sheet-Animation). `SPRITE_NEW/SET_POS/SET_VELOCITY/
  GET_X/Y/WIDTH/HEIGHT/SET_FLIP/SET_SCALE/TINT/TINT_CLEAR/ADD_ANIM/PLAY/PLAY_ONCE/
  CURRENT_ANIM/IS_FINISHED/SET_FRAME/GET_FRAME/UPDATE/COLLIDES/HIT_BOX/HIT_POINT`
  (in `builtins.rs`) + `SPRITE_DRAW` (Sheet-Frame als Sub-Rect, Camera-aware,
  Flip/Scale/Tint, in `graphics.rs`). 143_sprite_visual + 66_sprite_editor rendern.
  *Grenze:* Konsolen-Demos ohne `SCREEN` gehen nicht (Texturen brauchen GL-Kontext).
- `tween` — `Value::Tween`, 19 Easings, `TWEEN_NEW/_LOOP/_PINGPONG/VALUE/PROGRESS/
  DONE/RESTART/PAUSE/RESUME/REVERSE/EASINGS`. **Zeitbasiert** (Wall-Clock) → wie
  `RND`/`MILLIS` NICHT bit-identisch, aber funktional (47_collision_easing rendert).
- `json` — `Value::Json` (serde_json mit `preserve_order`). `JSON_PARSE/LOAD/
  STRINGIFY/PRETTY/GET_STRING/INT/FLOAT/BOOL/HAS/LEN/TYPE`, Pfad-Navigation
  (`"user.name"`/`"items.0"`). Bit-identisch inkl. `STRINGIFY` (Key-Reihenfolge).

- `scene` — Stack-Scene-Manager (VM-globaler State). `SCENE_PUSH/POP/SWITCH/
  CURRENT/DEPTH/HAS/RESET/SET_INT|FLOAT|STRING|BOOL/GET_*(_OR)/HAS_KEY/DELETE`.
  Bit-identisch.
- `save` — Save-Slots (`Value::Save`). `SAVE_NEW/LOAD/LOAD_OR_NEW/EXISTS/WRITE/
  DELETE_FILE/VERSION/SET_VERSION/SET_*/GET_*(_OR)/HAS/DELETE/CLEAR/KEYS`.
  JSON-Datei-Backend (serde_json pretty). Bit-identisch (inkl. Datei-Roundtrip).

**Core File-I/O** (kein Modul): `OPENFILE`/`CLOSEFILE`/`READLINE`/`READALL$`/
`ENDOFFILE`/`WRITELINE`/`WRITE`/`FILEEXISTS` (`Value::File`). Bit-identisch.

- `astar` — A*-Pathfinding (`Value::AStar`, [astar.rs](../rust/gb_runtime/src/astar.rs)
  portiert aus dem PyO3-Helper `gb_native`). `ASTAR_NEW/CLEAR/WIDTH/HEIGHT/SET_WALL/
  SET_PASSABLE/IS_WALL/SET_DIAGONAL/SET_HEURISTIC/SET_DIAGONAL_COST/FIND/PATH_LEN/
  PATH_X/PATH_Y/PATH_COST/CLEAR_PATH`. Bit-identisch inkl. counter-FIFO-Tie-Break.

- `particles` — `Value::Particles`. `PARTICLE_SYSTEM_NEW/SET_POS/COUNT/CLEAR/
  SET_VELOCITY/SET_LIFETIME/SET_GRAVITY/SET_COLOR/SET_SIZE/SET_FADE/SET_MODE/
  SET_COLOR_END/EMIT/UPDATE/DRAW`. 5 Render-Modi (circle/pixel/square/streak/
  glow), Fade + Farbverlauf. RNG-Emit → wie `tween` zeitabhängig/nicht
  bit-identisch, aber funktional (78_particle_catalog rendert alle Modi).

- `imgfx` — `IMAGE_SCALE/ROTATE/FLIP/TINT/COPY` (immutable, neues Handle). Über
  raylib-`Image` (CPU-Pixel) transformiert + `LoadTextureFromImage`. Texturen
  werden als `Tex { tex, img }` (GPU+CPU) gehalten.

- `ecs` — Entity-Component-System ([ecs.rs](../rust/gb_runtime/src/ecs.rs),
  Sparse-Set). `ECS_NEW_WORLD/NEW_ENTITY/DESTROY/ALIVE/COUNT`, `ADD_INT/FLOAT/
  STRING/BOOL/OBJ`, `HAS/REMOVE/GET*/GET_OR_*`, `QUERY/QUERY2/QUERY3` (sortierte
  Intersection), Bulk-Ops `INTEGRATE_FLOAT/INT/SCALE_FLOAT/FILL_*/CLAMP_FLOAT/
  REMOVE_DEAD/COUNT_WITH`. Bit-identisch.

- `ui` (Immediate-Mode) — `UI_LABEL/BUTTON/CHECKBOX/SLIDER/PROGRESS/PANEL/RADIO/
  END_FRAME/RESET` + Theme (`UI_THEME_SET/GET`, `UI_METRIC_SET/GET`,
  `UI_THEME_PRESET` dark/light/retro/contrast). State per String-ID auf der VM.
  **Neu portiert:** `UI_TEXTFIELD`/`UI_TEXTFIELD_SET` (Tastatur via neuem
  `Graphics::pop_text_input` = raylib `get_char_pressed`-Drain, Backspace-Edge,
  blinkender Caret) und `UI_WINDOW_BEGIN/END` (verschiebbare Immediate-Mode-
  Fenster: Offset-Threading durch ALLE Widgets via `UiState.offset_x/y`,
  Input-Gating überdeckter Fenster via `ui_mouse_gated`, Titel-Drag +
  Einklapp-Pfeil, Z-Order über Vorframe-Hit-Test `active_win`/`hover_win` in
  `UI_END_FRAME`). Beide per Headless-Screenshot verifiziert.
  **`UI_TABLE`** ist nun ebenfalls portiert: fixierte Kopfzeile, V/H-Scroll
  (Mausrad + Scrollbalken-Drag inkl. Track-Klick), Per-Zelle-Text- und
  -Hintergrundfarben, Hover-/Selektions-Highlight, klickbare Zeilen
  (Rueckgabe = geklickte Zeile) + `UI_TABLE_SELECTED`/`SET_SELECTED`/
  `HEADER_CLICK` (Sortier-Hook). State per id in `UiState.tables`; nutzt den
  neuen Clip-Stack + Mausrad. Verifiziert per Screenshot
  ([examples/43_ui_table.gb](../examples/43_ui_table.gb), Zell-Farben +
  beide Scrollbars).

**`gui` (Retained-Mode)** — Kern portiert ([gui.rs](../rust/gb_runtime/src/gui.rs)):
Window + Button/Label/Checkbox/Slider/TextInput/Panel, Drag/Z-Order/Fokus/Close,
programmierbares Theme (`GUI_THEME_SET/GET/PRESET`, `GUI_METRIC_SET/GET`,
`GUI_SET_COLOR`), Polling (`GUI_CLICKED/CHECKED/VALUE/TEXT`, Setter). Handles =
INTEGER (Window = Index, Widget = `(win<<20)|idx`); Z-Order über separate
`z_order`-Liste, damit Handles stabil bleiben. Render per Screenshot verifiziert
(`examples/45_gui.gb`); Interaktion ist ein 1:1-Port der (getesteten) Python-
Logik — headless nicht klickbar testbar. **FUNCREF-Callbacks**
(`GUI_ON_CLICK`/`GUI_ON_CHANGE`) feuern: `update()` sammelt ausgeloeste Handler
in `pending`, die VM leert die Queue nach `GUI_UPDATE` und ruft sie (parameter-
los) via `exec` auf — so kann ein Callback die GUI sicher verändern, neue
Events landen nächsten Frame.

**`GUI_TABLE`** ist nativ portiert: fixierte Kopfzeile, V/H-Scroll
(Mausrad + Scrollbalken-Drag), Hover-/Selektions-Highlight, klickbare Zeilen,
`GUI_TABLE_HEADERS/ROWS/COL_WIDTHS/SELECTED/SET_SELECTED/CLICKED/ROW_COUNT` +
`GUI_ON_CHANGE` bei Selektionswechsel. Layout aus einer Quelle (`table_geom`).
Dafür hat die Grafik neu: **Mausrad** (`pop_mouse_wheel`) und einen
**Clip-Stack** (`push_clip`/`pop_clip` → raylib-Scissor mit Verschnitt). Render
per Screenshot verifiziert ([examples/84_gui_table.gb](../examples/84_gui_table.gb));
Rust-Unit-Tests decken Callback-Queueing, Tabellen-Layout (`table_geom`) und
Press→Selektion ab (`build_runtime.py --test`). *Noch offen:* der Immediate-
Mode-`UI_TABLE` (separates Modul `ui`).

**Tabellen komplett:** `UI_TABLE` (Immediate-Mode) **und** `GUI_TABLE`
(Retained) sind nativ. Hardware/Netzwerk + `regex` bleiben außen vor.

**ENUM / STATIC CONST:** `_EnumNamespace`/`_ClassStaticNamespace` werden als
`{"ns": {name, members}}` serialisiert und in Rust als `Value::Namespace`
geladen; `LOAD_MEMBER` löst Member case-insensitiv auf. Damit gibt es **keine
serialize-Fehler mehr** (vorher 6).

**Nicht geplant:** Hardware/Netzwerk-Module (`bt`/`serial`/`usb`/`wifi`/`net`/
`html`/`db`) und `regex` (Python-`re` nicht bit-identisch nachbaubar). Das
erweiterte `audio`-Modul (Kanäle, Pan, Ton-Generierung) bleibt vorerst
Python-only — die **Core-Audio-Builtins** sind aber nativ (siehe unten).

## Audio (Core: SFX + Stream-Musik)

Native Audio über raylib (Modul `audio.rs`, feature-gated wie die Grafik —
raylib bundelt den Mixer mit). **Core-Builtins, kein `IMPORT` nötig:**
- `LOADSOUND(pfad$) -> SOUND` (Handle = INTEGER-Index), `PLAYSOUND(sound[,
  loops, lautstaerke])`, `STOPSOUND(sound)`.
- `PLAYMUSIC(pfad$[, loops, lautstaerke])`, `STOPMUSIC()` — ein Stream
  gleichzeitig; `gbrt` ruft `update_stream` pro `FLIP` (sonst stockt die
  Wiedergabe). Musik loopt (raylib-Default).

**Audio-Reaktivität (FFT):** `AUDIO_FFT(bands)` füllt ein `ARRAY OF FLOAT` mit
B logarithmisch verteilten Frequenzband-Pegeln (0..1) des **aktuell hörbaren**
Audios. Dafür hängt `Audio::new` via `AttachAudioMixedProcessor` einen
`extern "C"`-Callback an die gesamte raylib-Audio-Pipeline; der schiebt das
gemischte Mono-Signal in einen globalen Ringpuffer (`try_lock`, Audio-Thread
blockiert nie). `fft_bands` fenstert (Hann), rechnet eine eigene Radix-2-FFT
(1024), bündelt log-spaced, normalisiert per Auto-Gain und glättet per
Peak-Hold. (`AUDIO_FFT` ist nativ; ohne Mix-Tap füllt es Nullen.) So tanzen
Spektrum **und** Geometrie der Demo wirklich zur Musik.

WAV/OGG/MP3/FLAC je nach raylib-Build. Audio gehoert **nicht** zur
bit-identischen Garantie — wie `RND`/`MILLIS`/`tween` nur funktional.
*Grenze:* `loops` wird nativ (noch) nicht ausgewertet — SFX spielen einmal,
Musik loopt immer. Lifetime-Trick: das `RaylibAudio`-Gerät wird per `Box::leak`
zu `&'static`, damit `Sound`/`Music` in `Vec`/`Option` gehalten werden können
(kein self-referential struct). Demo: [examples/83_audio.gb](../examples/83_audio.gb).

## Schritt 6: 3D-Grafik (Modul `g3d`)

3D ist **native-only**: raylib hat eine echte 3D-Pipeline, der Tree-Walker nicht. Das
Modul `g3d` registriert die Builtins (damit der Compiler `CALL_BUILTIN`
emittiert); im Python/Tree-Walker-Pfad (F5) werfen sie eine klare Meldung
(„… nur in der nativen Runtime … mit F6"). In `gbrt` rendern sie über raylibs
`begin_mode3D`-API.

**Builtins** ([`g3d.py`](../gamebasic/modules/g3d.py), Rendering in
`graphics.rs`/`vm.rs`):
- `CAMERA3D(px,py,pz, tx,ty,tz, fovy)` — Perspektiv-Kamera (Up = +Y), pro Frame.
- `CAMERA_ORBIT(tx,ty,tz, radius, yaw, pitch[, fovy])` — Orbit-Kamera: blickt aus
  `radius` Abstand auf das Ziel `(tx,ty,tz)`, gesteuert über `yaw`/`pitch` in
  **Grad**. Spart die manuelle Kugelkoordinaten-Trigonometrie
  (`px = COS(pitch)*SIN(yaw)*r` …). `pitch` wird intern auf `±89.9°` geklemmt
  (kein Gimbal-Flip am Pol). `fovy` weglassen = aktuelle Brennweite behalten
  (sonst 45). Ideal für eine maus-/tastengesteuerte Umlauf-Kamera:

  ```basic
  ' yaw/pitch im Game-Loop aus der Maus aufaddieren, dann:
  CAMERA_ORBIT(0.0, 1.0, 0.0, 12.0, yaw, pitch)
  ```
- `CUBE` / `CUBE_WIRES` `(x,y,z, w,h,d, farbe)` — gefüllter / Drahtgitter-Quader.
- `SPHERE` / `SPHERE_WIRES` `(x,y,z, r, farbe)`.
- `CYLINDER(x,y,z, r_oben, r_unten, h, farbe)` — `r_oben=0` ⇒ Kegel.
- `PLANE(x,y,z, size_x, size_z, farbe)` — XZ-Ebene.
- `LINE3D(x1,y1,z1, x2,y2,z2, farbe)`, `POINT3D(x,y,z, farbe)`.
- `GRID3D(linien, abstand)` — Boden-Raster.

**Render-Modell** (erweitert das 2D-Recording): 3D-Cmds landen in einer eigenen
Liste `cmds3d`; beim `FLIP` rendert `gbrt` **zuerst** alle 3D-Cmds in einem
`begin_mode3D(cam3d)`-Block, **danach** die 2D-Layer obenauf — das 2D-HUD liegt
also immer über der Szene. Koordinaten sind Welt-Einheiten (kein Screen-Scale),
Farben `&HRRGGBB`. `cmds3d` wird pro Frame geleert; ohne `CAMERA3D` gilt ein
Default-Blick (schräg von vorn-oben auf den Ursprung).

Demo: [examples/82_3d_intro.gb](../examples/82_3d_intro.gb) (Würfel, Kugel,
Zylinder, Kegel, Linien, Gitter + 2D-HUD), per Screenshot verifiziert.

### 3D-Modelle (geladen + prozedural)

Über die Immediate-Mode-Primitive hinaus gibt es **wiederverwendbare
Modell-Handles** (INTEGER), die einmal erzeugt und über beliebig viele Frames
gezeichnet werden — anders als `CUBE`/`SPHERE`, die jeden Frame neu aufgebaut
werden.

- `LOADMODEL(pfad$) -> MODEL` — lädt OBJ/GLTF/IQM/… via raylib `LoadModel`.
- **Prozedurale Meshes** (kein Asset nötig) via `GenMesh*` →
  `LoadModelFromMesh`: `MESH_CUBE(w,h,d)`, `MESH_SPHERE(r, ringe, segmente)`,
  `MESH_CYLINDER(r, h, segmente)`, `MESH_TORUS(r, dicke, rad_seg, seiten)`,
  `MESH_KNOT(r, dicke, rad_seg, seiten)`, `MESH_PLANE(w, l, res_x, res_z)`,
  `MESH_HEIGHTMAP(bild, groesse_x, groesse_y, groesse_z)` (Terrain aus einer
  Graustufen-Image: Helligkeit = Höhe, `groesse_y` skaliert sie).
  Torus/Knot sind neue Formen ggü. den Immediate-Primitiven.
- **Zeichnen:** `MODEL(m, x,y,z, scale, farbe)` (`DrawModel`),
  `MODEL_EX(m, x,y,z, achse_x,achse_y,achse_z, winkel_grad, scale, farbe)`
  (`DrawModelEx` — Rotation um eine Achse), `MODEL_WIRES(m, x,y,z, scale, farbe)`.
- `MODEL_TEXTURE(m, bild)` — ein via `LOADIMAGE` geladenes Bild als
  Diffuse-/Albedo-Map (`MATERIAL_MAP_ALBEDO`) auf das Modell legen.

**Umsetzung** ([graphics.rs](../rust/gb_runtime/src/graphics.rs)): `models:
Vec<Model>` lebt über die ganze Laufzeit (Handles bleiben gültig). Die Draw-
Builtins emittieren `Cmd3D::Model`/`ModelEx`/`ModelWires` mit dem Model-**Index**
(nicht dem Model selbst → `Cmd3D` bleibt `Clone`); der 3D-Pass in `render_scene`
(jetzt mit `models: &[Model]`-Parameter) zeichnet sie via `draw_model[_ex|_wires]`.
GenMesh-Meshes werden mit `make_weak()` an `load_model_from_mesh` übergeben (kein
Doppel-Drop). Handle-Validierung in den Wrappern (`check_model`).

Demo [examples/88_3d_models.gb](../examples/88_3d_models.gb): rotierender Torus,
Knoten (Wireframe) und pulsierende Kugel auf einer Ebene, umkreisende Kamera +
2D-HUD — rein prozedural, **kein Modell-Asset im Repo nötig**. Per Screenshot
verifiziert (inkl. `MODEL_TEXTURE` mit `assets/coin.png` auf einem Würfel).

`MESH_HEIGHTMAP` baut aus dem CPU-`Image` (`self.textures[i].img`, bereits für
imgfx gehalten) via `GenMeshHeightmap` ein Terrain-Mesh. Demo
[examples/89_heightmap.gb](../examples/89_heightmap.gb): texturiertes, von einer
Kamera umkreistes Terrain mit Drahtgitter-Overlay (`examples/assets/heightmap.png`,
ein generiertes 129×129-Graustufen-PNG). Per Screenshot verifiziert.

### Billboards + Ray-Kollision / Picking

- `BILLBOARD(bild, x,y,z, groesse, farbe)` — eine `LOADIMAGE`-Textur im 3D-Raum,
  die über `DrawBillboard` immer zur Kamera zeigt (Bäume, Sprites, Funken).
  Eigene `Cmd3D::Billboard` (Textur-Index); der 3D-Pass hat Kamera **und**
  `textures` zur Hand.
- **Ray-Kollision** (liefert Distanz vom Ursprung zum Treffer oder `-1`,
  Trefferpunkt = `ursprung + richtung * distanz`):
  - `RAY_HIT_BOX(ox,oy,oz, dx,dy,dz, cx,cy,cz, sx,sy,sz)` — AABB (Mittelpunkt c,
    Vollgröße s) via `GetRayCollisionBox`.
  - `RAY_HIT_SPHERE(ox,oy,oz, dx,dy,dz, cx,cy,cz, r)` via `GetRayCollisionSphere`.
  - `RAY_HIT_MODEL(modell, ox,oy,oz, dx,dy,dz, px,py,pz[, scale])` via
    `GetRayCollisionMesh` über **alle Meshes** des bei `(px,py,pz)` mit
    `scale` platzierten Modells (Default `scale=1`). Distanz zum nächsten
    Treffer oder `-1`. So lassen sich auch geladene Meshes (`LOADMODEL`) und
    Prozedural-Modelle (`MESH_*`) picken, nicht nur Box/Sphere-Proxys.
  - `RAY_HIT_TRI(ox,oy,oz, dx,dy,dz, x1,y1,z1, x2,y2,z2, x3,y3,z3)` — ein
    einzelnes **Dreieck** via `GetRayCollisionTriangle` (Möller-Trumbore,
    **ohne Backface-Culling**: eine Fläche trifft auch von hinten).
  - `RAY_HIT_QUAD(ox,oy,oz, dx,dy,dz, <4 Punkte à x,y,z>)` — ein **Viereck**
    (intern zwei Dreiecke). Die vier Punkte müssen **reihum** liegen (im Kreis,
    nicht über Kreuz), sonst prüfen die Teildreiecke die falsche Fläche.
  - Die Richtung muss **nicht** normalisiert sein — gbrt normalisiert vor dem
    Test, sonst käme die Distanz in Vielfachen der Richtungslänge zurück
    (raylibs Rohverhalten).
- **Maus-Picking** (Strahl vom Cursor durch die aktuelle 3D-Kamera,
  `GetScreenToWorldRay` + Treffertest): `PICK_BOX(cx,cy,cz, sx,sy,sz)`,
  `PICK_SPHERE(cx,cy,cz, r)`, `PICK_MODEL(modell, px,py,pz[, scale])`,
  `PICK_TRI(<3 Punkte>)`, `PICK_QUAD(<4 Punkte>)` — ideal für Klick-Selektion.
  Nächstes Objekt = kleinste nicht-negative Distanz. TRI/QUAD treffen die
  **tatsächliche Fläche** statt eines Hüllkörpers: Bodenkacheln, Wandstücke,
  frei im Raum schwebende Panels (Demo
  [examples/151_picking_flaechen.gb](../examples/151_picking_flaechen.gb)).
- **Projektion in beide Richtungen** (durch die aktuelle 3D-Kamera):
  `WORLD_TO_SCREEN_X(wx,wy,wz)` / `WORLD_TO_SCREEN_Y(wx,wy,wz)` projizieren einen
  3D-Weltpunkt auf Bildschirm-Pixel (z.B. ein 2D-Label über ein 3D-Objekt
  setzen). Umgekehrt liefert `SCREEN_TO_WORLD_DIR_X/Y/Z(sx,sy)` die **Richtung**
  des Strahls durch einen Screen-Punkt; der **Ursprung** des Strahls ist die
  Kameraposition (`CAMERA3D_X/Y/Z`). Damit baut man eigene Treffertests, z.B.
  zusammen mit `RAY_HIT_MODEL`.

```basic
' Selbst-gebautes Maus-Picking eines Modells:
DIM ox AS FLOAT
DIM oy AS FLOAT
DIM oz AS FLOAT
ox = CAMERA3D_X() : oy = CAMERA3D_Y() : oz = CAMERA3D_Z()
DIM dist AS FLOAT
dist = RAY_HIT_MODEL(held, ox, oy, oz, _
        SCREEN_TO_WORLD_DIR_X(MOUSEX(), MOUSEY()), _
        SCREEN_TO_WORLD_DIR_Y(MOUSEX(), MOUSEY()), _
        SCREEN_TO_WORLD_DIR_Z(MOUSEX(), MOUSEY()), 0.0, 0.0, 0.0)
IF dist >= 0.0 THEN PRINT "getroffen bei ", dist
' (oder einfacher: PICK_MODEL(held, 0.0, 0.0, 0.0))
```
- **Cursor auf die Boden-Ebene projizieren** (Strahl vom Cursor → waagerechte
  Ebene bei Welt-Y `ebene_y`): `MOUSE_GROUND_X(ebene_y)` und
  `MOUSE_GROUND_Z(ebene_y)` liefern die Welt-X/Z des Treffpunkts,
  `MOUSE_GROUND_HIT(ebene_y)` → BOOLEAN, ob der Strahl die Ebene **vor** der
  Kamera trifft. Das ist der einfache „Wohin zeigt die Maus in der 3D-Welt?"-
  Helfer (Einheiten platzieren, Bauen auf dem Grund, Cursor-Marker) — ohne dass
  man selbst die Strahl-Ebenen-Mathematik aufstellen muss.

```basic
' Cursor-Marker auf dem Boden (y=0)
IF MOUSE_GROUND_HIT(0.0) THEN
    DIM wx AS FLOAT
    DIM wz AS FLOAT
    wx = MOUSE_GROUND_X(0.0)
    wz = MOUSE_GROUND_Z(0.0)
    MODEL(marker, wx, 0.0, wz, WHITE)
END IF
```

`RAY_HIT_*` ist reine Geometrie und deterministisch (verifiziert: Kugel/Box bei
`(5,0,0)` entlang +X → Distanz `4.0`, senkrechter Strahl → `-1.0`). `PICK_*`
hängt am Maus-/Fensterzustand (headless: Maus `(0,0)` → alles `-1`). Demo
[examples/90_billboards_picking.gb](../examples/90_billboards_picking.gb):
Coin-Billboards + per Maus selektierbarer Würfel/Kugel. Per Screenshot verifiziert.

### Kamera-Modi (`UpdateCamera`)

`cam3d` lebt über Frames hinweg. Statt jeden Frame `CAMERA3D(...)` neu zu setzen,
kann man die Kamera von raylib bewegen lassen:

- `CAMERA3D(...)` einmal initial setzen, dann pro Frame `CAMERA3D_UPDATE(mode)` —
  `mode`: `1`=free, `2`=orbital, `3`=first_person, `4`=third_person. raylib liest
  Tastatur/Maus (WASD + Mouse-Look) bzw. rotiert bei *orbital* automatisch.
- Getter: `CAMERA3D_X/Y/Z()` (Position) und `CAMERA3D_TARGET_X/Y/Z()` — z. B. für
  First-Person (Spielerposition = Kamera). Verifiziert (Position/Ziel exakt zurück).

### Beleuchtung (PBR / Cook-Torrance, bis zu 4 Lichter)

Echte Pro-Pixel-Beleuchtung über den eingebetteten Lighting-Shader (GLSL 330, als
`const LIGHT_VS`/`LIGHT_FS` im Crate — **kein Shader-Asset nötig**). Der Fragment-
Shader nutzt das **Cook-Torrance-PBR-Modell** (GGX-Normalverteilung, Smith-
Geometrie, Fresnel-Schlick) mit Reinhard-Tonemapping + Gamma; analytische Lichter
(directional/point), kein IBL:

- `LIGHT_ENABLE()` lädt den Lighting-Shader (einmal) und cached die Uniform-
  Locations (`viewPos`, `ambient`).
- `LIGHT_AMBIENT(farbe, intensitaet)` — Grundhelligkeit.
- `LIGHT_DIRECTIONAL(dx,dy,dz, farbe)` (Sonne, Richtung) / `LIGHT_POINT(x,y,z, farbe)`
  (Punktlicht) → Licht-Index (max. 4, sonst `-1`).
- `LIGHT_SET_POS/COLOR/ENABLED(idx, …)` — Lichter pro Frame animieren.
- `MODEL_LIT(modell)` — hängt den Lighting-Shader an die Materialien des Modells
  (setzt `material.shader` direkt über das ffi-Feld; der Shader bleibt in
  `Graphics.light_shader` am Leben). Erst danach wird das Modell beleuchtet.
- `MODEL_PBR(modell, metalness, roughness)` — PBR-Materialparameter (je 0..1):
  `metalness` 0 = Dielektrikum (Plastik/Stein), 1 = Metall (Specular nimmt die
  Albedo-Farbe an); `roughness` 0 = spiegelnd, 1 = matt. Pro Modell gespeichert
  (`pbr_params`-Map, Default 0 / 0.6) und in `render_scene` als Uniform vor jedem
  Modell-Draw gesetzt — zusammen mit `useNormalMap`. Albedo = `colDiffuse`
  (MODEL-Tint) × `texture0`. Demo [examples/95_pbr.gb](../examples/95_pbr.gb):
  Kugel-Gitter Metalness × Roughness (per Screenshot verifiziert).
- `LIGHT_ENV(himmel, boden, intensität)` — **analytisches Image-Based-Lighting**
  (`intensität` 0 = aus). Die Umgebung ist ein vertikaler Farbgradient
  (boden→himmel); der Shader fügt diffuse Hemisphären-Irradiance + eine
  roughness-abhängig verwischte Sky-**Reflexion** (reflektierter View-Vektor) +
  die analytische Environment-BRDF (Karis-Approximation statt LUT) hinzu, mit
  roughness-Fresnel. **Kein HDR-Asset, keine Cubemap-Passes** — eine reine
  Shader-Erweiterung. Erst damit wirken Metalle (`metalness` 1) wirklich
  metallisch (sie reflektieren die Umgebung statt dunkel zu bleiben). Demo
  [examples/96_ibl.gb](../examples/96_ibl.gb): Metall- vs. Dielektrikum-Reihe.
- `LIGHT_FOG(farbe, dichte)` — exponentieller Tiefen-Fog für die beleuchteten
  Modelle (`dichte 0` = aus). Ferne Objekte verblassen zur Fog-Farbe; im
  Fragment-Shader `mix(fogColor, finalColor, 1/exp((dist·dichte)²))`. Tipp:
  `CLS(fogColor)` für einen nahtlosen Horizont. Per Screenshot verifiziert
  (Säulenreihe verschwindet im Dunst, [examples/92_fog.gb](../examples/92_fog.gb)).
  *Grenze:* wirkt nur auf `MODEL_LIT`-Modelle (nutzen den Lighting-Shader),
  nicht auf Immediate-Primitive/Grid.

`flip()` ruft vor dem 3D-Pass `update_light_uniforms()`: `viewPos` (= Kamera-
Position), `ambient` und alle `lights[i].*`-Uniforms werden auf den Shader
geschrieben. raylib bindet `matModel`/`matNormal`/`mvp` automatisch über die
Standard-Uniform-Namen; nur `matModel` setzen wir explizit in die `locs`. Demo
[examples/91_lighting.gb](../examples/91_lighting.gb): Sonne + bewegtes Punktlicht
auf Kugel/Würfel/Torus, orbitale Kamera. Per Screenshot verifiziert (Diffus +
Specular-Highlights + Lichtkegel des Punktlichts am Boden).

### Schatten (Shadow-Mapping)

Echte Schlagschatten über einen Depth-Pass aus Sicht des Lichts (Port des
offiziellen raylib-`shaders_shadowmap`-Beispiels in die Recording-Pipeline):

- `SHADOW_ENABLE([auflösung])` — legt ein **sampleable Depth-FBO** an
  (`rlLoadFramebuffer` + `rlLoadTextureDepth(res, res, false)` + `rlFramebufferAttach`
  als `RL_ATTACHMENT_DEPTH/TEXTURE2D`; die Default-`RenderTexture`-Tiefe ist ein
  Renderbuffer, also *nicht* sampleable) und cached die Shader-Locations.
  Default 1024, geclamped auf 256…4096.
- `SHADOW_AREA(größe, distanz)` — halbe Kantenlänge des orthografischen
  Schatten-Frustums + Abstand der Licht-Kamera (kleiner = schärfer).
- `SHADOW_TARGET(x,y,z)` — Mittelpunkt des Schattenbereichs (z. B. dem Spieler
  folgen lassen).

**Ablauf pro Frame** (`render_shadow_map`, vor dem Haupt-Pass): aus dem ersten
`LIGHT_DIRECTIONAL` wird eine orthografische Licht-Kamera gebaut; `rlEnableFramebuffer`
→ `rlViewport(res)` → `BeginMode3D(lightCam)` → alle `MODEL`/`MODEL_EX`-Draws via
`ffi::DrawModel*` in die Depth-Map gerendert. `lightVP = lightView·lightProj`
(aus `rlGetMatrixModelview/Projection`) geht als Uniform in den Lighting-Shader;
die Depth-Textur wird an Texture-Unit 10 gebunden (`rlActiveTextureSlot`/
`rlEnableTexture`, der Sampler-Uniform auf 10 gesetzt — Material-Maps nutzen 0…2,
kein Clash). Der Fragment-Shader transformiert jeden Punkt in Light-Space und
vergleicht mit der Depth-Map (**3×3-PCF** + Normalen-abhängiger Bias gegen
Shadow-Acne); im Schatten bleiben 15 % Direktlicht (Ambient unberührt).
`shadowsEnabled`-Uniform (Default 0) → ohne `SHADOW_ENABLE` keinerlei Effekt,
bestehende Lighting-Demos unverändert.

*Caster/Receiver:* `MODEL_LIT`-Modelle (Immediate-Primitive/Grid werfen keine
Schatten). Ein schattenwerfendes directional Light. Demo
[examples/93_shadows.gb](../examples/93_shadows.gb): schwebende Kugel/Würfel/Torus
werfen weiche Schatten auf den Boden (per Screenshot verifiziert).

### Normal-Mapping

Pro-Pixel-Oberflächendetail ohne mehr Geometrie, integriert in den Lighting-
Shader:

- `MODEL_TEXTURE_NORMAL(modell, bild)` — eine via `LOADIMAGE` geladene Normal-Map
  (Tangent-Space, RGB = Normale·0.5+0.5) auf ein `MODEL_LIT`-Modell legen
  (`MATERIAL_MAP_NORMAL` = Shader-Sampler `texture2`).
- `MODEL_LIT` erzeugt jetzt zusätzlich die **Tangenten** (`gen_mesh_tangents` auf
  allen Meshes) — Voraussetzung für die TBN-Basis im Shader.

**Shader:** der VS reicht `vertexTangent` (world-space) durch; der FS baut aus
`fragNormal` + `fragTangent` eine TBN-Matrix, sampelt die Normal-Map und stört die
Normale pro Pixel. Ein `useNormalMap`-Uniform (Default 0) gated das — pro Modell
in `render_scene` gesetzt (1 nur für Modelle in `normal_mapped`). So bleiben lit
Modelle ohne Normal-Map **pixelgenau** wie zuvor (keine Abhängigkeit von einer
Default-Textur). Demo [examples/94_normalmap.gb](../examples/94_normalmap.gb):
Platte + Kugel links mit, rechts ohne Normal-Map unter kreisendem Punktlicht —
links wandern die Wellen-Bumps, rechts bleibt es glatt (per Screenshot
verifiziert; Normal-Map `examples/assets/normal_waves.png` prozedural generiert).

Der native 3D-Stack ist damit **vollständig**: Modelle, Meshes inkl. Heightmap,
Texturen, Normal-Maps, **PBR + analytisches IBL (`LIGHT_ENV`) + echtes
HDR-Cubemap-IBL (`LIGHT_ENV_HDR`)**, Billboards, Picking, Beleuchtung inkl. Fog
und Schatten, Kamera-Modi (siehe „Echtes HDR-Cubemap-IBL" oben). Third-Person-
Kollision ist Gameplay-Logik (kein Engine-Primitive).

## Shader / Post-Processing (native)

GPU-Fragment-Shader fuer Ganzbild-Effekte (CRT, Bloom, Vignette, …) — **nur
native** (raylib/OpenGL). Builtins: `SHADER_LOAD(pfad$_oder_glsl$)` (Datei ODER
GLSL-Quelltext → SHADER-Handle/-1), `SHADER_SET`/`SHADER_SET2`/`SHADER_SET3`
(float/vec2/vec3-Uniforms), `POSTFX(h)` (Frame durch den Shader; -1 = aus).

**Render-Modell:** Ist ein Post-Shader aktiv, rendert `FLIP` die ganze Szene
(3D + 2D + Scissor) nicht direkt auf den Screen, sondern in eine
`RenderTexture2D`; danach wird diese Textur full-screen durch
`BeginShaderMode(shader)` praesentiert (Y-flip wegen RT-Konvention). Der
Replay-Code ist generisch (`fn render_scene<D: RaylibDraw>`), laeuft also
identisch auf den Screen *oder* in die RenderTexture — `RaylibDrawHandle` und
`RaylibTextureMode` implementieren beide `RaylibDraw`. Shader-Handles liegen in
`Graphics.shaders`, der aktive Index in `post_shader_idx`.

Auf dem Tree-Walker (konsolen-only) werfen die Shader-Builtins "nur in der
nativen Runtime (gbrt)". Beispiel-Shader (GLSL 330):
[examples/assets/shaders/](../examples/assets/shaders/) (`crt.fs`/`bloom.fs`/
`vignette.fs`), Demo [examples/86_postfx_shaders.gb](../examples/86_postfx_shaders.gb)
(zyklisch AUS → CRT → BLOOM → VIGNETTE; CRT + Bloom per Screenshot verifiziert).

## TTF-Fonts (`LOADFONT` / `SETFONT` / `TEXT_SPACING`)

Eigene TrueType-/OpenType-Schriften statt nur des eingebauten Default-Fonts.
**Core-Builtins, kein `IMPORT` nötig** — nativ in gbrt (Tree-Walker konsolen-only):

- `LOADFONT(pfad$, groesse) -> FONT` — lädt eine TTF/OTF in der Basis-Größe
  `groesse` (Glyph-Auflösung) und liefert ein **FONT-Handle (INTEGER)**.
- `SETFONT(font)` — aktiviert den Font für nachfolgende `TEXT`-Aufrufe.
  `SETFONT(-1)` schaltet zurück auf den Default-Font.
- `TEXT_SPACING(px)` — Buchstabenabstand für TTF-Text (wirkt nativ über
  `DrawTextEx`).

`TEXT_SIZE` skaliert den aktiven Font weiterhin frei (nativ skaliert raylib die
einmal geladene Glyph-Textur). `TEXT_WIDTH` misst in der **aktiven** Schrift
(nativ `MeasureTextEx`) — damit funktioniert Zentrieren/Rechtsbündig auch mit
TTF. `TEXT_BOLD`/`TEXT_ITALIC` sind nativ No-Op (raylib hat keine synthetische
Variante — dafür eine fette/kursive Font-Datei laden).

**Native Umsetzung** ([graphics.rs](../rust/gb_runtime/src/graphics.rs)): `fonts:
Vec<Font>` (raylib `load_font_ex`), `active_font` (-1 = Default), `text_spacing`.
`Cmd::Text` trägt jetzt Font-Index + Spacing; beim Replay zeichnet ein gültiger
Index via `draw_text_ex(font, …)`, sonst der Default-`draw_text`. Der Tree-Walker
([graphics.py](../gamebasic/graphics.py)) ist konsolen-only und rendert keinen Text.

**Bit-Identität gilt nicht** (Renderer/Font-Metriken unterscheiden sich) — wie
bei der übrigen Grafik nur funktional. Es liegt **kein Font-Asset im Repo**;
Demo [examples/87_ttf_fonts.gb](../examples/87_ttf_fonts.gb) sucht einen
System-Font (`FILEEXISTS`) und fällt sonst auf den Default-Font zurück. Per
Screenshot verifiziert (Größen-Skalierung, Spacing, zentrierter Text via
`TEXT_WIDTH`).

## Showcase-Demos

[examples/97_pbr_reactor.gb](../examples/97_pbr_reactor.gb) — **„PBR REACTOR"**,
das Fullscreen-Schaustück der **neuen** Grafik-Pipeline: chrom­glänzende
**PBR**-Kugeln (eine pro FFT-Band, geglättet = sanftes „Atmen") um einen
rotierenden Chrom-Knoten auf einem spiegelnden Metallboden, mit **Szenenwechsel
alle 14 s** (RING → WAVE → HELIX), beleuchtet per **Image-Based-Lighting**
(`LIGHT_ENV`, animierte Sky-Farbe) + Sonne mit **Schatten** (`SHADOW_ENABLE`) +
zwei bass-pulsierenden Punktlichtern, dazu **Fog**, **Bloom** (`POSTFX`),
Glow-Funken auf dem Kick und ein 2D-FFT-Spektrum. Alles **echt FFT-reaktiv**
(`AUDIO_FFT`) zu einem Stereo-Techno-Track. `SET_FULLSCREEN(TRUE)`, Kamera kreist
mit Bass-Punch. Musik: „Technological Messup" von **josepharaoh99**, **CC0** —
einmalig holen mit `py examples/assets/download_techno.py` (läuft auch ohne, stumm
via `FILEEXISTS`-Guard). Nur nativ: `gbrun.py --native examples\97_pbr_reactor.gb`.

[examples/85_cybermatic_demo.gb](../examples/85_cybermatic_demo.gb) bündelt in
einem 1280×720-Frame, was die native Runtime kann — **audio-reaktiv** (echte
FFT der laufenden Musik via `AUDIO_FFT`) und mit **Szenen-Wechsel alle 16
Takte**: `TUNNEL` (zufliegende Wireframe-Ringe) → `RING` (Doppelring + Bass-
Kugel + Säule, Kamera-Punch/Shake) → `PLASMA` (audio-reaktives Würfel-Terrain).
Dazu durchgehend ein 2D-Overlay: FFT-Spektrum (`BOXES`-Bulk, oben+unten),
Glow-Funken + Cyber-Regen (zwei Partikelsysteme), pulsierender Titel,
Laufschrift, dezenter Beat-Flash. Nur nativ:
`gbrun.py --native examples\85_cybermatic_demo.gb` (oder F6).

Das Musik-Asset (~15 MB, „Cybermatic pulse" von **Alexandr Zhelanov**,
CC-BY 4.0) liegt **nicht** im Repo (zu groß) — einmalig holen mit
`py examples/assets/download_cybermatic.py`. Die Demo läuft auch ohne (stumm,
via `FILEEXISTS`-Guard). Provenienz/Lizenz: `examples/assets/CREDITS_cybermatic.txt`.

## Schritt 7: Standalone-Export (`gbrun.py --export` / Editor)

Ein GameBasic-Programm zu einer eigenständigen `.exe` bündeln, die **ohne
Python** läuft — Spiele ausliefern ohne Toolchain beim Endnutzer.

**Prinzip (kein Recompile):** Der kompilierte Bytecode (`.gbc`) wird an eine
Kopie von `gbrt.exe` **angehängt**. `gbrt` erkennt beim Start den Payload und
führt ihn aus. Das Anhängen von Daten ans Ende einer PE-`.exe` bricht sie nicht
(gleiches Prinzip wie PyInstaller-onefile) — der PE-Loader ignoriert Trailing-
Bytes. Layout der **letzten 16 Bytes** der gebundelten Exe:

```
[u64 Länge der .gbc-Bytes, little-endian][8 Byte Magic "GBRTPAY1"]
```

Die `.gbc`-Bytes liegen direkt vor diesem Footer.

**Runtime-Seite** ([main.rs](../rust/gb_runtime/src/main.rs)): `embedded_gbc()`
liest die eigene Exe (`current_exe()`), prüft den Magic-Footer und extrahiert den
Bytecode. Ist ein Payload da (Bundle-Modus), wechselt `gbrt` ins Exe-Verzeichnis
(damit relative Asset-Pfade beim Doppelklick von überall stimmen) und führt den
eingebetteten Bytecode aus. Ohne Payload bleibt der Dev-Modus (`gbrt datei.gbc`).
Beide Pfade teilen sich `run_gbc_text(text, label)`.

**Export-Seite** ([gamebasic/export.py](../gamebasic/export.py)):
`export_standalone(src_gb, gbrt_path, out_dir)` kompiliert in-memory zu `.gbc`,
hängt `<gbc><len><magic>` an die Runtime-Bytes und schreibt `<out>/<name>.exe`.
Der `assets/`-Ordner neben der Quelle wird mitkopiert (Konvention für
`LOADIMAGE("assets/…")` & Co.).

**Aufruf:**
```
.venv\Scripts\python.exe gbrun.py --export examples\89_heightmap.gb [ausgabe-ordner]
```
Default-Ausgabe: `<quelle>_dist/`. Im **Editor**: Menü *Ausführen → Export →
standalone .exe* bzw. **Ctrl+F6** (Toolbar-Button neben Run/Bench) — bündelt die
aktive Datei und öffnet den Ausgabeordner.

Verifiziert: `89_heightmap.gb` exportiert (~3.7 MB Exe), die Exe **ohne
Argumente aus fremdem Verzeichnis** gestartet, lädt das mitkopierte
`assets/heightmap.png` und rendert das Terrain (Screenshot). Dev-Modus
(`--native`, Konsolen-Programme bit-identisch) bleibt unverändert.

**Asset-Bündelung:** Der Export kopiert (a) einen `assets/`-Ordner neben der
Quelle wie gehabt UND (b) **jede im Quelltext als String-Literal referenzierte
Datei**, die relativ zur Quelle existiert — auch über `../` (z. B. ein Spiel in
`code/` mit `LOADIMAGE("../assets/sprites/x.png")` und Assets in `../assets/`).
Solche Pfade werden mit abgestreiftem `../` ins Bundle gelegt (`assets/sprites/x.png`).
Zur Laufzeit findet `resolve_asset_path` (builtins.rs) die gebündelte Kopie:
existiert der Original-Pfad nicht, werden führende `../` abgestreift und erneut
gesucht — greift bei LOADIMAGE/LOADSOUND/PLAYMUSIC/SHADER_LOAD/LOADMODEL/
TILED_LOAD/LOAD_ASSETS/ATLAS_LOAD/FILEEXISTS/LIGHT_ENV_HDR. Im Dev-Modus (Original
existiert) ändert sich nichts. Absolute Pfade werden nicht gebündelt.

**Grenzen:** Nur String-**Literale** werden erkannt (zur Laufzeit
zusammengesetzte Pfade nicht — dann Assets manuell in den Ausgabeordner kopieren).
Die `.gbc` ist unkomprimiert eingebettet (JSON); die Exe-Größe entspricht
`gbrt` + Bytecode. Cross-Compiling ist nicht vorgesehen — der Export bündelt das
`gbrt` der aktuellen Plattform.
