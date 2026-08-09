"""Sprite-Generator fuer einen generischen Plattformer-Satz -- 32x32, prozedural.

Eigenstaendiges "Twilight"-Thema (violetter Held, schiefer-violette Gegner,
Stahl-Cyan-Roehren, Kristall-Item-Boxen) -- bewusst NICHT an Nintendo-Marken/
-Figuren angelehnt, damit der Satz kommerziell nutzbar ist.

Zeichnet Spieler, Gegner, Tiles, Items und Deko mit einer kleinen Pixel-Canvas
(harte Kanten, begrenzte Palette = Pixelart-Look) und exportiert:
- pro animierter Figur eine `.dhsprite` (im Editor `dhsprites` bearbeitbar)
  + `.gif`-Vorschau,
- ein **Master-Spritesheet** `sheet.png` + `sheet.json` (ATLAS_LOAD-Manifest)
  mit ALLEN benannten Sprites in einem Raster,
- `_contact.png` zur Begutachtung.

Aufruf:  py examples/mario/make_sprites.py
"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

from PIL import Image

_ROOT = Path(__file__).resolve().parent.parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from drachenhauch.spriteeditor.document import SpriteDoc, Frame, Anim  # noqa: E402

OUT = Path(__file__).resolve().parent
S = 32  # Sprite-Kantenlaenge

# ----------------------------------------------------------------- Farben
T = (0, 0, 0, 0)
OL = (26, 18, 28, 255)
# Eigenstaendige "Twilight"-Palette (bewusst NICHT die Mario-Farben):
# Held = violette Kapuze + tuerkise Montur, Gegner = schiefer-violett,
# Panzer/Roehre = Stahl-Cyan, Item-Box = Kristall. Umbau fuer kommerzielle
# Nutzung -- siehe examples/ASSET-CREDITS.md.
RED = (150, 72, 200, 255)        # frueher Rot -> Violett (Kapuze/Arme/Pflanze)
RED_D = (98, 44, 138, 255)
RED_H = (196, 138, 234, 255)
SKIN = (252, 206, 168, 255)
SKIN_D = (212, 156, 116, 255)
BLUE = (34, 162, 156, 255)        # frueher Blau -> Tuerkis (Montur)
BLUE_D = (18, 108, 104, 255)
BLUE_H = (104, 212, 202, 255)
BROWN = (108, 94, 132, 255)       # frueher Braun -> Schiefer-Violett (Gegner/Haar)
BROWN_D = (66, 56, 90, 255)
BROWN_L = (152, 138, 178, 255)
STEEL = (74, 172, 178, 255)       # Panzer/Roehre (statt Gruen)
STEEL_D = (38, 112, 120, 255)
STEEL_H = (148, 224, 220, 255)
CRYST = (96, 196, 230, 255)       # Item-Box-Kristall (statt Gelb)
CRYST_D = (44, 126, 170, 255)
CRYST_H = (190, 238, 252, 255)
YEL = (250, 208, 72, 255)
YEL_D = (196, 150, 40, 255)
WHITE = (250, 250, 250, 255)
GREEN = (72, 184, 72, 255)
GREEN_D = (36, 116, 44, 255)
GREEN_H = (140, 224, 120, 255)
ORANGE = (240, 144, 36, 255)
ORANGE_D = (180, 96, 16, 255)
GRAY = (180, 180, 192, 255)
GRAY_D = (112, 112, 128, 255)
GRAY_H = (224, 224, 232, 255)
WATER = (60, 132, 244, 255)
WATER_D = (36, 96, 200, 255)
WATER_H = (150, 200, 252, 255)
BRICK = (196, 96, 44, 255)
BRICK_D = (132, 60, 26, 255)
BRICK_M = (96, 44, 18, 255)       # Mortar
DIRT = (196, 116, 56, 255)
DIRT_D = (150, 82, 36, 255)


class Canvas:
    """Winzige Pixel-Zeichenflaeche (keine Kantenglaettung)."""

    def __init__(self, w: int = S, h: int = S):
        self.im = Image.new("RGBA", (w, h), T)
        self.p = self.im.load()
        self.w, self.h = w, h

    def set(self, x, y, c):
        x, y = int(x), int(y)
        if 0 <= x < self.w and 0 <= y < self.h and c[3]:
            self.p[x, y] = c

    def rect(self, x0, y0, x1, y1, c):
        for y in range(int(y0), int(y1) + 1):
            for x in range(int(x0), int(x1) + 1):
                self.set(x, y, c)

    def hline(self, x0, x1, y, c):
        for x in range(int(x0), int(x1) + 1):
            self.set(x, y, c)

    def vline(self, x, y0, y1, c):
        for y in range(int(y0), int(y1) + 1):
            self.set(x, y, c)

    def disc(self, cx, cy, r, c):
        for y in range(int(cy - r), int(cy + r) + 1):
            for x in range(int(cx - r), int(cx + r) + 1):
                if (x - cx) ** 2 + (y - cy) ** 2 <= r * r + r * 0.4:
                    self.set(x, y, c)

    def bevel(self, x0, y0, x1, y1, base, light, dark):
        self.rect(x0, y0, x1, y1, base)
        self.hline(x0, x1, y0, light)
        self.vline(x0, y0, y1, light)
        self.hline(x0, x1, y1, dark)
        self.vline(x1, y0, y1, dark)

    def outline(self, c=OL):
        src = self.im.copy()
        sp = src.load()
        for y in range(self.h):
            for x in range(self.w):
                if sp[x, y][3] == 0:
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < self.w and 0 <= ny < self.h and sp[nx, ny][3] > 0:
                            self.p[x, y] = c
                            break


# ============================================================ HELD
def draw_hero(leg="idle", arm_up=False, lean=0, duck=False, dead=False):
    c = Canvas()
    cx = 16 + lean
    if duck:
        _hero_head(c, cx, 9, dead=dead)
        _hero_body(c, cx, 18, duck=True)
        c.rect(cx - 7, 26, cx - 1, 28, BLUE); c.rect(cx + 1, 26, cx + 7, 28, BLUE)
        _shoe(c, cx - 9, 28); _shoe(c, cx + 3, 28)
        c.outline()
        return c.im
    top = 4
    _hero_head(c, cx, top, dead=dead)
    _hero_body(c, cx, top + 9)
    if arm_up:
        _arm(c, cx + 7, top + 8, up=True); _arm(c, cx - 10, top + 12, up=False)
    elif dead:
        _arm(c, cx + 8, top + 9, up=True); _arm(c, cx - 11, top + 9, up=True)
    elif lean > 0:
        _arm(c, cx - 12, top + 11, up=False); _arm(c, cx + 7, top + 13, up=False)
    else:
        _arm(c, cx - 10, top + 12, up=False); _arm(c, cx + 7, top + 12, up=False)
    _hero_legs(c, cx, top + 20, leg)
    c.outline()
    return c.im


def _hero_head(c, cx, y, dead=False):
    c.rect(cx - 7, y + 2, cx - 1, y + 3, RED)
    c.rect(cx - 7, y + 1, cx + 5, y + 4, RED)
    c.rect(cx - 5, y, cx + 4, y + 1, RED)
    c.hline(cx - 3, cx + 2, y - 1, RED)
    c.rect(cx - 6, y + 1, cx + 4, y + 1, RED_H)
    c.disc(cx, y + 2, 2, WHITE)
    c.set(cx, y + 2, RED); c.set(cx - 1, y + 1, RED); c.set(cx + 1, y + 3, RED)
    c.rect(cx - 5, y + 4, cx + 5, y + 11, SKIN)
    c.rect(cx + 4, y + 5, cx + 5, y + 11, SKIN_D)
    c.rect(cx - 6, y + 4, cx - 4, y + 10, BROWN)
    c.rect(cx + 5, y + 5, cx + 6, y + 8, BROWN)
    if dead:
        c.set(cx, y + 6, OL); c.set(cx + 1, y + 5, OL)
        c.set(cx + 1, y + 6, OL); c.set(cx, y + 5, OL)
    else:
        c.rect(cx, y + 5, cx + 1, y + 7, WHITE)
        c.vline(cx + 1, y + 5, y + 7, OL)
    c.disc(cx + 4, y + 8, 2, SKIN); c.disc(cx + 5, y + 8, 1, SKIN_D)
    # Kein Schnurrbart (Differenzierung): nur ein kleiner, neutraler Mund.
    c.hline(cx - 2, cx + 1, y + 10, SKIN_D)


def _hero_body(c, cx, y, duck=False):
    h = 5 if duck else 9
    c.rect(cx - 7, y, cx + 7, y + 2, RED)
    c.rect(cx - 6, y + 1, cx + 6, y + h, BLUE)
    c.rect(cx - 6, y + 1, cx - 5, y + h, BLUE_D)
    c.rect(cx + 5, y + 1, cx + 6, y + h, BLUE_H)
    c.vline(cx - 3, y - 1, y + 2, BLUE); c.vline(cx + 3, y - 1, y + 2, BLUE)
    c.disc(cx - 3, y + 3, 1, YEL); c.disc(cx + 3, y + 3, 1, YEL)


def _arm(c, x, y, up=False):
    if up:
        c.rect(x, y - 4, x + 2, y, RED); c.disc(x + 1, y - 5, 2, SKIN)
    else:
        c.rect(x, y, x + 2, y + 4, RED); c.disc(x + 1, y + 5, 2, SKIN)


def _shoe(c, x, y):
    c.rect(x, y, x + 5, y + 1, BROWN); c.rect(x, y + 1, x + 6, y + 2, BROWN_D)


def _hero_legs(c, cx, y, leg):
    if leg == "stride_l":
        c.rect(cx - 7, y, cx - 3, y + 4, BLUE); _shoe(c, cx - 9, y + 4)
        c.rect(cx + 1, y, cx + 4, y + 3, BLUE); _shoe(c, cx + 2, y + 3)
    elif leg == "stride_r":
        c.rect(cx - 4, y, cx - 1, y + 3, BLUE); _shoe(c, cx - 7, y + 3)
        c.rect(cx + 3, y, cx + 7, y + 4, BLUE); _shoe(c, cx + 4, y + 4)
    elif leg == "tuck":
        c.rect(cx - 6, y, cx - 2, y + 2, BLUE); _shoe(c, cx - 9, y + 1)
        c.rect(cx + 2, y, cx + 6, y + 2, BLUE); _shoe(c, cx + 4, y + 1)
    else:
        c.rect(cx - 5, y, cx - 1, y + 4, BLUE); _shoe(c, cx - 8, y + 4)
        c.rect(cx + 1, y, cx + 5, y + 4, BLUE); _shoe(c, cx + 3, y + 4)


# ============================================================ GOOMBA
def draw_goomba(step=0, squash=False):
    c = Canvas()
    if squash:
        c.disc(16, 24, 13, BROWN)
        c.rect(3, 24, 28, 27, BROWN)
        c.rect(3, 26, 28, 28, BROWN_D)
        c.rect(8, 22, 12, 25, WHITE); c.rect(19, 22, 23, 25, WHITE)
        c.rect(10, 22, 12, 24, OL); c.rect(19, 22, 21, 24, OL)
        c.outline()
        return c.im
    c.disc(16, 12, 11, BROWN)              # Pilzkopf
    c.rect(6, 12, 26, 17, BROWN)
    c.rect(6, 15, 26, 18, BROWN_D)         # Unterkante Schatten
    c.disc(16, 9, 9, BROWN_L)              # heller Scheitel
    # Augen
    c.rect(8, 11, 13, 16, WHITE); c.rect(19, 11, 24, 16, WHITE)
    c.rect(10, 12, 12, 15, OL); c.rect(19, 12, 21, 15, OL)
    # boese Brauen
    c.rect(8, 9, 14, 11, BROWN_D); c.rect(18, 9, 24, 11, BROWN_D)
    c.set(13, 11, BROWN_D); c.set(19, 11, BROWN_D)
    # Mund
    c.hline(12, 20, 18, OL); c.set(11, 17, OL); c.set(21, 17, OL)
    # Fuesse (Schritt)
    if step == 0:
        c.rect(6, 24, 13, 27, BROWN_D); c.rect(19, 24, 26, 27, BROWN_D)
    else:
        c.rect(8, 24, 15, 27, BROWN_D); c.rect(17, 24, 24, 27, BROWN_D)
    c.outline()
    return c.im


# ============================================================ KOOPA
def draw_koopa(step=0, shell_only=False):
    c = Canvas()
    if shell_only:
        c.disc(16, 18, 12, STEEL)
        c.disc(16, 18, 12, STEEL_D)
        c.disc(16, 19, 9, STEEL)
        c.rect(4, 18, 28, 24, YEL); c.rect(4, 22, 28, 25, YEL_D)
        for sx in (10, 16, 22):
            c.disc(sx, 16, 2, STEEL_D)
        c.outline()
        return c.im
    # Kopf
    c.disc(22, 8, 5, YEL); c.rect(20, 6, 26, 11, YEL)
    c.rect(24, 5, 27, 8, YEL_D)
    c.rect(23, 6, 24, 8, WHITE); c.vline(24, 6, 8, OL)   # Auge
    c.rect(26, 9, 28, 11, ORANGE)                        # Schnabel
    # Panzer
    c.disc(13, 17, 10, STEEL)
    c.disc(13, 17, 10, STEEL_D)
    c.disc(13, 18, 7, STEEL)
    for sx, sy in ((9, 14), (16, 14), (12, 19)):
        c.disc(sx, sy, 2, STEEL_H)
    c.rect(4, 20, 22, 25, YEL); c.rect(4, 23, 22, 26, YEL_D)   # Bauchrand
    # Fuesse
    if step == 0:
        c.rect(6, 26, 11, 29, ORANGE_D); c.rect(15, 26, 20, 29, ORANGE_D)
    else:
        c.rect(8, 26, 13, 29, ORANGE_D); c.rect(16, 26, 21, 29, ORANGE_D)
    # Arm
    c.disc(22, 18, 3, YEL); c.disc(22, 18, 3, YEL_D)
    c.outline()
    return c.im


# ============================================================ PIRANHA
def draw_piranha(open_=True):
    c = Canvas()
    # Stiel
    c.rect(14, 16, 18, 31, GREEN); c.rect(14, 16, 15, 31, GREEN_D)
    c.rect(17, 16, 18, 31, GREEN_H)
    c.disc(11, 22, 2, GREEN_H); c.disc(21, 26, 2, GREEN_H)   # Blaetter
    # Kopf (rot mit weissen Punkten)
    c.disc(16, 10, 9, RED)
    c.disc(16, 10, 9, RED_D)
    c.disc(16, 9, 7, RED)
    for dx, dy in ((-4, -3), (4, -3), (-5, 2), (5, 2), (0, -5)):
        c.disc(16 + dx, 9 + dy, 1, WHITE)
    if open_:
        c.rect(10, 9, 22, 13, WHITE)         # offenes Maul
        for tx in range(11, 22, 3):
            c.vline(tx, 9, 10, RED_D); c.vline(tx, 12, 13, RED_D)  # Zaehne
        c.rect(11, 10, 21, 12, (40, 12, 12, 255))
    else:
        c.hline(10, 22, 11, RED_D)           # geschlossen
    c.outline()
    return c.im


# ============================================================ PARA (Flieger)
def draw_para(flap=0):
    c = Canvas()
    wy = 6 if flap == 0 else 12
    for wx, d in ((2, 1), (24, -1)):          # Fluegel
        c.disc(wx + 3, wy + 2, 4, WHITE)
        c.rect(wx, wy, wx + 6, wy + 4, WHITE)
        c.set(wx + 3, wy + 2, GRAY)
    base = draw_goomba(step=flap)
    c.im.alpha_composite(base)
    c.outline()
    return c.im


# ============================================================ TILES
def tile_ground():
    c = Canvas()
    c.rect(0, 0, 31, 31, DIRT)
    c.rect(0, 0, 31, 6, GREEN)               # Gras oben
    c.rect(0, 5, 31, 7, GREEN_D)
    for x in range(2, 32, 6):
        c.vline(x, 0, 4, GREEN_H)
    for (x, y) in ((6, 12), (20, 16), (12, 24), (26, 26), (3, 20)):
        c.disc(x, y, 1, DIRT_D)
    c.rect(0, 0, 0, 31, GREEN_H); c.rect(0, 8, 0, 31, DIRT_D)
    c.rect(31, 0, 31, 31, BROWN_D)
    return c.im


def tile_brick():
    c = Canvas()
    c.rect(0, 0, 31, 31, BRICK)
    c.rect(0, 0, 31, 1, (230, 140, 90, 255))
    for y in (0, 8, 16, 24):                 # Mortar-Reihen
        c.hline(0, 31, y, BRICK_M)
    for y, off in ((0, 0), (8, 16), (16, 0), (24, 16)):
        for x in range(off, 32, 32):
            pass
    # vertikale Fugen versetzt
    for y0, xs in ((1, (0, 16)), (9, (8, 24)), (17, (0, 16)), (25, (8, 24))):
        for x in xs:
            c.vline(x, y0, y0 + 6, BRICK_M)
    c.rect(0, 0, 1, 31, (230, 140, 90, 255)); c.rect(30, 0, 31, 31, BRICK_D)
    return c.im


def tile_question(frame=0):
    # Kristall-Item-Box (kein gelber "?"-Block): Cyan-Kristall + Diamant-Symbol.
    c = Canvas()
    shades = [CRYST, CRYST_H, CRYST]
    base = shades[frame % len(shades)]
    c.bevel(0, 0, 31, 31, base, CRYST_H, CRYST_D)
    c.rect(2, 2, 29, 29, base)
    c.bevel(1, 1, 30, 30, base, CRYST_H, CRYST_D)
    for rx, ry in ((3, 3), (28, 3), (3, 28), (28, 28)):
        c.disc(rx, ry, 1, CRYST_D)           # Nieten
    # Diamant-Symbol (Raute) statt "?"
    for i in range(8):
        c.hline(16 - i, 16 + i, 8 + i, WHITE)
        c.hline(16 - i, 16 + i, 24 - i, WHITE)
    for i in range(6):
        c.hline(16 - i, 16 + i, 10 + i, CRYST_D)
        c.hline(16 - i, 16 + i, 22 - i, CRYST_D)
    c.vline(15, 13, 19, CRYST_H)             # Glanz
    return c.im


def tile_question_used():
    c = Canvas()
    c.bevel(0, 0, 31, 31, BROWN, BROWN_L, BROWN_D)
    c.rect(2, 2, 29, 29, BROWN_D)
    c.bevel(2, 2, 29, 29, BROWN, BROWN_L, BROWN_D)
    for rx, ry in ((4, 4), (27, 4), (4, 27), (27, 27)):
        c.disc(rx, ry, 1, BROWN_D)
    return c.im


def tile_solid():
    c = Canvas()
    c.bevel(0, 0, 31, 31, GRAY, GRAY_H, GRAY_D)
    c.rect(3, 3, 28, 28, GRAY)
    c.bevel(3, 3, 28, 28, GRAY, GRAY_H, GRAY_D)
    c.hline(3, 28, 15, GRAY_D)
    return c.im


def _pipe(c, x0, x1, lip):
    body = STEEL
    c.rect(x0, 0, x1, 31, body)
    c.rect(x0, 0, x0 + 1, 31, STEEL_H)
    c.rect(x1 - 1, 0, x1, 31, STEEL_D)
    c.rect(x0 + 4, 0, x0 + 5, 31, STEEL_H)
    if lip:
        c.rect(x0, 0, x1, 7, body)
        c.rect(x0, 0, x1, 1, STEEL_H)
        c.rect(x0, 6, x1, 7, STEEL_D)
        c.rect(x0, 7, x1, 8, OL)


def pipe_top_l():
    c = Canvas(); _pipe(c, 2, 31, True); c.rect(0, 0, 1, 7, STEEL_H); return c.im


def pipe_top_r():
    c = Canvas(); _pipe(c, 0, 29, True); c.rect(30, 0, 31, 7, STEEL_D); return c.im


def pipe_body_l():
    c = Canvas(); _pipe(c, 2, 31, False); return c.im


def pipe_body_r():
    c = Canvas(); _pipe(c, 0, 29, False); return c.im


def coin(frame=0):
    c = Canvas()
    widths = [9, 6, 2, 6]
    w = widths[frame % 4]
    c.disc(16, 16, 12, T)                    # nichts (klar)
    for y in range(4, 28):
        # Ellipse: Halbbreite skaliert mit w/9
        hw = int(w * (1 - ((y - 16) / 13.0) ** 2) ** 0.5) if abs(y - 16) <= 13 else 0
        if hw > 0:
            c.hline(16 - hw, 16 + hw, y, YEL)
    if w >= 6:
        c.rect(14, 10, 17, 22, YEL_D)        # Praegung
        c.vline(15, 11, 21, (255, 240, 160, 255))
    c.outline(OL)
    return c.im


def mushroom():
    c = Canvas()
    c.disc(16, 13, 12, RED)                  # Kappe
    c.rect(4, 13, 28, 17, RED); c.rect(4, 16, 28, 18, RED_D)
    for (x, y, r) in ((9, 9, 3), (22, 10, 3), (16, 16, 2)):
        c.disc(x, y, r, WHITE)
    c.rect(9, 18, 23, 29, SKIN)              # Stiel
    c.rect(9, 18, 11, 29, SKIN_D)
    c.rect(13, 22, 15, 26, OL); c.rect(18, 22, 20, 26, OL)   # Augen
    c.outline()
    return c.im


def fireflower():
    c = Canvas()
    c.rect(15, 16, 17, 31, GREEN); c.rect(15, 16, 15, 31, GREEN_D)
    c.disc(9, 22, 3, GREEN_H); c.disc(23, 24, 3, GREEN_H)
    for a in range(0, 360, 60):
        px = 16 + 7 * math.cos(math.radians(a))
        py = 11 + 7 * math.sin(math.radians(a))
        c.disc(px, py, 3, ORANGE)
    c.disc(16, 11, 5, YEL); c.disc(16, 11, 3, WHITE)
    c.set(15, 10, OL); c.set(18, 12, OL)
    c.outline()
    return c.im


def star():
    c = Canvas()
    pts = []
    for i in range(5):
        a = -90 + i * 72
        pts.append((16 + 13 * math.cos(math.radians(a)),
                    16 + 13 * math.sin(math.radians(a))))
        a2 = -90 + 36 + i * 72
        pts.append((16 + 6 * math.cos(math.radians(a2)),
                    16 + 6 * math.sin(math.radians(a2))))
    from PIL import ImageDraw
    d = ImageDraw.Draw(c.im)
    d.polygon(pts, fill=YEL)
    c.rect(12, 14, 14, 18, OL); c.rect(18, 14, 20, 18, OL)   # Augen
    c.outline()
    return c.im


def oneup():
    # "Leben"-Item: Herz (statt gruener 1-Up-Pilz -> klar eigenstaendig).
    c = Canvas()
    PINK = (236, 96, 140, 255); PINK_D = (176, 52, 96, 255)
    PINK_H = (252, 168, 196, 255)
    c.disc(11, 12, 6, PINK); c.disc(21, 12, 6, PINK)
    for y in range(10, 28):
        hw = int((27 - y) * 0.95)
        if hw > 0:
            c.hline(16 - hw, 16 + hw, y, PINK)
    c.disc(10, 11, 2, PINK_H); c.disc(20, 11, 2, PINK_H)   # Glanz
    c.vline(9, 13, 16, PINK_D); c.vline(23, 13, 16, PINK_D)
    c.outline()
    return c.im


def water_top(frame=0):
    c = Canvas()
    c.rect(0, 6, 31, 31, WATER)
    c.rect(0, 6, 31, 31, WATER)
    off = 0 if frame == 0 else 3
    for x in range(0, 32, 8):
        c.disc((x + off) % 32, 6, 3, WATER_H)
    c.rect(0, 9, 31, 31, WATER)
    for x in range(2, 32, 9):
        c.hline(x, x + 3, 14 + (frame * 2), WATER_H)
    c.rect(0, 28, 31, 31, WATER_D)
    return c.im


def water_body():
    c = Canvas()
    c.rect(0, 0, 31, 31, WATER)
    for (x, y) in ((6, 8), (20, 14), (12, 22), (26, 26)):
        c.hline(x, x + 3, y, WATER_H)
    c.rect(0, 28, 31, 31, WATER_D)
    return c.im


def cloud():
    c = Canvas()
    c.disc(11, 18, 6, WHITE); c.disc(20, 16, 7, WHITE); c.disc(26, 19, 5, WHITE)
    c.rect(8, 18, 27, 24, WHITE)
    c.rect(8, 22, 27, 24, (210, 224, 248, 255))
    c.outline((120, 150, 210, 255))
    return c.im


def bush():
    c = Canvas()
    c.disc(9, 22, 6, GREEN); c.disc(18, 19, 7, GREEN); c.disc(25, 23, 5, GREEN)
    c.rect(4, 23, 28, 30, GREEN)
    c.rect(4, 27, 28, 30, GREEN_D)
    for x in range(4, 28, 5):
        c.disc(x, 20, 1, GREEN_H)
    c.outline()
    return c.im


def hill():
    c = Canvas()
    for y in range(8, 32):
        hw = int((y - 6) * 0.9)
        c.hline(16 - hw, 16 + hw, y, GREEN)
    c.rect(0, 30, 31, 31, GREEN_D)
    c.disc(12, 26, 1, GREEN_D); c.disc(20, 28, 1, GREEN_D)
    c.outline()
    return c.im


def flag():
    c = Canvas()
    c.rect(15, 1, 16, 31, GRAY); c.vline(15, 1, 31, GRAY_H)
    c.disc(15, 2, 2, YEL)
    pts = [(16, 4), (4, 9), (16, 14)]
    from PIL import ImageDraw
    d = ImageDraw.Draw(c.im)
    d.polygon(pts, fill=GREEN)
    c.disc(9, 9, 1, WHITE)
    c.outline()
    return c.im


# ----------------------------------------------------------------- Animierte Figuren
def anim_specs():
    """name -> (frames[(framename, img)], anims[Anim])."""
    return {
        "player": ([
            ("idle", draw_hero("idle")), ("run0", draw_hero("stride_l")),
            ("run1", draw_hero("tuck")), ("run2", draw_hero("stride_r")),
            ("jump", draw_hero("tuck", arm_up=True)),
            ("skid", draw_hero("stride_r", lean=3)),
            ("duck", draw_hero(duck=True)), ("dead", draw_hero("idle", dead=True)),
        ], [Anim("idle", 0, 0, 1), Anim("run", 1, 3, 12), Anim("jump", 4, 4, 1),
            Anim("skid", 5, 5, 1), Anim("duck", 6, 6, 1), Anim("dead", 7, 7, 1)]),
        "walker": ([("walk0", draw_goomba(0)), ("walk1", draw_goomba(1)),
                    ("squash", draw_goomba(squash=True))],
                   [Anim("walk", 0, 1, 6), Anim("squash", 2, 2, 1)]),
        "guard": ([("walk0", draw_koopa(0)), ("walk1", draw_koopa(1)),
                   ("shell", draw_koopa(shell_only=True))],
                  [Anim("walk", 0, 1, 5), Anim("shell", 2, 2, 1)]),
        "snapper": ([("open", draw_piranha(True)), ("closed", draw_piranha(False))],
                    [Anim("bite", 0, 1, 3)]),
        "glider": ([("fly0", draw_para(0)), ("fly1", draw_para(1))],
                 [Anim("fly", 0, 1, 7)]),
        "coin": ([("c0", coin(0)), ("c1", coin(1)), ("c2", coin(2)), ("c3", coin(3))],
                 [Anim("spin", 0, 3, 10)]),
        "itembox": ([("q0", tile_question(0)), ("q1", tile_question(1)),
                    ("q2", tile_question(0))],
                   [Anim("idle", 0, 2, 4)]),
        "water": ([("w0", water_top(0)), ("w1", water_top(1))],
                  [Anim("wave", 0, 1, 3)]),
    }


def static_tiles():
    """name -> img (Einzel-Tiles/Items fuers Master-Sheet)."""
    return {
        "ground": tile_ground(),
        "brick": tile_brick(),
        "itembox_used": tile_question_used(),
        "solid": tile_solid(),
        "tube_tl": pipe_top_l(), "tube_tr": pipe_top_r(),
        "tube_bl": pipe_body_l(), "tube_br": pipe_body_r(),
        "water_body": water_body(),
        "grow": mushroom(), "bolt": fireflower(),
        "shard": star(), "life": oneup(),
        "cloud": cloud(), "bush": bush(), "hill": hill(), "flag": flag(),
    }


# ----------------------------------------------------------------- Export
def export():
    OUT.mkdir(parents=True, exist_ok=True)
    specs = anim_specs()
    # 1) Pro animierte Figur: .dhsprite + .gif
    for name, (frames, anims) in specs.items():
        doc = SpriteDoc(S, S)
        doc.frames = [Frame(pixels=img, name=fn, duration_ms=120)
                      for fn, img in frames]
        doc.anims = anims
        doc.save_native(OUT / f"{name}.dhsprite")
        doc.save_sheet_png(OUT / f"{name}.png")   # horizontaler Strip fuer SPRITE_NEW
        doc.save_animated_gif(OUT / f"{name}.gif", scale=4)

    # 2) Master-Atlas: ALLE benannten Sprites in ein Raster
    named: list[tuple[str, Image.Image]] = []
    for name, (frames, _a) in specs.items():
        for fn, img in frames:
            named.append((f"{name}_{fn}", img))
    for name, img in static_tiles().items():
        named.append((name, img))
    cols = 10
    rows = (len(named) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * S, rows * S), T)
    sprites = {}
    for i, (nm, img) in enumerate(named):
        x, y = (i % cols) * S, (i // cols) * S
        sheet.alpha_composite(img, (x, y))
        sprites[nm] = [x, y, S, S]
    sheet.save(OUT / "sheet.png")
    (OUT / "sheet.json").write_text(json.dumps(
        {"image": "sheet.png", "sprites": sprites}, indent=2), encoding="utf-8")

    # 2b) Standalone-Spieler-Atlas fuer examples/77_tiled_platformer.dh
    #     (5 Frames: idle/walk_a/walk_b/jump/hit), generisch benannt. Dort ist die
    #     Welt 16er-gerastert -> Frames auf 16x16 herunterskalieren (Nearest).
    PS = 16
    pframes = dict(specs["player"][0])
    pmap = [("player_idle", "idle"), ("player_walk_a", "run0"),
            ("player_walk_b", "run2"), ("player_jump", "jump"),
            ("player_hit", "skid")]
    psheet = Image.new("RGBA", (len(pmap) * PS, PS), T)
    pspr = {}
    for i, (out_nm, src_nm) in enumerate(pmap):
        small = pframes[src_nm].resize((PS, PS), Image.NEAREST)
        psheet.alpha_composite(small, (i * PS, 0))
        pspr[out_nm] = [i * PS, 0, PS, PS]
    assets = OUT.parent / "assets"
    assets.mkdir(parents=True, exist_ok=True)
    psheet.save(assets / "player.png")
    (assets / "player_atlas.json").write_text(json.dumps(
        {"image": "player.png", "sprites": pspr}, indent=2), encoding="utf-8")

    # 3) Kontaktbogen (zum Begutachten)
    _contact(named, OUT / "_contact.png")
    print(f"  {len(specs)} animierte Figuren (.dhsprite + .gif)")
    print(f"  Master-Sheet: sheet.png ({cols}x{rows} = {len(named)} Sprites) + sheet.json")
    print("  Kontaktbogen: _contact.png")


def _contact(named, path, scale=4):
    cols = 10
    rows = (len(named) + cols - 1) // cols
    cell = S * scale
    pad = 6
    lab = 12
    W = cols * (cell + pad) + pad
    H = rows * (cell + lab + pad) + pad
    sheet = Image.new("RGBA", (W, H), (30, 34, 46, 255))
    from PIL import ImageDraw
    d = ImageDraw.Draw(sheet)
    for i, (nm, img) in enumerate(named):
        cxp = pad + (i % cols) * (cell + pad)
        cyp = pad + (i // cols) * (cell + lab + pad)
        sheet.alpha_composite(img.resize((cell, cell), Image.NEAREST), (cxp, cyp))
        d.text((cxp, cyp + cell + 1), nm, fill=(210, 220, 235, 255))
    sheet.save(path)


if __name__ == "__main__":
    export()
    print("fertig.")
