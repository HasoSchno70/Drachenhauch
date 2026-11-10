# Die IDE (ide/ide.dh)

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Weg C mit allen Staenden, Debugger, Dock, Plugins, Hover und Vervollstaendigung.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Hover und Vervollstaendigung hinter `objekt.` (2026-10-24)

Der Hover nahm zu `spieler.name` die ERSTE Definition `name` im Text (oft
die einer anderen Klasse), die Vervollstaendigung nach `s.` bot alle 2000
Befehle an. Jetzt (`lsp.rs`): `mitglied_stelle` erkennt das Wort hinter
einem Punkt (davor Name, `)` oder `]`, keine Zahl), `mitglied_fund` setzt
`compiler::MITGLIED_MARKE` an seine Stelle und uebersetzt -- in
`warn_unbekanntes_mitglied` haelt der Compiler (Thread-Speicher
`MITGLIED_FUND`) die Klasse samt `mitglied_fund` fest (Felder, PROPERTYs,
Methoden mit Parametern, eigene vor geerbten). So stimmt es auch ueber
Indizes, Aufrufe, `Self` und Importe mit Namensraum (`KERN.GARN`); Signatur
und Kommentar kommen aus dem Quelltext samt Importen
(`mitglied_definitionen` ueber `symbole::bereiche`: lokale DIMs in Methoden
sind keine Mitglieder). Das letzte Ergebnis wird gemerkt
(`LETZTER_FUND`). Ohne Uebersetzung (Fehler anderswo im Text) bleibt es
beim alten Weg. **Nur mit Ordner:** `hover`/`vervollstaendigung` nehmen
`basis: Option<&Path>`, `CODE_HOVER$`/`CODE_COMPLETE` ein viertes
Argument `basis$` -- ohne bleibt es beim Text, weil die Parameternamen der
IDE bis zu 60 Stellen je Pruefung fragen (je 0,2 s Uebersetzen). Dazu
`klassen_hover`: in einer Klasse gilt ihr Mitglied (lokale DIM/Parameter
der Methode verdecken es). Die IDE gibt `codeBasis$()` mit, oeffnet die
Liste hinter `objekt.` ohne drei Zeichen (`nachPunkt`, `vorschlagWort$`),
aber von selbst nur bei hoechstens 100 Eintraegen -- ohne bekannte Klasse
kommt die ganze Liste, und die soll nicht hinter jedem Satzpunkt aufgehen.
**Falle beim Testen:** die leere letzte Zeile einer Beilage faellt weg,
`GUI_TEXTAREA_GOTO` auf sie landet in der Zeile davor. Tests:
Rust-Test `mitglieder_hinter_dem_punkt`, je ein Fall in `ide_bausteine`
und `dhrt_lsp`, zwei in `werkzeug_ide_vorschlag` (drei Verfaelschungen der
IDE fallen je in ihrem Fall; gegen 2026.35 fallen beide Laufzeit-Faelle).

**Namensraeume und Methodenaufrufe (selber Tag, danach):** hinter `kern.`
(IMPORT ... AS kern) bietet die Vervollstaendigung die obersten
Definitionen der Dateien dieses Alias ohne `PRIVATE` an, der Hover zeigt
die Definition dort (`namensraum_mitglieder`/`namensraum_hover` in lsp.rs,
aus `preprocess::process` samt Herkunft und Namensraeumen -- OHNE
Uebersetzen, also auch bei einem Fehler im Text). `kommentar_darueber`
endet an der Marke `' === IMPORT ... ===`, sonst stand sie in der Doku der
ersten Definition einer eingebundenen Datei. Neu `CODE_METHOD_CALLS$(quelltext$
[, basis$])`: alle Methodenaufrufe mit bekannter Klasse in EINEM Lauf
(`compiler::methoden_aufrufe_sammeln`, Thread-Speicher `METHODEN_AUFRUFE`,
gefuellt in `warn_unbekanntes_mitglied` bei `ist_aufruf`; die Klasse ist
die, in der die Methode STEHT, die Signatur ihre Kopfzeile aus dem
Quelltext). `parameterSammeln` der IDE holt die Liste einmal je Pruefung
(`methodenAufrufeLesen`, Schluessel `Zeile:methode`) und nimmt sie fuer
Namen hinter einem Punkt; `signaturArgs` hat ein freiwilliges `basis`
(nur die Platzhalter hinter einem Punkt geben es mit -- EINE Stelle).
Tests: je ein Fall in `ide_bausteine` (gegen 2026.35: 2384 Eintraege statt
drei, Hover leer) und zwei in `werkzeug_ide_parameter_einfuegen`
(Methoden zweier Klassen samt geerbter, Platzhalter nach `ob.se`),
Rust-Test `import_marke_gehoert_nicht_zur_doku`.

## Dogfooding 14: IDE-Plugin "Klassen" (2026-10-24)

`examples/plugins/klassen/` (plugin.dh + leiste.dhform): alle Klassen und
Structs des Projekts als Baum in einer Leiste rechts, darunter Felder,
Konstanten, Eigenschaften und Methoden mit Typ, Filterfeld, "Nur die Datei
vorn", ein Klick springt hin; neu gelesen nach dem Sichern (beim
Reiterwechsel nur im Modus "Nur die Datei vorn" -- sonst klappte der Baum
bei jedem Sprung nach oben). Drei Luecken der Plugin-Schnittstelle:
(1) **Kein Befehl lieferte die Klassenstruktur** -- `CODE_SYMBOLS$` kennt
weder Felder noch Typen noch die Vorfahrin. Neu **`CODE_CLASSES$(text$)`**
(`lsp::klassen_struktur`, textbasiert ueber `symbole::bereiche`/
`definitionen`, also auch fuer eine Datei mit Fehler): je Klasse name, art,
basis (EXTENDS), zeile, bis, kommentar, mitglieder; je Mitglied name, art
(feld/konstante/sub/function/property), typ, kopf, zeile, kommentar. Lokale
Variablen der Methoden sind keine Felder, GET und SET einer PROPERTY EIN
Mitglied, `DIM a, b AS INTEGER` gibt beiden den Typ. Dabei:
`doku_der_definition` nahm bei einer Kopfzeile ueber mehrere Zeilen den
Fortsetzungsstrich ` _` mit in die Signatur (auch im Hover). (2) **Ein
Plugin konnte keinen Baum fuellen** -- neu `PLUGIN_BAUM(leiste, widget,
ebenen, texte [, auf])`, Nachricht `baum` (`pluginBaum` in ide.dh; ein
Eintrag haengt am letzten davor mit kleinerer Ebene, die Knotennummer ist
die Stelle in der Liste), `PLUGIN_SETZE ... "selected"` waehlt auch im
Baum. (3) **`GUI_CHANGED` auf einem Baum war stumm**, wenn ein Knoten
gewaehlt wurde -- der Abdruck (`Gui::abdruck`) kannte die Auswahl des
Baums nicht (sie liegt in `TreeState`); `pluginWert$` liefert fuer einen
Baum den gewaehlten Knoten. `PLUGIN_OEFFNEN` ohne Pfad springt im Reiter
vorn (ungesicherter Text). Tests `tests/pruef/ide/werkzeug_ide_plugin_klassen.dhtest`
(2: das Plugin aus examples/ in der echten IDE mit echtem Klick, Filter,
Kopfzeile; allein gestartet endet es gleich), Rust-Test
`klassen_mit_mitgliedern_ohne_uebersetzen`; Gegenproben (ohne Baum in
`pluginWert$`, `_`-Ordner mitgezaehlt, Baumauswahl nicht im Abdruck) fallen
je in ihrem Fall.

## Plugin-Oekosystem: Steckbrief, mitgelieferte, Vorlage, Text aendern (2026-10-25)

Plugins an ZWEI Stellen (`pluginsAusOrdner`): eigene unter `plugins/`
neben der ide.json (laufen, ausser `plugins_aus`) und **mitgelieferte** in
`examples/plugins/` (`pluginMitOrdner$` ueber `beispielOrdner`; aus, bis
`plugins_an` sie nennt -- `pluginGewollt`/`pluginGewolltSetzen`). Eigene
zuerst, ein gleicher Name verdeckt das mitgelieferte. Das Protokoll zaehlt
`plugins N` weiter nur die eigenen (alte Tests), dazu `mitgeliefert N`.
**Steckbrief** `plugin.json` (name, fassung, beschreibung, autor, ide):
Verwaltung zeigt "Titel Fassung (mitgeliefert)", Tooltip Beschreibung/Autor/
Pfad; `ide` neuer als `VERSION$()` (`fassungVergleich`) -> startet nicht,
Zustand "braucht IDE x". **Datenordner** je Plugin `plugin-daten/<name>/`
neben der ide.json, ueber `DH_PLUGIN_DATEN` (per SETENV vor WINDOW_OPEN,
**als REALPATH$** -- bei einer ide.json ohne Ordner ist der Konfig-Ordner
".", und das Plugin laeuft in seinem eigenen Ordner) -> `PLUGIN_DATENORDNER$`;
Schreibweise schreibt seine erlaubt.txt dorthin (vorher in seinen eigenen
Ordner, also bei einem mitgelieferten ins Repository/Beispielordner).
**Vorlage** Plugins -> Neues Plugin ... (`pluginAnlegen`: Name
`[A-Za-z0-9_-]+`, plugin.dh + plugin.json + leiste.dhform, sofort
angemeldet und geoeffnet; das Muster zaehlt das Wort an der Marke).
**Text aendern:** `PLUGIN_ERSETZEN(z, s, bz, bs, text$ [, datei$])` und
`PLUGIN_AUSWAHL_SETZEN` (Nachricht `tue` mit `ersetzen`/`waehlen`, SELECT +
INSERT = ein Undo-Schritt, mit Datei wird sie erst geoeffnet). Tests
`tests/pruef/ide/werkzeug_ide_plugin_oekosystem.dhtest` (4); sieben
Verfaelschungen fallen.

