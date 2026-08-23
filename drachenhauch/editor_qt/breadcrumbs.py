"""Breadcrumb-Leiste: zeigt den Scope-Pfad der Cursor-Position.

Eine schmale Leiste ueber dem Editor, z.B. ``Player  ›  Init`` wenn der
Cursor in der Methode ``Init`` der Klasse ``Player`` steht. Jedes Segment
ist anklickbar und springt zur Definitionszeile.

Gespeist aus `editor_qt.symbols.scope_path` -- die Leiste selbst haelt
keinen Parser, nur die Darstellung.
"""
from __future__ import annotations

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import (
    QHBoxLayout, QLabel, QSizePolicy, QToolButton, QWidget,
)

from .theme import COLORS, theme_signals

# Farbe pro Scope-Art (greift auf die Theme-Palette zurueck).
_KIND_COLOR = {
    "class": "type", "struct": "type",
    "function": "builtin", "sub": "builtin",
    "property": "kw_decl",
}
_KIND_GLYPH = {
    "class": "C", "struct": "S", "function": "ƒ", "sub": "▸",
    "property": "◆",
}


class Breadcrumbs(QWidget):
    """Klickbarer Scope-Pfad. `jump_requested(line)` bei Segment-Klick."""

    jump_requested = Signal(int)   # 1-basierte Zeile

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        self.setSizePolicy(QSizePolicy.Policy.Expanding,
                           QSizePolicy.Policy.Fixed)
        self._lay = QHBoxLayout(self)
        self._lay.setContentsMargins(8, 2, 8, 2)
        self._lay.setSpacing(4)
        self._file_name = ""
        self._scopes: list = []
        self._apply_style()
        # Bound-Method statt Lambda: eine Lambda-Closure haelt zwar `self` am
        # Leben, traegt aber KEINE Qt-Receiver-Info -- stirbt das C++-Objekt
        # (Parent-Fenster geschlossen/GC'd), disconnected Qt die Verbindung
        # nicht automatisch (das klappt nur bei Bound-Methods). Ein spaeteres
        # `theme_signals.changed.emit(...)` (z.B. beim naechsten
        # `MainWindow.__init__`) ruft dann auf ein geloeschtes QObject --
        # Use-after-free (siehe error_check.py fuer denselben Bugtyp).
        theme_signals.changed.connect(self._on_theme_changed)
        self._rebuild()

    def _on_theme_changed(self, _name=None) -> None:
        self._apply_style()
        self._rebuild()

    def _apply_style(self) -> None:
        self.setStyleSheet(
            f"background-color: {COLORS['bg_panel']}; "
            f"border-bottom: 1px solid {COLORS['border']};")

    # ------------------------------------------------------ API
    def set_path(self, file_name: str, scopes: list) -> None:
        """`scopes` ist eine Liste von symbols.Scope (aussen -> innen)."""
        # Nur neu bauen, wenn sich wirklich etwas geaendert hat (Cursor-Move
        # feuert oft, Scope wechselt selten).
        sig = (file_name, tuple((s.kind, s.name, s.line) for s in scopes))
        if sig == getattr(self, "_sig", None):
            return
        self._sig = sig
        self._file_name = file_name
        self._scopes = scopes
        self._rebuild()

    def clear(self) -> None:
        self.set_path("", [])

    # ------------------------------------------------------ intern
    def _clear_layout(self) -> None:
        while self._lay.count():
            item = self._lay.takeAt(0)
            if item is None:            # laut Qt moeglich, hier nie: count() > 0
                continue
            w = item.widget()
            if w is not None:
                w.deleteLater()

    def _sep(self) -> QLabel:
        lab = QLabel("›")
        lab.setStyleSheet(f"color: {COLORS['fg_muted']};")
        return lab

    def _segment(self, text: str, color_key: str, line: int | None,
                 glyph: str = "") -> QWidget:
        btn = QToolButton()
        btn.setAutoRaise(True)
        btn.setCursor(Qt.CursorShape.PointingHandCursor if line
                      else Qt.CursorShape.ArrowCursor)
        label = f"{glyph} {text}".strip() if glyph else text
        btn.setText(label)
        col = COLORS.get(color_key, COLORS["fg"])
        btn.setStyleSheet(
            "QToolButton { border: none; padding: 1px 4px; "
            f"color: {col}; background: transparent; }} "
            "QToolButton:hover { "
            f"background: {COLORS['bg_hover']}; border-radius: 3px; }}")
        if line:
            btn.clicked.connect(lambda _=False, ln=line: self.jump_requested.emit(ln))
        else:
            btn.setEnabled(False)
        return btn

    def _rebuild(self) -> None:
        self._clear_layout()
        if self._file_name:
            self._lay.addWidget(
                self._segment(self._file_name, "fg_muted", 1))
        if not self._scopes:
            if not self._file_name:
                ph = QLabel("")
                self._lay.addWidget(ph)
        for sc in self._scopes:
            self._lay.addWidget(self._sep())
            self._lay.addWidget(self._segment(
                sc.name, _KIND_COLOR.get(sc.kind, "fg"), sc.line,
                _KIND_GLYPH.get(sc.kind, "")))
        self._lay.addStretch(1)
