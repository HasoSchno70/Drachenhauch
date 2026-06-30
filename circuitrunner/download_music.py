"""Laedt die Hintergrundmusik fuer CIRCUIT RUNNER herunter.

Quelle: Juhani Junkala, "5 action chiptunes" / Retro Game Music Pack,
Lizenz **CC0 1.0** (Public Domain -- keine Attribution noetig). Bezogen vom
Internet Archive (kanonische CC0-Veroeffentlichung), als nahtlos loopende OGGs.

Die OGGs sind ~1 MB und CC0, daher direkt mit eingecheckt -- dieses Skript
dient der Reproduzierbarkeit. Aufruf:

    .venv\\Scripts\\python.exe circuitrunner\\download_music.py
"""
from __future__ import annotations

import sys
import urllib.parse
import urllib.request
from pathlib import Path

ITEM = "JuhaniJunkalafiveactionchiptunes"
BASE = f"https://archive.org/download/{ITEM}/"

# Ziel-Name -> Quelldatei (CC0)
TRACKS = {
    "title": "Juhani Junkala [Retro Game Music Pack] Title Screen.ogg",  # Menue
    "level": "Juhani Junkala [Retro Game Music Pack] Level 1.ogg",       # Spiel
}

OUT = Path(__file__).resolve().parent / "assets" / "music"


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    ok = 0
    for name, src in TRACKS.items():
        dst = OUT / f"{name}.ogg"
        url = BASE + urllib.parse.quote(src)
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "circuitrunner-dl"})
            with urllib.request.urlopen(req, timeout=60) as r:
                data = r.read()
            dst.write_bytes(data)
            print(f"  {name:6} <- {src}  ({len(data) // 1024} KB)")
            ok += 1
        except Exception as e:  # noqa: BLE001
            print(f"  FEHLER {name}: {e}")
    (OUT / "CREDITS.txt").write_text(
        "Hintergrundmusik: Juhani Junkala -- '5 action chiptunes' / Retro Game Music Pack\n"
        "Lizenz: CC0 1.0 (Public Domain). Keine Attribution noetig.\n"
        "Quelle: https://archive.org/details/" + ITEM + "\n"
        "Autor:  https://juhanijunkala.com\n\n"
        + "".join(f"{n}.ogg  <-  {s}\n" for n, s in TRACKS.items()),
        encoding="utf-8")
    print(f"{ok}/{len(TRACKS)} Tracks in {OUT}")
    return 0 if ok == len(TRACKS) else 1


if __name__ == "__main__":
    sys.exit(main())
