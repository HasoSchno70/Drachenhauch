# Modul `mqtt`

MQTT-Client (Version 3.1.1) — das im Maker-/IoT-Bereich dominante Pub/Sub-
Protokoll. Typischer Anwendungsfall: ein ESP32 publiziert Sensor-Werte auf
ein Topic, Drachenhauch subscribt darauf; umgekehrt publiziert Drachenhauch
Steuerbefehle, auf die der ESP32 subscribt hat.

```basic
IMPORT "mqtt"
```

Nativ in dhrt (Feature `net` — bereits im Standard-Build enthalten, keine
`--hardware`-Flag noetig), direkt gegen die OASIS-MQTT-3.1.1-Spezifikation
implementiert; TLS über dasselbe rustls wie das Mail-Modul.

## Umfang

**QoS 0 und 1** beim Senden und Empfangen, Abonnieren und Abmelden, der
letzte Wille und **TLS** (`mqtts://`, auch mit eigener CA). Nicht dabei:
**QoS 2** (vier Pakete je Nachricht, im Bastlerbereich kaum benutzt) und
Sitzungen über eine Verbindung hinaus — „Clean Session" ist immer an, eine
unbestätigte Nachricht wird also nicht von selbst neu geschickt. Für Sensor-
Werte und Steuerbefehle gegen einen Broker wie Mosquitto reicht QoS 0; QoS 1
nimmt man, wenn eine Nachricht sicher ankommen muss (ein Befehl, der einen
Motor stoppt).

## Übersicht

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `MQTT_CONNECT(host$, port, client_id$[, keepalive_s[, user$[, pass$]]])` | MQTT_HANDLE | Verbindung zum Broker aufbauen (`keepalive_s` Vorgabe 60); `mqtts://host` verbindet verschluesselt |
| `MQTT_TLS_CA(pfad$)` | — | eigene CA (PEM-Datei) fuer jedes folgende `mqtts://`-Verbinden; `""` = wieder die oeffentlichen Wurzeln |
| `MQTT_DISCONNECT(handle)` | — | Verbindung schliessen |
| `MQTT_IS_CONNECTED(handle)` | BOOLEAN | steht die Verbindung noch? |
| `MQTT_PUBLISH(handle, topic$, payload$[, retain[, qos]])` | BOOLEAN | Nachricht auf ein Topic senden; `retain` laesst der Broker sie fuer spaetere Abonnenten aufheben, `qos` 0 oder 1. FALSE, wenn die Verbindung weg ist |
| `MQTT_SUBSCRIBE(handle, topic$[, qos])` | BOOLEAN | Topic abonnieren -- eingehende Nachrichten holt danach `MQTT_UPDATE` ab; `qos` 0 oder 1. FALSE, wenn die Verbindung weg ist |
| `MQTT_UNSUBSCRIBE(handle, topic$)` | BOOLEAN | ein Abonnement beenden |
| `MQTT_PENDING(handle)` | INTEGER | wie viele QoS-1-Nachrichten der Broker noch nicht bestaetigt hat |
| `MQTT_FLUSH(handle[, ms])` | BOOLEAN | warten, bis alle QoS-1-Nachrichten bestaetigt sind (Vorgabe 2000 ms); FALSE, wenn die Zeit um ist |
| `MQTT_WILL(topic$, inhalt$[, retain])` | — | der **letzte Wille** fuer das naechste `MQTT_CONNECT`: reisst die Verbindung ohne Abmeldung ab, veroeffentlicht der Broker diese Nachricht |
| `MQTT_UPDATE(handle)` | — | **pro Frame aufrufen**: liest eingegangene Nachrichten ein und haelt die Verbindung mit PINGREQ am Leben |
| `MQTT_NEXT_MESSAGE(handle)` | BOOLEAN | eine Nachricht weiterruecken; FALSE, wenn die Warteschlange leer ist |
| `MQTT_MESSAGE_TOPIC(handle)` | STRING | Topic der aktuellen Nachricht |
| `MQTT_MESSAGE_PAYLOAD(handle)` | STRING | Inhalt der aktuellen Nachricht |

`keepalive_s` (Default 60) ist das MQTT-Keepalive-Intervall in Sekunden —
`MQTT_UPDATE` sendet automatisch ein PINGREQ, sobald mehr als die Hälfte
dieses Intervalls seit dem letzten Senden vergangen ist, damit der Broker
die Verbindung nicht wegen Inaktivität schließt.

## Eingehende Nachrichten: Cursor-Muster wie `db`

