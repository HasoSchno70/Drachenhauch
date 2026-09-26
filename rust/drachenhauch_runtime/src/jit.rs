//! Maschinencode fuer reine Zahlenfunktionen (M3, docs/entwurf-maschinencode.md).
//!
//! Uebersetzt wird eine FUNCTION/SUB nur, wenn sie REIN ist: Parameter,
//! Locals und Rueckgabe sind INTEGER, FLOAT oder BOOLEAN, und sie tut nichts
//! als rechnen, vergleichen, springen, globale Zahl-Variablen lesen und
//! schreiben und andere solche Funktionen rufen. Keine Builtins, keine
//! Ausgabe.
//!
//! **Globale Variablen im Schatten (M4 Schritt 1):** der Maschinencode liest
//! und schreibt sie in einer Schattenkopie (`Kontext::schatten`, je Platz ein
//! Merkbyte: 0 = noch nicht geholt, 1 = geholt, 2 = geaendert). Erst wenn der
//! Aufruf OHNE Fehler zurueckkommt, schreibt die VM die geaenderten Plaetze
//! in ihre Slots. Damit bleibt die Funktion fuer die VM rein: gibt der
//! Maschinencode auf, ist nichts geschehen, und die VM rechnet von vorn.
//!
//! **Schleifen (M4 Schritt 2/3)** werden beim ersten Ruecksprung als Bereich
//! uebersetzt, mit den Arten, die ihre Locals da haben; sie duerfen Felder
//! von INTEGER/FLOAT lesen und schreiben. Ein Fehler mitten im Bereich gibt
//! NICHT auf, sondern steigt am fehlerhaften Befehl aus (`Aussteig`): der
//! Stand davor geht an die VM, und sie fuehrt den Befehl selbst aus.
//!
//! **Die VM bleibt die Wahrheit** -- und weil eine reine Funktion keine
//! Nebenwirkung hat, ist das hier woertlich zu nehmen: bei JEDEM Fehler
//! (Ueberlauf, Division durch 0, verlustbehaftete Umwandlung, fehlendes
//! RETURN, zu tiefe Rekursion, Vergleich mit NaN) gibt der Maschinencode
//! auf, und die VM rechnet denselben Aufruf noch einmal. Sie liefert dann
//! genau ihre Meldung mit genau ihrer Zeile. Es gibt also keine zweite
//! Fassung einer Fehlermeldung, nur eine zweite Fassung des Rechnens -- und
//! die prueft `tests/pruef/jit.dhtest` samt der ganzen Sammlung mit und
//! ohne Maschinencode.
//!
//! **Seit M5 an per Vorgabe**; `DHRT_JIT=aus` laesst alles in der VM (die CI
//! faehrt beide Wege).

use std::collections::HashMap;

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{types, AbiParam, Block, InstBuilder, MemFlagsData, Type, UserFuncName};
use cranelift_codegen::ir::Value as CWert;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, FuncId, Linkage, Module};

use std::cell::RefCell;
use std::rc::Rc;

use crate::model::{op, Arg, Func, Program};
use crate::value::{value_eq, Value};
use crate::vm::Slot;

/// Die Art eines Wertes im Maschinencode. `N` ist das NIL, das der Aufruf
/// einer SUB auf den Stapel legt -- es darf nur wieder weggeworfen werden.
/// `Feld(nr)` ist ein Feld von INTEGER oder FLOAT, das eine Schleife beim
/// Eintritt vorfindet (nur in Bereichen, siehe `FeldInfo`); der Wert im
/// Maschinencode ist bedeutungslos, die Daten beschreibt der Kontext.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Art { I, F, B, N, Feld(u16), Obj(u16), W }

/// `W` (M4 Schritt 5): ein beliebiger Wert -- Text, MAP, Feld, Objekt --, der
/// in einem Werteplatz des Kontexts liegt (`Kontext::werte`: je Local ein
/// Platz, dahinter je Stapeltiefe einer). Der Maschinencode fasst ihn nie
/// selbst an, nur ueber die Helfer `w_*`; das Zaehlen der Verweise bleibt beim
/// Rust-Code. Nur in Bereichen im Wertemodus (`Bereich::modus_w`).

/// Was in einem Feld steht: Zahlen direkt (Speicher der VM) oder Werte
/// (ein Feld von Werten oder ein Tupel) -- dann mit der Art, die jedes
/// Element haben muss; ein Element anderer Art laesst aussteigen.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Elem { I, F, Wert(Art) }

/// Eine Klasse, deren Objekte im Bereich vorkommen: ihre Lage (die
/// Instanzen zeigen darauf -- dieselbe Lage heisst dieselbe Klasse) und ihr
/// Name (Schluessel in `Program::classes`). Objekte sind im Maschinencode
/// Zeiger auf `RefCell<Instance>`; ihre Felder liest und schreibt er ueber
/// die Helfer `feld_lesen_*`/`feld_setzen_*`, weil die Lage eines `Value`
/// im Speicher nicht festgelegt ist.
#[derive(Clone, Debug)]
struct Klasse {
    lage: *const crate::value::Layout,
    name: String,
}

fn klasse_nr(klassen: &mut Vec<Klasse>, lage: *const crate::value::Layout, name: &str) -> u16 {
    match klassen.iter().position(|k| std::ptr::eq(k.lage, lage)) {
        Some(i) => i as u16,
        None => { klassen.push(Klasse { lage, name: name.to_string() }); (klassen.len() - 1) as u16 }
    }
}

/// Art eines einzelnen Wertes, Objekte eingeschlossen.
fn wert_art(v: &Value, klassen: &mut Vec<Klasse>) -> Option<Art> {
    match v {
        Value::Instance(rc) => {
            let b = rc.borrow();
            Some(Art::Obj(klasse_nr(klassen, Rc::as_ptr(&b.layout), &b.class_name)))
        }
        _ => art_von_wert(v),
    }
}

/// Platz und Art des Feldes `name` einer Klasse. `mit_props`: eine PROPERTY
/// dieses Namens laesst den Bereich in der VM (LOAD/STORE_MEMBER fragen
/// danach, LOAD/STORE_FIELD in einer Methode nicht). Ein Feld, das selbst ein
/// Objekt haelt, hat die Art seiner deklarierten Klasse.
fn feld_von(prog: &Program, klassen: &mut Vec<Klasse>, k: usize, name: &str, mit_props: bool) -> Result<(usize, Art), String> {
    let kl = klassen[k].clone();
    let lage = unsafe { &*kl.lage };
    if mit_props {
        if let Some(ci) = prog.classes.get(kl.name.as_str()) {
            if ci.props_alle.contains(&name.to_lowercase()) { return Err(format!("PROPERTY {}", name)); }
        }
    }
    let platz = *lage.index.get(name).ok_or_else(|| format!("{} hat kein Feld {}", kl.name, name))? as usize;
    let typ = lage.typen[platz].as_str();
    let a = match art_von_typ(typ) {
        Some(a) => a,
        None => match prog.classes.get(typ) {
            Some(ci) if !typ.is_empty() => Art::Obj(klasse_nr(klassen, Rc::as_ptr(&ci.layout), typ)),
            _ => return Err(format!("Feld {} vom Typ '{}'", name, typ)),
        },
    };
    Ok((platz, a))
}

/// Woher ein Feld einer Schleife kommt: aus einem Local oder einem globalen
/// Platz -- beim Eintritt holt die VM es von dort und beschreibt es im
/// Kontext (Zeiger, Groessen, Schritte).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Quelle { Lokal(usize), Global(usize) }

#[derive(Clone, Copy, PartialEq, Debug)]
struct FeldInfo {
    quelle: Quelle,
    elem: Elem,
    dims: u8,
    /// Ein Tupel (unveraenderlich, eine Dimension) statt eines Feldes.
    tupel: bool,
}

impl FeldInfo {
    /// Die Art eines gelesenen Elements.
    fn art(&self) -> Art {
        match self.elem { Elem::I => Art::I, Elem::F => Art::F, Elem::Wert(a) => a }
    }
    /// Bytes je Element: 8 fuer Zahlen, sonst ein ganzer `Value`.
    fn breite(&self) -> i64 {
        match self.elem { Elem::Wert(_) => std::mem::size_of::<Value>() as i64, _ => 8 }
    }
}

/// Ein Feld von Zahlen (INTEGER/FLOAT), ein Feld von Werten oder ein Tupel,
/// dessen erstes Element eine Zahl, ein Wahrheitswert oder ein Objekt ist:
/// (Element, Dimensionen, Tupel?). Alles andere bleibt der VM.
fn feld_beschreiben(v: &Value, klassen: &mut Vec<Klasse>) -> Option<(Elem, u8, bool)> {
    match v {
        Value::Array(a) => {
            let a = a.borrow();
            if a.dims.is_empty() || a.dims.len() > 8 { return None; }
            let d = a.dims.len() as u8;
            match &a.cells {
                crate::value::Cells::Int(_) => Some((Elem::I, d, false)),
                crate::value::Cells::Float(_) => Some((Elem::F, d, false)),
                crate::value::Cells::Val(w) => Some((Elem::Wert(wert_art(w.first()?, klassen)?), d, false)),
            }
        }
        Value::Tuple(t) => Some((Elem::Wert(wert_art(t.first()?, klassen)?), 1, true)),
        _ => None,
    }
}

/// Passt ein Wert beim Eintritt noch zu dem Feld, fuer das gebaut wurde?
/// Bei Werten wird nur die Form geprueft -- jedes Element prueft der
/// Maschinencode beim Lesen.
fn feld_passt(v: &Value, fi: &FeldInfo) -> bool {
    match (v, fi.elem) {
        (Value::Tuple(t), Elem::Wert(_)) => fi.tupel && !t.is_empty(),
        (Value::Array(a), e) => {
            let a = a.borrow();
            !fi.tupel && a.dims.len() == fi.dims as usize && matches!((&a.cells, e),
                (crate::value::Cells::Int(_), Elem::I) | (crate::value::Cells::Float(_), Elem::F)
                | (crate::value::Cells::Val(_), Elem::Wert(_)))
        }
        _ => false,
    }
}

fn art_von_typ(t: &str) -> Option<Art> {
    match t {
        "integer" => Some(Art::I),
        "float" => Some(Art::F),
        "boolean" => Some(Art::B),
        _ => None,
    }
}

fn art_von_wert(v: &Value) -> Option<Art> {
    match v {
        Value::Int(_) => Some(Art::I),
        Value::Float(_) => Some(Art::F),
        Value::Bool(_) => Some(Art::B),
        _ => None,
    }
}

fn zahl(a: Art) -> bool { matches!(a, Art::I | Art::F) }
fn skalar(a: Art) -> bool { matches!(a, Art::I | Art::F | Art::B) }

/// Geteilt zwischen VM und Maschinencode (Versaetze fest: 0, 8, 16).
#[repr(C)]
pub struct Kontext {
    pub fehler: u64,
    pub tiefe: u64,
    pub grenze: u64,
    /// Schatten der globalen Plaetze (Werte als u64) und je Platz ein
    /// Merkbyte (0 frei, 1 geholt, 2 geaendert) -- Versatz 24 und 32.
    schatten: *mut u64,
    marken: *mut u8,
    /// Die Slots der VM und je Platz die erwartete Art (1 I, 2 F, 3 B) --
    /// nur fuer `global_holen`.
    slots: *const Option<Rc<RefCell<Slot>>>,
    n_slots: u64,
    arten: *const u8,
    /// Nur Bereiche: der Stapel beim Aussteigen mitten in der Schleife
    /// (Versatz 64) und die Beschreibung der Felder (Versatz 72): je Feld
    /// Zeiger auf die Daten, dann die Groessen, dann die Schritte.
    stapel: *mut u64,
    felder: *const u64,
    /// Nur Bereiche in einer Methode: `Self` als Zeiger (Versatz 80).
    selbst: u64,
    /// Nur Wertemodus: die Werteplaetze (Versatz 88; Local s = Platz s,
    /// Stapeltiefe d = Platz n_lokal_gesamt + d).
    werte: *mut Value,
    /// Nur Wertemodus: die VM, fuer Befehle der uebrigen Familien (Grafik,
    /// gui, Datei ...), und die Meldung, wenn einer davon scheitert -- die
    /// VM gibt sie an seiner Stelle aus, statt ihn noch einmal zu rufen.
    vm: *mut crate::vm::Vm<'static>,
    meldung: Option<String>,
    /// Nur Funktionen (nicht Bereiche): jede geschriebene Objekt-Feld mit dem
    /// alten Wert -- gibt der Maschinencode auf, stellt der Einstieg sie
    /// zurueck, und die VM rechnet den Aufruf von vorn.
    journal: *mut Vec<(u64, u64, Value)>,
    /// Nur Bereiche: das Journal, das der Bereich fuer die Dauer eines
    /// Methodenaufrufs nach `journal` legt. Gibt die Methode auf, steigt der
    /// Bereich vor dem Aufruf aus, und die VM ruft sie noch einmal -- ihre
    /// Schreibungen muessen dann zurueck sein. Die eigenen Schreibungen des
    /// Bereichs gehoeren NICHT hinein (sie gelten, und sie waeren bei
    /// 10 000 Objekten mal 100 Runden ein unnoetig grosses Journal).
    journal_bereich: *mut Vec<(u64, u64, Value)>,
}

const K_JOURNAL: i32 = std::mem::offset_of!(Kontext, journal) as i32;
const K_JOURNAL_BEREICH: i32 = std::mem::offset_of!(Kontext, journal_bereich) as i32;

/// Nach einem Methodenaufruf aus einem Bereich: gab sie auf, ihre
/// Feldschreibungen zuruecknehmen, sonst vergessen; das Journal wieder aus.
/// Reine Zahlenbefehle, die Cranelift nicht als Anweisung hat (M4 Schritt 8).
/// Dieselben Rust-Funktionen wie in builtins.rs -- darum bitgleich.
extern "C" fn mathe1(art: i64, x: f64) -> f64 {
    match art {
        0 => x.sin(), 1 => x.cos(), 2 => x.tan(), 3 => x.atan(),
        4 => x.exp(), 5 => x.ln(), 6 => x.asin(), _ => x.acos(),
    }
}

extern "C" fn mathe2(art: i64, x: f64, y: f64) -> f64 {
    if art == 0 { x.atan2(y) } else { x.hypot(y) }
}

/// Welche eingebauten Befehle der getypte Maschinencode selbst rechnet, und
/// mit welcher Art Ergebnis. Nur Zahlen (INTEGER/FLOAT) als Argumente; jeder
/// Fehlerfall der VM (ABS von MIN, SQR negativ, INT ausserhalb, LOG <= 0,
/// ASIN/ACOS ausserhalb [-1, 1]) steigt aus, und die VM meldet ihn. Keiner
/// dieser Namen wird von einer frueheren Familie beantwortet (nachgesehen),
/// der Merkplatz braucht also nicht gefragt zu werden.
fn zahl_befehl(name: &str, arts: &[Art]) -> Option<Art> {
    if arts.is_empty() || !arts.iter().all(|a| matches!(a, Art::I | Art::F)) { return None; }
    let gleich = arts.iter().all(|a| *a == arts[0]);
    match (name, arts.len()) {
        ("abs", 1) => Some(arts[0]),
        ("int" | "floor" | "ceil" | "round" | "sgn" | "sign", 1) => Some(Art::I),
        ("flt" | "sqr" | "sqrt" | "sin" | "cos" | "tan" | "atan" | "exp" | "log"
         | "asin" | "acos" | "deg" | "rad" | "frac", 1) => Some(Art::F),
        ("atan2" | "hypot", 2) => Some(Art::F),
        ("lerp", 3) => Some(Art::F),
        // Sie liefern einen der WERTE -- seine Art steht nur fest, wenn alle
        // dieselbe haben.
        ("min" | "max", _) if gleich => Some(arts[0]),
        ("clamp", 3) if gleich => Some(arts[0]),
        _ => None,
    }
}

/// Eine vorbelegte Konstante (Farbe, Taste, PI/TAU), die `LOAD_NAME` laedt:
/// sie hat keinen Platz, ist unveraenderlich (Zuweisen ist ein Fehler) und
/// wird darum als Zahl eingesetzt -- ausser ein Platz heisst genauso, dann
/// hat das Programm den Namen selbst angelegt (M4 Schritt 9).
fn vorbelegt(prog: &Program, f: &Func, ins: &crate::model::Instr) -> Option<Value> {
    let name = match f.constants.get(ins.arg.as_usize()) { Some(Value::Str(n)) => n.to_lowercase(), _ => return None };
    if !crate::vm::ist_vorbelegter_name(&name) || prog.global_names.iter().any(|g| g.eq_ignore_ascii_case(&name)) {
        return None;
    }
    match name.as_str() {
        "pi" => Some(Value::Float(std::f64::consts::PI)),
        "tau" => Some(Value::Float(std::f64::consts::TAU)),
        _ => crate::vm::DEFAULT_COLORS.iter().chain(crate::vm::DEFAULT_KEYS.iter())
            .find(|(n, _)| *n == name).map(|(_, v)| Value::Int(*v)),
    }
}

extern "C" fn journal_schluss(k: *mut Kontext) {
    let j = unsafe { (*k).journal };
    if j.is_null() { return; }
    unsafe {
        if (*k).fehler != 0 { journal_zurueck(&mut *j); } else { (*j).clear(); }
        (*k).journal = std::ptr::null_mut();
    }
}

const K_SCHATTEN: i32 = 24;
const K_MARKEN: i32 = 32;
const K_STAPEL: i32 = 64;
const K_FELDER: i32 = 72;
const K_SELBST: i32 = 80;
/// So viele Werte darf der Stapel an einer Aussteigestelle haben.
const MAX_STAPEL: usize = 64;

/// Aus dem Maschinencode gerufen, wenn ein globaler Platz zum ersten Mal
/// gelesen wird: Wert aus dem Slot der VM in den Schatten. Ein leerer Platz
/// (die Funktion lief vor dem DIM) oder ein Wert anderer Art setzt `fehler`
/// -- dann gibt der Maschinencode auf, und die VM meldet es selbst.
extern "C" fn global_holen(k: *mut Kontext, idx: u64) {
    let k = unsafe { &mut *k };
    let i = idx as usize;
    let slots = unsafe { std::slice::from_raw_parts(k.slots, k.n_slots as usize) };
    let art = unsafe { *k.arten.add(i) };
    let bits = match slots.get(i).and_then(|s| s.as_ref()) {
        Some(sl) => match (&sl.borrow().value, art) {
            (Value::Int(x), 1) => *x as u64,
            (Value::Float(f), 2) => f.to_bits(),
            (Value::Bool(b), 3) => *b as u64,
            _ => { k.fehler = 1; return; }
        },
        None => { k.fehler = 1; return; }
    };
    unsafe {
        *k.schatten.add(i) = bits;
        *k.marken.add(i) = 1;
    }
}

/// Einen Wert als Bits fuer den Maschinencode: `art` 1 INTEGER, 3 BOOLEAN,
/// 4 Objekt der Lage `lage` (dann der Zeiger). Passt er nicht, steigt der
/// Maschinencode aus (`fehler`).
fn wert_bits(k: *mut Kontext, v: Option<&Value>, art: u64, lage: u64) -> i64 {
    match (art, v) {
        (1, Some(Value::Int(x))) => *x,
        (3, Some(Value::Bool(b))) => *b as i64,
        (4, Some(Value::Instance(rc))) if Rc::as_ptr(&rc.borrow().layout) as u64 == lage => Rc::as_ptr(rc) as i64,
        _ => { unsafe { (*k).fehler = 1; } 0 }
    }
}

fn objekt<'a>(obj: u64) -> &'a RefCell<crate::value::Instance> {
    unsafe { &*(obj as *const RefCell<crate::value::Instance>) }
}

extern "C" fn feld_lesen_i(k: *mut Kontext, obj: u64, platz: u64, art: u64, lage: u64) -> i64 {
    match objekt(obj).try_borrow() {
        Ok(b) => wert_bits(k, b.fields.get(platz as usize), art, lage),
        Err(_) => { unsafe { (*k).fehler = 1; } 0 }
    }
}

extern "C" fn feld_lesen_f(k: *mut Kontext, obj: u64, platz: u64) -> f64 {
    match objekt(obj).try_borrow().ok().as_deref().and_then(|b| b.fields.get(platz as usize).cloned()) {
        Some(Value::Float(x)) => x,
        _ => { unsafe { (*k).fehler = 1; } 0.0 }
    }
}

/// Ein Zahlenfeld schreiben; den Wert hat der Maschinencode schon in die Art
/// des Feldes gewandelt (`art` 1 INTEGER, 3 BOOLEAN).
extern "C" fn feld_setzen_i(k: *mut Kontext, obj: u64, platz: u64, art: u64, w: i64) {
    feld_setzen(k, obj, platz, if art == 3 { Value::Bool(w != 0) } else { Value::Int(w) });
}

extern "C" fn feld_setzen_f(k: *mut Kontext, obj: u64, platz: u64, w: f64) {
    feld_setzen(k, obj, platz, Value::Float(w));
}

fn feld_setzen(k: *mut Kontext, obj: u64, platz: u64, v: Value) {
    if let Ok(mut b) = objekt(obj).try_borrow_mut() {
        if let Some(f) = b.fields.get_mut(platz as usize) {
            let alt = std::mem::replace(f, v);
            let j = unsafe { (*k).journal };
            if !j.is_null() { unsafe { (*j).push((obj, platz, alt)); } }
        }
    }
}

/// Die Feldschreibungen eines aufgegebenen Aufrufs zuruecknehmen (rueckwaerts).
fn journal_zurueck(j: &mut Vec<(u64, u64, Value)>) {
    while let Some((obj, platz, alt)) = j.pop() {
        if let Ok(mut b) = objekt(obj).try_borrow_mut() {
            if let Some(f) = b.fields.get_mut(platz as usize) { *f = alt; }
        }
    }
}

/// Ein Element eines Feldes von Werten oder eines Tupels (`p` zeigt auf den
/// `Value`; die Grenze hat der Maschinencode schon geprueft).
extern "C" fn element_i(k: *mut Kontext, p: u64, art: u64, lage: u64) -> i64 {
    wert_bits(k, Some(unsafe { &*(p as *const Value) }), art, lage)
}

extern "C" fn element_f(k: *mut Kontext, p: u64) -> f64 {
    match unsafe { &*(p as *const Value) } {
        Value::Float(x) => *x,
        _ => { unsafe { (*k).fehler = 1; } 0.0 }
    }
}

// --- Helfer fuer Werte (Wertemodus). Jeder Helfer, der scheitern kann, legt
// seine Operanden zurueck, bevor er `fehler` setzt: der Bereich steigt dann an
// diesem Befehl aus, und die VM findet den Stand davor.

fn w_platz<'a>(k: *mut Kontext, i: u64) -> &'a mut Value {
    unsafe { &mut *(*k).werte.add(i as usize) }
}
/// Einen Wert herausnehmen (der Platz wird NIL).
fn nimm(v: &mut Value) -> Value { std::mem::replace(v, Value::Nil) }
fn w_nehmen(k: *mut Kontext, i: u64) -> Value { nimm(w_platz(k, i)) }
fn w_fehler(k: *mut Kontext) { unsafe { (*k).fehler = 1; } }

