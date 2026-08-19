import pathlib
"""Headless-Tests fuer das Datenmodell + Code-Gen des `dhanim`-Editors.

Modell-Operationen (add/rename/remove State, Transitions, Params), JSON-
Roundtrip, und -- als Closed-Loop -- dass der Editor-Output vom Runtime-Modul
`animfsm` geladen werden kann + der Vorschau-Runner sauber parst.
"""
import json

import pytest

from drachenhauch.animeditor import (
    ANY_STATE, AnimDoc, Condition, History, Param, State, Transition,
    snap, unique_name,
)


# ----------------------------------------------------------------- Modell-Ops
def test_add_state_unique_names_and_default():
    doc = AnimDoc()
    a = doc.add_state(10, 10)
    b = doc.add_state(50, 10)
    assert a.name == "state"
    assert b.name == "state2"
    assert doc.default_state == "state"        # erster State wird Default


def test_rename_state_updates_transitions_and_default():
    doc = AnimDoc()
    doc.add_state(0, 0, "idle")
    doc.add_state(100, 0, "run")
    doc.add_transition("idle", "run")
    assert doc.rename_state("idle", "stand")
    assert doc.default_state == "stand"
    assert doc.transitions[0].from_state == "stand"
    # anim folgte dem Namen -> mitgezogen
    assert doc.state_by_name("stand").anim == "stand"


def test_rename_rejects_duplicate_and_star():
    doc = AnimDoc()
    doc.add_state(0, 0, "idle")
    doc.add_state(0, 0, "run")
    assert not doc.rename_state("idle", "run")     # Kollision
    assert not doc.rename_state("idle", ANY_STATE)  # "*" verboten


def test_remove_state_cleans_transitions():
    doc = AnimDoc()
    doc.add_state(0, 0, "idle")
    doc.add_state(0, 0, "run")
    doc.add_transition("idle", "run")
    doc.add_transition("run", "idle")
    doc.remove_state("run")
    assert doc.transitions == []
    assert doc.default_state == "idle"


def test_add_transition_validation():
    doc = AnimDoc()
    doc.add_state(0, 0, "idle")
    doc.add_state(0, 0, "run")
    assert doc.add_transition("idle", "run") is not None
    assert doc.add_transition("idle", "idle") is None      # Selbst-Loop
    assert doc.add_transition("idle", "ghost") is None     # Ziel unbekannt
    assert doc.add_transition("ghost", "run") is None      # Quelle unbekannt
    # Any-State als Quelle ist erlaubt
    assert doc.add_transition(ANY_STATE, "run") is not None


def test_add_remove_param_cleans_conditions():
    doc = AnimDoc()
    doc.add_state(0, 0, "idle")
    doc.add_state(0, 0, "run")
    doc.add_param("speed", "float")
    t = doc.add_transition("idle", "run")
    t.conditions.append(Condition("speed", "gt", 5.0))
    doc.remove_param("speed")
    assert doc.params == []
    assert t.conditions == []


def test_param_unique_names():
    doc = AnimDoc()
    doc.add_param("p")
    p2 = doc.add_param("p")
    assert p2.name == "p2"


def test_rename_param_updates_conditions():
    doc = AnimDoc()
    doc.add_state(0, 0, "idle")
    doc.add_state(0, 0, "run")
    doc.add_param("speed", "float")
    t = doc.add_transition("idle", "run")
    t.conditions.append(Condition("speed", "gt", 5.0))
    assert doc.rename_param("speed", "velocity")
    assert doc.param_by_name("velocity") is not None
    assert t.conditions[0].param == "velocity"
    # Kollision wird abgelehnt
    doc.add_param("other")
    assert not doc.rename_param("velocity", "other")


# ----------------------------------------------------------------- Helper
def test_snap():
    assert snap(0) == 0
    assert snap(4) == 8
    assert snap(11) == 8
    assert snap(12) == 16


def test_unique_name():
    assert unique_name("x", set()) == "x"
    assert unique_name("x", {"x"}) == "x2"
    assert unique_name("x", {"x", "x2"}) == "x3"


