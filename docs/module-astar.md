# Modul `astar`

A*-Pathfinding auf einem Tile-Grid. Drei Heuristiken, optionale Diagonalbewegung, Anti-Cornercutting.

```basic
IMPORT "astar"
```

## Übersicht

| Funktion | Rückgabe / Wirkung |
|---|---|
| `ASTAR_NEW(w, h)` | ASTAR_GRID — alles passierbar |
| `ASTAR_CLEAR(g)` | alle Walls + Pfad löschen |
| `ASTAR_WIDTH(g)` / `ASTAR_HEIGHT(g)` | INTEGER |
| `ASTAR_SET_WALL(g, x, y)` | Tile als unpassierbar markieren |
| `ASTAR_SET_PASSABLE(g, x, y)` | wieder freigeben |
| `ASTAR_IS_WALL(g, x, y)` | BOOLEAN |
| `ASTAR_SET_DIAGONAL(g, allow)` | BOOLEAN — Diagonal-Bewegung an/aus |
| `ASTAR_SET_HEURISTIC(g, name$)` | `"manhattan"` \| `"euclid"` \| `"chebyshev"` |
| `ASTAR_SET_DIAGONAL_COST(g, cost)` | Default `√2` ≈ 1.414 |
| `ASTAR_FIND(g, sx, sy, ex, ey)` | BOOLEAN — `TRUE` wenn Pfad gefunden |
| `ASTAR_PATH_LEN(g)` | INTEGER — `0` wenn kein Pfad |
| `ASTAR_PATH_X(g, idx)` / `ASTAR_PATH_Y(g, idx)` | INTEGER |
| `ASTAR_PATH_COST(g)` | FLOAT — Gesamtkosten |
| `ASTAR_CLEAR_PATH(g)` | nur Pfad-Daten löschen, Walls bleiben |

## Konzept

Ein Grid kennt nur **passierbar** vs. **Wand**. `ASTAR_FIND(start, ende)` sucht den kürzesten Pfad und legt ihn intern ab. `PATH_LEN` / `PATH_X` / `PATH_Y` lesen ihn aus.

```basic
IMPORT "astar"

DIM grid AS ASTAR_GRID
grid = ASTAR_NEW(20, 15)

ASTAR_SET_WALL(grid, 5, 5)
ASTAR_SET_WALL(grid, 5, 6)
ASTAR_SET_WALL(grid, 5, 7)

DIM ok AS BOOLEAN
ok = ASTAR_FIND(grid, 0, 0, 19, 14)

IF ok THEN
    DIM i AS INTEGER
    FOR i = 0 TO ASTAR_PATH_LEN(grid) - 1
        PRINT ASTAR_PATH_X(grid, i), ASTAR_PATH_Y(grid, i)
    NEXT i
END IF
```

Der Pfad enthält **Start UND Ziel** — also bei einem direkten Nachbar-Schritt ist `ASTAR_PATH_LEN` = 2.

## Diagonal-Bewegung

Standardmäßig nur 4-fach orthogonal (oben/unten/links/rechts). Mit `ASTAR_SET_DIAGONAL(grid, TRUE)` kommen die vier Diagonalen dazu — Pfade werden meist deutlich kürzer.

```basic
ASTAR_SET_DIAGONAL(grid, TRUE)
ASTAR_FIND(grid, 0, 0, 3, 3)
PRINT ASTAR_PATH_LEN(grid)        ' 4 (Start + 3 Diagonalschritte)
PRINT ASTAR_PATH_COST(grid)       ' ~4.24  (3 * √2)
```

**Anti-Cornercutting**: Diagonal-Schritte sind nur erlaubt, wenn beide angrenzenden Orthogonal-Zellen frei sind. Sonst könnte eine Einheit durch eine "L"-Wand-Ecke schlüpfen.

```
  ##.    Diagonal von (0,0) -> (1,1) ist NICHT erlaubt,
  #..    weil (1,0) und (0,1) Walls sind.
  ...
```

## Heuristiken

A* nutzt eine Heuristik, um zu schätzen wie weit ein Knoten noch vom Ziel weg ist. Je passender die Heuristik, desto weniger Knoten muss der Algorithmus expandieren.

