# Anhang A — Troubleshooting

Häufige Fehler beim Arbeiten mit GameBasic, sortiert nach typischer Auftrittsphase.

## Sektionen

1. Setup-Probleme — `gbrun.py` startet nicht, Editor öffnet nicht, Pygame fehlt
2. Sprache — `Variable nicht deklariert`, `Typkonflikt`, `End of File erwartet`
3. Spiel zeichnet nichts — `SCREEN` fehlt, `FLIP` fehlt, falsche Farb-Konstanten
4. Eingabe reagiert nicht — `KEYPRESSED` mit falschem Keycode, Game-Loop ohne `QUITREQUESTED`
5. Performance — zu viele Particles, zu langsamer Tree-Walker, wann `--vm`?
6. Tests / Asserts — wie man `gbrun.py --bench` zur Diagnose einsetzt

## Häufigste Fehler (Vorausschau)

- **`Variable 'x' nicht deklariert (DIM fehlt?)`** — typisch beim Vergessen von `DIM` vor erstem Gebrauch
- **`step ist Schluesselwort`** — Variablen-Namen nicht `step` nennen (FOR…STEP)
- **Pygame-Banner stört im Output** — wird automatisch unterdrückt; falls doch sichtbar, Ursache erklärt
- **`Argument(e) erwartet, erhalten 0`** bei Modul-Funktionen — vermutlich `IMPORT` vergessen

## Status

Skelett — wird in der iterativen Phase ausgearbeitet (Sammlung wächst während des Buch-Schreibens).
