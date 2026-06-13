"""Sprite-Generator fuer einen Mario-Clone -- baut Held + Gegner ueber das
Datenmodell des Sprite-Editors (`gamebasic.spriteeditor.document.SpriteDoc`)
und exportiert sie als `.gbsprite` (in `gbsprites` weiter bearbeitbar),
Atlas (PNG + JSON fuer `ATLAS_LOAD`) und animiertes GIF.

Die Pixelart ist als ASCII-Raster (ein Zeichen = ein Pixel) notiert; `parse()`
malt daraus ein RGBA-Bild. Das ist gut lesbar und im Editor nachbearbeitbar.

Aufruf:  py examples/mario/make_sprites.py
"""
from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image

# Projekt-Root in den Pfad, damit `gamebasic` importierbar ist.
_ROOT = Path(__file__).resolve().parent.parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from gamebasic.spriteeditor.document import SpriteDoc, Frame, Anim  # noqa: E402

OUT = Path(__file__).resolve().parent

# ----------------------------------------------------------------- Paletten
# Geteilte Grundfarben; '.' = transparent. Pro Figur eigene Map moeglich.
TRANSPARENT = (0, 0, 0, 0)

HERO_PAL = {
    ".": TRANSPARENT,
    "K": (24, 16, 24, 255),     # Outline / Auge (dunkel)
    "R": (228, 52, 36, 255),    # Rot (Kappe + Hemd)
    "r": (160, 28, 20, 255),    # Rot-Schatten
    "S": (252, 188, 132, 255),  # Haut
    "s": (208, 140, 92, 255),   # Haut-Schatten
    "B": (44, 76, 208, 255),    # Blaue Latzhose
    "b": (28, 44, 132, 255),    # Blau-Schatten
    "Y": (248, 204, 48, 255),   # Knopf (gelb)
    "N": (120, 68, 28, 255),    # Braun (Haar/Schnauzer/Schuhe)
    "W": (250, 250, 250, 255),  # Weiss (Auge)
}

GOOMBA_PAL = {
    ".": TRANSPARENT,
    "K": (24, 16, 16, 255),     # Outline / Augenkontur
    "N": (150, 86, 34, 255),    # Pilzkopf braun
    "n": (104, 56, 18, 255),    # Braun-Schatten
    "T": (228, 196, 140, 255),  # heller Gesichtsbereich
    "W": (250, 250, 250, 255),  # Auge weiss
    "F": (78, 44, 16, 255),     # Fuesse dunkelbraun
}

KOOPA_PAL = {
    ".": TRANSPARENT,
    "K": (20, 28, 16, 255),     # Outline
    "G": (72, 176, 64, 255),    # Panzer/Haut gruen
    "g": (40, 112, 40, 255),    # Gruen-Schatten
    "Y": (240, 208, 96, 255),   # Bauch/Kopf gelb
    "y": (196, 156, 52, 255),   # Gelb-Schatten
    "O": (236, 140, 36, 255),   # Fuesse/Schnabel orange
    "W": (250, 250, 250, 255),  # Auge
    "K2": None,                 # (Platzhalter, ungenutzt)
}


def parse(rows: list[str], pal: dict, w: int = 16, h: int = 16) -> Image.Image:
    """ASCII-Raster -> RGBA-Bild. Jede Zeile wird auf `w` mit '.' aufgefuellt;
    fehlende Zeilen unten ebenfalls transparent. Mehr als `w`/`h` -> Fehler."""
    if len(rows) > h:
        raise ValueError(f"zu viele Zeilen: {len(rows)} > {h}")
    img = Image.new("RGBA", (w, h), TRANSPARENT)
    px = img.load()
    for y, row in enumerate(rows):
        if len(row) > w:
            raise ValueError(f"Zeile {y} zu breit: {len(row)} > {w}: {row!r}")
        for x, ch in enumerate(row):
            col = pal.get(ch, TRANSPARENT)
            if col:
                px[x, y] = col
    return img


def make_doc(w: int, h: int, frames: list[tuple[str, list[str]]], pal: dict,
             anims: list[Anim]) -> SpriteDoc:
    """Baut ein SpriteDoc aus (name, ascii_rows)-Paaren + Anim-Bereichen."""
    doc = SpriteDoc(w, h)
    doc.frames = [Frame(pixels=parse(rows, pal, w, h), name=name,
                        duration_ms=120)
                  for name, rows in frames]
    doc.anims = anims
    doc.current_index = 0
    return doc


