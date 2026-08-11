"""Tests fuer die Sample-Instrument-Integration im Tracker-Editor (offscreen)."""
import os
import wave
from pathlib import Path

import numpy as np
import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _editor():
    try:
        from drachenhauch.trackereditor_qt import TrackerEditor
        return TrackerEditor(Path("."))
    except Exception as exc:  # pragma: no cover
        pytest.skip(f"Editor nicht konstruierbar: {exc}")


def _write_wav(path, freq=440, secs=0.1, sr=44100):
    t = np.arange(int(sr * secs)) / sr
    i16 = (np.sin(2 * np.pi * freq * t) * 30000).astype("<i2")
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(sr)
        w.writeframes(i16.tobytes())


def test_load_sample_file_creates_instrument(tmp_path):
    ed = _editor()
    p = tmp_path / "kick.wav"
    _write_wav(p)
    inst = ed._instrument_from_file(str(p))
    assert inst is not None and inst.kind == "sample"
    assert inst.name == "kick"


def test_editor_has_factory_presets():
    ed = _editor()
    # Out of the box: Presets im Pool + jedem Kanal ein Sound zugewiesen
    assert len(ed.song.instruments) >= 14
    names = [i.name for i in ed.song.instruments]
    assert "Fluegel (Piano)" in names and "Kick" in names
    for c in range(4):
        assert ed.song.channel_inst[c] is not None
    assert ed.sound_combos[0].count() == len(ed.song.instruments)


def test_assign_instrument_to_channel_via_sound_combo(tmp_path):
    ed = _editor()
    p = tmp_path / "lead.wav"
    _write_wav(p)
    idx = ed.song.add_instrument(ed._instrument_from_file(str(p)))
    ed._refresh_instruments()
    # Pro-Spur-Dropdown auf das neue Sample setzen -> zuweisen
    ed._on_sound_changed(0, idx)
    assert ed.song.channel_inst[0] == idx
    assert ed.song.instrument_for_channel(0).kind == "sample"
    # Preview rendert ueber das Sample (kein Crash)
    ed._play_note(0, 60)


def test_remove_instrument_updates_combo(tmp_path):
    ed = _editor()
    n0 = len(ed.song.instruments)
    p = tmp_path / "s.wav"; _write_wav(p)
    ed.song.add_instrument(ed._instrument_from_file(str(p)))
    ed._refresh_instruments()
    assert len(ed.song.instruments) == n0 + 1
    ed.inst_combo.setCurrentIndex(n0)      # das neue (letzte) Instrument
    ed._remove_instrument()
    assert len(ed.song.instruments) == n0


def test_note_length_in_playback():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set_rows(8)
    pat.set(0, 0, 60)            # Note in Reihe 0, naechste erst spaeter
    # Laenge = Reihen bis naechste Note (oder Pattern-Ende)
    assert ed._note_len_rows(pat, 0, 0) == 8
    pat.set(0, 3, 64)
    assert ed._note_len_rows(pat, 0, 0) == 3
    # _play_columns nutzt die Laenge ohne Fehler
    ed._play_columns(pat, 0)


def test_note_length_sustains_into_next_order_pattern_in_song_mode():
    """Review-Fund: im "Play Song"-Modus wurde die Notenlaenge bisher hart
    am Pattern-Ende gekappt, obwohl der WAV-Export (mixer.py) dieselbe Note
    ueber die Pattern-Grenze hinweg sustained, wenn im naechsten Order-
    Pattern auf demselben Kanal keine fruehere Note kommt -- die Live-
    Vorschau klang dadurch kuerzer/anders als das exportierte Ergebnis."""
    ed = _editor()
    p0 = ed.song.patterns[0]
    p0.set_rows(4)
    p0.set(0, 2, 60)                     # Note 2 Reihen vor Pattern-Ende
    idx1 = ed.song.add_pattern(rows=4)
    p1 = ed.song.patterns[idx1]
    p1.set(0, 1, 64)                     # naechste Note im Folge-Pattern
    ed.song.order = [0, idx1]

    # "Play Pattern"-Modus (order_pos=None): kappt weiterhin hart am Ende.
    assert ed._note_len_rows(p0, 0, 2) == 2
    # "Play Song"-Modus: sustained bis zur ersten Note im naechsten
    # Order-Pattern (2 Restreihen in p0 + 1 Reihe in p1 = 3).
    assert ed._note_len_rows(p0, 0, 2, order_pos=0) == 3