## Plugin-Schnittstelle: Code-Hilfen (2026-10-25)

Drei Wege, auf denen ein Plugin im Code-Feld mitredet, als Probe das
Beispiel `examples/plugins/schreibweise/` (Schreibfehler in Kommentaren).
(1) **Befunde mit Stelle und Schnellkorrektur:** `PLUGIN_BEFUND(...,
spalte, laenge)` und `PLUGIN_KORREKTUR(titel, text [, Bereich])` (haengt am
zuletzt gesammelten Befund; ohne Bereich gilt dessen Stelle, ohne Stelle die
ganze Zeile). Die IDE haengt Plugin-Befunde jetzt in DEN JSON von
`CODE_CHECK$` (`pluginBefundeAnhaengen`, vorher eine eigene Liste neben der
Problemliste) -- damit bekommen sie Wellenlinie, Tooltip, Text am
Zeilenende und Strg+. ohne eigenen Weg; ohne Spalte unterstreicht sie die
Zeile ohne Einrueckung wie die Laufzeit. (2) **Kontextmenue:**
`PLUGIN_KONTEXT(id, titel)` -> Eintrag in `ctxCode` hinter einem Trenner,
loest denselben Weg wie ein Befehl aus (`plugin:<name>:<id>`); jedes
`befehl`-Ereignis traegt jetzt das Wort an der Marke als `wert`
(`wortAnDerMarke$`, auch als Frage `PLUGIN_WORT$`). Beim Stoppen gesperrt,
beim Neustart wieder frei (ein Menue wird Eintraege nicht los).
(3) **Hover:** `PLUGIN_HOERE("hover")` -> beim Verweilen schickt
`mausNachziehen` ueber `pluginHoverFragen` (Nummer je Stelle) Wort, Zeile,
Spalte, Datei; `PLUGIN_HOVER_ANTWORT(text)` haengt sich an den Tooltip, nur
wenn Nummer UND Stelle noch stimmen (`pluginHoverAntwort`) -- die IDE wartet
nicht. **Falle beim Testen:** steht die Maus auf einem Befund, enthaelt der
Tooltip schon dessen Meldung und "Strg+. schlaegt eine Korrektur vor." --
ein Warten auf ein Wort aus der Meldung kam zu frueh. Tests
`tests/pruef/ide/werkzeug_ide_plugin_codehilfen.dhtest` (echte Maus ueber
`_hilfen/idemaus.dh`; Gegenprobe Zeichenkette ohne Hover); vier
Verfaelschungen fallen, die Pruefung der Stelle bei der Antwort ist
ungedeckt (der Test bewegt die Maus nicht vor der Antwort weg).

## Plugins: Testhilfe, Begruessung, Vervollstaendigung, Holen (2026-10-25)

**Schein-IDE** `ide/dh_plugin_probe.dh`: startet ein Plugin wie die IDE
(WINDOW_OPEN, Begruessung, `DH_PLUGIN_DATEN` = `probe-daten/` im
Arbeitsordner), beantwortet `frage` aus einem gesetzten Stand
(`PROBE_SETZE_TEXT/AUSWAHL/PROJEKT`), wendet `tue` (ersetzen, einfuegen,
waehlen ...) auf den Text an und merkt sich alles (`PROBE_WIDGET$`,
`PROBE_BEFUNDE$`, `PROBE_LETZTE$`); Ereignisse `PROBE_BEFEHL/EREIGNIS/
KLICK/GEAENDERT`, `PROBE_HOVER$`/`PROBE_VORSCHLAEGE` warten auf die
Antwort. `dhrt pruef` kennt ihre Namen (`BIBLIOTHEKEN` in pruef.rs).
**Begruessung:** die IDE schickt nach WINDOW_OPEN `{"art":"hallo",
"protokoll":3,"ide":VERSION$()}` (`PLUGIN_PROTOKOLL_FASSUNG`); die
Bibliothek verarbeitet sie selbst in `PLUGIN_EREIGNIS$` (liefert dann ""),
`PLUGIN_IDE_FASSUNG$`/`PLUGIN_PROTOKOLL` warten hoechstens 1 s darauf
(`plgHalloAbwarten`, andere Nachrichten gehen in `plgWarte`); ohne
Begruessung "" bzw. 1. **Vervollstaendigung:**
`PLUGIN_HOERE("vervollstaendigen")` -> `vervollstaendigen` fragt ueber
`pluginVervollFragen` (Nummer je Frage) VOR dem Abbruch bei leerer eigener
Liste; `PLUGIN_VORSCHLAEGE(feld)` -> `pluginVervollAntwort` haengt an, wenn
Nummer und Wortanfang noch stimmen, und oeffnet die Liste selbst, wenn die
IDE nichts kannte (`vervollFensterZeigen` herausgeloest). **Holen:**
dritter Ort `plugins/pakete/<name>/` (`plugGeholt`), Knoepfe Holen/Entfernen
in der Verwaltung -> `dhrt paket -C <pluginOrdner> hole <quelle>` bzw.
`entferne` als Prozess `plugHolProz`, danach `pluginsNeuLaden`. Tests
`tests/pruef/werkzeugkette/plugin_probe.dhtest` (4: die drei mitgelieferten Plugins und
ein eigenes, ohne IDE) und zwei Faelle mehr in
`werkzeug_ide_plugin_oekosystem.dhtest`; fuenf Verfaelschungen (ohne
Begruessung, ohne Frage, ohne `pakete`, Probe ohne Antworten, Bibliothek
ohne hallo) fallen je in ihrem Fall.
**Vorlage mit Pruefung, Aktualisieren** (selber Tag): "Neues Plugin ..."
schreibt dazu `test.dhtest` (Probe, `DH_PATH` = `pluginBibOrdner$()` als
REALPATH$, Plugin ueber `--- argumente {sammlung}`); der Vorlage-Fall laesst
sie per `dhrt test` laufen und verdirbt das Plugin als Gegenprobe. Knopf
**Aktualisieren** = `hole --erneuern` (`pluginsAktualisieren`). **Der Fund:**
ein laufendes geholtes Plugin hat seinen Ordner als Arbeitsordner, Windows
ersetzt/loescht ihn dann nicht -- `pluginHolenStarten` stoppt jetzt alle
laufenden geholten vorher, und `pluginHolenNachziehen` laedt IMMER neu
(sonst blieb nach einem Fehler das alte aus). Fall "ein plugin ueber http
holen und aktualisieren": ZIP mit Oberordner wie von GitHub ueber
`gegenserver.dh` (dateien), geaenderter Stand ohne --erneuern = Fehler +
alter laeuft, Aktualisieren nimmt ihn an. Gegen echtes GitHub ungeprueft.

## Debugger spricht (Datei, Zeile); Marken wandern; Lichtstreif (2026-09-27)

