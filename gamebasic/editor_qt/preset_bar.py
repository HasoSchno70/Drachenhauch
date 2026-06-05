"""Qt-Widget fuer die Preset-Bibliothek (Partikel/SFX-Editor).

Eine schmale Leiste: Combo der Preset-Namen + „Speichern unter..." +
„Loeschen". Bekommt eine `PresetLibrary` plus zwei Callbacks:

- ``get_state()`` -> Parameter-Dict des aktuellen Editor-Zustands
- ``apply_state(dict)`` -> setzt den Editor-Zustand aus einem Preset

So bleibt das Widget editor-agnostisch und wird von beiden Tools genutzt.
"""
from __future__ import annotations

from PySide6.QtWidgets import (
    QComboBox, QHBoxLayout, QInputDialog, QLabel, QMessageBox, QPushButton,
    QWidget,
)

from .preset_library import PresetLibrary


class PresetBar(QWidget):
    def __init__(self, library: PresetLibrary, get_state, apply_state,
                 parent=None):
        super().__init__(parent)
        self._lib = library
        self._get_state = get_state
        self._apply_state = apply_state

        row = QHBoxLayout(self)
        row.setContentsMargins(0, 0, 0, 0)
        row.addWidget(QLabel("Preset-Bibliothek:"))
        self.combo = QComboBox()
        self.combo.setMinimumWidth(160)
        self.combo.activated.connect(self._on_selected)
        row.addWidget(self.combo)
        self.btn_save = QPushButton("Speichern unter...")
        self.btn_save.clicked.connect(self._on_save)
        row.addWidget(self.btn_save)
        self.btn_del = QPushButton("Loeschen")
        self.btn_del.clicked.connect(self._on_delete)
        row.addWidget(self.btn_del)
        row.addStretch(1)

        self._refresh()

    # ------------------------------------------------------------ intern
    def _refresh(self, select: str | None = None) -> None:
        self.combo.blockSignals(True)
        self.combo.clear()
        self.combo.addItem("— Preset waehlen —", userData=None)
        for name in self._lib.names():
            label = f"★ {name}" if self._lib.is_builtin(name) else name
            self.combo.addItem(label, userData=name)
        if select is not None:
            i = self.combo.findData(select)
            if i >= 0:
                self.combo.setCurrentIndex(i)
        self.combo.blockSignals(False)
        self._update_del_button()

    def _update_del_button(self) -> None:
        name = self.combo.currentData()
        self.btn_del.setEnabled(bool(name) and not self._lib.is_builtin(name))

    def _on_selected(self, _index: int) -> None:
        name = self.combo.currentData()
        self._update_del_button()
        if not name:
            return
        state = self._lib.get(name)
        if state is not None:
            self._apply_state(state)

    def _on_save(self) -> None:
        cur = self.combo.currentData() or ""
        name, ok = QInputDialog.getText(
            self, "Preset speichern", "Name:", text=str(cur))
        if not ok:
            return
        name = name.strip()
        if not name:
            return
        if self._lib.is_builtin(name):
            QMessageBox.warning(
                self, "Werks-Preset",
                f'"{name}" ist ein Werks-Preset und kann nicht '
                'ueberschrieben werden. Bitte einen anderen Namen waehlen.')
            return
        if name in self._lib.user:
            ans = QMessageBox.question(
                self, "Ueberschreiben?",
                f'Preset "{name}" existiert bereits. Ueberschreiben?')
            if ans != QMessageBox.StandardButton.Yes:
                return
        self._lib.add(name, self._get_state())
        self._refresh(select=name)

    def _on_delete(self) -> None:
        name = self.combo.currentData()
        if not name or self._lib.is_builtin(name):
            return
        ans = QMessageBox.question(
            self, "Preset loeschen", f'Preset "{name}" loeschen?')
        if ans != QMessageBox.StandardButton.Yes:
            return
        self._lib.remove(name)
        self._refresh()
