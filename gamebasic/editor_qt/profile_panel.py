"""Ergebnis-Panel fuer den Profiler.

Zeigt die Hotpath-Auswertung eines Laufs: pro Zeile (Treffer/Zeit/Anteil +
Quelltext) oder pro Funktion. Doppelklick auf eine Zeile springt im Editor
dorthin. Speist sich aus `profiler.ProfileResult`.
"""
from __future__ import annotations

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QFont
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QHeaderView, QLabel, QPushButton, QTableWidget,
    QTableWidgetItem, QVBoxLayout, QWidget,
)

from .theme import COLORS, EDITOR_FONT_FAMILY, theme_signals


def _bar(frac: float, width: int = 10) -> str:
    n = max(0, min(width, round(frac * width)))
    return "█" * n + "░" * (width - n)


class ProfilePanel(QWidget):
    """Tabellarische Hotpath-Auswertung; Doppelklick -> Sprung."""

    jump_requested = Signal(int)   # 1-basierte Editor-Zeile

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        self._result = None
        self._mode = "lines"        # "lines" | "funcs"

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self._header = QFrame()
        hl = QHBoxLayout(self._header)
        hl.setContentsMargins(10, 6, 10, 6)
        self._title = QLabel("Profiler")
        tf = QFont(); tf.setBold(True)
        self._title.setFont(tf)
        hl.addWidget(self._title)
        self.btn_lines = QPushButton("Zeilen"); self.btn_lines.setCheckable(True)
        self.btn_lines.setChecked(True)
        self.btn_lines.clicked.connect(lambda: self._set_mode("lines"))
        self.btn_funcs = QPushButton("Funktionen"); self.btn_funcs.setCheckable(True)
        self.btn_funcs.clicked.connect(lambda: self._set_mode("funcs"))
        hl.addWidget(self.btn_lines)
        hl.addWidget(self.btn_funcs)
        hl.addStretch(1)
        self._status = QLabel("")
        hl.addWidget(self._status)
        layout.addWidget(self._header)

        self.table = QTableWidget()
        self.table.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self.table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)
        self.table.setFont(QFont(EDITOR_FONT_FAMILY))
        self.table.verticalHeader().setVisible(False)
        self.table.itemDoubleClicked.connect(self._on_double)
        layout.addWidget(self.table, 1)

        self._apply_style()
        theme_signals.changed.connect(lambda _n: self._apply_style())

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

    def clear(self) -> None:
        self.table.clear()
        self.table.setRowCount(0)
        self.table.setColumnCount(0)

    def _set_mode(self, mode: str) -> None:
        self._mode = mode
        self.btn_lines.setChecked(mode == "lines")
        self.btn_funcs.setChecked(mode == "funcs")
        self._render()

    def show_result(self, result) -> None:
        self._result = result
        total = result.total_time or 1e-9
        flag = " (abgebrochen)" if getattr(result, "stopped", False) else ""
        self.set_status(f"Gesamt {total * 1000:.1f} ms{flag}  ·  "
                        f"{len(result.lines)} Zeilen, {len(result.funcs)} Funktionen")
        self._render()

    def _render(self) -> None:
        r = self._result
        if r is None:
            return
        total = r.total_time or 1e-9
        accent = QColor(COLORS["accent"])
        if self._mode == "lines":
            cols = ["Zeile", "Treffer", "Zeit (ms)", "Anteil", "Quelltext"]
            rows = r.lines
        else:
            cols = ["Funktion", "Typ", "Aufrufe", "Zeit (ms)", "Anteil"]
            rows = r.funcs
        self.table.clear()
        self.table.setColumnCount(len(cols))
        self.table.setHorizontalHeaderLabels(cols)
        self.table.setRowCount(len(rows))

        for i, s in enumerate(rows):
            frac = s.time / total
            if self._mode == "lines":
                cells = [str(s.line), str(s.count), f"{s.time * 1000:.2f}",
                         f"{_bar(frac)} {frac * 100:4.1f}%", s.text]
                line_no = s.line
            else:
                cells = [s.name, s.kind, str(s.count), f"{s.time * 1000:.2f}",
                         f"{_bar(frac)} {frac * 100:4.1f}%"]
                line_no = s.line
            for j, text in enumerate(cells):
                it = QTableWidgetItem(text)
                it.setData(Qt.ItemDataRole.UserRole, line_no)
                if j in (0, 1, 2) and self._mode == "lines":
                    it.setTextAlignment(Qt.AlignmentFlag.AlignRight
                                        | Qt.AlignmentFlag.AlignVCenter)
                # Top-3-Hotspots farblich hervorheben.
                if i < 3 and j == (3 if self._mode == "lines" else 4):
                    it.setForeground(accent)
                self.table.setItem(i, j, it)
        hdr = self.table.horizontalHeader()
        hdr.setSectionResizeMode(QHeaderView.ResizeMode.ResizeToContents)
        last = len(cols) - 1
        hdr.setSectionResizeMode(last, QHeaderView.ResizeMode.Stretch)

    def _on_double(self, item: QTableWidgetItem) -> None:
        line = item.data(Qt.ItemDataRole.UserRole)
        if isinstance(line, int) and line > 0:
            self.jump_requested.emit(line)
