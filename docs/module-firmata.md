# Modul `firmata`

Direkte Pin-Steuerung eines Arduino/ESP32 (oder eines beliebigen anderen mit
[StandardFirmata](https://github.com/firmata/arduino) geflashten
Mikrocontrollers) über eine bestehende serielle Verbindung — ohne eigene
Sketch-Logik und ohne ein eigenes Text-Protokoll zu entwerfen. Einmalig in der
Arduino-IDE **Datei → Beispiele → Firmata → StandardFirmata** hochladen,
danach steuert GameBasic die Pins direkt.

```basic
IMPORT "firmata"
```

Nativ in gbrt (Feature `serial` — dieselbe `serialport`-Abhängigkeit wie das
[`serial`](module-serial.md)-Modul, keine zusätzliche Dependency). Bauen:
`python rust/build_runtime.py --hardware` (oder `--full`). Fehlt das Feature,
meldet jeder `FIRMATA_*`-Aufruf „nicht verfügbar" mit dem Bau-Hinweis.

## Bewusst nicht abgedeckt

Nur die Pin-I/O-Basis des Firmata-Protokolls: Pin-Modus, Digital- und
Analog-I/O. **I2C, Servo, OneWire, Stepper, Encoder und die
Capability-Query-SysEx** aus dem vollen Firmata-Sprachumfang sind nicht
implementiert — für die meisten Bastler-Projekte (LEDs, Taster, Potis,
Sensoren, einfache Motoren über PWM) reicht die Pin-I/O-Basis.

## Übersicht

| Funktion | Rückgabe |
|---|---|
| `FIRMATA_PORTS()` | STRING (komma-getrennte Liste verfügbarer Ports, wie `SERIAL_PORTS()`) |
| `FIRMATA_OPEN(port$, baud)` | FIRMATA_HANDLE |
| `FIRMATA_CLOSE(handle)` | — |
| `FIRMATA_IS_OPEN(handle)` | BOOLEAN |
| `FIRMATA_PIN_MODE(handle, pin, modus)` | — |
| `FIRMATA_DIGITAL_WRITE(handle, pin, wert)` | — |
| `FIRMATA_DIGITAL_READ(handle, pin)` | BOOLEAN |
| `FIRMATA_ANALOG_WRITE(handle, pin, wert)` | — |
| `FIRMATA_ANALOG_READ(handle, kanal)` | INTEGER (0..16383) |
| `FIRMATA_UPDATE(handle)` | — |

## Pin-Modi

`FIRMATA_PIN_MODE` nimmt rohe Firmata-Modus-Werte (keine GameBasic-Konstanten,
analog zu `JOYSTICK_BUTTON`s rohen raylib-Indizes):

| Wert | Modus |
|---|---|
| 0 | INPUT |
| 1 | OUTPUT |
| 2 | ANALOG (Pin als Analog-Eingang) |
| 3 | PWM |
| 11 | PULLUP (Digital-Eingang mit internem Pullup-Widerstand) |

## Wichtig: zwei verschiedene Nummerierungen

Das ist eine echte Eigenheit des Firmata-Protokolls selbst (gegen
StandardFirmata.ino verifiziert, nicht geraten) — **Schreiben und Lesen
sprechen nicht dieselbe Nummer für denselben physischen Pin:**

- `FIRMATA_PIN_MODE`, `FIRMATA_DIGITAL_WRITE`, `FIRMATA_DIGITAL_READ`,
  `FIRMATA_ANALOG_WRITE` nehmen die **rohe digitale Pin-Nummer** (z. B. Pin 9,
  10, 11 für PWM auf einem Uno).
- `FIRMATA_ANALOG_READ` nimmt den **Analog-Kanal** (A0 = 0, A1 = 1, A2 = 2, …)
  — NICHT dieselbe Nummer wie die digitale Pin-Bezeichnung des Boards.

`FIRMATA_ANALOG_WRITE`/`FIRMATA_ANALOG_READ` sind zusätzlich auf Pin/Kanal
0..15 begrenzt (das Firmata-Kommando-Byte hat dafür nur ein 4-Bit-Nibble
Platz; höhere Analog-Pins bräuchten das hier nicht implementierte
EXTENDED_ANALOG-SysEx-Kommando).

## Beispiel — LED blinken lassen

```basic
IMPORT "firmata"

DIM board AS FIRMATA_HANDLE
board = FIRMATA_OPEN("COM3", 57600)
' FIRMATA_OPEN blockiert ~2 Sekunden -- viele Boards resetten sich beim
' Oeffnen des Ports (DTR-Toggle) und brauchen eine kurze Bootzeit, bevor
' StandardFirmata Kommandos entgegennimmt. Bekannte Firmata-Eigenheit.

FIRMATA_PIN_MODE(board, 13, 1)   ' Pin 13 = OUTPUT (eingebaute LED auf vielen Boards)

DIM an AS BOOLEAN
an = TRUE
WHILE NOT QUITREQUESTED()
    FIRMATA_DIGITAL_WRITE(board, 13, an)
    an = NOT an
    SLEEP(500)
WEND

FIRMATA_CLOSE(board)
```

## Beispiel — Taster lesen + Poti auslesen

`FIRMATA_UPDATE` muss **pro Frame** aufgerufen werden (wie
`INPUT_UPDATE()`/`TIMER_UPDATE()`) — es liest alle aktuell verfügbaren Bytes
nicht-blockierend und aktualisiert die Digital-/Analog-Caches. Ohne
`FIRMATA_UPDATE` bleiben `FIRMATA_DIGITAL_READ`/`FIRMATA_ANALOG_READ` auf dem
letzten bekannten Stand stehen.

```basic
IMPORT "firmata"

DIM board AS FIRMATA_HANDLE
board = FIRMATA_OPEN("COM3", 57600)

FIRMATA_PIN_MODE(board, 2, 11)   ' Taster an Pin 2, PULLUP (gedrueckt = FALSE)
' Kanal 0 = A0 -- Poti an A0 angeschlossen, kein PIN_MODE noetig fuer reine Analog-Reads.

WHILE NOT QUITREQUESTED()
    FIRMATA_UPDATE(board)

    IF NOT FIRMATA_DIGITAL_READ(board, 2) THEN
        PRINT "Taster gedrueckt"
    END IF
    PRINT "Poti: ", FIRMATA_ANALOG_READ(board, 0)

    SLEEP(16)
WEND
```

## Fehlerbehandlung

`FIRMATA_OPEN` wirft, wenn der Port nicht existiert oder belegt ist — mit
`TRY/CATCH` abfangen, gleiches Muster wie `SERIAL_OPEN`:

```basic
TRY
    DIM board AS FIRMATA_HANDLE
    board = FIRMATA_OPEN("COM_NICHT_DA", 57600)
CATCH e
    PRINT "Konnte Board nicht oeffnen: ", e
END TRY
```

`FIRMATA_CLOSE` ist idempotent — doppeltes Schließen wirft nicht.

## Komplettes Beispiel

Siehe [examples/147_firmata.gb](../examples/147_firmata.gb).
