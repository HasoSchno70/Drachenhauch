//! AST-Knoten fuer GameBasic -- Rust-Port von `gamebasic/ast_nodes.py`.
//! Stufe 2 der Front-End-Portierung (siehe docs/rust-frontend-port.md).
//!
//! `to_json()` emittiert exakt die Struktur, die der Python-Test aus den
//! Dataclass-Feldern baut (`{"_": NodeName, feld: ...}`), damit der Parser
//! gegen Python verifiziert werden kann (`gbrt --ast`). Tupel (z.B.
//! `If.elseif_branches`) werden als JSON-Arrays serialisiert -- wie Pythons
//! Tuple→list. `line` ist in Python KEIN Dataclass-Feld (dynamisch gesetzt)
//! und wird daher hier nicht getrackt/serialisiert.

use serde_json::{json, Map, Value};

#[derive(Clone, Debug)]
pub enum NumV {
    Int(i64),
    Float(f64),
}

impl NumV {
    fn to_json(&self) -> Value {
        match self {
            NumV::Int(i) => Value::from(*i),
            NumV::Float(f) => Value::from(*f),
        }
    }
}

/// Ein Parameter (Python-Dataclass `Param`).
#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub type_name: String,
    pub default: Option<Node>,
    pub by_ref: bool,
    pub is_variadic: bool,
}

impl Param {
    fn to_json(&self) -> Value {
        json!({
            "_": "Param",
            "name": self.name,
            "type_name": self.type_name,
            "default": opt(&self.default),
            "by_ref": self.by_ref,
            "is_variadic": self.is_variadic,
        })
    }
}

/// Element von `CaseMatch.values`: bei `CASE IS <op>` ist das erste Element ein
/// roher Operator-String (kein Knoten!), sonst ein Ausdruck -- wie in Python.
#[derive(Clone, Debug)]
pub enum CaseVal {
    Op(String),
    Expr(Node),
}

/// Ein Match innerhalb eines CASE (Python-Dataclass `CaseMatch`).
#[derive(Clone, Debug)]
pub struct CaseMatch {
    pub kind: String,        // "value" | "range" | "is"
    pub values: Vec<CaseVal>,
}

impl CaseMatch {
    fn to_json(&self) -> Value {
        let vals: Vec<Value> = self.values.iter().map(|v| match v {
            CaseVal::Op(s) => json!(s),
            CaseVal::Expr(n) => n.to_json(),
        }).collect();
        json!({ "_": "CaseMatch", "kind": self.kind, "values": Value::Array(vals) })
    }
}

#[derive(Clone, Debug)]
pub enum Node {
    // --- Ausdruecke ---
    NumberLit(NumV),
    StringLit(String),
    BoolLit(bool),
    Identifier(String),
    BinaryOp { op: String, left: Box<Node>, right: Box<Node> },
    UnaryOp { op: String, operand: Box<Node> },
    TernaryExpr { cond: Box<Node>, then_expr: Box<Node>, else_expr: Box<Node> },
    Yield(Option<Box<Node>>),
    Call { callee: Box<Node>, args: Vec<Node> },
    ListComp { var: String, iterable: Box<Node>, filter: Option<Box<Node>>, transform: Box<Node> },
    DictComp { var: String, iterable: Box<Node>, filter: Option<Box<Node>>, key: Box<Node>, value: Box<Node> },
    SetComp { var: String, iterable: Box<Node>, filter: Option<Box<Node>>, transform: Box<Node> },
    TupleLit { elements: Vec<Node> },
    NamedArg { name: String, value: Box<Node> },
    New { class_name: String, args: Option<Vec<Node>> },
    MemberAccess { target: Box<Node>, name: String },
    IndexAccess { target: Box<Node>, indices: Vec<Node> },
    SliceAccess { target: Box<Node>, lo: Option<Box<Node>>, hi: Option<Box<Node>> },

