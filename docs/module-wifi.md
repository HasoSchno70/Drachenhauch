# Modul `wifi`

WLAN-Management: Netze scannen, aktuelle Verbindung abfragen, mit SSID + Passwort verbinden, gespeicherte Profile verwalten.

```basic
IMPORT "wifi"
```

## Plattform & Dependency

**Windows, Linux, macOS.** Keine externe Lib nötig — das Modul ruft jeweils die eingebaute Kommandozeilen-Verwaltung des Betriebssystems auf: Windows `netsh wlan`, Linux `nmcli` (NetworkManager), macOS `networksetup`/`airport`.

**Cross-Platform-Status:** Der Windows-Zweig ist gegen echte Hardware verifiziert. **Linux (`nmcli`) und macOS (`networksetup`/`airport`) sind neu und NICHT auf echter Hardware getestet** (Entwicklung läuft bisher nur unter Windows) — nur nach öffentlicher Doku (Linux) bzw. bestem Wissen (macOS) geschrieben. Der macOS-Zweig ist der unsicherste: `airport` (für `WIFI_SCAN`/`WIFI_SIGNAL`) liegt in einem privaten, undokumentierten Apple-Framework, dessen Verhalten sich zwischen macOS-Versionen schon mehrfach geändert hat und das neuere macOS-Versionen teils hinter einer Standortdienste-Berechtigung versteckt — schlägt es fehl, bekommst du eine klare Fehlermeldung statt eines stillen Fehlschlags. Rückmeldungen von echten Linux-/macOS-Nutzern sind ausdrücklich erwünscht (z.B. als GitHub Issue).

Auf sonstigen Plattformen (BSD, ...) wirft jeder Aufruf `DHRuntimeError` mit einem deutlichen Hinweis.

## Übersicht

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `WIFI_AVAILABLE()` | BOOLEAN (true wenn Windows + WLAN-Adapter) | laesst sich das Modul hier ueberhaupt nutzen? |
| `WIFI_CURRENT()` | STRING (aktuelle SSID, "" wenn nicht verbunden) | mit welchem Netz ist der Rechner verbunden? |
| `WIFI_SIGNAL()` | INTEGER (0..100, -1 wenn nicht verbunden) | Empfangsstaerke der aktuellen Verbindung |
| `WIFI_SCAN()` | STRING (multi-line `ssid\|signal_pct`, sortiert nach Signal) | erreichbare Netze suchen |
| `WIFI_CONNECT(ssid$, pass$)` | BOOLEAN (Profil anlegen + Connect-Befehl absetzen) | mit einem Netz verbinden -- legt dabei ein Windows-Profil an |
| `WIFI_DISCONNECT()` | BOOLEAN | Verbindung trennen |
| `WIFI_PROFILES()` | STRING (multi-line gespeicherter Profile) | welche Netze sind gespeichert? |
| `WIFI_DELETE_PROFILE(name$)` | BOOLEAN | gespeichertes Netz vergessen |

## Lokalisierung (Windows)

`netsh wlan` gibt seine Ausgabe in der System-Sprache aus (Deutsch / Englisch / Französisch …). Der Parser ist tolerant — verbundene/getrennte Zustände werden in DE/EN/FR/IT/ES erkannt, Profilnamen werden über das Stichwort „profil" / „Profile" gefunden. Sollte deine Sprache nicht erkannt werden: Issue / Patch willkommen.

`WIFI_AVAILABLE()` prüft NICHT (mehr) auf feste Wörter wie „WLAN"/„Wireless" — das war enger als die 5-Sprachen-Erkennung von `WIFI_CURRENT()` und lieferte auf jeder anderen Windows-Systemsprache fälschlich `FALSE`. Stattdessen zählt allein, ob `netsh wlan show interfaces` erfolgreich eine Schnittstelle auflisten konnte (sprachunabhängig).

Konsolen-Ausgabe von `netsh` ist auf Windows nicht verlässlich eine feste Kodierung (mal UTF-8, mal OEM-Codepage, je nach Windows-Version/Konfiguration) — das Modul versucht zuerst UTF-8, fällt bei ungültigen Bytes auf die OEM-Codepage zurück. Umlaute/Sonderzeichen in SSID-/Profilnamen (z.B. `WIFI_PROFILES()`) werden dadurch korrekt statt als Mojibake dargestellt.

`nmcli` (Linux) und `networksetup`/`airport` (macOS) liefern maschinenlesbaren bzw. sprachneutralen Output (`nmcli -t` = "terse", `networksetup`-Meldungen sind stabile Strings) — dort gibt es kein Lokalisierungsproblem.

## Plattform-Unterschiede

- **`WIFI_SIGNAL()`/`WIFI_SCAN()`-Werte**: Windows und Linux liefern echte Prozentwerte (0..100) direkt vom Treiber. macOS liefert nur RSSI in dBm (`airport`) — das Modul rechnet das mit einer groben, in der Netzwerkwelt gebräuchlichen Formel (`-50dBm≈100%`, `-100dBm≈0%`) auf 0..100 um; das ist eine Näherung, kein Treiber-Originalwert.
- **`WIFI_DISCONNECT()`**: Windows/Linux trennen die aktive Verbindung sauber. macOS' `networksetup` hat dafür keinen eigenen Befehl — der Workaround schaltet den WLAN-Funk kurz aus und wieder ein (deassoziiert von jedem Netz, WLAN bleibt danach an).
- **`WIFI_CONNECT`/Profile**: Auf allen drei Plattformen wird beim Verbinden ein Profil/eine Connection gespeichert (Windows-WLAN-Profil, NetworkManager-Connection, macOS-„Preferred Network"), das `WIFI_PROFILES()`/`WIFI_DELETE_PROFILE()` verwaltet.

## Beispiel — aktueller Status + Scan

```basic
IMPORT "wifi"

IF NOT WIFI_AVAILABLE() THEN
    PRINT "Kein WLAN-Adapter gefunden."
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

Siehe [examples/36_wifi.dh](../examples/36_wifi.dh).

## In der nativen Runtime (dhrt)

`wifi` laeuft nativ mit dem Cargo-Feature `wifi` (Windows via `netsh wlan`, Linux via `nmcli`, macOS via `networksetup`/`airport` -- alle drei per `std::process`, keine zusaetzliche Crate). Jeder Subprozess-Aufruf hat ein 10-Sekunden-Timeout -- ein haengender/ueberlasteter Netzwerk-Stack friert damit nicht mehr den Game-Loop ein. Bauen: `python rust/build_runtime.py --hardware`. Der `rust-check`-CI-Job (`.github/workflows/ci.yml`) kompiliert das `wifi`-Feature auf allen drei Plattformen (ubuntu/macos/windows-latest) -- das einzige automatische Cross-Platform-Signal bisher, da echte WLAN-Hardware in CI-Runnern fehlt.
