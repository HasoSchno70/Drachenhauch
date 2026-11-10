# Drachenhauch 2026.43

*Die Notizen zu dieser Fassung. Sie ist die Fassung **für Bastler**: ein
serieller Monitor in der IDE, Arduino-Pins mit Servo, I2C und Schrittmotor,
MQTT mit TLS, die Pins des Raspberry Pi samt eigenem ARM-Paket, Mikrofon und
Webcam, WebSocket, OSC und mDNS, Modbus, die MIDI-Uhr und MIDI-Dateien, und
Meldungen, wenn ein Gerät angesteckt oder abgezogen wird. Neun Module sind
dazugekommen. Was bestehende Programme bemerken können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Unter Linux funktionieren Render-Ziele jetzt** – `RENDERTARGET_*`,
  `POSTFX` und die HDR-Umgebung (`LIGHT_ENV_HDR`) blieben dort bisher schwarz,
  in jedem Linux-Paket. Die Ursache steckte in raylib (mit gcc übersetzt legte
  es leere Texturen als komprimierte an). Wer unter Linux deshalb auf Effekte
  verzichtet hat, bekommt sie jetzt. Windows und macOS sind unberührt.
* **`FONT_TO_IMAGE`** liest die Zeichenbilder statt des Atlas; das Ergebnis
  ist dasselbe, geht jetzt aber auch unter OpenGL ES.
* **`MQTT_UPDATE` liest alles, was angekommen ist**, nicht mehr ein Stück je
  Aufruf. Wer viele Nachrichten bekommt, sieht sie früher.
* **Ein MIDI-Eingang lässt nur noch SysEx weg.** Uhr-Takte, Zeitcode und
  Active Sensing kommen an, gehen aber in die neue MIDI-Uhr statt in die
  Warteschlange – `MIDI_NEXT` sieht also dasselbe wie bisher.

## Neu in dieser Fassung

### In der IDE

* **Serieller Monitor** (Ansicht → Serieller Monitor, Alt+4, Reiter „Seriell“
  neben dem Terminal): Port oder `tcp:host:port`, Baudrate, Zeilen mit oder
  ohne Uhrzeit, Senden mit wählbarem Zeilenende. **Kurve** wie der Plotter der
  Arduino-IDE: Zahlen, auch als `name:wert`, bis acht Reihen. Startet F5 ein
  Programm, das selbst den Port öffnet, gibt der Monitor ihn frei und
  verbindet danach wieder.

### Sprache

* **`IMPORT "modul" OPTIONAL`** und **`MODULE_AVAILABLE(modul$)`**: ein
  Programm kann ein Hardware-Modul wahlweise benutzen – ein Bau ohne
  `--hardware` warnt dann nicht, und das Programm fragt, ob es da ist.

### Arduino, ESP32, Netz der Dinge

* **Firmata:** Servos (`FIRMATA_SERVO_*`), I2C-Geräte (`FIRMATA_I2C_*`),
  Schrittmotoren über ConfigurableFirmata (`FIRMATA_STEPPER_*`), die Fassung
  der Firmware, und das Board auch über WLAN als `"tcp:host:port"`
  (StandardFirmataWiFi) – das geht in jedem Bau, ohne `--hardware`.
* **MQTT:** QoS 1 mit Bestätigung (`MQTT_PENDING`, `MQTT_FLUSH`),
  `MQTT_UNSUBSCRIBE`, TLS über `mqtts://host`, eigene Zertifizierungsstelle mit
  `MQTT_TLS_CA`.
* **`websocket`:** Client (`ws://`, `wss://`) und Server – eine Webseite oder
  ein ESP32 spricht live mit dem Programm. Ein Name wie `esp32.local` wird
  notfalls über mDNS aufgelöst.
* **`osc`:** Open Sound Control über UDP – TouchOSC und Open Stage Control auf
  dem Telefon, Pure Data, SuperCollider, Lichtpulte.
* **`mdns`:** Geräte beim Namen finden (`MDNS_RESOLVE$("esp32")`), Dienste
  suchen und das eigene Programm ankündigen.
* **`modbus`:** Modbus TCP und RTU (RS-485, oder RTU über einen Umsetzer im
  Netz) – Stromzähler, Wechselrichter, Heizungsregler, SPS. Register und
  Spulen lesen und schreiben, Ausnahmen im Klartext, 32-Bit- und Kommazahlen in
  zwei Registern in allen vier Wortfolgen. In jedem Bau.
