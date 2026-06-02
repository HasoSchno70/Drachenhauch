"""g3d-Modul fuer GameBasic -- 3D-Grafik ueber raylibs Mesh/Kamera-API.

3D ist **nur in der nativen Runtime** (`gbrt`, via F6 / `gbrun.py --native`)
verfuegbar -- der Python/pygame-Pfad (F5, Tree-Walker / Python-VM) kann kein
3D rendern. Dieses Modul registriert die Builtins nur, damit der Compiler sie
als `CALL_BUILTIN` erkennt; die eigentliche Umsetzung liegt in
`rust/gb_runtime/src/graphics.rs` + `vm.rs`. Im Python-Pfad werfen die
Funktionen eine klare Meldung.

Modell (Recording, wie 2D): pro Frame
  CLS(...)
  CAMERA3D(px,py,pz, tx,ty,tz, fovy)     ' Perspektiv-Kamera setzen
  CUBE(0,0,0, 2,2,2, RED) : SPHERE(3,0,0, 1, BLUE) : GRID3D(10, 1.0)
  ' danach 2D-HUD ganz normal: TEXT(...), BOX(...) -- liegt ueber dem 3D-Bild
  FLIP()

Beim FLIP werden zuerst alle 3D-Cmds in einem `begin_mode3D`-Block (mit der
gesetzten Kamera) gerendert, danach die 2D-Layer obendrauf. Koordinaten sind
Welt-Einheiten (FLOAT/INTEGER), Farben `0xRRGGBB`-INTEGER.

  IMPORT "g3d"
  SCREEN(640, 480, "3D", 1)
  WHILE NOT QUITREQUESTED()
      CLS(&H101820)
      CAMERA3D(6, 5, 6, 0, 0, 0, 45)
      GRID3D(10, 1.0)
      CUBE(0, 0.5, 0, 1, 1, 1, RED)
      CUBE_WIRES(0, 0.5, 0, 1, 1, 1, BLACK)
      TEXT(10, 10, "FPS")            ' 2D-HUD ueber der Szene
      FLIP() : SLEEP(16)
  WEND
"""
from __future__ import annotations

from ..builtins_registry import graphics_builtin
from ..errors import GBRuntimeError


def _native_only(name: str):
    raise GBRuntimeError(
        f"{name}: 3D wird nur von der nativen Runtime unterstuetzt -- mit F6 "
        f"bzw. 'gbrun.py --native' ausfuehren (nicht F5/Tree-Walker)")


@graphics_builtin("CAMERA3D", arity=7)
def _camera3d(g, *args):
    """Setzt die Perspektiv-Kamera fuer diesen Frame:
    CAMERA3D(pos_x, pos_y, pos_z, target_x, target_y, target_z, fovy_grad)."""
    _native_only("CAMERA3D")


@graphics_builtin("CUBE", arity=7)
def _cube(g, *args):
    """CUBE(x, y, z, breite, hoehe, tiefe, farbe) -- gefuellter Quader."""
    _native_only("CUBE")


@graphics_builtin("CUBE_WIRES", arity=7)
def _cube_wires(g, *args):
    """CUBE_WIRES(x, y, z, breite, hoehe, tiefe, farbe) -- Drahtgitter-Quader."""
    _native_only("CUBE_WIRES")


@graphics_builtin("SPHERE", arity=5)
def _sphere(g, *args):
    """SPHERE(x, y, z, radius, farbe) -- gefuellte Kugel."""
    _native_only("SPHERE")


@graphics_builtin("SPHERE_WIRES", arity=5)
def _sphere_wires(g, *args):
    """SPHERE_WIRES(x, y, z, radius, farbe) -- Drahtgitter-Kugel."""
    _native_only("SPHERE_WIRES")


@graphics_builtin("CYLINDER", arity=7)
def _cylinder(g, *args):
    """CYLINDER(x, y, z, radius_oben, radius_unten, hoehe, farbe) -- gefuellt.
    radius_oben=0 ergibt einen Kegel."""
    _native_only("CYLINDER")


