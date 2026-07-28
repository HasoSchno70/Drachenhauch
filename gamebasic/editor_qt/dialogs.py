"""Modale Dialoge: Suchen/Ersetzen, Goto-Line, Einstellungen.

Alle Dialoge greifen ueber einen `get_editor()`-Callback auf den
aktiven `CodeEditor` zu. Die `MainWindow`-Lebensdauer hostet sie als
Member, damit sie ueber Aufrufe hinweg State (zuletzt gesucht etc.)
behalten.
"""
from __future__ import annotations

import re
from typing import Callable, Optional

from PySide6.QtCore import Qt
from PySide6.QtGui import QIntValidator
from PySide6.QtWidgets import (
    QCheckBox, QComboBox, QDialog, QDialogButtonBox, QGridLayout,
    QHBoxLayout, QLabel, QLineEdit, QMessageBox, QPushButton, QSpinBox,
    QVBoxLayout, QWidget,
)


class FindReplaceDialog(QDialog):
    """Suchen / Ersetzen mit Optionen Gross/Klein, Whole-Word, Regex."""

    def __init__(self, parent: QWidget, get_editor: Callable, with_replace: bool = True):
        super().__init__(parent)
        self.get_editor = get_editor
        self.with_replace = with_replace
        self._matches: list[tuple[int, int]] = []
        self._cur_match = -1

        self.setWindowTitle("Suchen / Ersetzen" if with_replace else "Suchen")
        # Non-modal -- aber bleibt ueber dem MainWindow.
        self.setWindowFlag(Qt.WindowType.Tool, True)
        self.setModal(False)

        layout = QGridLayout(self)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setHorizontalSpacing(8)
        layout.setVerticalSpacing(6)

        layout.addWidget(QLabel("Suchen:"), 0, 0)
        self.find_entry = QLineEdit()
        self.find_entry.textChanged.connect(self._find_all)
        self.find_entry.returnPressed.connect(self.find_next)
        layout.addWidget(self.find_entry, 0, 1, 1, 3)

        if with_replace:
            layout.addWidget(QLabel("Ersetzen:"), 1, 0)
            self.replace_entry = QLineEdit()
            layout.addWidget(self.replace_entry, 1, 1, 1, 3)
            opt_row = 2
        else:
            self.replace_entry = None
            opt_row = 1

        opts_row = QHBoxLayout()
        opts_row.setSpacing(12)
        self.cb_case = QCheckBox("Gross/Klein")
        self.cb_word = QCheckBox("Ganzes Wort")
        self.cb_regex = QCheckBox("Regex")
        for cb in (self.cb_case, self.cb_word, self.cb_regex):
            cb.toggled.connect(self._find_all)
            opts_row.addWidget(cb)
        opts_row.addStretch(1)
        layout.addLayout(opts_row, opt_row, 0, 1, 4)

        btn_row = QHBoxLayout()
        btn_next = QPushButton("Naechster")
        btn_next.clicked.connect(self.find_next)
        btn_row.addWidget(btn_next)
        if with_replace:
            btn_one = QPushButton("Ersetzen")
            btn_one.clicked.connect(self.replace_one)
            btn_row.addWidget(btn_one)
            btn_all = QPushButton("Alle ersetzen")
            btn_all.clicked.connect(self.replace_all)
            btn_row.addWidget(btn_all)
        btn_row.addStretch(1)
        btn_close = QPushButton("Schliessen")
        btn_close.clicked.connect(self.close)
        btn_row.addWidget(btn_close)
        layout.addLayout(btn_row, opt_row + 1, 0, 1, 4)

        from .theme import COLORS, theme_signals
        self.status = QLabel("")
        self.status.setStyleSheet(f"color: {COLORS['fg_muted']};")
        layout.addWidget(self.status, opt_row + 2, 0, 1, 4)
        # Review-Fund: die Status-Label-Farbe wurde nur EINMAL hier gesetzt
        # und nie neu abonniert -- MainWindow haelt diesen Dialog ueber die
        # gesamte App-Laufzeit als Member am Leben (auch versteckt), ein
        # Dark<->Light-Wechsel liess den Status-Text bis zum naechsten
        # App-Start in der alten Theme-Farbe stehen.
        theme_signals.changed.connect(self._on_theme_changed)

        self.resize(520, 200)

    def _on_theme_changed(self, _name: str) -> None:
        from .theme import COLORS
        self.status.setStyleSheet(f"color: {COLORS['fg_muted']};")

    def show_for(self, prefill: str | None = None) -> None:
        ed = self.get_editor()
        if ed is not None:
            sel = ed.textCursor().selectedText()
            if prefill is None and sel:
                self.find_entry.setText(sel)
            elif prefill is not None:
                self.find_entry.setText(prefill)
        self.show()
        self.raise_()
        self.activateWindow()
        self.find_entry.selectAll()
        self.find_entry.setFocus()
        self._find_all()

    def closeEvent(self, ev):  # noqa: N802
        ed = self.get_editor()
        if ed is not None:
            ed.clear_find_hits()
        super().closeEvent(ev)

    def reject(self) -> None:  # noqa: N802 (Qt-API)
        # Review-Fund: QDialog.reject() (Escapes Default-Handler) ruft
        # standardmaessig nur hide() auf, OHNE ueber close()/closeEvent zu
        # laufen -- closeEvent() (oben) raeumt die Find-Highlights im
        # Editor auf, das griff bei Escape bisher NICHT (nur X-Button und
        # der "Schliessen"-Button rufen close() auf und funktionierten
        # schon). Escape liess die gelben Treffer-Markierungen im Editor
        # so bis zur naechsten Suche stehen, obwohl der Dialog laengst weg war.
        self.close()

    # ----------------------------------------------- Operations
    def _find_all(self, *_args) -> None:
        ed = self.get_editor()
        if ed is None:
            return
        q = self.find_entry.text()
        if not q:
            ed.clear_find_hits()
            self._matches = []
            self._cur_match = -1
            self.status.setText("")
            return
        if self.cb_regex.isChecked():
            try:
                re.compile(q)
            except re.error as exc:
                ed.clear_find_hits()
                self.status.setText(f"Regex-Fehler: {exc}")
                self._matches = []
                self._cur_match = -1
                return
        self._matches = ed.find_all(
            q,
            case_sensitive=self.cb_case.isChecked(),
            whole_word=self.cb_word.isChecked(),
            regex=self.cb_regex.isChecked(),
        )
        self._cur_match = -1
        self.status.setText(f"{len(self._matches)} Treffer" if q else "")

    def find_next(self) -> None:
        ed = self.get_editor()
        if ed is None:
            return
        if not self._matches:
            self._find_all()
        if not self._matches:
            self.status.setText("Keine Treffer")
            return
        self._cur_match = (self._cur_match + 1) % len(self._matches)
        start, _end = self._matches[self._cur_match]
        ed.go_to(start)
        self.status.setText(f"{self._cur_match + 1} von {len(self._matches)}")

    def replace_one(self) -> None:
        ed = self.get_editor()
        if ed is None or self.replace_entry is None:
            return
        if self._cur_match < 0 or self._cur_match >= len(self._matches):
            self.find_next()
            return
        repl = self.replace_entry.text()
        start, end = self._matches[self._cur_match]
        ed.replace_range(start, end, repl)
        self._find_all()
        self.find_next()

    def replace_all(self) -> None:
        ed = self.get_editor()
        if ed is None or self.replace_entry is None:
            return
        q = self.find_entry.text()
        if not q:
            return
        if self.cb_regex.isChecked():
            pat_str = q
        else:
            pat_str = re.escape(q)
        if self.cb_word.isChecked():
            pat_str = rf"\b{pat_str}\b"
        flags = 0 if self.cb_case.isChecked() else re.IGNORECASE
        try:
            pattern = re.compile(pat_str, flags)
        except re.error as exc:
            self.status.setText(f"Regex-Fehler: {exc}")
            return
        full = ed.get_text()
        new, count = pattern.subn(self.replace_entry.text(), full)
        if new != full:
            ed.replace_text_undoable(new)
            self.status.setText(f"{count} Treffer ersetzt.")
        else:
            self.status.setText("Keine Treffer.")


