# Recording and playing back input (`AUTOMATION_*`)

The runtime can **record the input state of every frame** — keys, mouse
buttons, mouse position, wheel, gamepad, touch — and feed it back into the
input later. The game notices nothing: during playback `KEYPRESSED`, `MOUSEX`,
`MOUSE_HIT` & co. return the recorded values.

Three things you need this for:

* **Demo/attract mode** — the game plays itself while nobody is playing.
* **Reproducible bug reports** — “this is how I walked through the wall”.
* **Automated game tests** — the same sequence on every run, no manual work.

Needs a window (`SCREEN`) and the native runtime `dhrt`.

## Commands

| Command | Effect |
|---|---|
| `AUTOMATION_RECORD(datei$)` | start recording (a running playback ends) |
| `AUTOMATION_STOP()` → INTEGER | ends recording **or** playback; writes the recording to its file and returns the number of events |
| `AUTOMATION_PLAY(datei$)` → INTEGER | load a recording and play it from the next frame on; returns the number of events loaded |
| `AUTOMATION_RECORDING()` → BOOLEAN | is a recording running? |
| `AUTOMATION_PLAYING()` → BOOLEAN | is a playback running? (becomes FALSE by itself after the last event) |
| `AUTOMATION_FRAME()` → INTEGER | frame number within the playback (0 = first) |
| `AUTOMATION_COUNT()` → INTEGER | events in the running recording or in the loaded file |

```basic
SCREEN(640, 400, "Demo", 1)

IF KEYHIT(KEY_F9) THEN
    IF AUTOMATION_RECORDING() THEN
        PRINT "gespeichert: " + STR$(AUTOMATION_STOP()) + " Ereignisse"
    ELSE
        AUTOMATION_RECORD("aufnahme.txt")
    END IF
END IF

IF KEYHIT(KEY_F10) THEN AUTOMATION_PLAY("aufnahme.txt")
```

Complete demo: [examples/153_automation.dh](../../examples/153_automation.dh)
(F9 records, F10 plays back; the trail shows that the path is identical).

## What you need to know

**What is recorded is the INPUT, not the course of the game.** Playback only
presses the same keys at the same time — everything else has to be the same
starting point:

* **Reset the starting state** before playback begins (position, score,
  level). Otherwise the same input acts on a different beginning.
* **Pin down randomness**: `RANDOMIZE 12345` with a fixed seed, otherwise the
  second run rolls different enemies.
* **Compute per frame, not per second**, if the path must be exactly the
  same. Playback counts in frames; with movement based on `DELTA()`, the same
  recording gives a slightly different path at a different frame rate.

**Timing.** Input is fed in at the end of every `FLIP` — right after the real
input for the next frame has been read, so the recorded values win. An event
from recording frame `N` therefore takes effect in program iteration `N+1`.

**`KEY_ANY_HIT` does not see the demo.** An attract mode typically stops as
soon as the player presses any key — and that is exactly what `KEY_ANY_HIT()`
is for. raylib, however, **also** puts fed-in keys into its “last pressed”
queue; unfiltered, the demo would have ended itself on its own first key
press. `dhrt` therefore hides what the running playback has fed in itself:
`KEY_ANY_HIT` reports only real input, while `KEYHIT`/`KEYPRESSED` still see
the recorded keys (that is the whole point). `JOYSTICK_ANY_BUTTON` does not
need this — there the playback only sets the button state, not raylib's “last
pressed button”.

```basic
IF attract AND (KEY_ANY_HIT() <> -1 OR JOYSTICK_ANY_BUTTON() <> -1) THEN
    AUTOMATION_STOP()          ' player takes over
    attract = FALSE
END IF
```

**Recording and playback exclude each other.** raylib never plays anything
back while a recording is running; `AUTOMATION_PLAY` reports this as an error
instead of silently swallowing it.

**A recording is never completely empty.** raylib records every change of the
mouse position — and the window opens wherever the pointer happens to be. So
even the first frame usually contains two events (`INPUT_MOUSE_POSITION` +
`INPUT_TOUCH_POSITION`), although nobody pressed anything. Whoever checks a
recording for “exactly N events” is really checking the computer's mouse
position.

**Playback holds the mouse position.** raylib records a mouse position only
when it has *changed* — between two such events the recording says “the mouse
is standing still”, and that is exactly what playback restores in every
frame. Without this, the computer it runs on fills the gap: under Windows,
even a window appearing or disappearing under the pointer sends a
`WM_MOUSEMOVE`, and with that raylib's mouse position is somewhere else. A
recorded click, however, spans three frames (position, button down, button
up), and a `gui` button only counts it when it is **released on the button
itself** — a foreign movement in between let the click arrive and have no
effect. This affected test runs with many simultaneous windows overlapping
each other:
replayed one by one, every case was green; in the full run a different one
failed each time. After the last event playback ends, and the mouse belongs
to the computer again.

This changes one small thing for programs that call `MOUSE_SET_POS`
**themselves** while a playback is running: within the same frame the program
still wins, but the position it set no longer stays — as soon as the program
stops setting it, the recording pulls it back in the next frame (measured:
`MOUSE_SET_POS(77,88)` in frames 4–8, after that the mouse is back on the
recorded position from frame 9 on; before, it stayed at 77,88). Whoever wants
to hold a position of their own keeps setting it or ends the playback with
`AUTOMATION_STOP()`.

**`AUTOMATION_PLAY` prints a raylib warning** — `AUTOMATION: [datei]
Issue reading line to buffer`, for *every* file, even a flawless one. raylib
reads up to the end of the file and complains about the last read attempt.
Cosmetic, not an error: the return value is correct anyway.

**The file format is raylib's text format** (one line per event:
`e <frame> <typ> <p0> <p1> <p2> <p3>`). It can be viewed with a text editor
and also written by hand — handy for test sequences you do not want to
“play in” first. The list holds at most 16384 events; a held key costs **one
event per frame**, so very long recordings fill up at some point (raylib then
silently stops recording).

**Limit of the testing:** playback is tested automatically
([tests/pruef/automation.dhtest](../../tests/pruef/automation.dhtest) writes
recording files itself and checks that `KEYPRESSED`/`MOUSEX`/`MOUSE_HIT`
return exactly the recorded values). For recording, it is verified that real
input is captured and written in the right format — the round trip “record
real key presses and play them back identically” can only be reproduced by
hand (synthetic Windows input does not reach GLFW reliably); that is what the
demo is for.
