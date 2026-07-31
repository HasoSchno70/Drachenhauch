# Modul `imgfx`

Bild-Effekte: skalieren, rotieren, spiegeln, einfärben. Jede Funktion gibt ein **neues** IMAGE zurück, das Original bleibt erhalten — Verkettung ist sicher.

```basic
IMPORT "imgfx"
```

## Übersicht

| Funktion | Rückgabe |
|---|---|
| `IMAGE_SCALE(img, w, h)` | IMAGE (neu, in `w × h`) — **bilinear geglättet** |
| `IMAGE_SCALE_NN(img, w, h)` | IMAGE (neu, in `w × h`) — **ohne Interpolation** (Nearest-Neighbour), für Pixelgrafik |
| `IMAGE_ROTATE(img, grad)` | IMAGE (neu, Bounding-Box wächst ggf.) |
| `IMAGE_FLIP(img, flipX, flipY)` | IMAGE (neu, gespiegelt) |
| `IMAGE_TINT(img, color)` | IMAGE (neu, RGB-multipliziert) |
| `IMAGE_COPY(img)` | IMAGE (tiefer Klon) |

### Weitere Filter (geben ebenfalls ein neues IMAGE zurück)

| Funktion | Rückgabe / Wirkung |
|---|---|
| `IMAGE_CROP(img, x, y, w, h)` | IMAGE (neu, Ausschnitt `w × h` ab `x,y`) |
| `IMAGE_RESIZE_CANVAS(img, w, h, offx, offy[, fill])` | IMAGE (neu): Bild auf Leinwand `w × h` ab Offset `offx,offy`; freie Fläche mit `fill` (Default schwarz) |
| `IMAGE_BLUR(img, radius)` | IMAGE (neu, Gauß-Weichzeichner, `radius` in px) |
| `IMAGE_BRIGHTNESS(img, n)` | IMAGE (neu): Helligkeit, `n` = `-255..255` |
| `IMAGE_CONTRAST(img, n)` | IMAGE (neu): Kontrast, `n` = `-100..100` |
| `IMAGE_GRAYSCALE(img)` | IMAGE (neu, in Graustufen) |
| `IMAGE_INVERT(img)` | IMAGE (neu, Farben invertiert) |
| `IMAGE_REPLACE_COLOR(img, from, to)` | IMAGE (neu): tauscht **exakt** die Farbe `from` gegen `to` |

### In ein Image zeichnen (MUTIEREND — verändert das übergebene IMAGE)

Anders als die Filter geben diese **kein** neues Handle zurück, sondern malen direkt in das Image. Ideal, um zur Ladezeit eine Grafik zusammenzubauen (z. B. leere Leinwand via `GENTEX_COLOR(w, h, farbe)`, dann bemalen). Nach jeder Operation wird die GPU-Textur neu hochgeladen — daher **einmal beim Aufbauen** nutzen, nicht in jedem Frame.

| Funktion | Wirkung |
|---|---|
| `IMAGE_DRAW_LINE(img, x1, y1, x2, y2, color)` | Linie ins Image |
| `IMAGE_DRAW_CIRCLE(img, cx, cy, r, color)` | gefüllter Kreis ins Image |
| `IMAGE_DRAW_RECT(img, x, y, w, h, color)` | gefülltes Rechteck ins Image |
| `IMAGE_DRAW_TEXT(img, x, y, text$, size, color)` | Text (Standard-Font) ins Image |

Komplette Fenster-Demo mit allen neuen Ops: [examples/122_imgfx.gb](../examples/122_imgfx.gb).

Alle Funktionen brauchen die native Grafik-Runtime (`gbrt` mit dem `graphics`-Feature). Die Bild-Pipeline wird durch ein vorangegangenes `LOADIMAGE` oder `SCREEN` initialisiert.

## Beispiel

```basic
IMPORT "imgfx"

DIM hero AS IMAGE
hero = LOADIMAGE("assets/hero.png")          ' z.B. 16x16

DIM klein AS IMAGE
klein = IMAGE_SCALE(hero, 8, 8)              ' halbe Groesse

DIM gross AS IMAGE
gross = IMAGE_SCALE(hero, 32, 32)            ' doppelte Groesse (weich)

DIM pixelig AS IMAGE
pixelig = IMAGE_SCALE_NN(hero, 32, 32)       ' doppelte Groesse, Pixel bleiben Pixel

DIM rot_held AS IMAGE
rot_held = IMAGE_TINT(hero, RGB(255, 80, 80))   ' rot getoent

DIM mirror AS IMAGE
mirror = IMAGE_FLIP(hero, TRUE, FALSE)       ' horizontal gespiegelt

DIM gedreht AS IMAGE
gedreht = IMAGE_ROTATE(hero, 45.0)           ' 45 Grad
```

