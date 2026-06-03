//! Laden des `.gbc`-Formats (siehe `gamebasic/serialize.py`) in Rust-Structs.
//!
//! Der const-Pool und die Code-Instruktionen werden beim Laden EINMALIG in
//! native Rust-Typen dekodiert (kein serde_json zur Laufzeit der Dispatch-
//! Schleife). Strukturelle Argumente (Slots, Indizes, Namen) bleiben als
//! `Arg` erhalten und werden pro Opcode interpretiert.

use std::rc::Rc;

use serde_json::Value as J;

use crate::value::Value;

/// Opcode-Konstanten -- muessen exakt zu `gamebasic/bytecode.py::Op` passen.
pub mod op {
    pub const LOAD_CONST: u16 = 1;
    pub const POP: u16 = 2;
    pub const DUP: u16 = 3;

    pub const LOAD_NAME: u16 = 10;
    pub const STORE_NAME: u16 = 11;
    pub const DECLARE_NAME: u16 = 12;
    pub const DECLARE_CONST: u16 = 13;
    pub const LOAD_LOCAL: u16 = 14;
    pub const STORE_LOCAL: u16 = 15;
    pub const DECLARE_LOCAL: u16 = 16;

    pub const ADD: u16 = 20;
    pub const SUB: u16 = 21;
    pub const MUL: u16 = 22;
    pub const DIV: u16 = 23;
    pub const MOD: u16 = 24;
    pub const POW: u16 = 25;
    pub const NEG: u16 = 26;
    pub const INT_DIV: u16 = 27;

    pub const EQ: u16 = 30;
    pub const NEQ: u16 = 31;
    pub const LT: u16 = 32;
    pub const GT: u16 = 33;
    pub const LEQ: u16 = 34;
    pub const GEQ: u16 = 35;
    pub const NOT: u16 = 36;

    pub const JUMP: u16 = 40;
    pub const JUMP_IF_FALSE: u16 = 41;
    pub const JUMP_IF_TRUE: u16 = 42;

    pub const CALL_USER: u16 = 50;
    pub const RETURN: u16 = 60;
    pub const RETURN_VOID: u16 = 61;

    pub const BAND: u16 = 62;
    pub const BOR: u16 = 63;
    pub const BXOR: u16 = 64;
    pub const SHL: u16 = 65;
    pub const SHR: u16 = 66;
    pub const BNOT: u16 = 67;

    // Spezialisierte Numeric-Numeric Ops.
    pub const ADD_NN: u16 = 100;
    pub const SUB_NN: u16 = 101;
    pub const MUL_NN: u16 = 102;
    pub const DIV_NN: u16 = 103;
    pub const LT_NN: u16 = 104;
    pub const GT_NN: u16 = 105;
    pub const LEQ_NN: u16 = 106;
    pub const GEQ_NN: u16 = 107;
    pub const EQ_NN: u16 = 108;
    pub const NEQ_NN: u16 = 109;
    pub const NEG_N: u16 = 110;

    pub const LOAD_GLOBAL_SLOT: u16 = 111;
    pub const STORE_GLOBAL_SLOT: u16 = 112;
    pub const DECLARE_GLOBAL_SLOT: u16 = 113;
    pub const DECLARE_GLOBAL_CONST_SLOT: u16 = 114;

    /// Coroutine: pop yield-Wert -> suspendieren -> push send-Wert beim Resume.
    pub const YIELD_VALUE: u16 = 115;

    pub const PRINT: u16 = 70;

    // Aufrufe / Werte
    pub const CALL_BUILTIN: u16 = 51;
    pub const CALL_METHOD: u16 = 52;
    pub const LOAD_FUNCREF: u16 = 53;
    pub const CALL_VALUE: u16 = 54;

    // Slice / IN / Comprehension
    pub const SLICE: u16 = 55;
    pub const IN_OP: u16 = 56;
    pub const BUILD_TUPLE_DYN: u16 = 57;

    pub const BUILD_TUPLE: u16 = 68;
    pub const UNPACK_TUPLE: u16 = 69;

    // OOP / Member
    pub const NEW_INSTANCE: u16 = 80;
    pub const LOAD_FIELD: u16 = 81;
    pub const STORE_FIELD: u16 = 82;
    pub const LOAD_MEMBER: u16 = 83;
    pub const STORE_MEMBER: u16 = 84;
    pub const DECLARE_STRUCT_NAME: u16 = 86;
    pub const DECLARE_STRUCT_LOCAL: u16 = 87;
    pub const LOAD_SELF: u16 = 88;

    // Arrays
    pub const LOAD_INDEX: u16 = 90;
    pub const STORE_INDEX: u16 = 91;
    pub const DECLARE_ARRAY_NAME: u16 = 92;
    pub const DECLARE_ARRAY_LOCAL: u16 = 93;

