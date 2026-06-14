//! Lexer/Tokenizer fuer GameBasic -- Rust-Port von `gamebasic/lexer.py` +
//! `tokens.py`. Erster Schritt der Front-End-Portierung (Lexer -> Parser ->
//! Compiler), damit `gbrt` perspektivisch ohne Python aus Quelltext Bytecode
//! erzeugt.
//!
//! Verifiziert gegen den Python-Lexer ueber `gbrt --tokens` (kanonischer
//! JSON-Dump `[TYP, wert, zeile]` pro Token) vs. Python -- siehe
//! `tests/test_rust_lexer_parity.py`. Case-insensitive (Keywords + Idents
//! lowercase), `'`/`REM` Kommentare, signifikante Newlines, implizite
//! Zeilenfortsetzung in Klammern, f-Strings, `&H`/`&B`-Literale.

use serde_json::Value as J;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tt {
    Number, Str, Ident,
    Dim, As, Integer, Float, StringType, Boolean, Print, Input, If, Then, Else,
    Elseif, End, While, Wend, For, To, Step, Next, Sub, Function, Return, Class,
    New, Extends, True, False, Nil, And, Or, Not, Mod, Const, Break, Continue, Image,
    Sound, Array, Of, Struct, File, Map, Try, Catch, Throw, Import, Select, Case,
    Is, Repeat, Until, Data, Read, Restore, Byref, Enum, Band, Bor, Bxor, Bnot,
    Shl, Shr, Tuple, With, Static, Funcref, In, Ellipsis, Where, Property,
    Operator, Yield, Coroutine,
    Plus, Minus, StarT, Slash, Intdiv, Caret, Eq, Neq, Lt, Gt, Leq, Geq,
    Lparen, Rparen, Lbracket, Rbracket, Lbrace, Rbrace, Comma, Dot, Semicolon,
    Colon, PlusEq, MinusEq, StarEq, SlashEq, Newline, Eof,
}

impl Tt {
    /// Exakt die Namen aus `tokens.py` (`TokenType.<NAME>.name`).
    pub fn name(&self) -> &'static str {
        use Tt::*;
        match self {
            Number => "NUMBER", Str => "STRING", Ident => "IDENT",
            Dim => "DIM", As => "AS", Integer => "INTEGER", Float => "FLOAT",
            StringType => "STRING_TYPE", Boolean => "BOOLEAN", Print => "PRINT",
            Input => "INPUT", If => "IF", Then => "THEN", Else => "ELSE",
            Elseif => "ELSEIF", End => "END", While => "WHILE", Wend => "WEND",
            For => "FOR", To => "TO", Step => "STEP", Next => "NEXT", Sub => "SUB",
            Function => "FUNCTION", Return => "RETURN", Class => "CLASS",
            New => "NEW", Extends => "EXTENDS", True => "TRUE", False => "FALSE",
            Nil => "NIL",
            And => "AND", Or => "OR", Not => "NOT", Mod => "MOD", Const => "CONST",
            Break => "BREAK", Continue => "CONTINUE", Image => "IMAGE",
            Sound => "SOUND", Array => "ARRAY", Of => "OF", Struct => "STRUCT",
            File => "FILE", Map => "MAP", Try => "TRY", Catch => "CATCH",
            Throw => "THROW", Import => "IMPORT", Select => "SELECT",
            Case => "CASE", Is => "IS", Repeat => "REPEAT", Until => "UNTIL",
            Data => "DATA", Read => "READ", Restore => "RESTORE", Byref => "BYREF",
            Enum => "ENUM", Band => "BAND", Bor => "BOR", Bxor => "BXOR",
            Bnot => "BNOT", Shl => "SHL", Shr => "SHR", Tuple => "TUPLE",
            With => "WITH", Static => "STATIC", Funcref => "FUNCREF", In => "IN",
            Ellipsis => "ELLIPSIS", Where => "WHERE", Property => "PROPERTY",
            Operator => "OPERATOR", Yield => "YIELD", Coroutine => "COROUTINE",
            Plus => "PLUS", Minus => "MINUS", StarT => "STAR", Slash => "SLASH",
            Intdiv => "INTDIV", Caret => "CARET", Eq => "EQ", Neq => "NEQ",
            Lt => "LT", Gt => "GT", Leq => "LEQ", Geq => "GEQ", Lparen => "LPAREN",
            Rparen => "RPAREN", Lbracket => "LBRACKET", Rbracket => "RBRACKET",
            Lbrace => "LBRACE", Rbrace => "RBRACE", Comma => "COMMA", Dot => "DOT",
            Semicolon => "SEMICOLON", Colon => "COLON", PlusEq => "PLUS_EQ",
            MinusEq => "MINUS_EQ", StarEq => "STAR_EQ", SlashEq => "SLASH_EQ",
            Newline => "NEWLINE", Eof => "EOF",
        }
    }
}