| Name | Beschreibung | Optimal bei |
|---|---|---|
| `manhattan` (Default) | `|dx| + |dy|` | nur orthogonale Bewegung |
| `euclid` | `√(dx² + dy²)` | beliebige Diagonalkosten |
| `chebyshev` | `max(|dx|, |dy|)` | Diagonal-Cost == 1 (einheitlich) |

```basic
ASTAR_SET_DIAGONAL(grid, TRUE)
ASTAR_SET_DIAGONAL_COST(grid, 1.0)        ' Diagonal-Schritt kostet 1 wie Ortho
ASTAR_SET_HEURISTIC(grid, "chebyshev")    ' optimal fuer diese Konfiguration
```

Heuristik-Namen sind case-insensitive (`"MANHATTAN"` geht auch).

## Pfad lesen

Nach erfolgreichem `ASTAR_FIND` gibt es zwei Wege, den Pfad zu konsumieren:

**Index-basiert**:

```basic
DIM i AS INTEGER
FOR i = 0 TO ASTAR_PATH_LEN(grid) - 1
    DIM x AS INTEGER
    DIM y AS INTEGER
    x = ASTAR_PATH_X(grid, i)
    y = ASTAR_PATH_Y(grid, i)
    ' i=0 ist Start, i=PATH_LEN-1 ist Ziel
    DrawTile(x, y)
NEXT i
```

**Schritt-für-Schritt für KI**:

```basic
' Nur den naechsten Schritt fuer eine bewegliche Einheit
IF ASTAR_PATH_LEN(grid) >= 2 THEN
    DIM nx AS INTEGER
    DIM ny AS INTEGER
    nx = ASTAR_PATH_X(grid, 1)        ' Index 0 ist die aktuelle Position
    ny = ASTAR_PATH_Y(grid, 1)
    MoveUnitTo(nx, ny)
END IF
```

## Performance

Heap-basierte Open-Liste (`heapq` intern). Komplexität ist `O((w·h) log(w·h))` im worst case. Für ein 200×200-Grid mit moderaten Hindernissen liegt eine Suche im Sub-Millisekunden-Bereich.

Wenn du die Karte oft updatest aber selten suchst: einmal `ASTAR_NEW`, danach nur Walls hinzufügen/entfernen und wiederholt `ASTAR_FIND` aufrufen. `ASTAR_CLEAR_PATH` wirft nur den letzten Pfad weg, nicht die Wall-Konfiguration.

## Externer Typ

`ASTAR_GRID` — opake Hülle um Wall-Bitfeld, Konfiguration und letzten Pfad.

## Beispiel: ASCII-Render

```basic
IMPORT "astar"

DIM grid AS ASTAR_GRID
grid = ASTAR_NEW(8, 6)
ASTAR_SET_DIAGONAL(grid, TRUE)

' Eine senkrechte Wand zwischen Start und Ziel
DIM y AS INTEGER
FOR y = 0 TO 4
    ASTAR_SET_WALL(grid, 4, y)
NEXT y

ASTAR_FIND(grid, 0, 0, 7, 0)

DIM hits[8, 6] AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO ASTAR_PATH_LEN(grid) - 1
    hits[ASTAR_PATH_X(grid, i), ASTAR_PATH_Y(grid, i)] = 1
NEXT i

DIM x AS INTEGER
FOR y = 0 TO 5
    DIM row AS STRING
    row = ""
    FOR x = 0 TO 7
        IF ASTAR_IS_WALL(grid, x, y) THEN
            row = row + "#"
        ELSEIF hits[x, y] = 1 THEN
            row = row + "*"
        ELSE
            row = row + "."
        END IF
    NEXT x
    PRINT row
NEXT y
```

## Siehe auch

- [`physics`](module-physics.md) — Kollisions-Tests + Vektor-Mathe für Pixel-genaue Kollisionen (komplementär: A* operiert auf Tiles, physics auf Punkten)
- Vollständiges Beispiel: [`examples/51_astar.dh`](../examples/51_astar.dh) — Labyrinth mit ASCII-Pfad-Visualisierung
