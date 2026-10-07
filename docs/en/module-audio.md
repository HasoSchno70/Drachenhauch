# Module `audio`

Extended audio API (native in the runtime `dhrt` via **Kira**/cpal — its own audio thread, decoupled from the game loop). Provides the typical game-engine building blocks: channels (per-sound control), pause/resume, fade in/out, stereo pan, music position, plus tone generation for procedural sounds.

Complements the core built-ins `LOADSOUND` / `PLAYSOUND` from [Graphics built-ins](builtins-grafik.md) — the simple calls are enough for "play a sound", `audio` brings the full audio-mixing toolkit.

```basic
IMPORT "audio"
```

## Mixer lifecycle

The audio system starts automatically with the first sound call. The sample
rate is determined by the **output device** -- since the switch to [Kira](https://github.com/tesselode/kira)
(2026-06-13, before that raylib audio) there is no fixed format any more that a
program could set.

| Function | Effect |
|---|---|
| `AUDIO_INIT([...])` | makes sure the audio system is running. **Accepts arguments and ignores them** -- they date from the mixer era and are only tolerated for compatibility |
| `AUDIO_SET_NUM_CHANNELS(n)` | number of simultaneous channels (default **16**, raise it for bullet hell) |
| `AUDIO_NUM_CHANNELS()` → INTEGER | current channel count |
| `AUDIO_BUSY_CHANNELS()` → INTEGER | how many channels are playing right now |

```basic
AUDIO_INIT(48000, 2, 256)         ' low latency for action
AUDIO_SET_NUM_CHANNELS(32)        ' 32 sounds possible at once
```

## Sound playback

`AUDIO_PLAY(sound, [loops, volume, fade_in_ms])` → `AUDIO_CHANNEL`

Plays a SOUND and returns a channel handle. With it you can pause the sound, stop it, change its volume and so on.

`loops` follows pygame semantics: `0` plays once (default), `N` repeats N times (i.e. N+1 passes), `-1` loops endlessly. `fade_in_ms` fades in from 0 to `volume`; `AUDIO_STOP(ch, fade_out_ms)` fades out accordingly and stops at the end of the fade. Both fades run non-blocking via the frame update (FLIP).

| Function | Effect |
|---|---|
| `AUDIO_PLAY(s, [loops, vol, fade_in_ms])` → AUDIO_CHANNEL | start a sound |
| `AUDIO_PAUSE(ch)` | pause the channel |
| `AUDIO_RESUME(ch)` | continue |
| `AUDIO_STOP(ch[, fade_out_ms])` | stop the channel, optionally fade out |
| `AUDIO_IS_PLAYING(ch)` → BOOLEAN | Is the channel playing right now? |
| `AUDIO_VOLUME(ch, vol)` | set the volume 0..1 |
| `AUDIO_GET_VOLUME(ch)` → FLOAT | current volume |
| `AUDIO_PAN(ch, left_vol, right_vol)` | stereo pan (both 0..1) |
| `AUDIO_PITCH(ch, faktor)` | pitch/speed: 1.0 = normal, 2.0 = an octave higher, 0.5 = lower |

**Global pause/resume:**

| Function | Effect |
|---|---|
| `AUDIO_PAUSE_ALL()` | all channels |
| `AUDIO_RESUME_ALL()` | all continue |
| `AUDIO_STOP_ALL()` | stop all |

## Example: bullet-hell SFX with fade-out

```basic
IMPORT "audio"

DIM laser AS SOUND
laser = LOADSOUND("assets/laser.wav")

DIM ch AS AUDIO_CHANNEL
WHILE shooting
    ch = AUDIO_PLAY(laser, 0, 0.7)     ' loop=0, vol=0.7
    SLEEP(120)
WEND

' Fade out at the end of the game:
AUDIO_STOP(ch, 500)                     ' 500 ms fade-out
```

## Stereo pan

`AUDIO_PAN(ch, left, right)` sets the volume of the two channels independently. That is how you build positional 3D sound: an enemy on the left of the screen sounds left.

```basic
' Sound localized to the left:
ch = AUDIO_PLAY(footstep)
AUDIO_PAN(ch, 1.0, 0.2)                 ' almost only left

' Compute the pan from the position:
DIM pan_left AS FLOAT
DIM pan_right AS FLOAT
pan_left  = 1.0 - (enemy_x / screen_w)
pan_right = enemy_x / screen_w
AUDIO_PAN(ch, pan_left, pan_right)
```

**Pan position + automatic movement:** For wandering sounds there are three
convenience built-ins that work with a single position (0 = left,
0.5 = centre, 1 = right) and only touch the pan -- the volume (and with it
running fades) stays untouched. The runtime drives the movement itself every
frame (FLIP), non-blocking:

| Function | Effect |
|---|---|
| `AUDIO_PAN_POS(ch, p)` | set the position directly (ends a running animation) |
| `AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms)` | one-time move from → to; stays at the target |
| `AUDIO_AUTOPAN(ch, periode_s[, tiefe])` | endless swinging left↔right (starts left); `tiefe` 0..1 = deflection around the centre, `periode_s <= 0` switches it off |

```basic
DIM ton AS SOUND
ton = AUDIO_TONE(440, 2000)

DIM ch AS AUDIO_CHANNEL
ch = AUDIO_PLAY(ton, -1, 0.8)           ' loop endlessly

AUDIO_AUTOPAN(ch, 6.0)                  ' swings left<->right every 6 s
' ... at some point:
AUDIO_PAN_SLIDE(ch, 0.0, 1.0, 2000)     ' wander to the right once in 2 s
AUDIO_AUTOPAN(ch, 0)                    ' swinging off (position stays)
```

Manual `AUDIO_PAN`/`AUDIO_PAN_POS` wins: it ends a running SLIDE/AUTOPAN
animation. Another `AUDIO_PLAY`/`AUDIO_STOP` also resets the animation.

## Music

The native runtime has a separate music channel for long tracks (loads by streaming instead of completely into RAM). Its own API:

| Function | Effect |
|---|---|
| `AUDIO_MUSIC_LOAD(path$)` | load a track (not playing yet) |
| `AUDIO_MUSIC_PLAY([loops[, fade_in_ms]])` | start (loops = -1 = endless = default; loops = N → N+1 passes) |
| `AUDIO_MUSIC_STOP([fade_out_ms])` | stop, optionally fade out (non-blocking) |
| `AUDIO_MUSIC_PAUSE()` | pause |
| `AUDIO_MUSIC_RESUME()` | continue |
| `AUDIO_MUSIC_VOLUME(vol)` | 0..1 |
| `AUDIO_MUSIC_GET_VOLUME()` → FLOAT | current |
| `AUDIO_MUSIC_PITCH(faktor)` | music pitch (1.0 = normal; survives LOAD/QUEUE — slow-motion effect) |
| `AUDIO_MUSIC_GET_PITCH()` → FLOAT | current pitch |
| `AUDIO_MUSIC_POSITION()` → FLOAT | seconds since start (from position 0) |
| `AUDIO_MUSIC_SEEK(sekunden)` | jump to this point (seconds from the beginning) |
| `AUDIO_MUSIC_BUSY()` → BOOLEAN | Playing right now? |
| `AUDIO_MUSIC_QUEUE(path$)` | next track, as soon as the current one ends |

**Seeking (`AUDIO_MUSIC_SEEK`)** — three things you want to know beforehand:

```basic
AUDIO_MUSIC_LOAD("lied.ogg")
AUDIO_MUSIC_PLAY()
AUDIO_MUSIC_SEEK(30.0)             ' continue from second 30
```

* **Play first, then seek.** Before `AUDIO_MUSIC_PLAY()` there is nothing to move yet; the call then reports an error instead of silently doing nothing (otherwise the piece would start at 0 again without anyone noticing).
* **It does not take effect immediately.** Measured, it takes ~0.3–0.4 s until `AUDIO_MUSIC_POSITION()` shows the new position — that long the stream still plays out its pre-buffer. If you query the position right after the jump, you get the old one.
Demo: [examples/160_musik_seek.dh](../../examples/160_musik_seek.dh) — a clickable progress bar; the red line shows the target position, and you can see the bar catch up with it.

* **MOD/XM cannot do it** and say so. Their timeline is patterns and rows, not seconds; how long it takes to reach a point depends on tempo changes in the piece itself. `AUDIO_MUSIC_POSITION()` still returns seconds for modules — counting along works, jumping to a point does not.

**Formats:** `.ogg`, `.mp3`, `.qoa` — **and tracker modules `.mod` (ProTracker/Amiga) + `.xm` (FastTracker II)**. Modules contain their own samples + pattern data and are **streamed in real time**: a Kira sound of its own (`ModuleSound` in `audio.rs`) drives the pure Rust player [`xmrs`/`xmrsplayer`](https://crates.io/crates/xmrs) on the audio thread -- 4+ channels, sample-based, with exact endless looping. (Until 2026-06-13 raylib handled this; since the switch to Kira it runs through its own player.) Simply load a `.mod`/`.xm` (e.g. from [modarchive.org](https://modarchive.org)):

```basic
AUDIO_MUSIC_LOAD("song.mod")
AUDIO_MUSIC_PLAY(-1)               ' loops -- real Amiga sound

' or via the core built-in:
PLAYMUSIC("song.xm", -1, 1.0)
```

`AUDIO_FFT` also taps the running mix with module music — ideal for reactive visualizers. Demo: [examples/115_modplayer.dh](../../examples/115_modplayer.dh) (module player with spectrum + drag & drop for your own module).

**Crossfade between tracks:**

```basic
AUDIO_MUSIC_LOAD("music/level1.ogg")
AUDIO_MUSIC_PLAY(-1, 1000)              ' fade-in 1s, loop

' At the boss entrance: let it fade out, then switch. The fade runs
' non-blocking in the game loop (FLIP drives it) -- an immediate
' AUDIO_MUSIC_LOAD would cut it off hard.
AUDIO_MUSIC_STOP(800)                   ' fade-out 800ms
WHILE AUDIO_MUSIC_BUSY()
    FLIP()                              ' keep frames running
WEND
AUDIO_MUSIC_LOAD("music/boss.ogg")
AUDIO_MUSIC_PLAY(-1, 800)
```

## Tone generation

For procedural sounds without audio files. Returns a `SOUND` object that you can use like a normal mixed SOUND (also with `PLAYSOUND` from the core):

| Function | Effect |
|---|---|
| `AUDIO_TONE(freq_hz, dauer_ms[, waveform$[, vol]])` | sine/square/saw/triangle tone |
| `AUDIO_NOISE(dauer_ms[, vol])` | white noise |
| `AUDIO_NOTE(wf$, freq, dauer_ms, attack_ms, decay_ms, sustain, release_ms, vol[, vib_depth, vib_speed, detune_cents, slide_halbtoene])` | a held note with a real ADSR envelope, vibrato, detune layer and portamento — the instrument for trackers and music |

**Waveforms** (case-insensitive): `"sine"`, `"square"`, `"saw"`, `"triangle"`, `"noise"`.

```basic
DIM beep AS SOUND
beep = AUDIO_TONE(440, 200)                ' A 440Hz, 200ms, sine
PLAYSOUND(beep)

DIM laser AS SOUND
laser = AUDIO_TONE(800, 80, "square", 0.5) ' square wave, quiet
PLAYSOUND(laser)

DIM explosion AS SOUND
explosion = AUDIO_NOISE(400, 0.8)
PLAYSOUND(explosion)
```

Generated sounds automatically get a short fade-in/out (5 ms) against clicks at the start/end.

### `AUDIO_SFX` (sfxr style + SID character)

The procedural effects synth with pitch slide, ADSR, vibrato, stereo width —
and **SID extensions** (pulse width/PWM + resonant low-pass sweep). The SID
arguments are all optional; omitting them reproduces exactly the previous
sound. The most convenient way to build `AUDIO_SFX` calls is in the
**[SFX generator](sfx-generator.md)** (`examples/183_sfx_generator.dh`) and
copy the DH code.

```
AUDIO_SFX(waveform$, freq, slide, attack_ms, sustain_ms, decay_ms,
          vib_depth, vib_speed, vol
          [, stereo_width, duty, pwm_depth, pwm_speed,
           flt_cutoff, flt_sweep, flt_res])
```

| Argument | Effect |
|---|---|
| `duty` | pulse width of the `square` wave (0.05..0.95; **0.5 = symmetric = as before**). Narrower = thinner/more nasal SID pulse. |
| `pwm_depth` / `pwm_speed` | pulse-width **modulation** (PWM): the pulse width swings around `duty` at `pwm_speed` Hz — the typical SID shimmer. |
| `flt_cutoff` | cutoff frequency of the resonant low-pass in Hz (**0 = off**). |
| `flt_sweep` | cutoff movement in Hz/s over the note (e.g. -8000 = acid sweep downwards). |
| `flt_res` | resonance 0..0.95 — emphasises the cutoff frequency (SID/TB-303 character). |

```basic
' SID pulse bass with PWM + filter sweep:
DIM s AS SOUND
s = AUDIO_SFX("square", 110, 0, 0, 600, 200, 0, 0, 0.8, _
              0.0, 0.25, 0.15, 5.0, 4000, -7000, 0.7)
PLAYSOUND(s)
```

### `AUDIO_NOTE` (a note, not an effect)

`AUDIO_SFX` knows three **times** (attack, sustain, decay) that together make
up the duration — exactly right for an effect, not for an instrument: there a
note is **held** as long as the player wants and then fades out. That is what
`AUDIO_NOTE` is for.

```
AUDIO_NOTE(waveform$, freq, dauer_ms, attack_ms, decay_ms, sustain, release_ms, vol
           [, vib_depth, vib_speed, detune_cents, slide_halbtoene])
```

| Argument | Effect |
|---|---|
| `dauer_ms` | the **held** time (until the next note) |
| `attack_ms` / `decay_ms` | rise to full, then fall to the sustain level |
| `sustain` | the level 0..1 that **remains** afterwards — at `0.0` the note falls silent after the decay, no matter how long it is held (piano); at `1.0` it stays full (organ) |
| `release_ms` | the fade-out — it is **appended at the end**, so the sound is `dauer_ms + release_ms` long and overlaps the next note as on a real synthesizer |
| `vib_depth` / `vib_speed` | vibrato as with `AUDIO_SFX` |
| `detune_cents` | a second layer underneath, detuned by that many cents (chorus) — turns a bare waveform into an instrument |
| `slide_halbtoene` | portamento: the pitch glides to the target over the held time, in semitones and exponentially (musically even), not in Hz/s as with `AUDIO_SFX` |

```basic
' A piano tone: fast attack, fades out by itself
DIM piano AS SOUND
piano = AUDIO_NOTE("triangle", 261.6, 500, 2, 700, 0.0, 200, 0.8, 0, 0, 6)
' An organ: stays as long as it is held
DIM orgel AS SOUND
orgel = AUDIO_NOTE("square", 261.6, 500, 6, 0, 1.0, 70, 0.6, 0.06, 6, 5)
```

The tracker in Drachenhauch (`examples/190_tracker.dh`) builds every note this way.

## Mixing sounds

Up to here every sound was a one-way street: build, play, save. Turning two
sounds into **one** was not possible — and that is exactly what a music
program needs that wants to deliver a song as a WAV.

| Function | Effect |
|---|---|
| `AUDIO_SOUND_NEW(dauer_ms)` → SOUND | silence of the given length, as a canvas |
| `AUDIO_SOUND_MIX(ziel, quelle, offset_ms[, vol[, pan]])` | **add** the source into the target from `offset_ms` |
| `AUDIO_SOUND_NORMALIZE(sound[, spitze])` → FLOAT | bring the loudest point to `spitze` (default 1.0); returns the factor |

Three things you need to know:

1. **There is no clamping.** Two loud sounds on top of each other give values
   above 1.0, and they stay — clamping in between would distort every
   overlay. Before playing or saving, `AUDIO_SOUND_NORMALIZE` therefore comes
   first.
2. **Whatever sticks out past the end is dropped.** A fade-out behind the
   last bar is normal, not an error.
3. **`pan` distributes the source as mono** (-1 left, 0 centre, +1 right,
   equal power). Without `pan` its own channels stay as they are — a stereo
   sound stays stereo, a mono sound stays in the centre at full volume.

A source with a different sample rate (a 48 kHz file) is converted while
mixing. If you mix into a sound that is currently **playing**, the runtime
works on a copy — the audio thread must not see the buffer change beneath it.

```basic
DIM song AS SOUND
song = AUDIO_SOUND_NEW(2000)                       ' 2 s canvas
AUDIO_SOUND_MIX(song, AUDIO_TONE(440, 500), 0)     ' A at 0 ms
AUDIO_SOUND_MIX(song, AUDIO_TONE(554, 500), 500, 0.8, -0.5)   ' C sharp at 500 ms, half left
AUDIO_SOUND_NORMALIZE(song)
AUDIO_SAVE_WAV(song, "akkord.wav")
```

## Looking at a sound and saving it

Two functions on the **SOUND handle** — so they apply to every sound source
(`AUDIO_TONE`, `AUDIO_NOISE`, `AUDIO_SFX`, loaded files), not just the
synthesizer.

| Function | Effect |
|---|---|
| `AUDIO_SOUND_WAVE(sound, anzahl)` → ARRAY OF FLOAT | `anzahl` points of the waveform, `-1..1` — for drawing |
| `AUDIO_SAVE_WAV(sound, pfad$[, bits])` | save as a WAV file (`bits` = 16, the default, or 8) |

`AUDIO_SOUND_WAVE` returns, per section, the sample with the **largest
magnitude**, with its sign — not the average. That is the whole trick:
averaged over 44100 values onto 600 pixels, an oscillation cancels out to
zero, and the display would show a line instead of a wave.

```basic
DIM s AS SOUND
s = AUDIO_SFX("square", 900, 600, 0, 40, 160, 0, 0, 0.7)

' Draw the waveform
DIM kurve AS ARRAY OF FLOAT
kurve = AUDIO_SOUND_WAVE(s, 300)
DIM i AS INTEGER
FOR i = 0 TO 298
    LINE(i, 100 - INT(kurve[i] * 90), i + 1, 100 - INT(kurve[i + 1] * 90), CYAN)
NEXT

' and store it as a file -- LOADSOUND reads it straight back
AUDIO_SAVE_WAV(s, "effekt.wav")
```

The frames are written **as they are**: the volume from `AUDIO_SFX` is
already in them. The playback volume of the last `AUDIO_PLAY` is *not* added
on top — otherwise a `volume` of `0.7` would end up as `0.49` in the file.
Mono stays single-channel; only when the left and right channels differ
(`stereo_width > 0`) does the file become stereo.

> The **SFX generator** in [`examples/183_sfx_generator.dh`](../../examples/183_sfx_generator.dh)
> uses both: it draws the curve live and saves the result as a WAV.
> It is itself written in Drachenhauch — on the `gui` module.

## Sampler (Amiga style): `SAMPLE_*`

Play a loaded PCM sample across the **whole keyboard** by resampling it --
higher note = played faster, just like **Paula** on the Amiga (and like
MOD/XM trackers do it). A single pluck/bass/drum sample thus becomes a whole
instrument.

| Function | Effect |
|---|---|
| `SAMPLE_LOAD(pfad$)` → SAMPLE | load WAV/OGG/QOA, normalized to mono |
| `SAMPLE_FROM_BUFFER(puffer, abtastrate)` → SAMPLE | sample from raw 16-bit PCM (little endian, signed, mono) |
| `SAMPLE_PLAY(sample, halbtoene, vol[, dur_ms])` → AUDIO_CHANNEL | play at a relative pitch |
| `SAMPLE_NOTE(sample, halbtoene, dauer_ms, attack_ms, decay_ms, sustain, release_ms, vol[, slide_halbtoene])` → SOUND | build a note as a sound (not play it), envelope like `AUDIO_NOTE` |
| `SAMPLE_SET_LOOP(sample, start, end[, art$])` | loop region in frames (for held notes), `art$` = `vorwaerts` (forwards, default) or `pingpong` |
| `SAMPLE_LEN(sample)` → FLOAT | length in seconds at the original pitch |

`halbtoene` is relative to the sample's original pitch: `0` = as recorded,
`12` = an octave higher, `-12` = an octave lower (fractional/float values too).
`dur_ms <= 0` plays the **whole sample once** (one-shot -- ideal for drums,
hits, plucks). `dur_ms > 0` builds a sound of fixed length: if a loop region
has been set via `SAMPLE_SET_LOOP`, it is repeated (held note), otherwise
silence follows after the end of the sample.

Resampled variants are **cached** (per `sample`/`halbtoene`/`dur_ms`), so a
repeated tone is cheap -- good for tracker-like players. The return value is
an `AUDIO_CHANNEL` -- so `AUDIO_PAN`, `AUDIO_VOLUME`, `AUDIO_STOP` etc. work
as with `AUDIO_PLAY`.

```basic
IMPORT "audio"
DIM pluck AS SAMPLE
pluck = SAMPLE_LOAD("assets/pluck.wav")     ' root note e.g. A3

SAMPLE_PLAY(pluck, 0, 0.8)                  ' original pitch
SAMPLE_PLAY(pluck, 12, 0.8)                 ' an octave higher
SAMPLE_PLAY(pluck, -5, 0.6)                 ' a fourth lower

' Held tone with a loop (e.g. frames 2000..8000 of the source):
SAMPLE_SET_LOOP(pluck, 2000, 8000)
SAMPLE_PLAY(pluck, 0, 0.7, 1000)            ' 1 s, loop region held
```

**`SAMPLE_NOTE` instead of `SAMPLE_PLAY`** when the sound should not start
immediately: it returns a `SOUND` that you place on an audio clock with
`AUDIO_PLAY_AT` or mix into another with `AUDIO_SOUND_MIX` — that is how the
tracker plays its sample instruments. The envelope is that of `AUDIO_NOTE`:
attack, decay to the sustain level, the fade-out is appended AT THE END
(length = `dauer_ms + release_ms`); the loop region holds the tone, without a
loop the note falls silent when the sample ends. `slide_halbtoene` glides to
the target pitch over the held time. The sound has 44100 Hz; the sample's
sample rate is already accounted for in the pitch.

`SAMPLE_FROM_BUFFER` takes samples that do not exist as a file — such as
Base64 in a JSON (`BUFFER_FROM_BASE64` before it). A value −32768..32767 is
divided by 32768; an odd length is an error.
**Ping-pong** runs the loop region backwards down again at its end instead of
jumping to its start — a sample whose ends do not match up does not click
that way.

```basic
DIM s AS SAMPLE
s = SAMPLE_FROM_BUFFER(BUFFER_FROM_BASE64(daten$), 22050)
SAMPLE_SET_LOOP(s, 1100, 2200, "pingpong")
DIM ton AS SOUND
ton = SAMPLE_NOTE(s, 7, 500, 5, 100, 0.6, 200, 0.8)   ' a fifth higher
AUDIO_PLAY_AT(ton, uhr, 16)
```

Demo: [examples/116_sampler.dh](../../examples/116_sampler.dh) — a pluck
sample plays a melody + bass across the whole keyboard (clickable).

> **Sample vs. module:** `SAMPLE_*` is the live primitive for triggering your
> own samples at variable pitch (sequencers, instruments, SFX variants). If you
> want a finished tracker piece, play a `.mod`/`.xm` via `PLAYMUSIC`
> (see above) -- it brings its own samples + patterns.

## Paula lo-fi (Amiga sound)

`AUDIO_LOFI(an[, bits[, cutoff_hz]])` switches on a **lo-fi mode** for all
sounds **synthesized** afterwards (`AUDIO_TONE`/`AUDIO_NOISE`/`AUDIO_SFX` +
`SAMPLE_PLAY`) -- the crunchy Amiga/Paula character:

- **Bit crush** to `bits` resolution (1..16, default **8** = Amiga).
- **LED low-pass** at `cutoff_hz` (default **3300 Hz** -- the famous
  Amiga 500 filter; `0` = off).

The chain runs in the order of the real Amiga: first 8-bit quantization
(DAC), then the analogue low-pass. It only affects **newly built** sounds --
the sample cache is cleared when switching, loaded files (`LOADSOUND`) and
music stay untouched.

```basic
AUDIO_LOFI(TRUE)                 ' 8-bit + 3.3 kHz -- classic Amiga sound
AUDIO_LOFI(TRUE, 4)              ' even crunchier (4-bit)
AUDIO_LOFI(TRUE, 8, 0.0)        ' 8-bit, filter off (raw bit crush)
AUDIO_LOFI(FALSE)                ' back to hi-fi
```

Demo: in [examples/116_sampler.dh](../../examples/116_sampler.dh), switchable
with `L` (A/B comparison hi-fi vs. Paula).

## Mixer buses (SFX/music master)

Thanks to Kira's mixer tracks, all SFX/sampler/synth sounds run on an
**SFX bus** and all music on a **music bus** (both flow into the **master**).
That way you control effects and music separately with one master fader,
independent of the individual volumes (they multiply in the mixer).

| Function | Effect |
|---|---|
| `AUDIO_BUS_VOLUME(bus$, vol)` | master volume of a bus (0..1) |
| `AUDIO_BUS_GET_VOLUME(bus$)` → FLOAT | current bus volume |

`bus$` is `"sfx"`, `"music"`, `"master"` or `"speech"` (case-insensitive;
`speech` carries the speech output of `SPEAK`, see below, and only has a
volume, no effects chain). Typical use: an options menu with separate
sliders.

```basic
AUDIO_BUS_VOLUME("music", 0.4)        ' music quieter
AUDIO_BUS_VOLUME("sfx", 0.8)          ' effects down a little
AUDIO_BUS_VOLUME("master", 0.0)       ' everything muted (pause menu)
PRINT AUDIO_BUS_GET_VOLUME("music")   ' 0.4
```

The `AUDIO_FFT` tap hangs on the master, so it still captures the whole mix.

## Real-time effects (filter / reverb / delay)

Every bus (`sfx`, `music`, `master`) has a **real-time effects chain** on the
audio thread — no buffer bake like `AUDIO_LOFI`, but real DSP that also
captures running/streamed sounds and can be controlled live.

| Function | Effect |
|---|---|
| `AUDIO_FILTER(bus$, cutoff_hz[, resonance])` | low-pass. `cutoff_hz` 20..20000 (≤0 or ≥20000 = open/off), `resonance` 0..1 (emphasis at the cutoff — the “weeoow" SID/acid character) |
| `AUDIO_REVERB(bus$, mix[, feedback[, damping]])` | reverb. `mix` 0..1 (0 = off), `feedback` 0..1 (reverb length, default 0.9), `damping` 0..1 (treble damping, default 0.1) |
| `AUDIO_DELAY(bus$, mix[, feedback[, time_ms]])` | echo. `mix` 0..1 (0 = off), `feedback` 0..0.95 (decay per repetition, default 0.5), `time_ms` echo time 1..4000 (changeable at run time, default 300; omitted = unchanged) |
| `AUDIO_DISTORTION(bus$, amount[, mix])` | overdrive/fuzz. `amount` 0..1 (0 = off, → 0..36 dB drive), `mix` 0..1 (default 1.0) |
| `AUDIO_COMPRESSOR(bus$, threshold_db, ratio[, makeup_db])` | dynamics compressor (glue/pump). `threshold_db` typ. −24..0, `ratio` ≥ 1 (1 = off), `makeup_db` level boost afterwards |
| `AUDIO_EQ(bus$, freq_hz, gain_db[, q])` | parametric bell EQ (one band). `gain_db` 0 = transparent, >0 boost / <0 cut, `q` bandwidth (default 1) |

Signal flow per bus: EQ → filter → distortion → compressor → reverb → delay.
All effects start neutral (no influence on the sound); a mix/cutoff activates
them. Parameters take effect immediately and can be animated (e.g. filter
sweeps).

```basic
' Cave level: music muffled + reverb:
AUDIO_FILTER("music", 1200, 0.3)       ' low-pass, slightly resonant
AUDIO_REVERB("master", 0.5, 0.9, 0.2)  ' reverb over everything

' Acid sweep on the music (in the game loop):
cutoff = 200 + 6000 * (0.5 + 0.5 * SIN(MILLIS() / 400.0))
AUDIO_FILTER("music", cutoff, 0.8)

' Echo on effects only:
AUDIO_DELAY("sfx", 0.4, 0.55)

' Everything off:
AUDIO_FILTER("music", 0, 0) : AUDIO_REVERB("master", 0.0) : AUDIO_DELAY("sfx", 0.0)

' Fuzz bass + mastering glue + bass boost:
AUDIO_DISTORTION("sfx", 0.5)             ' overdrive on effects
AUDIO_COMPRESSOR("master", -18, 4, 3)   ' compressor on the sum
AUDIO_EQ("music", 100, 6, 1.0)          ' +6 dB bell at 100 Hz
```

Demo: [examples/117_audiofx.dh](../../examples/117_audiofx.dh) — filter cutoff
by mouse, reverb/delay by key, with a live spectrum.

**Overall showcase:** [examples/118_audio_studio.dh](../../examples/118_audio_studio.dh)
— “Audio Studio", which puts the whole pipeline on one screen: module streaming
(music bus) + sampler arpeggio (SFX bus), mutable separately, master filter
by mouse, reverb/delay/distortion/lo-fi switchable, permanent mastering
compressor + bass EQ, live spectrum of the finished mix.

## Clock: timing for music and rhythm

A clock counts ticks on the **audio thread** — not in the game's frame rate.
That lets you schedule sounds exactly on the beat, even when the frame rate
drops. That is the difference from the `timer` module, which is processed
per frame.

| Function | Returns | Meaning |
|---|---|---|
| `AUDIO_CLOCK_NEW(ticks_pro_sekunde)` | AUDIO_CLOCK | create a clock — **it starts paused** |
| `AUDIO_CLOCK_START(clock)` | — | let it run (also again after `PAUSE`) |
| `AUDIO_CLOCK_PAUSE(clock)` | — | pause; the tick counter stays where it is |
| `AUDIO_CLOCK_STOP(clock)` | — | stop and set the tick counter to 0 |
| `AUDIO_CLOCK_TICKING(clock)` | BOOLEAN | is it running right now? |
| `AUDIO_CLOCK_TICKS(clock)` | INTEGER | how many ticks have passed? |
| `AUDIO_CLOCK_SET_SPEED(clock, ticks_pro_sekunde)` | — | change the tempo while running |
| `AUDIO_CLOCK_REMOVE(clock)` | — | dismantle the clock |

You do the conversion from BPM yourself — `bpm / 60 * unterteilungen`
(subdivisions):

```basic
IMPORT "audio"
DIM takt AS AUDIO_CLOCK
takt = AUDIO_CLOCK_NEW(120.0 / 60.0 * 4.0)   ' 120 BPM in sixteenths
AUDIO_CLOCK_START(takt)
AUDIO_PLAY_AT(schlag, takt, 8)               ' sounds on the 8th tick
```

`AUDIO_PLAY_AT` schedules the start; from then on the audio thread takes care
of it. There is **no** `UPDATE` you would have to call every frame.

## Spatial hearing: listener and emitter

A listener is the ear of the scene (usually the camera), an emitter a sound
source in space. The audio layer computes volume and panning itself.

| Function | Returns | Meaning |
|---|---|---|
| `AUDIO_LISTENER_NEW(x, y, z)` | AUDIO_LISTENER | place the ear; unrotated it looks along **-Z** |
| `AUDIO_LISTENER_SET_POSITION(listener, x, y, z)` | — | move the ear (e.g. following the camera) |
| `AUDIO_LISTENER_SET_ORIENTATION(listener, yaw_grad)` | — | viewing direction around the Y axis, **in degrees** |
| `AUDIO_LISTENER_REMOVE(listener)` | — | dismantle the listener |
| `AUDIO_EMITTER_NEW(listener, x, y, z [, min_dist [, max_dist]])` | AUDIO_EMITTER | sound source in space; at full volume up to `min_dist`, silent from `max_dist` |
| `AUDIO_EMITTER_SET_POSITION(emitter, x, y, z)` | — | move the source |
| `AUDIO_EMITTER_REMOVE(emitter)` | — | dismantle the source |

Playback is done with `AUDIO_PLAY_ON(sound, emitter, ...)`. The returned
channel can then be treated like any other (`AUDIO_PAUSE`, `AUDIO_STOP`,
`AUDIO_VOLUME` …).

The orientation is deliberately limited to **yaw** — the rotation around the
vertical axis. That covers top-down views and chase cameras without having to
deal with quaternions.

## Speaking: `SPEAK`

The system voice speaks any text — names, numbers, whatever the player has
typed in. For the audio game, the text adventure with a narrator, the
learning game for children who cannot read yet, the announcer in a strategy
game (“unit ready"). Before, this was only possible with prerecorded files.

| Function | Returns | Meaning |
|---|---|---|
| `SPEAK(text$[, unterbrechen])` | — | speaks `text$`; `unterbrechen` (interrupt) = abort the running announcement and speak immediately, otherwise append at the end |
| `SPEAK_STOP()` | — | discard everything running and scheduled |
| `SPEAKING()` | BOOLEAN | is an announcement still running or waiting? |
| `SPEAK_WAIT()` | — | blocks until everything has been spoken (for console programs before `INPUT`; in a game better ask `SPEAKING()`) |
| `SPEAK_VOICE(name$)` | — | choose a voice (a name from `SPEAK_VOICES()`, case does not matter); empty = system voice. Unknown = error |
| `SPEAK_VOICES()` | ARRAY OF STRING | the installed voices, e.g. `Katja`, `Stefan`, `Hedda` |
| `SPEAK_RATE(faktor)` | — | rate 0.5 (half as fast) .. 2.0 (double), default 1.0 |
| `SPEAK_SOUND(text$)` | SOUND | synthesize only — a sound as from `AUDIO_NOTE`, for `AUDIO_PLAY`, `AUDIO_PLAY_ON(emitter)`, `AUDIO_SOUND_MIX`, `AUDIO_SAVE_WAV` |

```basic
SPEAK("Willkommen. Wohin willst du gehen?")
SPEAK_WAIT()
INPUT antwort$

' in a game: do not wait, ask instead
IF treffer AND NOT SPEAKING() THEN SPEAK("Treffer!")
SPEAK("Achtung, hinter dir!", TRUE)        ' interrupts whatever is running

' a speaking character in space
DIM stimme AS SOUND
stimme = SPEAK_SOUND("Wer wagt es, mich zu stoeren?")
AUDIO_PLAY_ON(stimme, drache)              ' spatial, at its emitter
AUDIO_SAVE_WAV(stimme, "drache_01.wav")    ' or bake it into the game as a file
```

**A spoken line is a sound among sounds.** The voice delivers samples
(Windows: WinRT `Windows.Media.SpeechSynthesis`, measured 12–35 ms per
sentence, mono 16 kHz; macOS: `say`; Linux: `espeak-ng` — both via one process
per sentence, untested, without the tool an error with installation hint),
from that a `SOUND` is made, and it runs on the bus **`speech`**:
`AUDIO_BUS_VOLUME("speech", 0.8)` controls it, `AUDIO_PAUSE_ALL` pauses it,
`AUDIO_PUSH`/`AUDIO_POP` save its volume too. The same text with the same
voice and the same rate is not computed twice (a cache of 64 sentences, the
oldest is dropped).

**The queue runs on the audio thread.** An appended sentence gets its start
time passed to Kira (like `AUDIO_PLAY_AT`) — nobody has to check every frame,
and a console program waiting in `INPUT` still hears its three sentences one
after the other.

**If a screen reader is running, it speaks.** `SPEAK` first asks whether an
assistive program is listening (`GUI_SCREENREADER()`, also without gui). Then
the sentence goes as an announcement into the accessibility tree — with the
user's voice, rate and braille display, and without two voices at once.
`SPEAKING()` is then FALSE, `SPEAK_WAIT()` returns immediately. `SPEAK_SOUND`
is always synthesis: a sound is a sound.

## Saving state: AUDIO_PUSH / AUDIO_POP

All bus settings (volume, balance, filter, reverb, echo, distortion,
compressor, EQ — and the volume of the `speech` bus) are global. A forgotten reset only shows up scenes
later — that is what a stack is for, like `GFX_PUSH`/`GFX_POP` for graphics.

| Function | Returns | Meaning |
|---|---|---|
| `AUDIO_PUSH()` | — | push all bus settings onto the stack |
| `AUDIO_POP()` | — | bring them back; **without a previous `PUSH` this is an error** |
| `AUDIO_DEPTH()` | INTEGER | how deep is the stack right now? |

An `AUDIO_POP` releases a running `AUDIO_MODULATE` binding: it writes back the
same value that the modulator is currently controlling.

## External type

| Type | Effect |
|---|---|
| `AUDIO_CHANNEL` | handle to a mixer channel (return value of `AUDIO_PLAY`/`SAMPLE_PLAY`) |
| `SAMPLE` | handle to a loaded PCM sample (return value of `SAMPLE_LOAD`) |

`SOUND` comes from the core — `LOADSOUND` and `AUDIO_TONE`/`AUDIO_NOISE` both return a SOUND. Interchangeable with each other.

## Sound lifetime (`UNLOADSOUND` / `AUDIO_SOUND_COUNT`)

`AUDIO_TONE`/`AUDIO_NOISE`/`AUDIO_SFX` build a new `SOUND` buffer on **every**
call — a frame-based song player that synthesizes a tone per note thus
accumulates buffers over time. With **`UNLOADSOUND(s)`** you free a sound that
is no longer needed (stops the running instance and frees the frame buffer).
The handle stays valid as a tombstone — it is never recycled, so an old handle
never aliases a new sound — but another `PLAYSOUND`/`AUDIO_PLAY` on a freed
handle throws a clear message. **`AUDIO_SOUND_COUNT()`** returns the number of
live (not freed) slots — handy for tracking down sound leaks.

```basic
DIM t AS SOUND
t = AUDIO_TONE(440, 100, "square", 0.5)
PLAYSOUND(t)
' ... when the tone is finished and will not be needed again:
UNLOADSOUND(t)
PRINT AUDIO_SOUND_COUNT()        ' live slots (diagnostics)
```

You can free loaded files (`LOADSOUND`) the same way; usually you keep them
cached, though. Rule of thumb: only unload one-off sounds that were
**freshly synthesized per note** after use.

## Typical game patterns

**SFX manager with a volume master:**

```basic
DIM sfx_volume AS FLOAT
sfx_volume = 0.7

SUB PlaySfx(s AS SOUND)
    AUDIO_PLAY(s, 0, sfx_volume)
END SUB
```

**Pitch variation against repetitive sounds** (the same shot sounds different 100 times):

```basic
DIM ch AS AUDIO_CHANNEL
ch = AUDIO_PLAY(laser, 0, 0.7)
AUDIO_PITCH(ch, 0.9 + RANDF() * 0.2)    ' +-10% random pitch
```

**Sound cooldown against spam:**

```basic
DIM last_hit_ms AS INTEGER
last_hit_ms = 0

IF MILLIS() - last_hit_ms > 100 THEN
    AUDIO_PLAY(hit_sound)
    last_hit_ms = MILLIS()
END IF
```

**Music position for rhythm games:**

```basic
' Beat sync: every 60th beat frame
DIM beat_at AS FLOAT
beat_at = AUDIO_MUSIC_POSITION()
IF (beat_at MOD 0.5) < 0.05 THEN          ' every 500ms
    BeatHit()
END IF
```

## Examples

[examples/68_audio.dh](../../examples/68_audio.dh) demonstrates the full module API including tone generation, pan, music queue.

[examples/114_chiptune.dh](../../examples/114_chiptune.dh) — **4-channel chiptune demo in C64/Amiga style**: a complete piece of music without audio files. Lead (square + vibrato via `AUDIO_SFX`, panned right), chord arpeggio (left), square bass and drums (kick = sine pitch drop, snare/hi-hat = `AUDIO_NOISE`) run in parallel on the mixer; a frame-based pattern player (like the dhtracker export) plays a row every 125 ms. Plus VU meters per channel, a real `AUDIO_FFT` spectrum and a sine scroller.

[examples/115_modplayer.dh](../../examples/115_modplayer.dh) — **Amiga module player**: plays ProTracker `.mod`/`.xm` directly (`PLAYMUSIC`/`AUDIO_MUSIC_*`), with a real spectrum (`AUDIO_FFT`) and drag & drop for your own module. Ships a self-generated, public-domain demo module (`examples/assets/demo.mod`, generator `examples/assets/make_demo_mod.dh`).

[examples/190_tracker.dh](../../examples/190_tracker.dh) — **the tracker in Drachenhauch**: notes from `AUDIO_NOTE`, playback via `AUDIO_CLOCK`/`AUDIO_PLAY_AT` (scheduled two rows ahead, stopped by removing the clock), WAV export via `AUDIO_SOUND_NEW/MIX/NORMALIZE`.

[examples/116_sampler.dh](../../examples/116_sampler.dh) — **Amiga-style sampler**: a single pluck sample (`SAMPLE_LOAD`) is played across the whole keyboard via `SAMPLE_PLAY` (resampling = pitch like Paula). Auto melody + bass from the same sample, clickable keys, `L` switches on the Paula lo-fi mode.

## In the native runtime (dhrt)

The `audio` module runs natively via **Kira** (cpal) — its own audio thread, decoupled from the game loop (replaced raylib audio on 2026-06-13; included with the `graphics` feature, raylib remains for window/input). Audio output is **not** part of the deterministic bit-identical guarantee — like `RND`/`tween`. Notes:

- `SOUND` and `AUDIO_CHANNEL` are integer handles; a “channel” is the most recently started instance of a loaded/built sound.
- Fade in/out, `AUDIO_STOP` with a fade and `AUDIO_PAN_SLIDE` are **native Kira tweens** (run on the audio thread); `loops` via `loop_region` (endless) or restart counting (finite). `AUDIO_FFT` taps the mixer's main track through an effect.
- Volume is kept internally in decibels (Kira); the built-ins still take linear 0..1.
- Tone generation (`AUDIO_TONE`/`AUDIO_NOISE`/`AUDIO_SFX`) and the sampler build the waveform as a float buffer directly as Kira `StaticSoundData`.
- **Tracker modules** (`.mod`/`.xm`) as music are **streamed in real time** (a custom Kira sound that polls the pure Rust player `xmrs` on the audio thread): loaded immediately (no pre-render), exact endless looping, little RAM, with a pitch resampler + click-free volume fades. Stream formats (ogg/mp3/wav/flac) stream from disk.
- **FLAC** works as a sound (`LOADSOUND`) and as music. The music runs through a decoder of its
  own: with Kira's built-in one, Symphonia 0.6 did not jump back cleanly to the beginning after
  the first pass, and FLAC music with a loop (the default of `AUDIO_MUSIC_PLAY`)
  stayed silent. The custom decoder reopens the file before every jump.
