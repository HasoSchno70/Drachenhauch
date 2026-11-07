# Geraete und Netz fuer Bastler

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Tray, serieller Monitor, Firmata, MQTT, Raspberry Pi, Mikrofon, Webcam, WebSocket, OSC, mDNS, Modbus, MIDI-Uhr.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Tray-Symbol auf allen drei Systemen (2026-10-08)

`tray.rs`, Feature `tray` (in `graphics` enthalten, eigenes Feature wie
`a11y`, damit die CI es ohne raylib baut). Windows: `Shell_NotifyIconW` am
raylib-Fenster (seit 2026-10-04). **macOS:** `NSStatusItem` nur ueber die
Objective-C-Laufzeit (wie `finder.rs`), Zielklasse `DHTrayZiel` fuer Klick und
Menue; Rechts/Ctrl+Klick zeigt das Menue (`popUpStatusItemMenu:`), Ereignisse
kommen in GLFWs Ereignisschleife im Hauptfaden an. **Linux:**
StatusNotifierItem + `com.canonical.dbusmenu` ueber zbus 5 (blockierend,
dieselbe Fassung wie AccessKit), Anmeldung beim `StatusNotifierWatcher` und
erneut, wenn er (neu) erscheint (`waechter`-Faden auf NameOwnerChanged);
`TRAY_HIDE` gibt nur den Busnamen ab, die Verbindung bleibt. Ohne Leiste kein
Fehler, ohne D-Bus-Sitzung schon. **`dhrt pruef tray`** laesst das Symbol des
Systems echt durchlaufen (Linux gegen einen eigenen `dbus-daemon` mit einem
Waechter und einem fremden Teilnehmer; macOS mit echten AppKit-Objekten,
`performClick:`/`performActionForItemAtIndex:`), die Faelle in `tray.dhtest`
rufen es je System; die CI baut dafuer `tray` mit. Gegengeprueft unter Linux
mit GLibs `gdbus` als fremdem Leser (Eigenschaften, Layout `(u(ia{sv}av))`,
Event). Lokal laeuft Linux in der WSL (`~/.cargo`, `CARGO_TARGET_DIR=$HOME/dht`).

## Serieller Monitor in der IDE, `IMPORT ... OPTIONAL` (2026-11-04)

Erster Punkt der Bastler-Liste nach 2026.42. Die IDE hat einen Reiter
**Seriell** neben Ausgabe/Terminal (Alt+4, `serZeigen`): Klappliste mit
Port (bearbeitbar, alle 2 s neu aus `SERIAL_PORTS`, solange nichts verbunden
ist) oder `tcp:host:port` (Netzbruecke, `NET_TCP_CONNECT` -- ueber diesen
Weg prueft sich der Monitor ohne Geraet), Baud, Verbinden, Kurve, Zeit,
Leeren; unten Senden mit Zeilenende. `seriellNachziehen` je Bild:
`SERIAL_AVAILABLE` + `SERIAL_READ` bzw. `NET_RECV`, Zeilen ueber `serText`
(CR LF/LF), ein Rest ohne Ende nach 0,5 s. **Kurve** (`serKurveLesen`): eine
Zeile ist Daten, wenn ALLE Teile Zahlen sind (Trenner Leerraum/Komma/
Semikolon, `name:wert`, `Name: 22` wird zu `Name:22`) -- sonst Text, wie
der Plotter der Arduino-IDE; bis 8 Reihen in einem flachen Ring
(`serKWert[r * SER_PUNKTE + i]`, 600 Werte), eine Reihe traegt ihren letzten
Wert weiter, `serKAb` merkt, ab wann es sie gibt. Gezeichnet in ein IMAGE
hinter einem `GUI_IMAGE` (`serKurveZeichnen`, Leinwand-Befehle, hoechstens
jedes dritte Bild) -- so liegt die Kurve in der Fensterreihenfolge der gui
statt ueber allem wie eine Zeichenflaeche. `serVorStart` in
`prozessStartenDatei`: haelt der Monitor einen PORT und ruft das Programm
`SERIAL_OPEN`/`FIRMATA_OPEN`, gibt er ihn frei und verbindet nach dem
Programm wieder (`serWieder`). Port, Baud, Ende, Kurve, Zeit in der ide.json
(`seriell_*`, direkt in `konfig` gesetzt). **Der Laufzeit-Baustein dazu:**
die IDE konnte `serial` nicht WAHLWEISE benutzen -- ohne `IMPORT "serial"`
warnte `--check` an jedem Aufruf, mit ihm jeder Bau ohne `--hardware` (die
CI, bis dahin auch die macOS-/Linux-Pakete) beim Start. Neu
**`IMPORT "modul" OPTIONAL`** (Regex-Gruppe 3 in `preprocess::import_re`;
`NUR_OPTIONAL` je Vorverarbeitung -- ein zweiter, schlichter Import hebt es
auf; an einer `.dh`-Datei faellt die Zeile weg, wenn es sie nicht gibt) und
**`MODULE_AVAILABLE(modul$)`** (`preprocess::modul_verfuegbar`). Die Pakete
fuer macOS/Linux bauen seither mit `serial midi` (Linux: `libudev-dev`).
Tests `tests/pruef/sprache/import_optional.dhtest` (6; vier Verfaelschungen fallen
je in ihrem Fall) und `tests/pruef/ide/werkzeug_ide_seriell.dhtest` (4: ein
Einschub spielt das Geraet ueber `NET_TCP_LISTEN(0)` + `NET_LISTENER_PORT`;
fuenf Verfaelschungen der IDE fallen je in ihrem Fall). Mit echtem Arduino
ungeprueft.

