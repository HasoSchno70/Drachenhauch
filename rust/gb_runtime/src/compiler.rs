//! Compiler AST -> Bytecode (`.gbc`-JSON) -- Stufe 3 der Front-End-Portierung.
//!
//! **Stufe 3a:** main-only Konsolen-Programme (Skalar-Globals, Arithmetik/
//! Vergleich/Logik/Bitwise/Unär, PRINT, Builtin-Calls, IF/WHILE/BREAK/
//! CONTINUE, CONST). Emittiert die *generischen* Opcodes (kein Constant-
//! Folding, keine `_NN`-Spezialisierung, keine Inline-Caches) -- gbrt's VM
//! unterstützt beide, das Verhalten ist identisch. Verifiziert per Output-
//! Parität: `gbrt --runsrc` == Python-Tree-Walker (tests/test_rust_compiler_parity.py).
//!
//! Nicht-3a-Konstrukte (Funktionen, Klassen, Arrays, Comprehensions, SELECT,
//! Tupel, WITH, TRY, FOR, DATA, ...) liefern `Err("Stufe 3b: ...")` -- der
//! Parity-Test überspringt solche Programme. Spätere Stufen ergänzen sie.

use std::collections::HashMap;

use serde_json::{json, Value};

use crate::ast::{NumV, Node};

// Opcodes (Teilmenge; Werte aus bytecode.py / model::op).
mod oc {
    pub const LOAD_CONST: i64 = 1;
    pub const POP: i64 = 2;
    pub const DUP: i64 = 3;
    pub const LOAD_NAME: i64 = 10;
    pub const STORE_NAME: i64 = 11;
    pub const DECLARE_CONST: i64 = 13;
    pub const ADD: i64 = 20;
    pub const SUB: i64 = 21;
    pub const MUL: i64 = 22;
    pub const DIV: i64 = 23;
    pub const MOD: i64 = 24;
    pub const POW: i64 = 25;
    pub const NEG: i64 = 26;
    pub const INT_DIV: i64 = 27;
    pub const EQ: i64 = 30;
    pub const NEQ: i64 = 31;
    pub const LT: i64 = 32;
    pub const GT: i64 = 33;
    pub const LEQ: i64 = 34;
    pub const GEQ: i64 = 35;
    pub const NOT: i64 = 36;
    pub const JUMP: i64 = 40;
    pub const JUMP_IF_FALSE: i64 = 41;
    pub const JUMP_IF_TRUE: i64 = 42;
    pub const CALL_BUILTIN: i64 = 51;
    pub const CALL_VALUE: i64 = 54;
    pub const BAND: i64 = 62;
    pub const BOR: i64 = 63;
    pub const BXOR: i64 = 64;
    pub const SHL: i64 = 65;
    pub const SHR: i64 = 66;
    pub const BNOT: i64 = 67;
    pub const IN_OP: i64 = 56;
    pub const PRINT: i64 = 70;
    pub const LOAD_GLOBAL_SLOT: i64 = 111;
    pub const STORE_GLOBAL_SLOT: i64 = 112;
    pub const DECLARE_GLOBAL_SLOT: i64 = 113;
    pub const DECLARE_GLOBAL_CONST_SLOT: i64 = 114;
    pub const HALT: i64 = 99;
}

/// Konstanter Wert im Pool -- wird wie `serialize._enc` kodiert.
#[derive(Clone)]
enum CVal { Nil, Bool(bool), Int(i64), Float(f64), Str(String) }

fn enc(c: &CVal) -> Value {
    match c {
        CVal::Nil => Value::Null,
        CVal::Bool(b) => json!({ "b": b }),
        CVal::Int(i) => json!(i),
        CVal::Float(f) => json!({ "f": f }),
        CVal::Str(s) => json!(s),
    }
}

fn type_default(t: &str) -> CVal {
    match t {
        "integer" => CVal::Int(0),
        "float" => CVal::Float(0.0),
        "string" => CVal::Str(String::new()),
        "boolean" => CVal::Bool(false),
        _ => CVal::Nil,        // Klassen/externe Typen
    }
}

