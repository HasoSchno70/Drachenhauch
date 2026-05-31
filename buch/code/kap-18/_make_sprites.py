"""Einmal-Skript: zerlegt galaga_sheet.png in entity-spezifische Streifen
mit Transparenz. Output -> kap-18/sprites/.

Layout im Sheet (16x16 Zellen a 128px):
    Reihe 0    Spieler-Schiffe (rot/weiss)
    Reihe 1    Spieler-Schiffe (blau, mit Triebwerk)
    Reihe 2    Mix: Schiff + Schuesse + Explosionsfolge
    Reihen 4-5 Bienen (klein, je 1x1 Zelle)
    Reihen 6-7 Schmetterlinge (gross, je 2x2 Zellen)
    Reihen 8-9 Boss-Galagas (gross, je 2x2 Zellen)
    Reihe 10   Geschosse
    Reihe 11   Kleine Funken
    Reihen 12-15  Tractor-Beam-Szenen + Labels
"""
from PIL import Image
import numpy as np
from pathlib import Path

HERE     = Path(__file__).parent
SHEET    = HERE / "galaga_sheet.png"
OUT_DIR  = HERE / "sprites"
CELL     = 128

OUT_DIR.mkdir(exist_ok=True)
sheet = Image.open(SHEET).convert("RGBA")
print(f"Sheet: {sheet.size}, Modus: {sheet.mode}")


def remove_background(img: Image.Image) -> Image.Image:
    """Helle, kaum-saettigte Pixel als Hintergrund erkennen und transparent machen.
    Trifft sowohl weisse Backgrounds als auch das mittelgraue Raster der oberen Reihen."""
    arr = np.array(img)
    r = arr[:, :, 0].astype(int)
    g = arr[:, :, 1].astype(int)
    b = arr[:, :, 2].astype(int)
    bright = (r + g + b) / 3
    sat = arr[:, :, :3].max(axis=2).astype(int) - arr[:, :, :3].min(axis=2).astype(int)
    bg_mask = (bright > 175) & (sat < 28)
    arr[bg_mask, 3] = 0
    return Image.fromarray(arr)


def extract_strip(col: int, row: int, cells_w: int, cells_h: int,
                  target_w: int, target_h: int) -> Image.Image:
    box = (col * CELL, row * CELL, (col + cells_w) * CELL, (row + cells_h) * CELL)
    sub = sheet.crop(box)
    sub = remove_background(sub)
    return sub.resize((target_w, target_h), Image.NEAREST)


# Spieler: 1 Frame aus (Spalte 0, Reihe 0). Skaliert auf 24x24.
extract_strip(0, 0, 1, 1, 24, 24).save(OUT_DIR / "player.png")

# Bienen-Streifen: 2 Frames horizontal (cols 0-1, row 4) -> 32x16 (2x 16x16).
extract_strip(0, 4, 2, 1, 32, 16).save(OUT_DIR / "bee.png")

# Schmetterlings-Streifen: 4 Frames a 2x2 Zellen, horizontal von cols 0-7, rows 6-7
# Quelle: 1024x256, Ziel: 128x32 (4 Frames a 32x32).
extract_strip(0, 6, 8, 2, 128, 32).save(OUT_DIR / "butterfly.png")

# Boss-Streifen: gleiches Schema, rows 8-9. Boss soll groesser wirken: 192x48.
extract_strip(0, 8, 8, 2, 192, 48).save(OUT_DIR / "boss.png")

print("Fertig. Sprites in", OUT_DIR)
for f in sorted(OUT_DIR.glob("*.png")):
    im = Image.open(f)
    print(f"  {f.name:18s}  {im.size[0]}x{im.size[1]}")
