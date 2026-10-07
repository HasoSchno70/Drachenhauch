# Module `ui`

Immediate-mode UI: label, button, checkbox, slider, progress bar, panel, text field, radio group. Components are called anew every frame — no separate setup, no component objects. State (checkbox state, slider value, text field contents, radio selection) is managed internally via string IDs.

```basic
IMPORT "ui"
```

## Overview

| Function | Returns | Purpose |
|---|---|---|
| `UI_LABEL(x, y, text$[, color])` | — | draw text |
| `UI_BUTTON(id$, x, y, w, h, text$[, bg, fg])` | BOOLEAN | TRUE if clicked in this frame |
| `UI_CHECKBOX(id$, x, y, label$[, default])` | BOOLEAN | current toggle state |
| `UI_SLIDER(id$, x, y, w, min, max[, default])` | FLOAT | current value |
| `UI_PROGRESS(x, y, w, h, value, max[, fg, bg])` | — | read-only progress bar |
| `UI_PANEL(x, y, w, h[, title$[, bg]])` | — | container with an optional title |
| `UI_TEXTFIELD(id$, x, y, w, h[, placeholder$])` | STRING | current input text |
| `UI_TEXTFIELD_SET(id$, value$)` | — | set the value programmatically |
| `UI_RADIO(id$, x, y, options[, default_idx])` | INTEGER | selected index |
| `UI_TABLE(id$, x, y, w, h, headers, cells[, cell_colors[, col_widths[, cell_bg_colors]]])` | INTEGER | index of the clicked row (-1 if none) |
| `UI_TABLE_SELECTED(id$)` | INTEGER | persistently selected row (-1 if none) |
| `UI_TABLE_SET_SELECTED(id$, row)` | — | set the selection programmatically (-1 = none) |
| `UI_TABLE_HEADER_CLICK(id$)` | INTEGER | header column clicked in this frame (-1) — sorting hook |
| `UI_WINDOW_BEGIN(id$, titel$, x, y, w, h)` | BOOLEAN | movable window; FALSE if collapsed |
| `UI_WINDOW_END()` | — | close the window (always pair them) |
| `UI_END_FRAME()` | — | **mandatory** at the end of every frame before `FLIP()` |
| `UI_RESET()` | — | clear all UI state + reset theme/metrics |
| `UI_THEME_SET(key$, farbe)` / `UI_THEME_GET(key$)` | — / INT | set/read a single theme colour |
| `UI_THEME_PRESET(name$)` | — | colour scheme: dark/light/retro/contrast |
| `UI_METRIC_SET(key$, wert)` / `UI_METRIC_GET(key$)` | — / INT | set/read a layout size |

## Concept: immediate mode

Components are called anew every frame. That is typical for BASIC and very readable — no "register components / bind events / draw components" pattern.

```basic
IMPORT "ui"

SCREEN(320, 240, "UI-Demo", 2)

WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 25, 40))

    UI_LABEL(10, 10, "Mein Spiel", RGB(255, 220, 80))

    IF UI_BUTTON("start", 10, 40, 100, 30, "Start") THEN
        PRINT "Spiel startet!"
    END IF

    DIM ton_an AS BOOLEAN
    ton_an = UI_CHECKBOX("snd", 10, 80, "Sound an", TRUE)

    DIM lautstaerke AS FLOAT
    lautstaerke = UI_SLIDER("vol", 10, 110, 200, 0.0, 1.0, 0.7)

    UI_END_FRAME()              ' important!
    FLIP()
    SLEEP(16)
WEND
```

## ID system

Every stateful component (`UI_BUTTON`, `UI_CHECKBOX`, `UI_SLIDER`) needs a **unique string ID**. The UI module remembers the state per ID internally between frames.

```basic
UI_CHECKBOX("musik", 10, 50, "Musik")     ' ID = "musik"
UI_CHECKBOX("sound", 10, 70, "Sound")     ' ID = "sound" - a different checkbox
```

