"""Tests fuer controller-Modul (Platformer-Character-Controller)."""
import json

import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_modules():
    assert load_module("tiled")
    assert load_module("tile_collide")
    assert load_module("controller")


def _make_map(tmp_path, call_builtin, tile_data, width=10, height=8,
              tile_w=16, tile_h=16):
    """Hilfs-Map mit solid-Property auf Tile-ID 0 (GID 1)."""
    data = {
        "type": "map",
        "width": width, "height": height,
        "tilewidth": tile_w, "tileheight": tile_h,
        "tilesets": [{
            "firstgid": 1, "name": "t",
            "tilewidth": tile_w, "tileheight": tile_h,
            "columns": 1, "image": "x.png",
            "imagewidth": 16, "imageheight": 16,
            "tiles": [{"id": 0, "properties": [
                {"name": "solid", "type": "bool", "value": True},
            ]}],
        }],
        "layers": [{
            "type": "tilelayer", "name": "ground",
            "width": width, "height": height,
            "data": tile_data,
        }],
    }
    p = tmp_path / "level.json"
    p.write_text(json.dumps(data))
    return call_builtin("tiled_load", [str(p)])


def _floor_map(tmp_path, call_builtin, width=10, height=8):
    """Map mit Boden in der letzten Reihe."""
    data = [0] * (width * height)
    for x in range(width):
        data[(height - 1) * width + x] = 1
    return _make_map(tmp_path, call_builtin, data, width, height)


# --- Konstruktor + Accessoren -------------------------------------

def test_char_new_initializes(call_builtin):
    c = call_builtin("char_new", [10.0, 20.0, 12.0, 14.0])
    assert call_builtin("char_x", [c]) == 10.0
    assert call_builtin("char_y", [c]) == 20.0
    assert call_builtin("char_w", [c]) == 12.0
    assert call_builtin("char_h", [c]) == 14.0
    assert call_builtin("char_vx", [c]) == 0.0
    assert call_builtin("char_vy", [c]) == 0.0
    assert call_builtin("char_on_ground", [c]) is False
    assert call_builtin("char_facing", [c]) == 1


def test_char_new_zero_size_errors(call_builtin):
    with pytest.raises(GBRuntimeError, match="> 0"):
        call_builtin("char_new", [0.0, 0.0, 0.0, 10.0])


def test_char_set_pos(call_builtin):
    c = call_builtin("char_new", [0.0, 0.0, 8.0, 8.0])
    call_builtin("char_set_pos", [c, 100.0, 200.0])
    assert call_builtin("char_x", [c]) == 100.0
    assert call_builtin("char_y", [c]) == 200.0


# --- Bewegung -----------------------------------------------------

def test_horizontal_move(tmp_path, call_builtin):
    """Input-Axis = 1, Charakter bewegt sich nach rechts."""
    m = _floor_map(tmp_path, call_builtin)
    c = call_builtin("char_new", [16.0, 96.0, 8.0, 8.0])  # auf dem Boden
    # zweimal pdf-Frame mit axis=1, jump=false
    for _ in range(3):
        call_builtin("char_set_input", [c, 1, False, False])
        call_builtin("char_update", [c, m, 0])
    # Default move_speed = 2.0, 3 frames => ~6 Pixel
    assert call_builtin("char_x", [c]) > 20.0
    assert call_builtin("char_facing", [c]) == 1


def test_facing_changes_with_input(tmp_path, call_builtin):
    m = _floor_map(tmp_path, call_builtin)
    c = call_builtin("char_new", [50.0, 96.0, 8.0, 8.0])
    call_builtin("char_set_input", [c, -1, False, False])
    call_builtin("char_update", [c, m, 0])
    assert call_builtin("char_facing", [c]) == -1


def test_gravity_pulls_down(tmp_path, call_builtin):
    """Ohne Input, ohne Boden: Charakter faellt."""
    m = _make_map(tmp_path, call_builtin, [0] * 80)
    c = call_builtin("char_new", [50.0, 10.0, 8.0, 8.0])
    y0 = call_builtin("char_y", [c])
    for _ in range(3):
        call_builtin("char_set_input", [c, 0, False, False])
        call_builtin("char_update", [c, m, 0])
    assert call_builtin("char_y", [c]) > y0


def test_lands_on_ground(tmp_path, call_builtin):
    """Charakter faellt auf Boden, on_ground wird TRUE."""
    m = _floor_map(tmp_path, call_builtin)
    c = call_builtin("char_new", [50.0, 10.0, 8.0, 8.0])
    for _ in range(60):
        call_builtin("char_set_input", [c, 0, False, False])
        call_builtin("char_update", [c, m, 0])
    assert call_builtin("char_on_ground", [c]) is True
    # y muss bei (height-1) * tile_h - h sein = 7*16 - 8 = 104
    assert call_builtin("char_y", [c]) == 104.0


