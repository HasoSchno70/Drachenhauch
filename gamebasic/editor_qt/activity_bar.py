"""ActivityBar links: schmale Spalte mit Icons (a la VSCode).

Drei Eintraege: Files / Outline / Built-ins. Klick schaltet die
Sidebar zwischen den Panels um. Klick auf den aktuell aktiven
Eintrag blendet die Sidebar aus (Toggle).
"""
from __future__ import annotations

from PySide6.QtCore import QSize, Qt, Signal
from PySide6.QtWidgets import (
    QButtonGroup, QFrame, QToolButton, QVBoxLayout, QWidget,
)

from .icons import icons
from .theme import COLORS, theme_signals


# (key, icon_key, tooltip)
ENTRIES = [
    ("files",    "files",    "Dateien (Strg+Shift+E)"),
    ("outline",  "outline",  "Outline (Strg+Shift+O)"),
    ("builtins", "builtins", "Built-ins (Strg+Shift+B)"),
]


class ActivityBar(QWidget):
    selected = Signal(str)        # key der gewaehlten Ansicht
    sidebar_toggled = Signal(bool)  # True = sichtbar, False = versteckt

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        self.setObjectName("ActivityBar")
        self.setFixedWidth(56)
        self._buttons: dict[str, QToolButton] = {}
        self._active_key = "files"
        self._sidebar_visible = True

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 6, 0, 6)
        layout.setSpacing(2)

        self._group = QButtonGroup(self)
        self._group.setExclusive(False)

        for key, icon_key, tip in ENTRIES:
            btn = QToolButton()
            btn.setIcon(icons.get(icon_key))
            btn.setIconSize(QSize(22, 22))
            btn.setToolTip(tip)
            btn.setCheckable(True)
            btn.setFocusPolicy(Qt.FocusPolicy.NoFocus)
            btn.setMinimumHeight(44)
            btn.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonIconOnly)
            btn.clicked.connect(lambda _checked=False, k=key: self._on_clicked(k))
            self._group.addButton(btn)
            self._buttons[key] = btn
            layout.addWidget(btn)
        # Mapping key -> icon_key fuer Theme-Refresh.
        self._icon_keys = {key: ik for key, ik, _ in ENTRIES}

        layout.addStretch(1)

        self._buttons["files"].setChecked(True)
        self._apply_style()
        theme_signals.changed.connect(self._on_theme_changed)

    def set_active(self, key: str) -> None:
        if key not in self._buttons:
            return
        for k, btn in self._buttons.items():
            btn.setChecked(k == key)
        self._active_key = key
        self._sidebar_visible = True

    def _on_clicked(self, key: str) -> None:
        if key == self._active_key and self._sidebar_visible:
            # Toggle weg.
            self._sidebar_visible = False
            for btn in self._buttons.values():
                btn.setChecked(False)
            self.sidebar_toggled.emit(False)
            return
        for k, btn in self._buttons.items():
            btn.setChecked(k == key)
        self._active_key = key
        if not self._sidebar_visible:
            self._sidebar_visible = True
            self.sidebar_toggled.emit(True)
        self.selected.emit(key)

    def _apply_style(self) -> None:
        c = COLORS
        self.setStyleSheet(
            f"""
            #ActivityBar {{
                background: qlineargradient(x1:0, y1:0, x2:1, y2:0,
                    stop:0 {c['bg_panel']}, stop:1 {c['bg_alt']});
                border-right: 1px solid {c['border']};
            }}
            QToolButton {{
                background-color: transparent;
                color: {c['fg_muted']};
                border: 0;
                border-left: 3px solid transparent;
                border-radius: 0;
                font-weight: bold;
                font-size: 9pt;
            }}
            QToolButton:hover {{
                color: {c['fg']};
                background-color: {c['bg_hover']};
            }}
            QToolButton:checked {{
                color: {c['accent']};
                background-color: {c['bg']};
                border-left: 3px solid {c['accent']};
            }}
            """
        )

    def _on_theme_changed(self, _name: str) -> None:
        self._apply_style()
        # Icon-Cache wurde im IconProvider gecleart -- frische Pixmaps
        # liefern den Buttons.
        for key, btn in self._buttons.items():
            btn.setIcon(icons.get(self._icon_keys[key]))
