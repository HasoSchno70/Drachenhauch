"""Das Notenblatt in Drachenhauch (`examples/199_notenblatt.dh`, Weg B).

Bedient wird mit echten Klicks ueber die Eingabe-Wiedergabe (Note setzen,
nochmal klicken = entfernen, ziehen = verschieben, Modus ueber die Klappliste),
gesichert mit Strg+S. Drei fremde Leser pruefen, was dabei herauskommt:
`drachenhauch.score.document.ScoreDoc` (das Modell des Qt-Editors) fuer die
Stueck-Datei, `drachenhauch.tracker.song.Song` fuer das Tracker-Projekt, und
`drachenhauch.score.convert.to_tracker_song` als Referenz: der Tracker-Export
des Piloten muss Zelle fuer Zelle dasselbe Gitter liefern wie der Python-
Konverter -- Akkord-Reduktion, Staccato, NOTE_OFF und Pattern-Grenze
eingeschlossen.

Braucht ein Fenster und speist Eingaben ein -- `_BRAUCHT_GRAFIK` + `_SERIELL`.
Die Testkopie laeuft OHNE `WINDOW_MAXIMIZE` (1280 x 800): Blatt ab x = 250,
Spur i beginnt bei y = 104 + i * 190, Linie 1 bei +86, Beat b bei x = 326 + 64 b.
"""
from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

import pytest

from drachenhauch.score.convert import to_tracker_song
from drachenhauch.score.document import ScoreDoc
from drachenhauch.tracker.song import Song

_ROOT = Path(__file__).resolve().parent.parent
EDITOR = _ROOT / "examples" / "199_notenblatt.dh"
DEMO = _ROOT / "examples" / "notenblatt_demo.json"


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

KEY_UP, KEY_DOWN = 1, 2
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7
RL_LCTRL, RL_S, RL_Z, RL_T, RL_SPACE = 341, 83, 90, 84, 32
LINKS, RECHTS = 0, 1

GX0, GY0 = 250, 96 + 8
HALB, PPB, STAFF_H = 7, 64, 190


def _x(beat):
    return GX0 + 76 + int(beat * PPB)


def _y(spur, stufe):
    """y einer diatonischen Stufe (0 = unterste Linie, 4 = Mittellinie)."""
    return GY0 + spur * STAFF_H + 30 + 8 * HALB - stufe * HALB


# Werkzeugleiste (Menueleiste 26 px + Inhalts-y), Klapplisten oeffnen nach unten
# mit 22 px je Eintrag.
DD_MODUS = (334 + 60, 26 + 8 + 12)
BTN_SPUR_PLUS = (860 + 35, 26 + 7 + 13)
DD_INST_0 = (8 + 117, 96 + 90 + 12)


def _dd_eintrag(dd, idx):
    return (dd[0], dd[1] + 12 + idx * 22 + 11)


def _klick(frame, x, y, knopf=LINKS):
    return [(frame, MOUSE_POSITION, x, y), (frame + 1, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_BUTTON_DOWN, knopf), (frame + 3, MOUSE_BUTTON_UP, knopf)]


