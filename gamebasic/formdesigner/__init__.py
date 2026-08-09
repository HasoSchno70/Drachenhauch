"""Form-Designer (WYSIWYG, Xojo-Stil) -- Qt-freies Datenmodell.

`document.py` haelt das Formular-Modell (`FormDoc` + `Control`), liest/schreibt
das `.gbform`-JSON **exakt im Runtime-Format** (das `GUI_LOAD`/`GUI_FROM_JSON`
versteht -- siehe `rust/drachenhauch_runtime/src/gui.rs`) und generiert lauffaehigen
GameBasic-Code (Harness + Event-Handler-Stubs).

Die Qt-UI liegt in `gamebasic/formdesigner_qt.py`; dieses Paket bleibt Qt-frei
und damit headless testbar.
"""
from .document import (
    Control, FormDoc, FormProject, History, PALETTE, palette_spec, GRID,
    HANDLES, snap, resize_rect, FORM_THEMES, FORM_THEME_COLORS, theme_colors,
    EVENTS,
)

__all__ = [
    "Control", "FormDoc", "FormProject", "History", "PALETTE", "palette_spec",
    "GRID", "HANDLES", "snap", "resize_rect",
    "FORM_THEMES", "FORM_THEME_COLORS", "theme_colors", "EVENTS",
]
