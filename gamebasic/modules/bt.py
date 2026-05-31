"""Bluetooth-Low-Energy-Modul fuer GameBasic.

Klassisches Bluetooth (RFCOMM/SPP) wird *nicht* unterstuetzt - die
zustaendige Lib (pybluez) ist seit Jahren nicht mehr maintained. Modernes
BLE ueber 'bleak' deckt heute praktisch alle relevanten Anwendungsfaelle
ab (Sensoren, Wearables, Maker-Boards mit Nordic-Chip etc.).

Built-ins:
    BT_SCAN(timeout_sek)              -> STRING (multi-line "addr|name|rssi")
    BT_CONNECT(addr$)                 -> BT_HANDLE
    BT_DISCONNECT(handle)
    BT_IS_CONNECTED(handle)           -> BOOLEAN
    BT_SERVICES(handle)               -> STRING (multi-line UUIDs)
    BT_CHARACTERISTICS(handle, svc$)  -> STRING (multi-line "uuid|properties")
    BT_READ(handle, char_uuid$)       -> STRING (Bytes als latin-1)
    BT_WRITE(handle, char_uuid$, daten$)

Externe Dependency:
    pip install bleak

Async/Sync: bleak ist asyncio-basiert. Dieses Modul betreibt im Hintergrund
einen dedizierten Event-Loop in einem Daemon-Thread und reicht jeden Aufruf
synchron durch. Der GameBasic-VM blockiert pro Aufruf bis zur Antwort.

Bytes <-> String: Wie usb.* werden Bytes mit latin-1 round-trip-fest in
GameBasic-STRINGs verpackt.
"""
from __future__ import annotations

import asyncio
import threading

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type

try:
    from bleak import BleakClient as _BleakClient, BleakScanner as _BleakScanner
    _AVAILABLE = True
except ImportError:
    _BleakClient = None
    _BleakScanner = None
    _AVAILABLE = False


def _ensure():
    if not _AVAILABLE:
        raise GBRuntimeError(
            "Modul BT: 'bleak' ist nicht installiert.\n"
            "Installation:  pip install bleak"
        )


# --- Async/Sync-Bruecke -----------------------------------------------
_loop: asyncio.AbstractEventLoop | None = None
_loop_thread: threading.Thread | None = None
_loop_lock = threading.Lock()


def _get_loop() -> asyncio.AbstractEventLoop:
    global _loop, _loop_thread
    with _loop_lock:
        if _loop is not None:
            return _loop
        loop = asyncio.new_event_loop()

        def runner():
            asyncio.set_event_loop(loop)
            try:
                loop.run_forever()
            except Exception:
                pass

        t = threading.Thread(target=runner, daemon=True, name="bt-loop")
        t.start()
        _loop = loop
        _loop_thread = t
        return loop


def _await(coro, timeout: float | None = None):
    """Coroutine auf Hintergrund-Loop ausfuehren, blockiert bis Ergebnis."""
    fut = asyncio.run_coroutine_threadsafe(coro, _get_loop())
    try:
        return fut.result(timeout=timeout)
    except asyncio.TimeoutError:
        fut.cancel()
        raise GBRuntimeError("BT: Timeout beim Warten auf Bluetooth-Antwort")
    except Exception as exc:
        # Reraise als GB-Runtime-Error mit lesbarer Message
        raise GBRuntimeError(f"BT: {type(exc).__name__}: {exc}")


# --- Handle ----------------------------------------------------------
class _BTHandle:
    __slots__ = ("client", "address", "_disconnected")

    def __init__(self, client, address: str):
        self.client = client
        self.address = address
        self._disconnected = False

    def __repr__(self):
        s = "disconnected" if self._disconnected else "connected"
        return f"<BT '{self.address}' {s}>"


register_type("bt_handle", _BTHandle)


def _check_handle(v, fn: str) -> _BTHandle:
    if not isinstance(v, _BTHandle):
        raise TypeMismatchError(f"{fn} erwartet BT_HANDLE")
    if v._disconnected:
        raise GBRuntimeError(f"{fn}: Verbindung wurde bereits getrennt")
    return v


# --- Built-ins -------------------------------------------------------

