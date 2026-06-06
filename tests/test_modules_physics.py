"""Tests fuer das Physics-Modul (pure Collision-/Vektor-/Ray-Funktionen).

Golden-Tests gegen `gbrt` (Stufe B): IMPORT "physics" + PRINT. Frueher via
`call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _p(run_gb, *exprs):
    src = 'IMPORT "physics"\n' + "".join(f"PRINT {e}\n" for e in exprs)
    return _lines(run_gb(src))


# --- Collision: Box-Box ---------------------------------------------

def test_box_box_overlap(run_gb):
    assert _p(run_gb, "PHYSICS_BOX_BOX(0, 0, 10, 10, 5, 5, 10, 10)") == ["TRUE"]


def test_box_box_no_overlap(run_gb):
    assert _p(run_gb, "PHYSICS_BOX_BOX(0, 0, 10, 10, 20, 20, 5, 5)") == ["FALSE"]


def test_box_box_touch_edge_no_overlap(run_gb):
    assert _p(run_gb, "PHYSICS_BOX_BOX(0, 0, 10, 10, 10, 0, 5, 5)") == ["FALSE"]


# --- Collision: Circle-Circle ---------------------------------------

def test_circle_circle_overlap(run_gb):
    assert _p(run_gb, "PHYSICS_CIRCLE_CIRCLE(0, 0, 5, 6, 0, 5)") == ["TRUE"]


def test_circle_circle_no_overlap(run_gb):
    assert _p(run_gb, "PHYSICS_CIRCLE_CIRCLE(0, 0, 3, 100, 0, 3)") == ["FALSE"]


def test_circle_circle_touch(run_gb):
    assert _p(run_gb, "PHYSICS_CIRCLE_CIRCLE(0, 0, 5, 10, 0, 5)") == ["FALSE"]


# --- Collision: Box-Circle ------------------------------------------

def test_box_circle_circle_inside_box(run_gb):
    assert _p(run_gb, "PHYSICS_BOX_CIRCLE(0, 0, 20, 20, 10, 10, 3)") == ["TRUE"]


def test_box_circle_circle_corner_overlap(run_gb):
    assert _p(run_gb, "PHYSICS_BOX_CIRCLE(0, 0, 10, 10, 12, 12, 5)") == ["TRUE"]


def test_box_circle_no_overlap(run_gb):
    assert _p(run_gb, "PHYSICS_BOX_CIRCLE(0, 0, 10, 10, 100, 100, 5)") == ["FALSE"]


# --- Collision: Point-Box / Point-Circle ----------------------------

def test_point_in_box(run_gb):
    assert _p(run_gb, "PHYSICS_POINT_BOX(5, 5, 0, 0, 10, 10)") == ["TRUE"]


def test_point_outside_box(run_gb):
    assert _p(run_gb, "PHYSICS_POINT_BOX(15, 5, 0, 0, 10, 10)") == ["FALSE"]


def test_point_on_left_edge_inclusive(run_gb):
    """Linke obere Kante einschliesslich, rechte/untere ausschliesslich."""
    assert _p(run_gb,
              "PHYSICS_POINT_BOX(0, 0, 0, 0, 10, 10)",
              "PHYSICS_POINT_BOX(10, 5, 0, 0, 10, 10)") == ["TRUE", "FALSE"]


def test_point_in_circle(run_gb):
    assert _p(run_gb,
              "PHYSICS_POINT_CIRCLE(3, 4, 0, 0, 5)",   # genau Border = False
              "PHYSICS_POINT_CIRCLE(3, 4, 0, 0, 6)") == ["FALSE", "TRUE"]


# --- Distance / Length ---------------------------------------------

def test_distance_pythagoras(run_gb):
    assert _p(run_gb, "PHYSICS_DISTANCE(0, 0, 3, 4)") == ["5.0"]


def test_distance2_squared(run_gb):
    assert _p(run_gb, "PHYSICS_DISTANCE2(0, 0, 3, 4)") == ["25.0"]


def test_length(run_gb):
    assert _p(run_gb, "PHYSICS_LENGTH(3, 4)") == ["5.0"]


def test_length_zero(run_gb):
    assert _p(run_gb, "PHYSICS_LENGTH(0, 0)") == ["0.0"]


# --- Normalize -----------------------------------------------------

def test_norm_unit_vec(run_gb):
    assert _p(run_gb, "PHYSICS_NORM_X(3, 4)", "PHYSICS_NORM_Y(3, 4)") == ["0.6", "0.8"]


def test_norm_zero_vec(run_gb):
    assert _p(run_gb, "PHYSICS_NORM_X(0, 0)", "PHYSICS_NORM_Y(0, 0)") == ["0.0", "0.0"]


# --- Reflect -------------------------------------------------------

def test_reflect_perpendicular(run_gb):
    """Vektor (1, -1) prallt von horizontaler Wand (Normal 0, 1) ab -> (1, 1)."""
    assert _p(run_gb,
              "PHYSICS_REFLECT_X(1, -1, 0, 1)",
              "PHYSICS_REFLECT_Y(1, -1, 0, 1)") == ["1.0", "1.0"]


def test_reflect_normal_not_normalized(run_gb):
    """Normal muss nicht praenormalisiert sein."""
    assert _p(run_gb,
              "PHYSICS_REFLECT_X(1, -1, 0, 5)",
              "PHYSICS_REFLECT_Y(1, -1, 0, 5)") == ["1.0", "1.0"]


def test_reflect_zero_normal_returns_input(run_gb):
    """Entartete (0,0)-Normal -> Eingabe-Vektor unveraendert zurueckgeben."""
    assert _p(run_gb,
              "PHYSICS_REFLECT_X(3, 4, 0, 0)",
              "PHYSICS_REFLECT_Y(3, 4, 0, 0)") == ["3.0", "4.0"]


# --- Ray-Cast: Box -------------------------------------------------

def test_ray_box_hits(run_gb):
    assert _p(run_gb, "PHYSICS_RAY_BOX(0, 5, 10, 0, 5, 0, 10, 10)") == ["0.5"]


def test_ray_box_misses(run_gb):
    assert _p(run_gb, "PHYSICS_RAY_BOX(0, 50, 10, 0, 5, 0, 10, 10)") == ["-1.0"]


def test_ray_box_starts_inside(run_gb):
    assert _p(run_gb, "PHYSICS_RAY_BOX(10, 5, 5, 0, 5, 0, 10, 10)") == ["0.0"]


def test_ray_box_too_short(run_gb):
    assert _p(run_gb, "PHYSICS_RAY_BOX(0, 5, 4, 0, 5, 0, 10, 10)") == ["-1.0"]


def test_ray_box_zero_direction(run_gb):
    assert _p(run_gb, "PHYSICS_RAY_BOX(10, 5, 0, 0, 5, 0, 10, 10)") == ["0.0"]


def test_ray_box_zero_direction_outside(run_gb):
    assert _p(run_gb, "PHYSICS_RAY_BOX(50, 50, 0, 0, 5, 0, 10, 10)") == ["-1.0"]


# --- Ray-Cast: Circle ----------------------------------------------

def test_ray_circle_hits(run_gb):
    assert _p(run_gb, "PHYSICS_RAY_CIRCLE(0, 0, 10, 0, 5, 0, 1)") == ["0.4"]


def test_ray_circle_misses(run_gb):
    assert _p(run_gb, "PHYSICS_RAY_CIRCLE(0, 0, 10, 0, 5, 100, 1)") == ["-1.0"]


def test_ray_circle_starts_inside(run_gb):
    t = float(_p(run_gb, "PHYSICS_RAY_CIRCLE(5, 0, 10, 0, 5, 0, 5)")[0])
    assert 0 <= t <= 1


def test_ray_circle_zero_radius_no_hit(run_gb):
    t = float(_p(run_gb, "PHYSICS_RAY_CIRCLE(0, 0, 10, 0, 5, 0, 0)")[0])
    assert t in (-1.0, 0.5) or abs(t - 0.5) < 1e-6
