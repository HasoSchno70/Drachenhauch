//! VM-Kern (Schritt 2+3): stack-basierte Dispatch-Schleife.
//!
//! Skalar-Ops, Control-Flow, User-Calls, Strings/Arrays/Maps/Tupel, OOP
//! (Structs/Klassen/Methoden/Properties/Operator-Overloading), Slicing, IN,
//! DATA/READ, TRY/THROW und die puren Builtins (siehe builtins.rs).
//! Semantik 1:1 aus `drachenhauch/vm.py`, damit `stdout` bit-identisch bleibt.

/// Einen Wert fuer einen Platz vom Typ `$ty` passend machen (STORE_LOCAL,
/// STORE_GLOBAL_SLOT und die ADD_STORE_*-Befehle). Fast-Arm: Wert hat schon
/// den Zieltyp -> kein coerce-Aufruf. Ein Makro und keine Funktion, weil es in
/// der heissesten Schleife steht: als Funktion wurde es nicht eingebettet, und
/// `z = z + i` wurde gemessen 25 % langsamer.
macro_rules! passend {
    ($v:expr, $ty:expr, $wo:expr) => {{
        let v = $v;
        let ty: &str = $ty;
        match (&v, ty) {
            (Value::Int(_), "integer") | (Value::Float(_), "float")
            | (Value::Str(_), "string") | (Value::Bool(_), "boolean")
            | (_, "any") | (_, "") => v,
            (Value::Int(n), "float") => Value::Float(*n as f64),
            _ => coerce(v, ty, $wo)?,
        }
    }};
}

/// Den Anfang einer Meldung von `coerce` (`wo`, etwa "Lokale Variable")
/// durch einen mit Namen ersetzen ("Zuweisung an 'zaehler'"). Nur im
/// Fehlerfall -- vorher sagte die Meldung "Zuweisung an global: Erwartet
/// TIER ...", und welche Variable gemeint war, stand nirgends.
fn mit_namen(e: String, wo: &str, neu: &str) -> String {
    match e.strip_prefix(wo) {
        Some(rest) => format!("{}{}", neu, rest),
        None => e,
    }
}

#[cold]
fn zuweisung_an(name: Option<&String>) -> String {
    match name {
        Some(n) if !n.is_empty() => format!("Zuweisung an '{}'", crate::namensraum::lesbar_text(n)),
        _ => "Zuweisung".to_string(),
    }
}

/// Kennung eines Parameters fuer `coerce` in `bind_params`: ein fester Text,
/// den `Vm::fehler_benennen` spaeter durch den Namen ersetzt. Ein `format!`
/// in `bind_params` machte gemessen tools/tempo/teilchen.dh 9 % langsamer --
/// die Funktion wird in `dispatch` eingebettet, und die Codelage verschob sich.
const PARAM_KENNUNG: [&str; 16] = [
    "Parameter #0", "Parameter #1", "Parameter #2", "Parameter #3",
    "Parameter #4", "Parameter #5", "Parameter #6", "Parameter #7",
    "Parameter #8", "Parameter #9", "Parameter #10", "Parameter #11",
    "Parameter #12", "Parameter #13", "Parameter #14", "Parameter #15",
];

#[inline]
fn param_kennung(i: usize) -> &'static str {
    match PARAM_KENNUNG.get(i) { Some(k) => k, None => "Parameter" }
}

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Zahl der kopierten Locals eines Lambdas aus seinem Namen
/// (`__lambda_<nr>_<kopien>`, vergeben in compiler::expr_lambda).
fn lambda_kopien(name: &str) -> Option<usize> {
    name.strip_prefix("__lambda_")?.rsplit('_').next()?.parse().ok()
}

/// Sentinel-"Fehler", mit dem ein externes Stop-Signal (`dhrt profile`,
/// Editor-Stop-Button) die Dispatch-Schleife sauber abwickelt -- darf wie
/// `__DEBUG_STOP__` NICHT von TRY/CATCH gefangen werden.
const PROFILE_STOP: &str = "__PROFILE_STOP__";

/// Sentinel-"Fehler" fuer `EXIT(code)` -- wickelt die Dispatch-Schleife bis
/// `run()` ab. Wie die beiden Stop-Signale darf er NICHT von TRY/CATCH gefangen
/// werden: `EXIT` heisst "jetzt Schluss", und ein umschliessendes TRY duerfte
/// das nicht in ein CATCH umbiegen. Entscheidend ist auch hier das Flag
/// (`exit_code`), nicht der Text -- `THROW "__EXIT__"` bleibt ein normaler,
/// fangbarer Fehler.
const EXIT_REQUEST: &str = "__EXIT__";

/// Obergrenze fuer verschachtelte `exec`/`exec_byref`-Aufrufe (Rekursionstiefe).
/// `exec` rekursiert ueber den NATIVEN Rust-Stack (exec->run_frame->dispatch->
/// exec), es gibt kein DH-seitiges Frame-Limit -- ohne diese Grenze crasht eine
/// unendliche Rekursion den ganzen Prozess statt einen fangbaren Fehler zu werfen.
///
/// DER WERT HAENGT AN DER STACKGROESSE, nicht am Geschmack. Gemessen kostet ein
/// Aufrufrahmen ~6,6 KB (gleich fuer freie Funktion, Methode, BYREF, FUNCREF und
/// ueberladenen Operator), dazu ~80 KB Grundverbrauch. Die Grenze muss auf JEDER
/// Plattform vor dem Stack greifen, sonst kommt mal diese Meldung und mal ein
/// Absturz. Massgeblich ist die kleinste Plattform: Linux gibt dem Hauptthread
/// per Vorgabe 8 MB (RLIMIT_STACK, von Linker-Flags unerreichbar) -- das traegt
/// ~1250 Ebenen. 1000 laesst davon ein Sechstel Luft.
///
/// Windows und macOS bekommen ihre 64 MB ueber `build.rs`; dort waere
/// weit mehr moeglich, aber ein plattformabhaengiges Limit waere eine Falle:
/// Programme liefen auf dem einen Rechner und braechen auf dem anderen ab.
///
/// Vorher stand hier 3000 -- ein Wert, den keine Plattform je erreichte. Unter
/// Windows (1 MB Vorgabe) crashte der Prozess bereits bei 147.
const MAX_CALL_DEPTH: u32 = 1000;

use crate::builtins;
use crate::model::{op, Arg, ClassInfo, Func, Program};
use crate::value::{as_f64, is_num, value_eq, Cells, CoroState, DhArray, DhMap, Instance, Value};

/// Profiler-Sammler (Stufe B, Phase 3): pro Quell-Zeile Besuchs-Count +
/// kumulierte Zeit. Spiegelt `editor_qt/profiler.py`: die Zeit zwischen zwei
/// Zeilenwechseln wird der VORIGEN Zeile zugeschlagen (inkl. der von dort
/// gerufenen Funktionen). Aggregation pro SUB/FUNCTION macht der Editor via
/// `symbols.scan_scopes` -- dhrt liefert nur die rohen Zeilen-Daten.
struct ProfileSink {
    counts: HashMap<u32, u64>,
    times: HashMap<u32, f64>,
    last_line: u32,
    last_t: std::time::Instant,
    start: std::time::Instant,
}

impl ProfileSink {
    fn new() -> Self {
        let now = std::time::Instant::now();
        ProfileSink { counts: HashMap::new(), times: HashMap::new(),
                      last_line: 0, last_t: now, start: now }
    }
    /// Bei jedem Zeilenwechsel aufrufen (`line` = neue Zeile, != 0).
    fn tick(&mut self, line: u32) {
        let now = std::time::Instant::now();
        if self.last_line != 0 {
            *self.times.entry(self.last_line).or_insert(0.0) +=
                now.duration_since(self.last_t).as_secs_f64();
        }
        *self.counts.entry(line).or_insert(0) += 1;
        self.last_line = line;
        self.last_t = now;
    }
    /// Restzeit der letzten Zeile verbuchen, Gesamtzeit liefern.
    fn finalize(&mut self) -> f64 {
        let now = std::time::Instant::now();
        if self.last_line != 0 {
            *self.times.entry(self.last_line).or_insert(0.0) +=
                now.duration_since(self.last_t).as_secs_f64();
        }
        now.duration_since(self.start).as_secs_f64()
    }
}

/// Ergebnis eines Frame-Laufs: entweder Rueckgabe (Funktion/Coroutine fertig)
/// oder Suspendierung an einem YIELD (nur in Coroutinen).
enum Step {
    Return(Value),
    Yield(Value),
}

/// Bindet Argumente an die Parameter-Slots (mit Variadic + Defaults). Geteilt
/// von normalem Aufruf (exec) und Coroutine-Erststart.
/// Call-Argument zerlegen: bevorzugt das beim Laden gepackte Arg::Call,
/// faellt fuer rohe Listen (z.B. alte .dhc ohne specialize-Pass) zurueck.
#[inline]
/// 1D `ARRAY OF FLOAT`/`INTEGER` -> `Vec<f64>` (Faltungskern, Shader-Arrays).
/// Auf Modul-Ebene, weil mehrere Dispatch-Bloecke (`try_graphics`, imgfx) sie
/// brauchen -- die `gstrs`-Helfer liegen dagegen lokal im Haupt-Match.
fn gfloats(a: &[Value], i: usize, f: &str) -> R<Vec<f64>> {
    match a.get(i) {
        Some(Value::Array(arr)) => {
            let arr = arr.borrow();
            if arr.dims.len() != 1 {
                return Err(format!("{}: erwartet 1D ARRAY OF FLOAT", f));
            }
            arr.cells.iter()
                .map(|v| if is_num(&v) { Ok(as_f64(&v)) }
                         else { Err(format!("{}: ARRAY enthaelt einen Nicht-Zahlenwert", f)) })
                .collect()
        }
        _ => Err(format!("{}: erwartet 1D ARRAY OF FLOAT", f)),
    }
}

fn call_parts(arg: &Arg) -> (&str, usize, i32) {
    match arg {
        Arg::Call(n, c, i) => (n, *c as usize, *i),
        _ => {
            let l = arg.list();
            (l[0].str(), l[1].as_usize(),
             l.get(2).map(|a| a.as_i64() as i32).unwrap_or(-1))
        }
    }
}

/// `args` ist ein Iterator, damit CALL_USER/CALL_METHOD die Argumente direkt
/// vom Stapel des Aufrufers verschieben koennen (`stack.drain(split..)`) --
/// vorher entstand je Aufruf per `split_off` eine eigene Liste.
fn bind_params<I: ExactSizeIterator<Item = Value>>(fn_: &Func, args: I, mut locals: Vec<Value>) -> R<Vec<Value>> {
    // `locals` ist ein (ggf. gepoolter) Buffer -- Allokation wird
    // wiederverwendet, Inhalt kommt frisch aus den local_defaults.
    locals.clear();
    // Der haeufigste Fall (CALL_USER reicht immer alle Argumente, keine
    // Ausdruck-Vorgaben): die Parameter direkt hineinschieben und nur den
    // Rest aus den Vorgaben kopieren -- sonst wird jeder Parameterplatz erst
    // mit seiner Vorgabe belegt und gleich wieder ueberschrieben.
    if !fn_.is_variadic && args.len() == fn_.n_params
        && !fn_.param_default_is_expr.iter().any(|&b| b) {
        for (i, v) in args.enumerate() {
            locals.push(passend!(v, fn_.local_types[i].as_str(), param_kennung(i)));
        }
        let np = fn_.n_params;
        if fn_.local_defaults.len() > np {
            locals.extend_from_slice(&fn_.local_defaults[np..]);
        }
        let n_locals = fn_.local_types.len();
        if locals.len() < n_locals {
            locals.resize(n_locals, Value::Nil);
        }
        return Ok(locals);
    }
    locals.extend_from_slice(&fn_.local_defaults);
    let n_locals = fn_.local_types.len();
    if locals.len() < n_locals {
        locals.resize(n_locals, Value::Nil);
    }
    if fn_.is_variadic {
        let normal_n = fn_.n_params - 1;
        if args.len() < normal_n {
            return Err(format!("{}: erwartet mind. {} Argument(e), erhalten {}",
                fn_.name.to_uppercase(), normal_n, args.len()));
        }
        let mut it = args.into_iter();
        for i in 0..normal_n {
            let v = it.next().unwrap();
            locals[i] = passend!(v, fn_.local_types[i].as_str(), param_kennung(i));
        }
        let rest: Vec<Value> = it.collect();
        locals[fn_.n_params - 1] = Value::Tuple(Rc::new(rest));
    } else {
        // Review-Fund: `n_required == 0` wurde bisher als "nicht gesetzt"
        // gelesen und durch `n_params` ersetzt (= "alle Parameter Pflicht").
        // Das war aber der GUELTIGE Wert genau dann, wenn der ERSTE Parameter
        // schon einen Default hat (make_sig: n_required = Position des ersten
        // Parameters MIT Default) -- eine Methode/Init/FUNCREF, bei der ALLE
        // Parameter Defaults haben, liess sich dadurch nie mit 0 Argumenten
        // aufrufen (ueber NEW/CALL_METHOD/CALL_VALUE; der CALL_USER-Pfad
        // materialisiert immer alle n Argumente zur Compile-Zeit und war
        // deshalb nicht betroffen).
        let n_required = fn_.n_required;
        if args.len() < n_required || args.len() > fn_.n_params {
            // Ein Lambda heisst intern `__lambda_<nr>_<kopien>`; die Kopien
            // zaehlen nach aussen nicht mit, und der Name sagt niemandem etwas.
            if let Some(k) = lambda_kopien(&fn_.name) {
                return Err(format!("Lambda: erwartet {} Argument(e), erhalten {}",
                    fn_.n_params.saturating_sub(k), args.len().saturating_sub(k)));
            }
            return Err(format!("{}: erwartet {}..{} Argument(e), erhalten {}",
                fn_.name.to_uppercase(), n_required, fn_.n_params, args.len()));
        }
        let argn = args.len();
        for (i, v) in args.into_iter().enumerate() {
            // Ausdruck-Default + Nil-Sentinel (nicht uebergeben): NICHT coercen,
            // der Callee-Prolog berechnet den Wert.
            if fn_.param_default_is_expr.get(i).copied().unwrap_or(false)
                && matches!(v, Value::Nil) {
                locals[i] = Value::Nil;
            } else {
                // Schneller Typvergleich wie beim Speichern; `coerce` nur, wenn
                // der Wert nicht schon passt.
                locals[i] = passend!(v, fn_.local_types[i].as_str(), param_kennung(i));
            }
        }
        for i in argn..fn_.n_params {
            if fn_.param_default_is_expr.get(i).copied().unwrap_or(false) {
                locals[i] = Value::Nil;   // Sentinel -> Callee-Prolog
            } else {
                let default = fn_.param_defaults.get(i).cloned().unwrap_or(Value::Nil);
                locals[i] = coerce(default, &fn_.local_types[i], "Default-Parameter")?;
            }
        }
    }
    Ok(locals)
}

/// Baut ein COROUTINE-Handle. Der Aufruf fuehrt den Body NICHT aus -- das
/// passiert erst beim ersten CORO_RESUME. `fn_ptr` ist gueltig fuer die ganze
/// Programmlaufzeit (siehe `CoroState`).
fn make_coro(callee: &Func, args: Vec<Value>, self_obj: Option<Value>) -> Value {
    Value::Coroutine(Rc::new(RefCell::new(CoroState {
        fn_ptr: callee as *const Func,
        self_obj,
        name: callee.name.clone(),
        args,
        started: false,
        done: false,
        running: false,
        result: Value::Nil,
        locals: Vec::new(),
        stack: Vec::new(),
        ip: 0,
        try_handlers: Vec::new(),
    })))
}

fn expect_coro(v: &Value, fname: &str) -> R<Rc<RefCell<CoroState>>> {
    match v {
        Value::Coroutine(rc) => Ok(rc.clone()),
        _ => Err(format!("{}: Erwartet COROUTINE, erhalten {}", fname, v.type_name())),
    }
}

/// Operanden-Stack-Pop mit sauberem Fehler statt Panic. Ein Stack-Underflow
/// kann nur bei kaputtem/abgeschnittenem `.dhc` (oder einem Compiler-Bug)
/// auftreten -- statt eines Rust-Panics liefert die VM dann eine Meldung.
#[inline]
fn vm_pop(stack: &mut Vec<Value>) -> R<Value> {
    stack
        .pop()
        .ok_or_else(|| "VM: Stack underflow (kaputter/inkompatibler Bytecode?)".to_string())
}

/// Wie `vm_pop`, aber peekt das oberste Element (z.B. fuer DUP), ohne zu poppen.
#[inline]
fn vm_top(stack: &[Value]) -> R<&Value> {
    stack
        .last()
        .ok_or_else(|| "VM: Stack underflow (kaputter/inkompatibler Bytecode?)".to_string())
}

// Arg-Helfer fuer die Modul-Dispatcher (db/net/html/serial/usb/wifi/bt).
#[allow(dead_code)]
fn bi_int(a: &[Value], i: usize, fn_: &str) -> R<i64> {
    match a.get(i) {
        Some(Value::Int(n)) => Ok(*n),
        Some(v) => Err(format!("{}: erwartet INTEGER, erhalten {}", fn_, v.type_name())),
        None => Err(format!("{}: fehlendes Argument {}", fn_, i + 1)),
    }
}
#[allow(dead_code)]
fn bi_str<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
    match a.get(i) {
        Some(Value::Str(s)) => Ok(s),
        Some(v) => Err(format!("{}: erwartet STRING, erhalten {}", fn_, v.type_name())),
        None => Err(format!("{}: fehlendes Argument {}", fn_, i + 1)),
    }
}
/// Der Ordner, von dem aus ein CODE_*-Befehl IMPORTs aufloest: das Argument
/// an Stelle `i` -- ohne es None (der Befehl bleibt dann beim Text).
fn code_basis(a: &[Value], i: usize, fn_: &str) -> R<Option<std::path::PathBuf>> {
    if a.len() <= i { return Ok(None); }
    let b = bi_str(a, i, fn_)?;
    Ok(Some(if b.is_empty() { std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")) }
            else { std::path::PathBuf::from(b) }))
}
#[allow(dead_code)]
fn bi_num(a: &[Value], i: usize, fn_: &str) -> R<f64> {
    match a.get(i) {
        Some(Value::Int(n)) => Ok(*n as f64),
        Some(Value::Float(f)) => Ok(*f),
        Some(v) => Err(format!("{}: erwartet Zahl, erhalten {}", fn_, v.type_name())),
        None => Err(format!("{}: fehlendes Argument {}", fn_, i + 1)),
    }
}
#[allow(dead_code)]
fn bi_bool(a: &[Value], i: usize, fn_: &str) -> R<bool> {
    match a.get(i) {
        Some(Value::Bool(b)) => Ok(*b),
        Some(v) => Err(format!("{}: erwartet BOOLEAN, erhalten {}", fn_, v.type_name())),
        None => Err(format!("{}: fehlendes Argument {}", fn_, i + 1)),
    }
}
#[cfg(feature = "db")]
fn db_params(args: &[Value], fn_: &str) -> R<Vec<crate::db::Param>> {
    args.iter().map(|v| crate::db::dh_to_sql(v, fn_)).collect()
}

pub(crate) struct Slot {
    ty: String,
    pub(crate) value: Value,
    is_const: bool,
}

/// Zustand des `input`-Moduls: Action->Keycodes + Edge-Detection-Snapshots.
#[derive(Default)]
struct InputModule {
    actions: HashMap<String, Vec<i64>>,
    prev: HashSet<i64>,
    cur: HashSet<i64>,
}

/// Zustand des Immediate-Mode-`ui`-Moduls (per String-ID + globales Theme).
struct UiState {
    checkbox: HashMap<String, bool>,
    slider: HashMap<String, f64>,
    radio: HashMap<String, i64>,
    text: HashMap<String, String>,
    focused: Option<String>,
    prev_backspace: bool,
    // Immediate-Mode-Fenster: Offset fuer fenster-relative Widgets + Input-Gating.
    offset_x: i32,
    offset_y: i32,
    input_blocked: bool,
    windows: HashMap<String, (i32, i32, bool)>,   // id -> (x, y, collapsed)
    win_stack: Vec<(i32, i32, bool)>,              // gesicherte (off_x, off_y, blocked)
    drag_win: Option<String>,
    drag_off: (i32, i32),
    active_win: Option<String>,                    // Fenster mit Input (Vorframe-Hover)
    hover_win: Option<String>,                     // oberstes Fenster unter Maus (dieser Frame)
    was_mouse_down: bool,
    click_origin: Option<String>,
    frame_count: i64,
    theme: HashMap<String, i64>,
    metrics: HashMap<String, i64>,
    // UI_TABLE (Immediate-Mode): persistenter Scroll-/Selektions-State pro id.
    tables: HashMap<String, UiTable>,
}

#[derive(Default)]
struct UiTable {
    scroll_x: i32, scroll_y: i32,
    drag_v: bool, drag_h: bool, drag_off: i32,
    selected: i32, header_col: i32,
}

const DEFAULT_UI_THEME: &[(&str, i64)] = &[
    ("accent", 0x80C0FF), ("text_fg", 0xFFFFFF), ("muted_fg", 0x707080),
    ("button_bg", 0x40445C), ("panel_bg", 0x252840), ("panel_border", 0x60607A),
    ("panel_title_bg", 0x383C5C), ("field_bg", 0x1A1C2A), ("field_border", 0x808088),
    ("slider_track", 0x404060), ("progress_fg", 0x4CAF50), ("progress_bg", 0x303040),
    ("win_bg", 0x1A1C2A), ("win_border", 0x60607A), ("win_title_bg", 0x383C5C),
    ("win_title_bg_focus", 0x2A5C72),
];
const DEFAULT_UI_METRICS: &[(&str, i64)] = &[
    ("checkbox_size", 14), ("slider_h", 14), ("win_title_h", 20),
    // Plastik wie im Modul `gui`: 0 = flach (bisheriges Aussehen).
    ("corner_radius", 0), ("gradient", 0), ("gloss", 0), ("bevel", 0),
];

/// Die vier Plastik-Werte in einem Stueck.
///
/// Sie muessen VOR `self.gfx.as_mut()` gelesen werden: dieser Aufruf leiht
/// `self` veraenderlich aus, danach ist `self.ui_state` nicht mehr lesbar.
/// Genau deshalb reicht das Modul die Werte gebuendelt an die freie
/// Zeichenroutine weiter, statt dort erneut ins Thema zu greifen.
#[cfg(feature = "graphics")]
#[derive(Clone, Copy, Default)]
struct UiPlastik {
    rad: i32,
    grad: i32,
    gloss: i32,
    bevel: i32,
}

/// Flaeche mit Rand, wahlweise gewoelbt (erhaben) oder versenkt.
///
/// Gegenstueck zu `Gui::flaeche`. Erhaben = hell oben, dunkel unten, mit
/// Glanzkante; versenkt = umgekehrt, mit Schatten unter der Oberkante. Ohne
/// die Plastik-Metriken bleibt es die schlichte gefuellte Box wie bisher.
#[cfg(feature = "graphics")]
#[allow(clippy::too_many_arguments)]
fn ui_flaeche(g: &mut crate::graphics::Graphics, p: UiPlastik, x: i32, y: i32, w: i32, h: i32,
              fill: i64, border: i64, tief: bool) {
    let (x2, y2) = (x + w - 1, y + h - 1);
    if p.grad > 0 {
        let (oben, unten) = if tief {
            (ui_darken(fill, p.grad as i64), ui_lighten(fill, p.grad as f64 / 255.0))
        } else {
            (ui_lighten(fill, p.grad as f64 / 255.0), ui_darken(fill, p.grad as i64))
        };
        g.round_gradient(x, y, x2, y2, p.rad, oben, unten);
        if !tief && p.gloss > 0 && h >= 4 {
            let a = ((p.gloss.clamp(0, 100) as f64 / 100.0) * 255.0) as i64;
            // Runde Formen brauchen den Glanz ueber die GANZE Hoehe -- sonst
            // deckelt round_gradient seinen Radius und die Glanzflaeche ragt
            // seitlich ueber die Form hinaus (derselbe Fall wie im gui-Modul).
            let unten_y = if p.rad * 2 >= h { y2 } else { y + h / 2 };
            g.round_gradient(x, y, x2, unten_y, p.rad, (a << 24) | 0xFFFFFF, 0x01FF_FFFFu32 as i64);
        }
    } else if p.rad > 0 {
        g.round_rect(x, y, x2, y2, p.rad, fill, true);
    } else {
        g.box_fill(x, y, x2, y2, fill);
    }
    if p.bevel > 0 && w >= 4 {
        let ein = p.rad.max(1);
        let (o, u) = if tief {
            (0x60000000u32 as i64, 0x38FFFFFFu32 as i64)
        } else {
            (0x50FFFFFFu32 as i64, 0x40000000u32 as i64)
        };
        g.line(x + ein, y + 1, x2 - ein, y + 1, o);
        g.line(x + ein, y2 - 1, x2 - ein, y2 - 1, u);
    }
    if p.rad > 0 {
        g.round_rect(x, y, x2, y2, p.rad, border, false);
    } else {
        g.rect(x, y, x2, y2, border);
    }
}

impl UiState {
    fn new() -> Self {
        UiState {
            checkbox: HashMap::new(), slider: HashMap::new(), radio: HashMap::new(),
            text: HashMap::new(), focused: None, prev_backspace: false,
            offset_x: 0, offset_y: 0, input_blocked: false,
            windows: HashMap::new(), win_stack: Vec::new(), drag_win: None,
            drag_off: (0, 0), active_win: None, hover_win: None,
            was_mouse_down: false, click_origin: None, frame_count: 0,
            theme: DEFAULT_UI_THEME.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            metrics: DEFAULT_UI_METRICS.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            tables: HashMap::new(),
        }
    }
    fn th(&self, key: &str) -> i64 { *self.theme.get(key).unwrap_or(&0xFFFFFF) }
    fn metric(&self, key: &str) -> i64 { *self.metrics.get(key).unwrap_or(&14) }
    /// Plastik-Werte gebuendelt (siehe `UiPlastik`).
    #[cfg(feature = "graphics")]
    fn plastik(&self) -> UiPlastik {
        UiPlastik {
            rad: *self.metrics.get("corner_radius").unwrap_or(&0) as i32,
            grad: *self.metrics.get("gradient").unwrap_or(&0) as i32,
            gloss: *self.metrics.get("gloss").unwrap_or(&0) as i32,
            bevel: *self.metrics.get("bevel").unwrap_or(&0) as i32,
        }
    }
}

/// Theme-Presets (UI_THEME_PRESET). Nur Farben (keine Metriken).
#[cfg(feature = "graphics")]
fn ui_preset(name: &str) -> Option<Vec<(&'static str, i64)>> {
    let p: Vec<(&str, i64)> = match name {
        "dark" => DEFAULT_UI_THEME.to_vec(),
        "light" => vec![
            ("accent", 0x2A7DE1), ("text_fg", 0x202428), ("muted_fg", 0x90969C),
            ("button_bg", 0xD8DCE2), ("panel_bg", 0xECEFF3), ("panel_border", 0xB0B6BE),
            ("panel_title_bg", 0xD0D5DC), ("field_bg", 0xFFFFFF), ("field_border", 0x9AA0A8),
            ("slider_track", 0xC4C9D0), ("progress_fg", 0x2FA84F), ("progress_bg", 0xCBD0D6),
            ("win_bg", 0xF4F6F9), ("win_border", 0xA8AEB6), ("win_title_bg", 0xD0D5DC),
            ("win_title_bg_focus", 0x2A7DE1),
        ],
        // Plastische Glas-Themen -- Gegenstueck zu den gleichnamigen im Modul
        // `gui`. Verlauf/Glanz/Fase kommen aus den Metriken (siehe
        // ui_preset_metrics), damit ein Thema ein KOMPLETTER Look ist.
        "glas_dunkel" | "glas_dark" => vec![
            ("accent", 0x2FA8D8), ("text_fg", 0xEAF2F8), ("muted_fg", 0x8B97A6),
            ("button_bg", 0x39424F), ("panel_bg", 0x2C343F), ("panel_border", 0x161B22),
            ("panel_title_bg", 0x39424F), ("field_bg", 0x232A33), ("field_border", 0x161B22),
            ("slider_track", 0x232A33), ("progress_fg", 0x2FA8D8), ("progress_bg", 0x232A33),
            ("win_bg", 0x232A33), ("win_border", 0x151A21), ("win_title_bg", 0x2C343F),
            ("win_title_bg_focus", 0x1C6E96),
        ],
        "glas_hell" | "glas_light" => vec![
            ("accent", 0x2A8FD0), ("text_fg", 0x1E2530), ("muted_fg", 0x66707C),
            ("button_bg", 0xFBFCFE), ("panel_bg", 0xE2E8EF), ("panel_border", 0x63707F),
            ("panel_title_bg", 0xD2DAE3), ("field_bg", 0xFDFEFF), ("field_border", 0x63707F),
            ("slider_track", 0xCED6DF), ("progress_fg", 0x2A8FD0), ("progress_bg", 0xD6DDE5),
            ("win_bg", 0xE2E8EF), ("win_border", 0x6E7C8C), ("win_title_bg", 0xD2DAE3),
            ("win_title_bg_focus", 0x2A8FD0),
        ],
        "retro" => vec![
            ("accent", 0x33FF66), ("text_fg", 0x33FF66), ("muted_fg", 0x1F8C3C),
            ("button_bg", 0x0A1A0A), ("panel_bg", 0x041004), ("panel_border", 0x1F8C3C),
            ("panel_title_bg", 0x0A2A0A), ("field_bg", 0x020802), ("field_border", 0x1F8C3C),
            ("slider_track", 0x0A2A0A), ("progress_fg", 0x33FF66), ("progress_bg", 0x0A2A0A),
            ("win_bg", 0x020802), ("win_border", 0x1F8C3C), ("win_title_bg", 0x0A2A0A),
            ("win_title_bg_focus", 0x0F4F1F),
        ],
        "contrast" => vec![
            ("accent", 0xFFD400), ("text_fg", 0xFFFFFF), ("muted_fg", 0xAAAAAA),
            ("button_bg", 0x000000), ("panel_bg", 0x000000), ("panel_border", 0xFFD400),
            ("panel_title_bg", 0x202000), ("field_bg", 0x000000), ("field_border", 0xFFD400),
            ("slider_track", 0x303000), ("progress_fg", 0xFFD400), ("progress_bg", 0x303030),
            ("win_bg", 0x000000), ("win_border", 0xFFD400), ("win_title_bg", 0x202000),
            ("win_title_bg_focus", 0x4F4F00),
        ],
        _ => return None,
    };
    Some(p)
}

/// Metrik-Profil eines Presets. Nur die "glas_*"-Themen bringen Plastik mit;
/// alle anderen bleiben flach, damit bestehende Programme unveraendert
/// aussehen.
fn ui_preset_metrics(name: &str) -> Vec<(&'static str, i64)> {
    if name.starts_with("glas") {
        vec![("corner_radius", 5), ("gradient", 16), ("gloss", 26), ("bevel", 1)]
    } else {
        vec![("corner_radius", 0), ("gradient", 0), ("gloss", 0), ("bevel", 0)]
    }
}

/// Ruft ein reines Builtin auf und faengt einen evtl. Rust-Panic ab (z.B.
/// Index-out-of-bounds bei zu wenigen Argumenten in variadischen Builtins),
/// damit ein Tippfehler im DH-Programm NICHT die Runtime abstuerzen laesst,
/// sondern einen klaren Laufzeitfehler liefert.
/// Zahl der Befehlsfamilien in `Vm::familie_rufen`; die letzte sind die reinen.
pub(crate) const BUILTIN_FAMILIEN: u8 = 34;

fn safe_call_builtin(name: &str, args: &[Value]) -> Option<Result<Value, String>> {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    match catch_unwind(AssertUnwindSafe(|| crate::builtins::call_builtin(name, args))) {
        Ok(r) => r,
        Err(payload) => {
            let msg = payload.downcast_ref::<String>().cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "interner Fehler".to_string());
            let friendly = if msg.contains("index out of bounds") {
                format!("{}: zu wenige Argumente", name.to_uppercase())
            } else {
                format!("{}: interner Fehler ({})", name.to_uppercase(), msg)
            };
            Some(Err(friendly))
        }
    }
}

