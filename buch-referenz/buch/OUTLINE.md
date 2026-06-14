# GameBasic – Das Lehrbuch  ·  Gliederung & Fortschritt

Vollständiges Lehr- und Referenzbuch: lehrt Programmieren in GameBasic von Grund
auf UND erklärt jeden Befehl mit kleinem Beispielprogramm. Ausgabe: editierbares
`.docx` zum Drucken. Code wird durchgehend monospace im grauen Kasten mit blauer
Leiste dargestellt (Helfer `code`), Programm-Ausgabe im grünen Kasten.

## Build
- `node build_book.js` → `GameBasic-Lehrbuch.docx` (nutzt zuletzt gemessene ToC-Seiten).
- `<venv>\python.exe make_book.py` → Zwei-Pass-Build mit korrekten ToC-Seitenzahlen
  (LibreOffice→PDF→PyMuPDF misst Seiten). Vorschau-PNG: LibreOffice→PDF→fitz.

## Architektur
- `build_book.js` = fester Renderer + Bausteine `H` (h1/chapter/part/h2/p/pmix/
  bullet/bulletRich/code/cmd/tip/note/warn/figure).
- `content/NN_*.js` = je ein Kapitel, exportiert `(H) => [bloecke]`. Reihenfolge =
  Dateiname-Sortierung. **Neue Kapitel: einfach content/NN_*.js anlegen.**
- `cmd(name, syntax, desc, codeLines, {out, fig, caption})` = Standard-Befehlseintrag.
- Quellen fürs Befehlswissen: `gamebasic/editor_qt/builtin_index.json` (Signaturen),
  `gamebasic/editor_qt/builtin_docs.py` (Kurzbeschreibungen), `docs/*.md` (Prosa),
  `examples/*.gb`. Beispiele möglichst mit `gbrt run` verifizieren (Konsolen-Ausgabe).
- Screenshots für Grafik: `GBRT_FRAMES=N GBRT_SCREENSHOT=images/x.png gbrt run datei.gb`,
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
- [x] 66 vec2 (content/66_vec2.js, images/66_vec2.png) · [x] 67 m3d (content/67_m3d.js, images/67_m3d.png) · [ ] 68 json · 69 db · 70 regex · 71 audio (erweitert)
- [ ] 72 curves · 73 net · 74 html · 75 ecs · 76 serial/usb/wifi/bt

### Anhang
- [ ] A Befehls-Index (alphabetisch) · B Tastencodes · C Farb-Konstanten
- [ ] D Fehlermeldungen verstehen

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
Verweis auf Kap 16 fürs Tutorial). 100 Seiten, alle Ausgaben gegen gbrt verifiziert.
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
(figures/40_fenster.gb → images/40_fenster.png). 130 Seiten, gegen gbrt verifiziert.
Screenshot-Workflow: figures/NN_*.gb (Quelle) → `GBRT_FRAMES=N GBRT_SCREENSHOT=<ABS-Pfad>/images/x.png`
(absoluter Pfad nötig, gbrt chdirt ins figures/-Dir!) → H.figure("x.png", caption).
+ Kap 41 (2D-Zeichnen: PLOT/LINE/BOX/RECT/CIRCLE/ELLIPSE/TRIANGLE/POLYGON/ARC/TEXT/
TEXTROT) — mit Formen-Übersichts-Screenshot. GOTCHA dokumentiert: gefüllte TRIANGLE/
POLYGON nur bei CCW-Wicklung sichtbar (raylib-Culling); Engine-Fix als Task gespawnt.
+ Kap 42 (2D-Extras: LINEW, BOXROUND/RECTROUND, GRADIENTV/H, SPLINE, BLEND_MODE,
GENTEX_PERLIN/GRADIENT/CHECKED/COLOR/RADIAL, RENDERTARGET_*) — 2 Screenshots. 
+ Kap 43 (Bilder: LOADIMAGE/IMAGEWIDTH/HEIGHT/DRAWIMAGE/DRAWIMAGEPART/DRAWIMAGEROT +
imgfx IMAGE_SCALE/FLIP/ROTATE/TINT) — Screenshot mit Spritesheet. 144 Seiten.
figures/assets/ enthält hero.png+coin.png (aus examples/mario kopiert) + held1.png (Frame 0).
+ Kap 44 (Farben: RGB, RED/GREEN/BLUE-Extraktion, Konstanten, HSV, COLOR_LERP) — Screenshot
mit Palette/HSV-Regenbogen/COLOR_LERP. WICHTIG: gbrt-Hex = `&H` (NICHT `0x`!).
+ Kap 45 (Eingabe: KEYPRESSED/KEY_*-Konstanten, MOUSEX/Y/BUTTON/WHEEL/MOUSE_VISIBLE/LOCK,
input-Modul BIND/UPDATE/HELD/PRESSED/RELEASED/AXIS, Gamepad-Tipp) — Steuerungs-Screenshot.
151 Seiten.
+ Kap 46 (Sound: LOADSOUND/PLAYSOUND/STOPSOUND, PLAYMUSIC, audio-Modul AUDIO_TONE/NOISE/SFX,
UNLOADSOUND, AUDIO_VOLUME/PAN/BUS_VOLUME erwähnt) — als GRAFISCHES Mini-Klavier-Demo gebaut
(Tasten 1-8 = C-Dur), Screenshot zeigt die Klaviatur. 154 Seiten.
+ Kap 47 (Layer LAYER_DEFINE/LAYER/END/CLEAR, Atlas ATLAS_LOAD/DRAW/DRAW_FLIPPED/BATCH_DRAW/
FLUSH, Bulk PLOTS/CIRCLES/BOXES/LINES) — Layer+Bulk-Screenshot (Sternenfeld+Planeten+HUD).
GOTCHA: ATLAS_DRAW_FLIPPED-Flips brauchen 1/0 (NICHT TRUE/FALSE → „erwartet Zahl"). 158 Seiten.
**ATLAS_DRAW_FLIPPED-Inkonsistenz danach in gbrt BEHOBEN** (flip_x/flip_y TRUE/FALSE+1/0, echtes
flip_y, tint=Arg7; commit 8aa315f); Kap-47-Beispiel auf TRUE/FALSE umgestellt.
+ Kap 48 (3D-Grafik g3d: CAMERA3D, GRID3D, CUBE/_WIRES, SPHERE/_WIRES, CYLINDER/Kegel, PLANE/
LINE3D/POINT3D + Ausblick Modelle/Licht/Picking) — 3D-Szenen-Screenshot. **TEIL IV KOMPLETT
(Kap 40-48).** 163 Seiten.
+ Kap 49 ABSCHLUSSPROJEKT „Münzfang" (content/49_projekt.js, images/49_catch.png) — komplettes
spielbares Arcade-Spiel (fallende Münzen mit Korb fangen) in 5 Schritten aufgebaut: Fenster+
Steuerung, Arrays für Münzen, Fang-Kollision, AUDIO_TONE-Sound, Score-HUD; volles Listing +
Erweiterungsideen. Führt Game-Loop/Input/DELTA/Zeichnen/Arrays/Kollision/Sound/HUD zusammen.
168 Seiten. Spiel gegen gbrt verifiziert (läuft).
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
**Nächstes:** Kap 68 json / 69 db / 70 regex … (Teil V durch).
