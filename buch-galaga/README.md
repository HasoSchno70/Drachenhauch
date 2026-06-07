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

## Aufbau (schlank, ~10 Kapitel)

| # | Kapitel | Was dazukommt | Sprach-Thema |
|---|---|---|---|
| 1 | Erstes Fenster | `SCREEN`, Spielschleife, Stern-Hintergrund | Programmstruktur, `WHILE`, `FLIP` |
| 2 | Das Schiff | Sprite laden + zeichnen, mit Tastatur bewegen | Variablen, `IF`, `LOADIMAGE`/`DRAWIMAGE` |
| 3 | Sprites zeichnen | im `gbsprites`-Editor Pixel-Art + Animation | Werkzeuge, Frames, Export |
| 4 | Schießen | Bullet-Pool, nach oben fliegen | Arrays, `FOR`, Pools |
| 5 | Gegner & Formation | `Bug`-Klasse, Gitter, Reihen-Farben, Sway | Klassen, Methoden, `SIN`-Schwingung |
| 6 | Einflug-Manöver | geschwungener Einflug mit Bézier-Kurven | `curves`-Modul, `CURVE_BEZIER2`, Tupel |
| 7 | Sturzangriffe | Zustands-Automat: Formation → Sturz → zurück | `ENUM`, State-Machine |
| 8 | Bomben & Ausweichen | Gegner werfen Bomben, Bomben-Pool | mehr Pools, Timing |
| 9 | Treffer & Punkte | Kollisionen (AABB), Score, Leben, Game Over | Funktionen, `BOOLEAN`-Logik |
| 10 | Politur & Ausblick | Sound (`gbsfx`), Wellen, Standalone-`.exe` | Module, Export |

Jedes Kapitel motiviert das nächste durch ein konkretes Bedürfnis im Spiel.

## Voraussetzungen

- Eine GameBasic-Installation (native Runtime `gbrt` gebaut — siehe
  [Haupt-README](../README.md)).
- Der mitgelieferte **Qt-Editor** (`gbedit`) und der **Sprite-Editor**
  (`gbsprites`).
- Keine Vorkenntnisse.

## Die Sprites

Die Start-Sprites liegen fertig in [`assets/sprites/`](assets/sprites): das
Schiff (`player.png`), der Gegner (`bug.png`, 2 Frames Flügelschlag), der Schuss
(`bullet.png`) und die Bombe (`bomb.png`, 2 Frames). Jedes gibt es zusätzlich als
`.gbsprite` — **öffne sie in `gbsprites` und gestalte sie nach deinem Geschmack
um.** Erzeugt werden sie reproduzierbar von
[`assets/make_sprites.py`](assets/make_sprites.py).

> Der weiße Gegner wird im Spiel **pro Formationsreihe eingefärbt** (lila/rot/blau)
> — ein Sprite, viele Farben. Das spart Arbeit und ist genau die Technik des
> Originals.

## Bauen (PDF/EPUB)

Wie beim großen Buch via [Pandoc](https://pandoc.org); die Kapitel-Reihenfolge
steht in `build/pandoc.yaml` (kommt mit den Kapiteln dazu).
