"""Golden-Tests fuer die 2D-Kamera-Rotation (CAMERA_SET_ROTATION/CAMERA_ROTATION,
rotations-bewusstes CAMERA_S2W_X/Y). Reine Positions-Rotation um die
Bildschirm-Mitte -- siehe rust/drachenhauch_runtime/src/graphics.rs (cam_rotation) und
docs/module-camera.md.
"""


def test_rotation_defaults_to_zero_and_is_backward_compatible(run_gb):
    out = run_gb('''
IMPORT "camera"
PRINT CAMERA_ROTATION()
CAMERA_SET(10.0, 20.0, 2.0)
PRINT CAMERA_S2W_X(0.0)
PRINT CAMERA_S2W_Y(0.0)
''')
    lines = out.strip().splitlines()
    assert lines[0] == "0.0"
    # Ein-Argument-Form bleibt bei Rotation=0 exakt wie vor dem Feature.
    assert lines[1] == "10.0"
    assert lines[2] == "20.0"


def test_camera_reset_also_clears_rotation(run_gb):
    out = run_gb('''
IMPORT "camera"
CAMERA_SET_ROTATION(45.0)
CAMERA_RESET()
PRINT CAMERA_ROTATION()
''')
    assert out.strip() == "0.0"


def test_camera_set_optional_fourth_arg_sets_rotation(run_gb):
    out = run_gb('''
IMPORT "camera"
CAMERA_SET(0.0, 0.0, 1.0, 33.0)
PRINT CAMERA_ROTATION()
''')
    assert out.strip() == "33.0"


def test_rotation_90_degrees_matches_hand_calculation(run_gb):
    # SCREEN(200,200) -> Pivot (100,100). Weltpunkt (50,0) landet bei
    # CAMERA_SET_ROTATION(90) auf Screen (0,150) (siehe graphics.rs
    # camera_rotation_tests::ninety_degrees_matches_hand_calculation).
    out = run_gb('''
IMPORT "camera"
SCREEN(200, 200, "t")
CAMERA_RESET()
CAMERA_SET_ROTATION(90.0)
PRINT ROUND(CAMERA_S2W_X(0.0, 150.0))
PRINT ROUND(CAMERA_S2W_Y(150.0, 0.0))
''')
    lines = out.strip().splitlines()
    assert lines == ["50", "0"]


def test_rotation_180_degrees_is_point_reflection(run_gb):
    out = run_gb('''
IMPORT "camera"
SCREEN(200, 200, "t")
CAMERA_RESET()
CAMERA_SET_ROTATION(180.0)
PRINT ROUND(CAMERA_S2W_X(200.0, 200.0))
PRINT ROUND(CAMERA_S2W_Y(200.0, 200.0))
''')
    lines = out.strip().splitlines()
    assert lines == ["0", "0"]


def test_camera_follow_leaves_rotation_untouched(run_gb):
    # CAMERA_FOLLOW setzt nur x/y (ueber set_camera), Rotation bleibt unberuehrt
    # -- bei zoom=1 (Default) landet x/y bei target - screen/2.
    out = run_gb('''
IMPORT "camera"
CAMERA_SET_ROTATION(77.0)
CAMERA_FOLLOW(500.0, 500.0, 200.0, 200.0)
PRINT CAMERA_X()
PRINT CAMERA_Y()
PRINT CAMERA_ROTATION()
''')
    lines = out.strip().splitlines()
    assert lines == ["400.0", "400.0", "77.0"]
