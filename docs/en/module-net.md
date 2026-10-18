# Module `net`

TCP and UDP networking, native in `dhrt` via Rust's `std::net` (no additional crate). Cross-platform, **non-blocking** by default — fits game loops in which every frame has to return in <16 ms. Text goes out as UTF-8; raw bytes go as a `BUFFER` (TCP and UDP).

```basic
IMPORT "net"
```

## Overview

### TCP

| Function | Returns | Effect |
|---|---|---|
| `NET_TCP_LISTEN(port [, bind_addr$])` | NET_LISTENER | server socket on a port (0 = the OS chooses a free one). `bind_addr$` optional, default all IPv4 interfaces; `"::"` binds IPv6 instead |
| `NET_LISTENER_PORT(lst)` | INTEGER | the actual port (after LISTEN 0) |
| `NET_TCP_ACCEPT(lst)` | NET_SOCKET \| NIL | non-blocking: NIL if nobody connects |
| `NET_TCP_CONNECT(host, port)` | NET_SOCKET | client connection (5 s DNS timeout + 5 s connect timeout) |
| `NET_SEND(sock, text)` | INTEGER | bytes sent; instead of text a `BUFFER` (raw bytes) works too |
| `NET_RECV(sock, max_bytes)` | STRING | empty if nothing is there (non-blocking) |
| `NET_RECV_BYTES(sock, max_bytes)` | BUFFER | raw bytes without UTF-8 decoding, empty if nothing is there -- for binary protocols |
| `NET_PEER_ADDR(sock)` | STRING | remote IP |
| `NET_PEER_PORT(sock)` | INTEGER | remote port |
| `NET_IS_CONNECTED(sock)` | BOOLEAN | FALSE as soon as the other side has closed, dropped hard or a recv/send has failed |
| `NET_SET_TIMEOUT(sock, ms)` | — | `0` = non-blocking (default), `> 0` = timeout in ms, `< 0` = fully blocking |
| `NET_CLOSE(sock)` | — | close the socket |
| `NET_CLOSE_LISTENER(lst)` | — | close the listener |

### UDP

| Function | Returns | Effect |
|---|---|---|
| `NET_UDP_BIND(port [, bind_addr$])` | NET_UDP | bind a UDP socket to a port. `bind_addr$` optional, default all IPv4 interfaces; `"::"` binds IPv6 instead |
| `NET_UDP_OPEN()` | NET_UDP | UDP socket without bind (sending only) |
| `NET_UDP_PORT(sock)` | INTEGER | bound port (0 if OPEN) |
| `NET_UDP_SEND(sock, host, port, text)` | INTEGER | bytes sent; a `BUFFER` (raw bytes) works instead of text |
| `NET_UDP_RECV(sock, max_bytes)` | STRING | empty if nothing is there |
| `NET_UDP_RECV_BYTES(sock, max_bytes)` | BUFFER | one datagram as raw bytes, empty if nothing is there -- for game states and binary protocols |
| `NET_UDP_LAST_FROM(sock)` | STRING `"host:port"` | sender of the last RECV (empty before the first RECV) |
| `NET_UDP_LAST_HOST$(sock)` | STRING | only the sender's address (`"::1"` instead of `"::1:5000"`) |
| `NET_UDP_LAST_PORT(sock)` | INTEGER | only its port |
| `NET_UDP_SET_TIMEOUT(sock, ms)` | — | as above: `0` = non-blocking, `> 0` = timeout, `< 0` = fully blocking |
| `NET_UDP_CLOSE(sock)` | — | close the socket |

## Concept

- **TCP** — connection-oriented, guaranteed order, no lost packets. The classic choice for chat, turn-based games, lobby protocols.
- **UDP** — connectionless, packets can get lost or arrive out of order. The classic choice for real time (position updates in action games).

All sockets are **non-blocking by default** — RECV / ACCEPT return empty/NIL immediately if nothing is there. That way your game loop does not freeze.

`NET_TCP_CONNECT` has a one-time 5-second connect timeout (blocking), so that "server not reachable" fails within a finite time. After that the socket goes non-blocking. The preceding DNS resolution of `host` also has its own 5-second timeout — a hanging/slow DNS server therefore cannot block the game loop indefinitely.

## TCP echo server

```basic
IMPORT "net"

DIM lst AS NET_LISTENER
lst = NET_TCP_LISTEN(7000)
PRINT "Server auf Port 7000"

DIM clients AS ARRAY OF NET_SOCKET
DIM running AS BOOLEAN
running = TRUE
WHILE running
    ' Accept new connections
    DIM new_client AS NET_SOCKET
    new_client = NET_TCP_ACCEPT(lst)
    IF new_client <> NIL THEN
        PRINT "Neuer Client von ", NET_PEER_ADDR(new_client)
        ' ... push into the client list
    END IF

    ' Poll all clients
    ' ... for each client:
    '   nachricht = NET_RECV(client, 1024)
    '   IF LEN(data) > 0 THEN
    '       NET_SEND(client, "Echo: " + data)
    '   END IF

    SLEEP(16)
WEND
NET_CLOSE_LISTENER(lst)
```

