"""Fehlerklassen fuer GameBasic.

Nur noch drei Typen: `LexerError` und `ParseError` aus der Editor-Schicht
(Lexer/Parser bedienen Highlighting, LSP, Error-Check, Folding, Formatter),
`GBRuntimeError` fuer alles, was `conftest.run_gb` aus dhrts stderr
zurueckuebersetzt.

Entfernt: `TypeMismatchError` (seit dem Wegfall des Python-Tree-Walkers kann
es niemand mehr werfen -- Typpruefung passiert ausschliesslich in dhrt) und
`GameBasicError.set_line()` (hatte projektweit keinen einzigen Aufrufer).
"""


class GameBasicError(Exception):
    def __init__(self, message: str, line: int = 0, col: int = 0):
        self.message = message
        self.line = line
        self.col = col
        super().__init__(self._format())

    def _format(self):
        kind = self.__class__.__name__.replace("_", "")
        if self.line:
            return f"[Zeile {self.line}] {kind}: {self.message}"
        return f"{kind}: {self.message}"


class LexerError(GameBasicError):
    pass


class ParseError(GameBasicError):
    pass


class GBRuntimeError(GameBasicError):
    pass
