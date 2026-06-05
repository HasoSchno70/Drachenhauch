"""Snapshot-basierte Undo/Redo-Historie fuer die Begleit-Tools.

Partikel-Editor, SFX-Generator und Tracker haben kein eigenes Command-Modell;
ihr Zustand sind ein paar Widgets bzw. ein serialisierbares Datenmodell. Statt
jede Aenderung als Command zu fassen, snapshotten wir den GANZEN Zustand:

    undo = SnapshotUndo(capture, restore)        # capture()->Snapshot, restore(s)
    ...auf jede Aenderung:  undo.mark()           # debounced -> ein Undo-Schritt
    Strg+Z: undo.undo()    Strg+Y: undo.redo()

`mark()` ist *debounced*: rasch aufeinander folgende Aenderungen (z.B. ein
Slider-Drag oder das gleichzeitige Setzen mehrerer Felder durch ein Preset)
werden zu EINEM Undo-Schritt zusammengefasst. Beim `restore()` wird das
Aufzeichnen unterdrueckt, damit das programmatische Zuruecksetzen der Widgets
keinen neuen Schritt erzeugt.
"""
from __future__ import annotations

from typing import Any, Callable

from PySide6.QtCore import QObject, QTimer, Signal


class SnapshotUndo(QObject):
    """Generische Undo/Redo-Historie ueber Zustands-Snapshots.

    `capture()` liefert einen (per `==` vergleichbaren, unveraenderlichen bzw.
    nicht weiter mutierten) Snapshot des aktuellen Zustands; `restore(snap)`
    setzt den Zustand zurueck. `changed` feuert nach jeder Aenderung der
    Stacks (fuer Button-Enabled-State).
    """

    changed = Signal()

    def __init__(self, capture: Callable[[], Any], restore: Callable[[Any], None],
                 *, limit: int = 100, debounce_ms: int = 250,
                 parent: QObject | None = None):
        super().__init__(parent)
        self._capture = capture
        self._restore = restore
        self._limit = limit
        self._undo: list[Any] = []
        self._redo: list[Any] = []
        self._baseline = capture()
        self._suspend = False
        self._timer = QTimer(self)
        self._timer.setSingleShot(True)
        self._timer.setInterval(max(0, debounce_ms))
        self._timer.timeout.connect(self._commit)

    # -------------------------------------------------- Aufzeichnen
    def mark(self) -> None:
        """Eine moegliche Aenderung melden -- debounced eingebucht."""
        if self._suspend:
            return
        self._timer.start()

    def _commit(self) -> None:
        snap = self._capture()
        if snap == self._baseline:
            return
        self._undo.append(self._baseline)
        if len(self._undo) > self._limit:
            self._undo.pop(0)
        self._baseline = snap
        self._redo.clear()
        self.changed.emit()

    def flush(self) -> None:
        """Einen ausstehenden debounce-Commit sofort ausfuehren."""
        if self._timer.isActive():
            self._timer.stop()
            self._commit()

    def reset(self) -> None:
        """Historie verwerfen und den aktuellen Zustand als Basis nehmen
        (z.B. nach Neu/Oeffnen einer Datei)."""
        self._timer.stop()
        self._undo.clear()
        self._redo.clear()
        self._baseline = self._capture()
        self.changed.emit()

    # -------------------------------------------------- Abfragen
    def can_undo(self) -> bool:
        return bool(self._undo) or self._timer.isActive()

    def can_redo(self) -> bool:
        return bool(self._redo)

    # -------------------------------------------------- Undo/Redo
    def undo(self) -> None:
        self.flush()
        if not self._undo:
            return
        self._redo.append(self._baseline)
        self._baseline = self._undo.pop()
        self._apply(self._baseline)

    def redo(self) -> None:
        if not self._redo:
            return
        self._undo.append(self._baseline)
        self._baseline = self._redo.pop()
        self._apply(self._baseline)

    def _apply(self, snap: Any) -> None:
        self._suspend = True
        try:
            self._restore(snap)
        finally:
            self._suspend = False
        self.changed.emit()
