"""Lexer / Tokenizer fuer GameBasic.

Case-insensitive: Keywords UND Bezeichner werden auf Kleinbuchstaben normalisiert.
Kommentare: ' ... bis Zeilenende, oder REM ... bis Zeilenende.
Zeilenumbrueche sind signifikant (Statement-Trenner).
Zeilenfortsetzung:
    1. Explizit:  _\n am Zeilenende.
    2. Implizit:  Innerhalb offener Klammern ( [ wird ein Newline ignoriert,
       genau wie in Python.  Damit lassen sich lange Funktions-Aufrufe oder
       Array-Initialisierer auf mehrere Zeilen verteilen, ohne `_` setzen
       zu muessen.
"""
from .tokens import Token, TokenType, KEYWORDS
from .errors import LexerError


def _is_digit(ch: str) -> bool:
    """ASCII-Ziffer 0-9 -- bewusst NICHT `str.isdigit()`.

    `str.isdigit()` ist auch fuer Hochzahlen ('²', '³') und fremde
    Ziffernsysteme (z.B. arabisch-indisch '٣') True, aber `int()`/`float()`
    akzeptieren nur einen Teil davon. Das erzeugte zwei Fehler auf einmal:
    `PRINT 5²` warf eine ungefangene ValueError (der Highlighter faengt nur
    LexerError ab -> Ausnahme bis in Qts Paint-Pfad), und `PRINT ٣` lexte
    still als NUMBER(3) durch, obwohl dhrt es als "Unbekanntes Zeichen"
    ablehnt. dhrts Lexer nutzt `is_ascii_digit` -- hier dieselbe Regel,
    damit Editor und Laufzeit dieselbe Sprache sehen.
    """
    return "0" <= ch <= "9"


def _radix_int(digits: str, radix: int, what: str, line: int, col: int) -> int:
    """Hex-/Binaer-Literal -> INTEGER ueber die volle 64-Bit-Breite.

    Muss exakt dhrts `Lexer::radix_i64` entsprechen (`lexer.rs`), sonst
    sieht der Editor andere Werte als die Laufzeit. Bitmasken adressieren
    Bitmuster, keine Zahlengroessen: `&HFFFFFFFFFFFFFFFF` ("alle Bits
    gesetzt") wird darum als Zweierkomplement gedeutet und ergibt -1 --
    wie in C, Rust und der BASIC-Tradition. `&H8000000000000000` ist
    zugleich der einzige Weg, i64::MIN als Literal zu schreiben (der
    Dezimal-Pfad lext erst den positiven Betrag, der dann ueberlaeuft).
    Erst jenseits von 64 Bit ist es ein echter Fehler.
    """
    v = int(digits, radix)
    if v >= 1 << 64:
        raise LexerError(f"{what}-Literal zu gross (INTEGER ist 64-bit)", line, col)
    if v >= 1 << 63:
        v -= 1 << 64            # Zweierkomplement
    return v


def _split_fstring_spec(s: str):
    """Trennt einen f-String-Ausdruck am ersten Format-Spec-`:` auf Top-Level.

    `{x:.2f}` -> ("x", ".2f"). Ein `:` innerhalb von ()/[]/{} (z.B. Slice
    `s[1:3]`) oder String-Literalen zaehlt NICHT als Spec-Trenner. Liefert
    (ausdruck, spec) bzw. (ausdruck, None) wenn keine Spec vorhanden ist.
    """
    depth = 0
    in_str = False
    i = 0
    n = len(s)
    while i < n:
        ch = s[i]
        if in_str:
            if ch == '"':
                if i + 1 < n and s[i + 1] == '"':   # "" -> Escape
                    i += 2
                    continue
                in_str = False
        else:
            if ch == '"':
                in_str = True
            elif ch in "([{":
                depth += 1
            elif ch in ")]}":
                depth -= 1
            elif ch == ":" and depth == 0:
                return s[:i].strip(), s[i + 1:].strip()
        i += 1
    return s, None


