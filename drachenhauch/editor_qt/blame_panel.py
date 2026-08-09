"""Git-Blame-Panel: zeigt pro Zeile Commit/Autor/Datum/Zusammenfassung.

Synchronisiert sich mit dem Editor-Cursor (markiert die aktuelle Zeile) und
springt per Doppelklick in den Editor. Speist sich aus `gitinfo.BlameResult`.
"""
from __future__ import annotations

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QFont
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QHeaderView, QLabel, QPushButton, QTableWidget,
    QTableWidgetItem, QVBoxLayout, QWidget,
)

from .theme import COLORS, EDITOR_FONT_FAMILY, theme_signals


class BlamePanel(QWidget):
    """Tabellarische Blame-Ansicht; Doppelklick -> Sprung, Cursor-Sync."""

    jump_requested = Signal(int)          # 1-basierte Zeile
    refresh_requested = Signal()

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        self._line_to_row: dict[int, int] = {}

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self._header = QFrame()
        hl = QHBoxLayout(self._header)
        hl.setContentsMargins(10, 6, 10, 6)
        self._title = QLabel("Git-Blame")
        tf = QFont(); tf.setBold(True)
        self._title.setFont(tf)
        hl.addWidget(self._title)
        self.btn_refresh = QPushButton("Aktualisieren")
        self.btn_refresh.clicked.connect(self.refresh_requested.emit)
        hl.addWidget(self.btn_refresh)
        hl.addStretch(1)
        self._status = QLabel("")
        hl.addWidget(self._status)
        layout.addWidget(self._header)

        self.table = QTableWidget()
        self.table.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self.table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)
        self.table.setSelectionMode(QTableWidget.SelectionMode.SingleSelection)
        self.table.setFont(QFont(EDITOR_FONT_FAMILY))
        self.table.verticalHeader().setVisible(False)
        self.table.setColumnCount(5)
        self.table.setHorizontalHeaderLabels(
            ["Zeile", "Commit", "Autor", "Datum", "Zusammenfassung"])
        self.table.itemDoubleClicked.connect(self._on_double)
        layout.addWidget(self.table, 1)

        self._apply_style()
        # Bound-Method statt Lambda (Use-after-free-Fix, siehe breadcrumbs.py).
        theme_signals.changed.connect(self._apply_style)

    def _apply_style(self) -> None:
        c = COLORS
        self._header.setStyleSheet(f"background-color: {c['bg_panel']};")
        self._title.setStyleSheet(f"color: {c['fg']};")
        self._status.setStyleSheet(f"color: {c['fg_muted']};")
        self.table.setStyleSheet(
            f"QTableWidget {{ background-color: {c['bg_alt']}; color: {c['fg']}; "
            f"border: 0; outline: 0; gridline-color: {c['border']}; }} "
            f"QTableWidget::item {{ padding: 2px 4px; }} "
            f"QHeaderView::section {{ background-color: {c['bg_panel']}; "
            f"color: {c['fg_muted']}; border: 0; padding: 4px; }}")

    def set_status(self, text: str) -> None:
        self._status.setText(text)

    def show_message(self, text: str) -> None:
        """Leert die Tabelle und zeigt nur eine Statusmeldung (z.B. kein Repo)."""
        self.table.setRowCount(0)
        self._line_to_row = {}
        self.set_status(text)

    def show_result(self, result) -> None:
        self._line_to_row = {}
        muted = QColor(COLORS["fg_muted"])
        accent = QColor(COLORS["accent"])
        lines = result.lines
        self.table.setRowCount(len(lines))
        for i, bl in enumerate(lines):
            self._line_to_row[bl.line] = i
            commit = "•••" if bl.uncommitted else bl.sha
            author = "(uncommitted)" if bl.uncommitted else bl.author
            cells = [str(bl.line), commit, author, bl.date, bl.summary]
            for j, text in enumerate(cells):
                it = QTableWidgetItem(text)
                it.setData(Qt.ItemDataRole.UserRole, bl.line)
                if j == 0:
                    it.setTextAlignment(Qt.AlignmentFlag.AlignRight
                                        | Qt.AlignmentFlag.AlignVCenter)
                if bl.uncommitted:
                    it.setForeground(muted)
                elif j == 1:
                    it.setForeground(accent)
                self.table.setItem(i, j, it)
        hdr = self.table.horizontalHeader()
        hdr.setSectionResizeMode(QHeaderView.ResizeMode.ResizeToContents)
        hdr.setSectionResizeMode(4, QHeaderView.ResizeMode.Stretch)
        n_commits = len({bl.sha for bl in lines if not bl.uncommitted})
        self.set_status(f"{len(lines)} Zeilen · {n_commits} Commits")

    def set_current_line(self, line: int) -> None:
        """Markiert die Zeile `line` (vom Editor-Cursor) und scrollt hin."""
        row = self._line_to_row.get(line)
        if row is None:
            return
        self.table.blockSignals(True)
        self.table.selectRow(row)
        self.table.blockSignals(False)
        item = self.table.item(row, 0)
        if item is not None:
            self.table.scrollToItem(item)

    def _on_double(self, item: QTableWidgetItem) -> None:
        line = item.data(Qt.ItemDataRole.UserRole)
        if isinstance(line, int) and line > 0:
            self.jump_requested.emit(line)
