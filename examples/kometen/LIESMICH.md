# Kometen

Ein kleines Arcade-Spiel: ein Schiff zwischen Kometen, die beim Treffer
zerbrechen -- aus einem großen werden zwei mittlere, aus einem mittleren zwei
kleine. Jede Welle bringt mehr und schnellere Kometen. Die fünf besten
Ergebnisse stehen in einer Bestenliste (`bestenliste.json` neben dem
Programm).

- **Pfeile** oder **A/D** drehen, **Pfeil hoch** oder **W** gibt Schub,
  **Leertaste** schießt
- **P** hält an -- verliert das Fenster den Fokus, hält das Spiel von selbst
  an
- **M** schaltet den Ton, **F11** Vollbild
- ein Gamepad geht auch: linker Stick lenkt, A schießt, B gibt Schub

Bilder und Klänge entstehen im Programm (`IMAGE_FILL_POLY`, `AUDIO_SFX`),
Funken mit dem Modul `particles`, das Wackeln beim Treffer mit
`CAMERA_SHAKE`. Kometen und Schüsse sind Objekte in Listen, die
`ARRAY_FILTER` mit einem Lambda aufräumt.

Entstanden beim Dogfooding am 2026-10-10; dabei kamen `^` im Maschinencode,
der Zusammenfluss im Wertemodus und die Meldung bei `PARTICLE_UPDATE(p,
DELTA())` heraus.
