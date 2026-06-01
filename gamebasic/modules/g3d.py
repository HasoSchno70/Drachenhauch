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
