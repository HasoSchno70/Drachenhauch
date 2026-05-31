"""Tests fuer das sprite-Modul.

Wir bauen ein Sprite mit einer Pygame-Surface direkt (kein File-Loading,
kein SCREEN). SPRITE_DRAW pruefen wir nicht hier - das wuerde Pygame-Display
brauchen. Logik (Update, Animation, Kollision) ist davon unabhaengig.
"""
import os
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError
from gamebasic.interpreter import _Image


@pytest.fixture(scope="module", autouse=True)
def _load_sprite():
    assert load_module("sprite")


@pytest.fixture(scope="module")
def sheet_image():
    """Ein synthetisches 64x16 Pygame-Surface als _Image-Wrapper."""
    os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "hide")
    import pygame
    pygame.init()
    surf = pygame.Surface((64, 16))
    return _Image(surf, "<test-sheet>")


@pytest.fixture
def sprite(call_builtin, sheet_image):
    return call_builtin("sprite_new", [sheet_image, 16, 16])


# --- Konstruktion ----------------------------------------------------

def test_new_starts_at_origin(call_builtin, sprite):
    assert call_builtin("sprite_get_x", [sprite]) == 0.0
    assert call_builtin("sprite_get_y", [sprite]) == 0.0


def test_get_size(call_builtin, sprite):
    assert call_builtin("sprite_get_width", [sprite]) == 16
    assert call_builtin("sprite_get_height", [sprite]) == 16


def test_new_invalid_frame_size_raises(call_builtin, sheet_image):
    with pytest.raises(GBRuntimeError, match="> 0"):
        call_builtin("sprite_new", [sheet_image, 0, 16])


# --- Position & Velocity --------------------------------------------

def test_set_pos(call_builtin, sprite):
    call_builtin("sprite_set_pos", [sprite, 50.0, 75.0])
    assert call_builtin("sprite_get_x", [sprite]) == 50.0
    assert call_builtin("sprite_get_y", [sprite]) == 75.0


def test_velocity_moves_position(call_builtin, sprite):
    call_builtin("sprite_set_pos", [sprite, 0.0, 0.0])
    call_builtin("sprite_set_velocity", [sprite, 100.0, -50.0])
    call_builtin("sprite_update", [sprite, 1000])  # 1 Sekunde
    assert call_builtin("sprite_get_x", [sprite]) == 100.0
    assert call_builtin("sprite_get_y", [sprite]) == -50.0


# --- Animationen -----------------------------------------------------

def test_play_unknown_anim_raises(call_builtin, sprite):
    with pytest.raises(GBRuntimeError, match="unbekannte Animation"):
        call_builtin("sprite_play", [sprite, "fly"])


def test_walk_loops(call_builtin, sprite):
    call_builtin("sprite_add_anim", [sprite, "walk", 0, 3, 8.0])  # 125ms/frame
    call_builtin("sprite_play", [sprite, "walk"])
    assert call_builtin("sprite_get_frame", [sprite]) == 0
    call_builtin("sprite_update", [sprite, 125])
    assert call_builtin("sprite_get_frame", [sprite]) == 1
    call_builtin("sprite_update", [sprite, 125])
    assert call_builtin("sprite_get_frame", [sprite]) == 2
    call_builtin("sprite_update", [sprite, 125])
    assert call_builtin("sprite_get_frame", [sprite]) == 3
    # Loop zurueck zu 0
    call_builtin("sprite_update", [sprite, 125])
    assert call_builtin("sprite_get_frame", [sprite]) == 0


def test_play_once_stops_at_last_frame(call_builtin, sprite):
    call_builtin("sprite_add_anim", [sprite, "punch", 0, 3, 12.0])  # ~83ms/frame
    call_builtin("sprite_play_once", [sprite, "punch"])
    assert call_builtin("sprite_is_finished", [sprite]) is False
    call_builtin("sprite_update", [sprite, 400])  # > 333ms (4*83)
    assert call_builtin("sprite_get_frame", [sprite]) == 3
    assert call_builtin("sprite_is_finished", [sprite]) is True
    # Weitere Updates aendern frame nicht
    call_builtin("sprite_update", [sprite, 500])
    assert call_builtin("sprite_get_frame", [sprite]) == 3


def test_current_anim(call_builtin, sprite):
    call_builtin("sprite_add_anim", [sprite, "idle", 0, 0, 1.0])
    call_builtin("sprite_play", [sprite, "idle"])
    assert call_builtin("sprite_current_anim", [sprite]) == "idle"


def test_play_same_anim_is_idempotent(call_builtin, sprite):
    """Wiederholtes PLAY der gleichen Anim setzt nicht zurueck (Bug-Fall:
    laufen-laufen-laufen wuerde sonst staendig auf Frame 0 zuruckspringen)."""
    call_builtin("sprite_add_anim", [sprite, "walk", 0, 3, 8.0])
    call_builtin("sprite_play", [sprite, "walk"])
    call_builtin("sprite_update", [sprite, 250])  # frame 2
    f1 = call_builtin("sprite_get_frame", [sprite])
    call_builtin("sprite_play", [sprite, "walk"])  # erneut "walk" -> kein Reset
    f2 = call_builtin("sprite_get_frame", [sprite])
    assert f1 == f2


def test_set_frame(call_builtin, sprite):
    call_builtin("sprite_set_frame", [sprite, 7])
    assert call_builtin("sprite_get_frame", [sprite]) == 7


