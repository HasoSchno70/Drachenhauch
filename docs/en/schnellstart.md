# Quick start

Five minutes from a first look to your own program -- first in the browser,
then on your own machine.

## 1. Try it in the browser

The [playground](https://hasoschno70.github.io/Drachenhauch/spielwiese/index.html?lang=en)
runs Drachenhauch without installing anything. The first buttons in the bar
are three starter projects, marked with ★:

* [Mini game](https://hasoschno70.github.io/Drachenhauch/spielwiese/index.html?lang=en&beispiel=spiel)
  -- collect stars; change `SPEED` or `STARS`.
* [Form](https://hasoschno70.github.io/Drachenhauch/spielwiese/index.html?lang=en&beispiel=formular)
  -- a window with input, button and list; add a second button.
* [Paint](https://hasoschno70.github.io/Drachenhauch/spielwiese/index.html?lang=en&beispiel=malen)
  -- draw with the mouse; change `SIZE`.

Change a number, press **Run** -- that is your first change.

## 2. Install

[Download Drachenhauch](https://github.com/HasoSchno70/Drachenhauch/releases/latest):
an installer on Windows, a `.dmg` on macOS, a `.tar.gz` on Linux. Then start
**Drachenhauch**; the start screen asks "How would you like to work?" -- pick
**The IDE** to begin with.

## 3. Your first program

In the IDE choose **File → New** (Ctrl+N), paste this text and press **F5**:

```basic
' Collect the stars! The arrow keys move the dragon.
CONST TEMPO = 220.0
SCREEN(480, 320, "Drachenhauch", 2)

DIM x AS FLOAT = 240.0
DIM y AS FLOAT = 160.0
DIM sx AS INTEGER : sx = RANDINT(20, 460)
DIM sy AS INTEGER : sy = RANDINT(40, 300)
DIM score AS INTEGER = 0

WHILE NOT QUITREQUESTED()
    DIM dt AS FLOAT : dt = DELTA()
    IF KEY_DOWN(KEY_LEFT) THEN x = x - TEMPO * dt
    IF KEY_DOWN(KEY_RIGHT) THEN x = x + TEMPO * dt
    IF KEY_DOWN(KEY_UP) THEN y = y - TEMPO * dt
    IF KEY_DOWN(KEY_DOWN) THEN y = y + TEMPO * dt
    ' caught a star: one point, and it jumps somewhere else
    IF ABS(x - sx) < 20 AND ABS(y - sy) < 20 THEN
        score = score + 1
        sx = RANDINT(20, 460)
        sy = RANDINT(40, 300)
    END IF
    CLS(RGB(16, 22, 36))
    CIRCLE(sx, sy, 7, RGB(255, 220, 80))
    CIRCLE(INT(x), INT(y), 14, RGB(255, 120, 40))
    TEXT(10, 8, "Score: " + STR$(score))
    FLIP()
WEND
```

What is in it:

* `SCREEN` opens the window, `WHILE NOT QUITREQUESTED()` is the game loop,
  `FLIP` shows the frame you drew.
* `DELTA()` is the time since the last frame -- with it the dragon moves at
  the same speed on every machine.
* `KEY_DOWN` asks for a key being held, `KEY_HIT` for one just pressed.
* Every variable gets a type with `DIM`. A typo in a name shows up before
  the program starts -- the IDE underlines it while you type.

## 4. An application with a window

A form instead of a game -- the same language, plus the `gui` module:

```basic
IMPORT "gui"
SCREEN(420, 300, "Drachenhauch", 1)

DIM w AS GUI_WINDOW : w = GUI_WINDOW("Hello", 10, 10, 400, 280)
GUI_LABEL(w, "Name:", 20, 20)
DIM nameBox AS GUI_WIDGET : nameBox = GUI_TEXTINPUT(w, 20, 46, 240, 30, "Name")
DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(w, "OK", 270, 46, 110, 30)
DIM list AS GUI_WIDGET : list = GUI_LISTBOX(w, 20, 90, 360, 160, [])

WHILE NOT QUITREQUESTED()
    GUI_UPDATE()
    ' button or Enter in the field: add a greeting to the list
    IF GUI_CLICKED(ok) OR GUI_ENTERED(nameBox) THEN
        GUI_LISTBOX_ADD(list, "Hello, " + GUI_GET_TEXT$(nameBox) + "!")
    END IF
    CLS(RGB(30, 34, 44))
    GUI_DRAW()
    FLIP()
WEND
```

Larger applications are built in **Drachenhauch Studio**: click windows
together, write the events, F5 -- database, tables and reports included. It
starts from the same start screen.

## 5. Share it

```bash
dhrt --export spiel.dh
```

turns the program into a standalone file that runs without Drachenhauch
installed (in the IDE: **Run → Export as standalone program**, Ctrl+F6). The
export takes the smallest runtime that lacks none of the program's commands:

| Program | File size |
|---|---|
| without a window ("Hello world", a tool) | about 16 MB |
| with a database, network or mail | about 22 MB |
| a game or a window | about 35 MB |
| PDF, video, hardware | about 45 MB (the full runtime) |

## Where to go next

* **[Getting started](../../buecher/einstieg/README.md)** -- the book for
  beginners, chapter by chapter to your own game.
* **[Switching](umstieg.md)** -- if you know QBasic, VB, Blitz or Python:
  what is called differently here.
* **Examples** -- in the IDE under **Tools → Examples by topic**, more than
  230 programs from particle storms to invoices.
* **[Language reference](sprache.md)** and **[Command names](builtins-namen.md)**
  to look things up.
