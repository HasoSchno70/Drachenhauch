# Das gui-Modul

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Jede Ausbaustufe des Retained-Mode-Toolkits: Tabelle, Waehler, Dialoge, Menues, Listen, Layout, Stildatei, Popover, Bausteine, Rechtschreibung, Rechts-nach-links, Uebergaenge, Verlaeufe, Dateibaum.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Built-in-Module schreiben

### Modul `gui` (Retained-Mode-GUI)

Persistente Fenster/Widgets (externe Typen `GUI_WINDOW`/`GUI_WIDGET` via `register_type`). Aufbau einmalig, pro Frame `GUI_UPDATE()` + `GUI_DRAW()`; Events per Polling (`GUI_CLICKED`/`CHECKED`/`VALUE`/`TEXT`/`HOVERED`) **oder** FUNCREF-Callbacks (`GUI_ON_CLICK`/`GUI_ON_CHANGE`). Widgets: Button, Label, Checkbox, Slider, TextInput, Panel, **Separator** (`GUI_SEPARATOR` — Trennlinie), **GroupBox** (`GUI_GROUPBOX` — gerahmte Gruppe mit eingelassenem Titel), **Table** (`GUI_TABLE` — professionelle Tabelle: fixierte Kopfzeile, V/H-Scroll, persistente Zeilen-Selektion. **Zellmodell** (`Cell` in gui.rs): jede Zelle mit eigener Vorder-/Hintergrundfarbe, Ausrichtung und ART (`text`/`bild`/`haken`/`balken`/`knopf`) — `GUI_TABLE_SET_CELL/CELL_COLOR/CELL_ALIGN/CELL_KIND/CELL_IMAGE/CELL_VALUE`, dazu `GUI_TABLE_ROW_COLOR` (ganze Zeile) + `GUI_TABLE_COL_ALIGN`. Farben in drei Stufen Zelle>Zeile>Zebra, -1 = nicht gesetzt; Auswahl/Hover liegen HALBDURCHSICHTIG darüber, sonst deckte eine Zellfarbe sie zu. **Spaltenzahl frei** = breiteste Angabe (`TableState::n_cols`) — kürzere Zeilen sind leer, keine Reihenfolge-Pflicht mehr (vorher war jede Abweichung ein Fehler, und Zeilen-vor-Kopf umging die Prüfung und crashte das Zeichnen). **Sortieren** per Kopfklick (2. Klick dreht um, Pfeil zeigt es an) — ZAHLENWEISE wenn beide Zellen Zahlen sind, sonst Text; `GUI_TABLE_SORT/SORT_COL/SORT_DESC`. **Filterzeile** im Kopf (`GUI_TABLE_SET(t,"filterzeile",1)`) — Teiltext je Spalte, case-insensitiv, UND-verknüpft; `GUI_TABLE_FILTER/GET_FILTER`. **Spaltenbreiten ziehbar** (Kopfkante, ±4px Fangbereich). **WICHTIG — Datenzeile vs. Ansicht:** Sortieren/Filtern stellen die Daten NIE um, sie bauen nur `view: Vec<usize>`; alle Zeilenangaben nach außen sind DATENzeilen (eine gemerkte Nummer bleibt gültig), sichtbare Reihenfolge über `GUI_TABLE_VIEW_COUNT/VIEW_ROW`. Einstellungen über EINEN Setter `GUI_TABLE_SET(key$,wert)` wie bei `chart`. Layout aus einer Quelle `table_geom` für Hit-Test + Zeichnen. Doku `docs/module-gui.md`, Demos `examples/157_gui_tabelle.dh` + `examples/158_gui_tabelle_sqlite.dh` (an einer echten SQLite-DB: lesen/sortieren/filtern/UPDATE beim Bearbeiten/DELETE in einer Transaktion; **arbeitet auf einer Kopie via `VACUUM INTO`**, die Originaldatei wird nur gelesen -- Test `tests/pruef/beispiele/beispiel_sqlite_tabelle.dhtest` sichert genau diese Zusage ab). Muster fuer DB-Anbindung: DB ist die Wahrheit, Tabelle die Ansicht; je Zeile die id merken (Sortieren/Filtern stellen Datenzeilen nicht um); beim Mehrfach-Loeschen ERST alle ids einsammeln, dann loeschen. **Zellen bearbeiten:** Doppelklick auf eine Textzelle einer per `GUI_TABLE_COL_EDIT` freigegebenen Spalte oeffnet ein Eingabefeld IN der Zelle (Enter uebernimmt, ESC nimmt zurueck, Klick woanders uebernimmt); `GUI_TABLE_EDITING_ROW/COL` fragen den Zustand ab. Arbeitskopie `edit_text` -- die Zelle wird ERST beim Bestaetigen geschrieben, sonst koennte ESC nichts zuruecknehmen und jede Taste wuerde Sortierung/Filter neu anwerfen (die Zeile spraenge beim Tippen weg). Doppelklick-Erkennung im Gui (`last_click`, zeitbasiert via `g.get_time()`, 0.4 s + max 4 px Versatz); `editing_table` merkt die Tabelle, damit ein Klick woanders in O(1) uebernimmt statt einen verwaisten Editor stehen zu lassen. **ESC-Falle:** benutzt das Programm ESC zum Beenden, muss es `GUI_TABLE_EDITING_ROW < 0` abfragen. **Feste Spalten** (`GUI_TABLE_SET(t,"feste_spalten",n)`): die ersten n scrollen waagerecht nicht mit. ALLES was Spalten verortet geht ueber EINE Quelle -- `col_x` (Lage) + `col_clip` (sichtbarer Bereich), von Treffertest UND Zeichnen benutzt; ein fester Block waere sonst der sicherste Weg, beide auseinander laufen zu lassen. **Mehrfachauswahl** (`mehrfachauswahl`-Schalter, aus per Vorgabe): Strg+Klick schaltet eine Zeile um, Umschalt+Klick waehlt den Bereich ab dem Anker -- in der SICHTBAREN Reihenfolge (ueber Datenzeilen traefe es bei sortierter Tabelle etwas anderes); `GUI_TABLE_SEL_COUNT/SEL_ROW/IS_SELECTED/SELECT/CLEAR_SELECTION`. `GUI_TABLE_SELECTED` bleibt die zuletzt angeklickte Zeile (Rueckwaertskompatibilitaet), beim Zeilen-Loeschen ruecken alle Auswahl-Indizes dahinter auf. **Spalten umsortieren** (`spalten_verschiebbar`, per Vorgabe AUS): Kopfzelle seitwaerts ziehen, Tausch sobald sie ueber die Mitte des Nachbarn kommt. Klick und Zug sind DIESELBE Geste -- erst beim Loslassen entscheidet sich, ob sortiert (keine Bewegung) oder verschoben wurde (>= 5 px); ohne das wuerde jedes Verschieben nebenbei sortieren. Wie bei den Zeilen werden die DATEN nicht umgestellt: `col_order` bildet nur Anzeige-Position -> Datenspalte ab, `TGeom.col_widths` steht in ANZEIGE-Reihenfolge, `pos_at` liefert die Position und `col_at` die Datenspalte. `TableState::order()` bereinigt+ergaenzt die Liste bei JEDEM Abruf, damit eine spaeter dazugekommene Spalte nicht unter den Tisch faellt. `GUI_TABLE_MOVE_COL/COL_AT/COL_POS/RESET_COLS`; Reihenfolge + feste Spalten werden im .dhform gespeichert. **Textauswahl in der Zelle:** der Zell-Editor benutzt DIESELBE Routine wie das TextInput-Widget (`Gui::einzeiler_tasten`, aus `edit_textinput` herausgeloest) -- Markieren per Umschalt+Navigation oder Maus-Ziehen, Strg+A/C/V/X, Pos1/Ende. Beim Oeffnen ist alles markiert (erstes Tippen ersetzt). **Gotcha:** der oeffnende Doppelklick darf NICHT als Marke-setzen zaehlen -- `edit_maus_sperre` sperrt die Maus, bis die Taste einmal los war; ohne das hob derselbe Klick die Markierung sofort wieder auf (Ruecktaste loeschte dann ein Zeichen statt des Inhalts). Klicks IM Eingabefeld sind sowohl in `handle_press` als auch in `table_press` ausgenommen, sonst schlossen sie den Editor. Das Feld-Rechteck wird aus `table_geom` GERECHNET (`edit_cell_rect`), nicht beim Zeichnen gemerkt -- `draw_cell` ist `&self`, und eine zweite nachgefuehrte Quelle liefe auseinander. Zeilengruppen gibt es als Baumtabelle (`GUI_TREETABLE`, siehe unten)), **Radio** (`GUI_RADIO(win,group$,text,x,y)` — Gruppen mit gegenseitigem Ausschluss, `GUI_RADIO_SELECTED`), **Dropdown/ComboBox** (`GUI_DROPDOWN(win,x,y,w,h,items)` — aufklappendes Popup über allen Widgets, `GUI_DROPDOWN_SELECTED/TEXT/SET_SELECTED`, `GUI_SET_DROPDOWN`; bearbeitbar per `GUI_DROPDOWN_SET(dd, "bearbeitbar", 1)`, siehe unten), **ProgressBar** (`GUI_PROGRESS`, Wert via `GUI_SET_VALUE` 0..1). **Laufzeit-Manipulation** (`GUI_SET_BOUNDS`/`GUI_GET_X/Y/W/H`, `GUI_DESTROY`, `GUI_SET_VISIBLE`/`GUI_VISIBLE`, `GUI_KIND`, `GUI_FOCUS`, `GUI_HIT_TEST`, Window-Pendants + `GUI_WINDOW_WIDGET_COUNT/WIDGET`-Enumeration; Tombstone-Handles bleiben stabil) + **Serialisierung** (`GUI_SAVE/LOAD` Datei, `GUI_TO_JSON/FROM_JSON` String) — Basis für dynamische UIs + WYSIWYG-Editor. **Farbe als Text** `COLOR_HEX$(farbe)` / `COLOR_FROM_HEX(text$)` (core): vorher konnte ein Programm Hex-Text GAR NICHT in eine Farbe wandeln -- `VAL("&HFF8800")` ist 0, und `&H`-Literale gibt es nur im Quelltext. Kurzform `#F80` wird verdoppelt; Deckkraft 0 (= deckend) laesst `COLOR_HEX$` weg.

**Farbwaehler + Datumswaehler** (2026-08-30, gefunden beim Partikel-Piloten -- `gui` hatte keinen Farbwaehler): `GUI_COLORPICKER`/`GUI_PICKED_COLOR`/`GUI_SET_PICKED_COLOR` und `GUI_DATEPICKER`/`GUI_DATE`/`GUI_SET_DATE`. Datumsformat `JJJJ-MM-TT` wie `DATE$()` -- zwei Formate waeren eine Stolperfalle; ein neuer Waehler zeigt HEUTE. Rechenkerne als eigene Module mit Rust-`#[test]`s, bewusst OHNE Widget: `farbraum.rs` (RGB<->HSV, Rundweg ueber 4096 Farben geprueft) und `kalender.rs` (Schaltjahre, Monatslaengen, Zellers Wochentag). **Der Farbwaehler speichert HSV, nicht RGB** -- bei Schwarz ist der Ton unbestimmt, bei Grau die Saettigung; wer zurueckrechnet, verliert sie und der Zeiger springt. Geometrie je Widget aus EINER Quelle (`cp_geom`/`dp_geom`), von Zeichnen und Treffertest gemeinsam benutzt; `cp_drag` merkt, ob die Geste im Feld oder im Streifen begann (sonst springt die Farbe beim Ziehen ueber die Grenze). **Falle beim Zeichnen:** Alpha 0 bedeutet DECKEND -- der Verlauf durchsichtig->schwarz beginnt bei 0x01000000, mit 0x000000 war das Feld schlicht schwarz. **Erweiterungen:** `GUI_COLORPICKER_SET(cp,"alpha",1)` blendet einen Deckkraft-Streifen ein (auf Schachbrett-Grund, sonst sieht man dem Verlauf nicht an wo er durchsichtig wird); `GUI_PICKED_COLOR` liefert dann 0xAARRGGBB statt 0xRRGGBB -- ohne den Schalter bleibt es bei sechs Stellen, damit bestehende Programme unveraendert bleiben. **Deckkraft geht 1..255, nicht 0** (oberstes Byte 0 = deckend). `GUI_DATE_RANGE(dp,von$,bis$)` (leer = keine Grenze; ein Datum ausserhalb wird sofort hereingezogen, gesperrte Tage nehmen weder Klick noch Taste an) und `GUI_DATEPICKER_SET(dp,"wochenbeginn",n)` (0=Montag..6=Sonntag -- eine feste Wahl waere fuer die halbe Welt falsch). Vier Blaetterbereiche im Kopf (Jahr/Monat je Richtung), Umschalt+Bild = Jahressprung. `cp_drag` merkt jetzt die ZONE (0 Feld / 1 Ton / 2 Deckkraft) statt eines Schalters.

Doku `docs/module-gui.md`, Demo `examples/186_farbe_und_datum.dh`, Tests `tests/pruef/gui/gui_waehler.dhtest` + Tastatur in `tests/pruef/gui/gui_tastatur.dhtest`.

**Grosse Dateien im Code-Feld:** `GUI_TEXTAREA_VIEW(ta)` -> TUPLE `(erste_zeile, zeilen, start_zeichen, laenge_zeichen)` -- damit faerbt ein Programm nur den SICHTBAREN Ausschnitt. **Gemessen je Tastendruck:** 3000 Zeilen 26 ms -> 0,2 ms; 10000 Zeilen 88 ms -> 0,8 ms; 30000 Zeilen 272 ms -> 2,1 ms. Der Engpass beim ganze-Datei-Weg ist NICHT das Zeichnen (konstant ~0,4 ms, nur sichtbare Zeilen) und nicht `SYNTAX_SPANS` (2,4 ms bei 3000 Zeilen), sondern die BASIC-Schleife Art->Farbe ueber 16500 Abschnitte (17,6 ms; mit einer MAP statt vier IF-Vergleichen 9 ms). Der Ausschnitt ist nicht nur schneller, sondern auch RICHTIG, weil Kommentare und Zeichenketten in Drachenhauch an der ZEILE enden -- bei Blockkommentaren waere das die Falle. Tests `tests/pruef/gui/gui_textarea_view.dhtest`.

**Eigene Schrift gilt fuer die ganze gui:** `SETFONT(handle)` setzt die aktive Schrift, und jedes Widget ohne eigene folgt ihr (`wfont` = `font_wahl(w.font, g.active_font())`). **Falle:** `SETFONT` uebernimmt auch die GROESSE, mit der `LOADFONT(pfad, groesse)` den Zeichensatz gebaut hat -- nach einem SETFONT ist der Text also womoeglich groesser/kleiner als vorher. Doku `docs/module-gui.md` (Abschnitt Eigene Schrift).

**Vorgabe-Schrift der gui ist Sans** (2026-10-31, Wunsch des Nutzers: die
Pixelschrift sah "nach Amiga" aus): ohne GUI_FONT und solange das Programm
bei der Pixelschrift ist (`active_font == -1`, also kein SETFONT), tauscht
`try_gui` die eingebaute Sans (`LOADFONT("sans", 40)`, einmal geladen in
`Vm::gui_sans`, -2 = gibt es nicht) in der TEXTGROESSE DES PROGRAMMS ein --
TEXT_SIZE skaliert die gui also weiter. `TEXT` und die Zeichenbefehle
bleiben pixelig. `GUI_FONT(-1)` setzt `gui_pixel` = die alte Regel (folgt
der aktiven Schrift, auch der Pixelschrift). DejaVu bei 20 hat eine
kleinere Versalhoehe als die Pixelschrift bei 20 -- die Felder passen
unveraendert. Elf Pruefungen, die Lagen/Masse der Pixelschrift festhalten
(Umbruchstellen, Einzugslinien, Autosize, Tooltip-Lage), tragen dafuer
`GUI_FONT(-1)`; IDE und Werkzeuge hatten ohnehin eigene Schriften. Tests
zwei Faelle in `schrift_bausteine.dhtest`. **Danach auch `ui` und
Diagramme** (`Vm::sans_eintauschen`, eine Stelle fuer gui, `UI_*` ueber
`try_graphics_app`, CHART_DRAW selbst): ein Diagramm mit `schrift` -1 nimmt
die Schrift der Oberflaeche, eines mit eigener Schrift gilt waehrend des
Zeichnens auch als aktive -- vorher mass `text_width_at` mit der aktiven
und zeichnete mit `schrift`, eine eigene Diagrammschrift sass also schief.
`GUI_FONT(-1)` schaltet alle drei auf die Pixelschrift.

