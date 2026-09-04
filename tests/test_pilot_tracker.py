"""Der Tracker-Pilot, bedient wie von Hand (`examples/190_tracker.dh`).

Der fuenfte Pilot und der erste mit einer Zeitachse: Noten werden nicht
gemalt, sondern GESPIELT -- auf einer Audio-Uhr, samplegenau. Geprueft wird
deshalb an drei Ergebnissen, die ein FREMDER Leser bestaetigt:

- die Datei liest `drachenhauch.tracker.Song` (das Modell der Qt-Fassung),
  und eine Datei der Qt-Fassung kommt hier mit allen Spalten an;
- die WAV liest Pythons `wave`-Modul;
- der GB-Code laeuft durch `dhrt --check` und STARTET.

Wie bei den anderen Piloten wird an der Logik nichts geaendert. Die Kopie
bekommt eine PRINT-Zeile je Bild, die nur BESTEHENDE Werte ausliest, wird
aus dem Bild geschoben (der echte Mauszeiger redet sonst mit) und bekommt,
wo ein Dateidialog stuende, den Pfad direkt gesetzt.
"""
import json
import os
import re
import subprocess
import wave
from pathlib import Path

import numpy as np
import pytest

_ROOT = Path(__file__).resolve().parent.parent
_PILOT = _ROOT / "examples" / "190_tracker.dh"


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()),
                None)


_DHRT = _find_dhrt()
pytestmark = [pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut"),
              pytest.mark.seriell]

KEY_UP, KEY_DOWN = 1, 2
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7
# raylib-Tastencodes (die Aufnahme spricht raylib, das Programm KEY_*).
T_Z, T_Q, T_C, T_V, T_I, T_S, T_Y = 90, 81, 67, 86, 73, 83, 89
T_0, T_SPACE, T_HOME = 48, 32, 268
T_RIGHT, T_LEFT, T_DOWN, T_UP = 262, 263, 264, 265
T_STRG, T_UMSCH = 341, 340

# Gitter-Geometrie wie im Piloten (Konstanten dort).
RH_W, CELL_W, HDR_H, ROW_H = 40, 118, 24, 20

_LEISTE = ("bNeu bAuf bSpei bSpeiAls spBpm spKan bPlayPat bPlaySong bStop bZur bVor bWav "
           "cbStereo cbAmiga bCode lblPat ddPat lblRows spRows bPatNeu bPatDup bPatDel bPatLeer "
           "lblOkt spOkt lblZelle spVol lblSlide spSlide lblFx ddFx spFxp ddZInst bOff").split()

_PROBE = ('    PRINT "P " + STR$(curC) + " " + STR$(curR) + " " + STR$(IIF(selAn, 1, 0)) + _\n'
          '          " " + STR$(uPos) + " " + STR$(uAnz) + " " + STR$(IIF(spielt, 1, 0)) + _\n'
          '          " " + STR$(kopfRow) + " " + STR$(patAkt) + " " + STR$(patAnz) + " " + STR$(orderAnz) + _\n'
          '          " " + STR$(GUI_CANVAS_X(gitter)) + " " + STR$(GUI_CANVAS_Y(gitter)) + _\n'
          '          " " + STR$(IIF(dirty, 1, 0)) + " " + STR$(note[patAkt, 0, 0]) + " " + STR$(note[patAkt, 0, 1]) + _\n'
          '          " " + STR$(note[patAkt, 0, 2]) + " " + STR$(note[patAkt, 1, 0]) + " " + STR$(note[patAkt, 1, 1]) + _\n'
          '          " " + STR$(vol[patAkt, 0, 0]) + " " + STR$(slide[patAkt, 0, 0]) + " " + STR$(fx[patAkt, 0, 0]) + _\n'
          '          " " + STR$(fxp[patAkt, 0, 0]) + " " + STR$(inst[patAkt, 0, 0]) + _\n'
          '          " " + STR$(instAnz) + " " + STR$(kanaele) + " " + STR$(bpm) + " " + STR$(bildNr)'
          + "".join(' + _\n          " " + STR$(GUI_GET_X(%s)) + " " + STR$(GUI_GET_Y(%s)) + '
                    '" " + STR$(GUI_GET_W(%s)) + " " + STR$(GUI_GET_H(%s))' % (w, w, w, w)
                    for w in _LEISTE)
          + "\n")
