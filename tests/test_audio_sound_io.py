"""`AUDIO_SAVE_WAV` und `AUDIO_SOUND_WAVE` -- Klang als Datei und als Kurve.

Beide hängen am SOUND-Handle, nicht an `AUDIO_SFX`: sie gelten damit für
jede Klangquelle. Gebaut wurden sie für den SFX-Generator in Drachenhauch
(`examples/160_sfx_generator.dh`) — ohne sie kann ein Klang-Werkzeug weder
seine Wellenform zeigen noch etwas abliefern.

**Die WAV-Dateien werden von einem FREMDEN Leser gegengelesen** (Pythons
`wave`-Modul), nicht von der Runtime selbst. Ein Format, das nur der eigene
Schreiber wieder lesen kann, ist nicht geprüft — es ist nur in sich
konsistent.

Braucht einen dhrt mit Audio, steht darum in `conftest._BRAUCHT_GRAFIK`.
"""
import wave
from pathlib import Path

import numpy as np
import pytest


def _samples(pfad: Path):
    """WAV mit Pythons `wave`-Modul lesen -> (array (n, kanaele), sr, bits)."""
    with wave.open(str(pfad)) as w:
        n, ch, sw, sr = w.getnframes(), w.getnchannels(), w.getsampwidth(), w.getframerate()
        roh = w.readframes(n)
    if sw == 2:
        a = np.frombuffer(roh, "<i2").astype(np.float32) / 32767.0
    else:
        # 8-bit-WAV ist laut Spezifikation VORZEICHENLOS, 16-bit vorzeichenbehaftet.
        a = (np.frombuffer(roh, np.uint8).astype(np.float32) - 128.0) / 127.0
    return a.reshape(-1, ch), sr, sw * 8


_MUENZE = ('IMPORT "audio"\n'
           'DIM s AS SOUND\n'
           's = AUDIO_SFX("square", 900, 600, 0, 40, 160, 0, 0, 0.7)\n')


def test_wav_ist_fuer_einen_fremden_leser_gueltig(run_gb, tmp_path):
    run_gb(_MUENZE + 'AUDIO_SAVE_WAV(s, "t.wav")\n', base=tmp_path)
    a, sr, bits = _samples(tmp_path / "t.wav")
    assert sr == 44100 and bits == 16
    assert a.shape[1] == 1, "Mono-Klang muss einkanalig gespeichert werden"
    # 0 + 40 + 160 ms Huellkurve -> 200 ms
    assert abs(a.shape[0] / sr - 0.200) < 0.01, a.shape


def test_lautstaerke_wird_nicht_zweimal_gerechnet(run_gb, tmp_path):
    """Der Fehler, den erst das Nachmessen zeigte.

    Die Lautstärke aus `AUDIO_SFX` steckt bereits in den Frames. Wurde
    zusätzlich die Abspiel-Lautstärke des Slots aufmultipliziert, landete
    `0.7` als `0.49` in der Datei -- hörbar leiser als das, was das Programm
    abspielt, und in einem Werkzeug für Spiele-Effekte genau das Falsche.
    """
    run_gb(_MUENZE + 'AUDIO_SAVE_WAV(s, "t.wav")\n', base=tmp_path)
    a, _sr, _bits = _samples(tmp_path / "t.wav")
    spitze = float(np.abs(a).max())
    assert abs(spitze - 0.7) < 0.01, f"Spitze {spitze:.3f}, erwartet 0.70 (0.49 = zweimal gerechnet)"


def test_acht_bit_ist_kleiner_und_klingt_noch_gleich(run_gb, tmp_path):
    run_gb(_MUENZE + 'AUDIO_SAVE_WAV(s, "a.wav")\nAUDIO_SAVE_WAV(s, "b.wav", 8)\n',
           base=tmp_path)
    gross = (tmp_path / "a.wav").stat().st_size
    klein = (tmp_path / "b.wav").stat().st_size
    assert klein < gross / 1.9, (klein, gross)
    a8, _sr, bits = _samples(tmp_path / "b.wav")
    assert bits == 8
    a16, _sr, _b = _samples(tmp_path / "a.wav")
    # Dieselbe Kurve, nur gröber aufgelöst.
    assert np.abs(a8[:, 0] - a16[:, 0]).max() < 0.02


