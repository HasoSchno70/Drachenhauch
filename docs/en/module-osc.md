# Module `osc`

**Open Sound Control** — short messages over UDP that music and media
programs exchange with each other: TouchOSC and Open Stage Control on the
phone, Pure Data, Max/MSP, SuperCollider, Sonic Pi, VCV Rack, Resolume,
QLab, Reaper, many lighting desks and maker projects with an ESP32
(`CNMAT/OSC`). With it you control a Drachenhauch program from your phone or
send notes to a synthesizer.

```basic
IMPORT "osc"
```

A message is an **address** (`/synth/1/freq`) and a list of typed
**values**. Drachenhauch converts the values itself: `INTEGER` → `i`
(outside 32 bits `h`), `FLOAT` → `f`, `STRING` → `s`, `BOOLEAN` → `T`/`F`,
`BUFFER` → `b` (blob), `NIL` → `N`. In addition `d` (double), `t` (time tag),
`c` (character), `r` (colour), `m` (MIDI) and `I` are read; bundles
(`#bundle`) arrive as their individual messages.

## Receiving

TouchOSC, for example, sends `/1/fader1` with a number 0..1:

```basic
IMPORT "osc"
DIM o AS OSC
o = OSC_OPEN(8000)
WHILE TRUE
    WHILE OSC_NEXT(o)
        IF OSC_MATCH("/1/fader*", OSC_ADDRESS$(o)) THEN
            PRINT OSC_ADDRESS$(o); " = "; OSC_ARG(o, 0)
        END IF
    WEND
    SLEEP(10)
WEND
```

## Sending

```basic
IMPORT "osc"
DIM o AS OSC
o = OSC_OPEN()
OSC_SEND(o, "127.0.0.1", 57120, "/note", 60, 0.8, "piano")
```

Sent to the broadcast address (`"255.255.255.255"` or the network's, such as
`"192.168.1.255"`) a message reaches everyone on the network.

## Commands

| Command | What it does |
|---|---|
| `OSC_OPEN([port [, bind$]])` → OSC | open a connection that listens on `port` (default 0 = a free one — enough for sending); `bind$` default `"0.0.0.0"` |
| `OSC_CLOSE(osc)` | close the connection |
| `OSC_PORT(osc)` → INTEGER | the port it listens on |
| `OSC_SEND(osc, host$, port, adresse$, ...)` | send a message with any number of values; the address starts with `/` |
| `OSC_NEXT(osc)` → BOOLEAN | fetch the next arrived message; FALSE if there is none. Never waits |
| `OSC_ADDRESS$(osc)` → STRING | address of the fetched message |
| `OSC_TYPES$(osc)` → STRING | its type tags, e.g. `"fis"` |
| `OSC_COUNT(osc)` → INTEGER | how many values it has |
| `OSC_ARG(osc, index)` | value number `index` (from 0) — as INTEGER, FLOAT, STRING, BOOLEAN, BUFFER or NIL |
| `OSC_SENDER$(osc)` → STRING | the address it came from |
| `OSC_SENDER_PORT(osc)` → INTEGER | the port it came from — a reply goes there |
| `OSC_MATCH(muster$, adresse$)` → BOOLEAN | does the address match an OSC pattern? Per part: `?` one character, `*` any number, `[1-4]`/`[!a-z]` one out of a set, `{red,green}` one of the alternatives |

A packet that is not OSC is skipped silently; at most 4096 waiting messages
are kept (the oldest are dropped).

## Limits

- UDP only (no OSC over TCP/SLIP).
- Single messages are sent, no bundles with a time tag.
