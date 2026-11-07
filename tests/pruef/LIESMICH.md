# Prüfsammlungen

`dhrt test tests/pruef` läuft alle Sammlungen (`*.dhtest`), auch in den
Unterordnern; `--schnell` lässt die mit `--- langsam` aus. Das Format steht
in `docs/werkzeuge.md`.

| Ordner | Was darin geprüft wird |
|---|---|
| `sprache/` | Sprachmittel: Typen, Felder, Klassen, Lambdas, Zeichenketten, IMPORT, FFI |
| `laufzeit/` | Compiler, VM, Maschinencode, Befehlsverzeichnis, Meldungen, Konsole, Prozesse |
| `werkzeugkette/` | `dhrt check/fmt/lsp/debug/doku/pruef`, Export, Pakete, Bauskripte, Doku-Prüfungen |
| `gui/` | das gui-Modul, Widget für Widget |
| `grafik/` | Zeichnen, Bilder, Schrift, 3D, Shader, Video |
| `ton/` | Klang, Noten, Mikrofon, Sprachausgabe |
| `module/` | Datenformate und Dienste: JSON, DB, PDF, HTTP, Physik, Tiled … |
| `geraete/` | Netz und Hardware: MQTT, WebSocket, OSC, mDNS, Modbus, Pins, MIDI, Kamera |
| `fenster/` | Fenster und Eingabe: Tasten, Maus, Aufnahmen, Zwischenablage |
| `ide/` | die IDE (`ide/ide.dh`) |
| `studio/` | das Drachenhauch Studio |
| `werkzeuge/` | die Werkzeuge 183–199 und weitere Hilfsprogramme |
| `glutfaden/` | das Beispiel Glutfaden |
| `beispiele/` | die übrigen Beispielprogramme |
| `buecher/` | die Bücher |

Hilfsprogramme, die mehrere Sammlungen benutzen, liegen in `_hilfen/`. Eine
Sammlung in einem Unterordner erreicht sie über `{sammlung}/../_hilfen/`.
