"""Demo-Level-Bauer fuer CIRCUIT RUNNER (ASCII -> JSON-Set).

Schreibt `levels/circuit_runner.json` im selben Schema, das auch
convert_dat.py erzeugt -- so ist das Spiel sofort spielbar (ohne Downloads)
und das Levelformat wird abgedeckt. Ein Zeichen = eine 32x32-Map-Zelle;
nicht belegte Zellen werden mit Wand aufgefuellt.

    py circuitrunner/make_demo_levels.py
"""
from __future__ import annotations

import json
from pathlib import Path

W = H = 32

# Zeichen -> Tile-Code (CC-Objektcode = Tileset-Zelle).
# Bewegliche Dinge (Spieler/Monster/Block) landen im OBER-Layer, ihr
# Untergrund ist Boden; alles andere ist Terrain im Ober-Layer.
TILE = {
    " ": 0x00, ".": 0x00,
    "#": 0x01,
    "+": 0x02,                 # Chip
    "~": 0x03,                 # Wasser
    "^": 0x04,                 # Feuer
    "_": 0x0C,                 # Eis
    ":": 0x0B,                 # Dreck (wird zu Boden)
    ",": 0x2D,                 # Kies
    "X": 0x15,                 # Exit
    "O": 0x22,                 # Socket
    "?": 0x2F,                 # Hinweis
    "*": 0x2A,                 # Bombe
    "V": 0x2B,                 # Falle
    "H": 0x21,                 # Dieb
    "@": 0x29,                 # Teleporter
    "u": 0x12, "d": 0x0D, "l": 0x14, "i": 0x13,   # Force N/S/W/E
    "m": 0x25, "n": 0x26,      # Toggle zu / offen
    # Tueren 1-4 / Schluessel 5-8
    "1": 0x16, "2": 0x17, "3": 0x18, "4": 0x19,
    "5": 0x64, "6": 0x65, "7": 0x66, "8": 0x67,
    # Stiefel
    "w": 0x68, "f": 0x69, "s": 0x6A, "o": 0x6B,
    # Knoepfe
    "g": 0x23, "r": 0x24, "b": 0x27, "c": 0x28,
    "M": 0x31,                 # Klon-Maschine
}
# bewegliche Objekte: Code im Ober-Layer, Boden darunter
ENTITY = {
    "P": 0x6C,                 # Spieler-Start (Blick N)
    "B": 0x0A,                 # schiebbarer Block
    "a": 0x40, "f2": 0x44, "o2": 0x48, "T": 0x4C, "L": 0x50,
    "t": 0x54, "z": 0x58, "k": 0x5C, "p": 0x60,
}
# (Monster: bug a / tank T / glider L / teeth t / walker z(Hantel) /
#  blob k / paramecium p -- jeweils Blick N)


def build_level(title, rows, hint="", time=0, traps=None, cloners=None,
                monsters=None, password=""):
    upper = [0x01] * (W * H)       # mit Wand vorbelegen
    lower = [0x00] * (W * H)
    chips = 0
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if x >= W or y >= H:
                continue
            idx = y * W + x
            if ch in ENTITY:
                upper[idx] = ENTITY[ch]
                lower[idx] = 0x00
            elif ch in TILE:
                upper[idx] = TILE[ch]
                lower[idx] = 0x00
                if TILE[ch] == 0x02:
                    chips += 1
            else:
                upper[idx] = 0x00  # Unbekannt -> Boden
                lower[idx] = 0x00
    # restliche (nicht beschriebene) Zellen bleiben Wand
    for y in range(len(rows), H):
        pass

    def hx(t):
        return "".join(f"{v & 0xFF:02X}" for v in t)

    def pk(seq):
        return ";".join(",".join(str(v) for v in r) for r in (seq or []))

    return {
        "title": title, "number": 0, "time": time, "chips": chips,
        "hint": hint, "password": password, "width": W, "height": H,
        "upper": hx(upper), "lower": hx(lower),
        "traps": pk(traps), "cloners": pk(cloners), "monsters": pk(monsters),
    }


# ============================================================ LEVELS
def levels():
    out = []

    # --- 1: BOOT UP -----------------------------------------------------
    out.append(build_level("BOOT UP", [
        "##############",
        "#............#",
        "#.+..+..+..+.#",
        "#............#",
        "#.?.......O..#",
        "#....P.......#",
        "#............#",
        "#.+..+..+..+.#",
        "#............#",
        "#.........XX.#",
        "##############",
    ], hint="Sammle alle Chips, dann oeffnet sich der Sockel zum Ausgang.",
        monsters=[]))

    # --- 2: LOCKED ------------------------------------------------------
    out.append(build_level("LOCKED", [
        "#################",
        "#P...#5..#6....+#",
        "#....#...#......#",
        "####1####2#######",
        "#......#........#",
        "#.+....#..7..#..#",
        "#......#.....#.O#",
        "###3############.",
        "#........#.....X#",
        "#...8....3......#",
        "#........#......#",
        "#################",
    ], hint="Schluessel oeffnen gleichfarbige Tueren. Gruen bleibt erhalten.",
        monsters=[]))

    # --- 3: FLOODED -----------------------------------------------------
    out.append(build_level("FLOODED", [
        "###############",
        "#P....w......+#",
        "#.###########.#",
        "#.#~~~~~~~~~#.#",
        "#.#~~B~~B~~~#.#",
        "#.#~~~~~~~~~#.#",
        "#.+.........O#",
        "#.#~~~~~~~~~#.#",
        "#.#~~~~~~~~~#.#",
        "#.###########.#",
        "#....XX.......#",
        "###############",
    ], hint="Floesse (Bloecke) ins Wasser schieben macht Boden. Flossen schuetzen.",
        monsters=[]))

    # --- 4: SLIPPERY ----------------------------------------------------
    out.append(build_level("SLIPPERY", [
        "###############",
        "#P..o........+#",
        "#.iiiiiiiiii..#",
        "#.............#",
        "#._________..#",
        "#.+.......__.#",
        "#.________.O.#",
        "#.........dd.#",
        "#.+.......dd.#",
        "#....XX...dd.#",
        "###############",
    ], hint="Eis rutscht bis zur Wand, Force-Boeden schieben. Saug-Stiefel helfen.",
        monsters=[]))

    # --- 5: INFESTED ----------------------------------------------------
    out.append(build_level("INFESTED", [
        "###############",
        "#P..f........+#",
        "#.^^^^^^^^^^..#",
        "#.............#",
        "#.a....T....p.#",
        "#.+.........O.#",
        "#...........+.#",
        "#.**.......**.#",
        "#.....XX......#",
        "###############",
    ], hint="Feuer-Stiefel gegen Flammen. Monster und Bomben sind toedlich.",
        monsters=[(2, 4), (7, 4), (12, 4)]),)

    for i, lv in enumerate(out):
        lv["number"] = i + 1
    return out


def main():
    out = Path(__file__).resolve().parent / "levels" / "circuit_runner.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    data = {"name": "Circuit Runner", "ruleset": "ms", "levels": levels()}
    out.write_text(json.dumps(data, ensure_ascii=False, separators=(",", ":")),
                   encoding="utf-8")
    print(f"{len(data['levels'])} Demo-Level -> {out}")


if __name__ == "__main__":
    main()
