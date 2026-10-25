# Drachenhauch 2026.36

*Die Notizen zu dieser Fassung. Im Mittelpunkt steht **die IDE, die den Code
versteht**: Hover und Vervollständigung kennen die Klasse vor dem Punkt und
die Namensräume, Plugins reden im Code-Feld mit (Unterstreichen,
Schnellkorrektur, Tooltip, Kontextmenü), und ein neues Werkzeug zeigt die
Klassen eines Projekts als Diagramm. Dazu ein CHIP-8-Emulator als Beispiel.
**Für bestehende Programme ändern sich nur Kleinigkeiten**, sie stehen gleich
zuerst.*

## Was sich für bestehende Programme ändert

* **`GUI_CHANGED` meldet bei einem Baum die Auswahl eines Knotens.** Bisher
  blieb es dort stumm, obwohl die Doku „Auswahl“ nannte; ein Programm, das
  `GUI_CHANGED` auf einem Baum abfragt, bekommt jetzt TRUE, wenn ein Knoten
  angeklickt wurde.
* **Befunde von Plugins** sind jetzt im Code unterstrichen und stehen am
  Zeilenende, wie die Meldungen der Laufzeit. Vorher standen sie nur in der
  Liste der Probleme.
* **`dhrt test` lehnt Text hinter `--- programm` ab** (und hinter den anderen
  Köpfen ohne Inhalt wie `--- verzeichnis` oder `--- zeitgrenze`). Solcher
  Text wurde bisher ohne Meldung übergangen; der Quelltext eines Falls, der
  versehentlich hinter `--- programm` stand, lief also nie. Leerzeilen und
  Kommentare bleiben erlaubt. Keine der 444 Sammlungen im Repository war
  betroffen.

## Neu in dieser Fassung

### Die IDE kennt die Klasse vor dem Punkt

* **Hover und Vervollständigung hinter einem Punkt** zeigen die Felder und
  Methoden der Klasse des Objekts, auch über Vererbung, Feldelemente
  (`garne[0].`) und Rückgaben eigener Funktionen hinweg. Zu `spieler.hp`
  sagt der Hover jetzt, was `hp` ist.
* **Namensräume:** Nach `kern.` bietet die Vervollständigung die Namen des
  Namensraums an, der Hover zeigt ihre Kopfzeile -- ohne zu übersetzen, also
  auch bei einem Fehler anderswo im Text.
* **Parameternamen an Methodenaufrufen** stehen als Hinweis im Code, wie
  bisher schon an Funktionen.
* Dafür neu in der Laufzeit: `CODE_HOVER$` und `CODE_COMPLETE` nehmen als
  viertes Argument einen Ordner (dann lösen sie Mitglieder auf),
  `CODE_METHOD_CALLS$(text$ [, basis$])` liefert alle Methodenaufrufe samt
  Kopfzeile in einem Lauf.
* **Schwebende Leisten liegen über angedockten Fenstern.** Eine schwebende
  Leiste aus Widgets lag bisher unter einer angedockten Leiste, die ein ganzes
  Fenster ist.

### Plugins reden im Code-Feld mit

* **Befunde mit Stelle und Schnellkorrektur:** `PLUGIN_BEFUND` nimmt Spalte
  und Länge, `PLUGIN_KORREKTUR(titel$, text$)` hängt eine Korrektur an, die
  Strg+. anbietet.
* **Kontextmenü:** `PLUGIN_KONTEXT(id$, titel$)` legt einen Eintrag ins
  Rechtsklick-Menü des Code-Felds; das Plugin bekommt das angeklickte Wort.
* **Tooltip:** Nach `PLUGIN_HOERE("hover")` erfährt das Plugin Wort und
  Stelle unter der Maus und trägt mit `PLUGIN_HOVER_ANTWORT(text$)` zum
  Tooltip bei. Die IDE wartet nicht darauf.
* **Bäume in Leisten:** `PLUGIN_BAUM` füllt einen Baum der eigenen Leiste,
  dazu `PLUGIN_WORT$()` für das Wort an der Schreibmarke.
* Zwei neue Beispiele: `examples/plugins/klassen/` (alle Klassen des
  Projekts als Baum, ein Klick springt hin) und
  `examples/plugins/schreibweise/` (Schreibfehler in Kommentaren,
  korrigierbar mit Strg+.).
* Dafür neu: `CODE_CLASSES$(text$)` -- die Klassen und Structs eines Textes
  mit Vorfahrin, Feldern, Konstanten, Eigenschaften und Methoden samt Typ,
  ohne zu übersetzen.

### Werkzeug: Klassendiagramm

`examples/210_klassendiagramm.dh`, in der IDE unter **Werkzeuge →
Klassendiagramm**: alle Klassen des Projekts als Kästen, Vererbung als Pfeil
mit offenem Dreieck, ein Feld vom Typ einer anderen Klasse gestrichelt.
Kästen lassen sich ziehen (die Lage wird gemerkt), P sichert das Diagramm als
PNG, ein Doppelklick springt in der IDE an die Stelle. Dafür liest die IDE
jetzt die Ausgabe ihrer Werkzeuge: eine Zeile `DH_IDE_OEFFNEN|pfad|zeile`
öffnet die Datei dort. Handbuch: `docs/klassendiagramm.md`.

### Beispiel: CHIP-8

`examples/chip8/`: ein Emulator für die kleine virtuelle Maschine von 1977
(alle 35 Befehle, die Eigenheiten der Varianten als Schalter), ein Assembler
und ein eigenes Spiel. Start mit
`dhrt run examples/chip8/main.dh -- spiel.ch8`.

### Weitere Änderungen

* Der Hover zeigte bei einer Kopfzeile über mehrere Zeilen den
  Fortsetzungsstrich ` _` mitten in der Signatur.
* `PLUGIN_OEFFNEN` ohne Pfad springt im Reiter vorn (für einen ungesicherten
  Text).

## Unter der Haube

**Zahlen.** 2183 Befehle in 48 Modulen (neu: `CODE_CLASSES$`,
`CODE_METHOD_CALLS$`), 226 Beispiele auf der obersten Ebene von `examples/`,
**5473 Fälle in 444 Prüfsammlungen** (vorher 5445 in 440) und 569
Rust-Testfunktionen.

* Zwei Testfälle lasen die Ausgabe eines beendeten Kindprozesses nur einmal
  und verloren auf langsamen Läufern die letzte Zeile; sie warten jetzt wie
  die übrigen auf das Ende der Ausgabe.
* Zwei Bildfälle des CHIP-8-Beispiels überspringen sich ohne Fenster, statt
  zu scheitern.

## Was offen bleibt

* Ein Plugin-Beitrag zum Tooltip, der erst ankommt, wenn die Maus
  weitergezogen ist, wird verworfen -- geprüft ist das nicht.
* **raylib 6.2** ist erschienen, die Rust-Bindung raylib-rs hat aber noch
  keine Fassung dafür. Drachenhauch bleibt bei raylib 6.0, bis raylib-rs
  nachzieht.
* Funkenflug, Glutfaden und der Fotobetrachter sind nur unter Windows
  ausprobiert (mit dem GTK von Inkscape).
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
