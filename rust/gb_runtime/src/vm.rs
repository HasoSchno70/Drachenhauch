//! VM-Kern (Schritt 2+3): stack-basierte Dispatch-Schleife.
//!
//! Skalar-Ops, Control-Flow, User-Calls, Strings/Arrays/Maps/Tupel, OOP
//! (Structs/Klassen/Methoden/Properties/Operator-Overloading), Slicing, IN,
//! DATA/READ, TRY/THROW und die puren Builtins (siehe builtins.rs).
//! Semantik 1:1 aus `gamebasic/vm.py`, damit `stdout` bit-identisch bleibt.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Sentinel-"Fehler", mit dem ein externes Stop-Signal (`gbrt profile`,
/// Editor-Stop-Button) die Dispatch-Schleife sauber abwickelt -- darf wie
/// `__DEBUG_STOP__` NICHT von TRY/CATCH gefangen werden.
const PROFILE_STOP: &str = "__PROFILE_STOP__";

use crate::builtins;
use crate::model::{op, Arg, ClassInfo, Func, Program};
use crate::value::{as_f64, is_num, value_eq, Cells, CoroState, FieldVal, GbArray, GbMap, Instance, Value};

/// Profiler-Sammler (Stufe B, Phase 3): pro Quell-Zeile Besuchs-Count +
/// kumulierte Zeit. Spiegelt `editor_qt/profiler.py`: die Zeit zwischen zwei
/// Zeilenwechseln wird der VORIGEN Zeile zugeschlagen (inkl. der von dort
/// gerufenen Funktionen). Aggregation pro SUB/FUNCTION macht der Editor via
/// `symbols.scan_scopes` -- gbrt liefert nur die rohen Zeilen-Daten.
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
/// faellt fuer rohe Listen (z.B. alte .gbc ohne specialize-Pass) zurueck.
#[inline]
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

fn bind_params(fn_: &Func, args: Vec<Value>, mut locals: Vec<Value>) -> R<Vec<Value>> {
    // `locals` ist ein (ggf. gepoolter) Buffer -- Allokation wird
    // wiederverwendet, Inhalt kommt frisch aus den local_defaults.
    locals.clear();
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
            locals[i] = coerce(v, &fn_.local_types[i], "Parameter")?;
        }
        let rest: Vec<Value> = it.collect();
        locals[fn_.n_params - 1] = Value::Tuple(Rc::new(rest));
    } else {
        let n_required = if fn_.n_required == 0 { fn_.n_params } else { fn_.n_required };
        if args.len() < n_required || args.len() > fn_.n_params {
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
                locals[i] = coerce(v, &fn_.local_types[i], "Parameter")?;
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
/// kann nur bei kaputtem/abgeschnittenem `.gbc` (oder einem Compiler-Bug)
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
#[allow(dead_code)]
fn bi_num(a: &[Value], i: usize, fn_: &str) -> R<f64> {
    match a.get(i) {
        Some(Value::Int(n)) => Ok(*n as f64),
        Some(Value::Float(f)) => Ok(*f),
        Some(v) => Err(format!("{}: erwartet Zahl, erhalten {}", fn_, v.type_name())),
        None => Err(format!("{}: fehlendes Argument {}", fn_, i + 1)),
    }
}
#[cfg(feature = "db")]
fn db_params(args: &[Value], fn_: &str) -> R<Vec<rusqlite::types::Value>> {
    args.iter().map(|v| crate::db::gb_to_sql(v, fn_)).collect()
}

struct Slot {
    ty: String,
    value: Value,
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
];

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

/// Ruft ein reines Builtin auf und faengt einen evtl. Rust-Panic ab (z.B.
/// Index-out-of-bounds bei zu wenigen Argumenten in variadischen Builtins),
/// damit ein Tippfehler im GB-Programm NICHT die Runtime abstuerzen laesst,
/// sondern einen klaren Laufzeitfehler liefert.
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

// --- Debugger (Stufe B, `gbrt debug`) ---------------------------------------
#[derive(PartialEq, Clone, Copy)]
enum StepMode { Run, Over, Into, Out }

struct DebugState {
    // Zeile -> optionale (bereits geparste) Bedingung.
    breakpoints: HashMap<u32, Option<crate::ast::Node>>,
    step: StepMode,
    step_depth: u32,
    out_sent: usize,   // wie viel von vm.out schon als output-Event gesendet wurde
}

impl DebugState {
    fn new() -> Self {
        // Beim Start an der ersten Zeile anhalten -> der Editor setzt dann
        // Breakpoints und schickt `continue`.
        DebugState { breakpoints: HashMap::new(), step: StepMode::Into,
                     step_depth: 0, out_sent: 0 }
    }
}

/// Eine Zeile JSON-Kommando von stdin lesen (blockierend). None bei EOF.
fn dbg_read_cmd() -> Option<serde_json::Value> {
    use std::io::BufRead;
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line).unwrap_or(0) == 0 {
        return None;
    }
    serde_json::from_str(line.trim()).ok()
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
                return Ok(Value::str_rc(&format!("{}{}", x, y)));
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
        _ => Err(format!("eval: Operator '{}' nicht unterstuetzt", op)),
    }
}

pub struct Vm<'p> {
    prog: &'p Program,
    globals: HashMap<String, Rc<RefCell<Slot>>>,
    global_slots: Vec<Option<Rc<RefCell<Slot>>>>,
    data_ptr: usize,
    out: String,
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
    // Quell-Zeile der zuletzt ausgefuehrten Instruktion (fuer Laufzeitfehler-
    // Meldungen). Bei einem propagierenden Fehler haelt es die Zeile der
    // innersten fehlschlagenden Instruktion (sie lief zuletzt). 0 = unbekannt.
    cur_line: u32,
    // Lazy-Fehlerzeile: gesetzt vom INNERSTEN run_frame beim ersten Fehler,
    // von TRY/CATCH beim Konsumieren geloescht (s. run_frame).
    err_line_set: bool,
    // Profiler-Sink (Stufe B): None = kein Profiling-Overhead (Normalfall).
    prof: Option<ProfileSink>,
    // Externes Stop-Signal (Editor-Stop-Button bei `gbrt profile --stoppable`):
    // bei gesetztem Flag bricht die Dispatch-Schleife beim naechsten
    // Zeilenwechsel sauber ab (PROFILE_STOP), damit die bis dahin gesammelten
    // Profile-Daten noch ausgegeben werden (kein verlorener Prozess-Kill).
    stop: Option<Arc<AtomicBool>>,
    // Debugger-State (Stufe B): None = kein Debug-Overhead. Call-Tiefe fuer
    // Step over/into/out (inkrementiert pro `exec`).
    dbg: Option<DebugState>,
    depth: u32,
    #[cfg(feature = "graphics")]
    gfx: Option<crate::graphics::Graphics>,
    // Audio-Geraet (lazy bei erstem LOADSOUND/PLAYSOUND/PLAYMUSIC initialisiert).
    #[cfg(feature = "graphics")]
    audio: Option<crate::audio::Audio>,
    // Retained-Mode-GUI (Modul gui): persistente Fenster/Widgets.
    #[cfg(feature = "graphics")]
    gui: crate::gui::Gui,
    // Modul db (SQLite): Verbindungen + (eager geladene) Resultsets, INTEGER-Handles.
    #[cfg(feature = "db")]
    db_conns: Vec<Option<rusqlite::Connection>>,
    #[cfg(feature = "db")]
    db_results: Vec<crate::db::DbResult>,
    // Modul net: TCP-Listener/-Sockets + UDP-Sockets, INTEGER-Handles.
    #[cfg(feature = "net")]
    tcp_listeners: Vec<Option<(std::net::TcpListener, i64)>>,
    #[cfg(feature = "net")]
    tcp_socks: Vec<Option<crate::net::NetSock>>,
    #[cfg(feature = "net")]
    udp_socks: Vec<Option<crate::net::UdpSock>>,
    // Modul html: letzter HTTP-Status/-Header (fuer HTTP_STATUS/HTTP_HEADER).
    #[cfg(feature = "http")]
    http_status: i64,
    #[cfg(feature = "http")]
    http_headers: Vec<(String, String)>,
    // Hardware/IoT-Handles (INTEGER-Index in VM-Vecs).
    #[cfg(feature = "serial")]
    serial_ports: Vec<Option<crate::serial::Port>>,
    #[cfg(feature = "usb")]
    usb_devs: Vec<Option<hidapi::HidDevice>>,
    #[cfg(feature = "bt")]
    bt_periphs: Vec<Option<btleplug::platform::Peripheral>>,
}

type R<T> = Result<T, String>;

impl<'p> Vm<'p> {
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
            input_state: InputModule::default(),
            ui_state: UiState::new(),
            scene_stack: Vec::new(),
            timers: crate::timer::Timers::default(),
            pool_locals: Vec::new(),
            pool_stacks: Vec::new(),
            cur_line: 0,
            err_line_set: false,
            prof: None,
            stop: None,
            dbg: None,
            depth: 0,
            #[cfg(feature = "graphics")]
            gfx: None,
            #[cfg(feature = "graphics")]
            audio: None,
            #[cfg(feature = "graphics")]
            gui: crate::gui::Gui::new(),
            #[cfg(feature = "db")]
            db_conns: Vec::new(),
            #[cfg(feature = "db")]
            db_results: Vec::new(),
            #[cfg(feature = "net")]
            tcp_listeners: Vec::new(),
            #[cfg(feature = "net")]
            tcp_socks: Vec::new(),
            #[cfg(feature = "net")]
            udp_socks: Vec::new(),
            #[cfg(feature = "http")]
            http_status: 0,
            #[cfg(feature = "http")]
            http_headers: Vec::new(),
            #[cfg(feature = "serial")]
            serial_ports: Vec::new(),
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

    pub fn take_output(self) -> String {
        self.out
    }

    /// Fuer INPUT: gepufferten Output + Prompt SOFORT auf echtes stdout flushen
    /// (sonst erscheint der Prompt erst nach der Eingabe) und `self.out` leeren,
    /// damit der finale take_output() nichts doppelt schreibt.
    fn flush_and_prompt(&mut self, prompt: &str) {
        // Unter dem Profiler gibt es keine interaktive Konsole -> Prompt in den
        // Output-Puffer (landet im JSON-`output`-Feld), damit stdout sauber fuer
        // den JSON-Blob bleibt (sonst klebt das prompt-Praefix am JSON).
        if self.prof.is_some() {
            self.out.push_str(prompt);
            return;
        }
        use std::io::Write;
        let so = std::io::stdout();
        let mut h = so.lock();
        let _ = h.write_all(self.out.as_bytes());
        self.out.clear();
        let _ = h.write_all(prompt.as_bytes());
        let _ = h.flush();
    }

    /// Profiling fuer den naechsten `run()` aktivieren (Stufe B, `gbrt profile`).
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
    pub fn was_stopped(err: &str) -> bool {
        err == PROFILE_STOP
    }

    /// Eine INPUT-Zeile lesen. Bei aktivem Stop-Kanal (`gbrt profile
    /// --stoppable`) gehoert stdin dem Stop-Reader-Thread -- INPUT liefert dann
    /// "" (wie der frueher genutzte DEVNULL-stdin), statt zu blockieren oder mit
    /// dem Stop-Reader um die Eingabe zu konkurrieren.
    fn read_input_line(&self) -> String {
        if self.stop.is_some() {
            return String::new();
        }
        read_input_line()
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

    // ---------------------------------------------------------------- OOP
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

    fn is_property(&self, class_name: &str, name: &str) -> bool {
        // Member-Namen liegen lowercase vor (Compiler) -- nur im seltenen
        // gemischten Fall allozieren.
        let lowered;
        let target: &str = if name.bytes().any(|b| b.is_ascii_uppercase()) {
            lowered = name.to_lowercase();
            &lowered
        } else { name };
        let mut cur = self.prog.classes.get(class_name);
        while let Some(ci) = cur {
            if ci.properties.contains(target) {
                return true;
            }
            if ci.parent_name.is_empty() {
                break;
            }
            cur = self.prog.classes.get(&ci.parent_name);
        }
        false
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
            _ => Value::Nil,
        }
    }

    fn allocate_instance(&self, class_name: &str) -> Value {
        let mut fields: rustc_hash::FxHashMap<String, FieldVal> = rustc_hash::FxHashMap::default();
        // Kette parent-first sammeln.
        let mut chain: Vec<&ClassInfo> = Vec::new();
        let mut cur = self.prog.classes.get(class_name);
        while let Some(ci) = cur {
            chain.push(ci);
            cur = if ci.parent_name.is_empty() { None } else { self.prog.classes.get(&ci.parent_name) };
        }
        for ci in chain.iter().rev() {
            for fd in &ci.fields {
                let value = if !fd.array_dims.is_empty() {
                    let et = fd.type_name.clone();
                    let arr = GbArray::new(et.clone(), fd.array_dims.clone(), || self.element_default(&et));
                    Value::Array(Rc::new(RefCell::new(arr)))
                } else if self.prog.classes.get(&fd.type_name).map(|c| c.is_struct).unwrap_or(false) {
                    self.allocate_instance(&fd.type_name)
                } else {
                    self.element_default(&fd.type_name)
                };
                let ty = if fd.array_dims.is_empty() {
                    fd.type_name.clone()
                } else {
                    format!("array:{}", fd.type_name)
                };
                fields.insert(fd.name.clone(), FieldVal { ty, value });
            }
        }
        Value::Instance(Rc::new(RefCell::new(Instance {
            class_name: Rc::from(class_name),
            fields,
        })))
    }

    /// User-Operator-Overloading. Liefert Some(Ergebnis) wenn LHS oder RHS
    /// eine Instanz mit der Operator-Methode ist.
    fn user_op(&mut self, method: &str, a: &Value, b: &Value) -> R<Option<Value>> {
        if let Value::Instance(rc) = a {
            let cn = rc.borrow().class_name.clone();
            if let Some(m) = self.resolve_method(&cn, method) {
                return Ok(Some(self.exec(m, vec![b.clone()], Some(a.clone()))?));
            }
        }
        if let Value::Instance(rc) = b {
            let cn = rc.borrow().class_name.clone();
            if let Some(m) = self.resolve_method(&cn, method) {
                return Ok(Some(self.exec(m, vec![a.clone()], Some(b.clone()))?));
            }
        }
        Ok(None)
    }

    /// Debugging fuer den naechsten `run()` aktivieren (`gbrt debug`).
    pub fn enable_debug(&mut self) {
        self.dbg = Some(DebugState::new());
    }

    /// Restlichen (noch nicht gesendeten) Programm-Output als output-Event
    /// schicken (live-Ausgabe waehrend des Debuggens). Auch am Ende aufrufen.
    pub fn debug_flush_output(&mut self) {
        if let Some(mut dbg) = self.dbg.take() {
            if self.out.len() > dbg.out_sent {
                let chunk = self.out[dbg.out_sent..].to_string();
                dbg.out_sent = self.out.len();
                dbg_emit(&serde_json::json!({"event":"output","text":chunk}));
            }
            self.dbg = Some(dbg);
        }
    }

    /// Per-Zeile-Hook fuer den Debugger. take/restore von self.dbg vermeidet
    /// Borrow-Konflikte mit self.globals/out beim Snapshot/eval.
    fn debug_on_line(&mut self, fn_: &Func, locals: &[Value]) -> R<()> {
        let mut dbg = match self.dbg.take() { Some(d) => d, None => return Ok(()) };
        let r = self.debug_cycle(&mut dbg, fn_, locals);
        self.dbg = Some(dbg);
        r
    }

    fn debug_cycle(&mut self, dbg: &mut DebugState, fn_: &Func, locals: &[Value]) -> R<()> {
        let line = self.cur_line;
        let depth = self.depth;
        // Neue Ausgabe live nachschieben.
        if self.out.len() > dbg.out_sent {
            let chunk = self.out[dbg.out_sent..].to_string();
            dbg.out_sent = self.out.len();
            dbg_emit(&serde_json::json!({"event":"output","text":chunk}));
        }
        // Pause noetig? (Step-Regel ODER Breakpoint mit erfuellter Bedingung)
        let mut pause = match dbg.step {
            StepMode::Into => true,
            StepMode::Over => depth <= dbg.step_depth,
            StepMode::Out  => depth <  dbg.step_depth,
            StepMode::Run  => false,
        };
        if !pause {
            if let Some(cond) = dbg.breakpoints.get(&line) {
                pause = match cond {
                    None => true,
                    // Bedingung: fail-open (bei Eval-Fehler trotzdem anhalten).
                    Some(expr) => self.eval_node(expr, fn_, locals)
                        .map(|v| dbg_truthy(&v)).unwrap_or(true),
                };
            }
        }
        if !pause { return Ok(()); }
        dbg_emit(&serde_json::json!({
            "event": "paused", "line": line, "depth": depth,
            "locals": self.dbg_locals_json(fn_, locals),
            "globals": self.dbg_globals_json(),
        }));
        // Kommandos lesen bis continue/step/stop/EOF.
        loop {
            let cmd = match dbg_read_cmd() {
                Some(c) => c,
                None => return Err("__DEBUG_STOP__".into()),
            };
            match cmd.get("cmd").and_then(|v| v.as_str()).unwrap_or("") {
                "continue"  => { dbg.step = StepMode::Run; return Ok(()); }
                "step-over" => { dbg.step = StepMode::Over; dbg.step_depth = depth; return Ok(()); }
                "step-into" => { dbg.step = StepMode::Into; return Ok(()); }
                "step-out"  => { dbg.step = StepMode::Out; dbg.step_depth = depth; return Ok(()); }
                "stop"      => return Err("__DEBUG_STOP__".into()),
                "set-breakpoints" => self.dbg_set_breakpoints(dbg, &cmd),
                "eval" => {
                    let src = cmd.get("expr").and_then(|v| v.as_str()).unwrap_or("");
                    match crate::parser::parse_expression(src)
                        .and_then(|n| self.eval_node(&n, fn_, locals)) {
                        Ok(v) => dbg_emit(&serde_json::json!({
                            "event":"eval-result","value":v.fmt(),"type":v.type_name()})),
                        Err(e) => dbg_emit(&serde_json::json!({"event":"eval-error","message":e})),
                    }
                }
                _ => {}
            }
        }
    }

    fn dbg_set_breakpoints(&self, dbg: &mut DebugState, cmd: &serde_json::Value) {
        dbg.breakpoints.clear();
        if let Some(lines) = cmd.get("lines").and_then(|v| v.as_array()) {
            for l in lines {
                if let Some(ln) = l.as_u64() { dbg.breakpoints.insert(ln as u32, None); }
            }
        }
        if let Some(conds) = cmd.get("conditions").and_then(|v| v.as_object()) {
            for (k, expr) in conds {
                if let (Ok(ln), Some(src)) = (k.parse::<u32>(), expr.as_str()) {
                    // Bedingung vorab parsen; bei Parse-Fehler unbedingter BP.
                    let node = crate::parser::parse_expression(src).ok();
                    dbg.breakpoints.insert(ln, node);
                }
            }
        }
    }

    fn dbg_locals_json(&self, fn_: &Func, locals: &[Value]) -> serde_json::Value {
        let mut out = Vec::new();
        for (i, nm) in fn_.local_names.iter().enumerate() {
            // Compiler-Zwischenwerte ueberspringen (namenlos oder __-Praefix).
            if nm.is_empty() || nm.starts_with("__") { continue; }
            if let Some(v) = locals.get(i) {
                out.push(serde_json::json!({"name": nm, "type": v.type_name(), "value": dbg_short(v)}));
            }
        }
        serde_json::Value::Array(out)
    }

    fn dbg_globals_json(&self) -> serde_json::Value {
        let mut out = Vec::new();
        for (name, slot) in &self.globals {
            if name.starts_with("__") { continue; }
            let s = slot.borrow();
            if s.is_const { continue; }      // Baseline-Konstanten (Farben/Keys/PI) ausblenden
            out.push(serde_json::json!({"name": name, "type": s.value.type_name(), "value": dbg_short(&s.value)}));
        }
        out.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        serde_json::Value::Array(out)
    }

    /// Mini-Evaluator fuer Debugger-Bedingungen + `eval`: Identifier (Locals
    /// per Name / Globals), Literale, unaere/binaere Operatoren. Bewusst ein
    /// Subset (kein Member/Index/Call) -- deckt typische Breakpoint-Bedingungen.
    fn eval_node(&self, n: &crate::ast::Node, fn_: &Func, locals: &[Value]) -> R<Value> {
        use crate::ast::{Node, NumV};
        match n {
            Node::NumberLit(NumV::Int(i)) => Ok(Value::Int(*i)),
            Node::NumberLit(NumV::Float(f)) => Ok(Value::Float(*f)),
            Node::StringLit(s) => Ok(Value::str_rc(s)),
            Node::BoolLit(b) => Ok(Value::Bool(*b)),
            Node::Identifier(name) => {
                let key = name.to_lowercase();
                if let Some(i) = fn_.local_names.iter().position(|n| *n == key) {
                    if let Some(v) = locals.get(i) { return Ok(v.clone()); }
                }
                if let Some(slot) = self.globals.get(&key) {
                    return Ok(slot.borrow().value.clone());
                }
                Err(format!("eval: '{}' nicht gefunden", name))
            }
            Node::UnaryOp { op, operand } => {
                let v = self.eval_node(operand, fn_, locals)?;
                match op.as_str() {
                    "-" => match v { Value::Int(i) => Ok(Value::Int(-i)),
                                     _ => Ok(Value::Float(-as_f64(&v))) },
                    "not" => Ok(Value::Bool(!dbg_truthy(&v))),
                    _ => Err(format!("eval: unaerer Operator '{}' nicht unterstuetzt", op)),
                }
            }
            Node::BinaryOp { op, left, right } => {
                let a = self.eval_node(left, fn_, locals)?;
                let b = self.eval_node(right, fn_, locals)?;
                dbg_binop(op, &a, &b)
            }
            _ => Err("eval: Ausdruck nicht unterstuetzt (nur Vars/Literale/Operatoren)".into()),
        }
    }

    // ---------------------------------------------------------------- exec
    fn exec(&mut self, fn_: &'p Func, args: Vec<Value>, self_obj: Option<Value>) -> R<Value> {
        // Call-Tiefe fuer den Debugger (Step over/into/out). Inkrement pro Frame;
        // garantiert dekrementiert (auch bei Fehler/Return) via Wrapper.
        self.depth += 1;
        let r = self.exec_inner(fn_, args, self_obj);
        self.depth -= 1;
        r
    }

