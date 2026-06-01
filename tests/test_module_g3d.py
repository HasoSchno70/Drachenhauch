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
}


def test_all_builtins_registered():
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    for name in _ALL:
        assert name in GRAPHICS_BUILTINS, name


@pytest.mark.parametrize("name,arity", list(_ALL.items()))
def test_native_only_message(name, arity):
    with pytest.raises(GBRuntimeError, match="nativen Runtime"):
        _gr(name, *([1] * arity))


@pytest.mark.parametrize("name,arity", list(_ALL.items()))
def test_arity_checked_before_body(name, arity):
    # Zu wenige Argumente -> Arity-Fehler (nicht die native-only-Meldung).
    with pytest.raises(GBRuntimeError) as ei:
        _gr(name, *([1] * (arity - 1)))
    assert "nativen Runtime" not in str(ei.value)
