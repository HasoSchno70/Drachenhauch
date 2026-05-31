"""Serielle Kommunikation (RS-232 / USB-COM) fuer GameBasic.

Built-ins:
    SERIAL_PORTS()                       -> STRING  (komma-getrennte Liste)
    SERIAL_OPEN(port$, baud)             -> SERIAL_HANDLE
    SERIAL_CLOSE(handle)
    SERIAL_IS_OPEN(handle)               -> BOOLEAN
    SERIAL_WRITE(handle, s$)             -> INTEGER (geschriebene Bytes)
    SERIAL_READ(handle, n)               -> STRING  (bis zu n Bytes als utf-8/replace)
    SERIAL_READLINE(handle)              -> STRING  (bis Newline)
    SERIAL_AVAILABLE(handle)             -> INTEGER (wartende Bytes im Eingangspuffer)
    SERIAL_FLUSH(handle)                 (Eingangs- und Ausgangspuffer leeren)
    SERIAL_TIMEOUT(handle, sekunden)     (Read-Timeout setzen)

Externe Dependency:
    pip install pyserial

Beispiel:
    DIM port AS SERIAL_HANDLE
    port = SERIAL_OPEN("COM3", 9600)
    SERIAL_WRITE(port, "LED ON" + CHR$(10))
    PRINT SERIAL_READLINE(port)
    SERIAL_CLOSE(port)
"""
from __future__ import annotations

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type

try:
    import serial as _serlib  # pyserial
    _AVAILABLE = True
except ImportError:
    _serlib = None
    _AVAILABLE = False


def _ensure():
    if not _AVAILABLE:
        raise GBRuntimeError(
            "Modul SERIAL: 'pyserial' ist nicht installiert.\n"
            "Installation:  pip install pyserial"
        )


class _SerialHandle:
    __slots__ = ("ser", "port")

    def __init__(self, ser, port: str):
        self.ser = ser
        self.port = port

    def __repr__(self):
        state = "open" if self.ser is not None and self.ser.is_open else "closed"
        return f"<SERIAL '{self.port}' {state}>"


register_type("serial_handle", _SerialHandle)


def _check_handle(v, fn: str, *, allow_closed: bool = False) -> _SerialHandle:
    if not isinstance(v, _SerialHandle):
        raise TypeMismatchError(f"{fn} erwartet SERIAL_HANDLE")
    if not allow_closed and (v.ser is None or not v.ser.is_open):
        raise GBRuntimeError(f"{fn}: Port wurde bereits geschlossen")
    return v


@builtin("SERIAL_PORTS", arity=0)
def _ports():
    _ensure()
    from serial.tools import list_ports
    return ", ".join(p.device for p in list_ports.comports())


@builtin("SERIAL_OPEN", arity=2, types=("str", "int"))
def _open(port, baud):
    _ensure()
    try:
        s = _serlib.Serial(port, baudrate=int(baud), timeout=1.0)
    except _serlib.SerialException as exc:
        raise GBRuntimeError(f"SERIAL_OPEN: {exc}")
    except ValueError as exc:
        raise GBRuntimeError(f"SERIAL_OPEN: ungueltige Parameter ({exc})")
    return _SerialHandle(s, port)


@builtin("SERIAL_CLOSE", arity=1)
def _close(h):
    h = _check_handle(h, "SERIAL_CLOSE", allow_closed=True)
    if h.ser is not None and h.ser.is_open:
        h.ser.close()


@builtin("SERIAL_IS_OPEN", arity=1)
def _is_open(h):
    if not isinstance(h, _SerialHandle):
        raise TypeMismatchError("SERIAL_IS_OPEN erwartet SERIAL_HANDLE")
    return bool(h.ser is not None and h.ser.is_open)


@builtin("SERIAL_WRITE", arity=2, types=("any", "str"))
def _write(h, s):
    h = _check_handle(h, "SERIAL_WRITE")
    try:
        return int(h.ser.write(s.encode("utf-8")))
    except _serlib.SerialException as exc:
        raise GBRuntimeError(f"SERIAL_WRITE: {exc}")


@builtin("SERIAL_READ", arity=2, types=("any", "int"))
def _read(h, n):
    h = _check_handle(h, "SERIAL_READ")
    if n < 0:
        raise GBRuntimeError("SERIAL_READ: Anzahl muss >= 0 sein")
    try:
        data = h.ser.read(n)
    except _serlib.SerialException as exc:
        raise GBRuntimeError(f"SERIAL_READ: {exc}")
    return data.decode("utf-8", errors="replace")


@builtin("SERIAL_READLINE", arity=1)
def _readline(h):
    h = _check_handle(h, "SERIAL_READLINE")
    try:
        line = h.ser.readline()
    except _serlib.SerialException as exc:
        raise GBRuntimeError(f"SERIAL_READLINE: {exc}")
    return line.decode("utf-8", errors="replace")


@builtin("SERIAL_AVAILABLE", arity=1)
def _available(h):
    h = _check_handle(h, "SERIAL_AVAILABLE")
    try:
        return int(h.ser.in_waiting)
    except _serlib.SerialException as exc:
        raise GBRuntimeError(f"SERIAL_AVAILABLE: {exc}")


@builtin("SERIAL_FLUSH", arity=1)
def _flush(h):
    h = _check_handle(h, "SERIAL_FLUSH")
    h.ser.reset_input_buffer()
    h.ser.reset_output_buffer()


@builtin("SERIAL_TIMEOUT", arity=2, types=("any", "num"))
def _timeout(h, seconds):
    h = _check_handle(h, "SERIAL_TIMEOUT")
    if seconds < 0:
        raise GBRuntimeError("SERIAL_TIMEOUT: Sekunden muss >= 0 sein")
    h.ser.timeout = float(seconds)
