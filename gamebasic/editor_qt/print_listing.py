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

from PySide6.QtCore import QMarginsF, QRectF, QSizeF, Qt
from PySide6.QtGui import (
    QAction, QColor, QFont, QPageLayout, QPainter, QTextDocument,
)
from PySide6.QtWidgets import (
    QCheckBox, QComboBox, QDialog, QDialogButtonBox, QDoubleSpinBox,
    QHBoxLayout, QLabel, QRadioButton, QToolBar, QVBoxLayout,
)
from PySide6.QtPrintSupport import QPrinter, QPrintPreviewDialog

from .highlighter import line_color_spans
from .icons import icons


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
    """(start, length, klasse)-Spans einer Zeile -- delegiert an die
    gemeinsame Quelle `highlighter.line_color_spans` (gleiche Klassifikation
    wie Editor + Minimap)."""
    return line_color_spans(line)


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

        size_row = QHBoxLayout()
        size_row.addWidget(QLabel("Schriftgrösse:"))
        self.cmb_size = QComboBox()
        # (Anzeigetext, Punktgrösse)
        self._sizes = [("Klein", 8), ("Normal", 10), ("Gross", 12)]
        for label, pt in self._sizes:
            self.cmb_size.addItem(f"{label} ({pt} pt)", pt)
        self.cmb_size.setCurrentIndex(1)   # Normal
        size_row.addWidget(self.cmb_size, 1)
        lay.addLayout(size_row)

        # Beidseitig (Duplex)
        duplex_row = QHBoxLayout()
        duplex_row.addWidget(QLabel("Seiten:"))
        self.cmb_duplex = QComboBox()
        # (Anzeige, key)
        self._duplex = [
            ("Einseitig", "none"),
            ("Doppelseitig (lange Kante)", "long"),
            ("Doppelseitig (kurze Kante)", "short"),
        ]
        for label, key in self._duplex:
            self.cmb_duplex.addItem(label, key)
        duplex_row.addWidget(self.cmb_duplex, 1)
        lay.addLayout(duplex_row)

        # Raender (mm) + Lochrand fuer Lochung
        margin_row = QHBoxLayout()
        margin_row.addWidget(QLabel("Seitenrand:"))
        self.sp_margin = QDoubleSpinBox()
        self.sp_margin.setRange(0.0, 50.0)
        self.sp_margin.setSingleStep(1.0)
        self.sp_margin.setValue(12.0)
        self.sp_margin.setSuffix(" mm")
        margin_row.addWidget(self.sp_margin)
        margin_row.addSpacing(12)
        margin_row.addWidget(QLabel("Lochrand:"))
        self.sp_binding = QDoubleSpinBox()
        self.sp_binding.setRange(0.0, 40.0)
        self.sp_binding.setSingleStep(1.0)
        self.sp_binding.setValue(0.0)
        self.sp_binding.setSuffix(" mm")
        self.sp_binding.setToolTip(
            "Zusaetzlicher Rand auf der Bindeseite (links) zum Lochen.\n"
            "Bei „Doppelseitig (lange Kante)“ wird er auf Rueckseiten gespiegelt.")
        margin_row.addWidget(self.sp_binding)
        margin_row.addStretch(1)
        lay.addLayout(margin_row)

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

    @property
    def font_pt(self) -> int:
        return int(self.cmb_size.currentData())

    @property
    def duplex(self) -> str:
        """'none' | 'long' | 'short'."""
        return str(self.cmb_duplex.currentData())

    @property
    def margin_mm(self) -> float:
        return float(self.sp_margin.value())

    @property
    def binding_mm(self) -> float:
        return float(self.sp_binding.value())


