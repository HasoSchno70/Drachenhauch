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
  (`R`), Pipette (`I`), **Auswahl (`S`)**. **Rechtsklick** löscht immer (egal welches Werkzeug).
- **Auswahl-Werkzeug (`S`):** Ziehe ein Rechteck über die Tiles (gestrichelter Rahmen). Dann **`Strg+C`** kopiert die Region, **`Strg+X`** schneidet sie aus (kopieren + leeren), **`Strg+V`** stempelt das Clipboard mit der oberen-linken Ecke an der aktuellen Maus-Zelle (bzw. am Auswahl-Ursprung), **`Entf`** leert die Auswahl. Alles ist undobar. Praktisch zum Duplizieren von Map-Bereichen (Räume, Plattform-Muster).
- **Layer** (rechts, oben = vorne): `+` Tile-Layer, **`+◇` Object-Layer**, `−`
  löschen, `▲`/`▼` sortieren, Häkchen = Sichtbarkeit, Doppelklick = umbenennen.
  Die aktive Layer wird beim Malen verändert; die anderen werden (optional)
  abgeblendet dargestellt.
- **Tile-Eigenschaften ...** (links) — Per-Tile-Properties für das gewählte
  Tile, z. B. `solid` (bool), `damage` (int). Das Kollisionssystem
  ([`tile_collide`](#)) liest `solid` via `TILED_TILE_PROP_BOOL`.
- **Ansicht:** Gitter ein/aus, „Andere abblenden“, Zoom `Strg++`/`Strg+−`.
- **Rückgängig/Wiederholen:** `Strg+Z` / `Strg+Y` (Mal- **und** Objekt-Operationen).

### Object-Layer (Spawn-Punkte, Trigger, Zonen)

Ein **Object-Layer** (`+◇`) hält frei platzierte Objekte statt Tiles — ideal für
Spawn-Punkte, Trigger oder Zonen, die das Spiel zur Laufzeit ausliest (nicht
gezeichnet werden). Ist ein Object-Layer aktiv, schaltet der Canvas auf
Objekt-Bearbeitung:

- **Klick** auf eine leere Stelle → **Punkt-Objekt** (Spawn-Marker, auf die
  Zellmitte gesnappt). Es öffnet sich gleich der Bearbeiten-Dialog für Name/Typ.
- **Ziehen** → **Rechteck-Objekt** (Zone/Trigger, auf Zellen gerundet).
- **Klick auf ein Objekt** → auswählen; **ziehen** verschiebt es.
- **Doppelklick** → Name, **Typ** (Tiled-`class`) und **Properties** bearbeiten
  (z. B. `hp=int`, `active=bool`).
- **Entf** oder **Rechtsklick** → ausgewähltes Objekt löschen.

Im Spiel auslesen über `TILED_OBJECT_COUNT/NAME/TYPE/X/Y/WIDTH/HEIGHT` und
`TILED_OBJECT_PROP_*` (siehe Code-Hinweis im GB-Code-Export). Koordinaten sind
**Pixel** (wie Tiled).

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
- Orthogonale **Tile-Layer** (`tilelayer`) und **Object-Layer** (`objectgroup`):
  Objekte mit Name/Typ/Properties und Pixel-Koordinaten (Punkte als `point`,
  sonst Rechtecke). 1:1 das Format, das `TILED_LOAD` + `TILED_OBJECT_*` liest.

Datenmodell + Serialisierung liegen Qt-frei in `gamebasic/tilemap/document.py`
(headless getestet: `tests/test_tilemapeditor.py` prüft den Roundtrip durch
`TILED_LOAD`).
