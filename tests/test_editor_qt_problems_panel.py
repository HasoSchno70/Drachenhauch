"""Tests fuer das Probleme-Panel (alle Live-Diagnosen statt nur erstem Fehler)."""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _problem(line, msg, severity="error", phase="compile"):
    from gamebasic.editor_qt.error_check import ParseProblem
    return ParseProblem(line=line, message=msg, severity=severity, phase=phase)


def test_set_problems_lists_all():
    from gamebasic.editor_qt.problems_panel import ProblemsPanel
    p = ProblemsPanel()
    p.set_problems([_problem(3, "A"), _problem(7, "B", "warning", "preprocess")])
    assert p.list.count() == 2
    assert p._count.text() == "2"


def test_errors_sorted_before_warnings():
    from gamebasic.editor_qt.problems_panel import ProblemsPanel
    from PySide6.QtCore import Qt
    p = ProblemsPanel()
    p.set_problems([
        _problem(9, "warn", "warning", "preprocess"),
        _problem(2, "err", "error", "compile"),
    ])
    # Erster Eintrag = Error (Zeile 2), trotz spaeterer Listenposition.
    first = p.list.item(0)
    assert first.data(Qt.ItemDataRole.UserRole) == 2


def test_empty_shows_placeholder_not_counted():
    from gamebasic.editor_qt.problems_panel import ProblemsPanel
    p = ProblemsPanel()
    p.set_problems([])
    assert p._count.text() == "0"
    # Es gibt einen Platzhalter-Eintrag, aber er ist nicht anklickbar.
    from PySide6.QtCore import Qt
    assert p.list.count() == 1
    assert p.list.item(0).flags() == Qt.ItemFlag.NoItemFlags


def test_click_emits_jump_to_line():
    from gamebasic.editor_qt.problems_panel import ProblemsPanel
    p = ProblemsPanel()
    p.set_problems([_problem(5, "X")])
    got = []
    p.jump_requested.connect(lambda ln: got.append(ln))
    p._on_clicked(p.list.item(0))
    assert got == [5]
