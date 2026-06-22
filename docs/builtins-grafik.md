# Grafik-Built-ins

Grafik, Sound und Eingabe — nativ in der Runtime `gbrt` (raylib). Alle Befehle hier brauchen ein offenes Fenster — also muss vor allem anderen `SCREEN(...)` aufgerufen werden.

Wenn das `camera`-Modul aktiv ist und `CAMERA_SET` aufgerufen wurde, interpretieren alle Drawing-Befehle ihre Koordinaten als **World-Koordinaten** (siehe [Camera-Modul](module-camera.md)).

## Inhalt

- [Fenster und Frame](#fenster-und-frame)
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
| `QUITREQUESTED()` → BOOLEAN | TRUE wenn der User auf das Fenster-X geklickt hat |

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
| `ARC(x1, y1, x2, y2, start_rad, end_rad[, color[, width]])` | Bogen-Segment in der Bounding-Box; Winkel in Radiant, gegen den Uhrzeigersinn |
| `TEXT(x, y, s$[, color])` | Text bei (x, y) |
| `TEXTROT(x, y, s$, winkel[, skala[, farbe]])` | Text **zentriert** auf (x, y), um das Zentrum gedreht (Grad, wie `DRAWIMAGEROT`) und skaliert — für Score-Popups, schräge Labels. Nutzt aktiven Font/Größe |

> **Eckpunkt-Reihenfolge egal:** gefülltes `TRIANGLE` und `POLYGON` zeichnen
> unabhängig von der Wicklung — ob die Punkte im oder gegen den Uhrzeigersinn
> angegeben sind, die Fläche erscheint immer (gbrt dreht intern bei Bedarf um).

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

## Schrift

| Funktion | Wirkung |
|---|---|
| `TEXT(x, y, s$[, color])` | Text bei (x, y) in der aktiven Schrift |
| `TEXT_SIZE(px)` | Schriftgröße für folgende `TEXT`-Aufrufe (4–400) |
| `TEXT_WIDTH(s$)` | Pixelbreite von `s$` in der aktiven Schrift/Größe |
| `TEXT_HEIGHT()` | Zeilenhöhe der aktiven Schrift |
| `TEXT_BOLD(an)` / `TEXT_ITALIC(an)` | Fett/Kursiv (nativ No-Op — raylib ohne Fett/Kursiv) |
| `LOADFONT(pfad$, groesse)` → FONT | TTF/OTF laden → FONT-Handle (INTEGER) |
| `SETFONT(font)` | aktive Schrift setzen; `SETFONT(-1)` = Default-Font |
| `TEXT_SPACING(px)` | Buchstabenabstand für TTF (nativ) |

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

Demo: [examples/87_ttf_fonts.gb](../examples/87_ttf_fonts.gb).

## Bilder

| Funktion | Zweck |
|---|---|
| `LOADIMAGE(path$)` → IMAGE | Datei laden (PNG, JPG, BMP, …) |
| `IMAGEWIDTH(img)`, `IMAGEHEIGHT(img)` → INTEGER | Pixelgröße |
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

**Asset-Cache:** `LOADIMAGE` und `LOADSOUND` cachen ihre Ergebnisse automatisch. Mehrfache Aufrufe mit demselben (oder gleichbedeutendem) Pfad liefern dieselbe Surface zurück, ohne Disk-IO. Cache-Schlüssel sind sowohl der rohe Pfad als auch der normalisierte Absolut-Pfad, sodass verschiedene Schreibweisen (`"x.png"`, `"./x.png"`, absolut) denselben Eintrag treffen.

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

Vollständiges Beispiel: [examples/75_preloader.gb](examples/75_preloader.gb).

## Sprite-Atlas + Batch-Draw

Ein **Sprite-Atlas** ist EIN großes Bild mit benannten Sub-Rects (`x, y, w, h`). Statt 50 einzelner PNG-Dateien hat man ein Atlas-PNG + ein Manifest. Atlas-Sprites werden gebatcht — Hunderte Sprites in einem einzigen Draw-Call statt N separater Aufrufe.

| Funktion | Zweck |
|---|---|
| `ATLAS_LOAD(manifest_path$)` → SPRITE_ATLAS | Atlas aus JSON-Manifest laden |
| `ATLAS_DRAW(atlas, name$, x, y)` | einzelnes Sub-Sprite zeichnen (Camera-aware) |
| `ATLAS_DRAW_FLIPPED(atlas, name$, x, y[, flip_x[, flip_y[, tint]]])` | Sub-Sprite mit Spiegelung (X/Y, je `TRUE`/`FALSE` oder `1`/`0`); optional `tint` |
| `BATCH_DRAW(atlas, name$, x, y)` | Sub-Sprite an Batch-Queue anhängen |
| `BATCH_FLUSH()` | Queue jetzt rendern (gebatchter Draw-Call) |

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

' Pro Frame: 600 Tiles in einer Queue sammeln, dann EINMAL rendern.
FOR row = 0 TO 19
    FOR col = 0 TO 29
        BATCH_DRAW(atlas, "tile_grass", col * 16, row * 16)
    NEXT
NEXT
BATCH_FLUSH()   ' ein gebatchter Draw-Call fuer alle 600
```

**Auto-Flush** an wichtigen Punkten — die Queue wird automatisch geleert vor:
- `FLIP()` (sonst geht die Queue verloren)
- `LAYER(...)` (damit der Batch zum richtigen Layer geht)
- `ATLAS_DRAW(...)` (Direct-Call wahrt Reihenfolge)

**Zoom-Caveat:** Bei `CAMERA_SET`-Zoom ≠ 1 fällt jeder `BATCH_DRAW` automatisch auf `DRAWIMAGEPART` zurück (der Batch kann nicht skaliert zeichnen). Translation funktioniert mit Batch. Wer auf Zoom angewiesen ist und viele Sprites batchen will, bakt die Zoom-Stufe in den Atlas oder nutzt `ATLAS_DRAW` einzeln.

**Flipping für Charakter-Sprites:** `ATLAS_DRAW_FLIPPED(atlas, name$, x, y[, flip_x[, flip_y[, tint]]])` spiegelt das Sub-Sprite an X- oder Y-Achse. `flip_x`/`flip_y` akzeptieren `TRUE`/`FALSE` **oder** `1`/`0` (fehlend = `FALSE`); `tint` ist ein optionaler 7. Farb-Parameter. Klassisches Pattern für Walk-Animationen: nur eine Richtung (rechts) im Atlas, links wird per Flip abgeleitet:

```basic
DIM flip AS BOOLEAN
flip = CHAR_FACING(player) = -1     ' -1 = nach links
ATLAS_DRAW_FLIPPED(mario, "walk_a", x, y, flip, FALSE)
```

Flip erzeugt pro Aufruf ein frisch gespiegeltes Bild. Für viele wiederholte Flips desselben Sprites lohnt es sich, die gespiegelte Variante einmal vorberechnet als IMAGE zu cachen — für einen einzelnen Player-Sprite pro Frame ist der Overhead aber vernachlässigbar.

Vollständiges Beispiel: [examples/76_layers_atlas.gb](examples/76_layers_atlas.gb).

## Z-Layer-Rendering

Z-Layer geben dir explizite Render-Reihenfolge: Hintergrund → Sprites → UI, ohne dass du die Draw-Reihenfolge im Code akribisch verwalten musst. Jeder Layer ist eine off-screen Surface mit explizitem z-Wert. `FLIP` composiert alle Layer in z-Order (niedrigstes z = hinten, höchstes z = vorne) auf den Main-Buffer und blittet zum Screen.

| Funktion | Zweck |
|---|---|
| `LAYER_DEFINE(name$, z)` | Layer mit explizitem z registrieren (re-define aktualisiert nur z) |
| `LAYER(name$)` | aktiven Draw-Target auf Layer umschalten (auto-Define wenn neu) |
| `LAYER_END()` | zurück zum Main-Buffer (optional, FLIP macht's auch) |
| `LAYER_CLEAR(name$)` | Layer manuell leeren (selten, FLIP cleart alle Layer nach dem Composite) |

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

    FLIP()    ' composiert in z-Order, cleart Layer fuer naechsten Frame
WEND
```

**Layer-Surfaces** sind SRCALPHA (transparent außer wo gezeichnet wird). Pro Frame werden sie nach dem Composite gecleared — du musst sie nicht selbst leeren. Wenn ein Layer einen opaken Hintergrund haben soll (z.B. der bg-Layer): einfach `CLS(...)` als ersten Draw-Call auf dem Layer.

**Backwards-Compat:** Code ohne `LAYER_*`-Calls läuft unverändert direkt auf den Main-Buffer.

**Camera-Hinweis:** Camera ist global, gilt für alle Layer. Für UI ohne Camera-Effekt (z.B. HUD): vor dem `LAYER("ui")`-Draw `CAMERA_RESET()` rufen (benötigt `IMPORT "camera"`).

**Layer + Atlas kombiniert** (vollständig in [examples/76_layers_atlas.gb](examples/76_layers_atlas.gb)):

```basic
LAYER("bg")
FOR each tile:
    BATCH_DRAW(atlas, "tile_grass", x, y)
NEXT
BATCH_FLUSH()     ' Batch landet auf bg-Layer
LAYER("ui")       ' BATCH wuerde auto-flushen, aber ist hier schon leer
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

## Eingabe: Tastatur und Maus

| Funktion | Zweck |
|---|---|
| `KEYPRESSED(code)` → BOOLEAN | TRUE solange die Taste mit SDL-Code `code` gehalten wird |
| `MOUSEX()`, `MOUSEY()` → INTEGER | aktuelle Mausposition (in logischen Pixeln) |
| `MOUSEBUTTON(n)` → BOOLEAN | TRUE wenn Maustaste n gedrückt — **`0`=links, `1`=rechts, `2`=mitte** (raylib-Reihenfolge: rechts vor mitte!) |
| `MOUSEWHEEL()` → INTEGER | Mausrad-Delta seit dem letzten Aufruf (+ hoch / − runter / 0) |
| `MOUSE_VISIBLE(an)` | OS-Cursor zeigen/verstecken — verstecken, wenn das Spiel ein eigenes Fadenkreuz/Cursor-Sprite zeichnet |
| `MOUSE_LOCK(an)` | Cursor **fangen**: verstecken + im Fenster einsperren (relative Bewegung) — für First-Person-/Kamera-Maussteuerung; `FALSE` gibt frei |
| `MOUSE_HIDDEN()` → BOOLEAN | ist der Cursor gerade versteckt/gefangen? |
| `SCREENWIDTH()`, `SCREENHEIGHT()` → INTEGER | logische Fenstergröße (wie an `SCREEN` übergeben); 0 vor `SCREEN` |

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

Verfügbare Konstanten: `KEY_ESCAPE`, `KEY_RETURN`/`KEY_ENTER`, `KEY_SPACE`, `KEY_TAB`, `KEY_BACKSPACE`, `KEY_LEFT`, `KEY_RIGHT`, `KEY_UP`, `KEY_DOWN`, `KEY_A` bis `KEY_Z`, `KEY_0` bis `KEY_9`.

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
