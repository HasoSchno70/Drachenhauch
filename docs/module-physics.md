# Modul: physics

Pure Built-in-Funktionen für 2D-Spiel-Mathematik: Kollisions-Tests, Distanzen, Vektor-Operationen, Ray-Cast. Kein eigener „Welt"-Manager — der Aufrufer hält Position, Velocity etc. selbst, das Modul rechnet nur.

```basic
IMPORT "physics"
```

## Übersicht

| Built-in | Zweck | Returns |
|---|---|---|
| `PHYSICS_BOX_BOX(x1,y1,w1,h1, x2,y2,w2,h2)` | AABB-AABB-Overlap | `BOOLEAN` |
| `PHYSICS_CIRCLE_CIRCLE(cx1,cy1,r1, cx2,cy2,r2)` | Kreis-Kreis | `BOOLEAN` |
| `PHYSICS_BOX_CIRCLE(bx,by,bw,bh, cx,cy,cr)` | Box-Kreis | `BOOLEAN` |
| `PHYSICS_POINT_BOX(px,py, bx,by,bw,bh)` | Punkt in Box | `BOOLEAN` |
| `PHYSICS_POINT_CIRCLE(px,py, cx,cy,cr)` | Punkt in Kreis | `BOOLEAN` |
| `PHYSICS_DISTANCE(x1,y1, x2,y2)` | Euklidische Distanz | `FLOAT` |
| `PHYSICS_DISTANCE2(x1,y1, x2,y2)` | Quadrat-Distanz (kein sqrt) | `FLOAT` |
| `PHYSICS_LENGTH(vx, vy)` | Vektor-Länge | `FLOAT` |
| `PHYSICS_NORM_X(vx, vy)` | X-Anteil des normalisierten Vektors | `FLOAT` |
| `PHYSICS_NORM_Y(vx, vy)` | Y-Anteil des normalisierten Vektors | `FLOAT` |
| `PHYSICS_REFLECT_X(vx,vy, nx,ny)` | X-Komponente der Reflektion an Normal `n` | `FLOAT` |
| `PHYSICS_REFLECT_Y(vx,vy, nx,ny)` | Y-Komponente der Reflektion an Normal `n` | `FLOAT` |
| `PHYSICS_RAY_BOX(rx,ry, dx,dy, bx,by,bw,bh)` | Strahl-Box-Schnitt, `t ∈ [0..1]` oder `-1` | `FLOAT` |
| `PHYSICS_RAY_CIRCLE(rx,ry, dx,dy, cx,cy,cr)` | Strahl-Kreis-Schnitt, `t ∈ [0..1]` oder `-1` | `FLOAT` |

## Konventionen

- **Boxen** sind `(x, y, w, h)` mit `(x, y)` als linker oberer Ecke.
- **Kreise** sind `(cx, cy, r)` — Center und Radius.
- **Strahlen** sind `(rx, ry, dx, dy)` — Origin und **Richtungsvektor mit Länge**. `dx`/`dy` sind nicht normalisiert; die Länge bestimmt die Maximaldistanz, `t = 1` ist das Strahlen-Ende.
- **Normalen** für `REFLECT_*` müssen **nicht** vorab normalisiert sein — die Funktion macht das intern.
- **Multi-Return** ist in GB nicht möglich, deshalb haben Vektor-Operationen mit X/Y-Output zwei separate Funktionen.

## Beispiel: Ball prallt von einer Wand ab

```basic
IMPORT "physics"

' Ball-Velocity mit horizontaler Wand reflektieren
DIM vx AS FLOAT
DIM vy AS FLOAT
vx = 5.0
vy = -3.0

' Wand-Normal zeigt nach oben (0, 1)
DIM nvx AS FLOAT
DIM nvy AS FLOAT
nvx = PHYSICS_REFLECT_X(vx, vy, 0.0, 1.0)
nvy = PHYSICS_REFLECT_Y(vx, vy, 0.0, 1.0)

PRINT nvx, nvy   ' 5.0  3.0  (X bleibt, Y dreht)
```

## Beispiel: Spieler trifft Gegner-Hitbox?

```basic
IMPORT "physics"

DIM player_x AS INTEGER
DIM player_y AS INTEGER
DIM enemy_x  AS INTEGER
DIM enemy_y  AS INTEGER

' AABB-Test (Player und Enemy beide 32×32)
IF PHYSICS_BOX_BOX(player_x, player_y, 32, 32, enemy_x, enemy_y, 32, 32) THEN
    PRINT "Hit!"
END IF
```

## Beispiel: Schuss-Trajektorie (Ray-Cast)

```basic
IMPORT "physics"

' Schuss von Player (200, 100), Richtung Maus
DIM dx AS FLOAT
DIM dy AS FLOAT
dx = MOUSEX() - 200
dy = MOUSEY() - 100

' Trifft das Schuss-Strahl die Enemy-Box?
DIM t AS FLOAT
t = PHYSICS_RAY_BOX(200, 100, dx, dy, enemy_x, enemy_y, 32, 32)
IF t >= 0.0 THEN
    ' Treffer-Punkt (Strahl-Ende mit t skalieren)
    DIM hit_x AS FLOAT
    DIM hit_y AS FLOAT
    hit_x = 200 + dx * t
    hit_y = 100 + dy * t
    CIRCLE(hit_x, hit_y, 4, RED)
END IF
```

## Performance-Tipps

- **`PHYSICS_DISTANCE2`** ist schneller als `PHYSICS_DISTANCE`, da kein `sqrt` läuft. Wenn du nur „X näher als Y" prüfen willst, vergleiche die quadrierten Werte (`d² < r²` ist äquivalent zu `d < r` bei nicht-negativen Zahlen).
- **`PHYSICS_BOX_BOX`** ist die schnellste Kollision (nur 4 Vergleiche). Ideal als Broad-Phase-Filter vor teureren Tests.
- **`PHYSICS_RAY_BOX`** verwendet den Slab-Algorithmus — robust auch bei `dx=0` oder `dy=0`.

## Komplettes Beispiel

[examples/31_physics.gb](../examples/31_physics.gb) — Ball, der durch eine Box-Welt hüpft, mit Hindernis-Reflektion und einem Maus-Ray-Cast.
