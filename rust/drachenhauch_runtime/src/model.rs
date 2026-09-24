//! Das geladene Programm der VM (`Program`, `Func`, `Instr`) -- aus einer
//! `.dhc`-Datei (`load_program`, JSON) oder direkt vom Compiler
//! (`compiler::compile_to_program`, ohne JSON-Umweg). Beide Wege gehen durch
//! `func_bauen` und `programm_bauen`.
//!
//! Der const-Pool und die Code-Instruktionen werden beim Laden EINMALIG in
//! native Rust-Typen dekodiert (kein serde_json zur Laufzeit der Dispatch-
//! Schleife). Strukturelle Argumente (Slots, Indizes, Namen) bleiben als
//! `Arg` erhalten und werden pro Opcode interpretiert.

use std::rc::Rc;

use serde_json::Value as J;

use crate::value::Value;

/// Opcode-Konstanten -- muessen exakt zu `compiler::oc` passen (dieselben Nummern
/// stehen in jeder .dhc-Datei).
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

    // (100-110 waren spezialisierte _NN-Opcodes des geloeschten
    // Python-Compilers -- der Rust-Compiler emittiert sie nie; entfernt.
    // Die Nummern bleiben reserviert, damit Slot-Opcodes 111+ stabil sind.)

    /// Fusioniertes FOR-Ende (Inkrement + Weiter-Test + Ruecksprung in
    /// EINEM Dispatch) -- emittiert fuer FOR mit konstantem STEP.
    pub const FOR_NEXT: u16 = 116;

    pub const LOAD_GLOBAL_SLOT: u16 = 111;
    pub const STORE_GLOBAL_SLOT: u16 = 112;
    pub const DECLARE_GLOBAL_SLOT: u16 = 113;
    pub const DECLARE_GLOBAL_CONST_SLOT: u16 = 114;

    /// Coroutine: pop yield-Wert -> suspendieren -> push send-Wert beim Resume.
    pub const YIELD_VALUE: u16 = 115;

    pub const PRINT: u16 = 70;
    pub const INPUT_NAME: u16 = 71;
    pub const INPUT_LOCAL: u16 = 72;

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
    pub const BUILD_ARRAY: u16 = 117;   // Array-Literal [a, b, c]
    pub const FIN_END: u16 = 118;       // FINALLY: gestapelten Fehler weiterwerfen
    pub const CALL_SUPER: u16 = 119;    // SUPER.Methode(): Suche bei der Elternklasse beginnen
    pub const ADD_STORE_LOCAL: u16 = 120;       // x = x + e: addieren und in den lokalen Platz schreiben
    pub const ADD_STORE_GLOBAL_SLOT: u16 = 121; // dasselbe fuer einen globalen Platz
    pub const BIND_GLOBAL_SLOT: u16 = 122;      // globalen Eintrag (Feld/Map/Struktur) in seinen Platz haengen
    pub const TYP_PRUEFEN: u16 = 123;           // nur DHRT_TYPEN_PRUEFEN: Typ des Stapelwerts gegen den Compiler pruefen

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
    /// Beim Laden gepackte All-Int-Liste (FOR_NEXT) -- Zugriff ohne
    /// per-Element-Enum-Match im heissesten Opcode.
    Ints(Box<[i64]>),
    /// Beim Laden zerlegtes Call-Argument: (name, argc, fn_idx).
    /// fn_idx = vorab aufgeloester Funktions-Index (CALL_USER) oder -1.
    Call(Rc<str>, u32, i32),
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
    pub fn ints(&self) -> &[i64] {
        match self {
            Arg::Ints(v) => v,
            _ => panic!("Arg: erwartet Ints"),
        }
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
    /// Nur CALL_BUILTIN: welche Befehlsfamilie beim ersten Aufruf geantwortet
    /// hat (1-basiert, 0 = noch unbekannt). Die VM fragt sie beim naechsten
    /// Mal zuerst, statt den Namen durch alle Familien zu reichen
    /// (`Vm::builtin_rufen`).
    pub familie: std::cell::Cell<u8>,
    /// Nur CALL_METHOD: (Klasse, Methode) des letzten Aufrufs. Hat das naechste
    /// Objekt dieselbe Klasse, ist die Methode ohne Suche da (`op::CALL_METHOD`).
    /// Beide Zeiger gelten, solange das Programm lebt -- es wird nach dem
    /// Laden nicht mehr veraendert (wie `CoroState::fn_ptr`).
    pub methode: std::cell::Cell<(*const ClassInfo, *const Func)>,
    /// Superinstruktion (M2): beginnt hier eine Folge, die `dispatch` in einem
    /// Schritt ausfuehren kann? 0 = nein, sonst die Art (`VS_*`). Die Folge
    /// selbst bleibt unveraendert stehen -- klappt der schnelle Weg nicht,
    /// laeuft sie Befehl fuer Befehl wie immer. Gesetzt von `verschmelzen`.
    pub schnell: u8,
}

