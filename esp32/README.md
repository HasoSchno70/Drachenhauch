# ESP32 <-> GameBasic

Grundgerüst, um ein ESP32-Board (oder einen ESP8266, siehe unten) über MQTT mit
einem GameBasic-Programm reden zu lassen. Die Verkabelung ist fertig — WLAN,
Broker-Verbindung, Wiederverbinden, Nachrichtenempfang. Deine eigene Arbeit
kommt an vier markierte Stellen.

| Datei | Was |
|---|---|
| `gamebasic_esp32/gamebasic_esp32.ino` | der Sketch fürs Board |
| `../examples/159_esp32_bruecke.gb` | das GameBasic-Gegenstück |

## Wie es zusammenhängt

Board und Programm reden **nie direkt** miteinander, sondern über einen
MQTT-Broker (Mosquitto). Beide kennen nur dessen Adresse:

```
ESP32  --esp32/wert-->    [ Broker ]  --> GameBasic
ESP32  <--esp32/befehl--  [ Broker ]  <-- GameBasic
```

Das klingt umständlich, spart aber die ganze Verbindungsverwaltung: Das Board
muss nicht wissen, ob dein Spiel gerade läuft, und mehrere Programme können
denselben Sensor mithören, ohne dass das Board etwas davon merkt.

## Vorbereitung

**Broker** — Mosquitto muss laufen und Verbindungen aus dem Netz annehmen.
Ab Version 2 hört er ohne Zutun **nur auf localhost**, ein Board aus dem WLAN
wird also abgewiesen. In der `mosquitto.conf`:

```
listener 1883 0.0.0.0
listener_allow_anonymous true
```

Dazu eine Firewall-Freigabe für eingehend TCP 1883 — sinnvollerweise begrenzt
aufs eigene Subnetz, nicht für alles.

**Arduino-IDE** (einmalig):

1. *Datei → Grundeinstellungen → Zusätzliche Boardverwalter-URLs*:
   `https://espressif.github.io/arduino-esp32/package_esp32_index.json`
2. *Werkzeuge → Board → Boardverwalter* → „esp32" installieren
3. *Werkzeuge → Bibliotheken verwalten* → **PubSubClient** (Nick O'Leary)

**Im Sketch anpassen:** `WLAN_NAME`, `WLAN_PASSWORT`, `BROKER`.

## Die vier Stellen

| | wo | wofür |
|---|---|---|
| **1** | `setup()` | Hardware einrichten (`pinMode`, Sensor starten) |
| **2** | `nachrichtEmpfangen()` | auf Befehle aus GameBasic reagieren |
| **3** | `brokerVerbinden()` | weitere Themen abonnieren |
| **4** | `loop()` | eigene Messwerte senden |

## Ohne Board ausprobieren

Mosquitto bringt zwei Werkzeuge mit, mit denen du das Board vortäuschst:

```
mosquitto_pub -h 127.0.0.1 -t esp32/wert   -m "2500"
mosquitto_sub -h 127.0.0.1 -t esp32/befehl -v
```

Starte `examples/159_esp32_bruecke.gb`, schick einen Wert — und sieh zu, wie der
Befehl zurückkommt. So entwickelst du die GameBasic-Seite fertig, lange bevor
die Hardware auf dem Tisch liegt.

## Fallstricke, die im Code stehen

- **Client-Kennung muss im Netz eindeutig sein.** Zwei Geräte mit derselben
  Kennung werfen sich abwechselnd raus — der Fehler sieht nach einem Wackler
  im WLAN aus. Der Sketch hängt deshalb die MAC-Adresse an.
- **`nutzlast` ist keine Zeichenkette.** Sie hat kein abschließendes Null-Byte;
  direkt ausgegeben hängt Datenmüll dran. Der Sketch kopiert sie sauber.
- **PubSubClient wirft Nachrichten über 256 Byte still weg** — kein Fehler, sie
  kommen einfach nie an. Der Sketch setzt den Puffer auf 512.
- **Kein `delay()` in `loop()`.** Das legt auch `mqtt.loop()` lahm, und der
  Broker hält das Board für tot. Der Sketch zählt stattdessen die Zeit mit.
- **Du hörst dich selbst.** Wer `esp32/#` abonniert, bekommt auch die eigenen
  Sendungen zurück — MQTT kennt keine Ausnahme für den Absender. Ohne
  Abfangen baut man sich leicht eine Endlosschleife.
- **Letzter Wille.** Der Sketch meldet dem Broker beim Verbinden, was er im
  Namen des Boards senden soll, wenn dieses unsauber verschwindet
  (`esp32/status` = `offline`). Ohne das sieht ein abgestürztes Board genauso
  aus wie eines, das gerade nichts zu melden hat.

## ESP8266 statt ESP32

Der Sketch läuft fast unverändert. Zu ändern sind nur:

```cpp
#include <ESP8266WiFi.h>      // statt WiFi.h
```

und `analogRead(34)` → `analogRead(A0)` (der ESP8266 hat nur einen
Analogeingang).

## Nicht geprüft

Der `.ino`-Sketch ist **nicht übersetzt worden** — auf diesem Rechner fehlt der
ESP32-Kern in der Arduino-Toolchain (nur `arduino:avr` ist installiert). Das
GameBasic-Gegenstück dagegen ist gegen echtes Mosquitto durchgetestet.
