# Drachenhauch 2026.14

*Die Notizen zu dieser Fassung. Fünfzehn Tage und 175 Commits nach 2026.13,
und fast alle gehören zu einem Vorhaben: **Python fällt weg.** Die
Entwicklungsumgebung, alle Werkzeuge, alle Prüfungen und der Installer sind
jetzt in Drachenhauch oder in der Laufzeit selbst geschrieben. Übrig an
Python sind zwei Bauskripte, die nur die Standardbibliothek brauchen — wer den
Installer nimmt oder `dhrt test` laufen lässt, braucht gar keins.*

## Neu in dieser Fassung

### Die IDE ist ein Drachenhauch-Programm

`ide/ide.dh` ersetzt den Qt-Editor. Sie ist in dieser Fassung von einem
ersten Stand mit Reitern und Fehlerliste zu einer vollständigen Umgebung
gewachsen — gemessen nicht an einer Wunschliste, sondern an dem, was die
Qt-IDE konnte, und danach an dem, was beim Schreiben fehlt:

* **Schreiben:** Faltung, mehrere Schreibmarken und Spaltenauswahl,
  Einrückung und mitwachsendes Gerüst (`IF x THEN` + Enter setzt das
  `END IF` gleich mit), Schnipsel per Kürzel, Vorschläge beim Tippen mit
  Signatur-Platzhaltern, Farbfelder an `&H`-Literalen, Einrückungslinien,
  eine Suchleiste mit Groß/klein, ganzem Wort, regulärem Ausdruck und
  einzelnem Ersetzen.
* **Verstehen:** Hilfe zum Wort als Tooltip beim Überfahren, Strg+Klick zur
  Definition — auch über Dateigrenzen —, „Wer ruft das auf?“ als Baum, ein
  Symbolverzeichnis über das ganze Projekt, eine Pfadleiste über dem Code.
* **Umbauen, mit Vorschau:** im ganzen Projekt umbenennen (mit Einwand, wenn
  der Name schon vergeben ist oder eine Oberklasse ihn trägt), Parameter
  umsortieren, hinzufügen und entfernen samt allen Aufrufen, Unterprogramme
  und Klassen in eine andere Datei verschieben (der nötige `IMPORT` kommt
  mit), eine Auswahl in ein Unterprogramm herauslösen. Jeder Umbau zeigt
  vorher den Unterschied, einzelne Blöcke lassen sich auslassen, und ein
  Strg+Umschalt+Z nimmt ihn über alle Dateien zurück.
* **Laufen lassen:** Debugger mit Haltepunkten per Klick in die
  Nummernspalte (rechts: mit Bedingung), Ausdrücke im angehaltenen Programm,
  Profil mit Durchläufen je Zeile, ein Doppelklick auf `datei.dh:zeile` in
  der Ausgabe springt hin.
* **Drumherum:** git diff, log und blame im Fenster, das Handbuch gesetzt
  statt roh (mit Suche), eine Willkommensseite mit Beispiel-Galerie, Zoom mit
  Strg+Rad, Absturz-Wiederherstellung, Export einer eigenständigen `.exe`,
  Drucken und PDF.

Das Kreuz des Fensters fragt nach, wenn etwas ungesichert ist — vorher
beendete es die IDE wortlos, und ESC tat dasselbe (siehe unten).

### Die Werkzeuge auch

Sprite-Editor, Tilemap-Editor, Tracker, SFX-Generator und Partikel-Editor
gab es schon als Drachenhauch-Fassungen; dazu kamen **Form-Designer**,
**Anim-FSM-Editor** und **Notenblatt**. Alle stehen in der IDE unter
*Werkzeuge*, alle fragen beim Schließen nach, wenn etwas ungesichert ist, und
jedes hat ein neues Handbuch, das am Ende sagt, was die Qt-Fassung zusätzlich
konnte. Dabei gefunden und behoben: der Tracker verlor beim Sichern die
Sample-Instrumente einer Qt-Datei, der GB-Code des SFX-Generators ließ den
Pan weg, das Notenblatt verschwieg, was beim Umrechnen in den Tracker verloren
geht, und der Sprite-Editor kann jetzt 64 Bilder, 256 Punkte und 8 Ebenen
statt 16, 128 und 4.

### Neue Oberflächen-Bausteine

Was die IDE brauchte, gibt es jetzt für jedes Programm: einen **Dateibaum**,
**Reiter innerhalb eines Fensters**, **gesetzten Text** aus Markdown mit
Auswahl und Suche, einen **Uhrzeitwähler**, eine **Werkzeugleiste** mit 35
eingebauten Sinnbildern und Überlauf-Menü, eine **Statusleiste**, eine
**Pfadleiste**, ein **Gitter** (Tabelle im Zellmodus mit Tastatur,
Bereichen, Zwischenablage, Zahlen- und Auswahlspalten), **Knopfarten**
(primär, Gefahr, Umriss …) und **Listen** mit Gruppen, Filter, Zusatztext und
Tippen-zum-Springen. Der Textbereich kann falten, mehrere Marken führen, eine
Spalte auswählen, beim Umbruch einrücken und mit dem Mausrad rollen — bis
dahin rollte er gar nicht.

### Die Laufzeit

