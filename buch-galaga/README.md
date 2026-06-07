# Galaga — ein Arcade-Shooter, Schritt für Schritt mit GameBasic

Ein schlankes Lehrbuch, das **einen Galaga-Clone von Grund auf baut**: ein
Spielerschiff am unteren Rand, eine Formation bunter Gegner oben, die in
geschwungenen Bahnen **einfliegen**, in der Formation **schweben** und einzeln
im Bogen **herabstürzen** — und dabei **Bomben** abwerfen, denen du ausweichen
musst. Genau das Gefühl des Arcade-Originals.

Du lernst GameBasic nicht an trockenen Beispielen, sondern indem jedes Kapitel
ein konkretes Stück Spiel hinzufügt. Und: **die Sprites zeichnest du selbst** im
mitgelieferten Pixel-Editor `gbsprites`.

## Der Zielstand

Das fertige Spiel liegt als ein laufender Stand in
[`code/galaga.gb`](code/galaga.gb) — starte es jederzeit, um zu sehen, wohin die
Reise geht:

```
.venv\Scripts\python.exe gbrun.py buch-galaga\code\galaga.gb
```

Steuerung: **←/→** (oder **A/D**) bewegen, **Leertaste** schießen.

## Aufbau (~12 Kapitel)

| # | Kapitel | Was dazukommt | Sprach-Thema |
|---|---|---|---|
| 1 | Erstes Fenster | `SCREEN`, Spielschleife, ein paar Sterne als Hintergrund | Programmstruktur, `WHILE`, `FLIP` |
| 2 | Das Schiff | Sprite laden + zeichnen, mit Tastatur bewegen | Variablen, `IF`, `LOADIMAGE`/`DRAWIMAGE` |
| 3 | Sprites zeichnen | im `gbsprites`-Editor Pixel-Art + Animation | Werkzeuge, Frames, Export |
| 4 | Schießen | Bullet-Pool, nach oben fliegen | Arrays, `FOR`, Pools |
| 5 | Sternenhimmel mit Parallax | Stern-Arrays, Tiefen-Ebenen, Funkeln, Scrollen | Arrays von Daten, `SIN`, `RGB`, Parallax |
| 6 | Gegner & Formation | `Bug`-Klasse, Gitter, Reihen-Farben, Sway | Klassen, Methoden, `SIN`-Schwingung |
| 7 | Einflug-Manöver | geschwungener Einflug mit Bézier-Kurven | `curves`-Modul, `CURVE_BEZIER2`, Tupel |
| 8 | Sturzangriffe | Zustands-Automat: Formation → Sturz → zurück | `ENUM`, State-Machine |
| 9 | Bomben & Ausweichen | Gegner werfen Bomben, Bomben-Pool | mehr Pools, Timing |
| 10 | Treffer & Punkte | Kollisionen (AABB), Score, Leben, Game Over | Funktionen, `BOOLEAN`-Logik |
| 11 | Vollbild & Kamera | Spielfeld per Kamera-Zoom skalieren + zentrieren | `camera`-Modul, Letterbox, HUD-Trennung |
| 12 | Politur & Ausblick | Sound (`gbsfx`), Wellen/Level, Standalone-`.exe` | Module, Export |

Jedes Kapitel motiviert das nächste durch ein konkretes Bedürfnis im Spiel.
Der Plan darf wachsen — taucht unterwegs ein lohnendes Thema auf, bekommt es
ein eigenes Kapitel.

## Voraussetzungen

- Eine GameBasic-Installation (native Runtime `gbrt` gebaut — siehe
  [Haupt-README](../README.md)).
- Der mitgelieferte **Qt-Editor** (`gbedit`) und der **Sprite-Editor**
  (`gbsprites`).
- Keine Vorkenntnisse.

## Die Sprites

Die Start-Sprites liegen fertig in [`assets/sprites/`](assets/sprites): das
Schiff (`player.png`), drei mehrfarbige Gegner (`bug0/1/2.png`, je 2 Frames
Flügelschlag — eine Reihe pro Farbfamilie: violett-cyan, rot-gelb, blau-türkis),
der Schuss (`bullet.png`) und die Bombe (`bomb.png`, 2 Frames). Jedes gibt es
zusätzlich als `.gbsprite` — **öffne sie in `gbsprites` und gestalte sie nach
deinem Geschmack um.** Erzeugt werden sie reproduzierbar von
[`assets/make_sprites.py`](assets/make_sprites.py).

> Die drei Gegner teilen sich **dieselbe Form**, aber je eine eigene Farbfamilie
> (Flügel/Körper/Augen) — so sind die Reihen klar unterscheidbar und trotzdem
> jeder Gegner mehrfarbig. Eine Vorlage, drei Paletten: das spart Arbeit und ist
> genau die Idee des Originals.

## Die Musik

Der Hintergrund-Track [`assets/music/spaceshooter.mp3`](assets/music) ist ein
**frei lizenzierter** Arcade-Chiptune (*8-bit Epic Space Shooter Music* von
HydroGene, [OpenGameArt](https://opengameart.org/content/8-bit-epic-space-shooter-music),
**CC0** — keine Namensnennung nötig). Das ist bewusst **nicht** die Original-
Galaga-Musik (die ist urheberrechtlich geschützt), sondern ein Stück im selben
Geist. Tausch es gegen deinen eigenen Track (gleicher Name/Ordner) — und fehlt
die Datei, spielt das Spiel eine kleine **prozedurale** Melodie als Fallback.
Mit **M** schaltest du die Musik im Spiel an/aus. Details:
[`assets/music/CREDITS.txt`](assets/music/CREDITS.txt).

## Bauen (PDF/EPUB)

Wie beim großen Buch via [Pandoc](https://pandoc.org); die Kapitel-Reihenfolge
steht in `build/pandoc.yaml` (kommt mit den Kapiteln dazu).