class GotoLineDialog(QDialog):
    """Springe zu Zeilennummer."""

    def __init__(self, parent: QWidget, get_editor: Callable):
        super().__init__(parent)
        self.get_editor = get_editor
        self.setWindowTitle("Gehe zu Zeile")
        self.setModal(True)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.addWidget(QLabel("Zeile:"))
        self.entry = QLineEdit()
        self.entry.setValidator(QIntValidator(1, 1_000_000, self))
        self.entry.returnPressed.connect(self.accept)
        layout.addWidget(self.entry)

        bb = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel,
            parent=self,
        )
        bb.accepted.connect(self.accept)
        bb.rejected.connect(self.reject)
        layout.addWidget(bb)

        self.resize(260, 110)

    def exec_for_editor(self) -> None:
        self.entry.setFocus()
        self.entry.selectAll()
        if self.exec() != QDialog.DialogCode.Accepted:
            return
        ed = self.get_editor()
        if ed is None:
            return
        try:
            line = int(self.entry.text())
        except ValueError:
            return
        ed.goto_line(line)
        ed.setFocus()


class ShortcutsDialog(QDialog):
    """Listet alle registrierten Aktionen + ihre Tastenkuerzel.

    Aufrufer uebergibt eine Liste von `(label, shortcut)`-Tupeln, sortiert
    bzw. nach Kategorie gruppiert mit Trenner-Eintraegen `("--- Kategorie ---", "")`.
    """

    def __init__(self, parent: QWidget, entries: list[tuple[str, str]]):
        super().__init__(parent)
        self.setWindowTitle("Tastenkuerzel")
        self.setModal(True)
        self.resize(560, 640)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(8)

        from PySide6.QtWidgets import QTableWidget, QTableWidgetItem, QHeaderView
        from PySide6.QtGui import QBrush, QColor, QFont
        from PySide6.QtCore import Qt as _Qt
        from .theme import COLORS

        table = QTableWidget(len(entries), 2)
        table.setHorizontalHeaderLabels(["Aktion", "Shortcut"])
        table.verticalHeader().setVisible(False)
        table.verticalHeader().setDefaultSectionSize(24)   # kompakte Zeilen
        table.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)
        table.setShowGrid(False)
        table.setAlternatingRowColors(False)               # mit Headern unklar

        cat_brush = QBrush(QColor(COLORS["bg_panel"]))
        cat_fg = QBrush(QColor(COLORS["accent"]))
        sc_fg = QBrush(QColor(COLORS["fg_muted"]))
        bold_font = QFont(); bold_font.setBold(True)

        for row, (label, sc) in enumerate(entries):
            if label.startswith("---"):
                # Kategorie-Header: Span ueber beide Spalten, eigener BG.
                head_text = label.strip("- ").upper()
                head = QTableWidgetItem(head_text)
                head.setFont(bold_font)
                head.setForeground(cat_fg)
                head.setBackground(cat_brush)
                head.setFlags(_Qt.ItemFlag.ItemIsEnabled)
                table.setItem(row, 0, head)
                # Spalte 1 als leeres Item mit identischem BG, sonst zeigt Qt
                # die Default-Cell-Farbe und wir kriegen einen Knick.
                blank = QTableWidgetItem("")
                blank.setBackground(cat_brush)
                blank.setFlags(_Qt.ItemFlag.ItemIsEnabled)
                table.setItem(row, 1, blank)
                table.setSpan(row, 0, 1, 2)
                table.setRowHeight(row, 26)
                continue
            it_label = QTableWidgetItem("  " + label)
            it_sc = QTableWidgetItem(sc)
            it_sc.setForeground(sc_fg)
            it_sc.setTextAlignment(_Qt.AlignmentFlag.AlignRight | _Qt.AlignmentFlag.AlignVCenter)
            table.setItem(row, 0, it_label)
            table.setItem(row, 1, it_sc)
            table.setRowHeight(row, 22)

        table.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeMode.Stretch)
        table.horizontalHeader().setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        layout.addWidget(table)

        bb = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok, parent=self)
        bb.accepted.connect(self.accept)
        layout.addWidget(bb)