/// Arten der Superinstruktionen (M2). Alle beginnen mit zwei Operanden
/// (LOAD_LOCAL / LOAD_CONST / LOAD_GLOBAL_SLOT) und einem Rechen- oder
/// Vergleichsbefehl; danach:
pub const VS_WERT: u8 = 1;          // ... nichts: das Ergebnis liegt auf dem Stapel
pub const VS_LOKAL: u8 = 2;         // ... STORE_LOCAL
pub const VS_GLOBAL: u8 = 3;        // ... STORE_GLOBAL_SLOT
pub const VS_SPRUNG_FALSCH: u8 = 4; // ... JUMP_IF_FALSE (nur Vergleiche)
pub const VS_SPRUNG_WAHR: u8 = 5;   // ... JUMP_IF_TRUE (nur Vergleiche)

/// Markiert Folgen `a b OP [ZIEL]` fuer `dispatch` (siehe `Instr::schnell`).
///
/// Nur wo kein Sprung IN die Folge fuehrt -- sonst liefe an dieser Stelle
/// ein halber Rest. Sprungziele haben JUMP/JUMP_IF_*, FOR_NEXT (Rumpf) und
/// TRY_BEGIN (CATCH); ein YIELD kommt in keiner Folge vor.
fn verschmelzen(code: &mut [Instr]) {
    let mut ziel = vec![false; code.len() + 1];
    for ins in code.iter() {
        let t = match ins.op {
            op::JUMP | op::JUMP_IF_FALSE | op::JUMP_IF_TRUE | op::TRY_BEGIN => ins.arg.as_i64(),
            // Beim Laden steht das Argument noch als Liste da; gepackt
            // (`Arg::Ints`) wird es erst spaeter.
            op::FOR_NEXT => match &ins.arg {
                Arg::Ints(v) => v.get(6).copied().unwrap_or(-1),
                Arg::List(l) => l.get(6).map(|a| a.as_i64()).unwrap_or(-1),
                _ => -1,
            },
            _ => continue,
        };
        if t >= 0 && (t as usize) < ziel.len() { ziel[t as usize] = true; }
    }
    let operand = |o: u16| matches!(o, op::LOAD_LOCAL | op::LOAD_CONST | op::LOAD_GLOBAL_SLOT);
    let vergleich = |o: u16| matches!(o, op::LT | op::GT | op::LEQ | op::GEQ | op::EQ | op::NEQ);
    let rechnen = |o: u16| matches!(o, op::ADD | op::SUB | op::MUL | op::DIV | op::MOD);
    for p in 0..code.len().saturating_sub(2) {
        let (a, b, o) = (code[p].op, code[p + 1].op, code[p + 2].op);
        if !operand(a) || !operand(b) || !(rechnen(o) || vergleich(o)) { continue; }
        if ziel[p + 1] || ziel[p + 2] { continue; }
        let danach = if ziel.get(p + 3).copied().unwrap_or(true) { None } else { code.get(p + 3).map(|i| i.op) };
        code[p].schnell = match danach {
            Some(op::STORE_LOCAL) => VS_LOKAL,
            Some(op::STORE_GLOBAL_SLOT) => VS_GLOBAL,
            Some(op::JUMP_IF_FALSE) if vergleich(o) => VS_SPRUNG_FALSCH,
            Some(op::JUMP_IF_TRUE) if vergleich(o) => VS_SPRUNG_WAHR,
            _ => VS_WERT,
        };
    }
}

