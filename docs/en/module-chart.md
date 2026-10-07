# Module `chart` — charts

Pie/donut, bar, line/area and analogue gauge displays. Handle-based
like `gui` and `particles`: build and configure once, then draw
every frame.

```basic
IMPORT "chart"

DIM t AS CHART
t = CHART_NEW("tacho", 500, 300, 240, 240)
CHART_SET(t, "titel", "Drehzahl")
CHART_SET_NUM(t, "max", 8000)
CHART_ZONE(t, 6500, 8000, RED)

' per frame:
CHART_VALUE(t, drehzahl)
CHART_DRAW(t)
```

`CHART_DRAW` needs a window (`SCREEN`). Everything else — building, setting
data, querying statistics — also works in a console program.

## Chart types

`CHART_NEW(art$, x, y, breite, hoehe)`. The type understands German and English
names:

| Type | Names | What it shows |
|---|---|---|
| Pie | `kuchen`, `pie`, `donut` | shares of a whole; with `innenradius` (inner radius) > 0 as a ring |
| Bar | `balken`, `bar` | values per category, vertical or horizontal, grouped or stacked |
| Line | `linie`, `line`, `flaeche`, `area` | progressions over time, several series, optionally with an area underneath |
| Gauge | `tacho`, `gauge` | one value on a round scale with a needle and colour zones |
| Bar gauge | `leiste`, `balkenanzeige`, `bar_gauge` | one value on a horizontal or vertical bar, with a marker |
| Lamps | `led`, `lampen`, `zellen` | one value as a chain of discrete cells that light up to that point |

## Data

Two ways — the short one for pie and gauge, the full one for several series.

**Short** (one series, a colour per segment):

```basic
CHART_ADD(c, "Holz",  45.0, BROWN)     ' name, value, colour (colour optional)
CHART_ADD(c, "Stein", 30.0, GRAY)
```

**Full** (several series over shared categories):

```basic
DIM r AS INTEGER
DIM werte[7] AS FLOAT
r = CHART_SERIES(c, "Einnahmen")       ' -> series index
CHART_DATA(c, r, werte)                ' ARRAY OF FLOAT
CHART_LABEL(c, 0, "Mo")                ' label a category
```

> **Careful with the colour:** `0` is **black**, not "take the next
> palette colour". If you want the palette, leave the argument out (or pass
> `-1`). `CHART_SERIES(c, "Name", 0)` gives you a black series — an
> easily made and hard to spot mistake.

| Command | Effect |
|---|---|
| `CHART_SERIES(c, name$ [, farbe])` | create a series → index |
| `CHART_ADD(c, name$, wert [, farbe])` | category + value in series 0 → index |
| `CHART_DATA(c, reihe, werte)` | replace all values of a series (`ARRAY OF FLOAT`) |
| `CHART_SET_POINT(c, reihe, punkt, wert)` | change a single value |
| `CHART_PUSH(c, reihe, wert)` | append a value at the end (live curves, see `fenster`) |
| `CHART_VALUE(c, wert)` | short form for series 0, point 0 — the gauge way |
| `CHART_LABEL(c, punkt, text$)` | label a category |
| `CHART_GET(c, reihe, punkt)` | read a value |
| `CHART_COUNT(c)` / `CHART_SERIES_COUNT(c)` | number of points / series |
| `CHART_CLEAR(c)` | discard the data |
| `CHART_STAT(c, reihe, kennzahl$)` | `anzahl`/`summe`/`mittel`/`min`/`max` (count/sum/mean/min/max) |
| `CHART_BOUNDS(c, x, y, b, h)` | move/resize the area |
| `CHART_ZONE(c, von, bis, farbe [, name$])` / `CHART_ZONE_CLEAR(c)` | colour zones of the gauge scale, optionally labelled |
| `CHART_UPDATE(c, sekunden)` | keep the animation running (see below) |
| `CHART_DRAW(c)` | draw (needs `SCREEN`) |

## Configuring the look

There are about forty knobs — as one built-in each that would be an
unmanageable list. Instead there are **four setters** that take the name of the
property as a string:

```basic
CHART_SET(c,       "titel", "Rohstoffe")   ' text
CHART_SET_NUM(c,   "innenradius", 0.55)    ' number
CHART_SET_COLOR(c, "gitter", DARKGRAY)     ' colour
CHART_SET_FLAG(c,  "prozent", TRUE)        ' on/off
```

The price: a typo only shows up at run time. The message, however, always names
what would have been valid:

```
CHART_SET_NUM: unbekannte Eigenschaft 'innen_radius' (gueltig: min, max, innenradius, …)
```

### Text — `CHART_SET`

