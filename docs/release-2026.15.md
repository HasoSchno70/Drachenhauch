# Drachenhauch 2026.15

*Die Notizen zu dieser Fassung. Zwei Tage und 16 Pull Requests nach 2026.14,
der ersten Fassung ohne Python. Diesmal ging es um das, was man beim Arbeiten
jeden Tag anfasst: eine IDE, die sich besser lesen und bedienen lässt,
Oberflächen-Bausteine, die sich wie gewohnt verhalten, und Meldungen, die
Umsteigern aus anderen BASICs sagen, was gemeint ist. Und die IDE beginnt
jetzt mit einem Vorspann.*

## Neu in dieser Fassung

### Video, und ein Vorspann für die IDE

Das neue Modul **`video`** spielt MP4-Dateien mit H.264 ab — dem Format, das
praktisch jedes Programm und jedes Telefon schreibt. Jedes Bild landet in einem
gewöhnlichen `IMAGE`: zeichnen, skalieren, als Textur nehmen wie jedes andere
Bild (`VIDEO_LOAD`, `VIDEO_PLAY`, `VIDEO_DRAW`, `VIDEO_SEEK`, `VIDEO_LOOP` …).
Die Bilder dekodiert Ciscos openh264, das beim Bau der Laufzeit mitübersetzt
wird — beim Nutzer muss nichts installiert sein. Ton spielt das Modul nicht
ab.

Die IDE zeigt damit beim Start einen **Vorspann**. Jede Taste und jeder Klick
überspringt ihn; wer ihn gar nicht sehen will, schaltet ihn in den
Einstellungen ab („Vorspann beim Start“).

### Die IDE, besser zu lesen

* **Drachen-Symbol und Schriftzug:** Verknüpfungen, `.dh`-Dateien und das
  Fenster tragen das Logo, die Willkommensseite den Schriftzug; *Über
  Drachenhauch* zeigt beides.
* **Umlaute** in Menüs, Knöpfen und Meldungen — die Suche findet „Öffnen“
  auch, wenn man `oeffnen` tippt.
* **Codefarben je Thema** statt der dunklen Töne auch auf Weiß, eine
  **zweifarbige Gliederung**, **getönte Bereiche** (Projekt, Ausgabe,
  Probleme, Debugger), eigene **Schriftgrößen** für Listen und Handbuch, und
  gedämpfter Text, der den Kontrast nach WCAG hält.
* **Beispiele nach Themen:** alle Beispiele in 16 Gruppen, mit Filter und
  *Öffnen und starten*.
* **Handgriffe:** Klammern und Anführungszeichen schließen sich selbst,
  dieselbe Datei nebeneinander, Formatieren beim Sichern, alle Fundstellen,
  Faltung je Datei, Druckoptionen, ein Filter über dem Projektbaum.

### Debugger

Er zeigt jetzt den **Aufrufstapel**, hat eine Liste zum **Überwachen** von
Ausdrücken, läuft **bis zur Schreibmarke** (Strg+F10) und **setzt Werte** im
angehaltenen Programm (Doppelklick auf eine Variable).

### Oberfläche (`gui`)

* Neue Bausteine: **Akkordeon**, **Assistent** (mit Prüfung je Schritt),
  **Baumtabelle**, **Kachel** als Knopf, eine bearbeitbare **Klappliste**
  (Combobox).
* **Schriftstile** — fett, kursiv, unterstrichen, durchgestrichen, mit echten
  Schnitten der Schrift, wo es sie gibt (`TEXT_STYLE`, `GUI_SET_FONT_STYLE`).
  Vorher taten `TEXT_BOLD`/`TEXT_ITALIC` nichts.
* **Übergänge:** Überfahren, Aufklappen und Aufrollen blenden weich statt zu
  springen.
* **Bedienung wie bei einer Liste**, nun überall: Tabelle, Baum, Klappliste
  und Menüs haben Rollbalken, weiches Mausrad, Tippen-zum-Springen,
  Rechtsklick wählt, Umbenennen mit F2. Das Code-Feld wählt mit Doppelklick
  ein Wort und mit Dreifachklick eine Zeile und kann fett, kursiv und
  unterstrichen formatieren (als Markdown lesbar).
* Das **Mausrad** rollt nur noch das Fenster, das oben liegt.

### Werkzeuge

* Der **Tracker spielt Sample-Instrumente** (in 2026.14 noch offen) —
  Samples, Keymaps und eingelesene SoundFonts klingen beim Abspielen,
  Vorhören und in der WAV.
* Der **Form-Designer** kann Mehrfachauswahl, ein Menü *Anordnen*
  (ausrichten, gleiche Maße, verteilen) und Controls auf Reiterseiten; *Neu*
  und *Öffnen* fragen jetzt, wenn etwas ungesichert ist.

### Umstieg aus anderen BASICs

Fünf Runden lang wurde aufgeschrieben, woran jemand aus QBasic, FreeBASIC
oder Visual Basic stolpert — und dann entweder möglich gemacht oder mit einer
Meldung versehen, die sagt, wie es hier heißt: `DIM x AS T = wert`,
`EXIT FOR`, `SWAP`, `LET`, `?`, `INPUT "x"; v`, `EOF`, `CALL`, `BYVAL`, ein
Name allein ruft die SUB auf. Einiges davon war ein echter Fehler:
**`STEP 0` brachte dhrt zum Absturz**, eine Methode ohne Klammern tat still
nichts, und **Zeichnen ohne `SCREEN` tat still nichts** — jetzt sagt eine
Meldung, was fehlt. Ein unbekannter Befehl schlägt den gemeinten vor, und
eine falsche Argumentzahl nennt die Form des Befehls. Die Übersicht steht in
`docs/umstieg.md`.

### Laufzeit

* **FLAC als Musik mit Schleife** blieb stumm (ein Fehler in Symphonia beim
  Zurückspringen) — jetzt eigener Leser.
* Neue Befehle unter anderem `SAMPLE_FROM_BUFFER`, `SAMPLE_NOTE`,
  `GUI_CARD`, `GUI_WINDOW_AT`, `GUI_LISTBOX_SPANS`, `GUI_TEXTAREA_PAIRS`,
  `GUI_TEXTAREA_SHARE`; `READALL$` nimmt auch einen Pfad.

## Unter der Haube

**Zahlen.** Die Befehlsreferenz von 1911 auf **1993** Einträge, die
Modulliste von 47 auf **48** (`video`), die Beispiele von 215 auf **216**.
**3981 Fälle in 283 Prüfsammlungen** (vorher 3602 in 257) und **431
Rust-Testfunktionen** (vorher 404).

## Was offen bleibt

* Das Video-Modul kann nur H.264 und keinen Ton. H.264 ist patentbelastet;
  Cisco übernimmt die Lizenzgebühren nur für die fertigen openh264-Dateien,
  die Cisco selbst verteilt — diese Laufzeit übersetzt openh264 aus dem
  Quelltext.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt.
* Der Installer ist nicht signiert; SmartScreen meldet sich beim ersten Start.
