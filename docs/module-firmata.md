# Modul `firmata`

Direkte Steuerung eines Arduino/ESP32 (oder eines beliebigen anderen mit
[Firmata](https://github.com/firmata/arduino) geflashten Mikrocontrollers) —
ohne eigene Sketch-Logik und ohne ein eigenes Text-Protokoll zu entwerfen.
Einmalig in der Arduino-IDE **Datei → Beispiele → Firmata → StandardFirmata**
hochladen, danach steuert Drachenhauch Pins, Servos und I2C-Geräte direkt.

```basic
IMPORT "firmata"
```

Nativ in dhrt, in **jedem** Bau: über das Netz (`tcp:host:port`) braucht das
Modul kein Feature. Nur die serielle Leitung braucht die `serialport`-Crate
(Feature `serial`, wie das [`serial`](module-serial.md)-Modul) — bauen mit
`python rust/build_runtime.py --hardware`. Ohne das Feature meldet
`FIRMATA_OPEN("COM3", …)`, dass es nur `tcp:` gibt.

## Zwei Leitungen

- **Seriell**: `FIRMATA_OPEN("COM3", 57600)` bzw. `/dev/ttyUSB0`. StandardFirmata
  spricht 57600 Baud. Das Öffnen wartet zwei Sekunden, weil viele Boards
  beim Öffnen des Ports neu starten.
- **Netz**: `FIRMATA_OPEN("tcp:192.168.1.50:3030")` — für ein Board mit
  **StandardFirmataWiFi** (ESP32, ESP8266, Arduino mit WLAN) oder
  StandardFirmataEthernet; beide warten auf Port 3030. Eine Baudrate gibt es
  hier nicht, das zweite Argument darf fehlen, und gewartet wird nicht.

## Was das Modul kann

Pins (digital, analog, PWM), **Servos**, **I2C** (schreiben, einmal oder
fortlaufend lesen, 7- und 10-Bit-Adressen), die **Firmware-Abfrage** und
**Schrittmotoren**. Die ersten vier stecken in StandardFirmata. Schrittmotoren
nicht: dafür braucht das Board **ConfigurableFirmata** mit AccelStepperFirmata
(Arduino-IDE: Bibliothek „ConfigurableFirmata" installieren, Beispiel
„ConfigurableFirmata" hochladen). OneWire, Encoder, Scheduler und die
Capability-Abfrage aus dem vollen Protokoll sind nicht dabei.

## Übersicht

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `FIRMATA_PORTS()` | STRING (komma-getrennte Liste verfügbarer Ports, wie `SERIAL_PORTS()`) | welche seriellen Anschluesse gibt es? |
| `FIRMATA_OPEN(ziel$ [, baud])` | FIRMATA_HANDLE | Verbindung zum Board aufbauen: serieller Port oder `tcp:host:port` |
| `FIRMATA_CLOSE(handle)` | — | Verbindung schliessen |
| `FIRMATA_IS_OPEN(handle)` | BOOLEAN | steht die Verbindung noch? |
| `FIRMATA_FIRMWARE$(handle)` | STRING | Name und Fassung der Firmware, etwa `StandardFirmata.ino 2.5` |
| `FIRMATA_PIN_MODE(handle, pin, modus)` | — | Betriebsart eines Pins festlegen (Eingang, Ausgang, PWM, Analog) |
| `FIRMATA_DIGITAL_WRITE(handle, pin, wert)` | — | Pin auf HIGH oder LOW schalten |
| `FIRMATA_DIGITAL_READ(handle, pin)` | BOOLEAN | gemeldeten Zustand eines Eingangs lesen -- der Wert kommt aus dem letzten `FIRMATA_UPDATE` |
| `FIRMATA_ANALOG_WRITE(handle, pin, wert)` | — | PWM-Wert ausgeben (0..255), etwa fuer Helligkeit oder Motortempo |
| `FIRMATA_ANALOG_READ(handle, kanal)` | INTEGER (0..16383) | Analogwert lesen -- Achtung, hier zaehlt der **Kanal** (A0 = 0), nicht die Pin-Nummer |
| `FIRMATA_SERVO_ATTACH(handle, pin [, min_us, max_us])` | — | Servo an einen Pin haengen; die Pulsbreiten fuer 0 und 180 Grad (Vorgabe 544 und 2400) |
| `FIRMATA_SERVO_WRITE(handle, pin, winkel)` | — | Servo auf einen Winkel 0..180 stellen |
| `FIRMATA_I2C_CONFIG(handle [, verzoegerung_us])` | — | I2C einschalten; die erste I2C-Anfrage tut das sonst selbst |
| `FIRMATA_I2C_WRITE(handle, adresse, bytes)` | — | Bytes an ein I2C-Geraet schicken (Feld aus Zahlen oder BUFFER) |
| `FIRMATA_I2C_READ(handle, adresse, register, anzahl)` | ARRAY OF INTEGER | lesen und auf die Antwort warten (hoechstens eine Sekunde); Register -1 = ohne |
| `FIRMATA_I2C_REQUEST(handle, adresse, register, anzahl [, dauernd])` | — | lesen lassen, ohne zu warten; mit `dauernd` meldet das Board immer wieder |
| `FIRMATA_I2C_DATA(handle, adresse [, register])` | ARRAY OF INTEGER | die zuletzt gemeldeten Bytes, leer wenn noch nichts kam |
| `FIRMATA_I2C_STOP(handle, adresse)` | — | fortlaufendes Lesen beenden |
| `FIRMATA_STEPPER_CONFIG(handle, motor, art$, pin1, pin2 [, pin3, pin4])` | — | Schrittmotor 0..9 einrichten: `treiber` (Schritt + Richtung), `zweidraht`, `dreidraht`, `vierdraht` |
| `FIRMATA_STEPPER_SPEED(handle, motor, schritte_je_s)` | — | Hoechstgeschwindigkeit |
| `FIRMATA_STEPPER_ACCEL(handle, motor, schritte_je_s2)` | — | Beschleunigung, 0 = ohne |
| `FIRMATA_STEPPER_MOVE(handle, motor, schritte)` | — | um so viele Schritte fahren, das Vorzeichen ist die Richtung |
| `FIRMATA_STEPPER_TO(handle, motor, ziel)` | — | zu einer Lage fahren |
| `FIRMATA_STEPPER_STOP(handle, motor)` | — | anhalten |
| `FIRMATA_STEPPER_ZERO(handle, motor)` | — | die jetzige Lage wird 0 |
| `FIRMATA_STEPPER_POSITION(handle, motor)` | INTEGER | zuletzt gemeldete Lage (fragt zugleich nach der naechsten) |
| `FIRMATA_STEPPER_RUNNING(handle, motor)` | BOOLEAN | faehrt der Motor noch? TRUE bis das Board „fertig" meldet |
| `FIRMATA_UPDATE(handle)` | — | **pro Frame aufrufen**: holt Pin-Zustaende, I2C-Antworten und Motor-Meldungen ab |

## Pin-Modi

`FIRMATA_PIN_MODE` nimmt rohe Firmata-Modus-Werte (keine Drachenhauch-Konstanten,
analog zu `JOYSTICK_BUTTON`s rohen raylib-Indizes):

| Wert | Modus |
|---|---|
| 0 | INPUT |
| 1 | OUTPUT |
| 2 | ANALOG (Pin als Analog-Eingang) |
| 3 | PWM |
| 4 | SERVO (setzt `FIRMATA_SERVO_ATTACH` selbst) |
| 6 | I2C (setzt `FIRMATA_I2C_CONFIG` selbst) |
| 11 | PULLUP (Digital-Eingang mit internem Pullup-Widerstand) |

## Wichtig: zwei verschiedene Nummerierungen

Das ist eine echte Eigenheit des Firmata-Protokolls selbst (gegen
StandardFirmata.ino verifiziert, nicht geraten) — **Schreiben und Lesen
sprechen nicht dieselbe Nummer für denselben physischen Pin:**

- `FIRMATA_PIN_MODE`, `FIRMATA_DIGITAL_WRITE`, `FIRMATA_DIGITAL_READ`,
  `FIRMATA_ANALOG_WRITE`, `FIRMATA_SERVO_*` nehmen die **rohe digitale
  Pin-Nummer** (z. B. Pin 9, 10, 11 für PWM auf einem Uno).
- `FIRMATA_ANALOG_READ` nimmt den **Analog-Kanal** (A0 = 0, A1 = 1, A2 = 2, …)
  — NICHT dieselbe Nummer wie die digitale Pin-Bezeichnung des Boards.

`FIRMATA_ANALOG_READ` geht bis Kanal 15 (das Firmata-Kommando-Byte hat dafür
nur ein 4-Bit-Nibble Platz). Geschrieben werden auch höhere Pins:
`FIRMATA_ANALOG_WRITE` und `FIRMATA_SERVO_WRITE` nehmen dort das
EXTENDED_ANALOG-Kommando (ein ESP32 hat PWM auf Pin 25 und höher).

## Servo

`FIRMATA_SERVO_ATTACH` hängt den Servo an (StandardFirmata stellt den Pin dabei
selbst auf SERVO), danach stellt `FIRMATA_SERVO_WRITE` den Winkel. Die
Pulsbreiten sind die der Arduino-Servo-Bibliothek; ein Servo, der bei 0 oder
180 Grad brummt und nicht ganz hinkommt, braucht andere (oft 1000 und 2000).

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

Ein I2C-Gerät hat eine Adresse (ein OLED-Display meist `&H3C`, ein MPU6050
`&H68`) und meist Register, aus denen man liest. `FIRMATA_I2C_READ` fragt und
wartet — bequem für Programme ohne Bildschleife. Ein Spiel fragt mit
`FIRMATA_I2C_REQUEST` und holt die Antwort in einem späteren Bild mit
`FIRMATA_I2C_DATA`; mit `dauernd` = TRUE liest das Board von selbst weiter.

```basic
IMPORT "firmata"

DIM board AS FIRMATA_HANDLE
board = FIRMATA_OPEN("COM3", 57600)

' MPU6050 aufwecken: Register 0x6B auf 0
FIRMATA_I2C_WRITE(board, &H68, [&H6B, 0])

' Beschleunigung X: zwei Bytes ab Register 0x3B, das hohe zuerst
DIM b AS ARRAY OF INTEGER
b = FIRMATA_I2C_READ(board, &H68, &H3B, 2)
DIM x AS INTEGER : x = b[0] * 256 + b[1]
IF x >= 32768 THEN x = x - 65536
PRINT "X: "; x
```

Fortlaufend in der Bildschleife:

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

Liest man ohne Register (`-1`), meldet StandardFirmata die Antwort unter
Register 0; `FIRMATA_I2C_DATA(board, adresse)` ohne Register nimmt die neueste
Antwort der Adresse, gleich unter welchem Register.

## Schrittmotor

Schrittmotoren braucht **ConfigurableFirmata** auf dem Board (siehe oben);
StandardFirmata kennt sie nicht und antwortet nicht. Bis zu zehn Motoren
(0..9); Art `treiber` für einen Treiber mit Schritt- und Richtungs-Pin
(A4988, DRV8825, TMC2208), `vierdraht` für einen 28BYJ-48 an einem ULN2003.
Der Motor fährt auf dem Board weiter, auch wenn das Programm etwas anderes
tut; `FIRMATA_STEPPER_RUNNING` sagt, ob er noch fährt.

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

`FIRMATA_OPEN` wirft, wenn der Port nicht existiert oder belegt ist bzw. das
Board im Netz nicht antwortet — mit `TRY/CATCH` abfangen, gleiches Muster wie
`SERIAL_OPEN`. Legt das Board auf (Strom weg, WLAN weg), wirft das nächste
`FIRMATA_UPDATE` „die Verbindung zum Board ist beendet".

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

Siehe [examples/147_firmata.dh](../examples/147_firmata.dh).