* Das **pdf-Modul** schreibt über krilla: eingebettete Schriften, Unicode,
  eigene Schriften per `PDF_FONT_LOAD`.
* **ESC beendete jedes Programm mit gui** — raylibs Beenden-Taste. Jetzt
  gehört sie der Oberfläche, und `WINDOW_CLOSE_REQUESTED()` meldet das Kreuz,
  ohne das Programm zu beenden.
* **`CLIPBOARD_GET` stürzte ab**, wenn ein anderes Programm die
  Zwischenablage hielt, und `CLIPBOARD_SET` scheiterte dann still. Beides
  behoben: lesen liefert leer, setzen wartet kurz und meldet sonst einen
  Fehler.
* **Kein Fenster** (kein Bildschirm, kein OpenGL) ist ein abfangbarer Fehler
  mit Erklärung statt eines Absturzes.
* Neue Befehle, unter anderem: acht Zeichenbefehle für Bilder
  (`IMAGE_FILL_POLY`, `IMAGE_RING`, `IMAGE_GRADIENT` …), `BUFFER_DEFLATE`/
  `INFLATE`, Bytes über das Netz (`NET_RECV_BYTES`), `HTTPD_SET_HEADER`,
  `REGEX_FIND_POS`/`REGEX_ESCAPE$`, `JSON_GET_JSON`, `EXEPATH$`, `VERSION$`,
  `IMAGE_SAVE` ohne Alphakanal; dazu `dhrt bild` (ein Bild von einem
  Programm) und `dhrt run --bilder N`.

### Installer ohne Python — und Pakete für macOS und Linux

Der Windows-Installer enthält keine Python-Laufzeit und kein Qt mehr: **43 MB
statt 92**. Er bringt die IDE, alle Werkzeuge, das Handbuch, die
Beispiele, beide Bücher und die ESP32-Sketche mit und räumt eine alte
GameBasic-Installation auf. Neu sind ein `.dmg` für macOS und ein `.tar.gz`
mit `install.sh` für Linux; eine `.dh` per Doppelklick im Finder wird an die
IDE übergeben (belegt bis zur Laufzeit, auf einem echten Mac ungeprüft).

**Die Lizenzhinweise waren unvollständig.** Die Beilage mit den Lizenztexten
der Rust-Bibliotheken wurde für einen Bau ohne Hardware-Module erzeugt,
ausgeliefert wurde aber mit ihnen — die Texte von rund hundert Bibliotheken
(serielle Schnittstelle, USB, Bluetooth, MIDI, Dateidialoge …) fehlten, in
allen Fassungen bisher. Jetzt deckt die Beilage jeden möglichen Bau ab: 475
statt 370 Bibliotheken.

**Wer 2026.13 installiert hat:** der neue Installer hat eine eigene Kennung
und einen eigenen Ordner (`Drachenhauch-IDE`). Die alte Fassung bleibt
stehen, bis man sie unter *Einstellungen → Apps* entfernt; die Beispiele in
den öffentlichen Dokumenten teilen sich beide.

## Unter der Haube

**Die Prüfungen sind umgezogen.** Bis zu dieser Fassung liefen rund 4400
pytest-Tests; jetzt sind es **3602 Fälle in 257 Prüfsammlungen**
(`tests/pruef/*.dhtest`), die `dhrt test` selbst ausführt, dazu **404
Rust-Testfunktionen** (vorher 321, beide Male im Quelltext gezählt). Nicht umgezogen ist, was Python-Code prüfte, den es
nicht mehr gibt. Unterwegs wuchs das Format um alles, was die Werkzeuge
brauchten: ein vorhandenes Programm mit eingeschobener Aufnahme starten,
Dateien danach lesen, Bilder und WAVs prüfen, mehrere Läufe, Beilagen in
Base64. Die CI fährt kein pytest mehr.

**Zwei Ursachen für wacklige Klick-Tests gefunden.** Die Wiedergabe von
Eingabe-Aufnahmen verlor die Mausposition, sobald ein Fenster unter dem
Zeiger auftauchte, und die geteilte Zwischenablage ließ Fälle den Text fremder
Fälle einfügen. Beides ist in der Laufzeit behoben, nicht im Test umgangen.

**Zahlen.** Die Befehlsreferenz von 1715 auf **1911** Einträge, die
Modulliste bleibt bei 47, die Beispiele von 212 auf **215**.

## Was offen bleibt

* Auf einem echten Mac ist das Paket nicht ausprobiert (die Läufer der CI
  haben kein OpenGL), und es ist nicht beglaubigt — dafür fehlen ein
  Apple-Konto und ein Mac. Beim ersten Start hilft *Rechtsklick → Öffnen*
  bzw. ab macOS 15 *Datenschutz & Sicherheit → Dennoch öffnen*.
* Die Werkzeuge können einiges nicht, was die Qt-Fassungen konnten; jedes
  Handbuch listet es am Ende. Am deutlichsten: der Tracker spielt Sample-,
  Keymap- und SoundFont-Instrumente nicht ab (er sichert sie verlustfrei), der
  Tilemap-Editor kennt nur 16er-Kacheln, der Form-Designer keine
  Mehrfachauswahl.
* Der Installer ist nicht signiert; SmartScreen meldet sich beim ersten Start.
