"""Sprite-Editor-Submodule.

Aufgespalten aus dem ehemaligen Monolith `drachenhauch/spriteeditor_qt.py`
(urspr. 5300+ Zeilen, jetzt ~4200). Extrahierte Submodule:

- `document`: Frame, SpriteDoc, Persistenz (PIL only, kein Qt-Import)
- `icons`: Tool- und Action-Icon-Painter (~540 Zeilen QPainter-Code)
- `tools`: alle Pixel-Manipulation-Tools (Pencil/Eraser/Bucket/Line/...)

Tools nutzen `app` als Duck-Type-Context -- siehe Modul-Docstring von
`tools.py` fuer das erwartete Interface. Ein dedicated Tool-Context-
Protocol mit Type-Hints ist denkbar fuer eine spaetere Iteration; aktuell
ist die Coupling pragmatisch und liefert testbare Tools (siehe
`tests/test_spriteeditor_tools.py`).
"""