If you use the same ID twice per frame, there is **one** component with the value that won last (pointless). IDs must be unique.

`UI_LABEL` doesn't need an ID because it is stateless.

## UI_LABEL

```basic
UI_LABEL(x, y, text$[, color])
```

Draws text at `(x, y)`. The default colour is white. Functionally the same as `TEXT(x, y, text, color)`, but in UI style.

```basic
UI_LABEL(10, 10, "Score: " + STR$(score))
UI_LABEL(10, 30, "HP: " + STR$(hp), RGB(255, 100, 100))
```

## UI_BUTTON

```basic
UI_BUTTON(id$, x, y, w, h, text$[, bg, fg]) -> BOOLEAN
```

Draws a clickable button. Returns **TRUE** in the frame in which the user **released** the mouse over the button (after having pressed it before). Press-and-drag-away-and-release doesn't count — typical OK-button behaviour.

```basic
IF UI_BUTTON("ok", 10, 100, 80, 28, "OK") THEN
    PRINT "OK geklickt"
END IF

' With custom colours:
IF UI_BUTTON("danger", 100, 100, 80, 28, "Loeschen", RGB(160, 40, 40)) THEN
    daten_loeschen()
END IF
```

**Visuals:**
- Normal: bg colour
- Hover (mouse over it): brightened
- Press (mouse over it + pressed): darkened

## UI_CHECKBOX

```basic
UI_CHECKBOX(id$, x, y, label$[, default]) -> BOOLEAN
```

A toggle checkbox with the label to its right. The `default` value is set **only on the first call** of this ID — after that the state toggled by the user is kept.

```basic
DIM ton_an AS BOOLEAN
ton_an = UI_CHECKBOX("snd", 10, 50, "Sound", TRUE)        ' default in the 1st frame: TRUE

' Per frame: current value
IF ton_an THEN
    musik_starten()
END IF
```

Clicking the checkbox toggles it (on press, not release — no drag trick like with the button).

## UI_SLIDER

```basic
UI_SLIDER(id$, x, y, w, min, max[, default]) -> FLOAT
```

A horizontal value slider. `min` and `max` define the value range (FLOAT). The default lands within the range (it is clamped).

```basic
DIM lautstaerke AS FLOAT
lautstaerke = UI_SLIDER("vol", 10, 100, 200, 0.0, 1.0, 0.7)

DIM bg_helligkeit AS INTEGER
bg_helligkeit = INT(UI_SLIDER("bg", 10, 130, 200, 0, 100, 50))
```

The user clicks/drags in the slider area. The value changes as long as the mouse is pressed AND over the slider.

**Constraints:** `max > min` (otherwise an error on the call).

## UI_PROGRESS

```basic
UI_PROGRESS(x, y, w, h, value, max[, fg, bg])
```

A read-only progress bar — no ID, no state. Good for HP bars, loading displays, XP progress.

```basic
UI_PROGRESS(10, 50, 150, 12, hp, 100, RGB(220, 60, 60), RGB(60, 30, 30))
```

`value` is clamped to `[0, max]`. `max` must be `> 0` (otherwise an error). The default colours are green on dark grey.

## UI_PANEL

```basic
UI_PANEL(x, y, w, h[, title$[, bg]])
```

A visual container with a frame. If `title` is set, an 18 px high title bar is added at the top. Completely stateless — the game draws its components over the panel itself.

```basic
UI_PANEL(10, 10, 200, 130, "Held")
UI_LABEL(20, 35, "HP", RGB(220, 100, 100))
UI_PROGRESS(50, 36, 150, 12, hp, 100)
```

## UI_TEXTFIELD

```basic
UI_TEXTFIELD(id$, x, y, w, h[, placeholder$]) -> STRING
```

Text input. A **click** on the field sets the focus; **typed characters** are appended directly; **Backspace** deletes the last character; a **click outside** removes the focus. The returned STRING is the current contents.

