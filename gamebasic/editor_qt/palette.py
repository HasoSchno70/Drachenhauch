"""Quick-Open + Command-Palette: gemeinsamer Fuzzy-Picker-Dialog.

Beide Funktionen tippen denselben Workflow ab: Suchfeld oben, gefilterte
Ergebnisliste darunter, Pfeil-Up/Down navigiert, Enter waehlt aus, Esc
schliesst. Wir benutzen ein gemeinsames Dialog-Skelett `_PickerDialog`
und parametrisieren es ueber `entries` + Activation-Callback.
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QKeyEvent
from PySide6.QtWidgets import (
    QDialog, QLineEdit, QListWidget, QListWidgetItem, QMessageBox, QVBoxLayout,
    QWidget,
)

from .fuzzy import score
from .theme import COLORS


@dataclass
class PickerEntry:
    label: str               # Sichtbarer Text
    detail: str = ""         # Sekundaer-Info, halbtransparent rechts
    user_data: object = None # beliebiger Wert -> Activation-Callback


class _PickerDialog(QDialog):
    """Frameless modaler Picker am oberen Fenster-Drittel."""

    chosen = Signal(object)   # PickerEntry

    def __init__(self, parent: QWidget, placeholder: str, entries: list[PickerEntry]):
        super().__init__(parent)
        self._all_entries = entries
        self.setWindowFlag(Qt.WindowType.FramelessWindowHint, True)
        self.setWindowFlag(Qt.WindowType.Popup, True)
        self.setModal(True)
        self.setFixedWidth(640)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self.entry = QLineEdit()
        self.entry.setPlaceholderText(placeholder)
        self.entry.textChanged.connect(self._refilter)
        self.entry.installEventFilter(self)
        layout.addWidget(self.entry)

        self.list = QListWidget()
        # Review-Fund (waehrend Testerstellung entdeckt, nicht im urspruengl.
        # Review): `setUniformRowHeights` gibt es nur auf QTreeView/
        # QTreeWidget -- `self.list` ist ein QListWidget (QListView-Basis),
        # dessen Aequivalent `setUniformItemSizes` heisst. Der falsche
        # Methodenname liess JEDEN Aufruf von command_palette()/quick_open()
        # sofort mit AttributeError in __init__ abstuerzen -- Command
        # Palette und Quick-Open waren dadurch komplett unbenutzbar.
        self.list.setUniformItemSizes(True)
        self.list.itemActivated.connect(self._activate)
        layout.addWidget(self.list)

        self._apply_style()
        self._refilter("")

    def _apply_style(self) -> None:
        c = COLORS
        self.setStyleSheet(
            f"""
            _PickerDialog {{ background-color: {c['bg_panel']}; border: 1px solid {c['border']}; }}
            QLineEdit {{
                background-color: {c['bg_panel']};
                color: {c['fg']};
                border: 0;
                border-bottom: 1px solid {c['border']};
                padding: 10px 14px;
                font-size: 12pt;
            }}
            QListWidget {{
                background-color: {c['bg_panel']};
                color: {c['fg']};
                border: 0;
            }}
            QListWidget::item {{ padding: 6px 14px; }}
            QListWidget::item:selected {{ background-color: {c['sel']}; }}
            """
        )

    def _refilter(self, _q: str = "") -> None:
        q = self.entry.text()
        self.list.clear()
        if not q:
            entries = self._all_entries
            scores = [(0, e) for e in entries]
        else:
            scored: list[tuple[int, PickerEntry]] = []
            for e in self._all_entries:
                s_label = score(q, e.label)
                s_detail = score(q, e.detail) if e.detail else None
                if s_label is None and s_detail is None:
                    continue
                best = max(x for x in (s_label, s_detail) if x is not None)
                scored.append((best, e))
            scored.sort(key=lambda t: -t[0])
            scores = scored
        for s, e in scores[:200]:
            text = e.label if not e.detail else f"{e.label}    {e.detail}"
            it = QListWidgetItem(text)
            it.setData(Qt.ItemDataRole.UserRole, e)
            if e.detail:
                it.setForeground(QColor(COLORS["fg"]))
            self.list.addItem(it)
        if self.list.count() > 0:
            self.list.setCurrentRow(0)

    def _activate(self, item: QListWidgetItem) -> None:
        e = item.data(Qt.ItemDataRole.UserRole)
        if isinstance(e, PickerEntry):
            self.chosen.emit(e)
        self.accept()

    def eventFilter(self, obj, ev):  # noqa: N802
        # Pfeil-Up/Down + Enter aus dem Eingabefeld an die Liste durchreichen.
        if obj is self.entry and ev.type() == ev.Type.KeyPress:
            if isinstance(ev, QKeyEvent):
                k = ev.key()
                if k == Qt.Key.Key_Down:
                    row = min(self.list.currentRow() + 1, self.list.count() - 1)
                    self.list.setCurrentRow(row)
                    return True
                if k == Qt.Key.Key_Up:
                    row = max(self.list.currentRow() - 1, 0)
                    self.list.setCurrentRow(row)
                    return True
                if k in (Qt.Key.Key_Return, Qt.Key.Key_Enter):
                    cur = self.list.currentItem()
                    if cur is not None:
                        self._activate(cur)
                    return True
                if k == Qt.Key.Key_Escape:
                    self.reject()
                    return True
        return super().eventFilter(obj, ev)


def quick_open(parent: QWidget, project_root: Path,
               on_open: Callable[[Path], None]) -> None:
    """Sammelt alle .gb-Dateien im Projekt und zeigt einen Fuzzy-Picker.

    Aufrufer setzt `on_open(path)` als Callback fuer die Auswahl.
    """
    files = sorted(
        p for p in project_root.rglob("*.gb")
        if not p.name.startswith("_") and ".venv" not in p.parts
    )
    entries: list[PickerEntry] = []
    for p in files:
        try:
            rel = p.relative_to(project_root)
        except ValueError:
            rel = p
        rel_s = str(rel).replace("\\", "/")
        entries.append(PickerEntry(
            label=p.name,
            detail=str(rel.parent).replace("\\", "/") if rel.parent != Path(".") else "",
            user_data=p,
        ))
    dlg = _PickerDialog(parent, "Datei oeffnen ...", entries)

    def _open(e: PickerEntry) -> None:
        # Review-Fund: rief on_open() bisher ungeschuetzt aus dem Signal-
        # Handler auf -- eine Exception darin (z.B. Datei zwischen Scan und
        # Klick geloescht/gesperrt) liefe unbehandelt durch Qts Signal-
        # Dispatch (innerhalb von dlg.exec()s verschachtelter Event-Loop)
        # statt eine verstaendliche Fehlermeldung zu zeigen.
        try:
            on_open(e.user_data)
        except Exception as exc:  # noqa: BLE001 -- User soll eine Meldung sehen, kein Traceback
            QMessageBox.warning(parent, "Datei oeffnen fehlgeschlagen", str(exc))

    dlg.chosen.connect(_open)
    _show_centered(dlg, parent)
    dlg.exec()


def command_palette(parent: QWidget, commands: list[tuple[str, str, Callable]]) -> None:
    """Zeigt eine Auswahl aller Befehle.

    `commands` ist `[(label, shortcut_text, callback)]`.
    """
    entries = [
        PickerEntry(label=lbl, detail=sc, user_data=cb)
        for lbl, sc, cb in commands
    ]
    dlg = _PickerDialog(parent, "Befehl ausfuehren ...", entries)

    def _run(e: PickerEntry):
        cb = e.user_data
        if not callable(cb):
            return
        # Review-Fund: rief den Befehl bisher ungeschuetzt auf -- eine
        # Exception darin lief unbehandelt durch Qts Signal-Dispatch
        # (innerhalb von dlg.exec()s verschachtelter Event-Loop), statt
        # eine verstaendliche Fehlermeldung zu zeigen.
        try:
            cb()
        except Exception as exc:  # noqa: BLE001 -- User soll eine Meldung sehen, kein Traceback
            QMessageBox.warning(parent, "Befehl fehlgeschlagen", str(exc))

    dlg.chosen.connect(_run)
    _show_centered(dlg, parent)
    dlg.exec()


def _show_centered(dlg: _PickerDialog, parent: QWidget) -> None:
    """Zentriert den Picker horizontal im Parent, vertikal im oberen Drittel."""
    pg = parent.geometry()
    x = pg.x() + (pg.width() - dlg.width()) // 2
    y = pg.y() + max(60, pg.height() // 5)
    dlg.move(x, y)
