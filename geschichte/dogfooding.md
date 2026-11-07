# Dogfooding-Runden und Demos

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Notizzettel, Kometen, Werkstatt, Feed-Leser, textwerk, Sequencer, Duell, Verein, Messdaten, Texteditor, Fotos, CHIP-8, Landkarte, Bastelzentrale, Nachtflug, Lichterglanz.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Dogfooding: Notizzettel (2026-10-09)

`examples/notizen/main.dh` (SQLite, Liste mit Suche, formatierter Text als
Markdown, Menue mit Kuerzeln, PDF, Tray, Rueckfrage bei Ungesichertem),
gefahren mit ECHTEN Fensternachrichten und im Bild angesehen. Fuenf Funde in
der Laufzeit: (1) **`INSTRREV`** fehlte (letzte Fundstelle, ab 0, -1, mit
`start` die letzte dort oder davor; `rmatch_indices`). (2) **Kein Weg zum
Inhaltsbereich** -- ein Programm mit Menue musste die Hoehe der Leiste
raten: `GUI_WINDOW_CONTENT_W/H` (`Gui::window_inhalt`, ueber `content_top`).
(3) **`GUI_CLICKED` auf einer Liste war STUMM** (immer FALSE, wie frueher bei
Kaestchen) -- ein Klick auf einen Eintrag meldet jetzt, auch auf den schon
gewaehlten, und feuert `on_click`. (4) **Markieren beim Halten:** bekam ein
Textbereich oder Textfeld den Fokus, waehrend die Maustaste noch unten war
(Klick in die Liste -> Programm setzt den Fokus in den Text), hielt es das
fuer seinen eigenen Zug und markierte bis zur Maus -- das erste Tippen haette
die Notiz ersetzt. `Gui::zug_feld` = das Feld, in dem der Druck begann; nur
dort markiert Ziehen. (5) **Der Leer-Hinweis** von Liste/Baum/Tabelle lief
rechts hinaus -- jetzt umgebrochen (`Gui::leer_hinweis`, eine Routine fuer
alle drei). Dazu Doku: `PDF_NEW` legt die erste Seite schon an (ein
`PDF_PAGE` danach gab eine leere), und `LEFT$`/`MID$` im Handbuch nannten
noch "Game"/"Basic". **Nachgezogen (Wunsch des Nutzers):** der Platzhalter
von Textfeld und Textbereich bleibt sichtbar, bis getippt wird -- auch mit
Fokus (Textbereich: Platzhalter zeichnen und den Zweig mit der Schreibmarke
trotzdem laufen lassen). **`PRINT` kommt zeitnah an:** im Terminal jede
Zeile (`ausgabe_terminal`, IsTerminal), sonst nach einem Zeilenende, wenn
50 ms seit dem letzten Mal vergangen sind (`Vm::out_zeit`), und immer vor
SLEEP und (zeitlich) bei FLIP -- vorher sammelte dhrt alles bis zum
Programmende, auch im Terminal. **`PDF_PAGE` auf der noch leeren ersten
Seite** legt keine zweite an (`Dokument::seite_beginnen`). **Abgeschnittener
Text in der Statusleiste** endet mit `...` (`text_kuerzen`; ohne Bildfall).
Tests `gui_zug_nur_im_feld.dhtest`,
`gui_inhaltsbereich.dhtest`, `beispiel_notizen.dhtest`, je ein Fall in
`gui_listen`, `gui_liste_komfort`, `zeichenketten_stellen` (gegen 2026.25
fallen alle 7 neuen Laufzeit-Faelle).

## Dogfooding: Spiel "Kometen" (2026-10-10)

`examples/kometen/main.dh` (Schiff, Kometen als Objekte in Listen mit
`ARRAY_FILTER`-Lambdas, Partikel, `AUDIO_SFX`, `CAMERA_SHAKE`, Szenen,
Bestenliste ueber `save`, Pause bei Fokusverlust), bedient mit echten
Fensternachrichten (auch `WM_KILLFOCUS`), die Bilanz des Maschinencodes
gelesen. Funde: (1) **`^` im Maschinencode** fehlte ganz -- jede Schleife
mit `dx ^ 2 + dy ^ 2` blieb in der VM; jetzt `powf` wie `vm::pow`, INTEGER
^ feste Ganzzahl 0..16 als Ganzzahl (`fester_exponent`). (2) **Kurz-
geschlossenes AND/OR ueber ein Objektfeld** im Wertemodus: Wert und
Wahrheitswert trafen sich an einer Stelle -- `melden` erweitert zum Wert,
`ablegen` packt die Zahl vor dem Sprung ein. (3) **Wert an eine uebersetzte
Funktion** geht im Wertemodus ueber die VM (`w_funktion`). (4)
**`PARTICLE_UPDATE(p, DELTA())`** rundete 0,016 auf 0 ms, die Partikel
alterten nie -- jetzt ist eine Kommazahl zwischen 0 und 1 ein Fehler mit dem
richtigen Aufruf (`builtins::dt_ms`, auch SPRITE_/ANIM_FSM_UPDATE). Tempo
war kein Problem (rund 0,4 ms je Bild bei 100 Kometen, fast alles Zeichnen).
Bewusst so gelassen: Schleifen, die `getroffen` rufen, bleiben in der VM --
die Funktion schreibt in die Liste, ueber die die Schleife laeuft. Tests
`beispiel_kometen.dhtest`, zwei Faelle in `jit.dhtest` (je mit und ohne), ein
Fall in `modules_particles.dhtest`; Doku `entwurf-maschinencode.md` Schritt
23, `module-particles.md`.

## Demo Nachtflug (2026-10-26)