#[cfg(feature = "graphics")]
fn ui_in_box(mx: i32, my: i32, x: i32, y: i32, w: i32, h: i32) -> bool {
    x <= mx && mx < x + w && y <= my && my < y + h
}
#[cfg(feature = "graphics")]
fn ui_darken(color: i64, d: i64) -> i64 {
    let (r, g, b) = ((color >> 16) & 0xFF, (color >> 8) & 0xFF, color & 0xFF);
    ((r - d).max(0) << 16) | ((g - d).max(0) << 8) | (b - d).max(0)
}
#[cfg(feature = "graphics")]
fn ui_lighten(color: i64, factor: f64) -> i64 {
    let (r, g, b) = (((color >> 16) & 0xFF) as f64, ((color >> 8) & 0xFF) as f64, (color & 0xFF) as f64);
    let r = (r + (255.0 - r) * factor).min(255.0).max(0.0) as i64;
    let g = (g + (255.0 - g) * factor).min(255.0).max(0.0) as i64;
    let b = (b + (255.0 - b) * factor).min(255.0).max(0.0) as i64;
    (r << 16) | (g << 8) | b
}

/// Eine offene Kamera: der Faden, das IMAGE, in das ihre Bilder kommen,
/// und ein Puffer, der zwischen Kamera und Bild hin und her getauscht wird.
#[cfg(feature = "graphics")]
struct WebcamZustand {
    kamera: crate::webcam::Kamera,
    bild: i64,
    mass: (u32, u32),
    px: Vec<u8>,
}

/// Ein geladenes Video: der Decoder, das IMAGE, in das jedes neue Bild geht,
/// und die Uhr. `stand` = Bildzaehler der Grafik beim letzten Weiterstellen,
/// `geschrieben` = Nummer des Bildes, das im IMAGE steht.
#[cfg(all(feature = "video", feature = "graphics"))]
struct VideoZustand {
    video: crate::video::Video,
    bild: i64,
    zeit: f64,
    spielt: bool,
    schleife: bool,
    fertig: bool,
    stand: Option<u64>,
    geschrieben: i64,
}

// --- Debugger (Stufe B, `dhrt debug`) ---------------------------------------
#[derive(PartialEq, Clone, Copy)]
enum StepMode { Run, Over, Into, Out }

struct DebugState {
    // Zeile -> optionale (bereits geparste) Bedingung.
    breakpoints: HashMap<u32, Haltepunkt>,
    step: StepMode,
    step_depth: u32,
    out_sent: usize,   // wie viel von vm.out schon als output-Event gesendet wurde
    // Aufrufstapel: je laufender Funktion ihr Name und die Zeile, in der sie
    // GERUFEN wurde (die aktuelle Zeile der innersten steht in cur_line).
    // Aussen zuerst; der erste Eintrag ist das Hauptprogramm.
    stapel: Vec<(String, u32)>,
    // Ueberwachte Ausdruecke: bei jedem Halt ausgewertet (Quelltext + die
    // geparste Form; None = liess sich nicht lesen).
    watches: Vec<(String, Option<crate::ast::Node>)>,
    // "Bis hier laufen": ein Haltepunkt, der nach dem ersten Treffer vergeht.
    run_to: Option<u32>,
    // Kommandos von stdin, gelesen von einem eigenen Faden (debugger.rs) --
    // so kommen Haltepunkte auch an, waehrend das Programm laeuft.
    kanal: crate::debugger::Kanal,
    // Gemergte Zeile <-> (Datei, Zeile). None = ohne Karte (Zeilen roh).
    karte: Option<crate::debugger::Karte>,
    // Die Fehlerstelle eines frischen Fehlers (Funktion, Locals, Stapel,
    // Zeile) -- ein CATCH wirft sie weg; kommt der Fehler oben an, haelt der
    // Debugger mit diesem Stand an (`debug_fehler_halt`).
    fehler_stand: Option<FehlerStand>,
}

/// Ein Haltepunkt an einer (gemergten) Zeile: Bedingung, Trefferzahl,
/// Protokolltext -- und wie oft er schon getroffen wurde (nur gezaehlt, wenn
/// die Bedingung stimmt).
struct Haltepunkt {
    bed: Option<crate::ast::Node>,
    treffer: Option<(char, u32)>,
    log: Option<String>,
    zaehler: u32,
}

impl Haltepunkt {
    fn schlicht(&self) -> bool { self.bed.is_none() && self.treffer.is_none() && self.log.is_none() }
}

struct FehlerStand {
    // Zeigt in das Programm, das laenger lebt als der Lauf (wie
    // CoroState::fn_ptr); nach dem Laden wird es nicht mehr veraendert.
    fn_: *const Func,
    locals: Vec<Value>,
    stapel: Vec<(String, u32)>,
    zeile: u32,
}

impl DebugState {
    fn new() -> Self {
        // Beim Start an der ersten Zeile anhalten -> der Editor setzt dann
        // Breakpoints und schickt `continue`.
        DebugState { breakpoints: HashMap::new(), step: StepMode::Into,
                     step_depth: 0, out_sent: 0, stapel: Vec::new(),
                     watches: Vec::new(), run_to: None,
                     kanal: crate::debugger::Kanal::neu(), karte: None, fehler_stand: None }
    }

    /// (Datei, Zeile) einer gemergten Zeile -- ohne Karte die Zeile selbst.
    fn stelle(&self, gemergt: u32) -> (String, u32) {
        match &self.karte {
            Some(k) => k.stelle(gemergt),
            None => (String::new(), gemergt),
        }
    }

    /// Wo ein Haltepunkt in (datei, zeile) haelt: (gemergt, zeile in der Datei).
    fn haltestelle(&self, datei: &str, zeile: u32) -> Option<(u32, u32)> {
        match &self.karte {
            Some(k) => k.haltestelle(datei, zeile),
            None => Some((zeile, zeile)),
        }
    }

    fn gehoert_dazu(&self, datei: &str) -> bool {
        self.karte.as_ref().map(|k| k.gehoert_dazu(datei)).unwrap_or(true)
    }

    fn haupt(&self) -> String {
        self.karte.as_ref().map(|k| k.haupt().to_string()).unwrap_or_default()
    }
}

/// `DHRT_LIVE=1`: jede PRINT-Zeile sofort hinausschreiben. Gesetzt von
/// PROCESS_START fuer seine Kinder (prozess.rs); einmal gelesen.
fn live_ausgabe() -> bool {
    static LIVE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIVE.get_or_init(|| std::env::var("DHRT_LIVE").map(|v| v == "1").unwrap_or(false))
}

/// stdout ist ein Terminal: dann geht jede PRINT-Zeile sofort hinaus
/// (zeilenweise wie in C und Python). In eine Datei oder Leitung wird
/// gebuendelt -- dort zaehlt der Durchsatz.
fn ausgabe_terminal() -> bool {
    use std::io::IsTerminal;
    static T: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *T.get_or_init(|| std::io::stdout().is_terminal())
}

/// Ein JSON-Event als Zeile auf stdout schreiben + flushen.
fn dbg_emit(ev: &serde_json::Value) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut h = stdout.lock();
    let _ = writeln!(h, "{}", serde_json::to_string(ev).unwrap_or_default());
    let _ = h.flush();
}

fn dbg_truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Nil => false,
        _ => true,
    }
}

/// Wert fuer die Variablen-Anzeige formatieren (gekappt wie debugger._fmt_value).
fn dbg_short(v: &Value) -> String {
    let s = v.fmt();
    if s.chars().count() > 200 {
        let t: String = s.chars().take(200).collect();
        format!("{}...", t)
    } else { s }
}

fn dbg_arith(a: &Value, b: &Value, f: fn(f64, f64) -> f64) -> R<Value> {
    if !is_num(a) || !is_num(b) {
        return Err("eval: Operanden muessen Zahlen sein".into());
    }
    let r = f(as_f64(a), as_f64(b));
    if matches!(a, Value::Int(_)) && matches!(b, Value::Int(_)) {
        Ok(Value::Int(r as i64))
    } else {
        Ok(Value::Float(r))
    }
}

/// Binaerer Operator fuer den Debugger-Mini-Evaluator (Subset).
fn dbg_binop(op: &str, a: &Value, b: &Value) -> R<Value> {
    match op {
        "=" => Ok(Value::Bool(value_eq(a, b))),
        "<>" => Ok(Value::Bool(!value_eq(a, b))),
        "and" => Ok(Value::Bool(dbg_truthy(a) && dbg_truthy(b))),
        "or" => Ok(Value::Bool(dbg_truthy(a) || dbg_truthy(b))),
        "+" => {
            if let (Value::Str(x), Value::Str(y)) = (a, b) {
                return Ok(Value::str_rc(format!("{}{}", x, y)));
            }
            dbg_arith(a, b, |x, y| x + y)
        }
        "-" => dbg_arith(a, b, |x, y| x - y),
        "*" => dbg_arith(a, b, |x, y| x * y),
        "/" => {
            if as_f64(b) == 0.0 { return Err("eval: Division durch 0".into()); }
            Ok(Value::Float(as_f64(a) / as_f64(b)))
        }
        "<" => Ok(Value::Bool(as_f64(a) < as_f64(b))),
        ">" => Ok(Value::Bool(as_f64(a) > as_f64(b))),
        "<=" => Ok(Value::Bool(as_f64(a) <= as_f64(b))),
        ">=" => Ok(Value::Bool(as_f64(a) >= as_f64(b))),
        // MOD, \, ^ ... mit denselben Regeln wie die Befehle der VM -- bis
        // 2026-09-27 kannte der Debugger sie nicht, und eine Bedingung wie
        // `i MOD 2 = 0` hielt dann bei JEDEM Durchlauf (fail-open).
        _ => wert_rechnen(op, a, b)
            .ok_or_else(|| format!("eval: Operator '{}' geht mit diesen Werten nicht", op)),
    }
}

/// Ein laufender Aufruf auf dem Rahmenstapel (siehe `Vm::rahmen`): der
/// Zustand des Aufrufers, waehrend `gerufen` laeuft.
struct Rahmen<'p> {
    fn_: &'p Func,
    locals: Vec<Value>,
    stack: Vec<Value>,
    ip: usize,
    try_handlers: Vec<(usize, usize)>,
    gerufen: &'p Func,
}

pub struct Vm<'p> {
    prog: &'p Program,
    globals: HashMap<String, Rc<RefCell<Slot>>>,
    global_slots: Vec<Option<Rc<RefCell<Slot>>>>,
    data_ptr: usize,
    out: String,
    /// Wann `out` zuletzt hinausging. PRINT sammelt (schnell bei vielen
    /// Zeilen), geht aber nach spaetestens 50 ms hinaus -- vorher erst am
    /// Programmende, auch im Terminal: "Bitte warten ..." vor einer langen
    /// Rechnung oder einem SLEEP erschien erst danach (Dogfooding 2026-10-09).
    out_zeit: std::time::Instant,
    input_state: InputModule,
    ui_state: UiState,
    // Modul `scene`: globaler Stack (name, daten). Daten = key->typisierter Wert.
    scene_stack: Vec<(String, HashMap<String, Value>)>,
    // Modul `timer`: AFTER/EVERY-Callbacks + COOLDOWN-Sperren (timer.rs).
    timers: crate::timer::Timers,
    // Wiederverwendete Frame-Vecs (locals/stack) -- spart 2-3 Allokationen
    // pro Funktionsaufruf in heissen Call-Pfaden (fib, Methoden-Loops).
    pool_locals: Vec<Vec<Value>>,
    pool_stacks: Vec<Vec<Value>>,
    // Rahmenstapel (M2): ein Aufruf einer schlichten Funktion wechselt in
    // `dispatch` den Rahmen, statt ueber exec -> run_frame -> dispatch neu
    // einzusteigen. Hier liegt je laufendem Aufruf der Zustand des AUFRUFERS.
    // Jeder `run_frame` kennt seine Basis (die Laenge beim Eintritt).
    rahmen: Vec<Rahmen<'p>>,
    // Maschinencode (M3..M5, src/jit.rs): Vorgabe seit M5; `DHRT_JIT=aus`
    // laesst alles in der VM.
    #[cfg(feature = "jit")]
    jit: Option<crate::jit::Jit>,
    // Quell-Zeile der zuletzt ausgefuehrten Instruktion (fuer Laufzeitfehler-
    // Meldungen). Bei einem propagierenden Fehler haelt es die Zeile der
    // innersten fehlschlagenden Instruktion (sie lief zuletzt). 0 = unbekannt.
    cur_line: u32,
    // Lazy-Fehlerzeile: gesetzt vom INNERSTEN run_frame beim ersten Fehler,
    // von TRY/CATCH beim Konsumieren geloescht (s. run_frame).
    err_line_set: bool,
    // Review-Fund: TRY/CATCH-Bypass + `was_stopped()` verglichen bisher den
    // FEHLERTEXT gegen "__DEBUG_STOP__"/PROFILE_STOP -- ein DH-Programm mit
    // `THROW "__DEBUG_STOP__"` konnte diese internen Kontroll-Signale also
    // faelschlich vortaeuschen (TRY/CATCH faengt es dann nie, main.rs meldet
    // "gestoppt" statt den echten Fehler). Diese beiden Flags sind der
    // eigentliche Signalkanal -- THROW beruehrt sie nie, ein Stringinhalt
    // kann sie also nicht mehr faelschen.
    debug_stop_flag: bool,
    profile_stop_flag: bool,
    /// Das eine FILE-Handle der Standardeingabe (`STDIN()`), sobald es zum
    /// ersten Mal geholt wurde. Siehe `stdin_handle` -- zwei Handles waeren
    /// zwei Puffer auf derselben Leitung.
    stdin_datei: Option<Value>,
    // Von `EXIT(code)` gesetzt (WP A). Some(n) heisst: das Programm hat sich
    // selbst beendet, `run()` gibt EXIT_REQUEST zurueck und main.rs macht
    // daraus den Rueckgabewert des Prozesses. Derselbe Flag-statt-Text-Kanal
    // wie bei den beiden Stop-Signalen darueber.
    exit_code: Option<i32>,
    // WP E -- Pruefen: Zaehler fuer ASSERT/ASSERT_EQ und der Sammel-Modus.
    // `assert_sammeln = false` (Vorgabe) heisst: eine fehlgeschlagene Pruefung
    // bricht ab, wie `assert` es ueberall tut. Ein Pruefprogramm schaltet mit
    // ASSERT_COLLECT(TRUE) um und will dann ALLE Fehler sehen, nicht nur den
    // ersten.
    // WP F -- Angaben zum zuletzt aufgetretenen Fehler, fuer ERROR_LINE() und
    // ERROR_CODE$() im CATCH-Zweig.
    error_line: u32,
    error_code: String,
    // Nur zwischen einem THROW und dem Augenblick, in dem run_frame den Fehler
    // zum ersten Mal sieht, gesetzt -- daran erkennt die VM, dass der Code aus
    // genau diesem THROW stammt und nicht von einem frueheren.
    throw_code: String,
    throw_active: bool,
    /// Von FIN_END gesetzt: der naechste Fehler ist ein WEITERgeworfener und
    /// bringt seine Angaben (Zeile, Code) schon mit.
    rethrow: bool,
    assert_geprueft: i64,
    assert_fehler: i64,
    assert_sammeln: bool,
    // WP H -- Kindprozesse im Hintergrund. `shell_letztes` haelt Code und
    // stderr des zuletzt abgeholten Auftrags, damit SHELL_CODE()/SHELL_ERR$()
    // sie lesen koennen, ohne dass SHELL_RESULT$ drei Werte auf einmal
    // liefern muesste -- dasselbe Muster wie HTTP_STATUS()/HTTP_HEADER().
    shell_auftraege: crate::hintergrund::Auftraege<Result<crate::hintergrund::ShellErgebnis, String>>,
    /// Kindprozesse mit laufender Ausgabe (`PROCESS_*`, prozess.rs). Tombstone-
    /// Vec wie die Auftraege: eine Nummer bleibt gueltig, bis PROCESS_CLOSE.
    prozesse: Vec<Option<crate::prozess::Prozess>>,
    /// Auftraege, die eine EIGENE DH-Funktion ausfuehren (`TASK_*`).
    task_auftraege: crate::hintergrund::Auftraege<Result<crate::hintergrund::TaskErgebnis, String>>,
    /// Fenster als Prozess: die Kinder (WINDOW_*) und, wenn dieses Programm
    /// selbst ein Kind ist, die Verbindung zu den Eltern (PARENT_*). Die
    /// Elternverbindung wird beim ersten PARENT_*-Aufruf aufgebaut.
    fenster: crate::fenster::Fenster,
    eltern: Option<crate::fenster::Eltern>,
    eltern_geprueft: bool,
    shell_letzter_code: i64,
    shell_letzter_fehler: String,
    // WP E -- Melden: Pegel fuer LOG_*, aus der Umgebung (DH_LOG) gelesen.
    // None = noch nicht nachgesehen.
    log_pegel: Option<u8>,
    // Profiler-Sink (Stufe B): None = kein Profiling-Overhead (Normalfall).
    prof: Option<ProfileSink>,
    // Externes Stop-Signal (Editor-Stop-Button bei `dhrt profile --stoppable`):
    // bei gesetztem Flag bricht die Dispatch-Schleife beim naechsten
    // Zeilenwechsel sauber ab (PROFILE_STOP), damit die bis dahin gesammelten
    // Profile-Daten noch ausgegeben werden (kein verlorener Prozess-Kill).
    stop: Option<Arc<AtomicBool>>,
    // Debugger-State (Stufe B): None = kein Debug-Overhead. Call-Tiefe fuer
    // Step over/into/out (inkrementiert pro `exec`).
    dbg: Option<DebugState>,
    // Klassennamen, wie sie im Quelltext stehen (klein -> wie geschrieben),
    // fuer die Anzeige im Debugger; leer ohne Debugger.
    dbg_klassen: HashMap<String, String>,
    /// Variablen- und Feldnamen, wie sie geschrieben wurden (debugger.rs).
    dbg_namen: HashMap<String, String>,
    depth: u32,
    // `dhrt call`: die Ausgabe der Funktion wird GESAMMELT und als Feld
    // `ausgabe` der JSON-Zeile geliefert. Ohne dieses Flag schriebe ein von
    // PROCESS_START gesetztes DHRT_LIVE=1 jede PRINT-Zeile roh hinaus -- und
    // als Kind eines Programms laeuft `call` im Normalfall genau so.
    sammeln: bool,
    #[cfg(feature = "graphics")]
    gfx: Option<crate::graphics::Graphics>,
    // Audio-Geraet (lazy bei erstem LOADSOUND/PLAYSOUND/PLAYMUSIC initialisiert).
    #[cfg(feature = "graphics")]
    audio: Option<crate::audio::Audio>,
    // Retained-Mode-GUI (Modul gui): persistente Fenster/Widgets.
    #[cfg(feature = "graphics")]
    gui: crate::gui::Gui,
    /// Die Schrift des Programms, waehrend ein gui-Befehl mit der eigenen
    /// Schrift der gui (GUI_FONT) laeuft -- zurueckgetauscht wird am Ende
    /// des Befehls, bei GUI_UPDATE schon VOR den Rueckrufen (die sind
    /// Programmcode und sollen dessen Schrift sehen).
    gui_tausch: Option<(i64, i32)>,
    /// Die eingebaute Sans als Vorgabe der gui (einmal geladen; -2 = es gibt
    /// keine, etwa ohne das Feature `schriften`).
    gui_sans: Option<i64>,
    /// GUI_FONT(-1): die gui folgt ausdruecklich der Schrift des Programms,
    /// auch der Pixelschrift.
    gui_pixel: bool,
    // Sprachausgabe (SPEAK): Stimme, Tempo, Vorrat, geplantes Ende.
    #[cfg(feature = "graphics")]
    sprecher: crate::sprache::Sprecher,
    // Modul db (SQLite): Verbindungen + (eager geladene) Resultsets, INTEGER-Handles.
    #[cfg(feature = "db")]
    db_conns: Vec<Option<crate::db::Verbindung>>,
    #[cfg(feature = "db")]
    db_results: Vec<crate::db::DbResult>,
    // WP H: Abfragen, die im Hintergrund laufen. Eigene Verbindung je Auftrag
    // (siehe try_hintergrund) -- die des Programms kann waehrenddessen
    // weiterbenutzt werden.
    #[cfg(feature = "db")]
    db_auftraege: crate::hintergrund::Auftraege<Result<crate::db::DbResult, String>>,
    // Modul net: TCP-Listener/-Sockets + UDP-Sockets, INTEGER-Handles.
    #[cfg(feature = "net")]
    tcp_listeners: Vec<Option<(std::net::TcpListener, i64)>>,
    #[cfg(feature = "net")]
    tcp_socks: Vec<Option<crate::net::NetSock>>,
    #[cfg(feature = "net")]
    udp_socks: Vec<Option<crate::net::UdpSock>>,
    // Modul mqtt: baut auf std::net auf wie net, eigener Handle-Typ.
    #[cfg(feature = "net")]
    mqtt_clients: Vec<Option<crate::mqtt::Client>>,
    /// MQTT_WILL: der letzte Wille fuer das NAECHSTE MQTT_CONNECT.
    #[cfg(feature = "net")]
    mqtt_wille: Option<crate::mqtt::Wille>,
    /// MQTT_TLS_CA: eigene CA (PEM) fuer jedes folgende MQTT_CONNECT mit mqtts://.
    #[cfg(feature = "net")]
    mqtt_ca: Option<String>,
    /// Modul `httpd` -- laufende Webserver (Tombstone-Vec wie ueberall:
    /// eine vergebene Nummer bleibt gueltig, auch wenn ein frueherer
    /// Server schon gestoppt wurde).
    httpd_server: Vec<Option<crate::httpd::Server>>,
    /// Modul `pdf` -- offene Dokumente. Sie halten alle Seiten im
    /// Speicher, bis PDF_SAVE sie schreibt; ein Bericht mit hundert
    /// Seiten ist trotzdem nur ein paar hundert Kilobyte Text.
    #[cfg(feature = "pdf")]
    pdf_dok: Vec<Option<crate::pdf::Dokument>>,
    /// Modul `xlsx` -- offene Mappen (dasselbe Tombstone-Muster).
    xlsx_mappe: Vec<Option<crate::xlsx::Mappe>>,
    /// Modul `smtp` -- Nachrichten im Aufbau.
    #[cfg(feature = "smtp")]
    smtp_mail: Vec<Option<crate::smtp::Mail>>,
    // Modul html: letzter HTTP-Status/-Header (fuer HTTP_STATUS/HTTP_HEADER).
    #[cfg(feature = "http")]
    http_status: i64,
    #[cfg(feature = "http")]
    http_headers: Vec<(String, String)>,
    // Modul html (WP C): der ROHE Rumpf der letzten Antwort, fuer HTTP_BYTES().
    // Noetig, weil HTTP_GET & Co. den Rumpf verlustbehaftet nach UTF-8 wandeln
    // -- bei einem Bild oder einer Zip-Datei bliebe davon nichts Brauchbares.
    #[cfg(feature = "http")]
    http_body: Vec<u8>,
    // Modul html (WP C): Kopfzeilen, die JEDER folgende Aufruf mitschickt
    // (HTTP_SET_HEADER) -- fuer ein Token, das man einmal setzt statt es an
    // jeden einzelnen Aufruf zu haengen. Pro Aufruf uebergebene Kopfzeilen
    // gewinnen dagegen.
    #[cfg(feature = "http")]
    http_default_header: Vec<(String, String)>,
    // Modul html (WP C): Zeitgrenze in Sekunden fuer folgende Aufrufe.
    #[cfg(feature = "http")]
    http_timeout: u64,
    // Modul html: Abrufe im Hintergrund (HTTP_GET_START/READY/RESULT).
    #[cfg(feature = "http")]
    http_abrufe: crate::html::Abrufe,
    // Modul cloud: Basis-URL/API-Key aus CLOUD_CONFIGURE + letzte Fehlermeldung.
    #[cfg(feature = "http")]
    cloud_base_url: String,
    #[cfg(feature = "http")]
    cloud_api_key: String,
    #[cfg(feature = "http")]
    cloud_last_error: String,
    // Hardware/IoT-Handles (INTEGER-Index in VM-Vecs).
    #[cfg(feature = "midi")]
    midi_ins: Vec<Option<crate::midi::Eingang>>,
    // Modul video: je Griff ein Video samt seinem IMAGE und der Uhr.
    #[cfg(all(feature = "video", feature = "graphics"))]
    videos: Vec<Option<VideoZustand>>,
    /// Offene Kameras (WEBCAM_OPEN), Tombstone-Plaetze wie bei den Videos.
    #[cfg(feature = "graphics")]
    webcams: Vec<Option<WebcamZustand>>,
    #[cfg(feature = "midi")]
    midi_outs: Vec<Option<crate::midi::Ausgang>>,
    #[cfg(feature = "serial")]
    serial_ports: Vec<Option<crate::serial::Port>>,
    firmata_boards: Vec<Option<crate::firmata::Board>>,
    /// Der laufende Mikrofon-Eingang (MIC_START) und der Zustand von
    /// MIC_FFT (Nachleuchten, Verstaerkung) -- getrennt von AUDIO_FFT.
    #[cfg(feature = "graphics")]
    mikrofon: Option<crate::mikrofon::Lauf>,
    #[cfg(feature = "graphics")]
    mic_baender: Vec<f32>,
    #[cfg(feature = "graphics")]
    mic_agc: f32,
    /// WebSocket-Server und -Verbindungen (WS_*), OSC-Anschluesse,
    /// das Ergebnis der letzten mDNS-Suche und der mDNS-Antworter.
    #[cfg(feature = "net")]
    ws_server: Vec<Option<crate::websocket::Server>>,
    #[cfg(feature = "net")]
    ws_verbindungen: Vec<Option<crate::websocket::Verbindung>>,
    osc_anschluesse: Vec<Option<crate::osc::Osc>>,
    /// Modbus-Geraete (MODBUS_*) und gelesene/angelegte MIDI-Dateien.
    modbus_geraete: Vec<Option<crate::modbus::Geraet>>,
    midi_dateien: Vec<Option<crate::midifile::Datei>>,
    hotplug_aktuell: Option<crate::hotplug::Ereignis>,
    mdns_dienste: Vec<crate::mdns::Dienst>,
    mdns_typ: String,
    mdns_antworter: Option<crate::mdns::Antworter>,
    gpio_chips: Vec<Option<crate::pins::Chip>>,
    i2c_busse: Vec<Option<crate::pins::Bus>>,
    spi_geraete: Vec<Option<crate::pins::Spi>>,
    #[cfg(feature = "usb")]
    usb_devs: Vec<Option<hidapi::HidDevice>>,
    #[cfg(feature = "bt")]
    bt_periphs: Vec<Option<btleplug::platform::Peripheral>>,
}

