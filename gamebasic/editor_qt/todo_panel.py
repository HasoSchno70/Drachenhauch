"""TODO/FIXME-Panel.

Scannt den aktiven Editor nach Markern (TODO/FIXME/HACK/XXX/BUG) in
Kommentaren (`'...` oder `REM ...`) und listet sie. Klick springt zur Zeile.
"""
from __future__ import annotations

import re

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QFont
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QListWidget, QListWidgetItem, QVBoxLayout,
    QWidget,
)

from .theme import COLORS, theme_signals

# Marker im Kommentar-Teil einer Zeile.
_MARKER_RE = re.compile(r"\b(TODO|FIXME|HACK|XXX|BUG)\b\s*:?\s*(.*)", re.IGNORECASE)


def _comment_part(line: str) -> str | None:
    """Kommentar-Text einer Zeile (nach `'` oder `REM`), grob -- ignoriert
    Hochkommas in Strings nicht perfekt, reicht aber fuer Marker-Scan."""
    # einfachster robuster Ansatz: erstes ' das nicht in "..." steht.
    in_str = False
    for i, ch in enumerate(line):
        if ch == '"':
            in_str = not in_str
        elif ch == "'" and not in_str:
            return line[i + 1:]
    # REM-Kommentar
    m = re.match(r"\s*REM\b(.*)", line, re.IGNORECASE)
    if m:
        return m.group(1)
    return None


def scan_todos(source: str) -> list[tuple[int, str, str]]:
    """Liefert `(line_1based, marker, text)` je gefundenem Marker."""
    out: list[tuple[int, str, str]] = []
    for idx, line in enumerate(source.split("\n"), start=1):
        comment = _comment_part(line)
        if comment is None:
            continue
        m = _MARKER_RE.search(comment)
        if m:
            out.append((idx, m.group(1).upper(), m.group(2).strip()))
    return out


class TodoPanel(QWidget):
    """Liste der TODO/FIXME-Marker des aktiven Editors."""

    jump_requested = Signal(int)   # Zeilennummer (1-basiert)

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self._header = QFrame()
        hl = QHBoxLayout(self._header)
        hl.setContentsMargins(10, 6, 10, 6)
        self._title = QLabel("TODO / FIXME")
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

    def update_for(self, source: str | None) -> None:
        self.list.clear()
        items = scan_todos(source) if source else []
        marker_col = {
            "TODO": COLORS["info"], "FIXME": COLORS["danger"],
            "BUG": COLORS["danger"], "HACK": COLORS["builtin"],
            "XXX": COLORS["builtin"],
        }
        for line, marker, text in items:
            label = f"{marker}  {text}" if text else marker
            it = QListWidgetItem(f"{label}")
            it.setData(Qt.ItemDataRole.UserRole, line)
            it.setForeground(QColor(marker_col.get(marker, COLORS["fg"])))
            it.setToolTip(f"Zeile {line}: {marker} {text}")
            self.list.addItem(it)
        self._count.setText(str(len(items)))

    def _on_clicked(self, item: QListWidgetItem) -> None:
        line = item.data(Qt.ItemDataRole.UserRole)
        if line is not None:
            self.jump_requested.emit(int(line))
