"""Welcome-Panel beim Editor-Start ohne offene Datei.

Zeigt das GameBasic-Logo, ein paar Action-Buttons (Neu/Oeffnen/
Beispiele) und eine Liste der zuletzt geoeffneten Dateien. Wird als
Tab eingefuegt, sobald der Editor mit leerem Workspace startet.
Sobald der User eine Datei oeffnet oder einen neuen Tab anlegt,
verschwindet das Welcome.
"""
from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFont, QPixmap
from PySide6.QtWidgets import (
    QHBoxLayout, QLabel, QListWidget, QListWidgetItem, QPushButton,
    QSpacerItem, QSizePolicy, QVBoxLayout, QWidget,
)

from .theme import COLORS, theme_signals


class WelcomePanel(QWidget):
    """Standalone-Welcome-Widget. Wird als Tab eingefuegt."""

    new_file = Signal()
    open_dialog = Signal()
    examples = Signal()
    show_docs = Signal()
    open_path = Signal(Path)

    def __init__(self, project_root: Path, recent_files: list[str], parent=None):
        super().__init__(parent)
        self.project_root = project_root
        self._recent = recent_files

        outer = QVBoxLayout(self)
        outer.setContentsMargins(80, 60, 80, 60)
        outer.setSpacing(20)

        # Logo prominent oben
        self.logo_label = QLabel()
        self.logo_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self._load_logo()
        outer.addWidget(self.logo_label)

        # Subtitle
        self.subtitle = QLabel("BASIC fuer Spiele -- mit Pascal-strikter Typisierung und OOP.")
        self.subtitle.setAlignment(Qt.AlignmentFlag.AlignCenter)
        sub_font = self.subtitle.font()
        sub_font.setPointSize(11)
        self.subtitle.setFont(sub_font)
        self.subtitle.setStyleSheet(f"color: {COLORS['fg_muted']};")
        outer.addWidget(self.subtitle)

        # Action-Buttons
        actions_row = QHBoxLayout()
        actions_row.setSpacing(12)
        actions_row.addStretch(1)
        self.btn_new = QPushButton("Neu")
        self.btn_new.setShortcut("Ctrl+N")
        self.btn_new.clicked.connect(self.new_file)
        self.btn_open = QPushButton("Datei oeffnen ...")
        self.btn_open.setShortcut("Ctrl+O")
        self.btn_open.clicked.connect(self.open_dialog)
        self.btn_examples = QPushButton("Beispiele")
        self.btn_examples.clicked.connect(self.examples)
        self.btn_docs = QPushButton("Doku")
        self.btn_docs.setShortcut("F1")
        self.btn_docs.clicked.connect(self.show_docs)
        for b in (self.btn_new, self.btn_open, self.btn_examples, self.btn_docs):
            b.setMinimumHeight(36)
            b.setMinimumWidth(120)
            actions_row.addWidget(b)
        actions_row.addStretch(1)
        outer.addLayout(actions_row)

        # Recent-Liste
        self.recent_label = QLabel("Zuletzt geoeffnet")
        rfont = self.recent_label.font()
        rfont.setBold(True)
        rfont.setPointSize(11)
        self.recent_label.setFont(rfont)
        self.recent_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        outer.addWidget(self.recent_label)

        self.recent_list = QListWidget()
        self.recent_list.setMaximumHeight(220)
        self.recent_list.itemActivated.connect(self._on_recent_clicked)
        self.recent_list.itemClicked.connect(self._on_recent_clicked)
        self._populate_recent()
        outer.addWidget(self.recent_list)

        outer.addItem(QSpacerItem(0, 0, QSizePolicy.Policy.Minimum, QSizePolicy.Policy.Expanding))

        self._apply_style()
        theme_signals.changed.connect(self._on_theme_changed)

    def _load_logo(self) -> None:
        try:
            from .. import branding
            from PIL import Image
            from PIL.ImageQt import ImageQt
            if not branding.is_available():
                self.logo_label.setText("GameBasic")
                return
            pil = Image.open(branding._logo_path()).convert("RGBA")
            target_w = 480
            ratio = target_w / pil.size[0]
            target_h = max(1, int(pil.size[1] * ratio))
            scaled = pil.resize((target_w, target_h), Image.LANCZOS)
            qimg = ImageQt(scaled).copy()
            self._logo_pixmap = QPixmap.fromImage(qimg)
            self.logo_label.setPixmap(self._logo_pixmap)
        except Exception:
            self.logo_label.setText("GameBasic")

    def _populate_recent(self) -> None:
        self.recent_list.clear()
        for entry in self._recent:
            p = Path(entry)
            if not p.exists():
                continue
            label = p.name
            try:
                rel = p.relative_to(self.project_root)
                label = str(rel).replace("\\", "/")
            except ValueError:
                label = str(p)
            it = QListWidgetItem(label)
            it.setData(Qt.ItemDataRole.UserRole, p)
            self.recent_list.addItem(it)
        if self.recent_list.count() == 0:
            placeholder = QListWidgetItem("(keine Eintraege)")
            placeholder.setForeground(self.palette().color(self.foregroundRole()))
            placeholder.setFlags(Qt.ItemFlag.NoItemFlags)
            self.recent_list.addItem(placeholder)

    def _on_recent_clicked(self, item: QListWidgetItem) -> None:
        p = item.data(Qt.ItemDataRole.UserRole)
        if isinstance(p, Path):
            self.open_path.emit(p)

    def _apply_style(self) -> None:
        c = COLORS
        self.setStyleSheet(
            f"""
            WelcomePanel {{ background-color: {c['bg']}; }}
            QListWidget {{
                background-color: {c['bg_alt']};
                color: {c['fg']};
                border: 1px solid {c['border']};
                border-radius: 4px;
            }}
            QListWidget::item:hover {{ background-color: {c['bg_hover']}; }}
            QListWidget::item:selected {{ background-color: {c['sel']}; }}
            QPushButton {{
                background-color: {c['bg_panel']};
                color: {c['fg']};
                border: 1px solid {c['border']};
                border-radius: 4px;
                padding: 8px 16px;
            }}
            QPushButton:hover {{
                background-color: {c['bg_hover']};
                border-color: {c['accent']};
            }}
            """
        )
        self.subtitle.setStyleSheet(f"color: {c['fg_muted']};")

    def _on_theme_changed(self, _name: str) -> None:
        self._apply_style()
        # Recent-Liste re-populieren, damit gecachte Vordergrundfarben
        # (z.B. der Placeholder bei leerer Liste) frische Theme-Werte
        # bekommen.
        self._populate_recent()

    def update_recent(self, recent: list[str]) -> None:
        self._recent = recent
        self._populate_recent()
