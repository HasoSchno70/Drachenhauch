"""Golden-Tests fuer das native `animfsm`-Modul (Animations-State-Machine).

Konsolen-testbar: SPRITE_NEW(0, ...) braucht keine Grafik (der Texturindex 0
wird erst bei SPRITE_DRAW gebraucht), SPRITE_UPDATE/ANIM_FSM_* sind reine Logik.
Wir legen eine `.gbanim`-JSON in `tmp_path` und treiben die FSM ueber Parameter.
"""
import json
import textwrap

import pytest


def _hero_fsm():
    """Eine kleine idle/run/jump-FSM (Unity-Mecanim-Stil)."""
    return {
        "version": 1,
        "default": "idle",
        "params": [
            {"name": "speed", "type": "float", "default": 0.0},
            {"name": "jump", "type": "trigger"},
        ],
        "states": [
            {"name": "idle", "anim": "idle", "loop": True, "first": 0, "last": 3, "fps": 6.0},
            {"name": "run", "anim": "run", "loop": True, "first": 4, "last": 9, "fps": 12.0},
            {"name": "jump", "anim": "jump", "loop": False, "first": 10, "last": 12, "fps": 30.0},
        ],
        "transitions": [
            {"from": "idle", "to": "run", "conditions": [{"param": "speed", "op": "gt", "value": 0.1}]},
            {"from": "run", "to": "idle", "conditions": [{"param": "speed", "op": "lt", "value": 0.1}]},
            {"from": "*", "to": "jump", "conditions": [{"param": "jump", "op": "trigger"}]},
            {"from": "jump", "to": "idle", "wait_finished": True, "conditions": []},
        ],
    }


def _write_fsm(tmp_path, data, name="hero.gbanim"):
    (tmp_path / name).write_text(json.dumps(data), encoding="utf-8")


def test_default_state(run_gb, tmp_path):
    _write_fsm(tmp_path, _hero_fsm())
    src = textwrap.dedent('''
        IMPORT "animfsm"
        IMPORT "sprite"
        DIM sp AS SPRITE
        sp = SPRITE_NEW(0, 16, 16)
        DIM fsm AS ANIM_FSM
        fsm = ANIM_FSM_LOAD("hero.gbanim")
        ANIM_FSM_SETUP(fsm, sp)
        PRINT ANIM_FSM_STATE(fsm)
    ''')
    assert run_gb(src, base=tmp_path).strip() == "idle"


def test_idle_to_run_and_back(run_gb, tmp_path):
    _write_fsm(tmp_path, _hero_fsm())
    src = textwrap.dedent('''
        IMPORT "animfsm"
        IMPORT "sprite"
        DIM sp AS SPRITE
        sp = SPRITE_NEW(0, 16, 16)
        DIM fsm AS ANIM_FSM
        fsm = ANIM_FSM_LOAD("hero.gbanim")
        ANIM_FSM_SETUP(fsm, sp)
        ' anfahren -> run
        ANIM_FSM_SET_FLOAT(fsm, "speed", 5.0)
        DIM ch AS BOOLEAN
        ch = ANIM_FSM_UPDATE(fsm, sp, 16)
        PRINT ANIM_FSM_STATE(fsm)
        IF ch THEN PRINT "changed"
        ' stehenbleiben -> idle
        ANIM_FSM_SET_FLOAT(fsm, "speed", 0.0)
        ANIM_FSM_UPDATE(fsm, sp, 16)
        PRINT ANIM_FSM_STATE(fsm)
    ''')
    assert run_gb(src, base=tmp_path).strip().split("\n") == ["run", "changed", "idle"]


def test_any_state_trigger_jump(run_gb, tmp_path):
    _write_fsm(tmp_path, _hero_fsm())
    src = textwrap.dedent('''
        IMPORT "animfsm"
        IMPORT "sprite"
        DIM sp AS SPRITE
        sp = SPRITE_NEW(0, 16, 16)
        DIM fsm AS ANIM_FSM
        fsm = ANIM_FSM_LOAD("hero.gbanim")
        ANIM_FSM_SETUP(fsm, sp)
        ' aus idle per Any-State-Trigger nach jump
        ANIM_FSM_TRIGGER(fsm, "jump")
        ANIM_FSM_UPDATE(fsm, sp, 16)
        PRINT ANIM_FSM_STATE(fsm)
        ' jump ist one-shot -> nach genug Zeit wait_finished zurueck nach idle
        ANIM_FSM_UPDATE(fsm, sp, 300)
        PRINT ANIM_FSM_STATE(fsm)
    ''')
    assert run_gb(src, base=tmp_path).strip().split("\n") == ["jump", "idle"]


