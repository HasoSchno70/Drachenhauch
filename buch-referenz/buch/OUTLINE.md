# GameBasic – Das Lehrbuch  ·  Gliederung & Fortschritt

Vollständiges Lehr- und Referenzbuch: lehrt Programmieren in GameBasic von Grund
auf UND erklärt jeden Befehl mit kleinem Beispielprogramm. Ausgaben: editierbares
`.docx` zum Drucken und ein `.epub` zum Lesen am Gerät. Code wird durchgehend monospace im grauen Kasten mit blauer
Leiste dargestellt (Helfer `code`), Programm-Ausgabe im grünen Kasten.

## Build
- `node build_book.js` → `GameBasic-Lehrbuch.docx` (nutzt zuletzt gemessene ToC-Seiten).
- `<venv>\python.exe make_book.py` → Zwei-Pass-Build mit korrekten ToC-Seitenzahlen
  (LibreOffice→PDF→PyMuPDF misst Seiten). Vorschau-PNG: LibreOffice→PDF→fitz.
  **Achtung:** rendert das PDF nur ZWISCHENDURCH (zum Messen) und endet beim
  `.docx` — für ein PDF mit Seitenzahlen danach `make_book.render()` nachziehen.
- `node build_epub.js` → `GameBasic-Lehrbuch.epub` (EPUB 3, ein XHTML je Kapitel,
  nav.xhtml + NCX, Nachtmodus). Prüfung: `pytest tests/test_build_epub.py`.

## Architektur
- **Zwei Renderer, ein Inhalt.** `build_book.js` (→ .docx) und `build_epub.js`
  (→ .epub) stellen jeweils die Bausteine `H` bereit (h1/chapter/part/h2/p/pmix/
  bullet/bulletRich/code/cmd/tip/note/warn/figure/table/smallLabel/sig); die
  Kapitel bekommen sie injiziert und wissen nicht, wohin sie gesetzt werden.
  **Ein Kapitel darf deshalb NIE selbst `require("docx")` aufrufen** — dann kann
  der EPUB-Renderer den Block nicht darstellen. Fehlt ein Baustein, kommt er in
  BEIDE Renderer.
- Kein docx→epub-Konverter: das .docx ist auf A4 gesetzt, EPUB ist fließend.
- `content/NN_*.js` = je ein Kapitel, exportiert `(H) => [bloecke]`. Reihenfolge =
  Dateiname-Sortierung. **Neue Kapitel: einfach content/NN_*.js anlegen.**
- `cmd(name, syntax, desc, codeLines, {out, fig, caption})` = Standard-Befehlseintrag.
- Quellen fürs Befehlswissen: `drachenhauch/editor_qt/builtin_index.json` (Signaturen),
  `drachenhauch/editor_qt/builtin_docs.py` (Kurzbeschreibungen), `docs/*.md` (Prosa),
  `examples/*.gb`. Beispiele möglichst mit `dhrt run` verifizieren (Konsolen-Ausgabe).
- Screenshots für Grafik: `DHRT_FRAMES=N DHRT_SCREENSHOT=images/x.png dhrt run datei.gb`,
  PNG nach `buch-referenz/buch/images/`.

## Gliederung & Fortschritt
Legende: [x] fertig · [~] angefangen · [ ] offen

### Teil I — Erste Schritte
- [x] 00 Vorwort + Willkommen  (content/00_vorwort.js)
- [x] 01 Was ist GameBasic?  (content/01_was_ist.js)
- [x] 02 Installation, Editor & Programme starten  (content/02_start.js)
- [x] 03 Dein erstes Programm  (content/03_erstes_programm.js)

### Teil II — Die Sprache
- [x] 10 Variablen & Datentypen  (content/10_variablen.js)
- [x] 11 Operatoren & Ausdrücke  (content/11_operatoren.js)
- [x] 12 Ein-/Ausgabe: PRINT, INPUT, f-Strings  (content/12_ein_ausgabe.js)
- [x] 13 Verzweigungen: IF/ELSEIF/ELSE, SELECT CASE, IIF  (content/13_verzweigungen.js)
- [x] 14 Schleifen: FOR, WHILE, REPEAT, FOR EACH, BREAK/CONTINUE  (content/14_schleifen.js)
- [x] 15 Funktionen & SUBs (Parameter/BYREF/Defaults/Named/Variadic/FUNCREF/Rekursion)  (content/15_funktionen.js)
- [x] 16 Strings im Detail  (content/16_strings.js)
- [x] 17 Arrays  (content/17_arrays.js)
- [x] 18 Maps  (content/18_maps.js)
- [x] 19 Tupel & Destructuring  (content/19_tupel.js)
- [x] 20 Klassen & Objekte  (content/20_klassen.js)
- [x] 21 Vererbung, Properties, Operatoren, Static  (content/21_oop_fortgeschritten.js)
- [x] 22 ENUM  (content/22_enum.js)
- [x] 23 Comprehensions (List/Dict/Set)  (content/23_comprehensions.js)
- [x] 24 Fehlerbehandlung (TRY/CATCH/THROW)  (content/24_fehlerbehandlung.js)
- [x] 25 Coroutinen (YIELD)  (content/25_coroutinen.js)
- [x] 26 Module importieren (IMPORT)  (content/26_module.js)

