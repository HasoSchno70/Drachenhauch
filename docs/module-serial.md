# Modul `serial`

Serielle Kommunikation über RS-232 / USB-COM. Typischer Anwendungsfall: Daten von einem Arduino, ESP32 oder einem anderen Mikrocontroller lesen und schreiben.

```basic
IMPORT "serial"
```

## Externe Dependency

```
.venv\Scripts\python.exe -m pip install pyserial
```

Wenn `pyserial` nicht installiert ist, lädt das Modul trotzdem — der erste Aufruf einer `SERIAL_*`-Funktion wirft dann eine klare Meldung mit der `pip`-Anweisung.

## Übersicht

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `SERIAL_PORTS()` | STRING (komma-getrennte Liste verfügbarer Ports) | welche seriellen Anschluesse gibt es? |
| `SERIAL_OPEN(port$, baud)` | SERIAL_HANDLE | Anschluss oeffnen (`baud` muss zur Gegenstelle passen) |
| `SERIAL_CLOSE(handle)` | — | Anschluss schliessen |
| `SERIAL_IS_OPEN(handle)` | BOOLEAN | steht die Verbindung noch? |
| `SERIAL_WRITE(handle, s$)` | INTEGER (geschriebene Bytes) | Text senden |
| `SERIAL_READ(handle, n)` | STRING (bis zu n Bytes, max. 64 MiB) | bis zu n Bytes lesen |
| `SERIAL_READLINE(handle)` | STRING (bis Newline) | eine Zeile lesen -- bis zum Zeilenumbruch oder bis der Timeout greift |
| `SERIAL_AVAILABLE(handle)` | INTEGER (wartende Bytes im Eingangspuffer) | wie viel liegt bereit? (ohne zu warten) |
| `SERIAL_FLUSH(handle)` | — (Eingangs- und Ausgangspuffer leeren) | Puffer verwerfen |
| `SERIAL_TIMEOUT(handle, sekunden)` | — (Read-Timeout setzen, Default 1.0 s) | wie lange ein Lesen hoechstens wartet |

## Bytes ↔ STRING

`SERIAL_READ` und `SERIAL_READLINE` dekodieren als UTF-8 mit Replace-Strategie für nicht-dekodierbare Bytes. Für reine Text-Protokolle (Arduino-Serial-Print, NMEA, AT-Befehle) reicht das. Bei rohem Binär-Protokoll lieber Byte-Wert für Byte-Wert über `MID$` und `ASC` parsen.

`SERIAL_READ` haelt ein Mehrbyte-UTF-8-Zeichen (Umlaut/Sonderzeichen), das genau an einer Lesegrenze zerschnitten ankommt, korrekt bis zum naechsten `SERIAL_READ`-Aufruf zurueck, statt es als `�` anzuzeigen. `n` ist auf 64 MiB pro Aufruf begrenzt — ein versehentlich riesiges `n` (z.B. vertauscht mit einem anderen Parameter) wirft einen klaren Fehler statt eine Riesenallokation auszuloesen. `SERIAL_READLINE` dekodiert dagegen pro Aufruf einmalig lossy (kein Zwischenspeichern ueber Timeouts hinweg).

`SERIAL_WRITE` kodiert den STRING als UTF-8 — pure-ASCII-Bytes (0..127) gehen 1:1 raus, Sonderzeichen werden zu Mehr-Byte-UTF-8-Sequenzen.

## Beispiel — Arduino mit Echo-Loop

```basic
IMPORT "serial"

' Verfuegbare Ports auflisten
PRINT "Ports: ", SERIAL_PORTS()

DIM port AS SERIAL_HANDLE
port = SERIAL_OPEN("COM3", 9600)
SERIAL_TIMEOUT(port, 2.0)

' Befehl senden, Antwort lesen
SERIAL_WRITE(port, "LED ON" + CHR$(10))
PRINT "Antwort: ", SERIAL_READLINE(port)

SERIAL_CLOSE(port)
```

## Polling-Loop ohne Blockieren

`SERIAL_AVAILABLE` zeigt, wie viele Bytes ohne Warten gelesen werden können. Damit lässt sich eine Game-Loop realisieren, die nur dann liest, wenn Daten anliegen:

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

## Fehlerbehandlung

`SERIAL_OPEN` wirft, wenn der Port nicht existiert oder belegt ist. Mit `TRY/CATCH` abfangen:

```basic
TRY
    DIM port AS SERIAL_HANDLE
    port = SERIAL_OPEN("COM_NICHT_DA", 9600)
CATCH e
    PRINT "Konnte Port nicht oeffnen: ", e
END TRY
```

`SERIAL_CLOSE` ist idempotent — doppeltes Schließen wirft nicht.

Schreib- und Lesefehler nach dem Schließen werfen eine deutliche „Port wurde bereits geschlossen"-Meldung.

## Komplettes Beispiel

Siehe [examples/35_serial.dh](../examples/35_serial.dh).

## In der nativen Runtime (dhrt)

`serial` laeuft nativ mit dem Cargo-Feature `serial` (Crate `serialport` — **kein** `pyserial` noetig). Bauen: `python rust/build_runtime.py --hardware` (oder `--full`). Fehlt das Feature, meldet der Builtin „nicht verfuegbar“.