pub struct Func {
    pub name: String,
    pub n_params: usize,
    pub n_required: usize,
    pub is_variadic: bool,
    pub is_sub: bool,
    /// Coroutine gdw. der Body ein YIELD enthaelt -- Aufruf liefert dann ein
    /// COROUTINE-Handle statt die Funktion auszufuehren.
    pub is_coroutine: bool,
    pub return_type: String,
    pub param_defaults: Vec<Value>,
    /// Pro Parameter: ob er BYREF ist (Copy-In/Copy-Out). Leer = keine BYREF-
    /// Parameter (alte .dhc / Funktionen ohne BYREF). Nur die direkten
    /// CALL_USER-Aufrufe (freie Funktionen) werten das aus -- der Compiler kennt
    /// dort die Signatur statisch und emittiert das Write-Back.
    pub param_byref: Vec<bool>,
    /// Pro Parameter: ob der Default ein Nicht-Literal-Ausdruck ist (im Callee-
    /// Prolog zur Laufzeit ausgewertet). bind_params laesst solche Slots beim
    /// Nil-Sentinel statt zu coercen; der Prolog berechnet sie. Leer = keine.
    pub param_default_is_expr: Vec<bool>,
    pub local_types: Vec<String>,
    pub local_defaults: Vec<Value>,
    pub constants: Vec<Value>,
    pub code: Vec<Instr>,
    /// Quell-Zeile pro Instruktion (parallel zu `code`). Leer wenn der
    /// Compiler keine Zeilen getrackt hat -- dann faellt die Fehlermeldung auf
    /// "Zeile unbekannt" zurueck.
    pub lines: Vec<u32>,
    /// Debug-Namen pro Local-Slot (parallel zu `local_types`); leer fuer
    /// Compiler-Zwischenwerte. Nur fuer `dhrt debug` (Variablen-Inspektion);
    /// leer wenn der Compiler keine Namen getrackt hat (alte .dhc).
    pub local_names: Vec<String>,
}

pub struct FieldDecl {
    pub name: String,
    pub type_name: String,
    pub array_dims: Vec<i64>,
}

pub struct ClassInfo {
    /// Der eigene Name (wie der Schluessel in `Program::classes`) -- fuer den
    /// Merkplatz von CALL_METHOD (`Instr::methode`).
    pub name: String,
    pub parent_name: String,
    pub is_struct: bool,
    pub fields: Vec<FieldDecl>,
    pub methods: rustc_hash::FxHashMap<String, Func>,
    pub properties: std::collections::HashSet<String>,
    /// Hat diese Klasse ODER eine Vorfahrin eine PROPERTY? Einmal beim Laden
    /// gerechnet (`load_program`). Jeder `obj.x` fragt sonst die ganze Kette nach
    /// einer PROPERTY ab, und die meisten Klassen haben keine.
    pub props_kette: bool,
    /// Je PROPERTY-Name (klein, samt Vorfahren) die Klasse und der Methodenname
    /// von Getter bzw. Setter -- aufgeloest wie `resolve_method` (die Klasse
    /// selbst zuerst). Beim Laden gerechnet; vorher baute jeder Zugriff
    /// `format!("__get_{}")` und suchte die Kette ab.
    pub prop_get: rustc_hash::FxHashMap<String, (String, String)>,
    pub prop_set: rustc_hash::FxHashMap<String, (String, String)>,
    /// Alle PROPERTY-Namen der Kette, auch ohne Getter oder Setter.
    pub props_alle: rustc_hash::FxHashSet<String>,
}