type R<T> = Result<T, String>;

/// Zugriff auf ein Global, das noch nicht gesetzt ist.
///
/// Im gewoehnlichen Lauf heisst das: gelesen, bevor das `DIM` an der Reihe
/// war. Bei `dhrt call` heisst es etwas anderes und Haeufigeres -- dort laeuft
/// das Hauptprogramm gar nicht, also ist KEIN Global gesetzt, auch keine
/// `CONST`. Das ist die Zusage der Auftragsgrenze, und wer sie zum ersten Mal
/// trifft, soll sie hier erklaert bekommen statt "Global-Slot leer" zu lesen.
const GLOBAL_UNGESETZT: &str = concat!(
    "Zugriff auf eine globale Variable, die noch nicht gesetzt ist. ",
    "Laeuft das hier als Auftrag (`dhrt call` / TASK_START)? Dann ist das ",
    "erwartet: das Hauptprogramm laeuft dabei NICHT, also ist kein Global ",
    "gesetzt -- auch keine CONST, deren Wert erst beim Laufen feststeht (eine ",
    "mit festem Wert wie `CONST BREITE = 640` setzt der Compiler ein). Gib der Funktion als ",
    "Parameter mit, was sie braucht.");

// Die Methoden der VM stehen nach Themen in eigenen Dateien; hier bleiben der
// Zustand, der Lauf (exec, dispatch, Koroutinen), OOP und die Rueckrufe.
#[path = "vm_debug.rs"]
mod debug;
#[path = "vm_dienste.rs"]
mod dienste;
#[path = "vm_geraete.rs"]
mod geraete;
#[path = "vm_system.rs"]
mod system;
#[path = "vm_gui.rs"]
mod gui_befehle;
#[path = "vm_grafik.rs"]
mod grafik;
#[path = "vm_helfer.rs"]
mod helfer;
pub(crate) use helfer::*;

impl<'p> Vm<'p> {
    /// Meldung fuer einen globalen Platz, der gelesen wird, bevor er gesetzt
    /// ist -- mit dem NAMEN der Variable. Seit auch Felder, Maps und
    /// Strukturen einen Platz haben, traefe sie sonst jedes Programm, dessen
    /// SUB ein Feld liest, bevor dessen DIM gelaufen ist; vorher hiess es dort
    /// klar "Variable 'feld' nicht deklariert".
    #[inline(never)]
    fn global_ungesetzt(&self, idx: usize) -> String {
        match self.prog.global_names.get(idx).filter(|n| !n.is_empty()) {
            // Der Name HINTER dem alten Text: Aufrufer setzen ihr eigenes
            // Praefix davor ("TASK: ..."), und task.dhtest prueft genau diese
            // Folge.
            Some(n) => format!(
                "{} Im gewoehnlichen Lauf heisst das: das DIM von '{}' ist noch nicht gelaufen -- steht es weiter unten im Programm als die Stelle, die es benutzt?",
                GLOBAL_UNGESETZT, n),
            None => GLOBAL_UNGESETZT.to_string(),
        }
    }

    /// Maus fuer ui-Widgets: liefert Off-Screen-Koordinaten, wenn die aktuelle
    /// Fenster-Ebene keinen Input besitzt (von einem anderen Fenster ueberdeckt).
    /// Liefert (mx, my, down). Spiegelt ui._mouse + g.mouse_button(0).
    #[cfg(feature = "graphics")]
    fn ui_mouse_gated(&self) -> R<(i32, i32, bool)> {
        let g = self.gfx.as_ref().ok_or("UI-Builtin vor SCREEN aufgerufen")?;
        let down = g.mouse_button(0);
        if self.ui_state.input_blocked {
            Ok((-9999, -9999, down))
        } else {
            Ok((g.mouse_x() as i32, g.mouse_y() as i32, down))
        }
    }

    pub fn new(prog: &'p Program) -> Self {
        crate::value::klassen_setzen(prog.classes.iter().map(|(n, ci)| (n, ci.layout.vorfahren.as_slice())));
        let mut global_slots = Vec::with_capacity(prog.n_globals);
        for _ in 0..prog.n_globals {
            global_slots.push(None);
        }
        let mut vm = Vm {
            prog,
            globals: HashMap::new(),
            global_slots,
            data_ptr: 0,
            out: String::new(),
            out_zeit: std::time::Instant::now(),
            input_state: InputModule::default(),
            ui_state: UiState::new(),
            scene_stack: Vec::new(),
            timers: crate::timer::Timers::default(),
            pool_locals: Vec::new(),
            pool_stacks: Vec::new(),
            rahmen: Vec::new(),
            #[cfg(feature = "jit")]
            jit: if std::env::var("DHRT_JIT").map_or(true, |v| v != "aus") {
                match crate::jit::Jit::neu(prog) {
                    Ok(j) => Some(j),
                    // Uebersetzen scheiterte: melden, dann laeuft alles in der VM.
                    Err(e) => { eprintln!("dhrt: Maschinencode nicht moeglich -- {}", e); None }
                }
            } else { None },
            cur_line: 0,
            err_line_set: false,
            debug_stop_flag: false,
            stdin_datei: None,
            profile_stop_flag: false,
            exit_code: None,
            error_line: 0,
            error_code: String::new(),
            throw_code: String::new(),
            throw_active: false,
            rethrow: false,
            assert_geprueft: 0,
            assert_fehler: 0,
            assert_sammeln: false,
            shell_auftraege: Default::default(),
            prozesse: Vec::new(),
            task_auftraege: Default::default(),
            fenster: Default::default(), eltern: None, eltern_geprueft: false,
            shell_letzter_code: 0,
            shell_letzter_fehler: String::new(),
            log_pegel: None,
            prof: None,
            stop: None,
            dbg: None,
            dbg_klassen: HashMap::new(),
            dbg_namen: HashMap::new(),
            sammeln: false,
            depth: 0,
            #[cfg(feature = "graphics")]
            gfx: None,
            #[cfg(feature = "graphics")]
            audio: None,
            #[cfg(feature = "graphics")]
            gui: crate::gui::Gui::new(),
            gui_tausch: None,
            gui_sans: None,
            gui_pixel: false,
            #[cfg(feature = "graphics")]
            sprecher: crate::sprache::Sprecher::new(),
            #[cfg(feature = "db")]
            db_conns: Vec::new(),
            #[cfg(feature = "db")]
            db_results: Vec::new(),
            #[cfg(feature = "db")]
            db_auftraege: Default::default(),
            #[cfg(feature = "net")]
            tcp_listeners: Vec::new(),
            #[cfg(feature = "net")]
            tcp_socks: Vec::new(),
            #[cfg(feature = "net")]
            udp_socks: Vec::new(),
            #[cfg(feature = "net")]
            mqtt_clients: Vec::new(),
            #[cfg(feature = "net")]
            mqtt_wille: None,
            #[cfg(feature = "net")]
            mqtt_ca: None,
            httpd_server: Vec::new(),
            #[cfg(feature = "pdf")]
            pdf_dok: Vec::new(),
            xlsx_mappe: Vec::new(),
            #[cfg(feature = "smtp")]
            smtp_mail: Vec::new(),
            #[cfg(feature = "http")]
            http_status: 0,
            #[cfg(feature = "http")]
            cloud_base_url: String::new(),
            #[cfg(feature = "http")]
            cloud_api_key: String::new(),
            #[cfg(feature = "http")]
            cloud_last_error: String::new(),
            #[cfg(feature = "http")]
            http_headers: Vec::new(),
            #[cfg(feature = "http")]
            http_body: Vec::new(),
            #[cfg(feature = "http")]
            http_default_header: Vec::new(),
            #[cfg(feature = "http")]
            http_timeout: 10,
            #[cfg(feature = "http")]
            http_abrufe: crate::html::Abrufe::default(),
            #[cfg(feature = "midi")]
            midi_ins: Vec::new(),
            #[cfg(all(feature = "video", feature = "graphics"))]
            videos: Vec::new(),
            #[cfg(feature = "graphics")]
            webcams: Vec::new(),
            #[cfg(feature = "midi")]
            midi_outs: Vec::new(),
            #[cfg(feature = "serial")]
            serial_ports: Vec::new(),
            firmata_boards: Vec::new(),
            #[cfg(feature = "graphics")]
            mikrofon: None,
            #[cfg(feature = "graphics")]
            mic_baender: Vec::new(),
            #[cfg(feature = "graphics")]
            mic_agc: 1e-4,
            #[cfg(feature = "net")]
            ws_server: Vec::new(),
            #[cfg(feature = "net")]
            ws_verbindungen: Vec::new(),
            osc_anschluesse: Vec::new(),
            modbus_geraete: Vec::new(),
            midi_dateien: Vec::new(),
            hotplug_aktuell: None,
            mdns_dienste: Vec::new(),
            mdns_typ: String::new(),
            mdns_antworter: None,
            gpio_chips: Vec::new(),
            i2c_busse: Vec::new(),
            spi_geraete: Vec::new(),
            #[cfg(feature = "usb")]
            usb_devs: Vec::new(),
            #[cfg(feature = "bt")]
            bt_periphs: Vec::new(),
        };
        vm.register_default_globals();
        vm
    }

    /// Registriert die vordefinierten Globals (Farben, Tasten, PI) -- analog
    /// zu `vm._register_default_globals`. Werte identisch zu Python (Farben =
    /// 0xRRGGBB, Tasten = SDL/pygame-Keycodes), damit `LOAD_NAME "black"` etc.
    /// in serialisierten Programmen funktioniert.
    fn register_default_globals(&mut self) {
        let mut put = |name: &str, ty: &str, value: Value| {
            self.globals.insert(name.to_string(), Rc::new(RefCell::new(Slot {
                ty: ty.to_string(), value, is_const: true,
            })));
        };
        for (n, v) in DEFAULT_COLORS { put(n, "integer", Value::Int(*v)); }
        for (n, v) in DEFAULT_KEYS { put(n, "integer", Value::Int(*v)); }
        put("pi", "float", Value::Float(std::f64::consts::PI));
        put("tau", "float", Value::Float(std::f64::consts::TAU));
    }

    pub fn run(&mut self) -> R<()> {
        self.exec(&self.prog.main, Vec::new(), None)?;
        Ok(())
    }

    /// EINE Funktion aufrufen, ohne das Hauptprogramm zu fahren (WP H,
    /// `dhrt call`).
    ///
    /// Grundlage fuer `TASK_START`: ein Auftrag laeuft als eigener Prozess,
    /// damit DH-Code nicht ueber eine Thread-Grenze muss (`Value` haelt
    /// ueberall `Rc`, `Program` ist weder Send noch Sync -- siehe
    /// docs/intern/entwurf-task-start.md).
    ///
    /// DAS HAUPTPROGRAMM LAEUFT BEWUSST NICHT MIT. Damit sind auch die
    /// Globals nicht gesetzt -- und eine `CONST` auf oberster Ebene IST ein
    /// Global. Das ist keine Panne, sondern die Zusage: ein Auftrag sieht
    /// keine Globals, er bekommt mit, was er braucht. Dieselbe Grenze wie bei
    /// einem mit `AS` importierten Modul (WP I.1).
    pub fn call_named(&mut self, name: &str, args: Vec<Value>) -> R<Value> {
        let low = name.to_lowercase();
        let idx = match self.prog.fn_index.get(&low) {
            Some(i) => *i,
            None => {
                let mut bekannt: Vec<&str> =
                    self.prog.fn_index.keys().map(|k| k.as_str()).collect();
                bekannt.sort();
                return Err(format!(
                    "Funktion '{}' gibt es nicht. Bekannt: {}",
                    name,
                    if bekannt.is_empty() { "keine".to_string() }
                    else { bekannt.join(", ") }));
            }
        };
        self.exec(&self.prog.functions[idx], args, None)
    }

    pub fn take_output(self) -> String {
        self.out
    }

    /// Ausgabe sammeln statt sie laufend hinauszuschreiben (`dhrt call`).
    pub fn sammle_ausgabe(&mut self) {
        self.sammeln = true;
    }

    /// Fuer INPUT: gepufferten Output + Prompt SOFORT auf echtes stdout flushen
    /// (sonst erscheint der Prompt erst nach der Eingabe) und `self.out` leeren,
    /// damit der finale take_output() nichts doppelt schreibt.
    fn flush_and_prompt(&mut self, prompt: &str) {
        // Unter dem Profiler UND unter dem Debugger gehoert stdout nicht dem
        // Programm, sondern dem JSON (Profiler: ein Blob am Ende; Debugger: ein
        // Ereignis je Zeile). Der Prompt geht darum in den Output-Puffer, der
        // ohnehin als `output`-Ereignis bzw. im `output`-Feld herauskommt --
        // sonst klebt er mitten im JSON und macht es unlesbar.
        if self.prof.is_some() || self.dbg.is_some() {
            self.out.push_str(prompt);
            return;
        }
        self.flush_out();       // geteilt mit EPRINT/SHELL (try_os), siehe dort
        use std::io::Write;
        let so = std::io::stdout();
        let mut h = so.lock();
        let _ = h.write_all(prompt.as_bytes());
        let _ = h.flush();
    }

    /// Profiling fuer den naechsten `run()` aktivieren (Stufe B, `dhrt profile`).
    pub fn enable_profiler(&mut self) {
        self.prof = Some(ProfileSink::new());
    }

    /// Externes Stop-Signal installieren (Editor-Stop-Button). Wird das `flag`
    /// von aussen (z.B. einem stdin-Reader-Thread) auf `true` gesetzt, bricht
    /// `run()` beim naechsten Zeilenwechsel mit PROFILE_STOP ab -- die bisher
    /// gesammelten Profile-Daten bleiben so erhalten (statt durch Prozess-Kill
    /// verloren zu gehen). Endlos-Loop-Programme (Grafik/`WHILE TRUE`) lassen
    /// sich damit sauber profilieren.
    pub fn set_stop_flag(&mut self, flag: Arc<AtomicBool>) {
        self.stop = Some(flag);
    }

    /// Ob `run()` durch das externe Stop-Signal abgebrochen wurde.
    ///
    /// Review-Fund: verglich frueher den Fehlertext gegen PROFILE_STOP -- ein
    /// DH-Programm mit `THROW "__PROFILE_STOP__"` haette einen echten Fehler
    /// so als "sauber gestoppt" maskiert. Jetzt liest die Funktion das
    /// interne Flag, das THROW nie setzt (`err` bleibt fuer Kompatibilitaet
    /// im Signature, wird aber nicht mehr fuer die Entscheidung genutzt).
    pub fn was_stopped(&self, _err: &str) -> bool {
        self.profile_stop_flag
    }

