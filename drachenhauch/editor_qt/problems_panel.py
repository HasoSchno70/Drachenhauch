"""Probleme-Panel.

Listet ALLE Live-Diagnosen des aktiven Editors (Errors + Warnungen aus
`dhrt --check`) -- nicht nur das erste Problem wie die Statusbar. Klick
springt zur Zeile.

Als **Tabelle** (Zeile / Art / Meldung) statt als Liste: bei mehr als einer
Handvoll Diagnosen will man nach Zeile sortieren (der Reihe nach abarbeiten)
oder nach Art (erst die Fehler, dann die Warnungen). In einer Liste aus
zusammengesetzten Textzeilen geht beides nicht. Spaltenbreiten sind ziehbar,
ein Klick auf den Kopf sortiert -- dieselbe Erwartung wie an jede Tabelle.

Sortiert wird ueber `Qt.ItemDataRole.UserRole`, nicht ueber den angezeigten
Text: sonst stuende Zeile 10 vor Zeile 9, und "Fehler" vor "Warnung" waere
reiner Zufall der Alphabetisierung.

**Anfangsreihenfolge: nach ZEILE.** Vorher (als Liste) standen die Fehler
oben. Mit einer sortierbaren Tabelle geht das nicht mehr sauber: Qt sortiert
beim Einschalten der Sortierung sofort nach der aktiven Kopfspalte und wuerde
eine von Hand gesetzte Zwei-Schluessel-Reihenfolge gleich wieder umwerfen.
Statt das mit einem versteckten Sortier-Schluessel zu erzwingen -- der dann
beim Klick auf "Zeile" etwas anderes taete als "Zeile" verspricht -- sortiert
jede Spalte genau nach dem, was sie anzeigt. Fehler zuerst ist ein Klick auf
"Art".
"""
from __future__ import annotations

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QFont
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QHeaderView, QLabel, QTableWidget, QTableWidgetItem,
    QVBoxLayout, QWidget,
)

from .theme import COLORS, theme_signals


class _SortItem(QTableWidgetItem):
    """Zellwert, der nach seinem HINTERLEGTEN Wert sortiert.

    Ohne das sortierte die Zeilen-Spalte alphabetisch (10 vor 9) und die
    Art-Spalte nach Anfangsbuchstabe statt nach Schwere.
    """

    def __lt__(self, other):
        a = self.data(Qt.ItemDataRole.UserRole)
        b = other.data(Qt.ItemDataRole.UserRole) if isinstance(other, QTableWidgetItem) else None
        if a is not None and b is not None:
            return a < b
        return super().__lt__(other)