pub struct Program {
    pub n_globals: usize,
    /// Name je globalem Platz (fuer Meldungen); leer bei alten .dhc-Dateien.
    pub global_names: Vec<String>,
    pub main: Func,
    /// Alle freien User-Funktionen; der Index ist die vorab aufgeloeste
    /// CALL_USER-Referenz (siehe `resolve_calls` -- kein Hash-Lookup pro
    /// Aufruf mehr). Nach dem Laden immutabel (Coroutine-fn_ptr zeigt rein).
    pub functions: Vec<Func>,
    pub fn_index: rustc_hash::FxHashMap<String, usize>,
    pub classes: rustc_hash::FxHashMap<String, ClassInfo>,
    pub data: Vec<crate::value::Value>,
}

impl Program {
    /// Funktion per Name (LOAD_FUNCREF/CALL_VALUE/Callbacks -- kalte Pfade).
    ///
    /// **Schreibungs-unabhaengig**, und das ist kein Komfort, sondern noetig:
    /// der Compiler legt jeden Funktionsnamen klein ab, DH-Bezeichner sind
    /// ueberall sonst schreibungs-unabhaengig -- aber Namen, die NICHT aus
    /// dem Compiler kommen, tragen ihre Original-Schreibweise. Das sind die
    /// GUI-Callbacks aus einer `.dhform`-Datei (`GUI_LOAD`/`GUI_FROM_JSON`):
    /// der Form-Designer schreibt dort `dd1Changed`, und damit fand die
    /// Laufzeit den erzeugten `SUB dd1Changed()` beim Ausloesen NIE.
    ///
    /// Der Zweitversuch kostet nur im Fehlerfall etwas (dann steht ohnehin
    /// eine Fehlermeldung an) -- der Treffer laeuft unveraendert ueber den
    /// direkten Lookup.
    pub fn func(&self, name: &str) -> Option<&Func> {
        if let Some(&i) = self.fn_index.get(name) {
            return Some(&self.functions[i]);
        }
        let klein = name.to_lowercase();
        if klein == name {
            return None;                 // war schon klein -> gibt es wirklich nicht
        }
        self.fn_index.get(&klein).map(|&i| &self.functions[i])
    }
}