**TEXTAREA als Code-Feld** (2026-08-30): `GUI_TEXTAREA_SPANS(ta, starts, laengen, farben)` faerbt Zeichen-Abschnitte ein, `SYNTAX_SPANS(quelltext$)` -> TUPLE `(starts, laengen, arten)` zerlegt Drachenhauch-Quelltext (Arten: kommentar/text/zahl/schluessel/name/operator), `GUI_TEXTAREA_SET(ta, key$, wert)` mit `zeilennummern`/`aktive_zeile`/`tab_fuegt_ein`/`tabbreite`. **Der Hervorheber (`syntax.rs`) ist bewusst KEIN Lexer** -- der lexer.rs wirft Kommentare weg, expandiert f-Strings und bricht bei Fehlern ab; ein Editor sieht halb getippten Text und muss ihn trotzdem darstellen (offene Zeichenkette endet an der ZEILE, sonst faerbt ein Anfuehrungszeichen den Rest der Datei). Die Wortliste teilen sich beide ueber `lexer::keyword`. **Zwei Fallen:** (1) Farbige Laeufe werden ueber die Breite des VORSPANNS positioniert, nicht durch Addieren der Laufbreiten -- eine Textbreite enthaelt den Abstand ZWISCHEN Zeichen, aber keinen dahinter, also klebten die Woerter an jeder Farbgrenze zusammen; ausserdem rechnen Schreibmarke und Auswahl schon so. (2) Die Nummernspalte verschiebt ALLES, was eine Spalte verortet -> eine Quelle `ta_gutter()`, die Zeichnen, Treffertest und Schreibmarke gemeinsam fragen; der Text wird auf den Bereich rechts davon geclippt, sonst laeuft eine waagerecht gescrollte Zeile in die Zahlen. `scroll` ist beim TextArea die erste sichtbare ZEILE, `scroll_x` der Versatz in PIXELN. `GUI_SET_TEXT` loescht die Abschnitte (sie gehoerten zum alten Text). `tab_fuegt_ein` per Vorgabe AUS -- sonst kaeme man im Formular nicht mehr aus dem Feld heraus; an, rueckt TAB bis zur naechsten SPALTE ein. Doku `docs/module-gui.md`, Demo `examples/184_codefeld.dh`, Tests `tests/pruef/werkzeugkette/syntax_spans.dhtest` + 12 Rust-`#[test]`s in `syntax.rs` + Tabulator in `tests/pruef/gui/gui_tastatur.dhtest`.

**Dialog IM Fenster** `GUI_DIALOG(titel$, text$[, stil$])` -> GUI_WINDOW, `GUI_ANSWER(dlg)` (0 offen / 1 OK-Ja / 2 Abbrechen-Nein), `GUI_MODAL()`. **NICHT verwechseln mit `GUI_MESSAGE`/`GUI_CONFIRM`** -- das sind die schon vorhandenen NATIVEN, BLOCKIERENDEN OS-Kaesten (rfd, `filedialog.rs`, Feature `dialogs`, im Web-Bau nicht da). `GUI_DIALOG` ist der Kasten im eigenen Thema/Massstab, blockiert NICHT (gui ist ein Polling-Toolkit; ein blockierender Dialog muesste mitten im Bild des Aufrufers eine eigene Zeichenschleife drehen und dessen Layer-/Render-Ziel-Zustand uebernehmen). Die Antwort gilt GENAU EIN BILD, wie `GUI_CLICKED` -- eine Antwort ist ein Ereignis, kein Zustand; danach ist das Fenster zerstoert (Tombstone-Handle bleibt gueltig). Modalitaet wird in `handle_press` UND `menu_input` durchgesetzt (`modal: Option<usize>`), sonst waere es nur ein Fenster obenauf; dazu ein Schleier in `draw` -- ohne sichtbares Zeichen klickt man in den Hintergrund und wundert sich. Stil-Woerter absichtlich dieselben wie bei `GUI_CONFIRM` (`ok`/`janein`). **Falle:** Widget-Koordinaten sind relativ zum INHALTsbereich, die Fensterhoehe nicht -- ohne den `title_h`-Zuschlag rutscht die Knopfreihe unter den Rand. Doku `docs/module-gui.md`, Tests `tests/pruef/gui/gui_dialog.dhtest` (die Modalitaet mit Gegenprobe: derselbe Klick trifft ohne Dialog). **Ein Fenster ueber einer Zeichenflaeche braucht `GUI_DRAW_WINDOW(win)`** -- `GUI_DRAW` zeichnet Fenster und Zeichenflaechen in EINEM Durchgang, und der Inhalt einer Zeichenflaeche entsteht per Bauart DANACH (das Programm malt selbst hinein). Ohne den zusaetzlichen Aufruf ist ein Dialog mitten auf der Flaeche unsichtbar: er sperrt die Eingabe und ist nicht zu sehen, das Programm wirkt eingefroren. Ein zweites `GUI_DRAW` hilft NICHT (es zeichnet Fenster-Hintergrund und Zeichenflaechen neu). Ein unsichtbares oder zerstoertes Fenster zeichnet nichts, ist aber kein Fehler -- der Aufruf darf unbedingt in der Bildschleife stehen. Tests `tests/pruef/gui/gui_draw_window.dhtest` (bis 2026-09-16 pytest mit Pillow; seither `--- bild`, die Faelle mit und ohne Aufruf stehen als Paare). **Kontextmenue und Tooltip liegen ueber ALLEN Fenstern** und haben dasselbe Problem -- der Tooltip folgt der Maus und landet also staendig ueber einer Zeichenflaeche: dafuer `GUI_DRAW_TOP()`, und `GUI_DRAW(FALSE)` laesst die Schicht beim Hauptdurchgang weg. Zweimal zeichnen waere hier KEIN Ersatz: Tooltip und Kontextmenue haben einen halbdurchsichtigen Schlagschatten, der uebereinander dunkler wird -- und zwar nur dort, wo das eigene Zeichnen die erste Fassung nicht zugedeckt hat, also genau an einer Kante. Bei Fenstern (`GUI_DRAW_WINDOW`) bleibt genau das ein bekannter Rest: ein Fenster, das nur teilweise unter dem selbst Gezeichneten liegt, bekommt seinen Glanz dort zweimal.

**Anzeige-Massstab** `GUI_SCALE(faktor)` (0.5..4.0) + `GUI_SCALE_GET()`: alle gui-Masse sind feste Pixel, auf einem 4K-Schirm mit 200 % wurde eine Oberflaeche darum halb so gross wie gedacht. Der Faktor multipliziert JEDE Laenge, die HINEINgeht (Fenster-/Widget-Geometrie, Metriken, die 15 Layout-Konstanten via `sk()`, Spaltenbreiten, Zeilenhoehen, Schriftgroesse); nach aussen bleibt alles LOGISCH (`unsk()` in den Gettern und in `widget_json`/`to_json`) -- sonst wuechse eine `.dhform` bei jedem Speichern um den Faktor weiter. EINZIGE Ausnahme: `GUI_HIT_TEST` spricht Bildschirm-Pixel (die Maus liefert nichts anderes). **Muss vor dem ersten Fenster kommen** -- danach Fehler, weil Bestehendes nur naeherungsweise umzurechnen waere. **Zwei Fallen, die erst das gerenderte Bild zeigte:** (1) `g.text`/`g.text_width` in der Fenster-Chrome (Titel/Menue/Reiter/Popup/Tooltip) kannten den Massstab nicht -> `ctext`/`ctext_width`/`ctext_height`, und die Editier-Helfer massen unskaliert, waehrend `wtext` skaliert zeichnete (Schreibmarke und Auswahl sassen auf halber Strecke) -> Buendel `Mass{size,font}` durch die statischen Helfer. (2) Die senkrechte Zentrierung stand als Literal `(h - 14) / 2` im Code -- bei Massstab 2 sass der Text zu tief und wurde vom Clip-Rechteck abgeschnitten -> `self.wsize(g,wdg)`. `GUI_LOAD`/`GUI_FROM_JSON` umgehen `add_widget` und brauchen den Massstab eigens. Doku `docs/module-gui.md` (Abschnitt Massstab), Tests `tests/pruef/gui/gui_massstab.dhtest`.

**`GUI_CLICKED` gilt auch fuer Kaestchen, Kippschalter und Radioknoepfe** (seit
2026-09-01). Vorher war es dort STUMM -- immer FALSE, ohne Fehler; zwei von zwei
Programmen, die es versucht haben (die Piloten 187 und 189), hatten damit einen
toten Schalter. Ein Knopf setzt sein Flag beim LOSLASSEN, ein Kaestchen kippt
schon beim DRUECKEN -- fuer den Abfragenden ist beides "in diesem Bild
angeklickt", und beides meldet genau ein Bild lang. Der Zustand kommt weiter aus
`GUI_CHECKED`. Tests `tests/pruef/gui/gui_clicked_schalter.dhtest` (mit Gegenprobe:
danebengeklickt meldet nichts).

**Popover, Toast, Infoleiste, Suchfeld, Vorschlaege** (2026-10-31, Paket 1
aus `docs/entwurf-gui-wie-gtk.md`): eigenes Untermodul `gui_ueber.rs`
(`#[path]` wie das Dock). `GUI_POPOVER(anker, b, h [, seite$])` ist ein
randloses GUI_WINDOW mit `Window::pop` (Anker, Seite, Pfeil); gelegt in
`pop_lage` je Bild (weicht am Bildschirmrand aus, geht zu, wenn der Anker
nicht mehr zu sehen ist), gezeichnet in der OBEREN Schicht (`draw` laesst
Popover aus, `draw_top` -> `ueber_zeichnen`), in der Fensterreihenfolge
immer oben (`pops_nach_vorn` am Ende von `bring_to_front`). `GUI_TOAST`
(`Window::toasts`, Zeit ueber `g.delta()`, hoechstens vier, liest vor wie
GUI_ANNOUNCE) und `GUI_INFOBAR` (`Window::info`, ueber dem Inhalt, NICHT
verschiebend; `GUI_INFOBAR_BUTTON_RECT` fuer Pruefungen). Textfeld:
`Widget::feld` (`FeldZusatz`, nur bei Bedarf) mit Sinnbildern
(`GUI_TEXTINPUT_ICON`), Suchfeld (`GUI_TEXTINPUT_SET "suche"`,
`GUI_SEARCH_CHANGED` 150 ms nach dem letzten Anschlag) und Vorschlaegen
(`GUI_TEXTINPUT_COMPLETION`). **Ein Druck geht zuerst durch
`ueber_druck`** (in `update_innen`, gleich nach dem Legen): was dort landet
oder einen Popover schliesst, ist verbraucht -- `just_pressed`/`right_just`
werden danach ueberschattet. Die Lage des Textes im Textfeld geht ueber
`ti_rand` (Zeichnen, `marke_von`, Klick in `edit_textinput`,
`ti_rechtsklick`). Tasten: `feld_tasten` vor `edit_textinput`, `pop_esc`
vor dem Abbrechen-Knopf, beide setzen `liste_taste`. Farbteile im Vorschlag
werden ueber die Breite des GANZEN Textes minus den Rest gelegt. Tests
`tests/pruef/gui/gui_ueberlagerung.dhtest` (18; neun Verfaelschungen fallen je in
ihrem Fall). **Testfalle:** die Wiedergabe beginnt beim ERSTEN Ereignis --
ein spaeter Klick braucht ein Ereignis in Bild 0 davor.

**Stildatei** (2026-10-31, Paket 4 erste Stufe): `gui_stil.rs`, `#[path]`
wie Dock und Ueberlagerungen. `GUI_STYLESHEET`/`_LOAD` liest Regeln
(`art.klasse#name:zustand { eigenschaft: wert }`, Kommentare `/* */` und
`//`), Fehler mit Zeile; Gewicht Name 100, Klasse/Zustand 10, Art 1, gleich
= spaeter gewinnt. `stil_pass` (am Ende von GUI_UPDATE) loest je Widget nur
neu auf, wenn sich `(stil_stand, checked, enabled, variante)` oder Klassen/
Name (`stil_key = None`) geaendert haben -- in FUENF Schichten (Basis,
hover, fokus, aktiv, aus). Farben: `wcol` = GUI_SET_COLOR > `eigen`
(Basis) > Thema, dann `stil_misch` (hover/fokus/aktiv ueber
`ueber_t`/`fokus_t`/`druck_t` gemischt, gesperrt = Schicht aus); die
sieben frueheren `ov.get`-Stellen gehen ueber `eigen`/`stil_farbe`. Der
Knopf nimmt seine Hover-/Druckfarbe aus den Schichten (`bg_h`/`bg_d`),
nicht aus `stil_misch` -- sonst blendete er doppelt. Schrift ueber
`w_groesse`/`w_stil` (eigene vor Stildatei), Rundung ueber
`rad_jetzt` (Cell, in `draw_widget` gesetzt, in `flaeche` gelesen).
Themenfarben (`bg: accent`) werden erst beim Zeichnen aufgeloest und
folgen einem Themenwechsel. Die Knopfart zaehlt als Klasse. `GUI_GET_COLOR`
liefert die Farbe des jetzigen Zustands ohne Ueberblenden (-1 = Thema).
`.dhform` `name`/`klassen`. Tests `tests/pruef/gui/gui_stildatei.dhtest` (12;
sechs Verfaelschungen fallen je in ihrem Fall), Rust-Tests in
`gui_stil.rs`. Nachfahren, Uebergang je Farbe und `window#name` kamen mit
dem Rest (unten). **Zweite Stufe (selber Tag): Form je
Widget** -- `abstand`, `randbreite`, `schatten`, `verlauf`, `glanz`,
`fase`, `uebergang` (englisch padding/border-width/shadow/gradient/gloss/
bevel/transition), NUR in Regeln ohne fluechtigen Zustand (sonst Fehler
mit Zeile). Ohne Umbau Art fuer Art: `form_jetzt` (Cell `FormStil`, in
`draw_widget` neben `rad_jetzt` gesetzt) -> `m_form("gradient"|"gloss"|
"bevel")` ersetzt ueberall `self.m(...)` dieser drei, die Randbreite zieht
`flaeche` als Ringe nach innen; `schatten_zeichnen` vor dem Widget
(gestaffelte runde Rechtecke, Deckkraft 0 = deckend beachten);
`uebergaenge` nimmt je Widget `w_uebergang`; `abstand` wirkt im Mass nach
Inhalt (Knopf exakt Text + 2 x Abstand, Feld-Hoehe) und im Textfeld ueber
`ti_rand` (ersetzt die 5 Punkte Einzug -- ALLE Wege fragen ti_rand; die
Sinnbilder selbst ueber `ti_symbole`). Tests sechs Faelle mehr in
`gui_stildatei.dhtest` (18); sieben Verfaelschungen fallen je in ihrem Fall.
**Rest von Paket 4 (2026-11-01):** Nachfahren (`panel.leiste button`,
`layout > button`, `window#name button`): `Selektor::vorfahren` = Teile mit
"direkt"-Merker, `vorfahren_passen` (direkt = erstes Glied, sonst jede
Stelle). Die Kette (`ketten`, nur wenn `Stildatei::mit_vorfahren`): erst
das Layout, das das Widget als Kind fuehrt, sonst `tc_von`
(Reiterwerk/Akkordeon/Assistent), sonst `panel_von`, hoechstens 32 Stufen,
zuletzt das Fenster (`art "window"`, nur als Vorfahre erlaubt; Name per
`GUI_WINDOW_SET_NAME/GET_NAME$`, `.dhform` `name` am Fenster). Ihr Hash
steht im Schluessel `stil_key` -- eine Klasse am Behaelter oder ein Umzug
loest neu auf. Ein Zustand nur am letzten Teil. `abstand` jetzt auch fuer
Beschriftung (Zeichnen, `label_stelle`, Umbruch, Mass), Kaestchen/Radio/
Kippschalter (Luecke vor dem Text), Liste/Klappliste (`list_zeile_h` =
Schrift + 2 x senkrecht, `list_text_x`), Klappliste/Drehfeld (`w_pad`
statt Metrik `pad`) und Kachel; Baum/Tabelle/Textbereich nicht.
`uebergang: 120, bg 400, border 0` (`uebergang_lesen`): `Widget::rollen_t`
je Rolle (bg/fg/border/accent) x (hover/fokus/druck), nur wenn eine Regel
Rollendauern nennt; angelegt mit dem Stand VOR dem Schritt (sonst ist eine
lange Rolle im ersten Bild am Ziel -- vom Test gefunden), gelesen ueber
`rolle_t` in `stil_misch` und in der Grundfarbe des Knopfs. **Falle:**
`GUI_AUTOSIZE` misst SOFORT -- vor dem ersten GUI_UPDATE kennt ein Widget
seinen Stil noch nicht. Tests sechs Faelle mehr in `gui_stildatei.dhtest`
(24), zwei Rust-Tests; sechs Verfaelschungen fallen je in ihrem Fall.

