# Lichterglanz

Eine Weihnachtsdemo in Drachenhauch von Hans Schnorrenberger: knapp vier
Minuten Musik und Bild aus einem Programm, ohne eine einzige Klang- oder
Bilddatei. Die Reise geht über den Adventskranz: zu jeder Kerze ein Kapitel,
und am Heiligabend durchs Fenster in die Stube.

```
dhrt run main.dh                     # im Vollbild, von vorn
dhrt run main.dh -- --fenster        # im Fenster
dhrt run main.dh -- --ab 78          # ab Takt 78 (Heiligabend)
dhrt run main.dh -- --wav advent.wav # nur die Musik als WAV schreiben (trocken, ohne Hall)
dhrt run main.dh -- --mitschnitt advent.wav  # abspielen und mitschneiden, wie man es hört
dhrt run main.dh -- --leicht         # feste Stufe 3, für schwächere Grafikkarten
dhrt run main.dh -- --stufe 0        # feste Güte 0 (voll) bis 4
dhrt run main.dh -- --messen         # am Ende Bildrate und Spitze im Mischer
dhrt run main.dh -- --sparsam        # Klänge erst bei Bedarf (wenig Speicher)
```

Güte, sparsames Bauen der Klänge und das Startbild arbeiten wie in
[Nachtflug](../nachtflug/LIESMICH.md): sind die Bilder einer Szene langsamer
als 50 je Sekunde, geht sie eine von fünf Stufen herunter. Mit
`DH_OHNE_AUDIO=1` läuft die Demo ohne Ton.

Zum Weitergeben als eigenständiges Programm (der Ordner `assets/shader`
kommt mit):

```
dhrt --export main.dh Lichterglanz/Lichterglanz.exe
```

ESC beendet, F11 schaltet das Vollbild.

## Der Ablauf

| Takt | Szene | Was geschieht |
|---|---|---|
| 0 | Adventskranz | Tannengrün, Beeren, Samtband, vier rote Kerzen; die erste entzündet sich, die Kamera fährt in ihre Flamme |
| 8 | Erster Advent: das Dorf | eine Gasse zwischen Fachwerkhäusern, warme Fenster, in manchen ein Schwibbogen, Laternen, der Kirchturm am Ende |
| 24 | Kranz | die zweite Kerze |
| 26 | Zweiter Advent: der Weihnachtsmarkt | Buden mit leuchtender Theke und Lichterketten, Dampf aus den Glühweinbuden, ein Karussell, das sich auf der Audio-Uhr dreht |
| 42 | Kranz | die dritte Kerze |
| 44 | Dritter Advent: der Winterwald | ein zugefrorener See, verschneite Tannen, ein Reh im Mondlicht, Diamantstaub |
| 52 | Eisblumen | die Kamera senkt sich aufs Eis, sechsarmige Kristalle wachsen über den See und leuchten mit jedem Ton |
| 60 | Kranz | die vierte Kerze |
| 62 | Vierter Advent: der Flug zum Mond | tief über Hügel und Wälder, der Schlitten mit vier Rentieren zieht vor dem Mond vorbei, hinter sich Goldstaub; ab Takt 70 Polarlicht |
| 78 | Heiligabend | vor dem Fenster (Eisblumen in den Ecken), dann hindurch in die Stube: Kamin, Geschenke, der Baum schmückt sich zu „Stille Nacht“ |
| 96 | Mitternacht | es wird dunkler, zwölf Glockenschläge, jeder schickt einen Lichtring vom Baum aus |
| 99 | Der Stern | der Stern auf der Spitze geht auf |
| 102 | Abspann | das Dorf, über ihm der Stern; aus ihm steigen Funken auf und schreiben „Frohe Weihnachten“ |

## Die Musik

Festlich-elektronisch, 116 Schläge je Minute, ganz in Drachenhauch
synthetisiert (`musik.dh`). Gespielt wird auf einer Audio-Uhr: ein Tick ist
eine Sechzehntel, jedes Bild plant die nächsten Schritte voraus
(`AUDIO_PLAY_AT`), und die Bilder fragen dieselbe Uhr.

- **Das Glockenmotiv** aus „Shchedryk“ (Mykola Leontowytsch, 1916 -- als
  „Carol of the Bells“ bekannt, gemeinfrei): B-A-B-G in Sechzehnteln, je
  sechs Schritte, dreimal im Takt (6 + 6 + 4) -- so läuft es gegen den
  Viervierteltakt und ist jeden Takt wieder auf der Eins. Es zieht sich
  durch alles: Spieluhr im Intro, Celesta im Dorf und auf dem Markt, Zupfen
  im Schlittenflug.