extern "C" fn w_konst(k: *mut Kontext, p: u64, ziel: u64) {
    *w_platz(k, ziel) = unsafe { (*(p as *const Value)).clone() };
}
extern "C" fn w_kopie(k: *mut Kontext, von: u64, ziel: u64) {
    let v = w_platz(k, von).clone();
    *w_platz(k, ziel) = v;
}
extern "C" fn w_frei(k: *mut Kontext, i: u64) { drop(w_nehmen(k, i)); }
extern "C" fn w_ablegen_i(k: *mut Kontext, bits: i64, art: u64, ziel: u64) {
    *w_platz(k, ziel) = if art == 3 { Value::Bool(bits != 0) } else { Value::Int(bits) };
}
extern "C" fn w_ablegen_f(k: *mut Kontext, x: f64, ziel: u64) { *w_platz(k, ziel) = Value::Float(x); }

/// Einen Wert als Zahl heraus (in einen Platz der Art `art`: 1 INTEGER,
/// 3 BOOLEAN). Die Regeln wie `passend!`: INTEGER bleibt INTEGER; alles
/// andere -- auch eine Kommazahl mit Nachkommastellen -- rechnet die VM.
extern "C" fn w_zahl_i(k: *mut Kontext, i: u64, art: u64) -> i64 {
    match (art, w_platz(k, i)) {
        (1, Value::Int(x)) => { let x = *x; *w_platz(k, i) = Value::Nil; x }
        (3, Value::Bool(b)) => { let b = *b; *w_platz(k, i) = Value::Nil; b as i64 }
        _ => { w_fehler(k); 0 }
    }
}
extern "C" fn w_zahl_f(k: *mut Kontext, i: u64) -> f64 {
    match w_platz(k, i) {
        Value::Float(x) => { let x = *x; *w_platz(k, i) = Value::Nil; x }
        Value::Int(x) => { let x = *x as f64; *w_platz(k, i) = Value::Nil; x }
        _ => { w_fehler(k); 0.0 }
    }
}

/// Einen Wert in einen Platz mit Typ legen (`typ` als Text, wie im
/// Programm): mit `coerce` wie die VM; passt er nicht, steigt der
/// Bereich aus, und die VM meldet es.
extern "C" fn w_speichern(k: *mut Kontext, von: u64, ziel: u64, typ: u64, typ_len: u64) {
    let t = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(typ as *const u8, typ_len as usize)) };
    let v = w_nehmen(k, von);
    match crate::vm::coerce(v.clone(), t, "Lokale Variable") {
        Ok(cv) => *w_platz(k, ziel) = cv,
        Err(_) => { *w_platz(k, von) = v; w_fehler(k); }
    }
}

/// `a op b` fuer zwei schlichte Werte (Text, Zahl, Wahrheitswert) mit den
/// Regeln der VM. Vergleiche liefern 0/1, alles andere legt das Ergebnis in
/// `ziel`. `lokal` >= 0: der Platz, in den das Ergebnis geht (`x = x + e`) --
/// haelt er denselben Text wie `a`, gibt er ihn ab, damit angehaengt statt
/// kopiert wird (wie ADD_STORE_* in der VM). Alles andere (Objekte mit
/// OPERATOR, Felder, NaN ...) laesst aussteigen.
extern "C" fn w_op(k: *mut Kontext, o: u64, a_i: u64, b_i: u64, ziel: u64, lokal: i64) -> i64 {
    let schlicht = |v: &Value| matches!(v, Value::Int(_) | Value::Float(_) | Value::Str(_) | Value::Bool(_));
    let a = w_nehmen(k, a_i);
    let b = w_nehmen(k, b_i);
    // Der Platz gibt seinen Verweis GANZ ab (sofort fallen gelassen, sonst
    // hielten zwei den Text, und es wuerde nie angehaengt); zurueck kommt er
    // notfalls als Kopie von `a` -- es ist derselbe Text.
    let mut abgegeben = false;
    if lokal >= 0 {
        if let (Value::Str(ra), Value::Str(rl)) = (&a, &*w_platz(k, lokal as u64)) {
            if Rc::ptr_eq(ra, rl) { drop(w_nehmen(k, lokal as u64)); abgegeben = true; }
        }
    }
    let zurueck = |k: *mut Kontext, a: Value, b: Value, abgegeben: bool| {
        if abgegeben { *w_platz(k, lokal as u64) = a.clone(); }
        *w_platz(k, a_i) = a;
        *w_platz(k, b_i) = b;
        w_fehler(k);
    };
    if !schlicht(&a) || !schlicht(&b) { zurueck(k, a, b, abgegeben); return 0; }
    let o = o as u16;
    let vergleich = match o {
        op::EQ => Some(Ok(value_eq(&a, &b))),
        op::NEQ => Some(Ok(!value_eq(&a, &b))),
        op::LT => Some(crate::vm::cmp(&a, &b, '<')),
        op::GT => Some(crate::vm::cmp(&a, &b, '>')),
        op::LEQ => Some(crate::vm::cmp(&a, &b, 'l')),
        op::GEQ => Some(crate::vm::cmp(&a, &b, 'g')),
        _ => None,
    };
    if let Some(r) = vergleich {
        return match r { Ok(x) => x as i64, Err(_) => { zurueck(k, a, b, abgegeben); 0 } };
    }
    // Anhaengen an Ort und Stelle: `a` ist ein Text, den sonst niemand haelt.
    if o == op::ADD {
        if let Value::Str(mut links) = a {
            if let Some(t) = Rc::get_mut(&mut links) {
                match &b { Value::Str(r) => t.push_str(r), _ => t.push_str(&b.fmt()) }
                *w_platz(k, ziel) = Value::Str(links);
                return 0;
            }
            let a = Value::Str(links);
            return match crate::vm::wert_rechnen("+", &a, &b) {
                Some(v) => { *w_platz(k, ziel) = v; 0 }
                None => { zurueck(k, a, b, abgegeben); 0 }
            };
        }
    }
    let zeichen = match o {
        op::ADD => "+", op::SUB => "-", op::MUL => "*", op::DIV => "/",
        op::INT_DIV => "\\", op::MOD => "mod", _ => { zurueck(k, a, b, abgegeben); return 0; }
    };
    match crate::vm::wert_rechnen(zeichen, &a, &b) {
        Some(v) => { *w_platz(k, ziel) = v; 0 }
        None => { zurueck(k, a, b, abgegeben); 0 }
    }
}

extern "C" fn w_wahr(k: *mut Kontext, i: u64) -> i64 { w_nehmen(k, i).truthy() as i64 }

/// Darf ein Befehl der Familie `f` (Nummer aus `Vm::familie_rufen`) im
/// Wertemodus laufen? Nicht, was Drachenhauch-Code ruft -- dessen Locals und
/// Globale liegen waehrend des Bereichs in den Werteplaetzen: SORT mit
/// Vergleich (2), Coroutinen (4), TIMER_UPDATE und GUI_UPDATE (Rueckrufe),
/// Auftraege (8, lesen die Globalen). Und nicht ASSERT (braucht die Zeile).
fn befehl_im_bereich(f: u8, name: &str) -> bool {
    f != 0 && !matches!(f, 2 | 4 | 8) && !name.starts_with("assert")
        && !matches!(name, "gui_update" | "timer_update")
}

/// Ein eingebauter Befehl aus der Familie der REINEN (`builtins::call_builtin`
/// -- die VM hat an dieser Stelle schon einmal genau sie gefragt,
/// `Instr::familie`). Argumente aus den Plaetzen `basis..basis+argc`, das
/// Ergebnis nach `basis` (art 5) oder als Zahl (1 INTEGER, 3 BOOLEAN).
fn w_builtin_roh(k: *mut Kontext, ins: u64, basis: u64, argc: u64) -> Option<Value> {
    let ins = unsafe { &*(ins as *const crate::model::Instr) };
    let name = match &ins.arg { Arg::Call(n, _, _) => &n[..], _ => { w_fehler(k); return None; } };
    // Die Argumente liegen hintereinander in den Plaetzen -- als Scheibe
    // hinein, ohne sie herauszunehmen (kein Vec je Aufruf). Erst nach Erfolg
    // werden die Plaetze frei.
    let args: &[Value] = unsafe { std::slice::from_raw_parts((*k).werte.add(basis as usize), argc as usize) };
    if ins.familie.get() != crate::vm::BUILTIN_FAMILIEN {
        // Eine andere Familie: derselbe Weg wie CALL_BUILTIN in der VM. Ein
        // Fehler wird NICHT nachgerechnet -- der Befehl hat womoeglich schon
        // etwas getan (gezeichnet, geschrieben); die VM meldet ihn an seiner
        // Stelle.
        let vm = unsafe { &mut *(*k).vm };
        return match vm.builtin_rufen(name, args, &ins.familie) {
            Ok(v) => { for j in 0..argc { drop(w_nehmen(k, basis + j)); } Some(v) }
            Err(e) => { unsafe { (*k).meldung = Some(e); } w_fehler(k); None }
        };
    }
    match crate::builtins::call_builtin(name, args) {
        Some(Ok(v)) => {
            for j in 0..argc { drop(w_nehmen(k, basis + j)); }
            Some(v)
        }
        _ => { w_fehler(k); None }
    }
}
extern "C" fn w_builtin_i(k: *mut Kontext, ins: u64, basis: u64, argc: u64, art: u64) -> i64 {
    let Some(v) = w_builtin_roh(k, ins, basis, argc) else { return 0 };
    match (art, v) {
        (5, v) => { *w_platz(k, basis) = v; 0 }
        (1, Value::Int(x)) => x,
        (3, Value::Bool(b)) => b as i64,
        // Die Tabelle der Ergebnis-Typen (typen::builtin_typ) haette nicht
        // gestimmt -- sie ist mit DHRT_TYPEN_PRUEFEN geprueft. Der Befehl ist
        // gelaufen; der Wert kommt als Wert, und der Bereich steigt aus.
        (_, v) => { *w_platz(k, basis) = v; w_fehler(k); 0 }
    }
}
extern "C" fn w_builtin_f(k: *mut Kontext, ins: u64, basis: u64, argc: u64) -> f64 {
    match w_builtin_roh(k, ins, basis, argc) {
        Some(Value::Float(x)) => x,
        Some(v) => { *w_platz(k, basis) = v; w_fehler(k); 0.0 }
        None => 0.0,
    }
}

/// `obj.methode(args)`: das Objekt in `basis`, die Argumente dahinter; das
/// Ergebnis nach `basis`. Derselbe Weg wie CALL_METHOD in der VM
/// (`Vm::methode_rufen`: Merkplatz, Coroutine, Methoden von Text/Feld/MAP).
/// Ob die Methode laufen darf, hat `methoden_harmlos` beim Bauen geprueft.
/// Ein Fehler wird nicht nachgerechnet -- die Methode hat womoeglich schon
/// etwas getan; die VM meldet ihn an dieser Stelle.
extern "C" fn w_methode(k: *mut Kontext, ins: u64, basis: u64, argc: u64) {
    let ins: &'static crate::model::Instr = unsafe { &*(ins as *const crate::model::Instr) };
    let name = match &ins.arg { Arg::Call(n, _, _) => &n[..], _ => { w_fehler(k); return; } };
    let obj = w_nehmen(k, basis);
    let args: Vec<Value> = (1..=argc).map(|j| w_nehmen(k, basis + j)).collect();
    let vm = unsafe { &mut *(*k).vm };
    match vm.methode_rufen(ins, name, obj, args) {
        Ok(v) => *w_platz(k, basis) = v,
        Err(e) => { unsafe { (*k).meldung = Some(e); } w_fehler(k); }
    }
}

/// PRINT im Wertemodus (M4 Schritt 10): die Werte von den Plaetzen nehmen und
/// denselben Code der VM rufen (`Vm::drucken`). Kann nicht scheitern.
extern "C" fn w_drucken(k: *mut Kontext, ins: u64, basis: u64, n: u64) {
    let ins: &'static crate::model::Instr = unsafe { &*(ins as *const crate::model::Instr) };
    let items: Vec<Value> = (0..n).map(|j| w_nehmen(k, basis + j)).collect();
    let vm = unsafe { &mut *(*k).vm };
    vm.drucken(&ins.arg, &items);
}

/// Plaetze, die ein Befehl anfasst; Namen (LOAD_NAME ...) ueber `global_names`.
fn globale_plaetze(prog: &Program, g: &Func, ins: &crate::model::Instr) -> Vec<usize> {
    match ins.op {
        op::LOAD_GLOBAL_SLOT | op::STORE_GLOBAL_SLOT | op::ADD_STORE_GLOBAL_SLOT => vec![ins.arg.as_usize()],
        op::FOR_NEXT => match for_teile(&ins.arg) { Some(t) if t[0] == 1 => vec![t[1] as usize], _ => vec![] },
        op::LOAD_NAME | op::STORE_NAME | op::INPUT_NAME => match g.constants.get(ins.arg.as_usize()) {
            Some(Value::Str(n)) => prog.global_names.iter()
                .position(|x| !x.is_empty() && x.eq_ignore_ascii_case(n)).into_iter().collect(),
            _ => vec![],
        },
        _ => vec![],
    }
}

/// Darf ein Bereich `obj.name(...)` rufen? Die Methode laeuft in der VM,
/// waehrend der Bereich Locals und manche Globale bei sich haelt (Register,
/// Werteplaetze, Schatten). Also: keine Methode dieses Namens -- in keiner
/// Klasse, der Empfaenger steht erst beim Laufen fest -- und nichts, was sie
/// ruft, darf eine Globale beruehren, die der Bereich beruehrt; und nichts
/// darf Drachenhauch-Code ueber Umwege rufen (FUNCREF, Coroutinen,
/// Rueckrufe) oder das Programm beenden.
fn methoden_harmlos<'a>(prog: &'a Program, f: &'a Func, von: usize, bis: usize, name: &str) -> Result<(), String> {
    // Die Globalen des Bereichs samt aller freien Funktionen, die er ruft
    // (die lesen und schreiben ueber den Schatten).
    let mut bereich: Vec<usize> = Vec::new();
    let mut offen: Vec<&'a Func> = Vec::new();
    let mut gesehen: Vec<*const Func> = Vec::new();
    let rufe = |ins: &crate::model::Instr, offen: &mut Vec<&'a Func>| {
        if ins.op == op::CALL_USER {
            if let Arg::Call(_, _, i) = &ins.arg { if let Some(h) = prog.functions.get(*i as usize) { offen.push(h); } }
        }
    };
    for ins in &f.code[von..=bis] { bereich.extend(globale_plaetze(prog, f, ins)); rufe(ins, &mut offen); }
    while let Some(g) = offen.pop() {
        if gesehen.contains(&(g as *const Func)) { continue; }
        gesehen.push(g);
        for ins in &g.code { bereich.extend(globale_plaetze(prog, g, ins)); rufe(ins, &mut offen); }
    }
    // Alle Methoden dieses Namens, dazu die Operatoren und PROPERTYs, die sie
    // ueber `a + b` bzw. `obj.x` erreichen, und alles, was die rufen.
    let methoden = |n: &str| -> Vec<&'a Func> {
        prog.classes.values().flat_map(|c| c.methods.iter())
            .filter(|(m, _)| m.eq_ignore_ascii_case(n)).map(|(_, g)| g).collect()
    };
    let mut offen: Vec<&'a Func> = methoden(name);
    if offen.is_empty() && !crate::vm::ist_container_methode(name) {
        return Err(format!("Methode {} (keine Klasse hat sie)", name));
    }
    for c in prog.classes.values() {
        for (m, g) in &c.methods { if m.starts_with("__op_") { offen.push(g); } }
    }
    let mut gesehen: Vec<*const Func> = Vec::new();
    while let Some(g) = offen.pop() {
        if gesehen.contains(&(g as *const Func)) { continue; }
        gesehen.push(g);
        for ins in &g.code {
            for p in globale_plaetze(prog, g, ins) {
                if bereich.contains(&p) {
                    let gn = prog.global_names.get(p).cloned().unwrap_or_default();
                    return Err(format!("Methode {}: {} beruehrt die Globale '{}' des Bereichs", name, g.name, gn));
                }
            }
            match ins.op {
                op::CALL_USER => rufe(ins, &mut offen),
                op::CALL_METHOD => if let Arg::Call(n, _, _) = &ins.arg { offen.extend(methoden(n)); },
                op::LOAD_MEMBER | op::STORE_MEMBER => if let Some(Value::Str(n)) = g.constants.get(ins.arg.as_usize()) {
                    offen.extend(methoden(&format!("__get_{}", n)));
                    offen.extend(methoden(&format!("__set_{}", n)));
                },
                op::CALL_VALUE | op::CALL_SUPER =>
                    return Err(format!("Methode {}: {} ruft {}", name, g.name, befehl_name(ins.op))),
                op::CALL_BUILTIN => if let Arg::Call(n, _, _) = &ins.arg {
                    let n: &str = n;
                    if n == "sort" || n == "__comp_iter" || n.starts_with("coro_") || n.starts_with("task_")
                        || n == "gui_update" || n == "timer_update" || n == "exit" || n == "end" {
                        return Err(format!("Methode {}: {} ruft {}", name, g.name, n.to_uppercase()));
                    }
                },
                _ => {}
            }
        }
    }
    Ok(())
}

/// `c[i...]` lesen: Behaelter in `basis`, die Indizes dahinter; das Ergebnis
/// nach `basis`.
extern "C" fn w_index(k: *mut Kontext, basis: u64, n: u64) {
    let (c, idx): (&Value, &[Value]) = unsafe {
        (&*(*k).werte.add(basis as usize), std::slice::from_raw_parts((*k).werte.add(basis as usize + 1), n as usize))
    };
    match crate::vm::load_index(c, idx) {
        Ok(v) => {
            for j in 1..=n { drop(w_nehmen(k, basis + j)); }
            *w_platz(k, basis) = v;
        }
        Err(_) => w_fehler(k),
    }
}

/// `c[i...] = v`: Behaelter in `basis`, dann die Indizes, dann der Wert.
extern "C" fn w_setzen(k: *mut Kontext, basis: u64, n: u64) {
    let v = w_nehmen(k, basis + n + 1);
    let (c, idx): (&Value, &[Value]) = unsafe {
        (&*(*k).werte.add(basis as usize), std::slice::from_raw_parts((*k).werte.add(basis as usize + 1), n as usize))
    };
    match crate::vm::store_index(c, idx, v.clone()) {
        Ok(()) => { for j in 0..=n { drop(w_nehmen(k, basis + j)); } }
        Err(_) => { *w_platz(k, basis + n + 1) = v; w_fehler(k); }
    }
}

/// Die Helfer, die der Maschinencode ruft (Kennungen im Modul).
#[derive(Clone, Copy)]
struct Hilfe {
    holen: FuncId,
    lesen_i: FuncId,
    lesen_f: FuncId,
    setzen_i: FuncId,
    setzen_f: FuncId,
    element_i: FuncId,
    element_f: FuncId,
    journal_schluss: FuncId,
    mathe1: FuncId,
    mathe2: FuncId,
    w: [FuncId; 16],
}

/// Plaetze in `Hilfe::w`.
const W_KONST: usize = 0;
const W_KOPIE: usize = 1;
const W_FREI: usize = 2;
const W_ABLEGEN_I: usize = 3;
const W_ABLEGEN_F: usize = 4;
const W_ZAHL_I: usize = 5;
const W_ZAHL_F: usize = 6;
const W_SPEICHERN: usize = 7;
const W_OP: usize = 8;
const W_WAHR: usize = 9;
const W_BUILTIN_I: usize = 10;
const W_BUILTIN_F: usize = 11;
const W_INDEX: usize = 12;
const W_SETZEN: usize = 13;
const W_METHODE: usize = 14;
const W_DRUCKEN: usize = 15;

type Einstieg = unsafe extern "C" fn(*mut Kontext, *const u64, *mut u64);

/// Eine uebersetzte Funktion, wie die VM sie ruft.
pub struct Uebersetzt {
    einstieg: Einstieg,
    params: Vec<Art>,
    rueck: Option<Art>,
    fehlschlaege: std::cell::Cell<u32>,
    /// Globale Plaetze, die diese Funktion samt allen, die sie ruft,
    /// beruehren kann -- nach dem Aufruf werden nur sie zurueckgeschrieben
    /// und ihre Marken geloescht.
    beruehrt: Vec<usize>,
    /// Schreibt sie (oder eine, die sie ruft) globale Plaetze? Dann darf ein
    /// Bereich, der sie ruft, nicht mitten drin aussteigen (siehe `schleife`).
    schreibt_globale: bool,
}

/// Nach so vielen Rueckfaellen in die VM nimmt eine Funktion nur noch die
/// VM -- sonst rechnete ein Aufruf, der immer ueberlaeuft, jedes Mal doppelt.
const MAX_FEHLSCHLAEGE: u32 = 8;

pub struct Jit {
    // Haelt den ausfuehrbaren Speicher; darf erst mit dem Jit gehen. Im
    // RefCell, weil Schleifen erst beim Laufen uebersetzt werden.
    modul: RefCell<JITModule>,
    fctx: RefCell<FunctionBuilderContext>,
    ids: Vec<Option<FuncId>>,
    hilfe: Hilfe,
    glob: Globale,
    /// Uebersetzte Schleifen; `Instr::schleife` an der Ruecksprung-Stelle
    /// zeigt hierher (Index + 2; 1 = nie, 0 = noch nicht versucht).
    schleifen: RefCell<Vec<Schleife>>,
    zahl_schleifen: std::cell::Cell<usize>,
    zahl_laeufe: std::cell::Cell<u64>,
    zahl_aussteige: std::cell::Cell<u64>,
    fns: Vec<Option<Uebersetzt>>,
    /// Art je globalem Platz als Byte fuer `global_holen` (0 = keine Zahl).
    arten: Vec<u8>,
    schatten: std::cell::UnsafeCell<Vec<u64>>,
    marken: std::cell::UnsafeCell<Vec<u8>>,
    /// (Lage der Klasse, Methode) -> Index in `fns` (siehe `Globale::tafel`).
    mtab: HashMap<(usize, usize), usize>,
    /// Die Meldung eines Befehls, an dem ein Bereich zuletzt ausgestiegen ist
    /// (siehe `Kontext::meldung`); die VM holt sie mit `meldung_nehmen`.
    meldung: RefCell<Option<String>>,
}

/// Eine uebersetzte Schleife: Einstieg, die Arten der Locals, fuer die sie
/// gebaut ist, und je Ausgang die Locals, die zurueckgeschrieben werden.
struct Schleife {
    einstieg: unsafe extern "C" fn(*mut Kontext, *mut u64) -> i64,
    kopf: usize,
    start: Vec<Option<Art>>,
    ausgaenge: Vec<(usize, Vec<Option<Art>>)>,
    beruehrt: Vec<usize>,
    fehlschlaege: u32,
    felder: Vec<FeldInfo>,
    aussteige: Vec<Aussteig>,
    klassen: Vec<Klasse>,
    selbst: Option<u16>,
    /// Globale Plaetze, die der Bereich wie Locals fuehrt (Objekte), und wie
    /// viele echte Locals es gibt -- Platz `n_lokal + j` ist `dyn_glob[j]`.
    dyn_glob: Vec<usize>,
    n_lokal: usize,
    /// Wertemodus; und die Typen der Locals (die Texte, auf die der
    /// Maschinencode fuer `w_speichern` zeigt -- sie muessen so lange leben
    /// wie er).
    modus_w: bool,
    #[allow(dead_code)]
    typen: Vec<String>,
}

