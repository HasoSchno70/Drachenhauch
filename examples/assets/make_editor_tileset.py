"""Erzeugt ein 16x16-Tileset fuer den Tilemap-Editor (dhtilemap).

8 Spalten x 4 Reihen = 32 Tiles, 128x64 px, mit Alpha. Pixel-Art-Stil,
deterministisch (fester Seed). Ausgabe: editor_tileset.png neben diesem Skript.

Tile-Reihenfolge (lokale ID = Zeile*8 + Spalte; gid = ID+1):
  Reihe 0  Boden:   grass dirt stone sand water brick wood snow
  Reihe 1  Varianten: grass2 darkdirt cobble gravel deepwater metal log ice
  Reihe 2  Objekte: coin heart star key gem bush flower mushroom
  Reihe 3  Deko:    treetop trunk rock crate barrel sign ladder spikes

Aufruf:  .venv\\Scripts\\python.exe examples\\assets\\make_editor_tileset.py
"""
from __future__ import annotations

import random
from pathlib import Path

from PIL import Image, ImageDraw

TS = 16
COLS, ROWS = 8, 4
rng = random.Random(20260604)

img = Image.new("RGBA", (COLS * TS, ROWS * TS), (0, 0, 0, 0))
d = ImageDraw.Draw(img)


def px(x, y, ox, oy, c):
    img.putpixel((ox + x, oy + y), c)


def fill(ox, oy, c):
    d.rectangle([ox, oy, ox + TS - 1, oy + TS - 1], fill=c)


def noise(ox, oy, base, amt, n=40):
    """Streut leicht aufgehellte/abgedunkelte Pixel ein."""
    r, g, b = base[:3]
    for _ in range(n):
        x = rng.randrange(TS); y = rng.randrange(TS)
        k = rng.randint(-amt, amt)
        img.putpixel((ox + x, oy + y),
                     (max(0, min(255, r + k)), max(0, min(255, g + k)),
                      max(0, min(255, b + k)), 255))


def outline(ox, oy, c):
    d.rectangle([ox, oy, ox + TS - 1, oy + TS - 1], outline=c)


def cell(col, row):
    return (col * TS, row * TS)


# ---------------------------------------------------------- Reihe 0: Boden
def grass(ox, oy, top=(86, 170, 70), body=(120, 90, 55)):
    fill(ox, oy, body)
    noise(ox, oy, body, 18)
    d.rectangle([ox, oy, ox + TS - 1, oy + 4], fill=top)
    noise2 = top
    for _ in range(26):
        x = rng.randrange(TS)
        img.putpixel((ox + x, oy + rng.randint(0, 4)),
                     (noise2[0] - 20, noise2[1] - 20, noise2[2] - 10, 255))
    # Grashalme
    for _ in range(5):
        x = rng.randrange(1, TS - 1)
        img.putpixel((ox + x, oy + 5), (70, 150, 60, 255))


def flat(ox, oy, c, amt=16):
    fill(ox, oy, c)
    noise(ox, oy, c, amt)


def water(ox, oy, c=(58, 110, 200)):
    fill(ox, oy, c)
    for yy in range(TS):
        for xx in range(TS):
            if (xx + yy) % 6 == 0:
                img.putpixel((ox + xx, oy + yy), (c[0] + 30, c[1] + 30, c[2] + 25, 255))


def bricks(ox, oy, c=(170, 70, 55), mort=(90, 40, 30)):
    fill(ox, oy, c)
    noise(ox, oy, c, 12)
    for yy in (0, 8):
        d.line([ox, oy + yy, ox + TS - 1, oy + yy], fill=mort)
    # versetzte Fugen
    d.line([ox + 8, oy, ox + 8, oy + 7], fill=mort)
    d.line([ox + 0, oy + 8, ox + 0, oy + 15], fill=mort)
    d.line([ox + 12, oy + 8, ox + 12, oy + 15], fill=mort)


def wood(ox, oy, c=(150, 110, 65)):
    fill(ox, oy, c)
    noise(ox, oy, c, 14)
    for yy in (3, 7, 11, 15):
        d.line([ox, oy + yy, ox + TS - 1, oy + yy], fill=(c[0] - 35, c[1] - 30, c[2] - 20))