## Firmata: Servo, I2C, Schrittmotor, Netz (2026-11-05)

Zweiter Punkt der Bastler-Liste. `FIRMATA_OPEN` nimmt neben dem seriellen
Port `tcp:host:port` (`firmata::Leitung` Seriell|Tcp, TCP ohne Warten gelesen
und geschrieben -- ein WouldBlock beim Schreiben wartet kurz; Ok(0) beim
Lesen = Gegenseite hat aufgelegt, ein Fehler). Der Netzweg braucht kein
Feature: `mod firmata` ist ungated, `hardware_module_compiled("firmata")`
ist immer wahr, nur `seriell_oeffnen`/`ports` haengen an `serial` und
melden sonst "nur tcp:". Uebers Netz wird nicht 2 s auf den Autoreset
gewartet. Neu: **Servo** (`SERVO_CONFIG` 0x70, Vorgabe 544/2400 wie die
Arduino-Bibliothek; Winkel geht als Analogwert), **EXTENDED_ANALOG** (0x6F)
fuer Pins ab 16 beim Schreiben, **I2C** (0x78 Config -- die erste Anfrage
schickt es selbst, sonst ignoriert StandardFirmata alles --, 0x76 mit Art
in Bits 3-4, 10-Bit-Adresse Bit 5 + obere drei Bits; Antwort 0x77 Adresse/
Register/Daten in 7-Bit-Paaren; ohne Register meldet StandardFirmata
Register 0, darum `i2c_letzte` je Adresse neben `i2c_daten` je (Adresse,
Register); `FIRMATA_I2C_READ` wartet hoechstens 1 s auf eine Antwort mit
NEUERER Nummer als vor der Frage -- `i2c_zaehler`), **Firmware** (0x79, aus
der Meldung beim Start, sonst gefragt) und **Schrittmotor** ueber
AccelStepperFirmata 0x62 (NICHT in StandardFirmata, braucht
ConfigurableFirmata): 32-Bit-Zahlen als Betrag + Vorzeichenbit 0x08 im
fuenften Byte (kein Zweierkomplement, wie firmata.js), Kommazahlen als
23 Bit Ganzzahl * 10^(Exp-11) mit Vorzeichen in Bit 6 des vierten Bytes;
"fertig" (0x0A) setzt `stepper_laeuft` zurueck. Beispiel 147 nimmt das Ziel
als Argument -- dabei fiel ein alter Fehler auf: `INSTR(ports, ",") > 0` und
`LEFT$(ports, komma - 1)` machten aus "COM3,COM4" "COM" (INSTR zaehlt ab 0,
-1 = nicht da). Tests `tests/pruef/geraete/modules_firmata.dhtest` (15): das Board
spielt `_hilfen/firmataboard.dh` im SELBEN Programm (`NET_TCP_LISTEN`, dann
`FIRMATA_OPEN("tcp:...")`, dann `NET_TCP_ACCEPT` -- die Verbindung steht
schon im Rueckstau) und vergleicht die Bytes in Hex; Beispiel 147 als
eigener Prozess gegen ein gespieltes Board. Neun Verfaelschungen von
firmata.rs fallen je in ihrem Fall. Mit echtem Board ungeprueft.