def _zug(frame, x0, y0, x1, y1, knopf=LINKS, schritte=6):
    ev = [(frame, MOUSE_POSITION, x0, y0), (frame + 1, MOUSE_POSITION, x0, y0),
          (frame + 1, MOUSE_BUTTON_DOWN, knopf)]
    for k in range(1, schritte + 1):
        ev.append((frame + 1 + k, MOUSE_POSITION, x0 + (x1 - x0) * k // schritte, y0 + (y1 - y0) * k // schritte))
    ev.append((frame + 3 + schritte, MOUSE_BUTTON_UP, knopf))
    return ev


def _taste(frame, code, *halten):
    ev = [(frame, KEY_DOWN, h) for h in halten]
    ev += [(frame, KEY_DOWN, code), (frame + 2, KEY_UP, code)]
    ev += [(frame + 2, KEY_UP, h) for h in halten]
    return ev


def _editor(tmp_path, datei, frames, events):
    log = tmp_path / "score.log"
    ev = sorted(events, key=lambda e: e[0])
    zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
    for frame, typ, *params in ev:
        p = (list(params) + [0, 0, 0, 0])[:4]
        zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
    (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
    text = EDITOR.read_text(encoding="utf-8")
    text = text.replace('SETFPS(60)\n', 'SETFPS(60)\nAUTOMATION_PLAY("' + (tmp_path / "ev.txt").as_posix() + '")\n', 1)
    text = text.replace("WINDOW_MAXIMIZE()\n", "", 1)
    (tmp_path / "_nb").mkdir(exist_ok=True)
    quelle = tmp_path / "_nb" / "notenblatt_test.dh"
    quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle), "--", str(datei)], capture_output=True,
                       text=True, encoding="utf-8", errors="replace", timeout=180,
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DH_SCORE_LOG=str(log)),
                       cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    return log.read_text(encoding="utf-8").splitlines() if log.exists() else []


def _gesichert(log):
    assert any(z.startswith("gesichert ") for z in log), log


def test_editor_uebersetzt_ohne_befund():
    r = subprocess.run([str(_DHRT), "--check", str(EDITOR)], capture_output=True, text=True,
                       encoding="utf-8", timeout=120)
    assert r.returncode == 0 and r.stdout.strip() == "[]", r.stdout


def test_klick_setzt_eine_viertelnote_auf_die_mittellinie(tmp_path):
    """Mittellinie im Violinschluessel = H4 (MIDI 71); Beat 2, Viertel (Vorgabe)."""
    datei = tmp_path / "neu.json"
    ev = _klick(20, _x(2), _y(0, 4)) + _taste(50, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 90, ev)
    assert "note 0 2 71 1" in log, log
    _gesichert(log)
    doc = ScoreDoc.load_json(str(datei))
    assert len(doc.tracks) == 1 and doc.tracks[0].clef == "treble"
    n = doc.tracks[0].notes
    assert [(x.start_beat, x.dur_beat, x.pitch, x.rest) for x in n] == [(2.0, 1.0, 71, False)]
    song, warn = to_tracker_song(doc)          # der Qt-Konverter liest es
    assert warn == [] and song.patterns[0].data[0][8] == 71


def test_zweiter_klick_entfernt_und_rechtsklick_auch(tmp_path):
    datei = tmp_path / "neu.json"
    ev = _klick(20, _x(2), _y(0, 4)) + _klick(50, _x(2), _y(0, 4))       # setzen, entfernen
    ev += _klick(80, _x(4), _y(0, 6)) + _klick(110, _x(4), _y(0, 6), knopf=RECHTS)
    ev += _taste(140, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 180, ev)
    assert "entfernt 0 2" in log and "note 0 4 74 1" in log, log
    assert log.count("entfernt 0 4") == 1, log
    _gesichert(log)
    assert ScoreDoc.load_json(str(datei)).tracks[0].notes == []


def test_ziehen_verschiebt_und_strg_z_holt_zurueck(tmp_path):
    datei = tmp_path / "neu.json"
    ev = _klick(20, _x(2), _y(0, 4))
    ev += _zug(50, _x(2), _y(0, 4), _x(4), _y(0, 6))          # nach Beat 4, eine Terz hoeher
    ev += _taste(90, RL_Z, RL_LCTRL) + _taste(110, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 150, ev)
    assert "bewegt 0 4 74" in log, log
    _gesichert(log)
    assert "rueckgaengig 1" in log, log
    n = ScoreDoc.load_json(str(datei)).tracks[0].notes
    assert [(x.start_beat, x.pitch) for x in n] == [(2.0, 71)]


