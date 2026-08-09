#!/usr/bin/env python
"""Laedt das Musik-Asset fuer examples/97_pbr_reactor.dh.

Die OGG (~3.4 MB) liegt bewusst NICHT im Git-Repo. Dieses Skript holt sie
einmalig von OpenGameArt:

    py examples/assets/download_techno.py

Titel:  "Technological Messup" von josepharaoh99
Quelle: https://opengameart.org/content/cc0-upbeat-electronic-music
Lizenz: CC0 1.0 (Public Domain) -- keine Namensnennung erforderlich.
"""
import sys
import urllib.request
from pathlib import Path

URL = "https://opengameart.org/sites/default/files/tecnological_messup_v2.ogg"
DEST = Path(__file__).resolve().parent / "techno_messup.ogg"


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
        print("Manuell von der Quelle laden und als techno_messup.ogg hier ablegen.")
        return 1
    if DEST.stat().st_size < 100_000:
        print("FEHLER: Datei zu klein -- vermutlich kein gueltiger Download.")
        return 1
    print(f"OK ({DEST.stat().st_size // 1024} KB). Musik: josepharaoh99, CC0.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