**`dhrt debug` zaehlte Zeilen der GEMERGTEN Quelle** -- mit einem `IMPORT`
am Anfang hielt er "in Zeile 7" statt 3, und kein Haltepunkt nach dem IMPORT
traf je. `debugger.rs` hat jetzt eine `Karte` (aus `Herkunft`, die dafuer
`pfad` = kanonischer Pfad bekam; `debug_main` preprocesst ein zweites Mal)
und einen `Kanal`: stdin liest ein eigener Faden. Ereignisse tragen `file`
(Hauptdatei wie beim Aufruf, Importe kanonisch), `set-breakpoints` nimmt
zusaetzlich `breakpoints: [{file, line, condition}]` und antwortet mit
`breakpoints` (`actual` = wo er haelt: eine Zeile ohne Code rutscht zur
naechsten mit Code DERSELBEN Datei; `verified`). **Waehrend des Laufens
wirken nur `pause` und `set-breakpoints`/`set-watches` mit `"now": true`**
-- ohne den Schalter meint ein vorab geschicktes Kommando den naechsten
Halt (so schreibt `_hilfen/debugsitzung.dh` seine Skripte). INPUT unter
dem Debugger bekam frueher "" -- jetzt `{"event":"input"}` und
`{"cmd":"input","text":...}`; `run-to` ohne Haltestelle meldet
`run-to-error`. Die IDE schickt alle Haltepunkte aller Dateien, wartet nach
dem ersten Halt auf die Antwort (`debugErstWarten`), oeffnet die Datei des
Halts, F6 = Anhalten. **Marken im Code-Feld wandern mit ihrer Zeile**
(`marken_abgleichen` gleicht am TEXT ab wie `stile_abgleichen`, also fuer
jeden Aenderungsweg), `GUI_TEXTAREA_MARKS_GET` liest sie zurueck; die IDE
zieht damit Haltepunkte und Lesezeichen nach (`markenZurueckholen`). Neu
`REALPATH$`/`SAMEFILE` (Pfadvergleich ueber die Platte, die IDE vergleicht
Debugger-Pfade damit) und **`GUI_WINDOW_GLOW`** (+ `_SET/_STOP/_GLOWING`):
ein Lichtstreif ueber ein Fenster, additiv, aus sieben Dreiecks-Baendern
(mit dreien sah man Stufen), Zeit ueber `g.delta()`; die IDE zeigt ihn
beim Start und beim Starten eines Programms (`glanz` in der ide.json, unter
`DHRT_FRAMES` nur mit `DH_IDE_GLANZ=1`). Tests `dhrt_debug.dhtest` (+8),
`werkzeug_ide_haltepunkt.dhtest` (+4), `gui_glanz.dhtest`,
`gui_marken_wandern.dhtest`, `dateisystem.dhtest` (+2); Gegenprobe gegen den
Bau/die IDE davor: alle neuen Faelle fallen, der Fall "ohne now" nicht.
**Falle beim Testschreiben:** ein Pfad aus `CWD$()` in einem JSON-Kommando
braucht Schraegstriche -- ein Rueckstrich ist dort ein Escape, und die ganze
Zeile wird still verworfen.

**Lichtstreif mit drei Arten (2026-09-28, Wunsch des Nutzers):** der
Streif war ein Band, das ueber das Fenster lief. `GUI_WINDOW_GLOW_SET(win,
"art", ...)`: `streif` (Vorgabe, wie bisher), `lampe` (Fenster multipliziert
abgedunkelt, weicher Lichtkegel im flachen Bogen additiv darueber) und
`rahmen` (Streiflicht: eine schraege Lichtkante, `draw_streiflicht`; wo sie
Fensterrahmen oder Widget-Rahmen kreuzt, blitzt ein kurzes Stueck auf, der
Inhalt bekommt nur einen Hauch, nichts wird dunkler). Die IDE nimmt
`rahmen` von rechts nach links. Neuer Zeichenbefehl `Cmd::Lichtfleck` /
`Graphics::lichtfleck`: Ringe aus Dreiecken mit Farbe je Ecke ueber rlgl
(Profil `lichtfleck_anteil`: fast flach, Kosinus-Rand auf genau null),
Rueckseiten-Verwerfen dafuer aus -- gestapelte Ellipsen zeigten Stufen.
Tests zwei Faelle in `tests/pruef/gui/gui_glanz.dhtest` (Bildvergleich gegen
dasselbe Fenster ohne Licht; der Rahmen-Fall sucht die hellste Stelle je
Kante statt fester Pixel -- die Lage haengt an der Bildzeit); Gegenproben
ohne Lampen- bzw. Rahmen-Zweig fallen.

## Dock, Plugins und der Lichtstreif der IDE (2026-10-24)

**Lichtstreif ohne Stottern:** mit einer offenen Datei (Sitzung oder
Argument) pruefte die IDE sie gut eine halbe Sekunde nach dem Start -- bei
18000 Zeilen hielt das EIN Bild 614 ms fest, der Streif sprang um ein
Viertel. `Glanz::weiter` (gui.rs) geht hoechstens 50 ms je Bild weiter
(`GLANZ_SCHRITT_MAX`), die faellige Pruefung wartet `NOT
GUI_WINDOW_GLOWING(win)`, der Streif beim Start beginnt erst im vierten Bild
(`glanzStartNachziehen`). Die Parameternamen fragen eingebaute Befehle nur
mit dem Namen (`CODE_HOVER$(name, 1, 1)`, 0,05 ms) und lesen eigene
Funktionen aus ihrer Kopfzeile (`kopfZeilenSammeln`); doppelte Namen gehen
weiter ueber den Hover an der Stelle, OHNE `basis$` -- mit ihm
uebersetzte er fuer jeden Methodenaufruf (`ob.setz(` bekommt darum weiter
die Namen der ersten Klasse; der Hover der IDE selbst kennt die Klasse,
siehe "Hover und Vervollstaendigung hinter `objekt.`").
**Unter `DHRT_FRAMES` ist `DELTA()` fest 1/60** -- eine Bildzeit misst man
mit `GET_TIME()`. Tests `werkzeug_ide_lichtstreif.dhtest` (3, je eine
Gegenprobe faellt), Rust-Test `glanz_tests`.

**Dock (`gui_dock.rs`, Untermodul der gui per `#[path]`, sieht ihre privaten
Felder):** gehoert dem FENSTER (`Window::dock`), keine Widget-Art -- sonst
muesste es in die Palette des Form-Designers. Bereiche links/rechts/unten,
Gruppen mit Reitern, Trennlinien, schwebende Gruppen; alles aus EINER
Rechnung `DockState::rechnen` (Zeichnen, Treffer, Zug, GUI_DOCK_RECT).
Inhalt auf zwei Wegen: Widgets des Fensters (`GUI_DOCK_ADD`, `Widget::dock_von`;
`widget_shown` blendet aus, `im_panel_sichtbar` -> `dock_frei` sperrt, was
unter einer schwebenden Leiste liegt, `draw_window` zeichnet ihre Widgets
in `dock_zeichnen_oben`) -- VERTEILEN tut sie das Programm in GUI_DOCK_RECT
nach GUI_DOCK_CHANGED; oder ein ganzes Fenster (`GUI_DOCK_WINDOW`,
`Window::dock_host`, `dock_ordnen` haelt es in `z_order` direkt hinter dem
Gast, `bring_to_front` eines Kindes holt den Gast). `dock_pass` laeuft in
`update` NACH `update_innen` (ein Reiterklick steht so schon in
GUI_DOCK_CHANGED desselben Bildes); Ereignis = Aenderung der `unterschrift`.
Stand als JSON in logischen Punkten; `vorgemerkt` setzt spaeter angemeldete
Leisten (Plugins) an ihren alten Platz. **Schwebende Leisten ueber
angedockten Fenstern** (2026-10-24): die stehen in `z_order` hinter dem
Gast und kamen beim Zeichnen danach -- jetzt holt `draw` die obere Schicht
(`dock_oben_nachholen`) nach dem letzten angedockten Gast nach, sobald
`dock_hat_angedockte` (nicht bei Zoom), auch GUI_DRAW_WINDOW; `topmost_at`,
`window_at`, `hit_test` ueberspringen einen Gast, den eine schwebende
Leiste verdeckt (`dock_gast_verdeckt`). 27 Befehle `GUI_DOCK*`,
Tests `tests/pruef/gui/gui_dock.dhtest` (17, Aufbau in `_hilfen/dockaufbau.dh`,
Mauswege `_hilfen/dockmaus.dh`; vier Verfaelschungen fallen je in ihrem
Fall -- die erste Fassung des Ausblende-Falls war wertlos, weil die hintere
Liste ohne Rechteck gar nicht unter der vorderen lag), 4 Rust-Tests.

**Die IDE im Dock:** links Projekt + Gliederung, unten Ausgabe/Terminal und
Probleme/Debugger als Reiter (die Knoepfe "Ausgabe"/"Terminal" sind weg,
der Debugger-Reiter ist nur beim Debuggen da). `layout()` setzt das
Rechteck, `layoutLeisten` verteilt in GUI_DOCK_RECT, Kopf-Widgets (Filter,
Zaehler, Abbrechen) in GUI_DOCK_HEAD_RECT; die Mitte kommt aus
GUI_DOCK_CENTER. Vorgabe = alte Maße (links 220, unten 202, Rand 8, Kopf 22)
-- 378 von 382 IDE-Faellen liefen unveraendert. Anordnung in ide.json
`dock`, **Ansicht -> Leisten zuruecksetzen** (`dockVorgabe`). Falle: eine
`CATCH e` auf oberster Ebene kollidiert mit globalem `e` weiter unten.

