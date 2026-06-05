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

use serde_json::{json, Map, Value};

use crate::ast::{CaseMatch, CaseVal, NumV, Node};

// Opcodes (Teilmenge; Werte aus bytecode.py / model::op).
mod oc {
    pub const LOAD_CONST: i64 = 1;
    pub const POP: i64 = 2;
    pub const DUP: i64 = 3;
    pub const LOAD_NAME: i64 = 10;
    pub const STORE_NAME: i64 = 11;
    pub const DECLARE_NAME: i64 = 12;
    pub const DECLARE_CONST: i64 = 13;
    pub const LOAD_LOCAL: i64 = 14;
    pub const STORE_LOCAL: i64 = 15;
    pub const DECLARE_LOCAL: i64 = 16;
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
    pub const CALL_USER: i64 = 50;
    pub const CALL_BUILTIN: i64 = 51;
    pub const CALL_METHOD: i64 = 52;
    pub const LOAD_FUNCREF: i64 = 53;
    pub const CALL_VALUE: i64 = 54;
    pub const RETURN: i64 = 60;
    pub const RETURN_VOID: i64 = 61;
    pub const BAND: i64 = 62;
    pub const BOR: i64 = 63;
    pub const BXOR: i64 = 64;
    pub const SHL: i64 = 65;
    pub const SHR: i64 = 66;
    pub const BNOT: i64 = 67;
    pub const IN_OP: i64 = 56;
    pub const BUILD_TUPLE: i64 = 68;
    pub const UNPACK_TUPLE: i64 = 69;
    pub const BUILD_TUPLE_DYN: i64 = 57;
    pub const SLICE: i64 = 55;
    pub const TRY_BEGIN: i64 = 95;
    pub const TRY_END: i64 = 96;
    pub const THROW: i64 = 97;
    pub const YIELD_VALUE: i64 = 115;
    pub const PRINT: i64 = 70;
    pub const INPUT_NAME: i64 = 71;
    pub const INPUT_LOCAL: i64 = 72;
    pub const PUSH_DATA: i64 = 75;
    pub const RESET_DATA_PTR: i64 = 76;
    pub const LOAD_INDEX: i64 = 90;
    pub const STORE_INDEX: i64 = 91;
    pub const DECLARE_ARRAY_NAME: i64 = 92;
    pub const NEW_INSTANCE: i64 = 80;
    pub const LOAD_FIELD: i64 = 81;
    pub const STORE_FIELD: i64 = 82;
    pub const LOAD_MEMBER: i64 = 83;
    pub const STORE_MEMBER: i64 = 84;
    pub const DECLARE_STRUCT_NAME: i64 = 86;
    pub const LOAD_SELF: i64 = 88;
    pub const LOAD_GLOBAL_SLOT: i64 = 111;
    pub const STORE_GLOBAL_SLOT: i64 = 112;
    pub const DECLARE_GLOBAL_SLOT: i64 = 113;
    pub const DECLARE_GLOBAL_CONST_SLOT: i64 = 114;
    pub const HALT: i64 = 99;
}

/// Konstanter Wert im Pool -- wird wie `serialize._enc` kodiert.
#[derive(Clone)]
enum CVal {
    Nil, Bool(bool), Int(i64), Float(f64), Str(String),
    /// Tupel-Literal (z.B. der `()`-Default fuer `DIM x AS TUPLE`).
    Tuple(Vec<CVal>),
    /// ENUM-/STATIC-CONST-Namespace im const-Pool (-> {"ns": {...}}).
    Ns { name: String, members: Vec<(String, CVal)> },
}

fn enc(c: &CVal) -> Value {
    match c {
        CVal::Nil => Value::Null,
        CVal::Bool(b) => json!({ "b": b }),
        CVal::Int(i) => json!(i),
        CVal::Float(f) => json!({ "f": f }),
        CVal::Str(s) => json!(s),
        CVal::Tuple(items) => Value::Array(items.iter().map(enc).collect()),
        CVal::Ns { name, members } => {
            let mut m = Map::new();
            for (k, v) in members { m.insert(k.clone(), enc(v)); }
            json!({ "ns": { "name": name, "members": Value::Object(m) } })
        }
    }
}

fn type_default(t: &str) -> CVal {
    match t {
        "integer" => CVal::Int(0),
        "float" => CVal::Float(0.0),
        "string" => CVal::Str(String::new()),
        "boolean" => CVal::Bool(false),
        "tuple" => CVal::Tuple(vec![]),
        // image/sound/file/sprite_atlas/funcref/coroutine + Klassen/externe Typen
        _ => CVal::Nil,
    }
}

type CR = Result<(), String>;

struct Ctx {
    code: Vec<(i64, Value)>,
    consts: Vec<Value>,
    // Jeder Eintrag: (Patch-IPs, try_depth bei Schleifen-Eintritt). BREAK/
    // CONTINUE muessen pro dazwischenliegendem TRY ein TRY_END emittieren.
    break_patches: Vec<(Vec<usize>, i64)>,
    continue_patches: Vec<(Vec<usize>, i64)>,
    try_depth: i64,
    local_slots: HashMap<String, usize>,
    local_types: Vec<String>,
    local_defaults: Vec<CVal>,
    is_main: bool,
    is_sub: bool,
    current_class: Option<String>,
}

