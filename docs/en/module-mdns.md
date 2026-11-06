# Module `mdns`

**Multicast DNS** — find devices on your own network by name without knowing
their IP address: `esp32.local` instead of `192.168.178.47`. Plus **service
discovery** (DNS-SD): which web servers, printers or devices of your own are
on the network right now? And a Drachenhauch program can **announce** itself
so an ESP32, a phone or another program finds it.

```basic
IMPORT "mdns"
```

An ESP32 announces itself with three lines (`MDNS.begin("esp32")`,
`MDNS.addService("http", "tcp", 80)`); a Raspberry Pi with Avahi, a Mac and
Windows 10 and newer answer for their computer name by themselves.
Drachenhauch needs neither Avahi nor Bonjour for it.

## Resolving a name

```basic
IMPORT "mdns"
DIM ip AS STRING
ip = MDNS_RESOLVE$("esp32")
IF ip = "" THEN
    PRINT "nicht gefunden"
ELSE
    PRINT "esp32 ist "; ip
END IF
```

The name may be written with or without `.local`. [`WS_CONNECT`](module-websocket.md)
asks over mDNS by itself when the system does not know a `.local` name.

## Finding services

```basic
IMPORT "mdns"
DIM n AS INTEGER : n = MDNS_BROWSE("_http._tcp")
DIM i AS INTEGER
FOR i = 0 TO n - 1
    PRINT MDNS_SERVICE_NAME$(i); " -> "; MDNS_SERVICE_ADDRESS$(i); ":"; MDNS_SERVICE_PORT(i)
NEXT
```

Common service types: `_http._tcp` (web server), `_ws._tcp`, `_mqtt._tcp`,
`_osc._udp`, `_ipp._tcp` (printer), `_arduino._tcp` (Arduino OTA),
`_esphomelib._tcp`.

## Announcing yourself

```basic
IMPORT "mdns"
IMPORT "httpd"
DIM web AS HTTPD
web = HTTPD_START(8080)
PRINT MDNS_ANNOUNCE("Wetterstation", "_http._tcp", 8080, "pfad=/")
' -> wetterstation.local -- in the browser: http://wetterstation.local:8080/
```

## Commands

| Command | What it does |
|---|---|
| `MDNS_RESOLVE$(name$ [, ms])` → STRING | the IPv4 address of `name.local`, `""` if nobody answers within `ms` (default 1500) |
| `MDNS_BROWSE(dienst$ [, ms])` → INTEGER | look for all services of this type that answer within `ms` (default 2000); returns how many. Missing details (port, address) are asked for |
| `MDNS_SERVICE_NAME$(index)` → STRING | name of a service found (from 0, sorted by name), e.g. `"Kueche"` |
| `MDNS_SERVICE_HOST$(index)` → STRING | its computer name, e.g. `"kueche.local"` |
| `MDNS_SERVICE_ADDRESS$(index)` → STRING | its IPv4 address |
| `MDNS_SERVICE_PORT(index)` → INTEGER | its port |
| `MDNS_SERVICE_TXT$(index, schluessel$)` → STRING | an entry of its TXT data (`pfad=/`), `""` if it has none |
| `MDNS_ANNOUNCE(name$, dienst$, port [, txt$])` → STRING | announce a service of your own; returns the computer name it can be found under (`name.local`, lower case, spaces as `-`). `txt$` like `"pfad=/;version=2"`. Called again, further services are added |
| `MDNS_STOP()` | withdraw all announcements |

`ms` lies between 50 and 30000. The questions go out again after 300 and
900 ms — packets get lost on Wi-Fi.

## How it asks

Questions are sent from an ordinary UDP connection ("legacy unicast",
RFC 6762); those who answer reply directly. Only whoever announces needs
port 5353, which the system already uses — it is shared.

**Testing without a network:** `DHRT_MDNS_PORT=n` sends questions and
announcements through `127.0.0.1:n` instead of the group `224.0.0.251` — two
programs on one computer then find each other even where multicast does not
work.

## Limits

- IPv4 only.
- Questions go out on the network of the default route; computers with
  several networks (Wi-Fi and cable) ask on one of them.
- A name someone else already announces is not detected — whoever announces
  `esp32.local` twice gets both answers.
