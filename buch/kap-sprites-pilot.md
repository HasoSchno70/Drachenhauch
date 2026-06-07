# Kapitel — Sprites zeichnen und ins Spiel bringen

Bisher war unser Raumschiff ein gelber Kasten. Funktioniert — sieht aber nicht
nach Galaga aus. In diesem Kapitel machen wir aus den Kästchen echte **Sprites**:
kleine Pixel-Art-Bilder, die du **selbst im mitgelieferten Sprite-Editor
zeichnest** und dann mit wenigen Zeilen ins Spiel holst.

Am Ende fliegt ein richtiges Schiff über den Bildschirm, ein Gegner schlägt mit
den Flügeln (eine **Animation** aus zwei Bildern), und ein Schuss steigt nach
oben.

> **Du brauchst nichts zu malen können.** Die fertigen Sprites liegen schon im
> Buch (`buch/assets/sprites/`). Du kannst sie sofort benutzen **und** sie im
> Editor öffnen, um sie nach deinem Geschmack umzufärben. Wer selbst zeichnen
> will, bekommt hier die Schritt-für-Schritt-Anleitung.

## Lernziele

Nach diesem Kapitel:

- weißt du, was ein **Sprite** und ein **Sprite-Sheet** ist
- hast du im **`gbsprites`**-Editor ein Sprite gezeichnet und als PNG exportiert
- kennst du eine **Animation** aus mehreren Frames
- holst du Bilder mit `LOADIMAGE` ins Spiel und zeichnest sie mit `DRAWIMAGE`
- baust du aus einem Sheet ein animiertes `SPRITE` (`SPRITE_NEW` /
  `SPRITE_ADD_ANIM` / `SPRITE_PLAY` / `SPRITE_UPDATE` / `SPRITE_DRAW`)

## Was ist ein Sprite?

Ein **Sprite** ist ein kleines Bild, das du im Spiel bewegst — das Schiff, ein
Gegner, ein Schuss. Pixel-Art heißt: das Bild ist bewusst klein (z. B. 16×16
Pixel) und jeder Pixel wird einzeln gesetzt. Das ist genau der Look der
Arcade-Klassiker.

Eine **Animation** ist eine Reihe solcher Bilder, die nacheinander gezeigt
werden — wie ein Daumenkino. Liegen die einzelnen Bilder (die **Frames**)
nebeneinander in *einer* Datei, nennt man das ein **Sprite-Sheet**:

```
[ Frame 0 ][ Frame 1 ]      <- bee.png: zwei 16x16-Frames nebeneinander (32x16)
```

## Schritt 1 — Den Sprite-Editor öffnen

GameBasic bringt einen eigenen Pixel-Art-Editor mit: **`gbsprites`**. Starte ihn
über den Editor (Toolbar / Werkzeuge) oder von der Kommandozeile:

```
.venv\Scripts\python.exe gbrun.py --sprites
```

Du kannst auch direkt eine vorhandene Datei öffnen — probier gleich unser
Schiff aus, um zu sehen, wie es aufgebaut ist:

```
.venv\Scripts\python.exe gbrun.py --sprites buch\assets\sprites\player.gbsprite
```

## Schritt 2 — Das Schiff zeichnen

Wir zeichnen das Spielerschiff in **16×16 Pixeln**.

1. **Neues Sprite** anlegen, Größe **16×16**.
2. **Raster** einschalten (Tile-Preview/Grid) — so triffst du die Pixel genau.
3. In der **Palette** eine Farbe wählen, mit dem **Stift** Pixel setzen. Mit dem
   **Radierer** korrigierst du, mit der **Pipette** nimmst du eine vorhandene
   Farbe auf.

Als Vorlage dient diese Pixel-Karte. Jedes Zeichen ist ein Pixel; `.` bleibt
leer (durchsichtig):

```
................     W = Weiss   (Rumpf)
.......WW.......     C = Cyan    (Rumpf)
......WWWW......     Y = Gelb    (Cockpit)
......WCCW......     R = Rot     (Triebwerk)
......WCCW......
.....WWCCWW.....
.....WCYYCW.....
....WWCYYCWW....
...WWCCCCCCWW...
..WW.WCCCCW.WW..
..W..WRCCRW..W..
.....WRCCRW.....
......RRRR......
.......RR.......
................
```

> **Tipp Symmetrie:** Schiffe sind links/rechts spiegelsymmetrisch. Schalte im
> Editor die **X-Symmetrie** ein — dann malst du nur eine Hälfte, die andere
> entsteht automatisch. Halbe Arbeit, perfekt gerade.

Wenn es dir gefällt: **Speichern** als `.gbsprite` (dein bearbeitbares Format)
**und** **als PNG-Sheet exportieren** (`player.png`) — das PNG ist, was das Spiel
lädt.

## Schritt 3 — Den Gegner animieren (zwei Frames)

Der Gegner — eine kleine „Bee" — soll mit den Flügeln schlagen. Dafür brauchen
wir **zwei Frames**:

1. **Frame 0** zeichnen (Flügel oben).
2. **Frame hinzufügen** (Frame-Leiste) und **Frame 1** zeichnen (Flügel unten).
   Mit der **Onion-Skin**-Funktion siehst du Frame 0 blass durchscheinen — so
   bewegst du nur die Flügel und der Rest bleibt deckungsgleich.