**Bausteine** (2026-10-31, Paket 2): `gui_bausteine.rs`, Zusaetze je
Widget in `Widget::bau` (`Bauzusatz`, nur bei Bedarf). Fliesslayout = Art 3
(`"fluss"`) in `layout_anwenden` (`fluss_legen`), Auswahl per Klick in
`fluss_druck` (gleich nach `ueber_druck`, der Druck geht weiter ans Kind),
`GUI_CLICKED(layout)` meldet. Alt-Buchstaben (`GUI_MNEMONICS`): Anzeigetext
ueber `anzeige_text`/`ohne_marke` (Messen in `inhalt_mass_roh`, Zeichnen
ueber `wtext_mn`), ausgeloest in `mn_pruefen` direkt nach
`kuerzel_pruefen` (Menue-Kuerzel gehen vor); eine Beschriftung gibt den
Fokus an das naechste bedienbare Widget nach ihr. Beschriftung: Zeilen aus
EINER Quelle `label_zeilen` (Kuerzen, Umbruch, Zeichenversatz fuer die
Auswahl) fuer Zeichnen (`label_zeichnen`) und Treffer (`label_stelle`);
Markieren/Strg+C in `label_pass`, gekuerzt fuellt `ganzer_text` den
Hilfetext. Regler: Marken/Einrasten (`regler_einrasten` in `drag_slider`,
`regler_nachbar` fuer Pfeile). Kreisel = Fortschritt mit `kreis`
(`GUI_BUSY`, keine neue Widget-Art), mit Wert ein Ring. Datumsmarken
(`dp_marke` in `draw_datepicker`). Groessengruppen: `gruppen_pass` am Anfang
von `layout_pass` (feste Widgets sofort, nach Inhalt ueber `inhalt_mass` ->
`gruppe_anwenden`). Schriftwahl: ein Dialog aus gewoehnlichen Widgets
(`SchriftWahl`, Filter und Vorschau in `schriftwahl_pass`), die Wahl
bleibt im Tombstone lesbar; `DHRT_SCHRIFT_ORDNER` fuer Pruefungen. Tests
`tests/pruef/gui/gui_bausteine.dhtest` (15; zwoelf Verfaelschungen fallen je in
ihrem Fall -- der Kreisel erst mit dem Ring-Fall, weil der Balken seine
laufende Flaeche ebenfalls in Akzentfarbe malt).

**Rechtschreibung** (2026-10-31, Paket 3, erster Teil): `rechtschreibung.rs`
(ungated, Woerterbuecher als Handles in einem `thread_local`, das am
Programmende NICHT abgeraeumt wird -- das Freigeben der Windows-Pruefung beim
Abbau des Fadens hing, das Programm endete nie). Zwei Quellen, ein Vertrag:
unter Windows `ISpellChecker` (Win32_Globalization, Spannen in UTF-16 ->
Zeichen umgerechnet), sonst Hunspell ueber `spellbook` aus den ueblichen
Ordnern; `DHRT_WOERTERBUECHER` geht vor (Pruefungen, eigene Sammlungen) und
schaltet auch unter Windows auf Dateien. Befehle `SPELL_LOAD/SYSTEM/
LANGUAGES/LANGUAGE$/CHECK/SUGGEST/IGNORE/ERRORS/FREE` (builtins.rs).
`gui_rechtschreibung.rs`: `GUI_SPELL(feld, h)` / `GUI_SPELL_COUNT` fuer
Textfeld und Textbereich (`Widget::rs`); `rs_pass` am Ende von GUI_UPDATE
prueft 0,3 s nach der letzten Aenderung, bis dahin wandern die Wellen ueber
gemeinsamen Anfang/Ende mit (`verschieben`: an der Aenderung grenzend bleibt
ein Wort nur, wenn dort kein Wortzeichen steht). Die Wellen des Textbereichs
laufen durch dieselbe Zeichenschleife wie `wellen` (Fehler/Warnungen),
das Textfeld ueber `rs_textfeld_wellen`. Rechtsklick auf ein falsches Wort
(`rs_druck`, gleich nach `ueber_druck` -- geht dem Kontextmenue des
Programms vor) oder Umschalt+F10 (`rs_tasten`, neben `menue_tasten`)
oeffnet ein Menue mit Vorschlaegen und "ignorieren"; ein Vorschlag ist ein
Undo-Schritt mit on_change; solange es offen ist, gehoeren ihm die Tasten
(auch Enter/ESC -- Standard- und Abbrechen-Knopf liegen hinter derselben Sperre).
Passwortfelder pruefen nie. Tests `tests/pruef/module/rechtschreibung.dhtest`
(Hunspell-Beilagen, ein Fall mit der Windows-Pruefung samt Emoji vorn),
Rust-Tests in beiden Dateien.

**Rechts nach links** (2026-10-31, Paket 3, zweiter Teil):
`schreibrichtung.rs` (rein, Rust-Tests) ordnet eine Zeile mit
`unicode-bidi` (UAX 9, erstes starkes Zeichen) und formt Arabisch in die
Praesentationsformen (FE70-FEFF, Persisch aus Forms-A; Lam-Alef = EINE
Glyphe) -- nicht ueber rustybuzz, weil raylib Zeichen fuer Zeichen aus dem
Atlas zeichnet und die Formen dort Zeichen sind. `Zeile` traegt
`log_von_vis`/`vis_von_log`/`rtl` je logischem Zeichen; Steuerzeichen
(LRM, LRO, Isolate) fallen weg, Klammern werden gespiegelt. **Umgewandelt
wird beim Eintritt in Graphics** (`visuell`/`visuell_string` in
`text_emit`, `text_rot*`, `text_styled`, `text_nachgebildet` und
`breite_mit`) -- damit kann es jeder gezeichnete Text ohne Zutun. Wer schon
geordnete Stuecke zeichnet, schaltet `richtung_roh` (Cell) ein, sonst
drehte sich ein Stueck zweimal. Die gui (`gui_richtung.rs`): `Lage` =
Anzeige-Zeichen samt Vorspannbreiten; `marke(i)` nimmt das Zeichen NACH der
Marke (rtl: rechte Kante), `bereiche(a, b)` = mehrere Stuecke,
`index(x)` = Klick (eine Ligatur deckt zwei logische Zeichen).
`caret_index_at` selbst fragt die Lage zuerst -- damit stimmen ALLE
Klickwege auf einmal (Textfeld, Textbereich, Zelle, Rechtsklick,
Rechtschreibmenue). Schreibmarke/Auswahl/Wellen/Vorschau in
`zeichne_textfeld`, im Textbereich und in `marke_von` ueber
`zeilen_x`/`zeilen_bereiche`; Pfeile tauschen, wenn die Zeile als Ganzes
rtl steht; `GUI_TEXTAREA_POS_AT` ueber `Lage::zeichen_bei`. Eine eingefaerbte
Zeile von rechts nach links zeichnet `rtl_bunt` Stueck fuer Stueck roh
(`zeilen_farben` aus `zeile_bunt` herausgeloest), Inline-Formate fallen dort
weg. Neu `TEXT_VISUAL$(text$ [, richtung$])`. Bewusst nicht: volle
Textformung, Ausrichtung nach Richtung, Hinweise/Farbfelder (logische Lage),
pdf. Tests `tests/pruef/grafik/schreibrichtung.dhtest` (7; Bilder als Paar gegen
LRO-erzwungene Reihenfolge; fuenf Verfaelschungen fallen je in ihrem Fall --
die erste Fassung der Zeichen-Verfaelschung traf `visuell` statt
`visuell_string` und blieb gruen), 10 Rust-Tests.

**Uebergaenge** (2026-09-23, erster Punkt der Feinschliff-Liste): Metrik
`uebergang` (ms, Vorgabe 120, 0 = springen wie frueher). Je Widget
`ueber_t`/`druck_t`/`fokus_t` (0..1), dazu `LeisteState::hover_t` je
Eintrag, nachgefuehrt in `Gui::uebergaenge` (in `update`, mit `g.delta()` --
headless fest 1/60, also reproduzierbar); gezeichnet ueber `weich()`
(smoothstep) und `deckkraft()` (Alpha 0 = DECKEND, darum 1 statt 0 fuer
"weg"). **Nur das Ueberfahren blendet hinein**, Druecken und Fokus
erscheinen im selben Bild und blenden nur aus. Die Flags `hovered`/
`press_origin`/`focus_widget` bleiben die Wahrheit fuer alles, was
entscheidet. Erfasst: Knopf (alle Arten), Kachel, Kaestchen, Radio,
Klappliste, Werkzeugleiste, Fokusring, Kippschalter (vorher je Bild * 0,28,
also bildratenabhaengig). `mischen` gibt an den Enden die Farbe samt Alpha
unveraendert zurueck -- sonst machte ein ruhender Uebergang eine
halbdurchsichtige Farbe deckend. **Akkordeon** (selber Tag): `AkkState::auf_t`
je Abschnitt, weitergefuehrt in `akk_pass` (nicht in `uebergaenge` -- sonst
stuende die Lage ein Bild hinter dem Zustand, und mit `uebergang` 0 zeigte
ein GUI_UPDATE nach GUI_ACCORDION_OPEN noch den alten Stand); gemessen wird
immer die VOLLE Inhaltshoehe, sichtbar ist `voll * weich(t)`. Fehlt ein
Eintrag, gilt der Zustand -- ein beim Aufbau geoeffneter Abschnitt klappt
nicht beim ersten Bild auf. Kinder werden beim Zeichnen auf den sichtbaren
Teil beschnitten (`akk_ausschnitt`, eine Quelle mit `widget_shown`),
bedienbar sind sie erst ganz sichtbar. Das Dreieck dreht sich um seinen
Schwerpunkt; im Standbild sieht es bei 35 Grad wie ein Pfeil nach oben aus
(fast gleichseitig), in Bewegung nicht. **Baum** (selber Tag):
`TreeState::auf_t` je Knoten, gezeichnet ueber `tree_zeilen_weich` --
(Knoten, Hoehenanteil = Produkt der Oeffnungsgrade aller Vorfahren), jede
Zeile mit Anteil < 1 auf ihre Hoehe beschnitten; Treffertest/Tastatur/Rollen
bleiben bei `tree_visible`. `GUI_TREE_CLEAR` leert `auf_t` (sonst erbten neue
Knoten die Werte alter Nummern), der Dateibaum fuehrt es beim Neuaufbau am
WEG mit (`dateibaum_neu`) -- zu geht er sofort, seine Kinder sind dann nicht
mehr gelesen. **Klappliste**: `Gui::dd_auf_t`/`dd_auf_von`, nur AUF weich
(von oben herab, eingeblendet), zu sofort; im Bild des Klicks gilt "noch zu"
(`uebergaenge` lief vor dem Druck) -- mit "ganz offen" blitzte die volle
Liste einmal auf, gesehen nur im Kontaktbogen. **Zeilen und Menues**
(selber Tag): `Blende` (zwei Plaetze: die Zeile unter der Maus blendet ein,
die verlassene aus; zurueck auf die ausblendende = dort weiter) als
`Widget::zeile_b` fuer Liste/Tabelle/Baum/Klappliste und `Menu::b` je Popup,
dazu `Menu::auf_t` (Aufrollen, auch Untermenues; zu = sofort, zuruecksetzen).
Die Liste traegt ihre Maus-Zeile in `Widget::zeile_jetzt` -- eine schlichte
Liste hat keinen ListState, der sie halten koennte, und haette ihr
Ueberfahren sonst verloren. Tests `tests/pruef/gui/gui_liste_bedienung.dhtest`.
Tests `tests/pruef/gui/gui_uebergaenge.dhtest` (15,
Bildproben; Gegenprobe mit `uebergang` 0 bzw. dem Bau davor) und in
`tests/pruef/gui/gui_akkordeon.dhtest` "das aufklappen waechst ueber die zeit"
(Gegenprobe faellt); zwei alte Faelle dort brauchen seither `uebergang` 0
bzw. klicken spaeter ins Kind.

**Listen bequemer** (2026-09-23, mit den Uebergaengen): (1) **Rollbalken**
(`list_bar_geom` = EINE Quelle fuer Zeichnen, Klick, Ziehen; Griff ziehen,
Klick in die Rinne setzt ihn unter die Maus und zieht weiter,
`Gui::listbar_zug`; die Zeilen enden davor) -- vorher hatte eine Liste GAR
KEINEN, man sah nicht, wo man stand. (2) **Weiches Rollen** am Rad
(`Widget::rad_ziel`/`rad_letzt`: Radschritte addieren sich aufs Ziel; steht
der Stand nicht mehr da, wo das Rollen ihn hinschrieb, hat jemand anderes
gerollt, und das Rad gibt nach). (3) **Rechtsklick waehlt** die Zeile
(`liste_rechtsklick`; eine Mehrfachauswahl bleibt, wenn die Zeile
dazugehoert). (4) **Umbenennen**: `GUI_LISTBOX_SET(lb, "bearbeitbar", 1)` +
F2, `GUI_LISTBOX_EDIT/EDITED/EDITING`; Arbeitskopie `ListEdit`, dieselbe
Tastenroutine wie Textfeld und Zelle (`einzeiler_tasten`), Enter/Klick
daneben/Fokus weg uebernimmt, ESC verwirft, unveraendert ist kein Ereignis;
`.dhform` `bearbeitbar`. **Der Fund dabei:** ESC beendete das Umbenennen UND
drueckte den Abbrechen-Knopf des Fensters -- die Liste war im selben Bild
danach nicht mehr "am Bearbeiten", und die Fensterregel griff trotzdem;
`Gui::liste_taste` merkt sich, dass die Taste schon verbraucht ist (vom Test
gefunden). F2 gilt nur, wenn kein Menue-Kuerzel die Taste genommen hat (die
IDE legt F2 auf die Lesezeichen). Tests `tests/pruef/gui/gui_liste_bedienung.dhtest`
(20; gegen den Bau davor fallen alle 12, die Neues pruefen).

**Tabelle bequem wie die Liste** (selber Tag, Zeilenmodus): Doppelklick und
Enter setzen `TableState::doppel`, und `GUI_DOUBLE_CLICKED` gilt jetzt auch
fuer Tabellen (vorher ein Fehler "bisher nur fuer Listen") -- ein
Doppelklick auf eine BEARBEITBARE Textzelle bearbeitet weiter und meldet
nicht. F2 bearbeitet die erste freigegebene Textspalte der gewaehlten Zeile
(Zellmodus: wie bisher die aktuelle Zelle). Tippen springt
(`tabelle_tippen`, Regel wie `liste_tippen`) in der Sortierspalte, ohne
Sortierung in der ersten sichtbaren. Rechtsklick waehlt
(`tabelle_rechtsklick`). Das Rad rollt weich ueber dieselben
`Widget::rad_ziel`/`rad_letzt` wie die Liste -- der laufende Stand liegt in
`rad_letzt` als Kommazahl, weil `scroll_y` ganzzahlig ist (sonst bliebe die
Tabelle einen Punkt vor dem Ziel stehen). Neu `GUI_TABLE_PLACEHOLDER(tbl,
text$)` (Leer-Hinweis, `.dhform` `leer_text`). Rollbalken zum Ziehen hatte
die Tabelle schon. **Testfalle:** das Tippen fasst Tasten innerhalb einer
Sekunde ECHTER Zeit zu einem Wort -- ohne Fenster laufen 30 Bilder in
Bruchteilen davon, "b, b, k" wurde "bbk"; getrennte Woerter brauchen eigene
Faelle. Tests `tests/pruef/gui/gui_tabelle_bedienung.dhtest` (13; gegen den Bau
davor fallen die 10, die Neues pruefen). Im **Form-Designer** (197): Kaestchen "Umbenennen mit
F2" bei der Liste (`list.bearbeitbar`, abgewaehlt wird der Schluessel
ENTFERNT) und Feld "Leer-Hinweis" bei der Tabelle (`table.leer_text`), beides
im DH-Code (`GUI_LISTBOX_SET ... "bearbeitbar"`, `GUI_TABLE_PLACEHOLDER`);
Uebernehmen/Aktiviert/Hinweis ruecken dafuer eine Zeile tiefer. Tests in
`tests/pruef/werkzeuge/werkzeug_formdesigner.dhtest` (Inspektor-Fall + der
DH-Code-Fall mit allen Arten; gegen den alten Designer fallen beide).