def test_modus_pause_und_bindebogen_ueber_die_klappliste(tmp_path):
    datei = tmp_path / "neu.json"
    ev = _klick(20, _x(0), _y(0, 4)) + _klick(40, _x(2), _y(0, 4))
    ev += _klick(60, *DD_MODUS) + _klick(70, *_dd_eintrag(DD_MODUS, 1))      # Pause
    ev += _klick(90, _x(4), _y(0, 4))
    ev += _klick(110, *DD_MODUS) + _klick(120, *_dd_eintrag(DD_MODUS, 2))    # Bindebogen
    ev += _klick(140, _x(0), _y(0, 4)) + _klick(160, _x(2), _y(0, 4))
    ev += _taste(190, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 230, ev)
    assert "pause 0 4 1" in log, log
    assert "bogen 0 0 2" in log, log
    _gesichert(log)
    t = ScoreDoc.load_json(str(datei)).tracks[0]
    assert [(x.start_beat, x.rest) for x in t.notes] == [(0.0, False), (2.0, False), (4.0, True)]
    assert t.slurs == [(0.0, 2.0)]


def test_zweite_spur_und_instrument(tmp_path):
    datei = tmp_path / "neu.json"
    ev = _klick(20, *BTN_SPUR_PLUS)
    ev += _klick(40, _x(2), _y(1, 4))                                       # Note auf Spur 2
    ev += _klick(60, *DD_INST_0) + _klick(70, *_dd_eintrag(DD_INST_0, 10))  # Glocke fuer Spur 1
    ev += _taste(100, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 140, ev)
    assert "spur 2" in log and "note 1 2 71 1" in log, log
    assert "instrument 0 Glocke" in log, log
    _gesichert(log)
    doc = ScoreDoc.load_json(str(datei))
    assert [t.name for t in doc.tracks] == ["Stimme 1", "Stimme 2"]
    assert doc.tracks[0].instrument.name == "Glocke" and doc.tracks[0].instrument.env_decay_ms == 900
    assert [(x.start_beat, x.pitch) for x in doc.tracks[1].notes] == [(2.0, 71)]


def test_tracker_export_liefert_dasselbe_gitter_wie_der_qt_konverter(tmp_path):
    """Das Demo-Stueck hat alles, was der Konverter besonders behandelt: einen
    Akkord (-> hoechste Note), eine Staccato-Note (halbe Dauer), eine Note ueber
    die 64-Zeilen-Grenze (gekuerzt), NOTE_OFFs und zwei Spuren. Der Pilot muss
    Zelle fuer Zelle dasselbe schreiben wie `to_tracker_song`."""
    datei = tmp_path / "demo.json"
    shutil.copy(DEMO, datei)
    log = _editor(tmp_path, datei, 80, _taste(20, RL_T, RL_LCTRL))
    assert any(z.startswith("tracker ") for z in log), log
    ziel = tmp_path / "demo_tracker.json"
    assert ziel.exists()
    dh = Song.load_json(str(ziel))
    py, _warn = to_tracker_song(ScoreDoc.load_json(str(DEMO)))
    assert (dh.bpm, dh.channels, len(dh.patterns), dh.order) == (py.bpm, py.channels, len(py.patterns), py.order)
    assert len(py.patterns) == 2                                # die Grenze wird wirklich ueberschritten
    for a, b in zip(dh.patterns, py.patterns):
        assert a.rows == b.rows
        assert a.data == b.data
    assert dh.channel_inst == py.channel_inst
    assert [i.name for i in dh.instruments] == [i.name for i in py.instruments]


def test_leertaste_spielt_das_stueck(tmp_path):
    datei = tmp_path / "demo.json"
    shutil.copy(DEMO, datei)
    log = _editor(tmp_path, datei, 60, _taste(20, RL_SPACE))
    assert "geladen 2 18" in log, log
    assert any(z.startswith("spielt 17") for z in log), log     # 18 Ereignisse, eines ist eine Pause
