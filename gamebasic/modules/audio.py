"""Audio-Mixer-Modul fuer GameBasic.

Erweitert die Basis-Sound-Built-ins (LOADSOUND/PLAYSOUND/STOPSOUND/PLAYMUSIC/
STOPMUSIC) um Channel-Management, Pause/Resume/Fade, Music-Position und
Tone-Generation. Nutzt pygame.mixer + numpy.

Kategorien:

  Mixer-Lifecycle:
    AUDIO_INIT([freq[, channels[, buffer]]])
    AUDIO_SET_NUM_CHANNELS(n)
    AUDIO_NUM_CHANNELS()                 -> INTEGER
    AUDIO_BUSY_CHANNELS()                -> INTEGER  (aktive Channels)
    AUDIO_PAUSE_ALL()
    AUDIO_RESUME_ALL()
    AUDIO_STOP_ALL()

  Channel-Playback:
    AUDIO_PLAY(sound[, loops[, volume[, fade_in_ms]]])  -> AUDIO_CHANNEL
    AUDIO_PAUSE(ch)
    AUDIO_RESUME(ch)
    AUDIO_STOP(ch[, fade_out_ms])
    AUDIO_IS_PLAYING(ch)                 -> BOOLEAN
    AUDIO_VOLUME(ch, v)                  ' beide Stereo-Seiten gleich
    AUDIO_GET_VOLUME(ch)                 -> FLOAT
    AUDIO_PAN(ch, left, right)           ' separates Stereo-Volume

  Music (Streaming, ein Singleton-Track):
    AUDIO_MUSIC_LOAD(path)
    AUDIO_MUSIC_PLAY([loops[, fade_in_ms]])
    AUDIO_MUSIC_STOP([fade_out_ms])
    AUDIO_MUSIC_PAUSE()
    AUDIO_MUSIC_RESUME()
    AUDIO_MUSIC_VOLUME(v)
    AUDIO_MUSIC_GET_VOLUME()             -> FLOAT
    AUDIO_MUSIC_POSITION()               -> FLOAT (Sekunden seit Play)
    AUDIO_MUSIC_BUSY()                   -> BOOLEAN
    AUDIO_MUSIC_QUEUE(path)              ' nach aktuellem Track abspielen

  Tone-Generation (liefert SOUND, kompatibel mit PLAYSOUND und AUDIO_PLAY):
    AUDIO_TONE(freq_hz, dauer_ms[, waveform$[, volume]])  -> SOUND
    AUDIO_NOISE(dauer_ms[, volume])                       -> SOUND

  Waveforms (case-insensitive): "sine", "square", "saw", "triangle", "noise".

Konvention:
  loops = 0   -> einmal abspielen
  loops = -1  -> endlos
  loops = N   -> 1 + N Wiederholungen (pygame-Konvention)
  volume in [0.0, 1.0]; out-of-range wird geklemmt.

Implementierung:
  Lazy pygame-Init (Banner unterdrueckt). Audio funktioniert ohne Display,
  daher graphics-unabhaengig. Tone-Generation per numpy + pygame.sndarray.
"""
from __future__ import annotations

import os

import numpy as np

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from ..interpreter import _Sound
from . import register_type


# --- Lazy-Init -------------------------------------------------------

_pygame = None  # cached pygame-Modul


def _ensure_pygame():
    """Importiert pygame mit unterdruecktem Support-Prompt. Idempotent."""
    global _pygame
    if _pygame is None:
        os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "hide")
        import pygame  # noqa: WPS433 - intentional lazy import
        _pygame = pygame
    return _pygame


def _ensure_mixer():
    """pygame.mixer.init() wenn noch nicht initialisiert."""
    pg = _ensure_pygame()
    if not pg.mixer.get_init():
        pg.mixer.init()
    return pg


# --- Channel-Wrapper -------------------------------------------------

class _AudioChannel:
    """Wrapper um pygame.mixer.Channel.

    Channels werden vom Mixer beim ersten AUDIO_PLAY allokiert. Wir cachen
    die pygame-Channel-Referenz; sobald der Sound zu Ende ist, behandelt
    pygame den Channel als 'frei' und kann ihn wieder vergeben.
    """
    __slots__ = ("channel",)

    def __init__(self, channel):
        self.channel = channel

    def __repr__(self):
        return "<AUDIO_CHANNEL>"


