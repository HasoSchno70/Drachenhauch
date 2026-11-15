# Drachenhauch 2026.49

*Die Notizen zu dieser Fassung. Sie ist die Fassung **der lebendigen
Oberfläche**: Knöpfe in eigener Form, Häute aus Bildern und animierten GIFs,
Animationen, Klänge und Funken aus der Stildatei, eine Befehlspalette in
jedem Programm, ein Inspektor, Glas, Stildateien nach Uhrzeit und Datum,
Fenster von rechts nach links und selbst gezeichnete Widgets. Dazu Popups
über den Rand echter Fenster, Tabellen ohne Speichergrenze und volle
Textformung. Was bestehende Programme bemerken können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Strg+Umschalt+P öffnet in jedem Programm mit Menü- oder Werkzeugleiste
  die Befehlspalette.** Hat das Programm selbst ein Menükürzel auf dieser
  Taste, geht seines vor; `GUI_PALETTE_KEY(FALSE)` schaltet die Taste ab.
  Solange die Palette offen ist, bekommt kein Textfeld darunter die Tasten.
* **Aus der IDE gestartete Programme** öffnen mit **Strg+Umschalt+I** den
  Inspektor (die IDE setzt `DHRT_INSPEKTOR=1`). Ein Programm, das selbst
  gestartet wird, bemerkt davon nichts.
* **Ein Klick in ein rechtsbündiges Textfeld** (`GUI_SET_ALIGN`) setzt die
  Schreibmarke jetzt an die Klickstelle; vorher landete sie am Ende. Der
  Platzhalter eines solchen Feldes läuft nicht mehr über den Rand.
* **Die Vervollständigung der IDE** schlägt hinter `AS` und `OF` nur noch
  Typen vor -- `DIM w AS GUI_WIN` nahm vorher den ersten passenden Befehl.

## Neu in dieser Fassung

### Aussehen und Bewegung

```basic
IMPORT "gui"
SCREEN(640, 360, "Notizbuch")
DIM w AS GUI_WINDOW : w = GUI_WINDOW("Notizbuch", 10, 10, 600, 320)
DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(w, "Speichern", 20, 20, 160, 44)
GUI_SET_NAME(ok, "speichern")
GUI_STYLESHEET("button { klang: klick }" + CHR$(10) + _
    "button:hover { animation: puls 700ms }" + CHR$(10) + _
    "#speichern { form: pille; effekt: funken }")
```

* **Eigene Formen:** in der Stildatei `form: pille | kreis | raute | bild`,
  im Code `GUI_SET_SHAPE`. Die Form gilt auch für Treffer -- ein Klick
  neben den Kreis trifft nicht.
* **Häute aus Bildern:** `bild` und `bildrand` (9-Slice) je Zustand, auch für
  Kästchen, Optionen, Schalter und den Griff eines Reglers; animierte GIFs
  und Bildfolgen aus einem Atlas (`bild: figur.png 8 12fps`) laufen mit.
* **Animationen:** `puls`, `wackeln`, `schweben`, `huepfen`, `drehen`,
  `blinken` und eigene Abläufe (`@ablauf`) mit Lage, Größe, Deckkraft und
  Farbton -- in der Stildatei oder mit `GUI_ANIMATE`. Weiche
  Layout-Übergänge mit `gleiten: 200`.
* **Klänge und Teilchen:** `klang: klick | tick | ping | plopp | fehler |
  erfolg | datei.wav`, `effekt: funken | konfetti | sterne`, im Code
  `GUI_SOUND` und `GUI_BURST`. `GUI_SET_SOUNDS` gibt einen Schalter „Töne“.
* **„Animationen reduzieren“ des Systems** (Windows, macOS, GNOME) hält
  Animationen, GIF-Häute und Teilchen an; `GUI_SET_MOTION` überstimmt.
* **Glas:** `unschaerfe: 12` zeigt den Hintergrund verschwommen durch.
* **Stildatei nach Uhrzeit und Datum:** `@zeit 20:00-06:00 button { … }`,
  `@datum 12-01..12-26 #titel { … }` -- die gui löst neu auf, sobald eine
  Bedingung umschlägt.
* Neue Beispiele: `examples/220_formen_und_animationen.dh` und
  `examples/221_klang_funken_palette.dh`.

### Bedienen und Bauen