    /// Rueckgabewert aus `EXIT(code)`, falls das Programm sich selbst beendet
    /// hat. `None` = normales Ende oder echter Fehler. Vor `take_output()`
    /// lesen -- das konsumiert die VM.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Hinweis fuers Programmende, oder None. Siehe `Graphics::nie_gezeigt`.
    #[cfg(feature = "graphics")]
    pub fn hinweis_ohne_flip(&self) -> Option<&'static str> {
        self.gfx.as_ref().filter(|g| g.nie_gezeigt()).map(|_| {
            concat!("Hinweis: Das Fenster ist leer geblieben -- es wurde gezeichnet, aber FLIP() fehlt. ",
                    "dhrt sammelt die Zeichenbefehle und zeigt sie erst beim FLIP(); ",
                    "in einer Bildschleife steht es am Ende jedes Durchlaufs.")
        })
    }
    #[cfg(not(feature = "graphics"))]
    pub fn hinweis_ohne_flip(&self) -> Option<&'static str> { None }

    /// Ob `run()` durch den Debugger-Stop-Befehl (oder EOF auf stdin waehrend
    /// `dhrt debug`) abgebrochen wurde -- analog zu `was_stopped`, siehe
    /// dortiger Review-Fund-Kommentar.
    pub fn was_debug_stopped(&self) -> bool {
        self.debug_stop_flag
    }

    /// Eine INPUT-Zeile lesen. Bei aktivem Stop-Kanal (`dhrt profile
    /// --stoppable`) gehoert stdin dem Stop-Reader-Thread -- INPUT liefert dann
    /// "" (wie der frueher genutzte DEVNULL-stdin), statt zu blockieren oder mit
    /// dem Stop-Reader um die Eingabe zu konkurrieren.
    fn read_input_line(&self) -> String {
        // Kein interaktives stdin, wenn es jemand anderem gehoert: beim
        // Profiler dem Stop-Kanal, beim Debugger dem Kommando-Protokoll. Ohne
        // diese Sperre stahl ein INPUT dem Debugger eine Kommandozeile und
        // die Sitzung lief aus dem Tritt.
        if self.stop.is_some() || self.dbg.is_some() {
            return String::new();
        }
        // Gibt es das STDIN()-Handle schon, liest INPUT aus SEINEM Puffer --
        // es liest voraus (1 MiB, siehe `stdin_handle`), und was dort liegt,
        // saehe ein zweiter Leser nicht.
        if let Some(Value::File(f)) = &self.stdin_datei {
            let mut f = f.borrow_mut();
            let (kod, anfang, pfad) = (f.kod, f.am_anfang, f.path.clone());
            f.am_anfang = false;
            if let Some(r) = f.leser() {
                let mut roh: Vec<u8> = Vec::new();
                let _ = r.read_until(b'\n', &mut roh);
                while roh.last() == Some(&b'\n') || roh.last() == Some(&b'\r') { roh.pop(); }
                let z = crate::kodierung::dekodieren(&roh, kod, "INPUT", &pfad)
                    .unwrap_or_else(|_| String::from_utf8_lossy(&roh).into_owned());
                return if anfang { crate::kodierung::ohne_bom(z) } else { z };
            }
        }
        read_input_line()
    }

    /// Eine INPUT-Zeile -- unter dem Debugger aus einem `input`-Kommando.
    ///
    /// Bis 2026-09-27 bekam INPUT unter dem Debugger immer "" (stdin gehoert
    /// dort dem Protokoll), ein Programm mit einer Eingabe liess sich also
    /// gar nicht sinnvoll debuggen. Jetzt meldet es `{"event":"input"}` und
    /// wartet auf `{"cmd":"input","text":...}`; Haltepunkte und Ueberwachen
    /// nimmt es dabei weiter an, `stop` beendet. Ist stdin zu: "" wie frueher.
    fn eingabe_zeile(&mut self) -> R<String> {
        let mut dbg = match self.dbg.take() {
            Some(d) => d,
            None => return Ok(self.read_input_line()),
        };
        if self.out.len() > dbg.out_sent {
            let chunk = self.out[dbg.out_sent..].to_string();
            dbg.out_sent = self.out.len();
            dbg_emit(&serde_json::json!({"event":"output","text":chunk}));
        }
        let (datei, zeile) = dbg.stelle(self.cur_line);
        dbg_emit(&serde_json::json!({"event":"input", "line": zeile, "file": datei}));
        let r = loop {
            for cmd in dbg.kanal.lebende() {
                match crate::debugger::art(&cmd) {
                    "set-breakpoints" => self.dbg_set_breakpoints(&mut dbg, &cmd),
                    "set-watches" => Self::dbg_watches_setzen(&mut dbg, &cmd),
                    "pause" => dbg.step = StepMode::Into,
                    _ => {}
                }
            }
            match dbg.kanal.eingabe() {
                Ok(Some(t)) => break Ok(t),
                Ok(None) => {}
                Err(()) => {
                    self.debug_stop_flag = true;
                    break Err("__DEBUG_STOP__".to_string());
                }
            }
            if !dbg.kanal.auf_neues_warten(&mut Self::fenster_pumpe(self.fenster_da())) { break Ok(String::new()); }
        };
        self.dbg = Some(dbg);
        r
    }

    /// Das FILE-Handle der Standardeingabe -- IMMER dasselbe.
    ///
    /// Zwei eigene Handles waeren zwei Lesepuffer auf derselben Leitung: das
    /// eine liest voraus, dem anderen fehlen die Zeilen. Deshalb liefert
    /// `STDIN()` bei jedem Aufruf dasselbe Handle, und eine zweite Angabe
    /// einer ANDEREN Kodierung ist ein Fehler statt einer stillen Wirkungslosigkeit.
    ///
    /// `stdin().lock()` teilt sich seinen Puffer mit dem `read_line` in
    /// `read_input_line` (Rusts Stdin-Sperre ist wiedereintrittsfaehig und
    /// haengt an EINEM prozessweiten Puffer) -- INPUT und STDIN() lassen sich
    /// also im selben Programm mischen, ohne dass Zeilen verloren gehen.
    fn stdin_handle(&mut self, kod: crate::kodierung::Kodierung) -> R<Value> {
        if let Some(h) = &self.stdin_datei {
            if let Value::File(f) = h {
                let alt = f.borrow().kod;
                if alt != kod {
                    return Err(format!(
                        "STDIN: die Standardeingabe wurde in diesem Programm schon mit einer anderen Kodierung geoeffnet -- es gibt nur EIN Handle dafuer (sonst laesen zwei Puffer an derselben Leitung)"));
                }
            }
            return Ok(h.clone());
        }
        // Gehoert stdin jemand anderem (Stop-Kanal/Debugger), einen leeren
        // Strom geben: die uebliche Schleife laeuft dann null mal, statt dem
        // Protokoll eine Zeile wegzunehmen.
        // Ein eigener Puffer von 1 MiB: Rusts Standardeingabe liest in 8-KB-
        // Stuecken, und an einer Pipe kostet jeder Lesevorgang -- 400 000
        // Zeilen aus `cat datei |` (Git-Bash) brauchten 2,8 s statt 0,1 s mit
        // `< datei`, aus `type datei |` (cmd) 0,7 s (gemessen 2026-10-11,
        // Konsolen-Runde). INPUT liest danach aus DIESEM Puffer
        // (`read_input_line`), damit beide gemischt werden koennen.
        let strom: Box<dyn std::io::BufRead> = if self.stop.is_some() || self.dbg.is_some() {
            Box::new(std::io::empty())
        } else {
            Box::new(std::io::BufReader::with_capacity(1 << 20, std::io::stdin().lock()))
        };
        let h = Value::File(Rc::new(RefCell::new(crate::value::DhFile {
            path: "<Standardeingabe>".to_string(),
            h: crate::value::FileH::Strom(strom),
            kod,
            am_anfang: true,
        })));
        self.stdin_datei = Some(h.clone());
        Ok(h)
    }

    /// Profiler-Ergebnis abholen: Gesamtzeit + pro Zeile (count, time_secs).
    /// Leer, wenn kein Profiling aktiv war.
    pub fn take_profile(&mut self) -> (f64, Vec<(u32, u64, f64)>) {
        match self.prof.take() {
            None => (0.0, Vec::new()),
            Some(mut p) => {
                let total = p.finalize();
                let mut lines: Vec<(u32, u64, f64)> = p.counts.iter()
                    .map(|(&ln, &c)| (ln, c, *p.times.get(&ln).unwrap_or(&0.0)))
                    .collect();
                lines.sort_by_key(|&(ln, _, _)| ln);
                (total, lines)
            }
        }
    }

    /// `wert IS Typname` (der Compiler emittiert `__is_typ(wert, "name")`).
    ///
    /// Gehoert hierher und nicht nach `builtins.rs`, weil die Vererbungskette
    /// nur die VM kennt.
    /// Ist `kind` die Klasse `basis` oder stammt davon ab? (Gross/klein egal.)
    fn klasse_stammt_ab(&self, kind: &str, basis: &str) -> bool {
        let mut cur: Option<String> = Some(kind.to_string());
        let mut schritte = 0;
        while let Some(c) = cur {
            if c.eq_ignore_ascii_case(basis) { return true; }
            schritte += 1;
            if schritte > 64 { return false; }
            cur = match self.prog.classes.get(c.as_str()) {
                Some(ci) if !ci.parent_name.is_empty() => Some(ci.parent_name.to_string()),
                _ => None,
            };
        }
        false
    }

    fn try_typtest(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        // `__frisch(typ$)`: ein NEUER Startwert fuer ein DIM in einem
        // Schleifenrumpf (compiler.rs `dim_frisch`) -- leeres Feld, leere MAP,
        // neue Struct-Instanz, neutrales Element, NIL fuer Klassen. Ein DIM in
        // einer Schleife faengt seit 2026.28 jede Runde frisch an.
        if name == "__frisch" {
            let ty: &str = match a.first() { Some(Value::Str(t)) => t, _ => "" };
            return Ok(Some(if let Some(vt) = ty.strip_prefix("map:") {
                Value::Map(Rc::new(RefCell::new(DhMap::new(vt.to_string()))))
            } else if ty.starts_with("array:") {
                leeres_feld(&ty)
            } else {
                self.element_default(&ty)
            }));
        }
        if name == "__array_of_type" { return self.array_of_type(a).map(Some); }
        if name != "__is_typ" { return Ok(None); }
        let gesucht = match a.get(1) {
            Some(Value::Str(s)) => s.to_string(),
            _ => return Err("IS: interner Fehler -- Typname fehlt".into()),
        };
        let wert = a.first().unwrap_or(&Value::Nil);
        Ok(Some(Value::Bool(self.ist_vom_typ(wert, &gesucht))))
    }

    /// `wert IS gesucht`: eine Klasse schlaegt den Werttyp-Vergleich -- `x IS
    /// Gegner` fragt nach der Vererbungskette, nicht nach dem pauschalen
    /// "OBJECT". Sonst der Werttyp (`x IS INTEGER`, `x IS NIL`, `x IS VEC2`).
    fn ist_vom_typ(&self, wert: &Value, gesucht: &str) -> bool {
        if self.prog.classes.contains_key(gesucht) {
            return match wert {
                Value::Instance(rc) => {
                    let mut cur: Option<String> = Some(rc.borrow().class_name.to_string());
                    while let Some(c) = cur {
                        if c == gesucht { return true; }
                        cur = match self.prog.classes.get(c.as_str()) {
                            Some(ci) if !ci.parent_name.is_empty() => Some(ci.parent_name.to_string()),
                            _ => None,
                        };
                    }
                    false
                }
                _ => false,
            };
        }
        wert.type_name().eq_ignore_ascii_case(gesucht)
    }

    /// `ARRAY_OF_TYPE(feld, Typ)` (der Compiler emittiert
    /// `__array_of_type(feld, "typ")`): ein NEUES Feld mit den Elementen, die
    /// `IS Typ` sind, in ihrer Reihenfolge. Der Elementtyp des Ergebnisses ist
    /// `Typ` -- es passt also an ein `ARRAY OF Typ`, auch wenn das Feld selbst
    /// eine Vorfahrin als Elementtyp hat.
    fn array_of_type(&self, a: &[Value]) -> R<Value> {
        let gesucht = match a.get(1) {
            Some(Value::Str(s)) => s.to_string(),
            _ => return Err("ARRAY_OF_TYPE: interner Fehler -- Typname fehlt".into()),
        };
        let werte: Vec<Value> = match a.first() {
            Some(Value::Array(x)) => {
                let arr = x.borrow();
                if arr.dims.len() != 1 {
                    return Err("ARRAY_OF_TYPE: nur eindimensionale Felder".into());
                }
                arr.cells.to_values()
            }
            Some(Value::Tuple(t)) => t.to_vec(),
            Some(andere) => return Err(format!(
                "ARRAY_OF_TYPE: erwartet ein Feld oder TUPLE, erhalten {} -- Aufruf: ARRAY_OF_TYPE(feld, Typ)",
                andere.type_name())),
            None => return Err("ARRAY_OF_TYPE: erwartet ein Feld".into()),
        };
        let aus: Vec<Value> = werte.into_iter().filter(|v| self.ist_vom_typ(v, &gesucht)).collect();
        let mut neu = DhArray::new(gesucht, vec![aus.len() as i64], || Value::Nil);
        for (i, v) in aus.into_iter().enumerate() { neu.cells.set(i, v); }
        Ok(Value::Array(Rc::new(RefCell::new(neu))))
    }

    // ------------------------------------------------------- Rueckrufe
    /// Einen gespeicherten Rueckruf auf die auszufuehrende Funktion und den
    /// Empfaenger abbilden.
    ///
    /// Aufgeloest wird ERST HIER, nicht beim Registrieren: bei einer
    /// gebundenen Methode entscheidet die Klasse der Instanz, welche
    /// Ueberschreibung greift -- und die steht erst zur Aufrufzeit fest.
    fn rueckruf_aufloesen(&self, cb: &crate::value::Rueckruf, kontext: &str)
        -> R<(&'p Func, Option<Value>)>
    {
        match &cb.empfaenger {
            Some(obj) => {
                let cn = match obj {
                    Value::Instance(rc) => rc.borrow().class_name.clone(),
                    other => return Err(format!(
                        "{}: Rueckruf ist an {} gebunden, nicht an ein Objekt",
                        kontext, other.type_name())),
                };
                let f = self.resolve_method(&cn, &cb.name).ok_or_else(||
                    format!("{}: Methode '{}' existiert nicht in {}", kontext, cb.name, cn))?;
                Ok((f, Some(obj.clone())))
            }
            None => {
                let f = self.prog.func(cb.name.as_ref()).ok_or_else(||
                    format!("{}: Funktion '{}' existiert nicht", kontext, cb.name))?;
                Ok((f, None))
            }
        }
    }

    /// Einen parameterlosen Rueckruf ausfuehren (GUI_UPDATE, TIMER_UPDATE).
    fn rueckruf_rufen(&mut self, cb: &crate::value::Rueckruf, kontext: &str) -> R<()> {
        let (f, empf) = self.rueckruf_aufloesen(cb, kontext)?;
        self.rueckruf_exec(cb, f, &empf, Vec::new())?;
        Ok(())
    }

    /// Einen aufgeloesten Rueckruf mit Argumenten ausfuehren -- bei einem
    /// Lambda kommen seine kopierten Locals VOR die Argumente.
    fn rueckruf_exec(&mut self, cb: &crate::value::Rueckruf, f: &'p Func, empf: &Option<Value>,
                     args: Vec<Value>) -> R<Value> {
        if cb.vorab.is_empty() { return self.exec(f, args, empf.clone()).map_err(|e| Self::param_von(e, f)); }
        let mut alle = cb.vorab.clone();
        alle.extend(args);
        self.exec(f, alle, empf.clone()).map_err(|e| Self::param_von(e, f))
    }

    /// Fuer Rueckrufe aus einer fremden Bibliothek (ffi.rs): einen
    /// Funktionswert (FUNCREF, gebundene Methode, Lambda) mit Argumenten rufen.
    #[cfg_attr(not(feature = "ffi"), allow(dead_code))]
    pub(crate) fn wert_rufen(&mut self, wert: &Value, args: Vec<Value>, kontext: &str) -> R<Value> {
        let cb = crate::value::Rueckruf::aus_wert(wert)
            .ok_or_else(|| format!("{}: erwartet eine Funktion, erhalten {}", kontext, wert.type_name()))?;
        let (f, empf) = self.rueckruf_aufloesen(&cb, kontext)?;
        self.rueckruf_exec(&cb, f, &empf, args)
    }

    /// Wie viele Werte nimmt ein Funktionswert von aussen (`None` = beliebig
    /// viele)? Fuer ffi.rs, bevor ein Rueckruf an eine Bibliothek geht.
    #[cfg_attr(not(feature = "ffi"), allow(dead_code))]
    pub(crate) fn wert_stellen(&self, wert: &Value, kontext: &str) -> R<Option<usize>> {
        let cb = crate::value::Rueckruf::aus_wert(wert)
            .ok_or_else(|| format!("{}: erwartet eine Funktion, erhalten {}", kontext, wert.type_name()))?;
        let (f, _) = self.rueckruf_aufloesen(&cb, kontext)?;
        Ok(Self::rueckruf_stellen(&cb, f))
    }

    /// Wie viele Werte nimmt dieser Rueckruf von aussen? (`None` = beliebig
    /// viele.) Fuer eine Meldung, bevor ARRAY_MAP & Co. ins Leere rufen.
    fn rueckruf_stellen(cb: &crate::value::Rueckruf, f: &Func) -> Option<usize> {
        if f.is_variadic { None } else { Some(f.n_params.saturating_sub(cb.vorab.len())) }
    }

    // ---------------------------------------------------------------- OOP
    #[inline(never)]
    fn resolve_method(&self, class_name: &str, method: &str) -> Option<&'p Func> {
        // Methoden-Keys liegen lowercase vor (Compiler emittiert lowercase) --
        // nur im seltenen gemischten Fall allozieren.
        let lowered;
        let key: &str = if method.bytes().any(|b| b.is_ascii_uppercase()) {
            lowered = method.to_lowercase();
            &lowered
        } else { method };
        let mut cur = self.prog.classes.get(class_name);
        while let Some(ci) = cur {
            if let Some(m) = ci.methods.get(key) {
                return Some(m);
            }
            if ci.parent_name.is_empty() {
                break;
            }
            cur = self.prog.classes.get(&ci.parent_name);
        }
        None
    }

    /// Ist `name` eine PROPERTY von `class_name`, und welche Methode liest
    /// (`lesen`) bzw. schreibt sie? `None` = keine PROPERTY (ein Feld oder eine
    /// Methode), `Some(None)` = PROPERTY ohne diesen Zugriff. Aus den beim
    /// Laden gerechneten Tabellen (`ClassInfo::prop_get/prop_set`) -- vorher
    /// je Zugriff die Kette abgesucht und `format!("__get_{}")` gebaut.
    #[inline(never)]
    fn property_methode(&self, class_name: &str, name: &str, lesen: bool) -> Option<Option<&'p Func>> {
        let ci = self.prog.classes.get(class_name)?;
        if !ci.props_kette { return None; }
        let lowered;
        let key: &str = if name.bytes().any(|b| b.is_ascii_uppercase()) {
            lowered = name.to_lowercase();
            &lowered
        } else { name };
        if !ci.props_alle.contains(key) { return None; }
        let tabelle = if lesen { &ci.prop_get } else { &ci.prop_set };
        Some(tabelle.get(key).and_then(|(klasse, methode)|
            self.prog.classes.get(klasse.as_str()).and_then(|c| c.methods.get(methode.as_str()))))
    }


    fn element_default(&self, type_name: &str) -> Value {
        if let Some(ci) = self.prog.classes.get(type_name) {
            if ci.is_struct {
                return self.allocate_instance(type_name);
            }
        }
        match type_name {
            "integer" => Value::Int(0),
            "float" => Value::Float(0.0),
            "string" => Value::str_rc(""),
            "boolean" => Value::Bool(false),
            "tuple" => Value::Tuple(Rc::new(vec![])),
            // Mathe-Typen bekommen ihr NEUTRALES Element statt NIL -- genauso,
            // wie INTEGER mit 0 und STRING mit "" anfaengt. Ohne das ist ein
            // frisches `DIM m AS MAT4` unbrauchbar (jede Rechnung darauf
            // scheitert an NIL), und ein `DIM mats[N] AS MAT4` laesst sich
            // nicht schrittweise fuellen. Eine Quelle fuer alle drei Wege
            // (global, lokal, Array-Element): model::neutrales_element.
            // `DIM a AS ARRAY OF INTEGER` ohne Groesse: ein LEERES Feld, das
            // mit ARRAY_PUSH waechst -- nicht NIL. Vorher meldete das erste
            // ARRAY_PUSH "erwartet ARRAY", und man musste wissen, dass erst
            // `a = []` kommen muss.
            other if other.starts_with("array:") => leeres_feld(other),
            // Ebenso `DIM tab AS MAP OF INTEGER`: eine LEERE MAP. Global,
            // lokal und in einer Schleife war das laengst so (DECLARE_*,
            // `__frisch`); nur ein Klassen- oder STRUCT-Feld kam hier vorbei
            // und blieb NIL, bis das erste `tab["k"] = 1` mit "Index-Zuweisung
            // an NIL" abbrach. Die Vorlage legt sie je Objekt neu an
            // (`teilbar` verneint MAPs).
            other if other.starts_with("map:") =>
                Value::Map(Rc::new(RefCell::new(DhMap::new(other["map:".len()..].to_string())))),
            other => crate::model::neutrales_element(other).unwrap_or(Value::Nil),
        }
    }

    /// `obj.name` lesen (LOAD_MEMBER): Feld ueber den Merkplatz, PROPERTY,
    /// gebundene Methode oder Namensraum.
    #[inline(never)]
    pub(crate) fn member_laden(&mut self, instr: &crate::model::Instr, name_wert: &Value, obj: Value) -> Result<Value, String> {
        if let Value::Instance(rc) = &obj {
            // Schneller Weg: dieselbe Lage wie beim letzten Mal -- dann ist es
            // ein Feld (eine PROPERTY kommt nie in den Merkplatz), und sein
            // Platz steht fest.
            let b = rc.borrow();
            let (lage, platz) = instr.merk.get();
            if std::ptr::eq(Rc::as_ptr(&b.layout), lage) { return Ok(b.fields[platz].clone()); }
        }
        let name_owned;
        let name: &str = match name_wert {
            Value::Str(s) => s,
            v => { name_owned = v.fmt(); &name_owned }
        };
        match &obj {
            Value::Nil => Err(format!("Zugriff auf '.{}' bei NIL-Referenz{}", name, NIL_NEW)),
            Value::Namespace(ns) => match ns.members.get(&name.to_lowercase()) {
                Some(v) => Ok(v.clone()),
                None => Err(format!("{} hat keinen Member '{}'", ns.name, name)),
            },
            Value::Instance(rc) => {
                let cn = rc.borrow().class_name.clone();
                if let Some(getter) = self.property_methode(&cn, name, true) {
                    let getter = getter
                        .ok_or_else(|| format!("Property '{}' in {} hat keinen Getter", name, cn))?;
                    return self.exec(getter, vec![], Some(obj.clone()));
                }
                let feld = { let b = rc.borrow(); feld_platz(instr, &b, name).map(|i| b.fields[i].clone()) };
                match feld {
                    Some(v) => Ok(v),
                    // Kein Feld -- aber vielleicht eine Methode. `obj.methode`
                    // OHNE Klammern ist eine an DIESE Instanz gebundene FUNCREF;
                    // mit Klammern laeuft der Aufruf ueber CALL_METHOD und kommt
                    // hier gar nicht an.
                    None => match self.resolve_method(&cn, name) {
                        Some(_) => Ok(Value::BoundMethod(Rc::new((
                            obj.clone(), Rc::from(name.to_lowercase().as_str()))))),
                        None => Err(format!("Feld '{}' existiert nicht in {}", name, cn)),
                    },
                }
            }
            _ => Err(format!("Zugriff auf '.{}' bei nicht-Objekt ({})", name, obj.type_name())),
        }
    }

    /// `obj.name = v` (STORE_MEMBER): Feld ueber den Merkplatz oder PROPERTY.
    #[inline(never)]
    pub(crate) fn member_setzen(&mut self, instr: &crate::model::Instr, name_wert: &Value, obj: Value, v: Value) -> Result<(), String> {
        let v = match &obj {
            Value::Instance(rc) => {
                // Schneller Weg wie beim Lesen, fuer einen Wert, der schon den
                // Typ des Feldes hat.
                let mut b = rc.borrow_mut();
                let (lage, platz) = instr.merk.get();
                if std::ptr::eq(Rc::as_ptr(&b.layout), lage) {
                    let passt = matches!((&v, b.layout.typen[platz].as_str()),
                        (Value::Int(_), "integer") | (Value::Float(_), "float")
                        | (Value::Str(_), "string") | (Value::Bool(_), "boolean")
                        | (_, "any") | (_, ""));
                    if passt { b.fields[platz] = v; return Ok(()); }
                }
                v
            }
            _ => v,
        };
        let name_owned;
        let name: &str = match name_wert {
            Value::Str(s) => s,
            x => { name_owned = x.fmt(); &name_owned }
        };
        match &obj {
            Value::Nil => Err(format!("Zuweisung an '.{}' bei NIL-Referenz{}", name, NIL_NEW)),
            Value::Instance(rc) => {
                let cn = rc.borrow().class_name.clone();
                if let Some(setter) = self.property_methode(&cn, name, false) {
                    let setter = setter
                        .ok_or_else(|| format!("Property '{}' in {} hat keinen Setter (read-only)", name, cn))?;
                    self.exec(setter, vec![v], Some(obj.clone()))?;
                    return Ok(());
                }
                // Ein borrow_mut, Coerce-Fast-Arm; der format!-Fehlerkontext
                // entsteht nur im langsamen Weg.
                let mut rcb = rc.borrow_mut();
                let i = feld_platz(instr, &rcb, name)
                    .ok_or_else(|| format!("Feld '{}' existiert nicht in {}", name, cn))?;
                let lage = rcb.layout.clone();
                let cv = match (&v, lage.typen[i].as_str()) {
                    (Value::Int(_), "integer") | (Value::Float(_), "float")
                    | (Value::Str(_), "string") | (Value::Bool(_), "boolean")
                    | (_, "any") | (_, "") => v,
                    (Value::Int(n), "float") => Value::Float(*n as f64),
                    (_, ty) => {
                        // Die Sperre VOR coerce abgeben: bei `k.naechster = k`
                        // ist der Wert dasselbe Objekt, und die Pruefung der
                        // Klasse (value::klasse_pruefen) liest es.
                        drop(rcb);
                        let cv = coerce(v, ty, &format!("Zuweisung an {}.{}", cn, name))?;
                        rc.borrow_mut().fields[i] = cv;
                        return Ok(());
                    }
                };
                rcb.fields[i] = cv;
                Ok(())
            }
            _ => Err(format!("Zuweisung an '.{}' bei nicht-Objekt ({})", name, obj.type_name())),
        }
    }

    /// `a + b` wie ADD (Zahlen direkt, sonst `addieren`).
    #[inline]
    pub(crate) fn plus(&mut self, a: Value, b: Value) -> R<Value> {
        match zahlen_addieren(&a, &b) {
            Some(r) => r,
            None => self.addieren(a, b),
        }
    }

    /// ADD_STORE_MEMBER: `obj.name = a + b`. Kennt der Merkplatz des Befehls
    /// das Feld schon (ein Feld, nie eine PROPERTY -- die kommt nicht in den
    /// Merkplatz) und haelt es noch den Text `a`, wird dort angehaengt. Sonst
    /// genau ADD + STORE_MEMBER; das erste Mal fuellt `member_setzen` den
    /// Merkplatz.
    #[inline(never)]
    pub(crate) fn mitglied_addieren(&mut self, instr: &crate::model::Instr, name_wert: &Value, obj: Value,
                                    a: Value, b: Value) -> R<()> {
        let a = match &obj {
            Value::Instance(rc) => {
                let mut ib = rc.borrow_mut();
                let ib = &mut *ib;
                let (lage, platz) = instr.merk.get();
                if std::ptr::eq(Rc::as_ptr(&ib.layout), lage) && text_platz_typ(&ib.layout.typen[platz]) {
                    match an_ort_anhaengen(&mut ib.fields[platz], a, &b) {
                        Ok(()) => return Ok(()),
                        Err(a) => a,
                    }
                } else { a }
            }
            _ => a,
        };
        let v = self.plus(a, b)?;
        self.member_setzen(instr, name_wert, obj, v)
    }

    /// ADD_STORE_FIELD: `text = a + b` in einer Methode (Feld von Self).
    /// Wie `mitglied_addieren`, sonst ADD + STORE_FIELD.
    #[inline(never)]
    fn feld_addieren(&mut self, instr: &crate::model::Instr, name_wert: &Value, self_obj: Option<&Value>,
                     a: Value, b: Value) -> R<()> {
        let name_owned;
        let name: &str = match name_wert {
            Value::Str(s) => s,
            v => { name_owned = v.fmt(); &name_owned }
        };
        let a = match self_obj {
            Some(Value::Instance(rc)) => {
                let mut ib = rc.borrow_mut();
                let ib = &mut *ib;
                match feld_platz(instr, ib, name) {
                    Some(i) if text_platz_typ(&ib.layout.typen[i]) => match an_ort_anhaengen(&mut ib.fields[i], a, &b) {
                        Ok(()) => return Ok(()),
                        Err(a) => a,
                    },
                    _ => a,
                }
            }
            _ => a,
        };
        let v = self.plus(a, b)?;
        // Ab hier wie STORE_FIELD.
        let s = self_obj.ok_or_else(|| format!("STORE_FIELD '{}' ausserhalb Methodenkontext", name))?;
        let Value::Instance(rc) = s else { return Err("STORE_FIELD: self ist keine Instanz".into()) };
        let langsam_ty = {
            let mut rcb = rc.borrow_mut();
            let i = feld_platz(instr, &rcb, name)
                .ok_or_else(|| format!("Feld '{}' existiert nicht", name))?;
            let lage = rcb.layout.clone();
            let ty = lage.typen[i].as_str();
            match (&v, ty) {
                (Value::Int(_), "integer") | (Value::Float(_), "float")
                | (Value::Str(_), "string") | (Value::Bool(_), "boolean")
                | (_, "any") | (_, "") => { rcb.fields[i] = v; None }
                (Value::Int(n), "float") => { rcb.fields[i] = Value::Float(*n as f64); None }
                _ => Some((i, ty.to_string(), v)),
            }
        };
        if let Some((i, ty, v)) = langsam_ty {
            let cv = coerce(v, &ty, &format!("Zuweisung an Feld {}", name))?;
            rc.borrow_mut().fields[i] = cv;
        }
        Ok(())
    }

    /// ADD_STORE_INDEX: Stapel `[.., c, i1..in, a, b]` -- `c[i..] = a + b`,
    /// an Ort und Stelle, wenn das Element bzw. der MAP-Eintrag den Text `a`
    /// noch haelt (`index_anhaengen`), sonst ADD + STORE_INDEX.
    #[inline(never)]
    fn index_addieren(&mut self, stack: &mut Vec<Value>, n: usize) -> R<()> {
        let b = vm_pop(stack)?;
        let a = vm_pop(stack)?;
        let split = stack.len().checked_sub(n).filter(|&s| s >= 1)
            .ok_or("Interner VM-Fehler: Stack-Underflow")?;
        let a = match index_anhaengen(&stack[split - 1], &stack[split..], a, &b) {
            Ok(()) => { stack.truncate(split - 1); return Ok(()); }
            Err(a) => a,
        };
        let v = self.plus(a, b)?;
        let idx = stack.split_off(split);
        let c = vm_pop(stack)?;
        store_index(&c, &idx, v)
    }

    #[inline(never)]
    fn allocate_instance(&self, class_name: &str) -> Value {
        match self.prog.classes.get(class_name) {
            Some(ci) => self.objekt_aus(self.vorlage(class_name, ci)),
            // Eine Klasse, die es nicht gibt, hat keine Felder. EINE geteilte
            // leere Lage: eine je Instanz wuerde freigegeben, und ihre Adresse
            // koennte eine spaetere erben -- ein Merkplatz traefe dann falsch.
            None => Value::Instance(Rc::new(RefCell::new(Instance {
                class_name: Rc::from(class_name),
                layout: LEERE_LAGE.with(|l| l.clone()),
                fields: Vec::new(),
            }))),
        }
    }

    /// Die Vorlage der Klasse `ci` (Schluessel `class_name`), beim ersten
    /// Mal gerechnet. `set` statt `get_or_init`: ein Feld mit Elementen eines
    /// STRUCT baut beim Rechnen die Vorlage einer ANDEREN Klasse, und eine
    /// Kette, die zurueckfuehrt, darf nicht in einer Panik enden.
    fn vorlage(&self, class_name: &str, ci: &'p ClassInfo) -> &'p ObjektVorlage {
        if let Some(v) = ci.vorlage.get() { return v; }
        let v = self.vorlage_bauen(class_name, ci);
        let _ = ci.vorlage.set(v);
        ci.vorlage.get().expect("Vorlage eben gesetzt")
    }

    /// Wie NEW frueher je Objekt: die Kette parent-first, ein Feld, das eine
    /// Unterklasse noch einmal deklariert, ueberschreibt den Wert auf
    /// demselben Platz.
    #[inline(never)]
    fn vorlage_bauen(&self, class_name: &str, ci: &'p ClassInfo) -> ObjektVorlage {
        let layout = ci.layout.clone();
        let mut werte: Vec<Value> = vec![Value::Nil; layout.typen.len()];
        let mut frisch: Vec<Option<Frisch>> = (0..layout.typen.len()).map(|_| None).collect();
        let mut chain: Vec<&ClassInfo> = Vec::new();
        let mut cur = Some(ci);
        while let Some(c) = cur {
            chain.push(c);
            cur = if c.parent_name.is_empty() { None } else { self.prog.classes.get(&c.parent_name) };
        }
        for c in chain.iter().rev() {
            for fd in &c.fields {
                let Some(&i) = layout.index.get(&fd.name) else { continue };
                let i = i as usize;
                frisch[i] = if fd.struct_bytes > 0 {
                    // Ein STRUCT ... LAYOUT C: je Objekt ein eigener Puffer.
                    Some(Frisch::Puffer(fd.struct_bytes))
                } else if !fd.array_dims.is_empty() {
                    let et = fd.type_name.clone();
                    let arr = DhArray::new(et.clone(), fd.array_dims.clone(), || self.element_default(&et))
                        .mit_unten(fd.array_unten.clone());
                    if arr.cells.iter().all(|v| teilbar(&v)) { Some(Frisch::FeldKopie(arr)) }
                    else { Some(Frisch::Feld(et, fd.array_dims.clone(), fd.array_unten.clone())) }
                } else if self.prog.classes.get(&fd.type_name).map(|c| c.is_struct).unwrap_or(false) {
                    Some(Frisch::Struktur(fd.type_name.clone()))
                } else {
                    let v = self.element_default(&fd.type_name);
                    if teilbar(&v) { werte[i] = v; None } else { Some(Frisch::Wert(fd.type_name.clone())) }
                };
            }
        }
        ObjektVorlage {
            klasse: Rc::from(class_name),
            layout,
            werte,
            frisch: frisch.into_iter().enumerate().filter_map(|(i, f)| f.map(|f| (i, f))).collect(),
            init: self.resolve_method(class_name, "init").map(|f| f as *const Func),
        }
    }

    /// Ein neues Objekt nach der Vorlage: die teilbaren Werte geklont, Felder,
    /// Puffer und Strukturen je Objekt neu.
    fn objekt_aus(&self, v: &ObjektVorlage) -> Value {
        let mut fields = v.werte.clone();
        for (i, f) in &v.frisch {
            fields[*i] = match f {
                Frisch::Puffer(n) => crate::builtins::neuer_buffer(vec![0u8; *n]),
                Frisch::FeldKopie(a) => Value::Array(Rc::new(RefCell::new(DhArray {
                    element_type: a.element_type.clone(),
                    dims: a.dims.clone(),
                    strides: a.strides.clone(),
                    cells: a.cells.clone(),
                    unten: a.unten.clone(),
                }))),
                Frisch::Feld(et, dims, unten) => Value::Array(Rc::new(RefCell::new(
                    DhArray::new(et.clone(), dims.clone(), || self.element_default(et)).mit_unten(unten.clone())))),
                Frisch::Struktur(k) => self.allocate_instance(k),
                Frisch::Wert(t) => self.element_default(t),
            };
        }
        Value::Instance(Rc::new(RefCell::new(Instance {
            class_name: v.klasse.clone(),
            layout: v.layout.clone(),
            fields,
        })))
    }

    /// User-Operator-Overloading. Liefert Some(Ergebnis) wenn LHS oder RHS
    /// eine Instanz mit der Operator-Methode ist.
    ///
    /// `commutative` steuert den RHS-Reverse-Fallback: fuer +/*/=/<> ist
    /// `b.method(a)` (b als Self, a als Argument) semantisch gleichwertig zu
    /// `a OP b`, weil diese Operatoren kommutativ sein sollen -- das ist der
    /// dokumentierte Mechanismus, der `5 + money` unterstuetzt. Review-Fund:
    /// derselbe Fallback lief FRUEHER auch fuer -, /, MOD, <, >, <=, >= --
    /// dort berechnet `b.method(a)` aber `b OP a`, nicht `a OP b` (z.B. bei
    /// `5 - m` wurde still `m - 5` ausgefuehrt). Reflektierte Operatoren
    /// (`__radd__` etc.) sind laut CLAUDE.md bewusst nicht unterstuetzt --
    /// fuer die nicht-kommutativen Faelle daher lieber `None` (-> normaler
    /// Typ-Fehler) als ein stillschweigend falsches Ergebnis.
    /// `a + b` fuer alle Arten -- der eine Weg fuer ADD und ADD_STORE_*.
    ///
    /// Zeichenketten: haelt niemand ausser `a` den linken Text (ein
    /// Zwischenergebnis wie in `"x" + y + z`, oder der Platz hat seinen Verweis
    /// in ADD_STORE_* abgegeben), wird an Ort und Stelle angehaengt. Vorher
    /// entstand bei jedem `+` eine neue Kopie, und `s = s + "x"` in einer
    /// Schleife war quadratisch (400 000 Mal: 7 s).
    #[inline(never)]
    fn addieren(&mut self, a: Value, b: Value) -> R<Value> {
        // Numerischer Fast-Path zuerst: Int/Float-Paare sind der Normalfall
        // in heissen Schleifen -- Modul-/User-Operator-Checks kosten dort nur
        // (Semantik unveraendert: weder Zahlen noch der Sonderfall-Pfad
        // ueberlappen sich).
        if let Some(r) = zahlen_addieren(&a, &b) { return r; }
        if let Some(r) = module_op('+', &a, &b) { return r; }
        if let Some(r) = self.user_op("__op_add__", &a, &b, true)? { return Ok(r); }
        if let Value::Str(mut links) = a {
            // ein Zeichenindex (textindex.rs) haelt ein Weak -- erst loslassen
            crate::textindex::loslassen(&links);
            match Rc::get_mut(&mut links) {
                Some(s) => {
                    match &b { Value::Str(r) => s.push_str(r), _ => s.push_str(&b.fmt()) }
                }
                None => {
                    let rechts: std::borrow::Cow<str> = match &b {
                        Value::Str(r) => std::borrow::Cow::Borrowed(r.as_str()),
                        _ => std::borrow::Cow::Owned(b.fmt()),
                    };
                    let mut s = String::with_capacity(links.len() + rechts.len());
                    s.push_str(&links);
                    s.push_str(&rechts);
                    links = Rc::new(s);
                }
            }
            return Ok(Value::Str(links));
        }
        if let Value::Str(r) = &b {
            let mut s = a.fmt();
            s.push_str(r);
            return Ok(Value::Str(Rc::new(s)));
        }
        if let (Value::Array(x), Value::Array(y)) = (&a, &b) {
            return felder_verbinden(x, y);
        }
        require_number(&a, &b, "+")?;
        nn_add(a, b)
    }

    /// Einen eingebauten Befehl ausfuehren (CALL_BUILTIN).
    ///
    /// Die Befehle liegen in Familien (`try_db`, `try_gui`, ... und zuletzt
    /// die reinen in builtins.rs), und ein Name wurde bei JEDEM Aufruf der
    /// Reihe nach durch alle gereicht -- bei `ABS` 28 Absagen, bevor der
    /// richtige `match` kam. `merk` ist der Platz der Aufrufstelle: dort steht
    /// nach dem ersten Mal, welche Familie geantwortet hat, und sie wird
    /// zuerst gefragt. Sagt sie ab, laeuft die ganze Kette wie bisher.
    ///
    /// **Das ist nur richtig, weil die Familien am NAMEN entscheiden.** Die
    /// eine Ausnahme ist `SORT`: mit einer FUNCREF sortiert `try_array_hof`,
    /// sonst der reine Befehl -- an derselben Stelle kann beides kommen, also
    /// merkt sich `SORT` nichts. Wer eine Familie baut, die je nach ARGUMENT
    /// absagt, obwohl eine spaetere denselben Namen kennt, muss hier dasselbe
    /// tun.
    /// `obj.methode(args)` wie CALL_METHOD (fuer den Maschinencode, M4 Schritt
    /// 7): Methode einer Instanz ueber den Merkplatz, Coroutine als Handle,
    /// sonst die Methoden von Text/Feld/MAP. Die VM selbst nimmt weiter ihren
    /// schnellen Weg im Befehl (Argumente direkt vom Stapel).
    /// CALL_METHOD in den Maschinencode (M4 Schritt 7b), ausserhalb von
    /// `dispatch`, damit dessen Rahmen nicht waechst.
    #[cfg(feature = "jit")]
    #[inline(never)]
    fn methode_maschinencode(&self, m: &Func, rc: &Rc<RefCell<crate::value::Instance>>, args: &[Value]) -> Option<Value> {
        let j = self.jit.as_ref()?;
        let lage = Rc::as_ptr(&rc.borrow().layout);
        j.methode(lage, m, Rc::as_ptr(rc), args, self.depth, MAX_CALL_DEPTH, &self.global_slots)
    }

    /// PRINT mit seinen Werten -- dieselbe Stelle fuer den Befehl der VM und
    /// fuer den Maschinencode (M4 Schritt 10, `w_drucken`).
    pub(crate) fn drucken(&mut self, arg: &crate::model::Arg, items: &[Value]) {
        let l = arg.list();
        let newline = arg_truthy(&l[1]);
        for (i, it) in items.iter().enumerate() {
            if i > 0 && l[i + 1].str() != ";" { self.out.push(' '); }
            self.out.push_str(&it.fmt());
        }
        if newline { self.out.push('\n'); }
        // Zeilenweise hinaus, wenn jemand live mitliest (die IDE
        // ueber PROCESS_START setzt DHRT_LIVE): an einer Leitung
        // waere stdout sonst blockgepuffert, und die Ausgabe kaeme
        // erst am Ende auf einmal.
        if newline && (live_ausgabe() || ausgabe_terminal() || self.out_zeit.elapsed() >= std::time::Duration::from_millis(50)) { self.flush_out(); }
    }

    /// Eine Funktion des Programms rufen, fuer den Maschinencode (M4 Schritt
    /// 15): derselbe Weg wie CALL_USER ohne BYREF.
    #[cfg(feature = "jit")]
    pub(crate) fn funktion_rufen(&mut self, idx: usize, args: Vec<Value>) -> R<Value> {
        let g: &'p Func = self.prog.functions.get(idx).ok_or("Unbekannte Funktion")?;
        if g.is_coroutine { return Ok(make_coro(g, args, None)); }
        let ret = self.exec(g, args, None)?;
        Ok(if g.is_sub { Value::Nil } else { ret })
    }

    pub(crate) fn methode_rufen(&mut self, instr: &'p crate::model::Instr, method: &str, obj: Value, margs: Vec<Value>) -> R<Value> {
        match &obj {
            Value::Instance(rc) => {
                let (lage, f_p) = instr.merk.get();
                let getroffen = std::ptr::eq(Rc::as_ptr(&rc.borrow().layout), lage);
                let m: &'p Func = if getroffen {
                    unsafe { &*(f_p as *const Func) }
                } else {
                    let cn = rc.borrow().class_name.clone();
                    let m = self.resolve_method(&cn, method)
                        .ok_or_else(|| format!("Methode '{}' existiert nicht in {}", method, cn))?;
                    instr.merk.set((Rc::as_ptr(&rc.borrow().layout), m as *const Func as usize));
                    m
                };
                if m.is_coroutine { return Ok(make_coro(m, margs, Some(obj.clone()))); }
                let ret = self.exec(m, margs, Some(obj.clone()))?;
                Ok(if m.is_sub { Value::Nil } else { ret })
            }
            Value::Nil => Err(format!("Methodenaufruf '.{}' bei NIL-Referenz{}", method, NIL_NEW)),
            _ => {
                let kind = container_kind(&obj).ok_or_else(|| format!("Methodenaufruf '.{}' bei nicht-Objekt ({})", method, obj.type_name()))?;
                let bi = container_method(kind, &method.to_lowercase())
                    .ok_or_else(|| format!("{} hat keine Methode '{}'", kind.to_uppercase(), method))?;
                let mut call_args = Vec::with_capacity(margs.len() + 1);
                call_args.push(obj.clone());
                call_args.extend(margs);
                match safe_call_builtin(bi, &call_args) {
                    Some(Ok(v)) => Ok(v),
                    Some(Err(e)) => Err(e),
                    None => Err(unknown_builtin_msg(bi)),
                }
            }
        }
    }

    pub(crate) fn builtin_rufen(&mut self, name: &str, a: &[Value], merk: &std::cell::Cell<u8>) -> R<Value> {
        let f = merk.get();
        if f != 0 {
            if let Some(v) = self.familie_rufen(f, name, a)? { return Ok(v); }
        }
        for f in 1..=BUILTIN_FAMILIEN {
            if let Some(v) = self.familie_rufen(f, name, a)? {
                if name != "sort" { merk.set(f); }
                return Ok(v);
            }
        }
        Err(unknown_builtin_msg(name))
    }

    /// Eine Familie fragen; die Reihenfolge der Nummern IST die Reihenfolge,
    /// in der sie gefragt werden (erst typtest/array_hof, zuletzt die reinen).
    #[inline]
    fn familie_rufen(&mut self, f: u8, name: &str, a: &[Value]) -> R<Option<Value>> {
        match f {
            1 => self.try_typtest(name, a),
            2 => self.try_array_hof(name, a),
            3 => self.try_scene(name, a),
            4 => self.try_coro(name, a),
            5 => self.try_timer(name, a),
            6 => self.try_zeit(name, a),
            7 => self.try_os(name, a),
            8 => self.try_hintergrund(name, a),
            9 => self.try_db(name, a),
            10 => self.try_net(name, a),
            11 => self.try_midi(name, a),
            12 => self.try_mqtt(name, a),
            13 => self.try_httpd(name, a),
            14 => self.try_pdf(name, a),
            15 => self.try_xlsx(name, a),
            16 => self.try_smtp(name, a),
            17 => self.try_geld(name, a),
            18 => self.try_html(name, a),
            19 => self.try_cloud(name, a),
            20 => self.try_serial(name, a),
            21 => self.try_firmata(name, a),
            22 => self.try_usb(name, a),
            23 => self.try_wifi(name, a),
            24 => self.try_bt(name, a),
            25 => self.try_video(name, a),
            26 => self.try_gui(name, a),
            27 => self.try_graphics_app(name, a),
            28 => if name == "__ffi" { crate::ffi::vm_setzen(self); crate::ffi::rufen(a).map(Some) }
                  else { crate::ffi::zeiger_befehl(name, a).transpose() },
            29 => self.try_pins(name, a),
            30 => self.try_mikrofon(name, a),
            31 => self.try_webcam(name, a),
            32 => self.try_netzdienste(name, a),
            33 => self.try_geraete(name, a),
            _ => match safe_call_builtin(name, a) {
                Some(Ok(v)) => Ok(Some(v)),
                Some(Err(e)) if e.starts_with("__UNKNOWN_BUILTIN__:") => Err(unknown_builtin_msg(name)),
                Some(Err(e)) => Err(e),
                None => Err(unknown_builtin_msg(name)),
            },
        }
    }

    #[inline(never)]
    fn user_op(&mut self, method: &str, a: &Value, b: &Value, commutative: bool) -> R<Option<Value>> {
        if let Value::Instance(rc) = a {
            let cn = rc.borrow().class_name.clone();
            if let Some(m) = self.resolve_method(&cn, method) {
                return Ok(Some(self.exec(m, vec![b.clone()], Some(a.clone()))?));
            }
        }
        if commutative {
            if let Value::Instance(rc) = b {
                let cn = rc.borrow().class_name.clone();
                if let Some(m) = self.resolve_method(&cn, method) {
                    return Ok(Some(self.exec(m, vec![a.clone()], Some(b.clone()))?));
                }
            }
        }
        Ok(None)
    }

    // ---------------------------------------------------------------- exec
    fn exec<A>(&mut self, fn_: &'p Func, args: A, self_obj: Option<Value>) -> R<Value>
    where A: IntoIterator<Item = Value>, A::IntoIter: ExactSizeIterator {
        // Call-Tiefe fuer den Debugger (Step over/into/out). Inkrement pro Frame;
        // garantiert dekrementiert (auch bei Fehler/Return) via Wrapper.
        // Review-Fund: ohne Obergrenze rekursiert exec->run_frame->dispatch->exec
        // auf dem NATIVEN Stack -- eine ausufernde DH-Rekursion (oder eine
        // Property/Operator, die sich selbst aufruft) fuehrte zu einem
        // Stack-Overflow-Absturz statt einem fangbaren DHRuntimeError.
        self.depth += 1;
        if self.depth > MAX_CALL_DEPTH {
            self.depth -= 1;
            return Err(format!("Maximale Aufruftiefe ({}) ueberschritten -- unendliche Rekursion?", MAX_CALL_DEPTH));
        }
        let line = self.cur_line;
        if let Some(d) = self.dbg.as_mut() { d.stapel.push((fn_.name.clone(), line)); }
        let r = self.exec_inner(fn_, args, self_obj);
        if let Some(d) = self.dbg.as_mut() { d.stapel.pop(); }
        self.depth -= 1;
        r
    }

    fn exec_inner<A>(&mut self, fn_: &'p Func, args: A, self_obj: Option<Value>) -> R<Value>
    where A: IntoIterator<Item = Value>, A::IntoIter: ExactSizeIterator {
        let lbuf = self.pool_locals.pop().unwrap_or_default();
        let mut locals = bind_params(fn_, args.into_iter(), lbuf)?;
        let mut stack: Vec<Value> = self.pool_stacks.pop().unwrap_or_else(|| Vec::with_capacity(16));
        let mut ip: usize = 0;
        let mut try_handlers: Vec<(usize, usize)> = Vec::new();
        let step = self.run_frame(fn_, &mut locals, &mut stack, &mut ip, &mut try_handlers, self_obj.as_ref());
        locals.clear();
        stack.clear();
        if self.pool_locals.len() < 64 { self.pool_locals.push(locals); }
        if self.pool_stacks.len() < 64 { self.pool_stacks.push(stack); }
        match step? {
            Step::Return(v) => Ok(v),
            // Eine normale Funktion enthaelt kein YIELD (sonst waere sie eine
            // Coroutine und wuerde nicht via exec ausgefuehrt).
            Step::Yield(_) => Err("YIELD ausserhalb einer Coroutine".into()),
        }
    }

    /// Wie `exec`, liefert aber zusaetzlich die finalen Werte der BYREF-Param-
    /// Slots (in Param-Reihenfolge). Nur der direkte CALL_USER-Pfad nutzt das --
    /// dort kennt der Compiler die Signatur statisch und emittiert das Write-Back.
    fn exec_byref<A>(&mut self, fn_: &'p Func, args: A, self_obj: Option<Value>)
        -> R<(Value, Vec<Value>)>
    where A: IntoIterator<Item = Value>, A::IntoIter: ExactSizeIterator {
        self.depth += 1;
        if self.depth > MAX_CALL_DEPTH {
            self.depth -= 1;
            return Err(format!("Maximale Aufruftiefe ({}) ueberschritten -- unendliche Rekursion?", MAX_CALL_DEPTH));
        }
        let line = self.cur_line;
        if let Some(d) = self.dbg.as_mut() { d.stapel.push((fn_.name.clone(), line)); }
        let r = self.exec_byref_inner(fn_, args, self_obj);
        if let Some(d) = self.dbg.as_mut() { d.stapel.pop(); }
        self.depth -= 1;
        r
    }

    fn exec_byref_inner<A>(&mut self, fn_: &'p Func, args: A, self_obj: Option<Value>)
        -> R<(Value, Vec<Value>)>
    where A: IntoIterator<Item = Value>, A::IntoIter: ExactSizeIterator {
        let lbuf = self.pool_locals.pop().unwrap_or_default();
        let mut locals = bind_params(fn_, args.into_iter(), lbuf)?;
        let mut stack: Vec<Value> = self.pool_stacks.pop().unwrap_or_else(|| Vec::with_capacity(16));
        let mut ip: usize = 0;
        let mut try_handlers: Vec<(usize, usize)> = Vec::new();
        let step = self.run_frame(fn_, &mut locals, &mut stack, &mut ip, &mut try_handlers, self_obj.as_ref());
        // Review-Fund: applied `?` sofort auf `step` (statt wie exec_inner erst
        // zu binden), sodass bei einem Fehler `locals`/`stack` verworfen statt
        // recycelt wurden -- genau im Fall (BYREF-Call, z.B. in einer
        // TRY/CATCH-Retry-Schleife), in dem das Pooling am meisten bringt.
        // Finale Werte der BYREF-Param-Slots (in Param-Reihenfolge) auslesen --
        // vor dem Recycling, damit sie danach noch verfuegbar sind.
        let mut byref_vals = Vec::new();
        for (i, &is_br) in fn_.param_byref.iter().enumerate() {
            if is_br {
                byref_vals.push(locals.get(i).cloned().unwrap_or(Value::Nil));
            }
        }
        locals.clear();
        stack.clear();
        if self.pool_locals.len() < 64 { self.pool_locals.push(locals); }
        if self.pool_stacks.len() < 64 { self.pool_stacks.push(stack); }
        match step? {
            Step::Return(v) => Ok((v, byref_vals)),
            Step::Yield(_) => Err("YIELD ausserhalb einer Coroutine".into()),
        }
    }

    /// Treibt den Frame durch die Dispatch-Schleife inkl. TRY/CATCH-Unwinding,
    /// bis ein RETURN/HALT (Step::Return) oder ein YIELD (Step::Yield) faellt.
    fn run_frame(
        &mut self,
        fn_: &'p Func,
        locals: &mut Vec<Value>,
        stack: &mut Vec<Value>,
        ip: &mut usize,
        try_handlers: &mut Vec<(usize, usize)>,
        self_obj: Option<&Value>,
    ) -> R<Step> {
        // Rahmen oberhalb dieser Basis gehoeren zu Aufrufen, die `dispatch`
        // in diesem run_frame selbst begonnen hat (siehe `Vm::rahmen`).
        let basis = self.rahmen.len();
        loop {
            let akt = match self.rahmen.last() { Some(r) if self.rahmen.len() > basis => r.gerufen, _ => fn_ };
            match self.dispatch(akt, locals, stack, ip, try_handlers, self_obj, basis) {
                Ok(step) => return Ok(step),
                // Debugger-Abbruch (`stop`), Profiler-Stop-Signal und EXIT(code)
                // duerfen NICHT von TRY/CATCH gefangen werden -- unbedingt
                // durchreichen. Entscheidend ist das Flag, NICHT der Fehlertext:
                // ein DH-Programm mit `THROW "__DEBUG_STOP__"` erzeugt zwar
                // denselben String, setzt aber keines der Flags (Review-Fund).
                Err(e) if self.debug_stop_flag || self.profile_stop_flag
                          || self.exit_code.is_some() => {
                    while self.rahmen.len() > basis { self.rahmen_verlassen(locals, stack, ip, try_handlers); }
                    return Err(e);
                }
                Err(e) => {
                    let mut e = e;
                    // Die Funktion, in der der Fehler FIEL -- nicht die, mit
                    // der dispatch betreten wurde; es kann Rahmen gewechselt
                    // haben.
                    let fn_ = match self.rahmen.last() { Some(r) if self.rahmen.len() > basis => r.gerufen, _ => fn_ };
                    // Quell-Zeile lazy ermitteln: ip zeigt HINTER die
                    // fehlgeschlagene Instruktion. Nur der innerste Frame
                    // (Fehler-Ursprung) setzt sie; aeussere Frames sehen das
                    // Flag und lassen die innerste Zeile stehen.
                    if self.rethrow {
                        // Von FIN_END weitergeworfen: alle Angaben gehoeren
                        // weiterhin der urspruenglichen Fehlerstelle, hier ist
                        // also NICHTS zu aktualisieren.
                        self.rethrow = false;
                        self.err_line_set = true;
                    } else if !self.err_line_set {
                        if let Some(&ln) = fn_.lines.get(ip.saturating_sub(1)) {
                            if ln != 0 { self.cur_line = ln; }
                        }
                        self.err_line_set = true;
                        // Den Namen der Variable bzw. Funktion in die Meldung
                        // ("Zuweisung an 'rex'" statt "Zuweisung an global").
                        // Hier und nicht an den Speicherstellen in dispatch:
                        // dort machte der Fehlerzweig gemessen
                        // tools/tempo/teilchen.dh 9 % langsamer (Codelage).
                        e = self.fehler_benennen(e, fn_, ip.saturating_sub(1));
                        // WP F: Hier -- und nur hier -- sieht die VM einen
                        // FRISCHEN Fehler. Also der richtige Ort, um Fundstelle
                        // und Code fuer ERROR_LINE()/ERROR_CODE$() festzuhalten.
                        // Ein Fehler, der nicht von THROW kam, loescht den Code;
                        // sonst klebte er von einem frueheren THROW noch an.
                        self.error_line = self.cur_line;
                        // Unter dem Debugger: die Fehlerstelle festhalten, damit
                        // er dort anhalten kann, falls kein CATCH den Fehler nimmt.
                        if self.dbg.is_some() {
                            let stapel = self.dbg.as_ref().map(|d| d.stapel.clone()).unwrap_or_default();
                            let stand = FehlerStand { fn_: fn_ as *const Func, locals: locals.clone(),
                                                      stapel, zeile: self.cur_line };
                            if let Some(d) = self.dbg.as_mut() { d.fehler_stand = Some(stand); }
                        }
                        self.error_code = if self.throw_active {
                            std::mem::take(&mut self.throw_code)
                        } else {
                            String::new()
                        };
                        self.throw_active = false;
                    }
                    // Erst der innerste Rahmen, dann nach aussen -- wie frueher,
                    // als jeder Aufruf sein eigenes run_frame hatte.
                    loop {
                        match try_handlers.pop() {
                            Some((target, depth)) => {
                                self.err_line_set = false;   // Fehler konsumiert (CATCH)
                                if let Some(d) = self.dbg.as_mut() { d.fehler_stand = None; }
                                stack.truncate(depth);
                                stack.push(Value::str_rc(&e));
                                *ip = target;
                                break;
                            }
                            None if self.rahmen.len() > basis => {
                                self.rahmen_verlassen(locals, stack, ip, try_handlers);
                            }
                            None => return Err(e),
                        }
                    }
                }
            }
        }
    }

    /// Ein Aufruf ueber einen WERT (FUNCREF, gebundene Methode, Lambda): an
    /// der Aufrufstelle steht kein Name, den `fehler_benennen` lesen koennte --
    /// darum benennt CALL_VALUE den Parameter selbst, solange es die gerufene
    /// Funktion noch kennt. Nur im Fehlerfall.
    #[cold]
    #[inline(never)]
    fn param_von(e: String, tgt: &Func) -> String {
        let Some(rest) = e.strip_prefix("Parameter #") else { return e };
        let (zahl, rest) = rest.split_at(rest.find(':').unwrap_or(rest.len()));
        let Ok(n) = zahl.parse::<usize>() else { return e };
        let Some(p) = tgt.local_names.get(n) else { return e };
        if tgt.name.starts_with("__lambda") {
            return format!("Parameter '{}' einer Lambda-Funktion{}", p, rest);
        }
        format!("Parameter '{}' von {}{}", p, crate::namensraum::lesbar_text(&tgt.name), rest)
    }

    /// Eine Meldung von `coerce` um den Namen ergaenzen, den der fehlgeschlagene
    /// Befehl kennt (lokaler/globaler Platz, Funktion bei RETURN). Nur fuer
    /// einen frischen Fehler (`run_frame`).
    #[cold]
    #[inline(never)]
    fn fehler_benennen(&self, e: String, fn_: &Func, i: usize) -> String {
        let Some(ins) = fn_.code.get(i) else { return e };
        // "Parameter #n: ..." aus bind_params: der aufrufende Befehl nennt die
        // Funktion oder Methode, deren n-ter Parameter gemeint ist.
        if let Some(rest) = e.strip_prefix("Parameter #") {
            let (zahl, rest) = rest.split_at(rest.find(':').unwrap_or(rest.len()));
            let Ok(n) = zahl.parse::<usize>() else { return e };
            let wer = match &ins.arg { Arg::Call(name, _, _) => Some(name.to_string()), _ => None };
            let pname = wer.as_ref().and_then(|w| {
                if let Some(&fi) = self.prog.fn_index.get(w) {
                    return self.prog.functions.get(fi).and_then(|f| f.local_names.get(n).cloned());
                }
                // Eine Methode: alle Klassen, die sie haben, muessen den
                // Parameter gleich nennen -- sonst bleibt es bei der Nummer.
                let mut namen = self.prog.classes.values()
                    .filter_map(|c| c.methods.get(w.as_str()))
                    .filter_map(|m| m.local_names.get(n).cloned());
                let erster = namen.next()?;
                if namen.all(|x| x == erster) { Some(erster) } else { None }
            });
            return match (pname, wer) {
                (Some(p), Some(w)) => format!("Parameter '{}' von {}{}", p, crate::namensraum::lesbar_text(&w), rest),
                _ => format!("Parameter {}{}", n + 1, rest),
            };
        }
        match ins.op {
            op::STORE_LOCAL | op::ADD_STORE_LOCAL =>
                mit_namen(e, "Lokale Variable", &zuweisung_an(fn_.local_names.get(ins.arg.as_usize()))),
            op::STORE_GLOBAL_SLOT | op::ADD_STORE_GLOBAL_SLOT =>
                mit_namen(e, "Zuweisung an global", &zuweisung_an(self.prog.global_names.get(ins.arg.as_usize()))),
            op::RETURN =>
                mit_namen(e, "RETURN", &format!("RETURN von {}", crate::namensraum::lesbar_text(&fn_.name))),
            _ => e,
        }
    }

    /// Setzt eine Coroutine fort (`send` wird der YIELD-Ausdruck im Body).
    /// Liefert den naechsten YIELD-Wert bzw. den RETURN-Wert beim Ende.
    fn coro_resume(&mut self, co: &Rc<RefCell<CoroState>>, send: Value) -> R<Value> {
        if co.borrow().done {
            return Err("CORO_RESUME/SEND auf bereits beendeter Coroutine".into());
        }
        // Review-Fund: re-entranter Resume (die Coroutine ruft sich selbst
        // oder eine Kette anderer Coroutinen resumt sie zurueck) faende sonst
        // ihren Frame per mem::take() bereits geleert vor -- klarer Fehler
        // statt eines Index-Out-of-Bounds-Panics.
        if co.borrow().running {
            return Err("CORO_RESUME/SEND: Coroutine ist bereits aktiv (re-entranter Aufruf)".into());
        }
        co.borrow_mut().running = true;
        // Func-Zeiger ist fuer 'p (Programmlaufzeit) gueltig -- siehe CoroState.
        let ptr = co.borrow().fn_ptr;
        let fn_: &'p Func = unsafe { &*ptr };
        let self_obj = co.borrow().self_obj.clone();
        let started = co.borrow().started;

        let mut locals: Vec<Value>;
        let mut stack: Vec<Value>;
        let mut ip: usize;
        let mut try_handlers: Vec<(usize, usize)>;
        if !started {
            let args = std::mem::take(&mut co.borrow_mut().args);
            locals = bind_params(fn_, args.into_iter(), Vec::new())?;
            stack = Vec::with_capacity(16);
            ip = 0;
            try_handlers = Vec::new();
            co.borrow_mut().started = true;
        } else {
            let mut c = co.borrow_mut();
            locals = std::mem::take(&mut c.locals);
            stack = std::mem::take(&mut c.stack);
            ip = c.ip;
            try_handlers = std::mem::take(&mut c.try_handlers);
            drop(c);
            // Der YIELD-Ausdruck (`x = YIELD ...`) liefert den Sende-Wert.
            stack.push(send);
        }

        match self.run_frame(fn_, &mut locals, &mut stack, &mut ip, &mut try_handlers, self_obj.as_ref()) {
            Ok(Step::Yield(v)) => {
                let mut c = co.borrow_mut();
                c.locals = locals;
                c.stack = stack;
                c.ip = ip;
                c.try_handlers = try_handlers;
                c.running = false;
                Ok(v)
            }
            Ok(Step::Return(v)) => {
                let mut c = co.borrow_mut();
                c.done = true;
                c.running = false;
                c.result = v.clone();
                Ok(v)
            }
            Err(e) => {
                let mut c = co.borrow_mut();
                c.done = true;
                c.running = false;
                // Review-Fund: `result` blieb Nil, ohne dass CORO_RESULT einen
                // Hinweis auf den Fehlerfall geben konnte -- den Fehlertext
                // als Ersatzwert ablegen, statt stillschweigend Nil zu lassen.
                c.result = Value::str_rc(&e);
                drop(c);
                Err(e)
            }
        }
    }

    /// Eager: treibt die Coroutine bis zum Ende und sammelt alle YIELD-Werte
    /// (RETURN-Wert nicht enthalten). Fuer FOR EACH / Comprehensions.
    fn coro_drain(&mut self, co: &Rc<RefCell<CoroState>>) -> R<Vec<Value>> {
        let mut out = Vec::new();
        loop {
            if co.borrow().done {
                break;
            }
            let v = self.coro_resume(co, Value::Nil)?;
            if co.borrow().done {
                break; // letzter Resume hat beendet -> v ist der RETURN-Wert
            }
            out.push(v);
        }
        Ok(out)
    }

    /// CORO_*-Builtins (brauchen VM-State -> nicht in builtins.rs).
    /// Array-Higher-Order-Funktionen, die die VM brauchen (User-FUNCREF rufen).
    /// Aktuell nur `SORT(arr, comparator)`. Andere SORT-Formen macht builtins.rs.
    fn try_array_hof(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if name == "sort" && a.len() == 2 {
            if matches!(&a[1], Value::FuncRef(_) | Value::BoundMethod(_) | Value::Closure(_)) {
                return Ok(Some(self.sort_with_comparator(&a[0], &a[1])?));
            }
        }
        if matches!(name, "array_map" | "array_filter" | "array_reduce" | "array_find") {
            return self.array_mit_funktion(name, a).map(Some);
        }
        Ok(None)
    }

    /// ARRAY_MAP / ARRAY_FILTER / ARRAY_REDUCE / ARRAY_FIND: die Funktion
    /// (FUNCREF, gebundene Methode oder Lambda) je Element. Das Feld bleibt
    /// unveraendert -- MAP und FILTER liefern ein neues.
    fn array_mit_funktion(&mut self, name: &str, a: &[Value]) -> R<Value> {
        let gross = name.to_uppercase();
        let (soll, form) = match name {
            "array_reduce" => (3, "ARRAY_REDUCE(feld, funktion, start)"),
            "array_map" => (2, "ARRAY_MAP(feld, funktion)"),
            "array_filter" => (2, "ARRAY_FILTER(feld, funktion)"),
            _ => (2, "ARRAY_FIND(feld, funktion)"),
        };
        if a.len() != soll {
            return Err(format!("{}: erwartet {} Argumente, erhalten {} -- Aufruf: {}", gross, soll, a.len(), form));
        }
        let arr = match &a[0] {
            Value::Array(x) => x.clone(),
            andere => return Err(format!("{}: erwartet ARRAY, erhalten {}", gross, andere.type_name())),
        };
        if arr.borrow().dims.len() != 1 {
            return Err(format!("{}: nur eindimensionale Felder", gross));
        }
        let cb = crate::value::Rueckruf::aus_wert(&a[1]).ok_or_else(|| format!(
            "{}: erwartet eine Funktion als zweites Argument, erhalten {} -- z.B. {}(feld, FUNCTION(x) x * 2)",
            gross, a[1].type_name(), gross))?;
        let (f, empf) = self.rueckruf_aufloesen(&cb, &gross)?;
        let stellen = if name == "array_reduce" { 2 } else { 1 };
        if let Some(n) = Self::rueckruf_stellen(&cb, f) {
            if n != stellen {
                return Err(format!("{}: die Funktion nimmt {} Wert(e), {} gibt ihr {}{}", gross, n, gross, stellen,
                    if name == "array_reduce" { " (bisheriges Ergebnis, Element)" } else { " (das Element)" }));
            }
        }
        // Werte vorher herausziehen: die Funktion darf das Feld anfassen.
        let werte: Vec<Value> = arr.borrow().cells.to_values();
        match name {
            "array_map" => {
                let mut aus = Vec::with_capacity(werte.len());
                for v in werte { aus.push(self.rueckruf_exec(&cb, f, &empf, vec![v])?); }
                // Dieselbe Form wie das Feld: Platz i des Ergebnisses gehoert zu
                // Platz i des Feldes, also auch dieselbe Untergrenze.
                let neu = array_literal(aus);
                if let Value::Array(n) = &neu { n.borrow_mut().unten = arr.borrow().unten.clone(); }
                Ok(neu)
            }
            "array_filter" => {
                let mut aus = Vec::new();
                for v in werte {
                    match self.rueckruf_exec(&cb, f, &empf, vec![v.clone()])? {
                        Value::Bool(true) => aus.push(v),
                        Value::Bool(false) => {}
                        andere => return Err(format!(
                            "ARRAY_FILTER: die Funktion muss TRUE oder FALSE liefern, lieferte {} -- ein Vergleich wie x > 0 ist ein BOOLEAN",
                            andere.type_name())),
                    }
                }
                // Der Elementtyp bleibt der des Feldes (auch wenn nichts uebrig ist).
                let typ = arr.borrow().element_type.clone();
                let mut neu = DhArray::new(typ.clone(), vec![aus.len() as i64], || self.element_default(&typ));
                for (i, v) in aus.into_iter().enumerate() { neu.cells.set(i, v); }
                Ok(Value::Array(Rc::new(RefCell::new(neu))))
            }
            "array_reduce" => {
                let mut akku = a[2].clone();
                for v in werte { akku = self.rueckruf_exec(&cb, f, &empf, vec![akku, v])?; }
                Ok(akku)
            }
            _ => {
                let lo = arr.borrow().unten_von(0);
                for (i, v) in werte.into_iter().enumerate() {
                    match self.rueckruf_exec(&cb, f, &empf, vec![v])? {
                        Value::Bool(true) => return Ok(Value::Int(lo + i as i64)),
                        Value::Bool(false) => {}
                        andere => return Err(format!(
                            "ARRAY_FIND: die Funktion muss TRUE oder FALSE liefern, lieferte {}", andere.type_name())),
                    }
                }
                Ok(Value::Int(lo - 1))
            }
        }
    }

    /// SORT(arr, comparator-FUNCREF): stabil sortieren, wobei `comparator(x, y)`
    /// eine User-Funktion ist, die <0/0/>0 liefert (wie ein C-qsort-Comparator).
    fn sort_with_comparator(&mut self, arr_v: &Value, cmp: &Value) -> R<Value> {
        use std::cmp::Ordering;
        let arr = match arr_v {
            Value::Array(x) => x.clone(),
            _ => return Err("SORT erwartet ARRAY".to_string()),
        };
        let cb = crate::value::Rueckruf::aus_wert(cmp)
            .ok_or_else(|| "SORT: Comparator muss FUNCREF sein".to_string())?;
        if arr.borrow().dims.len() != 1 {
            return Err("SORT: nur 1D-Arrays".to_string());
        }
        let (func, empf) = self.rueckruf_aufloesen(&cb, "SORT")?;
        // Werte herausziehen -> kein Array-Borrow waehrend der Comparator laeuft.
        let mut vals: Vec<Value> = arr.borrow().cells.to_values();
        let mut error: Option<String> = None;
        vals.sort_by(|x, y| {
            if error.is_some() { return Ordering::Equal; }
            match self.rueckruf_exec(&cb, func, &empf, vec![x.clone(), y.clone()]) {
                Ok(Value::Int(i)) => i.cmp(&0),
                Ok(Value::Float(f)) => f.partial_cmp(&0.0).unwrap_or(Ordering::Equal),
                Ok(other) => {
                    error = Some(format!("SORT: Comparator muss INTEGER liefern, erhielt {}", other.type_name()));
                    Ordering::Equal
                }
                Err(e) => { error = Some(e); Ordering::Equal }
            }
        });
        if let Some(e) = error { return Err(e); }
        {
            let mut ab = arr.borrow_mut();
            for (i, v) in vals.into_iter().enumerate() { ab.cells.set(i, v); }
        }
        Ok(Value::Nil)
    }

    fn try_coro(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        // Builtin-Namen liegen im .dhc lowercase vor.
        if !name.starts_with("coro_") && name != "__comp_iter" { return Ok(None); }
        // Review-Fund: alle fuenf CORO_*-Arme indizierten `a[0]`/`a[1]` ohne
        // Arity-Pruefung (diese Namen tauchen in keiner Compiler-Arity-Tabelle
        // auf) -- `CORO_RESUME()` oder `CORO_SEND(c)` kompilierten anstandslos
        // und paniked dann mit einem Index-Out-of-Bounds statt eines
        // DHRuntimeError.
        let need = |n: usize, fname: &str| -> R<()> {
            if a.len() != n {
                Err(format!("{}: erwartet {} Argument(e), erhalten {}", fname, n, a.len()))
            } else { Ok(()) }
        };
        match name {
            "coro_resume" => {
                need(1, "CORO_RESUME")?;
                let co = expect_coro(&a[0], "CORO_RESUME")?;
                Ok(Some(self.coro_resume(&co, Value::Nil)?))
            }
            "coro_send" => {
                need(2, "CORO_SEND")?;
                let co = expect_coro(&a[0], "CORO_SEND")?;
                Ok(Some(self.coro_resume(&co, a[1].clone())?))
            }
            "coro_done" => {
                need(1, "CORO_DONE")?;
                let co = expect_coro(&a[0], "CORO_DONE")?;
                let d = co.borrow().done;
                Ok(Some(Value::Bool(d)))
            }
            "coro_result" => {
                need(1, "CORO_RESULT")?;
                let co = expect_coro(&a[0], "CORO_RESULT")?;
                let c = co.borrow();
                if !c.done {
                    return Err("CORO_RESULT: Coroutine ist noch nicht beendet".into());
                }
                Ok(Some(c.result.clone()))
            }
            "coro_close" => {
                need(1, "CORO_CLOSE")?;
                let co = expect_coro(&a[0], "CORO_CLOSE")?;
                let mut c = co.borrow_mut();
                c.done = true;
                c.locals.clear();
                c.stack.clear();
                c.try_handlers.clear();
                Ok(Some(Value::Nil))
            }
            // __comp_iter ueber eine Coroutine: eager drainen (FOR EACH / Comp).
            "__comp_iter" if matches!(a.first(), Some(Value::Coroutine(_))) => {
                let co = expect_coro(&a[0], "__comp_iter")?;
                let items = self.coro_drain(&co)?;
                Ok(Some(Value::Tuple(Rc::new(items))))
            }
            _ => Ok(None),
        }
    }

    /// Eine markierte Folge (`model::verschmelzen`) in einem Schritt: zwei
    /// Zahlen holen, rechnen oder vergleichen, und je nach Art das Ergebnis
    /// ablegen, speichern oder springen. `None`, sobald etwas nicht der
    /// sichere Fall ist (kein Zahlenpaar, Ueberlauf, Division durch 0, ein
    /// Typ, der beim Speichern umgewandelt werden muesste, ein leerer oder
    /// konstanter Platz) -- dann laeuft die Folge Befehl fuer Befehl, mit
    /// derselben Meldung wie immer. Bis dahin ist nichts veraendert.
    /// Liefert die Stelle, an der es weitergeht.
    #[inline]
    fn verschmolzen(&mut self, fn_: &'p Func, art: u8, p: usize, locals: &mut [Value],
                    stack: &mut Vec<Value>) -> Option<usize> {
        #[derive(Clone, Copy)]
        enum Z { I(i64), F(f64) }
        let code = &fn_.code;
        let hol = |vm: &Self, ins: &crate::model::Instr, locals: &[Value]| -> Option<Z> {
            let z = |v: &Value| match v { Value::Int(i) => Some(Z::I(*i)), Value::Float(f) => Some(Z::F(*f)), _ => None };
            match ins.op {
                op::LOAD_LOCAL => z(&locals[ins.arg.as_usize()]),
                op::LOAD_CONST => z(&fn_.constants[ins.arg.as_usize()]),
                _ => vm.global_slots[ins.arg.as_usize()].as_ref().and_then(|s| z(&s.borrow().value)),
            }
        };
        let a = hol(self, &code[p], locals)?;
        let b = hol(self, &code[p + 1], locals)?;
        let f = |z: Z| match z { Z::I(i) => i as f64, Z::F(x) => x };
        let o = code[p + 2].op;
        let r = match (o, a, b) {
            (op::ADD, Z::I(x), Z::I(y)) => Value::Int(x.checked_add(y)?),
            (op::SUB, Z::I(x), Z::I(y)) => Value::Int(x.checked_sub(y)?),
            (op::MUL, Z::I(x), Z::I(y)) => Value::Int(x.checked_mul(y)?),
            (op::ADD, ..) => Value::Float(f(a) + f(b)),
            (op::SUB, ..) => Value::Float(f(a) - f(b)),
            (op::MUL, ..) => Value::Float(f(a) * f(b)),
            (op::DIV, ..) => { if f(b) == 0.0 { return None; } Value::Float(f(a) / f(b)) }
            (op::LT, Z::I(x), Z::I(y)) => Value::Bool(x < y),
            (op::GT, Z::I(x), Z::I(y)) => Value::Bool(x > y),
            (op::LEQ, Z::I(x), Z::I(y)) => Value::Bool(x <= y),
            (op::GEQ, Z::I(x), Z::I(y)) => Value::Bool(x >= y),
            (op::EQ, Z::I(x), Z::I(y)) => Value::Bool(x == y),
            (op::NEQ, Z::I(x), Z::I(y)) => Value::Bool(x != y),
            // MOD und Vergleiche mit Kommazahlen: dieselben Funktionen wie
            // der Einzelbefehl; ein Fehler heisst "Befehl fuer Befehl".
            _ => {
                let w = |z: Z| match z { Z::I(i) => Value::Int(i), Z::F(x) => Value::Float(x) };
                let (va, vb) = (w(a), w(b));
                match o {
                    op::MOD => modulo(va, vb).ok()?,
                    op::LT => Value::Bool(cmp(&va, &vb, '<').ok()?),
                    op::GT => Value::Bool(cmp(&va, &vb, '>').ok()?),
                    op::LEQ => Value::Bool(cmp(&va, &vb, 'l').ok()?),
                    op::GEQ => Value::Bool(cmp(&va, &vb, 'g').ok()?),
                    _ => return None,
                }
            }
        };
        let genau = |v: &Value, ty: &str| match v {
            Value::Int(_) => ty == "integer",
            Value::Float(_) => ty == "float",
            Value::Bool(_) => ty == "boolean",
            _ => false,
        };
        match art {
            crate::model::VS_LOKAL => {
                let slot = code[p + 3].arg.as_usize();
                if !genau(&r, &fn_.local_types[slot]) { return None; }
                locals[slot] = r;
                Some(p + 4)
            }
            crate::model::VS_GLOBAL => {
                let slot = self.global_slots[code[p + 3].arg.as_usize()].as_ref()?;
                let mut sb = slot.borrow_mut();
                if sb.is_const || !genau(&r, &sb.ty) { return None; }
                sb.value = r;
                Some(p + 4)
            }
            crate::model::VS_SPRUNG_FALSCH | crate::model::VS_SPRUNG_WAHR => {
                let Value::Bool(w) = r else { return None };
                let springen = w == (art == crate::model::VS_SPRUNG_WAHR);
                Some(if springen { code[p + 3].arg.as_usize() } else { p + 4 })
            }
            _ => { stack.push(r); Some(p + 3) }
        }
    }

    /// Den obersten Rahmen verlassen: die Vecs des Gerufenen zurueck in den
    /// Vorrat, der Zustand des Aufrufers zurueck an seinen Platz. Liefert
    /// (Aufrufer, Gerufener).
    fn rahmen_verlassen(&mut self, locals: &mut Vec<Value>, stack: &mut Vec<Value>,
                        ip: &mut usize, try_handlers: &mut Vec<(usize, usize)>) -> (&'p Func, &'p Func) {
        let r = self.rahmen.pop().expect("Rahmen");
        let mut l = std::mem::replace(locals, r.locals);
        let mut st = std::mem::replace(stack, r.stack);
        l.clear();
        st.clear();
        if self.pool_locals.len() < 64 { self.pool_locals.push(l); }
        if self.pool_stacks.len() < 64 { self.pool_stacks.push(st); }
        *try_handlers = r.try_handlers;
        *ip = r.ip;
        self.depth -= 1;
        (r.fn_, r.gerufen)
    }

    fn dispatch(
        &mut self,
        fn_: &'p Func,
        locals: &mut Vec<Value>,
        stack: &mut Vec<Value>,
        ip: &mut usize,
        try_handlers: &mut Vec<(usize, usize)>,
        self_obj: Option<&Value>,
        basis: usize,
    ) -> R<Step> {
        let mut fn_ = fn_;
        let mut code = &fn_.code;
        let mut constants = &fn_.constants;
        let mut n = code.len();
        // Ein RETURN mit offenem Rahmen kehrt in den Aufrufer zurueck, statt
        // `dispatch` zu verlassen (siehe `Vm::rahmen`).
        macro_rules! zurueck {
            ($wert:expr) => {{
                let wert: Value = $wert;
                let (aufrufer, gerufen) = self.rahmen_verlassen(locals, stack, ip, try_handlers);
                stack.push(if gerufen.is_sub { Value::Nil } else { wert });
                fn_ = aufrufer;
                code = &fn_.code;
                constants = &fn_.constants;
                n = code.len();
                continue;
            }};
        }
        // Zeilen-Tracking nur, wenn jemand zusieht (Profiler/Stop/Debugger).
        // Fuer Laufzeitfehler wird die Quell-Zeile LAZY im Fehlerfall
        // ermittelt (run_frame) -- der Normalfall zahlt pro Instruktion nichts.
        let track_lines = self.prof.is_some() || self.stop.is_some() || self.dbg.is_some();
        // Methoden in den Maschinencode (M4 Schritt 7b): einmal je Eintritt
        // gefragt statt je Aufruf.
        #[cfg(feature = "jit")]
        let methoden_jit = self.jit.is_some() && !track_lines;

        loop {
            if *ip >= n {
                if self.rahmen.len() > basis { zurueck!(Value::Nil); }
                return Ok(Step::Return(Value::Nil));
            }
            let instr = &code[*ip];
            // Superinstruktion (M2): die ganze Folge in einem Schritt, wenn es
            // sicher geht; sonst Befehl fuer Befehl wie immer. Nicht, wenn
            // jemand die Zeile je Befehl braucht (Profiler, Debugger, Stop).
            if instr.schnell != 0 && !track_lines {
                if let Some(weiter) = self.verschmolzen(fn_, instr.schnell, *ip, locals, stack) {
                    *ip = weiter;
                    continue;
                }
            }
            let arg = &instr.arg;
            if track_lines {
                if let Some(ln) = fn_.lines.get(*ip) {
                    let ln = *ln;
                    if ln != 0 && ln != self.cur_line {
                        self.cur_line = ln;
                        if let Some(p) = self.prof.as_mut() {
                            p.tick(ln);
                        }
                        // Externes Stop-Signal: sauber abwickeln, damit die
                        // bisherigen Profile-Daten noch ausgegeben werden.
                        if let Some(s) = self.stop.as_ref() {
                            if s.load(Ordering::Relaxed) {
                                self.profile_stop_flag = true;
                                return Err(PROFILE_STOP.into());
                            }
                        }
                        if self.dbg.is_some() {
                            self.debug_on_line(fn_, locals)?;
                        }
                    }
                }
            }
            *ip += 1;

            match instr.op {
                op::LOAD_CONST => stack.push(constants[arg.as_usize()].clone()),
                // Review-Fund: war der einzige Stack-Konsument, der ein leeres
                // Ergebnis stillschweigend ignorierte (jeder andere Pop nutzt
                // vm_pop und meldet "Stack leer") -- ein Stack-Imbalance-Bug im
                // Compiler waere hier unsichtbar geblieben und erst an einer
                // spaeteren, voellig unabhaengigen Opcode-Stelle aufgefallen.
                op::POP => { vm_pop(stack)?; }
                op::DUP => { let v = vm_top(stack)?.clone(); stack.push(v); }

                // --- Lokale ---
                op::LOAD_LOCAL => stack.push(locals[arg.as_usize()].clone()),
                op::STORE_LOCAL => {
                    let slot = arg.as_usize();
                    let v = vm_pop(stack)?;
                    locals[slot] = passend!(v, fn_.local_types[slot].as_str(), "Lokale Variable");
                }
                op::FOR_NEXT => {
                    // Fusioniertes FOR-Ende: var += step, Weiter-Test, Sprung
                    // zum Body. Arg: [var_global, var_idx, end_slot,
                    // step_is_slot, step_idx, neg, body_target].
                    let l = arg.ints();
                    let var_global = l[0] == 1;
                    let var_idx = l[1] as usize;
                    let end_slot = l[2] as usize;
                    let step_is_slot = l[3] == 1;
                    let step_idx = l[4] as usize;
                    let neg = l[5] == 1;
                    let target = l[6] as usize;
                    let step_int = if step_is_slot {
                        match &locals[step_idx] { Value::Int(i) => Some(*i), _ => None }
                    } else {
                        match &constants[step_idx] { Value::Int(i) => Some(*i), _ => None }
                    };
                    let end_int = match &locals[end_slot] { Value::Int(i) => Some(*i), _ => None };
                    // Int-Fast-Path (der Normalfall); alles andere geht den
                    // generischen Weg mit exakt der Einzel-Opcode-Semantik.
                    let mut exit: Option<bool> = None;
                    if let (Some(st), Some(en)) = (step_int, end_int) {
                        if !var_global {
                            if let Value::Int(cur) = &locals[var_idx] {
                                let next = cur.checked_add(st).ok_or_else(|| int_overflow_msg("+"))?;
                                locals[var_idx] = Value::Int(next);
                                exit = Some(if neg { next < en } else { next > en });
                            }
                        } else if let Some(slot) = self.global_slots[var_idx].as_ref() {
                            let mut sb = slot.borrow_mut();
                            if !sb.is_const {
                                if let Value::Int(cur) = &sb.value {
                                    let next = cur.checked_add(st).ok_or_else(|| int_overflow_msg("+"))?;
                                    sb.value = Value::Int(next);
                                    exit = Some(if neg { next < en } else { next > en });
                                }
                            }
                        }
                    }
                    let exit = match exit {
                        Some(e) => e,
                        None => {
                            // Generisch: ADD + Store-Coerce + Vergleich -- wie
                            // die ehemalige Opcode-Folge.
                            let cur = if var_global {
                                let s = match self.global_slots[var_idx].as_ref() { Some(s) => s, None => return Err(self.global_ungesetzt(var_idx)) };
                                s.borrow().value.clone()
                            } else { locals[var_idx].clone() };
                            let stepv = if step_is_slot { locals[step_idx].clone() } else { constants[step_idx].clone() };
                            require_number(&cur, &stepv, "+")?;
                            let next = nn_add(cur, stepv)?;
                            if var_global {
                                let slot = match self.global_slots[var_idx].clone() { Some(s) => s, None => return Err(self.global_ungesetzt(var_idx)) };
                                if slot.borrow().is_const { return Err("CONST kann nicht ueberschrieben werden".into()); }
                                let ty = slot.borrow().ty.clone();
                                let cv = coerce(next.clone(), &ty, "Zuweisung an global")?;
                                slot.borrow_mut().value = cv;
                            } else {
                                let ty = &fn_.local_types[var_idx];
                                locals[var_idx] = coerce(next.clone(), ty, "Lokale Variable")?;
                            }
                            let endv = locals[end_slot].clone();
                            cmp(&next, &endv, if neg { '<' } else { '>' })?
                        }
                    };
                    if !exit {
                        // M4: eine Schleife im Maschinencode (jit::Jit::schleife).
                        #[cfg(feature = "jit")]
                        let vmp: *mut Vm = self;
                        // Mit Bilanz: eine Runde einer Schleife, die in der VM bleibt.
                        #[cfg(feature = "jit")]
                        if instr.schleife.get() == 1 { if let Some(j) = self.jit.as_ref() { j.vm_runde(&instr.schleife); } }
                        #[cfg(feature = "jit")]
                        if let Some(j) = self.jit.as_ref().filter(|_| !track_lines && stack.is_empty() && instr.schleife.get() != 1) {
                            {
                                if let Some(weiter) = j.schleife(vmp, self.prog, fn_, *ip - 1, target, locals, stack, self_obj,
                                                                 self.depth, MAX_CALL_DEPTH, &self.global_slots) {
                                    // Ein Befehl im Bereich ist gescheitert: sein Fehler,
                                    // an seiner Stelle (die Zeile nimmt run_frame von ip - 1).
                                    if let Some(e) = j.meldung_nehmen() { *ip = weiter + 1; return Err(e); }
                                    *ip = weiter;
                                    continue;
                                }
                            }
                        }
                        *ip = target;
                    }
                }

                // --- Slot-Globals ---
                op::LOAD_GLOBAL_SLOT => {
                    let s = match self.global_slots[arg.as_usize()].as_ref() { Some(s) => s, None => return Err(self.global_ungesetzt(arg.as_usize())) };
                    stack.push(s.borrow().value.clone());
                }
                op::STORE_GLOBAL_SLOT => {
                    let idx = arg.as_usize();
                    let v = vm_pop(stack)?;
                    let slot = match self.global_slots[idx].clone() { Some(s) => s, None => return Err(self.global_ungesetzt(idx)) };
                    let mut sb = slot.borrow_mut();
                    if sb.is_const {
                        return Err("CONST kann nicht ueberschrieben werden".into());
                    }
                    let neu = passend!(v, sb.ty.as_str(), "Zuweisung an global");
                    sb.value = neu;
                }
                // --- Arithmetik (generisch, mit User-Operator-Overloading) ---
                op::ADD => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    // Zahlen direkt hier -- `addieren` ist gross und wird nicht
                    // eingebettet; in einer heissen Schleife kostet der Aufruf.
                    let v = match zahlen_addieren(&a, &b) {
                        Some(r) => r?,
                        None => self.addieren(a, b)?,
                    };
                    stack.push(v);
                }
                op::ADD_STORE_LOCAL => {
                    // `x = x + e`: auf dem Stapel liegen der geladene Wert von x
                    // und e. Haelt der Platz noch GENAU diese Zeichenkette (e hat
                    // x nicht veraendert), gibt er seinen Verweis ab -- dann ist
                    // `a` der einzige Halter und `addieren` haengt an, statt zu
                    // kopieren. Sonst ist es ADD + STORE wie bisher.
                    let slot = arg.as_usize();
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    let v = match zahlen_addieren(&a, &b) {
                        Some(r) => r?,
                        None => {
                            if let (Value::Str(ra), Value::Str(rs)) = (&a, &locals[slot]) {
                                if Rc::ptr_eq(ra, rs) && ist_schlicht(&b) { locals[slot] = Value::Nil; }
                            }
                            self.addieren(a, b)?
                        }
                    };
                    locals[slot] = passend!(v, fn_.local_types[slot].as_str(), "Lokale Variable");
                }
                op::ADD_STORE_GLOBAL_SLOT => {
                    let idx = arg.as_usize();
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    let v = match zahlen_addieren(&a, &b) {
                        Some(r) => r?,
                        None => {
                            let slot = match self.global_slots[idx].clone() { Some(s) => s, None => return Err(self.global_ungesetzt(idx)) };
                            {
                                let mut sb = slot.borrow_mut();
                                if let (Value::Str(ra), Value::Str(rs)) = (&a, &sb.value) {
                                    if !sb.is_const && Rc::ptr_eq(ra, rs) && ist_schlicht(&b) {
                                        sb.value = Value::Nil;
                                    }
                                }
                            }
                            // Kein Borrow ueber `addieren` halten: ein Operator
                            // einer Klasse (`OPERATOR +`) darf dieselbe Globale lesen.
                            self.addieren(a, b)?
                        }
                    };
                    let slot_ref = match self.global_slots[idx].as_ref() { Some(s) => s, None => return Err(self.global_ungesetzt(idx)) };
                    let mut sb = slot_ref.borrow_mut();
                    if sb.is_const {
                        return Err("CONST kann nicht ueberschrieben werden".into());
                    }
                    let neu = passend!(v, sb.ty.as_str(), "Zuweisung an global");
                    sb.value = neu;
                }
                op::SUB => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(
                            x.checked_sub(*y).map(Value::Int).ok_or_else(|| int_overflow_msg("-"))?),
                        (Value::Float(x), Value::Float(y)) => stack.push(Value::Float(x - y)),
                        (Value::Int(x), Value::Float(y)) => stack.push(Value::Float(*x as f64 - y)),
                        (Value::Float(x), Value::Int(y)) => stack.push(Value::Float(x - *y as f64)),
                        _ => {
                            if let Some(r) = module_op('-', &a, &b) { stack.push(r?); }
                            else if let Some(r) = self.user_op("__op_sub__", &a, &b, false)? { stack.push(r); }
                            else { require_number(&a, &b, "-")?; stack.push(nn_arith(a, b, '-')?); }
                        }
                    }
                }
                op::MUL => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(
                            x.checked_mul(*y).map(Value::Int).ok_or_else(|| int_overflow_msg("*"))?),
                        (Value::Float(x), Value::Float(y)) => stack.push(Value::Float(x * y)),
                        (Value::Int(x), Value::Float(y)) => stack.push(Value::Float(*x as f64 * y)),
                        (Value::Float(x), Value::Int(y)) => stack.push(Value::Float(x * *y as f64)),
                        _ => {
                            if let Some(r) = module_op('*', &a, &b) { stack.push(r?); }
                            else if let Some(r) = self.user_op("__op_mul__", &a, &b, true)? { stack.push(r); }
                            else { stack.push(mul(a, b)?); }
                        }
                    }
                }
                op::DIV => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    if matches!(&a, Value::Int(_) | Value::Float(_)) && matches!(&b, Value::Int(_) | Value::Float(_)) {
                        stack.push(div(a, b)?);
                    } else if let Some(r) = module_op('/', &a, &b) { stack.push(r?); }
                    else if let Some(r) = self.user_op("__op_div__", &a, &b, false)? { stack.push(r); }
                    else { stack.push(div(a, b)?); }
                }
                op::MOD => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    if matches!(&a, Value::Int(_) | Value::Float(_)) && matches!(&b, Value::Int(_) | Value::Float(_)) {
                        stack.push(modulo(a, b)?);
                    } else if let Some(r) = self.user_op("__op_mod__", &a, &b, false)? { stack.push(r); }
                    else { stack.push(modulo(a, b)?); }
                }
                op::INT_DIV => { let b = vm_pop(stack)?; let a = vm_pop(stack)?; stack.push(int_div(a, b)?); }
                op::NEG => { let v = vm_pop(stack)?; stack.push(neg(v)?); }

                // --- Vergleich / Logik ---
                op::EQ => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x == y)),
                        _ => match self.user_op("__op_eq__", &a, &b, true)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(value_eq(&a, &b))) }
                    } }
                op::NEQ => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x != y)),
                        _ => match self.user_op("__op_ne__", &a, &b, true)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(!value_eq(&a, &b))) }
                    } }
                op::LT => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x < y)),
                        (Value::Float(_), Value::Float(_)) | (Value::Int(_), Value::Float(_)) | (Value::Float(_), Value::Int(_)) =>
                            stack.push(Value::Bool(cmp(&a, &b, '<')?)),
                        _ => match self.user_op("__op_lt__", &a, &b, false)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(cmp(&a, &b, '<')?)) }
                    } }
                op::GT => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x > y)),
                        (Value::Float(_), Value::Float(_)) | (Value::Int(_), Value::Float(_)) | (Value::Float(_), Value::Int(_)) =>
                            stack.push(Value::Bool(cmp(&a, &b, '>')?)),
                        _ => match self.user_op("__op_gt__", &a, &b, false)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(cmp(&a, &b, '>')?)) }
                    } }
                op::LEQ => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x <= y)),
                        (Value::Float(_), Value::Float(_)) | (Value::Int(_), Value::Float(_)) | (Value::Float(_), Value::Int(_)) =>
                            stack.push(Value::Bool(cmp(&a, &b, 'l')?)),
                        _ => match self.user_op("__op_le__", &a, &b, false)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(cmp(&a, &b, 'l')?)) }
                    } }
                op::GEQ => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x >= y)),
                        (Value::Float(_), Value::Float(_)) | (Value::Int(_), Value::Float(_)) | (Value::Float(_), Value::Int(_)) =>
                            stack.push(Value::Bool(cmp(&a, &b, 'g')?)),
                        _ => match self.user_op("__op_ge__", &a, &b, false)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(cmp(&a, &b, 'g')?)) }
                    } }
                op::NOT => { let v = vm_pop(stack)?; stack.push(Value::Bool(!v.truthy())); }

                // --- Kontrollfluss ---
                op::JUMP => {
                    let ziel = arg.as_usize();
                    // M4: Ruecksprung einer Schleife -> vielleicht Maschinencode.
                    #[cfg(feature = "jit")]
                    let vmp: *mut Vm = self;
                    #[cfg(feature = "jit")]
                    if ziel < *ip && instr.schleife.get() == 1 { if let Some(j) = self.jit.as_ref() { j.vm_runde(&instr.schleife); } }
                    #[cfg(feature = "jit")]
                    if let Some(j) = self.jit.as_ref().filter(|_| ziel < *ip && !track_lines && stack.is_empty() && instr.schleife.get() != 1) {
                        {
                            if let Some(weiter) = j.schleife(vmp, self.prog, fn_, *ip - 1, ziel, locals, stack, self_obj,
                                                             self.depth, MAX_CALL_DEPTH, &self.global_slots) {
                                if let Some(e) = j.meldung_nehmen() { *ip = weiter + 1; return Err(e); }
                                *ip = weiter;
                                continue;
                            }
                        }
                    }
                    *ip = ziel;
                }
                op::JUMP_IF_FALSE => { let v = vm_pop(stack)?; if !v.truthy() { *ip = arg.as_usize(); } }
                op::JUMP_IF_TRUE => { let v = vm_pop(stack)?; if v.truthy() { *ip = arg.as_usize(); } }

                // --- Aufrufe ---
                op::CALL_USER => {
                    let (fn_name, argc, idx) = call_parts(arg);
                    // Vorab aufgeloester Index (model::specialize_args) -- kein
                    // Hash-Lookup pro Aufruf; -1 -> Namens-Fallback.
                    let callee: &'p Func = if idx >= 0 {
                        &self.prog.functions[idx as usize]
                    } else {
                        self.prog.func(fn_name)
                            .ok_or_else(|| format!("Unbekannte Funktion: {}", fn_name.to_uppercase()))?
                    };
                    let split = stack.len() - argc;
                    // M3: uebersetzte reine Zahlenfunktion. Gibt sie auf (oder
                    // passt ein Argument nicht genau), rechnet die VM den Aufruf
                    // unten selbst -- mit ihrer Meldung und ihrer Zeile.
                    #[cfg(feature = "jit")]
                    if idx >= 0 && !track_lines {
                        if let Some(j) = self.jit.as_ref() {
                            if let Some(v) = j.rufen(idx as usize, &stack[split..], self.depth, MAX_CALL_DEPTH, &self.global_slots) {
                                stack.truncate(split);
                                stack.push(v);
                                continue;
                            }
                        }
                    }
                    // Die Argumente wandern per `drain` direkt vom Stapel in die
                    // Locals des Aufgerufenen -- vorher legte `split_off` je
                    // Aufruf eine eigene Liste an. Nur eine Coroutine braucht
                    // sie als Liste (sie laeuft erst spaeter).
                    if callee.is_coroutine {
                        let call_args = stack.split_off(split);
                        stack.push(make_coro(callee, call_args, None));
                    } else if !track_lines && !callee.param_byref.iter().any(|&b| b) {
                        // M2: Rahmen wechseln statt exec -> run_frame -> dispatch.
                        // Nicht, wenn jemand zusieht (Profiler, Debugger, Stop:
                        // die fuehren einen Stapel je exec).
                        if self.depth >= MAX_CALL_DEPTH {
                            return Err(format!("Maximale Aufruftiefe ({}) ueberschritten -- unendliche Rekursion?", MAX_CALL_DEPTH));
                        }
                        let lbuf = self.pool_locals.pop().unwrap_or_default();
                        let neu_locals = bind_params(callee, stack.drain(split..), lbuf)?;
                        let neu_stack = self.pool_stacks.pop().unwrap_or_else(|| Vec::with_capacity(16));
                        self.depth += 1;
                        self.rahmen.push(Rahmen {
                            fn_,
                            locals: std::mem::replace(locals, neu_locals),
                            stack: std::mem::replace(stack, neu_stack),
                            ip: *ip,
                            try_handlers: std::mem::take(try_handlers),
                            gerufen: callee,
                        });
                        fn_ = callee;
                        code = &fn_.code;
                        constants = &fn_.constants;
                        n = code.len();
                        *ip = 0;
                        continue;
                    } else if callee.param_byref.iter().any(|&b| b) {
                        // BYREF: finale Param-Werte mit zurueckgeben. Layout fuers
                        // Write-Back: [.., bv0, bv1, .., bv{m-1}, result].
                        let (ret, byref_vals) = self.exec_byref(callee, stack.drain(split..), None)?;
                        for v in byref_vals { stack.push(v); }
                        if !callee.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                    } else {
                        let ret = self.exec(callee, stack.drain(split..), None)?;
                        if !callee.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                    }
                }
                op::CALL_BUILTIN => {
                    let (name, argc, _) = call_parts(arg);
                    let split = stack.len() - argc;
                    // Args als Slice direkt vom Stack (kein split_off-Vec pro
                    // Call); Ergebnis erst nach dem Truncate pushen.
                    // Reihenfolge: array-HOF (FUNCREF-Comparator) -> scene (VM-State)
                    // -> coro (VM-State) -> Grafik -> pure.
                    // ASSERT braucht die Quell-Zeile fuer seine Meldung. Die
                    // wird sonst NUR beim Profilieren/Debuggen mitgefuehrt --
                    // der Normalfall zahlt bewusst nichts pro Instruktion
                    // (s. `track_lines` oben). Also hier gezielt fuer diese
                    // eine Familie nachschlagen; das erste Byte zu vergleichen
                    // haelt die Kosten fuer alle anderen Builtins bei einem
                    // Byte-Vergleich.
                    if !track_lines && name.as_bytes().first() == Some(&b'a') && name.starts_with("assert") {
                        self.cur_line = fn_.lines.get(*ip).copied().unwrap_or(0);
                    }
                    let v = {
                        let bargs: &[Value] = &stack[split..];
                        self.builtin_rufen(name, bargs, &instr.familie)?
                    };
                    stack.truncate(split);
                    stack.push(v);
                }
                op::CALL_METHOD => {
                    let (method, argc, _) = call_parts(arg);
                    let split = stack.len() - argc;
                    // Der haeufigste Fall -- Methode einer Instanz -- nimmt die
                    // Argumente per `drain` direkt vom Stapel (wie CALL_USER);
                    // das Objekt liegt darunter und wird danach abgeraeumt.
                    let mut erledigt = false;
                    if split > 0 {
                        if let Value::Instance(rc) = &stack[split - 1] {
                            // Merkplatz: dieselbe Klasse wie beim letzten Mal an
                            // dieser Stelle -> Methode ohne Suche (sonst je Aufruf
                            // Klassenname klonen und die Kette absuchen).
                            let (lage, f_p) = instr.merk.get();
                            let getroffen = std::ptr::eq(Rc::as_ptr(&rc.borrow().layout), lage);
                            let m: &'p Func = if getroffen {
                                unsafe { &*(f_p as *const Func) }
                            } else {
                                let cn = rc.borrow().class_name.clone();
                                let m = self.resolve_method(&cn, method)
                                    .ok_or_else(|| format!("Methode '{}' existiert nicht in {}", method, cn))?;
                                byref_ohne_typ(m, method, &cn, argc)?;
                                instr.merk.set((Rc::as_ptr(&rc.borrow().layout), m as *const Func as usize));
                                m
                            };
                            // M4 Schritt 7b: die Methode als Maschinencode, wenn sie
                            // fuer genau diese Klasse uebersetzt ist; sonst (oder
                            // wenn er aufgibt) rechnet die VM sie.
                            #[cfg(feature = "jit")]
                            if methoden_jit && !m.is_coroutine {
                                if let Some(v) = self.methode_maschinencode(m, rc, &stack[split..]) {
                                    stack.truncate(split - 1);
                                    stack.push(if m.is_sub { Value::Nil } else { v });
                                    continue;
                                }
                            }
                            if !m.is_coroutine {
                                let obj = stack[split - 1].clone();
                                let ret = self.exec(m, stack.drain(split..), Some(obj))?;
                                stack.pop();
                                if !m.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                                erledigt = true;
                            }
                        }
                    }
                    if !erledigt {
                    let margs = stack.split_off(split);
                    let obj = vm_pop(stack)?;
                    match &obj {
                        Value::Instance(rc) => {
                            let cn = rc.borrow().class_name.clone();
                            let m = self.resolve_method(&cn, method)
                                .ok_or_else(|| format!("Methode '{}' existiert nicht in {}", method, cn))?;
                            byref_ohne_typ(m, method, &cn, argc)?;
                            if m.is_coroutine {
                                stack.push(make_coro(m, margs, Some(obj.clone())));
                            } else {
                                let ret = self.exec(m, margs, Some(obj.clone()))?;
                                if !m.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                            }
                        }
                        Value::Nil => return Err(format!(
                            "Methodenaufruf '.{}' bei NIL-Referenz{}", method, NIL_NEW)),
                        _ => {
                            // Container-Methode -> Builtin
                            let kind = container_kind(&obj).ok_or_else(|| format!("Methodenaufruf '.{}' bei nicht-Objekt ({})", method, obj.type_name()))?;
                            let bi = container_method(kind, &method.to_lowercase())
                                .ok_or_else(|| format!("{} hat keine Methode '{}'", kind.to_uppercase(), method))?;
                            let mut call_args = Vec::with_capacity(margs.len() + 1);
                            call_args.push(obj.clone());
                            call_args.extend(margs);
                            match safe_call_builtin(bi, &call_args) {
                                Some(Ok(v)) => stack.push(v),
                                Some(Err(e)) => return Err(e),
                                None => return Err(unknown_builtin_msg(bi)),
                            }
                        }
                    }
                    }
                }

                op::LOAD_SELF => {
                    let s = self_obj.ok_or("LOAD_SELF (Self) ausserhalb Methodenkontext")?;
                    stack.push(s.clone());
                }
                op::LOAD_FIELD => {
                    // `Self.x` in einer Methode. Name direkt aus dem Const-Pool
                    // (&str) -- vorher entstand je Zugriff ein neuer String.
                    let name_owned;
                    let name: &str = match &constants[arg.as_usize()] {
                        Value::Str(s) => s,
                        v => { name_owned = v.fmt(); &name_owned }
                    };
                    let s = self_obj.ok_or_else(|| format!("LOAD_FIELD '{}' ausserhalb Methodenkontext", name))?;
                    if let Value::Instance(rc) = s {
                        let b = rc.borrow();
                        let i = feld_platz(instr, &b, name)
                            .ok_or_else(|| format!("Feld '{}' existiert nicht", name))?;
                        let v = b.fields[i].clone();
                        drop(b);
                        stack.push(v);
                    } else { return Err("LOAD_FIELD: self ist keine Instanz".into()); }
                }
                op::STORE_FIELD => {
                    // Ein borrow_mut, eine Suche, der schnelle Typvergleich --
                    // vorher je Zugriff: Name als String, zweite Suche, Typ
                    // geklont und die Fehlermeldung per format! VORAB gebaut.
                    let name_owned;
                    let name: &str = match &constants[arg.as_usize()] {
                        Value::Str(s) => s,
                        v => { name_owned = v.fmt(); &name_owned }
                    };
                    let s = self_obj.ok_or_else(|| format!("STORE_FIELD '{}' ausserhalb Methodenkontext", name))?;
                    let v = vm_pop(stack)?;
                    if let Value::Instance(rc) = s {
                        // Schneller Fall: Wert hat schon den Feldtyp -> direkt.
                        // Sonst die Sperre VOR `coerce` loesen: bei einem
                        // Klassentyp schaut es in den Wert, und das kann DIESES
                        // Objekt sein (`Self.naechster = Self`).
                        let langsam_ty = {
                            let mut rcb = rc.borrow_mut();
                            let i = feld_platz(instr, &rcb, name)
                                .ok_or_else(|| format!("Feld '{}' existiert nicht", name))?;
                            let lage = rcb.layout.clone();
                            let ty = lage.typen[i].as_str();
                            match (&v, ty) {
                                (Value::Int(_), "integer") | (Value::Float(_), "float")
                                | (Value::Str(_), "string") | (Value::Bool(_), "boolean")
                                | (_, "any") | (_, "") => { rcb.fields[i] = v; None }
                                (Value::Int(n), "float") => { rcb.fields[i] = Value::Float(*n as f64); None }
                                _ => Some((i, ty.to_string(), v)),
                            }
                        };
                        if let Some((i, ty, v)) = langsam_ty {
                            let cv = coerce(v, &ty, &format!("Zuweisung an Feld {}", name))?;
                            rc.borrow_mut().fields[i] = cv;
                        }
                    } else { return Err("STORE_FIELD: self ist keine Instanz".into()); }
                }
                op::LOAD_MEMBER => {
                    // Der langsame Weg liegt in `member_laden` (eigene
                    // Funktion): was in `dispatch` waechst, verschiebt die
                    // Codelage, und das kostete gemessen bis zu 8 % bei reinen
                    // Zahlenschleifen, die gar kein Objekt anfassen.
                    let obj = vm_pop(stack)?;
                    // Nur der schnelle Weg (dieselbe Klasse wie beim letzten
                    // Mal) steht hier; alles andere in `member_laden`.
                    if let Value::Instance(rc) = &obj {
                        let b = rc.borrow();
                        let (lage, platz) = instr.merk.get();
                        if std::ptr::eq(Rc::as_ptr(&b.layout), lage) {
                            let v = b.fields[platz].clone();
                            drop(b);
                            stack.push(v);
                            continue;
                        }
                    }
                    let v = self.member_laden(instr, &constants[arg.as_usize()], obj)?;
                    stack.push(v);
                }
                op::STORE_MEMBER => {
                    let v = vm_pop(stack)?;
                    let obj = vm_pop(stack)?;
                    self.member_setzen(instr, &constants[arg.as_usize()], obj, v)?;
                }
                // `ziel = ziel + e` fuer Feld, Mitglied und Element: wie ADD +
                // STORE_*, aber an Ort und Stelle angehaengt, wenn der Platz den
                // geladenen Text noch haelt (`an_ort_anhaengen`). Die Arbeit
                // liegt in eigenen Funktionen -- `dispatch` bleibt schlank.
                op::ADD_STORE_FIELD => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    self.feld_addieren(instr, &constants[arg.as_usize()], self_obj, a, b)?;
                }
                op::ADD_STORE_MEMBER => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    let obj = vm_pop(stack)?;
                    self.mitglied_addieren(instr, &constants[arg.as_usize()], obj, a, b)?;
                }
                op::ADD_STORE_INDEX => self.index_addieren(stack, arg.as_usize())?,
                // --- Arrays ---
                op::LOAD_INDEX => {
                    let num_dims = arg.as_usize();
                    if num_dims == 1 {
                        // 1D (der Normalfall): kein split_off-Vec; direkter
                        // Zugriff bei Int-Index in Bounds, sonst generisch
                        // (identische Fehler via load_index).
                        let ix = vm_pop(stack)?;
                        let arr = vm_pop(stack)?;
                        let mut fast = None;
                        if let (Value::Array(a), Value::Int(i)) = (&arr, &ix) {
                            let ab = a.borrow();
                            if ab.dims.len() == 1 && ab.unten.is_empty() && *i >= 0 && (*i as usize) < ab.cells.len() {
                                fast = Some(ab.cells.get(*i as usize));
                            }
                        }
                        match fast {
                            Some(v) => stack.push(v),
                            None => stack.push(load_index(&arr, std::slice::from_ref(&ix))?),
                        }
                    } else {
                        let split = stack.len() - num_dims;
                        let idx_vals = stack.split_off(split);
                        let arr = vm_pop(stack)?;
                        stack.push(load_index(&arr, &idx_vals)?);
                    }
                }
                op::STORE_INDEX => {
                    let num_dims = arg.as_usize();
                    if num_dims == 1 {
                        let v = vm_pop(stack)?;
                        let ix = vm_pop(stack)?;
                        let arr = vm_pop(stack)?;
                        let mut rest = Some(v);
                        if let (Value::Array(a), Value::Int(i)) = (&arr, &ix) {
                            let mut ab = a.borrow_mut();
                            if ab.dims.len() == 1 && ab.unten.is_empty() && *i >= 0 && (*i as usize) < ab.cells.len() {
                                let val = rest.take().unwrap();
                                let iu = *i as usize;
                                // Typisiertes Backing zuerst: Int/Float-Stores
                                // gehen ohne jeden String-Vergleich direkt rein.
                                match (&mut ab.cells, val) {
                                    (Cells::Int(vec), Value::Int(x)) => vec[iu] = x,
                                    (Cells::Float(vec), Value::Float(x)) => vec[iu] = x,
                                    (Cells::Float(vec), Value::Int(x)) => vec[iu] = x as f64,
                                    (_, val) => {
                                        let cv = match (&val, ab.element_type.as_str()) {
                                            (Value::Str(_), "string") | (Value::Bool(_), "boolean")
                                            | (_, "any") => val,
                                            _ => { let et = ab.element_type.clone(); coerce(val, &et, "Array-Element")? }
                                        };
                                        ab.cells.set(iu, cv);
                                    }
                                }
                            }
                        }
                        if let Some(val) = rest { store_index(&arr, std::slice::from_ref(&ix), val)?; }
                    } else {
                        let v = vm_pop(stack)?;
                        let split = stack.len() - num_dims;
                        let idx_vals = stack.split_off(split);
                        let arr = vm_pop(stack)?;
                        store_index(&arr, &idx_vals, v)?;
                    }
                }
                // --- Rueckgabe ---
                op::RETURN => {
                    let v = vm_pop(stack)?;
                    let v = coerce(v, &fn_.return_type, "RETURN")?;
                    if self.rahmen.len() > basis { zurueck!(v); }
                    return Ok(Step::Return(v));
                }
                op::RETURN_VOID => {
                    if self.rahmen.len() > basis { zurueck!(Value::Nil); }
                    return Ok(Step::Return(Value::Nil));
                }
                // Seltene Befehle stehen in `dispatch_selten` -- dort liegt ihr
                // Stapelrahmen, und die heisse Schleife hier bleibt schlank.
                _ => {
                    if let Some(step) = self.dispatch_selten(instr, fn_, locals, stack, ip, try_handlers)? {
                        if let Step::Return(v) = step {
                            if self.rahmen.len() > basis { zurueck!(v); }
                            return Ok(Step::Return(v));
                        }
                        return Ok(step);
                    }
                }
            }
        }
    }

    /// Die seltenen Befehle von `dispatch` (DECLARE, PRINT, INPUT, TRY, Tupel, NEW,
    /// YIELD ...). Eigene Funktion, damit ihr Stapelrahmen nicht bei JEDEM Aufruf
    /// einer Drachenhauch-Funktion mit angelegt wird: `dispatch` wird je Aufruf
    /// neu betreten. `Some(step)` = `dispatch` kehrt damit zurueck.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn dispatch_selten(
        &mut self,
        instr: &'p crate::model::Instr,
        fn_: &'p Func,
        locals: &mut Vec<Value>,
        stack: &mut Vec<Value>,
        ip: &mut usize,
        try_handlers: &mut Vec<(usize, usize)>,
    ) -> R<Option<Step>> {
        let arg = &instr.arg;
        let constants = &fn_.constants;
        match instr.op {
                // BYREF-Argument aus einer Variablen, die waehrend des Aufrufs
                // niemand sieht (der Compiler prueft das, `byref_leihen`): der
                // Wert wandert hinueber, damit der Aufgerufene der einzige
                // Halter ist und an einen Text anhaengen kann. Das Zurueck-
                // schreiben fuellt den Platz wieder. Unter dem Debugger bleibt
                // es eine Kopie -- der zeigt die Variablen des Aufrufers.
                op::LEIHE_LOCAL => {
                    let s = arg.as_usize();
                    let v = if self.dbg.is_some() { locals[s].clone() } else { std::mem::replace(&mut locals[s], Value::Nil) };
                    stack.push(v);
                }
                op::LEIHE_GLOBAL_SLOT => {
                    let idx = arg.as_usize();
                    let s = match self.global_slots[idx].as_ref() { Some(s) => s, None => return Err(self.global_ungesetzt(idx)) };
                    let mut sb = s.borrow_mut();
                    let v = if self.dbg.is_some() || sb.is_const { sb.value.clone() } else { std::mem::replace(&mut sb.value, Value::Nil) };
                    drop(sb);
                    stack.push(v);
                }
                op::DECLARE_LOCAL => {
                    let l = arg.list();
                    let slot = l[0].as_usize();
                    let ty = l[1].str();
                    if let Some(vt) = ty.strip_prefix("map:") {
                        if !matches!(locals[slot], Value::Map(_)) {
                            locals[slot] = Value::Map(Rc::new(RefCell::new(DhMap::new(vt.to_string()))));
                        }
                    } else if matches!(locals[slot], Value::Nil) {
                        locals[slot] = if ty.starts_with("array:") { leeres_feld(ty) } else { arg_value(&l[2]) };
                    }
                }

                op::DECLARE_GLOBAL_SLOT => {
                    let l = arg.list();
                    let slot_idx = l[0].as_usize();
                    let name = constants[l[1].as_usize()].fmt();
                    let ty = constants[l[2].as_usize()].fmt();
                    let default = if let Some(vt) = ty.strip_prefix("map:") {
                        Value::Map(Rc::new(RefCell::new(DhMap::new(vt.to_string()))))
                    } else {
                        let d = constants[l[3].as_usize()].clone();
                        // Der Compiler kann Mathe-Werte nicht als Konstante
                        // ablegen (CVal kennt kein MAT4) und legt NIL hin. Hier
                        // wird daraus das neutrale Element -- fuer alle anderen
                        // Typen liefert element_default wieder NIL, es aendert
                        // sich also nichts.
                        if matches!(d, Value::Nil) { self.element_default(&ty) } else { d }
                    };
                    if self.global_slots[slot_idx].is_none() {
                        let sl = Rc::new(RefCell::new(Slot { ty, value: default, is_const: false }));
                        self.globals.insert(name, sl.clone());
                        self.global_slots[slot_idx] = Some(sl);
                    }
                }
                op::DECLARE_GLOBAL_CONST_SLOT => {
                    let l = arg.list();
                    let slot_idx = l[0].as_usize();
                    let value = vm_pop(stack)?;
                    // Review-Fund: der Compiler emittiert [slot, name_idx,
                    // type_idx] (siehe stmt_const/emit_namespace_const) --
                    // dieser Zweig las bisher l[1] (den NAME-Index) statt
                    // l[2] (den TYPE-Index) als Typ-Konstante. `coerce(value,
                    // "<name-der-const>", ...)` traf so gut wie nie einen der
                    // bekannten Typnamen und lief in coerce()'s Catch-all
                    // (unveraendert durchreichen) -- eine typisierte
                    // `CONST X AS FLOAT = 1` wurde dadurch NIE tatsaechlich
                    // nach FLOAT gecoerct.
                    let ti = l[2].as_usize();
                    let (ty, value) = if matches!(constants[ti], Value::Nil) {
                        (infer_type(&value).to_string(), value)
                    } else {
                        let t = constants[ti].fmt();
                        let v = coerce(value, &t, "CONST")?;
                        (t, v)
                    };
                    if self.global_slots[slot_idx].is_none() {
                        let sl = Rc::new(RefCell::new(Slot { ty, value, is_const: true }));
                        self.global_slots[slot_idx] = Some(sl);
                    }
                }

                // --- Name-Globals ---
                op::LOAD_NAME => {
                    let name = constants[arg.as_usize()].fmt();
                    let s = self.globals.get(&name)
                        .ok_or_else(|| format!("Variable '{}' nicht deklariert (DIM fehlt?){}", name, crate::umstieg::name_hinweis(&name)))?;
                    stack.push(s.borrow().value.clone());
                }
                op::STORE_NAME => {
                    let name = constants[arg.as_usize()].fmt();
                    let slot = self.globals.get(&name)
                        .ok_or_else(|| format!("Variable '{}' nicht deklariert (DIM fehlt?){}", name, crate::umstieg::name_hinweis(&name)))?.clone();
                    if slot.borrow().is_const {
                        return Err(format!("CONST '{}' kann nicht ueberschrieben werden", name));
                    }
                    let v = vm_pop(stack)?;
                    let ty = slot.borrow().ty.clone();
                    slot.borrow_mut().value = coerce(v, &ty, &format!("Zuweisung an '{}'", name))?;
                }
                op::DECLARE_NAME => {
                    let l = arg.list();
                    let name = constants[l[0].as_usize()].fmt();
                    let ty = constants[l[1].as_usize()].fmt();
                    let default = if let Some(vt) = ty.strip_prefix("map:") {
                        Value::Map(Rc::new(RefCell::new(DhMap::new(vt.to_string()))))
                    } else if ty.starts_with("array:") {
                        leeres_feld(&ty)
                    } else {
                        constants[l[2].as_usize()].clone()
                    };
                    self.globals.entry(name).or_insert_with(|| {
                        Rc::new(RefCell::new(Slot { ty, value: default, is_const: false }))
                    });
                }
                op::DECLARE_CONST => {
                    let l = arg.list();
                    let name = constants[l[0].as_usize()].fmt();
                    let value = vm_pop(stack)?;
                    // Review-Fund: `type_idx` ist IMMER ein gueltiger Konstanten-
                    // Index (der Compiler backt auch den untypisierten Fall als
                    // add_const(Null) ein) -- der Arg selbst ist also nie
                    // Arg::None, dieser Zweig war faktisch tot. Die eigentliche
                    // "untypisiert?"-Frage steckt im WERT der Konstante.
                    let ti = l[1].as_usize();
                    let (ty, value) = if matches!(constants[ti], Value::Nil) {
                        (infer_type(&value).to_string(), value)
                    } else {
                        let t = constants[ti].fmt();
                        let v = coerce(value, &t, "CONST")?;
                        (t, v)
                    };
                    self.globals.entry(name).or_insert_with(|| {
                        Rc::new(RefCell::new(Slot { ty, value, is_const: true }))
                    });
                }

                op::POW => { let b = vm_pop(stack)?; let a = vm_pop(stack)?; stack.push(pow(a, b)?); }
                // --- Bitwise ---
                op::BAND => { let (x, y) = int_pair2(stack, "BAND")?; stack.push(Value::Int(x & y)); }
                op::BOR => { let (x, y) = int_pair2(stack, "BOR")?; stack.push(Value::Int(x | y)); }
                op::BXOR => { let (x, y) = int_pair2(stack, "BXOR")?; stack.push(Value::Int(x ^ y)); }
                op::SHL => {
                    let (x, y) = int_pair2(stack, "SHL")?;
                    // Review-Fund: Rusts `<<` ist nur fuer 0..=63 definiert -- ohne
                    // diese Grenze maskiert der Compiler die Schiebeweite still
                    // (release, falsches Ergebnis) bzw. paniked (debug).
                    if !(0..64).contains(&y) { return Err(format!("SHL: Schiebeweite muss 0..63 sein, erhalten {}", y)); }
                    stack.push(Value::Int(x << y));
                }
                op::SHR => {
                    let (x, y) = int_pair2(stack, "SHR")?;
                    if !(0..64).contains(&y) { return Err(format!("SHR: Schiebeweite muss 0..63 sein, erhalten {}", y)); }
                    stack.push(Value::Int(x >> y));
                }
                op::BNOT => { let v = vm_pop(stack)?; match v { Value::Int(i) => stack.push(Value::Int(!i)), _ => return Err("BNOT erwartet INTEGER".into()) } }


                // --- Tupel ---
                op::BUILD_TUPLE => {
                    let len = arg.as_usize();
                    if len == 0 { stack.push(Value::Tuple(Rc::new(vec![]))); }
                    else {
                        let split = stack.len() - len;
                        let items = stack.split_off(split);
                        stack.push(Value::Tuple(Rc::new(items)));
                    }
                }
                op::BUILD_ARRAY => {
                    let len = arg.as_usize();
                    let split = stack.len() - len;
                    let items = stack.split_off(split);
                    stack.push(array_literal(items));
                }
                op::UNPACK_TUPLE => {
                    let len = arg.as_usize();
                    let t = vm_pop(stack)?;
                    if let Value::Tuple(items) = t {
                        if items.len() != len {
                            return Err(format!("Tupel-Destructuring: {} Ziele, aber Tupel hat {} Element(e)", len, items.len()));
                        }
                        for v in items.iter().rev() { stack.push(v.clone()); }
                    } else {
                        return Err(format!("UNPACK_TUPLE: Erwartet TUPLE, erhalten {}", t.type_name()));
                    }
                }
                op::BUILD_TUPLE_DYN => {
                    let mut idx = stack.len();
                    while idx > 0 && !matches!(stack[idx - 1], Value::CompMarker) { idx -= 1; }
                    if idx == 0 { return Err("BUILD_TUPLE_DYN: kein COMP_MARKER".into()); }
                    let items = stack.split_off(idx); // ab Element nach Marker
                    stack.pop(); // Marker
                    stack.push(Value::Tuple(Rc::new(items)));
                }
                op::IN_OP => {
                    let hay = vm_pop(stack)?;
                    let needle = vm_pop(stack)?;
                    stack.push(Value::Bool(eval_in(&needle, &hay)?));
                }
                op::SLICE => {
                    let l = arg.list();
                    let has_lo = matches!(l[0], Arg::Val(Value::Bool(true)) | Arg::Int(1)) || arg_truthy(&l[0]);
                    let has_hi = arg_truthy(&l[1]);
                    let hi = if has_hi { Some(vm_pop(stack)?) } else { None };
                    let lo = if has_lo { Some(vm_pop(stack)?) } else { None };
                    let target = vm_pop(stack)?;
                    stack.push(apply_slice(&target, lo.as_ref(), hi.as_ref())?);
                }

                op::LOAD_FUNCREF => {
                    let name = constants[arg.as_usize()].fmt();
                    if !self.prog.fn_index.contains_key(&name) {
                        return Err(format!("FUNCREF: Funktion '{}' existiert nicht", name));
                    }
                    stack.push(Value::FuncRef(Rc::from(name.as_str())));
                }
                op::CALL_VALUE => {
                    let (cname, argc, _) = call_parts(arg);
                    let split = stack.len() - argc;
                    let call_args = stack.split_off(split);
                    let callee = vm_pop(stack)?;
                    match callee {
                        Value::FuncRef(name) => {
                            let tgt = self.prog.func(name.as_ref())
                                .ok_or_else(|| format!("FUNCREF: Funktion '{}' existiert nicht (mehr)", name))?;
                            if tgt.is_coroutine {
                                stack.push(make_coro(tgt, call_args, None));
                            } else {
                                let ret = self.exec(tgt, call_args, None).map_err(|e| Self::param_von(e, tgt))?;
                                if !tgt.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                            }
                        }
                        // Gebundene Methode (`f = spieler.tick`): dieselbe Ausfuehrung
                        // wie CALL_METHOD, nur dass der Empfaenger aus dem Wert kommt
                        // statt vom Stack. Die Methode wird ERST JETZT aufgeloest --
                        // so trifft sie auch, wenn die Instanz inzwischen einer
                        // abgeleiteten Klasse angehoert.
                        Value::BoundMethod(b) => {
                            let (recv, mname) = (&b.0, &b.1);
                            let cn = match recv {
                                Value::Instance(rc) => rc.borrow().class_name.clone(),
                                other => return Err(format!(
                                    "FUNCREF: '{}' ist an {} gebunden, nicht an ein Objekt",
                                    mname, other.type_name())),
                            };
                            let tgt = self.resolve_method(&cn, mname).ok_or_else(||
                                format!("FUNCREF: Methode '{}' existiert nicht (mehr) in {}", mname, cn))?;
                            let recv = recv.clone();
                            if tgt.is_coroutine {
                                stack.push(make_coro(tgt, call_args, Some(recv)));
                            } else {
                                let ret = self.exec(tgt, call_args, Some(recv)).map_err(|e| Self::param_von(e, tgt))?;
                                if !tgt.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                            }
                        }
                        // Lambda mit Kopien: die Kopien vor die Argumente.
                        Value::Closure(c) => {
                            let tgt = self.prog.func(c.0.as_ref())
                                .ok_or_else(|| format!("FUNCREF: Funktion '{}' existiert nicht (mehr)", c.0))?;
                            let mut alle = c.1.clone();
                            alle.extend(call_args);
                            let ret = self.exec(tgt, alle, None).map_err(|e| Self::param_von(e, tgt))?;
                            if !tgt.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                        }
                        other => return Err(format!(
                            "'{}' ist eine Variable vom Typ {} und kann nicht wie eine Funktion \
                             aufgerufen werden. Falls du den eingebauten Befehl '{}' meinst: \
                             benenne die Variable um -- eine Variable mit festem Wert-Typ \
                             (INTEGER, FLOAT, STRING ...) verdeckt einen Befehl nicht, eine \
                             FUNCREF schon.",
                            cname, other.type_name(), cname.to_uppercase())),
                    }
                }
                // `SUPER.Methode(...)` (WP G). Fast wie CALL_METHOD -- nur
                // beginnt die Suche bei der im Bytecode stehenden Klasse
                // (der Elternklasse der Aufrufstelle) statt bei der Klasse
                // des Objekts. Sonst fande sie die ueberschreibende Methode
                // wieder und riefe sich selbst, bis der Stapel voll ist.
                op::CALL_METHOD_BYREF => {
                    let l = arg.list();
                    let method = l[0].str();
                    let argc = l[1].as_usize();
                    let stellen: Vec<usize> = match &l[2] {
                        Arg::List(s) => s.iter().map(|a| a.as_usize()).collect(),
                        Arg::Ints(s) => s.iter().map(|&a| a as usize).collect(),
                        _ => Vec::new(),
                    };
                    let split = stack.len() - argc;
                    let margs = stack.split_off(split);
                    let obj = vm_pop(stack)?;
                    let rc = match &obj {
                        Value::Instance(rc) => rc.clone(),
                        Value::Nil => return Err(format!(
                            "Methodenaufruf '.{}' bei NIL-Referenz{}", method, NIL_NEW)),
                        _ => return Err(format!("Methodenaufruf '.{}' bei nicht-Objekt ({})", method, obj.type_name())),
                    };
                    let cn = rc.borrow().class_name.clone();
                    let m = self.resolve_method(&cn, method)
                        .ok_or_else(|| format!("Methode '{}' existiert nicht in {}", method, cn))?;
                    // Eine Unterklasse, die die Methode mit anderen BYREF-Stellen
                    // ueberschreibt, passt nicht zu dem, was der Compiler erfasst hat.
                    let hier: Vec<usize> = (0..argc).filter(|&i| m.param_byref.get(i).copied().unwrap_or(false)).collect();
                    if hier != stellen {
                        return Err(format!(
                            "{}.{}: die Methode der Klasse {} hat ihre BYREF-Parameter an anderen Stellen \
                             als die, nach der der Aufruf uebersetzt ist -- eine Ueberschreibung muss \
                             dieselben Parameter BYREF nehmen", crate::namensraum::lesbar_text(&cn).to_uppercase(), method,
                            crate::namensraum::lesbar_text(&cn).to_uppercase()));
                    }
                    if m.is_coroutine {
                        return Err(format!("{}: eine Coroutine kann keine BYREF-Parameter zurueckschreiben", method));
                    }
                    let (ret, werte) = self.exec_byref(m, margs, Some(obj))?;
                    // exec_byref liefert ALLE BYREF-Parameter; zurueck geht nur,
                    // was der Aufruf uebergeben hat.
                    let mut k = 0;
                    for (i, &br) in m.param_byref.iter().enumerate() {
                        if br {
                            if i < argc { stack.push(werte[k].clone()); }
                            k += 1;
                        }
                    }
                    if !m.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                }

                op::CALL_SUPER => {
                    let l = arg.list();
                    let start_class = l[0].str();
                    let method = l[1].str();
                    let argc = l[2].as_usize();
                    let split = stack.len() - argc;
                    let margs = stack.split_off(split);
                    let obj = vm_pop(stack)?;
                    let m = self.resolve_method(start_class, method).ok_or_else(|| format!(
                        "SUPER.{}: Methode existiert nicht in {}", method, start_class))?;
                    if m.is_coroutine {
                        stack.push(make_coro(m, margs, Some(obj)));
                    } else {
                        let ret = self.exec(m, margs, Some(obj))?;
                        if !m.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                    }
                }

                // --- OOP ---
                op::NEW_INSTANCE => {
                    let l = arg.list();
                    let class_name = l[0].str();
                    let argc = l[1].as_usize();
                    let has_init_args = arg_truthy(&l[2]);
                    // Merkplatz: die Klasse (der Name im Befehl steht fest).
                    let (_, ci_p) = instr.merk.get();
                    let ci: &'p ClassInfo = if ci_p != 0 {
                        unsafe { &*(ci_p as *const ClassInfo) }
                    } else {
                        let ci = self.prog.classes.get(class_name)
                            .ok_or_else(|| format!("Klasse '{}' nicht gefunden", class_name))?;
                        instr.merk.set((Rc::as_ptr(&ci.layout), ci as *const ClassInfo as usize));
                        ci
                    };
                    let vorlage = self.vorlage(class_name, ci);
                    let inst = self.objekt_aus(vorlage);
                    if has_init_args {
                        let split = stack.len() - argc;
                        if let Some(init) = vorlage.init {
                            let init: &'p Func = unsafe { &*init };
                            self.exec(init, stack.drain(split..), Some(inst.clone()))?;
                        } else if argc != 0 {
                            return Err(format!("Klasse {} hat keine SUB Init - Argumente bei NEW nicht moeglich", class_name));
                        } else {
                            stack.truncate(split);
                        }
                    }
                    stack.push(inst);
                }
                op::DECLARE_STRUCT_NAME => {
                    let l = arg.list();
                    let name = constants[l[0].as_usize()].fmt();
                    let class_name = l[1].str();
                    if !self.prog.classes.contains_key(class_name) {
                        return Err(format!("STRUCT '{}' nicht gefunden", class_name));
                    }
                    if !self.globals.contains_key(&name) {
                        let inst = self.allocate_instance(class_name);
                        self.globals.insert(name, Rc::new(RefCell::new(Slot { ty: class_name.to_string(), value: inst, is_const: false })));
                    }
                }
                op::TYP_PRUEFEN => {
                    // Nur mit DHRT_TYPEN_PRUEFEN uebersetzt: sagt der Compiler
                    // (`typ_von`) etwas, das zur Laufzeit nicht stimmt, ist das
                    // ein Fehler im COMPILER -- darum ein eigener Wortlaut.
                    let angabe = constants[arg.as_usize()].fmt();
                    let v = stack.last().unwrap_or(&Value::Nil);
                    let stimmt = match crate::typen::passt(&angabe, v) {
                        Ok(b) => b,
                        // Eine Klasse: NIL oder eine Instanz, die abstammt.
                        Err(k) => match v {
                            Value::Nil => true,
                            Value::Instance(rc) => self.klasse_stammt_ab(&rc.borrow().class_name, &k),
                            _ => false,
                        },
                    };
                    if !stimmt {
                        return Err(format!("Typpruefung (DHRT_TYPEN_PRUEFEN): der Compiler sagt {}, der Wert ist {} ({}) -- ein Fehler in compiler.rs typ_von",
                            angabe, v.type_name(), v.fmt()));
                    }
                }
                op::BIND_GLOBAL_SLOT => {
                    // Den eben unter dem Namen angelegten Eintrag (Feld, Map,
                    // Struktur) auch in seinen Platz haengen -- dasselbe Rc,
                    // also sehen Namens- und Platz-Zugriffe denselben Wert.
                    // Nach jedem Anlegen neu: ein DIM in einer Schleife legt
                    // unter dem Namen ein NEUES Feld an.
                    let l = arg.list();
                    let idx = l[0].as_usize();
                    let name = constants[l[1].as_usize()].fmt();
                    if let Some(s) = self.globals.get(&name) {
                        self.global_slots[idx] = Some(s.clone());
                    }
                }
                op::DECLARE_STRUCT_LOCAL => {
                    let l = arg.list();
                    let slot = l[0].as_usize();
                    let class_name = l[1].str();
                    if matches!(locals[slot], Value::Nil) {
                        if !self.prog.classes.contains_key(class_name) {
                            return Err(format!("STRUCT '{}' nicht gefunden", class_name));
                        }
                        locals[slot] = self.allocate_instance(class_name);
                    }
                }

                op::DECLARE_ARRAY_NAME => {
                    let l = arg.list();
                    let name = constants[l[0].as_usize()].fmt();
                    let elem_type = l[1].str().to_string();
                    let num_dims = l[2].as_usize();
                    let (dims, unten) = self.pop_feld_dims(stack, l, num_dims)?;
                    let et = elem_type.clone();
                    let arr = DhArray::new(elem_type.clone(), dims, || self.element_default(&et)).mit_unten(unten);
                    self.globals.insert(name, Rc::new(RefCell::new(Slot {
                        ty: format!("array:{}", elem_type),
                        value: Value::Array(Rc::new(RefCell::new(arr))),
                        is_const: false,
                    })));
                }
                op::DECLARE_ARRAY_LOCAL => {
                    let l = arg.list();
                    let slot = l[0].as_usize();
                    let elem_type = l[1].str().to_string();
                    let num_dims = l[2].as_usize();
                    let (dims, unten) = self.pop_feld_dims(stack, l, num_dims)?;
                    let et = elem_type.clone();
                    let arr = DhArray::new(elem_type, dims, || self.element_default(&et)).mit_unten(unten);
                    locals[slot] = Value::Array(Rc::new(RefCell::new(arr)));
                }

                // --- Exceptions ---
                op::TRY_BEGIN => try_handlers.push((arg.as_usize(), stack.len())),
                op::TRY_END => { try_handlers.pop(); }
                // Ende des FINALLY-Blocks auf dem FEHLER-Weg (WP F): der
                // Fehler, der uns hierher gebracht hat, liegt oben auf dem
                // Stack und geht jetzt weiter nach aussen. Auf dem normalen
                // Weg wird dieser Opcode nie erreicht -- dort steht eine
                // zweite Kopie des Blocks ohne ihn (siehe compiler::stmt_try).
                op::FIN_END => {
                    let v = vm_pop(stack)?;
                    let msg = match v { Value::Str(s) => s.to_string(), other => other.fmt() };
                    // Das ist ein WEITERwerfen, kein neuer Fehler: Fundstelle
                    // und Code beschreiben weiterhin die urspruengliche
                    // Stelle. Ohne dieses Flag saehe run_frame einen frischen
                    // Fehler, wuerde ERROR_LINE auf das Ende des
                    // FINALLY-Blocks setzen und ERROR_CODE$ leeren -- ein
                    // FINALLY dazwischen loeschte also die Angaben, wegen
                    // derer man sie ueberhaupt abfragt.
                    self.rethrow = true;
                    return Err(msg);
                }
                op::THROW => {
                    let v = vm_pop(stack)?;
                    let msg = match v { Value::Str(s) => s.to_string(), other => other.fmt() };
                    // Mit Code (`THROW code, meldung`) liegt der Code darunter.
                    // `matches!` statt `as_i64()`: die einstellige Form gibt
                    // gar kein Argument mit (Arg::Null), und `as_i64()` wuerde
                    // darauf panisch abbrechen statt "kein Code" zu bedeuten.
                    self.throw_code = if matches!(arg, crate::model::Arg::Int(2)) {
                        let c = vm_pop(stack)?;
                        match c { Value::Str(s) => s.to_string(), other => other.fmt() }
                    } else {
                        String::new()
                    };
                    // Markiert genau diesen einen Fehler als "von THROW". Die
                    // Auswertung passiert im innersten run_frame, das den
                    // Fehler als erstes sieht -- dazwischen kann kein anderer
                    // entstehen (siehe dort).
                    self.throw_active = true;
                    return Err(msg);
                }

                // --- DATA ---
                op::PUSH_DATA => {
                    if self.data_ptr >= self.prog.data.len() {
                        return Err("READ: keine DATA-Werte mehr (benutze RESTORE zum Reset)".into());
                    }
                    stack.push(self.prog.data[self.data_ptr].clone());
                    self.data_ptr += 1;
                }
                op::RESET_DATA_PTR => self.data_ptr = 0,

                op::YIELD_VALUE => {
                    // Coroutine: Wert abgeben und suspendieren. Beim Resume legt
                    // coro_resume den Sende-Wert auf den Stack (Wert von `x = YIELD`).
                    let mut yval = vm_pop(stack)?;
                    if !fn_.return_type.is_empty() {
                        yval = coerce(yval, &fn_.return_type, "YIELD")?;
                    }
                    return Ok(Some(Step::Yield(yval)));
                }

                // --- I/O ---
                // Arg: [count, newline, sep0, sep1, ...] -- sep_i = Trenner ZWISCHEN
                // item_i und item_{i+1}: "," -> Leerzeichen, ";" -> kein Trennzeichen.
                // Trailing-Trenner setzt newline=false (kein Zeilenumbruch).
                op::PRINT => {
                    let count = arg.list()[0].as_usize();
                    let items = stack.split_off(stack.len() - count);
                    self.drucken(arg, &items);
                }

                // --- INPUT (Konsolen-Eingabe) ---
                // Semantik 1:1 aus interpreter._exec_Input: Prompt (Default "? ",
                // sonst mit Leerzeichen-Suffix) live ausgeben, eine Zeile von
                // stdin lesen, auf den Ziel-Typ coercen. Da PRINT in self.out
                // puffert, wird der bisherige Output + Prompt VOR dem Lesen real
                // geflusht (sonst erscheint der Prompt erst nach der Eingabe).
                op::INPUT_NAME => {
                    let l = arg.list();
                    let name = constants[l[0].as_usize()].fmt();
                    let prompt = if arg_truthy(&l[1]) {
                        input_prompt(&vm_pop(stack)?.fmt())
                    } else { "? ".to_string() };
                    self.flush_and_prompt(&prompt);
                    let raw = self.eingabe_zeile()?;
                    let slot = self.globals.get(&name)
                        .ok_or_else(|| format!("Variable '{}' nicht deklariert (DIM fehlt?){}", name, crate::umstieg::name_hinweis(&name)))?
                        .clone();
                    if slot.borrow().is_const {
                        return Err(format!("CONST '{}' kann nicht ueberschrieben werden", name));
                    }
                    let ty = slot.borrow().ty.clone();
                    slot.borrow_mut().value = coerce_input(&raw, &ty)?;
                }
                op::INPUT_LOCAL => {
                    let l = arg.list();
                    let slot_idx = l[0].as_usize();
                    let prompt = if arg_truthy(&l[1]) {
                        input_prompt(&vm_pop(stack)?.fmt())
                    } else { "? ".to_string() };
                    self.flush_and_prompt(&prompt);
                    let raw = self.eingabe_zeile()?;
                    let ty = fn_.local_types[slot_idx].clone();
                    locals[slot_idx] = coerce_input(&raw, &ty)?;
                }

                op::HALT => return Ok(Some(Step::Return(Value::Nil))),

                other => return Err(format!("Opcode {} im Rust-VM noch nicht implementiert (ip={})", other, *ip - 1)),
        }
        Ok(None)
    }

    /// Groessen und Untergrenzen eines DIM mit Feld vom Stapel nehmen. Das
    /// vierte Argument des Befehls (`grenzen`, fehlt in alten .dhc) sagt je
    /// Dimension, ob sie als Bereich `von TO bis` kam (zwei Werte) oder als
    /// Groesse (einer).
    fn pop_feld_dims(&self, stack: &mut Vec<Value>, l: &[Arg], num_dims: usize) -> R<(Vec<i64>, Vec<i64>)> {
        let bereich: Vec<bool> = match l.get(3) {
            Some(Arg::List(b)) => b.iter().map(|x| x.as_usize() != 0).collect(),
            _ => vec![false; num_dims],
        };
        if !bereich.iter().any(|&b| b) {
            return Ok((self.pop_dims(stack, num_dims)?, Vec::new()));
        }
        let n_werte: usize = bereich.iter().map(|&b| if b { 2 } else { 1 }).sum();
        let split = stack.len() - n_werte;
        let raw = stack.split_off(split);
        let mut it = raw.into_iter();
        let mut groessen = Vec::with_capacity(num_dims);
        let mut unten = Vec::with_capacity(num_dims);
        for (k, &b) in bereich.iter().enumerate() {
            if b {
                let (von, bis) = (it.next().unwrap(), it.next().unwrap());
                let (Value::Int(lo), Value::Int(hi)) = (&von, &bis) else {
                    return Err(format!("DIM: die Grenzen eines Feldes muessen INTEGER sein, erhalten {} TO {}",
                                       von.type_name(), bis.type_name()));
                };
                if *hi < lo.saturating_sub(1) {
                    return Err(format!("DIM: in Dimension {} liegt die Obergrenze {} unter der Untergrenze {} -- [{} TO {}] ist schon leer",
                                       k + 1, hi, lo, lo, lo.saturating_sub(1)));
                }
                let n = hi.checked_sub(*lo).and_then(|d| d.checked_add(1)).ok_or_else(||
                    "Array-Groesse: Ganzzahl-Ueberlauf beim Berechnen der Gesamtgroesse".to_string())?;
                groessen.push(Value::Int(n));
                unten.push(*lo);
            } else {
                groessen.push(it.next().unwrap());
                unten.push(0);
            }
        }
        stack.extend(groessen);
        Ok((self.pop_dims(stack, num_dims)?, unten))
    }

    fn pop_dims(&self, stack: &mut Vec<Value>, num_dims: usize) -> R<Vec<i64>> {
        let split = stack.len() - num_dims;
        let raw = stack.split_off(split);
        let mut dims = Vec::with_capacity(num_dims);
        for d in raw {
            match d {
                Value::Int(i) if i >= 0 => dims.push(i),
                _ => return Err("Array-Groesse muss INTEGER >= 0 sein".into()),
            }
        }
        // Review-Fund: DhArray::new multipliziert die Dimensionen ungeprueft
        // (`acc *= dims[k]`) -- `DIM a[4294967296, 4294967296]` ueberlief den
        // i64-Akkumulator lautlos zu einem kleinen/negativen Wert (eine
        // Groessen-Diskrepanz, die spaeter beim Indexzugriff in einen rohen
        // Index-Out-Of-Bounds-Panic lief), und ein legitimer, aber riesiger
        // Wert wie `DIM a[100000, 100000]` (1e10 Elemente, ~80 GB) fuehrte zu
        // einem harten Allocator-Abort statt eines DH-Fehlers. Hier einmalig
        // mit checked_mul + einer Obergrenze validieren, BEVOR DhArray::new
        // (das selbst keinen Result-Rueckgabewert hat) ueberhaupt aufgerufen wird.
        const MAX_ARRAY_ELEMENTS: i64 = 100_000_000;
        let mut total: i64 = 1;
        for &d in &dims {
            total = total.checked_mul(d).ok_or_else(||
                "Array-Groesse: Ganzzahl-Ueberlauf beim Berechnen der Gesamtgroesse".to_string())?;
            if total > MAX_ARRAY_ELEMENTS {
                return Err(format!(
                    "Array-Groesse zu gross ({} Elemente, max. {})", total, MAX_ARRAY_ELEMENTS));
            }
        }
        Ok(dims)
    }

    /// Quell-Zeile des zuletzt ausgefuehrten Befehls (fuer Fehlermeldungen).
    /// 0 = unbekannt (z.B. wenn der Compiler keine Zeilen getrackt hat).
    pub fn error_line(&self) -> u32 {
        self.cur_line
    }

    /// FFI-Rueckrufe (2026-10-14): ein Fehler im Rueckruf wird nie per CATCH
    /// "verbraucht", `err_line_set` bliebe also stehen, und jeder spaetere
    /// Fehler trug die Zeile des ersten. Jeder Rueckruf beginnt darum frisch.
    #[cfg(feature = "ffi")]
    pub(crate) fn fehlerzeile_vergessen(&mut self) { self.err_line_set = false; }

    /// Die Zeile des ersten Rueckruf-Fehlers wieder einsetzen, wenn der Aufruf
    /// der Bibliothek ihn meldet -- sonst stuende dort die des letzten.
    #[cfg(feature = "ffi")]
    pub(crate) fn fehlerzeile_setzen(&mut self, zeile: u32) {
        if zeile == 0 { return; }
        self.cur_line = zeile;
        self.error_line = zeile;
        self.err_line_set = true;
    }

    /// Zugriff auf den frame_count (fuer main.rs-Schleifenlogik), 0 ohne Grafik.
    #[allow(dead_code)]
    pub fn frame_count(&self) -> u64 {
        #[cfg(feature = "graphics")]
        { return self.gfx.as_ref().map(|g| g.frame_count).unwrap_or(0); }
        #[allow(unreachable_code)]
        0
    }

    fn scene_top(&self) -> R<&(String, HashMap<String, Value>)> {
        self.scene_stack.last().ok_or_else(|| "Scene-Stack ist leer - SCENE_PUSH oder SCENE_SWITCH zuerst".to_string())
    }
    fn scene_top_mut(&mut self) -> R<&mut (String, HashMap<String, Value>)> {
        self.scene_stack.last_mut().ok_or_else(|| "Scene-Stack ist leer - SCENE_PUSH oder SCENE_SWITCH zuerst".to_string())
    }

    /// Gemeinsamer Handle-Lookup fuer die Netz-/Hardware-Module (db/net/serial/
    /// usb/bt): `vec[idx]` als `Some(_)`, sonst ein einheitlich formatierter
    /// Fehler `"{modul}: {was} {idx}"`. `was` traegt bewusst den vollen,
    /// modulspezifischen Wortlaut (z.B. "getrenntes BT_HANDLE" vs.
    /// "geschlossenes DB_CONN-Handle") -- nur das Lookup-Skelett war dupliziert.
    #[cfg(any(feature = "db", feature = "net", feature = "serial", feature = "usb", feature = "bt"))]
    fn handle_get<'a, T>(vec: &'a [Option<T>], idx: i64, modul: &str, was: &str) -> R<&'a T> {
        vec.get(idx as usize).and_then(|o| o.as_ref())
            .ok_or_else(|| format!("{}: {} {}", modul, was, idx))
    }
    // Ohne Feature-Gate: `pdf` und `xlsx` brauchen denselben Helfer und
    // haengen an keinem Feature -- ein Konsolenprogramm soll eine Rechnung
    // schreiben koennen, ohne dass das Netz mit eingebaut ist.
    fn handle_get_mut<'a, T>(vec: &'a mut [Option<T>], idx: i64, modul: &str, was: &str) -> R<&'a mut T> {
        vec.get_mut(idx as usize).and_then(|o| o.as_mut())
            .ok_or_else(|| format!("{}: {} {}", modul, was, idx))
    }

}

