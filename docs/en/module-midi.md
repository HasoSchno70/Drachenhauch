# Module `midi`

Reading notes from a connected instrument and sending some out.

```basic
IMPORT "midi"
```

Drachenhauch ships a tracker, a sampler, an sfxr synth, a score editor and
Kira buses — until now no connected keyboard could drive any of them. That is
exactly the bridge this module closes.

> **Hardware module.** It is only included in builds with `--hardware`
> (`rust/build_runtime.py --hardware`). Whether your `dhrt` has it is shown in
> the second line of `dhrt --version`. The foundation is `midir`
> (WinMM / ALSA / CoreMIDI); on Linux the build needs the ALSA headers.
>
> **Exception:** `MIDI_NOTE_NAME$` and `MIDI_NOTE_FREQ` only convert and are
> available in **every** build. If you build a note display or a tone from a
> note number, you need no device for it.

## Finding ports

| Function | Effect |
|---|---|
| `MIDI_IN_COUNT()` → INTEGER | How many inputs there are |
| `MIDI_OUT_COUNT()` → INTEGER | How many outputs there are |
| `MIDI_IN_NAME$(i)` → STRING | Name of input `i` |
| `MIDI_OUT_NAME$(i)` → STRING | Name of output `i` |

```basic
DIM i AS INTEGER
FOR i = 0 TO MIDI_IN_COUNT() - 1
    PRINT i; ": "; MIDI_IN_NAME$(i)
NEXT
```

Windows comes with an output out of the box (*Microsoft GS Wavetable
Synth*) — with it you can try out sending without any device connected.

## Opening and closing

| Function | Effect |
|---|---|
| `MIDI_IN_OPEN(i)` → `MIDI_IN` | Open an input |
| `MIDI_OUT_OPEN(i)` → `MIDI_OUT` | Open an output |
| `MIDI_IN_CLOSE(h)` / `MIDI_OUT_CLOSE(h)` | Close again |

Both handles are INTEGER indices (like `SERIAL_HANDLE`). After closing, every
further access is an error in plain words, not a silent nothing.

## Receiving

As with `db` and `mqtt`, through a **cursor**: `MIDI_NEXT` fetches the next
message into a buffer, and the other commands read from it.

| Function | Effect |
|---|---|
| `MIDI_NEXT(h)` → BOOLEAN | Fetch the next message; `FALSE` = nothing there |
| `MIDI_PENDING(h)` → INTEGER | How many are still waiting |
| `MIDI_IS_NOTE_ON(h)` / `MIDI_IS_NOTE_OFF(h)` / `MIDI_IS_CC(h)` | Kind of message |
| `MIDI_NOTE(h)` → INTEGER | Note number (0..127, 60 = C4) |
| `MIDI_VELOCITY(h)` → INTEGER | Velocity (0..127) |
| `MIDI_CC_NUMBER(h)` / `MIDI_CC_VALUE(h)` | Controller number and value |
| `MIDI_CHANNEL(h)` → INTEGER | Channel **1..16** |
| `MIDI_STATUS(h)` / `MIDI_DATA1(h)` / `MIDI_DATA2(h)` | The raw bytes |

```basic
IMPORT "midi"
IMPORT "audio"

DIM tastatur AS MIDI_IN
tastatur = MIDI_IN_OPEN(0)

SCREEN(320, 200)
WHILE NOT QUITREQUESTED()
    WHILE MIDI_NEXT(tastatur)
        IF MIDI_IS_NOTE_ON(tastatur) THEN
            AUDIO_TONE(MIDI_NOTE_FREQ(MIDI_NOTE(tastatur)), 300, "sine")
            PRINT MIDI_NOTE_NAME$(MIDI_NOTE(tastatur))
        END IF
    WEND
    FLIP()
WEND
```

The inner `WHILE MIDI_NEXT(...)` loop matters: several messages can have
arrived per frame (a chord alone is three).

**Two peculiarities of the protocol** you need to know:

1. **Most instruments do not send a “note off".** They send a *note on* with
   velocity 0. `MIDI_IS_NOTE_OFF` catches both forms — if you check
   `MIDI_STATUS` yourself instead, you get notes that never end.
2. **Channels count from 1 here**, as on every device display. The protocol
   has 0..15; the module does the conversion.

Clock, active-sensing and SysEx messages are **left out**. A keyboard sends
dozens of them per second, and none of them is a note.

The buffer holds **1024 unfetched messages**; if it overflows, the oldest one
is dropped. Whoever plays live wants to see the current keystroke, not the one
from ten seconds ago.

## Sending

| Function | Effect |
|---|---|
| `MIDI_NOTE_ON(h, kanal, note, anschlag)` | Strike a note |
| `MIDI_NOTE_OFF(h, kanal, note)` | Note off |
| `MIDI_CC(h, kanal, nr, wert)` | Controller (e.g. 7 = volume) |
| `MIDI_SEND(h, status, d1, d2)` | Raw message, for everything else |

```basic
DIM synth AS MIDI_OUT
synth = MIDI_OUT_OPEN(0)
MIDI_NOTE_ON(synth, 1, 60, 100)     ' strike C4
SLEEP(400)
MIDI_NOTE_OFF(synth, 1, 60)
MIDI_OUT_CLOSE(synth)
```

Note, velocity and controller values must lie between **0 and 127**, channels
between **1 and 16**. A larger value would set the status bit in the protocol
and be read as a completely different message — that is why it is rejected
instead of silently doing something else. The check runs **before** the
device is accessed.

## Converting

| Function | Effect |
|---|---|
| `MIDI_NOTE_NAME$(note)` → STRING | `60` → `"C4"`, empty outside 0..127 |
| `MIDI_NOTE_FREQ(note)` → FLOAT | `69` → `440.0` Hz — the bridge to `AUDIO_TONE` |

Octave numbering follows the common convention (note 60 = C4). Some
manufacturers call the same note C3 — that is a way of counting, not an
error. And note 71 is called **H** here (German note naming), not B.

## What is tested

There is **no MIDI device** attached to the development machine, nor at the
author's place. That shaped the design:

* The **decoding** of incoming messages works on raw bytes, not on the device
  type — and can therefore be tested completely with made-up messages. Nine
  Rust tests cover note on, both forms of note off, channels 1 and 16,
  controllers, the empty message and one that is too short. These tests run
  in **every** build, even without the feature.
* **Sending** runs for real: Windows ships a MIDI output, so in the test a
  triad really goes to the synthesizer and off again.
* **The whole loop too** — send, through the operating system, receive again
  — runs in the test as soon as a **virtual loopback port** is available
  (on Windows e.g. [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html),
  via `winget install TobiasErichsen.loopMIDI`). Such a port appears under the
  same name as input *and* output; that is exactly how the tests recognise it,
  and without it they skip themselves. **A real keyboard is not needed for
  this.** This also backs up the two promises above instead of merely
  claiming them: that a note on with velocity 0 arrives as a note off, and
  that the queue caps at 1024 and drops the **oldest** (1200 sent → 1024
  waiting, the first survivor is the 176th).

What is *not* tested this way: that a particular **instrument** behaves the
way the protocol intends. Feedback from someone with hardware remains
welcome.

## What it cannot do

* **No SysEx** — neither sending nor receiving. Device-specific sound data
  stays out.
* **No MIDI clock, no MIDI timecode.** If you want to run to a foreign clock,
  `audio` has one of its own with `AUDIO_CLOCK_*`.
* **No reading or writing of MIDI files** (`.mid`). For music from disk there
  is `PLAYMUSIC` (also `.mod`/`.xm`) and the tracker.
* **No virtual port** that other programs can see.
