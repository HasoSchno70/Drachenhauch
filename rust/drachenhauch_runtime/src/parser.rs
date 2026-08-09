//! Recursive-Descent-Parser fuer Drachenhauch -- Rust-Port von
//! `drachenhauch/parser.py`. Stufe 2 der Front-End-Portierung. Verifiziert gegen
//! Python via `dhrt --ast` (AST als kanonisches JSON) -- siehe
//! `tests/test_rust_parser_parity.rs`/`.py`.

use std::collections::HashSet;

use crate::ast::{CaseMatch, CaseVal, Node, NumV, Param};
use crate::lexer::{Token, Tt, Val};

pub struct ParseError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}
type R<T> = Result<T, ParseError>;

fn compound_op(tt: Tt) -> Option<&'static str> {
    match tt {
        Tt::PlusEq => Some("+"),
        Tt::MinusEq => Some("-"),
        Tt::StarEq => Some("*"),
        Tt::SlashEq => Some("/"),
        _ => None,
    }
}
fn is_assign_op(tt: Tt) -> bool {
    matches!(tt, Tt::Eq | Tt::PlusEq | Tt::MinusEq | Tt::StarEq | Tt::SlashEq)
}
fn type_token(tt: Tt) -> Option<&'static str> {
    Some(match tt {
        Tt::Integer => "integer", Tt::Float => "float", Tt::StringType => "string",
        Tt::Boolean => "boolean", Tt::Image => "image", Tt::Sound => "sound",
        Tt::File => "file", Tt::Tuple => "tuple", Tt::Funcref => "funcref",
        Tt::Coroutine => "coroutine",
        _ => return None,
    })
}
fn operator_name(tt: Tt) -> Option<&'static str> {
    Some(match tt {
        Tt::Plus => "__op_add__", Tt::Minus => "__op_sub__", Tt::StarT => "__op_mul__",
        Tt::Slash => "__op_div__", Tt::Mod => "__op_mod__", Tt::Eq => "__op_eq__",
        Tt::Neq => "__op_ne__", Tt::Lt => "__op_lt__", Tt::Gt => "__op_gt__",
        Tt::Leq => "__op_le__", Tt::Geq => "__op_ge__",
        _ => return None,
    })
}

