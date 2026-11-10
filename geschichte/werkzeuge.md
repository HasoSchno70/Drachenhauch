# Werkzeuge in Drachenhauch

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Die Editor-Piloten (SFX, Partikel, Tilemap, Sprite, Tracker, Form-Designer, Anim-FSM, Notenblatt), ihre Faktoren gegen Qt, das Auffrischen 2026-10, das Klassendiagramm.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

> ## Editoren in Drachenhauch selbst -- fuenf Piloten (2026-08-30 .. 09-04)
>
> Die Frage, ob die Qt-Editoren nach Drachenhauch koennen, ist an fuenf Faellen
> gemessen statt geschaetzt:
>
> | Editor | Qt | Drachenhauch | Faktor |
> |---|---|---|---|
> | SFX-Generator (`examples/183_sfx_generator.dh`) | 522 | 674 | 1,29 |
> | Partikel-Editor (`examples/185_partikel_editor.dh`) | 802 | 674 | 0,84 |
> | Tilemap-Editor (`examples/187_tilemap_editor.dh`) | 2428 | 1575 | 0,65 |
> | Sprite-Editor (`examples/189_sprite_editor.dh`) | 7379 | 2934 | 0,40 |
> | Tracker (`examples/190_tracker.dh`) | 3911 | 2165 | 0,55 |
> | Form-Designer (`examples/197_form_designer.dh`, Weg B) | 5055 | 1342 | 0,27 |
> | Anim-FSM-Editor (`examples/198_anim_fsm_editor.dh`, Weg B) | 1728 | 1341 | 0,78 |
> | Notenblatt (`examples/199_notenblatt.dh`, Weg B) | 1710 | 1536 | 0,90 |
>
> Die Zahlen sind gegen die Dateien geprueft (tests/test_editor_qt_piloten.py)
> -- zwei standen hier lange falsch: 400 statt 402 (von Anfang an falsch
> gezaehlt) und 1005 statt 1029 (der Sprite-Editor wuchs mit den
> `IMAGE_FREE`-Aufrufen). Die Faktoren aendert das nicht.
>
> **Kein Faktor davon ist ohne seinen Fall zu gebrauchen, und keiner ist ein
> Fortschritt gegenueber dem vorigen.** Beim Partikel-Editor zeichnet die
> Qt-Fassung ihre Vorschau von Hand (`paintEvent`, alle fuenf
> Darstellungsarten in QPainter), die Drachenhauch-Fassung ruft
> `PARTICLE_DRAW` -- rund 150 Zeilen Ersparnis kommen allein daher. Beim
> Tilemap-Editor ist die 0,63 aus demselben Grund zu guenstig. Nicht
> portiert ist dort inzwischen nichts Benennbares mehr.
> **Eine genaue Restzahl gab es dafuer schon vorher nicht mehr** -- der
> Qt-Eigenschaften-Dialog (167 Zeilen) bedient Kacheln UND Objekte, und
> beide Haelften sind inzwischen da, nur nicht als EIN Dialog. Die Zahl lag
> bei 620 (Faktor 0,42 gegen den Rest), als DH-Code-Export,
> Eigenschaften, Objekt-Ebenen und mehrere Tilesets noch alle fehlten.
> Wer hochrechnet, muss beide Fragen stellen: wie viel von dem Qt-Code
> dupliziert etwas, das die Laufzeit schon kann -- und wie viel von dem
> Qt-Code hat die Drachenhauch-Fassung gar nicht erst?
>
> **Der Tilemap-Pilot schreibt seit 2026-09-02 auch DH-Code** -- einen
> lauffaehigen Renderer, die Karte als Tiled-JSON und das Tileset-Bild, alle
> drei unter demselben Namen und in einem Zug. Das Bild kommt mit, obwohl die
> Karte seinen Pfad ohnehin nennt: der Pfad zeigt dorthin, wo das Tileset beim
> BEARBEITEN lag, und wer den Ordner weitergibt, haette sonst eine Karte, die
> auf nichts zeigt. Geprueft am BILD des erzeugten Programms, mit der
> Gegenprobe im Test: die leere Karte ergibt genau EINEN Farbton, dieselbe mit
> einer gemalten Kachel mehr. Uebersetzen allein bewiese nichts -- ein
> Renderer, der sein Tileset nicht findet oder die gid falsch aufloest,
> uebersetzt genauso und zeigt einen leeren Schirm.
>
> **Kachel-Eigenschaften kann der dritte Pilot seit 2026-09-02** -- ein
> [solid]-Kaestchen und beliebige Schluessel/Werte, mit der Liste dessen, was
> die gewaehlte Palettenkachel schon hat. Beim ABWAEHLEN von `solid` wird die
> Eigenschaft ENTFERNT, nicht auf FALSE gesetzt: sobald irgendeine Kachel ein
> `solid` traegt, schaltet `solid_aware` die Kollision von "jede Kachel
> blockiert" auf "nur die mit solid = TRUE" um -- ein zurueckgelassenes FALSE
> haelt diesen Schalter umgelegt, ohne dass man es sieht. Der Wert aus dem
> Eingabefeld wird GEDEUTET (true/false, Zahl mit Punkt -> FLOAT, ohne ->
> INTEGER, sonst Text) und das Ergebnis sofort angezeigt: eine 5 als Text
> sieht in der Datei fast aus wie eine 5 als Zahl, verhaelt sich aber anders.
> Dabei fiel eine weitere Modul-Luecke auf: es gab keine Moeglichkeit, die
> Schluessel einer Kachel AUFZUZAEHLEN (`TILED_TILE_HAS_PROP` beantwortet nur
> eine Frage, die man schon kennt) -- dafuer neu `TILED_TILE_PROP_KEYS` und
> `TILED_OBJECT_PROP_KEYS`, sortiert, weil eine Liste aus einer HashMap sich
> sonst bei jedem Auffrischen neu mischt.
>
> **Objekt-Ebenen kamen einen Tag spaeter** und machten aus derselben
> Zeichenflaeche ein zweites Werkzeug: Ziehen legt ein Rechteck an, ein Klick
> einen Punkt, ein Klick auf ein vorhandenes waehlt es aus (Name, Typ und
> beliebige Eigenschaften daneben, Entf loescht). **Die Ebenenart entscheidet,
> was die Maus tut** -- eine zweite Werkzeugleiste waere ein Schalter, den man
> vergisst umzulegen. Drei Dinge daran waren nicht offensichtlich:
> (1) `TILED_TILE_AT` auf einer Objekt-Ebene ist ein FEHLER, kein leeres
> Ergebnis -- die Zeichenschleife, die Nummern-Anzeige, die Zwischenablage und
> das Entf fuer die Kachel-Auswahl mussten alle eigens ausgenommen werden, und
> jede dieser Stellen brachte das Programm zum Abbruch statt zu einem falschen
> Bild. (2) Ein Rueckgaengig gehoert zu SEINER Ebene, und weil
> `verlaufAnwenden` dorthin zurueckschaltet, muss die linke Spalte mitgehen
> (`verlaufAnzeige`) -- sonst zeigt sie die Bloecke der vorigen Ebene, waehrend
> gemalt wird. Rueckgaengig deckt weiterhin nur KACHELN ab; ein geloeschtes
> Objekt ist weg. (3) `DIM blockA[7] AS GUI_WIDGET` muss mit Groesse
> deklariert werden -- ein Feld-Literal aus Widget-Handles wird als
> `ARRAY OF INTEGER` gedeutet und dann abgelehnt.
>
> **Mehrere Tilesets** (2026-09-02) waren der letzte grosse Rest und
> brauchten keinen einzigen neuen Befehl -- das `tiled`-Modul konnte sie
> laengst, der Pilot rechnete nur ueberall mit EINEM. Drei Stellen mussten
> auseinandergezogen werden: die Palette zeigt eines (`tsAkt`, Klappliste
> plus [+]), gemalt wird aus dessen GID-Bereich -- **gezeichnet aber jede
> Kachel mit IHREM eigenen** (`tsFuerGid`), sonst saehe die halbe Karte
> falsch aus, sobald zwei Tilesets im Spiel sind. Die Pipette schaltet mit
> um, sonst zeigte die Palette die aufgenommene Kachel gar nicht und der
> naechste Strich malte eine andere. **Entfernen gibt es bewusst NICHT**:
> die firstgids dahinter wuerden sich verschieben, und jede damit gemalte
> Kachel zeigte danach stumm auf ein anderes Bild -- genau der Fehler, den
> die Qt-Fassung 2026-07-26 hatte. Und die Palette ist jetzt ein
> AUSSCHNITT fester Groesse mit Rad-Rollen (Umschalt: seitwaerts): ein
> zweites Tileset hat selten dieselben Masse, und eine mitwachsende Palette
> haette die ganze linke Spalte bei jedem Umschalten verschoben.
> Faktor 0,52 -> 0,61.
>
> **Das Umsortieren der Ebenen** (2026-09-02) war der letzte Punkt und der
> einzige, der noch einen neuen Befehl brauchte: `TILED_MOVE_LAYER` (siehe
> `tiled` unten). Zwei Dinge daran sind nicht offensichtlich:
> (1) Die Reihenfolge IST die Zeichenreihenfolge, die erste Ebene liegt
> hinten -- die Knoepfe heissen deshalb "Nach hinten"/"Nach vorne" statt
> hoch/runter, und die Liste sagt es dazu ("weiter unten = weiter vorne");
> hoch/runter waere doppeldeutig, weil die Liste die hinterste oben zeigt.
> (2) **Ein aufgezeichneter Verlaufsschritt merkt sich die NUMMER seiner
> Ebene** -- nach dem Umstellen zeigt sie auf eine andere, und ein
> Rueckgaengig schriebe den Vorher-Stand der EINEN Ebene in die ANDERE, also
> eine leere Flaeche ueber fremde Kacheln. Die Nummern wandern deshalb mit
> (`verlaufUmnummerieren`), statt den Verlauf zu leeren; im Test ist das mit
> Gegenprobe belegt (ohne den Aufruf faellt er). Damit ist am dritten Piloten
> nichts Benennbares mehr offen. Faktor 0,61 -> 0,63.
>
> **Der dritte Pilot war der erste aus einer anderen Familie**: kein
> Regler-Editor mit Vorschau, sondern Werkzeuge mit Zugbewegung, eine
> Auswahl mit Zwischenablage, Rueckgaengig/Wiederholen und ein Dateiformat,
> das ein FREMDES Programm lesen koennen muss. Dabei kam heraus, dass das
> `tiled`-Modul lesen und aendern konnte, aber **nicht anlegen und nicht
> speichern** -- neun Befehle nachgeruestet (siehe `tiled` unten).
>
> **Der vierte Pilot hat meine eigene Vorhersage widerlegt.** Hier stand,
> beim Sprite-Editor sei "groesstenteils KEIN Duplikat" dessen, was die
> Laufzeit kann (Ebenen, Undo, Werkzeuge, Zwischenablage), man solle also
> NICHT mit 0,5 rechnen. Beim ersten Durchgang kam 0,14 heraus -- der BESTE
> Wert, nicht der schlechteste. Der Grund ist nicht Sprachkraft: die
> Qt-Fassung ist schlicht eine viel groessere Anwendung (Dialoge,
> Datei-Browser, `.dhsprite`-Format, GIF, Streifen-Einlesen, .gpl-Paletten,
> Statistik), und der nicht portierte ANTEIL war damit der groesste aller
> vier Faelle -- allein die benennbaren Bloecke waren damals 1595 der 7379
> Zeilen, dazu verstreutes mehr.
>
> **Und genau das laesst sich seither an EINER Datei ablesen.** Derselbe
> Pilot steht heute bei 0,28: 0,14 (1005 Zeilen) -> 0,17 (1243, mit eigenem
> Format und bewegtem GIF) -> 0,22 (1608, mit Lasso und Zauberstab) -> 0,24
> (1754, mit Verschieben) -> 0,25 (1877, mit .gpl-Paletten) -> 0,28 (2037,
> mit Kachel-Ansicht und Statistik) -> 0,32 (2339, mit Zuschneiden,
> Groesse aendern und Animationsbereichen) -> 0,33 (2413, mit der
> DH-Code-Ausgabe) -> 0,34 (2474, mit der .dhanim-Ausgabe) -> 0,35 (2604,
> mit benannten Einzelbildern) -> 0,37 (2696, mit Spiegeln und
> Vierteldrehen) -> 0,38 (2811, mit einer Dauer je Einzelbild). Nichts daran
> ist schlechter geworden -- es
> wurde nur weniger weggelassen. Aus 0,17 sind so 0,38 geworden, mehr als das
> Doppelte, ohne dass sich an der Sprache etwas geaendert haette.
> **Damit ist die eigentliche Lehre aus vier Punkten: der Faktor misst vor
> allem, wie viel man weglaesst.** Er taugt nicht zum Hochrechnen, in keine
> Richtung.
>
> **Spiegeln und Vierteldrehen** (2026-09-03) waren der naechste Rest im
> vierten Piloten -- und der Weg dorthin fand die Luecke, die diese Runde
> ausmacht: **`IMAGE_ROTATE` ist fuer Pixelgrafik unbrauchbar, auch bei 90
> Grad.** Gemessen an einem 16x16-Bild mit vier verschiedenfarbigen
> Eckpunkten: nach `IMAGE_ROTATE(b, 90.0)` sind ALLE VIER verschwunden, und
> selbst die einfarbige Flaeche kommt verwaschen zurueck (0x141414 ->
> 0x131413). Sie rechnet trigonometrisch und tastet neu ab -- dieselbe Falle
> wie `IMAGE_SCALE` gegen `IMAGE_SCALE_NN`, nur faellt sie hier staerker auf,
> weil eine Vierteldrehung eigentlich verlustfrei IST. Neu deshalb
> **`IMAGE_ROTATE_CW`/`IMAGE_ROTATE_CCW`** (raylibs eigene, die die Punkte
> nur umsortieren; Breite und Hoehe tauschen). Im Test steht die Gegenprobe
> mit drin: `IMAGE_ROTATE` verliert die Marken, die neuen halten sie, und
> vier Vierteldrehungen geben Punkt fuer Punkt das Original.
>
> Zwei Entscheidungen im Piloten dazu: (1) **Alle vier Wandlungen gelten
> fuer das GANZE Sprite** (alle Bilder, alle Ebenen). Die Qt-Fassung dreht
> bei quadratischen Sprites nur das aktuelle Bild und sonst alle -- dieselbe
> Taste haette damit zwei Bedeutungen, und die sieht man nicht. (2) Sie
> stehen **NICHT im Verlauf**: der zeichnet je Schritt EINE Ebene auf, und
> ein Rueckgaengig nach einer Wandlung ueber alle drehte genau diese eine
> zurueck -- sichtbar erst beim Umschalten der Ebene. Er wird deshalb
> geleert; die Gegenrichtung nimmt die Wandlung ohnehin zurueck.
> Faktor 0,35 -> 0,37.
>
> **Eine Dauer je Einzelbild** (2026-09-03) fand die naechste Modul-Luecke:
> **`IMAGE_SAVE_GIF` nahm EINE Bildrate fuer alles**, obwohl GIF von Haus
> aus eine Dauer je Bild kann -- und genau die braucht eine Bildfolge: eine
> Pose wird gehalten, der Lauf dazwischen nicht. Der dritte Parameter nimmt
> jetzt zusaetzlich ein FELD (siehe `imgfx` unten). Im Piloten ein Drehfeld
> neben der Bildliste; **0 heisst "der Tempo-Regler gilt"**, nicht 0 ms --
> sonst waere der Regler nach dem ersten Bild wirkungslos, ohne dass man es
> sieht. Vorschau und GIF-Ausgabe fragen dieselbe Stelle (`dauerMs`), sonst
> liefe das Bild anders als die Datei; die Zeit steht in der Bildliste,
> wandert beim Kopieren mit (anders als der NAME -- der ist eine Kennung,
> die Dauer eine Eigenschaft) und steht in der `.dhsprite`. Faktor 0,37 ->
> 0,38.
>
> **Dabei fielen zwei Fehler auf, die aelter sind als die Aenderung.**
> (1) `JSON_LEN` WIRFT bei einem Pfad, den es nicht gibt -- es liefert nicht
> 0. `bildnamen`, `bilddauern` und `bereiche` stehen alle nur dann in der
> `.dhsprite`, wenn es sie gibt; ohne `JSON_HAS` brach das Laden an der
> ersten fehlenden ab. Eine Datei MIT Bereich, aber OHNE Einzelbild-Namen
> verlor beim Laden also stillschweigend ihren Bereich, und gemeldet wurde
> nur eine Pfad-Meldung, die nach einem Programmierfehler aussieht. Kein
> Test hatte das je getroffen, weil in allen bisherigen Faellen Namen
> gesetzt waren.
> (2) Ein Wert, den das PROGRAMM in ein Bedienelement setzt, sieht fuer eine
> Aenderungs-Erkennung wie eine Eingabe aus -- nach dem Laden schrieb sie
> die alte Zahl sofort zurueck. Dafuer `msLetzt`, derselbe Spiegel wie
> `letzteAuffrischen` im Partikel-Editor. **Beide Male war der erste Test
> dazu gruen und wertlos**: er klickte [Neu], ohne den Kasten "Neues
> Sprite" zu bestaetigen -- es passierte gar nichts, und weil dann auch
> nichts zurueckgesetzt wurde, stimmte die Zusage zufaellig. Seither prueft
> er den Zwischenstand (erst 0, dann wieder da).
>
> **Der fuenfte Pilot ist der Tracker** (2026-09-04, `examples/190_tracker.dh`,
> Faktor 0.53 gegen die Qt-Familie `trackereditor_qt.py` + `tracker/*.py`;
> nicht portiert sind Sample-/Keymap-/SoundFont-Instrumente, VU-Meter,
> Pattern-Namen und der Sample-Offset-Effekt). Er ist der erste mit einer
> ZEITACHSE, und das hat drei Modul-Luecken freigelegt, die kein Spiel und
> keiner der vier Piloten davor je gebraucht hat:
> (1) **`AUDIO_SFX` kennt drei Zeiten, aber keinen Sustain-PEGEL** und kein
> Ausklingen hinter der gehaltenen Zeit -- eine Orgel und ein Klavier waren
> damit nicht zu unterscheiden. Neu `AUDIO_NOTE(wf, freq, dauer, attack,
> decay, sustain, release, vol[, vib, vib, detune, slide])`: Release haengt
> HINTEN an (Laenge = dauer + release, ueberlappt die naechste Note wie bei
> jedem Synthesizer), Detune-Schicht als Chorus, Portamento in Halbtoenen.
> (2) **Zwei Klaenge zu EINEM machen ging nicht** -- ein Song liess sich
> also nicht als WAV abliefern. Neu `AUDIO_SOUND_NEW` (Stille als Leinwand),
> `AUDIO_SOUND_MIX` (addieren ab Offset, vol, Equal-Power-Pan, Abtastrate
> wird angeglichen, Ueberhang faellt weg, NICHT geklemmt) und
> `AUDIO_SOUND_NORMALIZE`. Der Arc-Puffer wird in place beschrieben, solange
> ihn niemand haelt; spielt der Klang gerade, arbeitet eine Kopie.
> (3) **`JSON_APPEND_NULL`** -- eine Liste mit leeren Plaetzen liess sich
> nicht schreiben, und genau so notiert das Tracker-Format eine Reihe ohne
> Note. Die Datei ist deshalb DASSELBE JSON wie das der Qt-Fassung, in beide
> Richtungen gegen deren Leser geprueft (`drachenhauch.tracker.Song`).
>
> Drei Entscheidungen im Piloten: **Die Wiedergabe laeuft auf einer
> Audio-Uhr** (`AUDIO_CLOCK` + `AUDIO_PLAY_AT`, zwei Reihen voraus geplant),
> nicht bildgetrieben -- ein Bild sind 16 ms, und das hoert man bei
> Sechzehnteln. **Gestoppt wird durch ENTFERNEN der Uhr**, nicht durch
> Anhalten: ein Klang, der auf eine Uhr wartet, die es nicht mehr gibt,
> startet nie (Kira: `WhenToStart::Never`); angehalten kaeme er beim
> naechsten Start als Geisternote an einer anderen Stelle wieder. **Der
> Verlauf merkt sich den ganzen Song als JSON-Text** -- dieselbe Routine wie
> das Sichern, und sie deckt Noten, Patterns, Reihenfolge, Tempo und
> Instrumente ab, ohne fuer jede Aenderungsart eine eigene Buchfuehrung; der
> Cursor bleibt dabei stehen. WAV-Mischung und Wiedergabe sind EINE Routine
> (`reiheAusgeben`) mit zwei Zielen -- sonst klaenge die Datei anders als
> das, was man hoert.
>
> **Zwei Funde beim Testen.** (1) `IF Leertaste AND NOT spielt THEN starten`
> gefolgt von `IF Leertaste AND spielt THEN stopp` startete und stoppte im
> SELBEN Bild -- die zweite Zeile sah den eben gesetzten Zustand schon.
> Eine Abfrage, zwei Zweige. (2) **Eine eingespeiste Taste bleibt
> GEDRUECKT, bis ein KEY_UP kommt** -- raylib aendert den Tastenzustand nur
> ueber Ereignisse. Die Test-Harnesse der vier Piloten davor haben nie eine
> Taste losgelassen und es nie gemerkt, weil dort keine Taste zweimal kam:
> hier hing nach dem ersten Strg+C die Strg-Taste fuer den Rest des Laufs,
> Pfeile zaehlten nicht mehr, und ein zweites Z gab keine Flanke fuer
> `KEYHIT`. Und wieder einmal: **die ueberlappenden Kaestchen "Stereo" und
> "Amiga" und die abgeschnittenen Knoepfe beider Werkzeugleisten hat nur das
> BILD gezeigt** -- der Test dazu prueft jetzt alle 35 Rechtecke paarweise
> und gegen den Fensterrand. Tests `tests/test_pilot_tracker.py` (17, mit
> fremden Lesern fuer JSON, WAV und DH-Code; der DH-Code wird gestartet, nicht
> nur uebersetzt), `tests/pruef/ton/audio_note_mix.dhtest`.
>
> **Der sechste Pilot ist keine Werkzeug-Portierung, sondern eine
> Geschaeftsanwendung** (2026-09-05, `examples/196_rechnungen.dh`, 1100
> Zeilen, ohne Qt-Gegenstueck -- deshalb NICHT in `piloten.py`): Kunden,
> Artikel, Rechnungen mit Positionen, SQLite als Wahrheit, PDF und CSV,
> Menues mit Kuerzeln, drei Reiter, Formulare mit Pruefung, Rueckfrage
> "Speichern|Verwerfen|Abbrechen" bei ungesicherten Aenderungen, Einstellungen
> in einem modalen Fenster mit rollendem Panel, groessenveraenderbar ueber
> Anker + Layout-Behaelter. Geld ist INTEGER in Cent (`INT(19.99 * 100)` ist
> 1998), die MwSt wird je Position gerundet. Er sollte nach den sechs
> gui-Ausbaustufen zeigen, was einer ANWENDUNG noch fehlt -- vier Funde:
> (1) **`PDF_TEXT_WIDTH` ging nur bei Courier**, ein Betrag liess sich in
> Helvetica nicht rechtsbuendig setzen. Die Schriftmasse fuer Helvetica und
> Times kamen damals aus PyMuPDFs Base-14-Metriken; seit dem Umbau auf krilla
> (2026-09-14) sind die Schriften eingebettet und `PDF_TEXT_WIDTH` misst mit
> derselben Textformung, mit der gezeichnet wird. (2) **Kaestchen, Radio und Kippschalter waren nur so breit
> wie ihr Kaestchen**: die Beschriftung hing ausserhalb des Rechtecks, ein
> Klick auf den Text traf nichts, und in einem Layout-Behaelter lief der
> Text ueber den rechten Rand. Jetzt misst `auto_w` das Rechteck ueber den
> Text, gezeichnet werden Kaestchen/Ring/Pille in ihrer eigenen Groesse
> (`check_size`), unabhaengig von der Breite -- alte `.dhform`-Dateien mit
> schmalem `w` zeichnen deshalb unveraendert. (3) `GUI_SET_ACTIVE_TAB` gab
> es, stand aber nicht im `builtin_index.json` -- `--check` meldete den
> Befehl als unbekannt, die IDE bot ihn nicht an. (4) Ein Feld waechst nicht
> per `a = a + [x]` (Laufzeitfehler "erwartet Zahlen"), sondern mit
> `ARRAY_PUSH` -- kein Fehler der Sprache, aber die Meldung fuehrt in die
> Irre. Tests `tests/test_pilot_rechnungen.py` (11; Datenbank liest Pythons
> `sqlite3`, das PDF PyMuPDF, den Export das `csv`-Modul; die Knopflagen
> kommen aus einem ersten Lauf, weil Layout-Behaelter sie erst zur Laufzeit
> verteilen).
>
> **In der IDE erreichbar** (seit 2026-08-31): `Datei` -> `Werkzeuge in
> Drachenhauch` startet jeden Piloten ueber dieselbe Konsole wie jedes
> andere Programm, ein Eintrag darunter oeffnet alle fuenf Quelltexte als
> Tabs. Einzige Quelle editor_qt/piloten.py; ein Test prueft Existenz UND
> Zeilenzahl (er fand sofort zwei falsche).
>
> **Jeder Pilot liest seit 2026-08-31 zurueck, was er schreibt** (vorher
> stand hier "eigene Presets speichern/laden nicht portiert"): SFX und
> Partikel sichern ihre Regler als `.ini` (Schluessel = Beschriftung, also
> lesbar und von Hand aenderbar; ein unbekannter Schluessel wird UEBERGANGEN
> statt gemeldet), der Sprite-Editor holt einen Streifen samt Atlas-JSON
> zurueck (dort sind die EBENEN weg -- der Streifen IST das
> zusammengerechnete Bild) und hat seit 2026-08-31 ein eigenes Format MIT
> Ebenen: `.dhsprite` = ein Raster-PNG (Spalten = Einzelbilder, Zeilen =
> Ebenen) plus eine JSON-Datei daneben (Masse, Ebenennamen, Sichtbarkeit,
> ueber das json-Modul geschrieben -- Namen sind Nutzertext, die Maskierung
> will man nicht selbst schreiben). Bewusst NICHT die Pixel als base64 in
> die JSON wie die Qt-Fassung: ein PNG, das jeder Bildbetrachter oeffnet,
> ist mehr wert als ein Format, das nur sein Erzeuger ansehen kann. Der
> Tilemap-Editor
> konnte es von Anfang an ueber `TILED_LOAD`. Das Sichern kostete die
> Faktoren: SFX 0,77 -> 0,93, Partikel 0,48 -> 0,58. Alle vier Rundwege sind
> nachgefahren (malen -> sichern -> zuruecksetzen -> laden -> vergleichen).
>
> **Der Sprite-Pilot hat seit 2026-09-01 Lasso und Zauberstab** -- und mit
> ihnen eine echte Punkt-MASKE statt eines Auswahl-Rechtecks. Rechteck,
> Lasso und Zauberstab schreiben in dieselbe Maske, und ALLES was zeichnet
> fragt sie (`block` ist der einzige Ort, an dem Farbe ins Bild kommt);
> vorher galt die Auswahl nur fuers Kopieren, ein Strich lief einfach
> hindurch. Das Lasso fasst den gezogenen Weg als Vieleck auf und prueft ihn
> mit `PHYSICS_POINT_POLY` -- der Befehl war laengst da, nur nie fuer so
> etwas benutzt. **Zwei Fallen, beide erst im gerenderten Bild sichtbar:**
> Vieleck auf den ECKEN der Punkte und Testpunkt in ihrer MITTE laesst einen
> 45-Grad-Rand genau auf eine Kante fallen -- eine ganze Reihe einzelner
> Punkte blieb ungewaehlt, MITTEN in der Auswahl (beide Seiten muessen
> dasselbe Raster benutzen); und ein grob aufgezeichneter Weg braucht
> LUECKENLOSE Zwischenpunkte, sonst liegen Vieleck-Rand und gezeichneter
> Rand auseinander. Tests: `tests/test_pilot_sprite_auswahl.py` faehrt echte
> Mauswege ueber `AUTOMATION_PLAY` (die Fenstergeometrie steht erst zur
> Laufzeit fest, darum laeuft jeder Test zweimal).
>
> **Verschieben kam einen Tag spaeter** und war billig, weil die Maske schon
> da war: abheben (die Punkte kommen in eine eigene Ablage und werden im Bild
> GELOESCHT -- deshalb sieht man ein Loch statt einer Kopie), ziehen,
> absetzen; ohne Auswahl gilt die ganze Ebene. Die MASKE wandert mit, sonst
> zeigte sie auf die Stelle, wo das Verschobene nicht mehr ist. Dabei fiel
> ein Loch auf, das alle Werkzeuge betraf: die Zug-Abschluesse fragten
> `werkzeug`, nicht das Werkzeug, mit dem der Zug BEGANN -- ein
> Werkzeugwechsel mitten im Zug haette beim Verschieben das Abgehobene nie
> wieder abgesetzt (jetzt `ziehWz`). Und `GETPIXEL` + `GETALPHA` getrennt
> aufzuheben und spaeter nur die Farbe zu setzen macht aus jedem
> halbdurchsichtigen Punkt einen deckenden -- dafuer `mitAlpha`, das auch das
> Einfuegen benutzt.
>
> **GIMP-Paletten (.gpl)** kann er seit 2026-09-01 laden und sichern -- ein
> Textformat, und genau deshalb interessant: es ist das, was Aseprite, GIMP,
> Krita und die Paletten-Sammlungen im Netz sprechen. Die Trennung darin ist
> beliebig viel Leerraum samt Tabulatoren, und die Dateien werden von Hand
> bearbeitet: eine krumme Zeile wird UEBERGANGEN statt gemeldet, sonst
> verhindert ein Tippfehler die ganze Palette. Die Palette hat 16 Plaetze --
> was darueber hinausgeht, sagt die Statuszeile, statt es still fallen zu
> lassen. Geprueft wird gegen einen FREMDEN Leser (`_parse_gpl` des
> Qt-Sprite-Editors) und in der Gegenrichtung gegen dessen Schreiber.
>
> **Kachel-Ansicht und Statistik** kamen am 2026-09-01 dazu -- und mit ihnen
> der bisher schwerste Pilotenfund: **ein gui-Fenster ueber einer
> Zeichenflaeche war UNSICHTBAR.** `GUI_DRAW` zeichnet Fenster und
> Zeichenflaechen in EINEM Durchgang, und der Inhalt einer Zeichenflaeche
> entsteht per Bauart danach -- das Programm malt selbst hinein. Im Piloten
> ging der Kasten "Neues Sprite" damit auf, sperrte die Eingabe und war nicht
> zu sehen: das Programm wirkte eingefroren. Das stand seit dem ersten Tag im
> gemergten Piloten, und weder ein Test noch ein Bildschirmfoto hat es
> bemerkt -- niemand hatte je auf [Neu] geklickt und HINGESEHEN. Ein zweites
> `GUI_DRAW` hilft nicht (es zeichnet Fenster-Hintergrund und
> Zeichenflaechen neu, also genau das Gemalte); deshalb neu:
> **`GUI_DRAW_WINDOW(win)`** -- ein Fenster noch einmal, ueber allem, was
> inzwischen im Bild steht, samt Schleier wenn es das modale ist. Der
> Schleier liegt jetzt in EINER Routine (`schleier`), damit die zwei Wege
> nicht auseinanderlaufen. Tests `tests/pruef/gui/gui_draw_window.dhtest` mit der
> Gegenprobe IM Test: derselbe Ablauf einmal mit und einmal ohne den Aufruf,
> geprueft am Punkt in der Fenstermitte.
>
> **Zuschneiden, Groesse aendern und Animationsbereiche** (2026-09-01)
> brachten die letzten drei Funde. Zwei davon lagen seit dem ersten Tag im
> gemergten Piloten: (1) die beiden Neben-Fenster ("Neues Sprite",
> "Groesse aendern") waren nicht nur unsichtbar, sondern auch **nicht
> anklickbar** -- ein Klick bringt immer sein Fenster nach vorn, und der
> Klick auf den oeffnenden Knopf war einer auf das bildschirmfuellende
> Hauptfenster; danach lag es darueber und fing jeden Klick ab. `GUI_FOCUS`
> auf das erste Feld holt es nach vorn. (2) **`GUI_CLICKED` auf einem
> Kaestchen war stumm** -- siehe gui unten; der "sichtbar"-Schalter beider
> Piloten (187 UND 189) war deshalb tot. Zuschneiden geht ueber ALLE Ebenen,
> auch ausgeblendete: nach der Sichtbarkeit zu gehen wuerde Inhalt
> wegschneiden, den man gerade nicht sieht. Die Bereiche wandern beim
> Loeschen eines Bildes mit (`bereicheNachLoeschen`), sonst spielte die
> Vorschau danach etwas anderes, ohne dass sich sichtbar etwas geaendert
> haette.
>
> **Die DH-Code-Ausgabe schliesst den Kreis** (2026-09-01): [DH-Code]
> schreibt ein LAUFFAEHIGES Programm samt Blatt -- eine
> `SPRITE_ADD_ANIM`-Zeile je Bereich, ohne Bereiche eine ueber alles
> ("idle"), sonst faende `SPRITE_PLAY` nichts. Beides in EINEM Zug und unter
> demselben Namen; wer nur den Code schreibt, hat einen Verweis ins Leere,
> und das faellt erst beim Starten auf. Der Test uebersetzt ihn nicht, er
> STARTET ihn -- ob erzeugter Code uebersetzt, sagt nichts darueber, ob er
> sein Blatt findet und eine Animation kennt.
>
> **Dieselben Bereiche auch als Zustandsmaschine** ([dhanim], 2026-09-01): ein
> Zustand je Bereich, der erste als Start. Uebergaenge und Parameter bleiben
> LEER -- welcher Zustand wann in welchen wechselt, ist eine Aussage ueber das
> SPIEL, die der Sprite-Editor nicht treffen kann; die Datei ist eine Vorlage,
> die `dhanim` oeffnet und die `ANIM_FSM_LOAD` schon so laedt. Geprueft mit
> dem Leser der LAUFZEIT: laden, `ANIM_FSM_SETUP`, ein Schritt, Zustandsname
> vergleichen -- dass die JSON gueltig ist, waere die schwaechere Aussage.
>
> **Benannte Einzelbilder** (2026-09-01) schliessen die Liste: der Name wird
> zum Schluessel im Atlas, ein Programm schreibt dann
> `ATLAS_DRAW(atlas, "kopf", ...)` statt `"bild_0"`. Namen sind KENNUNGEN und
> kein Fliesstext -- gesaeubert auf Buchstaben, Ziffern, `_` und `-`, und das
> aus einem gemessenen Grund: das json-Modul liest einen Schluessel mit PUNKT
> als PFAD, aus "held.lauf" wuerde beim Schreiben ein verschachteltes Objekt
> statt eines Eintrags, und `streifenLaden` sucht spaeter mit derselben
> Punkt-Notation. Dass Nutzertext in den Atlas kommt, hat ausserdem
> `atlasJson$` vom handgebauten Text auf das json-Modul gebracht -- die alte
> Begruendung ("fuer eine so flache Datei kuerzer") galt nur, solange alle
> Schluessel `bild_0` hiessen. Geprueft mit `ATLAS_LOAD` der Laufzeit, samt
> Gegenprobe auf den alten Schluessel; dass der Pilot ein Format schreibt,
> das ATLAS_LOAD liest, stand bis dahin nur als Behauptung im Kopfkommentar.
>
> **Rueckgaengig hat seit 2026-09-02 auch der zweite** (Faktor 0,58 ->
> 0,78) -- und dort stellte sich eine Frage, die bei Kachel- und
> Pixel-Editoren gar nicht auftaucht: **wann ist ein Schritt fertig?** Ein
> gezogener Regler aendert sich in JEDEM Bild; jedes davon aufzuzeichnen
> ergibt nach einer Sekunde 60 Schritte, und Strg+Z nimmt ein Sechzigstel
> der Bewegung zurueck. Gesichert wird deshalb erst, wenn sich ein Bild lang
> nichts geaendert hat UND keine Maustaste mehr haengt -- ein Zug ist ein
> Schritt, und eine Werkseinstellung (20 Werte auf einmal) auf demselben Weg
> ebenfalls. Gespeichert wird der VOLLE Stand (20 Zahlen), nicht die
> Aenderung: bei der Groesse ist das kuerzer als jede Buchfuehrung darueber.
> Kein Ringpuffer wie bei den anderen beiden -- bei 20 Zahlen je Stand
> kostet das Aufruecken nichts, und eine Liste ohne Modulo versteht man noch.
> Zwei eigene Fallen: nach einem Rueckgaengig muss die Aenderungs-Erkennung
> den neuen Stand als BEKANNT ansehen (`letzteAuffrischen`), sonst zeichnet
> sie ihn als frische Aenderung auf und der Vor-Weg ist weg; und der
> Einfuegeplatz ist `uPos + 1` -- beim allerersten Stand gibt es aber keinen
> Vorgaenger, da muss es 0 sein (sonst steht Schritt 0 leer und der Start
> zaehlt als zwei).
>
> **Dabei fiel ein Fehler auf, den kein Test je gesehen hatte:** [Sichern]
> und [Laden] lagen seit dem 2026-08-31 genau auf den ersten beiden
> Eintraegen der Werkseinstellungs-Liste (beide bei x = 16, y = 494, die
> Liste ebenso). "Funken" war verdeckt und fing seine Klicks nicht mehr --
> der Knopf darueber tat es. Sichtbar nur im BILD. Der Test dazu fragt
> `GUI_HIT_TEST` statt zu klicken: ein echter Klick wuerde beim alten Stand
> einen Datei-Dialog oeffnen und den Lauf haengen lassen.
>
> **Seit 2026-09-02 haben ALLE VIER Rueckgaengig** -- und der erste (SFX)
> hat dabei den lehrreichsten Wert geliefert: **Faktor 1,18**, die
> Drachenhauch-Fassung ist LAENGER als die Qt-Fassung. Der Grund ist nicht
> Wortreichtum, sondern was in der Qt-Zahl FEHLT: dort kostet das Undo rund
> 15 Zeilen Verdrahtung, weil `SnapshotUndo` (140 Zeilen,
> editor_qt/undo_history.py) in einem gemeinsamen Modul liegt und von VIER
> Qt-Editoren benutzt wird -- gezaehlt wird es bei keinem. Rechnet man es
> dazu, steht es 631 zu 662, also wieder 0,95. Der Pilot traegt seine 90
> Zeilen selbst. **Damit misst der Faktor auch das: was die Vergleichszahl
> nicht enthaelt.** Der dritte und vierte Pilot loesen es anders (Ringpuffer,
> je Schritt der Vorher/Nachher-Stand der betroffenen Ebene; beim
> Sprite-Editor werden die Plaetze EINMAL angelegt und wiederverwendet --
> `IMAGE_FREE` gibt es seit dem Pilotenbefund zwar, aber gar nicht erst
> anzulegen ist billiger als anlegen und freigeben).
>
> **Und derselbe Layout-Fehler steckte auch im ersten** (gefunden beim
> Einbauen, gesehen nur im BILD): "Bereit." und die Dauer-Anzeige standen bei
> y = 482 und 504 -- mitten auf [Sichern] und [Laden] (478..506), Knopftext
> und Meldung uebereinander gedruckt. Beide Piloten hatten ihn seit dem
> 2026-08-31, beide durch dieselbe Ursache: neue Knoepfe eingesetzt, ohne zu
> pruefen, was dort schon lag. **Der erste Testversuch dafuer war gruen und
> wertlos** -- `GUI_HIT_TEST` liefert an der Stelle die zuletzt angelegte
> Beschriftung, also in beiden Lagen dasselbe; eine Beschriftung nimmt keine
> Klicks an, der Schaden ist rein optisch. Geprueft werden jetzt die
> RECHTECKE. Beim Partikel-Editor traf es eine LISTE, die Klicks sehr wohl
> annimmt -- dort ist der Treffertest die richtige Frage.

