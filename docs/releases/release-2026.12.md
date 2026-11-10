# Drachenhauch 2026.12

*Die Notizen zu dieser Fassung. Acht Tage und 89 Commits seit 2026.11, und
fast alle hängen an einer Frage: **kann man damit eine Anwendung schreiben,
nicht nur ein Spiel?** Die Antwort wurde nicht geschätzt, sondern gebaut —
sechs Programme lang.*

## Neu in dieser Fassung

### Sechs Programme, in Drachenhauch geschrieben

Die Frage, ob die Qt-Editoren des Projekts auch in Drachenhauch selbst gehen,
war lange eine Schätzung. Jetzt ist sie an fünf Fällen gemessen — jeder ein
Begleitwerkzeug, das es als Python/Qt-Programm schon gab, noch einmal in
Drachenhauch: der **SFX-Generator**, der **Partikel-Editor**, der
**Tilemap-Editor**, der **Sprite-Editor** und der **Tracker** (Beispiele 183,
185, 187, 189 und 190). Alle fünf lesen zurück, was sie schreiben, alle fünf
haben Rückgängig, und alle fünf sind aus der IDE heraus zu starten (`Datei` →
`Werkzeuge in Drachenhauch`).

Das sechste Programm ist kein Werkzeug, sondern eine **Geschäftsanwendung**:
Kunden, Artikel, Rechnungen mit Positionen, SQLite als Wahrheit, PDF und CSV,
Menüs mit Kürzeln, Formulare mit Prüfung, Rückfrage bei ungesicherten
Änderungen, ein Druckdialog (Beispiel 196, rund 1200 Zeilen). Es sollte
zeigen, was einer *Anwendung* nach dem gui-Ausbau noch fehlt.

**Die Lehre aus den Zeilenzahlen:** die Drachenhauch-Fassung ist zwischen
0,38 und 1,18 mal so lang wie die Qt-Fassung, und kein Faktor davon taugt zum
Hochrechnen. Er misst vor allem, wie viel man weglässt — beim Sprite-Editor
wuchs er von 0,14 auf 0,38, ohne dass sich an der Sprache etwas geändert
hätte, nur weil weniger fehlte. Und beim SFX-Generator liegt er über 1, weil
die Qt-Zahl ein gemeinsames Undo-Modul nicht mitzählt, das der Pilot selbst
trägt.

**Was die Piloten gefunden haben, hätte kein Spiel gefunden.** Jeder deckte
Modul-Lücken auf, die in fünf Jahren Spieleschreiben nie aufgefallen wären:

* Es gab keinen **Farbwähler** und keinen **Datumswähler** —
  `GUI_COLORPICKER` und `GUI_DATEPICKER` sind neu, mit HSV im Inneren, weil
  bei Schwarz der Farbton sonst verloren geht.
* Das `tiled`-Modul konnte Karten lesen und ändern, aber **nicht anlegen und
  nicht speichern** — jetzt gehen `TILED_NEW`, `TILED_SAVE`, Ebenen und
  Tilesets anlegen, Objekte und Eigenschaften setzen, Ebenen umsortieren
  (`TILED_MOVE_LAYER`). Geprüft am Leser des Qt-Editors, nicht am eigenen.
* Ein **Bild ließ sich nicht herstellen**, nur anzeigen — `IMAGE_NEW`,
  `IMAGE_CLEAR`, `IMAGE_DRAW_IMAGE`, `IMAGE_SAVE`, dazu `IMAGE_FREE` (1200
  Kopien zu 256×256: 393 MB gegen 91 MB) und **bewegte GIFs** mit einer Dauer
  je Bild (`IMAGE_SAVE_GIF`). Und `IMAGE_ROTATE` verliert bei 90 Grad die
  Punkte, weil es trigonometrisch neu abtastet — `IMAGE_ROTATE_CW` und
  `IMAGE_ROTATE_CCW` sortieren nur um.
* Eine gehaltene **Note mit echter Hüllkurve** (`AUDIO_NOTE`), und **Klänge
  zu einem mischen** (`AUDIO_SOUND_NEW`, `AUDIO_SOUND_MIX`,
  `AUDIO_SOUND_NORMALIZE`) — sonst ließ sich ein Song nicht als WAV
  abliefern. Dazu `JSON_APPEND_NULL`, weil eine Liste mit leeren Plätzen
  nicht zu schreiben war.
* Ein **gui-Fenster über einer Zeichenfläche war unsichtbar**: es sperrte die
  Eingabe, das Programm wirkte eingefroren. Dafür `GUI_DRAW_WINDOW` und
  `GUI_DRAW_TOP`.
* `KEYHIT(ASC("S"))` traf **still gar nichts** — nur Kleinbuchstaben
  zählten. Daran waren alle Tastenkürzel des Tilemap-Editors tot, ohne
  Fehler oder Warnung. Jetzt meinen beide Schreibweisen dieselbe Taste.