/// Argumente der heissen Opcodes beim Laden vor-zerlegen:
/// - CALL_*: `[name, argc]` -> `Arg::Call(name, argc, idx)`; idx = vorab
///   aufgeloester Funktions-Index (nur CALL_USER, sonst/-unbekannt -1 --
///   die VM faellt dann auf den Namens-Lookup samt gewohnter Meldung zurueck).
/// - FOR_NEXT: All-Int-Liste -> `Arg::Ints` (kein per-Element-Match).
fn specialize_args(code: &mut [Instr], fn_index: &rustc_hash::FxHashMap<String, usize>) {
    for ins in code.iter_mut() {
        match ins.op {
            op::FOR_NEXT => {
                if let Arg::List(l) = &ins.arg {
                    if !l.is_empty() && l.iter().all(|a| matches!(a, Arg::Int(_))) {
                        ins.arg = Arg::Ints(l.iter().map(|a| a.as_i64()).collect());
                    }
                }
            }
            op::CALL_USER | op::CALL_BUILTIN | op::CALL_METHOD | op::CALL_VALUE => {
                if let Arg::List(l) = &ins.arg {
                    if l.len() == 2 {
                        if let (Arg::Str(name), Arg::Int(argc)) = (&l[0], &l[1]) {
                            let idx = if ins.op == op::CALL_USER {
                                fn_index.get(name.as_ref()).map(|&i| i as i32).unwrap_or(-1)
                            } else { -1 };
                            ins.arg = Arg::Call(name.clone(), *argc as u32, idx);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Dekodierung
// ---------------------------------------------------------------------------

/// Dekodiert einen `.dhc`-Wert (Encoding aus `serialize.py::_enc`).
/// Neutrales Element der Mathe-Typen (MAT4 = Einheitsmatrix, VEC* = Null,
/// QUAT = Einheits-Quaternion). Alles andere: None -- bleibt NIL.
pub(crate) fn neutrales_element(t: &str) -> Option<Value> {
    match t {
        "mat4" => Some(Value::Mat4(std::rc::Rc::new(crate::builtins::mat4_identity()))),
        "quat" => Some(Value::Quat(0.0, 0.0, 0.0, 1.0)),
        "vec2" => Some(Value::Vec2(0.0, 0.0)),
        "geld" => Some(Value::Geld(0)),
        "vec3" => Some(Value::Vec3(0.0, 0.0, 0.0)),
        "vec4" => Some(Value::Vec4(0.0, 0.0, 0.0, 0.0)),
        _ => None,
    }
}

pub(crate) fn decode_value(j: &J) -> Value {
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
        J::String(s) => Value::Str(Rc::new(s.clone())),
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
                let mut members = rustc_hash::FxHashMap::default();
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
pub(crate) fn decode_arg(j: &J) -> Arg {
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
    let param_byref = get("param_byref")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_bool().unwrap_or(false)).collect())
        .unwrap_or_default();
    let param_default_is_expr = get("param_default_is_expr")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_bool().unwrap_or(false)).collect())
        .unwrap_or_default();
    let local_defaults: Vec<Value> = get("local_defaults")
        .as_array()
        .map(|a| a.iter().map(decode_value).collect())
        .unwrap_or_default();
    let local_types: Vec<String> = get("local_types")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("any").to_string()).collect())
        .unwrap_or_default();
    let code = get("code")
        .as_array()
        .map(|a| {
            a.iter()
                .map(|instr| {
                    let pair = instr.as_array().expect("instr ist [op, arg]");
                    (pair[0].as_u64().expect("op int") as u16, decode_arg(&pair[1]))
                })
                .collect()
        })
        .unwrap_or_default();
    let lines = get("lines")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0) as u32).collect())
        .unwrap_or_default();
    let local_names = get("local_names")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
        .unwrap_or_default();

    func_bauen(FuncRoh {
        name: get("name").as_str().unwrap_or("").to_string(),
        n_params: get("n_params").as_u64().unwrap_or(0) as usize,
        n_required: get("n_required").as_u64().unwrap_or(0) as usize,
        is_variadic: get("is_variadic").as_bool().unwrap_or(false),
        is_sub: get("is_sub").as_bool().unwrap_or(true),
        is_coroutine: get("is_coroutine").as_bool().unwrap_or(false),
        return_type: get("return_type").as_str().unwrap_or("").to_string(),
        param_defaults,
        param_byref,
        param_default_is_expr,
        local_types,
        local_defaults,
        constants,
        code,
        lines,
        local_names,
    })
}

/// Die Teile einer Funktion, schon als Laufzeit-Werte -- aus einer
/// .dhc-Datei (`decode_func`) ODER direkt vom Compiler
/// (`compiler::compile_to_program`, ohne JSON-Umweg). Beide Wege gehen durch
/// `func_bauen`, damit sie nicht auseinanderlaufen.
pub(crate) struct FuncRoh {
    pub name: String,
    pub n_params: usize,
    pub n_required: usize,
    pub is_variadic: bool,
    pub is_sub: bool,
    pub is_coroutine: bool,
    pub return_type: String,
    pub param_defaults: Vec<Value>,
    pub param_byref: Vec<bool>,
    pub param_default_is_expr: Vec<bool>,
    pub local_types: Vec<String>,
    pub local_defaults: Vec<Value>,
    pub constants: Vec<Value>,
    pub code: Vec<(u16, Arg)>,
    pub lines: Vec<u32>,
    pub local_names: Vec<String>,
}

