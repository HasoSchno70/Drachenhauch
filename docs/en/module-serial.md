# Module `serial`

Serial communication over RS-232 / USB COM. Typical use case: reading and writing data from an Arduino, ESP32 or another microcontroller.

```basic
IMPORT "serial"
```

## Requirement

`serial` is only included in a `dhrt` built with the hardware features
(crate `serialport`, no Python packages):

```
python rust/build_runtime.py --hardware
```

If the feature is missing, the first call to a `SERIAL_*` function reports that it is not available in this build.

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `SERIAL_PORTS()` | STRING (comma-separated list of available ports) | which serial ports are there? |
| `SERIAL_OPEN(port$, baud)` | SERIAL_HANDLE | open a port (`baud` must match the other side) |
| `SERIAL_CLOSE(handle)` | — | close the port |
| `SERIAL_IS_OPEN(handle)` | BOOLEAN | is the connection still up? |
| `SERIAL_WRITE(handle, s$)` | INTEGER (bytes written) | send text |
| `SERIAL_READ(handle, n)` | STRING (up to n bytes, max. 64 MiB) | read up to n bytes |
| `SERIAL_READLINE(handle)` | STRING (up to newline) | read a line -- up to the line break or until the timeout kicks in |
| `SERIAL_AVAILABLE(handle)` | INTEGER (bytes waiting in the input buffer) | how much is ready? (without waiting) |
| `SERIAL_FLUSH(handle)` | — (clear input and output buffers) | discard buffers |
| `SERIAL_TIMEOUT(handle, sekunden)` | — (set the read timeout, default 1.0 s) | how long a read waits at most |

## Bytes ↔ STRING

`SERIAL_READ` and `SERIAL_READLINE` decode as UTF-8 with a replace strategy for undecodable bytes. For pure text protocols (Arduino serial print, NMEA, AT commands) that is enough. For a raw binary protocol, better parse byte value by byte value via `MID$` and `ASC`.

`SERIAL_READ` correctly holds back a multi-byte UTF-8 character (umlaut/special character) that arrives split exactly at a read boundary until the next `SERIAL_READ` call, instead of displaying it as `�`. `n` is limited to 64 MiB per call — an accidentally huge `n` (e.g. swapped with another parameter) throws a clear error instead of triggering a huge allocation. `SERIAL_READLINE`, by contrast, decodes lossily once per call (no buffering across timeouts).

`SERIAL_WRITE` encodes the STRING as UTF-8 — pure ASCII bytes (0..127) go out 1:1, special characters become multi-byte UTF-8 sequences.

## Example — Arduino with an echo loop

```basic
IMPORT "serial"

' List available ports
PRINT "Ports: ", SERIAL_PORTS()

DIM port AS SERIAL_HANDLE
port = SERIAL_OPEN("COM3", 9600)
SERIAL_TIMEOUT(port, 2.0)

' Send a command, read the answer
SERIAL_WRITE(port, "LED ON" + CHR$(10))
PRINT "Antwort: ", SERIAL_READLINE(port)

SERIAL_CLOSE(port)
```

## Polling loop without blocking

`SERIAL_AVAILABLE` shows how many bytes can be read without waiting. That lets you build a game loop that only reads when data is there:

```basic
WHILE NOT QUITREQUESTED()
    IF SERIAL_AVAILABLE(port) > 0 THEN
        DIM zeile AS STRING
        zeile = SERIAL_READLINE(port)
        PRINT "Sensor: ", TRIM$(zeile)
    END IF
    SLEEP(16)
WEND
```

## Error handling

`SERIAL_OPEN` throws if the port does not exist or is in use. Catch it with `TRY/CATCH`:

```basic
TRY
    DIM port AS SERIAL_HANDLE
    port = SERIAL_OPEN("COM_NICHT_DA", 9600)
CATCH e
    PRINT "Konnte Port nicht oeffnen: ", e
END TRY
```

`SERIAL_CLOSE` is idempotent — closing twice does not throw.

Write and read errors after closing throw a clear "port has already been closed" message („Port wurde bereits geschlossen").

## Complete example

See [examples/35_serial.dh](../../examples/35_serial.dh).

## In the native runtime (dhrt)

`serial` runs natively with the Cargo feature `serial` (crate `serialport` — **no** `pyserial` needed). Build: `python rust/build_runtime.py --hardware` (or `--full`). If the feature is missing, the built-in reports "not available".