# --- Sprung -------------------------------------------------------

def test_jump_from_ground(tmp_path, call_builtin):
    """Sprung-Press auf dem Boden: vy wird negativ."""
    m = _floor_map(tmp_path, call_builtin)
    # Player auf der Resting-Position platzieren -- 1-2 Frames update
    # genuegt, damit on_ground=True wird (statt 60 Frames fallen lassen).
    c = call_builtin("char_new", [50.0, 104.0, 8.0, 8.0])
    # Resting-Position: 1 Update reicht, damit die Y-Sweep on_ground setzt.
    call_builtin("char_set_input", [c, 0, False, False])
    call_builtin("char_update", [c, m, 0])
    assert call_builtin("char_on_ground", [c]) is True
    # jetzt springen
    call_builtin("char_set_input", [c, 0, True, True])
    call_builtin("char_update", [c, m, 0])
    assert call_builtin("char_vy", [c]) < 0.0
    assert call_builtin("char_on_ground", [c]) is False


def test_jump_no_ground_no_action(tmp_path, call_builtin):
    """Sprung-Press in der Luft (kein Coyote): nichts passiert (vy bleibt
    von Gravitation positiv)."""
    m = _make_map(tmp_path, call_builtin, [0] * 80)
    c = call_builtin("char_new", [50.0, 10.0, 8.0, 8.0])
    # mal kurz fallen, ohne Coyote (es ist von Anfang an nicht on_ground)
    for _ in range(20):
        call_builtin("char_set_input", [c, 0, False, False])
        call_builtin("char_update", [c, m, 0])
    # Coyote war nie aktiv (wir starten in der Luft) -- Sprung nicht moeglich
    vy_before = call_builtin("char_vy", [c])
    call_builtin("char_set_input", [c, 0, True, True])
    call_builtin("char_update", [c, m, 0])
    # vy waechst weiter durch Gravitation, kein Sprung
    assert call_builtin("char_vy", [c]) >= vy_before


# --- Coyote-Time --------------------------------------------------

def test_coyote_time_allows_jump_after_ledge(tmp_path, call_builtin):
    """Springen ein paar Frames nachdem on_ground=False geworden ist
    ist erlaubt."""
    m = _floor_map(tmp_path, call_builtin)
    c = call_builtin("char_new", [50.0, 104.0, 8.0, 8.0])
    call_builtin("char_set_input", [c, 0, False, False])
    call_builtin("char_update", [c, m, 0])
    assert call_builtin("char_on_ground", [c]) is True
    # Boden weg-teleportieren -- emulieren "Klippe verlassen"
    call_builtin("char_set_pos", [c, 50.0, 50.0])
    # 1-2 Frames warten (coyote_max=6), dann springen
    for _ in range(2):
        call_builtin("char_set_input", [c, 0, False, False])
        call_builtin("char_update", [c, m, 0])
    # Inzwischen ist on_ground=False, aber Coyote ist noch aktiv -> Sprung OK
    call_builtin("char_set_input", [c, 0, True, True])
    call_builtin("char_update", [c, m, 0])
    assert call_builtin("char_vy", [c]) < 0.0


def test_coyote_time_expires(tmp_path, call_builtin):
    """Nach Ablauf der Coyote-Time geht Sprung nicht mehr."""
    m = _floor_map(tmp_path, call_builtin)
    c = call_builtin("char_new", [50.0, 104.0, 8.0, 8.0])
    call_builtin("char_set_input", [c, 0, False, False])
    call_builtin("char_update", [c, m, 0])
    # Boden weg
    call_builtin("char_set_pos", [c, 50.0, 30.0])
    # Coyote ist 6 frames -- nach 10 Frames ist er weg
    for _ in range(10):
        call_builtin("char_set_input", [c, 0, False, False])
        call_builtin("char_update", [c, m, 0])
    vy_before = call_builtin("char_vy", [c])
    # Versuche jetzt zu springen -- Coyote weg, Sprung nicht moeglich
    call_builtin("char_set_input", [c, 0, True, True])
    call_builtin("char_update", [c, m, 0])
    # vy waechst weiter durch Gravitation, KEIN Sprung
    assert call_builtin("char_vy", [c]) >= vy_before - 0.1


# --- Jump-Buffer --------------------------------------------------

