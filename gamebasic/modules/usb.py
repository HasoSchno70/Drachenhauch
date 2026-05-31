"""USB-HID-Modul fuer GameBasic.

Greift auf USB-HID-Geraete zu (Custom-Controller, Programmer, Maker-Boards
mit HID-Profil). Klassisches "raw USB" via libusb wird *nicht* unterstuetzt
weil es auf Windows einen extra Treiber (WinUSB/Zadig) braucht.

Built-ins:
    USB_LIST()                        -> STRING (multi-line "vid:pid|product")
    USB_OPEN(vid, pid)                -> USB_HANDLE
    USB_OPEN_PATH(pfad$)              -> USB_HANDLE
    USB_CLOSE(handle)
    USB_WRITE(handle, daten$)         -> INTEGER (geschriebene Bytes)
    USB_READ(handle, n, timeout_ms)   -> STRING  (Bytes als latin-1-String)
    USB_PRODUCT(handle)               -> STRING
    USB_MANUFACTURER(handle)          -> STRING
    USB_SERIAL(handle)                -> STRING

Externe Dependency:
    pip install hidapi

Bytes <-> String: USB-HID-Reports sind Roh-Bytes. Damit beliebige Bytewerte
durch GameBasic-STRINGs durchlaufen, kodiert dieses Modul mit latin-1
(jeder Codepoint 0..255 entspricht einem Byte).

Beispiel - Custom-Macropad lesen:
    DIM dev AS USB_HANDLE
    dev = USB_OPEN(&H1234, &H5678)
    PRINT "Modell: " + USB_PRODUCT(dev)
    DIM data$ AS STRING
    data$ = USB_READ(dev, 8, 500)
    USB_CLOSE(dev)
"""
from __future__ import annotations

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type

try:
    import hid as _hidlib
    _AVAILABLE = True
except ImportError:
    _hidlib = None
    _AVAILABLE = False


def _ensure():
    if not _AVAILABLE:
        raise GBRuntimeError(
            "Modul USB: 'hidapi' ist nicht installiert.\n"
            "Installation:  pip install hidapi"
        )


class _USBHandle:
    __slots__ = ("dev", "vid", "pid", "_open")

    def __init__(self, dev, vid: int, pid: int):
        self.dev = dev
        self.vid = vid
        self.pid = pid
        self._open = True

    def __repr__(self):
        s = "open" if self._open else "closed"
        return f"<USB {self.vid:04X}:{self.pid:04X} {s}>"


register_type("usb_handle", _USBHandle)


def _check_handle(v, fn: str, *, allow_closed: bool = False) -> _USBHandle:
    if not isinstance(v, _USBHandle):
        raise TypeMismatchError(f"{fn} erwartet USB_HANDLE")
    if not allow_closed and not v._open:
        raise GBRuntimeError(f"{fn}: Geraet wurde bereits geschlossen")
    return v


@builtin("USB_LIST", arity=0)
def _list():
    _ensure()
    try:
        items = _hidlib.enumerate()
    except Exception as exc:
        raise GBRuntimeError(f"USB_LIST: {exc}")
    lines = []
    for it in items:
        vid = it.get("vendor_id", 0)
        pid = it.get("product_id", 0)
        prod = it.get("product_string") or ""
        manu = it.get("manufacturer_string") or ""
        # vid:pid hex|product|manufacturer
        lines.append(f"{vid:04X}:{pid:04X}|{prod}|{manu}")
    return "\n".join(lines)


@builtin("USB_OPEN", arity=2, types=("int", "int"))
def _open(vid, pid):
    _ensure()
    try:
        d = _hidlib.device()
        d.open(int(vid), int(pid))
    except (IOError, OSError) as exc:
        raise GBRuntimeError(
            f"USB_OPEN: {vid:04X}:{pid:04X} nicht gefunden oder belegt ({exc})"
        )
    return _USBHandle(d, int(vid), int(pid))


@builtin("USB_OPEN_PATH", arity=1, types=("str",))
def _open_path(path):
    _ensure()
    try:
        d = _hidlib.device()
        # hidapi erwartet bytes fuer den Pfad
        d.open_path(path.encode("utf-8") if isinstance(path, str) else path)
    except (IOError, OSError) as exc:
        raise GBRuntimeError(f"USB_OPEN_PATH: '{path}' nicht erreichbar ({exc})")
    return _USBHandle(d, 0, 0)


@builtin("USB_CLOSE", arity=1)
def _close(h):
    h = _check_handle(h, "USB_CLOSE", allow_closed=True)
    if h._open:
        try:
            h.dev.close()
        except Exception:
            pass
        h._open = False


@builtin("USB_WRITE", arity=2, types=("any", "str"))
def _write(h, data):
    h = _check_handle(h, "USB_WRITE")
    # latin-1 ist round-trip-fest fuer Bytes 0..255
    try:
        b = data.encode("latin-1")
    except UnicodeEncodeError:
        raise GBRuntimeError(
            "USB_WRITE: STRING enthaelt Zeichen > 0xFF. "
            "Nur Bytes 0..255 sind erlaubt (latin-1)."
        )
    try:
        return int(h.dev.write(list(b)))
    except (IOError, OSError) as exc:
        raise GBRuntimeError(f"USB_WRITE: {exc}")


@builtin("USB_READ", arity=3, types=("any", "int", "int"))
def _read(h, n, timeout_ms):
    h = _check_handle(h, "USB_READ")
    if n < 0:
        raise GBRuntimeError("USB_READ: Anzahl muss >= 0 sein")
    try:
        data = h.dev.read(int(n), timeout_ms=int(timeout_ms))
    except (IOError, OSError) as exc:
        raise GBRuntimeError(f"USB_READ: {exc}")
    return bytes(data).decode("latin-1")


@builtin("USB_PRODUCT", arity=1)
def _product(h):
    h = _check_handle(h, "USB_PRODUCT")
    try:
        return h.dev.get_product_string() or ""
    except Exception:
        return ""


@builtin("USB_MANUFACTURER", arity=1)
def _manufacturer(h):
    h = _check_handle(h, "USB_MANUFACTURER")
    try:
        return h.dev.get_manufacturer_string() or ""
    except Exception:
        return ""


@builtin("USB_SERIAL", arity=1)
def _serial(h):
    h = _check_handle(h, "USB_SERIAL")
    try:
        return h.dev.get_serial_number_string() or ""
    except Exception:
        return ""
