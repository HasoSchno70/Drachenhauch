"""Traegt Uebersetzungen in den Katalog ein.

    python i18n/apply.py patch.json [katalog.json]

`patch.json` ist { "deutscher text": "english text", ... }. Eintraege, die
der Katalog nicht kennt, werden GEMELDET statt still geschluckt -- ein
Tippfehler im deutschen Schluessel wuerde sonst als "uebersetzt" gelten und
im Buch trotzdem deutsch erscheinen.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

HIER = Path(__file__).resolve().parent


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    patch = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    ziel = Path(sys.argv[2]) if len(sys.argv) > 2 else HIER / "en.json"
    katalog = json.loads(ziel.read_text(encoding="utf-8"))

    unbekannt, gesetzt, ueberschrieben = [], 0, 0
    for de, en in patch.items():
        if de not in katalog:
            unbekannt.append(de)
            continue
        if katalog[de]:
            ueberschrieben += 1
        katalog[de] = en
        gesetzt += 1

    ziel.write_text(json.dumps(katalog, ensure_ascii=False, indent=1), encoding="utf-8")

    offen = sum(1 for k, v in katalog.items() if k != "_veraltet" and not v)
    gesamt = sum(1 for k in katalog if k != "_veraltet")
    print(f"{gesetzt} eingetragen ({ueberschrieben} ersetzt), "
          f"{gesamt - offen}/{gesamt} uebersetzt "
          f"({100 * (gesamt - offen) / gesamt:.1f} %)")
    for u in unbekannt:
        print(f"  UNBEKANNTER SCHLUESSEL: {u[:70]!r}")
    return 1 if unbekannt else 0


if __name__ == "__main__":
    raise SystemExit(main())
