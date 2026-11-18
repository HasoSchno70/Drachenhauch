# Command names

Drachenhauch has more than 2500 built-in commands, and they grew over the
years: `LOADIMAGE` next to `IMAGE_SAVE`, `KEYPRESSED` (held) next to
`INPUT_PRESSED` (just pressed), `ZEIT_JETZT` in the middle of English names.
This page sets out how commands are named and gives the **new name** for
the old ones.

**The old names stay valid** — no program, example or book has to change.
New programs use the new ones. In the editor, the hover over an old name
shows the new one; completion and signature help know both.

## Contents

- [The rules](#the-rules)
- [The recommended way per area](#the-recommended-way-per-area)
- [New names for old commands](#new-names-for-old-commands)

## The rules

1. **Split by underscores, the area first:** `AREA_THING_ACTION` —
   `IMAGE_LOAD`, `FILE_EXISTS`, `GUI_LISTBOX_ITEMS`, `WINDOW_SET_TITLE`.
   Exempt are the basic words from BASIC: `PRINT`, `LEN`, `MID$`, `CLS`,
   `BOX`, `LINE`, `CIRCLE`, `SCREEN` and their relatives.
2. **Setting and reading:** `AREA_SET_PROPERTY` sets (the value comes
   last), `AREA_GET_PROPERTY` reads. A command that sets carries the `SET` in
   its name — `AUDIO_MUSIC_SET_PITCH`, not `AUDIO_MUSIC_PITCH(value)`.
3. **Input:** `_DOWN` means *held* (true every frame), `_HIT` means *went
   down in this frame*, `_RELEASED` *let go* — for keys (`KEY_DOWN`,
   `KEY_HIT`), mouse (`MOUSE_DOWN`, `MOUSE_HIT`), gamepad (`JOYSTICK_DOWN`,
   `JOYSTICK_HIT`) and actions (`INPUT_DOWN`, `INPUT_HIT`). The word
   `PRESSED` does not appear in new names: it meant "held" in `KEYPRESSED`
   and "just pressed" in `INPUT_PRESSED`.
4. **Words with a fixed meaning:** `_FREE` releases, `_COUNT` is a number of
   things (`MAP_COUNT`, `SET_COUNT`), `LEN` a length (text, bytes, array),
   `_LOAD`/`_SAVE` read and write files.
5. **The `$`:** a command that returns text may be written with or without
   `$` — `ENV_GET$("HOME")` and `ENV_GET("HOME")` are the same, and so are
   `CLIPBOARD_GET()` and `CLIPBOARD_GET$()`. Only where both forms exist as
   **different** commands does the spelling matter: `GUI_TOOLTIP(w, text$)`
   sets, `GUI_TOOLTIP$(w)` reads. A `$` on a command that does not return
   text (`LEN$`) stays an error.
6. **English:** command names are English. The modules `zeit` and `geld`
   can also be included as `IMPORT "time"` and `IMPORT "money"`. The type
   is still called `GELD` — a second type name would run all the way into
   the runtime and hide a class of your own called `Money`.
7. **New commands** follow these rules. A new name for an old command goes
   into `rust/drachenhauch_runtime/src/namen.rs`, into
   `daten/builtin_index.json` and here; the tests in `namen.rs` keep table
   and index together.
8. **Parameter names** are English as well, and they are interface:
   commands take named arguments — `CIRCLE(x: 10, y: 20, r: 5, color: red)`
   (see [Language](sprache.md#named-arguments)). Common short forms stay
   (`x`, `y`, `w`, `h`, `r`, `win`, `idx`), otherwise whole words: `color`,
   `path`, `text`, `width`, `count`, `encoding`. The same thing has the same
   name in every command. The command tables in the manual use the same
   names as `daten/builtin_index.json`; `dhrt pruef parameter` keeps them
   together.

## The recommended way per area

Some things can be done in several ways that grew over time. Recommended:

| Area | Recommended | Besides |
|---|---|---|
| Sound | `AUDIO_*` (buses, effects, spatial, music with `AUDIO_MUSIC_*`) | `SOUND_LOAD`/`SOUND_PLAY`, `MUSIC_PLAY` as the short form for a quick sound; `SAMPLE_*` for instruments |
| Input | `INPUT_BIND` and `INPUT_DOWN`/`INPUT_HIT` — keyboard and gamepad in one | `KEY_*`, `MOUSE_*`, `JOYSTICK_*` for raw queries |
| Clock | `MILLIS()` for timing inside the program, `TIME_NOW()` for date and time of day | `TIMER()` (the same clock in seconds), `GET_TIME()`, `DATE$()`, `TIME$()` |
| Files | `FILE_READ$`, `FILE_WRITE`, `FILE_READ_LINES` by path | `FILE_OPEN`/`FILE_READ_LINE$`/`FILE_CLOSE` with a handle for large files |

`TIMER()` is a clock; the family `TIMER_EVERY`, `TIMER_AFTER` … on the other
hand are callback timers from the `timer` module.

## New names for old commands

### held / just pressed / released: `_DOWN`, `_HIT`, `_RELEASED`

| Name | What it does |
|---|---|
| `KEY_DOWN(key: Code oder Buchstabe)` | TRUE as long as the key with SDL code code is held — every frame again. For moving/steering — formerly `KEYPRESSED` |
| `KEY_HIT(key: Code oder Buchstabe)` | TRUE only in the one frame in which the key goes down. For shooting, jumping, toggling — formerly `KEYHIT` |
| `KEY_RELEASED(key: Code oder Buchstabe)` | TRUE in the frame in which the key is released — for “charge up and fire on release”, say — formerly `KEYRELEASED` |
| `KEY_REPEAT(key: Code oder Buchstabe)` | like KEYHIT, but while held it also fires with the system key repeat. For text cursors and quantity input — formerly `KEYREPEAT` |
| `KEY_WAIT()` | stops until a key arrives and returns its code (-1 if the window is closed) — formerly `WAITKEY` |
| `MOUSE_DOWN(n: Zahl)` | TRUE if mouse button n is pressed — 0=left, 1=right, 2=middle (raylib order: right before middle!) — formerly `MOUSEBUTTON` |
| `MOUSE_X()` | current mouse position (in logical pixels) — formerly `MOUSEX` |
| `MOUSE_Y()` | current mouse position (in logical pixels) — formerly `MOUSEY` |
| `MOUSE_WHEEL()` | mouse wheel delta since the last call (+ up / − down / 0) — formerly `MOUSEWHEEL` |
| `MOUSE_WHEEL_X()` | wheel in both axes and as a decimal number; MOUSEWHEEL() only knows the vertical one and rounds — fine … — formerly `MOUSEWHEEL_X` |
| `MOUSE_WHEEL_Y()` | wheel in both axes and as a decimal number; MOUSEWHEEL() only knows the vertical one and rounds — fine … — formerly `MOUSEWHEEL_Y` |
| `MOUSE_SET_VISIBLE(on: BOOLEAN)` | show/hide the OS cursor — hide it when the game draws its own crosshair/cursor sprite — formerly `MOUSE_VISIBLE` |
| `JOYSTICK_DOWN(idx: Zahl, btn: Zahl)` | is the button being held? — formerly `JOYSTICK_BUTTON` |
| `INPUT_DOWN(action: STRING)` | BOOLEAN — currently pressed — formerly `INPUT_HELD` |
| `INPUT_HIT(action: STRING)` | BOOLEAN — edge: pressed RIGHT NOW — formerly `INPUT_PRESSED` |

### same name, different meaning

| Name | What it does |
|---|---|
| `GUI_LISTBOX_ITEMS(lb, items)` | replace the entries — formerly `GUI_SET_LISTBOX` |
| `GUI_DROPDOWN_ITEMS(dd, items)` | replace the entries — formerly `GUI_SET_DROPDOWN` |
| `GUI_DOCK_SET_SIZE(win, area$, px)` | width (left/right) or height (bottom) of an area — formerly `GUI_DOCK_SIZE` |
| `AUDIO_MUSIC_SET_PITCH(factor)` | music pitch (1.0 = normal; survives LOAD/QUEUE — slow-motion effect) — formerly `AUDIO_MUSIC_PITCH` |
| `SET_COUNT(set)` | number of elements — formerly `SET_SIZE` |

### gui: `GUI_SET_X` sets, `GUI_GET_X` reads

| Name | What it does |
|---|---|
| `GUI_GET_TEXT$(widget)` | STRING — text (label/button/TextInput) — formerly `GUI_TEXT` |
| `GUI_GET_VALUE(widget)` | FLOAT — slider value — formerly `GUI_VALUE` |
| `GUI_GET_CHECKED(widget)` | BOOLEAN — checkbox state — formerly `GUI_CHECKED` |
| `GUI_GET_ENABLED(widget)` | BOOLEAN — is it usable? — formerly `GUI_ENABLED` |
| `GUI_GET_VISIBLE(widget)` | show/hide (invisible = not drawn, not interactive) — formerly `GUI_VISIBLE` |
| `GUI_GET_ACTIVE_TAB(win)` | INTEGER — index of the active tab — formerly `GUI_ACTIVE_TAB` |
| `GUI_GET_DATE$(picker)` | chosen date as JJJJ-MM-TT — formerly `GUI_DATE` |
| `GUI_GET_TIME$(tp)` | STRING — the time as HH:MM:SS — like TIME$(), even without a seconds field — formerly `GUI_TIME$` |
| `GUI_GET_ERROR$(field)` | STRING / — / — read the message, set it from outside (e.g. from the database), clear all — formerly `GUI_ERROR` |
| `GUI_GET_PICKED_COLOR(picker)` | chosen colour (0xRRGGBB) — formerly `GUI_PICKED_COLOR` |
| `GUI_GET_MOTION()` | BOOLEAN — is the gui playing motion right now? — formerly `GUI_MOTION` |
| `GUI_GET_SOUNDS()` | all gui sounds on or off -- for a "sounds" switch in the settings — formerly `GUI_SOUNDS` |
| `GUI_SET_TOOLTIP(widget, text)` | hover help text (multi-line via \n; "" removes it) — formerly `GUI_TOOLTIP` |
| `GUI_GET_TOOLTIP$(widget)` | STRING — the help text that was set ("" if none) — formerly `GUI_TOOLTIP$` |
| `GUI_DOCK_SET_TITLE(win, name$, title$)` | relabel the tab — formerly `GUI_DOCK_TITLE` |
| `GUI_DOCK_GET_TITLE$(win, name$)` | STRING — the label of the tab — formerly `GUI_DOCK_TITLE$` |
| `GUI_STATUSBAR_SET_TIP(sb, field, tip$)` | tooltip of a field — formerly `GUI_STATUSBAR_TIP` |
| `GUI_STATUSBAR_GET_TIP$(sb, field)` | STRING — read the tooltip of a field — formerly `GUI_STATUSBAR_TIP$` |

### window and screen

| Name | What it does |
|---|---|
| `WINDOW_SET_FULLSCREEN(fs: BOOLEAN)` | switch fullscreen on or off — formerly `SET_FULLSCREEN` |
| `WINDOW_SET_POS(x: INTEGER, y: INTEGER)` | put the window at the OS pixel position (x, y) — formerly `SET_WINDOW_POS` |
| `WINDOW_SET_MONITOR(i: INTEGER)` | move the window to monitor i (an invalid index is ignored) — formerly `SET_WINDOW_MONITOR` |
| `WINDOW_SET_TITLE(title: STRING)` | change the window title while running — for the file name or the score, say — formerly `SETWINDOWTITLE` |
| `WINDOW_SET_FPS(n: INTEGER)` | target frame rate; 0 = as fast as possible — formerly `SETFPS` |
| `SCREEN_WIDTH()` | logical window size (as passed to SCREEN); 0 before SCREEN — formerly `SCREENWIDTH` |
| `SCREEN_HEIGHT()` | logical window size (as passed to SCREEN); 0 before SCREEN — formerly `SCREENHEIGHT` |
| `SCREENSHOT_SAVE(path: STRING)` | Saves the current screen contents as a PNG — formerly `SAVESCREENSHOT` |
| `QUIT_REQUESTED()` | TRUE when the program should end: window X / Alt+F4, the frame limit of DHRT_FRAMES/--bilder — and, by … — formerly `QUITREQUESTED` |

### drawing

| Name | What it does |
|---|---|
| `DRAW_IMAGE(image, x, y)` | draw the image at (x, y) — formerly `DRAWIMAGE` |
| `DRAW_IMAGE_FLIPPED(image, x, y[, flip_x[, flip_y]])` | with mirroring — formerly `DRAWIMAGEFLIPPED` |
| `DRAW_IMAGE_PART(image, sx, sy, sw, sh, x, y)` | draw a sub-rectangle from a sheet — formerly `DRAWIMAGEPART` |
| `DRAW_IMAGE_PART_EX(image, sx, sy, sw, sh, x, y, w, h [, color])` | a sub-rectangle stretched to w x h; color tints it, its alpha is the opacity (RGBA(255, 255, 255, 128) = … — formerly `DRAWIMAGEPARTEX` |
| `DRAW_IMAGE_ROT(image, x, y, angle_deg [, scale [, tint]])` | centred on (x,y), rotated by winkel degrees (about the centre), optionally scaled + tinted. Ideal for … — formerly `DRAWIMAGEROT` |
| `DRAW_IMAGE_9(image, x, y, width, height, left, top, right, bottom[, tint])` | 9-slice: the image stretched into the rectangle, the four corners at their own size, the edges in one … — formerly `DRAWIMAGE9` |
| `DRAW_TILEMAP(tileset, map_arr, tw, th, sx, sy)` | Draws a two-dimensional array of tile numbers as a map — formerly `DRAWTILEMAP` |
| `DRAW_FPS(x, y[, color])` | show the frame rate as “60 FPS” in the built-in font, in screen coordinates (the camera does not apply); … — formerly `DRAWFPS` |
| `BOX_ROUND(x1, y1, x2, y2, radius[, color])` | filled rectangle with rounded corners — formerly `BOXROUND` |
| `RECT_ROUND(x1, y1, x2, y2, radius[, color])` | rectangle outline with rounded corners — formerly `RECTROUND` |
| `CIRCLE_OUTLINE(x, y, r[, color])` | circle as outline only (counterpart to CIRCLE) — formerly `CIRCLEOUTLINE` |
| `ELLIPSE_OUTLINE(x1, y1, x2, y2[, color[, width]])` | ellipse as outline only — formerly `ELLIPSEOUTLINE` |
| `TRIANGLE_OUTLINE(x1, y1, x2, y2, x3, y3[, color[, width]])` | triangle as outline only — formerly `TRIANGLEOUTLINE` |
| `POLYGON_OUTLINE(points[, color[, width]])` | polygon as outline only — formerly `POLYGONOUTLINE` |
| `NGON_OUTLINE(x, y, corners, r, angle[, color[, width]])` | regular polygon as outline only; width as with LINEW — formerly `NGONOUTLINE` |
| `PIE_OUTLINE(x, y, r, from_deg, to_deg[, color])` | circle sector as outline only, with both radii — formerly `PIEOUTLINE` |

### images, fonts, sound, models: `AREA_VERB`

| Name | What it does |
|---|---|
| `IMAGE_LOAD(path: STRING)` | load a file (PNG, JPG, BMP, …) — formerly `LOADIMAGE` |
| `IMAGE_GET_PIXEL(image, x, y)` | read the pixel colour (&HRRGGBB) at (x, y); -1 for an index outside. Counterpart to PLOT (write) — for … — formerly `GETPIXEL` |
| `IMAGE_GET_ALPHA(image, x, y)` | opacity at (x, y), 0..255; -1 for an index outside. Needed because GETPIXEL returns a colour, and there … — formerly `GETALPHA` |
| `FONT_LOAD(path$, size[, chars$])` | load TTF/OTF/TTC → FONT handle (INTEGER); chars$ = script blocks ("kyrillisch, griechisch", "japanisch", … — formerly `LOADFONT` |
| `FONT_LOAD_IMAGE(image: IMAGE, separator_color: Zahl, first_char: Zahl)` | Loads a pixel font from a PNG in which the characters are set apart by a separator colour — formerly `LOADFONT_IMAGE` |
| `FONT_SET(handle: INTEGER [, size: INTEGER])` | set the active font; SETFONT(-1) = default font. Without size the size the font was loaded at applies … — formerly `SETFONT` |
| `SOUND_LOAD(path: STRING)` | load a sample (short effects: WAV, OGG) — formerly `LOADSOUND` |
| `SOUND_PLAY(s[, loops[, volume]])` | play, default loops=0, volume=1.0 — formerly `PLAYSOUND` |
| `SOUND_STOP(s)` | formerly `STOPSOUND` |
| `SOUND_FREE(s)` | stop the sound and free its buffer — against buffers piling up with many AUDIO_TONE/AUDIO_SFX/AUDIO_NOISE … — formerly `UNLOADSOUND` |
| `MUSIC_PLAY(path$[, loops[, volume]])` | stream a longer file (.ogg/.mp3/.qoa + tracker modules .mod/.xm — real Amiga sound), default loops=-1 … — formerly `PLAYMUSIC` |
| `MUSIC_STOP()` | one stream at a time. Refilling happens on the audio thread, not in the game loop: there is no … — formerly `STOPMUSIC` |
| `MODEL_LOAD(path$: STRING)` | INTEGER — load a model from a file (OBJ, GLTF, IQM …) — formerly `LOADMODEL` |

### files (by path), files (by handle), folders, paths

| Name | What it does |
|---|---|
| `FILE_EXISTS(p: STRING)` | file present? — formerly `FILEEXISTS` |
| `FILE_SIZE(path)` | size in bytes — formerly `FILESIZE` |
| `FILE_TIME(path$)` | last modified, seconds as in the module zeit — formerly `FILETIME` |
| `FILE_COPY(src, dst)` | copy a file — formerly `COPYFILE` |
| `FILE_DELETE(path)` | delete / rename·move — formerly `DELETEFILE` |
| `FILE_SAME(a$, b$)` | do both paths mean the same file? A text comparison goes wrong with / versus \, with a .. in the path, … — formerly `SAMEFILE` |
| `FILE_APPEND(p: STRING, text: STRING [, encoding$])` | append text at the end (creates the file) — formerly `APPENDFILE` |
| `FILE_READ$(f [, encoding$])` | read the whole rest — formerly `READALL$` |
| `FILE_READ_LINES(path [, encoding$])` | read a file as an array of lines — formerly `READLINES` |
| `FILE_READ_BYTES(path)` | the whole file as bytes — formerly `READALL_BYTES` |
| `FILE_WRITE(path, text [, encoding$])` | write text completely (overwrites/creates) — formerly `WRITEALL` |
| `FILE_WRITE_BYTES(path, buffer)` | bytes into a file (overwrites) — formerly `WRITEALL_BYTES` |
| `FILE_OPEN(path: STRING, mode: STRING [, encoding$])` | modes: "r" read, "w" write anew, "a" append — formerly `OPENFILE` |
| `FILE_CLOSE(f)` | formerly `CLOSEFILE` |
| `FILE_READ_LINE$(f)` | next line (without \n); at the end of the file "" like an empty line, so ask EOF(f) — formerly `READLINE` |
| `FILE_WRITE_LINE(f, text)` | writes + \n — formerly `WRITELINE` |
| `FILE_EOF(f)` | at the end? — formerly `ENDOFFILE` |
| `DIR_EXISTS(path)` | directory present? — formerly `DIREXISTS` |
| `DIR_LIST(path [, pattern$])` | entry names (sorted), optionally filtered — formerly `DIRLIST` |
| `DIR_LIST_REC(path$ [, pattern$])` | as above, but recursive — files only, relative paths with / — formerly `DIRLIST_REC` |
| `PATH_JOIN(a, b, ...)` | join path parts with / — formerly `PATHJOIN` |
| `PATH_DIR(p: STRING)` | directory part (without the last component) — formerly `DIRNAME` |
| `PATH_BASE(p: STRING)` | last path component (file/folder name) — formerly `BASENAME` |
| `PATH_REAL$(path$)` | the full, cleaned-up path: absolute, ./.. resolved, in the spelling on disk; if the file does not exist, … — formerly `REALPATH$` |
| `START_DIR$()` | the folder the program was called from — formerly `STARTDIR$` |
| `ARG_PATH$(number)` | argument no. n as a path from where the program was called (see below) — formerly `ARGPATH$` |
| `ENV_GET$(name, default = "")` | read an environment variable — formerly `GETENV$` |
| `ENV_SET(name, value)` | set an environment variable (this program + its children) — formerly `SETENV` |
| `URL_OPEN(address: STRING)` | Opens an address in the default browser — the manual or the game's page, for instance — formerly `OPENURL` |
| `DOC_OPEN(path$)` | Opens a local file with its default program – the PDF you have written in the viewer, the table in the … — formerly `OPENDOC` |

### MAP like ARRAY_ and SET_: with underscore, the number is COUNT

| Name | What it does |
|---|---|
| `MAP_GET(m, key)` | read, throws if the key is missing — formerly `MAPGET` |
| `MAP_GET_OR(m, key, default)` | read with a default — formerly `MAPGETOR` |
| `MAP_PUT(m, key, value)` | set / overwrite — formerly `MAPPUT` |
| `MAP_HAS(m, key)` | existence — formerly `MAPHAS` |
| `MAP_REMOVE(m, key)` | TRUE if removed — formerly `MAPREMOVE` |
| `MAP_CLEAR(m)` | Empties the whole map at once — formerly `MAPCLEAR` |
| `MAP_KEYS(m)` | all keys — formerly `MAPKEYS` |
| `MAP_VALUES(m)` | Gives all the values as an array (in insertion order) — formerly `MAPVALUES` |
| `MAP_ITEMS(m)` | Gives all the pairs as an array of (key, value) tuples — ideal with FOR EACH — formerly `MAPITEMS` |
| `MAP_COUNT(m)` | The number of key-value pairs — formerly `MAPSIZE` |
| `ARRAY_DIM_COUNT(arr)` | number of dimensions — formerly `DIMCOUNT` |
| `ARRAY_DIM_SIZE(arr, n)` | size of the n-th dimension (0-based) — formerly `DIMSIZE` |
| `MOVE_TOWARD(cur, target, maxdelta)` | move cur by at most maxd towards target — formerly `MOVETOWARD` |

### English instead of German (modules zeit and geld, FFI pointers)

| Name | What it does |
|---|---|
| `TIME_NOW()` | now, as a point in time — formerly `ZEIT_JETZT` |
| `TIME_UNIX()` | now as Unix time: seconds since 1970 in UTC, with fraction (like Python's time.time()) — formerly `ZEIT_UNIX` |
| `TIME_FROM_UNIX(unix_seconds)` | Unix seconds to a point in local time (with that day's daylight saving time) — formerly `ZEIT_AUS_UNIX` |
| `TIME_TO_UNIX(time)` | point in local time to Unix seconds — formerly `ZEIT_NACH_UNIX` |
| `TIME_FROM_PARTS(year: INTEGER, month: INTEGER, day: INTEGER, hour: INTEGER = 0, minute: INTEGER = 0, second: INTEGER = 0)` | point in time from individual values — formerly `ZEIT_AUS_TEILEN` |
| `TIME_PART(time: INTEGER, part: STRING)` | one field: jahr, monat, tag, stunde, minute, sekunde (year, month, day, hour, minute, second) — formerly `ZEIT_TEIL` |
| `TIME_FORMAT$(time: INTEGER, pattern: STRING)` | point in time according to a pattern — formerly `ZEIT_FORMAT$` |
| `TIME_TEXT$(time: INTEGER)` | point in time as JJJJ-MM-TT hh:mm:ss (YYYY-MM-DD hh:mm:ss) — formerly `ZEIT_TEXT$` |
| `TIME_PARSE(text: STRING)` | text to point in time; aborts on anything unreadable — formerly `ZEIT_PARSE` |
| `TIME_CAN_PARSE(text: STRING)` | could the text be read? (asks, does not abort) — formerly `ZEIT_LESBAR` |
| `TIME_ADD(time: INTEGER, seconds: INTEGER)` | add seconds (negative = back) — formerly `ZEIT_PLUS` |
| `TIME_DIFF(time: INTEGER, earlier: INTEGER)` | distance in seconds (negative if time is earlier) — formerly `ZEIT_DIFF` |
| `TIME_DURATION$(seconds: INTEGER)` | seconds as a readable span — formerly `ZEIT_DAUER$` |
| `TIME_WEEKDAY(time: INTEGER)` | 1 = Monday … 7 = Sunday — formerly `ZEIT_WOCHENTAG` |
| `MONEY_NEW(amount)` | from text ("19,99", "1.234,56") or a number — formerly `GELD_NEU` |
| `MONEY_ABS(g: GELD)` | amount without sign — formerly `GELD_ABS` |
| `MONEY_CENTS(g: GELD)` | to whole cents, rounded commercially (half up) — formerly `GELD_CENT` |
| `MONEY_FROM_CENTS(cent: INTEGER)` | from whole cents — formerly `GELD_AUS_CENT` |
| `MONEY_ROUND(g: GELD [, digits])` | commercial rounding, 2 places by default — formerly `GELD_RUNDEN` |
| `MONEY_SPLIT(g: GELD, count)` | split up without losing a cent — formerly `GELD_TEILEN` |
| `MONEY_TEXT$(g: GELD [, symbol$])` | 19,99 €, symbol selectable — formerly `GELD_TEXT$` |
| `MONEY_VALUE(g: GELD)` | step out deliberately (display in charts or the like) — formerly `GELD_ZAHL` |
| `BUFFER_FROM_POINTER(pointer, length)` | copies laenge (length) bytes behind a pointer into a new BUFFER; 0 bytes give an empty one, a null pointer … — formerly `BUFFER_AUS_ZEIGER` |
| `TEXT_FROM_POINTER$(pointer [, wide])` | copies the text behind a pointer up to the null character (UTF-8; with breit (wide) = TRUE wchar_t, i.e. … — formerly `TEXT_AUS_ZEIGER$` |
