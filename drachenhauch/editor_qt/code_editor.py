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
    QAbstractItemView, QCompleter, QInputDialog, QPlainTextEdit, QTextEdit,
    QToolTip, QWidget,
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
from .highlighter import DHHighlighter
from .snippets import SNIPPETS, expand_snippet, expand_snippet_full
from . import symbols as dh_symbols
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


_MISSING = object()   # Sentinel fuer "kein Eintrag" (Wert None ist gueltig)


class _LineTracker:
    """Verfolgt eine Menge zeilengebundener Marker (Bookmarks, Breakpoints
    + ihre Bedingungen, gefaltete Bloecke) robust gegen Zeilen-Verschiebung
    durch Edits.

    Ein rohes `set[int]`/`dict[int, T]` zeigt nach einem Edit OBERHALB des
    Markers auf die falsche Zeile -- die Zeilennummer selbst war nie mehr
    als eine Momentaufnahme. Dieser Tracker haelt pro Marker stattdessen
    einen `QTextCursor`: Qt aktualisiert dessen Position automatisch bei
    jedem Dokument-Edit (Einfuegen/Loeschen von Zeilen), die aktuelle Zeile
    ergibt sich jederzeit aus `cursor.blockNumber() + 1`. Die oeffentliche
    API bleibt zeilenbasiert (1-indiziert), damit Aufrufer unveraendert
    bleiben -- nur die interne Anker-Strategie aendert sich.

    Deckt NICHT den Fall ab, dass eine Operation die Zeile selbst per
    Delete+Insert ersetzt (z.B. `move_lines()` beim Zeilen-Verschieben) --
    dort kollabiert Qt einen Cursor auf der geloeschten Zeile an den
    Einfuegepunkt statt ihn "mitzuziehen". Solche Operationen migrieren
    Marker explizit ueber `remap()` (siehe `move_lines`)."""

    def __init__(self, document) -> None:
        self._document = document
        self._entries: list[tuple[QTextCursor, object]] = []

    def retarget(self, document) -> None:
        """Bindet den Tracker an ein NEUES Dokument um -- fuer
        `CodeEditor.setDocument()` (der Split-View-Editor teilt sich das
        Dokument des Primaer-Editors NACH seinem eigenen `__init__`, siehe
        `tabs.toggle_split()`). Review-Fund: ohne dieses Retarget zeigte der
        Tracker weiter auf das urspruengliche (jetzt verworfene) Dokument --
        `set()`s `findBlockByNumber()` griff dann ins Leere, Breakpoint-/
        Bookmark-/Fold-Klicks im Split-View-Gutter wurden zu stillen No-Ops.
        Bestehende Marker gehoeren zum ALTEN Dokument und werden verworfen
        (ihre QTextCursor-Positionen waeren nach dem Wechsel bedeutungslos)."""
        self._document = document
        self._entries.clear()

    @staticmethod
    def _line_of(cursor: QTextCursor) -> int:
        return cursor.blockNumber() + 1

    def _find_index(self, line: int) -> int | None:
        for i, (cur, _val) in enumerate(self._entries):
            if self._line_of(cur) == line:
                return i
        return None

    def __contains__(self, line: int) -> bool:
        return self._find_index(line) is not None

    def __bool__(self) -> bool:
        return bool(self._entries)

    def __len__(self) -> int:
        return len(self._entries)

    def get(self, line: int, default=None):
        i = self._find_index(line)
        return self._entries[i][1] if i is not None else default

    def set(self, line: int, value=None) -> None:
        """Marker auf `line` setzen (oder seinen Wert aktualisieren, falls
        dort schon einer existiert)."""
        block = self._document.findBlockByNumber(line - 1)
        if not block.isValid():
            return
        cursor = QTextCursor(block)
        i = self._find_index(line)
        if i is not None:
            self._entries[i] = (cursor, value)
        else:
            self._entries.append((cursor, value))

    def discard(self, line: int) -> None:
        i = self._find_index(line)
        if i is not None:
            del self._entries[i]

    def lines(self) -> list[int]:
        return sorted(self._line_of(cur) for cur, _val in self._entries)

    def items(self) -> list[tuple[int, object]]:
        return [(self._line_of(cur), val) for cur, val in self._entries]

    def clear(self) -> None:
        self._entries.clear()

    def remap(self, mapping: dict[int, int]) -> None:
        """Verschiebt Marker gemaess `mapping` (alte Zeile -> neue Zeile,
        1-basiert) -- fuer Operationen wie `move_lines()`, die eine Zeile
        per Delete+Insert an eine andere Stelle verschieben, wo die
        automatische Cursor-Verfolgung NICHT greift (siehe Klassen-Doc)."""
        for old_line, new_line in mapping.items():
            value = self.get(old_line, _MISSING)
            if value is _MISSING:
                continue
            self.discard(old_line)
            self.set(new_line, None if value is None else value)


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

        self._highlighter = DHHighlighter(self.document())

        self._line_area = _LineNumberArea(self)
        self.blockCountChanged.connect(self._update_viewport_margins)
        self.updateRequest.connect(self._on_update_request)
        self.cursorPositionChanged.connect(self._highlight_current_line)

        # Find-Hits als ExtraSelections-Layer (separat von Current-Line).
        self._find_hits: list[tuple[int, int]] = []

        # Color-Literale (&HRRGGBB / RGB(r,g,b)) -- als ExtraSelections gerendert
        # (Hintergrund = die Farbe, Schrift im Kontrast). Cache wird neu aufgebaut,
        # gerendert wird ueber die Text-Engine, damit nichts flackert/verschwindet
        # (kein Overlay-Clipping). Debounced wie der Fold-Scan (150ms) -- vorher
        # lief der volle Dokument-Scan synchron bei JEDEM textChanged, was bei
        # grossen Dateien beim Tippen spuerbare Latenz verursachen konnte.
        self._color_literals: list[tuple[int, int, QColor, str]] = []
        self._color_scan_timer = QTimer(self)
        self._color_scan_timer.setSingleShot(True)
        self._color_scan_timer.setInterval(150)
        self._color_scan_timer.timeout.connect(self._rescan_color_literals)
        self.textChanged.connect(self._color_scan_timer.start)

        # Debugger: Breakpoint-Zeilen (1-basiert) + aktuelle Stop-Zeile.
        # Wert je Zeile = Bedingung (Drachenhauch-Ausdruck) oder None fuer
        # einen unbedingten Breakpoint -- EIN Tracker statt zweier separater
        # Container, damit Zeile+Bedingung nie auseinanderlaufen koennen.
        self._breakpoints = _LineTracker(self.document())
        self._debug_line: int | None = None

        # Bookmarks (1-basierte Zeilen) -- Schnell-Navigation in langen Dateien.
        self._bookmarks = _LineTracker(self.document())

        # Multi-Cursor (Strg+D Add-Next-Occurrence). Liste sekundaerer
        # Selektionen als (start, end). Primaere Cursor bleibt im
        # `textCursor()`. Bei jeder Tasten-Eingabe werden alle Sekundaer-
        # Selektionen mit-gepresst.
        self._secondary: list[tuple[int, int]] = []

        # Live-Error-Check: laeuft async via QThread, debounced.
        self._error_problem: ParseProblem | None = None   # erstes Problem (Gutter)
        self._error_problems: list = []                   # alle (Underlines/Panel)
        self._error_checker = LiveErrorChecker(self)
        self._error_checker.problems_changed.connect(self._on_error_problems)
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
        # Wert je Start-Zeile = die END-Zeile als eigener Cursor (nicht nur
        # der rohe int) -- so bleibt bekannt, welcher exakte Block-Bereich
        # verborgen ist, selbst wenn die Fold-Region beim naechsten Rescan
        # nicht mehr an derselben Stelle gefunden wird (siehe
        # _rescan_fold_regions).
        self._folded = _LineTracker(self.document())
        self._fold_scan_timer = QTimer(self)
        self._fold_scan_timer.setSingleShot(True)
        self._fold_scan_timer.setInterval(150)
        self._fold_scan_timer.timeout.connect(self._rescan_fold_regions)
        self.textChanged.connect(self._fold_scan_timer.start)

        self._setup_completer()
        self._auto_complete_enabled = True

        # Signature-Help (Parameter-Hints beim Tippen eines Aufrufs).
        # `cursorPositionChanged` feuert bei JEDEM Tastendruck (nicht nur
        # Navigation); der Fallback fuer User-Funktionen (`_user_signature`)
        # scannt bei Bedarf das GANZE Dokument neu -- debounced wie
        # Color-Literal-/Fold-Scan (150ms), sonst gleiche Latenz-Gefahr beim
        # Tippen in grossen Dateien (Review-Fund).
        from .signature_help import SignaturePopup
        self._sig_popup = SignaturePopup(self)
        self._sig_help_timer = QTimer(self)
        self._sig_help_timer.setSingleShot(True)
        self._sig_help_timer.setInterval(80)
        self._sig_help_timer.timeout.connect(self._update_signature_help)
        self.cursorPositionChanged.connect(self._sig_help_timer.start)
        self.verticalScrollBar().valueChanged.connect(
            lambda _v: self._sig_popup.hide())

        theme_signals.changed.connect(self._on_theme_changed)

        self._update_viewport_margins(0)
        self._rescan_color_literals()
        self._highlight_current_line()
        self._rescan_fold_regions()

    def setDocument(self, document) -> None:  # noqa: N802 (Qt-API)
        """Review-Fund: `tabs.toggle_split()` erzeugt einen zweiten
        `CodeEditor()` und ruft danach `setDocument()` auf, um sich das
        Dokument des Primaer-Editors zu teilen -- die drei `_LineTracker`
        (Breakpoints/Bookmarks/Folds, im `__init__` gegen das TEMPORAERE
        Default-Dokument angelegt) zeigten ohne dieses Override weiter auf
        das verworfene Dokument. Breakpoint-/Bookmark-/Fold-Klicks im
        Split-View-Gutter wurden dadurch zu stillen No-Ops (`findBlockByNumber`
        gegen ein fast leeres Dokument liefert einen ungueltigen Block, kein
        Fehler). `getattr(..., None)` schuetzt gegen den Fall, dass Qt
        `setDocument()` schon waehrend `super().__init__()` aufruft, bevor
        diese Attribute existieren."""
        super().setDocument(document)
        for tracker in (getattr(self, "_breakpoints", None),
                        getattr(self, "_bookmarks", None),
                        getattr(self, "_folded", None)):
            if tracker is not None:
                tracker.retarget(document)
        # Der `DHHighlighter` aus `__init__` haengt als Kind am ALTEN
        # (Default-)Dokument -- `super().setDocument()` gibt das her und
        # zerstoert es, der Highlighter stirbt also mit. `self._highlighter`
        # zeigte danach auf ein totes C++-Objekt, und der naechste
        # Theme-Wechsel lief in `_on_theme_changed()` in ein
        # "RuntimeError: Internal C++ object (DHHighlighter) already deleted"
        # (real reproduzierbar: Split-View oeffnen, dann Theme umstellen --
        # `tabs.toggle_split()` ist der einzige Aufrufer dieses Overrides).
        # Der Split-View WILL keinen eigenen Highlighter (die Formate kommen
        # vom Primaer-Editor ueber die geteilten Block-Formate), deshalb wird
        # die tote Referenz nur geloescht, nicht neu aufgebaut.
        import shiboken6
        hl = getattr(self, "_highlighter", None)
        if hl is not None and not shiboken6.isValid(hl):
            self._highlighter = None

    # ------------------------------------------------------------ API
    def get_text(self) -> str:
        return self.toPlainText()

    def set_text(self, content: str) -> None:
        self.setPlainText(content)
        self.document().setModified(False)
        self._find_hits = []
        # Programmatischer Volltext-Ersatz (Datei laden/Recovery/...) --
        # sofort rescannen statt den Tipp-Debounce abzuwarten. Ruft intern
        # bereits _refresh_extra_selections() auf.
        self._color_scan_timer.stop()
        self._rescan_color_literals()

    def replace_text_undoable(self, content: str) -> None:
        """Wie `set_text()`, aber als EIN normaler (undo-faehiger) Edit --
        `setPlainText()` in `set_text()` loescht die komplette Undo-Historie
        (Qt-Doku), was fuer echtes Neu-Laden (Datei oeffnen/Recovery) richtig
        ist, aber fuer programmatische Edits INNERHALB einer offenen Datei
        (Rename Symbol/Format Document/Projekt-Ersetzen/Format-on-Save)
        ueberraschend ist -- der User erwartet dort Strg+Z. Cursor-basierter
        Ersatz in einem `beginEditBlock`/`endEditBlock` loescht die Historie
        nicht, sondern haengt EINEN Undo-Schritt an (Muster wie
        `editor_actions.py`s Kommentar-Toggle/Move-Lines)."""
        cursor = self.textCursor()
        cursor.beginEditBlock()
        try:
            cursor.select(QTextCursor.SelectionType.Document)
            cursor.insertText(content)
        finally:
            cursor.endEditBlock()

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
            self._popup().hide()

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
                    if self._breakpoints.get(line1b) is not None:
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
                # String-/Kommentar-Inhalt durch gleich lange Leerzeichen
                # ersetzen (Spalten-Positionen bleiben stabil) -- sonst wird
                # ein Color-Literal INNERHALB eines Kommentars/String-
                # Literals faelschlich als editierbarer Swatch erkannt und
                # ein Klick darauf ueberschreibt Text, der gar keine echte
                # Farbe ist (Review-Fund).
                scan_text = dh_symbols._strip_comment_and_strings(text)
                base = block.position()
                for s, e, color, kind in self._scan_color_swatches(scan_text):
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
        (Viewport-Koordinate) oder None. Klick-Bereich = das Literal selbst.

        Der Cache (`_color_literals`) ist debounced (150ms) -- vor dem
        eigentlichen Hit-Test wird IMMER frisch neu gescannt, sonst koennte
        ein Klick kurz nach schnellem Tippen (ohne 150ms Pause seit dem
        letzten Zeichen) einen veralteten Offset treffen und beim Ersetzen
        den FALSCHEN Text ueberschreiben (Review-Fund: Stale-Cache-Race)."""
        self._color_scan_timer.stop()
        self._rescan_color_literals()
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
            # Die Laufzeit liest Alpha=0 als "deckend" (Rueckwaerts-Kompat.
            # zu 24-bit-Farben, siehe RGB()/RGBA()-Builtins in builtins.rs)
            # -- ohne diesen Bump wuerde "voll transparent" im Picker
            # (Alpha-Regler auf 0) als das GENAUE GEGENTEIL geschrieben
            # (deckend) statt als das gemeinte Alpha=1 (Review-Fund).
            al_write = 1 if al == 0 else al
            repl = f"&H{al_write:02X}{r:02X}{g:02X}{b:02X}"
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

    def _on_error_problems(self, problems) -> None:
        self._error_problems = list(problems or [])
        # Erstes Problem fuer Gutter-Marker + Statusbar; Errors haben Vorrang
        # vor Warnungen, damit die wichtigste Meldung gewinnt.
        self._error_problem = None
        for p in self._error_problems:
            if self._error_problem is None or (
                    self._error_problem.severity != "error"
                    and p.severity == "error"):
                self._error_problem = p
        self._refresh_extra_selections()
        self._line_area.update()

    def current_error(self) -> ParseProblem | None:
        return self._error_problem

    def current_problems(self) -> list:
        return self._error_problems

    # ----------------------------------------------- Code-Folding
    def _rescan_fold_regions(self) -> None:
        self._fold_regions = scan_fold_regions(self.toPlainText())
        valid_starts = {s for s, _e, _k in self._fold_regions}
        for start_line in list(self._folded.lines()):
            if start_line in valid_starts:
                continue
            # Die Region an dieser (aktuellen, Cursor-getrackten) Start-
            # Zeile ist nach dem Rescan keine gueltige Fold-Region mehr
            # (Struktur geaendert/entfernt) -- NUR ihren tatsaechlich
            # verborgenen Bereich wieder sichtbar machen (Ende steht im
            # Tracker-Wert als eigener Cursor, unabhaengig vom neuen Scan).
            # Fruehere Notbremse "alles aufklappen" war noetig, weil rohe
            # int-Zeilen bei Edits nicht mehr zuverlaessig auf den
            # richtigen Block zeigten -- mit Cursor-Ankern nicht mehr.
            end_cursor = self._folded.get(start_line)
            end_line = end_cursor.blockNumber() + 1 if end_cursor is not None else start_line
            for ln in range(start_line + 1, end_line + 1):
                blk = self.document().findBlockByNumber(ln - 1)
                if blk.isValid():
                    blk.setVisible(True)
            self._folded.discard(start_line)
            self.document().markContentsDirty(0, self.document().characterCount())
            self.viewport().update()
        self._line_area.update()

    def _unfold_all_blocks(self) -> None:
        """Stellt sicher, dass jeder Block sichtbar ist."""
        for ln in range(1, self.document().blockCount() + 1):
            blk = self.document().findBlockByNumber(ln - 1)
            if blk.isValid() and not blk.isVisible():
                blk.setVisible(True)
        self.document().markContentsDirty(0, self.document().characterCount())
        self.viewport().update()

    def _block_at_y(self, y_pixel: int):
        """Liefert den sichtbaren Block, dessen vertikale Spanne `y_pixel`
        (Widget-lokale Pixel-Y-Koordinate) enthaelt, oder None.

        Review-Fund: `_handle_fold_click`/`_line_at_y` duplizierten beide
        exakt dieselbe Block-Lauf-Schleife (nur die Verwendung des
        gefundenen Blocks unterschied sich) -- jetzt eine gemeinsame Basis."""
        block = self.firstVisibleBlock()
        offset = self.contentOffset()
        top = self.blockBoundingGeometry(block).translated(offset).top()
        bottom = top + self.blockBoundingRect(block).height()
        while block.isValid():
            if block.isVisible() and top <= y_pixel < bottom:
                return block
            block = block.next()
            top = bottom
            bottom = top + self.blockBoundingRect(block).height()
        return None

    def _handle_fold_click(self, y_pixel: int) -> bool:
        """Identifiziert den Block bei der Y-Pixel-Position und togglet ihn,
        sofern er Block-Anfang einer Fold-Region ist."""
        block = self._block_at_y(y_pixel)
        if block is None:
            return False
        line = block.blockNumber() + 1
        for s, e, _k in self._fold_regions:
            if s == line:
                self._toggle_fold(s, e)
                return True
        return False

    def _line_at_y(self, y_pixel: int) -> int | None:
        """1-basierte Zeile an einer Y-Pixel-Position im Gutter (oder None)."""
        block = self._block_at_y(y_pixel)
        return block.blockNumber() + 1 if block is not None else None

    def _shift_line_markers(self, remap: dict[int, int]) -> None:
        """Migriert Bookmarks/Breakpoints/Folds gemaess `remap` (alte Zeile
        -> neue Zeile, 1-basiert). Gebraucht von `move_lines()`
        (`editor_actions.py`) -- siehe dort fuer die Begruendung, warum
        Cursor-Auto-Tracking dort nicht ausreicht."""
        for tracker in (self._breakpoints, self._bookmarks, self._folded):
            tracker.remap(remap)

    # ----------------------------------------------- Breakpoints / Debug
    def toggle_breakpoint(self, line: int) -> None:
        if line in self._breakpoints:
            self._breakpoints.discard(line)
        else:
            self._breakpoints.set(line, None)
        self._line_area.update()
        self.breakpoints_changed.emit()

    def edit_breakpoint_condition(self, line: int) -> None:
        """Fragt eine Bedingung (Drachenhauch-Ausdruck) fuer den Breakpoint auf
        `line` ab. Setzt automatisch einen Breakpoint, falls noch keiner da
        ist. Leere Eingabe -> unbedingter Breakpoint."""
        cur = self._breakpoints.get(line, None) or ""
        expr, ok = QInputDialog.getText(
            self, f"Breakpoint-Bedingung (Zeile {line})",
            "Anhalten, wenn Ausdruck wahr ist (leer = immer):", text=cur)
        if not ok:
            return
        expr = expr.strip()
        self._breakpoints.set(line, expr or None)
        self._line_area.update()
        self.breakpoints_changed.emit()

    def breakpoint_conditions(self) -> dict[int, str]:
        return {ln: cond for ln, cond in self._breakpoints.items() if cond}

    # ----------------------------------------------- Bookmarks
    def toggle_bookmark(self) -> None:
        line = self.textCursor().blockNumber() + 1
        if line in self._bookmarks:
            self._bookmarks.discard(line)
        else:
            self._bookmarks.set(line)
        self._line_area.update()

    def _goto_bookmark(self, forward: bool) -> None:
        if not self._bookmarks:
            return
        cur = self.textCursor().blockNumber() + 1
        marks = self._bookmarks.lines()
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
        return set(self._breakpoints.lines())

    def set_breakpoints(self, lines) -> None:
        """Ersetzt die komplette Breakpoint-Menge (z.B. nach externem
        State-Load). Bedingungen bereits vorhandener Zeilen bleiben
        erhalten, wenn diese Zeile auch in `lines` weiterhin vorkommt."""
        old_conditions = dict(self._breakpoints.items())
        self._breakpoints.clear()
        for ln in (int(x) for x in lines):
            self._breakpoints.set(ln, old_conditions.get(ln))
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
        entry_end = self._folded.get(start_line)
        folded_now = entry_end is not None
        new_visible = folded_now    # bei "war gefaltet" -> jetzt wieder sichtbar
        # Beim Auffalten den TATSAECHLICH verborgenen Bereich nutzen (der
        # Cursor-Wert im Tracker), nicht das uebergebene `end_line` -- die
        # koennten seit dem Falten auseinandergelaufen sein.
        actual_end = entry_end.blockNumber() + 1 if entry_end is not None else end_line
        for ln in range(start_line + 1, actual_end + 1):
            blk = self.document().findBlockByNumber(ln - 1)
            if not blk.isValid():
                continue
            blk.setVisible(new_visible)
        if folded_now:
            self._folded.discard(start_line)
        else:
            end_block = self.document().findBlockByNumber(end_line - 1)
            self._folded.set(
                start_line,
                QTextCursor(end_block if end_block.isValid() else self.document().lastBlock()),
            )
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
        return self._folded.lines()

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
        # Live-Diagnostik-Underlines: Wellenlinie je Problem-Zeile. Errors rot,
        # Warnungen gelb. Pro Zeile nur einmal (die hoechste Severity gewinnt),
        # damit doppelte Selektionen auf einer Zeile nicht flackern.
        from PySide6.QtGui import QTextCharFormat
        seen_lines: dict[int, str] = {}
        for p in self._error_problems:
            ln = max(1, p.line)
            prev = seen_lines.get(ln)
            if prev == "error":
                continue   # Error auf dieser Zeile schlaegt Warning
            seen_lines[ln] = p.severity if p.severity == "error" else (prev or "warning")
        for ln, sev in seen_lines.items():
            blk = self.document().findBlockByNumber(ln - 1)
            if not blk.isValid():
                continue
            sel = QTextEdit.ExtraSelection()
            fmt = sel.format
            fmt.setUnderlineStyle(QTextCharFormat.UnderlineStyle.WaveUnderline)
            fmt.setUnderlineColor(QColor(COLORS["error"] if sev == "error"
                                         else COLORS["warning"]))
            fmt.setProperty(QTextFormat.Property.FullWidthSelection, True)
            c = QTextCursor(blk)
            c.movePosition(QTextCursor.MoveOperation.EndOfBlock,
                           QTextCursor.MoveMode.KeepAnchor)
            sel.cursor = c
            selections.append(sel)
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
    def _popup(self) -> QAbstractItemView:
        """Das Completer-Popup.

        `QCompleter.popup()` ist mit `| None` angegeben, legt beim ersten
        Aufruf aber selbst eine Liste an -- None kaeme nur ohne Completer vor,
        und den setzt `_setup_completer` im Konstruktor. Ein Helfer statt
        sechsmal derselbe Umweg.
        """
        popup = self._completer.popup()
        assert popup is not None
        return popup

    def _setup_completer(self) -> None:
        # Statische Vorschlaege (Keywords/Builtins/Konstanten/Snippets) einmal
        # cachen; lokale Buffer-Symbole kommen on-demand dazu (revisions-gecacht).
        self._static_completions = all_completions()
        self._static_completions_lower = {s.lower() for s in self._static_completions}
        self._completion_pool_rev = -1
        self._completion_model = QStringListModel(list(self._static_completions), self)
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
            self._popup().hide()
            return
        self._show_completer(prefix)

    def _rebuild_completion_pool(self) -> None:
        """Wortliste = statische Vorschlaege + lokale Symbole (SUB/FUNCTION/
        DIM/CLASS/...) des aktuellen Buffers. Vor dem Anzeigen aufgerufen, so
        sind selbst frisch getippte Definitionen sofort vorschlagbar. Per
        Dokument-Revision gecacht -- bei unveraendertem Text kostenfrei."""
        rev = self.document().revision()
        if rev == self._completion_pool_rev:
            return
        self._completion_pool_rev = rev
        from .completer import local_definition_names
        locals_ = [n for n in local_definition_names(self.toPlainText())
                   if n.lower() not in self._static_completions_lower]
        if locals_:
            merged = sorted(set(self._static_completions) | set(locals_),
                            key=lambda s: (s.lower(), s))
            self._completion_model.setStringList(merged)
        else:
            self._completion_model.setStringList(list(self._static_completions))

    def _show_completer(self, prefix: str) -> None:
        self._rebuild_completion_pool()
        self._completer.setCompletionPrefix(prefix)
        popup = self._popup()
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
        if self._popup().isVisible():
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
        from .dhrt_meta import signature
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
            self._sig_help_timer.stop()
            self._sig_popup.hide()
        # Wenn das Completer-Popup sichtbar ist: Enter/Tab -> Auswahl uebernehmen,
        # Esc -> ausblenden, Up/Down -> Popup-Navigation (default-Forward).
        if self._popup().isVisible():
            if event.key() in (
                Qt.Key.Key_Enter, Qt.Key.Key_Return, Qt.Key.Key_Tab,
                Qt.Key.Key_Up, Qt.Key.Key_Down,
            ):
                event.ignore()
                return
            if event.key() == Qt.Key.Key_Escape:
                self._popup().hide()
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
            # Review-Fund: Tab/Enter sind KEINE "printable"-Zeichen
            # (`"\t".isprintable()`/`"\r".isprintable()` sind beide False),
            # fielen also durch diesen Zweig hindurch und landeten im
            # "sonstige Taste -> Multi-Selektion verwerfen"-Fallback unten --
            # ein User mit aktiver Multi-Selektion, der Tab zum Einruecken
            # oder Enter fuer eine neue Zeile drueckte, verlor seine
            # Selektion STILLSCHWEIGEND und nur der primaere Cursor wurde
            # bearbeitet. Snippet-Expansion (Tab im Single-Cursor-Pfad)
            # bleibt bewusst Single-Cursor-only -- mehrdeutig bei mehreren
            # Cursorn -- Multi-Cursor-Tab ruecktb daher immer nur ein.
            if key == Qt.Key.Key_Tab and not shift:
                self._multi_edit(lambda c: self._insert_indent(c))
                return
            if key == Qt.Key.Key_Backtab or (key == Qt.Key.Key_Tab and shift):
                self._multi_edit(lambda c: self._remove_indent(c))
                return
            if key in (Qt.Key.Key_Return, Qt.Key.Key_Enter):
                self._multi_edit(lambda c: self._auto_indent_newline(c))
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




    def _auto_indent_newline(self, cursor: QTextCursor | None = None) -> None:
        if cursor is None:
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

    def set_font_point_size(self, size: int) -> None:
        """Schriftgroesse absolut setzen (6..36). Quelle: Einstellungen-Dialog,
        damit die Groesse persistierbar ist (Strg+Mausrad bleibt transient)."""
        size = max(6, min(36, int(size)))
        f = self.font()
        if f.pointSize() == size:
            return
        f.setPointSize(size)
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
            self._sig_help_timer.stop()
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
                # Letzte Stufe: nur die Signatur. `get_doc` kennt inzwischen
                # zwei Quellen (die handgepflegte Tabelle und die aus `docs/`
                # erzeugte `builtin_prosa.json`) und deckt damit rund die
                # Haelfte des Befehlssatzes ab -- fuer den Rest ist die
                # Signatur immer noch besser als eine tote Tooltip.
                sig = self._builtin_signature(word)
                if sig is not None:
                    QToolTip.showText(ev.globalPos(), sig, self)
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
        # neu zeichnen lassen. Ein Split-View-Editor hat keinen eigenen
        # Highlighter (siehe `setDocument()`) -- er zeigt die Formate des
        # Primaer-Editors, der sie beim selben Signal ohnehin neu setzt.
        if self._highlighter is not None:
            self._highlighter._init_formats()
            self._highlighter.rehighlight()
        self._refresh_extra_selections()
        self._line_area.update()

    def refresh_completions(self) -> None:
        """Statische Vorschlaege neu einlesen (z.B. nach einem Theme-/Builtin-
        Reload) und den lokalen Symbol-Pool neu aufbauen. Lokale Symbole werden
        sonst ohnehin lazy vor jedem Anzeigen ergaenzt (`_rebuild_completion_pool`)."""
        self._static_completions = all_completions()
        self._static_completions_lower = {s.lower() for s in self._static_completions}
        self._completion_pool_rev = -1
        self._rebuild_completion_pool()