/// Keyword-Lookup (lowercase) -> Tt. Spiegelt `KEYWORDS` aus tokens.py.
fn keyword(text: &str) -> Option<Tt> {
    use Tt::*;
    Some(match text {
        "dim" => Dim, "as" => As, "integer" => Integer, "float" => Float,
        "string" => StringType, "boolean" => Boolean, "print" => Print,
        "input" => Input, "if" => If, "then" => Then, "else" => Else,
        "elseif" => Elseif, "elif" => Elseif, "end" => End, "while" => While,
        "wend" => Wend, "for" => For, "to" => To, "step" => Step, "next" => Next,
        "sub" => Sub, "function" => Function, "return" => Return, "class" => Class,
        "new" => New, "extends" => Extends, "true" => True, "false" => False,
        "nil" => Nil,
        "and" => And, "or" => Or, "not" => Not, "mod" => Mod, "const" => Const,
        "break" => Break, "continue" => Continue, "image" => Image,
        "sound" => Sound, "array" => Array, "of" => Of, "struct" => Struct,
        "file" => File, "map" => Map, "try" => Try, "catch" => Catch,
        "throw" => Throw, "import" => Import, "select" => Select, "case" => Case,
        "is" => Is, "repeat" => Repeat, "until" => Until, "data" => Data,
        "read" => Read, "restore" => Restore, "byref" => Byref, "enum" => Enum,
        "band" => Band, "bor" => Bor, "bxor" => Bxor, "bnot" => Bnot,
        "shl" => Shl, "shr" => Shr, "tuple" => Tuple, "with" => With,
        "static" => Static, "funcref" => Funcref, "in" => In, "where" => Where,
        "property" => Property, "operator" => Operator, "yield" => Yield,
        "coroutine" => Coroutine,
        _ => return None,
    })
}

/// Token-Wert -- entspricht `Token.value` in Python.
#[derive(Clone, Debug)]
pub enum Val {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tt: Tt,
    pub val: Val,
    pub line: usize,
    pub col: usize,
}