`examples/nachtflug/` (Wunsch des Nutzers Hans Schnorrenberger, der im
Titel und Abspann steht: eine Demo mit eigener Trance-Musik in Stereo und
Bildern, die sprachlos machen, nicht hektisch; nach dem ersten Hoeren
"traeumerischer, wie Armin van Buuren"). `musik.dh`: 128 BPM, a-Moll, 92
Takte, alles aus AUDIO_NOTE/AUDIO_SFX: sechzehn Takte Harmonie mit None und
Sexte, eine Melodie ueber sechzehn Takte, Supersaw (sieben Saegen), Klavier,
Chor aus Formant-gewichteten Obertoenen, pumpende Flaechen (eine Viertel
lang, langsamer Einsatz), Trance-Gate, Fuellungen je Phrase. Stereo und
Ping-Pong-Echo sind VORAB in die Klaenge gemischt (`AUDIO_SOUND_MIX` mit
pan, `mitEcho`) -- auf dem einen sfx-Bus haette ein Hall die Bassdrum
verwischt; Bus: Hall per Tweener (weit, wo die Bassdrum schweigt),
Kompressor, Filter per Tweener. **Die Klaenge brauchen 2,7 s** (der Chor
allein 8 x 200 ms) -- `vorbauen` baut je Bild einen, die schweren zuerst,
waehrend das Intro noch schwarz ist; was fehlt, entsteht beim ersten
Gebrauch. **AUDIO_NOTE-Vibrato ist ein FAKTOR** (0.12 = 12 % = zwei
Halbtoene), die erste Fassung leierte. Gespielt auf einer Audio-Uhr (eine
Sechzehntel = ein Tick, je Bild einen halben Takt voraus), dieselbe Routine
`schrittPlanen` schreibt ueber `spielen` auch die WAV (`--wav`). `--ab TAKT`
startet mitten im Stueck, `--messen` meldet Bildrate und Spitze am Mischer
(0.88). `main.dh`: fuenf Szenen, vier als Fragment-Shader ueber POSTFX
(`assets/shader/*.fs` -- unter assets, damit `dhrt --export` sie
mitnimmt; Raymarching; der Text liegt in texture0 und wird obenauf gelegt), die fuenfte eine Galaxie aus 2600 Lichtspuren (duenne, lange
Quader entlang der Umlaufbahn, MODEL_INSTANCED) ueber einer Scheibe, die
`glanz.fs` mit DERSELBEN Kamera und Drehung rechnet (Kamera als Uniform).
Bis 2026.37 zeichnete MODEL_INSTANCED je verschiedener Farbe einmal, die
Farben stuften darum in eine Tafel aus 24; seit der Farbe je Instanz (siehe
`m3d`) hat jede Spur ihre eigene. Beschriftung deutsch mit englischer
Zeile darunter (`zweisprachig`).
**Dritte Runde (selber Tag, alle vier Wuensche des Nutzers):** die Bilder
folgen der Melodie (`tonBei(n)`: Huellkurve 1 auf dem Anschlag, 0.3
solange gehalten, `tonHoehe`; Uniforms `ton`/`hoehe`/`farbe` in jedem
Shader, die Akkordfarbe zieht in main weich nach), Szenenwechsel durch
Licht (`uebergang` in jedem Shader: Lichtkreis mit Strahlen, `aus`/`ein`
je einen Takt; Vorsicht: ein Shader ohne gesetztes `ein` zeigt nur Licht,
main setzt es darum in jedem Bild), eine Stimme im Breakdown
(`SPEAK_SOUND` -> WAV -> `SAMPLE_LOAD` -> drei `SAMPLE_NOTE` um +-0.12
Halbtoene versetzt; hier gibt es nur deutsche Stimmen, darum deutsche
Saetze), `--mitschnitt` (AUDIO_RECORD_*, der Mix mit Hall wie gehoert) und
die Exe (`dhrt --export`, 40 MB). **Vierte Runde (2026-10-27):** die
Stolpersteine der Demo in der Laufzeit behoben -- MODEL_INSTANCED mit einer
Farbe je Instanz in einem Aufruf, AUDIO_NOTE schneller (Wellenform einmal
entschieden, Phase in Umdrehungen, Sinus als Zeiger, alle 64 Werte exakt
nachgestellt; Vibrato/Slide als Zeiger bzw. Faktor; Rust-Test
`schnelle_note_gleicht_der_alten` gegen die alte Rechnung): 50 Sinusnoten
mit Vibrato 212 -> 81 ms, die Klaenge der Demo 2,8 -> 1,9 s. Dazu
(Wunsch des Nutzers) ein neuer Tunnel (Neonringe mit Kern und Schein in
wechselnden Toenen, Schachtwand als Zylinder mit Panelen und Lichtimpulsen,
Funken in Polarkoordinaten, Ziel am Ende; ACES-Abbildung) und hinter der
Galaxie der Weltraum (`weltraum(d)` in glanz.fs: 3D-Rauschen ueber die
Blickrichtung, also fest im Raum -- Nebel mit Faeden und Staub, ein
Milchstrassen-Band, drei Sternschichten). **Falle:** der Hintergrund darf
nicht an der Helligkeit NACH der Scheibe maskiert werden, sonst verschwindet
er hinter ihrem Leuchten; maskiert wird nur an den Spuren. **Fuenfte Runde
(selber Tag, alle vier Ideen):** Rueckung ab Takt 72 (`RUECKUNG`, Akkord-
nummern 8..15 = dieselben Akkorde versetzt, `akkTon`/`grundTon`/
`versatzBei`) samt Gegenmelodie; Startbild, waehrend `vorbauen()` alles
baut (3,3 s, der Chor je Stimme, damit kein Schritt laenger als ein paar
Hundertstel dauert); `--leicht` (Uniform `guete`, Schleifen brechen bei
`N * G()` ab und die Schritte werden groesser; `MODEL_INSTANCED` mit
`anzahl`); Sternschnuppen (Winkel der Blickrichtung), Schnee (drei
Schichten, die auf die Kamera zufliegen), ein Komet (Kopf und Schweifende
im Shader in die Bildebene projiziert) und ein Lichterschwarm, der
"Drachenhauch" schreibt (Zielpunkte aus `RENDERTARGET_IMAGE`; `CIRCLES`
will INTEGER-Felder). **Falle:** `DH_OHNE_AUDIO` kennt die Laufzeit nicht --
ohne `tonDa()` haette das Startbild die Bilder der Szenen-Pruefungen
verbraucht. **Falle:** `fragTexCoord.y` zaehlt hier
nach oben -- kein `uv.y = -uv.y`. Etwa 60 Bilder je Sekunde in 1280x720.
Tests `tests/pruef/beispiele/beispiel_nachtflug.dhtest` (7: die WAV mit `--- ton`,
jede Szene am Bildschirmfoto); ein kaputter Shader und eine Fassung ohne
Panorama fallen. **Sechste Runde (2026-10-27, Wunsch des Nutzers):** der
Rueckweg zur Erde, 148 statt 92 Takte (4:41): Orbit-Breakdown (80..96,
`erde.fs`: Strahl gegen Kugel, Stadtlichter als Ballungen aus feinem
Punktraster, Atmosphaerenrand, Sonnenaufgang ueber dem Erdrand), dritter
Hoehepunkt beim Wiedereintritt (96..112, `eintritt.fs`: Plasma ueber
Richtung x Abstand -- 3D-Rauschen, darum ohne Naht bei atan), ueber den
Wolken (112..128, `wolken.fs`: Volumen -- Dichte aus 3D-Rauschen ueber
groben Feldern, die Luecken lassen; je Schritt zwei Proben zur Sonne. Ein
Hoehenfeld war erst eine geschlossene Decke, mit Loechern dann Tafelberge
mit senkrechten Waenden -- der Nutzer: "eine geschlossene Decke"; dazu
Wildgaense in V-Form, je Gans ein paar Weltstrecken -- Rumpf, Hals, Fluegel
mit Knick -- ins Bild projiziert, in den Wolkenmarsch eingesetzt, sobald
der Strahl ihre Tiefe passiert; von der Seite gesehen war das V eine Linie,
darum ueberholen sie die Kamera und werden von oben gesehen), Kueste
(128..136, `kueste.fs`: SDF-Klippe mit Saeulen und Baendern, Brandungs-
felsen, Turm mit Galerie, Waerterhaus, Moewen als Vs im Bild, Lichtkegel
als Abstand Blickstrahl/Kegelachse, eine Umdrehung in zwei Takten; die
fernen Landzungen nicht in der Spiegelung, sonst harte Flecken im Wasser),
Tagesanbruch am Meer. Musik: Akkorde 16..23 = der Morgen in C-Dur (`imMorgen`,
`morgenMelodie`), das Hauptthema kehrt ueber den Wolken zurueck,
Schlussakkord C-Dur, vier Saetze der Stimme. Dazu: Schnee faellt jetzt
(vier Schichten, pendelnd), und die wartenden Funken des Schwarms waren
sichtbar -- **RGBA nimmt 255 als deckend**, die Rechnung stand umgekehrt.
**Falle:** ein Shader, der nicht uebersetzt, ist bei raylib nur eine
WARNING; der Szenenfall faellt trotzdem, weil `shader()` dann abbricht. Im
Browser (https://hasoschno70.github.io/Nachtflug/, eigenes Repository
HasoSchno70/Nachtflug) streicht die Seite `SET_FULLSCREEN` aus der Quelle:
raylib kennt dort die Bildschirmgroesse nicht und setzte den Zeichenpuffer
auf die Groesse des Monitors -- man sah nur einen Teil des Bildes. Tests
jetzt 18 (vier Szenen, Laenge, Morgen-Harmonie, wartende Funken mit und ohne
Schwarm verglichen). **Siebte Runde (selber Tag):** Geraeusche der Reise
(Grollen, Wind, Gaenserufe, Brandung, Moewen, Nebelhorn -- `AUDIO_SFX` mit
Tiefpass; gemessen je Geraeusch an seinen Frequenzbaendern: das erste
Grollen lag fast ganz bei 41 Hz, auf Laptop-Lautsprechern also stumm) und
ein echter Sonnenaufgang im Ausklang (`aufgang` in ozean.fs); der Schwarm
startet dort, wo die Sonne gerade steht -- `sonneImBild` rechnet Richtung
und Kamera des Shaders in Drachenhauch nach. Test "die sonne geht im
ausklang auf" (hellster Punkt im Streifen rechts steigt).
**Tablet (2026-10-27, Fund des Nutzers):** auf einem Samsung-Tablet im
Browser stockten Berge, Sternennebel und Leuchtturm -- am Rechner
ungedrosselt die drei langsamsten Szenen (Nebel 249, Kueste 259, Wolken 300,
Berge 395 Bilder/s gegen 640 beim Ozean), und `--leicht` beschleunigte die
Kueste gar nicht (Kugelverfolgung endet frueh, die Schrittzahl ist nicht der
Engpass). Neu in der Laufzeit **`POSTFX(shader, massstab)`** (0.25..1): der
Shader rechnet in einem kleinen Ziel (`post_klein`, einmal je Groesse
angelegt), das weich auf den Schirm vergroessert wird; die Szene liest er dann
weich statt naechster Punkt (Text im Bild). Gemessen 0.7 -> 1,7..1,8x,
0.5 -> 2,8..4x. Die Demo waehlt die Guete je Szene selbst (fuenf Stufen aus
Schritten und Massstab, `gueteNachziehen`: die ersten 20 Bilder einer Szene
zaehlen nicht, dann je 30 Bilder im Mittel ueber 20 ms -> eine Stufe tiefer;
eine neue Szene beginnt eine ueber der tiefsten bisher); `--stufe N`/`--leicht`
fest, unter `DHRT_FRAMES` immer Stufe 0, damit Bildproben nicht an der Last
haengen. Der Nebel sammelt sein Licht jetzt mit der Schrittlaenge (`gew`),
sonst war er bei Stufe 4 ein Viertel dunkler. Tests: Paar in
`shader_uniforms_geometry.dhtest` (rechts x/120 mit und ohne Massstab) plus
Fehlerfall, Paar in `beispiel_nachtflug.dhtest` (SLEEP(30) je Bild senkt bis
Stufe 4; unter DHRT_FRAMES nicht); ohne das Heruntergehen faellt der erste.
**Schwache Handys (selber Tag):** die Demo blieb beim Laden stehen -- alle
Klaenge vorab sind 415 MB WASM-Speicher (gemessen, rund zwanzig Minuten
Stereo) und am Rechner schon 12 s Bauzeit im Browser. `--sparsam`: ein
Probelauf ueber die Partitur (`trocken`; `spielen` tut nichts), in dem jeder
Klang-Baustein `vormerken(schluessel, FUNCTION() baustein(args))` ruft --
die Partitur bleibt die einzige Stelle, die weiss, was wann erklingt, und
der Baustein liefert seine eigene Bauanweisung als Lambda. Gebaut wird 4
Takte vor dem ersten Einsatz, freigegeben 5 Takte nach dem letzten
(UNLOADSOUND stoppt auch geplante Instanzen, darum der Nachlauf); nach einer
Pause ueber 11 Takte bekommt ein Klang einen neuen Abschnitt (der Chor im
Breakdown und ueber den Wolken). Gemessen: vor dem Start 0,15 statt 3,5 s,
Klaenge hoechstens ~130 statt ~330 MB. **Am Rechner aus**: Bauen waehrend
der Fahrt kostete 4..9 Bilder ueber 25 ms je 25 s (die alte Fassung keins);
die Browser-Seite schaltet es bei `navigator.deviceMemory` < 8 ein (ohne die
Angabe nach der Geraeteart), `?sparsam` erzwingt es. Tests: Takt-fuer-Takt-
Simulation (nichts erst beim Einsatz gebaut, hoechstens 150 zugleich) und
die Gegenprobe ohne Schalter; ohne Freigeben bzw. ohne Bauanweisung im Lead
faellt der erste.