class AboutDialog(QDialog):
    """`Hilfe -> Ueber GameBasic` -- Logo + Version + Autor + Copyright."""

    def __init__(self, parent: QWidget):
        super().__init__(parent)
        self.setWindowTitle("Ueber GameBasic")
        self.setModal(True)
        self.setMinimumWidth(420)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 16)
        layout.setSpacing(14)

        # Logo
        logo_lbl = QLabel()
        logo_lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
        try:
            from .. import branding
            from PIL.ImageQt import ImageQt
            from PySide6.QtGui import QPixmap
            scaled = branding.scaled_logo_image(320)
            if scaled is not None:
                qimg = ImageQt(scaled).copy()
                self._logo_pixmap = QPixmap.fromImage(qimg)
                logo_lbl.setPixmap(self._logo_pixmap)
        except Exception:
            logo_lbl.setText("GameBasic")
        layout.addWidget(logo_lbl)

        # Titel
        title = QLabel("GameBasic Editor")
        title_font = title.font()
        title_font.setPointSize(14)
        title_font.setBold(True)
        title.setFont(title_font)
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(title)

        # Version
        try:
            from gamebasic import __version__ as ver
        except Exception:
            ver = ""
        if ver:
            ver_lbl = QLabel(f"Version {ver}")
            ver_lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
            layout.addWidget(ver_lbl)

        # Autor
        author = QLabel("Hans Schnorrenberger")
        author_font = author.font()
        author_font.setItalic(True)
        author.setFont(author_font)
        author.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(author)

        # Copyright
        from .theme import COLORS
        muted_style = f"color: {COLORS['fg_muted']};"
        copyright_lbl = QLabel("(C) 2026 Hans Schnorrenberger")
        copyright_lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
        copyright_lbl.setStyleSheet(muted_style)
        layout.addWidget(copyright_lbl)

        # Sprache + Beschreibung
        descr = QLabel(
            "BASIC-Dialekt mit strikter Typisierung und OOP, gemacht fuer Spiele.\n"
            "2D- & 3D-Grafik, Sound, GUI, Partikel, Module und mehr -- ausgefuehrt\n"
            "von der nativen Runtime 'gbrt' (Rust/raylib), exportierbar als .exe."
        )
        descr.setAlignment(Qt.AlignmentFlag.AlignCenter)
        descr.setStyleSheet(muted_style)
        layout.addWidget(descr)

        # OK
        bb = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok, parent=self)
        bb.accepted.connect(self.accept)
        layout.addWidget(bb)