    fn exec_inner(&mut self, fn_: &'p Func, args: Vec<Value>, self_obj: Option<Value>) -> R<Value> {
        let lbuf = self.pool_locals.pop().unwrap_or_default();
        let mut locals = bind_params(fn_, args, lbuf)?;
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
    fn exec_byref(&mut self, fn_: &'p Func, args: Vec<Value>, self_obj: Option<Value>)
        -> R<(Value, Vec<Value>)> {
        self.depth += 1;
        let r = self.exec_byref_inner(fn_, args, self_obj);
        self.depth -= 1;
        r
    }

    fn exec_byref_inner(&mut self, fn_: &'p Func, args: Vec<Value>, self_obj: Option<Value>)
        -> R<(Value, Vec<Value>)> {
        let lbuf = self.pool_locals.pop().unwrap_or_default();
        let mut locals = bind_params(fn_, args, lbuf)?;
        let mut stack: Vec<Value> = self.pool_stacks.pop().unwrap_or_else(|| Vec::with_capacity(16));
        let mut ip: usize = 0;
        let mut try_handlers: Vec<(usize, usize)> = Vec::new();
        let ret = match self.run_frame(fn_, &mut locals, &mut stack, &mut ip, &mut try_handlers, self_obj.as_ref())? {
            Step::Return(v) => v,
            Step::Yield(_) => return Err("YIELD ausserhalb einer Coroutine".into()),
        };
        // Finale Werte der BYREF-Param-Slots (in Param-Reihenfolge) auslesen.
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
        Ok((ret, byref_vals))
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
        loop {
            match self.dispatch(fn_, locals, stack, ip, try_handlers, self_obj) {
                Ok(step) => return Ok(step),
                // Debugger-Abbruch (`stop`) und Profiler-Stop-Signal duerfen
                // NICHT von TRY/CATCH gefangen werden -- unbedingt durchreichen.
                Err(e) if e == "__DEBUG_STOP__" || e == PROFILE_STOP => return Err(e),
                Err(e) => {
                    // Quell-Zeile lazy ermitteln: ip zeigt HINTER die
                    // fehlgeschlagene Instruktion. Nur der innerste Frame
                    // (Fehler-Ursprung) setzt sie; aeussere Frames sehen das
                    // Flag und lassen die innerste Zeile stehen.
                    if !self.err_line_set {
                        if let Some(&ln) = fn_.lines.get(ip.saturating_sub(1)) {
                            if ln != 0 { self.cur_line = ln; }
                        }
                        self.err_line_set = true;
                    }
                    match try_handlers.pop() {
                        Some((target, depth)) => {
                            self.err_line_set = false;   // Fehler konsumiert (CATCH)
                            stack.truncate(depth);
                            stack.push(Value::str_rc(&e));
                            *ip = target;
                        }
                        None => return Err(e),
                    }
                }
            }
        }
    }

    /// Setzt eine Coroutine fort (`send` wird der YIELD-Ausdruck im Body).
    /// Liefert den naechsten YIELD-Wert bzw. den RETURN-Wert beim Ende.
    fn coro_resume(&mut self, co: &Rc<RefCell<CoroState>>, send: Value) -> R<Value> {
        if co.borrow().done {
            return Err("CORO_RESUME/SEND auf bereits beendeter Coroutine".into());
        }
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
            locals = bind_params(fn_, args, Vec::new())?;
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
                Ok(v)
            }
            Ok(Step::Return(v)) => {
                let mut c = co.borrow_mut();
                c.done = true;
                c.result = v.clone();
                Ok(v)
            }
            Err(e) => {
                co.borrow_mut().done = true;
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
            if let Value::FuncRef(_) = &a[1] {
                return Ok(Some(self.sort_with_comparator(&a[0], &a[1])?));
            }
        }
        Ok(None)
    }

    /// SORT(arr, comparator-FUNCREF): stabil sortieren, wobei `comparator(x, y)`
    /// eine User-Funktion ist, die <0/0/>0 liefert (wie ein C-qsort-Comparator).
    fn sort_with_comparator(&mut self, arr_v: &Value, cmp: &Value) -> R<Value> {
        use std::cmp::Ordering;
        let arr = match arr_v {
            Value::Array(x) => x.clone(),
            _ => return Err("SORT erwartet ARRAY".to_string()),
        };
        let cmp_name = match cmp {
            Value::FuncRef(n) => n.clone(),
            _ => return Err("SORT: Comparator muss FUNCREF sein".to_string()),
        };
        if arr.borrow().dims.len() != 1 {
            return Err("SORT: nur 1D-Arrays".to_string());
        }
        let func: &'p Func = self.prog.func(cmp_name.as_ref())
            .ok_or_else(|| format!("SORT: Funktion '{}' existiert nicht", cmp_name))?;
        // Werte herausziehen -> kein Array-Borrow waehrend der Comparator laeuft.
        let mut vals: Vec<Value> = arr.borrow().cells.to_values();
        let mut error: Option<String> = None;
        vals.sort_by(|x, y| {
            if error.is_some() { return Ordering::Equal; }
            match self.exec(func, vec![x.clone(), y.clone()], None) {
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
        // Builtin-Namen liegen im .gbc lowercase vor.
        if !name.starts_with("coro_") && name != "__comp_iter" { return Ok(None); }
        match name {
            "coro_resume" => {
                let co = expect_coro(&a[0], "CORO_RESUME")?;
                Ok(Some(self.coro_resume(&co, Value::Nil)?))
            }
            "coro_send" => {
                let co = expect_coro(&a[0], "CORO_SEND")?;
                Ok(Some(self.coro_resume(&co, a[1].clone())?))
            }
            "coro_done" => {
                let co = expect_coro(&a[0], "CORO_DONE")?;
                let d = co.borrow().done;
                Ok(Some(Value::Bool(d)))
            }
            "coro_result" => {
                let co = expect_coro(&a[0], "CORO_RESULT")?;
                let c = co.borrow();
                if !c.done {
                    return Err("CORO_RESULT: Coroutine ist noch nicht beendet".into());
                }
                Ok(Some(c.result.clone()))
            }
            "coro_close" => {
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

    fn dispatch(
        &mut self,
        fn_: &'p Func,
        locals: &mut Vec<Value>,
        stack: &mut Vec<Value>,
        ip: &mut usize,
        try_handlers: &mut Vec<(usize, usize)>,
        self_obj: Option<&Value>,
    ) -> R<Step> {
        let code = &fn_.code;
        let constants = &fn_.constants;
        let n = code.len();
        // Zeilen-Tracking nur, wenn jemand zusieht (Profiler/Stop/Debugger).
        // Fuer Laufzeitfehler wird die Quell-Zeile LAZY im Fehlerfall
        // ermittelt (run_frame) -- der Normalfall zahlt pro Instruktion nichts.
        let track_lines = self.prof.is_some() || self.stop.is_some() || self.dbg.is_some();

        while *ip < n {
            let instr = &code[*ip];
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
                op::POP => { stack.pop(); }
                op::DUP => { let v = vm_top(stack)?.clone(); stack.push(v); }

                // --- Lokale ---
                op::LOAD_LOCAL => stack.push(locals[arg.as_usize()].clone()),
                op::STORE_LOCAL => {
                    let slot = arg.as_usize();
                    let v = vm_pop(stack)?;
                    // Fast-Arm: Wert hat schon den Zieltyp -> kein coerce-Call.
                    let ty = &fn_.local_types[slot];
                    locals[slot] = match (&v, ty.as_str()) {
                        (Value::Int(_), "integer") | (Value::Float(_), "float")
                        | (Value::Str(_), "string") | (Value::Bool(_), "boolean")
                        | (_, "any") | (_, "") => v,
                        (Value::Int(n), "float") => Value::Float(*n as f64),
                        _ => coerce(v, ty, "Lokale Variable")?,
                    };
                }
                op::DECLARE_LOCAL => {
                    let l = arg.list();
                    let slot = l[0].as_usize();
                    let ty = l[1].str();
                    if let Some(vt) = ty.strip_prefix("map:") {
                        if !matches!(locals[slot], Value::Map(_)) {
                            locals[slot] = Value::Map(Rc::new(RefCell::new(GbMap::new(vt.to_string()))));
                        }
                    } else if matches!(locals[slot], Value::Nil) {
                        locals[slot] = arg_value(&l[2]);
                    }
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
                                let s = self.global_slots[var_idx].as_ref().ok_or("Global-Slot leer")?;
                                s.borrow().value.clone()
                            } else { locals[var_idx].clone() };
                            let stepv = if step_is_slot { locals[step_idx].clone() } else { constants[step_idx].clone() };
                            require_number(&cur, &stepv, "+")?;
                            let next = nn_add(cur, stepv)?;
                            if var_global {
                                let slot = self.global_slots[var_idx].as_ref().ok_or("Slot leer")?.clone();
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
                    if !exit { *ip = target; }
                }

                // --- Slot-Globals ---
                op::LOAD_GLOBAL_SLOT => {
                    let s = self.global_slots[arg.as_usize()].as_ref().ok_or("Global-Slot leer")?;
                    stack.push(s.borrow().value.clone());
                }
                op::STORE_GLOBAL_SLOT => {
                    let idx = arg.as_usize();
                    let v = vm_pop(stack)?;
                    let slot = self.global_slots[idx].as_ref().ok_or("Slot leer")?.clone();
                    let mut sb = slot.borrow_mut();
                    if sb.is_const {
                        return Err("CONST kann nicht ueberschrieben werden".into());
                    }
                    // Fast-Arm: Typ passt schon -> kein ty-Clone, kein coerce.
                    sb.value = match (&v, sb.ty.as_str()) {
                        (Value::Int(_), "integer") | (Value::Float(_), "float")
                        | (Value::Str(_), "string") | (Value::Bool(_), "boolean")
                        | (_, "any") | (_, "") => v,
                        (Value::Int(n), "float") => Value::Float(*n as f64),
                        _ => { let ty = sb.ty.clone(); coerce(v, &ty, "Zuweisung an global")? }
                    };
                }
                op::DECLARE_GLOBAL_SLOT => {
                    let l = arg.list();
                    let slot_idx = l[0].as_usize();
                    let name = constants[l[1].as_usize()].fmt();
                    let ty = constants[l[2].as_usize()].fmt();
                    let default = if let Some(vt) = ty.strip_prefix("map:") {
                        Value::Map(Rc::new(RefCell::new(GbMap::new(vt.to_string()))))
                    } else {
                        constants[l[3].as_usize()].clone()
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
                    let (ty, value) = match &l[1] {
                        Arg::None => (infer_type(&value).to_string(), value),
                        ti => {
                            let t = constants[ti.as_usize()].fmt();
                            let v = coerce(value, &t, "CONST")?;
                            (t, v)
                        }
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
                        .ok_or_else(|| format!("Variable '{}' nicht deklariert (DIM fehlt?)", name))?;
                    stack.push(s.borrow().value.clone());
                }
                op::STORE_NAME => {
                    let name = constants[arg.as_usize()].fmt();
                    let slot = self.globals.get(&name)
                        .ok_or_else(|| format!("Variable '{}' nicht deklariert (DIM fehlt?)", name))?.clone();
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
                        Value::Map(Rc::new(RefCell::new(GbMap::new(vt.to_string()))))
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
                    let (ty, value) = match &l[1] {
                        Arg::None => (infer_type(&value).to_string(), value),
                        ti => {
                            let t = constants[ti.as_usize()].fmt();
                            let v = coerce(value, &t, "CONST")?;
                            (t, v)
                        }
                    };
                    self.globals.entry(name).or_insert_with(|| {
                        Rc::new(RefCell::new(Slot { ty, value, is_const: true }))
                    });
                }

                // --- Arithmetik (generisch, mit User-Operator-Overloading) ---
                op::ADD => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    // Numerischer Fast-Path zuerst: Int/Float-Paare sind der
                    // Normalfall in heissen Schleifen -- Modul-/User-Operator-
                    // Checks kosten dort nur (Semantik unveraendert: weder
                    // Zahlen noch der Sonderfall-Pfad ueberlappen sich).
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(
                            x.checked_add(*y).map(Value::Int).ok_or_else(|| int_overflow_msg("+"))?),
                        (Value::Float(x), Value::Float(y)) => stack.push(Value::Float(x + y)),
                        (Value::Int(x), Value::Float(y)) => stack.push(Value::Float(*x as f64 + y)),
                        (Value::Float(x), Value::Int(y)) => stack.push(Value::Float(x + *y as f64)),
                        _ => {
                            if let Some(r) = module_op('+', &a, &b) { stack.push(r?); }
                            else if let Some(r) = self.user_op("__op_add__", &a, &b)? { stack.push(r); }
                            else if matches!(a, Value::Str(_)) || matches!(b, Value::Str(_)) {
                                stack.push(Value::str_rc(&format!("{}{}", a.fmt(), b.fmt())));
                            } else { require_number(&a, &b, "+")?; stack.push(nn_add(a, b)?); }
                        }
                    }
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
                            else if let Some(r) = self.user_op("__op_sub__", &a, &b)? { stack.push(r); }
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
                            else if let Some(r) = self.user_op("__op_mul__", &a, &b)? { stack.push(r); }
                            else { stack.push(mul(a, b)?); }
                        }
                    }
                }
                op::DIV => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    if matches!(&a, Value::Int(_) | Value::Float(_)) && matches!(&b, Value::Int(_) | Value::Float(_)) {
                        stack.push(div(a, b)?);
                    } else if let Some(r) = module_op('/', &a, &b) { stack.push(r?); }
                    else if let Some(r) = self.user_op("__op_div__", &a, &b)? { stack.push(r); }
                    else { stack.push(div(a, b)?); }
                }
                op::MOD => {
                    let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    if matches!(&a, Value::Int(_) | Value::Float(_)) && matches!(&b, Value::Int(_) | Value::Float(_)) {
                        stack.push(modulo(a, b)?);
                    } else if let Some(r) = self.user_op("__op_mod__", &a, &b)? { stack.push(r); }
                    else { stack.push(modulo(a, b)?); }
                }
                op::POW => { let b = vm_pop(stack)?; let a = vm_pop(stack)?; stack.push(pow(a, b)?); }
                op::INT_DIV => { let b = vm_pop(stack)?; let a = vm_pop(stack)?; stack.push(int_div(a, b)?); }
                op::NEG => { let v = vm_pop(stack)?; stack.push(neg(v)?); }

                // --- Vergleich / Logik ---
                op::EQ => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x == y)),
                        _ => match self.user_op("__op_eq__", &a, &b)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(value_eq(&a, &b))) }
                    } }
                op::NEQ => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x != y)),
                        _ => match self.user_op("__op_ne__", &a, &b)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(!value_eq(&a, &b))) }
                    } }
                op::LT => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x < y)),
                        (Value::Float(_), Value::Float(_)) | (Value::Int(_), Value::Float(_)) | (Value::Float(_), Value::Int(_)) =>
                            stack.push(Value::Bool(cmp(&a, &b, '<')?)),
                        _ => match self.user_op("__op_lt__", &a, &b)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(cmp(&a, &b, '<')?)) }
                    } }
                op::GT => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x > y)),
                        (Value::Float(_), Value::Float(_)) | (Value::Int(_), Value::Float(_)) | (Value::Float(_), Value::Int(_)) =>
                            stack.push(Value::Bool(cmp(&a, &b, '>')?)),
                        _ => match self.user_op("__op_gt__", &a, &b)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(cmp(&a, &b, '>')?)) }
                    } }
                op::LEQ => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x <= y)),
                        (Value::Float(_), Value::Float(_)) | (Value::Int(_), Value::Float(_)) | (Value::Float(_), Value::Int(_)) =>
                            stack.push(Value::Bool(cmp(&a, &b, 'l')?)),
                        _ => match self.user_op("__op_le__", &a, &b)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(cmp(&a, &b, 'l')?)) }
                    } }
                op::GEQ => { let b = vm_pop(stack)?; let a = vm_pop(stack)?;
                    match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Bool(x >= y)),
                        (Value::Float(_), Value::Float(_)) | (Value::Int(_), Value::Float(_)) | (Value::Float(_), Value::Int(_)) =>
                            stack.push(Value::Bool(cmp(&a, &b, 'g')?)),
                        _ => match self.user_op("__op_ge__", &a, &b)? { Some(r) => stack.push(r), None => stack.push(Value::Bool(cmp(&a, &b, 'g')?)) }
                    } }
                op::NOT => { let v = vm_pop(stack)?; stack.push(Value::Bool(!v.truthy())); }

                // --- Bitwise ---
                op::BAND => { let (x, y) = int_pair2(stack)?; stack.push(Value::Int(x & y)); }
                op::BOR => { let (x, y) = int_pair2(stack)?; stack.push(Value::Int(x | y)); }
                op::BXOR => { let (x, y) = int_pair2(stack)?; stack.push(Value::Int(x ^ y)); }
                op::SHL => { let (x, y) = int_pair2(stack)?; if y < 0 { return Err("SHL: negativ".into()); } stack.push(Value::Int(x << y)); }
                op::SHR => { let (x, y) = int_pair2(stack)?; if y < 0 { return Err("SHR: negativ".into()); } stack.push(Value::Int(x >> y)); }
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

                // --- Kontrollfluss ---
                op::JUMP => *ip = arg.as_usize(),
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
                    let call_args = stack.split_off(split);
                    if callee.is_coroutine {
                        stack.push(make_coro(callee, call_args, None));
                    } else if callee.param_byref.iter().any(|&b| b) {
                        // BYREF: finale Param-Werte mit zurueckgeben. Layout fuers
                        // Write-Back: [.., bv0, bv1, .., bv{m-1}, result].
                        let (ret, byref_vals) = self.exec_byref(callee, call_args, None)?;
                        for v in byref_vals { stack.push(v); }
                        if !callee.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                    } else {
                        let ret = self.exec(callee, call_args, None)?;
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
                    let v = {
                        let bargs: &[Value] = &stack[split..];
                        if let Some(v) = self.try_array_hof(name, bargs)? { v }
                        else if let Some(v) = self.try_scene(name, bargs)? { v }
                        else if let Some(v) = self.try_coro(name, bargs)? { v }
                        else if let Some(v) = self.try_timer(name, bargs)? { v }
                        else if let Some(v) = self.try_db(name, bargs)? { v }
                        else if let Some(v) = self.try_net(name, bargs)? { v }
                        else if let Some(v) = self.try_html(name, bargs)? { v }
                        else if let Some(v) = self.try_serial(name, bargs)? { v }
                        else if let Some(v) = self.try_usb(name, bargs)? { v }
                        else if let Some(v) = self.try_wifi(name, bargs)? { v }
                        else if let Some(v) = self.try_bt(name, bargs)? { v }
                        else if let Some(v) = self.try_gui(name, bargs)? { v }
                        else if let Some(v) = self.try_graphics(name, bargs)? { v }
                        else {
                            match safe_call_builtin(name, bargs) {
                                Some(Ok(v)) => v,
                                Some(Err(e)) => {
                                    if e.starts_with("__UNKNOWN_BUILTIN__:") {
                                        return Err(unknown_builtin_msg(name));
                                    }
                                    return Err(e);
                                }
                                None => return Err(unknown_builtin_msg(name)),
                            }
                        }
                    };
                    stack.truncate(split);
                    stack.push(v);
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
                                let ret = self.exec(tgt, call_args, None)?;
                                if !tgt.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                            }
                        }
                        other => return Err(format!(
                            "'{}' ist eine Variable vom Typ {} und kann nicht wie eine Funktion \
                             aufgerufen werden. Falls du den eingebauten Befehl '{}' meinst: \
                             benenne die Variable um.",
                            cname, other.type_name(), cname.to_uppercase())),
                    }
                }
                op::CALL_METHOD => {
                    let (method, argc, _) = call_parts(arg);
                    let split = stack.len() - argc;
                    let margs = stack.split_off(split);
                    let obj = vm_pop(stack)?;
                    match &obj {
                        Value::Instance(rc) => {
                            let cn = rc.borrow().class_name.clone();
                            let m = self.resolve_method(&cn, method)
                                .ok_or_else(|| format!("Methode '{}' existiert nicht in {}", method, cn))?;
                            if m.is_coroutine {
                                stack.push(make_coro(m, margs, Some(obj.clone())));
                            } else {
                                let ret = self.exec(m, margs, Some(obj.clone()))?;
                                if !m.is_sub { stack.push(ret); } else { stack.push(Value::Nil); }
                            }
                        }
                        Value::Nil => return Err(format!("Methodenaufruf '.{}' bei NIL-Referenz", method)),
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

                // --- OOP ---
                op::NEW_INSTANCE => {
                    let l = arg.list();
                    let class_name = l[0].str();
                    let argc = l[1].as_usize();
                    let has_init_args = arg_truthy(&l[2]);
                    if !self.prog.classes.contains_key(class_name) {
                        return Err(format!("Klasse '{}' nicht gefunden", class_name));
                    }
                    let inst = self.allocate_instance(class_name);
                    if has_init_args {
                        let split = stack.len() - argc;
                        let init_args = stack.split_off(split);
                        if let Some(init) = self.resolve_method(class_name, "init") {
                            self.exec(init, init_args, Some(inst.clone()))?;
                        } else if !init_args.is_empty() {
                            return Err(format!("Klasse {} hat keine SUB Init - Argumente bei NEW nicht moeglich", class_name));
                        }
                    }
                    stack.push(inst);
                }
                op::LOAD_SELF => {
                    let s = self_obj.ok_or("LOAD_SELF (Self) ausserhalb Methodenkontext")?;
                    stack.push(s.clone());
                }
                op::LOAD_FIELD => {
                    let name = constants[arg.as_usize()].fmt();
                    let s = self_obj.ok_or_else(|| format!("LOAD_FIELD '{}' ausserhalb Methodenkontext", name))?;
                    if let Value::Instance(rc) = s {
                        let v = rc.borrow().fields.get(&name)
                            .ok_or_else(|| format!("Feld '{}' existiert nicht", name))?.value.clone();
                        stack.push(v);
                    } else { return Err("LOAD_FIELD: self ist keine Instanz".into()); }
                }
                op::STORE_FIELD => {
                    let name = constants[arg.as_usize()].fmt();
                    let s = self_obj.ok_or_else(|| format!("STORE_FIELD '{}' ausserhalb Methodenkontext", name))?;
                    let v = vm_pop(stack)?;
                    if let Value::Instance(rc) = s {
                        let ty = rc.borrow().fields.get(&name)
                            .ok_or_else(|| format!("Feld '{}' existiert nicht", name))?.ty.clone();
                        let cv = coerce(v, &ty, &format!("Zuweisung an Feld {}", name))?;
                        rc.borrow_mut().fields.get_mut(&name).unwrap().value = cv;
                    } else { return Err("STORE_FIELD: self ist keine Instanz".into()); }
                }
                op::LOAD_MEMBER => {
                    // Member-Name direkt aus dem Const-Pool (&str) -- fmt()
                    // allozierte vorher einen String PRO Zugriff.
                    let name_owned;
                    let name: &str = match &constants[arg.as_usize()] {
                        Value::Str(s) => s,
                        v => { name_owned = v.fmt(); &name_owned }
                    };
                    let obj = vm_pop(stack)?;
                    match &obj {
                        Value::Nil => return Err(format!("Zugriff auf '.{}' bei NIL-Referenz", name)),
                        Value::Namespace(ns) => {
                            match ns.members.get(&name.to_lowercase()) {
                                Some(v) => stack.push(v.clone()),
                                None => return Err(format!("{} hat keinen Member '{}'", ns.name, name)),
                            }
                        }
                        Value::Instance(rc) => {
                            let cn = rc.borrow().class_name.clone();
                            if self.is_property(&cn, &name) {
                                let getter = self.resolve_method(&cn, &format!("__get_{}", name.to_lowercase()))
                                    .ok_or_else(|| format!("Property '{}' in {} hat keinen Getter", name, cn))?;
                                let r = self.exec(getter, vec![], Some(obj.clone()))?;
                                stack.push(r);
                            } else {
                                let v = rc.borrow().fields.get(name)
                                    .ok_or_else(|| format!("Feld '{}' existiert nicht in {}", name, cn))?.value.clone();
                                stack.push(v);
                            }
                        }
                        _ => return Err(format!("Zugriff auf '.{}' bei nicht-Objekt ({})", name, obj.type_name())),
                    }
                }
                op::STORE_MEMBER => {
                    let name_owned;
                    let name: &str = match &constants[arg.as_usize()] {
                        Value::Str(s) => s,
                        v => { name_owned = v.fmt(); &name_owned }
                    };
                    let v = vm_pop(stack)?;
                    let obj = vm_pop(stack)?;
                    match &obj {
                        Value::Nil => return Err(format!("Zuweisung an '.{}' bei NIL-Referenz", name)),
                        Value::Instance(rc) => {
                            let cn = rc.borrow().class_name.clone();
                            if self.is_property(&cn, name) {
                                let setter = self.resolve_method(&cn, &format!("__set_{}", name.to_lowercase()))
                                    .ok_or_else(|| format!("Property '{}' in {} hat keinen Setter (read-only)", name, cn))?;
                                self.exec(setter, vec![v], Some(obj.clone()))?;
                            } else {
                                // Ein borrow_mut, Coerce-Fast-Arm; der
                                // format!-Fehlerkontext entsteht nur noch im
                                // Slow-Path (vorher bei JEDEM Store).
                                let mut rcb = rc.borrow_mut();
                                let f = rcb.fields.get_mut(name)
                                    .ok_or_else(|| format!("Feld '{}' existiert nicht in {}", name, cn))?;
                                let cv = match (&v, f.ty.as_str()) {
                                    (Value::Int(_), "integer") | (Value::Float(_), "float")
                                    | (Value::Str(_), "string") | (Value::Bool(_), "boolean")
                                    | (_, "any") | (_, "") => v,
                                    (Value::Int(n), "float") => Value::Float(*n as f64),
                                    _ => {
                                        let ty = f.ty.clone();
                                        coerce(v, &ty, &format!("Zuweisung an {}.{}", cn, name))?
                                    }
                                };
                                f.value = cv;
                            }
                        }
                        _ => return Err(format!("Zuweisung an '.{}' bei nicht-Objekt ({})", name, obj.type_name())),
                    }
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
                            if ab.dims.len() == 1 && *i >= 0 && (*i as usize) < ab.cells.len() {
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
                            if ab.dims.len() == 1 && *i >= 0 && (*i as usize) < ab.cells.len() {
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
                op::DECLARE_ARRAY_NAME => {
                    let l = arg.list();
                    let name = constants[l[0].as_usize()].fmt();
                    let elem_type = l[1].str().to_string();
                    let num_dims = l[2].as_usize();
                    let dims = self.pop_dims(stack, num_dims)?;
                    let et = elem_type.clone();
                    let arr = GbArray::new(elem_type.clone(), dims, || self.element_default(&et));
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
                    let dims = self.pop_dims(stack, num_dims)?;
                    let et = elem_type.clone();
                    let arr = GbArray::new(elem_type, dims, || self.element_default(&et));
                    locals[slot] = Value::Array(Rc::new(RefCell::new(arr)));
                }

                // --- Exceptions ---
                op::TRY_BEGIN => try_handlers.push((arg.as_usize(), stack.len())),
                op::TRY_END => { try_handlers.pop(); }
                op::THROW => {
                    let v = vm_pop(stack)?;
                    let msg = match v { Value::Str(s) => s.to_string(), other => other.fmt() };
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

                // --- Rueckgabe ---
                op::RETURN => {
                    let v = vm_pop(stack)?;
                    return Ok(Step::Return(coerce(v, &fn_.return_type, "RETURN")?));
                }
                op::RETURN_VOID => return Ok(Step::Return(Value::Nil)),
                op::YIELD_VALUE => {
                    // Coroutine: Wert abgeben und suspendieren. Beim Resume legt
                    // coro_resume den Sende-Wert auf den Stack (Wert von `x = YIELD`).
                    let mut yval = vm_pop(stack)?;
                    if !fn_.return_type.is_empty() {
                        yval = coerce(yval, &fn_.return_type, "YIELD")?;
                    }
                    return Ok(Step::Yield(yval));
                }

                // --- I/O ---
                // Arg: [count, newline, sep0, sep1, ...] -- sep_i = Trenner ZWISCHEN
                // item_i und item_{i+1}: "," -> Leerzeichen, ";" -> kein Trennzeichen.
                // Trailing-Trenner setzt newline=false (kein Zeilenumbruch).
                op::PRINT => {
                    let l = arg.list();
                    let count = l[0].as_usize();
                    let newline = arg_truthy(&l[1]);
                    if count > 0 {
                        let split = stack.len() - count;
                        let items = stack.split_off(split);
                        for (i, it) in items.iter().enumerate() {
                            if i > 0 && l[i + 1].str() != ";" { self.out.push(' '); }
                            self.out.push_str(&it.fmt());
                        }
                    }
                    if newline { self.out.push('\n'); }
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
                    let raw = self.read_input_line();
                    let slot = self.globals.get(&name)
                        .ok_or_else(|| format!("Variable '{}' nicht deklariert (DIM fehlt?)", name))?
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
                    let raw = self.read_input_line();
                    let ty = fn_.local_types[slot_idx].clone();
                    locals[slot_idx] = coerce_input(&raw, &ty)?;
                }

                op::HALT => return Ok(Step::Return(Value::Nil)),

                other => return Err(format!("Opcode {} im Rust-VM noch nicht implementiert (ip={})", other, *ip - 1)),
            }
        }
        Ok(Step::Return(Value::Nil))
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
        Ok(dims)
    }

    /// Quell-Zeile des zuletzt ausgefuehrten Befehls (fuer Fehlermeldungen).
    /// 0 = unbekannt (z.B. wenn der Compiler keine Zeilen getrackt hat).
    pub fn error_line(&self) -> u32 {
        self.cur_line
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

    /// Modul `scene` (globaler Stack-State, kein Grafik-Bezug).
    // ===================================================================
    // Modul db (SQLite, Feature `db`)
    // ===================================================================
    fn try_db(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("db_")) { return Ok(None); }
        #[cfg(feature = "db")]
        { return self.try_db_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    #[cfg(feature = "db")]
    fn db_conn(&self, idx: i64) -> R<&rusqlite::Connection> {
        self.db_conns.get(idx as usize).and_then(|o| o.as_ref())
            .ok_or_else(|| format!("DB: ungueltiges/geschlossenes DB_CONN-Handle {}", idx))
    }
    #[cfg(feature = "db")]
    fn db_res(&self, idx: i64) -> R<&crate::db::DbResult> {
        let r = self.db_results.get(idx as usize)
            .ok_or_else(|| format!("DB: ungueltiges DB_RESULT-Handle {}", idx))?;
        if r.closed { return Err("DB: Result wurde bereits geschlossen".into()); }
        Ok(r)
    }
    #[cfg(feature = "db")]
    fn db_res_mut(&mut self, idx: i64) -> R<&mut crate::db::DbResult> {
        let r = self.db_results.get_mut(idx as usize)
            .ok_or_else(|| format!("DB: ungueltiges DB_RESULT-Handle {}", idx))?;
        if r.closed { return Err("DB: Result wurde bereits geschlossen".into()); }
        Ok(r)
    }

    #[cfg(feature = "db")]
    fn try_db_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::db;
        let v = match name {
            "db_open" => {
                let conn = db::open(bi_str(a, 0, "DB_OPEN")?)?;
                self.db_conns.push(Some(conn));
                Value::Int((self.db_conns.len() - 1) as i64)
            }
            "db_close" => {
                let i = bi_int(a, 0, "DB_CLOSE")? as usize;
                if let Some(s) = self.db_conns.get_mut(i) { *s = None; }
                Value::Nil
            }
            "db_last_rowid" => Value::Int(self.db_conn(bi_int(a, 0, "DB_LAST_ROWID")?)?.last_insert_rowid()),
            "db_exec" => {
                let params = db_params(a.get(2..).unwrap_or(&[]), "DB_EXEC")?;
                let sql = bi_str(a, 1, "DB_EXEC")?.to_string();
                Value::Int(db::exec(self.db_conn(bi_int(a, 0, "DB_EXEC")?)?, &sql, &params)?)
            }
            "db_query" => {
                let params = db_params(a.get(2..).unwrap_or(&[]), "DB_QUERY")?;
                let sql = bi_str(a, 1, "DB_QUERY")?.to_string();
                let res = { let c = self.db_conn(bi_int(a, 0, "DB_QUERY")?)?; db::query(c, &sql, &params)? };
                self.db_results.push(res);
                Value::Int((self.db_results.len() - 1) as i64)
            }
            "db_next" => {
                let r = self.db_res_mut(bi_int(a, 0, "DB_NEXT")?)?;
                r.pos += 1;
                Value::Bool((r.pos as usize) < r.rows.len())
            }
            "db_col_count" => Value::Int(self.db_res(bi_int(a, 0, "DB_COL_COUNT")?)?.columns.len() as i64),
            "db_col_name" => { let i1 = bi_int(a, 1, "DB_COL_NAME")?; Value::str_rc(&self.db_res(bi_int(a, 0, "DB_COL_NAME")?)?.col_name(i1)?) }
            "db_close_result" => { self.db_res_mut(bi_int(a, 0, "DB_CLOSE_RESULT")?)?.closed = true; Value::Nil }
            "db_is_null" => { let i1 = bi_int(a, 1, "DB_IS_NULL")?; Value::Bool(self.db_res(bi_int(a, 0, "DB_IS_NULL")?)?.is_null(i1)?) }
            "db_get_string" => { let i1 = bi_int(a, 1, "DB_GET_STRING")?; Value::str_rc(&self.db_res(bi_int(a, 0, "DB_GET_STRING")?)?.get_string(i1)?) }
            "db_get_int" => { let i1 = bi_int(a, 1, "DB_GET_INT")?; Value::Int(self.db_res(bi_int(a, 0, "DB_GET_INT")?)?.get_int(i1)?) }
            "db_get_float" => { let i1 = bi_int(a, 1, "DB_GET_FLOAT")?; Value::Float(self.db_res(bi_int(a, 0, "DB_GET_FLOAT")?)?.get_float(i1)?) }
            "db_get_bool" => { let i1 = bi_int(a, 1, "DB_GET_BOOL")?; Value::Bool(self.db_res(bi_int(a, 0, "DB_GET_BOOL")?)?.get_bool(i1)?) }
            "db_begin" => { self.db_conn(bi_int(a, 0, "DB_BEGIN")?)?.execute_batch("BEGIN").map_err(|e| format!("DB_BEGIN: {}", e))?; Value::Nil }
            "db_commit" => { self.db_conn(bi_int(a, 0, "DB_COMMIT")?)?.execute_batch("COMMIT").map_err(|e| format!("DB_COMMIT: {}", e))?; Value::Nil }
            "db_rollback" => { self.db_conn(bi_int(a, 0, "DB_ROLLBACK")?)?.execute_batch("ROLLBACK").map_err(|e| format!("DB_ROLLBACK: {}", e))?; Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul net (std::net, Feature `net`)
    // ===================================================================
    fn try_net(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("net_")) { return Ok(None); }
        #[cfg(feature = "net")]
        { return self.try_net_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    #[cfg(feature = "net")]
    fn net_listener(&self, idx: i64) -> R<&(std::net::TcpListener, i64)> {
        self.tcp_listeners.get(idx as usize).and_then(|o| o.as_ref())
            .ok_or_else(|| format!("NET: ungueltiges/geschlossenes NET_LISTENER-Handle {}", idx))
    }
    #[cfg(feature = "net")]
    fn net_sock(&self, idx: i64) -> R<&crate::net::NetSock> {
        self.tcp_socks.get(idx as usize).and_then(|o| o.as_ref())
            .ok_or_else(|| format!("NET: ungueltiges/geschlossenes NET_SOCKET-Handle {}", idx))
    }
    #[cfg(feature = "net")]
    fn net_sock_mut(&mut self, idx: i64) -> R<&mut crate::net::NetSock> {
        self.tcp_socks.get_mut(idx as usize).and_then(|o| o.as_mut())
            .ok_or_else(|| format!("NET: ungueltiges/geschlossenes NET_SOCKET-Handle {}", idx))
    }
    #[cfg(feature = "net")]
    fn net_udp(&self, idx: i64) -> R<&crate::net::UdpSock> {
        self.udp_socks.get(idx as usize).and_then(|o| o.as_ref())
            .ok_or_else(|| format!("NET: ungueltiges/geschlossenes NET_UDP-Handle {}", idx))
    }
    #[cfg(feature = "net")]
    fn net_udp_mut(&mut self, idx: i64) -> R<&mut crate::net::UdpSock> {
        self.udp_socks.get_mut(idx as usize).and_then(|o| o.as_mut())
            .ok_or_else(|| format!("NET: ungueltiges/geschlossenes NET_UDP-Handle {}", idx))
    }

    #[cfg(feature = "net")]
    fn try_net_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::net;
        let v = match name {
            "net_tcp_listen" => {
                let (l, port) = net::listen(bi_int(a, 0, "NET_TCP_LISTEN")?)?;
                self.tcp_listeners.push(Some((l, port)));
                Value::Int((self.tcp_listeners.len() - 1) as i64)
            }
            "net_listener_port" => Value::Int(self.net_listener(bi_int(a, 0, "NET_LISTENER_PORT")?)?.1),
            "net_tcp_accept" => {
                let i = bi_int(a, 0, "NET_TCP_ACCEPT")?;
                let res = { let l = &self.net_listener(i)?.0; net::accept(l)? };
                match res {
                    Some(sock) => { self.tcp_socks.push(Some(sock)); Value::Int((self.tcp_socks.len() - 1) as i64) }
                    None => Value::Nil,
                }
            }
            "net_tcp_connect" => {
                let host = bi_str(a, 0, "NET_TCP_CONNECT")?.to_string();
                let s = net::connect(&host, bi_int(a, 1, "NET_TCP_CONNECT")?)?;
                self.tcp_socks.push(Some(s));
                Value::Int((self.tcp_socks.len() - 1) as i64)
            }
            "net_send" => { let i = bi_int(a, 0, "NET_SEND")?; let t = bi_str(a, 1, "NET_SEND")?.to_string(); Value::Int(net::send(self.net_sock_mut(i)?, &t)?) }
            "net_recv" => { let i = bi_int(a, 0, "NET_RECV")?; let n = bi_int(a, 1, "NET_RECV")?; Value::str_rc(&net::recv(self.net_sock_mut(i)?, n)?) }
            "net_peer_addr" => Value::str_rc(&self.net_sock(bi_int(a, 0, "NET_PEER_ADDR")?)?.peer_host),
            "net_peer_port" => Value::Int(self.net_sock(bi_int(a, 0, "NET_PEER_PORT")?)?.peer_port),
            "net_close" => { let i = bi_int(a, 0, "NET_CLOSE")? as usize; if let Some(s) = self.tcp_socks.get_mut(i) { *s = None; } Value::Nil }
            "net_close_listener" => { let i = bi_int(a, 0, "NET_CLOSE_LISTENER")? as usize; if let Some(s) = self.tcp_listeners.get_mut(i) { *s = None; } Value::Nil }
            "net_set_timeout" => { let i = bi_int(a, 0, "NET_SET_TIMEOUT")?; let ms = bi_int(a, 1, "NET_SET_TIMEOUT")?; net::set_timeout_tcp(&self.net_sock(i)?.stream, ms); Value::Nil }
            "net_udp_bind" => { let s = net::udp_bind(bi_int(a, 0, "NET_UDP_BIND")?)?; self.udp_socks.push(Some(s)); Value::Int((self.udp_socks.len() - 1) as i64) }
            "net_udp_open" => { let s = net::udp_open()?; self.udp_socks.push(Some(s)); Value::Int((self.udp_socks.len() - 1) as i64) }
            "net_udp_port" => Value::Int(self.net_udp(bi_int(a, 0, "NET_UDP_PORT")?)?.bound_port),
            "net_udp_send" => {
                let i = bi_int(a, 0, "NET_UDP_SEND")?;
                let h = bi_str(a, 1, "NET_UDP_SEND")?.to_string();
                let p = bi_int(a, 2, "NET_UDP_SEND")?;
                let t = bi_str(a, 3, "NET_UDP_SEND")?.to_string();
                Value::Int(net::udp_send(self.net_udp(i)?, &h, p, &t)?)
            }
            "net_udp_recv" => { let i = bi_int(a, 0, "NET_UDP_RECV")?; let n = bi_int(a, 1, "NET_UDP_RECV")?; Value::str_rc(&net::udp_recv(self.net_udp_mut(i)?, n)?) }
            "net_udp_last_from" => {
                let s = self.net_udp(bi_int(a, 0, "NET_UDP_LAST_FROM")?)?;
                if s.last_from.0.is_empty() { Value::str_rc("") } else { Value::str_rc(&format!("{}:{}", s.last_from.0, s.last_from.1)) }
            }
            "net_udp_set_timeout" => { let i = bi_int(a, 0, "NET_UDP_SET_TIMEOUT")?; let ms = bi_int(a, 1, "NET_UDP_SET_TIMEOUT")?; net::set_timeout_udp(&self.net_udp(i)?.sock, ms); Value::Nil }
            "net_udp_close" => { let i = bi_int(a, 0, "NET_UDP_CLOSE")? as usize; if let Some(s) = self.udp_socks.get_mut(i) { *s = None; } Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul html (HTTP/HTML/URL, Feature `http`)
    // ===================================================================
    fn try_html(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("http_") || name.starts_with("html_") || name.starts_with("url_")) { return Ok(None); }
        #[cfg(feature = "http")]
        { return self.try_html_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    #[cfg(feature = "http")]
    fn try_html_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::html;
        let v = match name {
            "http_get" => {
                let url = bi_str(a, 0, "HTTP_GET")?.to_string();
                match html::http_get(&url) {
                    Ok(r) => { self.http_status = r.status; self.http_headers = r.headers; Value::str_rc(&String::from_utf8_lossy(&r.body)) }
                    Err(e) => { if e.status != 0 { self.http_status = e.status; self.http_headers = e.headers; } return Err(e.msg); }
                }
            }
            "http_post" => {
                let url = bi_str(a, 0, "HTTP_POST")?.to_string();
                let body = bi_str(a, 1, "HTTP_POST")?.to_string();
                match html::http_post(&url, &body) {
                    Ok(r) => { self.http_status = r.status; self.http_headers = r.headers; Value::str_rc(&String::from_utf8_lossy(&r.body)) }
                    Err(e) => { if e.status != 0 { self.http_status = e.status; self.http_headers = e.headers; } return Err(e.msg); }
                }
            }
            "http_download" => {
                let url = bi_str(a, 0, "HTTP_DOWNLOAD")?.to_string();
                let path = bi_str(a, 1, "HTTP_DOWNLOAD")?.to_string();
                match html::http_get(&url) {
                    Ok(r) => {
                        self.http_status = r.status; self.http_headers = r.headers;
                        std::fs::write(&path, &r.body).map_err(|e| format!("HTTP_DOWNLOAD: Datei nicht schreibbar: {}", e))?;
                        Value::Int(r.body.len() as i64)
                    }
                    Err(e) => { if e.status != 0 { self.http_status = e.status; self.http_headers = e.headers; } return Err(e.msg); }
                }
            }
            "http_status" => Value::Int(self.http_status),
            "http_header" => {
                let n = bi_str(a, 0, "HTTP_HEADER")?.to_lowercase();
                Value::str_rc(self.http_headers.iter().find(|(k, _)| *k == n).map(|(_, v)| v.as_str()).unwrap_or(""))
            }
            "url_encode" => Value::str_rc(&html::url_encode(bi_str(a, 0, "URL_ENCODE")?)),
            "url_decode" => Value::str_rc(&html::url_decode(bi_str(a, 0, "URL_DECODE")?)),
            "html_text" => Value::str_rc(&html::html_text(bi_str(a, 0, "HTML_TEXT")?)),
            "html_get_attr" => Value::str_rc(&html::html_get_attr(bi_str(a, 0, "HTML_GET_ATTR")?, bi_str(a, 1, "HTML_GET_ATTR")?)),
            "html_find_all" => {
                let items = html::html_find_all(bi_str(a, 0, "HTML_FIND_ALL")?, bi_str(a, 1, "HTML_FIND_ALL")?);
                let n = items.len() as i64;
                let mut arr = GbArray::new("string".to_string(), vec![n], || Value::str_rc(""));
                for (i, s) in items.into_iter().enumerate() { arr.cells.set(i, Value::str_rc(&s)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul serial (Feature `serial`)
    // ===================================================================
    fn try_serial(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("serial_")) { return Ok(None); }
        #[cfg(feature = "serial")]
        { return self.try_serial_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }
    #[cfg(feature = "serial")]
    fn ser_port(&mut self, idx: i64) -> R<&mut crate::serial::Port> {
        self.serial_ports.get_mut(idx as usize).and_then(|o| o.as_mut())
            .ok_or_else(|| format!("SERIAL: ungueltiges/geschlossenes SERIAL_HANDLE {}", idx))
    }
    #[cfg(feature = "serial")]
    fn try_serial_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::serial;
        let v = match name {
            "serial_ports" => Value::str_rc(&serial::ports()),
            "serial_open" => {
                let p = serial::open(bi_str(a, 0, "SERIAL_OPEN")?, bi_int(a, 1, "SERIAL_OPEN")?)?;
                self.serial_ports.push(Some(p));
                Value::Int((self.serial_ports.len() - 1) as i64)
            }
            "serial_close" => { let i = bi_int(a, 0, "SERIAL_CLOSE")? as usize; if let Some(s) = self.serial_ports.get_mut(i) { *s = None; } Value::Nil }
            "serial_is_open" => { let i = bi_int(a, 0, "SERIAL_IS_OPEN")?; Value::Bool(self.serial_ports.get(i as usize).map(|o| o.is_some()).unwrap_or(false)) }
            "serial_write" => { let i = bi_int(a, 0, "SERIAL_WRITE")?; let s = bi_str(a, 1, "SERIAL_WRITE")?.to_string(); Value::Int(serial::write(self.ser_port(i)?, &s)?) }
            "serial_read" => { let i = bi_int(a, 0, "SERIAL_READ")?; let n = bi_int(a, 1, "SERIAL_READ")?; Value::str_rc(&serial::read(self.ser_port(i)?, n)?) }
            "serial_readline" => { let i = bi_int(a, 0, "SERIAL_READLINE")?; Value::str_rc(&serial::readline(self.ser_port(i)?)?) }
            "serial_available" => { let i = bi_int(a, 0, "SERIAL_AVAILABLE")?; Value::Int(serial::available(self.ser_port(i)?)?) }
            "serial_flush" => { let i = bi_int(a, 0, "SERIAL_FLUSH")?; serial::flush(self.ser_port(i)?); Value::Nil }
            "serial_timeout" => { let i = bi_int(a, 0, "SERIAL_TIMEOUT")?; let s = bi_num(a, 1, "SERIAL_TIMEOUT")?; serial::set_timeout(self.ser_port(i)?, s); Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul usb (Feature `usb`)
    // ===================================================================
    fn try_usb(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("usb_")) { return Ok(None); }
        #[cfg(feature = "usb")]
        { return self.try_usb_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }
    #[cfg(feature = "usb")]
    fn usb_dev(&self, idx: i64) -> R<&hidapi::HidDevice> {
        self.usb_devs.get(idx as usize).and_then(|o| o.as_ref())
            .ok_or_else(|| format!("USB: ungueltiges/geschlossenes USB_HANDLE {}", idx))
    }
    #[cfg(feature = "usb")]
    fn try_usb_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::usb;
        let v = match name {
            "usb_list" => Value::str_rc(&usb::list()?),
            "usb_open" => { let d = usb::open(bi_int(a, 0, "USB_OPEN")?, bi_int(a, 1, "USB_OPEN")?)?; self.usb_devs.push(Some(d)); Value::Int((self.usb_devs.len() - 1) as i64) }
            "usb_open_path" => { let d = usb::open_path(bi_str(a, 0, "USB_OPEN_PATH")?)?; self.usb_devs.push(Some(d)); Value::Int((self.usb_devs.len() - 1) as i64) }
            "usb_close" => { let i = bi_int(a, 0, "USB_CLOSE")? as usize; if let Some(s) = self.usb_devs.get_mut(i) { *s = None; } Value::Nil }
            "usb_write" => { let i = bi_int(a, 0, "USB_WRITE")?; let s = bi_str(a, 1, "USB_WRITE")?.to_string(); Value::Int(usb::write(self.usb_dev(i)?, &s)?) }
            "usb_read" => { let i = bi_int(a, 0, "USB_READ")?; let n = bi_int(a, 1, "USB_READ")?; let t = bi_int(a, 2, "USB_READ")?; Value::str_rc(&usb::read(self.usb_dev(i)?, n, t)?) }
            "usb_product" => Value::str_rc(&usb::product(self.usb_dev(bi_int(a, 0, "USB_PRODUCT")?)?)),
            "usb_manufacturer" => Value::str_rc(&usb::manufacturer(self.usb_dev(bi_int(a, 0, "USB_MANUFACTURER")?)?)),
            "usb_serial" => Value::str_rc(&usb::serial(self.usb_dev(bi_int(a, 0, "USB_SERIAL")?)?)),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul wifi (Feature `wifi`, Windows-only)
    // ===================================================================
    fn try_wifi(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("wifi_")) { return Ok(None); }
        #[cfg(feature = "wifi")]
        {
            use crate::wifi;
            let v = match name {
                "wifi_available" => Value::Bool(wifi::available()),
                "wifi_current" => Value::str_rc(&wifi::current()?),
                "wifi_signal" => Value::Int(wifi::signal()?),
                "wifi_scan" => Value::str_rc(&wifi::scan()?),
                "wifi_connect" => Value::Bool(wifi::connect(bi_str(a, 0, "WIFI_CONNECT")?, bi_str(a, 1, "WIFI_CONNECT")?)?),
                "wifi_disconnect" => Value::Bool(wifi::disconnect()?),
                "wifi_profiles" => Value::str_rc(&wifi::profiles()?),
                "wifi_delete_profile" => Value::Bool(wifi::delete_profile(bi_str(a, 0, "WIFI_DELETE_PROFILE")?)?),
                _ => return Ok(None),
            };
            return Ok(Some(v));
        }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    // ===================================================================
    // Modul bt (Feature `bt`, BLE async)
    // ===================================================================
    fn try_bt(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("bt_")) { return Ok(None); }
        #[cfg(feature = "bt")]
        { return self.try_bt_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }
    #[cfg(feature = "bt")]
    fn bt_periph(&self, idx: i64) -> R<&btleplug::platform::Peripheral> {
        self.bt_periphs.get(idx as usize).and_then(|o| o.as_ref())
            .ok_or_else(|| format!("BT: ungueltiges/getrenntes BT_HANDLE {}", idx))
    }
    #[cfg(feature = "bt")]
    fn try_bt_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::bt;
        let v = match name {
            "bt_scan" => Value::str_rc(&bt::scan(bi_num(a, 0, "BT_SCAN")?)?),
            "bt_connect" => { let p = bt::connect(bi_str(a, 0, "BT_CONNECT")?)?; self.bt_periphs.push(Some(p)); Value::Int((self.bt_periphs.len() - 1) as i64) }
            "bt_disconnect" => { let i = bi_int(a, 0, "BT_DISCONNECT")?; bt::disconnect(self.bt_periph(i)?)?; if let Some(s) = self.bt_periphs.get_mut(i as usize) { *s = None; } Value::Nil }
            "bt_is_connected" => { let i = bi_int(a, 0, "BT_IS_CONNECTED")?; Value::Bool(match self.bt_periphs.get(i as usize).and_then(|o| o.as_ref()) { Some(p) => bt::is_connected(p), None => false }) }
            "bt_services" => { let i = bi_int(a, 0, "BT_SERVICES")?; Value::str_rc(&bt::services(self.bt_periph(i)?)?) }
            "bt_characteristics" => { let i = bi_int(a, 0, "BT_CHARACTERISTICS")?; let s = bi_str(a, 1, "BT_CHARACTERISTICS")?.to_string(); Value::str_rc(&bt::characteristics(self.bt_periph(i)?, &s)?) }
            "bt_read" => { let i = bi_int(a, 0, "BT_READ")?; let c = bi_str(a, 1, "BT_READ")?.to_string(); Value::str_rc(&bt::read(self.bt_periph(i)?, &c)?) }
            "bt_write" => { let i = bi_int(a, 0, "BT_WRITE")?; let c = bi_str(a, 1, "BT_WRITE")?.to_string(); let d = bi_str(a, 2, "BT_WRITE")?.to_string(); bt::write(self.bt_periph(i)?, &c, &d)?; Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Modul `timer` (timer.rs): AFTER/EVERY/CANCEL/UPDATE + COOLDOWN.
    /// Kein Grafik-Bezug -- laeuft auch konsolen-only. TIMER_UPDATE feuert
    /// faellige FUNCREF-Callbacks nach dem gleichen Muster wie GUI_UPDATE.
    fn try_timer(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("timer_") || name == "cooldown") { return Ok(None); }
        fn t_int(a: &[Value], i: usize, fn_: &str) -> R<i64> {
            match a.get(i) { Some(Value::Int(n)) => Ok(*n), _ => Err(format!("{}: erwartet INTEGER (Arg {})", fn_, i + 1)) }
        }
        fn t_str<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) { Some(Value::Str(s)) => Ok(s), _ => Err(format!("{}: erwartet STRING (Arg {})", fn_, i + 1)) }
        }
        fn t_func(a: &[Value], i: usize, fn_: &str) -> R<String> {
            match a.get(i) { Some(Value::FuncRef(n)) => Ok(n.to_string()), _ => Err(format!("{}: erwartet FUNCREF (Arg {})", fn_, i + 1)) }
        }
        let r = match name {
            "timer_after" => {
                let ms = t_int(a, 0, "TIMER_AFTER")?;
                if ms < 0 { return Err("TIMER_AFTER: ms muss >= 0 sein".into()); }
                Value::Int(self.timers.after(ms, t_func(a, 1, "TIMER_AFTER")?))
            }
            "timer_every" => {
                let ms = t_int(a, 0, "TIMER_EVERY")?;
                if ms <= 0 { return Err("TIMER_EVERY: ms muss > 0 sein".into()); }
                Value::Int(self.timers.every(ms, t_func(a, 1, "TIMER_EVERY")?))
            }
            "timer_cancel" => { self.timers.cancel(t_int(a, 0, "TIMER_CANCEL")?); Value::Nil }
            "timer_active" => Value::Bool(self.timers.active(t_int(a, 0, "TIMER_ACTIVE")?)),
            "timer_count" => Value::Int(self.timers.count()),
            "timer_clear" => { self.timers.clear(); Value::Nil }
            "timer_update" => {
                // Faellige Callbacks NACH dem Einsammeln feuern -- ein Callback
                // darf selbst Timer registrieren/canceln; neue Eintraege werden
                // erst beim naechsten Update faellig geprueft.
                for fname in self.timers.take_due() {
                    let f = self.prog.func(fname.as_str()).ok_or_else(||
                        format!("TIMER-Callback: Funktion '{}' existiert nicht", fname))?;
                    self.exec(f, Vec::new(), None)?;
                }
                Value::Nil
            }
            "cooldown" => {
                let id = t_str(a, 0, "COOLDOWN")?.to_string();
                let ms = t_int(a, 1, "COOLDOWN")?;
                if ms < 0 { return Err("COOLDOWN: ms muss >= 0 sein".into()); }
                Value::Bool(self.timers.cooldown(&id, ms))
            }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }

    fn try_scene(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("scene_")) { return Ok(None); }
        fn sa<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) { Some(Value::Str(s)) => Ok(s), _ => Err(format!("{}: erwartet STRING", fn_)) }
        }
        let r = match name {
            "scene_push" => { self.scene_stack.push((sa(a, 0, "SCENE_PUSH")?.to_string(), HashMap::new())); Value::Nil }
            "scene_pop" => { if self.scene_stack.is_empty() { return Err("SCENE_POP: Stack ist bereits leer".into()); } self.scene_stack.pop(); Value::Nil }
            "scene_switch" => { let n = sa(a, 0, "SCENE_SWITCH")?.to_string(); self.scene_stack.clear(); self.scene_stack.push((n, HashMap::new())); Value::Nil }
            "scene_current" => Value::str_rc(self.scene_stack.last().map(|(n, _)| n.as_str()).unwrap_or("")),
            "scene_depth" => Value::Int(self.scene_stack.len() as i64),
            "scene_has" => { let n = sa(a, 0, "SCENE_HAS")?; Value::Bool(self.scene_stack.iter().any(|(x, _)| x == n)) }
            "scene_reset" => { self.scene_stack.clear(); Value::Nil }
            "scene_set_int" | "scene_set_float" | "scene_set_string" | "scene_set_bool" => {
                let key = sa(a, 0, "SCENE_SET")?.to_string();
                let v = match name {
                    "scene_set_int" => match a.get(1) { Some(Value::Int(i)) => Value::Int(*i), _ => return Err("SCENE_SET_INT: value muss INTEGER sein".into()) },
                    "scene_set_float" => match a.get(1) { Some(Value::Int(i)) => Value::Float(*i as f64), Some(Value::Float(f)) => Value::Float(*f), _ => return Err("SCENE_SET_FLOAT: value muss Zahl sein".into()) },
                    "scene_set_string" => match a.get(1) { Some(Value::Str(s)) => Value::Str(s.clone()), _ => return Err("SCENE_SET_STRING: value muss STRING sein".into()) },
                    _ => match a.get(1) { Some(Value::Bool(b)) => Value::Bool(*b), _ => return Err("SCENE_SET_BOOL: value muss BOOLEAN sein".into()) },
                };
                self.scene_top_mut()?.1.insert(key, v); Value::Nil
            }
            "scene_get_int" | "scene_get_float" | "scene_get_string" | "scene_get_bool" => {
                let key = sa(a, 0, "SCENE_GET")?;
                let data = &self.scene_top()?.1;
                let val = data.get(key).ok_or_else(|| format!("{}: Key '{}' nicht in Scene", name.to_uppercase(), key))?;
                let ok = scene_type_ok(val, name);
                if !ok { return Err(format!("{}: Key '{}' hat falschen Typ", name.to_uppercase(), key)); }
                val.clone()
            }
            "scene_get_int_or" | "scene_get_float_or" | "scene_get_string_or" | "scene_get_bool_or" => {
                let key = sa(a, 0, "SCENE_GET_OR")?;
                let def = a.get(1).cloned().unwrap_or(Value::Nil);
                let data = &self.scene_top()?.1;
                match data.get(key) { Some(v) if scene_type_ok(v, name) => v.clone(), _ => def }
            }
            "scene_has_key" => { let key = sa(a, 0, "SCENE_HAS_KEY")?; Value::Bool(self.scene_top()?.1.contains_key(key)) }
            "scene_delete" => { let key = sa(a, 0, "SCENE_DELETE")?.to_string(); self.scene_top_mut()?.1.remove(&key); Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }

    #[cfg(not(feature = "graphics"))]
    fn try_graphics(&mut self, _name: &str, _args: &[Value]) -> R<Option<Value>> {
        Ok(None)
    }

    #[cfg(not(feature = "graphics"))]
    fn try_gui(&mut self, _name: &str, _a: &[Value]) -> R<Option<Value>> { Ok(None) }

    /// Retained-Mode-GUI (Modul `gui`). Liefert Ok(None) wenn `name` kein
    /// gui-Builtin ist. GUI_UPDATE/GUI_DRAW brauchen die Grafik (self.gfx),
    /// alles andere nur den GUI-State (self.gui).
    #[cfg(feature = "graphics")]
    fn try_gui(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("gui_") { return Ok(None); }
        fn gi(a: &[Value], i: usize, f: &str) -> R<i64> {
            match a.get(i) { Some(Value::Int(n)) => Ok(*n),
                _ => Err(format!("{}: erwartet INTEGER (Arg {})", f, i + 1)) }
        }
        fn gs(a: &[Value], i: usize, f: &str) -> R<String> {
            match a.get(i) { Some(Value::Str(s)) => Ok(s.to_string()),
                _ => Err(format!("{}: erwartet STRING (Arg {})", f, i + 1)) }
        }
        fn gbool(a: &[Value], i: usize, f: &str) -> R<bool> {
            match a.get(i) { Some(Value::Bool(b)) => Ok(*b),
                _ => Err(format!("{}: erwartet BOOLEAN (Arg {})", f, i + 1)) }
        }
        fn gnum(a: &[Value], i: usize, f: &str) -> R<f64> {
            match a.get(i) { Some(Value::Int(n)) => Ok(*n as f64), Some(Value::Float(x)) => Ok(*x),
                _ => Err(format!("{}: erwartet Zahl (Arg {})", f, i + 1)) }
        }
        // FUNCREF-Arg -> Option<Name>; NIL entfernt den Callback.
        fn gfunc(a: &[Value], i: usize, f: &str) -> R<Option<String>> {
            match a.get(i) {
                Some(Value::Nil) | None => Ok(None),
                Some(Value::FuncRef(n)) => Ok(Some(n.to_string())),
                _ => Err(format!("{}: handler muss FUNCREF sein", f)),
            }
        }
        // 1D ARRAY OF STRING -> Vec<String>.
        fn gstrs(a: &[Value], i: usize, f: &str) -> R<Vec<String>> {
            match a.get(i) {
                Some(Value::Array(arr)) => {
                    let arr = arr.borrow();
                    if arr.element_type != "string" || arr.dims.len() != 1 {
                        return Err(format!("{}: erwartet 1D ARRAY OF STRING", f));
                    }
                    Ok(arr.cells.iter().map(|v| match v { Value::Str(s) => s.to_string(), o => o.fmt() }).collect())
                }
                _ => Err(format!("{}: erwartet ARRAY OF STRING", f)),
            }
        }
        // 2D ARRAY OF STRING -> Vec<Vec<String>>.
        fn gstrs2(a: &[Value], i: usize, f: &str) -> R<Vec<Vec<String>>> {
            match a.get(i) {
                Some(Value::Array(arr)) => {
                    let arr = arr.borrow();
                    if arr.element_type != "string" || arr.dims.len() != 2 {
                        return Err(format!("{}: erwartet 2D ARRAY OF STRING", f));
                    }
                    let (rows, cols) = (arr.dims[0] as usize, arr.dims[1] as usize);
                    let mut out = Vec::with_capacity(rows);
                    for r in 0..rows {
                        let mut row = Vec::with_capacity(cols);
                        for c in 0..cols {
                            row.push(match &arr.cells.get(r * cols + c) { Value::Str(s) => s.to_string(), o => o.fmt() });
                        }
                        out.push(row);
                    }
                    Ok(out)
                }
                _ => Err(format!("{}: erwartet 2D ARRAY OF STRING", f)),
            }
        }
        // 1D ARRAY OF INTEGER -> Vec<i32> (oder None bei NIL).
        fn gints_opt(a: &[Value], i: usize, f: &str) -> R<Option<Vec<i32>>> {
            match a.get(i) {
                Some(Value::Nil) | None => Ok(None),
                Some(Value::Array(arr)) => {
                    let arr = arr.borrow();
                    if arr.element_type != "integer" || arr.dims.len() != 1 {
                        return Err(format!("{}: erwartet 1D ARRAY OF INTEGER", f));
                    }
                    Ok(Some(arr.cells.iter().map(|v| match v { Value::Int(n) => n as i32, _ => 0 }).collect()))
                }
                _ => Err(format!("{}: erwartet ARRAY OF INTEGER oder NIL", f)),
            }
        }
        let r = match name {
            "gui_window" => Value::Int(self.gui.new_window(
                gs(a,0,"GUI_WINDOW")?, gi(a,1,"GUI_WINDOW")? as i32, gi(a,2,"GUI_WINDOW")? as i32,
                gi(a,3,"GUI_WINDOW")? as i32, gi(a,4,"GUI_WINDOW")? as i32)),
            "gui_window_movable" => { self.gui.window_movable(gi(a,0,"GUI_WINDOW_MOVABLE")?, gbool(a,1,"GUI_WINDOW_MOVABLE")?)?; Value::Nil }
            "gui_window_closable" => { self.gui.window_closable(gi(a,0,"GUI_WINDOW_CLOSABLE")?, gbool(a,1,"GUI_WINDOW_CLOSABLE")?)?; Value::Nil }
            "gui_window_visible" => { self.gui.window_visible(gi(a,0,"GUI_WINDOW_VISIBLE")?, gbool(a,1,"GUI_WINDOW_VISIBLE")?)?; Value::Nil }
            "gui_window_resizable" => { self.gui.window_resizable(gi(a,0,"GUI_WINDOW_RESIZABLE")?, gbool(a,1,"GUI_WINDOW_RESIZABLE")?)?; Value::Nil }
            "gui_window_chrome" => { self.gui.window_chrome(gi(a,0,"GUI_WINDOW_CHROME")?, gbool(a,1,"GUI_WINDOW_CHROME")?)?; Value::Nil }
            "gui_window_set_min_size" => { self.gui.window_min_size(gi(a,0,"GUI_WINDOW_SET_MIN_SIZE")?, gi(a,1,"GUI_WINDOW_SET_MIN_SIZE")? as i32, gi(a,2,"GUI_WINDOW_SET_MIN_SIZE")? as i32)?; Value::Nil }
            "gui_window_set_max_size" => { self.gui.window_max_size(gi(a,0,"GUI_WINDOW_SET_MAX_SIZE")?, gi(a,1,"GUI_WINDOW_SET_MAX_SIZE")? as i32, gi(a,2,"GUI_WINDOW_SET_MAX_SIZE")? as i32)?; Value::Nil }
            "gui_window_closed" => Value::Bool(self.gui.window_closed(gi(a,0,"GUI_WINDOW_CLOSED")?)?),
            "gui_button" => Value::Int(self.gui.button(gi(a,0,"GUI_BUTTON")?, gs(a,1,"GUI_BUTTON")?,
                gi(a,2,"GUI_BUTTON")? as i32, gi(a,3,"GUI_BUTTON")? as i32,
                gi(a,4,"GUI_BUTTON")? as i32, gi(a,5,"GUI_BUTTON")? as i32)?),
            "gui_label" => {
                let color = if a.len() >= 5 { Some(gi(a,4,"GUI_LABEL")?) } else { None };
                Value::Int(self.gui.label(gi(a,0,"GUI_LABEL")?, gs(a,1,"GUI_LABEL")?,
                    gi(a,2,"GUI_LABEL")? as i32, gi(a,3,"GUI_LABEL")? as i32, color)?)
            }
            "gui_checkbox" => {
                let def = if a.len() >= 5 { gbool(a,4,"GUI_CHECKBOX")? } else { false };
                Value::Int(self.gui.checkbox(gi(a,0,"GUI_CHECKBOX")?, gs(a,1,"GUI_CHECKBOX")?,
                    gi(a,2,"GUI_CHECKBOX")? as i32, gi(a,3,"GUI_CHECKBOX")? as i32, def)?)
            }
            "gui_slider" => {
                let mn = gnum(a,4,"GUI_SLIDER")?; let mx = gnum(a,5,"GUI_SLIDER")?;
                let def = if a.len() >= 7 { gnum(a,6,"GUI_SLIDER")? } else { mn };
                Value::Int(self.gui.slider(gi(a,0,"GUI_SLIDER")?, gi(a,1,"GUI_SLIDER")? as i32,
                    gi(a,2,"GUI_SLIDER")? as i32, gi(a,3,"GUI_SLIDER")? as i32, mn, mx, def)?)
            }
            "gui_panel" => {
                let title = if a.len() >= 6 { gs(a,5,"GUI_PANEL")? } else { String::new() };
                Value::Int(self.gui.panel(gi(a,0,"GUI_PANEL")?, gi(a,1,"GUI_PANEL")? as i32, gi(a,2,"GUI_PANEL")? as i32,
                    gi(a,3,"GUI_PANEL")? as i32, gi(a,4,"GUI_PANEL")? as i32, title)?)
            }
            "gui_separator" => Value::Int(self.gui.separator(gi(a,0,"GUI_SEPARATOR")?,
                gi(a,1,"GUI_SEPARATOR")? as i32, gi(a,2,"GUI_SEPARATOR")? as i32, gi(a,3,"GUI_SEPARATOR")? as i32)?),
            "gui_groupbox" => {
                let title = if a.len() >= 6 { gs(a,5,"GUI_GROUPBOX")? } else { String::new() };
                Value::Int(self.gui.groupbox(gi(a,0,"GUI_GROUPBOX")?, gi(a,1,"GUI_GROUPBOX")? as i32, gi(a,2,"GUI_GROUPBOX")? as i32,
                    gi(a,3,"GUI_GROUPBOX")? as i32, gi(a,4,"GUI_GROUPBOX")? as i32, title)?)
            }
            "gui_textinput" => {
                let ph = if a.len() >= 6 { gs(a,5,"GUI_TEXTINPUT")? } else { String::new() };
                Value::Int(self.gui.textinput(gi(a,0,"GUI_TEXTINPUT")?, gi(a,1,"GUI_TEXTINPUT")? as i32, gi(a,2,"GUI_TEXTINPUT")? as i32,
                    gi(a,3,"GUI_TEXTINPUT")? as i32, gi(a,4,"GUI_TEXTINPUT")? as i32, ph)?)
            }
            // --- Formular-Widgets (Phase 3) ---
            "gui_radio" => Value::Int(self.gui.radio(gi(a,0,"GUI_RADIO")?, gs(a,1,"GUI_RADIO")?, gs(a,2,"GUI_RADIO")?,
                gi(a,3,"GUI_RADIO")? as i32, gi(a,4,"GUI_RADIO")? as i32)?),
            "gui_radio_selected" => Value::Int(self.gui.radio_selected(gi(a,0,"GUI_RADIO_SELECTED")?)?),
            "gui_progress" => Value::Int(self.gui.progress(gi(a,0,"GUI_PROGRESS")?, gi(a,1,"GUI_PROGRESS")? as i32,
                gi(a,2,"GUI_PROGRESS")? as i32, gi(a,3,"GUI_PROGRESS")? as i32, gi(a,4,"GUI_PROGRESS")? as i32)?),
            "gui_dropdown" => Value::Int(self.gui.dropdown(gi(a,0,"GUI_DROPDOWN")?, gi(a,1,"GUI_DROPDOWN")? as i32,
                gi(a,2,"GUI_DROPDOWN")? as i32, gi(a,3,"GUI_DROPDOWN")? as i32, gi(a,4,"GUI_DROPDOWN")? as i32,
                gstrs(a,5,"GUI_DROPDOWN")?)?),
            "gui_dropdown_selected" => Value::Int(self.gui.dropdown_selected(gi(a,0,"GUI_DROPDOWN_SELECTED")?)?),
            "gui_dropdown_text" => Value::str_rc(&self.gui.dropdown_text(gi(a,0,"GUI_DROPDOWN_TEXT")?)?),
            "gui_dropdown_set_selected" => { self.gui.dropdown_set_selected(gi(a,0,"GUI_DROPDOWN_SET_SELECTED")?, gi(a,1,"GUI_DROPDOWN_SET_SELECTED")?)?; Value::Nil }
            "gui_set_dropdown" => { self.gui.set_dropdown_items(gi(a,0,"GUI_SET_DROPDOWN")?, gstrs(a,1,"GUI_SET_DROPDOWN")?)?; Value::Nil }
            // --- ListBox (teilt die item-Logik mit Dropdown) ---
            "gui_listbox" => Value::Int(self.gui.listbox(gi(a,0,"GUI_LISTBOX")?, gi(a,1,"GUI_LISTBOX")? as i32,
                gi(a,2,"GUI_LISTBOX")? as i32, gi(a,3,"GUI_LISTBOX")? as i32, gi(a,4,"GUI_LISTBOX")? as i32,
                gstrs(a,5,"GUI_LISTBOX")?)?),
            "gui_listbox_selected" => Value::Int(self.gui.dropdown_selected(gi(a,0,"GUI_LISTBOX_SELECTED")?)?),
            "gui_listbox_text" => Value::str_rc(&self.gui.dropdown_text(gi(a,0,"GUI_LISTBOX_TEXT")?)?),
            "gui_listbox_set_selected" => { self.gui.dropdown_set_selected(gi(a,0,"GUI_LISTBOX_SET_SELECTED")?, gi(a,1,"GUI_LISTBOX_SET_SELECTED")?)?; Value::Nil }
            "gui_set_listbox" => { self.gui.set_dropdown_items(gi(a,0,"GUI_SET_LISTBOX")?, gstrs(a,1,"GUI_SET_LISTBOX")?)?; Value::Nil }
            // --- Image + Canvas ---
            "gui_image" => Value::Int(self.gui.image(gi(a,0,"GUI_IMAGE")?, gi(a,1,"GUI_IMAGE")? as i32,
                gi(a,2,"GUI_IMAGE")? as i32, gi(a,3,"GUI_IMAGE")? as i32, gi(a,4,"GUI_IMAGE")? as i32, gi(a,5,"GUI_IMAGE")?)?),
            "gui_set_image" => { self.gui.set_image(gi(a,0,"GUI_SET_IMAGE")?, gi(a,1,"GUI_SET_IMAGE")?)?; Value::Nil }
            "gui_canvas" => Value::Int(self.gui.canvas(gi(a,0,"GUI_CANVAS")?, gi(a,1,"GUI_CANVAS")? as i32,
                gi(a,2,"GUI_CANVAS")? as i32, gi(a,3,"GUI_CANVAS")? as i32, gi(a,4,"GUI_CANVAS")? as i32)?),
            "gui_canvas_x" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_X")?)?.0 as i64),
            "gui_canvas_y" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_Y")?)?.1 as i64),
            "gui_canvas_w" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_W")?)?.2 as i64),
            "gui_canvas_h" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_H")?)?.3 as i64),
            "gui_clicked" => Value::Bool(self.gui.clicked(gi(a,0,"GUI_CLICKED")?)?),
            "gui_hovered" => Value::Bool(self.gui.hovered(gi(a,0,"GUI_HOVERED")?)?),
            "gui_checked" => Value::Bool(self.gui.checked(gi(a,0,"GUI_CHECKED")?)?),
            "gui_value" => Value::Float(self.gui.value(gi(a,0,"GUI_VALUE")?)?),
            "gui_text" => Value::str_rc(&self.gui.text(gi(a,0,"GUI_TEXT")?)?),
            "gui_set_text" => { self.gui.set_text(gi(a,0,"GUI_SET_TEXT")?, gs(a,1,"GUI_SET_TEXT")?)?; Value::Nil }
            "gui_set_checked" => { self.gui.set_checked(gi(a,0,"GUI_SET_CHECKED")?, gbool(a,1,"GUI_SET_CHECKED")?)?; Value::Nil }
            "gui_set_value" => { self.gui.set_value(gi(a,0,"GUI_SET_VALUE")?, gnum(a,1,"GUI_SET_VALUE")?)?; Value::Nil }
            "gui_on_click" => { self.gui.on_click(gi(a,0,"GUI_ON_CLICK")?, gfunc(a,1,"GUI_ON_CLICK")?)?; Value::Nil }
            "gui_on_change" => { self.gui.on_change(gi(a,0,"GUI_ON_CHANGE")?, gfunc(a,1,"GUI_ON_CHANGE")?)?; Value::Nil }
            "gui_theme" => { self.gui.theme_accent(gi(a,0,"GUI_THEME")?); Value::Nil }
            "gui_theme_set" => { self.gui.theme_set(gs(a,0,"GUI_THEME_SET")?, gi(a,1,"GUI_THEME_SET")?)?; Value::Nil }
            "gui_theme_get" => Value::Int(self.gui.theme_get(&gs(a,0,"GUI_THEME_GET")?)?),
            "gui_metric_set" => { self.gui.metric_set(gs(a,0,"GUI_METRIC_SET")?, gi(a,1,"GUI_METRIC_SET")? as i32)?; Value::Nil }
            "gui_metric_get" => Value::Int(self.gui.metric_get(&gs(a,0,"GUI_METRIC_GET")?)? as i64),
            "gui_set_color" => { self.gui.set_color(gi(a,0,"GUI_SET_COLOR")?, gs(a,1,"GUI_SET_COLOR")?, gi(a,2,"GUI_SET_COLOR")?)?; Value::Nil }
            "gui_theme_preset" => { self.gui.theme_preset(&gs(a,0,"GUI_THEME_PRESET")?)?; Value::Nil }
            "gui_reset" => { self.gui.reset(); Value::Nil }
            // --- Laufzeit-Manipulation (Geometrie / Lifecycle / Hit-Test) ---
            "gui_set_bounds" => { self.gui.set_bounds(gi(a,0,"GUI_SET_BOUNDS")?, gi(a,1,"GUI_SET_BOUNDS")? as i32, gi(a,2,"GUI_SET_BOUNDS")? as i32, gi(a,3,"GUI_SET_BOUNDS")? as i32, gi(a,4,"GUI_SET_BOUNDS")? as i32)?; Value::Nil }
            "gui_get_x" => Value::Int(self.gui.widget_bounds(gi(a,0,"GUI_GET_X")?, "GUI_GET_X")?.0 as i64),
            "gui_get_y" => Value::Int(self.gui.widget_bounds(gi(a,0,"GUI_GET_Y")?, "GUI_GET_Y")?.1 as i64),
            "gui_get_w" => Value::Int(self.gui.widget_bounds(gi(a,0,"GUI_GET_W")?, "GUI_GET_W")?.2 as i64),
            "gui_get_h" => Value::Int(self.gui.widget_bounds(gi(a,0,"GUI_GET_H")?, "GUI_GET_H")?.3 as i64),
            "gui_destroy" => { self.gui.destroy(gi(a,0,"GUI_DESTROY")?)?; Value::Nil }
            "gui_set_visible" => { self.gui.set_widget_visible(gi(a,0,"GUI_SET_VISIBLE")?, gbool(a,1,"GUI_SET_VISIBLE")?)?; Value::Nil }
            "gui_visible" => Value::Bool(self.gui.widget_visible(gi(a,0,"GUI_VISIBLE")?)?),
            "gui_kind" => Value::str_rc(self.gui.kind_name(gi(a,0,"GUI_KIND")?)?),
            "gui_focus" => { self.gui.focus(gi(a,0,"GUI_FOCUS")?)?; Value::Nil }
            "gui_set_enabled" => { self.gui.set_enabled(gi(a,0,"GUI_SET_ENABLED")?, gbool(a,1,"GUI_SET_ENABLED")?)?; Value::Nil }
            "gui_enabled" => Value::Bool(self.gui.enabled(gi(a,0,"GUI_ENABLED")?)?),
            "gui_set_font" => { self.gui.set_font(gi(a,0,"GUI_SET_FONT")?, gi(a,1,"GUI_SET_FONT")?)?; Value::Nil }
            "gui_set_font_size" => { self.gui.set_font_size(gi(a,0,"GUI_SET_FONT_SIZE")?, gi(a,1,"GUI_SET_FONT_SIZE")?)?; Value::Nil }
            "gui_set_anchor" => { self.gui.set_anchor(gi(a,0,"GUI_SET_ANCHOR")?, &gs(a,1,"GUI_SET_ANCHOR")?)?; Value::Nil }
            "gui_style_set" => { self.gui.style_set(gs(a,0,"GUI_STYLE_SET")?, gs(a,1,"GUI_STYLE_SET")?, gi(a,2,"GUI_STYLE_SET")?)?; Value::Nil }
            "gui_apply_style" => { self.gui.apply_style(gi(a,0,"GUI_APPLY_STYLE")?, &gs(a,1,"GUI_APPLY_STYLE")?)?; Value::Nil }
            "gui_hit_test" => Value::Int(self.gui.hit_test(gi(a,0,"GUI_HIT_TEST")? as i32, gi(a,1,"GUI_HIT_TEST")? as i32)),
            "gui_window_set_bounds" => { self.gui.window_set_bounds(gi(a,0,"GUI_WINDOW_SET_BOUNDS")?, gi(a,1,"GUI_WINDOW_SET_BOUNDS")? as i32, gi(a,2,"GUI_WINDOW_SET_BOUNDS")? as i32, gi(a,3,"GUI_WINDOW_SET_BOUNDS")? as i32, gi(a,4,"GUI_WINDOW_SET_BOUNDS")? as i32)?; Value::Nil }
            "gui_window_get_x" => Value::Int(self.gui.window_bounds(gi(a,0,"GUI_WINDOW_GET_X")?)?.0 as i64),
            "gui_window_get_y" => Value::Int(self.gui.window_bounds(gi(a,0,"GUI_WINDOW_GET_Y")?)?.1 as i64),
            "gui_window_get_w" => Value::Int(self.gui.window_bounds(gi(a,0,"GUI_WINDOW_GET_W")?)?.2 as i64),
            "gui_window_get_h" => Value::Int(self.gui.window_bounds(gi(a,0,"GUI_WINDOW_GET_H")?)?.3 as i64),
            "gui_window_destroy" => { self.gui.window_destroy(gi(a,0,"GUI_WINDOW_DESTROY")?)?; Value::Nil }
            "gui_window_widget_count" => Value::Int(self.gui.window_widget_count(gi(a,0,"GUI_WINDOW_WIDGET_COUNT")?)?),
            "gui_window_widget" => Value::Int(self.gui.window_widget(gi(a,0,"GUI_WINDOW_WIDGET")?, gi(a,1,"GUI_WINDOW_WIDGET")?)?),
            // --- Serialisierung (Layout als JSON) ---
            "gui_to_json" => Value::str_rc(&self.gui.to_json(gi(a,0,"GUI_TO_JSON")?)?),
            "gui_from_json" => Value::Int(self.gui.from_json(&gs(a,0,"GUI_FROM_JSON")?)?),
            "gui_save" => {
                let s = self.gui.to_json(gi(a,0,"GUI_SAVE")?)?;
                std::fs::write(gs(a,1,"GUI_SAVE")?, s).map_err(|e| format!("GUI_SAVE: {}", e))?;
                Value::Nil
            }
            "gui_load" => {
                let s = std::fs::read_to_string(gs(a,0,"GUI_LOAD")?).map_err(|e| format!("GUI_LOAD: {}", e))?;
                Value::Int(self.gui.from_json(&s)?)
            }
            // --- Tabelle ---
            "gui_table" => {
                let h = self.gui.table(gi(a,0,"GUI_TABLE")?, gi(a,1,"GUI_TABLE")? as i32, gi(a,2,"GUI_TABLE")? as i32,
                    gi(a,3,"GUI_TABLE")? as i32, gi(a,4,"GUI_TABLE")? as i32)?;
                if a.len() == 7 {
                    self.gui.table_set_headers(h, gstrs(a,5,"GUI_TABLE")?)?;
                    self.gui.table_set_rows(h, gstrs2(a,6,"GUI_TABLE")?)?;
                } else if a.len() == 6 {
                    return Err("GUI_TABLE: entweder ohne Daten oder mit headers UND cells aufrufen".into());
                }
                Value::Int(h)
            }
            "gui_table_headers" => { self.gui.table_set_headers(gi(a,0,"GUI_TABLE_HEADERS")?, gstrs(a,1,"GUI_TABLE_HEADERS")?)?; Value::Nil }
            "gui_table_rows" => { self.gui.table_set_rows(gi(a,0,"GUI_TABLE_ROWS")?, gstrs2(a,1,"GUI_TABLE_ROWS")?)?; Value::Nil }
            "gui_table_col_widths" => { self.gui.table_set_col_widths(gi(a,0,"GUI_TABLE_COL_WIDTHS")?, gints_opt(a,1,"GUI_TABLE_COL_WIDTHS")?)?; Value::Nil }
            "gui_table_selected" => Value::Int(self.gui.table_selected(gi(a,0,"GUI_TABLE_SELECTED")?)?),
            "gui_table_set_selected" => { self.gui.table_set_selected(gi(a,0,"GUI_TABLE_SET_SELECTED")?, gi(a,1,"GUI_TABLE_SET_SELECTED")?)?; Value::Nil }
            "gui_table_clicked" => Value::Int(self.gui.table_clicked(gi(a,0,"GUI_TABLE_CLICKED")?)?),
            "gui_table_row_count" => Value::Int(self.gui.table_row_count(gi(a,0,"GUI_TABLE_ROW_COUNT")?)?),
            "gui_update" => {
                {
                    let g = self.gfx.as_mut().ok_or("GUI_UPDATE: vor SCREEN aufgerufen")?;
                    self.gui.update(g);
                }
                // Ausgeloeste FUNCREF-Callbacks feuern (parameterlos), nachdem
                // der State-Update fertig ist -- so kann ein Callback die GUI
                // sicher veraendern; neu ausgeloeste Events landen naechsten Frame.
                for fname in self.gui.take_pending() {
                    let f = self.prog.func(fname.as_str()).ok_or_else(||
                        format!("GUI-Callback: Funktion '{}' existiert nicht", fname))?;
                    self.exec(f, Vec::new(), None)?;
                }
                Value::Nil
            }
            "gui_draw" => {
                let g = self.gfx.as_mut().ok_or("GUI_DRAW: vor SCREEN aufgerufen")?;
                self.gui.draw(g);
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }

    /// Audio-Geraet lazy initialisieren (bei erstem Sound-/Musik-Builtin).
    #[cfg(feature = "graphics")]
    fn audio_mut(&mut self) -> R<&mut crate::audio::Audio> {
        if self.audio.is_none() {
            self.audio = Some(crate::audio::Audio::new()?);
        }
        Ok(self.audio.as_mut().unwrap())
    }

    /// UI_TABLE (Immediate-Mode): scrollbare Tabelle mit fixierter Kopfzeile,
    /// optionalen Zell-Farben, Selektion + klickbaren Headern. 1:1-Port der
    /// Python-Referenz. Liefert die in diesem Frame geklickte Zeile (-1).
    #[cfg(feature = "graphics")]
    fn ui_table(&mut self, a: &[Value]) -> R<Value> {
        const HDR: i32 = 22; const ROW: i32 = 20; const SCR: i32 = 12; const PAD: i32 = 6;
        // --- Arg-Extraktion (lokal) ---
        fn gi(a: &[Value], i: usize, f: &str) -> R<i64> {
            match a.get(i) { Some(Value::Int(n)) => Ok(*n), _ => Err(format!("{}: erwartet INTEGER (Arg {})", f, i + 1)) }
        }
        fn gid(a: &[Value], f: &str) -> R<String> {
            match a.first() { Some(Value::Str(s)) => Ok(s.to_string()), _ => Err(format!("{}: id muss STRING sein", f)) }
        }
        fn str1d(v: &Value, f: &str) -> R<Vec<String>> {
            match v { Value::Array(arr) => { let arr = arr.borrow();
                if arr.element_type != "string" || arr.dims.len() != 1 { return Err(format!("{}: headers muss 1D ARRAY OF STRING sein", f)); }
                Ok(arr.cells.iter().map(|x| match x { Value::Str(s) => s.to_string(), o => o.fmt() }).collect()) }
                _ => Err(format!("{}: headers muss ARRAY OF STRING sein", f)) }
        }
        fn str2d(v: &Value, ncols: usize, f: &str) -> R<Vec<Vec<String>>> {
            match v { Value::Array(arr) => { let arr = arr.borrow();
                if arr.element_type != "string" || arr.dims.len() != 2 { return Err(format!("{}: cells muss 2D ARRAY OF STRING sein", f)); }
                let (r, c) = (arr.dims[0] as usize, arr.dims[1] as usize);
                if c != ncols { return Err(format!("{}: cells hat {} Spalten, headers {}", f, c, ncols)); }
                Ok((0..r).map(|ri| (0..c).map(|ci| match &arr.cells.get(ri * c + ci) { Value::Str(s) => s.to_string(), o => o.fmt() }).collect()).collect()) }
                _ => Err(format!("{}: cells muss 2D ARRAY OF STRING sein", f)) }
        }
        // Optionales 2D INTEGER-Array -> flach (row-major), Validierung dims.
        fn int2d_opt(a: &[Value], i: usize, nr: usize, nc: usize, f: &str) -> R<Option<Vec<i64>>> {
            match a.get(i) { None | Some(Value::Nil) => Ok(None),
                Some(Value::Array(arr)) => { let arr = arr.borrow();
                    if arr.element_type != "integer" || arr.dims.len() != 2 || arr.dims[0] as usize != nr || arr.dims[1] as usize != nc {
                        return Err(format!("{}: erwartet 2D ARRAY OF INTEGER [{}, {}]", f, nr, nc)); }
                    Ok(Some(arr.cells.iter().map(|x| match x { Value::Int(n) => n, _ => 0 }).collect())) }
                _ => Err(format!("{}: erwartet 2D ARRAY OF INTEGER", f)) }
        }
        fn int1d_opt(a: &[Value], i: usize, n: usize, f: &str) -> R<Option<Vec<i32>>> {
            match a.get(i) { None | Some(Value::Nil) => Ok(None),
                Some(Value::Array(arr)) => { let arr = arr.borrow();
                    if arr.element_type != "integer" || arr.dims.len() != 1 || arr.dims[0] as usize != n {
                        return Err(format!("{}: col_widths muss 1D ARRAY OF INTEGER ({}) sein", f, n)); }
                    Ok(Some(arr.cells.iter().map(|x| match x { Value::Int(v) => v as i32, _ => 0 }).collect())) }
                _ => Err(format!("{}: col_widths muss ARRAY OF INTEGER sein", f)) }
        }
        let in_box = |mx: i32, my: i32, x: i32, y: i32, w: i32, h: i32| mx >= x && mx < x + w && my >= y && my < y + h;

        let id = gid(a, "UI_TABLE")?;
        let mut x = gi(a, 1, "UI_TABLE")? as i32 + self.ui_state.offset_x;
        let mut y = gi(a, 2, "UI_TABLE")? as i32 + self.ui_state.offset_y;
        let w = gi(a, 3, "UI_TABLE")? as i32;
        let h = gi(a, 4, "UI_TABLE")? as i32;
        let headers = str1d(a.get(5).ok_or("UI_TABLE: headers fehlt")?, "UI_TABLE")?;
        let n_cols = headers.len();
        if n_cols == 0 { return Err("UI_TABLE: headers darf nicht leer sein".into()); }
        let cells = str2d(a.get(6).ok_or("UI_TABLE: cells fehlt")?, n_cols, "UI_TABLE")?;
        let n_rows = cells.len();
        let cell_colors = int2d_opt(a, 7, n_rows, n_cols, "UI_TABLE")?;
        let col_widths: Vec<i32> = match int1d_opt(a, 8, n_cols, "UI_TABLE")? {
            Some(cw) => cw,
            None => { let avail = w - SCR - 2; let per = (avail / n_cols as i32).max(40); vec![per; n_cols] }
        };
        let cell_bg = int2d_opt(a, 9, n_rows, n_cols, "UI_TABLE")?;
        let _ = (&mut x, &mut y);

        // --- State init ---
        let st = self.ui_state.tables.entry(id.clone()).or_insert_with(|| UiTable { selected: -1, ..Default::default() });
        if st.selected >= n_rows as i32 { st.selected = -1; }
        st.header_col = -1;

        // --- Geometrie ---
        let body_x = x + 1; let body_y = y + HDR;
        let body_w_raw = w - 2; let body_h_raw = h - HDR - 1;
        let total_w: i32 = col_widths.iter().sum();
        let total_h = n_rows as i32 * ROW;
        let mut need_v = total_h > body_h_raw;
        let need_h = total_w > body_w_raw - if need_v { SCR } else { 0 };
        if need_h && total_h > body_h_raw - SCR { need_v = true; }
        let body_w = body_w_raw - if need_v { SCR } else { 0 };
        let body_h = body_h_raw - if need_h { SCR } else { 0 };
        let max_sy = (total_h - body_h).max(0);
        let max_sx = (total_w - body_w).max(0);

        // --- Maus / Wheel ---
        let (mx, my, down) = self.ui_mouse_gated()?;
        let wheel = { let g = self.gfx.as_ref().ok_or("UI_TABLE: vor SCREEN")?; g.pop_mouse_wheel() };
        let was_down = self.ui_state.was_mouse_down;
        let just_pressed = down && !was_down;
        let just_released = !down && was_down;
        let over_table = in_box(mx, my, x, y, w, h);

        let st = self.ui_state.tables.get_mut(&id).unwrap();
        st.scroll_y = st.scroll_y.clamp(0, max_sy);
        st.scroll_x = st.scroll_x.clamp(0, max_sx);
        if over_table && wheel != 0 {
            st.scroll_y = (st.scroll_y - wheel as i32 * ROW).clamp(0, max_sy);
        }
        // Vertikale Scrollbar
        if need_v {
            let sb_x = x + body_w_raw - SCR + 1; let sb_y = body_y; let sb_h = body_h;
            let handle_h = ((sb_h as f64 * (body_h as f64 / total_h as f64)) as i32).max(16);
            if just_pressed && in_box(mx, my, sb_x, sb_y, SCR, sb_h) {
                let ratio = if max_sy > 0 { st.scroll_y as f64 / max_sy as f64 } else { 0.0 };
                let handle_y = sb_y + ((sb_h - handle_h) as f64 * ratio) as i32;
                if in_box(mx, my, sb_x, handle_y, SCR, handle_h) {
                    st.drag_v = true; st.drag_off = my - handle_y;
                } else {
                    let hy = (my - handle_h / 2).clamp(sb_y, sb_y + sb_h - handle_h);
                    st.drag_v = true; st.drag_off = handle_h / 2;
                    let tp = (hy - sb_y) as f64 / (sb_h - handle_h).max(1) as f64;
                    st.scroll_y = (tp * max_sy as f64) as i32;
                }
            }
            if st.drag_v && down {
                let nh = (my - st.drag_off).clamp(sb_y, sb_y + sb_h - handle_h);
                let tp = (nh - sb_y) as f64 / (sb_h - handle_h).max(1) as f64;
                st.scroll_y = (tp * max_sy as f64) as i32;
            }
            if just_released { st.drag_v = false; }
        } else { st.drag_v = false; }
        // Horizontale Scrollbar
        if need_h {
            let hs_x = body_x; let hs_y = y + h - SCR - 1; let hs_w = body_w;
            let handle_w = ((hs_w as f64 * (body_w as f64 / total_w as f64)) as i32).max(16);
            if just_pressed && in_box(mx, my, hs_x, hs_y, hs_w, SCR) {
                let ratio = if max_sx > 0 { st.scroll_x as f64 / max_sx as f64 } else { 0.0 };
                let handle_x = hs_x + ((hs_w - handle_w) as f64 * ratio) as i32;
                if in_box(mx, my, handle_x, hs_y, handle_w, SCR) {
                    st.drag_h = true; st.drag_off = mx - handle_x;
                } else {
                    let hx = (mx - handle_w / 2).clamp(hs_x, hs_x + hs_w - handle_w);
                    st.drag_h = true; st.drag_off = handle_w / 2;
                    let tp = (hx - hs_x) as f64 / (hs_w - handle_w).max(1) as f64;
                    st.scroll_x = (tp * max_sx as f64) as i32;
                }
            }
            if st.drag_h && down {
                let nx = (mx - st.drag_off).clamp(hs_x, hs_x + hs_w - handle_w);
                let tp = (nx - hs_x) as f64 / (hs_w - handle_w).max(1) as f64;
                st.scroll_x = (tp * max_sx as f64) as i32;
            }
            if just_released { st.drag_h = false; }
        } else { st.drag_h = false; }

        let (scroll_x, scroll_y, drag_v, drag_h) = (st.scroll_x, st.scroll_y, st.drag_v, st.drag_h);
        // Hover-Zeile (ueber Body, nicht ueber Scrollbalken)
        let hover_row = if over_table && in_box(mx, my, body_x, body_y, body_w, body_h) && !drag_v && !drag_h {
            let hr = (my - body_y + scroll_y) / ROW;
            if hr >= 0 && hr < n_rows as i32 { hr } else { -1 }
        } else { -1 };

        // --- Klick-Erkennung Zeile (Press+Release auf derselben) ---
        let mut clicked_row = -1;
        let sb_block = over_table && in_box(mx, my, x + body_w_raw - SCR + 1, body_y, SCR, body_h);
        if over_table && just_pressed && hover_row >= 0 && !sb_block {
            self.ui_state.click_origin = Some(format!("{}#row#{}", id, hover_row));
        }
        if over_table && just_released && hover_row >= 0 {
            if self.ui_state.click_origin.as_deref() == Some(format!("{}#row#{}", id, hover_row).as_str()) {
                clicked_row = hover_row;
            }
        }
        if clicked_row >= 0 { self.ui_state.tables.get_mut(&id).unwrap().selected = clicked_row; }

        // --- Header-Klick (fuer Sortierung) ---
        let header_col_at = |px: i32, py: i32| -> i32 {
            if !in_box(px, py, x, y, body_w_raw, HDR) { return -1; }
            let rel = px - (body_x - scroll_x);
            let mut acc = 0;
            for c in 0..n_cols { if acc <= rel && rel < acc + col_widths[c] { return c as i32; } acc += col_widths[c]; }
            -1
        };
        if over_table && just_pressed {
            let hc = header_col_at(mx, my);
            if hc >= 0 { self.ui_state.click_origin = Some(format!("{}#hdr#{}", id, hc)); }
        }
        if over_table && just_released {
            if let Some(co) = self.ui_state.click_origin.clone() {
                if let Some(rest) = co.strip_prefix(&format!("{}#hdr#", id)) {
                    if let Ok(oc) = rest.parse::<i32>() {
                        if oc == header_col_at(mx, my) { self.ui_state.tables.get_mut(&id).unwrap().header_col = oc; }
                    }
                }
            }
        }
        let selected = self.ui_state.tables[&id].selected;

        // --- Zeichnen ---
        let g = self.gfx.as_mut().ok_or("UI_TABLE: vor SCREEN")?;
        g.box_fill(x, y, x + w - 1, y + h - 1, 0x1A1C2A);
        g.rect(x, y, x + w - 1, y + h - 1, 0x60607A);
        g.box_fill(x + 1, y + 1, x + w - 2, y + HDR - 1, 0x383C5C);
        g.rect(x, y, x + w - 1, y + HDR - 1, 0x60607A);
        // Kopf-Zellen
        g.push_clip(x + 1, y + 1, body_w_raw, HDR - 2);
        let mut cx = body_x - scroll_x;
        for c in 0..n_cols {
            if cx + col_widths[c] > x + 1 && cx < x + 1 + body_w_raw {
                g.text(cx + PAD, y + 4, headers[c].clone(), 0xFFFFFF);
                if c < n_cols - 1 { g.line(cx + col_widths[c], y + 1, cx + col_widths[c], y + HDR - 2, 0x60607A); }
            }
            cx += col_widths[c];
        }
        g.pop_clip();
        // Body
        g.push_clip(body_x, body_y, body_w, body_h);
        let first = (scroll_y / ROW).max(0);
        let last = ((scroll_y + body_h) / ROW + 1).min(n_rows as i32);
        for r in first..last {
            let row_y = body_y + r * ROW - scroll_y;
            // Pass 1: Zell-Hintergruende
            if let Some(ref bg) = cell_bg {
                let mut bx = body_x - scroll_x;
                for c in 0..n_cols {
                    let cw = col_widths[c];
                    if bx + cw > body_x && bx < body_x + body_w {
                        let v = bg[r as usize * n_cols + c];
                        if v >= 0 { g.box_fill(bx, row_y, bx + cw - 1, row_y + ROW - 1, v); }
                    }
                    bx += cw;
                }
            }
            if r == selected { g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + ROW - 1, 0x2A4E6A); }
            if r == hover_row { g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + ROW - 1, 0x2A2E4A); }
            let mut cx = body_x - scroll_x;
            let row = &cells[r as usize];
            for c in 0..n_cols {
                let cw = col_widths[c];
                if cx + cw > body_x && cx < body_x + body_w {
                    let clip_x = (cx + 1).max(body_x);
                    let clip_w = (cw - 2).min((body_x + body_w) - clip_x);
                    g.push_clip(clip_x, row_y, clip_w, ROW);
                    let color = match &cell_colors { Some(cc) => cc[r as usize * n_cols + c], None => 0xFFFFFF };
                    g.text(cx + PAD, row_y + 3, row[c].clone(), color);
                    g.pop_clip();
                }
                cx += cw;
            }
            g.line(body_x, row_y + ROW - 1, body_x + body_w - 1, row_y + ROW - 1, 0x2A2E4A);
        }
        g.pop_clip();
        // Scrollbalken
        if need_v {
            let sb_x = x + body_w_raw - SCR + 1; let sb_h = body_h;
            let handle_h = ((sb_h as f64 * (body_h as f64 / total_h as f64)) as i32).max(16);
            let ratio = if max_sy > 0 { scroll_y as f64 / max_sy as f64 } else { 0.0 };
            let hy = body_y + ((sb_h - handle_h) as f64 * ratio) as i32;
            g.box_fill(sb_x, body_y, sb_x + SCR - 1, body_y + sb_h - 1, 0x252840);
            g.box_fill(sb_x + 2, hy, sb_x + SCR - 3, hy + handle_h - 1, if drag_v { 0x80C0FF } else { 0x60607A });
        }
        if need_h {
            let hs_y = y + h - SCR - 1; let hs_w = body_w;
            let handle_w = ((hs_w as f64 * (body_w as f64 / total_w as f64)) as i32).max(16);
            let ratio = if max_sx > 0 { scroll_x as f64 / max_sx as f64 } else { 0.0 };
            let hx = body_x + ((hs_w - handle_w) as f64 * ratio) as i32;
            g.box_fill(body_x, hs_y, body_x + hs_w - 1, hs_y + SCR - 1, 0x252840);
            g.box_fill(hx, hs_y + 2, hx + handle_w - 1, hs_y + SCR - 3, if drag_h { 0x80C0FF } else { 0x60607A });
        }
        Ok(Value::Int(clicked_row as i64))
    }

    #[cfg(feature = "graphics")]
    fn try_graphics(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        // Grafik-Koordinaten sind "intish": INTEGER direkt, FLOAT wird zu int
        // trunkiert (Richtung 0, wie Pythons int()). Der Tree-Walker macht das
        // gleiche -- seine Zeichenprimitive (PLOT/LINE/BOX/RECT/CIRCLE/GRADIENT*)
        // pruefen via `_check_int`, das in interpreter.py ein Alias auf
        // `_check_intish` ist (interpreter.py:3541). So sind F5 (Tree-Walker) und
        // F6 (gbrt) bei Float-Koordinaten konsistent (z.B. LINE(10.5, ...)).
        fn gi(a: &[Value], i: usize, fn_: &str) -> R<i64> {
            match a.get(i) {
                Some(Value::Int(n)) => Ok(*n),
                Some(Value::Float(f)) => Ok(*f as i64),
                Some(v) => Err(format!("{}: erwartet Zahl, erhalten {}", fn_, v.type_name())),
                None => Err(format!("{}: fehlendes Argument {}", fn_, i + 1)),
            }
        }
        fn gs<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) {
                Some(Value::Str(s)) => Ok(s),
                _ => Err(format!("{}: erwartet STRING", fn_)),
            }
        }
        fn gb(a: &[Value], i: usize) -> bool {
            matches!(a.get(i), Some(Value::Bool(true)))
        }
        fn need_f(a: &[Value], i: usize, fn_: &str) -> R<f64> {
            match a.get(i) {
                Some(Value::Int(n)) => Ok(*n as f64),
                Some(Value::Float(f)) => Ok(*f),
                _ => Err(format!("{}: Argument {} muss Zahl sein", fn_, i + 1)),
            }
        }
        // 1D-INTEGER-Array -> Vec<i32> (fuer Bulk-Draws).
        fn arr_i32(v: &Value, fn_: &str) -> R<Vec<i32>> {
            match v {
                Value::Array(a) => {
                    let a = a.borrow();
                    if let Some(ints) = a.cells.as_ints() {
                        return Ok(ints.iter().map(|&i| i as i32).collect());
                    }
                    let mut o = Vec::with_capacity(a.cells.len());
                    for x in a.cells.iter() {
                        match x { Value::Int(i) => o.push(i as i32), _ => return Err(format!("{}: ARRAY OF INTEGER noetig", fn_)) }
                    }
                    Ok(o)
                }
                _ => Err(format!("{}: ARRAY noetig", fn_)),
            }
        }
        // color-Arg fuer Bulk: INTEGER (alle gleich) oder ARRAY OF INTEGER.
        fn bulk_color(v: &Value, n: usize, fn_: &str) -> R<Vec<i64>> {
            match v {
                Value::Int(c) => Ok(vec![*c; n]),
                Value::Array(a) => {
                    let a = a.borrow();
                    if a.cells.len() != n { return Err(format!("{}: colors-Array muss so lang wie Koordinaten sein", fn_)); }
                    let mut o = Vec::with_capacity(n);
                    if let Some(ints) = a.cells.as_ints() { return Ok(ints.to_vec()); }
                    for x in a.cells.iter() { match x { Value::Int(i) => o.push(i), _ => return Err(format!("{}: color-ARRAY OF INTEGER noetig", fn_)) } }
                    Ok(o)
                }
                _ => Err(format!("{}: color muss INTEGER oder ARRAY sein", fn_)),
            }
        }
        // Grafik-Builtins lazy-initialisieren ein verstecktes Fenster, wenn noch
        // kein SCREEN aufgerufen wurde -- so funktionieren headless Bild-/Kamera-/
        // Sprite-Ops (LOADIMAGE, imgfx, CAMERA_*, SPRITE_*) ohne sichtbares Fenster,
        // wie das pygame-Lazy-Init im Tree-Walker. Ein spaeteres SCREEN macht das
        // Fenster sichtbar (reconfigure), statt ein zweites zu erzeugen.
        macro_rules! g {
            () => {{
                if self.gfx.is_none() {
                    self.gfx = Some(crate::graphics::Graphics::new_headless());
                }
                self.gfx.as_mut().unwrap()
            }};
        }
        let r = match name {
            "screen" => {
                let w = gi(a, 0, "SCREEN")? as i32;
                let h = gi(a, 1, "SCREEN")? as i32;
                let title = if a.len() >= 3 { gs(a, 2, "SCREEN")?.to_string() } else { "GameBasic".to_string() };
                let scale = if a.len() >= 4 { gi(a, 3, "SCREEN")? as i32 } else { 1 };
                if scale < 1 { return Err("SCREEN: skala muss >= 1 sein".into()); }
                match self.gfx.as_mut() {
                    Some(gfx) => gfx.reconfigure(w, h, &title, scale),
                    None => self.gfx = Some(crate::graphics::Graphics::new(w, h, &title, scale)),
                }
                Value::Nil
            }
            "cls" => { let c = if a.is_empty() { 0 } else { gi(a, 0, "CLS")? }; g!().cls(c); Value::Nil }
            "plot" => {
                let c = if a.len() == 3 { gi(a, 2, "PLOT")? } else { 0xFFFFFF };
                g!().plot(gi(a, 0, "PLOT")? as i32, gi(a, 1, "PLOT")? as i32, c); Value::Nil
            }
            "line" => {
                let c = if a.len() == 5 { gi(a, 4, "LINE")? } else { 0xFFFFFF };
                g!().line(gi(a,0,"LINE")? as i32, gi(a,1,"LINE")? as i32, gi(a,2,"LINE")? as i32, gi(a,3,"LINE")? as i32, c); Value::Nil
            }
            "box" => {
                let c = if a.len() == 5 { gi(a, 4, "BOX")? } else { 0xFFFFFF };
                g!().box_fill(gi(a,0,"BOX")? as i32, gi(a,1,"BOX")? as i32, gi(a,2,"BOX")? as i32, gi(a,3,"BOX")? as i32, c); Value::Nil
            }
            "rect" => {
                let c = if a.len() == 5 { gi(a, 4, "RECT")? } else { 0xFFFFFF };
                g!().rect(gi(a,0,"RECT")? as i32, gi(a,1,"RECT")? as i32, gi(a,2,"RECT")? as i32, gi(a,3,"RECT")? as i32, c); Value::Nil
            }
            "circle" => {
                let c = if a.len() == 4 { gi(a, 3, "CIRCLE")? } else { 0xFFFFFF };
                g!().circle(gi(a,0,"CIRCLE")? as i32, gi(a,1,"CIRCLE")? as i32, gi(a,2,"CIRCLE")? as i32, c); Value::Nil
            }
            "linew" => {
                let c = if a.len() == 6 { gi(a, 5, "LINEW")? } else { 0xFFFFFF };
                g!().line_thick(gi(a,0,"LINEW")? as i32, gi(a,1,"LINEW")? as i32, gi(a,2,"LINEW")? as i32,
                    gi(a,3,"LINEW")? as i32, need_f(a,4,"LINEW")?, c); Value::Nil
            }
            "boxround" => {
                let c = if a.len() == 6 { gi(a, 5, "BOXROUND")? } else { 0xFFFFFF };
                g!().round_rect(gi(a,0,"BOXROUND")? as i32, gi(a,1,"BOXROUND")? as i32, gi(a,2,"BOXROUND")? as i32,
                    gi(a,3,"BOXROUND")? as i32, gi(a,4,"BOXROUND")? as i32, c, true); Value::Nil
            }
            "rectround" => {
                let c = if a.len() == 6 { gi(a, 5, "RECTROUND")? } else { 0xFFFFFF };
                g!().round_rect(gi(a,0,"RECTROUND")? as i32, gi(a,1,"RECTROUND")? as i32, gi(a,2,"RECTROUND")? as i32,
                    gi(a,3,"RECTROUND")? as i32, gi(a,4,"RECTROUND")? as i32, c, false); Value::Nil
            }
            "gradientv" => {
                g!().gradient_rect(gi(a,0,"GRADIENTV")? as i32, gi(a,1,"GRADIENTV")? as i32, gi(a,2,"GRADIENTV")? as i32,
                    gi(a,3,"GRADIENTV")? as i32, gi(a,4,"GRADIENTV")?, gi(a,5,"GRADIENTV")?, true); Value::Nil
            }
            "gradienth" => {
                g!().gradient_rect(gi(a,0,"GRADIENTH")? as i32, gi(a,1,"GRADIENTH")? as i32, gi(a,2,"GRADIENTH")? as i32,
                    gi(a,3,"GRADIENTH")? as i32, gi(a,4,"GRADIENTH")?, gi(a,5,"GRADIENTH")?, false); Value::Nil
            }
            "spline" => {
                let xs = arr_i32(&a[0], "SPLINE")?;
                let ys = arr_i32(&a[1], "SPLINE")?;
                if xs.len() != ys.len() { return Err("SPLINE: xs und ys muessen gleich lang sein".into()); }
                let w = if a.len() >= 4 { need_f(a, 3, "SPLINE")? } else { 1.0 };
                let c = if a.len() >= 3 { gi(a, 2, "SPLINE")? } else { 0xFFFFFF };
                g!().spline(&xs, &ys, w, c); Value::Nil
            }
            // --- Blend-Modes (Batch 2) ---
            "blend_mode" => {
                let s = gs(a, 0, "BLEND_MODE")?.to_lowercase();
                let m = match s.as_str() {
                    "alpha" | "none" | "normal" => 0,
                    "add" | "additive" => 1,
                    "mult" | "multiply" | "multiplied" => 2,
                    "subtract" | "sub" => 4,
                    _ => return Err(format!("BLEND_MODE: unbekannter Modus '{}' (alpha/add/mult/subtract)", s)),
                };
                g!().blend_mode(m); Value::Nil
            }
            // --- Prozedurale Texturen (Batch 3) -> IMAGE-Handle ---
            "gentex_perlin" => Value::Int(g!().gen_tex_perlin(
                gi(a,0,"GENTEX_PERLIN")? as i32, gi(a,1,"GENTEX_PERLIN")? as i32, need_f(a,2,"GENTEX_PERLIN")?)?),
            "gentex_gradient" => Value::Int(g!().gen_tex_gradient(
                gi(a,0,"GENTEX_GRADIENT")? as i32, gi(a,1,"GENTEX_GRADIENT")? as i32,
                gi(a,2,"GENTEX_GRADIENT")?, gi(a,3,"GENTEX_GRADIENT")?, gb(a,4))?),
            "gentex_checked" => Value::Int(g!().gen_tex_checked(
                gi(a,0,"GENTEX_CHECKED")? as i32, gi(a,1,"GENTEX_CHECKED")? as i32,
                gi(a,2,"GENTEX_CHECKED")? as i32, gi(a,3,"GENTEX_CHECKED")? as i32,
                gi(a,4,"GENTEX_CHECKED")?, gi(a,5,"GENTEX_CHECKED")?)?),
            "gentex_color" => Value::Int(g!().gen_tex_color(
                gi(a,0,"GENTEX_COLOR")? as i32, gi(a,1,"GENTEX_COLOR")? as i32, gi(a,2,"GENTEX_COLOR")?)?),
            "gentex_radial" => Value::Int(g!().gen_tex_radial(
                gi(a,0,"GENTEX_RADIAL")? as i32, gi(a,1,"GENTEX_RADIAL")? as i32,
                gi(a,2,"GENTEX_RADIAL")?, gi(a,3,"GENTEX_RADIAL")?,
                if a.len() >= 5 { need_f(a,4,"GENTEX_RADIAL")? } else { 0.0 })?),
            // --- Clipboard + Drag&Drop (Batch 5) ---
            "clipboard_get" => Value::Str(g!().clipboard_get().into()),
            "clipboard_set" => { let s = gs(a,0,"CLIPBOARD_SET")?.to_string(); g!().clipboard_set(&s); Value::Nil }
            "files_dropped" => Value::Int(g!().dropped_files().len() as i64),
            "file_dropped" => {
                let i = gi(a, 0, "FILE_DROPPED")? as usize;
                Value::Str(g!().dropped_files().get(i).cloned().unwrap_or_default().into())
            }
            // --- Render-Targets (Batch 4) ---
            "rendertarget_new" => Value::Int(g!().rendertarget_new(
                gi(a,0,"RENDERTARGET_NEW")? as i32, gi(a,1,"RENDERTARGET_NEW")? as i32)?),
            "rendertarget_begin" => { g!().rendertarget_begin(gi(a,0,"RENDERTARGET_BEGIN")?)?; Value::Nil }
            "rendertarget_end" => { g!().rendertarget_end(); Value::Nil }
            "rendertarget_draw" => {
                let scale = if a.len() >= 4 { need_f(a,3,"RENDERTARGET_DRAW")? } else { 1.0 };
                let tint = if a.len() >= 5 { Some(gi(a,4,"RENDERTARGET_DRAW")?) } else { None };
                g!().rendertarget_draw(gi(a,0,"RENDERTARGET_DRAW")?,
                    gi(a,1,"RENDERTARGET_DRAW")? as i32, gi(a,2,"RENDERTARGET_DRAW")? as i32,
                    scale, tint)?; Value::Nil
            }
            "triangle" => {
                let c = if a.len() == 7 { gi(a, 6, "TRIANGLE")? } else { 0xFFFFFF };
                g!().triangle(gi(a,0,"TRIANGLE")? as i32, gi(a,1,"TRIANGLE")? as i32, gi(a,2,"TRIANGLE")? as i32,
                    gi(a,3,"TRIANGLE")? as i32, gi(a,4,"TRIANGLE")? as i32, gi(a,5,"TRIANGLE")? as i32, c); Value::Nil
            }
            "triangleoutline" => {
                let c = if a.len() >= 7 { gi(a, 6, "TRIANGLEOUTLINE")? } else { 0xFFFFFF };
                g!().triangle_outline(gi(a,0,"TRIANGLEOUTLINE")? as i32, gi(a,1,"TRIANGLEOUTLINE")? as i32,
                    gi(a,2,"TRIANGLEOUTLINE")? as i32, gi(a,3,"TRIANGLEOUTLINE")? as i32,
                    gi(a,4,"TRIANGLEOUTLINE")? as i32, gi(a,5,"TRIANGLEOUTLINE")? as i32, c); Value::Nil
            }
            "ellipse" => {
                let c = if a.len() == 5 { gi(a, 4, "ELLIPSE")? } else { 0xFFFFFF };
                g!().ellipse(gi(a,0,"ELLIPSE")? as i32, gi(a,1,"ELLIPSE")? as i32, gi(a,2,"ELLIPSE")? as i32, gi(a,3,"ELLIPSE")? as i32, c); Value::Nil
            }
            "ellipseoutline" => {
                let c = if a.len() >= 5 { gi(a, 4, "ELLIPSEOUTLINE")? } else { 0xFFFFFF };
                g!().ellipse_outline(gi(a,0,"ELLIPSEOUTLINE")? as i32, gi(a,1,"ELLIPSEOUTLINE")? as i32, gi(a,2,"ELLIPSEOUTLINE")? as i32, gi(a,3,"ELLIPSEOUTLINE")? as i32, c); Value::Nil
            }
            "arc" => {
                let start = match a.get(4) { Some(Value::Int(n)) => *n as f64, Some(Value::Float(f)) => *f, _ => return Err("ARC: start muss Zahl sein".into()) };
                let end = match a.get(5) { Some(Value::Int(n)) => *n as f64, Some(Value::Float(f)) => *f, _ => return Err("ARC: end muss Zahl sein".into()) };
                let c = if a.len() >= 7 { gi(a, 6, "ARC")? } else { 0xFFFFFF };
                g!().arc(gi(a,0,"ARC")? as i32, gi(a,1,"ARC")? as i32, gi(a,2,"ARC")? as i32, gi(a,3,"ARC")? as i32, start, end, c); Value::Nil
            }
            "polygon" | "polygonoutline" => {
                let filled = name == "polygon";
                let pts: Vec<i32> = match a.get(0) {
                    Some(Value::Array(arr)) => {
                        let arr = arr.borrow();
                        let mut v = Vec::with_capacity(arr.cells.len());
                        for x in arr.cells.iter() { match x { Value::Int(i) => v.push(i as i32), _ => return Err("POLYGON: Punkte muessen INTEGER sein".into()) } }
                        v
                    }
                    _ => return Err("POLYGON: Punkte muessen ein ARRAY OF INTEGER sein".into()),
                };
                let c = if a.len() >= 2 { gi(a, 1, "POLYGON")? } else { 0xFFFFFF };
                g!().polygon(&pts, c, filled)?; Value::Nil
            }
            "text" => {
                let c = if a.len() == 4 { gi(a, 3, "TEXT")? } else { 0xFFFFFF };
                let s = match a.get(2) { Some(Value::Str(s)) => s.to_string(), Some(v) => v.fmt(), None => String::new() };
                g!().text(gi(a,0,"TEXT")? as i32, gi(a,1,"TEXT")? as i32, s, c); Value::Nil
            }
            "textrot" => {
                // TEXTROT(x, y, s$, winkel[, skala[, farbe]]) -- zentriert auf (x,y)
                let x = gi(a, 0, "TEXTROT")? as i32;
                let y = gi(a, 1, "TEXTROT")? as i32;
                let s = match a.get(2) { Some(Value::Str(s)) => s.to_string(), Some(v) => v.fmt(), None => String::new() };
                let ang = need_f(a, 3, "TEXTROT")? as f32;
                let scl = if a.len() >= 5 { need_f(a, 4, "TEXTROT")? } else { 1.0 };
                if scl <= 0.0 { return Err("TEXTROT: skala muss > 0 sein".into()); }
                let c = if a.len() >= 6 { gi(a, 5, "TEXTROT")? } else { 0xFFFFFF };
                g!().text_rot(x, y, s, ang, scl as f32, c); Value::Nil
            }
            "text_size" => { g!().set_text_size(gi(a,0,"TEXT_SIZE")? as i32); Value::Nil }
            "text_width" => Value::Int(g!().text_width(gs(a,0,"TEXT_WIDTH")?) as i64),
            "text_height" => Value::Int(g!().text_height() as i64),
            "loadfont" => Value::Int(g!().load_font(gs(a,0,"LOADFONT")?, gi(a,1,"LOADFONT")? as i32)?),
            "setfont" => { g!().set_font(gi(a,0,"SETFONT")?)?; Value::Nil }
            "text_spacing" => { g!().set_text_spacing(gi(a,0,"TEXT_SPACING")? as i32); Value::Nil }
            "loadimage" => Value::Int(g!().load_texture(gs(a,0,"LOADIMAGE")?)?),
            "drawimage" => {
                let idx = gi(a, 0, "DRAWIMAGE")?;
                g!().draw_image(idx, gi(a,1,"DRAWIMAGE")? as i32, gi(a,2,"DRAWIMAGE")? as i32)?; Value::Nil
            }
            "drawimagerot" => {
                // DRAWIMAGEROT(img, x, y, winkel_grad [, skala [, tint]]) -- zentriert + gedreht.
                let idx = gi(a, 0, "DRAWIMAGEROT")?;
                let x = gi(a, 1, "DRAWIMAGEROT")? as i32;
                let y = gi(a, 2, "DRAWIMAGEROT")? as i32;
                let ang = need_f(a, 3, "DRAWIMAGEROT")? as f32;
                let scale = if a.len() >= 5 { need_f(a, 4, "DRAWIMAGEROT")? as f32 } else { 1.0 };
                let tint = if a.len() >= 6 { Some(gi(a, 5, "DRAWIMAGEROT")?) } else { None };
                g!().draw_image_rot(idx, x, y, ang, scale, tint)?; Value::Nil
            }
            "imagewidth" => Value::Int(g!().image_width(gi(a,0,"IMAGEWIDTH")?)?),
            "imageheight" => Value::Int(g!().image_height(gi(a,0,"IMAGEHEIGHT")?)?),
            "flip" => {
                g!().flip();
                // Musik-Stream nachfuettern (sonst stockt die Wiedergabe).
                if let Some(au) = self.audio.as_mut() { au.update(); }
                Value::Nil
            }
            "keypressed" => Value::Bool(g!().key_down(gi(a,0,"KEYPRESSED")?)),
            "inkey$" | "inkey" => Value::Str(g!().inkey().into()),
            "waitkey" => Value::Int(g!().waitkey()),
            // SCROLL verschiebt persistente Framebuffer-Pixel -- die native
            // Runtime zeichnet jeden Frame neu aus dem Command-Buffer (Layer
            // werden pro FLIP geleert), es gibt also keine persistenten Pixel
            // zum Verschieben. Graceful No-Op (Programm laeuft); SCROLL ist
            // Tree-Walker-only (siehe docs/befehlssatz-roadmap.md).
            "scroll" => { let _ = (gi(a,0,"SCROLL")?, gi(a,1,"SCROLL")?); Value::Nil }
            "joystick_count" => Value::Int(g!().joystick_count()),
            "joystick_name" => Value::Str(g!().joystick_name(gi(a,0,"JOYSTICK_NAME")?)?.into()),
            "joystick_axis" => Value::Float(g!().joystick_axis(gi(a,0,"JOYSTICK_AXIS")?, gi(a,1,"JOYSTICK_AXIS")?)?),
            "joystick_button" => Value::Bool(g!().joystick_button(gi(a,0,"JOYSTICK_BUTTON")?, gi(a,1,"JOYSTICK_BUTTON")?)?),
            "joystick_hat_x" => Value::Int(g!().joystick_hat_x(gi(a,0,"JOYSTICK_HAT_X")?, gi(a,1,"JOYSTICK_HAT_X")?)?),
            "joystick_hat_y" => Value::Int(g!().joystick_hat_y(gi(a,0,"JOYSTICK_HAT_Y")?, gi(a,1,"JOYSTICK_HAT_Y")?)?),
            "quitrequested" => Value::Bool(g!().quit_requested()),
            "mousex" => Value::Int(g!().mouse_x()),
            "mousey" => Value::Int(g!().mouse_y()),
            "mousebutton" => Value::Bool(g!().mouse_button(gi(a,0,"MOUSEBUTTON")?)),
            // Graceful ohne SCREEN (0 / 0) -- wie der Tree-Walker (_buf_size=(0,0),
            // pop_mouse_wheel ohne Fenster = 0).
            "mousewheel" => Value::Int(self.gfx.as_ref().map(|g| g.pop_mouse_wheel()).unwrap_or(0)),
            "screenwidth" => Value::Int(self.gfx.as_ref().map(|g| g.screen_width()).unwrap_or(0)),
            "screenheight" => Value::Int(self.gfx.as_ref().map(|g| g.screen_height()).unwrap_or(0)),
            // raylib-Default-Font hat keine Bold/Italic-Variante -> No-Op
            // (visuelle Abweichung, Programm laeuft). Arg wird ignoriert.
            "text_bold" | "text_italic" => Value::Nil,
            "sleep" => {
                let ms = gi(a, 0, "SLEEP")?.max(0) as u64;
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Value::Nil
            }
            "set_fullscreen" => { g!().set_fullscreen(gb(a, 0)); Value::Nil }
            "mouse_visible" => { g!().mouse_visible(gb(a, 0)); Value::Nil }
            "mouse_lock" => { g!().mouse_lock(gb(a, 0)); Value::Nil }
            "mouse_hidden" => Value::Bool(g!().mouse_hidden()),
            "delta" => Value::Float(g!().delta()),
            "fps" => Value::Int(g!().fps()),
            "setfps" => { g!().set_target_fps(gi(a, 0, "SETFPS")?); Value::Nil }
            "setwindowtitle" => { g!().set_window_title(gs(a, 0, "SETWINDOWTITLE")?); Value::Nil }
            "savescreenshot" => { g!().save_screenshot(gs(a, 0, "SAVESCREENSHOT")?); Value::Nil }
            // Natives OS-Fenster (das SCREEN-Fenster) steuern.
            "window_resizable" => { g!().window_resizable(gb(a, 0)); Value::Nil }
            "window_min_size" => { g!().window_min_size(gi(a,0,"WINDOW_MIN_SIZE")? as i32, gi(a,1,"WINDOW_MIN_SIZE")? as i32); Value::Nil }
            "window_max_size" => { g!().window_max_size(gi(a,0,"WINDOW_MAX_SIZE")? as i32, gi(a,1,"WINDOW_MAX_SIZE")? as i32); Value::Nil }
            "window_maximize" => { g!().window_maximize(); Value::Nil }
            "window_minimize" => { g!().window_minimize(); Value::Nil }
            "window_restore" => { g!().window_restore(); Value::Nil }
            "window_resized" => Value::Bool(g!().window_resized()),

            // --- Shader / Post-Processing ---
            "shader_load" => {
                // SHADER_LOAD(quelle$): Datei-Pfad ODER GLSL-Quelltext.
                let arg = gs(a, 0, "SHADER_LOAD")?;
                let resolved = crate::builtins::resolve_asset_path(arg);
                let code = match std::fs::read_to_string(&resolved) {
                    Ok(text) => text,
                    Err(_) => arg.to_string(), // kein Pfad -> als Code behandeln
                };
                Value::Int(g!().load_shader(&code))
            }
            "shader_set" => {
                g!().shader_set_float(gi(a,0,"SHADER_SET")?, gs(a,1,"SHADER_SET")?, need_f(a,2,"SHADER_SET")?);
                Value::Nil
            }
            "shader_set2" => {
                g!().shader_set_vec2(gi(a,0,"SHADER_SET2")?, gs(a,1,"SHADER_SET2")?,
                    need_f(a,2,"SHADER_SET2")?, need_f(a,3,"SHADER_SET2")?);
                Value::Nil
            }
            "shader_set3" => {
                g!().shader_set_vec3(gi(a,0,"SHADER_SET3")?, gs(a,1,"SHADER_SET3")?,
                    need_f(a,2,"SHADER_SET3")?, need_f(a,3,"SHADER_SET3")?, need_f(a,4,"SHADER_SET3")?);
                Value::Nil
            }
            "postfx" => { g!().set_postfx(gi(a, 0, "POSTFX")?); Value::Nil }

            // --- 3D (Modul g3d) ---
            "camera3d" => {
                g!().set_camera3d(
                    need_f(a,0,"CAMERA3D")? as f32, need_f(a,1,"CAMERA3D")? as f32, need_f(a,2,"CAMERA3D")? as f32,
                    need_f(a,3,"CAMERA3D")? as f32, need_f(a,4,"CAMERA3D")? as f32, need_f(a,5,"CAMERA3D")? as f32,
                    need_f(a,6,"CAMERA3D")? as f32);
                Value::Nil
            }
            "cube" | "cube_wires" => {
                let wires = name == "cube_wires";
                let f = if wires { "CUBE_WIRES" } else { "CUBE" };
                g!().cube(need_f(a,0,f)? as f32, need_f(a,1,f)? as f32, need_f(a,2,f)? as f32,
                          need_f(a,3,f)? as f32, need_f(a,4,f)? as f32, need_f(a,5,f)? as f32,
                          gi(a,6,f)?, wires);
                Value::Nil
            }
            "sphere" | "sphere_wires" => {
                let wires = name == "sphere_wires";
                let f = if wires { "SPHERE_WIRES" } else { "SPHERE" };
                g!().sphere(need_f(a,0,f)? as f32, need_f(a,1,f)? as f32, need_f(a,2,f)? as f32,
                            need_f(a,3,f)? as f32, gi(a,4,f)?, wires);
                Value::Nil
            }
            "cylinder" => {
                g!().cylinder(need_f(a,0,"CYLINDER")? as f32, need_f(a,1,"CYLINDER")? as f32, need_f(a,2,"CYLINDER")? as f32,
                              need_f(a,3,"CYLINDER")? as f32, need_f(a,4,"CYLINDER")? as f32, need_f(a,5,"CYLINDER")? as f32,
                              gi(a,6,"CYLINDER")?);
                Value::Nil
            }
            "plane" => {
                g!().plane(need_f(a,0,"PLANE")? as f32, need_f(a,1,"PLANE")? as f32, need_f(a,2,"PLANE")? as f32,
                           need_f(a,3,"PLANE")? as f32, need_f(a,4,"PLANE")? as f32, gi(a,5,"PLANE")?);
                Value::Nil
            }
            "line3d" => {
                g!().line3d(need_f(a,0,"LINE3D")? as f32, need_f(a,1,"LINE3D")? as f32, need_f(a,2,"LINE3D")? as f32,
                            need_f(a,3,"LINE3D")? as f32, need_f(a,4,"LINE3D")? as f32, need_f(a,5,"LINE3D")? as f32,
                            gi(a,6,"LINE3D")?);
                Value::Nil
            }
            "point3d" => {
                g!().point3d(need_f(a,0,"POINT3D")? as f32, need_f(a,1,"POINT3D")? as f32, need_f(a,2,"POINT3D")? as f32,
                             gi(a,3,"POINT3D")?);
                Value::Nil
            }
            "grid3d" => {
                g!().grid3d(gi(a,0,"GRID3D")? as i32, need_f(a,1,"GRID3D")? as f32);
                Value::Nil
            }
            "loadmodel" => Value::Int(g!().load_model(gs(a,0,"LOADMODEL")?)?),
            "model_load_anims" => Value::Int(g!().load_model_anims(gs(a,0,"MODEL_LOAD_ANIMS")?)?),
            "model_anim_count" => Value::Int(g!().anim_count(gi(a,0,"MODEL_ANIM_COUNT")?)?),
            "model_anim_frames" => Value::Int(g!().anim_frames(gi(a,0,"MODEL_ANIM_FRAMES")?, gi(a,1,"MODEL_ANIM_FRAMES")?)?),
            "model_anim_name" => Value::str_rc(&g!().anim_name(gi(a,0,"MODEL_ANIM_NAME")?, gi(a,1,"MODEL_ANIM_NAME")?)?),
            "model_animate" => { g!().model_animate(gi(a,0,"MODEL_ANIMATE")?, gi(a,1,"MODEL_ANIMATE")?, gi(a,2,"MODEL_ANIMATE")?, gi(a,3,"MODEL_ANIMATE")? as i32)?; Value::Nil }
            "mesh_cube" => Value::Int(g!().mesh_cube(
                need_f(a,0,"MESH_CUBE")? as f32, need_f(a,1,"MESH_CUBE")? as f32, need_f(a,2,"MESH_CUBE")? as f32)?),
            "mesh_sphere" => Value::Int(g!().mesh_sphere(
                need_f(a,0,"MESH_SPHERE")? as f32, gi(a,1,"MESH_SPHERE")? as i32, gi(a,2,"MESH_SPHERE")? as i32)?),
            "mesh_cylinder" => Value::Int(g!().mesh_cylinder(
                need_f(a,0,"MESH_CYLINDER")? as f32, need_f(a,1,"MESH_CYLINDER")? as f32, gi(a,2,"MESH_CYLINDER")? as i32)?),
            "mesh_torus" => Value::Int(g!().mesh_torus(
                need_f(a,0,"MESH_TORUS")? as f32, need_f(a,1,"MESH_TORUS")? as f32, gi(a,2,"MESH_TORUS")? as i32, gi(a,3,"MESH_TORUS")? as i32)?),
            "mesh_knot" => Value::Int(g!().mesh_knot(
                need_f(a,0,"MESH_KNOT")? as f32, need_f(a,1,"MESH_KNOT")? as f32, gi(a,2,"MESH_KNOT")? as i32, gi(a,3,"MESH_KNOT")? as i32)?),
            "mesh_plane" => Value::Int(g!().mesh_plane(
                need_f(a,0,"MESH_PLANE")? as f32, need_f(a,1,"MESH_PLANE")? as f32, gi(a,2,"MESH_PLANE")? as i32, gi(a,3,"MESH_PLANE")? as i32)?),
            "mesh_heightmap" => Value::Int(g!().mesh_heightmap(
                gi(a,0,"MESH_HEIGHTMAP")?, need_f(a,1,"MESH_HEIGHTMAP")? as f32, need_f(a,2,"MESH_HEIGHTMAP")? as f32, need_f(a,3,"MESH_HEIGHTMAP")? as f32)?),
            "model" => {
                g!().draw_model(gi(a,0,"MODEL")?, need_f(a,1,"MODEL")? as f32, need_f(a,2,"MODEL")? as f32,
                                need_f(a,3,"MODEL")? as f32, need_f(a,4,"MODEL")? as f32, gi(a,5,"MODEL")?)?;
                Value::Nil
            }
            "model_ex" => {
                g!().draw_model_ex(gi(a,0,"MODEL_EX")?, need_f(a,1,"MODEL_EX")? as f32, need_f(a,2,"MODEL_EX")? as f32, need_f(a,3,"MODEL_EX")? as f32,
                                   need_f(a,4,"MODEL_EX")? as f32, need_f(a,5,"MODEL_EX")? as f32, need_f(a,6,"MODEL_EX")? as f32,
                                   need_f(a,7,"MODEL_EX")? as f32, need_f(a,8,"MODEL_EX")? as f32, gi(a,9,"MODEL_EX")?)?;
                Value::Nil
            }
            "model_wires" => {
                g!().draw_model_wires(gi(a,0,"MODEL_WIRES")?, need_f(a,1,"MODEL_WIRES")? as f32, need_f(a,2,"MODEL_WIRES")? as f32,
                                      need_f(a,3,"MODEL_WIRES")? as f32, need_f(a,4,"MODEL_WIRES")? as f32, gi(a,5,"MODEL_WIRES")?)?;
                Value::Nil
            }
            "model_matrix" => {
                // MODEL_MATRIX(handle, mat [, tint]) -- Welt-Transform aus m3d MAT4.
                let mat = match a.get(1) {
                    Some(Value::Mat4(m)) => m.clone(),
                    _ => return Err("MODEL_MATRIX: Arg 2 muss MAT4 sein".into()),
                };
                let tint = if a.len() >= 3 { gi(a, 2, "MODEL_MATRIX")? } else { 0xFF_FFFF };
                g!().draw_model_matrix(gi(a, 0, "MODEL_MATRIX")?, mat, tint)?;
                Value::Nil
            }
            "model_instanced" => {
                // MODEL_INSTANCED(handle, matrizen [, tint]) -- GPU-Instancing:
                // dasselbe Modell mit N Welt-Matrizen (ARRAY OF MAT4 / TUPLE von
                // MAT4) in EINEM Draw-Call. mat -> [f32;16] (column-major) sammeln.
                let collect_mats = |els: &mut dyn Iterator<Item = &Value>| -> R<Vec<[f32; 16]>> {
                    let mut v = Vec::new();
                    for (k, el) in els.enumerate() {
                        match el {
                            Value::Mat4(m) => v.push(**m),
                            other => return Err(format!(
                                "MODEL_INSTANCED: Element {} ist kein MAT4 (sondern {})",
                                k, other.type_name())),
                        }
                    }
                    Ok(v)
                };
                let mats = match a.get(1) {
                    Some(Value::Array(arr)) => {
                        let b = arr.borrow();
                        if b.dims.len() != 1 {
                            return Err("MODEL_INSTANCED: Arg 2 muss ein 1D-ARRAY OF MAT4 sein".into());
                        }
                        match b.cells.as_vals() {
                            Some(vals) => collect_mats(&mut vals.iter())?,
                            None => return Err("MODEL_INSTANCED: Arg 2 muss ein ARRAY OF MAT4 sein".into()),
                        }
                    }
                    Some(Value::Tuple(t)) => collect_mats(&mut t.iter())?,
                    _ => return Err("MODEL_INSTANCED: Arg 2 muss ARRAY OF MAT4 oder TUPLE von MAT4 sein".into()),
                };
                let tint = if a.len() >= 3 { gi(a, 2, "MODEL_INSTANCED")? } else { 0xFF_FFFF };
                g!().draw_model_instanced(gi(a, 0, "MODEL_INSTANCED")?, mats, tint)?;
                Value::Nil
            }
            "model_texture" => {
                g!().model_set_texture(gi(a,0,"MODEL_TEXTURE")?, gi(a,1,"MODEL_TEXTURE")?)?;
                Value::Nil
            }
            "model_texture_normal" => {
                g!().model_set_normal(gi(a,0,"MODEL_TEXTURE_NORMAL")?, gi(a,1,"MODEL_TEXTURE_NORMAL")?)?;
                Value::Nil
            }
            "model_pbr" => {
                g!().model_pbr(gi(a,0,"MODEL_PBR")?, need_f(a,1,"MODEL_PBR")?, need_f(a,2,"MODEL_PBR")?)?;
                Value::Nil
            }
            "model_emissive" => {
                g!().model_emissive(gi(a,0,"MODEL_EMISSIVE")?, gi(a,1,"MODEL_EMISSIVE")?, need_f(a,2,"MODEL_EMISSIVE")?)?;
                Value::Nil
            }
            "billboard" => {
                g!().billboard(gi(a,0,"BILLBOARD")?, need_f(a,1,"BILLBOARD")? as f32, need_f(a,2,"BILLBOARD")? as f32,
                               need_f(a,3,"BILLBOARD")? as f32, need_f(a,4,"BILLBOARD")? as f32, gi(a,5,"BILLBOARD")?)?;
                Value::Nil
            }
            "ray_hit_box" => Value::Float(g!().ray_hit_box(
                need_f(a,0,"RAY_HIT_BOX")? as f32, need_f(a,1,"RAY_HIT_BOX")? as f32, need_f(a,2,"RAY_HIT_BOX")? as f32,
                need_f(a,3,"RAY_HIT_BOX")? as f32, need_f(a,4,"RAY_HIT_BOX")? as f32, need_f(a,5,"RAY_HIT_BOX")? as f32,
                need_f(a,6,"RAY_HIT_BOX")? as f32, need_f(a,7,"RAY_HIT_BOX")? as f32, need_f(a,8,"RAY_HIT_BOX")? as f32,
                need_f(a,9,"RAY_HIT_BOX")? as f32, need_f(a,10,"RAY_HIT_BOX")? as f32, need_f(a,11,"RAY_HIT_BOX")? as f32)),
            "ray_hit_sphere" => Value::Float(g!().ray_hit_sphere(
                need_f(a,0,"RAY_HIT_SPHERE")? as f32, need_f(a,1,"RAY_HIT_SPHERE")? as f32, need_f(a,2,"RAY_HIT_SPHERE")? as f32,
                need_f(a,3,"RAY_HIT_SPHERE")? as f32, need_f(a,4,"RAY_HIT_SPHERE")? as f32, need_f(a,5,"RAY_HIT_SPHERE")? as f32,
                need_f(a,6,"RAY_HIT_SPHERE")? as f32, need_f(a,7,"RAY_HIT_SPHERE")? as f32, need_f(a,8,"RAY_HIT_SPHERE")? as f32,
                need_f(a,9,"RAY_HIT_SPHERE")? as f32)),
            "pick_box" => Value::Float(g!().pick_box(
                need_f(a,0,"PICK_BOX")? as f32, need_f(a,1,"PICK_BOX")? as f32, need_f(a,2,"PICK_BOX")? as f32,
                need_f(a,3,"PICK_BOX")? as f32, need_f(a,4,"PICK_BOX")? as f32, need_f(a,5,"PICK_BOX")? as f32)),
            "pick_sphere" => Value::Float(g!().pick_sphere(
                need_f(a,0,"PICK_SPHERE")? as f32, need_f(a,1,"PICK_SPHERE")? as f32, need_f(a,2,"PICK_SPHERE")? as f32,
                need_f(a,3,"PICK_SPHERE")? as f32)),
            "mouse_ground_x" => Value::Float(g!().mouse_ground(need_f(a,0,"MOUSE_GROUND_X")? as f32).0 as f64),
            "mouse_ground_z" => Value::Float(g!().mouse_ground(need_f(a,0,"MOUSE_GROUND_Z")? as f32).1 as f64),
            "mouse_ground_hit" => Value::Bool(g!().mouse_ground(need_f(a,0,"MOUSE_GROUND_HIT")? as f32).2),

            // --- Kamera-Modi (raylib UpdateCamera) ---
            "camera3d_update" => { g!().camera3d_update(gi(a,0,"CAMERA3D_UPDATE")?); Value::Nil }
            // m3d: View-/Projektions-Matrix-Override (Ortho, Custom-Frustum, Gizmos).
            // CAMERA3D(...) setzt beide zurueck auf Standard-Perspektive.
            "camera3d_view" => {
                let m = match a.first() { Some(Value::Mat4(m)) => **m, _ => return Err("CAMERA3D_VIEW: Arg 1 muss MAT4 sein".into()) };
                g!().set_camera3d_view(m); Value::Nil
            }
            "camera3d_projection" => {
                let m = match a.first() { Some(Value::Mat4(m)) => **m, _ => return Err("CAMERA3D_PROJECTION: Arg 1 muss MAT4 sein".into()) };
                g!().set_camera3d_projection(m); Value::Nil
            }
            "camera3d_x" => Value::Float(g!().cam3d_pos().0),
            "camera3d_y" => Value::Float(g!().cam3d_pos().1),
            "camera3d_z" => Value::Float(g!().cam3d_pos().2),
            "camera3d_target_x" => Value::Float(g!().cam3d_target().0),
            "camera3d_target_y" => Value::Float(g!().cam3d_target().1),
            "camera3d_target_z" => Value::Float(g!().cam3d_target().2),

            // --- Beleuchtung (Blinn-Phong) ---
            "light_enable" => { g!().light_enable(); Value::Nil }
            "light_ambient" => { g!().light_ambient(gi(a,0,"LIGHT_AMBIENT")?, need_f(a,1,"LIGHT_AMBIENT")?); Value::Nil }
            "light_fog" => { g!().light_fog(gi(a,0,"LIGHT_FOG")?, need_f(a,1,"LIGHT_FOG")?); Value::Nil }
            "light_env" => { g!().light_env(gi(a,0,"LIGHT_ENV")?, gi(a,1,"LIGHT_ENV")?, need_f(a,2,"LIGHT_ENV")?); Value::Nil }
            "light_env_hdr" => {
                // LIGHT_ENV_HDR(pfad$[, intensitaet]) -- echtes HDR-Cubemap-IBL.
                let path = gs(a, 0, "LIGHT_ENV_HDR")?.to_string();
                let intensity = if a.len() >= 2 { need_f(a, 1, "LIGHT_ENV_HDR")? } else { 1.0 };
                g!().light_env_hdr(&path, intensity)?; Value::Nil
            }
            "skybox" => { g!().skybox(gb(a, 0)); Value::Nil }
            "light_directional" => Value::Int(g!().light_add(
                0, need_f(a,0,"LIGHT_DIRECTIONAL")? as f32, need_f(a,1,"LIGHT_DIRECTIONAL")? as f32,
                need_f(a,2,"LIGHT_DIRECTIONAL")? as f32, gi(a,3,"LIGHT_DIRECTIONAL")?)),
            "light_point" => Value::Int(g!().light_add(
                1, need_f(a,0,"LIGHT_POINT")? as f32, need_f(a,1,"LIGHT_POINT")? as f32,
                need_f(a,2,"LIGHT_POINT")? as f32, gi(a,3,"LIGHT_POINT")?)),
            "light_set_pos" => { g!().light_set_pos(gi(a,0,"LIGHT_SET_POS")?,
                need_f(a,1,"LIGHT_SET_POS")? as f32, need_f(a,2,"LIGHT_SET_POS")? as f32, need_f(a,3,"LIGHT_SET_POS")? as f32)?; Value::Nil }
            "light_set_color" => { g!().light_set_color(gi(a,0,"LIGHT_SET_COLOR")?, gi(a,1,"LIGHT_SET_COLOR")?)?; Value::Nil }
            "light_set_enabled" => { g!().light_set_enabled(gi(a,0,"LIGHT_SET_ENABLED")?, gb(a,1))?; Value::Nil }
            "model_lit" => { g!().model_lit(gi(a,0,"MODEL_LIT")?)?; Value::Nil }
            "shadow_enable" => {
                let res = if !a.is_empty() { gi(a,0,"SHADOW_ENABLE")? as i32 } else { 1024 };
                g!().shadow_enable(res)?; Value::Nil
            }
            "shadow_area" => { g!().shadow_area(need_f(a,0,"SHADOW_AREA")?, need_f(a,1,"SHADOW_AREA")?); Value::Nil }
            "shadow_target" => { g!().shadow_target(
                need_f(a,0,"SHADOW_TARGET")? as f32, need_f(a,1,"SHADOW_TARGET")? as f32, need_f(a,2,"SHADOW_TARGET")? as f32); Value::Nil }

            // --- Audio (Core: SFX + Stream-Musik) ---
            "loadsound" => {
                let path = gs(a, 0, "LOADSOUND")?.to_string();
                Value::Int(self.audio_mut()?.load_sound(&path)?)
            }
            "sample_load" => {
                // SAMPLE_LOAD(path$) -> SAMPLE (Amiga-Stil-Sampler)
                let path = gs(a, 0, "SAMPLE_LOAD")?.to_string();
                Value::Int(self.audio_mut()?.sample_load(&path)?)
            }
            "sample_set_loop" => {
                // SAMPLE_SET_LOOP(sample, start, end) -- Loop-Region in Frames
                let idx = gi(a, 0, "SAMPLE_SET_LOOP")?;
                let start = gi(a, 1, "SAMPLE_SET_LOOP")?;
                let end = gi(a, 2, "SAMPLE_SET_LOOP")?;
                self.audio_mut()?.sample_set_loop(idx, start, end)?;
                Value::Nil
            }
            "sample_len" => {
                // SAMPLE_LEN(sample) -> Sekunden bei Originaltonhoehe
                let idx = gi(a, 0, "SAMPLE_LEN")?;
                Value::Float(self.audio_mut()?.sample_len(idx)?)
            }
            "sample_play" => {
                // SAMPLE_PLAY(sample, halbtoene, vol[, dur_ms]) -> AUDIO_CHANNEL
                let idx = gi(a, 0, "SAMPLE_PLAY")?;
                let semis = need_f(a, 1, "SAMPLE_PLAY")?;
                let vol = if a.len() >= 3 { need_f(a, 2, "SAMPLE_PLAY")? } else { 1.0 };
                let dur = if a.len() >= 4 { gi(a, 3, "SAMPLE_PLAY")? } else { 0 };
                Value::Int(self.audio_mut()?.sample_play(idx, semis, vol, dur)?)
            }
            "audio_lofi" => {
                // AUDIO_LOFI(an[, bits[, cutoff_hz]]) -- Paula/Amiga-Lo-Fi.
                // Argument-Pruefung VOR der Audio-Initialisierung (golden-testbar).
                let on = match a.first() {
                    Some(v) => v.truthy(),
                    None => return Err("AUDIO_LOFI: erwartet (an[, bits[, cutoff_hz]])".into()),
                };
                let bits = if a.len() >= 2 {
                    let b = gi(a, 1, "AUDIO_LOFI")?;
                    if !(1..=16).contains(&b) {
                        return Err("AUDIO_LOFI: bits muss 1..16 sein".into());
                    }
                    b as u32
                } else { 8 };
                let cutoff = if a.len() >= 3 {
                    let c = need_f(a, 2, "AUDIO_LOFI")?;
                    if c < 0.0 { return Err("AUDIO_LOFI: cutoff_hz muss >= 0 sein".into()); }
                    c
                } else { 3300.0 };
                self.audio_mut()?.set_lofi(on, bits, cutoff);
                Value::Nil
            }
            "audio_bus_volume" => {
                // AUDIO_BUS_VOLUME(bus$, vol) -- Master pro Bus (sfx/music/master).
                // Bus-Name VOR der Audio-Initialisierung pruefen (golden-testbar).
                let bus = gs(a, 0, "AUDIO_BUS_VOLUME")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_BUS_VOLUME: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let v = need_f(a, 1, "AUDIO_BUS_VOLUME")?;
                self.audio_mut()?.set_bus_volume(&bus, v)?;
                Value::Nil
            }
            "audio_bus_get_volume" => {
                let bus = gs(a, 0, "AUDIO_BUS_GET_VOLUME")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_BUS_GET_VOLUME: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                Value::Float(self.audio_mut()?.get_bus_volume(&bus)?)
            }
            "audio_filter" => {
                // AUDIO_FILTER(bus$, cutoff_hz[, resonance]) -- Tiefpass.
                let bus = gs(a, 0, "AUDIO_FILTER")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_FILTER: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let cutoff = need_f(a, 1, "AUDIO_FILTER")?;
                let res = if a.len() >= 3 { need_f(a, 2, "AUDIO_FILTER")? } else { 0.0 };
                self.audio_mut()?.set_filter(&bus, cutoff, res)?;
                Value::Nil
            }
            "audio_reverb" => {
                // AUDIO_REVERB(bus$, mix[, feedback[, damping]]) -- Hall.
                let bus = gs(a, 0, "AUDIO_REVERB")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_REVERB: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let mix = need_f(a, 1, "AUDIO_REVERB")?;
                let fb = if a.len() >= 3 { need_f(a, 2, "AUDIO_REVERB")? } else { 0.9 };
                let damp = if a.len() >= 4 { need_f(a, 3, "AUDIO_REVERB")? } else { 0.1 };
                self.audio_mut()?.set_reverb(&bus, mix, fb, damp)?;
                Value::Nil
            }
            "audio_delay" => {
                // AUDIO_DELAY(bus$, mix[, feedback]) -- Echo (300 ms).
                let bus = gs(a, 0, "AUDIO_DELAY")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_DELAY: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let mix = need_f(a, 1, "AUDIO_DELAY")?;
                let fb = if a.len() >= 3 { need_f(a, 2, "AUDIO_DELAY")? } else { 0.5 };
                let time_ms = if a.len() >= 4 { gi(a, 3, "AUDIO_DELAY")? } else { 0 };
                self.audio_mut()?.set_delay(&bus, mix, fb, time_ms)?;
                Value::Nil
            }
            "audio_distortion" => {
                // AUDIO_DISTORTION(bus$, amount[, mix]) -- Overdrive/Fuzz.
                let bus = gs(a, 0, "AUDIO_DISTORTION")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_DISTORTION: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let amount = need_f(a, 1, "AUDIO_DISTORTION")?;
                let mix = if a.len() >= 3 { need_f(a, 2, "AUDIO_DISTORTION")? } else { 1.0 };
                self.audio_mut()?.set_distortion(&bus, amount, mix)?;
                Value::Nil
            }
            "audio_compressor" => {
                // AUDIO_COMPRESSOR(bus$, threshold_db, ratio[, makeup_db]).
                let bus = gs(a, 0, "AUDIO_COMPRESSOR")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_COMPRESSOR: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let thresh = need_f(a, 1, "AUDIO_COMPRESSOR")?;
                let ratio = need_f(a, 2, "AUDIO_COMPRESSOR")?;
                let makeup = if a.len() >= 4 { need_f(a, 3, "AUDIO_COMPRESSOR")? } else { 0.0 };
                self.audio_mut()?.set_compressor(&bus, thresh, ratio, makeup)?;
                Value::Nil
            }
            "audio_eq" => {
                // AUDIO_EQ(bus$, freq_hz, gain_db[, q]) -- parametrischer Bell-EQ.
                let bus = gs(a, 0, "AUDIO_EQ")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_EQ: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let freq = need_f(a, 1, "AUDIO_EQ")?;
                let gain = need_f(a, 2, "AUDIO_EQ")?;
                let q = if a.len() >= 4 { need_f(a, 3, "AUDIO_EQ")? } else { 1.0 };
                self.audio_mut()?.set_eq(&bus, freq, gain, q)?;
                Value::Nil
            }
            "playsound" => {
                // PLAYSOUND(sound[, loops, volume]). `loops` wird nativ ignoriert
                // (raylib-Sounds loopen nicht) -- SFX spielen einmal.
                let idx = gi(a, 0, "PLAYSOUND")?;
                let vol = if a.len() >= 3 { need_f(a, 2, "PLAYSOUND")? } else { 1.0 };
                self.audio_mut()?.play_sound(idx, vol)?;
                Value::Nil
            }
            "stopsound" => {
                let idx = gi(a, 0, "STOPSOUND")?;
                self.audio_mut()?.stop_sound(idx)?;
                Value::Nil
            }
            "unloadsound" => {
                // UNLOADSOUND(sound) -- stoppt + gibt den Frame-Puffer frei.
                // Index bleibt als Tombstone gueltig (kein Recycling).
                let idx = gi(a, 0, "UNLOADSOUND")?;
                self.audio_mut()?.unload_sound(idx)?;
                Value::Nil
            }
            "audio_sound_count" => {
                // AUDIO_SOUND_COUNT() -> Anzahl lebender Sound-Slots (Diagnose).
                Value::Int(self.audio_mut()?.sound_count())
            }
            "playmusic" => {
                // PLAYMUSIC(pfad$[, loops, volume]). Musik loopt (raylib-Default);
                // `loops` wird nativ nicht ausgewertet.
                let path = gs(a, 0, "PLAYMUSIC")?.to_string();
                let vol = if a.len() >= 3 { need_f(a, 2, "PLAYMUSIC")? } else { 1.0 };
                self.audio_mut()?.play_music(&path, vol)?;
                Value::Nil
            }
            "stopmusic" => { self.audio_mut()?.stop_music(); Value::Nil }
            "audio_fft" => {
                // AUDIO_FFT(arr): fuellt ein 1D ARRAY OF FLOAT mit B Band-Pegeln
                // (0..1) aus dem aktuell hoerbaren Audio (echte FFT).
                let arr = match a.first() {
                    Some(Value::Array(x)) => x.clone(),
                    _ => return Err("AUDIO_FFT: erwartet ARRAY OF FLOAT".into()),
                };
                let n = {
                    let b = arr.borrow();
                    if b.element_type != "float" || b.dims.len() != 1 {
                        return Err("AUDIO_FFT: erwartet 1D ARRAY OF FLOAT".into());
                    }
                    b.cells.len()
                };
                let mut tmp = vec![0.0f32; n];
                self.audio_mut()?.fft_bands(&mut tmp);
                let mut b = arr.borrow_mut();
                for i in 0..n { b.cells.set(i, Value::Float(tmp[i] as f64)); }
                Value::Nil
            }

            // --- erweitertes audio-Modul (AUDIO_*) ---
            "audio_init" => { self.audio_mut()?; Value::Nil }
            "audio_set_num_channels" => { let n = gi(a, 0, "AUDIO_SET_NUM_CHANNELS")?; if n < 0 { return Err("AUDIO_SET_NUM_CHANNELS: n muss >= 0 sein".into()); } self.audio_mut()?.set_num_channels(n); Value::Nil }
            "audio_num_channels" => Value::Int(self.audio_mut()?.get_num_channels()),
            "audio_busy_channels" => Value::Int(self.audio_mut()?.busy_channels()),
            "audio_pause_all" => { self.audio_mut()?.pause_all(); Value::Nil }
            "audio_resume_all" => { self.audio_mut()?.resume_all(); Value::Nil }
            "audio_stop_all" => { self.audio_mut()?.stop_all(); Value::Nil }
            "audio_play" => {
                // AUDIO_PLAY(sound[, loops[, volume[, fade_in_ms]]]) -- loops=0 einmal (Default), -1 endlos
                let idx = gi(a, 0, "AUDIO_PLAY")?;
                let loops = if a.len() >= 2 { gi(a, 1, "AUDIO_PLAY")? } else { 0 };
                let vol = if a.len() >= 3 { need_f(a, 2, "AUDIO_PLAY")? } else { 1.0 };
                let fade = if a.len() >= 4 { gi(a, 3, "AUDIO_PLAY")? } else { 0 };
                if fade < 0 { return Err("AUDIO_PLAY: fade_in_ms muss >= 0 sein".into()); }
                Value::Int(self.audio_mut()?.ch_play(idx, loops, vol, fade)?)
            }
            "audio_pause" => { let i = gi(a, 0, "AUDIO_PAUSE")?; self.audio_mut()?.ch_pause(i)?; Value::Nil }
            "audio_resume" => { let i = gi(a, 0, "AUDIO_RESUME")?; self.audio_mut()?.ch_resume(i)?; Value::Nil }
            "audio_stop" => {
                // AUDIO_STOP(ch[, fade_out_ms])
                let i = gi(a, 0, "AUDIO_STOP")?;
                let fade = if a.len() >= 2 { gi(a, 1, "AUDIO_STOP")? } else { 0 };
                if fade < 0 { return Err("AUDIO_STOP: fade_out_ms muss >= 0 sein".into()); }
                self.audio_mut()?.ch_stop(i, fade)?; Value::Nil
            }
            "audio_is_playing" => { let i = gi(a, 0, "AUDIO_IS_PLAYING")?; Value::Bool(self.audio_mut()?.ch_is_playing(i)?) }
            "audio_volume" | "audio_set_volume" => { let i = gi(a, 0, "AUDIO_VOLUME")?; let v = need_f(a, 1, "AUDIO_VOLUME")?; self.audio_mut()?.ch_set_volume(i, v)?; Value::Nil }
            "audio_get_volume" => { let i = gi(a, 0, "AUDIO_GET_VOLUME")?; Value::Float(self.audio_mut()?.ch_get_volume(i)?) }
            "audio_pan" => { let i = gi(a, 0, "AUDIO_PAN")?; let l = need_f(a, 1, "AUDIO_PAN")?; let r = need_f(a, 2, "AUDIO_PAN")?; self.audio_mut()?.ch_pan(i, l, r)?; Value::Nil }
            "audio_pitch" => {
                let i = gi(a, 0, "AUDIO_PITCH")?;
                let f = need_f(a, 1, "AUDIO_PITCH")?;
                if f <= 0.0 { return Err("AUDIO_PITCH: faktor muss > 0 sein".into()); }
                self.audio_mut()?.ch_pitch(i, f)?; Value::Nil
            }
            "audio_music_pitch" => {
                let f = need_f(a, 0, "AUDIO_MUSIC_PITCH")?;
                if f <= 0.0 { return Err("AUDIO_MUSIC_PITCH: faktor muss > 0 sein".into()); }
                self.audio_mut()?.music_set_pitch(f); Value::Nil
            }
            "audio_music_get_pitch" => Value::Float(self.audio_mut()?.music_get_pitch()),
            "audio_pan_pos" => { let i = gi(a, 0, "AUDIO_PAN_POS")?; let p = need_f(a, 1, "AUDIO_PAN_POS")?; self.audio_mut()?.ch_pan_pos(i, p)?; Value::Nil }
            "audio_pan_slide" => {
                // AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms) -- Positionen 0=links..1=rechts
                let i = gi(a, 0, "AUDIO_PAN_SLIDE")?;
                let von = need_f(a, 1, "AUDIO_PAN_SLIDE")?;
                let nach = need_f(a, 2, "AUDIO_PAN_SLIDE")?;
                let dauer = gi(a, 3, "AUDIO_PAN_SLIDE")?;
                if dauer <= 0 { return Err("AUDIO_PAN_SLIDE: dauer_ms muss > 0 sein".into()); }
                self.audio_mut()?.ch_pan_slide(i, von, nach, dauer)?; Value::Nil
            }
            "audio_autopan" => {
                // AUDIO_AUTOPAN(ch, periode_s[, tiefe]) -- periode_s <= 0 = aus
                let i = gi(a, 0, "AUDIO_AUTOPAN")?;
                let periode = need_f(a, 1, "AUDIO_AUTOPAN")?;
                let tiefe = if a.len() >= 3 { need_f(a, 2, "AUDIO_AUTOPAN")? } else { 1.0 };
                self.audio_mut()?.ch_autopan(i, periode, tiefe)?; Value::Nil
            }
            "audio_tone" => {
                let freq = need_f(a, 0, "AUDIO_TONE")?;
                let dur = gi(a, 1, "AUDIO_TONE")?;
                let wf = if a.len() >= 3 { gs(a, 2, "AUDIO_TONE")?.to_string() } else { "sine".to_string() };
                let vol = if a.len() >= 4 { need_f(a, 3, "AUDIO_TONE")? } else { 1.0 };
                Value::Int(self.audio_mut()?.tone(freq, dur, &wf, vol)?)
            }
            "audio_noise" => {
                let dur = gi(a, 0, "AUDIO_NOISE")?;
                let vol = if a.len() >= 2 { need_f(a, 1, "AUDIO_NOISE")? } else { 1.0 };
                Value::Int(self.audio_mut()?.noise(dur, vol)?)
            }
            "audio_sfx" => {
                // AUDIO_SFX(wf, freq, slide, atk, sus, dec, vib_d, vib_s, vol
                //   [, stereo_width, duty, pwm_depth, pwm_speed,
                //    flt_cutoff, flt_sweep, flt_res]) -- SID-Args optional.
                let wf = gs(a, 0, "AUDIO_SFX")?.to_string();
                let freq = need_f(a, 1, "AUDIO_SFX")?;
                let slide = need_f(a, 2, "AUDIO_SFX")?;
                let atk = gi(a, 3, "AUDIO_SFX")?;
                let sus = gi(a, 4, "AUDIO_SFX")?;
                let dec = gi(a, 5, "AUDIO_SFX")?;
                let vd = need_f(a, 6, "AUDIO_SFX")?;
                let vs = need_f(a, 7, "AUDIO_SFX")?;
                let vol = need_f(a, 8, "AUDIO_SFX")?;
                let optf = |i: usize, d: f64| -> R<f64> {
                    if a.len() > i { need_f(a, i, "AUDIO_SFX") } else { Ok(d) }
                };
                let width = optf(9, 0.0)?;
                let duty = optf(10, 0.5)?;
                let pwm_depth = optf(11, 0.0)?;
                let pwm_speed = optf(12, 0.0)?;
                let flt_cutoff = optf(13, 0.0)?;
                let flt_sweep = optf(14, 0.0)?;
                let flt_res = optf(15, 0.0)?;
                Value::Int(self.audio_mut()?.sfx(
                    &wf, freq, slide, atk, sus, dec, vd, vs, vol, width,
                    duty, pwm_depth, pwm_speed, flt_cutoff, flt_sweep, flt_res)?)
            }
            "audio_music_load" => { let p = gs(a, 0, "AUDIO_MUSIC_LOAD")?.to_string(); self.audio_mut()?.music_load(&p)?; Value::Nil }
            "audio_music_play" => {
                // AUDIO_MUSIC_PLAY([loops[, fade_in_ms]]) -- loops=-1 endlos (Default)
                let loops = if !a.is_empty() { gi(a, 0, "AUDIO_MUSIC_PLAY")? } else { -1 };
                let fade = if a.len() >= 2 { gi(a, 1, "AUDIO_MUSIC_PLAY")? } else { 0 };
                if fade < 0 { return Err("AUDIO_MUSIC_PLAY: fade_in_ms muss >= 0 sein".into()); }
                self.audio_mut()?.music_play(loops, fade); Value::Nil
            }
            "audio_music_stop" => {
                // AUDIO_MUSIC_STOP([fade_out_ms])
                let fade = if !a.is_empty() { gi(a, 0, "AUDIO_MUSIC_STOP")? } else { 0 };
                if fade < 0 { return Err("AUDIO_MUSIC_STOP: fade_out_ms muss >= 0 sein".into()); }
                self.audio_mut()?.music_stop(fade); Value::Nil
            }
            "audio_music_pause" => { self.audio_mut()?.music_pause(); Value::Nil }
            "audio_music_resume" => { self.audio_mut()?.music_resume(); Value::Nil }
            "audio_music_volume" | "audio_music_set_volume" => { let v = need_f(a, 0, "AUDIO_MUSIC_VOLUME")?; self.audio_mut()?.music_set_volume(v); Value::Nil }
            "audio_music_get_volume" => Value::Float(self.audio_mut()?.music_get_volume()),
            "audio_music_position" => Value::Float(self.audio_mut()?.music_position()),
            "audio_music_busy" => Value::Bool(self.audio_mut()?.music_busy()),
            "audio_music_queue" => { let p = gs(a, 0, "AUDIO_MUSIC_QUEUE")?.to_string(); self.audio_mut()?.music_queue(&p); Value::Nil }

            // --- Bulk-Draws ---
            "plots" => {
                let xs = arr_i32(&a[0], "PLOTS")?; let ys = arr_i32(&a[1], "PLOTS")?;
                let n = xs.len().min(ys.len());
                let cols = bulk_color(&a[2], n, "PLOTS")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.plot(xs[i], ys[i], cols[i]); }
                Value::Nil
            }
            "boxes" => {
                let x1=arr_i32(&a[0],"BOXES")?; let y1=arr_i32(&a[1],"BOXES")?; let x2=arr_i32(&a[2],"BOXES")?; let y2=arr_i32(&a[3],"BOXES")?;
                let n = x1.len().min(y1.len()).min(x2.len()).min(y2.len());
                let cols = bulk_color(&a[4], n, "BOXES")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.box_fill(x1[i], y1[i], x2[i], y2[i], cols[i]); }
                Value::Nil
            }
            "circles" => {
                let xs=arr_i32(&a[0],"CIRCLES")?; let ys=arr_i32(&a[1],"CIRCLES")?; let rs=arr_i32(&a[2],"CIRCLES")?;
                let n = xs.len().min(ys.len()).min(rs.len());
                let cols = bulk_color(&a[3], n, "CIRCLES")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.circle(xs[i], ys[i], rs[i], cols[i]); }
                Value::Nil
            }
            "lines" => {
                let x1=arr_i32(&a[0],"LINES")?; let y1=arr_i32(&a[1],"LINES")?; let x2=arr_i32(&a[2],"LINES")?; let y2=arr_i32(&a[3],"LINES")?;
                let n = x1.len().min(y1.len()).min(x2.len()).min(y2.len());
                let cols = bulk_color(&a[4], n, "LINES")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.line(x1[i], y1[i], x2[i], y2[i], cols[i]); }
                Value::Nil
            }

            // --- Bilder erweitert ---
            "drawimagepart" => {
                let idx = gi(a,0,"DRAWIMAGEPART")?;
                g!().draw_image_part(idx, gi(a,1,"DRAWIMAGEPART")? as i32, gi(a,2,"DRAWIMAGEPART")? as i32,
                    gi(a,3,"DRAWIMAGEPART")? as i32, gi(a,4,"DRAWIMAGEPART")? as i32,
                    gi(a,5,"DRAWIMAGEPART")? as i32, gi(a,6,"DRAWIMAGEPART")? as i32)?; Value::Nil
            }
            "drawimageflipped" => {
                let idx = gi(a,0,"DRAWIMAGEFLIPPED")?;
                let fh = gb(a, 3); let fv = gb(a, 4);
                g!().draw_image_flipped(idx, gi(a,1,"DRAWIMAGEFLIPPED")? as i32, gi(a,2,"DRAWIMAGEFLIPPED")? as i32, fh, fv)?; Value::Nil
            }
            "drawtilemap" => {
                if a.len() != 6 { return Err(format!("DRAWTILEMAP: erwartet 6 Argument(e), erhalten {}", a.len())); }
                let idx = gi(a, 0, "DRAWTILEMAP")?;
                let (vals, rows, cols) = match &a[1] {
                    Value::Array(arr) => {
                        let arr = arr.borrow();
                        if arr.element_type != "integer" {
                            return Err("DRAWTILEMAP: map muss ARRAY OF INTEGER sein".into());
                        }
                        if arr.dims.len() != 2 {
                            return Err("DRAWTILEMAP: map muss 2D sein (zeilen x spalten)".into());
                        }
                        let mut v = Vec::with_capacity(arr.cells.len());
                        if let Some(ints) = arr.cells.as_ints() {
                            v.extend_from_slice(ints);
                        } else {
                            for x in arr.cells.iter() {
                                match x {
                                    Value::Int(n) => v.push(n),
                                    _ => return Err("DRAWTILEMAP: map muss ARRAY OF INTEGER sein".into()),
                                }
                            }
                        }
                        (v, arr.dims[0] as i32, arr.dims[1] as i32)
                    }
                    _ => return Err("DRAWTILEMAP: map muss ARRAY OF INTEGER sein".into()),
                };
                let tw = gi(a, 2, "DRAWTILEMAP")? as i32;
                let th = gi(a, 3, "DRAWTILEMAP")? as i32;
                let sx = gi(a, 4, "DRAWTILEMAP")? as i32;
                let sy = gi(a, 5, "DRAWTILEMAP")? as i32;
                g!().draw_tilemap(idx, &vals, rows, cols, tw, th, sx, sy)?;
                Value::Nil
            }
            "load_assets" => Value::Int(g!().load_assets(gs(a,0,"LOAD_ASSETS")?)?),

            // --- Z-Layer ---
            "layer_define" => { g!().layer_define(gs(a,0,"LAYER_DEFINE")?, gi(a,1,"LAYER_DEFINE")? as i32); Value::Nil }
            "layer" => { let n = gs(a,0,"LAYER")?.to_string(); g!().layer(&n); Value::Nil }
            "layer_end" => { g!().layer_end(); Value::Nil }
            "layer_clear" => { let n = gs(a,0,"LAYER_CLEAR")?.to_string(); g!().layer_clear(&n); Value::Nil }

            // --- Sprite-Atlas ---
            "atlas_load" => Value::Int(g!().atlas_load(gs(a,0,"ATLAS_LOAD")?)?),
            "atlas_draw" | "batch_draw" => {
                let atlas = gi(a,0,"ATLAS_DRAW")?;
                let name = gs(a,1,"ATLAS_DRAW")?.to_string();
                let tint = if a.len() > 4 { Some(gi(a,4,"ATLAS_DRAW")?) } else { None };
                g!().atlas_draw(atlas, &name, gi(a,2,"ATLAS_DRAW")? as i32, gi(a,3,"ATLAS_DRAW")? as i32, false, tint)?; Value::Nil
            }
            "atlas_draw_flipped" => {
                let atlas = gi(a,0,"ATLAS_DRAW_FLIPPED")?;
                let name = gs(a,1,"ATLAS_DRAW_FLIPPED")?.to_string();
                let fh = gb(a, 4);
                let tint = if a.len() > 5 { Some(gi(a,5,"ATLAS_DRAW_FLIPPED")?) } else { None };
                g!().atlas_draw(atlas, &name, gi(a,2,"ATLAS_DRAW_FLIPPED")? as i32, gi(a,3,"ATLAS_DRAW_FLIPPED")? as i32, fh, tint)?; Value::Nil
            }
            "batch_flush" => Value::Nil, // Recording-Modell: alles flusht beim FLIP

            // --- Modul: input ---
            "input_bind" => {
                if a.len() < 2 { return Err("INPUT_BIND: erwartet Action + mind. 1 Key".into()); }
                let action = gs(a, 0, "INPUT_BIND")?.to_lowercase();
                let mut keys = Vec::with_capacity(a.len() - 1);
                for i in 1..a.len() { keys.push(gi(a, i, "INPUT_BIND")?); }
                self.input_state.actions.insert(action, keys);
                Value::Nil
            }
            "input_unbind" => { let action = gs(a, 0, "INPUT_UNBIND")?.to_lowercase(); self.input_state.actions.remove(&action); Value::Nil }
            "input_reset" => { self.input_state.actions.clear(); self.input_state.prev.clear(); self.input_state.cur.clear(); Value::Nil }
            "input_update" => {
                // Ohne Fenster (kein SCREEN) sind keine Tasten gedrueckt --
                // wie in der Python-VM (pygame ohne Display). Konsolen-Demos
                // laufen so bit-identisch.
                let keys: Vec<i64> = self.input_state.actions.values().flatten().copied().collect();
                let mut cur = HashSet::new();
                if let Some(g) = self.gfx.as_ref() {
                    for k in keys { if g.key_down(k) { cur.insert(k); } }
                }
                self.input_state.prev = std::mem::replace(&mut self.input_state.cur, cur);
                Value::Nil
            }
            "input_held" | "input_pressed" | "input_released" => {
                let action = gs(a, 0, "INPUT")?.to_lowercase();
                let keys = self.input_state.actions.get(&action)
                    .ok_or_else(|| format!("{}: Action '{}' nicht gebunden", name.to_uppercase(), action))?;
                let (cur, prev) = (&self.input_state.cur, &self.input_state.prev);
                let r = match name {
                    "input_held" => keys.iter().any(|k| cur.contains(k)),
                    "input_pressed" => keys.iter().any(|k| cur.contains(k) && !prev.contains(k)),
                    _ => keys.iter().any(|k| prev.contains(k) && !cur.contains(k)),
                };
                Value::Bool(r)
            }
            "input_axis" => {
                let neg = gs(a, 0, "INPUT_AXIS")?.to_lowercase();
                let pos = gs(a, 1, "INPUT_AXIS")?.to_lowercase();
                let held = |act: &str| -> R<bool> {
                    let keys = self.input_state.actions.get(act)
                        .ok_or_else(|| format!("Action '{}' nicht gebunden", act))?;
                    Ok(keys.iter().any(|k| self.input_state.cur.contains(k)))
                };
                let (n, p) = (held(&neg)?, held(&pos)?);
                Value::Int(if n && !p { -1 } else if p && !n { 1 } else { 0 })
            }
            "input_bound" => {
                let action = gs(a, 0, "INPUT_BOUND")?.to_lowercase();
                Value::Bool(self.input_state.actions.get(&action).map(|k| !k.is_empty()).unwrap_or(false))
            }
            "input_joy_count" => Value::Int(self.gfx.as_ref().map(|g| g.joy_count()).unwrap_or(0)),
            "input_joy_name" => {
                let idx = gi(a, 0, "INPUT_JOY_NAME")?;
                Value::Str(self.gfx.as_ref().map(|g| g.joy_name(idx)).unwrap_or_default().into())
            }
            "input_joy_axis" => {
                let pad = gi(a, 0, "INPUT_JOY_AXIS")?;
                let name = gs(a, 1, "INPUT_JOY_AXIS")?.to_lowercase();
                let axis_idx = match name.as_str() {
                    "left_x" => 0, "left_y" => 1, "right_x" => 2, "right_y" => 3,
                    "lt" => 4, "rt" => 5,
                    _ => return Err(format!(
                        "INPUT_JOY_AXIS: unbekannte Achse '{}' (erlaubt: left_x, left_y, right_x, right_y, lt, rt)", name)),
                };
                let mut v = self.gfx.as_ref().map(|g| g.joy_axis(pad, axis_idx)).unwrap_or(0.0);
                // Deadzone fuer Sticks (Trigger unangetastet) -- wie Python.
                if (name.starts_with("left_") || name.starts_with("right_")) && v.abs() < 0.15 {
                    v = 0.0;
                }
                Value::Float(v)
            }

            // --- Modul: camera ---
            "camera_set" => {
                let x = need_f(a, 0, "CAMERA_SET")?; let y = need_f(a, 1, "CAMERA_SET")?;
                let zoom = if a.len() == 3 { let z = need_f(a, 2, "CAMERA_SET")?; if z <= 0.0 { return Err("CAMERA_SET: zoom muss > 0 sein".into()); } z } else { 1.0 };
                g!().set_camera(x, y, zoom); Value::Nil
            }
            "camera_reset" => { g!().reset_camera(); Value::Nil }
            "camera_shake" => {
                // CAMERA_SHAKE(staerke[, dauer_ms]) -- staerke=0 stoppt sofort
                let amp = need_f(a, 0, "CAMERA_SHAKE")?;
                if amp < 0.0 { return Err("CAMERA_SHAKE: staerke muss >= 0 sein".into()); }
                let dur = if a.len() >= 2 { gi(a, 1, "CAMERA_SHAKE")? } else { 300 };
                if dur <= 0 { return Err("CAMERA_SHAKE: dauer_ms muss > 0 sein".into()); }
                g!().camera_shake(amp, dur as f64); Value::Nil
            }
            "camera_x" => Value::Float(g!().camera().0),
            "camera_y" => Value::Float(g!().camera().1),
            "camera_zoom" => Value::Float(g!().camera().2),
            "camera_follow" => {
                let (tx, ty, sw, sh) = (need_f(a,0,"CAMERA_FOLLOW")?, need_f(a,1,"CAMERA_FOLLOW")?, need_f(a,2,"CAMERA_FOLLOW")?, need_f(a,3,"CAMERA_FOLLOW")?);
                let z = g!().camera().2;
                g!().set_camera(tx - (sw / z) / 2.0, ty - (sh / z) / 2.0, z); Value::Nil
            }
            "camera_s2w_x" => Value::Float(g!().s2w_x(need_f(a, 0, "CAMERA_S2W_X")?)),
            "camera_s2w_y" => Value::Float(g!().s2w_y(need_f(a, 0, "CAMERA_S2W_Y")?)),

            // --- Modul: sprite (nur SPRITE_DRAW braucht Grafik) ---
            "sprite_draw" => {
                let sp = match a.first() {
                    Some(Value::Sprite(s)) => s.clone(),
                    _ => return Err("SPRITE_DRAW erwartet SPRITE".into()),
                };
                let s = sp.borrow();
                let g = self.gfx.as_mut().ok_or("SPRITE_DRAW vor SCREEN aufgerufen")?;
                g.draw_sprite(s.tex_idx, s.current_frame, s.frame_w, s.frame_h,
                    s.x as i32, s.y as i32, s.flip_x, s.flip_y, s.scale_x, s.scale_y, s.tint)?;
                Value::Nil
            }
            // --- Modul: imgfx (immutable, liefern neues IMAGE-Handle) ---
            "image_scale" => Value::Int(g!().image_scale(gi(a,0,"IMAGE_SCALE")?, gi(a,1,"IMAGE_SCALE")? as i32, gi(a,2,"IMAGE_SCALE")? as i32)?),
            "image_rotate" => Value::Int(g!().image_rotate(gi(a,0,"IMAGE_ROTATE")?, need_f(a,1,"IMAGE_ROTATE")? as f32)?),
            "image_flip" => Value::Int(g!().image_flip(gi(a,0,"IMAGE_FLIP")?, gb(a,1), gb(a,2))?),
            "image_tint" => { let c = gi(a,1,"IMAGE_TINT")?; if c < 0 || c > 0xFFFFFF { return Err("IMAGE_TINT: Farbe muss 0..0xFFFFFF sein".into()); } Value::Int(g!().image_tint(gi(a,0,"IMAGE_TINT")?, c)?) }
            "image_copy" => Value::Int(g!().image_copy(gi(a,0,"IMAGE_COPY")?)?),

            // --- Modul: ui (Immediate-Mode) ---
            "ui_label" => {
                let x = gi(a,0,"UI_LABEL")? as i32 + self.ui_state.offset_x; let y = gi(a,1,"UI_LABEL")? as i32 + self.ui_state.offset_y;
                let text = gs(a,2,"UI_LABEL")?.to_string();
                let c = if a.len() == 4 { gi(a,3,"UI_LABEL")? } else { self.ui_state.th("text_fg") };
                g!().text(x, y, text, c); Value::Nil
            }
            "ui_button" => {
                let id = gs(a,0,"UI_BUTTON")?.to_string();
                let (x,y,w,h) = (gi(a,1,"UI_BUTTON")? as i32 + self.ui_state.offset_x, gi(a,2,"UI_BUTTON")? as i32 + self.ui_state.offset_y, gi(a,3,"UI_BUTTON")? as i32, gi(a,4,"UI_BUTTON")? as i32);
                let text = gs(a,5,"UI_BUTTON")?.to_string();
                let bg_color = if a.len() >= 7 { gi(a,6,"UI_BUTTON")? } else { self.ui_state.th("button_bg") };
                let fg_color = if a.len() >= 8 { gi(a,7,"UI_BUTTON")? } else { self.ui_state.th("text_fg") };
                let (mx,my,down) = self.ui_mouse_gated()?;
                let hovered = ui_in_box(mx,my,x,y,w,h);
                if hovered && down && !self.ui_state.was_mouse_down { self.ui_state.click_origin = Some(id.clone()); }
                let clicked = hovered && !down && self.ui_state.was_mouse_down && self.ui_state.click_origin.as_deref() == Some(id.as_str());
                let bg = if hovered && down { ui_darken(bg_color, 40) } else if hovered { ui_lighten(bg_color, 0.25) } else { bg_color };
                let g = self.gfx.as_mut().unwrap();
                g.box_fill(x,y,x+w-1,y+h-1,bg); g.rect(x,y,x+w-1,y+h-1,fg_color); g.text(x+6, y+(h-14)/2, text, fg_color);
                Value::Bool(clicked)
            }
            "ui_checkbox" => {
                let id = gs(a,0,"UI_CHECKBOX")?.to_string();
                let (x,y) = (gi(a,1,"UI_CHECKBOX")? as i32 + self.ui_state.offset_x, gi(a,2,"UI_CHECKBOX")? as i32 + self.ui_state.offset_y);
                let label = gs(a,3,"UI_CHECKBOX")?.to_string();
                if !self.ui_state.checkbox.contains_key(&id) {
                    let init = if a.len() == 5 { matches!(a[4], Value::Bool(true)) } else { false };
                    self.ui_state.checkbox.insert(id.clone(), init);
                }
                let bs = self.ui_state.metric("checkbox_size") as i32;
                let (mx,my,down) = self.ui_mouse_gated()?;
                let hovered = ui_in_box(mx,my,x,y,bs,bs);
                if hovered && down && !self.ui_state.was_mouse_down {
                    let v = !self.ui_state.checkbox[&id]; self.ui_state.checkbox.insert(id.clone(), v);
                }
                let val = self.ui_state.checkbox[&id];
                let (border, fill, fg) = (self.ui_state.th("field_border"), self.ui_state.th("accent"), self.ui_state.th("text_fg"));
                let g = self.gfx.as_mut().unwrap();
                g.rect(x,y,x+bs-1,y+bs-1,border);
                if hovered { g.rect(x-1,y-1,x+bs,y+bs,fill); }
                if val { g.box_fill(x+3,y+3,x+bs-4,y+bs-4,fill); }
                g.text(x+bs+5, y, label, fg);
                Value::Bool(val)
            }
            "ui_slider" => {
                let id = gs(a,0,"UI_SLIDER")?.to_string();
                let (x,y,w) = (gi(a,1,"UI_SLIDER")? as i32 + self.ui_state.offset_x, gi(a,2,"UI_SLIDER")? as i32 + self.ui_state.offset_y, gi(a,3,"UI_SLIDER")? as i32);
                let (mn, mx_) = (need_f(a,4,"UI_SLIDER")?, need_f(a,5,"UI_SLIDER")?);
                if mx_ <= mn { return Err("UI_SLIDER: max muss > min sein".into()); }
                if !self.ui_state.slider.contains_key(&id) {
                    let init = if a.len() == 7 { need_f(a,6,"UI_SLIDER")?.max(mn).min(mx_) } else { mn };
                    self.ui_state.slider.insert(id.clone(), init);
                }
                let h = self.ui_state.metric("slider_h") as i32;
                let (mox,moy,down) = self.ui_mouse_gated()?;
                if ui_in_box(mox,moy,x,y,w,h) && down {
                    let rel = ((mox - x) as f64 / (w - 1).max(1) as f64).max(0.0).min(1.0);
                    self.ui_state.slider.insert(id.clone(), mn + rel * (mx_ - mn));
                }
                let val = self.ui_state.slider[&id];
                let (track, border, handle) = (self.ui_state.th("slider_track"), self.ui_state.th("text_fg"), self.ui_state.th("accent"));
                let handle_w = 10;
                let hx = x + ((val - mn) / (mx_ - mn) * (w - handle_w) as f64) as i32;
                let g = self.gfx.as_mut().unwrap();
                g.box_fill(x, y + h/2 - 1, x+w-1, y+h/2+1, track);
                g.rect(x,y,x+w-1,y+h-1,border);
                g.box_fill(hx, y, hx+handle_w-1, y+h-1, handle);
                Value::Float(val)
            }
            "ui_progress" => {
                let (x,y,w,h) = (gi(a,0,"UI_PROGRESS")? as i32 + self.ui_state.offset_x, gi(a,1,"UI_PROGRESS")? as i32 + self.ui_state.offset_y, gi(a,2,"UI_PROGRESS")? as i32, gi(a,3,"UI_PROGRESS")? as i32);
                let (value, maxv) = (need_f(a,4,"UI_PROGRESS")?, need_f(a,5,"UI_PROGRESS")?);
                if maxv <= 0.0 { return Err("UI_PROGRESS: max muss > 0 sein".into()); }
                let fg = if a.len() >= 7 { gi(a,6,"UI_PROGRESS")? } else { self.ui_state.th("progress_fg") };
                let bg = if a.len() >= 8 { gi(a,7,"UI_PROGRESS")? } else { self.ui_state.th("progress_bg") };
                if w >= 2 && h >= 2 {
                    let ratio = (value / maxv).max(0.0).min(1.0);
                    let fill_w = ((w - 2) as f64 * ratio) as i32;
                    let tfg = self.ui_state.th("text_fg");
                    let g = self.gfx.as_mut().unwrap();
                    g.box_fill(x,y,x+w-1,y+h-1,bg); g.rect(x,y,x+w-1,y+h-1,tfg);
                    if fill_w > 0 { g.box_fill(x+1, y+1, x+1+fill_w-1, y+h-2, fg); }
                }
                Value::Nil
            }
            "ui_panel" => {
                let (x,y,w,h) = (gi(a,0,"UI_PANEL")? as i32 + self.ui_state.offset_x, gi(a,1,"UI_PANEL")? as i32 + self.ui_state.offset_y, gi(a,2,"UI_PANEL")? as i32, gi(a,3,"UI_PANEL")? as i32);
                let title = if a.len() >= 5 { gs(a,4,"UI_PANEL")?.to_string() } else { String::new() };
                let bg = if a.len() >= 6 { gi(a,5,"UI_PANEL")? } else { self.ui_state.th("panel_bg") };
                let (border, tbg, fg) = (self.ui_state.th("panel_border"), self.ui_state.th("panel_title_bg"), self.ui_state.th("text_fg"));
                let g = self.gfx.as_mut().unwrap();
                g.box_fill(x,y,x+w-1,y+h-1,bg); g.rect(x,y,x+w-1,y+h-1,border);
                if !title.is_empty() {
                    g.box_fill(x,y,x+w-1,y+17,tbg); g.rect(x,y,x+w-1,y+17,border); g.text(x+6,y+2,title,fg);
                }
                Value::Nil
            }
            "ui_radio" => {
                let id = gs(a,0,"UI_RADIO")?.to_string();
                let (x,y) = (gi(a,1,"UI_RADIO")? as i32 + self.ui_state.offset_x, gi(a,2,"UI_RADIO")? as i32 + self.ui_state.offset_y);
                let opts: Vec<String> = match a.get(3) {
                    Some(Value::Array(arr)) => { let arr = arr.borrow();
                        if arr.element_type != "string" { return Err("UI_RADIO: options muss ARRAY OF STRING sein".into()); }
                        arr.cells.iter().map(|v| match v { Value::Str(s) => s.to_string(), o => o.fmt() }).collect() }
                    _ => return Err("UI_RADIO: options muss ARRAY OF STRING sein".into()),
                };
                let n = opts.len() as i64;
                if n == 0 { return Ok(Some(Value::Int(-1))); }
                if !self.ui_state.radio.contains_key(&id) {
                    let mut def = if a.len() >= 5 { gi(a,4,"UI_RADIO")? } else { 0 };
                    if def < 0 || def >= n { def = 0; }
                    self.ui_state.radio.insert(id.clone(), def);
                }
                let (mx,my,down) = self.ui_mouse_gated()?;
                let (row_h, radius) = (18i32, 5i32);
                for i in 0..n as i32 {
                    if ui_in_box(mx,my,x,y+i*row_h,200,row_h) && down && !self.ui_state.was_mouse_down {
                        self.ui_state.radio.insert(id.clone(), i as i64);
                    }
                }
                let sel = self.ui_state.radio[&id];
                let (fieldbg, accent, fg, border) = (self.ui_state.th("field_bg"), self.ui_state.th("accent"), self.ui_state.th("text_fg"), self.ui_state.th("field_border"));
                let g = self.gfx.as_mut().unwrap();
                for i in 0..n as i32 {
                    let cy = y + i*row_h + row_h/2; let cx = x + radius + 2;
                    g.circle(cx, cy, radius, fieldbg);
                    if i as i64 == sel { g.circle(cx,cy,radius,accent); g.circle(cx,cy,radius-2,fg); }
                    else { g.rect(cx-radius,cy-radius,cx+radius,cy+radius,border); }
                    g.text(x + 2*radius + 8, y + i*row_h + 2, opts[i as usize].clone(), fg);
                }
                Value::Int(sel)
            }
            "ui_textfield" => {
                let id = gs(a,0,"UI_TEXTFIELD")?.to_string();
                let x = gi(a,1,"UI_TEXTFIELD")? as i32 + self.ui_state.offset_x;
                let y = gi(a,2,"UI_TEXTFIELD")? as i32 + self.ui_state.offset_y;
                let w = gi(a,3,"UI_TEXTFIELD")? as i32;
                let h = gi(a,4,"UI_TEXTFIELD")? as i32;
                let placeholder = if a.len() >= 6 { gs(a,5,"UI_TEXTFIELD")?.to_string() } else { String::new() };
                self.ui_state.text.entry(id.clone()).or_insert_with(String::new);
                let blocked = self.ui_state.input_blocked;
                let (mx, my, down) = {
                    let g = self.gfx.as_ref().ok_or("UI_TEXTFIELD vor SCREEN")?;
                    let down = g.mouse_button(0);
                    if blocked { (-9999i32, -9999i32, down) } else { (g.mouse_x() as i32, g.mouse_y() as i32, down) }
                };
                let hovered = ui_in_box(mx, my, x, y, w, h);
                // Klick fokussiert / blurred
                if down && !self.ui_state.was_mouse_down {
                    if hovered { self.ui_state.focused = Some(id.clone()); }
                    else if self.ui_state.focused.as_deref() == Some(id.as_str()) { self.ui_state.focused = None; }
                }
                let is_focused = self.ui_state.focused.as_deref() == Some(id.as_str());
                if is_focused {
                    let typed = { let g = self.gfx.as_mut().ok_or("UI_TEXTFIELD vor SCREEN")?; g.pop_text_input() };
                    if !typed.is_empty() {
                        let clean: String = typed.chars().filter(|&c| c != '\t').collect();
                        self.ui_state.text.get_mut(&id).unwrap().push_str(&clean);
                    }
                    let bs = { let g = self.gfx.as_ref().ok_or("UI_TEXTFIELD vor SCREEN")?; g.key_down(8) };
                    if bs && !self.ui_state.prev_backspace {
                        self.ui_state.text.get_mut(&id).unwrap().pop();
                    }
                }
                // Zeichnen
                let border = if is_focused { self.ui_state.th("accent") } else { self.ui_state.th("field_border") };
                let (bg, fg, muted) = (self.ui_state.th("field_bg"), self.ui_state.th("text_fg"), self.ui_state.th("muted_fg"));
                let txt = self.ui_state.text[&id].clone();
                let caret_on = is_focused && (self.ui_state.frame_count / 16) % 2 == 0;
                {
                    let g = self.gfx.as_mut().ok_or("UI_TEXTFIELD vor SCREEN")?;
                    g.box_fill(x, y, x + w - 1, y + h - 1, bg);
                    g.rect(x, y, x + w - 1, y + h - 1, border);
                    if !txt.is_empty() {
                        g.text(x + 5, y + (h - 14) / 2, txt.clone(), fg);
                    } else if !placeholder.is_empty() && !is_focused {
                        g.text(x + 5, y + (h - 14) / 2, placeholder, muted);
                    }
                    if caret_on {
                        let mut cx = x + 5 + (txt.chars().count() as i32) * 8;
                        if cx > x + w - 3 { cx = x + w - 3; }
                        g.line(cx, y + 3, cx, y + h - 4, fg);
                    }
                }
                Value::str_rc(&txt)
            }
            "ui_textfield_set" => {
                let id = gs(a,0,"UI_TEXTFIELD_SET")?.to_string();
                let v = gs(a,1,"UI_TEXTFIELD_SET")?.to_string();
                self.ui_state.text.insert(id, v);
                Value::Nil
            }
            "ui_window_begin" => {
                let id = gs(a,0,"UI_WINDOW_BEGIN")?.to_string();
                let title = gs(a,1,"UI_WINDOW_BEGIN")?.to_string();
                let x = gi(a,2,"UI_WINDOW_BEGIN")? as i32;
                let y = gi(a,3,"UI_WINDOW_BEGIN")? as i32;
                let w = gi(a,4,"UI_WINDOW_BEGIN")? as i32;
                let h = gi(a,5,"UI_WINDOW_BEGIN")? as i32;
                let (mut wx, mut wy, mut collapsed) =
                    *self.ui_state.windows.entry(id.clone()).or_insert((x, y, false));
                // Fenster-Management nutzt IMMER die echte Maus (nie gated).
                let (mx, my, is_down) = {
                    let g = self.gfx.as_ref().ok_or("UI_WINDOW_BEGIN vor SCREEN")?;
                    (g.mouse_x() as i32, g.mouse_y() as i32, g.mouse_button(0))
                };
                let just_pressed = is_down && !self.ui_state.was_mouse_down;
                let owns = self.ui_state.active_win.is_none()
                    || self.ui_state.active_win.as_deref() == Some(id.as_str());
                let title_h = self.ui_state.metric("win_title_h") as i32;
                let full_h = if collapsed { title_h } else { h };
                if ui_in_box(mx, my, wx, wy, w, full_h) {
                    self.ui_state.hover_win = Some(id.clone());   // letzter Schreiber = oberstes
                }
                const COLLAPSE: i32 = 12;
                let cb_x = wx + 4;
                let cb_y = wy + (title_h - COLLAPSE) / 2;
                if owns && just_pressed {
                    if ui_in_box(mx, my, cb_x, cb_y, COLLAPSE, COLLAPSE) {
                        collapsed = !collapsed;
                    } else if ui_in_box(mx, my, wx, wy, w, title_h) {
                        self.ui_state.drag_win = Some(id.clone());
                        self.ui_state.drag_off = (mx - wx, my - wy);
                    }
                }
                if self.ui_state.drag_win.as_deref() == Some(id.as_str()) && is_down {
                    wx = mx - self.ui_state.drag_off.0;
                    wy = my - self.ui_state.drag_off.1;
                }
                self.ui_state.windows.insert(id.clone(), (wx, wy, collapsed));
                // Zeichnen
                let draw_h = if collapsed { title_h } else { h };
                let (win_bg, win_border) = (self.ui_state.th("win_bg"), self.ui_state.th("win_border"));
                let title_bg = if owns { self.ui_state.th("win_title_bg_focus") } else { self.ui_state.th("win_title_bg") };
                let fg = self.ui_state.th("text_fg");
                {
                    let g = self.gfx.as_mut().ok_or("UI_WINDOW_BEGIN vor SCREEN")?;
                    g.box_fill(wx, wy, wx + w - 1, wy + draw_h - 1, win_bg);
                    g.rect(wx, wy, wx + w - 1, wy + draw_h - 1, win_border);
                    g.box_fill(wx, wy, wx + w - 1, wy + title_h - 1, title_bg);
                    g.rect(wx, wy, wx + w - 1, wy + title_h - 1, win_border);
                    g.text(cb_x, cb_y - 2, (if collapsed { "+" } else { "-" }).to_string(), fg);
                    g.text(wx + COLLAPSE + 8, wy + 3, title, fg);
                }
                self.ui_state.win_stack.push((self.ui_state.offset_x, self.ui_state.offset_y, self.ui_state.input_blocked));
                self.ui_state.offset_x = wx;
                self.ui_state.offset_y = wy + title_h;
                self.ui_state.input_blocked = !owns;
                Value::Bool(!collapsed)
            }
            "ui_window_end" => {
                if let Some((ox, oy, blk)) = self.ui_state.win_stack.pop() {
                    self.ui_state.offset_x = ox;
                    self.ui_state.offset_y = oy;
                    self.ui_state.input_blocked = blk;
                }
                Value::Nil
            }
            "ui_table" => self.ui_table(a)?,
            "ui_table_selected" => {
                let id = gs(a,0,"UI_TABLE_SELECTED")?;
                Value::Int(self.ui_state.tables.get(id).map(|t| t.selected).unwrap_or(-1) as i64)
            }
            "ui_table_set_selected" => {
                let id = gs(a,0,"UI_TABLE_SET_SELECTED")?.to_string();
                let row = gi(a,1,"UI_TABLE_SET_SELECTED")?;
                if let Some(t) = self.ui_state.tables.get_mut(&id) { t.selected = if row >= 0 { row as i32 } else { -1 }; }
                Value::Nil
            }
            "ui_table_header_click" => {
                let id = gs(a,0,"UI_TABLE_HEADER_CLICK")?;
                Value::Int(self.ui_state.tables.get(id).map(|t| t.header_col).unwrap_or(-1) as i64)
            }
            "ui_end_frame" => {
                let (down, bs) = {
                    let g = self.gfx.as_ref().ok_or("UI_END_FRAME vor SCREEN")?;
                    (g.mouse_button(0), g.key_down(8))
                };
                if !down && self.ui_state.was_mouse_down {
                    self.ui_state.click_origin = None;
                    self.ui_state.drag_win = None;
                }
                self.ui_state.was_mouse_down = down;
                self.ui_state.prev_backspace = bs;
                self.ui_state.frame_count += 1;
                // Z-Order: oberstes gehovertes Fenster wird im NAECHSTEN Frame input-aktiv.
                self.ui_state.active_win = self.ui_state.hover_win.take();
                self.ui_state.offset_x = 0;
                self.ui_state.offset_y = 0;
                self.ui_state.input_blocked = false;
                self.ui_state.win_stack.clear();
                Value::Nil
            }
            "ui_reset" => {
                self.ui_state = UiState::new(); Value::Nil
            }
            "ui_theme_set" => {
                let k = gs(a,0,"UI_THEME_SET")?.to_string();
                if !self.ui_state.theme.contains_key(&k) { return Err(format!("UI_THEME_SET: unbekannter Schluessel '{}'", k)); }
                let c = gi(a,1,"UI_THEME_SET")?; if c < 0 || c > 0xFFFFFF { return Err("UI_THEME_SET: Farbe muss 0..0xFFFFFF sein".into()); }
                self.ui_state.theme.insert(k, c); Value::Nil
            }
            "ui_theme_get" => { let k = gs(a,0,"UI_THEME_GET")?; self.ui_state.theme.get(k).copied().map(Value::Int).ok_or_else(|| format!("UI_THEME_GET: unbekannter Schluessel '{}'", k))? }
            "ui_metric_set" => {
                let k = gs(a,0,"UI_METRIC_SET")?.to_string();
                if !self.ui_state.metrics.contains_key(&k) { return Err(format!("UI_METRIC_SET: unbekannter Schluessel '{}'", k)); }
                let v = gi(a,1,"UI_METRIC_SET")?; if v < 1 { return Err("UI_METRIC_SET: Wert muss >= 1 sein".into()); }
                self.ui_state.metrics.insert(k, v); Value::Nil
            }
            "ui_metric_get" => { let k = gs(a,0,"UI_METRIC_GET")?; self.ui_state.metrics.get(k).copied().map(Value::Int).ok_or_else(|| format!("UI_METRIC_GET: unbekannter Schluessel '{}'", k))? }
            "ui_theme_preset" => {
                let n = gs(a,0,"UI_THEME_PRESET")?.to_lowercase();
                let p = ui_preset(&n).ok_or_else(|| format!("UI_THEME_PRESET: unbekanntes Preset '{}'", n))?;
                for (k, v) in p { self.ui_state.theme.insert(k.to_string(), v); }
                Value::Nil
            }

            "particle_draw" => {
                let sys = match a.first() { Some(Value::Particles(p)) => p.clone(), _ => return Err("PARTICLE_DRAW erwartet PARTICLE_SYSTEM".into()) };
                let s = sys.borrow();
                let g = self.gfx.as_mut().ok_or("PARTICLE_DRAW vor SCREEN aufgerufen")?;
                let mode = s.mode.as_str();
                for p in &s.particles {
                    let col = particle_color(p.color, s.color_end, s.has_color_end, s.fade, p.age, p.lifetime);
                    let (x, y, sz) = (p.x as i32, p.y as i32, p.size);
                    match mode {
                        "pixel" => g.plot(x, y, col),
                        "square" => g.box_fill(x - sz, y - sz, x + sz, y + sz, col),
                        "streak" => g.line(x, y, (p.x - p.vx * 0.04) as i32, (p.y - p.vy * 0.04) as i32, col),
                        // glow additiv ist im Recording-Modell nicht direkt machbar -> Kreis-Approx.
                        _ => g.circle(x, y, sz.max(1), col),
                    }
                }
                Value::Nil
            }

            _ => return Ok(None), // kein Grafik-Builtin -> pure-Pfad
        };
        Ok(Some(r))
    }
}

// Vordefinierte Globals -- Werte IDENTISCH zu gamebasic/graphics.py
// (COLORS/KEYS). Von Hand synchron; Drift-Schutz: tests/test_constants_sync.py
// vergleicht jede Python-Konstante gegen PRINT-Output von gbrt.
const DEFAULT_COLORS: &[(&str, i64)] = &[
    ("black", 0), ("white", 16777215), ("gray", 8421504), ("lightgray", 12632256),
    ("darkgray", 4210752), ("red", 16711680), ("green", 65280), ("blue", 255),
    ("yellow", 16776960), ("cyan", 65535), ("magenta", 16711935), ("orange", 16753920),
    ("purple", 8388736), ("brown", 9127187), ("pink", 16761035), ("darkred", 8388608),
    ("darkgreen", 32768), ("darkblue", 128),
];

const DEFAULT_KEYS: &[(&str, i64)] = &[
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
    // Gamepad-Bind-Codes (negativ, kollidieren nicht mit Tasten) -- wie graphics.KEYS.
    ("joy_button_a", -100), ("joy_button_b", -101), ("joy_button_x", -102), ("joy_button_y", -103),
    ("joy_button_lb", -104), ("joy_button_rb", -105), ("joy_button_back", -106), ("joy_button_start", -107),
    ("joy_button_lstick", -108), ("joy_button_rstick", -109),
    ("joy_dpad_up", -200), ("joy_dpad_down", -201), ("joy_dpad_left", -202), ("joy_dpad_right", -203),
];

// ===========================================================================
// Helfer
// ===========================================================================

fn unknown_builtin_msg(name: &str) -> String {
    // Hardware-/IoT-Module sind hinter Cargo-Features (serial/usb/bt/wifi) und im
    // Default-Build NICHT enthalten -- der Dispatch faellt dann hierher durch. Das
    // Builtin EXISTIERT (in gbrt implementiert), es fehlt nur im aktuellen Build.
    // Klare, handlungsleitende Meldung statt "noch nicht verfuegbar".
    let hw_feature = if name.starts_with("serial_") { Some("serial") }
        else if name.starts_with("usb_") { Some("usb") }
        else if name.starts_with("bt_") { Some("bt") }
        else if name.starts_with("wifi_") { Some("wifi") }
        else { None };
    if let Some(feat) = hw_feature {
        return format!(
            "Builtin '{}' gehoert zum Hardware-Modul '{}', das in diesem gbrt-Build \
             fehlt. Neu bauen mit: python rust\\build_runtime.py --hardware",
            name.to_uppercase(), feat);
    }
    if builtins::is_graphics_builtin(name) {
        format!("Grafik-Builtin '{}' im Rust-Kern noch nicht verfuegbar (Schritt 4)", name.to_uppercase())
    } else {
        format!("Builtin '{}' im Rust-Kern noch nicht verfuegbar", name.to_uppercase())
    }
}

fn arg_truthy(a: &Arg) -> bool {
    match a {
        Arg::Int(i) => *i != 0,
        Arg::Val(Value::Bool(b)) => *b,
        Arg::None => false,
        _ => true,
    }
}

fn arg_value(a: &Arg) -> Value {
    match a {
        Arg::None => Value::Nil,
        Arg::Int(i) => Value::Int(*i),
        Arg::Str(s) => Value::Str(Rc::from(s.as_ref())),
        Arg::Val(v) => v.clone(),
        Arg::List(_) | Arg::Ints(_) | Arg::Call(..) => Value::Nil,
    }
}

fn container_kind(v: &Value) -> Option<&'static str> {
    match v {
        Value::Str(_) => Some("string"),
        Value::Array(_) => Some("array"),
        Value::Map(_) => Some("map"),
        Value::Tuple(_) => Some("tuple"),
        _ => None,
    }
}

fn container_method(kind: &str, method: &str) -> Option<&'static str> {
    Some(match (kind, method) {
        ("string", "upper") => "upper$", ("string", "lower") => "lower$",
        ("string", "length") | ("string", "len") => "len",
        ("string", "trim") => "trim$", ("string", "left") => "left$",
        ("string", "right") => "right$", ("string", "mid") => "mid$",
        ("string", "indexof") => "instr", ("string", "replace") => "replace$",
        ("string", "split") => "split$", ("string", "padl") => "padl$",
        ("string", "padr") => "padr$",
        ("array", "length") | ("array", "len") => "len",
        ("array", "sort") => "sort", ("array", "reverse") => "reverse",
        ("array", "indexof") => "array_indexof", ("array", "join") => "join$",
        ("map", "put") => "mapput", ("map", "get") => "mapget",
        ("map", "getor") => "mapgetor", ("map", "has") => "maphas",
        ("map", "keys") => "mapkeys",
        ("map", "size") | ("map", "length") | ("map", "len") => "mapsize",
        ("map", "remove") => "mapremove", ("map", "clear") => "mapclear",
        ("map", "values") => "mapvalues", ("map", "items") => "mapitems",
        ("tuple", "length") | ("tuple", "len") => "len",
        _ => return None,
    })
}

fn load_index(arr: &Value, idx_vals: &[Value]) -> R<Value> {
    match arr {
        Value::Array(a) => {
            let a = a.borrow();
            let mut ints = Vec::with_capacity(idx_vals.len());
            for ix in idx_vals {
                match ix { Value::Int(i) => ints.push(*i), v => return Err(format!("Array-Index muss INTEGER sein, erhalten {}", v.type_name())) }
            }
            let flat = a.flat_index(&ints)?;
            Ok(a.cells.get(flat))
        }
        Value::Str(s) => {
            if idx_vals.len() != 1 { return Err("String-Index braucht genau einen Wert".into()); }
            let ix = match &idx_vals[0] { Value::Int(i) => *i, v => return Err(format!("String-Index muss INTEGER sein, erhalten {}", v.type_name())) };
            let chars: Vec<char> = s.chars().collect();
            if ix < 0 || ix as usize >= chars.len() {
                return Err(format!("String-Index {} ausserhalb des Bereichs (Laenge {})", ix, chars.len()));
            }
            Ok(Value::str_rc(&chars[ix as usize].to_string()))
        }
        Value::Tuple(t) => {
            if idx_vals.len() != 1 { return Err("Tupel-Index braucht genau einen Wert".into()); }
            let ix = match &idx_vals[0] { Value::Int(i) => *i, v => return Err(format!("Tupel-Index muss INTEGER sein, erhalten {}", v.type_name())) };
            if ix < 0 || ix as usize >= t.len() {
                return Err(format!("Tupel-Index {} ausserhalb des Bereichs (Laenge {})", ix, t.len()));
            }
            Ok(t[ix as usize].clone())
        }
        Value::Nil => Err("Index-Zugriff auf NIL".into()),
        v => Err(format!("Index-Zugriff auf Nicht-Array ({})", v.type_name())),
    }
}

fn store_index(arr: &Value, idx_vals: &[Value], v: Value) -> R<()> {
    match arr {
        Value::Array(a) => {
            let mut a = a.borrow_mut();
            let mut ints = Vec::with_capacity(idx_vals.len());
            for ix in idx_vals {
                match ix { Value::Int(i) => ints.push(*i), x => return Err(format!("Array-Index muss INTEGER sein, erhalten {}", x.type_name())) }
            }
            let flat = a.flat_index(&ints)?;
            let cv = coerce(v, &a.element_type, "Array-Element")?;
            a.cells.set(flat, cv);
            Ok(())
        }
        Value::Nil => Err("Index-Zuweisung an NIL".into()),
        v => Err(format!("Index-Zuweisung an Nicht-Array ({})", v.type_name())),
    }
}

fn eval_in(needle: &Value, hay: &Value) -> R<bool> {
    match hay {
        Value::Str(s) => match needle {
            Value::Str(n) => Ok(s.contains(n.as_ref())),
            v => Err(format!("IN bei STRING: linke Seite muss STRING sein, erhalten {}", v.type_name())),
        },
        Value::Tuple(t) => Ok(t.iter().any(|x| value_eq(x, needle))),
        Value::Array(a) => {
            let a = a.borrow();
            if a.dims.len() != 1 { return Err("IN: nur 1D-Arrays".into()); }
            Ok(a.cells.iter().any(|x| value_eq(&x, needle)))
        }
        Value::Map(m) => match needle {
            Value::Str(n) => Ok(m.borrow().get(n).is_some()),
            v => Err(format!("IN bei MAP: Key muss STRING sein, erhalten {}", v.type_name())),
        },
        Value::Nil => Err("IN: rechte Seite ist NIL".into()),
        v => Err(format!("IN: rechte Seite muss STRING, TUPLE, ARRAY oder MAP sein, erhalten {}", v.type_name())),
    }
}

fn apply_slice(target: &Value, lo: Option<&Value>, hi: Option<&Value>) -> R<Value> {
    let to_idx = |o: Option<&Value>, def: i64| -> R<i64> {
        match o {
            None => Ok(def),
            Some(Value::Int(i)) => {
                if *i < 0 { Err("Slice: negativer Index".into()) } else { Ok(*i) }
            }
            Some(v) => Err(format!("Slice-Index muss INTEGER sein, erhalten {}", v.type_name())),
        }
    };
    match target {
        Value::Str(s) => {
            let chars: Vec<char> = s.chars().collect();
            let n = chars.len() as i64;
            let a = to_idx(lo, 0)?.min(n).max(0);
            let b = to_idx(hi, n)?.min(n).max(0);
            if a >= b { return Ok(Value::str_rc("")); }
            Ok(Value::str_rc(&chars[a as usize..b as usize].iter().collect::<String>()))
        }
        Value::Array(arr) => {
            let arr = arr.borrow();
            if arr.dims.len() != 1 { return Err("Slicing nur fuer 1D-Arrays".into()); }
            let n = arr.dims[0];
            let a = to_idx(lo, 0)?.min(n).max(0);
            let b = to_idx(hi, n)?.min(n).max(0);
            let slice = if a >= b { Cells::Val(vec![]) } else { arr.cells.slice(a as usize, b as usize) };
            let len = slice.len() as i64;
            let mut new = GbArray::new(arr.element_type.clone(), vec![len], || Value::Nil);
            new.cells = slice;
            Ok(Value::Array(Rc::new(RefCell::new(new))))
        }
        v => Err(format!("Slice-Zugriff: Erwartet STRING oder ARRAY, erhalten {}", v.type_name())),
    }
}

fn scene_type_ok(v: &Value, name: &str) -> bool {
    if name.contains("int") { matches!(v, Value::Int(_)) }
    else if name.contains("float") { matches!(v, Value::Float(_)) }
    else if name.contains("string") { matches!(v, Value::Str(_)) }
    else if name.contains("bool") { matches!(v, Value::Bool(_)) }
    else { false }
}

fn infer_type(v: &Value) -> &'static str {
    match v {
        Value::Bool(_) => "boolean",
        Value::Int(_) => "integer",
        Value::Float(_) => "float",
        Value::Str(_) => "string",
        _ => "any",
    }
}

fn require_number(a: &Value, b: &Value, op: &str) -> R<()> {
    if !is_num(a) || !is_num(b) {
        return Err(format!("Operator '{}' erwartet Zahlen, erhalten {} / {}", op, a.type_name(), b.type_name()));
    }
    Ok(())
}

fn int_pair2(stack: &mut Vec<Value>) -> R<(i64, i64)> {
    let b = vm_pop(stack)?;
    let a = vm_pop(stack)?;
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => Ok((*x, *y)),
        _ => Err("Bitwise erwartet INTEGER".into()),
    }
}

/// Partikel-Farbe pro Lebenszeit: optional Start->End-Verlauf + Fade.
#[cfg(feature = "graphics")]
fn particle_color(start: i64, end: i64, has_end: bool, fade: bool, age: i32, lifetime: i32) -> i64 {
    if !has_end && !fade { return start; }
    let life_t = (age as f64 / lifetime.max(1) as f64).clamp(0.0, 1.0);
    let mut sr = ((start >> 16) & 0xFF) as f64;
    let mut sg = ((start >> 8) & 0xFF) as f64;
    let mut sb = (start & 0xFF) as f64;
    if has_end {
        let (er, eg, eb) = (((end >> 16) & 0xFF) as f64, ((end >> 8) & 0xFF) as f64, (end & 0xFF) as f64);
        let inv = 1.0 - life_t;
        sr = sr * inv + er * life_t; sg = sg * inv + eg * life_t; sb = sb * inv + eb * life_t;
    }
    // FADE senkt jetzt das ALPHA (statt RGB Richtung Schwarz zu verdunkeln) ->
    // funktioniert auch im additiven Glow-Modus und auf hellem Hintergrund.
    // col() liest das obere Byte als Alpha; 0 = deckend, daher min. 1 (so wird
    // ein fast erloschener Partikel transparent, nicht ploetzlich deckend).
    let alpha: i64 = if fade { (((1.0 - life_t) * 255.0).round().clamp(1.0, 255.0)) as i64 } else { 255 };
    (alpha << 24)
        | ((sr.clamp(0.0, 255.0) as i64) << 16)
        | ((sg.clamp(0.0, 255.0) as i64) << 8)
        | (sb.clamp(0.0, 255.0) as i64)
}

/// Modul-Operator-Dispatch (entspricht `modules.dispatch_binary_op`): vec2 +
/// die m3d-Typen (VEC3/VEC4/QUAT/MAT4). Liefert None, wenn kein Operand ein
/// Modul-Typ ist (Standard-Pfad). Mat4-/Quat-Mathe teilt sich die puren
/// Helfer aus `builtins` (eine Quelle).
fn module_op(op: char, a: &Value, b: &Value) -> Option<R<Value>> {
    let is_mod = |v: &Value| matches!(v,
        Value::Vec2(..) | Value::Vec3(..) | Value::Vec4(..) | Value::Quat(..) | Value::Mat4(_));
    if !is_mod(a) && !is_mod(b) {
        return None;
    }
    let sf = |n: &Value| as_f64(n) as f32;     // Skalar als f32
    Some(match op {
        '+' => match (a, b) {
            (Value::Vec2(ax, ay), Value::Vec2(bx, by)) => Ok(Value::Vec2(ax + bx, ay + by)),
            (Value::Vec3(ax, ay, az), Value::Vec3(bx, by, bz)) => Ok(Value::Vec3(ax + bx, ay + by, az + bz)),
            (Value::Vec4(ax, ay, az, aw), Value::Vec4(bx, by, bz, bw)) => Ok(Value::Vec4(ax + bx, ay + by, az + bz, aw + bw)),
            _ => Err("+ : inkompatible Vektor-Operanden (gleicher Vektor-Typ erwartet)".into()),
        },
        '-' => match (a, b) {
            (Value::Vec2(ax, ay), Value::Vec2(bx, by)) => Ok(Value::Vec2(ax - bx, ay - by)),
            (Value::Vec3(ax, ay, az), Value::Vec3(bx, by, bz)) => Ok(Value::Vec3(ax - bx, ay - by, az - bz)),
            (Value::Vec4(ax, ay, az, aw), Value::Vec4(bx, by, bz, bw)) => Ok(Value::Vec4(ax - bx, ay - by, az - bz, aw - bw)),
            _ => Err("- : inkompatible Vektor-Operanden (gleicher Vektor-Typ erwartet)".into()),
        },
        '*' => match (a, b) {
            (Value::Vec2(x, y), n) if is_num(n) => Ok(Value::Vec2(x * as_f64(n), y * as_f64(n))),
            (n, Value::Vec2(x, y)) if is_num(n) => Ok(Value::Vec2(x * as_f64(n), y * as_f64(n))),
            (Value::Vec3(x, y, z), n) if is_num(n) => Ok(Value::Vec3(x * sf(n), y * sf(n), z * sf(n))),
            (n, Value::Vec3(x, y, z)) if is_num(n) => Ok(Value::Vec3(x * sf(n), y * sf(n), z * sf(n))),
            (Value::Vec4(x, y, z, w), n) if is_num(n) => Ok(Value::Vec4(x * sf(n), y * sf(n), z * sf(n), w * sf(n))),
            (n, Value::Vec4(x, y, z, w)) if is_num(n) => Ok(Value::Vec4(x * sf(n), y * sf(n), z * sf(n), w * sf(n))),
            (Value::Quat(ax, ay, az, aw), Value::Quat(bx, by, bz, bw)) => {
                let (x, y, z, w) = crate::builtins::m3_quat_mul((*ax, *ay, *az, *aw), (*bx, *by, *bz, *bw));
                Ok(Value::Quat(x, y, z, w))
            }
            (Value::Mat4(m), Value::Mat4(n)) => Ok(Value::Mat4(Rc::new(crate::builtins::m3_mul(m, n)))),
            (Value::Mat4(m), Value::Vec4(x, y, z, w)) => {
                let (rx, ry, rz, rw) = crate::builtins::m3_transform4(m, *x, *y, *z, *w);
                Ok(Value::Vec4(rx, ry, rz, rw))
            }
            (Value::Mat4(m), Value::Vec3(x, y, z)) => {
                let (rx, ry, rz) = crate::builtins::m3_transform_point(m, *x, *y, *z);
                Ok(Value::Vec3(rx, ry, rz))
            }
            _ => Err("* : nicht unterstuetzte Operanden (Vektor*Zahl, QUAT*QUAT, MAT4*MAT4/VEC4/VEC3)".into()),
        },
        '/' => match (a, b) {
            (Value::Vec2(x, y), n) if is_num(n) => {
                let d = as_f64(n);
                if d == 0.0 { Err("Division durch 0".into()) } else { Ok(Value::Vec2(x / d, y / d)) }
            }
            (Value::Vec3(x, y, z), n) if is_num(n) => {
                let d = sf(n);
                if d == 0.0 { Err("Division durch 0".into()) } else { Ok(Value::Vec3(x / d, y / d, z / d)) }
            }
            (Value::Vec4(x, y, z, w), n) if is_num(n) => {
                let d = sf(n);
                if d == 0.0 { Err("Division durch 0".into()) } else { Ok(Value::Vec4(x / d, y / d, z / d, w / d)) }
            }
            _ => Err("/ : erwartet Vektor / Zahl".into()),
        },
        _ => return None,
    })
}

fn int_overflow_msg(op: &str) -> String {
    format!("Ganzzahl-Ueberlauf bei '{}' (INTEGER ist 64-bit; Wertebereich ueberschritten)", op)
}

fn nn_add(a: Value, b: Value) -> R<Value> {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) =>
            x.checked_add(*y).map(Value::Int).ok_or_else(|| int_overflow_msg("+")),
        _ => Ok(Value::Float(as_f64(&a) + as_f64(&b))),
    }
}

fn nn_arith(a: Value, b: Value, op: char) -> R<Value> {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            let r = match op { '-' => x.checked_sub(*y), '*' => x.checked_mul(*y), _ => unreachable!() };
            r.map(Value::Int).ok_or_else(|| int_overflow_msg(&op.to_string()))
        }
        _ => { let (x, y) = (as_f64(&a), as_f64(&b)); Ok(Value::Float(match op { '-' => x - y, '*' => x * y, _ => unreachable!() })) }
    }
}

fn mul(a: Value, b: Value) -> R<Value> {
    match (&a, &b) {
        (Value::Str(s), Value::Int(n)) | (Value::Int(n), Value::Str(s)) => {
            return Ok(if *n > 0 { Value::str_rc(&s.repeat(*n as usize)) } else { Value::str_rc("") });
        }
        _ => {}
    }
    require_number(&a, &b, "*")?;
    nn_arith(a, b, '*')
}

fn div(a: Value, b: Value) -> R<Value> {
    require_number(&a, &b, "/")?;
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            if *y == 0 { return Err("Division durch 0".into()); }
            if x % y == 0 { Ok(Value::Int(x / y)) } else { Ok(Value::Float(*x as f64 / *y as f64)) }
        }
        _ => { let y = as_f64(&b); if y == 0.0 { return Err("Division durch 0".into()); } Ok(Value::Float(as_f64(&a) / y)) }
    }
}

