# Bastelzentrale

Ein Leitstand für alles, was am Rechner hängt: Links die Bereiche, rechts
der Inhalt, unten der Zustand.

```
dhrt run main.dh
```

| Bereich | Was er kann |
|---|---|
| Übersicht | Jeder Bereich in einer Zeile, eine Kurve aus den wichtigsten Werten, Mikrofonpegel, die Analogwerte des Arduino und alle Ereignisse (auch angesteckte und abgezogene Geräte). [Alle Demos starten] verbindet jeden Bereich mit seinem gespielten Gerät. |
| Seriell | Monitor mit Kurve: eine Zeile aus lauter Zahlen oder `name:wert`-Paaren kommt in die Kurve (bis vier Reihen), wie beim Plotter der Arduino-IDE. Senden mit Zeilenende. |
| Arduino | Über Firmata (StandardFirmata seriell oder StandardFirmataWiFi als `tcp:host:3030`): sieben Ausgänge schalten, A0 bis A5 als Balken, ein Servo an Pin 9. |
| MQTT | An einem Broker mitlesen (Abo, Vorgabe `#`), letzter Stand je Topic als Tabelle, Verlauf, Senden. |
| Modbus | Modbus TCP: zehn Halteregister ab einer Adresse alle 500 ms, mit Vorzeichen und hex, eins schreiben, Register 1 und 2 als Kurve. |
| Klang | Mikrofon als Messgerät: Pegel, Grundton mit Notennamen und Cent, Spektrum. MIDI: ankommende Noten, das Tempo einer fremden Uhr, eine eigene Uhr für Drumcomputer (60 bis 180 BPM). |
| Kamera | Vorschau, Bewegungsmelder (mittlere Helligkeitsänderung auf einem 16 x 12-Raster, Schwelle einstellbar) und Schnappschuss als PNG in den Startordner. |
| Raspberry Pi | Acht Ausgänge und vier Eingänge (mit Pull-up, entprellt) an der Stiftleiste, nach BCM-Nummer. Unter Linux auf dem Pi die echten Pins, sonst eine Demo. |

## Ohne Geräte

Jeder Bereich hat eine Quelle **Demo** (bei der Kamera das **Testbild** der
Laufzeit): ein gespielter Sensor am seriellen Anschluss, ein Board, dessen
Analogwerte wandern (und das an A5 einen Taster hat, der „drückt“, solange
Pin 13 leuchtet), ein Haus voller MQTT-Sensoren, ein Modbus-Gerät, eine
Tonleiter statt des Mikrofons. So lässt sich alles ausprobieren, bevor eine
Platine da ist.

## Die Seite für das Handy

Im selben Netz im Browser des Handys:

```
http://bastelzentrale.local:8090/
```

(mDNS; wo der Name nicht aufgeht, die Adresse des Rechners statt
`bastelzentrale.local`). Die Seite zeigt die Werte live über einen WebSocket
(Port 8091), schaltet die Pins des Arduino und des Pi, stellt den Servo und
sendet MQTT. Unter `/zustand` steht derselbe Stand als JSON, unter
`/befehl?c=...` nimmt die Bastelzentrale einen Befehl an.

Dieselben Befehle gehen über **OSC** auf Port 8000, etwa aus TouchOSC:

| Befehl (Seite, `/befehl`) | OSC | Wirkung |
|---|---|---|
| `pin 13 1` | `/pin/13 1` | Arduino-Ausgang an (0 = aus) |
| `servo 120` | `/servo 0.66` (0..1) oder `/servo 120` | Servo stellen |
| `gpio 17 1` | `/gpio/17 1` | Ausgang am Raspberry Pi |
| `mqtt werkstatt/licht an` | `/mqtt/werkstatt/licht "an"` | MQTT senden |
| `modbus 3 42` | `/modbus/3 42` | Register schreiben |

Die Ports lassen sich über die Umgebung ändern: `DH_BZ_WEB`, `DH_BZ_WS`,
`DH_BZ_OSC` (0 = ein freier), `DH_BZ_MDNS=0` lässt die Ankündigung weg.
`DH_BZ_DEMO=1` startet alle Demos gleich, `DH_BZ_BEREICH=n` zeigt gleich den
Bereich n.

## Laufzeit

Seriell, MIDI und die Pins des Pi sind mit `IMPORT ... OPTIONAL` eingebunden:
in einer Laufzeit ohne sie (Bau ohne `--hardware`) sagt der Bereich das,
statt dass das Programm abbricht. Die Pins gibt es nur unter Linux.

## Aufbau

`main.dh` legt Fenster, Seitenleiste und Statusleiste an und dreht die
Bildschleife; jeder Bereich steht in `bereiche/` mit `xxAufbau`, `xxTakt`,
`xxZeichnen` und `xxZustand$`. `bereiche/netz.dh` ist die Handy-Seite samt
WebSocket, mDNS und OSC, die Seite selbst liegt in `assets/handy.html`.
