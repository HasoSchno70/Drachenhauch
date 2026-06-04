"""Variablen-Panel fuer den Debugger.

Zeigt beim Pausieren die lokalen und globalen Variablen (Name / Wert / Typ)
als zwei Gruppen in einem Baum. Speist sich aus dem Snapshot-Dict, das
`DebugController.paused` liefert (`{"locals": [...], "globals": [...]}`,
jeder Eintrag `(name, type, value_str)`).
"""
from __future__ import annotations

from PySide6.QtCore import Qt
from PySide6.QtGui import QColor, QFont
from PySide6.QtWidgets import (
    QFrame, QLabel, QTreeWidget, QTreeWidgetItem, QVBoxLayout, QWidget,
)

from .theme import COLORS, EDITOR_FONT_FAMILY, theme_signals


class VariablesPanel(QWidget):
    """Locals/Globals-Baum waehrend einer Debug-Sitzung."""

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self._header = QFrame()
        from PySide6.QtWidgets import QHBoxLayout
        hl = QHBoxLayout(self._header)
        hl.setContentsMargins(10, 6, 10, 6)
        self._title = QLabel("Variablen")
        tf = QFont()
        tf.setBold(True)
        self._title.setFont(tf)
        hl.addWidget(self._title)
        hl.addStretch(1)
        self._status = QLabel("")
        hl.addWidget(self._status)
        layout.addWidget(self._header)

        self.tree = QTreeWidget()
        self.tree.setHeaderLabels(["Name", "Wert", "Typ"])
        self.tree.setRootIsDecorated(True)
        self.tree.setUniformRowHeights(True)
        self.tree.setColumnWidth(0, 160)
        self.tree.setColumnWidth(1, 220)
        self.tree.setFont(QFont(EDITOR_FONT_FAMILY))
        layout.addWidget(self.tree, 1)

        self._apply_style()
        theme_signals.changed.connect(lambda _n: self._apply_style())

    def _apply_style(self) -> None:
        c = COLORS
        self._header.setStyleSheet(f"background-color: {c['bg_panel']};")
        self._title.setStyleSheet(f"color: {c['fg']};")
        self._status.setStyleSheet(f"color: {c['fg_muted']};")
        self.tree.setStyleSheet(
            f"QTreeWidget {{ background-color: {c['bg_alt']}; color: {c['fg']}; "
            f"border: 0; outline: 0; }} "
            f"QTreeWidget::item {{ padding: 2px 4px; }} "
            f"QHeaderView::section {{ background-color: {c['bg_panel']}; "
            f"color: {c['fg_muted']}; border: 0; padding: 4px; }}"
        )

    def set_status(self, text: str) -> None:
        self._status.setText(text)

    def clear(self) -> None:
        self.tree.clear()

    def update_frames(self, frames: dict) -> None:
        self.tree.clear()
        accent = QColor(COLORS["accent"])
        bold = QFont()
        bold.setBold(True)

        def add_group(label: str, rows: list, expand: bool) -> None:
            grp = QTreeWidgetItem([f"{label} ({len(rows)})"])
            grp.setForeground(0, accent)
            grp.setFont(0, bold)
            grp.setFirstColumnSpanned(True)
            self.tree.addTopLevelItem(grp)
            for name, typ, value in rows:
                grp.addChild(QTreeWidgetItem([name, value, str(typ).upper()]))
            grp.setExpanded(expand)

        add_group("Lokal", frames.get("locals", []), True)
        add_group("Global", frames.get("globals", []), True)
