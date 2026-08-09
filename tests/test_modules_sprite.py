"""Tests fuer das sprite-Modul (Sprite-Logik: Position/Velocity/Animation/Kollision).

Golden-Tests gegen `dhrt` (Stufe B): das Sprite braucht ein IMAGE-Handle -- wir
erzeugen es headless via GENTEX_COLOR (dhrt zieht dafuer ein lazy verstecktes
Fenster hoch). SPRITE_DRAW selbst wird nicht getestet (rein nativ). Tests, die
fueher interne Felder lasen (`sprite.flip_x`/`scale_x`/`tinted`/`tint_color`),
ohne GB-Getter, sind auf No-Crash-Smoke + Validierung reduziert. Frueher via
`call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""
import pytest

from gamebasic.errors import GBRuntimeError

# IMAGE headless via GENTEX_COLOR + ein 16x16-Sprite in 's'.
_PRE = ('IMPORT "sprite"\nDIM img AS IMAGE\nimg = GENTEX_COLOR(64, 64, RGB(255, 0, 0))\n'
        'DIM s AS SPRITE\ns = SPRITE_NEW(img, 16, 16)\n')
# Variante mit zwei Sprites a/b fuer Kollision.
_PRE2 = ('IMPORT "sprite"\nDIM img AS IMAGE\nimg = GENTEX_COLOR(64, 64, RGB(255, 0, 0))\n'
         'DIM a AS SPRITE\na = SPRITE_NEW(img, 16, 16)\n'
         'DIM b AS SPRITE\nb = SPRITE_NEW(img, 16, 16)\n')


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


# --- Konstruktion ----------------------------------------------------

def test_new_starts_at_origin(run_gb):
    assert _lines(run_gb(_PRE + "PRINT SPRITE_GET_X(s)\nPRINT SPRITE_GET_Y(s)\n")) == \
        ["0.0", "0.0"]


def test_get_size(run_gb):
    assert _lines(run_gb(_PRE + "PRINT SPRITE_GET_WIDTH(s)\nPRINT SPRITE_GET_HEIGHT(s)\n")) == \
        ["16", "16"]


def test_new_invalid_frame_size_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="> 0"):
        run_gb('IMPORT "sprite"\nDIM img AS IMAGE\nimg = GENTEX_COLOR(64, 64, RGB(1, 1, 1))\n'
               "DIM s AS SPRITE\ns = SPRITE_NEW(img, 0, 16)\n")


# --- Position & Velocity --------------------------------------------

def test_set_pos(run_gb):
    out = _lines(run_gb(_PRE + "SPRITE_SET_POS(s, 50.0, 75.0)\n"
                        "PRINT SPRITE_GET_X(s)\nPRINT SPRITE_GET_Y(s)\n"))
    assert out == ["50.0", "75.0"]


def test_velocity_moves_position(run_gb):
    out = _lines(run_gb(_PRE + "SPRITE_SET_POS(s, 0.0, 0.0)\n"
                        "SPRITE_SET_VELOCITY(s, 100.0, -50.0)\nSPRITE_UPDATE(s, 1000)\n"
                        "PRINT SPRITE_GET_X(s)\nPRINT SPRITE_GET_Y(s)\n"))
    assert out == ["100.0", "-50.0"]


# --- Animationen -----------------------------------------------------

def test_play_unknown_anim_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="unbekannte Animation"):
        run_gb(_PRE + 'SPRITE_PLAY(s, "fly")\n')


def test_walk_loops(run_gb):
    out = _lines(run_gb(_PRE +
                        'SPRITE_ADD_ANIM(s, "walk", 0, 3, 8.0)\nSPRITE_PLAY(s, "walk")\n'
                        "PRINT SPRITE_GET_FRAME(s)\n"
                        "SPRITE_UPDATE(s, 125)\nPRINT SPRITE_GET_FRAME(s)\n"
                        "SPRITE_UPDATE(s, 125)\nPRINT SPRITE_GET_FRAME(s)\n"
                        "SPRITE_UPDATE(s, 125)\nPRINT SPRITE_GET_FRAME(s)\n"
                        "SPRITE_UPDATE(s, 125)\nPRINT SPRITE_GET_FRAME(s)\n"))
    assert out == ["0", "1", "2", "3", "0"]


def test_play_once_stops_at_last_frame(run_gb):
    out = _lines(run_gb(_PRE +
                        'SPRITE_ADD_ANIM(s, "punch", 0, 3, 12.0)\nSPRITE_PLAY_ONCE(s, "punch")\n'
                        "PRINT SPRITE_IS_FINISHED(s)\n"
                        "SPRITE_UPDATE(s, 400)\n"
                        "PRINT SPRITE_GET_FRAME(s)\nPRINT SPRITE_IS_FINISHED(s)\n"
                        "SPRITE_UPDATE(s, 500)\nPRINT SPRITE_GET_FRAME(s)\n"))
    assert out == ["FALSE", "3", "TRUE", "3"]


def test_current_anim(run_gb):
    out = _lines(run_gb(_PRE +
                        'SPRITE_ADD_ANIM(s, "idle", 0, 0, 1.0)\nSPRITE_PLAY(s, "idle")\n'
                        "PRINT SPRITE_CURRENT_ANIM(s)\n"))
    assert out == ["idle"]


def test_play_same_anim_is_idempotent(run_gb):
    out = _lines(run_gb(_PRE +
                        'SPRITE_ADD_ANIM(s, "walk", 0, 3, 8.0)\nSPRITE_PLAY(s, "walk")\n'
                        "SPRITE_UPDATE(s, 250)\nPRINT SPRITE_GET_FRAME(s)\n"
                        'SPRITE_PLAY(s, "walk")\nPRINT SPRITE_GET_FRAME(s)\n'))
    assert out[0] == out[1]


def test_set_frame(run_gb):
    assert _lines(run_gb(_PRE + "SPRITE_SET_FRAME(s, 7)\nPRINT SPRITE_GET_FRAME(s)\n")) == ["7"]


def test_add_anim_invalid_range_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="last >= first"):
        run_gb(_PRE + 'SPRITE_ADD_ANIM(s, "bad", 5, 2, 8.0)\n')


# --- Flip / Scale / Tint (kein GB-Getter -> Smoke + Validierung) -----

def test_set_flip_no_crash(run_gb):
    assert _lines(run_gb(_PRE + 'SPRITE_SET_FLIP(s, TRUE, FALSE)\nPRINT "ok"\n')) == ["ok"]


def test_set_scale_no_crash(run_gb):
    assert _lines(run_gb(_PRE + 'SPRITE_SET_SCALE(s, 2.0, 1.5)\nPRINT "ok"\n')) == ["ok"]


def test_set_scale_negative_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="> 0"):
        run_gb(_PRE + "SPRITE_SET_SCALE(s, -1.0, 1.0)\n")


def test_tint_and_clear_no_crash(run_gb):
    assert _lines(run_gb(_PRE +
                  "SPRITE_TINT(s, 16744512)\nSPRITE_TINT_CLEAR(s)\nPRINT \"ok\"\n")) == ["ok"]


def test_tint_out_of_range_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="0..0xFFFFFF"):
        run_gb(_PRE + "SPRITE_TINT(s, 16777217)\n")


def test_scale_does_not_affect_get_width(run_gb):
    """Scale ist rein visuell - GET_WIDTH/HEIGHT bleiben in Frame-Groesse."""
    out = _lines(run_gb(_PRE + "SPRITE_SET_SCALE(s, 3.0, 3.0)\n"
                        "PRINT SPRITE_GET_WIDTH(s)\nPRINT SPRITE_GET_HEIGHT(s)\n"))
    assert out == ["16", "16"]


# --- Kollision -------------------------------------------------------

def test_collides_overlap(run_gb):
    out = _lines(run_gb(_PRE2 + "SPRITE_SET_POS(a, 0.0, 0.0)\nSPRITE_SET_POS(b, 8.0, 8.0)\n"
                        "PRINT SPRITE_COLLIDES(a, b)\n"))
    assert out == ["TRUE"]


def test_collides_disjoint(run_gb):
    out = _lines(run_gb(_PRE2 + "SPRITE_SET_POS(a, 0.0, 0.0)\nSPRITE_SET_POS(b, 100.0, 100.0)\n"
                        "PRINT SPRITE_COLLIDES(a, b)\n"))
    assert out == ["FALSE"]


def test_collides_touching_does_not_count(run_gb):
    out = _lines(run_gb(_PRE2 + "SPRITE_SET_POS(a, 0.0, 0.0)\nSPRITE_SET_POS(b, 16.0, 0.0)\n"
                        "PRINT SPRITE_COLLIDES(a, b)\n"))
    assert out == ["FALSE"]


def test_collide_singular_alias(run_gb):
    out = _lines(run_gb(_PRE2 + "SPRITE_SET_POS(a, 0.0, 0.0)\nSPRITE_SET_POS(b, 8.0, 8.0)\n"
                        "PRINT SPRITE_COLLIDE(a, b)\n"))
    assert out == ["TRUE"]


# --- SPRITE_HIT_BOX --------------------------------------------------

def test_hit_box_overlap(run_gb):
    out = _lines(run_gb(_PRE + "SPRITE_SET_POS(s, 10.0, 10.0)\n"
                        "PRINT SPRITE_HIT_BOX(s, 20, 20, 10, 10)\n"))
    assert out == ["TRUE"]


def test_hit_box_no_overlap(run_gb):
    out = _lines(run_gb(_PRE + "SPRITE_SET_POS(s, 0.0, 0.0)\n"
                        "PRINT SPRITE_HIT_BOX(s, 100, 100, 10, 10)\n"))
    assert out == ["FALSE"]


def test_hit_box_contained(run_gb):
    out = _lines(run_gb(_PRE + "SPRITE_SET_POS(s, 50.0, 50.0)\n"
                        "PRINT SPRITE_HIT_BOX(s, 0, 0, 200, 200)\n"))
    assert out == ["TRUE"]


# --- SPRITE_HIT_POINT ------------------------------------------------

def test_hit_point_inside(run_gb):
    out = _lines(run_gb(_PRE + "SPRITE_SET_POS(s, 10.0, 10.0)\n"
                        "PRINT SPRITE_HIT_POINT(s, 15, 15)\n"))
    assert out == ["TRUE"]


def test_hit_point_outside(run_gb):
    out = _lines(run_gb(_PRE + "SPRITE_SET_POS(s, 10.0, 10.0)\n"
                        "PRINT SPRITE_HIT_POINT(s, 100, 100)\n"))
    assert out == ["FALSE"]


def test_hit_point_on_corner_inclusive_exclusive(run_gb):
    """Linke obere Ecke inclusive (HIT), rechte untere exclusive (MISS)."""
    out = _lines(run_gb(_PRE + "SPRITE_SET_POS(s, 10.0, 10.0)\n"
                        "PRINT SPRITE_HIT_POINT(s, 10, 10)\n"
                        "PRINT SPRITE_HIT_POINT(s, 26, 26)\n"))
    assert out == ["TRUE", "FALSE"]
