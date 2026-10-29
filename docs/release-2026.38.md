# Drachenhauch 2026.38

*Die Notizen zu dieser Fassung. Im Mittelpunkt stehen **die Werkzeuge**:
SFX-Generator, Tracker, Notenblatt, Partikel-, Tilemap-, Sprite- und
Anim-FSM-Editor schreiben in einer echten Schrift, haben Menü, Werkzeugleiste
und Statusleiste und können deutlich mehr. Dazu kommen zwei Demos
(Nachtflug, Lichterglanz), eine Landkarte als Beispiel und eine offene
Lizenz. **Drei Kleinigkeiten ändern sich für bestehende Programme**; sie
stehen gleich zuerst.*

## Was sich für bestehende Programme ändert

* **`\` rundet nach unten wie `FLOOR`**: `-7 \ 2` ist jetzt -4 statt -3.
  `MOD` hat das Vorzeichen des Teilers, und mit dem Abschneiden war
  `(a \ b) * b + a MOD b` bei negativen Zahlen nicht `a`. Für positive Zahlen
  ändert sich nichts. Gilt in der VM, beim Konstantenfalten und im
  Maschinencode.
* **`MODEL_INSTANCED` mit einer Farbe je Instanz** zeichnet in EINEM Aufruf
  statt je Farbe einmal (2600 Würfel: 2380 statt 653 Bilder je Sekunde). Die
  Farbe reist dafür in der untersten Zeile der Matrix mit -- eine
  Projektionsmatrix als Instanz-Matrix geht damit nicht mehr.
* **`DRAWIMAGEPARTEX` mit einem elften Argument ist ein Fehler.** Bisher
  wurde es still übergangen; das zehnte ist jetzt eine Farbe (siehe unten).

## Neu in dieser Fassung

### Die Werkzeuge

Alle sieben Werkzeuge in Drachenhauch (SFX-Generator, Tracker, Notenblatt,
Partikel-, Tilemap-, Sprite- und Anim-FSM-Editor) schreiben in DejaVu Sans
statt in der Pixelschrift, haben Menü mit Kürzeln, Werkzeugleiste mit
Sinnbildern und Statusleiste, und ihr Fenster wächst mit.

* **SFX-Generator:** zwölf Vorlagen, **Zufall** einer Klangart und
  **Mutieren**, ein Verlauf mit Namen je Schritt, die Hüllkurve lässt sich in
  der Wellenform ziehen, ein Pegelmesser beim Abspielen.
* **Tracker:** Raster in Mono-Schrift, Übersicht über alle Patterns der
  Reihenfolge zum Anklicken, Pegel je Kanal, Patterns umbenennen.
* **Notenblatt:** Schlüssel, Köpfe, Pausen und Vorzeichen aus der Notenschrift
  **Noto Music** (in `dhrt` eingebaut), Taktart und Taktnummern, Zoom,
  Eingabe per Tastatur und Klaviatur, Notenwerte als Knopfleiste.
* **Partikel-Editor:** zwei Farbwähler statt sechs Reglern, Zufall und
  Mutieren, zwölf Vorlagen, wählbarer Hintergrund, ein Streifen „ein Teilchen
  im Lauf seines Lebens“. Die `.ini` schrieb beide Farben unter denselben
  Schlüsseln (die Endfarbe überschrieb die Anfangsfarbe) -- jetzt getrennt,
  alte Dateien gehen weiter auf.
* **Tilemap-Editor:** Linie und Ellipse, Stempel aus mehreren Kacheln der
  Palette, Übersichtskarte, Kästchen für die Sichtbarkeit der Ebenen,
  Verschieben mit der mittleren Maustaste.
* **Sprite-Editor:** Werkzeugspalte neben der Zeichenfläche, Bildleiste mit
  Vorschaubildern, zuletzt benutzte Farben, **Deckkraft je Ebene** (beim
  Zeichnen, in jeder Ausgabe und im `.dhsprite`).
* **Anim-FSM-Editor:** der Graph zoomt (Mausrad) und lässt sich verschieben,
  **Anordnen** nach Stufen ab dem Startzustand, **Einpassen**, im Inspektor
  läuft die Animation des gewählten Zustands.

### Schriften und gui

* **`GUI_FONT(font [, groesse])`**: die gui hat ihre eigene Schrift. Wer
  zwischendurch mit einer anderen zeichnet, verdirbt die Oberfläche nicht
  mehr.
* **Fehlende Zeichen aus der eigenen Datei:** ein Zeichen, das eine geladene
  Schrift noch nicht gebacken hat (das ▶ bei `LOADFONT("sans")`), kommt beim
  nächsten Bild aus derselben Schriftdatei.
* **`FONT_GLYPH_BOX`** und **`FONT_BASELINE`**: die Maße einer Glyphe aus der
  Schriftdatei -- ein Zeichen sitzt damit genau (ein Notenschlüssel auf
  seiner Linie, ein Symbol mittig in einem Knopf).
* **`LOADFONT("noten")`**: Noto Music ist eingebaut (SIL Open Font License).
* **`GUI_FOCUS(-1)`** gibt den Fokus frei.
* **18 neue Sinnbilder** für Mal- und Editorwerkzeuge (`stift`, `radierer`,
  `fuellen`, `linie`, `rechteck`, `ellipse`, `pipette`, `auswahl`, `lasso`,
  `zauberstab`, `verschieben`, `spruehen`, `wuerfel`, `raster`, `auge`,
  `auge_zu`, `spiegeln`, `drehen`).
* **`DRAWIMAGEPARTEX(..., farbe)`** tönt den Ausschnitt; das Alpha der Farbe
  ist die Deckkraft.

### Grafik und Ton

* **`POSTFX(shader, massstab)`** (0.25..1): der Shader rechnet in einem
  kleinen Ziel -- auf schwachen Tablets 1,7- bis 4-mal schneller.
* **`SAMPLE_FROM_SOUND(sound)`**: ein Klang wird ohne Zwischendatei zum
  Sample.
* **`AUDIO_NOTE`** ist 1,5- bis 2,6-mal schneller.
* Das Handbuch sagt jetzt, dass `vib_depth` ein Faktor der Frequenz ist, keine
  Halbtöne (0.003..0.006 ist ein natürliches Vibrato).

### Beispiele und Demos

* **Landkarte** (`examples/karten/`): OpenStreetMap-Kacheln im Hintergrund
  geladen, Ortssuche, GPX-Spur mit Höhenprofil, Ausfuhr als PNG, PDF und auf
  den Drucker.
* **Nachtflug**: eine Demo mit synthetisierter Trance in Stereo und
  Shader-Szenen -- vom Neon-Tunnel durch kosmische Nebel bis zum Rückweg zur
  Erde; läuft auch im Browser und auf Tablets.
* **Lichterglanz**: eine Weihnachtsdemo, ebenfalls im Browser.

### Lizenz und Website

* **Drachenhauch ist Open Source**, wahlweise unter **MIT oder Apache 2.0**.
  Mit Drachenhauch geschriebene Programme gehören ihren Autoren und dürfen
  samt `dhrt` weitergegeben und verkauft werden.
* Englisches README, eine Website mit dem Handbuch und `llms.txt`.

## Unter der Haube

**Zahlen.** 2188 Befehle in 48 Modulen (neu: `GUI_FONT`, `GUI_FONT_GET`,
`FONT_GLYPH_BOX`, `FONT_BASELINE`, `SAMPLE_FROM_SOUND`), 226 Beispiele auf der
obersten Ebene von `examples/`, **5580 Fälle in 453 Prüfsammlungen** (vorher
5484 in 446) und 570 Rust-Testfunktionen.

* Kira 0.12.6.
* `build_wasm.py` beachtet `CARGO_TARGET_DIR` und linkt neu, wenn sich das
  Datenpaket geändert hat.
* Die Prüfungen der Werkzeuge fahren echte Klicks auch durch Menü und
  Werkzeugleiste; Lagen im Hauptfenster rechnen die Höhe der Menüleiste mit.

## Was offen bleibt

* **Der Form-Designer** ist noch nicht aufgefrischt; er kommt zusammen mit
  einer Xojo-ähnlichen Art, Oberflächen zu programmieren.
* **raylib 6.2** ist erschienen, die Rust-Bindung raylib-rs hat aber noch
  keine Fassung dafür. Drachenhauch bleibt bei raylib 6.0.
* Funkenflug, Glutfaden und der Fotobetrachter sind nur unter Windows
  ausprobiert (mit dem GTK von Inkscape).
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
