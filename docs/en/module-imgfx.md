# Module `imgfx`

Image effects: scale, rotate, mirror, tint. The filters return a **new** IMAGE, the original is kept — chaining is safe. Alongside them there are commands that paint **into** an image (`IMAGE_DRAW_*`), and since 2026-08-31 commands with which a program **produces** images instead of merely displaying them: [create, compose and save](#creating-composing-and-saving-images).

```basic
IMPORT "imgfx"
```

## Overview

| Function | Returns |
|---|---|
| `IMAGE_SCALE(img, w, h)` | IMAGE (new, in `w × h`) — **bilinearly smoothed** |
| `IMAGE_SCALE_NN(img, w, h)` | IMAGE (new, in `w × h`) — **without interpolation** (nearest neighbour), for pixel art |
| `IMAGE_ROTATE(img, grad)` | IMAGE (new, bounding box grows if needed) — **resamples**, see below |
| `IMAGE_ROTATE_CW(img)` | IMAGE (new, rotated 90° clockwise) — **exact**, `w × h` → `h × w` |
| `IMAGE_ROTATE_CCW(img)` | IMAGE (new, rotated 90° counter-clockwise) — **exact** |
| `IMAGE_FLIP(img, flipX, flipY)` | IMAGE (new, mirrored) |
| `IMAGE_TINT(img, color)` | IMAGE (new, RGB-multiplied) |
| `IMAGE_COPY(img)` | IMAGE (deep clone) |

### More filters (also return a new IMAGE)

| Function | Returns / effect |
|---|---|
| `IMAGE_CROP(img, x, y, w, h)` | IMAGE (new, section `w × h` starting at `x,y`) |
| `IMAGE_RESIZE_CANVAS(img, w, h, offx, offy[, fill])` | IMAGE (new): image on a canvas `w × h` at offset `offx,offy`; free area filled with `fill` (default black) |
| `IMAGE_BLUR(img, radius)` | IMAGE (new, Gaussian blur, `radius` in px) |
| `IMAGE_BRIGHTNESS(img, n)` | IMAGE (new): brightness, `n` = `-255..255` |
| `IMAGE_CONTRAST(img, n)` | IMAGE (new): contrast, `n` = `-100..100` |
| `IMAGE_GRAYSCALE(img)` | IMAGE (new, in greyscale) |
| `IMAGE_INVERT(img)` | IMAGE (new, colours inverted) |
| `IMAGE_REPLACE_COLOR(img, from, to)` | IMAGE (new): replaces **exactly** the colour `from` with `to` |
| `IMAGE_COLOR_TO_ALPHA(img, farbe [, schwelle])` | IMAGE (new): makes a background colour transparent like “Color to Alpha” in GIMP -- mixtures become semi-transparent and are unmixed, a glow stays soft; `schwelle` (threshold, 0..1) removes faint remnants completely. For cutting out logos and sprites in front of a plain background |

### Drawing into an image (MUTATING — changes the IMAGE passed in)

Unlike the filters these return **no** new handle, they paint directly into the image. Ideal for assembling a graphic at load time (e.g. an empty canvas via `IMAGE_NEW(w, h)`, then paint on it).

**Each of these calls re-uploads the whole texture** — measured at **1.16 µs
per call**. What that means depends on how many there are:

| | |
|---|---|
| one brush stroke, a few points per frame | irrelevant |
| an area of 64×64 point by point | 4.7 ms — just about fits into one frame |
| the same at 256×256 | around 75 ms — a visible stutter |

For a paint program that is the normal case and fine; anyone setting *whole
areas* is better off with `IMAGE_CLEAR`, `IMAGE_DRAW_RECT` at its real
size or `IMAGE_DRAW_IMAGE` instead of a loop over single points.

**Within ONE frame you only see the final state.** Whoever draws the same IMAGE
twice in the same frame and changes it in between gets **the new content both
times** — a before/after side by side does not work that way:

```basic
IMPORT "imgfx"
DIM b AS IMAGE : b = IMAGE_NEW(16, 16, &HFF0000)
DRAWIMAGE(b, 10, 10)                      ' shows GREEN, not red
IMAGE_DRAW_RECT(b, 0, 0, 16, 16, &H00FF00)
DRAWIMAGE(b, 40, 10)                      ' shows green
```

The reason is the drawing model: a drawing command only remembers *which*
texture it uses, and it is only looked up at `FLIP`. Anyone who needs two
states side by side needs two images (`IMAGE_COPY`).



| Function | Effect |
|---|---|
| `IMAGE_DRAW_LINE(img, x1, y1, x2, y2, color)` | line into the image |
| `IMAGE_DRAW_CIRCLE(img, cx, cy, r, color)` | filled circle into the image |
| `IMAGE_DRAW_RECT(img, x, y, w, h, color)` | filled rectangle into the image |
| `IMAGE_DRAW_TEXT(img, x, y, text$, size, color[, font])` | text into the image; without `font` the built-in font as before, with `font` one loaded via `LOADFONT` or `LOADFONT_IMAGE` (blended correctly over a semi-transparent background like `IMAGE_DRAW_IMAGE`; an SDF font does not work) |
| `IMAGE_TEXT(text$, groesse, farbe[, font])` | IMAGE — a new image exactly as large as the text (as wide as `TEXT_WIDTH` measures it), on a transparent background — for signs, labels on textures, billboards with text |
| `IMAGE_ALPHA_MASK(bild, maske)` | IMAGE — take the opacity from a second image (soft edges) |
| `IMAGE_ALPHA_CROP(bild, schwelle)` | IMAGE — cut away a transparent border; `schwelle` says from which opacity on a pixel counts |
| `IMAGE_ALPHA_PREMULTIPLY(bild)` | IMAGE — premultiply the colour by the opacity; prevents dark fringes when scaling |
| `IMAGE_DITHER(bild, r, g, b, a)` | IMAGE — reduce the colour depth and distribute the error. **Only 5,6,5,0 / 5,5,5,1 / 4,4,4,4** — anything else is rejected, because raylib would otherwise deliver an unusable format |
| `IMAGE_PALETTE(bild, max)` | ARRAY OF INTEGER — the most frequent colours of the image |
| `IMAGE_CHANNEL(bild, kanal)` | IMAGE — one channel (`0` red, `1` green, `2` blue, `3` opacity) as a greyscale image, for example to see and edit the opacity as a mask |

### Shapes with floating-point numbers and opacity

The commands above only know whole numbers, filled circles and lines one
point wide -- and each one re-uploads the texture. Anyone who **produces** a
graphic from many shapes (tiles, icons, buttons) needs more: a filled
polygon, a thick line, rounded corners, a ring, a gradient. That is what
these eight are for (built for the tile generator of Circuit Runner,
`circuitrunner/make_tiles.dh`):

| Function | Effect |
|---|---|
| `IMAGE_FILL_RECT(bild, x0, y0, x1, y1, farbe [, deckkraft])` | rectangle from corner to corner, both bounds included |
| `IMAGE_FRAME(bild, x0, y0, x1, y1, breite, farbe [, deckkraft])` | frame of width `breite`, drawn inwards |
| `IMAGE_FILL_ROUNDRECT(bild, x0, y0, x1, y1, radius, farbe [, deckkraft])` | rectangle with rounded corners |
| `IMAGE_FILL_ELLIPSE(bild, cx, cy, rx, ry, farbe [, deckkraft])` | filled ellipse around `cx, cy`; with `rx = ry` a circle |
| `IMAGE_RING(bild, cx, cy, r, breite, farbe [, deckkraft])` | circular ring from radius `r` inwards |
| `IMAGE_FILL_POLY(bild, xs, ys, farbe [, deckkraft])` | filled polygon from two arrays with the corner points, concave too |
| `IMAGE_POLYLINE(bild, xs, ys, breite, farbe [, deckkraft])` | polyline of width `breite` with round joints |
| `IMAGE_GRADIENT(bild, x0, y0, x1, y1, farbe1, farbe2 [, senkrecht [, deckkraft1, deckkraft2]])` | gradient in the rectangle, top to bottom by default; blends colour and opacity |

**Writing OVERWRITES, it does not blend.** A point gets exactly the colour
and opacity the command names -- with `deckkraft` 0 you punch a hole into an
area. Without `deckkraft` the colour applies as everywhere (`&Hrrggbb` is
opaque, `RGBA` with opacity 1..255). The opacity is a separate argument
because "completely transparent" cannot be written as a colour: the top byte
0 means opaque. Anyone who wants to blend draws into a separate image and
lays it on top with `IMAGE_DRAW_IMAGE`.

**Coordinates are floating-point numbers, a point sits on its integer
position**, bounds are inclusive. There is no anti-aliasing -- anyone who
wants soft edges draws at double or quadruple size and scales down with
`IMAGE_SCALE`. Whatever extends beyond the image border is dropped.

**The upload only happens at the next `FLIP`**, not per call: a thousand
shapes in one image therefore cost one upload instead of a thousand. Reading
(`GETPIXEL`, `IMAGE_SAVE`, `IMAGE_DRAW_IMAGE`, `IMAGE_SCALE` ...) uses the
image itself and not the texture anyway -- it sees the new state immediately.

## Creating, composing and saving images

These five turn an IMAGE into something a program can not only display but
also **produce**. Before, it was a one-way street: drawing into it worked,
but a transparent image could not be created, no image could be placed into
another, nothing could be erased, the opacity could not be read and nothing
at all could be saved.

| Function | Effect |
|---|---|
| `IMAGE_NEW(breite, hoehe [, farbe])` | IMAGE — a new image; **completely transparent without a colour** |
| `IMAGE_CLEAR(bild [, x, y, b, h])` | make an area completely transparent — the eraser; without a rectangle the whole image |
| `IMAGE_DRAW_IMAGE(ziel, quelle, x, y [, qx, qy, qb, qh] [, faerbung])` | draw one image into another, blended with opacity; optionally only a section of the source |
| `IMAGE_SAVE(bild, pfad$ [, mit_alpha])` | write the image to a file (`.png`, `.bmp`, `.jpg`, `.tga`); with `FALSE` without an opacity channel (RGB) — print services reject a PNG with an alpha channel, even if every point is opaque. The opacity is simply left out, not blended in: anyone who wants a background lays the image onto an opaque one first |
| `GETALPHA(bild, x, y)` | opacity of an image point, 0..255; `-1` outside (core, no IMPORT needed) |
| `IMAGE_FREE(bild)` | release the image and its graphics-memory texture |
| `IMAGE_FROM_BUFFER(puffer)` | IMAGE — an image from the bytes of an image file (PNG, JPG, BMP, GIF, QOI, DDS), for example from `HTTP_BYTES` or `BUFFER_FROM_BASE64`; the format is in the first bytes (core, no IMPORT needed) |
| `IMAGE_TO_BUFFER(bild [, mit_alpha])` | BUFFER — the image as PNG bytes, without a detour through a file; with `FALSE` without an opacity channel as with `IMAGE_SAVE` (core, no IMPORT needed) |
| `IMAGE_MIPMAPS(bild)` | INTEGER — create downscaled levels for the texture and filter trilinearly: a fine pattern on a distant 3D surface then no longer flickers. Returns the number of levels; after `IMAGE_DRAW_*` they are updated |
| `IMAGE_SAVE_GIF(bilder, pfad$ [, fps_oder_dauern [, wiederholen [, anzahl]]])` | write several images as an **animated GIF** |
| `IMAGE_LOAD_GIF(pfad$)` | ARRAY OF IMAGE — all frames of an animated GIF, already composed (even if a frame only renews a part); a file that is not a GIF yields a single image |
| `IMAGE_GIF_DELAYS(pfad$)` | ARRAY OF INTEGER — the duration of each frame in milliseconds, in the same order; the counterpart of the array that `IMAGE_SAVE_GIF` takes (core, no window needed) |

```basic
IMPORT "imgfx"
' Combine two layers into one image and save it.
DIM unten AS IMAGE : unten = IMAGE_NEW(64, 64, &H204060)
DIM oben AS IMAGE  : oben  = IMAGE_NEW(64, 64)          ' transparent
IMAGE_DRAW_CIRCLE(oben, 32, 32, 20, &HFFCC00)
IMAGE_CLEAR(oben, 28, 28, 8, 8)                          ' erase a hole

DIM fertig AS IMAGE : fertig = IMAGE_NEW(64, 64)
IMAGE_DRAW_IMAGE(fertig, unten, 0, 0)
IMAGE_DRAW_IMAGE(fertig, oben, 0, 0)
IMAGE_SAVE(fertig, "ebenen.png")
```

**Blending is correct even when the target is semi-transparent**
(Porter-Duff "over" rule): orange with opacity 60 over orange with opacity 59
stays orange and afterwards has opacity 105. Until 2026-09-14
`IMAGE_DRAW_IMAGE` went through raylib's `ImageDraw`, and that turned it into
a green (red 0x01) -- visible as a greenish glow around every softly drawn
shape. Over an opaque background the result is the same as before.

**Why `IMAGE_NEW` and not `GENTEX_COLOR`:** a completely transparent image
cannot be expressed through a **colour** at all. The colour convention reads
opacity 0 as *opaque* — only that way do `&Hrrggbb` and `RGB(r,g,b)` stay
opaque. `IMAGE_NEW` without a colour gets around that.

**Why `GETALPHA` and not a fourth byte in `GETPIXEL`:** for the same reason.
`GETPIXEL` returns a *colour* — a transparent point would arrive there as
opaque black. Anyone who wants to know whether a point is empty asks
`GETALPHA(bild, x, y) = 0`.

**`IMAGE_CLEAR` does not blend, it writes.** Blending in a transparent
rectangle would do nothing; erasing has to replace the point.

**`IMAGE_SAVE` and the extension.** It determines the format, and an unknown
one is rejected instead of silently doing nothing (raylib would then only
write a line into the log, and the program would believe it had saved).
A failed write — missing directory, no write permission — reports itself as
well; it is checked on the file itself, because the raylib binding discards
the success flag.

**`IMAGE_FREE` and what applies afterwards.** An image occupies main memory
*and* a texture in graphics memory; until 2026-08-31 both stayed until the
program ended. For a game that loads its images once that does not matter —
for anything that keeps creating them (an editor with undo steps, a preview,
an image viewer) it was a leak that grew with every step. Measured with 1200
copies of 256×256: **393 MB versus 91 MB.**

After releasing, the handle is **not reusable and is not handed out again
either**. Every further use reports itself in plain words
(`… wurde mit IMAGE_FREE freigegeben`) instead of silently pointing at some
other image — that is why the slot stays reserved. `GETPIXEL` and
`GETALPHA` keep their old promise and return `-1`.

`LOADIMAGE` remembers path → handle; when releasing, the entry is deleted
too, so a later `LOADIMAGE` of the same path loads afresh.

**What it does not know about:** a texture that went to a model via
`MODEL_TEXTURE` lives on there as a bare number (raylib's `Texture2D` is a
struct without reference counting) — that model then points into the void.
Images in a widget or a sprite atlas at least report themselves, because
their drawing path checks the handle. **Not creating is cheaper than creating
and releasing:** anyone who needs snapshots in a loop is better off creating
the slots once and overwriting them with `IMAGE_CLEAR` + `IMAGE_DRAW_IMAGE`.

**Animated GIFs.** `IMAGE_SAVE_GIF` takes an `ARRAY OF IMAGE` (or a
TUPLE) and writes an animation from it. `wiederholen` is the endless loop
(default TRUE), `anzahl` says **how many slots of the array count** — a
`DIM b[16] AS IMAGE` with three filled slots is the normal case, and the
empty ones would otherwise be an error.

**The third parameter means two things:**

| Form | Meaning |
|---|---|
| a number | **frames per second** for all (default 10) |
| an ARRAY / TUPLE | **duration per frame in milliseconds** |

```basic
IMPORT "imgfx"
DIM b[3] AS IMAGE
DIM i AS INTEGER
FOR i = 0 TO 2
    b[i] = IMAGE_NEW(16, 16)
    IMAGE_DRAW_CIRCLE(b[i], 4 + i * 4, 8, 3, &HE84B4B)
NEXT
IMAGE_SAVE_GIF(b, "lauf.gif", 8)                  ' 8 frames per second
IMAGE_SAVE_GIF(b, "pose.gif", [1000, 80, 80])     ' first pose is held
```

That the **unit changes** is intentional: a single frame has no frame rate,
it has a duration. Reading `[4, 12]` as “250 ms, then 83 ms” would be the
worse imposition. If there are fewer times than frames, that is an error —
silently repeating the last one would be a guess, and a wrong time cannot be
seen in the GIF, you only notice it.

This is exactly what it is for: a frame sequence holds a pose and lets the
movement in between run through quickly. The sprite editor
(`examples/189_sprite_editor.dh`) writes out its per-frame durations this way.

**What GIF cannot do, and what follows from it:**

- **At most 256 colours per frame.** Pixel art almost always stays below that —
  which is why the colour table is built *exactly* from the colours present as
  long as there are at most 255. Only above that are colours merged, and then
  colours shift. A method that always merges would already have falsified a
  four-colour sprite.
- **Transparency only all or nothing.** A point is transparent or opaque,
  nothing in between; the decision is made at opacity 128. The format knows
  nothing else.
- **One canvas for all frames.** Frames of different sizes are rejected
  instead of cropped — cropping would be a silent loss.
- GIF counts the duration per frame in hundredths of a second. Below 2 most
  viewers silently set their own (usually 10), which is why it is clamped
  there: otherwise the output would run **slower** than requested, without
  any hint.

**Not included: images via the clipboard.** `CLIPBOARD_GET`/`SET` can only
handle text. raylib's `GetClipboardImage` only exists on Windows, and a
command that does not exist on two of three operating systems is a trap.

Complete window demo with all the new operations: [examples/122_imgfx.dh](../../examples/122_imgfx.dh).

All functions need the native graphics runtime (`dhrt` with the `graphics` feature). The image pipeline is initialised by a preceding `LOADIMAGE` or `SCREEN`.

## Example

```basic
IMPORT "imgfx"

DIM hero AS IMAGE
hero = LOADIMAGE("assets/hero.png")          ' e.g. 16x16

DIM klein AS IMAGE
klein = IMAGE_SCALE(hero, 8, 8)              ' half size

DIM gross AS IMAGE
gross = IMAGE_SCALE(hero, 32, 32)            ' double size (smooth)

DIM pixelig AS IMAGE
pixelig = IMAGE_SCALE_NN(hero, 32, 32)       ' double size, pixels stay pixels

DIM rot_held AS IMAGE
rot_held = IMAGE_TINT(hero, RGB(255, 80, 80))   ' tinted red

DIM mirror AS IMAGE
mirror = IMAGE_FLIP(hero, TRUE, FALSE)       ' mirrored horizontally

DIM gedreht AS IMAGE
gedreht = IMAGE_ROTATE(hero, 45.0)           ' 45 degrees
```

## Important properties

**Immutable:** The original is never changed. `IMAGE_TINT(hero, ...)` returns a new image, `hero` stays unchanged.

**Chainable:** Effects can be combined:

```basic
DIM kombi AS IMAGE
kombi = IMAGE_ROTATE(IMAGE_TINT(IMAGE_SCALE(hero, 32, 32), RGB(255, 200, 0)), 30.0)
```

(Hero first scaled to 32×32, then tinted yellow, then rotated by 30°.)

**The bounding box grows with ROTATE:** If you rotate a 16×16 square by 45°, the resulting image gets larger (the corners stick out). At 90° it stays square — the image is rendered to fit a "new" bounding box.

```basic
DIM r0 AS IMAGE
r0 = IMAGE_ROTATE(hero, 0.0)                 ' still 16x16
DIM r45 AS IMAGE
r45 = IMAGE_ROTATE(hero, 45.0)               ' now 22x22
DIM r90 AS IMAGE
r90 = IMAGE_ROTATE(hero, 90.0)               ' 16x16 again
```

> **`IMAGE_ROTATE` is useless for pixel art — even at 90°.** It computes
> trigonometrically and resamples in the process. Measured on a 16×16 image
> with four differently coloured corner points: after
> `IMAGE_ROTATE(b, 90.0)` **all four have vanished**, and even the
> single-coloured area comes back washed out (`0x141414` → `0x131413`).
>
> For quarter turns therefore use `IMAGE_ROTATE_CW` / `IMAGE_ROTATE_CCW`: they
> only rearrange the points, so they lose nothing and invent nothing.
> Width and height swap in the process — 16×32 becomes 32×16. The same
> distinction as `IMAGE_SCALE` versus `IMAGE_SCALE_NN`; it is only more
> noticeable here because a quarter turn is actually lossless.

```basic
' Exact -- the only usable rotation for sprites:
DIM rechts AS IMAGE : rechts = IMAGE_ROTATE_CW(hero)     ' 16x32 -> 32x16
DIM links AS IMAGE  : links  = IMAGE_ROTATE_CCW(hero)
' Twice CW is a half turn, three times is CCW.
```

**TINT as RGB multiplication:** `IMAGE_TINT(img, color)` multiplies each pixel by `color`/255 per channel:

| Tint colour | Effect |
|---|---|
| `&HFFFFFF` (white) | unchanged |
| `&H000000` (black) | completely black |
| `&HFF0000` (red) | only the red channel survives |
| `&HFF8080` | pink-tinted (light red with some green/blue) |
| `&H808080` | half as bright |

**SCALE:** requires `w > 0` and `h > 0`. Negative values or zero throw.

**FLIP:** `flipX = TRUE` mirrors horizontally (left-right), `flipY = TRUE` vertically (top-bottom).

## Usage examples

**Pickup flash** (a coin briefly turns white before it disappears):

```basic
IMPORT "imgfx"

DIM coin AS IMAGE
coin = LOADIMAGE("assets/coin.png")
DIM coin_flash AS IMAGE
coin_flash = IMAGE_TINT(coin, RGB(255, 255, 255))

' In the game loop: while the pickup animation runs, show the brightened image
DRAWIMAGE(coin_flash, coin_x, coin_y)
```

**Preparing sprite-sheet variants** (instead of transforming every frame at runtime):

```basic
IMPORT "imgfx"

DIM hero_normal AS IMAGE
hero_normal = LOADIMAGE("assets/hero.png")
DIM hero_red AS IMAGE
hero_red = IMAGE_TINT(hero_normal, RGB(255, 80, 80))     ' hit
DIM hero_left AS IMAGE
hero_left = IMAGE_FLIP(hero_normal, TRUE, FALSE)         ' walking left
```

That way, without performance problems, every frame you only blit the right image, no re-transformation.

**Pixel-art scaling:**

```basic
IMPORT "imgfx"

' 16x16 sprite to 64x64 for a detail view in the inventory
DIM mini AS IMAGE
mini = LOADIMAGE("assets/sword.png")
DIM big AS IMAGE
big = IMAGE_SCALE(mini, 64, 64)
```

## Performance

`IMAGE_SCALE`, `IMAGE_ROTATE`, `IMAGE_FLIP`, `IMAGE_TINT` allocate a new image every time. Do them **once at load time**, not in every frame. If you want dynamic effects (e.g. a variable tint), use the [sprite module](module-sprite.md) with `SPRITE_TINT` instead — it takes care of that intelligently per frame.

## Complete example

See [examples/27_imgfx.dh](../../examples/27_imgfx.dh) — console demo with all effects and chaining.