@builtin("BT_SCAN", arity=1, types=("num",))
def _scan(timeout_s):
    _ensure()
    if timeout_s < 0:
        raise GBRuntimeError("BT_SCAN: Timeout muss >= 0 sein")

    async def _do():
        # Neuere bleak-Versionen geben dict[addr -> (BLEDevice, AdvData)]
        # zurueck. Aeltere geben list[BLEDevice]. Beide Faelle abdecken.
        try:
            data = await _BleakScanner.discover(
                timeout=float(timeout_s), return_adv=True
            )
        except TypeError:
            devices = await _BleakScanner.discover(timeout=float(timeout_s))
            return [(d.address, d.name or "?", getattr(d, "rssi", 0) or 0)
                    for d in devices]
        out = []
        for addr, payload in data.items():
            dev, adv = payload
            name = adv.local_name or dev.name or "?"
            rssi = getattr(adv, "rssi", 0) or 0
            out.append((addr, name, rssi))
        return out

    rows = _await(_do(), timeout=float(timeout_s) + 5.0)
    return "\n".join(f"{a}|{n}|{r}" for a, n, r in rows)


@builtin("BT_CONNECT", arity=1, types=("str",))
def _connect(addr):
    _ensure()
    client = _BleakClient(addr)

    async def _do():
        await client.connect()
        return client.is_connected

    ok = _await(_do(), timeout=20.0)
    if not ok:
        raise GBRuntimeError(f"BT_CONNECT: Konnte {addr} nicht verbinden")
    return _BTHandle(client, addr)


@builtin("BT_DISCONNECT", arity=1)
def _disconnect(h):
    h = _check_handle(h, "BT_DISCONNECT")

    async def _do():
        if h.client.is_connected:
            await h.client.disconnect()

    _await(_do(), timeout=10.0)
    h._disconnected = True


@builtin("BT_IS_CONNECTED", arity=1)
def _is_connected(h):
    if not isinstance(h, _BTHandle):
        raise TypeMismatchError("BT_IS_CONNECTED erwartet BT_HANDLE")
    if h._disconnected:
        return False
    try:
        return bool(h.client.is_connected)
    except Exception:
        return False


@builtin("BT_SERVICES", arity=1)
def _services(h):
    h = _check_handle(h, "BT_SERVICES")

    async def _do():
        # Modernes bleak: client.services (eager nach connect)
        # Aelter: get_services()
        try:
            svcs = h.client.services
            if not svcs:
                svcs = await h.client.get_services()
        except AttributeError:
            svcs = await h.client.get_services()
        return [s.uuid for s in svcs]

    uuids = _await(_do(), timeout=10.0)
    return "\n".join(uuids)


@builtin("BT_CHARACTERISTICS", arity=2, types=("any", "str"))
def _characteristics(h, svc_uuid):
    h = _check_handle(h, "BT_CHARACTERISTICS")

    async def _do():
        try:
            svcs = h.client.services
            if not svcs:
                svcs = await h.client.get_services()
        except AttributeError:
            svcs = await h.client.get_services()
        out = []
        for s in svcs:
            if s.uuid.lower() != svc_uuid.lower():
                continue
            for c in s.characteristics:
                props = ",".join(c.properties)
                out.append(f"{c.uuid}|{props}")
        return out

    rows = _await(_do(), timeout=10.0)
    return "\n".join(rows)


@builtin("BT_READ", arity=2, types=("any", "str"))
def _read(h, char_uuid):
    h = _check_handle(h, "BT_READ")

    async def _do():
        return bytes(await h.client.read_gatt_char(char_uuid))

    data = _await(_do(), timeout=10.0)
    return data.decode("latin-1")


@builtin("BT_WRITE", arity=3, types=("any", "str", "str"))
def _write(h, char_uuid, payload):
    h = _check_handle(h, "BT_WRITE")
    try:
        b = payload.encode("latin-1")
    except UnicodeEncodeError:
        raise GBRuntimeError(
            "BT_WRITE: STRING enthaelt Zeichen > 0xFF. "
            "Nur Bytes 0..255 sind erlaubt (latin-1)."
        )

    async def _do():
        await h.client.write_gatt_char(char_uuid, b)

    _await(_do(), timeout=10.0)
