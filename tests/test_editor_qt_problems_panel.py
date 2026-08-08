"""Tests fuer das Probleme-Panel (alle Live-Diagnosen statt nur erstem Fehler).

Seit dem Umbau eine TABELLE (Zeile / Art / Meldung) statt einer Liste: bei
mehr als einer Handvoll Diagnosen will man nach Zeile sortieren (der Reihe
nach abarbeiten) oder nach Art (erst die Fehler). Die Tests decken deshalb
zusaetzlich ab, dass ZAHLENWEISE bzw. nach SCHWERE sortiert wird -- nach
Anzeigetext stuende Zeile 10 vor Zeile 9.
"""
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


def _panel(problems):
    from gamebasic.editor_qt.problems_panel import ProblemsPanel
    p = ProblemsPanel()
    p.set_problems(problems)
    return p


def test_set_problems_lists_all():
    p = _panel([_problem(3, "A"), _problem(7, "B", "warning", "preprocess")])
    assert p.table.rowCount() == 2
    assert p._count.text() == "2"
    assert p.table.item(0, 2).text() in ("A", "B")


def test_anfangs_nach_zeile_sortiert():
    """Die Anfangsreihenfolge ist nach ZEILE. Vorher (als Liste) standen die
    Fehler oben; mit sortierbaren Kopfspalten waere das nur mit einem
    versteckten Sortier-Schluessel zu halten, der beim Klick auf "Zeile" etwas
    anderes taete als "Zeile" verspricht. Fehler zuerst ist ein Klick auf
    "Art"."""
    from PySide6.QtCore import Qt
    p = _panel([
        _problem(9, "warn", "warning", "preprocess"),
        _problem(2, "err", "error", "compile"),
    ])
    zeilen = [p.table.item(r, 0).data(Qt.ItemDataRole.UserRole)
              for r in range(p.table.rowCount())]
    assert zeilen == [2, 9], zeilen


def test_eigene_sortierung_ueberlebt_neue_diagnosen():
    """Wer nach Art sortiert hat, will das behalten, waehrend er weitertippt --
    sonst spraenge die Tabelle bei jedem Tastendruck zurueck."""
    from PySide6.QtCore import Qt
    p = _panel([_problem(1, "a"), _problem(5, "b")])
    p._kopf_geklickt(1)
    p.table.sortItems(1, Qt.SortOrder.DescendingOrder)
    p.set_problems([_problem(2, "w", "warning", "p"), _problem(8, "e", "error", "c")])
    arten = [p.table.item(r, 1).text() for r in range(p.table.rowCount())]
    assert arten[0].endswith("Warnung"), arten


def test_empty_shows_placeholder_not_counted():
    from PySide6.QtCore import Qt
    p = _panel([])
    assert p._count.text() == "0"
    # Eine Hinweiszeile, aber nicht anklickbar.
    assert p.table.rowCount() == 1
    assert p.table.item(0, 2).flags() == Qt.ItemFlag.NoItemFlags


def test_click_emits_jump_to_line():
    p = _panel([_problem(5, "X")])
    got = []
    p.jump_requested.connect(lambda ln: got.append(ln))
    p._on_cell(0, 0)
    assert got == [5]


def test_klick_in_jeder_spalte_springt():
    """Die Zeilennummer haengt an jeder Zelle -- wer auf die Meldung klickt,
    will genauso springen wie wer auf die Zeilennummer klickt."""
    p = _panel([_problem(5, "X")])
    got = []
    p.jump_requested.connect(lambda ln: got.append(ln))
    for spalte in (0, 1, 2):
        p._on_cell(0, spalte)
    assert got == [5, 5, 5]


def test_zeilenspalte_sortiert_zahlenweise():
    """Nach Anzeigetext stuende 10 vor 9 -- in einer Fehlerliste die
    haeufigste Enttaeuschung."""
    from PySide6.QtCore import Qt
    p = _panel([_problem(9, "neun"), _problem(10, "zehn"), _problem(2, "zwei")])
    p.table.sortItems(0, Qt.SortOrder.AscendingOrder)
    zeilen = [int(p.table.item(r, 0).text()) for r in range(p.table.rowCount())]
    assert zeilen == [2, 9, 10], zeilen


def test_artspalte_sortiert_nach_schwere():
    """Nicht nach Anfangsbuchstabe: Fehler gehoeren vor Warnungen, egal wie
    die beiden Woerter alphabetisch stehen."""
    from PySide6.QtCore import Qt
    p = _panel([
        _problem(1, "w1", "warning", "preprocess"),
        _problem(2, "e1", "error", "compile"),
        _problem(3, "w2", "warning", "preprocess"),
    ])
    p.table.sortItems(1, Qt.SortOrder.AscendingOrder)
    arten = [p.table.item(r, 1).text() for r in range(p.table.rowCount())]
    assert arten[0].endswith("Fehler"), arten


def test_neu_setzen_laesst_keine_zellen_stehen():
    """Beim Fuellen ist die Sortierung abgeschaltet -- sonst sortiert Qt nach
    jeder eingefuegten Zeile neu und die Zellen landen in fremden Zeilen."""
    p = _panel([_problem(1, "A"), _problem(2, "B"), _problem(3, "C")])
    p.set_problems([_problem(7, "nur eins")])
    assert p.table.rowCount() == 1
    assert p.table.item(0, 2).text() == "nur eins"
    assert p._count.text() == "1"
