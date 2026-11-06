# Modul `hotplug`

Melden, wenn ein Gerät **angesteckt** oder **abgezogen** wird: ein Arduino am
USB, ein MIDI-Keyboard, ein Gamecontroller über HID. Ein Programm kann dann
von selbst verbinden, statt dass man es neu startet, oder „Gerät getrennt“
anzeigen, statt bei der nächsten Abfrage abzubrechen.

```basic
IMPORT "hotplug"
```

Beobachtet wird, was dieser Bau kennt (`HOTPLUG_KINDS`):

| Art | Was | braucht |
|---|---|---|
| `"serial"` | serielle Anschlüsse (`COM5`, `/dev/ttyACM0`) | Feature `serial` |
| `"midi_in"`, `"midi_out"` | MIDI-Ein- und -Ausgänge | Feature `midi` |
| `"hid"` | USB-HID-Geräte, als `"VID:PID Produkt"` | Feature `usb` |

Die Namen sind dieselben, die `SERIAL_PORTS`, `MIDI_IN_NAME$` und `USB_LIST`
liefern — wer gemeldet bekommt, dass `COM5` da ist, öffnet `COM5`.

## Befehle

| Funktion | Wirkung |
|---|---|
| `HOTPLUG_NEXT()` → BOOLEAN | die nächste Meldung holen; FALSE, wenn keine da ist. Wartet nie |
| `HOTPLUG_KIND$()` → STRING | Art der geholten Meldung, etwa `"serial"` |
| `HOTPLUG_NAME$()` → STRING | Name des Geräts |
| `HOTPLUG_ADDED()` → BOOLEAN | TRUE = angesteckt, FALSE = abgezogen |
| `HOTPLUG_LIST(art$)` → ARRAY OF STRING | was von dieser Art gerade da ist, geordnet |
| `HOTPLUG_KINDS()` → ARRAY OF STRING | welche Arten dieser Bau beobachtet |
| `HOTPLUG_INTERVAL(ms)` | wie oft nachgesehen wird (100..10000, Vorgabe 500) |

```basic
IMPORT "hotplug"
IMPORT "serial"
DIM s AS SERIAL_HANDLE : s = -1
SCREEN(400, 120, "Arduino")
WHILE NOT QUITREQUESTED()
    WHILE HOTPLUG_NEXT()
        IF HOTPLUG_KIND$() = "serial" AND HOTPLUG_ADDED() AND s < 0 THEN
            s = SERIAL_OPEN(HOTPLUG_NAME$(), 115200)
        END IF
    WEND
    CLS(&H101018)
    TEXT(10, 50, IIF(s < 0, "Arduino anstecken ...", "verbunden"), &HE0E0E0)
    FLIP()
WEND
```

Was beim ersten Blick schon da ist, gilt als Ausgangslage und wird **nicht**
gemeldet — dafür ist `HOTPLUG_LIST`. Der erste Aufruf eines `HOTPLUG_`-Befehls
nimmt die Ausgangslage auf. Höchstens 256 Meldungen warten; die ältesten
fallen weg.

## Wie es arbeitet

Jedes System meldet das auf seine Weise (Windows `WM_DEVICECHANGE`, Linux
udev, macOS IOKit), und keine davon kennt alle drei Arten. Darum sieht ein
eigener Faden alle 500 ms in dieselben Listen und vergleicht mit dem Stand
davor — eine Meldung kommt also bis zu einer halben Sekunde nach dem
Einstecken. Ein HID-Gerät mit mehreren Schnittstellen zählt einmal.

Zum Prüfen ohne Gerät: `DHRT_GERAETE=datei` liest die Listen aus einer
Textdatei statt vom System, eine Zeile je Gerät als `art:name`. Wer die Datei
ändert, sieht die Meldungen (`tests/pruef/hotplug.dhtest`).

**Nicht geprüft:** echtes An- und Abstecken. Unter macOS sieht CoreMIDI neue
MIDI-Geräte womöglich erst, wenn das Programm ein Fenster hat (die
Ereignisschleife läuft dann); ungeprüft.
