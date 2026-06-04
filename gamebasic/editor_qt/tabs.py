"""Tab-Bereich mit mehreren Editor-Buffers (`QTabWidget`).

Jeder Tab haelt einen `CodeEditor` plus Pfad-Metadaten (`TabState`).
Dirty-Tracking laeuft ueber `QTextDocument.modificationChanged` -- der
Tab-Titel kriegt einen Punkt, wenn nicht gespeichert ist.
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from PySide6.QtCore import QSize, Qt, Signal
from PySide6.QtGui import QAction, QColor, QMouseEvent
from PySide6.QtWidgets import (
    QHBoxLayout, QMenu, QTabBar, QTabWidget, QToolButton, QWidget,
)

from .code_editor import CodeEditor
from .icons import icons
from .minimap import Minimap
from .theme import COLORS


@dataclass
class TabState:
    file_path: Optional[Path]
    editor: CodeEditor
    pane: QWidget        # Wrapper-Widget mit Editor + Minimap
    minimap: Minimap


class _TabBarMiddleClose(QTabBar):
    """`QTabBar` mit Mittelklick=Close + Rechtsklick-Kontextmenue."""

    middle_clicked = Signal(int)
    reveal_requested = Signal(int)
    close_requested_idx = Signal(int)
    close_others_requested = Signal(int)

    def mouseReleaseEvent(self, event: QMouseEvent):  # noqa: N802
        if event.button() == Qt.MouseButton.MiddleButton:
            idx = self.tabAt(event.position().toPoint())
            if idx >= 0:
                self.middle_clicked.emit(idx)
                return
        super().mouseReleaseEvent(event)

    def contextMenuEvent(self, event):  # noqa: N802
        idx = self.tabAt(event.pos())
        if idx < 0:
            return super().contextMenuEvent(event)
        menu = QMenu(self)
        act_reveal = QAction("Im Datei-Browser zeigen", menu)
        act_reveal.triggered.connect(lambda: self.reveal_requested.emit(idx))
        menu.addAction(act_reveal)
        menu.addSeparator()
        act_close = QAction("Tab schliessen", menu)
        act_close.triggered.connect(lambda: self.close_requested_idx.emit(idx))
        menu.addAction(act_close)
        act_close_others = QAction("Andere Tabs schliessen", menu)
        act_close_others.triggered.connect(lambda: self.close_others_requested.emit(idx))
        menu.addAction(act_close_others)
        menu.exec(event.globalPos())


class TabbedEditorArea(QTabWidget):
    active_changed = Signal(object)        # TabState | None
    dirty_changed = Signal()
    save_requested = Signal()              # Strg+S aus dem Editor
    run_requested = Signal()               # F5 aus dem Editor
    close_requested = Signal(object)       # TabState (vor Dirty-Pruefung)
    tab_opened = Signal(object)            # TabState (frisch eingefuegt)
    reveal_requested = Signal(object)      # TabState -- "Im Datei-Browser zeigen"
    close_others_requested = Signal(object)  # TabState -- "Andere schliessen"

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        bar = _TabBarMiddleClose(self)
        self.setTabBar(bar)
        bar.middle_clicked.connect(self._on_close_clicked)
        bar.close_requested_idx.connect(self._on_close_clicked)
        bar.reveal_requested.connect(self._on_reveal)
        bar.close_others_requested.connect(self._on_close_others)
        self.setTabsClosable(True)
        self.setMovable(True)
        self.setDocumentMode(True)
        self._states: list[TabState] = []
        # Welche Datei laeuft gerade + in welchem Modus ("py"/"native").
        # Treibt die Run-Markierung des zugehoerigen Tabs (Praefix + Farbe).
        self._running_path: Path | None = None
        self._running_mode: str = "py"
        self.tabCloseRequested.connect(self._on_close_clicked)
        self.currentChanged.connect(self._on_current_changed)

    def _on_reveal(self, idx: int) -> None:
        if 0 <= idx < len(self._states):
            self.reveal_requested.emit(self._states[idx])

    def _on_close_others(self, idx: int) -> None:
        if 0 <= idx < len(self._states):
            self.close_others_requested.emit(self._states[idx])

    # ------------------------------------------------------------- API
    @property
    def states(self) -> list[TabState]:
        return list(self._states)

    @property
    def active(self) -> TabState | None:
        idx = self.currentIndex()
        if 0 <= idx < len(self._states):
            return self._states[idx]
        return None

    def find_tab_for(self, path: Path) -> TabState | None:
        for st in self._states:
            if st.file_path is not None and st.file_path == path:
                return st
        return None

    def open_tab(self, file_path: Path | None, content: str | None) -> TabState:
        editor = CodeEditor()
        if content is not None:
            editor.set_text(content)
        # Pane: Editor + Minimap nebeneinander.
        pane = QWidget()
        pl = QHBoxLayout(pane)
        pl.setContentsMargins(0, 0, 0, 0)
        pl.setSpacing(0)
        pl.addWidget(editor, 1)
        minimap = Minimap(editor, pane)
        pl.addWidget(minimap, 0)

        st = TabState(file_path=file_path, editor=editor, pane=pane, minimap=minimap)
        self._states.append(st)

        editor.document().modificationChanged.connect(
            lambda _modified, s=st: self._on_dirty_changed(s)
        )
        editor.save_requested.connect(self.save_requested)
        editor.run_requested.connect(self.run_requested)

        idx = self.addTab(pane, self._tab_label(st))
        # Close-Button durch unsere theme-reaktive X-Variante ersetzen.
        self._install_close_button(idx, st)
        self.setCurrentIndex(idx)
        self.tab_opened.emit(st)
        self.dirty_changed.emit()
        return st

    def _install_close_button(self, idx: int, st: TabState) -> None:
        btn = QToolButton()
        btn.setIcon(icons.get("close"))
        btn.setIconSize(QSize(12, 12))
        btn.setAutoRaise(True)
        btn.setStyleSheet(
            "QToolButton { background: transparent; border: 0; padding: 2px; }"
            "QToolButton:hover { background: rgba(255,80,80,0.55); border-radius: 2px; }"
        )
        btn.clicked.connect(lambda _checked=False, s=st: self.close_requested.emit(s))
        self.tabBar().setTabButton(idx, QTabBar.ButtonPosition.RightSide, btn)

    def activate(self, st: TabState) -> None:
        if st in self._states:
            self.setCurrentIndex(self._states.index(st))

    def close_tab(self, st: TabState) -> None:
        if st not in self._states:
            return
        idx = self._states.index(st)
        self._states.pop(idx)
        self.removeTab(idx)

    def mark_clean(self, st: TabState) -> None:
        st.editor.document().setModified(False)
        idx = self._states.index(st) if st in self._states else -1
        if idx >= 0:
            self.setTabText(idx, self._tab_label(st))
        self.dirty_changed.emit()

    def is_dirty(self, st: TabState) -> bool:
        return st.editor.document().isModified()

    def update_tab_title(self, st: TabState) -> None:
        """Re-renders Tab-Label nach einem Save-As (Pfad geaendert)."""
        if st in self._states:
            idx = self._states.index(st)
            self.setTabText(idx, self._tab_label(st))

    def set_running(self, path: Path | None, mode: str = "py") -> None:
        """Markiert den Tab der laufenden Datei (Praefix ▶/⚙ + Akzent-Farbe).

        `path=None` raeumt die Markierung wieder ab. `mode` unterscheidet
        Tree-Walker (F5, "py") von nativer Runtime (F6, "native"). Wird die
        Datei in keinem Tab gehalten (z.B. ein Selection-Run aus einer
        Temp-Datei), bleibt schlicht kein Tab markiert.
        """
        self._running_path = path
        self._running_mode = mode
        bar = self.tabBar()
        for idx, st in enumerate(self._states):
            self.setTabText(idx, self._tab_label(st))
            is_running = path is not None and st.file_path == path
            # Ungueltige QColor -> QTabBar nimmt die Default-Textfarbe.
            bar.setTabTextColor(idx, QColor(COLORS["accent"]) if is_running else QColor())

    # -------------------------------------------------------- Internal
    def _tab_label(self, st: TabState) -> str:
        name = st.file_path.name if st.file_path else "(neu)"
        if st.file_path is not None and st.file_path == self._running_path:
            # Laufender Tab: ▶ Tree-Walker, ⚙ native Runtime.
            prefix = "⚙ " if self._running_mode == "native" else "▶ "
        else:
            prefix = "● " if st.editor.document().isModified() else "  "
        return prefix + name

    def _on_dirty_changed(self, st: TabState) -> None:
        self.update_tab_title(st)
        self.dirty_changed.emit()

    def _on_close_clicked(self, idx: int) -> None:
        # MainWindow entscheidet ueber Dirty-Prompt + Speichern -- wir
        # liefern nur die Anfrage als Signal.
        if 0 <= idx < len(self._states):
            self.close_requested.emit(self._states[idx])

    def _on_current_changed(self, idx: int) -> None:
        st = self._states[idx] if 0 <= idx < len(self._states) else None
        self.active_changed.emit(st)