- **Zwei eigene Melodien**: die Nacht in g-Moll (Dorf, Wald, Schlittenflug)
  und der Markt in B-Dur. Im Schlittenflug rückt alles einen Ganzton hinauf.
- **„Stille Nacht“** (Franz Xaver Gruber, 1818, gemeinfrei) im 6/8-Takt, zwei
  Strophen: erst als Spieluhr über Streichern, dann mit Klavier, Chor und
  wiegender Celesta. Je Ton hängt eine Kugel mehr am Baum -- 46 je Strophe,
  92 zur Mitternacht.
- **Instrumente**: Celesta, Spieluhr, Glocken, Röhrenglocken,
  Schlittenschellen, Streicher, Chor, Klavier, Supersaw-Flächen, Bassdrum
  nur im Dorf, auf dem Markt und im Schlittenflug.
- **Geräusche**: Kaminknistern, Schritte im Schnee, ferne Kirchenglocken,
  Stimmengewirr auf dem Markt, Wind und eine Eule im Wald, das Ticken der
  Uhr vor Mitternacht.
- **Stimme**: drei Sätze aus der Sprachausgabe des Systems (`SPEAK_SOUND`),
  geschichtet und mit Echo -- „Leise fällt der Schnee“, „Die Kerzen leuchten
  in der Nacht“, „Frohe Weihnachten“. Ohne Sprachausgabe bleibt es still.

## Der Schnee

Vor jeder Szene im Freien fällt Schnee in 2D. Die nahen Flocken sind echte
Kristalle: sechs Arme mit zwei bis vier Paaren Seitenäste, die Form aus der
Nummer der Flocke gewürfelt, jede dreht sich langsam (`LINES`, alle in einem
Aufruf). Was am unteren Rand ankommt, bleibt liegen: 160 Säulen, die mit
jeder Flocke wachsen und sich glätten -- über die Kapitel hinweg wird die
Decke höher. Drinnen (Kranz, Stube) ist sie ausgeblendet, im Abspann wieder
da. Im Schlittenflug fallen die Flocken schräg und schnell.

## Die Bilder

Jede Szene rechnet ein Fragment-Shader (`assets/shader`), über
`POSTFX(shader, massstab)` auf das ganze Bild gelegt; Text und Schnee liegen
in derselben Textur obenauf. Szenenwechsel gehen durch Licht: am Kranz in
die Flamme hinein, sonst ein Lichtkreis aus der Mitte.

- **kranz.fs** -- Raymarching: ein Ring aus zerzaustem Tannengrün mit Nadeln
  und Rauhreif, Beeren, Samtschleifen, vier Kerzen aus Wachs, das von innen
  leuchtet, Flammen mit blauem Saum, im Hintergrund unscharfe Lichter.
- **dorf.fs** -- eine Gasse aus wiederholten Häusern mit wechselnder Höhe;
  Schnee liegt, wo eine Fläche nach oben zeigt. Fachwerk, Andreaskreuze,
  Sprossenfenster und Schwibbögen werden auf die Fassade gerechnet. Im
  Abspann hebt die Kamera den Blick zum Stern.
- **markt.fs** -- Buden im Kreis, Glühbirnen als Abstandsfeld, deren Schein
  sich entlang des Strahls sammelt; das Karussell mit acht Pferden im
  Galopp, gestreiftem Dach und einem Lauflicht auf der Bassdrum.
- **wald.fs** -- Tannen auf einem Raster, ein spiegelnder See mit Rissen,
  ein Reh, Polarlicht als Schleier; die Eisblumen sind in einen Sektor
  gefaltete Kristalle, jeder mit eigenem Keim und eigenem Beginn.
- **schlitten.fs** -- ein Höhenfeld aus Rauschen mit Bäumen und Dorflichtern,
  der Mond groß am Horizont; Schlitten und Rentiere sind ein 2D-Schattenriss
  im Bild des Mondes, die Beine im Galopp auf dem Takt.
- **stube.fs** -- ein Raum mit Fensterwand, Kamin, Geschenken und dem Baum:
  die Kugeln liegen auf zwölf Ebenen zu je acht, ihre Nummer zählt von oben
  nach unten, sichtbar ist jede, deren Ton schon erklungen ist; die
  Lichterkette ist eine Schraube um den Baum.

Musik und Szenen prüft `tests/pruef/beispiel_lichterglanz.dhtest`.