## MQTT: QoS 1, Abmelden, TLS (2026-11-05)

Dritter Punkt der Bastler-Liste. `mqtt::Strom` (Klar|Tls wie in smtp.rs);
`MQTT_CONNECT("mqtts://host", ...)` (auch `ssl://`/`tls://`, `ziel_lesen`)
liest waehrend Handschlag und CONNACK mit 5 s Zeitgrenze und stellt danach
auf 1 ms -- vorher, mit 1 ms von Anfang an, scheiterte der Handschlag am
Warten. Ein falsches Zertifikat zeigt sich als SCHREIBfehler des CONNECT
(dort laeuft der Handschlag), darum die Meldung "TLS-Verbindung
fehlgeschlagen" an beiden Stellen. `MQTT_TLS_CA(pfad$)` (Vm::mqtt_ca, gilt
fuer jedes folgende Verbinden, `""` zurueck): dann NUR diese CA, PEM ueber
`rustls::pki_types::pem`. Das Feature `net` bringt dafuer rustls +
webpki-roots mit (waren ueber smtp/pg/ureq schon im Baum, Lockfile
unveraendert). **QoS 1:** `encode_publish` mit QoS-Bits und Paketnummer
hinter dem Topic; `unbestaetigt` (Nummer -> Zeit) bis PUBACK, eine
Nummer, die noch offen ist, wird nicht wieder vergeben; eingehende QoS-1-
Nachrichten bekommen ihr PUBACK in `update`. Kein Neusenden (Clean Session
ist immer an) -- `MQTT_PENDING` zeigt, was offen ist, `MQTT_FLUSH(h[, ms])`
wartet. `update` liest jetzt, bis nichts mehr da ist (vorher ein Stueck je
Bild). `MQTT_UNSUBSCRIBE` (0xA2, UNSUBACK 0xB0). QoS 2 ist ein Fehler mit
Hinweis. Tests: `modules_mqtt.dhtest` (+4; der Test-Broker
`_hilfen/mqttbroker.dh` kann QoS 1 -- bestaetigt und schickt mit Nummer 77
zurueck --, UNSUBSCRIBE, Modus `ohneack`, und schreibt jede Paketart nach
`PORTDATEI.log`); TLS nur in Rust-Tests, weil das net-Modul kein TLS
bedient: `tls_broker` ist ein rustls-SERVER mit fester Test-CA
(`rust/drachenhauch_runtime/testdaten/mqtt_tls`, Schluessel der CA nach dem
Ausstellen geloescht, gueltig bis 2126) -- Rundweg mit eigener CA und
Ablehnung ohne. Sechs Verfaelschungen fallen je in ihrem Fall. Gegen einen
echten Broker mit TLS ungeprueft.

## Raspberry Pi: Pins, GLES-Bau, ARM-Paket (2026-11-06)

