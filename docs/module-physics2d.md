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
| `PHYS2D_BODY_X/Y(w, idx)` | Position |
| `PHYS2D_BODY_ANGLE(w, idx) AS FLOAT` | Drehwinkel in Radiant (für Sprite-Rotation) |
| `PHYS2D_BODY_VX/VY(w, idx)` | lineare Geschwindigkeit (z.B. „falle ich?") |
| `PHYS2D_SET_VEL(w, idx, vx, vy)` | Geschwindigkeit setzen |
| `PHYS2D_APPLY_IMPULSE(w, idx, ix, iy)` | Impuls (z.B. Sprung/Schuss) |
| `PHYS2D_SET_POS(w, idx, x, y)` | Position teleportieren |
| `PHYS2D_SET_DYNAMIC(w, idx, dynamic)` | zwischen statisch und dynamisch umschalten — fuer Aufbauten, die erst **stehen** und dann zusammenfallen sollen (Mauer, Logo, Turm). Weckt den Koerper mit auf; ruhende Koerper laesst Rapier sonst schlafen und ein frisch dynamisch gemachter Klotz haenge reglos in der Luft |
| `PHYS2D_IS_DYNAMIC(w, idx) AS BOOLEAN` | ist der Koerper dynamisch? |
| `PHYS2D_LOCK_ROTATION(w, idx, locked)` | Rotation sperren — z.B. damit eine Spielfigur nicht umkippt |
| `PHYS2D_REMOVE(w, idx)` | Körper entfernen |
| `PHYS2D_COUNT(w) AS INTEGER` | Anzahl lebender Körper |

## Muster: Sprite an einen Körper koppeln

Position folgt aus `BODY_X`/`BODY_Y`, die Drehung aus `BODY_ANGLE` (Radiant).
Mit **`DRAWIMAGEROT`** (zentrierter, gedrehter Sprite-Blit) koppelt man ein Bild
direkt an einen Körper — `BODY_ANGLE` ist Radiant, der Blit erwartet Grad, also
`DEG(...)`:

```basic
DRAWIMAGEROT(img, PHYS2D_BODY_X(world, id), PHYS2D_BODY_Y(world, id), _
             DEG(PHYS2D_BODY_ANGLE(world, id)), 2.0)   ' zentriert, 2x skaliert
```

Komplettes Beispiel: [examples/145_physics2d_sprites.dh](../examples/145_physics2d_sprites.dh)
(purzelnde Sprites). Für reine Vektor-Optik zeichnet
[examples/112_physics2d.dh](../examples/112_physics2d.dh) Boxen als rotierte
Linien-Outlines und Kreise mit Spin-Linie.

## Platformer-Tipp

Für eine Spielfigur, die nicht umkippt: dynamische Box + `PHYS2D_LOCK_ROTATION(w,
id, TRUE)`, Bewegung über `PHYS2D_SET_VEL` (horizontale Komponente setzen,
vertikale aus der Physik lassen) und Sprung über `PHYS2D_APPLY_IMPULSE(w, id, 0,
-impuls)`. (Für rein Tile-basierte Platformer bleibt
[`tile_collide`](../CLAUDE.md)/`controller` die leichtere Wahl — `physics2d` ist
für *echte* Dynamik: Stapeln, Werfen, Rollen, Ragdolls, Sandbox.)

Externer Typ `PHYS2D_WORLD`. Implementierung
`rust/drachenhauch_runtime/src/physics2d.rs` (pure-Rust, ungated), Demo
[examples/112_physics2d.dh](../examples/112_physics2d.dh), Tests
`tests/test_physics2d.py`.
