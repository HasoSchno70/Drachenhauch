"""Tileset-Generator fuer CIRCUIT RUNNER -- 23x23, prozedural.

Erzeugt EIN Master-Sheet `assets/tiles.png` (16 Spalten x 8 Zeilen, je 23px),
in dem die **Zellen-Position dem Chip's-Challenge-Tile-Code entspricht**
(Code 0x00..0x7F). Die Engine zeichnet damit jede Kachel/Figur per
`DRAWIMAGEPART(sheet, (code%16)*23, (code//16)*23, 23, 23, ...)` -- eine
einzige Grafik-Vokabel-Quelle.

Eigenstaendiges "Neon-Circuit"-Thema (NICHT die Original-Grafik von Chip's
Challenge nachgezeichnet, nur Spielprinzip + Tile-Codes): dunkle Platinen-
Boeden, Cyan/Magenta-Akzente, eigene Roboter-Figuren.

Aufruf:  py circuitrunner/make_tiles.py
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from PIL import Image

_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

OUT = Path(__file__).resolve().parent
ASSETS = OUT / "assets"
S = 23  # Kantenlaenge

# ----------------------------------------------------------------- Palette
T = (0, 0, 0, 0)
OL = (10, 14, 20, 255)                 # Outline / dunkel
FLOOR = (30, 38, 52, 255)              # Platinen-Boden
FLOOR_D = (22, 28, 40, 255)
FLOOR_L = (44, 56, 74, 255)
GRID = (40, 52, 70, 255)
WALL = (96, 110, 134, 255)             # Wand
WALL_L = (140, 156, 184, 255)
WALL_D = (54, 64, 84, 255)
STEEL = (120, 130, 150, 255)
WATER = (40, 110, 220, 255)
WATER_L = (110, 180, 255, 255)
WATER_D = (24, 70, 160, 255)
FIRE = (250, 120, 30, 255)
FIRE_L = (255, 210, 80, 255)
FIRE_D = (200, 50, 16, 255)
ICE = (175, 225, 245, 255)
ICE_L = (225, 248, 255, 255)
ICE_D = (120, 180, 215, 255)
DIRT = (120, 92, 60, 255)
DIRT_D = (88, 64, 40, 255)
GRAVEL = (96, 98, 104, 255)
GRAVEL_L = (132, 134, 140, 255)
CHIPG = (70, 230, 140, 255)            # IC-Chip gruen
CHIPG_D = (30, 150, 90, 255)
CHIPG_L = (170, 255, 210, 255)
GOLD = (250, 205, 70, 255)
GOLD_D = (190, 150, 36, 255)
NEON = (90, 240, 230, 255)             # Cyan-Neon (Exit/Akzent)
NEON_D = (30, 150, 150, 255)
MAGENTA = (235, 70, 200, 255)
WHITE = (240, 244, 250, 255)
BLACK = (16, 18, 24, 255)
# Schluessel/Tuer-Farben
KEYBLUE = (70, 130, 250, 255)
KEYRED = (235, 70, 70, 255)
KEYGRN = (70, 210, 90, 255)
KEYYEL = (245, 215, 60, 255)
KEYBLUE_D = (40, 80, 180, 255)
KEYRED_D = (170, 40, 40, 255)
KEYGRN_D = (40, 150, 56, 255)
KEYYEL_D = (190, 160, 30, 255)


class C:
    """Winzige Pixel-Zeichenflaeche (harte Kanten)."""

    def __init__(self, w=S, h=S, bg=T):
        self.im = Image.new("RGBA", (w, h), bg)
        self.p = self.im.load()
        self.w, self.h = w, h

    def set(self, x, y, c):
        x, y = int(x), int(y)
        if 0 <= x < self.w and 0 <= y < self.h and c[3]:
            if c[3] == 255:
                self.p[x, y] = c
            else:  # alpha-blend
                bx = self.p[x, y]
                a = c[3] / 255.0
                self.p[x, y] = (
                    int(c[0] * a + bx[0] * (1 - a)),
                    int(c[1] * a + bx[1] * (1 - a)),
                    int(c[2] * a + bx[2] * (1 - a)),
                    max(bx[3], c[3]),
                )

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
                if (x - cx) ** 2 + (y - cy) ** 2 <= r * r + r * 0.5:
                    self.set(x, y, c)

    def ring(self, cx, cy, r, c):
        for y in range(int(cy - r), int(cy + r) + 1):
            for x in range(int(cx - r), int(cx + r) + 1):
                d = (x - cx) ** 2 + (y - cy) ** 2
                if (r - 1) ** 2 <= d <= r * r + r * 0.5:
                    self.set(x, y, c)

    def frame(self, c, t=1):
        for i in range(t):
            self.hline(i, self.w - 1 - i, i, c)
            self.hline(i, self.w - 1 - i, self.h - 1 - i, c)
            self.vline(i, i, self.h - 1 - i, c)
            self.vline(self.w - 1 - i, i, self.h - 1 - i, c)

    def outline(self, c=OL):
        src = self.im.copy()
        sp = src.load()
        for y in range(self.h):
            for x in range(self.w):
                if sp[x, y][3] == 0:
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < self.w and 0 <= ny < self.h and sp[nx, ny][3] > 200:
                            self.p[x, y] = c
                            break


def arrow(c, dx, dy, col, cx=11, cy=11, ln=6):
    """Pfeil in Richtung (dx,dy) ins Bild zeichnen."""
    for i in range(-ln, ln + 1):
        c.set(cx + dx * i * 0.0 + (0 if dx else i), cy + (0 if dy else i), col) if False else None
    # einfacher: Schaft + Spitze
    if dx != 0:
        sgn = 1 if dx > 0 else -1
        for i in range(-5, 6):
            c.set(cx + i, cy, col)
        for k in range(4):
            c.vline(cx + sgn * (5 - k), cy - k, cy + k, col)
    else:
        sgn = 1 if dy > 0 else -1
        for i in range(-5, 6):
            c.set(cx, cy + i, col)
        for k in range(4):
            c.hline(cx - k, cx + k, cy + sgn * (5 - k), col)


# ========================================================= BODEN / TERRAIN
def tile_floor():
    c = C(bg=FLOOR)
    c.frame(FLOOR_D, 1)
    c.hline(1, S - 2, 1, FLOOR_L)
    # zarte Platinen-Leiterbahn
    c.hline(4, 18, 7, GRID)
    c.vline(18, 7, 16, GRID)
    c.disc(4, 7, 1, NEON_D)
    c.disc(18, 16, 1, NEON_D)
    return c.im


def tile_wall():
    c = C(bg=WALL)
    c.frame(WALL_L, 1)
    c.hline(0, S - 1, S - 1, WALL_D)
    c.vline(S - 1, 0, S - 1, WALL_D)
    # Nieten
    for rx, ry in ((4, 4), (18, 4), (4, 18), (18, 18)):
        c.disc(rx, ry, 1, WALL_D)
        c.set(rx - 1, ry - 1, WALL_L)
    c.rect(8, 8, 14, 14, STEEL)
    c.frame(WALL_D, 0)
    return c.im


def tile_invis_wall():  # 05/2C -- als Boden zeichnen (unsichtbar)
    return tile_floor()


def tile_chip():
    c = C(bg=FLOOR)
    c.frame(FLOOR_D, 1)
    # IC-Gehaeuse
    c.rect(5, 5, 17, 17, CHIPG_D)
    c.rect(6, 6, 16, 16, CHIPG)
    c.rect(8, 8, 14, 14, CHIPG_L)
    c.rect(9, 9, 13, 13, CHIPG_D)
    # Beinchen
    for i in range(6, 17, 4):
        c.set(3, i, GOLD); c.set(4, i, GOLD)
        c.set(18, i, GOLD); c.set(19, i, GOLD)
        c.set(i, 3, GOLD); c.set(i, 4, GOLD)
        c.set(i, 18, GOLD); c.set(i, 19, GOLD)
    return c.im


def tile_water(frame=0):
    c = C(bg=WATER)
    c.rect(0, 0, S - 1, 2, WATER_D)
    for x in range(0, S, 6):
        off = (x + frame * 3) % S
        c.disc(off, 4, 2, WATER_L)
    for (x, y) in ((4, 10), (14, 13), (8, 17), (18, 9)):
        c.hline(x, x + 3, y, WATER_L)
    c.rect(0, S - 2, S - 1, S - 1, WATER_D)
    return c.im


def tile_fire(frame=0):
    c = C(bg=FLOOR)
    c.frame(FLOOR_D, 1)
    # Flammen
    base = 20
    for fx, h, col in ((6, 11, FIRE_D), (12, 8, FIRE), (16, 12, FIRE_D)):
        for y in range(base, base - h, -1):
            spread = max(1, (y - (base - h)) // 2)
            c.hline(fx - spread, fx + spread, y, col)
    for fx, h in ((7, 8), (12, 6), (15, 9)):
        for y in range(base, base - h, -1):
            spread = max(0, (y - (base - h)) // 2)
            c.hline(fx - spread, fx + spread, y, FIRE)
    c.disc(11, 16, 3, FIRE_L)
    c.disc(8, 15, 1, FIRE_L)
    c.disc(15, 16, 1, FIRE_L)
    return c.im


def tile_ice():
    c = C(bg=ICE)
    c.frame(ICE_L, 1)
    c.hline(0, S - 1, S - 1, ICE_D)
    # Risse / Glanz
    c.set(5, 5, ICE_L); c.hline(5, 9, 5, ICE_L); c.vline(9, 5, 9, ICE_L)
    c.hline(13, 18, 13, ICE_D); c.vline(13, 13, 17, ICE_D)
    c.disc(16, 6, 1, WHITE)
    return c.im


def tile_ice_corner(kind):  # 1A SE 1B SW 1C NW 1D NE
    c = C()
    c.im.alpha_composite(tile_ice())
    # zwei Waende an der Aussenecke andeuten (wo es NICHT weiter rutscht)
    a, b = {"SE": ((0, 1), (1, 0)), "SW": ((0, 1), (-1, 0)),
            "NW": ((0, -1), (-1, 0)), "NE": ((0, -1), (1, 0))}[kind]
    for (dx, dy) in (a, b):
        if dx > 0:
            c.rect(S - 3, 0, S - 1, S - 1, WALL)
        elif dx < 0:
            c.rect(0, 0, 2, S - 1, WALL)
        elif dy > 0:
            c.rect(0, S - 3, S - 1, S - 1, WALL)
        else:
            c.rect(0, 0, S - 1, 2, WALL)
    return c.im


def tile_dirt():
    c = C(bg=DIRT)
    c.frame(DIRT_D, 1)
    for (x, y) in ((5, 6), (15, 8), (9, 14), (17, 16), (4, 17), (12, 4)):
        c.disc(x, y, 1, DIRT_D)
    c.disc(8, 9, 1, DIRT)
    return c.im


def tile_gravel():
    c = C(bg=GRAVEL)
    c.frame((70, 72, 78, 255), 1)
    for (x, y) in ((4, 5), (13, 7), (8, 12), (17, 14), (6, 17), (15, 18), (11, 3)):
        c.disc(x, y, 1, GRAVEL_L)
        c.set(x + 1, y + 1, (60, 62, 68, 255))
    return c.im


def tile_force(dx, dy):
    c = C(bg=(70, 60, 120, 255))
    c.frame((110, 95, 180, 255), 1)
    for k in range(3):
        ox = -dx * 0 + 0
        cy = 6 + k * 5 if dy else 11
        cx = 11 if dy else 6 + k * 5
        col = (180, 160, 255, 255) if k == 1 else (130, 110, 210, 255)
        arrow(c, dx, dy, col, cx=cx, cy=cy, ln=2)
    return c.im


def tile_force_random():
    c = C(bg=(70, 60, 120, 255))
    c.frame((110, 95, 180, 255), 1)
    arrow(c, 1, 0, (180, 160, 255, 255), cx=11, cy=6)
    arrow(c, -1, 0, (180, 160, 255, 255), cx=11, cy=16)
    c.disc(11, 11, 2, WHITE)
    return c.im


def tile_exit():
    c = C(bg=BLACK)
    for r in range(11, 0, -1):
        col = NEON if (r // 2) % 2 == 0 else (20, 30, 40, 255)
        c.ring(11, 11, r, col)
    c.disc(11, 11, 2, WHITE)
    c.frame(NEON_D, 1)
    return c.im


def tile_socket():
    c = C(bg=FLOOR)
    c.frame(FLOOR_D, 1)
    c.rect(4, 4, 18, 18, (40, 44, 60, 255))
    c.frame((30, 34, 46, 255), 2)
    # Chip-Symbol durchgestrichen / Stecker
    c.rect(8, 7, 14, 15, STEEL)
    c.rect(9, 8, 13, 14, (60, 64, 80, 255))
    for i in range(8, 16, 2):
        c.set(6, i, GOLD); c.set(16, i, GOLD)
    c.disc(11, 11, 2, CHIPG)
    return c.im


def tile_door(col, cold):
    c = C(bg=cold)
    c.frame(col, 2)
    c.rect(3, 3, S - 4, S - 4, col)
    c.rect(5, 5, S - 6, S - 6, cold)
    # Schloss
    c.disc(11, 10, 3, GOLD)
    c.rect(10, 10, 12, 15, GOLD)
    c.disc(11, 10, 1, OL)
    c.hline(1, S - 2, 11, cold)  # Spalt
    return c.im


def tile_blue_wall():  # 1E/1F
    c = C(bg=KEYBLUE_D)
    c.frame(KEYBLUE, 2)
    c.rect(4, 4, 18, 18, KEYBLUE_D)
    c.disc(11, 11, 4, KEYBLUE)
    c.set(9, 9, (150, 190, 255, 255))
    return c.im


def tile_block():
    c = C(bg=DIRT)
    c.frame((150, 120, 80, 255), 1)
    c.hline(0, S - 1, S - 1, DIRT_D)
    c.vline(S - 1, 0, S - 1, DIRT_D)
    # Holz/Stahl-Kiste mit Diagonalen
    c.rect(3, 3, S - 4, S - 4, (140, 108, 70, 255))
    c.frame(DIRT_D, 0)
    for i in range(4, S - 4):
        c.set(i, i, DIRT_D)
        c.set(S - 1 - i, i, DIRT_D)
    return c.im


def tile_button(col):
    c = C(bg=FLOOR)
    c.frame(FLOOR_D, 1)
    c.disc(11, 11, 7, (30, 34, 46, 255))
    c.disc(11, 11, 5, col)
    c.disc(9, 9, 2, WHITE)
    return c.im


def tile_toggle(open_):
    if open_:
        c = C(bg=FLOOR)
        c.frame(FLOOR_D, 1)
        c.frame((70, 210, 90, 120), 2)
        c.disc(11, 11, 2, (70, 210, 90, 160))
        return c.im
    c = C(bg=(40, 80, 60, 255))
    c.frame((70, 210, 90, 255), 2)
    c.rect(3, 3, S - 4, S - 4, (50, 120, 80, 255))
    for y in range(4, S - 4, 4):
        c.hline(4, S - 5, y, (40, 90, 60, 255))
    return c.im


def tile_teleport():
    c = C(bg=BLACK)
    for r in range(10, 0, -1):
        col = MAGENTA if (r // 2) % 2 == 0 else (40, 10, 40, 255)
        c.ring(11, 11, r, col)
    c.disc(11, 11, 2, WHITE)
    c.frame((120, 40, 110, 255), 1)
    return c.im


def tile_bomb():
    c = C(bg=FLOOR)
    c.frame(FLOOR_D, 1)
    c.disc(11, 13, 7, BLACK)
    c.disc(9, 11, 2, (90, 96, 110, 255))
    # Zuendschnur
    c.vline(13, 3, 6, (150, 120, 70, 255))
    c.disc(13, 3, 1, FIRE_L)
    c.disc(13, 3, 2, FIRE)
    return c.im


def tile_trap():
    c = C(bg=FLOOR)
    c.frame(FLOOR_D, 1)
    c.rect(3, 3, 19, 19, (24, 26, 34, 255))
    c.frame((60, 64, 80, 255), 2)
    # Zaehne der Falle
    for x in range(4, 19, 3):
        c.vline(x, 4, 7, STEEL)
        c.vline(x, 15, 18, STEEL)
    c.rect(6, 9, 16, 13, BLACK)
    return c.im


def tile_hint():
    c = C(bg=FLOOR)
    c.frame(FLOOR_D, 1)
    c.rect(4, 4, 18, 18, (40, 44, 60, 255))
    c.frame((60, 64, 84, 255), 1)
    # "?"
    c.rect(8, 6, 13, 8, NEON)
    c.rect(12, 8, 14, 11, NEON)
    c.rect(10, 11, 13, 13, NEON)
    c.rect(10, 15, 12, 17, NEON)
    return c.im


def tile_thief():
    c = C(bg=FLOOR)
    c.frame(FLOOR_D, 1)
    # Maske / Dieb
    c.disc(11, 10, 6, (40, 44, 60, 255))
    c.rect(5, 8, 17, 11, BLACK)            # Augenmaske
    c.disc(8, 9, 1, NEON)
    c.disc(14, 9, 1, NEON)
    c.rect(7, 14, 15, 19, (60, 64, 84, 255))  # Umhang
    c.set(11, 16, MAGENTA)
    return c.im


def tile_cloner():
    c = C(bg=(40, 44, 60, 255))
    c.frame(STEEL, 2)
    c.rect(3, 3, S - 4, S - 4, (54, 58, 78, 255))
    # Maschinen-Schlitz
    c.rect(6, 6, 16, 16, (24, 26, 34, 255))
    c.frame((30, 34, 46, 255), 0)
    c.rect(8, 8, 14, 14, MAGENTA)
    c.rect(9, 9, 13, 13, (120, 30, 100, 255))
    for x in (4, 18):
        c.disc(x, 4, 1, NEON)
        c.disc(x, 18, 1, NEON)
    return c.im


def tile_thin_wall(dirs):
    c = C()
    c.im.alpha_composite(tile_floor())
    for d in dirs:
        if d == "N":
            c.rect(0, 0, S - 1, 2, WALL); c.hline(0, S - 1, 0, WALL_L)
        elif d == "S":
            c.rect(0, S - 3, S - 1, S - 1, WALL); c.hline(0, S - 1, S - 1, WALL_D)
        elif d == "W":
            c.rect(0, 0, 2, S - 1, WALL); c.vline(0, 0, S - 1, WALL_L)
        elif d == "E":
            c.rect(S - 3, 0, S - 1, S - 1, WALL); c.vline(S - 1, 0, S - 1, WALL_D)
    return c.im


# ========================================================= ITEMS
def item_key(col, cold):
    c = C()
    c.disc(8, 8, 4, col)
    c.disc(8, 8, 2, T)
    c.ring(8, 8, 3, col)
    c.rect(9, 10, 11, 19, col)         # Schaft
    c.rect(11, 16, 14, 17, col)        # Baerte
    c.rect(11, 18, 13, 19, col)
    c.set(7, 6, WHITE)
    c.outline()
    return c.im


def item_boot(col, cold, mark):
    c = C()
    c.rect(7, 4, 12, 15, col)          # Schaft
    c.rect(7, 15, 17, 19, col)         # Fuss
    c.rect(7, 15, 17, 16, cold)
    c.rect(7, 18, 17, 19, cold)
    c.set(8, 5, WHITE)
    # Markierung (Symbol)
    if mark == "water":
        c.disc(10, 10, 2, WATER_L)
    elif mark == "fire":
        c.disc(10, 10, 2, FIRE_L)
    elif mark == "ice":
        c.disc(10, 10, 2, ICE_L)
    elif mark == "force":
        c.disc(10, 10, 2, (200, 180, 255, 255))
    c.outline()
    return c.im


# ========================================================= FIGUREN (Basis = Blick N)
def _eyes(c, cy):
    c.disc(8, cy, 1, WHITE); c.disc(14, cy, 1, WHITE)
    c.set(8, cy, OL); c.set(14, cy, OL)


def base_bug():  # Kaefer (rot)
    c = C()
    c.disc(11, 12, 8, (210, 60, 60, 255))
    c.disc(11, 12, 8, (170, 40, 40, 255))
    c.disc(11, 11, 6, (230, 80, 80, 255))
    c.vline(11, 4, 19, (120, 24, 24, 255))     # Naht
    c.set(7, 9, OL); c.set(15, 9, OL)
    # Fuehler nach vorne (N)
    c.set(8, 4, OL); c.set(7, 3, OL)
    c.set(14, 4, OL); c.set(15, 3, OL)
    _eyes(c, 8)
    c.outline()
    return c.im


def base_fireball():
    c = C()
    c.disc(11, 12, 8, FIRE_D)
    c.disc(11, 12, 6, FIRE)
    c.disc(11, 11, 4, FIRE_L)
    c.disc(9, 9, 1, WHITE)
    # Flammenzunge nach vorne
    c.disc(11, 4, 2, FIRE)
    c.disc(11, 3, 1, FIRE_L)
    c.outline()
    return c.im


def base_ball():  # Pink Ball
    c = C()
    c.disc(11, 12, 8, (170, 40, 130, 255))
    c.disc(11, 12, 7, MAGENTA)
    c.disc(8, 9, 2, (255, 180, 240, 255))
    c.disc(11, 11, 1, WHITE)
    c.outline()
    return c.im


def base_tank():
    c = C()
    c.rect(4, 7, 18, 18, (70, 90, 150, 255))
    c.frame((110, 140, 210, 255), 1)
    c.rect(7, 10, 15, 16, (40, 60, 110, 255))
    # Lauf nach vorne
    c.rect(10, 1, 12, 8, STEEL)
    c.disc(11, 13, 2, NEON)
    for x in (5, 17):
        c.vline(x, 8, 17, (40, 56, 100, 255))
    c.outline()
    return c.im


def base_glider():
    c = C()
    # Dreieck/Papierflieger nach vorne (N)
    for y in range(3, 20):
        half = int((y - 3) * 0.6)
        c.hline(11 - half, 11 + half, y, (60, 190, 190, 255))
    c.vline(11, 3, 19, NEON)
    for y in range(8, 20):
        half = int((y - 3) * 0.6)
        c.set(11 - half, y, NEON_D); c.set(11 + half, y, NEON_D)
    c.disc(11, 8, 1, WHITE)
    c.outline()
    return c.im


def base_teeth():  # Verfolger (lila Blob mit Zaehnen)
    c = C()
    c.disc(11, 12, 8, (150, 70, 200, 255))
    c.disc(11, 11, 7, (180, 100, 230, 255))
    _eyes(c, 9)
    # Zaehne unten (Maul)
    c.rect(6, 15, 16, 17, WHITE)
    for x in range(7, 16, 2):
        c.vline(x, 15, 17, (120, 50, 160, 255))
    c.outline()
    return c.im


def base_walker():  # Hantel/Walker (grau)
    c = C()
    c.disc(11, 6, 4, STEEL)
    c.disc(11, 17, 4, STEEL)
    c.rect(9, 6, 13, 17, (90, 96, 110, 255))
    c.disc(11, 6, 3, (150, 160, 180, 255))
    c.disc(11, 17, 3, (150, 160, 180, 255))
    c.disc(11, 6, 1, NEON)
    c.disc(11, 17, 1, NEON)
    c.outline()
    return c.im


def base_blob():  # Amoebe (gruen)
    c = C()
    c.disc(11, 12, 8, (50, 150, 70, 255))
    c.disc(11, 12, 7, (80, 200, 100, 255))
    c.disc(8, 9, 2, (160, 240, 170, 255))
    c.disc(14, 14, 1, (160, 240, 170, 255))
    _eyes(c, 11)
    c.outline()
    return c.im


def base_paramecium():  # Tausendfuessler (gelb)
    c = C()
    for i, y in enumerate(range(4, 20, 3)):
        col = KEYYEL if i % 2 == 0 else KEYYEL_D
        c.disc(11, y, 4, col)
        c.set(6, y, OL); c.set(16, y, OL)     # Beinchen
    c.disc(11, 4, 4, KEYYEL)
    _eyes(c, 3)
    c.outline()
    return c.im


def base_player():  # Chip-Roboter, Blick N
    c = C()
    c.rect(6, 6, 16, 19, (60, 120, 200, 255))   # Koerper
    c.frame((110, 170, 240, 255), 1)
    c.rect(7, 2, 15, 8, (200, 210, 225, 255))   # Helm
    c.rect(7, 5, 15, 7, BLACK)                   # Visier
    c.disc(9, 6, 1, NEON); c.disc(13, 6, 1, NEON)
    c.disc(11, 12, 2, NEON)                       # Brust-Kern
    c.rect(4, 9, 6, 16, (50, 100, 170, 255))     # Arme
    c.rect(16, 9, 18, 16, (50, 100, 170, 255))
    c.outline()
    return c.im


def rot4(base):
    """Basis (Blick N) -> [N, W, S, E] per Drehung (CCW positiv)."""
    return [base, base.rotate(90), base.rotate(180), base.rotate(270)]


# ========================================================= ZUSAMMENBAU
def build():
    ASSETS.mkdir(parents=True, exist_ok=True)
    cells = {}  # code -> Image

    cells[0x00] = tile_floor()
    cells[0x01] = tile_wall()
    cells[0x02] = tile_chip()
    cells[0x03] = tile_water()
    cells[0x04] = tile_fire()
    cells[0x05] = tile_invis_wall()
    cells[0x06] = tile_thin_wall(["N"])
    cells[0x07] = tile_thin_wall(["W"])
    cells[0x08] = tile_thin_wall(["S"])
    cells[0x09] = tile_thin_wall(["E"])
    cells[0x0A] = tile_block()
    cells[0x0B] = tile_dirt()
    cells[0x0C] = tile_ice()
    cells[0x0D] = tile_force(0, 1)     # S
    cells[0x0E] = tile_block()         # Clone-Block N (gleiche Optik)
    cells[0x0F] = tile_block()
    cells[0x10] = tile_block()
    cells[0x11] = tile_block()
    cells[0x12] = tile_force(0, -1)    # N
    cells[0x13] = tile_force(1, 0)     # E
    cells[0x14] = tile_force(-1, 0)    # W
    cells[0x15] = tile_exit()
    cells[0x16] = tile_door(KEYBLUE, KEYBLUE_D)
    cells[0x17] = tile_door(KEYRED, KEYRED_D)
    cells[0x18] = tile_door(KEYGRN, KEYGRN_D)
    cells[0x19] = tile_door(KEYYEL, KEYYEL_D)
    cells[0x1A] = tile_ice_corner("SE")
    cells[0x1B] = tile_ice_corner("SW")
    cells[0x1C] = tile_ice_corner("NW")
    cells[0x1D] = tile_ice_corner("NE")
    cells[0x1E] = tile_blue_wall()
    cells[0x1F] = tile_blue_wall()
    cells[0x21] = tile_thief()
    cells[0x22] = tile_socket()
    cells[0x23] = tile_button(KEYGRN)
    cells[0x24] = tile_button(KEYRED)
    cells[0x25] = tile_toggle(False)
    cells[0x26] = tile_toggle(True)
    cells[0x27] = tile_button(DIRT)
    cells[0x28] = tile_button(KEYBLUE)
    cells[0x29] = tile_teleport()
    cells[0x2A] = tile_bomb()
    cells[0x2B] = tile_trap()
    cells[0x2C] = tile_invis_wall()
    cells[0x2D] = tile_gravel()
    cells[0x2E] = tile_floor()         # Pass once (Popup-Wall) -> Boden-Optik
    cells[0x2F] = tile_hint()
    cells[0x30] = tile_thin_wall(["S", "E"])
    cells[0x31] = tile_cloner()
    cells[0x32] = tile_force_random()
    cells[0x39] = tile_exit()
    cells[0x3A] = tile_exit()
    cells[0x3B] = tile_exit()

    # Chip schwimmend 3C-3F = Spieler-Frames im Wasser (einfach Spieler)
    pl = rot4(base_player())
    for i in range(4):
        cells[0x3C + i] = pl[i]

    # Figuren 40-63
    monster_bases = [base_bug, base_fireball, base_ball, base_tank,
                     base_glider, base_teeth, base_walker, base_blob,
                     base_paramecium]
    code = 0x40
    for fn in monster_bases:
        dirs = rot4(fn())
        for i in range(4):
            cells[code + i] = dirs[i]
        code += 4

    # Schluessel 64-67
    cells[0x64] = item_key(KEYBLUE, KEYBLUE_D)
    cells[0x65] = item_key(KEYRED, KEYRED_D)
    cells[0x66] = item_key(KEYGRN, KEYGRN_D)
    cells[0x67] = item_key(KEYYEL, KEYYEL_D)
    # Stiefel 68-6B
    cells[0x68] = item_boot(WATER, WATER_D, "water")     # Flippers
    cells[0x69] = item_boot(FIRE, FIRE_D, "fire")        # Fire boots
    cells[0x6A] = item_boot(ICE, ICE_D, "ice")           # Ice skates
    cells[0x6B] = item_boot((150, 120, 220, 255), (90, 70, 160, 255), "force")
    # Spieler 6C-6F
    for i in range(4):
        cells[0x6C + i] = pl[i]

    # ---- Sheet bauen (16 x 8)
    cols, rows = 16, 8
    sheet = Image.new("RGBA", (cols * S, rows * S), T)
    for code, img in cells.items():
        x = (code % cols) * S
        y = (code // cols) * S
        sheet.alpha_composite(img.convert("RGBA"), (x, y))
    sheet.save(ASSETS / "tiles.png")

    # Manifest (Doku, welcher Code wo)
    names = _names()
    (ASSETS / "tiles.json").write_text(json.dumps(
        {"image": "tiles.png", "tile": S, "cols": cols,
         "codes": {f"0x{c:02X}": names.get(c, "?") for c in sorted(cells)}},
        indent=2), encoding="utf-8")

    # ---- .gbsprite (im Sprite-Editor `gbsprites` zu oeffnen/bearbeiten)
    try:
        from gamebasic.spriteeditor.document import SpriteDoc, Frame
        doc = SpriteDoc(S, S)
        doc.frames = [Frame(pixels=cells[c].convert("RGBA"),
                            name=f"{c:02X}_{names.get(c, '?')}")
                      for c in sorted(cells)]
        doc.save_native(ASSETS / "tiles.gbsprite")
        print("Editierbar:    assets/tiles.gbsprite (gbsprites)")
    except Exception as e:  # SpriteDoc optional
        print(f"(.gbsprite uebersprungen: {e})")

    _contact(cells, names, ASSETS / "_contact.png")
    print(f"tiles.png  ({cols}x{rows} Zellen a {S}px) -- {len(cells)} Kacheln")
    print("Kontaktbogen: assets/_contact.png")


def _names():
    n = {0x00: "floor", 0x01: "wall", 0x02: "chip", 0x03: "water", 0x04: "fire",
         0x05: "invwall", 0x06: "wallN", 0x07: "wallW", 0x08: "wallS", 0x09: "wallE",
         0x0A: "block", 0x0B: "dirt", 0x0C: "ice", 0x0D: "force_S",
         0x0E: "cblockN", 0x0F: "cblockW", 0x10: "cblockS", 0x11: "cblockE",
         0x12: "force_N", 0x13: "force_E", 0x14: "force_W", 0x15: "exit",
         0x16: "door_blue", 0x17: "door_red", 0x18: "door_green", 0x19: "door_yellow",
         0x1A: "ice_SE", 0x1B: "ice_SW", 0x1C: "ice_NW", 0x1D: "ice_NE",
         0x1E: "blueblock_F", 0x1F: "blueblock_W", 0x21: "thief", 0x22: "socket",
         0x23: "btn_green", 0x24: "btn_red", 0x25: "toggle_closed", 0x26: "toggle_open",
         0x27: "btn_brown", 0x28: "btn_blue", 0x29: "teleport", 0x2A: "bomb",
         0x2B: "trap", 0x2C: "hiddenwall", 0x2D: "gravel", 0x2E: "passonce",
         0x2F: "hint", 0x30: "wallSE", 0x31: "cloner", 0x32: "force_rand",
         0x39: "exit2", 0x3A: "exit3", 0x3B: "exit4",
         0x64: "key_blue", 0x65: "key_red", 0x66: "key_green", 0x67: "key_yellow",
         0x68: "flippers", 0x69: "fireboots", 0x6A: "iceskates", 0x6B: "suction"}
    for base, nm in ((0x3C, "swim"), (0x40, "bug"), (0x44, "fireball"), (0x48, "ball"),
                     (0x4C, "tank"), (0x50, "glider"), (0x54, "teeth"), (0x58, "walker"),
                     (0x5C, "blob"), (0x60, "paramecium"), (0x6C, "chip")):
        for i, d in enumerate("NWSE"):
            n[base + i] = f"{nm}_{d}"
    return n


def _contact(cells, names, path, scale=4):
    cols = 16
    rows = 8
    cell = S * scale
    pad = 4
    lab = 11
    W = cols * (cell + pad) + pad
    H = rows * (cell + lab + pad) + pad
    from PIL import ImageDraw
    sheet = Image.new("RGBA", (W, H), (26, 30, 40, 255))
    d = ImageDraw.Draw(sheet)
    for code in range(128):
        cx = pad + (code % cols) * (cell + pad)
        cy = pad + (code // cols) * (cell + lab + pad)
        d.rectangle([cx, cy, cx + cell, cy + cell], outline=(50, 56, 72, 255))
        if code in cells:
            sheet.alpha_composite(cells[code].resize((cell, cell), Image.NEAREST), (cx, cy))
        d.text((cx + 1, cy + cell + 1), f"{code:02X}", fill=(150, 160, 180, 255))
    sheet.save(path)


if __name__ == "__main__":
    build()
    print("fertig.")