/// Was der Maschinencode ueber die globalen Plaetze wissen muss: ihre Art
/// (aus den DECLARE-Befehlen des Hauptprogramms; widersprechen sich zwei,
/// bleibt der Platz der VM) und ob sie Konstanten sind.
struct Globale {
    art: Vec<Option<Art>>,
    konst: Vec<bool>,
    /// Deklarierter Typ je Platz (fuer Objekte in `dyn_glob`).
    typ: Vec<String>,
    /// Die Methoden, die als Funktion uebersetzt werden koennen: je Klasse
    /// JEDE Methode, die sie sieht (auch geerbte), mit der Lage DIESER Klasse.
    /// Eine geerbte Methode bekommt also je Unterklasse eine eigene Fassung --
    /// sonst loeste `Self.hilfe()` darin statisch die Fassung der Vorfahrin
    /// auf, die VM aber die Ueberschreibung der Unterklasse. Index im Jit:
    /// `Program::functions.len() + j`.
    tafel: Vec<MEintrag>,
}

/// Ein Eintrag der Methodentafel (siehe `Globale::tafel`).
#[derive(Clone)]
struct MEintrag {
    lage: *const crate::value::Layout,
    klasse: String,
    name: String,
    func: *const Func,
}

/// Die Methode `name`, wie `Vm::resolve_method` sie fuer `klasse` findet:
/// erst die Klasse selbst, dann die Vorfahrinnen.
fn methode_suchen<'a>(prog: &'a Program, klasse: &str, name: &str) -> Option<&'a Func> {
    let mut c = prog.classes.get(klasse)?;
    loop {
        if let Some(f) = c.methods.get(name) { return Some(f); }
        if let Some((_, f)) = c.methods.iter().find(|(m, _)| m.eq_ignore_ascii_case(name)) { return Some(f); }
        if c.parent_name.is_empty() { return None; }
        c = prog.classes.get(c.parent_name.as_str())?;
    }
}

fn methoden_tafel(prog: &Program) -> Vec<MEintrag> {
    let mut klassen: Vec<&crate::model::ClassInfo> = prog.classes.values().filter(|c| !c.is_struct).collect();
    klassen.sort_by(|a, b| a.name.cmp(&b.name));
    let mut tafel = Vec::new();
    for c in klassen {
        let mut namen: Vec<String> = Vec::new();
        let mut k = Some(c);
        while let Some(ci) = k {
            for m in ci.methods.keys() { if !namen.iter().any(|n| n.eq_ignore_ascii_case(m)) { namen.push(m.clone()); } }
            k = if ci.parent_name.is_empty() { None } else { prog.classes.get(ci.parent_name.as_str()) };
        }
        namen.sort();
        for n in namen {
            if let Some(f) = methode_suchen(prog, &c.name, &n) {
                tafel.push(MEintrag { lage: Rc::as_ptr(&c.layout), klasse: c.name.clone(), name: n, func: f as *const Func });
            }
        }
    }
    tafel
}

/// Alle Funktionen des Jit: erst die freien, dann die Methodentafel, je mit
/// der Klasse (fuer `Self`).
fn alle_funktionen<'a>(prog: &'a Program, glob: &Globale) -> Vec<(&'a Func, Option<&'a crate::model::ClassInfo>, String)> {
    let mut v: Vec<(&Func, Option<&crate::model::ClassInfo>, String)> =
        prog.functions.iter().map(|f| (f, None, f.name.clone())).collect();
    for e in &glob.tafel {
        let f: &'a Func = unsafe { &*e.func };
        v.push((f, prog.classes.get(e.klasse.as_str()), format!("{}.{}", e.klasse, e.name)));
    }
    v
}

fn globale_lesen(prog: &Program) -> Globale {
    let n = prog.n_globals;
    let mut art: Vec<Option<Option<Art>>> = vec![None; n];
    let mut konst = vec![false; n];
    let mut typ = vec![String::new(); n];
    let f = &prog.main;
    for ins in &f.code {
        let (l, ist_konst) = match (ins.op, &ins.arg) {
            (op::DECLARE_GLOBAL_SLOT, Arg::List(l)) => (l, false),
            (op::DECLARE_GLOBAL_CONST_SLOT, Arg::List(l)) => (l, true),
            _ => continue,
        };
        let i = l[0].as_usize();
        if i >= n { continue; }
        let ty = match f.constants.get(l[2].as_usize()) { Some(Value::Str(t)) => t.to_string(), _ => String::new() };
        let a = art_von_typ(&ty);
        art[i] = Some(match art[i] { None => a, Some(alt) if alt == a => a, Some(_) => None });
        typ[i] = ty;
        if ist_konst { konst[i] = true; }
    }
    Globale { art: art.into_iter().map(|a| a.flatten()).collect(), konst, typ, tafel: methoden_tafel(prog) }
}

// ---------------------------------------------------------------------------
// Stufe 1: pruefen und Arten verfolgen
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Debug)]
struct Zustand {
    stapel: Vec<Art>,
    lokal: Vec<Option<Art>>,
}

/// Eine Schleife als eigener Uebersetzungsbereich: vom Kopf bis zum
/// Ruecksprung, mit den Arten, die die Locals beim Eintritt HABEN (die VM
/// kennt sie in dem Moment).
struct Bereich {
    von: usize,
    bis: usize,
    start: Zustand,
    /// Welche globalen Plaetze es beim Uebersetzen schon gibt. Ein Platz
    /// verschwindet nie wieder -- ein DECLARE fuer einen vorhandenen ist
    /// darum fuer immer ein Nichtstun (die VM legt nur leere Plaetze an).
    globale_da: Vec<bool>,
    /// Die Felder in Locals (schon als `Art::Feld` im Startzustand) und je
    /// globalem Platz, ob er gerade ein Feld von Zahlen haelt.
    felder: Vec<FeldInfo>,
    glob_felder: Vec<Option<(Elem, u8, bool)>>,
    /// Objekte: die Klassen, `Self` (wenn der Bereich es benutzt), die
    /// globalen Plaetze, die er wie Locals fuehrt, und die Typen aller
    /// Locals samt dieser.
    klassen: Vec<Klasse>,
    selbst: Option<u16>,
    dyn_glob: Vec<usize>,
    lok_typen: Vec<String>,
    /// Wertemodus: alles, was keine Zahl ist, ist `W` (keine Felder und
    /// Objekte mit Zeigern -- ein eingebauter Befehl koennte ein Feld wachsen
    /// lassen oder ein Objekt freigeben, auf das ein Zeiger zeigt).
    modus_w: bool,
}

struct Analyse {
    /// Zustand vor jedem erreichbaren Befehl -- bei einem Bereich auch an den
    /// Ausgaengen (Stellen ausserhalb, zu denen er springt).
    vor: Vec<Option<Zustand>>,
    /// Nur Bereich: die Stellen ausserhalb, an denen die VM weitermacht.
    ausgaenge: Vec<usize>,
    /// Art je Parameter und die Rueckgabe (None = SUB).
    params: Vec<Art>,
    rueck: Option<Art>,
    /// Funktionen, die gerufen werden (Index in `Program::functions`).
    gerufen: Vec<usize>,
    /// Globale Plaetze, die sie selbst liest oder schreibt, und die sie
    /// schreibt.
    beruehrt: Vec<usize>,
    geschrieben: Vec<usize>,
    /// Nur Bereich: die Felder, die er benutzt, und ob er in eins schreibt.
    felder: Vec<FeldInfo>,
    schreibt_felder: bool,
    klassen: Vec<Klasse>,
    selbst: Option<u16>,
    dyn_glob: Vec<usize>,
    /// Typ je Local (bei einem Bereich samt `dyn_glob`).
    typen: Vec<String>,
    modus_w: bool,
}

fn ziel(arg: &Arg) -> usize { arg.as_i64().max(0) as usize }

/// Ein globaler Platz, den ein Bereich wie ein Local fuehrt (`dyn_glob`),
/// wird in Analyse und Erzeugung als LOAD/STORE_LOCAL auf Platz
/// `n_echt + j` behandelt. Liefert (Befehl, Platz).
fn umleiten(ins: &crate::model::Instr, dyn_glob: &[usize], n_echt: usize) -> (u16, usize) {
    let neu = match ins.op {
        op::LOAD_GLOBAL_SLOT => op::LOAD_LOCAL,
        op::STORE_GLOBAL_SLOT => op::STORE_LOCAL,
        op::ADD_STORE_GLOBAL_SLOT => op::ADD_STORE_LOCAL,
        _ => return (ins.op, match ins.op { op::LOAD_LOCAL | op::STORE_LOCAL | op::ADD_STORE_LOCAL => ins.arg.as_usize(), _ => 0 }),
    };
    match dyn_glob.iter().position(|&g| g == ins.arg.as_usize()) {
        Some(j) => (neu, n_echt + j),
        None => (ins.op, 0),
    }
}

fn for_teile(arg: &Arg) -> Option<[i64; 7]> {
    let v: Vec<i64> = match arg {
        Arg::Ints(v) => v.to_vec(),
        Arg::List(l) => l.iter().map(|a| a.as_i64()).collect(),
        _ => return None,
    };
    if v.len() < 7 { return None; }
    Some([v[0], v[1], v[2], v[3], v[4], v[5], v[6]])
}

/// Kann ein Wert der Art `von` in einen Platz der Art `nach` (Parameter,
/// Local, Rueckgabe)? INTEGER <-> FLOAT geht (FLOAT -> INTEGER mit Pruefung
/// zur Laufzeit), BOOLEAN nur nach BOOLEAN -- alles andere ist in der VM ein
/// sicherer Fehler, und dann bleibt die Funktion dort.
fn passt(von: Art, nach: Art) -> bool {
    von == nach || (zahl(von) && zahl(nach))
}

fn analysieren(prog: &Program, glob: &Globale, f: &Func, bereich: Option<&Bereich>,
               klasse: Option<&crate::model::ClassInfo>) -> Result<Analyse, String> {
    let typen: Vec<String> = bereich.map_or_else(|| f.local_types.clone(), |b| b.lok_typen.clone());
    let n_lok = typen.len();
    let n_echt = f.local_types.len();
    // Deklarierte Art je Local (None = `any`, die Art folgt dem Wert); ein
    // Local anderen Typs (Text, Feld ...) ist "fremd": eine Funktion mit ihm
    // bleibt ganz in der VM, ein Bereich darf ihn nur nicht anfassen.
    let mut dekl: Vec<Option<Art>> = Vec::with_capacity(n_lok);
    let mut fremd = vec![false; n_lok];
    for (i, t) in typen.iter().enumerate() {
        match (art_von_typ(t), t.as_str()) {
            (Some(a), _) => dekl.push(Some(a)),
            (None, "any" | "") if i >= f.n_params || bereich.is_some() => dekl.push(None),
            _ if bereich.is_some() => { dekl.push(None); fremd[i] = true; }
            _ => return Err(format!("Platz {} hat den Typ '{}'", i, t)),
        }
    }
    let (params, rueck, start) = match bereich {
        Some(b) => (Vec::new(), None, b.start.clone()),
        None => {
            if f.is_coroutine { return Err("Coroutine".into()); }
            if f.is_variadic { return Err("variadisch".into()); }
            if f.param_byref.iter().any(|&b| b) { return Err("BYREF".into()); }
            if f.param_default_is_expr.iter().any(|&b| b) { return Err("Vorgabe als Ausdruck".into()); }
            if f.n_params > 16 { return Err("mehr als 16 Parameter".into()); }
            let params: Vec<Art> = (0..f.n_params).map(|i| dekl[i].unwrap()).collect();
            let rueck = if f.is_sub { None } else {
                Some(art_von_typ(&f.return_type).ok_or_else(|| format!("Rueckgabetyp '{}'", f.return_type))?)
            };
            let mut start = Zustand { stapel: Vec::new(), lokal: vec![None; n_lok] };
            for i in 0..n_lok {
                if i < f.n_params { start.lokal[i] = Some(params[i]); continue; }
                let d = f.local_defaults.get(i).cloned().unwrap_or(Value::Nil);
                match (&d, art_von_wert(&d)) {
                    (Value::Nil, _) => {}
                    (_, Some(a)) if dekl[i].map_or(true, |x| x == a) => start.lokal[i] = Some(a),
                    _ => return Err(format!("Vorgabe von Platz {}", i)),
                }
            }
            (params, rueck, start)
        }
    };
    let (von, bis) = match bereich { Some(b) => (b.von, b.bis), None => (0, usize::MAX) };

    let code = &f.code;
    let mut vor: Vec<Option<Zustand>> = vec![None; code.len() + 1];
    let mut gerufen = Vec::new();
    let mut beruehrt: Vec<usize> = Vec::new();
    let mut geschrieben: Vec<usize> = Vec::new();
    let mut ausgaenge: Vec<usize> = Vec::new();
    let mut felder: Vec<FeldInfo> = bereich.map_or(Vec::new(), |b| b.felder.clone());
    let mut schreibt_felder = false;
    let mut klassen: Vec<Klasse> = bereich.map_or(Vec::new(), |b| b.klassen.clone());
    let mut selbst = bereich.and_then(|b| b.selbst);
    // Eine Methode, als Funktion uebersetzt: `Self` ist ein Objekt genau
    // dieser Klasse (der Einstieg prueft die Lage).
    if let (None, Some(ci)) = (bereich, klasse) {
        klassen.push(Klasse { lage: Rc::as_ptr(&ci.layout), name: ci.name.clone() });
        selbst = Some(0);
    }
    let dyn_glob: Vec<usize> = bereich.map_or(Vec::new(), |b| b.dyn_glob.clone());
    let modus_w = bereich.map_or(false, |b| b.modus_w);
    // Im Wertemodus: ist einer der Operanden ein Wert, rechnet ein Helfer.
    let w_oder_skalar = |a: Art| skalar(a) || a == Art::W;
    let mut offen = vec![von];
    vor[von] = Some(start);

    // Einen Nachfolger mit einem Zustand versorgen; widersprechen sich zwei
    // Wege, bleibt die Funktion in der VM. Bei einem Bereich ist eine Stelle
    // ausserhalb ein Ausgang: dort muss der Stapel leer sein, und es wird
    // nicht weitergerechnet.
    let mut melden = |vor: &mut [Option<Zustand>], offen: &mut Vec<usize>, i: usize, z: &Zustand| -> Result<(), String> {
        if i >= vor.len() { return Err("Sprung ins Leere".into()); }
        let draussen = i < von || i > bis;
        if draussen && !z.stapel.is_empty() { return Err(format!("Ausgang {} mit Werten auf dem Stapel", i)); }
        match &vor[i] {
            None => {
                vor[i] = Some(z.clone());
                if draussen { ausgaenge.push(i); } else { offen.push(i); }
                Ok(())
            }
            Some(alt) if alt == z => Ok(()),
            Some(_) => Err(format!("zwei Wege treffen sich an Stelle {} mit verschiedenen Arten", i)),
        }
    };

    while let Some(ip) = offen.pop() {
        let mut z = vor[ip].clone().unwrap();
        if ip >= code.len() { continue; }  // Ende = RETURN Nil (siehe dispatch)
        let ins = &code[ip];
        macro_rules! pop { () => { z.stapel.pop().ok_or_else(|| format!("leerer Stapel an Stelle {}", ip))? } }
        let mut weiter = true;
        let (o, s_arg) = umleiten(ins, &dyn_glob, n_echt);
        match o {
            op::LOAD_CONST => {
                let c = f.constants.get(ins.arg.as_usize()).ok_or("Konstante fehlt")?;
                // Das Ende einer FUNCTION ohne RETURN: der Compiler legt einen
                // Text vor HALT -- das ist ein Ausstieg, die VM liefert NIL.
                let vor_halt = code.get(ip + 1).map_or(false, |n| n.op == op::HALT);
                match art_von_wert(c) {
                    Some(a) => z.stapel.push(a),
                    None if vor_halt => z.stapel.push(Art::N),
                    None if modus_w => z.stapel.push(Art::W),
                    None => return Err(format!("Konstante {} ist keine Zahl", c.type_name())),
                }
            }
            op::LOAD_LOCAL if matches!(z.lokal.get(s_arg).copied().flatten(), Some(Art::Feld(_) | Art::Obj(_) | Art::W)) => {
                z.stapel.push(z.lokal[s_arg].unwrap());
            }
            op::LOAD_SELF if selbst.is_some() => { z.stapel.push(Art::Obj(selbst.unwrap())); }
            op::LOAD_NAME if vorbelegt(prog, f, ins).is_some() => {
                z.stapel.push(if matches!(vorbelegt(prog, f, ins), Some(Value::Float(_))) { Art::F } else { Art::I });
            }
            op::LOAD_FIELD | op::LOAD_MEMBER | op::STORE_FIELD | op::STORE_MEMBER if bereich.is_some() || selbst.is_some() => {
                let name = match f.constants.get(ins.arg.as_usize()) { Some(Value::Str(n)) => n.to_string(), _ => return Err("Feldname".into()) };
                let wert = if matches!(o, op::STORE_FIELD | op::STORE_MEMBER) { Some(pop!()) } else { None };
                let k = if matches!(o, op::LOAD_FIELD | op::STORE_FIELD) {
                    selbst.ok_or("Self ohne Objekt")? as usize
                } else {
                    match pop!() { Art::Obj(k) => k as usize, _ => return Err("Mitglied von etwas, das kein Objekt ist".into()) }
                };
                let (_, a) = feld_von(prog, &mut klassen, k, &name, matches!(o, op::LOAD_MEMBER | op::STORE_MEMBER))?;
                match wert {
                    None => z.stapel.push(a),
                    Some(w) => {
                        if !skalar(a) || !passt(w, a) { return Err(format!("{:?} in ein Feld {:?}", w, a)); }
                        schreibt_felder = true;
                    }
                }
            }
            op::LOAD_GLOBAL_SLOT if glob.art.get(ins.arg.as_usize()).copied().flatten().is_none()
                && bereich.map_or(false, |b| b.glob_felder.get(ins.arg.as_usize()).copied().flatten().is_some()) => {
                let g = ins.arg.as_usize();
                let (elem, dims, tupel) = bereich.unwrap().glob_felder[g].unwrap();
                let nr = match felder.iter().position(|x| x.quelle == Quelle::Global(g)) {
                    Some(nr) => nr,
                    None => { felder.push(FeldInfo { quelle: Quelle::Global(g), elem, dims, tupel }); felder.len() - 1 }
                };
                z.stapel.push(Art::Feld(nr as u16));
            }
            op::LOAD_INDEX | op::STORE_INDEX if modus_w => {
                let n = ins.arg.as_usize();
                if o == op::STORE_INDEX { if !w_oder_skalar(pop!()) { return Err("Feld bekommt NIL".into()); } }
                for _ in 0..n { if !w_oder_skalar(pop!()) { return Err("Index, der kein Wert ist".into()); } }
                if pop!() != Art::W { return Err("Index auf etwas, das kein Wert ist".into()); }
                if o == op::LOAD_INDEX { z.stapel.push(Art::W); } else { schreibt_felder = true; }
            }
            op::CALL_METHOD if !modus_w => {
                let (name, argc) = match &ins.arg { Arg::Call(n, c, _) => (n.clone(), *c as usize), _ => return Err("Methode ohne Namen".into()) };
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc { args.push(pop!()); }
                args.reverse();
                let k = match pop!() { Art::Obj(k) => k as usize, _ => return Err(format!("Methode {} auf etwas, das kein Objekt ist", name)) };
                let lage = klassen[k].lage;
                let j = glob.tafel.iter().position(|e| std::ptr::eq(e.lage, lage) && e.name.eq_ignore_ascii_case(&name))
                    .ok_or_else(|| format!("Methode {} (nicht in der Tafel)", name))?;
                let g: &Func = unsafe { &*glob.tafel[j].func };
                if g.is_coroutine { return Err(format!("Methode {} ist eine Coroutine", name)); }
                if argc != g.n_params { return Err(format!("Methode {} mit {} statt {} Argumenten", name, argc, g.n_params)); }
                for (i, a) in args.iter().enumerate() {
                    let p = art_von_typ(&g.local_types[i]).ok_or_else(|| format!("{} hat einen Parameter vom Typ '{}'", name, g.local_types[i]))?;
                    if !passt(*a, p) { return Err(format!("Argument {} an {} passt nicht", i + 1, name)); }
                }
                gerufen.push(prog.functions.len() + j);
                z.stapel.push(if g.is_sub { Art::N } else { art_von_typ(&g.return_type).ok_or_else(|| format!("{} liefert '{}'", name, g.return_type))? });
                // Die Methode kann Felder schreiben -- auch die, die dieser Code liest.
                schreibt_felder = true;
            }
            op::CALL_METHOD if modus_w => {
                let (name, argc) = match &ins.arg { Arg::Call(n, c, _) => (n.clone(), *c as usize), _ => return Err("Methode ohne Namen".into()) };
                let b = bereich.ok_or("Methode ausserhalb eines Bereichs")?;
                methoden_harmlos(prog, f, b.von, b.bis, &name)?;
                for _ in 0..argc { if !w_oder_skalar(pop!()) { return Err("Argument NIL".into()); } }
                if pop!() != Art::W { return Err("Methode auf etwas, das kein Wert ist".into()); }
                z.stapel.push(Art::W);
                schreibt_felder = true;
            }
            op::CALL_BUILTIN if match &ins.arg {
                Arg::Call(n, c, _) => z.stapel.len() >= *c as usize
                    && zahl_befehl(n, &z.stapel[z.stapel.len() - *c as usize..]).is_some(),
                _ => false,
            } => {
                let (name, argc) = match &ins.arg { Arg::Call(n, c, _) => (n.clone(), *c as usize), _ => unreachable!() };
                let a = zahl_befehl(&name, &z.stapel[z.stapel.len() - argc..]).unwrap();
                z.stapel.truncate(z.stapel.len() - argc);
                z.stapel.push(a);
            }
            op::PRINT if modus_w => {
                let n = ins.arg.list()[0].as_usize();
                for _ in 0..n { if !w_oder_skalar(pop!()) { return Err("PRINT eines Feldes".into()); } }
                // Eine Nebenwirkung wie eine Feldschreibung: ein Bereich, der
                // aufgeben und die VM von vorn rechnen lassen muesste, druckte
                // sonst doppelt.
                schreibt_felder = true;
            }
            op::CALL_BUILTIN if modus_w => {
                let (name, argc) = match &ins.arg { Arg::Call(n, c, _) => (n.clone(), *c as usize), _ => return Err("Befehl ohne Namen".into()) };
                // Nur die REINEN Befehle, und nur, wenn die VM an genau dieser
                // Stelle schon sie gefragt hat (Merkplatz der Familie).
                if ins.familie.get() == 0 {
                    return Err(format!("eingebauter Befehl {} (an dieser Stelle noch nie gerufen)", name.to_uppercase()));
                }
                if !befehl_im_bereich(ins.familie.get(), &name) {
                    return Err(format!("eingebauter Befehl {} (ruft Drachenhauch-Code oder braucht die Zeile)", name.to_uppercase()));
                }
                for _ in 0..argc { if !w_oder_skalar(pop!()) { return Err("Argument NIL".into()); } }
                z.stapel.push(match crate::typen::builtin_typ(&name) {
                    Some(crate::typen::Typ::Int) => Art::I,
                    Some(crate::typen::Typ::Float) => Art::F,
                    _ => Art::W,
                });
                schreibt_felder = true;
            }
            op::LOAD_INDEX | op::STORE_INDEX if bereich.is_some() => {
                let n = ins.arg.as_usize();
                let wert = if ins.op == op::STORE_INDEX { Some(pop!()) } else { None };
                for _ in 0..n { if pop!() != Art::I { return Err("Feld-Index, der kein INTEGER ist".into()); } }
                let nr = match pop!() { Art::Feld(nr) => nr as usize, _ => return Err("Index auf etwas, das kein Feld von Zahlen ist".into()) };
                if felder[nr].dims as usize != n { return Err("Feld mit anderer Zahl von Indizes".into()); }
                let elem = felder[nr].art();
                match wert {
                    Some(w) => {
                        if !zahl(w) { return Err("Feld bekommt einen Nicht-Zahl-Wert".into()); }
                        if matches!(felder[nr].elem, Elem::Wert(_)) { return Err("Schreiben in ein Feld von Werten".into()); }
                        schreibt_felder = true;
                    }
                    None => z.stapel.push(elem),
                }
            }
            op::LOAD_LOCAL => {
                let s = s_arg;
                if fremd.get(s).copied().unwrap_or(false) { return Err(format!("Platz {} hat den Typ '{}'", s, typen[s])); }
                z.stapel.push(z.lokal.get(s).copied().flatten().ok_or_else(|| format!("Platz {} gelesen, bevor er belegt ist", s))?);
            }
            op::LOAD_GLOBAL_SLOT => {
                let g = ins.arg.as_usize();
                let a = glob.art.get(g).copied().flatten().ok_or("globale Variable, die keine Zahl ist")?;
                z.stapel.push(a);
                if !beruehrt.contains(&g) { beruehrt.push(g); }
            }
            op::STORE_GLOBAL_SLOT | op::ADD_STORE_GLOBAL_SLOT => {
                let g = ins.arg.as_usize();
                let d = glob.art.get(g).copied().flatten().ok_or("globale Variable, die keine Zahl ist")?;
                if glob.konst[g] { return Err("Zuweisung an eine Konstante".into()); }
                let mut a = pop!();
                if ins.op == op::ADD_STORE_GLOBAL_SLOT {
                    let x = pop!();
                    if modus_w && (a == Art::W || x == Art::W) && w_oder_skalar(a) && w_oder_skalar(x) {
                        a = Art::W;
                    } else {
                        if !zahl(a) || !zahl(x) { return Err("+ mit einem Nicht-Zahl-Wert".into()); }
                        a = if a == Art::I && x == Art::I { Art::I } else { Art::F };
                    }
                }
                if a != Art::W && !passt(a, d) { return Err(format!("{:?} in eine globale {:?}", a, d)); }
                if !beruehrt.contains(&g) { beruehrt.push(g); }
                if !geschrieben.contains(&g) { geschrieben.push(g); }
            }
            op::STORE_LOCAL | op::ADD_STORE_LOCAL => {
                let s = s_arg;
                let mut a = pop!();
                if o == op::ADD_STORE_LOCAL {
                    let x = pop!();
                    if modus_w && (a == Art::W || x == Art::W) && w_oder_skalar(a) && w_oder_skalar(x) {
                        a = Art::W;
                    } else {
                        if !zahl(a) || !zahl(x) { return Err("+ mit einem Nicht-Zahl-Wert".into()); }
                        a = if a == Art::I && x == Art::I { Art::I } else { Art::F };
                    }
                }
                if a == Art::N { return Err("NIL gespeichert".into()); }
                if a == Art::W {
                    // In einen Platz mit Zahlentyp: dort als Zahl (geprueft);
                    // sonst bleibt es ein Wert.
                    z.lokal[s] = Some(dekl[s].unwrap_or(Art::W));
                    melden(&mut vor, &mut offen, ip + 1, &z)?;
                    continue;
                }
                if let Art::Feld(nr) = a {
                    let t = typen[s].as_str();
                    let passend = match felder[nr as usize].elem { Elem::F => "array:float", Elem::I => "array:integer", Elem::Wert(_) => "" };
                    if !(matches!(t, "any" | "") || t == passend) { return Err(format!("Feld in Platz vom Typ '{}'", t)); }
                    z.lokal[s] = Some(a);
                    melden(&mut vor, &mut offen, ip + 1, &z)?;
                    continue;
                }
                if let Art::Obj(k) = a {
                    // In einen Platz ohne Typ oder vom Typ genau dieser Klasse.
                    let t = typen[s].to_lowercase();
                    if !(matches!(t.as_str(), "any" | "") || t == klassen[k as usize].name.to_lowercase()) {
                        return Err(format!("Objekt {} in Platz vom Typ '{}'", klassen[k as usize].name, typen[s]));
                    }
                    z.lokal[s] = Some(a);
                    melden(&mut vor, &mut offen, ip + 1, &z)?;
                    continue;
                }
                if fremd[s] { return Err(format!("Platz {} hat den Typ '{}'", s, typen[s])); }
                let nach = match dekl[s] { Some(d) => { if !passt(a, d) { return Err(format!("{:?} in Platz {:?}", a, d)); } d } None => a };
                z.lokal[s] = Some(nach);
            }
            op::DECLARE_LOCAL => {
                let l = match &ins.arg { Arg::List(l) => l, _ => return Err("DECLARE_LOCAL".into()) };
                let s = l[0].as_usize();
                let ty = l[1].str();
                if ty.starts_with("array:") || ty.starts_with("map:") { return Err("Feld oder MAP".into()); }
                if z.lokal[s].is_none() {
                    let d = match &l[2] { Arg::Int(i) => Value::Int(*i), Arg::Val(v) => v.clone(), Arg::None => Value::Nil, _ => return Err("DECLARE-Vorgabe".into()) };
                    if !matches!(d, Value::Nil) {
                        let a = art_von_wert(&d).ok_or("DECLARE-Vorgabe ist keine Zahl")?;
                        if dekl[s].map_or(false, |x| x != a) { return Err("DECLARE-Vorgabe passt nicht".into()); }
                        z.lokal[s] = Some(a);
                    }
                }
            }
            op::DECLARE_GLOBAL_SLOT if bereich.is_some() => {
                let g = match &ins.arg { Arg::List(l) => l[0].as_usize(), _ => return Err("DECLARE_GLOBAL_SLOT".into()) };
                if !bereich.unwrap().globale_da.get(g).copied().unwrap_or(false) {
                    return Err("legt eine globale Variable an".into());
                }
            }
            op::POP => { pop!(); }
            op::DUP => { let a = *z.stapel.last().ok_or("DUP auf leerem Stapel")?; z.stapel.push(a); }
            op::ADD | op::SUB | op::MUL | op::MOD | op::DIV | op::INT_DIV
                if modus_w && z.stapel.len() >= 2 && z.stapel[z.stapel.len() - 2..].iter().any(|&a| a == Art::W) => {
                let b = pop!(); let a = pop!();
                if !w_oder_skalar(a) || !w_oder_skalar(b) { return Err("Rechnen mit NIL".into()); }
                z.stapel.push(Art::W);
            }
            op::LT | op::GT | op::LEQ | op::GEQ | op::EQ | op::NEQ
                if modus_w && z.stapel.len() >= 2 && z.stapel[z.stapel.len() - 2..].iter().any(|&a| a == Art::W) => {
                let b = pop!(); let a = pop!();
                if !w_oder_skalar(a) || !w_oder_skalar(b) { return Err("Vergleich mit NIL".into()); }
                z.stapel.push(Art::B);
            }
            op::ADD | op::SUB | op::MUL | op::MOD => {
                let b = pop!(); let a = pop!();
                if !zahl(a) || !zahl(b) { return Err("Rechnen mit einem Nicht-Zahl-Wert".into()); }
                z.stapel.push(if a == Art::I && b == Art::I { Art::I } else { Art::F });
            }
            op::DIV => {
                let b = pop!(); let a = pop!();
                if !zahl(a) || !zahl(b) { return Err("/ mit einem Nicht-Zahl-Wert".into()); }
                z.stapel.push(Art::F);
            }
            op::INT_DIV => {
                let b = pop!(); let a = pop!();
                if a != Art::I || b != Art::I { return Err("\\ ohne zwei INTEGER".into()); }
                z.stapel.push(Art::I);
            }
            op::NEG => {
                let a = pop!();
                if !zahl(a) { return Err("Vorzeichen eines Nicht-Zahl-Werts".into()); }
                z.stapel.push(a);
            }
            op::LT | op::GT | op::LEQ | op::GEQ => {
                let b = pop!(); let a = pop!();
                if !zahl(a) || !zahl(b) { return Err("Vergleich mit einem Nicht-Zahl-Wert".into()); }
                z.stapel.push(Art::B);
            }
            op::EQ | op::NEQ => {
                let b = pop!(); let a = pop!();
                if !((zahl(a) && zahl(b)) || (a == Art::B && b == Art::B)) { return Err("= zwischen verschiedenen Arten".into()); }
                z.stapel.push(Art::B);
            }
            // Im Wertemodus auch auf einen Wert: `WHILE NOT QUITREQUESTED()` --
            // der Befehl liefert dort einen Wert, und ohne das blieb fast jede
            // Spielschleife der Beispiele in der VM (M4 Schritt 9).
            op::NOT => { let a = pop!(); if !(skalar(a) || (modus_w && a == Art::W)) { return Err("NOT auf NIL oder ein Feld".into()); } z.stapel.push(Art::B); }
            op::JUMP => { melden(&mut vor, &mut offen, ziel(&ins.arg), &z)?; weiter = false; }
            op::JUMP_IF_FALSE | op::JUMP_IF_TRUE => {
                let a = pop!();
                if !skalar(a) && !(modus_w && a == Art::W) { return Err("Sprung auf NIL oder ein Feld".into()); }
                melden(&mut vor, &mut offen, ziel(&ins.arg), &z)?;
            }
            op::CALL_USER => {
                let (argc, idx) = match &ins.arg { Arg::Call(_, c, i) => (*c as usize, *i), _ => return Err("Aufruf ohne Index".into()) };
                if idx < 0 { return Err("Aufruf ohne Index".into()); }
                let g = prog.functions.get(idx as usize).ok_or("Aufruf ins Leere")?;
                if argc != g.n_params { return Err(format!("Aufruf von {} mit {} statt {} Argumenten", g.name, argc, g.n_params)); }
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc { args.push(pop!()); }
                args.reverse();
                for (i, a) in args.iter().enumerate() {
                    let p = art_von_typ(&g.local_types[i]).ok_or_else(|| format!("{} hat einen Parameter vom Typ '{}'", g.name, g.local_types[i]))?;
                    if !passt(*a, p) { return Err(format!("Argument {} an {} passt nicht", i + 1, g.name)); }
                }
                gerufen.push(idx as usize);
                z.stapel.push(if g.is_sub { Art::N } else { art_von_typ(&g.return_type).ok_or_else(|| format!("{} liefert '{}'", g.name, g.return_type))? });
            }
            op::RETURN | op::RETURN_VOID | op::HALT if bereich.is_some() => {
                return Err("RETURN in der Schleife".into());
            }
            op::RETURN => {
                let a = pop!();
                match rueck {
                    Some(r) if passt(a, r) => {}
                    _ => return Err("RETURN passt nicht zum Rueckgabetyp".into()),
                }
                weiter = false;
            }
            op::RETURN_VOID | op::HALT => { weiter = false; }
            op::FOR_NEXT => {
                let t = for_teile(&ins.arg).ok_or("FOR_NEXT")?;
                let lok = |s: i64| z.lokal.get(s as usize).copied().flatten();
                let lauf = if t[0] == 1 {
                    let g = t[1] as usize;
                    if glob.konst.get(g).copied().unwrap_or(true) { return Err("FOR ueber eine Konstante".into()); }
                    if !beruehrt.contains(&g) { beruehrt.push(g); }
                    if !geschrieben.contains(&g) { geschrieben.push(g); }
                    glob.art.get(g).copied().flatten()
                } else { lok(t[1]) };
                let schritt = if t[3] == 1 { lok(t[4]) } else { f.constants.get(t[4] as usize).and_then(art_von_wert) };
                if lauf != Some(Art::I) || lok(t[2]) != Some(Art::I) || schritt != Some(Art::I) {
                    return Err("FOR nicht ueber INTEGER".into());
                }
                melden(&mut vor, &mut offen, t[6] as usize, &z)?;
            }
            andere => return Err(befehl_name(andere)),
        }
        if weiter { melden(&mut vor, &mut offen, ip + 1, &z)?; }
    }
    Ok(Analyse { vor, ausgaenge, params, rueck, gerufen, beruehrt, geschrieben, felder, schreibt_felder,
                 klassen, selbst, dyn_glob, typen, modus_w })
}

