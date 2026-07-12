# Modul `usb`

Zugriff auf USB-HID-Geräte: Custom-Controller, Macro-Pads, Programmer-Boards mit HID-Profil, Tastaturen, Mäuse, Gamepads.

Klassisches „raw USB" über `pyusb` / `libusb` wird **nicht** unterstützt — auf Windows braucht das einen WinUSB-Treiber pro Gerät (Zadig), auf Linux Root-Rechte oder udev-Regeln. HID ist deutlich portabler und für die meisten Maker-Anwendungen ausreichend.

```basic
IMPORT "usb"
```

## Externe Dependency

```
pip install hidapi
```

## Übersicht

| Funktion | Rückgabe |
|---|---|
| `USB_LIST()` | STRING (multi-line `vid:pid|product|manufacturer`) |
| `USB_OPEN(vid, pid)` | USB_HANDLE |
| `USB_OPEN_PATH(pfad$)` | USB_HANDLE |
| `USB_CLOSE(handle)` | — |
| `USB_WRITE(handle, daten$)` | INTEGER (geschriebene Bytes) |
| `USB_READ(handle, n, timeout_ms)` | STRING (Bytes als latin-1) |
| `USB_PRODUCT(handle)` | STRING |
| `USB_MANUFACTURER(handle)` | STRING |
| `USB_SERIAL(handle)` | STRING |

## Bytes ↔ STRING

USB-HID-Reports sind Roh-Bytes. Damit beliebige Bytewerte verlustfrei durch GameBasic-STRINGs laufen, kodiert dieses Modul mit **latin-1** — jeder Codepoint 0..255 entspricht einem Byte:

```basic
DIM data$ AS STRING
data$ = USB_READ(dev, 8, 500)
' data$[0] ist das erste Report-Byte als Zeichen mit Codepoint 0..255
PRINT ASC(data$)         ' erstes Byte als Integer
```

Schreiben funktioniert symmetrisch — der STRING darf Zeichen mit Codepoint 0..255 enthalten:

```basic
USB_WRITE(dev, CHR$(&H01) + CHR$(&HFF) + CHR$(&H00))
```

Wird ein Zeichen >255 übergeben, wirft `USB_WRITE`.

## VID/PID herausfinden

`USB_LIST()` liefert alle gerade erreichbaren HID-Geräte — jeder Aufruf scannt neu (neu angesteckte Geräte tauchen sofort auf, entfernte verschwinden):

```basic
IMPORT "usb"

DIM liste AS STRING
liste = USB_LIST()
PRINT liste
```

Beispiel-Ausgabe:

```
046D:C52B|USB Receiver|Logitech
1234:5678|MyMacroPad|Maker
```

VIDs/PIDs werden im Beispiel-Code typischerweise als Hex-Konstanten geschrieben:

```basic
DIM dev AS USB_HANDLE
dev = USB_OPEN(&H046D, &HC52B)     ' Logitech-Receiver
PRINT "Modell: ", USB_PRODUCT(dev)
PRINT "Hersteller: ", USB_MANUFACTURER(dev)
USB_CLOSE(dev)
```

## Read mit Timeout

`USB_READ(handle, n, timeout_ms)` blockiert bis zu `timeout_ms` Millisekunden auf Daten. `0` = sofort zurückkehren (non-blocking, gut für Game-Loops). Negativer Wert / `-1` = blockierend ohne Timeout. `n` ist auf 64 MiB pro Aufruf begrenzt — ein versehentlich riesiges `n` (z.B. vertauscht mit `timeout_ms`) wirft einen klaren Fehler statt eine Riesenallokation auszulösen.

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

## Berechtigungen

- **Windows**: HID ist für User-Apps freigegeben, kein Treiber-Trick nötig. **Tastaturen / Mäuse** sind allerdings vom System exklusiv geöffnet — `USB_OPEN` schlägt da fehl. Custom-HID-Geräte mit eigenem Verwendungszweck (Usage Page) gehen problemlos.
- **Linux**: hidraw braucht in der Regel udev-Regeln, sonst nur als root.
- **macOS**: ähnlich Windows, Tastaturen/Mäuse sind exklusiv.

## Fehlerbehandlung

`USB_OPEN` wirft mit Hex-VID:PID in der Meldung, wenn das Gerät nicht gefunden / belegt ist:

```basic
TRY
    DIM dev AS USB_HANDLE
    dev = USB_OPEN(&H1234, &H5678)
CATCH e
    PRINT "Geraet nicht erreichbar: ", e
END TRY
```

## Komplettes Beispiel

Siehe [examples/37_usb.gb](../examples/37_usb.gb).

## In der nativen Runtime (gbrt)

`usb` laeuft nativ mit dem Cargo-Feature `usb` (Crate `hidapi`). Bytes ↔ STRING per latin-1 wie im Python-Pfad. Bauen: `python rust/build_runtime.py --hardware` (oder `--full`).
