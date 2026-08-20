//! VM-Kern (Schritt 2+3): stack-basierte Dispatch-Schleife.
//!
//! Skalar-Ops, Control-Flow, User-Calls, Strings/Arrays/Maps/Tupel, OOP
//! (Structs/Klassen/Methoden/Properties/Operator-Overloading), Slicing, IN,
//! DATA/READ, TRY/THROW und die puren Builtins (siehe builtins.rs).
//! Semantik 1:1 aus `drachenhauch/vm.py`, damit `stdout` bit-identisch bleibt.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
/// exec), es gibt kein GB-seitiges Frame-Limit -- ohne diese Grenze crasht eine
/// unendliche Rekursion den ganzen Prozess statt einen fangbaren Fehler zu werfen.
const MAX_CALL_DEPTH: u32 = 3000;

use crate::builtins;
use crate::model::{op, Arg, ClassInfo, Func, Program};
use crate::value::{as_f64, is_num, value_eq, Cells, CoroState, FieldVal, GbArray, GbMap, Instance, Value};

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

// --- Debugger (Stufe B, `dhrt debug`) ---------------------------------------
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
    // Review-Fund: TRY/CATCH-Bypass + `was_stopped()` verglichen bisher den
    // FEHLERTEXT gegen "__DEBUG_STOP__"/PROFILE_STOP -- ein GB-Programm mit
    // `THROW "__DEBUG_STOP__"` konnte diese internen Kontroll-Signale also
    // faelschlich vortaeuschen (TRY/CATCH faengt es dann nie, main.rs meldet
    // "gestoppt" statt den echten Fehler). Diese beiden Flags sind der
    // eigentliche Signalkanal -- THROW beruehrt sie nie, ein Stringinhalt
    // kann sie also nicht mehr faelschen.
    debug_stop_flag: bool,
    profile_stop_flag: bool,
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
    /// Auftraege, die eine EIGENE GB-Funktion ausfuehren (`TASK_*`).
    task_auftraege: crate::hintergrund::Auftraege<Result<crate::hintergrund::TaskErgebnis, String>>,
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
    #[cfg(feature = "serial")]
    serial_ports: Vec<Option<crate::serial::Port>>,
    #[cfg(feature = "serial")]
    firmata_boards: Vec<Option<crate::firmata::Board>>,
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
    "gesetzt -- auch keine CONST auf oberster Ebene. Gib der Funktion als ",
    "Parameter mit, was sie braucht.");

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
            debug_stop_flag: false,
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
            task_auftraege: Default::default(),
            shell_letzter_code: 0,
            shell_letzter_fehler: String::new(),
            log_pegel: None,
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
            #[cfg(feature = "serial")]
            serial_ports: Vec::new(),
            #[cfg(feature = "serial")]
            firmata_boards: Vec::new(),
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
    /// damit GB-Code nicht ueber eine Thread-Grenze muss (`Value` haelt
    /// ueberall `Rc`, `Program` ist weder Send noch Sync -- siehe
    /// docs/entwurf-task-start.md).
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
    /// GB-Programm mit `THROW "__PROFILE_STOP__"` haette einen echten Fehler
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
            // Mathe-Typen bekommen ihr NEUTRALES Element statt NIL -- genauso,
            // wie INTEGER mit 0 und STRING mit "" anfaengt. Ohne das ist ein
            // frisches `DIM m AS MAT4` unbrauchbar (jede Rechnung darauf
            // scheitert an NIL), und ein `DIM mats[N] AS MAT4` laesst sich
            // nicht schrittweise fuellen. Eine Quelle fuer alle drei Wege
            // (global, lokal, Array-Element): model::neutrales_element.
            other => crate::model::neutrales_element(other).unwrap_or(Value::Nil),
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

    /// Debugging fuer den naechsten `run()` aktivieren (`dhrt debug`).
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
                None => { self.debug_stop_flag = true; return Err("__DEBUG_STOP__".into()); }
            };
            match cmd.get("cmd").and_then(|v| v.as_str()).unwrap_or("") {
                "continue"  => { dbg.step = StepMode::Run; return Ok(()); }
                "step-over" => { dbg.step = StepMode::Over; dbg.step_depth = depth; return Ok(()); }
                "step-into" => { dbg.step = StepMode::Into; return Ok(()); }
                "step-out"  => { dbg.step = StepMode::Out; dbg.step_depth = depth; return Ok(()); }
                "stop"      => { self.debug_stop_flag = true; return Err("__DEBUG_STOP__".into()); }
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
        // Review-Fund: ohne Obergrenze rekursiert exec->run_frame->dispatch->exec
        // auf dem NATIVEN Stack -- eine ausufernde GB-Rekursion (oder eine
        // Property/Operator, die sich selbst aufruft) fuehrte zu einem
        // Stack-Overflow-Absturz statt einem fangbaren DHRuntimeError.
        self.depth += 1;
        if self.depth > MAX_CALL_DEPTH {
            self.depth -= 1;
            return Err(format!("Maximale Aufruftiefe ({}) ueberschritten -- unendliche Rekursion?", MAX_CALL_DEPTH));
        }
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
        if self.depth > MAX_CALL_DEPTH {
            self.depth -= 1;
            return Err(format!("Maximale Aufruftiefe ({}) ueberschritten -- unendliche Rekursion?", MAX_CALL_DEPTH));
        }
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
        loop {
            match self.dispatch(fn_, locals, stack, ip, try_handlers, self_obj) {
                Ok(step) => return Ok(step),
                // Debugger-Abbruch (`stop`), Profiler-Stop-Signal und EXIT(code)
                // duerfen NICHT von TRY/CATCH gefangen werden -- unbedingt
                // durchreichen. Entscheidend ist das Flag, NICHT der Fehlertext:
                // ein GB-Programm mit `THROW "__DEBUG_STOP__"` erzeugt zwar
                // denselben String, setzt aber keines der Flags (Review-Fund).
                Err(e) if self.debug_stop_flag || self.profile_stop_flag
                          || self.exit_code.is_some() => return Err(e),
                Err(e) => {
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
                        // WP F: Hier -- und nur hier -- sieht die VM einen
                        // FRISCHEN Fehler. Also der richtige Ort, um Fundstelle
                        // und Code fuer ERROR_LINE()/ERROR_CODE$() festzuhalten.
                        // Ein Fehler, der nicht von THROW kam, loescht den Code;
                        // sonst klebte er von einem frueheren THROW noch an.
                        self.error_line = self.cur_line;
                        self.error_code = if self.throw_active {
                            std::mem::take(&mut self.throw_code)
                        } else {
                            String::new()
                        };
                        self.throw_active = false;
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
                                let s = self.global_slots[var_idx].as_ref().ok_or(GLOBAL_UNGESETZT)?;
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
                    let s = self.global_slots[arg.as_usize()].as_ref().ok_or(GLOBAL_UNGESETZT)?;
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
                            else if let Some(r) = self.user_op("__op_add__", &a, &b, true)? { stack.push(r); }
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
                op::POW => { let b = vm_pop(stack)?; let a = vm_pop(stack)?; stack.push(pow(a, b)?); }
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
                        if let Some(v) = self.try_array_hof(name, bargs)? { v }
                        else if let Some(v) = self.try_scene(name, bargs)? { v }
                        else if let Some(v) = self.try_coro(name, bargs)? { v }
                        else if let Some(v) = self.try_timer(name, bargs)? { v }
                        else if let Some(v) = self.try_zeit(name, bargs)? { v }
                        else if let Some(v) = self.try_os(name, bargs)? { v }
                        else if let Some(v) = self.try_hintergrund(name, bargs)? { v }
                        else if let Some(v) = self.try_db(name, bargs)? { v }
                        else if let Some(v) = self.try_net(name, bargs)? { v }
                        else if let Some(v) = self.try_mqtt(name, bargs)? { v }
                        else if let Some(v) = self.try_html(name, bargs)? { v }
                        else if let Some(v) = self.try_cloud(name, bargs)? { v }
                        else if let Some(v) = self.try_serial(name, bargs)? { v }
                        else if let Some(v) = self.try_firmata(name, bargs)? { v }
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

                // `SUPER.Methode(...)` (WP G). Fast wie CALL_METHOD -- nur
                // beginnt die Suche bei der im Bytecode stehenden Klasse
                // (der Elternklasse der Aufrufstelle) statt bei der Klasse
                // des Objekts. Sonst fande sie die ueberschreibende Methode
                // wieder und riefe sich selbst, bis der Stapel voll ist.
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
        // Review-Fund: GbArray::new multipliziert die Dimensionen ungeprueft
        // (`acc *= dims[k]`) -- `DIM a[4294967296, 4294967296]` ueberlief den
        // i64-Akkumulator lautlos zu einem kleinen/negativen Wert (eine
        // Groessen-Diskrepanz, die spaeter beim Indexzugriff in einen rohen
        // Index-Out-Of-Bounds-Panic lief), und ein legitimer, aber riesiger
        // Wert wie `DIM a[100000, 100000]` (1e10 Elemente, ~80 GB) fuehrte zu
        // einem harten Allocator-Abort statt eines GB-Fehlers. Hier einmalig
        // mit checked_mul + einer Obergrenze validieren, BEVOR GbArray::new
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
    #[cfg(any(feature = "db", feature = "net", feature = "serial", feature = "usb", feature = "bt"))]
    fn handle_get_mut<'a, T>(vec: &'a mut [Option<T>], idx: i64, modul: &str, was: &str) -> R<&'a mut T> {
        vec.get_mut(idx as usize).and_then(|o| o.as_mut())
            .ok_or_else(|| format!("{}: {} {}", modul, was, idx))
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
        Self::handle_get(&self.db_conns, idx, "DB", "ungueltiges/geschlossenes DB_CONN-Handle")
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
            "db_close_result" => {
                let r = self.db_res_mut(bi_int(a, 0, "DB_CLOSE_RESULT")?)?;
                r.closed = true;
                // rows leert sich NICHT von selbst durch das closed-Flag -- ohne
                // das hier bleiben eager geladene Zeilen bis Programmende im
                // Speicher, auch wenn artig geschlossen wird.
                r.rows = Vec::new();
                Value::Nil
            }
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
        Self::handle_get(&self.tcp_listeners, idx, "NET", "ungueltiges/geschlossenes NET_LISTENER-Handle")
    }
    #[cfg(feature = "net")]
    fn net_sock(&self, idx: i64) -> R<&crate::net::NetSock> {
        Self::handle_get(&self.tcp_socks, idx, "NET", "ungueltiges/geschlossenes NET_SOCKET-Handle")
    }
    #[cfg(feature = "net")]
    fn net_sock_mut(&mut self, idx: i64) -> R<&mut crate::net::NetSock> {
        Self::handle_get_mut(&mut self.tcp_socks, idx, "NET", "ungueltiges/geschlossenes NET_SOCKET-Handle")
    }
    #[cfg(feature = "net")]
    fn net_udp(&self, idx: i64) -> R<&crate::net::UdpSock> {
        Self::handle_get(&self.udp_socks, idx, "NET", "ungueltiges/geschlossenes NET_UDP-Handle")
    }
    #[cfg(feature = "net")]
    fn net_udp_mut(&mut self, idx: i64) -> R<&mut crate::net::UdpSock> {
        Self::handle_get_mut(&mut self.udp_socks, idx, "NET", "ungueltiges/geschlossenes NET_UDP-Handle")
    }

    #[cfg(feature = "net")]
    fn try_net_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::net;
        let v = match name {
            "net_tcp_listen" => {
                let bind_addr = match a.get(1) { Some(Value::Str(s)) => s.as_ref(), _ => "" };
                let (l, port) = net::listen(bi_int(a, 0, "NET_TCP_LISTEN")?, bind_addr)?;
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
            "net_is_connected" => Value::Bool(net::is_connected(self.net_sock(bi_int(a, 0, "NET_IS_CONNECTED")?)?)),
            "net_close" => { let i = bi_int(a, 0, "NET_CLOSE")? as usize; if let Some(s) = self.tcp_socks.get_mut(i) { *s = None; } Value::Nil }
            "net_close_listener" => { let i = bi_int(a, 0, "NET_CLOSE_LISTENER")? as usize; if let Some(s) = self.tcp_listeners.get_mut(i) { *s = None; } Value::Nil }
            "net_set_timeout" => { let i = bi_int(a, 0, "NET_SET_TIMEOUT")?; let ms = bi_int(a, 1, "NET_SET_TIMEOUT")?; net::set_timeout_tcp(&self.net_sock(i)?.stream, ms); Value::Nil }
            "net_udp_bind" => {
                let bind_addr = match a.get(1) { Some(Value::Str(s)) => s.as_ref(), _ => "" };
                let s = net::udp_bind(bi_int(a, 0, "NET_UDP_BIND")?, bind_addr)?;
                self.udp_socks.push(Some(s));
                Value::Int((self.udp_socks.len() - 1) as i64)
            }
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
    // Modul mqtt (baut auf std::net auf, Feature `net`)
    // ===================================================================
    fn try_mqtt(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("mqtt_")) { return Ok(None); }
        #[cfg(feature = "net")]
        { return self.try_mqtt_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }
    #[cfg(feature = "net")]
    fn mqtt_client(&mut self, idx: i64) -> R<&mut crate::mqtt::Client> {
        Self::handle_get_mut(&mut self.mqtt_clients, idx, "MQTT", "ungueltiges/geschlossenes MQTT_HANDLE")
    }
    #[cfg(feature = "net")]
    fn try_mqtt_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::mqtt;
        let v = match name {
            "mqtt_connect" => {
                let host = bi_str(a, 0, "MQTT_CONNECT")?.to_string();
                let port = bi_int(a, 1, "MQTT_CONNECT")?;
                let client_id = bi_str(a, 2, "MQTT_CONNECT")?.to_string();
                let keepalive = if a.len() >= 4 { bi_int(a, 3, "MQTT_CONNECT")? } else { 60 };
                let username = match a.get(4) { Some(Value::Str(s)) => Some(s.to_string()), _ => None };
                let password = match a.get(5) { Some(Value::Str(s)) => Some(s.to_string()), _ => None };
                let c = mqtt::connect(&host, port, &client_id, keepalive, username.as_deref(), password.as_deref())?;
                self.mqtt_clients.push(Some(c));
                Value::Int((self.mqtt_clients.len() - 1) as i64)
            }
            "mqtt_disconnect" => {
                let i = bi_int(a, 0, "MQTT_DISCONNECT")? as usize;
                if let Some(slot) = self.mqtt_clients.get_mut(i) {
                    if let Some(c) = slot.as_mut() { mqtt::disconnect(c); }
                    *slot = None;
                }
                Value::Nil
            }
            "mqtt_is_connected" => { let i = bi_int(a, 0, "MQTT_IS_CONNECTED")?; Value::Bool(self.mqtt_clients.get(i as usize).and_then(|o| o.as_ref()).map(mqtt::is_connected).unwrap_or(false)) }
            "mqtt_publish" => {
                let i = bi_int(a, 0, "MQTT_PUBLISH")?; let topic = bi_str(a, 1, "MQTT_PUBLISH")?.to_string(); let payload = bi_str(a, 2, "MQTT_PUBLISH")?.to_string();
                let retain = if a.len() >= 4 { bi_bool(a, 3, "MQTT_PUBLISH")? } else { false };
                mqtt::publish(self.mqtt_client(i)?, &topic, &payload, retain)?; Value::Nil
            }
            "mqtt_subscribe" => { let i = bi_int(a, 0, "MQTT_SUBSCRIBE")?; let topic = bi_str(a, 1, "MQTT_SUBSCRIBE")?.to_string(); mqtt::subscribe(self.mqtt_client(i)?, &topic)?; Value::Nil }
            "mqtt_update" => { let i = bi_int(a, 0, "MQTT_UPDATE")?; mqtt::update(self.mqtt_client(i)?)?; Value::Nil }
            "mqtt_next_message" => { let i = bi_int(a, 0, "MQTT_NEXT_MESSAGE")?; Value::Bool(mqtt::next_message(self.mqtt_client(i)?)) }
            "mqtt_message_topic" => { let i = bi_int(a, 0, "MQTT_MESSAGE_TOPIC")?; Value::str_rc(&mqtt::message_topic(self.mqtt_client(i)?)) }
            "mqtt_message_payload" => { let i = bi_int(a, 0, "MQTT_MESSAGE_PAYLOAD")?; Value::str_rc(&mqtt::message_payload(self.mqtt_client(i)?)) }
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

    /// Eine HTTP-Antwort in den VM-Zustand uebernehmen und als STRING liefern.
    ///
    /// Frueher stand dieser Block viermal wortgleich in `try_html_impl`. Mit
    /// `HTTP_BYTES` (WP C) kam eine fuenfte Sache dazu, die JEDER Pfad tun
    /// muss -- und genau so vergisst man sie in einem davon.
    #[cfg(feature = "http")]
    fn http_antwort(&mut self, r: Result<crate::html::HttpResult, crate::html::HttpErr>) -> R<Value> {
        match r {
            Ok(r) => {
                self.http_status = r.status;
                self.http_headers = r.headers;
                let text = String::from_utf8_lossy(&r.body).into_owned();
                self.http_body = r.body;      // roh, fuer HTTP_BYTES()
                Ok(Value::str_rc(&text))
            }
            Err(e) => {
                if e.status != 0 { self.http_status = e.status; self.http_headers = e.headers; }
                // Sonst gehoerte der Rumpf der VORIGEN Antwort und HTTP_BYTES
                // lieferte nach einem Fehlschlag alte Daten als neue aus.
                self.http_body.clear();
                Err(e.msg)
            }
        }
    }

    /// Baut aus `(methode, url [, rumpf [, kopfzeilen]])` eine Anfrage.
    /// Gemeinsam fuer HTTP_REQUEST und HTTP_REQUEST_START.
    #[cfg(feature = "http")]
    fn http_anfrage(&self, fn_: &str, a: &[Value]) -> R<crate::html::Anfrage> {
        if a.len() < 2 || a.len() > 4 {
            return Err(format!(
                "{}: erwartet 2..4 Argumente (methode, url [, rumpf [, kopfzeilen]]), erhalten {}",
                fn_, a.len()));
        }
        let mut anfrage = crate::html::Anfrage::neu(bi_str(a, 0, fn_)?, bi_str(a, 1, fn_)?);
        anfrage.timeout = self.http_timeout;
        // Rumpf: Text ODER Bytes. NIL/weggelassen = kein Rumpf.
        anfrage.body = match a.get(2) {
            None | Some(Value::Nil) => Vec::new(),
            Some(Value::Str(s)) => s.as_bytes().to_vec(),
            Some(Value::Buffer(b)) => b.borrow().clone(),
            Some(v) => return Err(format!(
                "{}: Rumpf erwartet STRING oder BUFFER, erhalten {}", fn_, v.type_name())),
        };
        // Erst die dauerhaften Kopfzeilen, dann die des Aufrufs -- gleiche
        // Namen ueberschreiben, der Aufruf gewinnt also gegen HTTP_SET_HEADER.
        anfrage.header = self.http_default_header.clone();
        match a.get(3) {
            None | Some(Value::Nil) => {}
            Some(Value::Map(m)) => {
                for (k, v) in m.borrow().entries().iter() {
                    match v {
                        Value::Str(s) => anfrage.header.push((k.clone(), s.to_string())),
                        andere => return Err(format!(
                            "{}: Kopfzeile '{}' erwartet STRING, erhalten {}",
                            fn_, k, andere.type_name())),
                    }
                }
            }
            Some(v) => return Err(format!(
                "{}: Kopfzeilen erwarten MAP OF STRING, erhalten {}", fn_, v.type_name())),
        }
        Ok(anfrage)
    }

    #[cfg(feature = "http")]
    fn try_html_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::html;
        let v = match name {
            "http_get" => {
                let mut anfrage = html::Anfrage::neu("GET", bi_str(a, 0, "HTTP_GET")?);
                anfrage.timeout = self.http_timeout;
                anfrage.header = self.http_default_header.clone();
                let r = html::http_request(&anfrage);
                self.http_antwort(r)?
            }
            "http_post" => {
                let mut anfrage = html::Anfrage::neu("POST", bi_str(a, 0, "HTTP_POST")?);
                anfrage.timeout = self.http_timeout;
                anfrage.body = bi_str(a, 1, "HTTP_POST")?.as_bytes().to_vec();
                anfrage.header = self.http_default_header.clone();
                anfrage.header.push(("Content-Type".to_string(),
                                     "application/x-www-form-urlencoded; charset=utf-8".to_string()));
                let r = html::http_request(&anfrage);
                self.http_antwort(r)?
            }
            // --- WP C: eine Anfrage, wie sie eine echte API braucht ---
            "http_request" => {
                let anfrage = self.http_anfrage("HTTP_REQUEST", a)?;
                let r = html::http_request(&anfrage);
                self.http_antwort(r)?
            }
            "http_request_start" => {
                let anfrage = self.http_anfrage("HTTP_REQUEST_START", a)?;
                Value::Int(self.http_abrufe.start(anfrage))
            }
            "http_bytes" => {
                if !a.is_empty() { return Err(format!("HTTP_BYTES: erwartet 0 Argumente, erhalten {}", a.len())); }
                Value::Buffer(Rc::new(RefCell::new(self.http_body.clone())))
            }
            "http_timeout" => {
                let s = bi_int(a, 0, "HTTP_TIMEOUT")?;
                // Obergrenze, damit ein vertipptes HTTP_TIMEOUT(600000) das
                // Programm nicht faktisch fuer immer haengen laesst.
                if !(1..=600).contains(&s) {
                    return Err(format!("HTTP_TIMEOUT: {} Sekunden liegen ausserhalb 1..600", s));
                }
                self.http_timeout = s as u64;
                Value::Nil
            }
            "http_set_header" => {
                let k = bi_str(a, 0, "HTTP_SET_HEADER")?.to_string();
                let v = bi_str(a, 1, "HTTP_SET_HEADER")?.to_string();
                // Sofort pruefen statt erst beim naechsten Aufruf: der Fehler
                // gehoert an die Zeile, in der die Kopfzeile gesetzt wird.
                html::pruefe_header(&k, &v)?;
                // Gleicher Name ersetzt -- zweimal denselben Namen zu schicken
                // waere sonst das Ergebnis, und welcher gilt, entschiede der
                // Server.
                if let Some(e) = self.http_default_header.iter_mut().find(|(n, _)| n.eq_ignore_ascii_case(&k)) {
                    e.1 = v;
                } else {
                    self.http_default_header.push((k, v));
                }
                Value::Nil
            }
            "http_clear_headers" => { self.http_default_header.clear(); Value::Nil }
            "http_download" => {
                let url = bi_str(a, 0, "HTTP_DOWNLOAD")?.to_string();
                let path = bi_str(a, 1, "HTTP_DOWNLOAD")?.to_string();
                match html::http_download(&url, &path) {
                    Ok(r) => { self.http_status = r.status; self.http_headers = r.headers; Value::Int(r.bytes) }
                    Err(e) => { if e.status != 0 { self.http_status = e.status; self.http_headers = e.headers; } return Err(e.msg); }
                }
            }
            // --- Abrufe im Hintergrund: starten, nachsehen, abholen ---
            "http_get_start" => {
                let mut anfrage = html::Anfrage::neu("GET", bi_str(a, 0, "HTTP_GET_START")?);
                anfrage.timeout = self.http_timeout;
                anfrage.header = self.http_default_header.clone();
                Value::Int(self.http_abrufe.start(anfrage))
            }
            "http_ready" => Value::Bool(self.http_abrufe.fertig(bi_int(a, 0, "HTTP_READY")?)),
            "http_result" => {
                let id = bi_int(a, 0, "HTTP_RESULT")?;
                match self.http_abrufe.abholen(id) {
                    Some(r) => self.http_antwort(r)?,
                    // Abholen ohne vorheriges HTTP_READY ist der haeufigste
                    // Anfaengerfehler -- deshalb sagt die Meldung, was fehlt.
                    None => return Err(format!(
                        "HTTP_RESULT: Abruf {} ist nicht fertig (oder schon abgeholt) \
                         -- erst HTTP_READY({}) abfragen", id, id)),
                }
            }
            "http_cancel" => { self.http_abrufe.abbrechen(bi_int(a, 0, "HTTP_CANCEL")?); Value::Nil }
            "http_pending" => Value::Int(self.http_abrufe.offen()),
            "http_url$" | "http_url" => Value::str_rc(&self.http_abrufe.url(bi_int(a, 0, "HTTP_URL$")?)),

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
    // Modul cloud (Cloud-Save + Leaderboard gegen cloudserver/server.py, Feature `http`)
    // ===================================================================
    fn try_cloud(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("cloud_") || name.starts_with("leaderboard_")) { return Ok(None); }
        #[cfg(feature = "http")]
        { return self.try_cloud_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    #[cfg(feature = "http")]
    fn cloud_require_configured(&self, ctx: &str) -> R<()> {
        if self.cloud_base_url.is_empty() {
            return Err(format!("{}: CLOUD_CONFIGURE(base_url, api_key) muss zuerst aufgerufen werden", ctx));
        }
        Ok(())
    }

    #[cfg(feature = "http")]
    fn try_cloud_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::cloud;
        fn bi_bool(a: &[Value], i: usize, fn_: &str) -> R<bool> {
            match a.get(i) {
                Some(Value::Bool(b)) => Ok(*b),
                Some(v) => Err(format!("{}: erwartet BOOLEAN, erhalten {}", fn_, v.type_name())),
                None => Err(format!("{}: fehlendes Argument {}", fn_, i + 1)),
            }
        }
        let v = match name {
            "cloud_configure" => {
                self.cloud_base_url = bi_str(a, 0, "CLOUD_CONFIGURE")?.to_string();
                self.cloud_api_key = bi_str(a, 1, "CLOUD_CONFIGURE")?.to_string();
                Value::Nil
            }
            "cloud_save" => {
                self.cloud_require_configured("CLOUD_SAVE")?;
                let player_id = bi_str(a, 0, "CLOUD_SAVE")?.to_string();
                let data = bi_str(a, 1, "CLOUD_SAVE")?.to_string();
                self.cloud_last_error.clear();
                match cloud::save_upload(&self.cloud_base_url, &self.cloud_api_key, &player_id, &data) {
                    Ok(()) => Value::Bool(true),
                    Err(e) => { self.cloud_last_error = e.msg; Value::Bool(false) }
                }
            }
            "cloud_load" | "cloud_load$" => {
                self.cloud_require_configured("CLOUD_LOAD")?;
                let player_id = bi_str(a, 0, "CLOUD_LOAD")?.to_string();
                self.cloud_last_error.clear();
                match cloud::save_download(&self.cloud_base_url, &self.cloud_api_key, &player_id) {
                    Ok(Some(data)) => Value::str_rc(&data),
                    Ok(None) => Value::str_rc(""),
                    Err(e) => { self.cloud_last_error = e.msg; Value::str_rc("") }
                }
            }
            "cloud_last_error$" | "cloud_last_error" => Value::str_rc(&self.cloud_last_error),
            "leaderboard_submit" => {
                self.cloud_require_configured("LEADERBOARD_SUBMIT")?;
                let board = bi_str(a, 0, "LEADERBOARD_SUBMIT")?.to_string();
                let pname = bi_str(a, 1, "LEADERBOARD_SUBMIT")?.to_string();
                let score = bi_num(a, 2, "LEADERBOARD_SUBMIT")?;
                let best_low = if a.len() >= 4 { bi_bool(a, 3, "LEADERBOARD_SUBMIT")? } else { false };
                self.cloud_last_error.clear();
                match cloud::leaderboard_submit(&self.cloud_base_url, &self.cloud_api_key, &board, &pname, score, best_low) {
                    Ok(updated) => Value::Bool(updated),
                    Err(e) => { self.cloud_last_error = e.msg; Value::Bool(false) }
                }
            }
            "leaderboard_fetch" => {
                self.cloud_require_configured("LEADERBOARD_FETCH")?;
                let board = bi_str(a, 0, "LEADERBOARD_FETCH")?.to_string();
                let n = bi_int(a, 1, "LEADERBOARD_FETCH")?;
                let ascending = if a.len() >= 3 { bi_bool(a, 2, "LEADERBOARD_FETCH")? } else { false };
                self.cloud_last_error.clear();
                let entries = match cloud::leaderboard_fetch(&self.cloud_base_url, &self.cloud_api_key, &board, n, ascending) {
                    Ok(e) => e,
                    Err(e) => { self.cloud_last_error = e.msg; Vec::new() }
                };
                let items: Vec<Value> = entries.into_iter()
                    .map(|e| Value::Tuple(Rc::new(vec![Value::str_rc(&e.name), Value::Float(e.score)])))
                    .collect();
                let cnt = items.len() as i64;
                let mut arr = GbArray::new("tuple".to_string(), vec![cnt], || Value::Tuple(Rc::new(vec![])));
                for (i, v) in items.into_iter().enumerate() { arr.cells.set(i, v); }
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
        Self::handle_get_mut(&mut self.serial_ports, idx, "SERIAL", "ungueltiges/geschlossenes SERIAL_HANDLE")
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
    // Modul firmata (Feature `serial`, keine eigene Cargo-Abhaengigkeit)
    // ===================================================================
    fn try_firmata(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("firmata_")) { return Ok(None); }
        #[cfg(feature = "serial")]
        { return self.try_firmata_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }
    #[cfg(feature = "serial")]
    fn firmata_board(&mut self, idx: i64) -> R<&mut crate::firmata::Board> {
        Self::handle_get_mut(&mut self.firmata_boards, idx, "FIRMATA", "ungueltiges/geschlossenes FIRMATA_HANDLE")
    }
    #[cfg(feature = "serial")]
    fn try_firmata_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::firmata;
        let v = match name {
            // Dieselbe Port-Liste wie SERIAL_PORTS() (serialport::available_ports) --
            // eigener Name, damit ein firmata-only-Script nicht extra IMPORT "serial" braucht.
            "firmata_ports" => Value::str_rc(&crate::serial::ports()),
            "firmata_open" => {
                let b = firmata::open(bi_str(a, 0, "FIRMATA_OPEN")?, bi_int(a, 1, "FIRMATA_OPEN")?)?;
                self.firmata_boards.push(Some(b));
                Value::Int((self.firmata_boards.len() - 1) as i64)
            }
            "firmata_close" => { let i = bi_int(a, 0, "FIRMATA_CLOSE")? as usize; if let Some(s) = self.firmata_boards.get_mut(i) { *s = None; } Value::Nil }
            "firmata_is_open" => { let i = bi_int(a, 0, "FIRMATA_IS_OPEN")?; Value::Bool(self.firmata_boards.get(i as usize).map(|o| o.is_some()).unwrap_or(false)) }
            "firmata_pin_mode" => {
                let i = bi_int(a, 0, "FIRMATA_PIN_MODE")?; let pin = bi_int(a, 1, "FIRMATA_PIN_MODE")?; let mode = bi_int(a, 2, "FIRMATA_PIN_MODE")?;
                firmata::pin_mode(self.firmata_board(i)?, pin, mode)?; Value::Nil
            }
            "firmata_digital_write" => {
                let i = bi_int(a, 0, "FIRMATA_DIGITAL_WRITE")?; let pin = bi_int(a, 1, "FIRMATA_DIGITAL_WRITE")?; let val = bi_bool(a, 2, "FIRMATA_DIGITAL_WRITE")?;
                firmata::digital_write(self.firmata_board(i)?, pin, val)?; Value::Nil
            }
            "firmata_digital_read" => {
                let i = bi_int(a, 0, "FIRMATA_DIGITAL_READ")?; let pin = bi_int(a, 1, "FIRMATA_DIGITAL_READ")?;
                Value::Bool(firmata::digital_read(self.firmata_board(i)?, pin)?)
            }
            "firmata_analog_write" => {
                let i = bi_int(a, 0, "FIRMATA_ANALOG_WRITE")?; let pin = bi_int(a, 1, "FIRMATA_ANALOG_WRITE")?; let val = bi_int(a, 2, "FIRMATA_ANALOG_WRITE")?;
                firmata::analog_write(self.firmata_board(i)?, pin, val)?; Value::Nil
            }
            "firmata_analog_read" => {
                let i = bi_int(a, 0, "FIRMATA_ANALOG_READ")?; let ch = bi_int(a, 1, "FIRMATA_ANALOG_READ")?;
                Value::Int(firmata::analog_read(self.firmata_board(i)?, ch)?)
            }
            "firmata_update" => { let i = bi_int(a, 0, "FIRMATA_UPDATE")?; firmata::update(self.firmata_board(i)?)?; Value::Nil }
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
        Self::handle_get(&self.usb_devs, idx, "USB", "ungueltiges/geschlossenes USB_HANDLE")
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
        { return self.try_wifi_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    #[cfg(feature = "wifi")]
    fn try_wifi_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
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
        Ok(Some(v))
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
        Self::handle_get(&self.bt_periphs, idx, "BT", "ungueltiges/getrenntes BT_HANDLE")
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

    /// Modul `zeit` (zeit.rs): mit Datum und Uhrzeit rechnen.
    ///
    /// Der Zeitwert ist eine ganze Zahl: Sekunden seit 1970 in ORTSZEIT --
    /// so passen `ZEIT_JETZT()` und ein aus der Datenbank gelesener Anstoss
    /// ohne Zeitzonen-Umrechnung zusammen. Alles hier ist reine Mathematik
    /// (die Tests dazu stehen in zeit.rs); nur JETZT fragt die Uhr.
    fn try_zeit(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("zeit_") { return Ok(None); }
        use crate::zeit;

        fn z_int(a: &[Value], i: usize, fn_: &str) -> R<i64> {
            match a.get(i) {
                Some(Value::Int(n)) => Ok(*n),
                Some(Value::Float(f)) => Ok(*f as i64),
                _ => Err(format!("{}: erwartet einen Zeitwert (INTEGER, Arg {})", fn_, i + 1)),
            }
        }
        fn z_str<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) {
                Some(Value::Str(s)) => Ok(s),
                _ => Err(format!("{}: erwartet STRING (Arg {})", fn_, i + 1)),
            }
        }

        let v = match name {
            "zeit_jetzt" => {
                let (j, mo, d, h, mi, s) = crate::builtins::local_datetime();
                Value::Int(zeit::aus_teilen(j, mo, d, h, mi, s))
            }
            "zeit_parse" => {
                let text = z_str(a, 0, "ZEIT_PARSE")?;
                match zeit::parse(text) {
                    Some(t) => Value::Int(t),
                    // Klartext statt stiller -1: ein unlesbares Datum ist fast
                    // immer ein Fehler in den Daten, und der faellt sonst erst
                    // viel spaeter als unsinnige Rechnung auf.
                    None => return Err(format!(
                        "ZEIT_PARSE: '{}' ist kein Zeitpunkt -- erwartet \
                         'JJJJ-MM-TT hh:mm:ss' (auch mit T, ohne Sekunden \
                         oder nur Datum)", text)),
                }
            }
            // ZEIT_TEXT$ gibt Text, ZEIT_LESBAR fragt nach. Getrennte Namen,
            // weil "lesbar" sich als beides lesen laesst -- ich habe die
            // zwei beim Schreiben der ersten Pruefung selbst verwechselt.
            "zeit_text$" | "zeit_text" =>
                Value::str_rc(&zeit::format(z_int(a, 0, "ZEIT_TEXT$")?, "")),
            "zeit_lesbar" => Value::Bool(zeit::parse(z_str(a, 0, "ZEIT_LESBAR")?).is_some()),
            "zeit_format$" | "zeit_format" => {
                let t = z_int(a, 0, "ZEIT_FORMAT$")?;
                let muster = if a.len() > 1 { z_str(a, 1, "ZEIT_FORMAT$")? } else { "" };
                Value::str_rc(&zeit::format(t, muster))
            }
            "zeit_teil" => {
                let t = z_int(a, 0, "ZEIT_TEIL")?;
                let was = z_str(a, 1, "ZEIT_TEIL")?;
                match zeit::teil(t, was) {
                    Some(n) => Value::Int(n),
                    None => return Err(format!(
                        "ZEIT_TEIL: '{}' ist kein Feld -- moeglich sind {}", was, zeit::TEILE)),
                }
            }
            "zeit_aus_teilen" => {
                Value::Int(zeit::aus_teilen(
                    z_int(a, 0, "ZEIT_AUS_TEILEN")?, z_int(a, 1, "ZEIT_AUS_TEILEN")?,
                    z_int(a, 2, "ZEIT_AUS_TEILEN")?,
                    if a.len() > 3 { z_int(a, 3, "ZEIT_AUS_TEILEN")? } else { 0 },
                    if a.len() > 4 { z_int(a, 4, "ZEIT_AUS_TEILEN")? } else { 0 },
                    if a.len() > 5 { z_int(a, 5, "ZEIT_AUS_TEILEN")? } else { 0 }))
            }
            "zeit_plus" => Value::Int(
                z_int(a, 0, "ZEIT_PLUS")?.saturating_add(z_int(a, 1, "ZEIT_PLUS")?)),
            "zeit_diff" => Value::Int(
                z_int(a, 0, "ZEIT_DIFF")?.saturating_sub(z_int(a, 1, "ZEIT_DIFF")?)),
            "zeit_dauer$" | "zeit_dauer" => Value::str_rc(&zeit::dauer(z_int(a, 0, "ZEIT_DAUER$")?)),
            "zeit_wochentag" => Value::Int(zeit::wochentag(z_int(a, 0, "ZEIT_WOCHENTAG")?)),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Betriebssystem-Builtins, die VM-Zustand brauchen (WP A). Die
    /// zustandsfreien (`ARGC`/`ARG$`/`GETENV$`/`SETENV`/`CWD$`/`CHDIR`) stehen
    /// in `builtins.rs`.
    ///
    /// Gemeinsamer Grund, warum diese vier hier stehen: **`PRINT` wird
    /// gepuffert** (`self.out`, geschrieben erst am Programmende). Wer daneben
    /// auf stderr schreibt oder ein Kindprogramm auf dasselbe Terminal laufen
    /// laesst, saehe die Ausgaben sonst in falscher Reihenfolge -- die eigenen
    /// PRINTs kaemen ganz zum Schluss. Darum flusht jeder dieser Befehle den
    /// Puffer zuerst.
    fn try_os(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if let Some(v) = self.try_pruefen(name, a)? { return Ok(Some(v)); }
        if !matches!(name, "exit" | "eprint" | "shell" | "shell_out$" | "shell_out") { return Ok(None); }
        fn o_str<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) { Some(Value::Str(s)) => Ok(s), _ => Err(format!("{}: erwartet STRING (Arg {})", fn_, i + 1)) }
        }
        let v = match name {
            "exit" => {
                if a.len() > 1 { return Err(format!("EXIT: erwartet 0..1 Argument(e), erhalten {}", a.len())); }
                let code = match a.first() {
                    None => 0,
                    Some(Value::Int(n)) => *n,
                    _ => return Err("EXIT: erwartet INTEGER (Rueckgabewert)".into()),
                };
                // Betriebssysteme uebertragen nur das untere Byte eines
                // Rueckgabewerts. Statt `EXIT(256)` still zu 0 werden zu lassen
                // -- also aus "Fehler" ein "alles gut" zu machen -- ist das ein
                // Fehler mit Ansage.
                if !(0..=255).contains(&code) {
                    return Err(format!("EXIT: Rueckgabewert {} liegt ausserhalb 0..255", code));
                }
                self.exit_code = Some(code as i32);
                return Err(EXIT_REQUEST.into());
            }
            "eprint" => {
                if a.len() != 1 { return Err(format!("EPRINT: erwartet 1 Argument, erhalten {}", a.len())); }
                let text = crate::builtins::str_of(&a[0]);
                self.flush_out();
                use std::io::Write;
                let se = std::io::stderr();
                let mut h = se.lock();
                let _ = writeln!(h, "{}", text);
                let _ = h.flush();
                Value::Nil
            }
            "shell" | "shell_out$" | "shell_out" => {
                let anzeige = if name == "shell" { "SHELL" } else { "SHELL_OUT$" };
                if a.is_empty() { return Err(format!("{}: erwartet mind. 1 Argument (Programm)", anzeige)); }
                let prog = o_str(a, 0, anzeige)?.to_string();
                // Argumente EINZELN statt als eine Kommandozeile -- damit gibt
                // es keine Quoting-Regeln zu lernen und keine Shell, die aus
                // einem Dateinamen mit Leerzeichen zwei Argumente macht. Wer
                // wirklich eine Shell will, ruft sie ausdruecklich auf
                // (SHELL("cmd", "/c", "dir | more")).
                let mut rest: Vec<String> = Vec::new();
                for i in 1..a.len() { rest.push(o_str(a, i, anzeige)?.to_string()); }
                let mut cmd = std::process::Command::new(&prog);
                cmd.args(&rest);
                self.flush_out();
                if name == "shell" {
                    // Erbt stdout/stderr -- das Kind schreibt direkt aufs
                    // Terminal, ohne Umweg ueber unseren Puffer.
                    let st = cmd.status().map_err(|e| format!("SHELL: '{}' laesst sich nicht starten: {}", prog, e))?;
                    // Kein Rueckgabewert (Unix: durch ein Signal beendet) ->
                    // -1. Das ist eindeutig, weil ein echter Rueckgabewert
                    // immer 0..255 ist.
                    Value::Int(st.code().map(|c| c as i64).unwrap_or(-1))
                } else {
                    let out = cmd.output().map_err(|e| format!("SHELL_OUT$: '{}' laesst sich nicht starten: {}", prog, e))?;
                    // stderr des Kindes bleibt stderr (durchgereicht), nur
                    // stdout ist das Ergebnis -- sonst mischten sich
                    // Fehlermeldungen unbemerkt in die Nutzdaten.
                    use std::io::Write;
                    let se = std::io::stderr();
                    let _ = se.lock().write_all(&out.stderr);
                    Value::str_rc(&String::from_utf8_lossy(&out.stdout))
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Hintergrund-Auftraege (WP H): SHELL_START, DB_QUERY_START und TASK_START.
    ///
    /// Dasselbe Muster ueberall -- starten, pro Bild nachsehen, abholen.
    /// `shell_`/`db_query_` laufen als reine Rust-Arbeit ohne VM. `task_`
    /// fuehrt dagegen GB-Code aus, und zwar in einem EIGENEN dhrt-Prozess:
    /// im Thread ginge es nicht, weil `Value` ueberall `Rc` haelt und
    /// `Program` damit weder Send noch Sync ist.
    fn try_hintergrund(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("shell_") || name.starts_with("db_query_")
             || name.starts_with("task_")) { return Ok(None); }
        let v = match name {
            "shell_start" => {
                if a.is_empty() { return Err("SHELL_START: erwartet mind. 1 Argument (Programm)".into()); }
                let prog = bi_str(a, 0, "SHELL_START")?.to_string();
                let mut rest: Vec<String> = Vec::new();
                for i in 1..a.len() { rest.push(bi_str(a, i, "SHELL_START")?.to_string()); }
                Value::Int(self.shell_auftraege.start(&prog.clone(),
                    move || crate::hintergrund::shell_arbeit(prog, rest)))
            }
            // ===== Auftraege mit eigener GB-Funktion (WP H, Weg C) =====
            // Der Auftrag laeuft als eigener dhrt-Prozess, nicht als Thread:
            // `Value` haelt ueberall `Rc`, `Program` ist weder Send noch Sync.
            // Die Grenze ist zugleich die Zusage -- ein Auftrag sieht KEINE
            // Globals des Hauptprogramms, auch keine CONST. Er bekommt mit,
            // was er braucht. Siehe docs/entwurf-task-start.md.
            "task_start" => {
                if a.is_empty() {
                    return Err("TASK_START: erwartet eine Funktion, z.B. TASK_START(Rechne, 42)".into());
                }
                let funktion = match &a[0] {
                    Value::FuncRef(n) => n.to_string(),
                    Value::Str(s) => s.to_string(),
                    andere => return Err(format!(
                        "TASK_START: erwartet eine Funktion als erstes Argument, \
                         bekommen {}. Schreib den Namen ohne Klammern: \
                         TASK_START(Rechne, 42)", andere.type_name())),
                };
                let datei = match crate::builtins::quelldatei() {
                    Some(d) => d.clone(),
                    None => return Err(
                        "TASK_START: die laufende Datei ist unbekannt -- \
                         Auftraege gibt es nur bei `dhrt run <datei.dh>`".into()),
                };
                let exe = std::env::current_exe().map_err(|e| format!(
                    "TASK_START: eigenen Programmpfad nicht gefunden: {}", e))?;
                let arg = match a.get(1) {
                    None => None,
                    Some(Value::Int(i)) => Some(i.to_string()),
                    Some(Value::Str(s)) => Some(s.to_string()),
                    Some(andere) => return Err(format!(
                        "TASK_START: das Argument geht ueber eine Prozessgrenze \
                         und muss INTEGER oder STRING sein, nicht {}. Fuer mehr \
                         reich JSON durch.", andere.type_name())),
                };
                let was = funktion.clone();
                Value::Int(self.task_auftraege.start(&was,
                    move || crate::hintergrund::task_arbeit(exe, datei, funktion, arg)))
            }
            "task_ready" => Value::Bool(self.task_auftraege.fertig(bi_int(a, 0, "TASK_READY")?)?),
            // Einmal abholbar, wie bei SHELL_RESULT$: `abholen` nimmt das
            // Ergebnis aus der Verwaltung. Darum gibt es KEIN zweites
            // TASK_OUTPUT$ daneben -- zwei Abholer wuerden sich gegenseitig
            // das Ergebnis wegnehmen, je nachdem wer zuerst fragt. Was der
            // Auftrag gedruckt hat, geht damit verloren: ein Auftrag rechnet,
            // er redet nicht.
            "task_result$" | "task_result" => {
                let id = bi_int(a, 0, "TASK_RESULT$")?;
                match self.task_auftraege.abholen(id)? {
                    Some(Ok(e)) => Value::str_rc(&e.ergebnis),
                    Some(Err(e)) => return Err(e),
                    None => return Err(format!(
                        "TASK_RESULT$: Auftrag {} ist noch nicht fertig -- erst \
                         TASK_READY({}) fragen", id, id)),
                }
            }
            "task_cancel" => { self.task_auftraege.abbrechen(bi_int(a, 0, "TASK_CANCEL")?); Value::Nil }
            "task_pending" => Value::Int(self.task_auftraege.offen()),

            "shell_ready" => Value::Bool(self.shell_auftraege.fertig(bi_int(a, 0, "SHELL_READY")?)?),
            "shell_result$" | "shell_result" => {
                let id = bi_int(a, 0, "SHELL_RESULT$")?;
                match self.shell_auftraege.abholen(id)? {
                    Some(Ok(e)) => {
                        self.shell_letzter_code = e.code;
                        self.shell_letzter_fehler = e.stderr;
                        Value::str_rc(&e.stdout)
                    }
                    Some(Err(msg)) => {
                        self.shell_letzter_code = -1;
                        self.shell_letzter_fehler = msg.clone();
                        return Err(msg);
                    }
                    // Abholen ohne vorheriges SHELL_READY ist der haeufigste
                    // Anfaengerfehler -- die Meldung sagt, was fehlt.
                    None => return Err(format!(
                        "SHELL_RESULT$: Auftrag {} ist nicht fertig (oder schon abgeholt)                          -- erst SHELL_READY({}) abfragen", id, id)),
                }
            }
            "shell_code" => {
                if !a.is_empty() { return Err("SHELL_CODE: erwartet 0 Argumente (gilt fuer den zuletzt abgeholten Auftrag)".into()); }
                Value::Int(self.shell_letzter_code)
            }
            "shell_err$" | "shell_err" => {
                if !a.is_empty() { return Err("SHELL_ERR$: erwartet 0 Argumente (gilt fuer den zuletzt abgeholten Auftrag)".into()); }
                Value::str_rc(&self.shell_letzter_fehler)
            }
            "shell_cancel" => { self.shell_auftraege.abbrechen(bi_int(a, 0, "SHELL_CANCEL")?); Value::Nil }
            "shell_pending" => Value::Int(self.shell_auftraege.offen()),
            _ => return self.try_hintergrund_db(name, a),
        };
        Ok(Some(v))
    }

    #[cfg(not(feature = "db"))]
    fn try_hintergrund_db(&mut self, _name: &str, _a: &[Value]) -> R<Option<Value>> { Ok(None) }

    /// Abfragen im Hintergrund (WP H).
    ///
    /// Der Auftrag oeffnet eine EIGENE Verbindung zur Datei, statt die des
    /// Programms mitzunehmen: eine `rusqlite::Connection` ist `Send`, aber
    /// nicht `Sync` -- sie mitzugeben hiesse, sie dem Hauptthread wegzunehmen,
    /// und der soll ja weiterarbeiten koennen. Preis dafuer: der Auftrag sieht
    /// nur, was schon festgeschrieben ist, nicht die offene Transaktion des
    /// Programms. Steht so in der Doku.
    #[cfg(feature = "db")]
    fn try_hintergrund_db(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        let v = match name {
            "db_query_start" => {
                let datei = bi_str(a, 0, "DB_QUERY_START")?.to_string();
                let sql = bi_str(a, 1, "DB_QUERY_START")?.to_string();
                let params = db_params(a.get(2..).unwrap_or(&[]), "DB_QUERY_START")?;
                let kurz = sql.chars().take(40).collect::<String>();
                Value::Int(self.db_auftraege.start(&kurz, move || {
                    let conn = rusqlite::Connection::open(&datei)
                        .map_err(|e| format!("DB_QUERY_START: {} ({})", e, datei))?;
                    crate::db::query(&conn, &sql, &params)
                }))
            }
            "db_query_ready" => Value::Bool(self.db_auftraege.fertig(bi_int(a, 0, "DB_QUERY_READY")?)?),
            "db_query_result" => {
                let id = bi_int(a, 0, "DB_QUERY_RESULT")?;
                match self.db_auftraege.abholen(id)? {
                    Some(Ok(res)) => {
                        self.db_results.push(res);
                        Value::Int((self.db_results.len() - 1) as i64)
                    }
                    Some(Err(msg)) => return Err(msg),
                    None => return Err(format!(
                        "DB_QUERY_RESULT: Auftrag {} ist nicht fertig (oder schon abgeholt)                          -- erst DB_QUERY_READY({}) abfragen", id, id)),
                }
            }
            "db_query_cancel" => { self.db_auftraege.abbrechen(bi_int(a, 0, "DB_QUERY_CANCEL")?); Value::Nil }
            "db_query_pending" => Value::Int(self.db_auftraege.offen()),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Pruefen und Melden (WP E): ASSERT-Familie und LOG_*.
    ///
    /// Braucht VM-Zustand (Zaehler, Sammel-Modus, Quell-Zeile, Ausgabe-Puffer)
    /// und steht darum hier statt in `builtins.rs`.
    fn try_pruefen(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("assert") || name.starts_with("log_") || name.starts_with("error_")) {
            return Ok(None);
        }
        let v = match name {
            // WP F -- Angaben zum zuletzt aufgetretenen Fehler. Im CATCH-Zweig
            // gedacht; ausserhalb liefern sie den letzten Stand (0 bzw. "").
            "error_line" => {
                if !a.is_empty() { return Err("ERROR_LINE: erwartet 0 Argumente".into()); }
                Value::Int(self.error_line as i64)
            }
            "error_code$" | "error_code" => {
                if !a.is_empty() { return Err("ERROR_CODE$: erwartet 0 Argumente".into()); }
                Value::str_rc(&self.error_code)
            }
            "assert" => {
                if a.is_empty() || a.len() > 2 {
                    return Err(format!("ASSERT: erwartet 1..2 Argumente (bedingung [, meldung]), erhalten {}", a.len()));
                }
                // Absichtlich streng auf BOOLEAN: `ASSERT(anzahl)` waere sonst
                // still "wahr, weil nicht null" -- und eine Pruefung, die aus
                // Versehen immer durchgeht, ist schlimmer als gar keine.
                let ok = match &a[0] {
                    Value::Bool(b) => *b,
                    andere => return Err(format!(
                        "ASSERT: erwartet BOOLEAN, erhalten {} -- einen Vergleich schreiben, \
                         z.B. ASSERT(anzahl > 0)", andere.type_name())),
                };
                let text = match a.get(1) {
                    Some(v) => crate::builtins::str_of(v),
                    None => String::new(),
                };
                self.assert_werten(ok, if text.is_empty() { "Bedingung nicht erfuellt".to_string() } else { text })?
            }
            "assert_eq" => {
                if a.len() < 2 || a.len() > 3 {
                    return Err(format!("ASSERT_EQ: erwartet 2..3 Argumente (ist, soll [, was]), erhalten {}", a.len()));
                }
                // Dieselbe Gleichheit wie der `=`-Operator der Sprache --
                // eine zweite Vorstellung davon, wann zwei Werte gleich sind,
                // waere die sicherste Art, Vertrauen zu verspielen.
                let ok = crate::value::value_eq(&a[0], &a[1]);
                let was = match a.get(2) {
                    Some(v) => crate::builtins::str_of(v),
                    None => String::new(),
                };
                let meldung = if ok {
                    was
                } else {
                    let kern = format!("erhalten {}, erwartet {}",
                                       crate::builtins::str_of(&a[0]),
                                       crate::builtins::str_of(&a[1]));
                    if was.is_empty() { kern } else { format!("{}: {}", was, kern) }
                };
                self.assert_werten(ok, meldung)?
            }
            "assert_collect" => {
                if a.len() != 1 { return Err(format!("ASSERT_COLLECT: erwartet 1 Argument, erhalten {}", a.len())); }
                self.assert_sammeln = match &a[0] {
                    Value::Bool(b) => *b,
                    andere => return Err(format!("ASSERT_COLLECT: erwartet BOOLEAN, erhalten {}", andere.type_name())),
                };
                Value::Nil
            }
            "assert_count" => { if !a.is_empty() { return Err("ASSERT_COUNT: erwartet 0 Argumente".into()); } Value::Int(self.assert_geprueft) }
            "assert_failed" => { if !a.is_empty() { return Err("ASSERT_FAILED: erwartet 0 Argumente".into()); } Value::Int(self.assert_fehler) }
            "assert_report" => {
                if !a.is_empty() { return Err("ASSERT_REPORT: erwartet 0 Argumente".into()); }
                // Die Bilanz gehoert zu den Nutzdaten (stdout), nicht zu den
                // Meldungen: sie ist das Ergebnis des Pruefprogramms.
                let zeile = if self.assert_fehler == 0 {
                    format!("ALLES GRUEN -- {} Pruefungen", self.assert_geprueft)
                } else {
                    format!("FEHLER: {} von {} Pruefungen", self.assert_fehler, self.assert_geprueft)
                };
                self.out.push_str(&zeile);
                self.out.push('\n');
                Value::Int(self.assert_fehler)
            }
            "log_debug" | "log_info" | "log_warn" | "log_error" => {
                if a.len() != 1 { return Err(format!("{}: erwartet 1 Argument, erhalten {}", name.to_uppercase(), a.len())); }
                let stufe = match name { "log_debug" => 0u8, "log_info" => 1, "log_warn" => 2, _ => 3 };
                if stufe >= self.log_schwelle() {
                    let (_, _, _, h, mi, s) = crate::builtins::local_datetime();
                    let marke = match stufe { 0 => "DEBUG", 1 => "INFO", 2 => "WARN", _ => "ERROR" };
                    let text = crate::builtins::str_of(&a[0]);
                    self.flush_out();
                    use std::io::Write;
                    let se = std::io::stderr();
                    let mut hd = se.lock();
                    let _ = writeln!(hd, "{:02}:{:02}:{:02} {:<5} {}", h, mi, s, marke, text);
                    let _ = hd.flush();
                }
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Eine Pruefung verbuchen. Im Sammel-Modus wird ein Fehlschlag gemeldet
    /// und weitergemacht, sonst bricht er ab (mit Datei:Zeile aus dem
    /// gewohnten Laufzeitfehler-Pfad).
    fn assert_werten(&mut self, ok: bool, meldung: String) -> R<Value> {
        self.assert_geprueft += 1;
        if ok { return Ok(Value::Nil); }
        self.assert_fehler += 1;
        if !self.assert_sammeln {
            return Err(format!("ASSERT fehlgeschlagen: {}", meldung));
        }
        // Fehlschlaege gehen nach stderr, damit die Nutzdaten auf stdout
        // sauber bleiben -- ein Pruefprogramm laesst sich so umleiten.
        self.flush_out();
        use std::io::Write;
        let se = std::io::stderr();
        let mut h = se.lock();
        let _ = if self.cur_line != 0 {
            writeln!(h, "FEHL  Zeile {}: {}", self.cur_line, meldung)
        } else {
            writeln!(h, "FEHL  {}", meldung)
        };
        let _ = h.flush();
        Ok(Value::Nil)
    }

    /// Ab welcher Stufe LOG_* etwas ausgibt. Aus `DH_LOG` (debug/info/warn/
    /// error/aus), einmal nachgesehen und dann gemerkt. Vorgabe: `info` --
    /// LOG_DEBUG schweigt also, bis jemand es einschaltet.
    fn log_schwelle(&mut self) -> u8 {
        if let Some(p) = self.log_pegel { return p; }
        let p = match std::env::var("DH_LOG").unwrap_or_default().to_ascii_lowercase().as_str() {
            "debug" => 0,
            "warn" => 2,
            "error" => 3,
            "aus" | "off" | "none" => 9,
            _ => 1,          // info (Vorgabe, auch bei unbekanntem Wert)
        };
        self.log_pegel = Some(p);
        p
    }

    /// Gepufferte `PRINT`-Ausgabe sofort auf echtes stdout schreiben und den
    /// Puffer leeren, damit `take_output()` am Ende nichts doppelt schreibt.
    /// Unter dem Profiler NICHT -- dort gehoert stdout dem JSON-Blob (gleiche
    /// Ueberlegung wie in `flush_and_prompt`).
    fn flush_out(&mut self) {
        if self.prof.is_some() || self.out.is_empty() { return; }
        use std::io::Write;
        let so = std::io::stdout();
        let mut h = so.lock();
        let _ = h.write_all(self.out.as_bytes());
        let _ = h.flush();
        self.out.clear();
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
                    // Ein LEERES Array passt immer: es kann keinen falschen
                    // Wert enthalten. Sonst braeuchte ein Widget, das seine
                    // Daten erst spaeter bekommt, einen Platzhalter-Eintrag.
                    if arr.cells.len() == 0 && arr.dims.len() <= 1 { return Ok(Vec::new()); }
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
                    if arr.cells.len() == 0 { return Ok(Vec::new()); }
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
            "gui_window_scrollable" => { self.gui.window_scrollable(gi(a,0,"GUI_WINDOW_SCROLLABLE")?, gbool(a,1,"GUI_WINDOW_SCROLLABLE")?)?; Value::Nil }
            "gui_tabs" => { self.gui.set_tabs(gi(a,0,"GUI_TABS")?, gstrs(a,1,"GUI_TABS")?)?; Value::Nil }
            "gui_set_tab" => { self.gui.set_widget_tab(gi(a,0,"GUI_SET_TAB")?, gi(a,1,"GUI_SET_TAB")? as i32)?; Value::Nil }
            "gui_active_tab" => Value::Int(self.gui.active_tab(gi(a,0,"GUI_ACTIVE_TAB")?)?),
            "gui_set_active_tab" => { self.gui.set_active_tab(gi(a,0,"GUI_SET_ACTIVE_TAB")?, gi(a,1,"GUI_SET_ACTIVE_TAB")? as i32)?; Value::Nil }
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
            "gui_toggle" => {
                let def = if a.len() >= 5 { gbool(a,4,"GUI_TOGGLE")? } else { false };
                Value::Int(self.gui.toggle(gi(a,0,"GUI_TOGGLE")?, gs(a,1,"GUI_TOGGLE")?,
                    gi(a,2,"GUI_TOGGLE")? as i32, gi(a,3,"GUI_TOGGLE")? as i32, def)?)
            }
            "gui_knob" => {
                let mn = gnum(a,4,"GUI_KNOB")?; let mx = gnum(a,5,"GUI_KNOB")?;
                let def = if a.len() >= 7 { gnum(a,6,"GUI_KNOB")? } else { mn };
                Value::Int(self.gui.knob(gi(a,0,"GUI_KNOB")?, gi(a,1,"GUI_KNOB")? as i32,
                    gi(a,2,"GUI_KNOB")? as i32, gi(a,3,"GUI_KNOB")? as i32, mn, mx, def)?)
            }
            "gui_slider" => {
                let mn = gnum(a,4,"GUI_SLIDER")?; let mx = gnum(a,5,"GUI_SLIDER")?;
                let def = if a.len() >= 7 { gnum(a,6,"GUI_SLIDER")? } else { mn };
                Value::Int(self.gui.slider(gi(a,0,"GUI_SLIDER")?, gi(a,1,"GUI_SLIDER")? as i32,
                    gi(a,2,"GUI_SLIDER")? as i32, gi(a,3,"GUI_SLIDER")? as i32, mn, mx, def)?)
            }
            "gui_spinner" => {
                let mn = gnum(a,4,"GUI_SPINNER")?; let mx = gnum(a,5,"GUI_SPINNER")?;
                let def = if a.len() >= 7 { gnum(a,6,"GUI_SPINNER")? } else { mn };
                let step = if a.len() >= 8 { gnum(a,7,"GUI_SPINNER")? } else { 1.0 };
                Value::Int(self.gui.spinner(gi(a,0,"GUI_SPINNER")?, gi(a,1,"GUI_SPINNER")? as i32,
                    gi(a,2,"GUI_SPINNER")? as i32, gi(a,3,"GUI_SPINNER")? as i32, mn, mx, def, step)?)
            }
            "gui_splitter" => {
                Value::Int(self.gui.splitter(gi(a,0,"GUI_SPLITTER")?, gi(a,1,"GUI_SPLITTER")? as i32,
                    gi(a,2,"GUI_SPLITTER")? as i32, gi(a,3,"GUI_SPLITTER")? as i32, gs(a,4,"GUI_SPLITTER")?,
                    gi(a,5,"GUI_SPLITTER")? as i32, gi(a,6,"GUI_SPLITTER")? as i32)?)
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
            "gui_textarea" => {
                let ph = if a.len() >= 6 { gs(a,5,"GUI_TEXTAREA")? } else { String::new() };
                Value::Int(self.gui.textarea(gi(a,0,"GUI_TEXTAREA")?, gi(a,1,"GUI_TEXTAREA")? as i32, gi(a,2,"GUI_TEXTAREA")? as i32,
                    gi(a,3,"GUI_TEXTAREA")? as i32, gi(a,4,"GUI_TEXTAREA")? as i32, ph)?)
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
            "gui_icon_button" => {
                let text = if a.len() >= 7 { gs(a,6,"GUI_ICON_BUTTON")? } else { String::new() };
                Value::Int(self.gui.icon_button(gi(a,0,"GUI_ICON_BUTTON")?, gi(a,1,"GUI_ICON_BUTTON")? as i32,
                    gi(a,2,"GUI_ICON_BUTTON")? as i32, gi(a,3,"GUI_ICON_BUTTON")? as i32, gi(a,4,"GUI_ICON_BUTTON")? as i32,
                    gi(a,5,"GUI_ICON_BUTTON")?, text)?)
            }
            "gui_set_icon" => { self.gui.set_icon(gi(a,0,"GUI_SET_ICON")?, gi(a,1,"GUI_SET_ICON")?)?; Value::Nil }
            "gui_toolbar" => Value::Int(self.gui.toolbar(gi(a,0,"GUI_TOOLBAR")?, gi(a,1,"GUI_TOOLBAR")? as i32,
                gi(a,2,"GUI_TOOLBAR")? as i32, gi(a,3,"GUI_TOOLBAR")? as i32, gi(a,4,"GUI_TOOLBAR")? as i32)?),
            "gui_canvas" => Value::Int(self.gui.canvas(gi(a,0,"GUI_CANVAS")?, gi(a,1,"GUI_CANVAS")? as i32,
                gi(a,2,"GUI_CANVAS")? as i32, gi(a,3,"GUI_CANVAS")? as i32, gi(a,4,"GUI_CANVAS")? as i32)?),
            "gui_canvas_x" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_X")?)?.0 as i64),
            "gui_canvas_y" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_Y")?)?.1 as i64),
            "gui_canvas_w" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_W")?)?.2 as i64),
            "gui_canvas_h" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_H")?)?.3 as i64),
            "gui_clicked" => Value::Bool(self.gui.clicked(gi(a,0,"GUI_CLICKED")?)?),
            "gui_menu" => Value::Int(self.gui.add_menu(gi(a,0,"GUI_MENU")?, gs(a,1,"GUI_MENU")?.to_string())?),
            "gui_context" => Value::Int(self.gui.add_context(gi(a,0,"GUI_CONTEXT")?)?),
            "gui_menu_item" => Value::Int(self.gui.add_menu_item(gi(a,0,"GUI_MENU_ITEM")?, gs(a,1,"GUI_MENU_ITEM")?.to_string())?),
            "gui_menu_separator" => { self.gui.add_menu_separator(gi(a,0,"GUI_MENU_SEPARATOR")?)?; Value::Nil }
            "gui_hovered" => Value::Bool(self.gui.hovered(gi(a,0,"GUI_HOVERED")?)?),
            "gui_checked" => Value::Bool(self.gui.checked(gi(a,0,"GUI_CHECKED")?)?),
            "gui_value" => Value::Float(self.gui.value(gi(a,0,"GUI_VALUE")?)?),
            "gui_text" => Value::str_rc(&self.gui.text(gi(a,0,"GUI_TEXT")?)?),
            "gui_set_text" => { self.gui.set_text(gi(a,0,"GUI_SET_TEXT")?, gs(a,1,"GUI_SET_TEXT")?)?; Value::Nil }
            "gui_tooltip" => { self.gui.set_tooltip(gi(a,0,"GUI_TOOLTIP")?, gs(a,1,"GUI_TOOLTIP")?)?; Value::Nil }
            "gui_set_checked" => { self.gui.set_checked(gi(a,0,"GUI_SET_CHECKED")?, gbool(a,1,"GUI_SET_CHECKED")?)?; Value::Nil }
            "gui_set_value" => { self.gui.set_value(gi(a,0,"GUI_SET_VALUE")?, gnum(a,1,"GUI_SET_VALUE")?)?; Value::Nil }
            "gui_on_click" => { self.gui.on_click(gi(a,0,"GUI_ON_CLICK")?, gfunc(a,1,"GUI_ON_CLICK")?)?; Value::Nil }
            "gui_on_change" => { self.gui.on_change(gi(a,0,"GUI_ON_CHANGE")?, gfunc(a,1,"GUI_ON_CHANGE")?)?; Value::Nil }
            "gui_on_hover" => { self.gui.on_hover(gi(a,0,"GUI_ON_HOVER")?, gfunc(a,1,"GUI_ON_HOVER")?)?; Value::Nil }
            "gui_on_leave" => { self.gui.on_leave(gi(a,0,"GUI_ON_LEAVE")?, gfunc(a,1,"GUI_ON_LEAVE")?)?; Value::Nil }
            "gui_on_focus" => { self.gui.on_focus(gi(a,0,"GUI_ON_FOCUS")?, gfunc(a,1,"GUI_ON_FOCUS")?)?; Value::Nil }
            "gui_on_blur" => { self.gui.on_blur(gi(a,0,"GUI_ON_BLUR")?, gfunc(a,1,"GUI_ON_BLUR")?)?; Value::Nil }
            "gui_theme" => { self.gui.theme_accent(gi(a,0,"GUI_THEME")?); Value::Nil }
            "gui_theme_set" => { self.gui.theme_set(gs(a,0,"GUI_THEME_SET")?, gi(a,1,"GUI_THEME_SET")?)?; Value::Nil }
            "gui_theme_get" => Value::Int(self.gui.theme_get(&gs(a,0,"GUI_THEME_GET")?)?),
            "gui_metric_set" => { self.gui.metric_set(gs(a,0,"GUI_METRIC_SET")?, gi(a,1,"GUI_METRIC_SET")? as i32)?; Value::Nil }
            "gui_metric_get" => Value::Int(self.gui.metric_get(&gs(a,0,"GUI_METRIC_GET")?)? as i64),
            "gui_skin" => {
                let rand = if a.len() >= 3 { gi(a,2,"GUI_SKIN")? as i32 } else { 0 };
                self.gui.set_skin(gs(a,0,"GUI_SKIN")?, gi(a,1,"GUI_SKIN")?, rand)?;
                Value::Nil
            }
            "gui_set_round" => { self.gui.set_round(gi(a,0,"GUI_SET_ROUND")?, gbool(a,1,"GUI_SET_ROUND")?)?; Value::Nil }
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
            "gui_table_clicked_col" => Value::Int(self.gui.table_clicked_col(gi(a,0,"GUI_TABLE_CLICKED_COL")?)?),
            // --- Zellen einzeln ---
            "gui_table_set_cell" => { self.gui.table_set_cell(gi(a,0,"GUI_TABLE_SET_CELL")?, gi(a,1,"GUI_TABLE_SET_CELL")?, gi(a,2,"GUI_TABLE_SET_CELL")?, gs(a,3,"GUI_TABLE_SET_CELL")?.to_string())?; Value::Nil }
            "gui_table_get_cell" => Value::str_rc(&self.gui.table_get_cell(gi(a,0,"GUI_TABLE_GET_CELL")?, gi(a,1,"GUI_TABLE_GET_CELL")?, gi(a,2,"GUI_TABLE_GET_CELL")?)?),
            "gui_table_cell_color" => { self.gui.table_cell_color(gi(a,0,"GUI_TABLE_CELL_COLOR")?, gi(a,1,"GUI_TABLE_CELL_COLOR")?, gi(a,2,"GUI_TABLE_CELL_COLOR")?, gi(a,3,"GUI_TABLE_CELL_COLOR")?, gi(a,4,"GUI_TABLE_CELL_COLOR")?)?; Value::Nil }
            "gui_table_cell_align" => { self.gui.table_cell_align(gi(a,0,"GUI_TABLE_CELL_ALIGN")?, gi(a,1,"GUI_TABLE_CELL_ALIGN")?, gi(a,2,"GUI_TABLE_CELL_ALIGN")?, &gs(a,3,"GUI_TABLE_CELL_ALIGN")?)?; Value::Nil }
            "gui_table_cell_kind" => { self.gui.table_cell_kind(gi(a,0,"GUI_TABLE_CELL_KIND")?, gi(a,1,"GUI_TABLE_CELL_KIND")?, gi(a,2,"GUI_TABLE_CELL_KIND")?, &gs(a,3,"GUI_TABLE_CELL_KIND")?)?; Value::Nil }
            "gui_table_cell_image" => { self.gui.table_cell_image(gi(a,0,"GUI_TABLE_CELL_IMAGE")?, gi(a,1,"GUI_TABLE_CELL_IMAGE")?, gi(a,2,"GUI_TABLE_CELL_IMAGE")?, gi(a,3,"GUI_TABLE_CELL_IMAGE")?)?; Value::Nil }
            "gui_table_cell_value" => { self.gui.table_cell_value(gi(a,0,"GUI_TABLE_CELL_VALUE")?, gi(a,1,"GUI_TABLE_CELL_VALUE")?, gi(a,2,"GUI_TABLE_CELL_VALUE")?, gnum(a,3,"GUI_TABLE_CELL_VALUE")?)?; Value::Nil }
            "gui_table_get_value" => Value::Float(self.gui.table_get_value(gi(a,0,"GUI_TABLE_GET_VALUE")?, gi(a,1,"GUI_TABLE_GET_VALUE")?, gi(a,2,"GUI_TABLE_GET_VALUE")?)?),
            // --- Zeilen / Spalten ---
            "gui_table_row_color" => { self.gui.table_row_color(gi(a,0,"GUI_TABLE_ROW_COLOR")?, gi(a,1,"GUI_TABLE_ROW_COLOR")?, gi(a,2,"GUI_TABLE_ROW_COLOR")?, gi(a,3,"GUI_TABLE_ROW_COLOR")?)?; Value::Nil }
            "gui_table_col_align" => { self.gui.table_col_align(gi(a,0,"GUI_TABLE_COL_ALIGN")?, gi(a,1,"GUI_TABLE_COL_ALIGN")?, &gs(a,2,"GUI_TABLE_COL_ALIGN")?)?; Value::Nil }
            "gui_table_add_row" => Value::Int(self.gui.table_add_row(gi(a,0,"GUI_TABLE_ADD_ROW")?, gstrs(a,1,"GUI_TABLE_ADD_ROW")?)?),
            "gui_table_remove_row" => { self.gui.table_remove_row(gi(a,0,"GUI_TABLE_REMOVE_ROW")?, gi(a,1,"GUI_TABLE_REMOVE_ROW")?)?; Value::Nil }
            "gui_table_clear" => { self.gui.table_clear(gi(a,0,"GUI_TABLE_CLEAR")?)?; Value::Nil }
            // --- tabellenweite Einstellungen ---
            "gui_table_set" => { self.gui.table_set_opt(gi(a,0,"GUI_TABLE_SET")?, &gs(a,1,"GUI_TABLE_SET")?, gnum(a,2,"GUI_TABLE_SET")?)?; Value::Nil }
            "gui_table_get" => Value::Float(self.gui.table_get_opt(gi(a,0,"GUI_TABLE_GET")?, &gs(a,1,"GUI_TABLE_GET")?)?),
            // --- Spalten-Reihenfolge ---
            "gui_table_move_col" => { self.gui.table_move_col(gi(a,0,"GUI_TABLE_MOVE_COL")?, gi(a,1,"GUI_TABLE_MOVE_COL")?, gi(a,2,"GUI_TABLE_MOVE_COL")?)?; Value::Nil }
            "gui_table_col_at" => Value::Int(self.gui.table_col_at(gi(a,0,"GUI_TABLE_COL_AT")?, gi(a,1,"GUI_TABLE_COL_AT")?)?),
            "gui_table_col_pos" => Value::Int(self.gui.table_col_pos(gi(a,0,"GUI_TABLE_COL_POS")?, gi(a,1,"GUI_TABLE_COL_POS")?)?),
            "gui_table_reset_cols" => { self.gui.table_reset_cols(gi(a,0,"GUI_TABLE_RESET_COLS")?)?; Value::Nil }
            // --- Mehrfachauswahl ---
            "gui_table_sel_count" => Value::Int(self.gui.table_sel_count(gi(a,0,"GUI_TABLE_SEL_COUNT")?)?),
            "gui_table_sel_row" => Value::Int(self.gui.table_sel_row(gi(a,0,"GUI_TABLE_SEL_ROW")?, gi(a,1,"GUI_TABLE_SEL_ROW")?)?),
            "gui_table_is_selected" => Value::Bool(self.gui.table_is_selected(gi(a,0,"GUI_TABLE_IS_SELECTED")?, gi(a,1,"GUI_TABLE_IS_SELECTED")?)?),
            "gui_table_select" => { self.gui.table_select(gi(a,0,"GUI_TABLE_SELECT")?, gi(a,1,"GUI_TABLE_SELECT")?, gbool(a,2,"GUI_TABLE_SELECT")?)?; Value::Nil }
            "gui_table_clear_selection" => { self.gui.table_clear_selection(gi(a,0,"GUI_TABLE_CLEAR_SELECTION")?)?; Value::Nil }
            // --- Zellen bearbeiten ---
            "gui_table_col_edit" => { self.gui.table_col_edit(gi(a,0,"GUI_TABLE_COL_EDIT")?, gi(a,1,"GUI_TABLE_COL_EDIT")?, gbool(a,2,"GUI_TABLE_COL_EDIT")?)?; Value::Nil }
            "gui_table_editing_row" => Value::Int(self.gui.table_editing_row(gi(a,0,"GUI_TABLE_EDITING_ROW")?)?),
            "gui_table_editing_col" => Value::Int(self.gui.table_editing_col(gi(a,0,"GUI_TABLE_EDITING_COL")?)?),
            // --- Sortieren / Filtern ---
            "gui_table_sort" => { self.gui.table_sort(gi(a,0,"GUI_TABLE_SORT")?, gi(a,1,"GUI_TABLE_SORT")?, gbool(a,2,"GUI_TABLE_SORT")?)?; Value::Nil }
            "gui_table_sort_col" => Value::Int(self.gui.table_sort_col(gi(a,0,"GUI_TABLE_SORT_COL")?)?),
            "gui_table_sort_desc" => Value::Bool(self.gui.table_sort_desc(gi(a,0,"GUI_TABLE_SORT_DESC")?)?),
            "gui_table_filter" => { self.gui.table_filter(gi(a,0,"GUI_TABLE_FILTER")?, gi(a,1,"GUI_TABLE_FILTER")?, gs(a,2,"GUI_TABLE_FILTER")?.to_string())?; Value::Nil }
            "gui_table_get_filter" => Value::str_rc(&self.gui.table_get_filter(gi(a,0,"GUI_TABLE_GET_FILTER")?, gi(a,1,"GUI_TABLE_GET_FILTER")?)?),
            "gui_table_view_count" => Value::Int(self.gui.table_view_count(gi(a,0,"GUI_TABLE_VIEW_COUNT")?)?),
            "gui_table_view_row" => Value::Int(self.gui.table_view_row(gi(a,0,"GUI_TABLE_VIEW_ROW")?, gi(a,1,"GUI_TABLE_VIEW_ROW")?)?),
            "gui_table_row_count" => Value::Int(self.gui.table_row_count(gi(a,0,"GUI_TABLE_ROW_COUNT")?)?),
            "gui_tree" => Value::Int(self.gui.tree(gi(a,0,"GUI_TREE")?, gi(a,1,"GUI_TREE")? as i32,
                gi(a,2,"GUI_TREE")? as i32, gi(a,3,"GUI_TREE")? as i32, gi(a,4,"GUI_TREE")? as i32)?),
            "gui_tree_add" => Value::Int(self.gui.tree_add(gi(a,0,"GUI_TREE_ADD")?, gi(a,1,"GUI_TREE_ADD")?, gs(a,2,"GUI_TREE_ADD")?)?),
            "gui_tree_clear" => { self.gui.tree_clear(gi(a,0,"GUI_TREE_CLEAR")?)?; Value::Nil }
            "gui_tree_selected" => Value::Int(self.gui.tree_selected(gi(a,0,"GUI_TREE_SELECTED")?)?),
            "gui_tree_set_selected" => { self.gui.tree_set_selected(gi(a,0,"GUI_TREE_SET_SELECTED")?, gi(a,1,"GUI_TREE_SET_SELECTED")?)?; Value::Nil }
            "gui_tree_label" => Value::str_rc(&self.gui.tree_label(gi(a,0,"GUI_TREE_LABEL")?, gi(a,1,"GUI_TREE_LABEL")?)?),
            "gui_tree_expand" => { self.gui.tree_expand(gi(a,0,"GUI_TREE_EXPAND")?, gi(a,1,"GUI_TREE_EXPAND")?, gbool(a,2,"GUI_TREE_EXPAND")?)?; Value::Nil }
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
                if arr.cells.len() > 0 && (arr.element_type != "string" || arr.dims.len() != 1) { return Err(format!("{}: headers muss 1D ARRAY OF STRING sein", f)); }
                Ok(arr.cells.iter().map(|x| match x { Value::Str(s) => s.to_string(), o => o.fmt() }).collect()) }
                _ => Err(format!("{}: headers muss ARRAY OF STRING sein", f)) }
        }
        fn str2d(v: &Value, ncols: usize, f: &str) -> R<Vec<Vec<String>>> {
            match v { Value::Array(arr) => { let arr = arr.borrow();
                if arr.cells.len() > 0 && (arr.element_type != "string" || arr.dims.len() != 2) { return Err(format!("{}: cells muss 2D ARRAY OF STRING sein", f)); }
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
        // F6 (dhrt) bei Float-Koordinaten konsistent (z.B. LINE(10.5, ...)).
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
        // Flag-Argument: akzeptiert TRUE/FALSE UND 1/0 (wie need_flag); fehlend = false.
        fn gflag(a: &[Value], i: usize) -> bool {
            match a.get(i) {
                Some(Value::Bool(b)) => *b,
                Some(Value::Int(n)) => *n != 0,
                Some(Value::Float(f)) => *f != 0.0,
                _ => false,
            }
        }
        fn need_f(a: &[Value], i: usize, fn_: &str) -> R<f64> {
            match a.get(i) {
                Some(Value::Int(n)) => Ok(*n as f64),
                Some(Value::Float(f)) => Ok(*f),
                _ => Err(format!("{}: Argument {} muss Zahl sein", fn_, i + 1)),
            }
        }
        // Genau `n` Zahlen-Argumente als f32 (fuer die koordinatenreichen
        // Geometrie-Builtins -- 15 einzelne need_f-Zeilen liest niemand mehr).
        fn fv(a: &[Value], n: usize, fn_: &str) -> R<Vec<f32>> {
            if a.len() != n {
                return Err(format!("{}: erwartet {} Zahlen, erhalten {}", fn_, n, a.len()));
            }
            (0..n).map(|i| Ok(need_f(a, i, fn_)? as f32)).collect()
        }
        // Optionaler Easing-Name fuer Fades/Slides (linear/in/out/inout,
        // Default "linear" bei fehlendem Argument). Pruefung VOR der Audio-
        // Initialisierung -> golden-testbar (wie die Bus-Namen-Checks).
        fn easing_name(a: &[Value], i: usize, fn_: &str) -> R<String> {
            let name = if a.len() > i { gs(a, i, fn_)?.to_lowercase() } else { "linear".to_string() };
            if !matches!(name.as_str(), "linear" | "in" | "out" | "inout") {
                return Err(format!("{}: unbekanntes Easing '{}' (linear, in, out, inout)", fn_, name));
            }
            Ok(name)
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
        /// Optionale Stueckzahl der Bulk-Zeichenbefehle lesen.
        ///
        /// Ohne sie zeichnen sie das GANZE Array -- wer einen Puffer fest
        /// dimensioniert und pro Bild nur teilweise fuellt, bekam die alten
        /// Werte der restlichen Plaetze mitgezeichnet. Zusaetzlich wird hier
        /// ein Argument zu viel abgelehnt: vorher wurde es still ignoriert,
        /// eine mitgegebene Stueckzahl also wirkungslos verschluckt.
        fn bulk_count(a: &[Value], idx: usize, n: usize, fn_: &str) -> R<usize> {
            if a.len() > idx + 1 {
                return Err(format!("{}: zu viele Argumente ({} statt hoechstens {})",
                                   fn_, a.len(), idx + 1));
            }
            match a.get(idx) {
                None => Ok(n),
                Some(v) => {
                    let c = match v {
                        Value::Int(i) => *i,
                        Value::Float(f) if f.fract() == 0.0 => *f as i64,
                        _ => return Err(format!("{}: anzahl muss eine ganze Zahl sein", fn_)),
                    };
                    if c < 0 { return Err(format!("{}: anzahl darf nicht negativ sein", fn_)); }
                    Ok((c as usize).min(n))
                }
            }
        }

        fn bulk_color(v: &Value, n: usize, fn_: &str) -> R<Vec<i64>> {
            match v {
                Value::Int(c) => Ok(vec![*c; n]),
                Value::Array(a) => {
                    let a = a.borrow();
                    // >= statt ==: mit einer Stueckzahl darf das Farb-Array so
                    // lang bleiben wie der Puffer, auch wenn weniger gezeichnet wird.
                    if a.cells.len() < n { return Err(format!("{}: colors-Array muss mindestens so lang wie die Stueckzahl sein", fn_)); }
                    let mut o = Vec::with_capacity(n);
                    if let Some(ints) = a.cells.as_ints() { return Ok(ints[..n].to_vec()); }
                    for x in a.cells.iter().take(n) { match x { Value::Int(i) => o.push(i), _ => return Err(format!("{}: color-ARRAY OF INTEGER noetig", fn_)) } }
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
                let title = if a.len() >= 3 { gs(a, 2, "SCREEN")?.to_string() } else { "Drachenhauch".to_string() };
                let scale = if a.len() >= 4 { gi(a, 3, "SCREEN")? as i32 } else { 1 };
                if scale < 1 { return Err("SCREEN: skala muss >= 1 sein".into()); }
                match self.gfx.as_mut() {
                    Some(gfx) => gfx.reconfigure(w, h, &title, scale),
                    None => self.gfx = Some(crate::graphics::Graphics::new(w, h, &title, scale)),
                }
                Value::Nil
            }
            "screen_native" => {
                let title = if !a.is_empty() { gs(a, 0, "SCREEN_NATIVE")?.to_string() } else { "Drachenhauch".to_string() };
                g!().screen_native(&title);
                Value::Nil
            }
            "screen_transparent" => {
                let w = gi(a, 0, "SCREEN_TRANSPARENT")? as i32;
                let h = gi(a, 1, "SCREEN_TRANSPARENT")? as i32;
                let title = if a.len() >= 3 { gs(a, 2, "SCREEN_TRANSPARENT")?.to_string() } else { "Drachenhauch".to_string() };
                let scale = if a.len() >= 4 { gi(a, 3, "SCREEN_TRANSPARENT")? as i32 } else { 1 };
                if scale < 1 { return Err("SCREEN_TRANSPARENT: skala muss >= 1 sein".into()); }
                // Transparenz ist ein Fenster-Erzeugungs-Flag (GLFW) -- nicht nachruestbar.
                if self.gfx.is_some() {
                    return Err("SCREEN_TRANSPARENT muss die ERSTE Grafik-Anweisung sein (vor LOADIMAGE/SCREEN/... -- Transparenz kann nicht nachtraeglich gesetzt werden)".into());
                }
                // w/h <= 0 -> ganzer aktueller Monitor (Vollbild-Overlay). Der Monitor
                // laesst sich erst nach der Fenster-Erzeugung abfragen, darum zuerst
                // transparent erzeugen, dann auf Monitorgroesse abdecken.
                let native = w <= 0 || h <= 0;
                let mut g = crate::graphics::Graphics::new_transparent(if native { 100 } else { w }, if native { 100 } else { h }, &title, scale);
                if native { g.cover_current_monitor(&title); }
                self.gfx = Some(g);
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
            "circleoutline" => {
                let c = if a.len() >= 4 { gi(a, 3, "CIRCLEOUTLINE")? } else { 0xFFFFFF };
                g!().circle_outline(gi(a,0,"CIRCLEOUTLINE")? as i32, gi(a,1,"CIRCLEOUTLINE")? as i32, gi(a,2,"CIRCLEOUTLINE")? as i32, c); Value::Nil
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
            "gentex_cellular" => Value::Int(g!().gen_tex_cellular(
                gi(a,0,"GENTEX_CELLULAR")? as i32, gi(a,1,"GENTEX_CELLULAR")? as i32,
                gi(a,2,"GENTEX_CELLULAR")?)?),
            "gentex_noise" => Value::Int(g!().gen_tex_noise(
                gi(a,0,"GENTEX_NOISE")? as i32, gi(a,1,"GENTEX_NOISE")? as i32,
                need_f(a,2,"GENTEX_NOISE")?)?),
            "gentex_gradient_box" => Value::Int(g!().gen_tex_gradient_square(
                gi(a,0,"GENTEX_GRADIENT_BOX")? as i32, gi(a,1,"GENTEX_GRADIENT_BOX")? as i32,
                need_f(a,2,"GENTEX_GRADIENT_BOX")?, gi(a,3,"GENTEX_GRADIENT_BOX")?,
                gi(a,4,"GENTEX_GRADIENT_BOX")?)?),
            // --- Bitmap-Font ---
            "loadfont_image" => Value::Int(g!().load_font_image(
                gi(a,0,"LOADFONT_IMAGE")?, gi(a,1,"LOADFONT_IMAGE")?, gi(a,2,"LOADFONT_IMAGE")?)?),
            "text_line_spacing" => { let n = gi(a,0,"TEXT_LINE_SPACING")?;
                                     g!().text_line_spacing(n); Value::Nil }
            // --- Clipboard + Drag&Drop (Batch 5) ---
            "clipboard_get" => Value::Str(g!().clipboard_get().into()),
            "clipboard_set" => { let s = gs(a,0,"CLIPBOARD_SET")?.to_string(); g!().clipboard_set(&s); Value::Nil }
            "files_dropped" => Value::Int(g!().dropped_files().len() as i64),
            "file_dropped" => {
                let i = gi(a, 0, "FILE_DROPPED")? as usize;
                Value::Str(g!().dropped_files().get(i).cloned().unwrap_or_default().into())
            }
            // --- Render-Targets (Batch 4) ---
            // RENDERTARGET_NEW(w, h [, behalten]) -- `behalten` = TRUE laesst den
            // Inhalt ueber das Bild hinaus stehen (Rueckkopplung/Nachzieher).
            "rendertarget_new" => Value::Int(g!().rendertarget_new(
                gi(a,0,"RENDERTARGET_NEW")? as i32, gi(a,1,"RENDERTARGET_NEW")? as i32,
                if a.len() >= 3 { gb(a, 2) } else { false })?),
            "rendertarget_clear" => {
                let farbe = if a.len() >= 2 { Some(gi(a,1,"RENDERTARGET_CLEAR")?) } else { None };
                g!().rendertarget_clear(gi(a,0,"RENDERTARGET_CLEAR")?, farbe)?;
                Value::Nil
            }
            "rendertarget_begin" => { g!().rendertarget_begin(gi(a,0,"RENDERTARGET_BEGIN")?)?; Value::Nil }
            "rendertarget_end" => { g!().rendertarget_end(); Value::Nil }
            "rendertarget_draw" => {
                let scale = if a.len() >= 4 { need_f(a,3,"RENDERTARGET_DRAW")? } else { 1.0 };
                let tint = if a.len() >= 5 { Some(gi(a,4,"RENDERTARGET_DRAW")?) } else { None };
                // Optionales 6. Argument: flip_v (vertikal spiegeln, fuer Boden-
                // Reflexionen). RENDERTARGET_DRAW(rt, x, y, scale, tint, TRUE).
                let flip_v = if a.len() >= 6 { gb(a, 5) } else { false };
                g!().rendertarget_draw(gi(a,0,"RENDERTARGET_DRAW")?,
                    gi(a,1,"RENDERTARGET_DRAW")? as i32, gi(a,2,"RENDERTARGET_DRAW")? as i32,
                    scale, tint, flip_v)?; Value::Nil
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
                let width = if a.len() >= 8 { Some(need_f(a, 7, "ARC")?) } else { None };
                g!().arc(gi(a,0,"ARC")? as i32, gi(a,1,"ARC")? as i32, gi(a,2,"ARC")? as i32, gi(a,3,"ARC")? as i32, start, end, width, c); Value::Nil
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
            "joystick_rumble" => {
                g!().joystick_rumble(gi(a,0,"JOYSTICK_RUMBLE")?, need_f(a,1,"JOYSTICK_RUMBLE")?, need_f(a,2,"JOYSTICK_RUMBLE")?, need_f(a,3,"JOYSTICK_RUMBLE")?)?;
                Value::Nil
            }
            "quitrequested" => Value::Bool(g!().quit_requested()),
            "mousex" => Value::Int(g!().mouse_x()),
            "mousey" => Value::Int(g!().mouse_y()),
            "mousebutton" => Value::Bool(g!().mouse_button(gi(a,0,"MOUSEBUTTON")?)),
            // --- Eingabe-Flanken: "genau in DIESEM Frame" ---------------------
            // MOUSEBUTTON/KEYPRESSED bleiben "gehalten" (Namen sind historisch,
            // sie umzudeuten wuerde bestehende Programme still kaputtmachen).
            "mouse_hit" => Value::Bool(g!().mouse_hit(gi(a,0,"MOUSE_HIT")?)),
            "mouse_released" => Value::Bool(g!().mouse_released(gi(a,0,"MOUSE_RELEASED")?)),
            "keyhit" => Value::Bool(g!().key_hit(gi(a,0,"KEYHIT")?)),
            "keyreleased" => Value::Bool(g!().key_released_edge(gi(a,0,"KEYRELEASED")?)),
            "keyrepeat" => Value::Bool(g!().key_repeat(gi(a,0,"KEYREPEAT")?)),
            "mouse_delta_x" => Value::Float(g!().mouse_delta_x()),
            "mouse_delta_y" => Value::Float(g!().mouse_delta_y()),
            "mouse_set_pos" => { let (x, y) = (gi(a,0,"MOUSE_SET_POS")?, gi(a,1,"MOUSE_SET_POS")?);
                                 g!().mouse_set_pos(x, y); Value::Nil }
            "mouse_on_screen" => Value::Bool(g!().mouse_on_screen()),
            "mouse_cursor" => { let s = gs(a,0,"MOUSE_CURSOR")?.to_string();
                                g!().mouse_cursor(&s)?; Value::Nil }
            "joystick_hit" => Value::Bool(g!().joystick_hit(gi(a,0,"JOYSTICK_HIT")?,
                                                            gi(a,1,"JOYSTICK_HIT")?)?),
            "joystick_released" => Value::Bool(g!().joystick_released(
                gi(a,0,"JOYSTICK_RELEASED")?, gi(a,1,"JOYSTICK_RELEASED")?)?),
            "joystick_any_button" => Value::Int(g!().joystick_any_button()),
            "joystick_axis_count" => Value::Int(g!().joystick_axis_count(
                gi(a,0,"JOYSTICK_AXIS_COUNT")?)?),
            // --- Touch + Gesten (raylib-Subsystem, war komplett ungenutzt) ----
            "touch_count" => Value::Int(g!().touch_count()),
            "touch_x" => Value::Float(g!().touch_x(gi(a,0,"TOUCH_X")?)),
            "touch_y" => Value::Float(g!().touch_y(gi(a,0,"TOUCH_Y")?)),
            "touch_id" => Value::Int(g!().touch_id(gi(a,0,"TOUCH_ID")?)),
            "gesture$" => Value::Str(g!().gesture().into()),
            "gesture_drag_x" => Value::Float(g!().gesture_drag_x()),
            "gesture_drag_y" => Value::Float(g!().gesture_drag_y()),
            "gesture_drag_angle" => Value::Float(g!().gesture_drag_angle()),
            "gesture_pinch_x" => Value::Float(g!().gesture_pinch_x()),
            "gesture_pinch_y" => Value::Float(g!().gesture_pinch_y()),
            "gesture_pinch_angle" => Value::Float(g!().gesture_pinch_angle()),
            "gesture_hold_time" => Value::Float(g!().gesture_hold_time()),
            // --- Fenster-Zustand + Politur ------------------------------------
            "window_focused" => Value::Bool(g!().window_focused()),
            "window_minimized" => Value::Bool(g!().window_minimized()),
            "window_maximized" => Value::Bool(g!().window_maximized()),
            "window_hidden" => Value::Bool(g!().window_hidden()),
            "window_is_fullscreen" => Value::Bool(g!().window_fullscreen_state()),
            "window_focus" => { g!().window_focus(); Value::Nil }
            "window_opacity" => { let v = need_f(a,0,"WINDOW_OPACITY")?;
                                  g!().window_opacity(v); Value::Nil }
            "window_icon" => { let i = gi(a,0,"WINDOW_ICON")?; g!().window_icon(i)?; Value::Nil }
            "get_time" => Value::Float(g!().get_time()),
            "openurl" => { let u = gs(a,0,"OPENURL")?.to_string(); g!().open_url(&u)?; Value::Nil }
            // Graceful ohne SCREEN (0 / 0) -- wie der Tree-Walker (_buf_size=(0,0),
            // pop_mouse_wheel ohne Fenster = 0).
            "mousewheel" => Value::Int(self.gfx.as_ref().map(|g| g.pop_mouse_wheel()).unwrap_or(0)),
            "mousewheel_x" => Value::Float(self.gfx.as_ref().map(|g| g.mouse_wheel_x()).unwrap_or(0.0)),
            "mousewheel_y" => Value::Float(self.gfx.as_ref().map(|g| g.mouse_wheel_y()).unwrap_or(0.0)),
            "key_name$" => Value::Str(g!().key_name(gi(a,0,"KEY_NAME$")?).into()),
            "key_any_hit" => Value::Int(g!().key_any_hit()),
            // Eingabe aufzeichnen/abspielen: Demo-Modus, Fehlerberichte zum
            // Nachspielen, automatische Spieltests.
            "automation_record" => { let p = gs(a,0,"AUTOMATION_RECORD")?.to_string(); g!().automation_record(&p)?; Value::Nil }
            "automation_stop" => Value::Int(g!().automation_stop()?),
            "automation_play" => { let p = gs(a,0,"AUTOMATION_PLAY")?.to_string(); Value::Int(g!().automation_play(&p)?) }
            "automation_recording" => Value::Bool(g!().automation_recording()),
            "automation_playing" => Value::Bool(g!().automation_playing()),
            "automation_frame" => Value::Int(g!().automation_frame()),
            "automation_count" => Value::Int(g!().automation_count()),
            "joystick_mappings" => Value::Int(g!().joystick_mappings(gs(a,0,"JOYSTICK_MAPPINGS")?)),
            "window_dpi_x" => Value::Float(g!().window_dpi_x()),
            "window_dpi_y" => Value::Float(g!().window_dpi_y()),
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
            // GFX_PUSH/GFX_POP: Zeichenzustand sichern und zurueckholen. Licht,
            // Nebel, Himmel, Schatten, Kamera, Schrift und Post-Effekt sind
            // global -- ohne das muss jede Szene beim Verlassen von Hand
            // aufraeumen, und eine vergessene Zeile faellt erst spaeter auf.
            "gfx_push" => { g!().gfx_push(); Value::Nil }
            "gfx_pop" => {
                if !g!().gfx_pop() {
                    return Err("GFX_POP: der Stapel ist leer -- zu jedem GFX_POP gehoert ein GFX_PUSH".into());
                }
                Value::Nil
            }
            "gfx_depth" => Value::Int(g!().gfx_depth()),
            // Dasselbe fuer die Audio-Busse: Effekte, die eine Szene anhaengt,
            // sollen den Rest des Programms nicht mitnehmen.
            "audio_push" => { self.audio_mut()?.audio_push(); Value::Nil }
            "audio_pop" => {
                if !self.audio_mut()?.audio_pop()? {
                    return Err("AUDIO_POP: der Stapel ist leer -- zu jedem AUDIO_POP gehoert ein AUDIO_PUSH".into());
                }
                Value::Nil
            }
            "audio_depth" => Value::Int(self.audio_mut()?.audio_depth()),
            // Natives OS-Fenster (das SCREEN-Fenster) steuern.
            "window_resizable" => { g!().window_resizable(gb(a, 0)); Value::Nil }
            "window_min_size" => { g!().window_min_size(gi(a,0,"WINDOW_MIN_SIZE")? as i32, gi(a,1,"WINDOW_MIN_SIZE")? as i32); Value::Nil }
            "window_max_size" => { g!().window_max_size(gi(a,0,"WINDOW_MAX_SIZE")? as i32, gi(a,1,"WINDOW_MAX_SIZE")? as i32); Value::Nil }
            "window_maximize" => { g!().window_maximize(); Value::Nil }
            "window_minimize" => { g!().window_minimize(); Value::Nil }
            "window_restore" => { g!().window_restore(); Value::Nil }
            "window_resized" => Value::Bool(g!().window_resized()),
            "window_undecorated" => { g!().window_undecorated(gb(a, 0)); Value::Nil }
            "window_topmost" => { g!().window_topmost(gb(a, 0)); Value::Nil }
            "window_esc_quit" => { g!().set_esc_quit(gb(a, 0)); Value::Nil }
            "window_passthrough" => { g!().window_passthrough(gb(a, 0)); Value::Nil }

            // --- Native OS-Datei-/Ordner-Dialoge (rfd; liefern Pfad oder "") ---
            // Eigenes Feature `dialogs`: im Web-Build gibt es keine
            // OS-Dialoge, und rfd wuerde den Build sogar verhindern (es zieht
            // js-sys/wasm-bindgen in einer Version nach, mit der cpals
            // WebAudio-Host nicht mehr uebersetzt). Ohne das Feature melden
            // die Aufrufe sich klar, statt still etwas Falsches zu liefern.
            #[cfg(feature = "dialogs")]
            "file_open_dialog" => {
                let title = if !a.is_empty() { gs(a, 0, "FILE_OPEN_DIALOG")?.to_string() } else { String::new() };
                let exts = if a.len() >= 2 { crate::filedialog::parse_exts(gs(a, 1, "FILE_OPEN_DIALOG")?) } else { vec![] };
                Value::str_rc(&crate::filedialog::open(&title, &exts))
            }
            #[cfg(feature = "dialogs")]
            "file_save_dialog" => {
                let title = if !a.is_empty() { gs(a, 0, "FILE_SAVE_DIALOG")?.to_string() } else { String::new() };
                let default_name = if a.len() >= 2 { gs(a, 1, "FILE_SAVE_DIALOG")?.to_string() } else { String::new() };
                let exts = if a.len() >= 3 { crate::filedialog::parse_exts(gs(a, 2, "FILE_SAVE_DIALOG")?) } else { vec![] };
                Value::str_rc(&crate::filedialog::save(&title, &default_name, &exts))
            }
            #[cfg(feature = "dialogs")]
            "folder_dialog" => {
                let title = if !a.is_empty() { gs(a, 0, "FOLDER_DIALOG")?.to_string() } else { String::new() };
                Value::str_rc(&crate::filedialog::folder(&title))
            }
            #[cfg(feature = "dialogs")]
            "gui_message" => {
                crate::filedialog::message(gs(a, 0, "GUI_MESSAGE")?, gs(a, 1, "GUI_MESSAGE")?);
                Value::Nil
            }
            #[cfg(feature = "dialogs")]
            "gui_confirm" => {
                // Drittes Argument: "janein" beschriftet die Knoepfe als Frage
                // statt als Anweisung. Vorgabe bleibt OK/Abbrechen.
                let stil = if a.len() > 2 { gs(a, 2, "GUI_CONFIRM")?.to_lowercase() } else { String::new() };
                let ja_nein = match stil.as_str() {
                    "" | "ok" | "okabbrechen" => false,
                    "janein" | "ja/nein" | "janein?" | "frage" => true,
                    other => return Err(format!(
                        "GUI_CONFIRM: '{}' ist kein Stil -- moeglich sind \"ok\" (Vorgabe) und \"janein\"", other)),
                };
                Value::Bool(crate::filedialog::confirm(
                    gs(a, 0, "GUI_CONFIRM")?, gs(a, 1, "GUI_CONFIRM")?, ja_nein))
            }
            #[cfg(not(feature = "dialogs"))]
            "file_open_dialog" | "file_save_dialog" | "folder_dialog"
            | "gui_message" | "gui_confirm" => {
                return Err(format!(
                    "{}: in diesem Build nicht verfuegbar -- native OS-Dialoge gibt es                      nur auf dem Desktop (Feature `dialogs`), nicht im Browser.",
                    name.to_uppercase()));
            }

            // --- Monitore / Display-Infos ---
            "monitor_count" => Value::Int(g!().monitor_count()),
            "current_monitor" => Value::Int(g!().current_monitor()),
            "monitor_width" => Value::Int(g!().monitor_width(gi(a, 0, "MONITOR_WIDTH")?)),
            "monitor_height" => Value::Int(g!().monitor_height(gi(a, 0, "MONITOR_HEIGHT")?)),
            "monitor_refresh" => Value::Int(g!().monitor_refresh(gi(a, 0, "MONITOR_REFRESH")?)),
            "monitor_name" => { let s = g!().monitor_name(gi(a, 0, "MONITOR_NAME")?); Value::str_rc(&s) }
            "monitor_x" => Value::Int(g!().monitor_x(gi(a, 0, "MONITOR_X")?)),
            "monitor_y" => Value::Int(g!().monitor_y(gi(a, 0, "MONITOR_Y")?)),
            "set_window_monitor" => { g!().set_window_monitor(gi(a, 0, "SET_WINDOW_MONITOR")?); Value::Nil }
            "window_x" => Value::Int(g!().window_x()),
            "window_y" => Value::Int(g!().window_y()),
            "set_window_pos" => { g!().set_window_pos(gi(a, 0, "SET_WINDOW_POS")?, gi(a, 1, "SET_WINDOW_POS")?); Value::Nil }

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
            "shader_set_array" => {
                let (h, n) = (gi(a,0,"SHADER_SET_ARRAY")?, gs(a,1,"SHADER_SET_ARRAY")?.to_string());
                let v = gfloats(a, 2, "SHADER_SET_ARRAY")?;
                g!().shader_set_array(h, &n, &v)?;
                Value::Nil
            }
            "shader_set_texture" => {
                let (h, n) = (gi(a,0,"SHADER_SET_TEXTURE")?, gs(a,1,"SHADER_SET_TEXTURE")?.to_string());
                let img = gi(a,2,"SHADER_SET_TEXTURE")?;
                g!().shader_set_texture(h, &n, img)?;
                Value::Nil
            }
            "shader_set_matrix" => {
                let (h, n) = (gi(a,0,"SHADER_SET_MATRIX")?, gs(a,1,"SHADER_SET_MATRIX")?.to_string());
                let m = match a.get(2) {
                    Some(Value::Mat4(m)) => **m,
                    _ => return Err("SHADER_SET_MATRIX: erwartet MAT4 (Modul m3d)".into()),
                };
                g!().shader_set_matrix(h, &n, &m)?;
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
            "camera_orbit" => {
                let fovy = if a.len() >= 7 { need_f(a, 6, "CAMERA_ORBIT")? as f32 } else { 0.0 };
                g!().camera_orbit(
                    need_f(a,0,"CAMERA_ORBIT")? as f32, need_f(a,1,"CAMERA_ORBIT")? as f32, need_f(a,2,"CAMERA_ORBIT")? as f32,
                    need_f(a,3,"CAMERA_ORBIT")? as f32, need_f(a,4,"CAMERA_ORBIT")? as f32, need_f(a,5,"CAMERA_ORBIT")? as f32,
                    fovy);
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
            "model_animate_blend" => {
                g!().model_animate_blend(
                    gi(a,0,"MODEL_ANIMATE_BLEND")?, gi(a,1,"MODEL_ANIMATE_BLEND")?,
                    gi(a,2,"MODEL_ANIMATE_BLEND")?, gi(a,3,"MODEL_ANIMATE_BLEND")? as i32,
                    gi(a,4,"MODEL_ANIMATE_BLEND")?, gi(a,5,"MODEL_ANIMATE_BLEND")? as i32,
                    need_f(a,6,"MODEL_ANIMATE_BLEND")? as f32,
                )?;
                Value::Nil
            }
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
                // Optionale Stueckzahl wie bei PLOTS/LINES/... -- ohne sie wird das
                // ganze Array gezeichnet, und ein fest dimensionierter Puffer
                // schleppt seine ungenutzten Plaetze mit ins Bild. Sie muss VOR
                // dem Einsammeln greifen: sonst stolpert das Einsammeln ueber
                // die noch nicht belegten (NIL-)Plaetze, die gar nicht gezeichnet
                // werden sollen.
                let mats = match a.get(1) {
                    Some(Value::Array(arr)) => {
                        let b = arr.borrow();
                        if b.dims.len() != 1 {
                            return Err("MODEL_INSTANCED: Arg 2 muss ein 1D-ARRAY OF MAT4 sein".into());
                        }
                        match b.cells.as_vals() {
                            Some(vals) => {
                                let n = bulk_count(a, 3, vals.len(), "MODEL_INSTANCED")?;
                                collect_mats(&mut vals.iter().take(n))?
                            }
                            None => return Err("MODEL_INSTANCED: Arg 2 muss ein ARRAY OF MAT4 sein".into()),
                        }
                    }
                    Some(Value::Tuple(t)) => {
                        let n = bulk_count(a, 3, t.len(), "MODEL_INSTANCED")?;
                        collect_mats(&mut t.iter().take(n))?
                    }
                    _ => return Err("MODEL_INSTANCED: Arg 2 muss ARRAY OF MAT4 oder TUPLE von MAT4 sein".into()),
                };
                let handle = gi(a, 0, "MODEL_INSTANCED")?;
                // tint darf eine Farbe ODER ein ARRAY OF INTEGER sein (eine Farbe
                // je Matrix). Der Instancing-Shader kennt nur EINE Farbe pro
                // Draw-Call -- raylibs DrawMeshInstanced uebertraegt nur die
                // Matrizen, keine Farb-Attribute. Deshalb wird hier nach Farben
                // GRUPPIERT: ein Draw-Call je verschiedener Farbe, nicht je
                // Instanz. Wer drei Farben nutzt, bekommt drei Draw-Calls fuer
                // beliebig viele Wuerfel; wer 1600 verschiedene nutzt, bekommt
                // 1600 -- dann ist die Gruppierung sinnlos und ein Farbverlauf
                // im Shader die bessere Antwort.
                match a.get(2) {
                    Some(Value::Array(arr)) => {
                        let farben = {
                            let b = arr.borrow();
                            if b.dims.len() != 1 {
                                return Err("MODEL_INSTANCED: tint-Array muss 1D sein".into());
                            }
                            if b.cells.len() < mats.len() {
                                return Err(format!(
                                    "MODEL_INSTANCED: tint-Array ist kuerzer als die Matrizen ({} < {})",
                                    b.cells.len(), mats.len()));
                            }
                            let mut v = Vec::with_capacity(mats.len());
                            for x in b.cells.iter().take(mats.len()) {
                                match x {
                                    Value::Int(i) => v.push(i),
                                    other => return Err(format!(
                                        "MODEL_INSTANCED: tint-Array braucht INTEGER-Farben (erhalten {})",
                                        other.type_name())),
                                }
                            }
                            v
                        };
                        // Reihenfolge des ersten Auftretens beibehalten -- so
                        // bleibt die Zeichenreihenfolge nachvollziehbar.
                        let mut gruppen: Vec<(i64, Vec<[f32; 16]>)> = Vec::new();
                        for (m, c) in mats.into_iter().zip(farben) {
                            match gruppen.iter_mut().find(|(gc, _)| *gc == c) {
                                Some((_, v)) => v.push(m),
                                None => gruppen.push((c, vec![m])),
                            }
                        }
                        for (c, ms) in gruppen {
                            g!().draw_model_instanced(handle, ms, c)?;
                        }
                    }
                    _ => {
                        let tint = if a.len() >= 3 { gi(a, 2, "MODEL_INSTANCED")? } else { 0xFF_FFFF };
                        g!().draw_model_instanced(handle, mats, tint)?;
                    }
                }
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
            // Picking auf ECHTER Geometrie statt nur Huellkoerper: einzelne
            // Dreiecke/Vierecke (Boden-Kacheln, Wandflaechen, In-Welt-Panels).
            "ray_hit_tri" => {
                let v = fv(a, 15, "RAY_HIT_TRI")?;
                Value::Float(g!().ray_hit_tri([v[0],v[1],v[2]], [v[3],v[4],v[5]],
                    [[v[6],v[7],v[8]], [v[9],v[10],v[11]], [v[12],v[13],v[14]]]))
            }
            "ray_hit_quad" => {
                let v = fv(a, 18, "RAY_HIT_QUAD")?;
                Value::Float(g!().ray_hit_quad([v[0],v[1],v[2]], [v[3],v[4],v[5]],
                    [[v[6],v[7],v[8]], [v[9],v[10],v[11]], [v[12],v[13],v[14]], [v[15],v[16],v[17]]]))
            }
            "pick_tri" => {
                let v = fv(a, 9, "PICK_TRI")?;
                Value::Float(g!().pick_tri([[v[0],v[1],v[2]], [v[3],v[4],v[5]], [v[6],v[7],v[8]]]))
            }
            "pick_quad" => {
                let v = fv(a, 12, "PICK_QUAD")?;
                Value::Float(g!().pick_quad([[v[0],v[1],v[2]], [v[3],v[4],v[5]],
                                             [v[6],v[7],v[8]], [v[9],v[10],v[11]]]))
            }
            "pick_box" => Value::Float(g!().pick_box(
                need_f(a,0,"PICK_BOX")? as f32, need_f(a,1,"PICK_BOX")? as f32, need_f(a,2,"PICK_BOX")? as f32,
                need_f(a,3,"PICK_BOX")? as f32, need_f(a,4,"PICK_BOX")? as f32, need_f(a,5,"PICK_BOX")? as f32)),
            "pick_sphere" => Value::Float(g!().pick_sphere(
                need_f(a,0,"PICK_SPHERE")? as f32, need_f(a,1,"PICK_SPHERE")? as f32, need_f(a,2,"PICK_SPHERE")? as f32,
                need_f(a,3,"PICK_SPHERE")? as f32)),
            "ray_hit_model" => {
                let scale = if a.len() >= 11 { need_f(a,10,"RAY_HIT_MODEL")? as f32 } else { 1.0 };
                Value::Float(g!().ray_hit_model(gi(a,0,"RAY_HIT_MODEL")?,
                    need_f(a,1,"RAY_HIT_MODEL")? as f32, need_f(a,2,"RAY_HIT_MODEL")? as f32, need_f(a,3,"RAY_HIT_MODEL")? as f32,
                    need_f(a,4,"RAY_HIT_MODEL")? as f32, need_f(a,5,"RAY_HIT_MODEL")? as f32, need_f(a,6,"RAY_HIT_MODEL")? as f32,
                    need_f(a,7,"RAY_HIT_MODEL")? as f32, need_f(a,8,"RAY_HIT_MODEL")? as f32, need_f(a,9,"RAY_HIT_MODEL")? as f32, scale))
            }
            "pick_model" => {
                let scale = if a.len() >= 5 { need_f(a,4,"PICK_MODEL")? as f32 } else { 1.0 };
                Value::Float(g!().pick_model(gi(a,0,"PICK_MODEL")?,
                    need_f(a,1,"PICK_MODEL")? as f32, need_f(a,2,"PICK_MODEL")? as f32, need_f(a,3,"PICK_MODEL")? as f32, scale))
            }
            "world_to_screen_x" => Value::Float(g!().world_to_screen(
                need_f(a,0,"WORLD_TO_SCREEN_X")? as f32, need_f(a,1,"WORLD_TO_SCREEN_X")? as f32, need_f(a,2,"WORLD_TO_SCREEN_X")? as f32).0 as f64),
            "world_to_screen_y" => Value::Float(g!().world_to_screen(
                need_f(a,0,"WORLD_TO_SCREEN_Y")? as f32, need_f(a,1,"WORLD_TO_SCREEN_Y")? as f32, need_f(a,2,"WORLD_TO_SCREEN_Y")? as f32).1 as f64),
            "screen_to_world_dir_x" => Value::Float(g!().screen_ray_dir(
                need_f(a,0,"SCREEN_TO_WORLD_DIR_X")? as f32, need_f(a,1,"SCREEN_TO_WORLD_DIR_X")? as f32).0 as f64),
            "screen_to_world_dir_y" => Value::Float(g!().screen_ray_dir(
                need_f(a,0,"SCREEN_TO_WORLD_DIR_Y")? as f32, need_f(a,1,"SCREEN_TO_WORLD_DIR_Y")? as f32).1 as f64),
            "screen_to_world_dir_z" => Value::Float(g!().screen_ray_dir(
                need_f(a,0,"SCREEN_TO_WORLD_DIR_Z")? as f32, need_f(a,1,"SCREEN_TO_WORLD_DIR_Z")? as f32).2 as f64),
            "getpixel" => Value::Int(g!().get_pixel(gi(a,0,"GETPIXEL")?, gi(a,1,"GETPIXEL")? as i32, gi(a,2,"GETPIXEL")? as i32)),
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
                // Review-Fund: `semis`/`dur_ms` waren voellig unbeschraenkt.
                // resample() leitet aus `semis` ein Pitch-Verhaeltnis
                // `2^(semis/12)` ab und daraus `out_len = n/ratio` -- ein
                // stark negativer Wert (z.B. -700) macht `ratio` astronomisch
                // klein, `out_len` ueberlief/saettigte auf usize::MAX, und
                // `Vec::with_capacity(out_len)` paniked mit einem
                // Kapazitaets-Ueberlauf (nicht fangbar per TRY/CATCH) --
                // direkt aus einem einzigen, gewoehnlich aussehenden
                // SAMPLE_PLAY-Aufruf heraus. +/-120 Halbtoene (10 Oktaven)
                // ist bereits weit jenseits jedes musikalischen Sinns.
                if !(-120.0..=120.0).contains(&semis) {
                    return Err("SAMPLE_PLAY: Halbtoene muss -120..120 sein".into());
                }
                let vol = if a.len() >= 3 { need_f(a, 2, "SAMPLE_PLAY")? } else { 1.0 };
                let dur = if a.len() >= 4 { gi(a, 3, "SAMPLE_PLAY")? } else { 0 };
                if !(0..=600_000).contains(&dur) {
                    return Err("SAMPLE_PLAY: dur_ms muss 0..600000 sein".into());
                }
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
                Value::Float(round_audio(self.audio_mut()?.get_bus_volume(&bus)?))
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
                // AUDIO_PLAY(sound[, loops[, volume[, fade_in_ms[, easing$]]]]) -- loops=0 einmal (Default), -1 endlos
                let idx = gi(a, 0, "AUDIO_PLAY")?;
                let loops = if a.len() >= 2 { gi(a, 1, "AUDIO_PLAY")? } else { 0 };
                let vol = if a.len() >= 3 { need_f(a, 2, "AUDIO_PLAY")? } else { 1.0 };
                let fade = if a.len() >= 4 { gi(a, 3, "AUDIO_PLAY")? } else { 0 };
                if fade < 0 { return Err("AUDIO_PLAY: fade_in_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 4, "AUDIO_PLAY")?;
                Value::Int(self.audio_mut()?.ch_play(idx, loops, vol, fade, &easing)?)
            }
            "audio_pause" => { let i = gi(a, 0, "AUDIO_PAUSE")?; self.audio_mut()?.ch_pause(i)?; Value::Nil }
            "audio_resume" => { let i = gi(a, 0, "AUDIO_RESUME")?; self.audio_mut()?.ch_resume(i)?; Value::Nil }
            "audio_stop" => {
                // AUDIO_STOP(ch[, fade_out_ms[, easing$]])
                let i = gi(a, 0, "AUDIO_STOP")?;
                let fade = if a.len() >= 2 { gi(a, 1, "AUDIO_STOP")? } else { 0 };
                if fade < 0 { return Err("AUDIO_STOP: fade_out_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 2, "AUDIO_STOP")?;
                self.audio_mut()?.ch_stop(i, fade, &easing)?; Value::Nil
            }
            "audio_is_playing" => { let i = gi(a, 0, "AUDIO_IS_PLAYING")?; Value::Bool(self.audio_mut()?.ch_is_playing(i)?) }
            "audio_volume" | "audio_set_volume" => { let i = gi(a, 0, "AUDIO_VOLUME")?; let v = need_f(a, 1, "AUDIO_VOLUME")?; self.audio_mut()?.ch_set_volume(i, v)?; Value::Nil }
            "audio_get_volume" => { let i = gi(a, 0, "AUDIO_GET_VOLUME")?; Value::Float(round_audio(self.audio_mut()?.ch_get_volume(i)?)) }
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
                // AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms[, easing$]) -- Positionen 0=links..1=rechts
                let i = gi(a, 0, "AUDIO_PAN_SLIDE")?;
                let von = need_f(a, 1, "AUDIO_PAN_SLIDE")?;
                let nach = need_f(a, 2, "AUDIO_PAN_SLIDE")?;
                let dauer = gi(a, 3, "AUDIO_PAN_SLIDE")?;
                if dauer <= 0 { return Err("AUDIO_PAN_SLIDE: dauer_ms muss > 0 sein".into()); }
                let easing = easing_name(a, 4, "AUDIO_PAN_SLIDE")?;
                self.audio_mut()?.ch_pan_slide(i, von, nach, dauer, &easing)?; Value::Nil
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
                // Review-Fund: keine Obergrenze -- `dur` geht direkt in
                // `vec![0.0f64; n]` (n = Samples fuer die Dauer), ein absurd
                // grosser Wert (z.B. eine fehlerhafte Berechnung) fuehrte zu
                // einer Mehrere-GB-Allokation / Allocator-Abort statt eines
                // GB-Fehlers.
                if !(0..=600_000).contains(&dur) {
                    return Err("AUDIO_TONE: dur_ms muss 0..600000 sein".into());
                }
                let wf = if a.len() >= 3 { gs(a, 2, "AUDIO_TONE")?.to_string() } else { "sine".to_string() };
                let vol = if a.len() >= 4 { need_f(a, 3, "AUDIO_TONE")? } else { 1.0 };
                Value::Int(self.audio_mut()?.tone(freq, dur, &wf, vol)?)
            }
            "audio_noise" => {
                let dur = gi(a, 0, "AUDIO_NOISE")?;
                if !(0..=600_000).contains(&dur) {
                    return Err("AUDIO_NOISE: dur_ms muss 0..600000 sein".into());
                }
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
                // AUDIO_MUSIC_PLAY([loops[, fade_in_ms[, easing$]]]) -- loops=-1 endlos (Default)
                let loops = if !a.is_empty() { gi(a, 0, "AUDIO_MUSIC_PLAY")? } else { -1 };
                let fade = if a.len() >= 2 { gi(a, 1, "AUDIO_MUSIC_PLAY")? } else { 0 };
                if fade < 0 { return Err("AUDIO_MUSIC_PLAY: fade_in_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 2, "AUDIO_MUSIC_PLAY")?;
                self.audio_mut()?.music_play(loops, fade, &easing)?; Value::Nil
            }
            "audio_music_stop" => {
                // AUDIO_MUSIC_STOP([fade_out_ms[, easing$]])
                let fade = if !a.is_empty() { gi(a, 0, "AUDIO_MUSIC_STOP")? } else { 0 };
                if fade < 0 { return Err("AUDIO_MUSIC_STOP: fade_out_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 1, "AUDIO_MUSIC_STOP")?;
                self.audio_mut()?.music_stop(fade, &easing)?; Value::Nil
            }
            "audio_music_pause" => { self.audio_mut()?.music_pause(); Value::Nil }
            "audio_music_resume" => { self.audio_mut()?.music_resume(); Value::Nil }
            "audio_music_volume" | "audio_music_set_volume" => { let v = need_f(a, 0, "AUDIO_MUSIC_VOLUME")?; self.audio_mut()?.music_set_volume(v); Value::Nil }
            "audio_music_get_volume" => Value::Float(round_audio(self.audio_mut()?.music_get_volume())),
            "audio_music_position" => Value::Float(self.audio_mut()?.music_position()),
            "audio_music_seek" => {
                let s = need_f(a, 0, "AUDIO_MUSIC_SEEK")?;
                // Nicht `s < 0.0` schreiben -- so faellt NAN mit durch.
                if !(s >= 0.0) {
                    return Err(format!(
                        "AUDIO_MUSIC_SEEK: Position muss >= 0 sein (war {})", s));
                }
                self.audio_mut()?.music_seek(s)?;
                Value::Nil
            }
            "audio_music_busy" => Value::Bool(self.audio_mut()?.music_busy()),
            "audio_music_queue" => { let p = gs(a, 0, "AUDIO_MUSIC_QUEUE")?.to_string(); self.audio_mut()?.music_queue(&p); Value::Nil }

            // --- Clock (Kira-Uhr fuer sample-genaues Musik-/Rhythmus-Timing) ---
            // --- Modulatoren: LFO + Tweener ---------------------------------
            // Kira faehrt sie auf dem Audio-Thread: ein Tremolo/Filter-Sweep
            // laeuft sample-genau weiter, auch wenn ein Frame einbricht -- und
            // das GB-Programm ruft dafuer NICHTS pro Frame.
            "audio_lfo_new" => {
                let wave = gs(a, 0, "AUDIO_LFO_NEW")?.to_string();
                let hz = need_f(a, 1, "AUDIO_LFO_NEW")?;
                if hz < 0.0 { return Err("AUDIO_LFO_NEW: Frequenz darf nicht negativ sein".into()); }
                let amp = if a.len() > 2 { need_f(a, 2, "AUDIO_LFO_NEW")? } else { 1.0 };
                let off = if a.len() > 3 { need_f(a, 3, "AUDIO_LFO_NEW")? } else { 0.0 };
                Value::Int(self.audio_mut()?.lfo_new(&wave, hz, amp, off)?)
            }
            "audio_lfo_set" => {
                let m = gi(a, 0, "AUDIO_LFO_SET")?;
                let hz = if a.len() > 1 { Some(need_f(a, 1, "AUDIO_LFO_SET")?) } else { None };
                let amp = if a.len() > 2 { Some(need_f(a, 2, "AUDIO_LFO_SET")?) } else { None };
                let off = if a.len() > 3 { Some(need_f(a, 3, "AUDIO_LFO_SET")?) } else { None };
                self.audio_mut()?.lfo_set(m, hz, amp, off)?; Value::Nil
            }
            "audio_lfo_waveform" => {
                let m = gi(a, 0, "AUDIO_LFO_WAVEFORM")?;
                let w = gs(a, 1, "AUDIO_LFO_WAVEFORM")?.to_string();
                self.audio_mut()?.lfo_waveform(m, &w)?; Value::Nil
            }
            "audio_tweener_new" => {
                let v = if a.is_empty() { 0.0 } else { need_f(a, 0, "AUDIO_TWEENER_NEW")? };
                Value::Int(self.audio_mut()?.tweener_new(v)?)
            }
            "audio_tweener_to" => {
                let m = gi(a, 0, "AUDIO_TWEENER_TO")?;
                let target = need_f(a, 1, "AUDIO_TWEENER_TO")?;
                let ms = if a.len() > 2 { need_f(a, 2, "AUDIO_TWEENER_TO")? } else { 0.0 };
                let ez = if a.len() > 3 { gs(a, 3, "AUDIO_TWEENER_TO")?.to_string() } else { String::new() };
                self.audio_mut()?.tweener_to(m, target, ms, &ez)?; Value::Nil
            }
            "audio_mod_remove" => {
                let m = gi(a, 0, "AUDIO_MOD_REMOVE")?;
                self.audio_mut()?.mod_remove(m)?; Value::Nil
            }
            "audio_modulate" => {
                let bus = gs(a, 0, "AUDIO_MODULATE")?.to_string();
                let target = gs(a, 1, "AUDIO_MODULATE")?.to_string();
                let m = gi(a, 2, "AUDIO_MODULATE")?;
                let lo = need_f(a, 3, "AUDIO_MODULATE")?;
                let hi = need_f(a, 4, "AUDIO_MODULATE")?;
                self.audio_mut()?.modulate(&bus, &target, m, lo, hi)?; Value::Nil
            }
            "audio_bus_pan" => {
                let bus = gs(a, 0, "AUDIO_BUS_PAN")?.to_string();
                let pos = need_f(a, 1, "AUDIO_BUS_PAN")?;
                self.audio_mut()?.bus_pan(&bus, pos)?; Value::Nil
            }
            "audio_clock_new" => {
                // AUDIO_CLOCK_NEW(ticks_per_second) -- Wertpruefung VOR der
                // Audio-Initialisierung (golden-testbar).
                let tps = need_f(a, 0, "AUDIO_CLOCK_NEW")?;
                if tps <= 0.0 { return Err("AUDIO_CLOCK_NEW: ticks_per_second muss > 0 sein".into()); }
                Value::Int(self.audio_mut()?.clock_new(tps)?)
            }
            "audio_clock_start" => { let c = gi(a, 0, "AUDIO_CLOCK_START")?; self.audio_mut()?.clock_start(c)?; Value::Nil }
            "audio_clock_pause" => { let c = gi(a, 0, "AUDIO_CLOCK_PAUSE")?; self.audio_mut()?.clock_pause(c)?; Value::Nil }
            "audio_clock_stop" => { let c = gi(a, 0, "AUDIO_CLOCK_STOP")?; self.audio_mut()?.clock_stop(c)?; Value::Nil }
            "audio_clock_ticking" => { let c = gi(a, 0, "AUDIO_CLOCK_TICKING")?; Value::Bool(self.audio_mut()?.clock_ticking(c)?) }
            "audio_clock_ticks" => { let c = gi(a, 0, "AUDIO_CLOCK_TICKS")?; Value::Int(self.audio_mut()?.clock_ticks(c)?) }
            "audio_clock_set_speed" => {
                let c = gi(a, 0, "AUDIO_CLOCK_SET_SPEED")?;
                let tps = need_f(a, 1, "AUDIO_CLOCK_SET_SPEED")?;
                if tps <= 0.0 { return Err("AUDIO_CLOCK_SET_SPEED: ticks_per_second muss > 0 sein".into()); }
                self.audio_mut()?.clock_set_speed(c, tps)?; Value::Nil
            }
            "audio_clock_remove" => { let c = gi(a, 0, "AUDIO_CLOCK_REMOVE")?; self.audio_mut()?.clock_remove(c)?; Value::Nil }
            "audio_play_at" => {
                // AUDIO_PLAY_AT(sound, clock, ticks[, volume[, loops]]) -- Start
                // exakt auf Tick `ticks` der Uhr `clock` statt sofort.
                let idx = gi(a, 0, "AUDIO_PLAY_AT")?;
                let clock = gi(a, 1, "AUDIO_PLAY_AT")?;
                let ticks = gi(a, 2, "AUDIO_PLAY_AT")?;
                let vol = if a.len() >= 4 { need_f(a, 3, "AUDIO_PLAY_AT")? } else { 1.0 };
                let loops = if a.len() >= 5 { gi(a, 4, "AUDIO_PLAY_AT")? } else { 0 };
                if ticks < 0 { return Err("AUDIO_PLAY_AT: ticks muss >= 0 sein".into()); }
                Value::Int(self.audio_mut()?.ch_play_at(idx, clock, ticks, vol, loops)?)
            }

            // --- Listener/Emitter (raeumliches Audio) ---
            "audio_listener_new" => {
                let x = need_f(a, 0, "AUDIO_LISTENER_NEW")?;
                let y = need_f(a, 1, "AUDIO_LISTENER_NEW")?;
                let z = need_f(a, 2, "AUDIO_LISTENER_NEW")?;
                Value::Int(self.audio_mut()?.listener_new(x, y, z)?)
            }
            "audio_listener_set_position" => {
                let l = gi(a, 0, "AUDIO_LISTENER_SET_POSITION")?;
                let x = need_f(a, 1, "AUDIO_LISTENER_SET_POSITION")?;
                let y = need_f(a, 2, "AUDIO_LISTENER_SET_POSITION")?;
                let z = need_f(a, 3, "AUDIO_LISTENER_SET_POSITION")?;
                self.audio_mut()?.listener_set_position(l, x, y, z)?; Value::Nil
            }
            "audio_listener_set_orientation" => {
                let l = gi(a, 0, "AUDIO_LISTENER_SET_ORIENTATION")?;
                let yaw = need_f(a, 1, "AUDIO_LISTENER_SET_ORIENTATION")?;
                self.audio_mut()?.listener_set_orientation(l, yaw)?; Value::Nil
            }
            "audio_listener_remove" => {
                let l = gi(a, 0, "AUDIO_LISTENER_REMOVE")?;
                self.audio_mut()?.listener_remove(l)?; Value::Nil
            }
            "audio_emitter_new" => {
                // AUDIO_EMITTER_NEW(listener, x, y, z[, min_dist[, max_dist]])
                let l = gi(a, 0, "AUDIO_EMITTER_NEW")?;
                let x = need_f(a, 1, "AUDIO_EMITTER_NEW")?;
                let y = need_f(a, 2, "AUDIO_EMITTER_NEW")?;
                let z = need_f(a, 3, "AUDIO_EMITTER_NEW")?;
                let min_d = if a.len() >= 5 { need_f(a, 4, "AUDIO_EMITTER_NEW")? } else { 1.0 };
                let max_d = if a.len() >= 6 { need_f(a, 5, "AUDIO_EMITTER_NEW")? } else { 100.0 };
                if min_d < 0.0 { return Err("AUDIO_EMITTER_NEW: min_dist muss >= 0 sein".into()); }
                if max_d <= min_d { return Err("AUDIO_EMITTER_NEW: max_dist muss > min_dist sein".into()); }
                Value::Int(self.audio_mut()?.emitter_new(l, x, y, z, min_d, max_d)?)
            }
            "audio_emitter_set_position" => {
                let e = gi(a, 0, "AUDIO_EMITTER_SET_POSITION")?;
                let x = need_f(a, 1, "AUDIO_EMITTER_SET_POSITION")?;
                let y = need_f(a, 2, "AUDIO_EMITTER_SET_POSITION")?;
                let z = need_f(a, 3, "AUDIO_EMITTER_SET_POSITION")?;
                self.audio_mut()?.emitter_set_position(e, x, y, z)?; Value::Nil
            }
            "audio_emitter_remove" => {
                let e = gi(a, 0, "AUDIO_EMITTER_REMOVE")?;
                self.audio_mut()?.emitter_remove(e)?; Value::Nil
            }
            "audio_play_on" => {
                // AUDIO_PLAY_ON(sound, emitter[, loops[, volume[, fade_in_ms[, easing$]]]])
                let idx = gi(a, 0, "AUDIO_PLAY_ON")?;
                let emitter = gi(a, 1, "AUDIO_PLAY_ON")?;
                let loops = if a.len() >= 3 { gi(a, 2, "AUDIO_PLAY_ON")? } else { 0 };
                let vol = if a.len() >= 4 { need_f(a, 3, "AUDIO_PLAY_ON")? } else { 1.0 };
                let fade = if a.len() >= 5 { gi(a, 4, "AUDIO_PLAY_ON")? } else { 0 };
                if fade < 0 { return Err("AUDIO_PLAY_ON: fade_in_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 5, "AUDIO_PLAY_ON")?;
                Value::Int(self.audio_mut()?.ch_play_on(idx, emitter, loops, vol, fade, &easing)?)
            }

            // --- Bulk-Draws ---
            "plots" => {
                let xs = arr_i32(&a[0], "PLOTS")?; let ys = arr_i32(&a[1], "PLOTS")?;
                let n = bulk_count(a, 3, xs.len().min(ys.len()), "PLOTS")?;
                let cols = bulk_color(&a[2], n, "PLOTS")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.plot(xs[i], ys[i], cols[i]); }
                Value::Nil
            }
            "boxes" => {
                let x1=arr_i32(&a[0],"BOXES")?; let y1=arr_i32(&a[1],"BOXES")?; let x2=arr_i32(&a[2],"BOXES")?; let y2=arr_i32(&a[3],"BOXES")?;
                let n = bulk_count(a, 5, x1.len().min(y1.len()).min(x2.len()).min(y2.len()), "BOXES")?;
                let cols = bulk_color(&a[4], n, "BOXES")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.box_fill(x1[i], y1[i], x2[i], y2[i], cols[i]); }
                Value::Nil
            }
            "circles" => {
                let xs=arr_i32(&a[0],"CIRCLES")?; let ys=arr_i32(&a[1],"CIRCLES")?; let rs=arr_i32(&a[2],"CIRCLES")?;
                let n = bulk_count(a, 4, xs.len().min(ys.len()).min(rs.len()), "CIRCLES")?;
                let cols = bulk_color(&a[3], n, "CIRCLES")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.circle(xs[i], ys[i], rs[i], cols[i]); }
                Value::Nil
            }
            "lines" => {
                let x1=arr_i32(&a[0],"LINES")?; let y1=arr_i32(&a[1],"LINES")?; let x2=arr_i32(&a[2],"LINES")?; let y2=arr_i32(&a[3],"LINES")?;
                let n = bulk_count(a, 5, x1.len().min(y1.len()).min(x2.len()).min(y2.len()), "LINES")?;
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
            "drawimagepartex" => {
                let idx = gi(a,0,"DRAWIMAGEPARTEX")?;
                g!().draw_image_part_ex(idx, gi(a,1,"DRAWIMAGEPARTEX")? as i32, gi(a,2,"DRAWIMAGEPARTEX")? as i32,
                    gi(a,3,"DRAWIMAGEPARTEX")? as i32, gi(a,4,"DRAWIMAGEPARTEX")? as i32,
                    gi(a,5,"DRAWIMAGEPARTEX")? as i32, gi(a,6,"DRAWIMAGEPARTEX")? as i32,
                    gi(a,7,"DRAWIMAGEPARTEX")? as i32, gi(a,8,"DRAWIMAGEPARTEX")? as i32)?; Value::Nil
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
                g!().atlas_draw(atlas, &name, gi(a,2,"ATLAS_DRAW")? as i32, gi(a,3,"ATLAS_DRAW")? as i32, false, false, tint)?; Value::Nil
            }
            "atlas_draw_flipped" => {
                // ATLAS_DRAW_FLIPPED(atlas, name, x, y[, flip_x[, flip_y[, tint]]])
                // flip_x/flip_y akzeptieren TRUE/FALSE und 1/0; tint optional (7. Arg).
                let atlas = gi(a,0,"ATLAS_DRAW_FLIPPED")?;
                let name = gs(a,1,"ATLAS_DRAW_FLIPPED")?.to_string();
                let fx = gflag(a, 4);
                let fy = gflag(a, 5);
                let tint = if a.len() > 6 { Some(gi(a,6,"ATLAS_DRAW_FLIPPED")?) } else { None };
                g!().atlas_draw(atlas, &name, gi(a,2,"ATLAS_DRAW_FLIPPED")? as i32, gi(a,3,"ATLAS_DRAW_FLIPPED")? as i32, fx, fy, tint)?; Value::Nil
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
                let zoom = if a.len() >= 3 { let z = need_f(a, 2, "CAMERA_SET")?; if z <= 0.0 { return Err("CAMERA_SET: zoom muss > 0 sein".into()); } z } else { 1.0 };
                g!().set_camera(x, y, zoom);
                if a.len() >= 4 { g!().set_camera_rotation(need_f(a, 3, "CAMERA_SET")?); }
                Value::Nil
            }
            "camera_reset" => { g!().reset_camera(); Value::Nil }
            "camera_set_rotation" => { g!().set_camera_rotation(need_f(a, 0, "CAMERA_SET_ROTATION")?); Value::Nil }
            "camera_rotation" => Value::Float(g!().camera_rotation()),
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
            // Zweites Argument (die jeweils andere Screen-Achse) ist optional --
            // ohne Rotation identisch zur alten Ein-Argument-Form; MIT aktiver
            // CAMERA_SET_ROTATION braucht die korrekte Umkehrung beide Werte
            // (Rotation mischt x/y), siehe s2w_x_rot/s2w_y_rot.
            "camera_s2w_x" => {
                let sx = need_f(a, 0, "CAMERA_S2W_X")?;
                Value::Float(if a.len() >= 2 { g!().s2w_x_rot(sx, need_f(a, 1, "CAMERA_S2W_X")?) } else { g!().s2w_x(sx) })
            }
            "camera_s2w_y" => {
                let sy = need_f(a, 0, "CAMERA_S2W_Y")?;
                Value::Float(if a.len() >= 2 { g!().s2w_y_rot(need_f(a, 1, "CAMERA_S2W_Y")?, sy) } else { g!().s2w_y(sy) })
            }

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
            "image_scale_nn" => Value::Int(g!().image_scale_nn(gi(a,0,"IMAGE_SCALE_NN")?, gi(a,1,"IMAGE_SCALE_NN")? as i32, gi(a,2,"IMAGE_SCALE_NN")? as i32)?),
            "image_rotate" => Value::Int(g!().image_rotate(gi(a,0,"IMAGE_ROTATE")?, need_f(a,1,"IMAGE_ROTATE")? as f32)?),
            "image_flip" => Value::Int(g!().image_flip(gi(a,0,"IMAGE_FLIP")?, gb(a,1), gb(a,2))?),
            "image_tint" => { let c = gi(a,1,"IMAGE_TINT")?; if c < 0 || c > 0xFFFFFF { return Err("IMAGE_TINT: Farbe muss 0..0xFFFFFF sein".into()); } Value::Int(g!().image_tint(gi(a,0,"IMAGE_TINT")?, c)?) }
            "image_copy" => Value::Int(g!().image_copy(gi(a,0,"IMAGE_COPY")?)?),
            "image_crop" => Value::Int(g!().image_crop(gi(a,0,"IMAGE_CROP")?, gi(a,1,"IMAGE_CROP")? as i32, gi(a,2,"IMAGE_CROP")? as i32, gi(a,3,"IMAGE_CROP")? as i32, gi(a,4,"IMAGE_CROP")? as i32)?),
            "image_resize_canvas" => {
                let fill = if a.len() >= 6 { gi(a,5,"IMAGE_RESIZE_CANVAS")? } else { 0 };
                Value::Int(g!().image_resize_canvas(gi(a,0,"IMAGE_RESIZE_CANVAS")?, gi(a,1,"IMAGE_RESIZE_CANVAS")? as i32, gi(a,2,"IMAGE_RESIZE_CANVAS")? as i32, gi(a,3,"IMAGE_RESIZE_CANVAS")? as i32, gi(a,4,"IMAGE_RESIZE_CANVAS")? as i32, fill)?)
            }
            "image_convolve" => {
                let k = gfloats(a, 1, "IMAGE_CONVOLVE")?;
                Value::Int(g!().image_convolve(gi(a,0,"IMAGE_CONVOLVE")?, &k)?)
            }
            "image_alpha_mask" => Value::Int(g!().image_alpha_mask(
                gi(a,0,"IMAGE_ALPHA_MASK")?, gi(a,1,"IMAGE_ALPHA_MASK")?)?),
            "image_alpha_crop" => Value::Int(g!().image_alpha_crop(
                gi(a,0,"IMAGE_ALPHA_CROP")?, need_f(a,1,"IMAGE_ALPHA_CROP")?)?),
            "image_alpha_premultiply" => Value::Int(g!().image_alpha_premultiply(
                gi(a,0,"IMAGE_ALPHA_PREMULTIPLY")?)?),
            "image_dither" => Value::Int(g!().image_dither(
                gi(a,0,"IMAGE_DITHER")?, gi(a,1,"IMAGE_DITHER")?, gi(a,2,"IMAGE_DITHER")?,
                gi(a,3,"IMAGE_DITHER")?, gi(a,4,"IMAGE_DITHER")?)?),
            "image_palette" => {
                let cols = g!().image_palette(gi(a,0,"IMAGE_PALETTE")?, gi(a,1,"IMAGE_PALETTE")?)?;
                crate::builtins::new_int_array(cols)
            }
            "image_blur" => Value::Int(g!().image_blur(gi(a,0,"IMAGE_BLUR")?, gi(a,1,"IMAGE_BLUR")? as i32)?),
            "image_brightness" => Value::Int(g!().image_brightness(gi(a,0,"IMAGE_BRIGHTNESS")?, gi(a,1,"IMAGE_BRIGHTNESS")? as i32)?),
            "image_contrast" => Value::Int(g!().image_contrast(gi(a,0,"IMAGE_CONTRAST")?, need_f(a,1,"IMAGE_CONTRAST")? as f32)?),
            "image_grayscale" => Value::Int(g!().image_grayscale(gi(a,0,"IMAGE_GRAYSCALE")?)?),
            "image_invert" => Value::Int(g!().image_invert(gi(a,0,"IMAGE_INVERT")?)?),
            "image_replace_color" => Value::Int(g!().image_replace_color(gi(a,0,"IMAGE_REPLACE_COLOR")?, gi(a,1,"IMAGE_REPLACE_COLOR")?, gi(a,2,"IMAGE_REPLACE_COLOR")?)?),
            "image_draw_line" => { g!().image_draw_line(gi(a,0,"IMAGE_DRAW_LINE")?, gi(a,1,"IMAGE_DRAW_LINE")? as i32, gi(a,2,"IMAGE_DRAW_LINE")? as i32, gi(a,3,"IMAGE_DRAW_LINE")? as i32, gi(a,4,"IMAGE_DRAW_LINE")? as i32, gi(a,5,"IMAGE_DRAW_LINE")?)?; Value::Nil }
            "image_draw_circle" => { g!().image_draw_circle(gi(a,0,"IMAGE_DRAW_CIRCLE")?, gi(a,1,"IMAGE_DRAW_CIRCLE")? as i32, gi(a,2,"IMAGE_DRAW_CIRCLE")? as i32, gi(a,3,"IMAGE_DRAW_CIRCLE")? as i32, gi(a,4,"IMAGE_DRAW_CIRCLE")?)?; Value::Nil }
            "image_draw_rect" => { g!().image_draw_rect(gi(a,0,"IMAGE_DRAW_RECT")?, gi(a,1,"IMAGE_DRAW_RECT")? as i32, gi(a,2,"IMAGE_DRAW_RECT")? as i32, gi(a,3,"IMAGE_DRAW_RECT")? as i32, gi(a,4,"IMAGE_DRAW_RECT")? as i32, gi(a,5,"IMAGE_DRAW_RECT")?)?; Value::Nil }
            "image_draw_text" => {
                let txt = gs(a,3,"IMAGE_DRAW_TEXT")?.to_string();
                g!().image_draw_text(gi(a,0,"IMAGE_DRAW_TEXT")?, gi(a,1,"IMAGE_DRAW_TEXT")? as i32, gi(a,2,"IMAGE_DRAW_TEXT")? as i32, &txt, gi(a,4,"IMAGE_DRAW_TEXT")? as i32, gi(a,5,"IMAGE_DRAW_TEXT")?)?;
                Value::Nil
            }

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
                let pl = self.ui_state.plastik();
                let g = self.gfx.as_mut().unwrap();
                ui_flaeche(g, pl, x, y, w, h, bg, fg_color, false);
                // Beschriftung mittig und im Knopf beschnitten -- wie im
                // Modul `gui`; links anzukleben und ueberzulaufen sieht bei
                // Knoepfen falsch aus.
                let sz = g.text_height();
                let tw = g.text_width(&text);
                let frei = (w - 12).max(0);
                let tx = x + 6 + ((frei - tw) / 2).max(0);
                let beschnitten = tw > frei;
                if beschnitten { g.push_clip(x + 6, y + 1, frei, (h - 2).max(0)); }
                g.text(tx, y + (h - sz) / 2, text, fg_color);
                if beschnitten { g.pop_clip(); }
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
                let feld = self.ui_state.th("field_bg");
                let pl = self.ui_state.plastik();
                let g = self.gfx.as_mut().unwrap();
                ui_flaeche(g, pl, x, y, bs, bs, feld, border, true);
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
                let (track, border, handle) = (self.ui_state.th("slider_track"), self.ui_state.th("field_border"), self.ui_state.th("accent"));
                let handle_w = 10;
                let hx = x + ((val - mn) / (mx_ - mn) * (w - handle_w) as f64) as i32;
                let pl = self.ui_state.plastik();
                let g = self.gfx.as_mut().unwrap();
                if pl.grad > 0 {
                    ui_flaeche(g, pl, x, y + h / 2 - 3, w, 6, track, border, true);
                    ui_flaeche(g, pl, hx, y, handle_w, h, handle, border, false);
                } else {
                    g.box_fill(x, y + h/2 - 1, x+w-1, y+h/2+1, track);
                    g.rect(x,y,x+w-1,y+h-1,border);
                    g.box_fill(hx, y, hx+handle_w-1, y+h-1, handle);
                }
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
                    let tfg = self.ui_state.th("field_border");
                    let pl = self.ui_state.plastik();
                    let g = self.gfx.as_mut().unwrap();
                    ui_flaeche(g, pl, x, y, w, h, bg, tfg, true);
                    if fill_w > 0 {
                        ui_flaeche(g, pl, x + 1, y + 1, fill_w, (h - 2).max(1), fg, fg, false);
                    }
                }
                Value::Nil
            }
            "ui_panel" => {
                let (x,y,w,h) = (gi(a,0,"UI_PANEL")? as i32 + self.ui_state.offset_x, gi(a,1,"UI_PANEL")? as i32 + self.ui_state.offset_y, gi(a,2,"UI_PANEL")? as i32, gi(a,3,"UI_PANEL")? as i32);
                let title = if a.len() >= 5 { gs(a,4,"UI_PANEL")?.to_string() } else { String::new() };
                let bg = if a.len() >= 6 { gi(a,5,"UI_PANEL")? } else { self.ui_state.th("panel_bg") };
                let (border, tbg, fg) = (self.ui_state.th("panel_border"), self.ui_state.th("panel_title_bg"), self.ui_state.th("text_fg"));
                let pl = self.ui_state.plastik();
                let g = self.gfx.as_mut().unwrap();
                ui_flaeche(g, pl, x, y, w, h, bg, border, false);
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
                        if arr.element_type != "string" && arr.cells.len() > 0 { return Err("UI_RADIO: options muss ARRAY OF STRING sein".into()); }
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
                    let pl = self.ui_state.plastik();
                    let g = self.gfx.as_mut().ok_or("UI_TEXTFIELD vor SCREEN")?;
                    ui_flaeche(g, pl, x, y, w, h, bg, border, true);
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
                let pl = self.ui_state.plastik();
                {
                    let g = self.gfx.as_mut().ok_or("UI_WINDOW_BEGIN vor SCREEN")?;
                    ui_flaeche(g, pl, wx, wy, w, draw_h, win_bg, win_border, false);
                    // Titelleiste: oben rund wie das Fenster, unten buendig --
                    // deshalb hier ohne eigene Rundung.
                    ui_flaeche(g, UiPlastik { rad: 0, ..pl }, wx, wy, w, title_h, title_bg, win_border, false);
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
                // Ein Preset ist ein KOMPLETTER Look: Farben UND Plastik.
                for (k, v) in ui_preset_metrics(&n) { self.ui_state.metrics.insert(k.to_string(), v); }
                Value::Nil
            }

            "chart_draw" => {
                if a.len() != 1 { return Err("CHART_DRAW: erwartet 1 Argument -- Aufruf: CHART_DRAW(diagramm)".into()); }
                let c = crate::builtins::chart_h(&a[0], "CHART_DRAW")?.clone();
                // CHART_DRAW wertet auch die Maus aus und schreibt Hover/Klick
                // ins Handle zurueck -> veraenderlicher Zugriff.
                let mut c = c.borrow_mut();
                let g = self.gfx.as_mut().ok_or("CHART_DRAW vor SCREEN aufgerufen")?;
                c.draw(g);
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

// Vordefinierte Globals -- Werte IDENTISCH zu drachenhauch/graphics.py
// (COLORS/KEYS). Von Hand synchron; Drift-Schutz: tests/test_constants_sync.py
// vergleicht jede Python-Konstante gegen PRINT-Output von dhrt.
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
    // Umschalt-/Steuertasten: bis hierher gab es KEINEN Code dafuer -- ein
    // "Sprint mit Shift" oder "Strg+S" war aus GB heraus nicht abfragbar.
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

// ===========================================================================
// Helfer
// ===========================================================================

fn unknown_builtin_msg(name: &str) -> String {
    // Hardware-/IoT-Module sind hinter Cargo-Features (serial/usb/bt/wifi) und im
    // Default-Build NICHT enthalten -- der Dispatch faellt dann hierher durch. Das
    // Builtin EXISTIERT (in dhrt implementiert), es fehlt nur im aktuellen Build.
    // Klare, handlungsleitende Meldung statt "noch nicht verfuegbar".
    let hw_feature = if name.starts_with("serial_") { Some("serial") }
        else if name.starts_with("usb_") { Some("usb") }
        else if name.starts_with("bt_") { Some("bt") }
        else if name.starts_with("wifi_") { Some("wifi") }
        else { None };
    if let Some(feat) = hw_feature {
        return format!(
            "Builtin '{}' gehoert zum Hardware-Modul '{}', das in diesem dhrt-Build \
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

fn int_pair2(stack: &mut Vec<Value>, op: &str) -> R<(i64, i64)> {
    let b = vm_pop(stack)?;
    let a = vm_pop(stack)?;
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => Ok((*x, *y)),
        _ => Err(format!("{} erwartet INTEGER, erhalten {} / {}", op, a.type_name(), b.type_name())),
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
    // Review-Fund: STRING ist unter keinem dieser Operatoren ein gueltiger
    // Vektor-Operand -- ohne diesen fruehen Bail griff `module_op` bei z.B.
    // `"pos: " + v` (v ein VEC2) trotzdem zu (is_mod(b)==true), landete im
    // Catch-all-Arm des '+'-Zweigs und lieferte `Some(Err("inkompatible
    // Vektor-Operanden"))` -- das hat den eigentlich vorgesehenen String-
    // Konkatenations-Fallback (und jedes User-`OPERATOR +`, das STRING als
    // `other` akzeptiert) NIE erreichen lassen.
    if matches!(a, Value::Str(_)) || matches!(b, Value::Str(_)) {
        return None;
    }
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
            if *n <= 0 { return Ok(Value::str_rc("")); }
            // Review-Fund: `String::repeat` abortiert den Prozess bei
            // Kapazitaets-Ueberlauf statt einen Fehler zu liefern -- ein
            // Deckel verhindert das bei absurd grossem `n` (z.B. aus einer
            // fehlerhaften Berechnung).
            const MAX_REPEAT_LEN: usize = 10_000_000;
            return match s.len().checked_mul(*n as usize) {
                Some(total) if total <= MAX_REPEAT_LEN => Ok(Value::str_rc(&s.repeat(*n as usize))),
                _ => Err(format!("* : String-Wiederholung zu gross (max. {} Zeichen)", MAX_REPEAT_LEN)),
            };
        }
        _ => {}
    }
    require_number(&a, &b, "*")?;
    nn_arith(a, b, '*')
}

/// Rundet f32-gestuetzte Audio-Werte (Lautstaerken 0..1) auf 6 Nachkommastellen,
/// damit die f32->f64-Verbreiterung nicht als „0.800000011920929" durchschlaegt
/// (siehe docs/drachenhauch-stolpersteine.md D2). 6 Stellen reichen fuer Volumes weit.
fn round_audio(f: f64) -> f64 { (f * 1_000_000.0).round() / 1_000_000.0 }

/// Baut aus den Werten eines Array-Literals `[a, b, c]` ein 1D-GbArray.
/// Element-Typ wird aus den Werten hergeleitet (wie ein homogenes GB-Array):
/// nur Ganzzahlen -> integer; Zahlen mit mind. einem Float -> float (Ints
/// werden hochgezogen); nur Strings -> string; nur Wahrheitswerte -> boolean;
/// gemischt -> any (generisches Value-Backing).
fn array_literal(vals: Vec<Value>) -> Value {
    let n = vals.len() as i64;
    let all_int = vals.iter().all(|v| matches!(v, Value::Int(_)));
    let all_num = vals.iter().all(|v| matches!(v, Value::Int(_) | Value::Float(_)));
    let all_str = vals.iter().all(|v| matches!(v, Value::Str(_)));
    let all_bool = vals.iter().all(|v| matches!(v, Value::Bool(_)));
    // Review-Fund: bei einem leeren Literal `[]` sind all_int/all_num/all_str/
    // all_bool alle vacuous-true (Iterator::all() auf einem leeren Iterator),
    // und all_int gewinnt per Reihenfolge -- ein leeres Array wurde also immer
    // als ARRAY OF INTEGER getypt, ein spaeteres ARRAY_PUSH(a, "x") scheiterte
    // dann mit einer fuer den Nutzer unerklaerlichen Typ-Fehlermeldung.
    let (etype, cells) = if vals.is_empty() {
        ("any", Cells::Val(vals))
    } else if all_int {
        ("integer", Cells::Int(vals.iter()
            .map(|v| if let Value::Int(i) = v { *i } else { 0 }).collect()))
    } else if all_num {
        ("float", Cells::Float(vals.iter().map(|v| match v {
            Value::Int(i) => *i as f64, Value::Float(f) => *f, _ => 0.0 }).collect()))
    } else if all_str {
        ("string", Cells::Val(vals))
    } else if all_bool {
        ("boolean", Cells::Val(vals))
    } else {
        ("any", Cells::Val(vals))
    };
    let arr = GbArray { element_type: etype.to_string(), dims: vec![n], strides: vec![1], cells };
    Value::Array(Rc::new(RefCell::new(arr)))
}

fn div(a: Value, b: Value) -> R<Value> {
    require_number(&a, &b, "/")?;
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            if *y == 0 { return Err("Division durch 0".into()); }
            // Review-Fund: i64::MIN / -1 ist UB-nah und paniked in Rust IMMER
            // (unabhaengig von overflow-checks) -- checked_div faengt das ab.
            let q = x.checked_div(*y).ok_or_else(|| int_overflow_msg("/"))?;
            let r = x.checked_rem(*y).ok_or_else(|| int_overflow_msg("/"))?;
            if r == 0 { Ok(Value::Int(q)) } else { Ok(Value::Float(*x as f64 / *y as f64)) }
        }
        _ => { let y = as_f64(&b); if y == 0.0 { return Err("Division durch 0".into()); } Ok(Value::Float(as_f64(&a) / y)) }
    }
}

fn int_div(a: Value, b: Value) -> R<Value> {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            if *y == 0 { return Err("Integer-Division durch 0".into()); }
            x.checked_div(*y).map(Value::Int).ok_or_else(|| int_overflow_msg("\\"))
        }
        // Review-Fund: dieser Zweig faengt JEDEN nicht-Int/Int-Fall ab, nicht
        // nur FLOAT -- die Meldung nannte faelschlich immer "FLOAT" auch fuer
        // BOOLEAN/STRING-Operanden.
        _ => Err(format!("\\ erwartet INTEGER, erhalten {} / {}", a.type_name(), b.type_name())),
    }
}

fn modulo(a: Value, b: Value) -> R<Value> {
    require_number(&a, &b, "MOD")?;
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            if *y == 0 { return Err("MOD durch 0".into()); }
            let m = x.checked_rem(*y).ok_or_else(|| int_overflow_msg("MOD"))?;
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
                if f.fract() != 0.0 {
                    Err(format!("{}: FLOAT {} passt nicht verlustfrei in INTEGER -- fuer ganzzahlige Division \\ statt / nehmen, sonst mit INT()/ROUND() runden", ctx, f))
                // Review-Fund: `f as i64` saettigt in Rust (seit 1.45) statt zu
                // ueberlaufen -- ohne diese Grenzpruefung wurde z.B. `1e19`
                // (fract()==0.0, aber weit ausserhalb i64) still zu
                // i64::MAX statt zu einem Fehler.
                } else if f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                    Ok(Value::Int(f as i64))
                } else {
                    Err(format!("{}: {}", ctx, int_overflow_msg("Zuweisung")))
                }
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
        // Pascal-Striktheit (dhrt-Haertung): TUPLE/FUNCREF nur mit passendem
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
        // map:/Klassen/sonstige -> Durchreichen (Referenz-Typen).
        _ => match target.strip_prefix("array:") {
            Some(elem) => coerce_array(value, elem, ctx),
            None => Ok(value),
        },
    }
}

/// Einen internen Typnamen so schreiben, wie er im Quelltext steht
/// (`array:integer` -> `ARRAY OF INTEGER`). Gegenstueck zu
/// `Compiler::typ_klartext` -- die interne Schreibweise sagt einem Nutzer nichts.
fn typ_lesbar(t: &str) -> String {
    match t.strip_prefix("array:") {
        Some(e) => format!("ARRAY OF {}", typ_lesbar(e)),
        None => t.to_uppercase(),
    }
}

/// Zuweisung an ein ARRAY-Ziel pruefen (`DIM a AS ARRAY OF STRING : a = ...`).
///
/// Frueher reichte coerce() alle Referenz-Typen einfach durch. Damit ging
/// `DIM texte AS ARRAY OF STRING : texte = zahlen` still durch, obwohl
/// `zahlen` ein ARRAY OF INTEGER ist -- bei einfachen Werten meldet die
/// Runtime so etwas seit jeher ("Erwartet STRING, erhalten INTEGER"), bei
/// Arrays nicht. Der Fehler fiel dann weit entfernt auf, beim Lesen eines
/// Elements, das den falschen Typ hatte.
///
/// Erlaubt bleibt, was nicht schiefgehen kann:
///   * NIL -- ein sizeless ARRAY ist bis zur ersten Zuweisung NIL.
///   * Ein leeres Literal `[]` kommt ohne Elementtyp an ("any") und bekommt
///     hier den des Ziels.
///   * Ein ARRAY OF ANY. Es kann jeden Wert enthalten; es einem engeren Ziel
///     zuzuweisen ist eine bewusste Entscheidung des Programmierers, so wie
///     das Auspacken einer Map. Erst der Schreibzugriff prueft wieder.
///   * Ein frisches Ganzzahl-Literal an einem FLOAT-Ziel (siehe unten).
fn coerce_array(value: Value, elem: &str, ctx: &str) -> R<Value> {
    if matches!(value, Value::Nil) { return Ok(value); }
    let ziel = if elem.is_empty() { "ARRAY".to_string() }
               else { format!("ARRAY OF {}", typ_lesbar(elem)) };
    let arr = match &value {
        Value::Array(a) => a,
        _ => return Err(format!("{}: Erwartet {}, erhalten {}",
                                ctx, ziel, value.type_name())),
    };
    // Ziel ohne Elementtyp (ARRAY OF ANY) nimmt jedes Array.
    if elem.is_empty() || elem == "any" { return Ok(value); }

    let ist = {
        let mut a = arr.borrow_mut();
        if a.element_type == "any" && a.cells.len() == 0 {
            a.element_type = elem.to_string();
        // Ein frisches `[1, 2, 3]` an einem FLOAT-Ziel: die Zellen umbauen,
        // statt die Zuweisung abzulehnen. Nur wenn dieses Array sonst
        // niemandem gehoert (strong_count == 1, also ein Literal von eben) --
        // ein vorhandenes ARRAY OF INTEGER darf nicht unter der Hand zu FLOAT
        // werden, sein bisheriger Name zeigt ja weiter auf dieselben Zellen.
        } else if a.element_type == "integer" && elem == "float"
                  && Rc::strong_count(arr) == 1 {
            let werte: Vec<f64> = a.cells.iter().map(|v| as_f64(&v)).collect();
            a.cells = Cells::Float(werte);
            a.element_type = "float".to_string();
        }
        a.element_type.clone()
    };
    if ist == elem || ist == "any" { return Ok(value); }
    Err(format!("{}: Erwartet {}, erhalten ARRAY OF {}", ctx, ziel, typ_lesbar(&ist)))
}
