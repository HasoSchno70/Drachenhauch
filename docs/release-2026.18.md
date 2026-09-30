# Drachenhauch 2026.18

*Die Notizen zu dieser Fassung. Zwei Tage und achtzehn Pull Requests nach
2026.17, fast alle für die IDE: sie schreibt jetzt mit (Typen, Parameter,
Fehlertexte hinter der Zeile), räumt mit auf (Dateien im Projektbaum,
Einrücken beim Einfügen) und kennt die Wege zurück (Zurück/Vorwärts, letzte
Änderung, Vergleich mit der gesicherten Fassung). Dazu eine englische
Oberfläche und eigene Farben.*

## Neu in dieser Fassung

### Beim Schreiben

* **Fehlertext am Zeilenende:** die Meldung der Prüfung steht rot bzw. gelb
  hinter ihrer Zeile, lesbar ohne die Maus hinzubewegen.
* **Typ-Hinweise:** hinter einer Zeile mit `FOR EACH` steht, was die
  Variable hält (`p: INTEGER`, `k: STRING, v: FLOAT`).
* **Parameternamen an Aufrufen:** hinter `kreis(100, 50, 20)` steht
  `kreis(x, y, r)`, auch für eingebaute Befehle.
* **Aufrufzähler** über jeder SUB/FUNCTION (wie oft sie im Projekt benutzt
  wird, ein Klick zeigt die Aufrufer) und ein **mitlaufender Blockkopf**:
  SUB, IF und FOR bleiben oben stehen, solange man in ihrem Rumpf ist.
* **Schnellkorrektur** (Strg+.): fehlende Variable anlegen, Tippfehler im
  Namen, fehlendes Blockende, fehlender IMPORT, unbenutzte Variable
  entfernen. Umbauten dazu: Variable einführen/einsetzen, SUB in FUNCTION.
* **Beim Einfügen einrücken:** ein eingefügter Block bekommt die
  Einrückung der Stelle, an der er landet (ein eigener Rückgängig-Schritt).
* **Groß/klein**, **Zeilen verbinden**, **Klammerpaare farbig** (jede Tiefe
  eine Farbe), **eigene Schnipsel** und **eigene Tastenbelegung**.

### Navigieren

* **Zurück / Vorwärts** (Strg+Alt+Links/Rechts, Seitentasten der Maus) über
  jeden Sprung, auch zwischen Dateien.
* **Nächstes / voriges Problem** (Alt+F8), **zur letzten Änderung**
  (Strg+Umschalt+Rücktaste), **im Projektbaum zeigen** (Strg+Alt+B).
* Die **Übersichtskarte** am rechten Rand lässt sich anklicken und ziehen.

### Rechtsklick

* **Im Code-Feld:** Ausschneiden, Kopieren, Einfügen, Zur Definition, Wer
  ruft das auf?, Umbenennen, Schnellkorrektur, Haltepunkt.
* **Auf einem Reiter:** Schließen, Andere schließen, Rechts davon schließen,
  Pfad kopieren, Im Projektbaum zeigen, Mit gesicherter Fassung vergleichen.
* **Im Projektbaum:** neue Datei, neuer Ordner, umbenennen (samt jedem
  `IMPORT` im Projekt, der die Datei meint), löschen, Pfad kopieren.
* **In der Ausgabe** und in den **Suchtreffern** (dort lassen sich einzelne
  Treffer ersetzen).

### Ausführen, Prüfen, Debuggen

* **Startargumente und Umgebung je Datei**, **Neustart beim Sichern**.
* **Ausgabe filtern und sichern.**
* **Ganzes Projekt prüfen** (Strg+Umschalt+F7) und **Prüfsammlungen**
  aus der IDE starten, samt Überblick über alle Sammlungen.
* **Werte der Variablen hinter den Zeilen** im Debugger-Halt.
* **Profil im Code:** nach einem Profillauf eine Wärmefarbe hinter jeder
  gemessenen Zeile und die Zahlen der teuersten.
* Ein **Terminal** in der unteren Leiste.

### Projekt und git

* **Neues Projekt aus Vorlage**, **Vorschau für Bilder und Klänge**, das Bild
  beim Überfahren eines `LOADIMAGE`-Pfads.
* In der Quellcodeverwaltung: **Zweige**, **Beiseitelegen** (stash),
  **Blame an der Zeile**, **Konflikte** auflösen.
* **Mit gesicherter Fassung vergleichen:** der Unterschied zwischen Reiter
  und Datei als farbiger diff, von dort sichern oder verwerfen.

### Aussehen

* **Englische Oberfläche** (Ansicht → Language: English, beim nächsten
  Start).
* **Eigene Farben** aus `farben.json` neben der `ide.json`: Code-Farben,
  Bereichstöne und jeder Schlüssel des GUI-Themas, je Thema.
* Der **Lichtstreif** kennt zwei weitere Arten (`lampe`, `rahmen`).

### Neue Befehle

`GUI_CONTEXT_WIDGET` und `GUI_CONTEXT_TABS`/`GUI_TAB_CONTEXT` (Kontextmenüs
für ein Widget bzw. die Reiterleiste), `GUI_TEXTAREA_HINTS` und
`GUI_TEXTAREA_HINT_CLICKED` (Text hinter dem Zeilenende, auch je Hinweis
eigene Farbe), `GUI_TEXTAREA_LINE_COLORS`/`_GET` (Bänder hinter ganzen
Zeilen), `GUI_TEXTAREA_SCROLL`, `CODE_NAMES` und `CODE_TYPES$`.
`PROCESS_START` nimmt ein Feld von Argumenten und eine MAP als Umgebung nur
des gestarteten Prozesses; `PROCESS_KILL` beendet den ganzen Prozessbaum.

## Unter der Haube

**Zahlen.** Die Befehlsreferenz wächst von 2014 auf **2024**
Einträge, es bleiben 48 Module und 216 Beispiele.
**4571 Fälle in 340 Prüfsammlungen** (vorher 4476 in
310) und **458 Rust-Testfunktionen** (vorher 451).

* `dhrt --check` meldet jetzt auch ein zweites `DIM` desselben Namens im
  selben Block, wenn der Typ gleich ist -- vorher überschrieb es stillschweigend
  die erste Variable. Gefunden, als ein neuer Menüeintrag der IDE den
  Alt+C-Schalter der Suche verdeckte.
* Ein eingefügter Block, dessen letzte Zeile `NEXT` oder `END IF` war, verlor
  im Code-Feld seine Einrückung. Behoben.
* Ein Kontextmenü eines Textbereichs öffnet nicht in dessen Nummernspalte
  (dort bleibt der Rechtsklick der für die Haltepunkt-Bedingung), und der
  Rechtsklick setzt vorher die Marke.

## Was offen bleibt

* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt.
* Der Installer ist nicht signiert; SmartScreen meldet sich beim ersten Start.
