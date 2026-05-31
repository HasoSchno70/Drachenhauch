"""Syntax-Highlighter fuer GameBasic (PySide6).

Nutzt den Lexer aus `gamebasic.lexer` -- so bleibt der Highlighter zu
100% konsistent mit der Sprache. Wenn ein Zeile zur Laufzeit nicht
sauber lext (z.B. unterminierter String), faerben wir die Zeile als
Kommentar-Fortsetzung und lassen den Rest in Ruhe.
"""
from __future__ import annotations

from PySide6.QtCore import Qt
from PySide6.QtGui import (
    QColor, QFont, QSyntaxHighlighter, QTextCharFormat, QTextDocument,
)

from ..tokens import TokenType
from ..lexer import Lexer
from ..errors import LexerError

from .theme import COLORS


# Token-Klassifikation (gleiche Gruppen wie der CTk-Editor).
_CONTROL_KW = {
    TokenType.IF, TokenType.THEN, TokenType.ELSE, TokenType.ELSEIF,
    TokenType.WHILE, TokenType.WEND, TokenType.FOR, TokenType.TO,
    TokenType.STEP, TokenType.NEXT, TokenType.RETURN,
    TokenType.BREAK, TokenType.CONTINUE, TokenType.END,
    TokenType.TRY, TokenType.CATCH, TokenType.THROW,
    TokenType.SELECT, TokenType.CASE, TokenType.IS, TokenType.WHERE,
    TokenType.REPEAT, TokenType.UNTIL,
}
_DECL_KW = {
    TokenType.DIM, TokenType.AS, TokenType.SUB, TokenType.FUNCTION,
    TokenType.CLASS, TokenType.NEW, TokenType.EXTENDS, TokenType.STRUCT,
    TokenType.CONST, TokenType.PRINT, TokenType.INPUT,
    TokenType.AND, TokenType.OR, TokenType.NOT, TokenType.MOD,
    TokenType.BAND, TokenType.BOR, TokenType.BXOR, TokenType.BNOT,
    TokenType.SHL, TokenType.SHR,
    TokenType.ARRAY, TokenType.OF, TokenType.MAP, TokenType.IMPORT,
    TokenType.BYREF, TokenType.ENUM, TokenType.WITH, TokenType.STATIC,
    TokenType.FUNCREF, TokenType.IN, TokenType.PROPERTY,
    TokenType.DATA, TokenType.READ, TokenType.RESTORE,
}
_TYPE_KW = {
    TokenType.INTEGER, TokenType.FLOAT, TokenType.STRING_TYPE,
    TokenType.BOOLEAN, TokenType.IMAGE, TokenType.SOUND, TokenType.FILE,
    TokenType.TUPLE,
}


# Bekannte Built-in-Namen (lowercase) -- stark aus dem CTk-Editor uebernommen.
BUILTIN_NAMES = frozenset({
    "str$", "str", "val", "int", "abs", "len", "chr$", "chr", "asc",
    "sqr", "rnd", "upper$", "upper", "lower$", "lower", "rgb",
    "left$", "left", "right$", "right", "mid$", "mid", "instr",
    "replace$", "replace", "trim$", "trim", "split$", "split",
    "join$", "join", "dimsize", "dimcount",
    "openfile", "closefile", "readline", "readall$", "readall",
    "endoffile", "writeline", "write", "fileexists",
    "screen", "cls", "plot", "line", "box", "rect", "circle", "text",
    "flip", "sleep", "keypressed", "quitrequested",
    "mousex", "mousey", "mousebutton",
    "loadimage", "drawimage", "drawimagepart", "imagewidth", "imageheight",
    "loadsound", "playsound", "stopsound", "playmusic", "stopmusic",
    "drawtilemap", "millis", "time$",
})


def _make_format(color: str, *, bold: bool = False, italic: bool = False) -> QTextCharFormat:
    f = QTextCharFormat()
    f.setForeground(QColor(color))
    if bold:
        f.setFontWeight(QFont.Weight.Bold)
    if italic:
        f.setFontItalic(True)
    return f