`MQTT_NEXT_MESSAGE`/`MQTT_MESSAGE_TOPIC`/`MQTT_MESSAGE_PAYLOAD` folgen
demselben Cursor-Muster wie `DB_NEXT` + `DB_GET_*` im `db`-Modul:
`MQTT_NEXT_MESSAGE` rückt die interne Warteschlange eine Nachricht weiter
und liefert `TRUE`, solange eine da war; `MQTT_MESSAGE_TOPIC`/`_PAYLOAD`
lesen dann Felder der **aktuellen** (zuletzt vorgerückten) Nachricht.

```basic
MQTT_UPDATE(h)
WHILE MQTT_NEXT_MESSAGE(h)
    PRINT MQTT_MESSAGE_TOPIC(h), ": ", MQTT_MESSAGE_PAYLOAD(h)
WEND
```

**`MQTT_UPDATE` muss pro Frame aufgerufen werden** (wie
`FIRMATA_UPDATE`/`INPUT_UPDATE`/`TIMER_UPDATE`) — es liest nicht-blockierend
alle aktuell verfügbaren Bytes, aktualisiert die Nachrichten-Warteschlange
und kümmert sich um die Keepalive-Pings. Ohne `MQTT_UPDATE` kommen weder
neue Nachrichten an noch wird die Verbindung am Leben gehalten.

## Beispiel — ESP32-Sensor abonnieren + Steuerbefehl publizieren

```basic
IMPORT "mqtt"

DIM h AS MQTT_HANDLE
h = MQTT_CONNECT("192.168.1.50", 1883, "drachenhauch-client")

MQTT_SUBSCRIBE(h, "haus/wohnzimmer/temperatur")

WHILE NOT QUITREQUESTED()
    MQTT_UPDATE(h)

    WHILE MQTT_NEXT_MESSAGE(h)
        IF MQTT_MESSAGE_TOPIC(h) = "haus/wohnzimmer/temperatur" THEN
            PRINT "Temperatur: ", MQTT_MESSAGE_PAYLOAD(h)
        END IF
    WEND

    IF KEYPRESSED(KEY_SPACE) THEN
        MQTT_PUBLISH(h, "haus/wohnzimmer/lampe", "AN")
    END IF

    SLEEP(16)
WEND

MQTT_DISCONNECT(h)
```

## Fehlerbehandlung

`MQTT_CONNECT` wirft, wenn der Broker nicht erreichbar ist, die Verbindung
abgelehnt wird, oder die Antwort (CONNACK) nicht innerhalb von 5 Sekunden
ankommt — mit `TRY/CATCH` abfangen:

```basic
TRY
    DIM h AS MQTT_HANDLE
    h = MQTT_CONNECT("broker-nicht-da.invalid", 1883, "client")
CATCH e
    PRINT "Konnte nicht verbinden: ", e
END TRY
```

`MQTT_DISCONNECT` ist idempotent — doppeltes Trennen wirft nicht.

**Bricht die Verbindung weg** (Broker neu gestartet, WLAN weg), ist das ein
**Zustand, kein Fehler**: `MQTT_UPDATE` wirft nicht, `MQTT_IS_CONNECTED`
wird FALSE, `MQTT_PUBLISH` und `MQTT_SUBSCRIBE` liefern FALSE. Bis 2026.30
warf `MQTT_UPDATE` (unter Windows Fehler 10054), und ein Programm ohne
`TRY` endete, sobald der Broker neu startete. Verschwindet der Broker ganz
ohne Abmeldung (Strom weg -- dann kommt auch kein Abbruch ueber das Netz),
merkt es der Client an der fehlenden Antwort auf sein PINGREQ: nach der
halben Keepalive-Zeit (mindestens 2 s) gilt die Verbindung als verloren.
Wiederverbinden ist Sache des Programms:

```basic
IMPORT "mqtt"
DIM h AS MQTT_HANDLE
DIM verbunden AS BOOLEAN
DIM naechsterVersuch AS INTEGER
SUB verbinden()
    TRY
        h = MQTT_CONNECT("127.0.0.1", 1883, "dashboard")
        MQTT_SUBSCRIBE(h, "werkstatt/#")
        verbunden = TRUE
    CATCH e
        naechsterVersuch = MILLIS() + 3000
    END TRY
END SUB
' je Bild:
IF verbunden THEN
    MQTT_UPDATE(h)
    IF NOT MQTT_IS_CONNECTED(h) THEN verbunden = FALSE : naechsterVersuch = MILLIS() + 3000
ELSEIF MILLIS() >= naechsterVersuch THEN
    verbinden()
END IF
```

