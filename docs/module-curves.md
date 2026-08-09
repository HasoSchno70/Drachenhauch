# Modul `curves`

Animation-Kurven: Bezier, Catmull-Rom, Hermite, Smoothstep, Lerp. Pure Functions — keine State, kein Setup. Komplementaer zum [`tween`](module-tween.md)-Modul, das Werte ueber Zeit animiert; `curves` rechnet einzelne Kurven-Punkte aus.

```basic
IMPORT "curves"
```

## Übersicht

| Funktion | Rueckgabe | Zweck |
|---|---|---|
| `CURVE_LERP(a, b, t)` | FLOAT | Lineare Interpolation `a + (b - a) * t` |
| `CURVE_SMOOTHSTEP(e0, e1, x)` | FLOAT | S-Kurve (`3t² - 2t³`), geclamped auf `[0, 1]` |
| `CURVE_SMOOTHERSTEP(e0, e1, x)` | FLOAT | C²-stetige S-Kurve (`6t⁵ - 15t⁴ + 10t³`) |
| `CURVE_BEZIER(t, p0, p1, p2, p3)` | FLOAT | Cubic Bezier 1D |
| `CURVE_BEZIER2(t, x0,y0, x1,y1, x2,y2, x3,y3)` | TUPLE (x, y) | Cubic Bezier 2D |
| `CURVE_CATMULL(t, p0, p1, p2, p3)` | FLOAT | Catmull-Rom 1D |
| `CURVE_CATMULL2(t, x0,y0, x1,y1, x2,y2, x3,y3)` | TUPLE (x, y) | Catmull-Rom 2D |
| `CURVE_HERMITE(t, p0, p1, m0, m1)` | FLOAT | Cubic Hermite mit Tangenten |

## Konzept

Alle Kurven nehmen einen Parameter `t` (typisch `0..1`) und liefern einen interpolierten Wert. Die Kurven-Typen unterscheiden sich darin, **wie** die Control-Points den Verlauf bestimmen:

- **Lerp:** zwei Punkte, gerade Linie. Trivial.
- **Smoothstep / Smootherstep:** zwei Punkte, weiche Beschleunigung am Anfang/Ende ("Ease-In-Out").
- **Bezier:** vier Punkte. P0/P3 sind Start/Ende, P1/P2 sind **Tangenten-Handles** (Kurve laeuft ihnen entgegen aber durch sie nicht hindurch).
- **Catmull-Rom:** vier Punkte. Kurve laeuft **durch** P1 und P2; P0 und P3 sind Tangenten-Stuetzen (Vorgaenger/Nachfolger).
- **Hermite:** zwei Punkte + zwei Tangenten-Vektoren. Maximale Kontrolle.

## Lerp und Smoothstep

Die einfachsten Bausteine fuer Animationen:

```basic
DIM t AS FLOAT
t = 0.3                       ' 30% Fortschritt
PRINT CURVE_LERP(100.0, 200.0, t)       ' 130.0  (linear)
PRINT CURVE_SMOOTHSTEP(0.0, 1.0, t)     ' ~0.216 (S-Kurve)
PRINT CURVE_SMOOTHERSTEP(0.0, 1.0, t)   ' ~0.163 (sanfter)
```

`CURVE_SMOOTHSTEP(e0, e1, x)` normalisiert `x` auf den Bereich `[e0, e1]` (geclamped), dann S-Kurve. Klassisch fuer Fade-In/Out:

```basic
DIM fade AS FLOAT
fade = CURVE_SMOOTHSTEP(0.0, 1.0, t)   ' t in [0..1] -> Alpha
```

## Bezier — visuell-intuitiv

Vier Punkte. P0 ist der Start, P3 ist das Ende. P1 und P2 ziehen die Kurve in ihre Richtung, aber sie laeuft nicht durch sie.

```basic
' Sprung-Trajektorie: Start, hoher Punkt, dann fallen, Landung
DIM pos AS TUPLE
DIM t AS FLOAT
FOR t = 0.0 TO 1.0 STEP 0.02
    pos = CURVE_BEZIER2(t,   0.0, 200.0,
                              100.0,  50.0,
                              200.0,  50.0,
                              300.0, 200.0)
    DRAWIMAGE(player, pos[0], pos[1])
NEXT
```