## Klassendiagramm (2026-10-25)

`examples/210_klassendiagramm.dh`: die Klassen eines Projekts als Kaesten
(Felder, Konstanten, Eigenschaften, Methoden mit Typ) aus `CODE_CLASSES$`,
Vererbung mit offenem Dreieck, ein Feld vom Typ einer anderen Klasse
gestrichelt; Ebenen nach der Vererbung, Kaesten ziehbar (Lagen in
`.klassendiagramm.json` im Projekt), F5/L/P, das ganze Diagramm als PNG
ueber ein Rendertarget. **Doppelklick springt in die IDE:** das Werkzeug
schreibt `DH_IDE_OEFFNEN|pfad|zeile`, und die IDE liest jetzt die Ausgabe
ihrer Werkzeuge (`werkzeugeAufraeumen` -> `werkzeugZeile`; vorher las sie
niemand, die Leitung haette sich gefuellt). Eine Datei, die es nicht gibt,
laesst die Marke stehen (sonst sprang der Reiter vorn auf die Zeile).
Werkzeuge -> Klassendiagramm nimmt den Projektordner. **Fund im
Pruefformat:** Text hinter `--- programm` (und den anderen Koepfen ohne
Inhalt) wurde stillschweigend uebergangen -- der Quelltext eines neuen
Falls lief so nie, und der Fall schwieg; jetzt ein Fehler, Leerzeilen und
Kommentare bleiben erlaubt (alle 444 Sammlungen lesen sich weiter ein).
Tests `tests/pruef/werkzeuge/werkzeug_klassendiagramm.dhtest` (4: Ebenen, Lagen,
Sprungwunsch, Bild, die IDE ueber einen echten Werkzeug-Prozess), Rust-Test
`text_hinter_programm_ist_ein_fehler`; sechs Verfaelschungen fallen.