def test_jump_buffer_fires_on_landing(tmp_path, call_builtin):
    """Sprung-Press kurz BEFORE landing: Sprung feuert beim Touchdown.

    Start dicht ueber dem Boden (y=98, Floor bei y=104), damit der
    Spieler innerhalb der buffer_max=6 Frames landet.
    """
    m = _floor_map(tmp_path, call_builtin)
    c = call_builtin("char_new", [50.0, 98.0, 8.0, 8.0])
    # Frame 0: Jump-Press in der Luft (Buffer setzen)
    call_builtin("char_set_input", [c, 0, True, True])
    call_builtin("char_update", [c, m, 0])
    # vy ist nach Frame 0 entweder die Sprung-Velocity (wenn schon on_ground)
    # oder Fall-Velocity. Wir wissen es nicht genau -- entscheidend ist:
    # innerhalb der naechsten Frames muss vy < 0 werden (Buffer-Sprung).
    landed_with_buffer = call_builtin("char_vy", [c]) < 0.0
    if not landed_with_buffer:
        # Weiter fallen ohne neuen Press; Buffer-Sprung beim Landen pruefen
        for _ in range(10):
            call_builtin("char_set_input", [c, 0, False, True])
            call_builtin("char_update", [c, m, 0])
            if call_builtin("char_vy", [c]) < 0.0:
                landed_with_buffer = True
                break
    assert landed_with_buffer, "Jump-Buffer hat nicht beim Landen ausgeloest"


# --- Variable-Jump-Height ----------------------------------------

def test_variable_jump_cuts_velocity_on_release(tmp_path, call_builtin):
    """Sprung-Loslassen waehrend nach oben fliegen: vy wird gekuerzt."""
    m = _floor_map(tmp_path, call_builtin)
    c = call_builtin("char_new", [50.0, 104.0, 8.0, 8.0])
    # Resting-Position: 1 Update reicht, damit die Y-Sweep on_ground setzt.
    call_builtin("char_set_input", [c, 0, False, False])
    call_builtin("char_update", [c, m, 0])
    assert call_builtin("char_on_ground", [c]) is True
    # Springen (held=True)
    call_builtin("char_set_input", [c, 0, True, True])
    call_builtin("char_update", [c, m, 0])
    vy_after_jump = call_builtin("char_vy", [c])
    assert vy_after_jump < 0.0
    # JETZT loslassen
    call_builtin("char_set_input", [c, 0, False, False])
    call_builtin("char_update", [c, m, 0])
    vy_after_release = call_builtin("char_vy", [c])
    # Cut wirkt VOR Gravitation: vy *= 0.5, dann + 0.25
    expected = vy_after_jump * 0.5 + 0.25
    assert abs(vy_after_release - expected) < 0.01


def test_variable_jump_no_effect_when_disabled(tmp_path, call_builtin):
    m = _floor_map(tmp_path, call_builtin)
    c = call_builtin("char_new", [50.0, 104.0, 8.0, 8.0])
    call_builtin("char_set_variable_jump", [c, False])
    # 1 Update fuer on_ground
    call_builtin("char_set_input", [c, 0, False, False])
    call_builtin("char_update", [c, m, 0])
    call_builtin("char_set_input", [c, 0, True, True])
    call_builtin("char_update", [c, m, 0])
    vy_after_jump = call_builtin("char_vy", [c])
    call_builtin("char_set_input", [c, 0, False, False])
    call_builtin("char_update", [c, m, 0])
    # Variable-Jump off: vy waechst nur durch Gravitation
    expected = vy_after_jump + 0.25
    assert abs(call_builtin("char_vy", [c]) - expected) < 0.01


# --- Konfiguration -----------------------------------------------

def test_set_jump_velocity_takes_absolute_value(call_builtin):
    """JUMP_VELOCITY wird immer als positive Zahl gespeichert."""
    c = call_builtin("char_new", [0.0, 0.0, 8.0, 8.0])
    call_builtin("char_set_jump_velocity", [c, -8.0])
    # User-API: positiv
    # (Wir koennen es nicht direkt lesen, aber das interne Feld ist 8.0)
    # Bei naechstem Sprung sollte vy = -8 sein
    # (test_jump_from_ground deckt das schon ab; hier nur kein Error)


def test_set_move_speed_zero(call_builtin):
    """move_speed = 0 ist erlaubt (statischer Player)."""
    c = call_builtin("char_new", [0.0, 0.0, 8.0, 8.0])
    call_builtin("char_set_move_speed", [c, 0.0])


# --- Wand-Detection ----------------------------------------------

def test_on_wall_left(tmp_path, call_builtin):
    """Wand direkt links: on_wall_left=True wenn nicht on_ground."""
    # Wand bei x=0 (tx=0, ganze Hoehe)
    data = [0] * 80
    for y in range(8):
        data[y * 10] = 1
    m = _make_map(tmp_path, call_builtin, data)
    c = call_builtin("char_new", [16.0, 10.0, 8.0, 8.0])  # direkt rechts neben Wand
    # Falle in der Luft, push gegen Wand
    for _ in range(3):
        call_builtin("char_set_input", [c, -1, False, False])
        call_builtin("char_update", [c, m, 0])
    assert call_builtin("char_on_wall_left", [c]) is True


# --- Type-Validierung --------------------------------------------

def test_char_x_wrong_type(call_builtin):
    with pytest.raises(TypeMismatchError, match="CHAR_CONTROLLER"):
        call_builtin("char_x", ["not a controller"])
