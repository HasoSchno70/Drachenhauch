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
| Waveform | The curve of the current sound (`AUDIO_SOUND_WAVE`), the largest amplitude per section. If it reaches the edge, **Clipping** appears next to it |
| Transport | **Abspielen** (Play); **Zufall** (Random) rolls new values *around the current ones* (volume and pan stay), plus a random waveform; **Auto-Play** (on) plays by itself as soon as a slider has come to rest for six frames — not at every intermediate value |
| Presets | Muenze, Laser, Explosion, Powerup, Treffer, Sprung, Blip, Motor (coin, laser, explosion, power-up, hit, jump, blip, engine) — one click sets all sliders and the waveform |
| Output | **WAV sichern ...** (Save WAV), **DH-Code kopieren** (Copy DH code), **Sichern ...**/**Laden ...** (Save/Load, `.ini`), checkbox **8 Bit (Lo-Fi)** for the WAV |
| History | **Zurueck**/**Vor** (Back/Forward) or Ctrl+Z/Ctrl+Y |
| Status line | Message and the duration of the sound (attack + sustain + decay in ms) along with the waveform |

The sliders are in four groups; their order is also the order of the
arguments of `AUDIO_SFX`:

| Group | Sliders |
|---|---|
| Tone | waveform (`square`, `saw`, `sine`, `triangle`, `noise`), frequency (50..8000 Hz), pitch slide (±8000 Hz/s, negative = falling), volume |
| Envelope | attack (0..2000 ms), sustain and decay (0..4000 ms) |
| SID / filter | pulse width (0.05..0.95; affects `square`), PWM depth, PWM speed, filter (cutoff frequency, 0 = off), filter sweep (Hz/s), resonance |
| Vibrato / stereo | vib depth, vib speed, stereo (width, 0 = mono), pan (L..R) |

Everything also works without a mouse: TAB moves between controls, Space and
Enter trigger, the arrow keys adjust a slider.

## Undo

One **drag** on a slider is **one** step, not sixty per second: it is only
recorded once nothing has changed for one frame and no mouse button is held
any more. A preset, a dice roll and a loaded `.ini` set all values at once
and are one step each the same way. The full state is remembered each time
(16 sliders and the waveform), up to 32 steps. A new drag after an undo cuts
off the way forward.

## Saving and loading

**Sichern ...** (Save) writes all sliders into an `.ini`, **Laden ...**
(Load) brings them back. The key is the slider's **label** (`Frequenz`,
`Pitch-Slide`, `Resonanz`, …), plus `wellenform` — so the file can be read
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
recording: labels do not lie on top of the buttons, one drag is one step,
back/forward via button and keys, a dice roll and a preset are one step each,
a new drag cuts off the way forward — and the DH code takes the pan along and
passes `dhrt --check`. The prompt on closing is checked by
`tests/pruef/werkzeug_kreuz.dhtest` with real window messages.
