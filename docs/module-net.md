# Modul `net`

TCP- und UDP-Netzwerk via Python-stdlib-Sockets. Cross-Platform, **non-blocking** als Default — passt zu Game-Loops, in denen jeder Frame in <16 ms zurueckkehren muss. Encoding ist immer UTF-8.

```basic
IMPORT "net"
```

## Übersicht

### TCP

| Funktion | Rueckgabe | Wirkung |
|---|---|---|
| `NET_TCP_LISTEN(port [, bind_addr$])` | NET_LISTENER | Server-Socket auf Port (0 = OS waehlt freien). `bind_addr$` optional, Default alle IPv4-Interfaces; `"::"` bindet stattdessen IPv6 |
| `NET_LISTENER_PORT(lst)` | INTEGER | tatsaechlicher Port (nach LISTEN 0) |
| `NET_TCP_ACCEPT(lst)` | NET_SOCKET \| NIL | non-blocking: NIL wenn niemand verbindet |
| `NET_TCP_CONNECT(host, port)` | NET_SOCKET | Client-Verbindung (5s DNS-Timeout + 5s Connect-Timeout) |
| `NET_SEND(sock, text)` | INTEGER | gesendete Bytes |
| `NET_RECV(sock, max_bytes)` | STRING | leer wenn nichts da (non-blocking) |
| `NET_PEER_ADDR(sock)` | STRING | Remote-IP |
| `NET_PEER_PORT(sock)` | INTEGER | Remote-Port |
| `NET_IS_CONNECTED(sock)` | BOOLEAN | FALSE sobald die Gegenseite geschlossen hat oder ein Recv/Send fehlgeschlagen ist |
| `NET_SET_TIMEOUT(sock, ms)` | — | 0 = blocking, sonst Timeout |
| `NET_CLOSE(sock)` | — | Socket schliessen |
| `NET_CLOSE_LISTENER(lst)` | — | Listener schliessen |

### UDP

| Funktion | Rueckgabe | Wirkung |
|---|---|---|
| `NET_UDP_BIND(port [, bind_addr$])` | NET_UDP | UDP-Socket auf Port binden. `bind_addr$` optional, Default alle IPv4-Interfaces; `"::"` bindet stattdessen IPv6 |
| `NET_UDP_OPEN()` | NET_UDP | UDP-Socket ohne Bind (nur Senden) |
| `NET_UDP_PORT(sock)` | INTEGER | gebundener Port (0 wenn OPEN) |
| `NET_UDP_SEND(sock, host, port, text)` | INTEGER | gesendete Bytes |
| `NET_UDP_RECV(sock, max_bytes)` | STRING | leer wenn nichts da |
| `NET_UDP_LAST_FROM(sock)` | STRING `"host:port"` | Absender des letzten RECV (leer vor dem ersten RECV) |
| `NET_UDP_SET_TIMEOUT(sock, ms)` | — | 0 = blocking |
| `NET_UDP_CLOSE(sock)` | — | Socket schliessen |

## Konzept

- **TCP** — verbindungsorientiert, garantierte Reihenfolge, keine verlorenen Pakete. Klassisch fuer Chat, Turn-Based, Lobby-Protokolle.
- **UDP** — verbindungslos, Pakete koennen verloren gehen oder verzaehlt ankommen. Klassisch fuer Real-Time (Position-Updates in Action-Spielen).

Alle Sockets sind **non-blocking by default** — RECV / ACCEPT liefern sofort leer/NIL zurueck wenn nichts da ist. So friert dein Game-Loop nicht ein.

Beim `NET_TCP_CONNECT` gibt es einen einmaligen 5-Sekunden-Connect-Timeout (blocking), damit "Server nicht erreichbar" innerhalb endlicher Zeit fehlschlaegt. Danach geht der Socket auf non-blocking. Die vorgelagerte DNS-Aufloesung von `host` hat ebenfalls ein eigenes 5-Sekunden-Timeout — ein haengender/langsamer DNS-Server kann den Game-Loop damit nicht unbegrenzt blockieren.

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
        DIM peer AS STRING
        peer = NET_UDP_LAST_FROM(srv)      ' z.B. "127.0.0.1:63332"
        PRINT "Update von "; peer; ": "; msg
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

