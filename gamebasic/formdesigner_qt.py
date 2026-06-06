"""Form-Designer (WYSIWYG, Xojo-Stil) -- PySide6-UI.

Eigenstaendiges Qt-Programm: links die Control-Palette, in der Mitte die
Design-Flaeche (Controls platzieren/selektieren/verschieben/loeschen), rechts
der Inspector (Eigenschaften + Events). Speichert/laedt `.gbform` (Runtime-
Format) und kann das Formular per "Run" mit `gbrt` starten (laedt das Layout +
generierte Event-Handler-Stubs -- der Xojo-Lauf).

Datenmodell + Code-Generierung liegen Qt-frei in `gamebasic/formdesigner/`.
Start: `gbform [datei.gbform]` bzw. `gbrun.py --form`.
"""
from __future__ import annotations

import os
import subprocess
import tempfile
from pathlib import Path

from PySide6.QtCore import Qt, QRect, QPoint, Signal
from PySide6.QtGui import QAction, QColor, QPainter, QPen, QBrush, QKeySequence, QFont
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QListWidget, QListWidgetItem, QDockWidget,
    QScrollArea, QFormLayout, QLineEdit, QSpinBox, QCheckBox, QPlainTextEdit,
    QFileDialog, QMessageBox, QLabel, QVBoxLayout, QDoubleSpinBox,
)

from .formdesigner import FormDoc, Control, PALETTE, palette_spec

try:
    from .editor_qt.theme import global_qss
except Exception:  # pragma: no cover - Theme optional
    def global_qss() -> str:
        return ""

PAD = 24          # Rand um das Fenster auf der Canvas
TITLE_H = 22      # Titelleisten-Hoehe (wie im gui-Modul)


def _find_gbrt():
    root = Path(__file__).resolve().parents[1]
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for v in ("release", "debug"):
        p = root / "rust" / "gb_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


def _col(i: int) -> QColor:
    return QColor((i >> 16) & 0xFF, (i >> 8) & 0xFF, i & 0xFF)