## Demo Lichterglanz (2026-10-28)

`examples/lichterglanz/` (Wunsch des Nutzers: eine zweite Demo wie Nachtflug,
Thema Advent und Weihnachten; gewaehlt: Adventskranz als Rahmen,
festlich-elektronische Musik, alle vier Extras -- liegender Schnee, echte
Kristalle, Geraeusche, Stimme). Gebaut aus dem Geruest von Nachtflug
(Audio-Uhr, `vormerken`/sparsam, Guete-Stufen, Lichtuebergang, Schwarm),
musik.dh neu: 116 BPM, 112 Takte (3:56), das Glockenmotiv aus "Shchedryk"
(gemeinfrei) als 6+6+4 gegen den Takt, eine Nacht-Melodie in g-Moll, eine
Markt-Melodie in B-Dur, Rueckung im Schlittenflug, "Stille Nacht" (Gruber,
gemeinfrei) im 6/8 ab Takt 78, zwei Strophen genau bis Mitternacht (96),
zwoelf Roehrenglocken-Schlaege, Stern bei 99. Neue Klaenge: Celesta,
Spieluhr, Roehrenglocke, Schellen, Klavier mit Dauer, Streicher mit Chor,
Schritte, Stimmengewirr, Eule, Kamin, Ticken. Sechs Shader (kranz, dorf,
markt, wald, schlitten, stube); im Abspann zeigt dorf.fs den Stern an
`sternPos`, aus dem der Lichterschwarm "Frohe Weihnachten / Merry
Christmas" aufsteigt. Die Kugeln am Baum zaehlt `snNotenBis` (je Ton eine,
92 bis Mitternacht), in stube.fs O(1) ueber Ebenen und Sektoren statt einer
Schleife ueber alle. Schnee in main.dh: 420 Flocken, die 46 nahen als
Kristalle (`LINES` in einem Aufruf), 160 Saeulen am unteren Rand, die beim
Landen wachsen (Flocke + halbe Menge auf die Nachbarn) -- drinnen
ausgeblendet. **Zwei Fallen beim Bau:** ein hohles Zimmer als
`-sdBox` ist von aussen negativ (die Kamera vor dem Fenster sah nur Himmel)
-- eine Schale aus zwei Kisten; und ein Richtungsvektor, der nur fast
normiert ist (1.002), macht den Mond fuenfmal groesser, weil `dot` dann
ueber 1 geht. Tests `tests/pruef/beispiele/beispiel_lichterglanz.dhtest` (16: WAV,
neun Szenenbilder, Stille Nacht und Kugeln, Glockenmotiv, Harmonie,
liegender Schnee, sparsam samt Gegenprobe).

## Bastelzentrale (2026-11-06)

Wunsch des Nutzers nach den sieben Bastler-Punkten: "ein super Programm fuer
Bastler mit allem, was wir jetzt gebaut haben" -- gewaehlt: ein Leitstand
plus Handy-Seite. `examples/bastelzentrale/`: `main.dh` (Fenster ohne Rahmen,
Seitenleiste, Statusleiste, Bildschleife; `reg(bereich, widget)` meldet jedes
Widget einem Bereich, `bereichZeigen` blendet um, Diagramme zeichnet nur der
aktive Bereich nach `GUI_DRAW(FALSE)`), je Bereich eine Datei in `bereiche/`
mit `xxAufbau/xxTakt/xxZeichnen/xxZustand$` (Seriell mit Kurve, Arduino ueber
Firmata, MQTT, Modbus TCP, Klang = Mikrofon + MIDI-Uhr, Kamera mit
Bewegungsmelder, Pi-Pins), `bereiche/netz.dh` (httpd 8090, WebSocket 8091,
OSC 8000, mDNS `bastelzentrale`; ein Befehlsweg `nzBefehl` fuer Seite,
`/befehl?c=` und OSC; `/zustand` = derselbe JSON-Stand wie im WebSocket),
`assets/handy.html` (Platzhalter `%%WSPORT%%`). **Jeder Bereich hat eine
Demo-Quelle**, `DH_BZ_DEMO=1` oder [Alle Demos starten] verbindet alle;
Ports ueber `DH_BZ_WEB/WS/OSC` (0 = frei), `DH_BZ_MDNS=0`, `DH_BZ_BEREICH`.
Seriell/MIDI/gpio sind `IMPORT ... OPTIONAL`. **Keine Laufzeit-Aenderung.**
Fallen: eine neue Diagrammreihe beginnt mit so vielen Nullen, wie das
Diagramm Punkte hat -- erst alle Reihen einer Zeile anlegen, dann schieben;
eine lokale `x0` in einer SUB verdeckt die Konstante `X0` (`--check` sagt
es); `bereiche/*.dh` nimmt `dhrt_check.dhtest` aus wie Glutfadens ansicht/.
**Beim Pruefen:** ein offener Browser-Tab mit der Seite verbindet sich alle
2 s neu und zaehlt als zweites Handy. Tests
`tests/pruef/beispiele/beispiel_bastelzentrale.dhtest` (2: ein Bild je Bereich samt
Probekamera und Lauflicht; die Bastelzentrale als Kind, der Fall ist Browser,
Handy und OSC-App gegen dieselben Werte); sechs Verfaelschungen fallen je in
ihrem Fall.

## Dogfooding 15: Landkarte (2026-10-26)

