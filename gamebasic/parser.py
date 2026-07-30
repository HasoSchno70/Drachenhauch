"""Recursive-Descent-Parser fuer GameBasic (Editor-Schicht).

WICHTIG zum Einordnen: dieser Parser fuehrt NICHTS mehr aus. Seit dem
Entfernen des Python-Tree-Walkers (Stufe B) ist `gbrt` die einzige
Laufzeit und bringt sein eigenes Front-End mit
(`rust/gb_runtime/src/parser.rs`). Was hier entsteht, bedient nur noch
die Editor-/Tooling-Schicht: LSP, Live-Error-Check (als FALLBACK, wenn
`gbrt --check` nicht auffindbar ist), Folding und Formatter. Das Outline
ist textbasiert (`editor_qt/symbols.py`) und nutzt den AST gar nicht.

Daraus folgt die Leitregel fuer Aenderungen hier: **akzeptiere genau das,
was gbrt auch ausfuehrt.** Zu streng heisst, der Editor streicht
laufenden Code rot an (und weil der Fallback nur das ERSTE Problem
liefert, verdeckt ein Fehlalarm alle echten Fehler der Datei); zu lax
heisst, der Editor schweigt und das Programm scheitert erst zur Laufzeit.
Im Zweifel gegen `gbrt --check` gegenpruefen, nicht raten.

Die vollstaendige Grammatik steht in CLAUDE.md (Klassen, PROPERTY,
OPERATOR, ENUM, SELECT CASE mit Guards, WITH, TRY, Comprehensions,
Coroutinen/YIELD, Tupel-Destructuring, Slicing, IIF, ...) -- eine
Kurzfassung hier zu pflegen ist mehrfach veraltet (sie beschrieb noch den
"Phase 1"-Sprachstand ohne Klassen).

Ausdrucks-Praezedenz (niedrig -> hoch):
  OR -> AND -> NOT -> Vergleich -> Bitwise-Binaer -> +,- -> *,/,MOD ->
    unaer +,-,BNOT -> ^ -> postfix -> primary

Die Bitwise-Binaer-Operatoren (BAND, BOR, BXOR, SHL, SHR) bilden EINE
Praezedenz-Ebene mit Links-Assoziativitaet. Wer C-aehnliche Reihenfolge
will (BAND < BXOR < BOR), klammert explizit. Das spart 4 Parser-Funktionen
und macht `IF flags BAND MASK THEN` lesbar genug ohne Klammer-Wirrwarr.
"""
from .tokens import Token, TokenType, KEYWORDS
from .errors import ParseError
from .ast_nodes import (
    NumberLit, StringLit, BoolLit, NilLit, Identifier, BinaryOp, UnaryOp, Call,
    Dim, MultiDim, Assign, Print, Input, If, While, For, ForEach, ExprStmt, Program,
    Param, SubDecl, FunctionDecl, Return,
    ClassDecl, New, MemberAccess, MemberAssign,
    Const, Break, Continue, IndexAccess, IndexAssign,
    Try, Throw, Select, CaseMatch,
    Repeat, Data, Read, Restore,
    EnumDecl, NamedArg, TupleLit, TupleAssign, With, SliceAccess,
    PropertyDecl, ListComp, DictComp, SetComp, ArrayLit, TernaryExpr, Yield,
)


# Zuweisungs-Operatoren: einfaches = und Compound-Forms (a += b etc.).
# Zweite Spalte ist der binaere Operator zum Desugaring (None = einfache
# Zuweisung).
_ASSIGN_OPS = {
    TokenType.EQ:       None,
    TokenType.PLUS_EQ:  "+",
    TokenType.MINUS_EQ: "-",
    TokenType.STAR_EQ:  "*",
    TokenType.SLASH_EQ: "/",
}


_TYPE_TOKENS = {
    TokenType.INTEGER: "integer",
    TokenType.FLOAT: "float",
    TokenType.STRING_TYPE: "string",
    TokenType.BOOLEAN: "boolean",
    TokenType.IMAGE: "image",
    TokenType.SOUND: "sound",
    TokenType.FILE: "file",
    TokenType.TUPLE: "tuple",
    TokenType.FUNCREF: "funcref",
    TokenType.COROUTINE: "coroutine",
}


