# Modul `g3d` — 3D-Grafik

Dreidimensionale Szenen: Formen, geladene Modelle, Licht und Schatten, Anklicken
im Raum. Gerendert wird über raylib, also nur in einem Bau **mit** Grafik — ohne
den melden die Befehle „nicht verfügbar", wie alle anderen Grafik-Befehle auch.

```basic
IMPORT "g3d"
```

Für die Mathematik dahinter (Vektoren, Quaternionen, Matrizen) gibt es das Modul
[`m3d`](module-m3d.md), für eine Umlauf-Kamera `CAMERA_ORBIT` aus
[`camera`](module-camera.md).

> **Winkel sind hier in GRAD** — `MODEL_EX(..., 90.0, ...)` dreht um eine
> Vierteldrehung. Das Modul `m3d` daneben rechnet im **Bogenmaß**; wer beides
> mischt, rechnet mit `RAD()` bzw. `DEG()` um.

## Übersicht

### Kamera

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `CAMERA3D(px, py, pz, tx, ty, tz, fovy)` | — | Kamera setzen: Standort, Blickziel, Öffnungswinkel in Grad (üblich 45). Oben ist +Y |
| `CAMERA3D_UPDATE(modus)` | — | raylib die Kamera bewegen lassen: `1` frei, `2` umkreisend, `3` Ego, `4` Verfolger — liest selbst Maus und WASD |
| `CAMERA3D_X()` / `CAMERA3D_Y()` / `CAMERA3D_Z()` | FLOAT | wo steht die Kamera gerade? |
| `CAMERA3D_TARGET_X()` / `CAMERA3D_TARGET_Y()` / `CAMERA3D_TARGET_Z()` | FLOAT | worauf blickt sie? |

### Formen

Sie werden **jedes Bild neu** gezeichnet — gut zum Ausprobieren, für Umrisse und
Hilfslinien. Wer dieselbe Form oft braucht, nimmt ein Modell (siehe unten).

| Funktion | Bedeutung |
|---|---|
| `CUBE(x, y, z, breite, hoehe, tiefe, farbe)` | gefüllter Quader |
| `CUBE_WIRES(x, y, z, breite, hoehe, tiefe, farbe)` | derselbe als Drahtgitter |
| `SPHERE(x, y, z, radius, farbe)` | Kugel um einen Mittelpunkt |
| `SPHERE_WIRES(x, y, z, radius, farbe)` | Kugel als Drahtgitter |
| `CYLINDER(x, y, z, r_oben, r_unten, hoehe, farbe)` | Zylinder — mit `r_oben = 0` wird ein Kegel daraus |
| `PLANE(x, y, z, size_x, size_z, farbe)` | flache Ebene in der XZ-Fläche (Boden) |
| `LINE3D(x1, y1, z1, x2, y2, z2, farbe)` | Linie zwischen zwei Punkten |
| `POINT3D(x, y, z, farbe)` | einzelner Punkt |
| `GRID3D(linien, abstand)` | Bodenraster — hilft ungemein beim Einschätzen von Entfernungen |
| `BILLBOARD(bild, x, y, z, groesse, farbe)` | Bild im Raum, das sich immer zur Kamera dreht (Bäume, Funken, Beschriftungen) |

### Modelle

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `LOADMODEL(pfad$)` | INTEGER | Modell aus einer Datei laden (OBJ, GLTF, IQM …) |
| `MESH_CUBE(breite, hoehe, tiefe)` | INTEGER | Quader als Modell — ohne Datei |
| `MESH_SPHERE(radius, ringe, segmente)` | INTEGER | Kugel; mehr Ringe und Segmente = runder und teurer |
| `MESH_CYLINDER(radius, hoehe, segmente)` | INTEGER | Zylinder |
| `MESH_TORUS(radius, dicke, rad_seg, seiten)` | INTEGER | Ring |
| `MESH_KNOT(radius, dicke, rad_seg, seiten)` | INTEGER | Kleeblattknoten |
| `MESH_PLANE(breite, laenge, res_x, res_z)` | INTEGER | Ebene mit Unterteilung |
| `MESH_HEIGHTMAP(bild, groesse_x, groesse_y, groesse_z)` | INTEGER | Gelände aus einem Graustufenbild: hell = hoch, `groesse_y` bestimmt wie hoch |
| `MODEL(modell, x, y, z, skala, farbe)` | — | zeichnen |
| `MODEL_EX(modell, x, y, z, achse_x, achse_y, achse_z, winkel_grad, skala, farbe)` | — | zeichnen und um eine Achse drehen |
| `MODEL_WIRES(modell, x, y, z, skala, farbe)` | — | als Drahtgitter zeichnen |
| `MODEL_TEXTURE(modell, bild)` | — | ein Bild als Oberfläche auflegen |
| `MODEL_TEXTURE_NORMAL(modell, bild)` | — | Normal-Map für Oberflächenstruktur (wirkt nur mit `MODEL_LIT`) |