# ============================================================ HELD (16x16)
# Roter Klempner mit blauer Latzhose -- Homage, eigenes Pixel-Layout.

H_IDLE = [
    "................",
    "....RRRRR.......",
    "...RRRRRRRR.....",
    "...NNNSSKSS.....",
    "..NSNSSKSSS.....",
    "..NSNNSSSSS.....",
    "..NNSSSSSS......",
    "....RRBRR.......",
    "...RRRBBRRR.....",
    "..RRRRBBBBRR....",
    "..SSRBYBBYBS....",
    "..SSBBBBBBSS....",
    "...BBBBBBBB.....",
    "...BBB..BBB.....",
    "..NNN....NNN....",
    "..NNNN..NNNN....",
]

# Lauf 1 -- Beine in Schrittstellung (eines vor, eines zurueck)
H_RUN1 = [
    "................",
    ".....RRRRR......",
    "....RRRRRRRR....",
    "....NNNSSKSS....",
    "...NSNSSKSSS....",
    "...NSNNSSSSS....",
    "...NNSSSSSS.....",
    "..SS.RRBRR.....",
    ".SSS RRRBBRR....",
    ".SS.RRRBBBBR....",
    "...RRBYBBYB.....",
    "...BBBBBBBB.....",
    "...BBBBBBB......",
    "..BBB..BBBB.....",
    ".NNN....NN......",
    ".NN.....NNNN....",
]

# Lauf 2 -- Beine fast zusammen (Durchgang)
H_RUN2 = [
    "................",
    "....RRRRR.......",
    "...RRRRRRRR.....",
    "...NNNSSKSS.....",
    "..NSNSSKSSS.....",
    "..NSNNSSSSS.....",
    "..NNSSSSSS......",
    "...RRRBRR..SS...",
    "..RRRRBBRRRSS...",
    "..RRRRBBBBRS....",
    "..SSRBYBBYB.....",
    "..SSBBBBBBSS....",
    "...BBBBBBBB.....",
    "....BBBBBB......",
    "...NNNNNN.......",
    "..NNN..NNN......",
]

# Lauf 3 -- andere Schrittstellung (gespiegelter Eindruck)
H_RUN3 = [
    "................",
    "....RRRRR.......",
    "...RRRRRRRR.....",
    "...NNNSSKSS.....",
    "..NSNSSKSSS.....",
    "..NSNNSSSSS.....",
    "..NNSSSSSS......",
    ".....RRBRR.SS...",
    "....RRRBBRRSSS..",
    "....RBBBBRRR.SS.",
    ".....BYBBYBRR...",
    "....BBBBBBBB....",
    "....BBBBBBB.....",
    "....BBBB.BBB....",
    "....NN....NNN...",
    "..NNNN....NN....",
]

# Sprung -- ein Arm hoch, Beine angewinkelt
H_JUMP = [
    ".........SS.....",
    "....RRRRRSSS....",
    "...RRRRRRRR.....",
    "...NNNSSKSS..S..",
    "..NSNSSKSSS.SS..",
    "..NSNNSSSSSS....",
    "..NNSSSSSS......",
    "..SSRRBRR......",
    ".SSRRRRBBRRR....",
    "..S.RRBBBBRR....",
    "...RRBYBBYBR....",
    "...BBBBBBBB.....",
    "..BBBB..BBBB....",
    ".NNNN....NNN....",
    "NNN.......NN....",
    "................",
]

# Bremsen / Skid -- lehnt zurueck, Arm ausgestreckt
H_SKID = [
    "................",
    "......RRRRR.....",
    ".....RRRRRRRR...",
    ".....SSKSSNNN...",
    "....SSSKSSNSN...",
    "....SSSSSSNSN...",
    ".....SSSSSNN....",
    "...SSRRBRR......",
    "..SS.RRBBRRR....",
    "..S.RRBBBBRRR...",
    "....BYBBYBRR....",
    "...BBBBBBBB.....",
    "...BBBBBBBB.....",
    "..BBB....BBB....",
    "..NNNN..NN......",
    "...NNN...NNNN...",
]

