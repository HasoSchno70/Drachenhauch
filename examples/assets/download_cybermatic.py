#!/usr/bin/env python
"""Laedt das Musik-Asset fuer examples/85_cybermatic_demo.dh.

Die OGG (~15 MB) liegt bewusst NICHT im Git-Repo. Dieses Skript holt sie
einmalig von OpenGameArt:

    py examples/assets/download_cybermatic.py

Titel:  "Cybermatic pulse" (LOOP) von Alexandr Zhelanov
Quelle: https://opengameart.org/content/cybermatic-pulse
Lizenz: CC-BY 4.0  -- Namensnennung erforderlich (siehe CREDITS_cybermatic.txt).
"""
import sys
import urllib.request
from pathlib import Path

URL = "https://opengameart.org/sites/default/files/Cybermatic%20pulse%20%28LOOP%29.ogg"
DEST = Path(__file__).resolve().parent / "cybermatic_pulse.ogg"


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
        print("Manuell von der Quelle laden und als cybermatic_pulse.ogg hier ablegen.")
        return 1
    if DEST.stat().st_size < 100_000:
        print("FEHLER: Datei zu klein -- vermutlich kein gueltiger Download.")
        return 1
    print(f"OK ({DEST.stat().st_size // 1024} KB). Musik: Alexandr Zhelanov, CC-BY 4.0.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
