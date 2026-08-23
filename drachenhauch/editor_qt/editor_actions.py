"""Editor-Aktionen: Indent / Outdent / Comment-Toggle / Move-Line /
Duplicate-Line.

Als Mixin -- die Methoden werden in `CodeEditor` ueber Multiple Inheritance
eingefuegt. Sie erwarten:
- `self.textCursor() -> QTextCursor`
- `self.document() -> QTextDocument`
- `self.setTextCursor(QTextCursor)`
- Konstante `INDENT_SPACES`

Pure-Logik-Tests fuer die zugrunde liegenden Algorithmen (Comment-Erkennung
etc.) gibt's nicht direkt -- hier dominiert die Qt-Cursor-Manipulation. Die
Funktionen sind aber alle nicht-zerstoerend bei leerem Buffer.
"""
from __future__ import annotations

from typing import TYPE_CHECKING

from PySide6.QtGui import QTextCursor


INDENT_SPACES = 4


# Ein Mixin ruft Methoden seines WIRTS (`CodeEditor` -> `QPlainTextEdit`), die
# es selbst nicht hat -- zur Laufzeit richtig, fuer eine Typpruefung unsichtbar.
# Die Basis nur WAEHREND DER PRUEFUNG auf den Wirt setzen macht sie sichtbar;
# zur Laufzeit bleibt es `object`, also genau das, was vorher dastand.
if TYPE_CHECKING:
    from PySide6.QtWidgets import QPlainTextEdit
    _Wirt = QPlainTextEdit
else:
    _Wirt = object