Vierter Punkt der Bastler-Liste. **Module `gpio`, `i2c`, `spi`** (`pins.rs`):
nicht die Register des Pi, sondern die Schnittstellen des Kernels --
`/dev/gpiochipN` (uAPI v2, je Pin eine eigene Leitungsanforderung,
`GPIO_V2_LINE_SET_CONFIG` zum Umstellen), `/dev/i2c-N` (SMBus zuerst --
der Kernel bildet ihn auf jedem I2C-Bus nach, und nur er laesst sich mit
`i2c-stub` pruefen --, rohes I2C/`I2C_RDWR` fuer den Rest; `i2c_weg` ist die
reine Entscheidung), `/dev/spidevB.C`. So geht es auf Pi 3/4/5 (drei
verschiedene GPIO-Chips; `GPIO_OPEN()` nimmt per `standard_chip` den mit der
Stiftleiste, `pinctrl-rp1`/`-bcm2711`/`-bcm2835`) und laesst sich ohne Pi
pruefen. Unter Linux in jedem Bau ohne Feature (Abhaengigkeit `libc` nur
fuer linux), anderswo eine Stub-Fassung mit "gibt es nur unter Linux";
`MODULE_AVAILABLE` und die IMPORT-Warnung (`hardware_missing_msg` mit
eigenem Satz fuer diese drei) wissen es. Befehlsfamilie 29
(`BUILTIN_FAMILIEN` = 30). Ein beschriebener Pin wird von selbst Ausgang,
ein gelesener Eingang; `GPIO_WATCH` + `GPIO_NEXT`/`GPIO_EVENT_*` holen
Flanken mit Kernel-Zeit (Schlange 1024, aelteste faellt weg), entprellt auf
Wunsch der Kernel. Die ioctl-Strukturen haben `const`-Asserts auf die
Groessen des Kernels (die Groesse steckt in der Nummer). Tests
`tests/pruef/geraete/pins.dhtest` (11): Faelle "sim" gegen `gpio-sim` und `i2c-stub`,
die `tests/pruef/_hilfen/pins_simulieren.sh` auf dem Linux-Laeufer der CI
einrichtet (DH_TEST_GPIO/_GPIO_SIM/_I2C; ohne sie ueberspringen sie sich --
lokal geht es nicht, der WSL-Kern hat beides nicht); 6 Rust-Tests.
**Ungeprueft:** SPI (kein Simulator) und alles auf echter Pi-Hardware.

**GLES-Bau** (Feature `gles` = `graphics` + `raylib/opengl_es_30`): die
Shader schreibt `fuer_ziel_uebersetzen` wie im Web auf `#version 300 es`
um; `dhrt --version` sagt "grafik (OpenGL ES)". `package.yml` baut auf
`ubuntu-24.04-arm` damit das Paket `linux-aarch64` und faehrt die
Grafik-Sammlungen unter xvfb (beide Linux-Laeufer). Lokal geprueft in der
WSL gegen Mesa llvmpipe (OpenGL ES 3.2) -- dafuer stehen dort jetzt die
raylib-Bauabhaengigkeiten, gdb und xvfb; Ziele `$HOME/dht_gl`/`dht_gles`.
**Der Fund dabei betrifft JEDES Linux-Paket:** das mit gcc uebersetzte
`rlLoadTexture` ruft fuer `data = NULL` `glCompressedTexImage2D` (gdb-
Rueckverfolgung), jedes Render-Ziel war "incomplete attachment" -- POSTFX
und RENDERTARGET blieben unter Linux schwarz, der HDR-Umgebung fehlte die
BRDF-Tabelle, und niemand sah es, weil die Grafik-Proben nur unter Windows
liefen. Jetzt legt `render_ziel` (graphics.rs) jedes Ziel selbst an, mit
leerem Puffer (`leere_textur`), auf allen Systemen gleich; ein Ziel, das
nicht vollstaendig wird, ist ein Fehler statt eines schwarzen Bildes.
`FONT_TO_IMAGE` liest die Zeichenbilder (`GlyphInfo.image`) statt den Atlas
zurueck -- unter OpenGL ES ist ein Grauwert-Atlas nicht lesbar.
`DHRT_RAYLIB_LOG=info` zeigt, was raylib beim Anlegen sagt (Treiber,
Fassung). **Bekannter Rest:** eine 1-Pixel-Linie genau auf einer
Pixelgrenze faellt unter GLES einen Punkt anders als auf dem Desktop
(Windows nimmt bei Senkrechten den linken, bei Waagerechten den unteren
Punkt -- kein fester Versatz bildet das nach); zwei Bildproben
(`grafik_formen_ausbau` gestrichelt, `gui_tabelle_kante`) fallen darum nur
unter GLES und stehen nicht im ARM-Lauf. Vergleich ueber 122 Grafik-
Sammlungen: Linux-GL und Linux-GLES fallen danach exakt gleich (die
uebrigen Unterschiede zu Windows sind Linux-Eigenheiten). Tests
`tests/pruef/grafik/gles.dhtest` (4); drei Gegenproben (NULL-Textur, leere
Zeichenbilder, kein Umschreiben der Shader) fallen je in ihrem Fall.
Doku `docs/raspberry-pi.md`, `docs/module-pins.md`, Beispiel
`examples/211_raspberry_pi.dh`, Lehrbuch Kapitel 76.

