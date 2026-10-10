# Module `m3d` — 3D maths (VEC3/VEC4/QUAT/MAT4)

Complete 3D linear algebra for hierarchical transforms (weapon→hand→arm→body),
skeleton/bone animation (quaternions), gizmos and custom projections. It
complements `vec2` (2D) and `g3d` (3D rendering). The maths is **immutable**
and pure (deterministic); f32 internally (render-native). MAT4 is
**column-major** (OpenGL/raylib) — directly usable for rendering.

```basic
IMPORT "m3d"
```

External types (usable with `DIM`): `VEC3`, `VEC4`, `QUAT`, `MAT4`.

## Angles are in radians

All functions that take an angle (`MAT4_ROTATE_*`, `QUAT_FROM_*`,
`MAT4_PERSPECTIVE`) work in **radians**, not degrees — unlike, for example,
`MODEL_EX(..., winkel_grad, ...)` in the `g3d` module. A quarter turn is
therefore `PI / 2`, not `90`:

```basic
m = MAT4_ROTATE_Y(PI / 2.0)      ' 90 degrees
m = MAT4_ROTATE_Y(RAD(90.0))     ' the same, if you prefer to think in degrees
```

`MAT4_ROTATE_Y(90.0)` is not an error but a rotation by 90 **radians**
(≈ 5157 degrees) — the result then simply looks wrong, without anyone
complaining.

## VEC3

| Function | Returns | Meaning |
|---|---|---|
| `VEC3_NEW(x, y, z)` | VEC3 | vector from three numbers |
| `VEC3_ZERO()` | VEC3 | the zero vector |
| `VEC3_X(v)` / `VEC3_Y(v)` / `VEC3_Z(v)` | FLOAT | read a single component |
| `VEC3_LENGTH(v)` | FLOAT | length of the vector |
| `VEC3_LENGTH_SQ(v)` | FLOAT | squared length — saves the square root when you only compare lengths |
| `VEC3_NORMALIZE(v)` | VEC3 | bring to length 1 (keeping the direction) |
| `VEC3_DOT(a, b)` | FLOAT | dot product — 0 at a right angle, positive for the same direction |
| `VEC3_CROSS(a, b)` | VEC3 | cross product — perpendicular to both, for example for surface normals |
| `VEC3_DISTANCE(a, b)` | FLOAT | distance between two points |
| `VEC3_LERP(a, b, t)` | VEC3 | interpolate linearly between the two (`t` 0..1) |
| `VEC3_NEG(v)` | VEC3 | opposite direction |
| `VEC3_SCALE(v, s)` | VEC3 | multiply by a number |
| `VEC3_REFLECT(v, n)` | VEC3 | reflect off a surface with normal `n` — bouncing |
| `VEC3_TRANSFORM(v, mat)` | VEC3 | send through the matrix as a **point** (w=1, translation applies) |
| `VEC3_TRANSFORM_DIR(v, mat)` | VEC3 | as a **direction** (w=0, translation is ignored) |

Operators: `a + b`, `a - b` (VEC3), `v * s` / `s * v` / `v / s` (scalar),
`=` / `<>`.

## VEC4

| Function | Returns | Meaning |
|---|---|---|
| `VEC4_NEW(x, y, z, w)` | VEC4 | vector from four numbers |
| `VEC4_FROM_VEC3(v, w)` | VEC4 | extend a VEC3 by a fourth component (`w=1` point, `w=0` direction) |
| `VEC4_X(v)` / `VEC4_Y(v)` / `VEC4_Z(v)` / `VEC4_W(v)` | FLOAT | read a single component |
| `VEC4_DOT(a, b)` | FLOAT | dot product over all four components |
| `VEC4_LENGTH(v)` | FLOAT | length of the vector |
| `VEC4_NORMALIZE(v)` | VEC4 | bring to length 1 |
| `VEC4_LERP(a, b, t)` | VEC4 | interpolate linearly (`t` 0..1) |

Operators as for VEC3 (`+ -`, scalar `* /`, `=` `<>`).

## QUAT

Rotation without gimbal lock.

