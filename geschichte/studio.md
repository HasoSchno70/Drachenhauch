# Drachenhauch Studio

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Alle Stufen, Datenbanken, Berichte, eigene Controls.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Drachenhauch Studio, Stufe 1 (2026-11-01)

Wunsch des Nutzers: der Form-Designer "wirkt billig", gewollt ist eine
eigenstaendige IDE im Stil von Xojo mit der Laufzeit dhrt. Entschieden:
eigene gui (nicht GTK), Fenster als Klasse mit Ereignis-Methoden, Projekt
als Ordner, Name "Drachenhauch Studio". Entwurf `docs/entwurf-studio.md`
(fuenf Stufen), Handbuch `docs/studio.md`. **Stufe 1 = ein Fenster
entwerfen**, `studio/` (studio.dh + modell/bibliothek/entwurf/inspektor/
oberflaeche.dh + studio.dhstil): Dock mit Navigator, Bibliothek (34
Bausteine in 8 Gruppen, Vorschaubilder aus IMAGE_*, Suche) und Inspektor
(Gruppen, Editor je Eigenschaft, Farben im Popover, Anker als vier
Kaestchen; ein Rueckgaengig-Schritt erst, wenn das Feld den Fokus abgibt).
**Das Modell ist das .dhform-JSON** (`doc`), das Entwurfsfenster
(`GUI_WINDOW_DESIGN`) nur die Ansicht -- nach jeder Aenderung
`GUI_FROM_JSON` neu; Undo = JSON-Text je Stand (`merken` uebergeht einen
gleichen Stand). Ausrichtungslinien (Kanten/Mitten der Nachbarn,
Fensterrand, 20 vom Rand, 12 neben dem Nachbarn; Alt = aus), acht Griffe,
Rahmenwahl, Anordnen, Zoom. **Drei Fallen:** (1) Strg+A als Menue-Kuerzel
nahm dem Textfeld mit Fokus die Taste -- es steht nur im Text des
Menuepunkts und wird in `tasten()` abgefragt; (2) die Lagen der
Bibliothekszeilen gelten erst NACH dem ersten GUI_UPDATE (die Stildatei
greift dort) -- das Protokoll misst in Bild 3; (3) eine Funktion `raster`
neben `CONST RASTER` (Namen schreibungsunabhaengig) -> `rasterWert`.
**Drei Laufzeit-Stuecke dabei:** `GUI_LISTBOX_ITEM_RECT(lb, i)` (Lage
einer Zeile auf dem Schirm, gefiltert = Nullen; fuer Pruefungen und
Ablegen aus der Liste); **`GUI_CHANGED` meldete einen Kippschalter in
jedem Bild seines Nachgleitens** (der Abdruck nahm `value`, das waehrend
der Animation wandert -- jetzt zaehlt nur `checked`); **die gewaehlte
Zeile eines Baums war im hellen Thema unlesbar** (dunkle Schrift auf der
Auswahl -- jetzt `lesbar_auf` wie bei der Liste). Tests
`tests/pruef/studio/werkzeug_studio.dhtest` (7, echte Klicks: Ablegen per Klick
und Zug, Einrasten samt rosa Linie im Bild, Griff, Anordnen,
Rueckgaengig/Wiederholen, Inspektor; Helfer `_hilfen/studiolauf.dh`
misst in einem ersten Lauf und baut daraus die Aufnahme), je ein Fall in
`gui_liste_komfort`, `messdaten_bausteine`, `gui_baum_bedienung`; fuenf von
sechs Verfaelschungen des Studios fallen je in ihrem Fall (das Uebergehen
gleicher Staende nicht -- seit dem Kippschalter-Fix entsteht keiner mehr).

**Stufe 2 = Projekt und Code** (selber Tag, Zweig `thema/studio-stufe2`):
`studio/projekt.dh` (Elemente app/fenster/modul/klasse, Methoden als
eigene Objekte, `codeZerlegen`/`codeSetzen$` mit Rundweg Zeichen fuer
Zeichen: Methoden einzeln, alles andere als "Eigenschaften" (`dekl`), der
Kommentar direkt ueber einer Methode gehoert ihr, Vorspann vor CLASS und
Nachspann hinter END CLASS woertlich; ein Modul nimmt seine Kommentarzeilen
oben als Vorspann, ausser sie stehen direkt ueber einer Methode),
`projekt.dhstudio` + je Element `.dh` (Fenster dazu `.dhform`) +
`.gitignore` mit `_studio/`; ein umbenanntes Element loescht beim Sichern
seine alten Dateien. `studio/code.dh`: die Mitte ist ein schliessbares
Reiterwerk (Layout je Fenster, Code je Methode, Eigenschaften je Element;
das Entwurfsfenster ist KEIN Kind, es wird gezeigt, solange sein Reiter
vorn ist -- `layoutVorn()` sperrt dafuer Maus, Tasten und Zeichnen des
Entwurfs). Das Layout, das vorn ist, steht in `doc`, die anderen als Text
im Element samt ihren Rueckgaengig-Staenden (`fensterAktivieren`).
Navigator aus Tabellen `navArt/navEl/navCtrl/navMeth`, Ereignisse unter
ihrem Control (Methode `<control>_<Ereignis>`, `ereignisseFuer(kind)`).
Doppelklick auf ein Control = Dialog "Ereignis hinzufuegen", dazu Methode,
Fenster, Modul, Klasse hinzufuegen; ein umbenanntes Control nimmt seine
Ereignis-Methoden und den Namen im Code (ganzes Wort) mit
(`controlUmbenannt`). Ist ein Code-Reiter vorn, sind Rueckgaengig,
Verdoppeln, Loeschen und Anordnen im Menue gesperrt -- ein gesperrter Punkt
nimmt dem Code-Feld die Taste nicht weg. Eine `.dhform` als Argument
bleibt ein einzelnes Fenster (Stufe-1-Verhalten), bis Code dazukommt.
**Drei Fallen:** (1) das letzte Argument von `GUI_TEXTAREA` ist der
PLATZHALTER -- der Rumpf stand nur grau da und ging beim Uebernehmen
verloren (Text per `GUI_SET_TEXT`); (2) was das Studio nach `GUI_DRAW`
zeichnet (Titelleiste, Griffe), lag ueber jedem Dialog -- offene Dialoge
kommen per `GUI_DRAW_WINDOW` noch einmal obenauf (`dialogeObenauf`);
(3) Python-Patches im Bash-Heredoc kamen zweimal nicht an -- Skripte per
Write. **Zwei Laufzeit-Stuecke:** `GUI_TABCONTROL_SET(tc, "schliessbar", 1)`
+ `GUI_TABCONTROL_CLOSED(tc)` (Kreuz je Kopf, Kopf 16 breiter,
`tc_kreuz` = eine Quelle fuer Zeichnen und Treffer, mittlere Maustaste,
geschlossen wird nichts; `.dhform` `tabctl.schliessbar`) und
`GUI_TREE_ITEM_RECT(tree, node)` (wie die Listenzeile, fuer echte Klicks
in den Navigator). Fehler am Code und Vervollstaendigung kommen mit Stufe
3 (brauchen die erzeugten Basisklassen). Tests
`tests/pruef/studio/werkzeug_studio2.dhtest` (8: drei ohne Fenster am
Projektmodell, fuenf mit echten Klicks -- Navigator oeffnet Methode und
getippter Code landet in der Datei, Doppelklick legt ein Ereignis an,
Kreuz schliesst, Fenster und Methode hinzufuegen, Umbenennen im Inspektor),
vier Faelle in `gui_reiterwerk.dhtest`; acht Verfaelschungen des Studios
fallen je genau in ihrem Fall.

