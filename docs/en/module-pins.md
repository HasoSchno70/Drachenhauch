# Modules `gpio`, `i2c` and `spi`

The pins of a **Raspberry Pi** straight from Drachenhauch — no Arduino in
between: switch LEDs and relays, read buttons, query I2C sensors, talk to SPI
chips.

```basic
IMPORT "gpio"
IMPORT "i2c"
IMPORT "spi"
```

The three modules talk to the interfaces of the **Linux kernel**
(`/dev/gpiochipN`, `/dev/i2c-N`, `/dev/spidevB.C`), not to the registers of
the Pi's chip. That is why they work on every Pi — the 3, 4 and 5 have three
different GPIO chips, the Pi 5 its RP1 behind PCIe — and on any other Linux
computer with pins (Orange Pi, BeagleBone, Rock Pi …). Under Linux they are in
**every** build of dhrt, without a feature. Under Windows and macOS the
commands exist too, but each one reports "only under Linux";
`MODULE_AVAILABLE("gpio")` tells you beforehand whether it works.

For an Arduino or ESP32 on a USB cable or in the WiFi,
[`firmata`](module-firmata.md) is the way.

## Preparing the Pi

- **GPIO** works right away. The user has to be in the group `gpio` (on
  Raspberry Pi OS they are from the start).
- **I2C** and **SPI** are switched off: `sudo raspi-config` → *Interface
  Options* → *I2C* or *SPI* → enable, reboot. After that there are
  `/dev/i2c-1` and `/dev/spidev0.0`/`/dev/spidev0.1`. Groups: `i2c` and `spi`.

If a permission is missing or an interface is off, the message says what to
do.

## Which number?

Pins are counted by their **GPIO numbers** (also "BCM numbers"), not by
their position on the header: the pin that pinout diagrams call "GPIO17"
(pin 11) is pin 17. `GPIO_FIND(chip, "GPIO17")` looks the number up by the
name the kernel gives the line, and `GPIO_LINE_NAME$` returns it.

| Header pin | GPIO | often used for |
|---|---|---|
| 3, 5 | 2, 3 | I2C (SDA, SCL) — do not use as GPIO while I2C is on |
| 11 | 17 | LED, button |
| 12 | 18 | LED, buzzer |
| 19, 21, 23, 24 | 10, 9, 11, 8 | SPI (MOSI, MISO, SCLK, CE0) |

## GPIO

`GPIO_OPEN()` without an argument takes the chip with the header
(`pinctrl-rp1` on the Pi 5, `pinctrl-bcm2711` on the Pi 4,
`pinctrl-bcm2835` on the Pi 3 and Zero). A pin you write to becomes an output
by itself; one you read becomes an input. You need `GPIO_MODE` for the
resistor (`"up"` for a button to ground) or for `"opendrain"`.

| Command | What it does |
|---|---|
| `GPIO_OPEN([chip$])` | Opens a GPIO chip: without an argument the one with the header, otherwise name (`"gpiochip0"`), number, path or label → `GPIO_CHIP` |
| `GPIO_CHIPS()` | Names of all GPIO chips as an array (`"gpiochip0"`, …) |
| `GPIO_CLOSE(chip)` | Closes the chip and releases all its pins |
| `GPIO_LABEL$(chip)` | Label of the chip, e.g. `"pinctrl-bcm2711"` |
| `GPIO_COUNT(chip)` | Number of lines of the chip |
| `GPIO_LINE_NAME$(chip, pin)` | Name the kernel gives the line (`"GPIO17"`), empty if none |
| `GPIO_FIND(chip, name$)` | Number of the line with this name, -1 if there is none |
| `GPIO_MODE(chip, pin, richtung$ [, widerstand$])` | Direction `"in"`/`"out"`/`"opendrain"`, resistor `"up"`/`"down"`/`"none"` (without: stays as it is) |
| `GPIO_WRITE(chip, pin, wert)` | Sets an output (`TRUE`/`FALSE` or 1/0); a pin that is still free becomes an output |
| `GPIO_READ(chip, pin)` | Reads the pin as `TRUE`/`FALSE`; a pin that is still free becomes an input |
| `GPIO_WATCH(chip, pin, flanke$ [, entprellen_ms])` | Reports edges `"rising"`/`"falling"`/`"both"` as events, `"none"` stops; the kernel debounces on request |
| `GPIO_NEXT(chip)` | Fetches the next event, `FALSE` if none is waiting — never waits |
| `GPIO_EVENT_PIN(chip)` | Pin of the event from `GPIO_NEXT` |
| `GPIO_EVENT_RISING(chip)` | `TRUE` for a rising edge, `FALSE` for a falling one |
| `GPIO_EVENT_TIME(chip)` | Time of the event in seconds (kernel clock, for the gaps between events) |
| `GPIO_RELEASE(chip, pin)` | Releases a pin, another program can then take it |

Words work in German or English: `"ein"`/`"ausgang"`, `"hoch"`/`"runter"`,
`"steigend"`/`"fallend"`/`"beide"`.

### LED and button