3. **Als Sheet exportieren** (horizontal) → `bee.png`, ein 32×16-Bild mit beiden
   Frames nebeneinander.

Den Schuss (`bullet.png`, 6×8, ein Frame) zeichnest du genauso schnell.

> **Abkürzung:** Alle drei Sprites liegen fertig in `buch/assets/sprites/`
> (`player.png`, `bee.png`, `bullet.png` + die `.gbsprite`-Originale). Du kannst
> mit den fertigen weitermachen und später eigene zeichnen.

## Schritt 4 — Bilder ins Spiel laden

Ein Bild holst du mit `LOADIMAGE` in eine `IMAGE`-Variable und zeichnest es mit
`DRAWIMAGE(bild, x, y)`. Pfade sind **relativ zur `.gb`-Datei**:

```basic
DIM shipImg AS IMAGE
shipImg = LOADIMAGE("../../assets/sprites/player.png")

' ... in der Spielschleife:
DRAWIMAGE(shipImg, 112, 150)
```

Für ein **statisches** Bild (Schiff, Schuss) ist das schon alles.

## Schritt 5 — Aus dem Sheet ein animiertes Sprite

Für die Bee mit ihren zwei Frames nehmen wir das `sprite`-Modul. Es schneidet
das Sheet in Frames und spielt die Animation ab:

```basic
IMPORT "sprite"

DIM beeImg AS IMAGE
beeImg = LOADIMAGE("../../assets/sprites/bee.png")

DIM bee AS SPRITE
bee = SPRITE_NEW(beeImg, 16, 16)       ' Sheet in 16x16-Frames schneiden
SPRITE_ADD_ANIM(bee, "fly", 0, 1, 6)   ' Animation "fly": Frame 0..1, 6 fps
SPRITE_PLAY(bee, "fly")
SPRITE_SET_POS(bee, 112, 40)

' ... in der Spielschleife:
SPRITE_UPDATE(bee, 16)                 ' ~16 ms vergangen -> Animation laeuft
SPRITE_DRAW(bee)
```

Die wichtigsten Funktionen:

| Funktion | Bedeutung |
|---|---|
| `SPRITE_NEW(bild, fw, fh)` | Sheet in `fw`×`fh`-Frames schneiden, liefert ein `SPRITE` |
| `SPRITE_ADD_ANIM(s, name$, von, bis, fps)` | benannte Animation aus den Frames `von..bis` |
| `SPRITE_PLAY(s, name$)` | Animation (in Schleife) starten |
| `SPRITE_SET_POS(s, x, y)` | Position setzen |
| `SPRITE_UPDATE(s, dt_ms)` | Animation um `dt_ms` Millisekunden weiterdrehen |
| `SPRITE_DRAW(s)` | das aktuelle Frame zeichnen (braucht ein offenes `SCREEN`) |

## Alles zusammen

Das ist der vollständige, lauffähige Stand
([`code/sprites/sprites_im_spiel.gb`](code/sprites/sprites_im_spiel.gb)):

```basic
IMPORT "sprite"
SCREEN(240, 200, "Galaga -- Sprites", 2)

DIM shipImg AS IMAGE
shipImg = LOADIMAGE("../../assets/sprites/player.png")
DIM shotImg AS IMAGE
shotImg = LOADIMAGE("../../assets/sprites/bullet.png")
DIM beeImg AS IMAGE
beeImg = LOADIMAGE("../../assets/sprites/bee.png")

DIM bee AS SPRITE
bee = SPRITE_NEW(beeImg, 16, 16)
SPRITE_ADD_ANIM(bee, "fly", 0, 1, 6)
SPRITE_PLAY(bee, "fly")
SPRITE_SET_POS(bee, 112, 40)

DIM shotY AS INTEGER
shotY = 118

WHILE NOT QUITREQUESTED()
    shotY = shotY - 2
    IF shotY < -8 THEN shotY = 150

    CLS(&H0C1020)
    DRAWIMAGE(shipImg, 112, 150)
    DRAWIMAGE(shotImg, 117, shotY)
    SPRITE_UPDATE(bee, 16)
    SPRITE_DRAW(bee)
    FLIP()
WEND
```

Starte es:

```
.venv\Scripts\python.exe gbrun.py buch\code\sprites\sprites_im_spiel.gb
```

Du siehst das Schiff unten, einen aufsteigenden Schuss und oben den Gegner, der
mit den Flügeln schlägt. **Das ist der Galaga-Look** — und alles davon ist
deine eigene Pixel-Art.

## Übung

1. **Umfärben:** Öffne `player.gbsprite` in `gbsprites` und gib dem Schiff eine
   andere Triebwerksfarbe (z. B. Cyan statt Rot). Exportiere neu und starte das
   Spiel — die Änderung ist sofort sichtbar.
2. **Schneller schlagen:** Ändere in `SPRITE_ADD_ANIM(..., 6)` die `6` auf `12`.
   Wie wirkt der Flügelschlag jetzt?
3. **Eigener Gegner:** Zeichne ein zweites Gegner-Sheet (`bee2.png`) mit einer
   anderen Farbe und lass es neben der ersten Bee fliegen.

## Ausblick

Im nächsten Kapitel hört das Schiff auf, festgenagelt zu sein: wir lesen die
**Tastatur** und bewegen es nach links/rechts. Danach bekommt der Schuss echtes
Leben (eine Bullet-Liste), und die Gegner ordnen sich zur **Galaga-Formation**.
