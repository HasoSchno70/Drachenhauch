"""Golden-Tests fuer das physics3d-Modul (Rapier3D-Starrkoerper).

run_gb spawnt `dhrt run` -> skippt automatisch, wenn dhrt nicht gebaut ist.
Konsolen-Test (keine Grafik): Schwerkraft/Kollision per qualitativer Booleans
pruefen (keine exakten Floats -> robust ueber Plattformen).
"""


def _gb(body: str) -> str:
    return 'IMPORT "physics3d"\n' + body


def test_ball_faellt_auf_boden(run_gb):
    out = run_gb(_gb(
        "DIM w AS PHYS_WORLD\n"
        "w = PHYS3D_NEW()\n"
        "DIM g AS INTEGER\n"
        "DIM b AS INTEGER\n"
        "g = PHYS3D_ADD_BOX(w, 0.0, 0.0, 0.0, 20.0, 0.5, 20.0, 0, 0.0)\n"
        "b = PHYS3D_ADD_SPHERE(w, 0.0, 10.0, 0.0, 0.5, 1, 0.3)\n"
        "DIM i AS INTEGER\n"
        "FOR i = 0 TO 179\n"
        "    PHYS3D_STEP(w, 0.0166)\n"
        "NEXT\n"
        "PRINT PHYS3D_BODY_Y(w, b) < 2.0\n"       # gefallen
        "PRINT PHYS3D_BODY_Y(w, b) > 0.5\n"       # ruht auf dem Boden (nicht durchgefallen)
        "PRINT PHYS3D_COUNT(w)\n"
    ))
    assert out.splitlines() == ["TRUE", "TRUE", "2"]


def test_gravity_und_impulse(run_gb):
    # Ohne Schwerkraft + seitlicher Impuls -> x waechst, y bleibt ~konstant.
    out = run_gb(_gb(
        "DIM w AS PHYS_WORLD\n"
        "w = PHYS3D_NEW()\n"
        "PHYS3D_SET_GRAVITY(w, 0.0, 0.0, 0.0)\n"
        "DIM b AS INTEGER\n"
        "b = PHYS3D_ADD_SPHERE(w, 0.0, 5.0, 0.0, 0.5, 1, 0.0)\n"
        "PHYS3D_APPLY_IMPULSE(w, b, 5.0, 0.0, 0.0)\n"
        "DIM i AS INTEGER\n"
        "FOR i = 0 TO 59\n"
        "    PHYS3D_STEP(w, 0.0166)\n"
        "NEXT\n"
        "PRINT PHYS3D_BODY_X(w, b) > 0.5\n"       # nach rechts bewegt
        "PRINT ABS(PHYS3D_BODY_Y(w, b) - 5.0) < 0.5\n"  # keine Schwerkraft -> y bleibt
    ))
    assert out.splitlines() == ["TRUE", "TRUE"]


def test_remove_und_count(run_gb):
    out = run_gb(_gb(
        "DIM w AS PHYS_WORLD\n"
        "w = PHYS3D_NEW()\n"
        "DIM a AS INTEGER\n"
        "DIM b AS INTEGER\n"
        "a = PHYS3D_ADD_SPHERE(w, 0.0, 5.0, 0.0, 0.5, 1, 0.0)\n"
        "b = PHYS3D_ADD_SPHERE(w, 2.0, 5.0, 0.0, 0.5, 1, 0.0)\n"
        "PRINT PHYS3D_COUNT(w)\n"
        "PHYS3D_REMOVE(w, a)\n"
        "PRINT PHYS3D_COUNT(w)\n"
    ))
    assert out.splitlines() == ["2", "1"]
