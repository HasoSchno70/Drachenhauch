# Star Pilot — Spiele bauen mit GameBasic

Ein Lehrbuch, das die Sprache GameBasic anhand eines durchgehenden Projekts vermittelt: einem klassischen Space-Shooter im Stil von Galaga. Jedes Kapitel führt ein neues Sprach-Konzept ein und baut damit ein konkretes Stück des Spiels.

Am Ende hast du ein vollständig spielbares Spiel mit Player, Enemies, Wellen, Boss-Fight, Particles, Highscore-Tabelle und Menüs — und nebenbei alle wichtigen Konzepte einer modernen Programmiersprache gelernt.

## Aufbau

| Phase | Kapitel | Was passiert |
|---|---|---|
| I — Erste Schritte | 1–4 | Sprache an der Konsole: Variablen, Kontrollfluss, Schleifen |
| II — Das Spiel beginnt | 5–8 | Erstes Fenster, Player bewegen, Schießen |
| III — Objektorientiert denken | 9–11 | Klassen, Vererbung, Kollisionen |
| IV — Spiel-Logik | 12–14 | ENUM, Sprites, Particles, Wellen-Choreographie |
| V — Spiel-Drumherum | 15–17 | Scenes, Save, UI, Pause |
| VI — Polish & Profi-Themen | 18 | Named Args, JSON-Wellen, TRY/CATCH |
| VII — Skalieren auf größere Spiele | [19](kap-19-performance-patterns.md), [20](kap-20-eigene-bulk-builtins.md) | Bulk-Ops, Sprite-Atlas + Batch, Z-Layer, LOAD_ASSETS; eigene Bulk-Builtins schreiben (cdef + Python-Fallback) |
| Anhang | A, B | Troubleshooting, Cython |

Die Reihenfolge ist nicht zufällig: jedes Kapitel motiviert das nächste durch ein konkretes Bedürfnis im Spiel.

## Lauffähige Code-Stände

Im Ordner [`code/`](code) liegt **pro Kapitel** ein vollständiger, ausführbarer Stand des Spiels. Du kannst jeden Stand direkt mit `gbrun.py` starten:

```
.venv\Scripts\python.exe gbrun.py buch\code\kap-09\main.gb
```

So siehst du den Fortschritt vom „gelben Kasten am Bildschirmrand" (Kap 5) bis zum fertigen Spiel (Kap 18).

## Voraussetzungen

- Eine GameBasic-Installation (siehe Haupt-[README](../README.md))
- Eine Texteditor (der mitgelieferte GameBasic-Editor reicht)
- Keine Vorkenntnisse — das Buch fängt bei `PRINT "Hallo"` an

## Bauen (PDF / EPUB / DOCX)

Das Buch wird in Markdown geschrieben und mit [Pandoc](https://pandoc.org) in die gewünschten Formate exportiert.

**PDF** (benötigt LaTeX):

```
pandoc --defaults=build/pandoc.yaml -o star-pilot.pdf
```

**EPUB**:

```
pandoc --defaults=build/pandoc.yaml -t epub -o star-pilot.epub
```

**DOCX** (z.B. für Reviewer mit Word):

```
pandoc --defaults=build/pandoc.yaml -t docx -o star-pilot.docx
```

Die Reihenfolge der Kapitel und alle gemeinsamen Optionen stehen in [`build/pandoc.yaml`](build/pandoc.yaml).

## Beitragen

Tippfehler, Verbesserungsvorschläge, eigene Erweiterungen des Spiels: gerne als Pull Request. Jedes Kapitel ist eine eigene `.md`-Datei — punktgenau editierbar.

## Lizenz

Privat (siehe Haupt-Projekt).