    // --- Anweisungen ---
    Dim { name: String, type_name: String, array_dims: Option<Vec<Node>> },
    MultiDim { dims: Vec<Node> },
    Const { name: String, type_name: Option<String>, value: Box<Node> },
    Break,
    Continue,
    Restore,
    IndexAssign { target: Box<Node>, indices: Vec<Node>, value: Box<Node> },
    TupleAssign { targets: Vec<Node>, value: Box<Node> },
    Try { body: Vec<Node>, catch_var: String, catch_block: Vec<Node> },
    Throw { value: Box<Node> },
    Assign { name: String, value: Box<Node> },
    Print { items: Vec<Node>, newline: bool },
    Input { prompt: Option<Box<Node>>, target: String },
    If { condition: Box<Node>, then_block: Vec<Node>,
         elseif_branches: Vec<(Node, Vec<Node>)>, else_block: Vec<Node> },
    Select { subject: Box<Node>, cases: Vec<(Vec<CaseMatch>, Option<Node>, Vec<Node>)>,
             else_block: Vec<Node> },
    While { condition: Box<Node>, body: Vec<Node> },
    For { var: String, start: Box<Node>, end: Box<Node>, step: Option<Box<Node>>, body: Vec<Node> },
    ForEach { var: String, iterable: Box<Node>, body: Vec<Node> },
    Repeat { body: Vec<Node>, condition: Box<Node> },
    Data { values: Vec<Node> },
    Read { targets: Vec<Node> },
    EnumDecl { name: String, members: Vec<(String, Option<Node>)> },
    ExprStmt { expr: Box<Node> },
    SubDecl { name: String, params: Vec<Param>, body: Vec<Node> },
    FunctionDecl { name: String, params: Vec<Param>, return_type: String, body: Vec<Node> },
    Return(Option<Box<Node>>),
    PropertyDecl { name: String, kind: String, func: Box<Node> },
    ClassDecl { name: String, parent: Option<String>, fields: Vec<Node>, methods: Vec<Node>,
                is_struct: bool, statics: Vec<Node>, properties: Vec<Node> },
    MemberAssign { target: Box<Node>, name: String, value: Box<Node> },
    With { var_name: String, target: Box<Node>, body: Vec<Node> },
    Program { statements: Vec<Node> },
}

fn opt(o: &Option<Node>) -> Value {
    match o { Some(n) => n.to_json(), None => Value::Null }
}
fn bopt(o: &Option<Box<Node>>) -> Value {
    match o { Some(n) => n.to_json(), None => Value::Null }
}
fn vecj(v: &[Node]) -> Value {
    Value::Array(v.iter().map(|n| n.to_json()).collect())
}

/// Baut ein Knoten-Objekt `{"_": tag, ...felder}` aus (Schluessel, Wert)-Paaren.
fn obj(tag: &str, fields: Vec<(&str, Value)>) -> Value {
    let mut m = Map::new();
    m.insert("_".into(), Value::String(tag.into()));
    for (k, v) in fields {
        m.insert(k.into(), v);
    }
    Value::Object(m)
}