class Lexer:
    def __init__(self, source: str):
        self.source = source
        self.pos = 0
        self.line = 1
        self.col = 1
        self.tokens: list[Token] = []
        # Klammertiefe fuer implizite Zeilenfortsetzung. Steigt bei ( und [,
        # faellt bei ) und ]. Solange > 0 werden Newlines unterdrueckt.
        self._paren_depth = 0

    def tokenize(self) -> list[Token]:
        while self.pos < len(self.source):
            self._scan_token()
        # Garantiere Schluss-NEWLINE vor EOF, vereinfacht Parser.
        # end_* explizit = start (Null-Breite am Dateiende): der Default 0
        # haette sonst eine Span geliefert, deren Ende vor dem Start liegt.
        if self.tokens and self.tokens[-1].type != TokenType.NEWLINE:
            self.tokens.append(Token(TokenType.NEWLINE, None, self.line, self.col,
                                     self.line, self.col))
        self.tokens.append(Token(TokenType.EOF, None, self.line, self.col,
                                 self.line, self.col))
        return self.tokens

    # ------------------------------------------------------------------
    def _peek(self, offset: int = 0) -> str:
        idx = self.pos + offset
        return self.source[idx] if idx < len(self.source) else ""

    def _advance(self) -> str:
        ch = self.source[self.pos]
        self.pos += 1
        if ch == "\n":
            self.line += 1
            self.col = 1
        else:
            self.col += 1
        return ch

    def _add(self, type_: TokenType, value, line: int, col: int):
        # End-Position = aktuelle Scanner-Position (nach Konsum des Tokens).
        self.tokens.append(Token(type_, value, line, col, self.line, self.col))

    def _error(self, msg: str, line: int, col: int):
        raise LexerError(msg, line, col)

    # ------------------------------------------------------------------
    def _scan_token(self):
        ch = self._peek()
        line, col = self.line, self.col

        # Whitespace ohne Newline
        if ch in " \t\r":
            self._advance()
            return

        # Newline
        if ch == "\n":
            self._advance()
            # Implizite Fortsetzung: Newlines innerhalb offener Klammern
            # werden vom Parser nicht gesehen.
            if self._paren_depth > 0:
                return
            if not self.tokens or self.tokens[-1].type != TokenType.NEWLINE:
                self._add(TokenType.NEWLINE, None, line, col)
            return

        # Zeilenfortsetzung: Unterstrich direkt vor Newline
        if ch == "_" and self._peek(1) in ("\n", "\r"):
            self._advance()
            if self._peek() == "\r":
                self._advance()
            if self._peek() == "\n":
                self._advance()
            return

        # Kommentar mit '
        if ch == "'":
            while self._peek() and self._peek() != "\n":
                self._advance()
            return

        # String
        if ch == '"':
            self._scan_string(line, col)
            return

        # Zahl
        if _is_digit(ch):
            self._scan_number(line, col)
            return

        # Hex- / Binary-Literal: &HFF, &B11010110 (BASIC-Konvention)
        if ch == "&":
            nxt = self._peek(1)
            if nxt in ("H", "h", "B", "b"):
                self._scan_hex_or_binary(line, col)
                return
            self._error("Erwartet H oder B nach '&' (Hex-/Binaer-Literal)",
                        line, col)

        # f-String: f"... {expr} ..."  -> wird zu (STR$(...) + "..." + STR$(...)) expandiert
        if (ch in ("f", "F")) and self._peek(1) == '"':
            self._scan_fstring(line, col)
            return

        # Bezeichner / Keyword
        if ch.isalpha() or ch == "_":
            self._scan_ident(line, col)
            return

        # Operatoren
        self._scan_operator(line, col)

    def _scan_string(self, line: int, col: int):
        self._advance()  # oeffnendes "
        chars: list[str] = []
        while True:
            c = self._peek()
            if c == "":
                self._error("Unterminierter String", line, col)
            if c == "\n":
                self._error("Zeilenumbruch im String nicht erlaubt", line, col)
            if c == '"':
                if self._peek(1) == '"':  # Escape ""
                    self._advance()
                    self._advance()
                    chars.append('"')
                    continue
                self._advance()
                break
            chars.append(self._advance())
        self._add(TokenType.STRING, "".join(chars), line, col)

    def _scan_number(self, line: int, col: int):
        # C-Stil Hex-/Binaer-Literale: 0xFF, 0b1010 (zusaetzlich zu &H/&B).
        # Nur wenn nach dem Praefix eine gueltige Ziffer folgt - sonst normal
        # als Dezimalzahl weiterlexen (0, 0.5 bleiben unveraendert).
        if self._peek() == "0" and self._peek(1) in ("x", "X", "b", "B"):
            prefix = self._peek(1)
            after = self._peek(2)
            if prefix in ("x", "X") and after and (_is_digit(after) or after.lower() in "abcdef"):
                self._advance()              # 0
                self._advance()              # x
                hexchars: list[str] = []
                while True:
                    c = self._peek()
                    if c and (_is_digit(c) or c.lower() in "abcdef"):
                        hexchars.append(self._advance())
                    else:
                        break
                self._add(TokenType.NUMBER,
                          _radix_int("".join(hexchars), 16, "Hex", line, col), line, col)
                return
            if prefix in ("b", "B") and after in ("0", "1"):
                self._advance()              # 0
                self._advance()              # b
                binchars: list[str] = []
                while self._peek() in ("0", "1"):
                    binchars.append(self._advance())
                self._add(TokenType.NUMBER,
                          _radix_int("".join(binchars), 2, "Binaer", line, col), line, col)
                return
        chars: list[str] = []
        while _is_digit(self._peek()):
            chars.append(self._advance())
        is_float = False
        if self._peek() == "." and _is_digit(self._peek(1)):
            is_float = True
            chars.append(self._advance())
            while _is_digit(self._peek()):
                chars.append(self._advance())
        text = "".join(chars)
        value = float(text) if is_float else int(text)
        self._add(TokenType.NUMBER, value, line, col)

    def _scan_fstring(self, line: int, col: int):
        """f"text {expr} text..."  -> emits ('text' + STR$(expr) + 'text' + ...)
        als Token-Sequenz.

        - {{ und }} sind Escapes fuer wortlich { bzw. }.
        - Jeder Ausdruck zwischen { } wird mit STR$(...) automatisch
          stringifiziert, damit `f"score: {x}"` auch fuer INTEGER `x`
          funktioniert.
        - Ein Ausdruck darf Klammern und Funktionsaufrufe enthalten.
        - Grenze: der Platzhalter-Scanner zaehlt nur Klammern, er kennt
          keine String-Literale. Ein `{`/`}` INNERHALB eines Strings im
          Platzhalter (z.B. f"{REPLACE$(s, "}", "-")}") beendet den
          Ausdruck darum zu frueh. dhrt hat dieselbe Grenze, das Programm
          scheitert also ohnehin -- Editor und Laufzeit sind sich einig.
        """
        self._advance()              # 'f'
        self._advance()              # '"'

        parts: list = []             # ('text', str) oder ('expr', str)
        cur: list[str] = []
        while True:
            c = self._peek()
            if c == "":
                self._error("Unterminierter f-String", line, col)
            if c == "\n":
                self._error("Zeilenumbruch im f-String nicht erlaubt", line, col)
            if c == '"':
                if self._peek(1) == '"':
                    cur.append('"')
                    self._advance()
                    self._advance()
                    continue
                self._advance()
                if cur:
                    parts.append(("text", "".join(cur)))
                break
            if c == "{":
                if self._peek(1) == "{":
                    cur.append("{")
                    self._advance()
                    self._advance()
                    continue
                if cur:
                    parts.append(("text", "".join(cur)))
                    cur = []
                self._advance()      # konsumiert {
                expr_chars: list[str] = []
                depth = 1
                while True:
                    cc = self._peek()
                    if cc == "":
                        self._error("Unterminierter Ausdruck im f-String", line, col)
                    if cc == "\n":
                        self._error("Zeilenumbruch im f-String-Ausdruck nicht erlaubt", line, col)
                    if cc == "{":
                        depth += 1
                    elif cc == "}":
                        depth -= 1
                        if depth == 0:
                            self._advance()
                            break
                    expr_chars.append(self._advance())
                # strip(): `f"{ }"` (nur Whitespace) rutschte an dieser
                # Pruefung vorbei und emittierte ein argumentloses `str$()`,
                # das erst spaeter als verwirrender Parse-Fehler auffiel.
                if not "".join(expr_chars).strip():
                    self._error("Leerer f-String-Ausdruck {}", line, col)
                parts.append(("expr", "".join(expr_chars)))
                continue
            if c == "}":
                if self._peek(1) == "}":
                    cur.append("}")
                    self._advance()
                    self._advance()
                    continue
                self._error("Einzelnes '}' im f-String (verwende '}}')", line, col)
            cur.append(self._advance())

        # Token-Sequenz emittieren: ( "text" + STR$(expr) + "text" + ... )
        # Wenn keine Parts: leerer String
        if not parts:
            self._add(TokenType.STRING, "", line, col)
            return

        self._add(TokenType.LPAREN, "(", line, col)
        first = True
        for kind, text in parts:
            if not first:
                self._add(TokenType.PLUS, "+", line, col)
            first = False
            if kind == "text":
                self._add(TokenType.STRING, text, line, col)
            else:
                # Format-Spec? `{x:.2f}` -> FORMAT$(x, "%.2f"); sonst STR$(x).
                expr_src, spec = _split_fstring_spec(text)
                if spec is not None and not spec:
                    # `f"{x:}"` -- Doppelpunkt ohne Spec. Frueher fiel das
                    # durch die truthiness-Pruefung in den str$-Zweig, wobei
                    # `expr_src = text` den Doppelpunkt WIEDER hereinholte
                    # -> `str$(x :)` mit Streu-COLON und einem Folgefehler
                    # weit weg von der Ursache.
                    self._error("Leere Format-Spec im f-String ({x:})", line, col)
                if spec:
                    fn_name, suffix = "format$", spec
                else:
                    fn_name, suffix = "str$", None
                    expr_src = text          # unveraendertes Verhalten ohne Spec
                self._add(TokenType.IDENT, fn_name, line, col)
                self._add(TokenType.LPAREN, "(", line, col)
                inner = Lexer(expr_src).tokenize()
                # Strip trailing NEWLINE + EOF
                while inner and inner[-1].type in (TokenType.NEWLINE, TokenType.EOF):
                    inner.pop()
                # Line/col ueberschreiben damit Fehlermeldungen auf den
                # f-String-Ort zeigen, nicht auf den synthetischen 1:1 input.
                # WICHTIG auch end_line/end_col mitziehen: die stammten sonst
                # aus dem Sub-Lexer (Zeile 1) und lagen damit VOR dem Start
                # -- eine invertierte Span. Der Highlighter clamped das zwar
                # weg, aber jeder Konsument ohne Clamp (LSP-Semantic-Tokens,
                # Hover, Selection-Ranges) haette einen rueckwaerts laufenden
                # Bereich bekommen.
                for tok in inner:
                    tok.line = line
                    tok.col = col
                    tok.end_line = self.line
                    tok.end_col = self.col
                self.tokens.extend(inner)
                if suffix is not None:
                    self._add(TokenType.COMMA, ",", line, col)
                    self._add(TokenType.STRING, "%" + suffix, line, col)
                self._add(TokenType.RPAREN, ")", line, col)
        self._add(TokenType.RPAREN, ")", line, col)

    def _scan_hex_or_binary(self, line: int, col: int):
        """&HFF -> 255 (Hex), &B11010110 -> 214 (Binary).

        Klassisch fuer BASIC.  Underscores im Literal sind nicht erlaubt
        (anders als Python - GameBasic bleibt strikt).
        """
        self._advance()              # &
        marker = self._advance()     # H oder B
        chars: list[str] = []
        if marker in ("H", "h"):
            while True:
                c = self._peek()
                # Wichtig: leerer String "" ist Substring von "abcdef" - daher
                # explizit pruefen ob c ueberhaupt da ist.
                if c and (_is_digit(c) or c.lower() in "abcdef"):
                    chars.append(self._advance())
                else:
                    break
            if not chars:
                self._error("Hex-Literal &H ohne Ziffern", line, col)
            value = _radix_int("".join(chars), 16, "Hex", line, col)
        else:                        # B/b
            while self._peek() in ("0", "1"):
                chars.append(self._advance())
            if not chars:
                self._error("Binaer-Literal &B ohne Ziffern", line, col)
            value = _radix_int("".join(chars), 2, "Binaer", line, col)
        self._add(TokenType.NUMBER, value, line, col)

    def _scan_ident(self, line: int, col: int):
        chars: list[str] = []
        while self._peek().isalnum() or self._peek() == "_":
            chars.append(self._advance())
        # Optionales Typsuffix $ (klassisch fuer String-Funktionen wie STR$, CHR$)
        if self._peek() == "$":
            chars.append(self._advance())
        text = "".join(chars).lower()

        # REM ist Komentar bis Zeilenende
        if text == "rem":
            while self._peek() and self._peek() != "\n":
                self._advance()
            return

        ttype = KEYWORDS.get(text, TokenType.IDENT)
        if ttype == TokenType.TRUE:
            self._add(TokenType.TRUE, True, line, col)
        elif ttype == TokenType.FALSE:
            self._add(TokenType.FALSE, False, line, col)
        else:
            self._add(ttype, text, line, col)

    def _scan_operator(self, line: int, col: int):
        ch = self._advance()
        if ch == "+":
            if self._peek() == "=":
                self._advance()
                self._add(TokenType.PLUS_EQ, "+=", line, col)
            else:
                self._add(TokenType.PLUS, "+", line, col)
        elif ch == "-":
            if self._peek() == "=":
                self._advance()
                self._add(TokenType.MINUS_EQ, "-=", line, col)
            else:
                self._add(TokenType.MINUS, "-", line, col)
        elif ch == "*":
            if self._peek() == "=":
                self._advance()
                self._add(TokenType.STAR_EQ, "*=", line, col)
            else:
                self._add(TokenType.STAR, "*", line, col)
        elif ch == "/":
            if self._peek() == "=":
                self._advance()
                self._add(TokenType.SLASH_EQ, "/=", line, col)
            else:
                self._add(TokenType.SLASH, "/", line, col)
        elif ch == "\\":
            self._add(TokenType.INTDIV, "\\", line, col)
        elif ch == "^":
            self._add(TokenType.CARET, "^", line, col)
        elif ch == "(":
            self._paren_depth += 1
            self._add(TokenType.LPAREN, "(", line, col)
        elif ch == ")":
            if self._paren_depth > 0:
                self._paren_depth -= 1
            self._add(TokenType.RPAREN, ")", line, col)
        elif ch == "[":
            self._paren_depth += 1
            self._add(TokenType.LBRACKET, "[", line, col)
        elif ch == "]":
            if self._paren_depth > 0:
                self._paren_depth -= 1
            self._add(TokenType.RBRACKET, "]", line, col)
        elif ch == "{":
            self._paren_depth += 1
            self._add(TokenType.LBRACE, "{", line, col)
        elif ch == "}":
            if self._paren_depth > 0:
                self._paren_depth -= 1
            self._add(TokenType.RBRACE, "}", line, col)
        elif ch == ",":
            self._add(TokenType.COMMA, ",", line, col)
        elif ch == ".":
            # `...` -> ELLIPSIS (Variadic-Marker in Param-Listen).
            # Reines `..` wird als zwei einzelne DOTs emittiert -- der
            # Parser akzeptiert das nirgends, gibt also einen klaren Fehler.
            if self._peek() == "." and self._peek(1) == ".":
                self._advance()
                self._advance()
                self._add(TokenType.ELLIPSIS, "...", line, col)
            else:
                self._add(TokenType.DOT, ".", line, col)
        elif ch == ";":
            self._add(TokenType.SEMICOLON, ";", line, col)
        elif ch == ":":
            self._add(TokenType.COLON, ":", line, col)
        elif ch == "=":
            self._add(TokenType.EQ, "=", line, col)
        elif ch == "<":
            if self._peek() == "=":
                self._advance()
                self._add(TokenType.LEQ, "<=", line, col)
            elif self._peek() == ">":
                self._advance()
                self._add(TokenType.NEQ, "<>", line, col)
            else:
                self._add(TokenType.LT, "<", line, col)
        elif ch == ">":
            if self._peek() == "=":
                self._advance()
                self._add(TokenType.GEQ, ">=", line, col)
            else:
                self._add(TokenType.GT, ">", line, col)
        else:
            self._error(f"Unbekanntes Zeichen: {ch!r}", line, col)