`examples/karten/`: `kern.dh` (Web-Mercator, Kachelnummern, Haversine,
Massstab, GPX mit Abschnitten/Routen/Wegpunkten -- ueber XML_CHILD statt
XML_AT, das je Aufruf von vorn sucht --, Marker als JSON und GPX),
`kacheln.dh` (HTTP_GET_START, hoechstens 4 zugleich, kleiner Vorrang =
naeher an der Mitte zuerst, HTTP_CANCEL fuer Kacheln, nach denen im letzten
Bild keiner fragte, IMAGE_FROM_BUFFER aus HTTP_BYTES, Zwischenspeicher
`<daten>/kacheln/z/x/y.png` ueber .teil + RENAME, `kMaxSpeicher` mit
IMAGE_FREE der am laengsten ungenutzten; solange eine Kachel laedt, zeichnet
`karteZeichnen` den passenden Teil einer groeberen), `main.dh` (Zeichenflaeche,
Ziehen, Rad um den Zeiger `zoomUm`, Doppelklick, Rechtsklick = Marker per
GUI_PROMPT, Nominatim-Suche, Hoehenprofil als `chart` mit CHART_X in km,
Ausfuhr PNG ueber Rendertarget, PDF A4 quer, Drucken). Eigener User-Agent,
Zuschreibung auf jeder Karte. Gegen die echten Server einmal von Hand
ausprobiert (eine Ansicht, eine Suche), die Tests nie. **Funde:** (1) `\`
schnitt Richtung null ab, MOD rundet nach unten -- `(a \ b) * b + a MOD b`
war bei negativen Zahlen nicht `a`; **`\` rundet jetzt nach unten wie FLOOR**
(Entscheidung des Nutzers; `vm::int_div`, das Falten geht dieselbe Funktion,
`jit.rs::rechnen` mit sdiv/srem-Korrektur). Verhaltensaenderung fuer die
Notizen von 2026.38. Das Handbuch sagte ausserdem, `\` nehme Kommazahlen --
es ist ein Fehler. (2) Die imgfx-Doku nannte `IMAGE_FROM_BUFFER`/`_TO_BUFFER`
"core, kein IMPORT", `--check` verlangt `IMPORT "imgfx"`. Tests
`tests/pruef/beispiele/beispiel_karten.dhtest` (5: Kern gegen JavaScript, Kacheln vom
Gegenserver am Bild der Ausfuhr samt Platte und Vergessen, Abbruch mit
langsamem Server, Spur/Marker/Bild/PDF, echte Maus aus einer Aufnahme) und
`tests/pruef/sprache/ganzzahl_division.dhtest` (VM, Falten, Maschinencode mit
DHRT_JIT=immer, Fehler); acht Verfaelschungen des Beispiels und die alte
Laufzeit bzw. ein Bau mit altem Maschinencode fallen je in ihrem Fall.

## Dogfooding 13: CHIP-8-Emulator (2026-10-24)

`examples/chip8/`: `cpu.dh` (Rechenkern ohne Fenster, alle 35 Befehle,
Eigenheiten als Merker c8SchiebeVY/c8LadenErhoehtI/c8SprungVX/c8LogikVF/
c8Umbruch), `asm.dh` (Assembler in Cowgod-Schreibweise, zwei Durchgaenge,
Fehler mit Zeile), `faenger.c8s` (eigenes Spiel), `main.dh` (Fenster,
Tastenfeld nach LAGE ueber KEY_1..KEY_V, Ton, Nachleuchten gegen das
XOR-Flackern, Laden per Argument oder Ablegen). **Keine Funde in der
Laufzeit** -- der Uebersetzer fing `RND` ohne Klammern ab, Maschinencode
und VM rechneten mit festem Zufall Befehl fuer Befehl gleich; der Kern
schafft ~1,2 Mio. Befehle/s (VM 0,7), obwohl `c8Schritt` (liest die
Text-Globale c8Fehler) und `c8Zeichnen` (schreibt ein globales Feld) in der
VM bleiben. Die Funde lagen im Beispiel selbst: `RETURN` vor dem Anhaengen
der Fehlerstelle, und das Spiel wartete ZUERST auf den Zeitgeber und
rechnete dann (ein Schritt kostete 4 statt 2 Bilder). **Falle:**
`dhrt bild` wechselt in den Ordner des Programms -- ein relatives Ziel
landet dort, im Repository; immer `CWD$() + "/..."`. Argumente hinter `--`
reicht `dhrt bild` an das Programm weiter. Tests
`tests/pruef/beispiele/beispiel_chip8.dhtest` (12; drei Verfaelschungen des Kerns
fallen je in ihrem Fall), Helfer `_hilfen/chip8.dh`.

## Dogfooding 12: Fotobetrachter mit GTK 3 (2026-10-14)

Wunsch des Nutzers: ein Fotobetrachter mit einer externen Bibliothek als
Oberflaeche, mit Styling. `examples/fotos/` -- GTK 3 ueber `DECLARE ... LIB`
(unter Windows das GTK von Inkscape), dunkles Stylesheet ueber
`GtkCssProvider`; HeaderBar, GtkStack (Uebersicht als FlowBox mit Karten,
Einzelansicht als Overlay mit runden Pfeilen und Infoleiste), Vorschaubilder
per `g_idle_add`, GdkPixbuf `at_scale` (nie groesser als das Original --
at_scale zieht kleine Bilder sonst HOCH), EXIF-Drehung
(`gdk_pixbuf_apply_embedded_orientation`), Drehen, Originalgroesse, Diaschau
(`g_timeout_add`), Vollbild, Tasten (`key-press-event` +
`gdk_event_get_keyval` mit BYREF). Je Signalform eine eigene
`g_signal_connect_data`-Zeile mit ALIAS. Funde in `ffi.rs`: (1) **Nachgeladene
Module fanden ihre Abhaengigkeiten nicht** -- `LOAD_WITH_ALTERED_SEARCH_PATH`
gilt nur fuer das eine Laden; das SVG-Lademodul von GdkPixbuf fand librsvg
nicht, GTK brach beim ersten Symbol ab. Jetzt haengt `ordner_in_suchpfad` den
Ordner einer ueber den vollen Pfad geladenen Bibliothek hinten an den PATH
des Prozesses, einmal. (2) **Ein Fehler im Rueckruf liess das Programm ohne
Meldung haengen** -- gemerkt und erst gemeldet, wenn die Bibliothek
zurueckkehrt, und `gtk_main` tut das nie. Jetzt `rueckruf_gescheitert`: der
erste Fehler bleibt gemerkt (mit Zeile), ein Waechter-Faden schreibt ihn nach
einer Sekunde in die Fehlerausgabe, wenn ihn niemand abgeholt hat
(`WACHE_NR` zaehlt die abgeholten, `WACHE_GEMELDET` -> jeder weitere sofort);
`LAUFEND` haelt die laufenden Bibliotheksfunktionen fuer den Namen.
(3) **Weitere Rueckrufe laufen nach einem Fehler weiter** (wie ctypes) --
vorher lieferten sie stumm 0, in GTK reagierte kein Knopf mehr, auch nicht der
zum Schliessen. VERHALTENSAENDERUNG fuer die Fassungsnotizen. (4) **Jeder
spaetere Fehler trug die Zeile des ersten**: `err_line_set` blieb stehen, weil
ein Rueckruf-Fehler nie per CATCH verbraucht wird -- `Vm::fehlerzeile_vergessen`
vor jedem Rueckruf, `fehlerzeile_setzen` beim Melden (`ERROR_LINE()` stimmt).
(5, 2026-10-17, Glutfaden Stufe 10) **Den gemerkten Fehler holte der
naechste Aufruf ab, der zurueckkehrte** -- in `gtk_main` also ein Aufruf IN
einem spaeteren Rueckruf (`cairo_set_source_rgba` beim naechsten Zeichnen),
der ihn warf; jeder weitere Rueckruf scheiterte damit, und die Meldung wuchs
um eine Schale je Rueckruf. `RR_FEHLER` traegt jetzt die Tiefe (`LAUFEND`)
des laufenden Aufrufs, `rueckruf_fehler(tiefe)` gibt ihn nur dem Aufruf
dieser Tiefe. Fall "der fehler eines rueckrufs gehoert dem aufruf, der ihn
ausgeloest hat" in `ffi.dhtest` (qsort, dessen spaetere Vergleiche `abs`
rufen).
Nebenbei: `gdk_pixbuf_save` heisst unter Windows `gdk_pixbuf_save_utf8` (in C
ein Makro), `gdk_pixbuf_savev` ueberall gleich; `gdk_pixbuf_get_from_window`
liefert unter Windows nur Weiss -- die Vorfuehrung (`--probe --foto ziel`)
zeichnet das Fenster mit `gtk_widget_draw` in eine Cairo-Flaeche. Tests
`ffi.dhtest` (qsort-Fall geaendert, Waechter mit langsamem qsort samt
`--- stderr` und Zeilen, Suchpfad nur Windows), `beispiel_fotos.dhtest` (Bilder
schreibt der Fall selbst, das JPEG mit von Hand eingesetztem EXIF-APP1);
gegen 2026.31 fallen alle neuen bzw. geaenderten Faelle.

## Dogfooding 11: Texteditor (2026-10-14)

Wunsch des Nutzers: ein Texteditor "mit richtiger Schrift". `examples/texteditor/`
(Reiter, Suchen/Weitersuchen/Ersetzen, Gehe zu Zeile, Zeilennummern,
Umbruch, Zoom, Dateien per Argument oder Ablegen, Rueckfragen nicht
blockierend ueber GUI_DIALOG/GUI_PROMPT, Statusleiste mit Zeile/Spalte,
Kodierung und Zeilenende -- die letzten beiden per Klick umstellbar).
Funde: (1) **Kein Weg zu erfahren, was `"auto"` gewaehlt hat** -- ein Editor
las cp1252 richtig und schrieb es beim Sichern still als UTF-8 zurueck. Neu
`FILE_ENCODING$(pfad$)` -> `utf8`/`utf8bom`/`cp1252`/`binaer`
(`kodierung::erkennen`), und jeder Name ausser `binaer` geht beim Schreiben
wieder hinein. Dafuer die Kodierung **`utf8bom`** (`utf-8-bom`/`utf-8-sig`):
liest wie utf8, schreibt `EF BB BF` -- nur vor eine LEERE Datei
(`kodierung::bom_vorn`; `kodieren` selbst setzt es nie, sonst stuende es vor
jeder WRITELINE-Zeile; WRITE am Handle und APPENDFILE fragen die
Dateilaenge). (2) **`SETFONT` uebernimmt die Lade-Groesse** -- wer bei 32
scharf laedt, bekam Menue, Reiter und Statusleiste in 32 (zweite Runde mit
demselben Stolperstein). Neu `SETFONT(font, groesse)` = SETFONT + TEXT_SIZE in
einem; ohne zweites Argument wie bisher. (3) **`IF x THEN DIM y ...` meldete
nur "Unerwartetes Token DIM"** -- jetzt sagt die Meldung, wie es geht (DIM/
CONST vor dem IF; FOR/WHILE/REPEAT/SELECT/TRY/WITH nur im mehrzeiligen IF,
`inline_statement_kern`). Alle waren vorher schon Fehler. Tests
`kodierung.dhtest` (+4, zwei Erwartungen mit der laengeren Namensliste),
`runtime_font_delta.dhtest` (+2), `umstieg.dhtest` (+2),
`beispiel_texteditor.dhtest` (4: cp1252+CRLF bleiben samt Zeichen, das cp1252
nicht kennt; BOM bleibt, Binaeres wird abgelehnt; Suchen mit Umlauf; echte
Klicks auf die Statusleiste). Gegen 2026.31 fallen alle neuen Faelle.

## Dogfooding 10: Messdaten-Auswertung und ein Spiel im Browser (2026-10-13)

Wunsch des Nutzers: beide Ideen, "und bitte mit normalem Zeichensatz".
**(a) `examples/messdaten/`** -- CSV einlesen (60000 Zeilen in 0,2 s),
Ausreisser nach Median/MAD der GANZEN Aufzeichnung, Luecken (vor dem
Aussortieren gezaehlt), gleitender Mittelwert, Tagesauswahl, Tabelle, CSV
und PDF-Bericht mit beiden Diagrammen als Bild; `beispieldaten.dh` schreibt
beim ersten Start eine Woche (`.gitignore`). Funde: (1) **Eine normale Schrift
gab es nur ueber einen festen Windows-Pfad** -- `LOADFONT("sans"|"serif"|
"mono"[-fett|-kursiv|-fett-kursiv])` nimmt die eingebauten DejaVu-Schriften
(`schriften.rs`, Feature `schriften`, das `pdf` mitbringt; in `font_herkunft`
als `eingebaut:sans`, Schnitte ueber dieselbe Familie; eine Datei gleichen
Namens gewinnt). (2) **`ZEIT_FORMAT$("%Y-%m-%d")` kam still woertlich
zurueck** (59580 solche Zeitstempel in einer CSV) -- %Y %y %m %d %H %M %S %a
%A %% gehen, ein unbekannter %-Platzhalter ist ein Fehler
(`zeit::format_pruefen`). (3) **`GUI_CHANGED(wdg)`** -- hat die BEDIENUNG
das Widget im letzten GUI_UPDATE geaendert? `abdruck` vorher/nachher nur fuer
beobachtete Widgets (erster Aufruf beginnt zu beobachten, liefert FALSE);
Programm-Setzungen zaehlen nicht. (4) **Das Liniendiagramm kannte nur "Punkt
Nummer i"**: eine Luecke von 2,5 h war unsichtbar -- `CHART_X(c, xwerte)`
(aufsteigend, gleich viele; CHART_DATA/PUSH mit anderer Zahl hebt es auf) und
`luecke_ab` (Linie unterbrechen); EINE Lagequelle `linien_xs` fuer Zeichnen,
Treffertest, Fadenkreuz, Beschriftung. (5) **Einzeln beschriftete Punkte
(Tagesanfaenge) fielen in die Luecken zwischen den k-ten** --
`linien_beschriftung`: sind nicht alle beschriftet, steht jede, die nicht
ueberlappt. (6) **`CHART_DATA` mit weniger Punkten liess Nullen stehen**
(Woche -> Tag), jetzt kuerzt es. (7) **Kein Weg von Gezeichnetem zu einem
Bild, und das PDF kannte keine Bilder** -- `RENDERTARGET_IMAGE(rt)` spielt die
Befehle des Ziels SOFORT ab (ohne FLIP, `render_scene` wie beim FLIP, Bild
gespiegelt, weil RenderTextures auf dem Kopf stehen), `PDF_IMAGE(p, bild, x,
y, b[, h])` (`pdf::Op::Bild`, krilla `raster-images`, Vorschau ueber
`Image::draw`, GDI-Druck ueber StretchDIBits auf weiss gerechnet).
**(b) `examples/webspiel/`** (Gluehwuermchen: Lichter antippen, Kombo als
Tonleiter, Gold = Zeit, Rot = Abzug, Bestenliste mit Namen) im Web-Playground:
(8) **`LOADFONT("sans")` ging im Web-Bau nicht** (nur Feature graphics) --
`build_wasm.py` schaltet `schriften` ein (18,8 MB statt 13,7), ohne Feature
sagt die Meldung das. (9) **Dateien waren nach dem Neuladen weg** (MEMFS) --
`web_speicher.rs`: `/spiel` als IDBFS-Arbeitsordner, `syncfs` beim Start,
hoechstens je Sekunde aus FLIP und am Ende; `/spiel/assets` ist ein Verweis
(IDBFS sammelt mit `lstat`, nachgesehen in `libidbfs.js`); `-lidbfs.js`.
Belegt mit einem Zaehler, der nach "Ausfuehren" (= Neuladen) auf 2 stand.
(10) **Ein schneller Klick/Antippen kam im Browser nie an** -- `flanken.rs`
war dort seit #308 ausgenommen; emscriptens GLFW kennt die Rueckrufe.
(11) **26 Warnzeilen `GetKeyName` je Bild** -- `belegung_auffrischen` lief
in jedem FLIP; im Browser gilt die Lage, `fenster_handle` liefert dort null,
kein verstecktes Fenster. (12) **`GUI_WINDOW("...", BREITE / 2 - 170, ...)`
brach ab** (`/` liefert FLOAT): Befehle nehmen jetzt eine GLATTE Kommazahl als
INTEGER wie eine Zuweisung (`builtins::glatt_int` in `need_int` und den
strengen `gi`), 2.5 bleibt ein Fehler mit Hinweis. (13) `GENTEX_RADIAL`
mit dichte > 1 gab still eine Scheibe -- jetzt ein Fehler, Doku nennt 0..1.
Grenze: auf dem Handy oeffnet die Leinwand keine Bildschirmtastatur. Tests
`messdaten_bausteine.dhtest` (15), `beispiel_messdaten.dhtest` (2),
`beispiel_webspiel.dhtest` (2, echte Klicks aus einer Aufnahme); alle neuen
Faelle fallen mit 2026.30. Falle beim Testschreiben: der Quelltext eines
Falls steht VOR `--- programm`, sonst wird er nicht eingeschoben.

## Dogfooding 9: Werkstatt-Dashboard ueber MQTT (2026-10-12)

`examples/werkstatt/` (main.dh: Tachos, Leisten, Verlauf, Luefter mit
Bestaetigung, Hitzealarm mit NOTIFY und Automatik, CSV, Wiederverbinden;
sensor.dh spielt das ESP32-Board). Dazu der Rechnungen-Pilot 196 auf die
Datenbindung fuer Geld und Steuersatz. Funde:
(1) **Kein letzter Wille**: das Board meldet `online` mit retain, und nach
einem Stromausfall hielt jedes spaeter gestartete Dashboard es fuer lebendig
(gemessen). Neu `MQTT_WILL(topic$, inhalt$[, retain])` fuer das NAECHSTE
`MQTT_CONNECT` (`Vm::mqtt_wille`, beim Verbinden verbraucht; im CONNECT
Will-Flag 0x04/Retain 0x20, Topic und Nachricht hinter der Kennung, Rust-Test
von Hand nach 3.1.1). (2) **`MQTT_UPDATE` warf, wenn der Broker abbrach**
(Windows 10054) -- ein Dashboard ohne TRY endete beim Neustart des Pi. Wie
beim net-Modul in Runde 7 ist das jetzt ein ZUSTAND: `MQTT_IS_CONNECTED`
FALSE, `MQTT_PUBLISH`/`MQTT_SUBSCRIBE` liefern BOOLEAN (`senden` in mqtt.rs).
(3) **Ein stumm verschwundener Broker** (kein RST) wurde nie bemerkt: jetzt
`Client::ping_offen`, nach der halben Keepalive-Zeit (min. 2 s) ohne Antwort
gilt die Verbindung als verloren. (4) **Diagramme kannten kein Dezimalkomma**
-- in der Pixelschrift ist der Punkt ein Pixel ("1.25 A" las sich wie
"125 A"), neben "30,0" aus ZAHL$ uneinheitlich: `CHART_SET_FLAG "komma"`
(eine Stelle, `fmt_value`, dazu die Prozentangabe). (5) **Listen schnitten
zu lange Eintraege mitten im Buchstaben ab**, der Rest war nirgends zu
lesen: jetzt "..." (`mit_auslassung`) und beim Verweilen der ganze Text als
Tooltip, wenn der Eintrag keinen eigenen hat (`Widget::ganzer_text`, in
GUI_UPDATE ueber `liste_textplatz`, dieselbe Platzrechnung wie das
Zeichnen; die Zeile einer schlichten Liste zaehlt jetzt als eigener
Tooltip-Teil). (6) **Klapplisten konnten keinen Wert binden** -- 196 rechnete
den Steuersatz von Hand (Index -> 19/7/0): `GUI_LISTBOX_SET_DATA/DATA$/
FIND_DATA` gelten auch fuer Klapplisten (`werte_ref`), eine gebundene
Klappliste mit Werten bindet den WERT (Zahl wenn moeglich, ohne Auswahl
NULL) und waehlt beim Laden den Eintrag mit diesem Wert (`klapp_werte`).
Zwei App-Lehren im LIESMICH: das Board muss beim Start seinen echten Zustand
melden (sonst glaubt das Dashboard ein liegengebliebenes "Luefter an"), und
Wunsch (`/set`) und Bestaetigung sind getrennte Topics. Tests:
`modules_mqtt.dhtest` +3 (der Test-Broker `_hilfen/mqttbroker.dh` schreibt das
CONNECT als Hex nach `PORTDATEI.connect`, Modus `stumm` beantwortet keine
Pings), `beispiel_werkstatt.dhtest` (4; NOTIFY per `--- ersetzen`, damit kein
Test eine Meldung auf den Bildschirm wirft), `gui_liste_komfort.dhtest` +2
(Bild, Paar), `gui_bindung.dhtest` +1, `werkzeug_rechnungen.dhtest` +1,
`modules_chart.dhtest` und Rust-Tests in mqtt.rs/chart.rs. Gegenproben gegen
die installierte 2026.30 (`C:\Program Files\Drachenhauch-IDE\dhrt.exe`).

## Dogfooding 8: Vereinsverwaltung an drei Datenbanken (2026-10-12)

`examples/verein/` (Mitglieder, Beitraege, Zahlungen, Beitragsliste als PDF;
SQLite, PostgreSQL oder MySQL ueber dasselbe Programm). Die Datenschicht war
dicht (Sonde: ids, getroffene Zeilen, Datum/BOOL/NULL, Rollback auf allen
drei gleich) -- die Funde lagen an den Naehten:
(1) **Die Datenbindung kannte kein gleichzeitiges Bearbeiten.** Neu
`GUI_FORM_VERSION(win, spalte$[, formular$])`: LOAD merkt sich den Stand
(`Window::form_db`, je Formular Spalte/Stand/Konflikt), SAVE schreibt
`... spalte = spalte + 1 WHERE id = ? AND spalte = ?`, ein INSERT setzt 1.
Trifft das UPDATE nichts, liefert SAVE **0** und `GUI_FORM_CONFLICT$` sagt
`geaendert` (Zeile da) oder `geloescht`; sechstes Argument TRUE ueberschreibt
und liest den Stand neu. Ohne LOAD (und ohne TRUE) ist SAVE mit Versionsspalte
ein Fehler. (2) **SAVE auf eine geloeschte Zeile meldete Erfolg** (lieferte
die id) -- jetzt auch ohne Versionsspalte 0/`geloescht`. Dafuer musste
(3) **MySQL getroffene statt geaenderte Zeilen zaehlen**: `CLIENT_FOUND_ROWS`
in db.rs -- vorher lieferte `DB_EXEC` fuer ein UPDATE ohne echte Aenderung 0
(SQLite/PostgreSQL 1). (4) **Geld in Cent liess sich nicht binden** (die Doku
sagte selbst "bindet es nicht und schreibt es selbst", Pilot 196 rechnet
ebenso von Hand): `zahlen` 3 = Geldfeld, getippt/gezeigt wie `48,50`,
gebunden als INTEGER 4850; eine Zahl aus FORM_SET/LOAD ist Cent, ein Text
steht so da. (5) **"0,99" ging als TEXT in die Datenbank** -- die
Postleitzahl-Regel griff bei jeder fuehrenden Null; jetzt nur vor einer Ziffer
(`nummer_mit_null`). (6) **Die Filterzeile der Tabelle war in allen Themen
schwarz** (im dunklen fiel es nicht auf): sie, die Haken- und Balkenzelle und
der Zell-Editor fragten nach den Schluesseln `field_bg`/`field_border`, die es
in keinem Thema gibt -- `th()` liefert dann 0; jetzt `win_bg`/`widget_border`
wie das Textfeld, der Platzhalter ueber `leise`. (7) **Das Euro neben der
Pixelschrift war halb so hoch**: die Ausweichschrift kam in derselben
Punktgroesse; `an_pixelschrift` (graphics.rs) gleicht sie am "H" an die
Grossbuchstabenhoehe der Pixelschrift (0,1..0,8 der Groesse) und deren
Grundlinie an, Zeichnen und Messen gleich. Tests: `gui_bindung.dhtest`
(+3), `db_server.dhtest` (+2; derselbe Ablauf `_hilfen/formversion.dh` gegen
alle drei Datenbanken), `gui_tabelle_felder.dhtest` (3, Bild),
`schriften_vorrat.dhtest` (+1), `beispiel_verein.dhtest` (3: der App-Ablauf
gegen SQLite/PostgreSQL/MySQL mit DERSELBEN Erwartung, PDF ueber
`pdftext.dh`), Rust-Tests `geldfeld_tests`.

## Dogfooding 3D: Bauklotz-Werkstatt (2026-10-10)

`examples/werkstatt3d/main.dh` (Kloetze setzen per Strahl, Licht, Schatten,
Vorschau-Klotz, Einsturz ueber physics3d, gui-Werkzeugfenster), mit echten
Fensternachrichten bedient. Fuenf Funde: (1) **Zahlen mit Exponent**
(`1e9`, `2.5E-3`) waren im Quelltext ein Parse-Fehler -- jetzt FLOAT
(`lexer::scan_number`; ein Exponent jenseits von FLOAT ist ein Fehler, nicht
inf). (2) **`KEYHIT("z")`**: ein Tastencode meint die LAGE nach der
US-Tastatur (KEY_Z ist auf QWERTZ die Y-Taste), fuer Strg+Z falsch. Ein
Buchstabe/eine Ziffer/das Leerzeichen als Text meint jetzt die beschriftete
Taste (`Graphics::taste_fuer_zeichen` ueber `belegung`, dieselbe Quelle wie
die gui-Kuerzel); vorher war Text dort ein Fehler mit Hinweis.
`fenstersender.ps1` kennt `buchstabe:z` (Scancode aus MapVirtualKey).
(3) **`MODEL_MATRIX` warf keinen Schatten** -- `render_shadow_map` kannte nur
MODEL/MODEL_EX. (4) **Beleuchtete Modelle waren immer deckend**: beide
Licht-Shader schrieben Alpha 1.0; jetzt `colDiffuse.a * tex.a`, der Nebel
mischt nur die Farbe. (5) **`RAY_HIT_BOX`/`RAY_HIT_MODEL`** normalisierten die
Richtung nicht (Laenge 2 = halbe Entfernung, Nullrichtung = inf), die anderen
RAY_HIT_* schon. Tests `g3d_licht.dhtest`, `g3d_strahlen.dhtest`,
`taste_nach_buchstabe.dhtest`, `beispiel_werkstatt3d.dhtest`, Faelle in
`chex_literal`/`umstieg`. **Gegenprobe gegen die installierte Fassung**
(`C:\Program Files\Drachenhauch-IDE\dhrt.exe`): `dhrt test` der alten Exe
startet auch die Kinder mit ihr. **Falle:** vor einer Verfaelschung
committen -- `git checkout datei` nahm die noch nicht committete Aenderung
mit. **Nachzug (3D-Reste):** `MODEL_MATRIX_WIRES(handle, mat [, farbe])`
(Dreiecksgitter wie MODEL_WIRES, also mit Diagonalen) und
`CUBE_MATRIX_WIRES(mat [, farbe])` (die 12 Kanten eines Einheitswuerfels --
der Umriss gedrehter Kloetze; Entscheidung des Nutzers: beide). Beide werfen
keinen Schatten. `PHYS3D_SLEEP/SLEEPING`, `PHYS2D_SLEEP/SLEEPING`
(`(w, idx [, schlafen])`, FALSE weckt): ein frisch aufgebauter Stapel aus 1323
Wuerfeln kostet schlafend 0,3 statt 20 ms je Schritt; ein Stoss weckt, ein
schlafender Koerper IN DER LUFT bleibt haengen -- die Werkstatt legt darum nur
aufliegende schlafen. `IMAGE_WIDTH`/`IMAGE_HEIGHT` als zweite Namen (Meldungen
nennen jetzt diese). Tests in `m3d`, `physics3d`, `physics2d`, `image_io`.

## Dogfooding Netz: Feed-Leser (2026-10-10)

`examples/feedleser/main.dh` (RSS 2.0, RSS 1.0/RDF und Atom; HTTP_GET_START
holt, ein Auftrag (TASK_START) zerlegt mit dem xml-Modul, SQLite als Ablage,
Vorschau als GUI_RICHTEXT, Symbol im Infobereich, Mitteilung bei Neuem,
TIMER_EVERY alle 15 Minuten), mit echten Fensternachrichten bedient. Sechs
Funde: (1) **`ZEIT_PARSE`** kannte keine Zeitzone und nicht das Datum aus RSS
und E-Mail (`Sat, 10 Oct 2026 08:30:00 +0200`) -- `zeit::parse_mit` liest
ISO mit `Z`/`+hh:mm` und RFC 822 samt Namen (GMT, UT, EST..PDT, CET/MEZ,
CEST/MESZ) und rechnet in ORTSZEIT um (Entscheidung des Nutzers;
`builtins::lokaler_versatz`, unter Windows `SystemTimeToTzSpecificLocalTime`,
sonst 0). (2) **`TASK_START`** gab Argumente ueber die Kommandozeile weiter --
ein Feed von 200 KB scheiterte; jetzt `dhrt call ... --args-stdin` mit JSON
auf der Standardeingabe. (3) Ein **unbekannter Schluessel** bei
`GUI_THEME_SET/GET`/`GUI_METRIC_SET/GET` nannte die gueltigen nicht
(`gui::unbekannt`). (4) **`€` und andere Zeichen ausserhalb Latin-1 in der
eingebauten Schrift** schalteten die GANZE Zeile auf die Ausweichschrift --
jetzt wechseln nur die fehlenden Zeichen (`std_schrift`/`pixel_ersatz`, Laeufe
wie bei geladenen Schriften; Entscheidung des Nutzers). (5) **Ein `DIM` in
einer Schleife faengt jede Runde frisch an** (Entscheidung des Nutzers, wie
FreeBASIC): bis 2026.27 behielt `DIM s AS STRING` im Rumpf den Wert der
vorigen Runde, `DIM f[2]` im Hauptprogramm dagegen nicht. Die Anlage bleibt;
dahinter setzt `Compiler::dim_frisch` den Startwert per gewoehnlicher
Zuweisung (Zahlen/Texte/Wahrheitswerte als Konstante -- das kann auch der
Maschinencode --, Felder/MAPs/Structs/Mathe-Typen ueber `__frisch` in
`try_typtest`); nur, wenn `break_patches` nicht leer ist (= in einer
Schleife). Ein lokales Feld mit Groesse wird bei erneutem DIM neu angelegt.
Ein Fall in `name_collision.dhtest` sicherte das alte Verhalten und ist
umgedreht. Gemessen: keine Tempo-Aenderung (bench mit und ohne
Maschinencode), eine Schleife mit DIM bleibt Maschinencode.
(6) **`HTML_TEXT`** hielt jedes `<` fuer einen Tag-Anfang (`a < b und c > d`
-> `a d`); jetzt wie im HTML-Tokenizer nur vor Buchstabe, `/`+Buchstabe, `!`,
`?` (`html::tag_beginnt`). Im Beispiel selbst: eine gescheiterte Einfuegung
liess die Transaktion offen (das naechste `DB_BEGIN` scheiterte) -> ROLLBACK;
die gewaehlte Tabellenzeile geht beim Neuaufbauen nicht verloren. Tests
`modules_zeit`, `task`, `gui_thema_schluessel`, `schriften_vorrat`,
`dim_in_schleife`, `modules_html`, `beispiel_feedleser.dhtest` (Gegenserver
`dateien`, Standard-Feeds per `--- ersetzen`, Symbol und Mitteilung
herausgenommen; drei Verfaelschungen des Beispiels fallen -- die zweite erst,
nachdem die zweite Runde die Fehler der Feeds mit ausgab).

## Dogfooding Konsole: textwerk (2026-10-11)

`examples/textwerk/main.dh` (suche wie grep, csv mit Gruppen/Summen/Sortieren,
zaehle wie wc), aus fremden Ordnern, ueber Pipes und als exportierte Exe
benutzt. Funde: (1) **Relative Pfade aus der Kommandozeile zeigten ins
Leere** -- `dhrt run` und die exportierte Exe wechseln in den Ordner des
Programms; die Exe setzte nicht einmal `DHRT_START_DIR`, auch
`examples/172_filter.dh` als Exe fand `daten/obst.txt` nicht. Neu
**`STARTDIR$()`** und **`ARGPATH$(n)`** (Argument vom Aufrufort aus; absolut,
leer und `-` bleiben; `builtins::startort`/`arg_pfad`), die Exe ruft
`ins_quellverzeichnis_exe` (Entscheidung des Nutzers). (2) **`INPUT` nach
`STDIN()` hing** -- das Handle hielt die Sperre der Standardeingabe; jetzt hat
das Handle einen eigenen Puffer (1 MiB) und `INPUT` liest danach aus ihm
(`read_input_line`). Pipe aus der Git-Bash: 400 000 Zeilen 2,8 -> 1,2 s (Python
braucht dort 1,8 s; mit `<` 0,08 s -- die Langsamkeit ist die MSYS-Pipe).
(3) **`FORMAT$` mit einer Maske ohne Platzhalter** (VB-Maske `"#,##0.00"`) gab
die Maske zurueck und verlor den Wert still -- jetzt ein Fehler. (4) Neu
**`ZAHL$(x [, stellen])`** (deutsch, Tausenderpunkte; `zahl_text`), denn
`NUMFMT$` ist das Idle-Spiel-Format und `EURO$` nimmt nur ganze Cent.
(5) Kodierung **`"auto"`** (nur lesen; UTF-8, sonst cp1252, Nullbyte =
Binaerdatei mit eigener Meldung; zeilenweise je Zeile entschieden).
(6) **`TERMINAL()`/`TERMINAL_WIDTH()`** (`IsTerminal`; Breite ueber
`GetConsoleScreenBufferInfo` bzw. `ioctl(TIOCGWINSZ)`, sonst `COLUMNS`;
`Win32_System_Console` im windows-Crate). (7) **`dhrt --export x.dh
meins.exe`** legte einen ORDNER `meins.exe` an -- ein Ziel auf `.exe` ist jetzt
der Dateiname. (8) Umstieg: `ISDIR`, `ISFILE`, `COMMAND$`, `CURDIR$`, `ISATTY`
(auf `ISDIR` schlug die Meldung CHDIR/ISSTR/MKDIR vor). Tests `startort`,
`konsole_werkzeug`, `beispiel_textwerk`, Faelle in `stdin`, `umstieg`,
`dhrt_export`; alle neuen fallen mit 2026.25, `beispiel_textwerk` mit `ARG$`
statt `ARGPATH$` (5 von 7). **Falle beim Testen:** Rueckstriche in einem
`node -e '...'` aus der Git-Bash kommen nicht an -- Daten mit `\` per
Write-Werkzeug schreiben.

## Dogfooding Musik: Sequencer (2026-10-11)

`examples/sequencer/main.dh` (16 Schritte x 6 Spuren, Audio-Uhr mit 12 Ticks je
Schritt fuer den Swing, drei Schritte voraus geplant, Hall/Echo/Wah auf dem
SFX-Bus, Sichern/Laden als JSON, WAV trocken und als Mitschnitt). Funde:
(1) **`AUDIO_PLAY_AT` stoppte die vorige Instanz desselben Klangs** -- eine
voraus geplante Hi-Hat loeschte die wartende, am Mischer kam von zwei
Einsaetzen einer an (gezaehlt ueber `AUDIO_FFT`). Jetzt `SoundSlot::geplant`
(weitere Instanzen samt Start-Tick); STOP/PAUSE/RESUME/VOLUME/PITCH/Pan wirken
ueber `alle()` auf alle, `AUDIO_PLAY`/`PLAY_ON`/`UNLOADSOUND` raeumen sie weg.
(2) **Kira meldet einen wartenden Klang schon als `Playing`** --
`instanz_zustand` vergleicht den Tick mit der Uhr: `AUDIO_IS_PLAYING` erst ab
dem Tick, neu `AUDIO_IS_WAITING`, eine entfernte Uhr = beides FALSE
(Entscheidung des Nutzers). Ein vergangener Tick klingt SOFORT -- der
Sequencer ueberspringt verpasste Schritte. (3) **Eine WAV mit Bus-Effekten
ging nicht** (`AUDIO_SOUND_MIX` ist trocken): neu `AUDIO_RECORD_START([max_s])`
/`STOP` -> SOUND /`RECORDING` am FFT-Abgriff (`AUFNAHME`, `TAP_RATE` aus
`init`/`on_change_sample_rate`; Mitschnitt des Mixers, Entscheidung des
Nutzers). **Kira wendet die Track-Lautstaerke NACH den Effekten an**, also
hinter dem Abgriff -- `MASTER_GAIN` (gesetzt in `AUDIO_BUS_VOLUME("master")`)
rechnet sie fuer Mitschnitt und Pegel ein; eine Master-Lautstaerke ueber
`AUDIO_MODULATE` kennen beide nicht (dokumentiert). (4) Neu **`AUDIO_PEAK()`**
(hoechster Ausschlag seit dem letzten Aufruf, `PEAK` per `fetch_update`); der
Sequencer klippte mit sechs Spuren bis 1,13 und laeuft jetzt mit SFX-Bus 0,32 (bei 0,4 schwankte die Spitze 0,88..1,01 -- Echo in ms gegen Noten auf Ticks)
und Pegel in der Statusleiste. (5) **Ein Menuekuerzel auf der Leertaste nahm
einem Kaestchen mit Fokus die Taste** (es kippte UND das Kuerzel feuerte):
`gui::widget_nimmt_taste` -- Leertaste/Enter fuer Knoepfe, Kaestchen,
Listen ..., Pfeile/Pos1/Ende/Bild fuer Regler, Listen, Baeume ..., Buchstaben
fuer Liste/Baum/Tabelle/Klappliste; F1..F12 bleiben ausgenommen. Tests
`audio_planen` (9), `beispiel_sequencer` (5), ein Fall in `gui_menu_ausbau`;
die neuen fallen mit 2026.25, vier Verfaelschungen des Sequencers (Swing,
stumm, Laden, Bus 1,0) je in ihrem Fall. **Falle:** eine Shift-Taste per
PostMessage haelt nicht (GLFW fragt `GetKeyState`) -- Umschalt-Kuerzel im Test
ueber das Menue.

## Dogfooding Netz: Duell (2026-10-11)

`examples/duell/` (`main.dh` Wirt/Gast, `spiel.dh` die gemeinsame Simulation
im festen Takt, `relay.dh` ein schlechtes Netz mit Verzoegerung, Schwankung,
Verlust): TCP fuer Hallo/Willkommen/Rundenende, UDP fuer Eingaben (je Paket
die zwei vorigen mit) und den Stand als Binaerpaket; der Gast rechnet sein
Schiff voraus und gleicht mit jedem Stand ab, der Wirt rechnet das Gastschiff
im Takt der GAST-Eingaben (nicht seiner eigenen Schritte). `--bot N
--ohne-fenster` fuer Pruefungen. Der Nutzer hat keinen zweiten Rechner --
geprueft ueber zwei Prozesse auf 127.0.0.1, das Relay und die WSL (Wirt als
Linux-dhrt, Gast unter Windows ueber 172.x). Funde in `net.rs`: (1) **UDP nur
Text** -- `NET_UDP_SEND` nimmt jetzt einen BUFFER, neu `NET_UDP_RECV_BYTES`.
(2) **Absender bei IPv6** nur als `"::1:5000"` -- neu `NET_UDP_LAST_HOST$`/
`NET_UDP_LAST_PORT` (das Referenzbuch empfahl `SPLIT(adr, ":")[0]`, bei IPv6
leer). (3) **Ein hart beendeter Gast liess den Wirt abbrechen**: Windows meldet
ein UDP-Paket an einen geschlossenen Port beim NAECHSTEN recv_from als 10054,
und TCP-RECV/SEND warfen bei RST -- `verbindung_weg` macht daraus einen
Zustand (leer/0, `NET_IS_CONNECTED` FALSE), UDP liest weiter. (4) **Die
letzte Nachricht vor NET_CLOSE ging verloren** (5 von 5), wenn noch
Ungelesenes da war oder danach noch etwas ankam (RST verwirft den
Empfangspuffer der Gegenseite) -- `Drop for NetSock` schliesst nachlaufend:
`shutdown(Write)` sofort, dann liest ein Faden ueber `try_clone` (haelt den
Socket offen) bis EOF, hoechstens 2 s. Nur Ungelesenes abzulesen reichte
nicht: in der CI von 2026.30 nahm der Wirt unter Windows schneller an, als
das HALLO ankam, und der Fall "Absage" fiel. (5) **Jedes UDP-Paket startete einen
Faden** fuer die Namensaufloesung, auch fuer `127.0.0.1` -- IP-Literale gehen
jetzt direkt: 2000 Pakete 231 -> 17 ms. Tests `modules_net` (+5, alle fallen
mit 2026.25), `beispiel_duell` (4, `--- langsam`: Schleife, Relay 80/30/10 %,
hart beendeter Gast + naechster, Absage) -- dazu seit der CI von 2026.30 ein
Fall mit 30 % Verlust und Saat 1: die LETZTEN Eingaben haben kein Paket nach
sich, das sie mittraegt; ging das letzte verloren, stand der Gast am Ende
einen Schritt weiter (unter macOS gesehen, unter Windows mit Saat 7 nie).
Der Gast sendet es jetzt nach, bis ein Stand mit genau dieser Eingabe kommt,
und der Wirt gibt Staenden im Nachlauf einen NEUEN Tick (der Gast nimmt nur
neuere an -- die Bestaetigung kam sonst gar nicht an). Eigene Falle: `VAL("172.23.27.16")`
ist 172.23 -- ein Port ist nur ein Argument aus Ziffern; mit `127.0.0.1` fiel
es nicht auf, weil die Vorgabe dieselbe Adresse war. Nicht geprueft: zwei
echte Rechner hinter Routern/Firewalls.
