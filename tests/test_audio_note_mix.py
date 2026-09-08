"""`AUDIO_NOTE` und `AUDIO_SOUND_NEW/MIX/NORMALIZE` -- Noten bauen und Klaenge
mischen.

Gefunden beim Tracker-Piloten (`examples/190_tracker.dh`): `AUDIO_SFX`
kennt drei ZEITEN, aber keinen Sustain-PEGEL -- eine Orgel, die so lange
klingt, wie man die Taste haelt, und ein Klavier, das von allein verstummt,
liessen sich damit nicht unterscheiden. Und zwei Klaenge zu EINEM zu machen
ging gar nicht; ein Song liess sich also nicht als WAV abliefern.

Geprueft wird am Ergebnis, nicht an der Behauptung: die WAV-Dateien liest
Pythons `wave`-Modul (ein FREMDER Leser), die Huellkurve wird nachgemessen.

Braucht einen dhrt mit Audio, steht darum in `conftest._BRAUCHT_GRAFIK`.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/audio_note_mix.dhtest` (dhrt test); hier bleiben nur die, die sich
mit den Proben von `--- ton` nicht ausdruecken lassen.
"""
import wave
from pathlib import Path

import numpy as np
import pytest

from drachenhauch.errors import DHRuntimeError

KOPF = 'IMPORT "audio"\n'


def _samples(pfad: Path):
    with wave.open(str(pfad)) as w:
        n, ch, sr = w.getnframes(), w.getnchannels(), w.getframerate()
        roh = w.readframes(n)
    a = np.frombuffer(roh, "<i2").astype(np.float32) / 32767.0
    return a.reshape(-1, ch), sr


def _huelle(a, sr, fenster_ms=10):
    """Spitzenpegel je Fenster -- die Huellkurve, grob abgetastet."""
    f = int(sr * fenster_ms / 1000)
    n = a.shape[0] // f
    return np.abs(a[: n * f, 0]).reshape(n, f).max(axis=1)


# ------------------------------------------------------------- AUDIO_NOTE


def test_slide_endet_auf_der_zieltonhoehe(run_gb, tmp_path):
    """Portamento um eine Oktave: am Ende der gehaltenen Zeit schwingt die
    Note doppelt so schnell wie am Anfang -- gezaehlt an den Nulldurchgaengen."""
    run_gb(KOPF + 'AUDIO_SAVE_WAV(AUDIO_NOTE("sine", 200, 1000, 0, 0, 1.0, 0, 1.0, 0, 0, 0, 12), "n.wav")\n',
           base=tmp_path)
    a, sr = _samples(tmp_path / "n.wav")
    s = a[:, 0]

    def hz(von_ms, bis_ms):
        seg = s[int(sr * von_ms / 1000):int(sr * bis_ms / 1000)]
        kreuz = np.count_nonzero(np.diff(np.signbit(seg)))
        return kreuz / 2.0 / ((bis_ms - von_ms) / 1000.0)

    assert 190 < hz(20, 120) < 225, hz(20, 120)
    assert 370 < hz(880, 980) < 420, hz(880, 980)


# ------------------------------------------------------------ Mischen


def test_in_sich_selbst_mischen_ist_ein_fehler(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'DIM m AS SOUND\nm = AUDIO_SOUND_NEW(200)\nAUDIO_SOUND_MIX(m, m, 0)\n')
    assert "derselbe" in str(e.value)