// Vordefinierte Globals (Farben, Tasten). Bis 2026-09-21 standen dieselben
// Werte in drachenhauch/graphics.py und ein pytest hielt beide gleich; seit
// dem Loeschen des Python-Teils ist das hier die einzige Quelle.
/// Ist das einer der vorbelegten Namen (Farben, Tasten, pi, tau)?
///
/// Sie bekommen beim Uebersetzen KEINEN Slot -- der Compiler kennt sie nicht
/// statisch, sie laufen ueber `LOAD_NAME` (siehe "Globals-as-Slots"). Fuer
/// die Tippfehler-Warnung in `compiler.rs` ist genau das die noetige
/// Ausnahme: ohne sie waere jedes `KEY_SPACE` ein Befund.
///
/// `name` muss lowercase sein -- Drachenhauch unterscheidet Gross- und
/// Kleinschreibung bei Namen nicht.
pub fn ist_vorbelegter_name(name: &str) -> bool {
    name == "pi" || name == "tau"
        || DEFAULT_COLORS.iter().any(|(n, _)| *n == name)
        || DEFAULT_KEYS.iter().any(|(n, _)| *n == name)
}

pub(crate) const DEFAULT_COLORS: &[(&str, i64)] = &[
    ("black", 0), ("white", 16777215), ("gray", 8421504), ("lightgray", 12632256),
    ("darkgray", 4210752), ("red", 16711680), ("green", 65280), ("blue", 255),
    ("yellow", 16776960), ("cyan", 65535), ("magenta", 16711935), ("orange", 16753920),
    ("purple", 8388736), ("brown", 9127187), ("pink", 16761035), ("darkred", 8388608),
    ("darkgreen", 32768), ("darkblue", 128),
];

