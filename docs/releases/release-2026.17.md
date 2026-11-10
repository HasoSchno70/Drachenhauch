# Drachenhauch 2026.17

*Die Notizen zu dieser Fassung. Einen Tag und neun Pull Requests nach
2026.16, und alle neun gelten der IDE: der Debugger hält jetzt dort, wo man
es ihm sagt, auch in importierten Dateien, Fehler stehen unterstrichen im
Code, und das Handbuch ist ein Buch zum Blättern statt einer Textwand.*

## Neu in dieser Fassung

### Der Debugger

Haltepunkte hielten nicht verlässlich. Das hatte sechs Ursachen, und alle
sind behoben:

* **Datei und Zeile stimmen.** `dhrt debug` zählte die Zeilen der
  zusammengefügten Quelle: nach einem `IMPORT` hielt kein Haltepunkt, und
  der Halt stand an der falschen Zeile. Jetzt nennt jedes Ereignis die echte
  Datei und ihre Zeile, und Haltepunkte in importierten Dateien halten.
* **Haltepunkte während des Laufens** wirken sofort, nicht erst beim
  nächsten Halt. F6 hält ein laufendes Programm an.
* **Marken wandern mit ihrer Zeile.** Wer über einem Haltepunkt eine Zeile
  einfügt, verschiebt ihn mit, statt ihn auf fremdem Code liegen zu lassen.
* Ein Haltepunkt auf einer Zeile **ohne Code** rutscht zur nächsten mit
  Code. `INPUT` fragt unter dem Debugger wirklich nach.

Dazu kam, was man beim Suchen eines Fehlers braucht:

* **Variablen als Baum.** Felder, MAPs, Tupel und Objekte lassen sich
  aufklappen, und was aufgeklappt war, bleibt es beim nächsten Halt.
* **Halt am Fehler.** Ein Laufzeitfehler, den kein `CATCH` nimmt, hält an
  seiner Stelle an, mit Variablen und Aufrufstapel.
* **Werte beim Überfahren.** Steht der Debugger, zeigt der Tooltip über einem
  Namen seinen Wert (`held.hp = 7`). Ausgewertet werden auch Felder und
  Indizes, aber nie ein Aufruf, damit Hinsehen nichts verändert.
* **Liste aller Haltepunkte** (Strg+Umschalt+F9) zum Ein- und Ausschalten.
  Dazu **Bedingungen**, eine **Trefferzahl** (ab dem 5. Mal, genau beim 5.,
  jedes 5. Mal) und **Protokollpunkte**, die eine Zeile in die Ausgabe
  schreiben statt anzuhalten.
* **Neu starten** (Strg+Umschalt+F5) in derselben Art (Lauf, Debugger,
  Profil).
* Ein Grafikfenster bleibt im Halt ansprechbar. Vorher meldete Windows es
  nach fünf Sekunden als „reagiert nicht“.
* Beim Suchen der Fehler aufgefallen: Bedingungen kannten `MOD`, `\` und `^`
  nicht. `i MOD 2 = 0` hielt deshalb bei jedem Durchlauf.

### Fehler im Code

Fehler (rot) und Warnungen (gelb) sind **an ihrer Stelle unterstrichen**,
die Meldung steht im Tooltip. Auch VS Code bekommt über `dhrt lsp` jetzt die
genaue Stelle statt der ganzen Zeile.

### Das Handbuch

* **Gesetzt wie ein Buch:** Codeblöcke in den Farben des Editors, Tabellen
  mit Kopf- und Zebra-Band, links ein **Inhaltsverzeichnis**, das beim Lesen
  mitgeht, **Zurück und Vorwärts** oben.
* **Knöpfe an jedem Codeblock:** *Kopieren*, *In neuen Reiter* und
  *Starten*. Ein Beispiel aus dem Handbuch läuft damit mit einem Klick.
* **Suche über alle Dokumente** („In allen“): die Treffer stehen je Dokument
  in der Inhaltsliste, ein Klick springt an die Stelle.
* Sprungmarken (`#fenster-und-frame`) treffen die Überschrift, nicht mehr
  die erste Erwähnung irgendwo darüber.

### Die IDE

* Die **Schrift der Oberfläche** lässt sich einstellen, Vorgabe 18 statt fest
  16.
* Der Schriftzug steht schwach **hinter dem Code**, abschaltbar.
* Beim Start und beim Starten eines Programms läuft ein **Lichtstreif** über
  das Fenster, ebenfalls abschaltbar.
* Das **Profil** stimmt nach einem `IMPORT` und springt in die richtige Datei.
* Vor dem Starten werden **alle** geänderten Reiter gesichert, nicht nur der
  vordere. Eine geänderte importierte Datei lief sonst in alter Fassung.
* Haltepunkte und Lesezeichen bleiben über das Beenden hinaus erhalten.
* Ein Ordner als Startargument wird das Projekt.
* Leere Listen sagen, dass sie leer sind („Keine Probleme gefunden“).

### Neue Befehle

`GUI_WINDOW_GLOW` (Lichtstreif) samt `_SET`, `_STOP` und `GUI_WINDOW_GLOWING`,
`GUI_TEXTAREA_SQUIGGLES` (Wellenlinien), `GUI_TEXTAREA_BACKGROUND`,
`GUI_TEXTAREA_MARKS_GET`, die Überschriften und Codeknöpfe des gesetzten
Texts (`GUI_RICHTEXT_HEADINGS`, `_HEADING_LEVELS`, `_GOTO_HEADING`,
`_HEADING_AT`, `_CODE_BUTTONS`, `_CODE_ACTION$`, `_CODE$`),
`IMAGE_COLOR_TO_ALPHA` (Hintergrundfarbe freistellen wie in GIMP),
`REALPATH$` und `SAMEFILE`.

## Unter der Haube

**Zahlen.** Die Befehlsreferenz wächst von 1997 auf **2014** Einträge. Es
bleiben 48 Module und 216 Beispiele. **4476 Fälle in 310 Prüfsammlungen**
(vorher 4401 in 297) und **451 Rust-Testfunktionen** (vorher 439).

`dhrt debug` spricht jetzt in Dateien und Zeilen, `set-breakpoints` nimmt
eine Liste mit Datei, Zeile, Bedingung, Trefferzahl und Protokolltext und
antwortet, wo jeder Haltepunkt wirklich hält. `dhrt profile` nennt die Datei
jeder Zeile. Das Protokoll steht in `docs/ide.md`.

## Was offen bleibt

* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt.
* Der Installer ist nicht signiert; SmartScreen meldet sich beim ersten Start.
