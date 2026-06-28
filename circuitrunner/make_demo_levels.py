"""Demo-Level-Bauer fuer CIRCUIT RUNNER (ASCII -> JSON-Set) mit Validierung.

Schreibt `levels/circuit_runner.json` im selben Schema, das auch
convert_dat.py erzeugt. Ein Zeichen = eine 32x32-Map-Zelle; nicht belegte
Zellen werden mit Wand aufgefuellt.

**Validierung** (verhindert unloesbare Level): von der Spieler-Startzelle aus
ein Flood-Fill (harte Waende blockieren, Tueren/Wasser/Force/etc. passierbar,
Sockel = geschlossen). Pruefung:
  - jeder Chip / Schluessel ist erreichbar,
  - der Ausgang ist OHNE Sockel NICHT erreichbar, MIT offenem Sockel schon
    (d. h. der Sockel sperrt den Ausgang -> Chips sammeln ist noetig).
Bei Verstoss -> AssertionError mit Levelname.

    py circuitrunner/make_demo_levels.py
"""
from __future__ import annotations

import json
from pathlib import Path

W = H = 32

TILE = {
    " ": 0x00, ".": 0x00,
    "#": 0x01,
    "+": 0x02,                 # Chip
    "~": 0x03,                 # Wasser
    "^": 0x04,                 # Feuer
    "_": 0x0C,                 # Eis
    ":": 0x0B,                 # Dreck
    ",": 0x2D,                 # Kies
    "X": 0x15,                 # Exit
    "O": 0x22,                 # Sockel
    "?": 0x2F,                 # Hinweis
    "*": 0x2A,                 # Bombe
    "V": 0x2B,                 # Falle
    "H": 0x21,                 # Dieb
    "@": 0x29,                 # Teleporter
    "u": 0x12, "d": 0x0D, "l": 0x14, "i": 0x13,   # Force N/S/W/E
    "m": 0x25, "n": 0x26,      # Toggle zu / offen
    "1": 0x16, "2": 0x17, "3": 0x18, "4": 0x19,   # Tueren blau/rot/gruen/gelb
    "5": 0x64, "6": 0x65, "7": 0x66, "8": 0x67,   # Schluessel
    "w": 0x68, "f": 0x69, "s": 0x6A, "o": 0x6B,   # Stiefel Wasser/Feuer/Eis/Force
    "g": 0x23, "r": 0x24, "b": 0x27, "c": 0x28,   # Knoepfe gruen/rot/braun/blau
    "M": 0x31,                 # Klon-Maschine
}
ENTITY = {
    "P": 0x6C,                 # Spieler-Start
    "B": 0x0A,                 # Block
    "a": 0x40, "T": 0x4C, "L": 0x50, "t": 0x54, "z": 0x58, "k": 0x5C, "p": 0x60,
    # bug a / tank T / glider L / teeth t / walker z / blob k / paramecium p
}

# harte Hindernisse fuers Flood-Fill (alles andere passierbar)
HARD = {0x01, 0x05, 0x2C, 0x30, 0x1F, 0x25, 0x31}   # Wand/Invis/Hidden/WallSE/BlueWall/ToggleZu/Cloner


def build_level(title, rows, hint="", time=0, traps=None, cloners=None,
                monsters=None, password=""):
    upper = [0x01] * (W * H)
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
                upper[idx] = 0x00
    _validate(title, upper)

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


def _reach(upper, socket_open):
    """Erreichbare Zellen von der Spielerzelle (harte Waende blockieren)."""
    p = next(i for i, t in enumerate(upper) if 0x6C <= t <= 0x6F)
    seen = {p}
    st = [p]
    while st:
        c = st.pop()
        x, y = c % W, c // W
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = x + dx, y + dy
            if 0 <= nx < W and 0 <= ny < H:
                n = ny * W + nx
                t = upper[n]
                blocked = t in HARD or (t == 0x22 and not socket_open)
                if n not in seen and not blocked:
                    seen.add(n)
                    st.append(n)
    return seen