**Stufe 3 = Starten und Bauen** (selber Tag, Zweig `thema/studio-stufe3`):
`studio/laufzeit.dh` (wird beim Starten als studio_laufzeit.dh nach _studio/ kopiert:
`StudioApp`, `StudioFenster` mit `Zeigen`/`Zumachen`/`Fenster()` und den
leeren Haken `studioAufbauen`/`studioTakt`/`studioZeichnen`, `studioLauf` =
die Bildschleife: Kreuz schliesst das Startfenster, GUI_WINDOW_CLOSED,
Groesse, geschlossene Fenster raus, `GUI_DRAW(FALSE)` + Zeichenflaechen +
`GUI_DRAW_WINDOW` fuer Nebenfenster + `GUI_DRAW_TOP`). `studio/erzeugen.dh`:
je Fenster `<F>_Basis EXTENDS StudioFenster` -- **gebaut aus dem Layout als
JSON** (`!"..."`-Stuecke zu 96 Zeichen, `on_*` herausgenommen,
`GUI_FROM_JSON`, Felder ueber `GUI_WINDOW_WIDGET`), Verdrahtung Klick/
Geaendert/Enter/Fokus/Maus-rein-raus ueber `GUI_ON_*` mit gebundenen
Methoden, Doppelklick/Maus/Zeichnen in `studioTakt`/`studioZeichnen`; alle
Rueckrufe sind parameterlos. `main.dh` bindet Laufzeit, Basisklassen und
`projekt/<Element>.dh` ein -- **der Code kommt im Editorstand nach
`_studio/projekt/`**, nicht aus dem Projektordner: nur so passt die
**Zeilenkarte** aus `codeSetzen$` (`karteMeth/karteZeile/karteEinzug`;
`KARTE_DEKL`, `KARTE_SONST`; das END einer Methode = Zeile hinter dem Rumpf,
dort meldet der Uebersetzer ein offenes IF) auf eine Meldung -- die erste
Fassung band die Dateien des Projektordners ein und lag eine Zeile daneben, weil die Datei von
Hand anders geschrieben war. Pruefen: `ganzerText$` setzt dieselben Teile im
Speicher zusammen (`gz*` je Zeile), `CODE_CHECK$` mit Projektordner, 600 ms
nach der letzten Aenderung und vor jedem Start; ein Fehler haelt an.
Befunde in der Problemliste (Klick oeffnet die Methode an der Zeile), im
Code-Feld als Welle/Marke/Hinweis. Starten = `dhrt run` (`DH_STUDIO_LAUF_BILDER`
fuer Pruefungen), Laufzeitfehler `x.dh:N` ueber die Karte an die Methode;
Warnungen des Uebersetzers fallen aus der Ausgabe (stehen unter Probleme).
Bauen = `dhrt --export main.dh bau/<name>(.exe)`. **Fund in der Laufzeit:**
der Export kopierte jede per IMPORT eingebundene `.dh` als "referenzierte
Datei" neben die Exe -- der ganze Quelltext lag mit im Bau; jetzt
uebergeht `bundle_referenced_assets` IMPORT-Zeilen (Rust-Test
`export_kopiert_importierte_quellen_nicht`). Tests
`tests/pruef/studio/werkzeug_studio3.dhtest` (5: Zeilenkarte ohne Fenster, F5
erzeugt + `--check` leer + ein Klick im erzeugten Programm ruft das
Ereignis, Laufzeitfehler an Methode/Zeile trotz anders geschriebener Datei,
Uebersetzungsfehler haelt an + Klick in die Problemliste springt "am Ende",
Strg+B baut eine Exe ohne Quelltext daneben und sie laeuft); sechs
Verfaelschungen fallen je in ihrem Fall. Offen: Vervollstaendigung im
Code-Feld, Ziehen/Rad/Taste/Timer (Stufe 4).

