# Module `bt` — Bluetooth Low Energy

Scan BLE devices, connect, read and write GATT characteristics. Typical use cases: sensors (heart rate, temperature, wearables), maker boards with a Nordic chip (nRF52, ESP32-BLE), smart plugs.

**Classic Bluetooth (RFCOMM/SPP) is not supported** — the library responsible for it, `pybluez`, has been unmaintained for years. Today BLE covers practically all relevant use cases.

```basic
IMPORT "bt"
```

## Requirement

`bt` is only included in a `dhrt` built with the hardware features
(crate `btleplug`, no Python packages):

```
python rust/build_runtime.py --hardware
```

On Windows this works with the built-in BT stack from Win 10 onwards — the Bluetooth adapter must be switched on.

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `BT_SCAN(timeout_sek)` | STRING (multi-line `addr|name|rssi`), 0..300s | search for devices in range |
| `BT_CONNECT(addr$)` | BT_HANDLE | establish a connection to a device |
| `BT_DISCONNECT(handle)` | — | disconnect |
| `BT_IS_CONNECTED(handle)` | BOOLEAN | is the connection still up? |
| `BT_SERVICES(handle)` | STRING (multi-line UUIDs) | which services does the device offer? |
| `BT_CHARACTERISTICS(handle, svc$)` | STRING (multi-line `uuid|properties`) | which characteristics does a service have -- and what can be done with them? |
| `BT_READ(handle, char_uuid$)` | STRING (bytes as latin-1) | read the value of a characteristic |
| `BT_WRITE(handle, char_uuid$, daten$)` | — | write the value of a characteristic |

## Async/sync

`btleplug` is asynchronous. The module drives it through an internal tokio runtime and passes every call through synchronously — the Drachenhauch VM blocks on each call until the answer arrives. You do not have to deal with coroutines or `await`.

All calls except `BT_SCAN` (which has its own time window, chosen by you) have an internal 10-second timeout — if a device is out of range or does not respond, `BT_CONNECT`/`BT_READ`/`BT_WRITE`/`BT_SERVICES`/`BT_CHARACTERISTICS` fails after 10 s at the latest with a catchable error message, instead of freezing the game loop indefinitely.

## Bytes ↔ STRING

GATT values are raw bytes. As in the USB module, they are packed into Drachenhauch STRINGs with **latin-1**, round-trip safe:

```basic
DIM raw$ AS STRING
raw$ = BT_READ(dev, "00002a19-0000-1000-8000-00805f9b34fb")  ' Battery Level
' raw$ is 1 byte; value via ASC(...)
PRINT "Akku: ", ASC(raw$), "%"
```

## Example — scan

```basic
IMPORT "bt"

PRINT "Scanne 5 Sekunden ..."
DIM gefunden AS STRING
gefunden = BT_SCAN(5.0)

DIM zeilen AS ARRAY OF STRING
zeilen = SPLIT$(gefunden, CHR$(10))
DIM i AS INTEGER
FOR i = 0 TO LEN(zeilen) - 1
    PRINT "  ", zeilen[i]
NEXT
```

Example output:

```
AA:BB:CC:DD:EE:FF|My Heart Rate|-67
11:22:33:44:55:66|ESP32-Sensor|-71
```

## Example — connect, read a characteristic

```basic
IMPORT "bt"

DIM dev AS BT_HANDLE
dev = BT_CONNECT("AA:BB:CC:DD:EE:FF")

' Which services does the device have?
PRINT BT_SERVICES(dev)

' Characteristics of a particular service
PRINT BT_CHARACTERISTICS(dev, "0000180f-0000-1000-8000-00805f9b34fb")

' Read the battery level (standard UUID 2A19)
DIM lvl AS STRING
lvl = BT_READ(dev, "00002a19-0000-1000-8000-00805f9b34fb")
PRINT "Akku: ", ASC(lvl), "%"

BT_DISCONNECT(dev)
```

## Standard UUIDs

BLE defines service and characteristic UUIDs as 128-bit values. Standardised profiles have a 16-bit short value (e.g. `2A19` for Battery Level), which is expanded to the form `0000XXXX-0000-1000-8000-00805f9b34fb`. A few useful ones:

| UUID | Meaning |
|---|---|
| `0000180f-0000-1000-8000-00805f9b34fb` | Battery Service |
| `00002a19-0000-1000-8000-00805f9b34fb` | Battery Level (1 byte 0..100) |
| `0000180d-0000-1000-8000-00805f9b34fb` | Heart Rate Service |
| `00002a37-0000-1000-8000-00805f9b34fb` | Heart Rate Measurement |
| `0000180a-0000-1000-8000-00805f9b34fb` | Device Information |

For proprietary devices, the UUIDs are in the manufacturer's data sheet.

## Error handling

All operations throw `DHRuntimeError` with a meaningful message. Typical causes: device out of range, BT adapter off, wrong UUID, the characteristic does not support the requested operation.

```basic
TRY
    DIM dev AS BT_HANDLE
    dev = BT_CONNECT("AA:BB:CC:DD:EE:FF")
CATCH e
    PRINT "BLE-Verbindung fehlgeschlagen: ", e
END TRY
```

## Complete example

See [examples/38_bt.dh](../../examples/38_bt.dh).

## In the native runtime (dhrt)

`bt` runs natively with the Cargo feature `bt` (crate `btleplug`; async is driven synchronously through an internal tokio runtime). Bytes ↔ STRING via latin-1. **Note:** `BT_CONNECT(addr$)` needs an address previously seen by `BT_SCAN`. `BT_SCAN` validates `timeout_sek` strictly (finite number 0..300) — a NaN/infinity value (e.g. from a calculation chain such as `POW(10,1000)`) throws a clean error instead of crashing the runtime. Build: `python rust/build_runtime.py --hardware`. Pulls in heavy dependencies (tokio/btleplug/windows) — hence not in the standard dev build.
