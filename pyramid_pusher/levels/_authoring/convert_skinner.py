"""
Konvertiert die frei verteilbare Skinner-Sokoban-Sammlung (Titel-Kommentar
STEHT HINTER dem Brett) in das von Pyramid Pusher bevorzugte XSB-Format
(Titel-Kommentar VOR dem Brett) und validiert grob jedes Level.

Quelle: David W. Skinner (Microban u.a.), frei verteilbar bei Namensnennung.
        Roh-Datei: source/skinner_arranged.txt (von raw.githubusercontent.com /
        KevinWang15/js-responsive-sokoban, sourcecode.se-Spiegel).

Aufruf: .venv\\Scripts\\python.exe pyramid_pusher\\levels\\_authoring\\convert_skinner.py
"""
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "source", "skinner_arranged.txt")
OUT = os.path.join(HERE, "..", "02_skinner_classics.xsb")

BOARD = set("#$.@+* ")


def is_board(line):
    return any(c in "#$.@*+" for c in line)


def clean_title(c):
    t = c.lstrip(";").strip()
    t = re.sub(r"\s*R--?\d+\s*$", "", t)        # Schwierigkeits-Suffix weg
    return t


def valid(rows):
    boxes = goals = players = 0
    for r in rows:
        for c in r:
            if c in "$*":
                boxes += 1
            if c in ".+*":
                goals += 1
            if c in "@+":
                players += 1
    return players == 1 and boxes == goals and boxes >= 1


def main():
    with open(SRC, "r", encoding="latin-1") as f:
        lines = [ln.rstrip("\n").rstrip("\r") for ln in f]

    levels = []          # (title, [rows])
    cur, title, in_comments = [], None, False

    def flush():
        nonlocal cur, title, in_comments
        if cur and any("#" in r for r in cur):
            if valid(cur):
                levels.append((title, cur))
        cur, title, in_comments = [], None, False

    for ln in lines:
        if is_board(ln):
            if in_comments:                 # Kommentare beendeten das vorige Brett
                flush()
            cur.append(ln)
        elif ln.strip().startswith(";"):
            if cur:
                in_comments = True
                if title is None and "copyright" not in ln.lower():
                    title = clean_title(ln)
        else:                               # Leerzeile / sonstiges
            if cur and in_comments:
                flush()
    flush()

    out = []
    out.append("; ============================================")
    out.append("; Klassische Sokoban-Level  --  David W. Skinner")
    out.append("; (Microban u.a.), frei verteilbar bei Namensnennung.")
    out.append("; Quelle: sourcecode.se-Sammlung. Aus dem After-Comment-")
    out.append("; Format konvertiert (convert_skinner.py).")
    out.append("; ============================================")
    out.append("")

    # Nach Sammlung -> Set-Nummer -> Level-Nummer sortieren, damit zusammen-
    # haengende Kapitel-Bloecke entstehen (Microban I, II, ... Sasquatch I, ...).
    def roman(s):
        v = {"I": 1, "V": 5, "X": 10, "L": 50, "C": 100}
        return sum(v.get(c, 0) for c in s)        # Skinner nutzt additive Romans

    coll_order = {"Microban": 0, "Sasquatch": 1}

    def sortkey(item):
        t = item[0] or ""
        if " / " in t:
            left, right = t.split(" / ", 1)
            parts = left.rsplit(" ", 1)
            name = parts[0]
            rn = parts[1] if len(parts) > 1 else ""
            m = re.match(r"Level\s+(\d+)", right.strip())
            lvl = int(m.group(1)) if m else 9000      # benannte Bonuslevel ans Set-Ende
            return (coll_order.get(name, 50), roman(rn), lvl)
        return (99, 0, 0)

    levels.sort(key=sortkey)

    for i, (t, rows) in enumerate(levels):
        out.append("; " + (t if t else "Skinner / Level " + str(i + 1)))
        out.extend(rows)
        out.append("")

    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(out))

    print(f"{len(levels)} gueltige Level -> {os.path.normpath(OUT)}")
    # kleine Statistik
    from collections import Counter
    sets = Counter((t or "?").split(" / ")[0] for t, _ in levels)
    for name, n in sets.most_common():
        print(f"   {n:4d}  {name}")


if __name__ == "__main__":
    main()
