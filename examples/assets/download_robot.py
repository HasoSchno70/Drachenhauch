#!/usr/bin/env python
"""Laedt ein geriggtes, animiertes 3D-Modell fuer die Skelett-Animations-Demo
(examples/108_skeletal_anim.dh).

Die .glb (~mehrere MB) liegt bewusst NICHT im Git-Repo. Dieses Skript holt sie
einmalig:

    py examples/assets/download_robot.py

Asset:  "robot.glb" -- ein geriggter Roboter mit mehreren Animationen
        (idle, walk, dance, ...), Teil der offiziellen raylib-Beispiel-Ressourcen.
Quelle: raysan5/raylib (examples/models/resources/models/gltf/robot.glb)
Lizenz: CC0 1.0 (Public Domain), (c) raysan5 -- keine Namensnennung erforderlich.

Falls die URL nicht mehr stimmt: ein beliebiges geriggtes .glb (mit Animationen)
nach examples/assets/robot.glb legen.
"""
import urllib.request
from pathlib import Path

URL = ("https://raw.githubusercontent.com/raysan5/raylib/master/"
       "examples/models/resources/models/gltf/robot.glb")
DEST = Path(__file__).resolve().parent / "robot.glb"


def main() -> int:
    if DEST.exists() and DEST.stat().st_size > 100_000:
        print(f"Schon vorhanden: {DEST} ({DEST.stat().st_size // 1024} KB)")
        return 0
    print(f"Lade {URL}\n  -> {DEST} ...")
    try:
        req = urllib.request.Request(URL, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=120) as r, open(DEST, "wb") as f:
            f.write(r.read())
    except Exception as exc:  # pragma: no cover
        print(f"FEHLER beim Download: {exc}")
        print("Manuell ein geriggtes .glb (mit Animationen) als "
              "examples/assets/robot.glb ablegen.")
        return 1
    if DEST.stat().st_size < 100_000:
        print("FEHLER: Datei zu klein -- vermutlich kein gueltiger Download.")
        return 1
    print(f"OK ({DEST.stat().st_size // 1024} KB). Modell: raysan5/raylib, CC0.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
