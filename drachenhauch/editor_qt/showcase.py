"""Kuratierte Showcase-Demos fuer das Welcome-Panel.

Die Liste steht seit 2026-09-15 in `examples/showcase.json` -- dieselbe
Datei lesen die IDE in Drachenhauch (`ide/ide.dh`, Willkommensseite) und der
Bild-Erzeuger `tools/showcase_bilder.dh`. Jeder Eintrag verweist auf eine
`examples/<file>` und ein Vorschaubild unter `examples/screenshots/<stem>.png`
(fehlt es, faellt das Panel auf eine Platzhalter-Karte zurueck).

`frames` steuert, nach wie vielen gerenderten Bildern das Vorschaubild
gezogen wird (genug, damit Animation/Aufbau sichtbar ist).
"""
from __future__ import annotations

import json
from pathlib import Path

_LISTE = Path(__file__).resolve().parents[2] / "examples" / "showcase.json"


def _laden() -> list[dict]:
    try:
        return json.loads(_LISTE.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return []


SHOWCASE: list[dict] = _laden()


def thumb_path(project_root: Path, entry: dict) -> Path:
    """Pfad zum (evtl. noch nicht existierenden) Thumbnail eines Eintrags."""
    stem = Path(entry["file"]).stem
    return project_root / "examples" / "screenshots" / f"{stem}.png"