# ----------------------------------------------------------------- Serialisierung
def _sample_doc() -> AnimDoc:
    doc = AnimDoc(sheet="assets/hero_walk.png", frame_w=16, frame_h=16, scale=5.0)
    doc.add_state(80, 120, "idle")
    doc.add_state(320, 120, "run")
    j = doc.add_state(200, 30, "jump")
    j.loop = False
    j.first, j.last, j.fps = 0, 3, 14.0
    doc.add_param("speed", "float")
    doc.add_param("jump", "trigger")
    t1 = doc.add_transition("idle", "run")
    t1.conditions.append(Condition("speed", "gt", 5.0))
    t2 = doc.add_transition("run", "idle")
    t2.conditions.append(Condition("speed", "lt", 5.0))
    t3 = doc.add_transition(ANY_STATE, "jump")
    t3.conditions.append(Condition("jump", "trigger"))
    t4 = doc.add_transition("jump", "idle")
    t4.wait_finished = True
    return doc


def test_roundtrip_dict():
    doc = _sample_doc()
    doc2 = AnimDoc.from_dict(doc.to_dict())
    assert doc2.sheet == "assets/hero_walk.png"
    assert doc2.scale == 5.0
    assert [s.name for s in doc2.states] == ["idle", "run", "jump"]
    assert doc2.state_by_name("jump").loop is False
    assert doc2.state_by_name("jump").last == 3
    assert {p.name for p in doc2.params} == {"speed", "jump"}
    assert doc2.transitions[0].conditions[0].op == "gt"
    assert doc2.transitions[3].wait_finished is True


def test_save_load_file(tmp_path):
    doc = _sample_doc()
    p = tmp_path / "hero.dhanim"
    doc.save(str(p))
    doc2 = AnimDoc.load(str(p))
    assert doc2.effective_default() == "idle"
    assert len(doc2.transitions) == 4


def test_to_dict_runtime_shape():
    """Das `.dhanim` muss die Runtime-Pflichtfelder enthalten."""
    d = _sample_doc().to_dict()
    assert d["default"] == "idle"
    assert {s["name"] for s in d["states"]} == {"idle", "run", "jump"}
    # Trigger-Parameter hat keinen default-Key
    jp = next(p for p in d["params"] if p["name"] == "jump")
    assert "default" not in jp
    # Bedingungs-Operator + Schwelle
    assert d["transitions"][0]["conditions"][0]["value"] == 5.0


# ----------------------------------------------------------------- Beispiel-Datei
import pathlib

_DEMO = pathlib.Path(__file__).resolve().parents[1] / "examples" / "anim_demo.dhanim"


def test_shipped_demo_loads_in_editor():
    doc = AnimDoc.load(str(_DEMO))
    assert [s.name for s in doc.states] == ["idle", "run", "jump", "fall"]
    assert doc.effective_default() == "idle"
    assert {p.name for p in doc.params} == {"speed", "grounded", "jump"}
    # alle Transitions zeigen auf existierende States
    names = doc.state_names() | {ANY_STATE}
    for t in doc.transitions:
        assert t.from_state in names and t.to_state in doc.state_names()


def test_shipped_demo_runtime_valid(run_gb, tmp_path):
    """Die Demo muss vom animfsm-Runtime ladbar sein (kein Validierungsfehler)."""
    import shutil
    shutil.copy(str(_DEMO), str(tmp_path / "anim_demo.dhanim"))
    src = (
        'IMPORT "animfsm"\n'
        'IMPORT "sprite"\n'
        'DIM sp AS SPRITE\n'
        'sp = SPRITE_NEW(0, 16, 16)\n'
        'DIM fsm AS ANIM_FSM\n'
        'fsm = ANIM_FSM_LOAD("anim_demo.dhanim")\n'
        'ANIM_FSM_SETUP(fsm, sp)\n'
        'PRINT ANIM_FSM_STATE(fsm)\n'
    )
    assert run_gb(src, base=tmp_path).strip() == "idle"