P0=(0,200) Start unten links, P1=(100,50) Tangenten-Handle oben, P2=(200,50) Tangenten-Handle oben, P3=(300,200) Ende unten rechts. Resultat: weiche Sprung-Trajektorie.

## Catmull-Rom — durch die Punkte

Im Gegensatz zu Bezier laeuft Catmull-Rom **durch** P1 und P2. P0 und P3 bestimmen die Tangenten an P1 (aus P2-P0) und P2 (aus P3-P1).

Praktisch fuer **Splines durch Waypoints**:

```basic
' 4 Waypoints einer Patroullie. Interpoliere zwischen wp1 und wp2,
' wp0 und wp3 sind die Nachbarn (fuer weichen Verlauf).
DIM pos AS TUPLE
pos = CURVE_CATMULL2(t,
                     wp0_x, wp0_y,
                     wp1_x, wp1_y,
                     wp2_x, wp2_y,
                     wp3_x, wp3_y)
```

Fuer einen Pfad aus N Waypoints iteriert man durch die Segmente: zwischen i=1 und i=2, dann i=2 und i=3, usw. — jedes Segment nutzt 4 aufeinanderfolgende Waypoints.

## Hermite — Tangenten-explizit

`CURVE_HERMITE(t, p0, p1, m0, m1)` interpoliert von `p0` zu `p1` mit expliziten Tangenten-Vektoren `m0` (am Start) und `m1` (am Ende). Wenn du die Geschwindigkeit am Anfang und Ende vorgeben willst:

```basic
' Slow-In, Slow-Out:
PRINT CURVE_HERMITE(t, 0.0, 100.0, 0.0, 0.0)
' Start mit 0-Tangente, Ende mit 0-Tangente -> wie SmoothStep

' Fast-Start, Slow-Out:
PRINT CURVE_HERMITE(t, 0.0, 100.0, 300.0, 0.0)
```

## `curves` vs `tween`

Beide animieren Werte, aber unterschiedlich:

| Aspekt | `tween` | `curves` |
|---|---|---|
| Modell | "Animiere von a nach b in N ms" | "Was ist der Wert bei t = 0.3?" |
| State | TWEEN-Handle, Update pro Frame | keine State, nur Funktion |
| Verwendung | langer Anim-Lebenszyklus | Pfad-Sampling, prozedural |
| Einfache Lerp | `TWEEN_NEW(0, 100, 1000, "linear")` | `CURVE_LERP(0, 100, t)` |

Fuer **Bewegung ueber Zeit**: `tween`. Fuer **Punkt-auf-Kurve-bestimmen**: `curves`. Beide koennen kombiniert werden: ein `tween` animiert `t` von 0 nach 1 ueber 2 Sekunden, dann ist `CURVE_BEZIER2(t, ...)` die Position.

## Game-Patterns

**Camera-Smoothing zu Target:**

```basic
cam_x = CURVE_LERP(cam_x, target_x, 0.1)   ' jeden Frame 10% naeher
cam_y = CURVE_LERP(cam_y, target_y, 0.1)
```

**Health-Bar-Animation:**

```basic
DIM display_hp AS FLOAT
display_hp = CURVE_LERP(display_hp, actual_hp, 0.15)   ' weich folgen
DRAW_BAR(display_hp)
```

**Cutscene-Kamerafahrt via Bezier:**

```basic
' t laeuft von 0 nach 1 ueber 3 Sekunden
DIM t AS FLOAT
t = (MILLIS() - cutscene_start) / 3000.0
IF t > 1.0 THEN t = 1.0

DIM pos AS TUPLE
pos = CURVE_BEZIER2(t, start_x, start_y, ctrl1_x, ctrl1_y,
                       ctrl2_x, ctrl2_y, end_x, end_y)
CAMERA_SET(pos[0], pos[1])
```

## Beispiel

[examples/74_curves_path.dh](../examples/74_curves_path.dh) zeigt die Kurven-Typen visuell — eine Animation laeuft entlang verschiedener Spline-Typen.
