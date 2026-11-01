# Drachenhauch 2026.40

*Die Notizen zu dieser Fassung. Sie bringt **das Drachenhauch Studio**: eine
eigene Entwicklungsumgebung für Programme mit Oberfläche im Stil von Xojo.
Man zeichnet Fenster, setzt Eigenschaften im Inspektor und schreibt den Code
je Ereignis; ein Fenster ist eine Klasse, das Projekt ein Ordner. Die
Laufzeit bekam dafür ein paar Bausteine, und eine Meldung des Übersetzers
sagt jetzt, was wirklich fehlt. **Zwei Kleinigkeiten ändern sich für
bestehende Programme**; sie stehen gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Ein fehlendes Blockende heißt jetzt so, wie es ist.** Kommt ein
  `END SUB`, während ein `IF` noch offen ist, meldete der Übersetzer
  „Erwartet IF nach END“ – das las sich, als fehle ein `IF`. Jetzt heißt es
  „END IF fehlt -- hier steht END SUB, aber das IF davor ist noch offen“
  (bzw. „Erwartet END IF“), für alle Blöcke von `IF` bis `STRUCT`. Wer eine
  solche Meldung als Text auswertet, passt den Vergleich an; die
  Schnellkorrektur der IDE kennt beide Formen.
* **`dhrt --export` legt keine eingebundenen Quelltexte mehr neben die
  Exe.** Eine per `IMPORT "datei.dh"` eingebundene Datei galt bisher als
  „verwendete Datei“ und wurde mitkopiert – der Quelltext lag dann offen im
  Bau. Er steckt ohnehin im Bytecode; Bilder, Klänge und andere Dateien
  kommen weiter mit.

## Neu in dieser Fassung

### Drachenhauch Studio

Gestartet aus dem Startmenü („Drachenhauch Studio“), aus der IDE (Werkzeuge
→ Drachenhauch Studio), per Doppelklick auf eine `.dhstudio`, unter macOS
und Linux mit `drachenhauch --studio [projekt]` oder direkt mit
`dhrt run studio/studio.dh`. Handbuch: [studio.md](studio.md).

* **Fenster entwerfen:** eine Bibliothek mit 34 Bausteinen in acht Gruppen
  (mit Vorschaubild und Suche), Ablegen per Klick oder Zug,
  Ausrichtungslinien an den Kanten und Mitten der Nachbarn, acht Griffe,
  Rahmenwahl, Anordnen, Zoom, Rückgängig. Der **Inspektor** zeigt die
  Eigenschaften in Gruppen, mit Farbwahl und den Ankern als vier
  Kästchen.
* **Projekt und Code:** App, Fenster, Module und Klassen im Navigator, ein
  Code-Reiter je Methode. Ein Doppelklick auf ein Control legt die Methode
  seines Ereignisses an (`knopfSichern_Klick`, `tabelle_Doppelklick`,
  `feldName_Geaendert` …); wer ein Control umbenennt, nimmt seine Methoden
  mit. Gespeichert wird als Ordner mit `projekt.dhstudio`, je Element einer
  `.dh` und je Fenster einer `.dhform` – lesbar und für git geeignet.
* **Starten, Bauen, Debuggen:** F5 prüft das Projekt, erzeugt das Programm
  in `_studio/` und startet es; Fehler des Übersetzers und Laufzeitfehler
  stehen an der Methode und Zeile, in der man sie geschrieben hat, nicht
  im erzeugten Code. Strg+B baut eine `.exe`, F7 debuggt mit Haltepunkten
  in der Nummernspalte, Variablen und Aufrufstapel.
* **Was ein Programm braucht:** Menü-Editor mit Kürzeln, Timer als
  unsichtbare Controls, Bilder im Projekt, `Meldung`, `Frage`, `Eingabe` und
  modale Fenster, die auf eine Antwort warten (`ZeigenModal`), Ereignisse
  mit Parametern (`Ziehen(x, y)`, `Rad(schritte)`, `Taste(taste)`) und eine
  Vervollständigung im Code-Feld, die die Felder der Fensterklasse kennt.
* **Beispielprojekt** `examples/studio/rechnungen/`: eine kleine
  Rechnungsverwaltung mit SQLite, Datenbindung, Prüfregeln, PDF und CSV –
  vier Fenster, eine Klasse, zwei Module.
* Der Form-Designer öffnet sein Formular mit Datei → Im Drachenhauch Studio
  öffnen (Strg+Umschalt+O) dort; er bleibt für einzelne Formulare.

### gui

* **Schließbare Reiter im Reiterwerk** (`GUI_TABCONTROL_SET(tc,
  "schliessbar", 1)`, `GUI_TABCONTROL_CLOSED`): ein Kreuz je Reiter, die
  mittlere Maustaste schließt ebenfalls; geschlossen wird nichts von selbst.
* **`GUI_LISTBOX_ITEM_RECT`** und **`GUI_TREE_ITEM_RECT`**: die Lage einer
  Zeile auf dem Bildschirm, für Prüfungen und zum Ablegen aus einer Liste.
* **`GUI_CHANGED` auf einem Kippschalter** meldete jedes Bild seines
  Nachgleitens als Änderung; jetzt zählt nur das Umschalten.
* **Die gewählte Zeile eines Baums** war im hellen Thema kaum lesbar
  (dunkle Schrift auf der Auswahl); die Schrift folgt jetzt ihrem Grund wie
  bei der Liste.

### Prüfen

* **`dhrt --check` warnt, wenn eine Methode eine geerbte gleichen Namens
  mit einer anderen Zahl von Parametern überschreibt.** Namen kennen keine
  Groß-/Kleinschreibung: eine Methode `zeigen(id)` in einem Fenster verdeckt
  das geerbte `Zeigen()`, und erst der Aufruf brach ab. Der Konstruktor
  `Init` ist ausgenommen. Gefunden am Beispielprojekt; im übrigen Bestand
  meldet die Warnung nichts.

## Unter der Haube

**Zahlen.** 2251 Befehle in 48 Modulen (4 neu: `GUI_TABCONTROL_SET`,
`GUI_TABCONTROL_CLOSED`, `GUI_LISTBOX_ITEM_RECT`, `GUI_TREE_ITEM_RECT`), 226
Beispiele auf der obersten Ebene von `examples/` (dazu das Studio-Projekt
unter `examples/studio/`), **5751 Fälle in 468 Prüfsammlungen** (vorher 5704
in 461) und 608 Rust-Testfunktionen.

* Installer und Pakete nehmen das Studio mit (Verknüpfung, Dateityp
  `.dhstudio` unter Windows, Linux und macOS) und **nur noch Beispiele, die
  git kennt**: aus einem Arbeitsordner mit heruntergeladenen Klängen und
  Modellen kamen 2026.39 zuerst 21 MB zu viel mit.
* Die Zählung im Befehlsverzeichnis stand auf 2248, es sind 2251.

## Was offen bleibt

* **Das Studio** hat noch keine englische Oberfläche, keine
  Layout-Behälter im Entwurf und keine Projektvorlagen.
* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das Studio, Funkenflug, Glutfaden und der Fotobetrachter sind nur unter
  Windows ausprobiert.
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
