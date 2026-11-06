# Module `modbus`

**Modbus** is the protocol spoken by almost every device in building and
industrial technology: electricity meters, inverters, battery storage,
heating and ventilation controllers, frequency converters, PLCs. With this
module a Drachenhauch program reads and writes their values.

```basic
IMPORT "modbus"
```

A device has four areas, each counted from address 0:

| Area | Contents | read | write |
|---|---|---|---|
| Coils | single switches | `MODBUS_READ_COILS` | `MODBUS_WRITE_COIL`, `MODBUS_WRITE_COILS` |
| Discrete inputs | single signals | `MODBUS_READ_DISCRETE` | — |
| Holding registers | 16-bit values, also settings | `MODBUS_READ_HOLDING` | `MODBUS_WRITE_REGISTER`, `MODBUS_WRITE_REGISTERS` |
| Input registers | 16-bit measurements | `MODBUS_READ_INPUT` | — |

What each address means is in the device's data sheet. Some data sheets
count from 1 or put the area digit in front ("40001" = holding register 0) —
then subtract one or the prefix.

## Connecting

| Function | Effect |
|---|---|
| `MODBUS_TCP(host$ [, port [, einheit]])` → MODBUS | Modbus TCP over the network; port default 502, device address default 1 |
| `MODBUS_RTU(anschluss$ [, baud [, format$ [, einheit]]])` → MODBUS | Modbus RTU over RS-485: `"COM4"`/`"/dev/ttyUSB0"` (feature `serial`), baud default 9600, format default `"8N1"` (`"8E1"`, `"8O1"`, `"8N2"`, ...); or `"tcp:host:port"` for an RTU converter on the network |
| `MODBUS_UNIT(m, einheit)` | device address for the following requests (0..255; with RTU, 0 means "to all", and no answer comes) |
| `MODBUS_TIMEOUT(m, ms)` | how long to wait for an answer (10..60000, default 1000) |
| `MODBUS_CLOSE(m)` | close |

Several devices on one RS-485 bus share a connection — switch `MODBUS_UNIT`
before each request.

```basic
IMPORT "modbus"
DIM zaehler AS MODBUS
zaehler = MODBUS_TCP("192.168.1.60")
DIM r AS ARRAY OF INTEGER
r = MODBUS_READ_INPUT(zaehler, 0, 2)
PRINT "Leistung: "; MODBUS_FLOAT(r, 0); " W"
MODBUS_CLOSE(zaehler)
```

## Reading and writing

| Function | Effect |
|---|---|
| `MODBUS_READ_COILS(m, adresse, anzahl)` → ARRAY OF BOOLEAN | read coils (1..2000) |
| `MODBUS_READ_DISCRETE(m, adresse, anzahl)` → ARRAY OF BOOLEAN | read discrete inputs (1..2000) |
| `MODBUS_READ_HOLDING(m, adresse, anzahl)` → ARRAY OF INTEGER | read holding registers (1..125), values 0..65535 |
| `MODBUS_READ_INPUT(m, adresse, anzahl)` → ARRAY OF INTEGER | read input registers (1..125) |
| `MODBUS_WRITE_COIL(m, adresse, an)` | set one coil |
| `MODBUS_WRITE_REGISTER(m, adresse, wert)` | write one holding register (0..65535 or −32768..−1) |
| `MODBUS_WRITE_COILS(m, adresse, werte)` | several coils (array of BOOLEAN or 0/1, up to 1968) |
| `MODBUS_WRITE_REGISTERS(m, adresse, werte)` | several holding registers (array of numbers, up to 123) |

Every command **waits for the answer**; a device answers within
milliseconds. If the device reports an error (an "exception"), that is a run
time error with the number and its meaning, such as *Ausnahme 2 (unerlaubte
Datenadresse)* (illegal data address). With RTU the checksum of every answer
is recomputed; a wrong one is an error, not a wrong number.

## 32-bit values and signs

A register has 16 bits. Larger values and floating point numbers occupy
**two** registers — and the order is not fixed by the specification: every
manufacturer picks their own, and the data sheet says which. As usual, the
order is named after the four bytes A (highest) to D:

| Order | Meaning |
|---|---|
| `"ABCD"` | high word first, high byte first (the most common, default) |
| `"CDAB"` | words swapped |
| `"BADC"` | bytes swapped within each word |
| `"DCBA"` | both swapped |

| Function | Effect |
|---|---|
| `MODBUS_FLOAT(register, stelle [, folge$])` → FLOAT | two registers from `stelle` as a floating point number (float32) |
| `MODBUS_INT32(register, stelle [, folge$])` → INTEGER | two registers as a signed integer |
| `MODBUS_UINT32(register, stelle [, folge$])` → INTEGER | two registers, unsigned |
| `MODBUS_INT16(wert)` → INTEGER | read one register as signed: 65535 → −1 |
| `MODBUS_FLOAT_REGS(wert [, folge$])` → ARRAY OF INTEGER | a floating point number as two registers, for writing |
| `MODBUS_INT32_REGS(wert [, folge$])` → ARRAY OF INTEGER | a 32-bit number as two registers |

```basic
IMPORT "modbus"
DIM wr AS MODBUS
wr = MODBUS_TCP("wechselrichter.local", 502, 1)
DIM r AS ARRAY OF INTEGER
r = MODBUS_READ_HOLDING(wr, 30775, 2)
PRINT "Einspeisung: "; MODBUS_INT32(r, 0); " W"
MODBUS_WRITE_REGISTERS(wr, 40000, MODBUS_FLOAT_REGS(21.5, "CDAB"))
```

## What is tested

The frames (checksum, Modbus TCP header, all eight functions, exceptions,
word orders) are checked by Rust tests against the examples of the
specification. The test collection `tests/pruef/modbus.dhtest` runs a device
played in Drachenhauch that recomputes its checksum itself and records every
request in hex — so the line is checked byte by byte, for TCP and for RTU
over the network, including exceptions, timeout, wrong checksum, a cut-off
answer and a late answer with a foreign transaction id.

**Not tested:** RTU on a real serial port, and no real device.

## Limits

- Only functions 1..6, 15 and 16 — not "read and write in one" (23), file
  access or device identification (43).
- No server: Drachenhauch asks devices, it does not play one.
- No Modbus ASCII.
