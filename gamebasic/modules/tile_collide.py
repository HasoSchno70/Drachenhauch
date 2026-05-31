"""Box-vs-Tilemap-Kollision fuer Platformer.

Klassisches separates-Achsen-Sweep-Pattern: erst X-Bewegung resolven, dann
Y. Liefert die neue Position + ein Hit-Flag pro Achse. Wer ein Spiel mit
Mario-/Metroid-Style-Physik baut, ruft pro Frame:

    (new_x, hit_x) = TILE_SWEEP_X(map, layer_idx, x, y, w, h, dx)
    x = new_x
    IF hit_x THEN vx = 0
    (new_y, hit_y) = TILE_SWEEP_Y(map, layer_idx, x, y, w, h, dy)
    y = new_y
    IF hit_y AND dy > 0 THEN
        on_ground = TRUE
        vy = 0
    END IF

Solid-Detection: per Tile-Property `solid: true` (in Tiled gesetzt).
Wenn kein einziger Tile-Type im Tileset ein `solid`-Property hat,
faellt das Modul auf "jeder GID > 0 ist solid" zurueck (Convention:
eine dedizierte Collision-Layer mit nur Wand-Tiles).

API:
    TILE_SWEEP_X(map, layer_idx, x, y, w, h, dx) -> TUPLE (new_x, hit_bool)
    TILE_SWEEP_Y(map, layer_idx, x, y, w, h, dy) -> TUPLE (new_y, hit_bool)
    TILE_IS_SOLID(map, layer_idx, tx, ty)        -> BOOLEAN
    TILE_AT_PIXEL(map, layer_idx, px, py)        -> INTEGER (GID at pixel-pos)

Konvention: x,y,w,h sind in PIXEL-Koordinaten. dx,dy auch in Pixel.
Tile-Koordinaten (tx, ty) sind in Tile-Einheiten.
"""
from __future__ import annotations

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError

# Lazy-Import: vermeidet Zirkular, wenn tile_collide vor tiled geladen wird
from .tiled import _TiledMap, _TiledLayer

# Natives Rust-Backend (optional). Spiegelt die Solid-Maske einmal nach Rust
# und fuehrt den Sweep dort aus (gleiche floor/Kanten-Logik wie _sweep_axis).
try:
    from ..gb_native import TileCollider as _NativeCollider  # type: ignore
except Exception:
    _NativeCollider = None

# Pro (id(map), layer_idx) -> nativer TileCollider. Wie _SOLID_CACHE nie
# invalidiert -- Maps sind nach dem Load immutable.
_COLLIDER_CACHE: dict = {}


# --- Solid-Detection-Cache ---------------------------------------
# Pro (TiledMap, layer_idx) -> tuple(solid_aware: bool, solid_set: set[int]).
# `solid_aware` heisst: das Tileset hat irgendwo ein `solid`-Property
# gesetzt, also nutzen wir das. Sonst Fallback "alles != 0 ist solid".
# Cache invalidiert nie -- Maps sind nach dem Load immutable.
_SOLID_CACHE: dict = {}


def _solid_check(m: _TiledMap, gid: int) -> bool:
    """Ist die GID solid? Liest Tile-Properties, mit Fallback."""
    if gid <= 0:
        return False
    # Per-Tileset checken, ob "solid"-Property existiert; cache global.
    key = id(m)
    aware = _SOLID_CACHE.get(key)
    if aware is None:
        any_solid_prop = False
        for ts in m.tilesets:
            for props in ts.tile_properties.values():
                if "solid" in props:
                    any_solid_prop = True
                    break
            if any_solid_prop:
                break
        _SOLID_CACHE[key] = any_solid_prop
        aware = any_solid_prop
    if aware:
        # Property-Modus: nur Tiles mit explizitem solid=true sind solid.
        v = m.tile_property(gid, "solid")
        return bool(v)
    else:
        # Fallback: jeder gesetzte Tile in der Layer ist solid.
        return gid > 0


def _check_map_args(m, layer_idx, fn) -> tuple:
    if not isinstance(m, _TiledMap):
        raise TypeMismatchError(f"{fn} erwartet TILED_MAP")
    if isinstance(layer_idx, bool) or not isinstance(layer_idx, int):
        raise TypeMismatchError(f"{fn}: layer_idx muss INTEGER sein")
    if layer_idx < 0 or layer_idx >= len(m.layers):
        raise GBRuntimeError(
            f"{fn}: Layer-Index {layer_idx} ausserhalb [0..{len(m.layers) - 1}]"
        )
    layer = m.layers[layer_idx]
    if layer.type != "tile":
        raise GBRuntimeError(
            f"{fn}: Layer {layer_idx} ('{layer.name}') ist kein Tile-Layer"
        )
    return (m, layer)


