# Modul `mdns`

**Multicast-DNS** — Geräte im eigenen Netz beim Namen finden, ohne ihre
IP-Adresse zu kennen: `esp32.local` statt `192.168.178.47`. Dazu die
**Dienstsuche** (DNS-SD): welche Webserver, Drucker oder eigenen Geräte gibt
es gerade im Netz? Und ein Drachenhauch-Programm kann sich selbst
**ankündigen**, damit ein ESP32, ein Telefon oder ein anderes Programm es
findet.

```basic
IMPORT "mdns"
```

Ein ESP32 meldet sich mit drei Zeilen (`MDNS.begin("esp32")`,
`MDNS.addService("http", "tcp", 80)`); ein Raspberry Pi mit Avahi, ein Mac
und Windows 10 und neuer antworten von selbst auf ihren Rechnernamen.
Drachenhauch braucht dafür weder Avahi noch Bonjour.

## Einen Namen auflösen

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

Der Name darf mit oder ohne `.local` stehen. [`WS_CONNECT`](module-websocket.md)
fragt selbst über mDNS nach, wenn das System einen `.local`-Namen nicht
kennt.

## Dienste finden

```basic
IMPORT "mdns"
DIM n AS INTEGER : n = MDNS_BROWSE("_http._tcp")
DIM i AS INTEGER
FOR i = 0 TO n - 1
    PRINT MDNS_SERVICE_NAME$(i); " -> "; MDNS_SERVICE_ADDRESS$(i); ":"; MDNS_SERVICE_PORT(i)
NEXT
```

Übliche Diensttypen: `_http._tcp` (Webserver), `_ws._tcp`, `_mqtt._tcp`,
`_osc._udp`, `_ipp._tcp` (Drucker), `_arduino._tcp` (Arduino-OTA),
`_esphomelib._tcp`.

## Sich ankündigen

```basic
IMPORT "mdns"
IMPORT "httpd"
DIM web AS HTTPD
web = HTTPD_START(8080)
PRINT MDNS_ANNOUNCE("Wetterstation", "_http._tcp", 8080, "pfad=/")
' -> wetterstation.local -- im Browser: http://wetterstation.local:8080/
```

## Befehle

| Befehl | Was es tut |
|---|---|
| `MDNS_RESOLVE$(name$ [, ms])` → STRING | die IPv4-Adresse zu `name.local`, `""` wenn bis `ms` (Vorgabe 1500) niemand antwortet |
| `MDNS_BROWSE(dienst$ [, ms])` → INTEGER | alle Dienste dieses Typs suchen, die bis `ms` (Vorgabe 2000) antworten; liefert ihre Zahl. Fehlende Angaben (Port, Adresse) werden nachgefragt |
| `MDNS_SERVICE_NAME$(index)` → STRING | Name eines gefundenen Dienstes (ab 0, nach Namen sortiert), etwa `"Kueche"` |
| `MDNS_SERVICE_HOST$(index)` → STRING | sein Rechnername, etwa `"kueche.local"` |
| `MDNS_SERVICE_ADDRESS$(index)` → STRING | seine IPv4-Adresse |
| `MDNS_SERVICE_PORT(index)` → INTEGER | sein Port |
| `MDNS_SERVICE_TXT$(index, schluessel$)` → STRING | ein Eintrag seiner TXT-Angaben (`pfad=/`), `""` wenn es ihn nicht gibt |
| `MDNS_ANNOUNCE(name$, dienst$, port [, txt$])` → STRING | einen eigenen Dienst ankündigen; liefert den Rechnernamen, unter dem er zu finden ist (`name.local`, klein, Leerzeichen als `-`). `txt$` wie `"pfad=/;version=2"`. Mehrfach aufgerufen kommen weitere Dienste dazu |
| `MDNS_STOP()` | alle Ankündigungen zurückziehen |

`ms` liegt zwischen 50 und 30000. Die Fragen gehen nach 0, 300 und 900 ms
noch einmal hinaus — im WLAN gehen Pakete verloren.

## Wie es fragt

Gefragt wird von einem gewöhnlichen UDP-Anschluss aus („legacy unicast“,
RFC 6762); die Antwortenden schicken direkt zurück. Den Anschluss 5353, den
das System schon benutzt, braucht nur, wer ankündigt — er wird geteilt.

**Prüfen ohne Netz:** `DHRT_MDNS_PORT=n` lässt Fragen und Ankündigen über
`127.0.0.1:n` laufen statt über die Gruppe `224.0.0.251` — zwei Programme auf
einem Rechner finden sich dann auch dort, wo kein Multicast geht.

## Grenzen

- Nur IPv4.
- Gefragt wird im Netz der Standard-Route; Rechner mit mehreren Netzen
  (WLAN und Kabel) fragen in einem davon.
- Ein Name, den schon jemand anderes ankündigt, wird nicht erkannt — wer
  zweimal `esp32.local` ankündigt, bekommt beide Antworten.