def _render_with_footer(doc: QTextDocument, printer: QPrinter, footer_left: str,
                        *, binding_mm: float = 0.0, mirror_binding: bool = False) -> None:
    """Druckt `doc` Seite fuer Seite und setzt unter jede Seite eine Fusszeile
    (Dateiname links, ``Seite X von Y`` rechts).

    Statt ``QTextDocument.print_`` (kann keine Kopf-/Fusszeilen) paginieren wir
    selbst: Die Dokument-Seitenhoehe wird um die Fusszeile verkleinert, dann
    zeichnen wir jede Seite per ``drawContents`` mit passendem Versatz.

    Der normale Seitenrand kommt aus den Drucker-Margins (`pageRect`). `binding_mm`
    ist ein ZUSAETZLICHER Lochrand auf der Bindeseite (links); bei
    `mirror_binding` (Doppelseitig lange Kante) wandert er auf Rueckseiten nach
    rechts, damit die Loecher beim Abheften fluchten.
    """
    painter = QPainter()
    if not painter.begin(printer):
        return
    try:
        # Das Dokument rechnet in 96-dpi-Logikpixeln; der Drucker hat eine viel
        # hoehere Aufloesung. Wir skalieren den Painter (genau wie es
        # QTextDocument.print_ intern tut) und arbeiten danach durchgehend in
        # 96-dpi-Logikpixeln.
        src_dpi = 96.0
        sx = printer.logicalDpiX() / src_dpi
        sy = printer.logicalDpiY() / src_dpi
        painter.scale(sx, sy)

        page = printer.pageRect(QPrinter.Unit.DevicePixel)
        page_w = page.width() / sx
        page_h = page.height() / sy

        # mm -> Logikpixel (96 dpi): unabhaengig von der Druckeraufloesung.
        bind = max(0.0, binding_mm) * src_dpi / 25.4
        bind = min(bind, page_w * 0.5)            # nie mehr als halbe Seite
        content_w = max(1.0, page_w - bind)

        footer_font = QFont("Arial")
        footer_font.setPixelSize(round(8 * src_dpi / 72.0))   # ~8 pt in Logikpixeln
        painter.setFont(footer_font)
        line_h = painter.fontMetrics().height()
        gap = line_h * 0.6
        footer_h = line_h + gap
        body_h = max(1.0, page_h - footer_h)

        doc.setPageSize(QSizeF(content_w, body_h))
        pages = max(1, doc.pageCount())

        for i in range(pages):
            if i > 0:
                printer.newPage()
            # Lochrand: Vorderseiten links, bei Spiegelung Rueckseiten rechts.
            binding_left = (not mirror_binding) or (i % 2 == 0)
            cx = bind if binding_left else 0.0
            # Seiteninhalt
            painter.save()
            painter.setClipRect(QRectF(cx, 0, content_w, body_h))
            painter.translate(cx, -i * body_h)
            doc.drawContents(painter, QRectF(0, i * body_h, content_w, body_h))
            painter.restore()
            # Fusszeile (folgt dem Lochrand)
            painter.save()
            painter.setFont(footer_font)
            painter.setPen(QColor("#808080"))
            frect = QRectF(cx, body_h + gap * 0.5, content_w, line_h)
            painter.drawText(frect, Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter,
                             footer_left)
            painter.drawText(frect, Qt.AlignmentFlag.AlignRight | Qt.AlignmentFlag.AlignVCenter,
                             f"Seite {i + 1} von {pages}")
            painter.restore()
    finally:
        painter.end()