## Verbindungsabbruch erkennen

Ein sauber geschlossener TCP-Socket liefert bei `NET_RECV` einfach weiterhin
`""` zurueck — genau wie "gerade nichts da" im non-blocking Betrieb. Ohne
weitere Pruefung merkt eine Warteschleife also nie, dass die Gegenseite weg
ist. `NET_IS_CONNECTED(sock)` unterscheidet die beiden Faelle:

```basic
WHILE NET_IS_CONNECTED(client)
    DIM data AS STRING
    data = NET_RECV(client, 1024)
    IF LEN(data) > 0 THEN ProcessMessage(data)
    SLEEP(16)
WEND
PRINT "Verbindung getrennt"
```

`NET_IS_CONNECTED` wird `FALSE`, sobald entweder die Gegenseite die
Verbindung sauber geschlossen hat (TCP-FIN) oder ein `NET_RECV`/`NET_SEND`
mit einem echten Fehler (nicht nur "gerade nichts da") fehlgeschlagen ist.

## UTF-8 ueber TCP

`NET_RECV` dekodiert den empfangenen Byte-Strom als UTF-8. Da TCP ein reiner
Byte-Strom ohne Nachrichtengrenzen ist, kann ein Mehrbyte-Zeichen (Umlaut,
Emoji) theoretisch genau zwischen zwei RECV-Aufrufen zerschnitten ankommen.
`NET_RECV` haelt so einen unvollstaendigen Rest intern zurueck und liefert
ihn erst mit den naechsten Bytes vervollstaendigt aus — ein zerschnittenes
Zeichen wird also NIE als kaputtes Zeichen (`�`) sichtbar, unabhaengig von
`max_bytes` oder Netzwerk-Timing. (Bei UDP stellt sich das Problem praktisch
nicht: ein Datagramm kommt immer komplett-oder-gar-nicht an; nur ein zu
klein gewaehltes `max_bytes` kann ein Datagramm abschneiden.)

## IPv6

`NET_TCP_LISTEN`/`NET_UDP_BIND` binden per Default auf allen IPv4-Interfaces
(wie bisher). Ein optionales zweites Argument `"::"` bindet stattdessen einen
reinen IPv6-Socket. Fuer Server, die sowohl IPv4- als auch IPv6-Clients
annehmen sollen, zwei Listener auf demselben Port oeffnen und beide pollen:

```basic
DIM lst4 AS NET_LISTENER
DIM lst6 AS NET_LISTENER
lst4 = NET_TCP_LISTEN(7000)
lst6 = NET_TCP_LISTEN(7000, "::")
```

Ausgehende Verbindungen (`NET_TCP_CONNECT`) sind bereits unabhaengig davon
IPv6-faehig — welche Adressfamilie verwendet wird, entscheidet die
DNS-Aufloesung von `host` automatisch.

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

**Server-Neustart auf demselben Port:** unter Windows (dem einzigen offiziell
unterstuetzten Zielsystem von gbrt) laesst sich ein Port direkt nach dem
Schliessen sofort wieder binden — anders als unter Linux/macOS gibt es hier
kein TIME_WAIT-bedingtes "Address already in use" ohne `SO_REUSEADDR`
(empirisch verifiziert). `NET_TCP_LISTEN` setzt daher bewusst kein
`SO_REUSEADDR`.

## Beispiel

[examples/72_net_chat.gb](../examples/72_net_chat.gb) zeigt einen kleinen Chat (TCP-Server + Client). UDP-Beispiele sind in den Tests (`tests/test_modules_net.py`) zu finden.

## In der nativen Runtime (gbrt)

`net` laeuft nativ mit dem Cargo-Feature `net` (reine `std::net`, keine zusaetzliche Crate). TCP-Listener/-Sockets + UDP, non-blocking per Default. Der Standard-Dev-Build (`python rust/build_runtime.py`) enthaelt `net` bereits.
