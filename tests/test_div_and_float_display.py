"""D1 + D2 aus docs/gamebasic-stolpersteine.md.

D1: `/` bleibt unveraendert (INTEGER wenn glatt, sonst FLOAT) -- aber die
    Fehlermeldung beim Zuweisen eines FLOAT-Ergebnisses an eine INTEGER-Variable
    weist jetzt auf `\\` (Ganzzahl-Division) hin.
D2: f32-gestuetzte Audio-Lautstaerken werden gerundet ausgegeben (0.8 statt
    0.800000011920929). Die allgemeine Float-Ausgabe bleibt unveraendert (korrekt).
"""
import pytest
from gamebasic.errors import GameBasicError


# --- D1 -----------------------------------------------------------------

def test_division_unchanged(run_gb):
    # Bewusstes Design bleibt: glatt -> INTEGER, sonst FLOAT.
    assert run_gb("PRINT 8 / 2\nPRINT 9 / 2\nPRINT 6 / 3\n") == "4\n4.5\n2\n"


def test_float_into_integer_hints_backslash(run_gb):
    with pytest.raises(GameBasicError, match=r"ganzzahlige Division \\ statt /"):
        run_gb("DIM x AS INTEGER\nx = 9 / 2\n")


def test_backslash_is_the_fix(run_gb):
    # Der vorgeschlagene Weg funktioniert.
    assert run_gb("DIM x AS INTEGER\nx = 9 \\ 2\nPRINT x\n") == "4\n"


# --- D2 -----------------------------------------------------------------

def test_general_float_display_unchanged(run_gb):
    # Kuerzeste round-trip-Form bleibt -- das ist korrekt, kein Bug.
    assert run_gb("PRINT 0.1 + 0.2\n") == "0.30000000000000004\n"


def test_audio_bus_volume_rounded(run_gb):
    out = run_gb('IMPORT "audio"\n'
                 'AUDIO_BUS_VOLUME("music", 0.8)\n'
                 'PRINT AUDIO_BUS_GET_VOLUME("music")\n')
    assert out == "0.8\n"


def test_audio_channel_volume_rounded(run_gb):
    out = run_gb('IMPORT "audio"\n'
                 'DIM s AS SOUND\n'
                 's = AUDIO_TONE(440.0, 100, "sine", 0.5)\n'
                 'DIM ch AS INTEGER\n'
                 'ch = AUDIO_PLAY(s)\n'
                 'AUDIO_SET_VOLUME(ch, 0.35)\n'
                 'PRINT AUDIO_GET_VOLUME(ch)\n')
    assert out == "0.35\n"
