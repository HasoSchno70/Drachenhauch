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
import re
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
FRAMES = {
    "kap05_1_wanderer": 60, "kap05_2_fallender_ball": 46,
    "kap05_3_spur": 38, "kap05_4_regen": 40, "kap05_5_monde": 70,
}


def quelle_finden(name):
    """Erst figures/, dann das Kapitelprogramm selbst.

    Ab Kapitel 5 haben die Programme eine eigene Schleife und laufen so
    lange, bis man sie beendet -- die lassen sich unveraendert aufnehmen,
    und eine zweite Quelle waere nur eine zweite Wahrheit. Nur die
    linearen Programme der ersten Kapitel brauchen die Schleifenfassung
    unter figures/, weil ein einzelner FLIP fuer die Aufnahme zu frueh
    kommt. Aus `kap05_1_wanderer` wird `../code/kap05/1_wanderer.dh`.
    """
    eigen = os.path.join(FIG, name + ".dh")
    if os.path.exists(eigen):
        return eigen
    m = re.match(r"(kap\d+)_(.+)$", name)
    if m:
        p = os.path.join(HIER, "..", "code", m.group(1), m.group(2) + ".dh")
        if os.path.exists(p):
            return p
    return None


def aufnehmen(name):
    quelle = quelle_finden(name)
    if quelle is None:
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
    if argv:
        namen = argv
    else:
        namen = {f[:-3] for f in os.listdir(FIG) if f.endswith(".dh")}
        code = os.path.join(HIER, "..", "code")
        for kap in sorted(os.listdir(code)):
            for f in sorted(os.listdir(os.path.join(code, kap))):
                if f.endswith(".dh") and f"{kap}_{f[:-3]}" not in namen:
                    # Nur Programme mit eigener Schleife: die linearen haben
                    # ihre Schleifenfassung schon unter figures/.
                    text = open(os.path.join(code, kap, f), encoding="utf-8").read()
                    if "WHILE" in text.upper():
                        namen.add(f"{kap}_{f[:-3]}")
        namen = sorted(namen)
    gut = sum(1 for n in namen if aufnehmen(n))
    print(f"{gut}/{len(namen)} Bilder")
    return 0 if gut == len(namen) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
