# Grafik-Built-ins

Grafik, Sound und Eingabe — nativ in der Runtime `dhrt` (raylib). Alle Befehle hier brauchen ein offenes Fenster — also muss vor allem anderen `SCREEN(...)` aufgerufen werden.

Wenn das `camera`-Modul aktiv ist und `CAMERA_SET` aufgerufen wurde, interpretieren alle Drawing-Befehle ihre Koordinaten als **World-Koordinaten** (siehe [Camera-Modul](module-camera.md)).

## Inhalt

- [Fenster und Frame](#fenster-und-frame)
- [Monitore und Fensterposition](#monitore-und-fensterposition)
- [Zeichnen](#zeichnen)
- [Bilder](#bilder)
- [Asset-Preloader (`LOAD_ASSETS`)](#asset-preloader)
- [Sprite-Atlas + Batch-Draw](#sprite-atlas--batch-draw)
- [Z-Layer-Rendering](#z-layer-rendering)
- [Tilemap](#tilemap)
- [Sound und Musik](#sound-und-musik)
- [Eingabe: Tastatur und Maus](#eingabe-tastatur-und-maus)

## Fenster und Frame

| Funktion | Zweck |
|---|---|
| `SCREEN(w, h[, titel$[, scale]])` | öffnet Fenster mit logischer Größe `w × h`. `scale > 1` macht jeden logischen Pixel größer (Retro-Look). |
| `FLIP()` | Frame an den Bildschirm ausgeben (synchronisiert auf 60 FPS) |
| `CLS([color])` | Buffer mit `color` füllen (Default: schwarz) |
| `SLEEP(ms)` | wartet ms Millisekunden, ohne dass das Fenster einfriert |
| `QUITREQUESTED()` → BOOLEAN | TRUE wenn der User das Fenster schliessen will: Fenster-X / Alt+F4 — **und per raylib-Default auch `ESC`**. Wer `ESC` im Spiel selbst nutzen will (Menü/Pause), schaltet das mit `WINDOW_ESC_QUIT(FALSE)` ab |
| `WINDOW_RESIZABLE(an)` | darf der Nutzer das Fenster ziehen? (Vorgabe: nein) |
| `WINDOW_MIN_SIZE(w, h)` | kleinste Fenstergröße beim Ziehen |
| `WINDOW_MAX_SIZE(w, h)` | größte Fenstergröße beim Ziehen |
| `WINDOW_MAXIMIZE()` | Fenster auf Bildschirmgröße bringen |
| `WINDOW_MINIMIZE()` | Fenster in die Leiste legen |
| `WINDOW_RESTORE()` | aus maximiert oder minimiert zurückholen |
| `SETWINDOWTITLE(titel$)` | Fenstertitel im Lauf ändern — etwa für den Dateinamen oder den Punktestand |
| `SET_FULLSCREEN(an)` | Vollbild ein- oder ausschalten |
| `WINDOW_IS_FULLSCREEN()` → BOOLEAN | läuft es gerade im Vollbild? |
| `WINDOW_FOCUSED()` → BOOLEAN | ist das Fenster im Vordergrund? — damit pausiert man, wenn der Nutzer wegklickt |
| `WINDOW_MINIMIZED()` / `WINDOW_MAXIMIZED()` / `WINDOW_HIDDEN()` → BOOLEAN | Zustand des Fensters abfragen |
| `WINDOW_FOCUS()` | das eigene Fenster nach vorne holen |
| `WINDOW_DPI_X()` / `WINDOW_DPI_Y()` → FLOAT | Bildschirm-Skalierung (1.0 normal, 2.0 HiDPI) — ohne sie weiß ein Programm nicht, ob seine Pixelgrößen auf dem Zielgerät winzig herauskommen |
| `FPS()` → INTEGER | gemessene Bilder je Sekunde |
| `SETFPS(n)` | Ziel-Bildrate; `0` = so schnell wie möglich |
| `FILES_DROPPED()` → INTEGER | wie viele Dateien wurden in diesem Bild ins Fenster gezogen? |
| `FILE_DROPPED(i)` → STRING | Pfad der `i`-ten davon |
| `CLIPBOARD_GET()` → STRING | Text aus der Zwischenablage lesen |
| `CLIPBOARD_SET(text$)` | Text in die Zwischenablage legen |
| `GFX_PUSH()` | Zeichenzustand sichern: Kamera, Ebenen, Licht, Umgebung, Schatten, 3D-Kamera, Schrift, `POSTFX` |
| `GFX_POP()` | ihn zurückholen — **ohne vorheriges `PUSH` ein Fehler** |
| `GFX_DEPTH()` → INTEGER | wie tief ist der Stapel? |

Klassischer Game-Loop:

```basic
SCREEN(320, 240, "Mein Spiel", 2)

WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 20, 30))

    ' ... zeichnen ...

    FLIP()
    SLEEP(16)
WEND
```

`scale=2` öffnet ein 640×480-Fenster, in dem aber alle Koordinaten weiterhin in 320×240 logischer Auflösung gerechnet werden — die Pixel werden hochskaliert.

## Monitore und Fensterposition

Display-Infos und das Platzieren des Programmfensters auf dem Desktop. Monitor-Index läuft von `0` bis `MONITOR_COUNT()-1`. Alle Maße hier sind **echte OS-Pixel** (kein `scale`), weil sie die Hardware bzw. die Lage auf dem Desktop beschreiben — nicht das logische `SCREEN`-Raster.

| Funktion | Zweck |
|---|---|
| `MONITOR_COUNT()` → INTEGER | Anzahl angeschlossener Monitore |
| `CURRENT_MONITOR()` → INTEGER | Index des Monitors, auf dem das Fenster gerade überwiegend liegt |
| `MONITOR_WIDTH(i)` → INTEGER | native Breite von Monitor `i` (px) |
| `MONITOR_HEIGHT(i)` → INTEGER | native Höhe von Monitor `i` (px) |
| `MONITOR_REFRESH(i)` → INTEGER | Bildwiederholrate von Monitor `i` (Hz) |
| `MONITOR_NAME(i)` → STRING | Anzeigename von Monitor `i` |
| `MONITOR_X(i)`, `MONITOR_Y(i)` → INTEGER | Position von Monitor `i` im virtuellen Desktop (px) |
| `SET_WINDOW_MONITOR(i)` | Fenster auf Monitor `i` schieben (ungültiger Index wird ignoriert) |
| `WINDOW_X()`, `WINDOW_Y()` → INTEGER | Position der linken oberen Fensterecke (px) |
| `SET_WINDOW_POS(x, y)` | Fenster an die OS-Pixelposition `(x, y)` setzen |

### Echtes natives Vollbild

| Funktion | Zweck |
|---|---|
| `SCREEN_NATIVE([titel$])` | Vollbild in der **echten Auflösung** des aktuellen Monitors |

`SCREEN_NATIVE()` ist der direkte Weg zu scharfem Vollbild und ersetzt `SCREEN(...)`:

```basic
SCREEN_NATIVE("Mein Spiel")
' SCREENWIDTH()/SCREENHEIGHT() liefern jetzt die Monitor-Auflösung
```

Der Unterschied zur Zusammensetzung aus `SCREEN(w, h)` **+** `SET_FULLSCREEN(TRUE)`: bei letzterem wird ein kleines Backbuffer (z. B. 1280×720) auf den Monitor **hochskaliert** → unscharf. `SCREEN_NATIVE()` rendert dagegen 1:1 in nativen Pixeln (logisches Raster = Monitor-Auflösung). Wer die logische Auflösung selbst festlegen will, nutzt weiterhin die manuelle Variante:

```basic
DIM m AS INTEGER : m = CURRENT_MONITOR()
SCREEN(MONITOR_WIDTH(m), MONITOR_HEIGHT(m), "Vollbild", 1)
SET_FULLSCREEN(TRUE)
```

Komplettes Beispiel mit allen Monitoren, Live-Fensterposition und Monitor-Wechsel: [`examples/120_monitors.dh`](../examples/120_monitors.dh).

### Transparente Fenster & Desktop-Overlay

Der Fenster-Hintergrund kann durchscheinen, sodass der Desktop sichtbar bleibt — für schwebende Overlays (Visualizer, HUD, Effekt) oder Fenster mit „Glas"-Hintergrund.

| Funktion | Zweck |
|---|---|
| `SCREEN_TRANSPARENT(w, h[, titel$[, scale]])` | öffnet ein Fenster mit **transparentem** Hintergrund; `w`/`h` = 0 → ganzer aktueller Monitor (Vollbild-Overlay) |
| `WINDOW_UNDECORATED(flag)` | Fensterrahmen/Titelleiste aus (`TRUE`) / ein (`FALSE`) |
| `WINDOW_TOPMOST(flag)` | Fenster immer im Vordergrund halten |
| `WINDOW_ESC_QUIT(an)` | `ESC` als Fenster-Schliessen-Taste an/aus (raylib-Default: **an**). Mit `FALSE` ist `ESC` eine ganz normale Taste (`QUITREQUESTED` wird dann nur noch durch Fenster-X / Alt+F4 ausgelöst) — für Spiele, die `ESC` fürs Pause-/Hauptmenü nutzen |
| `WINDOW_PASSTHROUGH(flag)` | Maus-Klicks zum Desktop **durchreichen** (klick-durchlässiges Widget) |

**Wichtig:** `SCREEN_TRANSPARENT(...)` muss die **allererste** Grafik-Anweisung sein (vor `LOADIMAGE`/`SCREEN`/…). Transparenz ist ein Fenster-Erzeugungs-Flag und lässt sich nicht nachträglich setzen. `WINDOW_UNDECORATED`/`WINDOW_TOPMOST`/`WINDOW_PASSTHROUGH` dagegen jederzeit. `WINDOW_PASSTHROUGH(TRUE)` braucht ein randloses Fenster (`WINDOW_UNDECORATED(TRUE)`); die Tastatur (z. B. `ESC`) erreicht das Fenster dann nur, solange es den Fokus hat — nach einem Desktop-Klick zum Beenden Stop-Knopf im Editor bzw. `Alt+F4`.

Im Transparent-Modus nimmt `CLS` das **Alpha-Byte wörtlich**:

```basic
CLS()              ' voll durchsichtig -> der Desktop scheint ueberall durch
CLS(&HC0101826)    ' halbtransparenter Hintergrund (Alpha 0xC0) -> Desktop schimmert gedimmt durch
```

(Im normalen, nicht-transparenten Modus bleibt der Hintergrund wie gehabt immer deckend.)

**Desktop-Overlay** (randlos, immer oben, voll durchsichtig):

```basic
SCREEN_TRANSPARENT(420, 420, "Overlay")
WINDOW_UNDECORATED(TRUE)
WINDOW_TOPMOST(TRUE)
WHILE NOT QUITREQUESTED()
    CLS()                       ' nur das Gezeichnete ist sichtbar
    CIRCLE(210, 210, 100, RGB(60, 200, 255))
    FLIP()
WEND
```

Beispiele: [`examples/123_overlay.dh`](../examples/123_overlay.dh) (Overlay), [`examples/124_glass_window.dh`](../examples/124_glass_window.dh) (Glas-Fenster), [`examples/125_vortex_overlay.dh`](../examples/125_vortex_overlay.dh) (Wirbel-Overlay), [`examples/126_audio_overlay.dh`](../examples/126_audio_overlay.dh) (klick-durchlässiger Musik-Visualizer via `AUDIO_FFT`).

> Hinweis: Transparenz wirkt im direkten Render-Pfad. Mit aktivem Post-Processing-Shader (`POSTFX`) wird der Bildschirm deckend präsentiert.

## Native Datei-Dialoge

Echte Windows-Dialoge zum Auswählen von Dateien/Ordnern. Alle drei sind **blockierend** (modaler Systemdialog) und geben den gewählten Pfad als STRING zurück — bei Abbruch einen **leeren String**. `endungen$` ist eine kommagetrennte Liste von Dateiendungen ohne Punkt (z. B. `"png,jpg,gb"`).

| Funktion | Zweck |
|---|---|
| `FILE_OPEN_DIALOG([titel$[, endungen$]])` → STRING | Datei zum Öffnen wählen |
| `FILE_SAVE_DIALOG([titel$[, default$[, endungen$]]])` → STRING | Datei zum Speichern wählen (`default$` = vorgeschlagener Dateiname) |
| `FOLDER_DIALOG([titel$])` → STRING | Ordner wählen |

```basic
DIM pfad AS STRING
pfad = FILE_OPEN_DIALOG("Bild laden", "png,jpg")
IF pfad <> "" THEN
    DIM bild AS IMAGE : bild = LOADIMAGE(pfad)
    ' ...
END IF
```

Komplettes Beispiel: [`examples/127_filedialog.dh`](../examples/127_filedialog.dh).

## Zeichnen

Farbe wird als 24-Bit-INTEGER (`&HRRGGBB`) angegeben, am einfachsten via `RGB(r, g, b)` oder über die [Farb-Konstanten](sprache.md#built-in-konstanten) (`RED`, `GREEN`, …).

| Funktion | Zweck |
|---|---|
| `PLOT(x, y[, color])` | einzelnes Pixel |
| `LINE(x1, y1, x2, y2[, color])` | Linie (1px) |
| `LINEW(x1, y1, x2, y2, breite[, color])` | Linie mit Strichbreite (Float) |
| `BOX(x1, y1, x2, y2[, color])` | gefülltes Rechteck |
| `RECT(x1, y1, x2, y2[, color])` | Rechteck-Rahmen (1px) |
| `BOXROUND(x1, y1, x2, y2, radius[, color])` | gefülltes Rechteck mit runden Ecken |
| `RECTROUND(x1, y1, x2, y2, radius[, color])` | Rechteck-Rahmen mit runden Ecken |
| `GRADIENTV(x1, y1, x2, y2, farbe1, farbe2)` | Rechteck mit **vertikalem** Farbverlauf (oben→unten) |
| `GRADIENTH(x1, y1, x2, y2, farbe1, farbe2)` | Rechteck mit **horizontalem** Farbverlauf (links→rechts) |
| `CIRCLE(x, y, r[, color])` | gefüllter Kreis |
| `CIRCLEOUTLINE(x, y, r[, color])` | Kreis nur als Kontur (Gegenstück zu `CIRCLE`) |
| `SPLINE(xs, ys[, color[, breite]])` | weiche Catmull-Rom-Kurve durch die Punkte; `xs`/`ys` sind `ARRAY OF INTEGER` gleicher Länge |
| `TRIANGLE(x1, y1, x2, y2, x3, y3[, color])` | gefülltes Dreieck |
| `TRIANGLEOUTLINE(x1, y1, x2, y2, x3, y3[, color[, width]])` | Dreieck nur als Kontur |
| `POLYGON(points[, color])` | gefülltes Polygon — `points` ist ein `ARRAY OF INTEGER` mit `[x1, y1, x2, y2, …]` (mind. 3 Punkte) |
| `POLYGONOUTLINE(points[, color[, width]])` | Polygon nur als Kontur |
| `ELLIPSE(x1, y1, x2, y2[, color])` | gefüllte Ellipse, eingepasst in die Bounding-Box |
| `ELLIPSEOUTLINE(x1, y1, x2, y2[, color[, width]])` | Ellipse nur als Kontur |
| `ARC(x1, y1, x2, y2, start_rad, end_rad[, color[, width]])` | Bogen-Segment in der Bounding-Box; Winkel in Radiant, gegen den Uhrzeigersinn. `width` (Float, optional) = Strichbreite; ohne Angabe 1px (zoom-unabhaengig), mit Angabe wie `LINEW`/`SPLINE` mit `CAMERA_SET`-Zoom skaliert |
| `TEXT(x, y, s$[, color])` | Text bei (x, y) |
| `TEXTROT(x, y, s$, winkel[, skala[, farbe]])` | Text **zentriert** auf (x, y), um das Zentrum gedreht (Grad, wie `DRAWIMAGEROT`) und skaliert — für Score-Popups, schräge Labels. Nutzt aktiven Font/Größe |

> **Eckpunkt-Reihenfolge egal:** gefülltes `TRIANGLE` und `POLYGON` zeichnen
> unabhängig von der Wicklung — ob die Punkte im oder gegen den Uhrzeigersinn
> angegeben sind, die Fläche erscheint immer (dhrt dreht intern bei Bedarf um).

```basic
SCREEN(320, 240, "Zeichnen-Demo", 2)

CLS(RGB(20, 20, 30))

PLOT(100, 50, RGB(255, 255, 255))
LINE(0, 0, 319, 239, RGB(255, 0, 0))
BOX(50, 50, 150, 100, RGB(0, 200, 0))     ' gefuellt
RECT(50, 110, 150, 160, RGB(0, 200, 0))   ' Rahmen
CIRCLE(200, 80, 30, RGB(255, 200, 0))
TRIANGLE(20, 200, 60, 200, 40, 230, RGB(255, 100, 255))

' Polygon mit Punkt-Array
DIM pent[10] AS INTEGER
pent[0] = 100 : pent[1] = 200 : pent[2] = 140 : pent[3] = 175
pent[4] = 180 : pent[5] = 200 : pent[6] = 165 : pent[7] = 245
pent[8] = 115 : pent[9] = 245
POLYGON(pent, RGB(0, 200, 200))

ELLIPSE(220, 180, 290, 230, RGB(0, 0, 255))
ARC(180, 180, 280, 280, 0.0, 3.14, RGB(255, 255, 0), 2)
TEXT(10, 200, "Hallo", RGB(255, 255, 255))

FLIP()
SLEEP(2000)
```

**Hinweis zu BOX/RECT:** beide nehmen `(x1, y1, x2, y2)` als Eckpunkte (inklusive). `BOX(0, 0, 9, 9)` zeichnet 10×10 Pixel.

## Transparenz und Blend-Modi

Farben können einen **Alpha-Kanal** tragen (`&Haarrggbb`, oberstes Byte = Deckkraft):

| Funktion | Zweck |
|---|---|
| `RGBA(r, g, b, a)` → INTEGER | Farbe mit Alpha (`a` 0..255, 255 = voll deckend) |
| `ALPHA(farbe)` → INTEGER | Alpha-Kanal (0..255) aus einer Farbe lesen |
| `BLEND_MODE(modus$)` | Mischmodus für folgende Draws: `"alpha"` (Standard), `"add"` (additiv – Glow/Licht), `"mult"` (multiplikativ – Schatten/Tönung), `"subtract"` |

`BLEND_MODE` gilt bis zum nächsten Aufruf — nach einem Effekt mit `BLEND_MODE("alpha")` zurückstellen.

```basic
' Glühende Funken: additiv überlagern -> helle Überlappungen
BLEND_MODE("add")
FOR i = 0 TO 20
    CIRCLE(RND(320), RND(240), 8, RGBA(255, 180, 60, 120))
NEXT
BLEND_MODE("alpha")          ' zurück zum Normal-Modus
```

## Clip-Rechteck (Scissor)

| Funktion | Wirkung |
|---|---|
| `SCISSOR(x, y, breite, hoehe)` | Folgende Draws werden auf dieses Rechteck beschnitten |
| `SCISSOR_END()` | Die oberste Beschränkung zurücknehmen |
| `SCISSOR_DEPTH()` → INTEGER | Wie viele Clips gerade offen sind |

Das ist ein **Stapel**: ein inneres `SCISSOR` wird mit dem äußeren
*geschnitten*, es ersetzt es nicht. Die Koordinaten sind Welt-Koordinaten
und folgen der Kamera wie jeder andere Draw.

```basic
' Eine scrollbare Liste: nur der Ausschnitt ist zu sehen
SCISSOR(20, 40, 200, 120)
FOR i = 0 TO 30
    TEXT(24, 44 + i * 20 - versatz, "Zeile " + STR$(i))
NEXT
SCISSOR_END()
```

Ein `SCISSOR_END()` ohne offenes `SCISSOR` ist ein **Fehler** — es nähme
sonst dem umgebenden Code (etwa einem `gui`-Fenster) seine Beschränkung weg,
und das fiele erst als Zeichnen über den Rand hinaus auf. Ein vergessenes
`SCISSOR_END()` wirkt dagegen nur bis zum Bildende.

Ohne diesen Befehl blieb für einen begrenzten Zeichenbereich nur der Umweg
über ein Render-Target (`RENDERTARGET_NEW`/`BEGIN`/`END`/`DRAW`) — also eine
zweite Zeichenfläche samt Speicher, wo ein Rechteck genügt.

## Schrift

| Funktion | Wirkung |
|---|---|
| `TEXT(x, y, s$[, color])` | Text bei (x, y) in der aktiven Schrift |
| `TEXT_SIZE(px)` | Schriftgröße für folgende `TEXT`-Aufrufe (4–400) |
| `TEXT_WIDTH(s$)` | Pixelbreite von `s$` in der aktiven Schrift/Größe |
| `TEXT_HEIGHT()` | Zeilenhöhe der aktiven Schrift |
| `TEXT_BOLD(an)` / `TEXT_ITALIC(an)` | Fett/Kursiv (nativ No-Op — raylib ohne Fett/Kursiv) |
| `LOADFONT(pfad$, groesse[, zeichen$])` → FONT | TTF/OTF/TTC laden → FONT-Handle (INTEGER); `zeichen$` = Schriftblöcke (`"kyrillisch, griechisch"`, `"japanisch"`, `"emoji"` …) oder die Zeichen selbst, die gebacken werden sollen |
| `SETFONT(font)` | aktive Schrift setzen; `SETFONT(-1)` = Default-Font |
| `TEXT_SPACING(px)` | Buchstabenabstand für TTF (nativ) |
| `TEXT_LINE_SPACING(px)` | Zeilenabstand für mehrzeiligen Text |

`LOADFONT` lädt eine eigene TrueType-/OpenType-Schrift; `TEXT_SIZE` skaliert sie
anschließend frei. `TEXT_WIDTH` misst in der **aktiven** Schrift — damit lässt
sich zentrieren/rechtsbündig setzen. Echte Glyphen rendert die native Runtime
(raylib `LoadFontEx`/`DrawTextEx`).

```basic
DIM titlefont AS INTEGER
titlefont = LOADFONT("assets/PressStart2P.ttf", 32)
SETFONT(titlefont)
TEXT_SIZE(32)
DIM w AS INTEGER
w = TEXT_WIDTH("GAME OVER")
TEXT(320 - w \ 2, 100, "GAME OVER", RGB(255, 60, 60))   ' zentriert
SETFONT(-1)                                             ' zurueck zum Default
```

Demo: [examples/87_ttf_fonts.dh](../examples/87_ttf_fonts.dh).

### Umlaute, Euro und fremde Schriften

`TEXT(x, y, "Köln")` zeichnet **Köln**, nicht `K?ln`, und `TEXT(x, y, "12,50 €")`
zeigt das Euro-Zeichen — bis September 2026 war es ein Fragezeichen, ebenso
`ő`, `ł`, `Ω`, `Я`, jedes Kanji und jedes Emoji (gemessen in
[entwurf-eingabemethoden.md](entwurf-eingabemethoden.md)). Dafür sorgen drei
Dinge:

- **Der Grund-Zeichenvorrat** jeder Schrift — der Ausweich-Schrift und jeder
  per `LOADFONT` geladenen — umfasst ASCII, Latin-1, Latin Extended-A/B (die
  Sprachen Mitteleuropas), Griechisch, Kyrillisch, die allgemeine
  Interpunktion (`… – „ “`) und `€`. Enthält die Schriftdatei ein Zeichen
  nicht, zeichnet raylib dort ein `?`; eine Pixel-Schrift für ein Retro-Spiel
  hat oft keine Umlaute.
- **Ohne eigene Schrift springt eine Ausweich-Schrift ein.** Die eingebaute
  raylib-Schrift kennt nur ASCII. Kommt ein Zeichen darüber hinaus vor,
  zeichnet die Runtime diesen Text mit einer Systemschrift (Windows: Segoe
  UI, macOS: SF/Helvetica, Linux: DejaVu/Liberation). Reiner ASCII-Text geht
  weiterhin durch die eingebaute Schrift.
- **Glyphen auf Zuruf.** Steht ein Zeichen in keiner geladenen Schrift
  (Kanji, Hangul, Emoji, Arabisch, Hebräisch, Thai), merkt sich die Runtime
  es beim Zeichnen oder Messen und backt es beim nächsten `FLIP` aus der
  passenden Systemschrift nach — Windows: MS Gothic, Malgun Gothic, Segoe UI
  Emoji, Segoe UI; macOS: Arial Unicode; Linux: Noto Sans CJK, sofern
  installiert. Gebacken wird nur, was gebraucht wurde, nicht der ganze Block.
  Gemessen: das erste Bild mit Kanji, Hangul, Emoji und Hebräisch zugleich
  kostet einmalig etwa 100 ms, jedes weitere neue Zeichen etwa 15 ms, danach
  nichts mehr. Das gilt auch in einer selbst geladenen Schrift: fehlt ihr ein
  Zeichen, springt für genau dieses Zeichen die Ausweich-Schrift ein.

Wer den Vorrat selbst bestimmen will, gibt ihn `LOADFONT` als drittes
Argument mit — Blocknamen, deutsch oder englisch, durch Komma getrennt:
`latein`, `griechisch`, `kyrillisch`, `hebraeisch`, `arabisch`, `thai`,
`japanisch`, `chinesisch`, `koreanisch`, `emoji`, `symbole`. Oder gleich die
Zeichen, die das Programm braucht (bei einem Spiel mit festen Texten das
Billigste). Der Grundvorrat ist immer dabei. Ein unbekannter Name ist ein
Fehler, der die bekannten aufzählt.

```basic
DIM jp AS INTEGER
jp = LOADFONT("C:/Windows/Fonts/msgothic.ttc", 24, "japanisch")
SETFONT(jp)
TEXT(10, 10, "東京 こんにちは")
DIM ru AS INTEGER
ru = LOADFONT("assets/schrift.ttf", 20, "kyrillisch, griechisch")
DIM titel AS INTEGER
titel = LOADFONT("assets/titel.ttf", 48, "SPIEL VORBEI 0123456789")   ' nur diese Zeichen
```

**Schriftsammlungen (`.ttc`)** gehen seit demselben Datum: raylib selbst
kann sie nicht lesen und tauschte die Schrift bisher **still** gegen seine
Bitmapschrift — `LOADFONT` gab ein Handle zurück, und der Text erschien in
der falschen Schrift ohne Meldung. Die Runtime löst jetzt die erste Schrift
der Sammlung heraus (die CJK-Schriften von Windows liegen alle nur so vor).
Eine Datei, die trotzdem keine Schrift ergibt, ist ein Fehler.

**Grenzen:** Emoji kommen einfarbig (raylib rastert keine Farbschriften);
Arabisch und Hebräisch erscheinen Zeichen für Zeichen von links nach rechts
ohne Verbindung der Buchstaben — Textformung und Rechts-nach-links sind
nicht gebaut. Wer eine bestimmte Glyphenform will (japanische statt
chinesische Formen), lädt seine Schrift selbst.

`TEXT_WIDTH` misst denselben Weg, den `TEXT` zeichnet — auch über die
Ausweich-Schriften hinweg: ein zentrierter Text mit Umlaut oder Kanji sitzt
dort, wo er gemessen wurde.

**Eingabemethoden (IME).** Wer Japanisch oder Chinesisch über eine
Eingabemethode tippt, bekommt das Bestätigte in ein `gui`-Textfeld wie
getippt; die Tipp-Warteschlange fasst 256 Zeichen je Bild (raylibs Vorgabe
von 16 hätte einen bestätigten Satz still gekürzt), und unter Windows steht
das Umwandlungsfenster der IME an der Schreibmarke des Feldes mit Fokus. Eine
Vorschau der Umwandlung im Feld selbst gibt es nicht.

## Bilder

| Funktion | Zweck |
|---|---|
| `LOADIMAGE(path$)` → IMAGE | Datei laden (PNG, JPG, BMP, …) |
| `IMAGEWIDTH(img)`, `IMAGEHEIGHT(img)` → INTEGER | Pixelgröße |
| `GETPIXEL(img, x, y)` → INTEGER | Pixelfarbe (`&HRRGGBB`) an `(x, y)` lesen; `-1` bei Index außerhalb. Gegenstück zu `PLOT` (schreiben) — für Kollision per Pixel, Maskierung, Farb-Sampling |
| `GETALPHA(img, x, y)` → INTEGER | Deckkraft an `(x, y)`, `0..255`; `-1` bei Index außerhalb. Nötig, weil `GETPIXEL` eine **Farbe** liefert und dort Deckkraft 0 *deckend* bedeutet — ein durchsichtiger Punkt käme als schwarzer zurück |
| `DRAWIMAGE(img, x, y)` | Bild bei (x, y) zeichnen |
| `DRAWIMAGEPART(img, sx, sy, sw, sh, x, y)` | Sub-Rechteck aus Sheet zeichnen |
| `DRAWIMAGEFLIPPED(img, x, y[, flipX[, flipY]])` | mit Spiegelung |
| `DRAWIMAGEROT(img, x, y, winkel[, skala[, tint]])` | **zentriert** auf (x,y), um `winkel` **Grad** gedreht (um die Mitte), optional skaliert + getönt. Ideal für rotierte Sprites / `physics2d` (`winkel = DEG(PHYS2D_BODY_ANGLE(...))`). Camera-aware. |

```basic
SCREEN(320, 240, "Bilder", 2)

DIM hero AS IMAGE
hero = LOADIMAGE("assets/hero.png")
PRINT "Bild ist ", IMAGEWIDTH(hero), "x", IMAGEHEIGHT(hero)

WHILE NOT QUITREQUESTED()
    CLS()
    DRAWIMAGE(hero, 100, 100)
    DRAWIMAGEFLIPPED(hero, 150, 100, TRUE, FALSE)   ' horizontal gespiegelt
    FLIP()
    SLEEP(16)
WEND
```

Für animierte Sprites mit Frame-Logik siehe [Sprite-Modul](module-sprite.md). Für Effekte wie Skalieren, Rotieren, Tinten siehe [imgfx-Modul](module-imgfx.md).

**Asset-Cache:** `LOADIMAGE` und `LOADSOUND` cachen ihre Ergebnisse automatisch. Mehrfache Aufrufe mit demselben (oder gleichbedeutendem) Pfad liefern dasselbe Bild zurück, ohne Disk-IO. Cache-Schlüssel sind sowohl der rohe Pfad als auch der normalisierte Absolut-Pfad, sodass verschiedene Schreibweisen (`"x.png"`, `"./x.png"`, absolut) denselben Eintrag treffen.

## Asset-Preloader

`LOAD_ASSETS(manifest_path$)` → INTEGER

Lädt alle Bilder und Sounds aus einem JSON-Manifest vorab in den Cache. Nach dem Preload sind alle nachfolgenden `LOADIMAGE`/`LOADSOUND`-Aufrufe Cache-Hits (kein Disk-IO mehr). Liefert die Gesamtanzahl geladener Assets.

**Manifest-Format:**

```json
{
  "images": {
    "player": "sprites/player.png",
    "enemy":  "sprites/enemy.png"
  },
  "sounds": [
    "sfx/jump.wav",
    "music/level1.ogg"
  ]
}
```

Beide Sektionen sind optional. Jede kann **Object** (Alias → Pfad) ODER **Liste** (nur Pfade) sein.

**Pfade** sind relativ zum Manifest-Verzeichnis (nicht zum laufenden Skript). So kann man `assets/manifest.json` zentral pflegen.

**Aliasing:** Bei Dict-Form trifft sowohl `LOADIMAGE("player")` als auch `LOADIMAGE("sprites/player.png")` den Cache. Das macht den Skript-Code lesbar (`LOADIMAGE("player")`), während im Manifest die echten Pfade stehen.

**Reihenfolge:** Nach `SCREEN(...)` aufrufen, damit Bilder direkt `convert_alpha`-optimiert werden.

```basic
SCREEN(640, 480, "Mein Spiel")

' Einmal: alle Assets laden
DIM n AS INTEGER
n = LOAD_ASSETS("assets/manifest.json")
PRINT n; " Assets geladen"

' Im Spiel: per Alias darauf zugreifen
DIM hero AS IMAGE
hero = LOADIMAGE("player")    ' Cache-Hit (Alias)
```

Vollständiges Beispiel: [examples/75_preloader.dh](../examples/75_preloader.dh).

## Sprite-Atlas

Ein **Sprite-Atlas** ist EIN großes Bild mit benannten Sub-Rects (`x, y, w, h`). Statt 50 einzelner PNG-Dateien hat man ein Atlas-PNG + ein Manifest — und spricht die Teile mit ihrem Namen an statt mit selbst ausgerechneten Rechtecken.

| Funktion | Zweck |
|---|---|
| `ATLAS_LOAD(manifest_path$)` → SPRITE_ATLAS | Atlas aus JSON-Manifest laden |
| `ATLAS_DRAW(atlas, name$, x, y)` | einzelnes Sub-Sprite zeichnen (Camera-aware) |
| `ATLAS_DRAW_FLIPPED(atlas, name$, x, y[, flip_x[, flip_y[, tint]]])` | Sub-Sprite mit Spiegelung (X/Y, je `TRUE`/`FALSE` oder `1`/`0`); optional `tint` |
| `BATCH_DRAW(atlas, name$, x, y)` | **Zweitname für `ATLAS_DRAW`** — zeichnet sofort, sammelt nichts |
| `BATCH_FLUSH()` | **tut nichts** (No-Op) — nur damit alter Code weiterläuft |

**Manifest-Format:**

```json
{
  "image": "tiles.png",
  "sprites": {
    "tile_grass": [0,  0, 16, 16],
    "tile_water": [16, 0, 16, 16],
    "player":     [0, 16, 24, 32]
  }
}
```

Rects sind `[x, y, w, h]` (Pixel im Atlas). Image-Pfad relativ zum Manifest.

**Pattern für Tilemaps** (viele Sprites pro Frame):

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/tiles_atlas.json")

' Pro Frame 600 Tiles -- jeder Aufruf zeichnet sofort (siehe Hinweis unten).
FOR row = 0 TO 19
    FOR col = 0 TO 29
        BATCH_DRAW(atlas, "tile_grass", col * 16, row * 16)
    NEXT
NEXT
BATCH_FLUSH()   ' No-Op -- gezeichnet wurde schon oben, Zeile fuer Zeile
```

> **Kein echtes Bündeln.** In `dhrt` ist `BATCH_DRAW` derselbe Aufruf wie `ATLAS_DRAW`
> (ein Zweig im Dispatch), und `BATCH_FLUSH()` tut gar nichts. Die Runtime arbeitet als
> **Aufzeichnungs-Modell**: jeder Zeichenbefehl hängt sofort ein `Cmd` an die aktive
> Ebene, und `FLIP()` spielt alle Ebenen in Z-Reihenfolge ab. Es gibt also nichts zu
> sammeln und **keine Ersparnis an Zeichenaufrufen** — bau deine Darstellung nicht
> darauf. Die Namen stammen aus der früheren pygame-Engine, wo sich Surface-Blits
> tatsächlich zu einem Aufruf zusammenfassen ließen; sie bleiben nur erhalten, damit
> alter Code weiterläuft. Wer wirklich viele gleichartige Dinge zeichnet, nimmt die
> Massen-Builtins (`PLOTS`, `BOXES`, `CIRCLES`, `LINES`) — die sparen echten Aufwand.
> Ein Zoom über `CAMERA_SET` wirkt auf `ATLAS_DRAW`/`BATCH_DRAW` gleichermaßen.

**Flipping für Charakter-Sprites:** `ATLAS_DRAW_FLIPPED(atlas, name$, x, y[, flip_x[, flip_y[, tint]]])` spiegelt das Sub-Sprite an X- oder Y-Achse. `flip_x`/`flip_y` akzeptieren `TRUE`/`FALSE` **oder** `1`/`0` (fehlend = `FALSE`); `tint` ist ein optionaler 7. Farb-Parameter. Klassisches Pattern für Walk-Animationen: nur eine Richtung (rechts) im Atlas, links wird per Flip abgeleitet:

```basic
DIM flip AS BOOLEAN
flip = CHAR_FACING(player) = -1     ' -1 = nach links
ATLAS_DRAW_FLIPPED(mario, "walk_a", x, y, flip, FALSE)
```

Flip erzeugt pro Aufruf ein frisch gespiegeltes Bild. Für viele wiederholte Flips desselben Sprites lohnt es sich, die gespiegelte Variante einmal vorberechnet als IMAGE zu cachen — für einen einzelnen Player-Sprite pro Frame ist der Overhead aber vernachlässigbar.

Vollständiges Beispiel: [examples/76_layers_atlas.dh](../examples/76_layers_atlas.dh).

## Z-Layer-Rendering

Z-Layer geben dir explizite Render-Reihenfolge: Hintergrund → Sprites → UI, ohne dass du die Draw-Reihenfolge im Code akribisch verwalten musst.

Ein Layer ist eine **Befehlsliste** mit einem z-Wert — kein Bildspeicher. Jeder
Zeichenaufruf hängt einen Eintrag an die gerade aktive Liste; `FLIP` sortiert die
Listen nach z (niedrigstes = hinten) und spielt sie in dieser Reihenfolge ab.
Das ist dasselbe Aufzeichnungs-Modell wie bei
[`BATCH_DRAW`](#sprite-atlas) — es gibt keine Zwischenbilder, die man auslesen
oder einzeln überblenden könnte.

| Funktion | Zweck |
|---|---|
| `LAYER_DEFINE(name$, z)` | Layer mit explizitem z registrieren (re-define aktualisiert nur z) |
| `LAYER(name$)` | aktiven Draw-Target auf Layer umschalten (auto-Define wenn neu) |
| `LAYER_END()` | zurück zum Main-Buffer (optional, FLIP macht's auch) |
| `LAYER_CLEAR(name$)` | Liste eines Layers manuell leeren (selten, `FLIP` leert ohnehin alle) |

**Klassisches Game-Loop-Pattern:**

```basic
LAYER_DEFINE("bg",      0)
LAYER_DEFINE("sprites", 10)
LAYER_DEFINE("ui",      100)

WHILE NOT QUITREQUESTED()
    LAYER("bg")
    CLS(RGB(20, 20, 50))
    DRAWIMAGE(parallax, 0, 0)

    LAYER("sprites")
    DRAWIMAGE(player, px, py)
    DRAWIMAGE(enemy, ex, ey)

    LAYER("ui")
    TEXT(10, 10, "Score: " + STR$(score))

    FLIP()    ' spielt die Ebenen in z-Order ab und leert sie danach
WEND
```

**Nach jedem `FLIP` sind die Listen leer** — du musst sie nicht selbst leeren. Wo
ein Layer nichts gezeichnet hat, bleibt schlicht sichtbar, was darunter liegt.
Wenn ein Layer einen deckenden Hintergrund haben soll (z.B. der bg-Layer):
`CLS(...)` als ersten Zeichenaufruf auf dem Layer.

**Backwards-Compat:** Code ohne `LAYER_*`-Calls läuft unverändert direkt auf den Main-Buffer.

**Camera-Hinweis:** Camera ist global, gilt für alle Layer. Für UI ohne Camera-Effekt (z.B. HUD): vor dem `LAYER("ui")`-Draw `CAMERA_RESET()` rufen (benötigt `IMPORT "camera"`).

**Layer + Atlas kombiniert** (vollständig in [examples/76_layers_atlas.dh](../examples/76_layers_atlas.dh)):

```basic
LAYER("bg")
FOR each tile:
    BATCH_DRAW(atlas, "tile_grass", x, y)
NEXT
BATCH_FLUSH()     ' No-Op; die Tiles haengen laengst an der bg-Liste
LAYER("ui")       ' schaltet nur die aktive Liste um -- nichts geht verloren
TEXT(10, 10, "Score: " + STR$(score))
FLIP()
```

## Tilemap

`DRAWTILEMAP(tileset, map, tileW, tileH, screenX, screenY)`

Zeichnet eine 2D-Karte aus Tile-Indizes. `tileset` ist ein Sheet, dessen Frames horizontal+vertikal angeordnet sind. `map` ist ein 2D `ARRAY OF INTEGER` mit Tile-Nummern (`-1` = transparent, kein Tile).

```basic
SCREEN(320, 240, "Tilemap-Demo", 2)

DIM tileset AS IMAGE
tileset = LOADIMAGE("assets/tileset.png")    ' z.B. 64x16, 4 Tiles a 16x16

DIM map[10, 15] AS INTEGER
DIM r AS INTEGER
DIM c AS INTEGER
FOR r = 0 TO 9
    FOR c = 0 TO 14
        IF r = 0 OR r = 9 OR c = 0 OR c = 14 THEN
            map[r, c] = 1                    ' Wand-Tile
        ELSE
            map[r, c] = 0                    ' Boden-Tile
        END IF
    NEXT
NEXT

WHILE NOT QUITREQUESTED()
    CLS()
    DRAWTILEMAP(tileset, map, 16, 16, 0, 0)
    FLIP()
    SLEEP(16)
WEND
```

`map[r, c]` ist `[zeile, spalte]`. Auflösung der Tile-Frames im Sheet: `tiles_pro_zeile = sheet_breite / tileW`.

## Sound und Musik

| Funktion | Zweck |
|---|---|
| `LOADSOUND(path$)` → SOUND | Sample laden (kurze Effekte: WAV, OGG) |
| `PLAYSOUND(s[, loops[, volume]])` | abspielen, default loops=0, volume=1.0 |
| `STOPSOUND(s)` | stoppen |
| `UNLOADSOUND(s)` | Sound stoppen **und seinen Puffer freigeben** — gegen Puffer-Akkumulation bei vielen `AUDIO_TONE`/`AUDIO_SFX`/`AUDIO_NOISE`-Noten (langer Song). Der Handle bleibt gültig, erneutes Abspielen wirft einen Fehler |
| `AUDIO_SOUND_COUNT()` → INTEGER | Anzahl lebender (nicht freigegebener) Sound-Slots — Diagnose gegen Sound-Lecks |
| `PLAYMUSIC(path$[, loops[, volume]])` | längere Datei streamen (`.ogg`/`.mp3`/`.qoa` **+ Tracker-Module `.mod`/`.xm`** — echter Amiga-Sound), default loops=-1 (endlos) |
| `STOPMUSIC()` | stoppen |

```basic
DIM coin_snd AS SOUND
coin_snd = LOADSOUND("assets/pickup.wav")

PLAYMUSIC("assets/menu.ogg", -1, 0.7)        ' Endlos, 70% Lautstärke

' Im Game-Loop, wenn Coin gesammelt:
PLAYSOUND(coin_snd)
```

## Viele Formen auf einmal

Wer tausend Sterne zeichnet, ruft nicht tausendmal `PLOT`. Die Massen-Befehle
nehmen Arrays und erledigen alles in einem Aufruf — das spart den Aufwand pro
Befehl, nicht das Zeichnen selbst.

| Funktion | Zweck |
|---|---|
| `PLOTS(xs, ys, farbe [, anzahl])` | viele Pixel; `farbe` als Zahl gilt für alle, als ARRAY je Pixel |
| `BOXES(x1s, y1s, x2s, y2s, farbe [, anzahl])` | viele Rechtecke |
| `CIRCLES(xs, ys, rs, farbe [, anzahl])` | viele Kreise |
| `LINES(x1s, y1s, x2s, y2s, farbe [, anzahl])` | viele Linien |

**`anzahl` weglassen heißt „das ganze Array"** — ein fest dimensionierter Puffer
schleppt seine ungenutzten Plätze also mit ins Bild. Wer nur die ersten `n`
Einträge belegt hat, muss `n` mitgeben.

## Texturen erzeugen

Bilder ohne Bilddatei — für Hintergründe, Lichter und Muster, die man nicht
mitliefern will. Alle liefern ein IMAGE, das sich wie ein geladenes zeichnen
lässt.

| Funktion | Zweck |
|---|---|
| `GENTEX_COLOR(breite, hoehe, farbe)` | einfarbige Fläche |
| `GENTEX_GRADIENT(breite, hoehe, farbe1, farbe2 [, vertikal])` | linearer Verlauf |
| `GENTEX_GRADIENT_BOX(breite, hoehe, dichte, farbe1, farbe2)` | rechteckiger Verlauf von innen nach außen — Vignetten |
| `GENTEX_RADIAL(breite, hoehe, innen, aussen [, dichte])` | runder Verlauf von der Mitte nach außen — weiche Lichter und Glows, additiv gezeichnet |
| `GENTEX_CHECKED(breite, hoehe, feld_x, feld_y, farbe1, farbe2)` | Schachbrett |
| `GENTEX_PERLIN(breite, hoehe, skala)` | Perlin-Rauschen — Wolken, Gelände, Marmor |
| `GENTEX_CELLULAR(breite, hoehe, kachel)` | Zellrauschen (Voronoi) — Steinboden, Risse, Schuppen |
| `GENTEX_NOISE(breite, hoehe, anteil)` | Weißrauschen — Sternenfelder, Filmkorn |

## Render-Ziele

Ein Render-Ziel ist eine Fläche außerhalb des Bildschirms, auf die man zeichnet
wie auf das Fenster — für Spiegel, Minikarten, Bild-im-Bild oder Nachzieheffekte.

| Funktion | Zweck |
|---|---|
| `RENDERTARGET_NEW(breite, hoehe [, behalten])` → INTEGER | Ziel anlegen. `behalten = TRUE` lässt den Inhalt über das Bild hinaus stehen — die Voraussetzung für Schweife |
| `RENDERTARGET_BEGIN(ziel)` | ab jetzt dorthin zeichnen |
| `RENDERTARGET_END()` | zurück auf das Fenster |
| `RENDERTARGET_DRAW(ziel, x, y [, skala [, tint [, gespiegelt]]])` | das Ziel wie ein Bild aufs Fenster stempeln |
| `RENDERTARGET_CLEAR(ziel [, farbe])` | von Hand leeren — nötig, wenn `behalten` an ist |

Ohne `behalten` wird das Ziel zu Beginn jedes Bildes durchsichtig geleert. Ein
Schweif entsteht dagegen so: `behalten = TRUE`, und pro Bild ein
`BLEND_MODE("mult")` mit einer dunkelgrauen Vollbild-`BOX` darüber — das lässt
Altes verblassen, statt es zu löschen.

Ein Ziel kann sich **nicht selbst** zeichnen; ein `RENDERTARGET_DRAW` innerhalb
eines anderen Ziels tut nichts.

## Shader und Post-Processing

| Funktion | Zweck |
|---|---|
| `SHADER_LOAD(pfad_oder_glsl$)` → INTEGER | Fragment-Shader laden — aus einer Datei oder direkt als Quelltext; `-1` bei Fehler |
| `POSTFX(shader)` | jedes fertige Bild durch diesen Shader schicken; `-1` schaltet ab |
| `SHADER_SET(shader, name$, wert)` | eine Zahl an ein `uniform float` geben |
| `SHADER_SET2(shader, name$, x, y)` | zwei Werte (`vec2`) |
| `SHADER_SET3(shader, name$, x, y, z)` | drei Werte (`vec3`) |
| `SHADER_SET_ARRAY(shader, name$, werte)` | ein `uniform float[]` aus einem `ARRAY OF FLOAT` füllen — Lichtpositionen, Verlaufsstufen |
| `SHADER_SET_MATRIX(shader, name$, mat)` | eine `MAT4` aus [`m3d`](module-m3d.md) übergeben |
| `SHADER_SET_TEXTURE(shader, name$, bild)` | einen **zweiten** Sampler belegen — Masken, Paletten, Überblendungen |

`SHADER_SET_TEXTURE` wird gemerkt und erst beim Zeichnen gesetzt. Der Grund ist
eine Eigenheit von raylib: die Zuweisung wirkt auf das *gerade aktive*
Shader-Programm, außerhalb des Zeichnens landete sie also am falschen — und der
Sampler bliebe schwarz.

## Eingabe: Tastatur und Maus

| Funktion | Zweck |
|---|---|
| `KEYPRESSED(code)` → BOOLEAN | TRUE **solange** die Taste mit SDL-Code `code` gehalten wird — jedes Frame erneut. Für Bewegen/Lenken. |
| `KEYHIT(code)` → BOOLEAN | TRUE nur in **dem einen Frame**, in dem die Taste heruntergeht. Für Schießen, Springen, Umschalten. |
| | *Buchstaben:* `KEY_A`…`KEY_Z` sind die klare Schreibweise. `ASC("s")` und `ASC("S")` meinen beide dieselbe Taste — bis 2026-08-31 traf die **große** Schreibweise still gar nichts, was wie ein vergessener Aufruf aussah. |
| `KEYRELEASED(code)` → BOOLEAN | TRUE in dem Frame, in dem die Taste losgelassen wird — etwa „aufladen und beim Loslassen feuern“ |
| `KEYREPEAT(code)` → BOOLEAN | wie `KEYHIT`, feuert beim Halten aber zusätzlich mit der System-Tastenwiederholung. Für Textcursor und Mengen-Eingabe. |
| `MOUSEX()`, `MOUSEY()` → INTEGER | aktuelle Mausposition (in logischen Pixeln) |
| `MOUSEBUTTON(n)` → BOOLEAN | TRUE wenn Maustaste n gedrückt — **`0`=links, `1`=rechts, `2`=mitte** (raylib-Reihenfolge: rechts vor mitte!) |
| `MOUSEWHEEL()` → INTEGER | Mausrad-Delta seit dem letzten Aufruf (+ hoch / − runter / 0) |
| `MOUSE_VISIBLE(an)` | OS-Cursor zeigen/verstecken — verstecken, wenn das Spiel ein eigenes Fadenkreuz/Cursor-Sprite zeichnet |
| `MOUSE_LOCK(an)` | Cursor **fangen**: verstecken + im Fenster einsperren (relative Bewegung) — für First-Person-/Kamera-Maussteuerung; `FALSE` gibt frei |
| `MOUSE_HIDDEN()` → BOOLEAN | ist der Cursor gerade versteckt/gefangen? |
| `SCREENWIDTH()`, `SCREENHEIGHT()` → INTEGER | logische Fenstergröße (wie an `SCREEN` übergeben); 0 vor `SCREEN` |
| `MOUSE_HIT(n)` → BOOLEAN | TRUE nur in **dem einen Frame**, in dem die Maustaste heruntergeht |
| `MOUSE_RELEASED(n)` → BOOLEAN | TRUE in dem Frame, in dem sie losgelassen wird |
| `MOUSE_DELTA_X()`, `MOUSE_DELTA_Y()` → FLOAT | wie weit sich die Maus seit dem letzten Bild bewegt hat — bei `MOUSE_LOCK` stehen `MOUSEX`/`MOUSEY` still, nur das hier bewegt sich noch |
| `MOUSEWHEEL_X()`, `MOUSEWHEEL_Y()` → FLOAT | Rad in **beiden** Achsen und als Kommazahl; `MOUSEWHEEL()` kennt nur die senkrechte und rundet — feine Touchpad-Schritte fallen dort auf 0 |
| `MOUSE_SET_POS(x, y)` | Zeiger an eine Stelle setzen |
| `MOUSE_ON_SCREEN()` → BOOLEAN | ist der Zeiger überhaupt im Fenster? |
| `MOUSE_CURSOR(form$)` | Zeigerform: `default`, `ibeam`, `crosshair`, `hand`, `resize_ew`, `resize_ns`, `resize_nwse`, `resize_nesw`, `resize_all`, `not_allowed` |
| `KEY_ANY_HIT()` → INTEGER | Code der zuletzt gedrückten Taste, `-1` = keine — fuer Belegungsdialoge |
| `KEY_NAME$(code)` → STRING | Anzeigename einer Taste (`LEER`, `LINKS`, `UMSCHALT`, `F5` …) |
| `INKEY$()` → STRING | zuletzt getipptes Zeichen oder `""` — wartet **nicht**, für Texteingabe im Spielablauf |
| `WAITKEY()` → INTEGER | **hält an**, bis eine Taste kommt, und liefert ihren Code (`-1`, wenn das Fenster geschlossen wird) |

**Tasten-Konstanten** (`KEY_*`) sind eingebaut:

```basic
IF KEYPRESSED(KEY_LEFT) THEN
    spieler_x = spieler_x - 2
END IF
IF KEYPRESSED(KEY_SPACE) THEN
    schiessen()
END IF
IF KEYPRESSED(KEY_ESCAPE) THEN
    BREAK
END IF
```

Verfügbare Konstanten:

| Gruppe | Konstanten |
|---|---|
| Sondertasten | `KEY_ESCAPE`, `KEY_RETURN`/`KEY_ENTER`, `KEY_SPACE`, `KEY_TAB`, `KEY_BACKSPACE` |
| Pfeile | `KEY_LEFT`, `KEY_RIGHT`, `KEY_UP`, `KEY_DOWN` |
| Buchstaben/Ziffern | `KEY_A` bis `KEY_Z`, `KEY_0` bis `KEY_9` |
| Funktionstasten | `KEY_F1` bis `KEY_F12` |
| Modifier | `KEY_LSHIFT`, `KEY_RSHIFT`, `KEY_LCTRL`, `KEY_RCTRL`, `KEY_LALT`, `KEY_RALT`, `KEY_LSUPER`, `KEY_RSUPER`, `KEY_CAPSLOCK` |
| Navigation | `KEY_INSERT`, `KEY_DELETE`, `KEY_HOME`, `KEY_END`, `KEY_PAGEUP`, `KEY_PAGEDOWN` |
| Ziffernblock | `KEY_KP0` bis `KEY_KP9`, `KEY_KP_ENTER`, `KEY_KP_PLUS`, `KEY_KP_MINUS`, `KEY_KP_MULTIPLY`, `KEY_KP_DIVIDE`, `KEY_KP_PERIOD` |

Der Ziffernblock hat eigene Codes — eine Spielsteuerung darf ihn getrennt von
der oberen Ziffernreihe belegen. Gamepad-Codes (`JOY_BUTTON_A`, `JOY_DPAD_UP`
und Verwandte) stehen in [module-input.md](module-input.md).


```basic
SCREEN(320, 240, "Maus-Demo", 2)

WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 20, 30))
    DIM mx AS INTEGER
    DIM my AS INTEGER
    mx = MOUSEX()
    my = MOUSEY()
    CIRCLE(mx, my, 8, RGB(255, 200, 80))
    IF MOUSEBUTTON(0) THEN
        TEXT(10, 220, "Klick!", RGB(255, 80, 80))
    END IF
    FLIP()
    SLEEP(16)
WEND
```

## Gamepad

Bis zu vier Geräte, durchnummeriert ab 0. Wie bei der Tastatur gibt es „wird
gehalten" und „genau jetzt gedrückt".

| Funktion | Zweck |
|---|---|
| `JOYSTICK_COUNT()` → INTEGER | wie viele Geräte sind angeschlossen? |
| `JOYSTICK_NAME(idx)` → STRING | Name des Geräts, wie es sich meldet |
| `JOYSTICK_BUTTON(idx, btn)` → BOOLEAN | wird der Knopf **gehalten**? |
| `JOYSTICK_HIT(idx, btn)` → BOOLEAN | genau in diesem Bild gedrückt |
| `JOYSTICK_RELEASED(idx, btn)` → BOOLEAN | genau in diesem Bild losgelassen |
| `JOYSTICK_ANY_BUTTON()` → INTEGER | zuletzt gedrückter Knopf, `-1` = keiner — für Belegungsdialoge |
| `JOYSTICK_AXIS(idx, achse)` → FLOAT | Stellung einer Analogachse, `-1.0` bis `+1.0` |
| `JOYSTICK_AXIS_COUNT(idx)` → INTEGER | wie viele Achsen hat das Gerät? |
| `JOYSTICK_HAT_X(idx, hat)` / `JOYSTICK_HAT_Y(idx, hat)` → INTEGER | Steuerkreuz als `-1`, `0` oder `+1` je Achse |
| `JOYSTICK_RUMBLE(idx, links, rechts, dauer_s)` | Vibration; die beiden Motoren getrennt (je `0.0`–`1.0`) |
| `JOYSTICK_MAPPINGS(sdl_db$)` → INTEGER | Belegungen aus der SDL-GameControllerDB nachladen — damit werden exotische Geräte richtig zugeordnet |

Für die Knöpfe gibt es Konstanten (`JOY_BUTTON_A` … `JOY_BUTTON_Y`, `JOY_DPAD_*`).
Wer lieber mit Aktionsnamen statt Nummern arbeitet, nimmt das Modul
[`input`](module-input.md) — dort heißt es dann `INPUT_BIND("springen", KEY_SPACE,
JOY_BUTTON_A)`.

Analogachsen liefern selten exakt 0. Ein kleiner Totbereich verhindert, dass die
Figur von allein wandert:

```basic
DIM ax AS FLOAT
ax = JOYSTICK_AXIS(0, 0)
IF ABS(ax) < 0.15 THEN ax = 0.0
```

## Touch und Gesten

| Funktion | Zweck |
|---|---|
| `TOUCH_COUNT()` → INTEGER | wie viele Finger liegen auf? |
| `TOUCH_X(i)` / `TOUCH_Y(i)` → FLOAT | Position des `i`-ten Fingers |
| `TOUCH_ID(i)` → INTEGER | Kennung dieses Fingers — bleibt über Bilder hinweg dieselbe, solange er liegen bleibt |
| `GESTURE$()` → STRING | erkannte Geste: `tap`, `doubletap`, `hold`, `drag`, `swipe_left`, `swipe_right`, `swipe_up`, `swipe_down`, `pinch_in`, `pinch_out` — `""` wenn keine |
| `GESTURE_DRAG_X()` / `GESTURE_DRAG_Y()` → FLOAT | Richtung eines Zugs |
| `GESTURE_DRAG_ANGLE()` → FLOAT | sein Winkel |
| `GESTURE_PINCH_X()` / `GESTURE_PINCH_Y()` → FLOAT | Abstand beim Auf- und Zuziehen |
| `GESTURE_PINCH_ANGLE()` → FLOAT | dessen Winkel |
| `GESTURE_HOLD_TIME()` → FLOAT | wie lange schon gehalten wird (Sekunden) |

`TOUCH_ID` ist der Unterschied zwischen „zwei Finger" und „welcher Finger": Beim
Ziehen mit mehreren wandert der Index `i`, sobald einer abhebt — die Kennung
bleibt. Demo: [examples/149_input_edges.dh](../examples/149_input_edges.dh).
