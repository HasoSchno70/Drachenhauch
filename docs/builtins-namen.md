# Befehlsnamen

Drachenhauch hat über 2500 eingebaute Befehle, und sie sind über Jahre
gewachsen: `LOADIMAGE` neben `IMAGE_SAVE`, `KEYPRESSED` (gehalten) neben
`INPUT_PRESSED` (gerade gedrückt), `ZEIT_JETZT` mitten in englischen Namen.
Diese Seite legt fest, wie Befehle heißen, und nennt für die alten den
**neuen Namen**.

**Die alten Namen bleiben gültig** — kein Programm, kein Beispiel und kein
Buch muss geändert werden. Neue Programme nehmen die neuen. Der Hover im
Editor nennt bei einem alten Namen den neuen, Vervollständigung und
Signaturhilfe kennen beide.

## Inhalt

- [Die Regeln](#die-regeln)
- [Der empfohlene Weg je Bereich](#der-empfohlene-weg-je-bereich)
- [Neue Namen für alte Befehle](#neue-namen-für-alte-befehle)

## Die Regeln

1. **Mit Unterstrich gegliedert, vorn der Bereich:** `BEREICH_DING_TUN` —
   `IMAGE_LOAD`, `FILE_EXISTS`, `GUI_LISTBOX_ITEMS`, `WINDOW_SET_TITLE`.
   Ausgenommen sind die Grundwörter aus BASIC: `PRINT`, `LEN`, `MID$`, `CLS`,
   `BOX`, `LINE`, `CIRCLE`, `SCREEN` und Verwandte.
2. **Setzen und Lesen:** `BEREICH_SET_EIGENSCHAFT` setzt (der Wert kommt
   zuletzt), `BEREICH_GET_EIGENSCHAFT` liest. Ein Befehl, der setzt, trägt das `SET`
   im Namen — `AUDIO_MUSIC_SET_PITCH`, nicht `AUDIO_MUSIC_PITCH(wert)`.
3. **Eingabe:** `_DOWN` heißt *gehalten* (jedes Bild wahr), `_HIT` heißt *in
   diesem Bild heruntergegangen*, `_RELEASED` *losgelassen* — für Tasten
   (`KEY_DOWN`, `KEY_HIT`), Maus (`MOUSE_DOWN`, `MOUSE_HIT`), Gamepad
   (`JOYSTICK_DOWN`, `JOYSTICK_HIT`) und Aktionen (`INPUT_DOWN`, `INPUT_HIT`).
   Das Wort `PRESSED` kommt in neuen Namen nicht vor: es hieß bei
   `KEYPRESSED` „gehalten“ und bei `INPUT_PRESSED` „gerade gedrückt“.
4. **Wörter mit fester Bedeutung:** `_FREE` gibt frei, `_COUNT` ist eine
   Anzahl (`MAP_COUNT`, `SET_COUNT`), `LEN` eine Länge (Text, Bytes, Feld),
   `_LOAD`/`_SAVE` lesen und schreiben Dateien.
5. **Das `$`:** Ein Befehl, der Text liefert, darf mit und ohne `$`
   geschrieben werden — `ENV_GET$("HOME")` und `ENV_GET("HOME")` sind
   dasselbe, ebenso `CLIPBOARD_GET()` und `CLIPBOARD_GET$()`. Nur wo es beide
   Formen als **verschiedene** Befehle gibt, zählt die Schreibweise:
   `GUI_TOOLTIP(w, text$)` setzt, `GUI_TOOLTIP$(w)` liest. Ein `$` an einem
   Befehl, der keinen Text liefert (`LEN$`), bleibt ein Fehler.
6. **Englisch:** Befehlsnamen sind englisch. Die Module `zeit` und `geld`
   lassen sich auch als `IMPORT "time"` und `IMPORT "money"` einbinden. Der
   Typ heißt weiter `GELD` — ein zweiter Typname liefe bis in die Laufzeit
   durch und verdeckte eine eigene Klasse namens `Money`.
7. **Neue Befehle** folgen diesen Regeln. Einen neuen Namen für einen alten
   Befehl trägt man in `rust/drachenhauch_runtime/src/namen.rs`, in
   `daten/builtin_index.json` und hier ein; die Tests in `namen.rs` halten
   Tabelle und Index zusammen.
8. **Parameternamen** sind ebenfalls englisch, und sie sind Schnittstelle:
   Befehle nehmen benannte Argumente — `CIRCLE(x: 10, y: 20, r: 5, color: red)`
   (siehe [Sprache](sprache.md#named-arguments)). Gängige Kürzel bleiben
   (`x`, `y`, `w`, `h`, `r`, `win`, `idx`), sonst ganze Wörter: `color`,
   `path`, `text`, `width`, `count`, `encoding`. Dasselbe Ding heißt in jedem
   Befehl gleich. Die Befehlstabellen im Handbuch nennen dieselben Namen wie
   `daten/builtin_index.json`; `dhrt pruef parameter` hält beide zusammen.
   Wo ältere Befehle abweichende Namen tragen, nimmt ein benanntes Argument
   auch den Zweitnamen derselben Gruppe: `p`/`path`, `digits`/`decimals`,
   `vol`/`volume`, `n`/`count`, `v`/`number`, `wf`/`waveform`, `alt`/`old`,
   `fn`/`handler`, `flag`/`on`, `bind`/`bind_addr`, `wdg`/`widget` —
   `FILE_EXISTS(path: "a.txt")` geht, obwohl der Parameter `p` heißt.
9. **Zeit:** Zeitspannen und Fristen zählen **Millisekunden** (Parameter
   `ms` oder `…_ms`): `TWEEN_NEW`, `TIMER_AFTER`, `SLEEP`, die Fades, jedes
   `…_SET_TIMEOUT`. Der Schritt eines Bildes zählt **Sekunden** wie `DELTA()`:
   `PHYS2D_STEP`, `PHYS3D_STEP` (ein Wert ab 1 ist dort ein Fehler — das
   wären Millisekunden), ebenso `CHAR_UPDATE` mit `dt`. Lage und Länge in
   einem Medium (Musik, Video, Sample, MIDI-Datei) zählen **Sekunden** als
   FLOAT (`SAMPLE_LEN`, `VIDEO_SEEK`, `MIDI_FILE_DURATION`); Noten in einer
   MIDI-Datei schreibt man in Vierteln, der Einheit der Musik. Alte Befehle,
   die eine Zeitspanne in Sekunden oder Bildern nehmen, bleiben dabei; neben
   ihnen gibt es die Fassung in ms: `HTTP_TIMEOUT` → `HTTP_SET_TIMEOUT`,
   `SERIAL_TIMEOUT` → `SERIAL_SET_TIMEOUT`, `BT_SCAN` → `BT_SCAN_MS`,
   `AUDIO_AUTOPAN` → `AUDIO_AUTOPAN_MS`, `CHAR_SET_COYOTE_TIME` →
   `CHAR_SET_COYOTE_MS`, `CHAR_SET_JUMP_BUFFER` → `CHAR_SET_JUMP_BUFFER_MS`;
   `SAMPLE_SET_LOOP` (Frames) → `SAMPLE_SET_LOOP_TIME` (Sekunden wie
   `SAMPLE_LEN`).
10. **Winkel:** Zeichnen zählt **Grad**, 0 = rechts, **im Uhrzeigersinn**
    auf dem Schirm (`PIE`, `DRAWIMAGEROT`, `CAMERA_SET_ROTATION`, `ARC_DEG`);
    Mathe zählt **Bogenmaß** (`SIN`, `ATAN2`, `VEC2_ANGLE`). Die Parameter
    heißen `…_deg` bzw. `…_rad`. Zwei alte Befehle bleiben bei Bogenmaß und
    haben eine Gradfassung: `ARC` (gegen den Uhrzeigersinn) → `ARC_DEG`,
    `PHYS2D_BODY_ANGLE` → `PHYS2D_BODY_ANGLE_DEG`.
11. **Rechtecke und Strahlen:** Gezeichnete Rechtecke nennen zwei Ecken
    (`BOX`, `IMAGE_FILL_RECT`: x1, y1, x2, y2, beide Ecken gehören dazu);
    Ausschnitte nennen Ecke und Größe (`IMAGE_DRAW_RECT`, `SCISSOR`, alle
    gui-Lagen); Physik nennt die Mitte und die halbe Größe (`PHYS2D_ADD_BOX`,
    `PHYS3D_ADD_BOX`), `RAY_HIT_BOX` die Mitte und die ganze Größe. Ein „Ray“
    in 2D ist eine Strecke, das Ergebnis der Anteil t von 0 bis 1
    (`PHYSICS_RAY_*`); in 3D ein Strahl, das Ergebnis die Entfernung
    (`RAY_HIT_*`). Kein Treffer ist -1; nur `PHYSICS_LINES_X` liefert dann NaN.
12. **Zählen:** Elemente zählen ab **0**, „keins“ ist **-1** (`INSTR`,
    `GUI_LISTBOX_SELECTED`, Feld-Indizes). Antworten von Dialogen zählen ab
    **1**, „keine“ ist **0** (`GUI_ANSWER`, `GUI_INFOBAR_ANSWER`, dort ist das Kreuz -1). Ausnahmen, die
    bleiben: die Dimension bei `LBOUND`/`UBOUND` zählt ab 1 (bei `DIMSIZE`
    ab 0), die Seiten von `PDF_PREVIEW` ab 1 wie im Druck, die Zeilen eines
    Textbereichs ab 1 wie im Editor (`GUI_TEXTAREA_GOTO`, `_SCROLL`) --
    nur `GUI_TEXTAREA_VIEW` liefert die erste Zeile ab 0 (`GUI_TEXTAREA_GET_VIEW`
    ab 1). Wer „keinen Gast“ annimmt, bekommt je nach Modul NIL
    (`NET_TCP_ACCEPT`), -1 (`WS_ACCEPT`) oder FALSE (`HTTPD_ACCEPT`); `INSTR`
    nimmt einen negativen Start als 0, `INSTRREV` liefert dann -1.

## Der empfohlene Weg je Bereich

Für manches gibt es mehrere Wege, die gewachsen sind. Empfohlen ist:

| Bereich | Empfohlen | Daneben |
|---|---|---|
| Ton | `AUDIO_*` (Busse, Effekte, räumlich, Musik mit `AUDIO_MUSIC_*`) | `SOUND_LOAD`/`SOUND_PLAY`, `MUSIC_PLAY` als Kurzform für ein schnelles Geräusch; `SAMPLE_*` für Instrumente |
| Eingabe | `INPUT_BIND` und `INPUT_DOWN`/`INPUT_HIT` — Tastatur und Gamepad in einem | `KEY_*`, `MOUSE_*`, `JOYSTICK_*` für rohe Abfragen |
| Uhr | `MILLIS()` für Abläufe im Programm, `TIME_NOW()` für Datum und Uhrzeit | `TIMER()` (dieselbe Uhr in Sekunden, beide zählen ab dem Start des Programms), `GET_TIME()` (zählt erst ab `SCREEN`), `DATE$()`, `TIME$()` |
| Dateien | `FILE_READ$`, `FILE_WRITE`, `FILE_READ_LINES` mit Pfad | `FILE_OPEN`/`FILE_READ_LINE$`/`FILE_CLOSE` mit Handle für große Dateien |

`TIMER()` ist eine Uhr; die Familie `TIMER_EVERY`, `TIMER_AFTER` … dagegen
sind Rückruf-Zeitgeber aus dem Modul `timer`.

## Neue Namen für alte Befehle

### gehalten / gerade gedrückt / losgelassen: `_DOWN`, `_HIT`, `_RELEASED`

| Name | Was es tut |
|---|---|
| `KEY_DOWN(key: Code oder Buchstabe)` | TRUE solange die Taste mit SDL-Code code gehalten wird — jedes Frame erneut. Für Bewegen/Lenken — bisher `KEYPRESSED` |
| `KEY_HIT(key: Code oder Buchstabe)` | TRUE nur in dem einen Frame, in dem die Taste heruntergeht. Für Schießen, Springen, Umschalten — bisher `KEYHIT` |
| `KEY_RELEASED(key: Code oder Buchstabe)` | TRUE in dem Frame, in dem die Taste losgelassen wird — etwa „aufladen und beim Loslassen feuern“ — bisher `KEYRELEASED` |
| `KEY_REPEAT(key: Code oder Buchstabe)` | wie KEYHIT, feuert beim Halten aber zusätzlich mit der System-Tastenwiederholung. Für Textcursor und … — bisher `KEYREPEAT` |
| `KEY_WAIT()` | hält an, bis eine Taste kommt, und liefert ihren Code (-1, wenn das Fenster geschlossen wird) — bisher `WAITKEY` |
| `MOUSE_DOWN(n: Zahl)` | TRUE wenn Maustaste n gedrückt — 0=links, 1=rechts, 2=mitte (raylib-Reihenfolge: rechts vor mitte!) — bisher `MOUSEBUTTON` |
| `MOUSE_X()` | aktuelle Mausposition (in logischen Pixeln) — bisher `MOUSEX` |
| `MOUSE_Y()` | aktuelle Mausposition (in logischen Pixeln) — bisher `MOUSEY` |
| `MOUSE_WHEEL()` | Mausrad-Delta seit dem letzten Aufruf (+ hoch / − runter / 0) — bisher `MOUSEWHEEL` |
| `MOUSE_WHEEL_X()` | Rad in beiden Achsen und als Kommazahl; MOUSEWHEEL() kennt nur die senkrechte und rundet — feine … — bisher `MOUSEWHEEL_X` |
| `MOUSE_WHEEL_Y()` | Rad in beiden Achsen und als Kommazahl; MOUSEWHEEL() kennt nur die senkrechte und rundet — feine … — bisher `MOUSEWHEEL_Y` |
| `MOUSE_SET_VISIBLE(on: BOOLEAN)` | OS-Cursor zeigen/verstecken — verstecken, wenn das Spiel ein eigenes Fadenkreuz/Cursor-Sprite zeichnet — bisher `MOUSE_VISIBLE` |
| `JOYSTICK_DOWN(idx: Zahl, btn: Zahl)` | wird der Knopf gehalten? — bisher `JOYSTICK_BUTTON` |
| `INPUT_DOWN(action: STRING)` | BOOLEAN — Aktuell gedrueckt — bisher `INPUT_HELD` |
| `INPUT_HIT(action: STRING)` | BOOLEAN — Edge: gerade JETZT gedrueckt — bisher `INPUT_PRESSED` |

### gleicher Name, andere Bedeutung

| Name | Was es tut |
|---|---|
| `GUI_LISTBOX_ITEMS(lb, items)` | Eintraege ersetzen — bisher `GUI_SET_LISTBOX` |
| `GUI_DROPDOWN_ITEMS(dd, items)` | Eintraege ersetzen — bisher `GUI_SET_DROPDOWN` |
| `GUI_DOCK_SET_SIZE(win, area$, px)` | Breite (links/rechts) bzw. Höhe (unten) eines Bereichs — bisher `GUI_DOCK_SIZE` |
| `AUDIO_MUSIC_SET_PITCH(factor)` | Musik-Pitch (1.0 = normal; ueberlebt LOAD/QUEUE — Slow-Motion-Effekt) — bisher `AUDIO_MUSIC_PITCH` |
| `SET_COUNT(set)` | Anzahl der Elemente — bisher `SET_SIZE` |

### gui: `GUI_SET_X` setzt, `GUI_GET_X` liest

| Name | Was es tut |
|---|---|
| `GUI_GET_TEXT$(widget)` | STRING — Text (Label/Button/TextInput) — bisher `GUI_TEXT` |
| `GUI_GET_VALUE(widget)` | FLOAT — Slider-Wert — bisher `GUI_VALUE` |
| `GUI_GET_CHECKED(widget)` | BOOLEAN — Checkbox-Zustand — bisher `GUI_CHECKED` |
| `GUI_GET_ENABLED(widget)` | BOOLEAN — ist es bedienbar? — bisher `GUI_ENABLED` |
| `GUI_GET_VISIBLE(widget)` | ein-/ausblenden (unsichtbar = nicht gezeichnet, nicht interaktiv) — bisher `GUI_VISIBLE` |
| `GUI_GET_ACTIVE_TAB(win)` | INTEGER — aktiver Reiter-Index — bisher `GUI_ACTIVE_TAB` |
| `GUI_GET_DATE$(picker)` | gewähltes Datum als JJJJ-MM-TT — bisher `GUI_DATE` |
| `GUI_GET_TIME$(tp)` | STRING — die Uhrzeit als HH:MM:SS — wie TIME$(), auch ohne Sekundenfeld — bisher `GUI_TIME$` |
| `GUI_GET_ERROR$(field)` | STRING / — / — Meldung lesen, von außen setzen (z. B. aus der Datenbank), alle löschen — bisher `GUI_ERROR` |
| `GUI_GET_PICKED_COLOR(picker)` | gewählte Farbe (0xRRGGBB) — bisher `GUI_PICKED_COLOR` |
| `GUI_GET_MOTION()` | BOOLEAN — spielt die gui gerade Bewegung ab? — bisher `GUI_MOTION` |
| `GUI_GET_SOUNDS()` | alle Klänge der gui an oder aus -- für einen Schalter „Töne“ in den Einstellungen — bisher `GUI_SOUNDS` |
| `GUI_SET_TOOLTIP(widget, text)` | Hover-Hilfetext (mehrzeilig via \n; "" entfernt ihn) — bisher `GUI_TOOLTIP` |
| `GUI_GET_TOOLTIP$(widget)` | STRING — der gesetzte Hilfetext ("" ohne) — bisher `GUI_TOOLTIP$` |
| `GUI_DOCK_SET_TITLE(win, name$, title$)` | den Reiter neu beschriften — bisher `GUI_DOCK_TITLE` |
| `GUI_DOCK_GET_TITLE$(win, name$)` | STRING — die Beschriftung des Reiters — bisher `GUI_DOCK_TITLE$` |
| `GUI_STATUSBAR_SET_TIP(sb, field, tip$)` | Tooltip eines Feldes — bisher `GUI_STATUSBAR_TIP` |
| `GUI_STATUSBAR_GET_TIP$(sb, field)` | STRING — Tooltip eines Feldes lesen — bisher `GUI_STATUSBAR_TIP$` |

### Fenster und Schirm

| Name | Was es tut |
|---|---|
| `WINDOW_SET_FULLSCREEN(fs: BOOLEAN)` | Vollbild ein- oder ausschalten — bisher `SET_FULLSCREEN` |
| `WINDOW_SET_POS(x: INTEGER, y: INTEGER)` | Fenster an die OS-Pixelposition (x, y) setzen — bisher `SET_WINDOW_POS` |
| `WINDOW_SET_MONITOR(i: INTEGER)` | Fenster auf Monitor i schieben (ungültiger Index wird ignoriert) — bisher `SET_WINDOW_MONITOR` |
| `WINDOW_SET_TITLE(title: STRING)` | Fenstertitel im Lauf ändern — etwa für den Dateinamen oder den Punktestand — bisher `SETWINDOWTITLE` |
| `WINDOW_SET_FPS(n: INTEGER)` | Ziel-Bildrate; 0 = so schnell wie möglich — bisher `SETFPS` |
| `SCREEN_WIDTH()` | logische Fenstergröße (wie an SCREEN übergeben); 0 vor SCREEN — bisher `SCREENWIDTH` |
| `SCREEN_HEIGHT()` | logische Fenstergröße (wie an SCREEN übergeben); 0 vor SCREEN — bisher `SCREENHEIGHT` |
| `SCREENSHOT_SAVE(path: STRING)` | Speichert den aktuellen Bildschirminhalt als PNG — bisher `SAVESCREENSHOT` |
| `QUIT_REQUESTED()` | TRUE wenn das Programm enden soll: Fenster-X / Alt+F4, das Bildlimit von DHRT_FRAMES/--bilder — und per … — bisher `QUITREQUESTED` |

### Zeichnen

| Name | Was es tut |
|---|---|
| `DRAW_IMAGE(image, x, y, w = -1, h = -1, angle = 0.0, tint = -1, flip_x = FALSE, flip_y = FALSE, sx = 0, sy = 0, sw = -1, sh = -1)` | Bild bei (x, y) zeichnen. Alles Weitere geht benannt: `w`/`h` Zielgröße, `angle` Grad um die Mitte, `tint` Tönung (ihr Alpha ist die Deckkraft), `flip_x`/`flip_y` spiegeln, `sx, sy, sw, sh` ein Ausschnitt — etwa `DRAW_IMAGE(held, x, y, angle: 30, flip_x: TRUE)` — bisher `DRAWIMAGE` |
| `DRAW_IMAGE_FLIPPED(image, x, y[, flip_x[, flip_y]])` | mit Spiegelung — bisher `DRAWIMAGEFLIPPED` |
| `DRAW_IMAGE_PART(image, sx, sy, sw, sh, x, y)` | Sub-Rechteck aus Sheet zeichnen — bisher `DRAWIMAGEPART` |
| `DRAW_IMAGE_PART_EX(image, sx, sy, sw, sh, x, y, w, h [, color])` | Sub-Rechteck auf die Größe w x h gezogen; color tönt es, ihr Alpha ist die Deckkraft (RGBA(255, 255, 255, … — bisher `DRAWIMAGEPARTEX` |
| `DRAW_IMAGE_ROT(image, x, y, angle_deg [, scale [, tint]])` | zentriert auf (x,y), um winkel Grad gedreht (um die Mitte), optional skaliert + getönt. Ideal für rotierte … — bisher `DRAWIMAGEROT` |
| `DRAW_IMAGE_9(image, x, y, width, height, left, top, right, bottom[, tint])` | 9-Slice: das Bild in das Rechteck gestreckt, die vier Ecken in ihrer Größe, die Ränder nur in einer … — bisher `DRAWIMAGE9` |
| `DRAW_TILEMAP(tileset, map_arr, tw, th, sx, sy)` | Zeichnet ein zweidimensionales Array von Kachel-Nummern als Karte — bisher `DRAWTILEMAP` |
| `DRAW_FPS(x, y[, color])` | die Bildrate als „60 FPS“ in der eingebauten Schrift zeigen, in Bildschirmkoordinaten (die Kamera gilt … — bisher `DRAWFPS` |
| `BOX_ROUND(x1, y1, x2, y2, radius[, color])` | gefülltes Rechteck mit runden Ecken — bisher `BOXROUND` |
| `RECT_ROUND(x1, y1, x2, y2, radius[, color])` | Rechteck-Rahmen mit runden Ecken — bisher `RECTROUND` |
| `CIRCLE_OUTLINE(x, y, r[, color])` | Kreis nur als Kontur (Gegenstück zu CIRCLE) — bisher `CIRCLEOUTLINE` |
| `ELLIPSE_OUTLINE(x1, y1, x2, y2[, color[, width]])` | Ellipse nur als Kontur — bisher `ELLIPSEOUTLINE` |
| `TRIANGLE_OUTLINE(x1, y1, x2, y2, x3, y3[, color[, width]])` | Dreieck nur als Kontur — bisher `TRIANGLEOUTLINE` |
| `POLYGON_OUTLINE(points[, color[, width]])` | Polygon nur als Kontur — bisher `POLYGONOUTLINE` |
| `NGON_OUTLINE(x, y, corners, r, angle[, color[, width]])` | regelmäßiges Vieleck nur als Kontur; width wie bei LINEW — bisher `NGONOUTLINE` |
| `PIE_OUTLINE(x, y, r, from_deg, to_deg[, color])` | Kreisstück nur als Kontur, mit beiden Radien — bisher `PIEOUTLINE` |
| `DRAW_GRADIENT_H(x1, y1, x2, y2, color1: INTEGER, color2: INTEGER)` | Rechteck mit horizontalem Farbverlauf (links→rechts) — bisher `GRADIENTH` |
| `DRAW_GRADIENT_V(x1, y1, x2, y2, color1: INTEGER, color2: INTEGER)` | Rechteck mit vertikalem Farbverlauf (oben→unten) — bisher `GRADIENTV` |
| `DRAW_TEXT_ROT(x, y, s, angle[, scale = 1.0[, color]])` | Text zentriert auf (x, y), um das Zentrum gedreht (Grad, wie DRAWIMAGEROT) und skaliert — für Score-Popups, … — bisher `TEXTROT` |
| `DRAW_BOX_ROT(x, y, width, height, angle[, color])` | gefülltes Rechteck zentriert auf (x, y), um seine Mitte gedreht (Grad, im Uhrzeigersinn wie DRAWIMAGEROT) — bisher `BOXROT` |
| `DRAW_LINE_DASHED(x1, y1, x2, y2, dash, gap[, color])` | gestrichelte Linie, Strich- und Lückenlänge in Punkten — bisher `LINEDASHED` |

### Bilder, Schrift, Ton, Modelle: `BEREICH_VERB`

| Name | Was es tut |
|---|---|
| `IMAGE_LOAD(path: STRING)` | Datei laden (PNG, JPG, BMP, …) — bisher `LOADIMAGE` |
| `IMAGE_GET_PIXEL(image, x, y)` | Pixelfarbe (&HRRGGBB) an (x, y) lesen; -1 bei Index außerhalb. Gegenstück zu PLOT (schreiben) — für … — bisher `GETPIXEL` |
| `IMAGE_GET_ALPHA(image, x, y)` | Deckkraft an (x, y), 0..255; -1 bei Index außerhalb. Nötig, weil GETPIXEL eine Farbe liefert und dort … — bisher `GETALPHA` |
| `FONT_LOAD(path$, size[, chars$])` | TTF/OTF/TTC laden → FONT-Handle (INTEGER); chars$ = Schriftblöcke ("kyrillisch, griechisch", … — bisher `LOADFONT` |
| `FONT_LOAD_IMAGE(image: IMAGE, separator_color: Zahl, first_char: Zahl)` | Lädt eine Pixelschrift aus einem PNG, in dem die Zeichen durch eine Trennfarbe voneinander abgesetzt sind — bisher `LOADFONT_IMAGE` |
| `FONT_SET(handle: INTEGER [, size: INTEGER])` | aktive Schrift setzen; SETFONT(-1) = Default-Font. Ohne size gilt danach die Größe, mit der die Schrift … — bisher `SETFONT` |
| `SOUND_LOAD(path: STRING)` | Sample laden (kurze Effekte: WAV, OGG) — bisher `LOADSOUND` |
| `SOUND_PLAY(s[, loops[, volume]])` | abspielen, default loops=0, volume=1.0 — bisher `PLAYSOUND` |
| `SOUND_STOP(s)` | Sound stoppen — bisher `STOPSOUND` |
| `SOUND_FREE(s)` | Sound stoppen und seinen Puffer freigeben — gegen Puffer-Akkumulation bei vielen … — bisher `UNLOADSOUND` |
| `MUSIC_PLAY(path$[, loops[, volume]])` | längere Datei streamen (.ogg/.mp3/.flac + Tracker-Module .mod/.xm — echter Amiga-Sound), default loops=-1 … — bisher `PLAYMUSIC` |
| `MUSIC_STOP()` | ein Stream gleichzeitig. Nachgefüllt wird auf dem Audio-Thread, nicht im Game-Loop: einen … — bisher `STOPMUSIC` |
| `MODEL_LOAD(path$: STRING)` | INTEGER — Modell aus einer Datei laden (OBJ, GLTF, IQM …) — bisher `LOADMODEL` |
| `IMAGE_WIDTH(image)` | Breite eines Bildes in Pixeln — bisher `IMAGEWIDTH` |
| `IMAGE_HEIGHT(image)` | Höhe eines Bildes in Pixeln — bisher `IMAGEHEIGHT` |
| `AUDIO_SET_VOLUME(ch, v)` | Lautstärke eines Kanals 0..1 — bisher `AUDIO_VOLUME` |
| `AUDIO_MUSIC_SET_VOLUME(v)` | Lautstärke der Musik 0..1 — bisher `AUDIO_MUSIC_VOLUME` |
| `FONT_LOAD_SDF(path$, size[, chars$]) AS INTEGER` | eine Schrift, die bei jeder Größe scharf bleibt (Abstandsfeld) — bisher `LOADFONT_SDF` |

### Dateien (mit Pfad), Dateien (mit Handle), Ordner, Pfade

| Name | Was es tut |
|---|---|
| `FILE_EXISTS(p: STRING)` | Datei vorhanden? — bisher `FILEEXISTS` |
| `FILE_SIZE(path)` | Größe in Bytes — bisher `FILESIZE` |
| `FILE_TIME(path$)` | zuletzt geändert, Sekunden wie im Modul zeit — bisher `FILETIME` |
| `FILE_COPY(src, dst)` | Datei kopieren — bisher `COPYFILE` |
| `FILE_DELETE(path)` | löschen / umbenennen·verschieben — bisher `DELETEFILE` |
| `FILE_SAME(a$, b$)` | meinen beide Pfade dieselbe Datei? Ein Textvergleich irrt bei / gegen \, bei einem .. im Pfad, bei kurzen … — bisher `SAMEFILE` |
| `FILE_APPEND(p: STRING, text: STRING [, encoding$])` | Text ans Ende hängen (legt die Datei an) — bisher `APPENDFILE` |
| `FILE_READ$(f [, encoding$])` | Rest komplett lesen — bisher `READALL$` |
| `FILE_READ_LINES(path [, encoding$])` | Datei als Zeilen-Array lesen — bisher `READLINES` |
| `FILE_READ_BYTES(path)` | ganze Datei als Bytes — bisher `READALL_BYTES` |
| `FILE_WRITE(path, text [, encoding$])` | Text komplett schreiben (überschreibt/erzeugt) — bisher `WRITEALL` |
| `FILE_WRITE_BYTES(path, buffer)` | Bytes in eine Datei (überschreibt) — bisher `WRITEALL_BYTES` |
| `FILE_OPEN(path: STRING, mode: STRING [, encoding$])` | Modi: "r" lesen, "w" neu schreiben, "a" anhängen — bisher `OPENFILE` |
| `FILE_CLOSE(f)` | schließt — bisher `CLOSEFILE` |
| `FILE_READ_LINE$(f)` | nächste Zeile (ohne \n); am Dateiende "" wie eine leere Zeile, darum EOF(f) fragen — bisher `READLINE` |
| `FILE_WRITE_LINE(f, text)` | schreibt + \n — bisher `WRITELINE` |
| `FILE_EOF(f)` | beim Ende? — bisher `ENDOFFILE` |
| `DIR_EXISTS(path)` | Verzeichnis vorhanden? — bisher `DIREXISTS` |
| `DIR_LIST(path, pattern$ = "", recursive = FALSE)` | Eintragsnamen (sortiert), optional gefiltert; mit `recursive: TRUE` wie `DIR_LIST_REC` — bisher `DIRLIST` |
| `DIR_LIST_REC(path$ [, pattern$])` | wie oben, aber rekursiv — nur Dateien, Pfade relativ mit / — bisher `DIRLIST_REC` |
| `PATH_JOIN(a, b, ...)` | Pfadteile mit / verbinden — bisher `PATHJOIN` |
| `PATH_DIR(p: STRING)` | Verzeichnis-Anteil (ohne letzten Bestandteil) — bisher `DIRNAME` |
| `PATH_BASE(p: STRING)` | letzter Pfad-Bestandteil (Datei-/Ordnername) — bisher `BASENAME` |
| `PATH_REAL$(path$)` | der volle, bereinigte Pfad: absolut, ./.. aufgelöst, in der Schreibweise der Platte; gibt es die Datei … — bisher `REALPATH$` |
| `START_DIR$()` | der Ordner, aus dem das Programm aufgerufen wurde — bisher `STARTDIR$` |
| `ARG_PATH$(number)` | Argument Nr. n als Pfad vom Aufrufort aus (siehe unten) — bisher `ARGPATH$` |
| `ENV_GET$(name, default = "")` | Umgebungsvariable lesen — bisher `GETENV$` |
| `ENV_SET(name, value)` | Umgebungsvariable setzen (dieses Programm + seine Kinder) — bisher `SETENV` |
| `URL_OPEN(address: STRING)` | Öffnet eine Adresse im Standardbrowser – etwa die Anleitung oder die Seite des Spiels — bisher `OPENURL` |
| `DOC_OPEN(path$)` | Öffnet eine lokale Datei mit ihrem Standardprogramm – die geschriebene PDF im Betrachter, die Tabelle in … — bisher `OPENDOC` |
| `FILE_RENAME(old, new)` | umbenennen oder verschieben — bisher `RENAME` |
| `FILE_TEMP$([prefix$ = "dh" [, extension$]]) AS STRING` | freier Name im Temp-Verzeichnis, leer angelegt — bisher `TEMPFILE$` |
| `FILE_SEEK(file, position)` | Position setzen (0 = Anfang) — bisher `SEEK` |
| `FILE_TELL(file) AS INTEGER` | aktuelle Position — bisher `TELL` |
| `FILE_WRITE_TEXT(f, text)` | schreibt ohne \n — bisher `WRITE` |
| `FILE_READ_BLOCK(file, count) AS BUFFER` | bis zu count Bytes vom Handle — bisher `READ_BYTES` |
| `FILE_WRITE_BLOCK(file, buffer)` | Bytes ans Handle — bisher `WRITE_BYTES` |
| `DIR_TEMP$() AS STRING` | das Temp-Verzeichnis des Systems — bisher `TEMPDIR$` |
| `DIR_CURRENT$() AS STRING` | aktuelles Arbeitsverzeichnis — bisher `CWD$` |
| `DIR_CHANGE(path)` | Arbeitsverzeichnis wechseln — bisher `CHDIR` |
| `DIR_CREATE(path)` | Verzeichnis anlegen (inkl. Eltern) — bisher `MKDIR` |
| `DIR_DELETE(path$ [, with_content])` | Verzeichnis löschen (leer; mit TRUE samt Inhalt) — bisher `RMDIR` |

### MAP wie ARRAY_ und SET_: mit Unterstrich, Anzahl heißt COUNT

| Name | Was es tut |
|---|---|
| `MAP_GET(m, key)` | lesen, wirft wenn Schlüssel fehlt — bisher `MAPGET` |
| `MAP_GET_OR(m, key, default)` | lesen mit Default — bisher `MAPGETOR` |
| `MAP_PUT(m, key, value)` | setzen / überschreiben — bisher `MAPPUT` |
| `MAP_HAS(m, key)` | Existenz — bisher `MAPHAS` |
| `MAP_REMOVE(m, key)` | TRUE wenn entfernt — bisher `MAPREMOVE` |
| `MAP_CLEAR(m)` | Leert die ganze Map auf einmal — bisher `MAPCLEAR` |
| `MAP_KEYS(m)` | alle Schlüssel — bisher `MAPKEYS` |
| `MAP_VALUES(m)` | Liefert alle Werte als Array (in Einfüge-Reihenfolge) — bisher `MAPVALUES` |
| `MAP_ITEMS(m)` | Liefert alle Paare als Array von (Schlüssel, Wert)-Tupeln – ideal mit FOR EACH — bisher `MAPITEMS` |
| `MAP_COUNT(m)` | Anzahl der Schlüssel-Wert-Paare — bisher `MAPSIZE` |
| `ARRAY_DIM_COUNT(arr)` | Anzahl Dimensionen — bisher `DIMCOUNT` |
| `ARRAY_DIM_SIZE(arr, n)` | Größe der n-ten Dimension (0-basiert) — bisher `DIMSIZE` |
| `MQTT_UNACKED_COUNT(handle)` | wie viele QoS-1-Nachrichten der Broker noch nicht bestätigt hat — bisher `MQTT_PENDING` |
| `MOVE_TOWARD(cur, target, maxdelta)` | cur um max. maxd Richtung target bewegen — bisher `MOVETOWARD` |

### Englisch statt Deutsch (Module zeit und geld, FFI-Zeiger)

| Name | Was es tut |
|---|---|
| `TIME_NOW()` | jetzt, als Zeitpunkt — bisher `ZEIT_JETZT` |
| `TIME_UNIX()` | jetzt als Unix-Zeit: Sekunden seit 1970 in UTC, mit Bruchteil (wie Pythons time.time()) — bisher `ZEIT_UNIX` |
| `TIME_FROM_UNIX(unix_seconds)` | Unix-Sekunden zu Zeitpunkt in Ortszeit (samt Sommerzeit jenes Tages) — bisher `ZEIT_AUS_UNIX` |
| `TIME_TO_UNIX(time)` | Zeitpunkt in Ortszeit zu Unix-Sekunden — bisher `ZEIT_NACH_UNIX` |
| `TIME_FROM_PARTS(year: INTEGER, month: INTEGER, day: INTEGER, hour: INTEGER = 0, minute: INTEGER = 0, second: INTEGER = 0)` | Zeitpunkt aus Einzelwerten — bisher `ZEIT_AUS_TEILEN` |
| `TIME_PART(time: INTEGER, part: STRING)` | ein Feld: jahr, monat, tag, stunde, minute, sekunde — bisher `ZEIT_TEIL` |
| `TIME_FORMAT$(time: INTEGER, pattern: STRING)` | Zeitpunkt nach Muster — bisher `ZEIT_FORMAT$` |
| `TIME_TEXT$(time: INTEGER)` | Zeitpunkt als JJJJ-MM-TT hh:mm:ss — bisher `ZEIT_TEXT$` |
| `TIME_PARSE(text: STRING)` | Text zu Zeitpunkt; bricht bei Unlesbarem ab — bisher `ZEIT_PARSE` |
| `TIME_CAN_PARSE(text: STRING)` | ließe sich der Text lesen? (fragt, bricht nicht ab) — bisher `ZEIT_LESBAR` |
| `TIME_ADD(time: INTEGER, seconds: INTEGER)` | Sekunden dazu (negativ = zurück) — bisher `ZEIT_PLUS` |
| `TIME_DIFF(time: INTEGER, earlier: INTEGER)` | Abstand in Sekunden (negativ, wenn time früher liegt) — bisher `ZEIT_DIFF` |
| `TIME_DURATION$(seconds: INTEGER)` | Sekunden als lesbare Spanne — bisher `ZEIT_DAUER$` |
| `TIME_WEEKDAY(time: INTEGER)` | 1 = Montag … 7 = Sonntag — bisher `ZEIT_WOCHENTAG` |
| `MONEY_NEW(amount)` | aus Text ("19,99", "1.234,56") oder Zahl — bisher `GELD_NEU` |
| `MONEY_ABS(g: GELD)` | Betrag ohne Vorzeichen — bisher `GELD_ABS` |
| `MONEY_CENTS(g: GELD)` | in ganze Cent, kaufmännisch gerundet — bisher `GELD_CENT` |
| `MONEY_FROM_CENTS(cent: INTEGER)` | aus ganzen Cent — bisher `GELD_AUS_CENT` |
| `MONEY_ROUND(g: GELD [, digits])` | kaufmännisch, Vorgabe 2 Stellen — bisher `GELD_RUNDEN` |
| `MONEY_SPLIT(g: GELD, count)` | aufteilen, ohne einen Cent zu verlieren — bisher `GELD_TEILEN` |
| `MONEY_TEXT$(g: GELD [, symbol$])` | 19,99 €, Symbol wählbar — bisher `GELD_TEXT$` |
| `MONEY_VALUE(g: GELD)` | bewusst herausgehen (Anzeige in Diagrammen o. Ä.) — bisher `GELD_ZAHL` |
| `BUFFER_FROM_POINTER(pointer, length)` | kopiert laenge Bytes hinter einem Zeiger in einen neuen BUFFER; 0 Bytes ergeben einen leeren, ein … — bisher `BUFFER_AUS_ZEIGER` |
| `TEXT_FROM_POINTER$(pointer [, wide])` | kopiert den Text hinter einem Zeiger bis zum Nullzeichen (UTF-8; mit breit = TRUE wchar_t, also UTF-16 … — bisher `TEXT_AUS_ZEIGER$` |
| `NUMBER_TEXT$(number [, digits]) AS STRING` | Zahl deutsch mit Tausenderpunkten und Dezimalkomma (400.000, 1.234.567,89); mit digits kaufmännisch gerundet — bisher `ZAHL$` |
| `BUFFER_POINTER(buffer) AS INTEGER` | die Adresse der Bytes eines BUFFER, für ein Struct-Feld, das auf einen anderen Puffer zeigt; ein leerer … — bisher `BUFFER_ZEIGER` |

### Der Bereich vorn: Text, Typen, System, Cloud

| Name | Was es tut |
|---|---|
| `TEXT_STARTS_WITH(s, prefix) AS BOOLEAN` | beginnt der Text so? — bisher `STARTSWITH` |
| `TEXT_ENDS_WITH(s, suffix) AS BOOLEAN` | endet der Text so? — bisher `ENDSWITH` |
| `TEXT_CONTAINS(s, part) AS BOOLEAN` | Teilstring enthalten? (Funktionsform von part IN s) — bisher `CONTAINS` |
| `TEXT_COUNT(text$, part$)` | Anzahl nicht-überlappender Vorkommen von part — bisher `COUNT` |
| `TEXT_REMOVE_SPACES$(s$)` | jeden Leerraum entfernen (Leerzeichen, Tabulator, Umbruch) — für Eingaben wie "DE89 3704 0044" — bisher `NOSPACES$` |
| `IS_NUM(x)` | eine Zahl? (BOOLEAN ist keine) — bisher `ISNUM` |
| `IS_INT(x)` | eine Ganzzahl? — bisher `ISINT` |
| `IS_STR(x)` | ein Text? — bisher `ISSTR` |
| `IS_BOOL(x)` | ein BOOLEAN? — bisher `ISBOOL` |
| `IS_NUMERIC(s) AS BOOLEAN` | als Zahl parsebar? — bisher `ISNUMERIC` |
| `SYSTEM_EXE_PATH$() AS STRING` | Pfad der laufenden Programmdatei — bisher `EXEPATH$` |
| `CLOUD_LEADERBOARD_SUBMIT(board$, name$, score[, best_low])` | Punkte einreichen; best_low = TRUE für „kleiner ist besser“ — bisher `LEADERBOARD_SUBMIT` |
| `CLOUD_LEADERBOARD_FETCH(board$, n[, ascending]) AS ARRAY OF TUPLE` | Die besten n Einträge als (name$, score)-Tupel, sortiert (Standard absteigend) — bisher `LEADERBOARD_FETCH` |

### Fristen: `_SET_TIMEOUT` in Millisekunden

| Name | Was es tut |
|---|---|
| `SMTP_SET_TIMEOUT(m: SMTP, ms)` | Frist je Schritt beim Versand — bisher `SMTP_TIMEOUT` |
| `MODBUS_SET_TIMEOUT(m, ms)` | wie lange auf eine Antwort gewartet wird — bisher `MODBUS_TIMEOUT` |

### Freigeben heißt `_FREE`, eine Verbindung schließen `_CLOSE`

| Name | Was es tut |
|---|---|
| `AUDIO_CLOCK_FREE(clock)` | eine Uhr freigeben — bisher `AUDIO_CLOCK_REMOVE` |
| `AUDIO_EMITTER_FREE(emitter)` | eine Schallquelle freigeben — bisher `AUDIO_EMITTER_REMOVE` |
| `AUDIO_LISTENER_FREE(listener)` | einen Zuhörer freigeben — bisher `AUDIO_LISTENER_REMOVE` |
| `AUDIO_MOD_FREE(mod: AUDIO_MOD)` | einen Modulator freigeben — bisher `AUDIO_MOD_REMOVE` |
| `GUI_WIDGET_FREE(widget)` | ein Widget entfernen (das Handle wird ungültig) — bisher `GUI_DESTROY` |
| `GUI_WINDOW_FREE(win)` | ein Fenster samt Widgets entfernen — bisher `GUI_WINDOW_DESTROY` |
| `PHYS2D_BODY_FREE(w, idx)` | einen Körper aus der Welt nehmen — bisher `PHYS2D_REMOVE` |
| `PHYS3D_BODY_FREE(w, idx)` | einen Körper aus der Welt nehmen — bisher `PHYS3D_REMOVE` |
| `BT_CLOSE(handle)` | die Verbindung schließen (wie DB_CLOSE, NET_CLOSE) — bisher `BT_DISCONNECT` |
| `MQTT_CLOSE(handle)` | die Verbindung schließen (wie DB_CLOSE, NET_CLOSE) — bisher `MQTT_DISCONNECT` |