class ProblemsPanel(QWidget):
    """Tabelle der Diagnosen (Errors/Warnungen) des aktiven Editors."""

    jump_requested = Signal(int)   # Zeilennummer (1-basiert)

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self._header = QFrame()
        hl = QHBoxLayout(self._header)
        hl.setContentsMargins(10, 6, 10, 6)
        self._title = QLabel("Probleme")
        tf = QFont()
        tf.setBold(True)
        self._title.setFont(tf)
        hl.addWidget(self._title)
        hl.addStretch(1)
        self._count = QLabel("0")
        hl.addWidget(self._count)
        layout.addWidget(self._header)

        self.table = QTableWidget(0, 3)
        self.table.setHorizontalHeaderLabels(["Zeile", "Art", "Meldung"])
        self.table.verticalHeader().setVisible(False)
        self.table.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self.table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)
        self.table.setSelectionMode(QTableWidget.SelectionMode.SingleSelection)
        self.table.setShowGrid(False)
        self.table.setSortingEnabled(True)
        # Eine Zeile je Problem. Qt bricht in Tabellen per Vorgabe um -- lange
        # Meldungen machten die Zeilen dann unterschiedlich hoch, und die
        # Liste laesst sich nicht mehr ueberfliegen. Der volle Text steht im
        # Tooltip.
        self.table.setWordWrap(False)
        self.table.setTextElideMode(Qt.TextElideMode.ElideRight)
        self.table.verticalHeader().setDefaultSectionSize(20)
        kopf = self.table.horizontalHeader()
        kopf.setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)
        kopf.setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        # Die Meldung bekommt den Rest -- sie ist das, was man wirklich lesen
        # will, und sie ist als einzige unterschiedlich lang.
        kopf.setSectionResizeMode(2, QHeaderView.ResizeMode.Stretch)
        # Hat der Nutzer selbst eine Spalte gewaehlt? Dann bei neuen Diagnosen
        # NICHT auf "nach Zeile" zurueckspringen -- wer nach Art sortiert hat,
        # will das behalten, waehrend er tippt.
        self._sortiert_vom_nutzer = False
        self.table.horizontalHeader().sectionClicked.connect(self._kopf_geklickt)
        self.table.cellActivated.connect(self._on_cell)
        self.table.cellClicked.connect(self._on_cell)
        layout.addWidget(self.table, 1)

        self._empty_hint = "Keine Probleme"
        self._apply_style()
        # Bound-Method statt Lambda (Use-after-free-Fix, siehe breadcrumbs.py).
        theme_signals.changed.connect(self._apply_style)

    def _apply_style(self) -> None:
        c = COLORS
        self._header.setStyleSheet(f"background-color: {c['bg_panel']};")
        self._title.setStyleSheet(f"color: {c['fg']};")
        self._count.setStyleSheet(f"color: {c['fg_muted']};")
        self.table.setStyleSheet(
            f"QTableWidget {{ background-color: {c['bg_alt']}; color: {c['fg']}; "
            f"border: 0; outline: 0; gridline-color: {c['border']}; }} "
            f"QTableWidget::item {{ padding: 2px 6px; }} "
            f"QTableWidget::item:hover {{ background-color: {c['bg_hover']}; }} "
            f"QTableWidget::item:selected {{ background-color: {c['sel']}; }} "
            f"QHeaderView::section {{ background-color: {c['bg_panel']}; "
            f"color: {c['fg_muted']}; border: 0; "
            f"border-bottom: 1px solid {c['border']}; padding: 3px 6px; }}"
        )

    def set_problems(self, problems) -> None:
        """`problems`: Liste von ParseProblem (line, message, severity, phase)."""
        problems = list(problems or [])
        # Waehrend des Fuellens abschalten: sonst sortiert Qt nach jeder
        # eingefuegten Zeile neu und die Zellen landen in fremden Zeilen.
        self.table.setSortingEnabled(False)
        self.table.clearContents()
        self.table.setRowCount(len(problems) if problems else 1)

        if not problems:
            hinweis = QTableWidgetItem(self._empty_hint)
            hinweis.setForeground(QColor(COLORS["fg_muted"]))
            hinweis.setFlags(Qt.ItemFlag.NoItemFlags)
            self.table.setItem(0, 0, QTableWidgetItem(""))
            self.table.setItem(0, 1, QTableWidgetItem(""))
            self.table.setItem(0, 2, hinweis)
            self._count.setText("0")
            self.table.setSortingEnabled(True)
            return

        for r, p in enumerate(problems):
            ist_fehler = p.severity == "error"
            farbe = QColor(COLORS["error"] if ist_fehler else COLORS["warning"])
            zeile = _SortItem(str(p.line))
            zeile.setData(Qt.ItemDataRole.UserRole, int(p.line))
            art = _SortItem("✕ Fehler" if ist_fehler else "⚠ Warnung")
            # Fehler vor Warnung, unabhaengig vom Anzeigetext.
            art.setData(Qt.ItemDataRole.UserRole, 0 if ist_fehler else 1)
            text = QTableWidgetItem(p.message)
            for it in (zeile, art, text):
                it.setForeground(farbe)
                it.setToolTip(f"{p.phase}: {p.message}")
                # Die Zeilennummer haengt an JEDER Zelle -- ein Klick soll
                # springen, egal welche Spalte man trifft.
                it.setData(Qt.ItemDataRole.UserRole + 1, int(p.line))
            self.table.setItem(r, 0, zeile)
            self.table.setItem(r, 1, art)
            self.table.setItem(r, 2, text)

        self._count.setText(str(len(problems)))
        # Erst die Spalte festlegen, dann einschalten: `setSortingEnabled`
        # sortiert sofort nach der AKTIVEN Kopfspalte -- ohne diese Zeile
        # haenge die Anfangsreihenfolge davon ab, was zuletzt angeklickt wurde.
        self.table.setSortingEnabled(True)
        if not self._sortiert_vom_nutzer:
            self.table.sortItems(0, Qt.SortOrder.AscendingOrder)

    def _kopf_geklickt(self, _spalte: int) -> None:
        self._sortiert_vom_nutzer = True

    def _on_cell(self, row: int, col: int) -> None:
        it = self.table.item(row, col)
        if it is None:
            return
        line = it.data(Qt.ItemDataRole.UserRole + 1)
        if line is not None:
            self.jump_requested.emit(int(line))
