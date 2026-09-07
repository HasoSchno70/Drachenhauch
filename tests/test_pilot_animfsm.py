"""Der Anim-FSM-Editor in Drachenhauch (`examples/198_anim_fsm_editor.dh`, Weg B).

Der Graph wird mit echten Klicks ueber die Eingabe-Wiedergabe bedient
(Doppelklick legt einen Zustand an, die rechte Maustaste zieht einen Uebergang,
ein Klick auf den Pfeil waehlt ihn), gesichert wird mit Strg+S. Die Datei
lesen ZWEI fremde Leser: `drachenhauch.animeditor.document.AnimDoc` (das
Modell des Qt-Editors) und die LAUFZEIT selbst -- `ANIM_FSM_LOAD` plus ein
Schritt der Maschine. Dass die JSON gueltig ist, waere die schwaechere
Aussage; ein Uebergang zu einem geloeschten Zustand laedt nicht.

Braucht ein Fenster und speist Eingaben ein -- `_BRAUCHT_GRAFIK` + `_SERIELL`.
Die Testkopie laeuft OHNE `WINDOW_MAXIMIZE`, damit die Lagen fest sind
(1280 x 800: Graph-Ursprung 290/20, Inspektor ab x = 940).
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import pytest

from drachenhauch.animeditor.document import AnimDoc

_ROOT = Path(__file__).resolve().parent.parent
EDITOR = _ROOT / "examples" / "198_anim_fsm_editor.dh"
DEMO = _ROOT / "examples" / "anim_demo.dhanim"
SHEET = _ROOT / "examples" / "assets" / "hero_walk.png"


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
RL_F5, RL_LCTRL, RL_S, RL_Z, RL_DELETE = 294, 341, 83, 90, 261
LINKS, RECHTS = 0, 1

# Lagen aus dem Quelltext des Editors bei 1280 x 800 (siehe Kopf):
GX0, GY0 = 270 + 20, 20                    # Graph-Ursprung auf dem Schirm
NODE_W, NODE_H = 150, 50
INSP_X = 1280 - 340
# anim_demo: idle (120,176), run (432,176), jump (272,40), fall (272,336)
IDLE = (GX0 + 120 + NODE_W // 2, GY0 + 176 + NODE_H // 2)
RUN = (GX0 + 432 + NODE_W // 2, GY0 + 176 + NODE_H // 2)
FALL = (GX0 + 272 + NODE_W // 2, GY0 + 336 + NODE_H // 2)
# idle -> run hat einen Rueckweg: die Linie liegt 9 px unter der Mitte
KANTE_IDLE_RUN = ((IDLE[0] + RUN[0]) // 2, IDLE[1] + 9)
BTN_BED_PLUS = (INSP_X + 8 + 65, 98 + 6 * 32 + 6 + 13)
BTN_PARAM_PLUS = (8 + 20, 26 + 310 + 12)     # Menueleiste 26 px + Inhalts-y


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
    log = tmp_path / "anim.log"
    ev = sorted(events, key=lambda e: e[0])
    zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
    for frame, typ, *params in ev:
        p = (list(params) + [0, 0, 0, 0])[:4]
        zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
    (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
    text = EDITOR.read_text(encoding="utf-8")
    text = text.replace('SETFPS(60)\n', 'SETFPS(60)\nAUTOMATION_PLAY("' + (tmp_path / "ev.txt").as_posix() + '")\n', 1)
    text = text.replace("WINDOW_MAXIMIZE()\n", "", 1)      # feste Lagen fuer die Klicks
    (tmp_path / "_ad").mkdir(exist_ok=True)
    quelle = tmp_path / "_ad" / "editor_test.dh"
    quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle), "--", str(datei)], capture_output=True,
                       text=True, encoding="utf-8", errors="replace", timeout=180,
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DH_ANIM_LOG=str(log)),
                       cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    return log.read_text(encoding="utf-8").splitlines() if log.exists() else []


def _demo(tmp_path):
    ziel = tmp_path / "held.dhanim"
    d = json.loads(DEMO.read_text(encoding="utf-8"))
    d["sheet"] = SHEET.as_posix()
    ziel.write_text(json.dumps(d, indent=2), encoding="utf-8")
    return ziel


def _laufzeit(tmp_path, datei, speed=None):
    """Der Leser der LAUFZEIT: laden, aufsetzen, optional `speed` setzen und
    einen Schritt rechnen; liefert den Zustandsnamen."""
    setz = f'ANIM_FSM_SET_FLOAT(fsm, "speed", {speed})\nANIM_FSM_UPDATE(fsm, sp, 16)\n' if speed is not None else ""
    src = ('IMPORT "animfsm"\nIMPORT "sprite"\nDIM sp AS SPRITE\nsp = SPRITE_NEW(0, 16, 16)\n'
           'DIM fsm AS ANIM_FSM\nfsm = ANIM_FSM_LOAD("' + datei.name + '")\nANIM_FSM_SETUP(fsm, sp)\n'
           + setz + 'PRINT ANIM_FSM_STATE(fsm)\n')
    f = tmp_path / "leser.dh"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120, cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return r.stdout.strip().splitlines()[-1]


def test_editor_uebersetzt_ohne_befund():
    r = subprocess.run([str(_DHRT), "--check", str(EDITOR)], capture_output=True, text=True,
                       encoding="utf-8", timeout=120)
    assert r.returncode == 0 and r.stdout.strip() == "[]", r.stdout


def test_doppelklick_legt_einen_zustand_an_und_beide_leser_lesen_ihn(tmp_path):
    datei = tmp_path / "neu.dhanim"
    ev = _klick(20, 700, 500) + _klick(26, 700, 500)      # Doppelklick auf freie Flaeche
    ev += _taste(60, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 100, ev)
    assert "neu-datei " + str(datei) in log, log
    assert "zustand state 336 456" in log, log             # (700-290-75, 500-20-25) aufs Raster
    assert any(z.startswith("gesichert ") for z in log), log
    doc = AnimDoc.load(str(datei))
    assert [(s.name, s.x, s.y, s.first, s.last) for s in doc.states] == [("state", 336, 456, 0, 0)]
    assert doc.effective_default() == "state"
    assert _laufzeit(tmp_path, datei) == "state"


def test_rechte_maustaste_zieht_einen_uebergang(tmp_path):
    datei = _demo(tmp_path)
    ev = _zug(20, IDLE[0], IDLE[1], FALL[0], FALL[1], knopf=RECHTS)
    ev += _taste(60, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 100, ev)
    assert "geladen 4 5" in log, log
    assert "verbunden idle fall" in log, log
    assert any(z.startswith("gesichert ") for z in log), log
    doc = AnimDoc.load(str(datei))
    assert len(doc.transitions) == 6
    t = doc.transitions[-1]
    assert (t.from_state, t.to_state, t.conditions, t.wait_finished) == ("idle", "fall", [], False)
    # Ohne Bedingung feuert der neue Uebergang sofort: nach einem Schritt steht die Maschine in fall.
    assert _laufzeit(tmp_path, datei, speed=0.0) == "fall"


def test_ziehen_und_rueckgaengig(tmp_path):
    datei = _demo(tmp_path)
    ev = _zug(20, IDLE[0], IDLE[1], IDLE[0] + 80, IDLE[1])
    ev += _taste(60, RL_Z, RL_LCTRL) + _taste(80, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 120, ev)
    assert "bewegt idle 200 176" in log, log
    assert "rueckgaengig 4 5" in log, log
    assert any(z.startswith("gesichert ") for z in log), log
    doc = AnimDoc.load(str(datei))
    assert (doc.state_by_name("idle").x, doc.state_by_name("idle").y) == (120, 176)


def test_entf_loescht_den_zustand_samt_seinen_uebergaengen(tmp_path):
    """fall haengt an zwei Uebergaengen (jump -> fall, fall -> idle). Bleibt einer
    stehen, lehnt ANIM_FSM_LOAD die Datei ab -- der Laufzeit-Leser ist hier
    die eigentliche Pruefung."""
    datei = _demo(tmp_path)
    ev = _klick(20, FALL[0], FALL[1]) + _taste(40, RL_DELETE) + _taste(60, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 100, ev)
    assert "gewaehlt zustand fall" in log, log
    assert "geloescht zustand fall 3 3" in log, log
    assert any(z.startswith("gesichert ") for z in log), log
    doc = AnimDoc.load(str(datei))
    assert sorted(doc.state_names()) == ["idle", "jump", "run"]
    assert all("fall" not in (t.from_state, t.to_state) for t in doc.transitions)
    assert _laufzeit(tmp_path, datei) == "idle"


def test_bedingung_am_pfeil_hinzufuegen(tmp_path):
    """Klick auf den Pfeil idle -> run waehlt ihn, [+ Bedingung] haengt eine an
    (erster Parameter `speed`, `>` 0). Die Maschine wechselt danach bei speed
    10 weiterhin nach run -- beide Bedingungen gelten."""
    datei = _demo(tmp_path)
    ev = _klick(20, *KANTE_IDLE_RUN) + _klick(40, *BTN_BED_PLUS) + _taste(70, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 110, ev)
    assert "gewaehlt uebergang 0" in log, log
    assert "bedingung 2" in log, log
    assert any(z.startswith("gesichert ") for z in log), log
    doc = AnimDoc.load(str(datei))
    t = doc.transitions[0]
    assert (t.from_state, t.to_state) == ("idle", "run")
    assert [(c.param, c.op, c.value) for c in t.conditions] == [("speed", "gt", 5.0), ("speed", "gt", 0.0)]
    assert _laufzeit(tmp_path, datei, speed=10.0) == "run"
    assert _laufzeit(tmp_path, datei, speed=1.0) == "idle"


def test_parameter_anlegen(tmp_path):
    datei = _demo(tmp_path)
    ev = _klick(20, *BTN_PARAM_PLUS) + _taste(50, RL_S, RL_LCTRL)
    log = _editor(tmp_path, datei, 90, ev)
    assert "parameter param" in log, log
    assert any(z.startswith("gesichert ") for z in log), log
    doc = AnimDoc.load(str(datei))
    assert [(p.name, p.ptype, p.default) for p in doc.params][-1] == ("param", "float", 0.0)
    assert _laufzeit(tmp_path, datei) == "idle"


def test_f5_schreibt_die_vorschau_und_sie_laeuft(tmp_path):
    datei = _demo(tmp_path)
    log = _editor(tmp_path, datei, 90, _taste(20, RL_F5))
    assert any(z.startswith("gestartet ") for z in log), log
    lauf = tmp_path / "held_vorschau.dh"
    assert lauf.exists()
    text = lauf.read_text(encoding="utf-8")
    assert 'ANIM_FSM_LOAD("held.dhanim")' in text and "UI_SLIDER" in text and "UI_BUTTON" in text
    assert SHEET.name in text                                  # das Blatt wurde gefunden
    r = subprocess.run([str(_DHRT), "--check", str(lauf)], capture_output=True, text=True,
                       encoding="utf-8", timeout=120)
    assert r.stdout.strip() == "[]", r.stdout
    # Nicht nur uebersetzen: ein paar Bilder laufen lassen -- ob das Blatt geladen
    # und die Maschine aufgesetzt wird, zeigt erst der Lauf.
    r = subprocess.run([str(_DHRT), "run", str(lauf)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120,
                       env=dict(os.environ, DHRT_FRAMES="5"), cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
