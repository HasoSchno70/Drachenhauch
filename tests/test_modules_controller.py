"""Tests fuer controller-Modul (Platformer-Character-Controller).

Golden-Tests gegen `dhrt` (Stufe B): Tiled-Map-Fixture in `tmp_path`, GB-Programm
treibt den Controller ueber CHAR_SET_INPUT/CHAR_UPDATE-Frames und PRINTet die
Zustaende. Frueher via `call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""
import json
import pytest

from gamebasic.errors import GBRuntimeError

_HEAD = ('IMPORT "tiled"\nIMPORT "tile_collide"\nIMPORT "controller"\n'
         'DIM m AS TILED_MAP\nm = TILED_LOAD("level.json")\n')


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _write_map(tmp_path, tile_data, width=10, height=8):
    spec = {
        "type": "map", "width": width, "height": height,
        "tilewidth": 16, "tileheight": 16,
        "tilesets": [{"firstgid": 1, "name": "t", "tilewidth": 16, "tileheight": 16,
                      "columns": 1, "image": "x.png", "imagewidth": 16, "imageheight": 16,
                      "tiles": [{"id": 0, "properties": [
                          {"name": "solid", "type": "bool", "value": True}]}]}],
        "layers": [{"type": "tilelayer", "name": "ground", "width": width,
                    "height": height, "data": tile_data}],
    }
    (tmp_path / "level.json").write_text(json.dumps(spec), encoding="utf-8")


def _floor(tmp_path, width=10, height=8):
    data = [0] * (width * height)
    for x in range(width):
        data[(height - 1) * width + x] = 1
    _write_map(tmp_path, data, width, height)


def _empty(tmp_path):
    _write_map(tmp_path, [0] * 80)


def _leftwall(tmp_path):
    data = [0] * 80
    for y in range(8):
        data[y * 10] = 1
    _write_map(tmp_path, data)


def _run(run_gb, tmp_path, body):
    return _lines(run_gb(_HEAD + body, base=tmp_path))


# --- Konstruktor + Accessoren -------------------------------------

def test_char_new_initializes(run_gb, tmp_path):
    _empty(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(10.0, 20.0, 12.0, 14.0)\n"
               "PRINT CHAR_X(c)\nPRINT CHAR_Y(c)\nPRINT CHAR_W(c)\nPRINT CHAR_H(c)\n"
               "PRINT CHAR_VX(c)\nPRINT CHAR_VY(c)\n"
               "PRINT CHAR_ON_GROUND(c)\nPRINT CHAR_FACING(c)\n")
    assert out == ["10.0", "20.0", "12.0", "14.0", "0.0", "0.0", "FALSE", "1"]


def test_char_new_zero_size_errors(run_gb, tmp_path):
    _empty(tmp_path)
    with pytest.raises(GBRuntimeError, match="> 0"):
        _run(run_gb, tmp_path,
             "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(0.0, 0.0, 0.0, 10.0)\n")


def test_char_set_pos(run_gb, tmp_path):
    _empty(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(0.0, 0.0, 8.0, 8.0)\n"
               "CHAR_SET_POS(c, 100.0, 200.0)\nPRINT CHAR_X(c)\nPRINT CHAR_Y(c)\n")
    assert out == ["100.0", "200.0"]


# --- Bewegung -----------------------------------------------------

def test_horizontal_move(run_gb, tmp_path):
    _floor(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(16.0, 96.0, 8.0, 8.0)\n"
               "DIM i AS INTEGER\nFOR i = 1 TO 3\n"
               "    CHAR_SET_INPUT(c, 1, FALSE, FALSE)\n    CHAR_UPDATE(c, m, 0)\nNEXT\n"
               "PRINT CHAR_X(c)\nPRINT CHAR_FACING(c)\n")
    assert float(out[0]) > 20.0
    assert out[1] == "1"


def test_facing_changes_with_input(run_gb, tmp_path):
    _floor(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 96.0, 8.0, 8.0)\n"
               "CHAR_SET_INPUT(c, -1, FALSE, FALSE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_FACING(c)\n")
    assert out == ["-1"]


def test_gravity_pulls_down(run_gb, tmp_path):
    _empty(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 10.0, 8.0, 8.0)\n"
               "PRINT CHAR_Y(c)\n"
               "DIM i AS INTEGER\nFOR i = 1 TO 3\n"
               "    CHAR_SET_INPUT(c, 0, FALSE, FALSE)\n    CHAR_UPDATE(c, m, 0)\nNEXT\n"
               "PRINT CHAR_Y(c)\n")
    assert float(out[1]) > float(out[0])


def test_lands_on_ground(run_gb, tmp_path):
    _floor(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 10.0, 8.0, 8.0)\n"
               "DIM i AS INTEGER\nFOR i = 1 TO 60\n"
               "    CHAR_SET_INPUT(c, 0, FALSE, FALSE)\n    CHAR_UPDATE(c, m, 0)\nNEXT\n"
               "PRINT CHAR_ON_GROUND(c)\nPRINT CHAR_Y(c)\n")
    assert out == ["TRUE", "104.0"]


# --- Sprung -------------------------------------------------------

def test_jump_from_ground(run_gb, tmp_path):
    _floor(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 104.0, 8.0, 8.0)\n"
               "CHAR_SET_INPUT(c, 0, FALSE, FALSE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_ON_GROUND(c)\n"
               "CHAR_SET_INPUT(c, 0, TRUE, TRUE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_VY(c)\nPRINT CHAR_ON_GROUND(c)\n")
    assert out[0] == "TRUE"
    assert float(out[1]) < 0.0
    assert out[2] == "FALSE"


def test_jump_no_ground_no_action(run_gb, tmp_path):
    _empty(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 10.0, 8.0, 8.0)\n"
               "DIM i AS INTEGER\nFOR i = 1 TO 20\n"
               "    CHAR_SET_INPUT(c, 0, FALSE, FALSE)\n    CHAR_UPDATE(c, m, 0)\nNEXT\n"
               "PRINT CHAR_VY(c)\n"
               "CHAR_SET_INPUT(c, 0, TRUE, TRUE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_VY(c)\n")
    assert float(out[1]) >= float(out[0])


# --- Coyote-Time --------------------------------------------------

def test_coyote_time_allows_jump_after_ledge(run_gb, tmp_path):
    _floor(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 104.0, 8.0, 8.0)\n"
               "CHAR_SET_INPUT(c, 0, FALSE, FALSE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_ON_GROUND(c)\n"
               "CHAR_SET_POS(c, 50.0, 50.0)\n"
               "DIM i AS INTEGER\nFOR i = 1 TO 2\n"
               "    CHAR_SET_INPUT(c, 0, FALSE, FALSE)\n    CHAR_UPDATE(c, m, 0)\nNEXT\n"
               "CHAR_SET_INPUT(c, 0, TRUE, TRUE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_VY(c)\n")
    assert out[0] == "TRUE"
    assert float(out[1]) < 0.0


def test_coyote_time_expires(run_gb, tmp_path):
    _floor(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 104.0, 8.0, 8.0)\n"
               "CHAR_SET_INPUT(c, 0, FALSE, FALSE)\nCHAR_UPDATE(c, m, 0)\n"
               "CHAR_SET_POS(c, 50.0, 30.0)\n"
               "DIM i AS INTEGER\nFOR i = 1 TO 10\n"
               "    CHAR_SET_INPUT(c, 0, FALSE, FALSE)\n    CHAR_UPDATE(c, m, 0)\nNEXT\n"
               "PRINT CHAR_VY(c)\n"
               "CHAR_SET_INPUT(c, 0, TRUE, TRUE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_VY(c)\n")
    assert float(out[1]) >= float(out[0]) - 0.1


# --- Jump-Buffer --------------------------------------------------

def test_jump_buffer_fires_on_landing(run_gb, tmp_path):
    """Sprung-Press kurz vor der Landung feuert beim Touchdown (vy < 0).

    Startet naeher am Boden als frueher (102.0 statt 98.0): Review-Fund-Fix
    in controller.rs machte das Jump-Buffer-Fenster exakt `jump_buffer_max`
    (6) Frames lang statt vorher effektiv 7 (Decrement lief am Frame-ENDE
    und wurde ausgerechnet auf dem Press-Frame uebersprungen -- 1 Frame
    laenger als der strukturell identische Coyote-Counter). Bei y=98.0 landet
    der Fall exakt auf dem 6. Frame -- also GENAU auf der (jetzt korrekten)
    Fenstergrenze, nicht mit Sicherheitsabstand. y=102.0 laesst den Fall
    bequem innerhalb des Fensters landen und testet damit wieder das
    eigentlich gemeinte Verhalten ("kurz vor der Landung"), statt zufaellig
    exakt an dessen Rand zu haengen.
    """
    _floor(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 102.0, 8.0, 8.0)\n"
               "CHAR_SET_INPUT(c, 0, TRUE, TRUE)\nCHAR_UPDATE(c, m, 0)\n"
               "DIM best AS FLOAT\nbest = CHAR_VY(c)\n"
               "DIM i AS INTEGER\nFOR i = 1 TO 10\n"
               "    CHAR_SET_INPUT(c, 0, FALSE, TRUE)\n    CHAR_UPDATE(c, m, 0)\n"
               "    IF CHAR_VY(c) < best THEN best = CHAR_VY(c)\nNEXT\n"
               "PRINT best\n")
    assert float(out[0]) < 0.0


# --- Variable-Jump-Height ----------------------------------------

def test_variable_jump_cuts_velocity_on_release(run_gb, tmp_path):
    _floor(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 104.0, 8.0, 8.0)\n"
               "CHAR_SET_INPUT(c, 0, FALSE, FALSE)\nCHAR_UPDATE(c, m, 0)\n"
               "CHAR_SET_INPUT(c, 0, TRUE, TRUE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_VY(c)\n"
               "CHAR_SET_INPUT(c, 0, FALSE, FALSE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_VY(c)\n")
    vy_jump = float(out[0])
    vy_release = float(out[1])
    assert vy_jump < 0.0
    assert abs(vy_release - (vy_jump * 0.5 + 0.25)) < 0.01


def test_variable_jump_no_effect_when_disabled(run_gb, tmp_path):
    _floor(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(50.0, 104.0, 8.0, 8.0)\n"
               "CHAR_SET_VARIABLE_JUMP(c, FALSE)\n"
               "CHAR_SET_INPUT(c, 0, FALSE, FALSE)\nCHAR_UPDATE(c, m, 0)\n"
               "CHAR_SET_INPUT(c, 0, TRUE, TRUE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_VY(c)\n"
               "CHAR_SET_INPUT(c, 0, FALSE, FALSE)\nCHAR_UPDATE(c, m, 0)\n"
               "PRINT CHAR_VY(c)\n")
    vy_jump = float(out[0])
    assert abs(float(out[1]) - (vy_jump + 0.25)) < 0.01


# --- Konfiguration -----------------------------------------------

def test_set_jump_velocity_takes_absolute_value(run_gb, tmp_path):
    """JUMP_VELOCITY-Setter darf nicht crashen (negativer Eingabewert)."""
    _empty(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(0.0, 0.0, 8.0, 8.0)\n"
               "CHAR_SET_JUMP_VELOCITY(c, -8.0)\nPRINT \"ok\"\n")
    assert out == ["ok"]


def test_set_move_speed_zero(run_gb, tmp_path):
    _empty(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(0.0, 0.0, 8.0, 8.0)\n"
               "CHAR_SET_MOVE_SPEED(c, 0.0)\nPRINT \"ok\"\n")
    assert out == ["ok"]


# --- Wand-Detection ----------------------------------------------

def test_on_wall_left(run_gb, tmp_path):
    _leftwall(tmp_path)
    out = _run(run_gb, tmp_path,
               "DIM c AS CHAR_CONTROLLER\nc = CHAR_NEW(16.0, 10.0, 8.0, 8.0)\n"
               "DIM i AS INTEGER\nFOR i = 1 TO 3\n"
               "    CHAR_SET_INPUT(c, -1, FALSE, FALSE)\n    CHAR_UPDATE(c, m, 0)\nNEXT\n"
               "PRINT CHAR_ON_WALL_LEFT(c)\n")
    assert out == ["TRUE"]


# --- Type-Validierung --------------------------------------------

def test_char_x_wrong_type(run_gb):
    with pytest.raises(GBRuntimeError, match="CHAR_CONTROLLER"):
        run_gb('IMPORT "controller"\nPRINT CHAR_X("not a controller")\n')
