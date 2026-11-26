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
    pub const TYP_PRUEFEN: u16 = 123;
    pub const CALL_METHOD_BYREF: u16 = 124;     // Methode mit BYREF-Parametern: [name, argc, [byref-Stellen]]           // nur DHRT_TYPEN_PRUEFEN: Typ des Stapelwerts gegen den Compiler pruefen
    pub const ADD_STORE_FIELD: u16 = 125;       // text = text + e in einer Methode (Feld von Self)
    pub const ADD_STORE_MEMBER: u16 = 126;      // obj.x = obj.x + e: [obj, a, b]
    pub const ADD_STORE_INDEX: u16 = 127;       // c[i] = c[i] + e: [c, i.., a, b] (Feld oder MAP)
    pub const LEIHE_LOCAL: u16 = 128;           // BYREF-Argument: Wert aus dem lokalen Platz nehmen statt kopieren
    pub const LEIHE_GLOBAL_SLOT: u16 = 129;     // dasselbe fuer einen globalen Platz

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
    /// Merkplatz fuer Objekte: (Lage der Klasse, was an ihr gefunden wurde).
    /// CALL_METHOD merkt sich die Methode (`*const Func` als Zahl), die
    /// Feldbefehle (LOAD/STORE_MEMBER, LOAD/STORE_FIELD) den Platz des
    /// Feldes. Hat das naechste Objekt dieselbe Lage (dieselbe Klasse -- jede
    /// hat ihr eigenes `Rc<Layout>`), ist es ohne Suche da. Ein Platz fuer
    /// beide: kein Befehl braucht beides, und ein groesseres `Instr` kostete
    /// gemessen 7 % bei reinen Zahlenschleifen. Die Zeiger gelten, solange das
    /// Programm lebt -- es wird nach dem Laden nicht mehr veraendert.
    pub merk: std::cell::Cell<(*const crate::value::Layout, usize)>,
    /// Superinstruktion (M2): beginnt hier eine Folge, die `dispatch` in einem
    /// Schritt ausfuehren kann? 0 = nein, sonst die Art (`VS_*`). Die Folge
    /// selbst bleibt unveraendert stehen -- klappt der schnelle Weg nicht,
    /// laeuft sie Befehl fuer Befehl wie immer. Gesetzt von `verschmelzen`.
    pub schnell: u8,
    /// Nur Ruecksprung (FOR_NEXT, JUMP nach hinten), nur mit Maschinencode:
    /// 0 = noch nicht versucht, 1 = die Schleife bleibt in der VM, n >= 2 =
    /// uebersetzt als Schleife n - 2 (`jit::Jit::schleife`).
    pub schleife: std::cell::Cell<u32>,
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
    /// Wie ein Befehl im QUELLTEXT hiess, wo der Compiler ihn auf einen
    /// anderen Namen abgebildet hat (`FILE_EXISTS` -> `fileexists`, namen.rs;
    /// `j_parse` -> `json_parse`). Ein CALL_BUILTIN zeigt mit seinem dritten
    /// Wert in `Arg::Call` hierher (sonst -1). Gebraucht nur im Fehlerfall:
    /// die Meldung nennt den geschriebenen Namen (`Vm::fehler_benennen`).
    pub anzeigen: Vec<Rc<str>>,
}