@graphics_builtin("PLANE", arity=6)
def _plane(g, *args):
    """PLANE(x, y, z, groesse_x, groesse_z, farbe) -- gefuellte XZ-Ebene um
    den Mittelpunkt (x,y,z)."""
    _native_only("PLANE")


@graphics_builtin("LINE3D", arity=7)
def _line3d(g, *args):
    """LINE3D(x1, y1, z1, x2, y2, z2, farbe) -- Linie im Raum."""
    _native_only("LINE3D")


@graphics_builtin("POINT3D", arity=4)
def _point3d(g, *args):
    """POINT3D(x, y, z, farbe) -- Punkt im Raum."""
    _native_only("POINT3D")


@graphics_builtin("GRID3D", arity=2)
def _grid3d(g, *args):
    """GRID3D(linien, abstand) -- Boden-Gitter in der XZ-Ebene (zentriert),
    Hilfsraster fuer die Orientierung."""
    _native_only("GRID3D")


# --- 3D-Modelle (geladen oder prozedural generiert) -----------------
# Alle liefern ein MODEL-Handle (INTEGER), das mit MODEL/MODEL_EX/
# MODEL_WIRES gezeichnet wird. Handles bleiben ueber Frames gueltig
# (anders als die Immediate-Primitive CUBE/SPHERE) -- einmal laden/erzeugen,
# beliebig oft zeichnen.

@graphics_builtin("LOADMODEL", arity=1)
def _load_model(g, *args):
    """LOADMODEL(pfad$) -> MODEL -- laedt ein 3D-Modell (OBJ/GLTF/IQM/...)."""
    _native_only("LOADMODEL")


@graphics_builtin("MESH_CUBE", arity=3)
def _mesh_cube(g, *args):
    """MESH_CUBE(breite, hoehe, tiefe) -> MODEL -- prozeduraler Quader."""
    _native_only("MESH_CUBE")


@graphics_builtin("MESH_SPHERE", arity=3)
def _mesh_sphere(g, *args):
    """MESH_SPHERE(radius, ringe, segmente) -> MODEL -- prozedurale Kugel."""
    _native_only("MESH_SPHERE")


@graphics_builtin("MESH_CYLINDER", arity=3)
def _mesh_cylinder(g, *args):
    """MESH_CYLINDER(radius, hoehe, segmente) -> MODEL -- prozeduraler Zylinder."""
    _native_only("MESH_CYLINDER")


@graphics_builtin("MESH_TORUS", arity=4)
def _mesh_torus(g, *args):
    """MESH_TORUS(radius, dicke, rad_segmente, seiten) -> MODEL -- Torus (Donut)."""
    _native_only("MESH_TORUS")


@graphics_builtin("MESH_KNOT", arity=4)
def _mesh_knot(g, *args):
    """MESH_KNOT(radius, dicke, rad_segmente, seiten) -> MODEL -- Kleeblatt-Knoten."""
    _native_only("MESH_KNOT")


@graphics_builtin("MESH_PLANE", arity=4)
def _mesh_plane(g, *args):
    """MESH_PLANE(breite, laenge, res_x, res_z) -> MODEL -- unterteilte XZ-Ebene."""
    _native_only("MESH_PLANE")


@graphics_builtin("MESH_HEIGHTMAP", arity=4)
def _mesh_heightmap(g, *args):
    """MESH_HEIGHTMAP(bild, groesse_x, groesse_y, groesse_z) -> MODEL -- Terrain
    aus einer (Graustufen-)Image (via LOADIMAGE). Helligkeit pro Pixel = Hoehe;
    groesse_y skaliert die Hoehe, groesse_x/z spannen das Terrain in der Ebene."""
    _native_only("MESH_HEIGHTMAP")


@graphics_builtin("MODEL", arity=6)
def _model(g, *args):
    """MODEL(modell, x, y, z, skalierung, farbe) -- Modell gefuellt zeichnen."""
    _native_only("MODEL")


