"""Code-Editor-Widget mit Zeilennummern, Snippets, Auto-Completion,
Hover-Tooltips und Find/Replace-API.

Aufbau:
- `QPlainTextEdit` als Editor.
- `_LineNumberArea` ist ein Kind-Widget links davon, das in `paintEvent`
  alle sichtbaren Zeilennummern zeichnet.
- Der Editor reserviert links den Platz via `setViewportMargins`.
- `QCompleter` mit gefuellter Wortliste fuer Auto-Completion (Strg+Leer
  oder automatisch ab 2 Zeichen, abschaltbar).
- Tab am Wortende prueft Snippet-Trigger; matched -> Body einfuegen,
  sonst -> Indent.
- Mouse-Hover zeigt Built-in-Doku als ToolTip.
"""
from __future__ import annotations

import re

from PySide6.QtCore import (
    QEvent, QPoint, QRect, QSize, Qt, QStringListModel, QTimer, Signal,
)
from PySide6.QtGui import (
    QColor, QFont, QPainter, QPen, QTextCursor, QTextFormat,
)
from PySide6.QtWidgets import (
    QCompleter, QInputDialog, QPlainTextEdit, QTextEdit, QToolTip, QWidget,
)

from .builtin_docs import get_doc
from .completer import all_completions
from .symbols import extract_user_doc
from .editor_actions import EditorActionsMixin
from .editor_intelligence import (
    AUTO_PAIRS, EditorIntelligenceMixin,
)
from .editor_multicursor import MultiCursorMixin
from .error_check import LiveErrorChecker, ParseProblem
from .folding import scan as scan_fold_regions
from .highlighter import GBHighlighter
from .snippets import SNIPPETS, expand_snippet, expand_snippet_full
from . import symbols as gb_symbols
from .theme import (
    COLORS, EDITOR_FONT_FAMILY, EDITOR_FONT_SIZE, theme_signals,
)


# Wie viele Zeichen ein Tab im Editor visuell breit ist. GB-Konvention =
# 4 Spaces; das entspricht der Konvention im uebrigen Codebase.
INDENT_SPACES = 4

# Breite der Fold-Marker-Spalte am rechten Rand der Zeilennummern-Spalte.
FOLD_MARGIN = 16
# Breite des linken Gutter-Bands fuer Breakpoint-Klicks/-Marker.
BP_ZONE = 16

# Identifier-Pattern fuer GB: Buchstabe oder _ vorne, alphanum/_ in der Mitte,
# optional ein Trailing-$ (Konvention fuer String-Variablen).
_IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*\$?")

# Color-Swatch-Detection. Wir markieren:
# 1. `&H` mit GENAU 6 Hex-Ziffern -- typische 24-bit-Color-Literal (RRGGBB).
#    1-5 Ziffern sind meist Bitmaske/Konstante, 7+ (ausser 8) ueberlaufen die
#    Range -- in beiden Faellen kein Swatch.
# 2. `&H` mit GENAU 8 Hex-Ziffern -- AARRGGBB (Alpha + Farbe, neue RGBA-Form).
# 3. `RGB(r, g, b)` und `RGBA(r, g, b, a)` mit Integer-Literalen 0..255.
#    Bei RGBA wird fuer die ANZEIGE nur RGB genutzt (deckend) -- man soll die
#    gewaehlte Farbe sehen; das Alpha bleibt aber fuers Editieren erhalten.
_HEX_COLOR_RE = re.compile(r"&H([0-9A-Fa-f]{6})(?![0-9A-Fa-f])")
_HEXA_COLOR_RE = re.compile(r"&H([0-9A-Fa-f]{8})(?![0-9A-Fa-f])")
_RGB_CALL_RE = re.compile(
    r"\bRGB\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)", re.IGNORECASE,
)
_RGBA_CALL_RE = re.compile(
    r"\bRGBA\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)", re.IGNORECASE,
)




# Smart-Indent-Trigger -- nach einer Zeile die mit einem dieser Patterns
# endet (case-insensitive), Enter rueckt zusaetzlich um INDENT_SPACES ein.
_INDENT_AFTER_PATTERNS = re.compile(
    r"^\s*(SUB|FUNCTION|CLASS|STRUCT|ENUM|WHILE|REPEAT|TRY|WITH|PROPERTY)\b"
    r"|\bTHEN\s*$"
    r"|^\s*FOR\b"
    r"|^\s*SELECT\s+CASE\b"
    r"|^\s*ELSE\s*$"
    r"|^\s*ELSEIF\b.*\bTHEN\s*$"
    r"|^\s*CATCH\b"
    r"|^\s*CASE\b",
    re.IGNORECASE,
)


class _LineNumberArea(QWidget):
    """Schmaler Streifen links neben dem Editor mit den Zeilennummern."""

    def __init__(self, editor: "CodeEditor"):
        super().__init__(editor)
        self._editor = editor
        # Wir wollen Klicks auf Fold-Pfeile abfangen.
        self.setMouseTracking(True)

    def sizeHint(self) -> QSize:  # noqa: N802 (Qt-API)
        return QSize(self._editor.line_number_area_width(), 0)

    def paintEvent(self, event):  # noqa: N802
        self._editor.paint_line_numbers(event)

    def mousePressEvent(self, event):  # noqa: N802
        if event.button() == Qt.MouseButton.LeftButton:
            x = int(event.position().x())
            w = self._editor.line_number_area_width()
            # Linkes Band -> Breakpoint toggeln.
            if x < BP_ZONE:
                line = self._editor._line_at_y(int(event.position().y()))
                if line is not None:
                    self._editor.toggle_breakpoint(line)
                    return
            # Rechte Spalte -> Fold-Marker.
            if x >= w - FOLD_MARGIN:
                handled = self._editor._handle_fold_click(int(event.position().y()))
                if handled:
                    return
        elif event.button() == Qt.MouseButton.RightButton:
            # Rechtsklick im Breakpoint-Band -> Bedingung bearbeiten.
            if int(event.position().x()) < BP_ZONE:
                line = self._editor._line_at_y(int(event.position().y()))
                if line is not None:
                    self._editor.edit_breakpoint_condition(line)
                    return
        super().mousePressEvent(event)


