#!/usr/bin/env python
# -*- coding: utf-8 -*-
r"""Erzeugt die Sprites, die die Programme aus Teil IV benutzen.

Warum ein Skript und nicht einfach ein paar PNG-Dateien: Damit sie
reproduzierbar sind. Wer eine Figur aendern will, aendert hier zwei Zeichen
und laesst das Skript neu laufen -- statt in einem Bildprogramm zu suchen,
wie die Farbe von damals hiess.

Die Pixel stehen als Buchstabenraster da, eine Zeile je Bildzeile. Ein Punkt
ist durchsichtig. Die Zuordnung Buchstabe -> Farbe steht in PALETTE.

Gezeichnet wird 16x16 und mit Faktor 2 auf 32x32 vergroessert -- mit harten
Kanten (NEAREST), damit die Pixel Pixel bleiben. Das Ergebnis sieht im
640x400-Fenster nach Pixelgrafik aus und ist gross genug, um es zu treffen.

Das Buch bringt seinen Lesern in Kapitel 20 bei, Sprites im mitgelieferten
Editor `dhsprites` selbst zu malen. Dieses Skript ist nur der Weg, wie die
Beispielfiguren des Buchs selbst entstanden sind.

Aufruf:  <venv>\python.exe mach_sprites.py
"""
import os
import sys

from PIL import Image

HIER = os.path.dirname(os.path.abspath(__file__))
FAKTOR = 2

# Wohin die fertigen Sprites gelegt werden. Neben assets/ auch in jedes
# Kapitelverzeichnis, das sie braucht -- denn ein Dateiname ohne Pfad zeigt
# auf das Verzeichnis des PROGRAMMS (siehe Kapitel 18). Im Buch soll schlicht
# LOADIMAGE("schiff.png") stehen und kein Pfad-Gestruepp.
ZIELE = [
    HIER,
    os.path.join(HIER, "..", "code", "kap19"),
    os.path.join(HIER, "..", "code", "kap21"),
    os.path.join(HIER, "..", "code", "kap22"),
    os.path.join(HIER, "..", "code", "kap23"),
]

PALETTE = {
    ".": (0, 0, 0, 0),          # durchsichtig
    "W": (235, 245, 255, 255),  # helles Weiss -- Kanten
    "C": (95, 190, 240, 255),   # Cyan -- Rumpf
    "B": (40, 90, 160, 255),    # dunkles Blau -- Schatten
    "R": (255, 140, 60, 255),   # Orange -- Antrieb
    "G": (120, 210, 120, 255),  # Gruen -- Gegner
    "D": (50, 130, 60, 255),    # dunkles Gruen
    "A": (30, 30, 40, 255),     # fast Schwarz -- Augen
    "Y": (255, 215, 80, 255),   # Gold -- Muenze
    "O": (200, 150, 30, 255),   # dunkles Gold
}

SCHIFF = [
    "................",
    ".......WW.......",
    "......WCCW......",
    "......WCCW......",
    ".....WCCCCW.....",
    ".....WCCCCW.....",
    "....WCCCCCCW....",
    "....WCBBBBCW....",
    "...WCCBBBBCCW...",
    "...WCCCCCCCCW...",
    "..WCCCCCCCCCCW..",
    "..WCCCCCCCCCCW..",
    ".WCCW.WCCW.WCCW.",
    ".WCCW.WCCW.WCCW.",
    "..RR...RR...RR..",
    "...R....R....R..",
]

GEGNER_A = [
    "................",
    "..G..........G..",
    "...G........G...",
    "..GGGGGGGGGGGG..",
    ".GGG.GGGGGG.GGG.",
    "GGGGGGGGGGGGGGGG",
    "GAAGGGGGGGGGGAAG",
    "GAAGGGGGGGGGGAAG",
    "GGGGGGGGGGGGGGGG",
    "GGG.DDDDDDDD.GGG",
    "GGG.D......D.GGG",
    "GG..D......D..GG",
    ".G..DD....DD..G.",
    "....D......D....",
    "...DD......DD...",
    "................",
]

GEGNER_B = [
    "................",
    "..G..........G..",
    "..GG........GG..",
    "..GGGGGGGGGGGG..",
    ".GGG.GGGGGG.GGG.",
    "GGGGGGGGGGGGGGGG",
    "GAAGGGGGGGGGGAAG",
    "GAAGGGGGGGGGGAAG",
    "GGGGGGGGGGGGGGGG",
    ".GG.DDDDDDDD.GG.",
    "..G.D......D.G..",
    "....D......D....",
    "...DD......DD...",
    "...D........D...",
    "..DD........DD..",
    "................",
]

