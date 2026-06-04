"""Markdown-Viewer fuer README/Doku-Dateien.

Rendert Markdown ueber `setHtml()` (siehe `_markdown_to_html` -- Qt's
`setMarkdown()` ignoriert unser Theme-Stylesheet). Zusaetzlich:
  * **Inhaltsverzeichnis** (links) aus den Headings -- Klick springt zur
    Stelle (via injizierte `<a name="tocN">`-Anker).
  * **Suche** (Strg+F) mit Weiter/Zurueck, Wrap-Around und Highlight aller
    Treffer.

Klick auf interne Links (\".gb\"-Anker oder andere .md-Dateien) liefert
ein Signal an MainWindow, das den Editor entsprechend wechselt.
"""
from __future__ import annotations

import re
from html import unescape
from pathlib import Path

from PySide6.QtCore import Qt, QUrl, Signal
from PySide6.QtGui import (
    QColor, QFont, QKeySequence, QShortcut, QTextCharFormat, QTextCursor,
    QTextDocument,
)
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QLineEdit, QListWidget, QListWidgetItem,
    QPushButton, QSplitter, QTextBrowser, QTextEdit, QVBoxLayout, QWidget,
)

from .theme import COLORS, EDITOR_FONT_FAMILY, theme_signals

# Heading-Tags im generierten HTML -- fuer Inhaltsverzeichnis + Anker-Injektion.
_HEADING_RE = re.compile(r"(<h([1-6])\b[^>]*>)(.*?)(</h\2>)", re.DOTALL)
_TAG_RE = re.compile(r"<[^>]+>")