def test_note_length_song_mode_without_next_note_uses_whole_next_pattern():
    ed = _editor()
    p0 = ed.song.patterns[0]
    p0.set_rows(4)
    p0.set(0, 2, 60)
    idx1 = ed.song.add_pattern(rows=5)   # keine Note im Folge-Pattern
    ed.song.order = [0, idx1]

    assert ed._note_len_rows(p0, 0, 2, order_pos=0) == 2 + 5


def test_note_length_song_mode_wraps_order_to_own_pattern():
    """Ein 1-Pattern-Order (Loop auf sich selbst) darf nicht crashen."""
    ed = _editor()
    p0 = ed.song.patterns[0]
    p0.set_rows(4)
    p0.set(0, 2, 60)
    ed.song.order = [0]
    assert ed._note_len_rows(p0, 0, 2, order_pos=0) >= 2


def test_keymap_dialog_builds_instrument(tmp_path):
    from drachenhauch.trackereditor_qt import _KeymapDialog
    from drachenhauch.tracker import Zone
    ed = _editor()
    # zwei Samples vorbereiten + per load_fn-Callback laden
    a = tmp_path / "a.wav"; _write_wav(a, freq=200)
    b = tmp_path / "b.wav"; _write_wav(b, freq=800)
    dlg = _KeymapDialog("Kit", [], ed._load_samples_from_file)
    # Zonen direkt einspeisen (statt File-Dialog) ueber das interne API
    sa, sra = ed._load_samples_from_file(str(a))
    sb, srb = ed._load_samples_from_file(str(b))
    import numpy as np
    dlg._zones.append(Zone(samples=np.asarray(sa, np.float32), sample_rate=sra,
                           root_note=60, lo_key=0, hi_key=59, name="a"))
    dlg._zones.append(Zone(samples=np.asarray(sb, np.float32), sample_rate=srb,
                           root_note=60, lo_key=60, hi_key=127, name="b"))
    dlg._rebuild_table()
    dlg._auto_drumkit()                      # jede Zone -> 1 Taste ab 36
    dlg._commit_table()
    zones = dlg.get_zones()
    assert zones[0].lo_key == 36 and zones[0].hi_key == 36
    assert zones[1].lo_key == 37
    # Im Editor anwenden
    from drachenhauch.tracker import Instrument
    inst = Instrument.keymap(dlg.get_name(), zones)
    ed.song.add_instrument(inst)
    ed._refresh_instruments()
    assert ed.song.instruments[-1].is_keymap()
    assert "▦" in ed.inst_combo.itemText(ed.inst_combo.count() - 1)


def _build_min_sf2(tmp_path):
    """Minimale gueltige SF2 (1 Preset -> 1 Zone -> 1 Sample)."""
    import struct
    def ck(tag, data):
        out = tag + struct.pack("<I", len(data)) + data
        return out + (b"\x00" if len(data) & 1 else b"")
    def lst(lt, *subs):
        body = lt + b"".join(subs)
        return b"LIST" + struct.pack("<I", len(body)) + body
    def nm(s):
        return s.encode("latin-1")[:20].ljust(20, b"\x00")
    samples = np.linspace(-20000, 20000, 200).astype("<i2").tobytes()
    sdta = lst(b"sdta", ck(b"smpl", samples))
    phdr = (nm("TestPiano") + struct.pack("<HHHIII", 0, 0, 0, 0, 0, 0)
            + nm("EOP") + struct.pack("<HHHIII", 0, 0, 1, 0, 0, 0))
    pbag = struct.pack("<HH", 0, 0) + struct.pack("<HH", 1, 0)
    pgen = struct.pack("<HH", 41, 0) + struct.pack("<HH", 0, 0)
    inst = (nm("TestInst") + struct.pack("<H", 0)
            + nm("EOI") + struct.pack("<H", 1))
    ibag = struct.pack("<HH", 0, 0) + struct.pack("<HH", 2, 0)
    igen = (struct.pack("<H", 43) + bytes([0, 127])
            + struct.pack("<H", 53) + struct.pack("<H", 0)
            + struct.pack("<HH", 0, 0))
    shdr = (nm("TestSample") + struct.pack("<IIIII", 0, 200, 50, 150, 22050)
            + struct.pack("<BbHH", 60, 0, 0, 1)
            + nm("EOS") + struct.pack("<IIIII", 0, 0, 0, 0, 0)
            + struct.pack("<BbHH", 0, 0, 0, 0))
    pdta = lst(b"pdta", ck(b"phdr", phdr), ck(b"pbag", pbag), ck(b"pgen", pgen),
               ck(b"inst", inst), ck(b"ibag", ibag), ck(b"igen", igen),
               ck(b"shdr", shdr))
    body = b"sfbk" + sdta + pdta
    p = tmp_path / "test.sf2"
    p.write_bytes(b"RIFF" + struct.pack("<I", len(body)) + body)
    return str(p)


