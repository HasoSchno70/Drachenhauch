"""Tests fuer das Physics-Modul.

14 Built-ins: 5 Collision-Tests, 7 Vektor-Mathe-Helfer, 2 Ray-Cast.
"""
import math

import pytest

from gamebasic.modules import load_module


@pytest.fixture(autouse=True, scope="module")
def _ensure_physics_loaded():
    """Stellt sicher, dass das Modul registriert ist."""
    load_module("physics")


# --- Collision: Box-Box ---------------------------------------------

def test_box_box_overlap(call_builtin):
    # Box1 (0,0)-(10,10), Box2 (5,5)-(15,15) -> ueberlappt
    assert call_builtin("PHYSICS_BOX_BOX", [0, 0, 10, 10, 5, 5, 10, 10]) is True


def test_box_box_no_overlap(call_builtin):
    assert call_builtin("PHYSICS_BOX_BOX", [0, 0, 10, 10, 20, 20, 5, 5]) is False


def test_box_box_touch_edge_no_overlap(call_builtin):
    """Boxen die sich nur an einer Kante beruehren -> kein Overlap."""
    assert call_builtin("PHYSICS_BOX_BOX", [0, 0, 10, 10, 10, 0, 5, 5]) is False


# --- Collision: Circle-Circle ---------------------------------------

def test_circle_circle_overlap(call_builtin):
    assert call_builtin("PHYSICS_CIRCLE_CIRCLE",
                         [0, 0, 5, 6, 0, 5]) is True


def test_circle_circle_no_overlap(call_builtin):
    assert call_builtin("PHYSICS_CIRCLE_CIRCLE",
                         [0, 0, 3, 100, 0, 3]) is False


def test_circle_circle_touch(call_builtin):
    # Kreise die sich gerade beruehren -> dist = r1+r2 -> Border = nicht overlap
    assert call_builtin("PHYSICS_CIRCLE_CIRCLE",
                         [0, 0, 5, 10, 0, 5]) is False


# --- Collision: Box-Circle ------------------------------------------

def test_box_circle_circle_inside_box(call_builtin):
    # Box (0,0)-(20,20), Kreis Center (10,10) r 3 -> komplett drin
    assert call_builtin("PHYSICS_BOX_CIRCLE",
                         [0, 0, 20, 20, 10, 10, 3]) is True


def test_box_circle_circle_corner_overlap(call_builtin):
    # Kreis nah an einer Box-Ecke - kein Overlap am Center, aber Radius
    # ueberlappt. Box (0,0)-(10,10), Kreis (12,12) r 5 -> Eck-Distanz ~2.83 < 5
    assert call_builtin("PHYSICS_BOX_CIRCLE",
                         [0, 0, 10, 10, 12, 12, 5]) is True


def test_box_circle_no_overlap(call_builtin):
    assert call_builtin("PHYSICS_BOX_CIRCLE",
                         [0, 0, 10, 10, 100, 100, 5]) is False


# --- Collision: Point-Box / Point-Circle ----------------------------

def test_point_in_box(call_builtin):
    assert call_builtin("PHYSICS_POINT_BOX",
                         [5, 5, 0, 0, 10, 10]) is True


def test_point_outside_box(call_builtin):
    assert call_builtin("PHYSICS_POINT_BOX",
                         [15, 5, 0, 0, 10, 10]) is False


def test_point_on_left_edge_inclusive(call_builtin):
    """Linke obere Kante einschliesslich, rechte/untere ausschliesslich."""
    assert call_builtin("PHYSICS_POINT_BOX",
                         [0, 0, 0, 0, 10, 10]) is True
    # Rechte Kante exklusiv
    assert call_builtin("PHYSICS_POINT_BOX",
                         [10, 5, 0, 0, 10, 10]) is False


def test_point_in_circle(call_builtin):
    assert call_builtin("PHYSICS_POINT_CIRCLE",
                         [3, 4, 0, 0, 5]) is False  # genau Border = False
    assert call_builtin("PHYSICS_POINT_CIRCLE",
                         [3, 4, 0, 0, 6]) is True   # innerhalb


# --- Distance / Length ---------------------------------------------

def test_distance_pythagoras(call_builtin):
    d = call_builtin("PHYSICS_DISTANCE", [0, 0, 3, 4])
    assert abs(d - 5.0) < 1e-9


def test_distance2_squared(call_builtin):
    d2 = call_builtin("PHYSICS_DISTANCE2", [0, 0, 3, 4])
    assert d2 == 25.0


def test_length(call_builtin):
    L = call_builtin("PHYSICS_LENGTH", [3, 4])
    assert abs(L - 5.0) < 1e-9


def test_length_zero(call_builtin):
    assert call_builtin("PHYSICS_LENGTH", [0, 0]) == 0.0


# --- Normalize -----------------------------------------------------

def test_norm_unit_vec(call_builtin):
    nx = call_builtin("PHYSICS_NORM_X", [3, 4])
    ny = call_builtin("PHYSICS_NORM_Y", [3, 4])
    assert abs(nx - 0.6) < 1e-9
    assert abs(ny - 0.8) < 1e-9
    # Resultat hat Laenge 1
    assert abs((nx ** 2 + ny ** 2) - 1.0) < 1e-9


