# Module `g3d` — 3D graphics

Three-dimensional scenes: shapes, loaded models, light and shadow, clicking
in space. Rendering goes through raylib, so only in a build **with** graphics —
without it the commands report “not available”, like all other graphics
commands.

```basic
IMPORT "g3d"
```

For the mathematics behind it (vectors, quaternions, matrices) there is the
module [`m3d`](module-m3d.md), for an orbit camera `CAMERA_ORBIT` from
[`camera`](module-camera.md).

> **Angles here are in DEGREES** — `MODEL_EX(..., 90.0, ...)` rotates by a
> quarter turn. The neighbouring module `m3d` works in **radians**; anyone
> mixing both converts with `RAD()` or `DEG()`.

## Overview

### Camera

| Function | Returns | Meaning |
|---|---|---|
| `CAMERA3D(px, py, pz, tx, ty, tz, fovy)` | — | set the camera: position, look-at target, field of view in degrees (usually 45). Up is +Y |
| `CAMERA3D_UPDATE(modus)` | — | let raylib move the camera: `1` free, `2` orbital, `3` first person, `4` third person — reads mouse and WASD itself |
| `CAMERA3D_MOVE(vor, rechts, hoch, gieren, neigen[, zoom])` | — | move the camera from the program (in units, forward/right on the ground plane) and turn it (degrees: yaw to the right, pitch upwards, at most 89 degrees); `zoom` changes the distance to the target (positive = farther away). For your own key bindings, gamepad or camera flights; works from the position that `CAMERA3D` last set — so do not call `CAMERA3D` again in every frame |
| `CAMERA3D_X()` / `CAMERA3D_Y()` / `CAMERA3D_Z()` | FLOAT | where is the camera right now? |
| `CAMERA3D_TARGET_X()` / `CAMERA3D_TARGET_Y()` / `CAMERA3D_TARGET_Z()` | FLOAT | what is it looking at? |

### Shapes

They are drawn **anew every frame** — good for trying things out, for outlines
and guide lines. Anyone who needs the same shape often uses a model (see below).

| Function | Meaning |
|---|---|
| `CUBE(x, y, z, breite, hoehe, tiefe, farbe)` | filled box |
| `CUBE_WIRES(x, y, z, breite, hoehe, tiefe, farbe)` | the same as a wireframe |
| `SPHERE(x, y, z, radius, farbe)` | sphere around a centre point |
| `SPHERE_WIRES(x, y, z, radius, farbe)` | sphere as a wireframe |
| `CYLINDER(x, y, z, r_oben, r_unten, hoehe, farbe)` | cylinder — with `r_oben = 0` it becomes a cone |
| `CYLINDER_WIRES(x, y, z, r_oben, r_unten, hoehe, farbe)` | the same as a wireframe |
| `CYLINDER_EX(x1, y1, z1, x2, y2, z2, r_anfang, r_ende, farbe)` | cylinder (or cone) from one point to another, in any direction — pipes, branches, beams |
| `CAPSULE(x1, y1, z1, x2, y2, z2, r, farbe)` | capsule between two points (cylinder with hemispheres) — the usual shape for characters and their collision |
| `CAPSULE_WIRES(x1, y1, z1, x2, y2, z2, r, farbe)` | the same as a wireframe |
| `TRIANGLE3D(x1, y1, z1, x2, y2, z2, x3, y3, z3, farbe)` | free triangle in space, visible from both sides |
| `CIRCLE3D(x, y, z, r, farbe)` | circle outline lying flat on the ground — target marker, range, shadow edge |
| `BBOX_WIRES(x1, y1, z1, x2, y2, z2, farbe)` | box as edges between two corners (the order does not matter); with `MODEL_BBOX` the frame around a model |
| `PLANE(x, y, z, size_x, size_z, farbe)` | flat plane in the XZ plane (ground) |
| `LINE3D(x1, y1, z1, x2, y2, z2, farbe)` | line between two points |
| `POINT3D(x, y, z, farbe)` | single point |
| `GRID3D(linien, abstand)` | ground grid — helps enormously with judging distances |
| `BILLBOARD(bild, x, y, z, groesse, farbe)` | image in space that always turns towards the camera (trees, sparks, labels) |
| `BILLBOARD_PART(bild, sx, sy, sb, sh, x, y, z, groesse, farbe)` | only a section of the image — a single frame from a sprite sheet, that is how characters walk in 3D; `groesse` is the height, the width follows the section |
| `BILLBOARD_EX(bild, x, y, z, breite, hoehe, winkel, farbe)` | with its own width and height, rotated around its centre (degrees) — spinning sparks, stretched beams |