**Baum bequem wie die Liste** (selber Tag): Rollbalken (derselbe
`list_bar_geom`, jetzt fuer Liste UND Baum -- der Rollstand liegt in `value`
bzw. `tree.scroll`, `roll_ist`/`roll_setze`), weiches Rad, Pos1/Ende/Bild,
Tippen springt (`baum_tippen`, sichtbare Knoten), Rechtsklick waehlt,
`GUI_DOUBLE_CLICKED` auch fuer Baeume (Doppelklick auf jeden Knoten, ein Ast
klappt dabei zusaetzlich um; Enter nur auf Blaettern -- auf einem Ast klappt
Enter), Umbenennen mit `GUI_TREE_SET "bearbeitbar"` + F2 bzw.
`GUI_TREE_EDIT/EDITED/EDITING` (dieselbe `ListEdit`-Arbeitskopie;
`edit_rect_any`/`edit_ende_any`/`wird_umbenannt` fassen Liste und Baum fuer
Klick-daneben, Fokus-weg und die Enter/ESC-Sperre zusammen) und
`GUI_TREE_PLACEHOLDER`. **Nicht beim Dateibaum**: sein Name ist ein
Dateiname, `GUI_TREE_EDIT` ist dort ein Fehler mit Hinweis auf `RENAME`.
Form-Designer: Kaestchen und Feld auch fuer den Baum (`tree.bearbeitbar`,
`tree.leer_text`). Tests `tests/pruef/gui/gui_baum_bedienung.dhtest` (14; gegen den
Bau davor fallen 13, der Endstand-Fall nicht) und ein Baum-Fall in
`werkzeug_formdesigner.dhtest`.

**Klappliste bequem wie die Liste** (selber Tag): das Popup zeigt hoechstens
`DD_MAX_ZEILEN` = 10 Eintraege (vorher so hoch wie die Liste lang -- hundert
Eintraege ragten ueber den Bildschirm), rollt mit dem Rad (vor allem anderen
abgefragt, weil es ueber allem liegt) und hat einen Rollbalken; ist unten kein
Platz, aber oben, klappt es NACH OBEN auf (`dropdown_popup_rect` = EINE Quelle,
kennt die Bildschirmhoehe ueber `Gui::schirm_h` aus GUI_UPDATE) und rollt dann
von unten herauf. Offen bewegen Pfeile/Bild/Pos1/Ende/Tippen nur `dd_mark`,
Enter/Leertaste uebernimmt, **ESC verwirft** -- vorher setzte jeder Pfeil die
Auswahl sofort und feuerte on_change. Beim Oeffnen steht die Auswahl markiert
in der MITTE. Zu waehlt Tippen gleich (`dd_tippen`). Neu
`GUI_DROPDOWN_PLACEHOLDER` (gedaempft, solange `sel` < 0; `.dhform`
`placeholder`), im Form-Designer ein Feld "Platzhalter". Tests
`tests/pruef/gui/gui_klappliste_bedienung.dhtest` (10; gegen den Bau davor fallen 8,
die zwei uebrigen pruefen, was gleich bleiben soll). Folge: ein Test, der einen Eintrag
jenseits der zehnten Zeile an fester Lage anklickte (Notenblatt, Instrument
"Glocke" = Eintrag 10), rollt jetzt erst einen Radschritt.

**Menues bequem wie die Liste** (selber Tag): **Kontextmenues per Tastatur**
-- `menue_tasten` fragte nur Leistenmenues (`tiefstes_popup` = Kontext ODER
Leiste), und ESC im offenen Kontextmenue drueckte den ABBRECHEN-Knopf des
Fensters, weil das Menue die Taste nicht nahm (gegen den alten Bau belegt).
Pos1/Ende, Tippen springt (Anfangsbuchstabe; eindeutig = gleich ausloesen
ueber `menue_ausloesen`, sonst zum naechsten Treffer). Am Rand weichen Popups
aus: in `popup_chain` (Kontextmenue links/ueber der Maus, Untermenue links vom
Eltern, Leistenmenue nach links; `Gui::schirm_b/schirm_h`). **Die Falle:**
gezeichnet wurde mit den ROHEN Lagen (`context_open`, `sub_chain`), nur der
Treffertest ging ueber `popup_chain` -- beide Zeichenstellen gehen jetzt
darueber. Tests `tests/pruef/gui/gui_menue_bedienung.dhtest` (8; gegen den Bau
davor fallen alle 8).

**Code-Feld bequem wie die Liste** (selber Tag, Textbereich): **Doppelklick
waehlt das Wort, Dreifachklick die Zeile** -- das gab es gar nicht
(`Widget::klick_n/klick_zeit/klick_idx`, 0,4 s an derselben Stelle;
`wort_um` nimmt `_` und `$` mit; `wort_zug` sperrt das Zusammenziehen beim
Halten). **Rechtsklick setzt die Schreibmarke** (`ta_rechtsklick` ueber
`ta_index_at`, dieselbe Rechnung wie der Klick; in der Auswahl bleibt sie,
die Nummernspalte bleibt dem Haltepunkt). **Rollbalken** im rechten
Innenabstand (`ta_bar_geom`, 5 px, ueber keinem Zeichen -- die Umbruchbreite
bleibt unveraendert), gegriffen im Hover-Durchlauf VOR dem Editieren
(`farbfeld_zug` sperrt den Druck fuer die Schreibmarke), gezogen ueber
`Gui::tabar_zug`; `rad_stand` haelt die Ansicht gegen die Schreibmarke.
**Weiches Rad** in Zeilenschritten (`roll_ist/roll_setze` kennen `scroll` des
Textbereichs). Tests `tests/pruef/gui/gui_codefeld_bedienung.dhtest` (9; die
Klicklagen sucht die Beilage `stelle.dh` zur Laufzeit mit
`GUI_TEXTAREA_POS_AT` -- `GUI_TEXTAREA_VIEW` zaehlt die erste Zeile ab 0).

**Textfeld bequem, Textbereich mit Inline-Formaten** (selber Tag): das
Textfeld nimmt Doppelklick (Wort), Dreifachklick (alles; im Passwortfeld
schon der Doppelklick -- Wortgrenzen verrieten die Leerzeichen) und
Rechtsklick (`ti_rechtsklick`, in der Auswahl bleibt sie) mit denselben
Feldern wie der Textbereich. **Inline-Formate:** `GUI_TEXTAREA_SET(ta,
"formatiert", 1)`, je Zeichen Stil-Bits aus `schnitt` (`Widget::stile`),
Strg+B/I/U auf die Auswahl (alle haben es -> weg, sonst dran) oder als
`tipp_stil` fuer das naechste Getippte (verfaellt, sobald getippt oder die
Marke bewegt ist), `GUI_TEXTAREA_STYLE/GET_STYLE$/MARKDOWN$/SET_MARKDOWN`,
`.dhform` `formatiert` + `markdown`. **Keiner der Textwege kennt die Bits**
-- `stile_text` merkt, zu welchem Text sie gehoeren, und `stile_abgleichen`
zieht sie ueber gemeinsamen Anfang/Ende nach (in `edit_textarea`,
`textarea_insert`, `text_undo` und je Bild in `uebergaenge`); das Neue
bekommt den Tippstil oder den Stil davor. Der Verlauf traegt die Stile mit
(`undo: Vec<(String, i32, Vec<u8>)>`), eine Formataenderung ist ein eigener
Schritt. **Gezeichnet nachgebildet** (`Graphics::text_nachgebildet` ->
`Cmd::TextStil` auf der Grundschrift): ein echter fetter Schnitt waere
breiter, und Marke, Auswahl, Klick messen am ungeformten Text. Markdown
(`markdown_aus`/`markdown_ein`): Marken SCHALTEN um, geschrieben werden nur
Wechsel, fett+kursiv in einem Zug (`**` gefolgt von `*` liest sich als `***`
= beide kippen -- mit Umschalten stimmt das trotzdem), `\` fuer `* ~ \ <u>`.
Tests `tests/pruef/gui/gui_textfeld_formate.dhtest` (12; Gegenprobe ohne
Klickzaehlung, ohne Rechtsklick, mit Tippstil, der nie verfaellt: genau die
5 betroffenen fallen), Rust-Tests `formate_tests`. Getter des Texts ist
`GUI_TEXT`, nicht `GUI_GET_TEXT`.

**Klammern schliessen** (selber Tag, erster Punkt der Editor-Liste aus der
Uebersicht): `GUI_TEXTAREA_PAIRS(ta, paare$)` (je zwei Zeichen ein Paar,
leer = aus; `Widget::paare`, `Gui::paar_tippen` statt `an_marken` fuer
GETIPPTE Zeichen -- Einfuegen geht absichtlich vorbei). Oeffnend setzt das
schliessende dahinter, gleich schliessend tritt darueber, eine Auswahl wird
umschlossen, die Ruecktaste zwischen einem leeren Paar nimmt beide; gepaart
wird nur vor Leerraum/Zeilenende/Satzzeichen/schliessendem Zeichen,
Anfuehrungszeichen nicht hinter einem Wortzeichen und nicht bei ungerader
Zahl davor in der Zeile. Die IDE: `klammernAn` (ide.json `klammern`,
Vorgabe an, `'` fehlt -- Kommentar), Kaestchen in einer ZWEITEN Spalte der
Einstellungen (das Fenster waechst nach rechts, die linke Kante bleibt --
sonst saessen alle bisherigen Klicklagen daneben), Palette. **Getippt wird
im Test mit echten WM_CHAR-Nachrichten** (`fenstersender.ps1` kennt jetzt
Folgen `tippe:...|anf|rueck|links|rechts`; GLFW nimmt die Taste aus dem
SCANCODE im lParam) -- eine Aufnahme fuellt die Zeichenwarteschlange nicht.
Tests `tests/pruef/gui/gui_klammern.dhtest` (10; jeder Fall kann nur MIT Paaren
bestehen -- die erste Fassung tippte `f(x)` ganz aus und waere ohne sie
genauso gruen gewesen), `tests/pruef/ide/werkzeug_ide_handgriffe.dhtest`.

**Dieselbe Datei nebeneinander** (selber Tag, vierter Punkt; zweiter und
dritter: Formatieren beim Sichern -- aus per Vorgabe, nie beim
automatischen Sichern, `CODE_FORMAT$` verweigert nur Unlesbares wie eine
offene Zeichenkette, eine halbe Anweisung rueckt es ein -- und Alle
Fundstellen, Strg+Umschalt+F12, `benutztWort` in jeder Schreibweise):
`GUI_TEXTAREA_SHARE(ansicht, von)` (`Widget::teil_von/teil_stand`,
`Gui::teil_pass` gleich nach dem Editieren in `update` -- wer danach den
Besitzer liest, sieht den Stand DIESES Bildes). Geaenderte Seite gewinnt:
die Ansicht schreibt samt Verlaufsschritt an den Besitzer, sonst zieht sie
nach; die Marke der anderen Seite rueckt ueber gemeinsamen Anfang/Ende
(`marke_nachziehen`). **Der Verlauf liegt nur beim Besitzer**, Strg+Z in
der Ansicht geht dorthin -- mit zwei Verlaeufen haette jede Seite eine
eigene Geschichte und nahm die Aenderung der anderen nicht zurueck. Die
IDE: `taSpiegel` links, bei jedem Einschalten `feldEinrichten`, folgt dem
Reiter (`spiegelNachziehen` je Bild), eigener Faerbe-Riegel
(`spiegelFaerbSchl`), `aktivesFeld()` meint die Ansicht, wenn sie den Fokus
hat. Tests `tests/pruef/gui/gui_teilen.dhtest` (5; der Strg+Z-Fall besteht nur
mit der Umleitung) und in `werkzeug_ide_handgriffe.dhtest`.
Die uebrigen Punkte derselben Liste, alle in der IDE und in
`werkzeug_ide_handgriffe.dhtest`: **Seitenleiste/untere Leiste** (Alt+1/2,
`leistenSetzen`; die untere kommt bei jedem `prozessStarten` zurueck),
**Profil nach Funktionen** (Kaestchen im Profilfenster, innerster Block aus
`CODE_SYMBOLS$`; Protokollzeilen heissen `profiltabelle ...`, weil alte
Tests die erste Zeile mit `profil ` lesen) und **Ausgabe leeren**,
**Faltung je Datei** (ide.json `falten_pfade`/`falten_zeilen`, zurueck in
`faltenZurueck` nach GUI_TEXTAREA_FOLDABLE -- Falle: `GUI_TEXTAREA_FOLDED`
fragt, ob eine Zeile VERBORGEN ist, die Kopfzeile ist es nie; ob gefaltet
wurde, sagt der Rueckgabewert von `GUI_TEXTAREA_FOLD`), **Druckoptionen**
(Schrift, Rand, Nummern, nur die Auswahl; mit den Vorgaben dasselbe Listing,
die Breite wird jetzt mit PDF_TEXT_WIDTH gemessen -- 92 feste Zeichen liefen
ueber den Rand), **Filter ueber dem Projektbaum** (in der Zeile des
Projektnamens, damit die Klicklagen des Baums bleiben) und **Umbau
wiederholen samt Probe**: das Zuruecknehmen verweigert sich, wenn eine
Datei seit dem Umbau geaendert wurde (`zurNachher`, `umbauGeaendert$`) --
vorher schrieb es die alten Texte blind zurueck. Die Reiternummer eines
Umbaus zaehlt nur noch ohne Weg (`umbauReiter`). Beispiele nach Themen
kamen danach (siehe unten).

**Form-Designer: Mehrfachauswahl, Anordnen, Reiterseiten** (2026-09-23):
`gewaehlt` (alle Indizes) neben `auswahl` (Bezug = zuletzt angeklickt); jede
Stelle, die frueher `auswahl = i` schrieb, geht ueber `auswahlSetzen`,
nach Rueckgaengig raeumt `auswahlBereinigen`. Umschalt/Strg+Klick schaltet,
ein Zug ueber leere Flaeche ist ein Auswahlrahmen (modus 3; ein Control auf
einer verborgenen Reiterseite liegt nicht darin), Strg+A NICHT am Menue --
ein Strg-Kuerzel gewaenne auch im Inspektor-Feld. Ziehen/Pfeile/Entf/Strg+D
fuer alle (Loeschen von hinten nach vorn), Griffe nur bei einem. Menue
**Anordnen** (`anordnen`): ausrichten/gleiche Masse am Bezug, Verteilen ab
drei mit gleichen Luecken (erstes und letztes stehen). **Reiterseiten:**
Feld `reiter` = `NAME:SEITE` (ab 1) je Control, `reiterNachziehen` (am Anfang
von `ansichtBauen`) baut daraus `tabctl.kinder` fuer Reiterwerk UND
Assistent -- die Laufzeit fuehrt Nummern, die sich bei jedem Loeschen
verschieben, der Designer Namen; `reiterAusDatei` beim Laden umgekehrt
(Namenlose bekommen einen), ein umbenanntes Reiterwerk nimmt seine Kinder
mit. DH-Code: `GUI_TABCONTROL_ADD` je Seite, die Zuordnungen NACH allen
Controls (`dhVarNamen`). Dazu Kaestchen *Tippen erlaubt* (Combobox) und
**[Neu]/[Oeffnen] fragen bei Ungesichertem** (`ersetzenAnfragen`, wie im
Notenblatt; bis dahin warfen sie es wortlos weg). Tests sieben Faelle in
`tests/pruef/werkzeuge/werkzeug_formdesigner.dhtest`; fuenf Verfaelschungen fallen je in
ihrem Fall.

