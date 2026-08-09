"""Generiert einen Mario-Stil-Charaktersprite fuer die Platformer-Demo.

5 Frames, je 16x16, NES-Style-Palette:
    0 = idle      (stehen, neutral)
    1 = walk_a    (linkes Bein vor)
    2 = walk_b    (rechtes Bein vor)
    3 = jump      (Beine angezogen, ein Arm hoch)
    4 = hit       (umgekippt, X-Augen)

Output (alle in examples/assets/):
    mario.dhsprite       - natives Editor-Format (zum Nach-Pixeln in dhsprites)
    mario.png            - horizontaler Sheet (5 * 16 = 80px breit)
    mario_atlas.json     - Sprite-Atlas mit benannten Frames

Verwendung im Spiel:
    DIM atlas AS SPRITE_ATLAS
    atlas = ATLAS_LOAD("assets/mario_atlas.json")
    ATLAS_DRAW(atlas, "mario_idle", x, y)
    ' Oder ueber SPRITE_NEW + SPRITE_ADD_ANIM (sheet-basiert).

Aufruf:
    .venv\\Scripts\\python.exe tools\\make_mario_sprite.py
"""
from __future__ import annotations

import base64
import io
import json
from pathlib import Path

from PIL import Image


# --- Palette ---------------------------------------------------------

# Klassisch NES-Mario-inspirierte Farben. .dhsprite hat Alpha, also "."
# ist transparent (Background).
PALETTE = {
    ".": (0, 0, 0, 0),           # transparent
    "R": (200, 30, 30, 255),     # rot (Cap, Overalls)
    "B": (40, 100, 250, 255),    # blau (Shirt, Overalls-Hosenbein)
    "S": (252, 200, 160, 255),   # skin
    "M": (90, 50, 0, 255),       # dunkles braun (Haar, Mustache)
    "X": (20, 20, 20, 255),      # schwarz (Augen-Pupille, Outlines)
    "W": (255, 255, 255, 255),   # weiss (Hosenkn?pfe, X-Augen)
    "O": (140, 80, 20, 255),     # schuh-braun
    "P": (240, 180, 150, 255),   # nasen-pink (etwas heller als Skin)
    "Y": (240, 200, 80, 255),    # gelb (Knoepfe Overalls)
}


# --- Frame-Designs ---------------------------------------------------
# 16x16 Strings. Jede Zeile = eine Pixel-Reihe, jeder Char = ein Pixel-
# Farbcode aus PALETTE. Whitespace zwischen Chars wird ignoriert.

IDLE = """
. . . . . R R R R R R . . . . .
. . . . R R R R R R R R . . . .
. . . . M M M S S S S S . . . .
. . . M M S S M S S S M . . . .
. . . M S S S M S X S M . . . .
. . . M S S S M S S S M . . . .
. . . . M M S S P S S . . . . .
. . . . . M M M M M . . . . . .
. . . . S S S S S S . . . . . .
. . . R B B Y B B B R . . . . .
. . R R B B B B B B R R . . . .
. R R R B B B B B B R R R . . .
. R . R B B B B B B R . R . . .
. R . . R R R R R R . . R . . .
. . . . O O . . O O . . . . . .
. . . . O O O . O O O . . . . .
"""

# Walk A: linkes Bein vor (links nach unten gewinkelt)
WALK_A = """
. . . . . R R R R R R . . . . .
. . . . R R R R R R R R . . . .
. . . . M M M S S S S S . . . .
. . . M M S S M S S S M . . . .
. . . M S S S M S X S M . . . .
. . . M S S S M S S S M . . . .
. . . . M M S S P S S . . . . .
. . . . . M M M M M . . . . . .
. . . . S S S S S S . . . . . .
. . . R B B Y B B B R . . . . .
. . R R B B B B B B R R . . . .
. R R R B B B B B B R R R . . .
. R . R R R R R B B R . R . . .
. . . R R R R R B B R . . . . .
. . . O O O . . R R . . . . . .
. . O O O O O . . . . . . . . .
"""

# Walk B: rechtes Bein vor (Spiegelung von A)
WALK_B = """
. . . . . R R R R R R . . . . .
. . . . R R R R R R R R . . . .
. . . . M M M S S S S S . . . .
. . . M M S S M S S S M . . . .
. . . M S S S M S X S M . . . .
. . . M S S S M S S S M . . . .
. . . . M M S S P S S . . . . .
. . . . . M M M M M . . . . . .
. . . . S S S S S S . . . . . .
. . . R B B Y B B B R . . . . .
. . R R B B B B B B R R . . . .
. R R R B B B B B B R R R . . .
. R . R B B R R R R R . R . . .
. . . . R R R R R R R . . . . .
. . . . R R . . O O O . . . . .
. . . . . . . O O O O O . . . .
"""

# Jump: beide Beine angezogen, ein Arm hoch
JUMP = """
. . . . R R R R R R . . . . . .
. . . R R R R R R R R . . . . .
. . . M M M S S S S R . . . . .
. . M M S S M S S S M . . . . .
. . M S X S M S S S M . . . . .
. . M S S S M S S S M . . . . .
. . . M M S S P S S . . . . . .
. R . . M M M M M . . . . . . .
. R . . S S S S S . . . . . . .
R R . R B B Y B B R . . . . . .
. R R R B B B B B B R R . . . .
. . R R B B B B B B R R . . . .
. . . R R B B B B R R . . . . .
. . . . R R R R R R . . . . . .
. . . O O O . . O O O . . . . .
. . . O O . . . . O O . . . . .
"""

