"""Built-in-Modul-Namen fuer GameBasic.

Stufe B: Die Python-Modul-Implementierungen wurden entfernt -- gbrt (Rust)
implementiert alle Built-in-Module nativ. Dieses Paket haelt nur noch die
*Namensliste*, die `preprocess.py` braucht, um `IMPORT "<modul>"` als Built-in
zu erkennen (und zu einem Kommentar zu machen), sowie der Editor-File-Browser
zum Anzeigen verfuegbarer Module.

Maßgeblich ist `rust/gb_runtime/src/preprocess.rs` MODULES -- bei neuen Modulen
hier UND dort ergaenzen (synchron halten).
"""
from __future__ import annotations

import re


# Modul-Namen muessen aus alphanumerischen Zeichen + Unterstrich bestehen
# (Buchstabe/Underscore am Anfang). Verhindert, dass z.B. "lib/foo" oder
# "../etc/passwd" als Modul-Name akzeptiert werden.
_MODULE_NAME_RE = re.compile(r"^[a-zA-Z][a-zA-Z0-9_]*$")

# Statische Liste der Built-in-Modul-Namen (= gbrts preprocess.rs MODULES).
KNOWN_MODULES: frozenset = frozenset({
    "astar", "audio", "bt", "camera", "controller", "curves", "db", "ecs",
    "g3d", "gui", "html", "imgfx", "input", "json", "m3d", "net", "particles",
    "physics", "physics3d", "regex", "save", "scene", "serial", "sprite",
    "tile_collide", "tiled", "tween", "ui", "usb", "vec2", "wifi",
})


def is_valid_module_name(name: str) -> bool:
    """True, wenn `name` ein syntaktisch gueltiger Modul-Name ist."""
    return bool(_MODULE_NAME_RE.match(name))


def is_known_module(name: str) -> bool:
    """True, wenn `name` ein Built-in-Modul ist (statische Liste, kein Import)."""
    return name.lower() in KNOWN_MODULES


def discover_modules() -> list:
    """Namen aller Built-in-Module (sortiert). Rein deklarativ -- gbrt
    implementiert sie nativ; der Editor-File-Browser nutzt nur die Liste."""
    return sorted(KNOWN_MODULES)