type CR = Result<(), String>;

struct Ctx {
    code: Vec<(i64, Value)>,
    consts: Vec<Value>,
    break_patches: Vec<Vec<usize>>,
    continue_patches: Vec<Vec<usize>>,
}

impl Ctx {
    fn new() -> Self {
        Ctx { code: vec![], consts: vec![], break_patches: vec![], continue_patches: vec![] }
    }
    fn add_const(&mut self, v: Value) -> i64 {
        if let Some(i) = self.consts.iter().position(|c| *c == v) {
            return i as i64;
        }
        self.consts.push(v);
        (self.consts.len() - 1) as i64
    }
    fn emit(&mut self, op: i64, arg: Value) -> usize {
        let ip = self.code.len();
        self.code.push((op, arg));
        ip
    }
    fn here(&self) -> usize { self.code.len() }
    fn patch(&mut self, ip: usize, target: usize) {
        self.code[ip].1 = json!(target);
    }
}

pub struct Compiler {
    global_slots: HashMap<String, usize>,
    global_vars: std::collections::HashSet<String>,
    ctx: Ctx,
}

impl Compiler {
    fn new() -> Self {
        Compiler { global_slots: HashMap::new(),
                   global_vars: std::collections::HashSet::new(), ctx: Ctx::new() }
    }

    fn alloc_slot(&mut self, name: &str) {
        let n = self.global_slots.len();
        self.global_slots.entry(name.to_string()).or_insert(n);
    }