**Stufe 4 = was ein Programm braucht** (selber Tag, Zweig
`thema/studio-stufe4`, auf Stufe 3). **Die Basisklasse fragt alle Ereignisse
je Bild ab** (`studioTakt`: `GUI_CLICKED`/`GUI_CHANGED`/`GUI_ENTERED`, Fokus
ueber `studioFokusAlt/Jetzt` aus `GUI_FOCUSED`, Maus rein/raus ueber ein Feld
`studioUeber_<name>`), statt `GUI_ON_*` zu setzen -- ein Rueckruf laeuft IN
`GUI_UPDATE`, und eine Methode darin koennte nicht auf einen Dialog warten.
Die Bildschleife ist jetzt `studioBild()` (FALSE = Programm endet; ueber eine
KOPIE der Fensterliste, weil ein Ereignis Fenster oeffnen und schliessen
kann), und `Frage/Meldung/Eingabe` (`studioAntwort`: Dialog in
`studioDialoge`, damit er ueber Zeichenflaechen obenauf kommt) und
`StudioFenster.ZeigenModal()` drehen selbst Bilder, bis die Antwort da ist;
endet das Programm dabei, liefert `Frage` 0. **Timer**: Layout-Liste
`timer` (name, intervall, aktiv, wiederholen), in der Basis ein Feld
`StudioTimer` mit `studioFaellig()` je Bild (MILLIS, nicht TIMER_EVERY);
im Studio eine Leiste unter dem Entwurf (`trayRect`/`trayBei`, `trayWahl`
neben `auswahl`; `auswahlSetzen` setzt es zurueck; der Inspektor fragt in
`giltFuer`/`pfadFuer$` zuerst `trayWahl`), Baustein "Timer" (Gruppe
Unsichtbar), Entf loescht. **Menues**: `studio/menue.dh`, Editor auf einer
Kopie `{"menus": ...}` (Pfade je Baumknoten; Einfuegen/Tauschen bauen die
Liste neu, das json-Modul haengt nur an), Name `mi<Text>` mit
`studio_auto`, bis man ihn aendert; OK = ein Rueckgaengig-Schritt, danach
`controlUmbenannt` je Umbenennung. Die Basis nimmt `timer` und `menus` aus
dem Layout-JSON und baut Menues per `GUI_MENU/GUI_SUBMENU/GUI_MENU_ITEM`
(benannte in ihr Feld, `GUI_CLICKED` = `<name>_Aktion`). `controlsVon`
haengt Timer und benannte Menueeintraege hinter die Widgets
(`ctrlWidgets` = wo sie beginnen) -- so kennen Navigator und
Ereignis-Dialog sie ohne eigenen Weg. **Bilder**: `bilder/` im Projekt,
Projekt -> Bild hinzufuegen (`DH_STUDIO_DATEI` fuer Pruefungen),
Widget-Schluessel `studio_bild`, Auswahl im Inspektor (Liste je Fuellen neu),
Vorschau im Entwurf (`entwurfBilder`), beim Erzeugen nach
`_studio/assets/bilder/` (Konvention von `dhrt --export`), `Bild(name)` in
der Laufzeit. **Debugger**: `studio/debug.dh`, Haltepunkte je
(Methode, Rumpfzeile) per Klick in die Nummernspalte oder F9, F7 =
`dhrt debug`, erster Halt -> `set-breakpoints` (Datei/Zeile ueber die
Zeilenkarte zurueck) + continue, ein Halt -> Methode und Zeile ueber
dieselbe Karte, Leiste "Debugger" mit Variablen (Self eine Ebene tief) und
Stapel, F10/F11/Umschalt+F11, Umschalt+F5 beendet. **Falle:** ein
gesperrter Menuepunkt nimmt auch sein Kuerzel nicht an -- `debugKnoepfe`
gibt Schritt/Hinein/Heraus beim Halt frei, sonst ging F10 ins Leere (vom
Test gefunden). Die Menueleiste verschiebt den Inhalt um 26 Punkte (auch im
erzeugten Programm). Tests `tests/pruef/studio/werkzeug_studio4.dhtest` (5: Timer
aus der Bibliothek ueber Suche + Doppelklick, Timer waehlen/Inspektor/Entf,
Menue-Editor, das erzeugte Programm mit Frage + Enter, modalem Fenster,
Timer, Strg+H und Bild-Pixel, Debugger mit Halt/Schritt/Weiter); zehn
Verfaelschungen fallen je in ihrem Fall. Bild hinzufuegen selbst (Dialog)
ist nur von Hand geprueft. **Nachtrag (selber Tag):** Ereignisse mit
Parametern (`ereignisParameter$`, der Ereignis-Dialog setzt sie in die
Kopfzeile): `Ziehen(x, y)` und `Rad(schritte)` der Zeichenflaeche (Feld
`studioZug_<name>`, ab dem Druck auf die Flaeche), `Taste(taste)` des
obersten Fensters (`StudioFenster.Taste`, in `studioBild` aus
`KEY_ANY_HIT`, nicht wenn ein Eingabefeld den Fokus hat). **Falle beim
Pruefen:** `KEY_ANY_HIT` blendet aus, was eine Wiedergabe einspeist -- der
Fall dafuer schickt echte Fensternachrichten
(`werkzeug_studio4_taste.dhtest`, seriell, nur Windows). Vervollstaendigung
im Code-Feld (`studio/vervoll.dh`): Strg+Leertaste oder hinter `.` (dann
nur bis 100 Eintraege), gefragt wird `CODE_COMPLETE` ueber `ganzerText$`,
die Zeile der Marke findet die gz-Karte -- so kennt sie Felder der
Basisklasse und die Laufzeit. **Falle:** `GUI_CHANGED` muss jedes Bild
gefragt werden, der erste Aufruf meldet nichts. Bild hinzufuegen hat
Strg+Umschalt+B und ist mit `DH_STUDIO_DATEI` geprueft. Vier Faelle mehr in
`werkzeug_studio4.dhtest` (8) plus die Taste-Sammlung; fuenf Verfaelschungen
fallen je in ihrem Fall.

**Stufe 5, erster Teil = erreichbar** (Zweig `thema/studio-stufe5`, auf
Stufe 4): **Installer** `Drachenhauch-IDE.iss` nimmt `studio\*.dh` und
`*.dhstil` nach `{app}\studio`, Verknuepfung "Drachenhauch Studio"
(eigene AppUserModelID `Drachenhauch.Studio` -- der Installer-Fall zaehlt
jetzt 11), Task `assocstudio` fuer `.dhstudio` -> `dhrt run studio.dh --
"%1"`. **Pakete**: `paket.dh` kopiert `studio/` (nur wenn es die Wurzel
hat -- die Faelle mit einer Attrappen-Wurzel haben keins), der Starter
`drachenhauch --studio [projekt]` (und eine `.dhstudio` als Argument)
startet das Studio, `install.sh` legt `drachenhauch-studio.desktop` und
den MIME-Typ `application/x-drachenhauch-studio` an und raeumt beides ab,
`Info.plist` meldet `.dhstudio` an (macOS gibt die Datei an die App, also
an die IDE, die sie ans Studio weiterreicht). **IDE**: Werkzeuge ->
Drachenhauch Studio (`studioStarten`, studio/studio.dh neben ide/; ohne
Argument mit dem Projektordner, wenn dort `projekt.dhstudio` liegt),
`dateiOeffnen` gibt eine `.dhstudio` ans Studio, `BAUM_MUSTER` zeigt sie.
**Form-Designer 197**: Datei -> Im Drachenhauch Studio oeffnen
(Strg+Umschalt+O, `studioPfad$`: DH_IDE_WURZEL, neben der Laufzeit,
macOS-Resources, `../studio`), ohne Datei ein Hinweis in der Statuszeile.
Tests `tests/pruef/studio/werkzeug_studio5.dhtest` (3; das gestartete Studio
erbt `DH_STUDIO_LOG`, der Fall wartet auf "projekt geladen" und beendet
es), Zeilen in `werkzeug_paket.dhtest` (Linux-Ordner, macOS-Bundle,
install.sh, `--studio` mit der echten Laufzeit). Der Installer baut mit
ISCC (lokal nachgesehen).