def test_stereo_bleibt_stereo(run_gb, tmp_path):
    run_gb('IMPORT "audio"\n'
           'DIM s AS SOUND\n'
           's = AUDIO_SFX("saw", 500, -300, 0, 60, 200, 0, 0, 0.6, 0.8)\n'
           'AUDIO_SAVE_WAV(s, "t.wav")\n', base=tmp_path)
    a, _sr, _bits = _samples(tmp_path / "t.wav")
    assert a.shape[1] == 2
    assert not np.array_equal(a[:, 0], a[:, 1]), "beide Kanäle identisch -- Verstimmung fehlt"


def test_mono_wird_nicht_unnoetig_verdoppelt(run_gb, tmp_path):
    """`stereo_width = 0` legt links und rechts denselben Wert -- daraus eine
    doppelt so grosse Datei zu machen wäre eine Überraschung ohne Gegenwert."""
    run_gb(_MUENZE + 'AUDIO_SAVE_WAV(s, "t.wav")\n', base=tmp_path)
    a, _sr, _bits = _samples(tmp_path / "t.wav")
    assert a.shape[1] == 1


def test_bittiefe_muss_8_oder_16_sein(run_gb, tmp_path):
    out = run_gb(_MUENZE + '''
TRY
    AUDIO_SAVE_WAV(s, "t.wav", 24)
    PRINT "kein Fehler"
CATCH e
    PRINT "abgelehnt"
END TRY
''', base=tmp_path)
    assert out.strip().splitlines()[-1] == "abgelehnt"


def test_wellenform_hat_die_gewuenschte_laenge_und_form(run_gb, tmp_path):
    out = run_gb(_MUENZE + '''
DIM w AS ARRAY OF FLOAT
w = AUDIO_SOUND_WAVE(s, 64)
DIM i AS INTEGER
DIM gross AS INTEGER
DIM ausserhalb AS INTEGER
FOR i = 0 TO 63
    IF ABS(w[i]) > 0.2 THEN gross = gross + 1
    IF ABS(w[i]) > 1.0 THEN ausserhalb = ausserhalb + 1
NEXT
PRINT LEN(w); " "; gross; " "; ausserhalb
''', base=tmp_path)
    laenge, gross, ausserhalb = out.strip().split()
    assert laenge == "64"
    assert ausserhalb == "0", "Werte müssen in -1..1 liegen"
    # Der Kern: Spitzenwerte, keine Mittelwerte. Über 44100 Samples auf 64
    # Punkte gemittelt läge fast alles bei 0 und die Anzeige zeigte einen
    # Strich -- genau darum liefert die Routine je Abschnitt das Sample mit
    # dem grössten Betrag.
    assert int(gross) > 40, f"nur {gross} von 64 Punkten deutlich != 0 -- gemittelt statt Spitze?"


def test_wellenform_feiner_als_der_klang_lang_ist(run_gb, tmp_path):
    """Mehr Punkte als Samples: jeder Abschnitt muss trotzdem mindestens ein
    Sample sehen, sonst wäre er leer und der Wert fiele auf 0."""
    out = run_gb('''
IMPORT "audio"
DIM s AS SOUND
s = AUDIO_SFX("square", 400, 0, 0, 1, 0, 0, 0, 0.9)
DIM w AS ARRAY OF FLOAT
w = AUDIO_SOUND_WAVE(s, 90000)
PRINT LEN(w)
''', base=tmp_path)
    assert out.strip() == "90000"


def test_anzahl_muss_sinnvoll_sein(run_gb, tmp_path):
    out = run_gb(_MUENZE + '''
TRY
    DIM w AS ARRAY OF FLOAT
    w = AUDIO_SOUND_WAVE(s, 0)
    PRINT "kein Fehler"
CATCH e
    PRINT "abgelehnt"
END TRY
''', base=tmp_path)
    assert out.strip().splitlines()[-1] == "abgelehnt"
