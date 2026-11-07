# Modul `modbus`

**Modbus** ist das Protokoll, das fast alle Geräte der Gebäude- und
Industrietechnik sprechen: Stromzähler, Wechselrichter, Batteriespeicher,
Heizungs- und Lüftungsregler, Frequenzumrichter, SPS. Mit diesem Modul liest
und schreibt ein Drachenhauch-Programm ihre Werte.

```basic
IMPORT "modbus"
```

Ein Gerät hat vier Bereiche, jeweils ab Adresse 0 gezählt:

| Bereich | Inhalt | lesen | schreiben |
|---|---|---|---|
| Spulen (coils) | einzelne Schalter | `MODBUS_READ_COILS` | `MODBUS_WRITE_COIL`, `MODBUS_WRITE_COILS` |
| Eingänge (discrete inputs) | einzelne Meldungen | `MODBUS_READ_DISCRETE` | — |
| Halteregister (holding) | 16-Bit-Werte, auch Einstellungen | `MODBUS_READ_HOLDING` | `MODBUS_WRITE_REGISTER`, `MODBUS_WRITE_REGISTERS` |
| Eingangsregister (input) | 16-Bit-Messwerte | `MODBUS_READ_INPUT` | — |

Welche Adresse was bedeutet, steht im Datenblatt des Geräts. Manche
Datenblätter zählen ab 1 oder schreiben die Bereichsziffer davor
(„40001“ = Halteregister 0) — dann eins bzw. den Vorsatz abziehen.

## Verbinden

| Funktion | Wirkung |
|---|---|
| `MODBUS_TCP(host$ [, port [, einheit]])` → MODBUS | Modbus TCP über das Netz; Port Vorgabe 502, Geräteadresse Vorgabe 1 |
| `MODBUS_RTU(anschluss$ [, baud [, format$ [, einheit]]])` → MODBUS | Modbus RTU über RS-485: `"COM4"`/`"/dev/ttyUSB0"` (Feature `serial`), Baud Vorgabe 9600, Format Vorgabe `"8N1"` (`"8E1"`, `"8O1"`, `"8N2"`, ...); oder `"tcp:host:port"` für einen RTU-Umsetzer im Netz |
| `MODBUS_UNIT(m, einheit)` | Geräteadresse für die folgenden Anfragen (0..255; bei RTU heißt 0 „an alle“, dann kommt keine Antwort) |
| `MODBUS_TIMEOUT(m, ms)` | wie lange auf eine Antwort gewartet wird (10..60000, Vorgabe 1000) |
| `MODBUS_CLOSE(m)` | schließen |

Mehrere Geräte an einem RS-485-Bus teilen sich eine Verbindung — vor jeder
Anfrage `MODBUS_UNIT` umstellen.

```basic
IMPORT "modbus"
DIM zaehler AS MODBUS
zaehler = MODBUS_TCP("192.168.1.60")
DIM r AS ARRAY OF INTEGER
r = MODBUS_READ_INPUT(zaehler, 0, 2)
PRINT "Leistung: "; MODBUS_FLOAT(r, 0); " W"
MODBUS_CLOSE(zaehler)
```

## Lesen und schreiben

| Funktion | Wirkung |
|---|---|
| `MODBUS_READ_COILS(m, adresse, anzahl)` → ARRAY OF BOOLEAN | Spulen lesen (1..2000) |
| `MODBUS_READ_DISCRETE(m, adresse, anzahl)` → ARRAY OF BOOLEAN | Eingänge lesen (1..2000) |
| `MODBUS_READ_HOLDING(m, adresse, anzahl)` → ARRAY OF INTEGER | Halteregister lesen (1..125), Werte 0..65535 |
| `MODBUS_READ_INPUT(m, adresse, anzahl)` → ARRAY OF INTEGER | Eingangsregister lesen (1..125) |
| `MODBUS_WRITE_COIL(m, adresse, an)` | eine Spule setzen |
| `MODBUS_WRITE_REGISTER(m, adresse, wert)` | ein Halteregister schreiben (0..65535 oder −32768..−1) |
| `MODBUS_WRITE_COILS(m, adresse, werte)` | mehrere Spulen (Feld aus BOOLEAN oder 0/1, bis 1968) |
| `MODBUS_WRITE_REGISTERS(m, adresse, werte)` | mehrere Halteregister (Feld aus Zahlen, bis 123) |