pub struct LexError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    src: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    toks: Vec<Token>,
    paren_depth: i32,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer { src: source.chars().collect(), pos: 0, line: 1, col: 1,
                toks: Vec::new(), paren_depth: 0 }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        while self.pos < self.src.len() {
            self.scan_token()?;
        }
        if self.toks.last().map(|t| t.tt) != Some(Tt::Newline)
            && !self.toks.is_empty()
        {
            self.push(Tt::Newline, Val::Nil, self.line, self.col);
        }
        self.push(Tt::Eof, Val::Nil, self.line, self.col);
        Ok(self.toks)
    }

    fn peek(&self, off: usize) -> char {
        self.src.get(self.pos + off).copied().unwrap_or('\0')
    }

    fn advance(&mut self) -> char {
        let ch = self.src[self.pos];
        self.pos += 1;
        if ch == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        ch
    }

    fn push(&mut self, tt: Tt, val: Val, line: usize, col: usize) {
        self.toks.push(Token { tt, val, line, col });
    }

    fn err(&self, msg: &str, line: usize, col: usize) -> LexError {
        LexError { msg: msg.to_string(), line, col }
    }

    fn scan_token(&mut self) -> Result<(), LexError> {
        let ch = self.peek(0);
        let (line, col) = (self.line, self.col);

        if ch == ' ' || ch == '\t' || ch == '\r' { self.advance(); return Ok(()); }

        if ch == '\n' {
            self.advance();
            if self.paren_depth > 0 { return Ok(()); }
            if self.toks.last().map(|t| t.tt) != Some(Tt::Newline) {
                self.push(Tt::Newline, Val::Nil, line, col);
            }
            return Ok(());
        }

        // Zeilenfortsetzung: `_` direkt vor Newline.
        if ch == '_' && (self.peek(1) == '\n' || self.peek(1) == '\r') {
            self.advance();
            if self.peek(0) == '\r' { self.advance(); }
            if self.peek(0) == '\n' { self.advance(); }
            return Ok(());
        }

        if ch == '\'' {
            while self.peek(0) != '\0' && self.peek(0) != '\n' { self.advance(); }
            return Ok(());
        }

        if ch == '"' { return self.scan_string(line, col); }
        if ch.is_ascii_digit() { return self.scan_number(line, col); }

        if ch == '&' {
            let nxt = self.peek(1);
            if matches!(nxt, 'H' | 'h' | 'B' | 'b') {
                return self.scan_hex_or_binary(line, col);
            }
            return Err(self.err(
                "Erwartet H oder B nach '&' (Hex-/Binaer-Literal)", line, col));
        }

        if (ch == 'f' || ch == 'F') && self.peek(1) == '"' {
            return self.scan_fstring(line, col);
        }

        if ch.is_alphabetic() || ch == '_' {
            return self.scan_ident(line, col);
        }

        self.scan_operator(line, col)
    }

    fn scan_string(&mut self, line: usize, col: usize) -> Result<(), LexError> {
        self.advance();
        let mut chars = String::new();
        loop {
            let c = self.peek(0);
            if c == '\0' { return Err(self.err("Unterminierter String", line, col)); }
            if c == '\n' {
                return Err(self.err("Zeilenumbruch im String nicht erlaubt", line, col));
            }
            if c == '"' {
                if self.peek(1) == '"' {
                    self.advance(); self.advance(); chars.push('"'); continue;
                }
                self.advance();
                break;
            }
            chars.push(self.advance());
        }
        self.push(Tt::Str, Val::Str(chars), line, col);
        Ok(())
    }

    fn scan_number(&mut self, line: usize, col: usize) -> Result<(), LexError> {
        let mut s = String::new();
        while self.peek(0).is_ascii_digit() { s.push(self.advance()); }
        let mut is_float = false;
        if self.peek(0) == '.' && self.peek(1).is_ascii_digit() {
            is_float = true;
            s.push(self.advance());
            while self.peek(0).is_ascii_digit() { s.push(self.advance()); }
        }
        let val = if is_float {
            Val::Float(s.parse::<f64>().unwrap_or(0.0))
        } else {
            // Zu grosse Ganzzahl-Literale NICHT still zu 0 machen, sondern als
            // Fehler melden (INTEGER ist 64-bit).
            match s.parse::<i64>() {
                Ok(n) => Val::Int(n),
                Err(_) => return Err(self.err(
                    "Ganzzahl-Literal zu gross (INTEGER ist 64-bit)", line, col)),
            }
        };
        self.push(Tt::Number, val, line, col);
        Ok(())
    }

    fn scan_hex_or_binary(&mut self, line: usize, col: usize) -> Result<(), LexError> {
        self.advance();                  // &
        let marker = self.advance();     // H/B
        let mut s = String::new();
        let value: i64;
        if marker == 'H' || marker == 'h' {
            loop {
                let c = self.peek(0);
                if c != '\0' && (c.is_ascii_digit()
                    || matches!(c.to_ascii_lowercase(), 'a'..='f')) {
                    s.push(self.advance());
                } else { break; }
            }
            if s.is_empty() {
                return Err(self.err("Hex-Literal &H ohne Ziffern", line, col));
            }
            value = i64::from_str_radix(&s, 16).unwrap_or(0);
        } else {
            while matches!(self.peek(0), '0' | '1') { s.push(self.advance()); }
            if s.is_empty() {
                return Err(self.err("Binaer-Literal &B ohne Ziffern", line, col));
            }
            value = i64::from_str_radix(&s, 2).unwrap_or(0);
        }
        self.push(Tt::Number, Val::Int(value), line, col);
        Ok(())
    }

    fn scan_ident(&mut self, line: usize, col: usize) -> Result<(), LexError> {
        let mut s = String::new();
        while self.peek(0).is_alphanumeric() || self.peek(0) == '_' {
            s.push(self.advance());
        }
        if self.peek(0) == '$' { s.push(self.advance()); }
        let text = s.to_lowercase();

        if text == "rem" {
            while self.peek(0) != '\0' && self.peek(0) != '\n' { self.advance(); }
            return Ok(());
        }

        match keyword(&text) {
            Some(Tt::True) => self.push(Tt::True, Val::Bool(true), line, col),
            Some(Tt::False) => self.push(Tt::False, Val::Bool(false), line, col),
            Some(tt) => self.push(tt, Val::Str(text), line, col),
            None => self.push(Tt::Ident, Val::Str(text), line, col),
        }
        Ok(())
    }

    fn scan_fstring(&mut self, line: usize, col: usize) -> Result<(), LexError> {
        self.advance();    // f
        self.advance();    // "
        // parts: (true=text|false=expr, content)
        let mut parts: Vec<(bool, String)> = Vec::new();
        let mut cur = String::new();
        loop {
            let c = self.peek(0);
            if c == '\0' { return Err(self.err("Unterminierter f-String", line, col)); }
            if c == '\n' {
                return Err(self.err("Zeilenumbruch im f-String nicht erlaubt", line, col));
            }
            if c == '"' {
                if self.peek(1) == '"' {
                    cur.push('"'); self.advance(); self.advance(); continue;
                }
                self.advance();
                if !cur.is_empty() { parts.push((true, std::mem::take(&mut cur))); }
                break;
            }
            if c == '{' {
                if self.peek(1) == '{' {
                    cur.push('{'); self.advance(); self.advance(); continue;
                }
                if !cur.is_empty() { parts.push((true, std::mem::take(&mut cur))); }
                self.advance();    // {
                let mut expr = String::new();
                let mut depth = 1;
                loop {
                    let cc = self.peek(0);
                    if cc == '\0' {
                        return Err(self.err("Unterminierter Ausdruck im f-String", line, col));
                    }
                    if cc == '\n' {
                        return Err(self.err("Zeilenumbruch im f-String-Ausdruck nicht erlaubt", line, col));
                    }
                    if cc == '{' { depth += 1; }
                    else if cc == '}' {
                        depth -= 1;
                        if depth == 0 { self.advance(); break; }
                    }
                    expr.push(self.advance());
                }
                if expr.is_empty() {
                    return Err(self.err("Leerer f-String-Ausdruck {}", line, col));
                }
                parts.push((false, expr));
                continue;
            }
            if c == '}' {
                if self.peek(1) == '}' {
                    cur.push('}'); self.advance(); self.advance(); continue;
                }
                return Err(self.err("Einzelnes '}' im f-String (verwende '}}')", line, col));
            }
            cur.push(self.advance());
        }

        if parts.is_empty() {
            self.push(Tt::Str, Val::Str(String::new()), line, col);
            return Ok(());
        }

        self.push(Tt::Lparen, Val::Str("(".into()), line, col);
        let mut first = true;
        for (is_text, text) in parts {
            if !first { self.push(Tt::Plus, Val::Str("+".into()), line, col); }
            first = false;
            if is_text {
                self.push(Tt::Str, Val::Str(text), line, col);
            } else {
                let (expr_src, spec) = split_fstring_spec(&text);
                let (fn_name, suffix): (&str, Option<String>) = if let Some(sp) = spec {
                    ("format$", Some(sp))
                } else {
                    ("str$", None)
                };
                let expr_to_lex = if suffix.is_some() { expr_src } else { text.clone() };
                self.push(Tt::Ident, Val::Str(fn_name.into()), line, col);
                self.push(Tt::Lparen, Val::Str("(".into()), line, col);
                // Sub-Lexer fuer den Ausdruck; trailing NEWLINE/EOF weg, line/col
                // auf den f-String-Ort ueberschreiben.
                let inner = Lexer::new(&expr_to_lex).tokenize()?;
                for mut tok in inner {
                    if matches!(tok.tt, Tt::Newline | Tt::Eof) { continue; }
                    tok.line = line;
                    tok.col = col;
                    self.toks.push(tok);
                }
                if let Some(sp) = suffix {
                    self.push(Tt::Comma, Val::Str(",".into()), line, col);
                    self.push(Tt::Str, Val::Str(format!("%{}", sp)), line, col);
                }
                self.push(Tt::Rparen, Val::Str(")".into()), line, col);
            }
        }
        self.push(Tt::Rparen, Val::Str(")".into()), line, col);
        Ok(())
    }

    fn scan_operator(&mut self, line: usize, col: usize) -> Result<(), LexError> {
        let ch = self.advance();
        macro_rules! one { ($tt:expr, $s:expr) => { self.push($tt, Val::Str($s.into()), line, col) } }
        match ch {
            '+' => if self.peek(0) == '=' { self.advance(); one!(Tt::PlusEq, "+="); } else { one!(Tt::Plus, "+"); },
            '-' => if self.peek(0) == '=' { self.advance(); one!(Tt::MinusEq, "-="); } else { one!(Tt::Minus, "-"); },
            '*' => if self.peek(0) == '=' { self.advance(); one!(Tt::StarEq, "*="); } else { one!(Tt::StarT, "*"); },
            '/' => if self.peek(0) == '=' { self.advance(); one!(Tt::SlashEq, "/="); } else { one!(Tt::Slash, "/"); },
            '\\' => one!(Tt::Intdiv, "\\"),
            '^' => one!(Tt::Caret, "^"),
            '(' => { self.paren_depth += 1; one!(Tt::Lparen, "("); },
            ')' => { if self.paren_depth > 0 { self.paren_depth -= 1; } one!(Tt::Rparen, ")"); },
            '[' => { self.paren_depth += 1; one!(Tt::Lbracket, "["); },
            ']' => { if self.paren_depth > 0 { self.paren_depth -= 1; } one!(Tt::Rbracket, "]"); },
            '{' => { self.paren_depth += 1; one!(Tt::Lbrace, "{"); },
            '}' => { if self.paren_depth > 0 { self.paren_depth -= 1; } one!(Tt::Rbrace, "}"); },
            ',' => one!(Tt::Comma, ","),
            '.' => if self.peek(0) == '.' && self.peek(1) == '.' {
                self.advance(); self.advance(); one!(Tt::Ellipsis, "...");
            } else { one!(Tt::Dot, "."); },
            ';' => one!(Tt::Semicolon, ";"),
            ':' => one!(Tt::Colon, ":"),
            '=' => one!(Tt::Eq, "="),
            '<' => if self.peek(0) == '=' { self.advance(); one!(Tt::Leq, "<="); }
                   else if self.peek(0) == '>' { self.advance(); one!(Tt::Neq, "<>"); }
                   else { one!(Tt::Lt, "<"); },
            '>' => if self.peek(0) == '=' { self.advance(); one!(Tt::Geq, ">="); }
                   else { one!(Tt::Gt, ">"); },
            _ => return Err(self.err(&format!("Unbekanntes Zeichen: {:?}", ch), line, col)),
        }
        Ok(())
    }
}

