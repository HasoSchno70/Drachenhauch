"""Fehlerklassen fuer GameBasic."""


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

    def set_line(self, line: int) -> "GameBasicError":
        """Setzt nachtraeglich eine Quell-Zeile (nur wenn noch keine gesetzt)
        und aktualisiert die formatierte Meldung. `str(e)` enthaelt die
        Zeile erst dann -- die Exception-Args werden bei __init__ EINMAL
        eingefroren, darum hier neu formatieren. Liefert self (chainbar)."""
        if line and not self.line:
            self.line = line
            self.args = (self._format(),)
        return self


class LexerError(GameBasicError):
    pass


class ParseError(GameBasicError):
    pass


class TypeMismatchError(GameBasicError):
    pass


class GBRuntimeError(GameBasicError):
    pass