```basic
DIM name AS STRING
name = UI_TEXTFIELD("name", 10, 50, 200, 26, "Hier tippen ...")
UI_LABEL(10, 90, "Hallo, " + name + "!")
```

`placeholder$` is shown in a muted colour as long as the field is empty AND doesn't have the focus.

A blinking cursor appears only in the focused field. Layout/Shift/dead keys are resolved by the OS — umlauts (`ä`, `ü`, …) and special characters work as in any normal input field.

```basic
' Pre-fill a value (e.g. when editing an entry):
UI_TEXTFIELD_SET("name", "Anna")
```

Tab/Enter are not handled by the field — you can poll `KEYPRESSED(13)` yourself if you want Enter as "submit".

## UI_RADIO

```basic
UI_RADIO(id$, x, y, options[, default_idx]) -> INTEGER
```

A vertical radio group — exactly one option is selected. `options` must be an `ARRAY OF STRING`. Clicking a row selects it. Returns the index (0-based) of the selected option, or `-1` for an empty array.

```basic
DIM diff_options AS ARRAY OF STRING
diff_options = SPLIT$("Leicht|Mittel|Schwer", "|")

DIM diff AS INTEGER
diff = UI_RADIO("diff", 10, 50, diff_options, 1)    ' default: "Mittel"

UI_LABEL(10, 130, "Gewaehlt: " + diff_options[diff])
```

The row height is 18 px, and the click area covers the whole row (200 px wide) so that long labels are clickable too.

## UI_TABLE

```basic
UI_TABLE(id$, x, y, w, h, headers, cells[, cell_colors[, col_widths[, cell_bg_colors]]]) -> INTEGER
```

A table with a fixed header, a scrollable body and optional per-cell colouring (foreground and background). Supports vertical mouse wheel scrolling and draggable scroll bars on both axes.

**Parameters:**

| Param | Type | Remark |
|---|---|---|
| `headers` | `ARRAY OF STRING` | column titles; `LEN(headers)` defines the number of columns |
| `cells` | 2D `ARRAY OF STRING` `[rows, cols]` | cell contents; the column count must match `headers` |
| `cell_colors` | 2D `ARRAY OF INTEGER` `[rows, cols]` *optional* | text RGB per cell; default white |
| `col_widths` | `ARRAY OF INTEGER` *optional* | pixel width per column; default = evenly distributed |
| `cell_bg_colors` | 2D `ARRAY OF INTEGER` `[rows, cols]` *optional* | background RGB per cell. **Value -1 = draw no background** (the default behaviour without a background override) |

**Returns:** the index of the row that was clicked in **this frame** (press + release on the same row, like the `UI_BUTTON` logic). `-1` if no row was clicked.

**Behaviour:**
- The header row (22 px) is fixed at the top — it doesn't scroll
- The body scrolls vertically and/or horizontally depending on the contents
- A scroll bar appears automatically when the contents are larger than the visible area
- The mouse wheel scrolls vertically (always)
- Dragging the scroll bar works on both axes
- Hover highlight on the row under the mouse pointer
- Cell text is automatically cut to the column width (clean pixel clipping, no "…")

**Example — a high score list with coloured HP values:**