## QoS 1: sicher ankommen

Mit QoS 1 bestätigt der Broker jede Nachricht. Bis dahin zählt sie in
`MQTT_PENDING`; `MQTT_FLUSH` wartet darauf — nützlich in einem Programm, das
eine Nachricht schickt und gleich endet. Abonniert man mit QoS 1, bestätigt
der Client eingehende Nachrichten selbst. QoS 1 heißt „mindestens einmal":
geht eine Bestätigung verloren, kommt dieselbe Nachricht ein zweites Mal.

```basic
IMPORT "mqtt"
DIM h AS MQTT_HANDLE : h = MQTT_CONNECT("192.168.1.50", 1883, "werkstatt-steuerung")
MQTT_PUBLISH(h, "werkstatt/motor", "stopp", FALSE, 1)
IF NOT MQTT_FLUSH(h, 3000) THEN PRINT "nicht bestaetigt: "; MQTT_PENDING(h)
MQTT_DISCONNECT(h)
```

Bricht die Verbindung ab, bevor die Bestätigung kam, schickt der Client die
Nachricht nicht von selbst neu; `MQTT_PENDING` zeigt, ob etwas offen war.

## Verschlüsselt: mqtts://

`MQTT_CONNECT("mqtts://broker.example.com", 8883, ...)` verbindet über TLS
(auch `ssl://`, `tls://`; `mqtt://` oder nur der Name = unverschlüsselt). Das
Zertifikat des Brokers wird gegen die öffentlichen Wurzeln geprüft — so gehen
HiveMQ Cloud, EMQX Cloud, AWS IoT. Ein Mosquitto mit selbst ausgestelltem
Zertifikat braucht dessen CA: `MQTT_TLS_CA("ca.crt")` vor dem Verbinden, dann
gilt NUR diese CA, bis `MQTT_TLS_CA("")` zurückstellt. Der Name in der
Adresse muss im Zertifikat stehen (Name oder IP-Adresse).

```basic
IMPORT "mqtt"
MQTT_TLS_CA("mosquitto-ca.crt")
DIM h AS MQTT_HANDLE
h = MQTT_CONNECT("mqtts://raspberrypi.local", 8883, "werkstatt", 60, "nutzer", "kennwort")
```

## Der letzte Wille

Ein Board meldet beim Start `online` mit `retain` -- und wenn ihm der Strom
ausgeht, bleibt dieses `online` beim Broker liegen: jedes Dashboard, das
spaeter startet, haelt das tote Geraet fuer lebendig. Dagegen hilft der
**letzte Wille**: eine Nachricht, die der Client beim Verbinden hinterlegt
und die der BROKER veroeffentlicht, sobald die Verbindung ohne Abmeldung
abreisst (Absturz, Strom, Netz). Ein sauberes `MQTT_DISCONNECT` loest ihn
nicht aus.

```basic
IMPORT "mqtt"
MQTT_WILL("werkstatt/status", "offline", TRUE)   ' gilt fuer das naechste CONNECT
DIM h AS MQTT_HANDLE : h = MQTT_CONNECT("127.0.0.1", 1883, "werkstatt-sensor")
MQTT_PUBLISH(h, "werkstatt/status", "online", TRUE)
```

Der Wille gilt fuer genau ein `MQTT_CONNECT` -- wer neu verbindet, setzt ihn
neu (die ESP32-Bibliotheken machen es genauso: `setWill` vor `connect`).

## Komplettes Beispiel

Siehe [examples/148_mqtt.dh](../examples/148_mqtt.dh) (Round-Trip gegen
einen lokalen Broker — z. B. [Mosquitto](https://mosquitto.org/), Default-Port
1883, kein Login nötig für einen lokalen Test-Broker).

## Ein echtes Board anbinden

[esp32/](../esp32/) enthält ein fertiges Grundgerüst für ESP32/ESP8266
(WLAN, Broker-Verbindung, Wiederverbinden, Empfang) mit vier markierten
Stellen für eigenen Code, dazu das Drachenhauch-Gegenstück
[examples/159_esp32_bruecke.dh](../examples/159_esp32_bruecke.dh).

Dort stehen auch die Fallstricke, die man sonst einzeln durchleidet:
eindeutige Client-Kennung, die 256-Byte-Grenze von PubSubClient, warum
`delay()` in `loop()` das Board für den Broker sterben lässt, und warum man
seine eigenen Nachrichten zurückbekommt, wenn man einen Platzhalter abonniert.