| Name | Meaning |
|---|---|
| `titel` | heading above the chart (title) |
| `einheit` | appended to every number (`" km/h"`) (unit) |
| `achse_x`, `achse_y` | axis labels (y is drawn rotated) |
| `legende` | legend: `aus` / `oben` / `unten` / `links` / `rechts` (off/top/bottom/left/right) |
| `werte` | values at the data point: `aus` / `innen` / `aussen` (off/inside/outside) |
| `ausrichtung` | bar: `senkrecht` / `waagerecht` (vertical/horizontal) |
| `zeigerform` | gauge pointer: `nadel` / `balken` / `pfeil` (needle/bar/arrow) |
| `zifferblatt` | gauge dial: `ring` / `segmente` / `striche` / `baender` (ring/segments/ticks/bands) |
| `wertanzeige` | gauge/bar gauge/lamps value display: `aus` / `innen` / `pille` / `blase` / `am_zeiger` (off/inside/pill/bubble/at the pointer) |
| `punktform` | line point shape: `kreis` / `quadrat` / `raute` / `dreieck` (circle/square/diamond/triangle) |

### Numbers — `CHART_SET_NUM`

| Name | Meaning |
|---|---|
| `min`, `max` | axis limits; unset = from the data |
| `innenradius` | 0…0.95, fraction of the outer radius → donut |
| `abstand` | pie: pull segments out. Bar: gap |
| `ecken` | corner radius |
| `rahmen_dicke`, `polster` | frame thickness, padding |
| `gitter` | step size of the value axis (grid); 0 = automatic (round steps) |
| `nachkomma` | decimal places of all numbers |
| `titel_groesse`, `text_groesse` | font sizes (title, text) |
| `schrift` | FONT handle from `LOADFONT`; -1 = default font |
| `start_winkel`, `end_winkel` | gauge arc in degrees (default 135…405) |
| `striche`, `unterstriche` | major and minor ticks of the gauge scale |
| `linien_dicke`, `punkt_radius` | line chart (line width, point radius) |
| `animation` | seconds the displayed value takes to follow; 0 = immediately |
| `fenster` | sliding window for `CHART_PUSH`; 0 = unlimited |
| `schatten` | offset of the drop shadow in pixels; 0 = none |
| `schatten_weich` | softness of the shadow in pixels |
| `blatt_teile`, `blatt_luecke`, `blatt_dicke` | division of the dial (parts, gap, thickness) |
| `fassung` | metallic ring around the gauge disc (pixels) |
| `strich` | dash length of dashed lines (pixels); 0 = solid |
| `glanz` | strength of the gloss edge on surfaces with a gradient (0 = none) |
| `deckkraft` | 0…1 opacity for **all** data colours |
| `flaeche_deckkraft` | 0…1 opacity for the area under a line |

### Colours — `CHART_SET_COLOR`

`hintergrund`, `rahmen`, `gitter`, `text`, `titel`, `achse`, `zeiger`,
`flaeche`, `verlauf`, `schatten`, `verlauf_ende` (background, frame, grid,
text, title, axis, pointer, area, gradient, shadow, gradient end), as well as
the scale gradient `skala_von`, `skala_mitte`, `skala_bis` (scale from, middle,
to).

Plus `CHART_PALETTE(c, farben)` with an `ARRAY OF INTEGER` — the
order from which series and segments without their own colour are served.

### Switches — `CHART_SET_FLAG`

`rahmen`, `gitter_x`, `gitter_y`, `prozent`, `flaeche`, `punkte`, `glatt`
(curve instead of a polyline), `null_linie`, `stapel`, `verlauf`
(background gradient), `verlauf_daten`, `schatten_daten`, `kurz`
(1.2M instead of 1200000, like `NUMFMT$`). In order: frame, x grid, y grid,
percent, area, points, smooth, zero line, stacked, gradient, gradient on the
data, shadow on the data, short.

### Themes

`CHART_THEME(c, "dunkel" | "hell" | "neon" | "pastell")` (dark, light, neon,
pastel) sets all colour roles and the palette at once. After that every single
colour can still be overridden — a later `CHART_THEME`, however, resets them again.

## Designing the gauge

Four kinds of dial, via `zifferblatt` (dial):

| Value | Appearance |
|---|---|
| `ring` | a continuous arc (default) |
| `segmente` | individual bars with a gap — the classic "progress bar in a circle" (segments) |
| `striche` | the same division, but only as narrow ticks at the outer edge (ticks) |
| `baender` | full sectors down to the centre (bands) |

`blatt_teile` (number), `blatt_luecke` (gap in degrees) and `blatt_dicke`
control the division; `fassung` puts a metallic-looking ring around the
disc (width in pixels, 0 = none).

**Labelled zones.** `CHART_ZONE` takes a name as a fifth argument,
which is rotated along the arc:

```basic
CHART_ZONE(t, 0,   350,  RED,    "SCHLECHT")
CHART_ZONE(t, 350, 700,  YELLOW, "NORMAL")
CHART_ZONE(t, 700, 1000, GREEN,  "GUT")
```

Text in the lower half is turned around automatically so that it is not
upside down.

**Value display** via `wertanzeige`:

| Value | Appearance |
|---|---|
| `innen` | plain, below the centre (default) |
| `pille` | a rounded capsule — **in the colour of the zone it falls in** (pill) |
| `blase` | a dark box with a tip (bubble) |
| `am_zeiger` | a small capsule at the tip of the pointer, moves with it |
| `aus` | none |

The gauge depends on `wertanzeige` alone. The `werte` switch stays responsible
for labelling individual data points in pie and bar charts — coupling the two
would mean having two switches for the same thing.

Setting `striche` = 0 turns off the scale ticks completely (useful with `baender`, where
the sectors reach down to the centre).

## Bar gauges and lamps

The linear siblings of the gauge — one value, the same colour zones, just
straight instead of round.

```basic
DIM l AS CHART
l = CHART_NEW("leiste", 20, 20, 460, 120)
CHART_SET_NUM(l, "max", 1000)
CHART_SET(l, "zeigerform", "balken")   ' fill only up to the value
CHART_VALUE(l, 630.0)
```