**Plugins (docs/ide-plugins.md):** eigene Programme `plugins/<name>/plugin.dh`
neben der ide.json, gestartet mit `WINDOW_OPEN` (TCP-Leitung, JSON-Zeilen),
die Seite des Plugins ist `ide/dh_plugin.dh` (NICHT `plugin.dh` -- sonst
importierte der Einstieg sich selbst), gefunden ueber `DH_PATH`, das die IDE
setzt (`pluginPfadSetzen`; in Pruefungen liegt die IDE-Kopie in
`_programm/`, darum ersatzweise `wurzel/ide`). Befehle (`plugin:<p>:<id>`
ueber `menueBefehl`/`paletteEintrag`, Menue **Plugins** am Ende der Leiste),
Leisten (`GUI_FROM_JSON` + `GUI_DOCK_WINDOW`, Widgets ueber ihre NUMMER in
der Form -- die Laufzeit liest `name` nicht, die Bibliothek bildet ihn ab),
Ereignisse (`pluginEreignis`: gesichert, geoeffnet, reiter, gestartet,
beendet, geprueft), Fragen/Antworten, Befunde in `pruefen`, Schnipsel in
`schnipselLaden`, Faerbung in `faerben` (`pluginFaerben`). Absturz =
`WINDOW_ALIVE` FALSE -> `pluginStoppen`. Die Plugin-Variablen stehen OBEN
(hinter `konfigFalsch`), weil Sitzung/Schnipsel sie beim Start lesen.
`dhrt pruef namen` kennt die SUBs/FUNCTIONs aus `pruef::BIBLIOTHEKEN`
(`ide/dh_plugin.dh`). Beispiel `examples/plugins/todo/`, Tests
`tests/pruef/ide/werkzeug_ide_plugins.dhtest` (4; drei Verfaelschungen fallen).

**Glutfaden Stufe 26 (selber Tag): Leisten und Plugins.** `ansicht/leisten.dh`:
vier GtkNotebooks einer Gruppe (`gtk_notebook_set_group_name`, Reiter
detachable/reorderable) -- links, rechts_oben, rechts, unten; `create-window`
(Rueckruf mit ZEIGER-Rueckgabe) gibt einem ausserhalb losgelassenen Reiter
ein Werkzeugfenster, dessen Kreuz die Leisten heimschickt. Ein Platz ohne
SICHTBARE Seite ist aus (`lsSichtbareSeiten` -- ein ausgeblendeter Reiter
eines Plugins zaehlt nicht), waehrend eines Zugs alle an. Verschoben wird per
`g_object_ref` + remove + append, das Widget bleibt dasselbe. Anordnung in
den Einstellungen (`panel_layout`, am ENDE der Datei geschrieben, sonst
verschob sich jede bestehende Einstellungsdatei), "Leisten zuruecksetzen" im
Logo-Menue. **Drei Fallen:** (1) beim Ende raeumt GTK die Notizbuecher
nacheinander ab und jedes meldet page-removed -- `leistenEnde()` in
`fensterZu` stellt die Handler ab, sonst Gtk-CRITICAL in stderr (jede Probe
rot); (2) `switch-page` kommt BEVOR die neue Seite vorn ist -- nachgezogen
wird im Takt danach, `seitenleisteReiter` zieht selbst nach; (3) eine Leiste
mit fester Hoehe (Uebersicht, Plugin-Form) gehoert NICHT in einen
Rollbereich: dessen Mindesthoehe ist 0, und die Teilung drueckte sie zum
Streifen (gesehen nur im Bild). `ansicht/plugins.dh`: dieselbe Leitung und
Bibliothek wie die IDE (`DH_PATH` auf `../../ide`, Installation `ide/` neben
der Laufzeit), Takt `g_timeout_add(50)`, die `.dhform` als GtkFixed
nachgebaut (label/button/textinput/checkbox/toggle/slider/listbox/dropdown),
Befehle im Logo-Menue unter Plugins und im Tastenweg nach den eigenen
Kuerzeln, Ereignisse gesichert/geoeffnet/geaendert, neu in `dh_plugin.dh`
`PLUGIN_MUSTER$` und `PLUGIN_STICHE` (ein Schritt ueber
`kern.zellenSetzenJe`), ausgeschaltet ueber `plugins_disabled`. Eine Frage
wird auch bei einem Fehler beantwortet. Tests
`tests/pruef/glutfaden/glutfaden_stufe26.dhtest` (2; echtes Plugin; sechs
Verfaelschungen fallen); die Befehlstafel hat 80 Eintraege.