## Wichtige Eigenschaften

**Immutable:** Das Original wird nie verändert. `IMAGE_TINT(hero, ...)` gibt ein neues Bild zurück, `hero` bleibt unverändert.

**Verkettbar:** Effekte lassen sich kombinieren:

```basic
DIM kombi AS IMAGE
kombi = IMAGE_ROTATE(IMAGE_TINT(IMAGE_SCALE(hero, 32, 32), RGB(255, 200, 0)), 30.0)
```

(Hero zuerst auf 32×32 skaliert, dann gelb getönt, dann um 30° gedreht.)

**Bounding-Box bei ROTATE wächst:** Wenn man ein 16×16-Quadrat um 45° dreht, wird das resultierende Bild größer (die Ecken stoßen heraus). Bei 90° bleibt's quadratisch — das Bild wird in eine "neue" Bounding-Box passend gerendert.

```basic
DIM r0 AS IMAGE
r0 = IMAGE_ROTATE(hero, 0.0)                 ' immer noch 16x16
DIM r45 AS IMAGE
r45 = IMAGE_ROTATE(hero, 45.0)               ' jetzt 22x22
DIM r90 AS IMAGE
r90 = IMAGE_ROTATE(hero, 90.0)               ' wieder 16x16
```

**TINT als RGB-Multiplikation:** `IMAGE_TINT(img, color)` multipliziert jeden Pixel mit `color`/255 pro Kanal:

| Tint-Farbe | Effekt |
|---|---|
| `&HFFFFFF` (Weiß) | unverändert |
| `&H000000` (Schwarz) | komplett schwarz |
| `&HFF0000` (Rot) | nur Rot-Kanal überlebt |
| `&HFF8080` | rosa-getönt (Hell-Rot mit etwas Grün/Blau) |
| `&H808080` | halb so hell |

**SCALE:** verlangt `w > 0` und `h > 0`. Negative oder Null wirft.

**FLIP:** `flipX = TRUE` spiegelt horizontal (links-rechts), `flipY = TRUE` vertikal (oben-unten).

## Anwendungs-Beispiele

**Pickup-Aufblitzen** (Coin wird kurz weiß bevor sie verschwindet):

```basic
IMPORT "imgfx"

DIM coin AS IMAGE
coin = LOADIMAGE("assets/coin.png")
DIM coin_flash AS IMAGE
coin_flash = IMAGE_TINT(coin, RGB(255, 255, 255))

' Im Game-Loop: solange Pickup-Animation laeuft, das aufgehellte Bild zeigen
DRAWIMAGE(coin_flash, coin_x, coin_y)
```

**Vorbereiten von Spritesheet-Variants** (statt jedes Frame zur Laufzeit zu transformieren):

```basic
IMPORT "imgfx"

DIM hero_normal AS IMAGE
hero_normal = LOADIMAGE("assets/hero.png")
DIM hero_red AS IMAGE
hero_red = IMAGE_TINT(hero_normal, RGB(255, 80, 80))     ' getroffen
DIM hero_left AS IMAGE
hero_left = IMAGE_FLIP(hero_normal, TRUE, FALSE)         ' nach links laufen
```

So wirst du ohne Performance-Probleme bei jedem Frame nur noch das richtige Bild blitten, keine Re-Transformation.

**Pixel-Art-Skalierung:**

```basic
IMPORT "imgfx"

' 16x16-Sprite auf 64x64 fuer Detail-Anzeige im Inventar
DIM mini AS IMAGE
mini = LOADIMAGE("assets/sword.png")
DIM big AS IMAGE
big = IMAGE_SCALE(mini, 64, 64)
```

## Performance

`IMAGE_SCALE`, `IMAGE_ROTATE`, `IMAGE_FLIP`, `IMAGE_TINT` allokieren jedes Mal ein neues Bild. Mache sie **einmal beim Laden**, nicht in jedem Frame. Wenn du dynamische Effekte willst (z.B. variabler Tint), nutze stattdessen das [Sprite-Modul](module-sprite.md) mit `SPRITE_TINT` — das kümmert sich pro Frame intelligent darum.

## Komplettes Beispiel

Siehe [examples/27_imgfx.gb](../examples/27_imgfx.gb) — Konsolen-Demo mit allen Effekten und Verkettung.