**Debugger: Aufrufstapel, Ueberwachen, bis zur Marke, Wert setzen**
(2026-09-23): `dhrt debug` fuehrt je laufender Funktion (Name, Zeile des
AUFRUFS) in `DebugState::stapel` -- geschoben in `exec`/`exec_byref`, nur
wenn der Debugger laeuft; `paused` traegt `stack` (innen zuerst) und
`watches`. Neue Kommandos `set-watches` (antwortet gleich mit einem
`watches`-Ereignis, im Halt will man den Wert jetzt sehen), `run-to` (ein
Haltepunkt, der beim NAECHSTEN Halt vergeht -- auch wenn ein anderer
Haltepunkt zuerst trifft, sonst hielte es spaeter unerwartet) und `set`
(Ausdruck, gewandelt mit `coerce` wie eine Zuweisung; erst lokal, dann
global; `debug_on_line` bekommt dafuer die Locals veraenderlich). Eine
`CONST` ist fuer den Debugger nicht da (eingesetzt), `set` meldet "nicht
gefunden". Die IDE: Stapel und Ueberwachen rechts neben den Variablen,
Doppelklick auf eine Variable = `GUI_PROMPT` -> `debugSetzen`, Strg+F10 =
`bisZurMarke` (ohne laufenden Debugger startet er ihn, `debugBis` wird beim
ersten Halt eingeloest). **Nebenfund in der gui:** F10 oeffnete die
Menueleiste auch, wenn ein Menue-Kuerzel die Taste schon genommen hatte (die
IDE: F10 = Schritt drueber) und mit Strg -- jetzt nur ohne Strg/Umschalt und
ohne gefeuertes Kuerzel. Tests zwei Faelle in `tests/pruef/werkzeugkette/dhrt_debug.dhtest`
(Protokoll), drei in `werkzeug_ide_handgriffe.dhtest`; drei Verfaelschungen
der IDE (ohne Stapel, `debugBis` uebergangen, Setzen ohne Wirkung) fallen je
in ihrem Fall.

**Bearbeitbare Klappliste** (2026-09-23): `GUI_DROPDOWN_SET(dd, "bearbeitbar", 1)`
-- eine Combobox: getippt wird ueber `edit_textinput` (dieselbe Routine wie
das Textfeld), der Text steht in `w.text`, `sel` folgt ihm
(`dd_frei_abgleichen`: der Eintrag, der genau so heisst, Gross/klein egal,
sonst -1). Die Liste SCHLAEGT VOR (klappt auf, markiert den ersten, der so
anfaengt), **uebernommen wird nur mit den Pfeilen** (`Gui::dd_pfeil`) --
ein Vorschlag, der bei Enter von selbst gewoenne, ersetzte "Rotwein" durch
"Rot". Enter mit offener Liste gehoert der Liste (`liste_taste`), sonst
schickt es ab wie aus einem Textfeld. Nur der Pfeil (`DD_PFEIL_B`) klappt
per Klick auf; gezeichnet ueber `zeichne_textfeld` (aus dem Textfeld-Arm
herausgeloest, `rechts` = Platz fuer den Pfeil). Wer per Tippen aufklappt,
merkt sich selbst als frisch geoeffnet (`dd_auf_von`), sonst setzte das
naechste Bild die Markierung auf die Auswahl zurueck. **Nebenfund:** ESC in
einer offenen FESTEN Klappliste schloss sie UND drueckte den
Abbrechen-Knopf -- jetzt verbraucht (`liste_taste`). `.dhform`
`bearbeitbar`. `fenstersender.ps1` kennt dafuer `auf/ab/enter/escape` in
Folgen, und Taste runter/hoch liegen 160 statt 80 ms auseinander (unter Last
kamen beide im selben Bild an, raylib sah keinen Druck). Tests
`tests/pruef/gui/gui_combobox.dhtest` (10; drei Verfaelschungen -- Enter nimmt
jeden Vorschlag, ESC nicht verbraucht, Abgleich mit Gross/klein -- lassen
je ihre Faelle fallen).

**Text und Formular** (2026-09-04, Punkt 1 des gui-Ausbaus -- Ziel: dass Drachenhauch genannt wird, wenn jemand fragt, womit er eine Anwendung schreiben soll): `GUI_SET_ALIGN(wdg, links|mitte|rechts)` fuer Beschriftung/Knopf/Textfeld; `GUI_SET_WRAP(label, breite)` bricht an Wortgrenzen um, die HOEHE folgt dem Text -- gemessen in `umbruch_layout` (in GUI_UPDATE, weil nur dort Graphics und Schreibzugriff zusammenkommen; das Zeichnen ist `&self`); `GUI_TEXTINPUT_SET(tf, key$, wert)` mit `passwort` (Punkte statt Zeichen -- Treffertest und Rollen messen an den PUNKTEN, sonst sitzt die Schreibmarke neben dem Text), `nur_lesen`, `maxlaenge` (schneidet ab, auch beim Einfuegen), `zahlen` (1 ganz, 2 Komma; Zwischenstand `-` erlaubt, sonst liesse sich keine negative Zahl tippen); `GUI_ENTERED`/`GUI_ON_ENTER`; **Strg+Z/Y in Textfeld und Textbereich** (Anschlaege innerhalb 0,8 s = EIN Schritt; `GUI_SET_TEXT` leert den Verlauf); `GUI_WINDOW_DEFAULT`/`GUI_WINDOW_CANCEL` (Enter/ESC druecken den Knopf -- aber die Taste gehoert zuerst dem Widget mit Fokus: Knopf/Kaestchen nehmen Enter selbst, Textbereich macht einen Umbruch, Zelle in Bearbeitung ihr Ende; aus einem TEXTFELD heraus ist Enter das Abschicken; der Standard-Knopf traegt den Akzent als Rahmen). Alles in der `.dhform` (auch der Tooltip -- der fehlte dort bisher) und im Form-Designer (Inspector + Codegen). **Testfalle:** raylibs Wiedergabe legt Tasten in die Tastenwarteschlange, aber KEINE Zeichen in die Zeichenwarteschlange -- Tests tippen ueber die Zwischenablage mit Strg+V, das laeuft durch dieselben Filter. Tests `tests/pruef/gui/gui_text_formular.dhtest`, Doku `docs/module-gui.md`.

**Menues** (2026-09-04, Punkt 2 des gui-Ausbaus): `GUI_MENU_ITEM(menu, label$[, kuerzel$])` / `GUI_MENU_SHORTCUT` -- Kuerzel werden als Text geschrieben (`Strg+S`, `Alt+Enter`, `F5`, `Entf`, deutsch oder englisch; `kuerzel_parsen` in gui.rs mit Rust-Tests, unbekannte Taste = Fehler beim Anlegen statt eines still stummen Kuerzels) und jedes Bild geprueft (`kuerzel_pruefen`), auch bei geschlossenem Menue -- **seit 2026-09-07 in ALLEN sichtbaren Fenstern**: zuerst im Fokus-Fenster, dann in den uebrigen von oben nach unten (Form-Designer, Anim-FSM und Notenblatt mussten sich vorher nach jedem Knopf im Nebenfenster den Fokus zurueckholen, sonst war Strg+S stumm -- dreimal derselbe Fund, dreimal zuerst vom Test gesehen); das Fokus-Fenster gewinnt bei gleichem Kuerzel, ein modales laesst nur seine eigenen zu, ein Entwurfsfenster (`GUI_WINDOW_DESIGN`) zaehlt nicht; die Modifier muessen GENAU passen, und **ohne Strg/Alt gehoert die Taste dem Textfeld mit Fokus** (ein `Entf`-Kuerzel loescht dort ein Zeichen) -- AUSSER F1..F12 (`ist_funktionstaste`, seit 2026-09-06: die IDE in Drachenhauch startete per F5 nie, weil das Code-Feld den Fokus hatte). `GUI_SUBMENU` (beliebig tief; `sub_chain` = offene Untermenues, `untermenues_folgen` oeffnet beim Ueberfahren und schliesst NICHT, wenn die Maus neben allen Popups ist -- sonst klappt es beim schraegen Hinueberfahren zu; ein Untermenue ist nie Kontextmenue, `Menu::unter`), `GUI_MENU_CHECK`/`GUI_MENU_CHECKED` (Klick oder Kuerzel kippt), `GUI_MENU_ENABLE` (gesperrt = kein Klick, kein Kuerzel), `GUI_MENU_ICON`, `GUI_MENU_TEXT`. Popup-Layout aus EINER Quelle `popup_layout` (Treffertest + Zeichnen). In der `.dhform` verschachtelt (`items` am Eintrag, `shortcut`, `checkable`/`checked`); der Form-Designer bearbeitet Menues nicht, schreibt sie aber jetzt in den DH-Code (`_gb_menus`). **Testfalle:** raylib meldet beim Lesen mancher Aufnahmedateien "Issue reading line to buffer" auf stdout, die Ereignisse kommen trotzdem an -- Tests filtern `WARNING:`-Zeilen. Tests `tests/pruef/gui/gui_menu_ausbau.dhtest`, Beispiel `examples/129_gui_menu.dh`.

**Listen** (2026-09-04, Punkt 3 des gui-Ausbaus): Eintraege einzeln (`GUI_LISTBOX_ADD/REMOVE/CLEAR/COUNT/ITEM/SET_ITEM/MOVE`, auch fuer Klapplisten -- vorher ging nur `GUI_SET_LISTBOX` als Ganzes; die Auswahl rueckt beim Einfuegen/Loeschen davor MIT, sie meint denselben Eintrag), `GUI_LISTBOX_SET(lb, "kaestchen"|"mehrfachauswahl", 1)`, `GUI_LISTBOX_ICON/COLOR` je Eintrag, `GUI_LISTBOX_CHECKED/SET_CHECKED`, Mehrfachauswahl mit denselben Abfragen wie die Tabelle (`IS_SELECTED/SELECT/SEL_COUNT/SEL_ROW/CLEAR_SELECTION`; Strg+Klick sammelt, Umschalt+Klick spannt vom Anker, ein Pfeil setzt die Menge auf eine Zeile), `GUI_DOUBLE_CLICKED` (bisher nur Listen). Zusatzzustand in `ListState` (`Widget::list`, nur bei Bedarf angelegt -- eine schlichte Liste bleibt, was sie war; `sync` haelt die Vektoren mit `items` gleich lang). **Ein Klick aufs Kaestchen kippt NUR den Haken**, die Auswahl bleibt -- sonst waehlte man beim Abhaken jedes Mal um. `handle_press` hat kein `g`, Strg/Umschalt kommen deshalb ueber `tasten_mod`, das `update` vor dem Druck fuellt. In der `.dhform` unter `list` (Sinnbilder nicht -- Textur-Handles). Designer: zwei Schalter im Inspector, Codegen. Tests `tests/pruef/gui/gui_listen.dhtest` (Klicks mit Strg/Umschalt echt eingespeist; der Versatz kommt aus einer Zeichenflaeche bei (0,0)), Beispiel `examples/192_gui_listen.dh`.

**Dialoge** (2026-09-04, Punkt 4 des gui-Ausbaus): `GUI_DIALOG(t, x, "Speichern|Verwerfen|Abbrechen")` -- eigene Knoepfe (1..5), die Antwort ist die NUMMER des Knopfs; Enter = der erste, ESC/Schliessen = der letzte (Systemdialog-Konvention). `dialog_auswerten` zaehlt Knoepfe statt auf den Text "Nein" zu pruefen; Enter/ESC laufen ueber die Standard-/Abbrechen-Knoepfe aus Punkt 1 (`default_btn`/`cancel_btn`). `GUI_PROMPT(t, x[, vorgabe[, knoepfe]])` = derselbe Dialog mit Textfeld (Fokus im Feld, Vorgabe markiert), `GUI_DIALOG_TEXT` liest ihn auch nach der Antwort (Tombstone-Fenster behaelt seine Widgets). `GUI_WINDOW_MODAL(win, an)` schaltet ein EIGENES Fenster modal -- `dialog_auswerten` nimmt Nicht-Dialoge aus (sonst beendete jeder Knopf darin das Fenster) und gibt frei, wenn das Fenster ausgeblendet oder zerstoert wird. Gemeinsamer Kern `dialog_frei`. Tests `tests/pruef/gui/gui_dialoge_ausbau.dhtest` (die Knopflage kommt aus einer 1x1-Zeichenflaeche IM Dialog), Beispiel `examples/193_gui_dialoge.dh`.

**Layout** (2026-09-04, Punkt 5 des gui-Ausbaus): `GUI_AUTOSIZE(wdg)` und `0` als Breite/Hoehe beim Anlegen = Mass nach Inhalt (`auto_w`/`auto_h`, gemessen im `layout_pass` in GUI_UPDATE; `GUI_SET_BOUNDS` hebt es auf). `GUI_LAYOUT(win, zeile|spalte|raster:N, x, y, w, h)` = neue Widget-Art `Kind::Layout` mit `LayoutState` (Kinder als Widget-Indizes, -1 = `GUI_LAYOUT_SPACER`), `GUI_LAYOUT_ADD(layout, wdg[, gewicht])` (Gewicht 0 = eigene Groesse, sonst Anteil am Rest; ein Widget in hoechstens EINEM Behaelter, kein Kreis), `GUI_LAYOUT_SET` (abstand, rand, ausrichtung, dehnen, rahmen). `layout_anwenden` laeuft je Bild von jedem WURZEL-Behaelter rekursiv; Beschriftungen darin werden immer exakt gemessen (die `GUI_LABEL`-Schaetzung `chars*8` stimmte fuer Zeilen nie). **Ein Behaelter ist Luft fuer Klicks** -- aus den drei Treffertests (Hover, Druck, `GUI_HIT_TEST`) ausgenommen, sonst schluckte er die Klicks seiner Kinder, die in der Reihenfolge NACH ihm kommen; nicht fokussierbar. Anker gelten fuer den Behaelter, die Kinder bekommen ihre Lage von ihm (relayout setzt sie, der naechste layout_pass korrigiert). In der `.dhform`: `layout` mit `kinder` als `[Index, Gewicht]`; der Form-Designer fuehrt die Kinder intern an NAMEN (`layout_von`/`layout_zuordnen`, Umrechnung nur an der Dateigrenze -- so ueberleben Zuordnungen Loeschen und Umsortieren ohne Index-Buchfuehrung), Paletteneintrag, Inspector (Art/Masse am Behaelter, "Layout"+"Gewicht" an jedem Control), Codegen. Bewusst NICHT: Constraints, Mindestgroessen. Tests `tests/pruef/gui/gui_layout.dhtest` (headless ueber GUI_GET_X/Y/W/H), Beispiel `examples/194_gui_layout.dh`.

**Feinschliff** (2026-09-05, Punkt 6 und Abschluss des gui-Ausbaus): `GUI_VSLIDER(win, x, y, h, min, max, default)` (Slider mit `vert`, Wert waechst nach oben; `drag_slider` bekommt jetzt auch `my`), `GUI_PROGRESS_SET(p, "unbestimmt", 1)` (Band aus `g.get_time()`), `GUI_IMAGE_MODE` (strecken/einpassen/fuellen/mitte/kacheln, `draw_bild` mit Clip), `GUI_TREE_ICON/COLOR` (TreeNode `icon`/`color`; sobald ein Knoten ein Sinnbild hat, rueckt JEDE Zeile ein), **rollendes Panel** `GUI_PANEL_ADD/REMOVE/SCROLL/SCROLL_GET` (`PanelState` am Panel, `panel_von` am Kind -- je Bild in `panel_pass` gesetzt, durch Layouts hindurch; der Scroll ist NUR ein Blick-Versatz in `abs_rect`, die Kinder behalten ihre Fensterlage; `im_panel_sichtbar` nimmt Herausgerolltes aus Hover/Druck/GUI_HIT_TEST; ein rollendes Panel ist wie ein Layout Luft fuer Klicks, weil `handle_press` den ERSTEN Treffer nimmt und das Panel vor seinen Kindern liegt; Mausrad NACH den Kindern abgefragt, weil `pop_mouse_wheel` nur einmal liefert; kein Panel im Panel), **Ziehen** `GUI_DRAGGABLE/DROP_TARGET` + `GUI_DRAGGING/DROPPED/DROP_TEXT/DROP_SOURCE/DRAG_INDEX/DROP_INDEX` (`DragState` ab 5 px Bewegung, `DropInfo` transient wie `clicked`; Nutzlast = Listenzeile/Baumknoten/Widget-Text; innerhalb derselben Liste sortiert `ablegen` selbst um; Schatten + Ablage-Rahmen in `draw_top`), **Cursorformen** `cursor_pass` (setzt nur bei Wechsel und nimmt nur zurueck, was sie selbst war -- `MOUSE_CURSOR` nach GUI_UPDATE gewinnt; `GUI_CURSORS(FALSE)` aus; nicht testbar, raylib liest die Form nicht zurueck). `.dhform`: `vertical`, `indeterminate`, `mode`, `draggable`, `drop_target`, `panel{kinder,scroll}`; Designer: Inspector-Felder + "Panel"-Zuordnung ueber dieselbe Namens-Buchfuehrung wie Layout (`layout_von(c, "panel")`), Codegen. Tests `tests/pruef/gui/gui_punkt6.dhtest` (Zuege echt eingespeist; Mausrad = Ereignis 8 mit params[1] = y), Beispiel `examples/195_gui_feinschliff.dh`.