```basic
IMPORT "ui"
SCREEN(560, 380, "Highscore", 2)

CONST ROWS AS INTEGER = 10
CONST COLS AS INTEGER = 3

DIM headers AS ARRAY OF STRING
headers = SPLIT$("Name|HP|Punkte", "|")

DIM cells[ROWS, COLS] AS STRING
DIM colors[ROWS, COLS] AS INTEGER
DIM widths[COLS] AS INTEGER
widths[0] = 140
widths[1] = 80
widths[2] = 100

' Build data + colours
DIM r AS INTEGER
FOR r = 0 TO ROWS - 1
    cells[r, 0] = "Spieler_" + STR$(r + 1)
    DIM hp AS INTEGER
    hp = 30 + (r * 17) MOD 70
    cells[r, 1] = STR$(hp) + "/100"
    cells[r, 2] = STR$(((r + 1) * 313) MOD 9999)

    colors[r, 0] = RGB(220, 220, 230)
    IF hp >= 70 THEN colors[r, 1] = RGB(100, 220, 100)
    IF hp >= 40 AND hp < 70 THEN colors[r, 1] = RGB(255, 220, 80)
    IF hp < 40 THEN colors[r, 1] = RGB(240, 80, 80)
    colors[r, 2] = RGB(220, 220, 230)
NEXT

DIM selected AS INTEGER
selected = -1

WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(27) THEN BREAK
    CLS(RGB(15, 18, 32))

    DIM clicked AS INTEGER
    clicked = UI_TABLE("scores", 10, 10, 340, 300, headers, cells, colors, widths)
    IF clicked >= 0 THEN selected = clicked

    IF selected >= 0 THEN
        UI_LABEL(10, 320, "Ausgewaehlt: " + cells[selected, 0])
    END IF

    UI_END_FRAME()
    FLIP()
    SLEEP(16)
WEND
```

**Tips:**
- **Per-cell background** is ideal for statistics traffic lights (HP green/yellow/red with a solid bg colour), zebra stripes (even rows `RGB(30, 34, 52)`, odd ones `-1`) or heatmaps. On the hovered row the hover highlight overlays the cell bgs — the user still sees clearly where the mouse pointer is.
- For **dynamic updates** (the list grows): declare the arrays with a maximum size and fill non-existent rows with empty strings.
- **Clicking on the scroll bar** only scrolls — the row underneath is not accidentally "clicked", because the scroll bar as a click target blocks the click detection.
- **Complete example** with per-cell colours (gold/silver/bronze, class colours, HP traffic light) and a detail panel: [examples/43_ui_table.dh](../../examples/43_ui_table.dh).

### Persistent selection

```basic
UI_TABLE_SELECTED(id$)         -> INTEGER   ' selected row, -1 if none
UI_TABLE_SET_SELECTED(id$, row)             ' set programmatically, -1 = none
```

`UI_TABLE` automatically sets the selection to the row that is clicked in a frame — unlike the return value (`>= 0` only in the click frame) it **persists** across frames and is drawn as a second highlight (below the hover). That way the game no longer has to keep track of the selection in a variable itself:

```basic
UI_TABLE("scores", 10, 10, 340, 300, headers, cells)
DIM sel AS INTEGER
sel = UI_TABLE_SELECTED("scores")
IF sel >= 0 THEN UI_LABEL(10, 320, "Ausgewaehlt: " + cells[sel, 0])
```

If the data shrinks (fewer rows than the selected index), the selection automatically falls back to `-1`. `UI_TABLE_SET_SELECTED` only takes effect after the table has been drawn once with `UI_TABLE` (the `id$` must be known).

### Clickable headers / sorting

```basic
UI_TABLE_HEADER_CLICK(id$)     -> INTEGER   ' clicked header column, -1 if none
```

Returns the column whose **header** was clicked in this frame (press + release on the same column). Since `UI_TABLE` is immediate mode and the data belongs to the game, the table does **not** sort by itself — it only reports the click; the game sorts its arrays and passes them in again in the next frame:

```basic
DIM col AS INTEGER
col = UI_TABLE_HEADER_CLICK("scores")
IF col >= 0 THEN
    sort_dir = IIF(sort_col = col, -sort_dir, 1)   ' toggle up/down
    sort_col = col
    sortiere_daten(cells, col, sort_dir)            ' your own sorting routine
END IF
```

Query it directly **after** the `UI_TABLE` call in the same frame.

**Complete example** with selection, clickable headers (ascending/descending) and a detail panel: [examples/81_table_select.dh](../../examples/81_table_select.dh).