pub struct FieldDecl {
    pub name: String,
    pub type_name: String,
    pub array_dims: Vec<i64>,
    /// Untergrenze je Dimension, leer = alle 0 (fehlt in alten .dhc).
    pub array_unten: Vec<i64>,
    /// Ein STRUCT ... LAYOUT C als Feld: so viele Bytes Puffer je Objekt.
    pub struct_bytes: usize,
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
    /// Feste Lage der Felder samt Vorfahren (`programm_bauen`).
    pub layout: std::rc::Rc<crate::value::Layout>,
    /// Was NEW fuer ein Objekt dieser Klasse braucht, beim ersten NEW
    /// gerechnet (`Vm::vorlage`). Vorher suchte jedes NEW die Vorfahren und
    /// je Feld Typ und Platz per Namen.
    pub vorlage: std::cell::OnceCell<crate::vm::ObjektVorlage>,
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
/// - CALL_BUILTIN `[name, argc, geschrieben]`: der dritte Wert ist der Name
///   im Quelltext; er kommt in die zurueckgegebene Liste (`Func::anzeigen`),
///   und `idx` zeigt darauf.
/// - FOR_NEXT: All-Int-Liste -> `Arg::Ints` (kein per-Element-Match).
fn specialize_args(code: &mut [Instr], fn_index: &rustc_hash::FxHashMap<String, usize>) -> Vec<Rc<str>> {
    let mut anzeigen: Vec<Rc<str>> = Vec::new();
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
                    } else if l.len() == 3 && ins.op == op::CALL_BUILTIN {
                        if let (Arg::Str(name), Arg::Int(argc), Arg::Str(geschrieben)) = (&l[0], &l[1], &l[2]) {
                            anzeigen.push(geschrieben.clone());
                            ins.arg = Arg::Call(name.clone(), *argc as u32, (anzeigen.len() - 1) as i32);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    anzeigen
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

/// Ein Wert aus dem Bytecode. Aus einer .dhc kann alles kommen: eine
/// beschaedigte Datei wird ein Lade-Fehler (bis 2026.53 eine Panik mit
/// Exit 101, `{"format":"dhc","main":"kaputt"}`).
pub(crate) fn wert_lesen(j: &J) -> Result<Value, String> {
    Ok(match j {
        J::Null => Value::Nil,
        J::Bool(b) => Value::Bool(*b), // sollte nur in {"b":..} auftreten
        J::Number(n) => match n.as_i64() {
            Some(i) => Value::Int(i),
            None => Value::Float(n.as_f64().ok_or_else(|| format!("Zahl {} passt nicht", n))?),
        },
        J::String(s) => Value::Str(Rc::new(s.clone())),
        J::Array(a) => {
            let teile: Result<Vec<Value>, String> = a.iter().map(wert_lesen).collect();
            Value::Tuple(Rc::new(teile?))
        }
        J::Object(map) => {
            if let Some(v) = map.get("f") {
                Value::Float(v.as_f64().ok_or("ein f-Wert ist keine Zahl")?)
            } else if let Some(v) = map.get("b") {
                Value::Bool(v.as_bool().ok_or("ein b-Wert ist kein Wahrheitswert")?)
            } else if map.contains_key("comp") {
                Value::CompMarker
            } else if let Some(v) = map.get("funcref") {
                Value::FuncRef(Rc::from(v.as_str().ok_or("ein funcref ist kein Name")?))
            } else if let Some(ns) = map.get("ns") {
                let obj = ns.as_object().ok_or("ein ns ist kein Objekt")?;
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let mut members = rustc_hash::FxHashMap::default();
                if let Some(m) = obj.get("members").and_then(|v| v.as_object()) {
                    for (k, mv) in m {
                        members.insert(k.clone(), wert_lesen(mv)?);
                    }
                }
                Value::Namespace(Rc::new(crate::value::Namespace { name, members }))
            } else {
                return Err(format!("unbekannter Wert {}", kurz(j)));
            }
        }
    })
}

/// Fuer Werte vom Compiler, die immer gueltig sind.
pub(crate) fn decode_value(j: &J) -> Value {
    wert_lesen(j).expect("Wert vom Compiler")
}

/// Ein Stueck JSON fuer eine Meldung, gekuerzt.
fn kurz(j: &J) -> String {
    let t = j.to_string();
    if t.chars().count() > 60 { format!("{}...", t.chars().take(60).collect::<String>()) } else { t }
}

/// Dekodiert ein Code-Argument. Plain-Zahl -> Int (Index/Slot), String -> Str,
/// null -> None, Array -> rekursive Liste. Getaggte Objekte ({"f"},{"b"},...)
/// werden als eingebetteter Wert behandelt.
pub(crate) fn arg_lesen(j: &J) -> Result<Arg, String> {
    Ok(match j {
        J::Null => Arg::None,
        J::Number(n) => match n.as_i64() {
            Some(i) => Arg::Int(i),
            None => Arg::Val(Value::Float(n.as_f64().ok_or_else(|| format!("Zahl {} passt nicht", n))?)),
        },
        J::String(s) => Arg::Str(Rc::from(s.as_str())),
        J::Array(a) => {
            let teile: Result<Vec<Arg>, String> = a.iter().map(arg_lesen).collect();
            Arg::List(teile?.into())
        }
        J::Object(_) => Arg::Val(wert_lesen(j)?),
        J::Bool(b) => Arg::Val(Value::Bool(*b)),
    })
}

/// Fuer Argumente vom Compiler, die immer gueltig sind.
pub(crate) fn decode_arg(j: &J) -> Arg {
    arg_lesen(j).expect("Argument vom Compiler")
}

/// Eine Liste von Werten (`constants`, `param_defaults` ...); fehlt sie, ist
/// sie leer.
fn werte_lesen(j: &J) -> Result<Vec<Value>, String> {
    match j.as_array() {
        Some(a) => a.iter().map(wert_lesen).collect(),
        None => Ok(Vec::new()),
    }
}

fn decode_func(j: &J) -> Result<Func, String> {
    let obj = j.as_object().ok_or_else(|| format!("eine Funktion ist kein Objekt: {}", kurz(j)))?;
    let get = |k: &str| obj.get(k).unwrap_or(&J::Null);
    let fname = get("name").as_str().unwrap_or("?");
    let im = |e: String| format!("in {}: {}", if fname.is_empty() { "?" } else { fname }, e);

    let constants = werte_lesen(get("constants")).map_err(im)?;
    let param_defaults = werte_lesen(get("param_defaults")).map_err(im)?;
    let param_byref = get("param_byref")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_bool().unwrap_or(false)).collect())
        .unwrap_or_default();
    let param_default_is_expr = get("param_default_is_expr")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_bool().unwrap_or(false)).collect())
        .unwrap_or_default();
    let local_defaults: Vec<Value> = werte_lesen(get("local_defaults")).map_err(im)?;
    let local_types: Vec<String> = get("local_types")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("any").to_string()).collect())
        .unwrap_or_default();
    let mut code: Vec<(u16, Arg)> = Vec::new();
    for (k, instr) in get("code").as_array().map(|a| a.as_slice()).unwrap_or(&[]).iter().enumerate() {
        let schlecht = || im(format!("Befehl {} ist kein [op, arg]: {}", k, kurz(instr)));
        let pair = instr.as_array().filter(|p| p.len() == 2).ok_or_else(schlecht)?;
        let op = pair[0].as_u64().filter(|&o| o <= u16::MAX as u64).ok_or_else(schlecht)? as u16;
        let arg = arg_lesen(&pair[1]).map_err(im)?;
        // Sprungziele liest schon das Laden (`verschmelzen`), FOR_NEXT dazu
        // als sieben Zahlen (`Arg::ints`) -- beides sonst eine Panik.
        let gut = match op {
            op::JUMP | op::JUMP_IF_FALSE | op::JUMP_IF_TRUE | op::TRY_BEGIN => matches!(arg, Arg::Int(_)),
            op::FOR_NEXT => matches!(&arg, Arg::List(l) if l.len() == 7 && l.iter().all(|a| matches!(a, Arg::Int(_)))),
            _ => true,
        };
        if !gut { return Err(schlecht()); }
        code.push((op, arg));
    }
    let lines = get("lines")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0) as u32).collect())
        .unwrap_or_default();
    let local_names = get("local_names")
        .as_array()
        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
        .unwrap_or_default();

    Ok(func_bauen(FuncRoh {
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
    }))
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
        merk: std::cell::Cell::new((std::ptr::null(), 0)),
        schnell: 0,
        schleife: std::cell::Cell::new(0),
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
        anzeigen: Vec::new(),
    }
}

