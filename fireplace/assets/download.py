#!/usr/bin/env python
"""Laedt die freien Assets fuer das Kamin-Feuer-Demo (examples-Stil):

    py fireplace/assets/download.py

Holt einmalig (liegen NICHT im Git-Repo):
  * fireplace.jpg -- Steinkamin-Innenansicht (Schloss Chillon)
      Quelle: Wikimedia Commons, Autor "CEllen", Lizenz CC BY-SA 4.0
      https://commons.wikimedia.org/wiki/File:Chillon_Castle_interior_view_with_the_fireplace.jpg
  * fire.ogg -- knisterndes Lagerfeuer (Loop-tauglich, ~1 min)
      Quelle: Wikimedia Commons / Freesound, Autor "Glaneur de sons", Lizenz CC BY 3.0
      https://commons.wikimedia.org/wiki/File:Campfire_sound_ambience.ogg

Beide Lizenzen verlangen Namensnennung -> siehe CREDITS.txt. Das Demo laeuft
auch OHNE die Dateien (prozeduraler Ersatz-Kamin, stummes Feuer).
"""
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
UA = {"User-Agent": "Mozilla/5.0 (Drachenhauch fireplace demo asset fetch)"}

ASSETS = [
    ("https://upload.wikimedia.org/wikipedia/commons/thumb/8/88/"
     "Chillon_Castle_interior_view_with_the_fireplace.jpg/"
     "1280px-Chillon_Castle_interior_view_with_the_fireplace.jpg",
     HERE / "fireplace.jpg", 50_000),
    ("https://upload.wikimedia.org/wikipedia/commons/b/b1/Campfire_sound_ambience.ogg",
     HERE / "fire.ogg", 50_000),
    # Wind (assets/wind.mp3) ist optional + selbst beizulegen (z.B. Pixabay).
    # Fehlt die Datei, laeuft das Demo einfach ohne Wind.
]


def fetch(url: str, dest: Path, min_size: int) -> bool:
    if dest.exists() and dest.stat().st_size > min_size:
        print(f"Schon vorhanden: {dest.name} ({dest.stat().st_size // 1024} KB)")
        return True
    print(f"Lade {dest.name} ...")
    try:
        req = urllib.request.Request(url, headers=UA)
        with urllib.request.urlopen(req, timeout=120) as r, open(dest, "wb") as f:
            f.write(r.read())
    except Exception as exc:  # pragma: no cover
        print(f"  FEHLER: {exc}")
        return False
    ok = dest.stat().st_size > min_size
    print(f"  {'OK' if ok else 'ZU KLEIN'} ({dest.stat().st_size // 1024} KB)")
    return ok


def main() -> int:
    all_ok = True
    for url, dest, ms in ASSETS:
        if not fetch(url, dest, ms):
            all_ok = False
    if all_ok:
        print("\nAlle Assets da. Lizenzen siehe CREDITS.txt (Namensnennung noetig).")
    else:
        print("\nMind. ein Download fehlte -- Demo nutzt dann den prozeduralen Ersatz.")
    return 0 if all_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