## Immediate-mode windows (UI_WINDOW_BEGIN / UI_WINDOW_END)

Movable, collapsible windows in immediate mode. Widgets drawn between
`UI_WINDOW_BEGIN` and `UI_WINDOW_END` are **window-relative**
(their `x`/`y` count from the top-left corner of the window contents,
below the title bar) and automatically move along when the window is dragged.

```basic
IMPORT "ui"
SCREEN(480, 360, "Fenster", 2)

WHILE NOT QUITREQUESTED()
    CLS(RGB(16, 20, 36))

    IF UI_WINDOW_BEGIN("settings", "Einstellungen", 120, 80, 220, 140) THEN
        IF UI_BUTTON("ok", 20, 90, 90, 28, "OK") THEN PRINT "OK"
        DIM snd AS BOOLEAN
        snd = UI_CHECKBOX("snd", 20, 30, "Sound", TRUE)
        DIM vol AS FLOAT
        vol = UI_SLIDER("vol", 20, 55, 170, 0.0, 1.0, 0.7)
    END IF
    UI_WINDOW_END()

    UI_END_FRAME()
    FLIP()
    SLEEP(16)
WEND
```

- **`UI_WINDOW_BEGIN(id$, titel$, x, y, w, h)`** → BOOLEAN. Returns **TRUE**
  if the window is open (draw the body), **FALSE** if it is collapsed — then
  skip the body via `IF ... THEN ... END IF`. `x, y` are the
  starting position; from the first frame on, the window remembers its (possibly
  dragged) position via the `id$`.
- **`UI_WINDOW_END()`** closes the current window. **Always** call it —
  even if `UI_WINDOW_BEGIN` returned FALSE (i.e. outside the `IF`).
- **Moving**: drag the title bar. **Collapsing**: click the `-`/`+` arrow
  on the left of the title bar.
- **Z-order / input**: windows are drawn in call order (called later
  = visually on top); mouse input goes to the topmost window under the
  cursor (hit test from the previous frame, as in Dear ImGui). A real
  "click brings to front" (reordering) doesn't exist in immediate mode —
  use the retained-mode module [`gui`](module-gui.md) for that.
- Widgets outside windows behave unchanged (offset 0).

## UI_END_FRAME

```basic
UI_END_FRAME()
```

**Mandatory** at the end of every frame, **before** `FLIP()`. It stores:
- the mouse state (for click edge detection in buttons/checkboxes/radios/tables)
- a keyboard snapshot (for the Backspace edge in the text field)
- the frame counter (for the blinking cursor)

Without this call, held mouse clicks count as continuous clicking (buttons fire every frame, checkboxes toggle endlessly) and a held Backspace deletes all characters at once.

## UI_RESET

```basic
UI_RESET()
```

Resets all UI state: checkbox values, slider values, click state. Useful on:
- game restart (settings back to default)
- switching menu screens (old UI IDs should be forgotten)

```basic
SUB neues_spiel()
    UI_RESET()
    state = "playing"
END SUB
```

## Complete example

A settings dialog with a volume slider, sound/music checkboxes, three difficulty buttons and a quit button:

```basic
IMPORT "ui"

CONST W AS INTEGER = 320
CONST H AS INTEGER = 240
SCREEN(W, H, "UI-Demo", 2)

DIM beendet AS BOOLEAN
beendet = FALSE

WHILE NOT QUITREQUESTED() AND NOT beendet
    IF KEYPRESSED(KEY_ESCAPE) THEN
        BREAK
    END IF

    CLS(RGB(20, 25, 40))

    UI_LABEL(10, 10, "Einstellungen", RGB(255, 220, 80))

    UI_LABEL(10, 50, "Lautstaerke:")
    DIM vol AS FLOAT
    vol = UI_SLIDER("vol", 110, 50, 180, 0.0, 1.0, 0.7)

    DIM ton_an AS BOOLEAN
    ton_an = UI_CHECKBOX("snd", 10, 100, "Sound an", TRUE)
    DIM musik AS BOOLEAN
    musik = UI_CHECKBOX("mus", 10, 120, "Musik an", FALSE)

    UI_LABEL(10, 150, "Schwierigkeit:")
    IF UI_BUTTON("easy", 110, 145, 60, 24, "Leicht") THEN
        PRINT "Leicht gewaehlt"
    END IF
    IF UI_BUTTON("hard", 175, 145, 60, 24, "Hart") THEN
        PRINT "Hart gewaehlt"
    END IF

    IF UI_BUTTON("quit", 240, 200, 70, 24, "Beenden", RGB(120, 40, 40)) THEN
        beendet = TRUE
    END IF

    UI_END_FRAME()
    FLIP()
    SLEEP(16)
WEND
```