## Ton-Werkzeuge ueberarbeitet (2026-10-29)

Wunsch des Nutzers: SFX-Generator, Tracker und Notenblatt komfortabler und
ansehnlicher, "der Notenschluessel sieht nicht perfekt aus". GTK war nicht
noetig -- alles mit der gui der Laufzeit, keine Laufzeit-Aenderung.
**Alle drei:** DejaVu ueber `LOADFONT("sans", 32, "▶■")` + `SETFONT(f, 15)`
statt der Pixelschrift (Zeichen ausserhalb des Grundvorrats muessen in die
Zeichenliste -- sonst `?`), Fenster groessenveraenderbar mit Ankern,
Werkzeugleiste mit Sinnbildern, Statusleiste, Umlaute in der Anzeige.
**SFX (183):** Menue mit Kuerzeln (R Zufall, M Mutieren, Leertaste), zwoelf
Vorlagen, **Zufall nach Art** (`zufallArt`: Grenzen je Vorlage),
**Mutieren** (6 % des Bereichs, nur Werte ungleich 0), der Verlauf als Liste
mit Namen je Schritt (`uName`, Klick holt ihn zurueck), die Wellenform gross
mit **Huellkurve zum Ziehen** (drei Punkte, Zeitachse bleibt waehrend des
Zugs stehen), Abspielkopf und Pegel (`AUDIO_PEAK`). Die `.ini`-Schluessel
behalten die alte Schreibweise (`parName`), angezeigt wird `anzeige`.
**Tracker (190):** Mono im Gitter, linke Spalte mit der Zelle, Song-Uebersicht
als Kachelreihe (Klick waehlt), **Pegel je Kanal** aus dem Anschlag (vier
Plaetze je Kanal nach Reihe -- es wird zwei Reihen VORAUS geplant, ein Platz
wuerde den klingenden Anschlag ueberschreiben), mitlaufende Reihe, Pattern
umbenennen (`GUI_PROMPT`), leuchtende Klaviatur. **Falle:** ein Drehfeld ist
mit DejaVu 15 genau 32 hoch, die Zeilen lagen 30 auseinander --
`flach()` setzt sie per `GUI_SET_BOUNDS` auf 28 (der Lage-Test fand es).
**Notenblatt (199):** Schluessel, Koepfe, Pausen, Vorzeichen aus **Noto
Music** (seit dem Folgeschritt in dhrt eingebaut, `LOADFONT("noten")`, OFL in
`daten/schriften/OFL-NotoMusic.txt`; ohne eingebaute Schriften bleiben die
gezeichneten Formen). Gesetzt ueber Masse in Font-Einheiten (`gBox`, ueber
`FONT_GLYPH_BOX` aus der Datei):
Notenkopf 270 Einheiten = ein Linienabstand, also `e = 2*HALB/268`,
raylib-Groesse `1787*e` (hhea 1389/-398), Oberkante = Ziel - (1389 - ey)*e;
G-Schluessel mit Einheit 244 auf der G-Linie, F-Schluessel 512 auf der
F-Linie (beides im Bild nachgemessen). **Falle:** die gui zeichnete mit der
AKTIVEN Schrift -- ohne `SETFONT(uiFont, 15)` nach jeder Glyphe schrieb die
Oberflaeche in der Notenschrift (seit dem Folgeschritt `GUI_FONT`). Dazu Taktart 4/4 und Taktnummern
(`LEFT_MARGIN` 76 -> 100, die Klicklagen der Tests +24), **Zoom** (die
Masse sind Variablen, `zoomRechnen`; beim Start nach der Fensterhoehe),
**Eingabe per Tastatur** an einer Eingabemarke (C D E F G A H B, P, 1..5,
Punkt, Pfeile, Ruecktaste), Klaviatur unten, Notenwerte als Knopfleiste.
Der Fokus geht nicht mehr in die Dauer-Klappliste, sondern wird mit
`GUI_FOCUS(-1)` freigegeben -- eine Klappliste mit Fokus nimmt Buchstaben
("Tippen springt"), "a" haette die Dauer auf Achtel gestellt. Tests: je
Werkzeug ein Fall "nichts liegt uebereinander" ueber ALLE Widgets, dazu
Zufall/Mutieren/Verlauf/Huellkurve, Umbenennen/Uebersicht, Tastatureingabe
(Gegenprobe: ohne `tastenEingabe()` faellt er).

