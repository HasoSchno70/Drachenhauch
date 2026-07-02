"""Welcome-Panel beim Editor-Start ohne offene Datei.

Zeigt das GameBasic-Logo, ein paar Action-Buttons (Neu/Oeffnen/Beispiele/
Doku), eine kuratierte **Showcase**-Galerie mit Screenshot-Karten (Klick
oeffnet die Demo) und eine Liste der zuletzt geoeffneten Dateien. Wird als
Tab eingefuegt, sobald der Editor mit leerem Workspace startet; verschwindet,
sobald der User eine Datei oeffnet oder einen neuen Tab anlegt.
"""
from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFont, QMouseEvent, QPixmap
from PySide6.QtWidgets import (
    QFrame, QGridLayout, QHBoxLayout, QLabel, QListWidget, QListWidgetItem,
    QPushButton, QScrollArea, QSizePolicy, QSpacerItem, QVBoxLayout, QWidget,
)

from .showcase import SHOWCASE, thumb_path
from .theme import COLORS, theme_signals

# Karten-Geometrie der Showcase-Galerie.
_CARD_W = 240
_THUMB_W = 220
_GRID_COLS = 3


class _ShowcaseCard(QFrame):
    """Klickbare Demo-Karte: Thumbnail + Titel + Kurzbeschreibung."""

    clicked = Signal(Path)

    def __init__(self, project_root: Path, entry: dict, parent=None):
        super().__init__(parent)
        self._path = (project_root / "examples" / entry["file"]).resolve()
        self.setObjectName("ShowcaseCard")
        self.setFixedWidth(_CARD_W)
        self.setCursor(Qt.CursorShape.PointingHandCursor)

        lay = QVBoxLayout(self)
        lay.setContentsMargins(8, 8, 8, 8)
        lay.setSpacing(6)

        self.thumb = QLabel()
        self.thumb.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.thumb.setFixedHeight(_THUMB_W * 9 // 16)
        self.thumb.setScaledContents(False)
        tp = thumb_path(project_root, entry)
        if tp.exists():
            pm = QPixmap(str(tp))
            if not pm.isNull():
                self.thumb.setPixmap(pm.scaledToWidth(
                    _THUMB_W, Qt.TransformationMode.SmoothTransformation))
        if self.thumb.pixmap() is None or self.thumb.pixmap().isNull():
            # Fallback ohne Thumbnail: Platzhalter mit Play-Glyph.
            self.thumb.setText("▶")
            self.thumb.setObjectName("ShowcaseThumbPlaceholder")
        lay.addWidget(self.thumb)

        title = QLabel(entry["title"])
        tf = title.font()
        tf.setBold(True)
        tf.setPointSize(10)
        title.setFont(tf)
        title.setWordWrap(True)
        title.setObjectName("ShowcaseTitle")
        lay.addWidget(title)

        desc = QLabel(entry["desc"])
        desc.setWordWrap(True)
        df = desc.font()
        df.setPointSize(8)
        desc.setFont(df)
        desc.setObjectName("ShowcaseDesc")
        lay.addWidget(desc)
        lay.addStretch(1)

    def mouseReleaseEvent(self, event: QMouseEvent):  # noqa: N802
        if event.button() == Qt.MouseButton.LeftButton and self.rect().contains(
                event.position().toPoint()):
            self.clicked.emit(self._path)
        super().mouseReleaseEvent(event)


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

        # Gesamtes Panel scrollbar machen -- die Showcase-Galerie braucht Platz.
        root = QVBoxLayout(self)
        root.setContentsMargins(0, 0, 0, 0)
        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setFrameShape(QFrame.Shape.NoFrame)
        root.addWidget(self.scroll)

        content = QWidget()
        self.scroll.setWidget(content)
        outer = QVBoxLayout(content)
        outer.setContentsMargins(60, 40, 60, 40)
        outer.setSpacing(18)

        # Logo prominent oben
        self.logo_label = QLabel()
        self.logo_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self._load_logo()
        outer.addWidget(self.logo_label)

        # Subtitle
        self.subtitle = QLabel(
            "BASIC fuer Spiele -- mit Pascal-strikter Typisierung und OOP.")
        self.subtitle.setAlignment(Qt.AlignmentFlag.AlignCenter)
        sub_font = self.subtitle.font()
        sub_font.setPointSize(11)
        self.subtitle.setFont(sub_font)
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

        # --- Showcase-Galerie -------------------------------------
        self.showcase_label = QLabel("Showcase")
        scf = self.showcase_label.font()
        scf.setBold(True)
        scf.setPointSize(11)
        self.showcase_label.setFont(scf)
        self.showcase_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        outer.addWidget(self.showcase_label)

        grid_host = QWidget()
        self.showcase_grid = QGridLayout(grid_host)
        self.showcase_grid.setSpacing(14)
        self.showcase_grid.setAlignment(Qt.AlignmentFlag.AlignHCenter)
        self._build_showcase()
        outer.addWidget(grid_host)

        # Recent-Liste
        self.recent_label = QLabel("Zuletzt geoeffnet")
        rfont = self.recent_label.font()
        rfont.setBold(True)
        rfont.setPointSize(11)
        self.recent_label.setFont(rfont)
        self.recent_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        outer.addWidget(self.recent_label)

        self.recent_list = QListWidget()
        self.recent_list.setMaximumHeight(180)
        self.recent_list.itemActivated.connect(self._on_recent_clicked)
        self.recent_list.itemClicked.connect(self._on_recent_clicked)
        self._populate_recent()
        outer.addWidget(self.recent_list)

        outer.addItem(QSpacerItem(
            0, 0, QSizePolicy.Policy.Minimum, QSizePolicy.Policy.Expanding))

        self._apply_style()
        theme_signals.changed.connect(self._on_theme_changed)

    # ---------------------------------------------------- Showcase
    def _build_showcase(self) -> None:
        for i, entry in enumerate(SHOWCASE):
            card = _ShowcaseCard(self.project_root, entry)
            card.clicked.connect(self.open_path)
            self.showcase_grid.addWidget(card, i // _GRID_COLS, i % _GRID_COLS)

    def _load_logo(self) -> None:
        try:
            from .. import branding
            from PIL.ImageQt import ImageQt
            scaled = branding.scaled_logo_image(480)
            if scaled is None:
                self.logo_label.setText("GameBasic")
                return
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
            WelcomePanel, QScrollArea {{ background-color: {c['bg']}; }}
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
            #ShowcaseCard {{
                background-color: {c['bg_alt']};
                border: 1px solid {c['border']};
                border-radius: 6px;
            }}
            #ShowcaseCard:hover {{
                border: 1px solid {c['accent']};
                background-color: {c['bg_hover']};
            }}
            #ShowcaseTitle {{ color: {c['fg']}; }}
            #ShowcaseDesc {{ color: {c['fg_muted']}; }}
            #ShowcaseThumbPlaceholder {{
                color: {c['accent']};
                background-color: {c['bg_panel']};
                border-radius: 4px;
                font-size: 32px;
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