register_type("audio_channel", _AudioChannel)


# --- Type-Helpers ----------------------------------------------------

def _check_audio_channel(v, fn: str) -> _AudioChannel:
    if not isinstance(v, _AudioChannel):
        raise TypeMismatchError(f"{fn} erwartet AUDIO_CHANNEL")
    return v


def _check_sound(v, fn: str) -> _Sound:
    if not isinstance(v, _Sound):
        raise TypeMismatchError(f"{fn} erwartet SOUND")
    return v


def _check_num(v, fn: str, label: str = "Zahl") -> float:
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return float(v)


def _check_int_strict(v, fn: str, label: str = "INTEGER") -> int:
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return v


def _check_str_strict(v, fn: str, label: str = "STRING") -> str:
    if not isinstance(v, str):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return v


def _clamp01(v: float) -> float:
    if v < 0.0:
        return 0.0
    if v > 1.0:
        return 1.0
    return v


# --- Mixer-Lifecycle -------------------------------------------------

@builtin("AUDIO_INIT", arity=(0, 3))
def _audio_init(*args):
    """AUDIO_INIT([freq[, channels[, buffer]]])

    Initialisiert pygame.mixer mit gewaehlten Parametern. Wenn schon
    initialisiert, wird er erst beendet und neu gestartet. Optional;
    sonst nutzt der Mixer beim ersten Sound-Call seine Defaults
    (44100Hz, 16-bit, stereo, 512 buffer)."""
    freq = _check_int_strict(args[0], "AUDIO_INIT", "freq INTEGER") if len(args) >= 1 else 44100
    channels = _check_int_strict(args[1], "AUDIO_INIT", "channels INTEGER") if len(args) >= 2 else 2
    buffer = _check_int_strict(args[2], "AUDIO_INIT", "buffer INTEGER") if len(args) >= 3 else 512
    if freq <= 0 or buffer <= 0 or channels not in (1, 2):
        raise GBRuntimeError("AUDIO_INIT: ungueltige Parameter")
    pg = _ensure_pygame()
    if pg.mixer.get_init():
        pg.mixer.quit()
    pg.mixer.init(frequency=freq, size=-16, channels=channels, buffer=buffer)
    return None


@builtin("AUDIO_SET_NUM_CHANNELS", arity=1, types=("int",))
def _audio_set_num_channels(n):
    if n < 0:
        raise GBRuntimeError("AUDIO_SET_NUM_CHANNELS: n muss >= 0 sein")
    pg = _ensure_mixer()
    pg.mixer.set_num_channels(n)
    return None


@builtin("AUDIO_NUM_CHANNELS", arity=0)
def _audio_num_channels():
    pg = _ensure_mixer()
    return int(pg.mixer.get_num_channels())


@builtin("AUDIO_BUSY_CHANNELS", arity=0)
def _audio_busy_channels():
    pg = _ensure_mixer()
    return int(pg.mixer.get_busy())


@builtin("AUDIO_PAUSE_ALL", arity=0)
def _audio_pause_all():
    pg = _ensure_mixer()
    pg.mixer.pause()
    return None


@builtin("AUDIO_RESUME_ALL", arity=0)
def _audio_resume_all():
    pg = _ensure_mixer()
    pg.mixer.unpause()
    return None


@builtin("AUDIO_STOP_ALL", arity=0)
def _audio_stop_all():
    pg = _ensure_mixer()
    pg.mixer.stop()
    return None


# --- Channel-Playback ------------------------------------------------

@builtin("AUDIO_PLAY", arity=(1, 4))
def _audio_play(*args):
    """AUDIO_PLAY(sound[, loops[, volume[, fade_in_ms]]]) -> AUDIO_CHANNEL.

    Liefert den vom Mixer ausgewaehlten Channel zurueck. Wenn alle
    Channels belegt sind, wirft pygame keinen Fehler; wir liefern dann
    ein AUDIO_CHANNEL-Wrapper mit None-Channel zurueck und AUDIO_IS_PLAYING
    liefert FALSE.
    """
    s = _check_sound(args[0], "AUDIO_PLAY")
    loops = _check_int_strict(args[1], "AUDIO_PLAY", "loops INTEGER") if len(args) >= 2 else 0
    volume = _check_num(args[2], "AUDIO_PLAY", "Lautstaerke") if len(args) >= 3 else 1.0
    fade_ms = _check_int_strict(args[3], "AUDIO_PLAY", "fade_in_ms INTEGER") if len(args) >= 4 else 0
    if fade_ms < 0:
        raise GBRuntimeError("AUDIO_PLAY: fade_in_ms muss >= 0 sein")
    _ensure_mixer()
    s.sound.set_volume(_clamp01(volume))
    ch = s.sound.play(loops=loops, fade_ms=fade_ms)
    return _AudioChannel(ch)