### Models

| Function | Returns | Meaning |
|---|---|---|
| `LOADMODEL(pfad$)` | INTEGER | load a model from a file (OBJ, GLTF, IQM …) |
| `MESH_CUBE(breite, hoehe, tiefe)` | INTEGER | box as a model — without a file |
| `MESH_SPHERE(radius, ringe, segmente)` | INTEGER | sphere; more rings and segments = rounder and more expensive |
| `MESH_CYLINDER(radius, hoehe, segmente)` | INTEGER | cylinder |
| `MESH_TORUS(radius, dicke, rad_seg, seiten)` | INTEGER | ring |
| `MESH_KNOT(radius, dicke, rad_seg, seiten)` | INTEGER | trefoil knot |
| `MESH_PLANE(breite, laenge, res_x, res_z)` | INTEGER | plane with subdivision |
| `MESH_HEIGHTMAP(bild, groesse_x, groesse_y, groesse_z)` | INTEGER | terrain from a greyscale image: bright = high, `groesse_y` determines how high |
| `MESH_CONE(radius, hoehe, segmente)` | INTEGER | cone standing on the ground (y from 0 to `hoehe`) |
| `MESH_HEMISPHERE(radius, ringe, segmente)` | INTEGER | upper half of a sphere, open at the bottom — domes, hills |
| `MESH_POLY(seiten, radius)` | INTEGER | flat regular polygon on the ground (3 to 256 sides) |
| `MESH_CUBICMAP(bild, breite, hoehe, tiefe)` | INTEGER | maze from an image: every white point becomes a cube of size `breite` x `hoehe` x `tiefe`, every black one gets a floor and a ceiling that is visible from inside; other colours stay empty. Point (x, y) lies at x·breite, y·tiefe. Use with `MODEL_LIT` and a light, otherwise walls and corridors look the same |
| `MODEL(modell, x, y, z, skala, farbe)` | — | draw |
| `MODEL_EX(modell, x, y, z, achse_x, achse_y, achse_z, winkel_grad, skala, farbe)` | — | draw and rotate around an axis |
| `MODEL_WIRES(modell, x, y, z, skala, farbe)` | — | draw as a wireframe |
| `MODEL_BBOX(modell)` | TUPLE | bounding box `(min_x, min_y, min_z, max_x, max_y, max_z)` in the model's own coordinates (without the position and scale used when drawing) — for collisions and for standing a model on the ground |
| `MODEL_SAVE(modell, pfad$)` | — | write all parts of the model as Wavefront OBJ (`.obj`) — keep editing a maze from `MESH_CUBICMAP` or a terrain in Blender, bring it back with `LOADMODEL`. Without material, texture and animation; the texture coordinates are written so that they are the right way round again after `LOADMODEL` |
| `MODEL_TEXTURE(modell, bild)` | — | apply an image as the surface |
| `MODEL_TEXTURE_NORMAL(modell, bild)` | — | normal map for surface structure (only works with `MODEL_LIT`) |

### Animated models (skeleton)

| Function | Returns | Meaning |
|---|---|---|
| `MODEL_LOAD_ANIMS(pfad$)` | INTEGER | load animations from a file (GLTF/IQM) |
| `MODEL_ANIM_COUNT(set)` | INTEGER | how many animations are in it? |
| `MODEL_ANIM_NAME(set, idx)` | STRING | name of an animation |
| `MODEL_ANIM_FRAMES(set, idx)` | INTEGER | how many frames does it have? |
| `MODEL_ANIMATE(modell, set, anim, frame)` | — | put the model into this pose; `frame` wraps around |
| `MODEL_ANIMATE_BLEND(modell, set, anim_a, frame_a, anim_b, frame_b, blend)` | — | blend smoothly between two animations (`blend` 0 = all A, 1 = all B) — for walking to running without a jump |

### Light and material

