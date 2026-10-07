# Audio modulators: LFO and tweener

A **modulator** is a value that changes by itself and thereby drives an audio
parameter. Instead of recalculating every frame, you say once *what* should
move *how* — Kira does the rest on the audio thread.

That is not only more convenient but also better: the movement continues
**sample-accurately**, even when the frame rate drops. A tremolo that is set
every frame in the BASIC loop audibly stutters at a hitch.

There are two kinds, both with the same handle type `AUDIO_MOD`:

| | oscillates | moves to a target | typical for |
|---|---|---|---|
| **LFO** | yes, endlessly | no | tremolo, vibrato, auto-wah, wobble bass |
| **Tweener** | no | yes, on command | ducking, filter sweep on a level change |

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `AUDIO_LFO_NEW(wellenform$, hz [, amplitude [, mitte]])` | AUDIO_MOD | create an oscillating modulator (`sine`, `triangle`, `saw`, `pulse`) |
| `AUDIO_LFO_SET(mod [, hz [, amplitude [, mitte]]])` | — | change the oscillation while running; omitted values stay |
| `AUDIO_LFO_WAVEFORM(mod, wellenform$)` | — | switch the waveform |
| `AUDIO_TWEENER_NEW([startwert])` | AUDIO_MOD | modulator that moves to a target on command |
| `AUDIO_TWEENER_TO(mod, ziel [, dauer_ms [, easing$]])` | — | move there (`linear`, `in`, `out`, `inout`) |
| `AUDIO_MODULATE(bus$, ziel$, mod, min, max)` | — | bind a modulator to a bus parameter; its range is mapped to `min..max` |
| `AUDIO_MOD_REMOVE(mod)` | — | free the modulator — the parameter stays at its last value |

## LFO

```basic
IMPORT "audio"
DIM wah AS AUDIO_MOD
wah = AUDIO_LFO_NEW("sine", 1.5)          ' 1.5 oscillations per second
AUDIO_MODULATE("sfx", "filter", wah, 200.0, 6000.0)
```

With this the filter cutoff of the SFX bus wanders continuously between 200
and 6000 Hz — without a single line in the main loop.

- `AUDIO_LFO_NEW(wellenform$, hz [, amplitude [, mitte]])` → `AUDIO_MOD`
  Waveforms: `sine`, `triangle`, `saw`, `pulse` (the German names `sinus`,
  `dreieck`, `saegezahn`, `rechteck` work too). The LFO oscillates between
  `mitte - amplitude` and `mitte + amplitude`; the default is amplitude 1 and
  centre 0, i.e. **-1 to +1**.
- `AUDIO_LFO_SET(modulator [, hz [, amplitude [, mitte]]])` — change at run time.
  Omitted values stay unchanged.
- `AUDIO_LFO_WAVEFORM(modulator, wellenform$)`

## Tweener

```basic
DIM hall AS AUDIO_MOD
hall = AUDIO_TWEENER_NEW(0.0)
AUDIO_MODULATE("sfx", "reverb", hall, 0.0, 0.9)
AUDIO_TWEENER_TO(hall, 1.0, 1200.0, "inout")   ' fade in smoothly over 1.2 s
```

- `AUDIO_TWEENER_NEW([startwert])` → `AUDIO_MOD`
- `AUDIO_TWEENER_TO(modulator, ziel, dauer_ms [, easing$])`
  Easings as with the fades: `linear`, `in`, `out`, `inout`.

## Binding and releasing

```basic
AUDIO_MODULATE(bus$, ziel$, modulator, min, max)
AUDIO_MOD_REMOVE(modulator)
```

- `bus$`: `sfx`, `music` or `master`
- `ziel$`:
  - `volume` — **tremolo**. `min`/`max` are a factor as with
    `AUDIO_BUS_VOLUME` (1.0 = unchanged), not decibels.
  - `pan` — **auto-pan**. -1 = fully left, +1 = fully right.
  - `filter` (cutoff in Hz), `resonance`, `reverb` (mix 0..1),
    `distortion` (amount in dB)

A fixed balance without a modulator: `AUDIO_BUS_PAN(bus$, pos)`. Until then
only a *single* channel could be panned (`AUDIO_PAN`), not the music as a
whole.

> In Kira, panning does not sit on the track but in an effect of its own.
> Since stage 6 it hangs in every bus chain — without it there would be
> neither `AUDIO_BUS_PAN` nor auto-pan.

The modulator's range is mapped to `min..max` — with an LFO at the default
amplitude that means **-1 → min** and **+1 → max**. If you build an LFO with
`amplitude = 0.5`, you accordingly only use the middle half of the range.

## Two things worth knowing

**LFO and tweener share the handle type.** That keeps `AUDIO_MODULATE`
simple — it takes both. But if you call an LFO function on a tweener (or vice
versa), the message says in plain words what is going on instead of just
“invalid handle".

**`AUDIO_MOD_REMOVE` frees the modulator**, but the binding to the parameter
stays at the last value. If you want a fixed value again, set it directly
(`AUDIO_FILTER("sfx", 2000.0)`).

## Demo and tests

- `examples/150_audio_modulatoren.dh` — auto-wah with a switchable waveform
  and a tweener on the reverb, with a spectrum display.
- `tests/pruef/audio_modulators.dhtest` — the core test **measures** through
  the FFT tap that the treble share of the output really fluctuates (measured:
  without modulation 0.065–0.218, with modulation 0.736–0.830; the threshold
  0.45 lies in the middle). A counter-check without modulation makes sure the
  test does not merely measure the noise's own jitter.
