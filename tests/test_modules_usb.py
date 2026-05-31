"""Tests fuer das usb-Modul (hidapi-Wrapper).

Wie bei serial: ohne echtes HID-Geraet decken wir Registrierung,
Type-Checks und Lib-fehlt-Pfad ab.
"""
import pytest

from gamebasic.modules import load_module, EXTERNAL_TYPES
from gamebasic.modules import usb as usb_mod
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("usb")


@pytest.fixture
def lib_missing(monkeypatch):
    monkeypatch.setattr(usb_mod, "_AVAILABLE", False)


# --- Registrierung ---------------------------------------------------

def test_module_registers_external_type():
    assert "usb_handle" in EXTERNAL_TYPES


def test_all_builtins_registered():
    from gamebasic.interpreter import BUILTINS
    expected = {
        "usb_list", "usb_open", "usb_open_path", "usb_close",
        "usb_write", "usb_read",
        "usb_product", "usb_manufacturer", "usb_serial",
    }
    assert expected <= set(BUILTINS.keys())


# --- Fehlende Lib ---------------------------------------------------

def test_list_without_lib(lib_missing, call_builtin):
    with pytest.raises(GBRuntimeError, match=r"hidapi.*installiert"):
        call_builtin("usb_list", [])


def test_open_without_lib(lib_missing, call_builtin):
    with pytest.raises(GBRuntimeError, match=r"hidapi.*installiert"):
        call_builtin("usb_open", [0x046D, 0xC52B])


def test_open_path_without_lib(lib_missing, call_builtin):
    with pytest.raises(GBRuntimeError, match=r"hidapi.*installiert"):
        call_builtin("usb_open_path", ["/dev/whatever"])


# --- Type-Checks ----------------------------------------------------

def test_close_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="USB_HANDLE"):
        call_builtin("usb_close", ["nope"])


def test_write_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="USB_HANDLE"):
        call_builtin("usb_write", ["x", "data"])


def test_read_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="USB_HANDLE"):
        call_builtin("usb_read", ["x", 64, 100])


def test_product_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="USB_HANDLE"):
        call_builtin("usb_product", [123])


def test_manufacturer_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="USB_HANDLE"):
        call_builtin("usb_manufacturer", [None])


def test_serial_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="USB_HANDLE"):
        call_builtin("usb_serial", [object()])


def test_open_args_must_be_int(call_builtin):
    with pytest.raises(TypeMismatchError):
        call_builtin("usb_open", ["nicht hex", 0x1234])


# --- USB_WRITE: Bytewert-Range-Check (laeuft ohne hidapi nicht ganz, da
#     _ensure() vorne ist - also nur mit Lib testen).

@pytest.mark.skipif(not usb_mod._AVAILABLE, reason="hidapi nicht installiert")
def test_list_returns_string_when_lib_present(call_builtin):
    out = call_builtin("usb_list", [])
    assert isinstance(out, str)
