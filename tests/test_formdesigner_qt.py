"""Konstruktions-Smoke-Tests fuer die Form-Designer-Qt-UI (offscreen).
Faengt Import-/API-/Wiring-Fehler; Interaktion (Maus) ist nicht abgedeckt.
"""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
pytest.importorskip("PySide6")

from PySide6.QtWidgets import QApplication   # noqa: E402

from gamebasic.formdesigner_qt import FormDesigner   # noqa: E402


def _app():
    return QApplication.instance() or QApplication([])


def test_construct_and_palette(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    assert win.canvas.doc is not None
    assert win.palette.count() >= 12          # alle Palette-Eintraege
    win.close()


def test_select_updates_inspector(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    c = win.canvas.doc.add("button", 10, 10)
    win.canvas._select(c)
    assert win.inspector._c is c
    win.close()


def test_inspector_apply_updates_control(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    c = win.canvas.doc.add("button", 10, 10)
    win.canvas._select(c)
    win.inspector.text.setText("Hallo")
    win.inspector.on_click.setText("on_ok")
    win.inspector.sx.setValue(42)
    win.inspector._apply()
    assert c.text == "Hallo" and c.on_click == "on_ok" and c.x == 42
    win.close()


def test_save_load_roundtrip(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    win.canvas.doc.title = "MyForm"
    win.canvas.doc.add("dropdown", 20, 20).items = ["a", "b"]
    p = tmp_path / "f.gbform"
    win.canvas.doc.save(str(p))
    win2 = FormDesigner(tmp_path)
    from gamebasic.formdesigner import FormDoc
    win2.canvas.set_doc(FormDoc.load(str(p)))
    assert win2.canvas.doc.title == "MyForm"
    assert win2.canvas.doc.controls[0].items == ["a", "b"]
    win.close(); win2.close()