# Ducken -- gestaucht
H_DUCK = [
    "................",
    "................",
    "................",
    "................",
    "....RRRRR.......",
    "...RRRRRRRR.....",
    "...NNNSSKSS.....",
    "..NSNNSSKSS.....",
    "..NNSSSSSSS.....",
    "..RRRBBBBRR.....",
    ".SSRBYBBYBRS....",
    ".SSBBBBBBBBS....",
    "..BBBBBBBBB.....",
    "..NNNN..NNNN....",
    ".NNNN....NNNN...",
    "................",
]

# Tod -- Arme hoch, kleines Sterne-Gesicht
H_DEAD = [
    "................",
    "....RRRRR.......",
    "...RRRRRRRR.....",
    "...NNNSSSSS.....",
    "..NSNSKSKSS.....",
    "..NSNSSSSSS.....",
    "..NNSKKKKS......",
    "S...RRBRR...S...",
    "SS.RRRBBRRR.SS..",
    ".SSRRRBBBBR.S...",
    "...RRBYBBYB.....",
    "...BBBBBBBB.....",
    "...BBBBBBBB.....",
    "...BBB..BBB.....",
    "..NNN....NNN....",
    "..NNN....NNN....",
]

HERO_FRAMES = [
    ("idle", H_IDLE),
    ("run0", H_RUN1), ("run1", H_RUN2), ("run2", H_RUN3),
    ("jump", H_JUMP),
    ("skid", H_SKID),
    ("duck", H_DUCK),
    ("dead", H_DEAD),
]
HERO_ANIMS = [
    Anim("idle", 0, 0, 1),
    Anim("run", 1, 3, 12),
    Anim("jump", 4, 4, 1),
    Anim("skid", 5, 5, 1),
    Anim("duck", 6, 6, 1),
    Anim("dead", 7, 7, 1),
]


# ============================================================ GOOMBA (16x16)
G_WALK1 = [
    "................",
    "................",
    "....nNNNNn......",
    "..nNNNNNNNNn....",
    ".nNNNNNNNNNNn...",
    ".NNWKNNNNKWNN...",
    ".NNWKNNNNKWNN...",
    ".NNNNNKNNNNN N..",
    ".nNNNNNNNNNNn...",
    "..TTTTTTTTTT....",
    "..TTTTTTTTTT....",
    "...TTTTTTTT.....",
    "....FFF FFF.....",
    "...FFFF.FFFF....",
    "..FFFF...FFFF...",
    "................",
]

G_WALK2 = [
    "................",
    "................",
    "....nNNNNn......",
    "..nNNNNNNNNn....",
    ".nNNNNNNNNNNn...",
    ".NNWKNNNNKWNN...",
    ".NNWKNNNNKWNN...",
    ".NNNNNKNNNNNN...",
    ".nNNNNNNNNNNn...",
    "..TTTTTTTTTT....",
    "..TTTTTTTTTT....",
    "...TTTTTTTT.....",
    "...FFF..FFF.....",
    "..FFFF...FFFF...",
    "...FF.....FF....",
    "................",
]

# Plattgedrueckt
G_SQUASH = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "...nNNNNNNn.....",
    ".nNNNNNNNNNNn...",
    ".NWKNNNNNNKWN...",
    ".NNNNNNNNNNNN...",
    ".TTTTTTTTTTTT...",
    ".FFFFFFFFFFFF...",
    "................",
    "................",
]

GOOMBA_FRAMES = [("walk0", G_WALK1), ("walk1", G_WALK2), ("squash", G_SQUASH)]
GOOMBA_ANIMS = [Anim("walk", 0, 1, 6), Anim("squash", 2, 2, 1)]


# ============================================================ KOOPA (16x16)
K_WALK1 = [
    "......OOO.......",
    ".....OYYYO......",
    ".....YWKYY......",
    ".....YYKYY......",
    "....OOYYY.......",
    "...GGGGG........",
    "..GGgggGGG......",
    ".GGgKKgGGGG.....",
    ".GgKKKKgGGG.....",
    ".GggKKgggGG.....",
    ".GGgggggGGG.....",
    "..GGGGGGGG......",
    "...YY..YY.......",
    "...OO..OO.......",
    "..OOO..OOO......",
    "................",
]

