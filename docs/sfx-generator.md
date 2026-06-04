# SFX-Generator

Standalone-Tool für Retro-Soundeffekte im sfxr-Stil: eigener Synthesizer mit Pitch-Slide, Hüllkurve und Vibrato, Live-Wellenform-Vorschau, Abspielen und Export als **WAV** (per `LOADSOUND` ladbar) oder GB-Code.

## Starten

Aus dem **Code-Editor**: Toolbar-Button (Lautsprecher-Symbol) oder `Datei → SFX-Generator öffnen ...` (`Strg+Shift+J`).

Standalone: `gbsfx` oder `.venv\Scripts\python.exe gbrun.py --sfx` (braucht `PySide6` + `numpy`).

## Bedienung

- **Presets** oben — `Pickup/Coin`, `Laser/Shoot`, `Explosion`, `Powerup`, `Hit/Hurt`, `Jump`, `Blip/Select`. Klick lädt + spielt.
- **Wellenform-Vorschau** zeigt das aktuelle Signal (Pitch-Sweep + Hüllkurve sichtbar).
- **Ton** — Waveform (`square`/`saw`/`sine`/`triangle`/`noise`), Frequenz, **Pitch-Slide** (Hz/s, negativ = fallend), Lautstärke.
- **Hüllkurve & Vibrato** — Attack / Sustain / Decay (ms), Vibrato-Tiefe + -Speed.
- **`▶ Abspielen`** spielt den aktuellen Effekt, **`Zufall`** würfelt neue Parameter (mit Abspielen).

## Export

- **`WAV exportieren ...`** schreibt eine `.wav` (44,1 kHz, 16-bit mono) und zeigt den Lade-Code:
  ```basic
  DIM snd AS SOUND
  snd = LOADSOUND("jump.wav")
  PLAYSOUND(snd)
  ```
- **`GB-Code`** — bei einem **einfachen** Ton (kein Pitch-Slide/Vibrato) das passende `AUDIO_TONE`/`AUDIO_NOISE`-Snippet:
  ```basic
  IMPORT "audio"
  DIM snd AS SOUND
  snd = AUDIO_TONE(660, 160, "square", 0.7)
  PLAYSOUND(snd)
  ```
  Bei Sweeps/Vibrato/Hüllkurve kann `AUDIO_TONE` das nicht abbilden — dann den Effekt als **WAV** exportieren und mit `LOADSOUND` laden.

## Hintergrund

Das `audio`-Modul erzeugt mit `AUDIO_TONE` nur **konstante** Töne. Für sfxr-typische Effekte (fallender Laser, Explosions-Decay) braucht es einen eigenen Synth — der lebt im Tool, das Ergebnis kommt als WAV-Asset ins Spiel.