impl Ctx {
    fn new() -> Self {
        Ctx { code: vec![], consts: vec![], break_patches: vec![], continue_patches: vec![],
              try_depth: 0,
              local_slots: HashMap::new(), local_types: vec![], local_defaults: vec![],
              is_main: true, is_sub: true, current_class: None }
    }
    /// Anonymer Local-Slot fuer Compiler-Zwischenwerte (Comprehensions, WITH,
    /// Tupel-Destructuring). KEIN DECLARE_LOCAL -- der Slot wird beim Frame-
    /// Eintritt aus `local_types`/`local_defaults` vorab alloziert (wie
    /// compiler._alloc_anon_slot).
    fn alloc_anon_slot(&mut self, type_name: &str) -> usize {
        let idx = self.local_types.len();
        self.local_types.push(type_name.to_string());
        self.local_defaults.push(CVal::Nil);
        self.local_slots.insert(format!("__anon_{}", idx), idx);
        idx
    }
    /// Benannter Local-Slot mit DECLARE_LOCAL (wie compiler._declare_local).
    /// Idempotent bei gleichem Namen.
    fn declare_local(&mut self, name: &str, type_name: &str) -> usize {
        if let Some(&s) = self.local_slots.get(name) { return s; }
        let slot = self.local_types.len();
        self.local_slots.insert(name.to_string(), slot);
        self.local_types.push(type_name.to_string());
        self.local_defaults.push(type_default(type_name));
        let d = enc(&type_default(type_name));
        self.emit(oc::DECLARE_LOCAL, json!([slot, type_name, d]));
        slot
    }
    fn alloc_temp(&mut self, type_name: &str) -> usize {
        let slot = self.local_types.len();
        self.local_slots.insert(format!("__tmp_{}", slot), slot);
        self.local_types.push(type_name.to_string());
        self.local_defaults.push(type_default(type_name));
        let d = enc(&type_default(type_name));
        self.emit(oc::DECLARE_LOCAL, json!([slot, type_name, d]));
        slot
    }
    fn reserve_temp(&mut self, name: &str) -> usize {
        if let Some(&s) = self.local_slots.get(name) { return s; }
        let slot = self.local_types.len();
        self.local_slots.insert(name.to_string(), slot);
        self.local_types.push("any".to_string());
        self.local_defaults.push(CVal::Nil);
        slot
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

/// Signatur einer User-Funktion (fuer CALL_USER-Aufloesung + FUNCREF).
struct FnSig {
    n_params: usize,
    n_required: usize,
    param_names: Vec<String>,
    param_defaults: Vec<Option<CVal>>,
    param_types: Vec<String>,
    is_variadic: bool,
    is_sub: bool,
    return_type: String,
    is_coroutine: bool,
}

/// Ein Feld einer Klasse/Struct.
struct FieldInfo { name: String, type_name: String, array_dims: Vec<i64> }

/// Klassen-Metadaten (Compile-Zeit). `methods`/`properties` fuer Dispatch +
/// Au_floesung; `compiled` haelt die fertigen Methoden-JSONs.
struct ClassInfo {
    name: String,
    parent_name: String,
    is_struct: bool,
    fields: Vec<FieldInfo>,
    method_sigs: HashMap<String, FnSig>,
    property_set: Vec<String>,         // lowercase
    compiled: Vec<(String, Value)>,    // method-name -> func-json
}

pub struct Compiler {
    global_slots: HashMap<String, usize>,
    global_vars: std::collections::HashSet<String>,
    fn_sigs: HashMap<String, FnSig>,
    compiled_fns: Vec<(String, Value)>,
    classes: HashMap<String, ClassInfo>,
    struct_names: std::collections::HashSet<String>,
    /// Externe Typen importierter Module (lowercase) -- gueltige DIM-Typen
    /// (z.B. "vec2", "json_handle", aliasiert auch "j_handle"). Aus
    /// preprocess::compile_env.
    external_types: std::collections::HashSet<String>,
    /// `(alias, modul)`-Paare aus `IMPORT "<modul>" AS <alias>` -- damit
    /// aliasierte Builtin-Aufrufe (`j_parse`) auf den kanonischen Namen
    /// (`json_parse`) zurueckabgebildet werden, den gbrt nativ kennt.
    builtin_aliases: Vec<(String, String)>,
    ctx: Ctx,
}

impl Compiler {
    fn new(external_types: std::collections::HashSet<String>,
           builtin_aliases: Vec<(String, String)>) -> Self {
        Compiler { global_slots: HashMap::new(),
                   global_vars: std::collections::HashSet::new(),
                   fn_sigs: HashMap::new(), compiled_fns: vec![],
                   classes: HashMap::new(),
                   struct_names: std::collections::HashSet::new(),
                   external_types, builtin_aliases, ctx: Ctx::new() }
    }

    /// Bekannter skalarer DIM-Typ: Werttyp ODER importierter externer Modul-Typ.
    fn is_known_value_type(&self, t: &str) -> bool {
        is_value_type(t) || self.external_types.contains(t)
    }

    /// Bildet einen aliasierten Builtin-Namen auf den kanonischen zurueck
    /// (`j_parse` -> `json_parse`, `v` -> `vec2`). Unveraendert, wenn kein
    /// Alias passt. So sieht die VM nur native Builtin-Namen.
    fn resolve_builtin_alias(&self, name: &str) -> String {
        for (alias, module) in &self.builtin_aliases {
            if name == alias {
                return module.clone();
            }
            if let Some(rest) = name.strip_prefix(&format!("{}_", alias)) {
                return format!("{}_{}", module, rest);
            }
        }
        name.to_string()
    }

    fn alloc_slot(&mut self, name: &str) {
        let n = self.global_slots.len();
        self.global_slots.entry(name.to_string()).or_insert(n);
    }

    /// Bekommt ein DIM einen Slot? Skalar (primitiv ODER Klasse), nicht
    /// Array/Map/Struct (die haben eigene Init-Ops).
    fn is_slot_dim(&self, type_name: &str, array_dims: &Option<Vec<Node>>) -> bool {
        array_dims.is_none()
            && !type_name.starts_with("array:") && !type_name.starts_with("map:")
            && !self.struct_names.contains(type_name)
    }

    /// Pre-Pass: Top-Level-Globals (Skalar-DIM/CONST) -> Slot-Index.
    fn collect_globals(&mut self, stmts: &[Node]) -> CR {
        for s in stmts {
            match s {
                Node::Dim { name, type_name, array_dims } => {
                    self.global_vars.insert(name.clone());
                    if self.is_slot_dim(type_name, array_dims) { self.alloc_slot(name); }
                }
                Node::MultiDim { dims } => {
                    for d in dims {
                        if let Node::Dim { name, type_name, array_dims } = d {
                            self.global_vars.insert(name.clone());
                            if self.is_slot_dim(type_name, array_dims) { self.alloc_slot(name); }
                        }
                    }
                }
                Node::Const { name, .. } => {
                    self.global_vars.insert(name.clone());
                    self.alloc_slot(name);
                }
                Node::For { var, .. } => {
                    self.global_vars.insert(var.clone());
                    self.alloc_slot(var);
                }
                Node::EnumDecl { name, .. } => { self.alloc_slot(name); }
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
            Node::For { var, start, end, step, body } =>
                self.stmt_for(var, start, end, step, body),
            Node::Break => self.emit_break(),
            Node::Continue => self.emit_continue(),
            Node::Return(value) => {
                if self.ctx.is_main {
                    return Err("RETURN nur in SUB/FUNCTION".into());
                }
                match value {
                    Some(v) => {
                        if self.ctx.is_sub {
                            return Err("SUB darf RETURN nicht mit Wert verwenden".into());
                        }
                        self.expr(v)?;
                        self.ctx.emit(oc::RETURN, Value::Null);
                    }
                    None => {
                        if !self.ctx.is_sub {
                            return Err("FUNCTION braucht einen Wert bei RETURN".into());
                        }
                        self.ctx.emit(oc::RETURN_VOID, Value::Null);
                    }
                }
                Ok(())
            }
            Node::IndexAssign { target, indices, value } => {
                self.expr(target)?;
                for ix in indices { self.expr(ix)?; }
                self.expr(value)?;
                self.ctx.emit(oc::STORE_INDEX, json!(indices.len()));
                Ok(())
            }
            Node::MemberAssign { target, name, value } => {
                self.expr(target)?;
                self.expr(value)?;
                let idx = self.ctx.add_const(json!(name));
                self.ctx.emit(oc::STORE_MEMBER, json!(idx));
                Ok(())
            }
            Node::EnumDecl { name, members } => self.stmt_enum(name, members),
            Node::Input { prompt, target } => self.stmt_input(prompt, target),
            Node::Data { .. } => Ok(()),     // Werte werden separat eingesammelt
            Node::Restore => { self.ctx.emit(oc::RESET_DATA_PTR, Value::Null); Ok(()) }
            Node::Read { targets } => {
                for t in targets {
                    self.ctx.emit(oc::PUSH_DATA, Value::Null);
                    self.emit_store_target(t)?;
                }
                Ok(())
            }
            Node::Select { subject, cases, else_block } =>
                self.stmt_select(subject, cases, else_block),
            Node::ForEach { var, iterable, body } => self.stmt_foreach(var, iterable, body),
            Node::Repeat { body, condition } => self.stmt_repeat(body, condition),
            Node::Try { body, catch_var, catch_block } =>
                self.stmt_try(body, catch_var, catch_block),
            Node::Throw { value } => {
                self.expr(value)?;
                self.ctx.emit(oc::THROW, Value::Null);
                Ok(())
            }
            Node::With { var_name, target, body } => self.stmt_with(var_name, target, body),
            Node::TupleAssign { targets, value } => self.stmt_tuple_assign(targets, value),
            other => Err(format!("Stufe 3e: Statement {} noch nicht unterstuetzt",
                                 node_name(other))),
        }
    }

    fn known_elem(&self, t: &str) -> bool {
        self.is_known_value_type(t) || self.classes.contains_key(t)
    }

    fn stmt_dim(&mut self, name: &str, type_name: &str, array_dims: &Option<Vec<Node>>) -> CR {
        // Sized-Array: `DIM x[10, 20] AS T` -- type_name = Element-Typ.
        if let Some(dims) = array_dims {
            if !self.known_elem(type_name) {
                return Err(format!("Stufe 3e: Array-Element-Typ '{}' nicht unterstuetzt", type_name));
            }
            for de in dims { self.expr(de)?; }
            let name_idx = self.ctx.add_const(json!(name));
            self.ctx.emit(oc::DECLARE_ARRAY_NAME, json!([name_idx, type_name, dims.len()]));
            return Ok(());
        }
        // ARRAY OF T / MAP OF T (groessenlos).
        if type_name.starts_with("array:") || type_name.starts_with("map:") {
            let name_idx = self.ctx.add_const(json!(name));
            let type_idx = self.ctx.add_const(json!(type_name));
            let default_idx = self.ctx.add_const(Value::Null);
            self.ctx.emit(oc::DECLARE_NAME, json!([name_idx, type_idx, default_idx]));
            return Ok(());
        }
        // STRUCT-Instanz -> Auto-Init via DECLARE_STRUCT_NAME.
        if self.struct_names.contains(type_name) {
            let name_idx = self.ctx.add_const(json!(name));
            self.ctx.emit(oc::DECLARE_STRUCT_NAME, json!([name_idx, type_name]));
            return Ok(());
        }
        // Skalar: Werttyp/externer Modul-Typ (Default je Typ) oder Klasse (NIL).
        if !self.is_known_value_type(type_name) && !self.classes.contains_key(type_name) {
            return Err(format!("Stufe 3e: DIM-Typ '{}' noch nicht unterstuetzt", type_name));
        }
        let name_idx = self.ctx.add_const(json!(name));
        let type_idx = self.ctx.add_const(json!(type_name));
        let default_idx = self.ctx.add_const(enc(&type_default(type_name)));
        if let Some(&slot) = self.global_slots.get(name) {
            self.ctx.emit(oc::DECLARE_GLOBAL_SLOT,
                          json!([slot as i64, name_idx, type_idx, default_idx]));
        } else if !self.ctx.is_main {
            // Innerhalb einer Funktion: ein ECHTER funktions-lokaler Slot, kein
            // globaler Name. Sonst kollidieren gleichnamige Locals verschiedener
            // Funktionen in self.globals (ein Aufruf ueberschreibt die Variable
            // -- z.B. eine FOR-Schleifenvariable -> Endlosschleife).
            self.ctx.declare_local(name, type_name);
        } else {
            self.ctx.emit(oc::DECLARE_NAME, json!([name_idx, type_idx, default_idx]));
        }
        Ok(())
    }

    fn expr_new(&mut self, class_name: &str, args: &Option<Vec<Node>>) -> CR {
        if !self.classes.contains_key(class_name) {
            return Err(format!("Klasse '{}' nicht gefunden", class_name));
        }
        let args = match args {
            None => { self.ctx.emit(oc::NEW_INSTANCE, json!([class_name, 0, false])); return Ok(()); }
            Some(a) => a,
        };
        let has_named = args.iter().any(|a| matches!(a, Node::NamedArg { .. }));
        if !has_named {
            for a in args { self.expr(a)?; }
            self.ctx.emit(oc::NEW_INSTANCE, json!([class_name, args.len(), true]));
            return Ok(());
        }
        // Named-Args bei NEW -> Init-Methode aufloesen.
        if self.find_method_sig(class_name, "init").is_none() {
            return Err(format!("NEW {} mit Named-Args, aber keine Init-Methode", class_name));
        }
        let actions = self.resolve_method_named_args(class_name, "init", args)?;
        let n = actions.len();
        for act in actions {
            match act {
                RArg::Expr(e) => self.expr(e)?,
                RArg::Default(cv) => { let c = self.ctx.add_const(enc(&cv)); self.ctx.emit(oc::LOAD_CONST, json!(c)); }
            }
        }
        self.ctx.emit(oc::NEW_INSTANCE, json!([class_name, n, true]));
        Ok(())
    }

    fn stmt_enum(&mut self, name: &str, members: &[(String, Option<Node>)]) -> CR {
        let mut out: Vec<(String, CVal)> = Vec::new();
        let mut next_auto: i64 = 0;
        for (mname, expr) in members {
            let v = match expr {
                None => next_auto,
                Some(e) => eval_int_literal(e).ok_or_else(||
                    format!("ENUM {}.{}: Wert muss Integer-Literal sein", name, mname))?,
            };
            out.push((mname.to_lowercase(), CVal::Int(v)));
            next_auto = v + 1;
        }
        let ns = CVal::Ns { name: name.to_string(), members: out };
        self.emit_namespace_const(name, ns);
        Ok(())
    }

    /// LOAD_CONST(ns) + DECLARE_GLOBAL_CONST_SLOT/DECLARE_CONST (wie ENUM/Static).
    fn emit_namespace_const(&mut self, name: &str, ns: CVal) {
        let name_idx = self.ctx.add_const(json!(name));
        let type_idx = self.ctx.add_const(Value::Null);
        let val_idx = self.ctx.add_const(enc(&ns));
        self.ctx.emit(oc::LOAD_CONST, json!(val_idx));
        if let Some(&slot) = self.global_slots.get(name) {
            self.ctx.emit(oc::DECLARE_GLOBAL_CONST_SLOT, json!([slot as i64, name_idx, type_idx]));
        } else {
            self.ctx.emit(oc::DECLARE_CONST, json!([name_idx, type_idx]));
        }
    }

    fn resolve_method_named_args<'a>(&self, class_name: &str, m: &str, args: &'a [Node])
        -> Result<Vec<RArg<'a>>, String> {
        let sig = self.find_method_sig(class_name, m).unwrap();
        resolve_args_with_sig(sig, m, args)
    }

    fn stmt_input(&mut self, prompt: &Option<Box<Node>>, target: &str) -> CR {
        let has_prompt = prompt.is_some();
        if let Some(p) = prompt { self.expr(p)?; }
        if let Some(&slot) = self.ctx.local_slots.get(target) {
            self.ctx.emit(oc::INPUT_LOCAL, json!([slot, has_prompt]));
        } else {
            let name_idx = self.ctx.add_const(json!(target));
            self.ctx.emit(oc::INPUT_NAME, json!([name_idx, has_prompt]));
        }
        Ok(())
    }

    /// Store-Code fuer ein READ-Ziel (Wert liegt oben auf dem Stack).
    fn emit_store_target(&mut self, target: &Node) -> CR {
        match target {
            Node::Identifier(name) => {
                if let Some(&slot) = self.ctx.local_slots.get(name) {
                    self.ctx.emit(oc::STORE_LOCAL, json!(slot));
                } else {
                    let idx = self.ctx.add_const(json!(name));
                    self.ctx.emit(oc::STORE_NAME, json!(idx));
                }
                Ok(())
            }
            Node::IndexAccess { target: arr, indices } => {
                let tmp = self.ctx.reserve_temp("__read_tmp");
                self.ctx.emit(oc::STORE_LOCAL, json!(tmp));
                self.expr(arr)?;
                for ie in indices { self.expr(ie)?; }
                self.ctx.emit(oc::LOAD_LOCAL, json!(tmp));
                self.ctx.emit(oc::STORE_INDEX, json!(indices.len()));
                Ok(())
            }
            _ => Err("Stufe 3d: READ-Ziel (MemberAccess) noch nicht unterstuetzt".into()),
        }
    }

    fn stmt_for(&mut self, var: &str, start: &Node, end: &Node,
                step: &Option<Box<Node>>, body: &[Node]) -> CR {
        // Schleifenvariable deklarieren (main: Slot oder Name).
        let name_idx = self.ctx.add_const(json!(var));
        let type_idx = self.ctx.add_const(json!("integer"));
        let default_idx = self.ctx.add_const(json!(0));
        if let Some(&slot) = self.global_slots.get(var) {
            self.ctx.emit(oc::DECLARE_GLOBAL_SLOT,
                          json!([slot as i64, name_idx, type_idx, default_idx]));
        } else if !self.ctx.is_main {
            // FOR-Variable in einer Funktion -> funktions-lokaler Slot (nicht
            // globaler Name), sonst korrumpiert ein gleichnamiges Local einer
            // aufgerufenen Funktion den Schleifenzaehler (Endlosschleife).
            self.ctx.declare_local(var, "integer");
        } else {
            self.ctx.emit(oc::DECLARE_NAME, json!([name_idx, type_idx, default_idx]));
        }
        // Konstanten-STEP-Richtung bestimmen.
        let step_num: Option<f64> = match step.as_deref() {
            None => Some(1.0),
            Some(Node::NumberLit(NumV::Int(i))) => Some(*i as f64),
            Some(Node::NumberLit(NumV::Float(f))) => Some(*f),
            _ => None,
        };
        let const_pos = step_num.map(|n| n > 0.0).unwrap_or(false);
        let const_neg = step_num.map(|n| n < 0.0).unwrap_or(false);
        // var = start
        self.expr(start)?;
        self.store_var(var);
        // end -> Temp
        let end_slot = self.ctx.alloc_temp("integer");
        self.expr(end)?;
        self.ctx.emit(oc::STORE_LOCAL, json!(end_slot));
        // step -> Temp (nur wenn nicht-konstant)
        let mut step_slot: i64 = -1;
        let step_const_val: Option<CVal> = match step.as_deref() {
            None => Some(CVal::Int(1)),
            Some(Node::NumberLit(NumV::Int(i))) => Some(CVal::Int(*i)),
            Some(Node::NumberLit(NumV::Float(f))) => Some(CVal::Float(*f)),
            _ => None,
        };
        if step_const_val.is_none() {
            let s = self.ctx.alloc_temp("integer");
            step_slot = s as i64;
            self.expr(step.as_deref().unwrap())?;
            self.ctx.emit(oc::STORE_LOCAL, json!(s));
        }
        let loop_start = self.ctx.here();
        self.break_continue_enter();
        let mut exit_jumps: Vec<usize> = vec![];
        if const_pos {
            self.load_var(var);
            self.ctx.emit(oc::LOAD_LOCAL, json!(end_slot));
            self.ctx.emit(oc::GT, Value::Null);
            exit_jumps.push(self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null));
        } else if const_neg {
            self.load_var(var);
            self.ctx.emit(oc::LOAD_LOCAL, json!(end_slot));
            self.ctx.emit(oc::LT, Value::Null);
            exit_jumps.push(self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null));
        } else {
            // Laufzeit-Richtung: step < 0 ?
            self.ctx.emit(oc::LOAD_LOCAL, json!(step_slot));
            let z = self.ctx.add_const(json!(0));
            self.ctx.emit(oc::LOAD_CONST, json!(z));
            self.ctx.emit(oc::LT, Value::Null);
            let neg_jump = self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null);
            self.load_var(var);
            self.ctx.emit(oc::LOAD_LOCAL, json!(end_slot));
            self.ctx.emit(oc::GT, Value::Null);
            exit_jumps.push(self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null));
            let body_jump = self.ctx.emit(oc::JUMP, Value::Null);
            let t = self.ctx.here(); self.ctx.patch(neg_jump, t);
            self.load_var(var);
            self.ctx.emit(oc::LOAD_LOCAL, json!(end_slot));
            self.ctx.emit(oc::LT, Value::Null);
            exit_jumps.push(self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null));
            let t2 = self.ctx.here(); self.ctx.patch(body_jump, t2);
        }
        for st in body { self.stmt(st)?; }
        // CONTINUE -> Inkrement
        let inc_target = self.ctx.here();
        let cont = self.ctx.continue_patches.pop().unwrap().0;
        for ip in cont { self.ctx.patch(ip, inc_target); }
        // var += step
        self.load_var(var);
        match step_const_val {
            Some(cv) => { let c = self.ctx.add_const(enc(&cv)); self.ctx.emit(oc::LOAD_CONST, json!(c)); }
            None => { self.ctx.emit(oc::LOAD_LOCAL, json!(step_slot)); }
        }
        self.ctx.emit(oc::ADD, Value::Null);
        self.store_var(var);
        self.ctx.emit(oc::JUMP, json!(loop_start));
        let end_ip = self.ctx.here();
        for ip in exit_jumps { self.ctx.patch(ip, end_ip); }
        let brk = self.ctx.break_patches.pop().unwrap().0;
        for ip in brk { self.ctx.patch(ip, end_ip); }
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
        let cont_patches = self.ctx.continue_patches.pop().unwrap().0;
        for ip in cont_patches { self.ctx.patch(ip, start); }
        self.ctx.emit(oc::JUMP, json!(start));
        let end = self.ctx.here();
        self.ctx.patch(exit, end);
        let break_patches = self.ctx.break_patches.pop().unwrap().0;
        for ip in break_patches { self.ctx.patch(ip, end); }
        Ok(())
    }

    fn break_continue_enter(&mut self) {
        let d = self.ctx.try_depth;
        self.ctx.break_patches.push((vec![], d));
        self.ctx.continue_patches.push((vec![], d));
    }
    fn emit_break(&mut self) -> CR {
        let loop_depth = match self.ctx.break_patches.last() {
            Some((_, d)) => *d,
            None => return Err("BREAK ausserhalb einer Schleife".into()),
        };
        for _ in 0..(self.ctx.try_depth - loop_depth) { self.ctx.emit(oc::TRY_END, Value::Null); }
        let ip = self.ctx.emit(oc::JUMP, Value::Null);
        self.ctx.break_patches.last_mut().unwrap().0.push(ip);
        Ok(())
    }
    fn emit_continue(&mut self) -> CR {
        let loop_depth = match self.ctx.continue_patches.last() {
            Some((_, d)) => *d,
            None => return Err("CONTINUE ausserhalb einer Schleife".into()),
        };
        for _ in 0..(self.ctx.try_depth - loop_depth) { self.ctx.emit(oc::TRY_END, Value::Null); }
        let ip = self.ctx.emit(oc::JUMP, Value::Null);
        self.ctx.continue_patches.last_mut().unwrap().0.push(ip);
        Ok(())
    }

    // ---------------------------------------------------- Variablen
    /// Ist `name` ein Feld der aktuellen Klasse (inkl. geerbter)?
    fn is_field(&self, name: &str) -> bool {
        let mut cur = self.ctx.current_class.as_deref();
        while let Some(cn) = cur {
            if let Some(ci) = self.classes.get(cn) {
                if ci.fields.iter().any(|f| f.name == name) { return true; }
                cur = if ci.parent_name.is_empty() { None } else { Some(&ci.parent_name) };
            } else { return false; }
        }
        false
    }

    /// Methode `m` entlang der Vererbung der Klasse `class_name` finden?
    fn resolve_method(&self, class_name: &str, m: &str) -> bool {
        let mut cur = Some(class_name.to_string());
        while let Some(cn) = cur {
            if let Some(ci) = self.classes.get(&cn) {
                if ci.method_sigs.contains_key(m) { return true; }
                cur = if ci.parent_name.is_empty() { None } else { Some(ci.parent_name.clone()) };
            } else { return false; }
        }
        false
    }

    /// Methoden-Signatur entlang der Vererbung (fuer Named-Args bei NEW).
    fn find_method_sig(&self, class_name: &str, m: &str) -> Option<&FnSig> {
        let mut cur = class_name.to_string();
        loop {
            let ci = self.classes.get(&cur)?;
            if let Some(s) = ci.method_sigs.get(m) { return Some(s); }
            if ci.parent_name.is_empty() { return None; }
            cur = ci.parent_name.clone();
        }
    }

    fn load_var(&mut self, name: &str) {
        if self.ctx.local_slots.contains_key(name) {
            let slot = self.ctx.local_slots[name];
            self.ctx.emit(oc::LOAD_LOCAL, json!(slot));
        } else if name == "self" && self.ctx.current_class.is_some() {
            self.ctx.emit(oc::LOAD_SELF, Value::Null);
        } else if self.is_field(name) {
            let idx = self.ctx.add_const(json!(name));
            self.ctx.emit(oc::LOAD_FIELD, json!(idx));
        } else if let Some(&slot) = self.global_slots.get(name) {
            self.ctx.emit(oc::LOAD_GLOBAL_SLOT, json!(slot));
        } else {
            let idx = self.ctx.add_const(json!(name));
            self.ctx.emit(oc::LOAD_NAME, json!(idx));
        }
    }
    fn store_var(&mut self, name: &str) {
        if self.ctx.local_slots.contains_key(name) {
            let slot = self.ctx.local_slots[name];
            self.ctx.emit(oc::STORE_LOCAL, json!(slot));
        } else if self.is_field(name) {
            let idx = self.ctx.add_const(json!(name));
            self.ctx.emit(oc::STORE_FIELD, json!(idx));
        } else if let Some(&slot) = self.global_slots.get(name) {
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
            Node::Identifier(name) => {
                // Bare User-Function -> FUNCREF (Variable/Feld verschattet sie).
                if !self.ctx.local_slots.contains_key(name)
                    && !self.is_field(name)
                    && !self.global_vars.contains(name)
                    && self.fn_sigs.contains_key(name)
                {
                    let idx = self.ctx.add_const(json!(name));
                    self.ctx.emit(oc::LOAD_FUNCREF, json!(idx));
                } else {
                    self.load_var(name);
                }
                Ok(())
            }
            Node::UnaryOp { op, operand } => self.expr_unary(op, operand),
            Node::BinaryOp { op, left, right } => self.expr_binary(op, left, right),
            Node::Call { callee, args } => self.expr_call(callee, args),
            Node::IndexAccess { target, indices } => {
                self.expr(target)?;
                for ix in indices { self.expr(ix)?; }
                self.ctx.emit(oc::LOAD_INDEX, json!(indices.len()));
                Ok(())
            }
            Node::MemberAccess { target, name } => {
                self.expr(target)?;
                let idx = self.ctx.add_const(json!(name));
                self.ctx.emit(oc::LOAD_MEMBER, json!(idx));
                Ok(())
            }
            Node::New { class_name, args } => self.expr_new(class_name, args),
            Node::TupleLit { elements } => {
                for el in elements { self.expr(el)?; }
                self.ctx.emit(oc::BUILD_TUPLE, json!(elements.len()));
                Ok(())
            }
            Node::TernaryExpr { cond, then_expr, else_expr } => {
                self.expr(cond)?;
                let jf = self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null);
                self.expr(then_expr)?;
                let jend = self.ctx.emit(oc::JUMP, Value::Null);
                let t = self.ctx.here(); self.ctx.patch(jf, t);
                self.expr(else_expr)?;
                let e = self.ctx.here(); self.ctx.patch(jend, e);
                Ok(())
            }
            Node::SliceAccess { target, lo, hi } => {
                self.expr(target)?;
                if let Some(l) = lo { self.expr(l)?; }
                if let Some(h) = hi { self.expr(h)?; }
                let arg = json!([enc(&CVal::Bool(lo.is_some())), enc(&CVal::Bool(hi.is_some()))]);
                self.ctx.emit(oc::SLICE, arg);
                Ok(())
            }
            Node::Yield(v) => {
                if self.ctx.is_main {
                    return Err("YIELD nur in SUB/FUNCTION erlaubt".into());
                }
                match v {
                    Some(ex) => self.expr(ex)?,
                    None => { let c = self.ctx.add_const(Value::Null); self.ctx.emit(oc::LOAD_CONST, json!(c)); }
                }
                self.ctx.emit(oc::YIELD_VALUE, Value::Null);
                Ok(())
            }
            Node::ListComp { var, iterable, filter, transform } =>
                self.expr_listcomp(var, iterable, filter, transform),
            Node::SetComp { var, iterable, filter, transform } => {
                self.expr_listcomp(var, iterable, filter, transform)?;
                self.ctx.emit(oc::CALL_BUILTIN, json!(["__set_dedup", 1]));
                Ok(())
            }
            Node::DictComp { var, iterable, filter, key, value } =>
                self.expr_dictcomp(var, iterable, filter, key, value),
            other => Err(format!("Stufe 3e: Ausdruck {} noch nicht unterstuetzt",
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
        // Methoden-Aufruf obj.method(...) (auch Container-Methoden wie .length()).
        if let Node::MemberAccess { target, name } = callee {
            if args.iter().any(|a| matches!(a, Node::NamedArg { .. })) {
                return Err(format!("{}: Named-Args nur bei SUB/FUNCTION/NEW", name));
            }
            self.expr(target)?;
            for a in args { self.expr(a)?; }
            self.ctx.emit(oc::CALL_METHOD, json!([name, args.len()]));
            return Ok(());
        }
        let name = match callee {
            Node::Identifier(n) => n.clone(),
            _ => return Err("Stufe 3e: aufrufbare Werte noch nicht unterstuetzt".into()),
        };
        let has_named = args.iter().any(|a| matches!(a, Node::NamedArg { .. }));
        // Impliziter Methoden-Aufruf: bare Name = Methode der eigenen Klasse.
        if let Some(cn) = self.ctx.current_class.clone() {
            if self.resolve_method(&cn, &name) {
                if has_named {
                    return Err(format!("{}: Named-Args bei Methoden-Call nicht unterstuetzt", name));
                }
                self.ctx.emit(oc::LOAD_SELF, Value::Null);
                for a in args { self.expr(a)?; }
                self.ctx.emit(oc::CALL_METHOD, json!([name, args.len()]));
                return Ok(());
            }
        }
        let is_local = self.ctx.local_slots.contains_key(&name);
        let is_global = self.global_vars.contains(&name);
        // User-Funktion (Variable gleichen Namens verschattet sie).
        if self.fn_sigs.contains_key(&name) && !is_local && !is_global {
            if self.fn_sigs[&name].is_variadic {
                if has_named { return Err(format!("{}: keine Named-Args bei variadic", name)); }
                for a in args { self.expr(a)?; }
                self.ctx.emit(oc::CALL_USER, json!([name, args.len()]));
                return Ok(());
            }
            let actions = self.resolve_named_args(&name, args)?;
            let n = actions.len();
            for act in actions {
                match act {
                    RArg::Expr(e) => self.expr(e)?,
                    RArg::Default(cv) => {
                        let c = self.ctx.add_const(enc(&cv));
                        self.ctx.emit(oc::LOAD_CONST, json!(c));
                    }
                }
            }
            self.ctx.emit(oc::CALL_USER, json!([name, n]));
            return Ok(());
        }
        if has_named {
            return Err(format!("{}: Named-Args nur bei SUB/FUNCTION", name));
        }
        // Globale FUNCREF-Variable -> CALL_VALUE, sonst Builtin.
        if is_global || is_local {
            self.load_var(&name);
            for a in args { self.expr(a)?; }
            self.ctx.emit(oc::CALL_VALUE, json!(args.len()));
        } else {
            // Aliasierten Builtin-Namen auf den kanonischen zurueckabbilden
            // (j_parse -> json_parse), damit gbrt ihn nativ findet.
            let bname = self.resolve_builtin_alias(&name);
            for a in args { self.expr(a)?; }
            self.ctx.emit(oc::CALL_BUILTIN, json!([bname, args.len()]));
        }
        Ok(())
    }

    // ---------------------------------------------------- 3e: Kontrollfluss
    /// SELECT CASE -- Subject bleibt auf dem Stack, pro Match per DUP geklont.
    /// Optionaler Guard (`WHERE`) springt bei FALSE zum naechsten Case.
    fn stmt_select(&mut self, subject: &Node,
                   cases: &[(Vec<CaseMatch>, Option<Node>, Vec<Node>)],
                   else_block: &[Node]) -> CR {
        self.expr(subject)?;                  // Stack: [subj]
        let mut end_jumps: Vec<usize> = vec![];
        for (matches, guard, block) in cases {
            let mut block_jumps: Vec<usize> = vec![];
            for m in matches { self.emit_case_match(m, &mut block_jumps)?; }
            let skip_jump = self.ctx.emit(oc::JUMP, Value::Null);
            let block_start = self.ctx.here();
            for ip in block_jumps { self.ctx.patch(ip, block_start); }
            let mut guard_skip: Option<usize> = None;
            if let Some(g) = guard {
                self.expr(g)?;                // Stack: [subj, guard_val]
                guard_skip = Some(self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null));
            }
            self.ctx.emit(oc::POP, Value::Null);   // Stack: []
            for st in block { self.stmt(st)?; }
            end_jumps.push(self.ctx.emit(oc::JUMP, Value::Null));
            let next_case = self.ctx.here();
            self.ctx.patch(skip_jump, next_case);
            if let Some(gs) = guard_skip { self.ctx.patch(gs, next_case); }
        }
        self.ctx.emit(oc::POP, Value::Null);       // kein Match -> subj poppen
        for st in else_block { self.stmt(st)?; }
        let end = self.ctx.here();
        for ip in end_jumps { self.ctx.patch(ip, end); }
        Ok(())
    }

    /// Ein CASE-Match. Stack vor/nach: [subj]. Bei Treffer wird ein
    /// JUMP_IF_TRUE-IP fuer den spaeteren Sprung zum block_start gesammelt.
    fn emit_case_match(&mut self, m: &CaseMatch, block_jumps: &mut Vec<usize>) -> CR {
        match m.kind.as_str() {
            "value" => {
                self.ctx.emit(oc::DUP, Value::Null);
                self.case_expr(&m.values[0])?;
                self.ctx.emit(oc::EQ, Value::Null);
                block_jumps.push(self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null));
            }
            "range" => {
                self.ctx.emit(oc::DUP, Value::Null);
                self.case_expr(&m.values[0])?;     // lo
                self.ctx.emit(oc::GEQ, Value::Null);
                let skip_to_next = self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null);
                self.ctx.emit(oc::DUP, Value::Null);
                self.case_expr(&m.values[1])?;     // hi
                self.ctx.emit(oc::LEQ, Value::Null);
                block_jumps.push(self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null));
                let t = self.ctx.here();
                self.ctx.patch(skip_to_next, t);
            }
            "is" => {
                let op = match &m.values[0] {
                    CaseVal::Op(s) => s.clone(),
                    _ => return Err("CASE IS: Operator-String erwartet".into()),
                };
                let code = match op.as_str() {
                    "=" => oc::EQ, "<>" => oc::NEQ, "<" => oc::LT, ">" => oc::GT,
                    "<=" => oc::LEQ, ">=" => oc::GEQ,
                    _ => return Err(format!("CASE IS: unbekannter Operator '{}'", op)),
                };
                self.ctx.emit(oc::DUP, Value::Null);
                self.case_expr(&m.values[1])?;
                self.ctx.emit(code, Value::Null);
                block_jumps.push(self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null));
            }
            other => return Err(format!("Interner Fehler: CASE-Match-Typ '{}'", other)),
        }
        Ok(())
    }

    fn case_expr(&mut self, v: &CaseVal) -> CR {
        match v {
            CaseVal::Expr(n) => self.expr(n),
            CaseVal::Op(_) => Err("CASE: unerwarteter Operator-Wert".into()),
        }
    }

    /// FOR EACH var IN iterable -- Desugar zu einem Index-Loop ueber
    /// __comp_iter(iterable) (TUPLE), wie compiler._stmt_ForEach.
    fn stmt_foreach(&mut self, var: &str, iterable: &Node, body: &[Node]) -> CR {
        if self.ctx.is_main {
            if !self.ctx.local_slots.contains_key(var) {
                let name_idx = self.ctx.add_const(json!(var));
                let type_idx = self.ctx.add_const(json!("any"));
                let default_idx = self.ctx.add_const(Value::Null);
                if let Some(&g) = self.global_slots.get(var) {
                    self.ctx.emit(oc::DECLARE_GLOBAL_SLOT,
                                  json!([g as i64, name_idx, type_idx, default_idx]));
                } else {
                    self.ctx.emit(oc::DECLARE_NAME, json!([name_idx, type_idx, default_idx]));
                }
            }
        } else if !self.ctx.local_slots.contains_key(var) {
            self.ctx.declare_local(var, "any");
        }
        let it_slot = self.ctx.alloc_temp("tuple");
        self.expr(iterable)?;
        self.ctx.emit(oc::CALL_BUILTIN, json!(["__comp_iter", 1]));
        self.ctx.emit(oc::STORE_LOCAL, json!(it_slot));
        let idx_slot = self.ctx.alloc_temp("integer");
        let z = self.ctx.add_const(json!(0));
        self.ctx.emit(oc::LOAD_CONST, json!(z));
        self.ctx.emit(oc::STORE_LOCAL, json!(idx_slot));
        let len_slot = self.ctx.alloc_temp("integer");
        self.ctx.emit(oc::LOAD_LOCAL, json!(it_slot));
        self.ctx.emit(oc::CALL_BUILTIN, json!(["len", 1]));
        self.ctx.emit(oc::STORE_LOCAL, json!(len_slot));
        let loop_start = self.ctx.here();
        self.break_continue_enter();
        self.ctx.emit(oc::LOAD_LOCAL, json!(idx_slot));
        self.ctx.emit(oc::LOAD_LOCAL, json!(len_slot));
        self.ctx.emit(oc::GEQ, Value::Null);
        let exit_jump = self.ctx.emit(oc::JUMP_IF_TRUE, Value::Null);
        self.ctx.emit(oc::LOAD_LOCAL, json!(it_slot));
        self.ctx.emit(oc::LOAD_LOCAL, json!(idx_slot));
        self.ctx.emit(oc::LOAD_INDEX, json!(1));
        self.store_var(var);
        for st in body { self.stmt(st)?; }
        let inc_target = self.ctx.here();
        let cont = self.ctx.continue_patches.pop().unwrap().0;
        for ip in cont { self.ctx.patch(ip, inc_target); }
        self.ctx.emit(oc::LOAD_LOCAL, json!(idx_slot));
        let one = self.ctx.add_const(json!(1));
        self.ctx.emit(oc::LOAD_CONST, json!(one));
        self.ctx.emit(oc::ADD, Value::Null);
        self.ctx.emit(oc::STORE_LOCAL, json!(idx_slot));
        self.ctx.emit(oc::JUMP, json!(loop_start));
        let end = self.ctx.here();
        self.ctx.patch(exit_jump, end);
        let brk = self.ctx.break_patches.pop().unwrap().0;
        for ip in brk { self.ctx.patch(ip, end); }
        Ok(())
    }

    /// REPEAT body UNTIL cond -- Post-Test-Loop (laeuft mind. einmal).
    fn stmt_repeat(&mut self, body: &[Node], condition: &Node) -> CR {
        let loop_start = self.ctx.here();
        self.break_continue_enter();
        for st in body { self.stmt(st)?; }
        let cond_pos = self.ctx.here();
        let cont = self.ctx.continue_patches.pop().unwrap().0;
        for ip in cont { self.ctx.patch(ip, cond_pos); }
        self.expr(condition)?;
        self.ctx.emit(oc::JUMP_IF_FALSE, json!(loop_start));   // TRUE = abbrechen
        let end = self.ctx.here();
        let brk = self.ctx.break_patches.pop().unwrap().0;
        for ip in brk { self.ctx.patch(ip, end); }
        Ok(())
    }

    /// TRY body CATCH [e] catch_block END TRY.
    fn stmt_try(&mut self, body: &[Node], catch_var: &str, catch_block: &[Node]) -> CR {
        let catch_jmp = self.ctx.emit(oc::TRY_BEGIN, Value::Null);
        self.ctx.try_depth += 1;
        for st in body { self.stmt(st)?; }
        self.ctx.try_depth -= 1;
        self.ctx.emit(oc::TRY_END, Value::Null);
        let end_jmp = self.ctx.emit(oc::JUMP, Value::Null);
        // Catch-Branch: der (String-)Exception-Wert liegt oben auf dem Stack.
        let ct = self.ctx.here();
        self.ctx.patch(catch_jmp, ct);
        if !catch_var.is_empty() {
            if self.ctx.is_main {
                if !self.ctx.local_slots.contains_key(catch_var) {
                    let name_idx = self.ctx.add_const(json!(catch_var));
                    let type_idx = self.ctx.add_const(json!("string"));
                    let default_idx = self.ctx.add_const(json!(""));
                    self.ctx.emit(oc::DECLARE_NAME, json!([name_idx, type_idx, default_idx]));
                }
            } else if !self.ctx.local_slots.contains_key(catch_var) {
                self.ctx.declare_local(catch_var, "string");
            }
            self.store_var(catch_var);
        } else {
            self.ctx.emit(oc::POP, Value::Null);
        }
        for st in catch_block { self.stmt(st)?; }
        let e = self.ctx.here();
        self.ctx.patch(end_jmp, e);
        Ok(())
    }

    /// WITH expr / body / END WITH -- Ziel in anonymen Slot, `.member` im
    /// Body wurde vom Parser zu `<var_name>.member` desugart.
    fn stmt_with(&mut self, var_name: &str, target: &Node, body: &[Node]) -> CR {
        let slot = self.ctx.alloc_anon_slot("any");
        self.expr(target)?;
        self.ctx.emit(oc::STORE_LOCAL, json!(slot));
        self.ctx.local_slots.insert(var_name.to_string(), slot);
        let r = (|| -> CR { for st in body { self.stmt(st)?; } Ok(()) })();
        self.ctx.local_slots.remove(var_name);
        r
    }

    /// (t1, ..., tn) = expr -- UNPACK_TUPLE + Store je Ziel.
    fn stmt_tuple_assign(&mut self, targets: &[Node], value: &Node) -> CR {
        self.expr(value)?;
        self.ctx.emit(oc::UNPACK_TUPLE, json!(targets.len()));
        for tgt in targets {
            match tgt {
                Node::Identifier(name) => self.store_var(name),
                Node::MemberAccess { target, name } => {
                    let tmp = self.ctx.alloc_anon_slot("any");
                    self.ctx.emit(oc::STORE_LOCAL, json!(tmp));
                    self.expr(target)?;
                    self.ctx.emit(oc::LOAD_LOCAL, json!(tmp));
                    let idx = self.ctx.add_const(json!(name));
                    self.ctx.emit(oc::STORE_MEMBER, json!(idx));
                }
                Node::IndexAccess { target, indices } => {
                    let tmp = self.ctx.alloc_anon_slot("any");
                    self.ctx.emit(oc::STORE_LOCAL, json!(tmp));
                    self.expr(target)?;
                    for ix in indices { self.expr(ix)?; }
                    self.ctx.emit(oc::LOAD_LOCAL, json!(tmp));
                    self.ctx.emit(oc::STORE_INDEX, json!(indices.len()));
                }
                _ => return Err("Ungueltiges Tupel-Assignment-Ziel".into()),
            }
        }
        Ok(())
    }

    /// List-Comprehension: Marker + Index-Loop + BUILD_TUPLE_DYN.
    fn expr_listcomp(&mut self, var: &str, iterable: &Node,
                     filter: &Option<Box<Node>>, transform: &Node) -> CR {
        let m = self.ctx.add_const(json!({ "comp": true }));
        self.ctx.emit(oc::LOAD_CONST, json!(m));
        let iter_slot = self.ctx.alloc_anon_slot("any");
        self.expr(iterable)?;
        self.ctx.emit(oc::CALL_BUILTIN, json!(["__comp_iter", 1]));
        self.ctx.emit(oc::STORE_LOCAL, json!(iter_slot));
        self.ctx.emit(oc::LOAD_LOCAL, json!(iter_slot));
        self.ctx.emit(oc::CALL_BUILTIN, json!(["len", 1]));
        let len_slot = self.ctx.alloc_anon_slot("integer");
        self.ctx.emit(oc::STORE_LOCAL, json!(len_slot));
        let cnt_slot = self.ctx.alloc_anon_slot("integer");
        let z = self.ctx.add_const(json!(0));
        self.ctx.emit(oc::LOAD_CONST, json!(z));
        self.ctx.emit(oc::STORE_LOCAL, json!(cnt_slot));
        let prev = self.ctx.local_slots.get(var).copied();
        let var_slot = self.ctx.alloc_anon_slot("any");
        self.ctx.local_slots.insert(var.to_string(), var_slot);
        let r = (|| -> CR {
            let loop_start = self.ctx.here();
            self.ctx.emit(oc::LOAD_LOCAL, json!(cnt_slot));
            self.ctx.emit(oc::LOAD_LOCAL, json!(len_slot));
            self.ctx.emit(oc::LT, Value::Null);
            let cond_jump = self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null);
            self.ctx.emit(oc::LOAD_LOCAL, json!(iter_slot));
            self.ctx.emit(oc::LOAD_LOCAL, json!(cnt_slot));
            self.ctx.emit(oc::LOAD_INDEX, json!(1));
            self.ctx.emit(oc::STORE_LOCAL, json!(var_slot));
            let mut skip_jump: Option<usize> = None;
            if let Some(f) = filter {
                self.expr(f)?;
                skip_jump = Some(self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null));
            }
            self.expr(transform)?;
            if let Some(sj) = skip_jump { let t = self.ctx.here(); self.ctx.patch(sj, t); }
            self.ctx.emit(oc::LOAD_LOCAL, json!(cnt_slot));
            let one = self.ctx.add_const(json!(1));
            self.ctx.emit(oc::LOAD_CONST, json!(one));
            self.ctx.emit(oc::ADD, Value::Null);
            self.ctx.emit(oc::STORE_LOCAL, json!(cnt_slot));
            self.ctx.emit(oc::JUMP, json!(loop_start));
            let end_pos = self.ctx.here();
            self.ctx.patch(cond_jump, end_pos);
            self.ctx.emit(oc::BUILD_TUPLE_DYN, Value::Null);
            Ok(())
        })();
        match prev {
            Some(p) => { self.ctx.local_slots.insert(var.to_string(), p); }
            None => { self.ctx.local_slots.remove(var); }
        }
        r
    }

    /// Dict-Comprehension: pro Iteration ein 2-Tupel (key, value); am Ende
    /// __dict_from_pairs.
    fn expr_dictcomp(&mut self, var: &str, iterable: &Node,
                     filter: &Option<Box<Node>>, key: &Node, value: &Node) -> CR {
        let m = self.ctx.add_const(json!({ "comp": true }));
        self.ctx.emit(oc::LOAD_CONST, json!(m));
        let iter_slot = self.ctx.alloc_anon_slot("any");
        self.expr(iterable)?;
        self.ctx.emit(oc::CALL_BUILTIN, json!(["__comp_iter", 1]));
        self.ctx.emit(oc::STORE_LOCAL, json!(iter_slot));
        self.ctx.emit(oc::LOAD_LOCAL, json!(iter_slot));
        self.ctx.emit(oc::CALL_BUILTIN, json!(["len", 1]));
        let len_slot = self.ctx.alloc_anon_slot("integer");
        self.ctx.emit(oc::STORE_LOCAL, json!(len_slot));
        let cnt_slot = self.ctx.alloc_anon_slot("integer");
        let z = self.ctx.add_const(json!(0));
        self.ctx.emit(oc::LOAD_CONST, json!(z));
        self.ctx.emit(oc::STORE_LOCAL, json!(cnt_slot));
        let prev = self.ctx.local_slots.get(var).copied();
        let var_slot = self.ctx.alloc_anon_slot("any");
        self.ctx.local_slots.insert(var.to_string(), var_slot);
        let r = (|| -> CR {
            let loop_start = self.ctx.here();
            self.ctx.emit(oc::LOAD_LOCAL, json!(cnt_slot));
            self.ctx.emit(oc::LOAD_LOCAL, json!(len_slot));
            self.ctx.emit(oc::LT, Value::Null);
            let cond_jump = self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null);
            self.ctx.emit(oc::LOAD_LOCAL, json!(iter_slot));
            self.ctx.emit(oc::LOAD_LOCAL, json!(cnt_slot));
            self.ctx.emit(oc::LOAD_INDEX, json!(1));
            self.ctx.emit(oc::STORE_LOCAL, json!(var_slot));
            let mut skip_jump: Option<usize> = None;
            if let Some(f) = filter {
                self.expr(f)?;
                skip_jump = Some(self.ctx.emit(oc::JUMP_IF_FALSE, Value::Null));
            }
            self.expr(key)?;
            self.expr(value)?;
            self.ctx.emit(oc::BUILD_TUPLE, json!(2));
            if let Some(sj) = skip_jump { let t = self.ctx.here(); self.ctx.patch(sj, t); }
            self.ctx.emit(oc::LOAD_LOCAL, json!(cnt_slot));
            let one = self.ctx.add_const(json!(1));
            self.ctx.emit(oc::LOAD_CONST, json!(one));
            self.ctx.emit(oc::ADD, Value::Null);
            self.ctx.emit(oc::STORE_LOCAL, json!(cnt_slot));
            self.ctx.emit(oc::JUMP, json!(loop_start));
            let end_pos = self.ctx.here();
            self.ctx.patch(cond_jump, end_pos);
            self.ctx.emit(oc::BUILD_TUPLE_DYN, Value::Null);
            self.ctx.emit(oc::CALL_BUILTIN, json!(["__dict_from_pairs", 1]));
            Ok(())
        })();
        match prev {
            Some(p) => { self.ctx.local_slots.insert(var.to_string(), p); }
            None => { self.ctx.local_slots.remove(var); }
        }
        r
    }

    fn finish(self, data: Vec<Value>) -> Value {
        let main = build_func(&self.ctx, "__main__", true, true, 0, 0,
                              false, false, "", &[], &[]);
        let mut functions = Map::new();
        for (name, fnj) in self.compiled_fns {
            functions.insert(name, fnj);
        }
        let mut classes = Map::new();
        for (cname, ci) in &self.classes {
            let mut methods = Map::new();
            for (mn, mj) in &ci.compiled { methods.insert(mn.clone(), mj.clone()); }
            let fields: Vec<Value> = ci.fields.iter().map(|f| json!({
                "name": f.name, "type_name": f.type_name, "array_dims": f.array_dims,
            })).collect();
            let mut props = ci.property_set.clone();
            props.sort();
            classes.insert(cname.clone(), json!({
                "name": ci.name, "parent_name": ci.parent_name, "is_struct": ci.is_struct,
                "fields": fields, "methods": Value::Object(methods), "properties": props,
            }));
        }
        json!({
            "format": "gbc", "version": 1,
            "n_globals": self.global_slots.len(),
            "main": main, "functions": Value::Object(functions),
            "classes": Value::Object(classes), "data": data,
        })
    }

    // ---------------------------------------------------- User-Funktionen
    fn register_stub(&mut self, decl: &Node) -> Result<(), String> {
        let (name, _, _, _, _) = fn_parts(decl);
        if self.fn_sigs.contains_key(name) {
            return Err(format!("'{}' bereits deklariert", name));
        }
        let sig = make_sig(decl)?;
        self.fn_sigs.insert(name.to_string(), sig);
        Ok(())
    }

    // ---------------------------------------------------- Klassen
    fn register_class(&mut self, decl: &Node) -> Result<(), String> {
        let (name, parent, fields, methods, is_struct, properties) = match decl {
            Node::ClassDecl { name, parent, fields, methods, is_struct, properties, .. } =>
                (name, parent, fields, methods, *is_struct, properties),
            _ => unreachable!(),
        };
        if self.classes.contains_key(name) {
            return Err(format!("Klasse '{}' bereits deklariert", name));
        }
        let mut field_infos = Vec::new();
        for f in fields {
            if let Node::Dim { name: fname, type_name, array_dims } = f {
                let dims: Vec<i64> = match array_dims {
                    None => vec![],
                    Some(des) => {
                        let mut v = vec![];
                        for de in des {
                            match de {
                                Node::NumberLit(NumV::Int(i)) => v.push(*i),
                                _ => return Err(format!(
                                    "Array-Feld '{}' braucht konstante INTEGER-Groesse", fname)),
                            }
                        }
                        v
                    }
                };
                field_infos.push(FieldInfo { name: fname.clone(),
                    type_name: type_name.clone(), array_dims: dims });
            }
        }
        let mut method_sigs = HashMap::new();
        for m in methods {
            let (mname, _, _, _, _) = fn_parts(m);
            method_sigs.insert(mname.to_string(), make_sig(m)?);
        }
        let property_set: Vec<String> = properties.iter().filter_map(|p| match p {
            Node::PropertyDecl { name, .. } => Some(name.to_lowercase()),
            _ => None,
        }).collect();
        self.classes.insert(name.clone(), ClassInfo {
            name: name.clone(),
            parent_name: parent.clone().unwrap_or_default(),
            is_struct,
            fields: field_infos,
            method_sigs,
            property_set,
            compiled: vec![],
        });
        Ok(())
    }

    fn compile_class_methods(&mut self, decl: &Node) -> Result<(), String> {
        let (cname, methods) = match decl {
            Node::ClassDecl { name, methods, .. } => (name.clone(), methods),
            _ => unreachable!(),
        };
        for m in methods {
            let (mname, params, body, is_sub, _rt) = fn_parts(m);
            let mut ctx = Ctx::new();
            ctx.is_main = false;
            ctx.is_sub = is_sub;
            ctx.current_class = Some(cname.clone());
            for p in params {
                let slot = ctx.local_types.len();
                ctx.local_slots.insert(p.name.clone(), slot);
                ctx.local_types.push(p.type_name.clone());
                ctx.local_defaults.push(type_default(&p.type_name));
            }
            let saved = std::mem::replace(&mut self.ctx, ctx);
            let r = (|| {
                for s in body { self.stmt(s)?; }
                if is_sub {
                    self.ctx.emit(oc::RETURN_VOID, Value::Null);
                } else {
                    let c = self.ctx.add_const(json!(format!("__missing_return:{}", mname)));
                    self.ctx.emit(oc::LOAD_CONST, json!(c));
                    self.ctx.emit(oc::HALT, Value::Null);
                }
                Ok::<(), String>(())
            })();
            let fn_ctx = std::mem::replace(&mut self.ctx, saved);
            r?;
            let sig = &self.classes[&cname].method_sigs[mname];
            let fnj = build_func(&fn_ctx, mname, false, is_sub, sig.n_params, sig.n_required,
                                 sig.is_variadic, sig.is_coroutine, &sig.return_type,
                                 &sig.param_defaults, &sig.param_names);
            self.classes.get_mut(&cname).unwrap().compiled.push((mname.to_string(), fnj));
        }
        Ok(())
    }

    fn emit_class_statics(&mut self, decl: &Node) -> Result<(), String> {
        let (name, statics) = match decl {
            Node::ClassDecl { name, statics, .. } => (name.clone(), statics),
            _ => unreachable!(),
        };
        if statics.is_empty() { return Ok(()); }
        let mut members: Vec<(String, CVal)> = Vec::new();
        for c in statics {
            if let Node::Const { name: cn, value, .. } = c {
                members.push((cn.to_lowercase(), eval_literal_default(value)?));
            }
        }
        let ns = CVal::Ns { name: name.clone(), members };
        self.emit_namespace_const(&name, ns);
        Ok(())
    }

    fn compile_function(&mut self, decl: &Node) -> Result<(), String> {
        let (name, params, body, is_sub, _rt) = fn_parts(decl);
        let mut ctx = Ctx::new();
        ctx.is_main = false;
        ctx.is_sub = is_sub;
        for p in params {
            let slot = ctx.local_types.len();
            ctx.local_slots.insert(p.name.clone(), slot);
            ctx.local_types.push(p.type_name.clone());
            ctx.local_defaults.push(type_default(&p.type_name));
        }
        let saved = std::mem::replace(&mut self.ctx, ctx);
        let r = (|| {
            for s in body { self.stmt(s)?; }
            if is_sub {
                self.ctx.emit(oc::RETURN_VOID, Value::Null);
            } else {
                let c = self.ctx.add_const(json!(format!("__missing_return:{}", name)));
                self.ctx.emit(oc::LOAD_CONST, json!(c));
                self.ctx.emit(oc::HALT, Value::Null);
            }
            Ok::<(), String>(())
        })();
        let fn_ctx = std::mem::replace(&mut self.ctx, saved);
        r?;
        let sig = &self.fn_sigs[name];
        let fnj = build_func(&fn_ctx, name, false, is_sub, sig.n_params, sig.n_required,
                             sig.is_variadic, sig.is_coroutine, &sig.return_type,
                             &sig.param_defaults, &sig.param_names);
        self.compiled_fns.push((name.to_string(), fnj));
        Ok(())
    }

    fn resolve_named_args<'a>(&self, name: &str, args: &'a [Node])
        -> Result<Vec<RArg<'a>>, String> {
        resolve_args_with_sig(&self.fn_sigs[name], name, args)
    }
}