**Stufe 5, Teil 4 = das Beispielprojekt** `examples/studio/rechnungen/`
(nach dem Piloten 196): App, Module Datenbank (SQLite, Beispieldaten beim
ersten Start, `naechsteNummer$`) und Ausgabe (CSV, `datumDeutsch$`),
Klasse Rechnung (laden/rechnen/sichern in einer Transaktion/PDF; Steuer je
Position gerundet), vier Fenster: Hauptfenster (Tabelle mit Filterzeile,
Menue mit Kuerzeln, Doppelklick oeffnet), RechnungFenster (modal),
Kunden-/ArtikelFenster (Datenbindung `GUI_FORM_LOAD/SAVE`, Regeln, Geldfeld,
Klappliste mit Werten, Rueckfrage bei Ungesichertem). Die Layouts sind aus
echten gui-Aufrufen ueber `GUI_TO_JSON` entstanden, die Namen danach in der
JSON gesetzt (GUI_SET_NAME schreibt klein); alle Dateien stehen so da, wie
`projektSchreiben` sie schreibt. Dazu im Studio: **Tabelle hat
"Doppelklick"** (`ereignisseFuer`), und in der Laufzeit eine
**`--check`-Warnung**, wenn eine Methode eine geerbte gleichen Namens mit
anderer Parameterzahl ueberschreibt (gefunden am Beispiel: `SUB
zeigen(kid)` verdeckte `StudioFenster.Zeigen()`; `Init` ausgenommen, Zeile
= eine vor der ersten Anweisung des Rumpfs). Die Repo-Durchlaeufe in
`dhrt_check.dhtest` lassen Ordner mit `projekt.dhstudio` aus. Tests
`tests/pruef/studio/werkzeug_studio_beispiel.dhtest` (2: Rundweg durch das Modell;
F5 im Studio + `--check` + eine Probe, die statt `studioLauf` die Teile
direkt aufruft -- Daten, Rundung, UNIQUE-Rollback, Liste, PDF, CSV,
Bindung, Regeln, RechnungFenster), ein Fall in
`compiler_warnungen.dhtest`.

**Englische Oberflaeche** (2026-11-03): `studio/sprache.dh` (`tr$`,
`trf$("... {0} ...", [werte])` -- erst uebersetzen, dann einsetzen --,
`kz$`), Tafel `studio/sprache/en.txt` (TSV wie die IDE), englische Vorlagen
unter `studio/vorlagen/en` (dieselben Projekte mit englischen Namen, Texten,
Kommentaren und Daten; Layouts aus den deutschen erzeugt, ein Fall in
`werkzeug_studio_vorlagen.dhtest` vergleicht den Aufbau -- wer eine Vorlage
aendert, zieht die andere mit). Sprache: `DH_STUDIO_SPRACHE` > `studio.json` (Ansicht ->
English interface, gilt beim naechsten Start) > `ide.json` daneben (nur
ohne `DH_STUDIO_KONFIG`) > `DHRT_LANG` > de; englisch setzt es
`DHRT_LANG=en` (CODE_CHECK$, gestartete Programme). **Schluessel bleiben
deutsch, uebersetzt wird beim ANZEIGEN:** Bausteinname und -gruppe (Sinnbild,
`bausteinName$`), Gruppen und Feldnamen des Inspektors, Werte einer "wahl"
(Woerter der Laufzeit) und **die Namen der Ereignisse** -- sie sind Teil der
Methodennamen (`Knopf1_Klick`), ein Projekt muss in beiden Sprachen gleich
bleiben. Was ins Fenster kommt (Grundname -> `Button1`, Text, Eintraege,
`MainWindow`), wird beim Anlegen uebersetzt. Zwei deutsche Texte mussten
dafuer eindeutig werden ("Wiederholt sich" fuer den Timer, "Anhalten" am
Debugger -- sonst hiessen Redo/Repeat bzw. Quit/Stop gleich). Der Testhelfer
`studiolauf.dh` setzt `DH_STUDIO_KONFIG` in den Fallordner ("-" = keine).
`DH_STUDIO_SPRACHPROBE=1` oeffnet jeden Dialog und protokolliert Texte und
fehlende Uebersetzungen. Tests `tests/pruef/studio/werkzeug_studio_englisch.dhtest`
(5: Tafel gegen den Quelltext, Probe englisch/deutsch, Umschalten, Herkunft
der Sprache); acht Verfaelschungen fallen je in ihrem Fall.

