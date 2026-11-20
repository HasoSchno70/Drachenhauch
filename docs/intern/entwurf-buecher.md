# Entwurf: zwei neue Bücher

Stand 2026-11-19. Zwei Bücher, die Drachenhauch für Leute öffnen, die mehr
wollen als den [Einstieg](../../buecher/einstieg/README.md), aber nicht das
Nachschlagewerk lesen möchten:

* **A — „Kleine Wunder“:** die Sprache lernen an Programmen, die staunen
  lassen, meist grafische Spielereien.
* **B — „Fenster, Formulare, Daten“:** Anwendungen mit Oberfläche bauen, erst
  von Hand in der IDE, dann im Studio, an einem durchgehenden Projekt.

Beide erklären genau, **warum** etwas so geschrieben wird, haben Übungen mit
Lösungen und Bildschirmfotos und dürfen mehrere hundert Seiten lang sein.

## Was es schon gibt, und wo die neuen Bücher ansetzen

| Buch | Seiten | Für wen | Was es zeigt |
|---|---|---|---|
| Der Einstieg | 231 | wer nie programmiert hat | Fenster, Zahlen, Schleifen, Pong, Snake, ein Instrument, erste Knöpfe, ein Karteikasten |
| Das Lehrbuch / Handbook | 576 / 567 | wer nachschlagen will | jedes Modul, jeder Befehl mit Beispiel |
| Galaga | rund 100 | Spieleleute | einen Arcade-Shooter von Grund auf |
| Tippspiel | rund 150 | Anwendungsleute | eine Datenbank-Anwendung mit gui von Hand, Netz, Reiter |
| **A — Kleine Wunder** | **350–450** | wer den Einstieg kennt oder aus einer anderen Sprache kommt | **die ganze Sprache**, je Kapitel ein Konzept an einem Programm mit Wow |
| **B — Fenster, Formulare, Daten** | **400–500** | wer Anwendungen bauen will | **gui von Hand, dann Studio**, Datenbank, Berichte, Drucken, Weitergeben |

Abgrenzung: A wiederholt nicht Pong und Snake, sondern beginnt dort, wo der
Einstieg endet, und geht bis zu Klassen, FUNCREF, Lambdas, Koroutinen,
Comprehensions, Tupeln, Shadern und 3D. B baut nicht noch einmal die
Tippspiel-Anwendung, sondern legt den Schwerpunkt auf das Studio und auf das,
was eine Anwendung „fertig“ macht: Prüfen von Eingaben, Datenbindung,
Berichte, Mehrsprachigkeit, Barrierefreiheit, Auslieferung.

## Beide nebeneinander

| | A — Kleine Wunder | B — Fenster, Formulare, Daten |
|---|---|---|
| Untertitel | Drachenhauch lernen mit Programmen, die staunen lassen | Anwendungen bauen mit IDE und Studio |
| Leser | kennt Variablen und Schleifen (Einstieg oder eine andere Sprache) | kennt die Sprache in den Grundzügen |
| Form | 26 Kapitel, je ein eigenständiges Programm von 30 bis 200 Zeilen | 24 Kapitel an **einem** wachsenden Projekt, der Mediathek |
| Roter Faden | ein Sprachkonzept je Kapitel, vom einfachen zum anspruchsvollen | eine Anwendung, die von „ein Fenster“ zu „ausgeliefert“ wächst |
| Werkzeug | IDE und `dhrt run` | Teil I IDE mit `gui` von Hand, Teile II–IV Studio |
| Bilder | ein großes Bild je Kapitel, dazu Zwischenstände, alle aus dem Programm erzeugt | Fotos von Fenstern, Studio und Inspektor, über Aufnahmen erzeugt |
| Übungen | je Kapitel drei Stufen: ändern, erweitern, neu erfinden | je Kapitel drei Stufen, die Lösungen sind Zweige des Projekts |
| Was man am Ende kann | jedes Sprachmittel von Drachenhauch einsetzen und begründen | eine Anwendung mit Datenbank bauen, prüfen, drucken, übersetzen, weitergeben |
| Wow | jedes Kapitel ein Bild, das man herumzeigt | am Ende eine Anwendung, die aussieht wie gekauft |
| Umfang | 350–450 Seiten | 400–500 Seiten |
| Sprache | deutsch zuerst, englisch über die Übersetzungstafel | deutsch zuerst, englisch über die Übersetzungstafel |

## Was beide gemeinsam haben

### Der Aufbau eines Kapitels

1. **Was wir bauen** -- ein großes Bild des Ergebnisses und zwei Sätze.
2. **Der Plan** -- was das Programm tun muss, in Worten, bevor eine Zeile
   Code dasteht.
3. **Schritt für Schritt** -- der Code in Stücken von höchstens 20 Zeilen,
   nach jedem Stück ein lauffähiger Zwischenstand (mit Bild, wo er etwas
   zeigt).
4. **Warum so?** -- die Entscheidungen, die der Code trifft, jeweils mit der
   Alternative, die man *nicht* nimmt, und dem Grund. Das ist der Kern der
   Bücher.
5. **Was schiefgehen kann** -- echte Meldungen von dhrt, wörtlich, mit
   Ursache und Abhilfe.
6. **Übungen** in drei Stufen: *Drehen* (eine Zahl, eine Farbe ändern und
   vorhersagen, was passiert), *Erweitern* (eine Funktion dazubauen),
   *Erfinden* (etwas Eigenes mit dem Gelernten). Lösungen im Anhang.