# Hit: umgekippt, X-Augen (W = white outline), Sterne weggelassen
HIT = """
. . . . . . . . . . . . . . . .
. . . . . . . R R . . . . . . .
. . R R . . R R R R R . . . . .
. R R R R R R R R R R M M M . .
. . R R R M M S X X S M S S . .
. . . M M S X X X X X S S S M .
. . . . M M M M M M M S S M . .
. . . . . . O O O O S S M . . .
. . . R R B B B B B B . . . . .
. . R R B B Y B B B B R . . . .
. R R B B B B B B B R R . . . .
. . R R B B B B B B R R . . . .
. . . R R R R R R R R . . . . .
. . . . . . . . . . . . . . . .
. . . . . . . . . . . . . . . .
. . . . . . . . . . . . . . . .
"""


FRAMES_DESIGN = [
    ("mario_idle",   IDLE,   125),
    ("mario_walk_a", WALK_A, 110),
    ("mario_walk_b", WALK_B, 110),
    ("mario_jump",   JUMP,   125),
    ("mario_hit",    HIT,    250),
]


def _render_frame(design: str) -> Image.Image:
    """Rendert eine 16x16-RGBA-Image aus dem String-Pattern."""
    img = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    rows = [r for r in design.strip("\n").splitlines() if r.strip()]
    if len(rows) != 16:
        raise ValueError(
            f"Frame muss 16 Zeilen haben, hat {len(rows)}"
        )
    for y, row in enumerate(rows):
        chars = row.split()
        if len(chars) != 16:
            raise ValueError(
                f"Zeile {y} muss 16 Zeichen haben, hat {len(chars)}: {row!r}"
            )
        for x, ch in enumerate(chars):
            color = PALETTE.get(ch)
            if color is None:
                raise ValueError(f"Unbekannter Farbcode '{ch}' in Zeile {y}")
            img.putpixel((x, y), color)
    return img


def _save_native_gbsprite(frames_with_dur, path: Path):
    """Schreibt die Frames im .dhsprite-Format (JSON + base64-RGBA pro
    Frame). Format-Spec siehe drachenhauch/spriteeditor/document.py."""
    data = {
        "version": 2,
        "width": 16,
        "height": 16,
        "frames": [],
    }
    for img, dur_ms in frames_with_dur:
        buf = io.BytesIO()
        img.save(buf, format="PNG")
        data["frames"].append({
            "data": base64.b64encode(buf.getvalue()).decode("ascii"),
            "duration_ms": dur_ms,
        })
    path.write_text(json.dumps(data, indent=2), encoding="utf-8")


def _save_sheet(frames, path: Path):
    """Horizontaler Sheet: Frame 0, 1, 2, ... nebeneinander."""
    n = len(frames)
    sheet = Image.new("RGBA", (16 * n, 16), (0, 0, 0, 0))
    for i, f in enumerate(frames):
        sheet.paste(f, (i * 16, 0))
    sheet.save(path, format="PNG")


def _save_atlas(frame_names, sheet_filename: str, path: Path):
    """JSON-Atlas-Manifest mit benannten Sub-Rects."""
    manifest = {
        "image": sheet_filename,
        "sprites": {
            name: [i * 16, 0, 16, 16]
            for i, name in enumerate(frame_names)
        },
    }
    path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")


def main():
    out_dir = Path(__file__).resolve().parent.parent / "examples" / "assets"
    out_dir.mkdir(exist_ok=True)
    print(f"Output-Verzeichnis: {out_dir}")

    # Frames rendern
    frames = []
    frames_with_dur = []
    names = []
    for name, design, dur in FRAMES_DESIGN:
        img = _render_frame(design)
        frames.append(img)
        frames_with_dur.append((img, dur))
        names.append(name)
        print(f"  {name}: 16x16, {dur}ms")

    # .dhsprite (native) -- zum Oeffnen + Nachpixeln im Editor
    gbsprite = out_dir / "mario.dhsprite"
    _save_native_gbsprite(frames_with_dur, gbsprite)
    print(f"  -> {gbsprite.name}")

    # PNG-Sheet (horizontal)
    sheet = out_dir / "mario.png"
    _save_sheet(frames, sheet)
    print(f"  -> {sheet.name}  ({16 * len(frames)}x16 px)")

    # Atlas-Manifest
    atlas = out_dir / "mario_atlas.json"
    _save_atlas(names, "mario.png", atlas)
    print(f"  -> {atlas.name}")

    print()
    print("Fertig. Im Spiel laden via:")
    print('  atlas = ATLAS_LOAD("assets/mario_atlas.json")')
    print('  ATLAS_DRAW(atlas, "mario_idle", x, y)')
    print()
    print("Nachpixeln im Editor:")
    print(f"  dhsprites {gbsprite}")


if __name__ == "__main__":
    main()
