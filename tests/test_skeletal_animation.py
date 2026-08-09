"""Skelett-Animation (g3d): MODEL_LOAD_ANIMS/MODEL_ANIMATE + MODEL_ANIMATE_BLEND.

MODEL_ANIMATE_BLEND nutzt raylib-rs 6.0s neue `UpdateModelAnimationEx` (siehe
graphics.rs `model_animate_blend`) -- blendet zwischen zwei Animationen
desselben Sets statt hartem Wechsel. Nutzt das vorhandene CC0-Robotermodell
aus examples/assets/robot.glb (bereits fuer examples/108_skeletal_anim.gb da).
"""
from pathlib import Path

import pytest

from drachenhauch.errors import DrachenhauchError

_ROBOT = (Path(__file__).resolve().parent.parent / "examples" / "assets" / "robot.glb")
_ROBOT_POSIX = _ROBOT.as_posix()


def _setup(extra: str) -> str:
    return f'''
IMPORT "g3d"
SCREEN(320, 240)
DIM robot AS INTEGER
DIM anims AS INTEGER
robot = LOADMODEL("{_ROBOT_POSIX}")
anims = MODEL_LOAD_ANIMS("{_ROBOT_POSIX}")
{extra}
'''


@pytest.fixture(scope="module", autouse=True)
def _require_robot_asset():
    if not _ROBOT.exists():
        pytest.skip("examples/assets/robot.glb fehlt (py examples/assets/download_robot.py)")


def test_model_animate_blend_smoke(run_gb):
    """Blend zwischen zwei verschiedenen Animationen darf nicht crashen."""
    src = _setup(
        'MODEL_ANIMATE_BLEND(robot, anims, 0, 5, 10, 3, 0.5)\n'
        'PRINT "OK"\n'
    )
    assert run_gb(src) == "OK\n"


def test_model_animate_blend_endpoints_match_single_frame_pose(run_gb):
    """blend=0.0/1.0 (reine A- bzw. B-Pose) darf ebenfalls nicht crashen --
    Endpunkte des Blend-Bereichs sind der haeufigste Off-by-one-Fehlerfall."""
    src = _setup(
        'MODEL_ANIMATE_BLEND(robot, anims, 0, 2, 10, 7, 0.0)\n'
        'MODEL_ANIMATE_BLEND(robot, anims, 0, 2, 10, 7, 1.0)\n'
        'PRINT "OK"\n'
    )
    assert run_gb(src) == "OK\n"


def test_model_animate_blend_loops_frame_like_model_animate(run_gb):
    """Frame-Index ausserhalb der Keyframe-Anzahl loopt (rem_euclid), analog
    zu MODEL_ANIMATE -- kein Fehler bei grossen/negativen Frame-Werten."""
    src = _setup(
        'MODEL_ANIMATE_BLEND(robot, anims, 0, 99999, 10, -5, 0.3)\n'
        'PRINT "OK"\n'
    )
    assert run_gb(src) == "OK\n"


def test_model_animate_blend_invalid_anim_index_raises(run_gb):
    src = _setup('MODEL_ANIMATE_BLEND(robot, anims, 0, 0, 9999, 0, 0.5)\n')
    with pytest.raises(DrachenhauchError, match="MODEL_ANIMATE_BLEND"):
        run_gb(src)


def test_model_animate_blend_invalid_model_raises(run_gb):
    src = _setup('MODEL_ANIMATE_BLEND(9999, anims, 0, 0, 1, 0, 0.5)\n')
    with pytest.raises(DrachenhauchError, match="MODEL_ANIMATE_BLEND"):
        run_gb(src)