**Mindestmasse und Formularpruefung** (2026-09-05, aus dem sechsten Piloten): `GUI_SET_MIN_SIZE(wdg, w, h)` -- ein gewichtetes Kind im Behaelter faellt nie darunter (die Zeile laeuft lieber ueber), ein verankertes Widget schrumpft nicht darunter; `GUI_LAYOUT_MIN_W/H(layout)` = Mass des Inhalts (`min_mass`, rekursiv; feste Kinder mit ihrer NATUERLICHEN Groesse `nat_w/nat_h` -- `w/h` tragen nach dem ersten Durchlauf die gedehnte Groesse, daraus liesse sich kein Minimum ableiten; gewichtete mit ihrem Mindestmass), der Wert fuer `WINDOW_MIN_SIZE`. **Formularpruefung:** `GUI_RULE(feld, art$[, a, b][, muster$][, meldung$])` (pflicht/zahl/ganz/bereich/laenge/email/datum/muster; an Klappliste und Kaestchen nur pflicht; die Argumentzahl haengt an der Art, der letzte Text ist die Meldung), `GUI_VALIDATE(win)` (Zahl der Fehler, prueft nur sichtbare + bedienbare Felder -- ein Fehler auf einem anderen Reiter waere einer ohne Ausweg; erstes falsches Feld bekommt den Fokus), `GUI_VALIDATE_WIDGET`, `GUI_ERROR/SET_ERROR/CLEAR_ERRORS`, `GUI_ERROR_LABEL` (Beschriftung zeigt die Meldung; `fehler_setzen` ist die EINE Stelle), `GUI_VALIDATE_LIVE` (beim Verlassen des Feldes, nicht je Anschlag), `GUI_RULES_CLEAR`. Roter Doppelrahmen am Feld, Meldung im Tooltip vor dem Hilfetext. **Leer laesst jede Regel ausser pflicht durch** -- sonst hiesse "bereich 1 10" zugleich "Pflichtfeld". Reine Regel-Logik `regel_pruefen` mit Rust-Tests; die regex-Fehlermeldung ist mehrzeilig, nur die letzte Zeile kommt in die Meldung. `.dhform`: `min_w/min_h`, `rules`, `error_label`; Designer: zwei Drehfelder + Zeile "Regeln" (`regeln_parsen`/`regeln_text`, Tippfehler werden uebergangen), Codegen. Bewusst NICHT: eigene Regel als FUNCREF (Programm prueft nach GUI_VALIDATE selbst und meldet ueber GUI_SET_ERROR). Der Pilot 196 benutzt beides statt seiner Handarbeit. Tests `tests/pruef/gui/gui_pruefung.dhtest`, Mindestmasse in `tests/pruef/gui/gui_layout.dhtest`.

**Zeilenumbruch und Datenbindung** (2026-09-05, die letzten zwei Punkte der Lueckenliste): `GUI_TEXTAREA_SET(ta, "umbruch", 1)` -- der Textbereich bricht an Wortgrenzen um (`ta_rows` = EINE Quelle sichtbarer Zeilen `(logische Zeile, von, bis)` fuer Zeichnen, Klick, Pfeile, Schreibmarke und Scroll; `ta_row_of`: am Umbruch gehoert die Marke zur NEUEN Zeile; Pos1/Ende/Pfeile laufen in sichtbaren Zeilen; Zeilennummern nur an der ersten Zeile eines Absatzes; `GUI_TEXTAREA_VIEW` zaehlt weiter logische Zeilen; `scroll_x` bleibt 0). Test-Falle: es gibt keine Schreibmarken-Abfrage -- ein eingefuegtes `#` (Strg+V) verraet die Lage, und nach GUI_SET_TEXT steht die Marke am ENDE, also erst sechsmal Pfeil hoch. **Datenbindung:** `GUI_BIND(wdg, schluessel$[, formular$])`, `GUI_FORM_GET/SET/CLEAR/CLEAN/CHANGED(win[, formular$])`, `GUI_FORM_LOAD/SAVE(win, db, tabelle$, id[, formular$])` (in vm.rs, weil nur die VM db UND gui hat; `#[cfg(feature = "db")]`; Spalte heisst `id`; Tabellen- und Schluesselnamen nur `[A-Za-z0-9_]`). `wert_text` (Vergleich/MAP) und `wert_typisiert` (Datenbank: Zahlenfeld -> Zahl, ausser fuehrende Null wie eine PLZ -> Text; Kaestchen -> BOOL; Klappliste -> Index; Regler -> FLOAT), `wert_setzen` tolerant ("ja"/1/TRUE, Eintragstext oder Index). `Window.form_stand` = sauberer Stand (beim Binden, nach SET/LOAD/SAVE/CLEAR/CLEAN). Der Pilot 196 laedt/speichert Kunden ueber die Bindung (keine Spaltenliste mehr) und nutzt sie beim Artikel nur fuer "geaendert?", weil er Preis und MwSt umrechnet. `.dhform`: `wrap_text`, `bind`, `form`; Designer: Felder "Bindung"/"Formular", Kaestchen "Zeilenumbruch", Codegen. Bewusst NICHT: Listen an Tabellen binden, Fremdschluessel, Verbunde. Tests `tests/pruef/gui/gui_bindung.dhtest` (bis 2026-09-16 pytest mit sqlite3; seither liest ein zweiter Prozess die Datei mit `typeof()`), `tests/pruef/gui/gui_umbruch.dhtest` (seriell).

**Drucken und OPENDOC** (2026-09-05, Wege A und C aus `docs/entwurf-drucken.md`): das pdf-Modul zeichnet seine Befehle jetzt AUF (`pdf::Op` je Seite, parallel zum Inhaltsstrom) und spielt sie auf drei Ziele: `PDF_SAVE` (wie immer), `PDF_PRINT(p[, drucker$[, kopien[, zieldatei$]]])` und `PDF_PREVIEW(p, seite[, breite_px]) -> IMAGE` (graphics.rs, raylibs Standardschrift). `drucken.rs` (ungated): Windows = GDI ueber das `windows`-Crate (schon im Baum, jetzt direkte Abhaengigkeit unter `[target.'cfg(windows)']` mit Gdi/Printing/Xps/Shell-Features) -- `CreateDCW` auf den Drucker, `StartDocW` mit `lpszOutput` = zieldatei, mm -> Geraeteeinheiten ueber `GetDeviceCaps`, `PHYSICALOFFSET` abgezogen, `TA_TOP` weil unser y die Oberkante ist, Schriften Helvetica->Arial/Times->Times New Roman/Courier->Courier New; `EnumPrintersW` Level 4, `GetDefaultPrinterW`; macOS/Linux = `lp -d -n` mit temporaerer PDF, `lpstat -a/-d`, zieldatei = die PDF selbst. `OPENDOC(pfad$)` = ShellExecute/open/xdg-open mit Endungsliste (kein Programmstarter). **Gemessen:** zwei Seiten durch "Microsoft Print to PDF" in eine Datei ~0,9 s; im Ergebnis stehen Text und der rechtsbuendige Betrag an seiner Stelle (Test `tests/pruef/module/drucken.dhtest`, laeuft auch auf dem Windows-Runner der CI, weil Windows den Drucker mitbringt; gelesen mit dem eigenen Leser `tests/pruef/_hilfen/pdftext.dh`). Kopien = der Auftrag n-mal, nicht DEVMODE. Bewusst NICHT: fremde PDFs drucken, nativer Druckdialog, Duplex/Papierfach. Pilot 196: Druckdialog aus Bordmitteln (Klappliste PRINTERS, Kopien, PDF_PREVIEW auf GUI_IMAGE) + "PDF oeffnen".

**Das pdf-Modul schreibt ueber krilla** (2026-09-14, auf Frage des Nutzers nach Rust-Bibliotheken wie typst): nicht typst selbst -- ein ganzes Satzsystem mit eigener Markup-Sprache passte nicht zur Befehlsfolge PDF_TEXT/PDF_LINE --, sondern **krilla**, die Bibliothek, mit der typst seine PDFs schreibt. Vorher standen in `pdf.rs` 440 Zeilen Handarbeit, die nur die vierzehn Standardschriften ohne Einbetten und nur cp1252 konnten, dazu `pdf_masse.rs` aus einem PyMuPDF-Skript. Jetzt: die Seiten zeichnen weiter nur `Op`s auf, `bauen()` spielt sie auf ein `krilla::Document` (von oben gezaehlt wie krilla selbst, Text-Y + Schriftgroesse = Grundlinie). **Schriften eingebettet und beschnitten**: eingebaut DejaVu Sans/Serif/Sans Mono in je vier Schnitten (Crate `dejavu`, 5,1 MB Schriftdaten im Bau) als `sans`/`serif`/`mono[-fett|-kursiv|-fett-kursiv]`, `helvetica`/`times`/`courier` bleiben Namen dafuer; `symbol`/`zapfdingbats` sind ein Fehler mit Hinweis. Neu `PDF_FONT_LOAD(p, pfad$, name$)`. **Text ist Unicode**; ein Zeichen, das die Schrift nicht hat, ist ein Fehler (`rustybuzz::Face::glyph_index`), kein leeres Kaestchen. **`PDF_TEXT_WIDTH` formt mit rustybuzz genau wie krillas `draw_text`** (Unterschneidung eingeschlossen) -- gegen PyMuPDF auf denselben Dateien nachgemessen: Serif/Mono gleich auf vier Stellen, `sans` schmaler um die Unterschneidungspaare. **Weiter reproduzierbar**: krilla schreibt kein Datum und bildet die Dokumentkennung aus dem Inhalt (Rust-Test + `--- nochmal`-Fall). **Folge fuer Tests**: im Inhaltsstrom stehen Glyphennummern statt Text; zwei Werkzeug-Sammlungen und `pdf.dhtest` lesen deshalb ueber `pdftext.dh` (Beilage, in Drachenhauch: Objekte, Seitenbaum, Ressourcen, ToUnicode-Tabellen, TJ-Zeichenketten) statt BUFFER_INFLATE + INSTR; Gegenprobe mit falschem Suchwort faellt. GDI-Druck bildet sans/serif/mono auf Arial/Times New Roman/Courier New ab, geladene Schriften auf Arial. Doku `docs/module-pdf.md`, Buch `79_dokumente.js`. **Stolpersteine beim Leser:** `VAL("3 0 R")` ist 0 (fuehrende Zahl selbst lesen), und eine MAP laesst sich nicht verschachtelt abfragen (`tabellen[akt]` ist ein Fehler) -- ein flacher Schluessel `schrift:glyph`.

**Fenster als Prozess** (2026-09-05, Weg B aus `docs/entwurf-native-fenster.md`): `WINDOW_OPEN(datei$[, args...])` startet einen zweiten `dhrt run` mit eigenem SCREEN (`fenster.rs`, ungated wie `hintergrund.rs`); Kanal = TCP auf 127.0.0.1, Port ueber `DHRT_ELTERN_PORT`, NICHT stdin/stdout (das Kind darf weiter PRINT); `WINDOW_SEND/RECV$/ALIVE/CLOSE`, im Kind `PARENT_SEND/RECV$/ALIVE` (Verbindung beim ersten PARENT_*-Aufruf; `PARENT_ALIVE` = FALSE ohne Eltern, damit dasselbe Programm allein laeuft). `Leitung` teilen sich beide Seiten: begrenzte Warteschlange (1024, aelteste faellt weg, wie MIDI), vor der Verbindung Gesendetes wird vorgemerkt, eine Nachricht ist EINE Zeile (Umbruch = Fehler); faellt die Verbindung, beendet sich das Kind (`std::process::exit`); `Drop` der Eltern killt die Kinder. WINDOW_OPEN nimmt dem Kind `DHRT_FRAMES`/`DHRT_SCREENSHOT`/`DHRT_CONTACT*` aus der Umgebung -- sonst stuerbe es in Tests nach N Bildern. Gemessen: 0,4 s bis zum ersten Bild, 1000 Nachrichten im Schub ~60 ms, EINE Runde 17-50 ms = ein bis drei Bilder des Kindes (es liest einmal je Bild; der Entwurf verlangte < 5 ms fuer den Kanal, und der Kanal schafft das -- die Bildschleife nicht). Kein geteilter Zustand, wie TASK_START. Pilot: `196_rechnung_fenster.dh` (64 Zeilen) + Strg+F in 196. Tests `tests/pruef/fenster/fenster_prozess.dhtest` (seriell, zwei Prozesse) + Rust-Tests in fenster.rs.

**Barrierefreiheit** (2026-09-06, Wege A und C aus `docs/entwurf-barrierefreiheit.md`): gemessen vor dem Bau hatte ein dhrt-Fenster im UIA-Baum NULL Nachkommen -- fuer Bildschirmleser, Lupe und Sprachsteuerung ein Titel und sonst nichts. Jetzt schiebt das gui-Modul je Bild einen vollstaendigen Baum an **AccessKit** (`a11y.rs`; Crates `accesskit` 0.25 + `accesskit_windows` 0.35, im `graphics`-Feature): `Gui::a11y_baum` (Fenster als Group/Dialog, Widgets mit Rolle nach `Kind`, Menueleiste, Reiter, Listeneintraege, Tabellenzeilen/-zellen, Baumknoten; Knoten-Nummern in `a11y::ids`, eine Quelle fuer Bauen und Zerlegen), `Gui::a11y_aktion` (Focus/Click/SetValue/Increment/Expand auf DENSELBEN Wegen wie Maus und Tastatur, damit `GUI_CLICKED`/`on_click`/`on_change` feuern). **Drei Fallen:** (1) der Windows-Adapter verlangt ein Fenster, das noch nie sichtbar war -- `graphics.rs` legt es deshalb IMMER versteckt an (`builder.hidden()`), haengt den Adapter ein und zeigt es dann; (2) die Aktivierungs-Anfrage kommt in der Fensterprozedur (raylibs EndDrawing), wo die gui nicht greifbar ist -- der Handler antwortet `None` und merkt sich nur, dass gefragt wurde; laut Vertrag muss dann das naechste `update_if_active` einen VOLLEN Baum liefern (GUI_UPDATE tut das, FLIP schickt ohne gui einen Baum nur mit dem Fenster); (3) **LNK2005 `CloseWindow`/`ShowCursor`**: raylibs C-Funktionen heissen wie user32-Exporte, und die Import-Bibliothek des `windows`-Crates stand im Link vor raylib -- mit ZWEI windows-Fassungen (0.61 fuers Drucken, 0.62 aus AccessKit) brach der Link; dhrt ist deshalb auf `windows` 0.62 gehoben, eine Fassung, und es linkt. Beschriftung eines Eingabefelds = die Beschriftung links daneben oder darueber (`a11y_beschriftung`, `labelled_by`), sonst der Tooltip. Baum nur, wenn ein Hilfsprogramm fragt (`a11y_aktiv`). Neu: `GUI_ANNOUNCE(text$[, dringend])` (Live-Knoten; wechselndes Leerzeichen, damit dieselbe Ansage zweimal eine Aenderung ist), `GUI_SCREENREADER()`, `GUI_SET_TAB_INDEX(wdg, n)` (Index > 0 zuerst, Rest in Anlege-Reihenfolge; `.dhform` `tab_index`, Designer-Drehfeld). Weg A dazu: **Menueleiste per F10 oder Alt allein** (`menue_tasten`, `menu_cursor` im tiefsten offenen Menue, Pfeile/Enter/ESC; solange ein Leistenmenue offen ist, gehoeren die Tasten dem Menue; Alt+X loescht den Alt-Merker ueber `key_any_pressed_except_alt`), **Tooltip bei Tastaturfokus** (`tip_fokus`, unter dem Widget, die Maus gewinnt), helles Thema: gedaempfter Text 2,76:1 -> 6,3:1, Akzent 3,8 -> 5,6:1. **Pruefstein:** `tests/pruef/gui/gui_barrierefreiheit.dhtest` (bis 2026-09-17 pytest) laesst einen FREMDEN Leser (UIA aus PowerShell, `System.Windows.Automation`, keine neue Abhaengigkeit) Namen zaehlen, den Knopf klicken, Text setzen, Fokus setzen -- laeuft auch auf dem Windows-Runner. Bewusst NICHT (noch): Schreibmarke zeichenweise, `ui`-Modul, Web. SPEAK (Weg B) kam am selben Tag, siehe unten. Pilot 196 sagt seine Statuszeile an. **macOS/Linux-Adapter (selber Tag):** `a11y.rs` hat je System einen `mod plattform` mit EINEM Vertrag (`neu/aktiv/aktionen/senden/fenster_lage/fenster_fokus`) -- macOS `accesskit_macos::SubclassingAdapter::for_window` auf GLFWs `NSWindow` nach `add_focus_forwarder_to_window_class("GLFWWindow")`, Fokus ueber `update_view_focus_state`; Linux `accesskit_unix::Adapter::new` (Handler muessen `Send` sein, D-Bus-Faden; `DeactivationHandler` setzt `aktiv` zurueck), Lage/Fokus meldet `a11y_bild_ende` je FLIP bei Aenderung (`set_root_window_bounds`, aussen = innen, unter Wayland 0/0). Damit sich das OHNE raylib fuer ein fremdes Ziel pruefen laesst, ist `a11y` ein eigenes Feature (graphics schaltet es ein; `mod a11y` mit `#[allow(dead_code)]`): `cargo check --features a11y --target aarch64-apple-darwin` bzw. `x86_64-unknown-linux-gnu` (rustup-Ziele nachinstalliert), und die CI macht denselben Check je Laeufer. Uebersetzt, nicht gelaufen -- kein Mac, kein Linux hier; die Doku sagt das. Nicht fuer emscripten (das ist auch `unix`): eigener cfg-Ausschluss.

