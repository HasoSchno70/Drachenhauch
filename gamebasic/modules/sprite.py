"""Sprite-Modul fuer GameBasic.

Ein Sprite ist ein animiertes Bild mit Position, Geschwindigkeit und
benannten Animationen. Frames werden aus einem Sheet (IMAGE mit
gerasterten Sub-Bildern) entnommen.

Built-ins (Lifecycle):
    SPRITE_NEW(image, frame_w, frame_h)  -> SPRITE
    SPRITE_SET_POS(sp, x, y)
    SPRITE_SET_VELOCITY(sp, vx, vy)
    SPRITE_GET_X(sp), SPRITE_GET_Y(sp)            -> FLOAT
    SPRITE_GET_WIDTH(sp), SPRITE_GET_HEIGHT(sp)   -> INTEGER
    SPRITE_SET_FLIP(sp, flipX, flipY)

Animationen:
    SPRITE_ADD_ANIM(sp, name$, first, last, fps)
    SPRITE_PLAY(sp, name$)            ' looping
    SPRITE_PLAY_ONCE(sp, name$)       ' einmal abspielen
    SPRITE_CURRENT_ANIM(sp)           -> STRING
    SPRITE_IS_FINISHED(sp)            -> BOOLEAN
    SPRITE_SET_FRAME(sp, idx)
    SPRITE_GET_FRAME(sp)              -> INTEGER

Game-Loop:
    SPRITE_UPDATE(sp, dt_ms)          ' Position + Animation
    SPRITE_DRAW(sp)                   ' Camera-aware via graphics

Kollision:
    SPRITE_COLLIDES(sp1, sp2)         -> BOOLEAN  (AABB)

Geschwindigkeit ist Pixel/Sekunde (vx, vy). FPS ist Frames pro Sekunde.
"""
from __future__ import annotations

from ..builtins_registry import builtin, graphics_builtin
from ..errors import GBRuntimeError, TypeMismatchError
from ..interpreter import _Image
from . import register_type


class _Sprite:
    __slots__ = (
        "image", "frame_w", "frame_h",
        "x", "y", "vx", "vy",
        "anims",            # name -> (first, last, fps)
        "current_anim",     # str ("" wenn keine)
        "current_frame",    # int (Index ins Sheet)
        "anim_time_ms",     # ms in der aktuellen Anim
        "looping",
        "finished",
        "flip_x", "flip_y",
        "scale_x", "scale_y",
        "tinted", "tint_color",
    )

    def __init__(self, image: _Image, frame_w: int, frame_h: int):
        self.image = image
        self.frame_w = frame_w
        self.frame_h = frame_h
        self.x = 0.0
        self.y = 0.0
        self.vx = 0.0
        self.vy = 0.0
        self.anims: dict = {}
        self.current_anim = ""
        self.current_frame = 0
        self.anim_time_ms = 0
        self.looping = True
        self.finished = False
        self.flip_x = False
        self.flip_y = False
        self.scale_x = 1.0
        self.scale_y = 1.0
        self.tinted = False
        self.tint_color = 0xFFFFFF

    def __repr__(self):
        return (f"<SPRITE @({self.x:.0f},{self.y:.0f}) "
                f"frame={self.current_frame} anim='{self.current_anim}'>")

    def play(self, name: str, looping: bool):
        if name not in self.anims:
            raise GBRuntimeError(
                f"SPRITE: unbekannte Animation '{name}' "
                f"(definiert: {', '.join(sorted(self.anims.keys())) or 'keine'})"
            )
        if name == self.current_anim and self.looping == looping and not self.finished:
            return  # idempotent
        self.current_anim = name
        first, _, _ = self.anims[name]
        self.current_frame = first
        self.anim_time_ms = 0
        self.looping = looping
        self.finished = False

    def update(self, dt_ms: int) -> None:
        dt = dt_ms / 1000.0
        self.x += self.vx * dt
        self.y += self.vy * dt
        if not self.current_anim or self.finished:
            return
        first, last, fps = self.anims[self.current_anim]
        if fps <= 0:
            return
        self.anim_time_ms += dt_ms
        frame_duration = 1000.0 / fps
        elapsed_frames = int(self.anim_time_ms / frame_duration)
        n_frames = last - first + 1
        if self.looping:
            self.current_frame = first + (elapsed_frames % n_frames)
        else:
            if elapsed_frames >= n_frames:
                self.current_frame = last
                self.finished = True
            else:
                self.current_frame = first + elapsed_frames


register_type("sprite", _Sprite)


# --- Helfer ----------------------------------------------------------

def _check_sprite(v, fn: str) -> _Sprite:
    if not isinstance(v, _Sprite):
        raise TypeMismatchError(f"{fn} erwartet SPRITE")
    return v


def _check_image(v, fn: str) -> _Image:
    if not isinstance(v, _Image):
        raise TypeMismatchError(f"{fn} erwartet IMAGE als 1. Argument")
    return v


