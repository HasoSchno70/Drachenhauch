"""Outline-Panel: zeigt CLASS/STRUCT/SUB/FUNCTION der aktiven Datei.

Click springt zur Definition. Filter-Eingabefeld oben filtert die
Liste live (Substring-Match auf Name oder Kind).
"""
from __future__ import annotations

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import (
    QFrame, QLabel, QLineEdit, QListWidget, QListWidgetItem, QVBoxLayout,
    QWidget,
)

from .symbols import _strip_comment_and_strings
from .theme import COLORS, theme_signals


class OutlinePanel(QWidget):
    jump_requested = Signal(int)  # 1-basierte Zeilennummer

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        self._items: list[tuple[str, str, int, int]] = []  # (kind, name, line, indent)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self.header = QLabel("Struktur")
        self.header.setStyleSheet(self._header_style())
        layout.addWidget(self.header)

        self.filter_entry = QLineEdit()
        self.filter_entry.setPlaceholderText("filtern ...")
        self.filter_entry.textChanged.connect(self._render)
        wrap = QFrame()
        wl = QVBoxLayout(wrap)
        wl.setContentsMargins(6, 6, 6, 4)
        wl.addWidget(self.filter_entry)
        layout.addWidget(wrap)

        self.list = QListWidget()
        self.list.itemActivated.connect(self._on_activated)
        self.list.itemClicked.connect(self._on_activated)
        layout.addWidget(self.list, 1)

        theme_signals.changed.connect(self._on_theme_changed)

    def _header_style(self) -> str:
        return (
            f"background-color: {COLORS['bg_panel']}; color: {COLORS['fg']}; "
            "font-weight: bold; padding: 6px 10px;"
        )

    def _on_theme_changed(self, _name: str) -> None:
        self.header.setStyleSheet(self._header_style())
        self._render()

    def update_for(self, source: str | None) -> None:
        if source is None:
            self._items = []
        else:
            self._items = self._scan(source)
        self._render()

    def _render(self, *_args) -> None:
        self.list.clear()
        q = self.filter_entry.text().strip().lower()
        items = self._items
        if q:
            items = [it for it in items if q in it[1].lower() or q in it[0].lower()]
        if not items:
            placeholder = QListWidgetItem(
                "Keine Treffer im Filter" if q else
                "Keine SUB/FUNCTION/CLASS in dieser Datei"
            )
            placeholder.setForeground(QColor(COLORS["fg_muted"]))
            placeholder.setFlags(Qt.ItemFlag.NoItemFlags)
            self.list.addItem(placeholder)
            return
        color_map = {
            "class":    COLORS["type"],
            "struct":   COLORS["type"],
            "function": COLORS["builtin"],
            "sub":      COLORS["builtin"],
            "property": COLORS["builtin"],
        }
        for kind, name, line, indent, params in items:
            label = f"{'  ' * indent}{kind.upper():9s} {name}{params}"
            it = QListWidgetItem(label)
            it.setForeground(QColor(color_map.get(kind, COLORS["fg"])))
            it.setData(Qt.ItemDataRole.UserRole, line)
            self.list.addItem(it)

    def _on_activated(self, item: QListWidgetItem) -> None:
        line = item.data(Qt.ItemDataRole.UserRole)
        if isinstance(line, int) and line > 0:
            self.jump_requested.emit(line)

    @staticmethod
    def _scan(source: str) -> list[tuple[str, str, int, int, str]]:
        """Liefert Liste von (kind, name, line, indent, params).

        `params` ist die kompakte Argumentliste fuer SUB/FUNCTION (z.B.
        ``"(a, b)"``), oder ``""`` bei CLASS/STRUCT/leeren Parameterlisten.
        """
        results: list[tuple[str, str, int, int, str]] = []
        container_stack: list[str] = []
        for ln_idx, raw in enumerate(source.split("\n"), start=1):
            # Review-Fund: dieser Scanner ist unabhaengig von symbols.py's
            # scan_scopes() (das Breadcrumbs/Outline-Symbole eigentlich schon
            # ueber EINE geteilte, comment-/string-aware Quelle beziehen
            # sollten) und war weder comment-/string-aware NOCH kannte er
            # PROPERTY GET/SET -- eine Klasse mit nur Properties zeigte gar
            # nichts im Outline, obwohl dieselbe Property in der Breadcrumb-
            # Leiste (die scan_scopes nutzt) korrekt als "◆ propname"
            # erscheint. Ohne Comment-Strip haette ausserdem z.B.
            # `' SUB fake()` als echter Opener durchgehen koennen.
            stripped = _strip_comment_and_strings(raw).strip()
            upper = stripped.upper()
            if upper.startswith("CLASS "):
                tail = stripped[6:].split(maxsplit=1)
                name = tail[0] if tail else "?"
                results.append(("class", name, ln_idx, len(container_stack), ""))
                container_stack.append("class")
                continue
            if upper.startswith("STRUCT "):
                tail = stripped[7:].split(maxsplit=1)
                name = tail[0] if tail else "?"
                results.append(("struct", name, ln_idx, len(container_stack), ""))
                container_stack.append("struct")
                continue
            if upper == "END CLASS":
                if container_stack and container_stack[-1] == "class":
                    container_stack.pop()
                continue
            if upper == "END STRUCT":
                if container_stack and container_stack[-1] == "struct":
                    container_stack.pop()
                continue
            if upper.startswith("SUB "):
                rest = stripped[4:]
                name, params = OutlinePanel._split_name_params(rest)
                results.append(("sub", name, ln_idx, len(container_stack), params))
            elif upper.startswith("FUNCTION "):
                rest = stripped[9:]
                name, params = OutlinePanel._split_name_params(rest)
                results.append(("function", name, ln_idx, len(container_stack), params))
            elif upper.startswith("PROPERTY GET ") or upper.startswith("PROPERTY SET "):
                # Wie SUB/FUNCTION ein Blatt (kein Nesting-Push/Pop noetig --
                # Drachenhauch verschachtelt keine PROPERTY-Bloecke ineinander).
                rest = stripped[13:]   # nach "PROPERTY GET "/"PROPERTY SET " (je 13 Zeichen)
                name, params = OutlinePanel._split_name_params(rest)
                results.append(("property", name, ln_idx, len(container_stack), params))
        return results

    @staticmethod
    def _split_name_params(rest: str) -> tuple[str, str]:
        """Trennt `Foo(a AS INTEGER, b AS INTEGER)` -> ("Foo", "(a, b)").

        Wir abstrahieren die Argument-Typen weg, damit lange Param-Listen
        nicht den Outline-Bereich sprengen. Default-Werte und BYREF-Marker
        werden ebenfalls weggelassen.
        """
        name_part, _, after_paren = rest.partition("(")
        name = name_part.strip() or "?"
        if not after_paren:
            return name, ""
        # Nach dem Closing-Paren alles weg.
        params_text, _, _ = after_paren.partition(")")
        if not params_text.strip():
            return name, "()"
        # Pro Komma-Segment nur das erste Identifier-Token nehmen.
        names: list[str] = []
        for seg in params_text.split(","):
            seg = seg.strip()
            if seg.startswith("..."):
                # Variadic: ...args
                tail = seg[3:].lstrip().split()
                names.append("..." + (tail[0] if tail else "args"))
                continue
            tokens = seg.replace("BYREF ", "").replace("byref ", "").split()
            if tokens:
                # Param-Name ist das erste Token, ggf. ohne `$`-Suffix.
                names.append(tokens[0])
        if not names:
            return name, "()"
        return name, "(" + ", ".join(names) + ")"