impl Node {
    pub fn to_json(&self) -> Value {
        use Node::*;
        match self {
            NumberLit(v) => obj("NumberLit", vec![("value", v.to_json())]),
            StringLit(s) => obj("StringLit", vec![("value", json!(s))]),
            BoolLit(b) => obj("BoolLit", vec![("value", json!(b))]),
            Identifier(n) => obj("Identifier", vec![("name", json!(n))]),
            BinaryOp { op, left, right } => obj("BinaryOp", vec![
                ("op", json!(op)), ("left", left.to_json()), ("right", right.to_json())]),
            UnaryOp { op, operand } => obj("UnaryOp", vec![
                ("op", json!(op)), ("operand", operand.to_json())]),
            TernaryExpr { cond, then_expr, else_expr } => obj("TernaryExpr", vec![
                ("cond", cond.to_json()), ("then_expr", then_expr.to_json()),
                ("else_expr", else_expr.to_json())]),
            Yield(v) => obj("Yield", vec![("value", bopt(v))]),
            Call { callee, args } => obj("Call", vec![
                ("callee", callee.to_json()), ("args", vecj(args))]),
            ListComp { var, iterable, filter, transform } => obj("ListComp", vec![
                ("var", json!(var)), ("iterable", iterable.to_json()),
                ("filter", bopt(filter)), ("transform", transform.to_json())]),
            DictComp { var, iterable, filter, key, value } => obj("DictComp", vec![
                ("var", json!(var)), ("iterable", iterable.to_json()),
                ("filter", bopt(filter)), ("key", key.to_json()), ("value", value.to_json())]),
            SetComp { var, iterable, filter, transform } => obj("SetComp", vec![
                ("var", json!(var)), ("iterable", iterable.to_json()),
                ("filter", bopt(filter)), ("transform", transform.to_json())]),
            TupleLit { elements } => obj("TupleLit", vec![("elements", vecj(elements))]),
            NamedArg { name, value } => obj("NamedArg", vec![
                ("name", json!(name)), ("value", value.to_json())]),
            New { class_name, args } => obj("New", vec![
                ("class_name", json!(class_name)),
                ("args", match args { Some(a) => vecj(a), None => Value::Null })]),
            MemberAccess { target, name } => obj("MemberAccess", vec![
                ("target", target.to_json()), ("name", json!(name))]),
            IndexAccess { target, indices } => obj("IndexAccess", vec![
                ("target", target.to_json()), ("indices", vecj(indices))]),
            SliceAccess { target, lo, hi } => obj("SliceAccess", vec![
                ("target", target.to_json()), ("lo", bopt(lo)), ("hi", bopt(hi))]),

            Dim { name, type_name, array_dims } => obj("Dim", vec![
                ("name", json!(name)), ("type_name", json!(type_name)),
                ("array_dims", match array_dims { Some(d) => vecj(d), None => Value::Null })]),
            MultiDim { dims } => obj("MultiDim", vec![("dims", vecj(dims))]),
            Const { name, type_name, value } => obj("Const", vec![
                ("name", json!(name)),
                ("type_name", match type_name { Some(t) => json!(t), None => Value::Null }),
                ("value", value.to_json())]),
            Break => obj("Break", vec![]),
            Continue => obj("Continue", vec![]),
            Restore => obj("Restore", vec![]),
            IndexAssign { target, indices, value } => obj("IndexAssign", vec![
                ("target", target.to_json()), ("indices", vecj(indices)),
                ("value", value.to_json())]),
            TupleAssign { targets, value } => obj("TupleAssign", vec![
                ("targets", vecj(targets)), ("value", value.to_json())]),
            Try { body, catch_var, catch_block } => obj("Try", vec![
                ("body", vecj(body)), ("catch_var", json!(catch_var)),
                ("catch_block", vecj(catch_block))]),
            Throw { value } => obj("Throw", vec![("value", value.to_json())]),
            Assign { name, value } => obj("Assign", vec![
                ("name", json!(name)), ("value", value.to_json())]),
            Print { items, newline } => obj("Print", vec![
                ("items", vecj(items)), ("newline", json!(newline))]),
            Input { prompt, target } => obj("Input", vec![
                ("prompt", bopt(prompt)), ("target", json!(target))]),
            If { condition, then_block, elseif_branches, else_block } => {
                let eifs: Vec<Value> = elseif_branches.iter()
                    .map(|(c, b)| Value::Array(vec![c.to_json(), vecj(b)]))
                    .collect();
                obj("If", vec![
                    ("condition", condition.to_json()), ("then_block", vecj(then_block)),
                    ("elseif_branches", Value::Array(eifs)), ("else_block", vecj(else_block))])
            }
            Select { subject, cases, else_block } => {
                let cs: Vec<Value> = cases.iter().map(|(matches, guard, block)| {
                    let m: Vec<Value> = matches.iter().map(|cm| cm.to_json()).collect();
                    Value::Array(vec![Value::Array(m),
                                      match guard { Some(g) => g.to_json(), None => Value::Null },
                                      vecj(block)])
                }).collect();
                obj("Select", vec![
                    ("subject", subject.to_json()), ("cases", Value::Array(cs)),
                    ("else_block", vecj(else_block))])
            }
            While { condition, body } => obj("While", vec![
                ("condition", condition.to_json()), ("body", vecj(body))]),
            For { var, start, end, step, body } => obj("For", vec![
                ("var", json!(var)), ("start", start.to_json()), ("end", end.to_json()),
                ("step", bopt(step)), ("body", vecj(body))]),
            ForEach { var, iterable, body } => obj("ForEach", vec![
                ("var", json!(var)), ("iterable", iterable.to_json()), ("body", vecj(body))]),
            Repeat { body, condition } => obj("Repeat", vec![
                ("body", vecj(body)), ("condition", condition.to_json())]),
            Data { values } => obj("Data", vec![("values", vecj(values))]),
            Read { targets } => obj("Read", vec![("targets", vecj(targets))]),
            EnumDecl { name, members } => {
                let ms: Vec<Value> = members.iter()
                    .map(|(n, e)| Value::Array(vec![json!(n), opt(e)]))
                    .collect();
                obj("EnumDecl", vec![("name", json!(name)), ("members", Value::Array(ms))])
            }
            ExprStmt { expr } => obj("ExprStmt", vec![("expr", expr.to_json())]),
            SubDecl { name, params, body } => obj("SubDecl", vec![
                ("name", json!(name)),
                ("params", Value::Array(params.iter().map(|p| p.to_json()).collect())),
                ("body", vecj(body))]),
            FunctionDecl { name, params, return_type, body } => obj("FunctionDecl", vec![
                ("name", json!(name)),
                ("params", Value::Array(params.iter().map(|p| p.to_json()).collect())),
                ("return_type", json!(return_type)), ("body", vecj(body))]),
            Return(v) => obj("Return", vec![("value", bopt(v))]),
            PropertyDecl { name, kind, func } => obj("PropertyDecl", vec![
                ("name", json!(name)), ("kind", json!(kind)), ("func", func.to_json())]),
            ClassDecl { name, parent, fields, methods, is_struct, statics, properties } =>
                obj("ClassDecl", vec![
                    ("name", json!(name)),
                    ("parent", match parent { Some(p) => json!(p), None => Value::Null }),
                    ("fields", vecj(fields)), ("methods", vecj(methods)),
                    ("is_struct", json!(is_struct)), ("statics", vecj(statics)),
                    ("properties", vecj(properties))]),
            MemberAssign { target, name, value } => obj("MemberAssign", vec![
                ("target", target.to_json()), ("name", json!(name)), ("value", value.to_json())]),
            With { var_name, target, body } => obj("With", vec![
                ("var_name", json!(var_name)), ("target", target.to_json()), ("body", vecj(body))]),
            Program { statements } => obj("Program", vec![("statements", vecj(statements))]),
        }
    }
}
