# Bauklotz-Werkstatt

Klötze im Raum setzen, von allen Seiten ansehen und am Ende einstürzen
lassen. Ein Klick setzt einen Klotz auf den Boden oder an die Fläche, auf die
die Maus zeigt; ein halbdurchsichtiger Vorschau-Klotz zeigt vorher, wo er
landet.

- **Linksklick** setzt, **Rechtsklick** nimmt weg, **rechts ziehen** dreht
  die Kamera, das **Mausrad** zoomt, **Pos1** stellt sie zurück
- **Mittelklick** übernimmt die Farbe des Klotzes unter der Maus
- **Strg+Z** nimmt den letzten Schritt zurück -- abgefragt mit
  `KEYHIT("z")`, also auf der Taste, auf der das Z steht
- **[Einsturz]** übergibt alles an `physics3d`; ein Klick stößt dann den
  Klotz unter der Maus an, **[Aufbauen]** stellt den Bau wieder her
- **[Sichern]**/**[Laden]** schreiben `bau.txt` neben das Programm

Licht und Schatten kommen aus `LIGHT_DIRECTIONAL` und `SHADOW_ENABLE`,
das Treffen aus `RAY_HIT_BOX` mit dem Strahl von `SCREEN_TO_WORLD_DIR_*`,
die gedrehten Klötze im Einsturz aus `MODEL_MATRIX`, das Werkzeugfenster aus
`gui`.

Entstanden beim Dogfooding am 2026-10-10; dabei kamen Zahlen mit Exponent im
Quelltext (`1e9`), `KEYHIT("z")` nach der Beschriftung, Schatten für
`MODEL_MATRIX`, die Deckkraft beleuchteter Modelle und die Entfernung von
`RAY_HIT_BOX`/`RAY_HIT_MODEL` in Welt-Einheiten heraus.