enum RArg<'a> { Expr(&'a Node), Default(CVal) }

/// Mappt (positional + named) Argumente auf die Param-Reihenfolge der Signatur,
/// fuellt Defaults. Wie compiler._resolve_named_args.
fn resolve_args_with_sig<'a>(sig: &FnSig, name: &str, args: &'a [Node])
    -> Result<Vec<RArg<'a>>, String> {
    let n = sig.n_params;
    let mut slots: Vec<Option<RArg<'a>>> = (0..n).map(|_| None).collect();
    let pos_count = args.iter().position(|a| matches!(a, Node::NamedArg { .. }))
        .unwrap_or(args.len());
    if pos_count > n {
        return Err(format!("{}: zu viele Argumente (max {})", name, n));
    }
    for j in pos_count..args.len() {
        if !matches!(args[j], Node::NamedArg { .. }) {
            return Err(format!("{}: positional Argument nach Named-Arg", name));
        }
    }
    for i in 0..pos_count { slots[i] = Some(RArg::Expr(&args[i])); }
    for j in pos_count..args.len() {
        if let Node::NamedArg { name: an, value } = &args[j] {
            let key = an.to_lowercase();
            let idx = sig.param_names.iter().position(|p| *p == key)
                .ok_or_else(|| format!("{}: kein Parameter '{}'", name, an))?;
            if slots[idx].is_some() {
                return Err(format!("{}: Parameter '{}' doppelt belegt", name, an));
            }
            slots[idx] = Some(RArg::Expr(value));
        }
    }
    let mut actions = Vec::new();
    for i in 0..n {
        match slots[i].take() {
            Some(a) => actions.push(a),
            None => match sig.param_defaults.get(i).and_then(|d| d.clone()) {
                Some(cv) => actions.push(RArg::Default(cv)),
                None => return Err(format!("{}: Parameter '{}' fehlt", name,
                    sig.param_names.get(i).cloned().unwrap_or_default())),
            },
        }
    }
    Ok(actions)
}

