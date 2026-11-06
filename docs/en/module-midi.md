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

SysEx is **left out**. Clock ticks, timecode and active sensing arrive, but
go to the **clock** (below) instead of the queue — a keyboard sends dozens of
them per second, and none of them is a note. Start, stop and song position
are also put in the queue (as before 2026.43).

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

## The clock

A sequencer, drum machine or DAW sends 24 **clock ticks** per quarter note,
plus start, stop, continue and the song position. The input works out tempo
and position from them:

| Function | Effect |
|---|---|
| `MIDI_CLOCK_BPM(h)` → FLOAT | tempo of the other side in beats per minute, averaged over the last 24 ticks; 0 if none came for half a second |
| `MIDI_CLOCK_RUNNING(h)` → BOOLEAN | after start or continue until stop |
| `MIDI_CLOCK_TICKS(h)` → INTEGER | ticks since start (24 per quarter; `\ 24` = beat); the song position sets it |

```basic
IMPORT "midi"
DIM ein AS MIDI_IN
ein = MIDI_IN_OPEN(0)
WHILE TRUE
    IF MIDI_CLOCK_RUNNING(ein) THEN
        PRINT FORMAT$(MIDI_CLOCK_BPM(ein), "%.1f"); " BPM, Schlag "; MIDI_CLOCK_TICKS(ein) \ 24
    END IF
    SLEEP(250)
WEND
```

Being the clock yourself — for a device that should follow Drachenhauch:

| Function | Effect |
|---|---|
| `MIDI_CLOCK_SEND(h, bpm)` | send 24 ticks per quarter from a thread of its own (1..999); a new tempo applies without a restart, 0 stops |
| `MIDI_START(h)` | start (FA): the device begins from the top |
| `MIDI_STOP(h)` | stop (FC) |
| `MIDI_CONTINUE(h)` | continue (FB): from where it stood |
| `MIDI_SONG_POSITION(h, sechzehntel)` | jump to a position (0..16383 sixteenths) |

The thread computes every tick from a fixed start, not from the previous one —
so no delay adds up. If the computer stalls for longer than one tick, it
starts counting afresh instead of catching up at double speed.

## MIDI files

Reading and writing `.mid` files works **in every build** — without a device
and without the feature. Outwardly the module works in **seconds** (reading:
tempo changes are already included) and in **quarter notes** (writing):

| Function | Effect |
|---|---|
| `MIDI_FILE_LOAD(pfad$)` → MIDI_FILE | read a file (format 0, 1 and 2) |
| `MIDI_FILE_NEW([ticks_je_viertel])` → MIDI_FILE | an empty file (default 480) |
| `MIDI_FILE_SAVE(mf, pfad$)` | write (format 1) |
| `MIDI_FILE_FREE(mf)` | release |
| `MIDI_FILE_TRACKS(mf)` → INTEGER | number of tracks |
| `MIDI_FILE_PPQ(mf)` → INTEGER | ticks per quarter; 0 for a file that counts in SMPTE frames |
| `MIDI_FILE_DURATION(mf)` → FLOAT | length in seconds |
| `MIDI_FILE_TEMPO(mf)` → FLOAT | tempo at the start (120 if none is given) |
| `MIDI_FILE_TRACK_NAME$(mf, spur)` → STRING | name of a track |
| `MIDI_FILE_NOTES(mf)` → INTEGER | number of notes (note on and note off combined) |
| `MIDI_FILE_NOTE(mf, i)` → TUPLE | `(beginn, dauer, kanal, note, anschlag, spur)`: start, duration, channel, note, velocity, track; ordered by start |
| `MIDI_FILE_EVENTS(mf)` → INTEGER | number of channel messages over all tracks |
| `MIDI_FILE_EVENT(mf, i)` → TUPLE | `(sekunde, spur, status, kanal, d1, d2)`, in time order; `status` as with `MIDI_STATUS` (144 = note on) |
| `MIDI_FILE_ADD_NOTE(mf, spur, kanal, note, anschlag, beginn, dauer)` | a note; start and duration in quarter notes |
| `MIDI_FILE_ADD_EVENT(mf, spur, zeitpunkt, status, d1 [, d2])` | another channel message (program 192, controller 176, ...; channel in the status byte) |
| `MIDI_FILE_SET_TEMPO(mf, bpm [, zeitpunkt])` | tempo from a point in time (default 0) |
| `MIDI_FILE_SET_TRACK_NAME(mf, spur, name$)` | name a track |

Tracks are created on first use (0..127). The tempo map is in track 0, as most
programs expect.

```basic
IMPORT "midi"
DIM mf AS MIDI_FILE
mf = MIDI_FILE_LOAD("lied.mid")
DIM t, dauer AS FLOAT
DIM kanal, note, anschlag, spur, i AS INTEGER
FOR i = 0 TO MIDI_FILE_NOTES(mf) - 1
    (t, dauer, kanal, note, anschlag, spur) = MIDI_FILE_NOTE(mf, i)
    PRINT FORMAT$(t, "%.2f"); " s  "; MIDI_NOTE_NAME$(note); "  "; FORMAT$(dauer, "%.2f"); " s"
NEXT
```

```basic
IMPORT "midi"
DIM mf AS MIDI_FILE
mf = MIDI_FILE_NEW()
MIDI_FILE_SET_TEMPO(mf, 100)
MIDI_FILE_SET_TRACK_NAME(mf, 1, "Melodie")
DIM i AS INTEGER
FOR i = 0 TO 7
    MIDI_FILE_ADD_NOTE(mf, 1, 1, 60 + i * 2, 100, i * 0.5, 0.5)
NEXT
MIDI_FILE_SAVE(mf, "tonleiter.mid")
```

A note off ends the **oldest** open note of the same key, and at the same time
the module writes note offs first, then program and controller changes, note
ons last — otherwise a repeated note would cut itself off, or still sound with
the old program.

### Playing

| Function | Effect |
|---|---|
| `MIDI_FILE_PLAY(h, mf [, ab_sekunde])` | play the file on an output, in a thread of its own |
| `MIDI_FILE_STOP(h)` | stop; notes still sounding are switched off |
| `MIDI_FILE_PLAYING(h)` → BOOLEAN | is it still playing? |
| `MIDI_FILE_POSITION(h)` → FLOAT | where it is, in seconds |

Playing needs the `midi` feature; the thread keeps time, not the frame loop —
a frame is 16 ms, and you can hear that.

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
* **No MIDI timecode** (MTC); the clock only knows ticks, start and stop.
* MIDI files: SysEx and meta events other than tempo and track name are read
  and written back, but not evaluated; writing only in quarter notes (not in
  SMPTE frames).
* **No virtual port** that other programs can see.
