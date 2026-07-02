"""Branding-Helpers: Logo laden und skalieren.

Wird vom Sprite-Editor verwendet, um das Master-Logo aus
`gamebasic/assets/logo.png` als App-Icon anzuzeigen. Pure Pillow-Logik
ohne UI-Toolkit-Bindung -- der Aufrufer macht aus dem zurueckgegebenen
PIL-Image das, was sein Toolkit braucht (QPixmap, etc.).
"""
from __future__ import annotations

from pathlib import Path

# Pillow ist optional. Falls nicht installiert: `is_available()` liefert
# False und das aufrufende UI laeuft ohne Logo weiter.
try:
    from PIL import Image
    _PIL_AVAILABLE = True
except ImportError:  # pragma: no cover
    Image = None  # type: ignore[assignment]
    _PIL_AVAILABLE = False


def _logo_path() -> Path:
    """Pfad zum Master-Logo. Existiert nicht zwangslaeufig."""
    return Path(__file__).resolve().parent / "assets" / "logo.png"


def is_available() -> bool:
    """True wenn Pillow installiert ist UND das Logo-File existiert."""
    return _PIL_AVAILABLE and _logo_path().exists()


def _smart_square_crop(img):
    """Croppt ein nicht-quadratisches Bild auf einen quadratischen Bereich.

    Heuristik fuer Branding-Screenshots, die typischerweise das Symbol
    links und den Schriftzug rechts haben: nimm das linke Bild-Drittel
    bis halbe Breite als quadratischen Crop. Falls das Bild bereits
    annaehernd quadratisch ist (< 10 % Verhaeltnis-Differenz), einfach
    durchreichen.
    """
    w, h = img.size
    if abs(w - h) < min(w, h) * 0.1:
        return img
    if w > h:
        # Breit -> Crop H x H, leicht links der Mitte
        anchor = int(w * 0.25)
        half = h // 2
        left = max(0, anchor - half)
        right = left + h
        if right > w:
            right = w
            left = right - h
        return img.crop((left, 0, right, h))
    else:
        # Hoch -> Crop W x W, vertikal zentriert
        top = (h - w) // 2
        return img.crop((0, top, w, top + w))


def scaled_logo_image(target_w: int):
    """Laedt das Master-Logo und skaliert es proportional auf `target_w`
    Pixel Breite. `None` wenn Pillow fehlt oder das Logo nicht existiert
    (`is_available()` False). Reiner PIL-Rueckgabewert -- der Aufrufer
    wandelt ihn in sein UI-Toolkit-Format um (z.B. QPixmap via ImageQt).
    Frueher duplizierte jeder Qt-Aufrufer (AboutDialog, WelcomePanel) das
    Laden+Skalieren identisch, nur `target_w` unterschied sich."""
    if not is_available():
        return None
    pil = Image.open(_logo_path()).convert("RGBA")
    ratio = target_w / pil.size[0]
    target_h = max(1, int(pil.size[1] * ratio))
    return pil.resize((target_w, target_h), Image.LANCZOS)