**Eigene Controls, Stufe 1** (2026-11-03, Vorbild Xojos ContainerControl):
Elementart `control` (`hatLayout(e)` = fenster ODER control -- alles, was
Entwurf, .dhform und Basisklasse betrifft, fragt dort; "fenster" allein
meint Startfenster und Menues). Bibliothek: je Control ein Baustein der Art
`studio_control` in der Gruppe "Projekt" (`projektBausteineNachziehen`,
hinter den eingebauten -- `bibEingebaut`; fehlt, solange ein Control selbst
im Entwurf steht). Im Layout des Fensters ist eine Instanz
`{"kind":"studio_control","studio_control":"Zaehler",...}`. **Ausgeklappt**
(`ausgeklappt` in modell.dh, fuer Entwurf UND Erzeugen): die Instanz wird
ein leerer Layout-Behaelter an ihrem Index (Anker, Gewicht, Layout-Platz
bleiben; zeichnet nichts, ist Luft fuer Klicks), die Teile kommen HINTER
alle Controls des Fensters (Nummern bis anzahl() bleiben die des Modells),
Lage verschoben, Anker entfernt und gemerkt, Layout-Kinder umnummeriert.
Der Entwurf haelt eine eigene Kopie der Zuordnung (`vk*` -- das Pruefen
klappt beim Erzeugen auch aus und ueberschriebe `ak*`), `vorschauLegen` je
Bild vor dem Zeichnen, `indexVonHandle` macht aus einem Teil die Instanz.
Erzeugt: `<C>_Basis EXTENDS StudioControl` mit `studioEinbauen(win,
behaelter, erster)`; das Fenster legt je Instanz ein Objekt an, ruft je Bild
`studioLegen` (Teile nach Ankern, `studioAchse` = `teilLage`) und
`studioTakt`, `Geoeffnet` der Controls vor dem des Fensters, `Zumachen`
setzt `studioOffen` der Controls zurueck. Reihenfolge: Controls vor App und
Fenstern. Tests `tests/pruef/studio/werkzeug_studio_controls.dhtest` (4: zwei
Instanzen im erzeugten Programm, eine in einer Zeile gestreckt; Vorschau
nach Ankern; Ablegen aus der Bibliothek; Klick auf einen Teil waehlt die
Instanz); sieben Verfaelschungen fallen je in ihrem Fall.
**Stufe 2 (selber Tag): eigene Ereignisse und Eigenschaften.** Alles Text in
der .dhform, damit es in die Felder des Inspektors und in Rueckgaengig passt:
am Control `studio_ereignisse` ("Erreicht; Geleert") und
`studio_eigenschaften` ("Ziel AS INTEGER = 3; Vorsatz = \"Anzahl: \"",
STRING/INTEGER/FLOAT/BOOLEAN, ohne AS STRING, Anfuehrungszeichen halten
Randleerzeichen), an der Instanz `studio_werte` ("Ziel = 2"). Gelesen in
modell.dh (`controlEreignisse`, `eigenschaftenLesen` -> `eigName/Typ/Wert`,
`werteLesen`, `wertLiteral$` = Literal oder "" wenn es nicht passt --
Unpassendes faellt still weg). Die Arten in `ctrlArten`/`arten` heissen fuer
eine Instanz `studio_control:<Name>`, `ereignisseFuer` loest daraus die
Ereignisse des Controls. Erzeugt: je Eigenschaft ein Feld, die Vorgabe am
Ende von `studioEinbauen`, die Werte der Instanz danach im Fenster;
`Self.Ausloesen` sammelt in `studioMeldungen`, das Fenster fragt nach dem
Takt des Controls `studioGemeldet` und leert am Ende seines Takts.
**Fund:** eine Instanz ist im Entwurf ein leerer Behaelter, Luft fuer
Klicks -- ein Klick auf eine Stelle ohne Teil traf nichts (`instanzBei` in
`trefferMitLayouts`). Tests zwei Faelle mehr in
`werkzeug_studio_controls.dhtest` (6); sechs Verfaelschungen fallen je in
ihrem Fall.
**Stufe 3 (selber Tag): Controls in Controls, Umbenennen, Loeschen.**
Ausgeklappt wird tiefenweise und zusammenhaengend (`instanzAusklappen`
rekursiv, `akWurzel` = die Fenster-Instanz eines Platzes, `akName` =
"zwei1_z2"): ein Block = die eigenen Teile, dahinter die Bloecke der inneren
Instanzen; seine Groesse ist fest (`controlFuss`). Darum gilt ein Control,
das sich selbst enthaelt (auch ueber andere, `controlEnthaelt`), UEBERALL als
nicht brauchbar (0 Teile, leerer Platz, Entwurf "{0} enthaelt sich selbst")
-- nur im Zusammenhang abzuschneiden machte die festen Versaetze
widerspruechlich. Erzeugt: ein Control baut seine inneren Instanzen in
`studioEinbauen` mit `"erster + versatz"` (`instanzenBauen`, gemeinsam mit
dem Fenster), Liste `studioKinder`; `studioGeoeffnet` (Kinder zuerst),
`studioSchliessen` und `Sichtbar` gehen rekursiv. Eine Reihenfolge der
Controls in main.dh braucht es nicht (Klassen verweisen vorwaerts -- die
Gegenprobe blieb gruen, die Ordnung ist wieder draussen). Bibliothek: im
Control weder es selbst noch eins, das es enthaelt. Umbenennen
(`instanzenUmbenennen` in code.dh) schreibt `studio_control` in den anderen
Layouts UND in deren `verlauf`; Loeschen verweigert mit Statuszeile, solange
`instanzenVon` etwas findet. Pruefhilfen: Protokoll `projektbausteine`,
`DH_STUDIO_LOESCHEN`. Tests fuenf Faelle mehr (11); sieben Verfaelschungen
fallen je in ihrem Fall.

