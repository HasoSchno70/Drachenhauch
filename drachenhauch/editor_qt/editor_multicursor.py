"""Multi-Cursor-Mixin (Strg+D = naechstes Vorkommen).

Sekundaere Selektionen werden in `self._secondary` als (start, end)-Tupel
gehalten. Jede Tasten-Eingabe in `keyPressEvent` (im Hauptmodul) wendet
sich auf alle Cursor an. Visualisierung laeuft ueber `ExtraSelections`,
gerendert in `_refresh_extra_selections` des Hauptmoduls.
"""
from __future__ import annotations

from PySide6.QtGui import QTextCursor


class MultiCursorMixin:
    """Strg+D-Erweiterung der primaeren Selektion + Multi-Edit-Support."""

    # `_secondary` wird im CodeEditor-__init__ als [] initialisiert.

    def add_next_occurrence(self) -> None:
        """Strg+D: erweitert die Multi-Selektion um das naechste Vorkommen
        des selektierten Texts.

        Ohne Selektion wird zuerst das Wort am primaeren Cursor markiert.
        Danach wird ab dem letzten bekannten Selektions-Ende vorwaerts
        gesucht; gefundenes Vorkommen wandert in `_secondary`.
        """
        cursor = self.textCursor()
        if not cursor.hasSelection():
            cursor.select(QTextCursor.SelectionType.WordUnderCursor)
            self.setTextCursor(cursor)
            self._refresh_extra_selections()
            return
        needle = cursor.selectedText()
        if not needle:
            return
        primary_end = max(cursor.selectionStart(), cursor.selectionEnd())
        last_end = primary_end
        for s, e in self._secondary:
            if e > last_end:
                last_end = e
        full = self.toPlainText()
        idx = full.find(needle, last_end)
        if idx < 0:
            existing = {(cursor.selectionStart(), cursor.selectionEnd())} | set(self._secondary)
            i = 0
            while True:
                idx = full.find(needle, i)
                if idx < 0:
                    return
                if (idx, idx + len(needle)) not in existing:
                    break
                i = idx + 1
        self._secondary.append((idx, idx + len(needle)))
        self._refresh_extra_selections()
        peek = QTextCursor(self.document())
        peek.setPosition(idx + len(needle))
        self.setTextCursor(peek)
        primary = QTextCursor(self.document())
        primary.setPosition(cursor.selectionStart())
        primary.setPosition(cursor.selectionEnd(), QTextCursor.MoveMode.KeepAnchor)
        self.setTextCursor(primary)
        self.ensureCursorVisible()

    def _clear_secondary_cursors(self) -> None:
        if self._secondary:
            self._secondary = []
            self._refresh_extra_selections()

    def _multi_edit(self, op) -> None:
        """Wendet `op(cursor)` auf primaer + alle sekundaeren Cursor an.

        QTextCursor folgt Document-Edits automatisch -- wir bauen alle
        Cursor-Objekte VORHER, editieren dann der Reihe nach (highest
        end-pos zuerst, damit `op()` selbst keine fruheren Positionen
        verschiebt) und lesen am Ende ihre tatsaechlichen End-Positionen.
        """
        doc = self.document()
        primary = self.textCursor()

        cursors: list[tuple[QTextCursor, bool]] = []
        for s, e in self._secondary:
            tc = QTextCursor(doc)
            tc.setPosition(s)
            tc.setPosition(e, QTextCursor.MoveMode.KeepAnchor)
            cursors.append((tc, False))
        cursors.append((primary, True))

        cursors.sort(key=lambda c: -c[0].selectionEnd())

        edit_anchor = self.textCursor()
        edit_anchor.beginEditBlock()
        try:
            for tc, _is_primary in cursors:
                op(tc)
        finally:
            edit_anchor.endEditBlock()

        new_secondary: list[tuple[int, int]] = []
        primary_after: int | None = None
        for tc, is_primary in cursors:
            pos = tc.position()
            if is_primary:
                primary_after = pos
            else:
                new_secondary.append((pos, pos))
        new_secondary.sort()
        self._secondary = new_secondary
        if primary_after is not None:
            np = self.textCursor()
            np.setPosition(primary_after)
            self.setTextCursor(np)
        self._refresh_extra_selections()
