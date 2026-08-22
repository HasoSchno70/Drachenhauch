# Drachenhauch – Der Einstieg · Gliederung & Fortschritt

Lehrbuch für Menschen ohne jede Vorkenntnis. Leitsatz: **erst sehen, dann
verstehen.** Jedes Kapitel bringt mehrere sehr kurze Programme mit großer
Wirkung, und jede Zeile davon wird erklärt. Fortgeschrittene Techniken sind
willkommen, solange das Programm kurz bleibt.

Legende: [x] fertig · [~] angefangen · [ ] offen

## Regeln für dieses Buch

- **Wenig Zeilen, viel Wirkung.** Ein Beispiel über 20 Zeilen braucht einen
  guten Grund. Lieber vier kleine Programme als eines mit vier Abschnitten.
- **Jede Zeile wird erklärt**, auch die, die schon einmal vorkam — beim ersten
  Mal ausführlich, danach knapp.
- **Nichts wird behauptet, was nicht gemessen ist.** Fehlermeldungen im Buch
  stehen im echten Wortlaut; jeder Codeblock geht durch `dhrt --check`; jedes
  Bild wird angesehen, bevor es bleibt. Beim Schreiben von Kapitel 1 und 2 hat
  genau das drei falsche Aussagen und zwei falsche Bilder gefunden.
- **Kein Kapitel bringt einen Befehl, den es nicht braucht.** Arrays kommen
  in Kapitel 9, weil eine Schlange ohne sie nicht zu bauen ist — nicht, weil
  sie im Lehrplan an der Reihe wären.
- **Der Abdruck IST die Datei.** `pruef_abdruck.js` haelt jedes vollstaendige
  Programm im Buch gegen `code/kapNN/`. Wer abtippt und danach in die
  mitgelieferte Datei sieht, soll dasselbe vorfinden.
- **Am Kapitelende Aufgaben**, die zum Verändern einladen, nicht zum Abfragen.
- **Fehlerküche**: jedes Kapitel endet mit einer Tabelle „Was du siehst / was
  meistens dahintersteckt".

## Teil I — Bilder aus Zahlen

- [x] 01 Dein erstes Fenster · `SCREEN CLS CIRCLE PLOT FLIP SLEEP RGB`
      — Sonne, Ampel, Smiley
- [x] 02 Zahlen mit Namen · `DIM` Zuweisung Rechnen `BOX TEXT STR$`
      — wachsendes Gesicht, zwei Gesichter, Mitte, Farbmischer
- [x] 03 Einmal schreiben, tausendmal malen · `FOR NEXT`
      — Farbverlauf, Tunnel, Spirale, Schachbrett
- [x] 04 Der Zufall · `RND RANDOMIZE`
      — Sternenhimmel, Konfetti, zufällige Landschaft
- [x] 05 Bewegung · die Spielschleife, `WHILE WEND`
      — fallender Ball, Regen, kreisende Monde
- [x] 06 Entscheiden · `IF THEN ELSE`
      — abprallender Ball, Farbwechsel bei Berührung
- [x] 07 Die Tastatur · `KEYDOWN KEYPRESSED`
      — du steuerst ein Raumschiff
- [x] 08 **Projekt: Pong** für zwei Spieler
- [x] 09 **Projekt: Snake** · Arrays, weil eine Schlange sie erzwingt

## Teil II — Klang

- [x] 10 Der erste Ton · `AUDIO_TONE PLAYSOUND`
      — Tonleiter, Melodie, Sirene
- [x] 11 Klang zum Bild · Treffer, Laser, Schritte
- [x] 12 **Projekt: ein kleines Instrument** — Tasten spielen Töne

## Teil III — Ordnung schaffen

- [x] 13 Eigene Befehle · `SUB FUNCTION`
      — ein Baum, ein Wald aus einer Zeile
- [x] 14 Arrays, was noch drinsteckt · `FOR EACH`, Arrays von Text, sortieren
      — 500 Funken, Wellenlinie. Das Nötigste steht schon in Kapitel 9.
- [x] 15 Nachschlagen · Maps
      — Farbnamen, Punktestände
- [x] 16 Dinge mit Eigenschaften · Klassen
      — Partikel, Planeten
- [x] 17 Text · `LEFT$ MID$ LEN INSTR`
      — Laufschrift, Schreibmaschine
- [x] 18 Was bleiben soll · Dateien lesen und schreiben
      — Highscore, der den Neustart überlebt

## Teil IV — Sprites und Bewegung

- [x] 19 Bilder laden · `LOADIMAGE DRAWIMAGE`
- [x] 20 Selbst malen · der Pixel-Editor `dhsprites`
- [x] 21 Animation · Frames, Zustände
- [x] 22 Zusammenstoß · Kollisionsprüfung
- [x] 23 **Projekt: ein Arcade-Spiel** mit eigenen Figuren

## Teil V — Fenster mit Knöpfen

- [x] 24 Der erste Knopf · `GUI_BUTTON GUI_CLICKED`
- [x] 25 Eingeben und auswählen · Textfeld, Liste, Auswahlfeld
- [x] 26 Ordnung im Fenster · Layout, Reiter, Gruppen
- [x] 27 Daten, die bleiben · Datenbank statt Datei

## Teil VI — Abschlussprojekt

- [ ] 28 Der Vokabeltrainer: Idee und Aufbau
- [ ] 29 Vokabeln anlegen und speichern
- [ ] 30 Abfragen — und merken, was schwerfällt (Karteikasten)
- [ ] 31 Fortschritt zeigen, Klang, letzter Schliff
- [ ] 32 Weitergeben · `dhrt --export`

## Anhang

- [ ] A Farben zum Nachschlagen
- [ ] B Die Befehle dieses Buchs auf einen Blick
- [ ] C Wie es weitergeht — die anderen drei Bände
