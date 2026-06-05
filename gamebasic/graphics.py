"""Pygame-Backend fuer GameBasic-Grafik.

Faerben werden als 24-Bit-RGB-INTEGERs uebergeben (0xRRGGBB). Das Modul stellt
benannte Konstanten (BLACK, WHITE, RED, ...) und eine RGB(r,g,b)-Funktion bereit;
beide liefern denselben Encoding-Stil, der direkt von den Zeichenbefehlen verstanden
wird.

Pygame wird beim ersten SCREEN-Aufruf gestartet (Lazy-Init). Wer kein SCREEN
benutzt, bezahlt keinen Pygame-Import.
"""
from .errors import GBRuntimeError


# --- Farben (RGB-Hex) -------------------------------------------------
COLORS = {
    "black":      0x000000,
    "white":      0xFFFFFF,
    "gray":       0x808080,
    "lightgray":  0xC0C0C0,
    "darkgray":   0x404040,
    "red":        0xFF0000,
    "green":      0x00FF00,
    "blue":       0x0000FF,
    "yellow":     0xFFFF00,
    "cyan":       0x00FFFF,
    "magenta":    0xFF00FF,
    "orange":     0xFFA500,
    "purple":     0x800080,
    "brown":      0x8B4513,
    "pink":       0xFFC0CB,
    "darkred":    0x800000,
    "darkgreen":  0x008000,
    "darkblue":   0x000080,
}

# --- Tastencodes (SDL2-Keycodes, kein pygame-Import noetig) -----------
KEYS = {
    "key_escape":   27,
    "key_return":   13,
    "key_enter":    13,
    "key_space":    32,
    "key_tab":      9,
    "key_backspace": 8,
    "key_left":     1073741904,
    "key_right":    1073741903,
    "key_up":       1073741906,
    "key_down":     1073741905,
}
# Buchstaben (lowercase 97..122 -> KEY_A..KEY_Z)
for _ord in range(26):
    KEYS[f"key_{chr(ord('a') + _ord)}"] = ord('a') + _ord
# Ziffern (48..57 -> KEY_0..KEY_9)
for _d in range(10):
    KEYS[f"key_{_d}"] = ord('0') + _d
# Funktionstasten F1..F12 (SDL2-Keycodes liegen bei 1073741882 + (n-1))
for _fn in range(1, 13):
    KEYS[f"key_f{_fn}"] = 1073741882 + (_fn - 1)

# Gamepad/Joystick-Konstanten. Negative Codes -- so kollidieren sie nicht
# mit Tastatur-Keycodes (alle positiv). INPUT_BIND akzeptiert beide; das
# input-Modul unterscheidet intern.
# Buttons (Xbox-Layout, pygame.joystick standard mapping):
KEYS["joy_button_a"]      = -100   # A (Xbox) / X (PS)
KEYS["joy_button_b"]      = -101   # B (Xbox) / Circle (PS)
KEYS["joy_button_x"]      = -102   # X (Xbox) / Square (PS)
KEYS["joy_button_y"]      = -103   # Y (Xbox) / Triangle (PS)
KEYS["joy_button_lb"]     = -104   # Left Bumper / L1
KEYS["joy_button_rb"]     = -105   # Right Bumper / R1
KEYS["joy_button_back"]   = -106   # Back / Select / Share
KEYS["joy_button_start"]  = -107   # Start / Options
KEYS["joy_button_lstick"] = -108   # Linker Stick-Klick (L3)
KEYS["joy_button_rstick"] = -109   # Rechter Stick-Klick (R3)
# DPad (intern als "Hat" in pygame, hier als Buttons exponiert)
KEYS["joy_dpad_up"]       = -200
KEYS["joy_dpad_down"]     = -201
KEYS["joy_dpad_left"]     = -202
KEYS["joy_dpad_right"]    = -203