@builtin("AUDIO_PAUSE", arity=1)
def _audio_pause(ch):
    ch = _check_audio_channel(ch, "AUDIO_PAUSE")
    if ch.channel is not None:
        ch.channel.pause()
    return None


@builtin("AUDIO_RESUME", arity=1)
def _audio_resume(ch):
    ch = _check_audio_channel(ch, "AUDIO_RESUME")
    if ch.channel is not None:
        ch.channel.unpause()
    return None


@builtin("AUDIO_STOP", arity=(1, 2))
def _audio_stop(*args):
    """AUDIO_STOP(ch[, fade_out_ms])"""
    ch = _check_audio_channel(args[0], "AUDIO_STOP")
    if ch.channel is None:
        return None
    if len(args) == 2:
        fade_ms = _check_int_strict(args[1], "AUDIO_STOP", "fade_out_ms INTEGER")
        if fade_ms < 0:
            raise GBRuntimeError("AUDIO_STOP: fade_out_ms muss >= 0 sein")
        ch.channel.fadeout(fade_ms)
    else:
        ch.channel.stop()
    return None


@builtin("AUDIO_IS_PLAYING", arity=1)
def _audio_is_playing(ch):
    ch = _check_audio_channel(ch, "AUDIO_IS_PLAYING")
    if ch.channel is None:
        return False
    return bool(ch.channel.get_busy())


@builtin("AUDIO_VOLUME", arity=2)
def _audio_volume(ch, v):
    """AUDIO_VOLUME(ch, v) - setzt links und rechts gleich."""
    ch = _check_audio_channel(ch, "AUDIO_VOLUME")
    vol = _clamp01(_check_num(v, "AUDIO_VOLUME", "Lautstaerke"))
    if ch.channel is not None:
        ch.channel.set_volume(vol)
    return None


@builtin("AUDIO_GET_VOLUME", arity=1)
def _audio_get_volume(ch):
    ch = _check_audio_channel(ch, "AUDIO_GET_VOLUME")
    if ch.channel is None:
        return 0.0
    return float(ch.channel.get_volume())


@builtin("AUDIO_PAN", arity=3)
def _audio_pan(ch, left, right):
    """AUDIO_PAN(ch, left, right) - separates Stereo-Volume.

    left=1.0, right=0.0 spielt nur links. Beide auf den jeweiligen
    Wert geclamped."""
    ch = _check_audio_channel(ch, "AUDIO_PAN")
    l = _clamp01(_check_num(left, "AUDIO_PAN", "left"))
    r = _clamp01(_check_num(right, "AUDIO_PAN", "right"))
    if ch.channel is not None:
        ch.channel.set_volume(l, r)
    return None


# --- Music-Streaming -------------------------------------------------

@builtin("AUDIO_MUSIC_LOAD", arity=1, types=("str",))
def _audio_music_load(path):
    pg = _ensure_mixer()
    try:
        pg.mixer.music.load(path)
    except Exception as exc:
        raise GBRuntimeError(f"AUDIO_MUSIC_LOAD: {exc}")
    return None


@builtin("AUDIO_MUSIC_PLAY", arity=(0, 2))
def _audio_music_play(*args):
    """AUDIO_MUSIC_PLAY([loops[, fade_in_ms]])"""
    loops = _check_int_strict(args[0], "AUDIO_MUSIC_PLAY", "loops INTEGER") if len(args) >= 1 else -1
    fade_ms = _check_int_strict(args[1], "AUDIO_MUSIC_PLAY", "fade_in_ms INTEGER") if len(args) >= 2 else 0
    if fade_ms < 0:
        raise GBRuntimeError("AUDIO_MUSIC_PLAY: fade_in_ms muss >= 0 sein")
    pg = _ensure_mixer()
    pg.mixer.music.play(loops=loops, fade_ms=fade_ms)
    return None


