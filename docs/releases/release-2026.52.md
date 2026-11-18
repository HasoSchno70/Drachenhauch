# Drachenhauch 2026.52

*Die Notizen zu dieser Fassung. Sie ist die Fassung **der offenen Tür**:
Wer Drachenhauch zum ersten Mal sieht, ist in fünf Minuten bei einem eigenen
Programm, Befehle nehmen benannte Argumente, dhrt spricht die Sprache des
Systems, ein exportiertes „Hallo Welt“ ist 16 statt 45 MB groß, und eine
Seite sagt, was von Fassung zu Fassung fest bleibt. Was bestehende Programme
bemerken können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Meldungen in der Sprache des Systems.** Ohne `DHRT_LANG` schreibt dhrt
  seine Meldungen auf einem deutschen System deutsch, auf jedem anderen
  englisch (vorher immer deutsch). Was ein Programm in einer
  `CATCH`-Variable sieht, bleibt deutsch. Wer Ausgaben vergleicht, setzt
  `DHRT_LANG=de`.
* **IDE und Studio folgen ohne Einstellung ebenfalls dem System.** Auf einem
  englischen Windows starten sie englisch; in der IDE stellt Ansicht →
  „Sprache: Deutsch“ beim nächsten Start um, das Studio folgt der Einstellung
  der IDE.
* **`dhrt --export` nimmt von selbst die kleinste passende Laufzeit**
  (vorher nur mit `--schlank`, das weiter geht). `--voll` nimmt die ganze --
  etwa für ein Programm, das mit `IMPORT … OPTIONAL` und `MODULE_AVAILABLE`
  zur Laufzeit nachsieht, was der Bau kann.
* **Eine `.dhc` aus einer anderen Fassung wird abgewiesen** („Diese .dhc
  stammt aus dhrt …“), statt still anders zu laufen. Eine ohne Fassung
  (vor 2026.52 übersetzt) läuft wie bisher.
* **`DIRLIST(pfad, "")`** liefert alle Einträge (vorher keinen).
* **Die Signaturen nennen englische Parameternamen** (`color` statt
  `farbe`) -- im Hover, in der Vervollständigung und in den Handbüchern.
  Parameter mit Vorgabe stehen nicht mehr im Gerüst der Vervollständigung.

## Neu in dieser Fassung

### Benannte Argumente an Befehlen

```basic
SCREEN(320, 200, title: "Benannt")
CIRCLE(x: 160, y: 100, r: 40, color: RGB(255, 120, 40))
PRINT MID$("Drachenhauch", start: 0, n: 7)
```

* **Jeder Befehl** nimmt `name: wert`, die Namen stehen in seiner Signatur.
  Das `$` am Namen ist frei, ein Schlüsselwort darf Name sein (`new:`,
  `to:`), und eine ausgelassene Stelle davor bekommt ihre Vorgabe -- 91
  Vorgaben sind am Code nachgemessen. Wo es keine gibt, sagt die Meldung,
  was mit anzugeben ist.
* **Die Namen sind englisch und überall gleich** (Regel 8 in
  [Befehlsnamen](../builtins-namen.md)); `dhrt pruef parameter` hält die
  Tabellen des Handbuchs am Index.
* **Varianten zusammengefasst:** `DRAW_IMAGE` nimmt `w`, `h`, `angle`,
  `tint`, `flip_x`, `flip_y` und einen Ausschnitt (`sx, sy, sw, sh`),
  `DIR_LIST` nimmt `recursive`. Die alten Befehle bleiben.
* **`ELLIPSE_OUTLINE`, `TRIANGLE_OUTLINE`, `POLYGON_OUTLINE`** zeichnen mit
  `width` jetzt wirklich dicker (die Signatur nannte es, der Code las es
  nie); `IMAGE_DRAW_IMAGE` nimmt `tint:` allein (`qb`/`qh = -1` = bis zum
  Rand); `RANGE` heißt im ersten Parameter `start_or_stop`.

### Englisch gleichberechtigt

* Die Sprache folgt dem System (Windows: Sprache der Oberfläche, sonst
  `LANGUAGE`/`LC_ALL`/`LC_MESSAGES`/`LANG`, unter macOS ohne sie die
  Einstellung des Systems). Neu: `SYSTEM_LANGUAGE$()` und
  `RUNTIME_LANGUAGE$()`.
* IDE und Studio geben ihre Sprache an gestartete Programme weiter, die
  Spielwiese die Sprache der Seite.

### Kleine Programme

* Neue kleinste Laufzeit **`dhrt-mini`** (Kern und Maschinencode, 16 MB)
  neben `dhrt-konsole` (22 MB) und `dhrt-spiel` (35 MB); die volle hat 45 MB.
  Der Export nimmt die kleinste, der keiner der Befehle des Programms fehlt,
  und übergeht eine kleine Laufzeit einer anderen Fassung.

### Stabilität und Änderungsprotokoll

* [Stabilität](../stabilitaet.md) sagt, was fest bleibt (Sprache,
  Befehlsnamen ohne Ablaufdatum, Argumente, Parameternamen, Dateien,
  exportierte Programme, Kommandozeile) und was sich ändern darf.
* Das [Änderungsprotokoll](../aenderungen.md) sammelt aus allen Notizen den
  Abschnitt „Was sich für bestehende Programme ändert“ (`dhrt doku
  aenderungen`).

### In fünf Minuten zum eigenen Programm

* [Schnellstart](../schnellstart.md): Spielwiese, Installieren, erstes Spiel,
  erstes Fenster, Weitergeben -- mit Größen.
* **Drei Startprojekte in der Spielwiese** (Mini-Spiel, Formular, Malen),
  direkt aufrufbar mit `?beispiel=`, deutsch und englisch.
* Die Startseite der Website und das README sagen in einem Satz, was
  Drachenhauch ist, nennen vier Gründe und drei Wege hinein.
* **Physik in der Spielwiese** geht wieder (fehlte dem Web-Bau seit
  2026.21), und beim Malen bleibt das Bild stehen.

## Unter der Haube

**Zahlen.** 2662 Befehle (2660) in 57 Modulen, 237 Beispiele auf der
obersten Ebene von `examples/`, **6120 Fälle in 518 Prüfsammlungen** (6085
in 516) und 732 Rust-Testfunktionen (728).

* **Benannte Argumente kosten zur Laufzeit nichts:** der Übersetzer legt die
  Werte in die Reihenfolge der Signatur, danach sieht der Aufruf aus wie
  einer nach Stelle.
* **1030 Signaturen** bekamen englische Namen; wo die Doku den besseren hatte
  (`key`, `conn`, `handle`), wurde der Index nachgezogen. Ein Rust-Test hält
  deutsche Namen fern.
* **`dhrt test` spielt seinen Fällen ein deutsches System**
  (`DHRT_SYSTEM_LANG=de`), damit die Erwartungen auf englischen CI-Läufern
  halten.
* **Jede `.dhc` trägt die Fassung, die sie übersetzt hat** (`"dhrt"`).
* `dhrt fmt` lässt ein Schlüsselwort als Argumentname klein.

## Was offen bleibt

* **11 Parameter haben keine Vorgabe**, weil Weglassen dort etwas anderes
  heißt als jeder Wert (etwa die Lichtfarbe eines Fensters behalten); wer
  einen Parameter dahinter benennt, gibt sie mit an.
* **Ein englischer Name für den Typ `GELD`** fehlt bewusst: er verdeckte
  eine eigene Klasse `Money`.
* **Flaggen-Emoji** fehlen in Segoe UI Emoji -- unter Windows stehen dort
  zwei Buchstaben.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
