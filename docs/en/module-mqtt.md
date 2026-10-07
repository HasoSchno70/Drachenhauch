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
packet ID tracking + a retry state machine), no `UNSUBSCRIBE`, no will
message, no TLS (`mqtts://`). For typical hobbyist use (publishing sensor
values, subscribing to control topics against a local broker such as
Mosquitto) QoS 0 is entirely sufficient — most ESP32 tutorials use QoS 0
anyway.

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `MQTT_CONNECT(host$, port, client_id$[, keepalive_s[, user$[, pass$]]])` | MQTT_HANDLE | establish the connection to the broker (`keepalive_s` default 60) |
| `MQTT_DISCONNECT(handle)` | — | close the connection |
| `MQTT_IS_CONNECTED(handle)` | BOOLEAN | is the connection still up? |
| `MQTT_PUBLISH(handle, topic$, payload$[, retain])` | — | send a message to a topic; with `retain` the broker keeps it for later subscribers |
| `MQTT_SUBSCRIBE(handle, topic$)` | — | subscribe to a topic -- incoming messages are then fetched by `MQTT_UPDATE` |
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