    /// Pre-Pass: Top-Level-Globals (Skalar-DIM/CONST) -> Slot-Index.
    fn collect_globals(&mut self, stmts: &[Node]) -> CR {
        for s in stmts {
            match s {
                Node::Dim { name, type_name, array_dims } => {
                    self.global_vars.insert(name.clone());
                    if array_dims.is_none() && is_simple_type(type_name) {
                        self.alloc_slot(name);
                    }
                }
                Node::MultiDim { dims } => {
                    for d in dims {
                        if let Node::Dim { name, type_name, array_dims } = d {
                            self.global_vars.insert(name.clone());
                            if array_dims.is_none() && is_simple_type(type_name) {
                                self.alloc_slot(name);
                            }
                        }
                    }
                }
                Node::Const { name, .. } => {
                    self.global_vars.insert(name.clone());
                    self.alloc_slot(name);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn stmt(&mut self, s: &Node) -> CR {
        match s {
            Node::Dim { name, type_name, array_dims } =>
                self.stmt_dim(name, type_name, array_dims),
            Node::MultiDim { dims } => {
                for d in dims { self.stmt(d)?; }
                Ok(())
            }
            Node::Const { name, type_name, value } =>
                self.stmt_const(name, type_name.as_deref(), value),
            Node::Assign { name, value } => {
                self.expr(value)?;
                self.store_var(name);
                Ok(())
            }
            Node::Print { items, .. } => {
                for it in items { self.expr(it)?; }
                self.ctx.emit(oc::PRINT, json!(items.len()));
                Ok(())
            }
            Node::ExprStmt { expr } => {
                self.expr(expr)?;
                self.ctx.emit(oc::POP, Value::Null);
                Ok(())
            }
            Node::If { condition, then_block, elseif_branches, else_block } =>
                self.stmt_if(condition, then_block, elseif_branches, else_block),
            Node::While { condition, body } => self.stmt_while(condition, body),
            Node::Break => self.emit_break(),
            Node::Continue => self.emit_continue(),
            other => Err(format!("Stufe 3b: Statement {} noch nicht unterstuetzt",
                                 node_name(other))),
        }
    }

    fn stmt_dim(&mut self, name: &str, type_name: &str, array_dims: &Option<Vec<Node>>) -> CR {
        if array_dims.is_some() {
            return Err("Stufe 3b: Array-DIM noch nicht unterstuetzt".into());
        }
        if !is_simple_type(type_name) {
            return Err(format!("Stufe 3b: DIM-Typ '{}' noch nicht unterstuetzt", type_name));
        }
        let name_idx = self.ctx.add_const(json!(name));
        let type_idx = self.ctx.add_const(json!(type_name));
        let default_idx = self.ctx.add_const(enc(&type_default(type_name)));
        let slot = self.global_slots[name] as i64;
        self.ctx.emit(oc::DECLARE_GLOBAL_SLOT,
                      json!([slot, name_idx, type_idx, default_idx]));
        Ok(())
    }

    fn stmt_const(&mut self, name: &str, type_name: Option<&str>, value: &Node) -> CR {
        self.expr(value)?;
        let name_idx = self.ctx.add_const(json!(name));
        let type_idx = match type_name {
            Some(t) => self.ctx.add_const(json!(t)),
            None => self.ctx.add_const(Value::Null),
        };
        let slot = self.global_slots[name] as i64;
        self.ctx.emit(oc::DECLARE_GLOBAL_CONST_SLOT, json!([slot, name_idx, type_idx]));
        Ok(())
    }

    fn stmt_if(&mut self, cond: &Node, then_block: &[Node],
               elseifs: &[(Node, Vec<Node>)], else_block: &[Node]) -> CR {
        let mut end_jumps: Vec<usize> = vec![];
        self.expr(cond)?;
        let mut false_jump = self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null);
        for st in then_block { self.stmt(st)?; }
        end_jumps.push(self.ctx.emit(oc::JUMP, Value::Null));
        for (ec, block) in elseifs {
            let tgt = self.ctx.here();
            self.ctx.patch(false_jump, tgt);
            self.expr(ec)?;
            false_jump = self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null);
            for st in block { self.stmt(st)?; }
            end_jumps.push(self.ctx.emit(oc::JUMP, Value::Null));
        }
        let else_tgt = self.ctx.here();
        self.ctx.patch(false_jump, else_tgt);
        for st in else_block { self.stmt(st)?; }
        let end = self.ctx.here();
        for j in end_jumps { self.ctx.patch(j, end); }
        Ok(())
    }

    fn stmt_while(&mut self, cond: &Node, body: &[Node]) -> CR {
        let start = self.ctx.here();
        self.expr(cond)?;
        let exit = self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null);
        self.break_continue_enter();
        for st in body { self.stmt(st)?; }
        // CONTINUE springt zum Schleifen-Anfang (Bedingung neu pruefen).
        let cont_patches = self.ctx.continue_patches.pop().unwrap();
        for ip in cont_patches { self.ctx.patch(ip, start); }
        self.ctx.emit(oc::JUMP, json!(start));
        let end = self.ctx.here();
        self.ctx.patch(exit, end);
        let break_patches = self.ctx.break_patches.pop().unwrap();
        for ip in break_patches { self.ctx.patch(ip, end); }
        Ok(())
    }

    fn break_continue_enter(&mut self) {
        self.ctx.break_patches.push(vec![]);
        self.ctx.continue_patches.push(vec![]);
    }
    fn emit_break(&mut self) -> CR {
        if self.ctx.break_patches.is_empty() {
            return Err("BREAK ausserhalb einer Schleife".into());
        }
        let ip = self.ctx.emit(oc::JUMP, Value::Null);
        self.ctx.break_patches.last_mut().unwrap().push(ip);
        Ok(())
    }
    fn emit_continue(&mut self) -> CR {
        if self.ctx.continue_patches.is_empty() {
            return Err("CONTINUE ausserhalb einer Schleife".into());
        }
        let ip = self.ctx.emit(oc::JUMP, Value::Null);
        self.ctx.continue_patches.last_mut().unwrap().push(ip);
        Ok(())
    }

    // ---------------------------------------------------- Variablen
    fn load_var(&mut self, name: &str) {
        if let Some(&slot) = self.global_slots.get(name) {
            self.ctx.emit(oc::LOAD_GLOBAL_SLOT, json!(slot));
        } else {
            let idx = self.ctx.add_const(json!(name));
            self.ctx.emit(oc::LOAD_NAME, json!(idx));
        }
    }
    fn store_var(&mut self, name: &str) {
        if let Some(&slot) = self.global_slots.get(name) {
            self.ctx.emit(oc::STORE_GLOBAL_SLOT, json!(slot));
        } else {
            let idx = self.ctx.add_const(json!(name));
            self.ctx.emit(oc::STORE_NAME, json!(idx));
        }
    }

    // ---------------------------------------------------- Ausdruecke
    fn expr(&mut self, e: &Node) -> CR {
        match e {
            Node::NumberLit(NumV::Int(i)) => {
                let c = self.ctx.add_const(json!(i)); self.ctx.emit(oc::LOAD_CONST, json!(c)); Ok(())
            }
            Node::NumberLit(NumV::Float(f)) => {
                let c = self.ctx.add_const(json!({ "f": f })); self.ctx.emit(oc::LOAD_CONST, json!(c)); Ok(())
            }
            Node::StringLit(s) => {
                let c = self.ctx.add_const(json!(s)); self.ctx.emit(oc::LOAD_CONST, json!(c)); Ok(())
            }
            Node::BoolLit(b) => {
                let c = self.ctx.add_const(json!({ "b": b })); self.ctx.emit(oc::LOAD_CONST, json!(c)); Ok(())
            }
            Node::Identifier(name) => { self.load_var(name); Ok(()) }
            Node::UnaryOp { op, operand } => self.expr_unary(op, operand),
            Node::BinaryOp { op, left, right } => self.expr_binary(op, left, right),
            Node::Call { callee, args } => self.expr_call(callee, args),
            other => Err(format!("Stufe 3b: Ausdruck {} noch nicht unterstuetzt",
                                 node_name(other))),
        }
    }

    fn expr_unary(&mut self, op: &str, operand: &Node) -> CR {
        self.expr(operand)?;
        match op {
            "-" => { self.ctx.emit(oc::NEG, Value::Null); }
            "not" => { self.ctx.emit(oc::NOT, Value::Null); }
            "bnot" => { self.ctx.emit(oc::BNOT, Value::Null); }
            _ => return Err(format!("Unbekannter unaerer Operator: {}", op)),
        }
        Ok(())
    }

    fn expr_binary(&mut self, op: &str, left: &Node, right: &Node) -> CR {
        if op == "and" {
            self.expr(left)?;
            self.ctx.emit(oc::DUP, Value::Null);
            let j = self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null);
            self.ctx.emit(oc::POP, Value::Null);
            self.expr(right)?;
            let t = self.ctx.here(); self.ctx.patch(j, t);
            return Ok(());
        }
        if op == "or" {
            self.expr(left)?;
            self.ctx.emit(oc::DUP, Value::Null);
            let j = self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null);
            self.ctx.emit(oc::POP, Value::Null);
            self.expr(right)?;
            let t = self.ctx.here(); self.ctx.patch(j, t);
            return Ok(());
        }
        self.expr(left)?;
        self.expr(right)?;
        let code = match op {
            "+" => oc::ADD, "-" => oc::SUB, "*" => oc::MUL, "/" => oc::DIV,
            "mod" => oc::MOD, "^" => oc::POW, "\\" => oc::INT_DIV,
            "=" => oc::EQ, "<>" => oc::NEQ, "<" => oc::LT, ">" => oc::GT,
            "<=" => oc::LEQ, ">=" => oc::GEQ,
            "band" => oc::BAND, "bor" => oc::BOR, "bxor" => oc::BXOR,
            "shl" => oc::SHL, "shr" => oc::SHR, "in" => oc::IN_OP,
            _ => return Err(format!("Unbekannter Operator: {}", op)),
        };
        self.ctx.emit(code, Value::Null);
        Ok(())
    }

    fn expr_call(&mut self, callee: &Node, args: &[Node]) -> CR {
        let name = match callee {
            Node::Identifier(n) => n.clone(),
            _ => return Err("Stufe 3b: aufrufbare Werte/Methoden noch nicht unterstuetzt".into()),
        };
        for a in args {
            if matches!(a, Node::NamedArg { .. }) {
                return Err("Stufe 3b: Named-Args noch nicht unterstuetzt".into());
            }
        }
        // 3a kennt keine User-Funktionen -> Identifier-Call ist ein Builtin
        // (sonst: globale FUNCREF-Variable -> CALL_VALUE).
        if self.global_vars.contains(&name) {
            self.load_var(&name);
            for a in args { self.expr(a)?; }
            self.ctx.emit(oc::CALL_VALUE, json!(args.len()));
        } else {
            for a in args { self.expr(a)?; }
            self.ctx.emit(oc::CALL_BUILTIN, json!([name, args.len()]));
        }
        Ok(())
    }

    fn finish_main(self) -> Value {
        let code: Vec<Value> = self.ctx.code.iter()
            .map(|(op, arg)| json!([op, arg])).collect();
        let lines: Vec<Value> = self.ctx.code.iter().map(|_| json!(0)).collect();
        let main = json!({
            "name": "__main__", "n_params": 0, "n_required": 0,
            "is_variadic": false, "is_sub": true, "is_main": true,
            "is_coroutine": false, "return_type": "",
            "param_defaults": [], "param_names": [],
            "local_types": [], "local_defaults": [],
            "constants": self.ctx.consts, "code": code, "lines": lines,
        });
        json!({
            "format": "gbc", "version": 1,
            "n_globals": self.global_slots.len(),
            "main": main, "functions": {}, "classes": {}, "data": [],
        })
    }
}