    // Exceptions
    pub const TRY_BEGIN: u16 = 95;
    pub const TRY_END: u16 = 96;
    pub const THROW: u16 = 97;

    // DATA / READ / RESTORE
    pub const PUSH_DATA: u16 = 75;
    pub const RESET_DATA_PTR: u16 = 76;

    pub const HALT: u16 = 99;
}

/// Roh-Argument einer Instruktion. Wird pro Opcode interpretiert.
#[derive(Clone)]
pub enum Arg {
    None,
    Int(i64),
    Str(Rc<str>),
    /// Geklammerte Tupel-Args (z.B. `(slot, name_idx, type_idx, default_idx)`).
    /// Elemente koennen Int/Str/None/verschachtelt sein.
    List(Vec<Arg>),
    /// Eingebetteter Laufzeitwert (z.B. DECLARE_LOCAL-Default).
    Val(Value),
}

impl Arg {
    pub fn as_i64(&self) -> i64 {
        match self {
            Arg::Int(i) => *i,
            _ => panic!("Arg: erwartet Int, erhalten anderes"),
        }
    }
    pub fn as_usize(&self) -> usize {
        self.as_i64() as usize
    }
    pub fn list(&self) -> &[Arg] {
        match self {
            Arg::List(v) => v,
            _ => panic!("Arg: erwartet List"),
        }
    }
    pub fn str(&self) -> &str {
        match self {
            Arg::Str(s) => s,
            _ => panic!("Arg: erwartet Str"),
        }
    }
}

pub struct Instr {
    pub op: u16,
    pub arg: Arg,
}

pub struct Func {
    pub name: String,
    pub n_params: usize,
    pub n_required: usize,
    pub is_variadic: bool,
    pub is_sub: bool,
    pub is_main: bool,
    /// Coroutine gdw. der Body ein YIELD enthaelt -- Aufruf liefert dann ein
    /// COROUTINE-Handle statt die Funktion auszufuehren.
    pub is_coroutine: bool,
    pub return_type: String,
    pub param_defaults: Vec<Value>,
    pub local_types: Vec<String>,
    pub local_defaults: Vec<Value>,
    pub constants: Vec<Value>,
    pub code: Vec<Instr>,
    /// Quell-Zeile pro Instruktion (parallel zu `code`). Leer wenn der
    /// Compiler keine Zeilen getrackt hat -- dann faellt die Fehlermeldung auf
    /// "Zeile unbekannt" zurueck.
    pub lines: Vec<u32>,
}

pub struct FieldDecl {
    pub name: String,
    pub type_name: String,
    pub array_dims: Vec<i64>,
}

pub struct ClassInfo {
    pub name: String,
    pub parent_name: String,
    pub is_struct: bool,
    pub fields: Vec<FieldDecl>,
    pub methods: std::collections::HashMap<String, Func>,
    pub properties: std::collections::HashSet<String>,
}

pub struct Program {
    pub n_globals: usize,
    pub main: Func,
    pub functions: std::collections::HashMap<String, Func>,
    pub classes: std::collections::HashMap<String, ClassInfo>,
    pub data: Vec<crate::value::Value>,
}

// ---------------------------------------------------------------------------
// Dekodierung
// ---------------------------------------------------------------------------

/// Dekodiert einen `.gbc`-Wert (Encoding aus `serialize.py::_enc`).
fn decode_value(j: &J) -> Value {
    match j {
        J::Null => Value::Nil,
        J::Bool(b) => Value::Bool(*b), // sollte nur in {"b":..} auftreten
        J::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().expect("Zahl"))
            }
        }
        J::String(s) => Value::Str(Rc::from(s.as_str())),
        J::Array(a) => Value::Tuple(Rc::new(a.iter().map(decode_value).collect())),
        J::Object(map) => {
            if let Some(v) = map.get("f") {
                Value::Float(v.as_f64().expect("f-Wert ist Zahl"))
            } else if let Some(v) = map.get("b") {
                Value::Bool(v.as_bool().expect("b-Wert ist Bool"))
            } else if map.contains_key("comp") {
                Value::CompMarker
            } else if let Some(v) = map.get("funcref") {
                Value::FuncRef(Rc::from(v.as_str().expect("funcref-Name")))
            } else if let Some(ns) = map.get("ns") {
                let obj = ns.as_object().expect("ns-Objekt");
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let mut members = std::collections::HashMap::new();
                if let Some(m) = obj.get("members").and_then(|v| v.as_object()) {
                    for (k, mv) in m {
                        members.insert(k.clone(), decode_value(mv));
                    }
                }
                Value::Namespace(Rc::new(crate::value::Namespace { name, members }))
            } else {
                panic!("decode_value: unbekanntes Objekt {:?}", map)
            }
        }
    }
}

