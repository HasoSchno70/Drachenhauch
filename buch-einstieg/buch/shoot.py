#!/usr/bin/env python
# -*- coding: utf-8 -*-
r"""Bilder fuers Einstiegsbuch aufnehmen.

Rendert jede figures/*.dh ueber dhrt nach images/<name>.png.

Warum figures/ neben code/ steht: der Screenshot faellt beim LETZTEN Frame.
Die Kapitelprogramme der ersten Teile zeichnen aber nur EINMAL und warten
dann mit SLEEP -- ein einzelner FLIP kommt zu frueh, das Bild wird schwarz.
Die Figurenquelle enthaelt deshalb dieselben Zeichenbefehle in einer kurzen
Schleife. Sie ist keine zweite Wahrheit: pruef_codebloecke.js prueft den
Abdruck im Buch, und beides laeuft durch `dhrt --check`.

SCALE=3 bei 640x400 -> 1920x1200. Gemessen: mehr geht nicht. Mit SCALE=4
waere das Fenster 2560x1600 und damit hoeher als der Bildschirm; Windows
schiebt es dann nach unten weg, und die Aufnahme bekam oben 179 schwarze
Zeilen. 1920 Breite ist zugleich das Mass des Referenzbuchs und reicht fuer
den 300-dpi-Druck.

Aufruf:  <venv>\python.exe shoot.py [nur_diese_basename ...]
"""
import os
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
FIG = os.path.join(HIER, "figures")
IMG = os.path.join(HIER, "images")
DHRT = os.path.join(HIER, "..", "..", "rust", "drachenhauch_runtime",
                    "target", "release", "dhrt.exe")
FONT = os.environ.get("BUCH_FONT", r"C:\Windows\Fonts\segoeui.ttf")
SCALE = "3"
FRAMES_VORGABE = 12

# Mehr Frames, wo sich etwas entwickeln muss (Animation, Physik, Partikel).
FRAMES = {}


def aufnehmen(name):
    quelle = os.path.join(FIG, name + ".dh")
    if not os.path.exists(quelle):
        print("  (keine Quelle)", name)
        return False
    ziel = os.path.join(IMG, name + ".png")
    frames = FRAMES.get(name, FRAMES_VORGABE)
    umgebung = dict(os.environ)
    umgebung.update({"DHRT_FRAMES": str(frames), "DHRT_SCALE": SCALE,
                     "DHRT_SCREENSHOT": ziel})
    if os.path.exists(FONT):
        umgebung["DHRT_FONT"] = FONT
    r = subprocess.run([DHRT, "run", quelle], env=umgebung,
                       capture_output=True, text=True, encoding="utf-8")
    if r.returncode != 0 or not os.path.exists(ziel):
        print("  FEHLER", name, (r.stderr or "").strip()[:200])
        return False
    print(f"  {name}.png  ({frames} Frames)")
    return True


def main(argv):
    os.makedirs(IMG, exist_ok=True)
    namen = argv or sorted(f[:-3] for f in os.listdir(FIG) if f.endswith(".dh"))
    gut = sum(1 for n in namen if aufnehmen(n))
    print(f"{gut}/{len(namen)} Bilder")
    return 0 if gut == len(namen) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
