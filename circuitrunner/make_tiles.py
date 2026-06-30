"""Tileset-Generator fuer CIRCUIT RUNNER -- 64x64, supersampled + geschattet.

Erzeugt ein Master-Sheet `assets/tiles.png` (16 Spalten x 8 Zeilen, je 64px),
Zellen-Position = Chip's-Challenge-Tile-Code (0x00..0x7F). Jede Kachel wird in
hoher Aufloesung (SS-fach) mit weichen Formen/Verlaeufen gezeichnet und dann
mit Kantenglaettung auf 64px herunterskaliert -> deutlich detaillierter und
farbiger als reine 8-bit-Pixelart, aber weiter im klaren Sprite-Stil.

Aufruf:  py circuitrunner/make_tiles.py
"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

OUT = Path(__file__).resolve().parent
ASSETS = OUT / "assets"
S = 64                 # finale Kantenlaenge
SS = 4                 # Supersampling-Faktor (Arbeitsaufloesung)
W = S * SS             # Arbeits-Kantenlaenge (256)


def _mix(a, b, t):
    t = max(0.0, min(1.0, t))
    return (int(a[0] + (b[0] - a[0]) * t), int(a[1] + (b[1] - a[1]) * t),
            int(a[2] + (b[2] - a[2]) * t),
            int(a[3] + (b[3] - a[3]) * t) if len(a) > 3 and len(b) > 3 else 255)


T = (0, 0, 0, 0)


class P:
    """Arbeits-Canvas (W x W), gezeichnet via PIL; Koordinaten in 0..S
    (werden *SS skaliert) -- man denkt also in 64er-Einheiten."""

    def __init__(self, bg=T):
        self.im = Image.new("RGBA", (W, W), bg)
        self.d = ImageDraw.Draw(self.im)

    def _s(self, v):
        return v * SS

    def gv(self, x0, y0, x1, y1, top, bot):
        """vertikaler Verlauf (in 64er-Koords)."""
        x0, y0, x1, y1 = self._s(x0), self._s(y0), self._s(x1), self._s(y1)
        h = max(1, y1 - y0)
        for y in range(int(y0), int(y1) + 1):
            self.d.line([(x0, y), (x1, y)], fill=_mix(top, bot, (y - y0) / h))

    def gh(self, x0, y0, x1, y1, left, right):
        x0, y0, x1, y1 = self._s(x0), self._s(y0), self._s(x1), self._s(y1)
        wdt = max(1, x1 - x0)
        for x in range(int(x0), int(x1) + 1):
            self.d.line([(x, y0), (x, y1)], fill=_mix(left, right, (x - x0) / wdt))

    def rect(self, x0, y0, x1, y1, col):
        self.d.rectangle([self._s(x0), self._s(y0), self._s(x1), self._s(y1)], fill=col)

    def rrect(self, x0, y0, x1, y1, rad, col, outline=None, ow=0):
        self.d.rounded_rectangle([self._s(x0), self._s(y0), self._s(x1), self._s(y1)],
                                 radius=self._s(rad), fill=col,
                                 outline=outline, width=int(self._s(ow)) if ow else 0)

    def disc(self, cx, cy, r, col):
        self.d.ellipse([self._s(cx - r), self._s(cy - r), self._s(cx + r), self._s(cy + r)], fill=col)

    def ellipse(self, cx, cy, rx, ry, col):
        self.d.ellipse([self._s(cx - rx), self._s(cy - ry), self._s(cx + rx), self._s(cy + ry)], fill=col)

    def ring(self, cx, cy, r, w, col):
        self.d.ellipse([self._s(cx - r), self._s(cy - r), self._s(cx + r), self._s(cy + r)],
                       outline=col, width=int(self._s(w)))

    def line(self, pts, col, w):
        self.d.line([(self._s(x), self._s(y)) for x, y in pts], fill=col, width=int(self._s(w)), joint="curve")

    def poly(self, pts, col, outline=None, ow=0):
        self.d.polygon([(self._s(x), self._s(y)) for x, y in pts], fill=col,
                       outline=outline, width=int(self._s(ow)) if ow else 0)

    def frame(self, x0, y0, x1, y1, col, w):
        self.d.rectangle([self._s(x0), self._s(y0), self._s(x1), self._s(y1)],
                         outline=col, width=int(self._s(w)))

    def finish(self, glow=False):
        im = self.im
        if glow:
            g = im.filter(ImageFilter.GaussianBlur(SS * 1.2))
            im = Image.alpha_composite(g, im)
        return im.resize((S, S), Image.LANCZOS)


# ------------------------------------------------------------------ Palette
OL = (10, 13, 20, 255)
# Boden (Platine)
FLOOR_T = (44, 54, 74, 255)
FLOOR = (32, 40, 56, 255)
FLOOR_B = (22, 28, 40, 255)
FLOOR_L = (60, 74, 98, 255)
TRACE = (54, 120, 120, 110)
# Wand (Metall)
WALL_T = (150, 166, 196, 255)
WALL = (104, 120, 150, 255)
WALL_B = (62, 74, 100, 255)
WALL_D = (40, 50, 70, 255)
STEEL = (130, 142, 166, 255)
# Materialien
WATER_T = (96, 180, 255, 255); WATER = (44, 120, 226, 255); WATER_B = (20, 64, 158, 255); WATER_FOAM = (200, 236, 255, 255)
FIRE_Y = (255, 234, 130, 255); FIRE_O = (255, 150, 40, 255); FIRE_R = (224, 64, 24, 255); FIRE_D = (150, 28, 16, 255)
ICE_T = (236, 250, 255, 255); ICE = (176, 224, 248, 255); ICE_B = (108, 170, 214, 255); ICE_D = (74, 130, 176, 255)
DIRT_T = (158, 120, 78, 255); DIRT = (124, 92, 58, 255); DIRT_B = (88, 64, 40, 255)
GRAVEL = (110, 112, 122, 255); GRAVEL_T = (148, 150, 160, 255); GRAVEL_B = (78, 80, 90, 255)
GOLD_T = (255, 232, 150, 255); GOLD = (242, 196, 70, 255); GOLD_B = (176, 130, 30, 255); GOLD_D = (120, 86, 18, 255)
CHIPG_T = (150, 255, 198, 255); CHIPG = (60, 214, 130, 255); CHIPG_B = (24, 140, 84, 255); CHIPG_D = (14, 92, 58, 255)
NEON = (110, 248, 240, 255); NEON_B = (24, 150, 156, 255); NEON_D = (16, 90, 100, 255)
MAG_T = (255, 170, 240, 255); MAG = (236, 84, 206, 255); MAG_B = (150, 36, 124, 255)
PURP_T = (190, 168, 255, 255); PURP = (132, 104, 214, 255); PURP_B = (78, 58, 150, 255)
WHITE = (245, 248, 252, 255); BLACK = (16, 18, 26, 255); SHINE = (255, 255, 255, 235)

# Schluessel-/Tuer-Farbsaetze: (hell, mittel, dunkel, tief)
COL = {
    "blue":   ((150, 196, 255, 255), (74, 138, 252, 255), (38, 86, 196, 255), (22, 52, 134, 255)),
    "red":    ((255, 156, 150, 255), (236, 78, 76, 255), (176, 40, 40, 255), (118, 24, 26, 255)),
    "green":  ((158, 248, 176, 255), (70, 210, 100, 255), (40, 150, 64, 255), (24, 100, 44, 255)),
    "yellow": ((255, 240, 150, 255), (248, 212, 60, 255), (192, 158, 32, 255), (132, 104, 20, 255)),
}


# ====================================================== TERRAIN
def t_floor():
    p = P()
    p.gv(0, 0, S, S, FLOOR_T, FLOOR_B)
    p.rrect(2, 2, S - 3, S - 3, 6, FLOOR)
    p.frame(2, 2, S - 3, S - 3, FLOOR_L, 1)
    # dezente Leiterbahnen + Lötpunkte (subtil, kein Pickup-Look)
    p.line([(10, 14), (10, 40), (28, 40)], TRACE, 1.4)
    p.line([(50, 20), (40, 20), (40, 50)], TRACE, 1.4)
    p.disc(10, 14, 1.8, NEON_D); p.disc(40, 50, 1.8, NEON_D); p.disc(28, 40, 1.6, NEON_D)
    return p.finish()


def t_wall():
    p = P()
    p.gv(0, 0, S, S, WALL_T, WALL_B)
    p.rrect(3, 3, S - 4, S - 4, 7, WALL)
    p.gv(5, 5, S - 6, 30, _mix(WALL, WALL_T, 0.5), WALL)        # oberes Glanzfeld
    p.frame(3, 3, S - 4, S - 4, WALL_T, 1.5)
    p.line([(3, S - 5), (S - 4, S - 5)], WALL_D, 2)
    # Mittelfuge (Kreuz)
    p.rect(S // 2 - 1, 8, S // 2 + 1, S - 9, WALL_D)
    p.rect(8, S // 2 - 1, S - 9, S // 2 + 1, WALL_D)
    # Nieten mit Glanz
    for rx, ry in ((12, 12), (S - 12, 12), (12, S - 12), (S - 12, S - 12)):
        p.disc(rx, ry, 3.2, WALL_D); p.disc(rx, ry, 2.4, STEEL); p.disc(rx - 0.7, ry - 0.7, 1.0, WALL_T)
    return p.finish()


def t_chip():
    base = t_floor()
    p = P()
    p.im.alpha_composite(base.resize((W, W), Image.NEAREST))
    p.disc(32, 32, 22, (16, 60, 44, 130))            # Glow
    p.rrect(14, 14, 50, 50, 4, CHIPG_D)
    p.gv(16, 16, 48, 48, CHIPG_T, CHIPG_B)
    p.rrect(22, 22, 42, 42, 3, CHIPG_D)
    p.rrect(24, 24, 40, 40, 2, _mix(CHIPG, CHIPG_T, 0.4))
    p.rect(27, 27, 37, 37, CHIPG_B)
    # Gold-Beinchen
    for i in range(18, 47, 6):
        p.rrect(8, i - 1, 14, i + 1, 1, GOLD); p.rrect(50, i - 1, 56, i + 1, 1, GOLD)
        p.rrect(i - 1, 8, i + 1, 14, 1, GOLD); p.rrect(i - 1, 50, i + 1, 56, 1, GOLD)
    p.disc(28, 28, 2, SHINE)
    return p.finish()


def t_water(frame=0):
    p = P()
    p.gv(0, 0, S, S, WATER_T, WATER_B)
    for k, yy in enumerate((10, 24, 40, 54)):
        col = _mix(WATER_FOAM, WATER_T, 0.2 + k * 0.15)
        pts = []
        x = -8.0
        while x <= S + 8:
            wy = yy + math.sin((x + frame * 8) * (2 * math.pi / 32) + k) * 3
            pts.append((x, wy)); x += 5
        p.line(pts, col, 1.8)
    bob = math.sin(frame * 1.5708) * 2
    p.disc(16, 16 + bob, 3, WATER_FOAM); p.disc(44, 30 - bob, 2, WATER_FOAM); p.disc(28, 48 + bob, 2.4, WATER_FOAM)
    p.gv(0, 0, S, 6, WATER_B, WATER_T)
    return p.finish()


def t_fire(frame=0):
    base = t_floor()
    p = P()
    p.im.alpha_composite(base.resize((W, W), Image.NEAREST))
    p.disc(32, 40, 22, (120, 40, 12, 120))
    fl = [1.0, 0.84, 1.14, 0.92][frame % 4]
    sx = [0, -2, 1, -1][frame % 4]
    fw = [1.0, 0.9, 1.1, 0.95][frame % 4]
    def flame(cx, base_y, w, h, col):
        pts = [(cx - w, base_y), (cx - w * 0.5, base_y - h * 0.55),
               (cx - w * 0.2, base_y - h * 0.2), (cx, base_y - h),
               (cx + w * 0.2, base_y - h * 0.2), (cx + w * 0.5, base_y - h * 0.55),
               (cx + w, base_y)]
        p.poly(pts, col)
    flame(32, 54, 17, 40 * fl, FIRE_D)
    flame(24 + sx, 54, 8, 26 * fw, FIRE_R); flame(40 - sx, 54, 9, 30 * fl, FIRE_R)
    flame(32 + sx, 54, 11, 34 * fl, FIRE_O)
    flame(32 + sx, 52, 6, 22 * fw, FIRE_Y)
    p.disc(32 + sx, 46, 4, FIRE_Y)
    return p.finish(glow=True)


def t_ice():
    p = P()
    p.gv(0, 0, S, S, ICE_T, ICE_B)
    p.rrect(2, 2, S - 3, S - 3, 6, ICE)
    p.frame(2, 2, S - 3, S - 3, ICE_T, 1.5)
    p.line([(3, S - 5), (S - 4, S - 5)], ICE_D, 2)
    # Risse
    p.line([(12, 12), (24, 18), (30, 30), (46, 24), (54, 40)], ICE_D, 1.4)
    p.line([(18, 50), (30, 30)], ICE_D, 1.2)
    # Glanz
    p.poly([(40, 10), (50, 12), (44, 22)], (255, 255, 255, 120))
    p.disc(48, 16, 2.4, WHITE)
    return p.finish()


def t_ice_corner(closed):
    p = P()
    p.im.alpha_composite(t_ice().resize((W, W), Image.NEAREST))
    tw = 13
    edges = {"NW": ("N", "W"), "NE": ("N", "E"), "SE": ("S", "E"), "SW": ("S", "W")}[closed]
    for e in edges:
        if e == "N": p.gv(0, 0, S, tw, WALL_T, WALL)
        elif e == "S": p.gv(0, S - tw, S, S, WALL, WALL_B)
        elif e == "W": p.gh(0, 0, tw, S, WALL_T, WALL)
        else: p.gh(S - tw, 0, S, S, WALL, WALL_B)
    cen = {"NW": (tw, tw), "NE": (S - tw, tw), "SE": (S - tw, S - tw), "SW": (tw, S - tw)}[closed]
    a0, a1 = {"NW": (0, 90), "NE": (90, 180), "SE": (180, 270), "SW": (270, 360)}[closed]
    cx, cy = cen
    for r, col, wdt in ((34, ICE_D, 3), (28, ICE_T, 2), (31, WHITE, 1)):
        pts = []
        deg = a0
        while deg <= a1:
            a = math.radians(deg)
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
            deg += 4
        if len(pts) > 1:
            p.line(pts, col, wdt)
    return p.finish()


def t_dirt():
    p = P()
    p.gv(0, 0, S, S, DIRT_T, DIRT_B)
    p.rrect(2, 2, S - 3, S - 3, 6, DIRT)
    for x, y, r in ((14, 18, 4), (40, 22, 5), (26, 40, 4), (48, 46, 4), (12, 48, 3), (34, 12, 3)):
        p.disc(x, y, r, DIRT_B); p.disc(x - 1, y - 1, r * 0.5, DIRT_T)
    p.frame(2, 2, S - 3, S - 3, _mix(DIRT_T, WHITE, 0.2), 1)
    return p.finish()


def t_gravel():
    p = P()
    p.gv(0, 0, S, S, GRAVEL_T, GRAVEL_B)
    p.rrect(2, 2, S - 3, S - 3, 6, GRAVEL)
    for x, y, r in ((12, 14, 4), (30, 12, 5), (48, 20, 4), (18, 32, 5), (40, 36, 5),
                    (54, 44, 4), (26, 50, 5), (12, 48, 3)):
        p.disc(x, y, r, _mix(GRAVEL, GRAVEL_B, 0.5)); p.disc(x - 1, y - 1, r * 0.55, GRAVEL_T)
    return p.finish()


def _arrow(p, dx, dy, cx, cy, col, ln=9):
    if dx:
        s = 1 if dx > 0 else -1
        p.line([(cx - s * ln, cy), (cx + s * ln * 0.3, cy)], col, 3)
        p.poly([(cx + s * ln, cy), (cx + s * (ln - 7), cy - 6), (cx + s * (ln - 7), cy + 6)], col)
    else:
        s = 1 if dy > 0 else -1
        p.line([(cx, cy - s * ln), (cx, cy + s * ln * 0.3)], col, 3)
        p.poly([(cx, cy + s * ln), (cx - 6, cy + s * (ln - 7)), (cx + 6, cy + s * (ln - 7))], col)


def t_force(dx, dy, frame=0):
    p = P()
    p.gv(0, 0, S, S, PURP_T, PURP_B)
    p.rrect(2, 2, S - 3, S - 3, 6, PURP)
    p.frame(2, 2, S - 3, S - 3, PURP_T, 1.5)
    sp = 20.0
    off = (frame % 4) * sp / 4.0
    bright = _mix(PURP_T, WHITE, 0.4)
    i = -1
    while i < 5:
        pos = i * sp + off
        if dy != 0:
            sgn = 1 if dy > 0 else -1
            cy = pos if dy > 0 else (S - pos)
            p.line([(18, cy - sgn * 7), (32, cy + sgn * 7), (46, cy - sgn * 7)], bright, 3.5)
        else:
            sgn = 1 if dx > 0 else -1
            cx = pos if dx > 0 else (S - pos)
            p.line([(cx - sgn * 7, 18), (cx + sgn * 7, 32), (cx - sgn * 7, 46)], bright, 3.5)
        i += 1
    return p.finish()


def t_force_random(frame=0):
    p = P()
    p.gv(0, 0, S, S, PURP_T, PURP_B)
    p.rrect(2, 2, S - 3, S - 3, 6, PURP)
    p.frame(2, 2, S - 3, S - 3, PURP_T, 1.5)
    dirs = [(0, -1), (1, 0), (0, 1), (-1, 0)]
    dx, dy = dirs[frame % 4]
    _arrow(p, dx, dy, 32, 32, _mix(PURP_T, WHITE, 0.35), 11)
    p.disc(32, 32, 3.4, WHITE)
    return p.finish()


def t_exit(frame=0):
    p = P(bg=BLACK)
    for r in range(30, 2, -3):
        t = r / 30.0
        col = _mix(WHITE, NEON, 1 - t) if ((r // 3) + frame) % 2 == 0 else (12, 24, 34, 255)
        p.ring(32, 32, r, 2.4, col)
    p.disc(32, 32, 4, WHITE)
    p.frame(2, 2, S - 3, S - 3, NEON_B, 2)
    return p.finish(glow=True)


def t_socket(frame=0):
    base = t_floor()
    p = P()
    p.im.alpha_composite(base.resize((W, W), Image.NEAREST))
    p.rrect(8, 8, S - 9, S - 9, 6, (34, 40, 56, 255))
    p.frame(8, 8, S - 9, S - 9, (54, 64, 86, 255), 2)
    p.rrect(22, 16, 42, 48, 3, STEEL)
    p.gv(24, 18, 40, 46, _mix(STEEL, WHITE, 0.2), (70, 78, 98, 255))
    for i in range(20, 47, 7):
        p.rrect(14, i - 1, 20, i + 1, 1, GOLD); p.rrect(44, i - 1, 50, i + 1, 1, GOLD)
    # pulsierender Energiekern
    pulse = [0.0, 0.5, 1.0, 0.5][frame % 4]
    p.disc(32, 32, 5 + pulse * 4, (60, 214, 130, int(60 + pulse * 90)))
    p.disc(32, 32, 4, CHIPG); p.disc(30, 30, 1.6, CHIPG_T)
    return p.finish()


def t_door(name):
    lt, md, dk, dp = COL[name]
    p = P()
    p.gv(0, 0, S, S, lt, dk)
    p.rrect(3, 3, S - 4, S - 4, 8, md)
    p.frame(3, 3, S - 4, S - 4, lt, 2)
    p.gv(6, 6, S - 7, 26, _mix(md, lt, 0.55), md)       # Glanz oben
    p.rrect(10, 10, S - 11, S - 11, 5, dk)
    p.rrect(12, 12, S - 13, S - 13, 4, md)
    # Schloss / Reide-Platte
    p.disc(32, 28, 7, GOLD_B); p.disc(32, 28, 5.4, GOLD); p.disc(30.5, 26.5, 2, GOLD_T)
    p.rrect(30, 28, 34, 42, 1.5, GOLD); p.disc(32, 28, 2.2, dp)
    p.line([(4, 32), (S - 5, 32)], _mix(dk, OL, 0.4), 1.4)   # Tuerspalt
    return p.finish()


def t_blue_wall():
    lt, md, dk, dp = COL["blue"]
    p = P()
    p.gv(0, 0, S, S, lt, dk)
    p.rrect(4, 4, S - 5, S - 5, 7, dp)
    p.frame(4, 4, S - 5, S - 5, lt, 2)
    p.disc(32, 32, 10, md); p.disc(28, 28, 4, lt)
    return p.finish()


def t_block():
    p = P()
    p.gv(0, 0, S, S, (176, 138, 92, 255), DIRT_B)
    p.rrect(3, 3, S - 4, S - 4, 6, DIRT)
    p.frame(3, 3, S - 4, S - 4, (200, 162, 110, 255), 2)
    p.line([(3, S - 5), (S - 4, S - 5)], DIRT_B, 2.5)
    # Metallbeschlag (Diagonalen + Mittelplatte)
    p.line([(8, 8), (S - 8, S - 8)], DIRT_B, 2); p.line([(S - 8, 8), (8, S - 8)], DIRT_B, 2)
    p.rrect(24, 24, 40, 40, 3, (150, 116, 76, 255))
    p.frame(24, 24, 40, 40, DIRT_B, 1)
    for rx, ry in ((10, 10), (S - 10, 10), (10, S - 10), (S - 10, S - 10)):
        p.disc(rx, ry, 2.4, DIRT_B); p.disc(rx - 0.6, ry - 0.6, 1, (210, 170, 120, 255))
    return p.finish()


def t_button(name, light):
    base = t_floor()
    p = P()
    p.im.alpha_composite(base.resize((W, W), Image.NEAREST))
    p.disc(32, 33, 16, (20, 24, 36, 255))
    p.disc(32, 32, 13, _mix(name, OL, 0.35))
    p.disc(32, 32, 11, name)
    p.disc(32, 31, 9, _mix(name, light, 0.4))
    p.disc(27, 27, 3.5, light)
    return p.finish()


def t_toggle(open_, frame=0):
    pulse = [0.0, 0.5, 1.0, 0.5][frame % 4]
    if open_:
        base = t_floor()
        p = P()
        p.im.alpha_composite(base.resize((W, W), Image.NEAREST))
        p.frame(4, 4, S - 5, S - 5, (90, 230, 130, int(70 + pulse * 120)), 3)
        p.disc(32, 32, 3 + pulse * 2.5, (90, 230, 130, int(110 + pulse * 90)))
        return p.finish()
    p = P()
    p.gv(0, 0, S, S, (96, 220, 140, 255), (44, 120, 78, 255))
    p.rrect(3, 3, S - 4, S - 4, 6, (60, 150, 96, 255))
    p.frame(3, 3, S - 4, S - 4, _mix((130, 240, 160, 255), WHITE, pulse * 0.5), 2)
    grid = _mix((44, 110, 72, 255), (120, 240, 160, 255), pulse)
    for y in range(10, S - 6, 10):
        p.line([(5, y), (S - 6, y)], grid, 1.4)
    for x in range(10, S - 6, 10):
        p.line([(x, 5), (x, S - 6)], grid, 1.4)
    return p.finish()


def t_teleport(frame=0):
    p = P(bg=BLACK)
    # nach innen wandernde Ringe (Phase ueber frame) = Sog
    for r in range(28, 2, -3):
        col = _mix(WHITE, MAG, r / 28.0) if ((r // 3) + frame) % 2 == 0 else (30, 10, 30, 255)
        p.ring(32, 32, r, 2.4, col)
    # rotierende Wirbel-Arme
    for k in range(2):
        a = (frame / 4.0 + k / 2.0) * 2 * math.pi
        for rr in (10, 18, 25):
            p.disc(32 + math.cos(a + rr * 0.16) * rr,
                   32 + math.sin(a + rr * 0.16) * rr, 2.0, MAG_T)
    p.disc(32, 32, 4, WHITE)
    p.frame(2, 2, S - 3, S - 3, MAG_B, 2)
    return p.finish(glow=True)


def t_bomb(frame=0):
    base = t_floor()
    p = P()
    p.im.alpha_composite(base.resize((W, W), Image.NEAREST))
    p.disc(31, 38, 17, (8, 10, 16, 255))
    p.disc(31, 38, 15, (30, 34, 46, 255))
    p.disc(31, 38, 14, BLACK)
    p.disc(24, 31, 5, (78, 86, 104, 255)); p.disc(22, 29, 2, (140, 148, 166, 255))
    p.line([(38, 22), (44, 14), (48, 16)], (160, 130, 80, 255), 2.4)
    # zuckender, spritzender Funke
    sr = [3.2, 2.4, 4.2, 2.8][frame % 4]
    jx = [0, 1.5, -1, 1][frame % 4]
    jy = [0, -1, 1.5, -1.5][frame % 4]
    p.disc(48 + jx, 15 + jy, sr + 1.6, FIRE_R)
    p.disc(48 + jx, 15 + jy, sr, FIRE_O)
    p.disc(48 + jx, 15 + jy, sr * 0.5, FIRE_Y)
    if frame % 2 == 0:
        p.disc(52, 11, 1.3, FIRE_Y); p.disc(44, 9, 1.0, FIRE_O)
    return p.finish(glow=True)


def t_trap():
    base = t_floor()
    p = P()
    p.im.alpha_composite(base.resize((W, W), Image.NEAREST))
    p.rrect(6, 6, S - 7, S - 7, 4, (22, 26, 36, 255))
    p.frame(6, 6, S - 7, S - 7, (60, 68, 88, 255), 2)
    for x in range(10, S - 8, 7):
        p.poly([(x, 10), (x + 4, 10), (x + 2, 18)], STEEL)
        p.poly([(x, S - 10), (x + 4, S - 10), (x + 2, S - 18)], STEEL)
    p.rrect(12, 26, S - 13, 38, 2, BLACK)
    return p.finish()


def t_hint():
    base = t_floor()
    p = P()
    p.im.alpha_composite(base.resize((W, W), Image.NEAREST))
    p.rrect(8, 8, S - 9, S - 9, 6, (40, 48, 66, 255))
    p.frame(8, 8, S - 9, S - 9, (66, 78, 104, 255), 2)
    # "?" geschwungen
    p.line([(24, 22), (32, 18), (40, 24), (34, 32), (32, 38)], NEON, 4)
    p.disc(32, 46, 2.6, NEON)
    return p.finish(glow=True)


def t_thief(frame=0):
    base = t_floor()
    p = P()
    p.im.alpha_composite(base.resize((W, W), Image.NEAREST))
    pulse = [0.0, 0.5, 1.0, 0.5][frame % 4]
    p.disc(32, 26, 14, (44, 50, 70, 255))
    p.disc(32, 22, 11, (60, 68, 92, 255))           # Kapuze
    p.rrect(18, 24, 46, 32, 3, BLACK)               # Maske
    eye = _mix(NEON_D, NEON, pulse)                  # flackernde Augen
    p.disc(25, 28, 2.4 + pulse * 0.9, eye); p.disc(39, 28, 2.4 + pulse * 0.9, eye)
    p.poly([(18, 40), (46, 40), (52, 58), (12, 58)], (54, 60, 82, 255))   # Umhang
    p.disc(32, 48, 3 + pulse * 1.6, _mix(MAG_B, MAG, pulse))   # pulsierender Orb
    return p.finish(glow=True)


def t_cloner(frame=0):
    pulse = [0.0, 0.5, 1.0, 0.5][frame % 4]
    p = P()
    p.gv(0, 0, S, S, (66, 74, 100, 255), (34, 38, 54, 255))
    p.rrect(3, 3, S - 4, S - 4, 6, (50, 56, 78, 255))
    p.frame(3, 3, S - 4, S - 4, STEEL, 2)
    p.rrect(14, 14, S - 15, S - 15, 4, (18, 20, 30, 255))
    p.rrect(22, 22, 42, 42, 3, MAG_B)
    p.gv(24, 24, 40, 40, _mix(MAG_T, WHITE, pulse * 0.6), MAG)
    # Eck-LEDs umlaufend blinken
    leds = ((10, 10), (S - 10, 10), (S - 10, S - 10), (10, S - 10))
    for j, (x, y) in enumerate(leds):
        p.disc(x, y, 2.4, NEON if j == frame % 4 else NEON_D)
    return p.finish(glow=True)


def t_thin_wall(dirs):
    p = P()
    p.im.alpha_composite(t_floor().resize((W, W), Image.NEAREST))
    for d in dirs:
        if d == "N": p.gv(0, 0, S, 9, WALL_T, WALL)
        elif d == "S": p.gv(0, S - 9, S, S, WALL, WALL_B)
        elif d == "W": p.gh(0, 0, 9, S, WALL_T, WALL)
        elif d == "E": p.gh(S - 9, 0, S, S, WALL, WALL_B)
    return p.finish()


# ====================================================== ITEMS
# Pulsierender Glow-Halo HINTER einem Pickup (Sammel-Schimmer)
def _glow_halo(p, col, frame, cx=32, cy=33, r0=15):
    pulse = [0.0, 0.5, 1.0, 0.5][frame % 4]
    p.disc(cx, cy, r0 + pulse * 4, (col[0], col[1], col[2], int(26 + pulse * 48)))


def i_key(name, frame=0):
    lt, md, dk, dp = COL[name]
    p = P()
    _glow_halo(p, lt, frame, cy=30, r0=14)           # Schimmer-Halo
    # Schaft
    p.gh(28, 28, 36, 56, lt, dk)
    p.rect(28, 28, 36, 56, md)
    p.gh(28, 28, 30, 56, lt, lt)                     # Glanzkante links
    p.rect(28, 28, 30, 56, lt)
    p.rect(34, 28, 36, 56, dk)
    # Reide (Bow) -- Ring oben
    p.disc(32, 18, 14, dk); p.disc(32, 18, 12, md); p.disc(31, 16, 9, lt)
    p.disc(32, 18, 7, T)                              # Loch (transparent stanzen)
    p.ring(32, 18, 8.5, 2.2, dk)
    p.disc(27, 13, 2.4, SHINE)                        # Glanzpunkt
    # Bart (Bit) unten rechts
    p.rect(36, 44, 44, 48, md); p.rect(36, 44, 44, 46, lt)
    p.rect(36, 52, 41, 56, md); p.rect(36, 52, 41, 54, lt)
    p.disc(32, 56, 4, md); p.disc(30, 55, 1.6, lt)   # abgerundete Spitze
    # wanderndes Funkeln
    tw = [(27, 13), (37, 18), (40, 50), (29, 44)][frame % 4]
    p.disc(tw[0], tw[1], 1.8, SHINE)
    return p.finish(glow=True)


def _boot_base(p, lt, md, dk):
    # Seitenansicht-Stiefel, Zehe rechts
    p.gv(18, 8, 36, 38, lt, md)                       # Schaft
    p.rrect(18, 8, 36, 14, 3, lt)
    p.rect(18, 8, 21, 38, dk)
    p.gv(18, 36, 50, 52, md, dk)                      # Fuss
    p.rrect(42, 38, 50, 52, 4, md)
    p.rrect(16, 50, 52, 57, 3, dk)                    # Sohle
    for y in range(16, 35, 5):
        p.line([(22, y), (34, y)], lt, 1.4)           # Schnuerung
    p.disc(23, 12, 2, SHINE)


def boot_flippers(frame=0):
    lt, md, dk, dp = COL["blue"]
    p = P()
    _glow_halo(p, lt, frame)
    for cx, sh in ((38, dk), (26, md)):
        p.gv(cx - 8, 8, cx + 8, 24, lt if sh is md else md, sh)
        p.rrect(cx - 8, 8, cx + 8, 24, 4, sh)
        p.poly([(cx - 9, 24), (cx + 9, 24), (cx + 16, 56), (cx - 16, 56)], sh)
        p.line([(cx, 26), (cx, 54)], lt if sh is md else _mix(sh, lt, 0.4), 2)
    p.disc(22, 14, 2.4, SHINE)
    return p.finish()


def boot_fire(frame=0):
    p = P()
    _glow_halo(p, FIRE_O, frame)
    _boot_base(p, (255, 150, 110, 255), (214, 72, 52, 255), (150, 40, 28, 255))
    p.disc(28, 22, 3.4, FIRE_Y); p.disc(28, 22, 1.6, WHITE)
    p.gv(16, 50, 52, 53, FIRE_O, FIRE_R)
    for gx in (24, 34, 44):
        p.disc(gx, 54, 1.8, FIRE_Y)
    return p.finish(glow=True)


def boot_ice(frame=0):
    p = P()
    _glow_halo(p, ICE, frame)
    _boot_base(p, ICE_T, (188, 220, 240, 255), (120, 162, 200, 255))
    p.disc(28, 20, 2.6, WHITE)
    p.rrect(14, 56, 52, 59, 1.5, (220, 238, 252, 255))   # Kufe
    p.rect(14, 60, 52, 62, STEEL)
    return p.finish()


def boot_suction(frame=0):
    p = P()
    _glow_halo(p, PURP, frame)
    _boot_base(p, PURP_T, (150, 124, 216, 255), (96, 72, 168, 255))
    for sx in (24, 34, 44):
        p.disc(sx, 55, 3.2, (70, 54, 128, 255)); p.disc(sx, 55, 1.6, (40, 30, 80, 255))
    return p.finish()


# ====================================================== FIGUREN
def _shade_ball(p, cx, cy, r, lt, md, dk, eyes=True, eye_y=None):
    p.disc(cx, cy, r, dk)
    p.disc(cx, cy, r - 1.5, md)
    p.disc(cx - r * 0.32, cy - r * 0.32, r * 0.55, _mix(md, lt, 0.55))
    p.disc(cx - r * 0.42, cy - r * 0.42, r * 0.22, lt)
    if eyes:
        ey = eye_y if eye_y is not None else cy - r * 0.2
        for ex in (cx - r * 0.42, cx + r * 0.42):
            p.disc(ex, ey, r * 0.22, WHITE); p.disc(ex, ey + 0.5, r * 0.1, OL)


def b_bug():
    p = P()
    _shade_ball(p, 32, 34, 22, (255, 120, 120, 255), (224, 72, 72, 255), (150, 36, 36, 255), eyes=False)
    p.line([(32, 12), (32, 56)], (130, 26, 26, 255), 2)
    for sx, sy in ((18, 24), (46, 24), (16, 38), (48, 38), (20, 50), (44, 50)):
        p.disc(sx, sy, 2, (120, 24, 24, 255))
    p.line([(24, 12), (18, 4)], OL, 2); p.line([(40, 12), (46, 4)], OL, 2)   # Fuehler
    p.disc(18, 4, 2, OL); p.disc(46, 4, 2, OL)
    for ex in (24, 40):
        p.disc(ex, 22, 3.4, WHITE); p.disc(ex, 23, 1.6, OL)
    return p.finish()


def b_fireball():
    p = P()
    _shade_ball(p, 32, 34, 21, FIRE_Y, FIRE_O, FIRE_R, eyes=False)
    p.disc(28, 30, 4, FIRE_Y); p.disc(26, 28, 1.6, WHITE)
    p.poly([(32, 10), (28, 18), (36, 18)], FIRE_O); p.poly([(32, 8), (30, 14), (34, 14)], FIRE_Y)
    return p.finish(glow=True)


def b_ball():
    p = P()
    _shade_ball(p, 32, 34, 21, MAG_T, MAG, MAG_B, eyes=False)
    p.disc(25, 26, 4, (255, 200, 245, 255)); p.disc(24, 25, 1.8, WHITE)
    return p.finish()


def b_tank():
    p = P()
    p.gv(8, 16, 56, 52, (120, 146, 210, 255), (44, 62, 116, 255))
    p.rrect(8, 16, 56, 52, 6, (86, 108, 178, 255))
    p.frame(8, 16, 56, 52, (140, 166, 224, 255), 2)
    p.rrect(18, 26, 46, 44, 4, (40, 58, 110, 255))
    p.disc(32, 35, 6, NEON); p.disc(30, 33, 2.4, WHITE)
    p.rrect(28, 2, 36, 20, 2, STEEL)                  # Lauf
    for x in (12, 52):
        for y in range(22, 50, 6):
            p.disc(x, y, 1.6, (28, 40, 80, 255))
    return p.finish()


def b_glider():
    p = P()
    p.poly([(32, 6), (50, 52), (32, 42), (14, 52)], (60, 200, 196, 255), outline=NEON_B, ow=1.5)
    p.poly([(32, 10), (32, 40), (44, 48)], (40, 160, 160, 255))
    p.line([(32, 8), (32, 42)], NEON, 2)
    p.disc(32, 20, 3, WHITE)
    return p.finish(glow=True)


def b_teeth():
    p = P()
    _shade_ball(p, 32, 30, 21, PURP_T, PURP, PURP_B, eyes=True, eye_y=24)
    p.rrect(16, 40, 48, 50, 3, WHITE)                 # Maul
    for x in range(18, 48, 6):
        p.poly([(x, 40), (x + 5, 40), (x + 2, 50)], PURP_B)
    return p.finish()


def b_walker():
    p = P()
    for cy in (16, 48):
        p.disc(32, cy, 9, (96, 104, 124, 255)); p.disc(32, cy, 7.5, STEEL); p.disc(29, cy - 3, 3, (190, 200, 220, 255))
        p.disc(32, cy, 2, NEON)
    p.gv(26, 16, 38, 48, (150, 160, 182, 255), (88, 96, 116, 255))
    p.rect(26, 16, 38, 48, (110, 120, 142, 255))
    return p.finish()


def b_blob():
    p = P()
    _shade_ball(p, 32, 34, 22, (160, 248, 178, 255), (78, 204, 104, 255), (40, 140, 64, 255), eyes=True, eye_y=30)
    p.disc(22, 22, 4, (180, 250, 190, 255)); p.disc(44, 44, 3, (180, 250, 190, 255))
    return p.finish(glow=True)


def b_paramecium():
    p = P()
    lt, md, dk = COL["yellow"][0], COL["yellow"][1], COL["yellow"][2]
    for i, y in enumerate(range(12, 56, 9)):
        col = md if i % 2 == 0 else dk
        p.disc(32, y, 9, col); p.disc(29, y - 2, 3.5, _mix(col, lt, 0.6))
        p.line([(20, y), (12, y - 2)], OL, 1.6); p.line([(44, y), (52, y - 2)], OL, 1.6)
    p.disc(32, 12, 9, lt)
    for ex in (28, 36):
        p.disc(ex, 9, 2.4, WHITE); p.disc(ex, 10, 1, OL)
    return p.finish()


# ---- Spieler (4 echte Richtungen + Geh-Frames) -- Neon-Circuit-Android
PB_T = (160, 208, 255, 255); PB = (88, 156, 238, 255); PB_M = (58, 118, 202, 255)
PB_D = (40, 86, 158, 255); PB_OUT = (18, 44, 96, 255)
PHE_T = (238, 246, 252, 255); PHE = (200, 216, 240, 255); PHE_D = (140, 154, 182, 255)
PVIS = (14, 22, 38, 255); PLEG = (56, 106, 184, 255); PFOOT = (28, 62, 116, 255)
PNE = (120, 244, 236, 255); PNE_D = (38, 150, 150, 255)


def _legs(p, step):
    pairs = ((20, 36) if step == 0 else (16, 40))
    for lx in pairs:
        p.rrect(lx, 47, lx + 8, 60, 3, PB_OUT)            # Outline
        p.gv(lx + 1, 47, lx + 7, 58, PB, PB_D)
        p.rrect(lx + 1, 47, lx + 7, 58, 2, PLEG)
        p.rrect(lx, 56, lx + 8, 60, 1, PFOOT)             # Stiefel
        p.disc(lx + 4, 51, 1.1, PNE_D)                    # Knie-LED


def _torso(p):
    p.rrect(11, 20, 53, 54, 9, PB_OUT)                    # Outline-Silhouette
    p.gv(14, 23, 50, 52, PB_T, PB_D)                      # Koerper-Verlauf
    p.rrect(14, 23, 50, 52, 7, PB)
    p.gv(18, 25, 46, 34, _mix(PB, PB_T, 0.6), PB)         # oberes Glanzfeld
    p.rect(14, 23, 18, 52, PB_M)                          # Schattenkante links
    # Schulterpolster
    p.rrect(6, 24, 17, 46, 5, PB_OUT); p.rrect(7, 25, 16, 45, 4, PB_M)
    p.rrect(47, 24, 58, 46, 5, PB_OUT); p.rrect(48, 25, 57, 45, 4, PB_M)
    # Neon-Schaltbahnen
    p.line([(32, 24), (32, 32)], PNE_D, 1.4)
    p.line([(21, 45), (28, 45)], PNE_D, 1.2); p.line([(36, 45), (43, 45)], PNE_D, 1.2)


def _helm(p, dome):
    p.disc(32, 17, 15, PB_OUT)                            # Helm-Outline
    p.disc(32, 17, 13.5, dome)
    p.disc(27, 12, 4.5, _mix(dome, WHITE, 0.5))           # Glanz


def player_front(step):
    p = P(); _legs(p, step); _torso(p)
    _helm(p, PHE)
    p.rrect(18, 12, 46, 23, 5, PB_OUT)                    # Visier-Band
    p.rrect(19, 13, 45, 22, 4, PVIS)
    p.rrect(20, 14, 44, 17, 2, _mix(PVIS, PNE, 0.30))     # Visier-Schimmer
    p.disc(25, 18, 2.7, PNE); p.disc(39, 18, 2.7, PNE)    # Augen
    p.disc(24, 16, 1.3, WHITE)
    p.disc(32, 38, 6, PB_OUT)                             # Brustkern
    p.disc(32, 38, 4.6, PNE_D); p.disc(32, 38, 3, PNE); p.disc(30.5, 36.5, 1.4, WHITE)
    return p.finish()


def player_back(step):
    p = P(); _legs(p, step); _torso(p)
    _helm(p, PHE_D)
    p.rrect(29, 3, 35, 15, 2, PB_OUT); p.disc(32, 4, 2.8, PNE)            # Antenne
    p.rrect(22, 28, 42, 48, 4, PB_OUT); p.gv(24, 30, 40, 46, PB_M, PB_D)  # Rueckenpanel
    p.rrect(24, 30, 40, 46, 3, PB_M)
    p.line([(32, 31), (32, 45)], PNE_D, 1.4); p.disc(32, 38, 2.4, PNE_D)
    return p.finish()


def player_side(step, left):
    p = P()
    if step == 0:
        p.rrect(27, 47, 37, 60, 3, PB_OUT); p.gv(28, 47, 36, 58, PB, PB_D)
        p.rrect(28, 47, 36, 58, 2, PLEG); p.rrect(27, 56, 37, 60, 1, PFOOT)
    else:
        for lx in (20, 37):
            p.rrect(lx, 47, lx + 9, 60, 3, PB_OUT)
            p.rrect(lx + 1, 47, lx + 8, 58, 2, PLEG); p.rrect(lx, 56, lx + 9, 60, 1, PFOOT)
    p.rrect(19, 20, 45, 54, 8, PB_OUT)                   # Rumpf-Profil
    p.gv(22, 23, 42, 52, PB_T, PB_D); p.rrect(22, 23, 42, 52, 6, PB)
    p.rect(22, 23, 26, 52, PB_M)
    p.rrect(39, 27, 50, 45, 5, PB_OUT); p.rrect(40, 28, 49, 44, 4, PB_M)  # vorderer Arm
    _helm(p, PHE)
    p.rrect(33, 12, 49, 23, 4, PB_OUT); p.rrect(34, 13, 48, 22, 3, PVIS)  # Visier rechts
    p.rrect(35, 14, 47, 17, 2, _mix(PVIS, PNE, 0.30))
    p.disc(43, 18, 2.7, PNE); p.disc(40, 15, 1.2, WHITE)
    p.disc(31, 38, 5, PB_OUT); p.disc(31, 38, 3.4, PNE_D); p.disc(31, 38, 2, PNE)
    im = p.finish()
    if left:
        im = im.transpose(Image.FLIP_LEFT_RIGHT)
    return im


def rot4(base):
    b = base if isinstance(base, Image.Image) else base
    return [b, b.rotate(90), b.rotate(180), b.rotate(270)]


# ====================================================== ZUSAMMENBAU
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
    cells[0x16] = t_door("blue"); cells[0x17] = t_door("red")
    cells[0x18] = t_door("green"); cells[0x19] = t_door("yellow")
    cells[0x1A] = t_ice_corner("NW"); cells[0x1B] = t_ice_corner("NE")
    cells[0x1C] = t_ice_corner("SE"); cells[0x1D] = t_ice_corner("SW")
    cells[0x1E] = t_blue_wall(); cells[0x1F] = t_blue_wall()
    cells[0x21] = t_thief(); cells[0x22] = t_socket()
    cells[0x23] = t_button(COL["green"][1], COL["green"][0]); cells[0x24] = t_button(COL["red"][1], COL["red"][0])
    cells[0x25] = t_toggle(False); cells[0x26] = t_toggle(True)
    cells[0x27] = t_button(DIRT, DIRT_T); cells[0x28] = t_button(COL["blue"][1], COL["blue"][0])
    cells[0x29] = t_teleport(); cells[0x2A] = t_bomb(); cells[0x2B] = t_trap()
    cells[0x2C] = t_floor(); cells[0x2D] = t_gravel(); cells[0x2E] = t_floor()
    cells[0x2F] = t_hint(); cells[0x30] = t_thin_wall(["S", "E"])
    cells[0x31] = t_cloner(); cells[0x32] = t_force_random()
    cells[0x39] = t_exit(); cells[0x3A] = t_exit(); cells[0x3B] = t_exit()

    idle = [player_back(0), player_side(0, True), player_front(0), player_side(0, False)]
    walk = [player_back(1), player_side(1, True), player_front(1), player_side(1, False)]
    for i in range(4):
        cells[0x3C + i] = idle[i]; cells[0x6C + i] = idle[i]; cells[0x70 + i] = walk[i]
    for fn, base in ((b_bug, 0x40), (b_fireball, 0x44), (b_ball, 0x48), (b_tank, 0x4C),
                     (b_glider, 0x50), (b_teeth, 0x54), (b_walker, 0x58), (b_blob, 0x5C),
                     (b_paramecium, 0x60)):
        dirs = rot4(fn())
        for i in range(4):
            cells[base + i] = dirs[i]
    cells[0x64] = i_key("blue"); cells[0x65] = i_key("red")
    cells[0x66] = i_key("green"); cells[0x67] = i_key("yellow")
    cells[0x68] = boot_flippers(); cells[0x69] = boot_fire()
    cells[0x6A] = boot_ice(); cells[0x6B] = boot_suction()

    # --- Animations-Frames (Wasser/Feuer/Rollbaender/Exit) in Zeilen 8-9 ---
    # je animiertes Tile 4 Frames; Engine zeigt anim_base(code)+anim_frame.
    anim = [
        (0x03, 128, [t_water(f) for f in range(4)]),
        (0x04, 132, [t_fire(f) for f in range(4)]),
        (0x0D, 136, [t_force(0, 1, f) for f in range(4)]),
        (0x12, 140, [t_force(0, -1, f) for f in range(4)]),
        (0x13, 144, [t_force(1, 0, f) for f in range(4)]),
        (0x14, 148, [t_force(-1, 0, f) for f in range(4)]),
        (0x32, 152, [t_force_random(f) for f in range(4)]),
        (0x15, 156, [t_exit(f) for f in range(4)]),
        # Zeilen 10-11: Teleporter/Bombe/Toggle/Sockel/Cloner
        (0x29, 160, [t_teleport(f) for f in range(4)]),
        (0x2A, 164, [t_bomb(f) for f in range(4)]),
        (0x25, 168, [t_toggle(False, f) for f in range(4)]),
        (0x26, 172, [t_toggle(True, f) for f in range(4)]),
        (0x22, 176, [t_socket(f) for f in range(4)]),
        (0x31, 180, [t_cloner(f) for f in range(4)]),
        # Zeilen 11-13: Dieb + Schluessel + Stiefel (Schimmer/Funkeln)
        (0x21, 184, [t_thief(f) for f in range(4)]),
        (0x64, 188, [i_key("blue", f) for f in range(4)]),
        (0x65, 192, [i_key("red", f) for f in range(4)]),
        (0x66, 196, [i_key("green", f) for f in range(4)]),
        (0x67, 200, [i_key("yellow", f) for f in range(4)]),
        (0x68, 204, [boot_flippers(f) for f in range(4)]),
        (0x69, 208, [boot_fire(f) for f in range(4)]),
        (0x6A, 212, [boot_ice(f) for f in range(4)]),
        (0x6B, 216, [boot_suction(f) for f in range(4)]),
    ]
    for code, abase, frames in anim:
        cells[code] = frames[0]
        for f, img in enumerate(frames):
            cells[abase + f] = img

    cols = 16
    rows = max(cells) // cols + 1
    sheet = Image.new("RGBA", (cols * S, rows * S), T)
    for code, img in cells.items():
        sheet.alpha_composite(img.convert("RGBA"), ((code % cols) * S, (code // cols) * S))
    sheet.save(ASSETS / "tiles.png")

    names = _names()
    (ASSETS / "tiles.json").write_text(json.dumps(
        {"image": "tiles.png", "tile": S, "cols": cols,
         "codes": {f"0x{c:02X}": names.get(c, "?") for c in sorted(cells)}},
        indent=2), encoding="utf-8")
    try:
        from gamebasic.spriteeditor.document import SpriteDoc, Frame
        doc = SpriteDoc(S, S)
        doc.frames = [Frame(pixels=cells[c].convert("RGBA"),
                            name=f"{c:02X}_{names.get(c, '?')}") for c in sorted(cells)]
        doc.save_native(ASSETS / "tiles.gbsprite")
    except Exception as e:
        print(f"(.gbsprite uebersprungen: {e})")
    _contact(cells, ASSETS / "_contact.png")
    print(f"tiles.png ({cols}x{rows} a {S}px, SS={SS}) -- {len(cells)} Kacheln")


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
                     (0x5C, "blob"), (0x60, "paramecium"), (0x6C, "chip"), (0x70, "chipwalk")):
        for i, d in enumerate("NWSE"):
            n[base + i] = f"{nm}_{d}"
    return n


def _contact(cells, path, scale=2):
    cols, rows = 16, 10
    cell = S * scale; pad = 4; lab = 11
    Wd = cols * (cell + pad) + pad
    Hh = rows * (cell + lab + pad) + pad
    sheet = Image.new("RGBA", (Wd, Hh), (24, 28, 38, 255))
    d = ImageDraw.Draw(sheet)
    for code in range(160):
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