/// Dekodiert ein Code-Argument. Plain-Zahl -> Int (Index/Slot), String -> Str,
/// null -> None, Array -> rekursive Liste. Getaggte Objekte ({"f"},{"b"},...)
/// werden als eingebetteter Wert behandelt.
fn decode_arg(j: &J) -> Arg {
    match j {
        J::Null => Arg::None,
        J::Number(n) => {
            if let Some(i) = n.as_i64() {
                Arg::Int(i)
            } else {
                Arg::Val(Value::Float(n.as_f64().unwrap()))
            }
        }
        J::String(s) => Arg::Str(Rc::from(s.as_str())),
        J::Array(a) => Arg::List(a.iter().map(decode_arg).collect()),
        J::Object(_) => Arg::Val(decode_value(j)),
        J::Bool(b) => Arg::Val(Value::Bool(*b)),
    }
}

fn decode_func(j: &J) -> Func {
    let obj = j.as_object().expect("func ist Objekt");
    let get = |k: &str| obj.get(k).unwrap_or(&J::Null);

    let constants = get("constants")
        .as_array()
        .map(|a| a.iter().map(decode_value).collect())
        .unwrap_or_default();
    let param_defaults = get("param_defaults")
        .as_array()
        .map(|a| a.iter().map(decode_value).collect())
        .unwrap_or_default();
    let local_defaults = get("local_defaults")
        .as_array()
        .map(|a| a.iter().map(decode_value).collect())
        .unwrap_or_default();
    let local_types = get("local_types")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("any").to_string()).collect())
        .unwrap_or_default();
    let code = get("code")
        .as_array()
        .map(|a| {
            a.iter()
                .map(|instr| {
                    let pair = instr.as_array().expect("instr ist [op, arg]");
                    Instr {
                        op: pair[0].as_u64().expect("op int") as u16,
                        arg: decode_arg(&pair[1]),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let lines = get("lines")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0) as u32).collect())
        .unwrap_or_default();

    Func {
        name: get("name").as_str().unwrap_or("").to_string(),
        n_params: get("n_params").as_u64().unwrap_or(0) as usize,
        n_required: get("n_required").as_u64().unwrap_or(0) as usize,
        is_variadic: get("is_variadic").as_bool().unwrap_or(false),
        is_sub: get("is_sub").as_bool().unwrap_or(true),
        is_main: get("is_main").as_bool().unwrap_or(false),
        is_coroutine: get("is_coroutine").as_bool().unwrap_or(false),
        return_type: get("return_type").as_str().unwrap_or("").to_string(),
        param_defaults,
        local_types,
        local_defaults,
        constants,
        code,
        lines,
    }
}

fn decode_class(j: &J) -> ClassInfo {
    let obj = j.as_object().expect("class ist Objekt");
    let get = |k: &str| obj.get(k).unwrap_or(&J::Null);
    let fields = get("fields")
        .as_array()
        .map(|a| {
            a.iter()
                .map(|f| {
                    let fo = f.as_object().unwrap();
                    FieldDecl {
                        name: fo.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                        type_name: fo.get("type_name").and_then(|x| x.as_str()).unwrap_or("any").to_string(),
                        array_dims: fo.get("array_dims")
                            .and_then(|x| x.as_array())
                            .map(|a| a.iter().filter_map(|d| d.as_i64()).collect())
                            .unwrap_or_default(),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let mut methods = std::collections::HashMap::new();
    if let Some(mo) = get("methods").as_object() {
        for (name, mj) in mo {
            methods.insert(name.clone(), decode_func(mj));
        }
    }
    let properties = get("properties")
        .as_array()
        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    ClassInfo {
        name: get("name").as_str().unwrap_or("").to_string(),
        parent_name: get("parent_name").as_str().unwrap_or("").to_string(),
        is_struct: get("is_struct").as_bool().unwrap_or(false),
        fields,
        methods,
        properties,
    }
}

pub fn load_program(j: &J) -> Result<Program, String> {
    let obj = j.as_object().ok_or("Top-Level ist kein Objekt")?;
    let fmt = obj.get("format").and_then(|v| v.as_str()).unwrap_or("");
    if fmt != "gbc" {
        return Err(format!("Unbekanntes Format: {:?}", fmt));
    }
    let n_globals = obj.get("n_globals").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let main = decode_func(obj.get("main").ok_or("kein main")?);
    let mut functions = std::collections::HashMap::new();
    if let Some(fobj) = obj.get("functions").and_then(|v| v.as_object()) {
        for (name, fj) in fobj {
            functions.insert(name.clone(), decode_func(fj));
        }
    }
    let mut classes = std::collections::HashMap::new();
    if let Some(cobj) = obj.get("classes").and_then(|v| v.as_object()) {
        for (name, cj) in cobj {
            classes.insert(name.clone(), decode_class(cj));
        }
    }
    let data = obj.get("data")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().map(decode_value).collect())
        .unwrap_or_default();
    Ok(Program {
        n_globals,
        main,
        functions,
        classes,
        data,
    })
}