**Datenbanken im Projekt, Stufe 1** (2026-11-03, Wunsch des Nutzers nach dem
Vergleich mit Xojo): Elementart `datenbank` (`hatCode(e)` = FALSE -- jede
Stelle, die Code schreibt oder zerlegt, fragt dort), Aufbau in
`<Name>.dhdb` (`e.schema`, JSON: `datei` + `tabellen[].spalten[]` mit `typ`
text/ganz/zahl/geld/datum/wahr, `pflicht`, `eindeutig`, `vorgabe`), `id
INTEGER PRIMARY KEY` steht nie im Schema. `studio/datenbank.dh` (ohne gui,
ohne Fenster pruefbar): jede Aenderung geht durch `dbAusfuehren$` = EINE
Transaktion auf die Entwicklungsdatei im Projektordner (Befehle davor fuer
RENAME/DROP, dann `dbAbgleichenMit`); bei einem Fehler bleiben Datei UND
Schema. Der Abgleich legt Fehlendes an, baut eine Tabelle neu auf, wenn
sich Art/Pflicht/Vorgabe einer Spalte geaendert haben (verglichen ueber
PRAGMA table_info samt dflt_value-Text; COALESCE in den Leerwert fuer
Pflichtspalten), laesst fremde Spalten und Tabellen stehen. Eindeutig ist
IMMER ein eigener Index `studio_u_<tabelle>_<spalte>` -- ALTER TABLE ADD
COLUMN kann kein UNIQUE. Ohne Ordner (oder solange er nicht existiert)
liefert `dbPfad$` "" und nur das Schema aendert sich; `sichern` und
`projektOeffnen` gleichen danach ab. `dbMeldung$` macht aus "UNIQUE
constraint failed" einen Satz. Erzeugt: studio_datenbanken.dh in _studio/
(vor allen Elementen importiert, auch in `ganzerText$`): je Datenbank `DIM
<Name> AS DB_CONN` als oberste Anweisung (offen, bevor irgendetwas laeuft),
CREATE TABLE IF NOT EXISTS, `studioDbSpalte` je Spalte, Indexe; der Pfad
ueber `DH_STUDIO_DATEN` (setzt `vorbereiten`), sonst neben dem Programm.
Bauen kopiert die Datei nach bau/, wenn dort keine liegt. Reiter
`studio/dbeditor.dh` (Reiter-Art `datenbank`, `Reiter.db` = `DbAnsicht`,
`r.feld` = -1): Tabellenliste, Spalten-Gitter (Art als Auswahlspalte,
Pflicht/Eindeutig als Hakenzellen) und Daten-Gitter (je Tabelle neu
angelegt, wahr als Auswahl); Aenderungen erkennt der Vergleich mit
`spStand`/`daStand`, nur wenn keine Zelle bearbeitet wird. Feste
Spaltenbreiten, damit `dbLagen` die Zellmitten protokollieren kann (Kopf
22, Zeile 20). Pruefhilfe `DH_STUDIO_OEFFNEN=Element[.Tabelle]`. **Fund in
der Laufzeit:** ein `GUI_TABCONTROL` schluckte JEDEN Klick auf die Kinder
seiner Seiten (handle_press nimmt den ersten Treffer, das Reiterwerk liegt
vor ihnen) -- kein Knopf auf einer Seite meldete GUI_CLICKED, kein Feld
bekam per Klick den Fokus; der Kommentar "unter den Koepfen gehoert die
Flaeche den Kindern" stand im Press-Zweig, der erst nach dem Treffer kam.
Jetzt in `luft_fuer_kinder`. Tests
`tests/pruef/studio/werkzeug_studio_datenbank.dhtest` (8: Kern, Neuaufbau mit
fremden Spalten, Werte je Art, erzeugtes Programm an aelterer Datei,
Projekt lesen/schreiben, Klicks ins Daten- und Spalten-Gitter, F5 + Bauen;
elf Verfaelschungen fallen je in ihrem Fall), ein Fall in
`gui_reiterwerk.dhtest` (faellt mit 2026.41).
**Stufe 2 (selber Tag): binden.** Fenster-Schluessel `studio_tabelle`
("Datenbank.tabelle"), Feld `bind` (GUI_BIND), Liste/Tabelle/Klappliste
`studio_quelle` + `studio_spalten` (";", leer = bei einer Tabelle alle,
sonst die erste); aufgeloest ueber `dbRef` (setzt `dbRefEl`/`dbRefT`). Die
Basis eines Fensters bekommt aus `dbBasisFelder/Aufbau/Methoden/Takt`
(datenbank.dh, von basisCode$ gerufen) `DatensatzId`, `Laden(id)`,
`Sichern()` (GUI_VALIDATE, dann GUI_FORM_SAVE mit -1 fuer neu), `Neu()`
(GUI_FORM_CLEAR + GUI_FORM_SET mit den Vorgaben der gebundenen Spalten +
GUI_FORM_CLEAN, am Ende von studioAufbauen gerufen), `Loeschen()`,
`Ungesichert()`, `Neuladen()` (je Liste ein SELECT, ids in
`studioIds_<name>`, Werte ueber `studioDbText$` aus studio_datenbanken.dh)
und `IdVon(w)`. Zeigt eine Liste die Tabelle des Fensters, laedt
`GUI_CHANGED` darauf den Datensatz (im Takt), und Neuladen waehlt die Zeile
von DatensatzId wieder. Navigator: Spalten unter jeder Tabelle (aufgeklappt);
`navVorUpdate/navNachUpdate` (dbeditor.dh) ziehen eine Spalte oder Tabelle
ins Fenster (`felderAblegen` -> `feldAnlegen`: Beschriftung `<spalte>Text`
+ Feld `<spalte>Feld`, Geld `zahlen` 3, Ja/Nein als Kaestchen), Doppelklick
auf eine Spalte legt sie unter das letzte Control; ein Klick auf Tabelle
oder Spalte oeffnet erst beim LOSLASSEN den Reiter (sonst waere das Layout
weg, bevor der Zug beginnt), mit Tasten gewaehlt sofort. Inspektor-Gruppe
Daten mit Auswahlen aus dem Projekt (`datenWahlFuellen`, Eintrag 0 =
"(keine)" = Schluessel weg); eine Datenquelle setzt den Tabellenkopf im
Entwurf (`quellKopfSetzen`). Tests `tests/pruef/studio/werkzeug_studio_datenbank2.dhtest`
(4: Methoden + Listen + ein Klick, den das erzeugte Programm selbst per
Aufnahme macht; Tabelle ziehen + F5; Doppelklick neben einer fremden
Tabelle; Auswahlen im Inspektor per Tastatur); zwoelf Verfaelschungen fallen
je in ihrem Fall.
**Stufe 3 (selber Tag): Verweise, Bedingung/Sortierung/Suche, Abfragen.**
Spalte `verweis` (Zieltabelle; `dbSpalteVerweis$` macht die Spalte `ganz`),
im Spalten-Gitter die sechste Spalte "Verweis auf"; `feldAnlegen` macht aus
ihr eine Klappliste mit `studio_quelle` = Ziel und `studio_spalten` =
`dbAnzeigeSpalte$` (erste Textspalte). Schema `abfragen[]` (name, sql),
Namen mit den Tabellen geteilt (`dbNameBelegt`); `dbAbfrageTesten$` laeuft
das SQL als `SELECT * FROM (sql) AS studio_q LIMIT n` (nur SELECT/WITH,
eine Anweisung) und schneidet die Stelle im Huell-SQL aus der Meldung
(`dbOhneHuelle$`); `dbAbfrageSetzen$` sichert nur, was geht. Eine
Datenquelle ist Tabelle ODER Abfrage (`dbQuelle` setzt `dbRefT`/`dbRefA`,
`dbRef` nur Tabellen); im Programm liest `studioQuelle(con, quelle,
spalten, filter, sort, desc, such)` (id oder `0 AS id` vorn, LIKE ueber die
gezeigten Spalten, ORDER BY), `studioZeile` zeigt; Klapplisten tragen die id
als Eintragswert (`GUI_LISTBOX_SET_DATA`) und behalten ihre Auswahl beim
Neuladen; der Takt laedt bei `GUI_CHANGED(suchfeld)` neu. `Abfrage(name)`
mit SELECT CASE ueber "name" und "datenbank.name", Gross/klein egal. Reiter:
Liste links = Tabellen, Kopf "Abfragen", Abfragen (`eintraege`/`eintragAbf`);
eine Abfrage zeigt SQL-Feld, [Ausfuehren] und ein Ergebnis-Gitter
(`GUI_TABLE_CLEAR` + `GUI_TABLE_HEADERS` -- `GUI_TABLE` mit Kopf ohne Zellen
ist ein Fehler); uebernommen wird beim Ausfuehren oder wenn das Feld den
Fokus abgibt. Navigator: Abfragen als Knoten `abfrage` (Pfad
`Daten.?name`), Klick oeffnet, Ziehen ins Fenster legt eine Tabelle mit
Datenquelle an (`abfrageAblegen`). Inspektor: `studio_sortieren` (Spalten
der Quelle), `studio_absteigend`, `studio_filter` (an der Datei geprueft,
`filterPruefen` -> GUI_SET_ERROR), `studio_suchfeld` (Textfelder des
Fensters). **Testfalle:** Knoten unter dem sichtbaren Navigator haben eine
Lage im Protokoll, nehmen aber keinen Klick an -- Projekte in Tests klein
halten. Tests `tests/pruef/studio/werkzeug_studio_datenbank3.dhtest` (7); 16
Verfaelschungen fallen je in ihrem Fall (Sortieren erst mit Testdaten, bei
denen Name und id verschieden ordnen). **Umbenennen im gebauten Programm:**
`dbUmbenennungMerken` haengt je Umbenennung `{art, tabelle, alt, neu}` an
`umbenannt` (nicht bei reinem Gross/klein); das erzeugte
studio_datenbanken.dh ruft nach DB_OPEN der Reihe nach `studioDbTabUm`/
`studioDbSpUm` (nur wenn alt da und neu nicht). Indexe heissen nach Tabelle
und Spalte: beim Spaltenumbenennen faellt `studio_u_<t>_<alt>` vorher weg,
beim Tabellenumbenennen jeder `studio_u_`-Index, der nicht mit dem neuen
Tabellennamen beginnt -- `CREATE UNIQUE INDEX IF NOT EXISTS` legt sie neu
an. **Inspektor:** ist ein Reiter eines anderen Elements vorn
(`inspFremdesElement`), zeigt er nur Name und Art; `reiterPruefen` fuellt
ihn bei jedem Reiterwechsel neu. Zwei Faelle in
`werkzeug_studio_datenbank.dhtest`, sechs Verfaelschungen fallen.
**Stufe 4 (2026-11-03): Bildspalten und Verbunde ueber Verweise.** Neu in
der Laufzeit `DB_GET_BUFFER(r, idx)` (BLOB wie gespeichert, Text als UTF-8,
NULL leer, Zahl = Fehler) -- ein BLOB kam vorher nur als Text heraus. Art
`bild` = SQL `BLOB` (Pflicht-Leerwert `X''`, keine Vorgabe); `dbIstBild`
prueft die Kennung (PNG/JPG/BMP/GIF) ohne Fenster. Daten-Gitter: Bildspalte
schreibgeschuetzt, Zelle "Bild (n KB)", Knopf **Bild …** (`dbBildWaehlen`,
`DH_STUDIO_DATEI` fuer Pruefungen), Entf = NULL. Ein Bild-Control haengt an
`studio_bildspalte`, NICHT an `bind` -- GUI_FORM_SAVE schriebe ihm einen
leeren Wert; Inspektor-Zeile "Bildspalte" (`dbSpaltenBild`, die Bindung
bietet keine Bildspalten mehr an). Erzeugt (`dbBildFelder`): je Control
`studioBild_/studioBildNeu_/studioBildGeaendert_`, Laden zeigt
(`studioDbBildLesen`), `BildSetzen(w, pfad)`/`BildLeeren(w)`, Sichern
schreibt, Ungesichert zaehlt mit; eine Tabelle zeigt Bildspalten per
`GUI_TABLE_CELL_KIND "bild"` (Bilder in `studioBilder_<liste>`, beim
Neuladen freigegeben). **Fund:** ein Fenster, das nur ein Bild bindet,
brach mit "kein Widget gebunden" ab -- ohne `bind` gehen Laden/Sichern/Neu
jetzt ueber `studioDbZeileDa`/`studioDbZeileNeu` statt GUI_FORM_*.
**Verbunde:** in `studio_spalten` auch `verweis.spalte`; `dbVerweisJoin$`
legt `(SELECT q.*, j0."name" AS "kunde.name" ... LEFT JOIN ...)` um die
Tabelle, wenn Spalten, Sortierung oder Bedingung `verweis.` nennen
(`dbQuellSql$` mit `bedarf`); `dbListSpaltenMit` (Schreibweise des Schemas),
`dbQuellSpaltenMit` fuer Sortieren und Entwurfskopf. **SQLite-Falle:** ein
unbekannter Name in doppelten Anfuehrungszeichen ist dort still ein Text --
`dbFilterFehler$` prueft solche Namen jetzt gegen die Spalten der Quelle
(`dbFilterNamePruefen$`). Tests `werkzeug_studio_datenbank4.dhtest` (3; das
Bild in der Zelle am Bildschirmfoto) und `werkzeug_studio_verbunde.dhtest`
(3); 15 Verfaelschungen fallen, eine ist gleichwertig (Bedingungspruefung
ohne Verbund -- wegen der Falle oben).
**Tempo des Designers (selber Tag):** gemessen mit `DH_STUDIO_TEMPO=1`
(je Bild `tempo <ms>` ins Protokoll, Rechenzeit vor FLIP) an einem Fenster
mit 300 Controls, Median ruhig/ziehen/Pfeile 1,7/6,3/2,4 -> 1,0/1,2/1,6 ms.
Die Einrast-Ziele sammelt `zieleSammeln` je Zug und Achse einmal (Schluessel
`zielZug` + Groesse + Auswahl; gesammelt wird nur bei gedrueckter Taste, und
dabei bewegt sich nur die Auswahl, die selbst kein Ziel ist), `feldText$`/
`feldZahl` fragen nur `JSON_TYPE` (liefert "missing") + Wert, und
`entwurfZeichnen` liest die Arten einmal je Bild. **Messfalle:** `dhrt
profile` schaltet den Maschinencode ab -- reine Zahlenschleifen (`einrasten`)
sehen dort teuer aus und kosten im echten Lauf fast nichts; gemessen wird
darum ohne Profiler. Test-only: `protokoll` sammelt und schreibt je Bild
einmal (`protokollSchreiben` vor FLIP und am Ende) -- vorher oeffnete jede
der Tausenden Lage-Zeilen die Datei.

