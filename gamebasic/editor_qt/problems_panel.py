"""Probleme-Panel.

Listet ALLE Live-Diagnosen des aktiven Editors (Errors + Warnungen aus
`gbrt --check`) -- nicht nur das erste Problem wie die Statusbar. Klick
springt zur Zeile.
"""
from __future__ import annotations

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QFont
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QListWidget, QListWidgetItem, QVBoxLayout,
    QWidget,
)

from .theme import COLORS, theme_signals


class ProblemsPanel(QWidget):
    """Liste der Diagnosen (Errors/Warnungen) des aktiven Editors."""

    jump_requested = Signal(int)   # Zeilennummer (1-basiert)

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self._header = QFrame()
        hl = QHBoxLayout(self._header)
        hl.setContentsMargins(10, 6, 10, 6)
        self._title = QLabel("Probleme")
        tf = QFont()
        tf.setBold(True)
        self._title.setFont(tf)
        hl.addWidget(self._title)
        hl.addStretch(1)
        self._count = QLabel("0")
        hl.addWidget(self._count)
        layout.addWidget(self._header)

        self.list = QListWidget()
        self.list.itemActivated.connect(self._on_clicked)
        self.list.itemClicked.connect(self._on_clicked)
        layout.addWidget(self.list, 1)

        self._empty_hint = "Keine Probleme"
        self._apply_style()
        theme_signals.changed.connect(lambda _n: self._apply_style())

    def _apply_style(self) -> None:
        c = COLORS
        self._header.setStyleSheet(f"background-color: {c['bg_panel']};")
        self._title.setStyleSheet(f"color: {c['fg']};")
        self._count.setStyleSheet(f"color: {c['fg_muted']};")
        self.list.setStyleSheet(
            f"QListWidget {{ background-color: {c['bg_alt']}; color: {c['fg']}; "
            f"border: 0; outline: 0; }} "
            f"QListWidget::item {{ padding: 3px 6px; }} "
            f"QListWidget::item:hover {{ background-color: {c['bg_hover']}; }} "
            f"QListWidget::item:selected {{ background-color: {c['sel']}; }}"
        )

    def set_problems(self, problems) -> None:
        """`problems`: Liste von ParseProblem (line, message, severity, phase)."""
        self.list.clear()
        problems = list(problems or [])
        # Errors zuerst, dann nach Zeile -- die wichtigsten oben.
        problems.sort(key=lambda p: (0 if p.severity == "error" else 1, p.line))
        for p in problems:
            is_err = p.severity == "error"
            icon = "✕" if is_err else "⚠"
            it = QListWidgetItem(f"{icon}  Zeile {p.line}: {p.message}")
            it.setData(Qt.ItemDataRole.UserRole, int(p.line))
            it.setForeground(QColor(COLORS["error"] if is_err else COLORS["warning"]))
            it.setToolTip(f"{p.phase}: {p.message}")
            self.list.addItem(it)
        n = len(problems)
        self._count.setText(str(n))
        if n == 0:
            placeholder = QListWidgetItem(self._empty_hint)
            placeholder.setForeground(QColor(COLORS["fg_muted"]))
            placeholder.setFlags(Qt.ItemFlag.NoItemFlags)
            self.list.addItem(placeholder)

    def _on_clicked(self, item: QListWidgetItem) -> None:
        line = item.data(Qt.ItemDataRole.UserRole)
        if line is not None:
            self.jump_requested.emit(int(line))
