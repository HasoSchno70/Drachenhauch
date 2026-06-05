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


# ---------------------------------------------------------------------------
# Konsolen-only Stub. Die Grafik-/Audio-Engine lebt seit dem gbrt-Umstieg in
# der nativen Runtime (Rust/raylib); der Tree-Walker (F5-Fallback, Profiler,
# Debugger, --bench, Test-Referenz) ist nur noch Konsolen-/Referenzpfad. pygame
# wurde entfernt. COLORS/KEYS (oben) bleiben reine Daten; die Graphics-Klasse
# ist konstruierbar (Kamera-Mathematik), wirft aber bei jedem Draw-/Audio-/
# Bild-/Font-Zugriff eine klare "nur in gbrt"-Meldung.
# ---------------------------------------------------------------------------


def _native_only(op: str = "Grafik"):
    raise GBRuntimeError(
        f"{op}: Grafik/Audio laeuft nur in der nativen Runtime (gbrt) -- per F6 "
        f"bzw. 'gbrun.py --native' starten. Der Tree-Walker (F5/Profiler/Debugger) "
        f"ist konsolen-only.")


class Graphics:
    """Konsolen-only Stub des frueheren pygame-Wrappers.

    Konstruierbar, damit Konsolen-Programme + Kamera-Mathematik (CAMERA_*) ohne
    Fenster laufen. Jeder andere Zugriff (Draw/Audio/Image/Font/Layer/...) wird
    von __getattr__ als native-only abgewiesen.
    """

    def __init__(self):
        self._cam_x = 0.0
        self._cam_y = 0.0
        self._cam_zoom = 1.0
        self._gb_engine = None      # vom Interpreter gesetzt

    # ---- Kamera: reine Mathematik, kein Fenster noetig ----------------
    def set_camera(self, x, y, zoom=1.0):
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
        return (int((x - self._cam_x) * self._cam_zoom),
                int((y - self._cam_y) * self._cam_zoom))

    def _s2w(self, sx, sy):
        """Screen -> World (float)."""
        if self._cam_zoom == 0:
            return float(sx), float(sy)
        return (sx / self._cam_zoom + self._cam_x,
                sy / self._cam_zoom + self._cam_y)

    def _scale_size(self, s):
        return max(0, int(s * self._cam_zoom))

    # ---- Input/Lifecycle: harmlose No-ops fuer den Konsolen-Pfad ------
    def keys_pressed(self) -> set:
        return set()

    def shutdown(self):
        pass

    # ---- Alles andere ist native-only (gbrt) -------------------------
    def __getattr__(self, name):
        # __init__-Attribute liegen in __dict__ und kommen nie hierher.
        # Dunder normal als AttributeError (Introspektion/copy/pickle).
        if name.startswith("__") and name.endswith("__"):
            raise AttributeError(name)
        _native_only(name)
