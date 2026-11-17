# Drachenhauch 2026.51

*Die Notizen zu dieser Fassung. Sie ist die Fassung **der Namen und der
Ruhe**: Befehle bekommen eine Leitlinie und für die alten Namen neue, die man
vorhersagen kann, eigene Namen müssen so geschrieben werden, wie sie angelegt
sind, die IDE wird ruhiger und bekommt den Drachenhauch-Nebel, Flächen
bekommen Farbverläufe, und Circuit Runner wird schöner und gemächlicher. Was
bestehende Programme bemerken können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Neue Warnung: anders geschrieben als angelegt.** Wer `CONST CW` anlegt
  und später `cw` schreibt, bekommt beim Übersetzen eine Warnung (bei
  `dhrt run` auf stderr, im Editor die gelbe Linie). Das Programm läuft
  unverändert -- Groß/klein zählt weiter nicht. Strg+. oder
  `dhrt fmt --schreibweise` schreibt die Namen so wie angelegt.
* **Ein Label misst seine Breite am Text**, bis `GUI_SET_BOUNDS` oder
  `GUI_SET_WRAP` eine setzt; `GUI_GET_W` liefert die echte Breite.
* **Die Schrift eines Knopfs mit Farbverlauf** richtet sich nach der
  ungünstigsten Stelle des Verlaufs, `GUI_A11Y_CHECK$` misst dort.
* **Listen im gespiegelten Fenster** (`GUI_WINDOW_RTL`) spiegeln jetzt auch
  ihr Inneres.
* **Circuit Runner:** die Roboter ziehen halb so oft wie bisher (Taste T im
  Menü schaltet aufs Original); aufgezeichnete Lösungen passen nur noch im
  Originaltempo.

## Neu in dieser Fassung

### Befehlsnamen, die man vorhersagen kann

```basic
IMPORT "time"
PRINT TIME_TEXT$(TIME_NOW())
PRINT FILE_EXISTS("notizen.txt")
PRINT ENV_GET("HOME", "?")
```

* **Eine Leitlinie** ([Befehlsnamen](../builtins-namen.md)): mit Unterstrich
  gegliedert, vorn der Bereich (`IMAGE_LOAD`, `WINDOW_SET_TITLE`), `_DOWN`
  heißt gehalten, `_HIT` gerade gedrückt, `_FREE` gibt frei, `_COUNT` ist
  eine Anzahl, Befehle sind englisch.
* **143 neue Namen für alte Befehle**, etwa `KEY_DOWN` statt `KEYPRESSED`
  (das „gehalten“ hieß, während `INPUT_PRESSED` „gerade gedrückt“ hieß),
  `GUI_LISTBOX_ITEMS` statt `GUI_SET_LISTBOX`, `MAP_GET`, `SCREEN_WIDTH`,
  `GUI_GET_TEXT$`, `TIME_NOW`, `MONEY_ROUND`. **Die alten bleiben gültig**;
  der Hover nennt beim alten den neuen.
* **Das `$` ist bei Text-Befehlen frei:** `ENV_GET` ist `ENV_GET$`,
  `CLIPBOARD_GET$` ist `CLIPBOARD_GET`.
* `IMPORT "time"` und `IMPORT "money"` sind die Module `zeit` und `geld`.

### Groß/klein egal, aber einheitlich

```basic
CONST CW = 4
PRINT cw                       ' Warnung: 'cw' ist als 'CW' angelegt ...
```

* Welche Deklaration gemeint ist, entscheidet der Bereich: eine lokale `cw`
  in einer SUB ist dort richtig geschrieben, auch neben einer globalen `CW`.
  Befehle und Schlüsselwörter schreibt jeder, wie er will.
* Im ganzen Repo (598 Dateien) meldet die Prüfung nichts.

### Farbverläufe und Drachenhauch-Nebel

```basic
IMPORT "gui"
GUI_STYLESHEET("button.los { farbverlauf: #2FA8D8 #7B4FD8 30 wandert 6s }" + CHR$(10) + "panel { nebel: 14% }")
```

* **`farbverlauf`** in der Stildatei: 2 bis 6 Farben, ein Winkel, auf Wunsch
  wandernd ohne Naht -- für Knöpfe, Panels, Karten und jede andere Fläche.
* **`nebel`**: weiche Schwaden ziehen über eine Fläche, für den Fenstergrund
  die Metrik `nebel`, die Farbe die Themenfarbe `nebel`. Im Leerlauf steht
  er, mit `GUI_SET_MOTION("aus")` ganz.
* **Hintergrundfenster:** `GUI_WINDOW_BACKGROUND(win, TRUE)` hält ein
  Fenster hinten -- ein angeklicktes randloses Hauptfenster verdeckt die
  anderen nicht mehr.
* **Reiter am Editor:** Themenfarbe `reiter` und Metrik `reiter_oben`; die
  Statusleiste nimmt einen gesetzten Grund an.

### Die IDE

* **Ruhiger Look:** dunkle Ränder bündig unter dem Editor, gedämpfte Ordner,
  der aktive Reiter in der Farbe des Editors, und über allem der
  Drachenhauch-Nebel (abschaltbar, Einstellung `nebel`).
* Leere Ausgabe und Problemliste sagen, was zu tun ist; die Statusleiste
  färbt sich, solange ein Programm läuft.
* Dialogtitel übersetzt, die englische Sprachtafel ohne Doppelte und Tote.

### Beispiel 222 und Circuit Runner

* **Beispiel 222** zeigt die gui in einem Rundgang.
* **Circuit Runner:** Monster und Blöcke sind Klassen mit Vererbung; die
  Kacheln haben 128 Punkte statt 64, das Spielfeld füllt die Schirmhöhe
  stufenlos, dazu Schatten, Leuchten, Funken und Glas-Karten.

## Unter der Haube

**Zahlen.** 2660 Befehle (vorher 2514, davon 143 neue Namen für alte) in 57
Modulen, 237 Beispiele auf der obersten Ebene von `examples/`, **6085 Fälle
in 516 Prüfsammlungen** (6039 in 508) und 728 Rust-Testfunktionen (709).

* **Neue Namen ohne neue Laufzeit:** der Übersetzer bildet einen neuen Namen
  auf den bisherigen ab (`namen.rs`), Laufzeit und Maschinencode bleiben
  unberührt.
* **Ein Fund unterwegs:** der Scanner für Editor und Umbenennen hielt in
  `!"…\"…"` das `\"` für das Ende der Zeichenkette.
* Ein Feld-Literal aus GUI-Handles (`[a, b]`) passt an `ARRAY OF GUI_WIDGET`.

## Was offen bleibt

* **Ein englischer Name für den Typ `GELD`** fehlt bewusst: er verdeckte
  eine eigene Klasse `Money`.
* **Benannte Argumente** gibt es noch nicht für eingebaute Befehle.
* **Flaggen-Emoji** fehlen in Segoe UI Emoji -- unter Windows stehen dort
  zwei Buchstaben.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
