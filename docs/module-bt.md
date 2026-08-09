# Modul `bt` — Bluetooth Low Energy

BLE-Geräte scannen, verbinden, GATT-Charakteristiken lesen und schreiben. Typischer Anwendungsfall: Sensoren (Heart Rate, Temperatur, Wearables), Maker-Boards mit Nordic-Chip (nRF52, ESP32-BLE), Smart-Plugs.

**Klassisches Bluetooth (RFCOMM/SPP) wird nicht unterstützt** — die zuständige Lib `pybluez` ist seit Jahren unmaintained. BLE deckt heute praktisch alle relevanten Anwendungsfälle ab.

```basic
IMPORT "bt"
```

## Externe Dependency

```
pip install bleak
```

Auf Windows funktioniert das mit dem eingebauten BT-Stack ab Win 10 — Bluetooth-Adapter muss aktiv sein.

## Übersicht

| Funktion | Rückgabe |
|---|---|
| `BT_SCAN(timeout_sek)` | STRING (multi-line `addr|name|rssi`), 0..300s |
| `BT_CONNECT(addr$)` | BT_HANDLE |
| `BT_DISCONNECT(handle)` | — |
| `BT_IS_CONNECTED(handle)` | BOOLEAN |
| `BT_SERVICES(handle)` | STRING (multi-line UUIDs) |
| `BT_CHARACTERISTICS(handle, svc$)` | STRING (multi-line `uuid|properties`) |
| `BT_READ(handle, char_uuid$)` | STRING (Bytes als latin-1) |
| `BT_WRITE(handle, char_uuid$, daten$)` | — |

## Async/Sync

`bleak` ist asyncio-basiert. Dieses Modul betreibt im Hintergrund einen dedizierten Event-Loop in einem Daemon-Thread und reicht jeden Aufruf synchron durch — der Drachenhauch-VM blockiert pro Aufruf bis zur Antwort. Du musst dich nicht um Coroutinen oder `await` kümmern.

Alle Calls ausser `BT_SCAN` (das sein eigenes, von dir gewähltes Zeitfenster hat) haben ein internes 10-Sekunden-Timeout — ist ein Gerät ausser Reichweite oder antwortet nicht, scheitert `BT_CONNECT`/`BT_READ`/`BT_WRITE`/`BT_SERVICES`/`BT_CHARACTERISTICS` nach spätestens 10s mit einer fangbaren Fehlermeldung, statt den Game-Loop unbegrenzt einzufrieren.

## Bytes ↔ STRING

GATT-Werte sind Roh-Bytes. Wie beim USB-Modul werden sie mit **latin-1** round-trip-fest in Drachenhauch-STRINGs verpackt:

```basic
DIM raw$ AS STRING
raw$ = BT_READ(dev, "00002a19-0000-1000-8000-00805f9b34fb")  ' Battery Level
' raw$ ist 1 Byte; Wert per ASC(...)
PRINT "Akku: ", ASC(raw$), "%"
```

## Beispiel — Scan

```basic
IMPORT "bt"

PRINT "Scanne 5 Sekunden ..."
DIM gefunden AS STRING
gefunden = BT_SCAN(5.0)

DIM zeilen AS ARRAY OF STRING
zeilen = SPLIT$(gefunden, CHR$(10))
DIM i AS INTEGER
FOR i = 0 TO LEN(zeilen) - 1
    PRINT "  ", zeilen[i]
NEXT
```

Beispiel-Ausgabe:

```
AA:BB:CC:DD:EE:FF|My Heart Rate|-67
11:22:33:44:55:66|ESP32-Sensor|-71
```

## Beispiel — verbinden, Characteristic lesen

```basic
IMPORT "bt"

DIM dev AS BT_HANDLE
dev = BT_CONNECT("AA:BB:CC:DD:EE:FF")

' Welche Services hat das Geraet?
PRINT BT_SERVICES(dev)

' Characteristics eines bestimmten Services
PRINT BT_CHARACTERISTICS(dev, "0000180f-0000-1000-8000-00805f9b34fb")

' Battery Level lesen (Standard-UUID 2A19)
DIM lvl AS STRING
lvl = BT_READ(dev, "00002a19-0000-1000-8000-00805f9b34fb")
PRINT "Akku: ", ASC(lvl), "%"

BT_DISCONNECT(dev)
```

## Standard-UUIDs

BLE definiert Service- und Charakteristik-UUIDs als 128-bit-Werte. Standardisierte Profile haben einen 16-bit-Kurzwert (z.B. `2A19` für Battery Level), der auf die Form `0000XXXX-0000-1000-8000-00805f9b34fb` expandiert wird. Einige nützliche:

| UUID | Bedeutung |
|---|---|
| `0000180f-0000-1000-8000-00805f9b34fb` | Battery Service |
| `00002a19-0000-1000-8000-00805f9b34fb` | Battery Level (1 Byte 0..100) |
| `0000180d-0000-1000-8000-00805f9b34fb` | Heart Rate Service |
| `00002a37-0000-1000-8000-00805f9b34fb` | Heart Rate Measurement |
| `0000180a-0000-1000-8000-00805f9b34fb` | Device Information |

Für proprietäre Geräte stehen die UUIDs im Hersteller-Datenblatt.

## Fehlerbehandlung

Alle Operationen werfen `DHRuntimeError` mit aussagekräftiger Meldung. Typische Ursachen: Gerät außer Reichweite, BT-Adapter aus, falsche UUID, Charakteristik unterstützt die angeforderte Operation nicht.

```basic
TRY
    DIM dev AS BT_HANDLE
    dev = BT_CONNECT("AA:BB:CC:DD:EE:FF")
CATCH e
    PRINT "BLE-Verbindung fehlgeschlagen: ", e
END TRY
```

## Komplettes Beispiel

Siehe [examples/38_bt.dh](../examples/38_bt.dh).

## In der nativen Runtime (dhrt)

`bt` laeuft nativ mit dem Cargo-Feature `bt` (Crate `btleplug` statt `bleak`; async wird ueber eine interne tokio-Runtime synchron getrieben). Scan/Connect/Services/Characteristics/Read/Write wie im Python-Pfad; Bytes ↔ STRING per latin-1. **Hinweis:** `BT_CONNECT(addr$)` braucht eine vorher per `BT_SCAN` gesehene Adresse. `BT_SCAN` validiert `timeout_sek` streng (endliche Zahl 0..300) — ein NaN/Infinity-Wert (z.B. aus einer Rechenkette wie `POW(10,1000)`) wirft einen sauberen Fehler statt die Runtime abstuerzen zu lassen. Bauen: `python rust/build_runtime.py --hardware`. Zieht schwere Abhaengigkeiten (tokio/btleplug/windows) — daher nicht im Standard-Dev-Build.
