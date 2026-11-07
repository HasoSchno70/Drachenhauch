# Microphone (`MIC_*`)

What arrives at the computer's input: how loud it is, which note is sounding
right now, the spectrum as bars, the waveform to draw — and a recording as a
sound or WAV file. That makes a tuner, a level meter, a game you control with
your voice, or a clap switch possible.

```basic
IMPORT "audio"
```

The commands belong to the `audio` module. Input is read through the same
layer that sound goes out through (cpal): WASAPI under Windows, CoreAudio
under macOS, ALSA under Linux (PulseAudio and PipeWire too, through their
ALSA bridge). **One** input runs at a time; it is opened in the device's
default format and mixed down to **one** channel.

## First steps

```basic
IMPORT "audio"
SCREEN(400, 120, "Pegel")
MIC_START()
WHILE NOT QUITREQUESTED()
    CLS(&H101018)
    DIM p AS FLOAT : p = MIC_LEVEL()
    BOX(20, 40, 20 + INT(p * 360), 80, IIF(p > 0.9, &HFF4040, &H40D060))
    TEXT(20, 90, MIC_DEVICE$())
    FLIP()
WEND
```

## Devices

| Command | What it does |
|---|---|
| `MIC_DEVICES()` → ARRAY OF STRING | the names of all inputs, the default first; empty if the computer has none |
| `MIC_START([geraet$])` | open an input — without a name the default. A name matches exactly or as a part, ignoring case (`"USB"` finds "Microphone (USB Audio Device)"); an unknown one is an error that lists the available ones. A running input is closed first |
| `MIC_STOP()` | close the input; a running recording is dropped |
| `MIC_ACTIVE()` → BOOLEAN | is an input running (and has it not reported an error)? |
| `MIC_ERROR$()` → STRING | the error the device reported (unplugged, refused) — otherwise `""` |
| `MIC_DEVICE$()` → STRING | name of the open input, otherwise `""` |
| `MIC_RATE()` → INTEGER | sample rate of the open input (usually 44100 or 48000), otherwise 0 |
| `MIC_SAMPLES()` → INTEGER | how many values arrived since `MIC_START` — that shows whether anything is coming at all |

Everything below requires a running input; without one it is an error — a
silent zero could not be told apart from "it is quiet".

## Level and pitch

| Command | What it does |
|---|---|
| `MIC_LEVEL()` → FLOAT | highest peak since the last call, 0 to 1 — asked once per frame, that is a level meter that misses no click |
| `MIC_RMS()` → FLOAT | RMS of the last 50 ms — what you hear as loudness (a sine with peak 1 has 0.707) |
| `MIC_PITCH()` → FLOAT | fundamental in Hz, 50 to 2000; 0 if it is quiet or no tone stands out (noise, speech between two syllables) |
| `MIC_FFT(baender)` | fills an `ARRAY OF FLOAT` with band levels 0..1, spread logarithmically, with afterglow and its own gain — the same calculation as `AUDIO_FFT`, but from the input |
| `MIC_WAVE(werte)` | fills an `ARRAY OF FLOAT` with the most recent values (-1..1), oldest first; at most 8192 |

`MIC_PITCH` uses the YIN method and also finds the fundamental of tones with
many overtones (guitar, voice, sawtooth). The frequency becomes a note name
(69 is A4 at 440 Hz, 12 semitones per octave):

```basic
IMPORT "audio"
FUNCTION notenname$(n AS INTEGER) AS STRING
    DIM namen AS ARRAY OF STRING
    namen = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "H"]
    RETURN namen[n MOD 12] + STR$(n \ 12 - 1)
END FUNCTION
MIC_START()
WHILE TRUE
    DIM f AS FLOAT : f = MIC_PITCH()
    IF f > 0 THEN
        DIM n AS FLOAT : n = 69 + 12 * LOG(f / 440) / LOG(2)
        PRINT notenname$(ROUND(n)); "  "; FORMAT$(f, "%.1f"); " Hz"
    END IF
    SLEEP(100)
WEND
```

## Recording

| Command | What it does |
|---|---|
| `MIC_RECORD_START([max_sekunden])` | record from now on, at most `max_sekunden` (default 600, at most 3600); a running recording starts over |
| `MIC_RECORDING()` → BOOLEAN | is a recording running? |
| `MIC_RECORD_STOP()` → SOUND | end the recording and return it as a sound — play it, mix it, save it with `AUDIO_SAVE_WAV`. Needs sound output |
| `MIC_RECORD_SAVE(pfad$)` → INTEGER | end the recording and save it as WAV (16 bit, one channel); returns the number of values. Works without sound output too |

```basic
IMPORT "audio"
MIC_START()
MIC_RECORD_START(5)
SLEEP(5000)
DIM s AS SOUND
s = MIC_RECORD_STOP()
PLAYSOUND(s)
```

## Permissions

- **Windows**: *Settings → Privacy & security → Microphone* — access for
  desktop apps must be on. If it is off, the input stays
  silent.
- **macOS** asks on the first `MIC_START`. The Drachenhauch app brings the
  reason text and the entitlement for it; whoever starts `dhrt` in the
  terminal gets the question for the terminal.
- **Linux**: the user needs access to `/dev/snd` (group `audio`, on the usual
  systems from the start).

## Testing without a microphone

`DHRT_MIKROFON=datei.wav` turns a WAV file into a microphone: it is then the
only device (`Datei: name.wav`) and is fed in real time and in a loop —
through the same path as the values of a real device. That is how
`tests/pruef/ton/mikrofon.dhtest` tests level, pitch, spectrum, waveform and
recording on computers without an input too.

## Limits

- One input at a time, one channel (stereo is averaged).
- There is no microphone in the browser yet.
- The sample rate is the device's; nothing is converted.
