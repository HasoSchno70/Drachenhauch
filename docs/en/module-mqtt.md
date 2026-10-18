# Module `mqtt`

MQTT client (version 3.1.1) — the dominant pub/sub protocol in the maker/IoT
world. Typical use case: an ESP32 publishes sensor values to a topic,
Drachenhauch subscribes to it; conversely, Drachenhauch publishes control
commands that the ESP32 has subscribed to.

```basic
IMPORT "mqtt"
```

Native in dhrt (feature `net` — already included in the standard build, no
`--hardware` flag needed, no new Cargo crate: plain `std::net`, implemented
directly against the OASIS MQTT 3.1.1 specification).

## Deliberately not covered

Only **QoS 0** (publish/subscribe without an ack handshake — QoS 1/2 would need
packet ID tracking + a retry state machine), no `UNSUBSCRIBE`, no TLS
(`mqtts://`). The last will (`MQTT_WILL`) is there since 2026-10-12. For typical hobbyist use (publishing sensor
values, subscribing to control topics against a local broker such as
Mosquitto) QoS 0 is entirely sufficient — most ESP32 tutorials use QoS 0
anyway.

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `MQTT_CONNECT(host$, port, client_id$[, keepalive_s[, user$[, pass$]]])` | MQTT_HANDLE | establish the connection to the broker (`keepalive_s` default 60) |
| `MQTT_DISCONNECT(handle)` | — | close the connection |
| `MQTT_IS_CONNECTED(handle)` | BOOLEAN | is the connection still up? |
| `MQTT_PUBLISH(handle, topic$, payload$[, retain])` | BOOLEAN | send a message to a topic; with `retain` the broker keeps it for later subscribers. FALSE if the connection is gone |
| `MQTT_SUBSCRIBE(handle, topic$)` | BOOLEAN | subscribe to a topic -- incoming messages are then fetched by `MQTT_UPDATE`. FALSE if the connection is gone |
| `MQTT_WILL(topic$, inhalt$[, retain])` | — | the **last will** for the next `MQTT_CONNECT`: if the connection breaks without a disconnect, the broker publishes this message |
| `MQTT_UPDATE(handle)` | — | **call once per frame**: reads in received messages and keeps the connection alive with PINGREQ |
| `MQTT_NEXT_MESSAGE(handle)` | BOOLEAN | advance by one message; FALSE when the queue is empty |
| `MQTT_MESSAGE_TOPIC(handle)` | STRING | topic of the current message |
| `MQTT_MESSAGE_PAYLOAD(handle)` | STRING | content of the current message |

`keepalive_s` (default 60) is the MQTT keepalive interval in seconds —
`MQTT_UPDATE` automatically sends a PINGREQ as soon as more than half of
this interval has passed since the last send, so that the broker does not
close the connection for inactivity.

## Incoming messages: cursor pattern like `db`

`MQTT_NEXT_MESSAGE`/`MQTT_MESSAGE_TOPIC`/`MQTT_MESSAGE_PAYLOAD` follow the
same cursor pattern as `DB_NEXT` + `DB_GET_*` in the `db` module:
`MQTT_NEXT_MESSAGE` advances the internal queue by one message and returns
`TRUE` as long as there was one; `MQTT_MESSAGE_TOPIC`/`_PAYLOAD` then read
fields of the **current** (most recently advanced) message.

```basic
MQTT_UPDATE(h)
WHILE MQTT_NEXT_MESSAGE(h)
    PRINT MQTT_MESSAGE_TOPIC(h), ": ", MQTT_MESSAGE_PAYLOAD(h)
WEND
```

**`MQTT_UPDATE` must be called once per frame** (like
`FIRMATA_UPDATE`/`INPUT_UPDATE`/`TIMER_UPDATE`) — it reads all currently
available bytes without blocking, updates the message queue and takes care
of the keepalive pings. Without `MQTT_UPDATE` no new messages arrive and the
connection is not kept alive.

## Example — subscribing to an ESP32 sensor + publishing a control command