* **Befehlspalette** (Strg+Umschalt+P): Suche über alle Einträge von Menüs,
  Werkzeugleisten und Kontextmenüs; Enter löst aus wie Maus und Kürzel.
  Ohne eine Zeile Code -- `GUI_PALETTE` & Co. für den Rest.
* **Inspektor** (`GUI_INSPECTOR`): das Widget unter dem Zeiger, die
  passenden Regeln der Stildatei mit Zeile und was je Zustand gilt.
  `GUI_STYLESHEET_LOAD(pfad$, TRUE)` lädt die Stildatei beim Speichern neu.
* **Selbst gezeichnete Widgets:** `GUI_CUSTOM(win, x, y, w, h [, rolle$])`
  wird gemalt wie eine Zeichenfläche und hat trotzdem Tab-Fokus, Klick,
  Tasten (`GUI_CUSTOM_KEY`, `GUI_CUSTOM_TEXT$`) und eine Rolle für den
  Bildschirmleser.
* **Von rechts nach links:** `GUI_WINDOW_RTL` spiegelt ein Fenster -- Lagen,
  Menü- und Werkzeugleiste, Kennzeichen von Kästchen und Schaltern, ein
  Regler wächst nach links, ein Textfeld steht rechtsbündig.
* **Bedienung als Prüffall:** `dhrt aufnehmen programm.dh ziel.dhtest`
  nimmt eine Bedienung auf und hängt sie als fertigen Fall an.

### Echte Fenster, Tabellen, Text

* **Popups über den Rand:** Kontextmenü, Klappliste, Vorschläge, Monat und
  Rechtschreib-Menü eines echten Fensters bekommen ein eigenes Fenster,
  wenn sie über seinen Rand ragen.
* **Zeichenflächen in echten Fenstern** und abgerissenen Leisten:
  `GUI_CANVAS_BEGIN`/`GUI_CANVAS_END`, `GUI_CANVAS_MOUSE`. Palette,
  Inspektor, Glas, Deckkraft und Farbton gehen dort ebenso.
* **Virtuelle Tabellen:** `GUI_TABLE_VIRTUAL(tbl, zeilen)` -- die Tabelle
  fragt nach (`GUI_TABLE_NEED_FROM`/`_COUNT`) und hält nur, was rund um den
  sichtbaren Bereich geliefert wurde (`GUI_TABLE_SUPPLY`).
* **Volle Textformung:** indische und südostasiatische Schriften,
  Ligaturen (`FONT_LIGATURES`); die Schreibmarke springt über ganze
  Silben, nachgebildetes Kursiv neigt die geformten Zeichen.
* **Nach außen:** Bilder und Dateilisten über die Zwischenablage,
  `DRAG_OUT_FILES`/`DRAG_OUT_TEXT` ziehen aus dem Programm hinaus
  (Windows).
* **Studio:** Dialoge können eigene Fenster sein (Menü Ansicht).

## Unter der Haube

**Zahlen.** 2507 Befehle (vorher 2462) in 57 Modulen, 236 Beispiele auf der
obersten Ebene von `examples/` (234), **6004 Fälle in 503
Prüfsammlungen** (5919 in 483) und 701 Rust-Testfunktionen (682).

* **Profil `gegenprobe`:** `python rust/build_runtime.py --gegenprobe` baut
  ohne LTO -- ein Neubau für eine Gegenprobe dauert rund 30 Sekunden statt
  vier Minuten.
* **Eine Quelle je Lage:** die Textlage eines Eingabefeldes, die Form eines
  Knopfes und die Lage eines Popups fragen Zeichnen, Treffer und Tastatur
  an derselben Stelle.
* **Echte Fenster unter `dhrt test`** lassen die echte Maus durch -- Fälle
  hängen nicht mehr davon ab, wo der Zeiger gerade steht.
* **Dateidialoge:** rfd 0.18; unter Linux weiter das xdg-Portal mit zenity.

## Was offen bleibt

* **Von rechts nach links:** Tabellen, Bäume und Textbereiche bleiben innen,
  wie sie sind (Spalten, Einrückung und Text von links).
* **„Animationen reduzieren“** ist nur unter Windows am echten System
  geprüft.
* **macOS:** echte Fenster bleiben dort beim Rückfall, bis sie auf einem Mac
  geprüft sind; Ziehen hinaus gibt es nur unter Windows.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
