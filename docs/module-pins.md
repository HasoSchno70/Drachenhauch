# Module `gpio`, `i2c` und `spi`

Die Pins eines **Raspberry Pi** direkt aus Drachenhauch — ohne Arduino
dazwischen: LEDs und Relais schalten, Taster lesen, I2C-Sensoren abfragen,
SPI-Bausteine ansprechen.

```basic
IMPORT "gpio"
IMPORT "i2c"
IMPORT "spi"
```

Die drei Module sprechen mit den Schnittstellen des **Linux-Kernels**
(`/dev/gpiochipN`, `/dev/i2c-N`, `/dev/spidevB.C`), nicht mit den Registern
des Pi-Chips. Darum gehen sie auf jedem Pi — 3, 4 und 5 haben drei
verschiedene GPIO-Chips, der Pi 5 seinen RP1 hinter PCIe — und auf jedem
anderen Linux-Rechner mit Pins (Orange Pi, BeagleBone, Rock Pi …). Sie stecken
unter Linux in **jedem** Bau von dhrt, ohne Feature. Unter Windows und macOS
gibt es die Befehle auch, jeder meldet aber „nur unter Linux";
`MODULE_AVAILABLE("gpio")` sagt vorher, ob es geht.

Für einen Arduino oder ESP32 am USB-Kabel oder im WLAN ist
[`firmata`](module-firmata.md) der Weg.

## Vorbereitung auf dem Pi

- **GPIO** geht sofort. Der Benutzer muss in der Gruppe `gpio` sein (auf
  Raspberry Pi OS ist er das von Anfang an).
- **I2C** und **SPI** sind ausgeschaltet: `sudo raspi-config` → *Interface
  Options* → *I2C* bzw. *SPI* → einschalten, neu starten. Danach gibt es
  `/dev/i2c-1` und `/dev/spidev0.0`/`/dev/spidev0.1`. Gruppen: `i2c` und `spi`.

Fehlt eine Berechtigung oder ist eine Schnittstelle aus, sagt die Meldung,
was zu tun ist.

## Welche Nummer?

