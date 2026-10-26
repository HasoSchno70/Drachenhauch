# Karten

Eine Landkarte aus OpenStreetMap-Kacheln, in Drachenhauch.

```
dhrt run main.dh
dhrt run main.dh -- teufelsberg.gpx
```

* **Bedienen:** Ziehen verschiebt die Karte, das Mausrad zoomt um den Punkt
  unter dem Zeiger, ein Doppelklick zoomt hinein, die Pfeiltasten schieben,
  Strg+Plus/Minus zoomen. Ein **Rechtsklick** setzt einen Marker.
* **Ort suchen:** oben rechts eintippen, Enter -- gesucht wird bei
  Nominatim, dem Suchdienst von OpenStreetMap. Ein Klick auf einen Treffer
  springt hin.
* **GPX-Spur** (Datei → GPX-Spur öffnen, oder als Argument): Länge, Auf- und
  Abstieg, Höhenprofil. Fährt die Maus über das Profil, zeigt ein blauer
  Punkt die Stelle auf der Karte. Wegpunkte der Datei werden Marker.
* **Marker** stehen rechts in der Liste (Doppelklick springt hin) und
  lassen sich als GPX sichern.
* **Ausfuhr:** die Ansicht als PNG, als PDF (Karte, Höhenprofil, Marker)
  oder auf den Standarddrucker.

## Wo die Daten liegen

Unter `%LOCALAPPDATA%\Drachenhauch\karten` (sonst `~/.cache/Drachenhauch/karten`):
`kacheln/` ist der Zwischenspeicher -- was einmal geladen war, ist auch
ohne Netz zu sehen --, dazu `marker.json` und `einstellungen.json` (die
letzte Ansicht). Ansicht → Zwischenspeicher leeren räumt die Kacheln weg.

## Fair bleiben

Die Kachelserver von OpenStreetMap werden von Freiwilligen betrieben und
sind für **maßvollen** Gebrauch da ([Nutzungsrichtlinie](https://operations.osmfoundation.org/policies/tiles/)).
Das Programm nennt sich mit eigenem Namen, holt höchstens vier Kacheln
zugleich, bricht Abrufe für Kacheln ab, die aus dem Bild gerollt sind, und
holt keine Kachel zweimal. Für viele Abrufe -- etwa ganze Gebiete für
unterwegs -- einen eigenen Kachelserver nehmen:

```
dhrt run main.dh -- --kacheln "https://mein.server/{z}/{x}/{y}.png"
```

Weitere Schalter: `--ort breite,laenge,zoom` (Startansicht), `--suche URL`
(mit `{q}` für den Suchtext), `--daten ORDNER`.

Die Kartendaten stehen unter der ODbL, „© OpenStreetMap-Mitwirkende“ steht
darum auf jeder Karte und in jedem PDF.

## Aufbau

* `kern.dh` -- rechnet ohne Fenster: Web-Mercator (Grad ↔ Weltpunkte),
  Kachelnummern, Entfernungen (Haversine), GPX lesen, Marker als JSON und GPX.
* `kacheln.dh` -- Kacheln holen (`HTTP_GET_START`), abbrechen
  (`HTTP_CANCEL`), aus den Bytes ein Bild machen (`IMAGE_FROM_BUFFER`),
  auf der Platte merken, im Speicher begrenzen.
* `main.dh` -- Fenster, Zeichnen, Maus, Suche, Ausfuhr.

Geprüft in `tests/pruef/beispiel_karten.dhtest` -- mit einem eigenen
Kachelserver auf 127.0.0.1, nie gegen die echten Server.