class _Canvas(QWidget):
    """Zeichnet das Formular + Controls und behandelt Platzieren/Selektieren/Ziehen."""
    selection_changed = Signal(object)   # Control | None
    doc_changed = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc = FormDoc()
        self.selected: Control | None = None
        self.place_kind: str | None = None      # aus der Palette "scharf geschaltet"
        self._drag = False
        self._drag_off = QPoint(0, 0)
        self.setMinimumSize(640, 480)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)

    def set_doc(self, doc: FormDoc):
        self.doc = doc
        self.selected = None
        self.selection_changed.emit(None)
        self._resize_to_doc()
        self.update()

    def _resize_to_doc(self):
        self.setMinimumSize(self.doc.w + 2 * PAD + 40, self.doc.h + 2 * PAD + 40)

    # -- Koordinaten: Canvas -> Control-relativ (Fenster-Inhalt) --
    def _to_ctrl(self, p: QPoint) -> tuple[int, int]:
        return (p.x() - PAD, p.y() - PAD - TITLE_H)

    def paintEvent(self, _ev):
        qp = QPainter(self)
        qp.fillRect(self.rect(), QColor(18, 22, 28))
        d = self.doc
        win = QRect(PAD, PAD, d.w, d.h)
        qp.fillRect(win, QColor(24, 34, 46))
        qp.setPen(QPen(QColor(46, 88, 110), 1))
        qp.drawRect(win)
        # Titelleiste
        qp.fillRect(QRect(PAD, PAD, d.w, TITLE_H), QColor(18, 90, 120))
        qp.setPen(QColor(230, 247, 255))
        qp.drawText(PAD + 6, PAD + 15, d.title)
        # Controls
        for c in d.controls:
            self._paint_control(qp, c)
        if self.selected is not None:
            self._paint_handles(qp, self.selected)

    def _paint_control(self, qp: QPainter, c: Control):
        x = PAD + c.x
        y = PAD + TITLE_H + c.y
        r = QRect(x, y, max(c.w, 4), max(c.h, 4))
        fill = {
            "button": QColor(38, 50, 63), "textinput": QColor(20, 26, 34),
            "dropdown": QColor(38, 50, 63), "listbox": QColor(30, 40, 52),
            "panel": QColor(30, 40, 52), "progress": QColor(30, 40, 52),
            "canvas": QColor(14, 20, 28), "image": QColor(40, 44, 52),
        }.get(c.kind, QColor(30, 40, 52))
        if c.kind in ("label", "checkbox", "radio"):
            qp.setPen(QColor(230, 230, 230) if c.enabled else QColor(120, 130, 145))
            label = c.text or c.kind
            if c.kind == "checkbox":
                qp.drawRect(QRect(x, y, c.h, c.h)); qp.drawText(x + c.h + 6, y + 13, label)
            elif c.kind == "radio":
                qp.drawEllipse(QRect(x, y, c.h, c.h)); qp.drawText(x + c.h + 6, y + 13, label)
            else:
                qp.drawText(x, y + 13, label)
            return
        qp.fillRect(r, fill)
        qp.setPen(QPen(QColor(70, 88, 110), 1))
        qp.drawRect(r)
        qp.setPen(QColor(230, 230, 230) if c.enabled else QColor(120, 130, 145))
        cap = c.text
        if c.kind == "dropdown":
            cap = (c.items[c.sel] if 0 <= c.sel < len(c.items) else "") + "  v"
        elif c.kind == "textinput":
            cap = c.text or c.placeholder
        elif c.kind == "progress":
            cap = f"{int(c.value)}%"
        elif c.kind == "listbox":
            cap = c.items[0] if c.items else ""
        elif c.kind == "image":
            cap = "[Bild]"
        elif c.kind == "canvas":
            cap = "Canvas"
        if cap:
            qp.drawText(x + 5, y + min(c.h - 4, 15), str(cap))

    def _paint_handles(self, qp: QPainter, c: Control):
        x = PAD + c.x
        y = PAD + TITLE_H + c.y
        qp.setPen(QPen(QColor(43, 196, 232), 2))
        qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRect(QRect(x - 2, y - 2, c.w + 4, c.h + 4))

    # -- Maus --
    def mousePressEvent(self, ev):
        cx, cy = self._to_ctrl(ev.position().toPoint())
        if self.place_kind:
            c = self.doc.add(self.place_kind, max(cx, 0), max(cy, 0))
            self.place_kind = None
            self._select(c)
            self.doc_changed.emit()
            self.update()
            return
        hit = self.doc.control_at(cx, cy)
        self._select(hit)
        if hit is not None:
            self._drag = True
            self._drag_off = QPoint(cx - hit.x, cy - hit.y)
        self.update()

    def mouseMoveEvent(self, ev):
        if self._drag and self.selected is not None:
            cx, cy = self._to_ctrl(ev.position().toPoint())
            self.selected.x = max(0, cx - self._drag_off.x())
            self.selected.y = max(0, cy - self._drag_off.y())
            self.selection_changed.emit(self.selected)   # Inspector live aktualisieren
            self.doc_changed.emit()
            self.update()

    def mouseReleaseEvent(self, _ev):
        self._drag = False

    def keyPressEvent(self, ev):
        if ev.key() in (Qt.Key.Key_Delete, Qt.Key.Key_Backspace) and self.selected is not None:
            self.doc.remove(self.selected)
            self._select(None)
            self.doc_changed.emit()
            self.update()
        else:
            super().keyPressEvent(ev)

    def _select(self, c: Control | None):
        self.selected = c
        self.selection_changed.emit(c)