| Function | Returns | Meaning |
|---|---|---|
| `LIGHT_ENABLE()` | — | switch lighting on (once; there is no counterpart for switching it off) |
| `LIGHT_AMBIENT(farbe, staerke)` | — | base brightness, so that shadow sides do not sink into black |
| `LIGHT_DIRECTIONAL(x, y, z, farbe)` | INTEGER | light from one direction, like the sun — returns the light number |
| `LIGHT_POINT(x, y, z, farbe)` | INTEGER | light from a point, like a lamp |
| `LIGHT_SET_POS(licht, x, y, z)` | — | move a light |
| `LIGHT_SET_COLOR(licht, farbe)` | — | change the light colour |
| `LIGHT_SET_ENABLED(licht [, an])` | — | switch a single light on or off |
| `LIGHT_FOG(farbe, dichte)` | — | fog with distance; makes far things fade |
| `MODEL_LIT(modell)` | — | this model is lit (otherwise it draws flat) |
| `MODEL_PBR(modell, metalness, roughness)` | — | material behaviour: `metalness` 0 = plastic, 1 = metal; `roughness` 0 = mirror-like, 1 = matte |
| `MODEL_EMISSIVE(modell, farbe, staerke)` | — | the model glows by itself — even through the fog |

### Environment and shadows

| Function | Returns | Meaning |
|---|---|---|
| `LIGHT_ENV(himmel, boden, intensitaet)` | — | ambient light from two colours — metals reflect it |
| `LIGHT_ENV_HDR(pfad$ [, intensitaet])` | — | a real panorama from a `.hdr` file instead of the two-colour approximation |
| `SKYBOX(an)` | — | also show the panorama as the background (needs `LIGHT_ENV_HDR`) |
| `SKYBOX_IMAGE(bild)` | — | sky from an ordinary image instead of a `.hdr` file: six faces as a strip (6:1 or 1:6) or as a cross (4:3 or 3:4), order +X, −X, +Y, −Y, +Z, −Z; shows it right away, `SKYBOX(FALSE)` hides it |
| `SHADOW_ENABLE([aufloesung])` | — | switch shadow casting on; the first directional light casts it |
| `SHADOW_AREA(groesse, dist)` | — | which section gets shadows — small = sharp, large = covers more |
| `SHADOW_TARGET(x, y, z)` | — | what that section is centred on (usually the player character) |

### Clicking and hits

Two kinds: `PICK_*` takes **the mouse ray automatically** — that is the usual
case. `RAY_HIT_*` gets the ray from you and suits shots, line of sight and
anything that does not hang on the mouse. Both return the distance to the hit
or `-1`.

| Function | Returns | Meaning |
|---|---|---|
| `PICK_BOX(cx, cy, cz, sx, sy, sz)` | FLOAT | does the mouse pointer hit this box? (centre and full size) |
| `PICK_SPHERE(cx, cy, cz, r)` | FLOAT | … this sphere? |
| `PICK_MODEL(modell, px, py, pz [, skala])` | FLOAT | … this model? (real triangles, not just a bounding volume) |
| `PICK_TRI(x1, y1, z1, x2, y2, z2, x3, y3, z3)` | FLOAT | … this triangle? |
| `PICK_QUAD(x1, y1, z1, x2, y2, z2, x3, y3, z3, x4, y4, z4)` | FLOAT | … this quad? The points must lie **in order around it** |
| `RAY_HIT_BOX(ox, oy, oz, dx, dy, dz, cx, cy, cz, sx, sy, sz)` | FLOAT | your own ray against a box |
| `RAY_HIT_SPHERE(ox, oy, oz, dx, dy, dz, cx, cy, cz, r)` | FLOAT | your own ray against a sphere |
| `RAY_HIT_MODEL(modell, ox, oy, oz, dx, dy, dz, px, py, pz [, skala])` | FLOAT | your own ray against a model |
| `RAY_HIT_TRI(ox, oy, oz, dx, dy, dz, x1, y1, z1, x2, y2, z2, x3, y3, z3)` | FLOAT | your own ray against a triangle |
| `RAY_HIT_QUAD(ox, oy, oz, dx, dy, dz, x1, y1, z1, x2, y2, z2, x3, y3, z3, x4, y4, z4)` | FLOAT | your own ray against a quad |
| `MOUSE_GROUND_X(hoehe)` / `MOUSE_GROUND_Z(hoehe)` | FLOAT | where does the mouse pointer hit the horizontal plane at this height? |
| `MOUSE_GROUND_HIT(hoehe)` | BOOLEAN | does it hit it at all? (not when looking at the sky) |
| `SCREEN_TO_WORLD_DIR_X(sx, sy)` / `SCREEN_TO_WORLD_DIR_Y(sx, sy)` / `SCREEN_TO_WORLD_DIR_Z(sx, sy)` | FLOAT | direction of the ray through a screen point — for your own hit tests |
| `WORLD_TO_SCREEN_X(wx, wy, wz)` / `WORLD_TO_SCREEN_Y(wx, wy, wz)` | FLOAT | where does a point of the world land on the screen? — for labels above characters |