def test_load_soundfont_adds_instrument(tmp_path, monkeypatch):
    """SF2 laden -> Preset-Dialog -> Keymap-Instrument im Pool."""
    from PySide6.QtWidgets import QFileDialog, QDialog
    from drachenhauch.trackereditor_qt import _Sf2PresetDialog
    path = _build_min_sf2(tmp_path)
    ed = _editor()
    n0 = len(ed.song.instruments)
    monkeypatch.setattr(QFileDialog, "getOpenFileName",
                        staticmethod(lambda *a, **k: (path, "")))
    # Dialog automatisch "akzeptieren" mit Preset (0,0)
    monkeypatch.setattr(_Sf2PresetDialog, "exec",
                        lambda self: QDialog.DialogCode.Accepted)
    monkeypatch.setattr(_Sf2PresetDialog, "selected", lambda self: (0, 0))
    ed._load_soundfont()
    assert len(ed.song.instruments) == n0 + 1
    inst = ed.song.instruments[-1]
    assert inst.is_keymap() and inst.name == "TestPiano"


def test_export_audio_renders_wav(tmp_path, monkeypatch):
    import wave
    from PySide6.QtWidgets import QFileDialog
    ed = _editor()
    ed.song.patterns[0].set(0, 0, 60)
    out = tmp_path / "song.wav"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        staticmethod(lambda *a, **k: (str(out), "")))
    # Info-Dialog unterdruecken
    from PySide6.QtWidgets import QMessageBox
    monkeypatch.setattr(QMessageBox, "information",
                        staticmethod(lambda *a, **k: None))
    # Render-Optionen-Dialog (modal) ueberspringen -> Mono.
    monkeypatch.setattr(ed, "_ask_render_options", lambda: (False, False))
    ed._export_audio()
    assert out.exists()
    with wave.open(str(out), "rb") as w:
        assert w.getnframes() > 0
        assert w.getnchannels() == 1


def test_mute_solo_audibility():
    ed = _editor()
    assert all(ed._audible(c) for c in range(4))     # Default: alle hoerbar
    ed.mute_btns[0].setChecked(True)
    assert not ed._audible(0) and ed._audible(1)
    ed.mute_btns[0].setChecked(False)
    ed.solo_btns[2].setChecked(True)                 # Solo -> nur Kanal 2
    assert ed._audible(2) and not ed._audible(0) and not ed._audible(1)
    ed.mute_btns[2].setChecked(True)                 # Mute schlaegt Solo
    assert not ed._audible(2)


def test_vu_meter_lights_and_decays():
    ed = _editor()
    ed._reset_vu()
    assert ed.vu_level[0] == 0.0
    ed._play_note(0, 60)                 # rendert + setzt VU-Pegel
    assert ed.vu_level[0] > 0.0
    before = ed.vu_level[0]
    ed._vu_decay()
    assert ed.vu_level[0] < before       # klingt ab
    ed._reset_vu()
    assert ed.vu_level[0] == 0.0 and ed.vu_meters[0].value() == 0