/// String-Wert eines Tokens (IDENT/Keyword/Operator), sonst leer.
fn sval(t: &Token) -> String {
    match &t.val { Val::Str(s) => s.clone(), _ => String::new() }
}

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    enum_names: HashSet<String>,
    with_counter: usize,
    with_stack: Vec<String>,
}

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self {
        Parser { toks, pos: 0, enum_names: HashSet::new(),
                 with_counter: 0, with_stack: Vec::new() }
    }

    fn peek(&self, off: usize) -> &Token {
        let i = self.pos + off;
        if i >= self.toks.len() { self.toks.last().unwrap() } else { &self.toks[i] }
    }
    fn tt(&self, off: usize) -> Tt { self.peek(off).tt }
    fn at_end(&self) -> bool { self.tt(0) == Tt::Eof }
    fn check(&self, t: Tt) -> bool { self.tt(0) == t }
    fn checks(&self, ts: &[Tt]) -> bool { ts.contains(&self.tt(0)) }

    /// Konsumiert und liefert das Token, wenn der Typ passt.
    fn eat(&mut self, t: Tt) -> Option<Token> {
        if self.tt(0) == t { let tok = self.peek(0).clone(); self.pos += 1; Some(tok) }
        else { None }
    }
    fn matches(&mut self, t: Tt) -> bool { self.eat(t).is_some() }

    fn expect(&mut self, t: Tt, msg: &str) -> R<Token> {
        if self.tt(0) == t { let tok = self.peek(0).clone(); self.pos += 1; Ok(tok) }
        else {
            let a = self.peek(0);
            // Leere `msg` (viele Call-Sites geben "" bei bereits per match/check
            // abgesicherten Erwartungen) faellt auf eine generische, aber nie
            // ganz leere Meldung zurueck -- verhindert stumme "zeile:spalte: "
            // Fehler an tatsaechlich erreichbaren Stellen (z.B. fehlende ')'/'=').
            let text = if msg.is_empty() { format!("Erwartet {:?}", t) } else { msg.to_string() };
            Err(ParseError { msg: text, line: a.line, col: a.col })
        }
    }

    fn err<T>(&self, msg: &str) -> R<T> {
        let a = self.peek(0);
        Err(ParseError { msg: msg.to_string(), line: a.line, col: a.col })
    }

    fn skip_newlines(&mut self) {
        while self.matches(Tt::Newline) || self.matches(Tt::Colon) {}
    }

    fn consume_terminator(&mut self) -> R<()> {
        if self.check(Tt::Newline) || self.check(Tt::Colon) {
            while self.matches(Tt::Newline) || self.matches(Tt::Colon) {}
            return Ok(());
        }
        if self.at_end() { return Ok(()); }
        self.err("Erwartet Zeilenende")
    }

    // ------------------------------------------------------ Eintritt
    pub fn parse(&mut self) -> R<Node> {
        self.skip_newlines();
        let mut stmts = Vec::new();
        while !self.at_end() {
            stmts.push(self.statement()?);
            self.skip_newlines();
        }
        Ok(Node::Program { statements: stmts })
    }

    fn statement(&mut self) -> R<Node> {
        // Quell-Zeile des ersten Tokens merken und das Statement damit umhuellen
        // (Stufe B: fuer lines[] -> Profiler/Debugger/Laufzeitfehler-Zeilen).
        let line = self.peek(0).line as u32;
        let body = self.statement_inner()?;
        Ok(Node::Stmt { line, body: Box::new(body) })
    }

    fn statement_inner(&mut self) -> R<Node> {
        match self.tt(0) {
            Tt::Dim => self.dim(),
            Tt::Print => self.print_stmt(),
            Tt::Input => self.input_stmt(),
            Tt::If => self.if_stmt(),
            Tt::Select => self.select(),
            Tt::While => self.while_stmt(),
            Tt::Repeat => self.repeat(),
            Tt::For => self.for_stmt(),
            Tt::Data => self.data(),
            Tt::Read => self.read(),
            Tt::Restore => self.restore(),
            Tt::Sub => self.sub_decl(),
            Tt::Function => self.function_decl(),
            Tt::Return => self.return_stmt(),
            Tt::Class => self.class_decl(),
            Tt::Struct => self.struct_decl(),
            Tt::Const => self.const_stmt(),
            Tt::Enum => self.enum_decl(),
            Tt::Break => { self.expect(Tt::Break, "")?; self.consume_terminator()?; Ok(Node::Break) }
            Tt::Continue => { self.expect(Tt::Continue, "")?; self.consume_terminator()?; Ok(Node::Continue) }
            Tt::Try => self.try_stmt(),
            Tt::Throw => self.throw_stmt(),
            Tt::With => self.with_stmt(),
            Tt::Dot if !self.with_stack.is_empty() => self.dot_assign_in_with(),
            Tt::Ident if self.is_assignment_lookahead() => self.assign(),
            Tt::Lparen if self.is_tuple_assign_lookahead() => self.tuple_assign(),
            _ => {
                let expr = self.expression()?;
                self.consume_terminator()?;
                // Review-Fund (Sicherheitsnetz, siehe inline_statement): kein
                // gueltiges GB-Programm berechnet einen Top-Level-Vergleich
                // und verwirft ihn -- jede Lvalue-Form, die trotz der obigen
                // Zweige noch durchrutscht, bricht hier klar statt lautlos.
                if let Node::BinaryOp { op, .. } = &expr {
                    if op.as_str() == "=" {
                        return self.err("'=' als Anweisung -- meintest du eine Zuweisung?");
                    }
                }
                Ok(Node::ExprStmt { expr: Box::new(expr) })
            }
        }
    }

    // ------------------------------------------------------ Lookaheads
    fn is_tuple_assign_lookahead(&self) -> bool {
        let mut i = 1;
        let mut depth = 1;
        let mut saw_top_comma = false;
        loop {
            let tt = self.tt(i);
            if tt == Tt::Eof || tt == Tt::Newline { return false; }
            if tt == Tt::Lparen || tt == Tt::Lbracket { depth += 1; }
            else if tt == Tt::Rparen || tt == Tt::Rbracket {
                depth -= 1;
                if depth == 0 { break; }
            } else if tt == Tt::Comma && depth == 1 { saw_top_comma = true; }
            i += 1;
        }
        saw_top_comma && self.tt(i + 1) == Tt::Eq
    }

    fn is_assignment_lookahead(&self) -> bool {
        let mut i = 1;
        loop {
            let t = self.tt(i);
            // Review-Fund: verlangte bisher striktes Tt::Ident nach dem Punkt
            // -- ein Membername, der zufaellig wie ein Keyword lexed (z.B.
            // `.image`, `.sound`, `.data`), liess die Lookahead-Erkennung
            // fehlschlagen, und `spr.image = x` wurde dann als normale
            // Ausdrucks-Anweisung geparst (`=` als Vergleichsoperator -> die
            // Zuweisung ging lautlos verloren). Gleiche Toleranz wie
            // postfix()/member_name_after_dot(): jedes Token mit einem
            // nicht-leeren String-Wert ist ein gueltiger Membername.
            if t == Tt::Dot && matches!(&self.peek(i + 1).val, Val::Str(s) if !s.is_empty()) {
                i += 2; continue;
            }
            if t == Tt::Lbracket {
                let mut depth = 1;
                i += 1;
                while depth > 0 {
                    let tt = self.tt(i);
                    if tt == Tt::Eof { return false; }
                    if tt == Tt::Lbracket { depth += 1; }
                    else if tt == Tt::Rbracket { depth -= 1; }
                    i += 1;
                }
                continue;
            }
            break;
        }
        is_assign_op(self.tt(i))
    }

    fn tuple_assign(&mut self) -> R<Node> {
        self.expect(Tt::Lparen, "")?;
        let mut targets = vec![self.tuple_assign_target()?];
        while self.matches(Tt::Comma) { targets.push(self.tuple_assign_target()?); }
        if targets.len() < 2 {
            return self.err("Tupel-Destructuring erwartet mindestens 2 Ziele");
        }
        self.expect(Tt::Rparen, "")?;
        self.expect(Tt::Eq, "Erwartet '=' nach Tupel-Zielen")?;
        let value = self.expression()?;
        self.consume_terminator()?;
        Ok(Node::TupleAssign { targets, value: Box::new(value) })
    }

    fn tuple_assign_target(&mut self) -> R<Node> {
        let name = sval(&self.expect(Tt::Ident, "Erwartet Variablenname als Tupel-Ziel")?);
        let mut target = Node::Identifier(name);
        loop {
            if self.matches(Tt::Dot) {
                let m = self.member_name_after_dot()?;
                target = Node::MemberAccess { target: Box::new(target), name: m };
            } else if self.matches(Tt::Lbracket) {
                let mut idxs = vec![self.expression()?];
                while self.matches(Tt::Comma) { idxs.push(self.expression()?); }
                self.expect(Tt::Rbracket, "Erwartet ']'")?;
                target = Node::IndexAccess { target: Box::new(target), indices: idxs };
            } else { break; }
        }
        Ok(target)
    }

    // ------------------------------------------------------ DIM/CONST/ENUM
    fn dim(&mut self) -> R<Node> {
        self.expect(Tt::Dim, "")?;
        let mut decls: Vec<(String, Option<Vec<Node>>)> = Vec::new();
        loop {
            // Hilfreichere Meldung, wenn ein reserviertes Wort als Name kommt
            // (z.B. DIM band -> band ist der BAND-Operator).
            let reserved = {
                let t = self.peek(0);
                if t.tt != Tt::Ident && t.tt != Tt::Str {
                    match &t.val {
                        Val::Str(s) if crate::lexer::keyword(s).is_some() => Some(s.to_uppercase()),
                        _ => None,
                    }
                } else { None }
            };
            if let Some(kw) = reserved {
                return self.err(&format!(
                    "'{}' ist ein reserviertes Wort und kann kein Variablenname sein - waehle einen anderen Namen", kw));
            }
            let name = sval(&self.expect(Tt::Ident, "Erwartet Variablenname nach DIM")?);
            let mut array_dims = None;
            if self.matches(Tt::Lbracket) {
                let mut dims = vec![self.expression()?];
                while self.matches(Tt::Comma) { dims.push(self.expression()?); }
                self.expect(Tt::Rbracket, "Erwartet ']'")?;
                array_dims = Some(dims);
            }
            decls.push((name, array_dims));
            if !self.matches(Tt::Comma) { break; }
        }
        self.expect(Tt::As, "Erwartet AS nach Variablenname")?;
        let type_name = self.parse_type()?;
        self.consume_terminator()?;
        if decls.len() == 1 {
            let (name, dims) = decls.pop().unwrap();
            Ok(Node::Dim { name, type_name, array_dims: dims })
        } else {
            let dims: Vec<Node> = decls.into_iter()
                .map(|(name, d)| Node::Dim { name, type_name: type_name.clone(), array_dims: d })
                .collect();
            Ok(Node::MultiDim { dims })
        }
    }

    fn const_stmt(&mut self) -> R<Node> {
        self.expect(Tt::Const, "")?;
        let name = sval(&self.expect(Tt::Ident, "Erwartet Konstantenname nach CONST")?);
        let mut type_name = None;
        if self.matches(Tt::As) { type_name = Some(self.parse_type()?); }
        self.expect(Tt::Eq, "Erwartet '=' nach CONST-Name")?;
        let value = self.expression()?;
        self.consume_terminator()?;
        Ok(Node::Const { name, type_name, value: Box::new(value) })
    }

    fn consume_member_name(&mut self) -> R<String> {
        let tok = self.peek(0);
        if let Val::Str(s) = &tok.val {
            if !s.is_empty() { let r = s.clone(); self.pos += 1; return Ok(r); }
        }
        self.err("Erwartet Member-Name in ENUM")
    }

    /// `.member` nach einem bereits konsumierten `.` -- akzeptiert JEDES
    /// Token mit einem String-Wert (nicht nur Tt::Ident), damit Member-Namen,
    /// die zufaellig wie ein Keyword lexen (`.image`, `.sound`, `.data`,
    /// `.next`, ...), funktionieren. Gleiche Toleranz wie `postfix()`s
    /// inline-Variante -- Review-Fund: `dot_assign_in_with`/
    /// `tuple_assign_target` nutzten stattdessen `expect(Tt::Ident, ...)` und
    /// scheiterten (oder liesen die Zuweisung im schlimmeren Fall gar nicht
    /// erst als Zuweisung erkennen) bei genau solchen Namen.
    fn member_name_after_dot(&mut self) -> R<String> {
        let tok = self.peek(0).clone();
        if let Val::Str(s) = &tok.val {
            if !s.is_empty() { self.pos += 1; return Ok(s.clone()); }
        }
        self.err("Erwartet Membername nach '.'")
    }

    fn enum_decl(&mut self) -> R<Node> {
        self.expect(Tt::Enum, "")?;
        let name = sval(&self.expect(Tt::Ident, "Erwartet ENUM-Name")?);
        let mut members: Vec<(String, Option<Node>)> = Vec::new();
        if self.matches(Tt::Eq) {
            loop {
                let m = self.consume_member_name()?;
                let ve = if self.matches(Tt::Eq) { Some(self.expression()?) } else { None };
                members.push((m, ve));
                if !self.matches(Tt::Comma) { break; }
            }
            self.consume_terminator()?;
        } else {
            self.consume_terminator()?;
            loop {
                self.skip_newlines();
                if self.check(Tt::End) { break; }
                if self.at_end() { return self.err("END ENUM erwartet"); }
                let m = self.consume_member_name()?;
                let ve = if self.matches(Tt::Eq) { Some(self.expression()?) } else { None };
                members.push((m, ve));
                if !self.matches(Tt::Comma) { self.consume_terminator()?; }
            }
            self.expect(Tt::End, "")?;
            self.expect(Tt::Enum, "Erwartet ENUM nach END")?;
            self.consume_terminator()?;
        }
        if members.is_empty() { return self.err("ENUM: mindestens ein Member erforderlich"); }
        let mut seen = HashSet::new();
        for (mn, _) in &members {
            if !seen.insert(mn.to_lowercase()) {
                return self.err("ENUM: Member doppelt deklariert");
            }
        }
        self.enum_names.insert(name.to_lowercase());
        Ok(Node::EnumDecl { name, members })
    }

    // ------------------------------------------------------ Call-Args
    fn call_args(&mut self) -> R<Vec<Node>> {
        let mut args = Vec::new();
        if self.check(Tt::Rparen) { return Ok(args); }
        args.push(self.call_arg()?);
        while self.matches(Tt::Comma) { args.push(self.call_arg()?); }
        Ok(args)
    }

    fn call_arg(&mut self) -> R<Node> {
        if self.tt(0) == Tt::Ident && self.tt(1) == Tt::Colon {
            let name = sval(self.peek(0));
            self.pos += 2;
            let value = self.expression()?;
            return Ok(Node::NamedArg { name, value: Box::new(value) });
        }
        self.expression()
    }

    // ------------------------------------------------------ WITH
    fn with_stmt(&mut self) -> R<Node> {
        self.expect(Tt::With, "")?;
        let target = self.expression()?;
        self.consume_terminator()?;
        let var_name = format!("__with_{}", self.with_counter);
        self.with_counter += 1;
        self.with_stack.push(var_name.clone());
        let body = self.block_until(&[Tt::End], "END WITH erwartet, Programmende erreicht");
        self.with_stack.pop();   // in jedem Fall abbauen (Erfolg UND Fehlerpfad)
        let body = body?;
        self.expect(Tt::End, "")?;
        self.expect(Tt::With, "Erwartet WITH nach END")?;
        self.consume_terminator()?;
        Ok(Node::With { var_name, target: Box::new(target), body })
    }

    fn dot_assign_in_with(&mut self) -> R<Node> {
        // Review-Fund: Position merken -- ein `.Method(...)`-STATEMENT (kein
        // `=`/Compound-Op danach, z.B. `.Draw()`) muss zurueckspulen und
        // generisch als Expression-Statement geparst werden (das der
        // Expressions-Parser primary()/postfix() ueber den with_stack schon
        // korrekt aufloest) -- vorher endete jede Methoden-Aufruf-Anweisung
        // in einem WITH-Block hart mit "Erwartet '=' oder Compound-Operator".
        let start = self.pos;
        let mut target = Node::Identifier(self.with_stack.last().unwrap().clone());
        loop {
            if self.matches(Tt::Dot) {
                let m = self.member_name_after_dot()?;
                target = Node::MemberAccess { target: Box::new(target), name: m };
            } else if self.matches(Tt::Lbracket) {
                let mut idxs = vec![self.expression()?];
                while self.matches(Tt::Comma) { idxs.push(self.expression()?); }
                self.expect(Tt::Rbracket, "Erwartet ']'")?;
                target = Node::IndexAccess { target: Box::new(target), indices: idxs };
            } else { break; }
        }
        let op_tt = self.tt(0);
        if !is_assign_op(op_tt) {
            self.pos = start;
            let expr = self.expression()?;
            self.consume_terminator()?;
            return Ok(Node::ExprStmt { expr: Box::new(expr) });
        }
        self.pos += 1;
        let cop = compound_op(op_tt);
        let mut rhs = self.expression()?;
        if let Some(op) = cop {
            rhs = Node::BinaryOp { op: op.into(), left: Box::new(target.clone()), right: Box::new(rhs) };
        }
        self.consume_terminator()?;
        match target {
            Node::MemberAccess { target, name } =>
                Ok(Node::MemberAssign { target, name, value: Box::new(rhs) }),
            Node::IndexAccess { target, indices } =>
                Ok(Node::IndexAssign { target, indices, value: Box::new(rhs) }),
            _ => self.err("Leeres `.` in WITH-Block (erwartet `.member`)"),
        }
    }

    // ------------------------------------------------------ TRY/THROW
    fn try_stmt(&mut self) -> R<Node> {
        self.expect(Tt::Try, "")?;
        self.consume_terminator()?;
        let body = self.block_until(&[Tt::Catch, Tt::End], "CATCH oder END TRY erwartet")?;
        let mut catch_var = String::new();
        let mut catch_block = Vec::new();
        if self.matches(Tt::Catch) {
            if self.check(Tt::Ident) { catch_var = sval(self.peek(0)); self.pos += 1; }
            self.consume_terminator()?;
            catch_block = self.block_until(&[Tt::End], "END TRY erwartet")?;
        }
        self.expect(Tt::End, "")?;
        self.expect(Tt::Try, "Erwartet TRY nach END")?;
        self.consume_terminator()?;
        Ok(Node::Try { body, catch_var, catch_block })
    }

    fn throw_stmt(&mut self) -> R<Node> {
        self.expect(Tt::Throw, "")?;
        let value = self.expression()?;
        self.consume_terminator()?;
        Ok(Node::Throw { value: Box::new(value) })
    }

    // ------------------------------------------------------ Typen
    fn parse_type(&mut self) -> R<String> {
        let t = self.tt(0);
        if t == Tt::Array {
            self.pos += 1;
            self.expect(Tt::Of, "Erwartet OF nach ARRAY")?;
            let elem = self.parse_type()?;
            return Ok(format!("array:{}", elem));
        }
        if t == Tt::Map {
            self.pos += 1;
            self.expect(Tt::Of, "Erwartet OF nach MAP")?;
            let first = self.parse_type()?;
            if self.matches(Tt::To) {
                if first != "string" {
                    return self.err("MAP-Schluessel muessen STRING sein (Phase 5a)");
                }
                let vt = self.parse_type()?;
                return Ok(format!("map:{}", vt));
            }
            return Ok(format!("map:{}", first));
        }
        if let Some(name) = type_token(t) { self.pos += 1; return Ok(name.to_string()); }
        if t == Tt::Ident {
            let v = sval(self.peek(0));
            self.pos += 1;
            if self.enum_names.contains(&v.to_lowercase()) { return Ok("integer".into()); }
            return Ok(v);
        }
        self.err("Erwartet Typ")
    }

    // ------------------------------------------------------ Assign
    fn assign(&mut self) -> R<Node> {
        let stmt = self.assign_from_lvalue()?;
        self.consume_terminator()?;
        Ok(stmt)
    }

    fn assign_from_lvalue(&mut self) -> R<Node> {
        let name = sval(&self.expect(Tt::Ident, "")?);
        let mut target = Node::Identifier(name);
        loop {
            if self.matches(Tt::Dot) {
                // Review-Fund: `expect(Tt::Ident, ...)` lehnte jeden Membernamen
                // ab, der zufaellig wie ein Keyword lexed (`.image`, `.sound`,
                // `.data`, `.next`, ...) -- kombiniert mit der (ebenfalls
                // gefixten) is_assignment_lookahead() fuehrte das dazu, dass
                // `spr.image = LoadImage(...)` entweder lautlos zu einem
                // verworfenen Vergleich wurde oder hier hart fehlschlug.
                let m = self.member_name_after_dot()?;
                target = Node::MemberAccess { target: Box::new(target), name: m };
            } else if self.matches(Tt::Lbracket) {
                const SLICE_ASSIGN_MSG: &str =
                    "Slice-Zuweisung (`x[a:b] = ...`) ist nicht unterstuetzt -- nutze eine Schleife.";
                if self.check(Tt::Colon) {
                    return self.err(SLICE_ASSIGN_MSG);
                }
                let mut indices = vec![self.expression()?];
                if self.check(Tt::Colon) {
                    return self.err(SLICE_ASSIGN_MSG);
                }
                // Review-Fund: die Slice-Pruefung galt bisher nur fuer den
                // ERSTEN Index -- `x[1, 2:3] = 5` ueberspringt sie und
                // scheitert stattdessen mit der generischen "Erwartet ']'"-
                // Meldung. Jetzt bei jedem weiteren Komma-Index ebenfalls
                // geprueft.
                while self.matches(Tt::Comma) {
                    indices.push(self.expression()?);
                    if self.check(Tt::Colon) {
                        return self.err(SLICE_ASSIGN_MSG);
                    }
                }
                self.expect(Tt::Rbracket, "Erwartet ']'")?;
                target = Node::IndexAccess { target: Box::new(target), indices };
            } else { break; }
        }
        let op_tt = self.tt(0);
        if !is_assign_op(op_tt) {
            return self.err("Erwartet '=' oder Compound-Operator");
        }
        self.pos += 1;
        let cop = compound_op(op_tt);
        let mut rhs = self.expression()?;
        if let Some(op) = cop {
            rhs = Node::BinaryOp { op: op.into(), left: Box::new(target.clone()), right: Box::new(rhs) };
        }
        match target {
            Node::Identifier(name) => Ok(Node::Assign { name, value: Box::new(rhs) }),
            Node::MemberAccess { target, name } =>
                Ok(Node::MemberAssign { target, name, value: Box::new(rhs) }),
            Node::IndexAccess { target, indices } =>
                Ok(Node::IndexAssign { target, indices, value: Box::new(rhs) }),
            _ => self.err("Ungueltige Zuweisungs-Linksseite"),
        }
    }

    // ------------------------------------------------------ PRINT/INPUT
    fn print_stmt(&mut self) -> R<Node> {
        self.expect(Tt::Print, "")?;
        // Review-Fund: fehlte Tt::Colon -- `PRINT : PRINT "x"` (klassisches
        // BASIC-Idiom fuer eine Leerzeile) warf "Unerwartetes Token COLON",
        // weil das leere `:` in print_items()->expression() lief.
        // print_items() selbst behandelt ':' als Trenner bereits korrekt
        // (Zeile weiter unten) -- nur dieser Leer-PRINT-Fall fehlte.
        if self.check(Tt::Newline) || self.check(Tt::Colon) || self.at_end() {
            self.consume_terminator()?;
            return Ok(Node::Print { items: vec![], seps: vec![], newline: true });
        }
        let (items, seps, newline) = self.print_items()?;
        self.consume_terminator()?;
        Ok(Node::Print { items, seps, newline })
    }

    /// PRINT-Liste: Ausdruecke getrennt durch ',' (Leerzeichen) oder ';' (kein
    /// Leerzeichen). Ein trailing ',' / ';' unterdrueckt den Zeilenumbruch.
    fn print_items(&mut self) -> R<(Vec<Node>, Vec<String>, bool)> {
        let mut items = vec![self.expression()?];
        let mut seps: Vec<String> = Vec::new();
        let mut newline = true;
        while self.checks(&[Tt::Comma, Tt::Semicolon]) {
            let sep = if self.matches(Tt::Comma) { "," } else { self.matches(Tt::Semicolon); ";" };
            if self.checks(&[Tt::Newline, Tt::Colon, Tt::Else]) || self.at_end() {
                newline = false;           // trailing Trenner -> kein Newline
                break;
            }
            seps.push(sep.to_string());
            items.push(self.expression()?);
        }
        Ok((items, seps, newline))
    }

    fn input_stmt(&mut self) -> R<Node> {
        self.expect(Tt::Input, "")?;
        let mut prompt = None;
        if self.check(Tt::Str) {
            prompt = Some(Box::new(Node::StringLit(sval(self.peek(0)))));
            self.pos += 1;
            self.expect(Tt::Comma, "Nach Prompt-String erwartet ',' und Variable")?;
        }
        let name = sval(&self.expect(Tt::Ident, "Erwartet Variable nach INPUT")?);
        self.consume_terminator()?;
        Ok(Node::Input { prompt, target: name })
    }

    // ------------------------------------------------------ IF
    fn if_stmt(&mut self) -> R<Node> {
        self.expect(Tt::If, "")?;
        let cond = self.expression()?;
        self.expect(Tt::Then, "Erwartet THEN nach Bedingung")?;
        if !self.check(Tt::Newline) {
            // Single-Line: alle ':'-getrennten Statements nach THEN gehoeren zum
            // THEN-Zweig (klassisches BASIC, konsistent mit dem Block-IF) -- der
            // ELSE-Zweig analog. Schluss jeweils bei NEWLINE / ELSE / EOF.
            let mut then_block = vec![self.inline_statement()?];
            while self.matches(Tt::Colon) {
                if self.checks(&[Tt::Newline, Tt::Else]) || self.at_end() { break; }
                then_block.push(self.inline_statement()?);
            }
            let mut else_block = Vec::new();
            if self.matches(Tt::Else) {
                else_block.push(self.inline_statement()?);
                while self.matches(Tt::Colon) {
                    if self.check(Tt::Newline) || self.at_end() { break; }
                    else_block.push(self.inline_statement()?);
                }
            }
            self.consume_terminator()?;
            return Ok(Node::If { condition: Box::new(cond), then_block,
                                 elseif_branches: vec![], else_block });
        }
        self.consume_terminator()?;
        let then_terms = [Tt::Elseif, Tt::Else, Tt::End];
        let then_block = self.block_until(&then_terms, "END IF erwartet, Programmende erreicht")?;
        let mut elseif_branches = Vec::new();
        while self.matches(Tt::Elseif) {
            let ec = self.expression()?;
            self.expect(Tt::Then, "")?;
            self.consume_terminator()?;
            let block = self.block_until(&then_terms, "END IF erwartet, Programmende erreicht")?;
            elseif_branches.push((ec, block));
        }
        let mut else_block = Vec::new();
        if self.matches(Tt::Else) {
            self.consume_terminator()?;
            else_block = self.block_until(&[Tt::End], "END IF erwartet, Programmende erreicht")?;
        }
        self.expect(Tt::End, "Erwartet END IF")?;
        self.expect(Tt::If, "Erwartet IF nach END")?;
        self.consume_terminator()?;
        Ok(Node::If { condition: Box::new(cond), then_block, elseif_branches, else_block })
    }

    // ------------------------------------------------------ SELECT
    fn select(&mut self) -> R<Node> {
        self.expect(Tt::Select, "")?;
        self.expect(Tt::Case, "Erwartet CASE nach SELECT")?;
        let subject = self.expression()?;
        self.consume_terminator()?;
        let mut cases: Vec<(Vec<CaseMatch>, Option<Node>, Vec<Node>)> = Vec::new();
        let mut else_block = Vec::new();
        while !self.check(Tt::End) {
            if self.at_end() { return self.err("END SELECT erwartet"); }
            self.expect(Tt::Case, "Erwartet CASE oder END SELECT")?;
            if self.matches(Tt::Else) {
                // Ein zweites CASE ELSE ist unerreichbar: der "CASE nach
                // CASE ELSE"-Check unten erzwingt ELSE als letzten Fall.
                self.consume_terminator()?;
                while !self.check(Tt::End) {
                    if self.check(Tt::Case) {
                        return self.err("CASE nach CASE ELSE - ELSE muss letzter Fall sein");
                    }
                    if self.at_end() { return self.err("END SELECT erwartet"); }
                    else_block.push(self.statement()?);
                }
                break;
            }
            let mut matches = vec![self.case_match()?];
            while self.matches(Tt::Comma) { matches.push(self.case_match()?); }
            let guard = if self.matches(Tt::Where) { Some(self.expression()?) } else { None };
            self.consume_terminator()?;
            let block = self.block_until(&[Tt::Case, Tt::End], "END SELECT erwartet")?;
            cases.push((matches, guard, block));
        }
        self.expect(Tt::End, "Erwartet END SELECT")?;
        self.expect(Tt::Select, "Erwartet SELECT nach END")?;
        self.consume_terminator()?;
        Ok(Node::Select { subject: Box::new(subject), cases, else_block })
    }

    fn case_match(&mut self) -> R<CaseMatch> {
        if self.matches(Tt::Is) {
            let op = match self.tt(0) {
                Tt::Eq => "=", Tt::Neq => "<>", Tt::Lt => "<", Tt::Gt => ">",
                Tt::Leq => "<=", Tt::Geq => ">=",
                _ => return self.err("CASE IS: Erwartet Vergleichsoperator (=, <>, <, >, <=, >=)"),
            };
            self.pos += 1;
            let expr = self.expression()?;
            return Ok(CaseMatch { kind: "is".into(),
                                  values: vec![CaseVal::Op(op.into()), CaseVal::Expr(expr)] });
        }
        let first = self.expression()?;
        if self.matches(Tt::To) {
            let second = self.expression()?;
            return Ok(CaseMatch { kind: "range".into(),
                                  values: vec![CaseVal::Expr(first), CaseVal::Expr(second)] });
        }
        Ok(CaseMatch { kind: "value".into(), values: vec![CaseVal::Expr(first)] })
    }

    fn inline_statement(&mut self) -> R<Node> {
        match self.tt(0) {
            Tt::Print => {
                self.pos += 1;
                // Review-Fund: fehlte Tt::Colon in dieser Leer-PRINT-Erkennung
                // (print_items() selbst behandelt ':' als Trenner korrekt) --
                // `PRINT : PRINT "x"` (klassisches BASIC fuer eine Leerzeile)
                // schickte das ':' in print_items()->expression() und warf
                // "Unerwartetes Token COLON".
                if self.checks(&[Tt::Newline, Tt::Else, Tt::Colon]) || self.at_end() {
                    return Ok(Node::Print { items: vec![], seps: vec![], newline: true });
                }
                let (items, seps, newline) = self.print_items()?;
                Ok(Node::Print { items, seps, newline })
            }
            Tt::Ident if self.is_assignment_lookahead() => self.assign_from_lvalue(),
            // Review-Fund: `Dot`/`Lparen`-Tupel-Ziele fehlten hier komplett
            // (im Gegensatz zu statement_inner) -- `IF ok THEN .hp = 0` in
            // einem WITH-Block bzw. `IF ok THEN (a, b) = f()` fielen auf den
            // generischen `_`-Zweig durch: `=` ist dort eine Vergleichs-
            // Operation in expression(), die Zuweisung ging also lautlos als
            // verworfener Vergleichswert verloren.
            Tt::Dot if !self.with_stack.is_empty() => self.dot_assign_in_with(),
            Tt::Lparen if self.is_tuple_assign_lookahead() => self.tuple_assign(),
            Tt::Return => {
                self.pos += 1;
                // Review-Fund: fehlte ebenfalls Tt::Colon -- `IF done THEN
                // RETURN : PRINT "x"` warf "Unerwartetes Token COLON", weil
                // RETURN ohne Operand hier nur vor Newline/Else/EOF erkannt
                // wurde (BREAK/CONTINUE direkt darunter brauchen das nicht,
                // die haben nie einen Operanden).
                let value = if !(self.checks(&[Tt::Newline, Tt::Else, Tt::Colon]) || self.at_end()) {
                    Some(Box::new(self.expression()?))
                } else { None };
                Ok(Node::Return(value))
            }
            Tt::Break => { self.pos += 1; Ok(Node::Break) }
            Tt::Continue => { self.pos += 1; Ok(Node::Continue) }
            Tt::Throw => { self.pos += 1; let v = self.expression()?; Ok(Node::Throw { value: Box::new(v) }) }
            _ => {
                let e = self.expression()?;
                // Review-Fund (Sicherheitsnetz): jede Lvalue-Form, die trotz
                // der obigen Zweige noch durchrutscht (z.B. ein Membername,
                // der wie ein Keyword lexed, als Ziel EINER einzelnen
                // Zuweisung ohne with_stack), soll nicht lautlos als
                // verworfener Vergleichswert enden -- kein gueltiges
                // GB-Programm berechnet einen Top-Level-Vergleich und
                // verwirft ihn.
                if let Node::BinaryOp { op, .. } = &e {
                    if op.as_str() == "=" {
                        return self.err("'=' als Anweisung -- meintest du eine Zuweisung?");
                    }
                }
                Ok(Node::ExprStmt { expr: Box::new(e) })
            }
        }
    }

    // ------------------------------------------------------ Loops
    fn while_stmt(&mut self) -> R<Node> {
        self.expect(Tt::While, "")?;
        let cond = self.expression()?;
        self.consume_terminator()?;
        let body = self.block_until(&[Tt::Wend], "WEND erwartet, Programmende erreicht")?;
        self.expect(Tt::Wend, "")?;
        self.consume_terminator()?;
        Ok(Node::While { condition: Box::new(cond), body })
    }

    fn repeat(&mut self) -> R<Node> {
        self.expect(Tt::Repeat, "")?;
        self.consume_terminator()?;
        let body = self.block_until(&[Tt::Until], "UNTIL erwartet, Programmende erreicht")?;
        self.expect(Tt::Until, "")?;
        let cond = self.expression()?;
        self.consume_terminator()?;
        Ok(Node::Repeat { body, condition: Box::new(cond) })
    }

    fn data(&mut self) -> R<Node> {
        self.expect(Tt::Data, "")?;
        let mut values = vec![self.data_literal()?];
        while self.matches(Tt::Comma) { values.push(self.data_literal()?); }
        self.consume_terminator()?;
        Ok(Node::Data { values })
    }

    fn data_literal(&mut self) -> R<Node> {
        let tok = self.peek(0).clone();
        match tok.tt {
            Tt::Number => { self.pos += 1; Ok(Node::NumberLit(num_of(&tok))) }
            Tt::Str => { self.pos += 1; Ok(Node::StringLit(sval(&tok))) }
            Tt::True => { self.pos += 1; Ok(Node::BoolLit(true)) }
            Tt::False => { self.pos += 1; Ok(Node::BoolLit(false)) }
            Tt::Minus => {
                self.pos += 1;
                let inner = self.peek(0).clone();
                if inner.tt != Tt::Number { return self.err("DATA: nach Minus erwartet eine Zahl"); }
                self.pos += 1;
                Ok(Node::UnaryOp { op: "-".into(), operand: Box::new(Node::NumberLit(num_of(&inner))) })
            }
            _ => self.err("DATA: Erwartet Literal (Zahl, String, TRUE, FALSE)"),
        }
    }

    fn read(&mut self) -> R<Node> {
        self.expect(Tt::Read, "")?;
        let mut targets = vec![self.read_target()?];
        while self.matches(Tt::Comma) { targets.push(self.read_target()?); }
        self.consume_terminator()?;
        Ok(Node::Read { targets })
    }

    fn read_target(&mut self) -> R<Node> {
        if self.tt(0) != Tt::Ident { return self.err("READ: Erwartet Variable als Ziel"); }
        self.postfix()
    }

    fn restore(&mut self) -> R<Node> {
        self.expect(Tt::Restore, "")?;
        self.consume_terminator()?;
        Ok(Node::Restore)
    }

    fn for_stmt(&mut self) -> R<Node> {
        self.expect(Tt::For, "")?;
        if self.check(Tt::Ident) && sval(self.peek(0)) == "each" && self.tt(1) == Tt::Ident {
            self.pos += 1; // 'each'
            let var = sval(&self.expect(Tt::Ident, "Erwartet Iterationsvariable nach FOR EACH")?);
            self.expect(Tt::In, "Erwartet IN nach FOR EACH <var>")?;
            let iterable = self.expression()?;
            self.consume_terminator()?;
            let body = self.block_until(&[Tt::Next], "NEXT erwartet, Programmende erreicht")?;
            self.expect(Tt::Next, "")?;
            if self.check(Tt::Ident) { self.pos += 1; }
            self.consume_terminator()?;
            return Ok(Node::ForEach { var, iterable: Box::new(iterable), body });
        }
        let var = sval(&self.expect(Tt::Ident, "Erwartet Schleifenvariable")?);
        self.expect(Tt::Eq, "")?;
        let start = self.expression()?;
        self.expect(Tt::To, "Erwartet TO")?;
        let end = self.expression()?;
        let step = if self.matches(Tt::Step) { Some(Box::new(self.expression()?)) } else { None };
        self.consume_terminator()?;
        let body = self.block_until(&[Tt::Next], "NEXT erwartet, Programmende erreicht")?;
        self.expect(Tt::Next, "")?;
        if self.check(Tt::Ident) { self.pos += 1; }
        self.consume_terminator()?;
        Ok(Node::For { var, start: Box::new(start), end: Box::new(end), step, body })
    }

    // ------------------------------------------------------ Params
    fn params(&mut self) -> R<Vec<Param>> {
        self.expect(Tt::Lparen, "")?;
        let mut params = Vec::new();
        if !self.check(Tt::Rparen) {
            let p = self.param()?;
            let mut seen_variadic = p.is_variadic;
            let mut seen_default = p.default.is_some();
            params.push(p);
            while self.matches(Tt::Comma) {
                if seen_variadic {
                    return self.err("Variadic-Parameter (`...args`) muss der letzte Parameter sein");
                }
                let p = self.param()?;
                if seen_default && p.default.is_none() && !p.is_variadic {
                    return self.err("Parameter ohne Default folgt auf einen Parameter mit Default");
                }
                if p.default.is_some() { seen_default = true; }
                if p.is_variadic { seen_variadic = true; }
                params.push(p);
            }
        }
        self.expect(Tt::Rparen, "")?;
        Ok(params)
    }

    fn param(&mut self) -> R<Param> {
        if self.matches(Tt::Ellipsis) {
            let name = sval(&self.expect(Tt::Ident, "Erwartet Variadic-Parametername nach '...'")?);
            return Ok(Param { name, type_name: "tuple".into(), default: None,
                              by_ref: false, is_variadic: true });
        }
        let by_ref = self.matches(Tt::Byref);
        let name = sval(&self.expect(Tt::Ident, "Erwartet Parametername")?);
        self.expect(Tt::As, "Erwartet AS nach Parametername")?;
        let type_name = self.parse_type()?;
        let mut default = None;
        if self.matches(Tt::Eq) {
            if by_ref { return self.err("BYREF-Parameter koennen keinen Default-Wert haben"); }
            default = Some(self.expression()?);
        }
        Ok(Param { name, type_name, default, by_ref, is_variadic: false })
    }

    /// Sammelt Statements, bis eines der `terminators`-Tokens erreicht ist
    /// (oder EOF -> Fehler `what`). Ersetzt die frueher ~13x fast identisch
    /// duplizierte "while !check(...) { if at_end() {err} push(statement()) }"
    /// Schleife (WITH/TRY/IF/SELECT-CASE/WHILE/REPEAT/FOR/FOR EACH/SUB/
    /// FUNCTION/OPERATOR/PROPERTY). Konsumiert den Terminator NICHT selbst --
    /// der Aufrufer prueft/erwartet ihn danach.
    fn block_until(&mut self, terminators: &[Tt], what: &str) -> R<Vec<Node>> {
        let mut body = Vec::new();
        while !self.checks(terminators) {
            if self.at_end() { return self.err(what); }
            body.push(self.statement()?);
        }
        Ok(body)
    }
    fn block_until_end(&mut self, what: &str) -> R<Vec<Node>> {
        self.block_until(&[Tt::End], what)
    }

    fn sub_decl(&mut self) -> R<Node> {
        self.expect(Tt::Sub, "")?;
        let name = sval(&self.expect(Tt::Ident, "Erwartet SUB-Name")?);
        let params = self.params()?;
        self.consume_terminator()?;
        let body = self.block_until_end("END SUB erwartet, Programmende erreicht")?;
        self.expect(Tt::End, "")?;
        self.expect(Tt::Sub, "Erwartet SUB nach END")?;
        self.consume_terminator()?;
        Ok(Node::SubDecl { name, params, body })
    }

    fn function_decl(&mut self) -> R<Node> {
        self.expect(Tt::Function, "")?;
        let name = sval(&self.expect(Tt::Ident, "Erwartet FUNCTION-Name")?);
        let params = self.params()?;
        self.expect(Tt::As, "Erwartet AS <Rueckgabetyp> nach Parameterliste")?;
        let return_type = self.parse_type()?;
        self.consume_terminator()?;
        let body = self.block_until_end("END FUNCTION erwartet, Programmende erreicht")?;
        self.expect(Tt::End, "")?;
        self.expect(Tt::Function, "Erwartet FUNCTION nach END")?;
        self.consume_terminator()?;
        Ok(Node::FunctionDecl { name, params, return_type, body })
    }

    fn operator_decl(&mut self) -> R<Node> {
        self.expect(Tt::Operator, "")?;
        let op_name = match operator_name(self.tt(0)) {
            Some(n) => n,
            None => return self.err("OPERATOR: erwartet einen der Operatoren + - * / MOD = <> < > <= >="),
        };
        self.pos += 1;
        let params = self.params()?;
        if params.len() != 1 { return self.err("OPERATOR: erwartet genau 1 Parameter (other)"); }
        if params[0].by_ref { return self.err("OPERATOR: Parameter darf nicht BYREF sein"); }
        if params[0].is_variadic { return self.err("OPERATOR: Parameter darf nicht variadic sein"); }
        self.expect(Tt::As, "Erwartet AS <Rueckgabetyp> nach OPERATOR-Parameter")?;
        let return_type = self.parse_type()?;
        self.consume_terminator()?;
        let body = self.block_until_end("END OPERATOR erwartet, Programmende erreicht")?;
        self.expect(Tt::End, "")?;
        self.expect(Tt::Operator, "Erwartet OPERATOR nach END")?;
        self.consume_terminator()?;
        Ok(Node::FunctionDecl { name: op_name.into(), params, return_type, body })
    }

    fn return_stmt(&mut self) -> R<Node> {
        self.expect(Tt::Return, "")?;
        // Review-Fund: fehlte Tt::Colon -- `x = 1 : RETURN : y = 2` warf
        // "Unerwartetes Token COLON", weil das ':' als Start eines
        // (nicht-existenten) Rueckgabewert-Ausdrucks interpretiert wurde.
        // BREAK/CONTINUE brauchen diese Pruefung nicht (nie ein Operand).
        let value = if !self.check(Tt::Newline) && !self.check(Tt::Colon) && !self.at_end() {
            Some(Box::new(self.expression()?))
        } else { None };
        self.consume_terminator()?;
        Ok(Node::Return(value))
    }

    // ------------------------------------------------------ CLASS
    fn property_decl(&mut self) -> R<(Node, Node)> {
        self.expect(Tt::Property, "")?;
        let kind_tok = self.peek(0).clone();
        let kind = sval(&kind_tok);
        if kind_tok.tt != Tt::Ident || (kind != "get" && kind != "set") {
            return self.err("Erwartet GET oder SET nach PROPERTY");
        }
        self.pos += 1;
        let prop_name = sval(&self.expect(Tt::Ident, "Erwartet Property-Name")?);
        let internal = format!("__{}_{}", kind, prop_name.to_lowercase());
        let params = self.params()?;
        let func = if kind == "get" {
            self.expect(Tt::As, "Erwartet AS <Rueckgabetyp> nach PROPERTY GET-Parametern")?;
            let return_type = self.parse_type()?;
            self.consume_terminator()?;
            let body = self.block_until_end("END PROPERTY erwartet, Programmende erreicht")?;
            self.expect(Tt::End, "")?;
            self.expect(Tt::Property, "Erwartet PROPERTY nach END")?;
            self.consume_terminator()?;
            if !params.is_empty() { return self.err("PROPERTY GET nimmt keine Parameter"); }
            Node::FunctionDecl { name: internal, params, return_type, body }
        } else {
            self.consume_terminator()?;
            let body = self.block_until_end("END PROPERTY erwartet, Programmende erreicht")?;
            self.expect(Tt::End, "")?;
            self.expect(Tt::Property, "Erwartet PROPERTY nach END")?;
            self.consume_terminator()?;
            if params.len() != 1 {
                return self.err("PROPERTY SET muss genau einen Parameter haben (den neuen Wert)");
            }
            Node::SubDecl { name: internal, params, body }
        };
        let pd = Node::PropertyDecl { name: prop_name, kind, func: Box::new(func.clone()) };
        Ok((pd, func))
    }

    fn class_decl(&mut self) -> R<Node> {
        self.expect(Tt::Class, "")?;
        let name = sval(&self.expect(Tt::Ident, "Erwartet Klassenname nach CLASS")?);
        let mut parent = None;
        if self.matches(Tt::Extends) {
            parent = Some(sval(&self.expect(Tt::Ident, "Erwartet Elternklassen-Name nach EXTENDS")?));
        }
        self.consume_terminator()?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut statics = Vec::new();
        let mut properties = Vec::new();
        while !self.check(Tt::End) {
            if self.at_end() { return self.err("END CLASS erwartet, Programmende erreicht"); }
            if self.check(Tt::Newline) { self.pos += 1; continue; }
            if self.check(Tt::Static) {
                self.pos += 1;
                if !self.check(Tt::Const) { return self.err("Erwartet CONST nach STATIC im CLASS-Body"); }
                statics.push(self.const_stmt()?);
            } else if self.check(Tt::Property) {
                let (pd, internal) = self.property_decl()?;
                properties.push(pd);
                methods.push(internal);
            } else if self.check(Tt::Dim) {
                fields.push(self.dim()?);
            } else if self.check(Tt::Sub) {
                methods.push(self.sub_decl()?);
            } else if self.check(Tt::Function) {
                methods.push(self.function_decl()?);
            } else if self.check(Tt::Operator) {
                methods.push(self.operator_decl()?);
            } else {
                return self.err("Unerwartet im CLASS-Body (erlaubt: DIM, SUB, FUNCTION, OPERATOR, STATIC CONST, PROPERTY)");
            }
        }
        self.expect(Tt::End, "")?;
        self.expect(Tt::Class, "Erwartet CLASS nach END")?;
        self.consume_terminator()?;
        Ok(Node::ClassDecl { name, parent, fields, methods, is_struct: false, statics, properties })
    }

    fn struct_decl(&mut self) -> R<Node> {
        self.expect(Tt::Struct, "")?;
        let name = sval(&self.expect(Tt::Ident, "Erwartet Name nach STRUCT")?);
        self.consume_terminator()?;
        let mut fields = Vec::new();
        while !self.check(Tt::End) {
            if self.at_end() { return self.err("END STRUCT erwartet, Programmende erreicht"); }
            if self.check(Tt::Newline) { self.pos += 1; continue; }
            if self.check(Tt::Dim) { fields.push(self.dim()?); }
            else { return self.err("Im STRUCT-Body sind nur DIM-Felder erlaubt"); }
        }
        self.expect(Tt::End, "")?;
        self.expect(Tt::Struct, "Erwartet STRUCT nach END")?;
        self.consume_terminator()?;
        Ok(Node::ClassDecl { name, parent: None, fields, methods: vec![],
                             is_struct: true, statics: vec![], properties: vec![] })
    }

    fn new_expr(&mut self) -> R<Node> {
        self.expect(Tt::New, "")?;
        let name = sval(&self.expect(Tt::Ident, "Erwartet Klassenname nach NEW")?);
        let mut args = None;
        if self.matches(Tt::Lparen) {
            args = Some(self.call_args()?);
            self.expect(Tt::Rparen, "")?;
        }
        Ok(Node::New { class_name: name, args })
    }

    // ------------------------------------------------------ Ausdruecke
    fn expression(&mut self) -> R<Node> {
        if self.check(Tt::Yield) { return self.yield_expr(); }
        self.or_expr()
    }

    fn yield_expr(&mut self) -> R<Node> {
        self.expect(Tt::Yield, "")?;
        if self.at_end() || self.checks(&[Tt::Newline, Tt::Eof, Tt::Colon, Tt::Rparen,
                                          Tt::Rbracket, Tt::Rbrace, Tt::Comma, Tt::Else]) {
            return Ok(Node::Yield(None));
        }
        Ok(Node::Yield(Some(Box::new(self.expression()?))))
    }

    fn or_expr(&mut self) -> R<Node> {
        let mut left = self.and_expr()?;
        while self.matches(Tt::Or) {
            let right = self.and_expr()?;
            left = Node::BinaryOp { op: "or".into(), left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn and_expr(&mut self) -> R<Node> {
        let mut left = self.not_expr()?;
        while self.matches(Tt::And) {
            let right = self.not_expr()?;
            left = Node::BinaryOp { op: "and".into(), left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn not_expr(&mut self) -> R<Node> {
        if self.matches(Tt::Not) {
            return Ok(Node::UnaryOp { op: "not".into(), operand: Box::new(self.not_expr()?) });
        }
        self.comparison()
    }

    fn comparison(&mut self) -> R<Node> {
        let mut left = self.bitwise()?;
        while self.checks(&[Tt::Eq, Tt::Neq, Tt::Lt, Tt::Gt, Tt::Leq, Tt::Geq, Tt::In]) {
            let op = match self.tt(0) {
                Tt::Eq => "=", Tt::Neq => "<>", Tt::Lt => "<", Tt::Gt => ">",
                Tt::Leq => "<=", Tt::Geq => ">=", Tt::In => "in", _ => unreachable!(),
            };
            self.pos += 1;
            let right = self.bitwise()?;
            left = Node::BinaryOp { op: op.into(), left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn bitwise(&mut self) -> R<Node> {
        let mut left = self.addition()?;
        while self.checks(&[Tt::Band, Tt::Bor, Tt::Bxor, Tt::Shl, Tt::Shr]) {
            let op = match self.tt(0) {
                Tt::Band => "band", Tt::Bor => "bor", Tt::Bxor => "bxor",
                Tt::Shl => "shl", Tt::Shr => "shr", _ => unreachable!(),
            };
            self.pos += 1;
            let right = self.addition()?;
            left = Node::BinaryOp { op: op.into(), left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn addition(&mut self) -> R<Node> {
        let mut left = self.multiplication()?;
        while self.checks(&[Tt::Plus, Tt::Minus]) {
            let op = if self.tt(0) == Tt::Plus { "+" } else { "-" };
            self.pos += 1;
            let right = self.multiplication()?;
            left = Node::BinaryOp { op: op.into(), left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn multiplication(&mut self) -> R<Node> {
        let mut left = self.unary()?;
        while self.checks(&[Tt::StarT, Tt::Slash, Tt::Mod, Tt::Intdiv]) {
            let op = match self.tt(0) {
                Tt::StarT => "*", Tt::Slash => "/", Tt::Mod => "mod", Tt::Intdiv => "\\",
                _ => unreachable!(),
            };
            self.pos += 1;
            let right = self.unary()?;
            left = Node::BinaryOp { op: op.into(), left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn unary(&mut self) -> R<Node> {
        if self.matches(Tt::Minus) {
            return Ok(Node::UnaryOp { op: "-".into(), operand: Box::new(self.unary()?) });
        }
        if self.matches(Tt::Plus) { return self.unary(); }
        if self.matches(Tt::Bnot) {
            return Ok(Node::UnaryOp { op: "bnot".into(), operand: Box::new(self.unary()?) });
        }
        self.power()
    }

    fn power(&mut self) -> R<Node> {
        let left = self.postfix()?;
        if self.matches(Tt::Caret) {
            let right = self.unary()?;
            return Ok(Node::BinaryOp { op: "^".into(), left: Box::new(left), right: Box::new(right) });
        }
        Ok(left)
    }

    fn postfix(&mut self) -> R<Node> {
        let mut expr = self.primary()?;
        loop {
            if self.matches(Tt::Lparen) {
                let args = self.call_args()?;
                self.expect(Tt::Rparen, "")?;
                expr = Node::Call { callee: Box::new(expr), args };
            } else if self.matches(Tt::Dot) {
                let m = self.peek(0).clone();
                if let Val::Str(s) = &m.val {
                    if !s.is_empty() {
                        self.pos += 1;
                        expr = Node::MemberAccess { target: Box::new(expr), name: s.clone() };
                        continue;
                    }
                }
                return self.err("Erwartet Membername nach '.'");
            } else if self.matches(Tt::Lbracket) {
                expr = self.index_or_slice(expr)?;
            } else { break; }
        }
        Ok(expr)
    }

    fn index_or_slice(&mut self, target: Node) -> R<Node> {
        if self.check(Tt::Colon) {
            self.pos += 1;
            let hi = if !self.check(Tt::Rbracket) { Some(Box::new(self.expression()?)) } else { None };
            self.expect(Tt::Rbracket, "Erwartet ']'")?;
            return Ok(Node::SliceAccess { target: Box::new(target), lo: None, hi });
        }
        let first = self.expression()?;
        if self.matches(Tt::Colon) {
            let hi = if !self.check(Tt::Rbracket) { Some(Box::new(self.expression()?)) } else { None };
            self.expect(Tt::Rbracket, "Erwartet ']'")?;
            return Ok(Node::SliceAccess { target: Box::new(target),
                                          lo: Some(Box::new(first)), hi });
        }
        let mut indices = vec![first];
        while self.matches(Tt::Comma) { indices.push(self.expression()?); }
        self.expect(Tt::Rbracket, "Erwartet ']'")?;
        Ok(Node::IndexAccess { target: Box::new(target), indices })
    }

    fn primary(&mut self) -> R<Node> {
        let tok = self.peek(0).clone();
        let t = tok.tt;
        if t == Tt::Dot && !self.with_stack.is_empty() {
            self.pos += 1;
            let m = self.peek(0).clone();
            match &m.val {
                Val::Str(s) if !s.is_empty() => {
                    self.pos += 1;
                    return Ok(Node::MemberAccess {
                        target: Box::new(Node::Identifier(self.with_stack.last().unwrap().clone())),
                        name: s.clone() });
                }
                _ => return self.err("Erwartet Membername nach '.' im WITH-Block"),
            }
        }
        match t {
            Tt::Number => { self.pos += 1; Ok(Node::NumberLit(num_of(&tok))) }
            Tt::Str => { self.pos += 1; Ok(Node::StringLit(sval(&tok))) }
            Tt::True => { self.pos += 1; Ok(Node::BoolLit(true)) }
            Tt::False => { self.pos += 1; Ok(Node::BoolLit(false)) }
            Tt::Nil => { self.pos += 1; Ok(Node::NilLit) }
            Tt::Ident => {
                if sval(&tok) == "iif" && self.tt(1) == Tt::Lparen {
                    self.pos += 2;
                    let cond = self.expression()?;
                    self.expect(Tt::Comma, "Erwartet ',' nach IIF-Bedingung")?;
                    let then_e = self.expression()?;
                    self.expect(Tt::Comma, "Erwartet ',' nach IIF-Then-Wert")?;
                    let else_e = self.expression()?;
                    self.expect(Tt::Rparen, "Erwartet ')' am Ende von IIF")?;
                    return Ok(Node::TernaryExpr { cond: Box::new(cond),
                        then_expr: Box::new(then_e), else_expr: Box::new(else_e) });
                }
                self.pos += 1;
                Ok(Node::Identifier(sval(&tok)))
            }
            Tt::New => self.new_expr(),
            Tt::Lbracket => {
                self.pos += 1;
                // Leeres Array-Literal []: Typ nicht herleitbar -> klarer Hinweis.
                if self.tt(0) == Tt::Rbracket {
                    return self.err("Leeres Array-Literal [] -- Typ unbekannt; nutze DIM ... AS ARRAY OF T");
                }
                let first = self.expression()?;
                // Disambiguierung: `[expr FOR ...]` = List-Comprehension,
                // sonst `[a, b, c]` = Array-Literal.
                if self.matches(Tt::For) {
                    let var = sval(&self.expect(Tt::Ident, "Erwartet Iter-Variablenname nach FOR")?);
                    self.expect(Tt::In, "Erwartet IN nach Iter-Variable")?;
                    let iterable = self.expression()?;
                    let filter = if self.matches(Tt::Where) { Some(Box::new(self.expression()?)) } else { None };
                    self.expect(Tt::Rbracket, "Erwartet ']' am Ende der Comprehension")?;
                    return Ok(Node::ListComp { var, iterable: Box::new(iterable), filter,
                                               transform: Box::new(first) });
                }
                let mut elements = vec![first];
                while self.matches(Tt::Comma) {
                    if self.tt(0) == Tt::Rbracket { break; }   // optionales Trailing-Komma
                    elements.push(self.expression()?);
                }
                self.expect(Tt::Rbracket, "Erwartet ']' am Ende des Array-Literals")?;
                Ok(Node::ArrayLit(elements))
            }
            Tt::Lbrace => {
                self.pos += 1;
                let first = self.expression()?;
                if self.matches(Tt::Colon) {
                    let value = self.expression()?;
                    if !self.matches(Tt::For) {
                        return self.err("Erwartet FOR in Dict-Comprehension `{key: val FOR var IN ...}`");
                    }
                    let var = sval(&self.expect(Tt::Ident, "Erwartet Iter-Variablenname nach FOR")?);
                    self.expect(Tt::In, "Erwartet IN nach Iter-Variable")?;
                    let iterable = self.expression()?;
                    let filter = if self.matches(Tt::Where) { Some(Box::new(self.expression()?)) } else { None };
                    self.expect(Tt::Rbrace, "Erwartet '}' am Ende der Dict-Comprehension")?;
                    return Ok(Node::DictComp { var, iterable: Box::new(iterable), filter,
                        key: Box::new(first), value: Box::new(value) });
                }
                if !self.matches(Tt::For) {
                    return self.err("Erwartet ':' (Dict-Comp) oder FOR (Set-Comp) im `{...}`-Block");
                }
                let var = sval(&self.expect(Tt::Ident, "Erwartet Iter-Variablenname nach FOR")?);
                self.expect(Tt::In, "Erwartet IN nach Iter-Variable")?;
                let iterable = self.expression()?;
                let filter = if self.matches(Tt::Where) { Some(Box::new(self.expression()?)) } else { None };
                self.expect(Tt::Rbrace, "Erwartet '}' am Ende der Set-Comprehension")?;
                Ok(Node::SetComp { var, iterable: Box::new(iterable), filter,
                                   transform: Box::new(first) })
            }
            Tt::Lparen => {
                self.pos += 1;
                let e = self.expression()?;
                if self.check(Tt::Comma) {
                    let mut elements = vec![e];
                    while self.matches(Tt::Comma) { elements.push(self.expression()?); }
                    self.expect(Tt::Rparen, "")?;
                    return Ok(Node::TupleLit { elements });
                }
                self.expect(Tt::Rparen, "")?;
                Ok(e)
            }
            _ => self.err(&format!("Unerwartetes Token {}", t.name())),
        }
    }
}

/// NumberLit-Wert aus einem NUMBER-Token.
fn num_of(t: &Token) -> NumV {
    match &t.val {
        Val::Int(i) => NumV::Int(*i),
        Val::Float(f) => NumV::Float(*f),
        _ => NumV::Int(0),
    }
}

/// AST als kanonisches JSON (`dhrt --ast`). Lext + parst die Quelle.
/// Parst einen einzelnen Ausdruck (fuer Debugger-Bedingungen + `eval`, Stufe B).
/// Lext die Quelle und parst genau eine Expression.
pub fn parse_expression(source: &str) -> Result<Node, String> {
    let toks = match crate::lexer::Lexer::new(source).tokenize() {
        Ok(t) => t,
        Err(e) => return Err(format!("Lexer {}:{}: {}", e.line, e.col, e.msg)),
    };
    let mut p = Parser::new(toks);
    p.expression().map_err(|e| format!("{}:{}: {}", e.line, e.col, e.msg))
}

pub fn dump_ast_json(source: &str) -> Result<String, String> {
    let toks = match crate::lexer::Lexer::new(source).tokenize() {
        Ok(t) => t,
        Err(e) => return Err(format!("Lexer {}:{}: {}", e.line, e.col, e.msg)),
    };
    let mut p = Parser::new(toks);
    match p.parse() {
        Ok(ast) => Ok(serde_json::to_string(&ast.to_json()).unwrap()),
        Err(e) => Err(format!("Parse {}:{}: {}", e.line, e.col, e.msg)),
    }
}