/// Integer-Literal (NumberLit int / UnaryOp +/-) oder None. Fuer ENUM-Member.
fn eval_int_literal(e: &Node) -> Option<i64> {
    match e {
        Node::NumberLit(NumV::Int(i)) => Some(*i),
        Node::UnaryOp { op, operand } if op == "-" => eval_int_literal(operand).map(|i| -i),
        Node::UnaryOp { op, operand } if op == "+" => eval_int_literal(operand),
        _ => None,
    }
}

/// (name, params, body, is_sub, return_type) eines SUB/FUNCTION-Knotens.
fn fn_parts(decl: &Node) -> (&str, &[crate::ast::Param], &[Node], bool, &str) {
    match decl {
        Node::SubDecl { name, params, body } => (name, params, body, true, ""),
        Node::FunctionDecl { name, params, return_type, body } =>
            (name, params, body, false, return_type),
        _ => unreachable!("fn_parts auf Nicht-Funktion"),
    }
}

/// FnSig aus einem SUB/FUNCTION/Methoden-Knoten.
fn make_sig(decl: &Node) -> Result<FnSig, String> {
    let (name, params, body, is_sub, return_type) = fn_parts(decl);
    for p in params {
        if p.by_ref {
            return Err(format!("{}: BYREF-Parameter '{}' im VM-Pfad nicht unterstuetzt",
                               name, p.name));
        }
    }
    let mut param_defaults: Vec<Option<CVal>> = Vec::new();
    for p in params {
        param_defaults.push(match &p.default {
            Some(d) => Some(eval_literal_default(d)?),
            None => None,
        });
    }
    let n_required = param_defaults.iter().position(|d| d.is_some()).unwrap_or(params.len());
    let is_variadic = params.last().map(|p| p.is_variadic).unwrap_or(false);
    Ok(FnSig {
        n_params: params.len(),
        n_required,
        param_names: params.iter().map(|p| p.name.to_lowercase()).collect(),
        param_defaults,
        param_types: params.iter().map(|p| p.type_name.clone()).collect(),
        is_variadic,
        is_sub,
        return_type: if is_sub { String::new() } else { return_type.to_string() },
        is_coroutine: body_has_yield(body),
    })
}

