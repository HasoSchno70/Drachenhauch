"""Markdown-Viewer fuer README/Doku-Dateien.

Rendert Markdown ueber Qt's natives `setMarkdown()` (gute Formatierung:
Tabellen, Listen, Code-Bloecke, vernuenftige Abstaende) und faerbt das
Ergebnis danach **programmatisch** theme-treu ein -- `setDefaultStyleSheet()`
greift bei `setMarkdown` naemlich NICHT (ein fruehrerer HTML-Umweg ueber
`toHtml()` zog zwar Farben, buk aber gequetschte Inline-Margins ein). Statt
CSS setzen wir also Palette (Text/Link) plus pro Block/Fragment die
Akzent-/Code-Farben und etwas mehr Zeilen-/Absatz-Luft.

Zusaetzlich:
  * **Inhaltsverzeichnis** (links) aus den Headings -- Klick scrollt zur
    Stelle (ueber die Block-Nummer + Dokument-Layout).
  * **Suche** (Strg+F) mit Weiter/Zurueck, Wrap-Around und Highlight aller
    Treffer.

Klick auf interne Links (\".gb\"-Anker oder andere .md-Dateien) liefert
ein Signal an MainWindow, das den Editor entsprechend wechselt.
"""
from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import Qt, QUrl, Signal
from PySide6.QtGui import (
    QColor, QFont, QFontInfo, QKeySequence, QPalette, QShortcut,
    QTextBlockFormat, QTextCharFormat, QTextCursor, QTextDocument,
)
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QLineEdit, QListWidget, QListWidgetItem,
    QPushButton, QSplitter, QTextBrowser, QTextEdit, QVBoxLayout, QWidget,
)

from .theme import COLORS, EDITOR_FONT_FAMILY, theme_signals

