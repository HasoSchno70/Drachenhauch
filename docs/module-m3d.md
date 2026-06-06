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

## VEC3

`VEC3_NEW(x,y,z)`, `VEC3_ZERO()`, `VEC3_X/Y/Z(v)`,
`VEC3_LENGTH(v)`, `VEC3_LENGTH_SQ(v)`, `VEC3_NORMALIZE(v)`,
`VEC3_DOT(a,b)`, `VEC3_CROSS(a,b)`, `VEC3_DISTANCE(a,b)`,
`VEC3_LERP(a,b,t)`, `VEC3_NEG(v)`, `VEC3_SCALE(v,s)`, `VEC3_REFLECT(v,n)`,
`VEC3_TRANSFORM(v,mat)` (als Punkt, w=1), `VEC3_TRANSFORM_DIR(v,mat)` (als
Richtung, w=0 — ignoriert Translation).

Operatoren: `a + b`, `a - b` (VEC3), `v * s` / `s * v` / `v / s` (Skalar),
`=` / `<>`.

## VEC4

`VEC4_NEW(x,y,z,w)`, `VEC4_FROM_VEC3(v,w)`, `VEC4_X/Y/Z/W(v)`,
`VEC4_DOT(a,b)`, `VEC4_LENGTH(v)`, `VEC4_NORMALIZE(v)`, `VEC4_LERP(a,b,t)`.
Operatoren wie VEC3 (`+ -`, Skalar `* /`, `=` `<>`).

## QUAT

Rotation ohne Gimbal-Lock. `QUAT_IDENTITY()`, `QUAT_NEW(x,y,z,w)`,
`QUAT_X/Y/Z/W(q)`, `QUAT_FROM_AXIS_ANGLE(ax,ay,az,winkel)`,
`QUAT_FROM_EULER(pitch,yaw,roll)`, `QUAT_MUL(a,b)`, `QUAT_NORMALIZE(q)`,
`QUAT_CONJUGATE(q)`, `QUAT_SLERP(a,b,t)`, `QUAT_TO_MAT4(q)`,
`QUAT_ROTATE_VEC3(q,v)`.

Operator: `a * b` = Komposition (erst b, dann a — wie Matrix-Multiplikation).

## MAT4 (column-major)

Konstruktoren: `MAT4_IDENTITY()`, `MAT4_TRANSLATE(x,y,z)`, `MAT4_SCALE(x,y,z)`,
`MAT4_ROTATE_X/Y/Z(winkel)`, `MAT4_ROTATE_AXIS(ax,ay,az,winkel)`,
`MAT4_FROM_QUAT(q)`, `MAT4_TRS(pos_v3, rot_quat, scale_v3)` (= T·R·S in einem).

Operationen: `MAT4_MUL(a,b)`, `MAT4_INVERT(m)` (wirft bei Determinante 0),
`MAT4_TRANSPOSE(m)`, `MAT4_LOOKAT(eye,target,up)`,
`MAT4_PERSPECTIVE(fovy,aspect,near,far)`,
`MAT4_ORTHO(left,right,bottom,top,near,far)`, `MAT4_GET(m,row,col)` (0..3),
`MAT4_TRANSFORM_VEC3(m,v)` / `MAT4_TRANSFORM_VEC4(m,v)`.

Operatoren: `m1 * m2` (Matrix-Produkt), `m * v4` → VEC4, `m * v3` → VEC3
(Punkt-Transform), `=` / `<>`.

> **Reihenfolge:** `A * B` wendet erst `B`, dann `A` an. Für eine Kette
> Welt = Eltern·Lokal schreibt man `MAT4_MUL(eltern, lokal)`.

## Rendering: MODEL_MATRIX (Core-Graphics-Builtin)

`MODEL_MATRIX(handle, mat [, tint])` zeichnet ein `g3d`-MODEL (aus `MESH_*` /
`LOADMODEL`) mit einer beliebigen Welt-Matrix. Damit werden hierarchische
Transforms, Instanzen mit eigener Pose und Gizmos möglich:

```basic
IMPORT "g3d" : IMPORT "m3d"
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
IMPORT "g3d" : IMPORT "m3d"
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
  sind **native-only** (gbrt / F6) — sie brauchen die raylib-3D-Pipeline.

Demos: [examples/103_m3d.gb](../examples/103_m3d.gb) (MODEL_MATRIX),
[examples/104_instancing.gb](../examples/104_instancing.gb) (MODEL_INSTANCED).
Tests: [tests/test_m3d.py](../tests/test_m3d.py).