# Vier Stufen einer sich drehenden Muenze: von voll bis zur Kante.
MUENZE = [
    [
        "................",
        "................",
        "....YYYYYYYY....",
        "...YYOOOOOOYY...",
        "..YYOOOOOOOOYY..",
        "..YOOOYYYYOOOY..",
        "..YOOYYOOYYOOY..",
        "..YOOYOOOOYOOY..",
        "..YOOYOOOOYOOY..",
        "..YOOYYOOYYOOY..",
        "..YOOOYYYYOOOY..",
        "..YYOOOOOOOOYY..",
        "...YYOOOOOOYY...",
        "....YYYYYYYY....",
        "................",
        "................",
    ],
    [
        "................",
        "................",
        ".....YYYYYY.....",
        "....YYOOOOYY....",
        "....YOOOOOOY....",
        "....YOOYYOOY....",
        "....YOYOOYOY....",
        "....YOYOOYOY....",
        "....YOYOOYOY....",
        "....YOYOOYOY....",
        "....YOOYYOOY....",
        "....YOOOOOOY....",
        "....YYOOOOYY....",
        ".....YYYYYY.....",
        "................",
        "................",
    ],
    [
        "................",
        "................",
        "......YYYY......",
        "......YOOY......",
        "......YOOY......",
        "......YOOY......",
        "......YOOY......",
        "......YOOY......",
        "......YOOY......",
        "......YOOY......",
        "......YOOY......",
        "......YOOY......",
        "......YOOY......",
        "......YYYY......",
        "................",
        "................",
    ],
    [
        "................",
        "................",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        "................",
        "................",
    ],
]


def pruefe(name, raster):
    """Ein Vertipper im Pixelraster faellt sonst erst im Bild auf."""
    if len(raster) != 16:
        raise SystemExit(f"{name}: {len(raster)} Zeilen, erwartet 16")
    for n, zeile in enumerate(raster):
        if len(zeile) != 16:
            raise SystemExit(f"{name}, Zeile {n}: {len(zeile)} Zeichen, erwartet 16")
        for c in zeile:
            if c not in PALETTE:
                raise SystemExit(f"{name}, Zeile {n}: unbekanntes Zeichen {c!r}")


def bild_aus(raster):
    im = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    px = im.load()
    for y, zeile in enumerate(raster):
        for x, c in enumerate(zeile):
            px[x, y] = PALETTE[c]
    return im.resize((16 * FAKTOR, 16 * FAKTOR), Image.NEAREST)


def streifen(raster_liste):
    """Mehrere Bilder nebeneinander -- ein Streifen, aus dem DRAWIMAGEPART
    einzelne Felder schneidet. So braucht ein Programm eine Datei statt
    vier."""
    teile = [bild_aus(r) for r in raster_liste]
    b, h = teile[0].size
    aus = Image.new("RGBA", (b * len(teile), h), (0, 0, 0, 0))
    for i, t in enumerate(teile):
        aus.paste(t, (i * b, 0))
    return aus


def raster_bild(raster, zelle=24):
    """Das Pixelraster einer Figur als Lehrbild: jedes Pixel ein Kaestchen,
    dazwischen ein Gitter. So sieht man beim Lesen, dass Pixelgrafik nichts
    weiter ist als ein Karopapier, auf dem einzelne Felder Farbe bekommen."""
    n = len(raster)
    hg = (20, 26, 44)
    linie = (55, 68, 100)
    im = Image.new("RGB", (n * zelle + 1, n * zelle + 1), hg)
    px = im.load()
    for y, zeile in enumerate(raster):
        for x, c in enumerate(zeile):
            f = PALETTE[c]
            if f[3] == 0:
                continue
            for dy in range(1, zelle):
                for dx in range(1, zelle):
                    px[x * zelle + dx, y * zelle + dy] = f[:3]
    for i in range(n + 1):
        for k in range(n * zelle + 1):
            px[i * zelle, k] = linie
            px[k, i * zelle] = linie
    return im


def nebeneinander(bilder, luecke=30, hg=(20, 26, 44)):
    breite = sum(b.width for b in bilder) + luecke * (len(bilder) - 1)
    aus = Image.new("RGB", (breite, max(b.height for b in bilder)), hg)
    x = 0
    for b in bilder:
        aus.paste(b, (x, 0))
        x += b.width + luecke
    return aus


def main():
    pruefe("SCHIFF", SCHIFF)
    pruefe("GEGNER_A", GEGNER_A)
    pruefe("GEGNER_B", GEGNER_B)
    for i, r in enumerate(MUENZE):
        pruefe(f"MUENZE[{i}]", r)

    fertig = {
        "schiff.png": bild_aus(SCHIFF),
        "gegner.png": streifen([GEGNER_A, GEGNER_B]),
        "muenze.png": streifen(MUENZE),
    }
    for ziel in ZIELE:
        if not os.path.isdir(ziel):
            continue
        for name, bild in fertig.items():
            bild.save(os.path.join(ziel, name))
        print(f"  -> {os.path.relpath(ziel, HIER)}")
    for name, bild in fertig.items():
        print(f"  {name:<12} {bild.width}x{bild.height}")

    # Lehrbilder fuers Buch -- direkt nach buch/images/, wo shoot.py auch
    # hinschreibt. Sie zeigen die Figuren als Karopapier statt als Sprite.
    bilder = os.path.join(HIER, "..", "buch", "images")
    if os.path.isdir(bilder):
        raster_bild(SCHIFF).save(os.path.join(bilder, "kap20_raster_schiff.png"))
        print("  kap20_raster_schiff.png")
        nebeneinander([raster_bild(GEGNER_A, 18), raster_bild(GEGNER_B, 18)]).save(
            os.path.join(bilder, "kap21_zwei_frames.png"))
        print("  kap21_zwei_frames.png")
    return 0


if __name__ == "__main__":
    sys.exit(main())