fn decode_class(j: &J) -> Result<ClassInfo, String> {
    let obj = j.as_object().ok_or_else(|| format!("eine Klasse ist kein Objekt: {}", kurz(j)))?;
    let get = |k: &str| obj.get(k).unwrap_or(&J::Null);
    let fields = get("fields")
        .as_array()
        .map(|a| {
            a.iter()
                .map(|f| {
                    let fo = f.as_object().ok_or_else(|| format!("ein Feld ist kein Objekt: {}", kurz(f)))?;
                    Ok(FieldDecl {
                        name: fo.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                        type_name: fo.get("type_name").and_then(|x| x.as_str()).unwrap_or("any").to_string(),
                        array_dims: fo.get("array_dims")
                            .and_then(|x| x.as_array())
                            .map(|a| a.iter().filter_map(|d| d.as_i64()).collect())
                            .unwrap_or_default(),
                        array_unten: fo.get("array_unten")
                            .and_then(|x| x.as_array())
                            .map(|a| a.iter().filter_map(|d| d.as_i64()).collect())
                            .unwrap_or_default(),
                        struct_bytes: fo.get("struct_bytes").and_then(|x| x.as_u64()).unwrap_or(0) as usize,
                    })
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()?
        .unwrap_or_default();
    let mut methods = rustc_hash::FxHashMap::default();
    if let Some(mo) = get("methods").as_object() {
        for (name, mj) in mo {
            methods.insert(name.clone(), decode_func(mj)?);
        }
    }
    let properties = get("properties")
        .as_array()
        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    Ok(ClassInfo {
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
        layout: Default::default(),
        vorlage: Default::default(),
    })
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
    // Bytecode einer anderen Fassung: Befehle und ihre Bedeutung koennen sich
    // geaendert haben -- lieber klar abweisen als still anders rechnen. Eine
    // .dhc ohne Stempel (vor 2026.52) laeuft wie bisher.
    if let Some(f) = obj.get("dhrt").and_then(|v| v.as_str()) {
        let eigen = env!("CARGO_PKG_VERSION");
        if f != eigen {
            return Err(format!(
                "Diese .dhc stammt aus dhrt {}, diese Laufzeit ist {} -- die Quelle mit dieser Fassung neu uebersetzen",
                f.trim_end_matches(".0"), eigen.trim_end_matches(".0")));
        }
    }
    let n_globals = obj.get("n_globals").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let global_names: Vec<String> = obj.get("global_names").and_then(|v| v.as_array())
        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
        .unwrap_or_default();
    let main = decode_func(obj.get("main").ok_or("kein main")?)?;
    let mut functions: Vec<(String, Func)> = Vec::new();
    if let Some(fobj) = obj.get("functions").and_then(|v| v.as_object()) {
        for (name, fj) in fobj {
            functions.push((name.clone(), decode_func(fj)?));
        }
    }
    let mut classes = rustc_hash::FxHashMap::default();
    if let Some(cobj) = obj.get("classes").and_then(|v| v.as_object()) {
        for (name, cj) in cobj {
            classes.insert(name.clone(), decode_class(cj)?);
        }
    }
    let data = werte_lesen(obj.get("data").unwrap_or(&J::Null))?;
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
    main.anzeigen = specialize_args(&mut main.code, &fn_index);
    for f in functions.iter_mut() { f.anzeigen = specialize_args(&mut f.code, &fn_index); }
    for c in classes.values_mut() {
        for m in c.methods.values_mut() { m.anzeigen = specialize_args(&mut m.code, &fn_index); }
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
    // Feste Lage der Felder je Klasse: Vorfahren zuerst, wie allocate_instance
    // sie anlegt.
    let namen: Vec<String> = classes.keys().cloned().collect();
    for k in namen {
        let mut kette: Vec<&ClassInfo> = Vec::new();
        let mut cur = classes.get(k.as_str());
        while let Some(ci) = cur {
            kette.push(ci);
            if ci.parent_name.is_empty() || kette.len() > 64 { break; }
            cur = classes.get(ci.parent_name.as_str());
        }
        let mut lage = crate::value::Layout::default();
        lage.vorfahren.push(k.clone());
        for ci in &kette {
            if !ci.parent_name.is_empty() { lage.vorfahren.push(ci.parent_name.clone()); }
        }
        for ci in kette.iter().rev() {
            for fd in &ci.fields {
                let ty = if fd.array_dims.is_empty() { fd.type_name.clone() } else { format!("array:{}", fd.type_name) };
                match lage.index.get(&fd.name) {
                    Some(&i) => lage.typen[i as usize] = ty,
                    None => { lage.index.insert(fd.name.clone(), lage.typen.len() as u32); lage.typen.push(ty); }
                }
            }
        }
        if let Some(ci) = classes.get_mut(&k) { ci.layout = std::rc::Rc::new(lage); }
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
            lines: Vec::new(), local_names: Vec::new(), anzeigen: Vec::new(),
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

    /// Ein kleines Programm mit allem, was in einer .dhc vorkommt: Funktion,
    /// Klasse mit Feld und Methode, Lambda, Tupel, FLOAT, Text, DATA.
    fn beispiel_dhc() -> J {
        let quelle = r#"
CLASS Punkt
  DIM x AS FLOAT
  FUNCTION laenge() AS FLOAT
    RETURN ABS(x) * 1.5
  END FUNCTION
END CLASS
FUNCTION doppelt(n AS INTEGER, z AS STRING = "a") AS INTEGER
  RETURN n * 2
END FUNCTION
DIM p AS Punkt = NEW Punkt()
p.x = -2.25
DIM f AS FUNCREF : f = FUNCTION(k) k + 1
DIM t AS TUPLE : t = (1, "zwei", 3.5, TRUE)
DIM i AS INTEGER
FOR i = 1 TO 3
  PRINT doppelt(i); f(i); p.laenge(); t
NEXT
DATA 1, "x", 2.5
"#;
        let toks = crate::lexer::Lexer::new(quelle).tokenize().unwrap_or_else(|e| panic!("lexen: {}", e.msg));
        let ast = crate::parser::Parser::new(toks).parse().unwrap_or_else(|e| panic!("parsen {}: {}", e.line, e.msg));
        let leer = std::collections::HashSet::new();
        crate::compiler::compile_to_gbc(&ast, &leer, &[], &leer, &[], "probe.dh").expect("uebersetzen").0
    }

    /// Alle Stellen eines JSON-Baums als Zeiger (`/main/code/3/1`).
    fn stellen(j: &J, pfad: String, aus: &mut Vec<String>) {
        aus.push(pfad.clone());
        match j {
            J::Array(a) => for (i, v) in a.iter().enumerate() { stellen(v, format!("{}/{}", pfad, i), aus) },
            J::Object(o) => for (k, v) in o { stellen(v, format!("{}/{}", pfad, k), aus) },
            _ => {}
        }
    }

    /// Eine beschaedigte .dhc ist ein Lade-Fehler, keine Panik. Bis 2026.53
    /// endete `{"format":"dhc","main":"kaputt"}` mit Exit 101. Jede Stelle
    /// der Datei wird gegen jeden Unsinn getauscht; `load_program` darf
    /// abweisen oder laden, aber nie in Panik geraten.
    #[test]
    fn kaputte_dhc_ist_ein_lade_fehler() {
        let gut = beispiel_dhc();
        assert!(load_program(&gut).is_ok());
        let fall = serde_json::json!({"format": "dhc", "main": "kaputt"});
        let e = load_program(&fall).err().expect("abgewiesen");
        assert!(e.contains("Funktion ist kein Objekt"), "{}", e);

        let mut alle = Vec::new();
        stellen(&gut, String::new(), &mut alle);
        let unsinn = [J::Null, serde_json::json!("x"), serde_json::json!(-1), serde_json::json!(1.5),
                      serde_json::json!(true), serde_json::json!([]), serde_json::json!({}),
                      serde_json::json!({"q": 1}), serde_json::json!([1]), serde_json::json!(1e300)];
        let mut paniken = Vec::new();
        for stelle in alle.iter().filter(|s| !s.is_empty()) {
            for u in &unsinn {
                let mut j = gut.clone();
                *j.pointer_mut(stelle).unwrap() = u.clone();
                let r = std::panic::catch_unwind(|| { let _ = load_program(&j); });
                if r.is_err() {
                    // Der Befehl, in dem die Stelle liegt, sagt am meisten.
                    let befehl: Vec<&str> = stelle.split('/').take(4).collect();
                    let befehl = gut.pointer(&befehl.join("/")).map(|b| b.to_string()).unwrap_or_default();
                    paniken.push(format!("{} = {} (in {})", stelle, u, befehl));
                }
            }
        }
        assert!(paniken.is_empty(), "{} Paniken beim Laden, etwa: {:?}", paniken.len(),
                &paniken[..paniken.len().min(20)]);
    }
}