/// Fuer den Grund in `dhrt --jit`: was die Funktion tut, das der
/// Maschinencode (noch) nicht kann.
fn befehl_name(o: u16) -> String {
    match o {
        op::PRINT => "Ausgabe (PRINT)".into(),
        op::INPUT_NAME | op::INPUT_LOCAL => "Eingabe (INPUT)".into(),
        op::LOAD_GLOBAL_SLOT | op::STORE_GLOBAL_SLOT | op::ADD_STORE_GLOBAL_SLOT
        | op::LOAD_NAME | op::STORE_NAME => "globale Variable".into(),
        op::CALL_BUILTIN => "eingebauter Befehl".into(),
        op::CALL_METHOD | op::LOAD_MEMBER | op::STORE_MEMBER | op::LOAD_FIELD
        | op::STORE_FIELD | op::NEW_INSTANCE | op::LOAD_SELF => "Objekt".into(),
        op::LOAD_INDEX | op::STORE_INDEX | op::DECLARE_ARRAY_LOCAL => "Feld".into(),
        op::CALL_VALUE | op::LOAD_FUNCREF => "FUNCREF".into(),
        op::TRY_BEGIN | op::TRY_END | op::THROW => "TRY/THROW".into(),
        op::POW => "^".into(),
        op::BAND | op::BOR | op::BXOR | op::SHL | op::SHR | op::BNOT => "Bit-Operator".into(),
        andere => format!("Befehl {}", andere),
    }
}

/// Welche Funktionen uebersetzt werden, und fuer die anderen: warum nicht.
/// Eine Funktion, die eine nicht uebersetzbare ruft, bleibt ebenfalls in der
/// VM (sie wird ganz uebersetzt oder gar nicht).
fn auswahl(prog: &Program, glob: &Globale) -> (Vec<Option<Analyse>>, Vec<String>) {
    let mut an: Vec<Option<Analyse>> = Vec::new();
    let mut gruende: Vec<String> = Vec::new();
    let alle = alle_funktionen(prog, glob);
    for (f, kl, _) in &alle {
        match analysieren(prog, glob, f, None, *kl) {
            Ok(a) => { an.push(Some(a)); gruende.push(String::new()); }
            Err(e) => { an.push(None); gruende.push(e); }
        }
    }
    loop {
        let mut raus = None;
        for (i, a) in an.iter().enumerate() {
            if let Some(a) = a {
                if let Some(&g) = a.gerufen.iter().find(|&&g| an[g].is_none()) {
                    raus = Some((i, g));
                    break;
                }
            }
        }
        match raus {
            Some((i, g)) => { an[i] = None; gruende[i] = format!("ruft {}, das in der VM bleibt", alle[g].2); }
            None => break,
        }
    }
    (an, gruende)
}

/// Fuer `dhrt --jit datei.dh`: je Funktion "uebersetzt" oder der Grund.
pub fn bericht(prog: &Program) -> Vec<(String, String)> {
    let glob = globale_lesen(prog);
    let (_, gruende) = auswahl(prog, &glob);
    alle_funktionen(prog, &glob).into_iter().zip(gruende).map(|((_, _, name), g)| {
        (name, if g.is_empty() { "uebersetzt".to_string() } else { format!("VM: {}", g) })
    }).collect()
}

// ---------------------------------------------------------------------------
// Stufe 2: Code erzeugen
// ---------------------------------------------------------------------------

fn cl_typ(a: Art) -> Type { if a == Art::F { types::F64 } else { types::I64 } }

struct Bauer<'a, 'b> {
    b: &'a mut FunctionBuilder<'b>,
    ctx: CWert,
    fehler: Block,
    vars: HashMap<(u8, usize, Art), Variable>,
    schatten: CWert,
    marken: CWert,
    holen: cranelift_codegen::ir::FuncRef,
    /// Globale Plaetze, die keine gerufene Funktion anfasst: beim Eintritt
    /// einmal geholt, dann wie Locals in Variablen, am Ende zurueck in den
    /// Schatten (`befoerdert_schreiben`).
    befoerdert: HashMap<usize, (Variable, Art)>,
    /// Je Feld: Zeiger auf die Daten, Groessen, Schritte (beim Eintritt
    /// geladen -- die Felder aendern ihre Groesse im Bereich nicht).
    felder: Vec<(CWert, Vec<CWert>, Vec<CWert>)>,
    /// Aussteigen mitten im Bereich (siehe `Jit::schleife`): statt aufzugeben
    /// wird der Stand VOR dem Befehl zurueckgeschrieben, und die VM macht bei
    /// genau diesem Befehl weiter. `punkt` ist dieser Stand (Stelle, Stapel,
    /// Arten der Locals), `aussteige` sammelt die Stellen.
    mitten: bool,
    punkt: Option<(usize, Vec<(CWert, Art)>, Vec<Option<Art>>)>,
    aussteige: Vec<Aussteig>,
    lok_zeiger: Option<CWert>,
    zu_tief: bool,
    /// `Self` als Zeiger (nur Bereiche in Methoden, die es benutzen).
    selbst: Option<CWert>,
    /// Die Helfer fuer Objekte und Werte, im Modul deklariert.
    h_lesen_i: cranelift_codegen::ir::FuncRef,
    h_lesen_f: cranelift_codegen::ir::FuncRef,
    h_setzen_i: cranelift_codegen::ir::FuncRef,
    h_setzen_f: cranelift_codegen::ir::FuncRef,
    h_element_i: cranelift_codegen::ir::FuncRef,
    h_element_f: cranelift_codegen::ir::FuncRef,
    h_journal: cranelift_codegen::ir::FuncRef,
    h_mathe1: cranelift_codegen::ir::FuncRef,
    h_mathe2: cranelift_codegen::ir::FuncRef,
    /// Wertemodus: die Helfer `w_*` und der erste Platz des Stapels unter den
    /// Werteplaetzen (= Zahl aller Locals samt `dyn_glob`).
    hw: Vec<cranelift_codegen::ir::FuncRef>,
    w_basis: i64,
}

/// Eine Stelle, an der ein Bereich mitten drin aussteigt: dort macht die VM
/// weiter, mit diesen Locals und diesem Stapel.
#[derive(Clone)]
struct Aussteig {
    stelle: usize,
    lokal: Vec<Option<Art>>,
    stapel: Vec<Art>,
}