fn int_div(a: Value, b: Value) -> R<Value> {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            if *y == 0 { return Err("Integer-Division durch 0".into()); }
            Ok(Value::Int(x / y))
        }
        _ => Err("\\ erwartet INTEGER (kein FLOAT)".into()),
    }
}

fn modulo(a: Value, b: Value) -> R<Value> {
    require_number(&a, &b, "MOD")?;
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            if *y == 0 { return Err("MOD durch 0".into()); }
            let m = x % y;
            let m = if m != 0 && (m < 0) != (*y < 0) { m + y } else { m };
            Ok(Value::Int(m))
        }
        _ => { let (x, y) = (as_f64(&a), as_f64(&b)); if y == 0.0 { return Err("MOD durch 0".into()); } Ok(Value::Float(x - y * (x / y).floor())) }
    }
}

fn pow(a: Value, b: Value) -> R<Value> {
    require_number(&a, &b, "^")?;
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) if *y >= 0 => {
            if *y > u32::MAX as i64 { return Err(int_overflow_msg("^")); }
            x.checked_pow(*y as u32).map(Value::Int).ok_or_else(|| int_overflow_msg("^"))
        }
        _ => Ok(Value::Float(as_f64(&a).powf(as_f64(&b)))),
    }
}

fn neg(v: Value) -> R<Value> {
    match v {
        Value::Int(i) => i.checked_neg().map(Value::Int).ok_or_else(|| int_overflow_msg("-")),
        Value::Float(f) => Ok(Value::Float(-f)),
        _ => Err("Unaeres '-' erwartet Zahl".into()),
    }
}