class CodeEditor(
    EditorActionsMixin,
    EditorIntelligenceMixin,
    MultiCursorMixin,
    QPlainTextEdit,
):
    """`QPlainTextEdit` mit Highlighter, Zeilennummern, Snippets, Completer.

    Funktionalitaet ist auf Mixins verteilt:
    - `EditorActionsMixin`: Indent / Comment-Toggle / Move-Line / Duplicate-Line
    - `EditorIntelligenceMixin`: Auto-Pair, Outdent-Trigger, Bracket-Match,
      Word-Highlight, Symbol-am-Cursor
    - `MultiCursorMixin`: Strg+D-Selection, Multi-Edit-Logik
    """

    save_requested = Signal()
    run_requested = Signal()
    open_requested = Signal()
    goto_definition_requested = Signal(str)   # Symbol-Name am Cursor
    peek_definition_requested = Signal(str)    # Symbol-Name am Cursor (Popup)
    find_references_requested = Signal(str)
    rename_requested = Signal(str)
    run_selection_requested = Signal(str)     # selektierter Code-Snippet
    breakpoints_changed = Signal()            # Gutter-Breakpoint getoggelt

    def __init__(self, parent=None):
        super().__init__(parent)
        font = QFont(EDITOR_FONT_FAMILY, EDITOR_FONT_SIZE)
        font.setStyleHint(QFont.StyleHint.Monospace)
        self.setFont(font)
        fm = self.fontMetrics()
        self.setTabStopDistance(fm.horizontalAdvance(" ") * INDENT_SPACES)
        self.setLineWrapMode(QPlainTextEdit.LineWrapMode.NoWrap)
        # Damit Mouse-Hover ohne gedrueckten Button feuert:
        self.setMouseTracking(True)

        self._highlighter = GBHighlighter(self.document())

        self._line_area = _LineNumberArea(self)
        self.blockCountChanged.connect(self._update_viewport_margins)
        self.updateRequest.connect(self._on_update_request)
        self.cursorPositionChanged.connect(self._highlight_current_line)

        # Find-Hits als ExtraSelections-Layer (separat von Current-Line).
        self._find_hits: list[tuple[int, int]] = []

        # Color-Literale (&HRRGGBB / RGB(r,g,b)) -- als ExtraSelections gerendert
        # (Hintergrund = die Farbe, Schrift im Kontrast). Cache wird bei jeder
        # Text-Aenderung neu aufgebaut; gerendert wird ueber die Text-Engine,
        # damit nichts flackert/verschwindet (kein Overlay-Clipping).
        self._color_literals: list[tuple[int, int, QColor, str]] = []
        self.textChanged.connect(self._rescan_color_literals)

        # Debugger: Breakpoint-Zeilen (1-basiert) + aktuelle Stop-Zeile.
        self._breakpoints: set[int] = set()
        # Conditional Breakpoints: Zeile -> GameBasic-Ausdruck (nur Zeilen
        # mit aktivem Breakpoint). Leer = unbedingter BP.
        self._bp_conditions: dict[int, str] = {}
        self._debug_line: int | None = None

        # Bookmarks (1-basierte Zeilen) -- Schnell-Navigation in langen Dateien.
        self._bookmarks: set[int] = set()

        # Multi-Cursor (Strg+D Add-Next-Occurrence). Liste sekundaerer
        # Selektionen als (start, end). Primaere Cursor bleibt im
        # `textCursor()`. Bei jeder Tasten-Eingabe werden alle Sekundaer-
        # Selektionen mit-gepresst.
        self._secondary: list[tuple[int, int]] = []

        # Live-Error-Check: laeuft async via QThread, debounced.
        self._error_problem: ParseProblem | None = None
        self._error_checker = LiveErrorChecker(self)
        self._error_checker.problem_changed.connect(self._on_error_problem)
        self._error_timer = QTimer(self)
        self._error_timer.setSingleShot(True)
        self._error_timer.setInterval(400)
        self._error_timer.timeout.connect(self._kick_error_check)
        self.textChanged.connect(self._error_timer.start)

        # Code-Folding. `_fold_regions` ist die Liste aller verfuegbaren
        # Klapp-Bloecke `(start_line, end_line, kind)` (1-basiert). `_folded`
        # enthaelt die start_line aller aktuell EINgeklappten Bloecke.
        # Re-Scan nach jedem textChanged mit kurzem Debounce, damit grosse
        # Buffer beim Tippen nicht jedes Mal komplett neu gescannt werden.
        self._fold_regions: list[tuple[int, int, str]] = []
        self._folded: set[int] = set()
        self._fold_scan_timer = QTimer(self)
        self._fold_scan_timer.setSingleShot(True)
        self._fold_scan_timer.setInterval(150)
        self._fold_scan_timer.timeout.connect(self._rescan_fold_regions)
        self.textChanged.connect(self._fold_scan_timer.start)

        self._setup_completer()
        self._auto_complete_enabled = True

        # Signature-Help (Parameter-Hints beim Tippen eines Aufrufs).
        from .signature_help import SignaturePopup
        self._sig_popup = SignaturePopup(self)
        self.cursorPositionChanged.connect(self._update_signature_help)
        self.verticalScrollBar().valueChanged.connect(
            lambda _v: self._sig_popup.hide())

        theme_signals.changed.connect(self._on_theme_changed)

        self._update_viewport_margins(0)
        self._rescan_color_literals()
        self._highlight_current_line()
        self._rescan_fold_regions()

    # ------------------------------------------------------------ API
    def get_text(self) -> str:
        return self.toPlainText()

    def set_text(self, content: str) -> None:
        self.setPlainText(content)
        self.document().setModified(False)
        self._find_hits = []
        self._refresh_extra_selections()

    def goto_line(self, line: int) -> None:
        line = max(1, line)
        block = self.document().findBlockByNumber(line - 1)
        if not block.isValid():
            block = self.document().lastBlock()
        cursor = QTextCursor(block)
        self.setTextCursor(cursor)
        self.centerCursor()

    def go_to(self, position: int) -> None:
        cursor = self.textCursor()
        cursor.setPosition(max(0, min(position, self.document().characterCount() - 1)))
        self.setTextCursor(cursor)
        self.ensureCursorVisible()

    def insert_at_cursor(self, text: str) -> None:
        self.textCursor().insertText(text)

    def set_auto_complete(self, enabled: bool) -> None:
        self._auto_complete_enabled = bool(enabled)
        if not enabled:
            self._completer.popup().hide()

    # ----------------------------------- Find / Replace API
    def find_all(self, query: str, *, case_sensitive: bool, whole_word: bool,
                 regex: bool) -> list[tuple[int, int]]:
        """Liefert (start, end) der Treffer als Zeichen-Offsets."""
        text = self.toPlainText()
        if not query:
            self._set_find_hits([])
            return []
        try:
            if regex:
                pat = query
            else:
                pat = re.escape(query)
            if whole_word:
                pat = rf"\b{pat}\b"
            flags = 0 if case_sensitive else re.IGNORECASE
            compiled = re.compile(pat, flags)
        except re.error:
            self._set_find_hits([])
            return []
        hits = [(m.start(), m.end()) for m in compiled.finditer(text)]
        self._set_find_hits(hits)
        return hits

    def clear_find_hits(self) -> None:
        self._set_find_hits([])

    def _set_find_hits(self, hits: list[tuple[int, int]]) -> None:
        self._find_hits = hits
        self._refresh_extra_selections()

    def replace_range(self, start: int, end: int, repl: str) -> None:
        cursor = self.textCursor()
        cursor.beginEditBlock()
        cursor.setPosition(start)
        cursor.setPosition(end, QTextCursor.MoveMode.KeepAnchor)
        cursor.insertText(repl)
        cursor.endEditBlock()

    # ------------------------------------------------ Line-Numbers
    def line_number_area_width(self) -> int:
        digits = max(2, len(str(max(1, self.blockCount()))))
        return 12 + self.fontMetrics().horizontalAdvance("9") * digits + FOLD_MARGIN

    def _update_viewport_margins(self, _new_block_count: int) -> None:
        self.setViewportMargins(self.line_number_area_width(), 0, 0, 0)

    def _on_update_request(self, rect: QRect, dy: int) -> None:
        if dy:
            self._line_area.scroll(0, dy)
        else:
            self._line_area.update(0, rect.y(), self._line_area.width(), rect.height())
        if rect.contains(self.viewport().rect()):
            self._update_viewport_margins(0)

    def resizeEvent(self, event):  # noqa: N802
        super().resizeEvent(event)
        cr = self.contentsRect()
        self._line_area.setGeometry(
            QRect(cr.left(), cr.top(), self.line_number_area_width(), cr.height())
        )

    def paint_line_numbers(self, event) -> None:
        painter = QPainter(self._line_area)
        painter.fillRect(event.rect(), QColor(COLORS["bg_alt"]))

        block = self.firstVisibleBlock()
        block_number = block.blockNumber()
        offset = self.contentOffset()
        top = self.blockBoundingGeometry(block).translated(offset).top()
        bottom = top + self.blockBoundingRect(block).height()

        active_line = self.textCursor().blockNumber()

        painter.setFont(self.font())
        col_active = QColor(COLORS["fg"])
        col_idle = QColor(COLORS["line_no"])
        col_fold = QColor(COLORS["accent"])
        col_err = QColor(COLORS["error"])
        right_pad = 6
        area_w = self._line_area.width()
        digit_w = area_w - FOLD_MARGIN
        # Mapping start_line -> end_line fuer schnellen Lookup beim Zeichnen.
        starts = {s: e for s, e, _ in self._fold_regions}
        err_line = self._error_problem.line if self._error_problem else None

        while block.isValid() and top <= event.rect().bottom():
            if block.isVisible() and bottom >= event.rect().top():
                # Zeilennummer
                painter.setPen(col_active if block_number == active_line else col_idle)
                painter.drawText(
                    0, int(top),
                    digit_w - right_pad,
                    self.fontMetrics().height(),
                    int(Qt.AlignmentFlag.AlignRight | Qt.AlignmentFlag.AlignVCenter),
                    str(block_number + 1),
                )
                # Fold-Marker (▾ offen / ▶ gefaltet) -- nur an Block-Anfaengen
                line1b = block_number + 1
                if line1b in starts:
                    folded = line1b in self._folded
                    glyph = "▶" if folded else "▾"
                    painter.setPen(col_fold)
                    painter.drawText(
                        digit_w, int(top),
                        FOLD_MARGIN, self.fontMetrics().height(),
                        int(Qt.AlignmentFlag.AlignCenter | Qt.AlignmentFlag.AlignVCenter),
                        glyph,
                    )
                # Error-Punkt: kleiner roter Kreis ganz links wenn diese
                # Zeile einen aktiven Parse-Fehler hat.
                if err_line is not None and line1b == err_line:
                    line_h = self.fontMetrics().height()
                    radius = max(2, line_h // 4)
                    cx = 4
                    cy = int(top) + line_h // 2
                    painter.setPen(col_err)
                    painter.setBrush(col_err)
                    painter.drawEllipse(cx - radius, cy - radius, radius * 2, radius * 2)
                    painter.setBrush(Qt.BrushStyle.NoBrush)
                # Bookmark: schmaler Mint-Balken am ganz linken Rand.
                if line1b in self._bookmarks:
                    line_h = self.fontMetrics().height()
                    painter.fillRect(0, int(top) + 1, 3, line_h - 2,
                                     QColor(COLORS["success"]))
                # Breakpoint: roter gefuellter Kreis im linken Gutter-Band.
                # Conditional Breakpoint: hohler Ring (Unterscheidung).
                if line1b in self._breakpoints:
                    line_h = self.fontMetrics().height()
                    r = max(3, line_h // 3)
                    cx = 8
                    cy = int(top) + line_h // 2
                    if line1b in self._bp_conditions:
                        pen = QPen(QColor(COLORS["danger"]))
                        pen.setWidth(2)
                        painter.setPen(pen)
                        painter.setBrush(Qt.BrushStyle.NoBrush)
                        painter.drawEllipse(cx - r, cy - r, r * 2, r * 2)
                    else:
                        painter.setPen(Qt.PenStyle.NoPen)
                        painter.setBrush(QColor(COLORS["danger"]))
                        painter.drawEllipse(cx - r, cy - r, r * 2, r * 2)
                        painter.setBrush(Qt.BrushStyle.NoBrush)
                # Debug-Stop-Zeile: gelber/accent Pfeil ueber dem Breakpoint.
                if self._debug_line is not None and line1b == self._debug_line:
                    line_h = self.fontMetrics().height()
                    painter.setPen(QColor(COLORS["accent"]))
                    painter.drawText(
                        0, int(top), BP_ZONE, line_h,
                        int(Qt.AlignmentFlag.AlignCenter
                            | Qt.AlignmentFlag.AlignVCenter),
                        "▶",
                    )
            block = block.next()
            top = bottom
            bottom = top + self.blockBoundingRect(block).height()
            block_number += 1

    # ------------------------------------------ Color-Swatches
    @staticmethod
    def _scan_color_swatches(text: str) -> list[tuple[int, int, QColor, str]]:
        """Liefert `(start_col, end_col, color, kind)` fuer alle erkannten
        Color-Literale in einer Zeile.

        `start_col`/`end_col` umschliessen das Literal, `color` traegt die
        gewaehlte Farbe (bei RGBA inkl. Alpha im QColor -- fuers Editieren),
        `kind` ist `"hex"` (`&HRRGGBB`), `"hexa"` (`&HAARRGGBB`), `"rgb"`
        (`RGB(r,g,b)`) oder `"rgba"` (`RGBA(r,g,b,a)`).
        """
        out: list[tuple[int, int, QColor, str]] = []
        for m in _HEX_COLOR_RE.finditer(text):
            try:
                v = int(m.group(1), 16)
            except ValueError:
                continue
            out.append((m.start(), m.end(),
                        QColor((v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF), "hex"))
        for m in _HEXA_COLOR_RE.finditer(text):
            try:
                v = int(m.group(1), 16)
            except ValueError:
                continue
            a = (v >> 24) & 0xFF
            a = 255 if a == 0 else a          # 0 = deckend (wie zur Laufzeit)
            out.append((m.start(), m.end(),
                        QColor((v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF, a), "hexa"))
        for m in _RGB_CALL_RE.finditer(text):
            try:
                r, g, b = int(m.group(1)), int(m.group(2)), int(m.group(3))
            except ValueError:
                continue
            if not (0 <= r <= 255 and 0 <= g <= 255 and 0 <= b <= 255):
                continue
            out.append((m.start(), m.end(), QColor(r, g, b), "rgb"))
        for m in _RGBA_CALL_RE.finditer(text):
            try:
                r, g, b, a = (int(m.group(1)), int(m.group(2)),
                              int(m.group(3)), int(m.group(4)))
            except ValueError:
                continue
            if not all(0 <= v <= 255 for v in (r, g, b, a)):
                continue
            out.append((m.start(), m.end(), QColor(r, g, b, a), "rgba"))
        return out

    def _rescan_color_literals(self) -> None:
        """Baut den Cache aller Color-Literale im Dokument neu auf (absolute
        Positionen) und aktualisiert die ExtraSelections. Billig genug fuer
        jeden Text-Change (Regex ueber kurze Zeilen)."""
        lits: list[tuple[int, int, QColor, str]] = []
        doc = self.document()
        block = doc.firstBlock()
        while block.isValid():
            text = block.text()
            if "&H" in text or "RGB" in text.upper():
                base = block.position()
                for s, e, color, kind in self._scan_color_swatches(text):
                    lits.append((base + s, base + e, color, kind))
            block = block.next()
        self._color_literals = lits
        self._refresh_extra_selections()

    @staticmethod
    def _swatch_text_color(color: QColor) -> QColor:
        """Gut lesbare Schriftfarbe auf `color` als Hintergrund: dunkel auf
        hellen Farben, hell auf dunklen (Helligkeit nach ITU-R BT.601)."""
        lum = 0.299 * color.red() + 0.587 * color.green() + 0.114 * color.blue()
        return QColor(25, 25, 25) if lum > 140 else QColor(240, 240, 240)

    def _swatch_at(self, pos) -> tuple | None:
        """`(abs_start, abs_end, color, kind)` des Color-Literals unter `pos`
        (Viewport-Koordinate) oder None. Klick-Bereich = das Literal selbst."""
        abs_pos = self.cursorForPosition(pos).position()
        for start, end, color, kind in self._color_literals:
            if start <= abs_pos < end:
                return (start, end, color, kind)
        return None

    def _edit_color_literal(self, abs_start: int, abs_end: int,
                            color: QColor, kind: str) -> None:
        """Oeffnet den Farbwaehler und ersetzt das Literal im selben Format.

        Bei `rgba`/`hexa` ist der Alpha-Kanal im Dialog editierbar (vorbelegt
        mit dem Literal-Alpha); die anderen Formen bleiben rein RGB."""
        from PySide6.QtWidgets import QColorDialog
        has_alpha = kind in ("rgba", "hexa")
        opts = QColorDialog.ColorDialogOption.ShowAlphaChannel if has_alpha \
            else QColorDialog.ColorDialogOption(0)
        new = QColorDialog.getColor(color, self, "Farbe waehlen", opts)
        if not new.isValid():
            return
        r, g, b, al = new.red(), new.green(), new.blue(), new.alpha()
        if kind == "rgb":
            repl = f"RGB({r}, {g}, {b})"
        elif kind == "rgba":
            repl = f"RGBA({r}, {g}, {b}, {al})"
        elif kind == "hexa":
            repl = f"&H{al:02X}{r:02X}{g:02X}{b:02X}"
        else:  # "hex"
            repl = f"&H{r:02X}{g:02X}{b:02X}"
        cur = QTextCursor(self.document())
        cur.setPosition(abs_start)
        cur.setPosition(abs_end, QTextCursor.MoveMode.KeepAnchor)
        cur.insertText(repl)

    def paintEvent(self, event):  # noqa: N802
        super().paintEvent(event)
        # Nach dem normalen Text-Render-Pass: Indent-Guides. (Color-Literale
        # laufen ueber ExtraSelections, nicht ueber diesen Overlay.)
        painter = QPainter(self.viewport())
        block = self.firstVisibleBlock()
        offset = self.contentOffset()
        viewport_h = self.viewport().height()
        guide_color = QColor(COLORS["indent_guide"])
        # Pixel-Breite eines Spaces -- fuer die Indent-Guide-Positionen.
        space_w = self.fontMetrics().horizontalAdvance(" ")
        while block.isValid():
            block_rect = self.blockBoundingGeometry(block).translated(offset)
            if block_rect.top() > viewport_h:
                break
            if not block.isVisible() or block_rect.bottom() < 0:
                block = block.next()
                continue
            text = block.text()
            # --- Indent-Guides ---------------------------------------
            # Zeichne fuer jede Indent-Stufe (4 Spaces) eine 1px-vertikale
            # Linie. Funktioniert auch bei leeren Zeilen, sofern
            # darueber/darunter eingeruckter Code steht -- naive Variante
            # zeichnet nur fuer Zeilen mit eigener Indentation.
            stripped_lstrip = text.lstrip(" ")
            indent_chars = len(text) - len(stripped_lstrip)
            n_guides = indent_chars // INDENT_SPACES
            if n_guides > 0:
                top_y = int(block_rect.top())
                bot_y = int(block_rect.bottom())
                painter.setPen(guide_color)
                for i in range(1, n_guides + 1):
                    x = int(block_rect.left()) + space_w * INDENT_SPACES * i
                    painter.drawLine(x, top_y, x, bot_y)
            # Color-Literale werden NICHT hier gezeichnet, sondern als
            # ExtraSelections (Hintergrund+Schriftfarbe) -- so flackert nichts
            # und nichts verschwindet bei Teil-Repaints (Cursor-Blinken/Scroll).
            block = block.next()

    # ----------------------------------------------- Live-Errors
    def _kick_error_check(self) -> None:
        # Aufrufer (MainWindow) setzt vorher self._error_base via
        # `set_error_base_path`, sodass IMPORT relativ aufgeloest werden
        # kann. Wenn nicht gesetzt: Cwd reicht (Standalone-Editor-Tests).
        base = getattr(self, "_error_base", None)
        self._error_checker.check(self.toPlainText(), base)

    def set_error_base_path(self, base) -> None:
        """Setzt das Verzeichnis, gegen das IMPORT-Pfade aufgeloest werden.

        Sinnvollerweise das Verzeichnis der Tab-Datei, sonst project_root.
        """
        self._error_base = base

    def _on_error_problem(self, problem) -> None:
        self._error_problem = problem
        self._refresh_extra_selections()
        self._line_area.update()

    def current_error(self) -> ParseProblem | None:
        return self._error_problem

    # ----------------------------------------------- Code-Folding
    def _rescan_fold_regions(self) -> None:
        self._fold_regions = scan_fold_regions(self.toPlainText())
        valid_starts = {s for s, _e, _k in self._fold_regions}
        dropped = self._folded - valid_starts
        if dropped:
            # Eine bisher gefaltete Region wurde durch Edits verschoben,
            # sodass ihre Start-Zeile nicht mehr stimmt. Da Line-Numbers
            # bei Edits nicht stabil sind, klappen wir defensiv alles auf,
            # damit kein "Geist-Block" unsichtbar haengen bleibt. Der User
            # kann jederzeit erneut falten.
            self._unfold_all_blocks()
            self._folded.clear()
        self._line_area.update()

    def _unfold_all_blocks(self) -> None:
        """Stellt sicher, dass jeder Block sichtbar ist."""
        for ln in range(1, self.document().blockCount() + 1):
            blk = self.document().findBlockByNumber(ln - 1)
            if blk.isValid() and not blk.isVisible():
                blk.setVisible(True)
        self.document().markContentsDirty(0, self.document().characterCount())
        self.viewport().update()

    def _handle_fold_click(self, y_pixel: int) -> bool:
        """Identifiziert den Block bei der Y-Pixel-Position und togglet ihn,
        sofern er Block-Anfang einer Fold-Region ist."""
        block = self.firstVisibleBlock()
        offset = self.contentOffset()
        top = self.blockBoundingGeometry(block).translated(offset).top()
        bottom = top + self.blockBoundingRect(block).height()
        while block.isValid():
            if block.isVisible() and top <= y_pixel < bottom:
                line = block.blockNumber() + 1
                for s, e, _k in self._fold_regions:
                    if s == line:
                        self._toggle_fold(s, e)
                        return True
                return False
            block = block.next()
            top = bottom
            bottom = top + self.blockBoundingRect(block).height()
        return False

    def _line_at_y(self, y_pixel: int) -> int | None:
        """1-basierte Zeile an einer Y-Pixel-Position im Gutter (oder None)."""
        block = self.firstVisibleBlock()
        offset = self.contentOffset()
        top = self.blockBoundingGeometry(block).translated(offset).top()
        bottom = top + self.blockBoundingRect(block).height()
        while block.isValid():
            if block.isVisible() and top <= y_pixel < bottom:
                return block.blockNumber() + 1
            block = block.next()
            top = bottom
            bottom = top + self.blockBoundingRect(block).height()
        return None

    # ----------------------------------------------- Breakpoints / Debug
    def toggle_breakpoint(self, line: int) -> None:
        if line in self._breakpoints:
            self._breakpoints.discard(line)
            self._bp_conditions.pop(line, None)   # Bedingung mit entfernen
        else:
            self._breakpoints.add(line)
        self._line_area.update()
        self.breakpoints_changed.emit()

    def edit_breakpoint_condition(self, line: int) -> None:
        """Fragt eine Bedingung (GameBasic-Ausdruck) fuer den Breakpoint auf
        `line` ab. Setzt automatisch einen Breakpoint, falls noch keiner da
        ist. Leere Eingabe -> unbedingter Breakpoint."""
        cur = self._bp_conditions.get(line, "")
        expr, ok = QInputDialog.getText(
            self, f"Breakpoint-Bedingung (Zeile {line})",
            "Anhalten, wenn Ausdruck wahr ist (leer = immer):", text=cur)
        if not ok:
            return
        expr = expr.strip()
        self._breakpoints.add(line)
        if expr:
            self._bp_conditions[line] = expr
        else:
            self._bp_conditions.pop(line, None)
        self._line_area.update()
        self.breakpoints_changed.emit()

    def breakpoint_conditions(self) -> dict[int, str]:
        return dict(self._bp_conditions)

    # ----------------------------------------------- Bookmarks
    def toggle_bookmark(self) -> None:
        line = self.textCursor().blockNumber() + 1
        if line in self._bookmarks:
            self._bookmarks.discard(line)
        else:
            self._bookmarks.add(line)
        self._line_area.update()

    def _goto_bookmark(self, forward: bool) -> None:
        if not self._bookmarks:
            return
        cur = self.textCursor().blockNumber() + 1
        marks = sorted(self._bookmarks)
        if forward:
            nxt = next((m for m in marks if m > cur), marks[0])      # wrap
        else:
            nxt = next((m for m in reversed(marks) if m < cur), marks[-1])
        blk = self.document().findBlockByNumber(nxt - 1)
        if blk.isValid():
            self.setTextCursor(QTextCursor(blk))
            self.ensureCursorVisible()

    def next_bookmark(self) -> None:
        self._goto_bookmark(True)

    def prev_bookmark(self) -> None:
        self._goto_bookmark(False)

    def set_word_wrap(self, on: bool) -> None:
        self.setLineWrapMode(
            QPlainTextEdit.LineWrapMode.WidgetWidth if on
            else QPlainTextEdit.LineWrapMode.NoWrap)

    def breakpoints(self) -> set[int]:
        return set(self._breakpoints)

    def set_breakpoints(self, lines) -> None:
        self._breakpoints = set(int(x) for x in lines)
        # Bedingungen ohne zugehoerigen Breakpoint verwerfen.
        self._bp_conditions = {ln: c for ln, c in self._bp_conditions.items()
                               if ln in self._breakpoints}
        self._line_area.update()

    def set_debug_line(self, line: int | None) -> None:
        """Markiert die aktuelle Stop-Zeile (Pfeil + Zeilen-Highlight)."""
        self._debug_line = line
        self._refresh_extra_selections()
        self._line_area.update()
        if line is not None:
            blk = self.document().findBlockByNumber(line - 1)
            if blk.isValid():
                cur = QTextCursor(blk)
                self.setTextCursor(cur)
                self.ensureCursorVisible()

    def _toggle_fold(self, start_line: int, end_line: int) -> None:
        """Schaltet die Sichtbarkeit von Bloecken (start, end] um."""
        folded_now = start_line in self._folded
        new_visible = folded_now    # bei "war gefaltet" -> jetzt wieder sichtbar
        for ln in range(start_line + 1, end_line + 1):
            blk = self.document().findBlockByNumber(ln - 1)
            if not blk.isValid():
                continue
            blk.setVisible(new_visible)
        if folded_now:
            self._folded.discard(start_line)
        else:
            self._folded.add(start_line)
        # Layout-Update erzwingen -- ohne diesen Hint wirken setVisible-Aenderungen
        # erst beim naechsten Repaint und der Scrollbalken bleibt veraltet.
        self.document().markContentsDirty(0, self.document().characterCount())
        self.viewport().update()
        self._line_area.update()

    def fold_block_at_cursor(self) -> None:
        """Faltet den innersten Block, der die Cursor-Zeile umgibt."""
        cur_line = self.textCursor().blockNumber() + 1
        # Kleinster (= innerster) enthaltender Block.
        match: tuple[int, int] | None = None
        for s, e, _k in self._fold_regions:
            if s <= cur_line <= e:
                if match is None or (e - s) < (match[1] - match[0]):
                    match = (s, e)
        if match is not None:
            self._toggle_fold(*match)

    def unfold_all(self) -> None:
        """Klappt alle Bloecke wieder auf."""
        if not self._folded:
            return
        self._unfold_all_blocks()
        self._folded.clear()
        self._line_area.update()

    def folded_starts(self) -> list[int]:
        """Liefert die Start-Zeilen aller aktuell eingeklappten Bloecke
        (1-basiert). Fuer Persistierung in Settings: nach Close den
        Wert wegspeichern, beim Re-Open via `apply_folded_starts()`
        wiederherstellen."""
        return sorted(self._folded)

    def apply_folded_starts(self, starts) -> None:
        """Klappt die uebergebenen Start-Zeilen wieder ein. Wird typisch
        nach dem `set_text()`-Load aufgerufen, sobald `_fold_regions`
        gescannt sind. Nicht-existente Start-Zeilen werden ignoriert
        (z.B. wenn die Datei zwischenzeitlich extern editiert wurde
        und die Zeilen-Nummern nicht mehr stimmen)."""
        if not starts:
            return
        # Re-scan damit wir aktuelle Fold-Regions haben.
        self._fold_regions = scan_fold_regions(self.toPlainText())
        valid = {s: e for s, e, _k in self._fold_regions}
        for s in starts:
            if s in valid and s not in self._folded:
                self._toggle_fold(s, valid[s])
        self._line_area.update()

    # ------------------------------------------ Current-Line + Find
    def _highlight_current_line(self) -> None:
        self._refresh_extra_selections()

    def _refresh_extra_selections(self) -> None:
        selections: list[QTextEdit.ExtraSelection] = []
        if not self.isReadOnly():
            sel = QTextEdit.ExtraSelection()
            sel.format.setBackground(QColor(COLORS["current_line"]))
            sel.format.setProperty(
                QTextFormat.Property.FullWidthSelection, True
            )
            cursor = self.textCursor()
            cursor.clearSelection()
            sel.cursor = cursor
            selections.append(sel)
        # Debug-Stop-Zeile: volle Zeile in gedaempftem Akzent hervorheben.
        if self._debug_line is not None:
            dbg_block = self.document().findBlockByNumber(self._debug_line - 1)
            if dbg_block.isValid():
                dbg_sel = QTextEdit.ExtraSelection()
                dbg_sel.format.setBackground(QColor(COLORS["accent_soft"]))
                dbg_sel.format.setProperty(
                    QTextFormat.Property.FullWidthSelection, True)
                dc = QTextCursor(dbg_block)
                dc.clearSelection()
                dbg_sel.cursor = dc
                selections.append(dbg_sel)
        # Find-Hits in akzent-Farbe ueberlagern.
        for start, end in self._find_hits:
            hit_sel = QTextEdit.ExtraSelection()
            hit_sel.format.setBackground(QColor(COLORS["find_hit"]))
            cursor = QTextCursor(self.document())
            cursor.setPosition(start)
            cursor.setPosition(end, QTextCursor.MoveMode.KeepAnchor)
            hit_sel.cursor = cursor
            selections.append(hit_sel)
        # Sekundaere Multi-Cursor-Selektionen.
        for start, end in self._secondary:
            sec_sel = QTextEdit.ExtraSelection()
            sec_sel.format.setBackground(QColor(COLORS["sel"]))
            cursor = QTextCursor(self.document())
            cursor.setPosition(start)
            cursor.setPosition(end, QTextCursor.MoveMode.KeepAnchor)
            sec_sel.cursor = cursor
            selections.append(sec_sel)
        # Word-Highlight: wenn der User ein einfaches Wort selektiert hat,
        # alle weiteren Vorkommen subtil markieren -- so siehst du auf einen
        # Blick, wo dieser Identifier sonst noch auftaucht.
        for s_, e_ in self._word_highlight_ranges():
            wh_sel = QTextEdit.ExtraSelection()
            from PySide6.QtGui import QTextCharFormat
            fmt = wh_sel.format
            # Halbtransparenter Akzent als Hintergrund.
            col = QColor(COLORS["accent"])
            col.setAlpha(60)
            fmt.setBackground(col)
            tc = QTextCursor(self.document())
            tc.setPosition(s_)
            tc.setPosition(e_, QTextCursor.MoveMode.KeepAnchor)
            wh_sel.cursor = tc
            selections.append(wh_sel)
        # Live-Error-Underline: rote Wellenlinie auf der Fehler-Zeile.
        if self._error_problem is not None:
            err_line = max(1, self._error_problem.line)
            err_block = self.document().findBlockByNumber(err_line - 1)
            if err_block.isValid():
                err_sel = QTextEdit.ExtraSelection()
                fmt = err_sel.format
                from PySide6.QtGui import QTextCharFormat
                fmt.setUnderlineStyle(QTextCharFormat.UnderlineStyle.WaveUnderline)
                fmt.setUnderlineColor(QColor(COLORS["error"]))
                fmt.setProperty(
                    QTextFormat.Property.FullWidthSelection, True
                )
                c = QTextCursor(err_block)
                c.movePosition(QTextCursor.MoveOperation.EndOfBlock,
                                QTextCursor.MoveMode.KeepAnchor)
                err_sel.cursor = c
                selections.append(err_sel)
        # Bracket-Matching: zwei kleine Highlights wenn der Cursor auf
        # oder direkt rechts neben einer Klammer steht.
        for start, end in self._matching_bracket_positions():
            br_sel = QTextEdit.ExtraSelection()
            fmt = br_sel.format
            fmt.setBackground(QColor(COLORS["accent"]))
            fmt.setForeground(QColor(COLORS["accent_text"]))
            c = QTextCursor(self.document())
            c.setPosition(start)
            c.setPosition(end, QTextCursor.MoveMode.KeepAnchor)
            br_sel.cursor = c
            selections.append(br_sel)
        # Color-Literale: das Literal selbst mit seiner Farbe hinterlegen, die
        # Schrift darueber im Kontrast. Als ExtraSelection (Teil des Text-
        # Renderings) -- ueberdeckt nie Nachbar-Code und flackert nicht.
        for start, end, color, _kind in self._color_literals:
            cl_sel = QTextEdit.ExtraSelection()
            # Hintergrund IMMER deckend zeichnen -- man soll die gewaehlte Farbe
            # klar sehen (auch bei RGBA mit niedrigem Alpha).
            opaque = QColor(color.red(), color.green(), color.blue())
            cl_sel.format.setBackground(opaque)
            cl_sel.format.setForeground(self._swatch_text_color(opaque))
            c = QTextCursor(self.document())
            c.setPosition(start)
            c.setPosition(end, QTextCursor.MoveMode.KeepAnchor)
            cl_sel.cursor = c
            selections.append(cl_sel)
        self.setExtraSelections(selections)




    # ------------------------------------------------- Completer
    def _setup_completer(self) -> None:
        self._completion_model = QStringListModel(all_completions(), self)
        self._completer = QCompleter(self._completion_model, self)
        self._completer.setWidget(self)
        self._completer.setCompletionMode(QCompleter.CompletionMode.PopupCompletion)
        self._completer.setCaseSensitivity(Qt.CaseSensitivity.CaseInsensitive)
        self._completer.setWrapAround(False)
        self._completer.activated.connect(self._insert_completion)

    def _insert_completion(self, completion: str) -> None:
        cursor = self.textCursor()
        prefix = self._completer.completionPrefix()
        cursor.movePosition(
            QTextCursor.MoveOperation.Left,
            QTextCursor.MoveMode.KeepAnchor,
            len(prefix),
        )
        cursor.insertText(completion)
        self.setTextCursor(cursor)

    def _current_word_prefix(self) -> str:
        cursor = self.textCursor()
        col = cursor.positionInBlock()
        text = cursor.block().text()
        # Rueckwaerts laufen, solange Identifier-Zeichen.
        i = col
        while i > 0 and (text[i - 1].isalnum() or text[i - 1] in ("_", "$")):
            i -= 1
        return text[i:col]

    def _maybe_show_completer(self, prefix: str) -> None:
        if not self._auto_complete_enabled:
            return
        if len(prefix) < 2:
            self._completer.popup().hide()
            return
        self._show_completer(prefix)

    def _show_completer(self, prefix: str) -> None:
        self._completer.setCompletionPrefix(prefix)
        popup = self._completer.popup()
        popup.setCurrentIndex(self._completer.completionModel().index(0, 0))
        cr = self.cursorRect()
        cr.setWidth(
            popup.sizeHintForColumn(0) + popup.verticalScrollBar().sizeHint().width()
        )
        # Popup-Position relativ zum Cursor-Rect, etwas tiefer (Rect ist Char-Rect).
        cr.translate(self.viewportMargins().left(), self.viewportMargins().top())
        self._completer.complete(cr)

    # ------------------------------------- Signature-Help
    def _update_signature_help(self) -> None:
        """Zeigt/aktualisiert das Parameter-Hint-Popup je nach Cursor-Kontext.

        Laeuft bei jeder Cursor-Bewegung -- bewusst guenstig gehalten:
        nur die letzten ~2000 Zeichen vor dem Cursor werden geparst, die
        volle Quelle (fuer User-Funktionen) erst geholt, wenn ein Aufruf
        erkannt wurde und kein Built-in passt."""
        from .signature_help import (
            find_active_call, render_signature_html,
        )
        # Completer-Popup hat Vorrang -- nicht ueberlagern.
        if self._completer.popup().isVisible():
            self._sig_popup.hide()
            return
        cur = self.textCursor()
        if cur.hasSelection():
            self._sig_popup.hide()
            return
        pos = cur.position()
        c = self.textCursor()
        c.setPosition(max(0, pos - 2000))
        c.setPosition(pos, QTextCursor.MoveMode.KeepAnchor)
        # selectedText nutzt U+2029 (Paragraph-Separator) als Zeilentrenner.
        text_before = c.selectedText().replace("\u2029", "\n")
        call = find_active_call(text_before)
        if call is None:
            self._sig_popup.hide()
            return
        name, arg_index = call
        sig = self._builtin_signature(name)
        if sig is None:
            sig = self._user_signature(name)
        if not sig:
            self._sig_popup.hide()
            return
        html = render_signature_html(sig, arg_index)
        rect = self.cursorRect()
        top_left = self.viewport().mapToGlobal(rect.bottomLeft())
        anchor_top = self.viewport().mapToGlobal(rect.topLeft())
        self._sig_popup.show_html(html, top_left, anchor_top)

    @staticmethod
    def _builtin_signature(name: str) -> str | None:
        # Kuratierte Doku-Signatur bevorzugen (benannte Params, z.B.
        # "LINE(x1, y1, x2, y2[, farbe])") -- die Registry liefert bei
        # variabler Arity nur "N Argumente".
        from .builtin_docs import get_doc
        doc = get_doc(name)
        if doc and doc[0]:
            return doc[0]
        from .gbrt_meta import signature
        return signature(name) or None

    def _user_signature(self, name: str) -> str | None:
        from .symbols import extract_user_doc
        res = extract_user_doc(self.get_text(), name)
        if res is None:
            return None
        sig, _doc = res
        # Nur SUB/FUNCTION (haben eine Param-Klammer) -- kein DIM/CONST/CLASS.
        return sig if "(" in sig else None

    # ------------------------------------- Multi-Cursor (Strg+D)



    # ------------------------------------- Tab / Auto-Indent / Snippet
    def keyPressEvent(self, event):  # noqa: N802
        # Esc raeumt das Signature-Help-Popup ab (ohne den Event zu schlucken --
        # weitere Esc-Handler wie Multi-Cursor laufen danach normal).
        if event.key() == Qt.Key.Key_Escape:
            self._sig_popup.hide()
        # Wenn das Completer-Popup sichtbar ist: Enter/Tab -> Auswahl uebernehmen,
        # Esc -> ausblenden, Up/Down -> Popup-Navigation (default-Forward).
        if self._completer.popup().isVisible():
            if event.key() in (
                Qt.Key.Key_Enter, Qt.Key.Key_Return, Qt.Key.Key_Tab,
                Qt.Key.Key_Up, Qt.Key.Key_Down,
            ):
                event.ignore()
                return
            if event.key() == Qt.Key.Key_Escape:
                self._completer.popup().hide()
                return

        key = event.key()
        mods = event.modifiers()
        ctrl = bool(mods & Qt.KeyboardModifier.ControlModifier)
        shift = bool(mods & Qt.KeyboardModifier.ShiftModifier)

        # Strg+Space -> Completer manuell oeffnen.
        if ctrl and key == Qt.Key.Key_Space:
            prefix = self._current_word_prefix()
            self._show_completer(prefix)
            return

        # Strg+D -> naechstes Vorkommen zur Multi-Selection.
        # Strg+Shift+D -> aktuelle Zeile duplizieren.
        if ctrl and key == Qt.Key.Key_D:
            if shift:
                self.duplicate_lines()
            else:
                self.add_next_occurrence()
            return

        # Strg+/ -> Comment-Toggle.
        if ctrl and key == Qt.Key.Key_Slash:
            self.comment_toggle()
            return

        # Alt+Up/Down -> Zeile(n) verschieben.
        if mods & Qt.KeyboardModifier.AltModifier and not ctrl:
            if key == Qt.Key.Key_Up:
                self.move_lines(-1)
                return
            if key == Qt.Key.Key_Down:
                self.move_lines(+1)
                return

        # Font-Zoom Strg+Plus/Minus/0
        if ctrl and not shift:
            if key in (Qt.Key.Key_Plus, Qt.Key.Key_Equal):
                self.adjust_font_size(+1)
                return
            if key == Qt.Key.Key_Minus:
                self.adjust_font_size(-1)
                return
            if key == Qt.Key.Key_0:
                self.reset_font_size()
                return

        # Strg+Shift+[ -> Block am Cursor falten/auffalten.
        # Strg+Shift+] -> alle aufklappen.
        if ctrl and shift and key == Qt.Key.Key_BracketLeft:
            self.fold_block_at_cursor()
            return
        if ctrl and shift and key == Qt.Key.Key_BracketRight:
            self.unfold_all()
            return

        # Modifier-Shortcuts an MainWindow weitergeben (S/O/N/F/H/G/...).
        if ctrl and key in (
            Qt.Key.Key_S, Qt.Key.Key_O, Qt.Key.Key_N, Qt.Key.Key_F,
            Qt.Key.Key_H, Qt.Key.Key_G, Qt.Key.Key_W, Qt.Key.Key_Q,
            Qt.Key.Key_T, Qt.Key.Key_Comma,
        ):
            event.ignore()
            return
        if key in (Qt.Key.Key_F1, Qt.Key.Key_F2, Qt.Key.Key_F3, Qt.Key.Key_F4):
            event.ignore()
            return

        # Esc loescht Multi-Selektion (vor allen anderen Esc-Handlern).
        if key == Qt.Key.Key_Escape and self._secondary:
            self._clear_secondary_cursors()
            return

        # Multi-Cursor aktiv: editierende Tasten gleichzeitig auf alle
        # Selektionen anwenden. Navigation/Funktion bleibt single-cursor.
        if self._secondary:
            text = event.text()
            if key == Qt.Key.Key_Backspace:
                self._multi_edit(lambda c: c.deletePreviousChar() if not c.hasSelection() else c.removeSelectedText())
                return
            if key == Qt.Key.Key_Delete:
                self._multi_edit(lambda c: c.deleteChar() if not c.hasSelection() else c.removeSelectedText())
                return
            if text and text.isprintable():
                self._multi_edit(lambda c, t=text: c.insertText(t))
                return
            # Bei anderen Tasten (Pfeile, F-Tasten, ...) Multi-Selektion
            # verwerfen und normal weiter.
            self._clear_secondary_cursors()

        # F5 -- Run
        if key == Qt.Key.Key_F5 and not mods:
            self.run_requested.emit()
            return

        # Shift+F5 -- Run Selection. Wenn nichts selektiert ist, faellt
        # die Aktion still aus -- der MainWindow-Handler entscheidet dann
        # was er damit tut (typischerweise: Statusbar-Hinweis, kein Run).
        if key == Qt.Key.Key_F5 and mods == Qt.KeyboardModifier.ShiftModifier:
            cur = self.textCursor()
            if cur.hasSelection():
                self.run_selection_requested.emit(cur.selectedText().replace(" ", "\n"))
            return

        # Alt+F12 -- Peek-Definition (Popup, ohne Sprung)
        if key == Qt.Key.Key_F12 and (mods & Qt.KeyboardModifier.AltModifier):
            sym = self._symbol_under_cursor()
            if sym:
                self.peek_definition_requested.emit(sym)
            return

        # F12 -- Goto-Definition
        if key == Qt.Key.Key_F12 and not (mods & Qt.KeyboardModifier.ShiftModifier):
            sym = self._symbol_under_cursor()
            if sym:
                self.goto_definition_requested.emit(sym)
            return

        # Shift+F12 -- Find-References
        if key == Qt.Key.Key_F12 and (mods & Qt.KeyboardModifier.ShiftModifier):
            sym = self._symbol_under_cursor()
            if sym:
                self.find_references_requested.emit(sym)
            return

        # F2 -- Rename
        if key == Qt.Key.Key_F2 and not mods:
            sym = self._symbol_under_cursor()
            if sym:
                self.rename_requested.emit(sym)
            return

        if key == Qt.Key.Key_Tab and not shift:
            if self._try_expand_snippet():
                return
            self._insert_indent()
            return
        if key == Qt.Key.Key_Backtab or (key == Qt.Key.Key_Tab and shift):
            self._remove_indent()
            return
        if key in (Qt.Key.Key_Return, Qt.Key.Key_Enter):
            self._auto_indent_newline()
            return

        # Auto-Pair: opener fuegt closer mit ein, closer skip-over wenn
        # er schon rechts vom Cursor steht. Wir greifen das vor super(),
        # damit Default-Insert nicht doppelt zuschlaegt.
        text_in = event.text()
        if not ctrl and len(text_in) == 1 and text_in in (set(AUTO_PAIRS) | set(AUTO_PAIRS.values())):
            if self._try_auto_pair(text_in):
                # Auch hier: nach Insert ggf. Completer + Outdent-Check.
                if text_in.isprintable():
                    prefix = self._current_word_prefix()
                    self._maybe_show_completer(prefix)
                self._maybe_outdent_line()
                return

        super().keyPressEvent(event)

        # Nach normaler Tasteneingabe: Completer maybe zeigen.
        if not ctrl and event.text() and event.text().isprintable():
            prefix = self._current_word_prefix()
            self._maybe_show_completer(prefix)
            # Outdent-Trigger: wurde gerade ein Outdent-Keyword komplett?
            self._maybe_outdent_line()

    def _try_expand_snippet(self) -> bool:
        """Versucht, am Cursor ein Snippet auszuloesen.

        Returns True, falls erfolgreich (-> Aufrufer macht keinen Indent).
        """
        cursor = self.textCursor()
        col = cursor.positionInBlock()
        block_text = cursor.block().text()
        # Token links vom Cursor extrahieren (lowercase Trigger).
        i = col
        while i > 0 and (block_text[i - 1].isalnum() or block_text[i - 1] == "_"):
            i -= 1
        token = block_text[i:col]
        if not token:
            return False
        body = SNIPPETS.get(token.lower())
        if body is None:
            return False
        # Whitespace-Prefix der aktuellen Zeile als Folge-Indent.
        leading = ""
        for ch in block_text:
            if ch in (" ", "\t"):
                leading += ch
            else:
                break
        expanded, cursor_offset, sel_length = expand_snippet_full(body, leading)
        # Trigger entfernen, dann Snippet einfuegen.
        cursor.beginEditBlock()
        cursor.movePosition(
            QTextCursor.MoveOperation.Left,
            QTextCursor.MoveMode.KeepAnchor,
            len(token),
        )
        start_pos = cursor.selectionStart()
        cursor.insertText(expanded)
        cursor.endEditBlock()
        # Cursor an Marker-Position setzen, Default-Text selektieren
        # (wenn Snippet einen ${1:default} hat). User kann sofort tippen
        # und ueberschreibt damit den Default.
        target = self.textCursor()
        target.setPosition(start_pos + cursor_offset)
        if sel_length > 0:
            target.setPosition(
                start_pos + cursor_offset + sel_length,
                QTextCursor.MoveMode.KeepAnchor,
            )
        self.setTextCursor(target)
        return True




    def _auto_indent_newline(self) -> None:
        cursor = self.textCursor()
        block_text = cursor.block().text()
        col = cursor.positionInBlock()
        i = 0
        while i < len(block_text) and block_text[i] in (" ", "\t"):
            i += 1
        leading = block_text[:i]
        # Smart-Indent: wenn die Zeile bis zum Cursor mit einem Block-
        # Opener endet, eine zusaetzliche Indent-Stufe einruecken.
        line_until_cursor = block_text[:col]
        if _INDENT_AFTER_PATTERNS.search(line_until_cursor.rstrip()):
            leading = leading + " " * INDENT_SPACES
        cursor.insertText("\n" + leading)

    # ----------------------------------------------- Auto-Pair

    # ----------------------------------------------- Outdent on type


    # ----------------------------------------------- Comment-Toggle

    # ----------------------------------------------- Move/Duplicate Line


    # ----------------------------------------------- Font-Zoom
    def adjust_font_size(self, delta: int) -> None:
        f = self.font()
        size = max(6, min(36, f.pointSize() + delta))
        f.setPointSize(size)
        self.setFont(f)
        # Tab-Stop muss neu berechnet werden, sonst werden Tabs schief.
        fm = self.fontMetrics()
        self.setTabStopDistance(fm.horizontalAdvance(" ") * INDENT_SPACES)
        self._line_area.update()
        self.viewport().update()

    def reset_font_size(self) -> None:
        f = self.font()
        f.setPointSize(EDITOR_FONT_SIZE)
        self.setFont(f)
        fm = self.fontMetrics()
        self.setTabStopDistance(fm.horizontalAdvance(" ") * INDENT_SPACES)
        self._line_area.update()
        self.viewport().update()


    # ----------------------------------------------- Maus
    def wheelEvent(self, event):  # noqa: N802
        # Strg + Mausrad -> Schriftgroesse anpassen. Pro Wheel-Tick eine
        # Stufe (1pt). `angleDelta().y()` liefert ein Vielfaches von 120 fuer
        # einen Standard-Tick -- wir teilen entsprechend, sodass kontinuierliche
        # High-Resolution-Wheels (Touchpad) nicht zu schnell zoomen.
        if event.modifiers() & Qt.KeyboardModifier.ControlModifier:
            delta = event.angleDelta().y()
            if delta == 0:
                return
            steps = delta // 120
            if steps == 0:
                steps = 1 if delta > 0 else -1
            self.adjust_font_size(int(steps))
            event.accept()
            return
        super().wheelEvent(event)

    def focusOutEvent(self, event):  # noqa: N802
        # Editor verliert Fokus -> Signature-Help-Popup wegblenden.
        if hasattr(self, "_sig_popup"):
            self._sig_popup.hide()
        super().focusOutEvent(event)

    def mousePressEvent(self, event):  # noqa: N802
        ctrl = event.modifiers() & Qt.KeyboardModifier.ControlModifier
        if event.button() == Qt.MouseButton.LeftButton and ctrl:
            # Ctrl+Click -> Goto-Definition.
            click_cursor = self.cursorForPosition(event.pos())
            self.setTextCursor(click_cursor)
            sym = self._symbol_under_cursor()
            if sym:
                self.goto_definition_requested.emit(sym)
            return
        # Click auf einen Color-Swatch -> Farbwaehler oeffnen.
        if event.button() == Qt.MouseButton.LeftButton and not ctrl:
            hit = self._swatch_at(event.position().toPoint())
            if hit is not None:
                self._edit_color_literal(*hit)
                return
        # Click ohne Modifier ausserhalb Multi-Selektion -> sekundaere
        # Cursor verwerfen.
        if event.button() == Qt.MouseButton.LeftButton and not ctrl:
            self._clear_secondary_cursors()
        super().mousePressEvent(event)

    # ----------------------------------------------- Hover-Tooltips
    def event(self, ev):  # noqa: D401
        if ev.type() == QEvent.Type.ToolTip:
            cursor = self.cursorForPosition(ev.pos())
            word = self._word_at_cursor(cursor)
            if word:
                # Erst Built-ins probieren -- wenn ein User-Symbol einen
                # Built-in-Namen schattet (was unueblich ist, aber moeglich),
                # wollen wir trotzdem die offizielle Doku zeigen.
                doc = get_doc(word)
                if doc is not None:
                    sig, desc = doc
                    QToolTip.showText(ev.globalPos(), f"{sig}\n\n{desc}", self)
                    return True
                # Fallback: Buffer nach SUB/FUNCTION/CLASS/...-Definition
                # absuchen und Comment-Block davor als Doc nehmen.
                user_doc = extract_user_doc(self.toPlainText(), word)
                if user_doc is not None:
                    sig, desc = user_doc
                    text = sig if not desc else f"{sig}\n\n{desc}"
                    QToolTip.showText(ev.globalPos(), text, self)
                    return True
            QToolTip.hideText()
            return True
        return super().event(ev)

    def _word_at_cursor(self, cursor: QTextCursor) -> str | None:
        block_text = cursor.block().text()
        col = cursor.positionInBlock()
        if col >= len(block_text):
            return None
        # Rueckwaerts und vorwaerts gehen, solange Identifier-Zeichen.
        a = col
        while a > 0 and (block_text[a - 1].isalnum() or block_text[a - 1] in ("_", "$")):
            a -= 1
        b = col
        while b < len(block_text) and (block_text[b].isalnum() or block_text[b] in ("_", "$")):
            b += 1
        word = block_text[a:b]
        return word or None

    # ----------------------------------------- Theme-Listener
    def _on_theme_changed(self, _name: str) -> None:
        # Highlighter-Formats neu anlegen, ExtraSelections + LineNumberArea
        # neu zeichnen lassen.
        self._highlighter._init_formats()
        self._highlighter.rehighlight()
        self._refresh_extra_selections()
        self._line_area.update()

    def refresh_completions(self) -> None:
        """Wortliste neu einlesen (z.B. nach Buffer-Aenderungen, falls
        wir spaeter lokale Identifier in den Pool aufnehmen)."""
        self._completion_model.setStringList(all_completions())