Both are **horizontal** by default; `ausrichtung` = `senkrecht` (vertical) stands them
upright (the marker then moves to the side so that it doesn't cover the bar).
With `led`, `blatt_teile` determines the number of cells, `blatt_luecke` the
spacing in pixels and `blatt_dicke` the height. `zeigerform` = `balken` (bar) only fills
up to the value; otherwise the whole scale is visible and only the marker moves.

The cells of an LED display can be clicked individually — `CHART_HOVER` returns
the index of the cell under the mouse.

**Colour of the scale.** If there are colour zones (`CHART_ZONE`), they win — the same
specification thus colours gauge, bar gauge and lamps. Without zones a directed
gradient runs `skala_von` → `skala_mitte` → `skala_bis` (default red → yellow → green,
in the tone of the current theme).

> This is deliberately **not** the palette from `CHART_PALETTE`. That one is
> categorical — eight easily distinguishable colours for eight series — and gives
> a rainbow when interpolated. A scale has to show you at a glance where "little"
> and where "a lot" is.

## Font

The charts write their text with the font from `schrift` — a
FONT handle from `LOADFONT`. `-1` (the default) is the built-in pixel font;
a real TTF is much easier to read at small sizes.

```basic
DIM schrift AS INTEGER
DIM fp AS STRING
schrift = -1
FOR EACH fp IN ("C:/Windows/Fonts/segoeui.ttf", "C:/Windows/Fonts/calibri.ttf")
    IF schrift < 0 AND FILEEXISTS(fp) THEN schrift = LOADFONT(fp, 32)
NEXT
CHART_SET_NUM(c, "schrift", schrift)
CHART_SET_NUM(c, "text_groesse", 15)
CHART_SET_NUM(c, "titel_groesse", 22)
```

Load the font **large** (32 here) and set the display size via
`text_groesse` / `titel_groesse` — raylib rasterises the glyphs at the
load size; loading small and displaying large gets blurry.

The size applies separately to the title (`titel_groesse`) and everything else
(`text_groesse`: axes, legend, values, zone labels, tooltips).

## Transparency

Colours in Drachenhauch are `0xAARRGGBB`. The top byte is the opacity, and
**0 means opaque** (so that old 24-bit colours stay unchanged).
Semi-transparency is most convenient via `RGBA(r, g, b, a)`:

```basic
CHART_SET_COLOR(c, "flaeche", RGBA(0, 160, 255, 90))   ' pale blue area
CHART_SET_COLOR(c, "schatten", RGBA(0, 0, 0, 140))     ' soft shadow
```

If you don't want to touch every colour individually, turn
`deckkraft` (opacity) instead — the control multiplies the opacity of **all** data colours and
works well for making two overlapping series readable.

## Shadows

`schatten` is the offset, `schatten_weich` the number of staggered copies, each with
a fraction of the opacity — together that gives the soft edge
(raylib has no blur for shapes). `schatten_daten` switches on shadows for
the data itself too: bars, pie segments and the
gauge pointer.

```basic
CHART_SET_NUM(c, "schatten", 6)
CHART_SET_NUM(c, "schatten_weich", 6)
CHART_SET_FLAG(c, "schatten_daten", TRUE)
```

## Gradients

Two separate switches:

- `verlauf` colours the **background** of the area from `hintergrund` to
  `verlauf`.
- `verlauf_daten` colours the **data**: bars and the area under a line
  get a vertical gradient plus a gloss edge over the upper
  half (strength via `glanz`), pie segments a darkened inner band. The target colour is `verlauf_ende`; if it is not set (-1),
  a darkened version of the respective series colour is taken automatically —
  so the gradient fits every palette without any extra effort.

## Mouse: highlighting, tooltips, clicks

`CHART_DRAW` evaluates the mouse itself — no extra
call is needed. Moving the mouse over a segment, a bar or a point
makes it light up softly; the pie slice also moves out, the
line point grows.

```basic
CHART_SET_FLAG(c, "tooltip", TRUE)      ' tooltip with name and value

' per frame, after CHART_DRAW:
IF CHART_CLICKED(c) >= 0 THEN
    PRINT "Angeklickt: " + CHART_HOVER_LABEL$(c)
END IF
```

| Command | Returns |
|---|---|
| `CHART_HOVER(c)` | point under the mouse, `-1` = none |
| `CHART_HOVER_SERIES(c)` | series under the mouse, `-1` = none |
| `CHART_HOVER_LABEL$(c)` | label of this point (`""` = none) |
| `CHART_HOVER_VALUE(c)` | its value |
| `CHART_CLICKED(c)` | point clicked in **this** frame, otherwise `-1` |
| `CHART_CLICKED_SERIES(c)` | the corresponding series |

All values apply **after** `CHART_DRAW` — before that they are `-1`.

Configurable: `hover` (effect completely off), `tooltip` (tooltip bubble),
`hover_tempo` (seconds for fading in/out, 0 = immediately), `hover_weite`
(how far to move out/grow), `hover_glanz` (how strongly to brighten).

The highlight mixes the colour towards **white** instead of scaling it up.
That is intentional: with saturated colours the largest channel already clamps at 255,
so only the smaller ones grow along — a highlighted orange visibly turned
yellow and looked like a different palette entry.

## Animation

Without `animation` the chart always shows the value that was set. With
`animation` > 0 the display follows smoothly — then
`CHART_UPDATE(c, DELTA())` has to run every frame:

```basic
CHART_SET_NUM(t, "animation", 0.4)
' per frame:
CHART_VALUE(t, drehzahl)
CHART_UPDATE(t, DELTA())
CHART_DRAW(t)
```

`CHART_GET` always returns the **set** value, never the one currently displayed —
so a program can reliably read its own data back.

## Line charts in detail

| Setting | Effect |
|---|---|
| `punkte` + `punkt_radius` | show data points |
| `punktform` | `kreis` / `quadrat` / `raute` / `dreieck` (circle/square/diamond/triangle) |
| `glatt` | curve instead of a polyline (Catmull-Rom) |
| `treppe` | step line — for switching states, tariff levels, stock levels |
| `strich` | dash length in pixels; 0 = solid |
| `flaeche` + `flaeche_deckkraft` | area under the curve |
| `fadenkreuz` | crosshair lines through the point under the mouse |
| `linien_dicke` | line width |

`glatt` (smooth) and `treppe` (steps) exclude each other — the step line wins, because it makes the
more precise statement.

With dashed lines the dash phase continues over the **whole** path,
not anew at every support point. Otherwise the pattern would visibly get denser
where the points are close together.

## Live curves

`fenster` (window) turns the line chart into a chart recorder: `CHART_PUSH` appends
at the end, and the oldest value drops out at the front.

```basic
CHART_SET_NUM(linie, "fenster", 120)
' per frame:
CHART_PUSH(linie, r, FPS())
```

## Camera

Like all other drawing commands, `CHART_DRAW` draws in world coordinates. For
a display that should stay fixed on screen, call `CAMERA_RESET()` beforehand —
just as with `TEXT`.

## Limits

- **Pie gradients are an approximation.** A real radial gradient is not available with the
  ring primitive; for pies `verlauf_daten` darkens the inner band
  instead. Visually that gives the same sense of depth, mathematically
  it is not a gradient.
- **Rounded corners and the gradient overlap.** The gradient is rectangular and
  sits inset by the corner radius; the narrow edge stays in the
  original colour.
- **No scatter plot, no second Y axis, no logarithmic scale.**
- **With the bar gauge pointer** the colour zones lie as a narrow outer edge on the
  ring, because the progress arc fills the same space.

## Files

- Implementation: [`rust/drachenhauch_runtime/src/chart.rs`](../../rust/drachenhauch_runtime/src/chart.rs)
  (data model + drawing), built-ins in `builtins.rs`, `CHART_DRAW` in `vm.rs`
- Demo: [`examples/154_chart.dh`](../../examples/154_chart.dh)
- Tests: [`tests/pruef/modules_chart.dhtest`](../../tests/pruef/modules_chart.dhtest) +
  Rust `#[test]`s in `chart.rs`
