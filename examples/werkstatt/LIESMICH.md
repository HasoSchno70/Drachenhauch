# Werkstatt

Ein Dashboard für die Sensoren in der Werkstatt, über MQTT: Temperatur und
Luftfeuchte als Tacho, Strom und Licht als Leisten, der Verlauf der letzten
zwei Minuten, die Tür, ein Schalter für den Lüfter, ein Hitzealarm (Meldung
des Systems, auf Wunsch schaltet das Dashboard den Lüfter selbst) und eine
Aufzeichnung als CSV.

```
dhrt run sensor.dh                (spielt das Board -- ohne Platine ausprobieren)
dhrt run main.dh                  (Broker 127.0.0.1:1883)
dhrt run main.dh -- 192.168.1.5   (Broker auf dem Raspberry Pi, auch host:port)
```

Gebraucht wird ein MQTT-Broker, etwa [Mosquitto](https://mosquitto.org/) auf
diesem Rechner oder einem Raspberry Pi.

## Die Topics

| Topic | Richtung | Inhalt |
|---|---|---|
| `werkstatt/temperatur` | Board → | °C mit Punkt (`23.5`) |
| `werkstatt/feuchte` | Board → | % |
| `werkstatt/licht` | Board → | Lux |
| `werkstatt/strom` | Board → | Ampere |
| `werkstatt/tuer` | Board → | `offen` / `zu` (retain) |
| `werkstatt/luefter` | Board → | `an` / `aus` -- der tatsächliche Zustand (retain) |
| `werkstatt/luefter/set` | → Board | `an` / `aus` -- der Wunsch |
| `werkstatt/status` | Board → | `online`, und `offline` vom Broker (letzter Wille) |

Ein echtes Board schickt dieselben Topics; das Grundgerüst dafür liegt in
[esp32/](../../esp32/).

## Was man daran sieht

- **Der letzte Wille.** Das Board meldet `online` mit `retain`. Geht ihm der
  Strom aus, bliebe das für immer beim Broker liegen -- jedes Dashboard, das
  später startet, hielte das tote Board für lebendig. Darum hinterlegt
  `sensor.dh` mit `MQTT_WILL` ein `offline`, das der Broker selbst
  veröffentlicht, sobald die Verbindung ohne Abmeldung abreißt.
- **Den echten Zustand melden.** Auch ein `an` für den Lüfter bleibt beim
  Broker liegen. Startet das Board neu (Lüfter aus), muss es das sofort
  sagen, sonst glaubt das Dashboard die alte Nachricht und schaltet bei
  Hitze nicht ein.
- **Wunsch und Bestätigung getrennt.** Das Dashboard schickt
  `werkstatt/luefter/set`; der Schalter folgt erst dem, was das Board auf
  `werkstatt/luefter` bestätigt.
- **Ein Broker, der weg ist, ist ein Zustand.** Startet der Raspberry Pi neu,
  wird `MQTT_IS_CONNECTED` FALSE, und das Dashboard versucht es alle drei
  Sekunden wieder -- es endet nicht.
- **Alarm mit Abstand.** Der Alarm kommt beim Überschreiten, nicht in jedem
  Bild; die Automatik schaltet den Lüfter zwei Grad unter der Grenze wieder
  ab, sonst ginge er im Takt der Messungen an und aus.

Geprüft wird es in `tests/pruef/beispiel_werkstatt.dhtest`: ohne Broker,
Alarm und Aufzeichnung, die Lüfter-Automatik gegen einen Test-Broker, der
dann wegbricht, und der letzte Wille des Sensors am Draht.
