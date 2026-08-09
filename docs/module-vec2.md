# Modul `vec2`

Immutable 2D-Vektor mit Operator-Overloading. Vec2-Werte sind **unveraenderlich** — jede Operation erzeugt einen neuen Vec2. Das macht Code lesbar und Bug-frei: niemand mutiert versehentlich den Vektor eines anderen.

```basic
IMPORT "vec2"
```

## Übersicht

| Funktion | Rueckgabe | Wirkung |
|---|---|---|
| `VEC2_NEW(x, y)` | VEC2 | Konstruktor |
| `VEC2_ZERO()` | VEC2 | (0, 0) |
| `VEC2_X(v)` / `VEC2_Y(v)` | FLOAT | Komponenten lesen |
| `VEC2_LENGTH(v)` | FLOAT | Euklidische Laenge |
| `VEC2_LENGTH_SQ(v)` | FLOAT | Laenge im Quadrat (spart sqrt) |
| `VEC2_NORMALIZE(v)` | VEC2 | Einheitsvektor (NIL bei Null-Vektor) |
| `VEC2_DOT(a, b)` | FLOAT | Skalarprodukt |
| `VEC2_CROSS(a, b)` | FLOAT | 2D-Kreuzprodukt (Skalar) |
| `VEC2_DISTANCE(a, b)` | FLOAT | Euklidische Distanz |
| `VEC2_LERP(a, b, t)` | VEC2 | Interpolation `a + (b - a) * t` |
| `VEC2_PERP(v)` | VEC2 | 90° gegen den Uhrzeigersinn: `(-y, x)` |
| `VEC2_REFLECT(v, normal)` | VEC2 | Reflektion an einer Wand-Normalen |
| `VEC2_ANGLE(v)` | FLOAT | Winkel in Radiant (von +x-Achse) |
| `VEC2_FROM_ANGLE(angle, length)` | VEC2 | Polar → Kartesisch |

## Operator-Overloading

Die arithmetischen Operatoren funktionieren direkt auf Vec2:

```basic
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(3.0, 4.0)
w = VEC2_NEW(1.0, 2.0)

PRINT v + w            ' Vec2(4.0, 6.0)
PRINT v - w            ' Vec2(2.0, 2.0)
PRINT v * 2.0          ' Vec2(6.0, 8.0)   Skalar-Multiplikation
PRINT 2.0 * v          ' Vec2(6.0, 8.0)   beide Reihenfolgen funktionieren
PRINT v / 2.0          ' Vec2(1.5, 2.0)
PRINT v = w            ' FALSE
PRINT v <> w           ' TRUE
```

**Multiplikation Vec2 * Vec2** ist nicht definiert — fuer Dot-Product nimm `VEC2_DOT(a, b)`, fuer komponentenweise Multiplikation `VEC2_NEW(VEC2_X(a) * VEC2_X(b), VEC2_Y(a) * VEC2_Y(b))`.

## Klassische Game-Patterns

**Player-Bewegung mit Velocity:**

```basic
DIM pos AS VEC2
DIM vel AS VEC2
pos = VEC2_NEW(100.0, 100.0)
vel = VEC2_NEW(0.0, 0.0)

' Pro Frame:
vel = vel * 0.95       ' Friction
pos = pos + vel        ' Movement
```

**Steering: Richtung zu Ziel:**

```basic
DIM target AS VEC2
DIM direction AS VEC2
target = VEC2_NEW(400.0, 300.0)
direction = VEC2_NORMALIZE(target - pos)
DIM speed AS FLOAT
speed = 3.0
vel = direction * speed
```

**Reflektion an einer Wand:**

```basic
DIM wall_normal AS VEC2
wall_normal = VEC2_NEW(0.0, -1.0)    ' Boden, Normale zeigt nach oben
vel = VEC2_REFLECT(vel, wall_normal)
```

**Drehung um 90° (klassisch fuer Turret-Pattern):**

```basic
DIM forward AS VEC2
DIM right AS VEC2
forward = VEC2_FROM_ANGLE(angle, 1.0)
right = VEC2_PERP(forward)       ' 90° gegen den Uhrzeigersinn
```

**Distanz-Check (squared spart sqrt):**

```basic
IF VEC2_LENGTH_SQ(target - pos) < 100.0 THEN     ' < 10 Pixel
    PRINT "Ziel erreicht!"
END IF
```

## Immutability

Vec2 ist **immutable**: nach `w = v` zeigen `w` und `v` zwar auf dasselbe Objekt, aber jede Operation erzeugt einen neuen Vec2. Es gibt also keinen Weg, ueber `w` etwas an `v` zu aendern.

```basic
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(1.0, 2.0)
w = v                       ' "Alias" -- aber irrelevant, weil immutable
v = v + VEC2_NEW(10.0, 0.0) ' v ist jetzt ein NEUES Vec2
PRINT w                     ' Vec2(1.0, 2.0) -- unveraendert
PRINT v                     ' Vec2(11.0, 2.0)
```

## Externer Typ

| Typ | Wirkung |
|---|---|
| `VEC2` | Immutable 2D-Vektor. `DIM v AS VEC2` |

## Beispiel

[examples/58_vec2.dh](../examples/58_vec2.dh) zeigt das volle API inkl. der Operator-Overloads in einer kleinen Demo (Player-Movement, Distanz-Check, Reflektion).