**Fund in der Laufzeit dabei (Maschinencode + FFI-Rueckrufe):** ein
Schleifenbereich im Wertemodus haelt Globale in seinen Plaetzen; rief er eine
VM-Funktion, die eine Bibliothek aufruft, und die Bibliothek rief einen
Rueckruf (GTK page-removed bei `gtk_notebook_remove_page`, qsort mit
Vergleich), sah der Rueckruf diese Globalen als NIL ("Index-Zugriff auf
NIL"). `gerufenes_harmlos` lehnt jetzt Gerufenes ab, das einen Rueckruf
VERGIBT (eine Konstante mit Rueckruf-Signatur, `ffi::signatur_mit_rueckruf`)
oder `__ffi` ruft, sobald Rueckrufe vergeben sind; und ein Bereich, der vorher
gebaut wurde und `__ffi` erreicht (`erreicht_ffi`, `Schleife::ffi_gerufen`),
wird beim naechsten Eintritt verworfen. Faelle in `tests/pruef/sprache/ffi.dhtest`
("ein bereich ruft eine funktion, die einen rueckruf vergibt", "... gebaut
bevor ein rueckruf vergeben war" -- `signal`/`raise` der C-Bibliothek, die
sich einen Rueckruf merkt); gegen den Bau davor fallen beide.

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
KEY_UP kommt (F5 = 294, F7 = 296). Tests: `tests/pruef/ide/ide_bausteine.dhtest`
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
Kacheln. Tests: sieben neue Faelle in `tests/pruef/werkzeuge/werkzeug_circuitrunner.dhtest`
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
mit `make_tiles.py` geloescht. Tests `tests/pruef/grafik/image_formen.dhtest` (7,
Gegenprobe ohne die Aufrufe: 7 von 7 fallen) und ein Fall "kachelsatz wird
gezeichnet" in `werkzeug_circuitrunner.dhtest`.

**Download-Programme der Beispiele (selber Tag):** die sechs `.py`-Downloader
neben den Beispielen sind Drachenhauch -- `examples/assets/download_cybermatic.dh`,
`download_hdri.dh`, `download_robot.dh`, `download_techno.dh` (gemeinsamer Teil
`examples/assets/_laden.dh` per `IMPORT`; `dhrt pruef beispiele` zaehlt nur die
obere Ebene von `examples/`, ein Helfer darunter ist kein Beispiel),
`fireplace/assets/download.dh` und `dhdemo/download_music.dh`. Adressen,
Mindestgroessen und Meldungen wie zuvor; dhdemo laedt in eine `.teil`-Datei und
speichert nur, was die XM- oder MOD-Kennung traegt. Optionale Argumente
(Adresse, Ziel, Mindestgroesse) sind fuer Pruefungen da. Der Gegenserver stellt
im echo-Modus Pfaden ab `/xm` "Extended Module: " voran, damit der Erfolgsweg
pruefbar ist. Tests `tests/pruef/werkzeuge/werkzeug_downloads.dhtest` (3; Gegenprobe mit
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
`MAPGET`. Tests `tests/pruef/werkzeuge/werkzeug_beispiel_assets.dhtest` (2; Gegenprobe
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
`tests/pruef/werkzeuge/werkzeug_pyramid_levels.dhtest` (3, darunter ein per
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
`tests/pruef/werkzeuge/werkzeug_beispiel_bilder.dhtest` (Punkt fuer Punkt gegen das
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
dann. Tests zwei Faelle in `tests/pruef/werkzeuge/werkzeug_beispiel_bilder.dhtest`
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
Test `tests/pruef/werkzeuge/werkzeug_platformer_sprites.dhtest`.
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
unveraendert. Test `tests/pruef/werkzeuge/werkzeug_pyramid_art.dhtest` (alle 16 Bilder
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
`tests/pruef/werkzeuge/werkzeug_cloudserver.dhtest` (5: alle Zusagen aus `test_server.py`
und mehr gegen EINEN Server, offener Modus mit Warnung, CORS an Vorabfrage und
Abweisung, das `cloud`-Modul Ende zu Ende, Neustart mit derselben Datenbank);
Gegenprobe ohne Schluesselpruefung: 3 von 5 fallen, ohne die Kopfzeilen der
CORS-Fall. `server.py`, `test_server.py` und `requirements.txt` sind geloescht.
Damit laeuft bei den Spielen nichts mehr ueber Python. Im selben Zug
`bench_dhrt.dh` statt `bench_dhrt.py` (Bestwert aus N Laeufen von `dhrt run`,
Ausgabe in derselben Form; ohne `--dhrt` misst er die Laufzeit, die ihn
ausfuehrt). Die Ausgabe des Kindes wird waehrend des Laufs geleert -- ein voller
Puffer hielte es an und verfaelschte die Zeit. Test
`tests/pruef/werkzeuge/werkzeug_bench.dhtest` (Form der Ausgabe plus Fehlerlauf;
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
direkt nach dem gesicherten Bild. Tests `tests/pruef/werkzeuge/werkzeug_buch.dhtest`
(apply gegen Node-Referenz und leerer Patch auf eine Kopie von `en.json`, beide
shoot-Werkzeuge an Beilagen) und `tests/pruef/werkzeugkette/dhrt_bild.dhtest` (der Test aus
`test_dhrt_werkzeuge.py`, seit 2026-09-16 `tests/pruef/werkzeugkette/dhrt_werkzeuge.dhtest`, plus der `WHILE TRUE`-Fall).
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
`tests/pruef/werkzeuge/werkzeug_buch_bauen.dhtest` (Messen an einem PDF aus dem
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
`tests/pruef/werkzeuge/werkzeug_showcase.dhtest` (5; Gegenprobe mit Alpha/Breite 400
und ohne showcase.json: die jeweiligen Faelle fallen).

**Die dhrt-Werkzeuge pruefen sich selbst (2026-09-16, Weg D):**
`tests/test_dhrt_werkzeuge.py` (31 Tests) und `tests/test_dhrt_call.py` (9) sind
geloescht; ihre Faelle stehen in `tests/pruef/werkzeugkette/dhrt_werkzeuge.dhtest` (21) und
`tests/pruef/werkzeugkette/dhrt_call.dhtest` (6). Gestartet wird dhrt als Kind ueber den neuen
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
stehen in `tests/pruef/buecher/buch_pruefungen.dhtest` (5). Die Kapitel sind
JavaScript-Module -- gestartet werden die Werkzeuge unter `tools/` und
`buch-referenz/buch/` ueber `tests/pruef/_hilfen/nodelauf.dh`
(`nlStarten(ordner, skript)` laeuft IM Ordner, weil sie ihre Dateien relativ
suchen; `zahlVor(text, wort)` liest die Bilanzzahl, die Schranke gegen den
leeren Lauf). Ohne Node meldet der Fall `UEBERSPRINGEN: node fehlt` und gilt
als uebersprungen -- **dafuer kann sich ein Fall seit dieser Runde selbst
ueberspringen** (die Zeile auf stdout oder stderr, `bewerten` in
pruefsammlung.rs, Doku `docs/werkzeuge.md`): fuer fremde Werkzeuge, die nicht
ueberall liegen. Die Alternative waere, die erwarteten Zeilen zu ERFINDEN --
gruen und wertlos. Dazu test_beispiele_gui_enden.py (pytest) -> `tests/pruef/beispiele/beispiele_gui_enden.dhtest`
(zehn Fenster-Programme, 8 s) -- und dafuer **`dhrt run --bilder N`**: dieselbe
Grenze wie `DHRT_FRAMES`, nur als Schalter, weil `PROCESS_START` seinen Kindern
die Variable abnimmt (sonst stuerbe ein gestartetes Spiel nach N Bildern). Ein
Programm mit `WHILE TRUE` laeuft trotzdem weiter -- genau daran erkennt man es;
die Gegenprobe (eine Zeile im Beispiel umgestellt) laesst den Fall nach 25 s
fallen.
Dazu test_dhrt_check.py (pytest, 18 Tests) -> `tests/pruef/werkzeugkette/dhrt_check.dhtest`
(9): die Diagnosen von `dhrt --check` samt Zeilennummern, die Warnungen
(unbekanntes Builtin, Hardware-IMPORT, doppeltes DIM mit anderem Typ -- seit
2026-09-30 auch mit gleichem Typ im selben Block, Faelle in
`compiler_warnungen.dhtest`) und die
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
den Mitteln von heute. `tests/pruef/sprache/bibliothek.dhtest` (9) startet das
Programm als Kind und setzt `DH_PATH`, `USERPROFILE` und `HOME` per SETENV --
der Benutzerordner zeigt dabei IMMER in den Fallordner, sonst entschiede eine
Bibliothek auf dem Rechner des Pruefenden mit. Der Pfadtrenner kommt aus
`dlPfadTrenner$` im Helfer `dhrtlauf.dh` (einen Befehl fuer das
Betriebssystem gibt es nicht; ein Windows-Pfad traegt den Doppelpunkt hinter
dem Laufwerk). In tests/test_bibliothek.py bleiben nur die drei Tests der
PYTHON-Aufloesung, die die Qt-Editoren fuer die Zeilen-Herkunft brauchen.
`tests/pruef/module/dateisystem.dhtest` (15) ersetzt die pytest-Datei ganz: die
fnmatch-Zusagen stehen als festes Ergebnis fuer den Baum der Beilagen, die
Listen werden vor der Ausgabe sortiert, und "welche Datei ist neuer" wartet
1,1 s zwischen Beilage und neuer Datei statt die Zeit auf 1970 zu setzen
(ohne das Warten faellt der Fall -- beide tragen dieselbe Sekunde).
Gegenproben mit verfaelschten Kopien: 6 von 9 und 10 von 15 fallen, die
uebrigen Faelle haengen nicht an der verfaelschten Stelle.

**Die Doku-Pruefungen ohne Python (2026-09-16, Weg D):** test_doku_aussagen,
test_docs_codebloecke, test_vscode_grammatik und test_builtin_prosa (pytest)
sind geloescht; ihre Faelle stehen in `tests/pruef/werkzeugkette/doku_pruefungen.dhtest`
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
test_gui_draw_window (9 Faelle nach `tests/pruef/gui/gui_draw_window.dhtest`),
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
`tests/pruef/beispiele/beispiele_laufen.dhtest` (23 Konsolen-Beispiele als Kind: Rueckgabe
0 und nicht leere Ausgabe, mehr ist bei Zeit/Zufall/Dateien keine Zusage) und
test_beispiel_sqlite_tabelle (pytest) -> `tests/pruef/beispiele/beispiel_sqlite_tabelle.dhtest`
(Demo 158 mit `dhrt run --bilder 20`: Zeilen, Dateizeit und Groesse des
Originals unveraendert, Arbeitskopie weg; VACUUM INTO kopiert alles).
`pyramid_pusher.db` ist nicht versioniert -- ohne sie ueberspringt sich der
Fall. Gegenproben: falscher Beispielpfad bzw. eine Demo-Kopie ohne das
Aufraeumen fallen; "Original unberuehrt" ist bewusst NICHT gegengeprueft, weil
die Mutation die echte Spielstand-Datenbank beschriebe.

**SPEAK und der Sprachserver ohne Python (2026-09-16, Weg D):**
test_speak (pytest) -> `tests/pruef/ton/speak.dhtest` (10, seriell; die WAV liest ein
`--- nachher` ueber ihre Bytes) und test_dhrt_lsp (pytest) ->
`tests/pruef/werkzeugkette/dhrt_lsp.dhtest` (11). Der LSP-Client `_hilfen/lspklient.dh`
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
und `OPENDOC` stehen in `tests/pruef/module/drucken.dhtest` (4). In
der pytest-Datei blieb NUR der Druck durch den echten Treiber
"Microsoft Print to PDF": dessen Datei ist kein PDF von dhrt, und der Leser
in Drachenhauch (`pdftext.dh`) kannte nur krilla-PDFs und keine Textlagen --
PyMuPDF blieb dafuer. **Am 2026-09-20 zog auch dieser Fall um**, weil der
Leser beides lernte (siehe "Die letzten vier pytest-Dateien"). Gegenproben nur an der Vorschau: eine Verfaelschung
der Druckbefehle koennte echt drucken.

**Export, Uhr und Firmata ohne Python (2026-09-17, Weg D):**
test_rust_export, test_export_assets und test_export_signierbar (pytest) ->
`tests/pruef/werkzeugkette/dhrt_export.dhtest` (6), test_builtins_uhr (pytest) ->
`tests/pruef/laufzeit/builtins_uhr.dhtest` (6), test_firmata_module (pytest) ->
`tests/pruef/geraete/modules_firmata.dhtest` (2). Die exportierte Exe startet der
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
test_gui_barrierefreiheit (pytest) -> `tests/pruef/gui/gui_barrierefreiheit.dhtest`
(7, seriell) und test_rekursionstiefe (pytest) -> zwei Faelle mehr in
`tests/pruef/laufzeit/rekursionstiefe.dhtest`, die `vm.rs` und `build.rs` lesen. Der
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
fallen), `tests/pruef/werkzeugkette/dhrt_lsp.dhtest` prueft das Ende mit.

**Die Buecher ohne Python (2026-09-17, Weg D):** fuenf pytest-Dateien, vier
Sammlungen. Signaturen im Referenzbuch gegen `builtin_index.json` -> drei Faelle
mehr in `tests/pruef/buecher/buch_pruefungen.dhtest` (`_hilfen/signatur.dh` baut
`arity` und `formen_je_befehl` als Zeichen-Scanner nach -- das regex-Modul
kennt keine Treffer mit mehreren Gruppen). Tippspiel-Band ->
`tests/pruef/buecher/buch_tippspiel.dhtest` (21: jeder Kapitelstand als Kopie, die
Pruefprogramme, Buch-Skript und docx-Build in den Fallordner). Einstiegsbuch ->
`tests/pruef/buecher/buch_einstieg.dhtest` (7). EPUB -> `tests/pruef/buecher/buch_epub.dhtest`
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
drei Faelle in `tests/pruef/werkzeuge/werkzeug_tracker.dhtest` an einer Beilage, die
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
kostete Zeit und Speicher. Tests `tests/pruef/ton/sample_note.dhtest` (14:
Tonhoehe an den Nulldurchgaengen, Loop-Arten an einer Rampe, Huellkurve) und
in `werkzeug_tracker.dhtest` ein Lied, das der Fall im `--- vorher` selbst
baut (Sinus mit bekannter Frequenz, Keymap mit stiller und Loop-Zone).
**Die alte Beilage taugte dafuer nicht:** ihr Sample hat 64 Werte (3 ms) --
ein 10-ms-Fenster sieht es gar nicht, der Fall "bleibt stumm" waere auch mit
Wiedergabe gruen geblieben; er prueft jetzt eine UNBEKANNTE Art. Gegenproben:
fuenf Verfaelschungen des Piloten (ohne Grundtaste, immer erste Zone, ohne
Loop, alte Stummregel, jede Art als Sample) und zwei der Laufzeit (pingpong
als vorwaerts, Schritt ohne Abtastrate) -- jede faellt.

**Der DH-Code des SFX-Generators nimmt den Pan mit (2026-09-17):** aus der
Bestandsaufnahme -- [DH-Code kopieren] gab nur `s = AUDIO_SFX(...)` heraus,
ohne IMPORT, ohne Abspielen und ohne den Pan-Regler. Der Pan liegt am
WIEDERGABE-Kanal, nicht im Klang (`AUDIO_SFX` kennt ihn nicht), und ein
fehlender Pan faellt niemandem auf -- der kopierte Code klang schlicht anders
als die Vorschau daneben. Jetzt kommt ein lauffaehiges Stueck heraus (IMPORT,
`DIM snd AS SOUND`, der Aufruf, dann `PLAYSOUND` bzw. `AUDIO_PLAY` +
`AUDIO_PAN` mit derselben Rechnung wie das Vorhoeren). Test: ein Fall in
`tests/pruef/werkzeuge/werkzeug_sfx.dhtest` prueft die Zeilen UND laesst `dhrt --check`
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
dort ein `import` ergaenzt, muss es in `tests/pruef/werkzeugkette/bauskripte.dhtest` in die
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
`tests/pruef/ide/werkzeug_ide_wiederherstellung.dhtest` (6); Gegenprobe mit
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
Titel mit Kennung). Tests `tests/pruef/fenster/fenster_schliessen.dhtest` (5) und
`tests/pruef/ide/werkzeug_ide_schliessen.dhtest` (2); Gegenproben: ohne den
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
Baustein in dhrt. Tests `tests/pruef/ide/werkzeug_ide_ausgabe.dhtest` (3:
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
(`kuerzel_taste`). Tests `tests/pruef/gui/gui_zoom_rad.dhtest` (4),
`tests/pruef/ide/werkzeug_ide_zoom.dhtest` (1), Rust-Test `plus_und_minus`.

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
`tests/pruef/gui/gui_textarea_pos.dhtest` (2, ohne feste Pixellagen: abgetastet,
nur die Wechsel ausgegeben) und `tests/pruef/ide/werkzeug_ide_tooltip.dhtest`
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
-- gegen den alten Bau faellt er. Tests `tests/pruef/gui/gui_rand_klick.dhtest`
(3) und `tests/pruef/ide/werkzeug_ide_haltepunkt.dhtest` (3: links + Debugger
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
Tests `tests/pruef/ide/werkzeug_ide_suche.dhtest` (10; fuenf Verfaelschungen
der IDE -- ohne Wortgrenze, immer ohne Gross/klein, Ersatz unmaskiert,
Ersetzen ohne Trefferpruefung, Lage per INSTR -- lassen je genau ihre
Faelle fallen), drei Faelle in `tests/pruef/module/modules_regex.dhtest`,
`tests/pruef/gui/gui_auswahl_ohne_fokus.dhtest` (2, als Paar; gegen den alten
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
Warnung. Tests: zwei Faelle mehr in `tests/pruef/werkzeuge/werkzeug_notenblatt.dhtest`
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
Faelle mehr in `tests/pruef/werkzeuge/werkzeug_sprite.dhtest` (70 x [Neu] Bild und
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
Tests `tests/pruef/werkzeuge/werkzeug_kreuz.dhtest` (24, echte Fensternachrichten ueber
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
`tests/pruef/grafik/grafik_aufraeumen.dhtest` haelt die Reihenfolge fest
(Gegenprobe mit der alten: faellt), und der Paket-Lauf laesst ein
Fensterprogramm unter xvfb mit Rueckgabe 0 enden. **Gemessen im
Paket-Lauf:** Linux (x86_64) 22 MB, install.sh in ein frisches HOME, die IDE
startet unter xvfb und endet sauber, die Sammlung laeuft mit Grafik ganz;
macOS (arm64) 22 MB, `codesign --verify` gueltig, die Laufzeit aus dem
Bundle fuehrt Programme aus, der Starter legt die Beispiele an -- **ein
Fenster geht auf den macOS-Laeufern nicht** (kein OpenGL); **seit
2026-10-11 auf einem echten Mac belegt** (der Nutzer: "es funktioniert",
Fassung 2026.27). Bekannt offen: eine `.dh` per
Doppelklick im Finder oeffnete die IDE nicht mit der Datei (seit demselben
Tag behoben, siehe "Dateien vom Finder" unten), keine
Beglaubigung (Notarisierung); ein gescheitertes Fenster endete in einem
Panic -- seit demselben Tag eine Meldung (naechster Absatz). Tests `tests/pruef/werkzeuge/werkzeug_paket.dhtest`
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
Tests `tests/pruef/laufzeit/kein_fenster.dhtest` (4, geben die Meldung nie selbst aus,
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
`tests/pruef/fenster/abgelegte_dateien.dhtest` (2) plus zwei Zeilen in
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
`tests/pruef/werkzeugkette/dhrt_test_format.dhtest` (3). **Die innere Sammlung schreibt
jeder Fall SELBST** (`WRITEALL` + `JOIN$`) statt als `--- datei`-Beilage:
ihre Zeilen beginnen mit `===` und `---`, und der aeussere Laeufer haelt
jede solche Zeile fuer seinen eigenen Trenner. (2) `test_drucken.py`:
`tests/pruef/module/drucken.dhtest` (+1, `--- system windows`, ueberspringt sich
ohne "Microsoft Print to PDF"). (3) `test_os_builtins.py`:
`tests/pruef/laufzeit/os_builtins.dhtest` (+2). (4) `test_midi_module.py`:
`tests/pruef/geraete/modules_midi.dhtest` (+6, Selbst-Ueberspringen je Stufe --
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
`tests/pruef/werkzeuge/werkzeug_installer.dhtest` (3, mit erfundener `cargo metadata` und
Attrappe statt cargo; Gegenprobe ohne Kleinschreibung beim Sortieren faellt).
**Falle:** es gibt keinen Befehl fuer den Namen des Betriebssystems -- `bauen.dh`
sucht darum einfach beide Dateinamen (`dhrt.exe`, `dhrt`).
**Seit 2026-09-21 hat er alles, was nur der Qt-Installer hatte:** die Buecher
(`.docx`/`.epub` nach `{app}\buecher`, wenn gebaut), die ESP32-Sketche, das
Aufraeumen von GameBasic (Ordner, Verknuepfungen, ProgID, `.gb` nur wenn es noch
auf uns zeigt) und das Signieren ueber `DH_SIGN_CERT`/`_PASS`/`_TS`. Signiert
wird eine KOPIE der Laufzeit (`<ausgabe>/stufe/dhrt.exe`, an ISCC als
`/DDhrtQuelle`) -- die gebaute laeuft womoeglich gerade als `bauen.dh` --, dann
der Installer; ein Fehlschlag bricht VOR ISCC ab statt nur zu warnen. Tests mit
Attrappen fuer ISCC/signtool (`.cmd`, die `PROCESS_START` direkt startet) in
`tests/pruef/werkzeuge/werkzeug_installer.dhtest`; vier Verfaelschungen fallen.
**Nur was git kennt (2026-11-01):** examples/ und esp32/ gehen mit allen
Unterordnern ins Paket -- im Arbeitsordner lagen dort 21,7 MB gitignorierter
Downloads (Installer 2026.39 zuerst 80 statt 60 MB; fruehere Fassungen
liefen in sauberen Worktrees). `paket.dh::gitFremdSammeln` fragt EINMAL
`git ls-files --others` (ignoriert UND nicht eingecheckt,
`core.quotepath=off`), `ordnerKopieren` uebergeht sie; ohne git oder wenn
die Wurzel nicht die Spitze ihres Repositorys ist (`rev-parse
--show-toplevel` + SAMEFILE), alles wie frueher, mit Hinweis. Windows: die
Beispiele kommen ueber eine Bereitstellung `stufe/examples` (je Bau neu) als
`/DBeispiele`/`/DEsp32` an die .iss. Tests drei Faelle in
`werkzeug_paket.dhtest`, einer in `werkzeug_installer.dhtest`; sechs
Verfaelschungen fallen je in ihrem Fall.
**Logo und Schriftzug (2026-09-21):** `dhrt.exe` traegt kein Symbol (ein
exportiertes Spiel ist eine Kopie davon) -- Verknuepfungen, `.dh`-Dateien und
die Deinstallation zeigten darum das nackte Programm. Der Installer legt jetzt
`drachenhauch.ico` nach `{app}` und nimmt es ueberall; dazu `daten/bilder/*.png`
(auch in den macOS-/Linux-Paketen). Die IDE laedt beides aus
`<wurzel>/daten/bilder/`: das Logo als `WINDOW_ICON`, der Schriftzug steht auf
der Willkommensseite statt des Titels als Text (vorab auf 240x112 skaliert,
der Satz rechts daneben, alles ab den Knoepfen bleibt an seiner Stelle); ohne
die Dateien bleibt es beim Text. Tests `tests/pruef/ide/werkzeug_ide_schriftzug.dhtest`
(3) und zwei Zeilen in `werkzeug_paket.dhtest`.
**Video und der Vorspann der IDE (2026-09-23, Wunsch des Nutzers):** neues
Modul `video` (Feature `video`, auf dem Desktop immer dabei) -- MP4 ueber die
Crate `mp4` (reines Rust), H.264 ueber `openh264` 0.9 (Ciscos Decoder, beim
Bau aus dem Quelltext uebersetzt). `src/video.rs` kennt kein Fenster und
liefert RGBA; die VM (`try_video`, `VideoZustand`) schreibt jedes neue Bild
per `image_punkte_setzen` in ein gewoehnliches IMAGE (hochgeladen beim FLIP).
Die Uhr geht mit `DELTA()`, hoechstens einmal je Bild weiter (`bild_zaehler`),
auch wenn DRAW und UPDATE beide gerufen werden; am Ende bleibt das letzte Bild
stehen. **Falle:** openh264 zieht per Vorgabe nach jedem Paket ein Bild heraus
(`Flush::Flush`) -- bei einem High-Profile-Video mit acht Slices je Bild kam
nach neun Bildern "out of memory" (dsOutOfMemory), danach nichts mehr; mit
`Flush::NoFlush` und `flush_remaining()` am Ende laufen alle 240. Und 120 mal
1/60 s aufsummiert liegt knapp unter 2,0 -- die Bildnummer bekommt 1e-6
Zugabe. Kein Ton, nur H.264. Die IDE spielt `daten/video/vorspann.mp4` vor
allem anderen (Taste/Klick ueberspringt, Kreuz beendet, Einstellung
`vorspann`); ein Testlauf mit `DHRT_FRAMES` zeigt ihn nur mit
`DH_IDE_VORSPANN=1`. Installer und Pakete nehmen `daten/video` mit. Tests
`tests/pruef/grafik/video.dhtest` (8), `tests/pruef/ide/werkzeug_ide_vorspann.dhtest` (4;
mit der Maus -- `KEY_ANY_HIT` blendet eingespielte Tasten aus). Doku
`docs/module-video.md`.

**Beispiele nach Themen (2026-09-23):** `examples/kategorien.json` ordnet alle
203 Beispiele (ohne `_*`/`bench_*`) 16 Themen zu; die IDE zeigt sie in einem
Fenster (`themenZeigen`, Werkzeuge-Menue, Willkommensseite, Palette) als Liste
mit Gruppenkoepfen, Filter und [Oeffnen und starten]. Die Beschreibung liest
`beispielBeschreibung$` aus dem Kopfkommentar: angehaengt wird die naechste
Zeile NUR, wenn der Satz sichtlich weitergeht (Bindestrich, Komma, offene
Klammer, Kleinbuchstabe) -- sonst holte ein Titel ohne Punkt ("Coin Quest")
die naechste Zeile mit; ein Satzende braucht zwei Buchstaben vor dem Punkt
(`z.B.` endet nichts). Tests vier Faelle in
`tests/pruef/ide/werkzeug_ide_handgriffe.dhtest`, darunter die Vollstaendigkeit
der JSON gegen `DIRLIST`.

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
im Form-Designer Palette, Feld "Beschreibung" und DH-Code. (2) **Das Mausrad
rollte alles zugleich**: die IDE blaetterte ihre Kacheln, sobald die Maus im
Kachelrechteck stand -- auch unter dem offenen Handbuch. Neu
**`GUI_WINDOW_AT(x, y)`** (oberstes sichtbares Fenster, -1 = keins); die IDE
blaettert nur, wenn dort `win` oben liegt. (3) **Bereichsfarben**
(`bereichsFarben` in ide.dh): Projekt/Gliederung/Zuletzt blau, Ausgabe und
Eingabe gruen, Probleme orange, Debugger violett -- `GUI_SET_COLOR(..., "bg")`
aus `GUI_THEME_GET("widget_bg")` gemischt (dunkel 20 %, hell 16 %), nach jedem
Themawechsel neu. Tests `tests/pruef/gui/gui_kachel.dhtest` (6), ein Fall in
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
`tests/pruef/gui/gui_liste_farben.dhtest` (4, Farben im Bild, Gegenprobe ohne
Abschnitte, Zeilenhoehe ueber Bild ab), `tests/pruef/ide/werkzeug_ide_lesbarkeit.dhtest`
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
ein Feld "Schriftstil" samt DH-Code), im gesetzten Text zusaetzlich
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
mit. Tests `tests/pruef/grafik/schriftstil.dhtest` (7: Stil-Rundweg samt PUSH/POP,
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
`baum`), Abschnitte/Schritte im Feld "Eintraege", DH-Code. Tests
`tests/pruef/gui/gui_akkordeon.dhtest` (6), `gui_assistent.dhtest` (7, einer davon liest den geladenen Assistenten -- der erste Satz Faelle pruefte nur das Schreiben, und die Verfaelschung des Lesers blieb gruen),
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
`tests/pruef/beispiele/beispiel_drei_widgets.dhtest` (4: Eintrag ueber den
Assistenten samt Pflichtfeld und Bereich, Ordner ohne Groesse, Filter,
echter Klick auf einen Akkordeon-Kopf), ein Fall mehr in
`gui_assistent.dhtest` (Klick bei x 190 trifft Zurueck; mit festen Knoepfen
lag dort Weiter) und das Beispiel in `beispiele_gui_enden.dhtest`.

**Stufe 53 (2026-09-14):** die CIRCUIT-RUNNER-Engine ohne Python -- die fuenf
Engine-Tests aus `test_circuitrunner.py` stehen in
`tests/pruef/werkzeuge/werkzeug_circuitrunner.dhtest`; in pytest bleiben nur die drei
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
`ausgaben$`); `tests/pruef/werkzeugkette/dhrt_debug.dhtest` (6). **VM-Haertung:** das Geruest
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
Neu `tests/pruef/sprache/sprach_erweiterungen.dhtest` (52 Faelle) und
`tests/pruef/werkzeugkette/dhrt_builtins.dhtest` (44); die drei Tests, die nur das
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
`test_smtp.py` ist geloescht, ihre 18 Tests stehen in `tests/pruef/module/smtp.dhtest`
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
`tests/pruef/module/pdf.dhtest`.

**Stufe 43 (2026-09-13):** der Sprite-Pilot ohne Python -- und damit hat
KEIN Werkzeug in Drachenhauch mehr einen pytest-Test.
`tests/pruef/werkzeuge/werkzeug_sprite.dhtest` (41 Faelle), die pytest-Datei
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
`tests/pruef/werkzeuge/werkzeug_tilemap.dhtest` (32 Faelle), die pytest-Datei
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
`tests/pruef/werkzeuge/werkzeug_rechnungen.dhtest` (14 Faelle; die zwei Pruefungen, die
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
`tests/pruef/werkzeuge/werkzeug_tracker.dhtest` (17 Faelle), `tests/test_pilot_tracker.py`
geloescht. Ohne neues Formatstueck: ein Einschub hinter `FLIP()` schreibt je
Bild `probe.txt`, ein zweiter hinter `bildNr = bildNr + 1` ruft die
Unterprogramme des Piloten (`zelleSetzen`, `ladeDatei`, `wavRendern`,
`dhCodeSichern`). Die drei fremden Leser sind ersetzt: die Datei liest das
json-Modul mit genau den Schluesseln, die `drachenhauch.tracker.Song` liest
(`patterns.N.data.KANAL.REIHE`, leer = `null`); die zwei Dateien der
Qt-Fassung sind Beilagen, einmal mit `Song.save_json` geschrieben und auf eine
Zeile gepackt; die Mono-WAV prueft `--- ton`, die Stereo-WAV ein
`--- nachher` ueber ihre Bytes (RIFF, `BUFFER_GET_I16`), weil `--- ton` keinen
Pegel je Kanal kennt; der DH-Code geht durch `PROCESS_START("dhrt",
"--check", ...)` und einen Start. Nur der Klick-Fall braucht die Geometrie des
Fensters und schreibt seine Aufnahme darum im dritten Bild selbst.
Gegenprobe ohne `AUTOMATION_PLAY`: alle 9 Faelle mit Eingaben fallen, die
uebrigen 8 rufen den Piloten direkt und brauchen keine.

**Stufe 39 (2026-09-13):** SFX- und Partikel-Pilot ohne Python --
`tests/pruef/werkzeuge/werkzeug_sfx.dhtest` und `werkzeug_partikel.dhtest` (je 8), dazu
`tests/pruef/laufzeit/builtin_registrierung.dhtest` fuer die 14 kleinen pytest-Dateien,
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
`tests/pruef/ide/werkzeug_ide_sonderfaelle.dhtest` schreibt die 140 Zeilen im
`--- vorher`, zaehlt die Seitenobjekte (`/Type /Page /Parent` -- `/Type /Page`
allein traefe auch `/Pages`) und entpackt die Inhaltsstroeme. **Falle:** die
Suche nach `stream\n` muss HINTER dem letzten `endstream` weitergehen, sonst
trifft sie `endstream\nendobj`. Tests `tests/pruef/module/buffer.dhtest` (+4, darunter
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
ohne Fenster). Neue Sammlung `tests/pruef/ide/werkzeug_ide_sonderfaelle.dhtest` (8);
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
Faelle nach `tests/pruef/ide/werkzeug_ide_13_25.dhtest`; in `tests/test_ide.py`
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
Faelle aus `tests/test_ide.py` nach `tests/pruef/ide/werkzeug_ide_6_12.dhtest`
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
`tests/pruef/ide/ide_bausteine.dhtest`: PROCESS_*, CODE_* als Golden der
JSON-Ausgabe, Textbereich-Befehle), aus `tests/test_ide.py` sind die 31 Faelle
der Staende 1 bis 5 nach `tests/pruef/ide/werkzeug_ide.dhtest` gezogen (110
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
Faelle stehen in `tests/pruef/werkzeuge/werkzeug_animfsm.dhtest` und
`tests/pruef/werkzeuge/werkzeug_notenblatt.dhtest` (je 7). Dafuer drei Stuecke im
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
test_pilot_formdesigner.py -> `tests/pruef/werkzeuge/werkzeug_formdesigner.dhtest`
(10 Faelle; der DH-Code-Fall ruft `dhCode$()`, laesst `dhrt --check` und
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
Laden nicht); **DH-Code** (Strg+G, `dhCode$`/`dhControl`): jedes Control
als Aufruf, ohne `GUI_LOAD`, Texte mit Umbruch als `!"..."`, sich selbst
messende Konstruktoren mit `GUI_SET_BOUNDS` hinterher, das Bild
uebersprungen. Die IDE: **eine `.dhform` oeffnet den Designer** (`werkzeugMit`,
`dhrt run 197 -- datei`), als Text ueber die Befehlspalette ("Formular als
Text oeffnen"). Tests: vier neue in test_pilot_formdesigner.py (seit Stufe 32 in `tests/pruef/werkzeuge/werkzeug_formdesigner.dhtest`) --
lesen mit `json`, nicht mit dem Qt-Modell; der DH-Code-Test legt JEDE Art an
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
-1-Builtins in IDE und Beispielen vergleichen alle mit `>= 0`. Tests `tests/pruef/gui/gui_gitter.dhtest` (6, darunter
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
`tests/pruef/gui/gui_gesetzter_text_auswahl.dhtest` (4, der Mauszug
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
Ausgaben ohne Bildnummer. Tests `tests/pruef/gui/gui_leisten.dhtest` (7),
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
`tests/pruef/gui/gui_liste_komfort.dhtest` (9), `tests/pruef/gui/gui_knopfarten.dhtest`
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
`tests/pruef/gui/gui_werkzeugleiste.dhtest` (9, darunter eine Bildprobe),
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
`tests/pruef/gui/gui_gesetzter_text.dhtest` (7), `tests/pruef/gui/gui_uhrzeit.dhtest`
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
Vorschau und DH-Code, die Seiten-Zuordnung der Controls aber nicht.
`docs/module-gui.md` sagt am Ende, was der gui weiter FEHLT (Zeitwaehler,
gesetzter Text, Akkordeon, Pfadleiste, Baum mit Spalten). Tests
`tests/pruef/gui/gui_dateibaum.dhtest` (13), `tests/pruef/gui/gui_reiterwerk.dhtest`
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
`tests/pruef/gui/gui_verlauf_hoehe.dhtest` (4, am Bild gemessen) und 4 neue in
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
(Waechter entfernt -> Test rot). Tests: `tests/pruef/gui/gui_menu_ausbau.dhtest`
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
`tests/pruef/gui/gui_reiter_kreuz.dhtest` (5, mit Gegenprobe) und 4 in
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
`tests/pruef/gui/gui_tab_meldet.dhtest` (4, mit Gegenprobe) und 5 neue in
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
`tests/pruef/gui/gui_geruest.dhtest` (10, mit Gegenprobe ohne die Listen) und
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
`tests/pruef/gui/gui_menu_ausbau.dhtest` haelt es fest (Kuerzel feuert, die
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
Tests: `dhrt bild` in `tests/pruef/werkzeugkette/dhrt_bild.dhtest`, vier neue in
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
Tests `tests/pruef/gui/gui_spaltenauswahl.dhtest` (6, mit Alt-Zug ueber echte
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
Tests `tests/pruef/gui/gui_einzugslinien.dhtest` (4, drei Bildproben mit
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
Tests `tests/pruef/gui/gui_abkuerzungen.dhtest` (5, mit Gegenprobe ohne die
Liste), `tests/pruef/gui/gui_baum_mehrfach.dhtest` (6, Klicks echt
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
Tests `tests/pruef/gui/gui_farbfelder.dhtest` (5, drei davon Bildproben mit
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
Tests `tests/pruef/gui/gui_auto_einzug.dhtest` (8, mit Gegenprobe ohne den
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
gruenen Test aus Stand 3. Tests `tests/pruef/gui/gui_faltung.dhtest` (16, mit
zwei Bildproben samt Gegenprobe), `tests/pruef/gui/gui_mehrfachmarken.dhtest`
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
Laeufe), `tests/pruef/werkzeugkette/code_format.dhtest`, Auswahlbereich in
`tests/pruef/ide/ide_bausteine.dhtest`. Offen gegen die Qt-IDE: Markdown gerendert,
Faltung, Minimap, geteilter Editor, Mehrfach-Marken, Schnipsel,
Signaturhilfe, Umbenennen, Git-Blame, Sitzung je Projekt.

## Startbildschirm: IDE oder Studio (2026-11-10)

Idee des Nutzers, gleich nach dem Ende des Form-Designers: nach dem Vorspann
"schick zwischen den beiden Editoren entscheiden" -- klassisch in der IDE oder
modern im Studio. Abgestimmt: jedes Mal fragen, "Nicht mehr fragen" merkt
die Wahl (`startwahl` in der ide.json), zurueck ueber die Einstellungen oder
Umschalt beim Start; mit dem Studio schliesst sich die IDE. Gebaut in
`ide/teile/startwahl.dh`, gerufen in `ide/ide.dh` gleich nach dem Vorspann --
VOR den DIMs der Teile-Dateien, darum dort nur lokale Variablen. Gezeichnet
ohne gui (das gibt es dort noch nicht): Verlauf, zwei Lichter, Schriftzug,
zwei Karten mit gezeichneter Vorschau (Code mit Farben bzw. ein Fenster im
Entwurf, dessen Knopf an der Ausrichtungslinie einrastet), eigene grosse
Schrift (96 px) fuer die Ueberschriften. Kein Bildschirmfoto als Vorschau:
das veraltet und passt nicht zu jeder Groesse.

Fund dabei: ein Kind aus PROCESS_START endet mit dem Aufrufer (Drop von
`Prozess` beendet den Baum) -- das Studio waere mit der IDE gleich wieder
zu gewesen. Neu `PROCESS_START_DETACHED` (prozess.rs `losgeloest`: ohne
Leitungen, eigene Gruppe, unter Windows DETACHED_PROCESS, ein Faden wartet
gegen Zombies; Argumente wie PROCESS_START, gemeinsamer `befehl`). Tests:
`laufzeit/process_detached.dhtest` (losgeloest ueberlebt, PROCESS_START
nicht, Fehler; Gegenprobe: losgeloest ueber `Prozess` -> Fall 1 faellt) und
`ide/werkzeug_ide_startwahl.dhtest` (fuenf Faelle, vier Gegenproben je an
Abfrage, Argument, Einstellung und Merken). Pruefungen sehen den Bildschirm
nur mit `DH_IDE_STARTWAHL=1`, `DH_IDE_STUDIO_TROCKEN=1` startet das Studio
dort nicht wirklich.
