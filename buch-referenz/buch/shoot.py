#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Screenshots fuer das Lehrbuch neu aufnehmen -- scharf statt pixelig.

Rendert jede figures/NN_*.gb ueber gbrt mit:
  GBRT_SCALE=4   -> 4x native Aufloesung (1920x1280; scharfe Formen/Sprites,
                    erfuellt die 300-dpi-Druckpruefung ohne Hoch-Skalieren)
  GBRT_FONT=...  -> echte TTF als Default-Font (scharfe, anti-aliaste Schrift
                    statt der pixeligen raylib-Bitmap-Schrift; Umlaute inkl.)

Pro Demo eine Frame-Zahl (Animationen/Physik brauchen einen entwickelten Stand).
Ergebnis -> images/NN_*.png. make_book.py/prepare_images() macht sie dann nur
noch alpha-flach (kein Hochskalieren noetig, da bereits 1920 breit).

Aufruf:  <venv>\\python.exe shoot.py [nur_diese_basename ...]
Optional Font ueber Umgebungsvariable BUCH_FONT (Default: Segoe UI).
"""
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FIG = os.path.join(HERE, "figures")
IMG = os.path.join(HERE, "images")
GBRT = os.path.join(HERE, "..", "..", "rust", "drachenhauch_runtime", "target", "release", "gbrt.exe")
FONT = os.environ.get("BUCH_FONT", r"C:\Windows\Fonts\segoeui.ttf")
SCALE = "4"

# Frames je Demo (Default 60). Genug, damit Animation/Physik einen schoenen
# Stand erreicht.
FRAMES = {
    "40_fenster": 20, "41_formen": 10, "42_blend": 10, "42_extras": 10,
    "43_bilder": 10, "44_farben": 10, "45_eingabe": 10, "46_sound": 10,
    "47_layer": 10, "48_3d": 20, "49_catch": 40, "50_sprite": 12,
    "51_animfsm": 30, "52_tween": 45, "54_particles": 50, "55_physics2d": 90,
    "56_camera": 40, "58_ui": 10, "59_gui": 10, "59a_chart": 10, "62_astar": 10,
    "63_tiled": 10, "64_tilecollide": 70, "65_controller": 45, "66_vec2": 10,
    "67_m3d": 20, "72_curves": 10,
}


def shoot(base):
    src = os.path.join(FIG, base + ".gb")
    if not os.path.exists(src):
        print("  (keine Quelle)", base); return False
    out = os.path.join(IMG, base + ".png")
    env = dict(os.environ)
    env["GBRT_SCALE"] = SCALE
    env["GBRT_FONT"] = FONT
    env["GBRT_FRAMES"] = str(FRAMES.get(base, 60))
    env["GBRT_SCREENSHOT"] = out
    r = subprocess.run([GBRT, "run", src], cwd=FIG, env=env,
                       capture_output=True, text=True, timeout=120)
    ok = os.path.exists(out) and r.returncode == 0
    print(("  OK  " if ok else "  FEHLER ") + base + (("  " + r.stderr.strip()[-120:]) if not ok else ""))
    return ok


def main():
    only = set(sys.argv[1:])
    bases = sorted(b[:-3] for b in os.listdir(FIG) if b.endswith(".gb"))
    if only:
        bases = [b for b in bases if b in only]
    n = sum(shoot(b) for b in bases)
    print(f"{n}/{len(bases)} Screenshots neu aufgenommen (scale {SCALE}, TTF).")


if __name__ == "__main__":
    main()