_PROPORTIONAL = QTextBlockFormat.LineHeightTypes.ProportionalHeight.value


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

        # Header mit Titel + Inhalts-Toggle + Such-/Refresh-Button
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
        # Theme-Wechsel: neu rendern + neu einfaerben.
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
            self.browser.setMarkdown(text)
            self._style_document()
            self._populate_toc(self._collect_headings())
        else:
            self.browser.setPlainText(text)
            self._populate_toc([])
        self._update_highlights()

    # ----------------------------------------------------- Theming
    def _style_document(self) -> None:
        """Faerbt das gerenderte Markdown theme-treu ein und gibt den
        Absaetzen etwas mehr Luft.

        Qt's `setMarkdown` setzt keine Foreground-Farben (Default-Schwarz auf
        dunklem Grund). Wir setzen daher die Palette (Text/Link) und ziehen pro
        Block/Fragment die Akzent-Farbe (Headings), Code-Farbe + dezenten
        Hintergrund (Inline-Code/Code-Bloecke) ein. Zusaetzlich mehr
        Zeilenhoehe und Absatz-Abstand fuer bessere Lesbarkeit -- Code-Bloecke
        bleiben eng.
        """
        c = COLORS
        pal = self.browser.palette()
        pal.setColor(QPalette.ColorRole.Text, QColor(c["fg"]))
        pal.setColor(QPalette.ColorRole.Link, QColor(c["link"]))
        self.browser.setPalette(pal)

        doc = self.browser.document()
        code_fmt = QTextCharFormat()
        code_fmt.setForeground(QColor(c["builtin"]))
        code_fmt.setBackground(QColor(c["bg_alt"]))

        blk = doc.firstBlock()
        while blk.isValid():
            level = blk.blockFormat().headingLevel()
            empty = not blk.text().strip()
            is_code = (not level) and not empty and self._block_is_code(blk)

            # Block-Format: Zeilenhoehe + Abstand (kopiert vorhandenes Format,
            # damit Listen-Einrueckung / Blockquote erhalten bleibt).
            bf = QTextBlockFormat(blk.blockFormat())
            if level:
                bf.setLineHeight(124, _PROPORTIONAL)
                bf.setTopMargin(16.0)
                bf.setBottomMargin(6.0)
            elif is_code or empty:
                # Code-Zeile oder Blank-Line im Code-Block -> eng halten.
                bf.setLineHeight(118, _PROPORTIONAL)
                bf.setBottomMargin(0.0)
            else:
                bf.setLineHeight(134, _PROPORTIONAL)
                bf.setBottomMargin(10.0)
            bcur = QTextCursor(blk)
            bcur.setBlockFormat(bf)

            if level:
                col = c["accent"] if level <= 2 else c["accent_hover"]
                ccur = QTextCursor(doc)
                ccur.setPosition(blk.position())
                ccur.setPosition(blk.position() + max(0, blk.length() - 1),
                                 QTextCursor.MoveMode.KeepAnchor)
                f = QTextCharFormat()
                f.setForeground(QColor(col))
                ccur.mergeCharFormat(f)
            else:
                # Code-Fragmente (Inline + Bloecke) einfaerben + Hintergrund.
                it = blk.begin()
                while not it.atEnd():
                    frag = it.fragment()
                    if frag.isValid() and self._is_mono(frag.charFormat()):
                        fc = QTextCursor(doc)
                        fc.setPosition(frag.position())
                        fc.setPosition(frag.position() + frag.length(),
                                       QTextCursor.MoveMode.KeepAnchor)
                        fc.mergeCharFormat(code_fmt)
                    it += 1
            blk = blk.next()

    @staticmethod
    def _is_mono(char_fmt) -> bool:
        """True, wenn das Fragment Monospace ist. Qt markiert Code ueber die
        Font-Familie (`Courier New`), NICHT ueber `fontFixedPitch()` (das
        bleibt False) -- daher loesen wir die echte Eigenschaft via QFontInfo
        auf."""
        return QFontInfo(char_fmt.font()).fixedPitch()

    def _block_is_code(self, blk) -> bool:
        """True, wenn das erste Fragment des Blocks Monospace ist -- bei einem
        Fenced-Code-Block ist die ganze Zeile Monospace, bei Prosa mit
        Inline-Code beginnt der Block mit normalem Text."""
        it = blk.begin()
        if it.atEnd():
            return False
        frag = it.fragment()
        return frag.isValid() and self._is_mono(frag.charFormat())

    # ---------------------------------------------------- Inhaltsverzeichnis
    def _collect_headings(self) -> list[tuple[int, str, int]]:
        """(level, text, block_number) je Heading-Block."""
        toc: list[tuple[int, str, int]] = []
        blk = self.browser.document().firstBlock()
        while blk.isValid():
            level = blk.blockFormat().headingLevel()
            text = blk.text().strip()
            if level and text:
                toc.append((level, text, blk.blockNumber()))
            blk = blk.next()
        return toc

    def _populate_toc(self, toc: list[tuple[int, str, int]]) -> None:
        self.toc.clear()
        min_level = min((lv for lv, _, _ in toc), default=1)
        for level, text, block_no in toc:
            indent = "    " * max(0, level - min_level)
            it = QListWidgetItem(f"{indent}{text}")
            it.setData(Qt.ItemDataRole.UserRole, block_no)
            it.setToolTip(text)
            self.toc.addItem(it)
        self.toc.setVisible(bool(toc) and self._toc_btn.isChecked())
        self._toc_btn.setEnabled(bool(toc))

    def _on_toc_clicked(self, item: QListWidgetItem) -> None:
        block_no = item.data(Qt.ItemDataRole.UserRole)
        if block_no is None:
            return
        doc = self.browser.document()
        blk = doc.findBlockByNumber(int(block_no))
        if not blk.isValid():
            return
        y = doc.documentLayout().blockBoundingRect(blk).top()
        self.browser.verticalScrollBar().setValue(int(y) - 8)

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