class EditorActionsMixin(_Wirt):
    """Indent + Comment-Toggle + Move-Line + Duplicate-Line."""

    if TYPE_CHECKING:
        # Liegt im Hauptmodul (`code_editor.py`), nicht im Wirt-Widget.
        def _shift_line_markers(self, remap: dict) -> None: ...

    # ----------------------------------------------- Indent / Outdent
    def _insert_indent(self, cursor: QTextCursor | None = None) -> None:
        if cursor is None:
            cursor = self.textCursor()
        if cursor.hasSelection():
            self._indent_selection(cursor, outdent=False)
            return
        col = cursor.positionInBlock()
        spaces = INDENT_SPACES - (col % INDENT_SPACES)
        if spaces == 0:
            spaces = INDENT_SPACES
        cursor.insertText(" " * spaces)

    def _remove_indent(self, cursor: QTextCursor | None = None) -> None:
        if cursor is None:
            cursor = self.textCursor()
        if cursor.hasSelection():
            self._indent_selection(cursor, outdent=True)
            return
        block_text = cursor.block().text()
        col = cursor.positionInBlock()
        n = 0
        while n < INDENT_SPACES and col - n - 1 >= 0 and block_text[col - n - 1] == " ":
            n += 1
        if n == 0:
            return
        cursor.beginEditBlock()
        for _ in range(n):
            cursor.deletePreviousChar()
        cursor.endEditBlock()

    def _indent_selection(self, cursor: QTextCursor, *, outdent: bool) -> None:
        doc = self.document()
        start = cursor.selectionStart()
        end = cursor.selectionEnd()
        first = doc.findBlock(start).blockNumber()
        last = doc.findBlock(end).blockNumber()
        if doc.findBlock(end).position() == end and first != last:
            last -= 1
        cursor.beginEditBlock()
        for line in range(first, last + 1):
            block = doc.findBlockByNumber(line)
            line_cursor = QTextCursor(block)
            if outdent:
                text = block.text()
                rm = 0
                while rm < INDENT_SPACES and rm < len(text) and text[rm] == " ":
                    rm += 1
                if rm:
                    for _ in range(rm):
                        line_cursor.deleteChar()
            else:
                line_cursor.insertText(" " * INDENT_SPACES)
        cursor.endEditBlock()

    # ----------------------------------------------- Comment-Toggle
    def comment_toggle(self) -> None:
        """Toggle `' `-Prefix auf den selektierten Zeilen (oder Cursor-Zeile).

        Wenn ALLE betroffenen nicht-leeren Zeilen bereits ein `'` als
        erstes Nicht-Whitespace-Zeichen haben, wird kommentiert entfernt;
        sonst wird vor jede Zeile ein `' ` eingefuegt am gemeinsamen Indent.
        """
        cursor = self.textCursor()
        doc = self.document()
        if cursor.hasSelection():
            first = doc.findBlock(cursor.selectionStart()).blockNumber()
            last = doc.findBlock(cursor.selectionEnd()).blockNumber()
            if doc.findBlock(cursor.selectionEnd()).position() == cursor.selectionEnd() and first != last:
                last -= 1
        else:
            first = last = cursor.blockNumber()

        all_commented = True
        any_nonempty = False
        for ln in range(first, last + 1):
            t = doc.findBlockByNumber(ln).text()
            stripped = t.lstrip(" ")
            if not stripped:
                continue
            any_nonempty = True
            if not stripped.startswith("'"):
                all_commented = False
                break
        if not any_nonempty:
            return

        edit = self.textCursor()
        edit.beginEditBlock()
        try:
            for ln in range(first, last + 1):
                blk = doc.findBlockByNumber(ln)
                t = blk.text()
                stripped = t.lstrip(" ")
                if not stripped:
                    continue
                indent = len(t) - len(stripped)
                bp = blk.position()
                if all_commented:
                    rm_len = 1
                    if len(stripped) > 1 and stripped[1] == " ":
                        rm_len = 2
                    e = QTextCursor(doc)
                    e.setPosition(bp + indent)
                    e.setPosition(bp + indent + rm_len, QTextCursor.MoveMode.KeepAnchor)
                    e.removeSelectedText()
                else:
                    e = QTextCursor(doc)
                    e.setPosition(bp + indent)
                    e.insertText("' ")
        finally:
            edit.endEditBlock()

    # ----------------------------------------------- Move/Duplicate Line
    def move_lines(self, direction: int) -> None:
        """Verschiebt die selektierten (oder Cursor-)Zeilen um `direction`
        (+1 = nach unten, -1 = nach oben)."""
        if direction not in (-1, 1):
            return
        cursor = self.textCursor()
        doc = self.document()
        first = doc.findBlock(cursor.selectionStart()).blockNumber()
        last = doc.findBlock(cursor.selectionEnd()).blockNumber()
        if cursor.hasSelection() and doc.findBlock(cursor.selectionEnd()).position() == cursor.selectionEnd() and first != last:
            last -= 1
        if direction < 0 and first == 0:
            return
        if direction > 0 and last >= doc.blockCount() - 1:
            return

        # 1-basierte Zeilen-Umordnung fuer Line-Marker (Breakpoints/
        # Bookmarks/Folds): der Text-Ersatz unten ist ein Delete+Insert
        # ueber den ganzen betroffenen Bereich, wodurch ein QTextCursor
        # AUF einer der verschobenen Zeilen NICHT automatisch mitwandert
        # (anders als bei Edits oberhalb/unterhalb eines Markers) --
        # Qt kollabiert ihn stattdessen an den Einfuegepunkt. Deshalb
        # migrieren wir betroffene Marker explizit ueber `remap()`
        # (Review-Fund: Alt+Up/Down liess Breakpoints/Bookmarks auf der
        # alten Zeile zurueck, die jetzt woanders liegt).
        marker_remap: dict[int, int] = {}

        if direction < 0:
            swap_ln = first - 1
            block_swap = doc.findBlockByNumber(swap_ln).text()
            block_lines = [doc.findBlockByNumber(i).text() for i in range(first, last + 1)]
            new_lines = block_lines + [block_swap]
            replace_first = swap_ln
            replace_last = last
            marker_remap[swap_ln + 1] = last + 1
            for i in range(first, last + 1):
                marker_remap[i + 1] = i
        else:
            swap_ln = last + 1
            block_swap = doc.findBlockByNumber(swap_ln).text()
            block_lines = [doc.findBlockByNumber(i).text() for i in range(first, last + 1)]
            new_lines = [block_swap] + block_lines
            replace_first = first
            replace_last = swap_ln
            marker_remap[swap_ln + 1] = first + 1
            for i in range(first, last + 1):
                marker_remap[i + 1] = i + 2

        edit = QTextCursor(doc)
        edit.beginEditBlock()
        try:
            edit.setPosition(doc.findBlockByNumber(replace_first).position())
            end_block = doc.findBlockByNumber(replace_last)
            edit.setPosition(
                end_block.position() + len(end_block.text()),
                QTextCursor.MoveMode.KeepAnchor,
            )
            edit.insertText("\n".join(new_lines))
        finally:
            edit.endEditBlock()

        self._shift_line_markers(marker_remap)

        new_first = replace_first if direction < 0 else first + 1
        new_last = new_first + (last - first)
        sel_start = doc.findBlockByNumber(new_first).position()
        sel_end_block = doc.findBlockByNumber(new_last)
        sel_end = sel_end_block.position() + len(sel_end_block.text())
        nc = self.textCursor()
        nc.setPosition(sel_start)
        nc.setPosition(sel_end, QTextCursor.MoveMode.KeepAnchor)
        self.setTextCursor(nc)

    def duplicate_lines(self) -> None:
        """Dupliziert die selektierte(n) Zeile(n) direkt darunter."""
        cursor = self.textCursor()
        doc = self.document()
        if cursor.hasSelection():
            first = doc.findBlock(cursor.selectionStart()).blockNumber()
            last = doc.findBlock(cursor.selectionEnd()).blockNumber()
            if doc.findBlock(cursor.selectionEnd()).position() == cursor.selectionEnd() and first != last:
                last -= 1
        else:
            first = last = cursor.blockNumber()
        chunk = "\n".join(doc.findBlockByNumber(i).text() for i in range(first, last + 1))
        last_block = doc.findBlockByNumber(last)
        insert_pos = last_block.position() + len(last_block.text())
        edit = QTextCursor(doc)
        edit.beginEditBlock()
        edit.setPosition(insert_pos)
        edit.insertText("\n" + chunk)
        edit.endEditBlock()
