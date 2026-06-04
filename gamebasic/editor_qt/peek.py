"""Peek-Definition: zeigt die Definition eines Symbols in einem Popup,
ohne zur Stelle zu springen.

Komplement zu Goto-Definition (F12, springt) und Hover-Doku (zeigt
Signatur+Doc). Peek (Alt+F12) zeigt zusaetzlich den Anfang des Bodys
direkt am Cursor -- praktisch um kurz nachzuschauen, ohne den Kontext
zu verlieren. Inhalt kommt aus `editor_qt.symbols`.
"""
from __future__ import annotations

from PySide6.QtCore import QPoint, Qt, Signal
from PySide6.QtGui import QFont
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QPlainTextEdit, QPushButton, QVBoxLayout,
)

from .highlighter import GBHighlighter
from .theme import COLORS, EDITOR_FONT_FAMILY

_MAX_BODY_LINES = 12


class PeekPopup(QFrame):
    """Schwebendes Popup mit Signatur, Doc und Body-Auszug einer Definition."""

    go_to = Signal(int)   # 1-basierte Zeile

    def __init__(self, parent=None):
        super().__init__(parent, Qt.WindowType.Popup)
        self._target_line = 0
        self.setObjectName("PeekPopup")
        self.setStyleSheet(
            f"#PeekPopup {{ background: {COLORS['tooltip_bg']}; "
            f"border: 1px solid {COLORS['accent']}; border-radius: 6px; }}")
        lay = QVBoxLayout(self)
        lay.setContentsMargins(10, 8, 10, 8)
        lay.setSpacing(6)

        self.sig_label = QLabel()
        sf = QFont(EDITOR_FONT_FAMILY); sf.setBold(True)
        self.sig_label.setFont(sf)
        self.sig_label.setStyleSheet(f"color: {COLORS['accent']};")
        self.sig_label.setTextInteractionFlags(
            Qt.TextInteractionFlag.TextSelectableByMouse)
        lay.addWidget(self.sig_label)

        self.doc_label = QLabel()
        self.doc_label.setWordWrap(True)
        self.doc_label.setStyleSheet(f"color: {COLORS['fg_muted']};")
        lay.addWidget(self.doc_label)

        self.body = QPlainTextEdit()
        self.body.setReadOnly(True)
        self.body.setFont(QFont(EDITOR_FONT_FAMILY, 10))
        self.body.setFrameShape(QFrame.Shape.NoFrame)
        self.body.setStyleSheet(
            f"QPlainTextEdit {{ background: {COLORS['bg']}; "
            f"color: {COLORS['fg']}; border: 1px solid {COLORS['border']}; "
            "border-radius: 4px; }")
        self._hl = GBHighlighter(self.body.document())
        lay.addWidget(self.body)

        footer = QHBoxLayout()
        self.hint = QLabel("Alt+F12 / Esc schliesst")
        self.hint.setStyleSheet(f"color: {COLORS['fg_muted']};")
        footer.addWidget(self.hint)
        footer.addStretch(1)
        self.go_btn = QPushButton("Gehe zu Definition (F12)")
        self.go_btn.setProperty("accent", True)
        self.go_btn.clicked.connect(self._emit_go)
        footer.addWidget(self.go_btn)
        lay.addLayout(footer)

        self.setMinimumWidth(420)
        self.setMaximumWidth(720)

    def _emit_go(self) -> None:
        self.go_to.emit(self._target_line)
        self.close()

    def keyPressEvent(self, e):  # noqa: N802
        if e.key() == Qt.Key.Key_Escape:
            self.close()
            return
        if e.key() in (Qt.Key.Key_Return, Qt.Key.Key_Enter):
            self._emit_go()
            return
        super().keyPressEvent(e)

    def show_at(self, global_point: QPoint, signature: str, doc: str,
                body: str, target_line: int) -> None:
        self._target_line = target_line
        self.sig_label.setText(signature)
        if doc.strip():
            self.doc_label.setText(doc)
            self.doc_label.show()
        else:
            self.doc_label.hide()
        self.body.setPlainText(body)
        # Hoehe an Zeilenzahl anpassen (bis _MAX_BODY_LINES).
        fm = self.body.fontMetrics()
        n = min(_MAX_BODY_LINES, max(1, body.count("\n") + 1))
        self.body.setFixedHeight(fm.lineSpacing() * n + 12)
        self.adjustSize()
        self.move(global_point)
        self.show()
        self.go_btn.setFocus()
