"""Form-Designer (WYSIWYG, Xojo-Stil) -- Qt-freies Datenmodell.

`document.py` haelt das Formular-Modell (`FormDoc` + `Control`), liest/schreibt
das `.dhform`-JSON **exakt im Runtime-Format** (das `GUI_LOAD`/`GUI_FROM_JSON`
versteht -- siehe `rust/drachenhauch_runtime/src/gui.rs`) und generiert lauffaehigen
Drachenhauch-Code (Harness + Event-Handler-Stubs).

Die Qt-UI liegt in `drachenhauch/formdesigner_qt.py`; dieses Paket bleibt Qt-frei
und damit headless testbar.
"""
from .document import (
    Control, FormDoc, FormProject, History, PALETTE, palette_spec, GRID,
    HANDLES, snap, resize_rect, FORM_THEMES, FORM_THEME_COLORS, theme_colors,
    EVENTS, hex_zu_int,
)

__all__ = [
    "Control", "FormDoc", "FormProject", "History", "PALETTE", "palette_spec",
    "GRID", "HANDLES", "snap", "resize_rect", "hex_zu_int",
    "FORM_THEMES", "FORM_THEME_COLORS", "theme_colors", "EVENTS",
]