7. **Zum Weiterlesen** -- die Stellen im Handbuch.

### Prüfen statt hoffen

* Jedes Programm und jeder Zwischenstand liegt als Datei in `code/` des
  Buchs und läuft in einer Prüfsammlung (wie `buch_pruefungen.dhtest`): Er
  übersetzt, und wo er etwas ausgibt, stimmt die Ausgabe.
* **Jede Lösung** einer Übung liegt ebenfalls als Datei vor und wird
  geprüft. Eine Lösung, die nicht läuft, ist schlimmer als keine.
* **Jedes Bild** erzeugt ein Skript (`dhrt bild` bzw.
  `DHRT_FRAMES`/`DHRT_SCREENSHOT`, bei Fenstern von IDE und Studio über
  Aufnahmen). Nach jeder Fassung lassen sich alle Bilder neu ziehen; ein
  Bild, das von Hand gemacht ist, veraltet still.
* Die Meldungen unter „Was schiefgehen kann“ entstehen aus echten Läufen
  (`--- fehler` in der Prüfsammlung), damit der Wortlaut im Buch der von
  dhrt ist.

### Werkzeuge

Dieselben wie für das Lehrbuch und den Einstieg: Inhalt als
`content/NN_*.js`, `tools/buch_bauen.dh` für das Word-Dokument mit
Seitenzahlen, `build_epub.js` für das EPUB, `i18n/en.json` als
Übersetzungstafel. Neu für beide Bücher:

* ein **Bildskript je Kapitel** (`bilder/NN_*.dh`), das die Bilder des
  Kapitels in `images/` schreibt;
* ein **Übungsanhang**, der die Lösungen aus `code/loesungen/` einliest,
  statt sie abzuschreiben.

### Arbeitsweise

Kapitel für Kapitel, je Kapitel ein Commit, mehrere Kapitel je PR. Zuerst
der Rahmen (Ordner, Bauskript, Prüfsammlung, Vorwort, ein Probekapitel),
damit Form und Ton feststehen, bevor die Masse geschrieben wird.

---

## Buch A — „Kleine Wunder“

*Drachenhauch lernen mit Programmen, die staunen lassen.*

**Versprechen:** Wer das Buch durcharbeitet, kann jedes Sprachmittel von
Drachenhauch einsetzen und sagen, warum er es an dieser Stelle nimmt. Jedes
Kapitel endet mit einem Programm, das man Freunden zeigt.

**Was der Leser mitbringt:** Variablen, Schleifen, `IF`, ein Fenster mit
`SCREEN` -- also das, was der Einstieg in seinen ersten zehn Kapiteln zeigt,
oder Erfahrung mit einer anderen Sprache.

### Vorwort und Kapitel 0: Die Werkstatt

* Wie man das Buch liest: tippen statt kopieren, erst vorhersagen, dann
  starten.
* Die IDE in fünf Minuten: Datei, F5, die Problemliste, F1 auf einem Wort.
* **Das erste Wunder in zehn Zeilen:** ein Farbkreis, der sich dreht
  (`CIRCLE`, `HSV`, `SIN`). Warum `DELTA()` und nicht „jedes Bild ein Stück“:
  dasselbe Programm läuft auf einem schnellen und einem langsamen Rechner
  gleich schnell.
* Übungen: Tempo, Farben, Zahl der Kreise.

### Teil I — Zeichnen mit Zahlen

**Kapitel 1: Sternenflug.** Sterne fliegen aus der Mitte auf den Betrachter
zu.

* *Sprache:* Felder mit fester Größe, `FOR`, `RANDINT`, die Perspektive als
  Division.