### Der gui-Ausbau: sechs Punkte und zwei Nachzüge

Das Ziel war ausgesprochen: dass Drachenhauch genannt wird, wenn jemand fragt,
womit er eine Anwendung schreiben soll. Sechs Punkte, jeder ein eigener Pull
Request mit Beispiel, Tests, Doku, Lehrbuch und Form-Designer:

1. **Text und Formular** — Ausrichtung, Zeilenumbruch in Beschriftungen
   (`GUI_SET_WRAP`), Passwortfeld, nur lesen, Höchstlänge, Zahlenfilter
   (`GUI_TEXTINPUT_SET`), Enter als Abschicken, Strg+Z/Y im Textfeld,
   Standard- und Abbrechen-Knopf je Fenster.
2. **Menüs** — Tastenkürzel als Text (`GUI_MENU_ITEM(m, "Speichern",
   "Strg+S")`), Untermenüs beliebig tief (`GUI_SUBMENU`), Häkchen, Sperren,
   Sinnbilder.
3. **Listen** — Einträge einzeln (`GUI_LISTBOX_ADD` und Geschwister),
   Kästchen, Sinnbilder und Farben je Eintrag, Mehrfachauswahl mit Strg und
   Umschalt, `GUI_DOUBLE_CLICKED`.
4. **Dialoge** — eigene Knopfsätze (`GUI_DIALOG(t, x,
   "Speichern|Verwerfen|Abbrechen")`), `GUI_PROMPT` mit Textfeld, ein
   eigenes Fenster modal schalten (`GUI_WINDOW_MODAL`).
5. **Layout** — Größe nach Inhalt (`GUI_AUTOSIZE`, oder 0 als Maß) und
   Behälter für Zeile, Spalte und Raster (`GUI_LAYOUT`), mit Gewichten und
   Platzhaltern. Ein Behälter ist Luft für Klicks.
6. **Feinschliff** — senkrechter Schieber (`GUI_VSLIDER`), unbestimmter
   Fortschritt, Bildmodi, Baumsymbole, ein **rollendes Panel**
   (`GUI_PANEL_ADD`), **Ziehen und Ablegen** (`GUI_DRAGGABLE`), Cursorformen.

Der sechste Pilot lieferte danach die Nachzüge: **Mindestmaße**
(`GUI_SET_MIN_SIZE`, `GUI_LAYOUT_MIN_W`) und eine **Formularprüfung** als
Baustein (`GUI_RULE`, `GUI_VALIDATE`, `GUI_ERROR_LABEL` — pflicht, zahl,
bereich, länge, email, datum, muster; ein Fehler auf einem anderen Reiter
wird nicht gemeldet, weil er einer ohne Ausweg wäre), **Zeilenumbruch im
Textbereich** und **Datenbindung**: `GUI_BIND` hängt ein Feld an einen
Schlüssel, `GUI_FORM_LOAD` und `GUI_FORM_SAVE` schreiben ein ganzes Formular
aus einer und in eine SQLite-Zeile, `GUI_FORM_CHANGED` sagt, ob etwas
ungesichert ist.

Davor schon: **Bedienung ohne Maus** (Tab durch alle Widgets, Fokusring),
`GUI_SCALE` für den 4K-Schirm, `GUI_DIALOG` als Kasten im eigenen Thema, und
das **Textfeld als Code-Feld** mit Syntax-Einfärbung (`SYNTAX_SPANS`,
`GUI_TEXTAREA_VIEW` — 30 000 Zeilen kosten je Anschlag 2 ms statt 272).

### Zwei Entscheidungen mit Untersuchung

**Ein zweites Fenster ist ein zweiter Prozess.** Drachenhauch bleibt bei
einem Betriebssystem-Fenster je Programm — raylib kennt kein zweites. Statt
eines Forks startet `WINDOW_OPEN(datei$)` einen zweiten `dhrt` mit eigenem
Schirm; die beiden reden über einen Textkanal (`WINDOW_SEND`,
`WINDOW_RECV$`, im Kind `PARENT_SEND`, `PARENT_RECV$`). Kein geteilter
Zustand, eine Nachricht ist eine Zeile, stirbt der Elternprozess, endet das
Kind. Gemessen: 0,4 s bis zum ersten Bild, eine Runde ein bis drei Bilder des
Kindes. Die Untersuchung davor steht in `entwurf-native-fenster.md`.