Gezählt wird nach den **GPIO-Nummern** (auch „BCM-Nummern"), nicht nach der
Lage auf der Stiftleiste: der Pin, der auf Belegungsplänen „GPIO17" heißt
(Stift 11), ist Pin 17. `GPIO_FIND(chip, "GPIO17")` sucht die Nummer über den
Namen, den der Kernel der Leitung gibt, und `GPIO_LINE_NAME$` nennt ihn.

| Stift | GPIO | oft benutzt für |
|---|---|---|
| 3, 5 | 2, 3 | I2C (SDA, SCL) — nicht als GPIO nehmen, wenn I2C an ist |
| 11 | 17 | LED, Taster |
| 12 | 18 | LED, Summer |
| 19, 21, 23, 24 | 10, 9, 11, 8 | SPI (MOSI, MISO, SCLK, CE0) |

## GPIO

`GPIO_OPEN()` ohne Argument nimmt den Chip mit der Stiftleiste
(`pinctrl-rp1` auf dem Pi 5, `pinctrl-bcm2711` auf dem Pi 4,
`pinctrl-bcm2835` auf Pi 3 und Zero). Ein Pin, den man beschreibt, wird von
selbst ein Ausgang; einer, den man liest, ein Eingang. `GPIO_MODE` braucht
man für den Widerstand (`"up"` für einen Taster gegen Masse) oder für
`"opendrain"`.

| Befehl | Was es tut |
|---|---|
| `GPIO_OPEN([chip$])` | Öffnet einen GPIO-Chip: ohne Argument den mit der Stiftleiste, sonst Name (`"gpiochip0"`), Nummer, Pfad oder Beschriftung → `GPIO_CHIP` |
| `GPIO_CHIPS()` | Namen aller GPIO-Chips als Feld (`"gpiochip0"`, …) |
| `GPIO_CLOSE(chip)` | Schließt den Chip und gibt alle seine Pins frei |
| `GPIO_LABEL$(chip)` | Beschriftung des Chips, etwa `"pinctrl-bcm2711"` |
| `GPIO_COUNT(chip)` | Zahl der Leitungen des Chips |
| `GPIO_LINE_NAME$(chip, pin)` | Name, den der Kernel der Leitung gibt (`"GPIO17"`), leer wenn keiner |
| `GPIO_FIND(chip, name$)` | Nummer der Leitung mit diesem Namen, -1 wenn es keine gibt |
| `GPIO_MODE(chip, pin, richtung$ [, widerstand$])` | Richtung `"in"`/`"out"`/`"opendrain"`, Widerstand `"up"`/`"down"`/`"none"` (ohne: bleibt, wie er ist) |
| `GPIO_WRITE(chip, pin, wert)` | Setzt einen Ausgang (`TRUE`/`FALSE` oder 1/0); ein noch freier Pin wird dabei Ausgang |
| `GPIO_READ(chip, pin)` | Liest den Pin als `TRUE`/`FALSE`; ein noch freier Pin wird dabei Eingang |
| `GPIO_WATCH(chip, pin, flanke$ [, entprellen_ms])` | Meldet Flanken `"rising"`/`"falling"`/`"both"` als Ereignisse, `"none"` hört auf; der Kernel entprellt auf Wunsch |
| `GPIO_NEXT(chip)` | Holt das nächste Ereignis, `FALSE` wenn keins wartet — wartet nie |
| `GPIO_EVENT_PIN(chip)` | Pin des Ereignisses aus `GPIO_NEXT` |
| `GPIO_EVENT_RISING(chip)` | `TRUE` bei steigender Flanke, `FALSE` bei fallender |
| `GPIO_EVENT_TIME(chip)` | Zeitpunkt des Ereignisses in Sekunden (Uhr des Kernels, für Abstände zwischen Ereignissen) |
| `GPIO_RELEASE(chip, pin)` | Gibt einen Pin frei, ein anderes Programm kann ihn dann nehmen |

Wörter gehen deutsch oder englisch: `"ein"`/`"ausgang"`, `"hoch"`/`"runter"`,
`"steigend"`/`"fallend"`/`"beide"`.

### LED und Taster

```basic
IMPORT "gpio"

DIM chip AS GPIO_CHIP
chip = GPIO_OPEN()
GPIO_MODE(chip, 27, "in", "up")      ' Taster zwischen GPIO27 und Masse

DIM i AS INTEGER
FOR i = 1 TO 10
    GPIO_WRITE(chip, 17, TRUE)       ' LED an GPIO17 (mit Vorwiderstand)
    SLEEP(200)
    GPIO_WRITE(chip, 17, FALSE)
    SLEEP(200)
    IF NOT GPIO_READ(chip, 27) THEN PRINT "Taster gedrueckt"
NEXT
GPIO_CLOSE(chip)
```

Mit dem Widerstand `"up"` liest ein offener Taster `TRUE` und ein gedrückter
`FALSE` — darum das `NOT`.

### Ereignisse statt Abfragen

Wer in einer Schleife fragt, verpasst einen kurzen Druck zwischen zwei
Abfragen. `GPIO_WATCH` lässt den Kernel jede Flanke mit Zeitpunkt aufheben
(höchstens 1024; darüber fällt die älteste weg), `GPIO_NEXT` holt sie ab:

```basic
IMPORT "gpio"

DIM chip AS GPIO_CHIP
chip = GPIO_OPEN()
GPIO_MODE(chip, 27, "in", "up")
GPIO_WATCH(chip, 27, "falling", 20)  ' 20 ms entprellen

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

`I2C_OPEN()` ohne Argument öffnet Bus 1, den an der Stiftleiste. Die Befehle
heißen und rechnen wie bei [`firmata`](module-firmata.md): beim Schreiben ist
das erste Byte das Register, beim Lesen steht das Register extra (-1 = ohne).
Gesprochen wird über **SMBus**, solange ein Auftrag hineinpasst (bis 32 Bytes
mit Register), sonst über rohes I2C mit wiederholtem Start.

| Befehl | Was es tut |
|---|---|
| `I2C_OPEN([bus])` | Öffnet einen I2C-Bus: Nummer (Vorgabe 1), `"i2c-1"` oder `"/dev/i2c-1"` → `I2C_BUS` |
| `I2C_CLOSE(bus)` | Schließt den Bus |
| `I2C_WRITE(bus, adresse, daten)` | Schickt Bytes (Feld aus Zahlen oder BUFFER); das erste ist das Register |
| `I2C_READ(bus, adresse, register, anzahl)` | Liest `anzahl` Bytes ab `register` (-1 = ohne Register) als Feld |
| `I2C_SCAN(bus)` | Adressen aller Bausteine, die antworten — wie `i2cdetect` |

Adressen über 127 sind 10-Bit-Adressen. Antwortet an einer Adresse nichts,
sagt die Meldung das und verweist auf `I2C_SCAN`.

### Einen Sensor lesen

```basic
IMPORT "i2c"

DIM bus AS I2C_BUS
bus = I2C_OPEN()
PRINT "Gefunden: "; JOIN$(I2C_SCAN(bus), ", ")

I2C_WRITE(bus, &H68, [&H6B, 0])               ' MPU6050 aufwecken
DIM d AS ARRAY OF INTEGER
d = I2C_READ(bus, &H68, &H3B, 6)              ' Beschleunigung X, Y, Z
DIM x AS INTEGER
x = d[0] * 256 + d[1]
IF x > 32767 THEN x = x - 65536
PRINT "X: "; x
I2C_CLOSE(bus)
```

## SPI

| Befehl | Was es tut |
|---|---|
| `SPI_OPEN(bus, cs [, takt_hz [, modus]])` | Öffnet `/dev/spidev<bus>.<cs>`; Takt Vorgabe 1 MHz, Modus 0–3 (Vorgabe 0) → `SPI_DEVICE` |
| `SPI_CLOSE(spi)` | Schließt das Gerät |
| `SPI_TRANSFER(spi, daten)` | Schickt Bytes und liefert, was gleichzeitig zurückkam — SPI tauscht immer |
| `SPI_WRITE(spi, daten)` | Schickt Bytes, die Antwort fällt weg |
| `SPI_READ(spi, anzahl)` | Schickt Nullen und liefert die Antwort |
| `SPI_SPEED(spi, takt_hz)` | Ändert den Takt |
| `SPI_MODE(spi, modus)` | Ändert den Modus (0–3, Taktpolarität und -phase) |

### Ein Analog-Digital-Wandler (MCP3008)

```basic
IMPORT "spi"

DIM adc AS SPI_DEVICE
adc = SPI_OPEN(0, 0, 1000000)
DIM antwort AS ARRAY OF INTEGER
antwort = SPI_TRANSFER(adc, [1, &H80, 0])     ' Kanal 0, einzeln
PRINT "Kanal 0: "; (antwort[1] BAND 3) * 256 + antwort[2]
SPI_CLOSE(adc)
```

## Nicht dabei

- **PWM** in Hardware (eine LED dimmen, ein Servo stellen) — der Pi braucht
  dafür ein Overlay und eine andere Schnittstelle des Kernels. Für Servos
  ist ein Arduino über [`firmata`](module-firmata.md) der einfachere Weg.
- **1-Wire** (DS18B20): mit dem Overlay `w1-gpio` legt der Kernel die Werte
  als Dateien unter `/sys/bus/w1/devices/` ab — die liest `READLINES`.
- Mehrere Leitungen in **einem** Auftrag (gleichzeitig umschalten): jeder Pin
  ist ein eigener.

## Geprüft

Ohne Pi: die CI lädt die Simulatoren des Kernels (`gpio-sim` für GPIO,
`i2c-stub` für I2C) auf einem gewöhnlichen Linux-Rechner und fährt
Schreiben, Lesen, Widerstände, Flanken samt Zeitpunkt, I2C-Register und den
Scan dagegen (`tests/pruef/pins.dhtest`). Für SPI gibt es keinen Simulator:
geprüft sind die Aufbauten der Aufträge und die Meldungen. Auf einem echten
Raspberry Pi ist das Modul **ungeprüft**.
