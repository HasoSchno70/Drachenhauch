# Drachenhauch 2026.50

*Die Notizen zu dieser Fassung. Sie ist die Fassung **der sparsamen und
zugänglichen Oberfläche**: Programme schlafen, solange nichts passiert (die
IDE braucht im Leerlauf ein Zwölftel der Rechenzeit), die Stildatei stellt
sich auf Fenstergröße, Thema und hohen Kontrast ein, eine eingebaute
Prüfung findet, was eine Oberfläche schwer bedienbar macht, Fenster von
rechts nach links sind jetzt auch innen gespiegelt, und Emoji kommen in
Farbe. Was bestehende Programme bemerken können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Farbige Knöpfe haben die lesbarere Schrift.** Auf einem Knopf mit
  eigener Grundfarbe (Primär, Erfolg, Warnung) wählt die gui die Schrift
  jetzt nach dem höheren Kontrast: auf dem Cyan des dunklen Themas, auf
  Grün und Orange steht dunkle Schrift statt weißer (vorher 2,1:1 bis
  3,4:1). Der Akzent von `modern_light` ist etwas dunkler (`#1D6DCE`).
* **Emoji kommen in Farbe.** Die Textfarbe gilt für sie nicht mehr, nur
  ihre Deckkraft.
* **Gespiegelte Fenster (`GUI_WINDOW_RTL`)** spiegeln jetzt auch Textbereiche,
  Tabellen und Bäume innen, und deren Rollbalken stehen links.
* **`WINDOW_WAIT_EVENTS`** wartet je Bild genau so lange, wie nichts
  ansteht: nach einer Eingabe läuft es 0,35 s ohne Warten nach, mit fester
  Bildzahl (`DHRT_FRAMES`) wartet es nie.
* **Schreibmarke und Hilfetext** rechnen in Sekunden statt in Bildern --
  bei 60 Bildern je Sekunde wie bisher.

## Neu in dieser Fassung

### Schlafen, solange nichts passiert

```basic
IMPORT "gui"
SCREEN(420, 200, "Ruhig", 1)
DIM w AS GUI_WINDOW : w = GUI_WINDOW("Ruhig", 10, 10, 400, 180)
DIM name AS GUI_WIDGET : name = GUI_TEXTINPUT(w, 20, 20, 240, 28, "Name")
WINDOW_WAIT_EVENTS(1)
WHILE NOT QUITREQUESTED()
    GUI_UPDATE()
    CLS(&H202830)
    GUI_DRAW()
    FLIP()
WEND
```

* Mit `WINDOW_WAIT_EVENTS(sekunden)` schläft `FLIP` bis zur nächsten
  Eingabe -- und die gui weckt das Programm selbst, wenn sie ein Bild
  braucht: zum Umschlagen der Schreibmarke, für einen fälligen Hilfetext,
  einen Toast, eine Animation. `GUI_NEXT_FRAME()` und `WINDOW_IDLE_WAIT()`
  sagen, wie es steht.
* **IDE und Studio** schlafen so, solange kein Programm läuft: gemessen
  braucht die IDE im Leerlauf 2,2 % eines Kerns statt 27,5 %.
* **Programme aus dem Studio** schlafen mit `Self.Leerlauf = TRUE` in
  `App.Geoeffnet`; Timer wecken sie rechtzeitig.

### Stildatei nach der Lage

* Vor einer Regel stehen jetzt auch `@breite < 600`, `@hoehe >= 500`,
  `@dunkel` / `@hell`, `@hochkontrast` und `@rtl` / `@ltr` -- je Fenster
  ausgewertet, kombinierbar mit `@zeit` und `@datum`. Wird ein Fenster
  schmal oder das Thema dunkel, löst die gui neu auf.
* Hoher Kontrast kommt vom System (Windows, macOS, GNOME);
  `GUI_SET_CONTRAST` überstimmt, `GUI_HIGH_CONTRAST()` und
  `GUI_SYSTEM_HIGH_CONTRAST()` fragen.

### Barrierefreiheit prüfen

* **`GUI_A11Y_CHECK$([win])`** meldet je Zeile, was eine Oberfläche schwer
  bedienbar macht: zu wenig Kontrast der Schrift (WCAG 4,5:1), ein Widget
  ohne Namen für den Bildschirmleser, zu kleine Ziele (WCAG 2.5.8, mit der
  Ausnahme für Abstand) und eine Tab-Reihenfolge, die gegen die Lage
  springt. Der Inspektor nennt die Befunde zum gewählten Widget.
* Ein Knopf nur mit Sinnbild heißt für den Bildschirmleser jetzt wie sein
  Hilfetext.

### Rechts nach links, auch innen

* Im gespiegelten Fenster stehen Textbereiche rechtsbündig (lange Zeilen
  zeigen ihr Ende rechts), Tabellen beginnen mit der ersten Spalte rechts,
  Bäume rücken von rechts ein; Klick, Auswahl, Umsortieren, Spaltenbreite
  und Pfeiltasten folgen. `GUI_SET_ALIGN` gilt jetzt auch für Textbereiche,
  `GUI_TABLE_GET_COL_WIDTHS` fragt die Spaltenbreiten ab.

### Emoji in Farbe

* In `TEXT` wie in der gui: Die Runtime malt die Farbglyphen der
  Emoji-Schrift selbst (COLR bei Segoe UI Emoji, Bitmaps bei Apple Color
  Emoji und Noto Color Emoji). Folgen mit Verbinder und Hauttöne setzt die
  Formung zu einem Zeichen zusammen.

## Unter der Haube

**Zahlen.** 2514 Befehle (vorher 2507) in 57 Modulen, 236 Beispiele auf der
obersten Ebene von `examples/`, **6039 Fälle in 508 Prüfsammlungen**
(6004 in 503) und 709 Rust-Testfunktionen (701).

* **Eine Quelle je Lage:** die gui meldet ihren Bildbedarf an der Stelle,
  die nach der Uhr zeichnet; die Zeilen eines Textbereichs richten sich an
  einer Stelle aus (`ta_versatz`), die Spalten einer Tabelle spiegeln in
  `col_x`.
* **Funde der eigenen Prüfung:** acht Kontrastfehler in den eingebauten
  Themen, behoben; alle sechs Themen bestehen `GUI_A11Y_CHECK$`.

## Was offen bleibt

* **Listen** spiegeln ihr Inneres im gespiegelten Fenster noch nicht.
* **Flaggen-Emoji** fehlen in Segoe UI Emoji -- unter Windows stehen dort
  zwei Buchstaben. Farb-Emoji unter macOS und Linux sind gebaut, aber nur
  unter Windows angesehen.
* **Hoher Kontrast** ist nur unter Windows gegen das echte System geprüft.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