def test_fx_column_sets_effect_on_cell():
    from drachenhauch.tracker.song import FX_ARP, FX_CODES
    ed = _editor()
    ed.grid.setCurrentCell(0, 0)
    ed._set_note(0, 0, 60)
    # Effekt + Parameter ueber die UI-Widgets setzen
    ed.fx_combo.setCurrentIndex(FX_CODES.index(FX_ARP))
    ed.fxp_spin.setValue(0x47)
    assert ed.song.patterns[ed.cur].get_fx(0, 0) == (FX_ARP, 0x47)
    # Zellentext zeigt den Effekt
    assert "Arp" in ed.grid.item(0, 0).text()
    # Re-Selektion spiegelt den Effekt zurueck in die Widgets
    ed.grid.setCurrentCell(0, 1)
    ed.grid.setCurrentCell(0, 0)
    assert ed.fx_combo.currentData() == FX_ARP
    assert ed.fxp_spin.value() == 0x47


def test_export_audio_stereo_hard_pan(tmp_path, monkeypatch):
    import wave
    from PySide6.QtWidgets import QFileDialog, QMessageBox
    ed = _editor()
    ed.song.patterns[0].set(0, 0, 60)
    out = tmp_path / "song_st.wav"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        staticmethod(lambda *a, **k: (str(out), "")))
    monkeypatch.setattr(QMessageBox, "information",
                        staticmethod(lambda *a, **k: None))
    monkeypatch.setattr(ed, "_ask_render_options", lambda: (True, True))
    ed._export_audio()
    with wave.open(str(out), "rb") as w:
        assert w.getnchannels() == 2


def test_instrument_dialog_applies_loop_and_env(tmp_path):
    from drachenhauch.trackereditor_qt import _InstrumentDialog
    ed = _editor()
    p = tmp_path / "pad.wav"; _write_wav(p, secs=0.2)
    inst = ed._instrument_from_file(str(p))
    dlg = _InstrumentDialog(inst)
    dlg.base.setValue(72)
    dlg.loop_mode.setCurrentText("pingpong")
    dlg.loop_start.setValue(100)
    dlg.loop_end.setValue(3000)
    dlg.atk.setValue(20)
    dlg.rel.setValue(40)
    dlg.sus.setValue(0.7)
    dlg.apply_to()
    assert inst.base_note == 72
    assert inst.loop_mode == "pingpong"
    assert inst.loop_start == 100 and inst.loop_end == 3000
    assert inst.has_loop() is True
    assert inst.env_attack_ms == 20 and inst.env_release_ms == 40
    assert abs(inst.env_sustain - 0.7) < 1e-6


def test_instrument_dialog_pan_slider_applies_and_labels(tmp_path):
    """Pan ist jetzt ein Schieberegler (-100..100) statt einer Spinbox --
    apply_to() muss weiterhin das erwartete -1.0..1.0-Instrument-Feld setzen,
    und das Label soll L/R/Mitte anzeigen."""
    from drachenhauch.trackereditor_qt import _InstrumentDialog
    ed = _editor()
    p = tmp_path / "pad.wav"; _write_wav(p, secs=0.2)
    inst = ed._instrument_from_file(str(p))
    dlg = _InstrumentDialog(inst)
    assert dlg.pan_label.text() == "Mitte"       # Default-Pan 0.0
    dlg.pan_slider.setValue(-40)
    assert dlg.pan_label.text() == "L 40%"
    dlg.apply_to()
    assert inst.pan == pytest.approx(-0.4)
    dlg.pan_slider.setValue(70)
    assert dlg.pan_label.text() == "R 70%"
    dlg.apply_to()
    assert inst.pan == pytest.approx(0.7)


def test_instrument_dialog_pan_slider_reflects_existing_instrument_pan(tmp_path):
    from drachenhauch.trackereditor_qt import _InstrumentDialog
    ed = _editor()
    p = tmp_path / "pad.wav"; _write_wav(p, secs=0.2)
    inst = ed._instrument_from_file(str(p))
    inst.pan = -0.5
    dlg = _InstrumentDialog(inst)
    assert dlg.pan_slider.value() == -50
    assert dlg.pan_label.text() == "L 50%"


def test_instrument_dialog_waveform_view_syncs_from_spinboxes(tmp_path):
    """Spinbox-Aenderung muss die Wellenform-Marker nachziehen (Anzeige)."""
    from drachenhauch.trackereditor_qt import _InstrumentDialog
    ed = _editor()
    p = tmp_path / "pad.wav"; _write_wav(p, secs=0.2)
    inst = ed._instrument_from_file(str(p))
    dlg = _InstrumentDialog(inst)
    dlg.loop_start.setValue(500)
    dlg.loop_end.setValue(4000)
    assert dlg.wave_view.loop_start == 500
    assert dlg.wave_view.loop_end == 4000


