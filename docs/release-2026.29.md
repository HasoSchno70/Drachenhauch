# Drachenhauch 2026.29

*Die Notizen zu dieser Fassung. Wieder zwei Dogfooding-Runden: ein
Kommandozeilen-Werkzeug und ein Step-Sequencer. Beide wurden benutzt wie von
jemandem, der sie braucht — aus fremden Ordnern, über Pipes, als exportierte
Exe, mit Tasten und Klicks. **Vier Änderungen betreffen bestehende
Programme** (gleich zuerst).*

## Was sich für bestehende Programme ändert

* **`AUDIO_PLAY_AT` stoppt die vorige Instanz desselben Klangs nicht mehr.**
  Ein Sequencer, der dieselbe Hi-Hat mehrere Schritte voraus plant, löschte
  damit bisher die wartende — am Mischer kam von zwei Einsätzen einer an.
  Jetzt legt jedes `AUDIO_PLAY_AT` eine weitere Instanz an; `AUDIO_STOP`,
  `AUDIO_PAUSE`, `AUDIO_RESUME`, `AUDIO_VOLUME`, `AUDIO_PITCH` und die
  Pan-Befehle wirken auf alle. `AUDIO_PLAY` spielt den Klang weiter genau
  einmal.
* **`AUDIO_IS_PLAYING` ist für einen geplanten Klang erst ab seinem Tick
  wahr.** Vorher galt er schon als spielend, während er noch wartete. Neu
  sagt `AUDIO_IS_WAITING`, ob er noch wartet; mit einer entfernten Uhr ist
  beides FALSE.
* **Ein Tastenkürzel ohne Strg oder Alt gehört dem Bedienelement mit Fokus,
  wenn es die Taste selbst benutzt.** Mit `Leertaste` als Kürzel für
  Spielen/Stopp kippte ein Kästchen mit Fokus UND das Kürzel feuerte. Jetzt
  gehen Leertaste und Enter an Knöpfe, Kästchen, Listen und Tabellen,
  Pfeile, Pos1, Ende und Bild an Regler, Listen, Bäume und Reiterwerke,
  Buchstaben an Listen, Bäume, Tabellen und Klapplisten. F1 bis F12 lösen
  weiter immer aus.
* **`FORMAT$` mit einer Maske ohne Platzhalter ist ein Fehler.** Eine
  VB-Maske wie `"#,##0.00"` gab bisher die Maske zurück, der Wert ging still
  verloren. Für deutsche Zahlen gibt es jetzt `ZAHL$` (unten).

## Neu in dieser Fassung

### Werkzeuge für die Kommandozeile

* **`STARTDIR$()`** ist der Ordner, aus dem das Programm gestartet wurde, und
  **`ARGPATH$(n)`** das n-te Argument als Pfad von dort aus. `dhrt run` und
  eine exportierte Exe wechseln in den Ordner des Programms — ein relativer
  Pfad aus der Kommandozeile zeigte darum ins Leere. Absolute Pfade, leere
  Argumente und `-` bleiben, wie sie sind.

  ```basic
  ' Aufruf aus C:\arbeit:  werkzeug.exe daten\obst.txt
  PRINT ARG$(0)        ' daten\obst.txt
  PRINT ARGPATH$(0)    ' C:\arbeit\daten\obst.txt
  PRINT STARTDIR$()    ' C:\arbeit
  ```
* **`INPUT` nach `STDIN()` hing.** Das Handle hielt die Standardeingabe fest.
  Jetzt liest `INPUT` danach aus demselben Puffer weiter. Eine Pipe mit
  400 000 Zeilen braucht 1,2 statt 2,8 s.
* **`ZAHL$(x [, stellen])`** schreibt Zahlen deutsch, mit Tausenderpunkten
  und Komma: `ZAHL$(1234567.891, 2)` ist `1.234.567,89`.