grass(*cell(0, 0))
flat(*cell(1, 0), c=(135, 95, 58))                 # dirt
flat(*cell(2, 0), c=(130, 130, 138))               # stone
flat(*cell(3, 0), c=(220, 200, 130))               # sand
water(*cell(4, 0))                                 # water
bricks(*cell(5, 0))                                # brick
wood(*cell(6, 0))                                  # wood
flat(*cell(7, 0), c=(235, 240, 248), amt=10)       # snow

# ---------------------------------------------------------- Reihe 1: Varianten
grass(*cell(0, 1), top=(70, 150, 60), body=(105, 78, 48))   # grass2
flat(*cell(1, 1), c=(98, 70, 42))                  # dark dirt
# cobble
ox, oy = cell(2, 1); flat(ox, oy, c=(120, 120, 128))
for cx in range(0, TS, 5):
    for cy in range(0, TS, 5):
        d.ellipse([ox + cx, oy + cy, ox + cx + 4, oy + cy + 4], outline=(80, 80, 88))
flat(*cell(3, 1), c=(150, 145, 130), amt=22)       # gravel
water(*cell(4, 1), c=(36, 70, 150))                # deep water
# metal
ox, oy = cell(5, 1); flat(ox, oy, c=(140, 145, 155), amt=8)
outline(ox, oy, (90, 95, 105))
for p in ((2, 2), (13, 2), (2, 13), (13, 13)):
    img.putpixel((ox + p[0], oy + p[1]), (60, 65, 75, 255))
# log (end grain)
ox, oy = cell(6, 1); fill(ox, oy, (150, 110, 65))
d.ellipse([ox + 2, oy + 2, ox + 13, oy + 13], fill=(170, 130, 80), outline=(110, 80, 45))
d.ellipse([ox + 5, oy + 5, ox + 10, oy + 10], outline=(110, 80, 45))
flat(*cell(7, 1), c=(180, 220, 240), amt=8)        # ice

# ---------------------------------------------------------- Reihe 2: Objekte
def icon_bg(ox, oy):
    pass  # transparent


def coin(ox, oy):
    d.ellipse([ox + 3, oy + 2, ox + 12, oy + 13], fill=(245, 205, 60), outline=(180, 140, 20))
    d.line([ox + 7, oy + 4, ox + 7, oy + 11], fill=(180, 140, 20))


def heart(ox, oy):
    c = (220, 60, 70)
    d.ellipse([ox + 3, oy + 3, ox + 8, oy + 8], fill=c)
    d.ellipse([ox + 8, oy + 3, ox + 13, oy + 8], fill=c)
    d.polygon([(ox + 3, oy + 6), (ox + 13, oy + 6), (ox + 8, oy + 13)], fill=c)


def star(ox, oy):
    c = (250, 220, 90)
    cx, cy = ox + 8, oy + 8
    import math
    pts = []
    for i in range(10):
        ang = -math.pi / 2 + i * math.pi / 5
        r = 6 if i % 2 == 0 else 2.6
        pts.append((cx + r * math.cos(ang), cy + r * math.sin(ang)))
    d.polygon(pts, fill=c, outline=(200, 160, 30))


def key(ox, oy):
    c = (230, 200, 90)
    d.ellipse([ox + 2, oy + 5, ox + 8, oy + 11], outline=c, width=2)
    d.line([ox + 8, oy + 8, ox + 13, oy + 8], fill=c, width=2)
    d.line([ox + 12, oy + 8, ox + 12, oy + 11], fill=c)


def gem(ox, oy):
    c = (90, 210, 200)
    d.polygon([(ox + 8, oy + 2), (ox + 13, oy + 7), (ox + 8, oy + 14), (ox + 3, oy + 7)],
              fill=c, outline=(40, 130, 130))
    d.line([ox + 3, oy + 7, ox + 13, oy + 7], fill=(40, 130, 130))