K_WALK2 = [
    "......OOO.......",
    ".....OYYYO......",
    ".....YWKYY......",
    ".....YYKYY......",
    "....OOYYY.......",
    "...GGGGG........",
    "..GGgggGGG......",
    ".GGgKKgGGGG.....",
    ".GgKKKKgGGG.....",
    ".GggKKgggGG.....",
    ".GGgggggGGG.....",
    "..GGGGGGGG......",
    "....YYYY........",
    "...OO..OO.......",
    "..OOO..OOO......",
    "................",
]

KOOPA_FRAMES = [("walk0", K_WALK1), ("walk1", K_WALK2)]
KOOPA_ANIMS = [Anim("walk", 0, 1, 5)]


# ============================================================ FLATTERER (16x16)
# Kleiner fliegender Gegner ("Para") -- Fluegel auf/ab.
P_FLY1 = [
    "................",
    "..WW......WW....",
    ".WWWW....WWWW...",
    ".WWWWnNNNWWWW...",
    "..WWnNNNNnWW....",
    "...nNNNNNNn.....",
    "..NNWKNNKWNN....",
    "..NNWKNNKWNN....",
    "..NNNNKKNNNN....",
    "..nNNNNNNNNn....",
    "...TTTTTTTT.....",
    "...TTTTTTTT.....",
    "....FF..FF......",
    "...FFF..FFF.....",
    "................",
    "................",
]

P_FLY2 = [
    "..WW......WW....",
    "...WW....WW.....",
    "....WnNNNW......",
    "...WnNNNNnW.....",
    "...nNNNNNNn.....",
    "..NNWKNNKWNN....",
    "..NNWKNNKWNN....",
    "..NNNNKKNNNN....",
    "..nNNNNNNNNn....",
    "...TTTTTTTT.....",
    "...TTTTTTTT.....",
    "....FFFFFF......",
    "...FF....FF.....",
    "................",
    "................",
    "................",
]

PARA_FRAMES = [("fly0", P_FLY1), ("fly1", P_FLY2)]
PARA_ANIMS = [Anim("fly", 0, 1, 7)]


# ----------------------------------------------------------------- Bau + Export
def build_all() -> dict[str, SpriteDoc]:
    return {
        "hero": make_doc(16, 16, HERO_FRAMES, HERO_PAL, HERO_ANIMS),
        "goomba": make_doc(16, 16, GOOMBA_FRAMES, GOOMBA_PAL, GOOMBA_ANIMS),
        "koopa": make_doc(16, 16, KOOPA_FRAMES, KOOPA_PAL, KOOPA_ANIMS),
        "para": make_doc(16, 16, PARA_FRAMES, GOOMBA_PAL, PARA_ANIMS),
    }


def export(docs: dict[str, SpriteDoc]) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for name, doc in docs.items():
        doc.save_native(OUT / f"{name}.gbsprite")
        doc.save_sheet_atlas(OUT / f"{name}.png", OUT / f"{name}.json",
                             name_prefix=name)
        doc.save_animated_gif(OUT / f"{name}.gif", scale=6)
        print(f"  {name}: {len(doc.frames)} frames -> "
              f"{name}.gbsprite / {name}.png+json / {name}.gif")


def contact_sheet(docs: dict[str, SpriteDoc], path: Path, scale: int = 8) -> None:
    """Alle Frames aller Figuren als ein grosses Uebersichts-PNG (zum
    Begutachten) -- pro Figur eine Zeile, Frames nebeneinander."""
    cell = 16 * scale
    pad = 8
    cols = max(len(d.frames) for d in docs.values())
    rows = len(docs)
    W = cols * (cell + pad) + pad
    H = rows * (cell + pad) + pad
    sheet = Image.new("RGBA", (W, H), (28, 32, 44, 255))
    for r, (name, doc) in enumerate(docs.items()):
        y = pad + r * (cell + pad)
        for c, f in enumerate(doc.frames):
            x = pad + c * (cell + pad)
            big = f.composite().resize((cell, cell), Image.NEAREST)
            sheet.alpha_composite(big, (x, y))
    sheet.save(path, format="PNG")
    print(f"  Kontaktbogen -> {path.name}")


if __name__ == "__main__":
    docs = build_all()
    export(docs)
    contact_sheet(docs, OUT / "_contact.png")
    print("fertig.")
