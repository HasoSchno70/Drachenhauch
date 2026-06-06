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
    QFileDialog, QMessageBox, QLabel, QVBoxLayout, QHBoxLayout, QDoubleSpinBox,
    QComboBox,
)

from .formdesigner import (
    FormDoc, FormProject, Control, History, PALETTE, palette_spec, GRID, HANDLES,
    snap, resize_rect,
)

try:
    from .editor_qt.theme import global_qss
except Exception:  # pragma: no cover - Theme optional
    def global_qss() -> str:
        return ""

try:
    from .editor_qt.highlighter import GBHighlighter
except Exception:  # pragma: no cover - Highlighter optional
    GBHighlighter = None

PAD = 24          # Rand um das Fenster auf der Canvas
TITLE_H = 22      # Titelleisten-Hoehe (wie im gui-Modul)
HANDLE = 8        # Kantenlaenge eines Resize-Griffs (px)

# Resize-Griff -> Maus-Cursor (diagonal/horizontal/vertikal)
_HANDLE_CURSORS = {
    "nw": Qt.CursorShape.SizeFDiagCursor, "se": Qt.CursorShape.SizeFDiagCursor,
    "ne": Qt.CursorShape.SizeBDiagCursor, "sw": Qt.CursorShape.SizeBDiagCursor,
    "n": Qt.CursorShape.SizeVerCursor, "s": Qt.CursorShape.SizeVerCursor,
    "e": Qt.CursorShape.SizeHorCursor, "w": Qt.CursorShape.SizeHorCursor,
}


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
    doc_replaced = Signal(object)        # FormDoc (komplett ersetzt: set_doc/Undo)
    handler_requested = Signal(object)   # Control (Doppelklick -> Code-Editor)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc = FormDoc()
        self.selected: Control | None = None
        self.place_kind: str | None = None      # aus der Palette "scharf geschaltet"
        self.snap_grid = True                    # Snap-to-Grid aktiv?
        # commit_history(pre_snapshot): vom Fenster gesetzt, legt einen
        # Undo-Checkpoint an. None = kein Undo (z.B. Standalone-Canvas).
        self.commit_history = None
        self._drag = False
        self._drag_off = QPoint(0, 0)
        self._resize_handle: str | None = None   # aktiver Resize-Griff beim Ziehen
        self._pending: dict | None = None        # Pre-Gesten-Snapshot (Drag/Resize)
        self.setMinimumSize(640, 480)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setMouseTracking(True)              # Hover-Cursor ueber den Griffen

    def set_doc(self, doc: FormDoc):
        self.doc = doc
        self.selected = None
        self._pending = None
        self.selection_changed.emit(None)
        self.doc_replaced.emit(doc)
        self._resize_to_doc()
        self.update()

    def _resize_to_doc(self):
        self.setMinimumSize(self.doc.w + 2 * PAD + 40, self.doc.h + 2 * PAD + 40)

    # -- Koordinaten: Canvas -> Control-relativ (Fenster-Inhalt) --
    def _to_ctrl(self, p: QPoint) -> tuple[int, int]:
        return (p.x() - PAD, p.y() - PAD - TITLE_H)

    def _snap(self, v: int) -> int:
        return snap(v) if self.snap_grid else int(v)

    def _handle_points(self, c: Control) -> dict[str, QPoint]:
        """Screen-Mittelpunkte der 8 Resize-Griffe des Controls."""
        x = PAD + c.x
        y = PAD + TITLE_H + c.y
        mx, my = x + c.w // 2, y + c.h // 2
        rx, by = x + c.w, y + c.h
        return {
            "nw": QPoint(x, y), "n": QPoint(mx, y), "ne": QPoint(rx, y),
            "e": QPoint(rx, my), "se": QPoint(rx, by), "s": QPoint(mx, by),
            "sw": QPoint(x, by), "w": QPoint(x, my),
        }

    def _handle_at(self, p: QPoint) -> str | None:
        """Welcher Resize-Griff des selektierten Controls liegt unter `p`?"""
        if self.selected is None:
            return None
        tol = HANDLE
        for name, hp in self._handle_points(self.selected).items():
            if abs(p.x() - hp.x()) <= tol and abs(p.y() - hp.y()) <= tol:
                return name
        return None

    def paintEvent(self, _ev):
        qp = QPainter(self)
        qp.fillRect(self.rect(), QColor(18, 22, 28))
        d = self.doc
        win = QRect(PAD, PAD, d.w, d.h)
        qp.fillRect(win, QColor(24, 34, 46))
        qp.setPen(QPen(QColor(46, 88, 110), 1))
        qp.drawRect(win)
        if self.snap_grid:
            self._paint_grid(qp, d)
        # Titelleiste
        qp.fillRect(QRect(PAD, PAD, d.w, TITLE_H), QColor(18, 90, 120))
        qp.setPen(QColor(230, 247, 255))
        qp.drawText(PAD + 6, PAD + 15, d.title)
        # Controls
        for c in d.controls:
            self._paint_control(qp, c)
        if self.selected is not None:
            self._paint_handles(qp, self.selected)

    def _paint_grid(self, qp: QPainter, d: FormDoc):
        """Dezente Raster-Punkte im Fenster-Inhaltsbereich (unter der Titelleiste)."""
        qp.setPen(QPen(QColor(60, 78, 96), 1))
        x0 = PAD
        y0 = PAD + TITLE_H
        gx = GRID
        while gx <= d.w:
            gy = GRID
            while gy <= d.h - TITLE_H:
                qp.drawPoint(x0 + gx, y0 + gy)
                gy += GRID
            gx += GRID

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
        # Selektionsrahmen
        qp.setPen(QPen(QColor(43, 196, 232), 1, Qt.PenStyle.DashLine))
        qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRect(QRect(x - 1, y - 1, c.w + 2, c.h + 2))
        # 8 Resize-Griffe
        qp.setPen(QPen(QColor(12, 18, 24), 1))
        qp.setBrush(QBrush(QColor(43, 196, 232)))
        s = HANDLE
        for hp in self._handle_points(c).values():
            qp.drawRect(QRect(hp.x() - s // 2, hp.y() - s // 2, s, s))

    # -- Maus --
    def mousePressEvent(self, ev):
        p = ev.position().toPoint()
        cx, cy = self._to_ctrl(p)
        if self.place_kind:
            pre = self.doc.to_dict()                         # Undo: Zustand vor dem Platzieren
            c = self.doc.add(self.place_kind, max(self._snap(cx), 0), max(self._snap(cy), 0))
            self.place_kind = None
            if self.commit_history:
                self.commit_history(pre)
            self._select(c)
            self.doc_changed.emit()
            self.update()
            return
        # Resize-Griff des bereits selektierten Controls?
        handle = self._handle_at(p)
        if handle is not None:
            self._resize_handle = handle
            self._pending = self.doc.to_dict()               # Pre-Resize-Snapshot
            return
        hit = self.doc.control_at(cx, cy)
        self._select(hit)
        if hit is not None:
            self._drag = True
            self._drag_off = QPoint(cx - hit.x, cy - hit.y)
            self._pending = self.doc.to_dict()               # Pre-Drag-Snapshot
        self.update()

    def mouseMoveEvent(self, ev):
        p = ev.position().toPoint()
        cx, cy = self._to_ctrl(p)
        if self._resize_handle is not None and self.selected is not None:
            c = self.selected
            nx, ny = self._snap(cx), self._snap(cy)
            c.x, c.y, c.w, c.h = resize_rect(c.x, c.y, c.w, c.h, self._resize_handle, nx, ny)
            self.selection_changed.emit(c)               # Inspector live aktualisieren
            self.doc_changed.emit()
            self.update()
            return
        if self._drag and self.selected is not None:
            self.selected.x = max(0, self._snap(cx - self._drag_off.x()))
            self.selected.y = max(0, self._snap(cy - self._drag_off.y()))
            self.selection_changed.emit(self.selected)   # Inspector live aktualisieren
            self.doc_changed.emit()
            self.update()
            return
        # Kein Knopf gedrueckt: Hover-Cursor ueber Resize-Griffen
        if not (ev.buttons() & Qt.MouseButton.LeftButton):
            handle = self._handle_at(p)
            self.setCursor(_HANDLE_CURSORS[handle] if handle else Qt.CursorShape.ArrowCursor)

    def mouseReleaseEvent(self, _ev):
        # Gesten-Ende: nur einen Undo-Checkpoint, falls sich wirklich was aenderte.
        if self._pending is not None:
            if self.commit_history and self._pending != self.doc.to_dict():
                self.commit_history(self._pending)
            self._pending = None
        self._drag = False
        self._resize_handle = None

    def mouseDoubleClickEvent(self, ev):
        cx, cy = self._to_ctrl(ev.position().toPoint())
        hit = self.doc.control_at(cx, cy)
        if hit is not None:
            self._drag = False
            self._pending = None        # kein Drag aus dem ersten Klick verschleppen
            self._select(hit)
            self.update()
            self.handler_requested.emit(hit)

    def keyPressEvent(self, ev):
        if ev.key() in (Qt.Key.Key_Delete, Qt.Key.Key_Backspace) and self.selected is not None:
            pre = self.doc.to_dict()                         # Undo: Zustand vor dem Loeschen
            self.doc.remove(self.selected)
            if self.commit_history:
                self.commit_history(pre)
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


class _CodePanel(QWidget):
    """Integrierter Code-Editor: pro Event-Handler ein GameBasic-Body. Die
    Combo listet die Handler des Formulars, der Editor zeigt/aendert den Body
    des gewaehlten (gespeichert in `doc.code[name]`)."""
    edited = Signal()             # Body geaendert
    session_started = Signal()    # ein Handler wurde frisch geladen (Undo-Basis)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc: FormDoc | None = None
        self.current: str | None = None
        self._loading = False

        lay = QVBoxLayout(self)
        top = QHBoxLayout()
        top.addWidget(QLabel("Handler:"))
        self.combo = QComboBox(); self.combo.setMinimumWidth(200)
        top.addWidget(self.combo, 1)
        lay.addLayout(top)
        self.sig = QLabel("")
        self.sig.setStyleSheet("color:#5fb6d6;")
        lay.addWidget(self.sig)
        self.editor = QPlainTextEdit()
        self.editor.setLineWrapMode(QPlainTextEdit.LineWrapMode.NoWrap)
        f = QFont("Consolas"); f.setStyleHint(QFont.StyleHint.Monospace); f.setPointSize(10)
        self.editor.setFont(f)
        self.editor.setTabStopDistance(4 * self.editor.fontMetrics().horizontalAdvance(" "))
        self._hl = None
        if GBHighlighter is not None:
            self._hl = GBHighlighter(self.editor.document())
        lay.addWidget(self.editor, 1)

        self.combo.currentIndexChanged.connect(self._on_combo)
        self.editor.textChanged.connect(self._on_text)
        self._show_empty()

    def set_doc(self, doc: FormDoc):
        self.doc = doc
        self.current = None
        self.refresh()

    def refresh(self):
        """Combo neu aus den Handler-Namen des Formulars fuellen (Auswahl halten)."""
        names = self.doc.handler_names() if self.doc else []
        self._loading = True
        self.combo.clear()
        self.combo.addItems(names)
        self._loading = False
        if self.current in names:
            self.show_handler(self.current)
        elif names:
            self.show_handler(names[0])
        else:
            self._show_empty()

    def show_handler(self, name: str):
        if not self.doc or name not in self.doc.handler_names():
            return
        self._loading = True
        idx = self.combo.findText(name)
        if idx >= 0:
            self.combo.setCurrentIndex(idx)
        self.current = name
        self.sig.setText(f"SUB {name}()      …      END SUB")
        self.editor.setReadOnly(False)
        self.editor.setPlainText(self.doc.code.get(name, ""))
        self._loading = False
        self.session_started.emit()

    def focus_editor(self):
        self.editor.setFocus()

    def detach_highlighter(self):
        """Highlighter vom Dokument loesen -- verhindert einen Use-after-free
        beim Interpreter-Shutdown (QSyntaxHighlighter ueberlebt sonst die
        Teardown-Race von Dokument + QApplication)."""
        if self._hl is not None:
            self._hl.setDocument(None)
            self._hl = None

    def _show_empty(self):
        self._loading = True
        self.current = None
        self.editor.clear()
        self.editor.setReadOnly(True)
        self.sig.setText("(kein Handler — Doppelklick auf ein Control)")
        self._loading = False

    def _on_combo(self):
        if self._loading or not self.doc:
            return
        name = self.combo.currentText()
        if name:
            self.show_handler(name)

    def _on_text(self):
        if self._loading or not self.doc or self.current is None:
            return
        self.doc.code[self.current] = self.editor.toPlainText()
        self.edited.emit()


class _OpenForm:
    """Ein im Designer geoeffnetes Formular mit eigenem Zustand: Dokument, Pfad,
    Undo-Historie und Dirty-Flag. (Undo ist pro Formular -- nicht uebergreifend.)"""
    def __init__(self, doc: FormDoc, path: Path | None = None):
        self.doc = doc
        self.path = path
        self.history = History()
        self.dirty = False


class FormDesigner(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = Path(project_root)
        # Multi-Form-Zustand
        self.forms: list[_OpenForm] = []
        self.active_index: int = -1
        self.project = FormProject()
        self.project_path: Path | None = None
        self._suppress_row = False
        self.setWindowTitle("GameBasic Form-Designer")
        self.resize(1100, 740)

        self.canvas = _Canvas()
        scroll = QScrollArea(); scroll.setWidget(self.canvas); scroll.setWidgetResizable(False)
        self.setCentralWidget(scroll)

        # Formular-Liste (Multi-Form-Navigator)
        self.form_list = QListWidget()
        self.form_list.currentRowChanged.connect(self._on_form_row)
        self._dock("Formulare", self.form_list, Qt.DockWidgetArea.LeftDockWidgetArea)

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

        # Code-Editor (integriert)
        self.code_panel = _CodePanel()
        self.code_dock = self._dock("Code", self.code_panel, Qt.DockWidgetArea.BottomDockWidgetArea)

        # Undo/Redo-Edit-Sessions (transient, pro Selektion/Handler)
        self.canvas.commit_history = self._commit_history
        self._insp_baseline: dict | None = None
        self._insp_dirty = False
        self._code_baseline: dict | None = None
        self._code_dirty = False

        self.canvas.selection_changed.connect(self.inspector.set_control)
        self.canvas.selection_changed.connect(self._on_selection_changed)
        self.canvas.doc_changed.connect(self._mark_dirty)
        self.canvas.doc_replaced.connect(self.code_panel.set_doc)
        self.canvas.handler_requested.connect(self._open_handler)
        self.inspector.changed.connect(self._on_inspector_changed)
        self.code_panel.session_started.connect(self._on_code_session)
        self.code_panel.edited.connect(self._on_code_edited)

        self._build_menu()
        self._add_open_form(FormDoc())     # ein leeres Start-Formular

    def closeEvent(self, ev):
        self.code_panel.detach_highlighter()
        super().closeEvent(ev)

    def _dock(self, title, widget, area):
        d = QDockWidget(title, self)
        d.setWidget(widget)
        self.addDockWidget(area, d)
        return d

    def _build_menu(self):
        m = self.menuBar().addMenu("&Datei")
        def act(name, shortcut, fn, menu=m):
            a = QAction(name, self)
            if shortcut:
                a.setShortcut(QKeySequence(shortcut))
            a.triggered.connect(fn); menu.addAction(a); return a
        act("Neues Formular", "Ctrl+N", self.new_form)
        act("Formular oeffnen...", "Ctrl+O", self.open_form)
        act("Formular schliessen", "Ctrl+W", self.close_form)
        m.addSeparator()
        act("Speichern", "Ctrl+S", self.save_form)
        act("Speichern unter...", "Ctrl+Shift+S", self.save_form_as)
        act("Alle speichern", "Ctrl+Alt+S", self.save_all)
        m.addSeparator()
        act("Projekt oeffnen...", None, self.open_project)
        act("Projekt speichern...", None, self.save_project)
        act("Als Startformular setzen", None, self.set_main_form)
        m.addSeparator()
        act("Ausfuehren (gbrt)", "F5", self.run_form)

        e = self.menuBar().addMenu("&Bearbeiten")
        self.act_undo = act("Rueckgaengig", "Ctrl+Z", self.undo, menu=e)
        self.act_redo = act("Wiederholen", "Ctrl+Y", self.redo, menu=e)
        # Zweites, uebliches Redo-Kuerzel (Strg+Umschalt+Z)
        redo2 = QAction(self)
        redo2.setShortcut(QKeySequence("Ctrl+Shift+Z"))
        redo2.triggered.connect(self.redo)
        self.addAction(redo2)

        v = self.menuBar().addMenu("&Ansicht")
        self.act_snap = QAction("Am Raster ausrichten", self, checkable=True)
        self.act_snap.setChecked(self.canvas.snap_grid)
        self.act_snap.setShortcut(QKeySequence("Ctrl+G"))
        self.act_snap.toggled.connect(self._toggle_snap)
        v.addAction(self.act_snap)

    def _toggle_snap(self, on: bool):
        self.canvas.snap_grid = on
        self.canvas.update()

    # -- Aktive Form + Navigator ------------------------------------------
    @property
    def active(self) -> _OpenForm:
        return self.forms[self.active_index]

    @property
    def history(self) -> History:
        return self.active.history

    @property
    def path(self) -> Path | None:
        return self.active.path if 0 <= self.active_index < len(self.forms) else None

    @path.setter
    def path(self, v):
        self.active.path = v

    def _add_open_form(self, doc: FormDoc, path: Path | None = None, switch: bool = True):
        of = _OpenForm(doc, path)
        self.forms.append(of)
        if switch:
            self._switch_to(len(self.forms) - 1)
        else:
            self._refresh_form_list()
        return of

    def _switch_to(self, index: int):
        if not (0 <= index < len(self.forms)):
            return
        self.active_index = index
        self._insp_baseline = None; self._insp_dirty = False
        self._code_baseline = None; self._code_dirty = False
        self.canvas.set_doc(self.active.doc)   # doc_replaced -> Code-Panel
        self._refresh_history_actions()
        self._refresh_form_list()
        self._update_title()

    def _set_active_doc(self, doc: FormDoc):
        """Aktives Dokument ersetzen (Undo/Redo) -- haelt forms + canvas synchron."""
        self.active.doc = doc
        self.canvas.set_doc(doc)

    def _refresh_form_list(self):
        self._suppress_row = True
        self.form_list.clear()
        for of in self.forms:
            title = of.doc.title or "(Formular)"
            star = " *" if of.dirty else ""
            crown = "★ " if (of.path and self.project.main and
                             self._rel(of.path) == self.project.main) else ""
            self.form_list.addItem(f"{crown}{title}{star}")
        if 0 <= self.active_index < len(self.forms):
            self.form_list.setCurrentRow(self.active_index)
        self._suppress_row = False

    def _on_form_row(self, row: int):
        if self._suppress_row or row < 0 or row == self.active_index:
            return
        self._switch_to(row)

    def _rel(self, p: Path) -> str:
        """Pfad relativ zum Projekt-Verzeichnis (fuer das `.gbproj`-Manifest)."""
        if self.project_path is not None:
            try:
                return str(Path(p).relative_to(self.project_path.parent)).replace("\\", "/")
            except ValueError:
                pass
        return Path(p).name

    # -- Undo/Redo --
    def _commit_history(self, snapshot: dict):
        """Vom Canvas gerufen (Platzieren/Loeschen/Drag/Resize): Checkpoint setzen."""
        self.history.push(snapshot)
        self._refresh_history_actions()

    def _on_selection_changed(self, _c):
        """Neue Selektion -> Basis fuer eine evtl. folgende Inspector-Edit-Session."""
        self._insp_baseline = self.canvas.doc.to_dict()
        self._insp_dirty = False

    def _on_inspector_changed(self):
        """Inspector-Aenderung. Alle Edits einer Selektion = EIN Undo-Schritt."""
        if not self._insp_dirty and self._insp_baseline is not None:
            self.history.push(self._insp_baseline)
            self._insp_dirty = True
            self._refresh_history_actions()
        self.canvas.update()
        self.code_panel.refresh()      # evtl. umbenannte Handler in die Combo uebernehmen
        self._mark_dirty()

    def _refresh_history_actions(self):
        self.act_undo.setEnabled(self.history.can_undo)
        self.act_redo.setEnabled(self.history.can_redo)

    def undo(self):
        if not self.history.can_undo:
            return
        prev = self.history.undo(self.canvas.doc.to_dict())
        self._set_active_doc(FormDoc.from_dict(prev))
        self._refresh_history_actions()
        self._mark_dirty()

    def redo(self):
        if not self.history.can_redo:
            return
        nxt = self.history.redo(self.canvas.doc.to_dict())
        self._set_active_doc(FormDoc.from_dict(nxt))
        self._refresh_history_actions()
        self._mark_dirty()

    # -- Code-Editor --
    def _open_handler(self, c: Control):
        """Doppelklick auf ein Control: Handler anlegen/anspringen + fokussieren."""
        if self.canvas.doc.primary_event(c) is None:
            self.statusBar().showMessage(f"{c.kind}: kein Event-Handler moeglich.", 3000)
            return
        pre = self.canvas.doc.to_dict()
        name = self.canvas.doc.ensure_handler(c)
        if pre != self.canvas.doc.to_dict():
            self._commit_history(pre)             # Handler-Erzeugung = Undo-Schritt
        self.code_panel.refresh()
        self.code_panel.show_handler(name)
        self.inspector.set_control(c)             # neuer Handler-Name im Inspector
        self.canvas.update()
        self.code_dock.show(); self.code_dock.raise_()
        self.code_panel.focus_editor()
        self._mark_dirty()

    def _on_code_session(self):
        self._code_baseline = self.canvas.doc.to_dict()
        self._code_dirty = False

    def _on_code_edited(self):
        """Code-Edits einer Handler-Sitzung = EIN Undo-Schritt."""
        if not self._code_dirty and self._code_baseline is not None:
            self.history.push(self._code_baseline)
            self._code_dirty = True
            self._refresh_history_actions()
        self._mark_dirty()

    # -- Aktionen --
    def _mark_dirty(self):
        was = self.active.dirty
        self.active.dirty = True
        self._update_title()
        if not was:
            self._refresh_form_list()    # nur bei Uebergang -> Stern erscheint

    def _update_title(self):
        of = self.active
        name = of.path.name if of.path else "(unbenannt)"
        star = "*" if of.dirty else ""
        proj = f"  [{self.project_path.name}]" if self.project_path else ""
        self.setWindowTitle(f"GameBasic Form-Designer{proj}  --  {name}{star}")

    def new_form(self):
        self._add_open_form(FormDoc())

    def open_form(self):
        fn, _ = QFileDialog.getOpenFileName(self, "Formular oeffnen", str(self.project_root),
                                            "GameBasic-Form (*.gbform);;Alle (*.*)")
        if not fn:
            return
        try:
            self._add_open_form(FormDoc.load(fn), Path(fn))
        except Exception as e:  # noqa: BLE001
            QMessageBox.critical(self, "Fehler", f"Konnte nicht laden:\n{e}")

    def close_form(self):
        if self.active.dirty:
            r = QMessageBox.question(self, "Schliessen",
                                     "Ungespeicherte Aenderungen verwerfen?",
                                     QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No)
            if r != QMessageBox.StandardButton.Yes:
                return
        i = self.active_index
        del self.forms[i]
        if not self.forms:
            self._add_open_form(FormDoc())        # nie ganz leer
        else:
            self._switch_to(min(i, len(self.forms) - 1))

    def save_form(self):
        if self.path is None:
            return self.save_form_as()
        self.canvas.doc.save(str(self.path))
        self.active.dirty = False
        self._update_title(); self._refresh_form_list()
        return True

    def save_form_as(self):
        fn, _ = QFileDialog.getSaveFileName(self, "Formular speichern", str(self.project_root),
                                            "GameBasic-Form (*.gbform)")
        if not fn:
            return False
        if not fn.endswith(".gbform"):
            fn += ".gbform"
        self.path = Path(fn)
        self.canvas.doc.save(fn)
        self.active.dirty = False
        self._update_title(); self._refresh_form_list()
        return True

    def save_all(self):
        start = self.active_index
        for i in range(len(self.forms)):
            if self.forms[i].dirty or self.forms[i].path is None:
                self._switch_to(i)
                if not self.save_form():
                    break                          # Abbruch im Speichern-Dialog
        self._switch_to(min(start, len(self.forms) - 1))

    def set_main_form(self):
        if self.path is None:
            self.statusBar().showMessage("Formular zuerst speichern.", 3000)
            return
        self.project.main = self._rel(self.path)
        self._refresh_form_list()
        self.statusBar().showMessage(f"Startformular: {self.project.main}", 3000)

    # -- Projekt (.gbproj) --
    def open_project(self):
        fn, _ = QFileDialog.getOpenFileName(self, "Projekt oeffnen", str(self.project_root),
                                            "GameBasic-Projekt (*.gbproj)")
        if fn:
            self.load_project_file(fn)

    def load_project_file(self, fn: str):
        try:
            proj = FormProject.load(fn)
        except Exception as e:  # noqa: BLE001
            QMessageBox.critical(self, "Fehler", f"Projekt nicht ladbar:\n{e}")
            return
        base = Path(fn).parent
        self.project = proj
        self.project_path = Path(fn)
        self.forms = []
        self.active_index = -1
        main_idx = 0
        for i, rel in enumerate(proj.forms):
            try:
                doc = FormDoc.load(str(base / rel))
            except Exception:  # noqa: BLE001 -- fehlende Form ueberspringen
                continue
            self._add_open_form(doc, base / rel, switch=False)
            if rel == proj.main:
                main_idx = len(self.forms) - 1
        if not self.forms:
            self._add_open_form(FormDoc(), switch=False)
        self._switch_to(min(main_idx, len(self.forms) - 1))

    def save_project(self):
        # Erst sicherstellen, dass jede Form einen Pfad hat (+ gespeichert ist).
        start = self.active_index
        for i in range(len(self.forms)):
            if self.forms[i].path is None:
                self._switch_to(i)
                if not self.save_form_as():
                    self._switch_to(start)
                    return
        if self.project_path is None:
            fn, _ = QFileDialog.getSaveFileName(self, "Projekt speichern", str(self.project_root),
                                                "GameBasic-Projekt (*.gbproj)")
            if not fn:
                self._switch_to(start)
                return
            if not fn.endswith(".gbproj"):
                fn += ".gbproj"
            self.project_path = Path(fn)
        # Alle Forms speichern + Manifest aufbauen (relativ zum Projektpfad).
        self.project.forms = []
        for of in self.forms:
            of.doc.save(str(of.path)); of.dirty = False
            self.project.add(self._rel(of.path))
        if self.project.main not in self.project.forms:
            self.project.main = self.project.forms[0] if self.project.forms else ""
        self.project.save(str(self.project_path))
        self._switch_to(min(start, len(self.forms) - 1))
        self.statusBar().showMessage(f"Projekt gespeichert: {self.project_path.name}", 3000)

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
        p = Path(initial_file)
        try:
            if p.suffix == ".gbproj":
                win.load_project_file(str(p))
            else:
                # leeres Start-Formular durch die geladene Datei ersetzen
                doc = FormDoc.load(str(p))
                of = win.forms[0]
                of.doc = doc; of.path = p
                win._switch_to(0)
        except Exception:
            pass
    win.show()
    return app.exec()