def _check_num(v, fn: str, name: str) -> float:
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn}: {name} muss Zahl sein")
    return float(v)


def _check_int(v, fn: str, name: str) -> int:
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn}: {name} muss INTEGER sein")
    return v


def _import_pygame():
    """SPRITE_DRAW (Rendering) laeuft nur in der nativen Runtime (gbrt). Die
    Sprite-Logik (Position/Animation/Kollision) ist davon unabhaengig und
    funktioniert auch im konsolen-only Tree-Walker."""
    raise GBRuntimeError(
        "SPRITE_DRAW (Sprite-Rendering) laeuft nur in der nativen Runtime "
        "(gbrt) -- per F6 bzw. 'gbrun.py --native' starten.")


# --- Lifecycle -------------------------------------------------------

@builtin("SPRITE_NEW", arity=3)
def _new(image, fw, fh):
    image = _check_image(image, "SPRITE_NEW")
    fw = _check_int(fw, "SPRITE_NEW", "frame_w")
    fh = _check_int(fh, "SPRITE_NEW", "frame_h")
    if fw <= 0 or fh <= 0:
        raise GBRuntimeError("SPRITE_NEW: frame_w und frame_h muessen > 0 sein")
    return _Sprite(image, fw, fh)


@builtin("SPRITE_SET_POS", arity=3)
def _set_pos(sp, x, y):
    sp = _check_sprite(sp, "SPRITE_SET_POS")
    sp.x = _check_num(x, "SPRITE_SET_POS", "x")
    sp.y = _check_num(y, "SPRITE_SET_POS", "y")
    return None


@builtin("SPRITE_SET_VELOCITY", arity=3)
def _set_vel(sp, vx, vy):
    sp = _check_sprite(sp, "SPRITE_SET_VELOCITY")
    sp.vx = _check_num(vx, "SPRITE_SET_VELOCITY", "vx")
    sp.vy = _check_num(vy, "SPRITE_SET_VELOCITY", "vy")
    return None


@builtin("SPRITE_GET_X", arity=1)
def _get_x(sp):
    return _check_sprite(sp, "SPRITE_GET_X").x


@builtin("SPRITE_GET_Y", arity=1)
def _get_y(sp):
    return _check_sprite(sp, "SPRITE_GET_Y").y


@builtin("SPRITE_GET_WIDTH", arity=1)
def _get_w(sp):
    return _check_sprite(sp, "SPRITE_GET_WIDTH").frame_w


@builtin("SPRITE_GET_HEIGHT", arity=1)
def _get_h(sp):
    return _check_sprite(sp, "SPRITE_GET_HEIGHT").frame_h


@builtin("SPRITE_SET_FLIP", arity=3, types=("any", "bool", "bool"))
def _set_flip(sp, fx, fy):
    sp = _check_sprite(sp, "SPRITE_SET_FLIP")
    sp.flip_x = fx
    sp.flip_y = fy
    return None


@builtin("SPRITE_SET_SCALE", arity=3)
def _set_scale(sp, sx, sy):
    """Skaliert das Sprite beim Zeichnen. GET_WIDTH/HEIGHT und Kollision
    bleiben in der originalen Frame-Groesse - Scale ist rein visuell."""
    sp = _check_sprite(sp, "SPRITE_SET_SCALE")
    sp.scale_x = _check_num(sx, "SPRITE_SET_SCALE", "sx")
    sp.scale_y = _check_num(sy, "SPRITE_SET_SCALE", "sy")
    if sp.scale_x <= 0 or sp.scale_y <= 0:
        raise GBRuntimeError("SPRITE_SET_SCALE: Skalierung muss > 0 sein")
    return None


@builtin("SPRITE_TINT", arity=2, types=("any", "int"))
def _tint(sp, color):
    """Multiplikativer Farb-Tint (RGB-MULT). 0xFFFFFF = neutral, 0x000000 = schwarz."""
    sp = _check_sprite(sp, "SPRITE_TINT")
    if color < 0 or color > 0xFFFFFF:
        raise GBRuntimeError("SPRITE_TINT: Farbe muss 0..0xFFFFFF sein")
    sp.tinted = True
    sp.tint_color = color
    return None


@builtin("SPRITE_TINT_CLEAR", arity=1)
def _tint_clear(sp):
    sp = _check_sprite(sp, "SPRITE_TINT_CLEAR")
    sp.tinted = False
    sp.tint_color = 0xFFFFFF
    return None


# --- Animationen -----------------------------------------------------