```basic
IMPORT "mqtt"

DIM h AS MQTT_HANDLE
h = MQTT_CONNECT("192.168.1.50", 1883, "drachenhauch-client")

MQTT_SUBSCRIBE(h, "haus/wohnzimmer/temperatur")

WHILE NOT QUITREQUESTED()
    MQTT_UPDATE(h)

    WHILE MQTT_NEXT_MESSAGE(h)
        IF MQTT_MESSAGE_TOPIC(h) = "haus/wohnzimmer/temperatur" THEN
            PRINT "Temperatur: ", MQTT_MESSAGE_PAYLOAD(h)
        END IF
    WEND

    IF KEYPRESSED(KEY_SPACE) THEN
        MQTT_PUBLISH(h, "haus/wohnzimmer/lampe", "AN")
    END IF

    SLEEP(16)
WEND

MQTT_DISCONNECT(h)
```

## Error handling

`MQTT_CONNECT` throws if the broker cannot be reached, the connection is
refused, or the answer (CONNACK) does not arrive within 5 seconds — catch it
with `TRY/CATCH`:

```basic
TRY
    DIM h AS MQTT_HANDLE
    h = MQTT_CONNECT("broker-nicht-da.invalid", 1883, "client")
CATCH e
    PRINT "Konnte nicht verbinden: ", e
END TRY
```

`MQTT_DISCONNECT` is idempotent — disconnecting twice does not throw.

**If the connection breaks** (broker restarted, Wi-Fi gone), that is a
**state, not an error**: `MQTT_UPDATE` does not throw, `MQTT_IS_CONNECTED`
becomes FALSE, `MQTT_PUBLISH` and `MQTT_SUBSCRIBE` return FALSE. Up to
2026.30 `MQTT_UPDATE` threw (error 10054 under Windows), and a program
without `TRY` ended as soon as the broker restarted. If the broker vanishes
without any goodbye (power gone -- then no abort comes over the network
either), the client notices it by the missing answer to its PINGREQ: after
half the keepalive time (at least 2 s) the connection counts as lost.
Reconnecting is up to the program:

```basic
IMPORT "mqtt"
DIM h AS MQTT_HANDLE
DIM verbunden AS BOOLEAN
DIM naechsterVersuch AS INTEGER
SUB verbinden()
    TRY
        h = MQTT_CONNECT("127.0.0.1", 1883, "dashboard")
        MQTT_SUBSCRIBE(h, "werkstatt/#")
        verbunden = TRUE
    CATCH e
        naechsterVersuch = MILLIS() + 3000
    END TRY
END SUB
' je Bild:
IF verbunden THEN
    MQTT_UPDATE(h)
    IF NOT MQTT_IS_CONNECTED(h) THEN verbunden = FALSE : naechsterVersuch = MILLIS() + 3000
ELSEIF MILLIS() >= naechsterVersuch THEN
    verbinden()
END IF
```

## The last will

A board reports `online` with `retain` at startup -- and when its power runs
out, that `online` stays at the broker: every dashboard that starts later
takes the dead device for alive. The remedy is the **last will**: a message
the client deposits when connecting and that the BROKER publishes as soon as
the connection breaks without a disconnect (crash, power, network). A clean
`MQTT_DISCONNECT` does not trigger it.

```basic
IMPORT "mqtt"
MQTT_WILL("werkstatt/status", "offline", TRUE)   ' gilt fuer das naechste CONNECT
DIM h AS MQTT_HANDLE : h = MQTT_CONNECT("127.0.0.1", 1883, "werkstatt-sensor")
MQTT_PUBLISH(h, "werkstatt/status", "online", TRUE)
```

The will applies to exactly one `MQTT_CONNECT` -- whoever reconnects sets it
again (the ESP32 libraries do the same: `setWill` before `connect`).

## Complete example

See [examples/148_mqtt.dh](../../examples/148_mqtt.dh) (round trip against a
local broker — e.g. [Mosquitto](https://mosquitto.org/), default port
1883, no login needed for a local test broker).

## Connecting a real board

[esp32/](../../esp32/) contains a ready-made skeleton for ESP32/ESP8266
(Wi-Fi, broker connection, reconnecting, receiving) with four marked
places for your own code, plus the Drachenhauch counterpart
[examples/159_esp32_bruecke.dh](../../examples/159_esp32_bruecke.dh).

It also describes the pitfalls you would otherwise suffer through one by one:
a unique client ID, the 256-byte limit of PubSubClient, why `delay()` in
`loop()` makes the board die as far as the broker is concerned, and why you
get your own messages back when you subscribe to a wildcard.
