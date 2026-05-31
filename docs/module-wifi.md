# Modul `wifi`

WLAN-Management unter Windows: Netze scannen, aktuelle Verbindung abfragen, mit SSID + Passwort verbinden, gespeicherte Profile verwalten.

```basic
IMPORT "wifi"
```

## Plattform & Dependency

**Nur Windows.** Das Modul ruft die eingebaute `netsh wlan`-CLI auf — keine externe Python-Lib nötig.

Linux nutzt typischerweise `nmcli`, macOS `networksetup` — diese sind hier nicht implementiert. Auf Nicht-Windows wirft jeder Aufruf `GBRuntimeError` mit einem deutlichen Hinweis.

## Übersicht

| Funktion | Rückgabe |
|---|---|
| `WIFI_AVAILABLE()` | BOOLEAN (true wenn Windows + WLAN-Adapter) |
| `WIFI_CURRENT()` | STRING (aktuelle SSID, "" wenn nicht verbunden) |
| `WIFI_SIGNAL()` | INTEGER (0..100, -1 wenn nicht verbunden) |
| `WIFI_SCAN()` | STRING (multi-line `ssid\|signal_pct`, sortiert nach Signal) |
| `WIFI_CONNECT(ssid$, pass$)` | BOOLEAN (Profil anlegen + Connect-Befehl absetzen) |
| `WIFI_DISCONNECT()` | BOOLEAN |
| `WIFI_PROFILES()` | STRING (multi-line gespeicherter Profile) |
| `WIFI_DELETE_PROFILE(name$)` | BOOLEAN |

## Lokalisierung

`netsh wlan` gibt seine Ausgabe in der System-Sprache aus (Deutsch / Englisch / Französisch …). Der Parser ist tolerant — verbundene/getrennte Zustände werden in DE/EN/FR/IT/ES erkannt, Profilnamen werden über das Stichwort „profil" / „Profile" gefunden. Sollte deine Sprache nicht erkannt werden: Issue / Patch willkommen.

## Beispiel — aktueller Status + Scan

```basic
IMPORT "wifi"

IF NOT WIFI_AVAILABLE() THEN
    PRINT "Kein WLAN-Adapter (oder kein Windows)."
ELSE
    DIM ssid AS STRING
    ssid = WIFI_CURRENT()

    IF LEN(ssid) > 0 THEN
        PRINT "Verbunden: ", ssid, "  (", WIFI_SIGNAL(), "%)"
    ELSE
        PRINT "Aktuell nicht verbunden."
    END IF

    PRINT "Scan ..."
    DIM zeilen AS ARRAY OF STRING
    zeilen = SPLIT$(WIFI_SCAN(), CHR$(10))
    DIM i AS INTEGER
    FOR i = 0 TO LEN(zeilen) - 1
        PRINT "  ", zeilen[i]
    NEXT
END IF
```

## `WIFI_CONNECT` — was es tut

```basic
WIFI_CONNECT("MeinNetz", "geheim123")
```

Zwei Schritte:

1. Es generiert ein WLAN-Profil-XML (WPA2-PSK / AES für passwortgeschützt, offen für leeres Passwort) und legt es via `netsh wlan add profile` an. Bei einer SSID, die bereits ein Profil hat, ersetzt es das Profil.
2. Es ruft `netsh wlan connect name=<ssid>` auf.

`WIFI_CONNECT` gibt sofort zurück — **es wartet nicht auf das tatsächliche Verbinden**. Ob die Verbindung wirklich steht, prüfst du mit `WIFI_CURRENT()`:

```basic
IF WIFI_CONNECT("MeinNetz", "geheim123") THEN
    ' Auf Verbindung warten
    DIM versuch AS INTEGER
    FOR versuch = 1 TO 10
        SLEEP(1000)
        IF WIFI_CURRENT() = "MeinNetz" THEN
            PRINT "Verbunden!"
            BREAK
        END IF
    NEXT
END IF
```

### Was nicht unterstützt wird

- **WPA-Enterprise** (RADIUS/802.1X): braucht extra-Felder im Profil-XML.
- **WPA3-SAE**: das Profil-XML-Schema müsste anders aussehen.
- **Hidden Networks** (SSID-Broadcast aus): scan-Liste sieht sie nicht; verbinden via expliziter SSID kann manchmal trotzdem klappen.
- **Captive Portals** (Cafe-WLAN mit Login-Seite): das Modul verbindet nur Layer 2 — die Browser-Anmeldung musst du dem User überlassen.

### Sicherheitshinweis

Das von `WIFI_CONNECT` erzeugte Profil legt das Passwort als **plaintext** im Windows-Profil-Store ab (`<protected>false</protected>`). Das ist das Standardverhalten von `netsh wlan add profile`. Für Skript-Verwendung okay, für Production / Mehrbenutzer-Maschinen nicht geeignet.

## Profile verwalten

```basic
PRINT WIFI_PROFILES()                   ' Liste aller gespeicherten Profile
WIFI_DELETE_PROFILE("AlteSchule_WLAN")  ' Profil entfernen
```

## Berechtigungen

`netsh wlan` läuft normal ohne Admin-Rechte. Lediglich `WIFI_CONNECT` mit neuer SSID (= neues Profil anlegen) **kann** je nach Group-Policy Admin-Rechte verlangen — in Privat-Setups nie ein Thema.

## Komplettes Beispiel

Siehe [examples/36_wifi.gb](../examples/36_wifi.gb).
