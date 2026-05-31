"""Builtins-Panel: gruppiertes Verzeichnis aller Built-ins, Sprach-
Schluesselwoerter und Konstanten. Doppelklick fuegt den Namen am Cursor
ein.

Implementierung als `QTreeWidget` -- nativer Dark-/Light-Style ueber
das App-QSS, plus farbige Items je nach Kategorie (gleiche Farben wie
der Editor-Highlighter).
"""
from __future__ import annotations

import re

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QBrush, QColor, QFont, QIcon, QPainter, QPen, QPixmap
from PySide6.QtWidgets import (
    QFrame, QLabel, QLineEdit, QTreeWidget, QTreeWidgetItem, QVBoxLayout,
    QWidget,
)

from .theme import COLORS, theme_signals


def _color_swatch_icon(hex_color: str, size: int = 14) -> QIcon:
    """Liefert ein kleines Quadrat-Icon mit `hex_color` als Fuellung."""
    pix = QPixmap(size * 2, size * 2)
    pix.setDevicePixelRatio(2.0)
    pix.fill(Qt.GlobalColor.transparent)
    p = QPainter(pix)
    p.setRenderHint(QPainter.RenderHint.Antialiasing, False)
    p.fillRect(1, 1, size - 2, size - 2, QColor(hex_color))
    pen = QPen(QColor(COLORS["border"]))
    pen.setWidth(1)
    p.setPen(pen)
    p.setBrush(Qt.BrushStyle.NoBrush)
    p.drawRect(0, 0, size - 1, size - 1)
    p.end()
    return QIcon(pix)


_HEX_IN_SIG_RE = re.compile(r"#([0-9A-Fa-f]{6})")


# Reihenfolge der Gruppen im Panel.
_GROUP_ORDER = [
    "anweisungen", "deklarationen", "operatoren", "typen", "literale",
    "standard", "grafik", "konstanten", "farben", "tasten",
]