fn cmp(a: &Value, b: &Value, op: char) -> R<bool> {
    let ord = if is_num(a) && is_num(b) {
        as_f64(a).partial_cmp(&as_f64(b))
    } else if let (Value::Str(x), Value::Str(y)) = (a, b) {
        Some(x.as_ref().cmp(y.as_ref()))
    } else {
        return Err(format!("Vergleich nicht moeglich: {} / {}", a.type_name(), b.type_name()));
    };
    let ord = ord.ok_or("Vergleich mit NaN")?;
    use std::cmp::Ordering::*;
    Ok(match op { '<' => ord == Less, '>' => ord == Greater, 'l' => ord != Greater, 'g' => ord != Less, _ => unreachable!() })
}

/// Prompt-Aufbereitung wie interpreter._exec_Input: leer -> "? ", sonst mit
/// Leerzeichen-Suffix.
fn input_prompt(p: &str) -> String {
    if p.is_empty() {
        "? ".to_string()
    } else if p.ends_with(' ') {
        p.to_string()
    } else {
        format!("{} ", p)
    }
}

/// Eine Zeile von stdin lesen; Zeilenende entfernen (wie Python input()).
/// EOF -> leerer String.
fn read_input_line() -> String {
    use std::io::BufRead;
    let mut line = String::new();
    let _ = std::io::stdin().lock().read_line(&mut line);
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }
    line
}

