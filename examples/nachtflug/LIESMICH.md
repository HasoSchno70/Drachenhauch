# Nachtflug

Eine Demo in Drachenhauch von Hans Schnorrenberger: gut viereinhalb Minuten
Musik und Bild aus einem Programm, ohne eine einzige Klang- oder Bilddatei.
Eine Reise vom nächtlichen Meer über die Berge durch einen Lichttunnel ins
All -- und wieder heim zur Erde, bis die Sonne über dem Meer aufgeht.

Im Browser: https://hasoschno70.github.io/Nachtflug/

```
dhrt run main.dh                     # im Vollbild, von vorn
dhrt run main.dh -- --fenster        # im Fenster
dhrt run main.dh -- --ab 48          # ab Takt 48 (der Breakdown)
dhrt run main.dh -- --wav nacht.wav  # nur die Musik als WAV schreiben (trocken, ohne Hall)
dhrt run main.dh -- --mitschnitt nacht.wav  # abspielen und mitschneiden, wie man es hört
dhrt run main.dh -- --leicht         # für schwächere Grafikkarten
dhrt run main.dh -- --messen         # am Ende Bildrate und Spitze im Mischer
```

Vor dem Start entstehen alle Klänge (rund vier Sekunden); so lange steht ein
ruhiges Startbild mit Titel und einer feinen Linie, die sich füllt. Mit
`DH_OHNE_AUDIO=1` läuft die Demo ohne Ton (so wie in den Prüfungen).

Zum Weitergeben als eigenständiges Programm (der Ordner `assets/shader`
kommt mit):

```
dhrt --export main.dh Nachtflug/Nachtflug.exe
```

ESC beendet, F11 schaltet das Vollbild.

## Die Musik

Uplifting Trance von a-Moll nach C-Dur, 128 Schläge je Minute, 148 Takte
(4:41). Jeder Klang entsteht im Programm (`musik.dh`):

* **Harmonie** über sechzehn Takte: a-Moll, F, C, G, F, G, e-Moll, a-Moll,
  jeder Akkord mit None oder Sexte -- weich und offen statt nur Dur und Moll.
  Auf dem Rückweg zur Erde eine eigene, helle Folge in C-Dur: C, G/B,
  a-Moll7, F-maj7, C/E, G, F, G-sus -- dieselben Töne, aber das Licht liegt
  jetzt auf dem Dur.
* **Melodie** über sechzehn Takte in punktierten Vierteln, die sich Phrase
  um Phrase hinaufschraubt, und ein **Morgen-Motiv** in derselben Form, in
  C-Dur und mit weiteren Bögen.
* **Supersaw-Lead**: sieben Sägen, auseinander verstimmt und über das
  Panorama verteilt, ein Sinus eine Oktave tiefer, Ping-Pong-Echo.
* **Klavier** im Intro (angedeutet) und im Breakdown, **Chor** („aah“ aus
  Obertönen, gewichtet mit den Formanten eines offenen A) im Breakdown und
  im zweiten Höhepunkt.
* **Flächen**, die unter der Bassdrum **pumpen**, und ein **Trance-Gate**,
  das den Akkord in Sechzehntel zerhackt.
* **Stimme** in den Breakdowns und im Ausklang: „Schließ die Augen“, „Flieg
  mit mir durch die Nacht“, „Flieg mit mir nach Hause“ und „Der Morgen ist
  da“, gesprochen von der Sprachausgabe des Systems
  (`SPEAK_SOUND`, unter Windows die Stimme Katja), langsamer, dreifach
  geschichtet und mit langem Echo. Ohne Sprachausgabe bleibt sie weg.
* **Bassdrum** aus einem fallenden Sinus, **Offbeat-Bass**, **Klatschen**,
  **Becken**, **Shaker**, eine **Füllung** am Ende jeder Phrase,
  **Glocken** als Funken im Intro und im Ausklang, **Aufstieg** und
  **Wirbel** vor den Höhepunkten.

Ablauf: Intro (das Filter öffnet sich), Aufbau, erster Höhepunkt mit der
Melodie, Breakdown (erst Klavier und Chor allein, dann der Lead leise
darüber, ein Aufstieg über vier Takte), zweiter Höhepunkt -- und in seiner
zweiten Hälfte die **Rückung**: alles einen Ganzton höher (a-Moll nach
h-Moll), die Melodie mit der Oktave darüber und eine **Gegenmelodie** in
halben Noten. Dann der Rückweg: ein **zweiter Breakdown** im Orbit, in dem
die Harmonie nach C-Dur aufhellt und das Klavier das Morgen-Motiv spielt,
ein **dritter Höhepunkt** beim Wiedereintritt mit dem Morgen-Motiv als
Lead, über den Wolken **kehrt das Hauptthema zurück** (mit Oktave und
Gegenmelodie), an der Küste klingt der Groove aus, und im Ausklang spielt
das Klavier allein -- der letzte Akkord ist C-Dur. Wo die Bassdrum
schweigt, wird der Hall weiter.

