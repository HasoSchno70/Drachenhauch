# Module `usb`

Access to USB HID devices: custom controllers, macro pads, programmer boards with a HID profile, keyboards, mice, gamepads.

Classic "raw USB" via `pyusb` / `libusb` is **not** supported — on Windows that needs a WinUSB driver per device (Zadig), on Linux root rights or udev rules. HID is considerably more portable and sufficient for most maker applications.

```basic
IMPORT "usb"
```

## Requirement

`usb` is only included in a `dhrt` built with the hardware features
(crate `hidapi`, no Python packages):

```
python rust/build_runtime.py --hardware
```

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `USB_LIST()` | STRING (multi-line `vid:pid|product|manufacturer`) | list connected HID devices |
| `USB_OPEN(vid, pid)` | USB_HANDLE | open a device via vendor and product ID |
| `USB_OPEN_PATH(pfad$)` | USB_HANDLE | open a device via its system path (unambiguous when two identical ones are plugged in) |
| `USB_CLOSE(handle)` | — | close the device |
| `USB_WRITE(handle, daten$)` | INTEGER (bytes written) | send a report to the device |
| `USB_READ(handle, n, timeout_ms)` | STRING (bytes as latin-1) | read up to n bytes; after `timeout_ms` whatever is there is returned |
| `USB_PRODUCT(handle)` | STRING | product name according to the device |
| `USB_MANUFACTURER(handle)` | STRING | manufacturer name according to the device |
| `USB_SERIAL(handle)` | STRING | serial number according to the device |

## Bytes ↔ STRING

USB HID reports are raw bytes. So that arbitrary byte values pass losslessly through Drachenhauch STRINGs, this module encodes with **latin-1** — every code point 0..255 corresponds to one byte:

```basic
DIM data$ AS STRING
data$ = USB_READ(dev, 8, 500)
' data$[0] is the first report byte as a character with code point 0..255
PRINT ASC(data$)         ' first byte as an integer
```

Writing works symmetrically — the STRING may contain characters with code points 0..255:

```basic
USB_WRITE(dev, CHR$(&H01) + CHR$(&HFF) + CHR$(&H00))
```

If a character >255 is passed, `USB_WRITE` throws.

## Finding the VID/PID

`USB_LIST()` returns all HID devices currently reachable — every call scans anew (newly plugged-in devices show up immediately, removed ones disappear):

```basic
IMPORT "usb"

DIM liste AS STRING
liste = USB_LIST()
PRINT liste
```

Example output:

```
046D:C52B|USB Receiver|Logitech
1234:5678|MyMacroPad|Maker
```

In example code, VIDs/PIDs are typically written as hex constants:

```basic
DIM dev AS USB_HANDLE
dev = USB_OPEN(&H046D, &HC52B)     ' Logitech receiver
PRINT "Modell: ", USB_PRODUCT(dev)
PRINT "Hersteller: ", USB_MANUFACTURER(dev)
USB_CLOSE(dev)
```

## Read with timeout

`USB_READ(handle, n, timeout_ms)` blocks for up to `timeout_ms` milliseconds waiting for data. `0` = return immediately (non-blocking, good for game loops). A negative value / `-1` = blocking without a timeout. `n` is limited to 64 MiB per call — an accidentally huge `n` (e.g. swapped with `timeout_ms`) throws a clear error instead of triggering a huge allocation.

```basic
WHILE NOT QUITREQUESTED()
    DIM rpt$ AS STRING
    rpt$ = USB_READ(dev, 8, 0)        ' non-blocking
    IF LEN(rpt$) > 0 THEN
        PRINT "Report: byte0=", ASC(rpt$)
    END IF
    SLEEP(16)
WEND
```

## Permissions

- **Windows**: HID is open to user apps, no driver tricks needed. **Keyboards / mice**, however, are opened exclusively by the system — `USB_OPEN` fails there. Custom HID devices with their own purpose (usage page) work without problems.
- **Linux**: hidraw usually needs udev rules, otherwise root only.
- **macOS**: similar to Windows, keyboards/mice are exclusive.

## Error handling

`USB_OPEN` throws with the hex VID:PID in the message if the device is not found / in use:

```basic
TRY
    DIM dev AS USB_HANDLE
    dev = USB_OPEN(&H1234, &H5678)
CATCH e
    PRINT "Geraet nicht erreichbar: ", e
END TRY
```

## Complete example

See [examples/37_usb.dh](../../examples/37_usb.dh).

## In the native runtime (dhrt)

`usb` runs natively with the Cargo feature `usb` (crate `hidapi`). Bytes ↔ STRING via latin-1. Build: `python rust/build_runtime.py --hardware` (or `--full`).
