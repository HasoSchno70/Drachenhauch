# Module `firmata`

Direct control of an Arduino/ESP32 (or any other microcontroller flashed with
[Firmata](https://github.com/firmata/arduino)) — without sketch logic of your
own and without designing a text protocol of your own. Upload **File →
Examples → Firmata → StandardFirmata** once in the Arduino IDE; after that
Drachenhauch controls pins, servos and I2C devices directly.

```basic
IMPORT "firmata"
```

Native in dhrt, in **every** build: over the network (`tcp:host:port`) the
module needs no feature. Only the serial line needs the `serialport` crate
(feature `serial`, like the [`serial`](module-serial.md) module) — build with
`python rust/build_runtime.py --hardware`. Without the feature,
`FIRMATA_OPEN("COM3", …)` reports that only `tcp:` is available.

## Two lines

- **Serial**: `FIRMATA_OPEN("COM3", 57600)` or `/dev/ttyUSB0`. StandardFirmata
  speaks 57600 baud. Opening waits two seconds because many boards restart
  when the port is opened.
- **Network**: `FIRMATA_OPEN("tcp:192.168.1.50:3030")` — for a board running
  **StandardFirmataWiFi** (ESP32, ESP8266, Arduino with WiFi) or
  StandardFirmataEthernet; both listen on port 3030. There is no baud rate
  here, the second argument may be left out, and there is no waiting.

## What the module can do

Pins (digital, analogue, PWM), **servos**, **I2C** (write, read once or
continuously, 7- and 10-bit addresses), the **firmware query** and **stepper
motors**. The first four are part of StandardFirmata. Stepper motors are not:
for those the board needs **ConfigurableFirmata** with AccelStepperFirmata
(Arduino IDE: install the "ConfigurableFirmata" library, upload the
"ConfigurableFirmata" example). OneWire, encoders, the scheduler and the
capability query from the full protocol are not included.

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `FIRMATA_PORTS()` | STRING (comma-separated list of available ports, like `SERIAL_PORTS()`) | which serial ports are there? |
| `FIRMATA_OPEN(ziel$ [, baud])` | FIRMATA_HANDLE | establish the connection to the board: serial port or `tcp:host:port` |
| `FIRMATA_CLOSE(handle)` | — | close the connection |
| `FIRMATA_IS_OPEN(handle)` | BOOLEAN | is the connection still up? |
| `FIRMATA_FIRMWARE$(handle)` | STRING | name and version of the firmware, e.g. `StandardFirmata.ino 2.5` |
| `FIRMATA_PIN_MODE(handle, pin, modus)` | — | set the operating mode of a pin (input, output, PWM, analogue) |
| `FIRMATA_DIGITAL_WRITE(handle, pin, wert)` | — | switch a pin HIGH or LOW |
| `FIRMATA_DIGITAL_READ(handle, pin)` | BOOLEAN | read the reported state of an input -- the value comes from the last `FIRMATA_UPDATE` |
| `FIRMATA_ANALOG_WRITE(handle, pin, wert)` | — | output a PWM value (0..255), e.g. for brightness or motor speed |
| `FIRMATA_ANALOG_READ(handle, kanal)` | INTEGER (0..16383) | read an analogue value -- careful, what counts here is the **channel** (A0 = 0), not the pin number |
| `FIRMATA_SERVO_ATTACH(handle, pin [, min_us, max_us])` | — | attach a servo to a pin; the pulse widths for 0 and 180 degrees (default 544 and 2400) |
| `FIRMATA_SERVO_WRITE(handle, pin, winkel)` | — | set a servo to an angle 0..180 |
| `FIRMATA_I2C_CONFIG(handle [, verzoegerung_us])` | — | switch I2C on; otherwise the first I2C request does it by itself |
| `FIRMATA_I2C_WRITE(handle, adresse, bytes)` | — | send bytes to an I2C device (array of numbers or BUFFER) |
| `FIRMATA_I2C_READ(handle, adresse, register, anzahl)` | ARRAY OF INTEGER | read and wait for the answer (at most one second); register -1 = none |
| `FIRMATA_I2C_REQUEST(handle, adresse, register, anzahl [, dauernd])` | — | have bytes read without waiting; with `dauernd` the board reports again and again |
| `FIRMATA_I2C_DATA(handle, adresse [, register])` | ARRAY OF INTEGER | the bytes reported last, empty if nothing has arrived yet |
| `FIRMATA_I2C_STOP(handle, adresse)` | — | end continuous reading |
| `FIRMATA_STEPPER_CONFIG(handle, motor, art$, pin1, pin2 [, pin3, pin4])` | — | set up stepper motor 0..9: `treiber` (step + direction), `zweidraht`, `dreidraht`, `vierdraht` |
| `FIRMATA_STEPPER_SPEED(handle, motor, schritte_je_s)` | — | maximum speed |
| `FIRMATA_STEPPER_ACCEL(handle, motor, schritte_je_s2)` | — | acceleration, 0 = none |
| `FIRMATA_STEPPER_MOVE(handle, motor, schritte)` | — | move by that many steps, the sign is the direction |
| `FIRMATA_STEPPER_TO(handle, motor, ziel)` | — | move to a position |
| `FIRMATA_STEPPER_STOP(handle, motor)` | — | stop |
| `FIRMATA_STEPPER_ZERO(handle, motor)` | — | the current position becomes 0 |
| `FIRMATA_STEPPER_POSITION(handle, motor)` | INTEGER | last reported position (and asks for the next one at the same time) |
| `FIRMATA_STEPPER_RUNNING(handle, motor)` | BOOLEAN | is the motor still moving? TRUE until the board reports "done" |
| `FIRMATA_UPDATE(handle)` | — | **call once per frame**: fetches pin states, I2C answers and motor reports |

## Pin modes

`FIRMATA_PIN_MODE` takes raw Firmata mode values (no Drachenhauch constants,
analogous to `JOYSTICK_BUTTON`'s raw raylib indices):

| Value | Mode |
|---|---|
| 0 | INPUT |
| 1 | OUTPUT |
| 2 | ANALOG (pin as analogue input) |
| 3 | PWM |
| 4 | SERVO (set by `FIRMATA_SERVO_ATTACH` itself) |
| 6 | I2C (set by `FIRMATA_I2C_CONFIG` itself) |
| 11 | PULLUP (digital input with internal pull-up resistor) |

## Important: two different numberings

This is a genuine quirk of the Firmata protocol itself (verified against
StandardFirmata.ino, not guessed) — **writing and reading do not use the same
number for the same physical pin:**

- `FIRMATA_PIN_MODE`, `FIRMATA_DIGITAL_WRITE`, `FIRMATA_DIGITAL_READ`,
  `FIRMATA_ANALOG_WRITE`, `FIRMATA_SERVO_*` take the **raw digital pin
  number** (e.g. pin 9, 10, 11 for PWM on an Uno).
- `FIRMATA_ANALOG_READ` takes the **analogue channel** (A0 = 0, A1 = 1, A2 = 2, …)
  — NOT the same number as the board's digital pin label.

`FIRMATA_ANALOG_READ` goes up to channel 15 (the Firmata command byte only has
room for a 4-bit nibble for this). Higher pins can be written:
`FIRMATA_ANALOG_WRITE` and `FIRMATA_SERVO_WRITE` use the EXTENDED_ANALOG
command there (an ESP32 has PWM on pin 25 and higher).

## Servo

`FIRMATA_SERVO_ATTACH` attaches the servo (StandardFirmata switches the pin to
SERVO by itself), after that `FIRMATA_SERVO_WRITE` sets the angle. The pulse
widths are those of the Arduino servo library; a servo that hums at 0 or 180
degrees and does not quite get there needs different ones (often 1000 and
2000).

```basic
IMPORT "firmata"

DIM board AS FIRMATA_HANDLE
board = FIRMATA_OPEN("COM3", 57600)
FIRMATA_SERVO_ATTACH(board, 9)

DIM w AS INTEGER
FOR w = 0 TO 180 STEP 10
    FIRMATA_SERVO_WRITE(board, 9, w)
    SLEEP(100)
NEXT
FIRMATA_CLOSE(board)
```

## I2C

An I2C device has an address (an OLED display usually `&H3C`, an MPU6050
`&H68`) and usually registers to read from. `FIRMATA_I2C_READ` asks and waits
— convenient for programs without a frame loop. A game asks with
`FIRMATA_I2C_REQUEST` and fetches the answer in a later frame with
`FIRMATA_I2C_DATA`; with `dauernd` = TRUE the board keeps reading by itself.

```basic
IMPORT "firmata"

DIM board AS FIRMATA_HANDLE
board = FIRMATA_OPEN("COM3", 57600)

' wake up the MPU6050: register 0x6B to 0
FIRMATA_I2C_WRITE(board, &H68, [&H6B, 0])

' acceleration X: two bytes from register 0x3B, the high one first
DIM b AS ARRAY OF INTEGER
b = FIRMATA_I2C_READ(board, &H68, &H3B, 2)
DIM x AS INTEGER : x = b[0] * 256 + b[1]
IF x >= 32768 THEN x = x - 65536
PRINT "X: "; x
```

Continuously in the frame loop:

```basic
IMPORT "firmata"

DIM board AS FIRMATA_HANDLE
board = FIRMATA_OPEN("COM3", 57600)
FIRMATA_I2C_REQUEST(board, &H68, &H3B, 6, TRUE)

WHILE NOT QUITREQUESTED()
    FIRMATA_UPDATE(board)
    DIM werte AS ARRAY OF INTEGER : werte = FIRMATA_I2C_DATA(board, &H68, &H3B)
    IF LEN(werte) = 6 THEN PRINT werte[0] * 256 + werte[1]
    SLEEP(16)
WEND
FIRMATA_I2C_STOP(board, &H68)
```

When reading without a register (`-1`), StandardFirmata reports the answer
under register 0; `FIRMATA_I2C_DATA(board, adresse)` without a register takes
the newest answer of the address, no matter under which register.

## Stepper motor

Stepper motors need **ConfigurableFirmata** on the board (see above);
StandardFirmata does not know them and does not answer. Up to ten motors
(0..9); type `treiber` for a driver with step and direction pin (A4988,
DRV8825, TMC2208), `vierdraht` for a 28BYJ-48 on a ULN2003. The motor keeps
moving on the board even while the program does something else;
`FIRMATA_STEPPER_RUNNING` tells whether it is still moving.

```basic
IMPORT "firmata"

DIM board AS FIRMATA_HANDLE
board = FIRMATA_OPEN("COM3", 57600)
FIRMATA_STEPPER_CONFIG(board, 0, "treiber", 2, 3)
FIRMATA_STEPPER_SPEED(board, 0, 800.0)
FIRMATA_STEPPER_ACCEL(board, 0, 400.0)

FIRMATA_STEPPER_MOVE(board, 0, 1600)
WHILE FIRMATA_STEPPER_RUNNING(board, 0)
    FIRMATA_UPDATE(board)
    PRINT FIRMATA_STEPPER_POSITION(board, 0)
    SLEEP(50)
WEND
```

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
' channel 0 = A0 -- potentiometer connected to A0, no PIN_MODE needed for plain analogue reads.

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

`FIRMATA_OPEN` throws if the port does not exist or is in use, or if the board
on the network does not answer — catch it with `TRY/CATCH`, the same pattern
as `SERIAL_OPEN`. If the board hangs up (power gone, WiFi gone), the next
`FIRMATA_UPDATE` throws "die Verbindung zum Board ist beendet".

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
