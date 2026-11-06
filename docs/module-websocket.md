# Modul `websocket`

Eine stehende Verbindung, über die beide Seiten jederzeit Nachrichten
schicken — das, womit eine Webseite live mit einem Programm spricht, ein
ESP32 seine Messwerte schickt (`ESPAsyncWebServer`, `arduinoWebSockets`) oder
ein Spiel mit einem Server redet.

```basic
IMPORT "websocket"
```

Drachenhauch kann beide Seiten: **Client** (`ws://` und `wss://` mit TLS)
und **Server**. Gelesen wird ohne Warten — eine Bildschleife fragt je Bild,
ob Nachrichten da sind. Ping wird von selbst mit Pong beantwortet, ein
Schließen mit einem Schließen, zerteilte Nachrichten werden zusammengesetzt.

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

Ein Server nimmt Verbindungen ohne Warten an; jede ist ein eigenes
`WEBSOCKET`. Eine Seite aus dem Browser verbindet sich mit
`new WebSocket("ws://rechner:9001/")`.

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
            ' an alle weitergeben
            DIM k AS INTEGER
            FOR k = 0 TO LEN(leute) - 1
                WS_SEND(leute[k], WS_MESSAGE$(leute[i]))
            NEXT
        WEND
    NEXT
    SLEEP(10)
WEND
```

## Befehle

| Befehl | Was es tut |
|---|---|
| `WS_CONNECT(url$)` → WEBSOCKET | mit einem Server verbinden (`ws://rechner[:port]/pfad`, `wss://…` mit TLS); wartet höchstens 5 s auf den Handschlag |
| `WS_LISTEN(port [, bind$])` → WS_SERVER | einen Server öffnen; Port 0 = ein freier, `bind$` Vorgabe `"0.0.0.0"` (alle Netze) |
| `WS_PORT(server)` → INTEGER | der Port des Servers (nach Port 0 der gewählte) |
| `WS_ACCEPT(server)` → WEBSOCKET | eine wartende Verbindung annehmen, sonst -1; wartet nie. Eine Anfrage, die kein WebSocket will, bekommt 400 und zählt nicht |
| `WS_STOP(server)` | den Server schließen (angenommene Verbindungen bleiben) |
| `WS_SEND(ws, daten)` → BOOLEAN | eine Nachricht schicken: ein Text als Textnachricht, ein `BUFFER` als binäre. FALSE, wenn die Verbindung zu ist |
| `WS_NEXT(ws)` → BOOLEAN | die nächste angekommene Nachricht holen; FALSE, wenn keine da ist |
| `WS_MESSAGE$(ws)` → STRING | die geholte Nachricht als Text |
| `WS_MESSAGE_BYTES(ws)` → BUFFER | die geholte Nachricht als Bytes |
| `WS_IS_BINARY(ws)` → BOOLEAN | war die geholte Nachricht binär? |
| `WS_PENDING(ws)` → INTEGER | wie viele Nachrichten noch warten |
| `WS_CONNECTED(ws)` → BOOLEAN | ist die Verbindung offen? |
| `WS_PING(ws)` → BOOLEAN | ein Ping schicken (die Gegenseite antwortet von selbst) |
| `WS_PATH$(ws)` → STRING | der Pfad, mit dem sich ein Client verbunden hat (`/raum?x=1`) |
| `WS_PEER$(ws)` → STRING | Adresse und Port der Gegenseite |
| `WS_CLOSE(ws [, code [, grund$]])` | sauber schließen; Code 1000 (Vorgabe) normal, 1001 weg, 4000..4999 eigene |
| `WS_CLOSE_CODE(ws)` → INTEGER | mit welchem Code die Verbindung zuging (1006 = ohne Abschied abgebrochen), 0 solange offen |
| `WS_CLOSE_REASON$(ws)` → STRING | der Grund dazu |

Eine Nachricht darf höchstens 16 MB groß sein.

## Mit dem Webserver zusammen

Eine Seite für den Browser liefert [`httpd`](module-httpd.md) aus, die
Live-Daten kommen über einen WebSocket-Server auf einem zweiten Port:

```basic
IMPORT "httpd"
IMPORT "websocket"
DIM web AS HTTPD
web = HTTPD_START(8080)
DIM live AS WS_SERVER
live = WS_LISTEN(8081)
```

## Grenzen

- Keine Erweiterungen (Kompression `permessage-deflate`), keine
  Unterprotokolle.
- Ein Server spricht nur unverschlüsselt (`ws://`); für `wss://` gehört ein
  Proxy davor (Caddy, nginx).
