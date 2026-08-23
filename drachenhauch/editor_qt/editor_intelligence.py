"""Editor-Intelligenz: Auto-Pair, Outdent-Trigger, Bracket-Match,
Word-Highlight, Symbol-Erkennung am Cursor.

Diese Logik haengt am Buffer-Inhalt + Cursor-Position. Als Mixin in
`CodeEditor` integriert -- erwartet die ueblichen QPlainTextEdit-Methoden
auf `self`.
"""
from __future__ import annotations

from typing import TYPE_CHECKING

from PySide6.QtGui import QTextCursor


# Auto-Pair-Tabelle: opener -> closer.
AUTO_PAIRS = {"(": ")", "[": "]", '"': '"'}

# Match-Highlights fuer Klammer-Paare.
BRACKET_PAIRS = {"(": ")", "[": "]", "{": "}"}
BRACKET_REVERSE = {v: k for k, v in BRACKET_PAIRS.items()}

# Outdent-Trigger -- Zeilen, die einen Block schliessen.
OUTDENT_EXACT = frozenset({
    "END IF", "END SUB", "END FUNCTION", "END CLASS", "END STRUCT",
    "END SELECT", "END TRY", "END ENUM", "END PROPERTY", "END WITH",
    "WEND", "ELSE", "NEXT", "CATCH",
})
OUTDENT_PREFIXES = (
    "ELSEIF ", "CATCH ", "CASE ", "NEXT ", "UNTIL ",
)


# Ein Mixin ruft Methoden seines WIRTS (`CodeEditor` -> `QPlainTextEdit`), die
# es selbst nicht hat -- zur Laufzeit richtig, fuer eine Typpruefung unsichtbar.
# Die Basis nur WAEHREND DER PRUEFUNG auf den Wirt setzen macht sie sichtbar;
# zur Laufzeit bleibt es `object`, also genau das, was vorher dastand.
if TYPE_CHECKING:
    from PySide6.QtWidgets import QPlainTextEdit
    _Wirt = QPlainTextEdit
else:
    _Wirt = object


