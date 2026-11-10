# Drachenhauch 2026.35

*Die Notizen zu dieser Fassung. Im Mittelpunkt stehen **andockbare Leisten
und Plugins**: Die gui bekommt ein Dock, die IDE stellt ihre Leisten hinein
und lädt eigene Programme als Plugins, Glutfaden bekommt beides auch. Dazu
prüft der Übersetzer mehr, und der volle Testlauf braucht 14 statt 38
Minuten. **Eine Änderung betrifft bestehende Programme**, sie steht gleich
zuerst.*

## Was sich für bestehende Programme ändert

* **Die Zuweisung einer ganzen MAP wird geprüft.** Bisher reichte die
  Laufzeit jede MAP an jedes MAP-Ziel durch: Eine `MAP OF STRING` ging an
  eine `MAP OF INTEGER`, ein Literal mit einer Katze an eine `MAP OF Hund`.
  Jetzt gelten dieselben Regeln wie für Felder:
  * Gleicher Werttyp oder der einer Unterklasse passt. Die MAP behält ihren
    Werttyp, jeder Schreibzugriff prüft ihn.
  * Bei einer MAP ohne Werttyp wird jeder Wert geprüft
    (`Wert 'mia': Erwartet HUND ...`). Gehört sie sonst niemandem, nimmt sie
    den Werttyp an; so wird eine frische MAP von Ganzzahlen auch eine von
    Kommazahlen.
  * Sonst bricht die Zuweisung ab.

  Ein MAP-Literal hat nur noch einen Werttyp, wenn alle Werte dieselbe Art
  haben. Bisher gab der erste ihn vor, und `{"a": 1, "b": "x"}` galt als
  `MAP OF INTEGER` mit einem Text darin. Im Repository bricht dadurch kein
  Programm ab (geprüft mit `dhrt --check` über alle `.dh`-Dateien und mit
  allen Prüfsammlungen).

## Neu in dieser Fassung

### Ein Dock für andockbare Leisten (`GUI_DOCK`)

Das Dock gehört einem Fenster und verteilt Leisten auf die Bereiche links,
rechts und unten. Mehrere Leisten in einer Gruppe werden Reiter, die
Trennlinien zwischen den Bereichen und Gruppen lassen sich ziehen.

* **Am Reiter gezogen** wandert eine Leiste: auf einen anderen Kopf als
  Reiter, in eine Gruppe als Teilung, an den Rand der Mitte in einen eigenen
  Bereich. Losgelassen sonst irgendwo, schwebt sie obenauf; ein Doppelklick
  auf ihren Kopf dockt sie wieder an.
* **Inhalt** sind entweder Widgets des Fensters (`GUI_DOCK_ADD`; das Programm
  verteilt sie in `GUI_DOCK_RECT`, sobald `GUI_DOCK_CHANGED` es meldet) oder
  ein ganzes Fenster (`GUI_DOCK_WINDOW`).
* **Die Anordnung** lässt sich als JSON sichern und wiederherstellen
  (`GUI_DOCK_STATE$`, `GUI_DOCK_SET_STATE`). Eine später angemeldete Leiste
  findet ihren gemerkten Platz.

27 neue Befehle, beschrieben in `docs/module-gui.md`.

### Die IDE: Leisten im Dock und Plugins

* **Leisten.** Links stehen Projekt und Gliederung, unten Ausgabe/Terminal
  und Probleme/Debugger als Reiter; der Reiter „Debugger“ ist nur beim
  Debuggen da. Die Vorgabe hat die bisherigen Maße. Die Anordnung steht in
  der `ide.json`, „Ansicht → Leisten zurücksetzen“ stellt die Vorgabe her.
* **Plugins** sind eigene Drachenhauch-Programme in
  `plugins/<name>/plugin.dh` neben der `ide.json`. Jedes läuft als eigener
  Prozess; ein Absturz reißt die IDE nicht mit. Ein Plugin kann:
  * Befehle mit Kürzel anmelden,
  * eine eigene Leiste aus einer `.dhform` zeigen,
  * auf Ereignisse hören (gesichert, geöffnet, geändert),
  * die IDE fragen (Text, Datei, Auswahl, Marke, Projekt),
  * Befunde in die Liste der Probleme schreiben,
  * Schnipsel beitragen und weitere Dateiarten einfärben.

  Plugins verwalten und neu laden geht ohne Neustart. Das Handbuch steht in
  `docs/ide-plugins.md`, ein Beispiel in `examples/plugins/todo/`.
