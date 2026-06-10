# Modul `physics2d` — echte 2D-Starrkörper-Physik (Rapier2D)

Ein **vollwertiger 2D-Physik-Solver** via [Rapier2D](https://rapier.rs):
Schwerkraft, Integration, Kollisionsauflösung, Restitution (Sprungkraft) und
Reibung. Das 2D-Pendant zu [`physics3d`](module-physics3d.md) — und der große
Unterschied zum [`physics`](module-physics.md)-Modul, das nur stateless
Kollisions-Mathematik liefert: hier simulieren echte dynamische Körper.

```basic
IMPORT "physics2d"

DIM world AS PHYS2D_WORLD
world = PHYS2D_NEW()

' statischer Boden + eine fallende Box
PHYS2D_ADD_BOX(world, 240, 400, 240, 10, FALSE, 0.2)      ' breit, statisch
DIM box AS INTEGER
box = PHYS2D_ADD_BOX(world, 240, 40, 16, 16, TRUE, 0.3)   ' dynamisch, springt

' --- pro Frame ---
PHYS2D_STEP(world, DELTA())
DIM x AS FLOAT
x = PHYS2D_BODY_X(world, box)
DIM y AS FLOAT
y = PHYS2D_BODY_Y(world, box)
DIM ang AS FLOAT
ang = PHYS2D_BODY_ANGLE(world, box)       ' Radiant -> Sprite rotieren
```

## Konventionen

- **Bildschirm-Koordinaten:** Y wächst nach **unten** (wie bei allen GB-Draw-
  Befehlen). Die Default-Schwerkraft zieht daher nach unten (positives Y).
- **Pixel-Maßstab:** intern ist `length_unit = 100` gesetzt (1 „Meter" = 100 px),
  damit pixel-große Welten stabil bleiben. Die Default-Schwerkraft ist
  `(0, 980)` px/s² (≈ Erdbeschleunigung). Eigene Einheiten via
  `PHYS2D_SET_GRAVITY`.
- **Box-Maße sind Halb-Extents:** `PHYS2D_ADD_BOX(w, x, y, hw, hh, …)` — `hw`/`hh`
  sind die **halbe** Breite/Höhe (Rapier-Cuboid). Ein 32×32-Sprite ⇒ `hw=hh=16`.
- **`dynamic`-/Lock-Flags** akzeptieren `TRUE`/`FALSE` **oder** `1`/`0`.
- **Körper-Index** ist ein stabiler INTEGER (auch nach `REMOVE` bleiben kleinere
  Indizes gültig — Tombstone-Slots).

## API

| Builtin | Wirkung |
|---|---|
| `PHYS2D_NEW() AS PHYS2D_WORLD` | neue Welt (Default-Gravitation 0,980) |
| `PHYS2D_SET_GRAVITY(w, gx, gy)` | Schwerkraft setzen |
| `PHYS2D_ADD_BOX(w, x, y, hw, hh, dynamic, bounce) AS INTEGER` | Rechteck (Halb-Extents), Restitution `bounce` 0..1 |
| `PHYS2D_ADD_CIRCLE(w, x, y, r, dynamic, bounce) AS INTEGER` | Kreis-Körper |
| `PHYS2D_STEP(w, dt)` | einen Zeitschritt simulieren (dt in s, intern auf 0.0001..0.05 geklemmt) |
| `PHYS2D_BODY_X / _Y(w, idx) AS FLOAT` | Position |
| `PHYS2D_BODY_ANGLE(w, idx) AS FLOAT` | Drehwinkel in Radiant (für Sprite-Rotation) |
| `PHYS2D_BODY_VX / _VY(w, idx) AS FLOAT` | lineare Geschwindigkeit (z.B. „falle ich?") |
| `PHYS2D_SET_VEL(w, idx, vx, vy)` | Geschwindigkeit setzen |
| `PHYS2D_APPLY_IMPULSE(w, idx, ix, iy)` | Impuls (z.B. Sprung/Schuss) |
| `PHYS2D_SET_POS(w, idx, x, y)` | Position teleportieren |
| `PHYS2D_LOCK_ROTATION(w, idx, locked)` | Rotation sperren — z.B. damit eine Spielfigur nicht umkippt |
| `PHYS2D_REMOVE(w, idx)` | Körper entfernen |
| `PHYS2D_COUNT(w) AS INTEGER` | Anzahl lebender Körper |

## Muster: Körper zeichnen

Position folgt direkt aus `BODY_X`/`BODY_Y`, die Drehung aus `BODY_ANGLE`
(Radiant). Kreise brauchen für die Drehung nur eine Radius-Linie; ein
**rotiertes Rechteck** zeichnet man als vier Linien (Ecken aus dem Winkel),
siehe [examples/112_physics2d.gb](../examples/112_physics2d.gb):

```basic
DIM ca AS FLOAT : ca = COS(ang)
DIM sa AS FLOAT : sa = SIN(ang)
' Ecke (-s,-s) der Box mit Halb-Seite s um (cx,cy) rotiert:
DIM ex AS INTEGER : ex = INT(cx + (-s) * ca - (-s) * sa)
DIM ey AS INTEGER : ey = INT(cy + (-s) * sa + (-s) * ca)
' … die übrigen drei Ecken analog, dann mit LINEW verbinden.
```

> **Hinweis:** Ein direkter *rotierter Sprite-Blit* (`DRAWIMAGE` mit Winkel) ist
> noch kein Core-Builtin — sinnvolle Folge-Erweiterung, um Sprites bequem an
> `BODY_ANGLE` zu koppeln. Bis dahin: Box-Outlines wie oben, oder Sprite ohne
> Rotation an `BODY_X/Y` setzen.

## Platformer-Tipp

Für eine Spielfigur, die nicht umkippt: dynamische Box + `PHYS2D_LOCK_ROTATION(w,
id, TRUE)`, Bewegung über `PHYS2D_SET_VEL` (horizontale Komponente setzen,
vertikale aus der Physik lassen) und Sprung über `PHYS2D_APPLY_IMPULSE(w, id, 0,
-impuls)`. (Für rein Tile-basierte Platformer bleibt
[`tile_collide`](../CLAUDE.md)/`controller` die leichtere Wahl — `physics2d` ist
für *echte* Dynamik: Stapeln, Werfen, Rollen, Ragdolls, Sandbox.)

Externer Typ `PHYS2D_WORLD`. Implementierung
`rust/gb_runtime/src/physics2d.rs` (pure-Rust, ungated), Demo
[examples/112_physics2d.gb](../examples/112_physics2d.gb), Tests
`tests/test_physics2d.py`.