pub(crate) const DEFAULT_KEYS: &[(&str, i64)] = &[
    ("key_escape", 27), ("key_return", 13), ("key_enter", 13), ("key_space", 32),
    ("key_tab", 9), ("key_backspace", 8),
    ("key_left", 1073741904), ("key_right", 1073741903), ("key_up", 1073741906), ("key_down", 1073741905),
    ("key_a", 97), ("key_b", 98), ("key_c", 99), ("key_d", 100), ("key_e", 101), ("key_f", 102),
    ("key_g", 103), ("key_h", 104), ("key_i", 105), ("key_j", 106), ("key_k", 107), ("key_l", 108),
    ("key_m", 109), ("key_n", 110), ("key_o", 111), ("key_p", 112), ("key_q", 113), ("key_r", 114),
    ("key_s", 115), ("key_t", 116), ("key_u", 117), ("key_v", 118), ("key_w", 119), ("key_x", 120),
    ("key_y", 121), ("key_z", 122),
    ("key_0", 48), ("key_1", 49), ("key_2", 50), ("key_3", 51), ("key_4", 52),
    ("key_5", 53), ("key_6", 54), ("key_7", 55), ("key_8", 56), ("key_9", 57),
    ("key_f1", 1073741882), ("key_f2", 1073741883), ("key_f3", 1073741884), ("key_f4", 1073741885),
    ("key_f5", 1073741886), ("key_f6", 1073741887), ("key_f7", 1073741888), ("key_f8", 1073741889),
    ("key_f9", 1073741890), ("key_f10", 1073741891), ("key_f11", 1073741892), ("key_f12", 1073741893),
    // Umschalt-/Steuertasten: bis hierher gab es KEINEN Code dafuer -- ein
    // "Sprint mit Shift" oder "Strg+S" war aus Drachenhauch heraus nicht abfragbar.
    ("key_lshift", 1073742049), ("key_rshift", 1073742053),
    ("key_lctrl", 1073742048), ("key_rctrl", 1073742052),
    ("key_lalt", 1073742050), ("key_ralt", 1073742054),
    ("key_lsuper", 1073742051), ("key_rsuper", 1073742055),
    ("key_capslock", 1073741881),
    // Navigationsblock
    ("key_insert", 1073741897), ("key_delete", 127),
    ("key_home", 1073741898), ("key_end", 1073741901),
    ("key_pageup", 1073741899), ("key_pagedown", 1073741902),
    // Ziffernblock (eigene Codes -- eine Spiel-Steuerung darf ihn getrennt belegen)
    ("key_kp0", 1073741922), ("key_kp1", 1073741913), ("key_kp2", 1073741914),
    ("key_kp3", 1073741915), ("key_kp4", 1073741916), ("key_kp5", 1073741917),
    ("key_kp6", 1073741918), ("key_kp7", 1073741919), ("key_kp8", 1073741920),
    ("key_kp9", 1073741921),
    ("key_kp_enter", 1073741912), ("key_kp_plus", 1073741911), ("key_kp_minus", 1073741910),
    ("key_kp_multiply", 1073741909), ("key_kp_divide", 1073741908), ("key_kp_period", 1073741923),
    // Gamepad-Bind-Codes (negativ, kollidieren nicht mit Tasten) -- wie graphics.KEYS.
    ("joy_button_a", -100), ("joy_button_b", -101), ("joy_button_x", -102), ("joy_button_y", -103),
    ("joy_button_lb", -104), ("joy_button_rb", -105), ("joy_button_back", -106), ("joy_button_start", -107),
    ("joy_button_lstick", -108), ("joy_button_rstick", -109),
    ("joy_dpad_up", -200), ("joy_dpad_down", -201), ("joy_dpad_left", -202), ("joy_dpad_right", -203),
];
