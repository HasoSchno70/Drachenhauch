"""Datenmodell + Tiled-JSON-I/O fuer den GameBasic-Tilemap-Editor.

Qt-frei, damit headless testbar (Roundtrip durch das `tiled`-Modul).
"""
from .document import TileLayer, TileMapDoc

__all__ = ["TileLayer", "TileMapDoc"]