### Teil III — Eingebaute Befehle (Referenz)
- [x] 30 Konsole & Ein-/Ausgabe  (content/30_konsole.js)
- [x] 31 Mathematik  (content/31_mathematik.js)
- [x] 32 Zufall  (content/32_zufall.js)
- [x] 33 Zeichenketten-Funktionen  (content/33_strings_ref.js)
- [x] 34 Typumwandlung & Prüfung  (content/34_typen.js)
- [x] 35 Array-Helfer (SORT/PUSH/POP/...)  (content/35_array_helfer.js)
- [x] 36 Map-Helfer  (content/36_map_helfer.js)
- [x] 37 Zeit & Datum  (content/37_zeit.js)
- [x] 38 Dateien  (content/38_dateien.js)

### Teil IV — Grafik, Sound & Spiele
- [x] 40 Das Fenster (SCREEN/FLIP/DELTA/FPS/Game-Loop)  (content/40_fenster.js, images/40_fenster.png)
- [x] 41 2D-Zeichnen (PLOT/LINE/BOX/RECT/CIRCLE/TEXT)  (content/41_zeichnen.js, images/41_formen.png)
- [x] 42 2D-Extras (LINEW/BOXROUND/GRADIENT/SPLINE/BLEND/GenTex/Render-Targets)  (content/42_extras.js, images/42_extras.png + 42_blend.png)
- [x] 43 Bilder (LOADIMAGE/DRAWIMAGE/DRAWIMAGEPART/...)  (content/43_bilder.js, images/43_bilder.png)
- [x] 44 Farben (RGB/HSV/COLOR_LERP)  (content/44_farben.js, images/44_farben.png)
- [x] 45 Eingabe (Tastatur/Maus/Gamepad)  (content/45_eingabe.js, images/45_eingabe.png)
- [x] 46 Sound (LOADSOUND/PLAYSOUND/PLAYMUSIC/AUDIO_*)  (content/46_sound.js, images/46_sound.png)
- [x] 47 Layer, Sprite-Atlas, Bulk-Draws  (content/47_layer_atlas.js, images/47_layer.png)
- [x] 48 3D-Grafik (g3d)  (content/48_3d.js, images/48_3d.png)
- [x] 49 Abschlussprojekt: Münzfang (Grafik+Sound-Spiel)  (content/49_projekt.js, images/49_catch.png)

### Teil V — Die Module
- [x] 50 sprite  (content/50_sprite.js, images/50_sprite.png) · [x] 51 animfsm  (content/51_animfsm.js, images/51_animfsm.png) · [x] 52 tween  (content/52_tween.js, images/52_tween.png) · [x] 53 timer  (content/53_timer.js, konsolen-Demo) · [x] 54 particles  (content/54_particles.js, images/54_particles.png)
- [x] 55 physics / physics2d / physics3d  (content/55_physics.js, images/55_physics2d.png) · [x] 56 camera  (content/56_camera.js, images/56_camera.png) · [x] 57 input (Kurzref/Querverweis Kap 45)  (content/57_input.js) · [x] 58 ui  (content/58_ui.js, images/58_ui.png) · [x] 59 gui  (content/59_gui.js, images/59_gui.png)
- [x] 60 scene (content/60_scene.js) · [x] 61 save (content/61_save.js) · [x] 62 astar (content/62_astar.js, images/62_astar.png) · [x] 63 tiled (content/63_tiled.js, images/63_tiled.png) · [x] 64 tile_collide (content/64_tile_collide.js, images/64_tilecollide.png) · [x] 65 controller (content/65_controller.js, images/65_controller.png)
- [x] 66 vec2 (content/66_vec2.js, images/66_vec2.png) · [x] 67 m3d (content/67_m3d.js, images/67_m3d.png) · [x] 68 json (content/68_json.js) · [x] 69 db (content/69_db.js) · [x] 70 regex (content/70_regex.js) · [x] 71 audio erweitert (content/71_audio.js)
- [x] 72 curves (content/72_curves.js, images/72_curves.png) · [x] 73 net (content/73_net.js) · [x] 74 html (content/74_html.js) · [x] 75 ecs (content/75_ecs.js) · [x] 76 serial/usb/wifi/bt (content/76_hardware.js) — **Teil V KOMPLETT**

### Teil VI — Eine Demo bauen
- [x] 80 Was eine Demo ist – und was sie zusammenhält  (content/80_demo.js, images/80_demo_titel.png)
      — AUDIO_MUSIC_POSITION als Uhr, AUDIO_FFT als Spektrum, Ablaufplan aus
      Sekunden, Ein-/Ausblenden, Schlagerkennung (roh vs. geglättet!), ein
      Shader für alle Szenen (POSTFX + mode-Uniform).