impl<'a, 'b> Bauer<'a, 'b> {
    fn var(&mut self, bereich: u8, nr: usize, a: Art) -> Variable {
        if let Some(v) = self.vars.get(&(bereich, nr, a)) { return *v; }
        let v = self.b.declare_var(cl_typ(a));
        self.vars.insert((bereich, nr, a), v);
        v
    }
    /// Bei `bed` != 0 in den Ausstieg springen, sonst weiter.
    fn aussteigen_wenn(&mut self, bed: CWert) {
        let weiter = self.b.create_block();
        match self.punkt.clone().filter(|_| self.mitten) {
            None => { self.b.ins().brif(bed, self.fehler, &[], weiter, &[]); }
            Some((stelle, st, lokal)) => {
                let raus = self.b.create_block();
                self.b.ins().brif(bed, raus, &[], weiter, &[]);
                self.b.switch_to_block(raus);
                self.befoerdert_schreiben();
                let lz = self.lok_zeiger.unwrap();
                for (i, a) in lokal.iter().enumerate() {
                    let Some(a) = a else { continue };
                    if matches!(a, Art::Feld(_) | Art::W) { continue; }
                    let v = self.var(1, i, *a);
                    let w = self.b.use_var(v);
                    self.b.ins().store(MemFlagsData::trusted(), w, lz, (i * 8) as i32);
                }
                if st.len() > MAX_STAPEL { self.zu_tief = true; }
                let sp = self.b.ins().load(types::I64, MemFlagsData::trusted(), self.ctx, K_STAPEL);
                for (d, (w, a)) in st.iter().enumerate().take(MAX_STAPEL) {
                    if skalar(*a) || matches!(a, Art::Obj(_)) { self.b.ins().store(MemFlagsData::trusted(), *w, sp, (d * 8) as i32); }
                }
                let nr = self.aussteige.len() as i64;
                self.aussteige.push(Aussteig { stelle, lokal, stapel: st.iter().map(|(_, a)| *a).collect() });
                let code = self.b.ins().iconst(types::I64, -2 - nr);
                self.b.ins().return_(&[code]);
            }
        }
        self.b.switch_to_block(weiter);
    }
    /// Adresse eines Feldelements, mit Pruefung jedes Index (sonst Ausstieg).
    fn feld_adresse(&mut self, nr: usize, idx: &[(CWert, Art)], breite: i64) -> CWert {
        let (zeiger, dims, schritte) = self.felder[nr].clone();
        let mut flach: Option<CWert> = None;
        for (k, (i, _)) in idx.iter().enumerate() {
            let aus = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, *i, dims[k]);
            self.aussteigen_wenn(aus);
            let teil = if idx.len() == 1 { *i } else { self.b.ins().imul(*i, schritte[k]) };
            flach = Some(match flach { None => teil, Some(f) => self.b.ins().iadd(f, teil) });
        }
        let off = if breite == 8 { self.b.ins().ishl_imm_s(flach.unwrap(), 3) } else { self.b.ins().imul_imm_s(flach.unwrap(), breite) };
        self.b.ins().iadd(zeiger, off)
    }
    fn iconst(&mut self, x: i64) -> CWert { self.b.ins().iconst(types::I64, x) }
    /// Ein Helfer `w_*` ohne Ergebnis.
    fn w_ruf(&mut self, h: usize, args: &[CWert]) {
        let mut a = vec![self.ctx];
        a.extend_from_slice(args);
        self.b.ins().call(self.hw[h], &a);
    }
    /// Ein Helfer `w_*` mit Ergebnis.
    fn w_ruf_w(&mut self, h: usize, args: &[CWert]) -> CWert {
        let mut a = vec![self.ctx];
        a.extend_from_slice(args);
        let r = self.b.ins().call(self.hw[h], &a);
        self.b.inst_results(r)[0]
    }
    fn w_slot(&self, tiefe: usize) -> i64 { self.w_basis + tiefe as i64 }
    /// Die Zahlen unter `st[von..]` in ihre Werteplaetze legen (ein Helfer
    /// rechnet danach nur mit Plaetzen).
    fn w_boxen(&mut self, st: &[(CWert, Art)], von: usize) {
        for d in von..st.len() {
            let (w, a) = st[d];
            let z = self.iconst(self.w_slot(d));
            match a {
                Art::F => self.w_ruf(W_ABLEGEN_F, &[w, z]),
                Art::I | Art::B => { let c = self.iconst(if a == Art::B { 3 } else { 1 }); self.w_ruf(W_ABLEGEN_I, &[w, c, z]) }
                _ => {}
            }
        }
    }
    /// Einen Wert aus Platz `von` in Local `s` legen: als Zahl, wenn das Local
    /// einen Zahlentyp hat, sonst als Wert (mit dem Typ des Locals geprueft).
    fn w_nach_lokal(&mut self, von: i64, s: usize, typ: &str, ziel: Option<Art>) -> Result<(), String> {
        let vz = self.iconst(von);
        match ziel {
            Some(a @ (Art::I | Art::B)) => {
                let c = self.iconst(if a == Art::B { 3 } else { 1 });
                let w = self.w_ruf_w(W_ZAHL_I, &[vz, c]);
                self.fehler_pruefen();
                let v = self.var(1, s, a); self.b.def_var(v, w);
            }
            Some(Art::F) => {
                let w = self.w_ruf_w(W_ZAHL_F, &[vz]);
                self.fehler_pruefen();
                let v = self.var(1, s, Art::F); self.b.def_var(v, w);
            }
            _ => {
                let t = if typ.is_empty() { "any" } else { typ };
                let (tp, tl) = (self.iconst(t.as_ptr() as i64), self.iconst(t.len() as i64));
                let sz = self.iconst(s as i64);
                self.w_ruf(W_SPEICHERN, &[vz, sz, tp, tl]);
                self.fehler_pruefen();
                let n = self.iconst(0);
                let v = self.var(1, s, Art::W); self.b.def_var(v, n);
            }
        }
        Ok(())
    }
    /// Nach einem Helfer, der scheitern kann: `fehler` gesetzt -> aussteigen.
    fn fehler_pruefen(&mut self) {
        let fl = self.b.ins().load(types::I64, MemFlagsData::trusted(), self.ctx, 0);
        self.aussteigen_wenn(fl);
    }
    /// Feld `platz` eines Objekts lesen (Art `a`; ein Objekt der Klasse mit
    /// Lage `lage`).
    fn feld_lesen(&mut self, obj: CWert, platz: usize, a: Art, lage: u64) -> CWert {
        let pl = self.b.ins().iconst(types::I64, platz as i64);
        if a == Art::F {
            let r = self.b.ins().call(self.h_lesen_f, &[self.ctx, obj, pl]);
            let w = self.b.inst_results(r)[0];
            self.fehler_pruefen();
            return w;
        }
        let code = match a { Art::B => 3, Art::Obj(_) => 4, _ => 1 };
        let c = self.b.ins().iconst(types::I64, code);
        let l = self.b.ins().iconst(types::I64, lage as i64);
        let r = self.b.ins().call(self.h_lesen_i, &[self.ctx, obj, pl, c, l]);
        let w = self.b.inst_results(r)[0];
        self.fehler_pruefen();
        w
    }
    fn feld_setzen(&mut self, obj: CWert, platz: usize, a: Art, w: CWert) {
        let pl = self.b.ins().iconst(types::I64, platz as i64);
        if a == Art::F {
            self.b.ins().call(self.h_setzen_f, &[self.ctx, obj, pl, w]);
        } else {
            let c = self.b.ins().iconst(types::I64, if a == Art::B { 3 } else { 1 });
            self.b.ins().call(self.h_setzen_i, &[self.ctx, obj, pl, c, w]);
        }
    }
    fn als_f(&mut self, v: CWert, a: Art) -> CWert {
        if a == Art::I { self.b.ins().fcvt_from_sint(types::F64, v) } else { v }
    }
    fn wahr(&mut self, v: CWert, a: Art) -> CWert {
        match a {
            Art::F => { let z = self.b.ins().f64const(0.0); self.b.ins().fcmp(FloatCC::NotEqual, v, z) }
            _ => self.b.ins().icmp_imm_s(IntCC::NotEqual, v, 0),
        }
    }
    fn bool64(&mut self, c: CWert) -> CWert { self.b.ins().uextend(types::I64, c) }
    /// Wie `passend!`/`coerce` beim Speichern: FLOAT -> INTEGER nur ohne
    /// Nachkommastellen und im Bereich, sonst Ausstieg.
    fn wandeln(&mut self, v: CWert, von: Art, nach: Art) -> CWert {
        match (von, nach) {
            (Art::I, Art::F) => self.b.ins().fcvt_from_sint(types::F64, v),
            (Art::F, Art::I) => {
                let t = self.b.ins().trunc(v);
                let krumm = self.b.ins().fcmp(FloatCC::NotEqual, t, v);   // auch NaN
                self.aussteigen_wenn(krumm);
                let lo = self.b.ins().f64const(i64::MIN as f64);
                let hi = self.b.ins().f64const(i64::MAX as f64);
                let u = self.b.ins().fcmp(FloatCC::LessThan, v, lo);
                let o = self.b.ins().fcmp(FloatCC::GreaterThan, v, hi);
                let aus = self.b.ins().bor(u, o);
                self.aussteigen_wenn(aus);
                self.b.ins().fcvt_to_sint_sat(types::I64, v)
            }
            _ => v,
        }
    }
    /// Globalen Platz lesen: steht er noch nicht im Schatten, holt ihn
    /// `global_holen` (und gibt bei einem leeren Platz auf).
    fn global_lesen(&mut self, g: usize, a: Art) -> CWert {
        if let Some((v, _)) = self.befoerdert.get(&g).copied() { return self.b.use_var(v); }
        self.global_holen_roh(g, a)
    }
    fn global_holen_roh(&mut self, g: usize, a: Art) -> CWert {
        let marke = self.b.ins().load(types::I8, MemFlagsData::trusted(), self.marken, g as i32);
        let holen = self.b.create_block();
        let da = self.b.create_block();
        self.b.ins().brif(marke, da, &[], holen, &[]);
        self.b.switch_to_block(holen);
        let nr = self.b.ins().iconst(types::I64, g as i64);
        self.b.ins().call(self.holen, &[self.ctx, nr]);
        let fl = self.b.ins().load(types::I64, MemFlagsData::trusted(), self.ctx, 0);
        self.aussteigen_wenn(fl);
        self.b.ins().jump(da, &[]);
        self.b.switch_to_block(da);
        self.b.ins().load(cl_typ(a), MemFlagsData::trusted(), self.schatten, (g * 8) as i32)
    }
    fn global_schreiben(&mut self, g: usize, w: CWert) {
        if let Some((v, _)) = self.befoerdert.get(&g).copied() { self.b.def_var(v, w); return; }
        self.global_schreiben_roh(g, w)
    }
    fn global_schreiben_roh(&mut self, g: usize, w: CWert) {
        self.b.ins().store(MemFlagsData::trusted(), w, self.schatten, (g * 8) as i32);
        let zwei = self.b.ins().iconst(types::I8, 2);
        self.b.ins().store(MemFlagsData::trusted(), zwei, self.marken, g as i32);
    }
    fn befoerdert_schreiben(&mut self) {
        let mut liste: Vec<(usize, Variable)> = self.befoerdert.iter().map(|(g, (v, _))| (*g, *v)).collect();
        liste.sort_by_key(|(g, _)| *g);
        for (g, v) in liste {
            let w = self.b.use_var(v);
            self.global_schreiben_roh(g, w);
        }
    }
    /// Vor jedem Ruecksprung aus einer Funktion.
    fn abschluss(&mut self) {
        self.befoerdert_schreiben();
        self.tiefe_minus();
    }
    fn tiefe_minus(&mut self) {
        let t = self.b.ins().load(types::I64, MemFlagsData::trusted(), self.ctx, 8);
        let t = self.b.ins().iadd_imm_s(t, -1);
        self.b.ins().store(MemFlagsData::trusted(), t, self.ctx, 8);
    }
}

/// Ein Bereich: (Kontext, Zeiger auf die Locals) -> Stelle, an der die VM
/// weitermacht (-1 = aufgegeben).
fn bereich_signatur(modul: &JITModule) -> cranelift_codegen::ir::Signature {
    let ptr = modul.target_config().pointer_type();
    let mut sig = modul.make_signature();
    sig.params.push(AbiParam::new(ptr));
    sig.params.push(AbiParam::new(ptr));
    sig.returns.push(AbiParam::new(types::I64));
    sig
}

fn signatur(modul: &JITModule, params: &[Art], rueck: Option<Art>) -> cranelift_codegen::ir::Signature {
    let mut sig = modul.make_signature();
    sig.params.push(AbiParam::new(modul.target_config().pointer_type()));
    for p in params { sig.params.push(AbiParam::new(cl_typ(*p))); }
    if let Some(r) = rueck { sig.returns.push(AbiParam::new(cl_typ(r))); }
    sig
}

