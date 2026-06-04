# Modul `net`

TCP- und UDP-Netzwerk via Python-stdlib-Sockets. Cross-Platform, **non-blocking** als Default — passt zu Game-Loops, in denen jeder Frame in <16 ms zurueckkehren muss. Encoding ist immer UTF-8.

```basic
IMPORT "net"
```

## Übersicht

### TCP

| Funktion | Rueckgabe | Wirkung |
|---|---|---|
| `NET_TCP_LISTEN(port)` | NET_LISTENER | Server-Socket auf Port (0 = OS waehlt freien) |
| `NET_LISTENER_PORT(lst)` | INTEGER | tatsaechlicher Port (nach LISTEN 0) |
| `NET_TCP_ACCEPT(lst)` | NET_SOCKET \| NIL | non-blocking: NIL wenn niemand verbindet |
| `NET_TCP_CONNECT(host, port)` | NET_SOCKET | Client-Verbindung (5s Timeout) |
| `NET_SEND(sock, text)` | INTEGER | gesendete Bytes |
| `NET_RECV(sock, max_bytes)` | STRING | leer wenn nichts da (non-blocking) |
| `NET_PEER_ADDR(sock)` | STRING | Remote-IP |
| `NET_PEER_PORT(sock)` | INTEGER | Remote-Port |
| `NET_SET_TIMEOUT(sock, ms)` | — | 0 = blocking, sonst Timeout |
| `NET_CLOSE(sock)` | — | Socket schliessen |
| `NET_CLOSE_LISTENER(lst)` | — | Listener schliessen |

### UDP

| Funktion | Rueckgabe | Wirkung |
|---|---|---|
| `NET_UDP_BIND(port)` | NET_UDP | UDP-Socket auf Port binden |
| `NET_UDP_OPEN()` | NET_UDP | UDP-Socket ohne Bind (nur Senden) |
| `NET_UDP_PORT(sock)` | INTEGER | gebundener Port (0 wenn OPEN) |
| `NET_UDP_SEND(sock, host, port, text)` | INTEGER | gesendete Bytes |
| `NET_UDP_RECV(sock, max_bytes)` | STRING | leer wenn nichts da |
| `NET_UDP_LAST_FROM(sock)` | TUPLE (host, port) | Absender des letzten RECV |
| `NET_UDP_SET_TIMEOUT(sock, ms)` | — | 0 = blocking |
| `NET_UDP_CLOSE(sock)` | — | Socket schliessen |

## Konzept

- **TCP** — verbindungsorientiert, garantierte Reihenfolge, keine verlorenen Pakete. Klassisch fuer Chat, Turn-Based, Lobby-Protokolle.
- **UDP** — verbindungslos, Pakete koennen verloren gehen oder verzaehlt ankommen. Klassisch fuer Real-Time (Position-Updates in Action-Spielen).

Alle Sockets sind **non-blocking by default** — RECV / ACCEPT liefern sofort leer/NIL zurueck wenn nichts da ist. So friert dein Game-Loop nicht ein.

Beim `NET_TCP_CONNECT` gibt es einen einmaligen 5-Sekunden-Connect-Timeout (blocking), damit "Server nicht erreichbar" innerhalb endlicher Zeit fehlschlaegt. Danach geht der Socket auf non-blocking.

## TCP-Echo-Server

```basic
IMPORT "net"

DIM lst AS NET_LISTENER
lst = NET_TCP_LISTEN(7000)
PRINT "Server auf Port 7000"

DIM clients AS ARRAY OF NET_SOCKET
DIM running AS BOOLEAN
running = TRUE
WHILE running
    ' Neue Verbindungen annehmen
    DIM new_client AS NET_SOCKET
    new_client = NET_TCP_ACCEPT(lst)
    IF new_client <> NIL THEN
        PRINT "Neuer Client von ", NET_PEER_ADDR(new_client)
        ' ... in client-Liste pushen
    END IF

    ' Alle Clients pollen
    ' ... fuer jeden Client:
    '   data = NET_RECV(client, 1024)
    '   IF LEN(data) > 0 THEN
    '       NET_SEND(client, "Echo: " + data)
    '   END IF

    SLEEP(16)
WEND
NET_CLOSE_LISTENER(lst)
```