**Sprachausgabe `SPEAK`** (2026-09-06, Weg C aus `docs/entwurf-speak.md`, `sprache.rs`): gemessen vor dem Bau -- Windows hat ZWEI Sprachausgaben: SAPI (`ISpVoice`, spricht am Mischer vorbei, nur die alten Desktopstimmen Hedda/Zira) und WinRT `Windows.Media.SpeechSynthesis` (Stefan, Katja, Hedda; liefert einen **WAV-Strom**, mono 16 kHz, 14 Woerter in 35 ms beim ersten und 12-13 ms bei jedem weiteren Mal). Der Unterschied entschied den Weg: **eine gesprochene Zeile ist ein `SOUND`** (`Audio::sound_aus_pcm` = derselbe Weg wie `AUDIO_NOTE`, samt Lo-Fi und 5-ms-Flanken), sie laeuft auf einem eigenen Bus `speech` (nur Lautstaerke, keine Effektkette; `AUDIO_BUS_VOLUME/GET_VOLUME` und `AUDIO_PUSH/POP` kennen ihn), und `SPEAK_SOUND` gibt sie heraus (raeumlich per `AUDIO_PLAY_ON`, mischen, `AUDIO_SAVE_WAV`). Befehle: `SPEAK(text$[, unterbrechen])`, `SPEAK_STOP`, `SPEAKING`, `SPEAK_WAIT`, `SPEAK_VOICE` (unbekannt = Fehler mit der Liste), `SPEAK_VOICES`, `SPEAK_RATE` (0.5..2.0), `SPEAK_SOUND`. **Die Warteschlange braucht kein Polling:** ein angehaengter Satz geht mit `StartTime::Delayed(rest)` an Kira (wie `AUDIO_PLAY_AT`), `Sprecher` merkt sich nur das geplante Ende, `SPEAKING()` vergleicht mit der Uhr -- ein Konsolenprogramm, das in `INPUT` haengt, hoert seine Saetze trotzdem nacheinander. Die Handles der gesprochenen Zeilen liegen in `Audio::speech_handles`, NICHT im Slot: `AUDIO_PLAY` auf demselben Slot stoppt die alte Instanz, ein zweites `SPEAK("Treffer")` darf das erste nicht abschneiden. Vorrat je (Text, Stimme, Tempo) mit 64 Plaetzen, der aelteste nicht laufende faellt weg; ein per `UNLOADSOUND` freigegebener Vorratsklang wird neu gerechnet (`sound_lebt`). **Bildschirmleser zuerst:** `SPEAK` fragt `g.a11y_aktiv()` -- laeuft ein Hilfsprogramm, geht der Satz als Ansage in den Baum (`Gui::announce` UND `Graphics::a11y_ansagen`; der Fenster-Baum, den FLIP ohne GUI_UPDATE schickt, traegt dafuer jetzt den Ansage-Knoten), `SPEAKING()` bleibt FALSE; `GUI_SCREENREADER()` fragt seither das Fenster direkt und stimmt auch ohne gui. **Zwei Fallen:** (1) `SynthesizeTextToStreamAsync` gibt es im `windows`-Crate nur mit `Media_Core` UND `Storage_Streams` (der Fehler sagt "Methode nicht gefunden", nicht "Feature fehlt"); (2) WinRT braucht einen angemeldeten Faden -- unter GLFW ist der Hauptfaden schon STA (RoInitialize meldet RPC_E_CHANGED_MODE, ignoriert), ein Konsolenprogramm ohne Fenster hat nichts, darum `RoInitialize(MULTITHREADED)` vor jedem Zugriff. `.join()` der IAsyncOperation wartet auf ein Win32-Ereignis, die Synthese liefert von einem Threadpool-Faden -- kein Deadlock im STA. macOS `say -o --file-format=WAVE --data-format=LEI16@22050` (Text ueber stdin, damit er mit `-` beginnen darf), Linux `espeak-ng --stdout --stdin -v <LANG>` -- beide ungeprueft, beide mit Fehler samt Installationshinweis, wenn das Werkzeug fehlt. Eigener WAV-Leser (`wav_lesen`, PCM 8/16/24/32 + Float, Kanaele gemittelt, Datenlaenge ueber das Ende = bis zum Ende, weil `--stdout` sie nicht nachtraegt) mit Rust-Tests. Pruefstein `tests/pruef/ton/speak.dhtest` (WAV im `--- nachher` Byte fuer Byte gelesen: mono, Huellkurve; Vorrat; angehaengt dauert > 1,6x unterbrochen; unbekannte Stimme -- dieser Fall war in der pytest-Fassung bis 2026-09-16 IMMER uebersprungen, weil die Hilfsfunktion "nicht gefunden" als fehlende Sprachausgabe wertete) und in `tests/pruef/gui/gui_barrierefreiheit.dhtest` der fremde UIA-Leser, der in einem Programm OHNE gui die Ansage als Text-Knoten findet. Bewusst NICHT: SSML, Wortgrenzen-Ereignisse, mitgelieferte Stimme (piper, 60 MB), SAPI.

**Fremde Schriften, Euro, Eingabemethoden** (2026-09-06, Wege A und B aus `docs/entwurf-eingabemethoden.md`): gemessen vor dem Bau war der Speicher Unicode, die ANZEIGE Latin-1 -- `zeichensatz()` in graphics.rs backte 246 Zeichen ohne `€`, ohne Latin Extended, Griechisch, Kyrillisch, CJK, Emoji; auch eine per LOADFONT geladene japanische Schrift bekam nur diese. Jetzt: (1) Grundvorrat ASCII..Kyrillisch + Interpunktion + `€` (~1000 Glyphen, Start +50 ms); (2) **Glyphen auf Zuruf** -- `glyphen_pruefen` merkt beim Aufzeichnen/Messen Codepunkte ohne Glyphe (`glyphen_fehlend`, RefCell weil Messen `&self` ist), `ausweich_nachladen` am FLIP-Anfang backt je Datei EINEN Eintrag (`Vec<Ausweich>`, Index 0 = Grundschrift, unveraendert) mit der Vereinigung aller bisher gebrauchten Zeichen (`ausweich_datei_fuer` waehlt die Datei je Block: msgothic.ttc/malgun.ttf/seguiemj.ttf/segoeui.ttf; macOS Arial Unicode; Linux Noto CJK), was keine Datei liefert -> `glyphen_unmoeglich`; Zeichnen UND Messen zerlegen einen Text in Laeufe je Font (`text_laeufe`, `zeichne_text`, `breite_mit` -- EINE Aufteilung, sonst misst ein Layout etwas anderes); (3) `LOADFONT(pfad$, groesse[, zeichen$])` mit Blocknamen (`BLOECKE`, deutsch/englisch) oder Zeichenkette (`zeichen_aus_namen`), Grundvorrat immer dabei; (4) **Schriftsammlungen .ttc**: raylib (stb ab Versatz 0) liest sie nicht und tauschte STILL gegen die Bitmapschrift -- `ttc_erste_schrift` loest die erste Schrift heraus (Tabellenverzeichnis kopieren, Versaetze neu), alles laeuft ueber `schrift_laden` + `LoadFontFromMemory`; eine Ersatzschrift ist jetzt ein Fehler (`keine_ersatzschrift`, Textur-ID der Standardschrift). Auf dieser Maschine gibt es KEIN CJK-.ttf, nur .ttc -- ohne (4) blieb CJK ein `?`. (5) Tipp-Warteschlange 16 -> 256 ueber `CFLAGS=-DMAX_CHAR_PRESSED_QUEUE=256` in build_runtime.py (cmake uebernimmt CFLAGS; nachgesehen im CMakeCache von raylib-sys). Weg B: `Gui::schreibmarke` (dieselbe Rechnung wie das Zeichnen der Marke) -> `Graphics::ime_position` -> `ImmSetCompositionWindow`/`ImmSetCandidateWindow` (Feature `Win32_UI_Input_Ime`), nur bei Aenderung; ohne IME hier nicht messbar. **Gemessen:** erstes Bild mit Kanji+Hangul+Emoji+Hebraeisch ~100 ms, weiteres Kanji ~15 ms. Tests `tests/pruef/grafik/schriften_vorrat.dhtest` (Bildvergleich echt gegen `?` ueber `dhrt bild` + GETPIXEL, Gegenprobe U+E000; bis 2026-09-16 pytest mit Pillow). Bewusst NICHT: Textformung/RTL, Farb-Emoji, Wahl der Schrift INNERHALB einer Sammlung. **Weg C (Vorschau im Feld) kam am selben Tag nach:** `ime.rs` (cfg windows) haengt per `SetWindowLongPtrW` einen zweiten Subclass an raylibs GLFW-Fenster (Kette ruft den Vorgaenger, also AccessKit), nimmt `WM_IME_STARTCOMPOSITION`/`COMPOSITION`/`ENDCOMPOSITION` selbst (dann entsteht das Systemfenster der IME gar nicht) und verschluckt `WM_IME_CHAR` (das Ergebnis kommt ueber `GCS_RESULTSTR`, sonst kaeme alles doppelt) -- NUR solange `feld_aktiv` (gui-Textfeld mit Fokus, von der VM je Bild gesetzt), sonst laeuft alles durch wie bisher (INKEY$, ui-Modul). Zustand in einem `static Mutex` (die Fensterprozedur ist eine freie `extern "system"`-Funktion), `abholen()` je Bild VOR `gui.update`: `Gui::ime_ergebnis` (Einfuegen an der Marke durch Hoechstlaenge/Zahlenfilter, Undo, on_change), `Gui::ime_vorschau` (Widget-Felder `vorschau`/`vorschau_marke`, transient), `anzeige_mit_vorschau` = EINE Quelle fuer Zeichnen, Unterstreichen, Schreibmarke und die Meldung an die IME. Waehrend einer Umwandlung filtert die IME die Tasten (GLFW: `VK_PROCESSKEY` -> kein Ereignis, win32_window.c:776). Ohne installierte IME NICHT gemessen -- Rust-Test `ime_ergebnis_und_vorschau` deckt die Zeichenkettenseite.

**Bedienung ohne Maus** (seit 2026-08-30): `TAB`/`SHIFT+TAB` laeuft durch ALLE bedienbaren Widgets (vorher nur TextInput/TextArea -- ein Fenster ohne Textfeld war per Tastatur gar nicht bedienbar); Leertaste/Enter loest aus, Pfeile verstellen Werte bzw. bewegen Auswahlen, `ESC` schliesst eine offene Klappliste. EINE Quelle dafuer ist `Kind::fokussierbar()` -- Tab-Zyklus, Klick-Fokus und Fokus-Ring fragen alle dort. Der Ring (Akzentfarbe, 2 px ausserhalb) wird an EINER Stelle am Ende von `draw_widget` gezeichnet; ohne sichtbaren Fokus waere die Navigation wertlos. Abfragbar mit `GUI_FOCUSED()` (-1 = keins), Gegenstueck zu `GUI_FOCUS`. Weil damit auch Knopf/Kaestchen/Klappliste Fokus fuehren, feuern `on_focus`/`on_blur` dort jetzt ebenfalls -- der Form-Designer bietet sie entsprechend an (`_FOKUS` in formdesigner/document.py). Tests `tests/pruef/gui/gui_tastatur.dhtest` (echte Tasten via Automation-Wiedergabe, darum in `_SERIELL`). Window-Drag an der Titelleiste, Z-Order (Klick bringt nach vorne), Fokus, Schliessen-Button, **resizeable Fenster** (`GUI_WINDOW_RESIZABLE` — am unteren-rechten Griff ziehbar, `GUI_WINDOW_SET_MIN_SIZE`/`MAX_SIZE` als Grenzen; in `.dhform`-JSON als `resizable`/`min_w`/`min_h`/`max_w`/`max_h`) + **Control-Anchoring** (`GUI_SET_ANCHOR(wdg, "lrtb")` — Reflow beim Resize: Widgets kleben an Kanten/dehnen sich; in JSON als `anchor`-Edge-String) + **randlos** (`GUI_WINDOW_CHROME(win, an)` — ohne Titelleiste/Rahmen, damit eine Form das OS-Fenster füllt; der Form-Designer-Run koppelt die Form so ans native OS-Fenster). **Widgets werden auf den Fenster-Innenbereich geclippt.** Cyan-Theme (programmierbar). Konstruktoren/Getter sind `@builtin`, nur `GUI_UPDATE`/`GUI_DRAW` sind `graphics_builtin`. Komplement zum Immediate-Mode-`ui`-Modul (dort `UI_WINDOW_BEGIN/END` + `UI_TABLE` mit `UI_TABLE_SELECTED`/`SET_SELECTED`/`HEADER_CLICK`). Doku `docs/module-gui.md`, Demos `examples/45_gui.dh` + `examples/81_table_select.dh` + **`examples/156_gui_alle_widgets.dh`** (alle 22 Widget-Arten in EINER Vollbild-Anwendung, jedes mit echter Aufgabe: Baum filtert Tabelle, Tabellenzeile fuellt Editor, Regler formen die Kurve auf der Zeichenflaeche -- der schnellste Weg, eine Widget-Art in Aktion zu sehen), Tests `tests/test_gui_*.py`.


**Module sind in dhrt/Rust implementiert** (`rust/drachenhauch_runtime/src/<modul>.rs` +
Dispatch in `vm.rs` `try_<modul>`; externe Typen + ihr Default in `vm.rs`/
`value.rs`). Neues Modul: `.rs` schreiben, im `vm.rs`-`CALL_BUILTIN`-Dispatch
einhängen, den Modul-Namen in `rust/drachenhauch_runtime/src/preprocess.rs` MODULES
ergänzen (sonst erkennt der Preprocessor `IMPORT "modul"` nicht). Dann ein Fall in
einer Prüfsammlung + `daten/builtin_index.json`.

**IMPORT-Auflösung in [preprocess.rs](rust/drachenhauch_runtime/src/preprocess.rs):**
Der Pfad wird **wörtlich** aufgelöst — es wird KEINE `.dh`-Endung angehängt.
1. Existiert der geschriebene Pfad als Datei? → textuelles Inkludieren
   (Quellcode-Modul). Dafür muss die Endung mitgeschrieben werden:
   `IMPORT "helfer.dh"`.
2. Sonst: ist der Name ein bekanntes Built-in-Modul (`MODULES`)? → die
   `IMPORT`-Zeile wird zu einem Kommentar (dhrt kennt das Modul nativ).
3. Sonst: Fehler.

Daraus folgt: **`IMPORT "json"` nimmt IMMER das eingebaute Modul**, auch wenn
ein `json.dh` daneben liegt — die Endung fehlt, also greift Regel 1 gar nicht.
Wer ein Built-in mit eigenem Code überschreiben will (z.B. für Tests), muss die
Endung schreiben: `IMPORT "json.dh"`.

