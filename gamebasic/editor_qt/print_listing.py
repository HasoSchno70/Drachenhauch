"""Listing-Druck fuer den Editor.

Erzeugt aus GameBasic-Quelltext eine druckfertige HTML-Darstellung -- wahlweise
farbig (Syntax-Highlighting) oder schwarz-weiss -- und schickt sie ueber eine
Druckvorschau an den Drucker. Gedruckt wird das aktive Listing (oder, wenn etwas
markiert ist, nur die Markierung).

Bewusst eine eigene, papier-freundliche Farbpalette (dunkel auf weiss) statt der
Editor-Theme-Farben: ein dunkles Editor-Theme wuerde sonst hell-auf-weiss und
damit unleserlich drucken. Die Token-Klassifikation kommt aus
:func:`highlighter.classify_token` -- identisch zu dem, was im Editor gefaerbt
wird, nur mit anderen Farben.
"""
from __future__ import annotations

from html import escape

from PySide6.QtGui import QFont, QTextDocument
from PySide6.QtWidgets import (
    QCheckBox, QDialog, QDialogButtonBox, QLabel, QRadioButton, QVBoxLayout,
)
from PySide6.QtPrintSupport import QPrinter, QPrintPreviewDialog

from ..lexer import Lexer
from ..errors import LexerError
from .highlighter import GBHighlighter, classify_token


# Papier-freundliche Farben (dunkel auf weiss), unabhaengig vom Editor-Theme.
_PRINT_COLORS = {
    "ctrl":    "#0000C0",
    "decl":    "#7F0055",
    "type":    "#1F7A8C",
    "string":  "#067D17",
    "number":  "#A05A00",
    "comment": "#808080",
    "builtin": "#0057B7",
    "ident":   "#1A1A1A",
    "bool":    "#A31515",
}
_BOLD_KEYS = {"ctrl", "decl", "bool"}
_DEFAULT_COLOR = "#1A1A1A"


def _line_spans(line: str) -> list[tuple[int, int, str | None]]:
    """Zerlegt eine Zeile in (start, length, klasse)-Spans.

    Spiegelt die Logik von :meth:`GBHighlighter.highlightBlock`: Kommentar
    abtrennen, den Rest lexen, f-Strings als String uebermalen. Liefert eine
    luecken-/ueberlappungsfreie Span-Liste in Reihenfolge.
    """
    n = len(line)
    if n == 0:
        return []
    keys: list[str | None] = [None] * n

    comment_start = GBHighlighter._find_comment_start(line)
    lex_target = line if comment_start < 0 else line[:comment_start]

    try:
        tokens = Lexer(lex_target).tokenize()
    except LexerError:
        tokens = []
    for tok in tokens:
        start, length = GBHighlighter._token_span(line, tok)
        if length <= 0:
            continue
        key = classify_token(tok)
        if key is None:
            continue
        for i in range(start, min(start + length, n)):
            keys[i] = key

    for fstart, flen in GBHighlighter._find_fstring_ranges(lex_target):
        for i in range(fstart, min(fstart + flen, n)):
            keys[i] = "string"

    if comment_start >= 0:
        for i in range(comment_start, n):
            keys[i] = "comment"

    spans: list[tuple[int, int, str | None]] = []
    i = 0
    while i < n:
        k = keys[i]
        j = i + 1
        while j < n and keys[j] == k:
            j += 1
        spans.append((i, j - i, k))
        i = j
    return spans


