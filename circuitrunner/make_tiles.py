"""Tileset-Generator fuer CIRCUIT RUNNER -- 32x32, prozedural, detailliert.

Erzeugt EIN Master-Sheet `assets/tiles.png` (16 Spalten x 8 Zeilen, je 32px),
in dem die **Zellen-Position dem Chip's-Challenge-Tile-Code entspricht**
(Code 0x00..0x7F). Die Engine zeichnet damit jede Kachel/Figur per
`DRAWIMAGEPART(sheet, (code%16)*32, (code//16)*32, 32, 32, ...)`.

Zusaetzlich: `assets/tiles.gbsprite` (im Editor `gbsprites` editierbar) und
`assets/icons/*.png` (HUD-Icons, in nativer Aufloesung scharf gezeichnet).

Eigenstaendiges "Neon-Circuit"-Thema (Spielprinzip + Tile-Codes nachgebaut,
keine Original-Grafik). Ruhige, dunkle Boeden (lesen NICHT als Pickups),
klare leuchtende Items, schattierte Figuren.

Aufruf:  py circuitrunner/make_tiles.py
"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

from PIL import Image

_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

OUT = Path(__file__).resolve().parent
ASSETS = OUT / "assets"
S = 32  # Kantenlaenge

# ----------------------------------------------------------------- Palette
T = (0, 0, 0, 0)
OL = (8, 11, 16, 255)
# Boden: ruhig, dunkel, klare Abgrenzung zur Wand
FLOOR = (28, 35, 48, 255)
FLOOR_D = (20, 26, 37, 255)
FLOOR_L = (38, 47, 63, 255)
# Wand: hell + plastisch (klarer Kontrast zum Boden)
WALL = (104, 120, 146, 255)
WALL_L = (150, 168, 198, 255)
WALL_D = (60, 72, 96, 255)
WALL_DD = (44, 54, 74, 255)
STEEL = (126, 138, 160, 255)
WATER = (44, 116, 222, 255)
WATER_L = (120, 196, 255, 255)
WATER_D = (24, 74, 168, 255)
WATER_DD = (16, 52, 124, 255)
FIRE = (252, 132, 36, 255)
FIRE_L = (255, 224, 110, 255)
FIRE_D = (206, 56, 20, 255)
FIRE_DD = (150, 32, 14, 255)
ICE = (182, 230, 248, 255)
ICE_L = (230, 250, 255, 255)
ICE_D = (124, 188, 222, 255)
ICE_DD = (92, 154, 190, 255)
DIRT = (134, 100, 64, 255)
DIRT_L = (170, 130, 88, 255)
DIRT_D = (96, 70, 44, 255)
GRAVEL = (98, 100, 108, 255)
GRAVEL_L = (138, 140, 148, 255)
GRAVEL_D = (70, 72, 80, 255)
CHIPG = (74, 236, 146, 255)
CHIPG_D = (28, 156, 92, 255)
CHIPG_DD = (16, 104, 62, 255)
CHIPG_L = (180, 255, 214, 255)
GOLD = (252, 208, 74, 255)
GOLD_D = (196, 152, 38, 255)
NEON = (96, 244, 234, 255)
NEON_D = (34, 158, 158, 255)
NEON_DD = (20, 96, 100, 255)
MAGENTA = (238, 78, 206, 255)
MAGENTA_D = (150, 36, 120, 255)
WHITE = (242, 246, 252, 255)
BLACK = (14, 16, 22, 255)
PURP = (120, 96, 196, 255)
PURP_L = (170, 148, 234, 255)
PURP_D = (78, 60, 140, 255)
# Schluessel/Tuer-Farben
KEYBLUE = (74, 138, 252, 255); KEYBLUE_D = (40, 84, 188, 255); KEYBLUE_L = (150, 190, 255, 255)
KEYRED = (238, 76, 76, 255);  KEYRED_D = (172, 42, 42, 255);  KEYRED_L = (255, 150, 150, 255)
KEYGRN = (74, 214, 96, 255);  KEYGRN_D = (40, 150, 58, 255);  KEYGRN_L = (160, 246, 176, 255)
KEYYEL = (248, 218, 64, 255); KEYYEL_D = (192, 162, 32, 255); KEYYEL_L = (255, 242, 158, 255)


def _mix(a, b, t):
    return (int(a[0] + (b[0] - a[0]) * t), int(a[1] + (b[1] - a[1]) * t),
            int(a[2] + (b[2] - a[2]) * t), 255)


class C:
    def __init__(self, w=S, h=S, bg=T):
        self.im = Image.new("RGBA", (w, h), bg)
        self.p = self.im.load()
        self.w, self.h = w, h

    def set(self, x, y, c):
        x, y = int(x), int(y)
        if 0 <= x < self.w and 0 <= y < self.h and c[3]:
            if c[3] == 255:
                self.p[x, y] = c
            else:
                bx = self.p[x, y]
                a = c[3] / 255.0
                self.p[x, y] = (int(c[0] * a + bx[0] * (1 - a)),
                                int(c[1] * a + bx[1] * (1 - a)),
                                int(c[2] * a + bx[2] * (1 - a)),
                                max(bx[3], c[3]))

    def rect(self, x0, y0, x1, y1, c):
        for y in range(int(y0), int(y1) + 1):
            for x in range(int(x0), int(x1) + 1):
                self.set(x, y, c)

    def vgrad(self, x0, y0, x1, y1, top, bot):
        y0, y1 = int(y0), int(y1)
        h = max(1, y1 - y0)
        for y in range(y0, y1 + 1):
            col = _mix(top, bot, (y - y0) / h)
            for x in range(int(x0), int(x1) + 1):
                self.set(x, y, col)

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

    def ring(self, cx, cy, r, c, t=1.4):
        for y in range(int(cy - r), int(cy + r) + 1):
            for x in range(int(cx - r), int(cx + r) + 1):
                d = (x - cx) ** 2 + (y - cy) ** 2
                if (r - t) ** 2 <= d <= r * r + r * 0.4:
                    self.set(x, y, c)

    def frame(self, c, t=1):
        for i in range(t):
            self.hline(i, self.w - 1 - i, i, c)
            self.hline(i, self.w - 1 - i, self.h - 1 - i, c)
            self.vline(i, i, self.h - 1 - i, c)
            self.vline(self.w - 1 - i, i, self.h - 1 - i, c)

    def bevel_in(self, light, dark):
        self.hline(0, self.w - 1, 0, light)
        self.vline(0, 0, self.h - 1, light)
        self.hline(0, self.w - 1, self.h - 1, dark)
        self.vline(self.w - 1, 0, self.h - 1, dark)

    def outline(self, c=OL):
        src = self.im.copy(); sp = src.load()
        for y in range(self.h):
            for x in range(self.w):
                if sp[x, y][3] < 8:
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < self.w and 0 <= ny < self.h and sp[nx, ny][3] > 180:
                            self.p[x, y] = c
                            break

    def shade(self, cx, cy, r, c):
        """weicher Glanzpunkt"""
        self.disc(cx, cy, r, c)


# ============================================================ TERRAIN
def t_floor():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, FLOOR_L, FLOOR_D)
    c.rect(1, 1, S - 2, S - 2, FLOOR)
    c.bevel_in(_mix(FLOOR, FLOOR_L, 0.5), FLOOR_D)
    # extrem dezente Maserung (kein Pickup-Look)
    c.set(7, 9, FLOOR_D); c.set(22, 14, FLOOR_D); c.set(13, 24, FLOOR_D)
    c.set(26, 6, FLOOR_L); c.set(5, 20, FLOOR_L)
    return c.im


def t_wall():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, WALL_L, WALL_D)
    c.rect(2, 2, S - 3, S - 3, WALL)
    c.bevel_in(WALL_L, WALL_DD)
    c.hline(2, S - 3, 2, WALL_L)
    c.vline(2, 2, S - 3, WALL_L)
    c.hline(2, S - 3, S - 3, WALL_DD)
    c.vline(S - 3, 2, S - 3, WALL_DD)
    # Nieten
    for rx, ry in ((6, 6), (S - 7, 6), (6, S - 7), (S - 7, S - 7)):
        c.disc(rx, ry, 1.6, WALL_DD); c.disc(rx, ry, 1, WALL_L)
    # Kreuzfuge
    c.rect(S // 2 - 1, 6, S // 2, S - 7, WALL_DD)
    c.rect(6, S // 2 - 1, S - 7, S // 2, WALL_DD)
    return c.im


def t_chip():
    c = C(); c.im.alpha_composite(t_floor())
    # leuchtendes IC
    c.disc(16, 16, 12, (10, 40, 28, 120))
    c.rect(7, 7, 24, 24, CHIPG_DD)
    c.vgrad(8, 8, 23, 23, CHIPG, CHIPG_D)
    c.rect(11, 11, 20, 20, CHIPG_DD)
    c.rect(12, 12, 19, 19, CHIPG_L)
    c.rect(13, 13, 18, 18, CHIPG_D)
    # Beinchen gold
    for i in range(9, 24, 4):
        c.rect(4, i, 6, i + 1, GOLD); c.rect(25, i, 27, i + 1, GOLD)
        c.rect(i, 4, i + 1, 6, GOLD); c.rect(i, 25, i + 1, 27, GOLD)
    c.set(14, 14, WHITE)
    return c.im


def t_water():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, WATER, WATER_DD)
    for (x, y, r) in ((8, 9, 2), (20, 13, 2), (13, 22, 2), (25, 25, 1), (5, 26, 1)):
        c.disc(x, y, r, WATER_L)
    for x in range(0, S, 7):
        c.hline(x, x + 4, 4, WATER_L)
        c.hline(x + 3, x + 7, 18, _mix(WATER, WATER_L, 0.5))
    c.rect(0, 0, S - 1, 1, WATER_D)
    return c.im


def t_fire():
    c = C(); c.im.alpha_composite(t_floor())
    for fx, base, h, w, col in ((10, 27, 17, 5, FIRE_D), (21, 27, 14, 4, FIRE_D),
                                (16, 28, 22, 6, FIRE)):
        for y in range(base, base - h, -1):
            t = (base - y) / h
            sw = max(1, int(w * (1 - t)))
            c.hline(fx - sw, fx + sw, y, col)
    for fx, base, h in ((12, 27, 12), (16, 28, 16), (20, 27, 11)):
        for y in range(base, base - h, -1):
            t = (base - y) / h
            sw = max(0, int(4 * (1 - t)))
            c.hline(fx - sw, fx + sw, y, FIRE_L if t > 0.55 else FIRE)
    c.disc(16, 22, 3, FIRE_L)
    return c.im


def t_ice():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, ICE_L, ICE_D)
    c.rect(1, 1, S - 2, S - 2, ICE)
    c.bevel_in(ICE_L, ICE_DD)
    # Risse
    pts = [(6, 6), (12, 9), (15, 15), (22, 12), (26, 20), (18, 24), (9, 22)]
    for i in range(len(pts) - 1):
        (x0, y0), (x1, y1) = pts[i], pts[i + 1]
        steps = max(abs(x1 - x0), abs(y1 - y0))
        for s in range(steps + 1):
            c.set(x0 + (x1 - x0) * s / steps, y0 + (y1 - y0) * s / steps, ICE_DD)
    c.disc(23, 7, 2, ICE_L)
    return c.im


def t_ice_corner(kind):
    c = C(); c.im.alpha_composite(t_ice())
    walls = {"SE": ("S", "E"), "SW": ("S", "W"), "NW": ("N", "W"), "NE": ("N", "E")}[kind]
    for d in walls:
        if d == "S":
            c.vgrad(0, S - 5, S - 1, S - 1, WALL, WALL_D)
        elif d == "N":
            c.vgrad(0, 0, S - 1, 4, WALL_L, WALL)
        elif d == "W":
            c.rect(0, 0, 4, S - 1, WALL)
        else:
            c.rect(S - 5, 0, S - 1, S - 1, WALL_D)
    return c.im


def t_dirt():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, DIRT_L, DIRT_D)
    c.rect(1, 1, S - 2, S - 2, DIRT)
    c.bevel_in(DIRT_L, DIRT_D)
    for (x, y, r) in ((8, 9, 2), (20, 11, 2), (13, 20, 2), (24, 23, 2), (6, 24, 1), (17, 6, 1)):
        c.disc(x, y, r, DIRT_D); c.disc(x - 1, y - 1, 1, DIRT_L)
    return c.im


def t_gravel():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, GRAVEL_L, GRAVEL_D)
    c.rect(1, 1, S - 2, S - 2, GRAVEL)
    for (x, y, r) in ((6, 7, 2), (15, 6, 2), (24, 10, 2), (9, 16, 2), (20, 18, 2),
                      (27, 22, 2), (13, 25, 2), (5, 25, 1)):
        c.disc(x, y, r, GRAVEL_L); c.disc(x + 1, y + 1, 1, GRAVEL_D)
    return c.im


def _arrow(c, dx, dy, col, cx, cy, ln=4):
    if dx != 0:
        sgn = 1 if dx > 0 else -1
        for i in range(-ln, ln + 1):
            c.set(cx + i, cy, col); c.set(cx + i, cy - 1, col); c.set(cx + i, cy + 1, col)
        for k in range(5):
            c.vline(cx + sgn * (ln - k), cy - k, cy + k, col)
    else:
        sgn = 1 if dy > 0 else -1
        for i in range(-ln, ln + 1):
            c.set(cx, cy + i, col); c.set(cx - 1, cy + i, col); c.set(cx + 1, cy + i, col)
        for k in range(5):
            c.hline(cx - k, cx + k, cy + sgn * (ln - k), col)


def t_force(dx, dy):
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, PURP, PURP_D)
    c.bevel_in(PURP_L, OL)
    for k in range(2):
        cx = 16 if dy else (10 + k * 12)
        cy = (8 + k * 14) if dy else 16
        _arrow(c, dx, dy, PURP_L if k == 0 else (200, 184, 255, 255), cx, cy, 4)
    return c.im


def t_force_random():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, PURP, PURP_D)
    c.bevel_in(PURP_L, OL)
    _arrow(c, 1, 0, PURP_L, 16, 9, 4)
    _arrow(c, -1, 0, PURP_L, 16, 23, 4)
    c.disc(16, 16, 3, WHITE)
    return c.im


def t_exit():
    c = C(bg=BLACK)
    for r in range(15, 0, -1):
        col = NEON if (r // 2) % 2 == 0 else (16, 26, 36, 255)
        c.ring(16, 16, r, col, 1.6)
    c.disc(16, 16, 3, WHITE)
    c.frame(NEON_D, 2)
    c.bevel_in(NEON, NEON_DD)
    return c.im


def t_socket():
    c = C(); c.im.alpha_composite(t_floor())
    c.rect(4, 4, 27, 27, (36, 42, 58, 255))
    c.frame((26, 32, 46, 255), 2)
    c.bevel_in((50, 58, 78, 255), (18, 22, 32, 255))
    c.rect(11, 9, 20, 23, STEEL)
    c.rect(12, 10, 19, 22, (58, 64, 84, 255))
    for i in range(11, 22, 3):
        c.rect(7, i, 9, i + 1, GOLD); c.rect(22, i, 24, i + 1, GOLD)
    c.disc(16, 16, 3, CHIPG); c.disc(15, 15, 1, CHIPG_L)
    return c.im


def t_door(col, cold, coll):
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, coll, cold)
    c.frame(col, 3)
    c.rect(4, 4, S - 5, S - 5, col)
    c.rect(6, 6, S - 7, S - 7, cold)
    c.bevel_in(coll, OL)
    # Schloss
    c.disc(16, 14, 4, GOLD); c.disc(16, 14, 2, cold)
    c.rect(15, 14, 17, 21, GOLD)
    c.hline(2, S - 3, 16, _mix(cold, OL, 0.4))   # Tuerspalt
    c.shade(10, 8, 2, coll)
    return c.im


def t_blue_wall():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, KEYBLUE, KEYBLUE_D)
    c.rect(3, 3, S - 4, S - 4, KEYBLUE_D)
    c.bevel_in(KEYBLUE_L, OL)
    c.disc(16, 16, 6, KEYBLUE); c.disc(13, 13, 2, KEYBLUE_L)
    return c.im


def t_block():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, DIRT_L, DIRT_D)
    c.rect(2, 2, S - 3, S - 3, DIRT)
    c.bevel_in(_mix(DIRT_L, WHITE, 0.3), DIRT_D)
    c.frame(DIRT_D, 2)
    # Metallband-Kreuz
    for i in range(3, S - 3):
        c.set(i, i, DIRT_D); c.set(i, i - 1, DIRT_L)
        c.set(S - 1 - i, i, DIRT_D)
    c.rect(13, 13, 18, 18, (150, 116, 76, 255))
    return c.im


def t_button(col, coll):
    c = C(); c.im.alpha_composite(t_floor())
    c.disc(16, 16, 10, (24, 28, 40, 255))
    c.disc(16, 16, 8, _mix(col, OL, 0.3))
    c.disc(16, 16, 7, col)
    c.disc(13, 13, 2.5, coll)
    return c.im


def t_toggle(open_):
    if open_:
        c = C(); c.im.alpha_composite(t_floor())
        c.frame((70, 214, 110, 150), 3)
        c.disc(16, 16, 3, (70, 214, 110, 170))
        return c.im
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, (70, 150, 100, 255), (40, 96, 64, 255))
    c.rect(3, 3, S - 4, S - 4, (52, 124, 84, 255))
    c.bevel_in((110, 200, 140, 255), OL)
    for y in range(5, S - 4, 5):
        c.hline(4, S - 5, y, (40, 96, 64, 255))
    for x in range(5, S - 4, 5):
        c.vline(x, 4, S - 5, (40, 96, 64, 255))
    return c.im


def t_teleport():
    c = C(bg=BLACK)
    for r in range(14, 0, -1):
        col = MAGENTA if (r // 2) % 2 == 0 else (34, 12, 34, 255)
        c.ring(16, 16, r, col, 1.6)
    c.disc(16, 16, 3, WHITE)
    c.frame(MAGENTA_D, 2)
    return c.im


def t_bomb():
    c = C(); c.im.alpha_composite(t_floor())
    c.disc(16, 19, 9, BLACK)
    c.disc(16, 19, 9, (20, 22, 30, 255))
    c.disc(12, 15, 3, (70, 76, 92, 255))
    c.vline(18, 5, 9, (150, 120, 70, 255)); c.set(19, 6, (150, 120, 70, 255))
    c.disc(19, 4, 1.5, FIRE_L); c.disc(19, 4, 2.4, FIRE)
    return c.im


def t_trap():
    c = C(); c.im.alpha_composite(t_floor())
    c.rect(3, 3, S - 4, S - 4, (22, 24, 32, 255))
    c.frame((58, 64, 82, 255), 2)
    for x in range(5, S - 4, 4):
        c.vline(x, 5, 9, STEEL); c.vline(x + 1, 5, 8, WALL_D)
        c.vline(x, S - 10, S - 6, STEEL)
    c.rect(7, 13, S - 8, 19, BLACK)
    return c.im


def t_hint():
    c = C(); c.im.alpha_composite(t_floor())
    c.rect(5, 5, S - 6, S - 6, (38, 44, 60, 255))
    c.frame((58, 66, 86, 255), 2)
    c.bevel_in((70, 80, 104, 255), OL)
    # "?" gross
    c.rect(11, 8, 19, 11, NEON)
    c.rect(17, 10, 20, 15, NEON)
    c.rect(13, 14, 18, 17, NEON)
    c.rect(13, 19, 16, 22, NEON)
    return c.im


def t_thief():
    c = C(); c.im.alpha_composite(t_floor())
    c.disc(16, 14, 8, (40, 46, 64, 255))
    c.rect(6, 11, 25, 16, BLACK)
    c.disc(11, 13, 1.5, NEON); c.disc(20, 13, 1.5, NEON)
    c.rect(9, 20, 22, 28, (54, 60, 82, 255))
    c.disc(16, 23, 2, MAGENTA)
    c.outline(OL)
    return c.im


def t_cloner():
    c = C()
    c.vgrad(0, 0, S - 1, S - 1, (54, 60, 82, 255), (34, 38, 54, 255))
    c.rect(3, 3, S - 4, S - 4, (46, 52, 72, 255))
    c.bevel_in(STEEL, OL)
    c.rect(8, 8, 23, 23, (20, 22, 32, 255))
    c.rect(11, 11, 20, 20, MAGENTA)
    c.rect(12, 12, 19, 19, MAGENTA_D)
    for x, y in ((6, 6), (25, 6), (6, 25), (25, 25)):
        c.disc(x, y, 1.5, NEON)
    return c.im


def t_thin_wall(dirs):
    c = C(); c.im.alpha_composite(t_floor())
    for d in dirs:
        if d == "N":
            c.vgrad(0, 0, S - 1, 4, WALL_L, WALL)
        elif d == "S":
            c.vgrad(0, S - 5, S - 1, S - 1, WALL, WALL_D)
        elif d == "W":
            c.rect(0, 0, 4, S - 1, WALL); c.vline(0, 0, S - 1, WALL_L)
        elif d == "E":
            c.rect(S - 5, 0, S - 1, S - 1, WALL); c.vline(S - 1, 0, S - 1, WALL_D)
    return c.im


# ============================================================ ITEMS
def i_key(col, cold, coll):
    c = C()
    c.ring(11, 11, 5, col, 2.2)
    c.disc(9, 9, 1.5, coll)
    c.rect(13, 13, 16, 27, col)
    c.vline(13, 13, 27, coll)
    c.rect(16, 22, 21, 24, col)
    c.rect(16, 25, 19, 27, col)
    c.outline(OL)
    return c.im


def i_boot(col, cold, coll, mark):
    c = C()
    c.rect(9, 5, 16, 21, col)
    c.rect(9, 21, 24, 27, col)
    c.rect(9, 5, 11, 27, coll)
    c.rect(9, 25, 24, 27, cold)
    c.rect(22, 21, 24, 27, cold)
    c.shade(11, 7, 1.5, coll)
    mc = {"water": WATER_L, "fire": FIRE_L, "ice": ICE_L, "force": (200, 184, 255, 255)}[mark]
    c.disc(14, 13, 2.4, mc)
    c.outline(OL)
    return c.im


# ============================================================ FIGUREN (Blick N)
def _eyes(c, cx, cy, sp=4, r=1.6):
    c.disc(cx - sp, cy, r + 0.6, WHITE); c.disc(cx + sp, cy, r + 0.6, WHITE)
    c.disc(cx - sp, cy, 1, OL); c.disc(cx + sp, cy, 1, OL)


def b_bug():
    c = C()
    c.disc(16, 17, 11, (188, 48, 48, 255))
    c.disc(16, 16, 10, (224, 72, 72, 255))
    c.disc(16, 13, 7, (244, 110, 110, 255))
    c.vline(16, 6, 27, (130, 26, 26, 255))
    for sx, sy in ((9, 12), (23, 12), (8, 19), (24, 19), (10, 24), (22, 24)):
        c.disc(sx, sy, 1.4, (120, 24, 24, 255))
    # Fuehler
    c.set(11, 5, OL); c.set(10, 3, OL); c.set(9, 2, OL)
    c.set(21, 5, OL); c.set(22, 3, OL); c.set(23, 2, OL)
    _eyes(c, 16, 11, 4)
    c.outline()
    return c.im


def b_fireball():
    c = C()
    c.disc(16, 17, 11, FIRE_DD)
    c.disc(16, 17, 9, FIRE_D)
    c.disc(16, 16, 7, FIRE)
    c.disc(16, 14, 4, FIRE_L)
    c.disc(13, 12, 1.5, WHITE)
    c.disc(16, 5, 3, FIRE); c.disc(16, 4, 1.5, FIRE_L)
    c.outline()
    return c.im


def b_ball():
    c = C()
    c.disc(16, 16, 11, MAGENTA_D)
    c.disc(16, 16, 10, MAGENTA)
    c.disc(12, 12, 3, (255, 186, 240, 255))
    c.disc(16, 16, 1.5, WHITE)
    c.outline()
    return c.im


def b_tank():
    c = C()
    c.vgrad(4, 9, 27, 26, (86, 108, 178, 255), (44, 62, 116, 255))
    c.rect(4, 9, 5, 26, (40, 58, 110, 255)); c.rect(26, 9, 27, 26, (40, 58, 110, 255))
    c.rect(9, 13, 22, 22, (34, 50, 96, 255))
    c.disc(16, 17, 3.5, NEON); c.disc(16, 17, 2, (180, 255, 250, 255))
    c.rect(14, 1, 18, 11, STEEL); c.vline(14, 1, 11, WALL_L)
    for x in (6, 25):
        for y in range(11, 25, 3):
            c.set(x, y, (24, 36, 72, 255))
    c.outline()
    return c.im


def b_glider():
    c = C()
    for y in range(4, 27):
        half = int((y - 4) * 0.52)
        c.hline(16 - half, 16 + half, y, _mix((60, 200, 196, 255), NEON_D, (y - 4) / 23))
    c.vline(16, 4, 26, NEON)
    for y in range(10, 27):
        half = int((y - 4) * 0.52)
        c.set(16 - half, y, NEON_DD); c.set(16 + half, y, NEON_DD)
    c.disc(16, 10, 1.6, WHITE)
    c.outline()
    return c.im


def b_teeth():
    c = C()
    c.disc(16, 16, 11, PURP_D)
    c.disc(16, 15, 10, PURP)
    c.disc(16, 12, 6, PURP_L)
    _eyes(c, 16, 12, 4)
    c.rect(8, 20, 24, 24, WHITE)
    for x in range(9, 24, 3):
        c.vline(x, 20, 24, PURP_D)
    c.outline()
    return c.im


def b_walker():
    c = C()
    c.disc(16, 8, 5, STEEL); c.disc(16, 24, 5, STEEL)
    c.rect(13, 8, 18, 24, (96, 104, 124, 255))
    c.disc(16, 8, 3.5, (158, 170, 192, 255)); c.disc(16, 24, 3.5, (158, 170, 192, 255))
    c.disc(16, 8, 1.5, NEON); c.disc(16, 24, 1.5, NEON)
    c.outline()
    return c.im


def b_blob():
    c = C()
    c.disc(16, 17, 11, (46, 140, 66, 255))
    c.disc(16, 16, 10, (78, 200, 100, 255))
    c.disc(11, 12, 3, (164, 244, 176, 255))
    c.disc(21, 20, 2, (164, 244, 176, 255))
    _eyes(c, 16, 15, 4)
    c.outline()
    return c.im


def b_paramecium():
    c = C()
    for i, y in enumerate(range(6, 27, 4)):
        col = KEYYEL if i % 2 == 0 else KEYYEL_D
        c.disc(16, y, 5, col)
        c.set(9, y, OL); c.set(7, y - 1, OL)
        c.set(23, y, OL); c.set(25, y - 1, OL)
    c.disc(16, 6, 5, KEYYEL_L)
    _eyes(c, 16, 5, 3, 1.2)
    c.outline()
    return c.im


def b_player():
    c = C()
    # Beine/Schuhe
    c.rect(10, 24, 14, 29, (44, 92, 160, 255)); c.rect(18, 24, 22, 29, (44, 92, 160, 255))
    # Koerper
    c.vgrad(8, 11, 23, 26, (84, 150, 230, 255), (48, 96, 168, 255))
    c.rect(8, 11, 9, 26, (40, 84, 150, 255)); c.rect(22, 11, 23, 26, (40, 84, 150, 255))
    # Arme
    c.rect(4, 13, 7, 22, (52, 100, 172, 255)); c.rect(24, 13, 27, 22, (52, 100, 172, 255))
    # Helm
    c.disc(16, 9, 7, (206, 216, 230, 255))
    c.rect(9, 7, 22, 12, (206, 216, 230, 255))
    c.rect(9, 8, 22, 11, BLACK)            # Visier
    c.disc(12, 9, 1.4, NEON); c.disc(20, 9, 1.4, NEON)
    c.set(11, 8, (150, 255, 250, 255))
    # Brustkern
    c.disc(16, 18, 2.6, NEON); c.disc(16, 18, 1.2, WHITE)
    c.outline()
    return c.im


def rot4(base):
    return [base, base.rotate(90), base.rotate(180), base.rotate(270)]


# ============================================================ ZUSAMMENBAU
def build():
    ASSETS.mkdir(parents=True, exist_ok=True)
    cells = {}
    cells[0x00] = t_floor(); cells[0x01] = t_wall(); cells[0x02] = t_chip()
    cells[0x03] = t_water(); cells[0x04] = t_fire(); cells[0x05] = t_floor()
    cells[0x06] = t_thin_wall(["N"]); cells[0x07] = t_thin_wall(["W"])
    cells[0x08] = t_thin_wall(["S"]); cells[0x09] = t_thin_wall(["E"])
    cells[0x0A] = t_block(); cells[0x0B] = t_dirt(); cells[0x0C] = t_ice()
    cells[0x0D] = t_force(0, 1)
    cells[0x0E] = t_block(); cells[0x0F] = t_block(); cells[0x10] = t_block(); cells[0x11] = t_block()
    cells[0x12] = t_force(0, -1); cells[0x13] = t_force(1, 0); cells[0x14] = t_force(-1, 0)
    cells[0x15] = t_exit()
    cells[0x16] = t_door(KEYBLUE, KEYBLUE_D, KEYBLUE_L)
    cells[0x17] = t_door(KEYRED, KEYRED_D, KEYRED_L)
    cells[0x18] = t_door(KEYGRN, KEYGRN_D, KEYGRN_L)
    cells[0x19] = t_door(KEYYEL, KEYYEL_D, KEYYEL_L)
    cells[0x1A] = t_ice_corner("NW"); cells[0x1B] = t_ice_corner("NE")
    cells[0x1C] = t_ice_corner("SE"); cells[0x1D] = t_ice_corner("SW")
    cells[0x1E] = t_blue_wall(); cells[0x1F] = t_blue_wall()
    cells[0x21] = t_thief(); cells[0x22] = t_socket()
    cells[0x23] = t_button(KEYGRN, KEYGRN_L); cells[0x24] = t_button(KEYRED, KEYRED_L)
    cells[0x25] = t_toggle(False); cells[0x26] = t_toggle(True)
    cells[0x27] = t_button(DIRT, DIRT_L); cells[0x28] = t_button(KEYBLUE, KEYBLUE_L)
    cells[0x29] = t_teleport(); cells[0x2A] = t_bomb(); cells[0x2B] = t_trap()
    cells[0x2C] = t_floor(); cells[0x2D] = t_gravel(); cells[0x2E] = t_floor()
    cells[0x2F] = t_hint(); cells[0x30] = t_thin_wall(["S", "E"])
    cells[0x31] = t_cloner(); cells[0x32] = t_force_random()
    cells[0x39] = t_exit(); cells[0x3A] = t_exit(); cells[0x3B] = t_exit()

    pl = rot4(b_player())
    for i in range(4):
        cells[0x3C + i] = pl[i]
    for fn, base in ((b_bug, 0x40), (b_fireball, 0x44), (b_ball, 0x48), (b_tank, 0x4C),
                     (b_glider, 0x50), (b_teeth, 0x54), (b_walker, 0x58), (b_blob, 0x5C),
                     (b_paramecium, 0x60)):
        dirs = rot4(fn())
        for i in range(4):
            cells[base + i] = dirs[i]
    cells[0x64] = i_key(KEYBLUE, KEYBLUE_D, KEYBLUE_L)
    cells[0x65] = i_key(KEYRED, KEYRED_D, KEYRED_L)
    cells[0x66] = i_key(KEYGRN, KEYGRN_D, KEYGRN_L)
    cells[0x67] = i_key(KEYYEL, KEYYEL_D, KEYYEL_L)
    cells[0x68] = i_boot(WATER, WATER_D, WATER_L, "water")
    cells[0x69] = i_boot(FIRE, FIRE_D, FIRE_L, "fire")
    cells[0x6A] = i_boot(ICE_D, ICE_DD, ICE_L, "ice")
    cells[0x6B] = i_boot(PURP, PURP_D, PURP_L, "force")
    for i in range(4):
        cells[0x6C + i] = pl[i]

    cols, rows = 16, 8
    sheet = Image.new("RGBA", (cols * S, rows * S), T)
    for code, img in cells.items():
        sheet.alpha_composite(img.convert("RGBA"), ((code % cols) * S, (code // cols) * S))
    sheet.save(ASSETS / "tiles.png")

    names = _names()
    (ASSETS / "tiles.json").write_text(json.dumps(
        {"image": "tiles.png", "tile": S, "cols": cols,
         "codes": {f"0x{c:02X}": names.get(c, "?") for c in sorted(cells)}},
        indent=2), encoding="utf-8")

    # HUD-Icons einzeln (fuer scharfes Zeichnen in nativer Aufloesung)
    icons = ASSETS / "icons"
    icons.mkdir(exist_ok=True)
    for code, nm in ((0x02, "chip"), (0x64, "key_blue"), (0x65, "key_red"),
                     (0x66, "key_green"), (0x67, "key_yellow"), (0x68, "flippers"),
                     (0x69, "fireboots"), (0x6A, "iceskates"), (0x6B, "suction"),
                     (0x15, "exit")):
        cells[code].save(icons / f"{nm}.png")

    try:
        from gamebasic.spriteeditor.document import SpriteDoc, Frame
        doc = SpriteDoc(S, S)
        doc.frames = [Frame(pixels=cells[c].convert("RGBA"),
                            name=f"{c:02X}_{names.get(c, '?')}") for c in sorted(cells)]
        doc.save_native(ASSETS / "tiles.gbsprite")
    except Exception as e:
        print(f"(.gbsprite uebersprungen: {e})")

    _contact(cells, ASSETS / "_contact.png")
    print(f"tiles.png ({cols}x{rows} a {S}px) -- {len(cells)} Kacheln + {10} HUD-Icons")


def _names():
    n = {0x00: "floor", 0x01: "wall", 0x02: "chip", 0x03: "water", 0x04: "fire",
         0x05: "invwall", 0x06: "wallN", 0x07: "wallW", 0x08: "wallS", 0x09: "wallE",
         0x0A: "block", 0x0B: "dirt", 0x0C: "ice", 0x0D: "force_S",
         0x0E: "cblockN", 0x0F: "cblockW", 0x10: "cblockS", 0x11: "cblockE",
         0x12: "force_N", 0x13: "force_E", 0x14: "force_W", 0x15: "exit",
         0x16: "door_blue", 0x17: "door_red", 0x18: "door_green", 0x19: "door_yellow",
         0x1A: "ice_NW", 0x1B: "ice_NE", 0x1C: "ice_SE", 0x1D: "ice_SW",
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


def _contact(cells, path, scale=3):
    from PIL import ImageDraw
    cols, rows = 16, 8
    cell = S * scale; pad = 4; lab = 11
    W = cols * (cell + pad) + pad
    Hh = rows * (cell + lab + pad) + pad
    sheet = Image.new("RGBA", (W, Hh), (24, 28, 38, 255))
    d = ImageDraw.Draw(sheet)
    names = _names()
    for code in range(128):
        cx = pad + (code % cols) * (cell + pad)
        cy = pad + (code // cols) * (cell + lab + pad)
        d.rectangle([cx, cy, cx + cell, cy + cell], outline=(46, 52, 68, 255))
        if code in cells:
            sheet.alpha_composite(cells[code].resize((cell, cell), Image.NEAREST), (cx, cy))
        d.text((cx + 1, cy + cell + 1), f"{code:02X}", fill=(150, 160, 180, 255))
    sheet.save(path)


if __name__ == "__main__":
    build()
    print("fertig.")