def _validate(title, upper):
    closed = _reach(upper, socket_open=False)
    opened = _reach(upper, socket_open=True)
    chips = [i for i, t in enumerate(upper) if t == 0x02]
    keys = [i for i, t in enumerate(upper) if 0x64 <= t <= 0x67]
    exits = [i for i, t in enumerate(upper) if t in (0x15, 0x39, 0x3A, 0x3B)]
    sockets = [i for i, t in enumerate(upper) if t == 0x22]
    bad = [c for c in chips if c not in closed]
    assert not bad, f"{title}: {len(bad)} Chip(s) unerreichbar @ {[(c%W,c//W) for c in bad]}"
    badk = [k for k in keys if k not in closed]
    assert not badk, f"{title}: {len(badk)} Schluessel unerreichbar @ {[(k%W,k//W) for k in badk]}"
    assert exits, f"{title}: kein Ausgang"
    assert sockets, f"{title}: kein Sockel"
    # Sockel muss erreichbar sein
    assert any(s in closed or any((s + d) in closed for d in (1, -1, W, -W)) for s in sockets), \
        f"{title}: Sockel nicht erreichbar"
    # Ausgang: ohne Sockel NICHT, mit offenem Sockel schon erreichbar
    reach_closed = any(e in closed for e in exits)
    reach_open = any(e in opened for e in exits)
    assert not reach_closed, f"{title}: Ausgang ohne Sockel erreichbar (Chips waeren sinnlos)"
    assert reach_open, f"{title}: Ausgang auch mit offenem Sockel nicht erreichbar"


# ============================================================ LEVELS
# Ausgangs-Tasche (Sockel oben offen zum Raum, Exit darunter eingemauert):
#   row r-1 (...) x7='.'      Raum-Zugang
#   row r     #O#            Sockel
#   row r+1   #X#            Exit (nur ueber Sockel erreichbar)
def levels():
    out = []

    # --- 1: BOOT UP -- Tutorial -----------------------------------------
    out.append(build_level("BOOT UP", [
        "###############",
        "#P...........#",
        "#..+..+..+.+.#",
        "#............#",
        "#..+..+..+.+.#",
        "#............#",
        "#.....+......#",
        "#.....#O#....#",
        "#.....#X#....#",
        "###############",
    ], hint="Sammle alle Chips. Der Sockel oeffnet sich erst, wenn keiner mehr da ist -- dann fuehrt er zum Ausgang.",
        monsters=[]))

    # --- 2: LOCKED -- Schluessel & Tueren -------------------------------
    out.append(build_level("LOCKED", [
        "###############",
        "#P..5...6...+.#",
        "#.+.........+.#",
        "#####1###2#####",
        "#.+.......+.7.#",
        "#...........+#",
        "#.....+......#",
        "#.....#O#....#",
        "#.....#X#....#",
        "###############",
    ], hint="Schluessel oeffnen gleichfarbige Tueren. Hinter den Tueren liegen weitere Chips und der Ausgang.",
        monsters=[]))

    # --- 3: FLOODED -- Wasser + Bloecke + Flossen -----------------------
    out.append(build_level("FLOODED", [
        "###############",
        "#P..w......+..#",
        "#.+.........+.#",
        "#.~~~~~~~~~~~.#",
        "#.~~B~~B~~B~~.#",
        "#.~~~~~~~~~~~.#",
        "#.+.........+.#",
        "#.....+......#",
        "#.....#O#....#",
        "#.....#X#....#",
        "###############",
    ], hint="Floesse (Bloecke) ins Wasser schieben macht begehbaren Boden. Flossen schuetzen vorm Ertrinken.",
        monsters=[]))

    # --- 4: SLIPPERY -- Eis + Force-Boeden + Saugstiefel ----------------
    out.append(build_level("SLIPPERY", [
        "###############",
        "#P..o......+..#",
        "#.iiiiiiiii..#",
        "#.+.........+#",
        "#.________..#",
        "#...........+#",
        "#.dddddddddd.#",
        "#.+...+......#",
        "#.....#O#....#",
        "#.....#X#....#",
        "###############",
    ], hint="Eis rutscht bis zur Wand. Force-Boeden schieben in Pfeilrichtung -- Saugstiefel machen dich immun.",
        monsters=[]))

    # --- 5: INFESTED -- Monster + Feuer + Bomben ------------------------
    out.append(build_level("INFESTED", [
        "###############",
        "#P..f......+..#",
        "#.^^^^^^^^^^.#",
        "#.+.......+..#",
        "#..a....p...+#",
        "#.+.........+#",
        "#.*..+..+..*.#",
        "#.....+......#",
        "#.....#O#....#",
        "#.....#X#....#",
        "###############",
    ], hint="Feuer-Stiefel schuetzen vor Flammen. Beruehre keine Roboter und keine Bomben.",
        monsters=[(3, 4), (8, 4)]),)

    for i, lv in enumerate(out):
        lv["number"] = i + 1
    return out


def main():
    out = Path(__file__).resolve().parent / "levels" / "circuit_runner.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    data = {"name": "Circuit Runner", "ruleset": "ms", "levels": levels()}
    out.write_text(json.dumps(data, ensure_ascii=False, separators=(",", ":")),
                   encoding="utf-8")
    print(f"{len(data['levels'])} Demo-Level (alle validiert) -> {out}")


if __name__ == "__main__":
    main()
