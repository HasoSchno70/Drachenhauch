"""Tests fuer das `firmata`-Modul (direkte Arduino/ESP32-Pin-Steuerung ueber
StandardFirmata). Die eigentliche Protokoll-Logik (Bit-Packing, Nachrichten-
Parsing) wird in Rust getestet (rust/drachenhauch_runtime/src/firmata.rs, `cargo test`)
-- hier nur Registrierung + IMPORT/Typ-Verdrahtung, analog zu
test_joystick_rumble_no_gamepad_graceful_error in test_language_extensions.py.
Kein echtes Arduino noetig (wie bei serial/usb/bt/wifi gibt es dafuer keine
run_gb-Golden-Tests in diesem Projekt).
"""
import pytest


def test_firmata_builtins_registered():
    from gamebasic.editor_qt.dhrt_meta import builtin_names_lower
    expected = {
        "firmata_ports", "firmata_open", "firmata_close", "firmata_is_open",
        "firmata_pin_mode", "firmata_digital_write", "firmata_digital_read",
        "firmata_analog_write", "firmata_analog_read", "firmata_update",
    }
    assert expected <= builtin_names_lower()


def test_firmata_is_a_known_module():
    from gamebasic.modules import is_known_module
    assert is_known_module("firmata")


def test_firmata_handle_type_check_and_import(run_gb):
    """IMPORT + DIM AS FIRMATA_HANDLE muessen kompilieren; der eigentliche
    FIRMATA_OPEN-Call schlaegt in diesem (Standard-, nicht --hardware-)Build
    kontrolliert zur Laufzeit fehl -- genau wie SERIAL_OPEN/USB_LIST/BT_SCAN
    ohne die jeweiligen Hardware-Features."""
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError, match="FIRMATA_OPEN"):
        run_gb('''
IMPORT "firmata"
DIM h AS FIRMATA_HANDLE
h = FIRMATA_OPEN("COM3", 57600)
''')