@builtin("SPRITE_ADD_ANIM", arity=5)
def _add_anim(sp, name, first, last, fps):
    sp = _check_sprite(sp, "SPRITE_ADD_ANIM")
    if not isinstance(name, str) or not name:
        raise TypeMismatchError("SPRITE_ADD_ANIM: name muss nicht-leerer STRING sein")
    first = _check_int(first, "SPRITE_ADD_ANIM", "first")
    last = _check_int(last, "SPRITE_ADD_ANIM", "last")
    fps = _check_num(fps, "SPRITE_ADD_ANIM", "fps")
    if first < 0 or last < first:
        raise GBRuntimeError(
            "SPRITE_ADD_ANIM: first >= 0 und last >= first noetig"
        )
    if fps < 0:
        raise GBRuntimeError("SPRITE_ADD_ANIM: fps muss >= 0 sein")
    sp.anims[name] = (first, last, fps)
    return None


@builtin("SPRITE_PLAY", arity=2, types=("any", "str"))
def _play(sp, name):
    _check_sprite(sp, "SPRITE_PLAY").play(name, looping=True)
    return None


@builtin("SPRITE_PLAY_ONCE", arity=2, types=("any", "str"))
def _play_once(sp, name):
    _check_sprite(sp, "SPRITE_PLAY_ONCE").play(name, looping=False)
    return None


@builtin("SPRITE_CURRENT_ANIM", arity=1)
def _current_anim(sp):
    return _check_sprite(sp, "SPRITE_CURRENT_ANIM").current_anim


@builtin("SPRITE_IS_FINISHED", arity=1)
def _is_finished(sp):
    return _check_sprite(sp, "SPRITE_IS_FINISHED").finished


@builtin("SPRITE_SET_FRAME", arity=2, types=("any", "int"))
def _set_frame(sp, idx):
    sp = _check_sprite(sp, "SPRITE_SET_FRAME")
    if idx < 0:
        raise GBRuntimeError("SPRITE_SET_FRAME: idx muss >= 0 sein")
    sp.current_frame = idx
    return None


@builtin("SPRITE_GET_FRAME", arity=1)
def _get_frame(sp):
    return _check_sprite(sp, "SPRITE_GET_FRAME").current_frame


# --- Game-Loop -------------------------------------------------------

@builtin("SPRITE_UPDATE", arity=2, types=("any", "int"))
def _update(sp, dt_ms):
    sp = _check_sprite(sp, "SPRITE_UPDATE")
    if dt_ms < 0:
        raise GBRuntimeError("SPRITE_UPDATE: dt_ms muss >= 0 sein")
    sp.update(dt_ms)
    return None


@graphics_builtin("SPRITE_DRAW", arity=1)
def _draw(g, sp):
    _check_sprite(sp, "SPRITE_DRAW")
    _import_pygame()   # native-only -> wirft im Tree-Walker


# --- Kollision -------------------------------------------------------

@builtin("SPRITE_COLLIDES", arity=2)
def _collides(a, b):
    a = _check_sprite(a, "SPRITE_COLLIDES")
    b = _check_sprite(b, "SPRITE_COLLIDES")
    return (a.x < b.x + b.frame_w
            and a.x + a.frame_w > b.x
            and a.y < b.y + b.frame_h
            and a.y + a.frame_h > b.y)


@builtin("SPRITE_COLLIDE", arity=2)
def _collide_alias(a, b):
    """Singular-Alias zu SPRITE_COLLIDES - praktischer beim Tippen."""
    a = _check_sprite(a, "SPRITE_COLLIDE")
    b = _check_sprite(b, "SPRITE_COLLIDE")
    return (a.x < b.x + b.frame_w
            and a.x + a.frame_w > b.x
            and a.y < b.y + b.frame_h
            and a.y + a.frame_h > b.y)


@builtin("SPRITE_HIT_BOX", arity=5, types=("any", "intish", "intish", "intish", "intish"))
def _hit_box(sp, x, y, w, h):
    """Kollidiert das Sprite mit einer freistehenden Box (x,y,w,h)?

    Gut fuer:
    - Wand-/Hindernis-Pruefung in einer Tilemap (eine Tile = eine Box)
    - Trigger-Zonen ohne eigenes Sprite (Door, Pickup-Area)
    - HUD-Overlap bei Maus-Position (kombiniert mit MOUSEX/MOUSEY)
    """
    sp = _check_sprite(sp, "SPRITE_HIT_BOX")
    return (sp.x < x + w
            and sp.x + sp.frame_w > x
            and sp.y < y + h
            and sp.y + sp.frame_h > y)


@builtin("SPRITE_HIT_POINT", arity=3, types=("any", "intish", "intish"))
def _hit_point(sp, x, y):
    """Liegt der Punkt (x,y) innerhalb der Sprite-Bounding-Box?

    Klassischer Use: Klick-Detection mit MOUSEX/MOUSEY.
        IF SPRITE_HIT_POINT(button, MOUSEX(), MOUSEY()) THEN ...
    """
    sp = _check_sprite(sp, "SPRITE_HIT_POINT")
    return (sp.x <= x < sp.x + sp.frame_w
            and sp.y <= y < sp.y + sp.frame_h)