* *Warum so:* Ein Stern ist drei Zahlen (x, y, z) in drei Feldern -- noch
  keine Klasse, weil wir sie erst brauchen, wenn ein Stern etwas *tut*.
  Warum `/` immer FLOAT liefert und wann man `\` will.
* *Was schiefgeht:* Division durch null, wenn z genau 0 wird; ein Index aus
  `/`.
* *Übungen:* Farbe nach Entfernung; Warp-Taste; Sterne als Striche statt
  Punkte.

**Kapitel 2: Plasma.** Wabernde Farben über das ganze Bild, wie in den
Demos der Neunziger.

* *Sprache:* verschachtelte Schleifen, `SIN`/`COS`, Ganzzahl und
  Kommazahl, `PLOT` gegen ein Bild, das man einmal zeichnet.
* *Warum so:* Erst Punkt für Punkt (langsam, aber verständlich), dann über
  ein kleines Bild, das hochskaliert wird -- und **warum** das schneller
  ist. Erste Begegnung mit dem Maschinencode: dieselbe Schleife, gemessen.
* *Was schiefgeht:* ein Plasma, das flimmert, weil die Zeit falsch eingeht.
* *Übungen:* andere Farbpalette; Plasma nur in einem Kreis; die Maus
  verzerrt es.

**Kapitel 3: Spirograph.** Kurven aus zwei kreisenden Rädern, die sich
langsam zu einem Muster schließen.

* *Sprache:* `SUB` und `FUNCTION` mit Parametern und Rückgabe, Vorgaben
  (`= 1.0`), benannte Argumente beim Aufruf.
* *Warum so:* Eine Funktion `punkt(t)` macht aus „Mathematik“ einen Namen;
  benannte Argumente machen `rad(r: 80, abstand: 30)` lesbar.
* *Übungen:* drei Räder; die Kurve färbt sich mit der Zeit; ein Regler je
  Rad (Vorgriff auf Teil V, mit `GUI_SLIDER`).

**Kapitel 4: Das Ziffernblatt.** Eine analoge Uhr mit Sekundenzeiger, der
weich springt.

* *Sprache:* das Zeit-Modul, `CONST`, Winkel in Grad und Bogenmaß,
  `LINE` mit Dicke, Text um einen Kreis.
* *Warum so:* Konstanten für alles, was „feststeht“ (Radius, Farben), und
  warum eine `CONST` großgeschrieben auffällt.
* *Übungen:* Weltzeituhr mit drei Städten; Stoppuhr; Mondphase.

### Teil II — Viele Dinge auf einmal

**Kapitel 5: Feuerwerk.** Raketen steigen auf und zerplatzen in
Funkenregen.

* *Sprache:* `ARRAY OF` mit `ARRAY_PUSH`, Löschen beim Durchlaufen
  (rückwärts!), `STRUCT` für einen Funken.
* *Warum so:* STRUCT statt drei Feldern: was zusammengehört, bleibt
  zusammen. Warum man beim Löschen rückwärts läuft.
* *Was schiefgeht:* der Index, der nach dem Löschen eins zu weit zeigt.
* *Übungen:* Raketen mit Schweif; Farben je Rakete; Ton beim Zerplatzen.

**Kapitel 6: Das Spiel des Lebens.** Conways Zellautomat, der Gleiter und
Kanonen entstehen lässt.

* *Sprache:* zweidimensionale Felder, `DIM f[1 TO 80, 1 TO 60]`, eigene
  Untergrenzen, das Rechnen mit einem zweiten Feld.
* *Warum so:* Warum man in ein **zweites** Feld schreibt und nicht in das,
  das man gerade liest.
* *Übungen:* Muster mit der Maus setzen; Ränder, die herumlaufen; Farbe
  nach Alter der Zelle.

**Kapitel 7: Fallender Sand.** Sand, Wasser und Stein fallen, rutschen und
stauen sich, wie in den „Falling-Sand“-Spielen.

* *Sprache:* `ENUM` für die Stoffe, `SELECT CASE`, Regeln als kleine
  Funktionen.
* *Warum so:* ENUM statt Zahlen 0, 1, 2: der Übersetzer kennt die Namen, ein
  Tippfehler fällt beim Übersetzen auf.
* *Übungen:* Feuer, das Holz frisst; Pinselgröße; Schwerkraft umdrehen.

**Kapitel 8: Schwarm.** Hunderte Vögel fliegen gemeinsam, ohne Anführer
(Boids).

* *Sprache:* die erste eigene **Klasse**, Methoden, `NEW`, Felder eines
  Objekts.
* *Warum so:* Hier tut ein Ding etwas (es schaut sich um und lenkt) -- jetzt
  lohnt die Klasse. Warum die drei Regeln (Abstand, Ausrichtung,
  Zusammenhalt) je eine Methode sind.
* *Was schiefgeht:* der Vogel ohne `NEW` (NIL); der Vogel, der sich selbst
  sieht (Teilen durch 0). Das geplante Zittern, wenn alle Vögel schon den
  neuen Ort der anderen sehen, gibt es nicht -- nachgemessen kaum ein
  Unterschied; der Text sagt das und begründet die zwei Schleifen trotzdem.
* *Übungen:* ein Raubvogel; Hindernisse; der Schwarm folgt der Maus.

### Teil III — Bewegung, die sich echt anfühlt

**Kapitel 9: Federn und Seile.** Eine Hängebrücke aus Massepunkten, an der
man zieht.

* *Sprache:* Vektoren (`VEC2`), Operatoren auf Werten, ein kleines
  Physikmodell (Verlet).
* *Warum so:* Warum Verlet hier stabiler ist als „Geschwindigkeit
  aufaddieren“, an zwei Läufen gezeigt.
* *Übungen:* ein Vorhang; ein Seil, das reißt; Wind.

**Kapitel 10: Bälle in der Kiste.** Kisten stapeln, Bälle werfen, alles
fällt glaubwürdig um -- mit der eingebauten Physik.

* *Sprache:* das Modul `physics2d`, Handles, Rückgaben prüfen.
* *Warum so:* Wann man eine fertige Physik nimmt (fast immer) und wann
  selbst rechnet (Kapitel 9). Feste Zeitschritte.
* *Übungen:* eine Kanone; ein Domino; Gelenke.

**Kapitel 11: Sanft ankommen.** Menüs, Knöpfe und Figuren gleiten
federnd an ihren Platz.

* *Sprache:* das Modul `tween`, Easing-Kurven, **FUNCREF** -- eine Funktion
  als Wert.
* *Warum so:* Eine Kurve als übergebene Funktion statt fünf fast gleicher
  Befehle.
* *Übungen:* eigene Easing-Funktion; ein Titelbildschirm; eine Kette von
  Bewegungen.

**Kapitel 12: Ein Garten wächst.** Pflanzen wachsen nach Regeln
(L-Systeme) zu Bäumen und Farnen.

* *Sprache:* **Rekursion**, Zeichenketten als Bauplan, `MAP` für die
  Regeln.
* *Warum so:* Warum eine rekursive Funktion hier natürlicher ist als eine
  Schleife -- und wo die Rekursion endet.
* *Was schiefgeht:* die Tiefe, die den Speicher sprengt.
* *Übungen:* Wind im Baum; Jahreszeiten; ein eigenes Regelwerk.

### Teil IV — Bilder, Licht und Farbe

**Kapitel 13: Die Mandelbrotmenge.** Hineinzoomen in das bekannteste
Fraktal, mit flüssigen Farben.

* *Sprache:* lange Rechnungen, `DO … LOOP`, frühes Aussteigen mit `EXIT`,
  der **Maschinencode** im Detail.
* *Warum so:* Messen statt vermuten: die innere Schleife mit und ohne
  `DHRT_JIT=aus`. Warum die Rechnung in einer Funktion steht, die nur
  Zahlen kennt.
* *Übungen:* Julia-Mengen; Zoom mit dem Mausrad; ein Bild als PNG sichern.

**Kapitel 14: Wolken und Landschaften.** Rauschen erzeugt Wolken, Berge
und eine Karte mit Seen.

* *Sprache:* `NOISE`, `FBM`, Bilder als Puffer, `IMAGE_NEW`, Farbverläufe.
* *Warum so:* Zufall, der „zusammenhängt“, gegen `RND`.
* *Übungen:* Inseln; eine Weltkarte mit Höhenlinien; Wolken, die ziehen.

**Kapitel 15: Leuchten.** Neon, Glühen und ein Bild, das nachzieht --
mit Render-Zielen und Shadern.

* *Sprache:* `RENDERTARGET_NEW`, `SHADER_LOAD`, `POSTFX`, Uniforms setzen.
* *Warum so:* Was ein Shader ist (ein Programm je Bildpunkt auf der
  Grafikkarte) und warum er hier schneller ist als jede Schleife.
* *Übungen:* Röhrenbildschirm-Effekt; ein Glühen, das mit der Musik pulst
  (Vorgriff auf Teil VI).

**Kapitel 16: Pixelkunst und Paletten.** Ein Retro-Bild mit begrenzter
Palette, Rasterung und Farbzyklen.

* *Sprache:* das Modul `imgfx`, Farben als Zahlen, Bitoperationen
  (`BAND`, `SHR`).
* *Warum so:* Warum eine Farbe eine Zahl ist und wie man sie zerlegt --
  und warum `BAND` schwächer bindet als `+`.
* *Übungen:* eigene Palette; ein Wasserfall aus Farbzyklen.

### Teil V — Spielereien mit Eingaben

**Kapitel 17: Malen mit Symmetrie.** Ein Malprogramm, das jeden Strich
sechsfach spiegelt wie ein Kaleidoskop.

* *Sprache:* Maus und Tastatur (`MOUSE_DOWN`, `KEY_HIT`), Zustand über
  mehrere Bilder, ein bleibendes Render-Ziel.
* *Warum so:* Warum ein Bild, das bleibt, ein eigenes Ziel braucht.
* *Übungen:* Pinsel; Rückgängig; als PNG sichern.

**Kapitel 18: Der Theremin.** Die Maus spielt einen Ton: links tief, rechts
hoch, oben leise.

* *Sprache:* das Modul `audio`, `AUDIO_TONE`, Modulatoren
  (`AUDIO_LFO_NEW`), Umrechnung von Frequenz und Tonhöhe.
* *Warum so:* Warum Tonhöhe logarithmisch ist und Lautstärke auch.
* *Übungen:* Tonleiter einrasten; Vibrato; zwei Stimmen.

**Kapitel 19: Ein Bild, das zuhört.** Ein Visualisierer, der zur Musik
tanzt.

* *Sprache:* `AUDIO_FFT`, Felder als Spektrum, Glätten über die Zeit,
  **Tupel** als Rückgabe.
* *Warum so:* Warum man Werte über mehrere Bilder glättet (ein Bild ist
  1/60 Sekunde).
* *Übungen:* Kreis statt Balken; Farbe nach Bass; Partikel auf jedem Schlag.

### Teil VI — Die Sprache für Fortgeschrittene

**Kapitel 20: Die Partikelmaschine.** Ein Baukasten für Effekte: Feuer,
Rauch, Schnee, Magie -- per Rezept.

* *Sprache:* Vererbung (`EXTENDS`), überschreibende Methoden, `SUPER`,
  Lambdas (`FUNCTION(p) …`) als Verhalten.
* *Warum so:* Vererbung für „ist ein“, Lambda für „verhält sich so“ -- und
  warum man meistens das Lambda will.
* *Übungen:* ein eigener Effekt; Effekte kombinieren; ein Editor mit
  Reglern.

**Kapitel 21: Geschichten, die warten.** Eine animierte Szene, in der
Figuren nacheinander auftreten, warten und sprechen.

* *Sprache:* **Koroutinen** (`YIELD`, `CORO_RESUME`), Abläufe ohne
  Zustandsautomat.
* *Warum so:* Derselbe Ablauf einmal als Zustandsautomat und einmal als
  Koroutine -- und warum die zweite lesbarer ist.
* *Übungen:* eine Zwischensequenz; ein Tutorial, das auf den Spieler
  wartet.

**Kapitel 22: Die Datenwerkstatt.** Aus einer CSV mit Wetterdaten wird ein
animiertes Diagramm des ganzen Jahres.

* *Sprache:* Comprehensions (`[x * 2 FOR x IN …]`, Dict und Set),
  `ARRAY_MAP`/`ARRAY_FILTER`, f-Strings.
* *Warum so:* Wann eine Comprehension klarer ist als eine Schleife und wann
  nicht.
* *Übungen:* eigene Daten; Durchschnitt je Monat; ein Diagramm, das man
  anklickt (`CHART_NEW`).

**Kapitel 23: Fehler, die man fängt.** Ein Programm, das Bilder aus einem
Ordner zeigt und mit kaputten Dateien leben kann.

* *Sprache:* `TRY`/`CATCH`/`FINALLY`, `THROW`, eigene Fehlermeldungen,
  `--check` als Werkzeug.
* *Warum so:* Welche Fehler man fängt (die von draußen) und welche man
  verhindert (die eigenen).
* *Übungen:* Diaschau mit Übergängen; ein Fortschrittsbalken.

### Teil VII — Die dritte Dimension

**Kapitel 24: Würfel, Licht und Schatten.** Eine 3D-Szene mit
Grundkörpern, Licht, Nebel und Schatten.

* *Sprache:* die Module `g3d`/`m3d`, Kamera, Licht, Matrizen als Werte.
* *Warum so:* Was eine Kamera „ist“ und warum man die Welt um sie herum
  rechnet.
* *Übungen:* Sonnensystem; Tag und Nacht; ein Labyrinth von oben.

**Kapitel 25: Flug über eine Landschaft.** Eine Landschaft aus einer
Höhenkarte (Kapitel 14), über die man fliegt.

* *Sprache:* `MESH_HEIGHTMAP`, Himmel (`SKYBOX`), Instancing für Bäume.
* *Warum so:* Warum man tausend Bäume mit einem Aufruf zeichnet.
* *Übungen:* Wasserfläche; Tageszeit; ein Ziel, das man anfliegt.

**Kapitel 26: Das große Finale -- eine Demo.** Alles zusammen: Musik,
Partikel, Shader und 3D in einer Demo mit Abspann, wie Nachtflug.

* *Sprache:* Programme in Dateien aufteilen (`IMPORT`), Namensräume,
  Szenen nacheinander.
* *Warum so:* Wie man ein großes Programm in kleine verständliche Teile
  schneidet.
* *Zum Schluss:* exportieren (`dhrt --export`) und in der Spielwiese
  zeigen.

### Anhänge

* **A:** Lösungen aller Übungen (aus `code/loesungen/`).
* **B:** Spickzettel der Sprache auf zwei Seiten.
* **C:** Farben, Tasten, Formeln (Winkel, Vektoren, Easing).
* **D:** Meldungen von dhrt, die im Buch vorkommen, mit Seitenverweis.
* **E:** Wie es weitergeht: Handbuch, Galerie, Bibliotheken.

---

## Buch B — „Fenster, Formulare, Daten“

*Anwendungen bauen mit IDE und Studio.*

**Versprechen:** Wer das Buch durcharbeitet, baut eine Anwendung mit
Datenbank, Formularen, Tabellen, Suche, Berichten und Druck, übersetzt sie,
macht sie barrierearm und gibt sie als Programm weiter -- und weiß, welche
Arbeit das Studio einem abnimmt, weil er sie in Teil I einmal von Hand
getan hat.

**Das Projekt: die Mediathek.** Eine Verwaltung für Bücher, Filme, Spiele
und Musik eines Haushalts: Einträge mit Cover, Suche und Filter,
Ausleihen an Freunde mit Erinnerung, Abfrage einer ISBN aus dem Netz,
Statistik als Diagramm, Ausleihliste als PDF, Export nach Excel, deutsch und
englisch. Jeder kennt so etwas, es braucht alle typischen Bausteine, und es
ist kein zweites Rechnungsprogramm.

**Was der Leser mitbringt:** die Sprache in den Grundzügen (Einstieg oder
Buch A bis Teil II): Variablen, Felder, Funktionen, eine Klasse.

### Vorwort und Kapitel 0: Was eine Anwendung braucht

* Spiel gegen Anwendung: eine Schleife, die zeichnet, gegen Fenster, die auf
  Ereignisse warten.
* Der Rundgang durch die fertige Mediathek -- mit Bildern jedes Fensters.
* IDE und Studio: wann man welche nimmt.

### Teil I — Von Hand: das gui-Modul in der IDE

Hier entsteht eine **schlichte erste Mediathek** in einer Datei. Sie läuft,
und genau daran sieht man später, was das Studio vereinfacht.

**Kapitel 1: Ein Fenster und ein Knopf.** Das erste Fenster mit
Beschriftung, Eingabefeld und Knopf.

* *Inhalt:* `GUI_WINDOW`, `GUI_LABEL`, `GUI_TEXTINPUT`, `GUI_BUTTON`, die
  Ereignisschleife, `GUI_CLICKED`.
* *Warum so:* Warum ein Ereignis genau ein Bild lang gilt und was das für
  die Schleife heißt; warum ein vom Programm gesetzter Wert keine Bedienung
  ist.
* *Übungen:* Eingabe mit Enter abschicken; ein zweiter Knopf; ein Zähler.

**Kapitel 2: Ordnung im Fenster.** Layouts statt Pixelzahlen, damit das
Fenster mitwächst.

* *Inhalt:* Layouts, Ränder und Abstände, Mindestmaße, der Maßstab
  (`GUI_SCALE`), logische Koordinaten.
* *Warum so:* Warum feste Koordinaten bei 150 % Bildschirmskalierung und in
  Englisch (längere Wörter) zerbrechen.
* *Übungen:* dasselbe Formular in zwei Spalten; ein Fenster, das sich die
  Größe merkt.

**Kapitel 3: Listen und Tabellen.** Alle Einträge in einer Tabelle mit
Sortieren, Filtern und Auswahl.

* *Inhalt:* `GUI_TABLE`, Spalten, Sortieren, Filterzeile, Mehrfachauswahl,
  Doppelklick.
* *Warum so:* Die Tabelle hält die Daten nicht -- warum die Daten in der
  Datenbank bleiben und die Tabelle nur zeigt.
* *Übungen:* Spalte für Ausleihe farbig; Suche über alle Spalten.

**Kapitel 4: Daten, die bleiben.** Die Mediathek in einer SQLite-Datei.

* *Inhalt:* `DB_OPEN`, `DB_EXEC`, `DB_QUERY`, Parameter (`?`) statt
  zusammengesetzter Texte, Transaktionen.
* *Warum so:* Warum man nie Werte in SQL hineinklebt (ein Titel mit
  Apostroph genügt), und warum eine Transaktion.
* *Was schiefgeht:* „database is locked“, ein vergessener Commit.
* *Übungen:* Ausleihen in einer eigenen Tabelle; Löschen mit Nachfrage.

**Kapitel 5: Formulare, die prüfen.** Ein Eintrag wird angelegt und
geändert, und falsche Eingaben fallen sofort auf.

* *Inhalt:* `GUI_BIND`, `GUI_FORM_LOAD`/`GUI_FORM_SAVE`, Regeln mit
  `GUI_RULE`, `GUI_VALIDATE`, Fehler am Feld.
* *Warum so:* Warum die Prüfung an das Feld gehört und nicht in den
  Speichern-Knopf.
* *Übungen:* ISBN-Prüfziffer als Regel; Pflichtfelder je Medienart.

**Kapitel 6: Menüs, Kürzel, Dialoge.** Eine Menüleiste, Strg+N, Strg+S,
und Dialoge, die warten.

* *Inhalt:* `GUI_MENU`, Kürzel, modale Dialoge, `FILE_OPEN_DIALOG`,
  `GUI_MESSAGE`, `WINDOW_CLOSE_REQUESTED`.
* *Warum so:* Warum ein Kürzel nur im Menü steht (sonst läuft der Befehl
  doppelt) und warum man beim Schließen nach Ungesichertem fragt.
* *Übungen:* zuletzt geöffnete Dateien; Rückgängig für das letzte Löschen.

**Kapitel 7: Was wir von Hand gelernt haben.** Ein Blick auf die 600
Zeilen der ersten Mediathek: welche davon „Handwerk“ sind (Layout, Laden,
Speichern, Ereignisse verteilen) -- und genau diese nimmt das Studio ab.

### Teil II — Das Studio

Die Mediathek wird neu und **richtig** gebaut, im Studio.

**Kapitel 8: Das erste Projekt.** Ein Projekt anlegen, ein Fenster
entwerfen, starten.

* *Inhalt:* Projektordner, `projekt.dhstudio`, das Fenster als Klasse, der
  Layout-Editor, Starten mit F5.
* *Warum so:* Warum ein Fenster eine Klasse ist (es hat Daten und
  Verhalten) und was das Studio daraus erzeugt.
* *Übungen:* ein Info-Fenster; das Startfenster festlegen.

**Kapitel 9: Der Inspektor.** Eigenschaften, Anker, Reihenfolge der
Tabulatortaste, Bilder.

* *Inhalt:* Inspektor, Anker und Größenänderung, Tab-Reihenfolge,
  Tooltips, Sinnbilder.
* *Warum so:* Warum Anker die Layouts aus Teil I ersetzen.
* *Übungen:* ein Fenster, das auf 800 × 600 und auf Vollbild gut aussieht.

**Kapitel 10: Code und Ereignisse.** Methoden des Fensters, Ereignisse
der Controls, Module für Gemeinsames.

* *Inhalt:* Ereignisse (`Geklickt`, `Geaendert`, `Doppelklick` …), eigene
  Methoden, Module, Klassen im Projekt.
* *Warum so:* Was in ein Fenster gehört und was in ein Modul (Faustregel:
  alles, was ohne Fenster Sinn hat, gehört nicht hinein).
* *Was schiefgeht:* eine Methode `zeigen`, die das geerbte `Zeigen`
  überschreibt -- und wie `--check` das meldet.
* *Übungen:* Statuszeile, die die Zahl der Einträge zeigt.

**Kapitel 11: Mehrere Fenster.** Hauptfenster, Eintrag bearbeiten (modal),
Ausleihen (nicht modal).

* *Inhalt:* `ZeigenModal`, Rückgaben aus einem Fenster, Fenster, die
  einander benachrichtigen.
* *Warum so:* Wann ein Fenster modal sein muss und wann es nervt.
* *Übungen:* ein Einstellungsfenster; ein Fenster pro Eintrag.

**Kapitel 12: Der Debugger.** Haltepunkte, Schritte, Variablen ansehen --
an einem echten Fehler der Mediathek.

* *Inhalt:* Haltepunkt, Einzelschritt, Werte, Fehler an der Methode, in der
  sie stehen.
* *Warum so:* Suchen mit System statt `PRINT` überall.

### Teil III — Daten richtig

**Kapitel 13: Das Datenbank-Element.** Tabellen und Verweise im Studio
entwerfen, statt SQL zu tippen.

* *Inhalt:* Datenbank-Element, Spaltenarten, Verweise, der Reiter mit den
  Daten, eine vorhandene Datei übernehmen.
* *Warum so:* Warum Verweise (Ausleihe zeigt auf Medium und Freund) statt
  Namen in jeder Zeile.
* *Übungen:* Tabelle für Freunde; Schlagwörter (viele zu viele).

**Kapitel 14: Datenbindung.** Formular und Tabelle hängen an der
Datenbank, Laden und Sichern sind ein Aufruf.

* *Inhalt:* Bindung im Inspektor, Tabelle an eine Abfrage, Filterzeile.
* *Warum so:* Was die Bindung tut, im Vergleich zu Kapitel 5 von Hand.
* *Übungen:* Filter „gerade ausgeliehen“; Sortierung merken.

**Kapitel 15: Abfragen, die rechnen.** Wer hat was wie lange? Überfällige
Ausleihen, Statistik je Medienart.

* *Inhalt:* Abfragen im Datenbank-Element, `JOIN`, Gruppieren, Datum
  rechnen mit dem Zeit-Modul.
* *Warum so:* Warum die Datenbank rechnet und nicht eine Schleife im
  Programm.
* *Übungen:* die zehn meistgeliehenen Titel; Ausleihen je Monat.

**Kapitel 16: Aus dem Netz.** ISBN eingeben, Titel, Autor und Cover kommen
von selbst.

* *Inhalt:* `HTTP_GET` im Hintergrund, `JSON_PARSE`, Bilder laden,
  Fortschritt und Abbruch, Fehler im Netz.
* *Warum so:* Warum eine Netzabfrage das Fenster nicht einfrieren darf.
* *Was schiefgeht:* kein Netz, falsche ISBN, ein Server, der nicht
  antwortet.
* *Übungen:* Film-Daten aus einer anderen Quelle; ein Cache.

### Teil IV — Fertig machen

**Kapitel 17: Berichte und Drucken.** Die Ausleihliste und ein Karteikärtchen
je Medium als PDF, mit Vorschau und Drucken.

* *Inhalt:* das Bericht-Element, Bänder, Gruppen und Summen, Parameter,
  PDF, Vorschau, Drucken.
* *Warum so:* Warum ein Bericht nicht „ein Fenster, das man druckt“ ist.
* *Übungen:* Etiketten mit Barcode; ein Jahresrückblick.

**Kapitel 18: Diagramme.** Die Statistik der Mediathek: Balken, Torte,
Verlauf -- anklickbar.

* *Inhalt:* das Modul `chart` (`CHART_NEW`, Reihen, Klick auf einen Punkt).
* *Übungen:* Diagramm im Bericht; Zeitraum wählen.

**Kapitel 19: Ein- und Ausfuhr.** Daten nach Excel und CSV, Einlesen einer
alten Liste.

* *Inhalt:* `xlsx`, `csv`, Kodierungen, Fehler in fremden Daten.
* *Warum so:* Warum man beim Einlesen jede Zeile prüft und eine Liste der
  Fehler zeigt, statt abzubrechen.

**Kapitel 20: Aussehen.** Themen, die Stildatei, ein eigenes Control (die
Cover-Kachel).

* *Inhalt:* `GUI_THEME`, Stildatei (`GUI_STYLESHEET_LOAD`), eigene Controls
  im Studio.
* *Warum so:* Warum Farben in die Stildatei gehören und nicht in den Code.
* *Übungen:* ein dunkles Thema; ein Control „Sternebewertung“.

**Kapitel 21: Für alle.** Barrierearm: Tastatur überall, Bildschirmleser,
Kontrast.

* *Inhalt:* Tab-Reihenfolge, Beschriftungen, `GUI_ANNOUNCE`, Kontrast
  prüfen.
* *Warum so:* Warum Barrierefreiheit Tastatur-Tauglichkeit für alle ist.

**Kapitel 22: Zwei Sprachen.** Die Mediathek auf Englisch.

* *Inhalt:* die Tafel `sprache/en.txt`, `tr$`/`trf$`, was übersetzt wird
  und was nicht (Daten bleiben, wie sie sind), die Sprache des Systems.
* *Warum so:* Warum der Schlüssel deutsch bleibt und erst beim Anzeigen
  übersetzt wird.

**Kapitel 23: Weitergeben.** Ein Programm für andere: Export, Symbol,
Einstellungen im Benutzerordner, ein Installer.

* *Inhalt:* Bauen im Studio, `dhrt --export` (schlank), Daten neben dem
  Programm oder im Benutzerordner, Fassungen, Tray-Symbol für die
  Erinnerung.
* *Warum so:* Warum die Datenbank nicht neben dem Programm liegen darf,
  wenn es unter „Programme“ installiert ist.

**Kapitel 24: Was nun?** Mehrere Benutzer (PostgreSQL), eine Web-Seite
für die Ausleihe (`httpd`), Pakete für eigene Bausteine, die Galerie.

### Anhänge

* **A:** Lösungen der Übungen (als Stände des Projekts).
* **B:** Spickzettel gui und Studio: Ereignisse, Eigenschaften, Kürzel.
* **C:** SQL in zwei Seiten.
* **D:** Von Hand oder Studio? Eine Entscheidungstafel.
* **E:** Meldungen, die im Buch vorkommen.

---

## Aufwand und Reihenfolge

| Schritt | Umfang |
|---|---|
| Rahmen beider Bücher (Ordner, Bauskript, Prüfsammlung, Bildskripte, Übungsanhang) | eine Sitzung |
| Probekapitel je Buch (A: Kapitel 5 Feuerwerk, B: Kapitel 1 und 8), um Ton und Form festzulegen | eine Sitzung |
| Buch A, je Teil 3–4 Kapitel | je Teil ein bis zwei Sitzungen, sieben Teile |
| Buch B, je Teil 4–6 Kapitel | je Teil zwei Sitzungen, vier Teile; das Projekt wächst mit |
| Englisch über die Übersetzungstafel | je Buch einige Sitzungen, nach dem Deutschen |

Empfehlung: **erst den Rahmen und die zwei Probekapitel**, dann Buch A
Teil für Teil (es zieht neue Leute an), Buch B parallel, sobald Teil I von
A steht. Jeder Teil ist ein PR.

## Stand: die Probekapitel (2026-11-19)

Rahmen und Probekapitel stehen: `buecher/wunder` (Vorwort, Kapitel 5
Feuerwerk, Anhang A) und `buecher/anwendungen` (Vorwort, Kapitel 1 von
Hand, Kapitel 8 im Studio, Anhang A), je als `.docx` und `.epub`.
Geprüft von `tests/pruef/buecher/buch_wunder_anwendungen.dhtest`,
`buch_anwendungen_studio.dhtest` und `buch_programme_laufen.dhtest` (die
beiden letzten langsam).

Danach (2026-11-19, gleicher Tag): Buch A Kapitel 0 (Werkstatt, Bilder der
IDE über `buecher/wunder/buch/ide_bilder.dh`) und Teil I, Kapitel 1–4
(Sternenflug, Plasma, Spirograph, Ziffernblatt), je mit Lösungen im
Anhang.

Danach (2026-11-20): Teil II, Kapitel 6–8 (Spiel des Lebens, Fallender
Sand, Schwarm), je mit Lösungen und Bild zur Erfinden-Übung. Funde: Ein
ENUM-Wert, den es nicht gibt, fiel erst zur Laufzeit auf und in nie
gelaufenem Code gar nicht (behoben, Übersetzungsfehler mit Vorschlag,
`geschichte/sprache.md`). Drei Aussagen aus dem Entwurf hielten dem
Nachmessen nicht stand und stehen jetzt richtig im Text: Ohne Zufall
werden die Sandhaufen nicht schief (das Wasser aber fließt nur nach links);
der Schwarm zittert nicht, wenn Entscheiden und Bewegen in einer Schleife
stehen; die erste Fassung der Felsen ließ in jedem fünften Bild einen Vogel
hinein.

Was beim Bauen festgelegt wurde:

* **Code kommt aus Dateien.** `buch/code_lesen.js` holt ganze Programme,
  Ausschnitte (`stueck(datei, von, bis)`) und in den Lösungen die Stellen
  zwischen `' >>> neu` und `' <<< neu`. Kein Abdruck wird abgeschrieben.
* **Meldungen sind geprüft.** `buch/pruef_meldungen.js` lässt jedes
  Fehlerbeispiel laufen, dessen Abdruck mit `datei.dh:N:` beginnt, und sucht
  den Wortlaut in der Ausgabe.
* **Bilder:** `shoot.dh` mit `SKALIERUNG` 2 (Programme 800×500); eine
  Bedienung als `figures/<name>.txt` daneben (Klicks in Fensterpixeln, also
  mal 2). Das Studio bedient `studio_bilder.dh` auf einer Kopie des
  Projekts (`_studio_bilder/`, nicht versioniert); Tasten dort als
  raylib-Codes (F5 = 294, N = 78), nicht als `KEY_*` von Drachenhauch.
* **Tippen kann keine Aufnahme**: Sie spielt Tasten, keine Zeichen. Bilder
  mit eingetipptem Text gibt es darum nicht; die Prüfsammlung ersetzt das
  Lesen des Felds (`--- ersetzen TRIM$(GUI_TEXT(feld))`).

Funde unterwegs: Die Vorlage „Formular“ des Studios trug Enter- und
Esc-Knopf als Nummer, der Inspektor zeigte „(keiner)“ (behoben,
`knoepfeBeimNamen`, `geschichte/studio.md`). Zwei Behauptungen im
Entwurf des Kapiteltexts hielten dem Nachmessen nicht stand (`RGBA` rundet
selbst; die vierte Zahl von `SCREEN` ist der Maßstab) -- jede Aussage
über das Verhalten wird vor dem Schreiben ausprobiert.

## Offene Fragen

* **Titel:** „Kleine Wunder“ / „Fenster, Formulare, Daten“ -- oder andere?
* **Projekt in B:** die Mediathek, oder lieber eine Vereinsverwaltung,
  ein Haushaltsbuch, eine Werkstattkartei?
* **Englisch** gleich mitziehen (langsamer) oder nach dem Deutschen?
* **Wo die Bücher erscheinen:** wie die anderen als `.docx` und `.epub`
  im Repository, im Installer und auf der Website.