fn eval_literal_default(e: &Node) -> Result<CVal, String> {
    match e {
        Node::NumberLit(NumV::Int(i)) => Ok(CVal::Int(*i)),
        Node::NumberLit(NumV::Float(f)) => Ok(CVal::Float(*f)),
        Node::StringLit(s) => Ok(CVal::Str(s.clone())),
        Node::BoolLit(b) => Ok(CVal::Bool(*b)),
        Node::UnaryOp { op, operand } if op == "-" => match eval_literal_default(operand)? {
            CVal::Int(i) => Ok(CVal::Int(-i)),
            CVal::Float(f) => Ok(CVal::Float(-f)),
            _ => Err("Default '-' nur auf Zahlen".into()),
        },
        _ => Err("Default-Parameter muessen Literale sein (VM-Pfad)".into()),
    }
}

fn body_has_yield(stmts: &[Node]) -> bool {
    fn ny(e: &Node) -> bool {
        match e {
            Node::Yield(_) => true,
            Node::BinaryOp { left, right, .. } => ny(left) || ny(right),
            Node::UnaryOp { operand, .. } => ny(operand),
            Node::Call { callee, args } => ny(callee) || args.iter().any(ny),
            Node::IndexAccess { target, indices } => ny(target) || indices.iter().any(ny),
            Node::TernaryExpr { cond, then_expr, else_expr } =>
                ny(cond) || ny(then_expr) || ny(else_expr),
            _ => false,
        }
    }
    fn ns(s: &Node) -> bool {
        match s {
            Node::ExprStmt { expr } | Node::Throw { value: expr } => ny(expr),
            Node::Assign { value, .. } | Node::Const { value, .. } => ny(value),
            Node::Return(Some(v)) => ny(v),
            Node::If { then_block, elseif_branches, else_block, .. } =>
                then_block.iter().any(ns)
                || elseif_branches.iter().any(|(_, b)| b.iter().any(ns))
                || else_block.iter().any(ns),
            Node::While { body, .. } | Node::For { body, .. }
            | Node::Repeat { body, .. } | Node::ForEach { body, .. } => body.iter().any(ns),
            _ => false,
        }
    }
    stmts.iter().any(ns)
}