## Mikrofon und Webcam (2026-11-06)

Fuenfter Punkt der Bastler-Liste. **Mikrofon** (`mikrofon.rs`, Befehle
`MIC_*` im Modul `audio`): cpal (dieselbe 0.18.1 wie Kira, jetzt direkte
Abhaengigkeit im graphics-Feature) in einem eigenen Faden -- ein cpal-Strom
ist nicht ueberall `Send`, er lebt im Faden bis MIC_STOP; der Rueckruf mischt
die Kanaele und schiebt die Werte in EINEN Puffer (`PUFFER`: Ring 8192,
Spitze seit der letzten Frage, Aufnahme). `MIC_PITCH` = YIN (Differenz-
funktion auf ihren laufenden Mittelwert bezogen, Schwelle 0,15, oertliches
Minimum, parabolisch verfeinert; 50..2000 Hz, 0 bei Effektivwert < 0,01),
`MIC_FFT` = dieselbe Rechnung wie AUDIO_FFT (`audio::baender_rechnen`,
herausgeloest, eigener Zustand `mic_baender`/`mic_agc` in der VM),
`MIC_RECORD_STOP` -> SOUND (braucht die Ausgabe), `MIC_RECORD_SAVE` -> WAV
ohne Ausgabe. Ohne laufendes Mikrofon ist jede Abfrage ein Fehler (eine
stille Null waere "es ist leise"). **`DHRT_MIKROFON=datei.wav`** spielt ein
Mikrofon (einziges Geraet "Datei: name", Echtzeit, Schleife, derselbe Puffer)
-- dieser Rechner hat keinen Eingang, die CI auch nicht. Tests
`tests/pruef/ton/mikrofon.dhtest` (13; Toene schreibt `_hilfen/tonwav.dh` als
Bibliothek, eingebunden per Beilage), 4 Rust-Tests; vier Verfaelschungen
(Spitze nie zurueck, Aufnahme ohne Grenze, Tonhoehe ohne Feinrechnung,
Wellenform die aeltesten) fallen je in ihrem Fall -- die Tonhoehen-Probe
musste dafuer auf 0,2 % bei 1250 Hz.