## How the image is produced

3D and 2D do not mix: dhrt draws the whole 3D scene **first** and everything
two-dimensional on top **afterwards**. A health bar made of `BOX`/`TEXT` thus
always lies above the scene, no matter in which order you write it.

The 3D commands are collected per frame and drawn at `FLIP()`; afterwards the
list is empty. So you call them again in every pass — just like `BOX` and
`CIRCLE` in 2D.

```basic
IMPORT "g3d"
SCREEN(640, 400, "Erste Schritte in 3D")

DIM t AS FLOAT
WHILE NOT QUITREQUESTED()
    CLS(&H101018)
    t = t + DELTA()

    CAMERA3D(6.0 * COS(t), 4.0, 6.0 * SIN(t), 0, 0, 0, 45)
    GRID3D(20, 1.0)
    CUBE(0, 0.5, 0, 1, 1, 1, &HE05070)
    SPHERE(2, 0.5, 0, 0.5, &H30A0FF)

    TEXT(10, 10, "2D liegt immer obenauf")
    FLIP()
WEND
```

Coordinates are world units, not screen pixels. Without `CAMERA3D` a default
camera looks at the origin diagonally from the front and above.

Demo: [examples/82_3d_intro.dh](../../examples/82_3d_intro.dh).

## Shapes or models?

raylib rebuilds the shapes above in every frame. For a few cubes that is
right. Anyone who draws the same shape a hundred times or needs a loaded
character uses a **model**: created once, drawn as often as you like.

```basic
IMPORT "g3d"
SCREEN(640, 400)

DIM ring AS INTEGER
DIM w AS FLOAT
ring = MESH_TORUS(1.0, 0.3, 16, 32)

WHILE NOT QUITREQUESTED()
    CLS(0)
    CAMERA3D(0, 3, 6, 0, 0, 0, 45)
    w = w + 60.0 * DELTA()
    MODEL_EX(ring, 0, 0, 0, 0, 1, 0, w, 1.0, WHITE)
    FLIP()
WEND
```

`MESH_HEIGHTMAP` builds a terrain from a greyscale image — bright spots become
mountains. Demo: [examples/89_heightmap.dh](../../examples/89_heightmap.dh), plus
[examples/88_3d_models.dh](../../examples/88_3d_models.dh) for torus, knot and
sphere entirely without asset files.

## Animated characters

Rigged models (GLTF/IQM) bring their animations along. You load them
separately from the model and set the pose per frame:

```basic
IMPORT "g3d"
SCREEN(640, 400)

DIM held AS INTEGER
DIM anims AS INTEGER
DIM bild AS INTEGER
held = LOADMODEL("assets/robot.glb")
anims = MODEL_LOAD_ANIMS("assets/robot.glb")
PRINT MODEL_ANIM_COUNT(anims), MODEL_ANIM_NAME(anims, 0)

WHILE NOT QUITREQUESTED()
    CLS(0)
    CAMERA3D(0, 2, 5, 0, 1, 0, 45)
    bild = bild + 1
    MODEL_ANIMATE(held, anims, 0, bild)
    MODEL_EX(held, 0, 0, 0, 0, 1, 0, 180.0, 1.0, WHITE)
    FLIP()
WEND
```

`MODEL_ANIMATE_BLEND` mixes two animations — that turns a hard switch from
walking to running into a smooth transition. Demo:
[examples/108_skeletal_anim.dh](../../examples/108_skeletal_anim.dh).

## Light

Without `LIGHT_ENABLE()` everything draws flat in its base colour. After it,
every model that has received `MODEL_LIT` is lit:

```basic
IMPORT "g3d"
SCREEN(640, 400)

DIM kugel AS INTEGER
DIM sonne AS INTEGER
kugel = MESH_SPHERE(1.0, 24, 32)

LIGHT_ENABLE()
LIGHT_AMBIENT(&H203040, 0.3)
sonne = LIGHT_DIRECTIONAL(-1, -1, -0.5, &HFFF0E0)
MODEL_LIT(kugel)
MODEL_PBR(kugel, 1.0, 0.25)

WHILE NOT QUITREQUESTED()
    CLS(&H0A0A10)
    CAMERA3D(0, 2, 5, 0, 0, 0, 45)
    MODEL(kugel, 0, 0, 0, 1.0, WHITE)
    FLIP()
WEND
```

**At most four lights** are possible at the same time. `LIGHT_FOG` makes
distant things fade; `MODEL_EMISSIVE` makes a model glow by itself, which
together with a bloom `POSTFX` yields real neon. Demos:
[91_lighting](../../examples/91_lighting.dh), [92_fog](../../examples/92_fog.dh),
[95_pbr](../../examples/95_pbr.dh),
[110_emissive_glow](../../examples/110_emissive_glow.dh).

## Environment and shadows

`LIGHT_ENV(himmel, boden, intensitaet)` gives metals something to reflect
without needing a file. Anyone who has a real panorama uses
`LIGHT_ENV_HDR("bild.hdr")` — and with `SKYBOX(TRUE)` it is visible as the
background too. Demos: [96_ibl](../../examples/96_ibl.dh),
[99_ibl_hdr](../../examples/99_ibl_hdr.dh).

Shadows cost three lines:

```basic
IMPORT "g3d"
SCREEN(640, 400)
SHADOW_ENABLE(2048)
SHADOW_AREA(20.0, 30.0)
SHADOW_TARGET(0, 0, 0)
```

The **first** directional light casts the shadow; models need `MODEL_LIT` to
cast and receive it. A smaller `SHADOW_AREA` value makes the shadow sharper
but covers less — which is why you make it follow the player character with
`SHADOW_TARGET`. Demo: [93_shadows](../../examples/93_shadows.dh).

## Clicking

For the normal case — “what did I click?” — `PICK_*` is enough:

```basic
IMPORT "g3d"
SCREEN(640, 400)

DIM d AS FLOAT
WHILE NOT QUITREQUESTED()
    CLS(0)
    CAMERA3D(4, 4, 4, 0, 0, 0, 45)
    d = PICK_BOX(0, 0, 0, 1, 1, 1)
    IF d >= 0 THEN
        CUBE(0, 0, 0, 1, 1, 1, &HFFD040)
    ELSE
        CUBE(0, 0, 0, 1, 1, 1, &H406080)
    END IF
    FLIP()
WEND
```

The return value is the **distance**, not just a yes/no — with several
candidates the smallest value wins, and `-1` means “not hit”.

For a ground tile under the mouse (strategy game, building site)
`MOUSE_GROUND_X/Z(hoehe)` is the shortest way; `WORLD_TO_SCREEN_X/Y` goes the
other way round and says where above a character its name belongs. Demos:
[90_billboards_picking](../../examples/90_billboards_picking.dh),
[151_picking_flaechen](../../examples/151_picking_flaechen.dh).

## Limits

* **At most four lights** at the same time; further ones are ignored.
* **Only the first directional light casts shadows.**
* `LIGHT_ENABLE()` cannot be switched off again. Individual lights can
  (`LIGHT_SET_ENABLED`), and `GFX_PUSH`/`GFX_POP` restore the whole state.
* **No sorting of transparent surfaces** — anyone drawing with alpha has to
  choose a suitable order themselves.
* The hit tests know **no backface culling**: a surface is hit from behind
  as well.
* `BATCH_DRAW`/`BATCH_FLUSH` have no effect here — dhrt collects drawing
  commands anyway and issues them at `FLIP`.

## Related

| What for | Where |
|---|---|
| Vectors, quaternions, matrices, custom projections | [`m3d`](module-m3d.md) |
| Orbit camera with yaw/pitch | `CAMERA_ORBIT` in [`camera`](module-camera.md) |
| Rigid-body physics in space | [`physics3d`](module-physics3d.md) |
| Sphere and triangle mathematics without a physics world | [`physics`](module-physics.md) |
| Fullscreen shader over the finished scene | `POSTFX` in [builtins-grafik](builtins-grafik.md) |
| How the 3D part is built internally (shaders, depth FBO) | [rust-runtime](rust-runtime.md) |