fn erzeugen(modul: &mut JITModule, prog: &Program, glob: &Globale, f: &Func, an: &Analyse, ids: &[Option<FuncId>],
            hilfe: &Hilfe, fctx: &mut FunctionBuilderContext, id: FuncId,
            bereich: Option<(usize, usize)>, befoerdert: &[usize], mitten: bool) -> Result<Vec<Aussteig>, String> {
    let tc = modul.target_config();
    let mut ctx = modul.make_context();
    ctx.func.signature = match bereich {
        Some(_) => bereich_signatur(modul),
        None => signatur(modul, &an.params, an.rueck),
    };
    let (von, bis) = bereich.unwrap_or((0, f.code.len().saturating_sub(1)));
    ctx.func.name = UserFuncName::user(0, id.as_u32());
    let code = &f.code;
    {
        let mut fb = FunctionBuilder::new(&mut ctx.func, fctx);
        let eintritt = fb.create_block();
        fb.append_block_params_for_function_params(eintritt);
        let fehler = fb.create_block();

        // Anfaenge der Grundbloecke: Sprungziele und alles hinter einem Sprung.
        let mut anfang = vec![false; code.len() + 1];
        anfang[von] = true;
        anfang[bis + 1] = true;
        for (i, ins) in code.iter().enumerate().skip(von).take(bis + 1 - von) {
            match ins.op {
                op::JUMP | op::JUMP_IF_FALSE | op::JUMP_IF_TRUE => { anfang[ziel(&ins.arg)] = true; anfang[i + 1] = true; }
                op::FOR_NEXT => { if let Some(t) = for_teile(&ins.arg) { anfang[t[6] as usize] = true; } anfang[i + 1] = true; }
                op::RETURN | op::RETURN_VOID | op::HALT => { anfang[i + 1] = true; }
                _ => {}
            }
        }
        let mut bloecke: HashMap<usize, Block> = HashMap::new();
        for i in 0..=code.len() {
            let drin = i >= von && i <= bis;
            if an.vor[i].is_some() && ((drin && anfang[i]) || an.ausgaenge.contains(&i)) {
                bloecke.insert(i, fb.create_block());
            }
        }

        fb.switch_to_block(eintritt);
        let ctxp = fb.block_params(eintritt)[0];
        let pwerte: Vec<CWert> = fb.block_params(eintritt)[1..].to_vec();
        // Bereich: der zweite Parameter zeigt auf die Locals (je 8 Byte).
        let lok_zeiger = if bereich.is_some() { Some(pwerte[0]) } else { None };
        let mitten = mitten && bereich.is_some();
        let schatten = fb.ins().load(types::I64, MemFlagsData::trusted(), ctxp, K_SCHATTEN);
        let marken = fb.ins().load(types::I64, MemFlagsData::trusted(), ctxp, K_MARKEN);
        let holen = modul.declare_func_in_func(hilfe.holen, fb.func);
        let h_lesen_i = modul.declare_func_in_func(hilfe.lesen_i, fb.func);
        let h_lesen_f = modul.declare_func_in_func(hilfe.lesen_f, fb.func);
        let h_setzen_i = modul.declare_func_in_func(hilfe.setzen_i, fb.func);
        let h_setzen_f = modul.declare_func_in_func(hilfe.setzen_f, fb.func);
        let h_element_i = modul.declare_func_in_func(hilfe.element_i, fb.func);
        let h_element_f = modul.declare_func_in_func(hilfe.element_f, fb.func);
        let h_journal = modul.declare_func_in_func(hilfe.journal_schluss, fb.func);
        let h_mathe1 = modul.declare_func_in_func(hilfe.mathe1, fb.func);
        let h_mathe2 = modul.declare_func_in_func(hilfe.mathe2, fb.func);
        let selbst = if an.selbst.is_some() { Some(fb.ins().load(types::I64, MemFlagsData::trusted(), ctxp, K_SELBST)) } else { None };
        let mut bau = Bauer { b: &mut fb, ctx: ctxp, fehler, vars: HashMap::new(), schatten, marken, holen,
                              befoerdert: HashMap::new(), felder: Vec::new(), mitten, punkt: None,
                              aussteige: Vec::new(), lok_zeiger, zu_tief: false, selbst,
                              h_lesen_i, h_lesen_f, h_setzen_i, h_setzen_f, h_element_i, h_element_f, h_journal, h_mathe1, h_mathe2,
                              hw: Vec::new(), w_basis: an.typen.len() as i64 };
        if an.modus_w {
            for id in hilfe.w { let r = modul.declare_func_in_func(id, bau.b.func); bau.hw.push(r); }
        }
        let mut klassen = an.klassen.clone();
        // Die Felder beschreibt der Kontext: je Feld Zeiger, Groessen, Schritte.
        if !an.felder.is_empty() {
            let fz = bau.b.ins().load(types::I64, MemFlagsData::trusted(), ctxp, K_FELDER);
            let mut off = 0i32;
            for fi in &an.felder {
                let d = fi.dims as i32;
                let z = bau.b.ins().load(types::I64, MemFlagsData::trusted(), fz, off * 8);
                let dims: Vec<CWert> = (0..d).map(|k| bau.b.ins().load(types::I64, MemFlagsData::trusted(), fz, (off + 1 + k) * 8)).collect();
                let schr: Vec<CWert> = (0..d).map(|k| bau.b.ins().load(types::I64, MemFlagsData::trusted(), fz, (off + 1 + d + k) * 8)).collect();
                bau.felder.push((z, dims, schr));
                off += 1 + 2 * d;
            }
        }

        // Tiefe wie `exec`: erst zaehlen, dann gegen die Grenze. Ein Bereich
        // ist kein Aufruf und zaehlt nicht.
        if bereich.is_none() {
            let t = bau.b.ins().load(types::I64, MemFlagsData::trusted(), ctxp, 8);
            let t = bau.b.ins().iadd_imm_s(t, 1);
            bau.b.ins().store(MemFlagsData::trusted(), t, ctxp, 8);
            let g = bau.b.ins().load(types::I64, MemFlagsData::trusted(), ctxp, 16);
            let zu_tief = bau.b.ins().icmp(IntCC::UnsignedGreaterThan, t, g);
            bau.aussteigen_wenn(zu_tief);
        }
        let start = an.vor[von].as_ref().unwrap();
        for (i, a) in start.lokal.iter().enumerate() {
            let Some(a) = a else { continue };
            let wert = if let Art::Feld(nr) = a {
                bau.b.ins().iconst(types::I64, *nr as i64)
            } else if *a == Art::W {
                bau.b.ins().iconst(types::I64, 0)
            } else if let Some(lz) = lok_zeiger {
                bau.b.ins().load(cl_typ(*a), MemFlagsData::trusted(), lz, (i * 8) as i32)
            } else if i < f.n_params { pwerte[i] } else {
                match f.local_defaults.get(i) {
                    Some(Value::Int(x)) => bau.b.ins().iconst(types::I64, *x),
                    Some(Value::Float(x)) => bau.b.ins().f64const(*x),
                    Some(Value::Bool(x)) => bau.b.ins().iconst(types::I64, *x as i64),
                    _ => return Err("Vorgabe".into()),
                }
            };
            let v = bau.var(1, i, *a);
            bau.b.def_var(v, wert);
        }
        for &g in befoerdert {
            let a = glob.art[g].unwrap();
            let w = bau.global_holen_roh(g, a);
            let v = bau.b.declare_var(cl_typ(a));
            bau.b.def_var(v, w);
            bau.befoerdert.insert(g, (v, a));
        }
        let b0 = bloecke[&von];
        bau.b.ins().jump(b0, &[]);

        // Den Stapel am Blockende in die Variablen legen, am Blockanfang holen.
        fn ablegen(bau: &mut Bauer, st: &[(CWert, Art)]) {
            for (d, (w, a)) in st.iter().enumerate() { let v = bau.var(0, d, *a); bau.b.def_var(v, *w); }
        }

        let mut ip = von;
        while ip <= bis && ip < code.len() {
            if an.vor[ip].is_none() { ip += 1; continue; }
            // Blockanfang
            let blk = bloecke[&ip];
            bau.b.switch_to_block(blk);
            let mut st: Vec<(CWert, Art)> = Vec::new();
            for (d, a) in an.vor[ip].as_ref().unwrap().stapel.iter().enumerate() {
                let v = bau.var(0, d, *a);
                st.push((bau.b.use_var(v), *a));
            }
            let mut offen = true;
            loop {
                let ins = &code[ip];
                let z = an.vor[ip].as_ref().unwrap();
                bau.punkt = Some((ip, st.clone(), z.lokal.clone()));
                let (o, s_arg) = umleiten(ins, &an.dyn_glob, f.local_types.len());
                let oben_w = st.last().map_or(false, |x| x.1 == Art::W);
                let zwei_w = st.len() >= 2 && st[st.len() - 2..].iter().any(|x| x.1 == Art::W);
                let nach_art = |ip: usize| an.vor.get(ip + 1).and_then(|z| z.as_ref()).and_then(|z| z.stapel.last().copied());
                match o {
                    op::POP if oben_w => {
                        let z = bau.iconst(bau.w_slot(st.len() - 1));
                        bau.w_ruf(W_FREI, &[z]);
                        st.pop();
                    }
                    op::DUP if oben_w => {
                        let (a, b) = (bau.iconst(bau.w_slot(st.len() - 1)), bau.iconst(bau.w_slot(st.len())));
                        bau.w_ruf(W_KOPIE, &[a, b]);
                        let n = bau.iconst(0);
                        st.push((n, Art::W));
                    }
                    op::LOAD_CONST if an.modus_w && nach_art(ip) == Some(Art::W) => {
                        let pz = bau.iconst(&f.constants[ins.arg.as_usize()] as *const Value as i64);
                        let z = bau.iconst(bau.w_slot(st.len()));
                        bau.w_ruf(W_KONST, &[pz, z]);
                        let n = bau.iconst(0);
                        st.push((n, Art::W));
                    }
                    op::LOAD_LOCAL if z.lokal.get(s_arg).copied().flatten() == Some(Art::W) => {
                        let (a, b) = (bau.iconst(s_arg as i64), bau.iconst(bau.w_slot(st.len())));
                        bau.w_ruf(W_KOPIE, &[a, b]);
                        let n = bau.iconst(0);
                        st.push((n, Art::W));
                    }
                    op::STORE_LOCAL | op::ADD_STORE_LOCAL
                        if (o == op::STORE_LOCAL && oben_w) || (o == op::ADD_STORE_LOCAL && zwei_w) => {
                        let s = s_arg;
                        let ziel_art = an.vor[ip + 1].as_ref().and_then(|z| z.lokal[s]).filter(|a| *a != Art::W);
                        let quelle = if o == op::ADD_STORE_LOCAL {
                            let d = st.len() - 2;
                            bau.w_boxen(&st, d);
                            // Haelt das Local den Text links, gibt es ihn ab (anhaengen).
                            let lokal = if z.lokal[s] == Some(Art::W) && ziel_art.is_none() { s as i64 } else { -1 };
                            let (c, a, b, zl, lk) = (bau.iconst(op::ADD as i64), bau.iconst(bau.w_slot(d)), bau.iconst(bau.w_slot(d + 1)),
                                                     bau.iconst(bau.w_slot(d)), bau.iconst(lokal));
                            bau.w_ruf_w(W_OP, &[c, a, b, zl, lk]);
                            bau.fehler_pruefen();
                            st.truncate(d);
                            bau.w_slot(d)
                        } else {
                            st.pop();
                            bau.w_slot(st.len())
                        };
                        bau.w_nach_lokal(quelle, s, &an.typen[s], ziel_art)?;
                    }
                    op::STORE_GLOBAL_SLOT | op::ADD_STORE_GLOBAL_SLOT
                        if (o == op::STORE_GLOBAL_SLOT && oben_w) || (o == op::ADD_STORE_GLOBAL_SLOT && zwei_w) => {
                        let g = ins.arg.as_usize();
                        let d = if o == op::ADD_STORE_GLOBAL_SLOT {
                            let d = st.len() - 2;
                            bau.w_boxen(&st, d);
                            let (c, a, b, zl, lk) = (bau.iconst(op::ADD as i64), bau.iconst(bau.w_slot(d)), bau.iconst(bau.w_slot(d + 1)),
                                                     bau.iconst(bau.w_slot(d)), bau.iconst(-1));
                            bau.w_ruf_w(W_OP, &[c, a, b, zl, lk]);
                            bau.fehler_pruefen();
                            st.truncate(d);
                            d
                        } else { st.pop(); st.len() };
                        let a = glob.art[g].unwrap();
                        let vz = bau.iconst(bau.w_slot(d));
                        let w = if a == Art::F {
                            bau.w_ruf_w(W_ZAHL_F, &[vz])
                        } else {
                            let c = bau.iconst(if a == Art::B { 3 } else { 1 });
                            bau.w_ruf_w(W_ZAHL_I, &[vz, c])
                        };
                        bau.fehler_pruefen();
                        bau.global_schreiben(g, w);
                    }
                    op::ADD | op::SUB | op::MUL | op::DIV | op::MOD | op::INT_DIV
                    | op::LT | op::GT | op::LEQ | op::GEQ | op::EQ | op::NEQ if an.modus_w && zwei_w => {
                        let d = st.len() - 2;
                        bau.w_boxen(&st, d);
                        let (c, a, b, zl, lk) = (bau.iconst(o as i64), bau.iconst(bau.w_slot(d)), bau.iconst(bau.w_slot(d + 1)),
                                                 bau.iconst(bau.w_slot(d)), bau.iconst(-1));
                        let r = bau.w_ruf_w(W_OP, &[c, a, b, zl, lk]);
                        bau.fehler_pruefen();
                        st.truncate(d);
                        if matches!(o, op::LT | op::GT | op::LEQ | op::GEQ | op::EQ | op::NEQ) {
                            st.push((r, Art::B));
                        } else {
                            let n = bau.iconst(0);
                            st.push((n, Art::W));
                        }
                    }
                    op::JUMP_IF_FALSE | op::JUMP_IF_TRUE if oben_w => {
                        let z = bau.iconst(bau.w_slot(st.len() - 1));
                        let w = bau.w_ruf_w(W_WAHR, &[z]);
                        st.pop();
                        ablegen(&mut bau, &st);
                        let (z_ja, z_nein) = (bloecke[&ziel(&ins.arg)], bloecke[&(ip + 1)]);
                        if o == op::JUMP_IF_TRUE {
                            bau.b.ins().brif(w, z_ja, &[], z_nein, &[]);
                        } else {
                            bau.b.ins().brif(w, z_nein, &[], z_ja, &[]);
                        }
                        offen = false;
                    }
                    op::CALL_BUILTIN if match &ins.arg {
                        Arg::Call(n, c, _) => st.len() >= *c as usize && {
                            let arts: Vec<Art> = st[st.len() - *c as usize..].iter().map(|(_, a)| *a).collect();
                            zahl_befehl(n, &arts).is_some()
                        },
                        _ => false,
                    } => {
                        let (name, argc) = match &ins.arg { Arg::Call(n, c, _) => (n.clone(), *c as usize), _ => unreachable!() };
                        let teil = st.split_off(st.len() - argc);
                        let erg = zahl_rechnen(&mut bau, &name, &teil);
                        st.push(erg);
                    }
                    op::CALL_BUILTIN => {
                        let argc = match &ins.arg { Arg::Call(_, c, _) => *c as usize, _ => unreachable!() };
                        let d = st.len() - argc;
                        bau.w_boxen(&st, d);
                        st.truncate(d);
                        let erg = nach_art(ip).unwrap();
                        let (ip_c, basis, n) = (bau.iconst(ins as *const crate::model::Instr as i64), bau.iconst(bau.w_slot(d)), bau.iconst(argc as i64));
                        let w = if erg == Art::F {
                            bau.w_ruf_w(W_BUILTIN_F, &[ip_c, basis, n])
                        } else {
                            let c = bau.iconst(match erg { Art::I => 1, Art::B => 3, _ => 5 });
                            bau.w_ruf_w(W_BUILTIN_I, &[ip_c, basis, n, c])
                        };
                        bau.fehler_pruefen();
                        st.push((w, erg));
                    }
                    op::CALL_METHOD if !an.modus_w => {
                        let (name, argc) = match &ins.arg { Arg::Call(n, c, _) => (n.clone(), *c as usize), _ => unreachable!() };
                        let teil = st.split_off(st.len() - argc);
                        let Some((obj, Art::Obj(k))) = st.pop() else { unreachable!() };
                        let lage = klassen[k as usize].lage;
                        let j = glob.tafel.iter().position(|e| std::ptr::eq(e.lage, lage) && e.name.eq_ignore_ascii_case(&name)).unwrap();
                        let g: &Func = unsafe { &*glob.tafel[j].func };
                        let idx = prog.functions.len() + j;
                        let mut args = Vec::with_capacity(argc + 1);
                        args.push(bau.ctx);
                        for (i, (w, a)) in teil.into_iter().enumerate() {
                            let p = art_von_typ(&g.local_types[i]).unwrap();
                            args.push(bau.wandeln(w, a, p));
                        }
                        // `Self` des Gerufenen; der Rufer hat sein eigenes beim
                        // Eintritt gelesen und braucht es nicht zurueck.
                        bau.b.ins().store(MemFlagsData::trusted(), obj, bau.ctx, K_SELBST);
                        if bereich.is_some() {
                            let jb = bau.b.ins().load(types::I64, MemFlagsData::trusted(), bau.ctx, K_JOURNAL_BEREICH);
                            bau.b.ins().store(MemFlagsData::trusted(), jb, bau.ctx, K_JOURNAL);
                        }
                        let fref = modul.declare_func_in_func(ids[idx].unwrap(), bau.b.func);
                        let ruf = bau.b.ins().call(fref, &args);
                        let erg = bau.b.inst_results(ruf).first().copied();
                        if bereich.is_some() { bau.b.ins().call(bau.h_journal, &[bau.ctx]); }
                        let fl = bau.b.ins().load(types::I64, MemFlagsData::trusted(), bau.ctx, 0);
                        bau.aussteigen_wenn(fl);
                        match erg {
                            Some(w) => st.push((w, art_von_typ(&g.return_type).unwrap())),
                            None => { let n = bau.b.ins().iconst(types::I64, 0); st.push((n, Art::N)); }
                        }
                    }
                    op::PRINT => {
                        let n = ins.arg.list()[0].as_usize();
                        let d = st.len() - n;
                        bau.w_boxen(&st, d);
                        st.truncate(d);
                        let (ip_c, basis, nc) = (bau.iconst(ins as *const crate::model::Instr as i64), bau.iconst(bau.w_slot(d)), bau.iconst(n as i64));
                        bau.w_ruf(W_DRUCKEN, &[ip_c, basis, nc]);
                    }
                    op::CALL_METHOD => {
                        let argc = match &ins.arg { Arg::Call(_, c, _) => *c as usize, _ => unreachable!() };
                        let d = st.len() - argc - 1;
                        bau.w_boxen(&st, d);
                        st.truncate(d);
                        let (ip_c, basis, n) = (bau.iconst(ins as *const crate::model::Instr as i64), bau.iconst(bau.w_slot(d)), bau.iconst(argc as i64));
                        bau.w_ruf(W_METHODE, &[ip_c, basis, n]);
                        bau.fehler_pruefen();
                        let z = bau.iconst(0);
                        st.push((z, Art::W));
                    }
                    op::LOAD_INDEX | op::STORE_INDEX if an.modus_w => {
                        let n = ins.arg.as_usize();
                        let d = st.len() - n - 1 - (o == op::STORE_INDEX) as usize;
                        bau.w_boxen(&st, d);
                        st.truncate(d);
                        let (basis, nz) = (bau.iconst(bau.w_slot(d)), bau.iconst(n as i64));
                        if o == op::LOAD_INDEX {
                            bau.w_ruf(W_INDEX, &[basis, nz]);
                            bau.fehler_pruefen();
                            let z = bau.iconst(0);
                            st.push((z, Art::W));
                        } else {
                            bau.w_ruf(W_SETZEN, &[basis, nz]);
                            bau.fehler_pruefen();
                        }
                    }
                    op::LOAD_INDEX => {
                        let n = ins.arg.as_usize();
                        let idx = st.split_off(st.len() - n);
                        let Some((_, Art::Feld(nr))) = st.pop() else { unreachable!() };
                        let fi = an.felder[nr as usize];
                        let adr = bau.feld_adresse(nr as usize, &idx, fi.breite());
                        let a = fi.art();
                        let w = match fi.elem {
                            Elem::I | Elem::F => bau.b.ins().load(cl_typ(a), MemFlagsData::trusted(), adr, 0),
                            Elem::Wert(Art::F) => {
                                let r = bau.b.ins().call(bau.h_element_f, &[bau.ctx, adr]);
                                let w = bau.b.inst_results(r)[0];
                                bau.fehler_pruefen();
                                w
                            }
                            Elem::Wert(a) => {
                                let (code, lage) = match a {
                                    Art::B => (3, 0), Art::Obj(k) => (4, klassen[k as usize].lage as i64), _ => (1, 0),
                                };
                                let c = bau.b.ins().iconst(types::I64, code);
                                let l = bau.b.ins().iconst(types::I64, lage);
                                let r = bau.b.ins().call(bau.h_element_i, &[bau.ctx, adr, c, l]);
                                let w = bau.b.inst_results(r)[0];
                                bau.fehler_pruefen();
                                w
                            }
                        };
                        st.push((w, a));
                    }
                    op::STORE_INDEX => {
                        let n = ins.arg.as_usize();
                        let (w, wa) = st.pop().unwrap();
                        let idx = st.split_off(st.len() - n);
                        let Some((_, Art::Feld(nr))) = st.pop() else { unreachable!() };
                        let fi = an.felder[nr as usize];
                        let a = fi.art();
                        let adr = bau.feld_adresse(nr as usize, &idx, fi.breite());
                        let w = bau.wandeln(w, wa, a);
                        bau.b.ins().store(MemFlagsData::trusted(), w, adr, 0);
                    }
                    op::LOAD_SELF => {
                        st.push((bau.selbst.unwrap(), Art::Obj(an.selbst.unwrap())));
                    }
                    op::LOAD_NAME if vorbelegt(prog, f, ins).is_some() => {
                        st.push(match vorbelegt(prog, f, ins) {
                            Some(Value::Float(x)) => (bau.b.ins().f64const(x), Art::F),
                            Some(Value::Int(x)) => (bau.iconst(x), Art::I),
                            _ => unreachable!(),
                        });
                    }
                    op::LOAD_FIELD | op::LOAD_MEMBER | op::STORE_FIELD | op::STORE_MEMBER => {
                        let name = match &f.constants[ins.arg.as_usize()] { Value::Str(n) => n.to_string(), _ => unreachable!() };
                        let schreiben = matches!(o, op::STORE_FIELD | op::STORE_MEMBER);
                        let wert = if schreiben { st.pop() } else { None };
                        let (obj, k) = if matches!(o, op::LOAD_FIELD | op::STORE_FIELD) {
                            (bau.selbst.unwrap(), an.selbst.unwrap())
                        } else {
                            let Some((w, Art::Obj(k))) = st.pop() else { unreachable!() };
                            (w, k)
                        };
                        let mit_props = matches!(o, op::LOAD_MEMBER | op::STORE_MEMBER);
                        let (platz, a) = feld_von(prog, &mut klassen, k as usize, &name, mit_props)?;
                        match wert {
                            None => {
                                let lage = match a { Art::Obj(k2) => klassen[k2 as usize].lage as u64, _ => 0 };
                                let w = bau.feld_lesen(obj, platz, a, lage);
                                st.push((w, a));
                            }
                            Some((w, wa)) => {
                                let w = bau.wandeln(w, wa, a);
                                bau.feld_setzen(obj, platz, a, w);
                            }
                        }
                    }
                    op::LOAD_CONST => {
                        let w = match &f.constants[ins.arg.as_usize()] {
                            Value::Int(x) => (bau.b.ins().iconst(types::I64, *x), Art::I),
                            Value::Float(x) => (bau.b.ins().f64const(*x), Art::F),
                            Value::Bool(x) => (bau.b.ins().iconst(types::I64, *x as i64), Art::B),
                            // Text vor HALT (siehe Analyse): wird nie benutzt.
                            _ => (bau.b.ins().iconst(types::I64, 0), Art::N),
                        };
                        st.push(w);
                    }
                    op::LOAD_LOCAL => {
                        let s = s_arg;
                        let a = z.lokal[s].unwrap();
                        let v = bau.var(1, s, a);
                        st.push((bau.b.use_var(v), a));
                    }
                    op::LOAD_GLOBAL_SLOT if glob.art[ins.arg.as_usize()].is_none() => {
                        // Ein Feld: welches, sagt der Zustand dahinter.
                        let a = *an.vor[ip + 1].as_ref().unwrap().stapel.last().unwrap();
                        let Art::Feld(nr) = a else { unreachable!() };
                        st.push((bau.b.ins().iconst(types::I64, nr as i64), a));
                    }
                    op::LOAD_GLOBAL_SLOT => {
                        let g = ins.arg.as_usize();
                        let a = glob.art[g].unwrap();
                        let w = bau.global_lesen(g, a);
                        st.push((w, a));
                    }
                    op::STORE_GLOBAL_SLOT | op::ADD_STORE_GLOBAL_SLOT => {
                        let g = ins.arg.as_usize();
                        let (mut w, mut a) = st.pop().unwrap();
                        if ins.op == op::ADD_STORE_GLOBAL_SLOT {
                            let (x, xa) = st.pop().unwrap();
                            (w, a) = rechnen(&mut bau, op::ADD, x, xa, w, a);
                        }
                        let w = bau.wandeln(w, a, glob.art[g].unwrap());
                        bau.global_schreiben(g, w);
                    }
                    op::STORE_LOCAL | op::ADD_STORE_LOCAL => {
                        let s = s_arg;
                        let (mut w, mut a) = st.pop().unwrap();
                        if o == op::ADD_STORE_LOCAL {
                            let (x, xa) = st.pop().unwrap();
                            (w, a) = rechnen(&mut bau, op::ADD, x, xa, w, a);
                        }
                        let nach = if matches!(a, Art::Feld(_) | Art::Obj(_)) { a } else { art_von_typ(&an.typen[s]).unwrap_or(a) };
                        let w = bau.wandeln(w, a, nach);
                        let v = bau.var(1, s, nach);
                        bau.b.def_var(v, w);
                    }
                    op::DECLARE_LOCAL => {
                        // Nur ein noch leerer Platz wird belegt (wie dispatch_selten).
                        let l = match &ins.arg { Arg::List(l) => l, _ => unreachable!() };
                        let s = l[0].as_usize();
                        if z.lokal[s].is_none() {
                            let w = match &l[2] {
                                Arg::Int(x) => Some((bau.b.ins().iconst(types::I64, *x), Art::I)),
                                Arg::Val(Value::Int(x)) => Some((bau.b.ins().iconst(types::I64, *x), Art::I)),
                                Arg::Val(Value::Float(x)) => Some((bau.b.ins().f64const(*x), Art::F)),
                                Arg::Val(Value::Bool(x)) => Some((bau.b.ins().iconst(types::I64, *x as i64), Art::B)),
                                _ => None,
                            };
                            if let Some((w, a)) = w { let v = bau.var(1, s, a); bau.b.def_var(v, w); }
                        }
                    }
                    op::DECLARE_GLOBAL_SLOT => {}   // Platz gibt es schon (siehe Bereich)
                    op::POP => { st.pop(); }
                    op::DUP => { let t = *st.last().unwrap(); st.push(t); }
                    op::ADD | op::SUB | op::MUL | op::DIV | op::MOD | op::INT_DIV => {
                        let (y, ya) = st.pop().unwrap(); let (x, xa) = st.pop().unwrap();
                        st.push(rechnen(&mut bau, ins.op, x, xa, y, ya));
                    }
                    op::NEG => {
                        let (x, a) = st.pop().unwrap();
                        if a == Art::I {
                            let min = bau.b.ins().icmp_imm_s(IntCC::Equal, x, i64::MIN);
                            bau.aussteigen_wenn(min);
                            st.push((bau.b.ins().ineg(x), Art::I));
                        } else {
                            st.push((bau.b.ins().fneg(x), Art::F));
                        }
                    }
                    op::LT | op::GT | op::LEQ | op::GEQ | op::EQ | op::NEQ => {
                        let (y, ya) = st.pop().unwrap(); let (x, xa) = st.pop().unwrap();
                        let c = vergleichen(&mut bau, ins.op, x, xa, y, ya);
                        st.push((bau.bool64(c), Art::B));
                    }
                    op::NOT if st.last().map_or(false, |(_, a)| *a == Art::W) => {
                        // Wie JUMP_IF: `w_wahr` nimmt den Wert vom Platz und
                        // sagt `truthy()` -- die VM rechnet `!v.truthy()`.
                        let z = bau.iconst(bau.w_slot(st.len() - 1));
                        st.pop();
                        let w = bau.w_ruf_w(W_WAHR, &[z]);
                        let n = bau.b.ins().bxor_imm(w, 1);
                        st.push((n, Art::B));
                    }
                    op::NOT => {
                        let (x, a) = st.pop().unwrap();
                        let w = bau.wahr(x, a);
                        let n = bau.b.ins().bxor_imm_s(w, 1);
                        st.push((bau.bool64(n), Art::B));
                    }
                    op::JUMP => {
                        ablegen(&mut bau, &st);
                        bau.b.ins().jump(bloecke[&ziel(&ins.arg)], &[]);
                        offen = false;
                    }
                    op::JUMP_IF_FALSE | op::JUMP_IF_TRUE => {
                        let (x, a) = st.pop().unwrap();
                        let w = bau.wahr(x, a);
                        ablegen(&mut bau, &st);
                        let (z_ja, z_nein) = (bloecke[&ziel(&ins.arg)], bloecke[&(ip + 1)]);
                        if ins.op == op::JUMP_IF_TRUE {
                            bau.b.ins().brif(w, z_ja, &[], z_nein, &[]);
                        } else {
                            bau.b.ins().brif(w, z_nein, &[], z_ja, &[]);
                        }
                        offen = false;
                    }
                    op::CALL_USER => {
                        let (argc, idx) = match &ins.arg { Arg::Call(_, c, i) => (*c as usize, *i as usize), _ => unreachable!() };
                        let g = &prog.functions[idx];
                        let mut args = Vec::with_capacity(argc + 1);
                        args.push(bau.ctx);
                        let teil = st.split_off(st.len() - argc);
                        for (i, (w, a)) in teil.into_iter().enumerate() {
                            let p = art_von_typ(&g.local_types[i]).unwrap();
                            args.push(bau.wandeln(w, a, p));
                        }
                        let fref = modul.declare_func_in_func(ids[idx].unwrap(), bau.b.func);
                        let ruf = bau.b.ins().call(fref, &args);
                        let erg = bau.b.inst_results(ruf).first().copied();
                        // Gab der Gerufene auf, gibt auch dieser auf.
                        let fl = bau.b.ins().load(types::I64, MemFlagsData::trusted(), bau.ctx, 0);
                        bau.aussteigen_wenn(fl);
                        match erg {
                            Some(w) => st.push((w, art_von_typ(&g.return_type).unwrap())),
                            None => { let n = bau.b.ins().iconst(types::I64, 0); st.push((n, Art::N)); }
                        }
                    }
                    op::RETURN => {
                        let (w, a) = st.pop().unwrap();
                        let w = bau.wandeln(w, a, an.rueck.unwrap());
                        bau.abschluss();
                        bau.b.ins().return_(&[w]);
                        offen = false;
                    }
                    op::RETURN_VOID if an.rueck.is_none() => {
                        bau.abschluss();
                        bau.b.ins().return_(&[]);
                        offen = false;
                    }
                    // Eine FUNCTION ohne RETURN liefert in der VM NIL -- das
                    // rechnet die VM nach.
                    op::RETURN_VOID | op::HALT => {
                        bau.b.ins().jump(fehler, &[]);
                        offen = false;
                    }
                    op::FOR_NEXT => {
                        let t = for_teile(&ins.arg).unwrap();
                        let global = t[0] == 1;
                        let v = bau.var(1, t[1] as usize, Art::I);
                        let cur = if global { bau.global_lesen(t[1] as usize, Art::I) } else { bau.b.use_var(v) };
                        let e = bau.var(1, t[2] as usize, Art::I);
                        let en = bau.b.use_var(e);
                        let schritt = if t[3] == 1 {
                            let s = bau.var(1, t[4] as usize, Art::I); bau.b.use_var(s)
                        } else {
                            match &f.constants[t[4] as usize] { Value::Int(x) => bau.b.ins().iconst(types::I64, *x), _ => unreachable!() }
                        };
                        let (next, ueber) = bau.b.ins().sadd_overflow(cur, schritt);
                        bau.aussteigen_wenn(ueber);
                        if global { bau.global_schreiben(t[1] as usize, next); } else { bau.b.def_var(v, next); }
                        let raus = if t[5] == 1 { bau.b.ins().icmp(IntCC::SignedLessThan, next, en) }
                                   else { bau.b.ins().icmp(IntCC::SignedGreaterThan, next, en) };
                        ablegen(&mut bau, &st);
                        bau.b.ins().brif(raus, bloecke[&(ip + 1)], &[], bloecke[&(t[6] as usize)], &[]);
                        offen = false;
                    }
                    _ => return Err("unerwarteter Befehl".into()),
                }
                ip += 1;
                if !offen { break; }
                if ip >= code.len() || anfang[ip] {
                    // Weiter in den naechsten Block (oder ans Ende = NIL).
                    ablegen(&mut bau, &st);
                    match bloecke.get(&ip) {
                        Some(b) if ip < code.len() || bereich.is_some() => { bau.b.ins().jump(*b, &[]); }
                        _ => {
                            if an.rueck.is_none() { bau.abschluss(); bau.b.ins().return_(&[]); }
                            else { bau.b.ins().jump(fehler, &[]); }
                        }
                    }
                    break;
                }
            }
        }
        // Bereich: an jedem Ausgang die Locals zurueck in den Speicher der VM
        // und die Stelle melden, an der sie weitermacht.
        if let Some(lz) = lok_zeiger {
            for &x in &an.ausgaenge {
                bau.b.switch_to_block(bloecke[&x]);
                bau.befoerdert_schreiben();
                let z = an.vor[x].as_ref().unwrap();
                for (i, a) in z.lokal.iter().enumerate() {
                    let Some(a) = a else { continue };
                    if matches!(a, Art::Feld(_) | Art::W) { continue; }
                    let v = bau.var(1, i, *a);
                    let w = bau.b.use_var(v);
                    bau.b.ins().store(MemFlagsData::trusted(), w, lz, (i * 8) as i32);
                }
                let x = bau.b.ins().iconst(types::I64, x as i64);
                bau.b.ins().return_(&[x]);
            }
        }
        // Ein erreichbares Ende ohne Befehl (Sprung hinter den letzten).
        if let Some(b) = bloecke.get(&code.len()).copied().filter(|_| bereich.is_none()) {
            bau.b.switch_to_block(b);
            if an.rueck.is_none() { bau.abschluss(); bau.b.ins().return_(&[]); }
            else { bau.b.ins().jump(fehler, &[]); }
        }

        // Der Ausstieg: Fehler melden, irgendeinen Wert liefern.
        bau.b.switch_to_block(fehler);
        let eins = bau.b.ins().iconst(types::I64, 1);
        bau.b.ins().store(MemFlagsData::trusted(), eins, bau.ctx, 0);
        if bereich.is_some() {
            let m = bau.b.ins().iconst(types::I64, -1);
            bau.b.ins().return_(&[m]);
        } else { match an.rueck {
            Some(Art::F) => { let z = bau.b.ins().f64const(0.0); bau.b.ins().return_(&[z]); }
            Some(_) => { let z = bau.b.ins().iconst(types::I64, 0); bau.b.ins().return_(&[z]); }
            None => { bau.b.ins().return_(&[]); }
        } }
        let zu_tief = bau.zu_tief;
        let aussteige = std::mem::take(&mut bau.aussteige);
        fb.seal_all_blocks();
        fb.finalize(tc);
        if zu_tief { return Err("mehr als 64 Werte auf dem Stapel".into()); }
        modul.define_function(id, &mut ctx).map_err(|e| format!("Cranelift: {:?}", e))?;
        modul.clear_context(&mut ctx);
        Ok(aussteige)
    }
}