* **`hotplug`:** meldet, wenn ein serieller Anschluss, ein MIDI-Gerät oder ein
  USB-HID-Gerät angesteckt oder abgezogen wird (`HOTPLUG_NEXT`).

### Raspberry Pi

* **Pins:** die Module `gpio` (samt Flanken mit Zeitstempel und Entprellen),
  `i2c` und `spi` – über die Schnittstellen des Linux-Kernels, damit auf Pi 3,
  4 und 5 gleich (und auf jedem anderen Linux-Board).
* **Ein Paket für ARM-Linux** (`linux-aarch64`) mit Grafik über OpenGL ES 3.0,
  wie der Pi sie kann. Anleitung im Handbuch unter „Raspberry Pi“.

### Ton und Bild

* **Mikrofon** (`MIC_*` im Modul `audio`): Pegel, Tonhöhe (ein Stimmgerät in
  wenigen Zeilen), Spektrum, Aufnahme als Klang oder WAV.
* **`webcam`:** Bilder einer Kamera in ein `IMAGE` – Video4Linux2 unter Linux,
  Media Foundation unter Windows, AVFoundation unter macOS.
* **MIDI-Uhr:** einer fremden Uhr folgen (`MIDI_CLOCK_BPM`, `_RUNNING`,
  `_TICKS`) oder selbst die Uhr sein (`MIDI_CLOCK_SEND`, `MIDI_START`,
  `MIDI_STOP`, `MIDI_CONTINUE`, `MIDI_SONG_POSITION`), aus einem eigenen
  Faden und damit unabhängig von der Bildrate.
* **MIDI-Dateien** (`.mid`) lesen, schreiben und an einem Ausgang abspielen
  (`MIDI_FILE_*`). Zeiten in Sekunden, Tempowechsel eingerechnet, Noten mit
  Dauer; Lesen und Schreiben gehen in jedem Bau.

### Neue Beispiele

`211_raspberry_pi` (Taster und LED), `212_stimmgeraet`, `213_webcam`,
`214_websocket_chat` (Webseite, WebSocket und mDNS in einem),
`215_osc_fernbedienung`, `216_modbus_monitor`, `217_midi_datei`
(Klavierrolle mit eingebautem Synthesizer), `218_geraete_melden`.

## Unter der Haube

**Zahlen.** 2424 Befehle in 57 Modulen (vorher 2252 in 48; neu `gpio`, `i2c`,
`spi`, `webcam`, `websocket`, `osc`, `mdns`, `modbus`, `hotplug`), 234
Beispiele auf der obersten Ebene von `examples/` (vorher 226), **5930 Fälle in
494 Prüfsammlungen** (vorher 5830 in 482) und 651 Rust-Testfunktionen (608).

* **Geprüft ohne Gerät, wo es geht:** GPIO und I2C gegen die Simulatoren des
  Kernels (`gpio-sim`, `i2c-stub`) auf dem Linux-Läufer der CI, die Kamera
  gegen `vivid` (dort lieferte sie echte Bilder, auf x86 und ARM), das
  Mikrofon über `DHRT_MIKROFON=datei.wav`, Firmata, MQTT und Modbus gegen
  gespielte Gegenstellen, die ihre Prüfsummen selbst rechnen und jedes Byte
  mitschreiben, WebSocket auch gegen den Client von Node, die MIDI-Uhr und das
  Abspielen über einen virtuellen Loopback-Port, das An- und Abstecken über
  `DHRT_GERAETE=datei`.
* Die Pakete für macOS und Linux bauen jetzt mit `serial` und `midi`.
* macOS: die App fragt nach Mikrofon und Kamera (Info.plist und
  Berechtigungen), sonst lieferte sie still Stille bzw. kein Bild.
* Das Lehrbuch hat Einträge für alle neuen Befehle (deutsch und englisch).

## Was offen bleibt

* **Echte Geräte sind weitgehend ungeprüft:** kein echter Raspberry Pi, kein
  SPI (dafür gibt es keinen Simulator), keine echte Kamera, kein
  angeschlossenes Mikrofon, kein Arduino und kein Modbus-Gerät an dieser
  Maschine. Rückmeldungen mit Hardware sind willkommen.
* Unter macOS sieht CoreMIDI neu angesteckte MIDI-Geräte womöglich erst, wenn
  das Programm ein Fenster hat – ungeprüft.
* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
