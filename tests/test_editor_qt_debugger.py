"""Tests fuer Conditional Breakpoints im Editor-Debugger.

Der Debugger laeuft sonst in einem Worker-Thread mit Qt-Signalen; hier
testen wir die thread-freie Kernlogik headless: Bedingungs-Parsing,
merged-Line-Mapping und die Auswertung gegen einen echten Interpreter.
"""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

from gamebasic.editor_qt.debugger import DebugController, _EDITOR_LABEL
from gamebasic.interpreter import Interpreter


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _interp_with(name, value, typ="int"):
    interp = Interpreter()
    interp.global_env.declare(name, typ, value)
    return interp


# --- _compile_condition --------------------------------------------

def test_compile_condition_valid():
    expr = DebugController._compile_condition("i > 5")
    assert expr is not None


def test_compile_condition_invalid_returns_none():
    # Kaputter Ausdruck -> None (fail-open: BP haelt dann unbedingt).
    assert DebugController._compile_condition("i > > 5") is None


# --- set_breakpoints / merged mapping (ohne origins) ---------------

def test_set_breakpoints_keeps_only_active_conditions():
    ctrl = DebugController()
    ctrl.set_breakpoints([3, 5], {3: "i > 5", 9: "nope"})
    assert ctrl._merged_bps == {3, 5}
    # Zeile 3 hat aktiven BP + Bedingung -> in merged_bp_conditions
    assert 3 in ctrl._merged_bp_conditions
    # Zeile 5 hat BP aber keine Bedingung -> unbedingt
    assert 5 not in ctrl._merged_bp_conditions
    # Zeile 9 hat eine "Bedingung" ohne BP -> verworfen
    assert 9 not in ctrl._merged_bps


def test_set_breakpoints_no_conditions():
    ctrl = DebugController()
    ctrl.set_breakpoints([2])
    assert ctrl._merged_bps == {2}
    assert ctrl._merged_bp_conditions == {}


# --- merged mapping mit origins ------------------------------------

def test_merged_conditions_with_origins():
    ctrl = DebugController()
    # origins[m] = (datei, original-Zeile); Index 0 ungenutzt.
    ctrl._origins = [None,
                     (_EDITOR_LABEL, 1),
                     (_EDITOR_LABEL, 2),
                     (_EDITOR_LABEL, 3)]
    ctrl.set_breakpoints([3], {3: "x = 1"})
    assert ctrl._merged_bps == {3}
    assert 3 in ctrl._merged_bp_conditions


# --- _condition_holds ----------------------------------------------

def test_condition_holds_true():
    ctrl = DebugController()
    ctrl._merged_bp_conditions = {3: ctrl._compile_condition("i > 5")}
    interp = _interp_with("i", 7)
    assert ctrl._condition_holds(3, interp) is True


def test_condition_holds_false():
    ctrl = DebugController()
    ctrl._merged_bp_conditions = {3: ctrl._compile_condition("i > 5")}
    interp = _interp_with("i", 2)
    assert ctrl._condition_holds(3, interp) is False


def test_condition_holds_no_condition_is_true():
    ctrl = DebugController()
    interp = _interp_with("i", 0)
    # Keine Bedingung fuer Zeile 4 -> unbedingt halten.
    assert ctrl._condition_holds(4, interp) is True


def test_condition_holds_eval_error_fails_open():
    ctrl = DebugController()
    # Bedingung referenziert eine undefinierte Variable -> Eval-Fehler.
    ctrl._merged_bp_conditions = {3: ctrl._compile_condition("unknown > 1")}
    interp = Interpreter()
    assert ctrl._condition_holds(3, interp) is True


def test_condition_holds_resets_in_condition_flag():
    ctrl = DebugController()
    ctrl._merged_bp_conditions = {3: ctrl._compile_condition("i > 5")}
    interp = _interp_with("i", 7)
    ctrl._condition_holds(3, interp)
    assert ctrl._in_condition is False