@builtin("AUDIO_MUSIC_STOP", arity=(0, 1))
def _audio_music_stop(*args):
    pg = _ensure_mixer()
    if len(args) == 1:
        fade_ms = _check_int_strict(args[0], "AUDIO_MUSIC_STOP", "fade_out_ms INTEGER")
        if fade_ms < 0:
            raise GBRuntimeError("AUDIO_MUSIC_STOP: fade_out_ms muss >= 0 sein")
        pg.mixer.music.fadeout(fade_ms)
    else:
        pg.mixer.music.stop()
    return None


@builtin("AUDIO_MUSIC_PAUSE", arity=0)
def _audio_music_pause():
    pg = _ensure_mixer()
    pg.mixer.music.pause()
    return None


@builtin("AUDIO_MUSIC_RESUME", arity=0)
def _audio_music_resume():
    pg = _ensure_mixer()
    pg.mixer.music.unpause()
    return None


@builtin("AUDIO_MUSIC_VOLUME", arity=1)
def _audio_music_volume(v):
    vol = _clamp01(_check_num(v, "AUDIO_MUSIC_VOLUME", "Lautstaerke"))
    pg = _ensure_mixer()
    pg.mixer.music.set_volume(vol)
    return None


@builtin("AUDIO_MUSIC_GET_VOLUME", arity=0)
def _audio_music_get_volume():
    pg = _ensure_mixer()
    return float(pg.mixer.music.get_volume())


@builtin("AUDIO_MUSIC_POSITION", arity=0)
def _audio_music_position():
    """Liefert die abgespielte Zeit in Sekunden seit dem letzten Play.

    Wichtig: pygame meldet -1, wenn nichts laeuft. Wir clampen auf 0.0."""
    pg = _ensure_mixer()
    pos_ms = pg.mixer.music.get_pos()
    if pos_ms < 0:
        return 0.0
    return pos_ms / 1000.0


@builtin("AUDIO_MUSIC_BUSY", arity=0)
def _audio_music_busy():
    pg = _ensure_mixer()
    return bool(pg.mixer.music.get_busy())


@builtin("AUDIO_MUSIC_QUEUE", arity=1, types=("str",))
def _audio_music_queue(path):
    """Queued einen weiteren Track. Wird automatisch nach dem aktuellen
    abgespielt."""
    pg = _ensure_mixer()
    try:
        pg.mixer.music.queue(path)
    except Exception as exc:
        raise GBRuntimeError(f"AUDIO_MUSIC_QUEUE: {exc}")
    return None


# --- Tone-Generation -------------------------------------------------

# Kurze Anti-Click-Envelope (linearer Fade-In/Out). 5ms ist genug, um
# digitalen Klick am Anfang/Ende zu vermeiden, aber kurz genug, dass
# kurze Pieptoene noch perkussiv sind.
_FADE_SECS = 0.005

_WAVEFORMS = ("sine", "square", "saw", "triangle", "noise")


def _generate_waveform(waveform: str, freq_hz: float, t: np.ndarray) -> np.ndarray:
    """t in Sekunden, Phase = 2*pi*f*t. Liefert Werte in [-1, 1]."""
    if waveform == "sine":
        return np.sin(2.0 * np.pi * freq_hz * t)
    if waveform == "square":
        # Vermeidet np.sign(0) = 0 -> saubere Square
        return np.where(np.sin(2.0 * np.pi * freq_hz * t) >= 0, 1.0, -1.0)
    if waveform == "saw":
        # Periodisch -1..+1, aufsteigend
        return 2.0 * (t * freq_hz - np.floor(0.5 + t * freq_hz))
    if waveform == "triangle":
        return 2.0 * np.abs(2.0 * (t * freq_hz - np.floor(0.5 + t * freq_hz))) - 1.0
    if waveform == "noise":
        return np.random.uniform(-1.0, 1.0, t.shape).astype(np.float64)
    raise GBRuntimeError(
        f"AUDIO_TONE: unbekannte Waveform '{waveform}' "
        f"(erlaubt: {', '.join(_WAVEFORMS)})"
    )