class RecoveryDialog(QDialog):
    """Beim Start nach Crash gefundene Auto-Save-Snapshots wiederherstellen.

    Zeigt eine Liste der nicht gespeicherten Tabs mit Checkboxen --
    User waehlt aus, welche wiederhergestellt werden sollen.
    """

    def __init__(self, parent: QWidget, entries: list):
        super().__init__(parent)
        self.entries = entries
        self.selected: list = []     # Liste von RecoveryEntry zum Wiederherstellen

        self.setWindowTitle("Wiederherstellen")
        self.setModal(True)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(10)
        info = QLabel(
            "Beim letzten Beenden waren noch ungespeicherte Aenderungen offen.\n"
            "Auswahl wiederherstellen?"
        )
        layout.addWidget(info)

        self._checks: list[QCheckBox] = []
        for e in entries:
            cb = QCheckBox(e.label)
            cb.setChecked(True)
            cb.entry = e            # type: ignore[attr-defined]  # piggy-back
            self._checks.append(cb)
            layout.addWidget(cb)

        bb = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel,
            parent=self,
        )
        bb.button(QDialogButtonBox.StandardButton.Ok).setText("Wiederherstellen")
        bb.button(QDialogButtonBox.StandardButton.Cancel).setText("Verwerfen")
        bb.accepted.connect(self._on_accept)
        bb.rejected.connect(self.reject)
        layout.addWidget(bb)

        self.resize(480, 260)

    def _on_accept(self) -> None:
        self.selected = [cb.entry for cb in self._checks if cb.isChecked()]  # type: ignore[attr-defined]
        self.accept()


