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
- [ ] 30 Konsole & Ein-/Ausgabe
- [ ] 31 Mathematik
- [ ] 32 Zufall
- [ ] 33 Zeichenketten-Funktionen
- [ ] 34 Typumwandlung & Prüfung
- [ ] 35 Array-Helfer (SORT/PUSH/POP/...)
- [ ] 36 Map-Helfer
- [ ] 37 Zeit & Datum
- [ ] 38 Dateien

### Teil IV — Grafik, Sound & Spiele
- [ ] 40 Das Fenster (SCREEN/FLIP/DELTA/FPS/Game-Loop)
- [ ] 41 2D-Zeichnen (PLOT/LINE/BOX/RECT/CIRCLE/TEXT)
- [ ] 42 2D-Extras (LINEW/BOXROUND/GRADIENT/SPLINE/BLEND/GenTex/Render-Targets)
- [ ] 43 Bilder (LOADIMAGE/DRAWIMAGE/DRAWIMAGEPART/...)
- [ ] 44 Farben (RGB/HSV/COLOR_LERP)
- [ ] 45 Eingabe (Tastatur/Maus/Gamepad)
- [ ] 46 Sound (LOADSOUND/PLAYSOUND/PLAYMUSIC/AUDIO_*)
- [ ] 47 Layer, Sprite-Atlas, Bulk-Draws
- [ ] 48 3D-Grafik (g3d)

### Teil V — Die Module
- [ ] 50 sprite · 51 animfsm · 52 tween · 53 timer · 54 particles
- [ ] 55 physics / physics2d / physics3d · 56 camera · 57 input · 58 ui · 59 gui
- [ ] 60 scene · 61 save · 62 astar · 63 tiled · 64 tile_collide · 65 controller
- [ ] 66 vec2 · 67 m3d · 68 json · 69 db · 70 regex · 71 audio (erweitert)
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
IMPORT ... AS Alias). **TEIL II KOMPLETT (Kap 10-26).** 87 Seiten, alle Ausgaben gegen
gbrt verifiziert, per LibreOffice gerendert. Ab Kap 17 auf User-Wunsch schwierige Themen ausführlicher.
**Nächstes:** Teil III — Kap 30 (Konsole & Ein-/Ausgabe), Referenzteil.