@graphics_builtin("MODEL_EX", arity=10)
def _model_ex(g, *args):
    """MODEL_EX(modell, x, y, z, achse_x, achse_y, achse_z, winkel_grad,
    skalierung, farbe) -- Modell mit Rotation um eine Achse zeichnen."""
    _native_only("MODEL_EX")


@graphics_builtin("MODEL_WIRES", arity=6)
def _model_wires(g, *args):
    """MODEL_WIRES(modell, x, y, z, skalierung, farbe) -- Drahtgitter."""
    _native_only("MODEL_WIRES")


@graphics_builtin("MODEL_TEXTURE", arity=2)
def _model_texture(g, *args):
    """MODEL_TEXTURE(modell, bild) -- ein via LOADIMAGE geladenes Bild als
    Diffuse-/Albedo-Textur auf das Modell legen."""
    _native_only("MODEL_TEXTURE")


# --- Billboards (zur Kamera ausgerichtete Texturen) -----------------

@graphics_builtin("BILLBOARD", arity=6)
def _billboard(g, *args):
    """BILLBOARD(bild, x, y, z, groesse, farbe) -- eine via LOADIMAGE geladene
    Textur im 3D-Raum, die immer zur Kamera zeigt (Baeume/Sprites/Funken)."""
    _native_only("BILLBOARD")


# --- Ray-Kollision / Picking ----------------------------------------
# Liefern die Distanz vom Ray-Ursprung zum Treffer oder -1 bei keinem Treffer.
# Trefferpunkt = ursprung + richtung * distanz.

@graphics_builtin("RAY_HIT_BOX", arity=12)
def _ray_hit_box(g, *args):
    """RAY_HIT_BOX(ox, oy, oz, dx, dy, dz, cx, cy, cz, sx, sy, sz) -> FLOAT --
    Distanz zum Treffer mit einer AABB (Mittelpunkt c, Vollgroesse s) oder -1."""
    _native_only("RAY_HIT_BOX")


@graphics_builtin("RAY_HIT_SPHERE", arity=10)
def _ray_hit_sphere(g, *args):
    """RAY_HIT_SPHERE(ox, oy, oz, dx, dy, dz, cx, cy, cz, r) -> FLOAT --
    Distanz zum Treffer mit einer Kugel oder -1."""
    _native_only("RAY_HIT_SPHERE")


@graphics_builtin("PICK_BOX", arity=6)
def _pick_box(g, *args):
    """PICK_BOX(cx, cy, cz, sx, sy, sz) -> FLOAT -- Mausstrahl durch die aktuelle
    3D-Kamera gegen eine AABB; Distanz oder -1. Fuer Klick-Selektion."""
    _native_only("PICK_BOX")


@graphics_builtin("PICK_SPHERE", arity=4)
def _pick_sphere(g, *args):
    """PICK_SPHERE(cx, cy, cz, r) -> FLOAT -- Mausstrahl gegen eine Kugel;
    Distanz oder -1."""
    _native_only("PICK_SPHERE")


# --- Kamera-Modi (raylib UpdateCamera) ------------------------------
# CAMERA3D(...) einmal initial setzen, dann pro Frame CAMERA3D_UPDATE(mode).

@graphics_builtin("CAMERA3D_UPDATE", arity=1)
def _camera3d_update(g, *args):
    """CAMERA3D_UPDATE(mode) -- Kamera per raylib-Controller bewegen (liest
    Tastatur/Maus). mode: 1=free, 2=orbital, 3=first_person, 4=third_person."""
    _native_only("CAMERA3D_UPDATE")


@graphics_builtin("CAMERA3D_X", arity=0)
def _camera3d_x(g, *args):
    """CAMERA3D_X() -> FLOAT -- aktuelle Kamera-Position X."""
    _native_only("CAMERA3D_X")


@graphics_builtin("CAMERA3D_Y", arity=0)
def _camera3d_y(g, *args):
    """CAMERA3D_Y() -> FLOAT -- aktuelle Kamera-Position Y."""
    _native_only("CAMERA3D_Y")