## TCP client

```basic
IMPORT "net"

DIM sock AS NET_SOCKET
sock = NET_TCP_CONNECT("127.0.0.1", 7000)

NET_SEND(sock, "Hallo Server")

DIM answer AS STRING
DIM waited AS INTEGER
waited = 0
WHILE answer = "" AND waited < 2000
    answer = NET_RECV(sock, 1024)
    SLEEP(50)
    waited = waited + 50
WEND
PRINT "Antwort:", answer

NET_CLOSE(sock)
```

## UDP game update (position sync)

```basic
IMPORT "net"

' --- Server ---
DIM srv AS NET_UDP
srv = NET_UDP_BIND(6000)

WHILE running
    DIM msg AS STRING
    msg = NET_UDP_RECV(srv, 256)
    IF LEN(msg) > 0 THEN
        DIM peer AS STRING
        peer = NET_UDP_LAST_FROM(srv)      ' e.g. "127.0.0.1:63332"
        PRINT "Update von "; peer; ": "; msg
    END IF
    SLEEP(16)
WEND

' --- Client ---
DIM cli AS NET_UDP
cli = NET_UDP_OPEN()                ' not bound (sending only)

WHILE running
    NET_UDP_SEND(cli, "192.168.1.10", 6000,
                 "POS " + STR$(player_x) + " " + STR$(player_y))
    SLEEP(16)
WEND
```

## Non-blocking vs. blocking

Default: all sockets are non-blocking. RECV/ACCEPT return empty/NIL immediately if nothing is there.

`NET_SET_TIMEOUT(sock, ms)` knows **three** cases — and `0` is not the one
you would expect:

| `ms` | Effect |
|---|---|
| `0` | **non-blocking** — that is the default; RECV returns immediately |
| `> 0` | timeout: RECV waits at most this long |
| `< 0` | fully blocking: RECV waits until data arrives. Freezes the frame |

```basic
NET_SET_TIMEOUT(sock, 2000)         ' wait at most 2 s
DIM answer AS STRING
answer = NET_RECV(sock, 1024)
IF answer = "" THEN
    PRINT "nichts gekommen"         ' a timeout is NOT an error
END IF
```

> **A timeout does not throw.** It returns an empty string — exactly like
> “nothing there right now" in non-blocking operation. If you need to tell the
> two cases apart, measure the time or ask `NET_IS_CONNECTED`.
> (Measured: `NET_SET_TIMEOUT(s, 400)` returns after 403 ms with `""`,
> without an error.)

## Detecting a dropped connection

A cleanly closed TCP socket simply keeps returning `""` from `NET_RECV` —
exactly like "nothing there right now" in non-blocking operation. Without a
further check a waiting loop therefore never notices that the other side is
gone. `NET_IS_CONNECTED(sock)` tells the two cases apart:

```basic
DIM nachricht AS STRING
WHILE NET_IS_CONNECTED(client)
    nachricht = NET_RECV(client, 1024)
    IF LEN(nachricht) > 0 THEN ProcessMessage(nachricht)
    SLEEP(16)
WEND
PRINT "Verbindung getrennt"
```

`NET_IS_CONNECTED` becomes `FALSE` as soon as either the other side has
closed the connection cleanly (TCP FIN) or a `NET_RECV`/`NET_SEND` has failed
with a real error (not just "nothing there right now").

**If the other side drops hard** (program ends without reading, crash, cable
pulled), that is not a runtime error: `NET_RECV` returns `""`,
`NET_RECV_BYTES` an empty buffer, `NET_SEND` 0, and `NET_IS_CONNECTED`
becomes `FALSE`. Until 2026-10-11 `NET_RECV` and `NET_SEND` threw "Eine
vorhandene Verbindung wurde vom Remotehost geschlossen" here -- a game host
died as soon as its guest closed the window.

With **UDP** there is no connection: a datagram to a port where nobody listens
any more simply disappears. Windows does report it on the next read;
`NET_UDP_RECV` then keeps reading instead of aborting. Whether the other side
is still there, with UDP, only shows in whether packets keep coming -- if you
need to know, count the time since the last packet.