| Function | Returns | Meaning |
|---|---|---|
| `QUAT_IDENTITY()` | QUAT | no rotation |
| `QUAT_NEW(x, y, z, w)` | QUAT | from the four components — rarely needed, usually you use `QUAT_FROM_*` |
| `QUAT_X(q)` / `QUAT_Y(q)` / `QUAT_Z(q)` / `QUAT_W(q)` | FLOAT | read a single component |
| `QUAT_FROM_AXIS_ANGLE(ax, ay, az, winkel)` | QUAT | rotation around an axis (angle in radians) |
| `QUAT_FROM_EULER(pitch, yaw, roll)` | QUAT | rotation from the three Euler angles (radians) |
| `QUAT_MUL(a, b)` | QUAT | two rotations one after the other — first `b`, then `a` |
| `QUAT_NORMALIZE(q)` | QUAT | bring to length 1; sensible after many concatenations, otherwise scaling creeps in |
| `QUAT_CONJUGATE(q)` | QUAT | the reverse rotation (at unit length = the inverse) |
| `QUAT_SLERP(a, b, t)` | QUAT | blend smoothly between two rotations (`t` 0..1) — what Euler angles cannot do |
| `QUAT_TO_MAT4(q)` | MAT4 | as a matrix, for example for `MODEL_MATRIX` |
| `QUAT_ROTATE_VEC3(q, v)` | VEC3 | rotate a vector without the detour through a matrix |

Operator: `a * b` = composition (first b, then a — like matrix multiplication).

## MAT4 (column-major)

| Function | Returns | Meaning |
|---|---|---|
| `MAT4_IDENTITY()` | MAT4 | changes nothing — the starting point of every chain |
| `MAT4_TRANSLATE(x, y, z)` | MAT4 | translate |
| `MAT4_SCALE(x, y, z)` | MAT4 | scale (per axis) |
| `MAT4_ROTATE_X(winkel)` / `MAT4_ROTATE_Y(winkel)` / `MAT4_ROTATE_Z(winkel)` | MAT4 | rotate around a principal axis (radians) |
| `MAT4_ROTATE_AXIS(ax, ay, az, winkel)` | MAT4 | rotate around an arbitrary axis (radians) |
| `MAT4_FROM_QUAT(q)` | MAT4 | rotation from a quaternion |
| `MAT4_TRS(pos, rot, scale)` | MAT4 | translation, rotation and scaling in one (= T·R·S) — the usual way to a model matrix |
| `MAT4_MUL(a, b)` | MAT4 | concatenate matrices — first `b`, then `a` |
| `MAT4_INVERT(m)` | MAT4 | inverse; **throws at determinant 0** (for example after `MAT4_SCALE(0, …)`) |
| `MAT4_TRANSPOSE(m)` | MAT4 | swap rows and columns |
| `MAT4_LOOKAT(eye, target, up)` | MAT4 | view matrix: from `eye` towards `target`, `up` says where up is |
| `MAT4_PERSPECTIVE(fovy, aspect, near, far)` | MAT4 | perspective projection (`fovy` in radians) |
| `MAT4_ORTHO(left, right, bottom, top, near, far)` | MAT4 | parallel projection — for blueprints, maps, 2D-in-3D |
| `MAT4_GET(m, row, col)` | FLOAT | read a single element, `row`/`col` each 0..3 |
| `MAT4_TRANSFORM_VEC3(m, v)` | VEC3 | send a point through the matrix |
| `MAT4_TRANSFORM_VEC4(m, v)` | VEC4 | send a four-component vector through the matrix (`w` is preserved) |

Operators: `m1 * m2` (matrix product), `m * v4` → VEC4, `m * v3` → VEC3
(point transform), `=` / `<>`.

> **Order:** `A * B` applies first `B`, then `A`. For a chain
> world = parent·local you write `MAT4_MUL(eltern, lokal)`.

## Rendering: MODEL_MATRIX (core graphics built-in)

`MODEL_MATRIX(handle, mat [, tint])` draws a `g3d` MODEL (from `MESH_*` /
`LOADMODEL`) with an arbitrary world matrix. This makes hierarchical
transforms, instances with their own pose and gizmos possible:

```basic
IMPORT "g3d"
IMPORT "m3d"
DIM box AS INTEGER
box = MESH_CUBE(1, 1, 1)
MODEL_LIT(box)
' Body -> arm -> hand: every level hangs on the parent matrix.
DIM body AS MAT4
body = MAT4_TRS(VEC3_NEW(0,0,0), QUAT_FROM_AXIS_ANGLE(0,1,0, t), VEC3_NEW(1,1.5,1))
MODEL_MATRIX(box, body, &HE08040)
DIM arm AS MAT4
arm = MAT4_MUL(body, MAT4_MUL(MAT4_TRANSLATE(1.2, 0.6, 0), MAT4_SCALE(1.2,0.3,0.3)))
MODEL_MATRIX(box, arm, &H50C0FF)
```

Lighting and shadows work as with `MODEL`: a model with `MODEL_LIT` casts its
shadow under `SHADOW_ENABLE` and receives others (until 2026.27 it cast none --
rotated physics bodies stood on the ground without a shadow).

Many instances = many `MODEL_MATRIX` calls (one draw per call). For very
many identical meshes `MODEL_INSTANCED` is the fast variant (see below).

## GPU instancing: MODEL_INSTANCED (core graphics built-in)

`MODEL_INSTANCED(handle, mats [, tint])` renders **the same mesh with N
world matrices in ONE draw call** (raylib `DrawMeshInstanced`) — instead of N
individual `MODEL_MATRIX` calls. `mats` is an `ARRAY OF MAT4` (or a
`TUPLE` of `MAT4`). Ideal for swarms, particle cubes, vegetation, voxel
fields — orders of magnitude faster than one `MODEL_MATRIX` per instance.

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
MODEL_INSTANCED(box, mats, &H50C0FF)   ' 1024 cubes, 1 draw call
```

The instancing path uses its own lean shader (ambient + up to 4 `LIGHT_*`
lights in Blinn-Phong; without an active light, flat albedo = `tint`). The
per-instance world transform comes as the vertex attribute
`instanceTransform` (not as the `matModel` uniform as with `MODEL_MATRIX`),
the normals are derived from it (correct for rotation + uniform scaling).

**Limit:** The instancing shader supports **no** PBR/IBL
(`MODEL_PBR`/`LIGHT_ENV*`), **no** shadows (`SHADOW_*`) and **no**
normal maps (`MODEL_TEXTURE_NORMAL`) — use `MODEL_MATRIX`/`MODEL_LIT` for
those. `MODEL_TEXTURE` (diffuse map) works, since it sets the material's
albedo texture.

## Custom camera: CAMERA3D_VIEW / CAMERA3D_PROJECTION

| Function | Meaning |
|---|---|
| `CAMERA3D_VIEW(mat)` | set your own view matrix instead of the one built from `CAMERA3D(...)` |
| `CAMERA3D_PROJECTION(mat)` | set your own projection matrix — for example parallel instead of perspective |

`CAMERA3D_VIEW(mat)` and `CAMERA3D_PROJECTION(mat)` override the view and
projection matrices built from `CAMERA3D(...)` — for ortho views, a custom
frustum or shadow tricks. `CAMERA3D(...)` resets both overrides to the
standard perspective.

```basic
CAMERA3D(5, 5, 5, 0, 0, 0, 45)                                  ' Reset + Position
CAMERA3D_PROJECTION(MAT4_ORTHO(-4, 4, -3, 3, 0.1, 100))         ' ortho instead of perspective
CAMERA3D_VIEW(MAT4_LOOKAT(VEC3_NEW(5,5,5), VEC3_ZERO(), VEC3_NEW(0,1,0)))
```

## Notes

- f32 internally → small rounding deviations for non-exact values (use
  `ROUND(...)` for comparisons).
- `MODEL_MATRIX` / `MODEL_INSTANCED` / `CAMERA3D_VIEW` / `CAMERA3D_PROJECTION`
  are **native-only** (dhrt / F6) — they need the raylib 3D pipeline.

Demos: [examples/103_m3d.dh](../../examples/103_m3d.dh) (MODEL_MATRIX),
[examples/104_instancing.dh](../../examples/104_instancing.dh) (MODEL_INSTANCED).
Tests: [tests/pruef/m3d.dhtest](../../tests/pruef/m3d.dhtest).