**Folgeschritt in der Laufzeit (2026-10-29, Wunsch des Nutzers: "die
Stolpersteine beheben"):** (1) **`GUI_FONT(font [, groesse])`** /
`GUI_FONT_GET()` -- die gui hat eine eigene Schrift; `Vm::try_gui` tauscht
sie fuer JEDEN gui-Befehl ein (`Graphics::schrift_tauschen`) und danach
zurueck, bei GUI_UPDATE schon VOR den Rueckrufen (`gui_tausch_zurueck`), die
sind Programmcode. (2) **Fehlende Zeichen aus der eigenen Datei**: ein
geladener Font, dem ein Zeichen fehlt, merkt es in `eigen_fehlt`, FLIP backt
ihn mit diesem Zeichen neu (`eigene_nachbacken`, gleiches Handle); was auch
dort fehlt, steht in `eigen_nie` und geht an die Ausweich-Schriften. Vorher
fragte die Laufzeit nur die Systemschriften -- das ▶ von `LOADFONT("sans")`
kam als "?" oder aus Segoe. (3) **`GUI_FOCUS(-1)`** gibt den Fokus frei.
(4) **`FONT_GLYPH_BOX(font, zeichen$, groesse)`** -> (x, y, b, h) relativ zum
Punkt, an den TEXT setzt, und **`FONT_BASELINE(font, groesse)`** -- ueber
`ttf-parser` (im Bau schon da, jetzt im graphics-Feature), Massstab wie stb
(`groesse / (hhea.asc - hhea.desc)`); und **`LOADFONT("noten")`** = Noto
Music eingebettet (`schriften.rs`, `daten/schriften/`, 124 KB; der
PDF-Fehlertext nennt sie mit, die OFL steht in den Drittanbieter-Lizenzen
ueber `installer/lizenzen.dh`). Die drei Werkzeuge benutzen alles davon; die
Kopie der Schrift in `examples/assets/` ist weg. Tests
`tests/pruef/grafik/schrift_bausteine.dhtest` (8; gegen 2026.37 fallen 7, der
achte ist die Gegenprobe ohne GUI_FONT).

## Grafik-Editoren aufgefrischt (2026-10-29)

Wunsch des Nutzers: nach den Ton-Werkzeugen auch Partikel (185), Tilemap
(187), Sprite (189) und Anim-FSM (198) -- der Form-Designer (197) bleibt fuer
spaeter, zusammen mit "Drachenhauch soll fuer GUIs aehnlich wie Xojo
programmierbar sein". Gemeinsam: `LOADFONT("sans")` + `GUI_FONT`, Umlaute in
der Oberflaeche (Schluessel in Dateien und Protokolle bleiben ASCII), Menue
mit Kuerzeln, Werkzeugleiste mit Sinnbildern, `GUI_STATUSBAR`. **Ein Kuerzel
steht nur im Menue** -- dieselbe Taste zusaetzlich mit KEYHIT abzufragen
liesse den Befehl zweimal laufen; Strg+C/X/V/D bleiben bei Tilemap und
Sprite KEYHIT, weil sie an der Auswahl haengen. **In der Laufzeit:** 18 neue
Sinnbilder (`stift`, `radierer`, `fuellen`, `linie`, `rechteck`, `ellipse`,
`pipette`, `auswahl`, `lasso`, `zauberstab`, `verschieben`, `spruehen`,
`wuerfel`, `raster`, `auge`, `auge_zu`, `spiegeln`, `drehen`) und
`DRAWIMAGEPARTEX(..., farbe)` -- das zehnte Argument toent, sein Alpha ist
die Deckkraft (`Cmd::TexturePartExTint`); ein elftes ist jetzt ein Fehler
(2026.37 schluckte es still). Tests `tests/pruef/grafik/drawimagepartex.dhtest`.

**Partikel:** Farbwaehler statt sechs RGB-Reglern (`p(idx)`/`setzeP` fuehren
die Farben weiter als sechs Anteile, damit Vorlagen, Verlauf und `.ini`
gleich bleiben), Zufall/Mutieren (Paare min <= max), vier Vorlagen mehr,
Hintergruende, ein Streifen "Teilchen im Lauf seines Lebens". **Fund:** die
`.ini` schrieb beide Farben unter `Rot`/`Gruen`/`Blau` -- die Endfarbe
ueberschrieb die Anfangsfarbe; jetzt `Rot Anfang`/`Rot Ende`, eine alte Datei
bringt ihre Endfarbe mit. **Anim-FSM:** Fenster oben (Menue + Leiste) und
unten (Status); `OBEN_H` misst die Menueleiste ueber die Laufzeit. Der Graph
hat Weltkoordinaten -- EINE Rechnung `schirmX/Y` hin, `weltX/Y` zurueck fuer
Zeichnen und Treffer; Rad zoomt um die Maus, mittlere Taste/Leertaste
verschiebt, `anordnen` (Breitensuche ab Start, Ziele von "Any State" in Stufe
1), `einpassen` (beim Oeffnen per Dialog und bei der Demo -- NICHT beim Start
mit Argument, die Pruefungen klicken feste Lagen bei Zoom 1). Rechts unten
laeuft die Animation des gewaehlten Zustands aus dem Blatt; die Demo nennt ihr
Blatt vom Wurzelordner aus, `blattPfad$` sucht darum auch eine Stufe ueber
der `.dhanim`. **Tilemap:** Linie/Ellipse ueber `formZellen` (eine Rechnung
fuer Vorschau und Setzen), Stempel aus der Palette (Zug = Block, nur der
Stift malt ihn ganz), Uebersicht rechts unten (`miniBauen` je Kachel die
Mittelfarbe, gebaut wenn `miniVeraltet`, beim Malen hoechstens jedes zehnte
Bild; ein Klick darin malt nicht), Ebenen-Kaestchen = Sichtbarkeit,
Werkzeug-Buchstaben nicht im Textfeld. **Sprite:** Werkzeugspalte (zwoelf
Sinnbild-Knoepfe, das aktive mit Rahmen -- flache Sinnbild-Knoepfe zeigen
eine Hintergrundfarbe kaum), Bildleiste mit Vorschaubildern, zuletzt
benutzte Farben, Deckkraft je Ebene (`ebDeck`, im `.dhsprite` als
`deckkraft`, nur wenn nicht deckend; `IMAGE_DRAW_IMAGE` mit Faerbung in jeder
Ausgabe). **Die Pruefungen:** ein Menue verschiebt alle Widget-Lagen im
Hauptfenster -- die Faelle rechnen `prOy()` = `GUI_CANVAS_Y(leinwand) -
GUI_GET_Y(leinwand)` dazu (Nebenfenster nicht, die rechnen mit
`GUI_WINDOW_GET_Y`); Knoepfe in der Leiste ueber `GUI_TOOLBAR_ITEM_X/W`; die
Anim-FSM-Aufnahmen sind um die Hoehe der neuen Leisten verschoben. Neue Faelle
je Editor (Partikel 5, Anim-FSM 5, Tilemap 5, Sprite 4), je mit Gegenprobe.

## Die Werkzeuge (in Drachenhauch)

Die frueheren Qt-Werkzeuge (`dhsprites`, `dhtilemap`, `dhform`, `dhscore`,
`dhanim`, `dhtracker`, `dhsfx`, `dhparticles`, Audio Studio) sind geloescht.
Ihre Nachfolger sind Drachenhauch-Programme, in der IDE unter *Werkzeuge*:

| Werkzeug | Programm | Handbuch | Pruefung |
|---|---|---|---|
| SFX-Generator | `examples/183_sfx_generator.dh` | `docs/sfx-generator.md` | `tests/pruef/werkzeuge/werkzeug_sfx.dhtest` |
| Partikel-Editor | `examples/185_partikel_editor.dh` | `docs/particle-editor.md` | `tests/pruef/werkzeuge/werkzeug_partikel.dhtest` |
| Tilemap-Editor | `examples/187_tilemap_editor.dh` | `docs/tilemap-editor.md` | `tests/pruef/werkzeuge/werkzeug_tilemap.dhtest` |
| Sprite-Editor | `examples/189_sprite_editor.dh` | `docs/sprite-editor.md` | `tests/pruef/werkzeuge/werkzeug_sprite.dhtest` |
| Tracker | `examples/190_tracker.dh` | `docs/tracker.md` | `tests/pruef/werkzeuge/werkzeug_tracker.dhtest` |
| Form-Designer | `examples/197_form_designer.dh` | `docs/form-designer.md` | `tests/pruef/werkzeuge/werkzeug_formdesigner.dhtest` |
| Anim-FSM-Editor | `examples/198_anim_fsm_editor.dh` | `docs/anim-editor.md` | `tests/pruef/werkzeuge/werkzeug_animfsm.dhtest` |
| Notenblatt | `examples/199_notenblatt.dh` | `docs/score-editor.md` | `tests/pruef/werkzeuge/werkzeug_notenblatt.dhtest` |
| Klassendiagramm | `examples/210_klassendiagramm.dh` | `docs/klassendiagramm.md` | `tests/pruef/werkzeuge/werkzeug_klassendiagramm.dhtest` |

Jedes Handbuch nennt am Ende, was die Qt-Fassung zusaetzlich konnte. [Neu] und [Oeffnen] im Notenblatt fragen
seit 2026-09-21 bei ungesicherten Aenderungen nach (PR #211; hier stand bis
2026-09-23 noch, sie taeten es nicht).

## Python-Abbau, Weg B: der Form-Designer in Drachenhauch (2026-09-06)

Der erste der vier Editoren ohne Piloten (Form-Designer, Anim-FSM,
Notenblatt, Audio Studio): `examples/197_form_designer.dh`, 860 Zeilen gegen
5055 der Qt-Fassung (Faktor 0,17 -- misst wie immer vor allem, was
weggelassen ist: Mehrfachauswahl, Layout-Zuordnung, Regeln/Bindung,
Menue-Editor, Code-Editor, DH-Code-Export, Projekte fehlen). **Bauweise:**
die Entwurfsflaeche ist ein ECHTES GUI_WINDOW im neuen Entwurfsmodus
`GUI_WINDOW_DESIGN(win, TRUE)` (gui.rs `Window::entwurf`: `handle_press`
kehrt nach dem Modal-Check um -- Fenster nach vorn, KEIN Fokus, kein
Rahmen-Zug; der Hover-Durchlauf ueberspringt es; `GUI_HIT_TEST` und alle
Setter gehen weiter). Damit zeichnet die Laufzeit die Controls selbst,
statt dass der Designer sie nachmalt. **Das Modell ist das .dhform-JSON**
(json-Modul), das Fenster nur die Ansicht: jede Aenderung schreibt ins JSON
und baut die Ansicht neu (`GUI_WINDOW_DESTROY` + `GUI_FROM_JSON`); waehrend
eines Zuges gehen x/y live per `GUI_SET_BOUNDS` ins Widget, gemerkt wird
beim Loslassen. Undo = JSON-Text je Stand mit Zeiger. Widget-Index im JSON
= `GUI_WINDOW_WIDGET(frm, i)`. Fremde Felder (`code`, `menus`, `tabs`)
laufen unveraendert durch -- der Test schreibt sie hinein und liest sie
zurueck. F5 schreibt `<name>_lauf.dh` (dasselbe Geruest wie
`generate_runner`) und startet es per PROCESS_START. **Fallen:** (1) eine
Liste meldet kein `GUI_CLICKED` -- die Palette wird ueber die AUSWAHL scharf;
(2) ein neu gebautes Fenster nimmt den Fokus, Kuerzel gelten im
Fokus-Fenster -- `ansichtBauen` merkt `GUI_FOCUSED()` und gibt ihn zurueck,
sonst waren Strg+S/F5 nach dem ersten Ablegen tot; (3) `JSON_TYPE` sagt nur
`number` -- beim Kopieren eines Teilbaums eine ganze Zahl ganz lassen, sonst
liest `GUI_FROM_JSON` aus `48.0` kein x mehr; (4) `CONST RASTER` und
`FUNCTION raster` kollidieren (Namen sind schreibungsunabhaengig). Tests
`tests/pruef/werkzeuge/werkzeug_formdesigner.dhtest` (seit Stufe 32; bis dahin test_pilot_formdesigner.py -- echte Klicks ueber die Wiedergabe:
Palette -> Form -> Strg+S, Datei gelesen mit `FormDoc.load`, dem Modell des
Qt-Designers; Ziehen + zweimal Strg+Z; F5-Laufprogramm uebersetzt;
Entwurfsmodus mit Gegenprobe). In der IDE unter Werkzeuge, im Installer
ohne Python als Verknuepfung.

**Der Anim-FSM-Editor** (2026-09-07, `examples/198_anim_fsm_editor.dh`, 1336
Zeilen gegen 1728 = `animeditor_qt.py` + animeditor/document.py, Faktor
0,77 -- der hoechste seit dem SFX-Generator, weil hier fast nichts
weggelassen ist: der Qt-Editor ist klein und die Laufzeit hat fuer einen
Graphen nichts, das man wiederverwenden koennte). Der Graph ist die erste
Familie mit KANTEN: Knoten (Zustaende), Pfeile mit Spitze, Beschriftung und
seitlichem Versatz bei Hin- und Rueckweg, die Pille "Any State", der
Eingangspfeil zum Startzustand -- alles mit LINEW/TRIANGLE/BOXROUND in
einem SCISSOR-Bereich zwischen zwei gui-Fenstern gemalt; Treffertest auf
einen Pfeil = Abstand Punkt/Strecke <= 7 px, Randpunkte der Knoten aus
EINER Geometrie (`kante`), die Zeichnen und Treffertest teilen. **Uebergaenge
zieht man mit der RECHTEN Maustaste** von Knoten zu Knoten (oder `L` +
links): ein Modus-Schalter wie in Qt ("Link-Modus") ist ein Schalter, den
man vergisst. Doppelklick auf freie Flaeche = Zustand (eigene Erkennung
ueber MILLIS, `GUI_DOUBLE_CLICKED` gibt es nur fuer Listen). Modell =
.dhanim-JSON, Undo = JSON-Text je Stand, Inspektor mit bis zu sechs
Bedingungszeilen (fest angelegte Widgets, sichtbar so viele wie noetig).
F5 schreibt `<name>_vorschau.dh` (dasselbe Geruest wie `generate_runner`:
ui-Regler je Parameter, Sprite rechts) und startet es; das Sprite-Blatt
wird wie eingetragen, neben der .dhanim oder vom `DHRT_START_DIR` aus
gesucht und ABSOLUT eingetragen. **Zwei Fallen:** (1) `first`/`last`/`x`/`y`
muessen GANZE Zahlen bleiben -- animfsm.rs liest sie mit `as_i64`, aus
`3.0` wird `None`, und der Zustand haette keine Bilder; deshalb JSON_SET_INT
an jeder Stelle, die sie schreibt. (2) Ein Klick auf einen Knopf im
Inspektor gibt dessen Fenster den Fokus, und Kuerzel galten nur im
Fokus-Fenster -- `fokusZurueck` nach jedem Knopf, sonst war Strg+S nach
`+ Bedingung` stumm (der Test sah es: die Datei blieb alt). Seit die Kuerzel
in allen Fenstern gelten (siehe Menues unten), ist der Umweg wieder draussen.
Tests `tests/pruef/werkzeuge/werkzeug_animfsm.dhtest` (7, seriell; bis Stufe 33 pytest): Doppelklick, Rechtsziehen,
Ziehen + Strg+Z, Entf (der Laufzeit-Leser ist hier die Pruefung -- ein
stehengebliebener Uebergang laedt nicht), Bedingung ueber den Inspektor
mit Wirkungsprobe (speed 10 -> run, speed 1 -> idle), Parameter, F5-Vorschau
uebersetzt UND laeuft. Audio Studio wird NICHT portiert (123 Zeilen
Reiterrahmen; das Werkzeuge-Menue der IDE ersetzt es).

**Das Notenblatt** (2026-09-07, `examples/199_notenblatt.dh`, 1449 Zeilen
gegen 1710 = `scoreeditor_qt.py` + score/document.py + score/convert.py,
Faktor 0,85 -- der hoechste der drei, weil nichts wegfaellt: Notensatz muss
man zeichnen, der Tracker-Konverter ist Logik) -- damit hat JEDER Qt-Editor
eine Drachenhauch-Fassung. Fuenf Linien je Spur, Tonhoehe <-> Linie ueber
den diatonischen Index wie in Qt (C->D und E->F je EIN Schritt), Hilfslinien,
Koepfe/Haelse/Faehnchen, Balkengruppen gleicher Dauer, Pausen, Boegen als
SPLINE, Vorschau an der Maus, Spielkopf; Werkzeuge oben, je Spur Name/
Schluessel/Instrument links (die 18 Tracker-Presets), bis zu 8 Spuren.
**Schluessel und Vorzeichen sind selbst gezeichnet:** die Notenzeichen
U+1D11E/U+266F kamen trotz Glyphen-auf-Zuruf als Fragezeichen -- eine Kurve
durch feste Punkte (SPLINE, will ARRAY OF INTEGER) und vier Striche sind
besser als ein `?` am Zeilenanfang. Wiedergabe auf einer Audio-Uhr
(AUDIO_CLOCK + AUDIO_PLAY_AT, alle Noten beim Start geplant; Stopp =
Uhr ENTFERNEN wie im Tracker-Piloten). **[In Tracker oeffnen] rechnet mit
denselben Regeln wie score/convert.py** und der Test vergleicht das
Gitter des Piloten Zelle fuer Zelle mit `to_tracker_song` am Demo-Stueck
`examples/notenblatt_demo.json` (Akkord, Staccato, Note ueber die
64-Zeilen-Grenze, zwei Spuren); der Tracker-Pilot 190 nimmt seither ein
Dateiargument. Drei fremde Leser: `ScoreDoc.load_json`, `Song.load_json`,
`to_tracker_song`. **Fallen:** `STEP` ist ein Schluesselwort (kein `CONST
STEP`); `GUI_MODAL()` liefert einen Wahrheitswert, kein Handle; Klapplisten
in Tests: Eintrag k liegt bei y + 24 + k * 22 + 11 unter dem Feld. Tests
`tests/pruef/werkzeuge/werkzeug_notenblatt.dhtest` (7, seriell; seit Stufe 33 ohne
Python -- das Tracker-Gitter ist dort ein fester, von `to_tracker_song`
aufgezeichneter Stand). Schluesselwechsel rueckt die
Noten ohne Dialog um Oktaven heran und sagt es (Strg+Z nimmt es zurueck).

## Form-Designer entfernt (2026-11-10)

Wunsch des Nutzers: der Form-Designer (`examples/197_form_designer.dh`, seit
2026-09-06 der erste Editor-Pilot von Weg B) wird nicht mehr gebraucht -- das
Drachenhauch Studio ersetzt ihn (es hat sein Fenstermodell uebernommen,
`studio/modell.dh`). Weg sind das Beispiel, `docs/form-designer.md` (de/en),
`werkzeug_formdesigner.dhtest` und seine zwei Faelle in
`werkzeug_kreuz.dhtest`, der Menueeintrag Werkzeuge -> Form-Designer samt
Befehl `wz_form` und Paletteneintrag, die Verknuepfung im Startmenue (der
Installer loescht die einer frueheren Installation, `[InstallDelete]`) und
der Eintrag in `examples/kategorien.json`. Eine `.dhform` im Projektbaum der
IDE geht jetzt im Studio auf (`studioStarten`, Status nennt die Datei statt
des Ordners); als Text weiter ueber die Befehlspalette. Die Handbuecher nennen
an seiner Stelle das Studio oder nur noch das Format. Tests: der Fall in
`werkzeug_ide_13_25.dhtest` erwartet `studio b_maske.dhform`, der Fall
"der form-designer findet das studio" in `werkzeug_studio5.dhtest` wurde zu
"eine dhform aus der ide geht im studio auf"; beide Faelle lenken
`DH_STUDIO_KONFIG` in den Fallordner. Der alte Pfad steht in
`GEDULDETE_PFADE` (nur noch in Entwuerfen genannt).
