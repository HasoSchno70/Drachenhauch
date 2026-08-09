"""Tool-Context-Protocol fuer den Sprite-Editor.

Tools (Pencil/Eraser/Bucket/...) nehmen einen `app`-Parameter und greifen
auf eine Reihe von Attributen und Methoden zu (siehe `tools.py`-Docstring).
Das war historisch implizites Duck-Typing; hier formalisieren wir das
erwartete Interface als `typing.Protocol`.

**Wertbeitrag:**
- Klare Dokumentation: was muss ein "Tool-Host" liefern?
- IDE-Autocomplete fuer Tool-Authoren beim Implementieren neuer Tools.
- Optional Type-Checking via mypy / pyright -- Tools koennen das
  Protocol als Type-Hint nutzen.

**Nicht-Ziele:**
- Keine Laufzeit-Pruefung (Python-Protocols sind structural, kein abstract
  base class).
- Keine API-Aenderung an existierenden Tools -- der `app`-Parameter
  bleibt; Tools laufen wie bisher.

Wer das Protocol fuer typed Tools nutzen will:

    from drachenhauch.spriteeditor.tool_context import ToolHost

    class MyTool(Tool):
        def begin(self, app: ToolHost, x: int, y: int, button) -> None:
            app.doc.current.snapshot()
            ...
"""
from __future__ import annotations

from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class _StatusBarProto(Protocol):
    """Minimaler Statusbar-Vertrag -- showMessage(text, timeout_ms)."""
    def showMessage(self, text: str, timeout_ms: int = 0) -> Any: ...


@runtime_checkable
class _CanvasProto(Protocol):
    """Subset der `SpriteCanvas`-API, das Tools tatsaechlich nutzen."""
    def invalidate_all(self) -> None: ...
    def set_preview(self, pil) -> None: ...
    def set_selection(self, x0: int, y0: int, x1: int, y1: int) -> None: ...
    def get_selection(self) -> Any: ...
    def set_selection_mask(self, mask) -> None: ...
    def get_selection_mask(self) -> Any: ...
    def clear_selection(self) -> None: ...


@runtime_checkable
class _DocProto(Protocol):
    """Subset der `SpriteDoc`-API, das Tools tatsaechlich nutzen."""
    width: int
    height: int
    current: Any   # `Frame`-Instanz mit `.pixels`, `.snapshot()`


@runtime_checkable
class ToolHost(Protocol):
    """Vertrag, den ein "Host" fuer ein Tool erfuellen muss.

    Diese Schnittstelle liefert die `SpriteEditorWindow`-Klasse zur Laufzeit.
    Tests koennen einen Mock implementieren (siehe test_spriteeditor_tools.py:
    `_MockApp` -- die ist Protocol-konform).
    """
    # Document + Canvas
    doc: _DocProto
    canvas: _CanvasProto

    # Aktuelle Vorder-/Hintergrundfarbe (RGBA-Tupel)
    fg: Any
    bg: Any

    # Brush-Eigenschaften
    brush_size: int
    symmetry_mode: str   # "none", "x", "y", "both"

    # Tool-Aktivierungs-Hilfen (z.B. Eyedropper springt zurueck zum
    # vorherigen Tool nach dem Pick).
    _previous_tool_name: Any

    # --- Methoden ---
    def in_bounds(self, x: int, y: int) -> bool: ...
    def mark_dirty(self) -> None: ...
    def set_fg(self, color: Any) -> None: ...
    def set_bg(self, color: Any) -> None: ...
    def activate_tool(self, name: str, _silent: bool = False) -> None: ...
    def statusBar(self) -> _StatusBarProto: ...
    def show_selection_context_menu(self, global_pos: Any) -> None: ...


# Praktisch fuer Type-Hints: TYPE_CHECKING-Import vermeidet Zirkular-Imports
# zur Laufzeit, aber gibt Editor-Tools die richtige Auto-Completion.
TOOL_HOST_DOC = """\
Was ein Tool-Host (typischerweise SpriteEditorWindow) liefern muss:

  Attribute:
    doc: SpriteDoc   -- aktuelles Dokument mit width/height/current
    canvas: SpriteCanvas -- Canvas-API fuer Preview/Selection/Invalidate
    fg, bg: RGBA-Tupel -- Vorder-/Hintergrundfarbe
    brush_size: int  -- 1..N
    symmetry_mode: str -- "none" | "x" | "y" | "both"
    _previous_tool_name: str | None -- fuer Eyedropper-Rueckkehr

  Methoden:
    in_bounds(x, y) -> bool
    mark_dirty() -> None
    set_fg(rgba) / set_bg(rgba) -> None
    activate_tool(name, _silent=False) -> None
    statusBar() -> Qt-StatusBar (mit showMessage)
    show_selection_context_menu(global_pos) -> None
"""
