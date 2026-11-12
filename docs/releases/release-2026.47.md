# Drachenhauch 2026.47

*Die Notizen zu dieser Fassung. Sie ist die Fassung **für eine ruhigere,
professionellere IDE**: eine Durchsicht der Oberfläche in dreizehn Punkten,
zwei Fehler, die gleich nach 2026.46 auffielen, und acht neue Abfragen für
die gui, mit denen ein Programm zurücklesen kann, was es gesetzt hat. Was
bestehende Programme bemerken können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Ein Klick geht an das oberste Widget.** Liegen zwei Widgets übereinander,
  bekam bisher das zuerst angelegte den Klick, gezeichnet wurde aber das
  zuletzt angelegte oben -- so ließ sich der OK-Knopf im Fenster „Über
  Drachenhauch“ nur mit Enter drücken. Jetzt fragen Klick, Zeichnen und
  `GUI_HIT_TEST` dieselbe Reihenfolge. Wer sich (unbeabsichtigt) darauf
  verlassen hat, dass ein darunterliegendes Widget den Klick bekommt, legt es
  zuletzt an.
* **Eine Beschriftung zeichnet ihren eigenen Grund und Rand:**
  `GUI_SET_COLOR(lbl, "bg", ..)` und `"border"` sowie `bg`/`border` aus der
  Stildatei galten für `GUI_LABEL` bisher nicht. Eine Stildatei, die Labels
  einen Grund gibt, zeigt ihn jetzt.
* **Der Rand eines großen Textbereichs im Fokus ist leiser:** über der
  Metrik `verlauf_hoehe` mischt sich der Akzent mit der Randfarbe. Ein am
  Widget gesetzter Akzent gilt unverändert.

## Neu in dieser Fassung

### IDE

* **Nach dem Startbildschirm** schreibt die IDE wieder in ihrer eigenen
  Schriftgröße -- vorher blieb die größere Schrift des Startbildschirms
  stehen.
* **Der Schriftzug hinter dem Code ist aus** (die Einstellungen, Strg+U, schalten
  ihn wieder ein); das Logo steht auf der Willkommensseite.
* **Pfadleiste mit Ordnern:** über dem Code steht `examples › 154_chart.dh ›
  sub stil_anwenden` statt noch einmal nur der Dateiname. Ein Klick auf einen
  Ordner zeigt ihn im Projektbaum.
* **Statuszeile kurz:** sie nennt den Weg ab dem Projekt; den ganzen Pfad
  zeigt ihr Hilfetext.
* **Probleme:** die Zahl steht im Reiter („Probleme (3)“), ohne Probleme
  steht daneben nichts mehr (das „keine“ sah aus wie ein zweiter Reiter);
  die Aufschlüsselung steht rechts und gedämpft.
* **Die Eingabezeile** für das laufende Programm erscheint nur, solange ein
  Programm, der Debugger oder das Terminal läuft; sonst bekommt die Ausgabe
  den Platz.
* **Ein Ton für alle Bereiche** statt vier Farben (blau, türkis, braun,
  violett) um den Code; die Hilfe zum Wort unter der Schreibmarke steht in
  einem eigenen Kasten unter der Problemliste. Eigene Töne gehen weiter über
  `farben.json`.
* **Parameternamen an Aufrufen** stehen nur noch in der Zeile der
  Schreibmarke (`"parameter_nur_zeile": false` in der `ide.json` zeigt sie
  wieder überall).
* **Werkzeugleiste mit größeren Sinnbildern**, die Filter über Projektbaum
  und Ausgabe sind Suchfelder mit Lupe und Kreuz.
* **Fehler oder Warnung:** das Handbuch sagt jetzt, was der Unterschied ist
  -- ein Fehler heißt, das Programm übersetzt nicht; ein Name, den es
  nirgends gibt, ist eine Warnung, denn das Programm läuft bis zu dieser
  Zeile.

### Studio

* Im Inspektor heißen die Zeilen **„Enter-Knopf“ und „Esc-Knopf“** -- die
  alten Beschriftungen waren abgeschnitten.

### gui

* **Acht Abfragen zu Settern:** `GUI_TOOLTIP$`, `GUI_TEXTINPUT_GET`
  (dieselben Schlüssel wie `GUI_TEXTINPUT_SET`, etwa `suche` oder
  `maxlaenge`), `GUI_TEXTINPUT_PLACEHOLDER$`, `GUI_STATUSBAR_TIP$`,
  `GUI_TOOLBAR_GET`, `GUI_DOCK_TITLE$`, `GUI_TEXTAREA_HINT$` und
  `GUI_TEXTAREA_HINT_COUNT`. Was ein Programm setzt, kann es jetzt auch
  zurücklesen -- vorher mussten sich Programme und Tests den Zustand selbst
  merken.

## Unter der Haube

**Zahlen.** 2441 Befehle (vorher 2433) in 57 Modulen, 233 Beispiele auf der
obersten Ebene von `examples/` (233), **5874 Fälle in 481
Prüfsammlungen** (5849 in 480) und 670 Rust-Testfunktionen (670).

* **Regeln für Prüfungen geschärft** (`CLAUDE.md`): eine Gegenprobe
  verfälscht die eine Stelle, nicht die ganze Änderung -- fällt ein Fall dann
  nur an „Variable nicht deklariert“, ist nichts bewiesen; was ein Fall
  misst, muss sich mit der Änderung bewegen; Zustand der gui fragt ein Test
  über die neuen Abfragen.

## Was offen bleibt

* **Echte Geräte sind weitgehend ungeprüft** (Raspberry Pi, SPI, Kamera,
  Mikrofon, Arduino, Modbus) -- Rückmeldungen mit Hardware sind willkommen.
* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
* Die fertigen Bücher und die Vorspann-Videos liegen noch im Repo (rund
  40 MB); sie sollen künftig nur noch als Downloads am Release hängen.