```basic
IMPORT "gpio"

DIM chip AS GPIO_CHIP
chip = GPIO_OPEN()
GPIO_MODE(chip, 27, "in", "up")      ' button between GPIO27 and ground

DIM i AS INTEGER
FOR i = 1 TO 10
    GPIO_WRITE(chip, 17, TRUE)       ' LED on GPIO17 (with series resistor)
    SLEEP(200)
    GPIO_WRITE(chip, 17, FALSE)
    SLEEP(200)
    IF NOT GPIO_READ(chip, 27) THEN PRINT "Taster gedrueckt"
NEXT
GPIO_CLOSE(chip)
```

With the resistor `"up"` an open button reads `TRUE` and a pressed one
`FALSE` — hence the `NOT`.

### Events instead of polling

Polling in a loop misses a short press between two polls. `GPIO_WATCH` makes
the kernel keep every edge with its time (at most 1024; beyond that the
oldest is dropped), `GPIO_NEXT` collects them:

```basic
IMPORT "gpio"

DIM chip AS GPIO_CHIP
chip = GPIO_OPEN()
GPIO_MODE(chip, 27, "in", "up")
GPIO_WATCH(chip, 27, "falling", 20)  ' debounce 20 ms

DIM zaehler AS INTEGER
WHILE zaehler < 5
    WHILE GPIO_NEXT(chip)
        zaehler = zaehler + 1
        PRINT "Druck "; zaehler; " um "; GPIO_EVENT_TIME(chip)
    WEND
    SLEEP(10)
WEND
GPIO_CLOSE(chip)
```

## I2C

`I2C_OPEN()` without an argument opens bus 1, the one on the header. The
commands are named and count like those of [`firmata`](module-firmata.md):
when writing, the first byte is the register; when reading, the register is a
separate argument (-1 = none). They talk **SMBus** as long as a request fits
(up to 32 bytes with a register), otherwise raw I2C with a repeated start.

| Command | What it does |
|---|---|
| `I2C_OPEN([bus])` | Opens an I2C bus: number (default 1), `"i2c-1"` or `"/dev/i2c-1"` → `I2C_BUS` |
| `I2C_CLOSE(bus)` | Closes the bus |
| `I2C_WRITE(bus, adresse, daten)` | Sends bytes (array of numbers or BUFFER); the first one is the register |
| `I2C_READ(bus, adresse, register, anzahl)` | Reads `anzahl` bytes from `register` (-1 = without register) as an array |
| `I2C_SCAN(bus)` | Addresses of all devices that answer — like `i2cdetect` |

Addresses above 127 are 10-bit addresses. If nothing answers at an address,
the message says so and points to `I2C_SCAN`.

### Reading a sensor

```basic
IMPORT "i2c"

DIM bus AS I2C_BUS
bus = I2C_OPEN()
PRINT "Gefunden: "; JOIN$(I2C_SCAN(bus), ", ")

I2C_WRITE(bus, &H68, [&H6B, 0])               ' wake up the MPU6050
DIM d AS ARRAY OF INTEGER
d = I2C_READ(bus, &H68, &H3B, 6)              ' acceleration X, Y, Z
DIM x AS INTEGER
x = d[0] * 256 + d[1]
IF x > 32767 THEN x = x - 65536
PRINT "X: "; x
I2C_CLOSE(bus)
```

## SPI

| Command | What it does |
|---|---|
| `SPI_OPEN(bus, cs [, takt_hz [, modus]])` | Opens `/dev/spidev<bus>.<cs>`; clock default 1 MHz, mode 0–3 (default 0) → `SPI_DEVICE` |
| `SPI_CLOSE(spi)` | Closes the device |
| `SPI_TRANSFER(spi, daten)` | Sends bytes and returns what came back at the same time — SPI always exchanges |
| `SPI_WRITE(spi, daten)` | Sends bytes, the answer is dropped |
| `SPI_READ(spi, anzahl)` | Sends zeros and returns the answer |
| `SPI_SPEED(spi, takt_hz)` | Changes the clock |
| `SPI_MODE(spi, modus)` | Changes the mode (0–3, clock polarity and phase) |

### An analogue-to-digital converter (MCP3008)

```basic
IMPORT "spi"

DIM adc AS SPI_DEVICE
adc = SPI_OPEN(0, 0, 1000000)
DIM antwort AS ARRAY OF INTEGER
antwort = SPI_TRANSFER(adc, [1, &H80, 0])     ' channel 0, single-ended
PRINT "Kanal 0: "; (antwort[1] BAND 3) * 256 + antwort[2]
SPI_CLOSE(adc)
```

## Not included

- **PWM** in hardware (dimming an LED, positioning a servo) — the Pi needs an
  overlay and a different kernel interface for it. For servos an Arduino via
  [`firmata`](module-firmata.md) is the simpler way.
- **1-Wire** (DS18B20): with the overlay `w1-gpio` the kernel puts the values
  into files under `/sys/bus/w1/devices/` — `READLINES` reads them.
- Several lines in **one** request (switching at the same time): every pin is
  its own.

## Tested

Without a Pi: the CI loads the kernel's simulators (`gpio-sim` for GPIO,
`i2c-stub` for I2C) on an ordinary Linux machine and runs writing, reading,
resistors, edges with their time, I2C registers and the scan against them
(`tests/pruef/pins.dhtest`). There is no simulator for SPI: what is tested is
the layout of the requests and the messages. On a real Raspberry Pi the module
is **untested**.