def _make_sound_from_wave(wave: np.ndarray, volume: float) -> _Sound:
    """Konvertiert ein float64-Array [-1, 1] zu einem pygame.mixer.Sound.

    Anti-Click-Envelope wird automatisch angewandt. Stereo wird vom
    aktuellen Mixer-Format abgeleitet."""
    pg = _ensure_mixer()
    init = pg.mixer.get_init()
    if init is None:
        raise GBRuntimeError("AUDIO_TONE: mixer nicht initialisiert")
    sample_rate, _size, n_channels = init
    n_samples = wave.shape[0]
    fade = min(int(sample_rate * _FADE_SECS), max(0, n_samples // 4))
    if fade > 0:
        env = np.ones(n_samples)
        env[:fade] = np.linspace(0.0, 1.0, fade)
        env[-fade:] = np.linspace(1.0, 0.0, fade)
        wave = wave * env
    samples = np.clip(wave * volume, -1.0, 1.0)
    samples_int = (samples * 32767.0).astype(np.int16)
    if n_channels == 2:
        samples_int = np.column_stack((samples_int, samples_int))
    sound = pg.sndarray.make_sound(samples_int)
    return _Sound(sound, "")


@builtin("AUDIO_TONE", arity=(2, 4))
def _audio_tone(*args):
    """AUDIO_TONE(freq_hz, dauer_ms[, waveform$[, volume]]) -> SOUND"""
    freq = _check_num(args[0], "AUDIO_TONE", "freq_hz")
    dur_ms = _check_int_strict(args[1], "AUDIO_TONE", "dauer_ms INTEGER")
    waveform = _check_str_strict(args[2], "AUDIO_TONE", "waveform STRING").lower() if len(args) >= 3 else "sine"
    volume = _clamp01(_check_num(args[3], "AUDIO_TONE", "Lautstaerke")) if len(args) >= 4 else 1.0
    if freq <= 0:
        raise GBRuntimeError("AUDIO_TONE: freq_hz muss > 0 sein")
    if dur_ms <= 0:
        raise GBRuntimeError("AUDIO_TONE: dauer_ms muss > 0 sein")
    pg = _ensure_mixer()
    sample_rate = pg.mixer.get_init()[0]
    n_samples = int(sample_rate * dur_ms / 1000.0)
    if n_samples <= 0:
        raise GBRuntimeError("AUDIO_TONE: dauer_ms zu klein fuer Sample-Rate")
    t = np.arange(n_samples, dtype=np.float64) / sample_rate
    wave = _generate_waveform(waveform, freq, t)
    return _make_sound_from_wave(wave, volume)


@builtin("AUDIO_NOISE", arity=(1, 2))
def _audio_noise(*args):
    """AUDIO_NOISE(dauer_ms[, volume]) -> SOUND  (Weisses Rauschen)"""
    dur_ms = _check_int_strict(args[0], "AUDIO_NOISE", "dauer_ms INTEGER")
    volume = _clamp01(_check_num(args[1], "AUDIO_NOISE", "Lautstaerke")) if len(args) >= 2 else 1.0
    if dur_ms <= 0:
        raise GBRuntimeError("AUDIO_NOISE: dauer_ms muss > 0 sein")
    pg = _ensure_mixer()
    sample_rate = pg.mixer.get_init()[0]
    n_samples = int(sample_rate * dur_ms / 1000.0)
    if n_samples <= 0:
        raise GBRuntimeError("AUDIO_NOISE: dauer_ms zu klein fuer Sample-Rate")
    wave = np.random.uniform(-1.0, 1.0, n_samples).astype(np.float64)
    return _make_sound_from_wave(wave, volume)


@builtin("AUDIO_FFT", arity=1)
def _audio_fft(arr):
    """AUDIO_FFT(bands) -- fuellt ein 1D ARRAY OF FLOAT mit Frequenzband-Pegeln
    (0..1) des aktuell hoerbaren Audios (echte FFT).

    NUR in der nativen Runtime (gbrt) liefert das echte Werte -- dort haengt
    ein Mix-Processor an der Audio-Pipeline. Im Python/pygame-Pfad gibt es
    keinen solchen Tap; die Funktion fuellt dann Nullen (kein Crash, keine
    Reaktivitaet). Laenge des Arrays = Anzahl der Baender."""
    from ..interpreter import _GBArray
    if (not isinstance(arr, _GBArray) or arr.element_type != "float"
            or len(arr.dims) != 1):
        raise TypeMismatchError("AUDIO_FFT: erwartet 1D ARRAY OF FLOAT")
    for i in range(arr.dims[0]):
        arr.values[i] = 0.0
    return None
