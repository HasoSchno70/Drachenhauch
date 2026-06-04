# Tilemap-/Level-Editor

In-House-Editor zum Malen von 2D-Leveln aus einem Tileset-Bild auf ein
Gitter — mit mehreren Layern und Per-Tile-Properties. Speichert/lädt als
**Tiled-JSON**, das das [`tiled`-Modul](#) (`TILED_LOAD`) direkt einliest, und
schließt damit den Kreis mit dem [Sprite-Editor](sprite-editor.md)
(dessen Atlas-PNG als Tileset dient).

## Starten

Aus dem **Code-Editor**: Toolbar-Button (Gitter-Symbol) oder
`Datei → Tilemap-/Level-Editor öffnen ...` (`Strg+Shift+G`). Standalone:
`gbtilemap` oder `gbrun.py --tilemap [datei.json]` (braucht `PySide6`).

## Bedienung

- **Tileset laden** (Werkzeugleiste) — ein PNG (z. B. ein Sprite-Atlas-Sheet).
  Spalten/Tile-Anzahl ergeben sich aus Bildgröße ÷ Tile-Größe. Links erscheint
  die **Palette**: ein Tile anklicken wählt es zum Malen.
- **Werkzeuge:** Stift (`B`), Radierer (`E`), Füllen/Bucket (`G`), Rechteck
  (`R`), Pipette (`I`). **Rechtsklick** löscht immer (egal welches Werkzeug).
- **Layer** (rechts, oben = vorne): `+`/`−` anlegen/löschen, `▲`/`▼` sortieren,
  Häkchen = Sichtbarkeit, Doppelklick = umbenennen. Die aktive Layer wird beim
  Malen verändert; die anderen werden (optional) abgeblendet dargestellt.
- **Tile-Eigenschaften ...** (links) — Per-Tile-Properties für das gewählte
  Tile, z. B. `solid` (bool), `damage` (int). Das Kollisionssystem
  ([`tile_collide`](#)) liest `solid` via `TILED_TILE_PROP_BOOL`.
- **Ansicht:** Gitter ein/aus, „Andere abblenden“, Zoom `Strg++`/`Strg+−`.
- **Rückgängig/Wiederholen:** `Strg+Z` / `Strg+Y` (Mal-Operationen).

## Speichern / Laden

`Speichern (unter)` schreibt **Tiled-JSON** (`.json`). Der Tileset-Bildpfad wird
relativ zur Map gespeichert — Map und Tileset-PNG also am besten ins selbe
Verzeichnis legen. `Öffnen` liest dasselbe Format zurück (auch von echtem
[Tiled](https://www.mapeditor.org/) exportierte Maps, sofern CSV-Tile-Daten und
ein eingebettetes Tileset).

## Im Spiel laden

```basic
IMPORT "tiled"
DIM lvl AS TILED_MAP
lvl = TILED_LOAD("level.json")
' ... TILED_TILE_AT / TILED_TILE_PROP_BOOL / DRAWIMAGEPART ...
```

## Export (GB-Code)

`GB-Code` erzeugt ein **selbstständiges Renderer-Programm**: es lädt das
Tileset per `LOADIMAGE`, die Map per `TILED_LOAD` und zeichnet jedes Tile mit
`DRAWIMAGEPART` (Quell-Rechteck aus `gid − 1`, Spalten = Tileset-Breite). Map
als `.json` speichern, Tileset-PNG daneben legen, dann ausführen. Läuft in
**beiden** Pfaden — Tree-Walker und native Runtime `gbrt`.

## Format-Details

- Ein eingebettetes Tileset, `firstgid = 1` → lokale Tile-ID = `gid − 1`.
- Tile-Daten als CSV-Liste von GIDs (kein base64), row-major, `0` = leer.
- Nur orthogonale Tile-Layer (Object-Layer für Spawn-Punkte können später
  dazukommen — bis dahin in Tiled selbst ergänzbar).

Datenmodell + Serialisierung liegen Qt-frei in `gamebasic/tilemap/document.py`
(headless getestet: `tests/test_tilemapeditor.py` prüft den Roundtrip durch
`TILED_LOAD`).
