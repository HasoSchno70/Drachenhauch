"""Kuratierte Showcase-Demos fuer das Welcome-Panel.

Single-Source-of-Truth: sowohl das Welcome-Panel (rendert die Karten) als
auch das Thumbnail-Generator-Skript (`tools/gen_showcase_thumbs.py`) lesen
diese Liste. Jeder Eintrag verweist auf eine `examples/<file>` und ein
Thumbnail unter `examples/screenshots/<stem>.png` (per Generator erzeugt;
fehlt es, faellt das Panel auf eine prozedurale Platzhalter-Karte zurueck).

`frames` steuert, nach wie vielen gerenderten Frames der Headless-Screenshot
fuer diese Demo gezogen wird (genug, damit Animation/Aufbau sichtbar ist).
"""
from __future__ import annotations

from pathlib import Path

# (file, title, desc, frames-fuer-Screenshot)
SHOWCASE: list[dict] = [
    {"file": "99_ibl_hdr.gb",        "title": "HDR Image-Based Lighting",
     "desc": "Echtes HDR-Cubemap-IBL -- Metalle spiegeln die Umgebung.",
     "frames": 150},
    {"file": "97_pbr_reactor.gb",    "title": "PBR-Reaktor",
     "desc": "Cook-Torrance-PBR mit Metalness/Roughness und Punktlichtern.",
     "frames": 150},
    {"file": "48_orbital.gb",        "title": "Orbital",
     "desc": "Sonnensystem mit Tween-Pulsation, Kometen-Trails und HUD.",
     "frames": 150},
    {"file": "93_shadows.gb",        "title": "Shadow-Mapping",
     "desc": "Directional-Light wirft echte Schatten (Depth-FBO + PCF).",
     "frames": 120},
    {"file": "86_postfx_shaders.gb", "title": "Post-FX-Shader",
     "desc": "CRT/Bloom/Vignette als Fragment-Shader ueber die Szene.",
     "frames": 120},
    {"file": "85_cybermatic_demo.gb", "title": "Cybermatic-Demo",
     "desc": "Audiovisuelle Demoscene-Show mit Effekt-Parts.",
     "frames": 220},
    {"file": "78_particle_catalog.gb", "title": "Partikel-Katalog",
     "desc": "Render-Modi und Farbverlaeufe des particles-Moduls.",
     "frames": 160},
    {"file": "65_amiga_demo.gb",     "title": "Amiga-Demo",
     "desc": "Copper-Bars, Sinus-Scroller und Bob-Plasma -- Retro pur.",
     "frames": 200},
    {"file": "70_hires_showcase.gb", "title": "Hi-Res-Showcase",
     "desc": "Hochaufgeloeste 2D-Grafik-Pipeline in Aktion.",
     "frames": 160},
    {"file": "77_tiled_platformer.gb", "title": "Tiled-Platformer",
     "desc": "Tiled-Map + Tile-Kollision + Character-Controller.",
     "frames": 160},
    {"file": "67_wobbler.gb",        "title": "Wobbler",
     "desc": "Sinus-verzerrtes Plasma -- klassischer Demo-Effekt.",
     "frames": 140},
    {"file": "34_schneefall.gb",     "title": "Schneefall",
     "desc": "Stimmungsvolles Partikel-Schneetreiben.",
     "frames": 160},
]


def thumb_path(project_root: Path, entry: dict) -> Path:
    """Pfad zum (evtl. noch nicht existierenden) Thumbnail eines Eintrags."""
    stem = Path(entry["file"]).stem
    return project_root / "examples" / "screenshots" / f"{stem}.png"
