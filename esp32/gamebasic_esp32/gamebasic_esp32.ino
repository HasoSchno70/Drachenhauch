// ===========================================================================
//  GameBasic <-> ESP32 -- Grundgeruest
// ===========================================================================
//  Das hier ist der Teil, den man bei jedem Projekt gleich schreibt: WLAN
//  verbinden, Broker verbinden, bei Abbruch wieder verbinden, Nachrichten
//  entgegennehmen. Deine eigene Arbeit kommt an die drei mit >>> markierten
//  Stellen weiter unten.
//
//  Laeuft unveraendert auf ESP32 und ESP8266 (siehe die Weiche gleich unten).
//
//  Vorbereitung in der Arduino-IDE (einmalig):
//    1. Datei -> Grundeinstellungen -> "Zusaetzliche Boardverwalter-URLs",
//       je nach Board (beide gleichzeitig geht auch, mit Komma getrennt):
//         ESP32:   https://espressif.github.io/arduino-esp32/package_esp32_index.json
//         ESP8266: https://arduino.esp8266.com/stable/package_esp8266com_index.json
//    2. Werkzeuge -> Board -> Boardverwalter -> "esp32" bzw. "esp8266"
//    3. Werkzeuge -> Bibliotheken verwalten -> "PubSubClient" (Nick O'Leary)
//
//  Gegenstueck in GameBasic: examples/159_esp32_bruecke.gb
// ===========================================================================

// Laeuft auf ESP32 UND ESP8266. Die Unterschiede stehen alle hier oben, damit
// weiter unten nur noch eine Fassung zu lesen ist -- zwei getrennte Sketche
// waeren binnen kurzem auseinandergelaufen.
#if defined(ARDUINO_ARCH_ESP8266)
  #include <ESP8266WiFi.h>
  // Der ESP8266 hat genau EINEN Analogeingang, und der heisst A0.
  #define ANALOG_PIN A0
#else
  #include <WiFi.h>
  // Beim ESP32 ist fast jeder Pin analogfaehig; 34 ist nur ein Beispiel.
  #define ANALOG_PIN 34
#endif

#include <PubSubClient.h>

// --------------------------------------------------------------- Einstellungen
const char* WLAN_NAME     = "HaSoSchno";
const char* WLAN_PASSWORT = "HIER-DEIN-WLAN-PASSWORT";

const char* BROKER      = "192.168.178.112";   // der PC mit Mosquitto
const uint16_t BROKER_PORT = 1883;

// Muss im ganzen Netz EINDEUTIG sein. Verbinden sich zwei Geraete mit
// derselben Kennung, wirft der Broker abwechselnd eines raus -- und man sucht
// den Fehler stundenlang im eigenen Code. Die MAC-Adresse haengt hinten dran,
// dann ist sie automatisch eindeutig, auch bei einem zweiten Board.
String kennung;

// Themen: worauf wir hoeren, worunter wir senden.
const char* THEMA_BEFEHLE = "esp32/befehl";     // GameBasic -> Board
const char* THEMA_WERTE   = "esp32/wert";       // Board -> GameBasic
const char* THEMA_STATUS  = "esp32/status";     // Lebenszeichen

// Wie oft gesendet wird (Millisekunden). NICHT mit delay() arbeiten -- das
// legt auch die MQTT-Verarbeitung lahm, und der Broker haelt das Board fuer
// tot. Deshalb wird die Zeit mitgezaehlt.
const unsigned long SENDE_ABSTAND = 2000;

WiFiClient netz;
PubSubClient mqtt(netz);
unsigned long letzteSendung = 0;

// --------------------------------------------------------------- WLAN
void wlanVerbinden() {
  if (WiFi.status() == WL_CONNECTED) return;

  Serial.print("WLAN: verbinde mit ");
  Serial.print(WLAN_NAME);
  WiFi.mode(WIFI_STA);            // reiner Client, kein eigener Hotspot
  WiFi.begin(WLAN_NAME, WLAN_PASSWORT);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);                   // hier ist delay in Ordnung: ohne WLAN
    Serial.print(".");            // gibt es ohnehin nichts zu tun
  }
  Serial.println();
  Serial.print("WLAN: verbunden, meine Adresse ist ");
  Serial.println(WiFi.localIP());
}

