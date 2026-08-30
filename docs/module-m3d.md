# Modul `m3d` — 3D-Mathe (VEC3/VEC4/QUAT/MAT4)

Vollständige 3D-Linalg für hierarchische Transforms (Waffe→Hand→Arm→Körper),
Skelett-/Bone-Animation (Quaternionen), Gizmos und Custom-Projektionen. Ergänzt
`vec2` (2D) und `g3d` (3D-Rendering). Die Mathe ist **immutable** und pure
(deterministisch); intern f32 (render-nativ). MAT4 ist **column-major**
(OpenGL/raylib) — direkt rendertauglich.

```basic
IMPORT "m3d"
```

Externe Typen (mit `DIM` nutzbar): `VEC3`, `VEC4`, `QUAT`, `MAT4`.

## Winkel sind im Bogenmaß

Alle Funktionen, die einen Winkel nehmen (`MAT4_ROTATE_*`, `QUAT_FROM_*`,
`MAT4_PERSPECTIVE`), rechnen im **Bogenmaß**, nicht in Grad — anders als etwa
`MODEL_EX(..., winkel_grad, ...)` im `g3d`-Modul. Eine Vierteldrehung ist also
`PI / 2`, nicht `90`:

```basic
m = MAT4_ROTATE_Y(PI / 2.0)      ' 90 Grad
m = MAT4_ROTATE_Y(RAD(90.0))     ' dasselbe, wenn man lieber in Grad denkt
```

`MAT4_ROTATE_Y(90.0)` ist kein Fehler, sondern eine Drehung um 90 **Radiant**
(≈ 5157 Grad) — das Ergebnis sieht dann einfach falsch aus, ohne dass sich
jemand beschwert.

## VEC3

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `VEC3_NEW(x, y, z)` | VEC3 | Vektor aus drei Zahlen |
| `VEC3_ZERO()` | VEC3 | der Nullvektor |
| `VEC3_X(v)` / `VEC3_Y(v)` / `VEC3_Z(v)` | FLOAT | einzelne Komponente lesen |
| `VEC3_LENGTH(v)` | FLOAT | Länge des Vektors |
| `VEC3_LENGTH_SQ(v)` | FLOAT | Länge zum Quadrat — spart die Wurzel, wenn man nur Längen vergleicht |
| `VEC3_NORMALIZE(v)` | VEC3 | auf Länge 1 bringen (Richtung behalten) |
| `VEC3_DOT(a, b)` | FLOAT | Skalarprodukt — 0 bei rechtem Winkel, positiv bei gleicher Richtung |
| `VEC3_CROSS(a, b)` | VEC3 | Kreuzprodukt — steht senkrecht auf beiden, etwa für Flächennormalen |
| `VEC3_DISTANCE(a, b)` | FLOAT | Abstand zweier Punkte |
| `VEC3_LERP(a, b, t)` | VEC3 | geradlinig zwischen beiden interpolieren (`t` 0..1) |
| `VEC3_NEG(v)` | VEC3 | Gegenrichtung |
| `VEC3_SCALE(v, s)` | VEC3 | mit einer Zahl multiplizieren |
| `VEC3_REFLECT(v, n)` | VEC3 | an einer Fläche mit Normale `n` spiegeln — Abprallen |
| `VEC3_TRANSFORM(v, mat)` | VEC3 | als **Punkt** durch die Matrix schicken (w=1, Verschiebung wirkt) |
| `VEC3_TRANSFORM_DIR(v, mat)` | VEC3 | als **Richtung** (w=0, Verschiebung wird ignoriert) |

Operatoren: `a + b`, `a - b` (VEC3), `v * s` / `s * v` / `v / s` (Skalar),
`=` / `<>`.

## VEC4

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `VEC4_NEW(x, y, z, w)` | VEC4 | Vektor aus vier Zahlen |
| `VEC4_FROM_VEC3(v, w)` | VEC4 | VEC3 um eine vierte Komponente ergänzen (`w=1` Punkt, `w=0` Richtung) |
| `VEC4_X(v)` / `VEC4_Y(v)` / `VEC4_Z(v)` / `VEC4_W(v)` | FLOAT | einzelne Komponente lesen |
| `VEC4_DOT(a, b)` | FLOAT | Skalarprodukt über alle vier Komponenten |
| `VEC4_LENGTH(v)` | FLOAT | Länge des Vektors |
| `VEC4_NORMALIZE(v)` | VEC4 | auf Länge 1 bringen |
| `VEC4_LERP(a, b, t)` | VEC4 | geradlinig interpolieren (`t` 0..1) |

Operatoren wie VEC3 (`+ -`, Skalar `* /`, `=` `<>`).

## QUAT

