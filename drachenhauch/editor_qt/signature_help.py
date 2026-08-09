"""Signature-Help / Parameter-Hints beim Tippen eines Funktionsaufrufs.

Tippt der User `LINE(`, zeigt ein dezentes Popup die Signatur, der gerade
aktive Parameter (nach Komma-Zaehlung) fett + Akzent-Farbe. Funktioniert
fuer Built-ins (Signatur aus `builtins_registry.signature_text`) UND
User-Funktionen (Signatur aus `symbols.extract_user_doc`).

Reine Anzeige -- kein Einfluss auf die Eingabe. Der CodeEditor ruft
`update(...)` bei Cursor-Bewegung; dieses Modul parst den Text vor dem
Cursor, findet den umschliessenden offenen `(` und den aktiven Argument-
Index und rendert das Popup.
"""
from __future__ import annotations

import html as _html

from PySide6.QtCore import Qt
from PySide6.QtGui import QFont
from PySide6.QtWidgets import QFrame, QLabel, QVBoxLayout

from .theme import COLORS, EDITOR_FONT_FAMILY

_IDENT = set(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$")


def _ident_before(text: str, paren_pos: int) -> str:
    """Bezeichner direkt links vom `(` an `paren_pos` (Spaces uebersprungen)."""
    i = paren_pos - 1
    while i >= 0 and text[i] in " \t":
        i -= 1
    end = i + 1
    while i >= 0 and text[i] in _IDENT:
        i -= 1
    return text[i + 1:end]


def find_active_call(text: str) -> tuple[str, int] | None:
    """Innerster offener Funktionsaufruf vor dem Cursor.

    Liefert `(funktionsname, aktiver_arg_index)` oder None, wenn der Cursor
    nicht in einer Argumentliste steht. `[...]`-Index-Ausdruecke und reine
    Gruppierungs-`(...)` (ohne Namen davor) liefern None.
    """
    stack: list[list] = []   # [opener, name, comma_count]
    i, n = 0, len(text)
    in_str = False
    while i < n:
        ch = text[i]
        if in_str:
            if ch == '"':
                in_str = False
            i += 1
            continue
        if ch == '"':
            in_str = True
        elif (ch in ("R", "r") and text[i:i + 3].upper() == "REM"
              and (i + 3 == n or not (text[i + 3].isalnum() or text[i + 3] == "_"))
              and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_"))):
            # Review-Fund: REM ist an JEDEM Wort-Anfang ein Kommentar bis
            # Zeilenende -- nicht nur am Zeilenanfang (die vorherige Fassung
            # dieses Checks gatete auf `at_line_start`, uebereinstimmend mit
            # einem falschen Kommentar, der behauptete, das sei auch
            # `symbols._strip_comment_and_strings`s Verhalten -- tatsaechlich
            # prueft DIE dort nur, ob das Zeichen davor kein Identifier-
            # Zeichen ist, NICHT ob es Zeilenanfang ist). Ein TRAILING REM-
            # Kommentar (z.B. "x = 1 REM siehe auch zoom(", ein gaengiger
            # Stil) wurde dadurch nicht erkannt -- die unbalancierte Klammer
            # darin erzeugte denselben Phantom-Stack-Frame-Bug, den dieser
            # Check eigentlich schon beheben sollte.
            j = text.find("\n", i)
            if j == -1:
                break
            i = j
            continue
        elif ch == "'":                      # Kommentar bis Zeilenende
            j = text.find("\n", i)
            if j == -1:
                break
            i = j
            continue
        elif ch == "(":
            stack.append(["(", _ident_before(text, i), 0])
        elif ch == "[":
            stack.append(["[", "", 0])
        elif ch in ")]":
            if stack:
                stack.pop()
        elif ch == "," and stack:
            stack[-1][2] += 1
        i += 1
    if not stack:
        return None
    opener, name, commas = stack[-1]
    if opener != "(" or not name:
        return None
    return name, commas


def _param_ranges(sig: str) -> tuple[list[tuple[int, int]], bool] | None:
    """Zeichen-Bereiche der einzelnen Parameter in der Signatur + variadic-Flag."""
    op = sig.find("(")
    if op == -1:
        return None
    depth = 0
    close = len(sig)
    for i in range(op, len(sig)):
        c = sig[i]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                close = i
                break
    inner = sig[op + 1:close]
    # `[` markiert optionale Params (`[, farbe]`) -> als variadic werten, damit
    # ein Argument hinter dem letzten Pflicht-Param noch hervorgehoben wird.
    variadic = any(t in inner for t in ("...", "Argumente", "*args", "["))
    ranges: list[tuple[int, int]] = []
    depth = 0           # nur echte Klammern zaehlen (nicht []), damit das
    seg_start = 0       # Komma in "[, farbe]" ein Trenner bleibt.
    for i, c in enumerate(inner):
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
        elif c == "," and depth == 0:
            ranges.append((op + 1 + seg_start, op + 1 + i))
            seg_start = i + 1
    ranges.append((op + 1 + seg_start, op + 1 + len(inner)))
    return ranges, variadic


def _trim_range(sig: str, s: int, e: int) -> tuple[int, int]:
    """Fuehrende/abschliessende Spaces + `[` `]` aus dem Bereich nehmen,
    sodass nur der Parameter-Name (ggf. mit Typ) fett wird -- nicht die
    optional-Klammern."""
    while s < e and sig[s] in " []":
        s += 1
    while e > s and sig[e - 1] in " []":
        e -= 1
    return s, e


def render_signature_html(sig: str, active: int) -> str:
    """Signatur als HTML, aktiver Parameter fett + Akzent-Farbe."""
    norm = COLORS["fg"]
    hi = COLORS["accent"]
    esc = _html.escape
    info = _param_ranges(sig)
    if info is None:
        return f'<span style="color:{norm}">{esc(sig)}</span>'
    ranges, variadic = info
    # Leere Parameterliste (`FOO()`) -> nichts hervorzuheben.
    if len(ranges) == 1 and sig[ranges[0][0]:ranges[0][1]].strip() == "":
        return f'<span style="color:{norm}">{esc(sig)}</span>'
    # Generische variadic-Form ("N Argumente") nicht hervorheben.
    if len(ranges) == 1 and "Argumente" in sig[ranges[0][0]:ranges[0][1]]:
        return f'<span style="color:{norm}">{esc(sig)}</span>'
    idx: int | None = active
    if active >= len(ranges):
        idx = len(ranges) - 1 if variadic else None
    if idx is None:
        return f'<span style="color:{norm}">{esc(sig)}</span>'
    s, e = _trim_range(sig, *ranges[idx])
    return (f'<span style="color:{norm}">{esc(sig[:s])}'
            f'<b style="color:{hi}">{esc(sig[s:e])}</b>'
            f'{esc(sig[e:])}</span>')


class SignaturePopup(QFrame):
    """Dezentes, nicht-fokussierbares Tooltip-Popup ueber dem Cursor."""

    def __init__(self, parent=None):
        super().__init__(parent, Qt.WindowType.ToolTip)
        self.setAttribute(Qt.WidgetAttribute.WA_ShowWithoutActivating)
        lay = QVBoxLayout(self)
        lay.setContentsMargins(9, 5, 9, 5)
        self._label = QLabel()
        self._label.setTextFormat(Qt.TextFormat.RichText)
        self._label.setFont(QFont(EDITOR_FONT_FAMILY))
        lay.addWidget(self._label)
        self.hide()

    def show_html(self, html_text: str, top_left, anchor_bottom) -> None:
        """Zeigt `html_text`. `anchor_bottom` = y-Linie, ueber die das Popup
        gesetzt wird (Cursor-Oberkante), `top_left` die globale x/y-Basis."""
        c = COLORS
        self.setStyleSheet(
            f"QFrame {{ background-color: {c['tooltip_bg']}; "
            f"border: 1px solid {c['accent']}; border-radius: 4px; }} "
            f"QLabel {{ background: transparent; }}"
        )
        self._label.setText(html_text)
        self.adjustSize()
        # Bevorzugt ueber der Cursor-Zeile platzieren (verdeckt nicht die
        # Eingabe); kein Platz oben -> darunter.
        x = top_left.x()
        y = anchor_bottom.y() - self.height() - 2
        if y < 0:
            y = top_left.y() + 2
        self.move(x, y)
        if not self.isVisible():
            self.show()