## Verlaeufe ohne Stufen, IDE ruhiger (2026-11-07)

Wunsch des Nutzers: die Farbverlaeufe sahen mit ihren Abstufungen "nach
Windows 95" aus. Ursache gemessen am Bildschirmfoto der IDE: `Cmd::RoundGradient`
fuellte zwar zeilenweise, schnitt die Farbe aber auf ganze 8-Bit-Stufen ab,
in allen drei Kanaelen im selben Moment -- auf der Ausgabe-Leiste Baender von
15 Zeilen, in der Gliederung von 32, jede mit einer geraden Kante
(Mach-Baender). Jetzt `graphics::verlauf_farbe`: je Zeile gegen eine eigene
Schwelle einer 16er-Bayer-Folge gerundet (Lage auf dem Schirm, Kanaele um
0/5/10 versetzt, Alpha um 3), die Enden sind genau `c1`/`c2`, gleiche
Kanaele bleiben unberuehrt. Dieselbe Rechnung fuer `Cmd::GradientRect`
(`GRADIENTV`/`GRADIENTH`, auch die Farbwaehler der gui), dort vorher raylibs
GPU-Verlauf mit denselben Baendern. Nicht angefasst: `Gradient4`,
`CircleGradient`, `Lichtfleck` (GPU-Dreiecke). Tests
`tests/pruef/gui/gui_verlauf_glatt.dhtest` (zaehlt "harte Kanten" = acht gleiche
Zeilen, dann acht andere; die alte Laufzeit hat 23 bzw. 16) und drei
Rust-Tests `verlauf_tests`.

**Grosse Flaechen leiser** (an `verlauf_hoehe` gekoppelt, ohne die Metrik
unveraendert): die Fase bekommt das QUADRAT von `verlauf_anteil` (eine
helle Linie ueber einer Leiste von 200 Punkten war die Kante aus Windows
95; Deckkraft nie unter 1, 0 hiesse deckend), der Fokusring den Anteil
selbst (um das Code-Feld lag er als zweiter Rahmen um den ersten). Tests
zwei Faelle in `gui_verlauf_hoehe.dhtest`, die alte Laufzeit faellt beide.

**In der IDE** (`ide/ide.dh`, je mit Vorher/Nachher-Bild an den Nutzer):
Bereichsfarben 20 -> 9 % (hell 16 -> 7 %; tuerkis, braun und blau um den
Code waren unruhig; `werkzeug_ide_schriftzug.dhtest` mit den neuen Werten),
der Schriftzug hinter dem Code 22/24 -> 14 (dunkel) / 10 (hell -- dort fiel
er staerker auf), der Kopf der Probleme-Leiste ohne das doppelte
"Probleme:" (der Reiter heisst schon so; drei Eintraege aus `en.txt` weg) und
die Ueberschrift der Hilfe in `muted_fg`, buendig mit ihrem Text.

## Projektbaum nach Dateiart gefaerbt (2026-11-06)

Wunsch des Nutzers: im Projektbaum der IDE war alles einfarbig. Neu
`GUI_FILETREE_COLOR(baum, muster$, farbe)` (`DateiBaum::farben`, Muster wie
beim Filter mit `;`, `"/"` meint Ordner, jedes andere Muster nur Dateien --
ein Ordner `bilder.png` ist kein Bild; das erste passende gilt, ein schon
gesetztes Muster behaelt seinen Platz, -1 nimmt es weg; `farbe_fuer` setzt
`TreeNode::color`, das der Baum laengst zeichnete; in der `.dhform` als
`filetree.farben`). Die IDE (`baumFarben`, aus `bereichsFarben`, also nach
jedem Themawechsel): Ordner, `.dh`, Werkzeugdateien, `.dhtest`, Daten,
Text, Bilder, Klaenge je eine Farbe (`B_*` in `codeFarben`, hell dunkler),
in `farben.json` als `baum_ordner` ... `baum_klang`. Tests zwei Faelle in
`gui_dateibaum.dhtest` (Bild: jede Farbe in genau ihrer Zeile, Ordner vor
Dateien; Rundweg ueber GUI_TO_JSON -- das ist EINGERUECKT, verglichen wird
ohne Leerraum), zwei in `werkzeug_ide_schriftzug.dhtest` (beide Themen,
eigene Farben).

## Menue und Klappliste ueber allen Fenstern (2026-11-10)

Fund des Nutzers im Studio: das aufgeklappte Menue "Ansicht" lag unter dem
Entwurfsfenster -- "Helles E..." und "English" waren zur Haelfte verdeckt.
Die gui zeichnete ein aufgeklapptes Leistenmenue und eine offene Klappliste
mit IHREM Fenster (`draw_window`), also unter jedem Fenster davor (das Studio
holt sein Entwurfsfenster jedes Bild nach vorn) und unter allem, was ein
Programm nach `GUI_DRAW(FALSE)` malt (die Titelleiste des Entwurfs). Der
Treffertest (`menu_input`) nahm den Klick aber vor allen Fenstern an -- ein
Klick auf das Verdeckte waehlte den Eintrag, den man nicht sah. Jetzt kommen
beide in die obere Schicht (`draw_top`, nach Popovern, vor dem
Kontextmenue), wie das Kontextmenue schon immer; `popups_zeichnen` an EINER
Stelle. Ausnahme `popups_oben`: ein Fenster im Entwurfsmodus mit Zoom oder
Zuschnitt behaelt sie, denn dort stehen sie in seiner vergroesserten Lage.
Tests drei Faelle in `gui_menue_obenauf.dhtest` (selbst gemalt, Fenster
davor, Klappliste), die alte Laufzeit faellt alle drei.

## Abfragen zu den Settern (2026-11-11)

`GUI_TOOLTIP$`, `GUI_TEXTINPUT_GET` (dieselben Schluessel wie
`GUI_TEXTINPUT_SET`), `GUI_TEXTINPUT_PLACEHOLDER$` (auch Textbereich),
`GUI_STATUSBAR_TIP$`, `GUI_TOOLBAR_GET`, `GUI_DOCK_TITLE$`,
`GUI_TEXTAREA_HINT$` und `GUI_TEXTAREA_HINT_COUNT`. Anlass: die Tests der
IDE-Durchsicht (geschichte/ide.md) konnten den Zustand nicht zuruecklesen.
`GUI_TEXTAREA_HINT$` zieht die Zeilen vorher nach (`marken_abgleichen`),
sonst gaebe es zwischen Tippen und naechstem Bild den alten Stand. Die
Gegenprobe verfaelschte alle acht in einem Bau (`gui_abfragen.dhtest`,
jeder Wertefall prueft einen anderen Getter). Dazu der Fokusrand: ein
Textbereich ueber `verlauf_hoehe` mischt den Akzent mit der Randfarbe
(mindestens 35 %), ein eigener Akzent am Widget bleibt
(`gui_verlauf_hoehe.dhtest`).

## Kleine Luecken gegenueber anderen GUIs (2026-11-12)

Aus einer Durchsicht "was fehlt gegenueber Qt, WinForms, GTK" die drei
kleinen Punkte (der grosse -- echte OS-Fenster -- steht als Weg E in
docs/intern/entwurf-native-fenster.md):
- Elf Abfragen zu Settern ohne Gegenstueck (`GUI_GET_ALIGN$`,
  `GUI_GET_ANCHOR$`, `GUI_GET_FONT_SIZE`, `GUI_GET_ICON$`/`GUI_GET_ICON`,
  `GUI_GET_ROUND`, `GUI_GET_MIN_W`/`_H`, `GUI_GET_WRAP`, `GUI_GET_TAB_INDEX`,
  `GUI_STYLE_GET`), gefunden durch einen Abgleich des Index (Setter ohne
  Getter).
- Rueckgaengig als API: `GUI_UNDO`/`GUI_REDO`/`GUI_CAN_UNDO`/`GUI_CAN_REDO`/
  `GUI_UNDO_CLEAR` -- derselbe Weg wie Strg+Z (`text_undo`), bei einer
  zweiten Ansicht der Besitzer.
- Aussehen des Systems: `systemaussehen.rs` (Registry, `defaults`,
  `gsettings`; zwei Sekunden gemerkt), `GUI_SYSTEM_DARK`,
  `GUI_SYSTEM_ACCENT`, `GUI_THEME_PRESET("system")`; fuer Pruefungen
  `DHRT_SYSTEM_DUNKEL`/`DHRT_SYSTEM_AKZENT`. Unbekannt heisst fuer das Preset
  dunkel, fuer `GUI_SYSTEM_DARK` FALSE.
Gegenprobe: alle Stellen in einem Bau verfaelscht, und ein Vergleich zeigte
JEDE Ausgabezeile neben der Erwartung (dhrt test nennt nur die erste) -- so
war keine Verfaelschung hinter einer frueher fallenden Zeile verdeckt.

## Echte OS-Fenster, Weg E Stufe 1 (2026-11-12)

`GUI_WINDOW_NATIVE(win, an)` / `GUI_WINDOW_IS_NATIVE(win)`. Bau nach
docs/intern/entwurf-native-fenster.md, Abschnitt 8:
- `nativfenster.rs`: zweites GLFW-Fenster mit geteiltem Kontext, eigene
  Rueckrufe (Knopf, Taste, Lage, drinnen, Rad, Zeichen, Fokus, Groesse),
  Eingang je Fenster, Zeigen per FBO + `glBlitFramebuffer`.
- `graphics.rs`: das Hauptfenster als GLFW-Handle gemerkt (statt "der
  aktuelle Kontext"), je natives Fenster ein Render-Ziel ueber
  `render_ziel`, Zeigen und Abholen in `flip` nach EndDrawing. Die
  Eingabeabfragen liefern den Stand eines nativen Fensters NUR in der Sicht
  der gui (`sicht`, gesetzt in GUI_UPDATE/GUI_DRAW) -- MOUSEX & Co. des
  Programms bleiben beim Hauptfenster.
- gui: Oberflaeche je Fenster (`nativ`), ein natives steht auf seiner bei
  (0, 0) ohne eigene Titelleiste; `topmost_at` sieht nur die Fenster der
  Flaeche, durch die die gui schaut; Popups im Fenster
  (`popups_oben` = falsch); `nativ_abgleich` je Bild: Kreuz, Groesse, Lage,
  Titel, sichtbar, Fokus, verwaiste Fenster.
Funde beim Bau:
- Das Bild stand auf dem Kopf: dhrts Ziele stehen schon richtig herum, der
  Spike spiegelte -- seine Proben lagen symmetrisch und sahen es nicht.
- Windows meldet WM_MOUSELEAVE mitunter, bevor der Druck ankommt: ein Druck
  haelt den Zeiger darum selbst fest, bis einschliesslich des Bildes, in dem
  losgelassen wird -- sonst traf der Klick die falsche Flaeche oder wurde nie
  einer.
- PowerShell gibt `$null` an einen string-Parameter als "" weiter:
  `FindWindow($null, ...)` suchte eine Fensterklasse "". Richtig:
  `[NullString]::Value`. fenstersender.ps1 findet ein Fenster jetzt zuerst
  ueber seinen Titel (auch das zweite eines Programms); neu fensterpixel.ps1
  (Farben aus dem Inhalt eines Fensters vom Bildschirm).
Pruefung `tests/pruef/fenster/native_fenster.dhtest` (9 Faelle, echte
Fensternachrichten), Gegenprobe in zwei verfaelschten Baeuen: jede der
sechs Stellen (Rueckfall, Fangregel, Flaechenfilter, Bildrichtung, Zeichen,
Kreuz) laesst ihren Fall fallen. macOS: Rueckfall, bis der Spike auf einem
Mac lief.

## Echte OS-Fenster, Weg E Stufe 2 (2026-11-12)

- Popups nach Oberflaeche: `draw_top_flaeche(g, f)` zeichnet die obere
  Schicht (Kontextmenue, Toasts, Vorschlaege, Monat, Zellwahl, Leistenmenue,
  Rechtschreibung, Hilfetext) nur fuer Fenster der Flaeche `f`; ein natives
  Fenster bekommt seine in `nativ_zeichnen`. Ein Druck auf einer anderen
  Flaeche schliesst fremde Popups (`fremde_popups_zu`) -- sonst verglichen
  ihre Treffertests Koordinaten zweier Flaechen.
- Modal und nativ: unter Windows wird das Hauptfenster Besitzer
  (`GWLP_HWNDPARENT`, `besitzer_setzen`) -- der Dialog bleibt darueber, ohne
  Eintrag in der Taskleiste. Der Schleier liegt ueber dem Hauptfenster, ein
  Klick hinein holt den Dialog nach vorn (`nativ_fokus_wunsch`).
- IME: die Umwandlung folgt dem Fenster mit dem Fokus des Systems
  (`ime_position` auf dessen HWND, Massstab 1).
- **Fund und behoben:** waehrend eines modalen Dialogs waren ALLE Menues
  gesperrt, auch das Kontextmenue im Dialog selbst (`menu_input` lief gar
  nicht). Jetzt gelten die des Dialogs, die dahinter bleiben gesperrt. Dabei
  zweiter Fund: die Schleife ueber die Menueleisten sah die Oberflaeche nicht
  -- ein Klick oben im Hauptfenster haette die Leiste eines nativen
  Fensters (bei 0, 0) geoeffnet.
- Pruefhelfer: `fensterpixel.ps1` liest jetzt per PrintWindow
  (PW_RENDERFULLCONTENT) statt vom Bildschirm -- ein nebenher laufendes
  Vollbildprogramm (eine Weihnachtsszene) liess den Bildfall mit fremden
  Farben fallen. `fensterbesitzer.ps1` nennt den Besitzer eines Fensters.
Pruefung: `native_fenster.dhtest` jetzt 18 Faelle; Gegenproben in vier
verfaelschten Baeuen (Besitzer, Kontextmenue-Flaeche, Schleier, Tasten,
Modal-Sperre; Menue-Sperre, Leisten-Flaeche, Sperre dahinter, Bildrichtung).
Linux (WSLg, X11-Klicks per xdotool): Klick im nativen Fenster und im
Hauptfenster getrennt gezaehlt, Tippen kommt an, Kontextmenue im modalen
nativen Dialog geht. Ein Besitzer ist dort noch nicht gesetzt (nur Windows).

## Echte OS-Fenster, Weg E Stufe 2 (Rest, 2026-11-12)

- Massstab je Schirm: ein natives Fenster auf einem Schirm anderer Dichte
  als das Hauptfenster bekommt k = Dichte seines Schirms / Dichte des
  Hauptfensters (`nativfenster::verhaeltnis`, glfwGetWindowContentScale).
  Fenster und Render-Ziel sind k-mal so gross; beim Abspielen skaliert eine
  rlgl-Matrix die Geometrie (glatt, ohne Rundung je Ecke). Der
  Abspiel-Faktor `s` der Ziele ist eine ganze Zahl -- 1.5 ging damit nicht,
  ein `f32` haette `render_scene` ganz umgestellt. Text zeichnet ohne die
  Matrix in k-facher Groesse (`ohne_rt_matrix`, sonst vergroesserte
  Glyphen = unscharf), der Scissor rechnet k selbst (sieht keine Matrix).
  Maus und Groesse vom System kommen in Pixeln und gehen durch k zurueck
  in Punkte; eine Groesse, die wir selbst gesetzt haben, ist keine
  Bedienung, und eine vom Nutzer gezogene behaelt ihre Pixel
  (`system_groesse`, sonst zuckte das Fenster um ein Pixel).
  Schirmwechsel prueft `nativ_bild` je Bild (nicht waehrend gedrueckt).
  Abfrage `GUI_WINDOW_NATIVE_SCALE`, Pruefhebel `DHRT_NATIV_MASSSTAB`.
- Vorzeigen: IDE-Einstellungen mit Schalter "Als eigenes Fenster"
  (`einstellungen_eigen`, aus per Vorgabe -- Aufnahmen in IDE-Tests
  erreichen ein natives Fenster nicht), Studio-Berichtsvorschau mit Knopf
  "Als eigenes Fenster" / "Zurueck ins Studio" (rechts angehaengt, die
  Knoepfe davor bleiben fuer die Klicklagen der Tests, wo sie waren).
Pruefung: zwei neue Faelle (Groesse und Farben bei 1.5 per
`fensterpixel.ps1 groesse`, Klick in Pixeln trifft den Knopf in Punkten);
Gegenprobe: Mausumrechnung und Matrix verfaelscht -- je genau ihr Fall
faellt.
