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
| `PHYSICS_POINT_TRI(px,py, ax,ay, bx,by, cx,cy)` | Punkt im Dreieck (baryzentrisch — der Umlaufsinn der Ecken ist egal) | `BOOLEAN` |
| `PHYSICS_LINES_HIT(ax,ay,bx,by, cx,cy,dx,dy)` | schneiden sich zwei **Strecken**? (nicht die unendlichen Geraden) | `BOOLEAN` |
| `PHYSICS_LINES_X(ax,ay,bx,by, cx,cy,dx,dy)` | X des Schnittpunkts — **`NAN`, wenn es keinen gibt**, also erst `PHYSICS_LINES_HIT` fragen | `FLOAT` |
| `PHYSICS_LINES_Y(ax,ay,bx,by, cx,cy,dx,dy)` | Y des Schnittpunkts (ebenso `NAN`) | `FLOAT` |
| `PHYSICS_POINT_LINE(px,py, ax,ay, bx,by, dicke)` | liegt der Punkt auf der Strecke? `dicke` gibt den erlaubten Abstand — für Klicks auf dünne Linien | `BOOLEAN` |
| `PHYSICS_CIRCLE_LINE(cx,cy,r, ax,ay, bx,by)` | berührt ein Kreis die Strecke? | `BOOLEAN` |
| `PHYSICS_POINT_POLY(px, py, xs, ys)` | Punkt im Vieleck (Strahl-Verfahren — funktioniert auch bei konkaven Formen) | `BOOLEAN` |
| `PHYSICS_DISTANCE3(x1,y1,z1, x2,y2,z2)` | Abstand zweier Punkte im Raum | `FLOAT` |
| `PHYSICS_SPHERE_SPHERE(x1,y1,z1,r1, x2,y2,z2,r2)` | berühren sich zwei Kugeln? — die Kugel-Näherung reicht für die meisten 3D-Treffer | `BOOLEAN` |
| `PHYSICS_BROAD_NEW()` | Broadphase anlegen | `PHYSICS_BROAD` |
| `PHYSICS_BROAD_CLEAR(b)` | alle Einträge verwerfen — pro Frame vor dem neuen Befüllen | — |
| `PHYSICS_BROAD_ADD(b, x, y, r)` | einen Kreis aufnehmen; liefert seinen Index | `INTEGER` |
| `PHYSICS_BROAD_COUNT(b)` | wie viele Kreise sind drin? | `INTEGER` |
| `PHYSICS_BROAD_QUERY(b)` | alle sich überlappenden Paare suchen; liefert deren Anzahl | `INTEGER` |
| `PHYSICS_BROAD_PAIR_COUNT(b)` | Anzahl der gefundenen Paare (wie der Rückgabewert von `QUERY`) | `INTEGER` |
| `PHYSICS_BROAD_PAIR_A(b, i)` | erster Kreis des Paars `i` (Index aus `ADD`) | `INTEGER` |
| `PHYSICS_BROAD_PAIR_B(b, i)` | zweiter Kreis des Paars `i` | `INTEGER` |

### Broadphase: viele Kreise auf einmal

Wer 500 Kugeln gegeneinander prüft, macht mit zwei verschachtelten Schleifen
125 000 Vergleiche pro Frame. Die Broadphase legt die Kreise in ein Gitter und
vergleicht nur Nachbarn — das kostet mit der Anzahl linear statt quadratisch.
Der Ablauf pro Frame:

```basic
IMPORT "physics"
PHYSICS_BROAD_CLEAR(b)                      ' 1. leeren
FOR i = 0 TO n - 1
    idx = PHYSICS_BROAD_ADD(b, x[i], y[i], r[i])   ' 2. befuellen
NEXT
paare = PHYSICS_BROAD_QUERY(b)              ' 3. suchen
FOR p = 0 TO paare - 1
    a = PHYSICS_BROAD_PAIR_A(b, p)          ' 4. Paare abholen
    c = PHYSICS_BROAD_PAIR_B(b, p)
NEXT
```

Die zurückgegebenen Zahlen sind die Indizes aus `PHYSICS_BROAD_ADD` — in der
Reihenfolge, in der man die Kreise hineingelegt hat. Auf die Reihenfolge der
**Paare** sollte man sich nicht verlassen.

## Konventionen

- **Boxen** sind `(x, y, w, h)` mit `(x, y)` als linker oberer Ecke.
- **Kreise** sind `(cx, cy, r)` — Center und Radius.
- **Strahlen** sind `(rx, ry, dx, dy)` — Origin und **Richtungsvektor mit Länge**. `dx`/`dy` sind nicht normalisiert; die Länge bestimmt die Maximaldistanz, `t = 1` ist das Strahlen-Ende.
- **Normalen** für `REFLECT_*` müssen **nicht** vorab normalisiert sein — die Funktion macht das intern.
- **Vektor-Operationen mit X/Y-Ausgabe haben zwei separate Funktionen** (`PHYSICS_NORM_X`/`PHYSICS_NORM_Y`). Das ist eine Entwurfsentscheidung, keine Not: Drachenhauch kann Mehrfach-Rückgabe sehr wohl, über [`BYREF`](sprache.md#byref-parameter-multi-return). Zwei pure Funktionen lassen sich aber direkt in einen Ausdruck schreiben, während `BYREF` zwei vorher deklarierte Variablen braucht.

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

[examples/142_physics.dh](../examples/142_physics.dh) — Ball, der durch eine Box-Welt hüpft, mit Hindernis-Reflektion und einem Maus-Ray-Cast.