/// INPUT-Rohwert auf den Ziel-Typ coercen (Semantik 1:1 aus interpreter).
fn coerce_input(raw: &str, ty: &str) -> R<Value> {
    match ty {
        "integer" => raw.trim().parse::<i64>().map(Value::Int)
            .map_err(|_| format!("Eingabe '{}' passt nicht zu INTEGER", raw)),
        "float" => raw.trim().parse::<f64>().map(Value::Float)
            .map_err(|_| format!("Eingabe '{}' passt nicht zu FLOAT", raw)),
        "string" => Ok(Value::str_rc(raw)),
        "boolean" => Ok(Value::Bool(matches!(
            raw.trim().to_lowercase().as_str(), "true" | "wahr" | "yes" | "ja" | "1"))),
        _ => Err(format!("Unbekannter Typ: {}", ty)),
    }
}

fn coerce(value: Value, target: &str, ctx: &str) -> R<Value> {
    match target {
        "" | "any" => Ok(value),
        "integer" => match value {
            Value::Int(_) => Ok(value),
            Value::Float(f) => {
                if f.fract() == 0.0 { Ok(Value::Int(f as i64)) }
                else { Err(format!("{}: FLOAT {} kann nicht ohne Verlust nach INTEGER (nutze INT())", ctx, f)) }
            }
            Value::Bool(_) => Err(format!("{}: Erwartet INTEGER, erhalten BOOLEAN", ctx)),
            _ => Err(format!("{}: Erwartet INTEGER, erhalten {}", ctx, value.type_name())),
        },
        "float" => match value {
            Value::Float(_) => Ok(value),
            Value::Int(i) => Ok(Value::Float(i as f64)),
            Value::Bool(_) => Err(format!("{}: Erwartet FLOAT, erhalten BOOLEAN", ctx)),
            _ => Err(format!("{}: Erwartet FLOAT, erhalten {}", ctx, value.type_name())),
        },
        "string" => match value {
            Value::Str(_) => Ok(value),
            _ => Err(format!("{}: Erwartet STRING, erhalten {}", ctx, value.type_name())),
        },
        "boolean" => match value {
            Value::Bool(_) => Ok(value),
            _ => Err(format!("{}: Erwartet BOOLEAN, erhalten {}", ctx, value.type_name())),
        },
        // Pascal-Striktheit (gbrt-Haertung): TUPLE/FUNCREF nur mit passendem
        // Wert zuweisbar. Nil bleibt erlaubt (uninitialisierter DECLARE-Default
        // umgeht coerce ohnehin, aber explizite Nil-Zuweisung soll nicht crashen).
        "tuple" => match value {
            Value::Tuple(_) | Value::Nil => Ok(value),
            _ => Err(format!("{}: Erwartet TUPLE, erhalten {}", ctx, value.type_name())),
        },
        "funcref" => match value {
            Value::FuncRef(_) | Value::Nil => Ok(value),
            _ => Err(format!("{}: Erwartet FUNCREF, erhalten {}", ctx, value.type_name())),
        },
        // array:/map:/Klassen/sonstige -> Durchreichen (Referenz-Typen).
        _ => Ok(value),
    }
}
