# Drachenhauch 2026.44

*Die Notizen zu dieser Fassung. Sie ist die Fassung **für den Feinschliff**:
der Projektbaum der IDE mit Farben und Sinnbildern je Dateiart, Verläufe ohne
Stufen, die Bastelzentrale als Leitstand für Bastler, eine Spielwiese auf der
Website, mit der man Drachenhauch im Browser ausprobiert, und eine neue
Warnung, die schon einen echten Fehler gefunden hat. Was bestehende Programme
bemerken können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Farbverläufe sehen anders aus – ohne Stufen.** Abgerundete Flächen der gui
  und `GRADIENTV`/`GRADIENTH` (auch in den Farbwählern) runden jede Zeile gegen
  eine eigene Schwelle, statt alle drei Kanäle im selben Moment auf die nächste
  Stufe zu springen. Die sichtbaren Bänder (bis zu 32 Zeilen breit) sind weg,
  die Enden sind genau die angegebenen Farben. Wer ein Bild Punkt für Punkt
  vergleicht, sieht dazwischen leicht andere Werte.
* **Auf großen gui-Flächen ist die Fase leiser** (mit `verlauf_hoehe`): eine
  helle Kante über einer Leiste von 200 Punkten wirkte wie ein Rahmen aus
  Windows 95. Ohne die Metrik bleibt alles, wie es war.
* **`--check` warnt bei Parametern und Klassenfeldern, die eine Konstante
  verdecken** – aber nur, wenn der Name im Block auch anders geschrieben
  vorkommt (`n` neben `N`). Bisher gab es die Warnung nur für lokale
  Variablen; `FUNCTION summe(n AS INTEGER) ... RETURN n + N` rechnete still
  `n + n`. Wer die Warnung sieht, hat sehr wahrscheinlich genau diesen Fehler.

## Neu in dieser Fassung

### In der IDE

* **Der Projektbaum zeigt die Dateiart:** ein Sinnbild je Art (Ordner offen und
  zu, Quelltext, Prüfsammlung, Formular, Daten, Text, Bild, Klang), in einer
  eigenen Farbe je Art, und die Dateien eines Ordners stehen nach Art geordnet –
  erst die Programme, dann Prüfungen, Daten, Texte. Die Farben lassen sich in
  `farben.json` anpassen.
* **Ruhigere Flächen:** die Bereichsfarben um den Code sind zurückgenommen,
  der Schriftzug hinter dem Code ebenso, die Probleme-Leiste nennt sich nicht
  mehr doppelt.
* **Verdeckte Konstanten:** zur neuen Warnung gibt es eine Schnellkorrektur
  (Strg+.), die die Variable umbenennt (`nLokal`, `vollFeld`) – nur die
  Stellen in ihrer Schreibweise, ein `N` meint danach wieder die Konstante.
  Dasselbe bekommt VS Code über den Sprachserver.
* **Bastelzentrale** unter Werkzeuge, in den Beispielen nach Themen und als
  erste Kachel der Galerie; die Galerie zeigt jetzt 24 Programme, darunter
  Webcam und MIDI-Datei mit eingebauten Testquellen.

### gui

* **`GUI_FILETREE_COLOR(baum, muster$, farbe)`** und
  **`GUI_FILETREE_ICON(baum, muster$, sinnbild$)`**: Farbe und Sinnbild je
  Dateiart im Dateibaum (Muster wie beim Filter, `"/"` meint Ordner, das
  erste passende gilt), **`GUI_FILETREE_SET(baum, "nach_art", 1)`** ordnet die
  Dateien eines Ordners nach diesen Mustern. Alles wird in der `.dhform`
  gesichert.
* **`GUI_TREE_ICON`** nimmt auch den Namen eines eingebauten Sinnbilds.
* **Neun neue Sinnbilder:** `ordner`, `ordner_offen`, `datei_code`,
  `datei_test`, `datei_bild`, `datei_daten`, `datei_text`, `datei_ton`,
  `datei_form`.

### Website

* **Die Spielwiese:** Drachenhauch läuft als WebAssembly direkt auf der
  Website – Code eintippen, Ausführen, Bild sehen, ohne etwas zu installieren;
  deutsch und englisch, mit einer Galerie fertiger Beispiele und einem kleinen
  Spiel als Demo. Ein Link teilt das eigene Programm.

### Neue Beispiele

* **`bastelzentrale/`**: ein Leitstand für Bastler mit einer Handy-Seite –
  serieller Monitor mit Kurve, Arduino über Firmata, MQTT, Modbus, Mikrofon und
  MIDI-Uhr, Kamera mit Bewegungsmelder, Raspberry-Pi-Pins; das Telefon steuert
  über WebSocket, OSC-Apps über UDP, gefunden wird alles über mDNS. Jeder
  Bereich hat eine Demo-Quelle, damit es auch ohne Geräte etwas zu sehen gibt.
* **Duell:** die Nachladezeit wirkte nie – das Feld `nachladen` verdeckte die
  Konstante `NACHLADEN`, ein Schiff konnte in jedem Bild schießen. Gefunden von
  der neuen Warnung.

## Unter der Haube

**Zahlen.** 2426 Befehle (vorher 2424) in 57 Modulen, 234 Beispiele auf der
obersten Ebene von `examples/`, **5949 Fälle in 497 Prüfsammlungen** (vorher
5930 in 494) und 669 Rust-Testfunktionen (663).

* **Der Web-Bau übersetzte seit 2026.43 nicht mehr** (das Webcam-Modul
  brauchte eine Bibliothek, die nur für den Desktop eingetragen war), und
  niemand sah es. Jetzt baut der Website-Lauf `dhrt.wasm` bei jeder Änderung,
  auch für jeden Pull Request, der den Web-Teil anfasst.
* **Die Prüfsammlungen stehen nach Themen in Unterordnern** von `tests/pruef/`
  (Sprache, Laufzeit, gui, IDE, Studio, Module, Geräte …), Übersicht in
  `tests/pruef/LIESMICH.md`.
* **Die IDE ist in Teile zerlegt:** `ide/ide.dh` hält Zustand, Fenster und
  Hauptschleife, die Unterprogramme stehen nach Themen in `ide/teile/` (13
  Dateien) und werden an ihrer alten Stelle eingebunden – das Programm ist
  Zeile für Zeile dasselbe. Dafür kopiert `dhrt test` bei `--- programm` jetzt
  auch eingebundene Dateien mit.
* **Ein wackliger Fall weniger:** das Warten auf Eingaben wurde im vollen Lauf
  von den Fenstern der anderen Fälle geweckt; es wird jetzt allein geprüft.
* Die Entwicklungsgeschichte steht jetzt in `geschichte/`.

## Was offen bleibt

* **Echte Geräte sind weitgehend ungeprüft** (Raspberry Pi, SPI, Kamera,
  Mikrofon, Arduino, Modbus) – Rückmeldungen mit Hardware sind willkommen.
* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
