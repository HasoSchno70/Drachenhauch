"""
Erzeugt + VALIDIERT den gebuendelten Original-Level-Satz fuer Pyramid Pusher.

Alle Level hier sind selbst entworfen (frei verwendbar). Ein kleiner
Sokoban-Solver (BFS ueber Push-Zuege) prueft JEDEN Level auf Loesbarkeit, bevor
er geschrieben wird -- so landet garantiert kein unloesbarer Level im Spiel.

Format = Standard-XSB:  # Wand   (Leer) Boden   @ Spieler   $ Kiste
                        . Ziel   * Kiste-auf-Ziel   + Spieler-auf-Ziel
Level durch Leerzeile getrennt; ';' beginnt eine Titel-/Kommentarzeile.

Aufruf:  .venv\\Scripts\\python.exe pyramid_pusher\\levels\\_authoring\\make_levels.py
"""
import os
from collections import deque

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "01_pyramid_pusher.xsb")

# (Titel, Level-Zeilen) -- bewusste Schwierigkeitskurve.
LEVELS = [
    ("Die erste Kammer", [
        "#######",
        "#     #",
        "# @$. #",
        "#     #",
        "#######",
    ]),
    ("Nach oben", [
        "#####",
        "#.  #",
        "#$  #",
        "#@  #",
        "#####",
    ]),
    ("Zwei Saeulen", [
        "######",
        "#.  .#",
        "#$  $#",
        "#  @ #",
        "######",
    ]),
    ("Die Ecke", [
        "#######",
        "#@    #",
        "# $## #",
        "# . # #",
        "#   # #",
        "#######",
    ]),
    ("Schiebereih", [
        "########",
        "#@ $ . #",
        "########",
        "",
    ]),
    ("Doppelpack", [
        "#######",
        "#  .  #",
        "# $$@ #",
        "#  .  #",
        "#######",
    ]),
    ("Der Korridor", [
        "#########",
        "#. $ @  #",
        "#########",
    ]),
    ("Vier Ziele", [
        "#######",
        "#.   .#",
        "# $ $ #",
        "#  @  #",
        "# $ $ #",
        "#.   .#",
        "#######",
    ]),
    ("Um die Wand", [
        "#######",
        "#  @  #",
        "# # # #",
        "#.$ $.#",
        "#######",
    ]),
    ("Das Lager", [
        "########",
        "#  #   #",
        "# $$ . #",
        "# @ #. #",
        "#  ##  #",
        "########",
    ]),
    ("Verschachtelt", [
        "#######",
        "#.    #",
        "#.$$ @#",
        "#.$   #",
        "#  #  #",
        "#######",
    ]),
    ("Die Grabkammer", [
        "########",
        "#   . ##",
        "# $$$ @#",
        "## . . #",
        " # ### #",
        " #     #",
        " #######",
    ]),
]


def parse(lines):
    walls, goals, boxes = set(), set(), set()
    player = None
    for y, row in enumerate(lines):
        for x, c in enumerate(row):
            p = (x, y)
            if c == '#':
                walls.add(p)
            elif c in '.+*':
                goals.add(p)
            if c in '$*':
                boxes.add(p)
            if c in '@+':
                player = p
    return walls, goals, frozenset(boxes), player


def solvable(lines, max_states=400000):
    walls, goals, boxes0, player0 = parse(lines)
    if player0 is None:
        return False, "kein Spieler"
    if len(boxes0) != len(goals):
        return False, f"#Kisten({len(boxes0)}) != #Ziele({len(goals)})"
    if not boxes0:
        return False, "keine Kiste"
    if boxes0 == goals:
        return False, "schon geloest"

    def reachable(player, boxes):
        # alle vom Spieler erreichbaren Felder (Normalisierung)
        seen = {player}
        dq = deque([player])
        while dq:
            x, y = dq.popleft()
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                n = (x + dx, y + dy)
                if n in seen or n in walls or n in boxes:
                    continue
                seen.add(n); dq.append(n)
        return seen

    def norm(player, boxes):
        return min(reachable(player, boxes))

    def is_dead(box, boxes):
        # einfache Eck-Deadlock-Erkennung (Kiste nicht auf Ziel, in Ecke)
        if box in goals:
            return False
        x, y = box
        up = (x, y - 1) in walls; dn = (x, y + 1) in walls
        lf = (x - 1, y) in walls; rt = (x + 1, y) in walls
        return (up and lf) or (up and rt) or (dn and lf) or (dn and rt)

    start = (norm(player0, boxes0), boxes0)
    seen = {start}
    dq = deque([(player0, boxes0)])
    n = 0
    while dq:
        player, boxes = dq.popleft()
        n += 1
        if n > max_states:
            return False, "Zustandslimit (zu komplex fuer Quick-Check)"
        if boxes == goals:
            return True, "ok"
        rset = reachable(player, boxes)
        for box in boxes:
            bx, by = box
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                # Spieler muss auf der Gegenseite stehen koennen
                pside = (bx - dx, by - dy)
                target = (bx + dx, by + dy)
                if pside not in rset:
                    continue
                if target in walls or target in boxes:
                    continue
                nb = set(boxes); nb.discard(box); nb.add(target)
                nbf = frozenset(nb)
                if is_dead(target, nbf):
                    continue
                key = (norm(box, nbf), nbf)
                if key in seen:
                    continue
                seen.add(key)
                dq.append((box, nbf))
    return False, "unloesbar (durchsucht)"


def main():
    out_lines = []
    out_lines.append("; ============================================")
    out_lines.append("; Pyramid Pusher -- Kampagne 'Sandstein'")
    out_lines.append("; Original-Level (frei verwendbar).")
    out_lines.append("; Eigene Saetze: weitere .xsb in diesen Ordner legen.")
    out_lines.append("; ============================================")
    out_lines.append("")
    all_ok = True
    for i, (title, lines) in enumerate(LEVELS, 1):
        body = [ln for ln in lines if ln != ""]
        ok, msg = solvable(body)
        status = "OK " if ok else "FAIL"
        print(f"  [{status}] {i:2d}. {title:<18} {msg}")
        if not ok:
            all_ok = False
        out_lines.append(f"; {i}. {title}")
        out_lines.extend(body)
        out_lines.append("")
    if not all_ok:
        print("\n!! Mindestens ein Level ist nicht loesbar -- NICHT geschrieben.")
        raise SystemExit(1)
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(out_lines))
    print(f"\nGeschrieben: {os.path.normpath(OUT)}  ({len(LEVELS)} Level, alle loesbar)")


if __name__ == "__main__":
    main()
