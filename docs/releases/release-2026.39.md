# Drachenhauch 2026.39

*Die Notizen zu dieser Fassung. Im Mittelpunkt steht **die gui**: sie
zeichnet in einer echten Schrift, lässt sich per Stildatei gestalten wie
GTK mit CSS, prüft die Rechtschreibung, schreibt von rechts nach links und
hat Popover, Toasts, ein Fließlayout und einige Bausteine mehr. Dazu Felder
mit eigener Untergrenze (`DIM tage[1 TO 31]`) und eine IDE, die ihre
Vererbung kennt. **Drei Kleinigkeiten ändern sich für bestehende
Programme**; sie stehen gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Die gui zeichnet in der eingebauten Sans statt in der Pixelschrift.**
  Ohne `GUI_FONT` und ohne `SETFONT` nehmen Fenster, Knöpfe, Listen, das
  Modul `ui` und die Diagramme DejaVu Sans in der Textgröße des Programms.
  `TEXT()` und die übrigen Zeichenbefehle bleiben pixelig, Spiele sehen aus
  wie bisher. Wer den alten Anblick will: `GUI_FONT(-1)` schaltet gui, `ui`
  und Diagramme gemeinsam zurück. Die Felder passen unverändert, weil die
  Versalhöhe der Sans kleiner ist als die der Pixelschrift.
* **Meldungen zählen die Dimension eines Feldes ab 1** („Index 5 ausserhalb
  [0..4] in Dimension 1“, vorher „Dimension 0“). Wer eine Meldung mit
  `CATCH` auswertet, prüft die Zahl nach.
* **Text mit Hebräisch oder Arabisch wird von rechts nach links
  gezeichnet**, in den Textbefehlen und jedem Widget (nicht im PDF),
  Arabisch mit seinen verbundenen Formen. Wer eine Zeile schon selbst umgedreht hat, bekommt sie
  jetzt doppelt gedreht und lässt das Umdrehen weg.

## Neu in dieser Fassung

### Die gui

* **Stildatei** (`GUI_STYLESHEET`, `GUI_STYLESHEET_LOAD`): Regeln nach Art,
  Klasse (`GUI_ADD_CLASS`), Name (`GUI_SET_NAME`) und Zustand (`:hover`,
  `:focus`, `:active`, `:disabled`, `:checked`), Vorrang wie in CSS.
  Farben, Rundung, Schrift, Innenabstand, Randbreite, Schatten, Verlauf,
  Glanz, Fase und Übergang je Widget; **Nachfahren** wie
  `panel.leiste button`, `layout > button` oder `window#einstellungen button`
  (`GUI_WINDOW_SET_NAME`), und der Übergang auch **je Farbe**
  (`uebergang: 120, bg 400`). Ein Fehler in der Datei ist eine Meldung mit
  Zeile.
* **Popover, Toast und Infoleiste** (`GUI_POPOVER`, `GUI_TOAST`,
  `GUI_INFOBAR`), dazu im Textfeld **Sinnbilder**, ein **Suchfeld** mit
  Verzögerung und **Vorschläge** beim Tippen.
* **Bausteine:** Fließlayout, Alt-Buchstaben in Beschriftungen
  (`GUI_MNEMONICS`), Beschriftungen, die sich kürzen und markieren lassen,
  Marken und Einrasten am Regler, ein Kreisel für „arbeitet“
  (`GUI_BUSY`), Marken im Kalender, Größengruppen und ein Dialog zur
  Schriftwahl.
* **Rechtschreibung** (`SPELL_*`, `GUI_SPELL`): unter Windows die Prüfung
  des Systems, sonst Hunspell-Wörterbücher; Wellenlinien in Textfeld und
  Textbereich, Vorschläge per Rechtsklick.
* **Rechts nach links** samt arabischer Formen, dazu
  `TEXT_VISUAL$(text$)`.
* **Eingebaute Sans als Vorgabe** (siehe oben), auch für `ui` und
  Diagramme. Ein Diagramm mit eigener Schrift sitzt jetzt richtig (es hatte
  mit der aktiven Schrift gemessen).

### Sprache

* **Felder mit eigener Untergrenze:** `DIM tage[1 TO 31]`,
  `DIM f[-2 TO 2, 5]`, Grenzen als Ausdrücke. Neu `LBOUND` und `UBOUND`.
  `ARRAY_INDEXOF`, `ARRAY_FIND`, `ARRAY_INSERT` und `ARRAY_REMOVE_AT` zählen
  in den Indizes des Feldes; `PRINT` zeigt die Grenzen, der Debugger zählt ab
  `LBOUND`, `dhrt --check` warnt bei einem festen Index außerhalb. Der
  Maschinencode rechnet solche Felder selbst (rund 60-mal schneller als in
  der VM); Felder ab 0 bleiben gleich schnell.

### IDE

* **Vererbung:** `ABSTRACT` steht in Gliederung, Klassendiagramm und
  Klassen-Plugin; F12 sucht in der eigenen Klasse und ihren Vorfahrinnen;
  an den Methoden stehen Hinweise „überschreibt“, „füllt aus“,
  „nirgends ausgefüllt“, auch über Dateigrenzen; Schnellkorrektur
  „Fehlende Methoden einfügen“.
* **Größe der Oberfläche** in den Einstellungen (100 bis 300 %, sonst wie
  das System).
* Neun Schnipsel mehr, kontextabhängige Wörter richtig eingefärbt,
  Willkommensseite und PBR-Reaktor im Vollbild berichtigt.
* Innen auf Klassen umgebaut: rund 35 Klassen ersetzen Gruppen paralleler
  Felder. Dabei fiel ein Absturz beim Ändern eines Kürzels auf,
  nachdem ein Plugin einen Befehl angemeldet hatte.

## Unter der Haube

**Zahlen.** 2247 Befehle in 48 Modulen (59 neu, darunter `GUI_STYLESHEET`,
`GUI_ADD_CLASS`, `GUI_POPOVER`, `GUI_TOAST`, `GUI_INFOBAR`, `GUI_SPELL`,
`SPELL_CHECK`, `TEXT_VISUAL$`, `LBOUND`, `UBOUND`), 226 Beispiele auf der
obersten Ebene von `examples/`, **5704 Fälle in 461 Prüfsammlungen** (vorher
5580 in 453) und 607 Rust-Testfunktionen.

* Neue Crates: `unicode-bidi` (Reihenfolge rechts nach links), `spellbook`
  (Hunspell-Wörterbücher).
* Die Zählung im Befehlsverzeichnis stand auf 2244, es sind 2247.

## Was offen bleibt

* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **Der Form-Designer** kennt Stildatei, Klassen und die neuen Bausteine
  noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Funkenflug, Glutfaden und der Fotobetrachter sind nur unter Windows
  ausprobiert; die Rechtschreibung mit Hunspell unter macOS und Linux nur
  über die Prüfsammlungen.
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