def test_add_anim_invalid_range_raises(call_builtin, sprite):
    with pytest.raises(GBRuntimeError, match="last >= first"):
        call_builtin("sprite_add_anim", [sprite, "bad", 5, 2, 8.0])


# --- Flip ------------------------------------------------------------

def test_set_flip_changes_state(call_builtin, sprite):
    call_builtin("sprite_set_flip", [sprite, True, False])
    assert sprite.flip_x is True
    assert sprite.flip_y is False


# --- Scale & Tint ----------------------------------------------------

def test_default_scale_is_one(sprite):
    assert sprite.scale_x == 1.0
    assert sprite.scale_y == 1.0


def test_set_scale(call_builtin, sprite):
    call_builtin("sprite_set_scale", [sprite, 2.0, 1.5])
    assert sprite.scale_x == 2.0
    assert sprite.scale_y == 1.5


def test_set_scale_negative_raises(call_builtin, sprite):
    with pytest.raises(GBRuntimeError, match="> 0"):
        call_builtin("sprite_set_scale", [sprite, -1.0, 1.0])


def test_default_no_tint(sprite):
    assert sprite.tinted is False


def test_tint_sets_state(call_builtin, sprite):
    call_builtin("sprite_tint", [sprite, 0xFF8040])
    assert sprite.tinted is True
    assert sprite.tint_color == 0xFF8040


def test_tint_clear(call_builtin, sprite):
    call_builtin("sprite_tint", [sprite, 0xFF0000])
    call_builtin("sprite_tint_clear", [sprite])
    assert sprite.tinted is False


def test_tint_out_of_range_raises(call_builtin, sprite):
    with pytest.raises(GBRuntimeError, match="0..0xFFFFFF"):
        call_builtin("sprite_tint", [sprite, 0x1000000])


def test_scale_does_not_affect_get_width(call_builtin, sprite):
    """Scale ist rein visuell - GET_WIDTH/HEIGHT bleiben in Frame-Groesse."""
    call_builtin("sprite_set_scale", [sprite, 3.0, 3.0])
    assert call_builtin("sprite_get_width", [sprite]) == 16
    assert call_builtin("sprite_get_height", [sprite]) == 16


# --- Kollision -------------------------------------------------------

def test_collides_overlap(call_builtin, sheet_image):
    a = call_builtin("sprite_new", [sheet_image, 16, 16])
    b = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [a, 0.0, 0.0])
    call_builtin("sprite_set_pos", [b, 8.0, 8.0])  # ueberlappt
    assert call_builtin("sprite_collides", [a, b]) is True


def test_collides_disjoint(call_builtin, sheet_image):
    a = call_builtin("sprite_new", [sheet_image, 16, 16])
    b = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [a, 0.0, 0.0])
    call_builtin("sprite_set_pos", [b, 100.0, 100.0])
    assert call_builtin("sprite_collides", [a, b]) is False


def test_collides_touching_does_not_count(call_builtin, sheet_image):
    a = call_builtin("sprite_new", [sheet_image, 16, 16])
    b = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [a, 0.0, 0.0])
    call_builtin("sprite_set_pos", [b, 16.0, 0.0])  # Kanten beruehren
    assert call_builtin("sprite_collides", [a, b]) is False


# --- SPRITE_COLLIDE Singular-Alias -----------------------------------

def test_collide_singular_alias(call_builtin, sheet_image):
    a = call_builtin("sprite_new", [sheet_image, 16, 16])
    b = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [a, 0.0, 0.0])
    call_builtin("sprite_set_pos", [b, 8.0, 8.0])
    assert call_builtin("sprite_collide", [a, b]) is True


# --- SPRITE_HIT_BOX --------------------------------------------------

def test_hit_box_overlap(call_builtin, sheet_image):
    sp = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [sp, 10.0, 10.0])  # bbox: 10..26
    # Box ueberschneidet sich
    assert call_builtin("sprite_hit_box", [sp, 20, 20, 10, 10]) is True


def test_hit_box_no_overlap(call_builtin, sheet_image):
    sp = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [sp, 0.0, 0.0])    # bbox: 0..16
    assert call_builtin("sprite_hit_box", [sp, 100, 100, 10, 10]) is False


def test_hit_box_contained(call_builtin, sheet_image):
    """Sprite vollstaendig in einer grossen Box -> Treffer."""
    sp = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [sp, 50.0, 50.0])
    assert call_builtin("sprite_hit_box", [sp, 0, 0, 200, 200]) is True


# --- SPRITE_HIT_POINT ------------------------------------------------

def test_hit_point_inside(call_builtin, sheet_image):
    sp = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [sp, 10.0, 10.0])  # bbox: 10..26
    assert call_builtin("sprite_hit_point", [sp, 15, 15]) is True


def test_hit_point_outside(call_builtin, sheet_image):
    sp = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [sp, 10.0, 10.0])
    assert call_builtin("sprite_hit_point", [sp, 100, 100]) is False


def test_hit_point_on_corner_inclusive_exclusive(call_builtin, sheet_image):
    """Linke obere Ecke ist inclusive (HIT), rechte untere exclusive (MISS)."""
    sp = call_builtin("sprite_new", [sheet_image, 16, 16])
    call_builtin("sprite_set_pos", [sp, 10.0, 10.0])
    assert call_builtin("sprite_hit_point", [sp, 10, 10]) is True
    assert call_builtin("sprite_hit_point", [sp, 26, 26]) is False  # x+w
