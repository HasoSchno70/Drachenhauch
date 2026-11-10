# Drachenhauch 2026.41

*Die Notizen zu dieser Fassung. Sie baut **das Drachenhauch Studio** weiter
aus: Layout-Behälter im Entwurf, Projektvorlagen, ein Fenster, das sich am
Griff größer ziehen lässt. Dazu nimmt das **Drehfeld** endlich getippte
Zahlen – das traf jedes Programm mit Drehfeld. Für bestehende Programme
ändert sich nichts, was sie brechen könnte; was sich anders verhält, steht
gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Ein Drehfeld (`GUI_SPINNER`) nimmt Tastatur-Eingaben.** Bisher kannte es
  nur die Pfeiltasten und seine zwei Knöpfe. Mit dem Fokus ist der Wert jetzt
  markiert, und das erste Getippte ersetzt ihn. Wer bisher Buchstaben- oder
  Zifferntasten abfragte, während ein Drehfeld den Fokus hatte, bekommt sie
  dort nicht mehr – sie gehören dem Feld, wie bei einem Textfeld.

## Neu in dieser Fassung

### Drachenhauch Studio

* **Layout-Behälter im Entwurf:** Zeile, Spalte und Raster nehmen Controls
  auf. Ein Baustein, der auf einem Layout landet, kommt hinein, ebenso ein
  Control, das man hineinzieht; eine blaue Marke zeigt die Stelle. Kinder
  lassen sich umsetzen, herausnehmen (neben jedes Layout ziehen) und mit den
  Pfeiltasten verschieben. Der Inspektor hat eine Gruppe „Layout“
  (Anordnung, Spalten, Abstand, Rand, Ausrichtung, Kinder dehnen, Rahmen),
  ein Kind hat statt Links/Oben/Anker sein Gewicht; im Navigator stehen die
  Kinder unter ihrem Layout.
* **Mehrfachauswahl in Layouts:** mehrere gewählte Controls kommen zusammen
  hinein – in eine Zeile von links nach rechts, sonst von oben nach unten –
  und werden zusammen umgesetzt oder herausgenommen.
* **Abstandhalter** (Bibliothek, Gruppe Behälter): nimmt in einem Layout den
  freien Platz und schiebt, was danach kommt, ans Ende – etwa OK und
  Abbrechen an den rechten Rand.
* **Das entworfene Fenster am Griff größer ziehen:** rechte Kante, untere
  Kante und Ecke, am Raster. Verankerte Controls gehen mit, wie die Laufzeit
  sie beim Vergrößern legt – auch, wenn man Breite oder Höhe im Inspektor
  tippt.
* **Projektvorlagen:** „Neues Projekt“ (Strg+Umschalt+N) fragt, womit es
  anfangen soll – *Leer*, *Formular mit Eingaben* (Felder in einer Spalte,
  Prüfregeln, OK und Abbrechen), *Adressbuch mit Datenbank* (SQLite, Tabelle
  mit Filterzeile, gebundene Felder, Neu/Sichern/Löschen) oder
  *Zeichenfläche mit Timer* (ein Ball, den ein Timer bewegt und den man
  ziehen kann). Jede Vorlage läuft sofort mit F5; Sichern fragt nach einem
  Ordner, die Vorlage selbst bleibt unberührt.
* **Zahlen im Inspektor** lassen sich jetzt tippen (siehe Drehfeld unten).
* Ein frisch gestartetes, unberührtes Projekt galt als geändert – „Neu“
  fragte gleich nach dem Start „Sichern / Verwerfen / Abbrechen“. Jetzt
  nicht mehr.

### gui

* **Drehfeld mit Tastatur:** Bearbeiten über dieselbe Routine wie im
  Textfeld; es bleiben nur Ziffern, Vorzeichen und Komma/Punkt. Eine Zahl im
  Bereich gilt schon beim Tippen (`on_change`, `GUI_CHANGED`), Enter oder
  ein Fokuswechsel klemmt auf min/max, ein ganzzahliges Feld rundet. **Enter
  meldet `GUI_ENTERED`** wie das Textfeld, ESC holt den Wert von vor dem
  Tippen zurück und drückt nicht den Abbrechen-Knopf des Fensters.

### Glutfaden

* **Themenfarben im dunklen Erscheinungsbild:** die Leisten (rechts stand
  ein weißer Block), das Logo-Menü, die Aufklapper und der Zoom-Knopf der
  Statusleiste zeichnete Adwaita in seinen eigenen hellen Farben. Sie
  nehmen jetzt die Farben des Themas und wechseln mit ihm.

## Unter der Haube

**Zahlen.** 2251 Befehle in 48 Modulen (keiner neu), 226 Beispiele auf der
obersten Ebene von `examples/` (dazu das Studio-Projekt unter
`examples/studio/` und vier Vorlagen unter `studio/vorlagen/`), **5771 Fälle
in 473 Prüfsammlungen** (vorher 5751 in 468) und 608 Rust-Testfunktionen.

* Installer und Pakete nehmen `studio/vorlagen` mit.
* Die Glutfaden-Farbprobe wartet, bis Menü und Aufklapper wirklich gezeichnet
  sind – unter voller Last hatte das Menü schon eine Größe, aber noch kein
  Bild.

## Was offen bleibt

* **Das Studio** hat noch keine englische Oberfläche.
* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das Studio, Funkenflug, Glutfaden und der Fotobetrachter sind nur unter
  Windows ausprobiert.
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
