"""Tests fuer das serial-Modul (pyserial-Wrapper).

Es gibt keinen echten COM-Port in der Test-Umgebung, daher koennen
Open/Read/Write nicht ohne weiteres geprueft werden. Stattdessen testen
wir hier die Registrierung, Type-Checks und das Verhalten bei fehlender
Lib (via Monkeypatch). Smoke-Tests, die echte Hardware brauchen, werden
mit Skip-Markern uebersprungen.
"""
import pytest

from gamebasic.modules import load_module, EXTERNAL_TYPES
from gamebasic.modules import serial as ser_mod
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("serial")


@pytest.fixture
def lib_missing(monkeypatch):
    """Simuliert: pyserial ist nicht installiert."""
    monkeypatch.setattr(ser_mod, "_AVAILABLE", False)


# --- Registrierung ---------------------------------------------------

def test_module_registers_external_type():
    assert "serial_handle" in EXTERNAL_TYPES


def test_all_builtins_registered():
    from gamebasic.interpreter import BUILTINS
    expected = {
        "serial_ports", "serial_open", "serial_close", "serial_is_open",
        "serial_write", "serial_read", "serial_readline",
        "serial_available", "serial_flush", "serial_timeout",
    }
    assert expected <= set(BUILTINS.keys())


# --- Fehlende Lib: alle Lib-pflichtigen Aufrufe melden klar ---------

def test_ports_without_lib(lib_missing, call_builtin):
    with pytest.raises(GBRuntimeError, match=r"pyserial.*installiert"):
        call_builtin("serial_ports", [])


def test_open_without_lib(lib_missing, call_builtin):
    with pytest.raises(GBRuntimeError, match=r"pyserial.*installiert"):
        call_builtin("serial_open", ["COM1", 9600])


# --- Type-Checks (laufen ohne pyserial, da sie vor _ensure greifen) -

def test_close_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="SERIAL_HANDLE"):
        call_builtin("serial_close", ["kein handle"])


def test_is_open_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="SERIAL_HANDLE"):
        call_builtin("serial_is_open", [42])


def test_write_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="SERIAL_HANDLE"):
        call_builtin("serial_write", ["nope", "daten"])


def test_read_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="SERIAL_HANDLE"):
        call_builtin("serial_read", [None, 10])


def test_readline_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="SERIAL_HANDLE"):
        call_builtin("serial_readline", [3.14])


def test_available_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="SERIAL_HANDLE"):
        call_builtin("serial_available", ["x"])


def test_flush_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="SERIAL_HANDLE"):
        call_builtin("serial_flush", [object()])


def test_timeout_rejects_non_handle(call_builtin):
    with pytest.raises(TypeMismatchError, match="SERIAL_HANDLE"):
        call_builtin("serial_timeout", ["x", 1.0])


# --- arity / type-spec ----------------------------------------------

def test_open_arity(call_builtin):
    with pytest.raises(GBRuntimeError, match="erwartet 2"):
        call_builtin("serial_open", ["COM1"])


def test_open_baud_must_be_int(call_builtin):
    with pytest.raises(TypeMismatchError):
        call_builtin("serial_open", ["COM1", "schnell"])


# --- echte Hardware (nur mit pyserial + idR ohne COM) ---------------

@pytest.mark.skipif(not ser_mod._AVAILABLE, reason="pyserial nicht installiert")
def test_ports_returns_string_when_lib_present(call_builtin):
    out = call_builtin("serial_ports", [])
    assert isinstance(out, str)


@pytest.mark.skipif(not ser_mod._AVAILABLE, reason="pyserial nicht installiert")
def test_open_nonexistent_port_errors(call_builtin):
    # Erwartet GBRuntimeError - SerialException wird sauber gewrappt.
    with pytest.raises(GBRuntimeError, match="SERIAL_OPEN"):
        call_builtin("serial_open", ["COM_DOES_NOT_EXIST_XYZ", 9600])
