# Plattformer-Sprites (32x32, "Twilight"-Thema)

Ein kompletter, detailreicher Sprite- und Tile-Satz für einen generischen Plattformer (eigenstaendiges "Twilight"-Thema, nicht an Nintendo angelehnt), prozedural gebaut über das Datenmodell des Sprite-Editors
(`drachenhauch.spriteeditor.document.SpriteDoc`). Alle Sprites sind **32×32 Pixel**.

## Master-Spritesheet

- **`sheet.png`** + **`sheet.json`** — EIN Spritesheet mit allen 44 benannten
  Sprites in einem 10×?-Raster, direkt über `ATLAS_LOAD("sheet.json")` +
  `ATLAS_DRAW(atlas, "<name>", x, y)` nutzbar.
- **`_contact.png`** — Übersicht aller Sprites mit Namen.

## Figuren (animiert)

Pro Figur: `.gbsprite` (im Editor `dhsprites` bearbeitbar) + `.png`
(horizontaler Strip für `SPRITE_NEW(sheet, 32, 32)`) + `.gif` (Vorschau).
Im Master-Sheet heißen die Frames `<figur>_<frame>`.

| Datei | Frames | Animationen |
|---|---|---|
| `player` | idle, run0–2, jump, skid, duck, dead | `idle`, `run`, `jump`, `skid`, `duck`, `dead` |
| `walker` | walk0–1, squash | `walk`, `squash` |
| `guard` | walk0–1, shell | `walk`, `shell` |
| `snapper` | open, closed | `bite` |
| `glider` | fly0–1 | `fly` (fliegender Gegner) |
| `coin` | c0–c3 | `spin` |
| `itembox` | q0–q2 | `idle` (Schimmer) |
| `water` | w0–w1 | `wave` |

## Tiles, Items & Deko (statisch, im Master-Sheet)

- **Boden/Bausteine:** `ground`, `brick`, `solid` (Stein), `itembox_used` (benutzter Block)
- **Röhre** (2×2 zusammensetzbar): `tube_tl`, `tube_tr`, `tube_bl`, `tube_br`
- **Wasser:** `water_body` (+ animierte Oberfläche `water_w0/w1`)
- **Items:** `grow`, `bolt`, `shard`, `life`
- **Deko:** `cloud`, `bush`, `hill`, `flag`

## Neu generieren / anpassen

Alle Sprites werden in `make_sprites.py` mit einer kleinen Pixel-Canvas
prozedural gezeichnet (harte Kanten, feste Palette). Ändern und neu bauen:

```
py examples/platformer/make_sprites.py
```

Oder eine `.gbsprite` direkt im Editor öffnen und pixeln:

```
dhsprites examples/platformer/hero.gbsprite
```

## Demo

`platformer_demo.gb` baut daraus ein kleines Level: Held läuft/springt/duckt sich
(Flip), ein Walker patrouilliert und lässt sich per Stomp plattmachen; das
Level (Boden, Röhre, Brick/?-Block, Wasser, animierte Münzen, Wolken/Busch/
Hügel) kommt aus dem Master-Atlas `sheet.json`.

```
dhrun.py examples/platformer/platformer_demo.gb
```

## Im eigenen Spiel verwenden

```basic
IMPORT "sprite"
' Animierter Held aus dem Strip:
DIM hero AS SPRITE
hero = SPRITE_NEW(LOADIMAGE("player.png"), 32, 32)
SPRITE_ADD_ANIM(hero, "run", 1, 3, 12)
SPRITE_PLAY(hero, "run")

' Tiles aus dem Master-Atlas:
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("sheet.json")
ATLAS_DRAW(atlas, "ground", 0, 224)
ATLAS_DRAW(atlas, "pipe_tl", 256, 160)
```
