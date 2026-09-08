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

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/audio_sound_io.dhtest` (dhrt test); hier bleiben nur die, die sich
mit den Proben von `--- ton` nicht ausdruecken lassen.
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
