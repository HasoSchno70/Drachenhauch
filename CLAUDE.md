# Drachenhauch

BASIC-Dialekt mit Pascal-strikter Typisierung und OOP, ausgelegt für Spiele.
**Eine Runtime: `dhrt`** (Rust/raylib) — sie ist Lexer → Parser → Compiler → VM
in einem und übernimmt Ausführung, Konsole, Grafik/Audio, Standalone-Export,
Sprachserver (`dhrt lsp`), Prüfsammlungen (`dhrt test`) und Doku-Werkzeuge.
Die IDE (`ide/ide.dh`) und alle Werkzeuge (`examples/183..199_*.dh`) sind
Drachenhauch-Programme. **Python gibt es nur noch in zwei Bauskripten**
(`rust/build_runtime.py`, `rust/build_wasm.py`, nur Standardbibliothek).

> ## ⚠️ STUFE C — der Python-Teil ist GELÖSCHT (2026-09-21)
> Mit `docs/entwurf-python-abbau.md` 7.7 Punkt 7 sind weg: das Paket
> `drachenhauch/` (Qt-IDE, Qt-Werkzeuge, Python-Lexer/-Preprocess, `synth.py`,
> `graphics.py`, modules/__init__.py), `dhrun.py`, die `dh*.cmd`/`dh.sh`-
> Starter, `pyproject.toml`, `requirements.txt`, `tests/*.py` samt
> `conftest.py`, tools/qt_tests_einzeln.py, der Qt-Installer
> (`build_installer.py`, `Drachenhauch.iss/.spec`, `gen_notices.py`) und die
> 14 Qt-`.dhsprite`-Dateien. Logo und Schriftzug liegen jetzt in `daten/bilder/`.
>
> **Folgen für die Arbeit:** Geprüft wird NUR über `dhrt test tests/pruef`
> (`.dhtest`-Sammlungen, Format in `docs/werkzeuge.md`) und Rust-`#[test]`s;
> die CI fährt kein pytest mehr. Neue Funktionen kommen in `rust/`, in die IDE
> oder in die Werkzeuge. Die Fassung steht nur noch in
> `rust/drachenhauch_runtime/Cargo.toml`.
>
> **WICHTIG:** Die vielen Abschnitte weiter unten, die drachenhauch/...py,
> `editor_qt`, pytest, `conftest.py`, `run_gb`, den Qt-Designer oder
> „`tests/test_*.py`" nennen, sind **Geschichte** -- die Dateien gibt es nicht
> mehr. Wo ein Abschnitt eine pytest-Datei als Test nennt, ist der Fall
> umgezogen (Weg D) oder mit Python entfallen.

> ## ⚠️ STUFE B — Tree-Walker + Python-Toolchain ENTFERNT (2026-06-06)
> Früher gab es zusätzlich einen Python-**Tree-Walker** (`interpreter.py`, Referenz
> + Built-in-Host), einen Python-**Compiler** (`compiler.py`/`bytecode.py`/
> `serialize.py` → `.dhc`) und zwei Bytecode-VMs (`vm.py`, `vm_native.pyx`). **Alle
> entfernt** — `dhrt` hat sie abgelöst und ist die EINZIGE Runtime. Ebenfalls weg:
> `builtins_registry.py`, alle `modules/*.py`-Implementierungen (dhrt
> reimplementiert die Module nativ in Rust), `export.py`, `environment.py`, pygame.
>
> **Folgen für die Arbeit:** Neue Builtins/Sprach-Features kommen NUR in
> `rust/drachenhauch_runtime/` (+ ein run_gb-Golden-Test). Es gibt KEIN „beide Pfade" /
> „drei Pfade" / „Parität gegen Tree-Walker" mehr — Korrektheit sichern
> run_gb-Golden-Tests (`assert run_gb(src) == expected`) + Rust-`#[test]`s.
>
> **WICHTIG:** Viele Feature-Abschnitte WEITER UNTEN erwähnen noch
> `interpreter.py` / `compiler.py` / `vm.py` / `vm_native.pyx` / „Tree-Walker" /
> „alle drei Pfade" / „in BEIDEN Pfaden umsetzen". Das sind **historische
> Implementierungsnotizen** — die Dateien existieren nicht mehr; gültig ist heute
> ausschließlich `dhrt` (`rust/drachenhauch_runtime/src/`: `lexer.rs`/`parser.rs`/
> `compiler.rs`/`vm.rs`/`builtins.rs`/Modul-`.rs`).

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
> bei 620 (Faktor 0,42 gegen den Rest), als GB-Code-Export,
> Eigenschaften, Objekt-Ebenen und mehrere Tilesets noch alle fehlten.
> Wer hochrechnet, muss beide Fragen stellen: wie viel von dem Qt-Code
> dupliziert etwas, das die Laufzeit schon kann -- und wie viel von dem
> Qt-Code hat die Drachenhauch-Fassung gar nicht erst?
>
> **Der Tilemap-Pilot schreibt seit 2026-09-02 auch GB-Code** -- einen
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
> GB-Code-Ausgabe) -> 0,34 (2474, mit der .dhanim-Ausgabe) -> 0,35 (2604,
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
> fremden Lesern fuer JSON, WAV und GB-Code; der GB-Code wird gestartet, nicht
> nur uebersetzt), `tests/pruef/audio_note_mix.dhtest`.
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
> nicht auseinanderlaufen. Tests `tests/pruef/gui_draw_window.dhtest` mit der
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
> **Die GB-Code-Ausgabe schliesst den Kreis** (2026-09-01): [GB-Code]
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

## Verzeichnisstruktur

```
rust/drachenhauch_runtime/   # >>> die Runtime dhrt (Rust/raylib)
  src/lexer.rs parser.rs compiler.rs vm.rs builtins.rs + <modul>.rs
  src/lsp.rs symbole.rs doku.rs pruef.rs pruefsammlung.rs   # Werkzeuge in dhrt
rust/build_runtime.py        # Bau der Runtime (Python 3, nur Standardbibliothek)
rust/build_wasm.py           # Bau fuer den Browser (dito)
daten/                       # builtin_index/docs/prosa.json (in dhrt eingebettet), bilder/
ide/ide.dh                   # die IDE, in Drachenhauch
examples/*.dh                # Demos; 183..199 sind die Werkzeuge (Sprite, Tracker, ...)
installer/                   # bauen.dh, paket.dh, Drachenhauch-IDE.iss, lizenzen.dh
tests/pruef/*.dhtest         # die Pruefsammlungen (dhrt test tests/pruef)
tools/*.dh, tools/*.js       # Buch-Bau, Showcase-Bilder, Buch-Exporte (Node)
docs/                        # Handbuch (die IDE liest es fuer F1)
vscode-drachenhauch/         # VS-Code-Erweiterung (startet dhrt lsp)
web/                         # Web-Playground (dhrt als WASM)
```

## Architektur-Pipeline

```
Quelle.dh -> preprocess (IMPORT) -> Lexer -> Parser -> AST -> Compiler -> Bytecode -> VM
             alles in dhrt: preprocess.rs, lexer.rs, parser.rs, compiler.rs, vm.rs
```

`dhrt run datei.dh` wechselt vor dem Lauf ins Verzeichnis der Quelle (relative
Asset- und IMPORT-Pfade) und hinterlegt den Ort des Aufrufers als
`DHRT_START_DIR`. `dhrt --export datei.dh [ziel]` hängt den Bytecode an eine
Kopie der eigenen Exe und kopiert `assets/`.

## Built-ins erweitern (in dhrt / Rust)

Builtins leben in `rust/drachenhauch_runtime/src/builtins.rs` (pure) bzw. `vm.rs`
(`try_graphics`/`try_*` — brauchen VM-/Fenster-State). Der Dispatch läuft über
`CALL_BUILTIN` (vm.rs) → der große Match in `builtins.rs`. Neuer Builtin:

1. In `builtins.rs` (oder dem passenden `try_*` in `vm.rs`) einen Match-Arm
   ergänzen: Arity + Typen selbst prüfen (Validierung gehört in den Wrapper, nicht
   ins Backend), Fehlermeldung im gewohnten Wortlaut (`"NAME: erwartet …"`).
2. Für den Editor: `daten/builtin_index.json` ergänzen (Name/kind/Signatur/
   Modul) — Vervollstaendigung, Hover und `--check` der Laufzeit ziehen daraus.
   Die Kurzbeschreibung fuer Hover/Tooltip schreibt man NICHT hier hin,
   sondern ins passende `docs/module-*.md` (Tabellenzeile
   ``| `NAME(args)` | was es tut |``); `dhrt doku prosa` sammelt
   sie nach `daten/builtin_prosa.json` ein, und
   `tests/pruef/doku_pruefungen.dhtest` haelt beides synchron. `builtin_docs.json`
   ist fuer ausfuehrlichere Texte da und gewinnt, wo es einen Eintrag hat.
   Beide Dateien bettet dhrt fuer `dhrt lsp` ein -> nach einer Aenderung neu
   bauen. **Alle drei liegen in `daten/`** (bis 2026-09-17 in
   `drachenhauch/editor_qt/`, siehe unten).
3. Einen `tests/`-Golden-Test schreiben (`assert run_gb('PRINT NAME(...)') == ...`).
   **Die Signatur in `builtin_index.json` muss stimmen** — der Compiler leitet
   daraus die erlaubte Argumentzahl ab und warnt bei Abweichung (`dhrt --check`).
   Formen, die er versteht: `NAME(a, b [, c])`, `NAME(a, b = "")` (Vorgabewert =
   optional), `NAME(a, b, ...)` (beliebig viele), `NAME(6..8 Argumente)`.
   `NAME(*args)` schaltet die Pruefung ab — nur nehmen, wenn es wirklich offen
   ist. Eine zu enge Signatur erzeugt Falsch-Alarme in fremdem Code.
4. Bei neuem Keyword: `lexer::KEYWORDS` ergaenzen und die VS-Code-Grammatik
   neu erzeugen (`dhrt doku grammatik`).

(Es gibt keine Python-Seite mehr, die mitgepflegt werden muesste.)

## Built-in-Module schreiben

### Modul `gui` (Retained-Mode-GUI)

Persistente Fenster/Widgets (externe Typen `GUI_WINDOW`/`GUI_WIDGET` via `register_type`). Aufbau einmalig, pro Frame `GUI_UPDATE()` + `GUI_DRAW()`; Events per Polling (`GUI_CLICKED`/`CHECKED`/`VALUE`/`TEXT`/`HOVERED`) **oder** FUNCREF-Callbacks (`GUI_ON_CLICK`/`GUI_ON_CHANGE`). Widgets: Button, Label, Checkbox, Slider, TextInput, Panel, **Separator** (`GUI_SEPARATOR` — Trennlinie), **GroupBox** (`GUI_GROUPBOX` — gerahmte Gruppe mit eingelassenem Titel), **Table** (`GUI_TABLE` — professionelle Tabelle: fixierte Kopfzeile, V/H-Scroll, persistente Zeilen-Selektion. **Zellmodell** (`Cell` in gui.rs): jede Zelle mit eigener Vorder-/Hintergrundfarbe, Ausrichtung und ART (`text`/`bild`/`haken`/`balken`/`knopf`) — `GUI_TABLE_SET_CELL/CELL_COLOR/CELL_ALIGN/CELL_KIND/CELL_IMAGE/CELL_VALUE`, dazu `GUI_TABLE_ROW_COLOR` (ganze Zeile) + `GUI_TABLE_COL_ALIGN`. Farben in drei Stufen Zelle>Zeile>Zebra, -1 = nicht gesetzt; Auswahl/Hover liegen HALBDURCHSICHTIG darüber, sonst deckte eine Zellfarbe sie zu. **Spaltenzahl frei** = breiteste Angabe (`TableState::n_cols`) — kürzere Zeilen sind leer, keine Reihenfolge-Pflicht mehr (vorher war jede Abweichung ein Fehler, und Zeilen-vor-Kopf umging die Prüfung und crashte das Zeichnen). **Sortieren** per Kopfklick (2. Klick dreht um, Pfeil zeigt es an) — ZAHLENWEISE wenn beide Zellen Zahlen sind, sonst Text; `GUI_TABLE_SORT/SORT_COL/SORT_DESC`. **Filterzeile** im Kopf (`GUI_TABLE_SET(t,"filterzeile",1)`) — Teiltext je Spalte, case-insensitiv, UND-verknüpft; `GUI_TABLE_FILTER/GET_FILTER`. **Spaltenbreiten ziehbar** (Kopfkante, ±4px Fangbereich). **WICHTIG — Datenzeile vs. Ansicht:** Sortieren/Filtern stellen die Daten NIE um, sie bauen nur `view: Vec<usize>`; alle Zeilenangaben nach außen sind DATENzeilen (eine gemerkte Nummer bleibt gültig), sichtbare Reihenfolge über `GUI_TABLE_VIEW_COUNT/VIEW_ROW`. Einstellungen über EINEN Setter `GUI_TABLE_SET(key$,wert)` wie bei `chart`. Layout aus einer Quelle `table_geom` für Hit-Test + Zeichnen. Doku `docs/module-gui.md`, Demos `examples/157_gui_tabelle.dh` + `examples/158_gui_tabelle_sqlite.dh` (an einer echten SQLite-DB: lesen/sortieren/filtern/UPDATE beim Bearbeiten/DELETE in einer Transaktion; **arbeitet auf einer Kopie via `VACUUM INTO`**, die Originaldatei wird nur gelesen -- Test `tests/pruef/beispiel_sqlite_tabelle.dhtest` sichert genau diese Zusage ab). Muster fuer DB-Anbindung: DB ist die Wahrheit, Tabelle die Ansicht; je Zeile die id merken (Sortieren/Filtern stellen Datenzeilen nicht um); beim Mehrfach-Loeschen ERST alle ids einsammeln, dann loeschen. **Zellen bearbeiten:** Doppelklick auf eine Textzelle einer per `GUI_TABLE_COL_EDIT` freigegebenen Spalte oeffnet ein Eingabefeld IN der Zelle (Enter uebernimmt, ESC nimmt zurueck, Klick woanders uebernimmt); `GUI_TABLE_EDITING_ROW/COL` fragen den Zustand ab. Arbeitskopie `edit_text` -- die Zelle wird ERST beim Bestaetigen geschrieben, sonst koennte ESC nichts zuruecknehmen und jede Taste wuerde Sortierung/Filter neu anwerfen (die Zeile spraenge beim Tippen weg). Doppelklick-Erkennung im Gui (`last_click`, zeitbasiert via `g.get_time()`, 0.4 s + max 4 px Versatz); `editing_table` merkt die Tabelle, damit ein Klick woanders in O(1) uebernimmt statt einen verwaisten Editor stehen zu lassen. **ESC-Falle:** benutzt das Programm ESC zum Beenden, muss es `GUI_TABLE_EDITING_ROW < 0` abfragen. **Feste Spalten** (`GUI_TABLE_SET(t,"feste_spalten",n)`): die ersten n scrollen waagerecht nicht mit. ALLES was Spalten verortet geht ueber EINE Quelle -- `col_x` (Lage) + `col_clip` (sichtbarer Bereich), von Treffertest UND Zeichnen benutzt; ein fester Block waere sonst der sicherste Weg, beide auseinander laufen zu lassen. **Mehrfachauswahl** (`mehrfachauswahl`-Schalter, aus per Vorgabe): Strg+Klick schaltet eine Zeile um, Umschalt+Klick waehlt den Bereich ab dem Anker -- in der SICHTBAREN Reihenfolge (ueber Datenzeilen traefe es bei sortierter Tabelle etwas anderes); `GUI_TABLE_SEL_COUNT/SEL_ROW/IS_SELECTED/SELECT/CLEAR_SELECTION`. `GUI_TABLE_SELECTED` bleibt die zuletzt angeklickte Zeile (Rueckwaertskompatibilitaet), beim Zeilen-Loeschen ruecken alle Auswahl-Indizes dahinter auf. **Spalten umsortieren** (`spalten_verschiebbar`, per Vorgabe AUS): Kopfzelle seitwaerts ziehen, Tausch sobald sie ueber die Mitte des Nachbarn kommt. Klick und Zug sind DIESELBE Geste -- erst beim Loslassen entscheidet sich, ob sortiert (keine Bewegung) oder verschoben wurde (>= 5 px); ohne das wuerde jedes Verschieben nebenbei sortieren. Wie bei den Zeilen werden die DATEN nicht umgestellt: `col_order` bildet nur Anzeige-Position -> Datenspalte ab, `TGeom.col_widths` steht in ANZEIGE-Reihenfolge, `pos_at` liefert die Position und `col_at` die Datenspalte. `TableState::order()` bereinigt+ergaenzt die Liste bei JEDEM Abruf, damit eine spaeter dazugekommene Spalte nicht unter den Tisch faellt. `GUI_TABLE_MOVE_COL/COL_AT/COL_POS/RESET_COLS`; Reihenfolge + feste Spalten werden im .dhform gespeichert. **Textauswahl in der Zelle:** der Zell-Editor benutzt DIESELBE Routine wie das TextInput-Widget (`Gui::einzeiler_tasten`, aus `edit_textinput` herausgeloest) -- Markieren per Umschalt+Navigation oder Maus-Ziehen, Strg+A/C/V/X, Pos1/Ende. Beim Oeffnen ist alles markiert (erstes Tippen ersetzt). **Gotcha:** der oeffnende Doppelklick darf NICHT als Marke-setzen zaehlen -- `edit_maus_sperre` sperrt die Maus, bis die Taste einmal los war; ohne das hob derselbe Klick die Markierung sofort wieder auf (Ruecktaste loeschte dann ein Zeichen statt des Inhalts). Klicks IM Eingabefeld sind sowohl in `handle_press` als auch in `table_press` ausgenommen, sonst schlossen sie den Editor. Das Feld-Rechteck wird aus `table_geom` GERECHNET (`edit_cell_rect`), nicht beim Zeichnen gemerkt -- `draw_cell` ist `&self`, und eine zweite nachgefuehrte Quelle liefe auseinander. **Nicht umgesetzt:** Zeilengruppen), **Radio** (`GUI_RADIO(win,group$,text,x,y)` — Gruppen mit gegenseitigem Ausschluss, `GUI_RADIO_SELECTED`), **Dropdown/ComboBox** (`GUI_DROPDOWN(win,x,y,w,h,items)` — aufklappendes Popup über allen Widgets, `GUI_DROPDOWN_SELECTED/TEXT/SET_SELECTED`, `GUI_SET_DROPDOWN`), **ProgressBar** (`GUI_PROGRESS`, Wert via `GUI_SET_VALUE` 0..1). **Laufzeit-Manipulation** (`GUI_SET_BOUNDS`/`GUI_GET_X/Y/W/H`, `GUI_DESTROY`, `GUI_SET_VISIBLE`/`GUI_VISIBLE`, `GUI_KIND`, `GUI_FOCUS`, `GUI_HIT_TEST`, Window-Pendants + `GUI_WINDOW_WIDGET_COUNT/WIDGET`-Enumeration; Tombstone-Handles bleiben stabil) + **Serialisierung** (`GUI_SAVE/LOAD` Datei, `GUI_TO_JSON/FROM_JSON` String) — Basis für dynamische UIs + WYSIWYG-Editor. **Farbe als Text** `COLOR_HEX$(farbe)` / `COLOR_FROM_HEX(text$)` (core): vorher konnte ein Programm Hex-Text GAR NICHT in eine Farbe wandeln -- `VAL("&HFF8800")` ist 0, und `&H`-Literale gibt es nur im Quelltext. Kurzform `#F80` wird verdoppelt; Deckkraft 0 (= deckend) laesst `COLOR_HEX$` weg.

**Farbwaehler + Datumswaehler** (2026-08-30, gefunden beim Partikel-Piloten -- `gui` hatte keinen Farbwaehler): `GUI_COLORPICKER`/`GUI_PICKED_COLOR`/`GUI_SET_PICKED_COLOR` und `GUI_DATEPICKER`/`GUI_DATE`/`GUI_SET_DATE`. Datumsformat `JJJJ-MM-TT` wie `DATE$()` -- zwei Formate waeren eine Stolperfalle; ein neuer Waehler zeigt HEUTE. Rechenkerne als eigene Module mit Rust-`#[test]`s, bewusst OHNE Widget: `farbraum.rs` (RGB<->HSV, Rundweg ueber 4096 Farben geprueft) und `kalender.rs` (Schaltjahre, Monatslaengen, Zellers Wochentag). **Der Farbwaehler speichert HSV, nicht RGB** -- bei Schwarz ist der Ton unbestimmt, bei Grau die Saettigung; wer zurueckrechnet, verliert sie und der Zeiger springt. Geometrie je Widget aus EINER Quelle (`cp_geom`/`dp_geom`), von Zeichnen und Treffertest gemeinsam benutzt; `cp_drag` merkt, ob die Geste im Feld oder im Streifen begann (sonst springt die Farbe beim Ziehen ueber die Grenze). **Falle beim Zeichnen:** Alpha 0 bedeutet DECKEND -- der Verlauf durchsichtig->schwarz beginnt bei 0x01000000, mit 0x000000 war das Feld schlicht schwarz. **Erweiterungen:** `GUI_COLORPICKER_SET(cp,"alpha",1)` blendet einen Deckkraft-Streifen ein (auf Schachbrett-Grund, sonst sieht man dem Verlauf nicht an wo er durchsichtig wird); `GUI_PICKED_COLOR` liefert dann 0xAARRGGBB statt 0xRRGGBB -- ohne den Schalter bleibt es bei sechs Stellen, damit bestehende Programme unveraendert bleiben. **Deckkraft geht 1..255, nicht 0** (oberstes Byte 0 = deckend). `GUI_DATE_RANGE(dp,von$,bis$)` (leer = keine Grenze; ein Datum ausserhalb wird sofort hereingezogen, gesperrte Tage nehmen weder Klick noch Taste an) und `GUI_DATEPICKER_SET(dp,"wochenbeginn",n)` (0=Montag..6=Sonntag -- eine feste Wahl waere fuer die halbe Welt falsch). Vier Blaetterbereiche im Kopf (Jahr/Monat je Richtung), Umschalt+Bild = Jahressprung. `cp_drag` merkt jetzt die ZONE (0 Feld / 1 Ton / 2 Deckkraft) statt eines Schalters.

Doku `docs/module-gui.md`, Demo `examples/186_farbe_und_datum.dh`, Tests `tests/pruef/gui_waehler.dhtest` + Tastatur in `tests/pruef/gui_tastatur.dhtest`.

**Grosse Dateien im Code-Feld:** `GUI_TEXTAREA_VIEW(ta)` -> TUPLE `(erste_zeile, zeilen, start_zeichen, laenge_zeichen)` -- damit faerbt ein Programm nur den SICHTBAREN Ausschnitt. **Gemessen je Tastendruck:** 3000 Zeilen 26 ms -> 0,2 ms; 10000 Zeilen 88 ms -> 0,8 ms; 30000 Zeilen 272 ms -> 2,1 ms. Der Engpass beim ganze-Datei-Weg ist NICHT das Zeichnen (konstant ~0,4 ms, nur sichtbare Zeilen) und nicht `SYNTAX_SPANS` (2,4 ms bei 3000 Zeilen), sondern die BASIC-Schleife Art->Farbe ueber 16500 Abschnitte (17,6 ms; mit einer MAP statt vier IF-Vergleichen 9 ms). Der Ausschnitt ist nicht nur schneller, sondern auch RICHTIG, weil Kommentare und Zeichenketten in Drachenhauch an der ZEILE enden -- bei Blockkommentaren waere das die Falle. Tests `tests/pruef/gui_textarea_view.dhtest`.

**Eigene Schrift gilt fuer die ganze gui:** `SETFONT(handle)` setzt die aktive Schrift, und jedes Widget ohne eigene folgt ihr (`wfont` = `font_wahl(w.font, g.active_font())`). **Falle:** `SETFONT` uebernimmt auch die GROESSE, mit der `LOADFONT(pfad, groesse)` den Zeichensatz gebaut hat -- nach einem SETFONT ist der Text also womoeglich groesser/kleiner als vorher. Doku `docs/module-gui.md` (Abschnitt Eigene Schrift).

**TEXTAREA als Code-Feld** (2026-08-30): `GUI_TEXTAREA_SPANS(ta, starts, laengen, farben)` faerbt Zeichen-Abschnitte ein, `SYNTAX_SPANS(quelltext$)` -> TUPLE `(starts, laengen, arten)` zerlegt Drachenhauch-Quelltext (Arten: kommentar/text/zahl/schluessel/name/operator), `GUI_TEXTAREA_SET(ta, key$, wert)` mit `zeilennummern`/`aktive_zeile`/`tab_fuegt_ein`/`tabbreite`. **Der Hervorheber (`syntax.rs`) ist bewusst KEIN Lexer** -- der lexer.rs wirft Kommentare weg, expandiert f-Strings und bricht bei Fehlern ab; ein Editor sieht halb getippten Text und muss ihn trotzdem darstellen (offene Zeichenkette endet an der ZEILE, sonst faerbt ein Anfuehrungszeichen den Rest der Datei). Die Wortliste teilen sich beide ueber `lexer::keyword`. **Zwei Fallen:** (1) Farbige Laeufe werden ueber die Breite des VORSPANNS positioniert, nicht durch Addieren der Laufbreiten -- eine Textbreite enthaelt den Abstand ZWISCHEN Zeichen, aber keinen dahinter, also klebten die Woerter an jeder Farbgrenze zusammen; ausserdem rechnen Schreibmarke und Auswahl schon so. (2) Die Nummernspalte verschiebt ALLES, was eine Spalte verortet -> eine Quelle `ta_gutter()`, die Zeichnen, Treffertest und Schreibmarke gemeinsam fragen; der Text wird auf den Bereich rechts davon geclippt, sonst laeuft eine waagerecht gescrollte Zeile in die Zahlen. `scroll` ist beim TextArea die erste sichtbare ZEILE, `scroll_x` der Versatz in PIXELN. `GUI_SET_TEXT` loescht die Abschnitte (sie gehoerten zum alten Text). `tab_fuegt_ein` per Vorgabe AUS -- sonst kaeme man im Formular nicht mehr aus dem Feld heraus; an, rueckt TAB bis zur naechsten SPALTE ein. Doku `docs/module-gui.md`, Demo `examples/184_codefeld.dh`, Tests `tests/pruef/syntax_spans.dhtest` + 12 Rust-`#[test]`s in `syntax.rs` + Tabulator in `tests/pruef/gui_tastatur.dhtest`.

**Dialog IM Fenster** `GUI_DIALOG(titel$, text$[, stil$])` -> GUI_WINDOW, `GUI_ANSWER(dlg)` (0 offen / 1 OK-Ja / 2 Abbrechen-Nein), `GUI_MODAL()`. **NICHT verwechseln mit `GUI_MESSAGE`/`GUI_CONFIRM`** -- das sind die schon vorhandenen NATIVEN, BLOCKIERENDEN OS-Kaesten (rfd, `filedialog.rs`, Feature `dialogs`, im Web-Bau nicht da). `GUI_DIALOG` ist der Kasten im eigenen Thema/Massstab, blockiert NICHT (gui ist ein Polling-Toolkit; ein blockierender Dialog muesste mitten im Bild des Aufrufers eine eigene Zeichenschleife drehen und dessen Layer-/Render-Ziel-Zustand uebernehmen). Die Antwort gilt GENAU EIN BILD, wie `GUI_CLICKED` -- eine Antwort ist ein Ereignis, kein Zustand; danach ist das Fenster zerstoert (Tombstone-Handle bleibt gueltig). Modalitaet wird in `handle_press` UND `menu_input` durchgesetzt (`modal: Option<usize>`), sonst waere es nur ein Fenster obenauf; dazu ein Schleier in `draw` -- ohne sichtbares Zeichen klickt man in den Hintergrund und wundert sich. Stil-Woerter absichtlich dieselben wie bei `GUI_CONFIRM` (`ok`/`janein`). **Falle:** Widget-Koordinaten sind relativ zum INHALTsbereich, die Fensterhoehe nicht -- ohne den `title_h`-Zuschlag rutscht die Knopfreihe unter den Rand. Doku `docs/module-gui.md`, Tests `tests/pruef/gui_dialog.dhtest` (die Modalitaet mit Gegenprobe: derselbe Klick trifft ohne Dialog). **Ein Fenster ueber einer Zeichenflaeche braucht `GUI_DRAW_WINDOW(win)`** -- `GUI_DRAW` zeichnet Fenster und Zeichenflaechen in EINEM Durchgang, und der Inhalt einer Zeichenflaeche entsteht per Bauart DANACH (das Programm malt selbst hinein). Ohne den zusaetzlichen Aufruf ist ein Dialog mitten auf der Flaeche unsichtbar: er sperrt die Eingabe und ist nicht zu sehen, das Programm wirkt eingefroren. Ein zweites `GUI_DRAW` hilft NICHT (es zeichnet Fenster-Hintergrund und Zeichenflaechen neu). Ein unsichtbares oder zerstoertes Fenster zeichnet nichts, ist aber kein Fehler -- der Aufruf darf unbedingt in der Bildschleife stehen. Tests `tests/pruef/gui_draw_window.dhtest` (bis 2026-09-16 pytest mit Pillow; seither `--- bild`, die Faelle mit und ohne Aufruf stehen als Paare). **Kontextmenue und Tooltip liegen ueber ALLEN Fenstern** und haben dasselbe Problem -- der Tooltip folgt der Maus und landet also staendig ueber einer Zeichenflaeche: dafuer `GUI_DRAW_TOP()`, und `GUI_DRAW(FALSE)` laesst die Schicht beim Hauptdurchgang weg. Zweimal zeichnen waere hier KEIN Ersatz: Tooltip und Kontextmenue haben einen halbdurchsichtigen Schlagschatten, der uebereinander dunkler wird -- und zwar nur dort, wo das eigene Zeichnen die erste Fassung nicht zugedeckt hat, also genau an einer Kante. Bei Fenstern (`GUI_DRAW_WINDOW`) bleibt genau das ein bekannter Rest: ein Fenster, das nur teilweise unter dem selbst Gezeichneten liegt, bekommt seinen Glanz dort zweimal.

**Anzeige-Massstab** `GUI_SCALE(faktor)` (0.5..4.0) + `GUI_SCALE_GET()`: alle gui-Masse sind feste Pixel, auf einem 4K-Schirm mit 200 % wurde eine Oberflaeche darum halb so gross wie gedacht. Der Faktor multipliziert JEDE Laenge, die HINEINgeht (Fenster-/Widget-Geometrie, Metriken, die 15 Layout-Konstanten via `sk()`, Spaltenbreiten, Zeilenhoehen, Schriftgroesse); nach aussen bleibt alles LOGISCH (`unsk()` in den Gettern und in `widget_json`/`to_json`) -- sonst wuechse eine `.dhform` bei jedem Speichern um den Faktor weiter. EINZIGE Ausnahme: `GUI_HIT_TEST` spricht Bildschirm-Pixel (die Maus liefert nichts anderes). **Muss vor dem ersten Fenster kommen** -- danach Fehler, weil Bestehendes nur naeherungsweise umzurechnen waere. **Zwei Fallen, die erst das gerenderte Bild zeigte:** (1) `g.text`/`g.text_width` in der Fenster-Chrome (Titel/Menue/Reiter/Popup/Tooltip) kannten den Massstab nicht -> `ctext`/`ctext_width`/`ctext_height`, und die Editier-Helfer massen unskaliert, waehrend `wtext` skaliert zeichnete (Schreibmarke und Auswahl sassen auf halber Strecke) -> Buendel `Mass{size,font}` durch die statischen Helfer. (2) Die senkrechte Zentrierung stand als Literal `(h - 14) / 2` im Code -- bei Massstab 2 sass der Text zu tief und wurde vom Clip-Rechteck abgeschnitten -> `self.wsize(g,wdg)`. `GUI_LOAD`/`GUI_FROM_JSON` umgehen `add_widget` und brauchen den Massstab eigens. Doku `docs/module-gui.md` (Abschnitt Massstab), Tests `tests/pruef/gui_massstab.dhtest`.

**`GUI_CLICKED` gilt auch fuer Kaestchen, Kippschalter und Radioknoepfe** (seit
2026-09-01). Vorher war es dort STUMM -- immer FALSE, ohne Fehler; zwei von zwei
Programmen, die es versucht haben (die Piloten 187 und 189), hatten damit einen
toten Schalter. Ein Knopf setzt sein Flag beim LOSLASSEN, ein Kaestchen kippt
schon beim DRUECKEN -- fuer den Abfragenden ist beides "in diesem Bild
angeklickt", und beides meldet genau ein Bild lang. Der Zustand kommt weiter aus
`GUI_CHECKED`. Tests `tests/pruef/gui_clicked_schalter.dhtest` (mit Gegenprobe:
danebengeklickt meldet nichts).

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
Ueberfahren sonst verloren. Tests `tests/pruef/gui_liste_bedienung.dhtest`.
Tests `tests/pruef/gui_uebergaenge.dhtest` (15,
Bildproben; Gegenprobe mit `uebergang` 0 bzw. dem Bau davor) und in
`tests/pruef/gui_akkordeon.dhtest` "das aufklappen waechst ueber die zeit"
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
IDE legt F2 auf die Lesezeichen). Tests `tests/pruef/gui_liste_bedienung.dhtest`
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
Faelle. Tests `tests/pruef/gui_tabelle_bedienung.dhtest` (13; gegen den Bau
davor fallen die 10, die Neues pruefen). Im **Form-Designer** (197): Kaestchen "Umbenennen mit
F2" bei der Liste (`list.bearbeitbar`, abgewaehlt wird der Schluessel
ENTFERNT) und Feld "Leer-Hinweis" bei der Tabelle (`table.leer_text`), beides
im GB-Code (`GUI_LISTBOX_SET ... "bearbeitbar"`, `GUI_TABLE_PLACEHOLDER`);
Uebernehmen/Aktiviert/Hinweis ruecken dafuer eine Zeile tiefer. Tests in
`tests/pruef/werkzeug_formdesigner.dhtest` (Inspektor-Fall + der
GB-Code-Fall mit allen Arten; gegen den alten Designer fallen beide).

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
`tree.leer_text`). Tests `tests/pruef/gui_baum_bedienung.dhtest` (14; gegen den
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
`tests/pruef/gui_klappliste_bedienung.dhtest` (10; gegen den Bau davor fallen 8,
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
darueber. Tests `tests/pruef/gui_menue_bedienung.dhtest` (8; gegen den Bau
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
Textbereichs). Tests `tests/pruef/gui_codefeld_bedienung.dhtest` (9; die
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
Tests `tests/pruef/gui_textfeld_formate.dhtest` (12; Gegenprobe ohne
Klickzaehlung, ohne Rechtsklick, mit Tippstil, der nie verfaellt: genau die
5 betroffenen fallen), Rust-Tests `formate_tests`. Getter des Texts ist
`GUI_TEXT`, nicht `GUI_GET_TEXT`.

**Text und Formular** (2026-09-04, Punkt 1 des gui-Ausbaus -- Ziel: dass Drachenhauch genannt wird, wenn jemand fragt, womit er eine Anwendung schreiben soll): `GUI_SET_ALIGN(wdg, links|mitte|rechts)` fuer Beschriftung/Knopf/Textfeld; `GUI_SET_WRAP(label, breite)` bricht an Wortgrenzen um, die HOEHE folgt dem Text -- gemessen in `umbruch_layout` (in GUI_UPDATE, weil nur dort Graphics und Schreibzugriff zusammenkommen; das Zeichnen ist `&self`); `GUI_TEXTINPUT_SET(tf, key$, wert)` mit `passwort` (Punkte statt Zeichen -- Treffertest und Rollen messen an den PUNKTEN, sonst sitzt die Schreibmarke neben dem Text), `nur_lesen`, `maxlaenge` (schneidet ab, auch beim Einfuegen), `zahlen` (1 ganz, 2 Komma; Zwischenstand `-` erlaubt, sonst liesse sich keine negative Zahl tippen); `GUI_ENTERED`/`GUI_ON_ENTER`; **Strg+Z/Y in Textfeld und Textbereich** (Anschlaege innerhalb 0,8 s = EIN Schritt; `GUI_SET_TEXT` leert den Verlauf); `GUI_WINDOW_DEFAULT`/`GUI_WINDOW_CANCEL` (Enter/ESC druecken den Knopf -- aber die Taste gehoert zuerst dem Widget mit Fokus: Knopf/Kaestchen nehmen Enter selbst, Textbereich macht einen Umbruch, Zelle in Bearbeitung ihr Ende; aus einem TEXTFELD heraus ist Enter das Abschicken; der Standard-Knopf traegt den Akzent als Rahmen). Alles in der `.dhform` (auch der Tooltip -- der fehlte dort bisher) und im Form-Designer (Inspector + Codegen). **Testfalle:** raylibs Wiedergabe legt Tasten in die Tastenwarteschlange, aber KEINE Zeichen in die Zeichenwarteschlange -- Tests tippen ueber die Zwischenablage mit Strg+V, das laeuft durch dieselben Filter. Tests `tests/pruef/gui_text_formular.dhtest`, Doku `docs/module-gui.md`.

**Menues** (2026-09-04, Punkt 2 des gui-Ausbaus): `GUI_MENU_ITEM(menu, label$[, kuerzel$])` / `GUI_MENU_SHORTCUT` -- Kuerzel werden als Text geschrieben (`Strg+S`, `Alt+Enter`, `F5`, `Entf`, deutsch oder englisch; `kuerzel_parsen` in gui.rs mit Rust-Tests, unbekannte Taste = Fehler beim Anlegen statt eines still stummen Kuerzels) und jedes Bild geprueft (`kuerzel_pruefen`), auch bei geschlossenem Menue -- **seit 2026-09-07 in ALLEN sichtbaren Fenstern**: zuerst im Fokus-Fenster, dann in den uebrigen von oben nach unten (Form-Designer, Anim-FSM und Notenblatt mussten sich vorher nach jedem Knopf im Nebenfenster den Fokus zurueckholen, sonst war Strg+S stumm -- dreimal derselbe Fund, dreimal zuerst vom Test gesehen); das Fokus-Fenster gewinnt bei gleichem Kuerzel, ein modales laesst nur seine eigenen zu, ein Entwurfsfenster (`GUI_WINDOW_DESIGN`) zaehlt nicht; die Modifier muessen GENAU passen, und **ohne Strg/Alt gehoert die Taste dem Textfeld mit Fokus** (ein `Entf`-Kuerzel loescht dort ein Zeichen) -- AUSSER F1..F12 (`ist_funktionstaste`, seit 2026-09-06: die IDE in Drachenhauch startete per F5 nie, weil das Code-Feld den Fokus hatte). `GUI_SUBMENU` (beliebig tief; `sub_chain` = offene Untermenues, `untermenues_folgen` oeffnet beim Ueberfahren und schliesst NICHT, wenn die Maus neben allen Popups ist -- sonst klappt es beim schraegen Hinueberfahren zu; ein Untermenue ist nie Kontextmenue, `Menu::unter`), `GUI_MENU_CHECK`/`GUI_MENU_CHECKED` (Klick oder Kuerzel kippt), `GUI_MENU_ENABLE` (gesperrt = kein Klick, kein Kuerzel), `GUI_MENU_ICON`, `GUI_MENU_TEXT`. Popup-Layout aus EINER Quelle `popup_layout` (Treffertest + Zeichnen). In der `.dhform` verschachtelt (`items` am Eintrag, `shortcut`, `checkable`/`checked`); der Form-Designer bearbeitet Menues nicht, schreibt sie aber jetzt in den GB-Code (`_gb_menus`). **Testfalle:** raylib meldet beim Lesen mancher Aufnahmedateien "Issue reading line to buffer" auf stdout, die Ereignisse kommen trotzdem an -- Tests filtern `WARNING:`-Zeilen. Tests `tests/pruef/gui_menu_ausbau.dhtest`, Beispiel `examples/129_gui_menu.dh`.

**Listen** (2026-09-04, Punkt 3 des gui-Ausbaus): Eintraege einzeln (`GUI_LISTBOX_ADD/REMOVE/CLEAR/COUNT/ITEM/SET_ITEM/MOVE`, auch fuer Klapplisten -- vorher ging nur `GUI_SET_LISTBOX` als Ganzes; die Auswahl rueckt beim Einfuegen/Loeschen davor MIT, sie meint denselben Eintrag), `GUI_LISTBOX_SET(lb, "kaestchen"|"mehrfachauswahl", 1)`, `GUI_LISTBOX_ICON/COLOR` je Eintrag, `GUI_LISTBOX_CHECKED/SET_CHECKED`, Mehrfachauswahl mit denselben Abfragen wie die Tabelle (`IS_SELECTED/SELECT/SEL_COUNT/SEL_ROW/CLEAR_SELECTION`; Strg+Klick sammelt, Umschalt+Klick spannt vom Anker, ein Pfeil setzt die Menge auf eine Zeile), `GUI_DOUBLE_CLICKED` (bisher nur Listen). Zusatzzustand in `ListState` (`Widget::list`, nur bei Bedarf angelegt -- eine schlichte Liste bleibt, was sie war; `sync` haelt die Vektoren mit `items` gleich lang). **Ein Klick aufs Kaestchen kippt NUR den Haken**, die Auswahl bleibt -- sonst waehlte man beim Abhaken jedes Mal um. `handle_press` hat kein `g`, Strg/Umschalt kommen deshalb ueber `tasten_mod`, das `update` vor dem Druck fuellt. In der `.dhform` unter `list` (Sinnbilder nicht -- Textur-Handles). Designer: zwei Schalter im Inspector, Codegen. Tests `tests/pruef/gui_listen.dhtest` (Klicks mit Strg/Umschalt echt eingespeist; der Versatz kommt aus einer Zeichenflaeche bei (0,0)), Beispiel `examples/192_gui_listen.dh`.

**Dialoge** (2026-09-04, Punkt 4 des gui-Ausbaus): `GUI_DIALOG(t, x, "Speichern|Verwerfen|Abbrechen")` -- eigene Knoepfe (1..5), die Antwort ist die NUMMER des Knopfs; Enter = der erste, ESC/Schliessen = der letzte (Systemdialog-Konvention). `dialog_auswerten` zaehlt Knoepfe statt auf den Text "Nein" zu pruefen; Enter/ESC laufen ueber die Standard-/Abbrechen-Knoepfe aus Punkt 1 (`default_btn`/`cancel_btn`). `GUI_PROMPT(t, x[, vorgabe[, knoepfe]])` = derselbe Dialog mit Textfeld (Fokus im Feld, Vorgabe markiert), `GUI_DIALOG_TEXT` liest ihn auch nach der Antwort (Tombstone-Fenster behaelt seine Widgets). `GUI_WINDOW_MODAL(win, an)` schaltet ein EIGENES Fenster modal -- `dialog_auswerten` nimmt Nicht-Dialoge aus (sonst beendete jeder Knopf darin das Fenster) und gibt frei, wenn das Fenster ausgeblendet oder zerstoert wird. Gemeinsamer Kern `dialog_frei`. Tests `tests/pruef/gui_dialoge_ausbau.dhtest` (die Knopflage kommt aus einer 1x1-Zeichenflaeche IM Dialog), Beispiel `examples/193_gui_dialoge.dh`.

**Layout** (2026-09-04, Punkt 5 des gui-Ausbaus): `GUI_AUTOSIZE(wdg)` und `0` als Breite/Hoehe beim Anlegen = Mass nach Inhalt (`auto_w`/`auto_h`, gemessen im `layout_pass` in GUI_UPDATE; `GUI_SET_BOUNDS` hebt es auf). `GUI_LAYOUT(win, zeile|spalte|raster:N, x, y, w, h)` = neue Widget-Art `Kind::Layout` mit `LayoutState` (Kinder als Widget-Indizes, -1 = `GUI_LAYOUT_SPACER`), `GUI_LAYOUT_ADD(layout, wdg[, gewicht])` (Gewicht 0 = eigene Groesse, sonst Anteil am Rest; ein Widget in hoechstens EINEM Behaelter, kein Kreis), `GUI_LAYOUT_SET` (abstand, rand, ausrichtung, dehnen, rahmen). `layout_anwenden` laeuft je Bild von jedem WURZEL-Behaelter rekursiv; Beschriftungen darin werden immer exakt gemessen (die `GUI_LABEL`-Schaetzung `chars*8` stimmte fuer Zeilen nie). **Ein Behaelter ist Luft fuer Klicks** -- aus den drei Treffertests (Hover, Druck, `GUI_HIT_TEST`) ausgenommen, sonst schluckte er die Klicks seiner Kinder, die in der Reihenfolge NACH ihm kommen; nicht fokussierbar. Anker gelten fuer den Behaelter, die Kinder bekommen ihre Lage von ihm (relayout setzt sie, der naechste layout_pass korrigiert). In der `.dhform`: `layout` mit `kinder` als `[Index, Gewicht]`; der Form-Designer fuehrt die Kinder intern an NAMEN (`layout_von`/`layout_zuordnen`, Umrechnung nur an der Dateigrenze -- so ueberleben Zuordnungen Loeschen und Umsortieren ohne Index-Buchfuehrung), Paletteneintrag, Inspector (Art/Masse am Behaelter, "Layout"+"Gewicht" an jedem Control), Codegen. Bewusst NICHT: Constraints, Mindestgroessen. Tests `tests/pruef/gui_layout.dhtest` (headless ueber GUI_GET_X/Y/W/H), Beispiel `examples/194_gui_layout.dh`.

**Feinschliff** (2026-09-05, Punkt 6 und Abschluss des gui-Ausbaus): `GUI_VSLIDER(win, x, y, h, min, max, default)` (Slider mit `vert`, Wert waechst nach oben; `drag_slider` bekommt jetzt auch `my`), `GUI_PROGRESS_SET(p, "unbestimmt", 1)` (Band aus `g.get_time()`), `GUI_IMAGE_MODE` (strecken/einpassen/fuellen/mitte/kacheln, `draw_bild` mit Clip), `GUI_TREE_ICON/COLOR` (TreeNode `icon`/`color`; sobald ein Knoten ein Sinnbild hat, rueckt JEDE Zeile ein), **rollendes Panel** `GUI_PANEL_ADD/REMOVE/SCROLL/SCROLL_GET` (`PanelState` am Panel, `panel_von` am Kind -- je Bild in `panel_pass` gesetzt, durch Layouts hindurch; der Scroll ist NUR ein Blick-Versatz in `abs_rect`, die Kinder behalten ihre Fensterlage; `im_panel_sichtbar` nimmt Herausgerolltes aus Hover/Druck/GUI_HIT_TEST; ein rollendes Panel ist wie ein Layout Luft fuer Klicks, weil `handle_press` den ERSTEN Treffer nimmt und das Panel vor seinen Kindern liegt; Mausrad NACH den Kindern abgefragt, weil `pop_mouse_wheel` nur einmal liefert; kein Panel im Panel), **Ziehen** `GUI_DRAGGABLE/DROP_TARGET` + `GUI_DRAGGING/DROPPED/DROP_TEXT/DROP_SOURCE/DRAG_INDEX/DROP_INDEX` (`DragState` ab 5 px Bewegung, `DropInfo` transient wie `clicked`; Nutzlast = Listenzeile/Baumknoten/Widget-Text; innerhalb derselben Liste sortiert `ablegen` selbst um; Schatten + Ablage-Rahmen in `draw_top`), **Cursorformen** `cursor_pass` (setzt nur bei Wechsel und nimmt nur zurueck, was sie selbst war -- `MOUSE_CURSOR` nach GUI_UPDATE gewinnt; `GUI_CURSORS(FALSE)` aus; nicht testbar, raylib liest die Form nicht zurueck). `.dhform`: `vertical`, `indeterminate`, `mode`, `draggable`, `drop_target`, `panel{kinder,scroll}`; Designer: Inspector-Felder + "Panel"-Zuordnung ueber dieselbe Namens-Buchfuehrung wie Layout (`layout_von(c, "panel")`), Codegen. Tests `tests/pruef/gui_punkt6.dhtest` (Zuege echt eingespeist; Mausrad = Ereignis 8 mit params[1] = y), Beispiel `examples/195_gui_feinschliff.dh`.

**Mindestmasse und Formularpruefung** (2026-09-05, aus dem sechsten Piloten): `GUI_SET_MIN_SIZE(wdg, w, h)` -- ein gewichtetes Kind im Behaelter faellt nie darunter (die Zeile laeuft lieber ueber), ein verankertes Widget schrumpft nicht darunter; `GUI_LAYOUT_MIN_W/H(layout)` = Mass des Inhalts (`min_mass`, rekursiv; feste Kinder mit ihrer NATUERLICHEN Groesse `nat_w/nat_h` -- `w/h` tragen nach dem ersten Durchlauf die gedehnte Groesse, daraus liesse sich kein Minimum ableiten; gewichtete mit ihrem Mindestmass), der Wert fuer `WINDOW_MIN_SIZE`. **Formularpruefung:** `GUI_RULE(feld, art$[, a, b][, muster$][, meldung$])` (pflicht/zahl/ganz/bereich/laenge/email/datum/muster; an Klappliste und Kaestchen nur pflicht; die Argumentzahl haengt an der Art, der letzte Text ist die Meldung), `GUI_VALIDATE(win)` (Zahl der Fehler, prueft nur sichtbare + bedienbare Felder -- ein Fehler auf einem anderen Reiter waere einer ohne Ausweg; erstes falsches Feld bekommt den Fokus), `GUI_VALIDATE_WIDGET`, `GUI_ERROR/SET_ERROR/CLEAR_ERRORS`, `GUI_ERROR_LABEL` (Beschriftung zeigt die Meldung; `fehler_setzen` ist die EINE Stelle), `GUI_VALIDATE_LIVE` (beim Verlassen des Feldes, nicht je Anschlag), `GUI_RULES_CLEAR`. Roter Doppelrahmen am Feld, Meldung im Tooltip vor dem Hilfetext. **Leer laesst jede Regel ausser pflicht durch** -- sonst hiesse "bereich 1 10" zugleich "Pflichtfeld". Reine Regel-Logik `regel_pruefen` mit Rust-Tests; die regex-Fehlermeldung ist mehrzeilig, nur die letzte Zeile kommt in die Meldung. `.dhform`: `min_w/min_h`, `rules`, `error_label`; Designer: zwei Drehfelder + Zeile "Regeln" (`regeln_parsen`/`regeln_text`, Tippfehler werden uebergangen), Codegen. Bewusst NICHT: eigene Regel als FUNCREF (Programm prueft nach GUI_VALIDATE selbst und meldet ueber GUI_SET_ERROR). Der Pilot 196 benutzt beides statt seiner Handarbeit. Tests `tests/pruef/gui_pruefung.dhtest`, Mindestmasse in `tests/pruef/gui_layout.dhtest`.

**Zeilenumbruch und Datenbindung** (2026-09-05, die letzten zwei Punkte der Lueckenliste): `GUI_TEXTAREA_SET(ta, "umbruch", 1)` -- der Textbereich bricht an Wortgrenzen um (`ta_rows` = EINE Quelle sichtbarer Zeilen `(logische Zeile, von, bis)` fuer Zeichnen, Klick, Pfeile, Schreibmarke und Scroll; `ta_row_of`: am Umbruch gehoert die Marke zur NEUEN Zeile; Pos1/Ende/Pfeile laufen in sichtbaren Zeilen; Zeilennummern nur an der ersten Zeile eines Absatzes; `GUI_TEXTAREA_VIEW` zaehlt weiter logische Zeilen; `scroll_x` bleibt 0). Test-Falle: es gibt keine Schreibmarken-Abfrage -- ein eingefuegtes `#` (Strg+V) verraet die Lage, und nach GUI_SET_TEXT steht die Marke am ENDE, also erst sechsmal Pfeil hoch. **Datenbindung:** `GUI_BIND(wdg, schluessel$[, formular$])`, `GUI_FORM_GET/SET/CLEAR/CLEAN/CHANGED(win[, formular$])`, `GUI_FORM_LOAD/SAVE(win, db, tabelle$, id[, formular$])` (in vm.rs, weil nur die VM db UND gui hat; `#[cfg(feature = "db")]`; Spalte heisst `id`; Tabellen- und Schluesselnamen nur `[A-Za-z0-9_]`). `wert_text` (Vergleich/MAP) und `wert_typisiert` (Datenbank: Zahlenfeld -> Zahl, ausser fuehrende Null wie eine PLZ -> Text; Kaestchen -> BOOL; Klappliste -> Index; Regler -> FLOAT), `wert_setzen` tolerant ("ja"/1/TRUE, Eintragstext oder Index). `Window.form_stand` = sauberer Stand (beim Binden, nach SET/LOAD/SAVE/CLEAR/CLEAN). Der Pilot 196 laedt/speichert Kunden ueber die Bindung (keine Spaltenliste mehr) und nutzt sie beim Artikel nur fuer "geaendert?", weil er Preis und MwSt umrechnet. `.dhform`: `wrap_text`, `bind`, `form`; Designer: Felder "Bindung"/"Formular", Kaestchen "Zeilenumbruch", Codegen. Bewusst NICHT: Listen an Tabellen binden, Fremdschluessel, Verbunde. Tests `tests/pruef/gui_bindung.dhtest` (bis 2026-09-16 pytest mit sqlite3; seither liest ein zweiter Prozess die Datei mit `typeof()`), `tests/pruef/gui_umbruch.dhtest` (seriell).

**Drucken und OPENDOC** (2026-09-05, Wege A und C aus `docs/entwurf-drucken.md`): das pdf-Modul zeichnet seine Befehle jetzt AUF (`pdf::Op` je Seite, parallel zum Inhaltsstrom) und spielt sie auf drei Ziele: `PDF_SAVE` (wie immer), `PDF_PRINT(p[, drucker$[, kopien[, zieldatei$]]])` und `PDF_PREVIEW(p, seite[, breite_px]) -> IMAGE` (graphics.rs, raylibs Standardschrift). `drucken.rs` (ungated): Windows = GDI ueber das `windows`-Crate (schon im Baum, jetzt direkte Abhaengigkeit unter `[target.'cfg(windows)']` mit Gdi/Printing/Xps/Shell-Features) -- `CreateDCW` auf den Drucker, `StartDocW` mit `lpszOutput` = zieldatei, mm -> Geraeteeinheiten ueber `GetDeviceCaps`, `PHYSICALOFFSET` abgezogen, `TA_TOP` weil unser y die Oberkante ist, Schriften Helvetica->Arial/Times->Times New Roman/Courier->Courier New; `EnumPrintersW` Level 4, `GetDefaultPrinterW`; macOS/Linux = `lp -d -n` mit temporaerer PDF, `lpstat -a/-d`, zieldatei = die PDF selbst. `OPENDOC(pfad$)` = ShellExecute/open/xdg-open mit Endungsliste (kein Programmstarter). **Gemessen:** zwei Seiten durch "Microsoft Print to PDF" in eine Datei ~0,9 s; im Ergebnis stehen Text und der rechtsbuendige Betrag an seiner Stelle (Test `tests/pruef/drucken.dhtest`, laeuft auch auf dem Windows-Runner der CI, weil Windows den Drucker mitbringt; gelesen mit dem eigenen Leser `tests/pruef/_hilfen/pdftext.dh`). Kopien = der Auftrag n-mal, nicht DEVMODE. Bewusst NICHT: fremde PDFs drucken, nativer Druckdialog, Duplex/Papierfach. Pilot 196: Druckdialog aus Bordmitteln (Klappliste PRINTERS, Kopien, PDF_PREVIEW auf GUI_IMAGE) + "PDF oeffnen".

**Das pdf-Modul schreibt ueber krilla** (2026-09-14, auf Frage des Nutzers nach Rust-Bibliotheken wie typst): nicht typst selbst -- ein ganzes Satzsystem mit eigener Markup-Sprache passte nicht zur Befehlsfolge PDF_TEXT/PDF_LINE --, sondern **krilla**, die Bibliothek, mit der typst seine PDFs schreibt. Vorher standen in `pdf.rs` 440 Zeilen Handarbeit, die nur die vierzehn Standardschriften ohne Einbetten und nur cp1252 konnten, dazu `pdf_masse.rs` aus einem PyMuPDF-Skript. Jetzt: die Seiten zeichnen weiter nur `Op`s auf, `bauen()` spielt sie auf ein `krilla::Document` (von oben gezaehlt wie krilla selbst, Text-Y + Schriftgroesse = Grundlinie). **Schriften eingebettet und beschnitten**: eingebaut DejaVu Sans/Serif/Sans Mono in je vier Schnitten (Crate `dejavu`, 5,1 MB Schriftdaten im Bau) als `sans`/`serif`/`mono[-fett|-kursiv|-fett-kursiv]`, `helvetica`/`times`/`courier` bleiben Namen dafuer; `symbol`/`zapfdingbats` sind ein Fehler mit Hinweis. Neu `PDF_FONT_LOAD(p, pfad$, name$)`. **Text ist Unicode**; ein Zeichen, das die Schrift nicht hat, ist ein Fehler (`rustybuzz::Face::glyph_index`), kein leeres Kaestchen. **`PDF_TEXT_WIDTH` formt mit rustybuzz genau wie krillas `draw_text`** (Unterschneidung eingeschlossen) -- gegen PyMuPDF auf denselben Dateien nachgemessen: Serif/Mono gleich auf vier Stellen, `sans` schmaler um die Unterschneidungspaare. **Weiter reproduzierbar**: krilla schreibt kein Datum und bildet die Dokumentkennung aus dem Inhalt (Rust-Test + `--- nochmal`-Fall). **Folge fuer Tests**: im Inhaltsstrom stehen Glyphennummern statt Text; zwei Werkzeug-Sammlungen und `pdf.dhtest` lesen deshalb ueber `pdftext.dh` (Beilage, in Drachenhauch: Objekte, Seitenbaum, Ressourcen, ToUnicode-Tabellen, TJ-Zeichenketten) statt BUFFER_INFLATE + INSTR; Gegenprobe mit falschem Suchwort faellt. GDI-Druck bildet sans/serif/mono auf Arial/Times New Roman/Courier New ab, geladene Schriften auf Arial. Doku `docs/module-pdf.md`, Buch `79_dokumente.js`. **Stolpersteine beim Leser:** `VAL("3 0 R")` ist 0 (fuehrende Zahl selbst lesen), und eine MAP laesst sich nicht verschachtelt abfragen (`tabellen[akt]` ist ein Fehler) -- ein flacher Schluessel `schrift:glyph`.

**Fenster als Prozess** (2026-09-05, Weg B aus `docs/entwurf-native-fenster.md`): `WINDOW_OPEN(datei$[, args...])` startet einen zweiten `dhrt run` mit eigenem SCREEN (`fenster.rs`, ungated wie `hintergrund.rs`); Kanal = TCP auf 127.0.0.1, Port ueber `DHRT_ELTERN_PORT`, NICHT stdin/stdout (das Kind darf weiter PRINT); `WINDOW_SEND/RECV$/ALIVE/CLOSE`, im Kind `PARENT_SEND/RECV$/ALIVE` (Verbindung beim ersten PARENT_*-Aufruf; `PARENT_ALIVE` = FALSE ohne Eltern, damit dasselbe Programm allein laeuft). `Leitung` teilen sich beide Seiten: begrenzte Warteschlange (1024, aelteste faellt weg, wie MIDI), vor der Verbindung Gesendetes wird vorgemerkt, eine Nachricht ist EINE Zeile (Umbruch = Fehler); faellt die Verbindung, beendet sich das Kind (`std::process::exit`); `Drop` der Eltern killt die Kinder. WINDOW_OPEN nimmt dem Kind `DHRT_FRAMES`/`DHRT_SCREENSHOT`/`DHRT_CONTACT*` aus der Umgebung -- sonst stuerbe es in Tests nach N Bildern. Gemessen: 0,4 s bis zum ersten Bild, 1000 Nachrichten im Schub ~60 ms, EINE Runde 17-50 ms = ein bis drei Bilder des Kindes (es liest einmal je Bild; der Entwurf verlangte < 5 ms fuer den Kanal, und der Kanal schafft das -- die Bildschleife nicht). Kein geteilter Zustand, wie TASK_START. Pilot: `196_rechnung_fenster.dh` (64 Zeilen) + Strg+F in 196. Tests `tests/pruef/fenster_prozess.dhtest` (seriell, zwei Prozesse) + Rust-Tests in fenster.rs.

**Barrierefreiheit** (2026-09-06, Wege A und C aus `docs/entwurf-barrierefreiheit.md`): gemessen vor dem Bau hatte ein dhrt-Fenster im UIA-Baum NULL Nachkommen -- fuer Bildschirmleser, Lupe und Sprachsteuerung ein Titel und sonst nichts. Jetzt schiebt das gui-Modul je Bild einen vollstaendigen Baum an **AccessKit** (`a11y.rs`; Crates `accesskit` 0.25 + `accesskit_windows` 0.35, im `graphics`-Feature): `Gui::a11y_baum` (Fenster als Group/Dialog, Widgets mit Rolle nach `Kind`, Menueleiste, Reiter, Listeneintraege, Tabellenzeilen/-zellen, Baumknoten; Knoten-Nummern in `a11y::ids`, eine Quelle fuer Bauen und Zerlegen), `Gui::a11y_aktion` (Focus/Click/SetValue/Increment/Expand auf DENSELBEN Wegen wie Maus und Tastatur, damit `GUI_CLICKED`/`on_click`/`on_change` feuern). **Drei Fallen:** (1) der Windows-Adapter verlangt ein Fenster, das noch nie sichtbar war -- `graphics.rs` legt es deshalb IMMER versteckt an (`builder.hidden()`), haengt den Adapter ein und zeigt es dann; (2) die Aktivierungs-Anfrage kommt in der Fensterprozedur (raylibs EndDrawing), wo die gui nicht greifbar ist -- der Handler antwortet `None` und merkt sich nur, dass gefragt wurde; laut Vertrag muss dann das naechste `update_if_active` einen VOLLEN Baum liefern (GUI_UPDATE tut das, FLIP schickt ohne gui einen Baum nur mit dem Fenster); (3) **LNK2005 `CloseWindow`/`ShowCursor`**: raylibs C-Funktionen heissen wie user32-Exporte, und die Import-Bibliothek des `windows`-Crates stand im Link vor raylib -- mit ZWEI windows-Fassungen (0.61 fuers Drucken, 0.62 aus AccessKit) brach der Link; dhrt ist deshalb auf `windows` 0.62 gehoben, eine Fassung, und es linkt. Beschriftung eines Eingabefelds = die Beschriftung links daneben oder darueber (`a11y_beschriftung`, `labelled_by`), sonst der Tooltip. Baum nur, wenn ein Hilfsprogramm fragt (`a11y_aktiv`). Neu: `GUI_ANNOUNCE(text$[, dringend])` (Live-Knoten; wechselndes Leerzeichen, damit dieselbe Ansage zweimal eine Aenderung ist), `GUI_SCREENREADER()`, `GUI_SET_TAB_INDEX(wdg, n)` (Index > 0 zuerst, Rest in Anlege-Reihenfolge; `.dhform` `tab_index`, Designer-Drehfeld). Weg A dazu: **Menueleiste per F10 oder Alt allein** (`menue_tasten`, `menu_cursor` im tiefsten offenen Menue, Pfeile/Enter/ESC; solange ein Leistenmenue offen ist, gehoeren die Tasten dem Menue; Alt+X loescht den Alt-Merker ueber `key_any_pressed_except_alt`), **Tooltip bei Tastaturfokus** (`tip_fokus`, unter dem Widget, die Maus gewinnt), helles Thema: gedaempfter Text 2,76:1 -> 6,3:1, Akzent 3,8 -> 5,6:1. **Pruefstein:** `tests/pruef/gui_barrierefreiheit.dhtest` (bis 2026-09-17 pytest) laesst einen FREMDEN Leser (UIA aus PowerShell, `System.Windows.Automation`, keine neue Abhaengigkeit) Namen zaehlen, den Knopf klicken, Text setzen, Fokus setzen -- laeuft auch auf dem Windows-Runner. Bewusst NICHT (noch): Schreibmarke zeichenweise, `ui`-Modul, Web. SPEAK (Weg B) kam am selben Tag, siehe unten. Pilot 196 sagt seine Statuszeile an. **macOS/Linux-Adapter (selber Tag):** `a11y.rs` hat je System einen `mod plattform` mit EINEM Vertrag (`neu/aktiv/aktionen/senden/fenster_lage/fenster_fokus`) -- macOS `accesskit_macos::SubclassingAdapter::for_window` auf GLFWs `NSWindow` nach `add_focus_forwarder_to_window_class("GLFWWindow")`, Fokus ueber `update_view_focus_state`; Linux `accesskit_unix::Adapter::new` (Handler muessen `Send` sein, D-Bus-Faden; `DeactivationHandler` setzt `aktiv` zurueck), Lage/Fokus meldet `a11y_bild_ende` je FLIP bei Aenderung (`set_root_window_bounds`, aussen = innen, unter Wayland 0/0). Damit sich das OHNE raylib fuer ein fremdes Ziel pruefen laesst, ist `a11y` ein eigenes Feature (graphics schaltet es ein; `mod a11y` mit `#[allow(dead_code)]`): `cargo check --features a11y --target aarch64-apple-darwin` bzw. `x86_64-unknown-linux-gnu` (rustup-Ziele nachinstalliert), und die CI macht denselben Check je Laeufer. Uebersetzt, nicht gelaufen -- kein Mac, kein Linux hier; die Doku sagt das. Nicht fuer emscripten (das ist auch `unix`): eigener cfg-Ausschluss.

**Sprachausgabe `SPEAK`** (2026-09-06, Weg C aus `docs/entwurf-speak.md`, `sprache.rs`): gemessen vor dem Bau -- Windows hat ZWEI Sprachausgaben: SAPI (`ISpVoice`, spricht am Mischer vorbei, nur die alten Desktopstimmen Hedda/Zira) und WinRT `Windows.Media.SpeechSynthesis` (Stefan, Katja, Hedda; liefert einen **WAV-Strom**, mono 16 kHz, 14 Woerter in 35 ms beim ersten und 12-13 ms bei jedem weiteren Mal). Der Unterschied entschied den Weg: **eine gesprochene Zeile ist ein `SOUND`** (`Audio::sound_aus_pcm` = derselbe Weg wie `AUDIO_NOTE`, samt Lo-Fi und 5-ms-Flanken), sie laeuft auf einem eigenen Bus `speech` (nur Lautstaerke, keine Effektkette; `AUDIO_BUS_VOLUME/GET_VOLUME` und `AUDIO_PUSH/POP` kennen ihn), und `SPEAK_SOUND` gibt sie heraus (raeumlich per `AUDIO_PLAY_ON`, mischen, `AUDIO_SAVE_WAV`). Befehle: `SPEAK(text$[, unterbrechen])`, `SPEAK_STOP`, `SPEAKING`, `SPEAK_WAIT`, `SPEAK_VOICE` (unbekannt = Fehler mit der Liste), `SPEAK_VOICES`, `SPEAK_RATE` (0.5..2.0), `SPEAK_SOUND`. **Die Warteschlange braucht kein Polling:** ein angehaengter Satz geht mit `StartTime::Delayed(rest)` an Kira (wie `AUDIO_PLAY_AT`), `Sprecher` merkt sich nur das geplante Ende, `SPEAKING()` vergleicht mit der Uhr -- ein Konsolenprogramm, das in `INPUT` haengt, hoert seine Saetze trotzdem nacheinander. Die Handles der gesprochenen Zeilen liegen in `Audio::speech_handles`, NICHT im Slot: `AUDIO_PLAY` auf demselben Slot stoppt die alte Instanz, ein zweites `SPEAK("Treffer")` darf das erste nicht abschneiden. Vorrat je (Text, Stimme, Tempo) mit 64 Plaetzen, der aelteste nicht laufende faellt weg; ein per `UNLOADSOUND` freigegebener Vorratsklang wird neu gerechnet (`sound_lebt`). **Bildschirmleser zuerst:** `SPEAK` fragt `g.a11y_aktiv()` -- laeuft ein Hilfsprogramm, geht der Satz als Ansage in den Baum (`Gui::announce` UND `Graphics::a11y_ansagen`; der Fenster-Baum, den FLIP ohne GUI_UPDATE schickt, traegt dafuer jetzt den Ansage-Knoten), `SPEAKING()` bleibt FALSE; `GUI_SCREENREADER()` fragt seither das Fenster direkt und stimmt auch ohne gui. **Zwei Fallen:** (1) `SynthesizeTextToStreamAsync` gibt es im `windows`-Crate nur mit `Media_Core` UND `Storage_Streams` (der Fehler sagt "Methode nicht gefunden", nicht "Feature fehlt"); (2) WinRT braucht einen angemeldeten Faden -- unter GLFW ist der Hauptfaden schon STA (RoInitialize meldet RPC_E_CHANGED_MODE, ignoriert), ein Konsolenprogramm ohne Fenster hat nichts, darum `RoInitialize(MULTITHREADED)` vor jedem Zugriff. `.join()` der IAsyncOperation wartet auf ein Win32-Ereignis, die Synthese liefert von einem Threadpool-Faden -- kein Deadlock im STA. macOS `say -o --file-format=WAVE --data-format=LEI16@22050` (Text ueber stdin, damit er mit `-` beginnen darf), Linux `espeak-ng --stdout --stdin -v <LANG>` -- beide ungeprueft, beide mit Fehler samt Installationshinweis, wenn das Werkzeug fehlt. Eigener WAV-Leser (`wav_lesen`, PCM 8/16/24/32 + Float, Kanaele gemittelt, Datenlaenge ueber das Ende = bis zum Ende, weil `--stdout` sie nicht nachtraegt) mit Rust-Tests. Pruefstein `tests/pruef/speak.dhtest` (WAV im `--- nachher` Byte fuer Byte gelesen: mono, Huellkurve; Vorrat; angehaengt dauert > 1,6x unterbrochen; unbekannte Stimme -- dieser Fall war in der pytest-Fassung bis 2026-09-16 IMMER uebersprungen, weil die Hilfsfunktion "nicht gefunden" als fehlende Sprachausgabe wertete) und in `tests/pruef/gui_barrierefreiheit.dhtest` der fremde UIA-Leser, der in einem Programm OHNE gui die Ansage als Text-Knoten findet. Bewusst NICHT: SSML, Wortgrenzen-Ereignisse, mitgelieferte Stimme (piper, 60 MB), SAPI.

**Fremde Schriften, Euro, Eingabemethoden** (2026-09-06, Wege A und B aus `docs/entwurf-eingabemethoden.md`): gemessen vor dem Bau war der Speicher Unicode, die ANZEIGE Latin-1 -- `zeichensatz()` in graphics.rs backte 246 Zeichen ohne `€`, ohne Latin Extended, Griechisch, Kyrillisch, CJK, Emoji; auch eine per LOADFONT geladene japanische Schrift bekam nur diese. Jetzt: (1) Grundvorrat ASCII..Kyrillisch + Interpunktion + `€` (~1000 Glyphen, Start +50 ms); (2) **Glyphen auf Zuruf** -- `glyphen_pruefen` merkt beim Aufzeichnen/Messen Codepunkte ohne Glyphe (`glyphen_fehlend`, RefCell weil Messen `&self` ist), `ausweich_nachladen` am FLIP-Anfang backt je Datei EINEN Eintrag (`Vec<Ausweich>`, Index 0 = Grundschrift, unveraendert) mit der Vereinigung aller bisher gebrauchten Zeichen (`ausweich_datei_fuer` waehlt die Datei je Block: msgothic.ttc/malgun.ttf/seguiemj.ttf/segoeui.ttf; macOS Arial Unicode; Linux Noto CJK), was keine Datei liefert -> `glyphen_unmoeglich`; Zeichnen UND Messen zerlegen einen Text in Laeufe je Font (`text_laeufe`, `zeichne_text`, `breite_mit` -- EINE Aufteilung, sonst misst ein Layout etwas anderes); (3) `LOADFONT(pfad$, groesse[, zeichen$])` mit Blocknamen (`BLOECKE`, deutsch/englisch) oder Zeichenkette (`zeichen_aus_namen`), Grundvorrat immer dabei; (4) **Schriftsammlungen .ttc**: raylib (stb ab Versatz 0) liest sie nicht und tauschte STILL gegen die Bitmapschrift -- `ttc_erste_schrift` loest die erste Schrift heraus (Tabellenverzeichnis kopieren, Versaetze neu), alles laeuft ueber `schrift_laden` + `LoadFontFromMemory`; eine Ersatzschrift ist jetzt ein Fehler (`keine_ersatzschrift`, Textur-ID der Standardschrift). Auf dieser Maschine gibt es KEIN CJK-.ttf, nur .ttc -- ohne (4) blieb CJK ein `?`. (5) Tipp-Warteschlange 16 -> 256 ueber `CFLAGS=-DMAX_CHAR_PRESSED_QUEUE=256` in build_runtime.py (cmake uebernimmt CFLAGS; nachgesehen im CMakeCache von raylib-sys). Weg B: `Gui::schreibmarke` (dieselbe Rechnung wie das Zeichnen der Marke) -> `Graphics::ime_position` -> `ImmSetCompositionWindow`/`ImmSetCandidateWindow` (Feature `Win32_UI_Input_Ime`), nur bei Aenderung; ohne IME hier nicht messbar. **Gemessen:** erstes Bild mit Kanji+Hangul+Emoji+Hebraeisch ~100 ms, weiteres Kanji ~15 ms. Tests `tests/pruef/schriften_vorrat.dhtest` (Bildvergleich echt gegen `?` ueber `dhrt bild` + GETPIXEL, Gegenprobe U+E000; bis 2026-09-16 pytest mit Pillow). Bewusst NICHT: Textformung/RTL, Farb-Emoji, Wahl der Schrift INNERHALB einer Sammlung. **Weg C (Vorschau im Feld) kam am selben Tag nach:** `ime.rs` (cfg windows) haengt per `SetWindowLongPtrW` einen zweiten Subclass an raylibs GLFW-Fenster (Kette ruft den Vorgaenger, also AccessKit), nimmt `WM_IME_STARTCOMPOSITION`/`COMPOSITION`/`ENDCOMPOSITION` selbst (dann entsteht das Systemfenster der IME gar nicht) und verschluckt `WM_IME_CHAR` (das Ergebnis kommt ueber `GCS_RESULTSTR`, sonst kaeme alles doppelt) -- NUR solange `feld_aktiv` (gui-Textfeld mit Fokus, von der VM je Bild gesetzt), sonst laeuft alles durch wie bisher (INKEY$, ui-Modul). Zustand in einem `static Mutex` (die Fensterprozedur ist eine freie `extern "system"`-Funktion), `abholen()` je Bild VOR `gui.update`: `Gui::ime_ergebnis` (Einfuegen an der Marke durch Hoechstlaenge/Zahlenfilter, Undo, on_change), `Gui::ime_vorschau` (Widget-Felder `vorschau`/`vorschau_marke`, transient), `anzeige_mit_vorschau` = EINE Quelle fuer Zeichnen, Unterstreichen, Schreibmarke und die Meldung an die IME. Waehrend einer Umwandlung filtert die IME die Tasten (GLFW: `VK_PROCESSKEY` -> kein Ereignis, win32_window.c:776). Ohne installierte IME NICHT gemessen -- Rust-Test `ime_ergebnis_und_vorschau` deckt die Zeichenkettenseite.

**Bedienung ohne Maus** (seit 2026-08-30): `TAB`/`SHIFT+TAB` laeuft durch ALLE bedienbaren Widgets (vorher nur TextInput/TextArea -- ein Fenster ohne Textfeld war per Tastatur gar nicht bedienbar); Leertaste/Enter loest aus, Pfeile verstellen Werte bzw. bewegen Auswahlen, `ESC` schliesst eine offene Klappliste. EINE Quelle dafuer ist `Kind::fokussierbar()` -- Tab-Zyklus, Klick-Fokus und Fokus-Ring fragen alle dort. Der Ring (Akzentfarbe, 2 px ausserhalb) wird an EINER Stelle am Ende von `draw_widget` gezeichnet; ohne sichtbaren Fokus waere die Navigation wertlos. Abfragbar mit `GUI_FOCUSED()` (-1 = keins), Gegenstueck zu `GUI_FOCUS`. Weil damit auch Knopf/Kaestchen/Klappliste Fokus fuehren, feuern `on_focus`/`on_blur` dort jetzt ebenfalls -- der Form-Designer bietet sie entsprechend an (`_FOKUS` in formdesigner/document.py). Tests `tests/pruef/gui_tastatur.dhtest` (echte Tasten via Automation-Wiedergabe, darum in `_SERIELL`). Window-Drag an der Titelleiste, Z-Order (Klick bringt nach vorne), Fokus, Schliessen-Button, **resizeable Fenster** (`GUI_WINDOW_RESIZABLE` — am unteren-rechten Griff ziehbar, `GUI_WINDOW_SET_MIN_SIZE`/`MAX_SIZE` als Grenzen; in `.dhform`-JSON als `resizable`/`min_w`/`min_h`/`max_w`/`max_h`) + **Control-Anchoring** (`GUI_SET_ANCHOR(wdg, "lrtb")` — Reflow beim Resize: Widgets kleben an Kanten/dehnen sich; in JSON als `anchor`-Edge-String) + **randlos** (`GUI_WINDOW_CHROME(win, an)` — ohne Titelleiste/Rahmen, damit eine Form das OS-Fenster füllt; der Form-Designer-Run koppelt die Form so ans native OS-Fenster). **Widgets werden auf den Fenster-Innenbereich geclippt.** Cyan-Theme (programmierbar). Konstruktoren/Getter sind `@builtin`, nur `GUI_UPDATE`/`GUI_DRAW` sind `graphics_builtin`. Komplement zum Immediate-Mode-`ui`-Modul (dort `UI_WINDOW_BEGIN/END` + `UI_TABLE` mit `UI_TABLE_SELECTED`/`SET_SELECTED`/`HEADER_CLICK`). Doku `docs/module-gui.md`, Demos `examples/45_gui.dh` + `examples/81_table_select.dh` + **`examples/156_gui_alle_widgets.dh`** (alle 22 Widget-Arten in EINER Vollbild-Anwendung, jedes mit echter Aufgabe: Baum filtert Tabelle, Tabellenzeile fuellt Editor, Regler formen die Kurve auf der Zeichenflaeche -- der schnellste Weg, eine Widget-Art in Aktion zu sehen), Tests `tests/test_gui_*.py`.


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

## Verfügbare Built-in-Module

| Modul | Funktionen (Auswahl) | Externer Typ |
|---|---|---|
| `json` | `JSON_PARSE/LOAD/STRINGIFY`, `JSON_GET_STRING/INT/FLOAT/BOOL`, `JSON_GET_JSON` (Teilbaum als Kopie), Pfad-Notation `"user.name"` / `"items.0"` | `JSON_HANDLE` |
| `db` | SQLite. `DB_OPEN/CLOSE`, `DB_EXEC/QUERY` mit `?`-Binding, `DB_NEXT`, `DB_GET_*`, `DB_BEGIN/COMMIT/ROLLBACK` | `DB_CONN`, `DB_RESULT` |
| `tween` | **Kein `TWEEN_UPDATE`** (Absicht, keine Luecke): ein Tween rechnet seinen Wert bei jedem Abruf aus `MILLIS()` aus, laeuft also in ECHTER Zeit weiter statt bildgetrieben wie `timer`/`input`/`gui`. Folgen: bei einbrechender Bildrate SPRINGT er statt langsamer zu werden, und unter `AUTOMATION_PLAY` ist er nicht reproduzierbar (`timer` ist es). Werteinterpolation. 13 Easings (`linear`, `out_bounce`, `out_elastic`, …), Pause/Resume/Reverse | `TWEEN` |
| `timer` | Geplante Aktionen ohne MILLIS-Buchführung: `TIMER_AFTER/EVERY(ms, fnref)` → ID (FUNCREF-Callbacks, parameterlos), `TIMER_UPDATE()` pro Frame feuert die fälligen (Muster wie INPUT_UPDATE/GUI_UPDATE; EVERY max. 1×/Update, kein Aufhol-Burst), `TIMER_CANCEL/ACTIVE/COUNT/CLEAR` (Tombstone-stabile IDs). Plus `COOLDOWN(id$, ms)` — String-ID-Ratenbegrenzer (TRUE wenn frei, startet dann die Sperre; braucht kein UPDATE). Konsolen-tauglich (kein Grafik-Bezug; `rust/drachenhauch_runtime/src/timer.rs` + `try_timer` in vm.rs). Doku `docs/module-timer.md`, Demo `examples/113_timer.dh`, Tests `tests/pruef/modules_timer.dhtest`. | — |
| `imgfx` | `IMAGE_SCALE/ROTATE/FLIP/TINT/COPY` — immutable, geben neues IMAGE zurück. **`IMAGE_SCALE` glaettet bilinear** (raylib `ImageResize`); fuer Pixelgrafik `IMAGE_SCALE_NN` (Nearest-Neighbour, `ImageResizeNN`) — Demo `examples/152_pixelart_skalierung.dh` | — |
| `particles` | Emitter mit Velocity/Lifetime/Gravity/Color/Size/Fade. `PARTICLE_EMIT/UPDATE/DRAW`. NumPy-vektorisiert. **Render-Modi** `PARTICLE_SET_MODE` (`circle`/`pixel`/`square`/`streak`/`glow` — `glow` wird in `PARTICLE_DRAW` (vm.rs) aktuell identisch zu `circle` gerendert, kein additives Blending im Recording-Modell; fuer echtes additives Leuchten `BLEND_MODE("add")` um die `PARTICLE_DRAW`-Aufrufe legen) + **Farbverlauf** `PARTICLE_SET_COLOR_END` (Start→End ueber die Lebenszeit, z.B. Feuer gelb→rot). | `PARTICLE_SYSTEM` |
| `physics` | Pure Functions: AABB-/Circle-Collision, Distance, Reflect, Normalize, Ray-Cast (Box+Circle). Kein State. Auch **3D-Mathematik ohne Physik-Welt**: `PHYSICS_SPHERE_SPHERE(x1,y1,z1,r1, x2,y2,z2,r2)` (Kugel-Naeherung), `PHYSICS_DISTANCE3`; dazu `PHYSICS_POINT_TRI(px,py, ax,ay, bx,by, cx,cy)` (Punkt im Dreieck, baryzentrisch — unabhaengig vom Umlaufsinn). Plus **Broadphase** (`PHYSICS_BROAD_NEW/ADD/QUERY/PAIR_A/PAIR_B`): O(n)-Kollisionspaare fuer viele Kreis-Entities (Uniform-Grid, nativ via `gb_native`). | `PHYSICS_BROAD` |
| `physics3d` | **Echte 3D-Starrkoerper-Physik via Rapier3D** (voller Solver: Schwerkraft, Integration, Kollisionsaufloesung, Restitution/Reibung — kein blosses Kollisions-Toolkit wie `physics`). `PHYS3D_NEW`, `PHYS3D_SET_GRAVITY`, `PHYS3D_ADD_BOX`/`PHYS3D_ADD_SPHERE(..., dynamic, bounce)`, `PHYS3D_STEP(w, dt)`, `PHYS3D_BODY_X/Y/Z` + `BODY_QX/QY/QZ/QW` (Quaternion -> `MAT4_TRS`/`MODEL_MATRIX`), `PHYS3D_SET_VEL`/`APPLY_IMPULSE`/`SET_POS`/`REMOVE`/`COUNT`. Koerper-Index stabil (Tombstones). Rapier3D ist pure-Rust (nalgebra) -> ungated in dhrt. Demo `examples/107_physics3d.dh`, Tests `tests/pruef/physics3d.dhtest`. | `PHYS_WORLD` |
| `physics2d` | **Echte 2D-Starrkoerper-Physik via Rapier2D** (voller Solver wie `physics3d`, nur 2D — fuer Stapeln/Werfen/Rollen/Sandbox; nicht zu verwechseln mit `physics` = nur Kollisions-Mathe). `PHYS2D_NEW`, `PHYS2D_SET_GRAVITY(w,gx,gy)`, `PHYS2D_ADD_BOX(w,x,y,hw,hh,dynamic,bounce)`/`PHYS2D_ADD_CIRCLE(w,x,y,r,...)`, `PHYS2D_STEP(w,dt)`, `PHYS2D_BODY_X/Y/ANGLE/VX/VY`, `PHYS2D_SET_VEL`/`APPLY_IMPULSE`/`SET_POS`/`LOCK_ROTATION`/**`SET_DYNAMIC`**/`IS_DYNAMIC`/`REMOVE`/`COUNT` (`SET_DYNAMIC` schaltet statisch<->dynamisch um -- fuer Aufbauten, die erst stehen und dann zusammenfallen). **Bildschirm-Konvention** (Y unten, Default-Gravitation 0/980), `length_unit=100` fuer Pixel-Stabilitaet; Box-Maße = Halb-Extents; `dynamic`-Flag akzeptiert TRUE/FALSE oder 1/0 (Helfer `need_flag`). Koerper-Index stabil (Tombstones). Rapier2D pure-Rust -> ungated. Doku `docs/module-physics2d.md`, Demo `examples/112_physics2d.dh`, Tests `tests/pruef/physics2d.dhtest`. | `PHYS2D_WORLD` |
| `camera` | World-Translation+Zoom+**Rotation** für **alle** Drawing-Befehle. `CAMERA_SET/RESET/FOLLOW`, `CAMERA_SET_ROTATION`/`CAMERA_ROTATION`, `CAMERA_S2W_X/Y`. Rotation dreht nur Positionen (um die Bildschirm-Mitte), keine automatische Kontur-Rotation von Formen/Sprites — siehe `docs/module-camera.md`. | — |
| `sprite` | Animiertes Sheet-basiertes Sprite. Position+Velocity, benannte Animationen mit FPS, `PLAY`/`PLAY_ONCE`, Flip, AABB-Kollision | `SPRITE` |
| `animfsm` | **Animations-State-Machine** (Unity-Mecanim-Stil), datengetrieben aus `.dhanim`-JSON (Editor `dhanim`): States (an Sprite-Anim gebunden) + Parameter (`bool`/`float`/`int`/`trigger`) + Transitions mit Bedingungen (`gt`/`lt`/`eq`/…, Any-State `*`, `wait_finished` für one-shot). `ANIM_FSM_LOAD/SETUP/UPDATE(fsm,sprite,dt)/SET_*/TRIGGER/STATE/FORCE`. Doku `docs/module-animfsm.md`, Demo `examples/111_anim_fsm.dh`, Tests `tests/pruef/animfsm.dhtest`. | `ANIM_FSM` |
| `ui` | Immediate-Mode-UI. `UI_LABEL`, `UI_BUTTON`, `UI_CHECKBOX`, `UI_SLIDER` mit String-IDs für State. Pflicht: `UI_END_FRAME()` vor `FLIP()`. **Plastischer Look wie `gui`:** Themen `glas_dunkel`/`glas_hell` + Metriken `gradient`/`gloss`/`bevel`/`corner_radius` (0 = flach, alle alten Themen unveraendert); ein Preset setzt Farben UND Plastik. Gemeinsame Flaechen-Routine `ui_flaeche` in vm.rs (erhaben/versenkt). **Gotcha:** die Plastik-Werte muessen VOR `self.gfx.as_mut()` gelesen werden -- der Aufruf leiht `self` veraenderlich aus, danach ist `self.ui_state` nicht mehr lesbar (daher `UiPlastik` als Buendel). | — |
| `scene` | Stack-basierter Scene-Manager. `SCENE_PUSH/POP/SWITCH/CURRENT`, pro-Scene-Daten via `SCENE_SET_INT/FLOAT/STRING/BOOL` + `_OR`-Variante. | — |
| `save` | Persistente Save-Slots, JSON-Backend, Versionsfeld. `SAVE_NEW/LOAD/LOAD_OR_NEW/WRITE`, `SAVE_SET/GET_INT/FLOAT/STRING/BOOL`. | `SAVE_HANDLE` |
| `astar` | A*-Pathfinding auf Tile-Grid. `ASTAR_NEW/SET_WALL/FIND/PATH_X/PATH_Y`. Manhattan/Euclid/Chebyshev, Diagonal-Toggle, Anti-Cornercutting. | `ASTAR_GRID` |
| `vec2` | 2D-Vektor mit Operator-Overloading (`+`, `-`, `*`, `/`, `=`, `<>`). `VEC2_NEW/X/Y/LENGTH/NORMALIZE/DOT/CROSS/DISTANCE/LERP/PERP/REFLECT/ANGLE/FROM_ANGLE`. Immutable. | `VEC2` |
| `m3d` | 3D-Mathe: **VEC3/VEC4/QUAT/MAT4** (immutable, Operator-Overloading `+ - * / = <>`, inkl. `mat*mat`/`mat*vec`/`quat*quat`). Quaternionen (`QUAT_FROM_AXIS_ANGLE/EULER/SLERP/ROTATE_VEC3`), Matrizen (`MAT4_TRS/MUL/INVERT/LOOKAT/PERSPECTIVE/ORTHO/...`, column-major). Rendering via **`MODEL_MATRIX(handle, mat[, tint])`** (hierarchische Transforms/Bones/Gizmos) + **`MODEL_INSTANCED(handle, mats[, tint[, anzahl]])`** -- `tint` darf eine Farbe ODER ein `ARRAY OF INTEGER` sein (eine Farbe je Matrix); die Laufzeit gruppiert dann nach Farben und zeichnet **einen Draw-Call je VERSCHIEDENER Farbe**, nicht je Instanz (raylibs `DrawMeshInstanced` uebertraegt nur Matrizen, keine Farb-Attribute -- bei sehr vielen verschiedenen Farben ist ein Verlauf im Shader die bessere Antwort). Echtes GPU-Instancing: dasselbe Mesh mit N MAT4-Welt-Matrizen aus einem `ARRAY OF MAT4`/`TUPLE` in EINEM Draw-Call via raylib `DrawMeshInstanced`; eigener schlanker Instancing-Shader mit Ambient+bis-4-Lichtern, kein PBR/IBL/Schatten/Normal-Maps) + **`CAMERA3D_VIEW/PROJECTION(mat)`** (Ortho/Custom-Frustum) — native-only (dhrt). Doku `docs/module-m3d.md`, Demos `examples/103_m3d.dh` + `examples/104_instancing.dh`, Tests `tests/pruef/m3d.dhtest`. | `VEC3`/`VEC4`/`QUAT`/`MAT4` |
| `input` | Action-basiertes Input-Mapping mit Edge-Detection. `INPUT_BIND/UNBIND/UPDATE`, `INPUT_HELD/PRESSED/RELEASED/AXIS/BOUND`. Multi-Key-Bindings. **Gamepad-Support**: `JOY_BUTTON_A..Y`, `JOY_DPAD_*` als Bind-Codes, `INPUT_JOY_AXIS(slot, "left_x")` mit Deadzone. | — |
| `regex` | Python-kompatible Pattern-Matching. `REGEX_MATCH/TEST/FIND/FIND_ALL/REPLACE/REPLACE_ONCE/SPLIT`. Pattern-Cache fuer wiederholte Aufrufe. | — |
| `audio` | Erweiterte Audio-API (nativ in dhrt ueber **Kira**/cpal -- eigener Audio-Thread, vom Game-Loop entkoppelt; loeste 2026-06-13 raylib-Audio ab, `rust/drachenhauch_runtime/src/audio.rs`). Channels, Pause/Resume/Fade (native Kira-Tweens), Stereo-Pan, Music-Position (lesen `AUDIO_MUSIC_POSITION`, setzen **`AUDIO_MUSIC_SEEK(sekunden)`** -- der Sprung wirkt erst nach ~0,3-0,4 s, weil der Stream seinen Vorlauf zu Ende spielt; **MOD/XM koennen es nicht** und melden das: ihre Zeitachse sind Pattern/Zeilen, die Sekunden bis dorthin haengen an Tempowechseln im Stueck. Ohne laufende Musik ist es ein FEHLER -- anders als bei PAUSE/RESUME, wo ein Nicht-Treffer nichts verliert, fiele hier ein `LOAD : SEEK : PLAY` lautlos auf Position 0 zurueck). Tone-Generation (`AUDIO_TONE`/`AUDIO_NOISE`) mit Sine/Square/Saw/Triangle/Noise. **`AUDIO_SFX`** -- prozeduraler sfxr-Stil-Synth (Waveform + Pitch-Slide + ADSR + Vibrato + optionale `stereo_width` fuer breiten Stereo-Sound; geteilte Mathematik in drachenhauch/synth.py; der SFX-Generator `dhsfx` exportiert solche Aufrufe, Pan via `AUDIO_PAN`). Liefert kompatible `SOUND`-Objekte (auch fuer `PLAYSOUND` nutzbar). **Tracker-Module** `.mod`/`.xm` laufen ueber `PLAYMUSIC`/`AUDIO_MUSIC_LOAD` in **Echtzeit gestreamt** (Kira-Custom-`Sound` `ModuleSound` pollt den reinen Rust-Player `xmrs`/`xmrsplayer` auf dem Audio-Thread; sofort geladen, exaktes Endlos-Loopen, Pitch-Resampler + Volume-Ramp im Sound, Steuerung via `Arc<ModShared>`-Atomics, Modul geleakt + im Drop freigegeben) -- echter 4-Kanal-Amiga-Sound, Demo `examples/115_modplayer.dh`. **Sampler `SAMPLE_*`** (Amiga/Paula-Prinzip): `SAMPLE_LOAD(pfad$)->SAMPLE`, `SAMPLE_PLAY(sample, halbtoene, vol[, dur_ms])->AUDIO_CHANNEL` (Resampling per linearer Interpolation = Tonhoehe wie Geschwindigkeit; resampelte Noten gecacht), `SAMPLE_SET_LOOP(s, a, e[, "pingpong"])`/`SAMPLE_LEN`, dazu `SAMPLE_FROM_BUFFER` (16-Bit-PCM) und `SAMPLE_NOTE` (Note als SOUND mit AUDIO_NOTE-Huellkurve, fuer AUDIO_PLAY_AT/MIX). One-Shot (dur<=0) fuer Drums/Hits, dur>0 + Loop-Region fuer gehaltene Noten. Reine Resampling-Mathematik = freie `resample()` in audio.rs (Rust-`#[test]`); Demo `examples/116_sampler.dh`. **Paula-Lo-Fi** `AUDIO_LOFI(an[, bits[, cutoff_hz]])` -- Bit-Crush (Default 8-bit) + LED-Tiefpass (Default 3300 Hz) fuer NEU synthetisierte Sounds (TONE/NOISE/SFX/SAMPLE_PLAY; Cache wird geleert); pure `lofi_chain()` mit Rust-`#[test]`. **Mixer-Busse** `AUDIO_BUS_VOLUME(bus$, vol)`/`AUDIO_BUS_GET_VOLUME(bus$)` mit `bus$` = `sfx`/`music`/`master` -- SFX-/Musik-Master getrennt (Kira-Sub-Tracks: SFX/Sampler/Synth -> sfx_track, Musik -> music_track, beide -> Main mit dem FFT-Tap; Bus×Sound-Volume multiplizieren). **Echtzeit-Effekte je Bus** (Kira-Effektkette am Track, live steuerbar, kein Buffer-Bake): `AUDIO_FILTER(bus$, cutoff_hz[, resonance])` (Tiefpass, SID/Acid-Sweep), `AUDIO_REVERB(bus$, mix[, feedback[, damping]])` (Hall), `AUDIO_DELAY(bus$, mix[, feedback[, time_ms]])` (Echo, eigener Ringpuffer-Effekt -> Zeit zur Laufzeit aenderbar, 1..4000 ms), `AUDIO_DISTORTION(bus$, amount[, mix])` (Overdrive/Fuzz), `AUDIO_COMPRESSOR(bus$, threshold_db, ratio[, makeup_db])` (Dynamik, ratio<=1=aus), `AUDIO_EQ(bus$, freq_hz, gain_db[, q])` (Glocken-EQ, gain 0=transparent); Signalfluss EQ->Filter->Distortion->Compressor->Reverb->Delay, neutral bis aktiviert, Demo `examples/117_audiofx.dh`. **Clock** `AUDIO_CLOCK_NEW(ticks_per_second)->AUDIO_CLOCK` (Kira-Uhr fuer sample-genaues Musik-/Rhythmus-Timing; startet pausiert) + `AUDIO_CLOCK_START/PAUSE/STOP/REMOVE`, `AUDIO_CLOCK_TICKING`/`AUDIO_CLOCK_TICKS`, `AUDIO_CLOCK_SET_SPEED` -- und **`AUDIO_PLAY_AT(sound, clock, ticks[, volume[, loops]])`**: Sound-Start exakt auf einen Clock-Tick geplant, getrieben vom Kira-Audio-Thread selbst (KEIN Polling/Update-Call noetig -- anders als das frame-getriebene `timer`-Modul). BPM->ticks_per_second rechnet der Aufrufer selbst um (`bpm / 60.0 * subdivisions`). Ticking-Status wird im Wrapper selbst mitgefuehrt (nicht direkt Kiras `ClockHandle::ticking()`), weil Kira das nur asynchron per Audio-Thread-Kommando spiegelt -- eine Abfrage direkt nach START/STOP koennte sonst kurz den alten Wert zeigen. **Nicht-lineare Easings** fuer Fades/Slides: optionaler trailing `easing$`-Parameter (`"linear"` Default/`"in"`/`"out"`/`"inout"`, quadratisch) bei `AUDIO_PLAY` (fade_in_ms), `AUDIO_STOP` (fade_out_ms), `AUDIO_PAN_SLIDE` (dauer_ms), `AUDIO_MUSIC_PLAY`/`AUDIO_MUSIC_STOP` (fade_in/out_ms) -- vorher liefen alle Tweens linear, obwohl Kira `Easing::{In,Out,InOut}Powi` eingebaut hat. Interner Helfer `FadeCurve` (audio.rs) konvertiert zu `kira::Easing` fuer den Kira-Tween-Pfad (Stream/Static-Sounds) UND dupliziert dieselbe Kurven-Mathematik als reine `apply()`-Funktion fuer den MOD/XM-Modul-Fade (eigener Atomics-Ramp in `ModShared`, kein Kira-Tween beteiligt) -- beide Pfade klingen dadurch identisch. Rust-`#[test]`s verifizieren die Kurven-Mathematik gegen Kiras eigene Formel. **Raeumliches Audio (Listener/Emitter):** `AUDIO_LISTENER_NEW(x,y,z)->AUDIO_LISTENER` ("Ohr" der Szene, z.B. Kamera-/Spielerposition; unrotiert blickt -Z), `AUDIO_LISTENER_SET_POSITION`/`AUDIO_LISTENER_SET_ORIENTATION(listener, yaw_grad)` (nur Y-Achsen-Yaw -- deckt die typische Top-Down-/3rd-Person-Kamera ab, ohne BASIC-Nutzern volle Quaternionen zuzumuten) + `AUDIO_LISTENER_REMOVE`; `AUDIO_EMITTER_NEW(listener,x,y,z[,min_dist[,max_dist]])->AUDIO_EMITTER` (ein raeumlicher Kira-Sub-Track, an einen Listener + Position gebunden; Kira berechnet Panning + lineare Lautstaerke-Abnahme zwischen min_dist=laut/max_dist=lautlos komplett selbst -- keine eigene DSP) + `AUDIO_EMITTER_SET_POSITION`/`AUDIO_EMITTER_REMOVE`; **`AUDIO_PLAY_ON(sound,emitter[,loops[,volume[,fade_in_ms[,easing$]]]])->AUDIO_CHANNEL`** startet einen Sound auf dem Emitter-Track statt dem flachen SFX-Bus -- der zurueckgegebene `AUDIO_CHANNEL` ist danach identisch mit `AUDIO_PAUSE`/`STOP`/`VOLUME`/... steuerbar (`StaticSoundHandle` unterscheidet nicht, von welchem Track-Typ es kommt). Listener/Emitter im selben Tombstone-Vec-Pattern wie Clocks (Kira kennt weder `remove_listener()` noch `remove_spatial_sub_track()` -- nur Handle-Drop). `mint`-Crate (winzige, abhaengigkeitsfrei Interop-Structs) baut die Position/Rotation-Werte fuer Kiras API, ohne `glam` direkt einzubinden. Rust-`#[test]`s verifizieren `yaw_quat()` (Einheits-Quaternion, korrekte Komponenten). Demo `examples/139_audio_spatial.dh`. **Modulatoren (LFO + Tweener):** `AUDIO_LFO_NEW(wellenform$, hz [, amplitude [, mitte]])` -> `AUDIO_MOD` (`sine`/`triangle`/`saw`/`pulse`), `AUDIO_LFO_SET`, `AUDIO_LFO_WAVEFORM`; `AUDIO_TWEENER_NEW([start])` + `AUDIO_TWEENER_TO(mod, ziel, dauer_ms [, easing$])`; gebunden per `AUDIO_MODULATE(bus$, ziel$, mod, min, max)` mit ziel$ = **`volume`** (Tremolo) / **`pan`** (Auto-Pan) / `filter` / `resonance` / `reverb` / `distortion`; dazu `AUDIO_BUS_PAN(bus$, pos)` fuer eine feste Bus-Balance (-1..+1) -- vorher liess sich nur ein EINZELNER Kanal pannen, entfernt per `AUDIO_MOD_REMOVE`. Der Wertebereich des Modulators (LFO: -1..+1 bei Standard-Amplitude) wird auf `min..max` abgebildet. **Der Punkt daran:** Kira faehrt sie auf dem AUDIO-Thread -- Tremolo, Vibrato, Wobble-Bass, Auto-Pan und Filter-Sweeps laufen sample-genau weiter, auch wenn die Bildrate einbricht, und das GB-Programm rechnet pro Frame NICHTS nach. LFO und Tweener teilen sich den Handle-Typ `AUDIO_MOD`; ein LFO-Aufruf auf einem Tweener (und umgekehrt) meldet das im Klartext. Doku `docs/module-audio-modulatoren.md`, Demo `examples/150_audio_modulatoren.dh`. | `AUDIO_CHANNEL`, `SAMPLE`, `AUDIO_CLOCK`, `AUDIO_LISTENER`, `AUDIO_EMITTER`, `AUDIO_MOD` |
**Noten und Mischen** (2026-09-04, gefunden beim Tracker-Piloten): `AUDIO_NOTE(wf$, freq, dauer_ms, attack, decay, sustain, release, vol[, vib_depth, vib_speed, detune_cents, slide_halbtoene])` -- eine gehaltene Note mit echter ADSR (Sustain-PEGEL, Release haengt hinten an, Laenge = dauer + release), Detune-Schicht und Portamento in Halbtoenen; `AUDIO_SFX` kann das nicht, es kennt nur drei Zeiten. `AUDIO_SOUND_NEW(dauer_ms)` (Stille), `AUDIO_SOUND_MIX(ziel, quelle, offset_ms[, vol[, pan]])` (addieren, NICHT geklemmt, ohne pan bleiben die Kanaele der Quelle), `AUDIO_SOUND_NORMALIZE(sound[, spitze])` (liefert den Faktor) -- damit wird aus vielen Klaengen eine WAV. Rechenkern `build_note_buffer` mit Rust-Tests an der nachgemessenen Huellkurve; Tests `tests/pruef/audio_note_mix.dhtest`.

**FLAC** (2026-09-21, Frage des Nutzers): als Klang ging es immer, als MUSIK mit Schleife (die
Vorgabe von `AUDIO_MUSIC_PLAY`) blieb es STUMM -- Kira meldete nichts, der Klang stand nur auf
`Stopped`. Ursache in Symphonia 0.6 (`symphonia-bundle-flac`, `PacketParser::resync`): der
Paketbauer wird nur zurueckgesetzt, wenn sich der Leser beim Resync BEWEGT; landet ein Sprung
genau auf dem ersten Frame (Dateien unter 16 KB immer, mit SEEKTABLE oft), bleibt der Zustand
vom Dateiende stehen und das naechste Lesen endet mit `UnexpectedEof`. Nachgestellt mit
Symphonia allein (lesen bis Ende, seek 0, lesen -> 1 Paket, dann Fehler). Ausweg: eigener
Kira-`Decoder` `src/flac_strom.rs`, der vor jedem Sprung die Datei NEU oeffnet; `musik_strom`
in audio.rs nimmt ihn fuer alles mit der Kennung `fLaC`, sonst bleibt Kiras Weg. `symphonia`
ist dafuer direkte Abhaengigkeit (dieselbe 0.6 wie in Kira). Tests `tests/pruef/audio_flac.dhtest`
(4: Beilagen aus einem kleinen FLAC-Schreiber ausserhalb der Laufzeit -- es gab hier kein
ffmpeg/flac --, mono und stereo Punkt fuer Punkt gegen die Sinusformel, Musik muss
zurueckspringen, kaputte Datei ist ein Fehler).

**Klang anschauen/sichern** (am SOUND-Handle, gilt also fuer TONE/NOISE/SFX/geladene Dateien): `AUDIO_SOUND_WAVE(sound, anzahl)` -> ARRAY OF FLOAT (je Abschnitt das Sample mit dem GROESSTEN BETRAG samt Vorzeichen -- gemittelt hebt sich eine Schwingung gegen null auf und die Anzeige zeigt einen Strich) und `AUDIO_SAVE_WAV(sound, pfad$[, bits])` (16 signed / 8 unsigned, so will es die WAV-Spezifikation; Mono bleibt einkanalig, Stereo erst wenn sich die Kanaele unterscheiden). **Falle, die erst das Nachmessen zeigte:** die Lautstaerke steckt schon in den Frames (`make_data_mono`), `slot.vol` obendrauf machte aus 0.7 eine 0.49 -- und `slot.vol` ist ohnehin die ABSPIEL-Lautstaerke des letzten AUDIO_PLAY. Gebaut fuer den SFX-Generator `examples/183_sfx_generator.dh`, Tests `tests/pruef/audio_sound_io.dhtest` (WAVs mit `--- ton` bzw. ueber ihre Bytes gegengelesen -- ein Format, das nur der eigene Schreiber liest, ist nicht geprueft).

| `chart` | **Diagramme.** `CHART_NEW(art$,x,y,b,h)` -> `CHART` mit art$ = `kuchen`/`donut` (Kuchen/Ring), `balken` (senkrecht/waagerecht, gruppiert/gestapelt), `linie`/`flaeche` (Verlaufskurven, gleitendes Fenster fuer Live-Werte), `tacho` (Rundskala mit Zeiger `nadel`/`balken`/`pfeil`, Farbzonen via `CHART_ZONE`). Daten kurz (`CHART_ADD(c,name$,wert[,farbe])`) oder voll (`CHART_SERIES` + `CHART_DATA`/`CHART_PUSH`/`CHART_SET_POINT`); dazu `CHART_GET/COUNT/SERIES_COUNT/LABEL/CLEAR/BOUNDS/STAT`. **Stil ueber vier String-Setter statt ~40 Builtins:** `CHART_SET` (Text), `CHART_SET_NUM` (Zahlen), `CHART_SET_COLOR` (Farben), `CHART_SET_FLAG` (Schalter) -- Schluessel-Tabellen `KEYS_STR/NUM/COLOR/FLAG` in `chart.rs`, unbekannter Schluessel = Fehler, der die gueltigen auflistet. `CHART_THEME` (dunkel/hell/neon/pastell) + `CHART_PALETTE`. **Alpha/Schatten/Verlaeufe:** alle Farben nehmen `RGBA()` (0xAARRGGBB, Alpha 0 = DECKEND -- Helfer `with_alpha`/`scale_rgb` heben das vorher an); `deckkraft`/`flaeche_deckkraft` als globale Regler, `schatten`+`schatten_weich` (gestaffelte Kopien, raylib hat keinen Formen-Weichzeichner) mit `schatten_daten` auch fuer Balken/Segmente/Zeiger, `verlauf` (Hintergrund) und `verlauf_daten` (Balken/Flaeche senkrecht, Kuchen als abgedunkeltes Innenband = Naeherung, kein Radialverlauf). `animation` + `CHART_UPDATE(c, DELTA())` laesst Werte nachziehen -- **ohne Animation zeichnet `draw` direkt die echten Werte** (`anzeige()`), sonst waere CHART_UPDATE auch ohne Animationswunsch Pflicht. Nur `CHART_DRAW` braucht ein Fenster (in `vm.rs`), alles andere ist pure. Neues Zeichen-Primitiv dafuer: `Cmd::Ring` (raylib `draw_ring`) deckt Kuchenstueck/Donut/Tacho-Bogen ab, plus `text_width_at` (Breite bei expliziter Groesse). **Farbe `0` ist SCHWARZ, nicht "Palette"** -- dafuer `-1` bzw. Argument weglassen. **Sechs Arten** (nicht vier): dazu `leiste`/`bar_gauge` (liegende oder stehende Leiste mit wanderndem Marker) und `led`/`lampen` (diskrete Zellen, leuchten bis zum Wert) -- beide einwertig wie der Tacho, teilen sich dessen Farbzonen. Sie setzen `ausrichtung` selbst auf `waagerecht`, weil die Vorgabe `senkrecht` nur fuer Balkendiagramme richtig ist. **Skalen-Farbverlauf** ist eine EIGENE Farbrolle (`skala_von`/`skala_mitte`/`skala_bis`, rot->gelb->gruen je Thema) -- NICHT die Palette: die ist kategorial und ergibt interpoliert einen Regenbogen ohne Richtung. Farbzonen schlagen den Verlauf. **Tacho-Gestaltung:** `zifferblatt` = `ring`/`segmente`/`striche`/`baender`, `blatt_teile`/`blatt_luecke`/`blatt_dicke`, `fassung` (metallischer Ring aus gestaffelten Ringen -- ein Verlauf ENTLANG eines Kreises geht mit `ring` nicht), `CHART_ZONE(..., name$)` beschriftet die Zone entlang des Bogens (untere Haelfte wird gedreht), `wertanzeige` = `aus`/`innen`/`pille`/`blase`/`am_zeiger` (Pille nimmt die Farbe der getroffenen Zone). Der Tacho haengt allein an `wertanzeige` -- ihn zusaetzlich an `werte` zu koppeln liess ihn stumm, weil das per Vorgabe `aus` ist. **Maus:** `CHART_DRAW` wertet sie selbst aus (kein Zusatzaufruf) -> `CHART_HOVER`/`_SERIES`/`_LABEL$`/`_VALUE`, `CHART_CLICKED`/`_SERIES`; Schalter `hover`/`tooltip`, Zahlen `hover_tempo`/`hover_weite`/`hover_glanz`. Damit die Maus nicht neben dem trifft, was zu sehen ist, liegt die Geometrie an EINER Stelle (`kuchen_geom`/`kuchen_stuecke`/`achsen_geom`/`balken_geom`/`legende_abzug`), die Treffertest UND Zeichnen benutzen. Die Hervorhebung mischt gegen WEISS statt RGB zu skalieren -- beim Skalieren klemmt der groesste Kanal bei 255 und hervorgehobenes Orange wurde gelb. **Linien:** `punktform` (kreis/quadrat/raute/dreieck), `treppe`, `strich` (Strichlaenge; Phase laeuft ueber den GANZEN Zug weiter, sonst verdichtet sich das Muster bei engen Stuetzpunkten), `fadenkreuz`. `glatt`+`treppe` schliessen sich aus, die Treppe gewinnt. Doku `docs/module-chart.md`, Demo `examples/154_chart.dh`, Tests `tests/pruef/modules_chart.dhtest` + Rust-`#[test]`s. | `CHART` |
| `curves` | Animation-Kurven (komplementaer zu `tween`'s Easings): `CURVE_BEZIER/BEZIER2`, `CURVE_CATMULL/CATMULL2`, `CURVE_HERMITE`, `CURVE_LERP`, `CURVE_SMOOTHSTEP`, `CURVE_SMOOTHERSTEP`. Pure Functions, kein State. | — |
| `net` | TCP + UDP via stdlib-Sockets (cross-platform). Default non-blocking fuer Game-Loops. `NET_TCP_LISTEN/ACCEPT/CONNECT`, `NET_SEND/RECV`, `NET_UDP_BIND/SEND/RECV`. Encoding: UTF-8. | `NET_LISTENER`, `NET_SOCKET`, `NET_UDP` |
| `midi` | Noten von einem angeschlossenen Instrument lesen und welche hinausschicken. Feature `midi` (Crate `midir`: WinMM/ALSA/CoreMIDI), also nur in `--hardware`-Bauten -- **ausser** `MIDI_NOTE_NAME$`/`MIDI_NOTE_FREQ`, die nur umrechnen und darum ungegatet in `midi.rs` stehen (samt Rust-`#[test]`s; nur so ist der nuetzlichste Teil auch auf einer Maschine ohne Anschluss pruefbar). Auflisten `MIDI_IN_COUNT/NAME$`, oeffnen `MIDI_IN_OPEN` -> `MIDI_IN`, empfangen ueber das Cursor-Muster von `db`/`mqtt` (`MIDI_NEXT` + `MIDI_NOTE/VELOCITY/CHANNEL/IS_NOTE_ON`), senden `MIDI_NOTE_ON/OFF/CC/SEND`. **Zwei Protokoll-Eigenheiten:** die meisten Instrumente schicken Note-AUS als Note-AN mit Anschlag 0 (`MIDI_IS_NOTE_OFF` faengt beide, sonst enden Toene nie), und Kanaele zaehlen nach aussen 1..16 statt 0..15. Die Entschluesselung eingehender Nachrichten arbeitet ueber ROHE BYTES statt ueber den Geraetetyp (`status_von`/`kanal_von`/`ist_note_aus`/... in midi.rs, die `&Eingang`-Fassungen sind Einzeiler darueber) -- haenge sie am Geraet, ist sie ohne Instrument NIRGENDS pruefbar, so ist sie es ueberall: neun ungegatete Rust-`#[test]`s mit erfundenen Nachrichten decken beide Note-aus-Formen, Kanal 1/16, Regler, leere und zu kurze Nachricht ab. Der Rueckruf laeuft auf midirs eigenem Faden in eine Warteschlange mit **1024** Plaetzen; beim Ueberlauf faellt die AELTESTE weg (wer live spielt, will den aktuellen Anschlag). Uhr/Active-Sensing/SysEx werden weggelassen. **Nicht umgesetzt:** SysEx, MIDI-Uhr/Timecode, `.mid`-Dateien, virtuelle Anschluesse. **Der ganze Kreis ist geprueft** -- Tests, die einen VIRTUELLEN Loopback-Port benutzen (loopMIDI, `winget install TobiasErichsen.loopMIDI`; ein Port unter dem Portnamen als Ein- UND Ausgang, genau daran erkannt, sonst uebersprungen). Damit sind Note-an-mit-Anschlag-0 und der 1024er-Deckel samt aelteste-faellt-weg belegt statt behauptet -- ein Keyboard braucht es dafuer nicht. Doku `docs/module-midi.md`, Demo `examples/181_midi.dh`. | `MIDI_IN`, `MIDI_OUT` |
| `mqtt` | **MQTT-3.1.1-Client** (das im Maker-/IoT-Bereich dominante Pub/Sub-Protokoll fuer ESP32/IoT-Steuerung) -- direkt gegen die OASIS-Spec via `std::net` implementiert, Feature `net` (bereits im Standard-Build, kein neues Crate). Nur **QoS 0** (kein Packet-ID-Ack-Handshake noetig), kein UNSUBSCRIBE/Will/TLS. `MQTT_CONNECT(host,port,client_id[,keepalive_s[,user[,pass]]])`, `MQTT_PUBLISH(h,topic,payload[,retain])`, `MQTT_SUBSCRIBE`, `MQTT_UPDATE` (Pro-Frame-Polling + automatisches Keepalive-PINGREQ), eingehende Nachrichten ueber Cursor-Muster wie `db` (`MQTT_NEXT_MESSAGE`/`MQTT_MESSAGE_TOPIC`/`MQTT_MESSAGE_PAYLOAD`, analog `DB_NEXT`+`DB_GET_*`). Doku `docs/module-mqtt.md`, Demo `examples/148_mqtt.dh`. | `MQTT_HANDLE` |
| `ecs` | Entity-Component-System. World mit Entity-IDs (INTEGER) und benannten typed Components (INT/FLOAT/STRING/BOOL/OBJ). Query 1/2/3-fach via Component-Intersection. `ECS_NEW_ENTITY`, `ECS_ADD_INT`, `ECS_QUERY2`, etc. Plus **Bulk-System-Ops** (`ECS_INTEGRATE_FLOAT`, `ECS_SCALE_FLOAT`, `ECS_FILL_*`, `ECS_CLAMP_FLOAT`, `ECS_REMOVE_DEAD`, `ECS_COUNT_WITH`) — siehe eigener Abschnitt unten. Nativ in `rust/drachenhauch_runtime/src/ecs.rs` (die fruehere Python-Fassung `modules/ecs_py.py` ist mit Stufe B entfernt). | `ECS_WORLD` |
| `html` | HTTP-GET/POST/DOWNLOAD + HTML-Parsing (pure stdlib). `HTTP_GET/POST/DOWNLOAD`, `HTTP_STATUS/HEADER`, `URL_ENCODE/DECODE`, `HTML_TEXT`, `HTML_FIND_ALL`. | — |
| `bt` | Bluetooth Low Energy (BLE) via `bleak`. Scan, Connect, Service/Characteristic-Listing, Read/Write/Notify auf Characteristics. Externer Dep, IoT/Sensor-Targets. | `BT_HANDLE` |
| `serial` | RS-232 / USB-COM nativ ueber die Rust-Crate `serialport` (kein `pyserial` noetig). `SERIAL_OPEN/READ/WRITE/READLINE/AVAILABLE/FLUSH/TIMEOUT`. | `SERIAL_HANDLE` |
| `firmata` | Direkte Arduino/ESP32-**Pin-Steuerung** ueber StandardFirmata (kein eigener Sketch/Text-Protokoll noetig -- einmalig StandardFirmata hochladen). Baut auf derselben `serialport`-Crate wie `serial` auf (Feature `serial`, keine neue Abhaengigkeit). `FIRMATA_PORTS/OPEN/CLOSE/IS_OPEN`, `FIRMATA_PIN_MODE`, `FIRMATA_DIGITAL_WRITE/READ`, `FIRMATA_ANALOG_WRITE/READ`, `FIRMATA_UPDATE` (Pro-Frame-Polling wie `INPUT_UPDATE`/`TIMER_UPDATE`). Nur Pin-I/O -- kein I2C/Servo/OneWire/Stepper/Encoder. **Zwei Nummerierungen** (echte Protokoll-Eigenheit): Schreiben nimmt die rohe digitale Pin-Nummer, `FIRMATA_ANALOG_READ` nimmt den Analog-**Kanal** (A0=0, A1=1, ...) -- nicht dieselbe Zahl fuer denselben physischen Pin. Doku `docs/module-firmata.md`, Demo `examples/147_firmata.dh`. | `FIRMATA_HANDLE` |
| `usb` | USB-HID via `hidapi`. Maker-Boards, Programmer, Custom-Controller. `USB_LIST/OPEN/READ/WRITE/PRODUCT`. | `USB_HANDLE` |
| `wifi` | WiFi-Management (Windows-only via `netsh wlan`). `WIFI_SCAN/CONNECT/DISCONNECT/CURRENT/SIGNAL/PROFILES`. | — |
| `tiled` | Tiled-Maps (JSON-Format, kein TMX) **lesen, aendern, anlegen und schreiben**. `TILED_LOAD`, Layer-/Tile-/Object-Access, Per-Tile/Per-Object-Custom-Properties (`solid`, `damage`, ...). Industriestandard fuer 2D-Level-Design. Plus **Bulk-Ops** fuer Generierung/Editor: `TILED_FILL_RECT`, `TILED_REPLACE`, `TILED_COUNT_GID`, `TILED_FLOOD_FILL` (Bucket-Fill). **Schreiben (2026-08-31, gefunden beim Tilemap-Piloten -- das Modul war bis dahin ein reiner Leser):** `TILED_NEW`, `TILED_ADD_LAYER`, `TILED_ADD_TILESET`, `TILED_TILESET_TILES`, `TILED_SAVE`, dazu `TILED_LAYER_RENAME` / `TILED_LAYER_VISIBLE` / `TILED_LAYER_SET_VISIBLE` / `TILED_REMOVE_LAYER` / **`TILED_MOVE_LAYER`** (die Reihenfolge der Ebenen IST ihre ZEICHENreihenfolge -- die erste liegt hinten, Umsortieren aendert also das Bild; herausnehmen und einsetzen, nicht tauschen, weil ein Tausch bei einem Sprung ueber mehrere Stellen etwas anderes waere). **Die `firstgid` vergibt die Laufzeit selbst** (deshalb braucht `TILED_ADD_TILESET` die Kachelzahl) -- sie von Hand setzen zu lassen waere die unangenehmste Fehlerquelle des Formats: ueberlappende Bereiche zerstoeren stillschweigend die Zuordnung ALLER Kacheln, ohne Fehlermeldung. **Die Sichtbarkeit einer Ebene ist nicht nur Anzeige**, Tiled speichert sie -- ohne den Setter liess sich eine ausgeblendete Ebene gar nicht so sichern. Der Namensindex (Name -> Position) wird bei Umbenennen/Entfernen neu aufgebaut, sonst zeigt ein stehengebliebener Eintrag stumm auf die Nachbar-Ebene. **Geprueft wird der Schreiber an einem FREMDEN Leser** (drachenhauch/tilemap/document.py, dem Modell des Qt-Editors), nicht am eigenen -- ein Format, das nur sein Schreiber wieder liest, ist nicht geprueft, sondern nur in sich stimmig. **Objekt-Ebenen und Eigenschaften anlegen (2026-09-02):** `TILED_ADD_OBJECT_LAYER`, `TILED_ADD_OBJECT`, `TILED_TILE_SET_PROP`/`REMOVE_PROP`, `TILED_OBJECT_SET_PROP`/`REMOVE_PROP` -- bis dahin konnte das Modul beides nur LESEN, also liess sich im Programm kein Spawn-Punkt setzen und keiner Kachel `solid` mitgeben. Der SCHREIBER war schon vollstaendig (`speichern` gab objectgroup und tiles aus), es gab nur nichts zu schreiben. **EIN Setzer je Ziel statt vier:** die Leser muessen typisiert sein (dort sagt der Aufrufer, was er erwartet), der Wert traegt seinen Typ beim Schreiben schon mit sich; ARRAY/MAP/Handles werden abgelehnt, weil Tiled genau vier Arten kennt. Adressiert wird ueber die GID wie beim Lesen, samt Flip-Bit-Maskierung -- sonst legte eine gespiegelte Kachel ihre Eigenschaft woanders ab. Geprueft gegen den FREMDEN Leser (`tests/pruef/tiled_objekte_eigenschaften.dhtest`). Objekte lassen sich auch wieder **entfernen und aendern** (`TILED_REMOVE_OBJECT`, `TILED_OBJECT_SET_NAME`/`SET_TYPE`/`SET_RECT` -- alle vier Masse auf einmal, weil Verschieben und Groessenaendern in einem Editor dieselbe Geste sind). Und `TILED_TILE_PROP_KEYS`/`TILED_OBJECT_PROP_KEYS` zaehlen die Schluessel AUF -- ohne sie liess sich nicht anzeigen, was eine Kachel hat (`TILED_TILE_HAS_PROP` beantwortet nur eine Frage, die man schon kennt); sortiert, weil die Ablage eine HashMap ist. **Zwei Schreiberfehler (2026-09-14):** `nextobjectid` stand fest auf 1 und die Objekt-Kennungen zaehlten je EBENE ab 1 -- Tiled vergab beim Bearbeiten doppelte Kennungen. Jetzt traegt jedes Objekt eine kartenweite `id` (`TiledMap::next_object_id`, beim Laden ueber `kennungen_ordnen` eindeutig gemacht, auch fuer alte Dateien mit Doppelten), und `nextobjectid` liegt ueber allen. Und die Eigenschaften (Kachel wie Objekt) kamen in HashMap-Reihenfolge heraus, dieselbe Karte zweimal gesichert gab zwei Dateien -- jetzt nach Namen sortiert, `tiles` nach Kachelnummer (`eigenschaften_json`/`kacheln_json`). Nicht angelegt werden koennen: isometrische/unendliche Karten und Objekte, die keine Rechtecke sind (Polygon, Ellipse). Doku `docs/module-tiled.md`, Editor `examples/187_tilemap_editor.dh`, Tests `tests/pruef/tiled_schreiben.dhtest`. | `TILED_MAP` |
| `tile_collide` | Box-vs-Tilemap-Kollision. `TILE_SWEEP_X/Y` mit separat-Achsen-Sweep-Pattern. Solid-Detection via `solid`-Property (mit Convention-Fallback). Klassische Platformer-Physik. Sweep nativ via `gb_native.TileCollider` (Solid-Maske einmal gespiegelt+gecacht), sonst Python-`_sweep_axis`. | — |
| `controller` | Character-Controller mit Coyote-Time, Jump-Buffer, Variable-Jump-Height. `CHAR_NEW/SET_INPUT/UPDATE`, `CHAR_X/Y/VX/VY`, `CHAR_ON_GROUND/WALL_LEFT/RIGHT`. Konfigurable Move-Speed, Jump-Velocity, Gravity, Coyote/Buffer-Frames, Variable-Jump-Cut. | `CHAR_CONTROLLER` |
| `g3d` | **3D-Grafik** (`dhrt`, Grafik-Feature — ohne `--no-graphics`-Build). Immediate-Primitive: `CAMERA3D`, `CUBE`/`CUBE_WIRES`, `SPHERE`/`SPHERE_WIRES`, `CYLINDER` (Kegel via r_oben=0), `PLANE`, `LINE3D`, `POINT3D`, `GRID3D`. **3D-Modelle** (wiederverwendbare MODEL-Handles): `LOADMODEL` (OBJ/GLTF), prozedural `MESH_CUBE/SPHERE/CYLINDER/TORUS/KNOT/PLANE` + `MESH_HEIGHTMAP` (Terrain aus Graustufen-Image), zeichnen via `MODEL`/`MODEL_EX` (Achsen-Rotation)/`MODEL_WIRES`, `MODEL_TEXTURE` (Diffuse-Map aus LOADIMAGE). **Skelett-Animation** (geriggte GLTF/IQM): `MODEL_LOAD_ANIMS(pfad$)` -> ANIM_SET (Integer-Handle), `MODEL_ANIM_COUNT/NAME/FRAMES`, `MODEL_ANIMATE(modell, set, anim_idx, frame)` setzt die Pose (frame loopt). Nutzt seit raylib-rs 6.0 dessen RAII-`ModelAnimations`-Collection (`load_model_animations`/`update_model_animation`; Unload automatisch im `Drop` -- loeste den fruehreren rohen-FFI-Workaround ab, der noetig war weil der 5.x-Wrapper die Structs flach kopierte und dann `UnloadModelAnimations` rief -> Use-after-free). **`MODEL_ANIMATE_BLEND(modell, set, anim_a, frame_a, anim_b, frame_b, blend)`** (neu in raylib 6.0 via `UpdateModelAnimationEx`): blendet weich zwischen zwei Animationen desselben Sets (`blend` 0.0=ganz A .. 1.0=ganz B), z.B. fuer Walk->Run-Uebergaenge statt hartem Anim-Wechsel. Demo `examples/108_skeletal_anim.dh` (CC0-Modell via `examples/assets/download_robot.dh`). **Billboards** `BILLBOARD` (Textur zeigt zur Kamera) + **Ray-Kollision/Picking** `RAY_HIT_BOX`/`RAY_HIT_SPHERE` (Distanz oder -1) und `PICK_BOX`/`PICK_SPHERE` (Mausstrahl, Klick-Selektion). **Picking auf echter Flaeche** (nicht nur Huellkoerper): `RAY_HIT_TRI(ursprung, richtung, 3 Punkte)`/`RAY_HIT_QUAD(ursprung, richtung, 4 Punkte)` + `PICK_TRI`/`PICK_QUAD` — Bodenkacheln, Wandstuecke, frei schwebende Panels. Ohne Backface-Culling (eine Flaeche trifft auch von hinten); die Vierecks-Punkte muessen **reihum** liegen; die Richtung wird vor dem Test normalisiert (sonst waere die Distanz in Vielfachen der Richtungslaenge, raylibs Rohverhalten). Demo `examples/151_picking_flaechen.dh`. **Beleuchtung** (PBR/Cook-Torrance, bis 4 Lichter): `LIGHT_ENABLE`/`LIGHT_AMBIENT`/`LIGHT_DIRECTIONAL`/`LIGHT_POINT`/`LIGHT_SET_POS/COLOR/ENABLED` + `MODEL_LIT(modell)` + `MODEL_PBR(modell, metalness, roughness)` (eingebetteter GGX-Shader) + `MODEL_EMISSIVE(modell, farbe, staerke)` (Eigenleuchten pro Modell — durchschlaegt den Fog; mit Bloom-`POSTFX` echter Neon-Glow, Demo `examples/110_emissive_glow.dh`) + `LIGHT_FOG(farbe, dichte)` (Tiefen-Fog) + `LIGHT_ENV(himmel, boden, intensitaet)` (analytisches IBL — Metalle spiegeln die Umgebung) + `LIGHT_ENV_HDR(pfad$ [, intensitaet])` (**echtes HDR-Cubemap-IBL**: laedt ein equirect-.hdr, berechnet Irradiance/Prefilter/BRDF-LUT-Maps, `useIBLMaps`-Gate; analytischer `LIGHT_ENV`-Pfad bleibt Fallback) + `SKYBOX(an)` (zeichnet die HDR-Umgebung als sichtbaren 3D-Hintergrund — env-Cubemap auf einen kamerazentrierten Wuerfel, ohne Depth-Write). **Schatten** `SHADOW_ENABLE([res])`/`SHADOW_AREA(groesse,dist)`/`SHADOW_TARGET(x,y,z)` (Shadow-Mapping via Depth-FBO + PCF; erstes directional Light wirft Schatten, MODEL_LIT-Modelle werfen+empfangen). **Normal-Mapping** `MODEL_TEXTURE_NORMAL(modell,bild)` (TBN-basiert, MODEL_LIT erzeugt Tangenten; useNormalMap-Gate -> lit Modelle ohne Map unveraendert). **Kamera-Modi** `CAMERA3D_UPDATE(mode)` (1=free/2=orbital/3=first_person/4=third_person, raylib UpdateCamera) + Getter `CAMERA3D_X/Y/Z`/`CAMERA3D_TARGET_X/Y/Z`. Render via raylib `begin_mode3D` beim FLIP (3D zuerst, 2D-HUD obenauf). Doku `docs/rust-runtime.md` (Schritt 6), Demos `examples/82_3d_intro.dh`, `88_3d_models.dh`, `90_billboards_picking.dh`, `91_lighting.dh`, `92_fog.dh`, `93_shadows.dh`, `94_normalmap.dh`, `95_pbr.dh`, `96_ibl.dh`, `99_ibl_hdr.dh`. | — |

**Zusätzlich als Core-Graphics-Built-ins** (kein IMPORT noetig, in dhrt
`vm.rs`/`try_graphics`):

| Bereich | Funktionen | Externer Typ |
|---|---|---|
| Asset-Cache | `LOAD_ASSETS(manifest.json)` — bulk-Preload mit Alias-Cache. `LOADIMAGE` / `LOADSOUND` cachen automatisch (rohem + abs Pfad). | — |
| Z-Layer | `LAYER_DEFINE(name, z)`, `LAYER(name)`, `LAYER_END()`, `LAYER_CLEAR(name)`. Layer-Surfaces mit SRCALPHA, FLIP composiert in z-Order und cleart. | — |
| Sprite-Atlas | `ATLAS_LOAD(manifest.json)` -> `SPRITE_ATLAS`. `ATLAS_DRAW(atlas, name, x, y)` zeichnet einzeln, Camera-aware. `BATCH_DRAW(...)`/`BATCH_FLUSH()` existieren aus Kompatibilitaet zur alten Python-Engine, sind in dhrt aber **kein echtes Batching**: `BATCH_DRAW` ist derselbe Dispatch-Arm wie `ATLAS_DRAW` (sofortiges Emit in den Layer-Command-Puffer), `BATCH_FLUSH()` ist ein No-Op (`vm.rs`: "Recording-Modell: alles flusht beim FLIP"). Kein separater Batch-Queue-Zustand, keine Draw-Call-Ersparnis gegenueber `ATLAS_DRAW`. | `SPRITE_ATLAS` |
| Bulk-Plot | `PLOTS(xs, ys, color [, anzahl])` — viele Pixel in EINEM Aufruf (vektorisiert), `color` = INT (alle gleich) oder ARRAY OF INT (pro Pixel). Groessenordnungen schneller als `PLOT` in einer Schleife (Starfields, Punktwolken). **`anzahl`** zeichnet nur die ersten n Eintraege — ohne sie wird IMMER das ganze Array gezeichnet, ein fest dimensionierter Puffer schleppt also seine ungenutzten Plaetze mit ins Bild. Ein Argument zu viel ist ein Fehler (frueher still ignoriert). | — |
| Bulk-Shapes | `BOXES(x1s,y1s,x2s,y2s,color[,anzahl])`, `CIRCLES(xs,ys,rs,color[,anzahl])`, `LINES(x1s,y1s,x2s,y2s,color[,anzahl])` — viele Shapes in EINEM Builtin-Call (spart den Dispatch pro Shape; gezeichnet wird pro Shape). `color` = INT oder ARRAY (darf laenger als `anzahl` sein). | — |
| Bulk-Tilemap | `TILED_FILL_RECT`, `TILED_REPLACE`, `TILED_COUNT_GID`, `TILED_FLOOD_FILL` (Bucket-Fill, nativ via `gb_native`) — siehe `tiled`-Modul. `DRAWTILEMAP` rendert intern via `blits()`-Batch (1 Call statt rows×cols). | — |
| 2D-Extras | **Nativ in dhrt:** `LINEW(x1,y1,x2,y2,breite[,c])` (dicke Linie), `BOXROUND`/`RECTROUND(x1,y1,x2,y2,radius[,c])` (runde Rechtecke gefuellt/Umriss), `GRADIENTV`/`GRADIENTH(x1,y1,x2,y2,c1,c2)` (Farbverlauf-Blocks), `SPLINE(xs,ys[,c[,breite]])` (Catmull-Rom durch Punkte). Demo `examples/100_2d_extras.dh`. | — |
| Clip-Rechteck | `SCISSOR(x,y,b,h)` / `SCISSOR_END()` / `SCISSOR_DEPTH()` -- Zeichnen auf ein Rechteck beschraenken. Ein **Stapel**: innen wird mit aussen GESCHNITTEN. Die Mechanik lag laengst in `graphics.rs` (`push_clip`/`pop_clip`, vom `gui`-Modul benutzt) und war nur nicht herausgefuehrt; neu sind die drei Builtins plus ein Zaehler, damit ein SCISSOR_END zu viel ein Fehler ist statt dem Umgebenden seinen Clip wegzunehmen. Vorher blieb dafuer nur ein RENDERTARGET. | — |
| Blend-Modes | `BLEND_MODE(modus$)` — `"alpha"`/`"add"`/`"mult"`/`"subtract"` fuer folgende Draws (Glow via additiv). **Nur native** (raylib `BeginBlendMode`); Tree-Walker konsolen-only -> wirft "nur dhrt". | — |
| Prozedurale Texturen | **Nur native** (raylib `GenImage*`): `GENTEX_PERLIN(w,h,skala)`, `GENTEX_GRADIENT(w,h,c1,c2,vertikal)`, `GENTEX_CHECKED(w,h,fx,fy,c1,c2)`, `GENTEX_COLOR(w,h,c)`, **`GENTEX_RADIAL(w,h,inner,outer[,density])`** (radialer Verlauf Mitte→Rand — weiche Glows/Lichter/Vignetten, additiv gezeichnet) -> IMAGE-Handle (mit `DRAWIMAGE`/`DRAWIMAGEROT` nutzbar). Demo `examples/101_blend_gentex.dh`. | — |
| Clipboard / Drag&Drop | **Nur native**: `CLIPBOARD_GET()->STRING` / `CLIPBOARD_SET(text$)` (System-Zwischenablage), `FILES_DROPPED()->INTEGER` (Anzahl gedroppter Dateien dieses Frame, unter macOS auch die vom Finder uebergebenen; seit 2026-09-19 je Bild EINMAL eingesammelt -- vorher verbrauchte der erste Aufruf die Liste) + `FILE_DROPPED(i)->STRING` (Pfad). Tree-Walker konsolen-only -> wirft "nur dhrt". **Die Zwischenablage gehoert jeweils EINEM Prozess** (2026-09-20): wer sie nicht bekommt, erfuhr davon NICHTS -- `glfwSetClipboardString` gibt void zurueck, raylib-rs' Result sagt nur, ob der Text ein Nullbyte enthielt. `CLIPBOARD_SET` liest jetzt ZURUECK und wiederholt (vier Versuche, 150 ms Pause -- gemessen deckt das ~700 ms Blockade ab und gibt nach 711 ms auf), danach ist es ein FEHLER statt Schweigen; sonst stuende beim naechsten Einfuegen der ALTE Inhalt da und der Schaden zeigte sich woanders. **Schlimmer war `CLIPBOARD_GET`: es STUERZTE AB** (0xC0000005) -- GLFW liefert NULL, und raylib-rs macht `CStr::from_ptr` darauf ungeprueft; jetzt ueber `raylib::ffi::GetClipboardText` mit NULL-Pruefung (`clipboard_text_roh`), leer ist die Antwort. Das traf JEDES Programm mit Strg+C/V. **Zwei Zahlen, die ueberraschen:** EIN `SetClipboardText` kostet gegen eine gehaltene Zwischenablage 261 ms (GLFWs drei `Sleep(1)` haengen an Windows' Timer-Granularitaet), und eine 5-ms-Pause beim Wiederholen bringt NICHTS -- die Folgeversuche laufen ins Leere. Tests `tests/pruef/zwischenablage.dhtest` (4, seriell; der Blocker haelt sie per OpenClipboard ECHT fest). **Der Test war dreimal gruen und wertlos**, bevor er etwas pruefte: mit festem Text (stand vom vorigen Lauf noch drin) und mit `MILLIS()` (= Zeit seit PROGRAMMstart, und der Ablauf dauert jedes Mal gleich lang) -- erst `UUID4$` ist eindeutig. | — |
| Render-Targets | **Nur native:** `RENDERTARGET_NEW(w,h[,behalten])->INTEGER` (Off-Screen-Render-Ziel; `behalten`=TRUE laesst den Inhalt ueber das Bild hinaus stehen -> **echte Rueckkopplung/Schweife**, `RENDERTARGET_CLEAR(rt[,farbe])` raeumt es von Hand), `RENDERTARGET_BEGIN(rt)` / `RENDERTARGET_END()` (folgende Draws ins Ziel — pro Frame transparent gecleart), `RENDERTARGET_DRAW(rt,x,y[,skala[,tint]])` (Ziel als Bild stempeln). dhrt: eigener Command-Buffer pro Target, beim FLIP vor der Hauptszene auf die RenderTexture gerendert (y-flip); Tree-Walker konsolen-only -> wirft "nur dhrt". Demo `examples/102_render_target.dh`. *Grenze:* RtDraw innerhalb eines anderen Targets = No-Op -- ein Target kann sich also auch NICHT selbst zeichnen. Schweife entstehen ueber `behalten`=TRUE plus Verblassen mit `BLEND_MODE("mult")` + Vollbild-`BOX` in dunklem Grau (Rezept + Tests: `tests/pruef/rendertarget_persistenz.dhtest`). | — |
| Zustand sichern | `GFX_PUSH()` / `GFX_POP()` — Zeichenzustand auf einen Stapel legen und zurueckholen: 2D-Kamera+Ruetteln, aktive Layer, Hintergrundfarbe, Licht (Ambient/Nebel/alle Lichtquellen), Umgebung (`LIGHT_ENV`, IBL-Schalter, `SKYBOX`), Schatten (an/Bereich/Ziel), 3D-Kamera samt View-/Projektions-Ueberschreibung, Schrift und `POSTFX`. **Nicht** enthalten: geladene Ressourcen (bleiben geladen — POP schaltet nur ihre Benutzung zurueck), die Schatten-AUFLOESUNG (haengt am allozierten Tiefenpuffer) und der Blend-Modus (ohnehin nur ein Bild lang gueltig). Analog `AUDIO_PUSH()` / `AUDIO_POP()` fuer alle Bus-Einstellungen (Lautstaerke, Balance, Filter, Hall, Echo, Verzerrer, Kompressor, EQ) — eine laufende `AUDIO_MODULATE`-Bindung wird dabei abgeloest, weil das Zurueckschreiben denselben Kira-Parameter beschreibt (empirisch belegt in `tests/pruef/gfx_push_pop.dhtest`). `GFX_DEPTH`/`AUDIO_DEPTH` liefern die Stapeltiefe, ein POP ohne PUSH ist ein Fehler. **Der Grund:** dieser Zustand ist global, und eine vergessene Ruecknahme faellt erst Szenen spaeter auf. | — |
| Fenster-Zustand | `WINDOW_FOCUSED()` (Spiel pausieren, wenn der Nutzer wegklickt), `WINDOW_MINIMIZED/MAXIMIZED/HIDDEN()`, `WINDOW_IS_FULLSCREEN()`, `WINDOW_FOCUS()` (nach vorne holen), `WINDOW_OPACITY(0..1)` (ganzes Fenster durchscheinend), **`WINDOW_ICON(bild)`** — ohne das trug jedes exportierte Spiel das raylib-Standardsymbol. `WINDOW_DPI_X/Y()` = Bildschirm-Skalierung (1.0 normal, 2.0 HiDPI/Retina — ohne sie weiss ein Programm nicht, ob seine Pixelgroessen auf dem Zielgeraet winzig herauskommen). `GET_TIME()` = monotone Sekunden seit Programmstart. `OPENURL(adresse$)` oeffnet den Standardbrowser — **bewusst auf http/https begrenzt**, weil raylib die Zeichenkette an die Shell weiterreicht und ein `file:`-Schema sonst ein Weg waere, aus einem GB-Programm Beliebiges zu starten. | — |
| Kompression | `COMPRESS$(text$)` / `DECOMPRESS$(gepackt$)` — DEFLATE, Ergebnis Base64 (GB-Strings sind UTF-8, roher Deflate-Output waere keins). Typisch ~9x kleiner bei Savegame-artigem Text; passt ueberall dorthin, wo heute schon `BASE64_ENCODE`-Ausgaben stehen. **Ungated** (miniz_oxide statt raylibs CompressData) — laeuft also auch in Konsolen-Programmen ohne Fenster. | — |
| Bild HERSTELLEN | `IMAGE_NEW(b, h [, farbe])` (**ohne Farbe vollstaendig durchsichtig** -- ueber eine FARBE ist das gar nicht auszudruecken, weil Deckkraft 0 als DECKEND gilt; `GENTEX_COLOR` kann es deshalb nicht), `IMAGE_CLEAR(bild [, x, y, b, h])` (Radierer -- SCHREIBT die Durchsichtigkeit, mischte es, waere es ein Nichts-Tun), `IMAGE_DRAW_IMAGE(ziel, quelle, x, y [, qx, qy, qb, qh] [, faerbung])` (Ebenen verrechnen, Ausschnitt einsetzen), `GETALPHA(bild, x, y)` (0..255, -1 ausserhalb -- noetig, weil `GETPIXEL` eine FARBE liefert und ein durchsichtiger Punkt dort als deckendes Schwarz ankaeme), `IMAGE_FREE(bild)` (Bild + Grafikspeicher-Textur freigeben -- gemessen 1200 Kopien zu 256x256: **393 MB gegen 91 MB**. Das Handle wird danach NICHT neu vergeben, jede weitere Benutzung meldet sich im Klartext statt still auf ein fremdes Bild zu zeigen; `GETPIXEL`/`GETALPHA` bleiben bei -1. Der Pfad-Cache von `LOADIMAGE` wird mitgeraeumt. **Weiss nichts von** Texturen, die per `MODEL_TEXTURE` an ein Modell gingen -- raylibs `Texture2D` ist ein Struct ohne Zaehlung. **Nicht anlegen ist billiger als anlegen und freigeben**), `IMAGE_SAVE_GIF(bilder, pfad$ [, fps_oder_dauern [, wiederholen [, anzahl]]])` (**bewegtes GIF** aus einem `ARRAY OF IMAGE`; raylib kann GIFs nur LESEN. **Das dritte Argument ist ZWEIERLEI:** eine Zahl sind Bilder je Sekunde fuer alle, ein FELD ist die Dauer JE BILD in Millisekunden. GIF kann das von Haus aus, und eine Bildfolge braucht es -- eine Pose wird gehalten, der Lauf dazwischen nicht. Dass die EINHEIT wechselt, ist Absicht: ein einzelnes Bild hat keine Bildrate, es hat eine Dauer; `[4, 12]` als "250 ms, dann 83 ms" zu lesen waere die schlechtere Zumutung. Zu wenige Zeiten sind ein FEHLER -- die letzte stillschweigend zu wiederholen waere eine Vermutung, und eine falsche Zeit sieht man dem GIF nicht an, man merkt sie nur. Ueber die pure-Rust-Crate `gif` -- ein LZW-Kodierer von Hand ist die Art Code, die auf den ersten Blick stimmt und im Randfall still etwas Falsches liefert. **Farbtafel EXAKT bei bis zu 255 Farben** -- ein Verfahren, das immer zusammenfasst, haette schon ein Vier-Farben-Sprite verfaelscht; darueber wird reduziert. Durchsichtigkeit nur ganz/gar nicht (Schwelle 128), EINE Leinwand fuer alle Bilder (abweichende Groesse = Fehler, nicht beschneiden), Dauer auf >= 2 Hundertstel geklemmt weil Betrachter darunter still ihre eigene nehmen. `anzahl` = wie viele Plaetze des Feldes gelten, sonst sind die leeren Plaetze eines `DIM b[16]` ein Fehler. Reine Kodier-Logik in `gifschreiber.rs` mit 8 Rust-`#[test]`s -- sie kennt raylib nicht und ist damit fuer sich pruefbar), `IMAGE_SAVE(bild, pfad$)` (png/bmp/jpg/tga; unbekannte Endung wird abgelehnt, raylib taete sonst still nichts; ob geschrieben wurde, sagt nur die Datei -- die Bindung wirft das Erfolgs-Flag weg). **Der Anlass:** ein IMAGE war eine Einbahnstrasse -- hineinzeichnen ja, aber nicht herstellen; Ton hatte seit dem SFX-Piloten `AUDIO_SAVE_WAV`, Bild gar nichts. **Nicht dabei:** Bilder ueber die Zwischenablage (raylibs `GetClipboardImage` gibt es nur unter Windows). Doku `docs/module-imgfx.md`, Demo `examples/188_bild_erzeugen.dh`, Tests `tests/pruef/image_io.dhtest`. | — |
| Bild-Verarbeitung (Ausbau) | `IMAGE_CONVOLVE(bild, kern)` — freie Faltung mit quadratischem, ungerade-seitigem Kern als flachem `ARRAY OF FLOAT` (Schaerfen, Kanten, Praegen; `IMAGE_BLUR` kann nur Gauss). `IMAGE_ALPHA_MASK/CROP/PREMULTIPLY` (weiche Raender, eng zuschneiden, dunkle Saeume beim Skalieren vermeiden). `IMAGE_DITHER(bild, r,g,b,a)` — **nur 5,6,5,0 / 5,5,5,1 / 4,4,4,4**; raylib warnt bei allem anderen bloss und liefert ein Bild mit ungueltigem Format (Textur wird schwarz), deshalb hier hart abgelehnt. `IMAGE_PALETTE(bild, max)` -> `ARRAY OF INTEGER` der haeufigsten Farben. | — |
| Textur-Generatoren (Ausbau) | `GENTEX_CELLULAR(w,h,kachel)` (Voronoi/Zellrauschen — Steinboden, Risse), `GENTEX_NOISE(w,h,anteil)` (Weissrauschen — Sternenfelder, Korn), `GENTEX_GRADIENT_BOX(w,h,dichte,c1,c2)` (rechteckiger Verlauf von innen nach aussen — Vignetten; das eckige Gegenstueck zu `GENTEX_RADIAL`). | — |
| Bitmap-Fonts | `LOADFONT_IMAGE(bild, trennfarbe, erstes_zeichen)` — Pixel-Schrift aus einem PNG, dessen Zeichen durch die Trennfarbe getrennt sind. Bleibt bewusst ungefiltert (nearest), damit Pixel-Schrift pixelig bleibt — anders als `LOADFONT` (TTF), das bilinear glaettet. `TEXT_LINE_SPACING(px)` fuer mehrzeiligen Text. **Nicht umgesetzt:** animierte GIFs (`LoadImageAnim` liefert nur Bild 0 nutzbar, raylib-rs macht `Image` readonly) und `GetClipboardImage` (Windows-only) — Begruendungen stehen im Quelltext. | — |
| Shader-Uniforms (Ausbau) | `SHADER_SET_ARRAY(sh, name$, werte)` fuellt ein `uniform float[]` aus einem `ARRAY OF FLOAT` (Lichtpositionen, Verlaufsstufen — vorher liess sich pro Aufruf nur EIN Wert setzen). `SHADER_SET_TEXTURE(sh, name$, bild)` belegt einen **zweiten Sampler** (Masken, Paletten-LUTs, Ueberblendungen). `SHADER_SET_MATRIX(sh, name$, mat)` nimmt eine `MAT4` aus `m3d`. **Wichtig:** raylibs `SetShaderValueTexture` ruft intern `glUniform1i` und wirkt damit auf das GERADE AKTIVE Programm — ausserhalb von `BeginShaderMode` landet die Zuweisung am falschen Shader und der Sampler bleibt schwarz. dhrt merkt sie deshalb vor und setzt sie beim Zeichnen (`shader_textures` in graphics.rs). | — |
| Linien-/Polygon-Geometrie | Im `physics`-Modul, pure Functions ohne Fenster: `PHYSICS_LINES_HIT` (schneiden sich zwei **Strecken**?) mit `PHYSICS_LINES_X/Y` fuer den Schnittpunkt (**NAN** wenn es keinen gibt — erst HIT fragen), `PHYSICS_POINT_LINE(px,py, ax,ay, bx,by, dicke)`, `PHYSICS_CIRCLE_LINE`, und `PHYSICS_POINT_POLY(px, py, xs, ys)` (Strahl-Verfahren, funktioniert auch bei konkaven Polygonen). | — |
| Eingabe-Flanken | **"genau in DIESEM Frame"** statt "wird gehalten": `MOUSE_HIT(n)`/`MOUSE_RELEASED(n)`, `KEYHIT(c)`/`KEYRELEASED(c)`, `KEYREPEAT(c)` (+ System-Auto-Repeat), `JOYSTICK_HIT/RELEASED(idx,btn)`. **Achtung:** `MOUSEBUTTON` und `KEYPRESSED` melden weiterhin *gehalten* — die Namen sind historisch und behalten ihre Bedeutung. Dazu `JOYSTICK_ANY_BUTTON()` (zuletzt gedrueckter Knopf, -1 = keiner — fuer Belegungsdialoge) und `JOYSTICK_AXIS_COUNT(idx)`. **Belegungsdialoge auch fuer die Tastatur:** `KEY_ANY_HIT()` (Code der zuletzt gedrueckten Taste, -1 = keine) + `KEY_NAME$(code)` (Anzeigename; GLFW kennt nur die druckbaren und die layout-abhaengig, fuer Sondertasten hat dhrt eine eigene Tabelle: `LEER`/`LINKS`/`UMSCHALT`/`F5`/…). `JOYSTICK_MAPPINGS(sdl_db$)` laedt SDL-GameControllerDB-Zeilen nach. **Neue Tastencodes** (vorher gab es dafuer GAR KEINE Konstante, „Sprint mit Umschalt" war nicht abfragbar): `KEY_LSHIFT/RSHIFT`, `KEY_LCTRL/RCTRL`, `KEY_LALT/RALT`, `KEY_LSUPER/RSUPER`, `KEY_CAPSLOCK`, `KEY_INSERT/DELETE/HOME/END/PAGEUP/PAGEDOWN`, Ziffernblock `KEY_KP0..KEY_KP9` + `KEY_KP_ENTER/PLUS/MINUS/MULTIPLY/DIVIDE/PERIOD` (in `vm.rs` DEFAULT_KEYS -- seit 2026-09-21 die einzige Liste). | — |
| Eingabe aufzeichnen/abspielen | `AUTOMATION_RECORD(datei$)` / `AUTOMATION_STOP()` (schreibt die Datei, liefert die Anzahl) / `AUTOMATION_PLAY(datei$)` + `AUTOMATION_RECORDING/PLAYING/FRAME/COUNT` — raylibs Automation-Events (Tasten/Maus/Rad/Gamepad/Touch je Frame). Fuer Demo-/Attract-Modus, nachspielbare Fehlerberichte, automatische Spieltests. Eingespeist wird in `automation_tick()` am **Ende jedes FLIP** (direkt nach dem Einlesen der echten Eingabe -> aufgezeichnete Werte gewinnen; ein Ereignis aus Aufnahme-Frame N wirkt im Durchlauf N+1). Die Liste liegt in einer **Box**, weil `SetAutomationEventList` sich einen rohen Zeiger merkt. Aufnahme und Wiedergabe schliessen sich aus (raylib spielt waehrend einer Aufnahme nichts ab -> klare Fehlermeldung). Aufgezeichnet wird die EINGABE, nicht der Ablauf: Startzustand zuruecksetzen, `RANDOMIZE` festnageln, pro Frame statt pro Sekunde rechnen. **`KEY_ANY_HIT` blendet aus, was die laufende Wiedergabe selbst einspeist** (`auto_injected_keys` in graphics.rs) -- raylib legt eingespeiste Tasten auch in seine "zuletzt gedrueckt"-Warteschlange, ohne den Filter braeche ein Attract-Modus ("Demo endet bei Tastendruck") an seiner eigenen Demo ab; `KEYHIT`/`KEYPRESSED` sehen sie weiterhin, `JOYSTICK_ANY_BUTTON` ist nicht betroffen. **Die Wiedergabe HAELT die Mausposition** (2026-09-20): raylib zeichnet eine Position NUR auf, wenn sie sich geaendert hat (`rcore.c`: "only saved if changed") -- zwischen zwei solchen Ereignissen sagt die Aufnahme "die Maus steht still", und was die Wiedergabe nicht selbst setzt, gehoert dem Rechner: schon ein Fenster, das unter dem Zeiger auftaucht oder verschwindet, schickt unter Windows ein WM_MOUSEMOVE, und GLFW schreibt damit raylibs Mausposition um. Ein aufgezeichneter Klick steht aber in drei Bildern (Position, Taste runter, Taste hoch), und ein gui-Knopf zaehlt ihn erst beim LOSLASSEN auf sich selbst -- **der Klick kam an und blieb wirkungslos**; die Bildzaehlung stimmte dabei die ganze Zeit (`automation_tick` zaehlt FLIPs, nicht Zeit). `automation_tick` spielt deshalb das zuletzt gespielte Positions-Ereignis in jedem Bild erneut ab, in dem die Aufnahme keine neue Position nennt -- dasselbe Ereignis noch einmal, weil `PlayAutomationEvent` fuer diesen Typ nur raylibs `currentPosition` schreibt und KEINEN echten Zeiger bewegt (anders als `MOUSE_SET_POS` -> `SetMousePosition`). **Das war die Ursache dafuer, dass `dhrt test tests/pruef` bei jedem Lauf ANDERE Klick-Faelle fallen liess, waehrend jeder einzeln gruen war** -- nicht die Last: bei acht gleichzeitigen Fenstern ueberdecken sie sich staendig, und jedes Auftauchen ist so ein Ereignis. Gemessen am Notenblatt-Fall "zweite spur und instrument": allein und ohne jede Last 3 von 10 Fehlschlaege, mit bewegtem Zeiger ueber dem Fenster 15 von 20, danach 0 von 20; im Fehl-Lauf standen 129 von 140 Bildern auf fremder Lage, in jedem gruenen null. `--hardware` macht keinen Unterschied (18 von 20 gegen 15 von 20 unter derselben Stoerung). Ueber vier volle Laeufe: 11 und 8 Fehl vorher, 6 und 4 nachher -- die 4 sind vorbestehende, in JEDEM Lauf fallende Faelle (zwei Buch, zwei werkzeug_paket). **Folge fuer ein Programm, das selbst `MOUSE_SET_POS` ruft:** im selben Bild gewinnt es weiter, die Lage bleibt aber nicht stehen -- sobald es aufhoert zu setzen, zieht die Aufnahme sie im naechsten Bild zurueck (gemessen). Tests `tests/pruef/automation_mausstand.dhtest`: gestoert wird mit einer ECHTEN Fensternachricht (PostMessage WM_MOUSEMOVE, Nachricht "mausweg" in `_hilfen/fenstersender.ps1`), nicht mit `MOUSE_SET_POS` -- das bewegte den Zeiger des Nutzers mit; der zweite Fall ist die Gegenprobe, dass die Nachricht ankommt. **`--- seriell` haette es nicht geheilt** -- die zweite Flake-Ursache derselben Suite (die geteilte System-Zwischenablage) traf einen Fall, der seriell markiert IST. Doku `docs/automation.md`, Demo `examples/153_automation.dh`, Tests `tests/pruef/automation.dhtest` (schreiben die Aufnahmedatei selbst — raylibs Textformat). | — |
| Maus-Blick + Cursor | `MOUSE_DELTA_X/Y()` (relative Bewegung — bei `MOUSE_LOCK` stehen MOUSEX/MOUSEY still, nur das Delta bewegt sich noch), `MOUSE_SET_POS(x,y)`, `MOUSE_ON_SCREEN()`, `MOUSEWHEEL_X/Y()` (Rad in **beiden** Achsen und als Kommazahl — `MOUSEWHEEL` liefert nur vertikal + ganzzahlig, feine Touchpad-Schritte fielen darin auf 0), `MOUSE_CURSOR(form$)` mit `default`/`ibeam`/`crosshair`/`hand`/`resize_ew`/`resize_ns`/`resize_nwse`/`resize_nesw`/`resize_all`/`not_allowed`. | — |
| Touch + Gesten | `TOUCH_COUNT()`, `TOUCH_X/Y(i)`, `TOUCH_ID(i)` (stabile Finger-Kennung ueber Frames). `GESTURE$()` liefert einen **Namen** statt einer Zahl: `tap`/`doubletap`/`hold`/`drag`/`swipe_left|right|up|down`/`pinch_in`/`pinch_out` (`""` = keine). Dazu `GESTURE_DRAG_X/Y/ANGLE`, `GESTURE_PINCH_X/Y/ANGLE`, `GESTURE_HOLD_TIME()`. Demo `examples/149_input_edges.dh`. | — |
| Game-Loop | `DELTA()` — Sekunden seit letztem `FLIP` (framerate-unabhaengige Bewegung: `x = x + speed * DELTA()`). `FPS()` / `SETFPS(n)` (Ziel-Framerate, 0 = ungedrosselt). `SET_FULLSCREEN(an)`, `SETWINDOWTITLE(s$)`, `SAVESCREENSHOT(pfad$)`. **Natives OS-Fenster** (das SCREEN-Fenster selbst): `WINDOW_RESIZABLE(an)` (vom OS aus groessenveraenderbar), `WINDOW_MIN_SIZE(w,h)`/`WINDOW_MAX_SIZE(w,h)`, `WINDOW_MAXIMIZE/MINIMIZE/RESTORE()`, `WINDOW_RESIZED()->BOOL`; `SCREENWIDTH()`/`SCREENHEIGHT()` liefern die **live**-Groesse (waechst mit dem Fenster). Demo `examples/106_windows.dh`. Nativ in dhrt (raylib). | — |
| Shader / Post-FX | **Nur native Runtime** (raylib/GPU): `SHADER_LOAD(pfad$_oder_glsl$)` -> SHADER-Handle (oder -1), `SHADER_SET(h, uniform$, f)` / `SHADER_SET2` (vec2) / `SHADER_SET3` (vec3), `POSTFX(h)` (Frame durch Fragment-Shader; -1 = aus). Szene -> RenderTexture -> Shader -> Screen. Tree-Walker konsolen-only -> wirft "nur dhrt". Beispiel-Shader `examples/assets/shaders/` (CRT/Bloom/Vignette), Demo `examples/86_postfx_shaders.dh`. | — |

Module mit eigenem Typ registrieren ihn lowercase (`register_type("json_handle", _JSONHandle)`),
GB-Code schreibt ihn in jeder Casing-Form (`DIM j AS JSON_HANDLE`).

## Convention: Wert-Typen in GB

| GB-Typ | Python-Typ | type-Spec |
|---|---|---|
| INTEGER | `int` (kein bool) | `"int"` |
| FLOAT | `float` | `"num"` (akzeptiert auch int) |
| STRING | `str` | `"str"` |
| BOOLEAN | `bool` | `"bool"` |
| Klasse / Externer Typ | Instanz / Handle | `"any"` (selbst prüfen) |
| ARRAY OF T | `_GBArray` | — (Parser-Form `array:T`) |
| MAP OF T | `_GBMap` | — (Parser-Form `map:T`) |
| FILE / IMAGE / SOUND | `_GBFile` / `_Image` / `_Sound` | — (eigene target-Strings) |
| SPRITE_ATLAS | `_SpriteAtlas` (image + frames-Dict) | — (eigener target-String `"sprite_atlas"`) |

**Mathe-Typen starten neutral:** `DIM m AS MAT4` ist die Einheitsmatrix,
`QUAT` das Einheits-Quaternion, `VEC2/3/4` der Nullvektor — nicht NIL. Genauso
wie INTEGER mit 0 und STRING mit `""` anfaengt; ein `DIM mats[N] AS MAT4` laesst
sich damit schrittweise fuellen. Eine Quelle fuer alle drei Wege (global, lokal,
Array-Element): `model::neutrales_element` (die Compiler-Konstanten koennen kein
MAT4 tragen, deshalb wird es beim Laden bzw. beim DECLARE nachgetragen).

**Bool ist KEINE Zahl** — `_check_num(True)` wirft, weil `isinstance(True, int)` zwar `True`
ist, aber `True` semantisch keine Zahl in GB ist.

## Camera-Wirkung auf Drawing

Wenn `CAMERA_SET` aufgerufen wurde, sind ab da alle Koordinaten in den
core-Grafik-Built-ins (`PLOT`, `LINE`, `BOX`, `RECT`, `CIRCLE`, `TEXT`,
`DRAWIMAGE*`, `DRAWTILEMAP`) **World-Koordinaten**. `TEXT` wird nur translatiert
(nicht gezoomt) — für scharfen HUD-Text vorher `CAMERA_RESET()`.

`PARTICLE_DRAW` ruft intern `g.circle()` und folgt der Camera automatisch.

## Arbeitsablauf: Branch + Pull Request

Nicht direkt auf `main` pushen. Fuer jede Aenderung ein Zweig, dann ein PR:

```
git switch -c thema/kurzer-name
# arbeiten, pro abgeschlossenem Punkt ein Commit
git push -u origin thema/kurzer-name
gh pr create --fill        # nutzt .github/pull_request_template.md
gh pr checks --watch       # die sechs Pflicht-Pruefungen abwarten
gh pr merge --squash       # oder --merge, je nach Umfang
```

**Warum das zaehlt:** `main` verlangt sechs Status-Pruefungen (`test (3.12)`,
`posix-test` und `rust-check` je auf ubuntu/macos/windows). Bei einem direkten
Push laufen die erst DANACH -- ein roter Bau steht dann schon auf `main`. Genau
so ist am 2026-08-29 ein macOS-Fehler auf `main` gelandet: ein Linker-Flag, das
auf Windows und Linux stillschweigend durchging und nur ld64 aufstiess. Im PR
haette die Pruefung ihn vor dem Merge abgefangen.

**Was die CI faengt, was diese Maschine nicht faengt:** Sie baut ohne Grafik,
ohne Hardware-Features und auf drei Betriebssystemen. Ein gruener Lauf hier
sagt darum wenig -- die Entwicklermaschine ist nicht nur reicher, sondern auch
nachsichtiger (der MSVC-Linker schluckt Optionen, die ld64 ablehnt).

## Build und Test

Gebraucht werden Rust (`cargo`) und ein beliebiges Python 3 für das Bauskript
(nur Standardbibliothek, kein venv):
```
python rust\build_runtime.py            # dhrt bauen (--hardware: serial/usb/bt/wifi/midi)
rust\drachenhauch_runtime\target\release\dhrt test tests\pruef    # alle Pruefsammlungen (~13 min)
rust\drachenhauch_runtime\target\release\dhrt test tests\pruef\json.dhtest --filter Text
```
`dhrt test` laeuft die Faelle einer Datei parallel (bis 8 Faeden), Dateien mit
`--- seriell` nacheinander; ohne Fenster oder Ton ueberspringt sich, was sie
braucht. Die CI fuehrt dasselbe auf Windows (mit Grafik), Linux und macOS
(ohne Grafik, `DHRT_OHNE_GRAFIK=1`) aus, dazu `cargo check`/`cargo test` je
System. `DH_OHNE_AUDIO=1` setzen, wenn der Rechner keine Soundkarte hat.
**Die Laufzeit baut NUR `rust/build_runtime.py` verlaesslich** (CFLAGS fuer die
Tipp-Warteschlange, cmake/libclang-Suche); `cargo build --bin dhrt` allein kann
veraltete Ergebnisse liefern.

**Headless prüfen:** `DHRT_FRAMES=n DHRT_SCREENSHOT=p.png dhrt run x.dh` liefert
EIN Bild (ein Augenblick). Für alles, was sich über die ZEIT falsch verhält
(zu früh umkippen, stehenbleibende Ränder, ruckelnde Bewegung) stattdessen den
**Kontaktbogen**: `DHRT_FRAMES=480 DHRT_CONTACT=bogen.png dhrt run x.dh` setzt
mehrere Bilder beschriftet als Raster in eine PNG (`DHRT_CONTACT_MAX`,
`_COLS`, `_EVERY`). Aus einem Programm heraus: `dhrt bild quelle.dh ziel.png [bilder]`.
Details: docs/rust-runtime.md.

## Häufige Fallstricke

- **Grafik/Audio brauchen ein Fenster bzw. eine Soundkarte:** Konsolen-Programme
  (PRINT/INPUT/Logik) laufen ueberall; ohne Bildschirm ist `SCREEN` ein
  abfangbarer Fehler ("Kein Fenster moeglich").
- **Escape-Folgen nur mit `!`** (seit 2026-09-08): `!"Zeile 1\nZeile 2"`,
  `!"Sie sagte \"Hallo\""`, `f!"Punkte: {p}\n"`. In einer NORMALEN
  Zeichenkette bleibt der Backslash woertlich -- gemessen stand er im Bestand
  63-mal in Zeichenketten, fast immer als Regex (`"\d+"`, `"\s+"`) oder
  Windows-Pfad, und aus `"assets\tiles.png"` waere still ein Tabulator
  geworden. Opt-in wie in FreeBASIC (`!"..."`). Erlaubt: `\n \t \r \\ \"
  \0 \e \uXXXX`; jede andere Folge ist ein Uebersetzungsfehler (kein stilles
  Weglassen). Beide Lexer (`lexer.rs` `scan_escape`, `lexer.py`
  `_scan_escape`), Hervorheber (`syntax.rs`, `highlighter.py`) und die
  VS-Code-Grammatik kennen es; Paritaets-Schnipsel in
  tests/test_rust_lexer_parity.py, Sammlung `tests/pruef/zeichenketten_escape.dhtest`,
  Doku `docs/sprache.md` (Strings).
- **Umstieg aus anderen BASICs** (seit 2026-09-21): `DIM x AS T = wert`,
  `EXIT FOR/DO/WHILE/SUB`, `END` allein, `END WHILE`, `SWAP`, `LET`, `?`,
  `INPUT "x"; v`, `m["k"]`, `LEN(map)`, Feld `+` Feld, ein Name allein ruft
  die SUB auf, `DIM a AS ARRAY OF T` ist LEER statt NIL, `VAL` liest die Zahl
  am Anfang samt `&HFF`. Fuer GOTO/REDIM/TYPE/`x%`/`!=`/UCASE$/DOUBLE ...
  sagt die Meldung, wie es hier heisst (`umstieg.rs`,
  `Parser::umsteiger_hinweis`). `swap`/`let`/`exit`/`do` sind KONTEXTUELL,
  keine Schluesselwoerter. Uebersicht `docs/umstieg.md`, Details
  `docs/stolpersteine.md` I, Tests `tests/pruef/umstieg.dhtest`.
  Fuenfte Runde (2026-09-23): **Zeichnen ohne SCREEN tat still nichts**
  (jetzt ein Fehler, `vm::ist_schirmbefehl`), **gezeichnet ohne FLIP** sagt am
  Programmende einen Satz, ein **unbekannter Befehl schlaegt den echten Namen
  vor** (`aehnlich.rs` -- Abstand UND Wortteile, eine Quelle auch fuer
  Variablen- und Mitglieds-Vorschlaege), ein **Dateiname statt eines
  geladenen Bildes/Klangs** nennt den Lader (`umstieg::text_statt_zahl`, dazu
  `KEYHIT("a")` und `RGB("FF0000")`), die Lader melden eine fehlende Datei
  gleich (`builtins::datei_da`), `DIM p AS Klasse` ohne NEW und `DIM f AS
  FILE` ohne OPENFILE sagen, was fehlt.
  Dritte/vierte Runde (2026-09-22): eine METHODE ohne Klammern
  (`spieler.springen`) tat still nichts; `JSON_PARSE("{\"a\": 1}")` meldete
  nur "Erwartet Rparen" (jetzt Hinweis auf `!"..."`); 61 der 62 Signaturen im
  `builtin_index.json` tragen jetzt Parameternamen statt "N..M Argumente"
  (Wache in `doku_pruefungen.dhtest`; dabei fiel auf, dass
  `ATLAS_DRAW_FLIPPED` mit 7 statt 6 Argumenten laeuft), und eine
  Argument-Fehlermeldung nennt die Form des Befehls (`vm::mit_signatur`).
  Zweite Runde (2026-09-22): `STEP 0` stuerzte dhrt ab (jetzt Fehler),
  `EOF(f)` neu (READLINE liefert am Ende `""` wie eine leere Zeile),
  `"5" = 5`/`TRUE = 1` warnen, `READALL$(pfad)`, `CALL`, `BYVAL`.
- **`step` ist Schlüsselwort** (FOR…STEP). Variablen entsprechend benennen
  (`i`, `iter`, `tick` statt `step`).
- **Vorbelegte Namen als Variable gehen ueberall** (seit 2026-08-31). `DIM red`,
  `DIM pi`, `DIM key_space` verschatten die eingebaute Konstante -- vorher ging
  das NUR auf oberster Ebene, in einem `IF`/`WHILE`/`FOR` warf dieselbe Zeile
  zur Laufzeit "CONST 'red' kann nicht ueberschrieben werden" (und `--check`
  schwieg). Ursache: `collect_globals` (compiler.rs) lief nur ueber die
  oberste Anweisungsliste, ein `DIM` im Block bekam keinen Slot und lief ueber
  den NAMEN -- `DECLARE_NAME` laesst einen vorhandenen Eintrag stehen,
  `DECLARE_GLOBAL_SLOT` ersetzt ihn. Der Durchlauf steigt jetzt in die Bloecke
  hinab (`globale_unterbloecke`), NICHT in SUB/FUNCTION/CLASS -- die haben
  eigene Plaetze. Nebenertrag: die Kollisions-Erkennung sah bis dahin nur
  Geschwister und liess `CONST Modus` oben + `DIM modus` im Block durch.
  Doku `docs/stolpersteine.md` H1, Tests `tests/pruef/name_collision.dhtest`.
- **Eine Variable darf heissen wie ein Builtin, und der Aufruf meint den
  Builtin** (seit 2026-09-04): `DIM deg AS FLOAT : deg = DEG(w)`,
  `len = LEN(s)`. Vorher lief jeder Variablenname vor einer Klammer ueber
  `CALL_VALUE` (den FUNCREF-Weg) und brach zur Laufzeit ab -- `--check`
  schwieg. Regel in `expr_call` (compiler.rs): angesagter Typ bekannt und
  kein FUNCREF + Builtin dieses Namens vorhanden -> Builtin; eine
  FUNCREF-Variable ruft weiter die Variable (und der Sonderfall "Typ
  unbekannt", zweimal verschieden deklariert); die FOR-EACH-Laufvariable
  laeuft gar nicht ueber diesen Weg und verdeckt nicht.
  Doku `docs/stolpersteine.md` H3, Tests `tests/pruef/variable_wie_builtin.dhtest`.
- **Tastencodes: Buchstaben gehen in BEIDER Schreibweise.** `KEY_A`..`KEY_Z`
  (= 97..122, SDL-Zaehlung) ist die klare Form; `ASC("s")` und `ASC("S")`
  meinen seit 2026-08-31 dieselbe Taste. Vorher galten nur die kleinen, und
  `KEYHIT(ASC("S"))` traf STILL gar nichts -- daran waren alle Tastenkuerzel
  des Tilemap-Editors tot, ohne Fehler oder Warnung. Ein totes Kuerzel sieht
  aus wie ein vergessener Aufruf, man sucht den Fehler also im eigenen
  Programm. Umsetzung in `map_key` (graphics.rs), Test
  `tests/pruef/automation.dhtest`
  (der hielt vorher das GEGENTEIL fest -- er nannte es in seiner eigenen
  Beschreibung schon einen "Fehler, der beim Schreiben nicht auffaellt").
- **`dhrt --check` meldet seit 2026-09-03 auch Tippfehler in Variablennamen.**
  `DIM zaehler` und spaeter `zaehlr = zaehlr + 1` ist zur Laufzeit ein
  Abbruch ("Variable nicht deklariert") -- aber eben erst, wenn die ZEILE
  laeuft; in einer selten genommenen SUB bleibt das beliebig lange still.
  Gefunden beim Sprite-Piloten, wo eine SUB eine Variable aus einem
  SCHWESTER-Programm ansprach. **Seit 2026-09-04 mit Gueltigkeitsbereich**
  (vorher stand hier "bewusst grobkoernig: geprueft wird gegen alle Namen, die
  IRGENDWO im Programm deklariert werden"). Die Locals einer Funktion landeten
  im GLOBALEN Satz -- damit war die Warnung genau dort blind, wo ein
  Tippfehler am ehesten unbemerkt bleibt: `punkte` in einer SUB ging durch,
  weil eine ANDERE SUB eine Variable dieses Namens hat. Jetzt bekommt jede
  SUB/FUNCTION/Methode einen eigenen Bereich (`lokale_namen`, Platz 0 = das
  Hauptprogramm), und `bekannte_namen` haelt nur noch, was zur Laufzeit
  wirklich global ist -- Top-Level-`DIM`/`FOR`/`FOR EACH`/`CATCH`/`ENUM`, und
  `CONST`, das auch in einer SUB global wird. **Zwei Meldungen statt einer:**
  gibt es den Namen woanders im Programm, sagt sie das ausdruecklich ("ist an
  dieser Stelle nicht sichtbar") statt "nirgends angelegt" -- wer ihn vor sich
  im Quelltext stehen sieht, sucht sonst lange nach einem Tippfehler, den es
  nicht gibt. Bewusst weiter still: eine Zuweisung, die im SELBEN
  Unterprogramm VOR ihrem `DIM` steht (gemessen faengt diese Ausnahme nichts
  weg -- sie ist das Netz unter der Umstellung, weil der Compiler linear
  uebersetzt). **Falle beim Nachziehen:** Sweep und Test filtern auf den
  Meldungstext, und der Marker muss ein Satz sein, den BEIDE Fassungen tragen
  ("Beim Laufen bricht diese Zeile ab") -- sonst uebersieht die Messung die
  halbe Warnung, und zwar stillschweigend. Ausgenommen sind die
  vorbelegten Konstanten (`vm::ist_vorbelegter_name` -- Farben, KEY_*, pi,
  tau), die absichtlich keinen Slot haben und ueber denselben LOAD_NAME-Weg
  laufen. **Der Beleg ist nicht der Test, sondern der Lauf ueber ALLES:** 384
  `.dh`-Dateien im Repo, 0 Meldungen. Die erste Fassung hatte 243 -- sie
  uebersah `DIM x[N] AS T` (eigener Compiler-Zweig), deshalb wird der Name
  jetzt EINMAL oben in `stmt_dim` gemerkt statt in fuenf Zweigen. Der zweite
  Fehlalarm (`CONST` INNERHALB einer SUB) tauchte erst am Buch-Beispiel
  `tippspiel.dh` auf, nachdem die 208 examples sauber waren.
  Dieser Lauf ist inzwischen ein TEST
  (`tests/pruef/dhrt_check.dhtest`, Fall "kein programm des repos meldet einen
  unbekannten namen")
  -- von Hand gelaufen faengt er den naechsten uebersehenen DIM-Zweig nicht.
  **Drei Wege enden in dem Rueckfall, nicht zwei:** `load_var`/`store_var` --
  und `INPUT x` (emittiert INPUT_NAME) sowie das `READ`-Ziel (geht nicht ueber
  `store_var`, es braucht den Zwischenspeicher fuer Feld-/Index-Ziele). Beide
  liefen anfangs an der Erfassung vorbei und blieben still, obwohl die VM ihr
  Ziel in genau demselben Verzeichnis sucht.
  Tests `tests/pruef/check_unbekannte_namen.dhtest`.
- **Neue Builtins/Sprach-Features NUR in dhrt** (`rust/drachenhauch_runtime/src/`):
  Builtin → `builtins.rs`/`vm.rs`; Sprach-Feature → `lexer.rs`/`parser.rs`/
  `ast.rs`/`compiler.rs`/`vm.rs`. Es gibt KEINE „beide Pfade"/Tree-Walker-Parität
  mehr — Korrektheit per Fall in einer **Pruefsammlung** (`tests/pruef/*.dhtest`)
  + ggf. Rust-`#[test]`. Bei neuem Keyword die VSCode-Grammatik regenerieren.
- **`IS NIL`/`IS NOT NIL` gibt es seit 2026-08-26** (vorher stand hier, es gebe
  sie nicht) — zusammen mit dem allgemeinen Typtest `x IS Typname`, siehe
  Abschnitt „Laufzeit-Typtest". `IS_NIL(x)` bleibt gleichwertig.

## Coroutines / YIELD

Eine `FUNCTION`/`SUB`, deren Body ein `YIELD` enthaelt, ist eine **Coroutine**.
Ihr Aufruf fuehrt den Body NICHT aus, sondern liefert ein `COROUTINE`-Handle.

```basic
FUNCTION zaehler() AS INTEGER
    YIELD 1
    YIELD 2
    RETURN 99            ' Endwert (optional), via CORO_RESULT abrufbar
END FUNCTION

DIM c AS COROUTINE
c = zaehler()
PRINT CORO_RESUME(c)     ' 1
PRINT CORO_RESUME(c)     ' 2
PRINT CORO_RESUME(c)     ' 99 (beendet) -- CORO_DONE(c) ist jetzt TRUE
```

**API** (Builtins, kein neuer Opcode ausser `YIELD_VALUE`):
- `CORO_RESUME(c)` -- fortsetzen bis zum naechsten YIELD, liefert den YIELD-Wert
  (bzw. den RETURN-Wert, wenn die Coroutine in diesem Schritt endet).
- `CORO_SEND(c, v)` -- wie RESUME, aber der `YIELD`-**Ausdruck** im Body
  evaluiert zu `v`: `DIM x AS INTEGER : x = YIELD 5`. Der Sende-Wert des
  ERSTEN Resume ist immer NIL (wie Python -- nicht lesen).
- `CORO_DONE(c)` -- BOOLEAN, ob beendet (RETURN/Ende/CLOSE).
- `CORO_RESULT(c)` -- finaler RETURN-Wert (wirft, wenn noch nicht beendet).
- `CORO_CLOSE(c)` -- suspendierte Coroutine abbauen (raeumt den Worker-Thread).
- `FOR EACH v IN coro` / Comprehensions -- treiben die Coroutine **eager** bis
  zum Ende (RETURN-Wert nicht enthalten). Vorsicht bei unendlichen Generatoren
  -- dort `CORO_RESUME`/`CORO_DONE` manuell verwenden.

**`YIELD` ist ein Ausdruck** (niedrige Praezedenz): `YIELD v` als Statement
verwirft den Sende-Wert; `x = YIELD v` liest ihn. Operand optional (`YIELD`).

**Mechanismus:** Jede Coroutine laeuft auf einem eigenen **Daemon-Thread** mit
striktem Ping-Pong (Queues; immer nur ein Thread laeuft gleichzeitig). Dadurch
bleibt die Ausgabe deterministisch und **bit-identisch** ueber Tree-Walker,
Python-VM und Cython-VM. Folgen davon:
- **Kein Cross-Frame-Yield:** ein Helfer mit `YIELD` ist selbst eine Coroutine
  (sein Aufruf liefert ein Handle), `YIELD` laeuft also nie ueber einen
  normalen Call hinweg.
- **Typ-Coercion:** in `FUNCTION ... AS T` werden YIELD- UND RETURN-Werte auf
  `T` gecoerct (ein Typ fuer beide Kanaele). SUB-Coroutinen yielden "any".
- **Idiom:** ein manueller `WHILE NOT CORO_DONE(c)`-Resume-Loop bekommt beim
  letzten (beendenden) Aufruf den RETURN-Wert; gibt der Generator einen
  typisierten `RETURN` zurueck, klappt die Zuweisung an eine typisierte
  Variable -- sonst `FOR EACH` nutzen.

**Implementierung:** in dhrt -- der Compiler markiert eine Funktion mit `YIELD`
als Coroutine, die VM legt beim YIELD einen Frame-Schnappschuss in einem
`Value::Coroutine` ab (`vm.rs`), getrieben wird ueber die `CORO_*`-Builtins.
Der frueher hier beschriebene Thread-Aufbau gehoerte zum Python-Tree-Walker
und ist mit Stufe B entfallen; die Frame-Schnappschuss-Fassung steht unten.

**Auch nativ (dhrt/Rust, `--native` + Standalone-`.exe`).** Statt Threads nutzt
die Rust-VM einen **Frame-Snapshot**: `dispatch` liefert `Step::Return | Yield`;
bei YIELD wird der Frame (ip/locals/stack/try_handlers) in einem
`Value::Coroutine` (`CoroState`) abgelegt und beim Resume restauriert. Moeglich
ist das, weil **kein Cross-Frame-YIELD** existiert -- nur der oberste
Coroutine-Frame muss fortsetzbar sein (verschachtelte Calls laufen normal
rekursiv). Kein OS-Thread -> raylib-Main-Thread bleibt sicher, deterministisch
per Konstruktion. Tree-Walker (thread-basiert) und dhrt (Frame-Snapshot) liefern
bit-identisch.

Use-Cases: Cutscene-DSL, prozedurale Generation, Boss-Patterns, NPC-Dialoge.
Doku-Demo [examples/98_coroutines.dh](examples/98_coroutines.dh), Tests
`tests/pruef/coroutines.dhtest` (Pruefsammlung fuer `dhrt test`, seit 2026-09-07 -- vorher tests/pruef/coroutines.dhtest).

## Input-Mapping (Modul `input`)

Statt hardcoded Keycodes ueberall (`KEYPRESSED(1073741904)`), bindet das
input-Modul Tastenkombinationen an benannte Actions:

```basic
IMPORT "input"
INPUT_BIND("move_left",  KEY_LEFT,  KEY_A)
INPUT_BIND("jump",       KEY_SPACE, KEY_W)

' --- Pro Frame ---
INPUT_UPDATE()                          ' Snapshot
IF INPUT_PRESSED("jump") THEN ...       ' Edge: gerade JETZT
IF INPUT_HELD("move_left") THEN ...     ' Held: dauerhaft
PRINT INPUT_AXIS("move_left", "move_right")   ' -1, 0, +1
```

**Edge-Detection** funktioniert ueber den `INPUT_UPDATE()`-Call am
Frame-Start: das Modul vergleicht den aktuellen Snapshot mit dem
vorigen. Ohne UPDATE bleiben PRESSED/RELEASED bei FALSE haengen.

**Multi-Key-Bindings:** Eine Action kann an N Tasten gebunden sein --
trifft sobald irgendeine davon gedrueckt ist.

**Action-Namen** sind case-insensitive (lower-case-Vergleich). Re-BIND
ueberschreibt die alte Liste.

**Beispiel:** [examples/59_input.dh](examples/59_input.dh).

## VEC2 + Operator-Overloading

Im `vec2`-Modul liefert `VEC2_NEW(x, y)` einen immutable 2D-Vektor.
Die arithmetischen Operatoren `+`, `-`, `*` (Skalar), `/`, `=` und `<>`
sind fuer VEC2 ueberladen:

```basic
IMPORT "vec2"
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(3.0, 4.0)
w = VEC2_NEW(1.0, 2.0)
PRINT v + w           ' Vec2(4.0, 6.0)
PRINT v * 2.0         ' Vec2(6.0, 8.0)
PRINT VEC2_LENGTH(v)  ' 5.0
```

**Operator-Hooks:** Vec2 registriert seine Operatoren ueber die
**Operator-Registry** (siehe Abschnitt unten) -- der Dispatch ist NICHT mehr
hardcoded. Generisches User-Operator-Overloading auf beliebigen Klassen gibt
es ebenfalls (`OPERATOR + (...) END OPERATOR`, siehe „Operator-Overloading auf
User-Klassen"). Wer einen weiteren mathematischen Wert-Typ als Modul einbaut,
ruft `register_operators(Typ, {...})` in seinem Modul -- ohne Eingriff in
interpreter.py / vm.py / vm_native.pyx.

**Werte sind immutable** -- `w = v` aliased nicht; jede Operation erzeugt
ein neues VEC2.

**Beispiel:** [examples/58_vec2.dh](examples/58_vec2.dh).

## Drei Halbheiten geschlossen: MAP-Literal, `FOR EACH k, v`, `DO ... LOOP`

```basic
DIM m AS MAP OF INTEGER
m = {"a": 1, "b": 2}          ' MAP-Literal, Gegenstueck zu [1, 2]
m = {}                        ' die leere MAP

FOR EACH k, v IN m            ' Schluessel UND Wert
    PRINT k; "="; v
NEXT

DO WHILE i < 5 : i = i + 1 : LOOP
DO : i = i + 1 : LOOP UNTIL i >= 5
```

- **MAP-Literal**: der Parser trennt am `FOR` -- `{k: v FOR ...}` bleibt die
  Dict-Comprehension, alles andere ist das Literal. Kompiliert ueber dieselbe
  Sammelstelle (`__dict_from_pairs`), also KEIN neuer Opcode. `{}` war vorher
  ein Parser-Fehler und ist jetzt die leere MAP (Test entsprechend gedreht).
- **`FOR EACH k, v`**: `ForEach` hat ein zweites, optionales Feld `var2`. Bei
  zwei Variablen schiebt der Compiler den Behaelter durch das interne
  `__paare` -- eine MAP liefert dann ihre Paare statt der Schluessel, alles
  andere geht unveraendert durch (ein TUPLE aus 2-Tupeln passt also auch).
  **Die Einzelvariablen-Form bleibt bei den SCHLUESSELN** -- sie umzudeuten
  haette bestehenden Code gebrochen.
- **`DO ... LOOP`**: rein im Parser, ohne AST-Knoten -- Kopfpruefung wird zu
  `While`, Fusspruefung zu `Repeat`. Damit erben beide BREAK/CONTINUE ohne
  Zutun. **`do` und `loop` sind KONTEXTUELL** (wie `each`), keine
  Schluesselwoerter: `DIM dO AS INTEGER` gibt es in `examples/127_filedialog.dh`,
  und ein neues Keyword haette das gebrochen. `DO` zaehlt nur als Schleife,
  wenn WHILE/UNTIL/Zeilenende folgt. Bedingung oben UND unten ist ein Fehler.
- Tests: `tests/pruef/sprach_symmetrie.dhtest` (18 Faelle, `dhrt test`; bis 2026-09-07 tests/pruef/sprach_symmetrie.dhtest).

## Laufzeit-Typtest: `IS` + `TYPEOF`

`TYPEOF` liefert bei einer Instanz den **Klassennamen** (gross), nicht mehr
das pauschale `"OBJECT"`; `x IS Typname` prueft samt Vererbungskette:

```basic
DIM t AS Tier
t = NEW Hund()
PRINT TYPEOF(t)      ' "HUND"
PRINT t IS Hund      ' TRUE
PRINT t IS Tier      ' TRUE   -- jede Elternklasse trifft
PRINT t IS NOT NIL   ' TRUE
```

- Rechts von `IS` steht ein **Typname, kein Ausdruck**: Klasse, Werttyp,
  Modultyp, `NIL`, `ARRAY`, `MAP`. Ein unbekannter Name ist ein
  **Uebersetzungsfehler** -- ein Tippfehler waere sonst still fuer immer FALSE.
- `NIL` ist keine Instanz: `t IS Tier` ist FALSE, solange `t` NIL ist.
- **`CASE IS > 5` bleibt unberuehrt** -- dort verschluckt `case_match` sein
  eigenes fuehrendes `IS`, bevor die Vergleichsebene ueberhaupt drankommt.
- Umsetzung: eigener AST-Knoten `IsTyp { wert, typ }` (parser.rs) -- der
  Typname liegt in einem EIGENEN FELD, damit `namensraum.rs` ihn wie jeden
  anderen Typnamen umschreiben kann (`x IS mathe.Punkt`). Der Compiler prueft
  den Namen und emittiert `__is_typ(wert, "name")`; die Vererbungskette laeuft
  `Vm::try_typtest` (vm.rs) ab -- die kennt nur die VM, darum nicht in
  builtins.rs. Kein neuer Opcode. Tests: `tests/pruef/typtest.dhtest`.

## Function References (FUNCREF)

User-Functions als first-class Werte fuer Higher-Order-Patterns:

```basic
FUNCTION square(x AS INTEGER) AS INTEGER
    RETURN x * x
END FUNCTION

DIM f AS FUNCREF
f = square            ' bare Identifier wird zur FUNCREF
PRINT f(5)            ' 25 -- Aufruf via Variable
```

Use-Cases: Sort-Comparator, Tween-Easing-Callbacks, Event-Handler.

```basic
FUNCTION twice(g AS FUNCREF, x AS INTEGER) AS INTEGER
    RETURN g(g(x))
END FUNCTION
```

**Gebundene Methoden** (2026-08-26): `obj.methode` OHNE Klammern ist eine
FUNCREF, die ihre Instanz mittraegt -- damit koennen die Rueckruf-Schnittstellen
auf ein Objekt zeigen statt auf eine freie SUB plus globale Variable:

```basic
DIM f AS FUNCREF
f = spieler.tick          ' traegt `spieler` mit
f()                       ' ruft spieler.tick() auf
GUI_ON_CLICK(knopf, spieler.klick)
TIMER_EVERY(500, gegner.zucken)
SORT(zahlen, regel.cmp)
```

Nach aussen ist das ein ganz normales FUNCREF (`TYPEOF` sagt `FUNCREF`,
`DIM f AS FUNCREF` nimmt es auf). Details:
- **Aufgeloest wird beim AUFRUF, nicht beim Binden** -- eine als Elternklasse
  gehaltene Instanz ruft die Ueberschreibung des Kindes.
- **Ein Feld gewinnt vor einer gleichnamigen Methode** (sonst haette die
  Erweiterung bestehenden Code umgedeutet); erst wenn kein Feld passt, wird
  nach einer Methode gesucht.
- **Gleichheit** vergleicht Instanz UND Methode; eine gebundene Methode ist nie
  gleich einer freien Funktion desselben Namens.
- **`GUI_SAVE`/`GUI_TO_JSON` schreiben einen gebundenen Handler NICHT** in die
  `.dhform` -- beim Laden gibt es die Instanz nicht, und nur den Namen zu
  schreiben waere eine Luege (er wuerde als freie Funktion gedeutet).
- **`TASK_START` lehnt sie ab**: der Auftrag laeuft als eigener Prozess und
  sieht das Objekt nicht -- das sagt eine eigene Meldung im Klartext.
- Implementierung: `Value::BoundMethod(Rc<(Empfaenger, Methodenname)>)` in
  `value.rs`; erzeugt im `LOAD_MEMBER`-Fallback, ausgefuehrt in `CALL_VALUE`
  (beides `vm.rs`). Was `gui`/`timer` speichern, ist ein `Rueckruf`
  (`value.rs`): Name **und** optionaler Empfaenger -- der Name muss bleiben,
  weil `gui.rs` ihn in die `.dhform` schreibt. Gefeuert wird ueber den einen
  Helfer `Vm::rueckruf_rufen`. Tests: `tests/pruef/gebundene_methoden.dhtest`
  (inkl. echtem GUI-Klick per Automation-Wiedergabe).

**Anonyme Funktionen/Closures gibt es NICHT** -- der Body einer FUNCREF sieht
nur Parameter und Globals/CONST. Wer Zustand braucht, nimmt eine gebundene
Methode (oben) oder uebergibt die Werte explizit als Parameter.

**Implementierung:**
- Type-Token `FUNCREF`. AST braucht keinen neuen Node -- bare Identifier
  in Expression-Position wird kontextabhaengig aufgeloest.
- Tree-Walker: `_eval_Identifier` liefert `_FuncRef(name)` wenn der Name
  keine Variable ist und in `self.functions` existiert. `_eval_Call`
  dispatched FuncRef-callees direkt.
- Compiler: `_global_vars`-Set wird vor Phase 5 gefuellt (alle Top-Level
  DIM/CONST/MultiDim). `_expr_Identifier` und `_expr_Call` checken Locals,
  Felder, Globals zuerst -- nur wenn keiner trifft, fallen sie auf
  Function-Lookup zurueck (Tree-Walker-konsistente Vorrang-Reihenfolge).
  Eine User-Variable mit gleichem Namen wie eine Function verschattet
  diese.
- Bytecode: `LOAD_FUNCREF` (53) und `CALL_VALUE` (54). LOAD_FUNCREF nimmt
  den Function-Namen aus dem const-Pool und pusht eine `_FuncRef`-Instanz.
  CALL_VALUE pop n args, pop callee (FuncRef), dispatch via `_exec`.
- VM-Pfade: gleicher Flow. `_FuncRef` aus `drachenhauch.interpreter` importiert.

**Einschraenkung Reihenfolge:** Im VM-Pfad wird `_global_vars` static aus
allen Top-Level-Statements gefuellt. Wer eine Function `foo` erst aufruft
und DANACH eine Variable `foo` deklariert, kriegt im VM einen
"Variable nicht deklariert"-Fehler bei der Function-Verwendung -- der
Tree-Walker ist hier dynamischer. Praxisrelevant ist das selten;
empfohlen: User-Variablen oben deklarieren oder anders benennen.

**Beispiel:** [examples/57_funcref.dh](examples/57_funcref.dh).

## Static Class Members

`STATIC CONST` innerhalb einer Klasse erzeugt klassen-bezogene Konstanten,
zugreifbar via `<ClassName>.<MEMBER>` -- analog zum ENUM-Pattern.

```basic
CLASS Player
    STATIC CONST MAX_HP AS INTEGER = 100
    STATIC CONST DEFAULT_NAME AS STRING = "Hero"
    DIM hp AS INTEGER
    SUB Init()
        Self.hp = Player.MAX_HP        ' Static aus Methode
    END SUB
END CLASS

PRINT Player.MAX_HP                     ' 100
```

**Werte muessen Compile-Zeit-Literale sein:** Number, String, Bool oder
negierte Number. Keine Ausdruecke -- gleiche Strenge wie ENUM-Member,
damit alle drei Pfade konsistent sind.

**Implementierung:** Klassen-Statics werden zur Klassen-Hoisting-Phase als
`_ClassStaticNamespace` gebaut und unter dem Klassen-Namen als globale
CONST registriert. MemberAccess auf das Namespace-Objekt liefert den
Member-Wert -- LOAD_MEMBER in beiden VMs erkennt den Typ explizit.

`_infer_type` bekommt `"class_static"` als neuen Type-String. `Self.hp =
Player.MAX_HP` funktioniert, weil `Player` als CONST-Variable im globalen
Scope existiert (mit dem Namespace als Wert), nicht als Klassen-Konstruktor
-- der NEW-Pfad geht ueber `self.classes[name]`, nicht ueber Identifier-
Lookup.

**Einschraenkungen:**
- Klassen mit Statics duerfen nicht den gleichen Namen wie eine andere
  globale Variable haben. (Klassen ohne Statics sind unbeeinflusst.)
- Static-Werte sind immutable nach Compile-Zeit -- es gibt kein
  `Player.MAX_HP = 200`. Wer Mutable-Class-State will, schreibt eine globale
  Variable mit "DIM ... AS Klasse".

**Beispiel:** [examples/56_static.dh](examples/56_static.dh).

## Properties (PROPERTY GET/SET)

Klassen koennen Property-Accessors deklarieren -- Member-Read und -Write
laufen dann durch User-Code statt direkt aufs Feld:

```basic
CLASS Player
    DIM _hp AS INTEGER

    PROPERTY GET hp() AS INTEGER
        RETURN Self._hp
    END PROPERTY

    PROPERTY SET hp(value AS INTEGER)
        IF value < 0 THEN value = 0
        IF value > 100 THEN value = 100
        Self._hp = value
    END PROPERTY
END CLASS

DIM p AS Player
p = NEW Player()
p.hp = 200      ' Setter laeuft -> clamped zu 100
PRINT p.hp      ' Getter laeuft -> 100
```

**Implementation:** Properties werden intern als Methoden mit den Namen
`__get_<name>` und `__set_<name>` registriert. Die Klasse merkt sich die
Property-Namen in einem `set` (`_ClassInfo.properties` /
`VMClassInfo.properties`). Im MemberAccess- und MemberAssign-Pfad wird
zuerst gegen das Property-Set geprueft, bei Treffer dispatcht der Code
zur Internal-Methode.

**Read-only / Write-only:** Wenn nur GET deklariert ist, wirft `obj.x = v`
einen Fehler. Wenn nur SET, wirft `PRINT obj.x`.

**Inheritance:** Properties werden automatisch vererbt (gleicher MRO-
Lookup wie Methoden).

**`GET`/`SET` sind keine Keywords:** Sie sind kontext-abhaengig nach
`PROPERTY`. So bleiben User-Methoden wie `FUNCTION Get() AS T` unbeeinflusst.

**Beispiel:** [examples/63_props_comp.dh](examples/63_props_comp.dh).

## List-Comprehensions

`[expr FOR var IN container]` und `[expr FOR var IN container WHERE filter]`.
Liefert ein TUPLE der transformierten Werte:

```basic
DIM evens AS TUPLE
evens = [n FOR n IN nums WHERE n MOD 2 = 0]

' Auch mit Method-Calls und Properties
DIM names AS TUPLE
names = [it.name FOR it IN cart WHERE it.price > 5]
```

**Iterable:** STRING (chars), TUPLE, 1D-ARRAY, MAP (Keys).

**Implementation:**
- AST `ListComp(var, iterable, filter, transform)`. Im Parser an LBRACKET
  in Primary-Position.
- Tree-Walker iteriert direkt ueber Python-Iterable.
- Compiler nutzt einen Marker-Singleton (`bytecode.COMP_MARKER`) und einen
  index-basierten Loop. Vor dem Loop wird der iterable durch das Built-in
  `__COMP_ITER` in ein TUPLE umgewandelt -- so funktioniert der gleiche
  LEN+Index-Mechanismus fuer alle Container-Typen.
- Neuer Op `BUILD_TUPLE_DYN=57`: sammelt alle Werte oberhalb des Markers
  zu einem Tupel.
- Iter-Variable wird als anonymer Local-Slot reserviert -- ueberlappt
  nicht mit existierenden globalen Variablen gleichen Namens.

**Bonus:** Strings haben jetzt auch normalen Index-Access (`s[0]`) -- nicht
nur Slicing. Das war fuer die Comprehension noetig und ist eine sinnvolle
generelle Erweiterung.

**Beispiel:** [examples/63_props_comp.dh](examples/63_props_comp.dh).

## SELECT CASE mit Guards

`CASE ... WHERE expr` -- der naechste Case wird probiert, wenn entweder das
Match-Pattern nicht trifft ODER der Guard-Ausdruck falsy ist:

```basic
SELECT CASE hp
    CASE IS <= 0
        ' tot
    CASE IS <= 30 WHERE has_potion
        ' low aber Trank verfuegbar -> heilen
    CASE IS <= 30
        ' low ohne Trank -> fliehen
    CASE ELSE
        ' OK
END SELECT
```

Guard-Expressions koennen auf normale Variablen zugreifen, inkl. der
Subject-Variable. Klassische Use-Cases:
- Permission-Checks: `CASE "delete" WHERE user IN ("admin", "moderator")`
- Numerische Conditions: `CASE 1 TO 100 WHERE n MOD 2 = 0`
- State-Combinationen: `CASE "save" WHERE dirty`

**Implementierung:** Compile-Zeit-Erweiterung von `_stmt_Select`. Nach dem
Match-Erfolg wird die Guard-Expression evaluiert (Subject bleibt am Stack
fuer den Body), und ein zusaetzlicher `JUMP_IF_FALSE` springt zum
naechsten Case wenn der Guard falsch ist. Kein neuer Bytecode -- die
existierenden Ops reichen.

**Kompatibilitaet:** Existierende SELECT-Statements ohne `WHERE` laufen
unveraendert -- der Parser erzeugt `(matches, guard=None, block)`-Tupel,
sowohl 2-Tuple- als auch 3-Tuple-Cases werden in den Pfaden akzeptiert
(Backward-Compat).

## IN-Operator

`x IN container` testet Mitgliedschaft auf String, Tupel, Array oder Map:

```basic
IF "World" IN "Hello World" THEN ...      ' Substring
IF 5 IN (1, 5, 9) THEN ...                ' Tupel
IF "name" IN m THEN ...                   ' Map-Key
IF 20 IN nums THEN ...                    ' Array-Element
```

Maps haben STRING-Keys -- `5 IN map` wirft TypeMismatch. Negation klassisch
mit `NOT (x IN c)`. Praezedenz wie `=`/`<>` (Comparison-Ebene), Bytecode:
neuer Op `IN_OP=56`.

## Variadic-Functions

`...args` als letzter Parameter sammelt alle restlichen Positional-Args
in ein TUPLE:

```basic
SUB log(level AS STRING, ...rest)
    DIM msg AS STRING
    msg = "[" + level + "]"
    DIM j AS INTEGER
    FOR j = 0 TO rest.length() - 1
        msg = msg + " " + STR$(rest[j])
    NEXT
    PRINT msg
END SUB

log("WARN")                          ' rest = ()
log("INFO", "App", "started")        ' rest = ("App", "started")
```

**Einschraenkungen:**
- Variadic muss letzter Parameter sein (Parser-Fehler sonst).
- Variadic-Slot kann nicht mit Named-Arg uebergeben werden (semantisch
  unklar -- werfen).
- TUPLE als Type stuetzt `length()`, `len()` und `[i]`-Index-Access.
- Keine Default-Werte fuer Variadic (immer `()` wenn leer).

**Implementation:** AST `Param.is_variadic`, `CompiledFunction.is_variadic`.
Im Tree-Walker `_resolve_args` und in den VMs `_exec` werden ueberzaehlige
Positional-Args in ein Tupel gesammelt; Default-Resolution wird fuer
variadic-Funktionen umgangen.

**Beispiel:** [examples/62_qol_sprint.dh](examples/62_qol_sprint.dh).

## Method-Syntax auf Built-in-Containern

Strings, Arrays und Maps haben Convenience-Methoden, die zu BUILTINs
delegieren:

```basic
PRINT "hello".upper()           ' "HELLO" (= UPPER$("hello"))
PRINT "  hi  ".trim().upper()   ' Method-Chain: "HI"
PRINT a.length()                ' = LEN(a)
m.put("k", 1)                   ' = MAPPUT(m, "k", 1)
PRINT m.has("k")                ' = MAPHAS(m, "k")
```

**Dispatch-Tabelle:** `interpreter.CONTAINER_METHODS` mappt
`(target_kind, method_name)` zu BUILTIN-Namen. Tree-Walker und beide
VMs konsumieren dieselbe Tabelle (Single-Source-of-Truth).

**Verfuegbare Methoden:**
- String: `upper`, `lower`, `length`/`len`, `trim`, `left`, `right`,
  `mid`, `indexof`, `replace`, `split`, `padl`, `padr`.
- Array: `length`/`len`.
- Map: `put`, `get`, `getor`, `has`, `keys`, `size`/`length`/`len`,
  `remove`, `clear`.

Wer einen weiteren Method-Alias will, fuegt einen Eintrag in
`CONTAINER_METHODS` hinzu -- kein VM/Bytecode-Change noetig.

**User-Klassen-Methoden gewinnen:** Nur wenn der Receiver kein User-
Instanz ist (sondern String/Array/Map), wird die Container-Methoden-
Tabelle konsultiert. So kollidiert `Foo.upper()` mit User-Klasse `Foo`
nicht.

**Beispiel:** [examples/61_method_syntax.dh](examples/61_method_syntax.dh).

## Slicing

`s[a:b]`, `s[a:]`, `s[:b]`, `s[:]` -- liefert Substring (String) oder
neues 1D-Array (Array, echte Kopie). Negative Indices und Step werden
NICHT unterstuetzt -- konsistent mit der existierenden strikten Index-
Validierung. Out-of-bounds (`s[0:1000]` bei `len(s)=11`) wird auf den
gueltigen Bereich geclampt.

```basic
PRINT "Hello World"[6:11]     ' "World"
DIM b AS ARRAY OF INTEGER
b = a[1:4]                    ' echte Kopie, kein Alias
```

**Multi-Dim-Slicing** (`g[0:2, 1:3]`) wird nicht unterstuetzt --
NumPy-Semantik. Slice-Assign (`s[a:b] = ...`) wird ebenfalls bewusst
abgelehnt (Laenge-Match unklar).

**Implementierung:** AST `SliceAccess(target, lo, hi)`. Parser
disambiguiert in `_index_or_slice` anhand des Top-Level-`:`. Bytecode
`SLICE` mit Flag-Tupel `(has_lo, has_hi)` -- die VM popt entsprechend
viele Werte.

**Beispiel:** [examples/60_slicing.dh](examples/60_slicing.dh).

## ELIF / String-Multiplikation

Kleine Quality-of-Life-Erweiterungen:

- **`ELIF`** ist Alias fuer `ELSEIF` -- gleiches Token, kein AST-Change.
- **`"-" * 40`** liefert einen String aus 40 Bindestrichen. Auch
  `40 * "-"`. Negative Counts liefern leeren String. Strikt INTEGER --
  kein Float, kein Bool. Im VM-Pfad wurde gleichzeitig `OP.MUL` strenger
  gemacht (war vorher zu lax bei Bool-Operanden).

## WITH ... END WITH

Klassisches BASIC-Konstrukt fuer kompakte Member-Bursts:

```basic
WITH player
    .x = 100
    .y = 50
    .hp = 100
    .name = "Alice"
END WITH
```

**Semantik:**
- WITH-Ziel wird **einmal** evaluiert (wichtig bei Side-Effects).
- Innerhalb des Body ist `.member` Shortcut fuer `<target>.member`.
- Auch in Read-Position: `len = SQR(.x * .x + .y * .y)`.
- Compound-Assigns funktionieren: `.points += 5`.
- Verschachtelte WITHs erlaubt; innerstes gewinnt (Stack-Semantik).

**Implementierung:** Reines Compile-Zeit-Desugar ohne neuen Bytecode.
- Parser haelt einen `_with_stack: list[str]` mit Compiler-generierten
  Variablen-Namen (`__with_<n>`).
- `_with_stmt` parst, generiert frischen Namen, pusht auf Stack, parst Body,
  popt, gibt `With(var_name, target, body)` zurueck.
- Im `_primary` und `_statement_inner`: wenn aktueller Stack nicht leer und
  Token = DOT, desugar zu `MemberAccess(Identifier(top), name)`.
- Tree-Walker: `_exec_With` setzt `env.vars[var_name] = {"type":"any","value":val}`,
  fuehrt body aus, entfernt den Slot wieder.
- Compiler: `_stmt_With` allokiert anonymen Local-Slot (`_alloc_anon_slot`),
  bindet `var_name -> slot` in `local_slots` waehrend Body-Compile, entfernt
  ihn danach. So wird `Identifier(__with_<n>)` zu `LOAD_LOCAL slot`.
- "any"-Type-Coerce ist passthrough (Tree-Walker `interpreter.py:_coerce`,
  VM `vm.py:_coerce_any`, Cython `vm_native.pyx`).

**Beispiel:** [examples/55_with.dh](examples/55_with.dh).

## Tupel + Destructuring

Mehrfach-Rueckgabewerte ohne BYREF-Krampf.

```basic
FUNCTION minmax(a AS INTEGER, b AS INTEGER) AS TUPLE
    IF a < b THEN RETURN (a, b)
    RETURN (b, a)
END FUNCTION

DIM lo AS INTEGER
DIM hi AS INTEGER
(lo, hi) = minmax(7, 3)        ' Destructuring
```

**Tupel-Literal:** `(a, b, c)` -- mindestens 2 Elemente. Eine einzelne
geklammerte Expression `(expr)` bleibt Klammer-Gruppierung. `(1,)`-Single-
Tupel wie in Python wird NICHT unterstuetzt (kein Use-Case).

**Destructuring-Assignment:** `(t1, t2, ..., tn) = expr`. Die `expr` muss zur
Laufzeit ein Tupel mit exakt n Elementen ergeben -- sonst `DHRuntimeError`.
Targets duerfen Identifier, MemberAccess oder IndexAccess sein
(`(p.x, p.y) = polar_to_cart(r, a)` funktioniert).

**Type-Annotation:** `DIM t AS TUPLE` -- generisch, akzeptiert beliebige
Tupel. Keine Element-Type-Annotation an der Sprachebene; wer striktere
Garantien braucht, prueft selbst beim Destructuring.

**Implementierung:**
- AST: `TupleLit(elements)`, `TupleAssign(targets, value)`.
- Bytecode: `BUILD_TUPLE n` und `UNPACK_TUPLE n` (Ops 68, 69).
- Im Compiler-`_stmt_TupleAssign` werden Member/Index-Targets ueber einen
  anonymen Local-Slot zwischengepuffert (per `_alloc_anon_slot`), weil
  STORE_MEMBER/STORE_INDEX die Receiver-Position vor dem Wert braucht.
- Wertsemantik = Python-`tuple` (immutable). `_fmt` erzeugt `(a, b, c)`
  fuer PRINT.
- Cython-VM muss nach Aenderungen am Tupel-Pfad neu kompiliert werden.

**Praktisch:** Beispiel [examples/54_tuple.dh](examples/54_tuple.dh) zeigt
Min/Max, Vektor-Reflexion, Polar-Konvertierung, Player-State als Tupel.

## Bitwise-Operatoren

Strikt INTEGER (kein FLOAT, kein BOOL). Sechs Operatoren als Keywords:

```basic
a BAND b      ' bit-and
a BOR  b      ' bit-or
a BXOR b      ' bit-xor
a SHL  n      ' shift-left  (n >= 0)
a SHR  n      ' shift-right (n >= 0)
BNOT a        ' unaer, bitweises NICHT (= ~a in Python)
```

**Praezedenz:** Alle binaeren Bitwise auf EINER Ebene, links-assoziativ.
Position zwischen `Vergleich` und `+,-`. Heisst:
- `a BAND b = c` parst als `(a BAND b) = c`.
- `a + b BAND c` parst als `(a + b) BAND c`.
- `1 BOR 2 BAND 3` parst als `((1 BOR 2) BAND 3) = 3` — wer C-Stil-Praezedenz
  will, klammert (`1 BOR (2 BAND 3)`).

`BNOT` liegt im `_unary` neben `-` und unaerem `+` — d.h. tighter binding als
`*`/`/`. `BNOT a BAND b` ist `(BNOT a) BAND b`.

**Type-Strictness:** Bool wird abgelehnt (gleiche Linie wie `_check_num`).
Negativer Shift-Count wirft `DHRuntimeError` statt nichtssagendem Python-Fehler.

**Keine alten Built-ins mehr:** Frueher gab's `BITAND/BITOR/BITXOR/BITNOT/SHL/SHR`
als Funktions-Built-ins. Mit den Operatoren ueberfluessig — entfernt.
`BITAND(a, b)` -> `a BAND b`. Im Tree-Walker (interpreter.py:1009-1027) und
in beiden VMs implementiert (Ops 62-67 in bytecode.py).

## SELECT CASE

Mehrweg-Verzweigung statt verschachtelter `IF/ELSEIF`-Ketten. Drei Match-Formen
pro CASE, beliebig kombinierbar:

```basic
SELECT CASE x
    CASE 1                       ' exakter Wert
        ...
    CASE 2, 3, 4                 ' Liste von Werten
        ...
    CASE 10 TO 20                ' Bereich (inklusiv)
        ...
    CASE IS > 100                ' Vergleich (=, <>, <, >, <=, >=)
        ...
    CASE 1, 5 TO 8, IS = 13      ' alle Formen mischbar
        ...
    CASE ELSE                    ' Fallback (optional, max. einmal, muss letzter sein)
        ...
END SELECT
```

**Garantie:** Subject-Ausdruck wird **einmal** evaluiert (auch bei
Side-Effects in Function-Calls). Der erste passende Case gewinnt.

**Implementierung (lehrreich):** Im Parser zu `Select(subject, cases, else_block)`,
Cases sind `(list[CaseMatch], list[Stmt])`-Tupel mit `kind ∈ {"value", "range"}`.
Im Compiler **kein neuer Bytecode** — der Subject bleibt während aller Match-Tests
auf dem Stack (per `DUP` geklont), Range-Tests werden zu `subj >= lo` (mit
`JUMP_IF_FALSE`) gefolgt von `subj <= hi` (mit `JUMP_IF_TRUE` zum Block) verkettet.
Cython-VM hat es ohne Neukompilation übernommen.

## ENUM

Typsichere Konstanten mit Namespace-Zugriff (`State.PLAYING`):

```basic
ENUM State = MENU, PLAYING, PAUSED       ' compact
ENUM Permission                          ' block
    NONE = 0
    READ = 1
    WRITE = 2
END ENUM
```

Auto-Nummerierung (0, 1, 2, …) oder explizit. Mixed: nach explicit zählt's
weiter (`A, B = 5, C` → A=0, B=5, C=6). Member-Namen dürfen Keywords sein
(`READ`, `FILE`, `DATA`, `NONE`) — der qualifizierte Zugriff ist eindeutig.

**Implementierung:** `EnumDecl(name, members)` AST-Node. Im Tree-Walker
und Compiler zur Compile-Zeit zu einem `_EnumNamespace`-Objekt aufgelöst,
als globale CONST abgelegt. `MemberAccess` erkennt `_EnumNamespace` (in
`interpreter.py`, `vm.py`, `vm_native.pyx`) und liefert den Member-Wert.
Member-Werte müssen Compile-Time-Integer-Literale sein (auch im
Tree-Walker — Konsistenz). `DIM x AS State` löst der Parser zu `INTEGER`
auf, indem er bekannte Enum-Namen in `self._enum_names` trackt.

Keywords als Member-Namen: Parser-Helfer `_consume_member_name` und
DOT-Zugriff in `_postfix` akzeptieren jedes Token mit string-`value`,
nicht nur `IDENT`.

## Named Arguments

`func(name: "Anna", age: 30)` mit Defaults. Lexer-Token `COLON`,
AST-Node `NamedArg(name, value)` als Element von `Call.args`.

**Tree-Walker** (`interpreter.py`): `_resolve_args(decl, raw_args, fn_name)`
mappt positional + named auf Param-Reihenfolge, liefert eine voll-lange
Liste mit `_DEFAULT_SENTINEL` für Slots, die der User nicht belegt hat.
`_invoke` evaluiert Sentinels via Default-Ausdruck im local_env. Funktioniert
mit BYREF (Sentinel-Slots können kein BYREF sein) und mit Param-
referenzierenden Defaults.

**Compiler** (`compiler.py`): `_resolve_named_args(fn, raw_args, fn_name)`
löst zur Compile-Zeit auf — Slots ohne Wert kriegen den evaluierten
Default-Literalwert direkt als `LOAD_CONST` emittiert. `param_names` ist
ein neues Feld auf `CompiledFunction` (Compile-Zeit-Info, VM nutzt es nicht).
Auch `NEW Klasse(...)` wird so resolved (Init-Methode lookup zur Compile-Zeit).

**Einschränkungen im VM-Pfad:**
- Built-ins haben keine deklarierten Param-Namen → werfen.
- Method-Calls (`obj.method(name: ...)`): Klasse erst zur Laufzeit
  bekannt → Compiler wirft. Tree-Walker kann's.

## Self + implizite Methoden-Aufrufe

Innerhalb einer Klassen-Methode:

```basic
CLASS Wave
    SUB Init()
        StartCurrent()         ' impliziter Methoden-Aufruf
    END SUB
    SUB StartCurrent()
        ...
    END SUB
END CLASS
```

`Self` als Identifier liefert die aktuelle Instanz; bare `MethodName(...)`
ohne `Self.`-Präfix dispatcht zuerst gegen die Methoden der eigenen Klasse
(und Superklassen), erst dann gegen globale Funktionen.

**Tree-Walker:** `Interpreter._method_stack: list[(_Instance, _ClassInfo)]`
wird in `_invoke` gepusht/gepoppt. `_eval_Identifier` erkennt `"self"` und
liefert die aktuelle Instanz. `_eval_Call` mit Identifier-callee prüft
zuerst `_resolve_method(current_cls, name)`.

**Compiler:** `_load_var` emittiert für `name == "self"` (innerhalb
`current_class != None`) den neuen Op `LOAD_SELF`. `_expr_Call` bei
Identifier-callee resolved Methoden via `_resolve_method_compile` und
emittiert `LOAD_SELF` + Args + `CALL_METHOD`. Damit Methode A in derselben
Klasse die Methode B sehen kann, registriert Phase 4a vor dem Body-
Kompilieren leere Stub-`CompiledFunction`s in `ci.methods`.

**Bytecode-Op:** `LOAD_SELF = 88` — push `self_obj` (im VM-`_exec` als
Parameter). Implementiert in `vm.py` und `vm_native.pyx`. Wer Self-Code
schreibt, muss daher `vm_native.pyx` neu kompilieren.

## Statement-Trenner Doppelpunkt

`x = 1 : y = 2` — Doppelpunkt trennt Statements wie Newline.
`Parser._consume_terminator` und `_skip_newlines` akzeptieren beide Token.
Funktioniert mit Named-Args nicht in Konflikt, weil dort der `IDENT COLON`-
Lookahead in `_call_arg` läuft (innerhalb von `(` ... `)`), wo der
Terminator gar nicht erst geprüft wird.

## f-Strings (String-Interpolation)

`f"text {expr} text..."` -- der Lexer expandiert das zur Token-Sequenz
`("text" + STR$(expr) + "text" + ...)`. Damit funktionieren f-Strings ohne
einen einzigen Eingriff in Parser, Interpreter, Compiler oder VMs:

```basic
DIM name AS STRING
DIM hp AS INTEGER
name = "Anna"
hp = 75
PRINT f"{name} hat {hp} HP"          ' "Anna hat 75 HP"
PRINT f"max: {MAX(a, b)}"            ' Methodenaufrufe in {} sind ok
PRINT f"literal {{nicht interpoliert}}, aber {hp}"
```

**Eigenschaften:**
- `{{` und `}}` sind Escapes fuer literale geschweifte Klammern.
- Verschachtelte f-Strings sind nicht erlaubt (`f"{f"..."}"`).
- Ausdruecke duerfen `(`, `)`, Methoden-Aufrufe, MemberAccess etc.
  enthalten -- der Tokenizer matched balanced braces.
- Ohne `f`-Prefix bleibt `"hi {name}"` ein wortlich enthaltener String mit
  geschweiften Klammern -- Opt-in.
- Editor-Highlighter erkennt f-Strings als Block und faerbt den ganzen
  Range einheitlich als String (siehe editor_qt/highlighter.py).

**Format-Specs** (`{expr:spec}`): ein Top-Level-`:` im Platzhalter trennt
einen printf-Spec ab -- der Lexer emittiert dann `FORMAT$(expr, "%spec")`
statt `STR$(expr)`:
```basic
PRINT f"FPS {fps:.1f}  Score {score:05d}"   ' "FPS 59.7  Score 00042"
```
Ein `:` innerhalb von `()`/`[]`/`{}` (z.B. Slice `s[0:3]`) oder String-Literalen
zaehlt NICHT als Spec-Trenner -- `_split_fstring_spec` in `lexer.py` trackt
Klammer-/String-Tiefe. Rein Lexer-basiert, daher in allen drei Pfaden gleich.

**Implementierung:** `lexer._scan_fstring` wird beim ersten `f"`-Lookahead
aufgerufen (lexer.py:114-115) und emittiert die
expandierte Token-Sequenz selbst -- mit Sub-Lexer fuer den Ausdrucks-Teil.

**Beispiel:** [examples/69_fstring.dh](examples/69_fstring.dh).

## Kontrollfluss: BREAK / CONTINUE / REPEAT / TRY

Diese sind implementiert (waren in aelteren Doku-Staenden nicht aufgefuehrt):

- **`BREAK`** / **`CONTINUE`** in `FOR`, `FOR EACH`, `WHILE`, `REPEAT`. Auch in
  Single-Line-IF (`IF v = 40 THEN BREAK`). Tree-Walker via `_BreakSignal`/
  `_ContinueSignal`-Exceptions; Compiler via `break_patches`/`continue_patches`-
  Stack (mit `try_depth` fuer korrektes `TRY_END`-Unwinding).
- **`REPEAT ... UNTIL cond`** -- Post-Test-Loop (laeuft mindestens einmal).
- **`WHILE cond ... WEND`** -- Pre-Test-Loop.
- **`TRY ... CATCH [e] ... END TRY`** + **`THROW value`**. Die Catch-Variable
  ist optional und faengt den (String-)Wert. Kein typed Catch -- `THROW` wirft
  beliebige Werte; Module-/Runtime-Fehler kommen als `DHRuntimeError`-Message.

```basic
FOR EACH e IN enemies
    IF e.dead THEN CONTINUE
    IF boss_killed THEN BREAK
    e.update()
NEXT

TRY
    riskante_op()
CATCH msg
    PRINT "Fehler: " + msg
END TRY
```

## FOR EACH

`FOR EACH var IN container ... NEXT` -- iteriert ueber STRING (Zeichen),
TUPLE, 1D-ARRAY oder MAP (Keys):

```basic
FOR EACH b IN bullets
    b.update()
NEXT
FOR EACH k IN scores        ' Map -> Keys
    PRINT k, MAPGET(scores, k)
NEXT
```

`each` ist **kontextuell**, kein Keyword: `FOR each = 1 TO 3` mit einer
Variable namens „each" bleibt ein regulaerer FOR (Disambiguierung im Parser:
FOR EACH nur wenn nach `each` ein IDENT statt `=` folgt).

**Implementierung:** AST-Node `ForEach(var, iterable, body)`. Tree-Walker
iteriert direkt (`_iter_for_comp`, wie Comprehensions). Compiler desugart zu
einem Vorwaerts-Index-Loop ueber `__comp_iter(iterable)` (-> TUPLE) +
`LOAD_INDEX` und nutzt den vorhandenen break/continue-Patch-Stack -- **kein
neuer Bytecode**, beide VMs unveraendert. Loop-Var wird als `"any"` deklariert.

## IIF (Inline-Ternary)

`IIF(cond, then, else)` -- echter **lazy** Ternary, nur EIN Zweig wird
ausgewertet (Short-Circuit):

```basic
dx = IIF(moving_left, -speed, speed)
PRINT IIF(x <> 0, 100 \ x, -1)    ' bei x=0: kein Division-Crash
```

`iif` ohne `(` bleibt ein normaler Bezeichner (kontextuell im Parser).
**Implementierung:** AST-Node `TernaryExpr(cond, then, else)`. Compiler
desugart zu `JUMP_IF_FALSE` (poppt die Bedingung) -- **kein neuer Bytecode**.
Tree-Walker: `_eval_TernaryExpr` evaluiert nur den gewaehlten Zweig.

## Array- & Map-Helfer

Reine Builtins (alle drei Pfade automatisch), auch als Container-Methoden
(`CONTAINER_METHODS`-Tabelle):

- `SORT(arr)` / `arr.sort()` -- 1D-Array IN PLACE aufsteigend (INTEGER/FLOAT/STRING).
- `REVERSE(arr)` / `arr.reverse()` -- 1D-Array IN PLACE umkehren.
- `ARRAY_INDEXOF(arr, v)` / `arr.indexof(v)` -- erster Index oder -1.
- `MAPVALUES(m)` / `m.values()` -- ARRAY aller Werte (Einfuege-Reihenfolge).
- `MAPITEMS(m)` / `m.items()` -- ARRAY von `(key, value)`-TUPELn (gut mit
  `FOR EACH` + Destructuring).

## Module-Imports mit Alias

`IMPORT "modul" AS alias` -- aliased die Built-ins / externen Typen unter
einem ersetzten Praefix:

```basic
IMPORT "json" AS j
DIM h AS J_HANDLE
h = J_PARSE("[1, 2, 3]")
PRINT J_GET_INT(h, "0")     ' 1
```

**Aliasing-Strategie:** Drachenhauch-Module teilen einen flachen Built-in-
Namespace -- es gibt kein echtes Namespacing. Der Alias dupliziert alle
Built-ins / Typen, deren Name mit `<modul>_` anfaengt, unter `<alias>_`.
Single-word-Namen (z.B. der externe Typ `vec2`) werden komplett ersetzt
(`vec2` -> `v` bei `IMPORT "vec2" AS v`).

**Konvention-basiert:** funktioniert fuer Module, deren Built-in-Praefix
dem Modul-Namen entspricht (json, db, tween, vec2, sprite, ecs, ...).
Module mit abweichendem Praefix (z.B. `imgfx` registriert `IMAGE_*`,
nicht `IMGFX_*`) sind nicht via Alias adressierbar -- der Praefix-Match
liefert dann leer.

**Idempotent + sticky:** zweimal mit unterschiedlichen Aliasen ist OK
(beide werden zusaetzlich registriert), aber doppelt mit demselben Alias
ist no-op.

## Dict/Set-Comprehensions

`{key: val FOR var IN iterable [WHERE filter]}` -- Dict-Comprehension,
liefert eine MAP. `{expr FOR var IN iterable [WHERE filter]}` -- Set-
Comprehension, liefert ein TUPLE mit deduplizierten Werten in der
Reihenfolge des ersten Auftretens.

```basic
' Dict-Comp: Quadrate als Map
DIM squares AS MAP OF INTEGER
squares = {STR$(x) + "sq": x * x FOR x IN (1, 2, 3, 4)}
PRINT MAPGET(squares, "3sq")     ' 9

' Set-Comp: eindeutige Mod-Werte
DIM distinct AS TUPLE
distinct = {x MOD 3 FOR x IN (0, 1, 2, 3, 4, 5, 6, 7, 8)}
PRINT distinct                    ' (0, 1, 2)
```

**Dict-Keys MUESSEN STRING sein** (Drachenhauch-MAP-Konvention). Der MAP-
Wert-Typ wird beim ersten Eintrag inferiert. Set-Comp ist eine pragmatische
Naeherung -- Drachenhauch hat keinen echten SET-Typ; das deduplizierte TUPLE
ist die nahe liegende Alternative.

**Implementierung:** Lexer kennt jetzt `LBRACE`/`RBRACE` (nur fuer Comp-
Position). Parser disambiguiert per `:`-Lookahead (Dict) oder direkt
`FOR` (Set). Compiler nutzt das existierende `BUILD_TUPLE_DYN`-Pattern
plus zwei interne Built-ins (`__SET_DEDUP`, `__DICT_FROM_PAIRS`) als
Final-Schritt -- keine neuen Bytecode-Ops noetig, Cython-VM ohne
Aenderung kompatibel.

**Beispiel:** [examples/71_dictcomp.dh](examples/71_dictcomp.dh).

## Operator-Overloading auf User-Klassen

Klassen koennen Operatoren ueberladen, indem sie `OPERATOR <op>`-Methoden
definieren -- analog zu `SUB`/`FUNCTION` im Class-Body:

```basic
CLASS Money
    DIM cents AS INTEGER

    OPERATOR + (other AS Money) AS Money
        DIM r AS Money
        r = NEW Money()
        r.cents = Self.cents + other.cents
        RETURN r
    END OPERATOR

    OPERATOR = (other AS Money) AS BOOLEAN
        RETURN Self.cents = other.cents
    END OPERATOR
END CLASS

DIM a AS Money
a = NEW Money()
a.cents = 100
DIM b AS Money
b = NEW Money()
b.cents = 200
PRINT (a + b).cents     ' 300
PRINT a = b             ' FALSE
```

**Erlaubte Operatoren:** `+`, `-`, `*`, `/`, `MOD`, `=`, `<>`, `<`, `>`,
`<=`, `>=`. Genau ein Parameter (`other`), Rueckgabetyp ist Pflicht.
BYREF und variadic sind nicht erlaubt.

**Implementierung:** Parser konvertiert `OPERATOR + (...)` zu einer Methode
mit reserviertem Namen `__op_add__` (siehe `parser._OPERATOR_NAMES`).
Tree-Walker (`_eval_BinaryOp`) und beide VMs (`OP.ADD/SUB/MUL/DIV/MOD/EQ/...`)
konsultieren via `_user_op(...)` die Methode auf LHS, dann auf RHS
(Reverse-Dispatch). Fallback ist der Standard-Pfad.

**Vererbung:** Operator-Methoden werden ueber die normale MRO gesucht --
Child-Klassen erben sie automatisch.

**Einschraenkungen:**
- Keine reflektierten Operatoren a la Python (`__radd__`). Wer `5 + money`
  unterstuetzen will, definiert `OPERATOR + (other AS INTEGER) AS Money`
  auf `Money` -- der Reverse-Dispatch greift dann.
- Kein Method-Overloading: pro Operator gibt's genau eine Methode.
  `Money + Money` und `Money + INTEGER` koennen nicht gleichzeitig
  definiert werden (man muesste type-switchen im Body).
- Operatoren auf Modul-Typen (Vec2 etc.) gewinnen vor User-Klassen --
  die Modul-Registry wird zuerst konsultiert.

**Beispiel:** [examples/70_operator.dh](examples/70_operator.dh).

## Operator-Registry

Modul-eigene Typen koennen arithmetische Operatoren (`+`, `-`, `*`, `/`)
ueberladen, ohne dass interpreter.py / vm.py / vm_native.pyx angefasst werden:

```python
# In drachenhauch/modules/<name>.py
from . import register_operators

def _op_add(a, b):
    if isinstance(a, _MyType) and isinstance(b, _MyType):
        return _MyType(...)
    raise TypeMismatchError("...")

register_operators(_MyType, {"+": _op_add, "-": _op_sub, "*": _op_mul, "/": _op_div})
```

**Dispatch:** Vor dem Standard-Pfad ruft Tree-Walker (`_eval_BinaryOp`) und
beide VMs (`OP.ADD/SUB/MUL/DIV`) `modules.dispatch_binary_op(op, a, b)`.
Wenn `type(a)` oder `type(b)` registriert ist, dispatcht zur Handler-Tabelle;
sonst liefert die Registry `NO_OP_MATCH` und der Standard-Pfad uebernimmt.

**Konvention:** Bei asymmetrischen Operatoren (z.B. `Skalar * Vec2`) muss
der Handler beide Reihenfolgen selbst akzeptieren, weil die Registry nur
einen Handler-Eintrag pro Typ kennt -- siehe `vec2._op_mul` als Pattern.

**Equality:** `=` und `<>` sind nicht in der Registry -- die Standard-
Python-Equality (`__eq__`/`__ne__` auf der Klasse) reicht. Die VMs nutzen
`a == b` direkt.

Wer einen neuen Math-Typ wie `_Mat3x3`, `_Complex` oder `_Quat` einbauen
will, schreibt nur sein Modul -- keine Aenderung an Interpreter oder VMs.

## Asset-Cache + `LOAD_ASSETS`

`Graphics` haelt zwei Caches: `_image_cache` und `_sound_cache`. Sowohl
`LOADIMAGE` als auch `LOADSOUND` pruefen zuerst den Cache und cachen
das Ergebnis unter zwei Schluesseln: dem rohen Pfad UND dem
normalisierten Absolut-Pfad. So treffen verschiedene Pfad-Schreibweisen
(`"sprites/x.png"` vs. `"./sprites/x.png"` vs. absolut) denselben Eintrag.

`LOAD_ASSETS(manifest.json)` praefuellt den Cache aus einem JSON-Manifest:

```json
{
  "images": { "player": "sprites/player.png", "enemy": "sprites/enemy.png" },
  "sounds": [ "sfx/jump.wav", "music/level1.ogg" ]
}
```

Beide Sektionen sind optional, jede kann **Dict** (Alias → Pfad) oder
**Liste** (nur Pfade) sein. Bei Dict-Form ist `LOADIMAGE("player")`
ein Cache-Hit (Alias) **und** `LOADIMAGE("sprites/player.png")` auch
(Pfad-Hit unter Absolut-Pfad). Pfade im Manifest sind relativ zum
Manifest-Verzeichnis.

`LOAD_ASSETS` liefert die Anzahl geladener Assets. Idiomatisch in der
Init-Phase nach `SCREEN(...)` aufrufen, damit Bilder direkt
`convert_alpha`-optimiert werden. Beispiel: [examples/75_preloader.dh](examples/75_preloader.dh).

## Z-Layer-Rendering

Layer sind named Compose-Surfaces mit explizitem z-Wert. Alle
draw-Methoden zeichnen auf `Graphics._buffer`; `LAYER("name")` lenkt
`_buffer` auf die Layer-Surface um. `FLIP` composiert alle Layer in
z-Order auf den `_main_buffer`, blittet zum Screen, und cleart die
Layer (transparent) fuer den naechsten Frame.

```basic
LAYER_DEFINE("bg", 0)
LAYER_DEFINE("sprites", 10)
LAYER_DEFINE("ui", 100)

LAYER("bg");      CLS(SKY); DRAWIMAGE(parallax, 0, 0)
LAYER("sprites"); DRAWIMAGE(player, x, y)
LAYER("ui");      TEXT(10, 10, "Score: 100")
FLIP()   ' composiert in z-Order, cleart fuer naechsten Frame
```

**Builtins:**
- `LAYER_DEFINE(name, z)` — registrieren mit z; redefine aktualisiert z
- `LAYER(name)` — switchen (auto-Define mit auto-z wenn neu)
- `LAYER_END()` — zurueck zum Main-Buffer (optional, FLIP macht's auch)
- `LAYER_CLEAR(name)` — manuell leeren (selten gebraucht)

**Implementation** ([rust/drachenhauch_runtime/src/graphics.rs](rust/drachenhauch_runtime/src/graphics.rs)):
kein Surface-Compositing (das war das alte, entfernte Python-`graphics.py`
via pygame) — dhrt ist ein **Command-Recording-Modell**. Jede Layer ist ein
`Layer { z, cmds: Vec<Cmd> }`; Draw-Aufrufe haengen sofort einen `Cmd` an
die *aktive* Layer (`self.active`, per `LAYER(name)` umgeschaltet) an, statt
etwas zu rendern. `FLIP()` sortiert alle Layer-Indizes nach `z` aufsteigend
(niedrigstes z = hinten, hoechstes z = vorne) und spielt deren `cmds` in
dieser Reihenfolge in EINEM `begin_drawing`-Block ab (`render_scene`),
danach werden alle `cmds`-Vecs geleert (Immediate-Mode: ein Frame lang
gueltig) und `self.active`/`self.active_rt` auf den Main-Buffer
zurueckgesetzt.

**Backwards-Compat:** Code ohne `LAYER_*`-Calls hat `_layer_order = []`,
der Compose-Pfad in FLIP ist ein No-Op und `_buffer` zeigt direkt auf
`_main_buffer`. Existierende Programme laufen unveraendert.

**Beispiel:** [examples/76_layers_atlas.dh](examples/76_layers_atlas.dh).

## Sprite-Atlas + Batch-Draw

Sprite-Atlas: EIN grosses Image + Dict von `name -> (x, y, w, h)`-Rects.
Mehrere Sub-Sprites teilen sich eine Textur -- Game-Engine-Pattern fuer
Tilemaps, Bullet-Hell, Tile-Drawing. Nativ in dhrt; Tree-Walker
konsolen-only -> wirft "nur dhrt".

> **`BATCH_DRAW`/`BATCH_FLUSH` sind heute reine API-Kompatibilitaet, KEIN
> echtes Batching.** Der Name + die Kommentare unten stammen aus der alten
> Python-Engine (pygame-Surface-Blits, die sich tatsaechlich zu einem
> gebatchten Blit sammeln liessen). dhrts Immediate-Mode ist ein
> Command-Recording-Modell: jeder Draw-Aufruf haengt sofort einen `Cmd` an
> die aktive Layer, `FLIP()` sortiert die Layer nach z und spielt sie ab.
> In diesem Modell IST `BATCH_DRAW` identisch zu `ATLAS_DRAW` (derselbe
> Match-Arm in `vm.rs`), und `BATCH_FLUSH()` ist ein No-Op -- es gibt
> keinen separaten Batch-Queue-Zustand, der etwas zu flushen haette. Die
> "Auto-Flush"/"Zoom-Caveat"-Absaetze unten beschreiben daher ein
> Verhalten, das in dhrt nicht (mehr) existiert; sie sind als historische
> Notiz stehen gelassen, nicht als aktuelle Verhaltensdokumentation.

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/tiles_atlas.json")

' Einzel-Draw (Camera-aware):
ATLAS_DRAW(atlas, "tile_grass", 0, 0)

' Batch-Pattern (schneller bei vielen Sprites):
FOR i = 0 TO 99
    BATCH_DRAW(atlas, "tile_grass", i * 16, 0)
NEXT
BATCH_FLUSH()   ' EIN gebatchter Draw-Call fuer 100 Sprites
```

**Manifest-Format:**
```json
{
  "image": "tiles.png",
  "sprites": {
    "tile_grass": [0,  0, 16, 16],
    "player":     [16, 0, 24, 32]
  }
}
```
Rects sind `[x, y, w, h]`. Bild-Pfad relativ zum Manifest.

**Builtins:**
- `ATLAS_LOAD(json)` -> `SPRITE_ATLAS`
- `ATLAS_DRAW(atlas, name, x, y)` — einzeln, Camera-aware
- `BATCH_DRAW(atlas, name, x, y)` — an Batch-Queue anhaengen
- `BATCH_FLUSH()` — Queue jetzt rendern (gebatchter Draw-Call)

**Auto-Flush** an den richtigen Punkten:
- vor FLIP (sonst geht die Queue verloren)
- vor LAYER-Switch (damit Batch zum richtigen Target geht)
- vor ATLAS_DRAW (Direct-Call) -- bewahrt Reihenfolge

**Zoom-Caveat:** Bei `CAMERA_SET`-Zoom ≠ 1 faellt jeder `BATCH_DRAW`
auf einen Einzel-Draw zurueck (kein Batch-Vorteil, weil der Batch nicht
skaliert zeichnen kann). Translation ist OK.

**Externer Typ:** `SPRITE_ATLAS` ist in TYPE_DEFAULTS / _coerce in
allen drei Pfaden (Tree-Walker, vm.py, vm_native.pyx) registriert.
`DIM x AS SPRITE_ATLAS` funktioniert direkt ohne IMPORT.

## ECS Bulk-System-Ops

Klassische ECS-Performance-Falle: pro-Entity-Loop in BASIC mit 6
Builtin-Calls/Entity. Beispiel-Bench: 500 Entities × 100 Frames mit
`ECS_GET_FLOAT`/`ECS_ADD_FLOAT` → 215 ms auf der Native-VM. Mit
`ECS_INTEGRATE_FLOAT(world, "px", "vx")` → **5 ms (43× schneller)**.

Die Bulk-Ops verarbeiten eine ganze Component-Schicht in einer cdef-
Loop, ohne Python-Dispatch-Overhead pro Entity:

| Builtin | Wirkung |
|---|---|
| `ECS_INTEGRATE_FLOAT(w, target, delta)` | `target += delta` fuer alle Entities mit beiden Components |
| `ECS_INTEGRATE_INT(w, target, delta)` | INT-Variante |
| `ECS_SCALE_FLOAT(w, target, factor)` | `target *= factor` (z.B. Friction) |
| `ECS_FILL_FLOAT(w, target, value)` | alle Werte = value (Reset) |
| `ECS_FILL_INT(w, target, value)` | INT-Variante |
| `ECS_CLAMP_FLOAT(w, target, lo, hi)` | Bounds-Clamp |
| `ECS_REMOVE_DEAD(w, name, threshold)` | Entities mit `value <= threshold` zerstoeren |
| `ECS_COUNT_WITH(w, name)` | O(1) Halter-Zaehlung |

**Implementation:** [rust/drachenhauch_runtime/src/ecs.rs](rust/drachenhauch_runtime/src/ecs.rs).
World und Components liegen als Sparse-Sets vor; die Bulk-Ops laufen dort als
eine Schleife ueber die ganze Component-Schicht statt als ein Builtin-Aufruf je
Entity -- daher der Unterschied. Die frueheren Python-/Cython-Fassungen
(`modules/ecs_py.py`, `ecs_native.pyx`) sind mit Stufe B entfernt.

**Beispiele:** [examples/bench_ecs_movement_v2.dh](examples/bench_ecs_movement_v2.dh)
(Integrate-only), [examples/bench_ecs_systems.dh](examples/bench_ecs_systems.dh)
(volles Bullet-Hell-Pattern mit 8 Systemen pro Frame).

**Game-Pattern-Lesson:** Wer ein Spiel-Hot-Path-System hat, das ueber
viele Entities laeuft, sollte es als Bulk-Op-Builtin schreiben statt
als pro-Entity-BASIC-Loop. Boilerplate fuer einen neuen Bulk-Builtin:
die Logik in `rust/drachenhauch_runtime/src/ecs.rs` + Dispatch-Arm in `vm.rs`
(`try_ecs`) + Eintrag in `daten/builtin_index.json` + run_gb-Golden-Test.

## Kein Cython mehr, kein `setup.py build_ext`

Früher gab es zwei Cython-Module (`array_native.pyx` = `_GBArray`,
`ecs_native.pyx` = ECS-`_World`/`_Component`), die Hot-Path-Code des
Tree-Walkers nach C verschoben. Mit dem Tree-Walker sind auch sie weg, ebenso
`setup.py`, `drachenhauch/interpreter.py` und die `modules/*.py` — die
gesamte Laufzeit-Performance liegt in `dhrt` (Rust). Neuer Cython-Code kommt
nicht mehr dazu; wer Hot-Path-Arbeit hat, macht sie in `builtins.rs`/`vm.rs`.

pygame ist ebenfalls entfernt: Grafik und Audio gibt es ausschließlich in
`dhrt`. Python bedient nur noch die Editoren und das Tooling.

## Performance-Optimierungen im Compiler/VM

Mehrere Stufen, die zusammen die Native-VM auf 3–14× ggue Tree-Walker
bringen. Vollstaendige Bench-Tabelle in [docs/PERFORMANCE.md](docs/PERFORMANCE.md).

### Spezialisierte Numeric-Opcodes

11 Opcodes (`ADD_NN`, `SUB_NN`, `MUL_NN`, `DIV_NN`, `LT_NN`, `GT_NN`,
`LEQ_NN`, `GEQ_NN`, `EQ_NN`, `NEQ_NN`, `NEG_N`). Der Compiler emittiert
sie ueber `_expr_type(e)`-Inference (best-effort, konservativ), wenn
beide Operanden statisch als numerisch bekannt sind. Liest aus:
- Locals via `local_types[slot]`
- Globals via `_global_types[name]`
- Function-Return-Types
- BinaryOp/UnaryOp rekursiv
- **IndexAccess auf typisierte Arrays** (`buf[i]` bei `ARRAY OF INTEGER/FLOAT`
  -> Element-Typ) und **`Self.feld`** mit statisch bekanntem Skalartyp.
  Sicher, weil typisierte Arrays/Felder homogen sind. Damit greift der
  `_NN`-Pfad auch bei `buf[i] + i + j` und `Self.total + n`.

Bei `bench_loop` (tighter Numeric-Loop): **1.46×** auf Native-VM.

FOR-Loop-Bookkeeping (Increment + Bound-Check) wird ebenfalls ueber
spec-ops emittiert.

### 1D-Array-Index-Fast-Path

`LOAD_INDEX`/`STORE_INDEX` in beiden VMs haben einen Fast-Path fuer den
haeufigsten Fall: 1D-`_GBArray`, ein int-Index in Bounds. Ueberspringt die
`isinstance`-Cascade (str/tuple/array) + Index-Validierungs-Loop. Alle
Edge-Cases (String/Tupel/Multidim/OOB/bool) fallen unveraendert in den
generischen Pfad -> identische Fehler, kein Bit-identisch-Risiko. Kein neuer
Opcode, kein Compiler-Eingriff.

### Tree-Walker-Dispatch-Cache

`Interpreter._eval`/`_exec` memoisieren die Handler-Methode pro Node-Typ
(`_eval_cache`/`_exec_cache`: `type(node) -> gebundene Methode`) statt pro
Node `getattr(self, f"_eval_{name}")` zu bauen. Wirkt auf ALLEM Tree-Walker-
Code (~13 % bei expression-dichten Frames) -- relevant, weil der Editor-Run
und die Bench-Equivalenz den Tree-Walker nutzen.

### Inline-Cache fuer OOP-Dispatch

`CompiledFunction.caches`-Liste parallel zu `code`. Monomorphic IC fuer
`CALL_METHOD`, `LOAD_MEMBER`, `STORE_MEMBER` auf `_Instance`-Receivern.
Hit-Check: `obj.cls is cache[0]`. Spart `_resolve_method`-Call und
Dict-Lookup. Bei `bench_method_dispatch`: **1.34×**.

Cache wird auch im stub-Pfad (Phase 4a) konsistent kopiert. (Beschreibt den
geloeschten Python-Compiler -- der Rust-Compiler emittiert keine Inline-Caches,
siehe die Stufe-B-Warnung oben.)

### Globals-as-Slots

Compile-Zeit-Aufloesung von Top-Level-DIM/CONST/Enum/For-var/Class-
Static-Namen zu Slot-Indizes (`_global_slots: dict[str, int]`).
Neue Opcodes:
- `LOAD_GLOBAL_SLOT idx`
- `STORE_GLOBAL_SLOT idx`
- `DECLARE_GLOBAL_SLOT (idx, name_idx, type, default)`
- `DECLARE_GLOBAL_CONST_SLOT (idx, name_idx, type)`

Die VM-Pfade fuehren `global_slots: list[_Slot]` parallel zum
`globals_`-Dict. `DECLARE_*_SLOT` schreibt den `_Slot` in BEIDE
Strukturen (gleiches Object), so bleiben name-basierte Ops
(`INPUT_NAME`, `LOAD_NAME`) konsistent.

Pre-registrierte Globals (`KEY_*`, `BLACK`, `WHITE`, `PI`, ...) leben
weiter nur im Dict, weil der Compiler sie nicht statisch erkennt --
sie gehen ueber den Fallback `LOAD_NAME`.

`Struct`/`Array`/`Map`-DIMs werden **nicht** slot-allokiert (ihre Init-
Pfade `DECLARE_STRUCT_NAME` / `DECLARE_ARRAY_NAME` haben spezielle
Allokation -- der Performance-Vorteil waere klein, der Kosten gross).

Bei `bench_loop`: **1.30×** zusaetzlich.

### Constant Folding

`_try_fold(e)` im Compiler — BinaryOp/UnaryOp mit konstanten Operanden
wird zu einem einzelnen `LOAD_CONST`. Konservativ:
- kein Folding bei Bool-in-Arithmetik
- kein Folding bei Division durch 0 (Runtime-Error besser)
- kein Folding bei extrem grossen POW-Werten (Safety-Cap)
- `and`/`or` werden NICHT gefoldet (Short-Circuit-Semantik bewahren)

Hilft Patterns wie `FOR i = 0 TO 100 - 1`, `width / 2`, `2 * 3.14`.

### Typed Array Backing + cdef `_GBArray`

`ARRAY OF INTEGER` nutzt `array.array('q')` (8-Byte signed int) statt
Python-Liste. `ARRAY OF FLOAT` analog `array.array('d')`. Spart
Box/Unbox bei jedem Zugriff. **64-bit-Limit fuer INTEGER-Arrays**
(-9.2e18..9.2e18) — Skalar-`DIM x AS INTEGER` bleibt arbitrary-
precision.

`_GBArray` ist eine reine Python-Klasse (inline in `interpreter.py`; die frühere
Cython-Variante `array_native.pyx` mit typed memoryviews wurde entfernt).
`get_at(indices)` / `set_at(indices, value)` bleiben die Zugriffs-API.

### Convert/Coerce-Fast-Path in der Python-VM

`vm.py` hat ein `_FAST_COERCE`-Dict mit pro-Typ-Funktionen statt einer
generischen `if`/`elif`-Cascade. Trifft den heissesten Pfad jeder
STORE-Op (Local/Global/Field/Index/Parameter-Binding).

## Die Werkzeuge (in Drachenhauch)

Die frueheren Qt-Werkzeuge (`dhsprites`, `dhtilemap`, `dhform`, `dhscore`,
`dhanim`, `dhtracker`, `dhsfx`, `dhparticles`, Audio Studio) sind geloescht.
Ihre Nachfolger sind Drachenhauch-Programme, in der IDE unter *Werkzeuge*:

| Werkzeug | Programm | Handbuch | Pruefung |
|---|---|---|---|
| SFX-Generator | `examples/183_sfx_generator.dh` | `docs/sfx-generator.md` | `tests/pruef/werkzeug_sfx.dhtest` |
| Partikel-Editor | `examples/185_partikel_editor.dh` | `docs/particle-editor.md` | `tests/pruef/werkzeug_partikel.dhtest` |
| Tilemap-Editor | `examples/187_tilemap_editor.dh` | `docs/tilemap-editor.md` | `tests/pruef/werkzeug_tilemap.dhtest` |
| Sprite-Editor | `examples/189_sprite_editor.dh` | `docs/sprite-editor.md` | `tests/pruef/werkzeug_sprite.dhtest` |
| Tracker | `examples/190_tracker.dh` | `docs/tracker.md` | `tests/pruef/werkzeug_tracker.dhtest` |
| Form-Designer | `examples/197_form_designer.dh` | `docs/form-designer.md` | `tests/pruef/werkzeug_formdesigner.dhtest` |
| Anim-FSM-Editor | `examples/198_anim_fsm_editor.dh` | `docs/anim-editor.md` | `tests/pruef/werkzeug_animfsm.dhtest` |
| Notenblatt | `examples/199_notenblatt.dh` | `docs/score-editor.md` | `tests/pruef/werkzeug_notenblatt.dhtest` |

Jedes Handbuch nennt am Ende, was die Qt-Fassung zusaetzlich konnte. Bekannte
Luecke im Notenblatt: [Neu] und [Oeffnen] fragen bei ungesicherten
Aenderungen NICHT nach, nur das Schliessen tut es (gefunden 2026-09-21).

## Python-Abbau, Weg A: Werkzeugkette in dhrt (2026-09-06)

Richtung des Nutzers: Python soll wegfallen, alles laeuft ueber Rust und
Drachenhauch (`docs/entwurf-python-abbau.md`, Empfehlung A -> C -> B, D je
Bereich). **A ist gebaut.** Drei Unterbefehle, alle im Repo-Kontext
(`doku::repo_wurzel` sucht von cwd aufwaerts nach `docs/` + Index):
`dhrt lsp` (siehe naechster Abschnitt), `dhrt doku prosa|grammatik|referenz`
(`doku.rs`: Prosa aus `docs/`-Tabellen und -Listen plus Referenzbuch ueber
`node tools/buch_cmd_export.js`; Grammatik aus `lexer::KEYWORDS` + Index +
Handdoku; Referenz aus dem Quelltext ueber `symbole.rs` -- das lag frueher
NICHT in dhrt, "weil der Rust-Lexer Kommentare wegwirft"; der Symbol-Scanner
liest den Text) und `dhrt pruef [bloecke|namen|zaehlungen|konstanten|pfade|
beispiele]` (`pruef.rs`: die Codebloecke laufen IN-PROZESS durch
`check_source`, kein dhrt-Start je Buendel; `GEDULDET`/`GEDULDETE_PFADE` mit
Begruendung je Eintrag; `pfade` prueft Inline-Code UND Markdown-Links, die
Duldung gilt nur fuer Inline-Code). `--pruefen` bei `doku` = schreibt nichts,
Rueckgabe 1 bei Abweichung -- so haengen `builtin_prosa.json` und die
VS-Code-Grammatik an der Suite; `dhrt pruef` laeuft dort ebenfalls (seit
2026-09-16 alles in `tests/pruef/doku_pruefungen.dhtest`, vorher vier
pytest-Dateien). **Geloescht:**
`drachenhauch/lsp/`, `drachenhauch/doku.py`, `tools/gen_builtin_prosa.py`,
`tools/pruef_docs.py`, `tools/pruef_doku_aussagen.py`,
`vscode-drachenhauch/build_grammar.py`, die zwei LSP-Testdateien.
`builtin_docs.py` haelt keine Tabelle mehr, sondern laedt `builtin_docs.json`
(dieselbe Datei bettet dhrt ein). `dhrun.py --doku` reicht an
`dhrt doku referenz` durch. **Bleibt Python:** `rust/build_runtime.py` (Bau),
tools/qt_tests_einzeln.py (Qt-Testlaeufer), der Installer (PyInstaller,
solange die IDE Python ist), `tools/*.js` (Node, Buch). **Falle beim Bau:**
`serde` musste als direkte Abhaengigkeit dazu (fuer den 1-Leerzeichen-
Einzug der JSON-Ausgabe; nur `serde_json` reichte nicht, weil der Trait
`Serialize` aus `serde` kommt).

## Python-Abbau, Weg B: der Form-Designer in Drachenhauch (2026-09-06)

Der erste der vier Editoren ohne Piloten (Form-Designer, Anim-FSM,
Notenblatt, Audio Studio): `examples/197_form_designer.dh`, 860 Zeilen gegen
5055 der Qt-Fassung (Faktor 0,17 -- misst wie immer vor allem, was
weggelassen ist: Mehrfachauswahl, Layout-Zuordnung, Regeln/Bindung,
Menue-Editor, Code-Editor, GB-Code-Export, Projekte fehlen). **Bauweise:**
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
`tests/pruef/werkzeug_formdesigner.dhtest` (seit Stufe 32; bis dahin test_pilot_formdesigner.py -- echte Klicks ueber die Wiedergabe:
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
Tests `tests/pruef/werkzeug_animfsm.dhtest` (7, seriell; bis Stufe 33 pytest): Doppelklick, Rechtsziehen,
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
`tests/pruef/werkzeug_notenblatt.dhtest` (7, seriell; seit Stufe 33 ohne
Python -- das Tracker-Gitter ist dort ein fester, von `to_tracker_song`
aufgezeichneter Stand). Schluesselwechsel rueckt die
Noten ohne Dialog um Oktaven heran und sagt es (Strg+Z nimmt es zurueck).

## Python-Abbau, Weg D: Pruefsammlungen fuer `dhrt test` (2026-09-07)

`dhrt test` lief bisher nur Pruefprogramme (`*_pruefung.dh`, ein Programm
mit ASSERT). Die Golden-Tests der Sprache sind aber Ausgabevergleiche --
gemessen 1084 der 3949 pytest-Tests in 79 Dateien (`assert run_gb(src) ==
"..."`). Dafuer gibt es jetzt **Pruefsammlungen** `*.dhtest`
(`pruefsammlung.rs`: Parser + Bewertung mit Rust-Tests; Laeufer in
`main.rs::sammlung_laufen`): `=== Name` beginnt einen Fall, dann der
Quelltext, dann `--- erwartet` (zeilenweise, Leerzeilen am Ende zaehlen
beiderseits nicht; `--- erwartet ungefaehr` laesst Zahlen um 1e-6
abweichen -- fuer alles, was `pytest.approx` brauchte, weil libm auf drei
Systemen in der letzten Stelle anders rundet), `--- enthaelt`, `--- fehler` (Abbruch +
Teiltext der Meldung), `--- datei name` (Beilage neben dem Programm), `--- verzeichnis name`,
`--- umgebung`. Jeder Fall ist ein eigener `dhrt run` in einem eigenen
Verzeichnis, die Faelle einer Datei laufen parallel (bis 8 Faeden);
`--filter Text` waehlt Faelle. KEIN_FENSTER-Meldungen und (mit
`DHRT_OHNE_GRAFIK=1`) fehlende Grafik-Builtins heissen "uebersprungen",
wie in conftest.py. Doku `docs/werkzeuge.md`. **Die Sammlungen liegen unter
`tests/pruef/`**; die CI ruft `dhrt test tests/pruef` direkt auf (bis
2026-09-20 ueber einen pytest-Anker), das Format
prueft `tests/pruef/dhrt_test_format.dhtest` am echten Laeufer. **Umgezogen und
aus `tests/` geloescht: 73 pytest-Dateien, 74 Sammlungen mit 1540 Faellen**
(Stand 2026-09-07, `dhrt test tests/pruef` laeuft sie in ~6 s; die 74. ist
`rekursionstiefe.dhtest`, dessen pytest-Datei mit drei Tests bleibt, die
`vm.rs`/`build.rs` LESEN statt etwas laufen zu lassen). Der Weg dorthin war
nicht Quelltext-Umbau, sondern **Aufzeichnung**: ein Wegwerf-Plugin (nicht
im Repo) hing sich in pytest an `run_gb`/`run_all`, schrieb je Aufruf
Quelltext, Ausgabe bzw. Fehlermeldung und die Dateien in `tmp_path` mit, und
ZWEI Laeufe mussten dasselbe liefern -- so fielen `TIME$`, Dateizeiten und
alles Zufaellige von selbst heraus. Ein Modul zog nur um, wenn JEDER seiner
Tests bestanden hatte und aufgezeichnet war; Faelle, deren Test mit
`pytest.approx` oder Vergleichen arbeitete oder lange Kommazahlen ausgibt,
bekamen `--- erwartet ungefaehr`. **Der dritte Schritt holte die 14 Module
nach, die beim zweiten liegen blieben** -- drei Ursachen, drei Mittel:
(1) "kein Aufruf" bei `[tw]/[vm]`-Laeufen war ein Loch des PLUGINS, nicht
der Tests: die `run_either`-Fixture hielt die urspruengliche `run_gb` fest,
bevor das Plugin sie umwickelte -- `run_either` mit umwickeln, und
gleiche Aufrufe aus `[tw]` und `[vm]` (oder `run_gb` + `run_vm`) werden EIN
Fall. (2) Binaere Beilagen: **`--- datei name base64`** (Block darf
umbrochen sein; `Fall::dateien` traegt seither `Vec<u8>`) -- fuer die
cp1252-Dateien von ini/xml/kodierung und die ZIP-Slip-Archive, die Pythons
`zipfile` mit `../`-Namen baut und dhrt selbst nie schriebe. (3) Zufall
(`UUID4$`, `RANDOM_BYTES`), Uhr (`TIME$`, `ZEIT_JETZT`) und absolute
Pfade im Quelltext (zip, csv, tiled) wurden zu SELBSTPRUEFENDEN Faellen: das
Programm prueft die Eigenschaft (Form per `REGEX_TEST`, Eindeutigkeit ueber
eine MAP, Pfadende per `RIGHT$`) und gibt TRUE aus; die 256-KB-Datei der
Hash-Tests entsteht im Programm (`BUFFER_NEW` + `WRITEALL_BYTES`), der
Erwartungswert stammt weiter von `hashlib`. **Der vierte Schritt (2026-09-08)
holte die Grafik-Tests**: `--- bild` prueft Punkte am Bildschirmfoto (der
Laeufer setzt `DHRT_SCREENSHOT` selbst und `DHRT_FRAMES=2`, wenn die
Umgebung des Falls keins nennt -- das erste Bild liegt bei doppelter
Pufferung noch nicht im Bild) oder, mit Namen, an einer Datei, die das
Programm geschrieben hat: `groesse B H`, `X Y #RRGGBB [+-N]`, `X Y nicht
#RRGGBB [+-N]`, `X Y = X2 Y2`, `X Y <> X2 Y2`. Die Toleranz je Kanal ist
das, was die pytest-Proben mit `g > 180 and r < 80` meinten; die Logik
(`Probe`, `bild_pruefen`) liegt in pruefsammlung.rs mit Rust-Tests, das
Dekodieren in main.rs hinter dem Grafik-Feature ueber raylibs
`Image::load_image` (ohne raylib ist der Fall uebersprungen, nicht falsch --
und raylibs INFO-Zeile beim Laden muss man abschalten, sonst steht sie in
der Bilanz). Die uebrigen Grafik-Tests starten dhrt selbst per
`subprocess.run` mit `DHRT_FRAMES` -- der Aufzeichner wickelt seither auch
`subprocess.run` um (Quelle aus dem Pfad im Aufruf, `DHRT_*` aus dem env,
Dateien vorher/nachher; die `_gbtest_`-Aufrufe der Fixture nimmt er aus,
sonst durchsucht er das System-Temp und faellt an gesperrten Dateien).
Stand danach: 96 Sammlungen mit 1729 Faellen, 90
pytest-Dateien geloescht. **Der fuenfte Schritt (selber Tag) nahm den Rest
der Konsolen- und gui-Goldens auf einmal**: der Aufzeichner lief ueber ALLE
verbleibenden Nicht-Qt-Dateien (zweimal, 1460 Tests je Lauf), und was
stabil war, wurde Sammlung -- 28 Dateien ganz, 24 teilweise. Dazu zwei
Formatstuecke: **`--- ton datei.wav`** prueft eine WAV, die das Programm mit
`AUDIO_SAVE_WAV` geschrieben hat (`kanaele`, `bits`, `abtastrate`, `dauer S
+-T`, `spitze X +-T`, `pegel VON BIS <X | >X | X +-T` als Spitzenwert je
10-ms-Fenster des ersten Kanals -- die `_huelle` der pytest-Tests --,
`kanaele gleich|verschieden`; eigener RIFF-Leser `wav_lesen` in
pruefsammlung.rs, weil `sprache::wav_lesen` die Kanaele mittelt), und
**`--- seriell`** vor dem ersten Fall laesst die Faelle einer Datei
NACHEINANDER laufen -- gebraucht von gui_text_formular, dessen Faelle ueber
die Zwischenablage tippen: parallel pastete ein Fall den Text des anderen.
Was NICHT umzog, steht als Ausnahmeliste im Sammlungsbauer mit Grund je
Test: Benutzerordner/DH_PATH (bibliothek), MILLIS-Werte (builtins_uhr),
Dateiname oder Spalte in einer Meldung, BOM in der Quelle, fremde
Umgebungsvariablen, gemischtes stdout/stderr, Portamento (Nulldurchgaenge
zaehlen). **Zwei Funde aus der ersten CI dieses Schritts:** (1) die
Shell-Tests riefen `cmd /c` und trugen in pytest `skipif(sys.platform !=
"win32")` -- dafuer **`--- system windows`** (auch posix/macos/linux):
der Fall gilt nur dort, anderswo ist er uebersprungen, nicht falsch.
(2) Ein Bau ohne Audio meldet "im Rust-Kern noch nicht verfuegbar" --
und ein Fall, der die Meldung mit TRY/CATCH faengt und AUSGIBT, endet mit
0 und der Meldung auf STDOUT; die Ueberspringregel prueft darum beide
Kanaele und nicht mehr den Rueckgabewert. **Der sechste Schritt (selber
Tag) nahm die `run_gb_roh`-Tests**: `--- eingabe` (Standardeingabe fuer
INPUT/STDIN(), `base64` fuer Bytes, Leerzeile am Ende = Endumbruch; der
Laeufer schreibt sie in ein gepipetes stdin und schliesst es -- ohne den
Block bleibt stdin `null`), `--- argumente` (eine Zeile je Argument, hinter
`--`), `--- rueckgabe N` (EXIT(N); ohne Angabe weiter 0) und `--- stderr`
(jede Zeile in der Fehlerausgabe, OHNE Abbruch -- fuer EPRINT; `fehler`
verlangt weiter Exit ungleich 0). Zwei Fallen dabei: der Kommentar aus dem
Docstring stand VOR dem Quelltext und verschob jede Zeilennummer in einer
Meldung um eins (jetzt dahinter), und `_dedent` nahm die fuehrende Leerzeile
weg, die pytest mitgeschrieben hatte (jetzt bleibt sie). Stand danach:
151 Sammlungen mit 2403 Faellen in ~65 s, 118 pytest-Dateien geloescht. Nicht uebertragbar und darum in pytest geblieben (Stand damals; gui_table_frozen_edge, gui_draw_window, gui_bindung und schriften_vorrat zogen am 2026-09-16 doch um):
gui_table_frozen_edge (Kantenprobe mit Schwellwert), gui_draw_window und schriften_vorrat (vergleichen zwei Laeufe miteinander), gui_bindung (SQLite-Datei), die 13 image_io-Tests mit Pillow als fremdem Leser fuer PNG/BMP/GIF, drei Shader-Bildproben, der Instancing-Render in m3d, drei Kontaktbogen-Tests und ein Index-Leser in input_edges. Bewusst NICHT umgezogen:
test_dateisystem.py (Dateizeiten, Gross/Klein je System -- seit 2026-09-16 doch
umgezogen, siehe unten), alles mit Ton
oder echter Eingabe-Wiedergabe (seriell), und Tests, die Quelltext oder
Bauskripte lesen. Regel fuer den weiteren Umzug: **eine pytest-Datei wird geloescht,
sobald ihre Faelle in einer Sammlung liegen** -- nie beides pflegen; ein
Verweis in der Doku wandert mit (`dhrt pruef pfade` findet ihn). Ein
Golden aus einer Aufzeichnung ist STRENGER als der Test davor (er prueft die
ganze Ausgabe, nicht eine Eigenschaft) -- wer einen solchen Fall aendert,
aendert die Erwartung bewusst. **Betriebssystem-Fehlertexte sind lokalisiert**
("Das System kann die angegebene Datei nicht finden" hier, "No such file or
directory" auf den Laeufern, und der os-error-Code weicht auch ab) -- in
`--- fehler` steht darum nur der eigene Teil der Meldung (Befehl, Pfad); die
ersten CI-Laeufe fanden genau zwei solche Faelle unter 1230. Und ein
Fehlerfall traegt den zufaelligen Namen der pytest-Testdatei
(`_gbtest_xxx.dh: Compile-Fehler: ...`) -- der Vergleich zweier Laeufe muss
ihn vorher abschneiden, sonst gilt jeder Compile-Fehler als instabil.

## Python-Abbau, Weg C: die IDE in Drachenhauch (Stufe 1 bis 3, 2026-09-06)

`ide/ide.dh` (~760 Zeilen) ist die IDE als Drachenhauch-Programm, gestartet
mit `dhrt run ide/ide.dh [-- datei.dh]`; Doku `docs/ide.md`. Stufe 1: Reiter
mit Code-Feldern, Projektbaum, Einfaerbung des sichtbaren Ausschnitts,
Fehlerliste (0,6 s nach der letzten Aenderung), Hilfe zum Wort,
Vervollstaendigung, Suchen/Ersetzen/Gehe zu Zeile, Zur Definition, Starten
mit laufender Ausgabe und Eingabezeile. **Drei Bausteine kamen dafuer in
dhrt** -- alle ohne IMPORT: (1) `prozess.rs` + `PROCESS_START/READ$/ERR$/
WRITE/CLOSE_INPUT/RUNNING/CODE/KILL/CLOSE` (Faeden lesen stdout/stderr
zeilenweise in Puffer, wie `WINDOW_RECV$`; "dhrt" als Programm = eigene Exe;
`DHRT_FRAMES`/`SCREENSHOT`/`CONTACT*` werden dem Kind genommen; Windows
`CREATE_NO_WINDOW`). **Falle:** ein `dhrt run`-Kind an einer Leitung sammelt
PRINT in `self.out` und schreibt erst am `flush_out()`-Punkt -- die IDE
saehe alles am Ende auf einmal. `PROCESS_START` setzt deshalb `DHRT_LIVE=1`,
und die VM flusht dann nach jeder PRINT-Zeile (`live_ausgabe()` in vm.rs,
einmal gelesen). (2) `CODE_CHECK$/HOVER$/COMPLETE/DEFINITION/REFERENCES/
SYMBOLS$` = derselbe Kern wie `dhrt lsp` (`lsp::diagnose` usw.), Positionen
1-basiert, Ergebnisse als JSON-Text bzw. ARRAY/TUPLE. (3) `GUI_TEXTAREA_
CURSOR/GOTO/SELECTION$/SELECT/INSERT/FIND` (gui.rs nach `textarea_view`;
GOTO/SELECT brauchen Grafik, weil sie den sichtbaren Ausschnitt nachziehen;
INSERT ersetzt eine Auswahl, schreibt Undo und feuert on_change). **Fallen
in der IDE:** `dhrt run` wechselt ins Verzeichnis der Quelle (`ide/`) -- den
Ort des Aufrufers hinterlegt dhrt seit 2026-09-06 als `DHRT_START_DIR`
(`ins_quellverzeichnis` in main.rs, alle vier chdir-Stellen), die IDE nimmt
ihn fuer Projektbaum und relatives Argument; `ARG$` ist 0-basiert (`ARG$(0)` = erstes Argument nach `--`);
`GUI_WINDOW_VISIBLE` hat keinen Getter -> eigenes Flag fuer das
Vervollstaendigungs-Fenster; eine Wiedergabe-Taste bleibt gedrueckt, bis
KEY_UP kommt (F5 = 294, F7 = 296). Tests: `tests/pruef/ide_bausteine.dhtest`
(Bausteine einzeln, Fenster ausserhalb des Schirms; bis Stufe 34 pytest) und `tests/test_ide.py`
(die IDE ueber `DH_IDE_LOG` + `AUTOMATION_PLAY`, seriell). Offen: Debugger/
Profiler-Fenster, Suche im Projekt, Befehlspalette, Handbuch, Drucken,
Installer ohne PyInstaller -- dann Weg B (Editoren), D nebenher.

**Stufe 2 (selber Tag):** Debugger als Client von `dhrt debug` (JSON-Zeilen:
stdout Ereignisse `paused`/`output`/`finished`/`error`, stdin Kommandos --
das Kind liest sie NUR, solange es steht; erster Halt ist Zeile 1, dort
schickt die IDE `set-breakpoints` und `continue`, wenn es Haltepunkte gibt,
sonst bleibt sie stehen), Profil aus `dhrt profile` (EINE JSON-Zeile am
Ende; serde sortiert die Schluessel, also NICHT am Anfang `total_time`
erkennen), Suche im Projekt (teilt sich die Liste mit den Problemen,
`problemModus`), Befehlspalette (alle Befehle laufen ueber EINE `befehl(k$)`
-- Menue, Kuerzel, Knoepfe, Palette), Marken in der Nummernspalte
(`GUI_TEXTAREA_MARKS(ta, zeilen, farben)`, gui.rs `marken`: Punkt + Farbhauch,
an der Zeilennummer, nicht am Text), Schriften (Segoe UI/Consolas, 32 px
geladen, 16 gezeichnet, `GUI_SET_FONT` je Code-Feld), Start `WINDOW_MAXIMIZE`,
Vollbild Alt+Enter. Kuerzel wie die Qt-IDE: F7 Debuggen (Pruefen jetzt
Umschalt+F7), F8/F10/F11/Umschalt+F11, F9 Haltepunkt, Strg+Umschalt+F/P/Y.
**Vier Fallen:** (1) `DIM a AS ARRAY OF T` ohne Groesse ist KEIN Feld --
`ARRAY_PUSH` meldet "erwartet ARRAY"; erst `a = []`. (2) Widget-Koordinaten
zaehlen ab dem INHALT unter Menue- und Reiterleiste (56 px) -- Stand 1
rechnete ab dem Fensterrand, Eingabezeile und Status lagen unter dem Rand,
und kein Test sah es, nur das Bild. (3) Ein zweites GUI-Fenster liegt hinter
dem bildschirmfuellenden Hauptfenster, und Menue-Kuerzel gelten nur im
Fenster mit Fokus -- das Debugger-Panel liegt darum IM Hauptfenster (rechts
unten statt der Problemliste); das Profilfenster holt `GUI_FOCUS(tblProfil)`
nach vorn. (4) `flush_out` schreibt im Debugger NICHT mehr roh (stdout ist
dort der Ereigniskanal; `DHRT_LIVE` haette PRINT-Zeilen hineingemischt).
Tests in `tests/test_ide.py` (Haltepunkt in Zeile 2 -> Halt in 2, nicht 1;
F10 -> 3; F8 -> Ende; ohne Haltepunkt Halt in 1; Profil; Projektsuche und
Palette tippen ueber die Zwischenablage, `CLIPBOARD_SET` wird in die
Testkopie eingeschoben -- die Kopie liegt darum in einem Unterordner,
sonst zaehlt die Suche sie mit).

**Stufe 3 (selber Tag):** Handbuch im Fenster (F1 = Wort unter der Marke in
ALLEN `docs/*.md` suchen, das Dokument mit den meisten Fundstellen in
Codeschrift gewinnt; Klappliste + Suche im Dokument; Textbereich mit
`umbruch`), Listing drucken (`PDF_PRINT` auf den Standarddrucker) oder als
PDF neben die Quelle (Courier 9 pt, 66 Zeilen/Seite, `PDF_TEXT` je Zeile in
TRY -- cp1252-fremde Zeichen sind dort ein Fehler), Werkzeuge-Menue (die
fuenf Piloten als eigene `dhrt run`-Prozesse in `werkzeuge[]`, beim Ende
gekillt), Ausdruck-Auswertung im angehaltenen Debugger (Eingabezeile ->
`{"cmd":"eval"}`, Strg+E holt sie), und `installer/Drachenhauch-IDE.iss`:
Installer OHNE Python (dhrt.exe + ide/ + docs/ + examples, 33 MB statt 92,
eigene AppId, Verknuepfung `dhrt run ide\ide.dh` mit den Beispielen als
Arbeitsverzeichnis, `.dh` oeffnet in der IDE). Ordner: `docs/` und
`examples/` neben `ide/` (`DH_IDE_WURZEL` uebersteuert -- die Tests
setzen es, weil ihre Kopie woanders liegt), Beispiele sonst unter
`%PUBLIC%\Documents\Drachenhauch\examples`. **Falle:** ISCC-Schalter
(`/DAppVersion=...`) NICHT aus Git Bash aufrufen -- MSYS schreibt sie in
Pfade um ("more than one script filename"); PowerShell nehmen.

**Circuit-Runner-Werkzeuge in Drachenhauch (2026-09-14, Teil 1 von 2):** auf
Wunsch des Nutzers sind die Python-Werkzeuge des Spiels portiert --
`circuitrunner/convert_dat.dh`, `make_demo_levels.dh`, `download_sfx.dh` und
`download_music.dh`; die `.py`-Dateien und `test_circuitrunner.py` sind
geloescht. **Die Messlatte war Byte-Gleichheit mit Python, nicht "sieht gleich
aus":** der Konverter liefert auf den 149 Leveln eines lokalen `CCLP1.dat`
dieselbe Datei wie `convert_dat.py`, der Demo-Bauer genau das eingecheckte
`levels/circuit_runner.json` (`JSON_STRINGIFY` behaelt die Einfuegereihenfolge
und schreibt kompakt wie Pythons `separators=(",", ":")`), die Downloads
schreiben dieselben `CREDITS.txt`, und die Adressen sind wie
`urllib.parse.quote` kodiert (Leerzeichen und eckige Klammern). **Zwei Funde
beim Portieren:** (1) eine lokale Variable `n` in `rleDecode` verdeckte die
Konstante `N` -- Namen sind schreibungsunabhaengig, die Schleife `n < N` lief
nie, jedes Level war leer; `--check` hatte es als Warnung gesagt. (2) Die
Zeichentabelle des Demo-Bauers ist eine Zeichenkette mit `INSTR`, keine MAP:
`O`/`o`, `B`/`b`, `L`/`l`, `T`/`t`, `M`/`m`, `P`/`p` meinen verschiedene
Kacheln. Tests: sieben neue Faelle in `tests/pruef/werkzeug_circuitrunner.dhtest`
-- der Konverter an einer .dat, die der Fall selbst schreibt (zweiter
RLE-Schreiber), Lynx-Kennung, fremde Datei, fehlende Argumente; der Demo-Bauer
gegen das Schema UND byte-gleich gegen die eingecheckte Datei; beide Downloads
gegen `gegenserver.dh` im echo-Modus, der den Pfad jeder Anfrage zurueckgibt.
**Gegenprobe mit gezielt kaputten Kopien der Werkzeuge:** alle sieben fallen --
aber erst im zweiten Anlauf. Drei Mutationen des ersten Anlaufs erreichten ihren
Fall gar nicht: eine falsche Lynx-Kennung fing die Suchschleife des Konverters
wieder auf, und ein ungeschuetztes Leerzeichen in der Musik-Adresse kodiert der
HTTP-Client selbst -- erst eckige Klammern kommen roh beim Server an. Beim
Schreiben der Tests fiel noch ein Stolperstein: `VAL("&H" + hex)` ist 0,
Hex-Text liest man ueber `INSTR` in `"0123456789ABCDEF"`. **Offen (Teil 2):**
`make_tiles.py` -- es zeichnet mit PIL gefuellte Polygone, dicke Linien,
abgerundete Rechtecke, Ringe und Verlaeufe auf 256x256 und skaliert mit
LANCZOS; die Bild-Befehle der Laufzeit koennen davon nur Kreise, Rechtecke und
1-px-Linien, und jeder Aufruf laedt die Textur neu hoch.

**Teil 2 (selber Tag): `make_tiles.dh` und acht neue Bild-Befehle.** Statt die
Formen in Drachenhauch Punkt fuer Punkt nachzubauen, zeichnen jetzt
`IMAGE_FILL_RECT`, `IMAGE_FRAME`, `IMAGE_FILL_ROUNDRECT`, `IMAGE_FILL_ELLIPSE`,
`IMAGE_RING`, `IMAGE_FILL_POLY`, `IMAGE_POLYLINE` und `IMAGE_GRADIENT` direkt in
die Bildpunkte (`leinwand.rs`, 13 Rust-Tests): Kommazahlen, optionale Deckkraft
0..255 (die Farbe allein kann "durchsichtig" nicht ausdruecken), und sie
UEBERSCHREIBEN wie PILs ImageDraw auf RGBA. Hochgeladen wird die Textur erst
beim FLIP (`Graphics::image_leinwand` + `tex_veraltet`), nicht je Aufruf --
gezeichnet wird ohnehin nur dort aus der Textur. **Der Fund dabei:
`IMAGE_DRAW_IMAGE` mischte ueber halbdurchsichtigem Ziel falsch** -- raylibs
`ImageDraw` machte aus Orange (Deckkraft 60 ueber 59) ein Gruen mit Rot 0x01,
sichtbar als gruenlicher Schein um Schluessel und Stiefel. Es mischt jetzt
selbst nach Porter-Duff (`Leinwand::bild_ueber`); ueber deckendem Grund aendert
sich nichts, `image_io` und `werkzeug_sprite` blieben gruen. **Verglichen wurde
Zelle fuer Zelle mit dem alten Blatt:** `tiles.json` ist byte-gleich, die
Deckung weicht je Kachel um hoechstens 6 % ab, die mittlere Farbe nur bei den
Kacheln mit weichem Schein (bis 35 je Kanal -- PIL verwischt gegen
durchsichtiges Schwarz und dunkelt den Rand ab, raylibs Weichzeichner nicht).
`tiles.dhsprite` entsteht nicht mehr (das Format des Qt-Sprite-Editors) und ist
mit `make_tiles.py` geloescht. Tests `tests/pruef/image_formen.dhtest` (7,
Gegenprobe ohne die Aufrufe: 7 von 7 fallen) und ein Fall "kachelsatz wird
gezeichnet" in `werkzeug_circuitrunner.dhtest`.

**Download-Programme der Beispiele (selber Tag):** die sechs `.py`-Downloader
neben den Beispielen sind Drachenhauch -- `examples/assets/download_cybermatic.dh`,
`download_hdri.dh`, `download_robot.dh`, `download_techno.dh` (gemeinsamer Teil
`examples/assets/_laden.dh` per `IMPORT`; `dhrt pruef beispiele` zaehlt nur die
obere Ebene von `examples/`, ein Helfer darunter ist kein Beispiel),
`fireplace/assets/download.dh` und `gbdemo/download_music.dh`. Adressen,
Mindestgroessen und Meldungen wie zuvor; gbdemo laedt in eine `.teil`-Datei und
speichert nur, was die XM- oder MOD-Kennung traegt. Optionale Argumente
(Adresse, Ziel, Mindestgroesse) sind fuer Pruefungen da. Der Gegenserver stellt
im echo-Modus Pfaden ab `/xm` "Extended Module: " voran, damit der Erfolgsweg
pruefbar ist. Tests `tests/pruef/werkzeug_downloads.dhtest` (3; Gegenprobe mit
kaputten Kopien -- Mindestgroesse ignoriert, Namen vertauscht, Kennung nicht
geprueft: 3 von 3 fallen). **Stolperstein beim Testschreiben:** in
`PRINT PROCESS_CODE(p); lauf$(p)` wird der Rueckgabewert VOR dem Warten gelesen
(-1), und die Ausgabe eines Kindes endet mit einem Umbruch, den `PRINT` doppelt.
**Asset-Erzeuger der Beispiele (selber Tag):** `examples/assets/make_demo_mod.dh`
und `make_pluck_sample.dh` schreiben Byte fuer Byte das eingecheckte `demo.mod`
und `pluck.wav` (vorher gegen die Ausgabe der `.py`-Fassungen geprueft, beide
gleich). Das Zupf-Sample braucht dafuer **Pythons Zufall nachgebaut**: MT19937
mit `random.seed(42)` (init_by_array mit `[42]`) und `random.random()` (53 Bit
aus zwei Worten) -- mit `RND` kaeme ein anderes Rauschen. 64-Bit-Ganzzahlen
reichen fuer alle Multiplikationen, `BAND &HFFFFFFFF` maskiert auch negative
Zwischenwerte richtig; Pythons `int()` rundet Richtung null, `INT` ab.
**Zwei Stolpersteine:** ein Feld von Feldern (`DIM a[4] AS ARRAY OF INTEGER`)
uebersetzt nicht, und eine MAP laesst sich nicht mit `m[schluessel]` lesen --
`MAPGET`. Tests `tests/pruef/werkzeug_beispiel_assets.dhtest` (2; Gegenprobe
mit einer anderen Periode bzw. Saat 43: beide fallen).
**Level-Werkzeuge von Pyramid Pusher (selber Tag):**
`pyramid_pusher/levels/_authoring/make_levels.dh` (die 12 eigenen Level samt
Loeser, der jeden Level vor dem Schreiben per Breitensuche ueber Schiebezuege
prueft) und `convert_skinner.dh` (366 Skinner-Level ins Titel-vor-Brett-Format,
sortiert nach Sammlung, roemischer Set-Nummer und Levelnummer, mit
`SORT(feld, vergleich)`, das stabil ist wie Pythons `sort`). Beide schreiben die
eingecheckten `.xsb` Byte fuer Byte; in `02_skinner_classics.xsb` aendert sich
nur die Kopfzeile, die das Werkzeug nennt. Die Quelle traegt 40 Latin-1-Bytes (je ein
`é` = 0xE9 in den 40 Copyright-Zeilen, die das Werkzeug ueberspringt) -- gelesen
mit `READLINES(pfad, "latin1")`, als UTF-8 waere die Datei nicht lesbar. Tests
`tests/pruef/werkzeug_pyramid_levels.dhtest` (3, darunter ein per
`--- ersetzen` unloesbar gemachter Level, der nicht geschrieben werden darf;
Gegenprobe: geaenderter Titel, Loeser sagt immer "ok", Sortierung ohne
Set-Nummer -- 3 von 3 fallen).
**Tileset des Tilemap-Editors (selber Tag):** `examples/assets/make_editor_tileset.dh`
statt der PIL-Fassung. Das Rauschen kommt aus demselben nachgebauten
`random.Random(20260604)` wie beim Zupf-Sample, dazu `randrange`/`randint`
ueber `getrandbits` mit Verwerfen -- alle Kacheln mit Rauschen sind Punkt fuer
Punkt die alten. **PIL nimmt bei einer Ellipse beide Randpunkte des Rechtecks
mit**: mit dem Radius als halbem Mittelpunktabstand wichen 479 Punkte ab, mit
einer Zugabe von 0.4 (gefuellt) bzw. 0.5 (Ring) nur noch 82, und 25 von 32
Kacheln sind gleich. Der Rest liegt in Vielecken, dem abgerundeten Fass und dem
Holzring -- PIL rastert dort anders als `leinwand.rs`. Das Blatt ist mit dem
neuen Programm neu geschrieben; die Tilemap-Pruefungen zaehlen nur Farben
gegeneinander und haengen nicht an den alten Punkten. Test
`tests/pruef/werkzeug_beispiel_bilder.dhtest` (Punkt fuer Punkt gegen das
eingecheckte Blatt; Gegenprobe mit Zugabe 0.0 fuer gefuellte Ellipsen: 193
Punkte weichen ab, der Fall faellt).
**Buch-Sprites (selber Tag):** `buch-galaga/assets/make_sprites.dh` und
`buch-einstieg/assets/mach_sprites.dh` zeichnen ihre Buchstabenraster Punkt fuer
Punkt wie die PIL-Fassungen -- dort gibt es keine Ellipsen, nur Einzelpunkte,
harte Vergroesserung und Einsetzen, also ist alles gleich (Galaga: sechs Sheets
plus `preview.png`; Einstieg: drei Sprites plus zwei Lehrbilder). Galagas
`.dhsprite`-Dateien (Format des Qt-Sprite-Editors) schreibt das neue Programm
nicht mehr, die eingecheckten bleiben liegen. **Die Lehrbilder im Einstiegsbuch
sind nicht die Ausgabe des Werkzeugs:** `make_book.py` (`prepare_images`) hat
sie fuer den Druck auf RGB gebracht und ganzzahlig auf mindestens 1500 Punkte
Breite vergroessert (4x und 3x) -- der Test vergroessert genauso und vergleicht
dann. Tests zwei Faelle in `tests/pruef/werkzeug_beispiel_bilder.dhtest`
(Gegenprobe: eine geaenderte Palettenfarbe bzw. ein Punkt im Schiff -- beide
fallen).
**Plattformer-Sprites (selber Tag):** `examples/platformer/make_sprites.dh`
statt der PIL-Fassung. Die Pixel-Leinwand (Punkte, Rechtecke, Kreisformel
`dx²+dy² <= r²+0.4r`, Umriss ueber vier Nachbarn) ist mit Pythons `int()`
(Richtung null) nachgebaut -- alle acht Figurenstreifen, der 16x16-Spieler-Atlas
und beide JSON-Dateien sind gleich (Pythons `write_text` schrieb unter Windows
CRLF, git haelt LF). **PIL verkleinert 32->16 mit dem Punkt (2x+1, 2y+1),
`IMAGE_SCALE_NN` mit (2x, 2y)** -- der Atlas tastet darum von Hand ab. Im
Master-Sheet weichen nur Stern (85 Punkte) und Fahne (9) ab, beide PIL-Vielecke;
ein nachgebautes Pillow-Scanline mit acht Varianten traf keinen genau, und im
Bild sind es einzelne Randpunkte. GIFs (je Bild 120 ms) und der Kontaktbogen
(Schrift der Laufzeit) sind neu geschrieben, die `.dhsprite`-Dateien nicht mehr.
Test `tests/pruef/werkzeug_platformer_sprites.dhtest`.
**Pyramid-Pusher-Kunst (selber Tag):** `pyramid_pusher/assets/gen_art.dh` statt
`gen_art.py` (PIL + numpy). Die Koernung kam aus `np.random.default_rng(seed)
.integers(lo, hi)`, und die ist nachgebaut statt ersetzt: SeedSequence (hashmix
/mix, acht 32-Bit-Woerter), PCG64 (128-Bit-Zustand, XSL-RR, erst die unteren 32
Bit einer Ausgabe, dann die oberen) und Lemires Verfahren fuer die Spanne.
**Ein INTEGER laeuft nicht still ueber, er bricht ab** -- der Zustand liegt in
acht 16-Bit-Stuecken, 32-Bit-Produkte gehen ueber `mal32`. Dazu Pythons
`round()` (bei genau .5 zur geraden Zahl). Ergebnis: 15 von 16 Bildern Punkt fuer
Punkt gleich, samt Held-Sheet und 512er-Lichtmaske (0,7 s); der Stern (PIL-Vieleck)
weicht an 17 Randpunkten ab -- mit abgeschnittenen Eckpunkten, mit Kommazahlen
waren es 55. Eingecheckt ist nur der neue Stern, die uebrigen PNGs sind
unveraendert. Test `tests/pruef/werkzeug_pyramid_art.dhtest` (alle 16 Bilder
gegen die eingecheckten; die ersten Zuege zweier Startwerte gegen numpys Zahlen).
**Cloud-Server (2026-09-15):** `cloudserver/server.dh` statt des
Flask-`server.py` -- `httpd` + `db` + `json`, dieselben Pfade, Fehlercodes und
dasselbe SQLite-Schema (eine vorhandene `cloud.db` laeuft weiter). **Die Luecke
dabei: `httpd` konnte keine eigenen Kopfzeilen setzen**, die CORS-Freigabe fuer
Spiele im Browser war also nicht nachzubauen. Neu `HTTPD_SET_HEADER(s, name$,
wert$)` (httpd.rs `kopfzeile_setzen`/`kopf_text`, Rust-Test): gilt fuer JEDE
folgende Antwort, nicht nur die naechste -- sonst muesste jede Antwortstelle
daran denken; Zeilenumbruch im Wert ist ein Fehler (Kopfzeilen-Einschleusung),
`Content-Type/Length/Connection` setzt `HTTPD_SEND` selbst. Zwei bewusste
Abweichungen: `updated_at` ist ganzzahlig (`ZEIT_JETZT`, es gibt keine
Unix-Kommazahl), und die CORS-Vorabfrage `OPTIONS` wird VOR der
Schluesselpruefung beantwortet -- der Browser schickt dabei keinen `X-Api-Key`,
die Flask-Fassung wies sie mit gesetztem Schluessel ab. Der Port geht als
`PORT=...` per EPRINT hinaus (Port 0 = frei). Test
`tests/pruef/werkzeug_cloudserver.dhtest` (5: alle Zusagen aus `test_server.py`
und mehr gegen EINEN Server, offener Modus mit Warnung, CORS an Vorabfrage und
Abweisung, das `cloud`-Modul Ende zu Ende, Neustart mit derselben Datenbank);
Gegenprobe ohne Schluesselpruefung: 3 von 5 fallen, ohne die Kopfzeilen der
CORS-Fall. `server.py`, `test_server.py` und `requirements.txt` sind geloescht.
Damit laeuft bei den Spielen nichts mehr ueber Python. Im selben Zug
`bench_dhrt.dh` statt `bench_dhrt.py` (Bestwert aus N Laeufen von `dhrt run`,
Ausgabe in derselben Form; ohne `--dhrt` misst er die Laufzeit, die ihn
ausfuehrt). Die Ausgabe des Kindes wird waehrend des Laufs geleert -- ein voller
Puffer hielte es an und verfaelschte die Zeit. Test
`tests/pruef/werkzeug_bench.dhtest` (Form der Ausgabe plus Fehlerlauf;
Gegenprobe ohne Pruefung des Rueckgabewerts faellt).
**Buch-Werkzeuge (selber Tag):** `buch-referenz/buch/shoot.dh`,
`buch-einstieg/buch/shoot.dh` und `buch-referenz/buch/i18n/apply.dh` statt der
`.py`-Fassungen. Aufgenommen wird ueber `dhrt bild` (PROCESS_START nimmt dem
Kind DHRT_FRAMES/SCREENSHOT ab, `DHRT_SCALE`/`DHRT_FONT` gehen per SETENV
durch). **Zwei Funde:** (1) `apply.py` schrieb unter Windows CRLF, der
eigentliche Schreiber `extract_strings.js` aber `JSON.stringify(k, null, 1)`
mit LF -- jeder Python-Lauf brach alle 3000 Zeilen von `en.json` um.
`apply.dh` schreibt im JS-Format (Maskierung ueber `JSON_APPEND_STRING` +
`JSON_STRINGIFY`, die gleich maskieren); ein leerer Patch laesst `en.json` Byte
fuer Byte stehen. Gelesen wird von Hand ueber BUFFER_GET: die Schluessel sind
deutsche Saetze mit PUNKT, und das json-Modul liest einen Punkt als Pfad.
(2) **`dhrt bild` endete nicht**: das Bildlimit wirkt nur ueber
`QUITREQUESTED()`, ein Programm mit `WHILE TRUE` sicherte sein Bild und lief
endlos weiter -- der Aufrufer wartete ewig (so haengte der erste Testlauf 300 s).
`dhrt bild` setzt jetzt `DHRT_BILD_ENDE`, und graphics.rs beendet den Prozess
direkt nach dem gesicherten Bild. Tests `tests/pruef/werkzeug_buch.dhtest`
(apply gegen Node-Referenz und leerer Patch auf eine Kopie von `en.json`, beide
shoot-Werkzeuge an Beilagen) und `tests/pruef/dhrt_bild.dhtest` (der Test aus
`test_dhrt_werkzeuge.py`, seit 2026-09-16 `tests/pruef/dhrt_werkzeuge.dhtest`, plus der `WHILE TRUE`-Fall).
**Buch-Build (selber Tag):** `tools/buch_bauen.dh -- <buchordner> [--lang de|en]`
ersetzt die vier `make_book.py` (PyMuPDF + PIL). Ein Werkzeug fuer alle vier
Buecher, je Buch Dateiname, Sprachen und Messart in einer Tabelle; Referenz- und
Einstiegsbuch messen genau (Textstueck mit Schriftgroesse >= 15, monoton
vorwaerts, sonst die erste Seite mit dem Text), Galaga und Tippspiel einfach.
**Das PDF liest es selbst** -- Objekte, Seitenbaum samt geerbten Ressourcen,
ToUnicode-Tabellen (codespacerange, bfchar, bfrange), Textbefehle mit `Tf`.
**Der Beleg ist gemessen, nicht behauptet:** `--nur-messen` an den vier
vorhandenen PDFs (Lehrbuch de/en mit 499/490 Seiten in je ~6 s, Einstieg,
Tippspiel) schreibt Byte fuer Byte die eingecheckten `toc_pages`-Dateien;
Gegenprobe mit Schwelle 100 statt 15: fuenf Seitenzahlen weichen ab. Zwei
Unterschiede der Schreiber, die das PDF des pdf-Moduls (krilla) im Test
aufdeckte: LibreOffice schreibt Hex-Zeichenketten `<01>` mit einem Byte je
Zeichen, krilla Literale mit zwei Bytes und Oktal-/`\b`/`\f`-Folgen -- beides
geht jetzt. Das PDF wird nach Pass 2 noch einmal gerendert (vorher endete das
Referenzbuch beim `.docx`). **Dafuer neu in der Laufzeit:
`IMAGE_SAVE(bild, pfad$ [, mit_alpha])`** -- mit `FALSE` ohne Alphakanal
(PNG-Farbart 2): Druckdienste lehnen ein PNG mit Alphakanal ab, auch wenn jeder
Punkt deckend ist; die Deckkraft wird weggelassen, nicht verrechnet
(`--nur-bilder` legt darum vorher auf Weiss). Test
`tests/pruef/werkzeug_buch_bauen.dhtest` (Messen an einem PDF aus dem
pdf-Modul: Ueberschrift gegen fruehere Erwaehnung, Gross/klein, fehlender Titel,
beide Messarten; Bilder: Farbart, Breite, Weiss unter Durchsichtigem, fertiges
Bild unberuehrt). PyMuPDF ist am 2026-09-20 ganz aus
requirements.txt gefallen: kein Test importiert es mehr, seit
`tests/pruef/_hilfen/pdftext.dh` auch fremde PDFs liest.

**Showcase in Drachenhauch (2026-09-15):** die kuratierte Beispiel-Galerie
steht in `examples/showcase.json` (file/title/desc/frames) statt in
editor_qt/showcase.py -- Qt-Panel, IDE und Erzeuger lesen dieselbe Datei.
`tools/showcase_bilder.dh [-- --wurzel DIR] [name ...]` ersetzt
`gen_showcase_thumbs.py`: `dhrt bild` je Eintrag, auf 480 Punkte Breite mit
`IMAGE_SCALE` (bilinear statt LANCZOS, also nicht punktgleich), gesichert mit
`IMAGE_SAVE(..., FALSE)` ohne Alpha; Rueckgabe 2, wenn nicht alle entstehen.
Die IDE zeigt die Liste als Karten (Bild, Titelknopf, Beschreibung, Tooltip),
bis zu 32, blaettert per Mausrad reihenweise (`kacheln reihe N`), nimmt
`examples/screenshots/<name>.png` vor der eigenen `dhrt bild`-Vorschau und
faellt ohne Liste auf die acht festen Kacheln zurueck (Geometrie der ersten
Karte unveraendert -- der alte Klick-Test trifft weiter). **Falle:** `LEN` auf
einer MAP ist ein Laufzeitfehler, gezaehlt wird selbst. Tests
`tests/pruef/werkzeug_showcase.dhtest` (5; Gegenprobe mit Alpha/Breite 400
und ohne showcase.json: die jeweiligen Faelle fallen).

**Die dhrt-Werkzeuge pruefen sich selbst (2026-09-16, Weg D):**
`tests/test_dhrt_werkzeuge.py` (31 Tests) und `tests/test_dhrt_call.py` (9) sind
geloescht; ihre Faelle stehen in `tests/pruef/dhrt_werkzeuge.dhtest` (21) und
`tests/pruef/dhrt_call.dhtest` (6). Gestartet wird dhrt als Kind ueber den neuen
Helfer `tests/pruef/_hilfen/dhrtlauf.dh` (`dlStarten(["fmt", ...])` fuellt
`dlAus`/`dlErr`/`dlCode`; `PROCESS_START` nimmt seine Argumente einzeln, ein
Feld laesst sich nicht ausbreiten -- daher die Staffel nach der Argumentzahl bis
acht). Der Sweep "der Bestand bleibt unter fmt ruhig" buendelt **sechs Dateien je
Aufruf** statt einer: mit 210 Prozessstarts dauerte die Sammlung 34 s, so 6 s.
**Der Fund dabei, und er steckt in der Laufzeit:** `PROCESS_START` setzt seinen
Kindern `DHRT_LIVE=1` (damit PRINT laufend ankommt) -- und damit verlor
`dhrt call` genau die Trennung, fuer die es gebaut ist: die Ausgabe der Funktion
ging roh nach stdout und das Feld `ausgabe` blieb LEER. Gemerkt hat es niemand,
weil Pythons `subprocess` die Variable nicht setzt; unter TASK_START (Auftrag als
Kindprozess) waere es der Normalfall gewesen. `dhrt call` sammelt jetzt
ausdruecklich (`Vm::sammle_ausgabe`, geprueft in `flush_out` neben Profiler und
Debugger).

**Dieselbe Runde, die Buch-Pruefungen:** `test_buch_struktur.py`,
`test_buch_verweise.py` und `test_buch_code.py` sind geloescht; ihre Faelle
stehen in `tests/pruef/buch_pruefungen.dhtest` (5). Die Kapitel sind
JavaScript-Module -- gestartet werden die Werkzeuge unter `tools/` und
`buch-referenz/buch/` ueber `tests/pruef/_hilfen/nodelauf.dh`
(`nlStarten(ordner, skript)` laeuft IM Ordner, weil sie ihre Dateien relativ
suchen; `zahlVor(text, wort)` liest die Bilanzzahl, die Schranke gegen den
leeren Lauf). Ohne Node meldet der Fall `UEBERSPRINGEN: node fehlt` und gilt
als uebersprungen -- **dafuer kann sich ein Fall seit dieser Runde selbst
ueberspringen** (die Zeile auf stdout oder stderr, `bewerten` in
pruefsammlung.rs, Doku `docs/werkzeuge.md`): fuer fremde Werkzeuge, die nicht
ueberall liegen. Die Alternative waere, die erwarteten Zeilen zu ERFINDEN --
gruen und wertlos. Dazu test_beispiele_gui_enden.py (pytest) -> `tests/pruef/beispiele_gui_enden.dhtest`
(zehn Fenster-Programme, 8 s) -- und dafuer **`dhrt run --bilder N`**: dieselbe
Grenze wie `DHRT_FRAMES`, nur als Schalter, weil `PROCESS_START` seinen Kindern
die Variable abnimmt (sonst stuerbe ein gestartetes Spiel nach N Bildern). Ein
Programm mit `WHILE TRUE` laeuft trotzdem weiter -- genau daran erkennt man es;
die Gegenprobe (eine Zeile im Beispiel umgestellt) laesst den Fall nach 25 s
fallen.
Dazu test_dhrt_check.py (pytest, 18 Tests) -> `tests/pruef/dhrt_check.dhtest`
(9): die Diagnosen von `dhrt --check` samt Zeilennummern, die Warnungen
(unbekanntes Builtin, Hardware-IMPORT, doppeltes DIM mit anderem Typ) und die
DURCHLAEUFE ueber den Bestand -- alle Beispiele ohne Fehler und ohne
"Unbekanntes Builtin", alle 390 `.dh` des Repos ohne Falschmeldung der
"nirgends angelegt"-Warnung (26 s; `dlCheckViele` im Helfer buendelt sieben
Dateien je Aufruf, und weil dhrt bei EINER Datei das nackte Array liefert,
bringt der Helfer das in dieselbe Form). Gegenprobe: ein
eingeschmuggeltes `Kapitel 99` und ein `Kapitel "Gibt es nicht wirklich"`
lassen die zwei Verweis-Faelle fallen (der Verweis muss dabei in einem STRING
stehen -- der Exporter wertet die Module aus und sieht Kommentare nicht).
**Falle, die das erst in der CI zeigte:** `dhrt pruef pfade` lief ueber
`.claude/worktrees/` mit, und `lebt()` trifft per Suffix -- eine geloeschte
Datei "lebte" also in einer Arbeitskopie weiter, lokal gruen, in der CI rot.
Der Durchlauf laesst `.claude` jetzt aus (wie `target`, `.venv`, `node_modules`).

**Import-Suchpfad und Dateisystem (2026-09-16, Weg D):** zwei Dateien, die
beim Umzug von 2026-09-08 als "nicht uebertragbar" liegen blieben, gehen mit
den Mitteln von heute. `tests/pruef/bibliothek.dhtest` (9) startet das
Programm als Kind und setzt `DH_PATH`, `USERPROFILE` und `HOME` per SETENV --
der Benutzerordner zeigt dabei IMMER in den Fallordner, sonst entschiede eine
Bibliothek auf dem Rechner des Pruefenden mit. Der Pfadtrenner kommt aus
`dlPfadTrenner$` im Helfer `dhrtlauf.dh` (einen Befehl fuer das
Betriebssystem gibt es nicht; ein Windows-Pfad traegt den Doppelpunkt hinter
dem Laufwerk). In tests/test_bibliothek.py bleiben nur die drei Tests der
PYTHON-Aufloesung, die die Qt-Editoren fuer die Zeilen-Herkunft brauchen.
`tests/pruef/dateisystem.dhtest` (15) ersetzt die pytest-Datei ganz: die
fnmatch-Zusagen stehen als festes Ergebnis fuer den Baum der Beilagen, die
Listen werden vor der Ausgabe sortiert, und "welche Datei ist neuer" wartet
1,1 s zwischen Beilage und neuer Datei statt die Zeit auf 1970 zu setzen
(ohne das Warten faellt der Fall -- beide tragen dieselbe Sekunde).
Gegenproben mit verfaelschten Kopien: 6 von 9 und 10 von 15 fallen, die
uebrigen Faelle haengen nicht an der verfaelschten Stelle.

**Die Doku-Pruefungen ohne Python (2026-09-16, Weg D):** test_doku_aussagen,
test_docs_codebloecke, test_vscode_grammatik und test_builtin_prosa (pytest)
sind geloescht; ihre Faelle stehen in `tests/pruef/doku_pruefungen.dhtest`
(14). Jeder Fall bekommt die Repo-Wurzel als Argument und macht `CHDIR`
dorthin, bevor er `dhrt pruef`/`dhrt doku --pruefen` startet -- beide suchen
die Wurzel vom Arbeitsordner aus, und ein Kind erbt ihn. Proben mit eigenen
Markdown-Dateien legen sie vorher in den Fallordner. Die Qualitaet von
`builtin_prosa.json` (echte Befehle, kein Markdown-Rest, kein abgeschnittener
Satz, Abdeckung ueber 45 %) prueft das json-Modul; dass die handgepflegte
Tabelle gewinnt, prueft `CODE_HOVER$` -- also der Hover der Laufzeit statt
`get_doc` des Qt-Editors. **Stolperstein:** `REGEX_FIND_ALL` liefert bei
einer Gruppe im Muster deren Inhalt, nicht den ganzen Treffer. Gegenprobe mit
voruebergehend verdorbener Doku (toter Link, erfundener Befehl, kaputter
Codeblock, verwaistes Dokument, veraenderte Grammatik): genau die sechs
betroffenen Faelle fallen.

**Drei gui-Bildtests ohne Python (2026-09-16, Weg D):**
test_gui_draw_window (9 Faelle nach `tests/pruef/gui_draw_window.dhtest`),
test_gui_bindung (5, `gui_bindung.dhtest`) und test_gui_table_frozen_edge (1,
`gui_tabelle_kante.dhtest`) sind geloescht -- vor einer Woche galten sie als
"nicht uebertragbar". Der Vergleich zweier Laeufe ist ein Paar von Faellen mit
`--- bild`; die Datenbank liest ein `--- nachher` mit `typeof()`; die
Akzent-Probe ("deutlich mehr Blau als Rot") liest ein `--- nachher` aus dem
Bild, das `DHRT_SCREENSHOT={fall}/kante.png` in `--- umgebung` ablegt.
**Zwei Funde beim Gegenproben:** die PLZ bewies in der alten Speicherprobe
nichts -- SQLite macht aus "80331" ueber die Spaltenaffinitaet ohnehin eine
Zahl, auch ohne Zahlenfeld; die Probe ist der Preis "19,99", der nur als
Zahlenfeld `real` wird. Und die Tooltip-Faelle haengen nachweislich an der
Wiedergabe (ohne `AUTOMATION_PLAY` fallen genau die zwei, die ihn erwarten).
Nebenbei: `_BRAUCHT_GRAFIK` in conftest.py nannte noch zwei geloeschte Dateien.
Dazu in `ci.yml` eine `concurrency`-Gruppe je Ereignisart und Commit: GitHub
hatte den Merge von #172 zweimal gemeldet und zwei main-Laeufe fuer denselben
Commit gestartet. Sie greift nur bei UEBERLAPPUNG -- kommt die doppelte Meldung
erst nach dem Ende des ersten Laufs, laeuft der zweite trotzdem.

**Beispiele ohne Python (2026-09-16, Weg D):** test_examples (pytest) ->
`tests/pruef/beispiele_laufen.dhtest` (23 Konsolen-Beispiele als Kind: Rueckgabe
0 und nicht leere Ausgabe, mehr ist bei Zeit/Zufall/Dateien keine Zusage) und
test_beispiel_sqlite_tabelle (pytest) -> `tests/pruef/beispiel_sqlite_tabelle.dhtest`
(Demo 158 mit `dhrt run --bilder 20`: Zeilen, Dateizeit und Groesse des
Originals unveraendert, Arbeitskopie weg; VACUUM INTO kopiert alles).
`pyramid_pusher.db` ist nicht versioniert -- ohne sie ueberspringt sich der
Fall. Gegenproben: falscher Beispielpfad bzw. eine Demo-Kopie ohne das
Aufraeumen fallen; "Original unberuehrt" ist bewusst NICHT gegengeprueft, weil
die Mutation die echte Spielstand-Datenbank beschriebe.

**SPEAK und der Sprachserver ohne Python (2026-09-16, Weg D):**
test_speak (pytest) -> `tests/pruef/speak.dhtest` (10, seriell; die WAV liest ein
`--- nachher` ueber ihre Bytes) und test_dhrt_lsp (pytest) ->
`tests/pruef/dhrt_lsp.dhtest` (11). Der LSP-Client `_hilfen/lspklient.dh`
schickt alle Anfragen, beendet den Server mit shutdown/exit und zerlegt DANACH:
`PROCESS_READ$` liest zeilenweise, ein LSP-Rumpf endet nicht mit einem Umbruch
und kaeme sonst erst mit der naechsten Kopfzeile. Zerlegt wird an den
Kopfzeilen, nicht ueber Content-Length (Bytes gegen Zeichen). **Zwei Funde:**
(1) der pytest-Fall "unbekannte Stimme ist ein Fehler im Klartext" war IMMER
uebersprungen -- seine Hilfsfunktion wertete "nicht gefunden" als fehlende
Sprachausgabe, und genau das steht in der erwarteten Meldung. (2) Die
Gliederung kennt Blockenden nur fuer CLASS/STRUCT/SUB/FUNCTION/PROPERTY; ein
`ENUM` reichte als Bereich nur ueber seine Kopfzeile (`symbole::bereiche`,
schon so in der Python-Vorlage; behoben 2026-09-17, siehe unten).
Gegenproben: vier bzw. drei Verfaelschungen, jede faellt.

**Drucken teilweise ohne Python (2026-09-16, Weg D):** Druckerliste,
`PDF_PREVIEW` (am Bild mit GETPIXEL) und die Fehlermeldungen von `PDF_PRINT`
und `OPENDOC` stehen in `tests/pruef/drucken.dhtest` (4). In
der pytest-Datei blieb NUR der Druck durch den echten Treiber
"Microsoft Print to PDF": dessen Datei ist kein PDF von dhrt, und der Leser
in Drachenhauch (`pdftext.dh`) kannte nur krilla-PDFs und keine Textlagen --
PyMuPDF blieb dafuer. **Am 2026-09-20 zog auch dieser Fall um**, weil der
Leser beides lernte (siehe "Die letzten vier pytest-Dateien"). Gegenproben nur an der Vorschau: eine Verfaelschung
der Druckbefehle koennte echt drucken.

**Export, Uhr und Firmata ohne Python (2026-09-17, Weg D):**
test_rust_export, test_export_assets und test_export_signierbar (pytest) ->
`tests/pruef/dhrt_export.dhtest` (6), test_builtins_uhr (pytest) ->
`tests/pruef/builtins_uhr.dhtest` (6), test_firmata_module (pytest) ->
`tests/pruef/modules_firmata.dhtest` (2). Die exportierte Exe startet der
Helfer `_hilfen/exportlauf.dh`; die "blanke Laufzeit" der Gegenprobe ist die
Exe ohne ihre Nutzlast (Laenge aus dem Footer) -- so braucht der Fall den
Pfad der laufenden dhrt nicht, und die Datei behaelt ihr Ausfuehrungsrecht.
**Zwei schwache Proben dabei gefunden:** das Bundle der absoluten Pfade
pruefte nur, dass es keinen `assets/`- und keinen `windows/`-Ordner gibt --
eine eingesammelte Datei landet aber neben der Exe, die Probe blieb gruen;
jetzt muss die Exe der einzige Eintrag sein. Und "GUI_CONFIRM nimmt drei
Argumente" suchte eine Compiler-Warnung in der Laufausgabe, wo sie nie steht;
der Fall fragt jetzt `dhrt --check`, mit vier Argumenten als Gegenprobe.
**Falle unter Windows:** eine gerade beendete Exe ist noch kurz abgebildet,
Ueberschreiben scheitert mit os error 1224 -- je Variante eine eigene Kopie.

**Barrierefreiheit, Rekursionstiefe und ENUM-Bereiche (2026-09-17, Weg D):**
test_gui_barrierefreiheit (pytest) -> `tests/pruef/gui_barrierefreiheit.dhtest`
(7, seriell) und test_rekursionstiefe (pytest) -> zwei Faelle mehr in
`tests/pruef/rekursionstiefe.dhtest`, die `vm.rs` und `build.rs` lesen. Der
fremde UIA-Leser laeuft weiter als PowerShell-Skript (Beilage), nur gestartet
per `PROCESS_START`; das Formular ist ein zweiter dhrt als Kind, und weil
`PROCESS_START` keine Prozessnummer nennt, sucht PowerShell das Fenster ueber
einen Titel mit Kennung (`_hilfen/uialeser.dh` sammelt die Ausgabe). Der
Kontrast der Themen wird in Drachenhauch gerechnet (WCAG-Leuchtdichte mit `^`).
Gegenproben: 7 von 7 und 2 von 2 fallen. **Dazu der offene Fund aus dem
LSP-Umzug:** ein `ENUM` ist jetzt ein Bereich bis `END ENUM`
(`symbole::bereiche`, SymbolKind Enum in `lsp::gliederung`); die Kurzform
`ENUM Farbe = ROT, GRUEN` bleibt EINE Zeile und kommt nicht auf den Stapel --
sonst verschluckte sie die SUBs dahinter bis zum naechsten END ENUM. Folge
in der IDE (sie liest dieselben Bereiche ueber `CODE_SYMBOLS$`): ein ENUM
laesst sich falten und steht in der Pfadleiste, solange die Marke darin
steht. Rust-Tests `enum_als_block_und_als_kurzform` und in
`gliederung_verschachtelt_und_enum` (Gegenprobe ohne `END ENUM`: beide
fallen), `tests/pruef/dhrt_lsp.dhtest` prueft das Ende mit.

**Die Buecher ohne Python (2026-09-17, Weg D):** fuenf pytest-Dateien, vier
Sammlungen. Signaturen im Referenzbuch gegen `builtin_index.json` -> drei Faelle
mehr in `tests/pruef/buch_pruefungen.dhtest` (`_hilfen/signatur.dh` baut
`arity` und `formen_je_befehl` als Zeichen-Scanner nach -- das regex-Modul
kennt keine Treffer mit mehreren Gruppen). Tippspiel-Band ->
`tests/pruef/buch_tippspiel.dhtest` (21: jeder Kapitelstand als Kopie, die
Pruefprogramme, Buch-Skript und docx-Build in den Fallordner). Einstiegsbuch ->
`tests/pruef/buch_einstieg.dhtest` (7). EPUB -> `tests/pruef/buch_epub.dhtest`
(7: der erste lokale ZIP-Kopf roh gelesen, OPF mit dem xml-Modul). **Zwei
Funde:** (1) die Zerleger-Probe war in pytest wertlos -- ohne die
Schraegstrich-Regel blieb `ECS_FILL_FLOAT/INT(...)` leer, weil `INT` in der
Probe nicht bekannt war; im echten Index steht es, und genau dafuer gibt es die
Regel. (2) **`XML_PARSE` ist nachsichtiger als ElementTree:** ein nacktes `&`,
`&nbsp;` und Attribute ohne Anfuehrungszeichen gehen durch (`<` und falsche
Schachtelung nicht). Fuer "fremde Daten lesen" ist das vertretbar, fuer eine
Wohlgeformtheits-Probe nicht -- der EPUB-Fall zaehlt die `&`-Stellen selbst und
traegt seine Gegenprobe im Fall. Gegenproben: 31 Verfaelschungen an echten
Dateien (Index, Kapiteltext, Anhang, Kapitelcode; danach `git checkout`) und
an Testkopien, dazu sieben kaputte EPUBs -- jede faellt. Eine Falle beim
Gegenproben: der Index fuehrt die Signatur `MID$(2..3 Argumente)` zweimal
(`MID` und `MID$`), und der Abgleich nimmt wie die Python-Fassung den letzten.

**Tracker-Datenverlust behoben (2026-09-17, aus der Bestandsaufnahme):** der
Tracker in Drachenhauch las ein Sample-, Keymap- oder SoundFont-Instrument
aus einer Datei der Qt-Fassung als stummen Platzhalter -- und schrieb es beim
Sichern als `synth` zurueck; die eingebetteten Samples waren danach weg, ohne
dass es jemand sah. Jetzt fuehrt `iRoh`/`iArt` das Original-JSON mit, das
Sichern schreibt es zurueck (nur Name, Lautstaerke, Pan und Huellkurve kommen
aus dem Piloten), Kopieren und Entfernen tragen es mit, und die Anzeige haengt
"hier stumm" an (`instName$`) statt es in den Namen zu schreiben. **Zweiter
Fund:** "hier stumm" stimmte nicht -- ein Sample-Instrument traegt keine
Wellenform, `square` ist die Vorgabe, und es klang als Rechteck; Wiedergabe,
WAV und Vorhoeren ueberspringen es jetzt. Dafuer fehlte im json-Modul das
Gegenstueck zu `JSON_SET_JSON`: neu **`JSON_GET_JSON(h, pfad$)`** (Teilbaum als
eigenes Dokument, eine Kopie; zwei Faelle in `json_schreiben.dhtest`). Tests
drei Faelle in `tests/pruef/werkzeug_tracker.dhtest` an einer Beilage, die
`Song.save_json` der Qt-Fassung geschrieben hat; Gegenprobe mit drei
verfaelschten Tracker-Kopien (Sichern als synth, nicht stumm, nicht
mitgetragen) -- jede laesst genau ihre Faelle fallen.

**Der Tracker spielt Sample-Instrumente (2026-09-21):** Sample, Keymap und
die als Keymap eingelesenen SoundFonts klingen jetzt beim Abspielen, Vorhoeren
und in der WAV, nach den Regeln der Qt-Fassung (`_resample`/`zone_for`):
Tonhoehe = Abstand zur Grundtaste, Loop nur mit forward/pingpong und
0 <= Anfang < Ende <= Laenge, sonst einmal durch; eine Keymap nimmt die
erste Zone, die die Taste abdeckt, sonst die naechstgelegene; Huellkurve und
Slide wie beim Synth, kein Vibrato. Eine unbekannte Art bleibt stumm ("hier
stumm"). **Zwei Laufzeit-Bruecken:** `SAMPLE_FROM_BUFFER(puffer, abtastrate)`
(16-Bit-PCM LE mono, /32768 wie der Qt-Leser; ungerade Laenge = Fehler) und
`SAMPLE_NOTE(sample, halbtoene, dauer, attack, decay, sustain, release,
vol[, slide])` -> SOUND -- `SAMPLE_PLAY` spielt SOFORT und liefert einen
Kanal, ein Tracker braucht aber einen Klang fuer `AUDIO_PLAY_AT`/
`AUDIO_SOUND_MIX`. Dazu `SAMPLE_SET_LOOP(..., art$)` mit `pingpong`
(`LoopArt`, `loop_falten` in audio.rs -- dieselbe Faltung fuer SAMPLE_PLAY
und SAMPLE_NOTE; die Huellkurve von AUDIO_NOTE ist dafuer als `huellkurve`
herausgeloest). Im Piloten: Zonen `zSamp/zLo/zHi/zRoot`, je Instrument die
Liste `iZonen`; **die Zonen gehoeren ihrem INHALT** (`zonenVon`, Schluessel
aus Rate, Tasten, Loop und Base64): Rueckgaengig laedt den ganzen Song neu,
und ein SAMPLE laesst sich nicht freigeben -- je Schritt neu dekodieren
kostete Zeit und Speicher. Tests `tests/pruef/sample_note.dhtest` (14:
Tonhoehe an den Nulldurchgaengen, Loop-Arten an einer Rampe, Huellkurve) und
in `werkzeug_tracker.dhtest` ein Lied, das der Fall im `--- vorher` selbst
baut (Sinus mit bekannter Frequenz, Keymap mit stiller und Loop-Zone).
**Die alte Beilage taugte dafuer nicht:** ihr Sample hat 64 Werte (3 ms) --
ein 10-ms-Fenster sieht es gar nicht, der Fall "bleibt stumm" waere auch mit
Wiedergabe gruen geblieben; er prueft jetzt eine UNBEKANNTE Art. Gegenproben:
fuenf Verfaelschungen des Piloten (ohne Grundtaste, immer erste Zone, ohne
Loop, alte Stummregel, jede Art als Sample) und zwei der Laufzeit (pingpong
als vorwaerts, Schritt ohne Abtastrate) -- jede faellt.

**Der GB-Code des SFX-Generators nimmt den Pan mit (2026-09-17):** aus der
Bestandsaufnahme -- [GB-Code kopieren] gab nur `s = AUDIO_SFX(...)` heraus,
ohne IMPORT, ohne Abspielen und ohne den Pan-Regler. Der Pan liegt am
WIEDERGABE-Kanal, nicht im Klang (`AUDIO_SFX` kennt ihn nicht), und ein
fehlender Pan faellt niemandem auf -- der kopierte Code klang schlicht anders
als die Vorschau daneben. Jetzt kommt ein lauffaehiges Stueck heraus (IMPORT,
`DIM snd AS SOUND`, der Aufruf, dann `PLAYSOUND` bzw. `AUDIO_PLAY` +
`AUDIO_PAN` mit derselben Rechnung wie das Vorhoeren). Test: ein Fall in
`tests/pruef/werkzeug_sfx.dhtest` prueft die Zeilen UND laesst `dhrt --check`
ueber den erzeugten Code laufen; Gegenprobe mit zwei verfaelschten Kopien
(ohne Pan-Zweig, ohne IMPORT) -- beide fallen. Faktor 1,18 -> 1,21.

**Das Befehlsverzeichnis liegt in `daten/` (2026-09-17):** `builtin_index.json`,
`builtin_docs.json` und `builtin_prosa.json` lagen in
`drachenhauch/editor_qt/`, also IM Python-Paket -- `dhrt` bettete sie von dort
ein (`include_str!`, viermal `compiler.rs`, zweimal `lsp.rs`) und erkannte die
Repo-Wurzel **an genau diesem Pfad** (`doku::repo_wurzel`). Damit liess sich
`drachenhauch/` nicht loeschen, obwohl an den Dateien nichts Python ist; es war
der letzte Punkt der Bestandsaufnahme, der keine Entscheidung brauchte. Neu
gelesen wird aus `daten/` von: den sechs `include_str!`, `doku.rs`
(Wurzel-Erkennung, Index-Leser, Schreibziel der Prosa, Namen fuer die
Grammatik), `pruef.rs`, den Anhaengen beider Buecher (`90_anhang_a.js`,
`36_anhang_c_weiter.js`), zwei Sammlungen (`buch_pruefungen`,
`doku_pruefungen`) und auf der Python-Seite **`dhrt_meta.daten_datei()`** --
die EINE Stelle, die den Ordner sucht, nach demselben Muster wie
`dhrt_locate.find_dhrt` (Bundle zuerst, dann Repo-Wurzel);
`builtin_docs.py` holt sie sich von dort. **Die stille Stelle war der
Installer:** `collect_data_files("drachenhauch")` sammelte die drei Dateien
nur, WEIL sie im Paket lagen -- die PyInstaller-Spec nennt `daten/` jetzt
ausdruecklich. Ohne den Eintrag waere der Hover der installierten IDE stumm
auf seinen Minimal-Satz zurueckgefallen (die Leser fangen eine fehlende Datei
ab, damit der Editor nie ganz blind ist), und das saehe man erst in der
fertigen Installation. Darum ist es nicht behauptet, sondern **im gebauten
Bundle nachgesehen** (`_internal/daten/` mit allen drei Dateien, der alte Ort
leer) samt Probe des Resolvers gegen genau dieses Verzeichnis. Gegenprobe mit
weggenommenem `daten/`: 11 von 14 Faellen in `doku_pruefungen` und 5 von 8
Tests in `test_dhrt_meta.py` fallen, `dhrt doku` meldet den neuen Pfad. Der
eingebettete Teil kann so nicht fallen -- ein falscher Pfad im `include_str!`
ist ein Baufehler, kein Laufzeitfehler. **Falle fuer den naechsten Umzug:**
`dhrt pruef pfade` prueft Pfade, die als Inline-Code in `docs/` stehen -- eine
vergessene Doku-Stelle ist ein Befund, nicht bloss ein Schoenheitsfehler
(genau einer kam so heraus, `docs/lsp.md`).

**Zwei Entscheidungen aus der Bestandsaufnahme (2026-09-18):** (d) alte
Qt-`.dhsprite`-Dateien muessen NICHT mehr aufgehen -- nur der Nutzer hat den
Qt-Sprite-Editor benutzt; die 14 im Repo (Galaga, Plattformer, PNG daneben)
fallen mit dem Qt-Editor. (c) **`rust/build_runtime.py` und
`rust/build_wasm.py` bleiben Python -- nur mit der Standardbibliothek.** Wer
dort ein `import` ergaenzt, muss es in `tests/pruef/bauskripte.dhtest` in die
feste Liste eintragen (`__future__`, `os`, `platform`, `subprocess`, `sys`,
`pathlib`, `shutil`); ein Paket von aussen oder ein Modul aus `drachenhauch/`
laesst den Fall fallen. So bleibt "Python ist weg" wahr bis auf diese
Bauhilfe, die ein beliebiges Python 3 ohne venv nimmt. (a) **Keine
Qt-Funktion blockiert das Loeschen, keine ist dauerhaft ausgeschlossen** --
was fehlt, wird gebaut: erst die acht Punkte der Stufe A
(`docs/entwurf-python-abbau.md`, am Ende von 7.7), der Rest bei Bedarf. Keine
eigene Datei haengt an einer Qt-Funktion (nachgesehen). (b) entschieden
am 2026-09-19: ja, macOS und Linux bekommen ein Paket ohne Python (unten).

**Absturz-Wiederherstellung in der IDE (2026-09-18, erster Punkt der Stufe
A):** jede laufende IDE schreibt alle 30 s (`wiederherstellung_s` in der
ide.json) die Texte ihrer ungesicherten Reiter nach
`wiederherstellung/<kennung>/stand.json` neben die ide.json -- in eine
Nebendatei und dann `RENAME`, damit ein Absturz mitten im Schreiben keinen
halben Stand hinterlaesst. **Ein Ordner JE IDE**, nicht einer fuer alle wie
in der Qt-Fassung (dort hielt eine zweite IDE die Sicherungen der ersten fuer
Absturzreste, und wer zuerst sauber beendete, loeschte die des anderen). Die
Uhrzeit im Stand ist das Lebenszeichen: der Start bietet nur an, was als
beendet markiert (`offen: false`) oder seit drei Takten stumm ist
(Wiederherstellen/Verwerfen/Spaeter; ESC = Spaeter). Zurueckgeholt wird
UNGESICHERT, ueber `tabOeffnen` statt `dateiOeffnen` (sonst oeffnete eine als
Text bearbeitete .dhform den Designer). **Der Fund dabei:** das Kreuz des
Fensters beendet die IDE OHNE Rueckfrage -- die Schleife endet an
`QUITREQUESTED()`, ungesicherte Arbeit war wortlos weg. Jetzt bleibt der
Stand dann als beendet liegen und kommt beim naechsten Start zurueck; die
Rueckfrage beim Kreuz kam am selben Tag (naechster Absatz). Tests
`tests/pruef/werkzeug_ide_wiederherstellung.dhtest` (6); Gegenprobe mit
sieben Verfaelschungen, jede faellt -- im ersten Anlauf fiel "kein Takt"
NICHT, weil die letzte Sicherung beim Ende dieselbe Protokollzeile schrieb
wie der Takt; sie heisst jetzt `sicherung N beendet`.

**Kreuz und ESC (2026-09-18):** neu `WINDOW_CLOSE_REQUESTED()` -- das Kreuz
(Alt+F4, Beenden-Taste) OHNE das Bildlimit, das `QUITREQUESTED` mitzaehlt;
das Fenster bleibt offen, raylib setzt `shouldClose` je Bild neu
(`rcore_desktop_glfw.c`: PollInputEvents liest das GLFW-Flag und setzt es
zurueck), es ist also ein Ereignis. Die IDE-Schleife laeuft jetzt
`WHILE NOT beenden`: Kreuz -> `beendenAnfragen()` (Rueckfrage nur bei
Ungesichertem), `QUITREQUESTED` ohne Kreuz = Bildlimit -> `BREAK`, die
Sicherung bleibt. **Der Fund dabei steckt in der Laufzeit: ESC beendete
JEDES Programm mit gui** -- raylibs Beenden-Taste ist per Vorgabe ESC, und
ein ESC, das einen Dialog abbrechen sollte, schloss die IDE samt
ungesicherter Arbeit. Das erste `GUI_UPDATE` nimmt ESC jetzt als
Beenden-Taste weg (`Graphics::esc_der_gui`), ausser das Programm hat
`WINDOW_ESC_QUIT` selbst gesetzt (`esc_ausdruecklich`). **Keine Aufnahme
kann das pruefen:** eingespielte Tasten (`INPUT_KEY_DOWN`) laufen am
GLFW-Tastenrueckruf vorbei, in dem raylib die Beenden-Taste prueft -- die
Tests schicken darum ECHTE Windows-Nachrichten (PostMessage WM_KEYDOWN/
WM_CLOSE, `tests/pruef/_hilfen/fenstersender.ps1`, Fenster ueber einen
Titel mit Kennung). Tests `tests/pruef/fenster_schliessen.dhtest` (5) und
`tests/pruef/werkzeug_ide_schliessen.dhtest` (2); Gegenproben: ohne den
ESC-Wechsel bzw. mit `quit_requested` statt `close_requested` fallen genau
die zwei betroffenen Faelle, mit der alten IDE-Schleife beide IDE-Faelle.
Die Werkzeuge 183-199 zogen einen Tag spaeter nach (siehe unten).

**Sprung aus der Ausgabe (2026-09-18, zweiter Punkt der Stufe A):** eine
Zeile der Ausgabe mit `datei.dh:zeile` (`Laufzeitfehler in spiel.dh:3: ...`,
`lib/x.dh:2: Parse-Fehler ...`, Warnungen) ist eingefaerbt, Doppelklick oder
Enter (eine Liste meldet Enter als `GUI_DOUBLE_CLICKED`) oeffnet die Datei
dort (`ausgabeOrt`/`ausgabeWeg$`/`ausgabeAnspringen` in ide.dh). Der Name
wird relativ zum Ordner des GELAUFENEN Programms gesucht (dhrt wechselt
dorthin, seine Meldungen sind relativ dazu), dann im Projekt; vor `.dh:`
steht beliebiger Text und der Name darf Leerzeichen tragen -- genommen wird
das laengste Stueck aus Woertern davor, das es als Datei gibt. Kein neuer
Baustein in dhrt. Tests `tests/pruef/werkzeug_ide_ausgabe.dhtest` (3:
Programm in `sub/`, Import aus `mit leer/`, Gegenprobe ohne Stelle); drei
Gegenproben (ohne die Abfrage, ohne Zusammenfuegen ueber Leerzeichen, ohne
den Programmordner) lassen genau ihre Faelle fallen. **Falle beim Test:**
die Wiedergabe setzt beim ERSTEN Ereignis an -- Enter bei Aufnahmebild 160
kam 140 Bilder nach F5, also VOR der Auswahl in Bild 150; eine Beilage darf
keine Leerzeichen im Namen tragen (die Datei legt ein `--- vorher` an).

**Zoom in der IDE und das Mausrad im Textbereich (2026-09-18, dritter
Punkt der Stufe A):** Strg+Plus/Minus (2 px) und Strg+0 (zurueck auf
`SCHRIFT_NORMAL` = 16) als Menue-Kuerzel, Strg+Rad (1 px je Schritt,
`radZoom`) ueberall im Fenster; `schriftSetzen` zieht das Drehfeld der
Einstellungen mit, sonst setzte es die Schrift bei offenen Einstellungen
zurueck. **Drei Stuecke in der Laufzeit:** (1) **ein Textbereich rollte mit
dem Rad GAR NICHT** -- gefunden beim Bau, das Code-Feld liess sich nur ueber
Marke und Tasten bewegen. Jetzt drei Zeilen je Schritt (`textarea_wheel`),
und die Editierschleife zieht den Ausschnitt nicht zur Marke zurueck,
solange Marke, Anker und Zeichenzahl dem Stand beim Rollen gleichen
(`Widget::rad_stand`) -- sie tat das in JEDEM Bild. (2) **Strg+Rad gehoert
dem Programm:** alle sieben Radabfragen der gui laufen ueber `Gui::rad`, das
mit Strg 0 liefert; `MOUSEWHEEL_Y` sieht den Wert weiter. (3) **`Plus`/`Minus`
im Kuerzel** (`Strg+Plus`, `Strg++`, `Strg+-`): raylib benennt Tasten nach
der US-Lage, das deutsche "+" liegt auf "]", das "-" auf "/" -- `K_PLUS`/
`K_MINUS` treffen darum beide Belegungen und den Ziffernblock
(`kuerzel_taste`). Tests `tests/pruef/gui_zoom_rad.dhtest` (4),
`tests/pruef/werkzeug_ide_zoom.dhtest` (1), Rust-Test `plus_und_minus`.

**Tooltip beim Ueberfahren und Strg+Klick zur Definition (2026-09-19,
vierter Punkt der Stufe A):** neu in der Laufzeit **`GUI_TEXTAREA_POS_AT(ta,
x, y)`** -> (zeile, spalte) des Zeichens unter einem Bildschirmpunkt, mit
derselben Rechnung wie der Klick (`ta_rows`, Nummernspalte, Scroll,
waagerechter Versatz); (0, 0) ausserhalb, in der Nummernspalte, unter der
letzten Zeile und HINTER dem Zeilenende -- dort zeigt die Maus auf nichts.
Die IDE (`mausNachziehen`) setzt nach 8 ruhigen Bildern den Tooltip des
Code-Felds auf `CODE_HOVER$` des Worts (umbrochen auf 72 Zeichen, hoechstens
14 Zeilen); die gui zeigt ihn nach ihrer eigenen Ruhezeit und nimmt ihn bei
jeder Bewegung weg, er steht also nie neben dem falschen Wort. **Strg+Klick
springt beim LOSLASSEN**, nicht beim Druecken: der Textbereich setzt die
Marke selbst beim Druck und zieht sie, solange die Taste haengt, als
Auswahl hinter der Maus her -- ein Sprung beim Druecken stand im naechsten
Bild wieder an der Klickstelle (der erste Testlauf zeigte es). Tests
`tests/pruef/gui_textarea_pos.dhtest` (2, ohne feste Pixellagen: abgetastet,
nur die Wechsel ausgegeben) und `tests/pruef/werkzeug_ide_tooltip.dhtest`
(4; der Helfer `_hilfen/idemaus.dh` sucht die Lage des Worts mit POS_AT und
schreibt die Aufnahme selbst). **Zwei Fallen beim Gegenproben:** (1) die
Verfaelschung "hinter dem Zeilenende das letzte Zeichen" blieb erst gruen,
weil beide Tests "hinter dem Ende" vom LETZTEN Treffer aus massen -- unter
dem Fehler liegt der am Feldrand, die Probe dahinter traf gar nichts; jetzt
vom Anfang des letzten Zeichens. (2) Eine Verfaelschung per `python -c
"..."` mit `!` darin kam in der Shell nicht an -- der Bau lief ueber die
unveraenderte Datei, und "gruen" hiess nichts; Verfaelschungen gehoeren in
eine Skriptdatei, und die Zeile wird vor dem Bau per grep nachgesehen.

**Haltepunkt per Klick in die Nummernspalte (2026-09-19, fuenfter Punkt der
Stufe A):** neu **`GUI_TEXTAREA_GUTTER_CLICKED(ta [, taste])`** -> Zeile (ab
1, logisch) des Klicks in die Nummernspalte, 0 = keiner; `taste` 0 links, 1
rechts; ein Bild lang wie `GUI_CLICKED`. Erkannt im Hover-Durchlauf von
`update` (`ta_rand_zeile`, dieselbe Zeilenrechnung wie ein Klick in den Text,
der Faltpfeil bleibt beim Falten), gilt also auch fuer ein Feld ohne Fokus und
fuer rechts. Die IDE (`randKlicksNachziehen`, jeder offene Reiter -- die
geteilte Ansicht zeigt zwei) schaltet links um wie F9 und fragt rechts nach
der Bedingung wie Umschalt+F9, beide fuer die ANGEKLICKTE Zeile
(`haltepunktUmschaltenIn`/`haltepunktBedingungFragenIn`; die Bedingung merkt
sich ihre Datei in `bedingungPfad`, statt beim Bestaetigen den Reiter zu
nehmen, der gerade vorn ist). **Der Fund:** ein Linksklick in die Spalte liess
die Marke im Druck-Bild stehen, aber die folgenden Bilder mit gehaltener
Taste liefen durch den Zug-Zweig und zogen sie doch an den Rand -- jetzt
sperrt der Druck den ganzen Zug (`farbfeld_zug`, wie beim Farbfeld). Gesehen
hat es erst der IDE-Fall; der Laufzeit-Fall war zweimal zu schwach: die Marke
stand vorher schon auf der angeklickten Zeile, und gefragt wurde nur im
Druck-Bild. Jetzt steht sie auf Zeile 4 und wird nach dem Loslassen gelesen
-- gegen den alten Bau faellt er. Tests `tests/pruef/gui_rand_klick.dhtest`
(3) und `tests/pruef/werkzeug_ide_haltepunkt.dhtest` (3: links + Debugger
haelt in 3 bei Marke in 1, zweimal = aus, rechts + Bedingung per
Zwischenablage haelt genau einmal; Helfer `prRand` in
`_hilfen/idemaus.dh`); ohne den Aufruf in der IDE fallen alle drei.

**Die Suchleiste (2026-09-19, sechster Punkt der Stufe A):** statt zweier
Eingabe-Kaesten hintereinander ein Fenster, das offen bleibt: Suchfeld mit
Weiter/Zurueck und Trefferzahl, Schalter **Gross/klein**, **Ganzes Wort**,
**Ausdruck** (Alt+C/Alt+W, in der ide.json gemerkt), alle Treffer im Code
eingefaerbt (`F_SUCHTREFFER`), Ersetzen **einzeln** (Strg+Umschalt+1:
ersetzt nur einen GENAU markierten Treffer, sonst springt der erste Druck
bloss hin -- man sieht jede Stelle vorher) und **alle** (Strg+Alt+Enter,
Auswahl ueber alles + INSERT = EIN Strg+Z). **Alle Suchen der IDE gehen
durch EIN Muster** (`suchMuster$`: woertlich per `REGEX_ESCAPE$`, ganze
Woerter zwischen `\b(?:...)\b`, ohne Gross/klein `(?i)`) -- vorher suchte
das Ersetzen im Feld ohne Ruecksicht auf den Regex-Schalter, und die
Projektsuche nimmt die Schalter jetzt mit. Gearbeitet wird ZEILENweise wie
die Trefferliste, sonst hiesse `^` beim Ersetzen etwas anderes; woertlich
bekommt der Ersatz verdoppelte Rueckstriche (`\1` bleibt Text).
**Zwei Laufzeit-Stuecke:** (1) **`REGEX_FIND_POS(text, muster [, ab])`**
-> (start, laenge) in Zeichen ab 0, und **`REGEX_ESCAPE$`** -- bis dahin
gab es keinen Weg zur LAGE eines Treffers, die IDE rechnete
`INSTR(zeile, REGEX_FIND(...))` und markierte bei `\bhp\b` das `hp` in
`hpmax`, wenn es davor stand. `ab` sucht ueber `find_at` weiter, damit `\b`
die Zeichen vor dem Suchbeginn noch sieht (Gegenprobe: mit abgeschnittenem
Text faellt der Wortgrenzen-Fall). (2) **Ein Textbereich zeigt seine
Auswahl auch OHNE Fokus**, gedaempft -- die Leiste nimmt dem Feld den
Fokus, und der markierte Treffer war unsichtbar (gesehen erst im Bild).
Tests `tests/pruef/werkzeug_ide_suche.dhtest` (10; fuenf Verfaelschungen
der IDE -- ohne Wortgrenze, immer ohne Gross/klein, Ersatz unmaskiert,
Ersetzen ohne Trefferpruefung, Lage per INSTR -- lassen je genau ihre
Faelle fallen), drei Faelle in `tests/pruef/modules_regex.dhtest`,
`tests/pruef/gui_auswahl_ohne_fokus.dhtest` (2, als Paar; gegen den alten
Bau faellt der erste). **Falle beim Schreiben der Tests:** `\\2` in einem
Bash-Heredoc kam als Steuerzeichen `\x02` an -- sogar in der Erwartung,
die damit falsch UND unsichtbar war; Rueckstriche gehoeren in eine
Skriptdatei. Und eine Aufnahme mit zu kleiner Zahl im Kopf (`c 20` bei 24
Ereignissen) verliert die letzten -- ein "Datei unveraendert" waere dann
auch ohne Sichern wahr; der Fall prueft darum das Sichern mit.

**Warnungen beim Umrechnen in den Tracker (2026-09-19, siebter Punkt der
Stufe A):** das Notenblatt (199) rechnete nach den Regeln von
score/convert.py, verschwieg aber, was dabei verloren geht -- ein Akkord
wurde still auf seine hoechste Note reduziert, eine Note an der
64-Zeilen-Grenze still gekuerzt. `trackerJson` fuellt jetzt
`trackerWarnungen` mit denselben vier Saetzen in derselben Reihenfolge
(je Spur erst die Akkorde, dann je Note gerundet -> fiel heraus ->
gekuerzt; Beats wie Pythons `:g` ueber `beat$`, weil `FORMAT$` kein `%g`
kennt), und [In Tracker] zeigt sie VOR dem Schreiben in einem Kasten
("Trotzdem oeffnen|Abbrechen", je Warnung zwei Zeilen -- der Kasten ist
auf 640 Punkte begrenzt, hoechstens acht plus "... und k weitere"); die
Qt-Fassung zeigte sie nur als Hinweis. **Dabei fiel eine Abweichung auf:**
"fiel aus dem Song heraus" prueft Python an der Song-LAENGE
(`start_row >= total_rows`), der Pilot an der Pattern-ZAHL -- eine Note,
die hinter das Ende rundet, landete dort in einer Zeile jenseits von
`rows` und fiel beim Ausgeben still weg; jetzt dieselbe Bedingung samt
Warnung. Tests: zwei Faelle mehr in `tests/pruef/werkzeug_notenblatt.dhtest`
(ein kleines Stueck, bei dem alle vier Regeln greifen: ESC schreibt
nichts, Enter schreibt das Gitter von `to_tracker_song`), der Demo-Fall
prueft die zwei Warnungen des Demo-Stuecks und drueckt Enter; die
Erwartungen kommen aus einem Lauf von `to_tracker_song`. Fuenf
Verfaelschungen (ohne Kasten, ohne je eine der vier Warnungen) lassen je
genau ihre Faelle fallen.

**Die Grenzen des Sprite-Editors (2026-09-19, achter und letzter Punkt der
Stufe A):** 64 Bilder, 256 px, 8 Ebenen statt 16 / 128 / 4. Die Zahlen
waren nicht das Problem, sondern die BAUART: der Pilot legte jedes Bild
jeder Ebene vorab an (`ebene[MAXBILD, MAXEB]`, 64 Bilder samt Textur schon
fuer ein leeres Sprite) -- bei 64 x 8 waeren es 512 geworden. Jetzt entsteht
ein Platz erst, wenn er gebraucht wird (`ebenenSichern` nach jedem
Wachsen von `anzBild`/`anzEb`, Buchfuehrung `ebDa`, weil ein nie angelegter
Platz KEIN Bild traegt und IMAGE_FREE darauf ein fremdes freigaebe). Ein
einmal angelegter Platz bleibt, auch wenn der Bereich schrumpft -- darum
bekommt er beim Groesse-Aendern und Drehen die neue Groesse mit (sonst kaeme
er mit der alten zurueck), und [Neu] bei Bildern bzw. Ebenen leert ihn,
statt ihn neu anzulegen; die Schleifen dafuer laufen nur noch ueber den
benutzten Bereich. Der Export-Streifen wird in `baueStreifen` in der
belegten Breite angelegt statt vorab `MAXBILD` Felder breit. **Warum 256
und nicht die 1024 der Qt-Fassung:** die 48 Verlaufsplaetze liegen fest in
Sprite-Groesse (bei 1024 x 1024 200 MB, bevor ein Punkt gemalt ist), und
Zuschneiden/Statistik gehen Punkt fuer Punkt -- gemessen am vollen Mass
Zuschneiden 1 s (je Bild EINMAL ueber die zusammengerechneten Ebenen statt
je Ebene, sonst acht Mal so viele Abfragen), Statistik 2 s. Tests: vier
Faelle mehr in `tests/pruef/werkzeug_sprite.dhtest` (70 x [Neu] Bild und
10 x [Neu] Ebene ergeben 64 und 8 mit 512 angelegten Plaetzen, vorher genau
einer; ein zurueckgelassener Platz kommt nach dem Groesse-Aendern leer und
in 48x40 zurueck; ein wieder angelegtes Bild und eine wieder angelegte
Ebene sind leer). Fuenf Verfaelschungen (alles vorab anlegen, alte Grenze,
Groesse nur fuer benutzte Plaetze, Ebene bzw. Bild nicht leeren) lassen je
genau ihren Fall fallen. Damit ist Stufe A erledigt.

**Rueckfrage beim Kreuz in den Werkzeugen (2026-09-19):** das Kreuz beendete
jedes der neun Werkzeuge 183-199 ohne Rueckfrage, und in 185, 187 und 189 tat
ESC dasselbe -- ungesicherte Arbeit war wortlos weg. Jetzt fragen beide
(Sichern|Verwerfen|Abbrechen), aber NUR, wenn etwas ungesichert ist; sonst
endet das Programm wie vorher sofort. Muster wie in der IDE: Schleife
`WHILE NOT beenden`, `WINDOW_CLOSE_REQUESTED()` -> `endeAnfragen()`,
`QUITREQUESTED()` ohne Kreuz = Bildlimit eines Testlaufs -> BREAK. Was
"ungesichert" heisst, ist je Werkzeug verschieden: SFX und Partikel haben
keinen Merker, sondern vergleichen ihre Regler mit einer eigenen Zeile im
Verlaufsspeicher (`U_GESICHERT`, gesetzt nach Sichern, Laden und
Werkseinstellung) -- zurueckgestellt ist damit wieder gesichert; der
Sprite-Editor bekam `ungesichert` (an jedem Verlaufsschritt, jeder Wandlung,
jeder Bild-/Ebenen-/Bereichsaenderung; gesichert heisst als .dhsprite, ein
Streifen-Export traegt keine Ebenen); Tilemap, Tracker und die drei Weg-B-
Werkzeuge hatten ihren Merker schon, 196 fragt ueber seine drei Formulare
(und `Ende` im Menue fragte dort bisher auch nicht). **Die Falle:** die
ESC-Abfrage muss NACH `GUI_UPDATE` stehen -- davor oeffnete dieselbe Taste
den Kasten, und das GUI_UPDATE desselben Bildes nahm sie gleich als
"Abbrechen" (der Kasten war nie zu sehen); und VOR der Auswertung der
Antwort, sonst oeffnet die abbrechende Taste ihn im selben Bild wieder.
Tests `tests/pruef/werkzeug_kreuz.dhtest` (24, echte Fensternachrichten ueber
`_hilfen/fenstersender.ps1`; je Werkzeug ungesichert fragt / gesichert endet
sofort, ESC fuer 185/187/189, SFX zurueckgestellt, Sprite ein Verlaufsschritt).
Gegenprobe gegen die alten Werkzeuge: alle 13 Faelle mit ungesicherter Arbeit
fallen; drei Verfaelschungen (Verlaufsschritt ohne Merker, SFX immer
ungesichert, ESC vor GUI_UPDATE) lassen je genau ihre Faelle fallen.

**Pakete fuer macOS und Linux ohne Python (2026-09-19, Entscheidung (b)):**
`installer/bauen.dh` baut unter Linux `Drachenhauch-IDE-<fassung>-linux-<arch>.tar.gz`
(Ordner mit dhrt, ide/, docs/, examples/, dem Starter `drachenhauch` und
`install.sh` -- fuer den Nutzer, ohne root, nach XDG: Menueeintrag,
`.dh`-Zuordnung ueber einen eigenen MIME-Typ, `~/.local/bin/drachenhauch`
und `dhrt`; `--entfernen` raeumt ab) und unter macOS
`Drachenhauch-IDE-<fassung>-macos-<arch>.dmg` (`Drachenhauch.app` mit dhrt
UND Starter in Contents/MacOS -- nur ein Programm dort zaehlt fuer macOS zum
Bundle --, Rest in Resources, Ad-hoc-Signatur, ein Verweis auf den
Programme-Ordner, LIESMICH). Der Teil steht in `installer/paket.dh`, die
Vorlagen in `installer/posix/` (LF erzwungen in `.gitattributes` -- ein
Starter mit CRLF laeuft nicht). **Die Beispiele kopiert der Starter beim
Start** nach Dokumente/Drachenhauch/examples (ohne Dokumente-Ordner nach
~/Drachenhauch), nur was dort fehlt (`cp -Rn`: Bearbeitetes bleibt, Neues
einer spaeteren Fassung kommt dazu), und nennt den Ordner der IDE ueber das
neue **`DH_IDE_BEISPIELE`** -- im Paket soll niemand schreiben, in einem
.app-Bundle braeche es die Signatur. `install.sh` fasst in ~/.local/bin nur
EIGENE Starter an (erkannt am Pfad darin), ein fremdes `dhrt` bleibt. **Die
Symbole liegen fertig im Repo** (`installer/symbole/`, erzeugt von
`installer/symbole.dh` aus dem Logo, `.icns` mit PNG-Eintraegen 256/512
ohne iconutil): Bilder brauchen in der Laufzeit einen GL-Kontext, also ein
verstecktes Fenster, und das gab es auf den Bau-Rechnern der CI nicht (Linux
ohne DISPLAY, macOS ohne passendes OpenGL) -- `bauen.dh` brach dort ab.
Nebenbei liegt das Logo so nicht mehr nur im Python-Paket. **Zum ersten Mal
lief dhrt MIT Grafik auf Linux und macOS** (neuer Job `paket-ohne-python`
in `package.yml`, cargo direkt ohne Python, CFLAGS wie build_runtime.py) --
und das fand einen **Absturz beim Beenden unter Linux**: `RaylibHandle` war
das erste Feld von `Graphics` und schloss beim Abraeumen das Fenster samt
GL-Kontext, bevor Schriften und Texturen freigegeben wurden; Mesa stuerzte
daran ab (gdb im CI-Lauf: drop(Graphics) -> UnloadFont -> rlUnloadTexture),
Windows verzeiht es. Jetzt ist es das letzte Feld,
`tests/pruef/grafik_aufraeumen.dhtest` haelt die Reihenfolge fest
(Gegenprobe mit der alten: faellt), und der Paket-Lauf laesst ein
Fensterprogramm unter xvfb mit Rueckgabe 0 enden. **Gemessen im
Paket-Lauf:** Linux (x86_64) 22 MB, install.sh in ein frisches HOME, die IDE
startet unter xvfb und endet sauber, die Sammlung laeuft mit Grafik ganz;
macOS (arm64) 22 MB, `codesign --verify` gueltig, die Laufzeit aus dem
Bundle fuehrt Programme aus, der Starter legt die Beispiele an -- **ein
Fenster geht auf den macOS-Laeufern nicht** (kein OpenGL), ob die IDE auf
einem echten Mac aufgeht, ist ungeprueft. Bekannt offen: eine `.dh` per
Doppelklick im Finder oeffnete die IDE nicht mit der Datei (seit demselben
Tag behoben, siehe "Dateien vom Finder" unten), keine
Beglaubigung (Notarisierung); ein gescheitertes Fenster endete in einem
Panic -- seit demselben Tag eine Meldung (naechster Absatz). Tests `tests/pruef/werkzeug_paket.dhtest`
(8: Linux-Ordner, macOS-Bundle samt .icns-Eintraegen, Symbole passen zum
Logo, unbekanntes System, install.sh ein/aus mit fremdem dhrt, der Starter
mit der echten Laufzeit zweimal, `DH_IDE_BEISPIELE` mit Gegenprobe); sieben
Verfaelschungen (CRLF, dhrt in Resources, `cp -R` ohne `-n`, fremdes dhrt
ueberschreiben, Entfernen ohne Menueeintrag, IDE ohne `DH_IDE_BEISPIELE`,
Symbole nicht mitkopiert) und zwei am Symbol-Werkzeug (falsche
.icns-Laenge, falsche Kante) lassen je genau ihre Faelle fallen.

**Kein Fenster: Meldung statt Panic (2026-09-19):** raylib-rs bricht mit
`panic!("Attempting to create window failed!")` ab, wenn `InitWindow`
scheitert (kein Bildschirm, kein passendes OpenGL) -- beim Nutzer kam eine
Rust-Rueckverfolgung an, Rueckgabe 101, nicht abfangbar. Jetzt faengt
`Graphics::fenster_bauen` (graphics.rs) den Panic mit `catch_unwind`, schaltet
dafuer kurz den Panic-Hook stumm (sonst stuende die Rueckverfolgung trotzdem
da) und liefert `Err`; `new`/`new_headless`/`new_transparent` geben
`Result` zurueck, und die drei Aufrufer in vm.rs machen daraus einen
gewoehnlichen Laufzeitfehler des BEFEHLS: `SCREEN: Kein Fenster moeglich
-- ...`, beim versteckten Fenster fuer Bildbefehle `LOADIMAGE: ...` bzw.
`IMAGE_NEW: ...` (nicht SCREEN, das es nie gab). Mit Zeile, per `TRY`
abfangbar, Rueckgabe 2. Unter Linux ohne `DISPLAY`/`WAYLAND_DISPLAY` sagt
die Meldung das und nennt `xvfb-run`, sonst "kein passendes OpenGL (3.3)"
und was ohne Fenster geht. **Pruefbar auf einem Rechner mit Bildschirm
ueber `DHRT_KEIN_FENSTER`**: der Schalter loest DENSELBEN Panic an
derselben Stelle aus, geht also durch das echte Fangen -- eine erste
Fassung gab einfach `Err` zurueck und haette am `catch_unwind` vorbei
geprueft. Der echte Fall laeuft im Paket-Lauf (Linux ohne xvfb, macOS ohne
OpenGL). "Kein Fenster moeglich" steht in beiden Erkennungslisten fuer
Maschinen ohne Bildschirm (`pruefsammlung::KEIN_FENSTER`, `conftest.py`).
Tests `tests/pruef/kein_fenster.dhtest` (4, geben die Meldung nie selbst aus,
sonst hielte der Laeufer sie fuer eine Maschine ohne Bildschirm); drei
Verfaelschungen (ohne `catch_unwind`, Hook nicht stumm, Bildbefehl ohne
eigenen Namen) lassen je genau ihre Faelle fallen.

**Dateien vom Finder und ins Fenster gezogen (2026-09-19):** macOS gibt eine
per Doppelklick geoeffnete Datei NICHT als Argument weiter, sondern als
Apple-Event an die laufende App -- NSApplication reicht es an seinen Delegate
(`application:openURLs:`), und das ist GLFWs `GLFWApplicationDelegate`, der die
Methode nicht kennt: das Ereignis ging still verloren. `finder.rs` reicht sie
per `class_addMethod` nach, BEVOR GLFW in `glfwInit` sein `[NSApp run]` dreht
(dort kommt das Start-Ereignis an) -- nur die Objective-C-Laufzeit, keine
Crates, darum ungegatet und von jedem macOS-Lauf der CI mit uebersetzt; die
Pfade gehen in eine Warteschlange. `Info.plist` meldet `.dh` als eigenen Typ
`de.drachenhauch.quelltext` an (Klartext/Quelltext) und die App als dessen
Editor. **Dabei fiel ein alter Fehler auf:** `FILES_DROPPED`/`FILE_DROPPED`
holten die Liste bei JEDEM Aufruf bei raylib ab, und raylib-rs gibt sie dabei
frei (`Drop = UnloadDroppedFiles`) -- nach `FILES_DROPPED()` lieferte
`FILE_DROPPED(0)` leer, `examples/115_modplayer.dh` hat so nie einen Pfad
bekommen. Jetzt sammelt `flip` je Bild EINMAL ein (raylib + Finder, Feld
`abgelegt`), und die Liste gilt dieses eine Bild. **Die IDE** oeffnet
abgelegte Dateien ueber dieselbe Weiche wie der Projektbaum (`dateiOeffnen`:
Reiter, Form-Designer oder System), ein Ordner wird das Projekt; waehrend
eines Kastens warten die Pfade (`abgelegteNachziehen`). Das stand in der
Luecken-Tabelle von `docs/entwurf-python-abbau.md` laengst als erledigt,
in `ide.dh` gab es aber keine Zeile dafuer. **Pruefung:**
`DHRT_ABLEGEN` (Pfade mit `;`) speist DIESELBE Warteschlange wie der Finder,
geprueft wird also auch das Abholen; `DHRT_OEFFNEN_PROTOKOLL` schreibt jeden
vom Finder uebergebenen Pfad mit. **Im Paket-Lauf auf macOS belegt:**
`lsregister` kennt den Typ, und `open -a Drachenhauch.app datei.dh` (derselbe
Weg ueber LaunchServices wie der Doppelklick) liefert den Pfad in der Laufzeit
ab -- durch den Shell-Starter und dessen `exec` hindurch. Dass die IDE ihn
dann als Reiter zeigt, ist auf einem echten Mac ungeprueft (die Laeufer haben
kein Fenster); unter Windows zeigt es der Test. Tests
`tests/pruef/abgelegte_dateien.dhtest` (2) plus zwei Zeilen in
`werkzeug_paket.dhtest`, Rust-Test `warteschlange_liefert_einmal`; vier
Verfaelschungen (Liste nicht je Bild geleert, Finder-Warteschlange nicht
abgeholt, IDE ohne Abfrage, Ordner wie Datei) lassen je genau ihre Faelle
fallen.

**Beglaubigung (Notarisierung) vorbereitet (2026-09-19):** Apple verlangt
fuer die Notarisierung, dass jedes Programm im Bundle mit **Hardened
Runtime** signiert ist -- ein Shell-Skript als Hauptprogramm (bis dahin
`installer/posix/drachenhauch` in Contents/MacOS) kann das nicht. **dhrt ist
jetzt selbst das Hauptprogramm** (`CFBundleExecutable` = dhrt): gestartet OHNE
Argumente (bzw. nur mit dem alten `-psn_...`) aus `<Name>.app/Contents/MacOS/`
mit einer IDE unter `Contents/Resources/ide/` kopiert es die Beispiele (nur
Fehlendes), setzt `DH_IDE_BEISPIELE`/`DH_IDE_WURZEL`, wechselt dorthin und
startet die IDE (`appstart.rs`, 4 Rust-Tests). Erkannt am Aufbau des Ordners,
nicht am System -- darum auch unter Windows pruefbar: der neue Fall in
`werkzeug_paket.dhtest` baut das Bundle mit der echten Laufzeit und startet
dessen `dhrt` ohne Argumente (Gegenprobe ohne den App-Start: faellt); mit
Argument bleibt es ein gewoehnliches dhrt. Der Linux-Starter bleibt ein
Skript. **`bauen.dh` signiert immer mit Hardened Runtime** (ad hoc:
`flags=0x10002(adhoc,runtime)`, im Paket-Lauf geprueft -- und dhrt laeuft
damit: Konsolenprogramm, App-Start, Datei vom Finder). Mit `DH_MAC_SIGNATUR`
per Developer ID (`--timestamp`, auch das `.dmg`), mit `DH_NOTAR_KEY`/
`_KEY_ID`/`_ISSUER` (App-Store-Connect-API-Schluessel) ueber `xcrun notarytool
submit --wait`, bei Ablehnung mit dem Protokoll des Auftrags, dann `stapler
staple` + `validate`. Zugangsdaten NUR aus der Umgebung. Der Paket-Lauf
richtet sie aus fuenf GitHub-Secrets ein (`MAC_ZERT_P12`, `MAC_ZERT_PASSWORT`,
`MAC_NOTAR_KEY_P8`, `MAC_NOTAR_KEY_ID`, `MAC_NOTAR_ISSUER`; kurzlebiger
Schluesselbund), nur wenn es sie gibt, und prueft dann `spctl --assess`.
Anleitung fuer das Apple-Konto in `installer/README.md`. **Ungeprueft, bis es
die Secrets gibt:** der Weg mit echter Identitaet und die Antwort von Apple.
Das LIESMICH nennt fuer nicht beglaubigte Fassungen auch den Weg ab macOS 15
(Systemeinstellungen -> Datenschutz & Sicherheit -> "Dennoch oeffnen"; der
ctrl-Klick reicht dort nicht mehr).

**Die letzten vier pytest-Dateien (2026-09-20, Weg D):** von den 95
verbliebenen pytest-Dateien pruefen nur vier etwas, das den Python-Abbau
UEBERLEBT (`docs/entwurf-python-abbau.md` 7.6) -- alle vier sind jetzt
Sammlungen, **keine ist aufgegeben worden**. (1) `test_dhrt_test.py`: der
Anker laeuft in der CI direkt (`dhrt test tests/pruef`, neuer Schritt VOR
pytest in beiden Job-Familien), das Format prueft
`tests/pruef/dhrt_test_format.dhtest` (3). **Die innere Sammlung schreibt
jeder Fall SELBST** (`WRITEALL` + `JOIN$`) statt als `--- datei`-Beilage:
ihre Zeilen beginnen mit `===` und `---`, und der aeussere Laeufer haelt
jede solche Zeile fuer seinen eigenen Trenner. (2) `test_drucken.py`:
`tests/pruef/drucken.dhtest` (+1, `--- system windows`, ueberspringt sich
ohne "Microsoft Print to PDF"). (3) `test_os_builtins.py`:
`tests/pruef/os_builtins.dhtest` (+2). (4) `test_midi_module.py`:
`tests/pruef/modules_midi.dhtest` (+6, Selbst-Ueberspringen je Stufe --
Feature, Ausgang, Loopback-Port).

**Neu dafuer `EXEPATH$()`** (builtins.rs neben `CWD$`): der Pfad der
laufenden Programmdatei -- unter `dhrt run` die Laufzeit, in einem
exportierten Spiel dessen eigene Exe. **Beide os_builtins-Faelle hingen an
DIESEM Loch:** `PROCESS_START` kennt "dhrt" als Namen der eigenen Laufzeit,
`SHELL`/`SHELL_OUT$` aber nicht, und wer seine Beilagen NEBEN der Exe sucht
(statt im Startverzeichnis), fand sie gar nicht. Zwei Fallen beim Bauen des
Reihenfolge-Falls: `SHELL` gibt Anfuehrungszeichen als `\"` weiter (die
msvcrt-Regel), was cmd nicht versteht -- die Umleitung steht darum in einer
SKRIPTDATEI, die der Fall schreibt; und cmd sucht bei gesetztem
`NoDefaultCurrentDirectoryInExePath` nicht im Arbeitsverzeichnis, der Aufruf
braucht `.\`.

**Der PDF-Leser liest jetzt auch FREMDE PDFs** -- `pdftext.dh` ist aus
`pdf.dhtest` nach `tests/pruef/_hilfen/pdftext.dh` gezogen (die Sammlung ist
dadurch 254 Zeilen kuerzer; eingebunden per Zweizeiler-Beilage mit
`IMPORT "{sammlung}/_hilfen/..."`, weil Platzhalter nur in Text-Beilagen
gelten). Gemessen an der Datei von "Microsoft Print to PDF": jeder einzelne
Unterschied liess ihn vorher LEER ausgehen -- Leerraum zwischen Schluessel
und Wert (`/Type /Pages` statt `/Type/Pages`, dafuer `pdfNach`/`pdfIst`),
`/Contents` als Feld (`[ 20 0 R ]`), Zeichenketten in Hex (`<0035>` statt
`(..)`), Stroeme ohne `/Filter` und mit CR LF hinter `stream`, `/Length`
statt Suche nach `endstream` (**gesucht wird `"/Length "` MIT Leerzeichen** --
sonst trifft es `/Length1`, die Groesse der eingebetteten Schriftdatei).
Dazu **Textstellen**: der Leser verfolgt `cm` und `Tm`/`Td` und fuellt
`pdfStellen` mit "seite|x|y|text" in PDF-Punkten. Damit prueft der Druck-Fall
die Lage des Betrags DIREKTER als vorher mit PyMuPDFs Textkaesten: er
vergleicht sie mit `190 - PDF_TEXT_WIDTH(...)`, also genau mit dem, was das
Programm gerechnet hat. Gegenprobe, dass der krilla-Weg unveraendert liest:
alle 28 Faelle in `pdf.dhtest` bleiben gruen.

**Der Druck-Fall fasst den Standarddrucker nicht an**, prueft aber, dass er
sich nicht verschoben hat: hat Windows "den zuletzt verwendeten Drucker als
Standard festlegen" eingeschaltet, verschiebt ihn jeder Druck -- dann faellt
der Fall auf, statt es stillschweigend zu tun. (Eine Ruecksetzung waere
selbst ungeprueft, weil die Einstellung hier aus ist.)

**Der MIDI-Loopback lief doch scharf** -- der Nutzer hatte loopMIDI laufen,
es fehlte nur der Port. Und damit zeigte sich sofort, was ohne ihn verborgen
blieb: **ein MIDI-Port ist ein geteiltes Betriebsmittel.** Die beiden
Loopback-Faelle oeffnen DENSELBEN Port, und parallel landen die Noten des
einen im Eingang des anderen -- der erste Fall bekam `an 1 96 100`, die
aelteste ueberlebende Note aus dem Pufferdeckel-Fall. Die Sammlung ist
seither `--- seriell`. Gegenprobe am Modul (`pop_front` -> `pop_back`, also
"die juengste faellt weg"): die erste ueberlebende ist dann 20 statt 96, der
Fall faellt. Ohne das `midi`-Feature ueberspringen sich alle sechs
Geraete-Faelle mit Begruendung, die CI bleibt also gruen.

**Nebenbei gemessen, nicht Teil dieser Runde:**
`werkzeug_notenblatt.dhtest` (Fall "zweite spur und instrument") fiel im
`--hardware`-Bau 3 von 8 Mal, im Standardbau 0 von 8. Die Last war bei
beiden Messungen nicht gleich, der Vergleich ist also nicht isoliert --
aber wer mit `--hardware` baut, sollte mit sporadisch roten Klick-Faellen
rechnen.

**Was der Anker verdeckt hat, und was ihn ersetzt.** Der erste CI-Lauf mit
dem direkten Aufruf meldete **24 Faelle rot, die tags zuvor ueber den
pytest-Anker gruen waren** -- bei GLEICHER Image-Version
(`windows-2025-vs2026`) und, nachgestellt, demselben Aufruf. Drei Ursachen,
alle gemessen:
(1) **Der Runner hat mal einen OpenGL-Treiber und mal nicht** (`WGL: The
driver does not appear to support OpenGL`). Getragen wird das jetzt von der
Ueberspring-Erkennung -- die lag bis dahin nur im Zweig `code != 0` und traf
damit genau die Faelle NICHT, die ein KIND starten: die fangen dessen
Scheitern ab, geben es als Text aus und enden selbst mit 0. Entschieden wird
seither am ERGEBNIS (`bewerten` als Huelle um `bewerten_roh`): nur was sonst
FEHL waere, wird uebersprungen -- ein Fall, der die Meldung ERWARTET
(`kein_fenster.dhtest`), bleibt gruen, sonst waere er stillschweigend nie
geprueft worden. In `KEIN_FENSTER` steht jetzt auch der SCHLUSSSATZ derselben
Meldung: wer sie in eine Tabellenspalte schneidet, verliert den Anfang.
(2) **`DH_OHNE_AUDIO=1` setzte nur conftest.py**, und der Anker erbte es --
der neue Schritt setzt es selbst, sonst sucht jedes Programm mit Ton ein
Geraet, das der Runner nicht hat.
(3) **`core.autocrlf=true`** auf GitHubs Windows-Images: die Byte-Vergleiche
gegen eingecheckte Dateien bekamen CRLF aus dem Checkout, waehrend das
Werkzeug LF schreibt (`xsb hat CRLF: True`) -- dafuer `.gitattributes` mit
`eol=lf` fuer `*.xsb`, `i18n/*.json` und die zwei circuitrunner-Dateien.
**Die Reihenfolge war es nicht** (dieselben sieben Dateien fielen vor wie
nach den pytest-Schritten), und die Fehlerausgabe des Kindes warf
`_hilfen/uialeser.dh` bis dahin WEG -- ohne sie sah ein Fall gar nicht, dass
sein Kind an SCREEN gestorben ist, und meldete einen Fehler ueber etwas ganz
anderes (jetzt `auslesenFehler`). Gegenproben: ohne die Huelle faellt der
Rust-Test; mit `DHRT_KEIN_FENSTER=1` werden die 24 uebersprungen, **ohne den
Schalter laufen dieselben 45 Faelle scharf durch** (0 uebersprungen) -- die
Reparatur legt nichts still.

**Der Installer ohne Python (2026-09-16):** `installer/bauen.dh` verpackt die
Python-freie Distribution -- Fassung aus `VERSION$()` (gepackt wird genau die
`dhrt.exe`, deren Nummer im Installer steht), Lizenzen ueber
`installer/lizenzen.dh`, dann ISCC auf `Drachenhauch-IDE.iss` (aus `%ISCC%` oder
zwei Standardpfaden; fehlt es, bleibt es bei der Lizenzdatei plus Hinweis).
**Das Bauen der Laufzeit bleibt bei `rust/build_runtime.py`** -- unter Windows
laesst sich eine laufende `.exe` nicht ueberschreiben, und `bauen.dh` laeuft in
ihr. `installer/lizenzen.dh` ersetzt `gen_notices.py` fuer diese Distribution:
Crates aus `cargo metadata` (Features `graphics db net http`), je Crate die
LICENSE-/COPYING-/NOTICE-Dateien aus dem Crate-Ordner und dessen `licenses/`
und `license_files/`, dazu der MPL-2.0-Volltext; Python-Pakete und Qt fehlen,
weil in dieser Distribution keins von beidem steckt. **Der Beleg ist ein
Vergleich, keine Behauptung:** der Abschnitt RUST-KOMPONENTEN ist Byte fuer Byte
der von `gen_notices.py` (370 Crates, 3,2 MB, 1,5 s). Drei Dinge mussten dafuer
genau stimmen: die Reihenfolge ist nach KLEINGESCHRIEBENEM Namen sortiert und
stabil (sortiert wird eine Liste von Plaetzen, damit die Parallel-Felder
zusammenbleiben), die Lizenzdateien werden nach dem KLEINGESCHRIEBENEN Pfad
geordnet (so vergleicht pathlib unter Windows), und ein Text wird wie Pythons
`read_text()` gelesen (`BUFFER_TO_STRING$` + CRLF -> LF; `READLINES` verloere
den abschliessenden Umbruch, den der MPL-Volltext braucht) und wie `strip()`
beschnitten. `THIRD-PARTY-NOTICES-IDE.txt` ist erzeugt und gitignoriert wie ihr
Qt-Gegenstueck -- `bauen.dh` schreibt sie vor jedem Verpacken neu. Tests
`tests/pruef/werkzeug_installer.dhtest` (3, mit erfundener `cargo metadata` und
Attrappe statt cargo; Gegenprobe ohne Kleinschreibung beim Sortieren faellt).
**Falle:** es gibt keinen Befehl fuer den Namen des Betriebssystems -- `bauen.dh`
sucht darum einfach beide Dateinamen (`dhrt.exe`, `dhrt`).
**Seit 2026-09-21 hat er alles, was nur der Qt-Installer hatte:** die Buecher
(`.docx`/`.epub` nach `{app}\buecher`, wenn gebaut), die ESP32-Sketche, das
Aufraeumen von GameBasic (Ordner, Verknuepfungen, ProgID, `.gb` nur wenn es noch
auf uns zeigt) und das Signieren ueber `GB_SIGN_CERT`/`_PASS`/`_TS`. Signiert
wird eine KOPIE der Laufzeit (`<ausgabe>/stufe/dhrt.exe`, an ISCC als
`/DDhrtQuelle`) -- die gebaute laeuft womoeglich gerade als `bauen.dh` --, dann
der Installer; ein Fehlschlag bricht VOR ISCC ab statt nur zu warnen. Tests mit
Attrappen fuer ISCC/signtool (`.cmd`, die `PROCESS_START` direkt startet) in
`tests/pruef/werkzeug_installer.dhtest`; vier Verfaelschungen fallen.
**Logo und Schriftzug (2026-09-21):** `dhrt.exe` traegt kein Symbol (ein
exportiertes Spiel ist eine Kopie davon) -- Verknuepfungen, `.dh`-Dateien und
die Deinstallation zeigten darum das nackte Programm. Der Installer legt jetzt
`drachenhauch.ico` nach `{app}` und nimmt es ueberall; dazu `daten/bilder/*.png`
(auch in den macOS-/Linux-Paketen). Die IDE laedt beides aus
`<wurzel>/daten/bilder/`: das Logo als `WINDOW_ICON`, der Schriftzug steht auf
der Willkommensseite statt des Titels als Text (vorab auf 240x112 skaliert,
der Satz rechts daneben, alles ab den Knoepfen bleibt an seiner Stelle); ohne
die Dateien bleibt es beim Text. Tests `tests/pruef/werkzeug_ide_schriftzug.dhtest`
(3) und zwei Zeilen in `werkzeug_paket.dhtest`.
**Kachel als Widget, Mausrad, Bereichsfarben (2026-09-21, Hinweise des
Nutzers):** (1) **`GUI_CARD(win, x, y, w, h, bild[, titel$[, text$]])`** --
Bild oben (16:10 der Innenbreite, `fuellen`), Titel kraeftig, Beschreibung in
der Schrift des Widgets umgebrochen; die GANZE Kachel ist ein Knopf (Klick
beim Loslassen, Enter/Leertaste, a11y-Rolle Button mit Beschreibung). Vorher
baute die IDE jede Kachel aus Bild + Knopf + Beschriftung in 12 px, und nur
der schmale Knopf nahm den Klick an. `Kind::Card` teilt sich mit
`Kind::Button` Druck, Loslassen, Tastatur und a11y-Klick; die Beschreibung
liegt im `placeholder`-Feld (die .dhform schreibt es schon), das Bild im
`sel`-Feld wie beim Bild. Passt die Beschreibung nicht, rueckt der Rest in die
letzte sichtbare Zeile und wird dort mit "..." gekuerzt -- sonst endete ein
Text, dessen zweites Wort nicht passte, nach dem ersten. `GUI_CARD_SET_TEXT`/
`GUI_CARD_TEXT$`, `GUI_SET_IMAGE`/`GUI_IMAGE_MODE` gelten auch fuer Kacheln;
im Form-Designer Palette, Feld "Beschreibung" und GB-Code. (2) **Das Mausrad
rollte alles zugleich**: die IDE blaetterte ihre Kacheln, sobald die Maus im
Kachelrechteck stand -- auch unter dem offenen Handbuch. Neu
**`GUI_WINDOW_AT(x, y)`** (oberstes sichtbares Fenster, -1 = keins); die IDE
blaettert nur, wenn dort `win` oben liegt. (3) **Bereichsfarben**
(`bereichsFarben` in ide.dh): Projekt/Gliederung/Zuletzt blau, Ausgabe und
Eingabe gruen, Probleme orange, Debugger violett -- `GUI_SET_COLOR(..., "bg")`
aus `GUI_THEME_GET("widget_bg")` gemischt (dunkel 20 %, hell 16 %), nach jedem
Themawechsel neu. Tests `tests/pruef/gui_kachel.dhtest` (6), ein Fall in
`werkzeug_showcase.dhtest` (Rad unter dem Handbuch; ohne die Fensterfrage
faellt er) und einer in `werkzeug_ide_schriftzug.dhtest` (Farbwert beider
Themen).

**Lesbarkeit der IDE (2026-09-21, Hinweise des Nutzers):** (1) **Umlaute in
der Oberflaeche** -- 174 Anzeigetexte (Menue, Knoepfe, Tooltips, Statuszeile,
Palette) schreiben jetzt ä/ö/ü/ß; Kennungen, `protokoll`-Zeilen, `CASE`-Namen
und Konfig-Schluessel bleiben ASCII (die Tests lesen sie). Die Suche in
Palette und Waehlern vergleicht ueber `ohneUmlaute$` -- wer `oeffnen` tippt,
findet "Öffnen". (2) **Codefarben je Thema**: die `F_*` sind keine CONST mehr,
`codeFarben(hell)` setzt sie nach dem Laden und bei jedem Themawechsel (und
leert `faerbSchluessel`/`gliederungSchluessel`, sonst bliebe das Alte stehen)
-- vorher standen die hellen Toene des dunklen Themas auch auf Weiss. (3) **Die
Gliederung ist zweifarbig**: neu in der Laufzeit **`GUI_LISTBOX_SPANS(lb,
eintrag, starts, laengen, farben)`** (`ListState.spans`, in allen
Umordnungen mitgefuehrt; gezeichnet werden Laeufe ueber die Breite des
Vorspanns wie im Textbereich; auf Auswahl und gesperrt gilt eine Farbe). (4)
**Die Zeilenhoehe einer Liste folgt ihrer Schrift** (`list_zeile_h`:
max(22, Schrift + 6), EINE Quelle fuer Zeichnen, Klick, Rad, Bild auf/ab,
a11y) -- vorher lief eine groessere Schrift in die naechste Zeile. (5)
**Gedaempfter Text ist lesbar**: `Gui::leise(bg)` mischt `muted_fg` gegen
`text_fg`, bis der Kontrast nach WCAG 4,5:1 erreicht (Platzhalter,
Zusatztext der Liste, Kachelbeschreibung) -- auf den getoenten Bereichen fiel
das Grau des dunklen Themas auf 3,2:1. (6) **Schriftgroessen** in den
Einstellungen: Listen (Gliederung, Ausgabe, Probleme; Vorgabe 18) und
Handbuch (16), in der ide.json `listen_schrift`/`handbuch_schrift`. (7)
**Info-Fenster mit Bild**: ein eigenes Fenster (`winUeber`) statt
`GUI_DIALOG` -- Symbol, Schriftzug, Text, OK (Enter/ESC). Tests
`tests/pruef/gui_liste_farben.dhtest` (4, Farben im Bild, Gegenprobe ohne
Abschnitte, Zeilenhoehe ueber Bild ab), `tests/pruef/werkzeug_ide_lesbarkeit.dhtest`
(8; die Gliederungsprobe sucht das Rechteck der Liste per `GUI_HIT_TEST`,
weil sonst die Schluesselwort-Farbe der Werkzeugleiste mitzaehlte), Rust-Test
`kontrast_nach_wcag`; sechs Verfaelschungen der IDE fallen je in ihrem Fall.

**Fett, kursiv, unterstrichen, durchgestrichen (2026-09-21, Frage des
Nutzers):** vorher waren `TEXT_BOLD`/`TEXT_ITALIC` No-Ops (raylib kennt
keine Schnitte), der gesetzte Text zeichnete Fett doppelt und Kursiv nur
gedaempft. Jetzt: **`TEXT_STYLE(stil$)`** / `TEXT_GET_STYLE$()` (Woerter
deutsch oder englisch, verbunden mit `+`/Komma/Leerzeichen; unbekannt =
Fehler mit Liste), `TEXT_BOLD`/`TEXT_ITALIC` schalten je ein Bit,
**`FONT_STYLE(font, stil$)`** / `FONT_HAS_STYLE`, **`GUI_SET_FONT_STYLE`** /
`GUI_GET_FONT_STYLE$` (in der `.dhform` als `font_style`, im Form-Designer
ein Feld "Schriftstil" samt GB-Code), im gesetzten Text zusaetzlich
`***beides***`, `~~durch~~`, `<u>unter</u>`; Verweise sind unterstrichen.
**Echte Schnitte, wo es sie gibt** (`schnitt.rs`, rein und mit Rust-Tests):
zu einer per LOADFONT geladenen Schrift sucht `schnitt_kandidaten` die
Datei daneben -- Windows-Familien nach Tabelle (segoeui -> segoeuib/i/z,
arial -> arialbd/ariali/arialbi ...), sonst `-Regular` -> `-Bold`/`-Italic`/
`-BoldItalic` (ersatzweise Oblique/Semibold); `Graphics::schnitt` laedt
einmal in derselben Groesse mit denselben Zeichen (`font_herkunft` je
Handle) und merkt auch Fehlschlaege (`schnitte`). Fehlt fett+kursiv, wird
der fette Schnitt genommen und nur die Neigung nachgebildet. **Sonst
Ersatz** (`Cmd::TextStil`, `zeichne_text_stil`): Fett = zweiter Zug um
`fett_versatz` (1 px je 16 px Schrift), Kursiv = Glyphenvierecke geschert
ueber rlgl (`zeichne_schraeg`, raylibs DrawTextCodepoint nachgebaut --
raylib kann drehen, nicht scheren), Linien als Rechtecke nach
`linie_unter`/`linie_durch`. Ohne Stil bleibt es beim alten `Cmd::Text`, jedes
bestehende Programm zeichnet wie zuvor (ausser denen mit TEXT_BOLD, die jetzt
wirklich fett sind). Das Messen (`text_width_stil`, `&self`) kennt nur schon
geladene Schnitte -- darum laedt `Gui::schnitte_laden` sie in GUI_UPDATE vor
dem Setzen, und `wfont` gibt der gui den Schnitt-Handle, damit Schreibmarke
und Auswahl mit der Schrift messen, die dasteht. GFX_PUSH/POP nimmt den Stil
mit. Tests `tests/pruef/schriftstil.dhtest` (7: Stil-Rundweg samt PUSH/POP,
Fehler, eingebaute Schrift ohne Schnitte, Segoe mit Schnitten und Tahoma ohne
Kursiv, der Ersatz am Bild -- mehr Punkte, Neigung ueber die mittlere x-Lage
oben gegen unten, Linien --, Widget samt `.dhform`, Widget-Linie im Bild),
zwei Faelle in `werkzeug_formdesigner.dhtest`, Rust-Tests in `schnitt.rs` und
`auszeichnungen_als_stil`; sieben Verfaelschungen der Laufzeit fallen je in
ihrem Fall.

**Akkordeon, Assistent, Baumtabelle (2026-09-21, Frage des Nutzers nach
fehlenden Widgets):** die letzten drei Punkte der Lueckenliste in
`docs/module-gui.md` -- sie steht jetzt leer. **`GUI_ACCORDION`**
(`Kind::Accordion`, `AkkState`: offen/hoehen/mehrere/lage/scroll) und
**`GUI_WIZARD`** (`Kind::Wizard`, `WzState`, Geometrie aus EINER Quelle
`wz_geom`) benutzen die Buchfuehrung des Reiterwerks (`tabctl.kinder`,
`tc_von`/`tc_seite`, `widget_shown`) statt einer eigenen. Beim Akkordeon
merkt `lage` die Lage eines Kindes RELATIV zu seinem Abschnitt, und
`akk_pass` (in GUI_UPDATE vor `layout_pass`) setzt die Fensterlage je Bild
neu -- klappt ein Abschnitt darueber auf, wandern die Kinder mit. **Beide
sind Luft fuer Klicks ihrer Kinder** (`luft_fuer_kinder` in `handle_press`):
der erste Treffer gewinnt, und der Behaelter liegt VOR seinen Kindern; ohne
die Ausnahme schluckte er den Klick. `GUI_HIT_TEST` liefert auf freier
Flaeche weiter den Behaelter (so waehlt ihn ein Designer an). Der Assistent
prueft mit `pruefen` nur die Felder DES Schritts (`wz_weiter` ->
`widget_pruefen`), das erste falsche bekommt den Fokus; Ereignisse
(`CHANGED/FINISHED/CANCELLED`) gelten ein Bild lang. **Die Baumtabelle ist
eine Tabelle mit Schalter `baum`** (`GUI_TREETABLE`, `TableState`
eltern/offen/ebene/hat_kinder, `rebuild_baum`), damit Sortieren, Filter,
Zellarten und Auswahl nicht doppelt entstehen: sortiert wird unter
Geschwistern, ein Filter zeigt Vorfahren eines Treffers mit, `remove_row`
haengt Kinder an die Eltern der entfernten Zeile, ein Kreis bei
`SET_PARENT` ist ein Fehler. Dreieck-Klick klappt ohne zu waehlen,
Rechts/Links klappen bzw. springen. **Der Fund dabei ist aelter:** eine
Tabelle aus `GUI_FROM_JSON`/`GUI_LOAD` zeigte KEINE Zeile, bis etwas
Sortieren oder Filtern anstiess -- der Lader baute `view` nie
(`ts.rebuild_view()` fehlte); Fall "eine geladene flache tabelle zeigt ihre
zeilen". Form-Designer: drei Palettenarten (Baumtabelle als `table` mit
`baum`), Abschnitte/Schritte im Feld "Eintraege", GB-Code. Tests
`tests/pruef/gui_akkordeon.dhtest` (6), `gui_assistent.dhtest` (7, einer davon liest den geladenen Assistenten -- der erste Satz Faelle pruefte nur das Schreiben, und die Verfaelschung des Lesers blieb gruen),
`gui_baumtabelle.dhtest` (7); zehn Verfaelschungen der Laufzeit (ohne
Kinder-Ausnahme, ohne Mitwandern, ohne Pruefung, ohne Vorfahren im Filter,
Kinder beim Entfernen nach oben, ohne Dreieck, drei .dhform-Schluessel,
ohne `rebuild_view` beim Laden) fallen je in ihrem Fall.
**Beispiel** `examples/200_gui_akkordeon_assistent.dh` (2026-09-21, Wunsch
des Nutzers): eine Projektablage mit allen dreien -- Einstellungen im
Akkordeon wirken sofort auf die Tabelle, der Assistent legt Eintraege in
den gewaehlten Ordner (Pflichtfeld, Groesse 1..99999, bei Ordnern
gesperrt). **Der Fund dabei steckt in der Laufzeit:** die Knoepfe des
Assistenten waren fest 112 Punkte breit, in einem 298 Punkte breiten lag
Abbrechen halb ueber Zurueck -- gesehen nur im BILD. `wz_geom` teilt jetzt
die Breite (hoechstens 112, mindestens 40). Tests
`tests/pruef/beispiel_drei_widgets.dhtest` (4: Eintrag ueber den
Assistenten samt Pflichtfeld und Bereich, Ordner ohne Groesse, Filter,
echter Klick auf einen Akkordeon-Kopf), ein Fall mehr in
`gui_assistent.dhtest` (Klick bei x 190 trifft Zurueck; mit festen Knoepfen
lag dort Weiter) und das Beispiel in `beispiele_gui_enden.dhtest`.

**Stufe 53 (2026-09-14):** die CIRCUIT-RUNNER-Engine ohne Python -- die fuenf
Engine-Tests aus `test_circuitrunner.py` stehen in
`tests/pruef/werkzeug_circuitrunner.dhtest`; in pytest bleiben nur die drei
Tests der Python-Werkzeuge `convert_dat.py` und `make_demo_levels.py`, und die
Datei steht nicht mehr in `_BRAUCHT_GRAFIK`. pytest hatte die Engine-Quelle
vor `WHILE NOT QUITREQUESTED()` abgeschnitten und in einem Ordner mit den
Assets laufen lassen; jetzt nimmt jeder Fall die Engine per `--- programm`,
schiebt sich hinter den Kommentar der Hauptschleife und endet mit `EXIT(0)`.
**Zwei Stellen, die das brauchte:** die Engine laedt `assets/...` relativ, und
ihre Programmkopie liegt in `_programm/` -- `--- ersetzen` auf `CONST
SAVEPATH` wechselt darum per `CHDIR` in den Ordner des Spiels UND legt die
Sicherung in den Fallordner (sonst schriebe jeder Lauf ueber die echte
`circuitrunner.save`; ihr Datum ist nachgesehen unveraendert). Die Level baut
`tests/pruef/_hilfen/circuitlevel.dh` mit dem json-Modul und schreibt sie in
den Fallordner (`DHRT_START_DIR`); seine Namen beginnen mit `pr`, weil der
Einschub mitten in der Engine steht und deren Globale nicht verdecken darf.
Eingebunden wird er ueber eine Beilage in `_programm/`, weil ein relativer
IMPORT dort aufgeloest wird, wo die Quelle liegt. Die Zeiten sind
Wahrheitswerte in der Ausgabe (30 s +-1). Alle fuenf liefen beim ersten Mal --
Gegenprobe gegen eine Engine-Kopie, in der `reorder_monsters`,
`find_password`, `world_tick`, `tat` und `best_for` sofort zurueckkehren:
5 von 5 fallen.

**Stufe 52 (2026-09-14):** Debugger, VM-Haertung und der geraetefreie Teil
von MIDI ohne Python -- `test_dhrt_debug.py` (6) und `test_vm_hardening.py` (2)
sind geloescht, aus `test_midi_module.py` sind sieben Tests gezogen. **Debugger:**
`tests/pruef/_hilfen/debugsitzung.dh` startet `dhrt debug` per PROCESS_START,
schreibt die JSON-Kommandos, schliesst stdin und liest die Ereignisse mit dem
json-Modul (`art$`, `feld$`, `paar$` fuer globals/locals, `ersteMit`,
`ausgaben$`); `tests/pruef/dhrt_debug.dhtest` (6). **VM-Haertung:** das Geruest
kommt von `dhrt --dumpbc`, `JSON_SET_JSON` setzt `main.code` auf ADD mit leerem
Stapel, der Fall startet es als `.dhc` und prueft Rueckgabe, Meldung und dass
kein Panic kam (`vm_haertung.dhtest`, 2). **MIDI:** Registrierung, Handle-Typen,
Notennamen, Frequenzen und "Auflistung oder klare Meldung" -- der Bau sagt in
der `dabei:`-Zeile von `dhrt --version`, ob er das Feature hat
(`modules_midi.dhtest`, 6). In pytest bleiben nur die Faelle mit echtem oder
virtuellem Anschluss (Wertebereiche mit Feature, Senden, Loopback-Kreis,
Pufferdeckel). Alle 14 Faelle liefen beim ersten Mal -- Gegenprobe mit
kaputten Kopien (Debugger startet `run` statt `debug`, gueltiger statt
kaputter Bytecode, andere Notennummern plus ein erfundener Befehlsname):
14 von 14 fallen.

**Stufe 51 (2026-09-14):** Webserver, MQTT und INPUT ohne Python --
`test_httpd.py` (16), `test_mqtt_module.py` (4) und `test_input_stdin.py` (4)
sind geloescht. **httpd mit vertauschten Rollen:** in pytest war dhrt der
Server und Python der Client; jetzt ist der Server die Beilage `srv.dh` als
Kind (Port ueber `EPRINT("PORT=...")`), und der Fall ist der Client ueber
`tests/pruef/_hilfen/httpklient.dh` -- ROH ueber das net-Modul, weil ein
fertiger Client `/../geheim.txt` wegnormalisiert haette (`httpd.dhtest`, 16).
**MQTT:** `_hilfen/mqttbroker.dh` ersetzt den Python-Broker (CONNACK,
PINGRESP, SUBACK, PUBLISH zurueck an den Absender) mit NET_RECV_BYTES und
Puffern (`modules_mqtt.dhtest`, 2; die neun Befehlsnamen stehen in
`builtin_registrierung.dhtest`). **INPUT:** drei Faelle mit `--- eingabe` in
`stdin.dhtest`; pytest ging dafuer ueber `dhrun.py`. Der leere INPUT auf eine
Zahl ist ein Laufzeitfehler -- der Fall prueft Abbruch UND dass der Prompt
vorher dastand. **Zwei Stolpersteine beim Schreiben:** (1) `PROCESS_ERR$`
liefert, was im Puffer liegt, also auch MEHRERE Zeilen -- bricht ein Server
gleich nach der Portzeile ab, steht seine Meldung in derselben Portion, und
`VAL` darauf ist 0; der Klient zerlegt jetzt in Zeilen und wartet nach dem
Ende fuenf leere Abfragen ab. (2) Ein SUBACK aus `BUFFER_NEW(5)` traegt den
Rueckgabecode 0 schon -- ein angehaengtes weiteres 0-Byte las der Client als
Paket vom Typ 0, und die Nachricht kam nie an. Gegenprobe ohne Server bzw.
Broker: 16 von 16 und 1 von 2 fallen (der uebrige prueft den Fehler ohne
Broker).

**Stufe 50 (2026-09-14):** zwei grosse Golden-Dateien, die beim Umzug von
2026-09-08 stehen geblieben waren -- `test_language_extensions.py` (59 Tests)
und `test_dhrt_builtins.py` (34, einige mit mehreren Laeufen) sind geloescht.
Neu `tests/pruef/sprach_erweiterungen.dhtest` (52 Faelle) und
`tests/pruef/dhrt_builtins.dhtest` (44); die drei Tests, die nur das
Befehlsverzeichnis fragten (TIMER, JOYSTICK_*, INKEY$/WAITKEY), sind ein Fall in
`builtin_registrierung.dhtest`. Ein Generator zog Quelltext und Zusagen aus dem
Syntaxbaum, liess jedes Programm laufen, pruefte `==`/`in`/`raises` am echten
Lauf und nahm erst dann die GANZE Ausgabe als Erwartung -- strenger als die
Teiltext-Zusagen davor. Zwei Zusagen waren nicht zu uebernehmen: das
Python-Suchmuster `Parameter|Argument` (die Meldung heisst `foo: Parameter 'a'
fehlt`), und ein unterminierter f-String meldet mit folgendem Zeilenumbruch
etwas anderes als am Dateiende -- in einer Sammlung folgt immer einer, der Fall
prueft nur `f-String`. Die drei doppelten `vm`-Faelle (einst der Python-VM-Pfad)
fielen weg. Gegenprobe mit `EXIT(0)` als erster Zeile jedes Falls: 91 von 98
fallen; die 7 uebrigen sind die Lexer-, Parser- und Compile-Fehler, die vor
jeder Zeile des Programms abbrechen.

**Stufe 49 (2026-09-14):** die letzte der Restdateien aus Stufe 44 bis 48 --
`test_smtp.py` ist geloescht, ihre 18 Tests stehen in `tests/pruef/smtp.dhtest`
(jetzt 26 Faelle). Zwei Helfer unter `tests/pruef/_hilfen/`: **`smtpserver.dh`**,
der `_MiniServer` in Drachenhauch (eine Verbindung; Faehigkeiten, Anmeldung und
Empfaenger ueber Argumente; Befehle nach `befehle.txt`, die Nachricht nach
`daten.txt`, beides VOR der Antwort geschrieben), gestartet ueber
`smtpstart.dh` (`starten`/`befehl`/`daten$`/`beenden`); und **`mime.dh`** statt
Pythons `email`-Modul -- Kopfzeilen entfaltet, RFC-2047-Woerter dekodiert
(Leerraum zwischen zwei Woertern faellt weg), Teile nach der Grenze aus dem
Parameter, Rumpf als Bytes. Eingebunden wird beides ueber eine zweizeilige
Beilage mit `IMPORT "{sammlung}/_hilfen/..."` -- Platzhalter gelten in
Text-Beilagen, im Quelltext nicht. **Der Fund:** ein langer Umlaut-Betreff
wurde zu Zeilen mit 81 Zeichen gefaltet; RFC 2047 erlaubt fuer eine Zeile mit
kodierten Woertern 76, und `kodiere_wort` rechnete nur die 75 fuer das Wort,
nicht den Feldnamen davor. Pythons Leser nahm es klaglos -- der Fall prueft
jetzt die Zeilenlaenge, `smtp.rs` schneidet 39 statt 45 Bytes je Wort
(Rust-Test `gefaltete_betreffzeilen_bleiben_unter_76_zeichen`).

**Stufe 48 (2026-09-14):** die HTTP- und Cloud-Tests ohne Python -- und
dafuer ein Byte-Weg im net-Modul. `test_modules_html.py` (13), `test_http_request.py`
(29 mit den sechs Methoden) und `test_modules_cloud.py` (9) sind geloescht. Ihre
Mock-Server liefen als Faden im pytest-Prozess; ein Drachenhauch-Programm kann
neben einem blockierenden `HTTP_GET` nichts bedienen. Darum startet jeder Fall
den **Gegenserver `tests/pruef/_hilfen/gegenserver.dh`** als Kind
(`PROCESS_START`, Pfad ueber `--- argumente {sammlung}/_hilfen/...`): Modi
routen/abbruch/langsam/echo/cloud, freier Port in eine Datei im Fallordner,
mehrere Verbindungen zugleich (zwei langsame Abrufe in 322 statt 600 ms), und
nach 30 s ohne Verbindung beendet er sich selbst -- ein abgebrochenes
Pruefprogramm kann ihn nicht mehr beenden. **Der Fund dabei:** `NET_SEND` und
`NET_RECV` konnten nur UTF-8 -- ein Rumpf mit 00 FF oder eine Antwort mit rohen
Bytes liess sich in Drachenhauch weder senden noch empfangen. Neu nimmt
`NET_SEND` einen BUFFER, und `NET_RECV_BYTES` liefert einen; ein von `NET_RECV`
zurueckgehaltenes angefangenes Zeichen gibt es zuerst heraus (Loopback-Tests in
`modules_net.dhtest`). Die Programme zog der Generator aus dem Syntaxbaum
(f-Strings mit `{base}` werden `basis + "..."`, der Servermodus kommt aus dem
`with`), lief sie und verglich die Ausgaben mit den pytest-Zusagen, bevor sie
Erwartung wurden; Port und lokalisierte Fehlertexte stehen nur als Teilaussage.
Gegenprobe ohne Server: 46 von 51 fallen -- der zunaechst gruen gebliebene
Zeitgrenzen-Fall prueft jetzt, dass der Fehler nach rund einer Sekunde kommt
und nicht sofort. Offen in pytest: `test_smtp.py` (braucht einen SMTP-Gegenserver).

**Stufe 47 (2026-09-14):** die beiden `--check`-Dateien ohne Python --
`test_check_unbekannte_namen.py` (21 Faelle) und `test_compiler_warnungen.py`
(55) geloescht, ihre Faelle stehen in den gleichnamigen Sammlungen. Jeder Fall
laesst `dhrt --check` per `PROCESS_START` auf eine Beilage laufen, liest das
JSON (in ein Objekt verpackt, weil `JSON_LEN` einen Pfad braucht) und gibt
seine Zusagen als Wahrheitswerte aus. Die 55 Warnungsfaelle hat der Generator
nicht abgeschrieben, sondern aus dem Python-Syntaxbaum gezogen:
`any(A in m and B in m ...)` wird "eine Meldung mit [A|B]", `not any(...)`
dasselbe mit FALSE, `len([...]) == 1` eine Zaehlung; die Schleife ueber zwei
Quelltexte wird zu zwei Faellen. Beide Sammlungen liefen beim ersten Mal gruen
-- darum je eine Gegenprobe: mit einer Marke, die nie vorkommt, fallen 9 von 22
Namensfaellen, mit einer Datei, die es nicht gibt, 28 von 58 Warnungsfaellen
(genau die mit einer positiven Zusage). Offen in pytest: modules_cloud, smtp,
modules_html, http_request (brauchen einen Gegenserver neben dem Programm).

**Stufe 46 (2026-09-14):** dritter Block Restdateien aus Weg D -- 35 Faelle
an vier Sammlungen, die pytest-Dateien `test_tiled_objekte_eigenschaften`,
`test_gfx_push_pop`, `test_hintergrund` und `test_image_io` geloescht; dazu
Kira 0.12.3 -> 0.12.4 (nur Lockfile). Die fremden Leser: statt `TileMapDoc`
das json-Modul nach den Regeln des Tiled-Formats, statt Pillow
`LOADIMAGE`/`GETPIXEL`/`GETALPHA` (raylibs stb_image liest, was stb_image_write
schrieb) plus die Bytes (PNG-/BMP-Kopf; im GIF zaehlt `21 F9 04` die Bilder und
traegt die Dauer in Hundertsteln, `NETSCAPE2.0` die Wiederholung), statt
Pythons `sqlite3` ein `--- vorher` mit dem db-Modul. Die Bildvergleiche von
GFX_PUSH/POP laufen im `--- nachher` Punkt fuer Punkt, und jeder Fall prueft
zusaetzlich, dass die Stoerung WAEHREND des PUSH das Bild veraendert hat --
sonst waere "gleich" auch ohne Wirkung gruen. **Fund:** `TILED_SAVE` schreibt
Eigenschaften in HashMap-Reihenfolge (zwei Laeufe, zwei Reihenfolgen) und
`nextobjectid` bleibt 1, obwohl die Objekte die Nummern 1 und 2 tragen -- in
Tiled bekaeme das naechste Objekt eine doppelte Nummer. Die Leser der
Sammlung sortieren; die Behebung ist eine eigene Aufgabe. Offen in pytest:
modules_cloud, check_unbekannte_namen, smtp, modules_html, http_request,
compiler_warnungen.

**Stufe 45 (2026-09-14):** zweiter Block Restdateien aus Weg D -- 28 Faelle
an zehn bestehende Sammlungen, 8 pytest-Dateien geloescht
(`test_gui_form_runner`, `test_shader_uniforms_geometry`,
`test_fenster_prozess`, `test_automation`, `test_rust_run_parity`,
`test_m3d`, `test_namensraeume`, `test_kontaktbogen`), aus `test_pruefen.py`
zehn von elf Tests und aus `test_modules_audio.py` die zwei Seek-Tests. Kein
neues Formatstueck: Pillow-Pixelproben wurden `--- bild`, Bildmasse
`groesse`, die wandernde Kachel im Kontaktbogen ein `--- nachher` mit
`LOADIMAGE`/`GETPIXEL`; die Log-Tests, die pruefen, was NICHT in stderr
steht, lassen ein Kind per `PROCESS_START` laufen und sammeln
`PROCESS_READ$`/`PROCESS_ERR$` getrennt (`DH_LOG` ueber `SETENV`); der
Herzschlag des Fensterkinds wird im `--- nachher` zweimal mit
`SLEEP` dazwischen gelesen; Musikdateien kommen ueber
`--- argumente {sammlung}/../../examples/assets/...`. Die Zeitmessungen des
Fensterkanals sind Wahrheitswerte in der Ausgabe, `fenster_prozess.dhtest`
traegt dafuer `--- seriell`. Gegenprobe: ohne `AUTOMATION_PLAY` fallen beide
Klick-Faelle, ohne `SHADER_SET_*` alle drei Shader-Faelle. **Bleibt in
pytest:** die Reihenfolge von PRINT und LOG_* in einem gemeinsamen Strom und
die Synth-Mathematik in Python.

**Stufe 44 (2026-09-14):** Durchgang ueber die Restdateien aus Weg D --
Dateien, deren uebertragbare Faelle schon 2026-09-08 umgezogen waren und in
denen einzelne Tests hingen, die damals nicht gingen und mit den Mitteln der
Stufen 32 bis 43 jetzt gehen. 16 Faelle an die bestehenden Sammlungen
angehaengt, 8 pytest-Dateien geloescht (`test_pdf`, `test_audio_sound_io`,
`test_audio_note_mix`, `test_input_edges`, `test_variable_wie_builtin`,
`test_import_meldungen`, `test_stdin`, `test_dhrt_profile`), aus
`test_os_builtins.py` drei von fuenf Tests. Die Mittel: `--- nochmal` (PDF
zweimal gleich, mit einer Sekunde Abstand), WAV-Bytes im `--- nachher` (8 Bit
gegen 16 Bit, Nulldurchgaenge beim Portamento), `PROCESS_START("dhrt",
"--check"/"--dumpbc"/"profile"/"run", ...)` samt `--- datei base64` fuer die
BOM-Datei, `--- eingabe`/`--- fehler` fuer STDIN. **Zwei Stolpersteine:**
`--dumpbc` gibt EINGERUECKTES JSON ueber viele Zeilen aus (wer nur die Zeile mit
`{` nimmt, parst `{`), und eine Zeilennummer in `--- fehler` zaehlt die
Kommentarzeilen des Falls mit. **Bleibt in pytest:** die Reihenfolge von PRINT
und EPRINT in EINER Datei (ein Kind ueber PROCESS_START hat getrennte Kanaele)
und `SHELL_OUT$` mit der eigenen Exe (`SHELL` kennt `dhrt` nicht als Namen der
laufenden Laufzeit, anders als `PROCESS_START`); `test_module_alias.py`
prueft das Python-`preprocess` der Editoren, nicht dhrt. Verweise auf die
geloeschten Dateien in CLAUDE.md, `docs/` und drei Kommentaren nachgezogen --
darunter der Hinweis, die PDF-Schriftmasse wuerden in `test_pdf.py` gegen
PyMuPDF gemessen: das tut seit 2026-09-08 ein Golden mit PyMuPDFs Werten in
`tests/pruef/pdf.dhtest`.

**Stufe 43 (2026-09-13):** der Sprite-Pilot ohne Python -- und damit hat
KEIN Werkzeug in Drachenhauch mehr einen pytest-Test.
`tests/pruef/werkzeug_sprite.dhtest` (41 Faelle), die pytest-Datei
`test_pilot_sprite_auswahl.py` geloescht. Kein neues Formatstueck; die
zweizeiligen Ersetzungen der pytest-Fassung (`GUI_WINDOW_VISIBLE(winGr, FALSE)`
plus `DIM grOffen`, weil die erste Zeile dreimal vorkommt) ersetzen jetzt die
EINDEUTIGE `DIM ...Offen`-Zeile durch den Zusatz plus sie selbst. Fremde Leser
ersetzt: Pillow (GIF-Zeiten) durch die Bytes (Graphic Control Extension
`21 F9 04`, Verzoegerung in Hundertstelsekunden), `_parse_gpl` der Qt-Fassung
durch dieselben Regeln in Drachenhauch, `_write_gpl` durch eine Beilage in
genau seinem Format; ob erzeugter Code, eine Maschine oder ein Atlas laeuft,
sagt `dhrt bild` ueber seinen Rueckgabewert (`PROCESS_CODE`) -- ein Atlas-Schluessel,
den es nicht gibt, beendet ihn mit Fehler, das ist die Gegenprobe. Ein
Programm, das nur schreibt statt ausgibt (Zustand der Maschine), geht in eine
Datei, weil `dhrt bild` die Ausgabe nicht weiterreicht. Gegenprobe ohne
`AUTOMATION_PLAY`: 40 von 41 fallen, der uebrige hat keine Eingaben. **Zwei
Faelle waren schon in pytest wertlos:** "drehen tauscht breite und hoehe" lief
auf dem 32x32-Start-Sprite, wo ein Tausch nicht auffaellt -- jetzt erst ueber
"Groesse aendern" auf 48x40; und "namen kommen ueber den streifen zurueck"
merkt sich jetzt den Stand nach dem Export. Weitere Zwischenstaende (Punkt
links oben vor der Wandlung, Bereich vor [Neu]) stehen jeweils als eigene
Probe-Zeile, damit ein Endstand nicht zufaellig stimmt.

**Stufe 42 (2026-09-13):** der Tilemap-Pilot ohne Python --
`tests/pruef/werkzeug_tilemap.dhtest` (32 Faelle), die pytest-Datei
`test_pilot_tilemap.py` geloescht. **Neues Formatstueck `--- ersetzen ALT`**: der Rest der Kopfzeile
muss GENAU EINMAL im Programm stehen und wird durch den Block ersetzt
(`{sammlung}`/`{fall}` gelten; `pruefsammlung::ersetzen_einmal`, Rust-Test
`ersetzen_genau_einmal`). Die pytest-Fassung ersetzte Text in der Kopie --
das Vollbild, die Dateidialoge (keine Aufnahme erreicht einen nativen Dialog),
das Ablesen der Eingabefelder und den relativen Tileset-Pfad (die Kopie liegt
in `_programm/`). Die Platzhalter werden dafuer jetzt VOR der Programmkopie
berechnet. Die fremden Leser sind ersetzt: das Qt-Modell `TileMapDoc` durch das
json-Modul mit denselben Fragen (Punkt = Breite und Hoehe 0, Eigenschaften am
ersten Tileset, GID -> Tileset mit der groessten `firstgid <= gid`), PIL durch
`dhrt bild` + `GETPIXEL` beim erzeugten Renderer und durch `IMAGE_NEW` +
`IMAGE_SAVE` im `--- vorher` fuer das grosse Tileset. Die zwei pytest-Tests mit
je zwei Laeufen (Renderer leer/gemalt, solid setzen/entfernen) sind je zwei
Faelle, die Startzustaende von Ebene und Tileset eigene Gegenproben.
**Stolperstein:** eine MAP laesst sich nicht per Index beschreiben
(`m[k] = 1` ist "Index-Zuweisung an Nicht-Array") -- `MAPPUT`. Gegenprobe ohne
`AUTOMATION_PLAY`: 30 von 32 fallen, die zwei ohne Eingaben nicht. Im ersten
Anlauf waren es 29 -- "die pipette schaltet das tileset um" pruefte (schon in
pytest) nur den Endstand `tsAkt 0`, der ohne jede Eingabe auch stimmt; er
merkt sich jetzt, dass das zweite Tileset zwischendurch aktiv war.

**Stufe 41 (2026-09-13):** der Rechnungen-Pilot ohne Python --
`tests/pruef/werkzeug_rechnungen.dhtest` (14 Faelle; die zwei Pruefungen, die
in pytest je EIN Test mit zwei Laeufen waren, sind jetzt je zwei Faelle),
`tests/test_pilot_rechnungen.py` geloescht. Die vier fremden Leser sind
ersetzt: Pythons `sqlite3` durch das db-Modul im `--- nachher`, PyMuPDF durch
`BUFFER_INFLATE` ueber die Inhaltsstroeme, das `csv`-Modul durch `READLINES`
und die Python-Eltern des Rechnungsfensters durch das net-Modul
(`NET_TCP_LISTEN` + `SETENV("DHRT_ELTERN_PORT")` + `PROCESS_START`) -- ein
Teilnehmer, der `WINDOW_OPEN` nicht kennt, wie vorher. Der Pfad des Kindes
kommt ueber `--- argumente {sammlung}/...` in den Hauptlauf, der ihn in eine
Datei schreibt: ein `--- nachher` kennt die Sammlung nicht. Einen Befehl, der
den vom System gewaehlten Port nennt, gibt es nicht -- der Fall lauscht auf
einem festen (48123), die Sammlung laeuft seriell. Einschuebe: ein
1x1-Nullpunkt hinter `GUI_WINDOW_CHROME` (Fenster- in Bildschirmlagen), der
Startzustand hinter `GUI_FOCUS(tfKName)`, Aufnahme und Probe hinter `FLIP()`.
Alle 14 Faelle waren beim ersten Lauf gruen -- die Gegenprobe ohne
`AUTOMATION_PLAY` laesst alle 10 mit Eingaben fallen; die uebrigen 4 (erster
Start, Geldrechnung, Fenstergroesse, Rechnungsfenster) brauchen keine.

**Stufe 40 (2026-09-13):** der Tracker-Pilot ohne Python --
`tests/pruef/werkzeug_tracker.dhtest` (17 Faelle), `tests/test_pilot_tracker.py`
geloescht. Ohne neues Formatstueck: ein Einschub hinter `FLIP()` schreibt je
Bild `probe.txt`, ein zweiter hinter `bildNr = bildNr + 1` ruft die
Unterprogramme des Piloten (`zelleSetzen`, `ladeDatei`, `wavRendern`,
`gbCodeSichern`). Die drei fremden Leser sind ersetzt: die Datei liest das
json-Modul mit genau den Schluesseln, die `drachenhauch.tracker.Song` liest
(`patterns.N.data.KANAL.REIHE`, leer = `null`); die zwei Dateien der
Qt-Fassung sind Beilagen, einmal mit `Song.save_json` geschrieben und auf eine
Zeile gepackt; die Mono-WAV prueft `--- ton`, die Stereo-WAV ein
`--- nachher` ueber ihre Bytes (RIFF, `BUFFER_GET_I16`), weil `--- ton` keinen
Pegel je Kanal kennt; der GB-Code geht durch `PROCESS_START("dhrt",
"--check", ...)` und einen Start. Nur der Klick-Fall braucht die Geometrie des
Fensters und schreibt seine Aufnahme darum im dritten Bild selbst.
Gegenprobe ohne `AUTOMATION_PLAY`: alle 9 Faelle mit Eingaben fallen, die
uebrigen 8 rufen den Piloten direkt und brauchen keine.

**Stufe 39 (2026-09-13):** SFX- und Partikel-Pilot ohne Python --
`tests/pruef/werkzeug_sfx.dhtest` und `werkzeug_partikel.dhtest` (je 8), dazu
`tests/pruef/builtin_registrierung.dhtest` fuer die 14 kleinen pytest-Dateien,
die nur fragten, ob Fenster-/Monitor-/Dialogbefehle im Verzeichnis stehen
(68 Namen ueber `CODE_COMPLETE`, mit Gegenprobe). 16 pytest-Dateien geloescht.
**Neues Formatstueck `--- einschub nach MARKE`** (mehrere erlaubt): die
Piloten brauchen ZWEI Stellen -- oben Helfer, in der Bildschleife hinter
`FLIP()` die Messung. Die pytest-Fassung las die Klicklagen aus einem ersten
Lauf; hier schreibt der Einschub im dritten Bild, wenn die Geometrie
(`GUI_CANVAS_X - GUI_GET_X`) feststeht, die Aufnahme selbst und spielt sie ab,
und `prProbe(k)` schreibt je Bild nach `_programm/probe.txt` (geprueft mit
`--- inhalt`). **Falle:** die Wiedergabe setzt beim ERSTEN Ereignis an, nicht
bei Bild 0 -- ohne ein Ereignis in Bild 0 lief alles 6 Bilder frueher als die
Proben, und die Faelle mit Klicks fielen scheinbar grundlos (die Zuege
dagegen nicht, weil ihre Probe spaet genug lag). Gegenprobe ohne
`AUTOMATION_PLAY`: in beiden Sammlungen fallen 6 von 8, die uebrigen zwei
haben keine Eingaben. Beim ersten Versuch waren es beim Partikel-Piloten nur
5 -- "zurueck nimmt den ganzen zug zurueck" pruefte (wie schon in pytest) nur
den Endstand, und "wie vorher" stimmt auch, wenn nie gezogen wurde; er misst
jetzt auch den Stand nach dem Zug. Rust-Test `weitere_einschuebe`.

**Stufe 38 (2026-09-13):** der letzte Fall -- `tests/test_ide.py` ist
geloescht. Das PDF-Listing hing an PyMuPDF, weil das pdf-Modul jede Seite mit
FlateDecode packt und Drachenhauch nur `COMPRESS$`/`DECOMPRESS$` hatte (Text
rein, Base64 raus, rohes Deflate ohne zlib-Kopf). Neu, allgemein statt fuer
den einen Test: **`BUFFER_DEFLATE(b [, format$])` / `BUFFER_INFLATE(b [,
format$])`** auf rohen Bytes, Vorgabe `"zlib"` (PDF, PNG, Netz), `"roh"` wie in
ZIP; kaputte Daten und ein unbekanntes Format sind Fehler, entpackt wird
hoechstens 512 MB (`decompress_to_vec_zlib_with_limit`). Der Fall in
`tests/pruef/werkzeug_ide_sonderfaelle.dhtest` schreibt die 140 Zeilen im
`--- vorher`, zaehlt die Seitenobjekte (`/Type /Page /Parent` -- `/Type /Page`
allein traefe auch `/Pages`) und entpackt die Inhaltsstroeme. **Falle:** die
Suche nach `stream\n` muss HINTER dem letzten `endstream` weitergehen, sonst
trifft sie `endstream\nendobj`. Tests `tests/pruef/buffer.dhtest` (+4, darunter
ein von Pythons `zlib` gepackter Strom als fremder Schreiber).

**Stufe 37 (2026-09-13):** die Sonderfaelle der IDE ohne Python -- und dafuer
drei neue Formatstuecke. Was in `tests/test_ide.py` blieb, brauchte mehr als
EINEN Lauf: zwei IDE-Laeufe hintereinander (Sitzung, Umbruch, Sitzung je
Projektordner), ein git-Repository VOR dem Lauf oder eine Messung im
Bildschirmfoto. **`--- vorher`** laesst vor dem Hauptlauf ein Hilfsprogramm in
Drachenhauch im Fallordner laufen (das Repository: `PROCESS_START("git", ...)`
init/config/add/commit, danach `WRITEALL` fuer die Aenderung), **`--- zwischen`**
eins nach dem Hauptlauf (aus dem Foto des ersten Laufs die Mitte des Farbfelds
messen und die Aufnahme fuer den zweiten schreiben, oder eine Aufnahme mit
`DELETEFILE` wegnehmen), **`--- nochmal [in ORDNER]`** startet das Programm
erneut mit der Umgebung des Falls (Blockzeilen = Argumente, ohne `in` im
Fallordner). `--- zwischen` und `--- nochmal` laufen in der Reihenfolge der
Datei (`Fall::schritte`), ohne `--- nachher` gilt die Erwartung dem letzten
Lauf. Im Laeufer geht jeder Programmlauf durch EINEN Aufbau (`befehl`), sonst
liefe ein zweiter mit anderer Bildzahl; Hilfsprogramme durch `hilfslauf`. Der
Einschub spielt die Aufnahme nur ab, wenn es sie gibt
(`IF FILEEXISTS("ev.txt") THEN AUTOMATION_PLAY("ev.txt")`). Die Hervorhebung
zaehlt ihre Farbbaender jetzt in Drachenhauch (`LOADIMAGE` + `GETPIXEL` gehen
ohne Fenster). Neue Sammlung `tests/pruef/werkzeug_ide_sonderfaelle.dhtest` (8);
in `tests/test_ide.py` bleibt **nur das PDF-Listing** -- das pdf-Modul packt
seine Seiten mit FlateDecode, ein Leser in Drachenhauch saehe den Text nicht.
**Zwei Gegenproben:** ohne `AUTOMATION_PLAY` fallen alle 6 Faelle mit Aufnahme;
ohne die `--- nochmal`-Bloecke fallen auch die beiden Sitzungsfaelle, die gar
keine Aufnahme haben. **Der Fund dieser Stufe kam aus der CI von Stufe 36:**
auf macOS lieferte "ide uebersetzt ohne befund" `check  0` statt
`check [] 0`. Die Leseschleife auf `PROCESS_READ$` brach ab, sobald EINE
Abfrage leer war und der Prozess nicht mehr lief -- das Kind kann aber schon
beendet sein, bevor sein Lesefaden die letzte Zeile abgelegt hat. Alle 71
solchen Schleifen in sechs Sammlungen lesen jetzt bis zu fuenf leeren Abfragen
im Abstand von 30 ms. **Stolperstein:** `EPRINT "text"` ohne Klammern ist ein
Parse-Fehler, `EPRINT("text")` geht. Rust-Test
`vorher_zwischen_nochmal_in_ihrer_reihenfolge`.

**Stufe 36 (2026-09-13):** die Staende 13 bis 25 der IDE ohne Python -- 58
Faelle nach `tests/pruef/werkzeug_ide_13_25.dhtest`; in `tests/test_ide.py`
bleiben **9 Sonderfaelle** (PDF-Listing mit PyMuPDF, Sitzung/Umbruch/Sitzung
je Projekt mit zwei IDE-Laeufen, git blame und zweimal git diff mit
Repository, die Hervorhebung als Farbbaender im Bild, das Farbfeld mit einem
Messlauf vorab). Hinzu kamen keine Formatstuecke, nur Lagen: das
Parameter-Fenster (Knoepfe ab 420/230), das Vorschau-Fenster (Kaestchen der
Dateien, Zeilen des Unterschieds ab y = 160 je 22, [Block auslassen] und
[Pruefen] bei y = 681), die mittlere Maustaste auf einem Reiter, Beilagen in
Unterordnern (eine Datei unter lib/ im Fallordner) und eine eigene
Doku-Wurzel fuers Handbuch.
Verschobene und umgebaute Dateien gehen im `nachher`-Programm durch
`dhrt --check`. **Stolperstein beim Schreiben:** ein erwarteter Dateianfang
mit Zeilenumbruch stand als Zeichenkette im Leseprogramm -- ein Umbruch in
einer Zeichenkette ist ein Lexer-Fehler; der Text kommt jetzt ueber
`CHR$(10)` (wie `"` ueber `CHR$(34)`). Gegenprobe ohne `AUTOMATION_PLAY`:
56 der 58 Faelle fallen; die zwei uebrigen ("ein geaenderter Reiter fragt
vor dem Schliessen", "ein Klick auf einen Ordner klappt ihn um") pruefen,
dass NICHTS passiert -- in pytest genauso schwach. **Nebenfund der vollen
Pruefung:** `dhrt test tests/pruef` braucht mit den IDE-Sammlungen 780 s
(sie laufen in Echtzeit und nacheinander), der pytest-Anker
gab nur 600 s und riss im seriellen Durchgang --
er bekam 1800 s. (Seit 2026-09-20 ruft die CI `dhrt test tests/pruef` direkt
auf, ohne Zeitgrenze von pytest.)

**Stufe 35 (2026-09-13):** die Staende 6 bis 12 der IDE ohne Python -- 43
Faelle aus `tests/test_ide.py` nach `tests/pruef/werkzeug_ide_6_12.dhtest`
(eigene Datei, `--- seriell`; die Laeufer nehmen seriell markierte Dateien
nacheinander, die Zwischenablage bleibt also ungeteilt), 67 bleiben. Neu im
Umgang, nicht im Format: eine **Voreinstellung der IDE** (`regex`,
`autosichern`, `geruest`) kommt als Beilage `ide.json` in `_programm/`, weil
`DH_IDE_KONFIG` relativ zu ihr zeigt; **Mausklicks** auf Projektbaum und
Werkzeugleiste treffen dieselben festen Lagen wie in pytest (die Kopie laeuft
dort wie hier MIT `WINDOW_MAXIMIZE`); das **Vorschaubild einer Kachel** prueft
`--- bild _programm/vorschau/09_shapes.png` statt Pillow; der
**herausgeloeste Code** geht im `nachher`-Programm durch
`PROCESS_START("dhrt", "--check", ...)`. **Stolperstein:** mit relativem
`DH_IDE_KONFIG=ide.json` lag das Vorschaubild nicht neben der Sitzung --
die IDE leitet den Vorschau-Ordner aus dem Konfig-Ordner ab und reicht den
relativen Pfad an `dhrt bild` weiter, und das wechselt ins Verzeichnis des
BEISPIELS; das Bild landete unter `wurzel/examples/vorschau/`. pytest gab
einen absoluten Pfad und sah es nie. Der Fall setzt `{fall}/_programm/ide.json`.
In pytest geblieben: die zwei git-diff-Faelle (Repository), die Hervorhebung
(zaehlt Farbbaender im Bild) und das Farbfeld (erster Lauf misst die Lage).
Gegenprobe ohne `AUTOMATION_PLAY`: 40 der 42 Faelle mit Aufnahme fallen; die
zwei uebrigen ("beim Sammeln geht noch nichts auf", "ohne die Einstellung
wird nicht gesichert") erwarten, dass nichts passiert -- in pytest genauso
schwach.

**Stufe 34 (2026-09-13):** die IDE selbst, erster grosser Teil ohne
Python -- ohne neues Formatstueck, alles mit dem aus Stufe 32/33.
test_ide_bausteine.py ist geloescht (12 Faelle in
`tests/pruef/ide_bausteine.dhtest`: PROCESS_*, CODE_* als Golden der
JSON-Ausgabe, Textbereich-Befehle), aus `tests/test_ide.py` sind die 31 Faelle
der Staende 1 bis 5 nach `tests/pruef/werkzeug_ide.dhtest` gezogen (110
bleiben). **Das Protokoll liest ein `--- nachher`-Programm**
(`READLINES("_programm/ide.log")` + `hat`/`zahl`/`erste$`/`alle$`/`endet`),
nicht `--- inhalt` -- das sucht Teiltexte, und `geprueft 1` stuende auch in
`geprueft 10`; Reihenfolgen ("erst Halt in 2, dann 3") und Zaehlungen gehen
nur so. Gesicherte Dateien kommen als `|Zeile|` in die Erwartung (ein
Leerzeichen am Zeilenende bleibt sichtbar). In pytest blieb, was ZWEI
IDE-Laeufe hintereinander braucht (Sitzung, Umbruch gemerkt -- `nachher`
erbt die Umgebung des Falls nicht, und `PROCESS_START` nimmt dem Kind
`DHRT_FRAMES`), ein git-Repository oder PyMuPDF. **Stolperstein:** derselbe
Fallordner kam als `DHRT_START_DIR` mit 8.3-Kurznamen (`HANSGA~1`) und aus
`CWD$()` mit langem Namen an -- verglichen werden die letzten zwei
Pfadteile. Gegenprobe ohne `AUTOMATION_PLAY`: 26 der 28 Faelle mit Aufnahme
fallen; die zwei uebrigen ("geteilte Ansicht braucht zwei Dateien",
"krummer Name") erwarten, dass NICHTS passiert, und waren schon in pytest so
schwach.

**Stufe 33 (2026-09-13):** die naechsten zwei Werkzeuge ohne Python --
`test_pilot_animfsm.py` und `test_pilot_notenblatt.py` sind geloescht, ihre
Faelle stehen in `tests/pruef/werkzeug_animfsm.dhtest` und
`tests/pruef/werkzeug_notenblatt.dhtest` (je 7). Dafuer drei Stuecke im
Format: **`--- streichen ZEILE`** nimmt die erste genau so lautende Zeile aus
der Programmkopie (`WINDOW_MAXIMIZE()` -- sonst treffen die Klicks keine
festen Lagen; fehlt die Zeile, ist der Fall ein Fehler), **`--- nachher`**
laesst nach dem Lauf ein zweites Programm im Fallordner laufen, dessen
Ausgabe statt der des Hauptlaufs verglichen wird (erst wird der Hauptlauf mit
leerer Erwartung bewertet, damit ein fehlendes Fenster weiter "uebersprungen"
heisst), und **Platzhalter in Text-Beilagen**, mit Schraegstrichen -- ein
Rueckstrich im Pfad waere in einer JSON-Beilage ein Escape. Die Leser der
Qt-Modelle (`AnimDoc`, `ScoreDoc`, `Song`) ersetzt Drachenhauch selbst:
`JSON_*` plus `ANIM_FSM_LOAD` und ein Schritt der Maschine; `to_tracker_song`
ist ein fester, einmal aufgezeichneter Stand des Gitters. Die Gegenprobe
(ohne `AUTOMATION_PLAY`) laesst alle 14 Faelle fallen. **Zwei
Stolpersteine:** `PROCESS_READ$` liefert die Zeile MIT Umbruch (ohne
`TRIM$` steht eine Leerzeile in der Ausgabe), und `PRINT` einer
zusammengesetzten Namensliste traegt das letzte Leerzeichen mit.
Rust-Test `streichen_und_nachher`.

**Stufe 32 (2026-09-13):** Werkzeuge ohne Python pruefen. Die Tests der
Drachenhauch-Werkzeuge (IDE, Form-Designer, Anim-FSM, Notenblatt) hingen an
pytest, weil nur Python ein VORHANDENES Programm mit eingeschobener Aufnahme
starten und danach seine Dateien lesen konnte. `.dhtest` kann das jetzt:
**`--- programm pfad [nach MARKE]`** (Pfad relativ zur Sammlung; der
Quelltext des Falls wird hinter der ersten Zeile MARKE eingeschoben,
`pruefsammlung::einschieben`, eine fehlende Marke ist ein Fehler), **`---
inhalt datei`** / **`--- ohne datei`** (Zeilen in einer Datei nach dem Lauf)
und die Platzhalter **`{sammlung}`/`{fall}`** in Umgebung und Argumenten. Der
Laeufer kopiert das Programm nach `_programm/` im Fallordner (ein `_`-Ordner,
den die Werkzeuge beim Durchsuchen uebergehen) und startet das Kind JETZT MIT
dem Fallordner als Arbeitsordner -- er ist damit `DHRT_START_DIR`, also das
"Projekt" eines Werkzeugs. Zwei Wege je Werkzeug: echte Klicks (Einschub
`AUTOMATION_PLAY` hinter `SETFPS(60)`) oder direkt ueber seine
Unterprogramme (Einschub hinter der Bereit-Zeile, endet mit `EXIT(0)`) --
der zweite prueft die Logik ohne Bildschirmlagen. **Als Beleg umgezogen:**
test_pilot_formdesigner.py -> `tests/pruef/werkzeug_formdesigner.dhtest`
(10 Faelle; der GB-Code-Fall ruft `gbCode$()`, laesst `dhrt --check` und
`dhrt bild` als Prozesse laufen; der Palette-Fall liest gui.rs mit
`READLINES`). Stolperstein beim Schreiben: `READALL$` nimmt ein FILE-Handle,
keinen Pfad -- fuer Pfade `READLINES`. Rust-Test
`programm_einschub_und_dateiproben`.

**Stufe 31 (2026-09-13):** der Hinweis des Nutzers -- er hatte in Stufe 30
eine Aenderung am Qt-Designer gesehen; **Python soll komplett wegfallen**.
Stufe 30 hatte das Kaestchen "Zellmodus" noch in `formdesigner_qt.py`
eingebaut; das ist die letzte Aenderung dort. Die neuen Faehigkeiten landen
jetzt im Designer in Drachenhauch (`examples/197_form_designer.dh`):
**Palette** mit allen 30 Arten der Laufzeit (vorher 25 -- dieselbe Drift wie
beim Qt-Designer 2026-08-31, jetzt mit Test gegen `Kind::from_str`) und dem
**Gitter** (Palettenart `grid`, geschrieben als `table` mit `zellmodus` und
`col_edit`); **Felder je Art** im Inspektor (`artFeld`/`artSichtbar`/
`artFuellen`/`artUebernehmen`: Eintraege, Spalten, Breiten, Bearbeitbar,
Spaltenarten, Auswahl, Zellmodus, Min/Max/Wert -- Listen mit Semikolon; eine
Spalte mit Auswahlliste wird zur Auswahlspalte, sonst wirkte die Liste beim
Laden nicht); **GB-Code** (Strg+G, `gbCode$`/`gbControl`): jedes Control
als Aufruf, ohne `GUI_LOAD`, Texte mit Umbruch als `!"..."`, sich selbst
messende Konstruktoren mit `GUI_SET_BOUNDS` hinterher, das Bild
uebersprungen. Die IDE: **eine `.dhform` oeffnet den Designer** (`werkzeugMit`,
`dhrt run 197 -- datei`), als Text ueber die Befehlspalette ("Formular als
Text oeffnen"). Tests: vier neue in test_pilot_formdesigner.py (seit Stufe 32 in `tests/pruef/werkzeug_formdesigner.dhtest`) --
lesen mit `json`, nicht mit dem Qt-Modell; der GB-Code-Test legt JEDE Art an
und laesst den Code durch `--check` und einen Lauf; zwei in `tests/test_ide.py`.

**Stufe 30 (2026-09-13):** die Frage des Nutzers -- gibt es ein
Gitter-Steuerelement? Halb: `GUI_TABLE` konnte sortieren, filtern und Zellen
per Doppelklick bearbeiten, aber es gab keine aktuelle Zelle, keinen Bereich,
keine Zwischenablage, keine Zahlen- oder Auswahlspalten -- und **eine Tabelle
mit Fokus nahm ueberhaupt keine Taste an** (`table_keys` kannte nur Editor und
Filter; die Pfeile bewegten nicht einmal die Zeilenauswahl). Jetzt ist das
Gitter ein **Zellmodus derselben Tabelle** (`TableState::zellmodus`,
`GUI_TABLE_SET "zellmodus"`, `GUI_GRID` legt eine an), damit Sortieren,
Filter, feste Spalten und Zellarten nicht doppelt entstehen. Aktuelle Zelle
und Anker sind DATENzeile/-spalte (`cur_r/cur_c`, `ber_r/ber_c`), der
Bereich ist das Rechteck in der SICHTBAREN Reihenfolge (`bereich()`), ueber
Datenzeilen waere er nach dem Sortieren ein Flickenteppich. Tastatur
`table_nav_keys` (Pfeile/Bild/Pos1/Ende, Umschalt = Bereich, Tab mit
Zeilenumbruch, Tippen ERSETZT, Enter/F2 bearbeitet, Entf leert, Leertaste
kippt Haken, Strg+A/C/X/V); in der Bearbeitung ruecken Enter und Tab weiter
(`table_schritt`, unter der letzten Zeile mit `zeilen_anhaengen` eine neue).
**Tab gehoert dem Gitter** (`tab_belegt`), hinaus mit Strg+Tab. Kopiert wird
Tabulator-Text; eingefuegt nach den Regeln der Spalte (`zelle_schreiben`:
gesperrt oder unpassend = uebergangen, nicht halb geschrieben;
`GUI_TABLE_PASTE` zaehlt). `GUI_TABLE_COL_TYPE` (text/ganz/zahl/auswahl --
Zahlen filtern beim Tippen mit `zahl_erlaubt` wie das Textfeld) und
`GUI_TABLE_COL_CHOICES` (Auswahlliste in der oberen Schicht, `wahl_geom` =
eine Quelle fuer Zeichnen und Klick). `GUI_TABLE_SET_CURRENT` rollt ueber
`sicht_holen` + `tabellen_pass`, weil ein Setter keine Geometrie hat.
`.dhform` `zellmodus`/`zeilen_anhaengen`/`col_type`/`col_choices`; Designer
Kaestchen "Zellmodus" + Codegen. Die IDE: das Profil ist ein Gitter
(Strg+C kopiert Zeilen). **Stolperstein, vom Gitter aufgedeckt:**
`IF GUI_TABLE_CLICKED(tblProfil) THEN` -- das Builtin liefert -1 fuer
"keins", und -1 ist WAHR; `profilAnspringen` lief seit Stufe 2 in jedem Bild
und nagelte nach dem ersten Klick ins Profil die Marke fest. Sichtbar wurde
es erst, als Pfeile im Gitter eine Zeile waehlten und ein fremder Test
(Aufrufer-Baum) rot wurde; `--check` sagt dazu nichts. Die uebrigen
-1-Builtins in IDE und Beispielen vergleichen alle mit `>= 0`. Tests `tests/pruef/gui_gitter.dhtest` (6, darunter
die Pfeiltasten in einer gewoehnlichen Tabelle), ein Rust-Test.

**Stufe 29 (2026-09-13):** das Handbuch bedienbar, als Ausbau von
`GUI_RICHTEXT`. **Auswahl** (`RichState` anker/marke/hat_auswahl): eine
Stelle ist (Zeile, Zeichen im ZEILENTEXT) -- `rt_zeilentext` fuegt die Laeufe
einer Zeile zusammen, mit Leerzeichen, wo eine Luecke zwischen ihnen liegt
(`**fett**bar` bleibt ein Wort). Damit findet die Suche ueber Wortgrenzen
und Kopiertes ist lesbar; Leerzeilen des Satzes kommen beim Kopieren nicht
mit. **Der Druck misst nichts** -- `handle_press` hat keine Grafik; er merkt
Druckpunkt und Doppelklick, `rt_auswahl_pass` (in `update` direkt nach dem
Druck) misst Anker und Marke und zieht nach, solange die Taste haengt
(Doppelklick = Wort um die Stelle). Strg+A/Strg+C im `widget_keys`-Zweig,
`GUI_RICHTEXT_SELECTION$/SELECT_ALL/CLEAR_SELECTION`. **Suche:**
`GUI_RICHTEXT_FIND_NEXT/PREV` (ab ANFANG der Auswahl + 1 bzw. vor ihr, mit
Umlauf; markiert und rollt ins Bild), `GUI_RICHTEXT_MARK_ALL` (schwacher
Hauch unter jeder Fundstelle, liefert die Zahl). Neu setzen (Text, Breite)
hebt die Auswahl auf. Gezeichnet wird je Lauf der Teil im Bereich, an seinen
Zeichen gemessen, dazu die Luecke zwischen zwei Laeufen -- sonst saehe eine
Auswahl ueber drei Woerter aus wie drei. Die IDE: Weiter/Zurueck-Knoepfe,
Enter/Umschalt+Enter, Zahl der Fundstellen, Protokoll `hbsuche`. Tests
`tests/pruef/gui_gesetzter_text_auswahl.dhtest` (4, der Mauszug
geometriefrei: von vor der ersten bis hinter die letzte Zeile = alles, gegen
SELECT_ALL), einer in `tests/test_ide.py`.

**Stufe 28 (2026-09-13):** drei Punkte aus der eigenen Lueckenliste der
gui, alle drei Leisten. **`GUI_STATUSBAR`** (gui.rs `StatusState`, Kind
`statusbar`): Felder mit fester Breite oder Anteil am Rest (`sb_geom`),
Ausrichtung, Tooltip, klickbar (`GUI_STATUSBAR_CLICKED`); **`GUI_SET_TEXT`
meint Feld 0**, damit eine Beschriftung als Statuszeile ohne Umschreiben
ersetzbar ist. Der Anlass steckte in der IDE: jeder Pfeil schrieb "Zeile,
Spalte" in dieselbe Beschriftung und loeschte damit die letzte Meldung.
**`GUI_BREADCRUMB`** (`PfadState`, Kind `breadcrumb`): Teile mit
unsichtbarem Wert, Winkel dazwischen, der letzte kraeftiger; passt nicht
alles, fallen die VORDEREN Teile unter ein "..." (`pf_geom`, vom letzten
Teil rueckwaerts), dessen Tooltip den ganzen Pfad nennt. Breiten misst
`leisten_pass` in GUI_UPDATE, der Klick liest dieselbe Zahl.
`GUI_BREADCRUMB_SET` mit demselben Pfad aendert nichts -- ein Programm setzt
ihn gern je Bild, und die Messung ginge sonst jedes Mal verloren.
**Ueberlauf der Werkzeugleiste** (`tb_layout` liefert jetzt sichtbare,
versteckte Eintraege und den >>-Knopf): was nicht passt, steht in einem
Menue (`leiste_popup`, gezeichnet in `draw_top`, Klick zuerst in
`handle_press`); Luecken fallen weg, ein Trenner an der Bruchstelle
verschwindet. Die IDE: Statusleiste mit Meldung | Zeile/Spalte (Klick = Gehe
zu Zeile) | Umbruch (Klick = umschalten); die Symbolspur ist eine Pfadleiste
(Wert = Zeile des Blocks, Klick springt, `pfad sprung N`) -- die
Protokollzeile `spur ...` blieb, zwei Tests lesen ihr letztes Element.
**Stolperstein beim Schreiben der Tests:** `--- nicht enthaelt` gibt es im
dhtest-Format nicht; eine Gegenprobe gehoert in `--- erwartet` mit
Ausgaben ohne Bildnummer. Tests `tests/pruef/gui_leisten.dhtest` (7),
ein Rust-Test, einer in `tests/test_ide.py`; Form-Designer-Palette 30.

**Stufe 27 (2026-09-13):** zwei Fragen des Nutzers -- koennen Knoepfe
mehrere Farben haben, und hat die Liste Funktionen, die sie komfortabel
machen? Beides war nur halb da. **Knoepfe:** `GUI_SET_COLOR` gab es, aber
nur EINE Grundfarbe, Ueberfahren/Druecken pauschal +-30, und die Schrift
blieb weiss -- ein gelber Knopf war unlesbar. Neu `GUI_BUTTON_VARIANT` /
`GUI_BUTTON_GET_VARIANT$` (`KNOPF_ARTEN`: standard, primaer = Akzent,
erfolg, warnung, gefahr, umriss, flach, link), Rollen `hover`/`pressed` fuer
`GUI_SET_COLOR` und `GUI_STYLE_SET`, und **die Schrift folgt dem Grund**
(`lesbar_auf`, gewichtet wie das Auge), solange `fg` nicht gesetzt ist;
eine eigene Farbe gewinnt vor der Art. **Listen** (`ListState`, alles je
Eintrag ueber `sync`/`einfuegen`/`entfernen`/`tragen`/`umordnen`
gleich lang): unsichtbarer Wert (`SET_DATA/DATA$/FIND_DATA`), Zusatztext
rechts (`DETAIL`), Tooltip je Eintrag (`TIP`, `hover_teil` wie bei der
Leiste), gesperrte Eintraege (`ENABLE`) und Gruppenkoepfe (`HEADER`) --
beide weder anklickbar noch mit Pfeilen erreichbar --, Filter
(`FILTER/GET_FILTER$/VIEW_COUNT/VIEW_ROW`, Kopf bleibt, solange seine
Gruppe etwas zeigt), Leer-Hinweis (`PLACEHOLDER`), natuerliches Sortieren
INNERHALB der Gruppen (`SORT`, `natuerlich_vergleichen`), `FIND`,
`SCROLL_TO`, Bild auf/ab, **Enter meldet GUI_DOUBLE_CLICKED**, und
**Tippen springt** (`liste_tippen`: Zeichen aus der Tipp-Warteschlange,
ersatzweise die Buchstabentasten -- die Wiedergabe einer Aufnahme fuellt nur
Tasten). **Eine Quelle `liste_ansicht`** fuer Zeichnen, Klick, Tastatur,
Rad, Tooltip und Bildschirmleser -- wie `view` bei der Tabelle; nach aussen
bleiben alle Nummern Eintragsnummern. **Stolperstein, gleich beseitigt:**
im hellen Thema war die GEWAEHLTE Zeile unlesbar (dunkle Schrift auf
dunkelblauer Auswahl) -- gesehen nur im Bild; die Schrift einer gewaehlten
Zeile folgt jetzt ihrem Grund. Die IDE nutzt beides: Palette mit
Zusatztext (Kuerzel/Ordner rechts statt angehaengt), Uebernehmen/Drucken/
Neue Datei als `primaer`, Debugger-Stopp als `gefahr`. Tests
`tests/pruef/gui_liste_komfort.dhtest` (9), `tests/pruef/gui_knopfarten.dhtest`
(3, darunter eine Bildprobe).

**Stufe 26 (2026-09-13):** der Hinweis des Nutzers -- die Werkzeugleiste
der IDE sah nicht schoen aus und war keine Laufzeit-gui. Beides stimmte:
`GUI_TOOLBAR` war ein dekorativer Streifen, die IDE legte zwoelf
`GUI_ICON_BUTTON` darauf, schob sie in `layout()` von Hand, nahm ein
Text-`|` als Trenner und 16-Punkt-Bilder aus `IMAGE_DRAW_RECT`, die das
Zeichnen krumm auf 20 Punkte streckte -- jede in einer anderen Farbe.
**Jetzt hat die Leiste Eintraege** (`GUI_TOOLBAR_ADD/SEPARATOR/SPACER/
CLICKED/ENABLE/CHECKABLE/SET_CHECKED/SET_ICON/SET_TIP/SET_TEXT/SET/
ITEM_X/ITEM_W/COUNT/CLEAR`, gui.rs `LeisteState`): sie verteilt sie selbst
(`tb_geom` = EINE Quelle fuer Zeichnen, Treffertest, Tooltip und a11y),
ein Klick zaehlt beim Loslassen auf DEMSELBEN Knopf, Trenner/Luecken/
gesperrte nehmen keinen an, jeder Eintrag hat seinen Tooltip (`hover_teil`
startet den Verweil neu), kippbare Knoepfe tragen einen Hauch Akzent.
**Eingebaute Sinnbilder** (`SINNBILDER`, 35 Namen, `fn sinnbild`): Striche
auf einem 16er-Raster in der ECHTEN Groesse, eine Strichstaerke fuer alle,
Textfarbe des Themas, eigene Farbe nur fuer start/stopp/pruefen/haltepunkt;
ein unbekannter Name ist ein Fehler mit der Liste. Dieselben Namen nehmen
`GUI_ICON_BUTTON`, `GUI_SET_ICON` und `GUI_MENU_ICON` -- die IDE hat
`symbolBild` samt `SYM_*`-Farben geloescht, Menue und Leiste sehen gleich
aus. **Ohne Eintraege bleibt die Leiste Deko und Luft fuer Klicks**, sonst
traefen die Knoepfe alter Programme (136, 156, 187, 189) nicht mehr.
**Zwei Funde, beide erst im Lauf:** (1) Beschriftungen an der Zeichenzahl
zu schaetzen (wie beim Reiterwerk) liess "Sichern" in den naechsten Knopf
laufen -- gesehen nur im BILD; jetzt misst `leisten_pass` in GUI_UPDATE
(`text_b`), der Treffertest liest dieselbe Zahl. (2) **Eine
`--- datei`-Beilage ohne Leerzeile am Ende wird ohne Zeilenumbruch
geschrieben, und raylibs Aufnahme-Leser verliert die LETZTE Zeile** --
meist das Loslassen. Zwei Faelle ("Trenner", "Wegziehen") waren dadurch
gruen, ohne je geklickt zu haben, weil dort ohnehin nichts passieren
sollte; `docs/werkzeuge.md` sagt es jetzt. Tests
`tests/pruef/gui_werkzeugleiste.dhtest` (9, darunter eine Bildprobe),
2 Rust-Tests, 2 in `tests/test_ide.py`.

**Stufe 25 (2026-09-12):** zwei Widgets aus der eigenen Lueckenliste -- und
damit ist sie um zwei Punkte KUERZER geworden, nicht laenger.
**`GUI_RICHTEXT`** setzt Markdown, statt es anzuzeigen (Ueberschriften,
Absaetze mit Umbruch, Aufzaehlungen verschachtelt und nummeriert,
Codebloecke, Tabellen mit echten Spalten, Zitate, Linien, Verweise; was er
nicht kennt, steht als Text da). **Gesetzt wird in GUI_UPDATE** (`rt_pass`
-> `rt_setzen`), nicht beim Zeichnen: nur dort kommen Graphics zum Messen
und Schreibzugriff zusammen, und so kostet ein Dokument mit 2000 Zeilen
seinen Satz EINMAL; neu gesetzt wird nur bei geaenderter Quelle, Breite,
Schriftgroesse oder Massstab (`stand`). **Aufeinander folgende Zeilen sind
EIN Absatz** (Markdown-Regel) -- die Dokumente in `docs/` sind auf 76
Spalten umbrochen, Zeile fuer Zeile gesetzt ergaeben sie einen
ausgefransten Block. Fett = zweiter Zug um einen Punkt versetzt (aus einer
Schrift laesst sich keine Strichstaerke rechnen), kursiv = gedaempft.
**Zwei Fehler zeigte erst das gerenderte BILD:** das Leerzeichen VOR einer
Auszeichnung ging verloren (`Ein **fetter** Anfang` -> `Einfetter Anfang`:
es steht am Ende des vorigen Stuecks und fiel beim Zerlegen weg), und eine
gestauchte Tabellenspalte lief in ihre Nachbarin (unter das breiteste WORT
darf keine Spalte gestaucht werden). Dazu **`GUI_TIMEPICKER`**: nach aussen
EIN Format `HH:MM:SS` wie `TIME$()`, auch ohne Sekundenfeld; an der Grenze
laeuft es um; die Minute hat eine Schrittweite. **Die IDE setzt ihr
Handbuch damit** -- 180 Zeilen Handarbeit weniger, und Verweise sind jetzt
anklickbar (Dokument oder Sprungmarke). Dazu ihre drei offenen Punkte:
**Haken im Baum schraenken das Projekt ein** (eine Stelle,
`projektDateien`, also gilt es fuer Umbauten, Suche und Symbolverzeichnis
gleich; die Beschriftung ueber dem Baum sagt es), **ohne Vorschau wird
trotzdem geprueft** (gezaehlt wird der ZUWACHS je Datei -- eine Datei, die
schon nicht uebersetzt, wuerde sonst jeden Umbau in die Vorschau zwingen),
und **alle vier Umbauten sammeln in Schritten**. **Der Stolperstein dieser
Runde, gefunden von einem Test aus Stand 24:** das Baum-Kaestchen sass
zwischen Dreieck und Namen, also dort, wo man klickt, um eine Datei zu
oeffnen -- es steht jetzt GANZ LINKS (`tree_kast_w`). Tests
`tests/pruef/gui_gesetzter_text.dhtest` (7), `tests/pruef/gui_uhrzeit.dhtest`
(6), vier neue in `tests/test_ide.py`.

**Stufe 24 (2026-09-12):** kein Punkt aus der eigenen Liste, sondern die
Frage des Nutzers -- der Dateibaum gehoert in die LAUFZEIT, und welche
Widgets fehlen sonst noch? **`GUI_FILETREE`** (gui.rs `DateiBaum` am
`TreeState`, Zeichnen und Treffertest bleiben die des Baums): liest die
Wurzel und jeden AUFGEKLAPPTEN Ordner selbst, ein `target`-Ordner kostet
also nichts, solange niemand hineinsieht -- und jeder Ordner bekommt sein
Dreieck, auch ein leerer, weil einmal vorsorglich hineinzusehen genau das
waere, was der Baum vermeidet. **Nach aussen spricht er ueber WEGE, nicht
ueber Knoten-Nummern**: die Liste entsteht bei jedem Aufklappen neu
(`dateibaum_neu`), Auswahl kommt ueber den Weg zurueck, Zugeklapptes faellt
aus der Auswahl (wie in jedem Dateimanager) -- ein HAKEN dagegen ueberlebt
es, er war eine Entscheidung (`gehakt: Vec<String>`). Einstellungen:
`GUI_FILETREE_FILTER/SKIP` (Muster mit `;`, Ordner wie Dateien),
`ordner_zuerst`, `verborgene`, `nur_ordner`, `mehrfachauswahl`,
`kaestchen`, `klick_klappt` und `auffrischen` (ms; ohne den Takt zeigt ein
Baum eine frisch angelegte Datei erst, wenn jemand ihn anstoesst -- und
darauf zu kommen ist niemandes Aufgabe). Dazu **Haken im Baum**
(`GUI_TREE_SET "kaestchen"` + `GUI_TREE_CHECKED/SET_CHECKED`) -- er war die
letzte Auswahl-Art ohne sie; Klick aufs Kaestchen kippt NUR den Haken, mit
Tastatur gehoert die Leertaste dem Haken und Enter dem Aufklappen. Und
**`GUI_TABCONTROL`**: Reiter INNERHALB eines Fensters (bisher nur am
Fenster) -- die Kinder behalten ihre Lage, eine Seite blendet sie nur ein
und aus (`tc_von`/`tc_seite`, je Bild in `tabctl_pass` gesetzt wie beim
rollenden Panel; `widget_shown` ist die eine Stelle, die es durchsetzt).
Die Kopfbreite wird an der ZEICHENZAHL geschaetzt, nicht gemessen: der
Treffertest laeuft in `handle_press`, und dort gibt es keine Grafik. Die
IDE benutzt den Dateibaum (ihre Buchfuehrung aus Dateiliste, Knoten-Nummern
und Schluessel ist weg); im Form-Designer ist das Reiterwerk Palette,
Vorschau und GB-Code, die Seiten-Zuordnung der Controls aber nicht.
`docs/module-gui.md` sagt am Ende, was der gui weiter FEHLT (Zeitwaehler,
gesetzter Text, Akkordeon, Pfadleiste, Baum mit Spalten). Tests
`tests/pruef/gui_dateibaum.dhtest` (13), `tests/pruef/gui_reiterwerk.dhtest`
(7), zwei neue in `tests/test_ide.py`.

**Stufe 23 (2026-09-12):** die drei Punkte nach Stand 22. **Der Baum zeigt
auch Begleitdateien** (`.dhform`, `.dhsprite`, `.json`, `.md`, `.csv`,
`.png` ...). Was der Editor nicht bearbeiten kann, geht per `OPENDOC` ans
System; `pruefen` bleibt bei `.dh` (eine `.json` durch den Uebersetzer zu
schicken fuellt die Liste mit Meldungen ueber etwas, das kein Programm
ist). **Dabei fiel ein Fehler auf, den Stand 22 erst scharf gemacht hat:**
`dateiOeffnen` setzte `projektOrdner = DIRNAME(pfad)` -- ein Klick auf eine
Datei im Unterordner machte den UNTERordner zum Projekt, und der Baum zeigte
danach die halbe Arbeit nicht mehr. Der Ordner wandert jetzt nur noch, wenn
die Datei ausserhalb liegt. **Umbau in Schritten:** die beiden
projektweiten Umbauten (Umbenennen, Ersetzen) sammeln je Bild 25 Dateien
(`laufSchritt` aus der Hauptschleife), die Statuszeile zaehlt mit, ESC
bricht ab -- in EINEM Bild erledigt stuende die IDE bei vierhundert Dateien
so lange, dass man sie fuer haengend haelt, und abbrechen laesst sich
nichts, was gar nicht zum Zeichnen kommt. **Das Pruefen nennt die Fehler**
(`umbauBefunde`, oben im Unterschied) statt nur ihrer Zahl. **Der
Test-Harnisch musste wieder mit:** `ide.json`, `ide.log` und `ev.txt` lagen
IM Projektordner -- seit der Baum auch `.json`/`.txt` zeigt, standen sie
mitten in der Dateiliste; jetzt liegen sie im Geschwisterordner. Tests:
3 neue in `tests/test_ide.py`.

**Stufe 22 (2026-09-12):** die drei Punkte nach Stand 21. **Projektbaum mit
Unterordnern:** die Ordner-Knoten entstehen unterwegs und werden gemerkt
(`ordPfad`/`ordKnoten`), damit zwei Dateien aus demselben Unterordner unter
DEMSELBEN Knoten landen; aufgeklappt, weil ein Baum, der seine Ordner zuhaelt, beim
Oeffnen weniger zeigt als vorher. **Umbau vorab pruefen:** `dhrt --check`
lief bis dahin erst auf dem ERGEBNIS -- also erst, nachdem die Dateien schon
anders aussahen; jetzt ueber die Texte, die geschrieben WUERDEN, mit der
Blockauswahl von jetzt. Nur auf Verlangen (ein Umbau ueber zwoelf Dateien
braeuchte sonst zwoelf Uebersetzungslaeufe, ehe man den Unterschied sieht),
und `.dhform` wird ausgenommen -- das ist JSON, da meldete der Uebersetzer
jede Zeile. **Neu angelegte Datei bekommt einen Reiter:** erkannt am
Vorher-Stand aus dem Rueckname-Vorrat (leer = die Datei gab es nicht). Vorn
bleibt aber der Reiter, an dem man gearbeitet hat -- sonst schriebe die
gemerkte Marke in eine fremde Datei. Tests: 3 neue in `tests/test_ide.py`.

**Stufe 21 (2026-09-12):** die drei Punkte nach Stand 20. **Nachbessern**
(Strg+Umschalt+G): der zuletzt uebernommene Umbau noch einmal, mit
derselben Blockauswahl -- gerechnet wird aus den SCHRITTEN, nicht aus dem,
was jetzt in der Datei steht; `umbauLeeren` raeumt die Schritte darum NICHT
mehr weg. **Unterordner:** alles, was "im ganzen Projekt" heisst, ging
ueber `DIRLIST` und sah nur den flachen Ordner -- jetzt `projektDateien()`
(DIRLIST_REC) an 13 Stellen. Uebergangen werden Ordner, die mit `.` oder
`_` anfangen, dazu `target`/`__pycache__`: dort liegt Erzeugtes, und ein
Bau-Ordner kann zehntausend Dateien haben. **Der Test-Harnisch musste
mit:** `tests/test_ide.py` legte seine IDE-Kopie in `tmp_path/_ide`, also
IN den Projektordner -- mit Unterordnern haette jeder Umbau sie
mitgenommen; sie liegt jetzt in einem Geschwisterordner. **Konstante zaehlt
fuer den IMPORT:** beim Verschieben pruefte `brauchtQuelle` nur die Namen
aus `CODE_SYMBOLS$` -- und das kennt CONST/DIM NICHT. Ein Unterprogramm,
das `MAXHP` benutzt, wanderte ohne IMPORT und uebersetzte nicht mehr;
gepruefte werden jetzt auch die CONST-/DIM-Zeilen auf OBERSTER Ebene (die
Symbolbereiche einmal einsammeln, nicht je Zeile neu parsen). Tests: 4 neue
in `tests/test_ide.py`.

**Stufe 20 (2026-09-12):** die drei Punkte nach Stand 19. **Bloecke
auslassen:** hinter der Vorschau liegt jetzt keine Textausgabe mehr,
sondern eine Folge von SCHRITTEN (`opArt` 0 bleibt / 1 alt / 2 neu, dazu
`opHunk`); ein zusammenhaengender Lauf geaenderter Schritte ist ein Block,
und ein ausgelassener liefert beim Zusammenbauen die ALTE Zeile. **Der Text
der Datei entsteht aus denselben Schritten, die man sieht** --
`umbauAnzeigeBauen` schreibt `umbauTexte[k]` neu und baut die Anzeige, also
kann beides nicht auseinanderlaufen. Falle: nach dem Kippen geht der Fokus
auf den KNOPF, nicht in die Anzeige -- dort waere Enter ein Zeilenumbruch im
Unterschied statt des Uebernehmens. **Formulare ziehen mit:** eine
`.dhform` nennt ihre Rueckrufe beim NAMEN (`"on_click": "malen"`), und
GUI_LOAD sucht sie spaeter unter genau dem -- wer umbenennt und die Datei
stehen laesst, hat einen Knopf, der nichts mehr tut, ohne Fehlermeldung.
Ersetzt wird im TEXT und nicht ueber das json-Modul: ein neu geschriebenes
JSON haette eine andere Form, und die Vorschau zeigte die ganze Datei als
geaendert statt der einen Zeile. **Aufrufer zeigen die Durchlaeufe:** nach
einem Profillauf steht an jeder Stelle `[12x]` (`profDatei`/`profZeilen`/
`profAnzahl` aus `profilAnzeigen`); ohne Lauf steht dort NICHTS -- eine Null
waere eine Aussage, die niemand gemessen hat. Tests: 4 neue in
`tests/test_ide.py`.

**Stufe 19 (2026-09-12):** die drei Punkte nach Stand 18. **Abwaehlen in
der Vorschau:** links die betroffenen Dateien mit Kaestchen
(`GUI_LISTBOX_SET "kaestchen"`). Gefiltert wird GANZ OBEN in
`umbauSchreiben` -- vor dem Ausblenden des Fensters (die Kaestchen sitzen
darin, `GUI_WINDOW_SHOWN` ist danach FALSE) und vor dem Vorrat zum
Zuruecknehmen, sonst schriebe ein Strg+Umschalt+Z eine Datei zurueck, die
gar nicht geaendert wurde. Dabei der Fund: **eine Liste nimmt Enter fuer
sich**, der Standard-Knopf kommt dann nicht mehr dran -- wer gerade
abgewaehlt hat, steht aber genau dort; Enter und ESC werden fuer den
Listen-Fokus eigens abgefragt (dasselbe Muster wie in der Befehlspalette).
**Einwand, wenn der Name im Ziel schon steht** (`symbolInDatei$`): zwei
gleichen Namens in einer Datei sind KEIN Uebersetzungsfehler, der zweite
gewinnt -- das merkt man erst, wenn das Falsche laeuft. **CONST/DIM
verschieben:** steht die Marke auf so einer Zeile auf oberster Ebene, ist
DIE ZEILE die Einheit (`einzelzeileName$`); in einem Unterprogramm zaehlt
sie nicht, dort ist sie lokal. Tests: 3 neue in `tests/test_ide.py`.

**Stufe 18 (2026-09-12):** die drei Punkte nach Stand 17 -- und der LOOK.
**Der Baustein: Metrik `verlauf_hoehe`** (gui.rs `verlauf_anteil`): ueber
dieser Hoehe nehmen Verlauf UND Glanz im Verhaeltnis ab, ein Fuenftel
bleibt stehen; 0 = ueberall gleich, also unveraendert fuer jedes bestehende
Programm. Der Grund: `gradient` ist ein fester Helligkeitsabstand, egal wie
hoch die Flaeche ist -- ein Knopf von 28 Pixeln darf sich woelben, eine
Liste von 400 bekommt denselben Abstand ueber die ganze Hoehe und sieht aus
wie ein Schatten. Genau daran krankte die IDE (Code-Feld, Ausgabe,
Problemliste, Projektbaum sind alles grosse Flaechen); sie setzt jetzt
`themaFeinschliff` nach JEDEM Preset -- ein Preset setzt die Metriken mit --
mit Verlauf 9, Glanz 12, verlauf_hoehe 120, Schatten 12. **Das Verschobene
nimmt mit, was es braucht:** ruft der Block Unterprogramme, die in der
Quelle bleiben, importiert die ZIELdatei die Quelle (bisher ging der IMPORT
nur in die andere Richtung). **Umbau mehrfach zuruecknehmen:** ein Stapel
von zehn statt eines Standes; drei Felder plus eine Grenzliste
(`zurAnzahl`), weil es kein Feld von Feldern gibt. Dabei fiel auf, dass ein
Umbau die MARKE ans Dateiende setzte (Auswahl ueber alles + INSERT) -- der
naechste Griff traf dann nichts mehr; `parameterUebernehmen` und
`umbauZurueck` merken sie sich jetzt. **Aufrufer kennen FUNCREF:** ein Name
OHNE Klammern (`f = malen`, `GUI_ON_CLICK(knopf, malen)`) ist die Stelle,
an der entschieden wird, dass er spaeter laeuft. Tests
`tests/pruef/gui_verlauf_hoehe.dhtest` (4, am Bild gemessen) und 4 neue in
`tests/test_ide.py`.

**Stufe 17 (2026-09-12):** die drei Punkte nach Stand 16. **Klassen
verschieben:** steht die Marke in einer CLASS, ist die KLASSE die Einheit
(`klasseUm` statt des alten `inKlasse`, das nur abgelehnt hat) -- eine
Methode allein waere ohne ihre Klasse kein Unterprogramm mehr. Dabei musste
die IMPORT-Frage von `aufrufInZeile` auf **`benutztWort`** umgestellt
werden: eine Klasse wird nicht nur mit `NEW Tier(` benutzt, sondern auch
als `DIM t AS Tier` -- der Aufruf-Test haette die Datei fuer
importfrei gehalten, und sie haette nicht mehr uebersetzt. **Verschieben
legt die Zieldatei an** (erster Eintrag im Waehler); dafuer musste
`umbauAlt$` eine Datei aushalten, die es noch NICHT gibt (leerer
Vorher-Stand statt Abbruch). **Einwand bei Vererbung:** heisst eine Methode
in der Oberklasse genauso, ist die Umbenennung in EINER Klasse keine
Umbenennung, sondern eine Trennung -- ohne Uebersetzungsfehler, gerufen
wird von da an die Fassung der Oberklasse. `klassenLesen` liest Name,
`EXTENDS`-Basis (nur im TEXT der Kopfzeile, `CODE_SYMBOLS$` kennt sie
nicht) und Methoden aller Projektklassen; gefragt wird **nicht an der
Marke**, welche Klasse gemeint ist (die steht vielleicht auf einem Aufruf),
sondern welche Klassen eine Methode dieses Namens haben. Nebenbei: die
**Pfeiltasten waehlen im Waehler** -- seit Stand 4 kam man ohne Maus nur an
den ersten Eintrag, wer den zweiten wollte, musste ihn wegfiltern. Tests:
6 neue in `tests/test_ide.py`.

**Stufe 16 (2026-09-12):** die drei Punkte nach Stand 15 -- und ein
LAUFZEIT-Fehler, den der eigene Test fand. **Unterprogramm verschieben**
(Strg+Umschalt+V): die Zeilen wandern in eine andere Datei des Projekts,
samt den Kommentarzeilen darueber (allein zurueckgelassen beschreiben sie
nichts), und JEDE Datei, die es ruft, bekommt den `IMPORT` der Zieldatei --
in Drachenhauch fuegt IMPORT den Text ein, ohne diesen Teil waere das
Verschieben ein Umbau, der die Uebersetzung kaputt macht (der Test prueft
darum mit `dhrt --check`, nicht am Text). Eine METHODE laesst sich nicht
verschieben (`inKlasse`). **Umbauten fuer Methoden:** ging fast schon --
`held.setze(` findet der Sucher, weil der Punkt kein Wortzeichen ist; was
fehlte, war die Marke IN der Argumentliste (`parameterFragen` fragt jetzt
`aufrufUmDieMarke`, dieselbe Frage wie die Signaturhilfe). **Letzten Umbau
zuruecknehmen** (Strg+Umschalt+Z): `umbauSchreiben` hebt den Vorher-Stand
auf; Strg+Z im Feld nimmt nur EINEN Reiter zurueck, ein Umbau ueber sechs
Dateien waere sonst sechsmal zurueckzunehmen, in sechs Reitern, die man
dafuer erst oeffnen muesste. **Der Laufzeit-Fund:** ein Menue-Kuerzel
feuerte, und der Textbereich verarbeitete dieselbe Taste NOCH EINMAL --
`Strg+Umschalt+V` machte sein Fenster auf UND fuegte unbemerkt die
Zwischenablage ein (im Test stand danach ein `#` mitten im verschobenen
Block). `kuerzel_gefeuert` galt seit Stufe 12 nur fuer die Navigation, nicht
fuer den Strg-Block (A/C/X/V) und nicht fuer Strg+Z/Y -- also haette auch
das neue Strg+Umschalt+Z nebenbei ein Redo ausgeloest. Gegenprobe gefahren
(Waechter entfernt -> Test rot). Tests: `tests/pruef/gui_menu_ausbau.dhtest`
(+1) und 7 neue in `tests/test_ide.py`.

**Stufe 15 (2026-09-12):** die drei Punkte nach Stand 14 -- und die erste
Stufe OHNE neuen Laufzeit-Baustein; alles ging mit dem, was schon da war.
**Einwand statt stiller Umbau:** wer auf einen Namen umbenennt, den es im
Projekt schon gibt (`nameVergeben$` fragt den Symbolindex, nicht den
Uebersetzer -- zwei Unterprogramme desselben Namens sind KEIN
Uebersetzungsfehler, das zweite verdeckt das erste), bekommt den Einwand
OBEN in der Vorschau; `umbauWarnung` schaltet die Vorschau dafuer ein,
auch wenn sie abgeschaltet ist -- wer einen Einwand nur in die Statuszeile
schreibt, hat ihn nicht vorgebracht. **Parameter hinzufuegen und
entfernen:** `paramFolge` ist jetzt kein reiner Tausch mehr, sondern sagt
je Platz, WOHER er kommt (>= 0 = altes Argument, negativ = neuer
Parameter). Zwei Texte je neuem Parameter, weil die DEFINITION eine
Deklaration braucht (`hp AS INTEGER = 0`) und der Aufruf einen Wert (`0`)
-- und die Definition erkennt `parameterInText$` daran, dass ihre Stelle
die von `defStelle` ist; sonst bliebe sie der Sonderfall, den sie bisher
gerade nicht war. `alteZahl` (die Parameterzahl VORHER) entscheidet, welche
Aufrufe angefasst werden -- vorher war das `LEN(folge)`, was beim
Hinzufuegen jeden Aufruf uebergangen haette. **Aufrufer als Baum:** unter
jedem Aufruf die Aufrufer seines Unterprogramms, drei Ebenen tief; ein
Name steht hoechstens EINMAL im Baum (`gesehen`), sonst kaeme man bei
A ruft B ruft A nicht heraus. Tests: 8 neue in `tests/test_ide.py`.

**Reiter schliessen (2026-09-12, Fund des Nutzers):** ein Reiter war mit
der MAUS gar nicht zu schliessen, nur mit Strg+W -- die Reiterleiste der
Laufzeit hatte kein Kreuz. Neu `GUI_TABS_CLOSABLE(win, an)` +
`GUI_TAB_CLOSED(win)` (transient wie GUI_CLICKED; Kreuz oder MITTLERE
Maustaste, wie im Browser). **Geschlossen wird NICHTS** -- an einem Reiter
haengen die Widgets des Programms, und vielleicht will es vorher fragen.
Geometrie aus EINER Quelle (`tab_kreuz_rect`, unskaliert wie TABBAR_H
selbst), sonst schloesse ein Klick neben dem Kreuz einen Reiter. Dabei
fielen zwei aeltere Fehler auf: Strg+W nahm ungesicherte Aenderungen
WORTLOS mit (jetzt `tabSchliessenFragen` mit Sichern/Verwerfen/Abbrechen),
und beim Schliessen eines Reiters LINKS vom aktiven ruecken die uebrigen
auf -- `aktiverTab` zeigte danach auf die Datei daneben (mit Strg+W nie
aufgefallen, weil es immer den vorderen schliesst). Tests
`tests/pruef/gui_reiter_kreuz.dhtest` (5, mit Gegenprobe) und 4 in
`tests/test_ide.py`.

**Stufe 14 (2026-09-12):** die drei Punkte nach Stand 13.
**Der Baustein: `GUI_TEXTAREA_SET(ta, "tab_meldet", 1)` +
`GUI_TEXTAREA_TAB_HIT(ta)`** -- der Tabulator gehoert damit ganz dem
Aufrufer: das Feld rueckt nicht ein, sieht nicht nach Abkuerzungen, es
MELDET nur (transient wie GUI_CLICKED). Der Grund ist die REIHENFOLGE: wer
im selben Feld drei Bedeutungen hat (zum naechsten Platzhalter, Schnipsel
aufklappen, sonst einruecken), kann sie nur selbst der Reihe nach abfragen
-- die Laufzeit entschiede die ersten beiden vorher. Dann gehoert ihm aber
auch das Einruecken (`GUI_TEXTAREA_CURSOR` sagt die Spalte). Ohne den
Schalter bleibt alles wie zuvor. **Signatur-Platzhalter:** die
Vervollstaendigung setzt aus `CIRC` ein `CIRCLE(x, y, r)` mit markiertem
`x`; die Namen kommen aus `CODE_HOVER$` (Optionales ab `[` faellt weg,
`*args` und `6..8 Argumente` geben keine). **Gemerkt wird dafuer NICHTS als
ein Schalter** -- wo der naechste Platzhalter steht, rechnet der Sprung
jedes Mal neu aus dem Text (`aufrufUmDieMarke` + `argsZerlegen`); eine
Buchfuehrung ueber Stellen liefe beim ersten Tippen auseinander, und genau
tippen will man ja. Nebenbei gerade gezogen: der Tabulator ERSETZTE eine
Auswahl durch Leerzeichen, jetzt rueckt er einen markierten Block ein.
**Parameter umsortieren geht durch das GANZE Projekt** -- die Vorschau aus
Stand 13 konnte mehrere Dateien laengst, es wurde ihr nur keine gegeben;
die Umstell-Logik liegt dafuer in `parameterInText$` (Text rein, Text
raus). Ein anderer Reiter mit ungesicherten Aenderungen bricht ab (auf der
Platte staende sonst ein anderer Text als im Editor).
**"Wer ruft das auf?" (Umschalt+F12)** ist die Gegenrichtung zu F12: alle
Aufrufstellen ueber alle Dateien des Projekts, in der Liste der
Projektsuche; die Definition steht nicht dabei. Gesucht wird ZEILENweise,
weil Kommentar und Zeichenkette in Drachenhauch an der Zeile enden -- damit
ist eine Zeile fuer sich vollstaendig zu lesen und die Umrechnung
Zeichenstelle -> Zeilennummer faellt weg. Tests
`tests/pruef/gui_tab_meldet.dhtest` (4, mit Gegenprobe) und 5 neue in
`tests/test_ide.py`. **docs/ide.md hat jetzt eine Uebersichtstabelle der
Staende** -- welcher Stand was brachte, gefolgt von dem, was sie HEUTE kann.

**Stufe 13 (2026-09-12):** die drei Punkte, die nach Stand 12 anstanden.
**Der Baustein: `GUI_TEXTAREA_CLOSE_WORDS(ta, oeffner, schluesse)`** -- das
Geruest, das beim Tippen mitwaechst: `IF x > 0 THEN` plus Enter setzt das
`END IF` gleich mit darunter, die Marke bleibt dazwischen. Drei
Bedingungen, jede mit ihrem Grund: (1) die Zeile muss einen Block OEFFNEN,
und zwar nach DERSELBEN Frage, die schon ueber die Einrueckung entscheidet
(dafuer `oeffnet` aus `neuer_einzug` herausgeloest) -- getrennt beantwortet
bekaeme die einzeilige Form `IF x THEN y = 1` ein `END IF`, obwohl sie
nicht einmal einrueckt; (2) hinter der Marke darf nichts stehen, sonst
landet der Rest der Zeile HINTER dem Abschluss; (3) der Block darf nicht
schon geschlossen sein (`bereits_geschlossen` geht nach unten: tiefer
Eingerruecktes ist der Rumpf, eine Zeile aus der `aus`-Liste auf gleicher
Hoehe gehoert noch dazu) -- ohne das saete jedes Enter am Ende einer
bestehenden SUB-Zeile ein zweites `END SUB`. Bei mehreren Marken bleibt es
beim blossen Umbruch. In der IDE stehen die Paare (`CASE`/`ELSE`/`ELSEIF`
fehlen mit Absicht -- sie ruecken ein, schliessen aber nichts; `REPEAT`
bekommt `UNTIL TRUE`, weil ein nacktes `UNTIL` nicht uebersetzt).
**Vorschau vor dem Umbau** (Einstellung, per Vorgabe an): bis dahin zeigte
die IDE HINTERHER, wie viele Fehler dazugekommen sind -- die Antwort auf
die falsche Frage. Jeder Umbau geht jetzt durch EINE Stelle
(`umbauVormerken` + `umbauStarten`) mit zwei Schreibwegen: in den Reiter
(Auswahl + INSERT = EIN Undo-Schritt; das Herausloesen brauchte vorher
zwei) oder auf die Platte samt Nachziehen offener Reiter. Der Unterschied
kommt aus derselben LCS-Rechnung wie das Nebeneinander, gefaerbt wie
`git diff` (`gitFaerben` zu `diffFaerben(ta)` verallgemeinert).
**Parameter umsortieren** (Strg+Umschalt+U) ist der erste Umbau, der die
AUFRUFE mitziehen muss -- die Reihenfolge der Argumente IST die Bedeutung,
ein vergessener Aufruf uebergibt stumm das Falsche. Gearbeitet wird auf dem
GANZEN Text (ein Aufruf darf ueber mehrere Zeilen gehen), die Argumente
werden als Textstuecke vertauscht; die Definition braucht keinen
Sonderfall, `SUB name(` sieht fuer den Sucher aus wie ein Aufruf. Ein
Aufruf mit anderer Argumentzahl (weggelassener Vorgabewert) oder mit
BENANNTEN Argumenten wird uebergangen und gezaehlt -- ihn still
umzustellen waere schlimmer als es zu lassen. Tests
`tests/pruef/gui_geruest.dhtest` (10, mit Gegenprobe ohne die Listen) und
6 neue in `tests/test_ide.py`; die Knopflagen des Parameter-Fensters
rechnet der Test aus der festen Fensterlage plus `title_h` = 30 (Glas).

**Stufe 12 (2026-09-11):** die eigene Restliste abgearbeitet -- **Auswahl
erweitern** (Strg+Umschalt+Hoch nimmt die naechstgroessere Klammer: Wort,
Zeile, Block, Elternblock, ganze Datei; Runter geht denselben Weg zurueck,
ein Stapel merkt sich jede Stufe statt sie neu zu erraten), **Auswahl in
ein Unterprogramm herausloesen** (Strg+Umschalt+R) und **zwei Reiter
nebeneinander vergleichen**.
**Das Herausloesen raet die Parameter nicht:** globale Namen sieht ein SUB
ohnehin, also sind es genau die LOKALEN des umgebenden Unterprogramms, die
in den gewaehlten Zeilen vorkommen -- und wer darin auch ZUGEWIESEN wird,
geht BYREF, sonst kaeme der neue Wert nie zurueck. Danach zaehlt die IDE
die Fehler nach (`CODE_CHECK$` vorher/nachher): eine unausgewogene Auswahl
(das FOR drin, das NEXT nicht) ergibt Code, der nicht mehr uebersetzt, und
das steht in der Statuszeile statt sich beim naechsten Starten zu zeigen.
Das Vergleichen rechnet die laengste gemeinsame Teilfolge auf ZEILEN (ohne
git -- die Reiter muessen nicht gesichert sein) und markiert die
abweichenden Zeilen in BEIDEN Feldern; ueber 1500 Zeilen faellt es auf den
Zeile-fuer-Zeile-Vergleich zurueck, weil die Tabelle n*m Zellen hat.
**Der Fund, und er steckt in der LAUFZEIT:** ein Menue-Kuerzel feuerte,
und der Textbereich verarbeitete DIESELBE Taste noch einmal --
`Strg+Umschalt+Hoch` loeste den Befehl aus UND schob die Auswahl eine Zeile
hoch, also sah der Befehl eine andere Auswahl als die markierte. Ein
Kuerzel, das gefeuert hat, nimmt dem Feld die Taste jetzt weg
(`kuerzel_gefeuert` in gui.rs). Ein Fall in
`tests/pruef/gui_menu_ausbau.dhtest` haelt es fest (Kuerzel feuert, die
Marke bleibt stehen); dazu 5 neue in `tests/test_ide.py` -- das
Herausloesen geprueft an der gesicherten Datei UND daran, dass sie noch
uebersetzt.

**Stufe 11 (2026-09-11):** was der alte Qt-Editor an ANSEHEN hatte --
eine **Werkzeugleiste mit Sinnbildern** unter dem Menue (neu, oeffnen,
sichern, starten, stoppen, pruefen, debuggen, suchen, Handbuch,
Einstellungen; jeder Knopf ruft denselben Befehl wie sein Menuepunkt und
nennt im Tooltip sein Kuerzel) und **Kacheln mit Bildschirmfotos wichtiger
Beispiele** auf der Willkommensseite.
**Der Baustein: `dhrt bild <quelle.dh> <ziel.png> [bilder]`** -- ein
Programm N Bilder lang laufen lassen und das letzte sichern. Das konnte die
Laufzeit ueber `DHRT_FRAMES`/`DHRT_SCREENSHOT` laengst; aus einem PROGRAMM
heraus kam man dort aber nicht hin, weil `PROCESS_START` genau diese
Variablen dem Kind abnimmt (sonst stuerbe ein gestartetes Spiel nach N
Bildern). Das Fenster wird dabei aus dem Blick geschoben
(`DHRT_ABSEITS`, graphics.rs) -- nachgemessen: ein Fenster bei -3000/-3000
liefert DASSELBE Bild, gesichert wird der Zeichenpuffer und nicht der
Bildschirm.
**Drei Stolpersteine, alle vom Test gefunden:**
(1) `beispielOrdner` faellt auf `%PUBLIC%\Documents\Drachenhauch\examples`
zurueck, wenn im angegebenen Ordner `183_sfx_generator.dh` fehlt -- ein
Test mit einem eigenen Beispiel-Ordner bekam still die ECHTEN Beispiele
(320x240 statt 200x120). Die Markierungsdatei muss mit.
(2) Die Werkzeugleiste schiebt ALLES um ihre Hoehe nach unten; drei Tests
klickten danach im Projektbaum daneben. Die Zeilen stehen jetzt an EINER
Stelle (`_baum_y` in tests/test_ide.py).
(3) Ein Befehl, der einen Datei-Dialog oeffnet, wenn seine Eingabe fehlt,
HAENGT im Test (180 s Zeitgrenze) -- der Vergleich nimmt darum die im
Projektbaum gewaehlte Datei, und der Dialog ist nur der Rueckfall.
Dazu: `LOADIMAGE` merkt sich den PFAD; ein neu erzeugtes Vorschaubild kaeme
ohne `IMAGE_FREE` als das alte zurueck.
Tests: `dhrt bild` in `tests/pruef/dhrt_bild.dhtest`, vier neue in
`tests/test_ide.py` (darunter die Vorschau mit einem eigenen
Beispiel-Ordner und einer Farbprobe im erzeugten PNG).

**Stufe 10 (2026-09-11):** die erste Stufe OHNE Vorlage -- gegen die
Qt-IDE war nichts mehr offen, gemessen wird jetzt an dem, was beim
Schreiben fehlt. **Der Baustein: Spaltenauswahl** im Textbereich
(`GUI_TEXTAREA_SELECT_COLUMNS`, mit der Maus Alt gedrueckt halten und
ziehen) -- ein RECHTECK statt eines Laufs. Sie kostete fast nichts, weil
die Maschinerie seit den mehreren Schreibmarken dalag: jede Zusatzmarke
hatte immer schon Marke UND Anker, eine Spaltenauswahl ist also nur eine
Marke je Zeile mit derselben Spaltenpaarung. Eine Zeile, die nicht so weit
reicht, bekommt ihre Marke am ENDE statt still herauszufallen -- sonst
faende man mitten im Block eine Zeile, in der das Getippte fehlt, und zwar
erst hinterher.
In der IDE dazu: **Marke an jedes Zeilenende der Auswahl**
(Strg+Umschalt+I -- damit schreibt man eine Liste in einem Zug um),
**Zeilen sortieren / Doppelte entfernen / Leerraum am Zeilenende
entfernen** (auf der Auswahl, ohne Auswahl auf der ganzen Datei),
**im ganzen Projekt umbenennen** (Strg+Umschalt+F6) und **zwei Dateien
vergleichen** (`git diff --no-index` im git-Fenster).
**Der Kniff beim Umbenennen ueber Dateien:** `CODE_RENAME$` arbeitet auf
EINEM Text und braucht eine Stelle, an der der Name steht -- in einer
fremden Datei kennt man sie nicht. Statt zu raten, wo ein Vorkommen
KEIN Kommentar ist, werden die Fundstellen der Reihe nach probiert, bis
eine den Text wirklich aendert. Eine Stelle im Kommentar aendert nichts,
und dann kommt eben die naechste dran.
Tests `tests/pruef/gui_spaltenauswahl.dhtest` (6, mit Alt-Zug ueber echte
Maus-Ereignisse) und 5 neue in `tests/test_ide.py`.

**Stufe 9 (2026-09-11):** die Restliste gegen die Qt-IDE abgearbeitet --
gegen sie ist jetzt nichts Benennbares mehr offen. **Der Baustein:**
`GUI_TEXTAREA_SET(ta, "einzugslinien", 1)` -- ein feiner Strich je
Einrueckungsstufe, unter dem Text; die Breite einer Stufe wird an
`tabbreite` Leerzeichen GEMESSEN statt geraten, die erste Stufe bleibt frei
(ihr Strich laege am linken Rand des Textes), und eine LEERE Zeile nimmt
die kleinere Tiefe ihrer beiden nicht-leeren Nachbarn -- sonst risse die
Linie in jedem Absatz auf, und gerade dort will man sie sehen. Dazu
**`VERSION$()`**: die Fassung der Laufzeit, die gerade laeuft. Ein Programm
hatte keinen Weg, sie zu erfahren, und ein eingetippter Text veraltet --
genau so stand der Ueber-Kasten der IDE auf "Stand 7", als sie laengst
weiter war.
In der IDE dazu: **Ausgabe durchsuchen** (Strg+Umschalt+A -- bei hunderten
Zeilen der einzige Weg ohne Scrollen; derselbe Text noch einmal heisst
weitersuchen), **Marke auf die NAECHSTE Fundstelle** (Strg+Umschalt+N,
schrittweise statt alle auf einmal, mit Umlauf), **im ganzen Projekt
ersetzen** (Strg+Umschalt+H: erst zaehlen und fragen, dann schreiben; es
bricht ab, wenn ein Reiter ungesicherte Aenderungen hat -- die
ueberschrieben die Datei sonst beim naechsten Sichern), **andere Reiter
schliessen** und die Einrueckungslinien als Schalter.
**Der Fund dieser Runde ist aelter als die Stufe:** derselbe Weg kam von
der Kommandozeile mit Rueckstrichen und aus `PATHJOIN` mit Schraegstrichen,
verglichen wurde WORTWOERTLICH -- dieselbe Datei bekam aus dem Projektbaum
einen ZWEITEN Reiter. Jetzt vergleicht `wegGleich$` in einer Schreibweise;
Gross/klein bleibt, wie es ist, weil das unter Linux zwei verschiedene
Dateien waeren. Gesehen hat es der Test "andere Reiter schliessen": er
zaehlte 3 statt 2.
Tests `tests/pruef/gui_einzugslinien.dhtest` (4, drei Bildproben mit
Gegenprobe) und 7 neue in `tests/test_ide.py`.

**Stufe 8 (2026-09-11):** die Liste aus Stufe 7 abgearbeitet. **Zwei
Bausteine in dhrt**, beide nach dem bekannten Muster (die Laufzeit kennt
keine Sprache, der Aufrufer sagt die Woerter):
(1) `GUI_TEXTAREA_ABBREV(ta, woerter)` + `GUI_TEXTAREA_ABBREV_HIT(ta)` --
steht links der Marke eines der Woerter, MELDET der Tabulator es, statt
einzuruecken; was an seine Stelle kommt, setzt der Aufrufer. Geprueft wird
nur im Feld, das den Tabulator ohnehin hat (`tab_fuegt_ein`) -- sonst
gehoert die Taste dem Fokus-Wechsel, und ein Wort ohne Treffer waere eine
TOTE Taste. (2) `GUI_TREE_SET(tree, "mehrfachauswahl", 1)` plus
`SEL_COUNT/SEL_NODE/IS_SELECTED/SELECT/CLEAR_SELECTION` -- dieselben
Abfragen wie bei Liste und Tabelle; der Baum war die letzte Auswahl-Art
ohne sie. Strg+Klick sammelt, Umschalt+Klick spannt in der SICHTBAREN
Reihenfolge (ueber Knoten-Nummern traefe der Bereich bei zugeklappten
Aesten etwas anderes als das, was man sieht).
In der IDE dazu: **Schnipsel per Kuerzel** (`for` tippen, Tabulator --
jeder der 13 hat ein kurzes Wort), **Symbolverzeichnis ueber ALLE Dateien
des Projekts** (Strg+Umschalt+S; auf Zuruf gebaut, nicht je Bild, und ein
offener Reiter zaehlt mit dem Stand IM Editor statt mit dem auf der
Platte), **Zur Definition und Definition hier zeigen ueber Dateigrenzen**
(derselbe Index -- `CODE_DEFINITION` sieht nur den Text, den man ihm
gibt) und **Mehrfachauswahl im Projektbaum** samt "Gewaehlte Dateien
oeffnen" (Strg+Umschalt+E).
**Der Fund dieser Runde:** jedes Oeffnen baute den Projektbaum neu auf
(`GUI_TREE_CLEAR`), und das raeumt die Auswahl weg -- die Mehrfachauswahl
kam damit nie ueber EINE Datei hinaus, weil schon der erste Klick eine
Datei oeffnet. `baumFuellen` merkt sich jetzt die Dateiliste und laesst den
Baum stehen, wenn sie dieselbe ist. Beim SAMMELN wird ausserdem nicht
geoeffnet (nur bei genau einer Auswahl, und wenn es die angeklickte ist),
sonst kaeme mit jedem Strg+Klick ein Reiter dazu.
Tests `tests/pruef/gui_abkuerzungen.dhtest` (5, mit Gegenprobe ohne die
Liste), `tests/pruef/gui_baum_mehrfach.dhtest` (6, Klicks echt
eingespeist) und 7 neue in `tests/test_ide.py`.

**Stufe 7 (2026-09-09):** die Liste aus Stufe 6 abgearbeitet. **Ein
Baustein in dhrt:** `GUI_TEXTAREA_SWATCHES(ta, starts, laengen, farben)` +
`GUI_TEXTAREA_SWATCH_CLICKED` -- ein kleines Farbquadrat zu einem Stueck
Text; welche Stelle eine FARBE meint, weiss wieder nur der Aufrufer (die
IDE sucht `&H`-Literale mit sechs oder acht Stellen). **Die Quadrate
stehen am ENDE ihrer Zeile**, nicht direkt hinter dem Stueck: dort lagen
sie bei `CLS(&HFF8800)` genau auf der Klammer -- das hat erst das Bild
gezeigt. Zwei Fallen vom Test: ein Klick setzte im NAECHSTEN Bild doch
die Marke (die Taste ist da noch unten, der Zug-Zweig lief) -> das
Widget merkt sich, dass die Geste auf einem Feld begann; und die
Rechtecke kommen aus EINER Quelle (`ta_farbfeld_rects`) fuer Zeichnen und
Treffertest.
In der IDE dazu: **Einstellungen** (Strg+U, alle Schalter an einer
Stelle, wirken SOFORT -- ein Thema, das man erst nach dem Schliessen
sieht, waehlt man blind), **Verzeichnis der eingebauten Befehle**
(Strg+F3; ohne neuen Baustein -- die Liste ist `CODE_COMPLETE` mit leerem
Text, die Beschreibung derselbe `CODE_HOVER$` ueber einen Text, der nur
aus dem Namen besteht), **Definition hier zeigen** (Alt+F12, zehn Zeilen
ohne die Stelle zu verlassen), **Symbolspur** ueber dem Code,
**automatisch sichern** (nur was schon einen Namen hat -- sonst kaeme
mitten im Tippen ein Datei-Dialog) und ein mit dem Wortanfang
vorbelegter Schnipsel-Filter.
**Ein Fund, der nur beim ZWEITEN Start auftritt:** eine Einstellung darf
nicht in eine Variable lesen, die weiter unten angelegt wird -- beim
ersten Start steht der Schluessel noch nicht in der Datei und die Zeile
laeuft gar nicht, beim zweiten bricht sie mit "Slot leer" ab. Gefunden
hat es der Farbfeld-Test, weil er ZWEIMAL laeuft (der erste Lauf sagt,
wo das Feld liegt -- die Geometrie steht erst zur Laufzeit fest).
Tests `tests/pruef/gui_farbfelder.dhtest` (5, drei davon Bildproben mit
Gegenprobe) und 7 neue in `tests/test_ide.py`.

**Stufe 6 (2026-09-09):** die Liste aus Stufe 5 abgearbeitet, Schwerpunkt
"beim Schreiben". **Ein Baustein in dhrt**, wieder nach dem Muster der
Faltung (die Laufzeit kennt keine Sprache, der Aufrufer sagt die Woerter):
**Einrueckung beim Zeilenumbruch** -- `GUI_TEXTAREA_SET(ta, "auto_einzug",
1)` uebernimmt die Einrueckung der laufenden Zeile,
`GUI_TEXTAREA_INDENT_WORDS(ta, anfang, ende, aus)` gibt drei Wortlisten
dazu. ZWEI Listen fuer "mehr" und nicht eine: `SUB` oeffnet am ANFANG,
`THEN` am ENDE -- mit einer Liste ruckte auch ein `END SUB` die naechste
Zeile ein, und `IF x THEN y = 1` rueckt richtigerweise nicht ein. Das
Ausruecken (`aus`) prueft "die Zeile IST das Wort", nicht "faengt damit
an": so rueckt sie genau einmal aus, `END IF` aendert nichts mehr, und ein
eingefuegter Block trifft es nicht. **Falle:** das Ausruecken lief zuerst
ueber `an_marken`, und das setzt die Marke ANS ENDE des Eingesetzten --
hier faellt aber etwas VOR ihr weg; die Marke sprang an den Zeilenanfang
und die naechste Eingabe landete vor der Einrueckung.
In der IDE dazu: **Fundstellen und Klammernpaar farbig** (ueber dieselben
GUI_TEXTAREA_SPANS -- der zuletzt genannte Abschnitt gewinnt; dabei kam
ein Riegel dazu, der laengst haette dasein muessen: `faerben` lief in JEDEM
Bild durch SYNTAX_SPANS), **Vorschlagsliste beim Tippen** ab drei Zeichen
mit dem Fokus IM Code-Feld (sonst tippte man in die Liste weiter),
**git diff und log** farbig im Fenster plus geaenderte Zeilen am Rand (nur
nach dem Sichern gefragt -- ein Prozessstart je Bild braechte die IDE zum
Stehen), **Suche mit regulaerem Ausdruck** (Suchen, Ersetzen und
Projektsuche fragen EINE Stelle `passtSuche`), **Lesezeichen ueber Dateien
hinweg** samt Rueckwaerts und Liste, **Reiter wieder oeffnen**
(Strg+Umschalt+T), **Faltung nach Einrueckung** (eine FOR-Schleife ist kein
Symbol) und eine **Uebersichtskarte, die Woerter zeichnet**.
**Vier Funde, alle vom Test:** REGEX_TEST nimmt (text, muster) und meldet
falsch herum einfach FALSE; der Faltschluessel stand auf "" -- das ist die
echte Gliederung einer Datei ohne SUB, und gerade sie bekam damit nie
faltbare Bloecke; eine neue Protokollzeile mit dem Dateinamen brach einen
gruenen Test aus Stand 4; und MID$/INSTR zaehlen ab 0 (siehe Stufe 5).
Tests `tests/pruef/gui_auto_einzug.dhtest` (8, mit Gegenprobe ohne den
Schalter) und 10 neue in `tests/test_ide.py`, darunter die Hervorhebung
AM BILD mit Gegenprobe (`mehr` bleibt aus).

**Stufe 5 (2026-09-09):** die Restliste aus `docs/ide.md` abgearbeitet.
**Zwei Bausteine kamen dafuer in dhrt**, beide im Textbereich und beide so
gebaut, dass ein Feld OHNE sie sich unveraendert verhaelt: (1) **Faltung**
(`GUI_TEXTAREA_FOLDABLE/FOLD/FOLD_ALL/FOLDED/FOLDS`) sitzt in `ta_rows` --
der EINEN Quelle sichtbarer Zeilen, die Zeichnen, Klick, Pfeile,
Schreibmarke und Scroll schon vorher gemeinsam fragten; eine verborgene
Zeile faellt dort weg, und damit fuer alle auf einmal. WELCHE Zeilen einen
Block bilden, sagt der Aufrufer (die IDE nimmt CODE_SYMBOLS$) -- die
Laufzeit kennt keine Sprache. Der ENGSTE Block gewinnt, eine Marke im
Verborgenen KLAPPT AUF statt auszuweichen (bei einem Suchtreffer will man
die Fundstelle sehen), und die Falten wandern bei Aenderungen darueber mit
(`falten_nachziehen`); GUI_SET_TEXT raeumt sie weg. (2) **Mehrere
Schreibmarken** (`GUI_TEXTAREA_ADD_CARET/CARETS/CLEAR_CARETS`, Alt+Klick,
ESC): jede Aenderung laeuft durch EINE Stelle (`an_marken`), die sagt,
welcher Bereich weicht und was hineinkommt, und von HINTEN nach vorn
arbeitet -- dann bleiben die Stellen der offenen Marken gueltig und es
braucht keine Buchfuehrung. Kopieren/Ausschneiden bleiben bei der
fuehrenden Marke, Strg+A raeumt die weiteren weg. Dazu **CODE_RENAME$**
(ganze Woerter ueber `symbole::fundstellen`, ohne Kommentare und
Zeichenketten, Schluesselwoerter abgelehnt -- CODE_REFERENCES haette es
nicht getan, es wirft die Spalten weg) und **GUI_WINDOW_SHOWN**, das
fehlende Gegenstueck zu GUI_WINDOW_VISIBLE.
In der IDE: Faltung (F4/Strg+F4/Umschalt+F4, Dreieck in der
Nummernspalte), Zeilenumbruch (Alt+Z, gilt fuer ALLE Reiter), **Sitzung je
Projektordner** (als LISTE von {ordner, sitzung, aktiv}, weil ein Pfad als
JSON-Schluessel mit der Punkt-Notation kollidiert; die globale Sitzung ist
nur noch der Weg aus Stand 4 heraus -- sonst schleppte ein unbekannter
Ordner die Reiter des letzten Projekts mit), Umbenennen (Umschalt+F6),
Schnipsel (Strg+J, 13 Geruester, `|` als Marken-Platz), Signaturhilfe
(`aufrufUmDieMarke` laeuft die Zeile rueckwaerts und zaehlt Klammern),
**geteilte Ansicht** (Alt+G -- zwei REITER nebeneinander ueber
`GUI_SET_TAB(feld, -1)`, nicht zwei Ansichten auf dieselbe Datei: die
waeren ein Abgleich bei jedem Anschlag), Uebersichtskarte, git blame
(Strg+Umschalt+B ueber SHELL_OUT$), das **Handbuch gesetzt** statt roh
(Tabellen werden zur Begriffsliste -- eine Zeile je Zelle liefe rechts
hinaus), Marken auf jede Fundstelle (Strg+Umschalt+L) und gezeichnete
Menue-Symbole (IMAGE_NEW + IMAGE_CLEAR sticht die Loecher, damit sie auf
beiden Themen liegen).
**Fallen, die dabei auffielen:** MID$ zaehlt ab 0 und INSTR liefert -1
(mein Einzug kam um ein Leerzeichen zu kurz); in raylibs Aufnahmeformat ist
1 = KEY_UP und 2 = KEY_DOWN, vertauscht bleibt nach dem ersten Tastendruck
alles gedrueckt und jeder weitere Kuerzel-Test schweigt; die Wiedergabe
setzt beim ERSTEN Ereignis an, egal welche Nummer es traegt;
GUI_TEXTAREA_FIND sucht Teiltexte (aus `hp` wurde auch das in `hpmax` eine
Marke); und eine neue Protokollzeile mit demselben ersten Wort brach einen
gruenen Test aus Stand 3. Tests `tests/pruef/gui_faltung.dhtest` (16, mit
zwei Bildproben samt Gegenprobe), `tests/pruef/gui_mehrfachmarken.dhtest`
(6, seriell) und 12 neue in `tests/test_ide.py`.

**Stufe 4 (2026-09-09, die Handgriffe des Alltags):** gemessen an den
Menues der Qt-IDE statt an der Liste aus dem Entwurf. Willkommensseite
(auf der Flaeche der Code-Felder, weg mit dem ersten Reiter), Sitzung und
Einstellungen in `%APPDATA%\Drachenhauch\ide.json` (`zuletzt`, `sitzung`,
`aktiv`, `hell`, `schrift`; `DH_IDE_KONFIG` uebersteuert -- **die Tests
MUESSEN es setzen**, sonst schreibt jeder Lauf dem Nutzer eine fremde
Sitzung), Zuletzt-geoeffnet als Untermenue mit acht festen Eintraegen
(ein Menue wird Eintraege nicht wieder los, freie werden gesperrt), Datei
im Projekt oeffnen (Strg+Umschalt+O; dasselbe Fenster wie die Palette,
`paletteArt`, unscharfer Filter = Teiltext ODER Zeichen der Reihe nach),
Zeilen-Handgriffe (Kommentar Strg+K, duplizieren Strg+D, loeschen
Strg+Umschalt+K, Alt+Hoch/Runter, Alt+Rechts/Links -- Tab gehoert dem Feld),
Formatieren Umschalt+Alt+F, Lesezeichen Strg+F2/F2, Gliederung links
unten (aus `CODE_SYMBOLS$`, im Pruef-Takt), bedingte Haltepunkte
Umschalt+F9 (der Kern konnte `conditions` im set-breakpoints-Kommando
schon, nur die IDE fragte nie), Export Strg+F6 (`dhrt --export` als
eigener Prozess, Ziel `<name>_dist/`), Druckdialog (PRINTERS, Kopien,
PDF-Ausweg), TODO/FIXME (die Projektsuche mit zweitem Suchtext),
Tastenkuerzel-Uebersicht Strg+F1 (aus der Befehlsliste, die Palette und
Uebersicht teilen), Schriftgroesse. **Drei Bausteine kamen in dhrt:**
`GUI_TEXTAREA_SELECTION_RANGE(ta)` -> (z1, s1, z2, s2) -- der markierte
TEXT sagt nicht, WELCHE Zeilen gemeint sind; die Handgriffe schreiben dann
ueber SELECT + INSERT, damit jeder ein Undo-Schritt bleibt (GUI_SET_TEXT
leert den Verlauf); `CODE_FORMAT$(quelltext$[, einruecken])` = der Kern
von `dhrt fmt` (`formatiere` in main.rs, leer bei Syntaxfehler);
`GUI_WINDOW_TITLE(win, titel$)`. **Fallen:** `GUI_TEXTAREA_SELECT(ta, z,
1, z, 1000000)` klemmt ans Zeilenende -- so waehlt man eine ganze Zeile
ohne ihre Laenge zu kennen; endet eine Auswahl in Spalte 1 der naechsten
Zeile, gehoert die nicht dazu; `formatiere` liefert ohne Endumbruch, die
IDE haengt ihn wieder an, sonst gilt jede Datei als "geaendert";
`build_runtime.py` meldet einen Compile-Fehler mit Exit 0 (das Protokoll
lesen, nicht den Rueckgabewert). Tests `tests/test_ide.py` (7 neue,
seriell: Kommentar und Duplizieren ueber die gesicherte Datei, Formatieren,
bedingter Haltepunkt mit Gegenprobe -- GENAU ein Halt statt fuenf --,
Export laeuft die erzeugte Exe, Gliederung + Waehler, Sitzung ueber zwei
Laeufe), `tests/pruef/code_format.dhtest`, Auswahlbereich in
`tests/pruef/ide_bausteine.dhtest`. Offen gegen die Qt-IDE: Markdown gerendert,
Faltung, Minimap, geteilter Editor, Mehrfach-Marken, Schnipsel,
Signaturhilfe, Umbenennen, Git-Blame, Sitzung je Projekt.

## Sprachserver `dhrt lsp` + VS-Code-Erweiterung

Externe Editor-Unterstuetzung via **LSP** -- und der Server ist die Runtime
selbst: `dhrt lsp` (stdio, JSON-RPC). **Weg A aus
`docs/entwurf-python-abbau.md` (2026-09-06):** bis dahin rechnete
`drachenhauch/lsp/` in Python nach, was dhrt beim Uebersetzen laengst wusste.
Jetzt: [`src/lsp.rs`](rust/drachenhauch_runtime/src/lsp.rs) (Rahmung,
Dokumentspeicher, Verfahren, Hover-Daten) und
[`src/symbole.rs`](rust/drachenhauch_runtime/src/symbole.rs) (Port von
editor_qt/symbols.py: Definitionen, Fundstellen, Bloecke, Kommentar-Doku --
zeilenweise mit ausgeblendeten Kommentaren/Zeichenketten, bewusst KEIN Lexer,
weil ein Sprachserver halb getippten Text sieht; Spalten in ZEICHEN). Diagnose
= dieselbe Kette wie `--check` (`check_source`), auf Pufferzeilen
zurueckgerechnet (`Herkunft` aus preprocess; Fehler in importierter Datei ->
Zeile 1 mit `in datei:zeile ->`). **Diagnose im eigenen Faden** mit
Generationszaehler und 120 ms Sammelzeit -- sonst warteten Hover und
Vervollstaendigung hinter jedem didChange; stdout hinter einem Mutex. Hover:
`builtin_docs.json` (handgepflegt; die Tabelle aus `builtin_docs.py` ist seither
eine JSON-Datei, die Python laedt und dhrt per `include_str!` einbettet) vor
`builtin_prosa.json` (aus `docs/`) vor der Signatur aus dem Index; eigene
Symbole ueber den Kommentarblock ueber der Definition. Vervollstaendigung:
Index (`compiler::builtin_eintraege`), `lexer::KEYWORDS` (neue Liste, Test
haelt sie an `keyword()` fest -- 75 Eintraege), Farben/Tasten aus vm.rs, PI/TAU,
eigene Definitionen. Tests: Rust-`#[test]`s in beiden Dateien +
`tests/pruef/dhrt_lsp.dhtest` (echter Prozess ueber
stdio, Client `_hilfen/lspklient.dh`; prueft nebenbei die KEYWORDS aus
`tokens.py` gegen die Vervollstaendigung; bis 2026-09-16 pytest). Der
Qt-Editor benutzt weiter seine Python-Bausteine (`symbols.py`,
`error_check.py`) -- die gehen mit der IDE (Weg C). VS-Code-Erweiterung in
[`vscode-drachenhauch/`](vscode-drachenhauch/): `extension.js` startet
`dhrt lsp` (Einstellung `drachenhauch.dhrtPath`), die TextMate-Grammatik wird
aus `lexer::KEYWORDS` + Index **generiert** (`dhrt doku grammatik`). Doku
[docs/lsp.md](docs/lsp.md). **Bei neuen Keywords/Built-ins:** Grammatik neu
generieren (`dhrt doku grammatik`).

## Front-End-Portierung nach Rust (Lexer → Parser → Compiler)

**Laufendes Ziel:** Python langfristig nur noch in den Editoren — `dhrt` soll
selbst aus Quelltext Bytecode erzeugen (heute macht das die Python-Toolchain).
Die Front-End-Stufen werden **inkrementell** nach Rust portiert, **jede gegen
Python verifiziert** (cargo+rustc vorhanden → hier beweisbar). **ALLE 5 STUFEN
FERTIG:** Lexer (1) + Parser (2) + Compiler 3a–3e (3) + Preprocess/IMPORT (4) +
Verdrahtung (5). **`dhrt run datei.dh` ist ein eigenständiger End-to-End-Lauf
ohne Python** (preprocess→lex→parse→compile→VM, chdir ins Datei-Verzeichnis für
relative Asset-/IMPORT-Pfade; `dhrt datei.dh` ohne `run` per `.dh`-Auto-Detect
genauso, `.dhc` läuft weiter den direkten VM-Pfad). Debug-Einstiege
`dhrt --tokens` / `--ast` / `--preprocess` / `--runsrc` geben Token-Strom bzw.
AST bzw. gemergte Quelle aus bzw. führen ohne chdir aus (Dev/Parity).
Parity: tests/test_rust_lexer_parity.py
(215) + tests/test_rust_preprocess_parity.py (8)
+ [`tests/pruef/rust_run_parity.dhtest`](tests/pruef/rust_run_parity.dhtest) (2) = 225.
**Parser- und Compiler-Parity gibt es NICHT MEHR** (dieser Absatz nannte lange
beide, mit 107 bzw. 71 Tests): sie verglichen gegen den Python-Parser und
-Compiler, und die sind mit Stufe B gelöscht. Das Compiler-Gate ist stattdessen Output-Parität — siehe unten.
Dateien: [`src/lexer.rs`](rust/drachenhauch_runtime/src/lexer.rs),
[`src/ast.rs`](rust/drachenhauch_runtime/src/ast.rs),
[`src/parser.rs`](rust/drachenhauch_runtime/src/parser.rs),
[`src/preprocess.rs`](rust/drachenhauch_runtime/src/preprocess.rs),
[`src/compiler.rs`](rust/drachenhauch_runtime/src/compiler.rs). Plan/Stufen/Gotchas:
[docs/rust-frontend-port.md](docs/rust-frontend-port.md).
**Compiler-Gate = Output-Parität** (`dhrt --runsrc` stdout == Python-TW), NICHT
byte-exakter Bytecode: dhrt's VM kann beide Opcode-Formen, der Rust-Compiler
emittiert die generischen (kein Folding/`_NN`/IC) → identisches Verhalten, viel
weniger Code. **3a–3e fertig:** Skalar/Arithmetik/IF/WHILE/Builtins (3a) + FOR/Arrays/Index/
INPUT/DATA-READ/Locals (3b) + User-SUB/FUNCTION (Rekursion/CALL_USER/Named-Args/
Defaults/Variadic/FUNCREF, 3c) + Klassen/Structs (NEW/Member/Self/Methoden-
Calls/Vererbung/Properties/Operatoren/STATIC/ENUM, 3d) + SELECT/FOR EACH/REPEAT/
Tupel+Destructuring/WITH/TRY-CATCH-THROW/Slicing/List-Set-Dict-Comprehensions/
IIF/Coroutinen-YIELD + TUPLE/COROUTINE/FUNCREF/IMAGE-DIM-Typen (3e). Nicht-
unterstützt → `Err("Stufe 3e: ...")`. **Stufe 4 (Preprocess) fertig:**
`src/preprocess.rs` portiert `preprocess.process()` — `IMPORT "datei.dh"`
rekursiv inlinen (mit `' === IMPORT … ===`-Markern + `seen`-Dedup),
`IMPORT "modul"[ AS x]` → Kommentar (dhrt hat Modul-Builtins nativ). Importierte
Module liefern ihre externen Typen (`MODULE_TYPES`) an `compile_to_gbc(ast,
external_types)` → `DIM v AS VEC2` kompiliert nach `IMPORT "vec2"`. `--runsrc`
schaltet Preprocess vor. **Stufe 5 (Verdrahtung) fertig:** `dhrt run datei.dh`
(+ `dhrt datei.dh` per `.dh`-Auto-Detect) — eigenständiger End-to-End-Lauf ohne
Python, chdir ins Datei-Verzeichnis (relative IMPORT-/Laufzeit-Pfade); gemeinsame
Kette `compile_and_run_source` (geteilt mit `--runsrc`, das ohne chdir läuft).
Tree-Walker bleibt Referenz + `@builtin`-Host. AST-Parity-Gotchas: `.line` kein
Feld, `Param.by_ref` ist ein Token. 3e-Gotcha: `_collect_data` rekursiert in
SELECT/TRY, aber NICHT FOR EACH/WITH. 4-Gotcha: `MODULES`/`MODULE_TYPES` hardcoded,
mit `modules.discover_modules()` synchron halten. **Anschluss-Features fertig:**
(a) **Selbst-Export** `dhrt --export datei.dh [out]` — kompiliert Quelle → `.dhc`,
hängt Payload an Kopie der eigenen Exe (wie `export.py`, aber ohne Python),
kopiert `assets/`. (b) **Aliasierte Modul-IMPORTs** `IMPORT "json" AS j` —
`compile_env` liefert `(alias, modul)`-Paare, der Compiler bildet `j_parse`→
`json_parse` / `v`→`vec2` zurück (dhrt findet sie nativ), aliasierte externe
Typen (`j_handle`/`v`) sind gültige DIM-Typen. (c) **WASM (gebaut + verifiziert)** — emscripten-Einstieg
kompiliert `/program.dh` selbst (kein Pyodide), `build_wasm.py` bettet die Quelle
ein + verdrahtet das Windows-emscripten-Env automatisch (`setup_emscripten_env`:
CC/CXX/AR/Linker→`.exe`, bindgen-Includes, Ninja). `node web/dhrt.js` ==
Tree-Walker verifiziert. Toolchain (emscripten 6.0.0 + wasm-Target) installiert.

## Web-Playground (dhrt → WASM)

`dhrt` als WebAssembly (emscripten) im Browser. **Der Web-Build laeuft** (Stand
2026-08-03, in `docs/web-playground.md` verifiziert): Konsolen- UND
Grafik-Programme werden **im Browser kompiliert** und ausgefuehrt. Teile:
[`rust/build_wasm.py`](rust/build_wasm.py) (bettet `.dh` als `web/program.dh`
ein, `.dhc` bleibt als Fallback; dazu `cargo`+emscripten-Build, tolerant wenn
die Toolchain fehlt), cfg-gegateter WASM-Einstieg in
[`rust/drachenhauch_runtime/src/main.rs`](rust/drachenhauch_runtime/src/main.rs)
(`#[cfg(target_os = "emscripten")]` liest `/program.dh` zuerst, `/program.dhc`
als Rueckfall), Web-Harness [`web/`](web/) (`index.html` + `playground.js`),
Ton ueber ein eigenes Kira-Backend auf OpenAL
([`src/web_audio.rs`](rust/drachenhauch_runtime/src/web_audio.rs) -- cpal hat
keinen emscripten-Host mehr).

**Zwei Aussagen, die hier lange falsch standen** (die richtige Fassung stand
schon in `docs/web-playground.md`): der blockierende Render-Loop in `vm.run()`
war die *frueher* genannte Kernhuerde -- mit ASYNCIFY laeuft er durch. Und der
Compiler ist **nicht** Python geblieben: seit dem Rust-Frontend kompiliert der
Playground die Quelle selbst, **Pyodide braucht es nicht**.

Doku/Grenzen: [docs/web-playground.md](docs/web-playground.md).
Tests [`tests/pruef/build_wasm.dhtest`](tests/pruef/build_wasm.dhtest) (Geruest/Harness,
nicht der emscripten-Build; bis 2026-09-21 pytest).

## Build und Test (Kurzform)

```
python rust\build_runtime.py                                        # Runtime dhrt
rust\drachenhauch_runtime\target\release\dhrt test tests\pruef       # Pruefsammlungen
rust\drachenhauch_runtime\target\release\dhrt run ide\ide.dh          # die IDE
rust\drachenhauch_runtime\target\release\dhrt run examples\<datei>.dh # ein Programm
```