def bush(ox, oy):
    c = (60, 140, 60)
    for cx, cy, r in ((6, 9, 4), (10, 9, 4), (8, 6, 4)):
        d.ellipse([ox + cx - r, oy + cy - r, ox + cx + r, oy + cy + r], fill=c)


def flower(ox, oy):
    d.line([ox + 8, oy + 8, ox + 8, oy + 14], fill=(60, 140, 60))
    for dx, dy in ((-3, 0), (3, 0), (0, -3), (0, 3)):
        d.ellipse([ox + 8 + dx - 2, oy + 6 + dy - 2, ox + 8 + dx + 2, oy + 6 + dy + 2],
                  fill=(230, 110, 170))
    d.ellipse([ox + 6, oy + 4, ox + 10, oy + 8], fill=(250, 220, 90))


def mushroom(ox, oy):
    d.rectangle([ox + 6, oy + 8, ox + 9, oy + 13], fill=(230, 220, 200))
    d.ellipse([ox + 3, oy + 4, ox + 12, oy + 10], fill=(210, 60, 60))
    for p in ((6, 6), (9, 6), (7, 8)):
        img.putpixel((ox + p[0], oy + p[1]), (245, 245, 245, 255))


for fn, c in ((coin, 0), (heart, 1), (star, 2), (key, 3),
              (gem, 4), (bush, 5), (flower, 6), (mushroom, 7)):
    fn(*cell(c, 2))

# ---------------------------------------------------------- Reihe 3: Deko
def treetop(ox, oy):
    c = (50, 130, 55)
    d.ellipse([ox + 1, oy + 1, ox + 14, oy + 14], fill=c, outline=(35, 95, 40))
    noise(ox, oy, c, 18, n=20)


def trunk(ox, oy):
    fill(ox, oy, (0, 0, 0, 0))
    d.rectangle([ox + 5, oy, ox + 10, oy + 15], fill=(120, 80, 45), outline=(85, 55, 30))


def rock(ox, oy):
    d.polygon([(ox + 3, oy + 13), (ox + 5, oy + 6), (ox + 9, oy + 4),
               (ox + 13, oy + 9), (ox + 12, oy + 13)],
              fill=(140, 140, 148), outline=(90, 90, 98))


def crate(ox, oy):
    d.rectangle([ox + 1, oy + 1, ox + 14, oy + 14], fill=(165, 120, 65), outline=(95, 65, 35))
    d.line([ox + 1, oy + 1, ox + 14, oy + 14], fill=(95, 65, 35))
    d.line([ox + 14, oy + 1, ox + 1, oy + 14], fill=(95, 65, 35))


def barrel(ox, oy):
    d.rounded_rectangle([ox + 3, oy + 1, ox + 12, oy + 14], radius=3,
                        fill=(150, 100, 55), outline=(95, 60, 30))
    for yy in (4, 8, 11):
        d.line([ox + 3, oy + yy, ox + 12, oy + yy], fill=(95, 60, 30))


def sign(ox, oy):
    d.rectangle([ox + 7, oy + 6, ox + 8, oy + 15], fill=(110, 75, 40))
    d.rectangle([ox + 2, oy + 2, ox + 13, oy + 8], fill=(175, 135, 80), outline=(110, 75, 40))


def ladder(ox, oy):
    for x in (5, 10):
        d.line([ox + x, oy, ox + x, oy + 15], fill=(150, 110, 65))
    for yy in (2, 7, 12):
        d.line([ox + 5, oy + yy, ox + 10, oy + yy], fill=(150, 110, 65))


def spikes(ox, oy):
    c = (170, 175, 185)
    for i in range(4):
        bx = ox + i * 4
        d.polygon([(bx, oy + 15), (bx + 2, oy + 5), (bx + 4, oy + 15)],
                  fill=c, outline=(110, 115, 125))


for fn, c in ((treetop, 0), (trunk, 1), (rock, 2), (crate, 3),
              (barrel, 4), (sign, 5), (ladder, 6), (spikes, 7)):
    fn(*cell(c, 3))

out = Path(__file__).resolve().parent / "editor_tileset.png"
img.save(out)
print(f"geschrieben: {out}  ({img.width}x{img.height}, {COLS*ROWS} Tiles a {TS}px)")