def _decorate_preview(preview: QPrintPreviewDialog) -> None:
    """Macht die Vorschau-Werkzeugleiste lesbar.

    `QPrintPreviewDialog` benutzt monochrome Theme-Icons -- im dunklen Editor-
    Theme sind die kaum zu erkennen, und der Drucken-Knopf sitzt unscheinbar
    ganz rechts. Wir (1) beschriften alle Knoepfe mit Text und (2) setzen einen
    grossen, eindeutigen „Drucken"-Knopf mit eigenem (theme-faehigem) Icon an
    den Anfang der Leiste.
    """
    bars = preview.findChildren(QToolBar)
    if not bars:
        return
    tb = bars[0]
    tb.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextBesideIcon)

    # Englische Standard-Labels eindeutschen (greift nur, wenn Qt nicht ohnehin
    # schon lokalisiert ist -- sonst matcht der Key nicht und wir lassen es).
    de = {
        "Fit width": "Breite einpassen", "Fit page": "Seite einpassen",
        "Zoom out": "Verkleinern", "Zoom in": "Vergrössern",
        "Portrait": "Hochformat", "Landscape": "Querformat",
        "First page": "Erste Seite", "Previous page": "Zurück",
        "Next page": "Weiter", "Last page": "Letzte Seite",
        "Show single page": "Einzelseite", "Show facing pages": "Doppelseite",
        "Show overview of all pages": "Übersicht",
        "Page setup": "Seite einrichten", "Print": "Drucken",
    }
    for act in tb.actions():
        if act.text() in de:
            act.setText(de[act.text()])
            act.setToolTip(de[act.text()] if not act.toolTip() else act.toolTip())

    # Native Print-Action finden (Text 'Print'/'Drucken'); sonst: konventionell
    # die letzte echte Action.
    print_act = None
    for act in tb.actions():
        t = (act.text() or "").lower()
        if "print" in t or "druck" in t:
            print_act = act
            break
    if print_act is None:
        real = [a for a in tb.actions() if not a.isSeparator()]
        print_act = real[-1] if real else None
    if print_act is None:
        return

    print_act.setIcon(icons.get("print"))

    # Eigener, prominenter Drucken-Knopf ganz vorne.
    actions = tb.actions()
    first = actions[0] if actions else None
    my_print = QAction(icons.get("print"), "Drucken", preview)
    my_print.setToolTip("Listing drucken")
    my_print.triggered.connect(print_act.trigger)
    if first is not None:
        tb.insertAction(first, my_print)
        tb.insertSeparator(first)
    else:
        tb.addAction(my_print)


_DUPLEX_MODES = {
    "none": QPrinter.DuplexMode.DuplexNone,
    "long": QPrinter.DuplexMode.DuplexLongSide,
    "short": QPrinter.DuplexMode.DuplexShortSide,
}


def print_code(parent, *, code: str, title: str, color: bool,
               line_numbers: bool, font_pt: int = 10, duplex: str = "none",
               margin_mm: float = 12.0, binding_mm: float = 0.0) -> None:
    """Oeffnet eine Druckvorschau fuer das gegebene Listing."""
    html = build_listing_html(code, color=color, line_numbers=line_numbers,
                              title=title, font_pt=font_pt)
    doc = QTextDocument()
    mono = QFont("Consolas")
    mono.setPointSize(font_pt)
    mono.setStyleHint(QFont.StyleHint.Monospace)   # saubere Ersatz-Schrift, falls Consolas fehlt
    mono.setFixedPitch(True)
    doc.setDefaultFont(mono)
    doc.setDocumentMargin(0)
    doc.setHtml(html)

    printer = QPrinter(QPrinter.PrinterMode.HighResolution)
    printer.setDocName(title)
    if not color:
        printer.setColorMode(QPrinter.ColorMode.GrayScale)
    printer.setDuplex(_DUPLEX_MODES.get(duplex, QPrinter.DuplexMode.DuplexNone))
    # Gleichmaessiger Seitenrand (der Lochrand kommt zusaetzlich pro Seite dazu).
    m = max(0.0, margin_mm)
    printer.setPageMargins(QMarginsF(m, m, m, m), QPageLayout.Unit.Millimeter)

    mirror = duplex == "long"

    preview = QPrintPreviewDialog(printer, parent)
    preview.setWindowTitle(f"Druckvorschau – {title}")
    preview.paintRequested.connect(
        lambda pr: _render_with_footer(doc, pr, title,
                                       binding_mm=binding_mm, mirror_binding=mirror))
    _decorate_preview(preview)
    preview.resize(1040, 760)
    preview.exec()
