# Mario-Clone Sprites

Ein kompletter Sprite-Satz für einen Mario-artigen Platformer, gebaut über das
Datenmodell des Sprite-Editors (`gbsprites`). Alle Figuren sind 16×16 Pixel.

## Figuren

| Datei | Frames | Animationen |
|---|---|---|
| `hero` | idle, run0–2, jump, skid, duck, dead | `idle`, `run`, `jump`, `skid`, `duck`, `dead` |
| `goomba` | walk0–1, squash | `walk`, `squash` |
| `koopa` | walk0–1 | `walk` |
| `para` | fly0–1 | `fly` (fliegender Gegner) |

Pro Figur liegen vier Dateien hier:

- **`<name>.gbsprite`** — im Sprite-Editor (`gbsprites <name>.gbsprite`) weiter
  bearbeitbar (Frames, Ebenen, Anim-Bereiche, Dauern).
- **`<name>.png`** — horizontales Sprite-Sheet (Frame-Breite 16) für
  `SPRITE_NEW(sheet, 16, 16)`.
- **`<name>.json`** — Atlas-Manifest für `ATLAS_LOAD` (benannte Frame-Rects).
- **`<name>.gif`** — animierte Vorschau (6× hochskaliert).

`_contact.png` zeigt alle Frames aller Figuren auf einen Blick.

## Neu generieren / anpassen

Die Pixelart ist als ASCII-Raster in `make_sprites.py` notiert (ein Zeichen =
ein Pixel, Palette pro Figur). Bearbeiten und neu bauen:

```
py examples/mario/make_sprites.py
```

Oder die `.gbsprite`-Dateien direkt im Editor öffnen und malen:

```
gbsprites examples/mario/hero.gbsprite
```

## Demo

`mario_demo.gb` zeigt die Sprites in Aktion: Held läuft (Pfeile), springt
(Leertaste/Hoch), bremst und duckt sich; zwei Goombas patrouillieren und lassen
sich per Stomp plattmachen; ein Para schwebt in einer Sinuswelle vorbei.

```
gbrun.py examples/mario/mario_demo.gb
```

## Im eigenen Spiel verwenden

```basic
IMPORT "sprite"
DIM sheet AS IMAGE
sheet = LOADIMAGE("hero.png")
DIM hero AS SPRITE
hero = SPRITE_NEW(sheet, 16, 16)
SPRITE_ADD_ANIM(hero, "idle", 0, 0, 1)
SPRITE_ADD_ANIM(hero, "run",  1, 3, 12)
SPRITE_ADD_ANIM(hero, "jump", 4, 4, 1)
SPRITE_PLAY(hero, "run")
' im Loop:  SPRITE_UPDATE(hero, dt) : SPRITE_DRAW(hero)
```