def _check_num(v, fn, label):
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn}: {label} muss Zahl sein")
    return float(v)


def _tile_gid(layer: _TiledLayer, tx: int, ty: int) -> int:
    """Liefert die GID an Tile-Koordinate (tx, ty). 0 falls out-of-bounds."""
    if tx < 0 or ty < 0 or tx >= layer.width or ty >= layer.height:
        return 0
    return layer.tiles[ty * layer.width + tx]


def _tile_is_solid_at(m: _TiledMap, layer: _TiledLayer, tx: int, ty: int) -> bool:
    """Ist das Tile an (tx, ty) solid? Out-of-Bounds = solid (Welt-Rand
    blockiert) -- gleiche Konvention wie viele Tile-Engines."""
    if tx < 0 or ty < 0 or tx >= layer.width or ty >= layer.height:
        return True
    gid = layer.tiles[ty * layer.width + tx]
    return _solid_check(m, gid)


# --- Native-Collider (Rust) --------------------------------------

def _get_collider(m: _TiledMap, layer: _TiledLayer, layer_idx: int):
    """Liefert den nativen TileCollider fuer (map, layer) oder None, wenn
    das Rust-Backend fehlt. Die Solid-Maske wird einmal gebaut und gecacht."""
    if _NativeCollider is None:
        return None
    key = (id(m), layer_idx)
    c = _COLLIDER_CACHE.get(key)
    if c is None:
        solid = [_solid_check(m, gid) for gid in layer.tiles]
        c = _NativeCollider(
            layer.width, layer.height,
            float(m.tile_w), float(m.tile_h), solid,
        )
        _COLLIDER_CACHE[key] = c
    return c


# --- Sweep-Funktionen --------------------------------------------

