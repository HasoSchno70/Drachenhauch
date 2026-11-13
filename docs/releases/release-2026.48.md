# Drachenhauch 2026.48

*Die Notizen zu dieser Fassung. Sie ist die Fassung **der echten Fenster**:
ein gui-Fenster kann ein eigenes Fenster des Systems werden -- über den Rand
des Programms hinaus, auf den zweiten Schirm, als modaler Dialog über dem
Hauptfenster; Dock-Leisten reißen ab und docken wieder an. Dazu große
Tabellen, die auch mit einer Million Zeilen flüssig bleiben, elf Abfragen,
Rückgängig als Befehl und das Aussehen des Systems. Was bestehende Programme
bemerken können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Während eines modalen Dialogs gelten seine eigenen Menüs.** Bisher war
  während eines modalen Dialogs jedes Menü gesperrt -- auch das Kontextmenü
  und die Menüleiste im Dialog selbst. Jetzt gehen die des Dialogs, die der
  Fenster dahinter bleiben gesperrt.
* **`GUI_DOCK_WHERE$` kennt einen neuen Ort:** `fenster` für eine
  abgerissene Leiste. Ein Programm, das die Antwort mit `SELECT CASE`
  auswertet, sieht diesen Wert nur, wenn eine Leiste abgerissen wird.
* **Aufnahmen (`AUTOMATION_RECORD`)** schreiben für echte Fenster zusätzliche
  Zeilen `n ...` ans Ende der Datei. raylibs Leser übergeht sie; alte
  Aufnahmen spielen unverändert.

## Neu in dieser Fassung

### Echte Fenster

```basic
IMPORT "gui"
SCREEN(800, 600, "Editor")
DIM palette AS GUI_WINDOW : palette = GUI_WINDOW("Werkzeuge", 520, 10, 220, 300)
IF NOT GUI_WINDOW_NATIVE(palette, TRUE) THEN PRINT "bleibt im Hauptfenster"
```

* **`GUI_WINDOW_NATIVE(win, an)`** macht ein gui-Fenster zu einem eigenen
  Fenster des Systems -- mit Titelleiste und Kreuz des Systems, eigenem
  Eintrag in der Taskleiste, frei über den Rand und auf den zweiten Monitor.
  Widgets, Ereignisse, Stildatei und Daten bleiben dieselben.
  `GUI_WINDOW_IS_NATIVE` fragt es ab.
* **Modale Dialoge als echtes Fenster:** das Hauptfenster wird Besitzer (der
  Dialog bleibt darüber, ohne eigenen Taskleisten-Eintrag; unter Linux mit
  X11 über `WM_TRANSIENT_FOR`), das Hauptfenster ist abgedunkelt, Enter und
  ESC wirken im Dialog.
* **Kontextmenü, Hilfetext, Vorschläge, Monat, Zellwahl** und die
  Umwandlung einer Eingabemethode stehen im Fenster, zu dem sie gehören.
* **Schirme mit anderer Skalierung:** ein Fenster auf einem Schirm mit
  150 % neben einem mit 100 % wird dort entsprechend größer gezeichnet,
  Schrift scharf gesetzt; `GUI_WINDOW_NATIVE_SCALE` nennt den Faktor.
* **Dock-Leisten reißen ab:** über den Rand des Hauptfensters gezogen, wird
  eine Leiste ein eigenes Fenster; ihren Kopf über einen Bereich gezogen,
  dockt sie wieder an. Das gilt für beide Arten -- Leisten mit eigenem
  Fenster (`GUI_DOCK_WINDOW`) und Leisten aus Widgets des Gastfensters
  (`GUI_DOCK_ADD`, wie in IDE und Studio). `GUI_DOCK_MOVE(..., "fenster")`
  reißt vom Programm aus ab, `GUI_DOCK_STATE$` merkt sich die Lage.
* **Bildschirmleser** finden die Widgets im Fenster, in dem sie stehen, und
  bedienen sie dort; **Aufnahmen** nehmen die echten Fenster mit.
* **Wo es keine echten Fenster gibt** (Browser, macOS vorerst, ohne
  Bildschirm), bleibt alles im Hauptfenster -- ein Programm läuft überall
  gleich.
* Neues Beispiel: `examples/219_echte_fenster.dh` (Palette, modaler Dialog,
  abreißbare Leisten).

### IDE und Studio

* **IDE:** die Einstellungen (Strg+U) können ein eigenes Fenster sein --
  Schalter „Als eigenes Fenster“, aus per Vorgabe. Die Leisten der IDE
  (Gliederung, Ausgabe, Probleme ...) lassen sich herausziehen.
* **Studio:** die Berichtsvorschau hat einen Knopf „Als eigenes Fenster“ --
  auf den zweiten Schirm, während das Studio daneben frei bleibt.

### gui

* **Große Tabellen:** gemessen mit einer Million Zeilen zu fünf Spalten --
  eine Zelle setzen (auch sortiert oder gefiltert) 3 ms statt 1,4 s,
  sortieren 0,13 s statt 1,3 s, filtern 0,04 s statt 0,14 s, Speicher der
  Tabelle rund ein Viertel weniger. Gezeichnet werden ohnehin nur die
  sichtbaren Zeilen.
* **Elf Abfragen zu Settern:** `GUI_GET_ALIGN$`, `GUI_GET_ANCHOR$`,
  `GUI_GET_FONT_SIZE`, `GUI_GET_ICON$`/`GUI_GET_ICON`, `GUI_GET_ROUND`,
  `GUI_GET_MIN_W`/`_H`, `GUI_GET_WRAP`, `GUI_GET_TAB_INDEX`, `GUI_STYLE_GET`.
* **Rückgängig als Befehl:** `GUI_UNDO`, `GUI_REDO`, `GUI_CAN_UNDO`,
  `GUI_CAN_REDO`, `GUI_UNDO_CLEAR` -- derselbe Weg wie Strg+Z; ein Menü
  „Bearbeiten → Rückgängig“ kann sich jetzt richtig ausgrauen.
* **Aussehen des Systems:** `GUI_SYSTEM_DARK`, `GUI_SYSTEM_ACCENT` und
  `GUI_THEME_PRESET("system")` folgen dem dunklen Modus und der
  Akzentfarbe von Windows, macOS und Linux.

## Unter der Haube

**Zahlen.** 2462 Befehle (vorher 2441) in 57 Modulen, 234 Beispiele auf der
obersten Ebene von `examples/` (233), **5919 Fälle in 483
Prüfsammlungen** (5874 in 481) und 682 Rust-Testfunktionen (670).

* **Prüfhelfer für echte Fenster:** Fensternachrichten an ein bestimmtes
  Fenster (der Fenstersender kennt `+` für Nachrichten im selben Bild),
  Pixel aus einem verdeckten Fenster (`PrintWindow`), der Besitzer eines
  Fensters, ein fremder UIA-Leser je Fenster.
* **Funde beim Bauen:** ein Druck hält den Zeiger fest (Windows meldet das
  Verlassen mitunter vor dem Druck); verschiebt ein Programm ein Fenster
  unter dem Zeiger, meldet Windows danach den echten Zeiger.

## Was offen bleibt

* **Popups über den Rand eines echten Fensters** hinaus gibt es noch nicht.
* **macOS:** echte Fenster bleiben dort beim Rückfall, bis sie auf einem Mac
  geprüft sind; Bildschirmleser unter macOS und Linux sind ungeprüft.
* **Tabellen ohne Speichergrenze** (ein virtueller Modus, der Zeilen erst
  auf Nachfrage holt) fehlen noch.
* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