class _Inspector(QWidget):
    """Eigenschaften + Events des gewaehlten Controls editieren."""
    changed = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self._c: Control | None = None
        self._loading = False
        self._form = QFormLayout(self)
        self.name = QLineEdit(); self.text = QLineEdit()
        self.sx = QSpinBox(); self.sy = QSpinBox(); self.sw = QSpinBox(); self.sh = QSpinBox()
        for s in (self.sx, self.sy, self.sw, self.sh):
            s.setRange(0, 4000)
        self.enabled = QCheckBox("aktiviert")
        self.checked = QCheckBox("angehakt")
        self.on_click = QLineEdit(); self.on_change = QLineEdit()
        self.items = QPlainTextEdit(); self.items.setMaximumHeight(90)
        self.vmin = QDoubleSpinBox(); self.vmax = QDoubleSpinBox(); self.vval = QDoubleSpinBox()
        for s in (self.vmin, self.vmax, self.vval):
            s.setRange(-1e6, 1e6)
        self._rows = []
        self._add("Name", self.name)
        self._add("Text", self.text)
        self._add("X", self.sx); self._add("Y", self.sy)
        self._add("Breite", self.sw); self._add("Hoehe", self.sh)
        self._add("on_click", self.on_click)
        self._add("on_change", self.on_change)
        self._add("Items (1/Zeile)", self.items)
        self._add("Min", self.vmin); self._add("Max", self.vmax); self._add("Wert", self.vval)
        self._add("", self.enabled)
        self._add("", self.checked)
        # Signale
        self.name.editingFinished.connect(self._apply)
        self.text.editingFinished.connect(self._apply)
        for s in (self.sx, self.sy, self.sw, self.sh, self.vmin, self.vmax, self.vval):
            s.valueChanged.connect(self._apply)
        self.on_click.editingFinished.connect(self._apply)
        self.on_change.editingFinished.connect(self._apply)
        self.items.textChanged.connect(self._apply)
        self.enabled.toggled.connect(self._apply)
        self.checked.toggled.connect(self._apply)
        self.set_control(None)

    def _add(self, label, widget):
        self._form.addRow(label, widget)
        self._rows.append((label, widget))

    def _show(self, widget, on: bool):
        widget.setVisible(on)
        lbl = self._form.labelForField(widget)
        if lbl is not None:
            lbl.setVisible(on)

    def set_control(self, c: Control | None):
        self._c = c
        self._loading = True
        if c is None:
            for _, w in self._rows:
                self._show(w, False)
            self._loading = False
            return
        sp = palette_spec(c.kind)
        self.name.setText(c.name); self.text.setText(c.text)
        self.sx.setValue(c.x); self.sy.setValue(c.y); self.sw.setValue(c.w); self.sh.setValue(c.h)
        self.on_click.setText(c.on_click); self.on_change.setText(c.on_change)
        self.items.setPlainText("\n".join(c.items))
        self.vmin.setValue(c.min); self.vmax.setValue(c.max); self.vval.setValue(c.value)
        self.enabled.setChecked(c.enabled); self.checked.setChecked(c.checked)
        has_text = bool(sp and sp.has_text)
        has_items = bool(sp and sp.has_items)
        events = sp.events if sp else ()
        is_range = c.kind in ("slider", "progress")
        is_check = c.kind in ("checkbox", "radio")
        for w in (self.name, self.sx, self.sy, self.sw, self.sh, self.enabled):
            self._show(w, True)
        self._show(self.text, has_text)
        self._show(self.items, has_items)
        self._show(self.on_click, "on_click" in events)
        self._show(self.on_change, "on_change" in events)
        self._show(self.vmin, is_range); self._show(self.vmax, is_range); self._show(self.vval, is_range)
        self._show(self.checked, is_check)
        self._loading = False

    def _apply(self):
        if self._loading or self._c is None:
            return
        c = self._c
        c.name = self.name.text().strip()
        c.text = self.text.text()
        c.x, c.y, c.w, c.h = self.sx.value(), self.sy.value(), self.sw.value(), self.sh.value()
        c.on_click = self.on_click.text().strip()
        c.on_change = self.on_change.text().strip()
        items = [ln for ln in self.items.toPlainText().splitlines() if ln != ""]
        c.items = items
        if c.kind == "dropdown" and items and c.sel < 0:
            c.sel = 0
        c.min, c.max, c.value = self.vmin.value(), self.vmax.value(), self.vval.value()
        c.enabled = self.enabled.isChecked()
        c.checked = self.checked.isChecked()
        self.changed.emit()