* **Kodierung `"auto"`** beim Lesen: UTF-8, sonst cp1252. Eine Datei mit
  Nullbytes ist eine Binärdatei und bekommt eine eigene Meldung.
* **`TERMINAL()`** sagt, ob die Ausgabe in ein Terminal geht, und
  **`TERMINAL_WIDTH()`** liefert dann die Zahl der Spalten. Farben und eine
  Fortschrittszeile gehören nur dorthin.
* **`dhrt --export x.dh meins.exe`** legte einen Ordner `meins.exe` an. Ein
  Ziel auf `.exe` ist jetzt der Dateiname.
* Wer aus einem anderen BASIC kommt, bekommt für `ISDIR`, `ISFILE`,
  `COMMAND$`, `CURDIR$` und `ISATTY` den Namen, der hier gilt.

### Musik

* **`AUDIO_RECORD_START([max_sekunden])`**, **`AUDIO_RECORD_STOP()`** und
  **`AUDIO_RECORDING()`** schneiden den fertigen Mix mit — hinter allen
  Effekten der Busse, so wie er aus dem Lautsprecher kommt. `AUDIO_SOUND_MIX`
  baut dagegen trocken; Hall und Echo wirken nur live.

  ```basic
  IMPORT "audio"
  AUDIO_RECORD_START()
  ' ... spielen lassen, bis das Stück durch ist ...
  AUDIO_SAVE_WAV(AUDIO_RECORD_STOP(), "aufnahme.wav")
  ```
* **`AUDIO_PEAK()`** liefert den höchsten Ausschlag am Master seit dem
  letzten Aufruf. Einmal je Bild abgefragt ist das eine Pegelanzeige; über
  1.0 übersteuert der Mixer.
* Ein Tick, der schon vorbei ist, klingt sofort. Die Doku zeigt, wie man
  nach einem Hänger verpasste Schritte überspringt, statt sie auf einmal
  nachzuholen.

### Zwei neue Beispiele

* **`examples/textwerk`**: ein Kommandozeilen-Werkzeug mit drei Befehlen —
  `suche` wie grep, `csv` mit Gruppen, Summen und Sortieren, `zaehle` wie wc.
  Es liest Dateien und Pipes und läuft auch als exportierte Exe.
* **`examples/sequencer`**: sechzehn Schritte, sechs Spuren, Swing, Hall,
  Echo und Wah. Gespielt wird auf einer Audio-Uhr, drei Schritte voraus
  geplant. Muster lassen sich sichern, als trockene WAV schreiben oder samt
  Effekten mitschneiden, und die Statusleiste zeigt den Pegel.

## Unter der Haube

**Zahlen.** **2130** Befehle in 48 Modulen (vorher 2120), 225 Beispiele auf
der obersten Ebene von `examples/` (die zwei neuen liegen in eigenen
Ordnern). **5019 Fälle in 392 Prüfsammlungen** (vorher 4980 in 387) und
**549 Rust-Testfunktionen**.

* Einsätze von Klängen zählen die Prüfungen am Mischer, über `AUDIO_FFT`,
  nicht über `AUDIO_IS_PLAYING`. Der Sequencer wird an seinen WAVs geprüft:
  Schritte, Swing, stumme Spuren und der Mitschnitt mit Hall.
* Kira dreht die Master-Lautstärke erst hinter den Effekten. Mitschnitt und
  Pegel rechnen sie darum selbst ein. Eine Master-Lautstärke über
  `AUDIO_MODULATE` kennen sie nicht.

## Was offen bleibt

* `dhrt test` kennt keine Zeitgrenze je Fall; ein hängender Fall hält den
  ganzen Lauf an. Das kommt als Nächstes.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Ob eine echte Linux-Leiste das Tray-Symbol zeigt, ist nicht ausprobiert.
  Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
* `ZEIT_PARSE` rechnet die Ortszeit nur unter Windows um; unter macOS und
  Linux gilt UTC.
