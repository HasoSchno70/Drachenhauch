"""Tests fuer die Module-Reset-Funktion.

`reset_for_tests()` ist gedacht fuer Test-Suites, die Test-Isolation
zwischen Modul-Loads brauchen. Wir testen das Default-Verhalten ohne
`reload`-Flag -- die `reload=True`-Variante stoert die globale Modul-
Identitaet und ist nur fuer dedizierte Test-Suiten gedacht, die ihren
eigenen sys.modules-Snapshot fahren.

Snapshot/Restore: vor jedem Test machen wir einen Snapshot des globalen
State, nach dem Test stellen wir ihn wieder her. So koennen wir hier
`reset_for_tests()` ungestraft rufen, ohne andere Test-Files zu
beeintraechtigen.
"""
import pytest

from gamebasic.modules import (
    EXTERNAL_TYPES,
    OPERATORS_BY_TYPE,
    _loaded,
    load_module,
    reset_for_tests,
)


@pytest.fixture(autouse=True)
def _snapshot_global_state():
    """Vor jedem Test: globalen Module-State + die BUILTINS/GRAPHICS_BUILTINS-
    Dicts wegspeichern. Nach dem Test: restore. Wichtig: BUILTINS muss mit
    rein, sonst hinterlaesst `reset_for_tests(reload=True)` neue Wrapper-Refs
    (aus dem Re-Import) waehrend EXTERNAL_TYPES auf die alten Klassen restored
    wird -- inkonsistent, bricht nachfolgende Tests."""
    from gamebasic import builtins_registry as _br
    saved_loaded = set(_loaded)
    saved_types = dict(EXTERNAL_TYPES)
    saved_ops = dict(OPERATORS_BY_TYPE)
    saved_b = dict(_br.BUILTINS)
    saved_g = dict(_br.GRAPHICS_BUILTINS)
    yield
    _loaded.clear()
    _loaded.update(saved_loaded)
    EXTERNAL_TYPES.clear()
    EXTERNAL_TYPES.update(saved_types)
    OPERATORS_BY_TYPE.clear()
    OPERATORS_BY_TYPE.update(saved_ops)
    _br.BUILTINS.clear()
    _br.BUILTINS.update(saved_b)
    _br.GRAPHICS_BUILTINS.clear()
    _br.GRAPHICS_BUILTINS.update(saved_g)


def test_reset_clears_loaded_set():
    load_module("vec2")
    assert "vec2" in _loaded
    reset_for_tests()
    assert _loaded == set()


def test_reset_clears_external_types():
    # Voraussetzung: vec2 ist registriert. In der gesamten Suite ist das
    # ueblicherweise der Fall (autouse-Fixtures laden Module). Im
    # Solo-Run muessten wir's explizit machen, aber wir testen hier nur
    # dass die `EXTERNAL_TYPES.clear()`-Aktion wirkt -- die Anwesenheit
    # eines beliebigen Eintrags reicht.
    EXTERNAL_TYPES["__test_marker__"] = object
    reset_for_tests()
    assert "__test_marker__" not in EXTERNAL_TYPES


def test_reset_clears_operators():
    OPERATORS_BY_TYPE[object] = {"+": lambda a, b: None}
    reset_for_tests()
    assert OPERATORS_BY_TYPE == {}


def test_reset_default_is_idempotent_load():
    """`reset_for_tests()` ohne `reload` loescht nur die Dicts. Ein
    erneutes `load_module(...)` traegt den Namen wieder in `_loaded`
    ein, aber sys.modules cached das Modul -- die Decorators feuern
    NICHT erneut. Damit bleiben EXTERNAL_TYPES leer fuer dieses Modul.

    Das ist by-design: `reload=True` ist die Variante mit Side-Effekten
    auf andere Tests; default ist sicher."""
    reset_for_tests()
    assert _loaded == set()
    load_module("vec2")
    assert "vec2" in _loaded  # _loaded.add wurde aufgerufen