### Bewegte Modelle (Skelett)

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `MODEL_LOAD_ANIMS(pfad$)` | INTEGER | Animationen aus einer Datei laden (GLTF/IQM) |
| `MODEL_ANIM_COUNT(set)` | INTEGER | wie viele Animationen sind darin? |
| `MODEL_ANIM_NAME(set, idx)` | STRING | Name einer Animation |
| `MODEL_ANIM_FRAMES(set, idx)` | INTEGER | wie viele Einzelbilder hat sie? |
| `MODEL_ANIMATE(modell, set, anim, frame)` | — | Modell in diese Haltung bringen; `frame` läuft im Kreis |
| `MODEL_ANIMATE_BLEND(modell, set, anim_a, frame_a, anim_b, frame_b, blend)` | — | weich zwischen zwei Animationen überblenden (`blend` 0 = ganz A, 1 = ganz B) — für Gehen nach Rennen ohne Sprung |

### Licht und Material

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `LIGHT_ENABLE()` | — | Beleuchtung einschalten (einmalig; es gibt kein Gegenstück zum Ausschalten) |
| `LIGHT_AMBIENT(farbe, staerke)` | — | Grundhelligkeit, damit Schattenseiten nicht schwarz absaufen |
| `LIGHT_DIRECTIONAL(x, y, z, farbe)` | INTEGER | Licht aus einer Richtung, wie die Sonne — liefert die Lichtnummer |
| `LIGHT_POINT(x, y, z, farbe)` | INTEGER | Licht von einem Punkt aus, wie eine Lampe |
| `LIGHT_SET_POS(licht, x, y, z)` | — | Licht bewegen |
| `LIGHT_SET_COLOR(licht, farbe)` | — | Lichtfarbe ändern |
| `LIGHT_SET_ENABLED(licht [, an])` | — | einzelnes Licht an- oder ausschalten |
| `LIGHT_FOG(farbe, dichte)` | — | Nebel mit der Entfernung; lässt Weites verblassen |
| `MODEL_LIT(modell)` | — | dieses Modell wird beleuchtet (sonst zeichnet es flach) |
| `MODEL_PBR(modell, metalness, roughness)` | — | Materialverhalten: `metalness` 0 = Kunststoff, 1 = Metall; `roughness` 0 = spiegelnd, 1 = matt |
| `MODEL_EMISSIVE(modell, farbe, staerke)` | — | das Modell leuchtet selbst — auch durch den Nebel hindurch |

### Umgebung und Schatten

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `LIGHT_ENV(himmel, boden, intensitaet)` | — | Umgebungslicht aus zwei Farben — Metalle spiegeln es |
| `LIGHT_ENV_HDR(pfad$ [, intensitaet])` | — | echtes Panorama aus einer `.hdr`-Datei statt der Zwei-Farben-Näherung |
| `SKYBOX(an)` | — | das Panorama auch als Hintergrund zeigen (braucht `LIGHT_ENV_HDR`) |
| `SHADOW_ENABLE([aufloesung])` | — | Schattenwurf einschalten; das erste gerichtete Licht wirft ihn |
| `SHADOW_AREA(groesse, dist)` | — | welcher Ausschnitt Schatten bekommt — klein = scharf, groß = deckt mehr ab |
| `SHADOW_TARGET(x, y, z)` | — | worauf dieser Ausschnitt zentriert ist (meist die Spielfigur) |

### Anklicken und Treffer

