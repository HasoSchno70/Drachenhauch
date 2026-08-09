"""Tests fuer zeilengebundene Marker (Bookmarks/Breakpoints/Folds), die
Edits oberhalb ueberstehen muessen -- vorher rohe int-Zeilen, die nach
Zeilen-Verschiebung auf die falsche Stelle zeigten (Review-Fund)."""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _editor(src: str):
    from drachenhauch.editor_qt.code_editor import CodeEditor
    ed = CodeEditor()
    ed.set_text(src)
    return ed


def _cursor_at_block(ed, block_idx: int):
    from PySide6.QtGui import QTextCursor
    return QTextCursor(ed.document().findBlockByNumber(block_idx))


def test_bookmark_survives_edit_above_it():
    ed = _editor("A\nB\nC")
    ed.setTextCursor(_cursor_at_block(ed, 2))    # Zeile 3 ("C")
    ed.toggle_bookmark()
    assert ed._bookmarks.lines() == [3]

    _cursor_at_block(ed, 0).insertText("X\nY\n")
    assert ed._bookmarks.lines() == [5]          # "C" ist jetzt Zeile 5


def test_breakpoint_and_condition_survive_edit_above():
    ed = _editor("A\nB\nC")
    ed.toggle_breakpoint(3)
    ed._breakpoints.set(3, "x > 1")
    assert ed.breakpoints() == {3}
    assert ed.breakpoint_conditions() == {3: "x > 1"}

    _cursor_at_block(ed, 0).insertText("X\nY\n")
    assert ed.breakpoints() == {5}
    assert ed.breakpoint_conditions() == {5: "x > 1"}


FOLD_SRC = (
    "SUB foo()\n"      # 1
    "    PRINT 1\n"    # 2
    "END SUB\n"        # 3
    "SUB bar()\n"      # 4
    "    PRINT 2\n"    # 5
    "END SUB\n"        # 6
)


def test_fold_state_survives_unrelated_edit_above():
    ed = _editor(FOLD_SRC)
    ed._rescan_fold_regions()
    ed._toggle_fold(1, 3)
    ed._toggle_fold(4, 6)
    assert ed.folded_starts() == [1, 4]

    # Kopfzeile einfuegen -- beide Regionen verschieben sich um eine
    # Zeile, der Fold-Status muss folgen (Cursor-Tracking), nicht
    # verworfen werden (frueher: globales Unfold als Notbremse).
    _cursor_at_block(ed, 0).insertText("REM header\n")
    ed._rescan_fold_regions()
    assert ed.folded_starts() == [2, 5]


def test_fold_invalidation_only_drops_the_affected_region():
    """Wird eine gefaltete Region durch einen Edit strukturell ungueltig
    (hier: `END SUB` von foo verschwindet), darf NUR ihr eigener
    verborgener Bereich wieder aufklappen -- eine andere, unbetroffene
    gefaltete Region (bar) muss gefaltet bleiben."""
    from PySide6.QtGui import QTextCursor
    ed = _editor(FOLD_SRC)
    ed._rescan_fold_regions()
    ed._toggle_fold(1, 3)
    ed._toggle_fold(4, 6)
    assert ed.folded_starts() == [1, 4]

    blk = ed.document().findBlockByNumber(2)   # Zeile 3 ("END SUB" von foo)
    c = QTextCursor(blk)
    c.select(QTextCursor.SelectionType.LineUnderCursor)
    c.insertText("    PRINT 3")                # kein "END SUB" mehr -> Struktur weg
    ed._rescan_fold_regions()

    assert ed.folded_starts() == [4]           # foo-Fold entfernt, bar bleibt
    assert ed.document().findBlockByNumber(1).isVisible()      # foo Zeile 2 wieder sichtbar
    assert not ed.document().findBlockByNumber(4).isVisible()  # bar Zeile 5 bleibt verborgen


def test_move_lines_up_carries_breakpoint_and_bookmark():
    ed = _editor("A\nB\nC\nD")
    ed.toggle_breakpoint(2)
    ed._bookmarks.set(2)
    ed.setTextCursor(_cursor_at_block(ed, 1))   # Zeile 2 ("B")
    ed.move_lines(-1)

    assert ed.toPlainText().split("\n")[:2] == ["B", "A"]
    assert ed.breakpoints() == {1}
    assert ed._bookmarks.lines() == [1]


def test_move_lines_down_carries_marker():
    ed = _editor("A\nB\nC")
    ed.toggle_breakpoint(1)
    ed.setTextCursor(_cursor_at_block(ed, 0))   # Zeile 1 ("A")
    ed.move_lines(1)

    assert ed.toPlainText().split("\n")[:2] == ["B", "A"]
    assert ed.breakpoints() == {2}