def build_listing_html(text: str, *, color: bool, line_numbers: bool,
                       title: str, font_pt: int = 10) -> str:
    """Baut die druckfertige HTML-Darstellung eines Listings."""
    # Der Editor liefert \n-getrennte Zeilen; Selektionen koennen
    # (Qt-Paragraph-Separator) enthalten.
    normalized = (text.replace(" ", "\n").replace(" ", "\n")
                  .replace("\r\n", "\n").replace("\r", "\n"))
    lines = normalized.split("\n")
    width = len(str(len(lines))) if lines else 1

    out: list[str] = []
    for idx, line in enumerate(lines, start=1):
        parts: list[str] = []
        if line_numbers:
            num = str(idx).rjust(width)
            parts.append(f'<span style="color:#A0A0A0">{escape(num)}  </span>')
        for start, length, key in _line_spans(line):
            chunk = escape(line[start:start + length])
            if key is None:
                parts.append(chunk)
                continue
            styles = []
            if color:
                styles.append(f"color:{_PRINT_COLORS[key]}")
            if key in _BOLD_KEYS:
                styles.append("font-weight:bold")
            if key == "comment":
                styles.append("font-style:italic")
            if styles:
                parts.append(f'<span style="{";".join(styles)}">{chunk}</span>')
            else:
                parts.append(chunk)
        out.append("".join(parts))

    body = "\n".join(out)
    header = (
        f'<div style="font-family:Arial,sans-serif; font-size:11pt; '
        f'font-weight:bold; color:#000; border-bottom:1px solid #888; '
        f'padding-bottom:3px; margin-bottom:8px;">{escape(title)}</div>'
    )
    return (
        f'<html><body>{header}'
        f'<pre style="font-family:Consolas,\'Courier New\',monospace; '
        f'font-size:{font_pt}pt; color:{_DEFAULT_COLOR}; '
        f'white-space:pre-wrap; line-height:122%; margin:0;">{body}</pre>'
        f'</body></html>'
    )


class PrintOptionsDialog(QDialog):
    """Kleiner Dialog: Farbe/SW, Zeilennummern, (Umfang bei Selektion)."""

    def __init__(self, parent, *, has_selection: bool):
        super().__init__(parent)
        self.setWindowTitle("Listing drucken")
        lay = QVBoxLayout(self)

        lay.addWidget(QLabel("Darstellung:"))
        self.rb_color = QRadioButton("Farbe (Syntax-Hervorhebung)")
        self.rb_bw = QRadioButton("Schwarz-Weiss")
        self.rb_color.setChecked(True)
        lay.addWidget(self.rb_color)
        lay.addWidget(self.rb_bw)

        self.cb_lines = QCheckBox("Zeilennummern")
        self.cb_lines.setChecked(True)
        lay.addWidget(self.cb_lines)

        self.rb_all = None
        self.rb_sel = None
        if has_selection:
            lay.addWidget(QLabel("Umfang:"))
            self.rb_sel = QRadioButton("Nur markierter Bereich")
            self.rb_all = QRadioButton("Ganzes Listing")
            self.rb_sel.setChecked(True)
            lay.addWidget(self.rb_sel)
            lay.addWidget(self.rb_all)

        buttons = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel,
            parent=self,
        )
        buttons.button(QDialogButtonBox.StandardButton.Ok).setText("Drucken ...")
        buttons.accepted.connect(self.accept)
        buttons.rejected.connect(self.reject)
        lay.addWidget(buttons)

    @property
    def color(self) -> bool:
        return self.rb_color.isChecked()

    @property
    def line_numbers(self) -> bool:
        return self.cb_lines.isChecked()

    @property
    def selection_only(self) -> bool:
        return self.rb_sel is not None and self.rb_sel.isChecked()


def print_code(parent, *, code: str, title: str, color: bool,
               line_numbers: bool) -> None:
    """Oeffnet eine Druckvorschau fuer das gegebene Listing."""
    html = build_listing_html(code, color=color, line_numbers=line_numbers, title=title)
    doc = QTextDocument()
    mono = QFont("Consolas", 10)
    mono.setStyleHint(QFont.StyleHint.Monospace)   # saubere Ersatz-Schrift, falls Consolas fehlt
    mono.setFixedPitch(True)
    doc.setDefaultFont(mono)
    doc.setHtml(html)

    printer = QPrinter(QPrinter.PrinterMode.HighResolution)
    printer.setDocName(title)
    if not color:
        printer.setColorMode(QPrinter.ColorMode.GrayScale)

    preview = QPrintPreviewDialog(printer, parent)
    preview.setWindowTitle(f"Druckvorschau – {title}")
    preview.paintRequested.connect(doc.print_)
    preview.resize(940, 720)
    preview.exec()