def test_trigger_consumed_after_update(run_gb, tmp_path):
    """Ein Trigger wirkt nur fuer EIN Update -- danach ist er verbraucht."""
    _write_fsm(tmp_path, _hero_fsm())
    src = textwrap.dedent('''
        IMPORT "animfsm"
        IMPORT "sprite"
        DIM sp AS SPRITE
        sp = SPRITE_NEW(0, 16, 16)
        DIM fsm AS ANIM_FSM
        fsm = ANIM_FSM_LOAD("hero.gbanim")
        ANIM_FSM_SETUP(fsm, sp)
        ANIM_FSM_TRIGGER(fsm, "jump")
        ANIM_FSM_UPDATE(fsm, sp, 16)
        PRINT ANIM_FSM_STATE(fsm)
        ' jump fertig -> idle
        ANIM_FSM_UPDATE(fsm, sp, 300)
        PRINT ANIM_FSM_STATE(fsm)
        ' KEIN neuer Trigger: bleibt idle (Trigger war verbraucht)
        ANIM_FSM_UPDATE(fsm, sp, 16)
        PRINT ANIM_FSM_STATE(fsm)
    ''')
    assert run_gb(src, base=tmp_path).strip().split("\n") == ["jump", "idle", "idle"]


def test_force_state(run_gb, tmp_path):
    _write_fsm(tmp_path, _hero_fsm())
    src = textwrap.dedent('''
        IMPORT "animfsm"
        IMPORT "sprite"
        DIM sp AS SPRITE
        sp = SPRITE_NEW(0, 16, 16)
        DIM fsm AS ANIM_FSM
        fsm = ANIM_FSM_LOAD("hero.gbanim")
        ANIM_FSM_SETUP(fsm, sp)
        ANIM_FSM_FORCE(fsm, sp, "run")
        PRINT ANIM_FSM_STATE(fsm)
    ''')
    assert run_gb(src, base=tmp_path).strip() == "run"


def test_get_params(run_gb, tmp_path):
    _write_fsm(tmp_path, _hero_fsm())
    src = textwrap.dedent('''
        IMPORT "animfsm"
        IMPORT "sprite"
        DIM sp AS SPRITE
        sp = SPRITE_NEW(0, 16, 16)
        DIM fsm AS ANIM_FSM
        fsm = ANIM_FSM_LOAD("hero.gbanim")
        ANIM_FSM_SET_FLOAT(fsm, "speed", 3.5)
        PRINT ANIM_FSM_GET_FLOAT(fsm, "speed")
    ''')
    assert run_gb(src, base=tmp_path).strip() == "3.5"


def test_setup_registers_anims_on_sprite(run_gb, tmp_path):
    """ANIM_FSM_SETUP registriert die Frame-Ranges als Sprite-Animationen."""
    _write_fsm(tmp_path, _hero_fsm())
    src = textwrap.dedent('''
        IMPORT "animfsm"
        IMPORT "sprite"
        DIM sp AS SPRITE
        sp = SPRITE_NEW(0, 16, 16)
        DIM fsm AS ANIM_FSM
        fsm = ANIM_FSM_LOAD("hero.gbanim")
        ANIM_FSM_SETUP(fsm, sp)
        ' default idle: frames 0..3 -> nach genug Zeit loopt der Frame im Bereich
        PRINT SPRITE_CURRENT_ANIM(sp)
    ''')
    assert run_gb(src, base=tmp_path).strip() == "idle"


def test_unknown_param_errors(run_gb, tmp_path):
    from drachenhauch.errors import DrachenhauchError
    _write_fsm(tmp_path, _hero_fsm())
    src = textwrap.dedent('''
        IMPORT "animfsm"
        IMPORT "sprite"
        DIM sp AS SPRITE
        sp = SPRITE_NEW(0, 16, 16)
        DIM fsm AS ANIM_FSM
        fsm = ANIM_FSM_LOAD("hero.gbanim")
        ANIM_FSM_SET_FLOAT(fsm, "nope", 1.0)
    ''')
    with pytest.raises(DrachenhauchError):
        run_gb(src, base=tmp_path)


def test_load_validates_bad_transition(run_gb, tmp_path):
    from drachenhauch.errors import DrachenhauchError
    bad = _hero_fsm()
    bad["transitions"].append({"from": "idle", "to": "ghost", "conditions": []})
    _write_fsm(tmp_path, bad)
    src = textwrap.dedent('''
        IMPORT "animfsm"
        DIM fsm AS ANIM_FSM
        fsm = ANIM_FSM_LOAD("hero.gbanim")
    ''')
    with pytest.raises(DrachenhauchError):
        run_gb(src, base=tmp_path)
