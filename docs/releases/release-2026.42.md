# Drachenhauch 2026.42

*Die Notizen zu dieser Fassung. Sie macht **das Drachenhauch Studio** zu einem
Werkzeug für Anwendungen mit Daten: Datenbanken im Projekt, Fenster und Listen
an Tabellen gebunden, Berichte für Rechnungen und Listen, eigene Controls und
eine englische Oberfläche. Dazu zwei Korrekturen in der Laufzeit, die
bestehende Programme bemerken können; sie stehen gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Ein Reiterwerk (`GUI_TABCONTROL`) gibt Klicks an die Widgets seiner
  Seiten weiter.** Bisher schluckte es jeden Klick auf ein Kind: Knöpfe auf
  einer Seite meldeten nie `GUI_CLICKED`, Felder bekamen per Klick keinen
  Fokus. Wer sich mit Tastatur oder `GUI_FOCUS` darum herumgeholfen hat,
  bekommt die Klicks jetzt zusätzlich.
* **`PDF_PREVIEW` setzt Text in der Schrift des PDFs** – den eingebauten
  DejaVu-Schnitten oder einer mit `PDF_FONT_LOAD` geladenen Schrift – und
  Zeichen für Zeichen an die Stelle, die auch das PDF rechnet. Vorher stand
  der Text in raylibs Pixelschrift, und ein rechtsbündiger Betrag endete ein
  Stück zu früh. Vorschaubilder sehen deshalb anders aus als bisher, dafür
  wie das Papier.

## Neu in dieser Fassung

### Drachenhauch Studio

* **Englische Oberfläche:** Menüs, Werkzeugleiste, Bibliothek, Inspektor,
  alle Dialoge, Meldungen und der Debugger; umschalten unter Ansicht →
  English interface (gilt beim nächsten Start). Die vier Projektvorlagen gibt
  es auch auf Englisch. Die Namen der Ereignisse (`Knopf1_Klick`) bleiben
  deutsch – sie sind Teil des Projektcodes, ein Projekt bleibt in beiden
  Sprachen dasselbe.
* **Eigene Controls** (wie Xojos ContainerControl): ein Projektelement mit
  eigenem Layout und Code, das man in jedes Fenster ablegt. Ein Control kann
  eigene Ereignisse (`Self.Ausloesen("Erreicht")`) und Eigenschaften haben,
  deren Werte man je Instanz im Inspektor setzt, und andere Controls
  enthalten. Umbenennen nimmt alle Instanzen mit, Löschen geht nur ohne
  Instanzen.
* **Datenbanken im Projekt:** Projekt → Datenbank hinzufügen. Ein Reiter
  zeigt die Tabellen mit einem Spalten-Gitter (Art, Pflicht, Eindeutig,
  Vorgabe, Verweis auf eine andere Tabelle) und einem Daten-Gitter; jede
  Änderung gilt sofort für die SQLite-Datei im Projektordner, in einer
  Transaktion. Das erzeugte Programm legt fehlende Tabellen und Spalten
  selbst an und benennt auch in einer älteren Datei um, was im Studio
  umbenannt wurde.
  * **Binden:** Eine Spalte oder ganze Tabelle ins Fenster gezogen legt
    Beschriftung und passendes, gebundenes Feld an; ein Verweis wird eine
    Klappliste. Ein Fenster mit Tabelle bekommt `Laden`, `Sichern`, `Neu`,
    `Loeschen` und `Ungesichert`.
  * **Listen an Tabellen:** Liste, Tabelle und Klappliste mit Datenquelle,
    Sortierung, Bedingung, Suchfeld und Spalten über Verweise
    (`kunde.name`, das Studio legt den Verbund selbst).
  * **Abfragen** mit Namen, im Reiter mit SQL und Ergebnis, als Datenquelle
    und im Code als `Abfrage("name")`.
  * **Bildspalten:** im Daten-Gitter ein Bild aus einer Datei setzen, im
    Fenster ein gebundenes Bild-Control, in Tabellen das Bild in der Zelle.
* **Berichte:** gedruckte Seiten aus den Daten des Projekts, etwa eine
  Rechnung oder eine Liste (Projekt → Bericht hinzufügen).
  * Bänder: Berichtskopf, Seitenkopf, Gruppenkopf, Detail, Gruppenfuß,
    Seitenfuß, Berichtsfuß; Gruppen über Wertwechsel, Summen je Gruppe und
    Bericht, Seitenumbruch und „Seite n von m“.
  * Die Quelle ist eine Tabelle oder Abfrage, die Bedingung darf Parameter
    haben (`rechnung = :id`).
  * Entworfen wird auf der Seite in Millimetern: Elemente aus einer Leiste
    ablegen, Spalten der Quelle hineinziehen, ziehen mit Raster und
    Ausrichtungslinien, Griffe, Bandhöhe, Eigenschaften, Rückgängig.
  * **Nur Zeilen mit:** ein Element gilt nur für Zeilen mit einem Wert
    (`mwst = 19`); so steht die Steuer je Satz in der Rechnung.
  * **Bildspalten:** ein Bild-Element zeigt je Zeile das Bild aus einer
    Spalte; Bilder werden eingepasst statt verzerrt.
  * Im Programm ist ein Bericht ein Objekt mit `Pdf`, `Drucken`, `Vorschau`
    und `Erzeugen`; im Studio zeigt die Vorschau ihn mit den Entwicklungsdaten.
* **Das Beispielprojekt Rechnungen** benutzt alles davon: ein
  Datenbank-Element mit der Abfrage `Belegzeilen`, und die Rechnung setzt der
  Bericht `Rechnungsbeleg` – Anschrift, Positionen, Netto, Steuer je Satz,
  Gesamt. Neu ist Datei → Vorschau.
* **Tempo des Designers** bei 300 Controls: ziehen 6,3 → 1,2 ms je Bild,
  ruhig 1,7 → 1,0 ms.

### Laufzeit

* **`DB_GET_BUFFER(r, idx)`** liest eine Spalte als Bytes – ein BLOB, wie er
  gespeichert ist. Bisher kam ein BLOB nur als Text heraus.

## Unter der Haube

**Zahlen.** 2252 Befehle in 48 Modulen (neu: `DB_GET_BUFFER`), 226 Beispiele
auf der obersten Ebene von `examples/` (dazu das Studio-Projekt unter
`examples/studio/` und je vier Vorlagen auf Deutsch und Englisch unter
`studio/vorlagen/`), **5830 Fälle in 482 Prüfsammlungen** (vorher 5771 in
473) und 608 Rust-Testfunktionen.

* **Abhängigkeiten:** `zip` 2.4 → 9.2 und `rustls` 0.23.46 schließen sieben
  Sicherheitswarnungen, darunter eine kritische (zwei widersprüchliche
  Kopfzeilen in einem ZIP). Beim Entpacken wird ein Eintrag mit unlesbarem
  Namen übersprungen wie ein unsicherer.
* Das Lehrbuch hat einen Eintrag für `DB_GET_BUFFER` (deutsch und englisch).
* Installer und Pakete nehmen `studio/sprache` und die englischen Vorlagen
  mit.

## Was offen bleibt

* **Die letzten Studio-Änderungen (Berichte) sind ohne die macOS-Prüfungen
  der CI zusammengeführt** – GitHub hatte seit dem 2. November oft keine
  macOS-Läufer. Linux und Windows sind geprüft.
* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das Studio, Funkenflug, Glutfaden und der Fotobetrachter sind nur unter
  Windows ausprobiert.
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
