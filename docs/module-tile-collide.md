# Modul `tile_collide`

Box-vs-Tilemap-Kollision fuer Platformer. Klassisches **separates-Achsen-Sweep**-Pattern: erst X-Bewegung resolven, dann Y. Liefert die neue Position + ein Hit-Flag pro Achse.

Benoetigt eine geladene `TILED_MAP` (siehe [tiled-Modul](module-tiled.md)).

```basic
IMPORT "tiled"
IMPORT "tile_collide"
```

## Übersicht

| Funktion | Rueckgabe |
|---|---|
| `TILE_SWEEP_X(map, layer_idx, x, y, w, h, dx)` | TUPLE `(new_x, hit_bool)` |
| `TILE_SWEEP_Y(map, layer_idx, x, y, w, h, dy)` | TUPLE `(new_y, hit_bool)` |
| `TILE_IS_SOLID(map, layer_idx, tx, ty)` | BOOLEAN (Tile-Koords) |
| `TILE_AT_PIXEL(map, layer_idx, px, py)` | INTEGER (GID an Pixel-Pos) |

**Konvention:** `x, y, w, h, dx, dy` sind in **Pixeln**. `tx, ty` in **Tile-Einheiten**. `dy > 0` = nach unten (Gravitation in Screen-Coords).

## Klassisches Platformer-Update

```basic
' Pro Frame:
INPUT_UPDATE()

' Horizontal: User-Input → Velocity
vx = INPUT_AXIS("left", "right") * 2.0

' Spring-Logik
IF INPUT_PRESSED("jump") AND on_ground THEN
    vy = -5.0
    on_ground = FALSE
END IF

' Gravitation
vy = vy + 0.3
IF vy > 6.0 THEN vy = 6.0

' Kollision: X-Sweep zuerst, dann Y mit dem neuen X
DIM rx AS TUPLE
rx = TILE_SWEEP_X(level, 0, px, py, PW, PH, vx)
px = rx[0]
IF rx[1] THEN vx = 0.0      ' Wand -> Velocity weg

DIM ry AS TUPLE
ry = TILE_SWEEP_Y(level, 0, px, py, PW, PH, vy)
py = ry[0]
IF ry[1] THEN
    IF vy > 0.0 THEN on_ground = TRUE    ' Boden-Touch
    vy = 0.0
ELSE
    on_ground = FALSE                    ' nichts unter uns
END IF
```

Dieses Muster ist die Standard-Platformer-Physik. Separater X- und Y-Sweep verhindert "Wand-Klettern" durch Diagonal-Hits.

## Solid-Detection

Wann ist ein Tile "solid", also kollisionsfaehig? Zwei Modi:

**Modus 1: Per-Tile-Property** (bevorzugt)

In Tiled: Tile auswaehlen → Custom Properties → `solid: bool = true`. Das Modul nutzt automatisch diesen Modus, sobald **irgendein** Tile im Tileset eine `solid`-Property hat.

Vorteil: Du kannst eine MISCHUNG aus solid und nicht-solid Tiles in derselben Layer haben. Klassischer Fall: Boden-Tiles solid, Gras-Spitzen-Deko nicht solid (Player kann durchlaufen).

**Modus 2: Convention-Fallback**

Wenn das Tileset GAR KEINE `solid`-Properties hat: jeder GID > 0 in der Layer gilt als solid. Convention: du legst eine dedizierte Collision-Layer (z.B. `collision`) an, in der nur Wand-Tiles liegen.

Ist ein 1:1-Mapping einfacher, weil du die Collision-Daten visuell vom Rendering trennst.

## Welt-Rand

`TILE_IS_SOLID` liefert `TRUE` fuer Tile-Koordinaten ausserhalb der Map (negative, oder >= width/height). So blockiert der Welt-Rand automatisch — der Player kann nicht aus dem Level fallen, ohne dass du es im Spiel-Code abfangen musst.

Wenn du **moechtest**, dass der Player nach unten aus der Welt faellt (Death-Plane), pruefst du das selbst zusaetzlich:

```basic
IF py > MAP_HEIGHT_IN_PIXELS THEN
    Die()
END IF
```

## Zwei Sweeps-Reihenfolge: warum X zuerst?

Mathematisch waere "echt" eine **diagonale** Bewegung. Praktisch macht man X und Y SEPARAT, weil:

- Reine Diagonal-Sweeps kollidieren manchmal mit Ecken so, dass der Player auf einer Wand "haengt" wenn er nach links-unten will. Separate Achsen vermeiden das.
- Die Reihenfolge "X dann Y" priorisiert horizontale Bewegung — typisches Platformer-Gefuehl.
- Standard-Implementierung in fast jedem 2D-Engine (Godot, Unity-2D-Tilemap, ...) macht's so.

Wer einen Top-Down-Shooter mit echter Diagonalbewegung will, sweept X und Y unabhaengig — die Reihenfolge ist dann egal, beide werden gleich behandelt.

## One-Way-Platforms

V1 hat keine direkte one-way-platform Unterstuetzung (Plattform die man von unten durchspringen aber von oben drauflanden kann). Implementierung als User-Code:

```basic
' Vor dem Y-Sweep: pruefe, ob die Plattform unter uns ist UND wir nach unten fallen
' UND aktuell NICHT auf der Plattform stehen.
' Wenn alles ja: nimm Plattform-Tiles temporaer als solid; sonst nicht.
```

Das ist Game-spezifisch. Wenn du das oft brauchst, koennte ein zukuenftiges `tile_collide_oneway`-Builtin die Last vereinfachen. Heute aber als BASIC-Code drin.

## Slopes

Auch nicht v1. Tile-basiertes Slope-Handling braucht entweder spezielle Per-Tile-Property (`slope: "up_right"`) oder Polygon-Collision, beides ist eine Extension fuer eine spaetere Runde.

## Ad-hoc Punkt-Tests

`TILE_AT_PIXEL(map, layer_idx, px, py)` ist nuetzlich fuer ad-hoc-Tests jenseits des Box-Sweeps: "ist unter dem Player gerade ein Stein-Tile?".

```basic
DIM gid_below AS INTEGER
gid_below = TILE_AT_PIXEL(level, 0, px + PW / 2, py + PH + 1)
IF TILED_TILE_PROP_STRING(level, gid_below, "type") = "lava" THEN
    Burn()
END IF
```

## Performance

`TILE_SWEEP_X/Y` iteriert ueber die ueberlappenden Tiles in der Bewegungsrichtung — bei typischen Player-Geschwindigkeiten (< 8 Pixel/Frame) sind das 2-3 Tiles. Pro Frame: 2 Sweeps × 2-3 Tile-Checks = 4-6 Operationen. Vernachlaessigbar selbst bei mehreren tausend kollidierenden Entities.

Solid-Detection wird beim ersten Aufruf pro Map gecached — kein wiederholtes Property-Lookup.

## Beispiel

[examples/77_tiled_platformer.gb](../examples/77_tiled_platformer.gb) — komplettes Pattern mit Tiled-Map, Atlas, Tile-Layer, Z-Layer, Input-Mapping und separat-Achsen-Sweep.
