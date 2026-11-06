# Module `websocket`

A standing connection over which both sides can send messages at any time —
what a web page uses to talk live to a program, what an ESP32 sends its
readings over (`ESPAsyncWebServer`, `arduinoWebSockets`), or how a game talks
to a server.

```basic
IMPORT "websocket"
```

Drachenhauch can do both sides: **client** (`ws://` and `wss://` with TLS)
and **server**. Reading never waits — a frame loop asks once per frame
whether messages have arrived. Ping is answered with pong by itself, a close
with a close, fragmented messages are put back together.

## Client

```basic
IMPORT "websocket"
DIM w AS WEBSOCKET
w = WS_CONNECT("ws://esp32.local/ws")
WS_SEND(w, !"{\"befehl\": \"messen\"}")
WHILE WS_CONNECTED(w)
    WHILE WS_NEXT(w)
        PRINT "kam: "; WS_MESSAGE$(w)
    WEND
    SLEEP(20)
WEND
PRINT "zu: "; WS_CLOSE_CODE(w); " "; WS_CLOSE_REASON$(w)
```

## Server

A server accepts connections without waiting; each one is its own
`WEBSOCKET`. A page in the browser connects with
`new WebSocket("ws://computer:9001/")`.

```basic
IMPORT "websocket"
DIM s AS WS_SERVER
s = WS_LISTEN(9001)
DIM leute AS ARRAY OF INTEGER
leute = []
WHILE TRUE
    DIM neu AS WEBSOCKET : neu = WS_ACCEPT(s)
    IF neu >= 0 THEN ARRAY_PUSH(leute, neu)
    DIM i AS INTEGER
    FOR i = 0 TO LEN(leute) - 1
        WHILE WS_NEXT(leute[i])
            ' pass it on to everyone
            DIM k AS INTEGER
            FOR k = 0 TO LEN(leute) - 1
                WS_SEND(leute[k], WS_MESSAGE$(leute[i]))
            NEXT
        WEND
    NEXT
    SLEEP(10)
WEND
```

## Commands

| Command | What it does |
|---|---|
| `WS_CONNECT(url$)` → WEBSOCKET | connect to a server (`ws://computer[:port]/path`, `wss://…` with TLS); waits at most 5 s for the handshake |
| `WS_LISTEN(port [, bind$])` → WS_SERVER | open a server; port 0 = a free one, `bind$` default `"0.0.0.0"` (all networks) |
| `WS_PORT(server)` → INTEGER | the server's port (after port 0 the chosen one) |
| `WS_ACCEPT(server)` → WEBSOCKET | accept a waiting connection, otherwise -1; never waits. A request that does not want a WebSocket gets 400 and does not count |
| `WS_STOP(server)` | close the server (accepted connections stay) |
| `WS_SEND(ws, daten)` → BOOLEAN | send a message: a text as a text message, a `BUFFER` as a binary one. FALSE if the connection is closed |
| `WS_NEXT(ws)` → BOOLEAN | fetch the next arrived message; FALSE if there is none |
| `WS_MESSAGE$(ws)` → STRING | the fetched message as text |
| `WS_MESSAGE_BYTES(ws)` → BUFFER | the fetched message as bytes |
| `WS_IS_BINARY(ws)` → BOOLEAN | was the fetched message binary? |
| `WS_PENDING(ws)` → INTEGER | how many messages are still waiting |
| `WS_CONNECTED(ws)` → BOOLEAN | is the connection open? |
| `WS_PING(ws)` → BOOLEAN | send a ping (the other side answers by itself) |
| `WS_PATH$(ws)` → STRING | the path a client connected with (`/room?x=1`) |
| `WS_PEER$(ws)` → STRING | address and port of the other side |
| `WS_CLOSE(ws [, code [, grund$]])` | close cleanly; code 1000 (default) normal, 1001 going away, 4000..4999 your own |
| `WS_CLOSE_CODE(ws)` → INTEGER | the code the connection closed with (1006 = broken off without goodbye), 0 while open |
| `WS_CLOSE_REASON$(ws)` → STRING | the reason that came with it |

A message can be at most 16 MB.

## Together with the web server

[`httpd`](module-httpd.md) serves a page for the browser, the live data come
through a WebSocket server on a second port:

```basic
IMPORT "httpd"
IMPORT "websocket"
DIM web AS HTTPD
web = HTTPD_START(8080)
DIM live AS WS_SERVER
live = WS_LISTEN(8081)
```

## Limits

- No extensions (`permessage-deflate` compression), no subprotocols.
- A server only speaks unencrypted (`ws://`); for `wss://` put a proxy in
  front (Caddy, nginx).
