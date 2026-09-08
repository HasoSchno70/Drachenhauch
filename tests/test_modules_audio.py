"""Tests fuer die reine Synth-Mathematik (`drachenhauch.synth`).

Stufe B: Die Audio-Wiedergabe/Synthese-Builtins (AUDIO_TONE/NOISE/SFX/...) laufen
nur nativ in dhrt; der frueher hier getestete Tree-Walker-"nur nativ"-Gate
entfaellt mit dem Tree-Walker (Phase 8). Was bleibt, ist die geteilte Synth-
Mathematik in `drachenhauch/synth.py` (von Builtin UND dhsfx-Export genutzt, reines
numpy -- in Phase 8 behalten).

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/modules_audio.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
from pathlib import Path

import numpy as np

from drachenhauch.synth import synthesize, svf_lowpass

_ROOT = Path(__file__).resolve().parent.parent


def test_synth_duty_default_unchanged():
    # duty=0.5 (Default) muss bit-genau die alte sign(sin)-Rechteckwelle sein.
    old = synthesize("square", 440.0, 0.0, 0, 100, 0, sr=44100)
    new = synthesize("square", 440.0, 0.0, 0, 100, 0, sr=44100, duty=0.5)
    assert np.array_equal(old, new)


def test_synth_narrow_duty_changes_wave():
    # Schmale Pulsbreite -> ueberwiegend bei -1 (anderer Klang).
    w = synthesize("square", 220.0, 0.0, 0, 100, 0, sr=44100, duty=0.1)
    assert float(np.mean(w)) < -0.3


def test_synth_filter_attenuates_high_freq():
    # 8 kHz durch tiefen Cutoff -> zweite Haelfte deutlich leiser.
    raw = synthesize("saw", 8000.0, 0.0, 0, 100, 0, sr=44100)
    flt = synthesize("saw", 8000.0, 0.0, 0, 100, 0, sr=44100, flt_cutoff=300.0)
    h = len(raw) // 2
    assert np.abs(flt[h:]).max() < np.abs(raw[h:]).max() * 0.6


def test_svf_bypass_when_cutoff_zero():
    sig = np.array([0.3, -0.7, 0.5, -0.2])
    assert np.array_equal(svf_lowpass(sig, 0.0, 0.0, 0.0, 44100), sig)


def test_synth_stereo_shape_and_channels():
    mono = synthesize("saw", 1000, -1400, 0, 30, 150)
    st = synthesize("saw", 1000, -1400, 0, 30, 150, stereo_width=0.6)
    assert mono.ndim == 1
    assert st.ndim == 2 and st.shape[1] == 2
    assert not np.allclose(st[:, 0], st[:, 1])      # Detune -> L != R
    # Noise: L/R dekorreliert
    nst = synthesize("noise", 200, 0, 0, 50, 100, stereo_width=0.5)
    assert not np.allclose(nst[:, 0], nst[:, 1])


def test_synth_matches_envelope_shape():
    w = synthesize("square", 440.0, 0.0, 0, 50, 50, sr=44100)
    assert w.shape[0] == int(44100 * 100 / 1000)
    assert abs(w[-1]) < 0.1         # Decay laeuft am Ende auf ~0 aus
    assert np.abs(w).max() <= 1.0
    # Mit Attack-Ramp startet das Signal bei ~0.
    wa = synthesize("square", 440.0, 0.0, 30, 30, 30, sr=44100)
    assert abs(wa[0]) < 0.1


# --- AUDIO_MUSIC_PLAY/STOP: Argument-Validierung (dhrt-Golden) ---------------
# Die Wiedergabe selbst braucht ein Audio-Geraet (nicht headless testbar);
# die Wrapper-Validierung in vm.rs laeuft aber VOR der Audio-Initialisierung
# und ist damit golden-testbar.


# --- AUDIO_MUSIC_SEEK --------------------------------------------------------
# Die Zahlenpruefung liegt im Wrapper (vm.rs) und laeuft VOR der Audio-
# Initialisierung -- deshalb auch ohne Soundkarte nachweisbar.


def test_music_seek_springt_wirklich(run_gb, tmp_path):
    """Position vor und nach dem Sprung -- der eigentliche Nachweis.

    **Ein Sprung wirkt nicht sofort.** Gemessen (zweimal identisch, in
    100-ms-Schritten abgefragt): die Position bleibt noch ~300 ms auf dem
    alten Wert und steht ab ~400 ms auf der neuen Stelle. Das ist der
    gepufferte Vorlauf des Streams, der erst leergespielt wird. Der erste
    Anlauf dieses Tests wartete 200 ms und schlug deshalb fehl -- die 800 ms
    hier sind der doppelte gemessene Abstand.

    Zwei Abfragen, weil die erste allein nichts beweist: `POSITION() < 2.0`
    ist auch dann wahr, wenn ueberhaupt nichts spielt.

    Braucht ein Audio-Geraet.
    """
    import shutil
    shutil.copy(_ROOT / "examples" / "assets" / "ambient.ogg", tmp_path / "m.ogg")
    src = '\n'.join([
        'IMPORT "audio"',
        'AUDIO_MUSIC_LOAD("m.ogg")',
        'AUDIO_MUSIC_PLAY()',
        'SLEEP(300)',
        'PRINT AUDIO_MUSIC_POSITION() > 0.1',      # laeuft ueberhaupt
        'PRINT AUDIO_MUSIC_POSITION() < 2.0',      # noch am Anfang
        'AUDIO_MUSIC_SEEK(5.0)',
        'SLEEP(800)',
        'PRINT AUDIO_MUSIC_POSITION() > 4.0',      # nach dem Sprung
    ])
    assert run_gb(src, base=tmp_path).split() == ["TRUE", "TRUE", "TRUE"]


def test_music_seek_auf_modul_sagt_es_klar(run_gb, tmp_path):
    """MOD/XM haben keine Sekunden-Achse -- das muss der Aufrufer erfahren.

    Braucht ein Audio-Geraet (die Meldung kommt aus audio.rs).
    """
    import shutil
    shutil.copy(_ROOT / "examples" / "assets" / "demo.mod", tmp_path / "m.mod")
    src = '\n'.join([
        'IMPORT "audio"',
        'AUDIO_MUSIC_LOAD("m.mod")',
        'AUDIO_MUSIC_PLAY()',
        'TRY',
        '    AUDIO_MUSIC_SEEK(5.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src, base=tmp_path)
    assert "MOD-/XM-Musik laesst sich nicht auf eine Sekunde setzen" in out


# --- Zusagen aus docs/module-audio.md, die vorher kein Test festhielt ------
#
# Beim Nachmessen der Doku fiel auf: die Vorgabewerte und die
# loops-Semantik standen zwar beschrieben, aber nichts pinnte sie fest. Genau
# so konnte der Mixer-Abschnitt jahrelang eine Kanal-Vorgabe von 8 behaupten,
# waehrend es 16 sind.
