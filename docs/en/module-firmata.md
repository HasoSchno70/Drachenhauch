# Module `firmata`

Direct pin control of an Arduino/ESP32 (or any other microcontroller flashed
with [StandardFirmata](https://github.com/firmata/arduino)) over an existing
serial connection — without sketch logic of your own and without designing a
text protocol of your own. Upload **File → Examples → Firmata →
StandardFirmata** once in the Arduino IDE; after that Drachenhauch controls
the pins directly.

```basic
IMPORT "firmata"
```

Native in dhrt (feature `serial` — the same `serialport` dependency as the
[`serial`](module-serial.md) module, no additional dependency). Build:
`python rust/build_runtime.py --hardware` (or `--full`). If the feature is
missing, every `FIRMATA_*` call reports "not available" with the build hint.

## Deliberately not covered

Only the pin I/O basics of the Firmata protocol: pin mode, digital and
analogue I/O. **I2C, servo, OneWire, stepper, encoder and the capability
query SysEx** from the full Firmata feature set are not implemented — for
most hobbyist projects (LEDs, push buttons, potentiometers, sensors, simple
motors via PWM) the pin I/O basics are enough.

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `FIRMATA_PORTS()` | STRING (comma-separated list of available ports, like `SERIAL_PORTS()`) | which serial ports are there? |
| `FIRMATA_OPEN(port$, baud)` | FIRMATA_HANDLE | establish the connection to the board (StandardFirmata must be running there) |
| `FIRMATA_CLOSE(handle)` | — | close the connection |
| `FIRMATA_IS_OPEN(handle)` | BOOLEAN | is the connection still up? |
| `FIRMATA_PIN_MODE(handle, pin, modus)` | — | set the operating mode of a pin (input, output, PWM, analogue) |
| `FIRMATA_DIGITAL_WRITE(handle, pin, wert)` | — | switch a pin HIGH or LOW |
| `FIRMATA_DIGITAL_READ(handle, pin)` | BOOLEAN | read the reported state of an input -- the value comes from the last `FIRMATA_UPDATE` |
| `FIRMATA_ANALOG_WRITE(handle, pin, wert)` | — | output a PWM value (0..255), e.g. for brightness or motor speed |
| `FIRMATA_ANALOG_READ(handle, kanal)` | INTEGER (0..16383) | read an analogue value -- careful, what counts here is the **channel** (A0 = 0), not the pin number |
| `FIRMATA_UPDATE(handle)` | — | **call once per frame**: fetches the pin states reported by the board |

## Pin modes

`FIRMATA_PIN_MODE` takes raw Firmata mode values (no Drachenhauch constants,
analogous to the raw raylib indices of `JOYSTICK_BUTTON`):

| Value | Mode |
|---|---|
| 0 | INPUT |
| 1 | OUTPUT |
| 2 | ANALOG (pin as analogue input) |
| 3 | PWM |
| 11 | PULLUP (digital input with internal pull-up resistor) |

## Important: two different numbering schemes

This is a genuine quirk of the Firmata protocol itself (verified against
StandardFirmata.ino, not guessed) — **writing and reading do not use the same
number for the same physical pin:**

- `FIRMATA_PIN_MODE`, `FIRMATA_DIGITAL_WRITE`, `FIRMATA_DIGITAL_READ`,
  `FIRMATA_ANALOG_WRITE` take the **raw digital pin number** (e.g. pin 9,
  10, 11 for PWM on an Uno).
- `FIRMATA_ANALOG_READ` takes the **analogue channel** (A0 = 0, A1 = 1, A2 = 2, …)
  — NOT the same number as the board's digital pin label.

`FIRMATA_ANALOG_WRITE`/`FIRMATA_ANALOG_READ` are additionally limited to pin/channel
0..15 (the Firmata command byte only has room for a 4-bit nibble for this;
higher analogue pins would need the EXTENDED_ANALOG SysEx command, which is
not implemented here).

## Example — blinking an LED

```basic
IMPORT "firmata"

DIM board AS FIRMATA_HANDLE
board = FIRMATA_OPEN("COM3", 57600)
' FIRMATA_OPEN blocks for ~2 seconds -- many boards reset themselves when
' the port is opened (DTR toggle) and need a short boot time before
' StandardFirmata accepts commands. A known Firmata quirk.

FIRMATA_PIN_MODE(board, 13, 1)   ' pin 13 = OUTPUT (built-in LED on many boards)

DIM an AS BOOLEAN
an = TRUE
WHILE NOT QUITREQUESTED()
    FIRMATA_DIGITAL_WRITE(board, 13, an)
    an = NOT an
    SLEEP(500)
WEND

FIRMATA_CLOSE(board)
```

## Example — reading a push button + a potentiometer

`FIRMATA_UPDATE` must be called **once per frame** (like
`INPUT_UPDATE()`/`TIMER_UPDATE()`) — it reads all currently available bytes
without blocking and updates the digital/analogue caches. Without
`FIRMATA_UPDATE`, `FIRMATA_DIGITAL_READ`/`FIRMATA_ANALOG_READ` stay at the
last known state.

```basic
IMPORT "firmata"

DIM board AS FIRMATA_HANDLE
board = FIRMATA_OPEN("COM3", 57600)

FIRMATA_PIN_MODE(board, 2, 11)   ' push button on pin 2, PULLUP (pressed = FALSE)
' Channel 0 = A0 -- potentiometer on A0, no PIN_MODE needed for pure analogue reads.

WHILE NOT QUITREQUESTED()
    FIRMATA_UPDATE(board)

    IF NOT FIRMATA_DIGITAL_READ(board, 2) THEN
        PRINT "Taster gedrueckt"
    END IF
    PRINT "Poti: ", FIRMATA_ANALOG_READ(board, 0)

    SLEEP(16)
WEND
```

## Error handling

`FIRMATA_OPEN` throws if the port does not exist or is in use — catch it with
`TRY/CATCH`, the same pattern as `SERIAL_OPEN`:

```basic
TRY
    DIM board AS FIRMATA_HANDLE
    board = FIRMATA_OPEN("COM_NICHT_DA", 57600)
CATCH e
    PRINT "Konnte Board nicht oeffnen: ", e
END TRY
```

`FIRMATA_CLOSE` is idempotent — closing twice does not throw.

## Complete example

See [examples/147_firmata.dh](../../examples/147_firmata.dh).