pub(crate) fn func_bauen(r: FuncRoh) -> Func {
    let mut local_defaults = r.local_defaults;
    // Mathe-Typen starten mit ihrem neutralen Element statt mit NIL. Der
    // Compiler kann das nicht ablegen (seine Konstanten kennen kein MAT4),
    // also wird es hier beim Laden nachgetragen -- genauso wie fuer globale
    // Variablen in vm.rs. Alle anderen Typen bleiben, wie sie sind.
    for (i, t) in r.local_types.iter().enumerate() {
        if i < local_defaults.len() && matches!(local_defaults[i], Value::Nil) {
            if let Some(v) = neutrales_element(t) {
                local_defaults[i] = v;
            }
        }
    }
    let mut code: Vec<Instr> = r.code.into_iter().map(|(op, arg)| Instr {
        op,
        arg,
        familie: std::cell::Cell::new(0),
        methode: std::cell::Cell::new((std::ptr::null(), std::ptr::null())),
        schnell: 0,
    }).collect();
    if std::env::var_os("DHRT_OHNE_VERSCHMELZEN").is_none() {
        verschmelzen(&mut code);
    }
    Func {
        name: r.name,
        n_params: r.n_params,
        n_required: r.n_required,
        is_variadic: r.is_variadic,
        is_sub: r.is_sub,
        is_coroutine: r.is_coroutine,
        return_type: r.return_type,
        param_defaults: r.param_defaults,
        param_byref: r.param_byref,
        param_default_is_expr: r.param_default_is_expr,
        local_types: r.local_types,
        local_defaults,
        constants: r.constants,
        code,
        lines: r.lines,
        local_names: r.local_names,
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
    let mut methods = rustc_hash::FxHashMap::default();
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
        props_kette: false,
        prop_get: Default::default(),
        prop_set: Default::default(),
        props_alle: Default::default(),
    }
}

pub fn load_program(j: &J) -> Result<Program, String> {
    let obj = j.as_object().ok_or("Top-Level ist kein Objekt")?;
    let fmt = obj.get("format").and_then(|v| v.as_str()).unwrap_or("");
    // "gbc" ist der alte Name aus der GameBasic-Zeit. Er wird weiter gelesen,
    // weil das eine Zeile kostet und eine herumliegende Datei sonst mit
    // "Unbekanntes Format" abgewiesen wuerde -- geschrieben wird nur "dhc".
    if fmt != "dhc" && fmt != "gbc" {
        return Err(format!("Unbekanntes Format: {:?}", fmt));
    }
    let n_globals = obj.get("n_globals").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let global_names: Vec<String> = obj.get("global_names").and_then(|v| v.as_array())
        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
        .unwrap_or_default();
    let main = decode_func(obj.get("main").ok_or("kein main")?);
    let mut functions: Vec<(String, Func)> = Vec::new();
    if let Some(fobj) = obj.get("functions").and_then(|v| v.as_object()) {
        for (name, fj) in fobj {
            functions.push((name.clone(), decode_func(fj)));
        }
    }
    let mut classes = rustc_hash::FxHashMap::default();
    if let Some(cobj) = obj.get("classes").and_then(|v| v.as_object()) {
        for (name, cj) in cobj {
            classes.insert(name.clone(), decode_class(cj));
        }
    }
    let data = obj.get("data")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().map(decode_value).collect())
        .unwrap_or_default();
    Ok(programm_bauen(n_globals, global_names, main, functions, classes, data))
}