/// Baut das `_enc_func`-JSON aus einem fertig kompilierten Ctx.
#[allow(clippy::too_many_arguments)]
fn build_func(ctx: &Ctx, name: &str, is_main: bool, is_sub: bool,
              n_params: usize, n_required: usize, is_variadic: bool,
              is_coroutine: bool, return_type: &str,
              param_defaults: &[Option<CVal>], param_names: &[String]) -> Value {
    let code: Vec<Value> = ctx.code.iter().map(|(op, arg)| json!([op, arg])).collect();
    let lines: Vec<Value> = ctx.code.iter().map(|_| json!(0)).collect();
    let local_defaults: Vec<Value> = ctx.local_defaults.iter().map(enc).collect();
    let pdef: Vec<Value> = param_defaults.iter()
        .map(|d| match d { Some(cv) => enc(cv), None => Value::Null }).collect();
    json!({
        "name": name, "n_params": n_params, "n_required": n_required,
        "is_variadic": is_variadic, "is_sub": is_sub, "is_main": is_main,
        "is_coroutine": is_coroutine, "return_type": return_type,
        "param_defaults": pdef, "param_names": param_names,
        "local_types": ctx.local_types.clone(), "local_defaults": local_defaults,
        "constants": ctx.consts.clone(), "code": code, "lines": lines,
    })
}