/// + - * / MOD \ mit den Regeln der VM (`zahlen_addieren`, `nn_arith`,
/// `div`, `modulo`, `int_div`); jeder Fehlerfall ist ein Ausstieg.
/// Ein Befehl aus `zahl_befehl`, gerechnet wie builtins.rs. Die Art des
/// Ergebnisses sagt `zahl_befehl`; hier steht nur, WIE.
fn zahl_rechnen(bau: &mut Bauer, name: &str, args: &[(CWert, Art)]) -> (CWert, Art) {
    let arts: Vec<Art> = args.iter().map(|(_, a)| *a).collect();
    let art = zahl_befehl(name, &arts).unwrap();
    // `need_num`: eine Ganzzahl geht als Kommazahl hinein, auch wo das
    // Ergebnis wieder eine Ganzzahl ist (INT(2^53 + 1) rundet wie in der VM).
    let f: Vec<CWert> = args.iter().map(|(w, a)| bau.als_f(*w, *a)).collect();
    let m1 = |bau: &mut Bauer, k: i64, x: CWert| -> CWert {
        let c = bau.iconst(k);
        let r = bau.b.ins().call(bau.h_mathe1, &[c, x]);
        bau.b.inst_results(r)[0]
    };
    let w = match name {
        "abs" if args[0].1 == Art::I => {
            let x = args[0].0;
            let min = bau.b.ins().icmp_imm_s(IntCC::Equal, x, i64::MIN);
            bau.aussteigen_wenn(min);
            bau.b.ins().iabs(x)
        }
        "abs" => bau.b.ins().fabs(f[0]),
        "int" => {
            let fl = bau.b.ins().floor(f[0]);
            let lo = bau.b.ins().f64const(i64::MIN as f64);
            let hi = bau.b.ins().f64const(i64::MAX as f64);
            let u = bau.b.ins().fcmp(FloatCC::UnorderedOrLessThan, fl, lo);   // auch NaN
            let o = bau.b.ins().fcmp(FloatCC::GreaterThan, fl, hi);
            let aus = bau.b.ins().bor(u, o);
            bau.aussteigen_wenn(aus);
            bau.b.ins().fcvt_to_sint_sat(types::I64, fl)
        }
        // `as i64` in Rust saettigt und macht aus NaN 0 -- wie fcvt_to_sint_sat.
        "floor" => { let r = bau.b.ins().floor(f[0]); bau.b.ins().fcvt_to_sint_sat(types::I64, r) }
        "ceil" => { let r = bau.b.ins().ceil(f[0]); bau.b.ins().fcvt_to_sint_sat(types::I64, r) }
        // round_half_even == IEEE-Runden zur geraden Zahl (`nearest`).
        "round" => { let r = bau.b.ins().nearest(f[0]); bau.b.ins().fcvt_to_sint_sat(types::I64, r) }
        "sgn" | "sign" => {
            let z = bau.b.ins().f64const(0.0);
            let pos = bau.b.ins().fcmp(FloatCC::GreaterThan, f[0], z);
            let neg = bau.b.ins().fcmp(FloatCC::LessThan, f[0], z);
            let p = bau.bool64(pos);
            let n = bau.bool64(neg);
            bau.b.ins().isub(p, n)
        }
        "flt" => f[0],
        "sqr" | "sqrt" => {
            let z = bau.b.ins().f64const(0.0);
            let neg = bau.b.ins().fcmp(FloatCC::LessThan, f[0], z);
            bau.aussteigen_wenn(neg);
            bau.b.ins().sqrt(f[0])
        }
        "sin" => m1(bau, 0, f[0]),
        "cos" => m1(bau, 1, f[0]),
        "tan" => m1(bau, 2, f[0]),
        "atan" => m1(bau, 3, f[0]),
        "exp" => m1(bau, 4, f[0]),
        "log" => {
            let z = bau.b.ins().f64const(0.0);
            let aus = bau.b.ins().fcmp(FloatCC::LessThanOrEqual, f[0], z);
            bau.aussteigen_wenn(aus);
            m1(bau, 5, f[0])
        }
        "asin" | "acos" => {
            let lo = bau.b.ins().f64const(-1.0);
            let hi = bau.b.ins().f64const(1.0);
            let u = bau.b.ins().fcmp(FloatCC::LessThan, f[0], lo);
            let o = bau.b.ins().fcmp(FloatCC::GreaterThan, f[0], hi);
            let aus = bau.b.ins().bor(u, o);
            bau.aussteigen_wenn(aus);
            m1(bau, if name == "asin" { 6 } else { 7 }, f[0])
        }
        "deg" => {
            let c = bau.b.ins().f64const(180.0);
            let pi = bau.b.ins().f64const(std::f64::consts::PI);
            let m = bau.b.ins().fmul(f[0], c);
            bau.b.ins().fdiv(m, pi)
        }
        "rad" => {
            let c = bau.b.ins().f64const(180.0);
            let pi = bau.b.ins().f64const(std::f64::consts::PI);
            let m = bau.b.ins().fmul(f[0], pi);
            bau.b.ins().fdiv(m, c)
        }
        "frac" => { let t = bau.b.ins().trunc(f[0]); bau.b.ins().fsub(f[0], t) }
        "atan2" | "hypot" => {
            let c = bau.iconst(if name == "atan2" { 0 } else { 1 });
            let r = bau.b.ins().call(bau.h_mathe2, &[c, f[0], f[1]]);
            bau.b.inst_results(r)[0]
        }
        "lerp" => {
            let d = bau.b.ins().fsub(f[1], f[0]);
            let m = bau.b.ins().fmul(d, f[2]);
            bau.b.ins().fadd(f[0], m)
        }
        "min" | "max" => {
            // Wie builtins.rs: der erste gewinnt bei Gleichstand, verglichen
            // wird ueber f64, geliefert wird der WERT.
            let cc = if name == "min" { FloatCC::LessThan } else { FloatCC::GreaterThan };
            let (mut best, mut bf) = (args[0].0, f[0]);
            for i in 1..args.len() {
                let take = bau.b.ins().fcmp(cc, f[i], bf);
                best = bau.b.ins().select(take, args[i].0, best);
                bf = bau.b.ins().select(take, f[i], bf);
            }
            best
        }
        "clamp" => {
            let u = bau.b.ins().fcmp(FloatCC::LessThan, f[0], f[1]);
            let o = bau.b.ins().fcmp(FloatCC::GreaterThan, f[0], f[2]);
            let r = bau.b.ins().select(o, args[2].0, args[0].0);
            bau.b.ins().select(u, args[1].0, r)
        }
        _ => unreachable!(),
    };
    (w, art)
}

fn rechnen(bau: &mut Bauer, o: u16, x: CWert, xa: Art, y: CWert, ya: Art) -> (CWert, Art) {
    if xa == Art::I && ya == Art::I && o != op::DIV {
        let r = match o {
            op::ADD => { let (r, u) = bau.b.ins().sadd_overflow(x, y); bau.aussteigen_wenn(u); r }
            op::SUB => { let (r, u) = bau.b.ins().ssub_overflow(x, y); bau.aussteigen_wenn(u); r }
            op::MUL => { let (r, u) = bau.b.ins().smul_overflow(x, y); bau.aussteigen_wenn(u); r }
            _ => {
                // MOD und \: durch 0 und MIN / -1 steigen aus.
                let null = bau.b.ins().icmp_imm_s(IntCC::Equal, y, 0);
                bau.aussteigen_wenn(null);
                let a = bau.b.ins().icmp_imm_s(IntCC::Equal, x, i64::MIN);
                let b = bau.b.ins().icmp_imm_s(IntCC::Equal, y, -1);
                let beide = bau.b.ins().band(a, b);
                bau.aussteigen_wenn(beide);
                if o == op::INT_DIV {
                    bau.b.ins().sdiv(x, y)
                } else {
                    // Vorzeichen folgt dem Teiler (wie Python): -17 MOD 5 = 3.
                    let m = bau.b.ins().srem(x, y);
                    let nz = bau.b.ins().icmp_imm_s(IntCC::NotEqual, m, 0);
                    let mn = bau.b.ins().icmp_imm_s(IntCC::SignedLessThan, m, 0);
                    let yn = bau.b.ins().icmp_imm_s(IntCC::SignedLessThan, y, 0);
                    let versch = bau.b.ins().bxor(mn, yn);
                    let korr = bau.b.ins().band(nz, versch);
                    let my = bau.b.ins().iadd(m, y);
                    bau.b.ins().select(korr, my, m)
                }
            }
        };
        return (r, Art::I);
    }
    let fx = bau.als_f(x, xa);
    let fy = bau.als_f(y, ya);
    let r = match o {
        op::ADD => bau.b.ins().fadd(fx, fy),
        op::SUB => bau.b.ins().fsub(fx, fy),
        op::MUL => bau.b.ins().fmul(fx, fy),
        op::DIV | op::MOD => {
            let z = bau.b.ins().f64const(0.0);
            let null = bau.b.ins().fcmp(FloatCC::Equal, fy, z);
            bau.aussteigen_wenn(null);
            let q = bau.b.ins().fdiv(fx, fy);
            if o == op::DIV { q } else {
                let fl = bau.b.ins().floor(q);
                let p = bau.b.ins().fmul(fy, fl);
                bau.b.ins().fsub(fx, p)
            }
        }
        _ => unreachable!("\\ nur mit zwei INTEGER"),
    };
    (r, Art::F)
}

/// Vergleiche wie `dispatch`/`cmp`/`value_eq`: zwei INTEGER direkt, sonst
/// als Kommazahlen; `<` & Co. mit NaN sind in der VM ein Fehler (Ausstieg),
/// `=` mit NaN ist FALSE.
fn vergleichen(bau: &mut Bauer, o: u16, x: CWert, xa: Art, y: CWert, ya: Art) -> CWert {
    if (xa == Art::I && ya == Art::I) || (xa == Art::B && ya == Art::B) {
        let cc = match o {
            op::LT => IntCC::SignedLessThan, op::GT => IntCC::SignedGreaterThan,
            op::LEQ => IntCC::SignedLessThanOrEqual, op::GEQ => IntCC::SignedGreaterThanOrEqual,
            op::EQ => IntCC::Equal, _ => IntCC::NotEqual,
        };
        return bau.b.ins().icmp(cc, x, y);
    }
    let fx = bau.als_f(x, xa);
    let fy = bau.als_f(y, ya);
    match o {
        op::EQ => bau.b.ins().fcmp(FloatCC::Equal, fx, fy),
        op::NEQ => bau.b.ins().fcmp(FloatCC::NotEqual, fx, fy),
        _ => {
            let nan = bau.b.ins().fcmp(FloatCC::Unordered, fx, fy);
            bau.aussteigen_wenn(nan);
            let cc = match o {
                op::LT => FloatCC::LessThan, op::GT => FloatCC::GreaterThan,
                op::LEQ => FloatCC::LessThanOrEqual, _ => FloatCC::GreaterThanOrEqual,
            };
            bau.b.ins().fcmp(cc, fx, fy)
        }
    }
}

/// Einstieg fuer die VM: (Kontext, Argumente als u64, Platz fuer das Ergebnis).
fn trampolin(modul: &mut JITModule, fctx: &mut FunctionBuilderContext, ziel_id: FuncId,
             params: &[Art], rueck: Option<Art>, nr: u32) -> Result<FuncId, String> {
    let ptr = modul.target_config().pointer_type();
    let mut sig = modul.make_signature();
    for _ in 0..3 { sig.params.push(AbiParam::new(ptr)); }
    let id = modul.declare_function(&format!("einstieg_{}", nr), Linkage::Local, &sig)
        .map_err(|e| format!("{:?}", e))?;
    let tc = modul.target_config();
    let mut ctx = modul.make_context();
    ctx.func.signature = sig;
    ctx.func.name = UserFuncName::user(1, id.as_u32());
    {
        let mut fb = FunctionBuilder::new(&mut ctx.func, fctx);
        let b = fb.create_block();
        fb.append_block_params_for_function_params(b);
        fb.switch_to_block(b);
        let p = fb.block_params(b).to_vec();
        let mut args = vec![p[0]];
        for (i, a) in params.iter().enumerate() {
            args.push(fb.ins().load(cl_typ(*a), MemFlagsData::trusted(), p[1], (i * 8) as i32));
        }
        let fref = modul.declare_func_in_func(ziel_id, fb.func);
        let ruf = fb.ins().call(fref, &args);
        if rueck.is_some() {
            let w = fb.inst_results(ruf)[0];
            fb.ins().store(MemFlagsData::trusted(), w, p[2], 0);
        }
        fb.ins().return_(&[]);
        fb.seal_all_blocks();
        fb.finalize(tc);
    }
    modul.define_function(id, &mut ctx).map_err(|e| format!("Cranelift: {:?}", e))?;
    modul.clear_context(&mut ctx);
    Ok(id)
}

