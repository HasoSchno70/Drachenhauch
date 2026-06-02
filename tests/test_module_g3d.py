"""Tests fuer das g3d-Modul (3D, native-only).

3D rendert nur in der nativen Runtime (gbrt). Im Python-Pfad muessen die
Builtins (a) registriert sein, damit der Compiler CALL_BUILTIN emittiert, und
(b) eine klare Meldung werfen statt zu craschen. Das Rendering selbst wird
nativ per Screenshot verifiziert (examples/82_3d_intro.gb), nicht hier.
"""
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("g3d")


def _gr(name, *args):
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    return GRAPHICS_BUILTINS[name.lower()](None, list(args))


_ALL = {
    "camera3d": 7, "cube": 7, "cube_wires": 7, "sphere": 5, "sphere_wires": 5,
    "cylinder": 7, "plane": 6, "line3d": 7, "point3d": 4, "grid3d": 2,
    # 3D-Modelle (geladen + prozedural)
    "loadmodel": 1, "mesh_cube": 3, "mesh_sphere": 3, "mesh_cylinder": 3,
    "mesh_torus": 4, "mesh_knot": 4, "mesh_plane": 4, "mesh_heightmap": 4,
    "model": 6, "model_ex": 10, "model_wires": 6, "model_texture": 2,
    "model_texture_normal": 2, "model_pbr": 3,
    # Billboards + Ray-Kollision/Picking
    "billboard": 6, "ray_hit_box": 12, "ray_hit_sphere": 10,
    "pick_box": 6, "pick_sphere": 4,
    # Kamera-Modi
    "camera3d_update": 1, "camera3d_x": 0, "camera3d_y": 0, "camera3d_z": 0,
    "camera3d_target_x": 0, "camera3d_target_y": 0, "camera3d_target_z": 0,
    # Beleuchtung + Fog + IBL
    "light_enable": 0, "light_ambient": 2, "light_fog": 2, "light_env": 3,
    "light_directional": 4, "light_point": 4,
    "light_set_pos": 4, "light_set_color": 2, "light_set_enabled": 2, "model_lit": 1,
    # Shadow-Mapping (SHADOW_ENABLE separat -- variable arity)
    "shadow_area": 2, "shadow_target": 3,
}


def test_all_builtins_registered():
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    for name in _ALL:
        assert name in GRAPHICS_BUILTINS, name


@pytest.mark.parametrize("name,arity", list(_ALL.items()))
def test_native_only_message(name, arity):
    with pytest.raises(GBRuntimeError, match="nativen Runtime"):
        _gr(name, *([1] * arity))


@pytest.mark.parametrize("name,arity", [(n, a) for n, a in _ALL.items() if a > 0])
def test_arity_checked_before_body(name, arity):
    # Zu wenige Argumente -> Arity-Fehler (nicht die native-only-Meldung).
    # (Nur fuer arity > 0 sinnvoll -- bei arity 0 gibt es kein "zu wenig".)
    with pytest.raises(GBRuntimeError) as ei:
        _gr(name, *([1] * (arity - 1)))
    assert "nativen Runtime" not in str(ei.value)


def test_shadow_enable_registered_and_native_only():
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    assert "shadow_enable" in GRAPHICS_BUILTINS
    # Variable arity (0,1): beide Aufruf-Formen sind gueltig -> native-only-Meldung.
    for args in ([], [2048]):
        with pytest.raises(GBRuntimeError, match="nativen Runtime"):
            _gr("shadow_enable", *args)
    # Zu viele Argumente -> Arity-Fehler (nicht native-only).
    with pytest.raises(GBRuntimeError) as ei:
        _gr("shadow_enable", 1, 2)
    assert "nativen Runtime" not in str(ei.value)
