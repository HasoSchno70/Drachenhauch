"""FindInProject: nicht-modaler Suchen-Dialog ueber alle .gb-Dateien.

Suche laeuft im Worker-Thread (`QThread`), Treffer trickeln per Signal
in die Tree-View und sind sofort klickbar. Optionen wie im
`FindReplaceDialog`: Gross/Klein, Whole-Word, Regex.

Doppelklick auf einen Treffer oeffnet die Datei und springt zur Zeile.
"""
from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from PySide6.QtCore import QObject, Qt, QThread, Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import (
    QCheckBox, QDialog, QDialogButtonBox, QGridLayout, QHBoxLayout, QLabel,
    QLineEdit, QPushButton, QTreeWidget, QTreeWidgetItem, QVBoxLayout, QWidget,
)

from .theme import COLORS


@dataclass
class _Hit:
    file: Path
    line: int
    text: str


class _Worker(QObject):
    hit = Signal(object)        # _Hit
    finished = Signal(int)      # Total-Treffer

    def __init__(self, root: Path, query: str, *, case_sensitive: bool,
                 whole_word: bool, regex: bool):
        super().__init__()
        self.root = root
        self.query = query
        self.case_sensitive = case_sensitive
        self.whole_word = whole_word
        self.regex = regex
        self._abort = False

    def abort(self) -> None:
        self._abort = True

    def run(self) -> None:
        try:
            pat_str = self.query if self.regex else re.escape(self.query)
            if self.whole_word:
                pat_str = rf"\b{pat_str}\b"
            flags = 0 if self.case_sensitive else re.IGNORECASE
            pattern = re.compile(pat_str, flags)
        except re.error:
            self.finished.emit(0)
            return

        total = 0
        for path in sorted(self.root.rglob("*.gb")):
            if self._abort:
                break
            if ".venv" in path.parts or path.name.startswith("_"):
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            for ln_idx, line in enumerate(text.split("\n"), start=1):
                if self._abort:
                    break
                if pattern.search(line):
                    self.hit.emit(_Hit(file=path, line=ln_idx, text=line.rstrip()))
                    total += 1
        self.finished.emit(total)


class FindInProjectDialog(QDialog):
    open_requested = Signal(Path, int)   # Datei + Zeile

    def __init__(self, parent: QWidget, project_root: Path):
        super().__init__(parent)
        self.project_root = project_root
        self._thread: QThread | None = None
        self._worker: _Worker | None = None

        self.setWindowTitle("Im Projekt suchen")
        self.setWindowFlag(Qt.WindowType.Tool, True)
        self.setModal(False)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(8)

        # Eingabezeile + Optionen
        row = QHBoxLayout()
        self.entry = QLineEdit()
        self.entry.setPlaceholderText("Suchen ...")
        self.entry.returnPressed.connect(self._start)
        row.addWidget(self.entry, 1)
        self.btn_go = QPushButton("Suchen")
        self.btn_go.clicked.connect(self._start)
        row.addWidget(self.btn_go)
        self.btn_stop = QPushButton("Stop")
        self.btn_stop.clicked.connect(self._abort)
        self.btn_stop.setEnabled(False)
        row.addWidget(self.btn_stop)
        layout.addLayout(row)

        opts = QHBoxLayout()
        opts.setSpacing(12)
        self.cb_case = QCheckBox("Gross/Klein")
        self.cb_word = QCheckBox("Ganzes Wort")
        self.cb_regex = QCheckBox("Regex")
        for cb in (self.cb_case, self.cb_word, self.cb_regex):
            opts.addWidget(cb)
        opts.addStretch(1)
        layout.addLayout(opts)

        self.tree = QTreeWidget()
        self.tree.setHeaderLabels(["Datei / Zeile", "Vorschau"])
        self.tree.setColumnWidth(0, 280)
        self.tree.itemActivated.connect(self._on_activated)
        self.tree.itemDoubleClicked.connect(self._on_activated)
        layout.addWidget(self.tree, 1)

        self.status = QLabel("")
        self.status.setStyleSheet(f"color: {COLORS['fg_muted']};")
        layout.addWidget(self.status)

        bb = QDialogButtonBox(QDialogButtonBox.StandardButton.Close, parent=self)
        bb.rejected.connect(self.close)
        bb.button(QDialogButtonBox.StandardButton.Close).clicked.connect(self.close)
        layout.addWidget(bb)

        self.resize(820, 540)

        self._files_seen: dict[str, QTreeWidgetItem] = {}

    def show_for(self, prefill: str | None = None) -> None:
        if prefill is not None:
            self.entry.setText(prefill)
        self.show()
        self.raise_()
        self.activateWindow()
        self.entry.setFocus()
        self.entry.selectAll()

    def closeEvent(self, ev):  # noqa: N802
        self._abort()
        super().closeEvent(ev)

    # ---------------------------------------------------- Suche
    def _start(self) -> None:
        q = self.entry.text()
        if not q:
            return
        self._abort()
        self.tree.clear()
        self._files_seen.clear()
        self.status.setText("Suche laeuft ...")
        self.btn_go.setEnabled(False)
        self.btn_stop.setEnabled(True)

        self._worker = _Worker(
            self.project_root, q,
            case_sensitive=self.cb_case.isChecked(),
            whole_word=self.cb_word.isChecked(),
            regex=self.cb_regex.isChecked(),
        )
        self._thread = QThread(self)
        self._worker.moveToThread(self._thread)
        self._thread.started.connect(self._worker.run)
        self._worker.hit.connect(self._on_hit)
        self._worker.finished.connect(self._on_finished)
        # Cleanup-Reihenfolge: worker.finished -> thread.quit -> thread.deleteLater.
        self._worker.finished.connect(self._thread.quit)
        self._thread.finished.connect(self._cleanup_thread)
        self._thread.start()

    def _abort(self) -> None:
        if self._worker is not None:
            self._worker.abort()
        if self._thread is not None and self._thread.isRunning():
            self._thread.quit()
            self._thread.wait(2000)
        self._cleanup_thread()
        self.btn_go.setEnabled(True)
        self.btn_stop.setEnabled(False)

    def _cleanup_thread(self) -> None:
        if self._thread is not None:
            self._thread.deleteLater()
            self._thread = None
        self._worker = None

    def _on_hit(self, hit: _Hit) -> None:
        try:
            rel = hit.file.relative_to(self.project_root)
        except ValueError:
            rel = hit.file
        key = str(rel)
        parent = self._files_seen.get(key)
        if parent is None:
            parent = QTreeWidgetItem(self.tree, [str(rel).replace("\\", "/"), ""])
            parent.setExpanded(True)
            self._files_seen[key] = parent
        child = QTreeWidgetItem(parent, [f"Zeile {hit.line}", hit.text])
        child.setData(0, Qt.ItemDataRole.UserRole, (hit.file, hit.line))
        child.setForeground(1, QColor(COLORS["fg_muted"]))

    def _on_finished(self, total: int) -> None:
        self.status.setText(f"{total} Treffer.")
        self.btn_go.setEnabled(True)
        self.btn_stop.setEnabled(False)

    def _on_activated(self, item: QTreeWidgetItem, _col: int = 0) -> None:
        data = item.data(0, Qt.ItemDataRole.UserRole)
        if isinstance(data, tuple):
            file, line = data
            self.open_requested.emit(Path(file), int(line))
