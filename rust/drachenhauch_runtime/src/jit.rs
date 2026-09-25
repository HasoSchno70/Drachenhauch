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
//! die prueft `tests/pruef/jit.dhtest` samt der ganzen Sammlung unter
//! `DHRT_JIT=immer`.

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
use crate::value::Value;
use crate::vm::Slot;

/// Die Art eines Wertes im Maschinencode. `N` ist das NIL, das der Aufruf
/// einer SUB auf den Stapel legt -- es darf nur wieder weggeworfen werden.
/// `Feld(nr)` ist ein Feld von INTEGER oder FLOAT, das eine Schleife beim
/// Eintritt vorfindet (nur in Bereichen, siehe `FeldInfo`); der Wert im
/// Maschinencode ist bedeutungslos, die Daten beschreibt der Kontext.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Art { I, F, B, N, Feld(u16), Obj(u16) }

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
extern "C" fn feld_setzen_i(obj: u64, platz: u64, art: u64, w: i64) {
    if let Ok(mut b) = objekt(obj).try_borrow_mut() {
        if let Some(f) = b.fields.get_mut(platz as usize) {
            *f = if art == 3 { Value::Bool(w != 0) } else { Value::Int(w) };
        }
    }
}

extern "C" fn feld_setzen_f(obj: u64, platz: u64, w: f64) {
    if let Ok(mut b) = objekt(obj).try_borrow_mut() {
        if let Some(f) = b.fields.get_mut(platz as usize) { *f = Value::Float(w); }
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
}

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
}

/// Was der Maschinencode ueber die globalen Plaetze wissen muss: ihre Art
/// (aus den DECLARE-Befehlen des Hauptprogramms; widersprechen sich zwei,
/// bleibt der Platz der VM) und ob sie Konstanten sind.
struct Globale {
    art: Vec<Option<Art>>,
    konst: Vec<bool>,
    /// Deklarierter Typ je Platz (fuer Objekte in `dyn_glob`).
    typ: Vec<String>,
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
    Globale { art: art.into_iter().map(|a| a.flatten()).collect(), konst, typ }
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

fn analysieren(prog: &Program, glob: &Globale, f: &Func, bereich: Option<&Bereich>) -> Result<Analyse, String> {
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
    let selbst = bereich.and_then(|b| b.selbst);
    let dyn_glob: Vec<usize> = bereich.map_or(Vec::new(), |b| b.dyn_glob.clone());
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
                    None => return Err(format!("Konstante {} ist keine Zahl", c.type_name())),
                }
            }
            op::LOAD_LOCAL if matches!(z.lokal.get(s_arg).copied().flatten(), Some(Art::Feld(_) | Art::Obj(_))) => {
                z.stapel.push(z.lokal[s_arg].unwrap());
            }
            op::LOAD_SELF if selbst.is_some() => { z.stapel.push(Art::Obj(selbst.unwrap())); }
            op::LOAD_FIELD | op::LOAD_MEMBER | op::STORE_FIELD | op::STORE_MEMBER if bereich.is_some() => {
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
                    if !zahl(a) || !zahl(x) { return Err("+ mit einem Nicht-Zahl-Wert".into()); }
                    a = if a == Art::I && x == Art::I { Art::I } else { Art::F };
                }
                if !passt(a, d) { return Err(format!("{:?} in eine globale {:?}", a, d)); }
                if !beruehrt.contains(&g) { beruehrt.push(g); }
                if !geschrieben.contains(&g) { geschrieben.push(g); }
            }
            op::STORE_LOCAL | op::ADD_STORE_LOCAL => {
                let s = s_arg;
                let mut a = pop!();
                if o == op::ADD_STORE_LOCAL {
                    let x = pop!();
                    if !zahl(a) || !zahl(x) { return Err("+ mit einem Nicht-Zahl-Wert".into()); }
                    a = if a == Art::I && x == Art::I { Art::I } else { Art::F };
                }
                if a == Art::N { return Err("NIL gespeichert".into()); }
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
            op::NOT => { let a = pop!(); if !skalar(a) { return Err("NOT auf NIL oder ein Feld".into()); } z.stapel.push(Art::B); }
            op::JUMP => { melden(&mut vor, &mut offen, ziel(&ins.arg), &z)?; weiter = false; }
            op::JUMP_IF_FALSE | op::JUMP_IF_TRUE => {
                let a = pop!();
                if !skalar(a) { return Err("Sprung auf NIL oder ein Feld".into()); }
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
                 klassen, selbst, dyn_glob, typen })
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
    for f in &prog.functions {
        match analysieren(prog, glob, f, None) {
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
            Some((i, g)) => { an[i] = None; gruende[i] = format!("ruft {}, das in der VM bleibt", prog.functions[g].name); }
            None => break,
        }
    }
    (an, gruende)
}