- [x] 81 Die acht Szenen – und was in jeder steckt  (content/81_demo_szenen.js,
      images/81_demo_sterne|wuerfel|pbr|physik|tunnel|klang|abspann.png)
      — Scroller, LINES-Bulk, MODEL_INSTANCED mit Farb-Array, PBR+HDR+SKYBOX,
      physics2d mit PHYS2D_SET_DYNAMIC, behaltenes RENDERTARGET, Effektkette,
      Spatial Audio, AUDIO_SFX-Klangtastatur. Jede Szene mit ihrem Stolperstein.
- [x] 82 Ins Netz stellen – GameBasic im Browser  (content/82_web.js,
      images/82_playground.png)
      — build_wasm.py + http.server, Playground (Galerie, ▶ Demo, Link teilen),
      Tabelle „Was im Browser anders ist" (Ton erst nach Klick, Leinwand nach
      Programmende schwarz, keine Datei-Dialoge, keine Hardware-Module), FLIP als
      Atempause für den Tab, Touch fürs Handy, „Größe ist Wartezeit“
      (rechnen statt laden). Buch damit 392 Seiten.

### Anhang
- [x] A Befehls-Index (alphabetisch)  (content/90_anhang_a.js — auto-generiert aus builtin_index.json)
- [x] B Tastencodes  (content/91_anhang_b.js) · [x] C Farb-Konstanten  (content/92_anhang_c.js)
- [x] D Fehlermeldungen verstehen  (content/93_anhang_d.js)