See also [examples/33_ui.dh](../../examples/33_ui.dh) — a complete runnable demo.

## Tips

- **Always call `UI_END_FRAME()`** — otherwise click edge detection breaks.
- **Choose descriptive IDs** — not `"a"`, `"b"`, but `"musik"`, `"vol"`, `"start"`. That helps with debugging.
- **Condition on the button call**: `IF UI_BUTTON(...) THEN ... END IF` is the idiomatic form. The button is **always** drawn — the `IF` condition only picks up the click evaluation.
- **Layout via variables**: with many components you accumulate a `y_offset` and work with constants:
  ```basic
  DIM y AS INTEGER
  y = 50
  UI_LABEL(10, y, "...")
  y = y + 30
  UI_BUTTON("a", 10, y, 80, 24, "A")
  y = y + 30
  UI_BUTTON("b", 10, y, 80, 24, "B")
  ```
- **Scroll trick**: for lists with many buttons you can add a `scroll_y` variable to all `y` values — a cheap scroll list.

## Changing the look (theme & metrics)

The default colours of all components can be changed globally — either with a ready-made scheme or per individual colour. (Colours you pass directly as an argument, e.g. `UI_BUTTON(..., bg, fg)`, still take precedence.)

```basic
UI_THEME_PRESET("dark")       ' default
UI_THEME_PRESET("light")      ' light scheme
UI_THEME_PRESET("retro")      ' green on black
UI_THEME_PRESET("contrast")   ' black/yellow

UI_THEME_SET("button_bg", RGB(60, 40, 90))   ' one colour, globally
DIM c AS INTEGER
c = UI_THEME_GET("accent")
```

Theme keys: `accent`, `text_fg`, `muted_fg`, `button_bg`, `panel_bg`, `panel_border`, `panel_title_bg`, `field_bg`, `field_border`, `slider_track`, `progress_fg`, `progress_bg`, `win_bg`, `win_border`, `win_title_bg`, `win_title_bg_focus`.

Sizes via metrics:

```basic
UI_METRIC_SET("checkbox_size", 20)   ' larger checkbox
UI_METRIC_SET("slider_h", 18)
UI_METRIC_SET("win_title_h", 24)     ' title bar of the UI_WINDOW_BEGIN windows
```

`UI_RESET()` also resets the theme + metrics to the defaults.

## Limitations (current state)

- **No dropdown / list box** — so far easily substituted with a radio group + panel or a row of buttons.
- **Layout is absolute** — no automatic layouts like "horizontal/vertical stack". You place the pixels yourself.
- **Fixed font** — the standard sans-serif of the native runtime. You can change the size globally with `TEXT_SIZE(n)` — but that affects all subsequent TEXT/UI calls.
- **No multi-line text field** — `UI_TEXTFIELD` is single-line. Enter is not handled; you can use it yourself via `KEYPRESSED(13)` as a submit trigger.

If you need one of these features, it is easy to add in a later iteration.

See also [examples/42_ui_extended.dh](../../examples/42_ui_extended.dh) for a demo of all the new components combined (HP bars, a settings panel with text field + radio + slider).