@graphics_builtin("CAMERA3D_Z", arity=0)
def _camera3d_z(g, *args):
    """CAMERA3D_Z() -> FLOAT -- aktuelle Kamera-Position Z."""
    _native_only("CAMERA3D_Z")


@graphics_builtin("CAMERA3D_TARGET_X", arity=0)
def _camera3d_target_x(g, *args):
    """CAMERA3D_TARGET_X() -> FLOAT -- aktueller Kamera-Zielpunkt X."""
    _native_only("CAMERA3D_TARGET_X")


@graphics_builtin("CAMERA3D_TARGET_Y", arity=0)
def _camera3d_target_y(g, *args):
    """CAMERA3D_TARGET_Y() -> FLOAT -- aktueller Kamera-Zielpunkt Y."""
    _native_only("CAMERA3D_TARGET_Y")


@graphics_builtin("CAMERA3D_TARGET_Z", arity=0)
def _camera3d_target_z(g, *args):
    """CAMERA3D_TARGET_Z() -> FLOAT -- aktueller Kamera-Zielpunkt Z."""
    _native_only("CAMERA3D_TARGET_Z")


# --- Beleuchtung (Blinn-Phong, bis zu 4 Lichter) --------------------

@graphics_builtin("LIGHT_ENABLE", arity=0)
def _light_enable(g, *args):
    """LIGHT_ENABLE() -- Beleuchtung aktivieren (laedt den Lighting-Shader).
    Modelle leuchten erst nach MODEL_LIT(modell)."""
    _native_only("LIGHT_ENABLE")


@graphics_builtin("LIGHT_AMBIENT", arity=2)
def _light_ambient(g, *args):
    """LIGHT_AMBIENT(farbe, intensitaet) -- Grundhelligkeit (Umgebungslicht)."""
    _native_only("LIGHT_AMBIENT")


@graphics_builtin("LIGHT_FOG", arity=2)
def _light_fog(g, *args):
    """LIGHT_FOG(farbe, dichte) -- exponentieller Tiefen-Fog fuer beleuchtete
    Modelle (MODEL_LIT). dichte 0 = aus; typisch 0.02..0.15. Ferne Objekte
    verblassen zur Fog-Farbe (atmosphaerische Tiefe)."""
    _native_only("LIGHT_FOG")


@graphics_builtin("LIGHT_DIRECTIONAL", arity=4)
def _light_directional(g, *args):
    """LIGHT_DIRECTIONAL(dx, dy, dz, farbe) -> INTEGER -- gerichtetes Licht
    (Sonne); (dx,dy,dz) ist die Lichtrichtung. Liefert den Licht-Index oder -1."""
    _native_only("LIGHT_DIRECTIONAL")


@graphics_builtin("LIGHT_POINT", arity=4)
def _light_point(g, *args):
    """LIGHT_POINT(x, y, z, farbe) -> INTEGER -- Punktlicht an (x,y,z).
    Liefert den Licht-Index oder -1 (max. 4 Lichter)."""
    _native_only("LIGHT_POINT")


@graphics_builtin("LIGHT_SET_POS", arity=4)
def _light_set_pos(g, *args):
    """LIGHT_SET_POS(idx, x, y, z) -- Licht-Position (bzw. -Richtung bei
    directional) animieren."""
    _native_only("LIGHT_SET_POS")


@graphics_builtin("LIGHT_SET_COLOR", arity=2)
def _light_set_color(g, *args):
    """LIGHT_SET_COLOR(idx, farbe) -- Lichtfarbe aendern."""
    _native_only("LIGHT_SET_COLOR")


@graphics_builtin("LIGHT_SET_ENABLED", arity=2)
def _light_set_enabled(g, *args):
    """LIGHT_SET_ENABLED(idx, an) -- Licht ein-/ausschalten."""
    _native_only("LIGHT_SET_ENABLED")


@graphics_builtin("MODEL_LIT", arity=1)
def _model_lit(g, *args):
    """MODEL_LIT(modell) -- den Lighting-Shader an ein Modell haengen, damit es
    von den Lichtern beleuchtet wird."""
    _native_only("MODEL_LIT")
