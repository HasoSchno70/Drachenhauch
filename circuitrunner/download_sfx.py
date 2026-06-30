"""Laedt die Soundeffekte fuer CIRCUIT RUNNER herunter.

Quelle: Kenney "Interface Sounds" (https://kenney.nl/assets/interface-sounds),
Lizenz **CC0 1.0** (Public Domain -- keine Attribution noetig, frei nutz- und
weiterverteilbar). Bezogen ueber den jsDelivr-CDN-Spiegel des GitHub-Repos
`Calinou/kenney-interface-sounds`.

Die WAVs sind klein und CC0, daher direkt mit eingecheckt -- dieses Skript
dient der Reproduzierbarkeit / zum Aktualisieren. Aufruf:

    .venv\\Scripts\\python.exe circuitrunner\\download_sfx.py
"""
from __future__ import annotations

import sys
import urllib.request
from pathlib import Path

BASE = ("https://cdn.jsdelivr.net/gh/Calinou/kenney-interface-sounds@master/"
        "addons/kenney_interface_sounds/")

# Spiel-Event -> Kenney-Quelldatei (CC0)
SOUNDS = {
    "step":    "tick_001.wav",          # Schritt (kurzer Tick)
    "chip":    "confirmation_001.wav",  # Daten-Chip eingesammelt
    "door":    "toggle_001.wav",        # Tuer/Schluessel/Toggle
    "block":   "drop_001.wav",          # Block geschoben (dumpfer Stoss)
    "die":     "glitch_002.wav",        # Spieler stirbt (Stoerung)
    "win":     "maximize_009.wav",      # Level geschafft (aufsteigend)
    "blocked": "back_002.wav",          # blockiert (Wand-Stoss)
}

OUT = Path(__file__).resolve().parent / "assets" / "sfx"


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    ok = 0
    for name, src in SOUNDS.items():
        dst = OUT / f"{name}.wav"
        try:
            with urllib.request.urlopen(BASE + src, timeout=30) as r:
                data = r.read()
            dst.write_bytes(data)
            print(f"  {name:8} <- {src:22} ({len(data)} B)")
            ok += 1
        except Exception as e:  # noqa: BLE001
            print(f"  FEHLER {name}: {e}")
    # Lizenz-/Quellnachweis ablegen
    (OUT / "CREDITS.txt").write_text(
        "Soundeffekte: Kenney 'Interface Sounds' -- https://kenney.nl/assets/interface-sounds\n"
        "Lizenz: CC0 1.0 (Public Domain). Keine Attribution noetig.\n"
        "Spiegel: https://github.com/Calinou/kenney-interface-sounds\n\n"
        + "".join(f"{n}.wav  <-  {s}\n" for n, s in SOUNDS.items()),
        encoding="utf-8")
    print(f"{ok}/{len(SOUNDS)} Sounds in {OUT}")
    return 0 if ok == len(SOUNDS) else 1


if __name__ == "__main__":
    sys.exit(main())
