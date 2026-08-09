# Modul `physics3d` — echte 3D-Starrkörper-Physik (Rapier3D)

Ein **vollwertiger 3D-Physik-Solver** via [Rapier3D](https://rapier.rs):
Schwerkraft, Integration, Kollisionsauflösung und Restitution (Sprungkraft).
Das 3D-Pendant zu [`physics2d`](module-physics2d.md) — und der große
Unterschied zum [`physics`](module-physics.md)-Modul, das nur stateless
Kollisions-Mathematik liefert: hier simulieren echte dynamische Körper.

> **Nur native Runtime** (`dhrt` / F6 im Editor). Die `PHYS_WORLD` hält die
> Simulations-Pipeline; einzelne Körper werden über einen **Integer-Index**
> angesprochen (Rückgabe von `PHYS3D_ADD_*`).

```basic
IMPORT "physics3d"

DIM w AS PHYS_WORLD
w = PHYS3D_NEW()                              ' Default-Schwerkraft (0, -9.81, 0)

' Statischer Boden + dynamischer Würfel, der draufprallt
DIM boden AS INTEGER
boden = PHYS3D_ADD_BOX(w, 0.0, 0.0, 0.0,  10.0, 0.5, 10.0,  FALSE, 0.3)
DIM kiste AS INTEGER
kiste = PHYS3D_ADD_BOX(w, 0.0, 8.0, 0.0,  0.5, 0.5, 0.5,    TRUE,  0.4)

WHILE NOT QUITREQUESTED()
    PHYS3D_STEP(w, DELTA())                   ' eine Simulationsstufe
    ' ... Position + Rotation des Körpers abfragen und rendern ...
    DIM px AS FLOAT
    px = PHYS3D_BODY_X(w, kiste)
    FLIP()
WEND
```

## Welt

| Funktion | Rückgabe | Wirkung |
|---|---|---|
| `PHYS3D_NEW()` | `PHYS_WORLD` | neue Welt; Default-Schwerkraft `(0, -9.81, 0)` |
| `PHYS3D_SET_GRAVITY(w, gx, gy, gz)` | — | Schwerkraft-Vektor setzen (z.B. `(0, -20, 0)` für „schwerere" Welt, `(0,0,0)` für Schwerelosigkeit) |
| `PHYS3D_STEP(w, dt)` | — | die Simulation um `dt` Sekunden vorrücken (typisch `DELTA()` oder ein fester Schritt wie `1.0/60.0`) |
| `PHYS3D_COUNT(w)` | INTEGER | Anzahl lebender Körper |

## Körper anlegen

`dynamic` ist `TRUE` (bewegt sich, fällt) oder `FALSE` (statisch, unbeweglich —
Boden/Wände). `bounce` ist die Restitution `0..1` (0 = kein Abprall, 1 = voller
Abprall). Beide Flags akzeptieren `TRUE`/`FALSE` **oder** `1`/`0`. Rückgabe ist
der **Körper-Index** (INTEGER) für alle folgenden Aufrufe.

| Funktion | Rückgabe | Wirkung |
|---|---|---|
| `PHYS3D_ADD_BOX(w, x, y, z, hx, hy, hz, dynamic, bounce)` | INTEGER (idx) | Quader bei `(x,y,z)` mit **Halb-Ausdehnungen** `hx,hy,hz` (also Vollgröße `2·h`) |
| `PHYS3D_ADD_SPHERE(w, x, y, z, r, dynamic, bounce)` | INTEGER (idx) | Kugel bei `(x,y,z)` mit Radius `r` |

## Körper abfragen und steuern

| Funktion | Rückgabe | Wirkung |
|---|---|---|
| `PHYS3D_BODY_X(w, idx)` / `_Y` / `_Z` | FLOAT | aktuelle Position |
| `PHYS3D_BODY_QX(w, idx)` / `_QY` / `_QZ` / `_QW` | FLOAT | Rotation als **Quaternion** `(x, y, z, w)` |
| `PHYS3D_SET_POS(w, idx, x, y, z)` | — | Position hart setzen (teleportieren) |
| `PHYS3D_SET_VEL(w, idx, vx, vy, vz)` | — | Linear-Geschwindigkeit setzen |
| `PHYS3D_APPLY_IMPULSE(w, idx, ix, iy, iz)` | — | einmaligen Impuls geben (Sprung, Schuss, Explosion) |
| `PHYS3D_REMOVE(w, idx)` | — | Körper aus der Welt entfernen |

## Rendern (Quaternion → Transform)

`PHYS3D_BODY_*` liefert Position **und** ein Rotations-Quaternion. Zum Zeichnen
baut man daraus mit dem [`m3d`](module-m3d.md)-Modul eine Welt-Matrix und
rendert ein Modell darüber — die Box/Kugel ist nur der Kollisions-Proxy:

```basic
IMPORT "g3d"
IMPORT "m3d"
IMPORT "physics3d"

DIM one AS VEC3
one = VEC3_NEW(1.0, 1.0, 1.0)

' ... pro Frame, pro Körper idx:
DIM pos AS VEC3
pos = VEC3_NEW(PHYS3D_BODY_X(w, idx), PHYS3D_BODY_Y(w, idx), PHYS3D_BODY_Z(w, idx))
DIM rot AS QUAT
rot = QUAT_NEW(PHYS3D_BODY_QX(w, idx), PHYS3D_BODY_QY(w, idx), _
               PHYS3D_BODY_QZ(w, idx), PHYS3D_BODY_QW(w, idx))
DIM m AS MAT4
m = MAT4_TRS(pos, rot, one)          ' Translation · Rotation · Scale
MODEL_MATRIX(meinModell, m, WHITE)
```

## Abgrenzung

- **`physics3d`** (dieses Modul) — *echte* dynamische 3D-Körper mit Solver.
- **[`physics`](module-physics.md)** — nur stateless Kollisions-/Vektor-Mathe (kein State, keine Schwerkraft).
- **[`physics2d`](module-physics2d.md)** — das 2D-Pendant (Rapier2D).

Vollständiges Beispiel: [examples/107_physics3d.gb](../examples/107_physics3d.gb).