**Webcam** (Modul `webcam`, `webcam.rs`, Typ WEBCAM): je System ein Weg in
einem eigenen Faden, der RGBA in eine `Ablage` legt; `WEBCAM_UPDATE`/
`WEBCAM_IMAGE` schreiben es in ein IMAGE (`image_punkte_setzen` wie beim
Video; neue Groesse = neues IMAGE). Linux V4L2 (Strukturen mit const-Asserts:
capability 104, format 208, requestbuffers 20, buffer 88; mmap, 4 Puffer,
poll; YUYV/MJPG/RGB3/BGR3/XR24/AR24/BGR4, ueber 640x480 MJPEG zuerst),
Windows Media Foundation (Source Reader mit VIDEO_PROCESSING nach RGB32,
negativer Stride = unten zuerst), macOS AVFoundation ueber die
Objective-C-Laufzeit (Delegat-Klasse `DHKameraZiel`, BGRA, Dispatch-Schlange).
MJPEG ueber zune-jpeg (schon im Baum): **zune setzt die Standard-Huffman-
Tafeln nur bei der Kennung "AVI1"** -- fehlt die DHT, kommt ein APP0 "AVI1"
hinter den Anfang (`mjpeg_vorbereiten`; vom Rust-Test gefunden).
**`DHRT_WEBCAM=test`** ersetzt alle Kameras durch "Testbild" (acht
Farbbalken, ein Quadrat wandert 8 Punkte je Bild, 30/s). Eigenes Feature
`webcam` (graphics schaltet es ein), damit die CI den Weg jedes Systems ohne
raylib uebersetzt (`cargo check/test --features webcam` im rust-check);
geprueft hier fuer Linux in der WSL und fuer macOS in einer Kopie ohne
mimalloc (dessen C-Bau scheitert fuer fremde Ziele). Linux-Paketlauf:
`tests/pruef/_hilfen/kamera_simulieren.sh` laedt **vivid** (Testtreiber des
Kernels, linux-modules-extra) und setzt `DH_TEST_WEBCAM=vivid`. Tests
`tests/pruef/geraete/webcam.dhtest` (10, zwei davon gegen vivid), 6 Rust-Tests.
**Ungeprueft:** jede echte Kamera (dieser Rechner hat keine) und der
cpal-Eingang mit echtem Mikrofon. macOS: Info.plist traegt
NSMicrophone-/NSCameraUsageDescription, `dhrt.entitlements` die
Berechtigungen `device.audio-input`/`device.camera` -- mit Hardened Runtime
liefert macOS sonst still Stille bzw. kein Bild (Probe in
`werkzeug_paket.dhtest`). Doku `docs/module-mikrofon.md`,
`docs/module-webcam.md`, Beispiele `examples/212_stimmgeraet.dh`,
`examples/213_webcam.dh`, Lehrbuch Kapitel 71 und 76.

## WebSocket, OSC und mDNS (2026-11-06)

