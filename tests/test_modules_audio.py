"""Tests fuer die reine Synth-Mathematik (`drachenhauch.synth`).

Stufe B: Die Audio-Wiedergabe/Synthese-Builtins (AUDIO_TONE/NOISE/SFX/...) laufen
nur nativ in dhrt; der frueher hier getestete Tree-Walker-"nur nativ"-Gate
entfaellt mit dem Tree-Walker (Phase 8). Was bleibt, ist die geteilte Synth-
Mathematik in `drachenhauch/synth.py` (von Builtin UND dhsfx-Export genutzt, reines
numpy -- in Phase 8 behalten).
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

def test_music_play_stop_fade_validation(run_gb):
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_MUSIC_PLAY(-1, -5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_MUSIC_STOP(-1)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_MUSIC_PLAY: fade_in_ms muss >= 0 sein" in out
    assert "AUDIO_MUSIC_STOP: fade_out_ms muss >= 0 sein" in out


def test_play_stop_fade_validation(run_gb):
    # AUDIO_PLAY(sound[, loops[, volume[, fade_in_ms]]]) / AUDIO_STOP(ch[, fade_out_ms])
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PLAY(0, -1, 1.0, -5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_STOP(0, -1)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_PLAY: fade_in_ms muss >= 0 sein" in out
    assert "AUDIO_STOP: fade_out_ms muss >= 0 sein" in out


def test_pan_slide_validation(run_gb):
    # AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms) -- dauer_ms wird im Wrapper
    # (vor der Audio-Initialisierung) geprueft -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PAN_SLIDE(0, 0.0, 1.0, 0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    assert "AUDIO_PAN_SLIDE: dauer_ms muss > 0 sein" in run_gb(src)


def test_pitch_validation(run_gb):
    # faktor <= 0 wird im Wrapper (vor der Audio-Initialisierung) geprueft.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PITCH(0, 0.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_MUSIC_PITCH(-1.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_PITCH: faktor muss > 0 sein" in out
    assert "AUDIO_MUSIC_PITCH: faktor muss > 0 sein" in out


def test_sample_type_compiles(run_gb):
    # SAMPLE ist ein externer Typ des audio-Moduls. DIM ... AS SAMPLE
    # initialisiert KEIN Audio-Geraet -> headless golden-testbar (verifiziert
    # die Typ-Verdrahtung preprocess MODULE_TYPES + Compiler).
    src = '\n'.join([
        'IMPORT "audio"',
        'DIM s AS SAMPLE',
        'PRINT "sample-typ ok"',
    ])
    assert "sample-typ ok" in run_gb(src)


def test_lofi_validation(run_gb):
    # AUDIO_LOFI(an[, bits[, cutoff_hz]]) -- Argument-Pruefung laeuft VOR der
    # Audio-Initialisierung -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_LOFI(TRUE, 99)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_LOFI(TRUE, 8, -1.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_LOFI: bits muss 1..16 sein" in out
    assert "AUDIO_LOFI: cutoff_hz muss >= 0 sein" in out


def test_bus_volume_validation(run_gb):
    # AUDIO_BUS_VOLUME/GET: unbekannter Bus wird VOR der Audio-Init geprueft
    # -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_BUS_VOLUME("foo", 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    PRINT STR$(AUDIO_BUS_GET_VOLUME("bar"))',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_BUS_VOLUME: unbekannter Bus 'foo' (sfx, music, master)" in out
    assert "AUDIO_BUS_GET_VOLUME: unbekannter Bus 'bar' (sfx, music, master)" in out


def test_bus_effects_validation(run_gb):
    # AUDIO_FILTER/REVERB/DELAY: unbekannter Bus wird VOR der Audio-Init
    # geprueft -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_FILTER("nope", 1000, 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_REVERB("nope", 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_DELAY("nope", 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_FILTER: unbekannter Bus 'nope' (sfx, music, master)" in out
    assert "AUDIO_REVERB: unbekannter Bus 'nope' (sfx, music, master)" in out
    assert "AUDIO_DELAY: unbekannter Bus 'nope' (sfx, music, master)" in out


def test_bus_dynamics_eq_validation(run_gb):
    # AUDIO_DISTORTION/COMPRESSOR/EQ: unbekannter Bus -> klare Meldung vor
    # der Audio-Init (golden-testbar).
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_DISTORTION("nope", 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_COMPRESSOR("nope", -18, 4)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_EQ("nope", 100, 3)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_DISTORTION: unbekannter Bus 'nope' (sfx, music, master)" in out
    assert "AUDIO_COMPRESSOR: unbekannter Bus 'nope' (sfx, music, master)" in out
    assert "AUDIO_EQ: unbekannter Bus 'nope' (sfx, music, master)" in out


def test_unloadsound_is_wired(run_gb):
    # UNLOADSOUND(s) gibt einen Sound-Puffer frei. Der Arg-Check (gi) laeuft
    # VOR der Audio-Initialisierung -> headless golden-testbar und beweist,
    # dass das Builtin verdrahtet ist (sonst "unbekannter Builtin").
    src = '\n'.join([
        'TRY',
        '    UNLOADSOUND()',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    assert "UNLOADSOUND: fehlendes Argument 1" in run_gb(src)


def test_audio_clock_type_compiles(run_gb):
    # AUDIO_CLOCK ist ein externer Typ des audio-Moduls (Kira-Uhr fuer
    # sample-genaues Musik-/Rhythmus-Timing). DIM ... AS AUDIO_CLOCK
    # initialisiert KEIN Audio-Geraet -> headless golden-testbar (verifiziert
    # preprocess MODULE_TYPES + Compiler-Verdrahtung).
    src = '\n'.join([
        'IMPORT "audio"',
        'DIM c AS AUDIO_CLOCK',
        'PRINT "audio-clock-typ ok"',
    ])
    assert "audio-clock-typ ok" in run_gb(src)


def test_audio_clock_new_validation(run_gb):
    # AUDIO_CLOCK_NEW(ticks_per_second) -- Wertpruefung laeuft VOR der
    # Audio-Initialisierung -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_CLOCK_NEW(0.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_CLOCK_NEW(-2.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert out.count("AUDIO_CLOCK_NEW: ticks_per_second muss > 0 sein") == 2


def test_audio_clock_set_speed_validation(run_gb):
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_CLOCK_SET_SPEED(0, -1.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    assert "AUDIO_CLOCK_SET_SPEED: ticks_per_second muss > 0 sein" in run_gb(src)


def test_audio_play_at_validation(run_gb):
    # AUDIO_PLAY_AT(sound, clock, ticks[, volume[, loops]]) -- ticks < 0 wird
    # VOR der Audio-Initialisierung geprueft -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PLAY_AT(0, 0, -1)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    assert "AUDIO_PLAY_AT: ticks muss >= 0 sein" in run_gb(src)


def test_easing_validation_on_fade_builtins(run_gb):
    # Nicht-lineare Tween-Easings (Kira: Easing::{In,Out,InOut}Powi) fuer
    # Fades/Slides -- optionaler trailing easing$-Parameter, unbekannter Name
    # wird VOR der Audio-Initialisierung geprueft -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PLAY(0, 0, 1.0, 100, "bounce")',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_STOP(0, 100, "bounce")',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_PAN_SLIDE(0, 0.0, 1.0, 100, "bounce")',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_MUSIC_PLAY(-1, 100, "bounce")',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_MUSIC_STOP(100, "bounce")',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_PLAY: unbekanntes Easing 'bounce' (linear, in, out, inout)" in out
    assert "AUDIO_STOP: unbekanntes Easing 'bounce' (linear, in, out, inout)" in out
    assert "AUDIO_PAN_SLIDE: unbekanntes Easing 'bounce' (linear, in, out, inout)" in out
    assert "AUDIO_MUSIC_PLAY: unbekanntes Easing 'bounce' (linear, in, out, inout)" in out
    assert "AUDIO_MUSIC_STOP: unbekanntes Easing 'bounce' (linear, in, out, inout)" in out


def test_easing_defaults_to_linear_when_omitted(run_gb):
    # Ohne easing$-Argument bleibt das bestehende Verhalten unveraendert --
    # die alten fade_in_ms/fade_out_ms/dauer_ms-Validierungen laufen weiter,
    # ohne dass ein Easing-Fehler dazwischenfunkt.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PLAY(0, -1, 1.0, -5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_PAN_SLIDE(0, 0.0, 1.0, 0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_PLAY: fade_in_ms muss >= 0 sein" in out
    assert "AUDIO_PAN_SLIDE: dauer_ms muss > 0 sein" in out


def test_audio_listener_emitter_types_compile(run_gb):
    # AUDIO_LISTENER/AUDIO_EMITTER sind externe Typen des audio-Moduls.
    # DIM ... AS ... initialisiert KEIN Audio-Geraet -> headless golden-
    # testbar (verifiziert preprocess MODULE_TYPES + Compiler-Verdrahtung).
    src = '\n'.join([
        'IMPORT "audio"',
        'DIM l AS AUDIO_LISTENER',
        'DIM e AS AUDIO_EMITTER',
        'PRINT "listener/emitter-typ ok"',
    ])
    assert "listener/emitter-typ ok" in run_gb(src)


def test_audio_emitter_new_validation(run_gb):
    # AUDIO_EMITTER_NEW(listener, x, y, z[, min_dist[, max_dist]]) --
    # min_dist/max_dist-Pruefung laeuft VOR der Audio-Initialisierung ->
    # headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_EMITTER_NEW(0, 0.0, 0.0, 0.0, -1.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_EMITTER_NEW(0, 0.0, 0.0, 0.0, 10.0, 5.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_EMITTER_NEW: min_dist muss >= 0 sein" in out
    assert "AUDIO_EMITTER_NEW: max_dist muss > min_dist sein" in out


def test_audio_play_on_validation(run_gb):
    # AUDIO_PLAY_ON(sound, emitter[, loops[, volume[, fade_in_ms[, easing$]]]])
    # -- fade_in_ms/easing$-Pruefung laeuft VOR der Audio-Initialisierung.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PLAY_ON(0, 0, 0, 1.0, -5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_PLAY_ON(0, 0, 0, 1.0, 100, "bounce")',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_PLAY_ON: fade_in_ms muss >= 0 sein" in out
    assert "AUDIO_PLAY_ON: unbekanntes Easing 'bounce' (linear, in, out, inout)" in out


# --- AUDIO_MUSIC_SEEK --------------------------------------------------------
# Die Zahlenpruefung liegt im Wrapper (vm.rs) und laeuft VOR der Audio-
# Initialisierung -- deshalb auch ohne Soundkarte nachweisbar.

def test_music_seek_validation(run_gb):
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_MUSIC_SEEK(-1.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    assert "AUDIO_MUSIC_SEEK: Position muss >= 0 sein (war -1)" in run_gb(src)


def test_music_seek_ohne_musik_meldet_sich(run_gb):
    """Der Sprung braucht einen laufenden Griff -- den gibt es erst ab PLAY.

    Waere das ein stiller Nicht-Treffer (wie bei PAUSE/RESUME), wuerde
    `AUDIO_MUSIC_LOAD(...) : AUDIO_MUSIC_SEEK(30.0) : AUDIO_MUSIC_PLAY()`
    lautlos bei 0 anfangen. Braucht ein Audio-Geraet (die Meldung kommt aus
    audio.rs, also nach der Initialisierung).
    """
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_MUSIC_SEEK(1.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    assert "AUDIO_MUSIC_SEEK: es laeuft keine Musik" in run_gb(src)


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

def test_unloadsound_zaehler_und_tombstone(run_gb):
    """UNLOADSOUND gibt frei, der Handle bleibt gueltig (Tombstone) und ein
    erneutes PLAYSOUND darauf meldet sich im Klartext."""
    out = run_gb('IMPORT "audio"\n'
                 'DIM t AS SOUND\nDIM v AS SOUND\n'
                 't = AUDIO_TONE(440, 30, "square", 0.3)\n'
                 'PRINT AUDIO_SOUND_COUNT()\n'
                 'UNLOADSOUND(t)\n'
                 'PRINT AUDIO_SOUND_COUNT()\n'
                 'TRY\n'
                 '    PLAYSOUND(t)\n'
                 '    PRINT "kein Fehler"\n'
                 'CATCH e\n'
                 '    PRINT e\n'
                 'END TRY\n'
                 'v = AUDIO_TONE(880, 30, "sine", 0.3)\n'
                 'PRINT STR$(v) = STR$(t)\n')
    zeilen = [z for z in out.splitlines() if z.strip()]
    assert zeilen[0] == "1" and zeilen[1] == "0"
    assert "freigegeben" in zeilen[2], zeilen
    assert zeilen[3] == "FALSE", "Handle wurde recycelt -- Tombstone kaputt"


def test_lautstaerke_ist_linear_und_wird_geklemmt(run_gb):
    """Die Doku sagt '0..1' und (im Anhang) dass Kira intern in Dezibel
    rechnet -- nach aussen muss es linear bleiben, und Werte darueber werden
    geklemmt statt zu werfen."""
    out = run_gb('IMPORT "audio"\n'
                 'DIM t AS SOUND\nDIM ch AS AUDIO_CHANNEL\n'
                 't = AUDIO_TONE(440, 50, "sine", 0.3)\n'
                 'ch = AUDIO_PLAY(t, -1, 1.0)\n'
                 'AUDIO_VOLUME(ch, 0.25)\n'
                 'PRINT AUDIO_GET_VOLUME(ch)\n'
                 'AUDIO_VOLUME(ch, 5.0)\n'
                 'PRINT AUDIO_GET_VOLUME(ch)\n'
                 'AUDIO_STOP(ch)\n')
    assert [z for z in out.split() if z] == ["0.25", "1.0"]


def test_loops_semantik(run_gb):
    """`0` spielt einmal, `-1` endlos -- gemessen an einem 60-ms-Ton."""
    out = run_gb('IMPORT "audio"\n'
                 'DIM t AS SOUND\nDIM ch AS AUDIO_CHANNEL\n'
                 't = AUDIO_TONE(440, 60, "sine", 0.2)\n'
                 'ch = AUDIO_PLAY(t, 0, 0.2)\n'
                 'SLEEP(250)\n'
                 'PRINT AUDIO_IS_PLAYING(ch)\n'
                 'ch = AUDIO_PLAY(t, -1, 0.2)\n'
                 'SLEEP(250)\n'
                 'PRINT AUDIO_IS_PLAYING(ch)\n'
                 'AUDIO_STOP(ch)\n')
    assert [z for z in out.split() if z] == ["FALSE", "TRUE"]


def test_music_seek_ohne_play_meldet_sich(run_gb):
    """Doku: 'der Aufruf meldet dann einen Fehler, statt stillschweigend
    nichts zu tun' -- sonst begaenne das Stueck unbemerkt wieder bei 0."""
    out = run_gb('IMPORT "audio"\n'
                 'TRY\n'
                 '    AUDIO_MUSIC_SEEK(10.0)\n'
                 '    PRINT "kein Fehler"\n'
                 'CATCH e\n'
                 '    PRINT e\n'
                 'END TRY\n')
    assert "keine Musik" in out, out