Zwei Sorten: `PICK_*` nimmt **automatisch den Mausstrahl** — das ist der übliche
Fall. `RAY_HIT_*` bekommt den Strahl von dir und eignet sich für Schüsse, Sicht
und alles, was nicht an der Maus hängt. Beide liefern die Entfernung bis zum
Treffer oder `-1`.

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `PICK_BOX(cx, cy, cz, sx, sy, sz)` | FLOAT | trifft der Mauszeiger diesen Quader? (Mittelpunkt und volle Größe) |
| `PICK_SPHERE(cx, cy, cz, r)` | FLOAT | … diese Kugel? |
| `PICK_MODEL(modell, px, py, pz [, skala])` | FLOAT | … dieses Modell? (echte Dreiecke, nicht nur ein Hüllkörper) |
| `PICK_TRI(x1, y1, z1, x2, y2, z2, x3, y3, z3)` | FLOAT | … dieses Dreieck? |
| `PICK_QUAD(x1, y1, z1, x2, y2, z2, x3, y3, z3, x4, y4, z4)` | FLOAT | … dieses Viereck? Die Punkte müssen **reihum** liegen |
| `RAY_HIT_BOX(ox, oy, oz, dx, dy, dz, cx, cy, cz, sx, sy, sz)` | FLOAT | eigener Strahl gegen einen Quader |
| `RAY_HIT_SPHERE(ox, oy, oz, dx, dy, dz, cx, cy, cz, r)` | FLOAT | eigener Strahl gegen eine Kugel |
| `RAY_HIT_MODEL(modell, ox, oy, oz, dx, dy, dz, px, py, pz [, skala])` | FLOAT | eigener Strahl gegen ein Modell |
| `RAY_HIT_TRI(ox, oy, oz, dx, dy, dz, x1, y1, z1, x2, y2, z2, x3, y3, z3)` | FLOAT | eigener Strahl gegen ein Dreieck |
| `RAY_HIT_QUAD(ox, oy, oz, dx, dy, dz, x1, y1, z1, x2, y2, z2, x3, y3, z3, x4, y4, z4)` | FLOAT | eigener Strahl gegen ein Viereck |
| `MOUSE_GROUND_X(hoehe)` / `MOUSE_GROUND_Z(hoehe)` | FLOAT | wo trifft der Mauszeiger die waagerechte Ebene auf dieser Höhe? |
| `MOUSE_GROUND_HIT(hoehe)` | BOOLEAN | trifft er sie überhaupt? (bei Blick zum Himmel nicht) |
| `SCREEN_TO_WORLD_DIR_X(sx, sy)` / `SCREEN_TO_WORLD_DIR_Y(sx, sy)` / `SCREEN_TO_WORLD_DIR_Z(sx, sy)` | FLOAT | Richtung des Strahls durch einen Bildschirmpunkt — für eigene Treffertests |
| `WORLD_TO_SCREEN_X(wx, wy, wz)` / `WORLD_TO_SCREEN_Y(wx, wy, wz)` | FLOAT | wo landet ein Punkt der Welt auf dem Bildschirm? — für Beschriftungen über Figuren |

## Wie das Bild entsteht

3D und 2D mischen sich nicht: dhrt zeichnet **zuerst** die ganze 3D-Szene und
**danach** alles Zweidimensionale obenauf. Ein Lebensbalken aus `BOX`/`TEXT` liegt
also immer über der Szene, egal in welcher Reihenfolge man ihn hinschreibt.

Die 3D-Befehle sammeln sich pro Bild und werden beim `FLIP()` gezeichnet; danach
ist die Liste leer. Man ruft sie also in jedem Durchgang neu auf — genau wie `BOX`
und `CIRCLE` in 2D.

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

Koordinaten sind Welt-Einheiten, keine Bildschirmpixel. Ohne `CAMERA3D` blickt
eine Vorgabe-Kamera schräg von vorn-oben auf den Ursprung.

Demo: [examples/82_3d_intro.dh](../examples/82_3d_intro.dh).

## Formen oder Modelle?

Die Formen oben baut raylib in jedem Bild neu auf. Für ein paar Würfel ist das
richtig. Wer dieselbe Form hundertmal zeichnet oder eine geladene Figur braucht,
nimmt ein **Modell**: einmal erzeugt, beliebig oft gezeichnet.

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

`MESH_HEIGHTMAP` baut aus einem Graustufenbild ein Gelände — helle Stellen werden
zu Bergen. Demo: [examples/89_heightmap.dh](../examples/89_heightmap.dh), dazu
[examples/88_3d_models.dh](../examples/88_3d_models.dh) für Torus, Knoten und
Kugel ganz ohne Asset-Dateien.

## Bewegte Figuren

Geriggte Modelle (GLTF/IQM) bringen ihre Animationen mit. Man lädt sie getrennt
vom Modell und setzt pro Bild die Haltung:

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