/// Wie `_split_fstring_spec` in lexer.py: trennt am ersten Top-Level-`:`.
fn split_fstring_spec(s: &str) -> (String, Option<String>) {
    let chars: Vec<char> = s.chars().collect();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if in_str {
            if ch == '"' {
                if i + 1 < chars.len() && chars[i + 1] == '"' { i += 2; continue; }
                in_str = false;
            }
        } else {
            match ch {
                '"' => in_str = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                ':' if depth == 0 => {
                    let a: String = chars[..i].iter().collect();
                    let b: String = chars[i + 1..].iter().collect();
                    return (a.trim().to_string(), Some(b.trim().to_string()));
                }
                _ => {}
            }
        }
        i += 1;
    }
    (s.to_string(), None)
}

/// Kanonischer JSON-Dump `[TYP, wert, zeile]` pro Token (eine Zeile je Token).
/// Identisches Format auf der Python-Seite -> Parity-Vergleich.
pub fn dump_tokens_json(source: &str) -> Result<String, LexError> {
    let toks = Lexer::new(source).tokenize()?;
    let mut out = String::new();
    for t in &toks {
        let v: J = match &t.val {
            Val::Nil => J::Null,
            Val::Bool(b) => J::Bool(*b),
            Val::Int(i) => J::from(*i),
            Val::Float(f) => J::from(*f),
            Val::Str(s) => J::String(s.clone()),
        };
        let arr = J::Array(vec![J::String(t.tt.name().to_string()), v,
                                J::from(t.line as i64)]);
        out.push_str(&serde_json::to_string(&arr).unwrap());
        out.push('\n');
    }
    Ok(out)
}