Rotation ohne Gimbal-Lock.

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `QUAT_IDENTITY()` | QUAT | keine Drehung |
| `QUAT_NEW(x, y, z, w)` | QUAT | aus den vier Komponenten — selten nötig, meist nimmt man `QUAT_FROM_*` |
| `QUAT_X(q)` / `QUAT_Y(q)` / `QUAT_Z(q)` / `QUAT_W(q)` | FLOAT | einzelne Komponente lesen |
| `QUAT_FROM_AXIS_ANGLE(ax, ay, az, winkel)` | QUAT | Drehung um eine Achse (Winkel im Bogenmaß) |
| `QUAT_FROM_EULER(pitch, yaw, roll)` | QUAT | Drehung aus den drei Eulerwinkeln (Bogenmaß) |
| `QUAT_MUL(a, b)` | QUAT | zwei Drehungen hintereinander — erst `b`, dann `a` |
| `QUAT_NORMALIZE(q)` | QUAT | auf Länge 1 bringen; nach vielen Verkettungen sinnvoll, sonst schleicht sich Skalierung ein |
| `QUAT_CONJUGATE(q)` | QUAT | die umgekehrte Drehung (bei Einheitslänge = die Inverse) |
| `QUAT_SLERP(a, b, t)` | QUAT | weich zwischen zwei Drehungen überblenden (`t` 0..1) — das, was Euler nicht kann |
| `QUAT_TO_MAT4(q)` | MAT4 | als Matrix, etwa für `MODEL_MATRIX` |
| `QUAT_ROTATE_VEC3(q, v)` | VEC3 | einen Vektor drehen, ohne den Umweg über eine Matrix |

Operator: `a * b` = Komposition (erst b, dann a — wie Matrix-Multiplikation).

## MAT4 (column-major)

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `MAT4_IDENTITY()` | MAT4 | verändert nichts — Ausgangspunkt jeder Kette |
| `MAT4_TRANSLATE(x, y, z)` | MAT4 | verschieben |
| `MAT4_SCALE(x, y, z)` | MAT4 | skalieren (je Achse) |
| `MAT4_ROTATE_X(winkel)` / `MAT4_ROTATE_Y(winkel)` / `MAT4_ROTATE_Z(winkel)` | MAT4 | um eine Hauptachse drehen (Bogenmaß) |
| `MAT4_ROTATE_AXIS(ax, ay, az, winkel)` | MAT4 | um eine beliebige Achse drehen (Bogenmaß) |
| `MAT4_FROM_QUAT(q)` | MAT4 | Drehung aus einem Quaternion |
| `MAT4_TRS(pos, rot, scale)` | MAT4 | Verschiebung, Drehung und Skalierung in einem (= T·R·S) — der übliche Weg zu einer Modell-Matrix |
| `MAT4_MUL(a, b)` | MAT4 | Matrizen verketten — erst `b`, dann `a` |
| `MAT4_INVERT(m)` | MAT4 | Umkehrung; **wirft bei Determinante 0** (etwa nach `MAT4_SCALE(0, …)`) |
| `MAT4_TRANSPOSE(m)` | MAT4 | Zeilen und Spalten tauschen |
| `MAT4_LOOKAT(eye, target, up)` | MAT4 | Blickmatrix: von `eye` auf `target`, `up` gibt oben an |
| `MAT4_PERSPECTIVE(fovy, aspect, near, far)` | MAT4 | perspektivische Projektion (`fovy` im Bogenmaß) |
| `MAT4_ORTHO(left, right, bottom, top, near, far)` | MAT4 | parallele Projektion — für Baupläne, Karten, 2D-in-3D |
| `MAT4_GET(m, row, col)` | FLOAT | einzelnes Element lesen, `row`/`col` je 0..3 |
| `MAT4_TRANSFORM_VEC3(m, v)` | VEC3 | Punkt durch die Matrix schicken |
| `MAT4_TRANSFORM_VEC4(m, v)` | VEC4 | Vierervektor durch die Matrix schicken (`w` bleibt erhalten) |

Operatoren: `m1 * m2` (Matrix-Produkt), `m * v4` → VEC4, `m * v3` → VEC3
(Punkt-Transform), `=` / `<>`.

> **Reihenfolge:** `A * B` wendet erst `B`, dann `A` an. Für eine Kette
> Welt = Eltern·Lokal schreibt man `MAT4_MUL(eltern, lokal)`.

## Rendering: MODEL_MATRIX (Core-Graphics-Builtin)

`MODEL_MATRIX(handle, mat [, tint])` zeichnet ein `g3d`-MODEL (aus `MESH_*` /
`LOADMODEL`) mit einer beliebigen Welt-Matrix. Damit werden hierarchische
Transforms, Instanzen mit eigener Pose und Gizmos möglich:

```basic
IMPORT "g3d"
IMPORT "m3d"
DIM box AS INTEGER
box = MESH_CUBE(1, 1, 1)
MODEL_LIT(box)
' Körper -> Arm -> Hand: jede Ebene haengt an der Eltern-Matrix.
DIM body AS MAT4
body = MAT4_TRS(VEC3_NEW(0,0,0), QUAT_FROM_AXIS_ANGLE(0,1,0, t), VEC3_NEW(1,1.5,1))
MODEL_MATRIX(box, body, &HE08040)
DIM arm AS MAT4
arm = MAT4_MUL(body, MAT4_MUL(MAT4_TRANSLATE(1.2, 0.6, 0), MAT4_SCALE(1.2,0.3,0.3)))
MODEL_MATRIX(box, arm, &H50C0FF)
```

Viele Instanzen = viele `MODEL_MATRIX`-Aufrufe (ein Draw pro Aufruf). Für sehr
viele identische Meshes ist `MODEL_INSTANCED` die performante Variante (siehe
unten).

## GPU-Instancing: MODEL_INSTANCED (Core-Graphics-Builtin)

`MODEL_INSTANCED(handle, mats [, tint])` rendert **dasselbe Mesh mit N
Welt-Matrizen in EINEM Draw-Call** (raylib `DrawMeshInstanced`) — statt N
einzelner `MODEL_MATRIX`-Aufrufe. `mats` ist ein `ARRAY OF MAT4` (oder ein
`TUPLE` von `MAT4`). Ideal für Schwärme, Partikel-Würfel, Vegetation, Voxel-
Felder — Größenordnungen schneller als ein `MODEL_MATRIX` pro Instanz.

```basic
IMPORT "g3d"
IMPORT "m3d"
DIM box AS INTEGER
box = MESH_CUBE(0.7, 0.7, 0.7)

DIM mats[1024] AS MAT4
DIM i AS INTEGER
FOR i = 0 TO 1023
    mats[i] = MAT4_TRANSLATE((i MOD 32) * 1.4, 0, (i \ 32) * 1.4)
NEXT
MODEL_INSTANCED(box, mats, &H50C0FF)   ' 1024 Wuerfel, 1 Draw-Call
```

Der Instancing-Pfad nutzt einen eigenen, schlanken Shader (Ambient + bis zu 4
`LIGHT_*`-Lichter in Blinn-Phong; ohne aktives Licht flaches Albedo = `tint`).
Die per-Instanz-Welt-Transform kommt als Vertex-Attribut `instanceTransform`
(nicht als `matModel`-Uniform wie bei `MODEL_MATRIX`), die Normalen werden daraus
abgeleitet (korrekt für Rotation + uniforme Skalierung).

**Grenze:** Der Instancing-Shader unterstützt **kein** PBR/IBL
(`MODEL_PBR`/`LIGHT_ENV*`), **keine** Schatten (`SHADOW_*`) und **keine**
Normal-Maps (`MODEL_TEXTURE_NORMAL`) — dafür `MODEL_MATRIX`/`MODEL_LIT` nutzen.
`MODEL_TEXTURE` (Diffuse-Map) wirkt, da es die Albedo-Textur des Materials setzt.

## Custom-Kamera: CAMERA3D_VIEW / CAMERA3D_PROJECTION

| Funktion | Bedeutung |
|---|---|
| `CAMERA3D_VIEW(mat)` | eigene Blickmatrix setzen statt der aus `CAMERA3D(...)` gebauten |
| `CAMERA3D_PROJECTION(mat)` | eigene Projektionsmatrix setzen — etwa parallel statt perspektivisch |

`CAMERA3D_VIEW(mat)` und `CAMERA3D_PROJECTION(mat)` überschreiben die aus
`CAMERA3D(...)` gebauten View-/Projektions-Matrizen — für Ortho-Ansichten,
Custom-Frustum oder Shadow-Tricks. `CAMERA3D(...)` setzt beide Overrides zurück
auf die Standard-Perspektive.

```basic
CAMERA3D(5, 5, 5, 0, 0, 0, 45)                                  ' Reset + Position
CAMERA3D_PROJECTION(MAT4_ORTHO(-4, 4, -3, 3, 0.1, 100))         ' Ortho statt Perspektive
CAMERA3D_VIEW(MAT4_LOOKAT(VEC3_NEW(5,5,5), VEC3_ZERO(), VEC3_NEW(0,1,0)))
```

## Hinweise

- Intern f32 → bei nicht-exakten Werten kleine Rundungsabweichungen (zum
  Vergleichen `ROUND(...)` nutzen).
- `MODEL_MATRIX` / `MODEL_INSTANCED` / `CAMERA3D_VIEW` / `CAMERA3D_PROJECTION`
  sind **native-only** (dhrt / F6) — sie brauchen die raylib-3D-Pipeline.

Demos: [examples/103_m3d.dh](../examples/103_m3d.dh) (MODEL_MATRIX),
[examples/104_instancing.dh](../examples/104_instancing.dh) (MODEL_INSTANCED).
Tests: [tests/test_m3d.py](../tests/test_m3d.py).