## TCP-Client

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

## UDP-Game-Update (Position-Sync)

```basic
IMPORT "net"

' --- Server ---
DIM srv AS NET_UDP
srv = NET_UDP_BIND(6000)

WHILE running
    DIM msg AS STRING
    msg = NET_UDP_RECV(srv, 256)
    IF LEN(msg) > 0 THEN
        DIM peer AS TUPLE
        peer = NET_UDP_LAST_FROM(srv)
        PRINT "Update von ", peer[0]; ":"; peer[1]; ": "; msg
    END IF
    SLEEP(16)
WEND

' --- Client ---
DIM cli AS NET_UDP
cli = NET_UDP_OPEN()                ' nicht gebunden (nur Senden)

WHILE running
    NET_UDP_SEND(cli, "192.168.1.10", 6000,
                 "POS " + STR$(player_x) + " " + STR$(player_y))
    SLEEP(16)
WEND
```

## Non-blocking vs. Blocking

Default: alle Sockets sind non-blocking. RECV/ACCEPT liefern sofort leer/NIL wenn nichts da ist.

Wer **blockierend** lesen will (z.B. weil man weiss, dass jetzt Antwort kommen MUSS): `NET_SET_TIMEOUT(sock, ms)` mit `ms > 0` setzt einen Timeout. `ms = 0` setzt vollblockierendes Lesen — RECV wartet auf Daten. Vorsicht in Game-Loops: friert den Frame ein.

```basic
NET_SET_TIMEOUT(sock, 2000)         ' 2s Timeout
DIM answer AS STRING
answer = NET_RECV(sock, 1024)
' Wirft GBRuntimeError nach 2s wenn nichts kommt
```

## Externe Typen

| Typ | Wirkung |
|---|---|
| `NET_LISTENER` | TCP-Server-Socket (von `NET_TCP_LISTEN`) |
| `NET_SOCKET` | TCP-Verbindung (von `NET_TCP_ACCEPT` oder `NET_TCP_CONNECT`) |
| `NET_UDP` | UDP-Socket (gebunden oder offen) |

## Game-Patterns

**Lobby-Discovery via UDP-Broadcast** — siehe Beispiel.

**Chat-Protokoll mit Length-Prefix** (gegen Fragmentierung):

```basic
' Senden: 4-stellige Laenge + Text
DIM len_str AS STRING
len_str = STR$(LEN(msg))
WHILE LEN(len_str) < 4
    len_str = "0" + len_str
WEND
NET_SEND(sock, len_str + msg)
```

**Position-Sync (UDP), 30 Hz** — sende alle ~33ms eine Position, statt jeden Frame. Spart Bandbreite und Server-Load.

## Caveat: Lokales Testen

`127.0.0.1` (Localhost) funktioniert immer. Wenn du Server auf einem Rechner und Client auf einem anderen testen willst:
- Server muss auf einer Interface-IP binden (Default `""` = alle Interfaces, OK).
- Firewall muss den Port durchlassen.
- Client braucht die LAN-IP des Servers.

## Beispiel

[examples/72_net_chat.gb](../examples/72_net_chat.gb) zeigt einen kleinen Chat (TCP-Server + Client). UDP-Beispiele sind in den Tests (`tests/test_modules_net.py`) zu finden.

## In der nativen Runtime (gbrt)

`net` laeuft nativ mit dem Cargo-Feature `net` (reine `std::net`, keine zusaetzliche Crate). TCP-Listener/-Sockets + UDP, non-blocking per Default. Der Standard-Dev-Build (`python rust/build_runtime.py`) enthaelt `net` bereits.