class FormDesigner(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = Path(project_root)
        self.path: Path | None = None
        self.setWindowTitle("GameBasic Form-Designer")
        self.resize(1100, 740)

        self.canvas = _Canvas()
        scroll = QScrollArea(); scroll.setWidget(self.canvas); scroll.setWidgetResizable(False)
        self.setCentralWidget(scroll)

        # Palette
        self.palette = QListWidget()
        for sp in PALETTE:
            it = QListWidgetItem(sp.label); it.setData(Qt.ItemDataRole.UserRole, sp.kind)
            self.palette.addItem(it)
        self.palette.itemClicked.connect(self._arm_place)
        self._dock("Controls", self.palette, Qt.DockWidgetArea.LeftDockWidgetArea)

        # Inspector
        self.inspector = _Inspector()
        self._dock("Inspector", self.inspector, Qt.DockWidgetArea.RightDockWidgetArea)

        self.canvas.selection_changed.connect(self.inspector.set_control)
        self.canvas.doc_changed.connect(self._mark_dirty)
        self.inspector.changed.connect(self.canvas.update)

        self._build_menu()
        self.canvas.set_doc(FormDoc())
        self._update_title()

    def _dock(self, title, widget, area):
        d = QDockWidget(title, self)
        d.setWidget(widget)
        self.addDockWidget(area, d)
        return d

    def _build_menu(self):
        m = self.menuBar().addMenu("&Datei")
        def act(name, shortcut, fn):
            a = QAction(name, self)
            if shortcut:
                a.setShortcut(QKeySequence(shortcut))
            a.triggered.connect(fn); m.addAction(a); return a
        act("Neu", "Ctrl+N", self.new_form)
        act("Oeffnen...", "Ctrl+O", self.open_form)
        act("Speichern", "Ctrl+S", self.save_form)
        act("Speichern unter...", "Ctrl+Shift+S", self.save_form_as)
        m.addSeparator()
        act("Ausfuehren (gbrt)", "F5", self.run_form)

    # -- Aktionen --
    def _mark_dirty(self):
        self._update_title(dirty=True)

    def _update_title(self, dirty=False):
        name = self.path.name if self.path else "(unbenannt)"
        self.setWindowTitle(f"GameBasic Form-Designer  --  {name}{'*' if dirty else ''}")

    def new_form(self):
        self.path = None
        self.canvas.set_doc(FormDoc())
        self._update_title()

    def open_form(self):
        fn, _ = QFileDialog.getOpenFileName(self, "Formular oeffnen", str(self.project_root),
                                            "GameBasic-Form (*.gbform);;Alle (*.*)")
        if not fn:
            return
        try:
            self.canvas.set_doc(FormDoc.load(fn))
            self.path = Path(fn); self._update_title()
        except Exception as e:  # noqa: BLE001
            QMessageBox.critical(self, "Fehler", f"Konnte nicht laden:\n{e}")

    def save_form(self):
        if self.path is None:
            return self.save_form_as()
        self.canvas.doc.save(str(self.path))
        self._update_title()

    def save_form_as(self):
        fn, _ = QFileDialog.getSaveFileName(self, "Formular speichern", str(self.project_root),
                                            "GameBasic-Form (*.gbform)")
        if not fn:
            return
        if not fn.endswith(".gbform"):
            fn += ".gbform"
        self.path = Path(fn)
        self.canvas.doc.save(fn)
        self._update_title()

    def run_form(self):
        gbrt = _find_gbrt()
        if gbrt is None:
            QMessageBox.warning(self, "gbrt fehlt", "Native Runtime nicht gebaut:\n"
                                "python rust/build_runtime.py")
            return
        tmp = Path(tempfile.mkdtemp(prefix="gbform_"))
        form_path = tmp / "form.gbform"
        self.canvas.doc.save(str(form_path))
        runner = self.canvas.doc.generate_runner("form.gbform",
                                                 screen_title=self.canvas.doc.title)
        gb = tmp / "run.gb"
        gb.write_text(runner, encoding="utf-8")
        subprocess.Popen([str(gbrt), "run", str(gb)], cwd=str(tmp))

    def _arm_place(self, item: QListWidgetItem):
        self.canvas.place_kind = item.data(Qt.ItemDataRole.UserRole)
        self.statusBar().showMessage(f"Platzieren: {item.text()} -- auf die Flaeche klicken", 4000)


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance() or QApplication([])
    app.setStyleSheet(global_qss())
    win = FormDesigner(project_root)
    if initial_file and Path(initial_file).exists():
        try:
            win.canvas.set_doc(FormDoc.load(str(initial_file)))
            win.path = Path(initial_file); win._update_title()
        except Exception:
            pass
    win.show()
    return app.exec()
