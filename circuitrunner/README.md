# Circuit Runner

Ein kachelbasiertes Puzzle-Spiel im Geist von **Chip's Challenge** (Atari Lynx /
MS), geschrieben in GameBasic. Sammle alle Daten-Chips, dann öffnet der Sockel
den Weg zum Ausgang — weiche Wasser, Feuer und Robotern aus, nutze Schlüssel,
Stiefel und schiebbare Blöcke.

> **Eigenständig, aber kompatibel.** Eigener Name und eigene Neon-Circuit-Grafik
> (kein Original-Artwork nachgezeichnet), aber **kompatibel mit dem originalen
> Levelformat** — heruntergeladene Fansite-Levelsets (`.dat`/`.ccl`, wie sie
> *Tile World* / *CCEdit* nutzen) lassen sich konvertieren und spielen.

![Tileset](assets/_contact.png)

## Spielen

```
gbrun.py circuitrunner\circuitrunner.gb        # oder im Editor F5
```

Läuft im **randlosen Vollbild**. Im Menü mit den Pfeiltasten ein **Level-Set**
wählen, mit LEER/ENTER starten.

| Taste | Wirkung |
|---|---|
| Pfeile / WASD | bewegen + schieben |
| LEER / ENTER | bestätigen / weiter |
| R | Level neu starten |
| N | Level überspringen (Entwicklung) |
| ESC | zurück / Menü |

## Eigene & heruntergeladene Level

Das Spiel scannt beim Start **alle `*.json`-Sets** in `levels/` und
`levels/_downloaded/` und bietet sie im Menü zur Auswahl an.

### Fansite-Sets (.dat / .ccl) konvertieren

Lade ein Set von einer Fansite (z. B. die frei verteilbaren *Chip's Challenge
Level Packs* CCLP1–5 von [bitbusters.club](https://bitbusters.club/)) und wandle
es um:

```
py circuitrunner\convert_dat.py  pfad\zu\CCLP1.dat
```

Das schreibt `levels/CCLP1.json` (bzw. nach `levels/_downloaded/`, wenn dort
abgelegt) — beim nächsten Start erscheint das Set im Menü.

> Heruntergeladene Sets bitte in `levels/_downloaded/` ablegen (ist
> gitignoriert, da urheberrechtlich geschützt — nur die Daten der jeweiligen
> Autoren, nicht in diesem Repo enthalten).

### Eigene Level bauen

`make_demo_levels.py` erzeugt die mitgelieferten Demo-Level aus ASCII-Karten —
eine einfache Vorlage für eigene Sets. Jedes Level wird beim Bau **validiert**
(Flood-Fill: alle Chips/Schlüssel erreichbar, Ausgang nur über den Sockel) —
so können keine unlösbaren Level entstehen:

```
py circuitrunner\make_demo_levels.py     # -> levels/circuit_runner.json
```

## Grafik (64×64, supersampled)

Alle Kacheln werden **programmatisch** erzeugt (`make_tiles.py`, PIL) in ein
Master-Sheet `assets/tiles.png` (16×8 Zellen à 64 px; intern 4× supersampled
und kantengeglättet gezeichnet — detailliert/farbig statt flach 8-bit), in dem
die **Zellen-Position dem Tile-Code entspricht** (0x00–0x7F). Die Engine
zeichnet jede Kachel/Figur per `DRAWIMAGEPART(sheet, code)`.

```
py circuitrunner\make_tiles.py
```

Zusätzlich wird `assets/tiles.gbsprite` exportiert (**im Sprite-Editor
`gbsprites` zu öffnen und bearbeiten** — jede Kachel ein benannter Frame). Die
HUD-Icons zeichnet die Engine direkt aus dem Sheet (`DRAWIMAGEPARTEX`), skaliert
in nativer Auflösung — keine separaten Icon-Dateien.

Gerendert wird zweischichtig: das Spielfeld kommt pixelgenau aus einem
Render-Target (ganzzahlig hochskaliert), HUD/Menü-Text und Icons dagegen in
**nativer Bildschirmauflösung** (TTF-Font `assets/font.ttf`, Skalierung via
`TEXTROT`) — dadurch ist die Schrift gestochen scharf statt klobig.

## Levelformat (JSON-Set)

```jsonc
{ "name": "Circuit Runner", "ruleset": "ms|lynx",
  "levels": [ {
    "title": "...", "number": 1, "time": 0, "chips": 11,
    "hint": "...", "password": "ABCD",
    "width": 32, "height": 32,
    "upper": "<2048 Hex-Zeichen>",   // Ober-Layer: 1024 Tile-Codes
    "lower": "<2048 Hex-Zeichen>",   // Terrain unter Figuren/Blöcken
    "traps":   "bx,by,tx,ty;...",    // Knopf -> Falle
    "cloners": "bx,by,mx,my;...",    // Knopf -> Klon-Maschine
    "monsters":"x,y;..."             // Bewegungsreihenfolge
  } ] }
```

Die Tile-Codes (0x00–0x6F) sind identisch zum originalen Chip's-Challenge-
Objektsatz (siehe `assets/tiles.json` für die Namensliste).

## Mechanik

Voller Satz: Chips + Sockel + Ausgang · 4 Schlüssel & Türen (grün bleibt
erhalten) · Wasser (Flossen) · Feuer (Feuer-Stiefel) · Eis + Eis-Ecken
(Schlittschuhe) · Force-Böden + Zufalls-Force (Saug-Stiefel, Eingabe-Override) ·
schiebbare Blöcke (Block → Wasser = Dreck, → Bombe = Explosion) · Bomben · dünne
Wände & Eckwände · blaue Schein-/Echt-Blöcke · verdeckte Wände · Pass-once ·
Dieb · Teleporter · Toggle-Wände + grüner Knopf · Tanks + blauer Knopf · Cloner +
roter Knopf · Fallen.

**9 Monstertypen** mit eigener KI: Käfer (linke Wand), Paramecium (rechte Wand),
Gleiter, Feuerball, Ball (Bounce), Tank, Walker (Zufall bei Block), Blob
(Zufall), Teeth (Verfolger). Teeth & Blob bewegen sich halb so schnell.

## Dateien

| Datei | Zweck |
|---|---|
| `circuitrunner.gb` | die Spiel-Engine (GameBasic) |
| `make_tiles.py` | Tileset-Generator → `assets/tiles.png` + `.gbsprite` |
| `convert_dat.py` | `.dat`/`.ccl` → JSON-Set (echte Fansite-Level) |
| `make_demo_levels.py` | ASCII → `levels/circuit_runner.json` (5 Demos) |
| `levels/*.json` | Level-Sets (im Menü wählbar) |

Tests: `tests/test_circuitrunner.py` (Konverter-Round-Trip + Demo-Schema).

## Grenzen / Ideen

- Monster-Bewegungsreihenfolge ist Lese-Reihenfolge statt der `monsters`-Liste (kann bei sehr präzisen Puzzles abweichen).
- Kein Passwort-Eingang / Level-Sprung per Passwort, kein Highscore/Save (Fortschritt nicht persistiert).
- Monster bewegen sich mit halbem Tempo (`MON_EVERY=2`); Eis-/Force-Sliding läuft auf dieser Taktung statt voller CC-Geschwindigkeit.

## Lizenz / Hinweis

Grafik und Code sind eigenständig. „Chip's Challenge" ist eine eingetragene
Marke der jeweiligen Rechteinhaber; dieses Projekt ist ein unabhängiger Klon
und nicht damit verbunden. Heruntergeladene Levelsets gehören ihren Autoren.