def _rgb_tuple(color: int):
    c = int(color) & 0xFFFFFF
    return ((c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF)


class _Layer:
    """Ein Z-Layer im Compose-Stack. Surface wird lazy beim ersten Use
    allokiert (SCREEN muss da sein, sonst wissen wir die Groesse nicht)."""
    __slots__ = ("name", "z", "surface")

    def __init__(self, name: str, z: int, surface):
        self.name = name
        self.z = z
        self.surface = surface  # pygame.Surface oder None

    def __repr__(self):
        return f"<Layer {self.name!r} z={self.z}>"


class Graphics:
    """Lazy-initialisierter Pygame-Wrapper."""

    def __init__(self):
        self._pygame = None
        self._screen = None      # Display-Surface (sichtbar, ggf. skaliert)
        self._buffer = None      # AKTIVES Draw-Target: zeigt entweder auf
                                 # _main_buffer ODER auf eine Layer-Surface.
                                 # Alle draw-Methoden zeichnen auf _buffer --
                                 # damit funktioniert Layer-Switching ohne dass
                                 # jede draw-Methode geaendert werden muss.
        self._main_buffer = None # Original-Buffer (Compose-Ziel beim FLIP)
        self._scale = 1          # Faktor: Display = Buffer * scale
        self._buf_size = (0, 0)
        # Render-Targets (RENDERTARGET_*): Off-Screen-Surfaces. BEGIN lenkt
        # _buffer auf das Target um (Stack fuer Verschachtelung), END stellt
        # den vorigen Draw-Target wieder her.
        self._render_targets = []
        self._rt_stack = []
        # Z-Layer-State. Wenn der User LAYER("name") aufruft, wird
        # _buffer auf die Layer-Surface umgeleitet. FLIP composited
        # alle Layer in z-Order auf _main_buffer und cleart sie.
        # Wenn keine Layer benutzt werden, ist _current_layer None und
        # _buffer == _main_buffer -- Backwards-Compat zur Direct-Mode.
        self._layers: dict = {}            # name -> _Layer
        self._layer_order: list = []       # nach z aufsteigend
        self._current_layer = None         # _Layer oder None
        self._clock = None
        # Font-State: aktuelle Groesse + Stil. Cache pro (size, bold, italic)
        # damit wiederholte TEXT-Calls nicht jedes mal SysFont rufen.
        self._font_size = 20
        self._font_bold = False
        self._font_italic = False
        self._font_cache: dict = {}
        # TTF-Fonts (LOADFONT): Liste von (pfad, basis_groesse). Handle = Index.
        # _active_font = -1 -> Default-SysFont; sonst TTF aus _fonts[idx].
        # _text_spacing wirkt nur native (raylib DrawTextEx); pygame ignoriert es.
        self._fonts: list = []
        self._active_font = -1
        self._text_spacing = 0
        # Asset-Caches: key (abs-Path ODER Alias) -> Surface bzw. Sound.
        # LOADIMAGE / LOADSOUND pruefen diese vor dem File-IO. LOAD_ASSETS
        # nutzt sie zum Pre-Warmen. Aliase werden beim Manifest-Load
        # registriert -- LOADIMAGE("player") trifft den Cache, wenn das
        # Manifest "player" als Alias hatte.
        self._image_cache: dict = {}
        self._sound_cache: dict = {}
        # Sprite-Batch-Queue: Liste von (surface, dest_pos, src_rect)-Tupeln.
        # ATLAS_DRAW_BATCH haengt an, BATCH_FLUSH ruft pygame.Surface.blits()
        # mit der ganzen Liste auf einmal -- spart Python-Call-Overhead bei
        # hunderten von Sprites pro Frame (Tilemap, Bullet-Hell, etc.).
        self._batch: list = []
        self._keys: set = set()
        # Text-Input: pygame liefert pro Tastendruck ein ev.unicode mit dem
        # tatsaechlichen Zeichen (Layout/Shift/Dead-Keys werden vom OS
        # aufgeloest). Wir akkumulieren das hier und das UI-Modul drained
        # den Buffer einmal pro Frame.
        self._text_input: list = []
        # FIFO-Queue der gedrueckten Keycodes (auch Special-Keys wie ESC,
        # Pfeile, Enter). INKEY$/WAITKEY drainen hier.
        self._key_queue: list = []
        self._closed = False
        self._mouse_x = 0
        self._mouse_y = 0
        self._mouse_buttons = (False, False, False)
        # Mausrad-Akkumulator: pygame liefert MOUSEWHEEL-Events mit ev.y
        # (positiv = nach oben gescrollt). Wir akkumulieren und drainen,
        # damit kein Event verloren geht selbst wenn der UI-Code nicht in
        # jedem Frame liest.
        self._mouse_wheel_y: int = 0
        # Echter Clip-Stack: jedes push_clip schiebt das vorherige Rect drauf,
        # pop_clip stellt es wieder her. Ohne Stack wuerde der innere
        # pop_clip den aeusseren Clip ebenfalls aufheben (Bug fixed).
        self._clip_stack: list = []
        # Joystick-Handles (lazy): erst beim ersten Zugriff initialisieren,
        # damit Programme ohne Gamepad keinen Pygame-Joystick-Init brauchen.
        self._joysticks: list = []
        self._joystick_init_done = False
        # Programmstart-Referenz fuer TIMER() (Sekunden seit erstem Zugriff).
        self._timer_start: float | None = None
        # Camera (in World-Koordinaten). Zeichenbefehle interpretieren ihre
        # Argumente als World-Koordinaten und uebersetzen sie hier in
        # Screen-Pixel. Default = identitaet.
        self._cam_x = 0.0
        self._cam_y = 0.0
        self._cam_zoom = 1.0
        # Fullscreen-State; wird in screen() bzw. set_fullscreen() gesetzt.
        self._fullscreen = False

    # ---- Camera ------------------------------------------------------
    def set_camera(self, x: float, y: float, zoom: float = 1.0):
        if zoom <= 0:
            raise GBRuntimeError("CAMERA: zoom muss > 0 sein")
        self._cam_x = float(x)
        self._cam_y = float(y)
        self._cam_zoom = float(zoom)

    def reset_camera(self):
        self._cam_x = 0.0
        self._cam_y = 0.0
        self._cam_zoom = 1.0

    def get_camera(self):
        return (self._cam_x, self._cam_y, self._cam_zoom)

    def _w2s(self, x, y):
        """World -> Screen (int Pixel)."""
        sx = (x - self._cam_x) * self._cam_zoom
        sy = (y - self._cam_y) * self._cam_zoom
        return int(sx), int(sy)

    def _s2w(self, sx, sy):
        """Screen -> World (float)."""
        if self._cam_zoom == 0:
            return float(sx), float(sy)
        wx = sx / self._cam_zoom + self._cam_x
        wy = sy / self._cam_zoom + self._cam_y
        return wx, wy

    def _scale_size(self, s):
        return max(0, int(s * self._cam_zoom))

    # ---- Lifecycle ---------------------------------------------------
    def _ensure_pygame(self):
        if self._pygame is not None:
            return
        # Pygame's Hello-Banner unterdruecken - die Ausgabe stoert in
        # CLI-Programmen und macht --bench-Vergleiche unzuverlaessig.
        import os as _os
        _os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "hide")
        try:
            import pygame
        except ImportError as exc:
            raise GBRuntimeError(
                "Modul 'pygame' nicht installiert. "
                "Installation: 'py -m pip install pygame' "
                "oder in PyCharm ueber Settings -> Project -> Python Interpreter."
            ) from exc
        pygame.init()
        self._pygame = pygame
        self._clock = pygame.time.Clock()
        self._target_fps = 60      # SETFPS aenderbar; FLIP taktet darauf
        self._delta = 0.0          # Sekunden seit dem letzten FLIP (DELTA())

    def _require_screen(self):
        self._ensure_pygame()
        if self._screen is None:
            raise GBRuntimeError(
                "SCREEN(breite, hoehe) muss zuerst aufgerufen werden"
            )

    def shutdown(self):
        if self._pygame is not None:
            try:
                self._pygame.quit()
            except Exception:
                pass

    # ---- Befehle -----------------------------------------------------
    def screen(self, width: int, height: int, title: str = "GameBasic",
               scale: int = 1, fullscreen: bool = False):
        """Oeffnet das Fenster. scale > 1 macht jeden logischen Pixel groesser
        (jeder Befehl rechnet in logischen Pixeln, das Fenster ist scale-fach
        groesser). Toll fuer Retro-Look bei Jump'n'Run mit niedriger Aufloesung.
        Wenn fullscreen=True, oeffnet pygame im Vollbild und der scale-fache
        Buffer wird auf die Monitor-Aufloesung gestretcht.
        """
        self._ensure_pygame()
        pg = self._pygame
        if scale < 1:
            scale = 1
        self._scale = int(scale)
        w = int(width)
        h = int(height)
        self._buf_size = (w, h)
        self._fullscreen = bool(fullscreen)
        flags = pg.FULLSCREEN if self._fullscreen else 0
        # Sichtbares Fenster ist scale-fach groesser (oder Monitor-Aufloesung
        # bei Fullscreen - pygame waehlt den naechsten passenden Mode).
        self._screen = pg.display.set_mode(
            (w * self._scale, h * self._scale), flags)
        pg.display.set_caption(str(title))
        # Zeichnen passiert weiterhin in logischer Aufloesung. _main_buffer
        # ist das Compose-Target; _buffer ist initial dasselbe Objekt.
        self._main_buffer = pg.Surface((w, h))
        self._main_buffer.fill((0, 0, 0))
        self._buffer = self._main_buffer
        # Vorhandene Layer auf die neue Aufloesung anpassen (falls SCREEN
        # nach LAYER_DEFINE aufgerufen wird).
        for lyr in self._layers.values():
            lyr.surface = pg.Surface((w, h), pg.SRCALPHA)
        self._current_layer = None
        if self._scale == 1 and not self._fullscreen:
            self._screen.blit(self._main_buffer, (0, 0))
        else:
            scaled = pg.transform.scale(self._main_buffer, self._screen.get_size())
            self._screen.blit(scaled, (0, 0))

    def delta(self) -> float:
        """Sekunden seit dem letzten FLIP (fuer framerate-unabhaengige Bewegung)."""
        return float(getattr(self, "_delta", 0.0))

    def fps(self) -> int:
        """Aktuelle Bilder/Sekunde (gleitend)."""
        self._ensure_pygame()
        return int(round(self._clock.get_fps()))

    def set_target_fps(self, n: int):
        """Ziel-Framerate fuer FLIP (0 = ungedrosselt)."""
        self._target_fps = max(0, int(n))

    def save_screenshot(self, path: str):
        """Speichert den aktuellen Frame als Bild (PNG/JPG je nach Endung)."""
        self._require_screen()
        self._pygame.image.save(self._screen, str(path))

    def set_window_title(self, title: str):
        self._ensure_pygame()
        self._pygame.display.set_caption(str(title))

    # --- Shader / Post-Processing: GPU-only, im pygame-Pfad No-Op ----------
    def load_shader(self, src: str) -> int:
        return -1   # pygame hat keine GLSL-Pipeline -> kein Shader

    def shader_set_float(self, h, name, v):
        return None

    def shader_set_vec2(self, h, name, x, y):
        return None

    def shader_set_vec3(self, h, name, x, y, z):
        return None

    def set_postfx(self, h):
        return None

    def set_fullscreen(self, fullscreen: bool):
        """Toggle Fullscreen zur Laufzeit - der logische Buffer bleibt gleich,
        nur das Display-Surface wird neu erzeugt."""
        self._require_screen()
        pg = self._pygame
        new_fs = bool(fullscreen)
        if new_fs == getattr(self, "_fullscreen", False):
            return
        self._fullscreen = new_fs
        flags = pg.FULLSCREEN if new_fs else 0
        w, h = self._buf_size
        self._screen = pg.display.set_mode(
            (w * self._scale, h * self._scale), flags)
        pg.display.flip()
        self._closed = False
        self._pump()

    def cls(self, color: int = 0x000000):
        self._require_screen()
        self._buffer.fill(_rgb_tuple(color))

    # ---- Z-Layer-System ---------------------------------------------
    # Layers sind named Compose-Surfaces mit explizitem z-Wert. Alle
    # draw-Methoden zeichnen auf self._buffer; LAYER("foo") redirected
    # self._buffer auf die Layer-Surface. FLIP composiert in z-Order
    # auf _main_buffer und blittet wie gehabt.
    #
    # Typischer Game-Loop:
    #   LAYER_DEFINE("bg", 0)
    #   LAYER_DEFINE("sprites", 10)
    #   LAYER_DEFINE("ui", 100)
    #   ...
    #   LAYER("bg");      CLS(SKY); DRAWIMAGE(parallax, 0, 0)
    #   LAYER("sprites"); DRAWIMAGE(player, x, y)
    #   LAYER("ui");      TEXT(10, 10, "Score: 100")
    #   FLIP()  -- composit + clear-fuer-naechsten-Frame

    def _ensure_layer_surface(self, lyr):
        """Lazy-Allokation der Layer-Surface. Verlangt SCREEN."""
        if lyr.surface is not None:
            return
        self._require_screen()
        pg = self._pygame
        lyr.surface = pg.Surface(self._buf_size, pg.SRCALPHA)

    def layer_define(self, name: str, z: int):
        """Registriert (oder aktualisiert) einen Layer mit z-Order. Bereits
        existierender Layer kriegt nur ein neues z."""
        if name in self._layers:
            self._layers[name].z = int(z)
        else:
            self._layers[name] = _Layer(name, int(z), None)
        # Order nach z neu aufbauen (stabil, name-sekundaer fuer determinism)
        self._layer_order = sorted(
            self._layers.keys(),
            key=lambda n: (self._layers[n].z, n),
        )

    def layer_use(self, name: str):
        """Switcht den aktiven Draw-Target auf den genannten Layer. Falls
        nicht definiert: auto-create mit dem naechsthoeheren z (sodass
        Reihenfolge der ersten Verwendung der z-Order entspricht)."""
        self._require_screen()
        # Pending Batch auf dem alten Target flushen, sonst landen die
        # gequeueten Sprites auf dem NEUEN Layer (falsche Reihenfolge).
        if self._batch:
            self._flush_batch_now()
        lyr = self._layers.get(name)
        if lyr is None:
            next_z = (max((l.z for l in self._layers.values()), default=-1) + 1)
            self.layer_define(name, next_z)
            lyr = self._layers[name]
        self._ensure_layer_surface(lyr)
        self._current_layer = lyr
        self._buffer = lyr.surface

    def layer_end(self):
        """Verlaesst den Layer-Kontext -- nachfolgende draw-Calls gehen
        direkt auf den Main-Buffer (ohne Layer-Composite-Latenz). Wird
        beim FLIP automatisch aufgerufen, falls vergessen."""
        if self._batch:
            self._flush_batch_now()
        self._current_layer = None
        self._buffer = self._main_buffer

    def layer_clear(self, name: str):
        """Cleart einen Layer manuell (transparent). Selten gebraucht --
        FLIP cleart alle Layer am Frame-Ende automatisch."""
        lyr = self._layers.get(name)
        if lyr is None or lyr.surface is None:
            return
        lyr.surface.fill((0, 0, 0, 0))

    # ---- Sprite-Atlas + Batch-Draw ----------------------------------
    # Atlas = ein Image + dict[name -> (x, y, w, h)]. Mehrere Sub-Sprites
    # teilen sich eine pygame.Surface -- Cache-freundlicher beim Wechseln
    # zwischen "Sprites" als hunderte Einzel-LOADIMAGE, und vor allem
    # batch-baufaehig via pygame.Surface.blits().

    def load_sprite_atlas(self, manifest_path: str):
        """Laedt einen Atlas aus einer JSON-Datei.

        Manifest-Format:
            {
              "image": "atlas.png",
              "sprites": {
                "tile_grass": [0, 0, 32, 32],
                "player_idle": [32, 0, 24, 32]
              }
            }
        Pfad zum Bild ist relativ zum Manifest. Sprite-Rects sind
        [x, y, w, h] (Pixel im Atlas).
        """
        import json as _json
        import os as _os
        from .interpreter import _Image, _SpriteAtlas
        try:
            with open(manifest_path, "r", encoding="utf-8") as f:
                manifest = _json.load(f)
        except FileNotFoundError:
            raise GBRuntimeError(
                f"ATLAS_LOAD: Manifest nicht gefunden: {manifest_path}"
            )
        except Exception as exc:
            raise GBRuntimeError(
                f"ATLAS_LOAD: Manifest-Lesefehler '{manifest_path}': {exc}"
            )
        if not isinstance(manifest, dict):
            raise GBRuntimeError(
                "ATLAS_LOAD: Manifest muss ein JSON-Object sein"
            )
        img_rel = manifest.get("image")
        if not isinstance(img_rel, str):
            raise GBRuntimeError(
                "ATLAS_LOAD: Feld 'image' fehlt oder ist kein STRING"
            )
        sprites = manifest.get("sprites")
        if not isinstance(sprites, dict):
            raise GBRuntimeError(
                "ATLAS_LOAD: Feld 'sprites' muss ein Object sein"
            )
        manifest_dir = _os.path.dirname(_os.path.abspath(manifest_path))
        img_path = _os.path.normpath(_os.path.join(manifest_dir, img_rel))
        surf = self.load_image(img_path)
        frames = {}
        for name, rect in sprites.items():
            if not isinstance(name, str):
                raise GBRuntimeError(
                    "ATLAS_LOAD: Sprite-Name muss STRING sein"
                )
            if (not isinstance(rect, list) or len(rect) != 4
                    or not all(isinstance(v, (int, float)) for v in rect)):
                raise GBRuntimeError(
                    f"ATLAS_LOAD: Sprite '{name}' Rect muss [x, y, w, h] sein"
                )
            frames[name] = (int(rect[0]), int(rect[1]),
                            int(rect[2]), int(rect[3]))
        img_wrapper = _Image(surf, img_path)
        return _SpriteAtlas(img_wrapper, frames, manifest_path)

    def _atlas_frame(self, atlas, name: str):
        rect = atlas.frames.get(name)
        if rect is None:
            raise GBRuntimeError(
                f"ATLAS_DRAW: Sprite '{name}' nicht im Atlas "
                f"(verfuegbar: {', '.join(list(atlas.frames.keys())[:5])}...)"
            )
        return rect

    def atlas_draw(self, atlas, name: str, x, y):
        """Einmal-Draw eines Atlas-Sub-Sprite. Camera-aware."""
        self._require_screen()
        # Pending Batch flushen, damit dieser Draw die korrekte Reihenfolge
        # behaelt (Batch wurde frueher angeordnet als dieser Direct-Call).
        if self._batch:
            self._flush_batch_now()
        rect = self._atlas_frame(atlas, name)
        self.draw_image_part(
            atlas.image.surface,
            rect[0], rect[1], rect[2], rect[3],
            x, y,
        )

    def atlas_draw_flipped(self, atlas, name: str, x, y,
                           flip_x: bool, flip_y: bool):
        """Atlas-Sub-Sprite mit horizontaler/vertikaler Spiegelung. Klassisch
        fuer Charakter-Sprites, die laufen-links/laufen-rechts aus EINEM
        Frame ableiten -- statt zwei separate Atlas-Frames zu pflegen.

        Implementation: subsurface() extrahiert den Rect, transform.flip()
        spiegelt, dann blit. Kein Caching -- bei vielen Wiederholungen mit
        gleichem (atlas, name, flip_x, flip_y) sollte der Caller einmal
        eine eigene gespiegelte Surface erzeugen statt es jedes Frame neu
        machen zu lassen.
        """
        self._require_screen()
        if self._batch:
            self._flush_batch_now()
        if not (flip_x or flip_y):
            # Optimierung: ohne Flip = normales atlas_draw
            return self.atlas_draw(atlas, name, x, y)
        rect = self._atlas_frame(atlas, name)
        sub = atlas.image.surface.subsurface(rect)
        flipped = self._pygame.transform.flip(sub, flip_x, flip_y)
        sx, sy = self._w2s(x, y)
        # Zoom-Pfad: bei zoom != 1 zusaetzlich skalieren wie draw_image_part
        if self._cam_zoom == 1.0:
            self._buffer.blit(flipped, (sx, sy))
        else:
            tw = max(1, int(flipped.get_width() * self._cam_zoom))
            th = max(1, int(flipped.get_height() * self._cam_zoom))
            self._buffer.blit(
                self._pygame.transform.scale(flipped, (tw, th)),
                (sx, sy),
            )

    def atlas_draw_batch(self, atlas, name: str, x, y):
        """Haengt einen Sub-Sprite an die Batch-Queue. BATCH_FLUSH (oder
        irgendein Direct-Draw bzw. FLIP) gibt den ganzen Stapel an
        pygame.Surface.blits() in einem Call -- spart Python-Overhead
        bei vielen Sprites."""
        self._require_screen()
        rect = self._atlas_frame(atlas, name)
        # Camera-Translation anwenden. Bei Zoom != 1 muesste pro-Sprite
        # skaliert werden, was den Batch-Vorteil kostet -- in dem Fall
        # zurueck zum draw_image_part-Pfad fallen.
        if self._cam_zoom == 1.0:
            sx, sy = self._w2s(x, y)
            self._batch.append((atlas.image.surface, (sx, sy), rect))
        else:
            self.draw_image_part(
                atlas.image.surface,
                rect[0], rect[1], rect[2], rect[3],
                x, y,
            )

    def _flush_batch_now(self):
        """Interner Flush -- ohne require_screen-Check, weil die Aufrufer
        das schon gemacht haben."""
        if not self._batch:
            return
        # pygame.Surface.blits(seq, doreturn=False) ist ein einziger
        # C-Call fuer alle Eintraege -- der eigentliche Batch-Win.
        self._buffer.blits(self._batch, doreturn=False)
        self._batch.clear()

    def batch_flush(self):
        """Public-API zum manuellen Flush des Batch. Auto-Flush passiert
        ohnehin bei Direct-Draws und FLIP."""
        self._require_screen()
        self._flush_batch_now()

    def plot(self, x, y, color: int = 0xFFFFFF):
        self._require_screen()
        sx, sy = self._w2s(x, y)
        try:
            self._buffer.set_at((sx, sy), _rgb_tuple(color))
        except IndexError:
            pass  # Pixel ausserhalb der Surface ignorieren

    def plot_many(self, xs, ys, colors):
        """Bulk-Plot vieler Pixel in EINEM Aufruf -- Groessenordnungen
        schneller als PLOT in einer Schleife: kein Builtin-Dispatch pro
        Pixel, und die Schreibzugriffe laufen vektorisiert ueber
        `pygame.surfarray` (numpy) statt N x set_at.

        `xs`, `ys` sind numerische Sequenzen gleicher Laenge. `colors` ist
        entweder ein einzelner INT (alle Pixel gleich) oder eine Sequenz
        gleicher Laenge (Farbe pro Pixel). Kamera-Transform wird beruecksichtigt.
        """
        self._require_screen()
        import numpy as np
        import pygame.surfarray as surfarray
        ax = np.asarray(xs, dtype=np.float64)
        ay = np.asarray(ys, dtype=np.float64)
        n = ax.shape[0]
        if ay.shape[0] != n:
            raise GBRuntimeError("PLOTS: xs und ys muessen gleich lang sein")
        if n == 0:
            return
        sx = ((ax - self._cam_x) * self._cam_zoom).astype(np.intp)
        sy = ((ay - self._cam_y) * self._cam_zoom).astype(np.intp)
        bw, bh = self._buffer.get_size()
        mask = (sx >= 0) & (sx < bw) & (sy >= 0) & (sy < bh)
        sxm = sx[mask]
        sym = sy[mask]
        if sxm.size == 0:
            return
        arr = surfarray.pixels3d(self._buffer)
        try:
            if isinstance(colors, (list, tuple)) or hasattr(colors, "__len__"):
                cc = np.asarray(colors, dtype=np.int64)
                if cc.shape[0] != n:
                    raise GBRuntimeError(
                        "PLOTS: colors-Array muss so lang wie xs sein")
                cc = cc[mask]
                arr[sxm, sym, 0] = (cc >> 16) & 0xFF
                arr[sxm, sym, 1] = (cc >> 8) & 0xFF
                arr[sxm, sym, 2] = cc & 0xFF
            else:
                col = int(colors)
                arr[sxm, sym, 0] = (col >> 16) & 0xFF
                arr[sxm, sym, 1] = (col >> 8) & 0xFF
                arr[sxm, sym, 2] = col & 0xFF
        finally:
            del arr  # Surface entsperren (pixels3d lockt sie)

    def line(self, x1, y1, x2, y2, color: int = 0xFFFFFF):
        self._require_screen()
        sx1, sy1 = self._w2s(x1, y1)
        sx2, sy2 = self._w2s(x2, y2)
        self._pygame.draw.line(
            self._buffer, _rgb_tuple(color), (sx1, sy1), (sx2, sy2),
        )

    def box(self, x1, y1, x2, y2, color: int = 0xFFFFFF):
        """Gefuelltes Rechteck."""
        self._require_screen()
        sx1, sy1 = self._w2s(x1, y1)
        sx2, sy2 = self._w2s(x2, y2)
        x = min(sx1, sx2)
        y = min(sy1, sy2)
        w = abs(sx2 - sx1) + 1
        h = abs(sy2 - sy1) + 1
        self._pygame.draw.rect(self._buffer, _rgb_tuple(color), (x, y, w, h))

    def rect(self, x1, y1, x2, y2, color: int = 0xFFFFFF):
        """Rahmen ohne Fuellung."""
        self._require_screen()
        sx1, sy1 = self._w2s(x1, y1)
        sx2, sy2 = self._w2s(x2, y2)
        x = min(sx1, sx2)
        y = min(sy1, sy2)
        w = abs(sx2 - sx1) + 1
        h = abs(sy2 - sy1) + 1
        self._pygame.draw.rect(self._buffer, _rgb_tuple(color), (x, y, w, h), 1)

    def circle(self, x, y, r, color: int = 0xFFFFFF):
        self._require_screen()
        sx, sy = self._w2s(x, y)
        self._pygame.draw.circle(
            self._buffer, _rgb_tuple(color),
            (sx, sy), self._scale_size(r),
        )

    # --- 2D-Extras (Batch 1) ------------------------------------------
    def line_thick(self, x1, y1, x2, y2, w, color: int = 0xFFFFFF):
        self._require_screen()
        sx1, sy1 = self._w2s(x1, y1)
        sx2, sy2 = self._w2s(x2, y2)
        width = max(1, int(w * self._cam_zoom))
        self._pygame.draw.line(self._buffer, _rgb_tuple(color), (sx1, sy1), (sx2, sy2), width)

    def round_rect(self, x1, y1, x2, y2, radius, color: int = 0xFFFFFF, filled: bool = True):
        self._require_screen()
        sx1, sy1 = self._w2s(x1, y1)
        sx2, sy2 = self._w2s(x2, y2)
        x = min(sx1, sx2); y = min(sy1, sy2)
        w = abs(sx2 - sx1) + 1; h = abs(sy2 - sy1) + 1
        rad = max(0, min(self._scale_size(radius), min(w, h) // 2))
        self._pygame.draw.rect(self._buffer, _rgb_tuple(color), (x, y, w, h),
                               0 if filled else 1, border_radius=rad)

    def gradient_rect(self, x1, y1, x2, y2, c1: int, c2: int, vertical: bool = True):
        self._require_screen()
        sx1, sy1 = self._w2s(x1, y1)
        sx2, sy2 = self._w2s(x2, y2)
        x = min(sx1, sx2); y = min(sy1, sy2)
        w = abs(sx2 - sx1) + 1; h = abs(sy2 - sy1) + 1
        r1, g1, b1 = _rgb_tuple(c1); r2, g2, b2 = _rgb_tuple(c2)
        n = h if vertical else w
        if n <= 1:
            self._pygame.draw.rect(self._buffer, _rgb_tuple(c1), (x, y, w, h)); return
        line = self._pygame.draw.line
        buf = self._buffer
        for i in range(n):
            t = i / (n - 1)
            col = (int(r1 + (r2 - r1) * t), int(g1 + (g2 - g1) * t), int(b1 + (b2 - b1) * t))
            if vertical:
                line(buf, col, (x, y + i), (x + w - 1, y + i))
            else:
                line(buf, col, (x + i, y), (x + i, y + h - 1))

    def spline(self, xs, ys, w, color: int = 0xFFFFFF):
        """Catmull-Rom-Spline durch die Punkte (Endpunkte verdoppelt)."""
        self._require_screen()
        pts = [self._w2s(x, y) for x, y in zip(xs, ys)]
        width = max(1, int(w * self._cam_zoom))
        if len(pts) < 2:
            return
        if len(pts) < 4:
            self._pygame.draw.lines(self._buffer, _rgb_tuple(color), False, pts, width)
            return
        ext = [pts[0]] + pts + [pts[-1]]
        curve = []
        steps = 16
        for i in range(1, len(ext) - 2):
            p0, p1, p2, p3 = ext[i - 1], ext[i], ext[i + 1], ext[i + 2]
            for s in range(steps + 1):
                t = s / steps; t2 = t * t; t3 = t2 * t
                cx = 0.5 * (2 * p1[0] + (-p0[0] + p2[0]) * t
                            + (2 * p0[0] - 5 * p1[0] + 4 * p2[0] - p3[0]) * t2
                            + (-p0[0] + 3 * p1[0] - 3 * p2[0] + p3[0]) * t3)
                cy = 0.5 * (2 * p1[1] + (-p0[1] + p2[1]) * t
                            + (2 * p0[1] - 5 * p1[1] + 4 * p2[1] - p3[1]) * t2
                            + (-p0[1] + 3 * p1[1] - 3 * p2[1] + p3[1]) * t3)
                curve.append((cx, cy))
        self._pygame.draw.lines(self._buffer, _rgb_tuple(color), False, curve, width)

    # --- Blend-Modes (Batch 2): nativ; pygame-Primitive kennen kein Blend-Flag
    #     -> graceful No-Op (Programm laeuft, nur ohne additiven/Multiply-Effekt).
    def blend_mode(self, mode):
        return None

    # --- Prozedurale Texturen (Batch 3): native-only (liefern ein IMAGE-Handle,
    #     das pygame ohne echte Implementierung nicht erzeugen kann).
    def _native_gfx_only(self, name):
        raise GBRuntimeError(
            f"{name}: nur in der nativen Runtime (gbrt). Ueber den Run-Button "
            f"(F5) bzw. 'gbrun.py --native' starten; Profiler/Debugger "
            f"(Tree-Walker) unterstuetzen es nicht.")

    def gen_tex_perlin(self, *a):   self._native_gfx_only("GENTEX_PERLIN")
    def gen_tex_gradient(self, *a): self._native_gfx_only("GENTEX_GRADIENT")
    def gen_tex_checked(self, *a):  self._native_gfx_only("GENTEX_CHECKED")
    def gen_tex_color(self, *a):    self._native_gfx_only("GENTEX_COLOR")

    # --- Clipboard + Drag&Drop (Batch 5): nativ; pygame graceful (leer / keine
    #     Drops), damit F5-Programme nicht crashen.
    def clipboard_get(self):    return ""
    def clipboard_set(self, s): return None
    def files_dropped(self):    return 0
    def file_dropped(self, i):  return ""

    # --- Render-Targets (Batch 4): Off-Screen-Surfaces (Immediate-Mode) ---
    def rendertarget_new(self, w, h):
        self._require_screen()
        surf = self._pygame.Surface((max(1, int(w)), max(1, int(h))),
                                    self._pygame.SRCALPHA)
        self._render_targets.append(surf)
        return len(self._render_targets) - 1

    def _check_rt(self, idx, name):
        if (isinstance(idx, bool) or not isinstance(idx, int)
                or idx < 0 or idx >= len(self._render_targets)):
            raise GBRuntimeError(f"{name}: ungueltiges RENDERTARGET-Handle {idx}")
        return idx

    def rendertarget_begin(self, idx):
        self._require_screen()
        i = self._check_rt(idx, "RENDERTARGET_BEGIN")
        self._rt_stack.append(self._buffer)
        self._buffer = self._render_targets[i]
        # Pro Frame transparent clearen (wie die native Runtime).
        self._buffer.fill((0, 0, 0, 0))

    def rendertarget_end(self):
        if self._rt_stack:
            self._buffer = self._rt_stack.pop()

    def rendertarget_draw(self, idx, x, y, scale=1.0, tint=None):
        self._require_screen()
        i = self._check_rt(idx, "RENDERTARGET_DRAW")
        surf = self._render_targets[i]
        sx, sy = self._w2s(x, y)
        factor = scale * self._cam_zoom
        if factor != 1.0:
            nw = max(1, int(surf.get_width() * factor))
            nh = max(1, int(surf.get_height() * factor))
            surf = self._pygame.transform.scale(surf, (nw, nh))
        if tint is not None and tint != 0xFFFFFF:
            surf = surf.copy()
            surf.fill((*_rgb_tuple(tint), 255),
                      special_flags=self._pygame.BLEND_RGBA_MULT)
        self._buffer.blit(surf, (sx, sy))

    # --- Bulk-Primitive: viele Shapes in EINEM Aufruf -------------------
    # Spart den Builtin-Dispatch pro Shape (1 Call statt N). Die pygame-
    # Draws bleiben pro Shape (pygame hat kein Batch fuer Primitive), aber
    # der Interpreter-Overhead -- der dominante Kostenfaktor laut
    # docs/PERFORMANCE.md -- entfaellt. `colors` ist ein INT (alle gleich)
    # oder eine Sequenz (Farbe pro Shape).

    def _bulk_color(self, colors, n):
        """Liefert (single_tuple_or_None, per_index_list_or_None)."""
        if isinstance(colors, int) and not isinstance(colors, bool):
            return _rgb_tuple(colors), None
        if len(colors) != n:
            raise GBRuntimeError("Bulk-Draw: colors-Array muss so lang wie die "
                                 "Koordinaten sein")
        return None, [_rgb_tuple(c) for c in colors]

    def box_many(self, x1s, y1s, x2s, y2s, colors):
        self._require_screen()
        n = len(x1s)
        if not (len(y1s) == len(x2s) == len(y2s) == n):
            raise GBRuntimeError("BOXES: alle Koordinaten-Arrays muessen gleich lang sein")
        col, cols = self._bulk_color(colors, n)
        rect = self._pygame.draw.rect
        buf = self._buffer
        w2s = self._w2s
        for i in range(n):
            ax, ay = w2s(x1s[i], y1s[i])
            bx, by = w2s(x2s[i], y2s[i])
            x = ax if ax < bx else bx
            y = ay if ay < by else by
            rect(buf, col if col is not None else cols[i],
                 (x, y, abs(bx - ax) + 1, abs(by - ay) + 1))

    def circle_many(self, xs, ys, rs, colors):
        self._require_screen()
        n = len(xs)
        if not (len(ys) == len(rs) == n):
            raise GBRuntimeError("CIRCLES: alle Arrays muessen gleich lang sein")
        col, cols = self._bulk_color(colors, n)
        draw_c = self._pygame.draw.circle
        buf = self._buffer
        w2s = self._w2s
        ssize = self._scale_size
        for i in range(n):
            sx, sy = w2s(xs[i], ys[i])
            draw_c(buf, col if col is not None else cols[i], (sx, sy), ssize(rs[i]))

    def line_many(self, x1s, y1s, x2s, y2s, colors):
        self._require_screen()
        n = len(x1s)
        if not (len(y1s) == len(x2s) == len(y2s) == n):
            raise GBRuntimeError("LINES: alle Koordinaten-Arrays muessen gleich lang sein")
        col, cols = self._bulk_color(colors, n)
        draw_l = self._pygame.draw.line
        buf = self._buffer
        w2s = self._w2s
        for i in range(n):
            draw_l(buf, col if col is not None else cols[i],
                   w2s(x1s[i], y1s[i]), w2s(x2s[i], y2s[i]))

    def triangle(self, x1, y1, x2, y2, x3, y3, color: int = 0xFFFFFF):
        """Gefuelltes Dreieck."""
        self._require_screen()
        pts = [
            self._w2s(x1, y1),
            self._w2s(x2, y2),
            self._w2s(x3, y3),
        ]
        self._pygame.draw.polygon(self._buffer, _rgb_tuple(color), pts)

    def triangle_outline(self, x1, y1, x2, y2, x3, y3,
                         color: int = 0xFFFFFF, width: int = 1):
        """Dreieck nur als Kontur."""
        self._require_screen()
        pts = [
            self._w2s(x1, y1),
            self._w2s(x2, y2),
            self._w2s(x3, y3),
        ]
        self._pygame.draw.polygon(
            self._buffer, _rgb_tuple(color), pts, max(1, int(width)),
        )

    def polygon(self, points, color: int = 0xFFFFFF):
        """Gefuelltes Polygon. `points` ist eine flache Liste
        [x1, y1, x2, y2, ...] mit mindestens 3 Punkten (6 Werte)."""
        self._require_screen()
        pts = self._pairs_to_points(points)
        if len(pts) < 3:
            raise GBRuntimeError(
                "POLYGON: braucht mindestens 3 Punkte (6 Werte)"
            )
        self._pygame.draw.polygon(self._buffer, _rgb_tuple(color), pts)

    def polygon_outline(self, points, color: int = 0xFFFFFF, width: int = 1):
        """Polygon nur als Kontur."""
        self._require_screen()
        pts = self._pairs_to_points(points)
        if len(pts) < 3:
            raise GBRuntimeError(
                "POLYGONOUTLINE: braucht mindestens 3 Punkte (6 Werte)"
            )
        self._pygame.draw.polygon(
            self._buffer, _rgb_tuple(color), pts, max(1, int(width)),
        )

    def ellipse(self, x1, y1, x2, y2, color: int = 0xFFFFFF):
        """Gefuellte Ellipse, eingepasst in die Bounding-Box (x1,y1)-(x2,y2)."""
        self._require_screen()
        sx1, sy1 = self._w2s(x1, y1)
        sx2, sy2 = self._w2s(x2, y2)
        x = min(sx1, sx2)
        y = min(sy1, sy2)
        w = abs(sx2 - sx1) + 1
        h = abs(sy2 - sy1) + 1
        self._pygame.draw.ellipse(self._buffer, _rgb_tuple(color), (x, y, w, h))

    def ellipse_outline(self, x1, y1, x2, y2,
                         color: int = 0xFFFFFF, width: int = 1):
        """Ellipse nur als Kontur."""
        self._require_screen()
        sx1, sy1 = self._w2s(x1, y1)
        sx2, sy2 = self._w2s(x2, y2)
        x = min(sx1, sx2)
        y = min(sy1, sy2)
        w = abs(sx2 - sx1) + 1
        h = abs(sy2 - sy1) + 1
        self._pygame.draw.ellipse(
            self._buffer, _rgb_tuple(color), (x, y, w, h), max(1, int(width)),
        )

    def arc(self, x1, y1, x2, y2, start_angle: float, end_angle: float,
            color: int = 0xFFFFFF, width: int = 1):
        """Bogen-Segment in der Bounding-Box (x1,y1)-(x2,y2).

        Winkel sind in Radiant. 0 = positive X-Achse, gegen den Uhrzeigersinn
        positiv (so wie bei MATH-Funktionen). Pygame's Konvention ist gleich.
        """
        self._require_screen()
        sx1, sy1 = self._w2s(x1, y1)
        sx2, sy2 = self._w2s(x2, y2)
        x = min(sx1, sx2)
        y = min(sy1, sy2)
        w = abs(sx2 - sx1) + 1
        h = abs(sy2 - sy1) + 1
        self._pygame.draw.arc(
            self._buffer, _rgb_tuple(color), (x, y, w, h),
            float(start_angle), float(end_angle),
            max(1, int(width)),
        )

    @staticmethod
    def _pairs_to_points(values) -> list:
        """Konvertiert eine flache Liste/Sequenz [x1,y1,x2,y2,...] in
        eine Liste von (x, y)-Tupeln. Akzeptiert Listen, Tuples und
        GB-Arrays (`_GBArray` mit `.values`-Attribut, ohne natives
        Iterator-Protokoll)."""
        # _GBArray hat keine __iter__-Methode, aber das interne .values
        # ist iterable (Liste oder array.array bei typed-backing) - die
        # nutzen wir direkt.
        import array as _array_mod
        gb_values = getattr(values, "values", None)
        if isinstance(gb_values, (list, _array_mod.array)):
            seq = gb_values
        elif isinstance(values, (list, tuple)):
            seq = values
        else:
            try:
                seq = list(values)
            except TypeError:
                raise GBRuntimeError(
                    "POLYGON: Punkte muessen ein ARRAY OF INTEGER sein"
                )
        if len(seq) % 2 != 0:
            raise GBRuntimeError(
                "POLYGON: Punkt-Liste muss eine gerade Anzahl Werte haben"
                " ([x1, y1, x2, y2, ...])"
            )
        if len(seq) < 6:
            # Wird in den Aufrufern auch nochmal geprueft, aber hier kommt
            # eine klarere Meldung als 'Polygon mit 2 Punkten' weiter unten.
            pass
        pts: list = []
        for i in range(0, len(seq), 2):
            try:
                xi = int(seq[i])
                yi = int(seq[i + 1])
            except (TypeError, ValueError):
                raise GBRuntimeError(
                    "POLYGON: Werte muessen INTEGER sein"
                )
            pts.append((xi, yi))
        return pts

    def _get_font(self):
        """Liefert das aktuell konfigurierte Font-Objekt (gecachet).

        Ist ein TTF via LOADFONT/SETFONT aktiv, wird ein
        ``pygame.font.Font(pfad, _font_size)`` gebaut (pro Groesse gecachet),
        sonst der Default-SysFont. TEXT_SIZE skaliert in beiden Faellen."""
        if self._active_font >= 0:
            path, _base = self._fonts[self._active_font]
            key = ("ttf", self._active_font, self._font_size,
                   self._font_bold, self._font_italic)
            f = self._font_cache.get(key)
            if f is None:
                f = self._pygame.font.Font(path, self._font_size)
                f.set_bold(self._font_bold)
                f.set_italic(self._font_italic)
                self._font_cache[key] = f
            return f
        key = (self._font_size, self._font_bold, self._font_italic)
        f = self._font_cache.get(key)
        if f is None:
            f = self._pygame.font.SysFont(
                None, self._font_size,
                bold=self._font_bold, italic=self._font_italic,
            )
            self._font_cache[key] = f
        return f

    def load_font(self, path: str, size: int) -> int:
        """Laedt einen TTF/OTF-Font und liefert ein FONT-Handle (INTEGER).
        ``size`` ist die Basis-Groesse; TEXT_SIZE skaliert beim Zeichnen."""
        self._require_screen()
        if size < 4 or size > 400:
            raise GBRuntimeError("LOADFONT: Groesse muss zwischen 4 und 400 liegen")
        try:
            # Frueh validieren, damit ein fehlender Pfad sofort meldet.
            self._pygame.font.Font(path, int(size))
        except Exception as e:
            raise GBRuntimeError(f"LOADFONT: Font '{path}' nicht ladbar: {e}")
        self._fonts.append((path, int(size)))
        return len(self._fonts) - 1

    def set_font(self, handle: int):
        """Aktiviert einen via LOADFONT geladenen Font. -1 = Default-Font."""
        self._require_screen()
        if handle < -1 or handle >= len(self._fonts):
            raise GBRuntimeError(f"SETFONT: ungueltiges FONT-Handle {handle}")
        self._active_font = int(handle)

    def text_spacing(self, px: int):
        """Buchstabenabstand fuer TTF-Text (wirkt nur in der nativen Runtime)."""
        self._require_screen()
        self._text_spacing = int(px)

    def text(self, x, y, s: str, color: int = 0xFFFFFF):
        """Text wird nur translatiert, nicht skaliert (sonst verschwommen).
        Fuer scharfen HUD-Text vorher CAMERA_RESET() aufrufen."""
        self._require_screen()
        font = self._get_font()
        surf = font.render(str(s), True, _rgb_tuple(color))
        sx, sy = self._w2s(x, y)
        self._buffer.blit(surf, (sx, sy))

    def text_size(self, size: int):
        """Setzt die Schriftgroesse fuer nachfolgende TEXT-Aufrufe."""
        self._require_screen()
        if size < 4:
            raise GBRuntimeError("TEXT_SIZE: Groesse muss >= 4 sein")
        if size > 400:
            raise GBRuntimeError("TEXT_SIZE: Groesse muss <= 400 sein")
        self._font_size = int(size)

    def text_bold(self, on: bool):
        self._require_screen()
        self._font_bold = bool(on)

    def text_italic(self, on: bool):
        self._require_screen()
        self._font_italic = bool(on)

    def text_width(self, s: str) -> int:
        """Pixelbreite des STRINGs in der aktuell eingestellten Schrift."""
        self._require_screen()
        return int(self._get_font().size(str(s))[0])

    def text_height(self) -> int:
        """Hoehe der aktuellen Schrift in Pixel."""
        self._require_screen()
        return int(self._get_font().get_linesize())

    # ---- Scrolling ---------------------------------------------------
    def scroll(self, dx: int, dy: int):
        """Verschiebt den aktuellen Buffer um (dx, dy) Pixel.

        Nutzt pygame's Surface.scroll - in C, sehr schnell. Pixel die
        rausgeschoben werden, sind weg; freier Bereich bleibt unveraendert
        (mit dem aktuellen Inhalt) - i.d.R. mit CLS oder PLOT/RECT
        ueberzeichnen.
        """
        self._require_screen()
        # pygame's Surface.scroll erwartet Pixel-Offsets
        self._buffer.scroll(int(dx), int(dy))

    # ---- Bilder ------------------------------------------------------
    def load_image(self, path: str):
        # Cache-Check zuerst -- Hit unter dem rohen Schluessel (Alias-Pfad).
        # Wenn die User-Datei via LOAD_ASSETS unter einem Alias registriert
        # wurde, trifft genau dieser Pfad.
        cached = self._image_cache.get(path)
        if cached is not None:
            return cached
        # Fallback-Schluessel: normalisierter absoluter Pfad. So trifft
        # LOADIMAGE("assets/sprites/x.png") auch nach LOAD_ASSETS mit
        # gleicher Resolved-Datei, selbst wenn die Schreibweise abweicht
        # (z.B. "assets\\sprites\\x.png" vs "assets/sprites/x.png").
        import os as _os
        abs_key = _os.path.normpath(_os.path.abspath(path))
        cached = self._image_cache.get(abs_key)
        if cached is not None:
            # Auch unter dem rohen Pfad spiegeln fuer schnelleren Folge-Hit
            self._image_cache[path] = cached
            return cached
        self._ensure_pygame()
        try:
            surf = self._pygame.image.load(path)
        except Exception as exc:
            raise GBRuntimeError(f"Konnte Bild nicht laden: {path} ({exc})")
        if self._screen is not None:
            surf = surf.convert_alpha()
        # Beide Schluessel cachen
        self._image_cache[path] = surf
        self._image_cache[abs_key] = surf
        return surf

    def cache_image_under(self, key: str, surface):
        """Registriert eine geladene Surface unter einem zusaetzlichen
        Schluessel (typisch: Alias aus LOAD_ASSETS-Manifest)."""
        self._image_cache[key] = surface

    def _zoom_surface(self, surface):
        """Bei zoom != 1 skaliert die Bild-Surface fuer Camera-Zoom."""
        if self._cam_zoom == 1.0:
            return surface
        w = max(1, int(surface.get_width() * self._cam_zoom))
        h = max(1, int(surface.get_height() * self._cam_zoom))
        return self._pygame.transform.scale(surface, (w, h))

    def draw_image(self, surface, x, y):
        self._require_screen()
        sx, sy = self._w2s(x, y)
        self._buffer.blit(self._zoom_surface(surface), (sx, sy))

    def draw_image_part(self, surface, src_x, src_y, src_w, src_h, x, y):
        self._require_screen()
        screen_x, screen_y = self._w2s(x, y)
        rect = (int(src_x), int(src_y), int(src_w), int(src_h))
        if self._cam_zoom == 1.0:
            self._buffer.blit(surface, (screen_x, screen_y), rect)
        else:
            # Erst den Ausschnitt extrahieren, dann skalieren.
            tw = max(1, int(int(src_w) * self._cam_zoom))
            th = max(1, int(int(src_h) * self._cam_zoom))
            sub = surface.subsurface(rect)
            self._buffer.blit(
                self._pygame.transform.scale(sub, (tw, th)),
                (screen_x, screen_y),
            )

    def draw_tilemap(self, surface, values, rows, cols, tw, th, sx, sy):
        """Zeichnet eine 2D-Tilemap (flache row-major `values`; Tile < 0 =
        transparent) in EINEM `blits()`-Call -- spart den Python-Overhead
        hunderter Einzel-blits. Bei cam_zoom != 1.0 Fallback auf
        Einzel-Zeichnung (mit Skalierung)."""
        self._require_screen()
        tw = int(tw)
        th = int(th)
        tiles_per_row = max(1, surface.get_width() // max(tw, 1))
        if self._cam_zoom == 1.0:
            ox, oy = self._w2s(sx, sy)   # bei zoom==1 reine Translation
            seq = []
            ap = seq.append
            for r in range(rows):
                base = r * cols
                dy = oy + r * th
                for c in range(cols):
                    tile = values[base + c]
                    if tile < 0:
                        continue
                    sc = tile % tiles_per_row
                    sr = tile // tiles_per_row
                    ap((surface, (ox + c * tw, dy),
                        (sc * tw, sr * th, tw, th)))
            if seq:
                self._buffer.blits(seq, doreturn=False)
        else:
            for r in range(rows):
                base = r * cols
                for c in range(cols):
                    tile = values[base + c]
                    if tile < 0:
                        continue
                    sc = tile % tiles_per_row
                    sr = tile // tiles_per_row
                    self.draw_image_part(surface, sc * tw, sr * th, tw, th,
                                         sx + c * tw, sy + r * th)

    def draw_image_flipped(self, surface, x, y, flip_x: bool, flip_y: bool):
        self._require_screen()
        if flip_x or flip_y:
            surface = self._pygame.transform.flip(surface, flip_x, flip_y)
        sx, sy = self._w2s(x, y)
        self._buffer.blit(self._zoom_surface(surface), (sx, sy))

    def image_size(self, surface):
        return surface.get_width(), surface.get_height()

    # ---- Sound -------------------------------------------------------
    def _ensure_mixer(self):
        self._ensure_pygame()
        if not self._pygame.mixer.get_init():
            self._pygame.mixer.init()

    def load_sound(self, path: str):
        # Cache-Pattern wie load_image -- doppelter Schluessel (roh + abs)
        # fuer Alias-Kompatibilitaet und stabile Pfad-Normalisierung.
        cached = self._sound_cache.get(path)
        if cached is not None:
            return cached
        import os as _os
        abs_key = _os.path.normpath(_os.path.abspath(path))
        cached = self._sound_cache.get(abs_key)
        if cached is not None:
            self._sound_cache[path] = cached
            return cached
        self._ensure_mixer()
        try:
            snd = self._pygame.mixer.Sound(path)
        except Exception as exc:
            raise GBRuntimeError(f"Konnte Sound nicht laden: {path} ({exc})")
        self._sound_cache[path] = snd
        self._sound_cache[abs_key] = snd
        return snd

    def cache_sound_under(self, key: str, sound):
        """Registriert einen geladenen Sound unter einem zusaetzlichen
        Schluessel."""
        self._sound_cache[key] = sound

    def play_sound(self, sound, loops: int = 0, volume: float = 1.0):
        self._ensure_mixer()
        sound.set_volume(max(0.0, min(1.0, float(volume))))
        sound.play(loops=loops)

    def stop_sound(self, sound):
        self._ensure_mixer()
        sound.stop()

    def play_music(self, path: str, loops: int = -1, volume: float = 1.0):
        self._ensure_mixer()
        try:
            self._pygame.mixer.music.load(path)
        except Exception as exc:
            raise GBRuntimeError(f"Konnte Musik nicht laden: {path} ({exc})")
        self._pygame.mixer.music.set_volume(max(0.0, min(1.0, float(volume))))
        self._pygame.mixer.music.play(loops=loops)

    def stop_music(self):
        self._ensure_mixer()
        self._pygame.mixer.music.stop()

    def flip(self):
        self._require_screen()
        # Pending Sprite-Batch flushen, BEVOR der Layer-Wechsel passiert,
        # damit der Batch auf dem AKTUELLEN Target landet (typisch der
        # gerade aktive Layer).
        if self._batch:
            self._flush_batch_now()
        # Falls der User waehrend des Frames in einem Layer-Block "vergisst",
        # LAYER_END zu rufen: einfach jetzt zurueckspringen, damit das
        # Composite den Main-Buffer trifft.
        self._current_layer = None
        self._buffer = self._main_buffer
        # Layer in z-Order auf den Main-Buffer composen. Layer-Surfaces
        # haben SRCALPHA, also wird transparent korrekt ueberlagert.
        # Reihenfolge: niedrigstes z zuerst (= hinten), hoechstes z zuletzt.
        if self._layer_order:
            for name in self._layer_order:
                lyr = self._layers[name]
                if lyr.surface is None:
                    continue
                self._main_buffer.blit(lyr.surface, (0, 0))
                # Layer fuer den naechsten Frame leeren (transparent).
                lyr.surface.fill((0, 0, 0, 0))
        if self._scale == 1:
            self._screen.blit(self._main_buffer, (0, 0))
        else:
            # Nearest-Neighbor-Scaling fuer gestochen scharfe Pixel
            scaled = self._pygame.transform.scale(
                self._main_buffer, self._screen.get_size())
            self._screen.blit(scaled, (0, 0))
        self._pygame.display.flip()
        self._pump()
        # Auf die Ziel-FPS takten und die vergangene Frame-Zeit merken (DELTA()).
        self._delta = self._clock.tick(self._target_fps) / 1000.0

    def sleep_ms(self, ms: int):
        self._ensure_pygame()
        # Events weiter pumpen, sonst friert das Fenster ein.
        target = self._pygame.time.get_ticks() + max(int(ms), 0)
        while True:
            now = self._pygame.time.get_ticks()
            if now >= target:
                break
            self._pump()
            self._pygame.time.wait(min(16, target - now))

    # ---- Eingabe -----------------------------------------------------
    def key_pressed(self, code: int) -> bool:
        self._ensure_pygame()
        self._pump()
        return int(code) in self._keys

    def quit_requested(self) -> bool:
        self._ensure_pygame()
        self._pump()
        return self._closed

    # ---- Text-/Key-Input fuer das UI-Modul ---------------------------
    def pop_text_input(self) -> str:
        """Drained den Text-Input-Buffer und gibt die gesammelten Zeichen
        zurueck. UI-Komponenten rufen das einmal pro Frame auf solange sie
        Fokus haben - dann landet jeder gedrueckte druckbare Buchstabe genau
        in einem Field."""
        self._ensure_pygame()
        self._pump()
        s = "".join(self._text_input)
        self._text_input.clear()
        return s

    def keys_pressed(self) -> set:
        """Snapshot aller aktuell gedrueckten Keys (SDL-Keycodes).
        Read-only - der Aufrufer modifiziert das Set nicht.
        UI-Modul nutzt das fuer Edge-Detection (Backspace, Enter etc.)."""
        self._ensure_pygame()
        self._pump()
        return set(self._keys)

    def inkey(self) -> str:
        """Non-blocking: gibt das naechste getippte druckbare Zeichen zurueck,
        oder leeren STRING wenn keine Eingabe ansteht.

        Klassisches BASIC-INKEY$ - typischer Loop:
            DO
                k$ = INKEY$()
            UNTIL LEN(k$) > 0
        """
        self._ensure_pygame()
        self._pump()
        if not self._text_input:
            return ""
        ch = self._text_input.pop(0)
        return ch

    def waitkey(self) -> int:
        """Blockiert bis eine Taste gedrueckt wird. Gibt den SDL-Keycode
        zurueck (z.B. 27 = ESC, 13 = ENTER, KEY_*-Konstanten)."""
        self._ensure_pygame()
        pg = self._pygame
        # Vorhandene Queue-Eintraege erst verarbeiten
        self._pump()
        while not self._key_queue and not self._closed:
            pg.time.wait(15)   # 15 ms - kein CPU-Spin, gute Reaktionszeit
            self._pump()
        if self._closed:
            return -1
        return int(self._key_queue.pop(0))

    def pop_mouse_wheel(self) -> int:
        """Drained den Mausrad-Akkumulator. Positive Werte = nach oben,
        negative = nach unten. Pro Notch typisch +1/-1."""
        self._ensure_pygame()
        self._pump()
        v = self._mouse_wheel_y
        self._mouse_wheel_y = 0
        return v

    def push_clip(self, x: int, y: int, w: int, h: int):
        """Schraenkt das Zeichnen aufs angegebene Rechteck ein.  Verschachtelt
        sauber mit aeusserem Clip (Intersection): ein inneres push/pop laesst
        den aeusseren Clip danach wieder gelten.

        Anschliessend immer pop_clip() rufen.
        """
        self._require_screen()
        new_rect = self._pygame.Rect(int(x), int(y), int(w), int(h))
        # Pygame's get_clip() liefert immer ein gueltiges Rect (Surface-
        # Groesse wenn nicht explizit gesetzt). Das speichern wir und
        # schneiden den neuen Bereich damit auf den Eltern-Clip zu.
        prev = self._buffer.get_clip()
        self._clip_stack.append(prev)
        new_rect = new_rect.clip(prev)
        self._buffer.set_clip(new_rect)

    def pop_clip(self):
        self._require_screen()
        if self._clip_stack:
            self._buffer.set_clip(self._clip_stack.pop())
        else:
            self._buffer.set_clip(None)

    # ---- TIMER ------------------------------------------------------
    def timer(self) -> float:
        """Sekunden seit erstem TIMER-Aufruf als FLOAT (sub-millisekunden-genau).

        Erster Aufruf liefert 0.0; jeder weitere die seitdem verstrichene Zeit.
        Genau genug fuer Delta-Time in Game-Loops.
        """
        import time as _time
        if self._timer_start is None:
            self._timer_start = _time.perf_counter()
            return 0.0
        return _time.perf_counter() - self._timer_start

    # ---- Joystick / Gamepad ------------------------------------------
    def _init_joysticks(self):
        if self._joystick_init_done:
            return
        self._ensure_pygame()
        pg = self._pygame
        pg.joystick.init()
        n = pg.joystick.get_count()
        self._joysticks = []
        for i in range(n):
            j = pg.joystick.Joystick(i)
            j.init()
            self._joysticks.append(j)
        self._joystick_init_done = True

    def joystick_count(self) -> int:
        self._init_joysticks()
        return len(self._joysticks)

    def _joystick_at(self, idx: int):
        self._init_joysticks()
        if idx < 0 or idx >= len(self._joysticks):
            raise GBRuntimeError(
                f"Joystick-Index {idx} ausserhalb [0..{len(self._joysticks) - 1}]"
                if self._joysticks else
                f"Joystick-Index {idx} - kein Gamepad angeschlossen"
            )
        return self._joysticks[idx]

    def joystick_name(self, idx: int) -> str:
        return self._joystick_at(idx).get_name()

    def joystick_axis(self, idx: int, axis: int) -> float:
        self._pump()
        j = self._joystick_at(idx)
        if axis < 0 or axis >= j.get_numaxes():
            return 0.0
        return float(j.get_axis(axis))

    def joystick_button(self, idx: int, btn: int) -> bool:
        self._pump()
        j = self._joystick_at(idx)
        if btn < 0 or btn >= j.get_numbuttons():
            return False
        return bool(j.get_button(btn))

    def joystick_hat_x(self, idx: int, hat: int) -> int:
        self._pump()
        j = self._joystick_at(idx)
        if hat < 0 or hat >= j.get_numhats():
            return 0
        return int(j.get_hat(hat)[0])

    def joystick_hat_y(self, idx: int, hat: int) -> int:
        self._pump()
        j = self._joystick_at(idx)
        if hat < 0 or hat >= j.get_numhats():
            return 0
        return int(j.get_hat(hat)[1])

    def mouse_x(self) -> int:
        self._ensure_pygame()
        self._pump()
        return self._mouse_x // self._scale if self._scale > 1 else self._mouse_x

    def mouse_y(self) -> int:
        self._ensure_pygame()
        self._pump()
        return self._mouse_y // self._scale if self._scale > 1 else self._mouse_y

    def mouse_button(self, n: int) -> bool:
        self._ensure_pygame()
        self._pump()
        idx = int(n)
        if 0 <= idx < len(self._mouse_buttons):
            return self._mouse_buttons[idx]
        return False

    # ---- Intern ------------------------------------------------------
    def _pump(self):
        pg = self._pygame
        for ev in pg.event.get():
            if ev.type == pg.QUIT:
                self._closed = True
            elif ev.type == pg.KEYDOWN:
                self._keys.add(ev.key)
                # Keycode in FIFO-Queue fuer INKEY$/WAITKEY
                self._key_queue.append(int(ev.key))
                # Druckbare Zeichen ins Text-Input-Buffer geben.
                # ev.unicode ist leer fuer Modifikator/Funktions-Keys, "" fuer
                # Tab/Backspace/Enter; wir filtern nach control-Codes.
                ch = getattr(ev, "unicode", "")
                if ch and (ord(ch) >= 32 or ch == "\t"):
                    if ord(ch) != 127:  # Delete-Key liefert "\x7f"
                        self._text_input.append(ch)
                if ev.key == 27:  # ESC
                    pass  # Optional: self._closed = True
            elif ev.type == pg.KEYUP:
                self._keys.discard(ev.key)
            elif ev.type == pg.MOUSEMOTION:
                self._mouse_x, self._mouse_y = ev.pos
            elif ev.type == pg.MOUSEBUTTONDOWN:
                self._update_mouse()
            elif ev.type == pg.MOUSEBUTTONUP:
                self._update_mouse()
            elif ev.type == pg.MOUSEWHEEL:
                self._mouse_wheel_y += int(ev.y)

    def _update_mouse(self):
        pg = self._pygame
        self._mouse_buttons = pg.mouse.get_pressed()
        self._mouse_x, self._mouse_y = pg.mouse.get_pos()