class EditorIntelligenceMixin(_Wirt):
    """Auto-Pair + Outdent-Trigger + Bracket-Match + Word-Highlight + Symbol-Helpers."""

    # ----------------------------------------------- Auto-Pair
    def _try_auto_pair(self, ch: str) -> bool:
        """`ch` ist ein eingehendes printable-Zeichen.

        Liefert True wenn die Eingabe vom Auto-Pair-Mechanismus konsumiert
        wurde (in dem Fall darf der Default-Insert nicht mehr laufen).
        """
        cursor = self.textCursor()
        if ch in AUTO_PAIRS.values():
            doc_text = self.toPlainText()
            pos = cursor.position()
            if pos < len(doc_text) and doc_text[pos] == ch:
                if not cursor.hasSelection():
                    cursor.movePosition(QTextCursor.MoveOperation.Right)
                    self.setTextCursor(cursor)
                    return True
        if ch in AUTO_PAIRS:
            closer = AUTO_PAIRS[ch]
            if cursor.hasSelection():
                sel = cursor.selectedText()
                cursor.beginEditBlock()
                cursor.insertText(ch + sel + closer)
                cursor.endEditBlock()
                return True
            cursor.beginEditBlock()
            cursor.insertText(ch + closer)
            cursor.endEditBlock()
            cursor.movePosition(QTextCursor.MoveOperation.Left)
            self.setTextCursor(cursor)
            return True
        return False

    # ----------------------------------------------- Outdent on type
    def _maybe_outdent_line(self) -> None:
        """Pruefe nach jedem printable-Insert, ob die aktuelle Zeile zu
        einem Outdent-Trigger geworden ist, und reduziere ggf. das Indent."""
        cursor = self.textCursor()
        block_text = cursor.block().text()
        stripped = block_text.strip()
        if not stripped:
            return
        upper = stripped.upper()
        match = upper in OUTDENT_EXACT or any(
            upper.startswith(p) for p in OUTDENT_PREFIXES
        )
        if not match:
            return
        cur_indent = len(block_text) - len(block_text.lstrip(" "))
        target_indent = self._indent_of_previous_with_lower_indent(cursor.blockNumber(), cur_indent)
        if target_indent is None or target_indent >= cur_indent:
            return
        diff = cur_indent - target_indent
        line_start = cursor.block().position()
        col_in_block = cursor.positionInBlock()
        edit = QTextCursor(self.document())
        edit.setPosition(line_start)
        edit.setPosition(line_start + diff, QTextCursor.MoveMode.KeepAnchor)
        edit.beginEditBlock()
        edit.removeSelectedText()
        edit.endEditBlock()
        new_cursor = self.textCursor()
        new_cursor.setPosition(line_start + max(0, col_in_block - diff))
        self.setTextCursor(new_cursor)

    def _indent_of_previous_with_lower_indent(self, block_no: int, cur_indent: int) -> int | None:
        """Sucht rueckwaerts ab `block_no` die naechste nicht-leere Zeile,
        deren Indent kleiner ist als `cur_indent`."""
        doc = self.document()
        n = block_no - 1
        while n >= 0:
            blk = doc.findBlockByNumber(n)
            text = blk.text()
            if text.strip():
                indent = len(text) - len(text.lstrip(" "))
                if indent < cur_indent:
                    return indent
            n -= 1
        return 0

    # ----------------------------------------------- Bracket-Matching
    def _matching_bracket_positions(self) -> list[tuple[int, int]]:
        """Liefert bis zu zwei (start, end) Char-Ranges fuer das aktuelle
        Klammer-Paar -- oder eine leere Liste."""
        cursor = self.textCursor()
        if cursor.hasSelection():
            return []
        text = self.toPlainText()
        pos = cursor.position()
        for probe in (pos, pos - 1):
            if probe < 0 or probe >= len(text):
                continue
            ch = text[probe]
            if ch in BRACKET_PAIRS:
                target = self._find_matching_bracket(text, probe, +1)
                if target is not None:
                    return [(probe, probe + 1), (target, target + 1)]
            if ch in BRACKET_REVERSE:
                target = self._find_matching_bracket(text, probe, -1)
                if target is not None:
                    return [(probe, probe + 1), (target, target + 1)]
        return []

    @staticmethod
    def _find_matching_bracket(text: str, start: int, direction: int) -> int | None:
        if start < 0 or start >= len(text):
            return None
        opener = text[start]
        if direction > 0:
            closer = BRACKET_PAIRS.get(opener)
        else:
            # `opener` wird hier absichtlich zu `str | None` -- die Pruefung
            # direkt darunter faengt das ab.
            opener, closer = BRACKET_REVERSE.get(opener), opener  # type: ignore[assignment]
        if closer is None or opener is None:
            return None
        depth = 0
        in_string = False
        i = start
        while 0 <= i < len(text):
            c = text[i]
            if c == '"' and (i == 0 or text[i - 1] != "\\"):
                in_string = not in_string
            elif not in_string:
                if c == opener:
                    depth += 1
                elif c == closer:
                    depth -= 1
                    if depth == 0:
                        return i
            i += direction
        return None

    # ----------------------------------------------- Word-Highlight
    def _word_highlight_ranges(self) -> list[tuple[int, int]]:
        """Liefert die Char-Ranges aller weiteren Vorkommen der aktuell
        selektierten Identifier-Sequenz."""
        cursor = self.textCursor()
        if not cursor.hasSelection():
            return []
        sel_text = cursor.selectedText()
        if " " in sel_text or len(sel_text) < 2 or len(sel_text) > 80:
            return []
        if not all(ch.isalnum() or ch in ("_", "$") for ch in sel_text):
            return []
        text = self.toPlainText()
        # Review-Fund: suchte bisher im rohen Dokumenttext, ohne Kommentare/
        # Strings auszusparen -- ein Doppelklick auf einen Bezeichner
        # highlightete so auch dessen Vorkommen INNERHALB eines '/REM-
        # Kommentars oder String-Literals, obwohl das eigentliche Find-
        # References-Feature (symbols.scan_references) genau das korrekt
        # ausspart. `masked` hat dieselbe Laenge/Spalten-Positionen wie
        # `text` (Kommentare/Strings durch Leerzeichen ersetzt) -- ein
        # echter Identifier kann nie aus Leerzeichen bestehen, daher findet
        # die Suche darin automatisch nur noch echte Code-Vorkommen.
        from . import symbols as _sym
        masked = "\n".join(_sym._strip_comment_and_strings(ln) for ln in text.split("\n"))
        out: list[tuple[int, int]] = []
        ssel = cursor.selectionStart()
        esel = cursor.selectionEnd()
        i = 0
        n = len(sel_text)
        while True:
            idx = masked.find(sel_text, i)
            if idx < 0:
                break
            i = idx + 1
            before = text[idx - 1] if idx > 0 else ""
            after = text[idx + n] if idx + n < len(text) else ""
            if before.isalnum() or before in ("_", "$"):
                continue
            if after.isalnum() or after in ("_", "$"):
                continue
            if idx == ssel and idx + n == esel:
                continue
            out.append((idx, idx + n))
            if len(out) >= 200:
                break
        return out

    # ----------------------------------------------- Symbol unter Cursor
    def _symbol_under_cursor(self) -> str | None:
        """Liefert den Identifier-Namen, ueber dem der Cursor steht."""
        cursor = self.textCursor()
        text = cursor.block().text()
        col = cursor.positionInBlock()
        if col < 0 or col > len(text):
            return None
        a = col
        while a > 0 and (text[a - 1].isalnum() or text[a - 1] in ("_", "$")):
            a -= 1
        b = col
        while b < len(text) and (text[b].isalnum() or text[b] in ("_", "$")):
            b += 1
        word = text[a:b].rstrip("$")
        return word or None