**Berichte, Stufe 1 (2026-11-03, Entwurf `docs/entwurf-berichte.md`):**
Element `bericht` (Beschreibung in `e.schema`, Datei `<Name>.dhreport`,
`hatCode` FALSE). `studio/berichte_laufzeit.dh` legt aus und setzt (Prefix
`studioB`, kein Fenster): erst alle Zeilen lesen (`studioBLesen`), dann
auslegen (`studioBAuslegen`: Baender, Gruppen ueber Wertwechsel, Umbruch mit
Seitenfuss/Seitenkopf), dann zeichnen -- erst danach steht m fuer "Seite n
von m". `:parameter` setzt `studioBParameter$` als SQL-Literal ein
(ausserhalb von Text, Gross/klein egal, fehlend = Fehler). `studio/berichte.dh`
= Modell + erzeugter Code (`berichtSql$` mit `dbQuellSql$` und `bedarf`, also
`kunde.name` ueber Verweise; je Bericht `CLASS StudioBericht_<Name>` mit
`Erzeugen/Pdf/Drucken/Vorschau` und eine globale Variable unter dem Namen,
in studio_berichte.dh unter _studio/); `studio/berichte_reiter.dh` = Reiter
(Seite mit Baendern, gezeichnet in der Hauptschleife) und Vorschau
(PDF_PREVIEW, ein Feld je Parameter). Die Vorschau im Programm steht in
`laufzeit.dh` (`studioBerichtVorschau`, braucht die Bildschleife). **Falle:**
`band` ist das Schluesselwort BAND. **Laufzeit:** `PDF_PREVIEW` setzt Text in
der Schrift des PDFs (Schriftdaten ueber `Dokument::schrift_quelle`, je
Schrift und Pixelgroesse ein raylib-Font in `Graphics::vorschau_schriften`)
und Zeichen fuer Zeichen an die Stelle, die rustybuzz formt -- raylib rundet
jeden Vorschub ab, ein rechtsbuendiger Betrag endete 12 Punkte zu frueh.
Tests `werkzeug_studio_berichte.dhtest` (3), ein Fall in `drucken.dhtest`;
neun Verfaelschungen fallen.
**Stufe 2 (selber Tag): entwerfen.** Gearbeitet wird auf `r.ber.j` (JSON-
Handle im `BerichtAnsicht` des Reiters); `berichtMerken` legt nach jedem
fertigen Schritt den Text in `staende` (Rueckgaengig je Reiter) und in
`e.schema`. Modell in berichte.dh (`berichtBandEinfuegen` nach
`berichtBandSchluessel` -- Gruppenkoepfe aussen zuerst, Fuesse innen zuerst;
`berichtGruppeDazu/Weg` mit Nachruecken; `berichtElementNeu`,
`berichtSpalteElement`: Fuss = Summe, Seitenkopf = Ueberschrift, sonst Feld;
`berichtSpaltenTyp$`). Jedes Band hat eine Leiste mit Namen (`BER_STRIP`)
statt einer Namensspalte -- eine Quelle fuer Lagen ist `berBandY/Y2`,
`berBandBeiY`, `berMmImBand`, `berElRect`. Maus in `berichtMaus` (nach
`mausAufEntwurf`, setzt den Zeiger darueber), Spaltenzug ueber
`berichtVorUpdate` vor GUI_UPDATE; Treffer: Griff, Element ohne Rand,
Bandkante, Element mit 3 Punkten Rand (sonst gewann an der Kante das
Element darueber -- vom Test gefunden). Raster 1 mm (`berRaster`),
Ausrichtungslinien (`berEinrasten`, waagerecht gegen die ganze Seite,
senkrecht im Band). Eigenschaften = feste Zeilen je Name
(`BER_EIGENSCHAFTEN`), sichtbar je Auswahl (Element/Band/Bericht).
Menue: `zurueck/vor` und Verdoppeln/Loeschen/Alles gehen bei
`berichtVorn()` an den Bericht, `werkzeugeNachziehen` gibt sie frei.
**Falle (zweimal):** das Feld `band` einer Klasse ist das Schluesselwort
BAND -- `bandNr`. Und ein Feld-Literal aus Widget-Handles ist ARRAY OF
INTEGER. Bildspalten setzt die Auslegung noch nicht (abgelehnt). Tests
`werkzeug_studio_berichte2.dhtest` (6, Projekt aus
`_hilfen/berichtprojekt.dh`); 15 Verfaelschungen fallen je in ihrem Fall.
**Stufe 3 (2026-11-04): im Einsatz.** Das Beispiel
`examples/studio/rechnungen` druckt seine Rechnung ueber den Bericht
`Rechnungsbeleg`: das neue Datenbank-Element `Daten` (Tabellen mit
Verweisen, Datei `rechnungen.db` im Projektordner, gitignoriert) hat die
Abfrage `Belegzeilen` (ROW_NUMBER als Positionsnummer, Netto/Steuer/Brutto
je Zeile -- gerechnet wird in der Abfrage), das Modul `Datenbank` setzt nur
noch `db = Daten` und fuellt Beispieldaten, `Rechnung.alsPdf` ist
`Rechnungsbeleg.Pdf(pfad, {"id": Self.id})` (die Rechnung muss gesichert
sein), neu Datei -> Vorschau. Dafuer die Element-Eigenschaft **`wo`** ("Nur
Zeilen mit", `mwst = 19`, auch `<> < > <= >=`; `studioBWoZerlegen`/
`studioBPasst` in berichte_laufzeit.dh): eine Summe rechnet nur ueber die
passenden Zeilen, jedes andere Element erscheint nur, wenn sein Band eine
solche hat -- die Steuer je Satz ohne Unterbericht. Nebenfund: `anzahl` ohne
Spalte zaehlte 0. Im Inspektor heisst "Bereich" jetzt "Summe ueber" (das
englische "Panel" kam vom gleichnamigen gui-Schluessel). Tests ein Fall in
`werkzeug_studio_berichte.dhtest` (vier Verfaelschungen fallen), und
`werkzeug_studio_beispiel.dhtest` vergleicht `.dhdb`/`.dhreport` im Rundweg
und liest die PDFs (Positionen, Steuer zu 19 und 7 %, eine Rechnung nur zu
7 % ohne 19-%-Zeile).
**Bildspalten (selber Tag):** ein Bild-Element mit `spalte` zeigt je Zeile
das Bild dieser BLOB-Spalte. `studioBBildSpaltenVon(j)` sammelt die Spalten
der Bild-Elemente, `studioBLesen` liest nur diese mit `DB_GET_BUFFER` (Wert =
Marke "[Bild]", Puffer in `studioBPufferStelle`/`studioBPuffer`) -- vorher
gingen alle Spalten durch DB_GET_STRING. Gesetzt ueber `IMAGE_FROM_BUFFER`
in TRY (kaputte Bytes bleiben leer) und `studioBBildSetzen` (eingepasst und
mittig, auch fuer Dateien aus bilder/ -- vorher verzerrt). Eine gezogene
Bildspalte wird ein Bild-Element (`berichtSpalteElement`), `spalte` gilt im
Inspektor auch fuer Bilder. Test "bildspalten" in
`werkzeug_studio_berichte.dhtest` (Pixel der Vorschau: Quadrat mittig,
breites Bild fuellt, NULL/kaputt leer); vier Verfaelschungen fallen, der
weggenommene `imgfx`-Import im erzeugten Code ist gleichwertig (das
Datenbank-Element importiert es schon).
