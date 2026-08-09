# Drachenhauch – The Textbook

*[Deutsch](README.md) · English overview*

**The book itself is written in German** (`buch/Drachenhauch-Lehrbuch.docx`) — a complete
teach-yourself-and-reference book for Drachenhauch covering the whole language plus every
built-in command, each with its own small example program. This page exists so that
readers who don't read German can still see what the book covers and decide whether it's
worth running through a translator.

If you'd like to help translate it (fully or chapter by chapter), see the "Build" section
in [`buch/OUTLINE.md`](buch/OUTLINE.md) — content lives in small, self-contained
`buch/content/NN_*.js` files, one per chapter, so a translated edition could reuse the same
build pipeline.

- Build instructions & full outline (German): [`buch/OUTLINE.md`](buch/OUTLINE.md)
- Build: `cd buch && node build_book.js` → `Drachenhauch-Lehrbuch.docx`. With correct
  table-of-contents page numbers: `python make_book.py`.

## Table of contents (translated)

**Part I — Getting started**
- Foreword & welcome
- What is Drachenhauch?
- Installation, editor & running programs
- Your first program

**Part II — The language**
- Variables & data types
- Operators & expressions
- Input/output: PRINT, INPUT, f-strings
- Branching: IF/ELSEIF/ELSE, SELECT CASE, IIF
- Loops: FOR, WHILE, REPEAT, FOR EACH, BREAK/CONTINUE
- Functions & subs (parameters/BYREF/defaults/named/variadic/FUNCREF/recursion)
- Strings in detail
- Arrays
- Maps
- Tuples & destructuring
- Classes & objects
- Inheritance, properties, operator overloading, static members
- ENUM
- Comprehensions (list/dict/set)
- Error handling (TRY/CATCH/THROW)
- Coroutines (YIELD)
- Importing modules (IMPORT)

**Part III — Built-in commands (reference)**
- Console & I/O
- Math
- Random numbers
- String functions
- Type conversion & checking
- Array helpers (SORT/PUSH/POP/…)
- Map helpers
- Time & date
- Files

**Part IV — Graphics, sound & games**
- The window (SCREEN/FLIP/DELTA/FPS/game loop)
- 2D drawing (PLOT/LINE/BOX/RECT/CIRCLE/TEXT)
- 2D extras (thick lines/rounded rects/gradients/splines/blend modes/procedural textures/render targets)
- Images (LOADIMAGE/DRAWIMAGE/DRAWIMAGEPART/…)
- Colors (RGB/HSV/COLOR_LERP)
- Input (keyboard/mouse/gamepad)
- Sound (LOADSOUND/PLAYSOUND/PLAYMUSIC/AUDIO_*)
- Layers, sprite atlas, bulk draws
- 3D graphics (`g3d`)
- Capstone project: "Coin Catch" (a small graphics+sound game)

**Part V — The modules**

One chapter per built-in module: `sprite`, `animfsm` (animation state machine), `tween`,
`timer`, `particles`, `physics`/`physics2d`/`physics3d` (rigid-body physics), `camera`,
`input`, `ui` (immediate-mode UI), `gui` (retained-mode windows/widgets), `scene`
(scene stack), `save` (save slots), `astar` (pathfinding), `tiled` (Tiled map loader),
`tile_collide` (platformer tile collision), `controller` (character controller), `vec2`,
`m3d` (3D math: vectors/quaternions/matrices), `json`, `db` (SQLite), `regex`, `audio`
(advanced), `curves` (animation curves), `net` (TCP/UDP), `html` (HTTP + HTML parsing),
`ecs` (entity-component-system), and the hardware modules `serial`/`usb`/`wifi`/`bt`.

**Appendix**
- A — Command index (alphabetical, auto-generated from the engine's built-in registry)
- B — Key codes
- C — Color constants
- D — Understanding error messages

## Status

Complete: all five parts (chapters 0–76) plus the appendix, 289 pages, every example
verified against the real `dhrt` runtime. See [`buch/OUTLINE.md`](buch/OUTLINE.md) for the
detailed, chapter-by-chapter build log (German).
