# ESP32 <-> GameBasic

Grundgerüst, um ein ESP32-Board (oder einen ESP8266, siehe unten) über MQTT mit
einem GameBasic-Programm reden zu lassen. Die Verkabelung ist fertig — WLAN,
Broker-Verbindung, Wiederverbinden, Nachrichtenempfang. Deine eigene Arbeit
kommt an vier markierte Stellen.

| Datei | Was |
|---|---|
| `drachenhauch_esp32/drachenhauch_esp32.ino` | der Sketch fürs Board |
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

1. *Datei → Grundeinstellungen → Zusätzliche Boardverwalter-URLs* — je nach
   Board eine davon (beide gleichzeitig gehen auch, mit Komma getrennt):
   - ESP32: `https://espressif.github.io/arduino-esp32/package_esp32_index.json`
   - ESP8266: `https://arduino.esp8266.com/stable/package_esp8266com_index.json`
2. *Werkzeuge → Board → Boardverwalter* → „esp32" bzw. „esp8266" installieren
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

**Nichts zu ändern** — derselbe Sketch läuft auf beiden. Die zwei Unterschiede
stehen als `#if defined(ARDUINO_ARCH_ESP8266)` ganz oben in der Datei
gebündelt: die WLAN-Kopfdatei heißt anders, und der ESP8266 hat genau einen
Analogeingang (`A0` statt frei wählbarer Pins). Weiter unten ist deshalb nur
eine Fassung zu lesen.

Zwei getrennte Sketche wären der naheliegende Weg gewesen — und binnen kurzem
auseinandergelaufen, sobald jemand nur einen von beiden pflegt.

## Geprüft

Übersetzt gegen `esp32:esp32@3.3.11`, `esp8266:esp8266@3.1.2` und
PubSubClient 2.8, jeweils mit `--warnings all` — **keine Warnung im eigenen
Code** auf keinem der vier Boards:

| Board | FQBN | Ergebnis |
|---|---|---|
| ESP32 | `esp32:esp32:esp32` | OK — 68 % Flash, 14 % RAM |
| ESP8266 NodeMCU 1.0 | `esp8266:esp8266:nodemcuv2` | OK — 23 % Flash, 35 % RAM |
| ESP32-C3 | `esp32:esp32:esp32c3` | OK |
| ESP32-S3 | `esp32:esp32:esp32s3` | OK |

```
arduino-cli compile --fqbn esp32:esp32:esp32 esp32/drachenhauch_esp32
```

(Die einzige Warnung im ESP8266-Bau steckt in PubSubClients eigenem Quelltext,
nicht in diesem Sketch.)

Die `arduino-cli` liegt übrigens in der Arduino-IDE mit drin:
`C:\Program Files\Arduino IDE\resources\app\lib\backend\resources\arduino-cli.exe`
— sie benutzt denselben Datenordner wie die IDE, installierte Kerne und
Bibliotheken stehen also in beiden zur Verfügung.

**GameBasic-Seite:** gegen echtes Mosquitto durchgetestet, mit `mosquitto_pub`
als Board-Attrappe.

Was ein Compiler NICHT beweisen kann: dass der Sketch auf echter Hardware das
Richtige tut. Er zeigt nur, dass nichts Unsinniges dasteht.
