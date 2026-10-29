# SFX generator (`examples/183_sfx_generator.dh`)

A tool for retro sound effects in sfxr style, written in Drachenhauch on its
own `gui` module. It builds its sound with [`AUDIO_SFX`](module-audio.md) —
that is, with exactly the call that will later be in the game —, shows the
waveform, plays it and hands it out as a WAV or as finished DH code. The
counterpart of the particle editor
([`particle-editor.md`](particle-editor.md)); music is made with the
[tracker](tracker.md).

```
dhrt run examples/183_sfx_generator.dh
```

In the [IDE](ide.md) it is in the **Tools** menu and runs there as a separate
program. It does not need Python.

## The window

| Area | What it does |
|---|---|
| Waveform | Large in the middle: the envelope of the sound (`AUDIO_SOUND_WAVE`, the largest amplitude per column) over a time axis in ms, with the **envelope** on top as a yellow line with three points (A, S, D) — they can be dragged with the mouse, one drag is one step. A click elsewhere plays; while playing a line runs along, and a bar on the right shows the output level (`AUDIO_PEAK`). If the wave reaches the edge, **Übersteuert** (clipping) appears next to it |
| Toolbar and menu | **Öffnen**/**Sichern** (open/save `.ini`, Ctrl+O/Ctrl+S), **WAV** (Ctrl+E), **DH-Code** (Ctrl+Shift+C), **Abspielen** (play, Space), **Stopp** (Shift+Space), **Zurück**/**Vor** (Ctrl+Z/Ctrl+Y); below it **Auto-Play** (plays by itself as soon as a slider has come to rest for six frames) and **WAV mit 8 Bit (Lo-Fi)** |
| Presets | Münze, Laser, Explosion, Powerup, Treffer, Sprung, Blip, Motor, Schuss, Klick, Glocke, Alarm (coin, laser, explosion, power-up, hit, jump, blip, engine, shot, click, bell, alarm) — one click sets all sliders and the waveform. Every preset is also a **kind** |
| Random and mutate | **Zufall** (R) rolls a new sound of the selected kind — within the limits that make a laser a laser (volume and pan stay). **Mutieren** (M) shifts every value that already does something by at most 6 % of its range; the waveform stays |
| History | List at the bottom left: every step with a name ("Vorlage: Laser", "Zufall: Laser", "Mutiert", "Tonrutsch", "Hüllkurve"), a click brings it back; the way forward is dimmed |
| Status bar | Message, duration of the sound (attack + hold + decay in ms) with the waveform, step in the history |

The window can be resized freely; the waveform grows with it, presets and
sliders stay in place. The font is DejaVu Sans (`LOADFONT("sans")`) instead of
the pixel font.

The sliders are in four groups; their order is also the order of the
arguments of `AUDIO_SFX`:

| Group | Sliders |
|---|---|
| Tone | waveform (`square`, `saw`, `sine`, `triangle`, `noise`), frequency (50..8000 Hz), Tonrutsch (pitch slide, ±8000 Hz/s, negative = falling), volume |
| Envelope | Anschwellen (attack, 0..2000 ms), Halten (sustain) and Ausklingen (decay, each 0..4000 ms) — also draggable in the waveform |
| SID / filter | pulse width (0.05..0.95; affects `square`), PWM depth, PWM speed, filter (cutoff frequency, 0 = off), filter sweep (Hz/s), resonance |
| Vibrato / stereo | vibrato depth, vibrato speed, stereo width (0 = mono), Links/Rechts (pan) |

Everything also works without a mouse: TAB moves between controls, Enter
triggers, the arrow keys adjust a slider; every slider explains itself in a
tooltip.

## Undo

One **drag** on a slider is **one** step, not sixty per second: it is only
recorded once nothing has changed for one frame and no mouse button is held
any more. A preset, a dice roll and a loaded `.ini` set all values at once
and are one step each the same way. The full state is remembered each time
(16 sliders and the waveform), up to 32 steps. A new drag after an undo cuts
off the way forward.

## Saving and loading

**Sichern ...** (Save) writes all sliders into an `.ini`, **Laden ...**
(Load) brings them back. The key is the slider's **name** in the earlier
spelling (`Frequenz`, `Pitch-Slide`, `Lautstaerke`, `Attack`, …, even where
the display now says something else — old files keep loading), plus
`wellenform` — so the file can be read
and edited by hand. An unknown key is skipped instead of reported, and a
value outside the slider's range is clamped to the range.

On closing (close button, Alt+F4 or ESC) the generator asks
(Sichern / Verwerfen / Abbrechen — Save / Discard / Cancel), but only if the
sliders are not where they were at the last save, load or choice of a
preset. Anyone who sets them back to exactly that state is not asked.

## Output

**WAV sichern ...** (Save WAV) writes the sound with `AUDIO_SAVE_WAV`
(16 bit, 8 bit with the checkbox). Any program loads the file with
`LOADSOUND`:

```basic
DIM snd AS SOUND
snd = LOADSOUND("effekt.wav")
PLAYSOUND(snd)
```

**DH-Code kopieren** (Copy DH code) puts a runnable snippet onto the
clipboard: the `IMPORT`, the `AUDIO_SFX` call with all 16 values and the
playback. The sound is then created at run time; the game needs no WAV. For
the preset "Muenze" (coin):

```basic
IMPORT "audio"

DIM snd AS SOUND
snd = AUDIO_SFX("square", 900, 600, 0, 40, 160, 0.00, 0, 0.70, 0.00, 0.50, 0.00, 0.0, 0, 0, 0.00)
PLAYSOUND(snd)
```

If the pan slider is not in the centre, it comes along. It belongs to the
**playback channel**, not to the sound — `AUDIO_SFX` only knows the stereo
width —, and without it the copied code would sound different from the
preview. Instead of `PLAYSOUND` the code then reads (here pan 0.5):

```basic
IMPORT "audio"

DIM snd AS SOUND
snd = AUDIO_SFX("square", 900, 600, 0, 40, 160, 0.00, 0, 0.70, 0.00, 0.50, 0.00, 0.0, 0, 0, 0.00)
DIM ch AS AUDIO_CHANNEL
ch = AUDIO_PLAY(snd)
AUDIO_PAN(ch, 0.50, 1.00)
```

What the individual arguments do is described under `AUDIO_SFX` in
[`module-audio.md`](module-audio.md).

## Compared with the old Qt version

The former Qt version (`dhsfx`) had a **preset library** with named custom
sounds in one shared file. Here every custom sound is a separate `.ini` that
you store wherever you like.

## Tests

`tests/pruef/werkzeug_sfx.dhtest` drives real mouse clicks and keys from a
recording: no control lies on top of another or past the edge, one drag is
one step, back/forward via toolbar and keys, a dice roll and a preset are one
step each, a new drag cuts off the way forward, random stays with the kind,
mutating is one step, a click in the history brings a state back, the end
point of the envelope can be dragged — and the DH code takes the pan along and
passes `dhrt --check`. The prompt on closing is checked by
`tests/pruef/werkzeug_kreuz.dhtest` with real window messages.