**Drucken ohne raylib.** Das pdf-Modul zeichnet seine Befehle jetzt auf und
spielt sie auf drei Ziele: die PDF wie bisher, den Drucker (`PDF_PRINT`,
unter Windows über GDI, sonst über `lp`) und eine Vorschau als Bild
(`PDF_PREVIEW`). Dazu `PRINTERS`, `PRINTER_DEFAULT$` und `OPENDOC` — das
Gegenstück zu `OPENURL` für eine Datei auf der Platte. Der Test schickt zwei
Seiten durch einen echten Treiber („Microsoft Print to PDF") und liest sie
zurück. Die Untersuchung: `entwurf-drucken.md`.

### Die Werkzeugkette merkt mehr

* `dhrt --check` findet **Tippfehler in Variablennamen**, und zwar mit
  Gültigkeitsbereich: eine Variable, die es nur in einer anderen SUB gibt,
  wird als „an dieser Stelle nicht sichtbar" gemeldet statt als unbekannt.
  Der Beleg ist der Lauf über alle 384 Programme im Repo — null Meldungen —
  und der Lauf ist seither ein Test.
* Eine **Variable darf heißen wie ein Builtin**, und der Aufruf meint den
  Builtin: `deg = DEG(w)` brach zur Laufzeit ab, und `--check` schwieg.
* `DIM red` in einem Block ging nicht — nur auf oberster Ebene. Jetzt
  bekommt ein `DIM` im `IF` denselben Platz.
* Der Compiler **warnt, wenn ein Modul-Builtin ohne seinen IMPORT** benutzt
  wird; tiefe Rekursion meldet ihre Grenze statt abzustürzen.
* Die **Hover-Doku im Editor kommt aus `docs/`** — aus einer zweiten
  Handtabelle waren 54 Prozent der Befehle beschrieben, jetzt sind es 100.
  Der Highlighter kannte 72 von 1558 Befehlen; jetzt alle, mit
  Drift-Schutz.
* Der **Form-Designer kennt alle 24 Widget-Arten** der Laufzeit. Es fehlten
  neun, und das war nirgends gemessen; jetzt prüft ein Test die Palette
  gegen den Quelltext der Laufzeit.
* Beide **Bücher sind gegengelesen**: 134 Verweise ins Leere und sechs
  falsche Zahlen im Einstieg, 34 Funde im Lehrbuch, und jeder der Befehle
  hat einen Referenzeintrag. Die Prüfwerkzeuge dafür laufen jetzt in der
  Suite, nicht mehr von Hand.

## Unter der Haube

**Kein Push mehr direkt auf `main`.** Jede Änderung geht über einen Zweig
und einen Pull Request, und `main` verlangt sechs Prüfungen — Python-Tests,
POSIX-Tests und den Rust-Bau je auf Ubuntu, macOS und Windows. Der Anlass war
ein Linker-Flag, das Windows und Linux stillschweigend schluckten und nur
ld64 auf dem Mac ablehnte: ein roter Bau, der auf `main` stand, bevor die CI
ihn sah. Im Pull Request hätte die Prüfung ihn abgefangen. Alle 89 Commits
dieser Fassung sind so gelaufen.

**Was nur das Bild zeigt.** Auffallend oft in dieser Fassung: der Test war
grün und wertlos, und erst das gerenderte Bild zeigte den Fehler — verdeckte
Knöpfe, überlappende Kästchen, ein unsichtbares Fenster, ein Text mitten auf
einem Knopf. Mehrere Tests prüfen deshalb jetzt die *Rechtecke* aller
Bedienelemente paarweise gegeneinander und gegen den Fensterrand, statt einen
Klick zu simulieren, der auch beim falschen Stand trifft.

**Zahlen.** Die Befehlsreferenz ist von 1558 auf **1712** Einträge
gewachsen, die Modulliste bleibt bei 47. Die Beispiele von 196 auf **212**.
Die Testsuite zählt **4428** gesammelte Prüfungen, 2026.11 waren es mit
demselben Befehl 4359 — die 3803 der letzten Notizen zählten anders, die
beiden Zahlen hier sind gleich gemessen. Dazu Rust-Tests für
alles, was sich ohne Fenster rechnen lässt (Farbraum, Kalender, Kürzel,
Regeln, Hüllkurven, GIF-Kodierer, Drucker-Schriftabbildung).

## Was offen bleibt

* Der Druckpfad über `lp` und `lpstat` ist gegen die Handbuchseiten
  geschrieben, nicht gegen einen echten CUPS — nur Windows war hier zu
  prüfen. Und Windows kann „den zuletzt benutzten Drucker zum Standard
  machen"; wer aus einem Programm auf *Print to PDF* druckt, sieht das
  danach in den Einstellungen.
* Zwei Architekturpunkte stehen noch als eigene Entscheidungen aus:
  **Barrierefreiheit** (ein Bildschirmleser sieht in ein raylib-Fenster
  nicht hinein) und **Eingabemethoden** für Schriften, die man nicht Taste
  für Taste tippt.
* Die Piloten sind Werkzeuge, keine Ablösung: die Qt-Editoren bleiben die
  vollständigeren Programme. Was in Drachenhauch fehlt, steht bei jedem
  Piloten im Kopfkommentar.