/// DATA-Literale rekursiv einsammeln (wie compiler._collect_data, 3b-Subset).
fn collect_data(stmts: &[Node], out: &mut Vec<Value>) -> Result<(), String> {
    for s in stmts {
        match s {
            Node::Data { values } => {
                for lit in values { out.push(enc(&data_literal(lit)?)); }
            }
            Node::If { then_block, elseif_branches, else_block, .. } => {
                collect_data(then_block, out)?;
                for (_, b) in elseif_branches { collect_data(b, out)?; }
                collect_data(else_block, out)?;
            }
            // Hinweis: 1:1 zu compiler._collect_data -- KEIN ForEach/With
            // (Python sammelt dort kein DATA), aber Select + Try.
            Node::While { body, .. } | Node::For { body, .. }
            | Node::Repeat { body, .. }
            | Node::SubDecl { body, .. } | Node::FunctionDecl { body, .. } =>
                collect_data(body, out)?,
            Node::Select { cases, else_block, .. } => {
                for (_, _, blk) in cases { collect_data(blk, out)?; }
                collect_data(else_block, out)?;
            }
            Node::Try { body, catch_block, .. } => {
                collect_data(body, out)?;
                collect_data(catch_block, out)?;
            }
            Node::ClassDecl { methods, .. } => {
                for m in methods {
                    if let Node::SubDecl { body, .. } | Node::FunctionDecl { body, .. } = m {
                        collect_data(body, out)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn data_literal(lit: &Node) -> Result<CVal, String> {
    match lit {
        Node::NumberLit(NumV::Int(i)) => Ok(CVal::Int(*i)),
        Node::NumberLit(NumV::Float(f)) => Ok(CVal::Float(*f)),
        Node::StringLit(s) => Ok(CVal::Str(s.clone())),
        Node::BoolLit(b) => Ok(CVal::Bool(*b)),
        Node::UnaryOp { op, operand } if op == "-" => match data_literal(operand)? {
            CVal::Int(i) => Ok(CVal::Int(-i)),
            CVal::Float(f) => Ok(CVal::Float(-f)),
            _ => Err("DATA: '-' nur auf Zahlen".into()),
        },
        _ => Err("DATA: ungueltiges Literal".into()),
    }
}

/// Bekannter skalarer Werttyp (= compiler._TYPE_DEFAULTS-Schluessel). Klassen
/// und (kuenftig) externe Modul-Typen werden separat geprueft.
fn is_value_type(t: &str) -> bool {
    matches!(t, "integer" | "float" | "string" | "boolean"
        | "image" | "sound" | "sprite_atlas" | "file"
        | "tuple" | "funcref" | "coroutine")
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

/// AST -> `.gbc`-JSON. `external_types` sind die von importierten Modulen
/// registrierten Typ-Namen (lowercase) -- damit `DIM x AS VEC2` & Co. nach
/// `IMPORT "vec2"` kompilieren. `builtin_aliases` sind `(alias, modul)`-Paare
/// fuer `IMPORT "json" AS j` (Builtin-Namen-Rueckabbildung). Beide aus
/// preprocess::compile_env. Fehler bei nicht unterstuetzten Konstrukten.
pub fn compile_to_gbc(ast: &Node, external_types: &std::collections::HashSet<String>,
                      builtin_aliases: &[(String, String)])
    -> Result<Value, String> {
    let stmts = match ast {
        Node::Program { statements } => statements,
        _ => return Err("Erwartet Program-Knoten".into()),
    };
    // Funktionen / Klassen / Hauptprogramm trennen.
    let mut fn_decls: Vec<&Node> = vec![];
    let mut cls_decls: Vec<&Node> = vec![];
    let mut main_stmts: Vec<&Node> = vec![];
    for s in stmts {
        match s {
            Node::SubDecl { .. } | Node::FunctionDecl { .. } => fn_decls.push(s),
            Node::ClassDecl { .. } => cls_decls.push(s),
            _ => main_stmts.push(s),
        }
    }
    let main_owned: Vec<Node> = main_stmts.iter().map(|s| (*s).clone()).collect();
    let mut c = Compiler::new(external_types.clone(), builtin_aliases.to_vec());
    // Struct-Namen vor dem Slot-Pre-Pass kennen (Structs bekommen keinen Slot).
    for cd in &cls_decls {
        if let Node::ClassDecl { name, is_struct: true, .. } = cd {
            c.struct_names.insert(name.clone());
        }
    }
    c.collect_globals(&main_owned)?;
    // Klassen-Statics bekommen einen Slot.
    for cd in &cls_decls {
        if let Node::ClassDecl { name, statics, .. } = cd {
            if !statics.is_empty() { c.alloc_slot(name); }
        }
    }
    // Phase 1: Klassen registrieren (Felder/Methoden-Sigs/Properties).
    for cd in &cls_decls { c.register_class(cd)?; }
    // Phase 2/3: Funktions-Stubs + -Bodies (Forward-Refs/Rekursion).
    for d in &fn_decls { c.register_stub(d)?; }
    for d in &fn_decls { c.compile_function(d)?; }
    // Phase 4: Methoden kompilieren.
    for cd in &cls_decls { c.compile_class_methods(cd)?; }
    // Phase 5: Hauptprogramm (Klassen-Statics zuerst hoisten).
    for cd in &cls_decls { c.emit_class_statics(cd)?; }
    for s in &main_owned { c.stmt(s)?; }
    c.ctx.emit(oc::HALT, Value::Null);
    let mut data = Vec::new();
    collect_data(stmts, &mut data)?;
    Ok(c.finish(data))
}