def test_instrument_dialog_waveform_drag_updates_spinboxes(tmp_path):
    """Ziehen am Wellenform-Marker (simuliert per set_loop + Signal, wie ein
    echter Drag es ausloesen wuerde) muss die Spinboxen nachziehen."""
    from drachenhauch.trackereditor_qt import _InstrumentDialog
    ed = _editor()
    p = tmp_path / "pad.wav"; _write_wav(p, secs=0.2)
    inst = ed._instrument_from_file(str(p))
    dlg = _InstrumentDialog(inst)
    dlg.wave_view.loop_changed.emit(777, 5000)
    assert dlg.loop_start.value() == 777
    assert dlg.loop_end.value() == 5000


def test_waveform_view_drag_via_mouse_events(tmp_path):
    """End-to-end: echte Maus-Events auf dem _WaveformView-Widget bewegen
    den Loop-Ende-Marker (Regressionstest fuer die Hit-Test-/Drag-Logik)."""
    from PySide6.QtCore import QPointF, Qt
    from PySide6.QtGui import QMouseEvent
    from drachenhauch.trackereditor_qt import _WaveformView
    import numpy as np
    samples = np.sin(np.linspace(0, 20, 2000)).astype(np.float32)
    w = _WaveformView(samples, loop_start=0, loop_end=1000)
    w.resize(400, 100)
    end_x = w._frame_to_x(1000)

    press = QMouseEvent(QMouseEvent.Type.MouseButtonPress, QPointF(end_x, 50), QPointF(end_x, 50),
                        Qt.MouseButton.LeftButton, Qt.MouseButton.LeftButton,
                        Qt.KeyboardModifier.NoModifier)
    w.mousePressEvent(press)
    assert w._drag == "end"

    move = QMouseEvent(QMouseEvent.Type.MouseMove, QPointF(end_x + 40, 50), QPointF(end_x + 40, 50),
                       Qt.MouseButton.NoButton, Qt.MouseButton.LeftButton,
                       Qt.KeyboardModifier.NoModifier)
    w.mouseMoveEvent(move)
    assert w.loop_end > 1000

    release = QMouseEvent(QMouseEvent.Type.MouseButtonRelease, QPointF(end_x + 40, 50), QPointF(end_x + 40, 50),
                          Qt.MouseButton.LeftButton, Qt.MouseButton.NoButton,
                          Qt.KeyboardModifier.NoModifier)
    w.mouseReleaseEvent(release)
    assert w._drag is None


# --- WA_DeleteOnClose auf Dialogen/Export-Fenstern (Review-Fund: fehlte -----
# --- vorher, jedes wiederholte Oeffnen sammelte hidden Kind-Widgets an) -----

def test_instrument_dialog_has_delete_on_close(tmp_path):
    from drachenhauch.trackereditor_qt import _InstrumentDialog
    from PySide6.QtCore import Qt
    ed = _editor()
    inst = ed.song.instruments[0]
    dlg = _InstrumentDialog(inst)
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_keymap_dialog_has_delete_on_close():
    from drachenhauch.trackereditor_qt import _KeymapDialog
    from PySide6.QtCore import Qt
    ed = _editor()
    dlg = _KeymapDialog("Kit", [], ed._load_samples_from_file)
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_sf2_preset_dialog_has_delete_on_close():
    from drachenhauch.trackereditor_qt import _Sf2PresetDialog
    from PySide6.QtCore import Qt
    dlg = _Sf2PresetDialog([(0, 0, "Piano")])
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_show_code_window_has_delete_on_close():
    from PySide6.QtCore import Qt
    ed = _editor()
    ed._show_code("PRINT 1")
    assert ed._code_dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_render_options_dialog_has_delete_on_close(monkeypatch):
    from PySide6.QtCore import Qt
    from PySide6.QtWidgets import QDialog
    captured = {}
    orig_exec = QDialog.exec
    def _capture_exec(self):
        captured["dlg"] = self
        return QDialog.DialogCode.Rejected
    monkeypatch.setattr(QDialog, "exec", _capture_exec)
    ed = _editor()
    ed._ask_render_options()
    assert captured["dlg"].testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