Jeder Befehl **wartet auf die Antwort**; ein Gerät antwortet in Millisekunden.
Meldet das Gerät einen Fehler (eine „Ausnahme“), ist das ein Laufzeitfehler
mit der Nummer und ihrer Bedeutung, etwa *Ausnahme 2 (unerlaubte
Datenadresse)*. Bei RTU wird die Prüfsumme jeder Antwort nachgerechnet; eine
falsche ist ein Fehler, keine falsche Zahl.

## 32-Bit-Werte und Vorzeichen

Ein Register hat 16 Bit. Größere Werte und Kommazahlen liegen in **zwei**
Registern — und in welcher Reihenfolge, legt die Spezifikation nicht fest:
jeder Hersteller wählt selbst, das Datenblatt sagt welche. Die Reihenfolge
heißt hier wie üblich nach den vier Bytes A (höchstes) bis D:

| Folge | Bedeutung |
|---|---|
| `"ABCD"` | hohes Wort zuerst, hohes Byte zuerst (die häufigste, Vorgabe) |
| `"CDAB"` | Wörter getauscht |
| `"BADC"` | Bytes je Wort getauscht |
| `"DCBA"` | beides getauscht |

| Funktion | Wirkung |
|---|---|
| `MODBUS_FLOAT(register, stelle [, folge$])` → FLOAT | zwei Register ab `stelle` als Kommazahl (float32) |
| `MODBUS_INT32(register, stelle [, folge$])` → INTEGER | zwei Register als Ganzzahl mit Vorzeichen |
| `MODBUS_UINT32(register, stelle [, folge$])` → INTEGER | zwei Register ohne Vorzeichen |
| `MODBUS_INT16(wert)` → INTEGER | ein Register mit Vorzeichen lesen: 65535 → −1 |
| `MODBUS_FLOAT_REGS(wert [, folge$])` → ARRAY OF INTEGER | eine Kommazahl als zwei Register, zum Schreiben |
| `MODBUS_INT32_REGS(wert [, folge$])` → ARRAY OF INTEGER | eine 32-Bit-Zahl als zwei Register |

```basic
IMPORT "modbus"
DIM wr AS MODBUS
wr = MODBUS_TCP("wechselrichter.local", 502, 1)
DIM r AS ARRAY OF INTEGER
r = MODBUS_READ_HOLDING(wr, 30775, 2)
PRINT "Einspeisung: "; MODBUS_INT32(r, 0); " W"
MODBUS_WRITE_REGISTERS(wr, 40000, MODBUS_FLOAT_REGS(21.5, "CDAB"))
```

## Was geprüft ist

Die Rahmen (Prüfsumme, Kopf von Modbus TCP, alle acht Funktionen, Ausnahmen,
Wortfolgen) prüfen Rust-Tests gegen die Beispiele der Spezifikation. Die
Prüfsammlung `tests/pruef/geraete/modbus.dhtest` lässt ein gespieltes Gerät in
Drachenhauch laufen, das seine Prüfsumme selbst nachrechnet und jede Anfrage
in Hex festhält — so ist die Leitung Byte für Byte geprüft, für TCP und für
RTU über das Netz, samt Ausnahmen, Zeitgrenze, falscher Prüfsumme,
abgebrochener Antwort und einer verspäteten Antwort mit fremder Kennung.

**Nicht geprüft:** RTU an einer echten seriellen Schnittstelle, und kein
echtes Gerät.

## Grenzen

- Nur die Funktionen 1..6, 15 und 16 — nicht „Lesen und Schreiben in einem“
  (23), Dateizugriff oder Gerätekennung (43).
- Kein Server: Drachenhauch fragt Geräte, es spielt keines.
- Modbus ASCII nicht.
