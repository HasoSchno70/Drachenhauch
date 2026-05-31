"""Tests fuer das bt-Modul (bleak/BLE-Wrapper).

BLE-Operationen brauchen ein echtes BT-Adapter + Geraet in der Naehe -
das simulieren wir hier nicht. Geprueft werden Registrierung,
Type-Checks und der Fehlerpfad bei fehlender Lib.
"""
import pytest

from gamebasic.modules import load_module, EXTERNAL_TYPES
from gamebasic.modules import bt as bt_mod
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("bt")


@pytest.fixture
def lib_missing(monkeypatch):
    monkeypatch.setattr(bt_mod, "_AVAILABLE", False)


# --- Registrierung ---------------------------------------------------

def test_module_registers_external_type():
    assert "bt_handle" in EXTERNAL_TYPES


def test_all_builtins_registered():
    from gamebasic.interpreter import BUILTINS
    expected = {
        "bt_scan", "bt_connect", "bt_disconnect", "bt_is_connected",
        "bt_services", "bt_characteristics", "bt_read", "bt_write",
    }
    assert expected <= set(BUILTINS.keys())


# --- Fehlende Lib ---------------------------------------------------

def test_scan_without_lib(lib_missing, call_builtin):
    with pytest.raises(GBRuntimeError, match=r"bleak.*installiert"):
        call_builtin("bt_scan", [1.0])


def test_connect_without_lib(lib_missing, call_builtin):
    with pytest.raises(GBRuntimeError, match=r"bleak.*installiert"):
        call_builtin("bt_connect", ["AA:BB:CC:DD:EE:FF"])


# --- Type-Checks ----------------------------------------------------

def test_disconnect_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="BT_HANDLE"):
        call_builtin("bt_disconnect", ["x"])


def test_is_connected_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="BT_HANDLE"):
        call_builtin("bt_is_connected", [42])


def test_services_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="BT_HANDLE"):
        call_builtin("bt_services", [None])


def test_characteristics_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="BT_HANDLE"):
        call_builtin("bt_characteristics", ["x", "00001800-0000-1000-8000-00805f9b34fb"])


def test_read_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="BT_HANDLE"):
        call_builtin("bt_read", ["x", "00002a00-0000-1000-8000-00805f9b34fb"])


def test_write_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="BT_HANDLE"):
        call_builtin("bt_write", ["x", "00002a00-0000-1000-8000-00805f9b34fb", "data"])


@pytest.mark.skipif(not bt_mod._AVAILABLE, reason="bleak nicht installiert")
def test_scan_negative_timeout_with_lib(call_builtin):
    with pytest.raises(GBRuntimeError, match=">= 0"):
        call_builtin("bt_scan", [-1.0])