# ----------------------------------------------------------------- Closed-Loop
def test_editor_output_loads_in_runtime(run_gb, tmp_path):
    """Editor-Output -> `.dhanim` -> ANIM_FSM_LOAD: identischer Default-State."""
    doc = _sample_doc()
    doc.sheet = ""   # kein echtes Bild im Test -> Sprite mit Dummy-Handle
    doc.save(str(tmp_path / "hero.dhanim"))
    src = (
        'IMPORT "animfsm"\n'
        'IMPORT "sprite"\n'
        'DIM sp AS SPRITE\n'
        'sp = SPRITE_NEW(0, 16, 16)\n'
        'DIM fsm AS ANIM_FSM\n'
        'fsm = ANIM_FSM_LOAD("hero.dhanim")\n'
        'ANIM_FSM_SETUP(fsm, sp)\n'
        'PRINT ANIM_FSM_STATE(fsm)\n'
        'ANIM_FSM_SET_FLOAT(fsm, "speed", 9.0)\n'
        'ANIM_FSM_UPDATE(fsm, sp, 16)\n'
        'PRINT ANIM_FSM_STATE(fsm)\n'
    )
    assert run_gb(src, base=tmp_path).strip().split("\n") == ["idle", "run"]


# ----------------------------------------------------------------- Codegen
def _parses(src: str):
    """Prueft, dass ERZEUGTER .dh-Code gueltig ist -- ueber `dhrt --check`.

    Frueher lief das durch den Python-Parser. `dhrt --check` ist der bessere
    Pruefer: er nimmt den Compiler mit und meldet damit auch unbekannte
    Builtins und falsche Argumentzahlen, nicht nur Syntax. (Siehe
    docs/entwurf-python-parser-entfernen.md, Abschnitt 3 A.)
    """
    import json
    import os
    import subprocess
    import tempfile
    import sys
    sys.path.insert(0, str(pathlib.Path(__file__).parent))
    from conftest import _DHRT
    if _DHRT is None:
        import pytest
        pytest.skip("native Runtime 'dhrt' nicht gebaut")
    fd, tmp = tempfile.mkstemp(suffix=".dh")
    os.close(fd)
    try:
        pathlib.Path(tmp).write_text(src, encoding="utf-8")
        r = subprocess.run([str(_DHRT), "--check", tmp],
                           capture_output=True, text=True, encoding="utf-8",
                           timeout=60)
        roh = (r.stdout or "").strip()
        probleme = json.loads(roh) if roh else []
        fehler = [p for p in probleme if p.get("severity", "error") == "error"]
        assert not fehler, "erzeugter Code ist nicht gueltig: " + json.dumps(
            fehler, ensure_ascii=False)
    finally:
        os.unlink(tmp)
    return True

def test_generated_runner_parses():
    doc = _sample_doc()
    src = doc.generate_runner("hero.dhanim", title="Hero")
    assert 'ANIM_FSM_LOAD("hero.dhanim")' in src
    assert "UI_SLIDER" in src          # float-Parameter speed
    assert "UI_BUTTON" in src          # trigger-Parameter jump
    assert "UI_END_FRAME()" in src
    assert _parses(src) is not None


def test_generated_runner_no_sheet_uses_placeholder():
    doc = AnimDoc(sheet="")
    doc.add_state(0, 0, "idle")
    src = doc.generate_runner("x.dhanim")
    assert "SPRITE_NEW(0," in src      # Dummy-Handle
    assert "SPRITE_DRAW" not in src    # stattdessen Platzhalter-BOX
    assert _parses(src) is not None


def test_generated_runner_with_bool_param_parses():
    doc = AnimDoc()
    doc.add_state(0, 0, "idle")
    doc.add_state(0, 0, "fall")
    doc.add_param("grounded", "bool")
    t = doc.add_transition("idle", "fall")
    t.conditions.append(Condition("grounded", "is_false"))
    src = doc.generate_runner("x.dhanim")
    assert "UI_CHECKBOX" in src
    assert _parses(src) is not None


# ----------------------------------------------------------------- History
def test_history_undo_redo():
    h = History()
    s0 = {"v": 0}
    h.push(s0)
    assert h.can_undo and not h.can_redo
    cur = {"v": 1}
    restored = h.undo(cur)
    assert restored == s0
    assert not h.can_undo and h.can_redo
    again = h.redo(restored)
    assert again == cur