class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.pos = 0
        # Beim Parsen gesammelte ENUM-Namen (lower-case). Wird benutzt, um
        # `DIM x AS Color` als `DIM x AS INTEGER` zu behandeln, ohne dass der
        # Interpreter/VM den Enum-Namen kennen muss. Forward-Declarations
        # sind bewusst nicht unterstuetzt - das Enum muss vor seiner
        # Verwendung deklariert sein.
        self._enum_names: set = set()
        # WITH-Block-Stack: Compiler-generierte Variablen-Namen, in denen
        # das jeweilige WITH-Ziel gespeichert ist. `.member`-Shortcuts im
        # Body werden zu `MemberAccess(Identifier(top_of_stack), name)`
        # de-sugared.
        self._with_counter: int = 0
        self._with_stack: list = []
        # Verschachtelungstiefe der Ausdrucks-Rekursion (siehe _expression).
        self._expr_depth: int = 0

    # ---- Helfer ------------------------------------------------------
    def _peek(self, offset=0) -> Token:
        idx = self.pos + offset
        if idx >= len(self.tokens):
            return self.tokens[-1]
        return self.tokens[idx]

    def _at_end(self) -> bool:
        return self._peek().type == TokenType.EOF

    def _check(self, *types) -> bool:
        return self._peek().type in types

    def _match(self, *types):
        if self._check(*types):
            tok = self._peek()
            self.pos += 1
            return tok
        return None

    def _expect(self, type_, msg=None) -> Token:
        if self._check(type_):
            tok = self._peek()
            self.pos += 1
            return tok
        actual = self._peek()
        raise ParseError(
            msg or f"Erwartet {type_.name}, gefunden {actual.type.name}",
            actual.line, actual.col,
        )

    def _skip_newlines(self):
        # Akzeptiert auch leere Doppelpunkt-Statements (`: : :`) - die haben
        # keinen Effekt und werden behandelt wie Leerzeilen.
        while self._match(TokenType.NEWLINE) or self._match(TokenType.COLON):
            pass

    def _consume_terminator(self):
        # Sowohl Newline als auch Doppelpunkt sind Statement-Trenner
        # (klassisches BASIC: `x = 1 : y = 2`).  Mehrere hintereinander
        # werden geschluckt, damit `: \n :` keinen Stoerfaktor bildet.
        if self._check(TokenType.NEWLINE) or self._check(TokenType.COLON):
            while self._match(TokenType.NEWLINE) or self._match(TokenType.COLON):
                pass
            return
        if self._at_end():
            return
        tok = self._peek()
        raise ParseError(
            f"Erwartet Zeilenende, gefunden {tok.type.name}",
            tok.line, tok.col,
        )

    # ---- Eintritt ----------------------------------------------------
    def parse(self) -> Program:
        self._skip_newlines()
        stmts = []
        while not self._at_end():
            stmts.append(self._statement())
            self._skip_newlines()
        return Program(stmts)

    # ---- Statements --------------------------------------------------
    def _statement(self):
        # Beginn-Zeile fuers AST-Knoten merken (fuer Fehlermeldungen).
        start_line = self._peek().line
        stmt = self._statement_inner()
        try:
            stmt.line = start_line
        except (AttributeError, TypeError):
            pass
        return stmt

    def _statement_inner(self):
        tok = self._peek()
        t = tok.type
        if t == TokenType.DIM:
            return self._dim()
        if t == TokenType.PRINT:
            return self._print()
        if t == TokenType.INPUT:
            return self._input()
        if t == TokenType.IF:
            return self._if()
        if t == TokenType.SELECT:
            return self._select()
        if t == TokenType.WHILE:
            return self._while()
        if t == TokenType.REPEAT:
            return self._repeat()
        if t == TokenType.FOR:
            return self._for()
        if t == TokenType.DATA:
            return self._data()
        if t == TokenType.READ:
            return self._read()
        if t == TokenType.RESTORE:
            return self._restore()
        if t == TokenType.SUB:
            return self._sub_decl()
        if t == TokenType.FUNCTION:
            return self._function_decl()
        if t == TokenType.RETURN:
            return self._return()
        if t == TokenType.CLASS:
            return self._class_decl()
        if t == TokenType.STRUCT:
            return self._struct_decl()
        if t == TokenType.CONST:
            return self._const()
        if t == TokenType.ENUM:
            return self._enum_decl()
        if t == TokenType.BREAK:
            return self._break()
        if t == TokenType.CONTINUE:
            return self._continue()
        if t == TokenType.TRY:
            return self._try_stmt()
        if t == TokenType.THROW:
            return self._throw_stmt()
        if t == TokenType.WITH:
            return self._with_stmt()
        # Innerhalb eines WITH-Blocks: `.member = expr` oder `.member.x = expr`
        # ist ein Assignment auf das aktuelle WITH-Ziel.
        if t == TokenType.DOT and self._with_stack:
            return self._dot_assign_in_with()
        # Zuweisungs-Erkennung: IDENT (.IDENT)* '='
        if t == TokenType.IDENT and self._is_assignment_lookahead():
            return self._assign()
        # Tupel-Destructuring: `(x, y, z) = expr`
        # Lookahead: `( IDENT (.IDENT|[expr])* (, IDENT (.IDENT|[expr])*)+ ) =`
        if t == TokenType.LPAREN and self._is_tuple_assign_lookahead():
            return self._tuple_assign()
        # Fallback: Ausdrucks-Statement
        expr = self._expression()
        # Sicherheitsnetz: ein Top-Level `=` ist fast immer eine gemeinte
        # ZUWEISUNG, deren Ziel der Lookahead nicht erkannt hat -- der
        # Ausdruck wuerde sonst als Vergleich geparst, sein Ergebnis
        # verworfen und die Zuweisung verschwaende spurlos. Genau diese
        # Stille liess die Keyword-Member-Faelle (`spr.image = 5`) so lange
        # unentdeckt. gbrt hat denselben Wortlaut (parser.rs:171/783).
        if isinstance(expr, BinaryOp) and expr.op == "=":
            raise ParseError(
                "'=' als Anweisung -- meintest du eine Zuweisung?",
                tok.line, tok.col,
            )
        self._consume_terminator()
        return ExprStmt(expr)

    def _is_tuple_assign_lookahead(self) -> bool:
        """Lookahead fuer `(x, y[, ...]) = expr`. Beginnt mit LPAREN.

        Wir scannen die Klammer-Inhalte ueber ausgewogene `(` `[` und pruefen,
        ob nach dem schliessenden `)` ein `=` folgt UND ob mindestens ein
        Komma auf der OBERSTEN Klammer-Ebene auftritt. Das letzte Kriterium
        unterscheidet Tupel-Assign von z.B. `(a + b) = ...` (kein Komma) oder
        einer Klammer-Expression.

        Der eigentliche Inhalt wird hier nicht validiert -- `_tuple_assign`
        ruft `_assign_lvalue_expr` pro Target und wirft saubere Fehler.
        """
        i = 1   # nach LPAREN
        depth = 1
        saw_top_comma = False
        while True:
            tt = self._peek(i).type
            if tt == TokenType.EOF or tt == TokenType.NEWLINE:
                return False
            if tt == TokenType.LPAREN or tt == TokenType.LBRACKET:
                depth += 1
            elif tt == TokenType.RPAREN or tt == TokenType.RBRACKET:
                depth -= 1
                if depth == 0:
                    break
            elif tt == TokenType.COMMA and depth == 1:
                saw_top_comma = True
            i += 1
        # nach `)` muss `=` kommen
        return saw_top_comma and self._peek(i + 1).type == TokenType.EQ

    def _tuple_assign(self, consume_term: bool = True):
        """Parst `(target, target, ...) = expr`.

        `consume_term=False` fuer den Single-Line-IF-Pfad (siehe
        `_dot_assign_in_with`).
        """
        self._expect(TokenType.LPAREN)
        targets = [self._tuple_assign_target()]
        while self._match(TokenType.COMMA):
            targets.append(self._tuple_assign_target())
        if len(targets) < 2:
            raise ParseError(
                "Tupel-Destructuring erwartet mindestens 2 Ziele",
                self._peek().line, self._peek().col,
            )
        self._expect(TokenType.RPAREN)
        self._expect(TokenType.EQ, "Erwartet '=' nach Tupel-Zielen")
        value = self._expression()
        if consume_term:
            self._consume_terminator()
        return TupleAssign(targets, value)

    def _tuple_assign_target(self):
        """Ein einzelnes Ziel in Tupel-Destructuring. Erlaubt:
            x                       -> Identifier
            x.y                     -> MemberAccess
            x[i], x[i, j]           -> IndexAccess
            x.y[i].z                -> Kette davon

        Compound-Operatoren machen hier keinen Sinn -- es ist immer einfache
        Zuweisung.
        """
        name_tok = self._expect(TokenType.IDENT, "Erwartet Variablenname als Tupel-Ziel")
        return self._lvalue_chain(Identifier(name_tok.value))

    def _member_name_after_dot(self) -> str:
        """Membername nach `.` -- akzeptiert IDENT UND Keywords.

        Beim qualifizierten Zugriff (`obj.member`) gibt es keine
        Mehrdeutigkeit mit Sprach-Keywords, deshalb muessen Felder, die
        zufaellig wie ein Keyword lexen (`.image`, `.sound`, `.data`,
        `.next`, ...), funktionieren. `_postfix` (Lesen) war schon so
        tolerant, die drei Zuweisungs-Pfade dagegen verlangten IDENT --
        `spr.image = x` wurde dadurch nicht einmal als Zuweisung ERKANNT
        und landete still als verworfener `=`-Vergleich im AST.
        Spiegelt `Parser::member_name_after_dot` in parser.rs, wo derselbe
        Fund bereits gefixt wurde.
        """
        tok = self._peek()
        if isinstance(tok.value, str) and tok.value:
            self.pos += 1
            return tok.value
        raise ParseError(
            f"Erwartet Membername nach '.', gefunden {tok.type.name}",
            tok.line, tok.col,
        )

    def _lvalue_chain(self, target_expr):
        """`(.member | [index, ...])*` auf einem schon geparsten Ziel.

        EINE Stelle fuer alle Zuweisungs-Kontexte (`_assign_from_lvalue`,
        `_tuple_assign_target`, `_dot_assign_in_with`). Vorher existierte
        diese Schleife dort dreimal nebeneinander und die Kopien waren
        bereits auseinandergelaufen: nur eine akzeptierte Keyword-Member-
        namen, nur eine hatte die freundliche Slice-Zuweisungs-Meldung,
        und die Slice-Pruefung deckte nur den ERSTEN Index ab
        (`x[1, 2:3] = 5` fiel darum in ein generisches "Erwartet ']'").

        Aufrufe `(...)` gehoeren bewusst NICHT hierher -- ein Funktions-
        aufruf ist kein Zuweisungsziel. Der Lese-Pfad `_postfix` bleibt
        deshalb getrennt (er kann zusaetzlich Calls und Slices).
        """
        while True:
            if self._match(TokenType.DOT):
                target_expr = MemberAccess(target_expr, self._member_name_after_dot())
            elif self._match(TokenType.LBRACKET):
                indices = []
                while True:
                    # Slice-Zuweisung (`arr[1:5] = ...`) ist nicht unterstuetzt
                    # (Laenge muss matchen? Truncate?) -- klar melden statt
                    # generischem "Erwartet ']'". Gilt fuer JEDEN Index, nicht
                    # nur den ersten.
                    if self._check(TokenType.COLON):
                        raise ParseError(
                            "Slice-Zuweisung (`x[a:b] = ...`) ist nicht "
                            "unterstuetzt -- nutze eine Schleife.",
                            self._peek().line, self._peek().col,
                        )
                    indices.append(self._expression())
                    if self._check(TokenType.COLON):
                        raise ParseError(
                            "Slice-Zuweisung (`x[a:b] = ...`) ist nicht "
                            "unterstuetzt -- nutze eine Schleife.",
                            self._peek().line, self._peek().col,
                        )
                    if not self._match(TokenType.COMMA):
                        break
                self._expect(TokenType.RBRACKET, "Erwartet ']'")
                target_expr = IndexAccess(target_expr, indices)
            else:
                return target_expr

    def _is_assignment_lookahead(self) -> bool:
        i = 1
        while True:
            t = self._peek(i).type
            # Membername: wie `_member_name_after_dot` auch Keywords zulassen
            # -- sonst wird `spr.image = 5` gar nicht erst als Zuweisung
            # erkannt und verschwindet still als `=`-Vergleich.
            nxt = self._peek(i + 1)
            if t == TokenType.DOT and isinstance(nxt.value, str) and nxt.value:
                i += 2
                continue
            if t == TokenType.LBRACKET:
                # ueberspringe ausgewogene [ ... ]
                depth = 1
                i += 1
                while depth > 0:
                    tt = self._peek(i).type
                    if tt == TokenType.EOF:
                        return False
                    if tt == TokenType.LBRACKET:
                        depth += 1
                    elif tt == TokenType.RBRACKET:
                        depth -= 1
                    i += 1
                continue
            break
        return self._peek(i).type in _ASSIGN_OPS

    def _dim(self):
        """Parst ein DIM-Statement.

        Single:  `DIM x AS INTEGER`             -> Dim
                 `DIM x[10] AS INTEGER`         -> Dim mit array_dims=[10]
                 `DIM x[10, 20] AS INTEGER`     -> Dim mit array_dims=[10, 20]
        Multi:   `DIM a, b, c AS INTEGER`       -> MultiDim mit drei Dim-Eintraegen
                 `DIM x[10], y, z[5] AS FLOAT`  -> MultiDim mit gemischten Skalaren/Arrays

        Bei Single-DIM gibt's einen Dim-Node zurueck (Backward-Compat). Bei
        Multi-DIM einen MultiDim-Node, der vom Interpreter und Compiler
        ueber bestehende Single-Dim-Logik abgewickelt wird.
        """
        self._expect(TokenType.DIM)
        # Pro Variable: (name, array_dims-Liste-oder-None). array_dims kann
        # eindimensional ("DIM x[10]") oder mehrdimensional ("DIM x[10, 20]")
        # sein - das innere RBRACKET schliesst den Dimension-Block ab, ein
        # nachfolgendes COMMA leitet eine NEUE Variable ein.
        decls: list = []
        while True:
            nxt = self._peek()
            if (nxt.type != TokenType.IDENT and isinstance(nxt.value, str)
                    and nxt.value in KEYWORDS):
                raise ParseError(
                    f"'{nxt.value.upper()}' ist ein reserviertes Wort und kann "
                    f"kein Variablenname sein - waehle einen anderen Namen",
                    nxt.line, nxt.col)
            name_tok = self._expect(TokenType.IDENT,
                                     "Erwartet Variablenname nach DIM")
            array_dims = None
            if self._match(TokenType.LBRACKET):
                array_dims = [self._expression()]
                while self._match(TokenType.COMMA):
                    array_dims.append(self._expression())
                self._expect(TokenType.RBRACKET, "Erwartet ']'")
            decls.append((name_tok.value, array_dims))
            # Nach Komma kommt eine weitere Variable. Sonst AS.
            if not self._match(TokenType.COMMA):
                break
        self._expect(TokenType.AS, "Erwartet AS nach Variablenname")
        type_name = self._parse_type()
        self._consume_terminator()
        if len(decls) == 1:
            name, dims = decls[0]
            return Dim(name, type_name, dims)
        return MultiDim([Dim(name, type_name, dims) for name, dims in decls])

    def _const(self):
        self._expect(TokenType.CONST)
        name_tok = self._expect(TokenType.IDENT, "Erwartet Konstantenname nach CONST")
        type_name = None
        if self._match(TokenType.AS):
            type_name = self._parse_type()
        self._expect(TokenType.EQ, "Erwartet '=' nach CONST-Name")
        value = self._expression()
        self._consume_terminator()
        return Const(name_tok.value, type_name, value)

    def _enum_decl(self):
        """Parst ein ENUM. Zwei Schreibweisen:

          Compact:   ENUM Name = M1, M2, M3
          Block:     ENUM Name
                       M1
                       M2 = 5
                       M3
                     END ENUM

        Members ohne explizite Wert-Zuweisung werden auto-nummeriert: 0, 1,
        2, ...  Bei expliziten Werten zaehlt der naechste implizite Member
        von diesem Wert + 1 weiter.

        Member-Namen duerfen auch GB-Keywords sein (READ, DATA, FILE etc.) -
        weil sie ueber `EnumName.Member` qualified zugegriffen werden, gibt
        es keine Mehrdeutigkeit. So kann man auch ENUM-Member wie NONE,
        READ, WRITE, EXEC fuer Datei-Permissions benutzen."""
        enum_tok = self._expect(TokenType.ENUM)
        name_tok = self._expect(TokenType.IDENT, "Erwartet ENUM-Name")
        members: list = []

        if self._match(TokenType.EQ):
            # Compact-Form: ENUM Name = M1, M2, M3
            while True:
                m_name = self._consume_member_name()
                value_expr = None
                if self._match(TokenType.EQ):
                    value_expr = self._expression()
                members.append((m_name, value_expr))
                if not self._match(TokenType.COMMA):
                    break
            self._consume_terminator()
        else:
            # Block-Form: bis END ENUM
            self._consume_terminator()
            while True:
                self._skip_newlines()
                if self._check(TokenType.END):
                    break
                if self._at_end():
                    raise ParseError(
                        "END ENUM erwartet",
                        enum_tok.line, enum_tok.col,
                    )
                m_name = self._consume_member_name()
                value_expr = None
                if self._match(TokenType.EQ):
                    value_expr = self._expression()
                members.append((m_name, value_expr))
                # Member-Trenner: Newline oder Komma (beide tolerieren)
                if not self._match(TokenType.COMMA):
                    self._consume_terminator()
            self._expect(TokenType.END)
            self._expect(TokenType.ENUM, "Erwartet ENUM nach END")
            self._consume_terminator()

        if not members:
            raise ParseError(
                f"ENUM {name_tok.value}: mindestens ein Member erforderlich",
                enum_tok.line, enum_tok.col,
            )
        # Member-Namen muessen pro Enum eindeutig sein
        seen = set()
        for mname, _ in members:
            key = mname.lower()
            if key in seen:
                raise ParseError(
                    f"ENUM {name_tok.value}: Member '{mname}' doppelt deklariert",
                    enum_tok.line, enum_tok.col,
                )
            seen.add(key)

        self._enum_names.add(name_tok.value.lower())
        return EnumDecl(name_tok.value, members)

    def _call_args(self) -> list:
        """Parst die Argument-Liste eines Funktionsaufrufs (ohne die Klammern).

        Akzeptiert Mix aus positional und named:
            func()                              -> []
            func(1, 2)                          -> [1, 2]
            func(x: 5)                          -> [NamedArg("x", 5)]
            func(1, 2, name: "Anna")            -> [1, 2, NamedArg("name", "Anna")]
            func(name: "x", 5)                  -> Fehler im Caller (Reihenfolge)

        Wir akzeptieren hier alle Reihenfolgen - die Validierung "positional
        muss vor named kommen" macht gbrts Compiler, weil
        er den Funktions-Kontext fuer schoenere Fehlermeldungen hat.
        """
        args: list = []
        if self._check(TokenType.RPAREN):
            return args
        args.append(self._call_arg())
        while self._match(TokenType.COMMA):
            args.append(self._call_arg())
        return args

    def _call_arg(self):
        """Ein einzelnes Argument: Ausdruck ODER `name: Ausdruck`.

        Lookahead: IDENT COLON -> NamedArg; sonst normale Expression.
        Wichtig: nur IDENT (nicht beliebige Tokens), damit man named-args
        nicht aus Versehen mit Keyword-Konflikten triggert. Auch nicht in
        verschachtelten Klammern - der Parser prueft das _direkt_ zu Beginn
        des Arguments.
        """
        if (self._peek(0).type == TokenType.IDENT
                and self._peek(1).type == TokenType.COLON):
            name = self._peek(0).value
            self.pos += 2  # IDENT + COLON
            value = self._expression()
            return NamedArg(name, value)
        return self._expression()

    def _consume_member_name(self) -> str:
        """Konsumiert ein ENUM-Member-Namen-Token. Akzeptiert IDENT und
        Keywords gleichermassen - beim qualifizierten Zugriff (`Name.Member`)
        gibt es keine Mehrdeutigkeit mit Sprach-Keywords."""
        tok = self._peek()
        if isinstance(tok.value, str) and tok.value:
            self.pos += 1
            return tok.value
        raise ParseError(
            f"Erwartet Member-Name in ENUM, gefunden {tok.type.name}",
            tok.line, tok.col,
        )

    def _break(self):
        self._expect(TokenType.BREAK)
        self._consume_terminator()
        return Break()

    def _continue(self):
        self._expect(TokenType.CONTINUE)
        self._consume_terminator()
        return Continue()

    def _with_stmt(self):
        """WITH expr / body / END WITH

        Innerhalb des Body wird `.member` (am Statement-Anfang oder als
        Primary-Expression) zu `<with-var>.member` aufgeloest. Das WITH-Ziel
        wird einmal evaluiert und in einer impliziten lokalen Variable
        gespeichert.
        """
        with_tok = self._expect(TokenType.WITH)
        target = self._expression()
        self._consume_terminator()
        var_name = f"__with_{self._with_counter}"
        self._with_counter += 1
        self._with_stack.append(var_name)
        body: list = []
        while not self._check(TokenType.END):
            if self._at_end():
                raise ParseError("END WITH erwartet, Programmende erreicht",
                                 with_tok.line, with_tok.col)
            body.append(self._statement())
        self._with_stack.pop()
        self._expect(TokenType.END)
        self._expect(TokenType.WITH, "Erwartet WITH nach END")
        self._consume_terminator()
        return With(var_name, target, body)

    def _dot_assign_in_with(self, consume_term: bool = True):
        """`.member = expr` oder `.member.sub = expr` oder `.arr[i] = expr`
        innerhalb eines WITH-Blocks. De-sugared zu MemberAssign/IndexAssign
        auf dem aktuellen WITH-Ziel.

        `consume_term=False` fuer den Single-Line-IF-Pfad: dort darf der
        Terminator nicht geschluckt werden, sonst verschwindet ein
        folgendes ELSE (`IF c THEN .x = 1 ELSE .y = 2`).
        """
        # Wir bauen einen Lvalue beginnend mit `Identifier(__with_n)` und
        # erlauben dann denselben Postfix-Pfad wie _assign_from_lvalue.
        start = self.pos
        target_expr: object = self._lvalue_chain(Identifier(self._with_stack[-1]))
        # Erstes Token MUSS DOT sein (vom Caller geprueft).
        op_tok = self._peek()
        if op_tok.type not in _ASSIGN_OPS:
            # KEINE Zuweisung -- dann ist es ein Ausdruck-Statement, allen
            # voran der Methodenaufruf `.update()`. Der lief hier frueher in
            # einen harten "Erwartet '=' ..."-Fehler, weil die Lvalue-Kette
            # `(` gar nicht kennt (ein Aufruf ist kein Zuweisungsziel).
            # Ergebnis: der Editor strich voellig gueltigen Code rot an --
            # und weil der Fallback-Check nur das ERSTE Problem liefert,
            # verdeckte dieser Fehlalarm alle echten Fehler der Datei.
            # gbrt fuehrt dieselbe Zeile korrekt aus; der Rewind hier
            # spiegelt `dot_assign_in_with` in parser.rs.
            self.pos = start
            expr = self._expression()
            if consume_term:
                self._consume_terminator()
            return ExprStmt(expr)
        self.pos += 1
        compound_op = _ASSIGN_OPS[op_tok.type]
        rhs = self._expression()
        if compound_op is not None:
            rhs = BinaryOp(compound_op, target_expr, rhs)
        if consume_term:
            self._consume_terminator()
        if isinstance(target_expr, MemberAccess):
            return MemberAssign(target_expr.target, target_expr.name, rhs)
        if isinstance(target_expr, IndexAccess):
            return IndexAssign(target_expr.target, target_expr.indices, rhs)
        # Reines `__with_n` ohne `.member` -- macht keinen Sinn als Statement.
        raise ParseError(
            "Leeres `.` in WITH-Block (erwartet `.member`)",
            op_tok.line, op_tok.col,
        )

    def _try_stmt(self):
        try_tok = self._expect(TokenType.TRY)
        self._consume_terminator()
        body: list = []
        while not self._check(TokenType.CATCH) and not self._check(TokenType.END):
            if self._at_end():
                raise ParseError("CATCH oder END TRY erwartet",
                                 try_tok.line, try_tok.col)
            body.append(self._statement())
        catch_var = ""
        catch_block: list = []
        if self._match(TokenType.CATCH):
            # Optional: Name fuer die Exception-Variable
            if self._check(TokenType.IDENT):
                catch_var = self._peek().value
                self.pos += 1
            self._consume_terminator()
            while not self._check(TokenType.END):
                if self._at_end():
                    raise ParseError("END TRY erwartet",
                                     try_tok.line, try_tok.col)
                catch_block.append(self._statement())
        self._expect(TokenType.END)
        self._expect(TokenType.TRY, "Erwartet TRY nach END")
        self._consume_terminator()
        return Try(body, catch_var, catch_block)

    def _throw_stmt(self):
        self._expect(TokenType.THROW)
        value = self._expression()
        self._consume_terminator()
        return Throw(value)

    def _parse_type(self) -> str:
        """Parst Primitivtyp, ARRAY OF <type>, MAP OF [STRING TO] <type>,
        oder Klassennamen (IDENT)."""
        type_tok = self._peek()
        if type_tok.type == TokenType.ARRAY:
            self.pos += 1
            self._expect(TokenType.OF, "Erwartet OF nach ARRAY")
            elem = self._parse_type()
            return f"array:{elem}"
        if type_tok.type == TokenType.MAP:
            self.pos += 1
            self._expect(TokenType.OF, "Erwartet OF nach MAP")
            first = self._parse_type()
            if self._match(TokenType.TO):
                # MAP OF KEY TO VALUE - nur STRING-Keys erlaubt (MAP-Konvention)
                if first != "string":
                    raise ParseError(
                        "MAP-Schluessel muessen STRING sein",
                        type_tok.line, type_tok.col,
                    )
                value_type = self._parse_type()
                return f"map:{value_type}"
            return f"map:{first}"
        if type_tok.type in _TYPE_TOKENS:
            self.pos += 1
            return _TYPE_TOKENS[type_tok.type]
        if type_tok.type == TokenType.IDENT:
            self.pos += 1
            # Enum-Typ: vom Parser zu INTEGER aufgeloest, damit Interpreter
            # und VM keinen besonderen Typ-Mechanismus brauchen. Member-Werte
            # sind sowieso Integer.
            type_name = str(type_tok.value)
            if type_name.lower() in self._enum_names:
                return "integer"
            return type_name
        raise ParseError(
            "Erwartet Typ (INTEGER, FLOAT, STRING, BOOLEAN, IMAGE, SOUND, "
            "FILE, ARRAY OF ..., MAP OF ..., oder Klassenname)",
            type_tok.line, type_tok.col,
        )

    def _assign(self):
        stmt = self._assign_from_lvalue()
        self._consume_terminator()
        return stmt

    def _assign_from_lvalue(self):
        """Parst IDENT (.IDENT | [expr])* (= | += | -= | *= | /=) expr.

        Compound-Operatoren werden im Parser zu `target = target OP value`
        de-sugared. AST und Interpreter/VM kennen nur das einfache Assign.
        """
        name_tok = self._expect(TokenType.IDENT)
        target_expr: object = self._lvalue_chain(Identifier(name_tok.value))
        # Zuweisungs-Operator (= oder Compound-Form)
        op_tok = self._peek()
        if op_tok.type not in _ASSIGN_OPS:
            raise ParseError(
                f"Erwartet '=' oder Compound-Operator, gefunden {op_tok.type.name}",
                op_tok.line, op_tok.col,
            )
        self.pos += 1
        compound_op = _ASSIGN_OPS[op_tok.type]
        rhs = self._expression()
        if compound_op is not None:
            # Desugar: target OP= rhs  ->  target = target OP rhs
            # Achtung: target wird zweimal evaluiert (read + write). Das ist
            # die uebliche BASIC-Semantik; bei Index-Zielen mit Side-Effects
            # in den Indices muss der User aufpassen.
            rhs = BinaryOp(compound_op, target_expr, rhs)
        if isinstance(target_expr, Identifier):
            return Assign(target_expr.name, rhs)
        if isinstance(target_expr, MemberAccess):
            return MemberAssign(target_expr.target, target_expr.name, rhs)
        if isinstance(target_expr, IndexAccess):
            return IndexAssign(target_expr.target, target_expr.indices, rhs)
        raise ParseError("Ungueltige Zuweisungs-Linksseite", 0, 0)

    def _print(self):
        self._expect(TokenType.PRINT)
        # COLON zaehlt wie NEWLINE als Terminator: `PRINT : PRINT "x"` ist
        # die klassische Leerzeilen-Redewendung. Ohne COLON hier lief das in
        # "Unerwartetes Token COLON" -- ein Fehlalarm auf Code, den gbrt
        # ausfuehrt (per --check verifiziert).
        if self._check(TokenType.NEWLINE, TokenType.COLON) or self._at_end():
            self._consume_terminator()
            return Print([])
        items, seps, newline = self._print_items()
        self._consume_terminator()
        return Print(items, seps, newline)

    def _print_items(self):
        """PRINT-Liste: Ausdruecke getrennt durch ',' (Leerzeichen) oder ';'
        (kein Leerzeichen). Ein trailing ',' / ';' unterdrueckt den Zeilenumbruch.
        Liefert (items, seps, newline)."""
        items = [self._expression()]
        seps = []
        newline = True
        while self._check(TokenType.COMMA, TokenType.SEMICOLON):
            sep = "," if self._check(TokenType.COMMA) else ";"
            self._match(TokenType.COMMA, TokenType.SEMICOLON)
            if self._check(TokenType.NEWLINE, TokenType.COLON, TokenType.ELSE) or self._at_end():
                newline = False           # trailing Trenner -> kein Newline
                break
            seps.append(sep)
            items.append(self._expression())
        return items, seps, newline

    def _input(self):
        self._expect(TokenType.INPUT)
        prompt = None
        if self._check(TokenType.STRING):
            prompt = StringLit(self._peek().value)
            self.pos += 1
            self._expect(TokenType.COMMA, "Nach Prompt-String erwartet ',' und Variable")
        name_tok = self._expect(TokenType.IDENT, "Erwartet Variable nach INPUT")
        self._consume_terminator()
        return Input(prompt, name_tok.value)

    def _if(self):
        if_tok = self._expect(TokenType.IF)
        cond = self._expression()
        self._expect(TokenType.THEN, "Erwartet THEN nach Bedingung")

        # Single-Line: IF cond THEN stmt [: stmt ...] [ELSE stmt [: stmt ...]] NEWLINE
        # Alle ':'-getrennten Statements nach THEN gehoeren zum THEN-Zweig
        # (klassisches BASIC, konsistent mit dem Block-IF), der ELSE-Zweig analog.
        if not self._check(TokenType.NEWLINE):
            then_block = [self._inline_statement()]
            while self._match(TokenType.COLON):
                if self._check(TokenType.NEWLINE, TokenType.ELSE) or self._at_end():
                    break
                then_block.append(self._inline_statement())
            else_stmts = []
            if self._match(TokenType.ELSE):
                else_stmts.append(self._inline_statement())
                while self._match(TokenType.COLON):
                    if self._check(TokenType.NEWLINE) or self._at_end():
                        break
                    else_stmts.append(self._inline_statement())
            self._consume_terminator()
            return If(cond, then_block, [], else_stmts)

        # Block: IF cond THEN \n ... [ELSEIF cond THEN \n ...] [ELSE \n ...] END IF
        self._consume_terminator()
        # `_at_end()`-Wache wie bei WHILE/FOR/TRY/SELECT: ohne sie meldet ein
        # nicht geschlossenes IF "Unerwartetes Token EOF" auf der LETZTEN
        # Zeile der Datei statt "END IF erwartet" an der oeffnenden Zeile --
        # beim Tippen der Normalfall und dann maximal irrefuehrend.
        then_block = []
        while not self._check(TokenType.ELSEIF, TokenType.ELSE, TokenType.END):
            if self._at_end():
                raise ParseError("END IF erwartet", if_tok.line, if_tok.col)
            then_block.append(self._statement())

        elseif_branches = []
        while self._match(TokenType.ELSEIF):
            ec = self._expression()
            self._expect(TokenType.THEN)
            self._consume_terminator()
            block = []
            while not self._check(TokenType.ELSEIF, TokenType.ELSE, TokenType.END):
                if self._at_end():
                    raise ParseError("END IF erwartet", if_tok.line, if_tok.col)
                block.append(self._statement())
            elseif_branches.append((ec, block))

        else_block = []
        if self._match(TokenType.ELSE):
            self._consume_terminator()
            while not self._check(TokenType.END):
                if self._at_end():
                    raise ParseError("END IF erwartet", if_tok.line, if_tok.col)
                else_block.append(self._statement())

        self._expect(TokenType.END, "Erwartet END IF")
        self._expect(TokenType.IF, "Erwartet IF nach END")
        self._consume_terminator()
        return If(cond, then_block, elseif_branches, else_block)

    def _select(self):
        """SELECT CASE <expr>
              CASE val1
                  ...
              CASE val1, val2, ..., valN
                  ...
              CASE lo TO hi
                  ...
              CASE ELSE
                  ...
           END SELECT
        """
        sel_tok = self._expect(TokenType.SELECT)
        self._expect(TokenType.CASE, "Erwartet CASE nach SELECT")
        subject = self._expression()
        self._consume_terminator()

        cases: list = []
        else_block: list = []
        saw_else = False

        while not self._check(TokenType.END):
            if self._at_end():
                raise ParseError("END SELECT erwartet",
                                 sel_tok.line, sel_tok.col)
            self._expect(TokenType.CASE, "Erwartet CASE oder END SELECT")
            if self._match(TokenType.ELSE):
                if saw_else:
                    raise ParseError("Mehr als ein CASE ELSE im SELECT-Block",
                                     sel_tok.line, sel_tok.col)
                saw_else = True
                self._consume_terminator()
                while not self._check(TokenType.END):
                    if self._check(TokenType.CASE):
                        raise ParseError(
                            "CASE nach CASE ELSE - ELSE muss letzter Fall sein",
                            sel_tok.line, sel_tok.col,
                        )
                    if self._at_end():
                        raise ParseError("END SELECT erwartet",
                                         sel_tok.line, sel_tok.col)
                    else_block.append(self._statement())
                break  # Nach ELSE kommt nur noch END SELECT
            # Normaler CASE: eine oder mehrere Match-Specs (durch Komma getrennt).
            matches: list = []
            matches.append(self._case_match())
            while self._match(TokenType.COMMA):
                matches.append(self._case_match())
            # Optionaler Guard: `CASE 1, 2 WHERE expr`
            guard = None
            if self._match(TokenType.WHERE):
                guard = self._expression()
            self._consume_terminator()
            block: list = []
            while not self._check(TokenType.CASE, TokenType.END):
                if self._at_end():
                    raise ParseError("END SELECT erwartet",
                                     sel_tok.line, sel_tok.col)
                block.append(self._statement())
            cases.append((matches, guard, block))

        self._expect(TokenType.END, "Erwartet END SELECT")
        self._expect(TokenType.SELECT, "Erwartet SELECT nach END")
        self._consume_terminator()
        return Select(subject, cases, else_block)

    def _case_match(self) -> "CaseMatch":
        """Parst einen einzelnen Match:
            <expr>                   - exakter Wert
            <expr> TO <expr>         - Bereich (inklusiv)
            IS <op> <expr>           - Vergleich (=, <>, <, >, <=, >=)
        """
        if self._match(TokenType.IS):
            op_tok = self._peek()
            op_map = {
                TokenType.EQ: "=", TokenType.NEQ: "<>",
                TokenType.LT: "<", TokenType.GT: ">",
                TokenType.LEQ: "<=", TokenType.GEQ: ">=",
            }
            if op_tok.type not in op_map:
                raise ParseError(
                    "CASE IS: Erwartet Vergleichsoperator (=, <>, <, >, <=, >=)",
                    op_tok.line, op_tok.col,
                )
            self.pos += 1
            expr = self._expression()
            return CaseMatch("is", [op_map[op_tok.type], expr])
        first = self._expression()
        if self._match(TokenType.TO):
            second = self._expression()
            return CaseMatch("range", [first, second])
        return CaseMatch("value", [first])

    def _inline_statement(self):
        """Statement ohne Terminator-Konsum (fuer Single-Line-IF)."""
        tok = self._peek()
        t = tok.type
        if t == TokenType.PRINT:
            self.pos += 1
            if (self._check(TokenType.NEWLINE, TokenType.ELSE, TokenType.COLON)
                    or self._at_end()):
                return Print([])
            items, seps, newline = self._print_items()
            return Print(items, seps, newline)
        if t == TokenType.IDENT and self._is_assignment_lookahead():
            return self._assign_from_lvalue()
        # `IF c THEN .x = 1` (im WITH-Block) und `IF c THEN (a, b) = f()`
        # liefen hier still ueber den ExprStmt-Fallback: `=` wurde als
        # VERGLEICH geparst, die Zuweisung verschwand spurlos aus dem AST --
        # ohne jede Fehlermeldung.
        #
        # gbrt unterstuetzt beide Formen NICHT (verifiziert per `--check`:
        # "Erwartet Zeilenende"). Sie hier zu akzeptieren waere also die
        # falsche Richtung -- der Editor bliebe stumm bei Code, der zur
        # Laufzeit scheitert. Stattdessen dieselbe Grenze klar benennen,
        # statt sie stillschweigend zu verschlucken.
        if t == TokenType.DOT and self._with_stack:
            raise ParseError(
                "Zuweisung an `.member` ist im einzeiligen IF nicht "
                "unterstuetzt -- nutze die mehrzeilige IF/END IF-Form.",
                tok.line, tok.col,
            )
        if t == TokenType.LPAREN and self._is_tuple_assign_lookahead():
            raise ParseError(
                "Tupel-Zuweisung ist im einzeiligen IF nicht unterstuetzt "
                "-- nutze die mehrzeilige IF/END IF-Form.",
                tok.line, tok.col,
            )
        if t == TokenType.RETURN:
            self.pos += 1
            value = None
            if not (self._check(TokenType.NEWLINE, TokenType.ELSE, TokenType.COLON)
                    or self._at_end()):
                value = self._expression()
            return Return(value)
        if t == TokenType.BREAK:
            self.pos += 1
            return Break()
        if t == TokenType.CONTINUE:
            self.pos += 1
            return Continue()
        if t == TokenType.THROW:
            self.pos += 1
            value = self._expression()
            return Throw(value)
        # Fallback
        expr = self._expression()
        # Sicherheitsnetz: ein Top-Level `=` ist fast immer eine gemeinte
        # ZUWEISUNG, deren Ziel der Lookahead nicht erkannt hat -- der
        # Ausdruck wuerde sonst als Vergleich geparst, sein Ergebnis
        # verworfen und die Zuweisung verschwaende spurlos. Genau diese
        # Stille liess die Keyword-Member-Faelle (`spr.image = 5`) so lange
        # unentdeckt. gbrt hat denselben Wortlaut (parser.rs:171/783).
        if isinstance(expr, BinaryOp) and expr.op == "=":
            raise ParseError(
                "'=' als Anweisung -- meintest du eine Zuweisung?",
                tok.line, tok.col,
            )
        return ExprStmt(expr)

    def _while(self):
        self._expect(TokenType.WHILE)
        cond = self._expression()
        self._consume_terminator()
        body = []
        while not self._check(TokenType.WEND):
            if self._at_end():
                raise ParseError("WEND erwartet, Programmende erreicht",
                                 self._peek().line, self._peek().col)
            body.append(self._statement())
        self._expect(TokenType.WEND)
        self._consume_terminator()
        return While(cond, body)

    def _repeat(self):
        """REPEAT body UNTIL <expr>.  Post-Test-Schleife."""
        self._expect(TokenType.REPEAT)
        self._consume_terminator()
        body = []
        while not self._check(TokenType.UNTIL):
            if self._at_end():
                raise ParseError("UNTIL erwartet, Programmende erreicht",
                                 self._peek().line, self._peek().col)
            body.append(self._statement())
        self._expect(TokenType.UNTIL)
        cond = self._expression()
        self._consume_terminator()
        return Repeat(body, cond)

    def _data(self):
        """DATA literal, literal, ..."""
        self._expect(TokenType.DATA)
        values = [self._data_literal()]
        while self._match(TokenType.COMMA):
            values.append(self._data_literal())
        self._consume_terminator()
        return Data(values)

    def _data_literal(self):
        """Erlaubte DATA-Werte: NumberLit, StringLit, BoolLit, -NumberLit."""
        tok = self._peek()
        if tok.type == TokenType.NUMBER:
            self.pos += 1
            return NumberLit(tok.value)
        if tok.type == TokenType.STRING:
            self.pos += 1
            return StringLit(tok.value)
        if tok.type == TokenType.TRUE:
            self.pos += 1
            return BoolLit(True)
        if tok.type == TokenType.FALSE:
            self.pos += 1
            return BoolLit(False)
        if tok.type == TokenType.MINUS:
            self.pos += 1
            inner = self._peek()
            if inner.type != TokenType.NUMBER:
                raise ParseError(
                    "DATA: nach Minus erwartet eine Zahl",
                    inner.line, inner.col,
                )
            self.pos += 1
            return UnaryOp("-", NumberLit(inner.value))
        raise ParseError(
            "DATA: Erwartet Literal (Zahl, String, TRUE, FALSE) - Ausdruecke "
            "sind nicht erlaubt",
            tok.line, tok.col,
        )

    def _read(self):
        """READ target, target, ...  (Targets sind Assignment-Ziele.)"""
        self._expect(TokenType.READ)
        targets = [self._read_target()]
        while self._match(TokenType.COMMA):
            targets.append(self._read_target())
        self._consume_terminator()
        return Read(targets)

    def _read_target(self):
        """Holt ein Assignment-Ziel: IDENT, IDENT[idx], IDENT.member, etc.

        Wir parsen es als _postfix-Expr und akzeptieren alle Formen die
        auch normale Zuweisungen akzeptieren wuerden.
        """
        tok = self._peek()
        if tok.type != TokenType.IDENT:
            raise ParseError(
                "READ: Erwartet Variable als Ziel",
                tok.line, tok.col,
            )
        return self._postfix()

    def _restore(self):
        self._expect(TokenType.RESTORE)
        self._consume_terminator()
        return Restore()

    def _for(self):
        self._expect(TokenType.FOR)
        # FOR EACH var IN expr ... NEXT  ("each" ist kontextuell, kein Keyword:
        # `FOR each = ...` mit Variable "each" bleibt ein regulaerer FOR, weil
        # dort das naechste Token '=' und nicht ein IDENT ist).
        if (self._check(TokenType.IDENT) and self._peek().value == "each"
                and self._peek(1).type == TokenType.IDENT):
            self.pos += 1   # 'each'
            var = self._expect(TokenType.IDENT,
                               "Erwartet Iterationsvariable nach FOR EACH").value
            self._expect(TokenType.IN, "Erwartet IN nach FOR EACH <var>")
            iterable = self._expression()
            self._consume_terminator()
            body = []
            while not self._check(TokenType.NEXT):
                if self._at_end():
                    raise ParseError("NEXT erwartet, Programmende erreicht",
                                     self._peek().line, self._peek().col)
                body.append(self._statement())
            self._expect(TokenType.NEXT)
            if self._check(TokenType.IDENT):
                self.pos += 1
            self._consume_terminator()
            return ForEach(var, iterable, body)
        var = self._expect(TokenType.IDENT, "Erwartet Schleifenvariable").value
        self._expect(TokenType.EQ)
        start = self._expression()
        self._expect(TokenType.TO, "Erwartet TO")
        end = self._expression()
        step = None
        if self._match(TokenType.STEP):
            step = self._expression()
        self._consume_terminator()
        body = []
        while not self._check(TokenType.NEXT):
            if self._at_end():
                raise ParseError("NEXT erwartet, Programmende erreicht",
                                 self._peek().line, self._peek().col)
            body.append(self._statement())
        self._expect(TokenType.NEXT)
        # Optional NEXT <var>
        if self._check(TokenType.IDENT):
            self.pos += 1
        self._consume_terminator()
        return For(var, start, end, step, body)

    # ---- SUB / FUNCTION / RETURN ------------------------------------
    def _params(self):
        """Parst '(' [param {',' param}] ')'.

        Default-Werte sind erlaubt: `name AS TYPE = expr`. Sobald ein
        Parameter einen Default hat, muessen alle folgenden auch einen
        haben (Python-Regel - vermeidet Mehrdeutigkeit beim Aufruf).

        Variadic (`...args`) ist erlaubt -- muss aber LETZTER Parameter
        sein (sonst koennte der Parser nicht entscheiden, wo der
        Variadic-Bereich endet).
        """
        self._expect(TokenType.LPAREN)
        params = []
        seen_default = False
        seen_variadic = False
        if not self._check(TokenType.RPAREN):
            p = self._param()
            if p.is_variadic:
                seen_variadic = True
            seen_default = p.default is not None
            params.append(p)
            while self._match(TokenType.COMMA):
                if seen_variadic:
                    raise ParseError(
                        "Variadic-Parameter (`...args`) muss der letzte "
                        "Parameter sein -- nichts darf danach kommen",
                        self._peek().line, self._peek().col,
                    )
                p = self._param()
                if seen_default and p.default is None and not p.is_variadic:
                    raise ParseError(
                        f"Parameter '{p.name}' ohne Default folgt auf einen "
                        f"Parameter mit Default - das ist nicht erlaubt",
                        self._peek().line, self._peek().col,
                    )
                if p.default is not None:
                    seen_default = True
                if p.is_variadic:
                    seen_variadic = True
                params.append(p)
        self._expect(TokenType.RPAREN)
        return params

    def _param(self) -> Param:
        # `...name` -> variadic. Sammelt alle restlichen Args als TUPLE.
        # Hat KEINEN expliziten AS-Type (immer TUPLE) und KEINEN Default.
        if self._match(TokenType.ELLIPSIS):
            name_tok = self._expect(TokenType.IDENT, "Erwartet Variadic-Parametername nach '...'")
            return Param(str(name_tok.value), "tuple", default=None, by_ref=False, is_variadic=True)
        # Optionales BYREF vor dem Namen -> Parameter wird per Referenz
        # uebergeben (Caller-Variable wird modifiziert nach SUB/FUNCTION-
        # Return). Nur fuer einfache zuweisbare Argumente erlaubt.
        by_ref = self._match(TokenType.BYREF)
        name_tok = self._expect(TokenType.IDENT, "Erwartet Parametername")
        self._expect(TokenType.AS, "Erwartet AS nach Parametername")
        type_name = self._parse_type()
        default = None
        if self._match(TokenType.EQ):
            if by_ref:
                raise ParseError(
                    "BYREF-Parameter koennen keinen Default-Wert haben",
                    name_tok.line, name_tok.col,
                )
            default = self._expression()
        return Param(str(name_tok.value), type_name, default, by_ref=by_ref)

    def _sub_decl(self):
        sub_tok = self._expect(TokenType.SUB)
        name_tok = self._expect(TokenType.IDENT, "Erwartet SUB-Name")
        params = self._params()
        self._consume_terminator()
        body = []
        while not self._check(TokenType.END):
            if self._at_end():
                raise ParseError("END SUB erwartet, Programmende erreicht",
                                 sub_tok.line, sub_tok.col)
            body.append(self._statement())
        self._expect(TokenType.END)
        self._expect(TokenType.SUB, "Erwartet SUB nach END")
        self._consume_terminator()
        return SubDecl(name_tok.value, params, body)

    def _function_decl(self):
        fn_tok = self._expect(TokenType.FUNCTION)
        name_tok = self._expect(TokenType.IDENT, "Erwartet FUNCTION-Name")
        params = self._params()
        self._expect(TokenType.AS, "Erwartet AS <Rueckgabetyp> nach Parameterliste")
        return_type = self._parse_type()
        self._consume_terminator()
        body = []
        while not self._check(TokenType.END):
            if self._at_end():
                raise ParseError("END FUNCTION erwartet, Programmende erreicht",
                                 fn_tok.line, fn_tok.col)
            body.append(self._statement())
        self._expect(TokenType.END)
        self._expect(TokenType.FUNCTION, "Erwartet FUNCTION nach END")
        self._consume_terminator()
        return FunctionDecl(name_tok.value, params, return_type, body)

    # Mapping: Operator-Token -> interner Methoden-Name. Die internen Namen
    # werden im Class-Methods-Dict gespeichert; der BinaryOp-Dispatch in
    # gbrts VM lookt sie unter diesem Namen auf.
    _OPERATOR_NAMES = {
        TokenType.PLUS:  "__op_add__",
        TokenType.MINUS: "__op_sub__",
        TokenType.STAR:  "__op_mul__",
        TokenType.SLASH: "__op_div__",
        TokenType.MOD:   "__op_mod__",
        TokenType.EQ:    "__op_eq__",
        TokenType.NEQ:   "__op_ne__",
        TokenType.LT:    "__op_lt__",
        TokenType.GT:    "__op_gt__",
        TokenType.LEQ:   "__op_le__",
        TokenType.GEQ:   "__op_ge__",
    }

    def _operator_decl(self):
        """Operator-Overloading: `OPERATOR <op> (other AS T) AS T ... END OPERATOR`.

        Erlaubt nur in Class-Body. Wird intern als Methode mit reserviertem
        Namen registriert (z.B. `__op_add__` fuer `OPERATOR +`). Der
        Der BinaryOp-Dispatch in gbrts VM lookt diese Methoden
        auf, wenn ein Operand eine User-Instanz mit passender Operator-Methode
        ist.

        Genau ein Parameter (`other`). Ein Rueckgabetyp ist verpflichtend --
        der Operator MUSS einen Wert liefern (sonst ist's keine Expression).
        """
        op_tok = self._expect(TokenType.OPERATOR)
        # Naechstes Token ist der Operator selbst.
        op_kind = self._peek().type
        op_name = self._OPERATOR_NAMES.get(op_kind)
        if op_name is None:
            tok = self._peek()
            raise ParseError(
                f"OPERATOR: erwartet einen der Operatoren "
                f"+ - * / MOD = <> < > <= >=, "
                f"gefunden {tok.type.name}",
                tok.line, tok.col,
            )
        self.pos += 1  # Operator-Token konsumieren
        params = self._params()
        if len(params) != 1:
            raise ParseError(
                f"OPERATOR: erwartet genau 1 Parameter (other), "
                f"erhalten {len(params)}",
                op_tok.line, op_tok.col,
            )
        if params[0].by_ref:
            raise ParseError(
                "OPERATOR: Parameter darf nicht BYREF sein",
                op_tok.line, op_tok.col,
            )
        if params[0].is_variadic:
            raise ParseError(
                "OPERATOR: Parameter darf nicht variadic sein",
                op_tok.line, op_tok.col,
            )
        self._expect(TokenType.AS,
                     "Erwartet AS <Rueckgabetyp> nach OPERATOR-Parameter")
        return_type = self._parse_type()
        self._consume_terminator()
        body = []
        while not self._check(TokenType.END):
            if self._at_end():
                raise ParseError(
                    "END OPERATOR erwartet, Programmende erreicht",
                    op_tok.line, op_tok.col,
                )
            body.append(self._statement())
        self._expect(TokenType.END)
        self._expect(TokenType.OPERATOR, "Erwartet OPERATOR nach END")
        self._consume_terminator()
        return FunctionDecl(op_name, params, return_type, body)

    def _return(self):
        self._expect(TokenType.RETURN)
        value = None
        # Wie bei `_print`: COLON beendet das Statement (`x = 1 : RETURN : ...`).
        if not self._check(TokenType.NEWLINE, TokenType.COLON) and not self._at_end():
            value = self._expression()
        self._consume_terminator()
        return Return(value)

    # ---- CLASS -------------------------------------------------------
    def _property_decl(self):
        """`PROPERTY GET name() AS T ... END PROPERTY`
           `PROPERTY SET name(value AS T) ... END PROPERTY`

        Liefert ein (PropertyDecl, FunctionDecl|SubDecl)-Tupel: der
        PropertyDecl markiert die Klasse, der zweite Wert ist die interne
        Methoden-Repraesentation mit dem reservierten Namen
        `__get_<name>` bzw. `__set_<name>`.
        """
        prop_tok = self._expect(TokenType.PROPERTY)
        kind_tok = self._peek()
        if kind_tok.type != TokenType.IDENT or kind_tok.value not in ("get", "set"):
            raise ParseError(
                f"Erwartet GET oder SET nach PROPERTY, gefunden "
                f"{kind_tok.type.name}({kind_tok.value!r})",
                kind_tok.line, kind_tok.col,
            )
        kind = kind_tok.value
        self.pos += 1
        name_tok = self._expect(TokenType.IDENT, "Erwartet Property-Name")
        prop_name = name_tok.value
        internal_name = f"__{kind}_{prop_name.lower()}"
        params = self._params()
        # In gbrts Compiler: getter ist FUNCTION, setter ist SUB.
        if kind == "get":
            self._expect(TokenType.AS, "Erwartet AS <Rueckgabetyp> nach PROPERTY GET-Parametern")
            return_type = self._parse_type()
            self._consume_terminator()
            body = []
            while not self._check(TokenType.END):
                if self._at_end():
                    raise ParseError("END PROPERTY erwartet, Programmende erreicht",
                                     prop_tok.line, prop_tok.col)
                body.append(self._statement())
            self._expect(TokenType.END)
            self._expect(TokenType.PROPERTY, "Erwartet PROPERTY nach END")
            self._consume_terminator()
            if len(params) != 0:
                raise ParseError(
                    "PROPERTY GET nimmt keine Parameter",
                    prop_tok.line, prop_tok.col,
                )
            func = FunctionDecl(internal_name, params, return_type, body)
        else:   # "set"
            self._consume_terminator()
            body = []
            while not self._check(TokenType.END):
                if self._at_end():
                    raise ParseError("END PROPERTY erwartet, Programmende erreicht",
                                     prop_tok.line, prop_tok.col)
                body.append(self._statement())
            self._expect(TokenType.END)
            self._expect(TokenType.PROPERTY, "Erwartet PROPERTY nach END")
            self._consume_terminator()
            if len(params) != 1:
                raise ParseError(
                    "PROPERTY SET muss genau einen Parameter haben (den neuen Wert)",
                    prop_tok.line, prop_tok.col,
                )
            func = SubDecl(internal_name, params, body)
        return PropertyDecl(prop_name, kind, func), func

    def _class_decl(self):
        cls_tok = self._expect(TokenType.CLASS)
        name_tok = self._expect(TokenType.IDENT, "Erwartet Klassenname nach CLASS")
        parent = None
        if self._match(TokenType.EXTENDS):
            parent_tok = self._expect(TokenType.IDENT, "Erwartet Elternklassen-Name nach EXTENDS")
            parent = parent_tok.value
        self._consume_terminator()
        fields = []
        methods = []
        statics: list = []
        properties: list = []
        while not self._check(TokenType.END):
            if self._at_end():
                raise ParseError("END CLASS erwartet, Programmende erreicht",
                                 cls_tok.line, cls_tok.col)
            if self._check(TokenType.NEWLINE):
                self.pos += 1
                continue
            if self._check(TokenType.STATIC):
                self.pos += 1
                if not self._check(TokenType.CONST):
                    tok = self._peek()
                    raise ParseError(
                        f"Erwartet CONST nach STATIC im CLASS-Body, "
                        f"gefunden {tok.type.name}",
                        tok.line, tok.col,
                    )
                statics.append(self._const())
            elif self._check(TokenType.PROPERTY):
                pd, internal_method = self._property_decl()
                properties.append(pd)
                methods.append(internal_method)
            elif self._check(TokenType.DIM):
                fields.append(self._dim())
            elif self._check(TokenType.SUB):
                methods.append(self._sub_decl())
            elif self._check(TokenType.FUNCTION):
                methods.append(self._function_decl())
            elif self._check(TokenType.OPERATOR):
                methods.append(self._operator_decl())
            else:
                tok = self._peek()
                raise ParseError(
                    f"Unerwartet im CLASS-Body: {tok.type.name} "
                    f"(erlaubt: DIM, SUB, FUNCTION, OPERATOR, "
                    f"STATIC CONST, PROPERTY)",
                    tok.line, tok.col,
                )
        self._expect(TokenType.END)
        self._expect(TokenType.CLASS, "Erwartet CLASS nach END")
        self._consume_terminator()
        return ClassDecl(name_tok.value, parent, fields, methods,
                         statics=statics, properties=properties)

    def _struct_decl(self):
        struct_tok = self._expect(TokenType.STRUCT)
        name_tok = self._expect(TokenType.IDENT, "Erwartet Name nach STRUCT")
        self._consume_terminator()
        fields = []
        while not self._check(TokenType.END):
            if self._at_end():
                raise ParseError("END STRUCT erwartet, Programmende erreicht",
                                 struct_tok.line, struct_tok.col)
            if self._check(TokenType.NEWLINE):
                self.pos += 1
                continue
            if self._check(TokenType.DIM):
                fields.append(self._dim())
            else:
                tok = self._peek()
                raise ParseError(
                    f"Im STRUCT-Body sind nur DIM-Felder erlaubt (gefunden {tok.type.name})",
                    tok.line, tok.col,
                )
        self._expect(TokenType.END)
        self._expect(TokenType.STRUCT, "Erwartet STRUCT nach END")
        self._consume_terminator()
        return ClassDecl(
            name_tok.value, parent=None, fields=fields, methods=[], is_struct=True
        )

    def _new_expr(self):
        self._expect(TokenType.NEW)
        name_tok = self._expect(TokenType.IDENT, "Erwartet Klassenname nach NEW")
        args = None
        if self._match(TokenType.LPAREN):
            args = self._call_args()
            self._expect(TokenType.RPAREN)
        return New(name_tok.value, args)

    # ---- Ausdruecke --------------------------------------------------
    # Jede Verschachtelungsebene kostet ~12 Stack-Frames durch die feste
    # Praezedenz-Kette. Ohne eigene Grenze knallt tief verschachtelter Code
    # in Pythons RecursionError -- die faengt `_check_syntax_only` zwar ab,
    # meldet dem Nutzer dann aber "maximum recursion depth exceeded" auf
    # Zeile 1 statt zu sagen, WO und WAS das Problem ist.
    #
    # Die Grenze muss deutlich UNTER dem physisch Moeglichen liegen: gemessen
    # sind unter pytest nur ~78 Ebenen drin, bevor Python selbst abbricht --
    # und wie viel Stack uebrig ist, haengt davon ab, wie tief der Aufrufer
    # schon steckt (Editor-Thread, LSP-Handler, Test-Runner sind
    # unterschiedlich tief). 40 laesst rund die Haelfte des Budgets als
    # Reserve und ist fuer echten Code weit jenseits von allem Sinnvollen
    # (typische Ausdruecke bleiben unter 10 Ebenen).
    _MAX_EXPR_DEPTH = 40

    def _expression(self):
        # YIELD ist ein niedrig-praezedenter Ausdruck: `x = YIELD a + b`
        # parst als `x = YIELD (a + b)`. Als Statement (`YIELD v`) faellt es
        # ueber den ExprStmt-Fallback hier herein.
        self._expr_depth += 1
        try:
            if self._expr_depth > self._MAX_EXPR_DEPTH:
                tok = self._peek()
                raise ParseError(
                    f"Ausdruck zu tief verschachtelt (max. "
                    f"{self._MAX_EXPR_DEPTH} Ebenen)", tok.line, tok.col)
            if self._check(TokenType.YIELD):
                return self._yield_expr()
            return self._or_expr()
        finally:
            self._expr_depth -= 1

    def _yield_expr(self):
        self._expect(TokenType.YIELD)
        # Optionaler Operand: kein Wert, wenn ein Terminator/Klammer-Ende folgt.
        if self._at_end() or self._check(
            TokenType.NEWLINE, TokenType.EOF, TokenType.COLON,
            TokenType.RPAREN, TokenType.RBRACKET, TokenType.RBRACE,
            TokenType.COMMA, TokenType.ELSE,
        ):
            return Yield(None)
        return Yield(self._expression())

    def _or_expr(self):
        left = self._and_expr()
        while self._match(TokenType.OR):
            right = self._and_expr()
            left = BinaryOp("or", left, right)
        return left

    def _and_expr(self):
        left = self._not_expr()
        while self._match(TokenType.AND):
            right = self._not_expr()
            left = BinaryOp("and", left, right)
        return left

    def _not_expr(self):
        if self._match(TokenType.NOT):
            return UnaryOp("not", self._not_expr())
        return self._comparison()

    def _comparison(self):
        left = self._bitwise()
        while self._check(TokenType.EQ, TokenType.NEQ, TokenType.LT,
                          TokenType.GT, TokenType.LEQ, TokenType.GEQ,
                          TokenType.IN):
            tok = self._peek()
            self.pos += 1
            op = {
                TokenType.EQ: "=",
                TokenType.NEQ: "<>",
                TokenType.LT: "<",
                TokenType.GT: ">",
                TokenType.LEQ: "<=",
                TokenType.GEQ: ">=",
                TokenType.IN: "in",
            }[tok.type]
            right = self._bitwise()
            left = BinaryOp(op, left, right)
        return left

    def _bitwise(self):
        # Alle Bitwise-Binaer-Operatoren auf einer Ebene, links-assoziativ.
        # Strikt INTEGER -- die Type-Pruefung macht der Interpreter/VM.
        left = self._addition()
        while self._check(TokenType.BAND, TokenType.BOR, TokenType.BXOR,
                          TokenType.SHL, TokenType.SHR):
            tok = self._peek()
            self.pos += 1
            op = {
                TokenType.BAND: "band",
                TokenType.BOR:  "bor",
                TokenType.BXOR: "bxor",
                TokenType.SHL:  "shl",
                TokenType.SHR:  "shr",
            }[tok.type]
            right = self._addition()
            left = BinaryOp(op, left, right)
        return left

    def _addition(self):
        left = self._multiplication()
        while self._check(TokenType.PLUS, TokenType.MINUS):
            op = "+" if self._peek().type == TokenType.PLUS else "-"
            self.pos += 1
            right = self._multiplication()
            left = BinaryOp(op, left, right)
        return left

    def _multiplication(self):
        left = self._unary()
        while self._check(TokenType.STAR, TokenType.SLASH, TokenType.MOD,
                          TokenType.INTDIV):
            tok = self._peek()
            self.pos += 1
            op = {
                TokenType.STAR: "*", TokenType.SLASH: "/",
                TokenType.MOD: "mod", TokenType.INTDIV: "\\",
            }[tok.type]
            right = self._unary()
            left = BinaryOp(op, left, right)
        return left

    def _unary(self):
        if self._match(TokenType.MINUS):
            return UnaryOp("-", self._unary())
        if self._match(TokenType.PLUS):
            return self._unary()
        if self._match(TokenType.BNOT):
            return UnaryOp("bnot", self._unary())
        return self._power()

    def _power(self):
        left = self._postfix()
        if self._match(TokenType.CARET):
            right = self._unary()  # rechts-assoziativ
            return BinaryOp("^", left, right)
        return left

    def _postfix(self):
        expr = self._primary()
        while True:
            if self._match(TokenType.LPAREN):
                # Ein Literal ist nicht aufrufbar. `PRINT 1(2)` parste hier
                # klaglos als Call durch, waehrend gbrt es ablehnt -- der
                # Editor blieb also stumm bei Code, der zur Laufzeit
                # scheitert (per --check verifiziert). Identifier,
                # Member-Zugriffe und Index-Zugriffe bleiben aufrufbar
                # (`f(1)`, `o.m(1)`, `arr[0](1)` mit FUNCREF).
                if isinstance(expr, (NumberLit, StringLit, BoolLit, NilLit)):
                    tok = self._peek()
                    raise ParseError(
                        "Literal ist nicht aufrufbar", tok.line, tok.col)
                args = self._call_args()
                self._expect(TokenType.RPAREN)
                expr = Call(expr, args)
            elif self._match(TokenType.DOT):
                # Member-Namen duerfen Keywords sein (z.B. ENUM mit Member
                # NONE, READ, FILE etc.) - der qualifizierte Zugriff ist
                # eindeutig.
                m_tok = self._peek()
                if isinstance(m_tok.value, str) and m_tok.value:
                    self.pos += 1
                    expr = MemberAccess(expr, m_tok.value)
                else:
                    raise ParseError(
                        f"Erwartet Membername nach '.', gefunden {m_tok.type.name}",
                        m_tok.line, m_tok.col,
                    )
            elif self._match(TokenType.LBRACKET):
                expr = self._index_or_slice(expr)
            else:
                break
        return expr

    def _index_or_slice(self, target):
        """Liest den Inhalt von `[ ... ]` und liefert IndexAccess oder
        SliceAccess. Disambiguierung anhand des Top-Level-Doppelpunkts:

            arr[1, 2]   -> IndexAccess (Multi-Dim)
            arr[i]      -> IndexAccess
            arr[a:b]    -> SliceAccess
            arr[:b]     -> SliceAccess (lo = None -> 0)
            arr[a:]     -> SliceAccess (hi = None -> len)
            arr[:]      -> SliceAccess (volle Kopie)

        Slicing ist nur 1D -- Multi-Dim-Slicing wuerde NumPy-aehnliche
        Semantik brauchen.
        """
        # Spezialfall: direkt nach `[` kommt `:` -> Slice mit lo=None.
        if self._check(TokenType.COLON):
            self.pos += 1
            hi = None
            if not self._check(TokenType.RBRACKET):
                hi = self._expression()
            self._expect(TokenType.RBRACKET, "Erwartet ']'")
            return SliceAccess(target, None, hi)

        first = self._expression()

        # Slice-Form: `[expr : ...]`
        if self._match(TokenType.COLON):
            hi = None
            if not self._check(TokenType.RBRACKET):
                hi = self._expression()
            self._expect(TokenType.RBRACKET, "Erwartet ']'")
            return SliceAccess(target, first, hi)

        # Multi-Dim oder 1D-Index
        indices = [first]
        while self._match(TokenType.COMMA):
            indices.append(self._expression())
        self._expect(TokenType.RBRACKET, "Erwartet ']'")
        return IndexAccess(target, indices)

    def _primary(self):
        tok = self._peek()
        t = tok.type
        # Innerhalb eines WITH-Blocks: `.member` als Expression-Start ist
        # Shortcut fuer `<with-var>.member`. Die Postfix-Schleife uebernimmt
        # weitere `.x[y]`-Ketten.
        if t == TokenType.DOT and self._with_stack:
            self.pos += 1
            m_tok = self._peek()
            if not (isinstance(m_tok.value, str) and m_tok.value):
                raise ParseError(
                    f"Erwartet Membername nach '.' im WITH-Block, "
                    f"gefunden {m_tok.type.name}",
                    m_tok.line, m_tok.col,
                )
            self.pos += 1
            return MemberAccess(Identifier(self._with_stack[-1]), m_tok.value)
        if t == TokenType.NUMBER:
            self.pos += 1
            return NumberLit(tok.value)
        if t == TokenType.STRING:
            self.pos += 1
            return StringLit(tok.value)
        if t == TokenType.TRUE:
            self.pos += 1
            return BoolLit(True)
        if t == TokenType.FALSE:
            self.pos += 1
            return BoolLit(False)
        if t == TokenType.NIL:
            self.pos += 1
            return NilLit()
        if t == TokenType.IDENT:
            # IIF(cond, a, b) -- lazy Ternary (nur EIN Zweig wird ausgewertet).
            # Kontextabhaengig: nur als `iif(` behandelt; `iif` als blosser
            # Bezeichner bleibt eine Variable.
            if tok.value == "iif" and self._peek(1).type == TokenType.LPAREN:
                self.pos += 2   # 'iif' und '(' konsumieren
                cond = self._expression()
                self._expect(TokenType.COMMA, "Erwartet ',' nach IIF-Bedingung")
                then_e = self._expression()
                self._expect(TokenType.COMMA, "Erwartet ',' nach IIF-Then-Wert")
                else_e = self._expression()
                self._expect(TokenType.RPAREN, "Erwartet ')' am Ende von IIF")
                return TernaryExpr(cond, then_e, else_e)
            self.pos += 1
            return Identifier(tok.value)
        if t == TokenType.NEW:
            return self._new_expr()
        if t == TokenType.LBRACKET:
            # `[...]` ist entweder eine List-Comprehension
            # (`[expr FOR var IN iterable [WHERE filter]]`) ODER ein
            # Array-Literal (`[a, b, c]`). Disambiguierung: FOR nach dem
            # ersten Ausdruck -> Comprehension, sonst Array-Literal.
            self.pos += 1
            if self._peek().type == TokenType.RBRACKET:
                raise ParseError(
                    "Leeres Array-Literal [] -- Typ unbekannt; "
                    "nutze DIM ... AS ARRAY OF T",
                    self._peek().line, self._peek().col,
                )
            first = self._expression()
            if self._match(TokenType.FOR):
                var_tok = self._expect(TokenType.IDENT, "Erwartet Iter-Variablenname nach FOR")
                self._expect(TokenType.IN, "Erwartet IN nach Iter-Variable")
                iterable = self._expression()
                filter_expr = None
                if self._match(TokenType.WHERE):
                    filter_expr = self._expression()
                self._expect(TokenType.RBRACKET, "Erwartet ']' am Ende der Comprehension")
                return ListComp(var_tok.value, iterable, filter_expr, first)
            elements = [first]
            while self._match(TokenType.COMMA):
                if self._peek().type == TokenType.RBRACKET:
                    break   # optionales Trailing-Komma
                elements.append(self._expression())
            self._expect(TokenType.RBRACKET, "Erwartet ']' am Ende des Array-Literals")
            return ArrayLit(elements)
        if t == TokenType.LBRACE:
            # Dict- oder Set-Comprehension:
            #   `{key: val FOR var IN iterable [WHERE filter]}`  -> DictComp
            #   `{expr FOR var IN iterable [WHERE filter]}`      -> SetComp
            self.pos += 1
            first = self._expression()
            # Disambiguate via `:` (dann Dict) oder direkt `FOR` (dann Set).
            if self._match(TokenType.COLON):
                value = self._expression()
                if not self._match(TokenType.FOR):
                    raise ParseError(
                        "Erwartet FOR in Dict-Comprehension "
                        "`{key: val FOR var IN ...}`",
                        self._peek().line, self._peek().col,
                    )
                var_tok = self._expect(
                    TokenType.IDENT, "Erwartet Iter-Variablenname nach FOR")
                self._expect(TokenType.IN, "Erwartet IN nach Iter-Variable")
                iterable = self._expression()
                filter_expr = None
                if self._match(TokenType.WHERE):
                    filter_expr = self._expression()
                self._expect(TokenType.RBRACE,
                             "Erwartet '}' am Ende der Dict-Comprehension")
                return DictComp(var_tok.value, iterable, filter_expr,
                                first, value)
            if not self._match(TokenType.FOR):
                raise ParseError(
                    "Erwartet ':' (Dict-Comp) oder FOR (Set-Comp) im "
                    "`{...}`-Block",
                    self._peek().line, self._peek().col,
                )
            var_tok = self._expect(
                TokenType.IDENT, "Erwartet Iter-Variablenname nach FOR")
            self._expect(TokenType.IN, "Erwartet IN nach Iter-Variable")
            iterable = self._expression()
            filter_expr = None
            if self._match(TokenType.WHERE):
                filter_expr = self._expression()
            self._expect(TokenType.RBRACE,
                         "Erwartet '}' am Ende der Set-Comprehension")
            return SetComp(var_tok.value, iterable, filter_expr, first)
        if t == TokenType.LPAREN:
            self.pos += 1
            e = self._expression()
            # Tupel-Literal: nach erstem Element kommt ein Komma -> sammle
            # weitere Elemente. Single-Element bleibt normale Klammer-
            # Gruppierung. `(1,)`-Single-Tupel-Syntax wie in Python wird NICHT
            # unterstuetzt (kein Use-Case in GameBasic, einfacher Parser).
            if self._check(TokenType.COMMA):
                elements = [e]
                while self._match(TokenType.COMMA):
                    elements.append(self._expression())
                self._expect(TokenType.RPAREN)
                return TupleLit(elements)
            self._expect(TokenType.RPAREN)
            return e
        raise ParseError(
            f"Unerwartetes Token {t.name}",
            tok.line, tok.col,
        )