Sechster Punkt der Bastler-Liste, drei Module, eine Befehlsfamilie
(`try_netzdienste`, Familie 32, `BUILTIN_FAMILIEN` = 33). **`websocket`**
(`websocket.rs`, Feature `net`, Typen WEBSOCKET/WS_SERVER): eigene Umsetzung
nach RFC 6455 -- Client `ws://`/`wss://` (rustls wie mqtt), Server ueber
`WS_LISTEN`/`WS_ACCEPT` (ohne Warten; der Handschlag einer angenommenen
Verbindung wartet hoechstens 3 s, eine Anfrage ohne Upgrade bekommt 400 und
zaehlt nicht). Nach dem Handschlag ist der Strom nicht wartend; jeder
WS_NEXT/WS_CONNECTED pumpt (Rahmen zusammensetzen, Ping -> Pong, Schliessen
beantworten), Cursor-Muster wie MQTT. **Maskierung je Seite geprueft** (ein
Server muss unmaskierte Client-Rahmen mit 1002 abweisen, RFC 6455 5.1 --
erst beim Durchsehen nachgetragen). Grenze 16 MB. `WS_CONNECT` fragt fuer
`.local` selbst ueber mDNS, wenn das System den Namen nicht kennt (Linux
ohne nss-mdns). **`osc`** (`osc.rs`, ungated, Typ OSC): OSC 1.0 ueber UDP,
liest i f s b h d t T F N I c r m S und Buendel (verschachtelt, einzeln
abgeliefert), schreibt aus DH-Werten (INTEGER ausserhalb 32 Bit -> h),
`OSC_MATCH` mit ? * [] [!] {}. Ein Windows-10054 nach einem Paket an einen
geschlossenen Port ist kein Fehler. **`mdns`** (`mdns.rs`, ungated):
Fragen per **legacy unicast** (RFC 6762 6.7) von einem gewoehnlichen
Anschluss -- 5353 braucht nur, wer ankuendigt (`socket2` mit
SO_REUSEADDR/-PORT, Feature `all`, schon im Baum); Wiederholung nach 0/300/
900 ms, fehlende SRV/A werden nachgefragt; der Antworter (Faden) antwortet
direkt mit Kennung, wenn die Frage nicht von 5353 kam, sonst an die Gruppe,
meldet sich beim Ankuendigen einmal von selbst und zieht beim Stopp mit
TTL 0 zurueck. **`DHRT_MDNS_PORT=n`** laesst beides ueber 127.0.0.1:n
laufen. Tests `tests/pruef/geraete/websocket.dhtest` (7: Gegenserver
`_hilfen/wsecho.dh` als Prozess, roher Client ueber net mit dem
RFC-Schluessel, **Node ab 22 als fremder Client**), `osc.dhtest` (5:
Leitungsformat Byte fuer Byte gegen die Spezifikation, von Hand gebaute
Buendel), `mdns.dhtest` (6, seriell; der Fall mit echtem Multicast auf 5353
lief hier, ohne Multicast ueberspringt er sich); Rust-Tests mit den
Pruefwerten aus RFC 6455 und OSC 1.0. Doku `docs/module-websocket.md`,
`module-osc.md`, `module-mdns.md`, Beispiele `examples/214_websocket_chat.dh`
(httpd + WebSocket + mDNS, mit curl und Node nachgefahren) und
`examples/215_osc_fernbedienung.dh`, Lehrbuch Kapitel 73.

## Modbus, MIDI-Uhr und .mid-Dateien, Ein-/Ausstecken (2026-11-06)

Siebter Punkt der Bastler-Liste, eine neue Befehlsfamilie (`try_geraete`,
Familie 33, `BUILTIN_FAMILIEN` = 34) fuer `modbus_`/`hotplug_`; MIDI bleibt
in Familie 11. **`modbus`** (`modbus.rs`, in JEDEM Bau, Typ MODBUS):
`MODBUS_TCP` (MBAP-Kopf, Kennung je Anfrage hochgezaehlt; eine Antwort mit
fremder Kennung gehoert zu einer zu spaeten Frage und wird uebergangen) und
`MODBUS_RTU` (seriell mit Feature `serial`, sonst `"tcp:host:port"` = RTU-
Rahmen ueber einen Umsetzer im Netz -- so ist RTU auch ohne Geraet
pruefbar). RTU kennt keine Laenge im Rahmen: `rtu_laenge` rechnet sie aus
den ersten drei Bytes, vor jeder Anfrage wird der Eingang geleert, CRC-16
der Antwort wird geprueft, Adresse 0 = an alle (Schreiben wartet nicht,
Lesen ist ein Fehler). Funktionen 1..6, 15, 16, Bereiche VOR dem Senden
geprueft (`bereich`: 2000/125/1968/123, nicht ueber 65535), Ausnahmen im
Klartext (`ausnahme_text`). 32-Bit-Werte: `folge_lesen` (ABCD/CDAB/BADC/
DCBA, `zwei_register`/`als_register`), `MODBUS_FLOAT/INT32/UINT32` lesen
aus einem Feld ab einer Stelle (zaehlt wie das Feld, `unten_von`),
`_REGS` liefert zwei Register zum Schreiben. Jeder Befehl WARTET auf die
Antwort -- darum spielt die Pruefung das Geraet als eigenen Prozess
(`tests/pruef/_hilfen/modbusgeraet.dh`, eigene CRC in Drachenhauch,
Anfragen in Hex nach `anfragen.txt`, VOR der Antwort geschrieben;
Stoerungen stumm/kaputt/fremd/kurz). **Falle im Beispiel:** ein
Verbindungsaufbau ins Leere dauert unter Windows ~2 s -- wer den
Wiederholungstakt VOR dem Versuch merkt, versucht in jedem Bild neu.