**`NET_CLOSE` closes lingering:** it ends its own direction at once (the
other side then reads the end), but keeps reading in the background, for at
most two seconds, whatever the other side still sends, until it closes too.
If anything arrives at a socket that is fully closed, the operating system
drops the connection hard -- and the other side throws away what it was just
sent (measured: a "BESETZT" refusal before closing did not arrive in 5 of 5
attempts when the other side's "HALLO" came in only afterwards). `NET_CLOSE`
does not hold anything up meanwhile. The same holds at the end of the
program.

## UTF-8 over TCP

`NET_RECV` decodes the received byte stream as UTF-8. Since TCP is a pure
byte stream without message boundaries, a multi-byte character (umlaut,
emoji) can in theory arrive cut exactly between two RECV calls. `NET_RECV`
holds such an incomplete remainder back internally and only returns it once
completed by the next bytes — so a cut character NEVER shows up as a broken
character (`�`), regardless of `max_bytes` or network timing. (With UDP the
problem practically does not arise: a datagram always arrives completely or
not at all; only a `max_bytes` chosen too small can cut off a datagram.)

## Bytes instead of text

`NET_SEND` and `NET_RECV` speak UTF-8. For everything else -- an image, a
packed file, a device with its own format, an HTTP response with raw bytes --
`NET_SEND` takes a `BUFFER`, and `NET_RECV_BYTES` returns one. Mixing the two
loses nothing: what `NET_RECV` held back as a started multi-byte character,
`NET_RECV_BYTES` hands out first.

```basic
DIM b AS BUFFER : b = BUFFER_FROM_HEX("00fffe0a")
NET_SEND(sock, b)
DIM roh AS BUFFER : roh = NET_RECV_BYTES(sock, 4096)
PRINT BUFFER_TO_HEX$(roh)
```

The same with **UDP**: `NET_UDP_SEND` takes a `BUFFER`, and
`NET_UDP_RECV_BYTES` returns one datagram as a `BUFFER`. A game state with
floating-point numbers is not UTF-8 -- through `NET_UDP_RECV` bytes above 127
would arrive as replacement characters.

```basic
IMPORT "net"
DIM sock AS NET_UDP : sock = NET_UDP_BIND(0, "127.0.0.1")
DIM paket AS BUFFER : paket = BUFFER_NEW(9)
BUFFER_SET(paket, 0, 2)                  ' kind of packet
BUFFER_SET_F32(paket, 1, 412.5)          ' x
BUFFER_SET_F32(paket, 5, 288.0)          ' y
NET_UDP_SEND(sock, "127.0.0.1", NET_UDP_PORT(sock), paket)
SLEEP(20)
DIM rein AS BUFFER : rein = NET_UDP_RECV_BYTES(sock, 512)
IF BUFFER_LEN(rein) > 0 THEN
    PRINT BUFFER_GET_F32(rein, 1); " von "; NET_UDP_LAST_HOST$(sock); ":"; NET_UDP_LAST_PORT(sock)
END IF
```

An address like `127.0.0.1` or `::1` goes out without name resolution; only a
name (`"spielserver.local"`) is resolved for every packet (at most 2 s). If you
send to a name often, resolve it once -- for instance through the other
side's first packet and `NET_UDP_LAST_HOST$`.

## IPv6

`NET_TCP_LISTEN`/`NET_UDP_BIND` bind to all IPv4 interfaces by default (as
before). An optional second argument `"::"` binds a pure IPv6 socket instead.
For servers that should accept both IPv4 and IPv6 clients, open two listeners
on the same port and poll both:

```basic
DIM lst4 AS NET_LISTENER
DIM lst6 AS NET_LISTENER
lst4 = NET_TCP_LISTEN(7000)
lst6 = NET_TCP_LISTEN(7000, "::")
```

Outgoing connections (`NET_TCP_CONNECT`) are IPv6-capable regardless — which
address family is used is decided automatically by the DNS resolution of
`host`.

## External types

| Type | Effect |
|---|---|
| `NET_LISTENER` | TCP server socket (from `NET_TCP_LISTEN`) |
| `NET_SOCKET` | TCP connection (from `NET_TCP_ACCEPT` or `NET_TCP_CONNECT`) |
| `NET_UDP` | UDP socket (bound or open) |

## Game patterns

**Lobby discovery via UDP broadcast** — see the example.

**Chat protocol with a length prefix** (against fragmentation):

```basic
' Send: 4-digit length + text
DIM len_str AS STRING
len_str = STR$(LEN(msg))
WHILE LEN(len_str) < 4
    len_str = "0" + len_str
WEND
NET_SEND(sock, len_str + msg)
```

**Position sync (UDP), 30 Hz** — send a position every ~33 ms instead of every frame. Saves bandwidth and server load.

## Caveat: local testing

`127.0.0.1` (localhost) always works. If you want to test the server on one machine and the client on another:
- The server must bind to an interface IP (default `""` = all interfaces, OK).
- The firewall must let the port through.
- The client needs the server's LAN IP.

**Restarting a server on the same port:** under Windows (the only officially
supported target system of dhrt) a port can be bound again immediately after
closing — unlike under Linux/macOS there is no TIME_WAIT-induced "Address
already in use" without `SO_REUSEADDR` (verified empirically).
`NET_TCP_LISTEN` therefore deliberately does not set `SO_REUSEADDR`.

## Example

[examples/72_net_chat.dh](../../examples/72_net_chat.dh) shows a small chat (TCP server + client).

[examples/duell/main.dh](../../examples/duell/main.dh) is a real-time game for two: connecting and the end of a round over TCP, the game state 30 times a second as a binary packet over UDP, and the guest predicts its own ship and reconciles it with every state from the host. [examples/duell/relay.dh](../../examples/duell/relay.dh) imitates a bad network (delay, jitter, loss) -- so all of this can be tried on ONE computer: two windows, host and guest over `127.0.0.1`.

## In the native runtime (dhrt)

`net` runs natively with the Cargo feature `net` (pure `std::net`, no additional crate). TCP listeners/sockets + UDP, non-blocking by default. The standard dev build (`python rust/build_runtime.py`) already includes `net`.
