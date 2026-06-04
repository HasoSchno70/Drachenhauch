# SFX-Generator

Standalone-Tool für Retro-Soundeffekte im sfxr-Stil: eigener Synthesizer mit Pitch-Slide, Hüllkurve und Vibrato, Live-Wellenform-Vorschau, Abspielen und Export als **WAV** (per `LOADSOUND` ladbar) oder GB-Code.

## Starten

Aus dem **Code-Editor**: Toolbar-Button (Lautsprecher-Symbol) oder `Datei → SFX-Generator öffnen ...` (`Strg+Shift+J`).

Standalone: `gbsfx` oder `.venv\Scripts\python.exe gbrun.py --sfx` (braucht `PySide6` + `numpy`).

## Bedienung

- **Presets** oben — `Pickup/Coin`, `Laser/Shoot`, `Explosion`, `Powerup`, `Hit/Hurt`, `Jump`, `Blip/Select`. Klick lädt + spielt.
- **Wellenform-Vorschau** zeigt das aktuelle Signal (Pitch-Sweep + Hüllkurve sichtbar).
- **Ton** — Waveform (`square`/`saw`/`sine`/`triangle`/`noise`), Frequenz, **Pitch-Slide** (Hz/s, negativ = fallend), Lautstärke.
- **Hüllkurve & Vibrato** — Attack / Sustain / Decay (ms), Vibrato-Tiefe + -Speed, **Stereo-Breite** (0 = mono, >0 = breiter per Detune; bei `noise` dekorreliert) und **Pan** (links −1 … +1 rechts).
- **`▶ Abspielen`** spielt den aktuellen Effekt, **`Zufall`** würfelt neue Parameter (mit Abspielen).

## Export

- **`WAV exportieren ...`** schreibt eine `.wav` (44,1 kHz, 16-bit mono) und zeigt den Lade-Code:
  ```basic
  DIM snd AS SOUND
  snd = LOADSOUND("jump.wav")
  PLAYSOUND(snd)
  ```
- **`GB-Code`** — der `AUDIO_SFX`-Aufruf, der den Effekt **prozedural zur Laufzeit** erzeugt (kein WAV-Asset nötig):
  ```basic
  IMPORT "audio"
  DIM snd AS SOUND
  snd = AUDIO_SFX("saw", 1000, -1400, 0, 30, 150, 0, 0, 0.7)
  PLAYSOUND(snd)
  ```

## `AUDIO_SFX` — der native Synth

```
AUDIO_SFX(waveform$, freq, slide, attack_ms, sustain_ms, decay_ms,
          vib_depth, vib_speed, volume[, stereo_width]) -> SOUND
```

Der Effekt wird im Spiel selbst synthetisiert — `slide` ist der Pitch-Sweep (Hz/s, negativ = fallend), Attack/Sustain/Decay die Hüllkurve, `vib_depth`/`vib_speed` ein optionales Vibrato. Der optionale `stereo_width` (0…1) macht den Sound **stereo/breiter** (Detune zwischen L/R; bei `noise` dekorreliert). Komplementär zu `AUDIO_TONE` (das nur konstante Töne kann). Läuft in **beiden** Pfaden — Tree-Walker **und** native Runtime `gbrt` (gleicher Synth, [`gamebasic/synth.py`](../gamebasic/synth.py) bzw. `rust/gb_runtime/src/audio.rs`).

**Pan** (Position links/rechts) wird beim Abspielen gesetzt — der Export schreibt dafür `AUDIO_PLAY` + `AUDIO_PAN(ch, left, right)` dazu:
```basic
snd = AUDIO_SFX("saw", 1000, -1400, 0, 30, 150, 0, 0, 0.7, 0.6)
DIM ch AS AUDIO_CHANNEL
ch = AUDIO_PLAY(snd)
AUDIO_PAN(ch, 0.5, 1)        ' rechts gepannt
```

`WAV exportieren` bleibt als Alternative, falls du den Effekt lieber als Asset bündeln willst.
