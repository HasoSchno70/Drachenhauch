"""Kira-Modulatoren: LFO + Tweener (Etappe 5 des Ausbaus).

Der Kern-Test misst wirklich: ein Rauschen laeuft auf dem SFX-Bus, ein LFO
faehrt den Filter-Cutoff, und ueber den FFT-Abgriff wird geprueft, dass der
Hoehenanteil des AUSGANGS tatsaechlich schwankt. Ohne diese Messung koennte
man nur belegen, dass die Aufrufe nicht abstuerzen.

Braucht ein Audio-Geraet -- ohne eines (CI/headless) wird uebersprungen.
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    return next((_ROOT / "rust" / "gb_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "gb_runtime" / "target" / v / exe).exists()), None)


_GBRT = _find_gbrt()
pytestmark = pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")

_NO_DEVICE = "Audio-Geraet konnte nicht initialisiert werden"


def _run(src: str, tmp_path, timeout=90):
    p = tmp_path / "t.gb"
    p.write_text('IMPORT "audio"\n' + src, encoding="utf-8")
    r = subprocess.run([str(_GBRT), "run", str(p)], capture_output=True, text=True,
                       encoding="utf-8", timeout=timeout, cwd=str(tmp_path))
    if _NO_DEVICE in (r.stderr or ""):
        pytest.skip("kein Audio-Geraet in dieser Umgebung")
    r.out = [w for ln in (r.stdout or "").splitlines()
             if not ln.startswith(("WARNING:", "INFO:")) for w in ln.split()]
    return r


# --------------------------------------------------------- der eigentliche Test
def test_lfo_actually_sweeps_the_filter(tmp_path):
    # Belegt, dass Kira den Modulator wirklich auf den Bus-Parameter legt:
    # der gemessene Hoehenanteil des Ausgangs muss deutlich schwanken.
    #
    # Schwelle 0.45 ist gemessen, nicht geraten: ueber je 5 Laeufe lag die
    # Schwankung OHNE Modulation bei 0.065-0.218 (Rauschen zappelt von selbst)
    # und MIT Modulation bei 0.736-0.830. 0.45 liegt mittig in dieser Luecke.
    r = _run('DIM n AS SOUND\nn = AUDIO_NOISE(4000)\n'
             'DIM ch AS AUDIO_CHANNEL\nch = AUDIO_PLAY(n, -1, 1.0)\n'
             'DIM lfo AS AUDIO_MOD\nlfo = AUDIO_LFO_NEW("sine", 2.0, 1.0, 0.0)\n'
             'AUDIO_MODULATE("sfx", "filter", lfo, 200.0, 8000.0)\n'
             'DIM bands[16] AS FLOAT\n'
             'DIM i AS INTEGER\nDIM hoch AS FLOAT\nDIM tief AS FLOAT\n'
             'FOR i = 0 TO 40\n'
             '    SLEEP(25)\n'
             '    AUDIO_FFT(bands)\n'
             '    DIM h AS FLOAT\n'
             '    h = bands[13] + bands[14] + bands[15]\n'
             '    IF h > hoch THEN hoch = h\n'
             '    IF i = 0 OR h < tief THEN tief = h\n'
             'NEXT\n'
             'PRINT (hoch - tief) > 0.45\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE"], f"Hoehenanteil schwankte nicht: {r.stdout}"


def test_without_modulation_the_spectrum_stays_put(tmp_path):
    # Gegenprobe zum Test oben: ohne AUDIO_MODULATE muss die Schwankung klar
    # unter der Schwelle bleiben -- sonst wuerde der Test oben nur das
    # Eigenzappeln des Rauschens messen und auch ohne die Funktion bestehen.
    r = _run('DIM n AS SOUND\nn = AUDIO_NOISE(4000)\n'
             'DIM ch AS AUDIO_CHANNEL\nch = AUDIO_PLAY(n, -1, 1.0)\n'
             'AUDIO_FILTER("sfx", 2000.0)\n'
             'DIM bands[16] AS FLOAT\n'
             'DIM i AS INTEGER\nDIM hoch AS FLOAT\nDIM tief AS FLOAT\n'
             'FOR i = 0 TO 40\n'
             '    SLEEP(25)\n'
             '    AUDIO_FFT(bands)\n'
             '    DIM h AS FLOAT\n'
             '    h = bands[13] + bands[14] + bands[15]\n'
             '    IF h > hoch THEN hoch = h\n'
             '    IF i = 0 OR h < tief THEN tief = h\n'
             'NEXT\n'
             'PRINT (hoch - tief) < 0.45\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE"], f"Spektrum schwankte ohne Modulation: {r.stdout}"


# ----------------------------------------------------------------- Handhabung
def test_lfo_and_tweener_can_be_created_and_removed(tmp_path):
    r = _run('DIM a AS AUDIO_MOD\na = AUDIO_LFO_NEW("sine", 3.0)\n'
             'DIM b AS AUDIO_MOD\nb = AUDIO_TWEENER_NEW(0.5)\n'
             'PRINT a >= 0\nPRINT b >= 0\nPRINT a <> b\n'
             'AUDIO_MOD_REMOVE(a)\nAUDIO_MOD_REMOVE(b)\nPRINT "ok"\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE", "TRUE", "TRUE", "ok"]


def test_all_waveforms_are_accepted(tmp_path):
    names = ["sine", "sinus", "triangle", "dreieck", "saw", "saegezahn",
             "pulse", "puls", "square", "rechteck", "SINE"]
    src = "".join(f'AUDIO_LFO_NEW("{n}", 1.0)\n' for n in names) + 'PRINT "ok"\n'
    r = _run(src, tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["ok"]


def test_unknown_waveform_is_rejected(tmp_path):
    r = _run('AUDIO_LFO_NEW("wobbel", 1.0)\n', tmp_path)
    assert r.returncode != 0
    assert "AUDIO_LFO" in r.stderr and "wobbel" in r.stderr


def test_negative_frequency_is_rejected_before_touching_the_device(tmp_path):
    # Wertpruefung liegt bewusst VOR der Audio-Initialisierung (wie bei
    # AUDIO_CLOCK_NEW) -- damit auch ohne Geraet pruefbar.
    p = tmp_path / "n.gb"
    p.write_text('IMPORT "audio"\nAUDIO_LFO_NEW("sine", -1.0)\n', encoding="utf-8")
    r = subprocess.run([str(_GBRT), "run", str(p)], capture_output=True, text=True,
                       encoding="utf-8", timeout=60, cwd=str(tmp_path))
    assert r.returncode != 0
    assert "AUDIO_LFO_NEW" in r.stderr and "negativ" in r.stderr


def test_tweener_call_on_an_lfo_says_so(tmp_path):
    # Beide teilen sich den Handle-Typ AUDIO_MOD -- die Meldung muss erklaeren,
    # warum der Aufruf nicht passt, statt nur "ungueltig" zu sagen.
    r = _run('DIM m AS AUDIO_MOD\nm = AUDIO_LFO_NEW("sine", 2.0)\n'
             'AUDIO_TWEENER_TO(m, 1.0, 100.0)\n', tmp_path)
    assert r.returncode != 0
    assert "LFO" in r.stderr and "Tweener" in r.stderr


def test_lfo_call_on_a_tweener_says_so(tmp_path):
    r = _run('DIM m AS AUDIO_MOD\nm = AUDIO_TWEENER_NEW(0.0)\n'
             'AUDIO_LFO_SET(m, 5.0)\n', tmp_path)
    assert r.returncode != 0
    assert "Tweener" in r.stderr and "LFO" in r.stderr


def test_bad_handles_are_rejected(tmp_path):
    for call in ("AUDIO_MOD_REMOVE(99)", 'AUDIO_LFO_SET(99, 1.0)',
                 'AUDIO_MODULATE("sfx", "filter", 99, 100.0, 200.0)'):
        r = _run(call + "\n", tmp_path)
        assert r.returncode != 0, call
        assert "AUDIO_MOD" in r.stderr


def test_unknown_modulation_target_and_bus_are_rejected(tmp_path):
    r = _run('DIM m AS AUDIO_MOD\nm = AUDIO_LFO_NEW("sine", 1.0)\n'
             'AUDIO_MODULATE("sfx", "lautstaerke", m, 0.0, 1.0)\n', tmp_path)
    assert r.returncode != 0 and "AUDIO_MODULATE" in r.stderr

    r2 = _run('DIM m AS AUDIO_MOD\nm = AUDIO_LFO_NEW("sine", 1.0)\n'
              'AUDIO_MODULATE("gitarre", "filter", m, 0.0, 1.0)\n', tmp_path)
    assert r2.returncode != 0


def test_all_modulation_targets_are_accepted(tmp_path):
    src = ('DIM m AS AUDIO_MOD\nm = AUDIO_LFO_NEW("sine", 1.0)\n'
           'AUDIO_MODULATE("sfx", "filter", m, 200.0, 4000.0)\n'
           'AUDIO_MODULATE("sfx", "resonance", m, 0.0, 10.0)\n'
           'AUDIO_MODULATE("music", "reverb", m, 0.0, 1.0)\n'
           'AUDIO_MODULATE("master", "distortion", m, 0.0, 12.0)\n'
           'PRINT "ok"\n')
    r = _run(src, tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["ok"]


def test_tweener_runs_and_accepts_easings(tmp_path):
    src = ('DIM m AS AUDIO_MOD\nm = AUDIO_TWEENER_NEW(0.0)\n'
           'AUDIO_MODULATE("sfx", "filter", m, 200.0, 8000.0)\n'
           + "".join(f'AUDIO_TWEENER_TO(m, 1.0, 20.0, "{e}")\n'
                     for e in ("linear", "in", "out", "inout", ""))
           + 'PRINT "ok"\n')
    r = _run(src, tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["ok"]


def test_unknown_easing_is_rejected(tmp_path):
    r = _run('DIM m AS AUDIO_MOD\nm = AUDIO_TWEENER_NEW(0.0)\n'
             'AUDIO_TWEENER_TO(m, 1.0, 10.0, "huepfend")\n', tmp_path)
    assert r.returncode != 0 and "huepfend" in r.stderr
