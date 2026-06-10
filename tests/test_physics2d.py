"""Golden-Tests fuer das native `physics2d`-Modul (Rapier2D).

Reine Logik (kein Grafik-State) -> konsolen-testbar: Welt aufbauen, simulieren,
Koerper-Positionen abfragen. Qualitative Asserts (Bereiche), keine Bit-Floats.
"""
import re


def _num(s: str) -> float:
    return float(s.strip().split("\n")[-1])


def test_box_falls_and_rests_on_floor(run_gb):
    # Dynamische Box faellt unter Schwerkraft auf einen statischen Boden und
    # bleibt darauf liegen (Boden-Oberkante 390, Box-Halbhoehe 16 -> Ruhe ~374).
    src = '''
        IMPORT "physics2d"
        DIM w AS PHYS2D_WORLD
        w = PHYS2D_NEW()
        DIM floor AS INTEGER
        floor = PHYS2D_ADD_BOX(w, 240.0, 400.0, 240.0, 10.0, FALSE, 0.0)
        DIM ball AS INTEGER
        ball = PHYS2D_ADD_BOX(w, 240.0, 50.0, 16.0, 16.0, TRUE, 0.0)
        DIM i AS INTEGER
        FOR i = 1 TO 180
            PHYS2D_STEP(w, 0.0166667)
        NEXT
        PRINT INT(PHYS2D_BODY_Y(w, ball))
    '''
    y = _num(run_gb(src))
    assert 365.0 <= y <= 380.0, f"Box sollte auf dem Boden ruhen (~374), war {y}"


def test_gravity_pulls_down_by_default(run_gb):
    # Default-Schwerkraft zieht nach UNTEN (+Y, Bildschirm-Konvention).
    src = '''
        IMPORT "physics2d"
        DIM w AS PHYS2D_WORLD
        w = PHYS2D_NEW()
        DIM b AS INTEGER
        b = PHYS2D_ADD_CIRCLE(w, 100.0, 100.0, 8.0, TRUE, 0.0)
        DIM i AS INTEGER
        FOR i = 1 TO 30
            PHYS2D_STEP(w, 0.0166667)
        NEXT
        IF PHYS2D_BODY_Y(w, b) > 100.0 THEN PRINT "fell"
    '''
    assert run_gb(src).strip() == "fell"


def test_set_gravity_direction(run_gb):
    # Eigene Schwerkraft nach OBEN -> Koerper steigt (y sinkt).
    src = '''
        IMPORT "physics2d"
        DIM w AS PHYS2D_WORLD
        w = PHYS2D_NEW()
        PHYS2D_SET_GRAVITY(w, 0.0, -980.0)
        DIM b AS INTEGER
        b = PHYS2D_ADD_CIRCLE(w, 100.0, 300.0, 8.0, TRUE, 0.0)
        DIM i AS INTEGER
        FOR i = 1 TO 30
            PHYS2D_STEP(w, 0.0166667)
        NEXT
        IF PHYS2D_BODY_Y(w, b) < 300.0 THEN PRINT "rose"
    '''
    assert run_gb(src).strip() == "rose"


def test_impulse_moves_body(run_gb):
    # Impuls nach rechts -> positive x-Geschwindigkeit + x waechst.
    src = '''
        IMPORT "physics2d"
        DIM w AS PHYS2D_WORLD
        w = PHYS2D_NEW()
        PHYS2D_SET_GRAVITY(w, 0.0, 0.0)
        DIM b AS INTEGER
        b = PHYS2D_ADD_CIRCLE(w, 50.0, 50.0, 8.0, TRUE, 0.0)
        PHYS2D_APPLY_IMPULSE(w, b, 500.0, 0.0)
        DIM i AS INTEGER
        FOR i = 1 TO 20
            PHYS2D_STEP(w, 0.0166667)
        NEXT
        IF PHYS2D_BODY_VX(w, b) > 0.0 AND PHYS2D_BODY_X(w, b) > 50.0 THEN PRINT "moved"
    '''
    assert run_gb(src).strip() == "moved"


def test_count_and_remove(run_gb):
    src = '''
        IMPORT "physics2d"
        DIM w AS PHYS2D_WORLD
        w = PHYS2D_NEW()
        DIM a AS INTEGER
        a = PHYS2D_ADD_CIRCLE(w, 10.0, 10.0, 4.0, TRUE, 0.0)
        DIM b AS INTEGER
        b = PHYS2D_ADD_CIRCLE(w, 20.0, 20.0, 4.0, TRUE, 0.0)
        PRINT PHYS2D_COUNT(w)
        PHYS2D_REMOVE(w, a)
        PRINT PHYS2D_COUNT(w)
    '''
    assert run_gb(src).strip().split("\n") == ["2", "1"]


def test_lock_rotation_keeps_angle_zero(run_gb):
    # Box auf schiefer Lage: ohne Lock wuerde sie kippen; mit Lock bleibt der
    # Winkel ~0 (Spielfigur-Pattern).
    src = '''
        IMPORT "physics2d"
        DIM w AS PHYS2D_WORLD
        w = PHYS2D_NEW()
        DIM floor AS INTEGER
        floor = PHYS2D_ADD_BOX(w, 240.0, 400.0, 240.0, 10.0, FALSE, 0.0)
        DIM box AS INTEGER
        box = PHYS2D_ADD_BOX(w, 100.0, 50.0, 16.0, 16.0, TRUE, 0.0)
        PHYS2D_LOCK_ROTATION(w, box, TRUE)
        PHYS2D_APPLY_IMPULSE(w, box, 200.0, 0.0)
        DIM i AS INTEGER
        FOR i = 1 TO 180
            PHYS2D_STEP(w, 0.0166667)
        NEXT
        IF ABS(PHYS2D_BODY_ANGLE(w, box)) < 0.05 THEN PRINT "upright"
    '''
    assert run_gb(src).strip() == "upright"


def test_static_floor_does_not_move(run_gb):
    src = '''
        IMPORT "physics2d"
        DIM w AS PHYS2D_WORLD
        w = PHYS2D_NEW()
        DIM floor AS INTEGER
        floor = PHYS2D_ADD_BOX(w, 240.0, 400.0, 240.0, 10.0, FALSE, 0.0)
        DIM i AS INTEGER
        FOR i = 1 TO 60
            PHYS2D_STEP(w, 0.0166667)
        NEXT
        PRINT INT(PHYS2D_BODY_Y(w, floor))
    '''
    assert _num(run_gb(src)) == 400.0


def test_bad_world_type_errors(run_gb):
    from gamebasic.errors import GameBasicError
    import pytest
    src = '''
        IMPORT "physics2d"
        PHYS2D_STEP(42, 0.016)
    '''
    with pytest.raises(GameBasicError):
        run_gb(src)
