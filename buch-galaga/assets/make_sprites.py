"""Erzeugt die Sprites fuer das Galaga-Buch.

Zeichnet Pixel-Art programmatisch ueber das SpriteDoc-Modell und exportiert je
Sprite ein PNG-Sheet (fuers Spiel) + eine .gbsprite (zum Bearbeiten in
dhsprites). preview.png zur Sichtkontrolle.

Aufruf:  .venv\\Scripts\\python.exe buch-galaga\\assets\\make_sprites.py
"""
import sys
from pathlib import Path
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from drachenhauch.spriteeditor.document import SpriteDoc

OUT = Path(__file__).resolve().parent / "sprites"
OUT.mkdir(parents=True, exist_ok=True)

PAL = {
    ".": (0, 0, 0, 0),
    "W": (236, 240, 255, 255), "C": (70, 180, 255, 255), "Y": (250, 210, 70, 255),
    "R": (228, 64, 64, 255), "B": (70, 90, 180, 255), "G": (90, 220, 120, 255),
    "O": (245, 150, 50, 255), "P": (190, 100, 240, 255), "D": (30, 40, 70, 255),
}

PLAYER = [
    "................", ".......WW.......", "......WWWW......", "......WCCW......",
    "......WCCW......", ".....WWCCWW.....", ".....WCYYCW.....", "....WWCYYCWW....",
    "...WWCCCCCCWW...", "..WW.WCCCCW.WW..", "..W..WRCCRW..W..", ".....WRCCRW.....",
    "......RRRR......", ".......RR.......", "................", "................",
]
# Gegner-Vorlage: EINE Form, aber Regionen -- so wird derselbe Bug pro Reihe in
# einer eigenen mehrfarbigen Familie eingefaerbt (Reihen klar unterscheidbar).
#   w = Fluegel   b = Koerper   e = Auge   # = dunkle Kante   . = transparent
BUG_A = [
    "................", "...w........w...", "....w......w....", "....ww....ww....",
    "...wwwwwwwwww...", "..wweewweewwww..", "..wweewweewwww..", "..wwwwbbbbwwww..",
    ".wwwwwbbbbwwwww.", "ww.wwwbbbbwww.ww", "......wbbw......", ".....wwbbww.....",
    "......#..#......", "................", "................", "................",
]
BUG_B = [
    "................", "...w........w...", "....w......w....", "....ww....ww....",
    "...wwwwwwwwww...", "..wweewweewwww..", "..wweewweewwww..", "..wwwwbbbbwwww..",
    "...wwwbbbbwww...", ".wwwwwbbbbwwwww.", "ww....wbbw....ww", "......wwbbww....",
    "......#..#......", "................", "................", "................",
]
# Drei Farb-Familien (Fluegel / Koerper / Auge / Kante). Jeder Gegner mehrfarbig,
# die Reihen klar verschieden: violett-cyan / rot-gelb / blau-tuerkis.
BUG_THEMES = [
    {"w": (190, 100, 240), "b": (236, 240, 255), "e": (70, 180, 255), "#": (60, 30, 90)},
    {"w": (228, 64, 64),   "b": (250, 210, 70),  "e": (236, 240, 255), "#": (90, 24, 24)},
    {"w": (70, 120, 210),  "b": (90, 220, 200),  "e": (236, 240, 255), "#": (24, 40, 80)},
]


def paint_regions(img, grid, theme):
    for y, row in enumerate(grid):
        assert len(row) == 16, f"Bug-Zeile {y}: Breite {len(row)} != 16"
        for x, ch in enumerate(row):
            img.putpixel((x, y), (0, 0, 0, 0) if ch == "." else (*theme[ch], 255))


def make_bug(name, theme):
    doc = SpriteDoc(16, 16)
    paint_regions(doc.frames[0].pixels, BUG_A, theme)
    doc.add_frame()
    paint_regions(doc.frames[-1].pixels, BUG_B, theme)
    doc.save_sheet_png(OUT / f"{name}.png")
    doc.save_native(OUT / f"{name}.gbsprite")
    return doc
BULLET = [
    "..CC..", ".CWWC.", ".CWWC.", ".CWWC.", ".CWWC.", ".CWWC.", ".CWWC.", "..CC..",
]
BOMB_A = [
    "..RR..", ".RYYR.", "RYOOYR", "RYOOYR", ".RYYR.", "..RR..",
]
BOMB_B = [
    "..OO..", ".OWWO.", "OWYYWO", "OWYYWO", ".OWWO.", "..OO..",
]


def paint(img, grid):
    for y, row in enumerate(grid):
        for x, ch in enumerate(row):
            img.putpixel((x, y), PAL[ch])


def make(name, frames, w, h):
    for fi, g in enumerate(frames):
        assert len(g) == h, f"{name} f{fi}: {len(g)} Zeilen != {h}"
        for ri, row in enumerate(g):
            assert len(row) == w, f"{name} f{fi} z{ri}: Breite {len(row)} != {w}"
    doc = SpriteDoc(w, h)
    paint(doc.frames[0].pixels, frames[0])
    for g in frames[1:]:
        doc.add_frame()
        paint(doc.frames[-1].pixels, g)
    doc.save_sheet_png(OUT / f"{name}.png")
    doc.save_native(OUT / f"{name}.gbsprite")
    return doc


docs = {
    "player": make("player", [PLAYER], 16, 16),
    "bug0": make_bug("bug0", BUG_THEMES[0]),
    "bug1": make_bug("bug1", BUG_THEMES[1]),
    "bug2": make_bug("bug2", BUG_THEMES[2]),
    "bullet": make("bullet", [BULLET], 6, 8),
    "bomb": make("bomb", [BOMB_A, BOMB_B], 6, 6),
}

tiles = [(n, f.pixels) for n, d in docs.items() for f in d.frames]
scale, gap = 8, 3
tw = sum(t.width for _, t in tiles) + gap * (len(tiles) + 1)
th = max(t.height for _, t in tiles) + 2 * gap
preview = Image.new("RGBA", (tw * scale, th * scale), (20, 24, 34, 255))
x = gap
for _, t in tiles:
    big = t.resize((t.width * scale, t.height * scale), Image.NEAREST)
    preview.alpha_composite(big, (x * scale, gap * scale))
    x += t.width + gap
preview.save(OUT / "preview.png")
print("OK ->", OUT)
for p in sorted(OUT.glob("*")):
    print("  ", p.name)