**MIDI-Uhr** (`midi.rs`, `Takt` rein mit Rust-Tests): `in_open` laesst nur
noch SysEx weg (vorher `Ignore::All`); F8/F1/FE gehen nur in den Takt (eigene
`Instant`-Uhr im Rueckruf, midirs Zeitstempel hat keinen bekannten Anfang),
FA/FB/FC/F2 zusaetzlich weiter in die Warteschlange -- wie vor 2026.43, als
midir nur F1/F8/FE filterte. Tempo = Mittel der letzten 24 Abstaende, 0 nach
0,5 s Stille, eine Pause ueber 1 s beginnt das Mittel neu.
`MIDI_CLOCK_SEND` (`takte_schlagen`: Schlag n auf n * Abstand ab festem
Anfang, faellt er mehr als einen Abstand zurueck, zaehlt er neu; Tempo per
`AtomicU64`, ein neues Tempo ohne Neustart). Der Ausgang haelt die
Verbindung jetzt hinter `Arc<Mutex>` (Uhr- und Spieler-Faden senden mit),
`Drop` haelt beide Faeden an. **`.mid`-Dateien** (`midifile.rs`, ungated,
Typ MIDI_FILE): lesen Format 0/1/2 mit laufendem Status (Meta/SysEx heben
ihn auf), Tempo-Karte ueber alle Spuren, SMPTE (29 = 29,97); Noten gepaart
(Note-aus beendet die AELTESTE offene derselben Spur/Kanal/Taste), alles nach
aussen in Sekunden, Schreiben in Vierteln (Format 1, Tempo in Spur 0).
Zur selben Zeit geordnet nach `rang`: Meta, Note-aus, Programm/Regler,
Note-an -- beim Schreiben UND in der Ansicht (sonst klingt eine Note mit dem
alten Programm bzw. schneidet sich eine wiederholte selbst ab).
`MIDI_FILE_PLAY` spielt die Ansicht in einem Faden ab und schaltet beim
Anhalten klingende Noten aus. Die IMPORT-Warnung fuer `midi` sagt in einem
Bau ohne das Feature jetzt "nur zum Teil" (Dateien und Notenrechnung gehen).
**`hotplug`** (`hotplug.rs`): ein Faden vergleicht alle 500 ms die Listen
(serialport, midir, `usb::list`), geordnet und ohne Doppelte (ein HID-Geraet
meldet sich je Schnittstelle), Ausgangslage beim ersten `HOTPLUG_`-Aufruf
ohne Meldung, hoechstens 256 Meldungen; `DHRT_GERAETE=datei` (`art:name` je
Zeile) ersetzt die Listen fuer Pruefungen. Tests `modbus.dhtest` (7),
`midi_dateien.dhtest` (5, Beispieldatei der SMF-Spezifikation, Schreiben Byte
fuer Byte), `hotplug.dhtest` (3), zwei Loopback-Faelle in
`modules_midi.dhtest` (Uhr mit Tempowechsel, Abspielen samt Anhalten;
loopMIDI), Rust-Tests mit den Beispielen der Spezifikationen; 13
Verfaelschungen fallen je in ihrem Fall (die Tempo-Verfaelschung verdeckte
anfangs die "Note-aus beim Anhalten" -- die Datei lief halb so schnell).
Beispiele `216_modbus_monitor.dh`, `217_midi_datei.dh` (Klavierrolle,
eingebauter Synthesizer auf einer Audio-Uhr), `218_geraete_melden.dh`; Doku
`docs/module-modbus.md`, `module-hotplug.md`, `module-midi.md`, Buch
Kapitel 76/77. **Ungeprueft:** RTU an einer echten seriellen Schnittstelle,
jedes echte Modbus-Geraet, echtes An-/Abstecken (die Listen selbst stimmen
hier mit COM1, loopMIDI und den HID-Geraeten ueberein).