// --------------------------------------------------------------- Empfang
// Wird vom MQTT-Client aufgerufen, sobald eine Nachricht ankommt.
// ACHTUNG: nutzlast ist KEINE Zeichenkette im C-Sinn -- sie hat kein
// abschliessendes Null-Byte. Wer sie direkt an Serial.println() gibt, bekommt
// Datenmuell hinterher. Deshalb wird sie hier sauber kopiert.
void nachrichtEmpfangen(char* thema, byte* nutzlast, unsigned int laenge) {
  String inhalt;
  inhalt.reserve(laenge);
  for (unsigned int i = 0; i < laenge; i++) inhalt += (char)nutzlast[i];

  Serial.print("<- ");
  Serial.print(thema);
  Serial.print(" = ");
  Serial.println(inhalt);

  // >>> HIER 2: Auf Befehle aus GameBasic reagieren ----------------------
  //
  //   if (inhalt == "an")  digitalWrite(2, HIGH);
  //   if (inhalt == "aus") digitalWrite(2, LOW);
  //
  //   Mehrere Themen unterscheidest du ueber den Parameter `thema`:
  //   if (strcmp(thema, "esp32/licht") == 0) { ... }
  //
  // ----------------------------------------------------------------------
}

// --------------------------------------------------------------- Broker
void brokerVerbinden() {
  while (!mqtt.connected()) {
    Serial.print("MQTT: verbinde mit ");
    Serial.print(BROKER);
    Serial.print(" als ");
    Serial.println(kennung);

    // Letzter Wille: sagt der Broker ALLEN Abonnenten, wenn dieses Board
    // unsauber verschwindet (Stromausfall, Funkloch). Ohne das merkt
    // GameBasic nie, dass der Sensor weg ist -- es kommen nur einfach keine
    // Werte mehr, was genauso aussieht wie "gerade nichts zu melden".
    bool ok = mqtt.connect(kennung.c_str(),
                           nullptr, nullptr,       // Benutzer, Passwort
                           THEMA_STATUS, 0, true,  // Wille: Thema, QoS, behalten
                           "offline");
    if (ok) {
      Serial.println("MQTT: verbunden");
      mqtt.publish(THEMA_STATUS, "online", true);   // true = Broker merkt es sich
      mqtt.subscribe(THEMA_BEFEHLE);

      // >>> HIER 3: Weitere Themen abonnieren ----------------------------
      //   mqtt.subscribe("esp32/licht");
      //   Platzhalter gehen auch:  mqtt.subscribe("esp32/#");
      // ------------------------------------------------------------------
    } else {
      Serial.print("MQTT: fehlgeschlagen, Code ");
      Serial.print(mqtt.state());   // -2 = Netz weg, 5 = nicht erlaubt
      Serial.println(" - neuer Versuch in 3 s");
      delay(3000);
    }
  }
}

// --------------------------------------------------------------- Start
void setup() {
  Serial.begin(115200);
  delay(200);
  Serial.println();
  Serial.println("GameBasic-ESP32-Bruecke startet");

  kennung = "esp32-" + WiFi.macAddress();
  kennung.replace(":", "");

  wlanVerbinden();
  mqtt.setServer(BROKER, BROKER_PORT);
  mqtt.setCallback(nachrichtEmpfangen);

  // PubSubClient wirft Nachrichten ueber 256 Byte STILL weg -- kein Fehler,
  // sie kommen einfach nie an. Fuer laengere Texte (z.B. JSON) hochsetzen.
  mqtt.setBufferSize(512);

  // >>> HIER 1: Deine Hardware einrichten --------------------------------
  //
  //   pinMode(2, OUTPUT);           // eingebaute LED vieler Boards
  //   dht.begin();                  // ein Sensor
  //   servo.attach(13);
  //
  // ----------------------------------------------------------------------
}

// --------------------------------------------------------------- Hauptschleife
void loop() {
  wlanVerbinden();       // WLAN kann jederzeit abreissen
  brokerVerbinden();     // der Broker auch
  mqtt.loop();           // MUSS staendig laufen: hier wird empfangen

  unsigned long jetzt = millis();
  if (jetzt - letzteSendung >= SENDE_ABSTAND) {
    letzteSendung = jetzt;

    // >>> HIER 4: Deine Messwerte schicken -------------------------------
    //
    //   Ersetze die Beispielzeile durch echte Werte, z.B.:
    //     float grad = dht.readTemperature();
    //     mqtt.publish("esp32/temperatur", String(grad, 1).c_str());
    //
    //   MQTT kennt nur Bytes -- Zahlen muessen also in Text. Auf der
    //   GameBasic-Seite holst du sie mit VAL() zurueck.

    long wert = analogRead(ANALOG_PIN);   // Beispiel: roher Analogwert
    mqtt.publish(THEMA_WERTE, String(wert).c_str());

    // --------------------------------------------------------------------

    Serial.print("-> ");
    Serial.print(THEMA_WERTE);
    Serial.print(" = ");
    Serial.println(wert);
  }
}