Gespielt wird auf einer **Audio-Uhr**: jedes Bild plant die nächsten
Sechzehntel voraus (und baut nebenbei einen der Klänge, die noch fehlen), die Uhr spielt sie sample-genau -- und die Bilder fragen
dieselbe Uhr, darum sitzt jeder Puls auf der Bassdrum, auch wenn die Bildrate
schwankt. Echo und Stereobreite sind in die Klänge eingerechnet
(`AUDIO_SOUND_MIX` mit Panorama); Hall, Filter und Kompressor liegen auf dem
Bus.

## Die Bilder

| Takte | Szene | Wie |
|---|---|---|
| 0–16 | Ozean bei Nacht, Polarlicht, Mondglitzern | Fragment-Shader `assets/shader/ozean.fs` |
| 16–32 | Tiefflug durch ein Tal, ein Fluss aus Licht, Wellen im Takt | Raymarching über ein Höhenfeld, `flug.fs` |
| 32–48 | Lichttunnel aus Neonringen mit weißem Kern, Wände mit Lichtimpulsen, vorbeirasende Funken, ein helles Ziel am Ende; die Farben folgen dem Akkord | `tunnel.fs` |
| 48–64 | Sternennebel, zum Aufstieg hin schneller und heller | Volumen-Raymarching, `nebel.fs` |
| 64–80 | Galaxie aus 2.600 Lichtspuren in eigenen Farben, auf jeden Schlag läuft eine Welle nach außen; dahinter kosmische Nebel, das Band einer fernen Milchstraße und drei Schichten Sterne | echte 3D-Szene (`MODEL_INSTANCED`), die leuchtende Scheibe und der Weltraum dahinter rechnet `glanz.fs` mit derselben Kamera |
| 80–96 | Erdaufgang im Orbit: die Erde kommt näher, Stadtlichter an den Küsten unter Wolken, der Rand der Atmosphäre leuchtet von hinten -- am Ende geht die Sonne über dem Erdrand auf | Strahl gegen Kugel, Rauschen auf der Kugel, `erde.fs` |
| 96–112 | Wiedereintritt: Plasma strömt am Rand nach außen und glüht mit der Bassdrum, Funken fliegen vorbei, unten das Wolkenmeer im ersten Licht | `eintritt.fs` |
| 112–128 | Über den Wolken: Gleitflug über ein aufgerissenes Wolkenmeer mit einzelnen Türmen, durch die Lücken glitzert tief unten das Meer, ein Zug Wildgänse in V-Form zieht der Sonne entgegen, der Himmel geht von Violett über Rosa nach Gold | Volumen-Raymarching durch die Wolkenschicht, je Schritt ein Blick zur Sonne (Licht und Schatten), `wolken.fs` |
| 128–136 | Küste im Morgengrauen: zerklüftete Klippen, Brandungsfelsen, Gischt, ein Leuchtturm mit Galerie, dessen Strahl im Takt kreist, das Haus des Wärters mit erleuchtetem Fenster, Möwen, ferne Landzungen, rosa Wolkenstreifen | Raymarching gegen Felsen, Turm und Haus (SDF), der Lichtkegel im Dunst, `kueste.fs` |
| 136–148 | Tagesanbruch auf dem Meer, wo alles begann; Abspann | wieder `ozean.fs` |

Was pulsiert, pulsiert mit der Bassdrum; was aufblitzt, blitzt mit der
**Melodie** (Polarlicht und Mondglitzern, ein Funkenlauf auf dem Fluss, ein
Rahmen im Tunnel, der Nebel, in der Galaxie ein Ring, der umso weiter außen
liegt, je höher der Ton ist), und die **Farbe** geht mit dem Akkord. Die
Szenen gehen **durch Licht** ineinander über: ein Lichtkreis mit Strahlen
wächst aus der Mitte, die nächste Szene erscheint, während er nach außen
weicht.

Dazu kleine Momente: Sternschnuppen über dem Ozean, Schnee im Tiefflug, ein
Komet mit zwei Schweifen, der durch die Galaxie zieht, ein Blitz auf der
Rückung -- und im Abspann steigt ein Schwarm aus Funken aus der Sonne auf
und schreibt „Drachenhauch“ (die Zielpunkte kommen aus dem Schriftzug
selbst, in ein Render-Ziel geschrieben und mit `RENDERTARGET_IMAGE` als Bild
abgetastet).

Die Beschriftungen stehen deutsch und darunter englisch.

Die Shader-Szenen rechnen das ganze Bild im Fragment-Shader (`POSTFX`); was
das Programm vorher zeichnet (die Schrift), legt der Shader obenauf.
Gezeichnet wird in 1280×720, im Vollbild skaliert die Laufzeit hoch.

Geprüft in `tests/pruef/beispiel_nachtflug.dhtest`: die Musik an der
ausgeschriebenen WAV (Stereo, Länge, Pegel), jede Szene an einem
Bildschirmfoto.
