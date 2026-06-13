# SFX-Generator

Tool für Retro-Soundeffekte im sfxr-Stil: eigener Synthesizer mit Pitch-Slide, Hüllkurve, Vibrato und **SID-Charakter** (Pulsbreite/PWM + resonanter Filter-Sweep), Live-Wellenform-Vorschau, Abspielen und Export als **WAV** (per `LOADSOUND` ladbar) oder GB-Code. Ist der **SFX-Tab im [Audio Studio](tracker.md#audio-studio)** (`gbsound`).

## Starten

Am bequemsten als Tab im **Audio Studio**: `gbsound` (oder `gbrun.py --audio`) → Reiter „SFX-Generator". Auch einzeln aus dem **Code-Editor** (Toolbar-Button / `Datei → SFX-Generator öffnen ...`, `Strg+Shift+J`) oder standalone `gbsfx` / `gbrun.py --sfx` (öffnen ebenfalls das Studio auf dem SFX-Tab). Braucht `PySide6` + `numpy`.

## Bedienung

Die UI ist eine **Fader-Bank im sfxr-Stil**: links eine **Preset-Leiste**, oben die große **Wellenform-Vorschau**, darunter vier farbcodierte **Parameter-Karten** mit beschrifteten Schiebereglern (Live-Wert rechts):

- **Ton** (cyan) — Waveform (`square`/`saw`/`sine`/`triangle`/`noise`, als Dropdown), Frequenz, **Pitch-Slide** (Hz/s, negativ = fallend), Lautstärke.
- **Hüllkurve** (mint) — Attack / Sustain / Decay (ms).
- **SID / Filter** (magenta) — **Pulsbreite** + **PWM-Tiefe/-Speed** (für `square`) und ein **resonanter Tiefpass** (Cutoff / Sweep Hz/s / Resonanz). 0 = neutral.
- **Vibrato / Stereo** (amber) — Vibrato-Tiefe + -Speed, **Stereo-Breite** (0 = mono, >0 = breiter per Detune; bei `noise` dekorreliert) und **Pan** (links −1 … +1 rechts).

- **Preset-Leiste** links — `Pickup/Coin`, `Laser/Shoot`, `Explosion`, `Powerup`, `Hit/Hurt`, `Jump`, `Blip/Select`. Klick lädt + spielt.
- **Preset-Bibliothek** (oben) — speichere eigene Sounds als benannte Presets („Speichern unter...", persistiert unter `~/.gamebasic/presets/sfx.json`) und lade sie über die Combo wieder.
- **`▶ Abspielen`** spielt den aktuellen Effekt, **`🎲 Zufall`** würfelt neue Parameter (mit Abspielen).
- **↶/↷** (oder `Strg+Z` / `Strg+Y`) machen Änderungen rückgängig bzw. wieder her — ein Preset-Laden, ein `Zufall` oder ein Fader-Drag zählt je als ein Schritt.

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

Der Effekt wird im Spiel selbst synthetisiert — `slide` ist der Pitch-Sweep (Hz/s, negativ = fallend), Attack/Sustain/Decay die Hüllkurve, `vib_depth`/`vib_speed` ein optionales Vibrato. Der optionale `stereo_width` (0…1) macht den Sound **stereo/breiter** (Detune zwischen L/R; bei `noise` dekorreliert). Komplementär zu `AUDIO_TONE` (das nur konstante Töne kann). Läuft über die native Runtime `gbrt`; die Synth-Mathematik ist geteilt — [`gamebasic/synth.py`](../gamebasic/synth.py) (Editor-Vorschau) bzw. `rust/gb_runtime/src/audio.rs` (nativ).

**Pan** (Position links/rechts) wird beim Abspielen gesetzt — der Export schreibt dafür `AUDIO_PLAY` + `AUDIO_PAN(ch, left, right)` dazu:
```basic
snd = AUDIO_SFX("saw", 1000, -1400, 0, 30, 150, 0, 0, 0.7, 0.6)
DIM ch AS AUDIO_CHANNEL
ch = AUDIO_PLAY(snd)
AUDIO_PAN(ch, 0.5, 1)        ' rechts gepannt
```

`WAV exportieren` bleibt als Alternative, falls du den Effekt lieber als Asset bündeln willst.
