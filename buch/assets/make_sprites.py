"""Erzeugt die Start-Sprites fuer das Buch (Galaga-Clone).

Zeichnet die Pixel-Art programmatisch ueber das projekteigene SpriteDoc-Modell
und exportiert je Sprite ein PNG-Sheet (fuers Spiel) UND eine .gbsprite-Datei
(zum Oeffnen/Bearbeiten in `gbsprites`). Zusaetzlich ein hochskaliertes
Vorschaubild `preview.png` zur Sichtkontrolle.

Aufruf:  .venv\\Scripts\\python.exe buch\\assets\\make_sprites.py
"""
import sys
from pathlib import Path
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))   # Projekt-Root
from gamebasic.spriteeditor.document import SpriteDoc

OUT = Path(__file__).resolve().parent / "sprites"
OUT.mkdir(parents=True, exist_ok=True)

PAL = {
    ".": (0, 0, 0, 0),
    "W": (236, 240, 255, 255),   # weiss
    "C": (70, 180, 255, 255),    # cyan
    "Y": (250, 210, 70, 255),    # gelb (Cockpit/Augen)
    "R": (228, 64, 64, 255),     # rot
    "B": (70, 90, 180, 255),     # blau (Fluegel)
    "G": (90, 220, 120, 255),    # gruen
    "O": (245, 150, 50, 255),    # orange
    "D": (30, 40, 70, 255),      # dunkelblau (Schatten)
}

# --- Pixel-Grids (jede Zeile genau so breit wie das Sprite) ---
PLAYER = [
    "................",
    "................",
    ".......WW.......",
    "......WWWW......",
    "......WCCW......",
    "......WCCW......",
    ".....WWCCWW.....",
    ".....WCYYCW.....",
    "....WWCYYCWW....",
    "...WWCCCCCCWW...",
    "..WW.WCCCCW.WW..",
    "..W..WRCCRW..W..",
    ".....WRCCRW.....",
    "......RRRR......",
    ".......RR.......",
    "................",
]

BEE_A = [
    "................",
    "...Y........Y...",
    "....Y......Y....",
    "....RR....RR....",
    "...RRRRRRRRRR...",
    "..RRWWRRWWRRRR..",
    "..RRWWRRWWRRRR..",
    "..RRRRRRRRRRRR..",
    ".BBRRRRRRRRRRBB.",
    "BB.RRRRRRRRRR.BB",
    "......OYYO......",
    ".....OORROO.....",
    "......O..O......",
    "................",
    "................",
    "................",
]

BEE_B = [
    "................",
    "...Y........Y...",
    "....Y......Y....",
    "....RR....RR....",
    "...RRRRRRRRRR...",
    "..RRWWRRWWRRRR..",
    "..RRWWRRWWRRRR..",
    "..RRRRRRRRRRRR..",
    "...RRRRRRRRRR...",
    ".BBRRRRRRRRRRBB.",
    "BB....OYYO....BB",
    "......OORROO....",
    "......O..O......",
    "................",
    "................",
    "................",
]

BULLET = [
    "..CC..",
    ".CWWC.",
    ".CWWC.",
    ".CWWC.",
    ".CWWC.",
    ".CWWC.",
    ".CWWC.",
    "..CC..",
]


def paint(img: Image.Image, grid):
    for y, row in enumerate(grid):
        for x, ch in enumerate(row):
            img.putpixel((x, y), PAL[ch])


def make(name: str, frames: list, w: int, h: int):
    for fi, g in enumerate(frames):
        assert len(g) == h, f"{name} frame {fi}: {len(g)} Zeilen != {h}"
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
    "bee": make("bee", [BEE_A, BEE_B], 16, 16),
    "bullet": make("bullet", [BULLET], 6, 8),
}

# Hochskalierte Vorschau (alle Frames nebeneinander, x8)
tiles = []
for name, doc in docs.items():
    for f in doc.frames:
        tiles.append(f.pixels)
scale = 8
gap = 4
tw = sum(t.width for t in tiles) + gap * (len(tiles) + 1)
th = max(t.height for t in tiles) + 2 * gap
preview = Image.new("RGBA", (tw * scale, th * scale), (20, 24, 34, 255))
x = gap
for t in tiles:
    big = t.resize((t.width * scale, t.height * scale), Image.NEAREST)
    preview.alpha_composite(big, (x * scale, gap * scale))
    x += t.width + gap
preview.save(OUT / "preview.png")
print("OK -> ", OUT)
for p in sorted(OUT.glob("*")):
    print("  ", p.name)
