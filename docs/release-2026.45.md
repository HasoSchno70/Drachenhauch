# Drachenhauch 2026.45

*Die Notizen zu dieser Fassung. Sie ist die Fassung **für das Studio**: eine
echte Anwendung -- eine Kühlschrank-Verwaltung mit 13 Fenstern, Datenbank,
Diagrammen, Bericht, Netzabfragen und englischer Oberfläche -- ist im
Drachenhauch Studio nachgebaut worden, und was ihr fehlte, kann das Studio
jetzt: zwei Sprachen, Fremdschlüssel, Kontextmenüs, Werkzeug- und
Statusleiste, Ziehen und Ablegen, ein Datumsfeld. Dazu Lichterglanz mit Dorf
und Markt. Was bestehende Programme bemerken können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Enter auf einem Kästchen, Schalter, Optionsfeld oder Zahlenfeld drückt den
  Standard-Knopf** (wie unter Windows, macOS, Qt und im Web). Vorher schaltete
  Enter das Kästchen um, und ein Dialog ließ sich nicht mit Enter bestätigen;
  umgeschaltet wird mit der Leertaste.
* **In einer nicht bearbeitbaren Klappliste wählt `GUI_SET_TEXT` den Eintrag**
  mit diesem Text, und `GUI_TEXT` liefert die Auswahl. Vorher setzte es einen
  Text, den die Liste nicht zeigte.
* **`HTTP_STATUS` ist 0, wenn eine Anfrage ohne Antwort scheiterte** (kein
  Netz, Zeitüberschreitung) -- vorher stand dort der Status der Anfrage davor.
* **Ein Klick in das Monatsgitter (`GUI_DATEPICKER`) trifft den Tag auch mit
  `wochenbeginn` ungleich Montag.** Vorher zählte der Klick ab Montag, mit
  „Woche ab Sonntag“ traf er den Tag daneben.
* **`AUDIO_SFX`: der Tiefpass schaukelt sich nicht mehr auf.** Bei Resonanz
  und hoher Grenzfrequenz kippte er auf NaN, und der ganze Ton blieb danach
  stumm (gefunden mit Lichterglanz im Browser). Darunter klingt alles wie
  bisher.
* **Im Studio erreicht eine Taste mit Strg oder Alt und jede F-Taste das
  Ereignis `Taste` auch aus einem Eingabefeld**, außer das Feld braucht sie
  selbst (kopieren, einfügen, Pfeile …) oder es ist AltGr. Ein Fenster ohne
  Menüleiste hat so Kürzel.
* **Das Studio baut beim Abgleich einer Datenbank Tabellen ohne
  Fremdschlüssel-Aktionen um.** Vorher nahm `DROP TABLE` beim Umbau über
  `ON DELETE CASCADE` alle Zeilen mit, die auf die Tabelle zeigten -- bei einer
  übernommenen Datei mit Fremdschlüsseln ein stiller Datenverlust.

## Neu in dieser Fassung

### Im Studio

* **Ein Projekt in zwei Sprachen:** eine Tafel `sprache/<code>.txt` im
  Projektordner; Fenster, Menüs samt Kürzeln, Diagrammtitel und Berichte
  übersetzt die Laufzeit selbst, im Code `tr$`/`trf$`, gewählt mit
  `SpracheWaehlen` oder über `DHRT_LANG`.
* **Datenbank aus einer vorhandenen SQLite-Datei übernehmen**, Arten und
  Vorgaben bleiben, wie sie in der Datei stehen; ein Verweis wird mit „Beim
  Löschen“ (verbieten, mitlöschen, leeren) zum Fremdschlüssel.
* **Werkzeug- und Statusleiste** mit Knöpfen, Feldern und den Ereignissen
  `Knopf(nr)`/`Feld(nr)`; die Menüleiste kommt obendrauf, ein großes Fenster
  bleibt in der Mitte des Entwurfs.
* **Kontextmenüs im Menü-Editor**, an ein Widget oder das Fenster, auch in
  einem eigenen Control.
* **Ziehen und Ablegen** (Ziehbar, Ablage, Ereignis `Abgelegt`), Ereignisse
  `Takt` und `Zelle`, modale Fenster mit Enter/ESC, Bausteine Diagramm und
  Datumsfeld, in der Laufzeit `Auswahl`, `EingabeZahl` und `FarbeWaehlen`.

### gui

* **`GUI_DATEFIELD`**: ein Feld mit dem Datum, das den Monat zum Wählen
  aufklappt; **`GUI_TABLE_COL_TYPE(tbl, spalte, "datum")`** klappt ihn beim
  Bearbeiten unter der Zelle auf.
* **`GUI_SHOWN`**, **`GUI_DROPDOWN_COUNT`**, **`GUI_DROPDOWN_ITEM`**,
  **`GUI_TABCONTROL_ITEM_RECT`**, **`GUI_WINDOW_CLIP`**.
* Reiter- und Spaltenköpfe, die nicht passen, werden schmaler bzw. enden mit
  „...“ -- vorher lagen Reiter jenseits des Rands, und ein Spaltenkopf schrieb
  in den nächsten.
* Ein modales Fenster nimmt den Fokus und gibt ihn zurück; ein Doppelklick auf
  einen Knopf in einer Tabellenzelle öffnet die Zeile nicht mehr.

### Laufzeit und Werkzeuge

* **`READALL$(pfad, kodierung$)`** liest wie `READLINES` mit Kodierung.
* `--check` warnt, wenn eine Methode die geerbte in anderer Schreibweise
  ersetzt; `dhrt run` sagt, wenn Argumente ohne `--` das Programm nicht
  erreichen.
* Fehlt ein Zeichen in der ersten Ausweichschrift, fragt die gui die nächste
  (Wecker, Stoppuhr, Sanduhr und Stern aus der Emoji-Schrift).

### Neue Beispiele

* **`studio/kuehlschrank/`**: die Kühlschrank-Verwaltung als Studio-Projekt --
  Vorrat, Lagerorte, Einkauf mit Gangplan und Einkaufszettel, Rezepte,
  Statistik, Scannen über Open Food Facts, Rezepte von Spoonacular,
  Benachrichtigungen, Sicherung der Datenbank, deutsch und englisch.
* **Lichterglanz**: ein Dorf, das sich aufbaut, ein Markt mit Leben, ein Baum
  ohne Grafikfehler, festlichere Musik; im Browser ohne Tonabbruch.

## Unter der Haube

**Zahlen.** 2432 Befehle (vorher 2426) in 57 Modulen, 234 Beispiele auf der
obersten Ebene von `examples/` (die Kühlschrank-App steht in
`examples/studio/`), **6068 Fälle in 518 Prüfsammlungen** (vorher 5949 in 497)
und 670 Rust-Testfunktionen (669).

* **`vm.rs` und `gui.rs` sind nach Themen in Teile zerlegt** (`vm_*.rs`,
  `gui_*.rs`); nur verschoben, das Verhalten ist dasselbe.
* Die Entwicklung der Kühlschrank-App mit allen 46 Funden steht in
  `geschichte/kuehlschrank.md`.
* Zwei Glutfaden-Proben, die unter Last wackelten, warten jetzt auf das, was
  sie prüfen.

## Was offen bleibt

* **Echte Geräte sind weitgehend ungeprüft** (Raspberry Pi, SPI, Kamera,
  Mikrofon, Arduino, Modbus) -- Rückmeldungen mit Hardware sind willkommen.
* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
