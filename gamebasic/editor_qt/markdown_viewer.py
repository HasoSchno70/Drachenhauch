"""Markdown-Viewer fuer README/Doku-Dateien.

Nutzt `QTextBrowser.setMarkdown()` (Qt 5.14+) -- Qt rendert das selbst,
inklusive Headings, Listen, Tabellen, Inline-Code und Fenced-Code-Bloecken
mit eigener Schriftart.

Klick auf interne Links (\".gb\"-Anker oder andere .md-Dateien) liefert
ein Signal an MainWindow, das den Editor entsprechend wechselt.
"""
from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import Qt, QUrl, Signal
from PySide6.QtGui import QFont, QTextDocument
from PySide6.QtWidgets import (
    QHBoxLayout, QLabel, QPushButton, QTextBrowser, QVBoxLayout, QWidget,
)

from .theme import COLORS, EDITOR_FONT_FAMILY, theme_signals


class MarkdownViewer(QWidget):
    """Standalone-Toolfenster mit Markdown-Inhalt."""

    open_gb_file = Signal(Path)

    def __init__(self, project_root: Path, parent: QWidget | None = None):
        # Tool-Window: bleibt ueber MainWindow, eigener Eintrag in Taskbar.
        super().__init__(parent, Qt.WindowType.Window)
        self.project_root = project_root
        self._current_path: Path | None = None

        self.setWindowTitle("Doku")
        self.resize(900, 700)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Header mit Titel + Refresh-Button
        self._header = QWidget()
        hl = QHBoxLayout(self._header)
        hl.setContentsMargins(12, 8, 12, 8)
        self._title = QLabel("Doku")
        title_font = QFont()
        title_font.setBold(True)
        title_font.setPointSize(11)
        self._title.setFont(title_font)
        hl.addWidget(self._title)
        hl.addStretch(1)
        self._refresh_btn = QPushButton("Neu laden")
        self._refresh_btn.clicked.connect(self._reload)
        hl.addWidget(self._refresh_btn)
        layout.addWidget(self._header)

        # Browser
        self.browser = QTextBrowser()
        self.browser.setOpenLinks(False)        # Wir handlen Links selbst.
        self.browser.setOpenExternalLinks(False)
        self.browser.anchorClicked.connect(self._on_anchor)
        # Fonts -- Code-Blocks bekommen die Editor-Font, fuer alles andere
        # bleibt die UI-Font.
        self.browser.setFont(QFont("Segoe UI", 11))
        layout.addWidget(self.browser, 1)

        self._apply_style()
        theme_signals.changed.connect(self._on_theme_changed)

    def _apply_style(self) -> None:
        c = COLORS
        self._header.setStyleSheet(f"background-color: {c['bg_panel']};")
        self.browser.setStyleSheet(
            f"""
            QTextBrowser {{
                background-color: {c['bg']};
                color: {c['fg']};
                border: 0;
                padding: 16px;
            }}
            """
        )
        # Markdown-spezifisches Styling. Qt's QTextBrowser interpretiert
        # nur ein begrenztes CSS-Subset, aber Foreground/Background +
        # font-family + padding ziehen.
        self.browser.document().setDefaultStyleSheet(
            f"""
            body {{ color: {c['fg']}; }}
            h1 {{ color: {c['accent']}; padding-bottom: 4px; }}
            h2 {{ color: {c['accent']}; padding-top: 8px; }}
            h3, h4 {{ color: {c['accent_hover']}; }}
            code {{
                font-family: "{EDITOR_FONT_FAMILY}", monospace;
                color: {c['builtin']};
                background-color: {c['bg_alt']};
            }}
            pre {{
                font-family: "{EDITOR_FONT_FAMILY}", monospace;
                color: {c['fg']};
                background-color: {c['bg_alt']};
                padding: 8px;
            }}
            pre code {{
                color: {c['fg']};
                background-color: transparent;
            }}
            a {{ color: {c['link']}; }}
            blockquote {{
                color: {c['fg_muted']};
                border-left: 3px solid {c['border']};
                padding-left: 8px;
            }}
            table {{ border-collapse: collapse; }}
            th, td {{
                border: 1px solid {c['border']};
                padding: 4px 8px;
            }}
            th {{ background-color: {c['bg_panel']}; color: {c['fg']}; }}
            hr {{ border: 1px solid {c['border']}; }}
            """
        )
        # Neu rendern, damit das aktualisierte Stylesheet wirkt.
        if self._current_path is not None:
            self._render(self._current_path)

    def _on_theme_changed(self, _name: str) -> None:
        self._apply_style()

    # -------------------------------------------------------- API
    def show_file(self, path: Path) -> None:
        path = Path(path).resolve()
        if not path.exists():
            self.browser.setPlainText(f"Datei nicht gefunden: {path}")
            self.show()
            self.raise_()
            return
        self._current_path = path
        self._render(path)
        self.setWindowTitle(f"{path.name} -- Doku")
        self._title.setText(path.name)
        self.show()
        self.raise_()
        self.activateWindow()

    def _render(self, path: Path) -> None:
        try:
            text = path.read_text(encoding="utf-8")
        except OSError as exc:
            self.browser.setPlainText(f"Lese-Fehler: {exc}")
            return
        # `searchPaths` erlaubt relative Bilder/Links aufzuloesen.
        self.browser.setSearchPaths([str(path.parent)])
        if path.suffix.lower() in (".md", ".markdown"):
            self.browser.setHtml(self._markdown_to_html(text))
        else:
            self.browser.setPlainText(text)

    def _markdown_to_html(self, text: str) -> str:
        """Markdown -> HTML, damit unser Theme-Stylesheet greift.

        Qt's `QTextBrowser.setMarkdown()` ignoriert das via
        `setDefaultStyleSheet()` gesetzte CSS komplett -- die Texte bekommen
        keinerlei Foreground-Farbe und rendern im dunklen Default (auf dem
        dunklen Editor-Hintergrund praktisch unlesbar). `setHtml()` dagegen
        konsultiert das Default-Stylesheet und backt unsere Theme-Farben in
        jedes Element. Also wandeln wir Markdown ueber ein Wegwerf-Dokument
        zu HTML und rendern dieses HTML.
        """
        conv = QTextDocument()
        conv.setDefaultFont(self.browser.font())   # Schriftgroesse beibehalten
        conv.setMarkdown(text)
        html = conv.toHtml()
        # Qt backt fuer Links hart das Default-Blau `#0000ff` als Inline-Style
        # ein -- das ueberstimmt unsere CSS-`a`-Regel und ist auf dunklem
        # Grund kaum lesbar. Es ist die EINZIGE von conv eingebackte Farbe,
        # daher koennen wir sie gefahrlos gegen die Theme-Link-Farbe tauschen.
        html = html.replace("color:#0000ff", f"color:{COLORS['link']}")
        return html

    def _reload(self) -> None:
        if self._current_path is not None:
            self._render(self._current_path)

    def _on_anchor(self, url: QUrl) -> None:
        # Externe URLs (http*) -> Default-Browser oeffnen.
        scheme = url.scheme().lower()
        if scheme in ("http", "https", "mailto"):
            from PySide6.QtGui import QDesktopServices
            QDesktopServices.openUrl(url)
            return

        target = url.toLocalFile() or url.toString()
        if not target:
            return
        cand = Path(target)
        if not cand.is_absolute() and self._current_path is not None:
            cand = (self._current_path.parent / target).resolve()
        if cand.suffix.lower() == ".gb" and cand.exists():
            self.open_gb_file.emit(cand)
            return
        if cand.exists() and cand.suffix.lower() in (".md", ".markdown"):
            self.show_file(cand)
            return
        # Anker innerhalb derselben Datei (#section) -- Browser scrollt selbst.
        if url.fragment():
            self.browser.scrollToAnchor(url.fragment())