* **Der Lichtstreif stockt beim Start nicht mehr.** Mit einer offenen Datei
  prüfte die IDE sie gut eine halbe Sekunde nach dem Start; bei 18 000
  Zeilen hielt das ein Bild 614 ms fest, und der Streif sprang um ein
  Viertel seines Wegs. Jetzt geht der Streif höchstens 50 ms je Bild weiter,
  und die Prüfung wartet, bis er durch ist. Die Parameternamen an Aufrufen
  kommen dabei viel schneller: 1938 Hinweise in 225 ms statt 60 je Prüfung.

### Glutfaden: Leisten und Plugins

* **Andockbare Leisten.** Übersicht, Muster, Info und Fortschritt stehen in
  vier Notizbüchern einer Gruppe: links, rechts oben, rechts und unten. Am
  Reiter gezogen wandern sie, außerhalb losgelassen bekommen sie ein eigenes
  Fenster. Die Anordnung steht in den Einstellungen, „Leisten zurücksetzen“
  im Logo-Menü.
* **Plugins** über dieselbe Leitung wie die IDE, in `plugins/<name>/` im
  Einstellungsordner: Befehle mit Kürzel im Logo-Menü, eine Leiste aus einer
  `.dhform`, Ereignisse. Neu sind `PLUGIN_MUSTER$` (das Muster als JSON) und
  `PLUGIN_STICHE` (Stiche setzen, ein Schritt im Verlauf, also rückgängig zu
  machen).

### Der Übersetzer prüft mehr

* `dhrt --check` kennt die **Rückgabetypen eigener Funktionen und Methoden**.
  `k = hunde()` an ein `ARRAY OF Katze` warnt jetzt, wenn `hunde()` als
  `FUNCTION ... AS ARRAY OF Hund` angesagt ist.
* **MAPs, die nie passen**, werden gemeldet wie Felder: `MAP OF STRING` an
  `MAP OF INTEGER`, `MAP OF Hund` an `MAP OF Katze`, ein Feld an eine MAP.
* **Unbekannte Felder hinter Feldelementen.** `m.farben[0].garn.rgb` blieb
  bisher still, `g.rgb` nicht. Jetzt trägt ein Element den Typ seines Feldes
  bzw. seiner MAP:

  ```basic
  CLASS Garn
      DIM farbe AS INTEGER
  END CLASS
  DIM garne AS ARRAY OF Garn
  garne = [NEW Garn()]
  PRINT garne[0].farbe
  ```

  Mit `garne[0].frabe` kommt die Warnung samt „Meintest du 'farbe'?“.

### Weitere Änderungen

* Ein Feld ohne Typ, das noch einem `ARRAY OF ANY` gehört, nimmt bei der
  Übergabe an ein getyptes Feld dessen Typ nicht an (beschrieben in
  `docs/sprache.md`).
* **Rückrufe einer Bibliothek** sahen in einem Fall die Globalen als NIL:
  Rief eine Schleife im Maschinencode eine Funktion, die eine Bibliothek
  aufruft, und rief die Bibliothek einen Rückruf zurück (GTK, `qsort` mit
  Vergleich), fehlten ihm die Globalen, die die Schleife gerade hielt.
  Solche Schleifen bleiben jetzt in der VM.

## Unter der Haube

**Zahlen.** 2181 Befehle in 48 Modulen (neu: 27 `GUI_DOCK_*`), 225 Beispiele
auf der obersten Ebene von `examples/`, **5445 Fälle in 440 Prüfsammlungen**
(vorher 5408 in 436) und 565 Rust-Testfunktionen.

* **`dhrt test` läuft Dateien nebeneinander.** Ein voller Lauf brauchte
  38 Minuten, 84 % davon gingen auf Dateien, die nur nacheinander liefen, weil
  sie über die Zwischenablage tippen. Jedes Programm, das `dhrt test` startet,
  bekommt jetzt eine eigene Zwischenablage (`DHRT_ZWISCHENABLAGE=privat`), und
  Dateien ohne `--- seriell` teilen sich bis zu acht Plätze. Der volle Lauf
  braucht 14 Minuten, `--schnell` (ohne die Sammlungen mit `--- langsam`)
  rund 6.
* Die Sprachprobe von Glutfaden schrieb ihre Dateien neben das Programm, also
  ins Repository. Sie schreibt jetzt in den Ordner des Falls.

## Was offen bleibt

* Eine schwebende Leiste aus Widgets liegt unter einer angedockten Leiste,
  die ein ganzes Fenster ist.
* Der Hover der IDE kennt den Typ eines Objekts nicht und sagt zu
  `spieler.hp` nichts.
* **raylib 6.2** ist erschienen, die Rust-Bindung raylib-rs hat aber noch
  keine Fassung dafür. Drachenhauch bleibt bei raylib 6.0, bis raylib-rs
  nachzieht.
* Funkenflug, Glutfaden und der Fotobetrachter sind nur unter Windows
  ausprobiert (mit dem GTK von Inkscape).
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