class SettingsDialog(QDialog):
    """Einstellungen: Theme, Schriftgroesse, Zeilenumbruch, Minimap,
    Auto-Completion-Trigger, Format-on-Save."""

    def __init__(self, parent: QWidget, settings: dict):
        super().__init__(parent)
        self.settings = settings
        self.setWindowTitle("Einstellungen")
        self.setModal(True)

        from .theme import EDITOR_FONT_SIZE

        layout = QGridLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setHorizontalSpacing(12)
        layout.setVerticalSpacing(10)
        row = 0

        # Theme
        layout.addWidget(QLabel("Theme:"), row, 0)
        self.theme_combo = QComboBox()
        self.theme_combo.addItem("Dark (VSCode Dark+)", "dark")
        self.theme_combo.addItem("Light (VSCode Light+)", "light")
        current_theme = settings.get("theme", "dark")
        idx = self.theme_combo.findData(current_theme)
        self.theme_combo.setCurrentIndex(max(0, idx))
        layout.addWidget(self.theme_combo, row, 1)
        row += 1

        # Schriftgroesse
        layout.addWidget(QLabel("Schriftgroesse:"), row, 0)
        self.spin_font = QSpinBox()
        self.spin_font.setRange(6, 36)
        self.spin_font.setSuffix(" pt")
        self.spin_font.setValue(int(settings.get("editor_font_size", EDITOR_FONT_SIZE)))
        layout.addWidget(self.spin_font, row, 1)
        row += 1

        # Zeilenumbruch
        self.cb_wrap = QCheckBox("Zeilenumbruch")
        self.cb_wrap.setChecked(bool(settings.get("word_wrap", False)))
        layout.addWidget(self.cb_wrap, row, 0, 1, 2)
        row += 1

        # Minimap
        self.cb_minimap = QCheckBox("Minimap anzeigen")
        self.cb_minimap.setChecked(bool(settings.get("minimap_visible", True)))
        layout.addWidget(self.cb_minimap, row, 0, 1, 2)
        row += 1

        # Auto-Trigger
        self.cb_auto = QCheckBox("Auto-Completion automatisch beim Tippen oeffnen")
        self.cb_auto.setChecked(bool(settings.get("autocomplete_auto", True)))
        layout.addWidget(self.cb_auto, row, 0, 1, 2)
        row += 1

        # Format-on-Save
        self.cb_format_on_save = QCheckBox(
            "Beim Speichern automatisch formatieren"
        )
        self.cb_format_on_save.setChecked(
            bool(settings.get("format_on_save", False))
        )
        layout.addWidget(self.cb_format_on_save, row, 0, 1, 2)
        row += 1

        # Buttons
        bb = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel,
            parent=self,
        )
        bb.accepted.connect(self.accept)
        bb.rejected.connect(self.reject)
        layout.addWidget(bb, row, 0, 1, 2)

        self.resize(440, 260)

    def selected_theme(self) -> str:
        return self.theme_combo.currentData() or "dark"

    def font_size(self) -> int:
        return int(self.spin_font.value())

    def word_wrap_enabled(self) -> bool:
        return self.cb_wrap.isChecked()

    def minimap_enabled(self) -> bool:
        return self.cb_minimap.isChecked()

    def auto_complete_enabled(self) -> bool:
        return self.cb_auto.isChecked()

    def format_on_save_enabled(self) -> bool:
        return self.cb_format_on_save.isChecked()