fn is_simple_type(t: &str) -> bool {
    !t.starts_with("array:") && !t.starts_with("map:")
        && matches!(t, "integer" | "float" | "string" | "boolean")
}

fn node_name(n: &Node) -> &'static str {
    match n {
        Node::For { .. } => "For", Node::ForEach { .. } => "ForEach",
        Node::SubDecl { .. } => "SubDecl", Node::FunctionDecl { .. } => "FunctionDecl",
        Node::ClassDecl { .. } => "ClassDecl", Node::Select { .. } => "Select",
        Node::IndexAccess { .. } => "IndexAccess", Node::IndexAssign { .. } => "IndexAssign",
        Node::MemberAccess { .. } => "MemberAccess", Node::MemberAssign { .. } => "MemberAssign",
        Node::TupleLit { .. } => "TupleLit", Node::TupleAssign { .. } => "TupleAssign",
        Node::ListComp { .. } => "ListComp", Node::DictComp { .. } => "DictComp",
        Node::SetComp { .. } => "SetComp", Node::With { .. } => "With",
        Node::Try { .. } => "Try", Node::Throw { .. } => "Throw",
        Node::Data { .. } => "Data", Node::Read { .. } => "Read",
        Node::Restore => "Restore", Node::EnumDecl { .. } => "EnumDecl",
        Node::Input { .. } => "Input", Node::New { .. } => "New",
        Node::SliceAccess { .. } => "SliceAccess", Node::Yield(_) => "Yield",
        Node::TernaryExpr { .. } => "TernaryExpr", Node::Return(_) => "Return",
        _ => "?",
    }
}

/// AST -> `.gbc`-JSON (Stufe 3a). Fehler bei nicht unterstuetzten Konstrukten.
pub fn compile_to_gbc(ast: &Node) -> Result<Value, String> {
    let stmts = match ast {
        Node::Program { statements } => statements,
        _ => return Err("Erwartet Program-Knoten".into()),
    };
    // 3a: keine Funktionen/Klassen.
    for s in stmts {
        if matches!(s, Node::SubDecl { .. } | Node::FunctionDecl { .. } | Node::ClassDecl { .. }) {
            return Err("Stufe 3b: SUB/FUNCTION/CLASS noch nicht unterstuetzt".into());
        }
    }
    let mut c = Compiler::new();
    c.collect_globals(stmts)?;
    for s in stmts { c.stmt(s)?; }
    c.ctx.emit(oc::HALT, Value::Null);
    Ok(c.finish_main())
}