/// Fuer `dhrt --jit datei.dh`: je Funktion "uebersetzt" oder der Grund.
pub fn bericht(prog: &Program) -> Vec<(String, String)> {
    let (_, gruende) = auswahl(prog, &globale_lesen(prog));
    prog.functions.iter().zip(gruende).map(|(f, g)| {
        (f.name.clone(), if g.is_empty() { "uebersetzt".to_string() } else { format!("VM: {}", g) })
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
                    if matches!(a, Art::Feld(_)) { continue; }
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
            self.b.ins().call(self.h_setzen_f, &[obj, pl, w]);
        } else {
            let c = self.b.ins().iconst(types::I64, if a == Art::B { 3 } else { 1 });
            self.b.ins().call(self.h_setzen_i, &[obj, pl, c, w]);
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
        let selbst = if an.selbst.is_some() { Some(fb.ins().load(types::I64, MemFlagsData::trusted(), ctxp, K_SELBST)) } else { None };
        let mut bau = Bauer { b: &mut fb, ctx: ctxp, fehler, vars: HashMap::new(), schatten, marken, holen,
                              befoerdert: HashMap::new(), felder: Vec::new(), mitten, punkt: None,
                              aussteige: Vec::new(), lok_zeiger, zu_tief: false, selbst,
                              h_lesen_i, h_lesen_f, h_setzen_i, h_setzen_f, h_element_i, h_element_f };
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
                match o {
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
                    if matches!(a, Art::Feld(_)) { continue; }
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
        let s_setzen_i = sig_h(&[i, i, i, i], None);
        let s_setzen_f = sig_h(&[i, i, types::F64], None);
        let s_element_i = sig_h(&[ptr, i, i, i], Some(i));
        let s_element_f = sig_h(&[ptr, i], Some(types::F64));
        let mut dekl = |name: &str, sg: &cranelift_codegen::ir::Signature| modul.declare_function(name, Linkage::Import, sg)
            .map_err(|e| format!("{:?}", e));
        let hilfe = Hilfe {
            holen: holen_id,
            lesen_i: dekl("dh_feld_lesen_i", &s_lesen_i)?,
            lesen_f: dekl("dh_feld_lesen_f", &s_lesen_f)?,
            setzen_i: dekl("dh_feld_setzen_i", &s_setzen_i)?,
            setzen_f: dekl("dh_feld_setzen_f", &s_setzen_f)?,
            element_i: dekl("dh_element_i", &s_element_i)?,
            element_f: dekl("dh_element_f", &s_element_f)?,
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
        let mut einstiege: Vec<Option<FuncId>> = vec![None; an.len()];
        for (i, a) in an.iter().enumerate() {
            if let (Some(a), Some(id)) = (a, ids[i]) {
                let bef: Vec<usize> = a.beruehrt.iter().copied()
                    .filter(|p| !a.gerufen.iter().any(|&g| ber[g].contains(p))).collect();
                erzeugen(&mut modul, prog, &glob, &prog.functions[i], a, &ids, &hilfe, &mut fctx, id, None, &bef, false)
                    .map_err(|e| format!("{}: {}", prog.functions[i].name, e))?;
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
        Ok(Jit { modul: RefCell::new(modul), fctx: RefCell::new(fctx), ids, hilfe, glob,
                 schleifen: RefCell::new(Vec::new()),
                 zahl_schleifen: std::cell::Cell::new(0), zahl_laeufe: std::cell::Cell::new(0),
                 zahl_aussteige: std::cell::Cell::new(0),
                 fns, arten,
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
    pub fn schleife(&self, prog: &Program, f: &Func, ruecksprung: usize, kopf: usize, locals: &mut [Value],
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
        let mut k = Kontext {
            fehler: 0, tiefe: tiefe as u64, grenze: grenze as u64,
            schatten: schatten.as_mut_ptr(), marken: marken.as_mut_ptr(),
            slots: slots.as_ptr(), n_slots: slots.len() as u64, arten: self.arten.as_ptr(),
            stapel: st_puffer.as_mut_ptr(), felder: besch.as_ptr(), selbst: selbst_zeiger,
        };
        let aus = unsafe { (s.einstieg)(&mut k, lok.as_mut_ptr()) };
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
        if !ok { s.fehlschlaege += 1; return None; }
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
                ablegen(i, wert(art, lok[i]), locals);
            }
            for (d, art) in a.stapel.iter().enumerate() { stapel.push(wert(art, st_puffer[d])); }
            self.zahl_aussteige.set(self.zahl_aussteige.get() + 1);
            return Some(a.stelle);
        }
        let aus = aus as usize;
        let (_, arten) = s.ausgaenge.iter().find(|(x, _)| *x == aus)?;
        self.zahl_laeufe.set(self.zahl_laeufe.get() + 1);
        for (i, a) in arten.iter().enumerate() {
            let Some(a) = a else { continue };
            ablegen(i, wert(a, lok[i]), locals);
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
                                klassen, selbst: selbst_k, dyn_glob, lok_typen };
        let an = analysieren(prog, &self.glob, f, Some(&bereich))?;
        if an.gerufen.iter().any(|&g| self.ids.get(g).copied().flatten().is_none()) {
            return Err("ruft eine Funktion, die in der VM bleibt".into());
        }
        // Mitten drin aussteigen geht nur, wenn kein Gerufener globale Plaetze
        // schreibt: dessen halbe Arbeit laege sonst schon im Schatten, und die
        // VM riefe ihn noch einmal. Ohne das darf der Bereich keine Felder
        // schreiben -- aufgeben hiesse dann, die VM wiederholt Geschriebenes.
        let mitten = an.gerufen.iter().all(|&g| self.fns.get(g).and_then(|u| u.as_ref()).map_or(false, |u| !u.schreibt_globale));
        if an.schreibt_felder && !mitten {
            return Err("schreibt in ein Feld und ruft eine Funktion, die globale Variablen schreibt".into());
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
            start: bereich.start.lokal.clone(),
            ausgaenge,
            beruehrt,
            fehlschlaege: 0,
            felder: an.felder.clone(),
            aussteige,
            klassen: an.klassen.clone(),
            selbst: an.selbst,
            dyn_glob: an.dyn_glob.clone(),
            n_lokal: f.local_types.len(),
        });
        self.zahl_schleifen.set(self.zahl_schleifen.get() + 1);
        Ok(nr)
    }

    pub fn rufen(&self, idx: usize, args: &[Value], tiefe: u32, grenze: u32,
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
            stapel: std::ptr::null_mut(), felder: std::ptr::null(), selbst: 0,
        };
        let mut erg = 0u64;
        unsafe { (u.einstieg)(&mut k, roh.as_ptr(), &mut erg) };
        let ok = k.fehler == 0;
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