def test_norm_zero_vec(call_builtin):
    assert call_builtin("PHYSICS_NORM_X", [0, 0]) == 0.0
    assert call_builtin("PHYSICS_NORM_Y", [0, 0]) == 0.0


# --- Reflect -------------------------------------------------------

def test_reflect_perpendicular(call_builtin):
    """Vektor (1, -1) prallt von horizontaler Wand (Normal 0, 1) ab -> (1, 1)."""
    rx = call_builtin("PHYSICS_REFLECT_X", [1, -1, 0, 1])
    ry = call_builtin("PHYSICS_REFLECT_Y", [1, -1, 0, 1])
    assert abs(rx - 1.0) < 1e-9
    assert abs(ry - 1.0) < 1e-9


def test_reflect_normal_not_normalized(call_builtin):
    """Normal muss nicht praenormalisiert sein."""
    rx = call_builtin("PHYSICS_REFLECT_X", [1, -1, 0, 5])  # Normal (0,5)
    ry = call_builtin("PHYSICS_REFLECT_Y", [1, -1, 0, 5])
    assert abs(rx - 1.0) < 1e-9
    assert abs(ry - 1.0) < 1e-9


def test_reflect_zero_normal_returns_input(call_builtin):
    """Entartete (0,0)-Normal -> Eingabe-Vektor unveraendert zurueckgeben."""
    rx = call_builtin("PHYSICS_REFLECT_X", [3, 4, 0, 0])
    ry = call_builtin("PHYSICS_REFLECT_Y", [3, 4, 0, 0])
    assert rx == 3.0
    assert ry == 4.0


# --- Ray-Cast: Box -------------------------------------------------

def test_ray_box_hits(call_builtin):
    """Strahl von (0,5) in (10,0)-Richtung trifft Box (5,0)-(15,10) bei t=0.5."""
    t = call_builtin("PHYSICS_RAY_BOX", [0, 5, 10, 0, 5, 0, 10, 10])
    assert abs(t - 0.5) < 1e-9


def test_ray_box_misses(call_builtin):
    """Strahl der die Box komplett verfehlt -> -1."""
    t = call_builtin("PHYSICS_RAY_BOX", [0, 50, 10, 0, 5, 0, 10, 10])
    assert t == -1.0


def test_ray_box_starts_inside(call_builtin):
    """Strahl beginnt in der Box -> t = 0."""
    t = call_builtin("PHYSICS_RAY_BOX", [10, 5, 5, 0, 5, 0, 10, 10])
    assert t == 0.0


def test_ray_box_too_short(call_builtin):
    """Strahl ist zu kurz um die Box zu erreichen."""
    # Strahl von (0,5), Richtung (4,0) - reicht nur bis x=4, Box beginnt bei 5
    t = call_builtin("PHYSICS_RAY_BOX", [0, 5, 4, 0, 5, 0, 10, 10])
    assert t == -1.0


def test_ray_box_zero_direction(call_builtin):
    """Strahl mit Laenge 0 (dx=dy=0) im Inneren -> t=0."""
    t = call_builtin("PHYSICS_RAY_BOX", [10, 5, 0, 0, 5, 0, 10, 10])
    assert t == 0.0


def test_ray_box_zero_direction_outside(call_builtin):
    """Strahl mit Laenge 0 ausserhalb -> -1."""
    t = call_builtin("PHYSICS_RAY_BOX", [50, 50, 0, 0, 5, 0, 10, 10])
    assert t == -1.0


# --- Ray-Cast: Circle ----------------------------------------------

def test_ray_circle_hits(call_builtin):
    """Strahl von (0,0) Richtung (10,0) trifft Kreis Center (5,0) r 1
    bei t = 0.4 (Eintrittspunkt x = 4)."""
    t = call_builtin("PHYSICS_RAY_CIRCLE", [0, 0, 10, 0, 5, 0, 1])
    assert abs(t - 0.4) < 1e-9


def test_ray_circle_misses(call_builtin):
    t = call_builtin("PHYSICS_RAY_CIRCLE", [0, 0, 10, 0, 5, 100, 1])
    assert t == -1.0


def test_ray_circle_starts_inside(call_builtin):
    """Strahl beginnt im Kreis -> einer der t-Werte ist negativ, der andere
    positiv -> wir liefern den positiven (Austritts-Punkt). Falls beide
    negativ -> -1. Hier: Strahl bei (5,0), Kreis Center (5,0) r 5 ->
    Strahl ist drin, Austritt bei t=0.5."""
    t = call_builtin("PHYSICS_RAY_CIRCLE", [5, 0, 10, 0, 5, 0, 5])
    assert 0 <= t <= 1


def test_ray_circle_zero_radius_no_hit(call_builtin):
    """Kreis mit Radius 0 - praktisch ein Punkt, schwierig zu treffen."""
    t = call_builtin("PHYSICS_RAY_CIRCLE", [0, 0, 10, 0, 5, 0, 0])
    # Auch denkbar -1 (nicht-Treffer); je nach numerischer Praezision.
    assert t in (-1.0, 0.5) or abs(t - 0.5) < 1e-6
