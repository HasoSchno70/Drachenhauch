"""`SPEAK` -- Sprachausgabe als Klang unter Klaengen (docs/entwurf-speak.md, Weg C).

Der Pruefstein aus dem Entwurf: `SPEAK_SOUND` liefert einen Klang mit
Huellkurve, `AUDIO_SAVE_WAV` schreibt ihn, Pythons `wave`-Modul liest ihn
zurueck -- ein FREMDER Leser, nicht der eigene Schreiber. `SPEAKING` geht an
und wieder aus; Anhaengen dauert doppelt so lang wie Unterbrechen; eine
unbekannte Stimme ist ein Fehler im Klartext.

Braucht die Systemstimme (Windows: WinRT; macOS: `say`; Linux: `espeak-ng`)
und ein Audio-Geraet. Fehlt eines davon, wird uebersprungen -- die Meldung
sagt, was fehlt. Die Dauern werden gemessen, deshalb in `_SERIELL`.
"""
import wave
from pathlib import Path

import numpy as np
import pytest

from drachenhauch.errors import DHRuntimeError


def _samples(pfad: Path):
    with wave.open(str(pfad)) as w:
        n, ch, sw, sr = w.getnframes(), w.getnchannels(), w.getsampwidth(), w.getframerate()
        roh = w.readframes(n)
    assert sw == 2
    a = np.frombuffer(roh, "<i2").astype(np.float32) / 32767.0
    return a.reshape(-1, ch), sr


@pytest.fixture
def speak(run_gb):
    """run_gb, das ohne Sprachausgabe oder Audio-Geraet ueberspringt statt zu fallen."""
    def _run(src, base=None):
        try:
            return run_gb(src, base=base)
        except DHRuntimeError as e:
            m = str(e)
            if any(x in m for x in ("nicht gefunden", "keine Sprachausgabe", "Audio-Geraet",
                                    "keine Systemstimme")):
                pytest.skip("keine Sprachausgabe auf dieser Maschine: " + m.strip().splitlines()[-1][:120])
            raise
    return _run


def test_gesprochenes_ist_ein_klang_den_ein_fremder_leser_liest(speak, tmp_path):
    speak('DIM s AS SOUND\n'
          's = SPEAK_SOUND("Willkommen bei Drachenhauch. Der Schatz liegt im Norden.")\n'
          'AUDIO_SAVE_WAV(s, "t.wav")\n', base=tmp_path)
    a, sr = _samples(tmp_path / "t.wav")
    assert sr >= 8000, sr
    assert a.shape[1] == 1, "Sprache ist mono"
    dauer = a.shape[0] / sr
    assert 1.0 < dauer < 8.0, dauer
    # Eine Huellkurve, kein Strich: laut in der Mitte, still an den Raendern.
    assert float(np.abs(a).max()) > 0.1
    rand = float(np.abs(a[: sr // 100]).max())
    mitte = float(np.abs(a[len(a) // 3: 2 * len(a) // 3]).max())
    assert mitte > 0.05 and rand < mitte, (rand, mitte)


def test_derselbe_satz_kommt_aus_dem_vorrat(speak):
    out = speak('PRINT SPEAK_SOUND("Treffer") = SPEAK_SOUND("Treffer")\n'
                'DIM a AS SOUND : a = SPEAK_SOUND("Treffer")\n'
                'SPEAK_RATE(1.5)\n'
                'PRINT a = SPEAK_SOUND("Treffer")\n')
    assert out == "TRUE\nFALSE\n", out


def test_ein_freigegebener_klang_wird_neu_gerechnet(speak):
    # UNLOADSOUND auf den Vorrat darf SPEAK nicht auf einen toten Slot laufen lassen.
    out = speak('DIM a AS SOUND : a = SPEAK_SOUND("Treffer")\n'
                'UNLOADSOUND(a)\n'
                'SPEAK("Treffer")\n'
                'PRINT SPEAKING()\n')
    assert out == "TRUE\n", out


def test_speaking_geht_an_und_wieder_aus(speak):
    out = speak('SPEAK("Eins zwei drei")\n'
                'PRINT SPEAKING()\n'
                'SPEAK_WAIT()\n'
                'PRINT SPEAKING()\n')
    assert out == "TRUE\nFALSE\n", out


def test_stop_macht_sofort_still(speak):
    out = speak('SPEAK("Eins zwei drei vier fuenf sechs")\n'
                'SPEAK_STOP()\n'
                'PRINT SPEAKING()\n')
    assert out == "FALSE\n", out


def test_anhaengen_reiht_unterbrechen_ersetzt(speak):
    """Zwei angehaengte Saetze dauern rund doppelt so lang wie einer, der den
    ersten unterbricht. Gemessen ueber SPEAK_WAIT -- die Warteschlange laeuft
    auf Kiras Audio-Faden, nicht ueber FLIP."""
    out = speak('DIM t AS INTEGER\n'
                't = MILLIS()\n'
                'SPEAK("Der Drache erwacht")\n'
                'SPEAK("Der Drache erwacht")\n'
                'SPEAK_WAIT()\n'
                'DIM zwei AS INTEGER : zwei = MILLIS() - t\n'
                't = MILLIS()\n'
                'SPEAK("Der Drache erwacht")\n'
                'SPEAK("Der Drache erwacht", TRUE)\n'
                'SPEAK_WAIT()\n'
                'DIM eins AS INTEGER : eins = MILLIS() - t\n'
                'PRINT zwei ; " " ; eins\n')
    zwei, eins = [int(x) for x in out.split()]
    assert eins > 300, (zwei, eins)
    assert zwei > 1.6 * eins, f"angehaengt {zwei} ms, unterbrochen {eins} ms"


def test_stimmen_liste_und_wahl(speak):
    out = speak('DIM v AS ARRAY OF STRING\n'
                'v = SPEAK_VOICES()\n'
                'PRINT LEN(v) > 0\n'
                'SPEAK_VOICE(v[0])\n'
                'SPEAK_VOICE(UPPER$(v[0]))\n'
                'SPEAK_VOICE("")\n'
                'PRINT SPEAK_SOUND("Hallo") >= 0\n')
    assert out == "TRUE\nTRUE\n", out


def test_unbekannte_stimme_ist_ein_fehler_im_klartext(speak):
    with pytest.raises(DHRuntimeError, match="SPEAK_VOICE: Stimme 'Niemand' nicht gefunden"):
        speak('SPEAK_VOICE("Niemand")\n')


def test_tempo_hat_grenzen(run_gb):
    # Reine Pruefung, braucht keine Stimme: der Fehler kommt vor der Synthese.
    with pytest.raises(DHRuntimeError, match=r"SPEAK_RATE: faktor muss 0\.5 \.\. 2\.0"):
        run_gb('SPEAK_RATE(5)\n')


def test_sprache_hat_einen_eigenen_bus(run_gb):
    out = run_gb('AUDIO_BUS_VOLUME("speech", 0.3)\n'
                 'PRINT AUDIO_BUS_GET_VOLUME("speech")\n'
                 'AUDIO_PUSH()\n'
                 'AUDIO_BUS_VOLUME("speech", 1.0)\n'
                 'AUDIO_POP()\n'
                 'PRINT AUDIO_BUS_GET_VOLUME("speech")\n')
    assert out == "0.3\n0.3\n", out