## Status
Session 1 (2026-06-13): Pipeline + Renderer + Teil I komplett + Teil II Kap 10–11.
Session 2 (2026-06-13): Teil II Kap 12 (Ein-/Ausgabe) + Kap 13 (Verzweigungen:
IF/ELSEIF/ELSE inkl. einzeilig, SELECT CASE mit Liste/TO/IS/WHERE-Guard, IIF) +
Kap 14 (Schleifen) + Kap 15 (Funktionen & SUBs) + Kap 16 (Strings) + Kap 17 (Arrays:
DIM[n], 0-Index+Bounds, LEN/Iteration/FOR EACH, SORT/REVERSE/INDEXOF, Aggregate
SUM/AVG/MIN/MAX/FILL, dynamisch PUSH/POP/INSERT/REMOVE_AT/REDIM, Slicing,
**Alias-Stolperstein b=a kopiert NICHT → ARRAY_COPY**, mehrdimensional) + Kap 18 (Maps: MAPPUT/GET/GETOR/HAS/IN/SIZE/REMOVE/CLEAR,
FOR EACH über Keys, MAPKEYS/VALUES/ITEMS, Methoden-Syntax, STR$-Keys/Cache,
Alias-Hinweis) + Kap 19 (Tupel & Destructuring: Literal (a,b,...), Index/length,
unveränderlich, Destructuring (a,b)=tupel, mehrere Rückgabewerte, Tausch-Trick,
FOR EACH) + Kap 20 (Klassen & Objekte: Bauplan/Objekt-Modell, CLASS+DIM-Felder,
NEW, SUB Init/Self, Methoden SUB/FUNCTION, impliziter Methodenaufruf, viele
unabhängige Objekte, Verweis-Semantik) + Kap 21 (Vererbung EXTENDS/Überschreiben/
Polymorphie, Properties GET/SET, Operator-Überladung OPERATOR, STATIC CONST).
+ Kap 22 (ENUM: kompakt/Block, Auto-Nummerierung, eigene Werte, gemischt,
SELECT CASE, Keyword-Member) + Kap 23 (Comprehensions: List `[e FOR v IN s]` +WHERE,
Dict `{k:v FOR...}`→MAP, Set `{e FOR...}`→dedup-TUPLE) + Kap 24 (Fehlerbehandlung:
TRY/CATCH[e]/END TRY, THROW, wann nutzen + TRYVAL-Hinweis) + Kap 25 (Coroutinen:
YIELD, CORO_RESUME/DONE/RESULT, FOR EACH, CORO_SEND zweiweg, CORO_CLOSE) + Kap 26
(Module: eingebaut IMPORT "x" OHNE Endung, eigene Datei IMPORT "x.gb" MIT Endung,
IMPORT ... AS Alias). **TEIL II KOMPLETT (Kap 10-26).** Danach: ToC-Seitenzahlen-Bug behoben
(measure() erkennt Überschriften per Schriftgröße ≥15 + monotone Suche) und
mehrzeilige cmd-Syntax (\n) rendert jetzt als echte Zeilenumbrüche. **Teil III
gestartet:** Kap 30 (Konsole & Ein-/Ausgabe: PRINT/INPUT/FORMAT$, Verweise auf
Teil II + Teil IV). + Kap 31 (Mathematik: ABS/SQR/POW/HYPOT/EXP/LOG, FLOOR/CEIL/ROUND, MIN/MAX/CLAMP/
SIGN, LERP/REMAP, SIN/COS/TAN/ATAN2/DEG/RAD, WRAP/PINGPONG/MOVETOWARD, PI/TAU).
+ Kap 32 (Zufall: RND/RANDINT/RANDF, CHOICE/WEIGHTED_CHOICE, SHUFFLE, RANDOMIZE-Seed).
+ Kap 33 (Zeichenketten-Funktionen: kompakte bulletRich-Referenz aller String-Fns,
Verweis auf Kap 16 fürs Tutorial). 100 Seiten, alle Ausgaben gegen dhrt verifiziert.
Ab Kap 17 auf User-Wunsch schwierige Themen ausführlicher; Referenzkapitel
+ Kap 34 (Typumwandlung & Prüfung) + Kap 35 (Array-Helfer: LEN/DIMCOUNT/DIMSIZE,
SORT/REVERSE/INDEXOF, SUM/AVG/MIN/MAX/FILL/COPY, PUSH/POP/INSERT/REMOVE_AT/REDIM).
**NEUE VORGABE ab hier (User 2026-06-13): jeder Befehl bekommt mind. EIN eigenes
Code-Beispiel** — Kap 33 (Strings-Ref) + Kap 34-Encoding deshalb von kompakt auf
cmd+Beispiel nachgerüstet. + Kap 36 (Map-Helfer: MAPPUT/GET/GETOR/HAS/SIZE/REMOVE/
CLEAR/KEYS/VALUES/ITEMS + Methoden-Syntax) + Kap 37 (Zeit & Datum: MILLIS, SLEEP,
TIME$, DATE$; nicht-deterministische Werte als Kommentar statt Ausgabe-Kasten).
+ Kap 38 (Dateien: OPENFILE/CLOSEFILE/WRITELINE/WRITE/READLINE/ENDOFFILE/READALL$,
WRITEALL/READLINES/APPENDFILE/FILESIZE, FILEEXISTS/COPYFILE/RENAME/DELETEFILE,
MKDIR/DIREXISTS/DIRLIST, PATHJOIN/BASENAME/DIRNAME). **TEIL III KOMPLETT (Kap 30-38).**
**Teil IV gestartet (Grafik/Sound/Spiele):** Kap 40 (Das Fenster & Game-Loop: SCREEN/CLS/
FLIP/QUITREQUESTED-Loop, DELTA, FPS/SETFPS, SCREENWIDTH/HEIGHT) — MIT echtem Screenshot
(figures/40_fenster.gb → images/40_fenster.png). 130 Seiten, gegen dhrt verifiziert.
Screenshot-Workflow: figures/NN_*.gb (Quelle) → `DHRT_FRAMES=N DHRT_SCREENSHOT=<ABS-Pfad>/images/x.png`
(absoluter Pfad nötig, dhrt chdirt ins figures/-Dir!) → H.figure("x.png", caption).
+ Kap 41 (2D-Zeichnen: PLOT/LINE/BOX/RECT/CIRCLE/ELLIPSE/TRIANGLE/POLYGON/ARC/TEXT/
TEXTROT) — mit Formen-Übersichts-Screenshot. GOTCHA dokumentiert: gefüllte TRIANGLE/
POLYGON nur bei CCW-Wicklung sichtbar (raylib-Culling); Engine-Fix als Task gespawnt.
+ Kap 42 (2D-Extras: LINEW, BOXROUND/RECTROUND, GRADIENTV/H, SPLINE, BLEND_MODE,
GENTEX_PERLIN/GRADIENT/CHECKED/COLOR/RADIAL, RENDERTARGET_*) — 2 Screenshots. 
+ Kap 43 (Bilder: LOADIMAGE/IMAGEWIDTH/HEIGHT/DRAWIMAGE/DRAWIMAGEPART/DRAWIMAGEROT +
imgfx IMAGE_SCALE/FLIP/ROTATE/TINT) — Screenshot mit Spritesheet. 144 Seiten.
figures/assets/ enthält hero.png+coin.png (aus examples/mario kopiert) + held1.png (Frame 0).
+ Kap 44 (Farben: RGB, RED/GREEN/BLUE-Extraktion, Konstanten, HSV, COLOR_LERP) — Screenshot
mit Palette/HSV-Regenbogen/COLOR_LERP. WICHTIG: dhrt-Hex = `&H` (NICHT `0x`!).
+ Kap 45 (Eingabe: KEYPRESSED/KEY_*-Konstanten, MOUSEX/Y/BUTTON/WHEEL/MOUSE_VISIBLE/LOCK,
input-Modul BIND/UPDATE/HELD/PRESSED/RELEASED/AXIS, Gamepad-Tipp) — Steuerungs-Screenshot.
151 Seiten.
+ Kap 46 (Sound: LOADSOUND/PLAYSOUND/STOPSOUND, PLAYMUSIC, audio-Modul AUDIO_TONE/NOISE/SFX,
UNLOADSOUND, AUDIO_VOLUME/PAN/BUS_VOLUME erwähnt) — als GRAFISCHES Mini-Klavier-Demo gebaut
(Tasten 1-8 = C-Dur), Screenshot zeigt die Klaviatur. 154 Seiten.
+ Kap 47 (Layer LAYER_DEFINE/LAYER/END/CLEAR, Atlas ATLAS_LOAD/DRAW/DRAW_FLIPPED/BATCH_DRAW/
FLUSH, Bulk PLOTS/CIRCLES/BOXES/LINES) — Layer+Bulk-Screenshot (Sternenfeld+Planeten+HUD).
GOTCHA: ATLAS_DRAW_FLIPPED-Flips brauchen 1/0 (NICHT TRUE/FALSE → „erwartet Zahl"). 158 Seiten.
**ATLAS_DRAW_FLIPPED-Inkonsistenz danach in dhrt BEHOBEN** (flip_x/flip_y TRUE/FALSE+1/0, echtes
flip_y, tint=Arg7; commit 8aa315f); Kap-47-Beispiel auf TRUE/FALSE umgestellt.
+ Kap 48 (3D-Grafik g3d: CAMERA3D, GRID3D, CUBE/_WIRES, SPHERE/_WIRES, CYLINDER/Kegel, PLANE/
LINE3D/POINT3D + Ausblick Modelle/Licht/Picking) — 3D-Szenen-Screenshot. **TEIL IV KOMPLETT
(Kap 40-48).** 163 Seiten.
+ Kap 49 ABSCHLUSSPROJEKT „Münzfang" (content/49_projekt.js, images/49_catch.png) — komplettes
spielbares Arcade-Spiel (fallende Münzen mit Korb fangen) in 5 Schritten aufgebaut: Fenster+
Steuerung, Arrays für Münzen, Fang-Kollision, AUDIO_TONE-Sound, Score-HUD; volles Listing +
Erweiterungsideen. Führt Game-Loop/Input/DELTA/Zeichnen/Arrays/Kollision/Sound/HUD zusammen.
168 Seiten. Spiel gegen dhrt verifiziert (läuft).
Buch-Kerninhalt (Teile I-IV + Projekt) komplett. **Teil V gestartet:** Kap 50 sprite
(SPRITE_NEW/SET_POS/VELOCITY/ADD_ANIM/PLAY/PLAY_ONCE/SET_FLIP/SET_SCALE/UPDATE/DRAW/COLLIDES) mit
animiertem Screenshot. 173 Seiten. GOTCHA: SPRITE_UPDATE will dt als INTEGER-ms (INT(DELTA()*1000)).
+ Kap 51 animfsm (ANIM_FSM_LOAD/SETUP/SET_FLOAT-BOOL-INT/TRIGGER/UPDATE/STATE/FORCE; .gbanim-JSON
States/Params/Transitions) — Screenshot zeigt aktuellen Zustand. figures/assets/held.gbanim. 177 Seiten.
+ Kap 52 tween (TWEEN_NEW/_LOOP/_PINGPONG, TWEEN_VALUE/DONE, PAUSE/RESUME/REVERSE, Easings) —
Easing-Balken-Screenshot. zeitbasiert (kein UPDATE nötig). 180 Seiten.
+ Kap 53 timer (TIMER_AFTER/EVERY/UPDATE/CANCEL/ACTIVE/COUNT/CLEAR + COOLDOWN; FUNCREF-Callbacks)
— konsolentauglich, kein Screenshot, Konsolen-Beispiele verifiziert. 183 Seiten.
+ Kap 54 particles (PARTICLE_SYSTEM_NEW/SET_VELOCITY/GRAVITY/LIFETIME/SIZE/COLOR/COLOR_END/MODE/
FADE/EMIT/UPDATE/DRAW/COUNT) — Funken-Fontäne-Screenshot (Glow, gelb→rot). 187 Seiten.
+ Kap 55 physics/physics2d/physics3d (Kollisions-Mathe PHYSICS_*, echte 2D-/3D-Engine
PHYS2D_*/PHYS3D_NEW/ADD_BOX/ADD_SPHERE/STEP/BODY_*) — Bälle-Stapel-Screenshot.
**Engine-Fix nebenbei:** PHYS3D_ADD_BOX/ADD_SPHERE dynamic-Flag akzeptiert jetzt TRUE/FALSE
(vorher nur 1/0; need_flag statt need_num — konsistent mit physics2d). 192 Seiten.
+ Kap 56 camera (CAMERA_SET/FOLLOW/RESET/S2W_X-Y/SHAKE; Welt- vs. Bildschirm-Koord., HUD nach
RESET) — Kamera-folgt-Spieler-Screenshot. 195 Seiten.
+ Kap 57 input — KURZES Recap/Querverweis auf Kap 45 (war dort schon ausführlich), kompakte
Referenz + Grundmuster, kein Screenshot. 196 Seiten.
+ Kap 58 ui (Immediate-Mode: UI_PANEL/LABEL/BUTTON/CHECKBOX/SLIDER, UI_END_FRAME Pflicht vor FLIP)
— Einstellungs-Panel-Screenshot. GOTCHA: Variable nicht `sound` nennen (Typ-Keyword SOUND). 197 S.
+ Kap 59 gui (Retained-Mode: GUI_WINDOW/LABEL/BUTTON/CHECKBOX/SLIDER, GUI_UPDATE/DRAW, Polling
GUI_CLICKED/CHECKED/VALUE; Widgets einmal anlegen) — verschiebbares-Fenster-Screenshot. 204 S.
+ Kap 60 scene (SCENE_SWITCH/PUSH/POP/CURRENT/DEPTH, SCENE_SET/GET_INT-FLOAT-STRING-BOOL[_OR];
Stack + Per-Scene-Daten) — logisch, kein Screenshot, Konsole verifiziert. 205 S.
+ Kap 61 save (SAVE_NEW/LOAD_OR_NEW/EXISTS/WRITE/DELETE_FILE, SET/GET_INT-FLOAT-STRING-BOOL[_OR],
VERSION; Highscore-Muster) — logisch, kein Screenshot, Roundtrip verifiziert. 206 S.
+ Kap 62 astar (ASTAR_NEW/SET_WALL/IS_WALL/SET_DIAGONAL/FIND/PATH_LEN/PATH_X/PATH_Y; Pfad inkl.
Start+Ziel) — Gitter-mit-Pfad-Screenshot. 213 S.
+ Kap 63 tiled (TILED_LOAD/WIDTH/HEIGHT/TILE_WIDTH/AT/SET/TILE_PROP_*/OBJECT_*; Render per
DRAWIMAGEPART gid→Tileset; Bulk FILL_RECT/REPLACE/FLOOD_FILL) — Tilemap-Screenshot.
figures/levels/level1.json + figures/assets/tiles.png. 215 S.
+ Kap 64 tile_collide (TILE_SWEEP_X/Y → Tupel (neue_pos, hit), TILE_IS_SOLID; solid-Property
oder GID>0; getrennt-Achsen-Sweep, Plattformer-Muster) — Spieler-landet-Screenshot. 217 S.
+ Kap 65 controller (CHAR_NEW/SET_INPUT/UPDATE, CHAR_X/Y/VX/VY/ON_GROUND/ON_WALL, SET_MOVE_SPEED/
JUMP_VELOCITY/GRAVITY/COYOTE_TIME/JUMP_BUFFER/VARIABLE_JUMP, SET_POS/VX/VY; Coyote/Puffer/var. Sprung)
— Character-auf-Tilemap-Screenshot. 220 S.
+ Kap 66 vec2 (VEC2_NEW/X/Y/LENGTH/NORMALIZE/DOT/DISTANCE/LERP/PERP/REFLECT/ANGLE, Operatoren
+/-/*/=, immutable) — Vektor-Pfeil-Screenshot. 224 S.
+ Kap 67 m3d (VEC3/VEC4/QUAT/MAT4: VEC3_NEW/DOT/CROSS/NORMALIZE, QUAT_FROM_AXIS_ANGLE/MUL/SLERP,
MAT4_TRS/MUL/IDENTITY, MODEL_MATRIX/INSTANCED) — rotierte-Würfel-Screenshot. 225 S.
+ Kap 68 json (JSON_PARSE/LOAD, Pfad-Notation "user.name"/"hobbies.0", JSON_GET_STRING/INT/FLOAT/
BOOL, JSON_LEN/HAS/TYPE, JSON_STRINGIFY/PRETTY, TRY/CATCH bei Typfehler) — Konsole, alle Ausgaben
gegen dhrt verifiziert. + Kap 69 db (SQLite: DB_OPEN ":memory:"/Datei, DB_EXEC mit ?-Binding,
DB_LAST_ROWID, DB_QUERY/NEXT/GET_*/CLOSE_RESULT, DB_IS_NULL [NIL ist KEIN Literal → NULL per
weggelassener Spalte], DB_COL_COUNT/NAME, Transaktionen BEGIN/COMMIT/ROLLBACK) — Konsole, verifiziert.
+ Kap 70 regex (REGEX_MATCH/TEST/FIND/FIND_ALL [Capture-Gruppe extrahiert], REPLACE/REPLACE_ONCE
[\\1 im Ersatz ok], SPLIT; Backslash in GB-Strings NICHT escapen; **WARN: Rust-Regex = keine
Lookarounds/Backrefs IM Muster**) — Konsole, verifiziert. + Kap 71 audio erweitert (Kanäle
AUDIO_PLAY/STOP/PAUSE/RESUME/IS_PLAYING/SET_VOLUME/GET_VOLUME/PITCH, Panorama PAN_POS/PAN_SLIDE/
AUTOPAN, AUDIO_SFX [wellenform$ = STRING!]/LOFI, Samples SAMPLE_LOAD/PLAY/SET_LOOP/LEN, Musik
MUSIC_PAUSE/RESUME/STOP/POSITION/SET_VOLUME/PITCH, Busse BUS_VOLUME/GET_VOLUME sfx/music/master,
Effekte REVERB/DELAY/FILTER/DISTORTION/COMPRESSOR/EQ, FFT-Hinweis) — kein Screenshot (Klang nicht
abdruckbar), Aufrufe gegen dhrt geprüft. **Nächstes:** Kap 72 curves / 73 net / 74 html / 75 ecs /
76 serial-usb-wifi-bt, dann Anhang (A Befehls-Index / B Tastencodes / C Farben / D Fehlermeldungen).
+ Kap 72 curves (content/72_curves.js, images/72_curves.png) — CURVE_LERP/SMOOTHSTEP/SMOOTHERSTEP,
BEZIER/BEZIER2 (Handles), CATMULL/CATMULL2 (durch die Punkte), HERMITE; LERP-Nachzieh-Muster.
Screenshot: Bezier+Catmull-Vergleich. + Kap 73 net (content/73_net.js, Konsole) — TCP LISTEN/ACCEPT/
CONNECT/SEND/RECV/PEER_ADDR/CLOSE, UDP BIND/OPEN/SEND/RECV/LAST_FROM; non-blocking by default.
**STOLPERSTEINE: NET_TCP_ACCEPT-Leere via IS_NIL prüfen (NICHT <> NIL); NET_UDP_LAST_FROM gibt STRING
"host:port" (NICHT Tupel — docs/module-net.md war veraltet, mit-korrigiert).** TCP+UDP-Loopback gegen
dhrt verifiziert. + Kap 74 html (content/74_html.js) — HTTP_GET/POST/DOWNLOAD/STATUS/HEADER, URL_ENCODE/
DECODE, HTML_TEXT/FIND_ALL/GET_ATTR; live gegen example.com verifiziert (Status 200, h1=Example Domain),
URL/HTML-Parser offline verifiziert. + Kap 75 ecs (content/75_ecs.js, Konsole) — Mentales Modell
(Entity=ID, Component=Daten, System=Query-Loop), NEW_WORLD/ENTITY/DESTROY/ALIVE/COUNT, ADD_*/GET_*/
GET_OR_*/HAS/REMOVE, QUERY/2/3, Bulk INTEGRATE/SCALE/CLAMP/FILL/REMOVE_DEAD/COUNT_WITH (40× schneller).
Alle Ausgaben verifiziert. + Kap 76 serial/usb/wifi/bt (content/76_hardware.js) — 4 Hardware-Module in
EINEM Kapitel. **WICHTIG: brauchen Spezial-Build `python rust\\build_runtime.py --hardware` (Standard-dhrt
wirft klare Meldung) → nicht live verifizierbar, Code-Beispiele ohne out:.** serial (Arduino/COM), usb
(HID), wifi (netsh, nur Windows), bt (BLE); Returns meist STRING-Listen (SPLIT), Roh-Bytes via ASC/CHR$.
**>>> TEIL V (Module, Kap 50-76) KOMPLETT — 67 content-Module, 74 Überschriften. <<<**
+ **Anhang KOMPLETT** (neuer Renderer-Helfer `H.table(rows, {headers, widths, mono})` in build_book.js
— über Seiten umbrechende Tabelle, optional Farb-`swatch` pro Zelle): A Befehls-Index (content/90_anhang_a.js,
**auto-generiert: liest beim Build builtin_index.json → bleibt mit der Engine in Sync**, 1010 Befehle
alphabetisch nach Buchstabe, Signatur + Modul-Spalte, Kern = „—") · B Tastencodes (content/91_anhang_b.js,
KEY_*/JOY_* aus graphics.py, gegen dhrt verifiziert) · C Farb-Konstanten (content/92_anhang_c.js, 18 Farben
mit Swatch/RGB/Hex, verifiziert) · D Fehlermeldungen (content/93_anhang_d.js, 9 häufige Meldungen mit
exaktem dhrt-Wortlaut + Ursache/Lösung). Tabellen per LibreOffice-Render geprüft (Swatches/Header sauber).
**>>> BUCH-GESAMTSTRUKTUR KOMPLETT: Teile I-V (Kap 0-76) + Anhang A-D, 71 content-Module, 79 Überschriften,
289 PDF-Seiten. <<<** **Nächstes (optional, Politur): TOC/Vorwort-Feinschliff, kompletter Korrekturlauf
(Tippfehler/Konsistenz), evtl. echte dhsprites-Screenshots wo noch Platzhalter.**

**Nachtrag 2026-07-12: 89 seit der Fertigstellung (14.06.) neu hinzugekommene Builtins nachgezogen**
(per `git diff` gegen `builtin_index.json` ermittelt, siehe Memory `project_buch_referenz_update_2026_07`).
Alle Beispiele gegen frisch gebauten `dhrt` verifiziert (Standard-Build `graphics db net http`, kein
`--hardware` nötig für diese Charge). Betroffen: Kap 34 (FLT), Kap 38 (FILE_OPEN_DIALOG/SAVE_DIALOG/
FOLDER_DIALOG, native Dialoge ohne out-Kasten), Kap 40 (SCREEN_NATIVE/SCREEN_TRANSPARENT, WINDOW_ESC_QUIT/
PASSTHROUGH/TOPMOST/UNDECORATED/X/Y/SET_WINDOW_POS, neuer Monitor-Abschnitt MONITOR_COUNT/WIDTH/HEIGHT/
NAME/REFRESH/X/Y/CURRENT_MONITOR/SET_WINDOW_MONITOR), Kap 41 (CIRCLEOUTLINE), Kap 43 (GETPIXEL,
DRAWIMAGEPARTEX, 11× neue IMAGE_*-Filter: BLUR/BRIGHTNESS/CONTRAST/CROP/GRAYSCALE/INVERT/REPLACE_COLOR/
RESIZE_CANVAS + die 4 in-place-DRAW_*-Befehle), Kap 48 (CAMERA_ORBIT/PICK_MODEL/RAY_HIT_MODEL/
MODEL_ANIMATE_BLEND/WORLD_TO_SCREEN_*/SCREEN_TO_WORLD_DIR_* als Ausblick-Bullets ergänzt — Audit hatte
CAMERA_ORBIT+WORLD/SCREEN-Helfer faelschlich Kap 56 zugeordnet, gehören aber inhaltlich zu g3d/Kap 48),
Kap 59 gui (größter Brocken: Tree-View, Menüleiste+Kontextmenü, Tabs, Splitter, Spinner, Tooltip,
Textarea, Toolbar+IconButton, GUI_CONFIRM/MESSAGE ohne out-Kasten, GUI_WINDOW_SCROLLABLE — ca. 25
Builtins mit je eigenem verifiziertem Beispiel), Kap 71 audio (AUDIO_CLOCK_* komplett + räumliches Audio
AUDIO_LISTENER_*/AUDIO_EMITTER_*/AUDIO_PLAY_AT/AUDIO_PLAY_ON), Kap 73 (NET_IS_CONNECTED), Kap 76
(wifi-Beschreibung korrigiert: läuft seit der Cross-Platform-Migration auch auf Linux/macOS, nicht mehr
nur Windows). Ad-hoc-Korrekturlauf (Node-Extraktion aller (codeLines,out)-Paare aus den geänderten
content/*.js + Python-Runner gegen dhrt) bestätigt: keine echten Fehler, nur erwartete Fortsetzungs-
Fragmente (Variable aus vorigem cmd()-Block im selben Kapitel, etabliertes Muster) und der vorbestehende
`<port>`-Platzhalter in Kap 73. Zwei-Pass-Build (`make_book.py`) lief sauber durch (79/79 ToC-Seiten).
Anhang A ist auto-generiert aus builtin_index.json und zieht die neuen Builtins beim nächsten Build
automatisch nach — kein manueller Eingriff nötig.


## Durchsicht 2026-08-04 — "ist alles aktuell?"

Systematisch geprüft statt quergelesen, mit Skripten gegen die echten Quellen:

* **Veraltete Begriffe:** keine. Kein „Tree-Walker", kein `interpreter.py`,
  kein pygame/Cython/Pyodide mehr im Text.
* **Befehle im Buch, die es nicht mehr gibt:** keiner (alle 629 `cmd()`-Titel
  gegen `builtin_index.json` geprüft; die 56 „Unbekannten" waren durchweg
  Prosa-Titel wie „SAVE_SET_…").
* **Befehle, die es gibt und die nirgends erklärt waren:** 8 — alle aus der
  Arbeit dieser Woche. Ergänzt: `GFX_PUSH/POP/DEPTH` + `RENDERTARGET_CLEAR`
  und der `behalten`-Parameter (Kap 42), `AUDIO_PUSH/POP/DEPTH` (Kap 71),
  `PHYS2D_SET_DYNAMIC`/`IS_DYNAMIC` (Kap 55), Stückzahl bei den Bulk-Befehlen
  (Kap 47), `MODEL_INSTANCED` mit Farb-Array (Kap 67).
* **Signaturen:** 2 Abweichungen. `PLOTS` fehlte die Stückzahl (Buch-Fehler,
  behoben). Bei `SORT(array, absteigend)` war das **Buch im Recht und der
  Index falsch** — die Laufzeit kann absteigend sortieren, die zu enge
  Signatur erzeugte einen Falsch-Alarm der Argumentzahl-Prüfung in jedem
  korrekten Aufruf. `builtin_index.json` korrigiert.
* **Alle 260 Beispiele mit abgedruckter Ausgabe gegen `dhrt` laufen lassen:**
  233 stimmen, 24 sind Fortsetzungs-Fragmente (etabliertes Muster), 3 Treffer.
  Davon einer echt: das `SCENE_SET_INT`-Beispiel druckte „100", lief allein
  aber nicht (ohne aktive Szene bricht es ab) — Zeile ergänzt. Die anderen
  beiden sind der `<port>`-Platzhalter (Kap 73) und ein Beispiel, das eine
  eigene Datei importiert.
* **Setzfehler quer durchs Buch:** `warn(text, titel)` nimmt den Text ZUERST
  (anders als `tip(titel, text)`). Alle 14 zweiargumentigen Aufrufe waren
  title-first geschrieben — in jedem dieser Kästen stand die lange Erklärung
  fett als Überschrift und der kurze Titel klein darunter. Alle 14 gedreht,
  die Reihenfolge an der Hilfsfunktion dokumentiert.

Buch danach 396 Seiten (vorher 392).
