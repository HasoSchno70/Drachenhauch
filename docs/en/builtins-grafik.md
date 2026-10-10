# Graphics built-ins

Graphics, sound and input — native in the runtime `dhrt` (raylib). All commands here need an open window — so `SCREEN(...)` must be called before anything else.

When the `camera` module is active and `CAMERA_SET` has been called, all drawing commands interpret their coordinates as **world coordinates** (see [camera module](module-camera.md)).

## Contents

- [Window and frame](#window-and-frame)
- [Monitors and window position](#monitors-and-window-position)
- [Drawing](#drawing)
- [Images](#images)
- [Asset preloader (`LOAD_ASSETS`)](#asset-preloader)
- [Sprite atlas + batch draw](#sprite-atlas)
- [Z-layer rendering](#z-layer-rendering)
- [Tilemap](#tilemap)
- [Sound and music](#sound-and-music)
- [Input: keyboard and mouse](#input-keyboard-and-mouse)

## Window and frame

| Function | Purpose |
|---|---|
| `SCREEN(w, h[, titel$[, scale]])` | opens a window with the logical size `w × h`. `scale > 1` makes every logical pixel bigger (retro look). |
| `FLIP()` | output the frame to the screen (synchronized to 60 FPS) |
| `CLS([color])` | fill the buffer with `color` (default: black) |
| `SLEEP(ms)` | waits ms milliseconds without freezing the window |
| `QUITREQUESTED()` → BOOLEAN | TRUE when the program should end: window X / Alt+F4, the frame limit of `DHRT_FRAMES`/`--bilder` — **and, by raylib default, also `ESC`**, EXCEPT in a program with `gui`: there `ESC` belongs to the widgets from the first `GUI_UPDATE` on (cancel a dialog, close a dropdown), otherwise every cancel would end the whole program. `WINDOW_ESC_QUIT` sets it explicitly |
| `WINDOW_CLOSE_REQUESTED()` → BOOLEAN | TRUE when the USER wants to close the window (the X, Alt+F4, the quit key) -- without the frame limit. The window stays open: a program with unsaved work asks first and then ends itself. An event, not a state -- it holds for one frame |
| `WINDOW_RESIZABLE(an)` | may the user drag the window's size? (default: no) |
| `WINDOW_MIN_SIZE(w, h)` | smallest window size when dragging |
| `WINDOW_MAX_SIZE(w, h)` | largest window size when dragging |
| `WINDOW_MAXIMIZE()` | bring the window to screen size |
| `WINDOW_MINIMIZE()` | put the window into the taskbar |
| `WINDOW_RESTORE()` | bring it back from maximized or minimized |
| `SETWINDOWTITLE(titel$)` | change the window title while running — for the file name or the score, say |
| `SET_FULLSCREEN(an)` | switch fullscreen on or off |
| `WINDOW_IS_FULLSCREEN()` → BOOLEAN | is it running in fullscreen right now? |
| `WINDOW_FOCUSED()` → BOOLEAN | is the window in the foreground? — use it to pause when the user clicks away |
| `WINDOW_MINIMIZED()` / `WINDOW_MAXIMIZED()` / `WINDOW_HIDDEN()` → BOOLEAN | query the window's state |
| `WINDOW_FOCUS()` | bring your own window to the front |
| `WINDOW_HIDE()` / `WINDOW_SHOW()` | make the window disappear, from the taskbar too, and bring it back — for tools that live in the tray |
| `TRAY_SHOW([bild[, hinweis$]])` | show an icon in the notification area — on Windows in the taskbar, on macOS in the menu bar, on Linux in the desktop's panel (via D-Bus); without an image the one from `WINDOW_ICON`, otherwise the default icon; the hint appears on hover (default: the window title) |
| `TRAY_HIDE()` / `TRAY_SHOWN()` → BOOLEAN | remove the icon; is it showing right now? It disappears by itself when the program ends |
| `TRAY_TOOLTIP(hinweis$)` | change the hint (to show progress, say) |
| `TRAY_MENU(eintraege$)` | the right-click menu (on macOS also Ctrl+click); the entries stand in one text separated by vertical bars, a single `-` is a separator, at most 64 entries |
| `TRAY_CLICKED()` / `TRAY_DOUBLE_CLICKED()` → BOOLEAN | was the icon clicked in this frame? Holds for exactly one frame, like `GUI_CLICKED`; Linux does not report a double click |
| `TRAY_MENU_CLICKED$()` → STRING | the text of the menu entry chosen in this frame, otherwise `""` |
| `NOTIFY(titel$[, text$])` | a system notification — under Windows at the tray icon (without an icon it creates one), under macOS via `osascript`, under Linux via `notify-send` |
| `NOTIFY_CLICKED()` → BOOLEAN | was the notification clicked in this frame? (Windows) |
| `WINDOW_DPI_X()` / `WINDOW_DPI_Y()` → FLOAT | screen scaling (1.0 normal, 2.0 HiDPI) — without it a program cannot know whether its pixel sizes come out tiny on the target device |
| `WINDOW_RENDER_WIDTH()` / `WINDOW_RENDER_HEIGHT()` → INTEGER | the window's real pixel size — larger than `SCREENWIDTH()`/`SCREENHEIGHT()` on a HiDPI screen |
| `WINDOW_WAIT_EVENTS(sekunden)` | `FLIP` waits for input, at most this long — a tool that only reacts to clicks then needs hardly any CPU time; `0` switches it off |
| `FPS()` → INTEGER | measured frames per second |
| `DRAWFPS(x, y[, farbe])` | show the frame rate as “60 FPS” in the built-in font, in screen coordinates (the camera does not apply); without a colour green, below 30 orange, below 15 red |
| `SETFPS(n)` | target frame rate; `0` = as fast as possible |
| `FILES_DROPPED()` → INTEGER | how many files were dragged into the window in this frame or (macOS) handed over by the Finder? Holds for exactly one frame, can be queried any number of times |
| `FILE_DROPPED(i)` → STRING | path of the `i`-th of them |
| `CLIPBOARD_GET()` → STRING | read text from the clipboard; empty if another program is holding it right now |
| `CLIPBOARD_SET(text$)` | put text into the clipboard — an **error** if that does not succeed |
| `GFX_PUSH()` | save the drawing state: camera, layers, light, environment, shadows, 3D camera, font, `POSTFX` |
| `GFX_POP()` | restore it — **an error without a preceding `PUSH`** |
| `GFX_DEPTH()` → INTEGER | how deep is the stack? |

### The tray icon on macOS and Linux

The `TRAY_*` commands behave the same on all three systems; what the system
contributes differs:

* **macOS:** the icon sits in the menu bar, scaled to 18 points. A click is
  `TRAY_CLICKED`, two in quick succession also `TRAY_DOUBLE_CLICKED`; the
  right button or Ctrl+click opens the menu.
* **Linux:** the icon registers with the panel over D-Bus (KDE's
  *StatusNotifierItem* protocol, menu included). KDE Plasma, Xfce, LXQt and
  Cinnamon show it; **GNOME only with the "AppIndicator"
  extension**. Without a panel the icon stays invisible, but `TRAY_SHOW` is
  not an error — if the panel starts later, the icon registers by itself.
  Without a D-Bus session (over SSH, say) `TRAY_SHOW` is an error. The
  protocol has no double click.
* **Notifications** (`NOTIFY`) go through `osascript` on macOS and
  `notify-send` on Linux; neither reports a click on them.
* `dhrt pruef tray` runs the system's icon once for real — create, read
  back, click and menu choice — and says what came out.

A whole tool that lives in the notification area is
`examples/209_pausenwecker.dh`: menu, a hint that counts down, a
notification, and the close button only hides the window.

### The clipboard belongs to one program at a time

Under Windows only **one** process can have the clipboard open at any time.
Whoever needs it at that moment is turned away — and in everyday use that
happens constantly: a password manager, a clipboard history, a second program
that is copying right now.

`CLIPBOARD_SET` therefore waits and tries again (measured up to about
**700 ms** of blocking; after that it gives up after 711 ms). If it still does
not succeed, that is a **runtime error** — not silence. The reason for this
is unpleasantly concrete: if writing fails silently, the **old** contents
appear on the next paste, and the damage shows up somewhere else entirely.
Whoever wants to catch it uses `TRY ... CATCH`.

`CLIPBOARD_GET` returns an **empty text** in that case. Until 2026-09-20 it
crashed there (access violation) — GLFW returns a null pointer, which was
accessed unchecked. That hit every program with Ctrl+C/V, not just tests.

```basic
TRY
    CLIPBOARD_SET(text$)
CATCH e
    TEXT(10, 10, "Kopieren ging gerade nicht")
END TRY
```

Classic game loop:

```basic
SCREEN(320, 240, "Mein Spiel", 2)

WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 20, 30))

    ' ... draw ...

    FLIP()
    SLEEP(16)
WEND
```

`scale=2` opens a 640×480 window in which all coordinates are still computed in the 320×240 logical resolution — the pixels are scaled up.

## Monitors and window position

Display information and placing the program window on the desktop. The monitor index runs from `0` to `MONITOR_COUNT()-1`. All measurements here are **real OS pixels** (no `scale`), because they describe the hardware or the position on the desktop — not the logical `SCREEN` grid.

| Function | Purpose |
|---|---|
| `MONITOR_COUNT()` → INTEGER | number of connected monitors |
| `CURRENT_MONITOR()` → INTEGER | index of the monitor the window mostly lies on right now |
| `MONITOR_WIDTH(i)` → INTEGER | native width of monitor `i` (px) |
| `MONITOR_PHYSICAL_WIDTH(i)` / `MONITOR_PHYSICAL_HEIGHT(i)` → INTEGER | size of monitor `i` in millimetres, as it reports it (0 if it does not) — together with `MONITOR_WIDTH`, the real pixel density |
| `MONITOR_HEIGHT(i)` → INTEGER | native height of monitor `i` (px) |
| `MONITOR_REFRESH(i)` → INTEGER | refresh rate of monitor `i` (Hz) |
| `MONITOR_NAME(i)` → STRING | display name of monitor `i` |
| `MONITOR_X(i)`, `MONITOR_Y(i)` → INTEGER | position of monitor `i` in the virtual desktop (px) |
| `SET_WINDOW_MONITOR(i)` | move the window to monitor `i` (an invalid index is ignored) |
| `WINDOW_X()`, `WINDOW_Y()` → INTEGER | position of the window's top left corner (px) |
| `SET_WINDOW_POS(x, y)` | put the window at the OS pixel position `(x, y)` |

### Real native fullscreen

| Function | Purpose |
|---|---|
| `SCREEN_NATIVE([titel$])` | fullscreen in the **real resolution** of the current monitor |

`SCREEN_NATIVE()` is the direct route to sharp fullscreen and replaces `SCREEN(...)`:

```basic
SCREEN_NATIVE("Mein Spiel")
' SCREENWIDTH()/SCREENHEIGHT() now return the monitor resolution
```

The difference from combining `SCREEN(w, h)` **+** `SET_FULLSCREEN(TRUE)`: with the latter a small back buffer (e.g. 1280×720) is **scaled up** to the monitor → blurry. `SCREEN_NATIVE()`, by contrast, renders 1:1 in native pixels (logical grid = monitor resolution). Whoever wants to set the logical resolution themselves still uses the manual variant:

```basic
DIM m AS INTEGER : m = CURRENT_MONITOR()
SCREEN(MONITOR_WIDTH(m), MONITOR_HEIGHT(m), "Vollbild", 1)
SET_FULLSCREEN(TRUE)
```

Complete example with all monitors, live window position and monitor switching: [`examples/120_monitors.dh`](../../examples/120_monitors.dh).

### Transparent windows & desktop overlay

The window background can show through so that the desktop stays visible — for floating overlays (visualizer, HUD, effect) or windows with a “glass” background.

| Function | Purpose |
|---|---|
| `SCREEN_TRANSPARENT(w, h[, titel$[, scale]])` | opens a window with a **transparent** background; `w`/`h` = 0 → the whole current monitor (fullscreen overlay) |
| `WINDOW_UNDECORATED(flag)` | window frame/title bar off (`TRUE`) / on (`FALSE`) |
| `WINDOW_TOPMOST(flag)` | keep the window always in the foreground |
| `WINDOW_ESC_QUIT(an)` | `ESC` as the window close key on/off (raylib default: **on**, with `gui` **off** from the first `GUI_UPDATE` on). With `FALSE`, `ESC` is a completely normal key (`QUITREQUESTED` is then only triggered by window X / Alt+F4) — for games that use `ESC` for the pause/main menu. An explicit `WINDOW_ESC_QUIT(TRUE)` applies with `gui` too |
| `WINDOW_PASSTHROUGH(flag)` | **pass** mouse clicks through to the desktop (click-through widget) |

**Important:** `SCREEN_TRANSPARENT(...)` must be the **very first** graphics statement (before `LOADIMAGE`/`SCREEN`/…). Transparency is a window creation flag and cannot be set afterwards. `WINDOW_UNDECORATED`/`WINDOW_TOPMOST`/`WINDOW_PASSTHROUGH`, on the other hand, at any time. `WINDOW_PASSTHROUGH(TRUE)` needs a borderless window (`WINDOW_UNDECORATED(TRUE)`); the keyboard (e.g. `ESC`) then only reaches the window while it has the focus — after a desktop click, end it with the stop button in the editor or `Alt+F4`.

In transparent mode `CLS` takes the **alpha byte literally**:

```basic
CLS()              ' fully transparent -> the desktop shows through everywhere
CLS(&HC0101826)    ' semi-transparent background (alpha 0xC0) -> desktop shimmers through dimmed
```

(In the normal, non-transparent mode the background stays opaque as always.)

**Desktop overlay** (borderless, always on top, fully transparent):

```basic
SCREEN_TRANSPARENT(420, 420, "Overlay")
WINDOW_UNDECORATED(TRUE)
WINDOW_TOPMOST(TRUE)
WHILE NOT QUITREQUESTED()
    CLS()                       ' only what is drawn is visible
    CIRCLE(210, 210, 100, RGB(60, 200, 255))
    FLIP()
WEND
```

Examples: [`examples/123_overlay.dh`](../../examples/123_overlay.dh) (overlay), [`examples/124_glass_window.dh`](../../examples/124_glass_window.dh) (glass window), [`examples/125_vortex_overlay.dh`](../../examples/125_vortex_overlay.dh) (vortex overlay), [`examples/126_audio_overlay.dh`](../../examples/126_audio_overlay.dh) (click-through music visualizer via `AUDIO_FFT`).

> Note: transparency works in the direct render path. With an active post-processing shader (`POSTFX`) the screen is presented opaque.

## Native file dialogs

Real Windows dialogs for choosing files/folders. All three are **blocking** (modal system dialog) and return the chosen path as a STRING — an **empty string** on cancel. `endungen$` is a comma-separated list of file extensions without the dot (e.g. `"png,jpg,gb"`).

| Function | Purpose |
|---|---|
| `FILE_OPEN_DIALOG([titel$[, endungen$]])` → STRING | choose a file to open |
| `FILE_SAVE_DIALOG([titel$[, default$[, endungen$]]])` → STRING | choose a file to save (`default$` = suggested file name) |
| `FOLDER_DIALOG([titel$])` → STRING | choose a folder |

```basic
DIM pfad AS STRING
pfad = FILE_OPEN_DIALOG("Bild laden", "png,jpg")
IF pfad <> "" THEN
    DIM bild AS IMAGE : bild = LOADIMAGE(pfad)
    ' ...
END IF
```

Complete example: [`examples/127_filedialog.dh`](../../examples/127_filedialog.dh).

## Drawing

Colours are given as a 24-bit INTEGER (`&HRRGGBB`), most easily via `RGB(r, g, b)` or the [colour constants](sprache.md#built-in-constants) (`RED`, `GREEN`, …).

| Function | Purpose |
|---|---|
| `PLOT(x, y[, color])` | single pixel |
| `LINE(x1, y1, x2, y2[, color])` | line (1px) |
| `LINEW(x1, y1, x2, y2, breite[, color])` | line with a stroke width (float) |
| `BOX(x1, y1, x2, y2[, color])` | filled rectangle |
| `RECT(x1, y1, x2, y2[, color])` | rectangle outline (1px) |
| `BOXROUND(x1, y1, x2, y2, radius[, color])` | filled rectangle with rounded corners |
| `RECTROUND(x1, y1, x2, y2, radius[, color])` | rectangle outline with rounded corners |
| `GRADIENTV(x1, y1, x2, y2, farbe1, farbe2)` | rectangle with a **vertical** colour gradient (top→bottom) |
| `GRADIENTH(x1, y1, x2, y2, farbe1, farbe2)` | rectangle with a **horizontal** colour gradient (left→right) |
| `GRADIENT4(x1, y1, x2, y2, oben_links, oben_rechts, unten_rechts, unten_links)` | rectangle with one colour in each corner, clockwise from top left; mixed in between |
| `BOXROT(x, y, breite, hoehe, winkel[, farbe])` | filled rectangle **centred** on (x, y), rotated about its centre (degrees, clockwise like `DRAWIMAGEROT`) |
| `CIRCLE(x, y, r[, color])` | filled circle |
| `CIRCLEOUTLINE(x, y, r[, color])` | circle as outline only (counterpart to `CIRCLE`) |
| `CIRCLE_GRADIENT(x, y, r, innen, aussen)` | circle with a gradient from the centre (`innen`) to the edge (`aussen`); with `RGBA` as the edge colour, a soft glow |
| `PIE(x, y, r, von_grad, bis_grad[, farbe])` | filled circle sector (pie slice); angles in **degrees**, 0 = right, clockwise |
| `PIEOUTLINE(x, y, r, von_grad, bis_grad[, farbe])` | circle sector as outline only, with both radii |
| `NGON(x, y, ecken, r, winkel[, farbe])` | filled regular polygon (3..1000 corners) around (x, y), radius out to the corners, rotated by `winkel` degrees |
| `NGONOUTLINE(x, y, ecken, r, winkel[, farbe[, breite]])` | regular polygon as outline only; `breite` as with `LINEW` |
| `LINEDASHED(x1, y1, x2, y2, strich, luecke[, farbe])` | dashed line, dash and gap length in pixels |
| `SPLINE(xs, ys[, color[, breite]])` | smooth Catmull-Rom curve through the points; `xs`/`ys` are `ARRAY OF INTEGER` of equal length |
| `SPLINE_BASIS(xs, ys[, farbe[, breite]])` | B-spline: even smoother than `SPLINE`, but does **not** run through the points, it is pulled by them (at least 4) |
| `SPLINE_BEZIER(xs, ys[, farbe[, breite]])` | chain of cubic Bézier curves: start, then two control points and one end each (4, 7, 10 … points) — the way drawing programs store paths |
| `BEZIER(x1, y1, kx, ky, x2, y2[, farbe[, breite]])` | quadratic Bézier curve from (x1, y1) to (x2, y2), bent towards the control point (kx, ky) |
| `BEZIER3(x1, y1, k1x, k1y, k2x, k2y, x2, y2[, farbe[, breite]])` | cubic Bézier curve with two control points (S-curves, connecting lines between nodes) |
| `TRIANGLE(x1, y1, x2, y2, x3, y3[, color])` | filled triangle |
| `TRIANGLEOUTLINE(x1, y1, x2, y2, x3, y3[, color[, width]])` | triangle as outline only |
| `POLYGON(points[, color])` | filled polygon — `points` is an `ARRAY OF INTEGER` with `[x1, y1, x2, y2, …]` (at least 3 points) |
| `POLYGONOUTLINE(points[, color[, width]])` | polygon as outline only |
| `ELLIPSE(x1, y1, x2, y2[, color])` | filled ellipse, fitted into the bounding box |
| `ELLIPSEOUTLINE(x1, y1, x2, y2[, color[, width]])` | ellipse as outline only |
| `ARC(x1, y1, x2, y2, start_rad, end_rad[, color[, width]])` | arc segment in the bounding box; angles in radians, counter-clockwise. `width` (float, optional) = stroke width; without it 1px (independent of zoom), with it scaled with the `CAMERA_SET` zoom like `LINEW`/`SPLINE` |
| `TEXT(x, y, s$[, color])` | text at (x, y) |
| `TEXTROT(x, y, s$, winkel[, skala[, farbe]])` | text **centred** on (x, y), rotated about the centre (degrees, like `DRAWIMAGEROT`) and scaled — for score pop-ups, slanted labels. Uses the active font/size |

> **Corner order does not matter:** filled `TRIANGLE` and `POLYGON` draw
> regardless of the winding — whether the points are given clockwise or
> counter-clockwise, the area always appears (dhrt flips it internally when
> needed).

```basic
SCREEN(320, 240, "Zeichnen-Demo", 2)

CLS(RGB(20, 20, 30))

PLOT(100, 50, RGB(255, 255, 255))
LINE(0, 0, 319, 239, RGB(255, 0, 0))
BOX(50, 50, 150, 100, RGB(0, 200, 0))     ' filled
RECT(50, 110, 150, 160, RGB(0, 200, 0))   ' outline
CIRCLE(200, 80, 30, RGB(255, 200, 0))
TRIANGLE(20, 200, 60, 200, 40, 230, RGB(255, 100, 255))

' polygon with a point array
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

**Note on BOX/RECT:** both take `(x1, y1, x2, y2)` as corner points (inclusive). `BOX(0, 0, 9, 9)` draws 10×10 pixels.

## Transparency and blend modes

Colours can carry an **alpha channel** (`&Haarrggbb`, top byte = opacity):

| Function | Purpose |
|---|---|
| `RGBA(r, g, b, a)` → INTEGER | colour with alpha (`a` 0..255, 255 = fully opaque) |
| `ALPHA(farbe)` → INTEGER | read the alpha channel (0..255) from a colour |
| `BLEND_MODE(modus$)` | blend mode for subsequent draws: `"alpha"` (default), `"add"` (additive – glow/light), `"mult"` (multiplicative – shadow/tint), `"subtract"` |

`BLEND_MODE` applies until the next call — after an effect, set it back with `BLEND_MODE("alpha")`.

```basic
' Glowing sparks: overlay additively -> bright overlaps
BLEND_MODE("add")
FOR i = 0 TO 20
    CIRCLE(RND(320), RND(240), 8, RGBA(255, 180, 60, 120))
NEXT
BLEND_MODE("alpha")          ' back to normal mode
```

## Clip rectangle (scissor)

| Function | Effect |
|---|---|
| `SCISSOR(x, y, breite, hoehe)` | subsequent draws are clipped to this rectangle |
| `SCISSOR_END()` | remove the topmost restriction |
| `SCISSOR_DEPTH()` → INTEGER | how many clips are currently open |

This is a **stack**: an inner `SCISSOR` is *intersected* with the outer one,
it does not replace it. The coordinates are world coordinates and follow the
camera like any other draw.

```basic
' A scrollable list: only the visible section shows
SCISSOR(20, 40, 200, 120)
FOR i = 0 TO 30
    TEXT(24, 44 + i * 20 - versatz, "Zeile " + STR$(i))
NEXT
SCISSOR_END()
```

A `SCISSOR_END()` without an open `SCISSOR` is an **error** — otherwise it
would take away the restriction of the surrounding code (a `gui` window, for
instance), and that would only show up as drawing beyond the edge. A
forgotten `SCISSOR_END()`, on the other hand, only has an effect until the
end of the frame.

Without this command, the only way to a limited drawing area was the detour
through a render target (`RENDERTARGET_NEW`/`BEGIN`/`END`/`DRAW`) — that is,
a second canvas with its own memory, where a rectangle is enough.

## Font

| Function | Effect |
|---|---|
| `TEXT(x, y, s$[, color])` | text at (x, y) in the active font |
| `TEXT_SIZE(px)` | font size for subsequent `TEXT` calls (4–400) |
| `TEXT_WIDTH(s$)` | pixel width of `s$` in the active font/size |
| `TEXT_HEIGHT()` | line height of the active font |
| `TEXT_STYLE(stil$)` | style for subsequent `TEXT` calls: `fett` (bold), `kursiv` (italic), `unterstrichen` (underlined), `durchgestrichen` (strikethrough), joined with `+` (`"fett+unterstrichen"`); `"normal"` switches back |
| `TEXT_GET_STYLE$()` | the current style as text, e.g. `"fett+kursiv"` |
| `TEXT_BOLD(an)` / `TEXT_ITALIC(an)` | switch only bold or italic on or off; the rest of the style stays |
| `FONT_STYLE(font, stil$)` → FONT | the real bold/italic face of a loaded font (looks for the file next to it, e.g. `segoeuib.ttf` for `segoeui.ttf`); if there is none, `font` itself comes back |
| `FONT_HAS_STYLE(font, stil$)` | TRUE if this face exists as a file -- otherwise the runtime imitates it |
| `LOADFONT(pfad$, groesse[, zeichen$])` → FONT | load TTF/OTF/TTC → FONT handle (INTEGER); `zeichen$` = script blocks (`"kyrillisch, griechisch"`, `"japanisch"`, `"emoji"` …) or the characters themselves that should be baked |
| `LOADFONT_SDF(pfad$, groesse[, zeichen$])` → FONT | a font that stays sharp at **any** size: baked as a distance field, drawn with a shader that keeps the edge soft by one pixel. An ordinary font blurs as soon as it is drawn larger than it was baked (titles, camera zoom). Without `zeichen$` ASCII and Latin-1 including €; baking costs more per character than with `LOADFONT` (about 130 ms for this selection) |
| `FONT_HAS_GLYPH(font, zeichen$)` → BOOLEAN | does the font itself have a glyph for it? If not, `TEXT` draws it with the fallback font; `-1` = built-in font |
| `FONT_GLYPH_WIDTH(font, zeichen$, groesse)` → FLOAT | how far `TEXT` advances after this character at `groesse` points — measured like `TEXT_WIDTH`, without switching the active font |
| `FONT_TO_IMAGE(font[, erstes[, letztes]])` → IMAGE | the characters `erstes`..`letztes` (default 32..126, at most 256) as an image in the format of `LOADFONT_IMAGE`: each character white in a transparent field, magenta (`&HFF00FF`) outside and in between. That turns a vector font into a pixel font that you can touch up and load again with `LOADFONT_IMAGE(bild, &HFF00FF, erstes)` |
| `SETFONT(font)` | set the active font; `SETFONT(-1)` = default font |
| `TEXT_SPACING(px)` | letter spacing for TTF (native) |
| `TEXT_LINE_SPACING(px)` | line spacing for multi-line text |

`LOADFONT` loads a TrueType/OpenType font of your own; `TEXT_SIZE` then
scales it freely. `TEXT_WIDTH` measures in the **active** font — which lets
you centre or right-align text. The native runtime renders real glyphs
(raylib `LoadFontEx`/`DrawTextEx`).

**Bold and italic use the real face if there is one.** For a font loaded via
`LOADFONT`, the runtime looks for the face's file in the same folder -- for
Windows fonts by their fixed names (`segoeui` → `segoeuib`/`segoeuii`/
`segoeuiz`, `arial` → `arialbd` …), otherwise by `-Bold`/`-Italic`/
`-BoldItalic` (`NotoSans-Regular.ttf` → `NotoSans-Bold.ttf`). It is loaded
once, at the same size and with the same characters. **If there is no face**
(the built-in font, a bitmap font, a font without siblings), the runtime
draws bold as a second stroke offset by one pixel and italic with slanted
glyphs -- the style is visible either way. Underline and strikethrough are
lines in the text colour and work with any font. `TEXT_WIDTH` measures with
the style; `TEXTROT` does not know it.

```basic
DIM f AS INTEGER : f = LOADFONT("C:/Windows/Fonts/segoeui.ttf", 32)
SETFONT(f)
TEXT_STYLE("fett")
TEXT(20, 20, "Überschrift", WHITE)
TEXT_STYLE("kursiv+unterstrichen")
TEXT(20, 60, "Hervorgehoben", YELLOW)
TEXT_STYLE("normal")
```

```basic
DIM titlefont AS INTEGER
titlefont = LOADFONT("assets/PressStart2P.ttf", 32)
SETFONT(titlefont)
TEXT_SIZE(32)
DIM w AS INTEGER
w = TEXT_WIDTH("GAME OVER")
TEXT(320 - w \ 2, 100, "GAME OVER", RGB(255, 60, 60))   ' centred
SETFONT(-1)                                             ' back to the default
```

Demo: [examples/87_ttf_fonts.dh](../../examples/87_ttf_fonts.dh).

### Umlauts, the euro sign and foreign scripts

`TEXT(x, y, "Köln")` draws **Köln**, not `K?ln`, and `TEXT(x, y, "12,50 €")`
shows the euro sign — until September 2026 it was a question mark, as were
`ő`, `ł`, `Ω`, `Я`, every kanji and every emoji (measured in
[entwurf-eingabemethoden.md](../entwurf-eingabemethoden.md) (German)). Three
things take care of this:

- **The basic character set** of every font — the fallback font and every
  font loaded via `LOADFONT` — covers ASCII, Latin-1, Latin Extended-A/B (the
  languages of Central Europe), Greek, Cyrillic, general punctuation
  (`… – „ “`) and `€`. If the font file does not contain a character, raylib
  draws a `?` there; a pixel font for a retro game often has no umlauts.
- **Without a font of your own, a fallback font steps in.** The built-in
  raylib font knows ASCII and Latin-1 (characters 32 to 255, so `äöüÄÖÜß` as
  well). If a character beyond that occurs (such as `€` or kanji), the
  runtime draws this text with a system font (Windows: Segoe UI, macOS:
  SF/Helvetica, Linux: DejaVu/Liberation). German text with umlauts stays in
  the built-in font -- until 2026-10-01 every text with a character beyond
  ASCII switched to the fallback, and in a user interface “Löschen” stood in
  a different font from “Buchen” next to it.
- **Glyphs on demand.** If a character is in no loaded font (kanji, hangul,
  emoji, Arabic, Hebrew, Thai), the runtime notes it while drawing or
  measuring and bakes it at the next `FLIP` from the matching system font —
  Windows: MS Gothic, Malgun Gothic, Segoe UI Emoji, Segoe UI; macOS: Arial
  Unicode; Linux: Noto Sans CJK, if installed. Only what was needed is baked,
  not the whole block. Measured: the first frame with kanji, hangul, emoji
  and Hebrew at once costs about 100 ms once, every further new character
  about 15 ms, after that nothing. This also applies within a font you loaded
  yourself: if it lacks a character, the fallback font steps in for exactly
  that character.

Whoever wants to choose the character set themselves passes it to `LOADFONT`
as a third argument — block names, German or English, separated by commas:
`latein`, `griechisch`, `kyrillisch`, `hebraeisch`, `arabisch`, `thai`,
`japanisch`, `chinesisch`, `koreanisch`, `emoji`, `symbole`. Or simply the
characters the program needs (the cheapest option for a game with fixed
texts). The basic set is always included. An unknown name is an error that
lists the known ones.

```basic
DIM jp AS INTEGER
jp = LOADFONT("C:/Windows/Fonts/msgothic.ttc", 24, "japanisch")
SETFONT(jp)
TEXT(10, 10, "東京 こんにちは")
DIM ru AS INTEGER
ru = LOADFONT("assets/schrift.ttf", 20, "kyrillisch, griechisch")
DIM titel AS INTEGER
titel = LOADFONT("assets/titel.ttf", 48, "SPIEL VORBEI 0123456789")   ' only these characters
```

**Font collections (`.ttc`)** have worked since the same date: raylib itself
cannot read them and used to swap the font **silently** for its bitmap font —
`LOADFONT` returned a handle, and the text appeared in the wrong font without
any message. The runtime now extracts the first font of the collection (the
CJK fonts of Windows all exist only in this form). A file that still does not
yield a font is an error.

**Limits:** emoji come out in one colour (raylib does not rasterize colour
fonts); Arabic and Hebrew appear character by character from left to right
without the letters joining — text shaping and right-to-left are not built.
Whoever wants a particular glyph form (Japanese instead of Chinese forms)
loads their font themselves.

`TEXT_WIDTH` measures along the same path that `TEXT` draws — across the
fallback fonts too: a centred text with an umlaut or kanji sits where it was
measured.

**Input methods (IME).** Whoever types Japanese or Chinese through an input
method sees the conversion underlined at the cursor in a `gui` text input
and gets the confirmed text inserted directly (Windows; see
[module-gui.md](module-gui.md#accessibility)). Without a focused text
input — `INKEY$`, the `ui` module — the confirmed text arrives as if typed
via the typing queue, which holds 256 characters per frame (raylib's default
of 16 would have silently shortened a confirmed sentence).

## Images

| Function | Purpose |
|---|---|
| `LOADIMAGE(path$)` → IMAGE | load a file (PNG, JPG, BMP, …) |
| `IMAGEWIDTH(img)`, `IMAGEHEIGHT(img)` → INTEGER | pixel size; equivalently `IMAGE_WIDTH`/`IMAGE_HEIGHT` in the naming scheme of the other `IMAGE_*` commands |
| `GETPIXEL(img, x, y)` → INTEGER | read the pixel colour (`&HRRGGBB`) at `(x, y)`; `-1` for an index outside. Counterpart to `PLOT` (write) — for per-pixel collision, masking, colour sampling |
| `GETALPHA(img, x, y)` → INTEGER | opacity at `(x, y)`, `0..255`; `-1` for an index outside. Needed because `GETPIXEL` returns a **colour**, and there opacity 0 means *opaque* — a transparent pixel would come back as a black one |
| `DRAWIMAGE(img, x, y)` | draw the image at (x, y) |
| `DRAWIMAGEPART(img, sx, sy, sw, sh, x, y)` | draw a sub-rectangle from a sheet |
| `DRAWIMAGEFLIPPED(img, x, y[, flipX[, flipY]])` | with mirroring |
| `DRAWIMAGEROT(img, x, y, winkel[, skala[, tint]])` | **centred** on (x,y), rotated by `winkel` **degrees** (about the centre), optionally scaled + tinted. Ideal for rotated sprites / `physics2d` (`winkel = DEG(PHYS2D_BODY_ANGLE(...))`). Camera-aware. |
| `DRAWIMAGE9(img, x, y, breite, hoehe, links, oben, rechts, unten[, tint])` | **9-slice**: the image stretched into the rectangle, the four corners at their own size, the edges in one direction only, the centre in both — for frames, panels and buttons that grow to any size without distorting. Edges in pixels of the image; if the target does not fit, the corners shrink proportionally. Camera-aware. |

```basic
SCREEN(320, 240, "Bilder", 2)

DIM hero AS IMAGE
hero = LOADIMAGE("assets/hero.png")
PRINT "Bild ist ", IMAGEWIDTH(hero), "x", IMAGEHEIGHT(hero)

WHILE NOT QUITREQUESTED()
    CLS()
    DRAWIMAGE(hero, 100, 100)
    DRAWIMAGEFLIPPED(hero, 150, 100, TRUE, FALSE)   ' mirrored horizontally
    FLIP()
    SLEEP(16)
WEND
```

For animated sprites with frame logic, see the [sprite module](module-sprite.md). For effects such as scaling, rotating, tinting, see the [imgfx module](module-imgfx.md).

**Asset cache:** `LOADIMAGE` and `LOADSOUND` cache their results automatically. Repeated calls with the same (or an equivalent) path return the same image, without disk I/O. The cache keys are both the raw path and the normalized absolute path, so that different spellings (`"x.png"`, `"./x.png"`, absolute) hit the same entry.

## Asset preloader

`LOAD_ASSETS(manifest_path$)` → INTEGER

Preloads all images and sounds from a JSON manifest into the cache. After the preload, all subsequent `LOADIMAGE`/`LOADSOUND` calls are cache hits (no more disk I/O). Returns the total number of assets loaded.

**Manifest format:**

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

Both sections are optional. Each can be an **object** (alias → path) OR a **list** (paths only).

**Paths** are relative to the manifest's directory (not to the running script). That way `assets/manifest.json` can be maintained centrally.

**Aliasing:** with the dict form, both `LOADIMAGE("player")` and `LOADIMAGE("sprites/player.png")` hit the cache. This keeps the script code readable (`LOADIMAGE("player")`), while the real paths are in the manifest.

**Order:** call it after `SCREEN(...)` so that images are `convert_alpha`-optimized directly.

```basic
SCREEN(640, 480, "Mein Spiel")

' Once: load all assets
DIM n AS INTEGER
n = LOAD_ASSETS("assets/manifest.json")
PRINT n; " Assets geladen"

' In the game: access them by alias
DIM hero AS IMAGE
hero = LOADIMAGE("player")    ' cache hit (alias)
```

Complete example: [examples/75_preloader.dh](../../examples/75_preloader.dh).

## Sprite atlas

A **sprite atlas** is ONE big image with named sub-rects (`x, y, w, h`). Instead of 50 individual PNG files you have one atlas PNG + a manifest — and address the parts by their name instead of by rectangles you computed yourself.

| Function | Purpose |
|---|---|
| `ATLAS_LOAD(manifest_path$)` → SPRITE_ATLAS | load an atlas from a JSON manifest |
| `ATLAS_DRAW(atlas, name$, x, y)` | draw a single sub-sprite (camera-aware) |
| `ATLAS_DRAW_FLIPPED(atlas, name$, x, y[, flip_x[, flip_y[, tint]]])` | sub-sprite with mirroring (X/Y, each `TRUE`/`FALSE` or `1`/`0`); optional `tint` |
| `BATCH_DRAW(atlas, name$, x, y)` | **second name for `ATLAS_DRAW`** — draws immediately, collects nothing |
| `BATCH_FLUSH()` | **does nothing** (no-op) — only so that old code keeps running |

**Manifest format:**

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

Rects are `[x, y, w, h]` (pixels in the atlas). Image path relative to the manifest.

**Pattern for tilemaps** (many sprites per frame):

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/tiles_atlas.json")

' 600 tiles per frame -- each call draws immediately (see the note below).
FOR row = 0 TO 19
    FOR col = 0 TO 29
        BATCH_DRAW(atlas, "tile_grass", col * 16, row * 16)
    NEXT
NEXT
BATCH_FLUSH()   ' no-op -- drawing already happened above, line by line
```

> **No real batching.** In `dhrt`, `BATCH_DRAW` is the same call as `ATLAS_DRAW`
> (one branch in the dispatch), and `BATCH_FLUSH()` does nothing at all. The runtime
> works as a **recording model**: every drawing command immediately appends a `Cmd` to
> the active layer, and `FLIP()` plays back all layers in Z order. So there is nothing
> to collect and **no saving in draw calls** — do not build your rendering on it. The
> names come from the former pygame engine, where surface blits really could be
> combined into one call; they are only kept so that old code keeps running. Whoever
> really draws many things of the same kind uses the bulk built-ins (`PLOTS`,
> `BOXES`, `CIRCLES`, `LINES`) — those save real effort.
> A zoom via `CAMERA_SET` affects `ATLAS_DRAW`/`BATCH_DRAW` equally.

**Flipping for character sprites:** `ATLAS_DRAW_FLIPPED(atlas, name$, x, y[, flip_x[, flip_y[, tint]]])` mirrors the sub-sprite on the X or Y axis. `flip_x`/`flip_y` accept `TRUE`/`FALSE` **or** `1`/`0` (missing = `FALSE`); `tint` is an optional 7th colour parameter. Classic pattern for walk animations: only one direction (right) in the atlas, left is derived by flipping:

```basic
DIM flip AS BOOLEAN
flip = CHAR_FACING(player) = -1     ' -1 = facing left
ATLAS_DRAW_FLIPPED(mario, "walk_a", x, y, flip, FALSE)
```

Flip creates a freshly mirrored image on each call. For many repeated flips of the same sprite it pays to cache the mirrored variant once, precomputed, as an IMAGE — for a single player sprite per frame, though, the overhead is negligible.

Complete example: [examples/76_layers_atlas.dh](../../examples/76_layers_atlas.dh).

## Z-layer rendering

Z-layers give you an explicit render order: background → sprites → UI, without having to manage the draw order meticulously in your code.

A layer is a **command list** with a z value — not an image buffer. Every
draw call appends an entry to the currently active list; `FLIP` sorts the
lists by z (lowest = at the back) and plays them back in that order. This is
the same recording model as with [`BATCH_DRAW`](#sprite-atlas) — there are no
intermediate images you could read back or blend individually.

| Function | Purpose |
|---|---|
| `LAYER_DEFINE(name$, z)` | register a layer with an explicit z (re-defining only updates z) |
| `LAYER(name$)` | switch the active draw target to the layer (auto-define if new) |
| `LAYER_END()` | back to the main buffer (optional, FLIP does it too) |
| `LAYER_CLEAR(name$)` | empty a layer's list manually (rarely needed, `FLIP` empties all of them anyway) |

**Classic game loop pattern:**

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

    FLIP()    ' plays the layers in z order and empties them afterwards
WEND
```

**After every `FLIP` the lists are empty** — you do not have to empty them
yourself. Where a layer drew nothing, whatever lies beneath simply stays
visible. If a layer should have an opaque background (e.g. the bg layer):
`CLS(...)` as the first draw call on the layer.

**Backwards compat:** code without `LAYER_*` calls runs unchanged, directly on the main buffer.

**Camera note:** the camera is global and applies to all layers. For UI without a camera effect (e.g. a HUD): call `CAMERA_RESET()` before drawing on `LAYER("ui")` (needs `IMPORT "camera"`).

**Layers + atlas combined** (complete in [examples/76_layers_atlas.dh](../../examples/76_layers_atlas.dh)):

```basic
LAYER("bg")
FOR each tile:
    BATCH_DRAW(atlas, "tile_grass", x, y)
NEXT
BATCH_FLUSH()     ' no-op; the tiles have long been on the bg list
LAYER("ui")       ' only switches the active list -- nothing gets lost
TEXT(10, 10, "Score: " + STR$(score))
FLIP()
```

## Tilemap

`DRAWTILEMAP(tileset, map, tileW, tileH, screenX, screenY)`

Draws a 2D map from tile indices. `tileset` is a sheet whose frames are arranged horizontally+vertically. `map` is a 2D `ARRAY OF INTEGER` with tile numbers (`-1` = transparent, no tile).

```basic
SCREEN(320, 240, "Tilemap-Demo", 2)

DIM tileset AS IMAGE
tileset = LOADIMAGE("assets/tileset.png")    ' e.g. 64x16, 4 tiles of 16x16

DIM map[10, 15] AS INTEGER
DIM r AS INTEGER
DIM c AS INTEGER
FOR r = 0 TO 9
    FOR c = 0 TO 14
        IF r = 0 OR r = 9 OR c = 0 OR c = 14 THEN
            map[r, c] = 1                    ' wall tile
        ELSE
            map[r, c] = 0                    ' floor tile
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

`map[r, c]` is `[zeile, spalte]` (row, column). Layout of the tile frames in the sheet: `tiles_pro_zeile = sheet_breite / tileW`.

## Sound and music

| Function | Purpose |
|---|---|
| `LOADSOUND(path$)` → SOUND | load a sample (short effects: WAV, OGG) |
| `PLAYSOUND(s[, loops[, volume]])` | play, default loops=0, volume=1.0 |
| `STOPSOUND(s)` | stop |
| `UNLOADSOUND(s)` | stop the sound **and free its buffer** — against buffers piling up with many `AUDIO_TONE`/`AUDIO_SFX`/`AUDIO_NOISE` notes (a long song). The handle stays valid; playing it again raises an error |
| `AUDIO_SOUND_COUNT()` → INTEGER | number of live (not freed) sound slots — diagnostics against sound leaks |
| `PLAYMUSIC(path$[, loops[, volume]])` | stream a longer file (`.ogg`/`.mp3`/`.qoa` **+ tracker modules `.mod`/`.xm`** — real Amiga sound), default loops=-1 (endless) |
| `STOPMUSIC()` | stop |

```basic
DIM coin_snd AS SOUND
coin_snd = LOADSOUND("assets/pickup.wav")

PLAYMUSIC("assets/menu.ogg", -1, 0.7)        ' endless, 70% volume

' In the game loop, when a coin is collected:
PLAYSOUND(coin_snd)
```

## Many shapes at once

Whoever draws a thousand stars does not call `PLOT` a thousand times. The
bulk commands take arrays and do everything in one call — that saves the
overhead per command, not the drawing itself.

| Function | Purpose |
|---|---|
| `PLOTS(xs, ys, farbe [, anzahl])` | many pixels; `farbe` as a number applies to all, as an ARRAY per pixel |
| `BOXES(x1s, y1s, x2s, y2s, farbe [, anzahl])` | many rectangles |
| `CIRCLES(xs, ys, rs, farbe [, anzahl])` | many circles |
| `LINES(x1s, y1s, x2s, y2s, farbe [, anzahl])` | many lines |

**Leaving out `anzahl` means “the whole array”** — so a buffer with a fixed
size drags its unused slots into the picture. Whoever has only filled the
first `n` entries must pass `n`.

## Generating textures

Images without an image file — for backgrounds, lights and patterns you do
not want to ship. All of them return an IMAGE that can be drawn like a
loaded one.

| Function | Purpose |
|---|---|
| `GENTEX_COLOR(breite, hoehe, farbe)` | single-coloured area |
| `GENTEX_GRADIENT(breite, hoehe, farbe1, farbe2 [, vertikal])` | linear gradient |
| `GENTEX_GRADIENT_BOX(breite, hoehe, dichte, farbe1, farbe2)` | rectangular gradient from the inside out — vignettes |
| `GENTEX_RADIAL(breite, hoehe, innen, aussen [, dichte])` | round gradient from the centre outwards — soft lights and glows, drawn additively |
| `GENTEX_CHECKED(breite, hoehe, feld_x, feld_y, farbe1, farbe2)` | checkerboard |
| `GENTEX_PERLIN(breite, hoehe, skala)` | Perlin noise — clouds, terrain, marble |
| `GENTEX_CELLULAR(breite, hoehe, kachel)` | cellular noise (Voronoi) — stone floors, cracks, scales |
| `GENTEX_NOISE(breite, hoehe, anteil)` | white noise — star fields, film grain |

## Render targets

A render target is an off-screen surface you draw on just like on the window
— for mirrors, minimaps, picture-in-picture or trail effects.

| Function | Purpose |
|---|---|
| `RENDERTARGET_NEW(breite, hoehe [, behalten])` → INTEGER | create a target. `behalten = TRUE` keeps the contents beyond the frame — the precondition for trails |
| `RENDERTARGET_BEGIN(ziel)` | draw there from now on |
| `RENDERTARGET_END()` | back to the window |
| `RENDERTARGET_DRAW(ziel, x, y [, skala [, tint [, gespiegelt]]])` | stamp the target onto the window like an image |
| `RENDERTARGET_CLEAR(ziel [, farbe])` | clear by hand — needed when `behalten` is on |

Without `behalten`, the target is cleared to transparent at the start of
every frame. A trail, by contrast, comes about like this: `behalten = TRUE`,
and each frame a `BLEND_MODE("mult")` with a dark grey fullscreen `BOX` over
it — that makes old content fade instead of deleting it.

A target **cannot draw itself**; a `RENDERTARGET_DRAW` inside another target
does nothing.

## Shaders and post-processing

| Function | Purpose |
|---|---|
| `SHADER_LOAD(pfad_oder_glsl$)` → INTEGER | load a fragment shader — from a file or directly as source text; `-1` on error |
| `POSTFX(shader)` | send every finished frame through this shader; `-1` switches it off |
| `SHADER_SET(shader, name$, wert)` | pass a number to a `uniform float` |
| `SHADER_SET2(shader, name$, x, y)` | two values (`vec2`) |
| `SHADER_SET3(shader, name$, x, y, z)` | three values (`vec3`) |
| `SHADER_SET_ARRAY(shader, name$, werte)` | fill a `uniform float[]` from an `ARRAY OF FLOAT` — light positions, gradient stops |
| `SHADER_SET_MATRIX(shader, name$, mat)` | pass a `MAT4` from [`m3d`](module-m3d.md) |
| `SHADER_SET_TEXTURE(shader, name$, bild)` | occupy a **second** sampler — masks, palettes, cross-fades |

`SHADER_SET_TEXTURE` is remembered and only set when drawing. The reason is
a quirk of raylib: the assignment affects the *currently active* shader
program, so outside of drawing it would land on the wrong one — and the
sampler would stay black.

## Input: keyboard and mouse

| Function | Purpose |
|---|---|
| `KEYPRESSED(code)` → BOOLEAN | TRUE **as long as** the key with SDL code `code` is held — every frame again. For moving/steering. |
| `KEYHIT(code)` → BOOLEAN | TRUE only in **the one frame** in which the key goes down. For shooting, jumping, toggling. |
| | *Letters:* `KEY_A`…`KEY_Z` are the clear spelling. `ASC("s")` and `ASC("S")` both mean the same key — until 2026-08-31 the **upper-case** spelling silently hit nothing at all, which looked like a forgotten call. |
| | *Position or label:* a key code means the **position** of the key on the US keyboard — `KEY_Z` is the key labelled Y on a German keyboard, the one labelled W on a French one. That is right for WASD. For shortcuts like Ctrl+Z write the **letter as text**: `KEYHIT("z")` means the key the Z is printed on (also for `KEYPRESSED`, `KEYRELEASED`, `KEYREPEAT`; letters, digits and the space). |
| `KEYRELEASED(code)` → BOOLEAN | TRUE in the frame in which the key is released — for “charge up and fire on release”, say |
| `KEYREPEAT(code)` → BOOLEAN | like `KEYHIT`, but while held it also fires with the system key repeat. For text cursors and quantity input. |
| `MOUSEX()`, `MOUSEY()` → INTEGER | current mouse position (in logical pixels) |
| `MOUSEBUTTON(n)` → BOOLEAN | TRUE if mouse button n is pressed — **`0`=left, `1`=right, `2`=middle** (raylib order: right before middle!) |
| `MOUSEWHEEL()` → INTEGER | mouse wheel delta since the last call (+ up / − down / 0) |
| `MOUSE_VISIBLE(an)` | show/hide the OS cursor — hide it when the game draws its own crosshair/cursor sprite |
| `MOUSE_LOCK(an)` | **capture** the cursor: hide it + lock it into the window (relative movement) — for first-person/camera mouse control; `FALSE` releases it |
| `MOUSE_HIDDEN()` → BOOLEAN | is the cursor hidden/captured right now? |
| `SCREENWIDTH()`, `SCREENHEIGHT()` → INTEGER | logical window size (as passed to `SCREEN`); 0 before `SCREEN` |
| `MOUSE_HIT(n)` → BOOLEAN | TRUE only in **the one frame** in which the mouse button goes down |
| `MOUSE_RELEASED(n)` → BOOLEAN | TRUE in the frame in which it is released |
| `MOUSE_DELTA_X()`, `MOUSE_DELTA_Y()` → FLOAT | how far the mouse has moved since the last frame — with `MOUSE_LOCK`, `MOUSEX`/`MOUSEY` stand still and only this still moves |
| `MOUSEWHEEL_X()`, `MOUSEWHEEL_Y()` → FLOAT | wheel in **both** axes and as a decimal number; `MOUSEWHEEL()` only knows the vertical one and rounds — fine touchpad steps drop to 0 there |
| `MOUSE_SET_POS(x, y)` | put the pointer at a position |
| `MOUSE_OFFSET(x, y)` | offset added to the mouse position before `MOUSE_SCALE` applies — when the picture sits offset in the window |
| `MOUSE_SCALE(sx, sy)` | scale for the mouse position: `MOUSEX()` = (raw + offset) · sx — when a small picture is shown enlarged; 1 = as before |
| `MOUSE_ON_SCREEN()` → BOOLEAN | is the pointer in the window at all? |
| `MOUSE_CURSOR(form$)` | pointer shape: `default`, `ibeam`, `crosshair`, `hand`, `resize_ew`, `resize_ns`, `resize_nwse`, `resize_nesw`, `resize_all`, `not_allowed`, plus `warten` (wait), `arbeitet` (busy), `hilfe` (help) (under Windows the real system pointers, the hourglass spins), `kopieren` (copy), `stift` (pen), `pipette` (eyedropper) and your own from `MOUSE_CURSOR_NEW`; stays in place and takes precedence over the gui, `auto` hands the pointer back to the gui |
| `MOUSE_CURSOR_GET$()` | the pointer shape currently in effect (the program's, otherwise the gui's, otherwise `default`) |
| `MOUSE_CURSOR_NEW(bild, bx, by)` | a pointer from an image (up to 256x256, hot spot `bx`,`by`); returns the name (`eigen1` ...) which then works for `MOUSE_CURSOR` and `GUI_SET_CURSOR`. The image is copied |
| `KEY_ANY_HIT()` → INTEGER | code of the last key pressed, `-1` = none — for key-binding dialogs |
| `KEY_NAME$(code)` → STRING | display name of a key (`LEER`, `LINKS`, `UMSCHALT`, `F5` …) |
| `INKEY$()` → STRING | last typed character or `""` — does **not** wait, for text input during gameplay |
| `WAITKEY()` → INTEGER | **stops** until a key arrives and returns its code (`-1` if the window is closed) |

**A click between two frames is not lost.** If press and release arrive
before the next frame begins (a tap on the touchpad, a played-back click, a
program that is running slowly right now), the button counts as pressed for
one frame: `MOUSE_HIT` and `MOUSEBUTTON` in one frame, `MOUSE_RELEASED` in the
next, and a gui button fires. Before, nobody saw such a click — raylib reads
the buttons only once per frame, as a state. The same applies to the
keyboard (`KEYHIT`/`KEYPRESSED` in one frame, `KEYRELEASED` in the next);
typed characters (`INKEY$`, text inputs) were never affected.

**Key constants** (`KEY_*`) are built in:

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

Available constants:

| Group | Constants |
|---|---|
| Special keys | `KEY_ESCAPE`, `KEY_RETURN`/`KEY_ENTER`, `KEY_SPACE`, `KEY_TAB`, `KEY_BACKSPACE` |
| Arrows | `KEY_LEFT`, `KEY_RIGHT`, `KEY_UP`, `KEY_DOWN` |
| Letters/digits | `KEY_A` to `KEY_Z`, `KEY_0` to `KEY_9` |
| Function keys | `KEY_F1` to `KEY_F12` |
| Modifiers | `KEY_LSHIFT`, `KEY_RSHIFT`, `KEY_LCTRL`, `KEY_RCTRL`, `KEY_LALT`, `KEY_RALT`, `KEY_LSUPER`, `KEY_RSUPER`, `KEY_CAPSLOCK` |
| Navigation | `KEY_INSERT`, `KEY_DELETE`, `KEY_HOME`, `KEY_END`, `KEY_PAGEUP`, `KEY_PAGEDOWN` |
| Numeric keypad | `KEY_KP0` to `KEY_KP9`, `KEY_KP_ENTER`, `KEY_KP_PLUS`, `KEY_KP_MINUS`, `KEY_KP_MULTIPLY`, `KEY_KP_DIVIDE`, `KEY_KP_PERIOD` |

The numeric keypad has codes of its own — a game control may bind it
separately from the top row of digits. Gamepad codes (`JOY_BUTTON_A`,
`JOY_DPAD_UP` and relatives) are in [module-input.md](module-input.md).


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

Up to four devices, numbered from 0. As with the keyboard there is “being
held” and “pressed right now”.

| Function | Purpose |
|---|---|
| `JOYSTICK_COUNT()` → INTEGER | how many devices are connected? |
| `JOYSTICK_NAME(idx)` → STRING | name of the device, as it reports itself |
| `JOYSTICK_BUTTON(idx, btn)` → BOOLEAN | is the button being **held**? |
| `JOYSTICK_HIT(idx, btn)` → BOOLEAN | pressed in exactly this frame |
| `JOYSTICK_RELEASED(idx, btn)` → BOOLEAN | released in exactly this frame |
| `JOYSTICK_ANY_BUTTON()` → INTEGER | last button pressed, `-1` = none — for key-binding dialogs |
| `JOYSTICK_AXIS(idx, achse)` → FLOAT | position of an analogue axis, `-1.0` to `+1.0` |
| `JOYSTICK_AXIS_COUNT(idx)` → INTEGER | how many axes does the device have? |
| `JOYSTICK_HAT_X(idx, hat)` / `JOYSTICK_HAT_Y(idx, hat)` → INTEGER | D-pad as `-1`, `0` or `+1` per axis |
| `JOYSTICK_RUMBLE(idx, links, rechts, dauer_s)` | vibration; the two motors separately (each `0.0`–`1.0`) |
| `JOYSTICK_MAPPINGS(sdl_db$)` → INTEGER | load mappings from the SDL GameControllerDB — this makes exotic devices map correctly |

There are constants for the buttons (`JOY_BUTTON_A` … `JOY_BUTTON_Y`,
`JOY_DPAD_*`). Whoever prefers working with action names instead of numbers
uses the module [`input`](module-input.md) — there it reads
`INPUT_BIND("springen", KEY_SPACE, JOY_BUTTON_A)`.

Analogue axes rarely return exactly 0. A small dead zone keeps the character
from wandering off on its own:

```basic
DIM ax AS FLOAT
ax = JOYSTICK_AXIS(0, 0)
IF ABS(ax) < 0.15 THEN ax = 0.0
```

## Touch and gestures

| Function | Purpose |
|---|---|
| `TOUCH_COUNT()` → INTEGER | how many fingers are touching? |
| `TOUCH_X(i)` / `TOUCH_Y(i)` → FLOAT | position of the `i`-th finger |
| `TOUCH_ID(i)` → INTEGER | identifier of this finger — stays the same across frames as long as it stays down |
| `GESTURE$()` → STRING | recognized gesture: `tap`, `doubletap`, `hold`, `drag`, `swipe_left`, `swipe_right`, `swipe_up`, `swipe_down`, `pinch_in`, `pinch_out` — `""` if none |
| `GESTURE_DRAG_X()` / `GESTURE_DRAG_Y()` → FLOAT | direction of a drag |
| `GESTURE_DRAG_ANGLE()` → FLOAT | its angle |
| `GESTURE_PINCH_X()` / `GESTURE_PINCH_Y()` → FLOAT | distance when pinching open and closed |
| `GESTURE_PINCH_ANGLE()` → FLOAT | its angle |
| `GESTURE_HOLD_TIME()` → FLOAT | how long it has been held (seconds) |

`TOUCH_ID` is the difference between “two fingers” and “which finger”: when
dragging with several, the index `i` shifts as soon as one lifts — the
identifier stays. Demo: [examples/149_input_edges.dh](../../examples/149_input_edges.dh).