class GBHighlighter(QSyntaxHighlighter):
    """Lex-basierter Highlighter mit Block-State fuer Multi-Line-Comments."""

    # Block-State-Konstanten -- aktuell hat GB keine Multi-Line-Tokens
    # (Strings sind single-line, Kommentare auch). Die State-Maschine ist
    # vorbereitet falls spaeter z.B. /* ... */ oder Multi-Line-Strings
    # hinzukommen.
    STATE_NORMAL = 0

    def __init__(self, document: QTextDocument):
        super().__init__(document)
        self._init_formats()

    def _init_formats(self) -> None:
        self.fmt_ctrl     = _make_format(COLORS["kw_ctrl"], bold=True)
        self.fmt_decl     = _make_format(COLORS["kw_decl"], bold=True)
        self.fmt_type     = _make_format(COLORS["type"])
        self.fmt_string   = _make_format(COLORS["string"])
        self.fmt_number   = _make_format(COLORS["number"])
        self.fmt_comment  = _make_format(COLORS["comment"], italic=True)
        self.fmt_builtin  = _make_format(COLORS["builtin"])
        self.fmt_ident    = _make_format(COLORS["identifier"])
        self.fmt_operator = _make_format(COLORS["operator"])
        self.fmt_bool     = _make_format(COLORS["bool"], bold=True)

    def highlightBlock(self, text: str) -> None:  # noqa: N802 (Qt-API)
        self.setCurrentBlockState(self.STATE_NORMAL)
        if not text:
            return

        # Kommentare per Hand erkennen -- der Lexer ueberspringt Comments
        # (REM oder ' bis Zeilenende), wir wollen sie aber faerben.
        comment_start = self._find_comment_start(text)
        lex_target = text if comment_start < 0 else text[:comment_start]

        try:
            tokens = Lexer(lex_target).tokenize()
        except LexerError:
            # Kaputte Zeile (z.B. unterminierter String) -- nichts faerben.
            tokens = []

        for tok in tokens:
            if tok.type == TokenType.EOF or tok.type == TokenType.NEWLINE:
                continue
            start, length = self._token_span(text, tok)
            if length <= 0:
                continue
            fmt = self._format_for(tok)
            if fmt is not None:
                self.setFormat(start, length, fmt)

        # f-Strings: der Lexer expandiert sie zu vielen Tokens mit
        # identischer col/end_col-Range, was die Token-Schleife oben
        # zu inkonsistenter Faerbung fuehrt (welcher Token zuletzt schreibt
        # gewinnt). Wir uebermalen darum den gesamten f"..."-Bereich nach
        # dem Token-Pass mit der String-Farbe -- so sehen f-Strings im
        # Editor wie normale Strings aus.
        for fstart, flen in self._find_fstring_ranges(lex_target):
            self.setFormat(fstart, flen, self.fmt_string)

        if comment_start >= 0:
            self.setFormat(comment_start, len(text) - comment_start, self.fmt_comment)

    @staticmethod
    def _find_fstring_ranges(text: str) -> list[tuple[int, int]]:
        """Liefert die (start, length)-Spans aller f-String-Literale in der
        Zeile -- d.h. jedes Vorkommen von `f"..."` (oder `F"..."`).

        Keine Behandlung von Escapes innerhalb der f-Strings; `""` als
        Zeichen-Escape im String wird wie ein Stringende interpretiert
        und neugestartet. Das matched die Lexer-Semantik. Verschachtelte
        Anfuehrungszeichen brauchts hier nicht: f-Strings sind single-line
        und enden beim ersten unescaped `"`.
        """
        ranges: list[tuple[int, int]] = []
        i = 0
        n = len(text)
        in_str = False
        while i < n:
            ch = text[i]
            if in_str:
                if ch == '"':
                    if i + 1 < n and text[i + 1] == '"':
                        i += 2
                        continue
                    in_str = False
                i += 1
                continue
            # f-String: f" oder F" am Wortanfang (kein ident-char davor)
            if ch in ("f", "F") and i + 1 < n and text[i + 1] == '"':
                if i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_"):
                    start = i
                    i += 2  # nach f"
                    while i < n:
                        c2 = text[i]
                        if c2 == '"':
                            if i + 1 < n and text[i + 1] == '"':
                                i += 2
                                continue
                            i += 1
                            break
                        i += 1
                    ranges.append((start, i - start))
                    continue
            if ch == '"':
                in_str = True
                i += 1
                continue
            i += 1
        return ranges

    @staticmethod
    def _find_comment_start(text: str) -> int:
        """Liefert Spalte des Kommentar-Starts in der Zeile -- oder -1.

        GB-Kommentare: `'` oder `REM ` bis zum Zeilenende, sofern nicht in
        einem String. Wir laufen die Zeile ein Mal durch und tracken
        in_string.
        """
        in_string = False
        i = 0
        n = len(text)
        while i < n:
            ch = text[i]
            if in_string:
                if ch == '"':
                    in_string = False
                i += 1
                continue
            if ch == '"':
                in_string = True
                i += 1
                continue
            if ch == "'":
                return i
            # REM gefolgt von Whitespace oder EOL
            if (ch in ("R", "r")
                    and i + 2 < n
                    and text[i:i + 3].lower() == "rem"
                    and (i + 3 == n or not text[i + 3].isalnum() and text[i + 3] != "_")):
                # Nur wenn am Wortanfang (i==0 oder davor kein ident-char)
                if i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_"):
                    return i
            i += 1
        return -1

    @staticmethod
    def _token_span(full_text: str, tok) -> tuple[int, int]:
        """Liefert (start_col, length) eines Tokens in der Zeile.

        `tok.col` und `tok.end_col` sind 1-basiert. Da wir line-by-line
        lexen, liegen Start und Ende immer auf derselben Zeile, und
        `end_col - col` ist die exakte Laenge inkl. Quoten bei Strings.
        """
        start = max(0, tok.col - 1)
        end = min(max(start, tok.end_col - 1), len(full_text))
        return (start, end - start)

    def _format_for(self, tok):
        t = tok.type
        if t == TokenType.STRING:
            return self.fmt_string
        if t == TokenType.NUMBER:
            return self.fmt_number
        if t in (TokenType.TRUE, TokenType.FALSE):
            return self.fmt_bool
        if t in _CONTROL_KW:
            return self.fmt_ctrl
        if t in _TYPE_KW:
            return self.fmt_type
        if t in _DECL_KW:
            return self.fmt_decl
        if t == TokenType.IDENT:
            v = tok.value if isinstance(tok.value, str) else ""
            if v.lower() in BUILTIN_NAMES:
                return self.fmt_builtin
            return self.fmt_ident
        # Operatoren / Punctuation -- in Editor-Default-Foreground belassen.
        return None