`MODEL_ANIMATE_BLEND` mischt zwei Animationen — damit wird aus einem harten
Wechsel von Gehen auf Rennen ein weicher Übergang. Demo:
[examples/108_skeletal_anim.dh](../examples/108_skeletal_anim.dh).

## Licht

Ohne `LIGHT_ENABLE()` zeichnet alles flach in seiner Grundfarbe. Danach wird jedes
Modell beleuchtet, das `MODEL_LIT` bekommen hat:

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

Es sind **höchstens vier Lichter** gleichzeitig möglich. `LIGHT_FOG` lässt Fernes
verblassen; `MODEL_EMISSIVE` macht ein Modell selbstleuchtend, was zusammen mit
einem Bloom-`POSTFX` echtes Neon ergibt. Demos:
[91_lighting](../examples/91_lighting.dh), [92_fog](../examples/92_fog.dh),
[95_pbr](../examples/95_pbr.dh),
[110_emissive_glow](../examples/110_emissive_glow.dh).

## Umgebung und Schatten

`LIGHT_ENV(himmel, boden, intensitaet)` gibt Metallen etwas zum Spiegeln, ohne dass
eine Datei nötig wäre. Wer ein echtes Panorama hat, nimmt
`LIGHT_ENV_HDR("bild.hdr")` — und mit `SKYBOX(TRUE)` ist es auch als Hintergrund zu
sehen. Demos: [96_ibl](../examples/96_ibl.dh),
[99_ibl_hdr](../examples/99_ibl_hdr.dh).

Schatten kosten drei Zeilen:

```basic
IMPORT "g3d"
SCREEN(640, 400)
SHADOW_ENABLE(2048)
SHADOW_AREA(20.0, 30.0)
SHADOW_TARGET(0, 0, 0)
```

Den Schatten wirft das **erste** gerichtete Licht; Modelle brauchen `MODEL_LIT`, um
zu werfen und zu empfangen. Ein kleinerer `SHADOW_AREA`-Wert macht den Schatten
schärfer, deckt aber weniger ab — deshalb führt man ihn mit `SHADOW_TARGET` der
Spielfigur nach. Demo: [93_shadows](../examples/93_shadows.dh).

## Anklicken

Für den Normalfall — „was habe ich angeklickt?" — genügt `PICK_*`:

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

Der Rückgabewert ist die **Entfernung**, nicht nur ein Ja/Nein — bei mehreren
Kandidaten gewinnt der kleinste Wert, und `-1` heißt „nicht getroffen".

Für eine Bodenkachel unter der Maus (Strategiespiel, Bauplatz) ist
`MOUSE_GROUND_X/Z(hoehe)` der kürzeste Weg; `WORLD_TO_SCREEN_X/Y` geht andersherum
und sagt, wo über einer Figur ihr Name hingehört. Demos:
[90_billboards_picking](../examples/90_billboards_picking.dh),
[151_picking_flaechen](../examples/151_picking_flaechen.dh).

## Grenzen

* **Höchstens vier Lichter** gleichzeitig; weitere werden ignoriert.
* **Nur das erste gerichtete Licht wirft Schatten.**
* `LIGHT_ENABLE()` lässt sich nicht wieder ausschalten. Einzelne Lichter schon
  (`LIGHT_SET_ENABLED`), und `GFX_PUSH`/`GFX_POP` stellen den ganzen Zustand
  zurück.
* **Keine Sortierung durchsichtiger Flächen** — wer mit Alpha zeichnet, muss die
  Reihenfolge selbst passend wählen.
* Die Treffertests kennen **kein Backface-Culling**: eine Fläche trifft auch von
  hinten.
* `BATCH_DRAW`/`BATCH_FLUSH` sind hier ohne Wirkung — dhrt sammelt Zeichenbefehle
  ohnehin und gibt sie beim `FLIP` aus.

## Verwandtes

| Wofür | Wo |
|---|---|
| Vektoren, Quaternionen, Matrizen, eigene Projektionen | [`m3d`](module-m3d.md) |
| Umlauf-Kamera mit yaw/pitch | `CAMERA_ORBIT` in [`camera`](module-camera.md) |
| Starrkörper-Physik im Raum | [`physics3d`](module-physics3d.md) |
| Kugel- und Dreiecks-Mathematik ohne Physik-Welt | [`physics`](module-physics.md) |
| Vollbild-Shader über die fertige Szene | `POSTFX` in [builtins-grafik](builtins-grafik.md) |
| Wie der 3D-Teil intern gebaut ist (Shader, Depth-FBO) | [rust-runtime](rust-runtime.md) |