class MarkdownViewer(QWidget):
    """Standalone-Toolfenster mit Markdown-Inhalt."""

    open_gb_file = Signal(Path)

    def __init__(self, project_root: Path, parent: QWidget | None = None):
        # Tool-Window: bleibt ueber MainWindow, eigener Eintrag in Taskbar.
        super().__init__(parent, Qt.WindowType.Window)
        self.project_root = project_root
        self._current_path: Path | None = None

        self.setWindowTitle("Doku")
        self.resize(960, 720)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Header mit Titel + Inhalts-Toggle + Refresh-Button
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
        self._toc_btn = QPushButton("☰ Inhalt")
        self._toc_btn.setCheckable(True)
        self._toc_btn.setChecked(True)
        self._toc_btn.setToolTip("Inhaltsverzeichnis ein-/ausblenden")
        self._toc_btn.toggled.connect(self._toggle_toc)
        hl.addWidget(self._toc_btn)
        self._find_btn = QPushButton("Suchen")
        self._find_btn.setToolTip("Im Dokument suchen (Strg+F)")
        self._find_btn.clicked.connect(self._toggle_find)
        hl.addWidget(self._find_btn)
        self._refresh_btn = QPushButton("Neu laden")
        self._refresh_btn.clicked.connect(self._reload)
        hl.addWidget(self._refresh_btn)
        layout.addWidget(self._header)

        # Suchen-Leiste (default versteckt)
        self._find_bar = QFrame()
        fl = QHBoxLayout(self._find_bar)
        fl.setContentsMargins(12, 4, 12, 4)
        fl.addWidget(QLabel("Suchen:"))
        self.find_entry = QLineEdit()
        self.find_entry.setPlaceholderText("im Dokument suchen ...")
        self.find_entry.textChanged.connect(self._update_highlights)
        self.find_entry.returnPressed.connect(lambda: self._find(forward=True))
        fl.addWidget(self.find_entry, 1)
        self._find_count = QLabel("")
        fl.addWidget(self._find_count)
        prev_btn = QPushButton("Zurueck")
        prev_btn.clicked.connect(lambda: self._find(forward=False))
        fl.addWidget(prev_btn)
        next_btn = QPushButton("Weiter")
        next_btn.clicked.connect(lambda: self._find(forward=True))
        fl.addWidget(next_btn)
        close_btn = QPushButton("X")
        close_btn.setFixedWidth(26)
        close_btn.clicked.connect(self._hide_find)
        fl.addWidget(close_btn)
        self._find_bar.setVisible(False)
        layout.addWidget(self._find_bar)

        # Splitter: Inhaltsverzeichnis | Browser
        self._splitter = QSplitter(Qt.Orientation.Horizontal)
        self.toc = QListWidget()
        self.toc.setMaximumWidth(320)
        self.toc.itemClicked.connect(self._on_toc_clicked)
        self.toc.itemActivated.connect(self._on_toc_clicked)
        self._splitter.addWidget(self.toc)

        self.browser = QTextBrowser()
        self.browser.setOpenLinks(False)        # Wir handlen Links selbst.
        self.browser.setOpenExternalLinks(False)
        self.browser.anchorClicked.connect(self._on_anchor)
        self.browser.setFont(QFont("Segoe UI", 11))
        self._splitter.addWidget(self.browser)
        self._splitter.setStretchFactor(0, 0)
        self._splitter.setStretchFactor(1, 1)
        self._splitter.setSizes([220, 740])
        layout.addWidget(self._splitter, 1)

        # Strg+F oeffnet die Suche, Esc schliesst sie.
        sc_find = QShortcut(QKeySequence(QKeySequence.StandardKey.Find), self)
        sc_find.activated.connect(self._toggle_find)
        sc_esc = QShortcut(QKeySequence("Esc"), self.find_entry)
        sc_esc.activated.connect(self._hide_find)

        self._apply_style()
        theme_signals.changed.connect(self._on_theme_changed)

    def _apply_style(self) -> None:
        c = COLORS
        self._header.setStyleSheet(f"background-color: {c['bg_panel']};")
        self._find_bar.setStyleSheet(f"background-color: {c['bg_alt']};")
        self.toc.setStyleSheet(
            f"""
            QListWidget {{
                background-color: {c['bg_alt']};
                color: {c['fg']};
                border: 0;
                border-right: 1px solid {c['border']};
                outline: 0;
                padding: 6px 2px;
            }}
            QListWidget::item {{ padding: 3px 6px; border-radius: 4px; }}
            QListWidget::item:hover {{ background-color: {c['bg_hover']}; }}
            QListWidget::item:selected {{
                background-color: {c['sel']}; color: {c['fg']};
            }}
            """
        )
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
            self._populate_toc([])
            return
        # `searchPaths` erlaubt relative Bilder/Links aufzuloesen.
        self.browser.setSearchPaths([str(path.parent)])
        if path.suffix.lower() in (".md", ".markdown"):
            html = self._markdown_to_html(text)
            html, toc = self._inject_toc(html)
            self.browser.setHtml(html)
            self._populate_toc(toc)
        else:
            self.browser.setPlainText(text)
            self._populate_toc([])
        self._update_highlights()

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

    def _inject_toc(self, html: str) -> tuple[str, list[tuple[int, str]]]:
        """Sammelt Headings fuers Inhaltsverzeichnis und injiziert pro Heading
        einen `<a name="tocN">`-Anker, damit das TOC dorthin springen kann.

        Index-Alignment ist garantiert, weil TOC-Eintraege UND Anker aus
        derselben sequentiellen Heading-Iteration stammen.
        """
        toc: list[tuple[int, str]] = []

        def repl(m: re.Match) -> str:
            open_tag, level, inner, close_tag = (
                m.group(1), int(m.group(2)), m.group(3), m.group(4))
            text = unescape(_TAG_RE.sub("", inner)).strip()
            if not text:
                return m.group(0)
            idx = len(toc)
            toc.append((level, text))
            return f'{open_tag}<a name="toc{idx}"></a>{inner}{close_tag}'

        return _HEADING_RE.sub(repl, html), toc

    def _populate_toc(self, toc: list[tuple[int, str]]) -> None:
        self.toc.clear()
        min_level = min((lv for lv, _ in toc), default=1)
        for idx, (level, text) in enumerate(toc):
            indent = "    " * max(0, level - min_level)
            it = QListWidgetItem(f"{indent}{text}")
            it.setData(Qt.ItemDataRole.UserRole, idx)
            it.setToolTip(text)
            self.toc.addItem(it)
        # TOC nur zeigen, wenn es Eintraege gibt UND der User es nicht
        # ausgeblendet hat.
        self.toc.setVisible(bool(toc) and self._toc_btn.isChecked())
        self._toc_btn.setEnabled(bool(toc))

    def _on_toc_clicked(self, item: QListWidgetItem) -> None:
        idx = item.data(Qt.ItemDataRole.UserRole)
        if idx is not None:
            self.browser.scrollToAnchor(f"toc{idx}")

    def _toggle_toc(self, on: bool) -> None:
        self.toc.setVisible(on and self.toc.count() > 0)

    # ----------------------------------------------------- Suche
    def _toggle_find(self) -> None:
        if self._find_bar.isVisible():
            self._hide_find()
        else:
            self._find_bar.setVisible(True)
            self.find_entry.setFocus()
            self.find_entry.selectAll()
            self._update_highlights()

    def _hide_find(self) -> None:
        self._find_bar.setVisible(False)
        self.browser.setExtraSelections([])
        self._find_count.setText("")

    def _find(self, *, forward: bool = True) -> None:
        q = self.find_entry.text()
        if not q:
            return
        flags = QTextDocument.FindFlag(0)
        if not forward:
            flags |= QTextDocument.FindFlag.FindBackward
        if not self.browser.find(q, flags):
            # Wrap-around vom Anfang/Ende.
            cur = self.browser.textCursor()
            cur.movePosition(
                QTextCursor.MoveOperation.End if not forward
                else QTextCursor.MoveOperation.Start)
            self.browser.setTextCursor(cur)
            self.browser.find(q, flags)

    def _update_highlights(self) -> None:
        """Hebt alle Treffer der Suchanfrage hervor (ExtraSelections)."""
        if not self._find_bar.isVisible():
            self.browser.setExtraSelections([])
            return
        q = self.find_entry.text()
        extra: list[QTextEdit.ExtraSelection] = []
        if q:
            doc = self.browser.document()
            fmt = QTextCharFormat()
            fmt.setBackground(QColor(COLORS["find_hit"]))
            cur = QTextCursor(doc)
            while True:
                cur = doc.find(q, cur)
                if cur.isNull():
                    break
                sel = QTextEdit.ExtraSelection()
                sel.cursor = cur
                sel.format = fmt
                extra.append(sel)
        self.browser.setExtraSelections(extra)
        self._find_count.setText(f"{len(extra)} Treffer" if q else "")

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
