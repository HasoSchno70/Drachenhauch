# Drachenhauch 2026.30

*Die Notizen zu dieser Fassung. Ein Tag nach 2026.29: die Netz-Runde (ein
Echtzeit-Spiel für zwei, gebaut und geprüft ohne zweiten Rechner) und eine
Zeitgrenze für die Prüfungen. **Eine Änderung betrifft bestehende
Programme** (gleich zuerst).*

## Was sich für bestehende Programme ändert

* **Bricht die Gegenseite einer Verbindung hart ab, ist das kein
  Laufzeitfehler mehr.** Programm beendet, Absturz, Kabel gezogen:
  `NET_RECV` liefert dann `""`, `NET_RECV_BYTES` einen leeren Puffer,
  `NET_SEND` 0, und `NET_IS_CONNECTED` wird FALSE — so stand es schon in der
  Doku. Bisher warfen `NET_RECV` und `NET_SEND` „Eine vorhandene Verbindung
  wurde vom Remotehost geschlossen“, und unter Windows warf auch
  `NET_UDP_RECV`, sobald ein früheres Paket an einen Port gegangen war, an
  dem niemand mehr lauschte. Ein Spielwirt starb so, sobald sein Gast das
  Fenster schloss. Wer den Fehler mit TRY abgefangen hat, fragt jetzt
  `NET_IS_CONNECTED`; bei UDP sagt nur die Zeit seit dem letzten Paket, ob
  der andere noch da ist.

  ```basic
  IMPORT "net"
  DIM lst AS NET_LISTENER : lst = NET_TCP_LISTEN(0, "127.0.0.1")
  DIM gast AS NET_SOCKET : gast = NET_TCP_CONNECT("127.0.0.1", NET_LISTENER_PORT(lst))
  SLEEP(50)
  DIM s AS NET_SOCKET : s = NET_TCP_ACCEPT(lst)
  NET_CLOSE(gast)
  SLEEP(100)
  DIM t AS STRING : t = NET_RECV(s, 100)
  IF NOT NET_IS_CONNECTED(s) THEN PRINT "Gast ist weg"
  ```

## Neu in dieser Fassung

### Netz

* **UDP mit Bytes:** `NET_UDP_SEND` nimmt jetzt auch einen `BUFFER`, und
  **`NET_UDP_RECV_BYTES`** liefert ein Datagramm als `BUFFER`. Ein
  Spielstand mit Kommazahlen ist kein UTF-8 — bisher ließ er sich über UDP
  weder senden noch unverfälscht empfangen.
* **`NET_UDP_LAST_HOST$`** und **`NET_UDP_LAST_PORT`** nennen den Absender
  getrennt. `NET_UDP_LAST_FROM` liefert `"::1:5000"`, und am Doppelpunkt
  zu trennen ginge bei IPv6 schief.
* **Die letzte Nachricht vor `NET_CLOSE` kommt an.** Hatte die Gegenseite
  noch etwas geschickt, das niemand gelesen hatte, brach das Schließen die
  Verbindung hart ab, und die Gegenseite verwarf, was ihr gerade noch
  geschickt worden war — eine Absage „BESETZT“ kam in 5 von 5 Versuchen
  nicht an. `NET_CLOSE` und das Programmende schließen jetzt nachlaufend:
  das eigene Ende geht sofort hinaus, gelesen wird im Hintergrund weiter,
  bis die Gegenseite auch schließt (höchstens 2 s, ohne das Programm
  anzuhalten).
* **UDP an eine Adresse ist schneller:** bisher startete jedes Paket einen
  Faden für die Namensauflösung, auch an `127.0.0.1`. 2000 Pakete brauchen
  jetzt 17 statt 231 ms.

### Ein neues Beispiel

* **`examples/duell`**: ein Weltraum-Duell für zwei über das Netz. Der Gast
  verbindet sich über TCP, das Spiel läuft über UDP: der Gast schickt
  30-mal je Sekunde seine Eingabe, der Wirt rechnet die Welt und schickt den
  Stand als Binärpaket zurück. Der Gast rechnet sein eigenes Schiff voraus
  und gleicht es mit jedem Stand ab. **`relay.dh`** spielt ein schlechtes
  Netz nach (Verzögerung, Schwankung, Verlust) — alles lässt sich auf
  einem Rechner ausprobieren. Geprüft ist es zwischen zwei Prozessen, über
  das Relay mit 10 % Verlust und zwischen Windows und Linux (WSL); nicht
  geprüft sind zwei echte Rechner hinter Routern.

### Prüfungen

* **`dhrt test` hat eine Zeitgrenze je Fall** (Vorgabe 300 s). Ein
  hängender Fall wird samt den Programmen, die er gestartet hat, beendet
  und als FEHL mit Namen und den letzten Zeilen gemeldet — bisher hielt er
  den ganzen Lauf an, ohne Meldung. `--- zeitgrenze N` gilt für eine
  Sammlung oder einen Fall, `dhrt test --zeitgrenze N` ändert die Vorgabe
  (0 = aus). Gemessen: kein Fall des Repos braucht über 60 s.

### Kleineres

* Der Sequencer (`examples/sequencer`) klippte im Mitschnitt ab und zu: die
  Spitze schwankte zwischen 0,88 und 1,01, weil das Echo in Millisekunden
  läuft und die Noten auf Uhr-Ticks liegen. Der SFX-Bus läuft jetzt auf
  32 statt 40 Prozent.

## Unter der Haube

**Zahlen.** **2133** Befehle in 48 Modulen (vorher 2130), 225 Beispiele auf
der obersten Ebene von `examples/` (das neue liegt in einem eigenen
Ordner). **5035 Fälle in 394 Prüfsammlungen** (vorher 5019 in 392) und
**550 Rust-Testfunktionen**.

* Das Duell prüft sich selbst: Wirt und Gast laufen als zwei Prozesse ohne
  Fenster (`--bot N --ohne-fenster`), und am Ende muss das Schiff des Gastes
  bei beiden an derselben Stelle stehen — auch über das Relay.
* Ein voller Prüflauf hing einmal 73 Minuten in einem IDE-Fall, vermutlich
  weil sich der Bildschirm abgeschaltet hatte. Die neue Zeitgrenze beendet
  so etwas nach fünf Minuten.

## Was offen bleibt

* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Ob ein Fenster stehen bleibt, solange der Bildschirm aus ist, ist nicht
  geklärt.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
* `ZEIT_PARSE` rechnet die Ortszeit nur unter Windows um; unter macOS und
  Linux gilt UTC.
