"""Laedt die Musikmodule fuer die Drachenhauch-Demo herunter.

Quelle: **The Mod Archive**, Rubrik *Public Domain* -- dort listet die Seite
ausschliesslich Module, deren Urheber sie gemeinfrei gestellt haben. Alle vier
Stuecke stammen von **Drozerix**, der sein gesamtes Werk gemeinfrei
veroeffentlicht.

Die Module sind zusammen ~0,6 MB und gemeinfrei, liegen also mit im Repo --
dieses Skript dient der Reproduzierbarkeit und dem Nachweis der Herkunft.
Aufruf:

    .venv\\Scripts\\python.exe gbdemo\\download_music.py

Ein anderes Stueck einsetzen: Modul-Nummer von der Detailseite ablesen
(`modarchive.org/index.php?request=view_by_moduleid&query=<nr>`), unten
eintragen, Skript laufen lassen, in `gbdemo.dh` `MUSIK` umstellen.
"""
from __future__ import annotations

import sys
import urllib.request
from pathlib import Path

# Zielname -> Modul-Nummer bei modarchive.org (alle "Public Domain", Drozerix)
TRACKS = {
    "stardust_jam.mod": 201039,       # ~6:24 -- das Stueck, auf das die Demo geschnitten ist
    "silicon_dancer.mod": 209692,     # ~4:05
    "neon_techno.mod": 178172,        # ~3:58
    "mecanum_overdrive.xm": 175349,   # ~2:57 -- 4 Kanaele, treibend
    "assembly.xm": 209551,            # ~1:55 -- nach der Demoparty benannt
    "keygen_wraith.xm": 207854,       # ~1:33 -- 6 Kanaele, BPM 240
    "building_energy.xm": 185456,     # ~1:29 -- 24 Kanaele, dichtester Satz
    "cyber_spider.xm": 192354,        # ~1:03 -- 10 Kanaele, schnellstes Stueck
}

BASE = "https://api.modarchive.org/downloads.php?moduleid="
OUT = Path(__file__).resolve().parent / "assets" / "music"


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    fehler = 0
    for name, modid in TRACKS.items():
        ziel = OUT / name
        try:
            req = urllib.request.Request(f"{BASE}{modid}", headers={"User-Agent": "Mozilla/5.0"})
            with urllib.request.urlopen(req, timeout=60) as r:
                daten = r.read()
        except Exception as e:                     # noqa: BLE001 -- Netzfehler nur melden
            print(f"  FEHLER {name}: {e}")
            fehler += 1
            continue
        # Grobpruefung, damit eine Fehlerseite nicht als Modul im Repo landet
        kopf_ok = daten[:17] == b"Extended Module: " or (
            len(daten) > 1084 and daten[1080:1084] in (b"M.K.", b"M!K!", b"4CHN", b"6CHN", b"8CHN")
        )
        if not kopf_ok:
            print(f"  FEHLER {name}: sieht nicht nach einem Modul aus ({len(daten)} Bytes)")
            fehler += 1
            continue
        ziel.write_bytes(daten)
        print(f"  ok {name} ({len(daten) // 1024} KB)")
    return 1 if fehler else 0


if __name__ == "__main__":
    sys.exit(main())