impl Jit {
    /// Uebersetzt jede Funktion des Programms, die rein ist (siehe oben).
    pub fn neu(prog: &Program) -> Result<Jit, String> {
        let builder = JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names())
            .map_err(|e| format!("Cranelift: {:?}", e))?;
        let mut builder = builder;
        builder.symbol("dh_global_holen", global_holen as *const u8);
        builder.symbol("dh_feld_lesen_i", feld_lesen_i as *const u8);
        builder.symbol("dh_feld_lesen_f", feld_lesen_f as *const u8);
        builder.symbol("dh_feld_setzen_i", feld_setzen_i as *const u8);
        builder.symbol("dh_feld_setzen_f", feld_setzen_f as *const u8);
        builder.symbol("dh_element_i", element_i as *const u8);
        builder.symbol("dh_element_f", element_f as *const u8);
        builder.symbol("dh_journal_schluss", journal_schluss as *const u8);
        builder.symbol("dh_mathe1", mathe1 as *const u8);
        builder.symbol("dh_mathe2", mathe2 as *const u8);
        let w_namen: [(&str, *const u8); 16] = [
            ("dh_w_konst", w_konst as *const u8), ("dh_w_kopie", w_kopie as *const u8),
            ("dh_w_frei", w_frei as *const u8), ("dh_w_ablegen_i", w_ablegen_i as *const u8),
            ("dh_w_ablegen_f", w_ablegen_f as *const u8), ("dh_w_zahl_i", w_zahl_i as *const u8),
            ("dh_w_zahl_f", w_zahl_f as *const u8), ("dh_w_speichern", w_speichern as *const u8),
            ("dh_w_op", w_op as *const u8), ("dh_w_wahr", w_wahr as *const u8),
            ("dh_w_builtin_i", w_builtin_i as *const u8), ("dh_w_builtin_f", w_builtin_f as *const u8),
            ("dh_w_index", w_index as *const u8), ("dh_w_setzen", w_setzen as *const u8),
            ("dh_w_methode", w_methode as *const u8), ("dh_w_drucken", w_drucken as *const u8),
        ];
        for (n, f) in w_namen { builder.symbol(n, f); }
        let mut modul = JITModule::new(builder);
        let glob = globale_lesen(prog);
        let (an, _) = auswahl(prog, &glob);
        let ptr = modul.target_config().pointer_type();
        let mut hsig = modul.make_signature();
        hsig.params.push(AbiParam::new(ptr));
        hsig.params.push(AbiParam::new(types::I64));
        let holen_id = modul.declare_function("dh_global_holen", Linkage::Import, &hsig)
            .map_err(|e| format!("{:?}", e))?;
        // Die Helfer fuer Objekte und Werte (siehe `feld_lesen_i` usw.).
        let sig_h = |params: &[Type], rueck: Option<Type>| {
            let mut sg = modul.make_signature();
            for t in params { sg.params.push(AbiParam::new(*t)); }
            if let Some(r) = rueck { sg.returns.push(AbiParam::new(r)); }
            sg
        };
        let i = types::I64;
        let s_lesen_i = sig_h(&[ptr, i, i, i, i], Some(i));
        let s_lesen_f = sig_h(&[ptr, i, i], Some(types::F64));
        let s_setzen_i = sig_h(&[ptr, i, i, i, i], None);
        let s_setzen_f = sig_h(&[ptr, i, i, types::F64], None);
        let s_element_i = sig_h(&[ptr, i, i, i], Some(i));
        let s_element_f = sig_h(&[ptr, i], Some(types::F64));
        let s_journal = sig_h(&[ptr], None);
        let s_mathe1 = sig_h(&[i, types::F64], Some(types::F64));
        let s_mathe2 = sig_h(&[i, types::F64, types::F64], Some(types::F64));
        let f64t = types::F64;
        let w_sigs: Vec<(&str, cranelift_codegen::ir::Signature)> = [
            ("dh_w_konst", vec![ptr, i, i], None),
            ("dh_w_kopie", vec![ptr, i, i], None),
            ("dh_w_frei", vec![ptr, i], None),
            ("dh_w_ablegen_i", vec![ptr, i, i, i], None),
            ("dh_w_ablegen_f", vec![ptr, f64t, i], None),
            ("dh_w_zahl_i", vec![ptr, i, i], Some(i)),
            ("dh_w_zahl_f", vec![ptr, i], Some(f64t)),
            ("dh_w_speichern", vec![ptr, i, i, i, i], None),
            ("dh_w_op", vec![ptr, i, i, i, i, i], Some(i)),
            ("dh_w_wahr", vec![ptr, i], Some(i)),
            ("dh_w_builtin_i", vec![ptr, i, i, i, i], Some(i)),
            ("dh_w_builtin_f", vec![ptr, i, i, i], Some(f64t)),
            ("dh_w_index", vec![ptr, i, i], None),
            ("dh_w_setzen", vec![ptr, i, i], None),
            ("dh_w_methode", vec![ptr, i, i, i], None),
            ("dh_w_drucken", vec![ptr, i, i, i], None),
        ].into_iter().map(|(n, pa, r): (&str, Vec<Type>, Option<Type>)| (n, sig_h(&pa, r))).collect();
        let mut dekl = |name: &str, sg: &cranelift_codegen::ir::Signature| modul.declare_function(name, Linkage::Import, sg)
            .map_err(|e| format!("{:?}", e));
        let mut w_ids = [holen_id; 16];
        for (j, (n, sg)) in w_sigs.iter().enumerate() { w_ids[j] = dekl(n, sg)?; }
        let hilfe = Hilfe {
            holen: holen_id,
            lesen_i: dekl("dh_feld_lesen_i", &s_lesen_i)?,
            lesen_f: dekl("dh_feld_lesen_f", &s_lesen_f)?,
            setzen_i: dekl("dh_feld_setzen_i", &s_setzen_i)?,
            setzen_f: dekl("dh_feld_setzen_f", &s_setzen_f)?,
            element_i: dekl("dh_element_i", &s_element_i)?,
            element_f: dekl("dh_element_f", &s_element_f)?,
            journal_schluss: dekl("dh_journal_schluss", &s_journal)?,
            mathe1: dekl("dh_mathe1", &s_mathe1)?,
            mathe2: dekl("dh_mathe2", &s_mathe2)?,
            w: w_ids,
        };
        // Beruehrte Plaetze je Funktion samt allen, die sie ruft.
        let mut ber: Vec<Vec<usize>> = an.iter().map(|a| a.as_ref().map_or(Vec::new(), |a| a.beruehrt.clone())).collect();
        loop {
            let mut neu = false;
            for i in 0..an.len() {
                let Some(a) = &an[i] else { continue };
                for &g in &a.gerufen {
                    for p in ber[g].clone() {
                        if !ber[i].contains(&p) { ber[i].push(p); neu = true; }
                    }
                }
            }
            if !neu { break; }
        }
        let mut schreibt: Vec<bool> = an.iter().map(|a| a.as_ref().map_or(false, |a| !a.geschrieben.is_empty())).collect();
        loop {
            let mut neu = false;
            for i in 0..an.len() {
                let Some(a) = &an[i] else { continue };
                if !schreibt[i] && a.gerufen.iter().any(|&g| schreibt[g]) { schreibt[i] = true; neu = true; }
            }
            if !neu { break; }
        }
        let mut ids: Vec<Option<FuncId>> = vec![None; an.len()];
        for (i, a) in an.iter().enumerate() {
            if let Some(a) = a {
                let sig = signatur(&modul, &a.params, a.rueck);
                ids[i] = Some(modul.declare_function(&format!("dh_{}", i), Linkage::Local, &sig)
                    .map_err(|e| format!("{:?}", e))?);
            }
        }
        let mut fctx = FunctionBuilderContext::new();
        let alle = alle_funktionen(prog, &glob);
        let mut einstiege: Vec<Option<FuncId>> = vec![None; an.len()];
        for (i, a) in an.iter().enumerate() {
            if let (Some(a), Some(id)) = (a, ids[i]) {
                let bef: Vec<usize> = a.beruehrt.iter().copied()
                    .filter(|p| !a.gerufen.iter().any(|&g| ber[g].contains(p))).collect();
                erzeugen(&mut modul, prog, &glob, alle[i].0, a, &ids, &hilfe, &mut fctx, id, None, &bef, false)
                    .map_err(|e| format!("{}: {}", alle[i].2, e))?;
                einstiege[i] = Some(trampolin(&mut modul, &mut fctx, id, &a.params, a.rueck, i as u32)?);
            }
        }
        modul.finalize_definitions().map_err(|e| format!("Cranelift: {:?}", e))?;
        let mut fns = Vec::with_capacity(an.len());
        for (i, a) in an.into_iter().enumerate() {
            fns.push(match (a, einstiege[i]) {
                (Some(a), Some(eid)) => {
                    let p = modul.get_finalized_function(eid);
                    Some(Uebersetzt {
                        einstieg: unsafe { std::mem::transmute::<*const u8, Einstieg>(p) },
                        params: a.params,
                        rueck: a.rueck,
                        fehlschlaege: std::cell::Cell::new(0),
                        beruehrt: std::mem::take(&mut ber[i]),
                        schreibt_globale: schreibt[i],
                    })
                }
                _ => None,
            });
        }
        let arten = glob.art.iter().map(|a| match a { Some(Art::I) => 1, Some(Art::F) => 2, Some(Art::B) => 3, _ => 0 }).collect();
        let n = prog.n_globals;
        let mtab: HashMap<(usize, usize), usize> = glob.tafel.iter().enumerate()
            .map(|(j, e)| ((e.lage as usize, e.func as usize), prog.functions.len() + j)).collect();
        Ok(Jit { modul: RefCell::new(modul), fctx: RefCell::new(fctx), ids, hilfe, glob,
                 schleifen: RefCell::new(Vec::new()),
                 zahl_schleifen: std::cell::Cell::new(0), zahl_laeufe: std::cell::Cell::new(0),
                 zahl_aussteige: std::cell::Cell::new(0),
                 meldung: RefCell::new(None),
                 fns, arten, mtab,
                 schatten: std::cell::UnsafeCell::new(vec![0; n]),
                 marken: std::cell::UnsafeCell::new(vec![0; n]) })
    }

    /// Ruft Funktion `idx` mit den Argumenten vom Stapel der VM. `None`, wenn
    /// sie nicht uebersetzt ist, ein Argument nicht genau passt (die VM
    /// wandelt oder meldet) oder der Maschinencode aufgab -- dann rechnet die
    /// VM den Aufruf selbst.
    /// Die VM steht an einem Ruecksprung (`ruecksprung`) zum Kopf `kopf` einer
    /// Schleife. Beim ersten Mal wird der Bereich fuer die Arten uebersetzt,
    /// die die Locals JETZT haben; danach laeuft er, wenn sie wieder passen.
    /// `Some(stelle)`: die Schleife lief im Maschinencode bis zu einem
    /// Ausgang, die Locals und Globals sind nachgetragen. `None`: die VM
    /// macht am Kopf weiter wie immer (auch, wenn der Maschinencode aufgab --
    /// dann ist nichts geschehen).
    /// Ist ein Bereich an einem Befehl ausgestiegen, der gescheitert ist?
    /// Dann gibt die VM diese Meldung an seiner Stelle aus.
    pub fn meldung_nehmen(&self) -> Option<String> { self.meldung.borrow_mut().take() }

    pub fn schleife(&self, vm: *mut crate::vm::Vm<'_>, prog: &Program, f: &Func, ruecksprung: usize, kopf: usize, locals: &mut [Value],
                    stapel: &mut Vec<Value>, selbst: Option<&Value>, tiefe: u32, grenze: u32,
                    slots: &[Option<Rc<RefCell<Slot>>>]) -> Option<usize> {
        let marke = &f.code[ruecksprung].schleife;
        if marke.get() == 0 {
            match self.schleife_bauen(prog, f, kopf, locals, selbst, slots) {
                Ok(nr) => marke.set(nr as u32 + 2),
                Err(e) => {
                    // Mit DHRT_JIT_BILANZ: warum eine Schleife in der VM bleibt.
                    if std::env::var_os("DHRT_JIT_BILANZ").is_some() {
                        eprintln!("jit: Schleife an Stelle {} in {} bleibt in der VM: {}", kopf,
                                  if f.name.is_empty() { "(Hauptprogramm)" } else { &f.name }, e);
                    }
                    marke.set(1);
                    return None;
                }
            }
        }
        let nr = marke.get();
        if nr < 2 { return None; }
        let nr = (nr - 2) as usize;
        let mut sch = self.schleifen.borrow_mut();
        let s = &mut sch[nr];
        if s.kopf != kopf || s.fehlschlaege >= MAX_FEHLSCHLAEGE { return None; }
        // Passen die Arten der Locals noch zu denen, fuer die gebaut wurde?
        // Hinter den echten Locals kommen die globalen Plaetze, die der
        // Bereich wie Locals fuehrt (`dyn_glob`).
        if locals.len() != s.n_lokal { return None; }
        let mut dyn_werte: Vec<Value> = Vec::with_capacity(s.dyn_glob.len());
        for &g in &s.dyn_glob { dyn_werte.push(slots.get(g)?.as_ref()?.borrow().value.clone()); }
        let mut lok = vec![0u64; s.start.len()];
        for (i, a) in s.start.iter().enumerate() {
            let Some(a) = a else { continue };
            let v = if i < s.n_lokal { &locals[i] } else { &dyn_werte[i - s.n_lokal] };
            lok[i] = match (a, v) {
                (Art::I, Value::Int(x)) => *x as u64,
                (Art::F, Value::Float(x)) => x.to_bits(),
                (Art::B, Value::Bool(b)) => *b as u64,
                (Art::Feld(_), _) => 0,   // geprueft mit den Feldern unten
                (Art::Obj(k), Value::Instance(rc)) if std::ptr::eq(Rc::as_ptr(&rc.borrow().layout), s.klassen[*k as usize].lage)
                    => Rc::as_ptr(rc) as u64,
                (Art::W, Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Nil) => return None,
                (Art::W, _) => 0,
                _ => return None,
            };
        }
        let selbst_zeiger = match s.selbst {
            None => 0,
            Some(k) => match selbst {
                Some(Value::Instance(rc)) if std::ptr::eq(Rc::as_ptr(&rc.borrow().layout), s.klassen[k as usize].lage) => Rc::as_ptr(rc) as u64,
                _ => return None,
            },
        };
        // Die Felder: holen, pruefen (Art und Dimensionen wie beim Bauen) und
        // beschreiben. `werte` haelt sie am Leben, solange der Bereich laeuft;
        // dort aendert niemand ihre Groesse, also bleiben die Zeiger gueltig.
        let mut werte: Vec<Value> = Vec::with_capacity(s.felder.len());
        let mut besch: Vec<u64> = Vec::new();
        for fi in &s.felder {
            let v = match fi.quelle {
                Quelle::Lokal(i) => locals.get(i)?.clone(),
                Quelle::Global(g) => slots.get(g)?.as_ref()?.borrow().value.clone(),
            };
            if !feld_passt(&v, fi) { return None; }
            match &v {
                Value::Array(a) => {
                    let mut ab = a.borrow_mut();
                    let zeiger = match &mut ab.cells {
                        crate::value::Cells::Int(x) => x.as_mut_ptr() as u64,
                        crate::value::Cells::Float(x) => x.as_mut_ptr() as u64,
                        crate::value::Cells::Val(x) => x.as_ptr() as u64,
                    };
                    besch.push(zeiger);
                    for &d in &ab.dims { besch.push(d as u64); }
                    for &st in &ab.strides { besch.push(st as u64); }
                }
                Value::Tuple(t) => { besch.push(t.as_ptr() as u64); besch.push(t.len() as u64); besch.push(1); }
                _ => return None,
            }
            werte.push(v);
        }
        let mut st_puffer = [0u64; MAX_STAPEL];
        let schatten = unsafe { &mut *self.schatten.get() };
        let marken = unsafe { &mut *self.marken.get() };
        if marken.len() < slots.len() { return None; }
        // Wertemodus: die Werte wandern aus der VM in die Werteplaetze (nicht
        // kopiert -- sonst hielten zwei einen Text, und `s = s + x` haengte nie
        // an Ort und Stelle an). Zurueck an jedem Ausgang, und auch beim
        // Aufgeben vor dem ersten Befehl.
        let mut bereich_journal: Vec<(u64, u64, Value)> = Vec::new();
        let mut arena: Vec<Value> = Vec::new();
        if s.modus_w {
            arena = vec![Value::Nil; s.start.len() + MAX_STAPEL];
            for (i, a) in s.start.iter().enumerate() {
                if *a != Some(Art::W) { continue; }
                arena[i] = if i < s.n_lokal { nimm(&mut locals[i]) } else {
                    let v = nimm(&mut slots[s.dyn_glob[i - s.n_lokal]].as_ref()?.borrow_mut().value);
                    v
                };
            }
        }
        let mut k = Kontext {
            fehler: 0, tiefe: tiefe as u64, grenze: grenze as u64,
            schatten: schatten.as_mut_ptr(), marken: marken.as_mut_ptr(),
            slots: slots.as_ptr(), n_slots: slots.len() as u64, arten: self.arten.as_ptr(),
            stapel: st_puffer.as_mut_ptr(), felder: besch.as_ptr(), selbst: selbst_zeiger,
            werte: arena.as_mut_ptr(),
            vm: vm as *mut crate::vm::Vm<'static>, meldung: None, journal: std::ptr::null_mut(),
            journal_bereich: &mut bereich_journal,
        };
        let aus = unsafe { (s.einstieg)(&mut k, lok.as_mut_ptr()) };
        if aus <= -2 { *self.meldung.borrow_mut() = k.meldung.take(); }
        // >= 0: an einem Ausgang; <= -2: mitten drin ausgestiegen -- beides
        // ist ein gueltiger Stand der VM. -1: aufgegeben, nichts geschehen.
        let ok = (k.fehler == 0 && aus >= 0) || aus <= -2;
        for &g in &s.beruehrt {
            if ok && marken[g] == 2 {
                if let Some(sl) = slots.get(g).and_then(|x| x.as_ref()) {
                    let bits = schatten[g];
                    sl.borrow_mut().value = match self.arten[g] {
                        1 => Value::Int(bits as i64),
                        2 => Value::Float(f64::from_bits(bits)),
                        _ => Value::Bool(bits != 0),
                    };
                }
            }
            marken[g] = 0;
        }
        if !ok {
            // Aufgegeben, nichts geschehen: die Werte gehen zurueck.
            for (i, a) in s.start.iter().enumerate() {
                if *a != Some(Art::W) { continue; }
                let v = nimm(&mut arena[i]);
                if i < s.n_lokal { locals[i] = v; }
                else if let Some(Some(sl)) = slots.get(s.dyn_glob[i - s.n_lokal]) { sl.borrow_mut().value = v; }
            }
            s.fehlschlaege += 1;
            return None;
        }
        let n_gesamt = s.start.len();
        let arena_z: *mut Value = arena.as_mut_ptr();
        // Ein Wert aus Platz i der Werteplaetze (nur Wertemodus).
        let aus_arena = |i: usize| unsafe { nimm(&mut *arena_z.add(i)) };
        let wert = |a: &Art, bits: u64| match a {
            Art::I => Value::Int(bits as i64),
            Art::F => Value::Float(f64::from_bits(bits)),
            Art::B => Value::Bool(bits != 0),
            Art::Feld(nr) => werte[*nr as usize].clone(),
            Art::N => Value::Nil,
            // Ein Objekt, das die VM noch haelt (in einem Local, einem
            // globalen Platz oder einem Feld): noch ein Verweis darauf.
            Art::Obj(_) => unsafe {
                let p = bits as *const RefCell<crate::value::Instance>;
                Rc::increment_strong_count(p);
                Value::Instance(Rc::from_raw(p))
            },
            Art::W => Value::Nil,   // kommt aus den Werteplaetzen, siehe unten
        };
        // Zurueck in die VM: ein echtes Local oder ein globaler Platz.
        let n_lokal = s.n_lokal;
        let dyn_glob = s.dyn_glob.clone();
        let ablegen = |i: usize, v: Value, locals: &mut [Value]| {
            if i < n_lokal { locals[i] = v; }
            else if let Some(Some(sl)) = slots.get(dyn_glob[i - n_lokal]) { sl.borrow_mut().value = v; }
        };
        if aus <= -2 {
            // Mitten drin ausgestiegen: die VM fuehrt den Befehl an `stelle`
            // selbst aus -- mit genau dem Stand davor.
            let a = s.aussteige.get((-2 - aus) as usize)?;
            for (i, art) in a.lokal.iter().enumerate() {
                let Some(art) = art else { continue };
                let v = if *art == Art::W { aus_arena(i) } else { wert(art, lok[i]) };
                ablegen(i, v, locals);
            }
            for (d, art) in a.stapel.iter().enumerate() {
                stapel.push(if *art == Art::W { aus_arena(n_gesamt + d) } else { wert(art, st_puffer[d]) });
            }
            self.zahl_aussteige.set(self.zahl_aussteige.get() + 1);
            return Some(a.stelle);
        }
        let aus = aus as usize;
        let (_, arten) = s.ausgaenge.iter().find(|(x, _)| *x == aus)?;
        self.zahl_laeufe.set(self.zahl_laeufe.get() + 1);
        for (i, a) in arten.iter().enumerate() {
            let Some(a) = a else { continue };
            let v = if *a == Art::W { aus_arena(i) } else { wert(a, lok[i]) };
            ablegen(i, v, locals);
        }
        Some(aus)
    }

    fn schleife_bauen(&self, prog: &Program, f: &Func, kopf: usize, locals: &[Value], selbst: Option<&Value>,
                      slots: &[Option<Rc<RefCell<Slot>>>]) -> Result<usize, String> {
        let code = &f.code;
        // Das Ende: der letzte Ruecksprung auf diesen Kopf.
        let mut bis = None;
        for (i, ins) in code.iter().enumerate().skip(kopf) {
            let t = match ins.op {
                op::JUMP | op::JUMP_IF_FALSE | op::JUMP_IF_TRUE => ziel(&ins.arg),
                op::FOR_NEXT => for_teile(&ins.arg).map_or(usize::MAX, |t| t[6] as usize),
                _ => continue,
            };
            if t == kopf { bis = Some(i); }
        }
        let bis = bis.ok_or("kein Ruecksprung")?;
        // Nur der Kopf darf von aussen angesprungen werden.
        for (i, ins) in code.iter().enumerate() {
            if i >= kopf && i <= bis { continue; }
            let t = match ins.op {
                op::JUMP | op::JUMP_IF_FALSE | op::JUMP_IF_TRUE | op::TRY_BEGIN => ziel(&ins.arg),
                op::FOR_NEXT => for_teile(&ins.arg).map_or(usize::MAX, |t| t[6] as usize),
                _ => continue,
            };
            if t > kopf && t <= bis { return Err("Sprung mitten in die Schleife".into()); }
        }
        let mut felder: Vec<FeldInfo> = Vec::new();
        let mut klassen: Vec<Klasse> = Vec::new();
        let mut lokal: Vec<Option<Art>> = Vec::with_capacity(locals.len());
        for (i, v) in locals.iter().enumerate() {
            lokal.push(match feld_beschreiben(v, &mut klassen) {
                Some((elem, dims, tupel)) => {
                    felder.push(FeldInfo { quelle: Quelle::Lokal(i), elem, dims, tupel });
                    Some(Art::Feld((felder.len() - 1) as u16))
                }
                None => wert_art(v, &mut klassen),
            });
        }
        let globale_da = slots.iter().map(|x| x.is_some()).collect();
        let glob_felder: Vec<Option<(Elem, u8, bool)>> = slots.iter()
            .map(|x| x.as_ref().and_then(|sl| feld_beschreiben(&sl.borrow().value, &mut klassen))).collect();
        // Globale Plaetze ohne festen Zahlentyp (ein Objekt, eine Zahl in
        // einer Variable ohne Typ wie der Laufvariable von FOR EACH, oder noch
        // NIL) fuehrt der Bereich wie Locals: ihre Art folgt dem, was
        // hineinkommt.
        let mut dyn_glob: Vec<usize> = Vec::new();
        let mut lok_typen: Vec<String> = f.local_types.clone();
        let mut benutzt_selbst = false;
        for ins in &code[kopf..=bis] {
            match ins.op {
                op::LOAD_SELF | op::LOAD_FIELD | op::STORE_FIELD => benutzt_selbst = true,
                op::LOAD_GLOBAL_SLOT | op::STORE_GLOBAL_SLOT | op::ADD_STORE_GLOBAL_SLOT => {
                    let g = ins.arg.as_usize();
                    if self.glob.art.get(g).copied().flatten().is_some() || dyn_glob.contains(&g) { continue; }
                    if glob_felder.get(g).copied().flatten().is_some() { continue; }
                    let Some(Some(sl)) = slots.get(g) else { continue };
                    let v = sl.borrow().value.clone();
                    if !matches!(v, Value::Instance(_) | Value::Nil | Value::Int(_) | Value::Float(_) | Value::Bool(_)) { continue; }
                    dyn_glob.push(g);
                    lok_typen.push(self.glob.typ.get(g).cloned().unwrap_or_default());
                    lokal.push(wert_art(&v, &mut klassen));
                }
                _ => {}
            }
        }
        let selbst_k = match selbst {
            Some(Value::Instance(rc)) if benutzt_selbst => {
                let b = rc.borrow();
                Some(klasse_nr(&mut klassen, Rc::as_ptr(&b.layout), &b.class_name))
            }
            _ => None,
        };
        let start = Zustand { stapel: Vec::new(), lokal };
        let bereich = Bereich { von: kopf, bis, start, globale_da, felder, glob_felder,
                                klassen, selbst: selbst_k, dyn_glob, lok_typen, modus_w: false };
        let an = match analysieren(prog, &self.glob, f, Some(&bereich), None) {
            Ok(an) => an,
            Err(e) => {
                // Zweiter Versuch im Wertemodus: Texte, MAPs, eingebaute
                // Befehle -- alles, was keine Zahl ist, als Wert.
                let w = self.wertebereich(f, kopf, bis, locals, slots);
                match analysieren(prog, &self.glob, f, Some(&w), None) {
                    Ok(an) => an,
                    Err(e2) => return Err(format!("{}; mit Werten: {}", e, e2)),
                }
            }
        };
        if an.gerufen.iter().any(|&g| self.ids.get(g).copied().flatten().is_none()) {
            return Err("ruft eine Funktion, die in der VM bleibt".into());
        }
        // Mitten drin aussteigen geht nur, wenn kein Gerufener globale Plaetze
        // schreibt: dessen halbe Arbeit laege sonst schon im Schatten, und die
        // VM riefe ihn noch einmal. Ohne das darf der Bereich keine Felder
        // schreiben -- aufgeben hiesse dann, die VM wiederholt Geschriebenes.
        let mitten = an.gerufen.iter().all(|&g| self.fns.get(g).and_then(|u| u.as_ref()).map_or(false, |u| !u.schreibt_globale));
        if (an.schreibt_felder || an.modus_w) && !mitten {
            return Err("schreibt in ein Feld (oder rechnet mit Werten) und ruft eine Funktion, die globale Variablen schreibt".into());
        }
        let mut beruehrt = an.beruehrt.clone();
        for &g in &an.gerufen {
            if let Some(Some(u)) = self.fns.get(g) {
                for &p in &u.beruehrt { if !beruehrt.contains(&p) { beruehrt.push(p); } }
            }
        }
        let mut modul = self.modul.borrow_mut();
        let mut fctx = self.fctx.borrow_mut();
        let nr = self.schleifen.borrow().len();
        let sig = bereich_signatur(&modul);
        let id = modul.declare_function(&format!("schleife_{}", nr), Linkage::Local, &sig)
            .map_err(|e| format!("{:?}", e))?;
        let bef: Vec<usize> = an.beruehrt.iter().copied()
            .filter(|p| !an.gerufen.iter().any(|&g| self.fns.get(g).and_then(|u| u.as_ref()).map_or(true, |u| u.beruehrt.contains(p)))).collect();
        let aussteige = erzeugen(&mut modul, prog, &self.glob, f, &an, &self.ids, &self.hilfe, &mut fctx, id, Some((kopf, bis)), &bef, mitten)?;
        modul.finalize_definitions().map_err(|e| format!("Cranelift: {:?}", e))?;
        let p = modul.get_finalized_function(id);
        let ausgaenge = an.ausgaenge.iter().map(|&x| (x, an.vor[x].as_ref().unwrap().lokal.clone())).collect();
        self.schleifen.borrow_mut().push(Schleife {
            einstieg: unsafe { std::mem::transmute::<*const u8, unsafe extern "C" fn(*mut Kontext, *mut u64) -> i64>(p) },
            kopf,
            // Der Zustand am Kopf aus der Analyse, die gilt (im Wertemodus
            // ist es nicht der des ersten Versuchs).
            start: an.vor[kopf].as_ref().map_or_else(Vec::new, |z| z.lokal.clone()),
            ausgaenge,
            beruehrt,
            fehlschlaege: 0,
            felder: an.felder.clone(),
            aussteige,
            klassen: an.klassen.clone(),
            selbst: an.selbst,
            dyn_glob: an.dyn_glob.clone(),
            n_lokal: f.local_types.len(),
            modus_w: an.modus_w,
            typen: an.typen,
        });
        self.zahl_schleifen.set(self.zahl_schleifen.get() + 1);
        Ok(nr)
    }

    /// Der Bereich im Wertemodus: Zahlen bleiben Zahlen, alles andere (ausser
    /// NIL) ist ein Wert; jeder globale Platz ohne festen Zahlentyp, den die
    /// Schleife benutzt, wird wie ein Local gefuehrt.
    fn wertebereich(&self, f: &Func, kopf: usize, bis: usize, locals: &[Value],
                    slots: &[Option<Rc<RefCell<Slot>>>]) -> Bereich {
        let art = |v: &Value| match v {
            Value::Nil => None,
            Value::Int(_) => Some(Art::I),
            Value::Float(_) => Some(Art::F),
            Value::Bool(_) => Some(Art::B),
            _ => Some(Art::W),
        };
        let mut lokal: Vec<Option<Art>> = locals.iter().map(art).collect();
        let mut dyn_glob: Vec<usize> = Vec::new();
        let mut lok_typen: Vec<String> = f.local_types.clone();
        for ins in &f.code[kopf..=bis] {
            if !matches!(ins.op, op::LOAD_GLOBAL_SLOT | op::STORE_GLOBAL_SLOT | op::ADD_STORE_GLOBAL_SLOT) { continue; }
            let g = ins.arg.as_usize();
            if self.glob.art.get(g).copied().flatten().is_some() || dyn_glob.contains(&g) { continue; }
            let Some(Some(sl)) = slots.get(g) else { continue };
            dyn_glob.push(g);
            lok_typen.push(self.glob.typ.get(g).cloned().unwrap_or_default());
            lokal.push(art(&sl.borrow().value));
        }
        Bereich {
            von: kopf, bis,
            start: Zustand { stapel: Vec::new(), lokal },
            globale_da: slots.iter().map(|x| x.is_some()).collect(),
            felder: Vec::new(),
            glob_felder: vec![None; slots.len()],
            klassen: Vec::new(),
            selbst: None,
            dyn_glob,
            lok_typen,
            modus_w: true,
        }
    }

    pub fn rufen(&self, idx: usize, args: &[Value], tiefe: u32, grenze: u32,
                 slots: &[Option<Rc<RefCell<Slot>>>]) -> Option<Value> {
        self.rufen_mit(idx, 0, args, tiefe, grenze, slots)
    }

    /// Eine Methode als Maschinencode (M4 Schritt 7b): nur, wenn sie fuer
    /// GENAU die Klasse des Objekts uebersetzt ist (`lage`) -- eine
    /// Unterklasse hat eine eigene Fassung oder bleibt in der VM.
    pub fn methode(&self, lage: *const crate::value::Layout, m: *const Func, obj: *const RefCell<crate::value::Instance>,
                   args: &[Value], tiefe: u32, grenze: u32, slots: &[Option<Rc<RefCell<Slot>>>]) -> Option<Value> {
        let idx = *self.mtab.get(&(lage as usize, m as usize))?;
        self.rufen_mit(idx, obj as u64, args, tiefe, grenze, slots)
    }

    fn rufen_mit(&self, idx: usize, selbst: u64, args: &[Value], tiefe: u32, grenze: u32,
                 slots: &[Option<Rc<RefCell<Slot>>>]) -> Option<Value> {
        let u = self.fns.get(idx)?.as_ref()?;
        if u.fehlschlaege.get() >= MAX_FEHLSCHLAEGE || args.len() != u.params.len() { return None; }
        let mut roh = [0u64; 16];
        for (i, (p, v)) in u.params.iter().zip(args).enumerate() {
            roh[i] = match (p, v) {
                (Art::I, Value::Int(x)) => *x as u64,
                (Art::F, Value::Float(x)) => x.to_bits(),
                (Art::F, Value::Int(x)) => (*x as f64).to_bits(),
                (Art::B, Value::Bool(b)) => *b as u64,
                _ => return None,
            };
        }
        // Schatten und Marken gehoeren dem Jit; waehrend des Aufrufs benutzt
        // sie nur der Maschinencode, und der ruft nichts zurueck in die VM.
        let schatten = unsafe { &mut *self.schatten.get() };
        let marken = unsafe { &mut *self.marken.get() };
        if marken.len() < slots.len() { return None; }
        let mut k = Kontext {
            fehler: 0, tiefe: tiefe as u64, grenze: grenze as u64,
            schatten: schatten.as_mut_ptr(), marken: marken.as_mut_ptr(),
            slots: slots.as_ptr(), n_slots: slots.len() as u64, arten: self.arten.as_ptr(),
            stapel: std::ptr::null_mut(), felder: std::ptr::null(), selbst, werte: std::ptr::null_mut(),
            vm: std::ptr::null_mut(), meldung: None, journal: std::ptr::null_mut(), journal_bereich: std::ptr::null_mut(),
        };
        let mut journal: Vec<(u64, u64, Value)> = Vec::new();
        k.journal = &mut journal;
        let mut erg = 0u64;
        unsafe { (u.einstieg)(&mut k, roh.as_ptr(), &mut erg) };
        let ok = k.fehler == 0;
        if !ok { journal_zurueck(&mut journal); }
        // Nur ohne Fehler kommen die geaenderten Plaetze in die VM; die
        // Marken werden in jedem Fall wieder frei.
        for &g in &u.beruehrt {
            if ok && marken[g] == 2 {
                if let Some(sl) = slots.get(g).and_then(|s| s.as_ref()) {
                    let bits = schatten[g];
                    sl.borrow_mut().value = match self.arten[g] {
                        1 => Value::Int(bits as i64),
                        2 => Value::Float(f64::from_bits(bits)),
                        _ => Value::Bool(bits != 0),
                    };
                }
            }
            marken[g] = 0;
        }
        if !ok {
            u.fehlschlaege.set(u.fehlschlaege.get() + 1);
            return None;
        }
        Some(match u.rueck {
            Some(Art::I) => Value::Int(erg as i64),
            Some(Art::F) => Value::Float(f64::from_bits(erg)),
            Some(Art::B) => Value::Bool(erg != 0),
            _ => Value::Nil,
        })
    }

}

/// `DHRT_JIT_BILANZ=1`: am Ende auf stderr, wie viele Schleifen uebersetzt
/// wurden und wie oft sie liefen -- fuer Pruefungen, die sehen wollen, DASS
/// der Maschinencode lief.
impl Drop for Jit {
    fn drop(&mut self) {
        if std::env::var_os("DHRT_JIT_BILANZ").is_some() {
            let f = self.fns.iter().filter(|f| f.is_some()).count();
            eprintln!("jit: {} Funktionen, {} Schleifen, {} Schleifenlaeufe",
                      f, self.zahl_schleifen.get(), self.zahl_laeufe.get());
            if self.zahl_aussteige.get() > 0 {
                eprintln!("jit: {} mal mitten in einer Schleife ausgestiegen", self.zahl_aussteige.get());
            }
        }
    }
}