def _sweep_axis(m: _TiledMap, layer: _TiledLayer,
                x: float, y: float, w: float, h: float,
                delta: float, axis: int) -> tuple:
    """Achsen-aligned Sweep.

    axis=0 -> X-Bewegung um delta; axis=1 -> Y-Bewegung um delta.
    Liefert (neue_pos, hit_flag).

    Algorithmus: gehe in Schritten von max 1-Tile-Breite, pruefe pro Schritt
    ob die zukuenftige Box mit einem soliden Tile ueberlappt. Bei Treffer:
    snap an die Tile-Grenze.
    """
    tw = m.tile_w
    th = m.tile_h
    if tw <= 0 or th <= 0:
        # Map hat keine Tile-Groesse -- keine Kollision moeglich
        if axis == 0:
            return (x + delta, False)
        return (y + delta, False)

    if delta == 0.0:
        return ((x if axis == 0 else y), False)

    if axis == 0:
        new_x = x + delta
        # Rechte/linke Box-Kante; Box ist (x, y, x+w, y+h)
        if delta > 0:
            # nach rechts: pruefe vordere Kante (x + w)
            front_edge = new_x + w
            tx_far = int(front_edge // tw)
            # Wenn die rechte Kante GENAU auf einer Tile-Grenze liegt
            # (front_edge % tw == 0), wollen wir das Tile an tx_far NICHT
            # mit pruefen, sonst kollidieren wir mit dem Tile rechts vom
            # Spalt. Klassisches floor/ceil-Problem.
            if front_edge == tx_far * tw:
                tx_far -= 1
            tx_old = int((x + w - 1) // tw) if x + w > 0 else -1
            # Y-Span (welche Tile-Reihen die Box ueberlappt)
            ty_top = int(y // th)
            ty_bot = int((y + h - 1) // th)
            for tx in range(tx_old + 1, tx_far + 1):
                for ty in range(ty_top, ty_bot + 1):
                    if _tile_is_solid_at(m, layer, tx, ty):
                        # Snap: rechte Box-Kante an linke Kante des Tiles
                        new_x = float(tx * tw) - w
                        return (new_x, True)
            return (new_x, False)
        else:
            # nach links: pruefe vordere Kante (x)
            front_edge = new_x
            tx_far = int(front_edge // tw)
            tx_old = int(x // tw)
            ty_top = int(y // th)
            ty_bot = int((y + h - 1) // th)
            for tx in range(tx_old - 1, tx_far - 1, -1):
                for ty in range(ty_top, ty_bot + 1):
                    if _tile_is_solid_at(m, layer, tx, ty):
                        # Snap: linke Box-Kante an rechte Kante des Tiles
                        new_x = float((tx + 1) * tw)
                        return (new_x, True)
            return (new_x, False)
    else:
        new_y = y + delta
        if delta > 0:
            front_edge = new_y + h
            ty_far = int(front_edge // th)
            if front_edge == ty_far * th:
                ty_far -= 1
            ty_old = int((y + h - 1) // th) if y + h > 0 else -1
            tx_left = int(x // tw)
            tx_right = int((x + w - 1) // tw)
            for ty in range(ty_old + 1, ty_far + 1):
                for tx in range(tx_left, tx_right + 1):
                    if _tile_is_solid_at(m, layer, tx, ty):
                        new_y = float(ty * th) - h
                        return (new_y, True)
            return (new_y, False)
        else:
            front_edge = new_y
            ty_far = int(front_edge // th)
            ty_old = int(y // th)
            tx_left = int(x // tw)
            tx_right = int((x + w - 1) // tw)
            for ty in range(ty_old - 1, ty_far - 1, -1):
                for tx in range(tx_left, tx_right + 1):
                    if _tile_is_solid_at(m, layer, tx, ty):
                        new_y = float((ty + 1) * th)
                        return (new_y, True)
            return (new_y, False)


@builtin("TILE_SWEEP_X", arity=7,
         types=("any", "int", "num", "num", "num", "num", "num"))
def _b_sweep_x(m, layer_idx, x, y, w, h, dx):
    """Bewegt eine Box (x, y, w, h) um `dx` entlang der X-Achse und stoppt
    am ersten soliden Tile im genannten Layer. Liefert TUPLE (new_x, hit).
    """
    m, layer = _check_map_args(m, layer_idx, "TILE_SWEEP_X")
    fx = _check_num(x, "TILE_SWEEP_X", "x")
    fy = _check_num(y, "TILE_SWEEP_X", "y")
    fw = _check_num(w, "TILE_SWEEP_X", "w")
    fh = _check_num(h, "TILE_SWEEP_X", "h")
    fdx = _check_num(dx, "TILE_SWEEP_X", "dx")
    if fw <= 0 or fh <= 0:
        raise GBRuntimeError("TILE_SWEEP_X: w und h muessen > 0 sein")
    c = _get_collider(m, layer, layer_idx)
    if c is not None:
        return c.sweep_x(fx, fy, fw, fh, fdx)
    return _sweep_axis(m, layer, fx, fy, fw, fh, fdx, axis=0)


@builtin("TILE_SWEEP_Y", arity=7,
         types=("any", "int", "num", "num", "num", "num", "num"))
def _b_sweep_y(m, layer_idx, x, y, w, h, dy):
    """Bewegt eine Box entlang der Y-Achse und stoppt am ersten soliden
    Tile. Liefert TUPLE (new_y, hit).
    Konvention: dy > 0 = nach unten (= Gravitation in Screen-Coords)."""
    m, layer = _check_map_args(m, layer_idx, "TILE_SWEEP_Y")
    fx = _check_num(x, "TILE_SWEEP_Y", "x")
    fy = _check_num(y, "TILE_SWEEP_Y", "y")
    fw = _check_num(w, "TILE_SWEEP_Y", "w")
    fh = _check_num(h, "TILE_SWEEP_Y", "h")
    fdy = _check_num(dy, "TILE_SWEEP_Y", "dy")
    if fw <= 0 or fh <= 0:
        raise GBRuntimeError("TILE_SWEEP_Y: w und h muessen > 0 sein")
    c = _get_collider(m, layer, layer_idx)
    if c is not None:
        return c.sweep_y(fx, fy, fw, fh, fdy)
    return _sweep_axis(m, layer, fx, fy, fw, fh, fdy, axis=1)


@builtin("TILE_IS_SOLID", arity=4, types=("any", "int", "int", "int"))
def _b_is_solid(m, layer_idx, tx, ty):
    """Ist das Tile an (tx, ty) im Layer solid? Out-of-Bounds liefert TRUE
    (Welt-Rand blockiert)."""
    m, layer = _check_map_args(m, layer_idx, "TILE_IS_SOLID")
    return _tile_is_solid_at(m, layer, tx, ty)


@builtin("TILE_AT_PIXEL", arity=4, types=("any", "int", "num", "num"))
def _b_at_pixel(m, layer_idx, px, py):
    """Liefert das GID an Pixel-Position (px, py). 0 wenn out-of-bounds.
    Praktisch fuer ad-hoc-Punkt-Tests jenseits des Box-Sweeps."""
    m, layer = _check_map_args(m, layer_idx, "TILE_AT_PIXEL")
    fpx = _check_num(px, "TILE_AT_PIXEL", "px")
    fpy = _check_num(py, "TILE_AT_PIXEL", "py")
    tw = m.tile_w
    th = m.tile_h
    if tw <= 0 or th <= 0:
        return 0
    tx = int(fpx // tw)
    ty = int(fpy // th)
    return _tile_gid(layer, tx, ty)
