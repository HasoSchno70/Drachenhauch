"""Tests fuer das Note-Off-Event (Qt-frei + Editor).

Note-Off (`NOTE_OFF = -1`, gespeichert im selben `data[c][r]`-Feld wie eine
echte Note) schneidet eine klingende Note VOR der naechsten Note im Kanal ab,
statt sie -- wie bisher IMMER -- bis zur naechsten Note durchklingen zu lassen.
Live-GB-Export (kein Sustain-Konzept dort) behandelt es wie eine leere Zelle;
der WAV-Mixer nutzt es als Sustain-Grenze ohne selbst Klang zu erzeugen."""
import numpy as np
import pytest

from gamebasic.tracker import NOTE_OFF, Pattern, Song
from gamebasic.tracker.mixer import render_song, _note_events


def test_note_off_clears_vol_slide_fx():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_vol(0, 0, 10)
    p.set_slide(0, 0, 5)
    from gamebasic.tracker.song import FX_ARP
    p.set_fx(0, 0, FX_ARP, 0x47)
    p.set(0, 0, NOTE_OFF)
    assert p.data[0][0] == NOTE_OFF
    assert p.vol[0][0] is None
    assert p.slide[0][0] is None
    assert p.get_fx(0, 0) == (0, 0)


def test_flatten_treats_note_off_as_silence():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set(0, 4, NOTE_OFF)
    total, channels = s.flatten()
    assert channels[0][0] != 0
    assert channels[0][4] == 0          # kein Ton fuer Note-Off im Live-Player


def test_gb_code_no_trigger_on_note_off_row():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set(0, 4, NOTE_OFF)
    code = s.gb_code()
    assert "trk0[0] = " in code
    assert "trk0[4] = " not in code     # 0 -> kein Zuweisungs-Statement noetig


def test_mixer_note_events_includes_note_off_as_boundary():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set(0, 4, NOTE_OFF)
    events = _note_events(s)
    rows = [e[0] for e in events[0]]
    assert 0 in rows and 4 in rows


def test_render_song_cuts_note_short_at_note_off():
    """Ein Note-Off VOR der naechsten Note muss die Sustain-Laenge tatsaechlich
    verkuerzen (verglichen mit einer Fassung ohne Note-Off)."""
    s_off = Song()
    s_off.patterns[0].set_rows(16)
    s_off.patterns[0].set(0, 0, 60)
    s_off.patterns[0].set(0, 4, NOTE_OFF)
    s_off.patterns[0].set(0, 12, 60)
    mix_off = render_song(s_off, sr=8000, tail_ms=0)

    s_full = Song()
    s_full.patterns[0].set_rows(16)
    s_full.patterns[0].set(0, 0, 60)
    s_full.patterns[0].set(0, 12, 60)
    mix_full = render_song(s_full, sr=8000, tail_ms=0)

    # Nach dem Note-Off (Reihe 4) muss der erste Take nahezu Stille sein,
    # waehrend der zweite (kein Note-Off) noch klingt.
    row_samples = int(8000 * s_off.row_ms() / 1000.0)
    probe = row_samples * 6          # deutlich hinter dem Note-Off bei Reihe 4
    assert abs(mix_off[probe]) < 1e-3
    assert np.abs(mix_full[probe]).sum() > 0 or np.abs(mix_full[probe - 5:probe + 5]).sum() > 0


def test_render_song_note_off_itself_produces_no_extra_sound():
    """Ein Note-Off-Event darf selbst keinen Klick/Ton beitragen -- nur die
    vorherige Note abschneiden."""
    s = Song()
    s.patterns[0].set(0, 0, NOTE_OFF)   # Note-Off ohne vorherige Note
    mix = render_song(s, sr=8000, tail_ms=0)
    assert np.abs(mix).max() < 1e-6 or mix.size == 0 or np.allclose(mix, 0.0)


def test_editor_set_note_off_button():
    import os
    os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
    os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
    from pathlib import Path
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    from gamebasic.trackereditor_qt import TrackerEditor
    ed = TrackerEditor(Path("."))
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    ed.grid.setCurrentCell(0, 0)
    ed._set_note_off()
    assert pat.data[0][0] == NOTE_OFF
    assert "OFF" in ed._cell_text(0, NOTE_OFF)


def test_editor_zero_key_sets_note_off():
    import os
    os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
    os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
    from pathlib import Path
    from PySide6.QtCore import Qt
    from PySide6.QtTest import QTest
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    from gamebasic.trackereditor_qt import TrackerEditor
    ed = TrackerEditor(Path("."))
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    ed.grid.setCurrentCell(0, 0)
    QTest.keyClick(ed, Qt.Key.Key_0)
    assert pat.data[0][0] == NOTE_OFF


def test_block_transpose_skips_note_off():
    from gamebasic.tracker import block_transpose
    p = Pattern("P", rows=2)
    p.set(0, 0, NOTE_OFF)
    block_transpose(p, 0, 0, 0, 0, 12)
    assert p.data[0][0] == NOTE_OFF


def test_block_interpolate_ignores_note_off_as_endpoint():
    """Note-Off (Reihe 4) darf NICHT als Rampen-Endpunkt herhalten (sonst
    wuerde zwischen 60 und der sinnlosen "Tonhoehe" -1 interpoliert). Reihe 0
    (60) und Reihe 8 (72) bleiben die gueltigen Endpunkte, die Rampe laeuft
    also ueber die Note-Off-Zelle bei Reihe 4 hinweg (bleibt dort stehen)."""
    from gamebasic.tracker import block_interpolate
    p = Pattern("P", rows=9)
    p.set(0, 0, 60)
    p.set(0, 4, NOTE_OFF)
    p.set(0, 8, 72)
    block_interpolate(p, 0, 0, 0, 8)
    assert p.data[0][0] == 60
    assert p.data[0][4] == NOTE_OFF        # bereits belegt -> von der Rampe uebersprungen
    assert p.data[0][8] == 72
    for r in (1, 2, 3, 5, 6, 7):
        v = p.data[0][r]
        assert v is not None and 60 <= v <= 72   # zwischen den echten Endpunkten