/// Ein geladenes Programm fertig machen -- fuer `load_program` (.dhc) UND fuer
/// den Compiler (`compiler::compile_to_program`, ohne JSON-Umweg): Aufruf-
/// Indizes, vor-zerlegte Argumente, PROPERTY-Tabellen.
pub(crate) fn programm_bauen(n_globals: usize, global_names: Vec<String>, main: Func,
                             benannte: Vec<(String, Func)>,
                             mut classes: rustc_hash::FxHashMap<String, ClassInfo>,
                             data: Vec<Value>) -> Program {
    let mut functions: Vec<Func> = Vec::with_capacity(benannte.len());
    let mut fn_index = rustc_hash::FxHashMap::default();
    for (name, f) in benannte {
        fn_index.insert(name, functions.len());
        functions.push(f);
    }
    // Heisse Opcode-Args vor-zerlegen (main + Funktionen + Methoden).
    let mut main = main;
    specialize_args(&mut main.code, &fn_index);
    for f in functions.iter_mut() { specialize_args(&mut f.code, &fn_index); }
    for c in classes.values_mut() {
        for m in c.methods.values_mut() { specialize_args(&mut m.code, &fn_index); }
    }
    // PROPERTY irgendwo in der Kette? (props_kette)
    let mit_props: Vec<String> = classes.keys().filter(|k| {
        let mut cur = classes.get(k.as_str());
        let mut tiefe = 0;
        while let Some(ci) = cur {
            if !ci.properties.is_empty() { return true; }
            tiefe += 1;
            if ci.parent_name.is_empty() || tiefe > 64 { break; }
            cur = classes.get(ci.parent_name.as_str());
        }
        false
    }).cloned().collect();
    for k in mit_props { if let Some(ci) = classes.get_mut(&k) { ci.props_kette = true; } }
    // PROPERTY-Tabellen je Klasse: alle Namen der Kette, Getter/Setter so
    // aufgeloest wie resolve_method (erst die Klasse, dann aufwaerts).
    let namen: Vec<String> = classes.keys().cloned().collect();
    for k in namen {
        let mut kette: Vec<&ClassInfo> = Vec::new();
        let mut cur = classes.get(k.as_str());
        let mut kette_namen: Vec<String> = Vec::new();
        while let Some(ci) = cur {
            kette.push(ci);
            kette_namen.push(if kette_namen.is_empty() { k.clone() } else { kette[kette.len() - 2].parent_name.clone() });
            if ci.parent_name.is_empty() || kette.len() > 64 { break; }
            cur = classes.get(ci.parent_name.as_str());
        }
        let mut alle = rustc_hash::FxHashSet::default();
        for ci in &kette { for p in &ci.properties { alle.insert(p.to_lowercase()); } }
        let mut get = rustc_hash::FxHashMap::default();
        let mut set = rustc_hash::FxHashMap::default();
        for p in &alle {
            for (art, ziel) in [("__get_", &mut get), ("__set_", &mut set)] {
                let key = format!("{}{}", art, p);
                if let Some(i) = kette.iter().position(|ci| ci.methods.contains_key(&key)) {
                    ziel.insert(p.clone(), (kette_namen[i].clone(), key));
                }
            }
        }
        if let Some(ci) = classes.get_mut(&k) { ci.props_alle = alle; ci.prop_get = get; ci.prop_set = set; }
    }
    Program {
        n_globals,
        global_names,
        main,
        functions,
        fn_index,
        classes,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leere_func(name: &str) -> Func {
        Func {
            name: name.to_string(), n_params: 0, n_required: 0, is_variadic: false,
            is_sub: true, is_coroutine: false, return_type: String::new(),
            param_defaults: Vec::new(), param_byref: Vec::new(),
            param_default_is_expr: Vec::new(), local_types: Vec::new(),
            local_defaults: Vec::new(), constants: Vec::new(), code: Vec::new(),
            lines: Vec::new(), local_names: Vec::new(),
        }
    }

    /// Der Compiler legt Namen KLEIN ab; GUI-Callbacks aus einer `.dhform`
    /// tragen dagegen die Schreibweise des Form-Designers (`dd1Changed`).
    /// Ohne den schreibungs-unabhaengigen Zweitversuch meldete die Laufzeit
    /// beim Ausloesen "Funktion 'dd1Changed' existiert nicht" -- also bei
    /// JEDEM automatisch erzeugten Handler.
    #[test]
    fn func_findet_auch_bei_anderer_schreibweise() {
        let mut fn_index = rustc_hash::FxHashMap::default();
        fn_index.insert("dd1changed".to_string(), 0usize);
        let p = Program {
            n_globals: 0,
            global_names: Vec::new(),
            main: leere_func("__main__"),
            functions: vec![leere_func("dd1changed")],
            fn_index,
            classes: rustc_hash::FxHashMap::default(),
            data: Vec::new(),
        };
        assert!(p.func("dd1changed").is_some());     // wie der Compiler ablegt
        assert!(p.func("dd1Changed").is_some());     // wie der Designer schreibt
        assert!(p.func("DD1CHANGED").is_some());
        assert!(p.func("gibtesnicht").is_none());
        assert!(p.func("GibtEsNicht").is_none());    // Zweitversuch erfindet nichts
    }
}