_FELDER = ("curC curR selAn uPos uAnz spielt kopfRow patAkt patAnz orderAnz gx gy dirty "
           "n00 n01 n02 n10 n11 v00 s00 f00 fp00 i00 instAnz kanaele bpm bild").split()
_FELDER += [w + a for w in _LEISTE for a in ("X", "Y", "W", "H")]


def _kopie(tmp_path, zusatz=""):
    src = _PILOT.read_text(encoding="utf-8")
    assert src.count("SETFPS(60)") == 1
    src = src.replace("SETFPS(60)", "SETFPS(60)\nSET_WINDOW_POS(-3000, -3000)", 1)
    assert src.count("    FLIP()\nWEND") == 1
    src = src.replace("    FLIP()\nWEND", _PROBE + "    FLIP()\nWEND")
    assert src.count("    bildNr = bildNr + 1\n") == 1
    src = src.replace("    bildNr = bildNr + 1\n", "    bildNr = bildNr + 1\n" + zusatz)
    ziel = tmp_path / "pilot.dh"
    ziel.write_text(src, encoding="utf-8")
    return ziel


def _events(tmp_path, events):
    events = sorted(events, key=lambda e: e[0])
    lines = ["# Test-Aufnahme", "c %d" % len(events)]
    for frame, typ, *params in events:
        p = (list(params) + [0, 0, 0, 0])[:4]
        lines.append("e %d %d %d %d %d %d // Event: test"
                     % (frame, typ, p[0], p[1], p[2], p[3]))
    (tmp_path / "ev.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _lauf(tmp_path, frames, events=None, zusatz=""):
    quelle = _kopie(tmp_path, zusatz)
    if events is not None:
        _events(tmp_path, events)
        text = quelle.read_text(encoding="utf-8")
        text = text.replace("SETFPS(60)", 'SETFPS(60)\nAUTOMATION_PLAY("ev.txt")', 1)
        quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=240,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    zeilen = [ln for ln in (r.stdout or "").splitlines() if ln.startswith("P ")]
    assert zeilen, (r.stdout, r.stderr)
    return [dict(zip(_FELDER, [int(v) for v in re.split(r"\s+", ln)[1:]])) for ln in zeilen]


# Eine eingespeiste Taste bleibt GEDRUECKT, bis ein KEY_UP kommt -- raylib
# aendert den Tastenzustand nur ueber Ereignisse. Ohne das Loslassen hing
# nach dem ersten Strg+C die Strg-Taste fuer den Rest des Laufs, und ein
# zweites Z gab keine Flanke mehr fuer KEYHIT.
def _taste(frame, code):
    return [(frame, KEY_DOWN, code), (frame + 1, KEY_UP, code)]


def _mit(modifier, frame, taste, dauer=4):
    ev = [(frame + i, KEY_DOWN, modifier) for i in range(dauer)]
    ev += [(frame + 1, KEY_DOWN, taste), (frame + 2, KEY_UP, taste),
           (frame + dauer, KEY_UP, modifier)]
    return ev


def _strg(frame, taste, dauer=4):
    return _mit(T_STRG, frame, taste, dauer)


def _umsch(frame, taste, dauer=4):
    return _mit(T_UMSCH, frame, taste, dauer)


def _klick(frame, x, y):
    return [(frame, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_BUTTON_DOWN, 0),
            (frame + 2, MOUSE_BUTTON_UP, 0)]


def _zelle(geo, c, r):
    """Bildschirm-Mitte der Zelle (c, r) -- die Zeichenflaeche kennt beides."""
    return (geo["gx"] + RH_W + c * CELL_W + CELL_W // 2,
            geo["gy"] + HDR_H + r * ROW_H + ROW_H // 2)


def _rect(geo, name):
    return geo[name + "X"], geo[name + "Y"], geo[name + "W"], geo[name + "H"]


def _ueberlappt(a, b):
    ax, ay, aw, ah = a
    bx, by, bw, bh = b
    return ax < bx + bw and bx < ax + aw and ay < by + bh and by < ay + ah


# ------------------------------------------------------------ Layout
def test_die_werkzeugleisten_ueberlappen_nicht_und_passen_ins_fenster(tmp_path):
    """Im ersten Bild lagen 'Stereo' und 'Amiga' uebereinander, und der
    letzte Knopf jeder Leiste war abgeschnitten -- gesehen nur im BILD.
    Geprueft werden die RECHTECKE: ein Treffertest saehe den Text nicht."""
    geo = _lauf(tmp_path, 3)[-1]
    rects = {w: _rect(geo, w) for w in _LEISTE}
    for a in _LEISTE:
        for b in _LEISTE:
            if a < b:
                assert not _ueberlappt(rects[a], rects[b]), (a, b, rects[a], rects[b])
    for w, (x, y, bw, bh) in rects.items():
        assert x + bw <= 1400 - 8, (w, x + bw)


# ------------------------------------------------------------ Noten setzen
def test_die_tastatur_setzt_noten_und_rueckt_weiter(tmp_path):
    geo = _lauf(tmp_path, 40, _taste(5, T_Z) + _taste(12, T_Q))[-1]
    assert geo["n00"] == 60, "Z = C der gewaehlten Oktave (4 -> MIDI 60)"
    assert geo["n01"] == 72, "Q = C eine Oktave hoeher"
    assert geo["curR"] == 2, "nach jeder Note eine Reihe weiter"
    assert geo["dirty"] == 1


def test_null_setzt_note_aus_und_entf_loescht(tmp_path):
    ev = _taste(5, T_Z) + _taste(12, T_0) + _taste(20, T_UP) + _taste(26, T_UP) + _taste(32, 261)
    geo = _lauf(tmp_path, 45, ev)[-1]
    assert geo["n01"] == -1, "OFF in Reihe 1"
    assert geo["n00"] == -2, "Reihe 0 wieder leer (Entf)"


def test_ein_klick_setzt_den_cursor(tmp_path):
    geo0 = _lauf(tmp_path, 3)[-1]
    x, y = _zelle(geo0, 1, 5)
    geo = _lauf(tmp_path, 30, _klick(5, x, y) + _taste(14, T_Z))[-1]
    assert (geo["n10"], geo["n11"]) == (-2, -2)
    assert geo["curC"] == 1 and geo["curR"] == 6, "Note in (1,5) gesetzt, Cursor eins tiefer"


# ------------------------------------------------------------ Verlauf
def test_am_anfang_gibt_es_nichts_zurueckzunehmen(tmp_path):
    geo = _lauf(tmp_path, 3)[-1]
    assert (geo["uPos"], geo["uAnz"]) == (1, 1)


def test_strg_z_nimmt_eine_note_zurueck_und_strg_y_holt_sie(tmp_path):
    ev = _taste(5, T_Z) + _taste(12, T_Q) + _strg(24, T_Z) + _strg(40, T_Y)
    geos = _lauf(tmp_path, 60, ev)
    # Die Bildnummer der Aufnahme und die des Programms laufen um ein paar
    # Bilder auseinander -- gefragt wird nach dem ZUSTAND, nicht der Nummer.
    zwischen = [g for g in geos if (g["n00"], g["n01"]) == (60, -2) and g["uAnz"] == 3]
    assert zwischen, "nach Strg+Z ist nur die letzte Note weg, der Vor-Weg bleibt"
    assert zwischen[0]["uPos"] == 2
    ende = geos[-1]
    assert (ende["n00"], ende["n01"]) == (60, 72)
    assert ende["uPos"] == 3


def test_ein_rueckgaengig_laesst_den_cursor_stehen(tmp_path):
    """Der Verlauf stellt den ganzen Song wieder her -- die Arbeitsstelle
    gehoert nicht dazu, sonst spraenge man bei jedem Strg+Z nach oben."""
    ev = _taste(5, T_Z) + _taste(12, T_Q) + _taste(19, T_RIGHT) + _strg(28, 90)
    geo = _lauf(tmp_path, 45, ev)[-1]
    assert (geo["curC"], geo["curR"]) == (1, 2)


# ------------------------------------------------------------ Bloecke
def test_block_kopieren_und_einfuegen(tmp_path):
    ev = (_taste(5, T_Z) + _taste(12, T_Q)                # Reihen 0 und 1
          + _taste(20, T_HOME)                            # Cursor (0,0)
          + _umsch(26, T_DOWN)                            # Auswahl 0..1
          + _strg(34, T_C)
          + _taste(42, T_RIGHT) + _taste(48, T_HOME)      # Cursor (1,0)
          + _strg(56, T_V))
    geos = _lauf(tmp_path, 75, ev)
    assert any(g["selAn"] == 1 and g["curR"] == 1 for g in geos), "Umschalt+Pfeil waehlt einen Block"
    ende = geos[-1]
    assert (ende["n10"], ende["n11"]) == (60, 72), "der Block steht auch in Kanal 2"
    assert (ende["n00"], ende["n01"]) == (60, 72), "und ist im Original geblieben"


def test_transponieren_laesst_das_schlagzeug_in_ruhe(tmp_path):
    zusatz = ('    IF bildNr = 3 THEN\n'
              '        zelleSetzen(0, 0, 0, 60) : zelleSetzen(0, 3, 0, 60)\n'
              '        PRINT "D " + STR$(note[0, 3, 0])\n'
              '    END IF\n'
              '    IF bildNr = 40 THEN PRINT "D " + STR$(note[0, 3, 0])\n')
    # Block ueber alle vier Kanaele in Reihe 0, dann Strg+Umschalt+Auf.
    ev = (_taste(6, T_HOME) + _umsch(12, T_RIGHT) + _umsch(18, T_RIGHT) + _umsch(24, T_RIGHT)
          + [(30 + i, KEY_DOWN, T_STRG) for i in range(4)]
          + [(30 + i, KEY_DOWN, T_UMSCH) for i in range(4)]
          + [(31, KEY_DOWN, T_UP), (32, KEY_UP, T_UP), (34, KEY_UP, T_STRG), (34, KEY_UP, T_UMSCH)])
    quelle = _kopie(tmp_path, zusatz)
    _events(tmp_path, ev)
    text = quelle.read_text(encoding="utf-8").replace("SETFPS(60)", 'SETFPS(60)\nAUTOMATION_PLAY("ev.txt")', 1)
    quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=240,
                       env=dict(os.environ, DHRT_FRAMES="45"), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    p = [ln for ln in r.stdout.splitlines() if ln.startswith("P ")]
    d = [ln for ln in r.stdout.splitlines() if ln.startswith("D ")]
    ende = dict(zip(_FELDER, [int(v) for v in re.split(r"\s+", p[-1])[1:]]))
    assert ende["n00"] == 72, "Ton-Kanal eine Oktave hoeher"
    assert d == ["D 60", "D 60"], "der Schlag im Drum-Kanal bleibt, was er war"


# ------------------------------------------------------------ Datei
def test_die_datei_liest_die_qt_fassung(tmp_path):
    """Der FREMDE Leser: `Song.load_json` aus dem Qt-Tracker."""
    from drachenhauch.tracker import Song
    zusatz = '    IF bildNr = 2 THEN dateiPfad = "song.json"\n'
    ev = _taste(5, T_Z) + _taste(12, T_Q) + _strg(24, T_S)
    _lauf(tmp_path, 40, ev, zusatz)
    song = Song.load_json(str(tmp_path / "song.json"))
    assert song.channels == 4 and song.bpm == 120
    assert song.patterns[0].data[0][:3] == [60, 72, None]
    assert song.patterns[0].data[1][0] is None
    assert len(song.instruments) == 18
    assert [i.name for i in song.instruments[:2]] == ["Fluegel (Piano)", "E-Piano"]
    assert song.channel_inst == [0, 4, 7, 14]
    assert song.instruments[0].env_decay_ms == 700 and song.instruments[0].waveform == "triangle"


def test_eine_datei_der_qt_fassung_kommt_mit_allen_spalten_an(tmp_path):
    from drachenhauch.tracker import Song, FX_ARP
    s = Song(channels=6)
    s.bpm = 140
    p = s.patterns[0]
    p.set(0, 0, 64)
    p.set_vol(0, 0, 9)
    p.set_slide(0, 0, -3)
    p.set_fx(0, 0, FX_ARP, 0x47)
    from drachenhauch.tracker.presets import factory_instruments
    s.instruments = factory_instruments()
    s.channel_inst[0] = 2
    p.set_inst(0, 0, 5)
    s.add_pattern("Refrain", rows=8)
    s.order = [0, 1, 0]
    s.save_json(str(tmp_path / "qt.json"))
    zusatz = '    IF bildNr = 3 THEN ladeDatei("qt.json")\n'
    geo = _lauf(tmp_path, 8, zusatz=zusatz)[-1]
    assert (geo["kanaele"], geo["bpm"], geo["patAnz"], geo["orderAnz"]) == (6, 140, 2, 3)
    assert (geo["n00"], geo["v00"], geo["s00"], geo["f00"], geo["fp00"], geo["i00"]) == (64, 9, -3, 1, 0x47, 5)
    assert geo["instAnz"] == 18


def test_der_rundweg_erhaelt_effekte_und_reihenfolge(tmp_path):
    """Qt schreibt, der Pilot liest UND schreibt, Qt liest wieder: nichts
    darf unterwegs verloren gehen."""
    from drachenhauch.tracker import Song, FX_VIB
    s = Song()
    s.patterns[0].set(1, 3, 67)
    s.patterns[0].set_vol(1, 3, 12)
    s.patterns[0].set_fx(1, 3, FX_VIB, 0x52)
    s.patterns[0].set(3, 0, 36)
    s.channel_vol[2] = 0.5
    s.add_pattern()
    s.order = [1, 0]
    s.save_json(str(tmp_path / "qt.json"))
    zusatz = ('    IF bildNr = 3 THEN ladeDatei("qt.json")\n'
              '    IF bildNr = 5 THEN dateiPfad = "zurueck.json" : speichern(FALSE)\n')
    _lauf(tmp_path, 8, zusatz=zusatz)
    z = Song.load_json(str(tmp_path / "zurueck.json"))
    assert z.order == [1, 0] and len(z.patterns) == 2
    assert z.patterns[0].data[1][3] == 67 and z.patterns[0].vol[1][3] == 12
    assert z.patterns[0].get_fx(1, 3) == (FX_VIB, 0x52)
    assert z.patterns[0].data[3][0] == 36
    assert z.channel_vol[2] == 0.5


# ------------------------------------------------------------ Ausgabe
def _wav(pfad):
    with wave.open(str(pfad)) as w:
        n, ch, sr = w.getnframes(), w.getnchannels(), w.getframerate()
        roh = w.readframes(n)
    return np.frombuffer(roh, "<i2").astype(np.float32).reshape(-1, ch) / 32767.0, sr


def test_die_wav_hat_die_laenge_des_songs_und_die_noten_darin(tmp_path):
    zusatz = ('    IF bildNr = 3 THEN\n'
              '        zelleSetzen(0, 0, 0, 60) : zelleSetzen(0, 0, 8, 67)\n'
              '        wavRendern("out.wav")\n'
              '    END IF\n')
    _lauf(tmp_path, 6, zusatz=zusatz)
    a, sr = _wav(tmp_path / "out.wav")
    # 16 Reihen x 125 ms + 800 ms Nachlauf
    assert abs(a.shape[0] / sr - 2.8) < 0.02, a.shape[0] / sr
    assert a.shape[1] == 1, "ohne Stereo-Haken mono"
    assert abs(np.abs(a).max() - 1.0) < 0.02, "normalisiert"
    f = int(sr * 0.05)
    h = np.abs(a[:, 0][: (a.shape[0] // f) * f]).reshape(-1, f).max(axis=1)
    assert h[0] > 0.3 and h[20] > 0.3, "Klang bei 0 ms und bei 1000 ms (Reihe 8)"
    assert h[-2] < 0.05, "am Ende Stille"


def test_stereo_und_amiga_pan_legen_den_ersten_kanal_nach_links(tmp_path):
    zusatz = ('    IF bildNr = 3 THEN\n'
              '        zelleSetzen(0, 0, 0, 60)\n'
              '        renderStereo = TRUE : renderAmiga = TRUE\n'
              '        wavRendern("out.wav")\n'
              '    END IF\n')
    _lauf(tmp_path, 6, zusatz=zusatz)
    a, _sr = _wav(tmp_path / "out.wav")
    assert a.shape[1] == 2
    assert np.abs(a[:, 0]).max() > 3 * np.abs(a[:, 1]).max()


def test_der_gb_code_laeuft_durch_check_und_startet(tmp_path):
    zusatz = ('    IF bildNr = 3 THEN\n'
              '        zelleSetzen(0, 0, 0, 60) : vol[0, 0, 0] = 9 : slide[0, 0, 0] = 2\n'
              '        zelleSetzen(0, 3, 4, 60)\n'
              '        gbCodeSichern("song_gb.dh")\n'
              '    END IF\n')
    _lauf(tmp_path, 6, zusatz=zusatz)
    code = (tmp_path / "song_gb.dh").read_text(encoding="utf-8")
    assert "trkV0[0] = 60" in code, "Lautstaerke 9/15 -> 60 %"
    assert "trkSl0[0] =" in code and "AUDIO_SFX(" in code
    assert "trk3[4] = 1" in code, "der Drum-Kanal schreibt einen Schlag, keine Frequenz"
    chk = subprocess.run([str(_DHRT), "--check", "song_gb.dh"], capture_output=True, text=True,
                         encoding="utf-8", errors="replace", timeout=120, cwd=str(tmp_path))
    assert chk.returncode == 0 and json.loads(chk.stdout.strip() or "[]") == [], chk.stdout
    # Uebersetzen allein bewiese nichts -- er muss STARTEN.
    lauf = (tmp_path / "lauf.dh")
    lauf.write_text(code + "\nDIM t AS INTEGER\nFOR t = 1 TO 3\n    TRACKER_UPDATE(130.0)\nNEXT\n"
                    "PRINT trkRow\n", encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", "lauf.dh"], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120, cwd=str(tmp_path),
                       env=dict(os.environ, DHRT_FRAMES="2"))
    assert r.returncode == 0, r.stderr
    assert r.stdout.strip().endswith("3")


# ------------------------------------------------------------ Wiedergabe
def test_die_leertaste_startet_die_uhr_und_stoppt_sie_wieder(tmp_path):
    geos = _lauf(tmp_path, 150, _taste(5, T_SPACE) + _taste(120, T_SPACE))
    laeuft = [g for g in geos if 10 < g["bild"] < 110]
    assert all(g["spielt"] == 1 for g in laeuft)
    # 120 BPM = eine Reihe je 125 ms = alle 7,5 Bilder; nach 100 Bildern
    # steht der Kopf bei Reihe 12 oder 13. Weniger als 8 hiesse, die Uhr
    # laeuft nicht in echter Zeit.
    assert max(g["kopfRow"] for g in laeuft) >= 8, "der Wiedergabekopf wandert in echter Zeit"
    assert geos[-1]["spielt"] == 0 and geos[-1]["kopfRow"] == -1


def test_ein_geplantes_pattern_zeigt_sich_im_song_modus(tmp_path):
    """Song-Modus mit zwei Patterns: die Anzeige folgt dem Kopf."""
    zusatz = ('    IF bildNr = 3 THEN\n'
              '        patternNeu(FALSE) : reihenSetzen(1)\n'
              '        order[1] = 1 : orderAnz = 2\n'
              '        patternWaehlen(0) : reihenSetzen(1)\n'
              '        starten(TRUE)\n'
              '    END IF\n')
    geos = _lauf(tmp_path, 120, zusatz=zusatz)
    assert {g["patAkt"] for g in geos if g["bild"] > 10} >= {0, 1}
