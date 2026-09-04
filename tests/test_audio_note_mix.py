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
def test_die_laenge_ist_dauer_plus_release(run_gb, tmp_path):
    run_gb(KOPF + 'AUDIO_SAVE_WAV(AUDIO_NOTE("square", 220, 400, 0, 0, 1.0, 100, 0.8), "n.wav")\n',
           base=tmp_path)
    a, sr = _samples(tmp_path / "n.wav")
    assert abs(a.shape[0] / sr - 0.500) < 0.005, "400 ms gehalten + 100 ms Ausklingen"


def test_sustain_null_verstummt_obwohl_gehalten(run_gb, tmp_path):
    """Das Klavier: Decay auf 0, danach ist NICHTS mehr da -- auch wenn die
    Note noch eine halbe Sekunde 'gehalten' wird. Genau das kann AUDIO_SFX
    nicht ausdruecken."""
    run_gb(KOPF + 'AUDIO_SAVE_WAV(AUDIO_NOTE("triangle", 220, 800, 2, 200, 0.0, 0, 0.8), "n.wav")\n',
           base=tmp_path)
    a, sr = _samples(tmp_path / "n.wav")
    h = _huelle(a, sr)
    assert h[1] > 0.4, "kurz nach dem Anschlag ist die Note laut"
    assert h[40:].max() < 0.02, "nach 400 ms ist sie weg, obwohl 800 ms gehalten"


def test_sustain_pegel_bleibt_stehen(run_gb, tmp_path):
    """Die Orgel-Variante mit halbem Pegel: nach dem Decay bleibt die Note
    bei 0.5 -- weder weg noch voll."""
    run_gb(KOPF + 'AUDIO_SAVE_WAV(AUDIO_NOTE("square", 220, 800, 0, 100, 0.5, 0, 1.0), "n.wav")\n',
           base=tmp_path)
    a, sr = _samples(tmp_path / "n.wav")
    h = _huelle(a, sr)
    assert 0.45 < h[30] < 0.56 and 0.45 < h[70] < 0.56, h[[30, 70]]


def test_release_faellt_auf_null(run_gb, tmp_path):
    run_gb(KOPF + 'AUDIO_SAVE_WAV(AUDIO_NOTE("square", 220, 200, 0, 0, 1.0, 300, 1.0), "n.wav")\n',
           base=tmp_path)
    a, sr = _samples(tmp_path / "n.wav")
    h = _huelle(a, sr)
    # 0..200 ms voll, dann linear auf 0 bis 500 ms
    assert h[10] > 0.9
    assert 0.40 < h[34] < 0.62, h[34]      # Mitte des Ausklingens (350 ms -> 0.5)
    assert h[-1] < 0.1


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


def test_fehler_im_klartext(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'DIM s AS SOUND\ns = AUDIO_NOTE("harfe", 220, 100, 0, 0, 1.0, 0, 1.0)\n')
    assert "AUDIO_NOTE" in str(e.value) and "harfe" in str(e.value)
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'DIM s AS SOUND\ns = AUDIO_NOTE("sine", 220, 0, 0, 0, 1.0, 0, 1.0)\n')
    assert "dauer_ms" in str(e.value)


# ------------------------------------------------------------ Mischen
def test_mischen_addiert_an_der_richtigen_stelle(run_gb, tmp_path):
    run_gb(KOPF + 'DIM m AS SOUND\nm = AUDIO_SOUND_NEW(1000)\n'
           'AUDIO_SOUND_MIX(m, AUDIO_TONE(440, 200, "square", 0.5), 300)\n'
           'AUDIO_SAVE_WAV(m, "m.wav")\n', base=tmp_path)
    a, sr = _samples(tmp_path / "m.wav")
    assert a.shape == (sr, 1), "Mono-Quelle ohne pan bleibt mono, Leinwand 1 s"
    h = _huelle(a, sr)
    assert h[:29].max() < 0.01, "vor 300 ms Stille"
    assert h[31:49].min() > 0.4, "300..500 ms der Ton"
    assert h[52:].max() < 0.01, "danach wieder Stille"


def test_ueberlagerung_wird_nicht_geklemmt_und_normalize_holt_sie_zurueck(run_gb, tmp_path):
    """Zwei gleiche Toene uebereinander ergeben 1.6 -- ueber 1.0, und das
    bleibt so stehen. NORMALIZE bringt die Spitze auf 1.0 und meldet den
    Faktor; erst dann wird gesichert."""
    out = run_gb(KOPF + 'DIM m AS SOUND\nm = AUDIO_SOUND_NEW(500)\n'
                 'DIM t AS SOUND\nt = AUDIO_TONE(440, 400, "square", 0.8)\n'
                 'AUDIO_SOUND_MIX(m, t, 0)\nAUDIO_SOUND_MIX(m, t, 0)\n'
                 'PRINT FORMAT$(AUDIO_SOUND_NORMALIZE(m), "%.3f")\n'
                 'AUDIO_SAVE_WAV(m, "m.wav")\n', base=tmp_path)
    assert out.strip() == "0.625"          # 1.0 / 1.6
    a, _sr = _samples(tmp_path / "m.wav")
    assert abs(np.abs(a).max() - 1.0) < 0.01


def test_pan_verteilt_auf_links_und_rechts(run_gb, tmp_path):
    run_gb(KOPF + 'DIM m AS SOUND\nm = AUDIO_SOUND_NEW(400)\n'
           'AUDIO_SOUND_MIX(m, AUDIO_TONE(440, 300, "square", 0.5), 0, 1.0, -1.0)\n'
           'AUDIO_SAVE_WAV(m, "m.wav")\n', base=tmp_path)
    a, _sr = _samples(tmp_path / "m.wav")
    assert a.shape[1] == 2, "mit pan wird die Datei stereo"
    assert np.abs(a[:, 0]).max() > 0.45 and np.abs(a[:, 1]).max() < 0.01


def test_was_ueber_das_ende_ragt_faellt_weg(run_gb, tmp_path):
    """Kein Fehler -- ein Ausklingen hinter dem letzten Takt ist normal."""
    run_gb(KOPF + 'DIM m AS SOUND\nm = AUDIO_SOUND_NEW(200)\n'
           'AUDIO_SOUND_MIX(m, AUDIO_TONE(440, 1000), 150)\n'
           'AUDIO_SOUND_MIX(m, AUDIO_TONE(440, 100), 5000)\n'
           'AUDIO_SAVE_WAV(m, "m.wav")\n', base=tmp_path)
    a, sr = _samples(tmp_path / "m.wav")
    assert abs(a.shape[0] / sr - 0.2) < 0.005


def test_in_sich_selbst_mischen_ist_ein_fehler(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'DIM m AS SOUND\nm = AUDIO_SOUND_NEW(200)\nAUDIO_SOUND_MIX(m, m, 0)\n')
    assert "derselbe" in str(e.value)