class BuiltinsPanel(QWidget):
    insert_requested = Signal(str)  # Identifier (uppercase)

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        self._all_items: list[tuple[str, str, str, str]] = []  # (group, name, sig, color_key)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self.header = QLabel("Built-ins")
        self.header.setStyleSheet(self._header_style())
        layout.addWidget(self.header)

        self.filter_entry = QLineEdit()
        self.filter_entry.setPlaceholderText("filtern ...")
        self.filter_entry.textChanged.connect(self._render)
        wrap = QFrame()
        wl = QVBoxLayout(wrap)
        wl.setContentsMargins(6, 6, 6, 4)
        wl.addWidget(self.filter_entry)
        layout.addWidget(wrap)

        self.tree = QTreeWidget()
        self.tree.setHeaderHidden(True)
        self.tree.setRootIsDecorated(True)
        self.tree.setUniformRowHeights(True)
        self.tree.setFont(QFont("Consolas", 10))
        self.tree.itemDoubleClicked.connect(self._on_double_clicked)
        self.tree.itemActivated.connect(self._on_double_clicked)
        layout.addWidget(self.tree, 1)

        self._collect()
        self._render()

        theme_signals.changed.connect(self._on_theme_changed)

    def _header_style(self) -> str:
        return (
            f"background-color: {COLORS['bg_panel']}; color: {COLORS['fg']}; "
            "font-weight: bold; padding: 6px 10px;"
        )

    def _on_theme_changed(self, _name: str) -> None:
        self.header.setStyleSheet(self._header_style())
        self._render()

    # ---------------------------------------------------- Datasource
    def _collect(self) -> None:
        groups: dict[str, list[tuple[str, str, str]]] = {}
        seen_ids: set[int] = set()

        # 1) Funktions-Built-ins
        try:
            from ..interpreter import BUILTINS, GRAPHICS_BUILTINS
            from ..builtins_registry import signature_text
        except Exception:
            BUILTINS = GRAPHICS_BUILTINS = {}
            def signature_text(_n):  # type: ignore
                return ""

        for src_dict in (BUILTINS, GRAPHICS_BUILTINS):
            for wrapper in src_dict.values():
                wid = id(wrapper)
                if wid in seen_ids:
                    continue
                seen_ids.add(wid)
                primary = getattr(wrapper, "_gb_primary", None)
                if primary is None:
                    continue
                inner = getattr(wrapper, "__wrapped__", None)
                mod = getattr(inner, "__module__", "") if inner else ""
                kind = getattr(wrapper, "_gb_kind", "core")
                if mod.startswith("gamebasic.modules."):
                    group = mod.rsplit(".", 1)[-1]
                elif kind == "graphics":
                    group = "grafik"
                else:
                    group = "standard"
                sig = signature_text(primary) or primary
                groups.setdefault(group, []).append((primary, sig, "builtin"))

        # 2) Sprachkonstrukte
        anweisungen = [
            "IF", "ELSE", "ELSEIF", "END IF",
            "FOR", "TO", "STEP", "NEXT",
            "WHILE", "WEND",
            "REPEAT", "UNTIL",
            "SELECT CASE", "CASE", "CASE ELSE", "END SELECT",
            "BREAK", "CONTINUE", "RETURN",
            "TRY", "CATCH", "THROW", "END TRY",
            "PRINT", "INPUT",
            "DATA", "READ", "RESTORE",
        ]
        for kw in anweisungen:
            groups.setdefault("anweisungen", []).append((kw, kw, "kw_ctrl"))

        deklarationen = [
            "DIM", "AS", "CONST",
            "SUB", "END SUB",
            "FUNCTION", "END FUNCTION",
            "CLASS", "END CLASS", "EXTENDS", "NEW",
            "STRUCT", "END STRUCT",
            "IMPORT",
            "ARRAY OF", "MAP OF",
        ]
        for kw in deklarationen:
            groups.setdefault("deklarationen", []).append((kw, kw, "kw_decl"))

        for kw in ("AND", "OR", "NOT", "MOD", "BAND", "BOR", "BXOR", "BNOT", "SHL", "SHR"):
            groups.setdefault("operatoren", []).append((kw, kw, "kw_decl"))

        for kw in ("INTEGER", "FLOAT", "STRING", "BOOLEAN", "IMAGE", "SOUND", "FILE"):
            groups.setdefault("typen", []).append((kw, kw, "type"))

        for kw in ("TRUE", "FALSE"):
            groups.setdefault("literale", []).append((kw, kw, "bool"))

        # 3) Konstanten
        try:
            from ..graphics import COLORS as _C, KEYS as _K
            import math as _math
            groups.setdefault("konstanten", []).append(
                ("PI", f"PI = {_math.pi:.6f}", "number"),
            )
            for n, val in _C.items():
                up = n.upper()
                groups.setdefault("farben", []).append(
                    (up, f"{up} = #{val:06X}", "number"),
                )
            for n, val in _K.items():
                up = n.upper()
                groups.setdefault("tasten", []).append(
                    (up, f"{up} = {val}", "number"),
                )
        except Exception:
            pass

        order = list(_GROUP_ORDER)
        order += sorted(g for g in groups if g not in order)
        self._all_items = []
        for g in order:
            if g not in groups:
                continue
            for name, sig, color in sorted(groups[g], key=lambda t: t[0]):
                self._all_items.append((g, name, sig, color))

    # --------------------------------------------------- Rendering
    def _render(self, *_args) -> None:
        q = self.filter_entry.text().strip().lower()
        filtered: dict[str, list[tuple[str, str, str]]] = {}
        for group, name, sig, color in self._all_items:
            if q and q not in name.lower() and q not in group.lower():
                continue
            filtered.setdefault(group, []).append((name, sig, color))

        # Original-Reihenfolge erhalten
        order: list[str] = []
        for group, _n, _s, _c in self._all_items:
            if group in filtered and group not in order:
                order.append(group)

        self.tree.clear()
        force_open = bool(q)
        muted = QColor(COLORS["fg_muted"])
        for group in order:
            items = filtered[group]
            head = QTreeWidgetItem([f"{group.upper()}  ({len(items)})"])
            head.setForeground(0, muted)
            head_font = head.font(0)
            head_font.setBold(True)
            head.setFont(0, head_font)
            head.setFlags(Qt.ItemFlag.ItemIsEnabled)
            self.tree.addTopLevelItem(head)
            for name, sig, color_key in items:
                if sig.startswith(name):
                    args_part = sig[len(name):]
                else:
                    args_part = ""
                display = name + args_part
                child = QTreeWidgetItem([display])
                child.setForeground(0, QColor(COLORS.get(color_key, COLORS["builtin"])))
                child.setData(0, Qt.ItemDataRole.UserRole, name)
                # Color-Swatch fuer Eintraege der "farben"-Gruppe.
                if group == "farben":
                    m = _HEX_IN_SIG_RE.search(sig)
                    if m is not None:
                        child.setIcon(0, _color_swatch_icon(f"#{m.group(1)}"))
                head.addChild(child)
            head.setExpanded(force_open)

    def _on_double_clicked(self, item: QTreeWidgetItem, _column: int = 0) -> None:
        name = item.data(0, Qt.ItemDataRole.UserRole)
        if isinstance(name, str) and name:
            self.insert_requested.emit(name)
