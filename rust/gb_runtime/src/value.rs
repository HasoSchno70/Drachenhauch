//! GameBasic-Laufzeitwerte fuer die Rust-VM.
//!
//! Skalare (Int/Float/Str/Bool/Tuple) sind Wert-Typen (immutable). Arrays,
//! Maps und Instanzen sind Referenz-Typen (`Rc<RefCell<…>>`) -- Zuweisung
//! aliased, genau wie in der Python-VM (dort dasselbe Objekt im Slot).
//!
//! `fmt`/`truthy` muessen bit-identisch zu `gamebasic/vm.py` sein.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::model::Func;

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Rc<str>),
    Tuple(Rc<Vec<Value>>),
    FuncRef(Rc<str>),
    CompMarker,
    Array(Rc<RefCell<GbArray>>),
    Map(Rc<RefCell<GbMap>>),
    Instance(Rc<RefCell<Instance>>),
    /// Modul `vec2`: immutabler 2D-Vektor (Wert-Semantik wie ein Skalar).
    Vec2(f64, f64),
    /// ENUM- / STATIC-CONST-Namespace (`State.PLAYING`, `Player.MAX_HP`).
    /// Liegt als CONST-Wert im Programm; MemberAccess loest Member auf.
    Namespace(Rc<Namespace>),
    /// Modul `sprite`: animiertes Sheet-Sprite (Referenz-Typ).
    Sprite(Rc<RefCell<SpriteObj>>),
    /// FILE-Handle (core File-I/O).
    File(Rc<RefCell<GbFile>>),
    /// Modul `tween`: zeitbasierte Interpolation (Referenz-Typ).
    Tween(Rc<RefCell<TweenObj>>),
    /// Modul `json`: geparstes JSON (immutable, read-only).
    Json(Rc<serde_json::Value>),
    /// Modul `save`: Save-Container (Version + geordnete Daten).
    Save(Rc<RefCell<SaveHandle>>),
    /// Modul `astar`: Pathfinding-Grid (Referenz-Typ).
    AStar(Rc<RefCell<crate::astar::AStarGrid>>),
    /// Modul `particles`: Partikel-Emitter (Referenz-Typ).
    Particles(Rc<RefCell<ParticleSys>>),
    /// Modul `ecs`: Entity-Component-System-World (Referenz-Typ).
    Ecs(Rc<RefCell<crate::ecs::World>>),
    /// Coroutine-Handle (YIELD). Haelt den suspendierten Frame; die VM treibt
    /// ihn synchron via CORO_RESUME/SEND (kein Thread -- raylib-Main-Thread-
    /// sicher, deterministisch). Siehe `CoroState`.
    Coroutine(Rc<RefCell<CoroState>>),
}

/// Suspendierter Zustand einer Coroutine. Der Frame (ip/locals/stack/
/// try_handlers) wird beim YIELD hier abgelegt und beim Resume restauriert.
///
/// `fn_ptr` ist ein roher Zeiger auf die `Func` im geladenen Programm. Das ist
/// sound, weil das `Program` die gesamte Laufzeit (Lifetime `'p`) lebt und nach
/// dem Laden immutable ist -- es ueberlebt also alle Coroutinen. So braucht
/// `Value` keinen Lifetime-Parameter.
pub struct CoroState {
    pub fn_ptr: *const Func,
    pub self_obj: Option<Value>,
    pub name: String,
    pub args: Vec<Value>,   // nur bis zum ersten Resume (Parameter-Bindung)
    pub started: bool,
    pub done: bool,
    pub result: Value,
    // Suspendierter Frame:
    pub locals: Vec<Value>,
    pub stack: Vec<Value>,
    pub ip: usize,
    pub try_handlers: Vec<(usize, usize)>,
}

pub struct Particle {
    pub x: f64, pub y: f64, pub vx: f64, pub vy: f64,
    pub lifetime: i32, pub age: i32, pub size: i32, pub color: i64,
}

pub struct ParticleSys {
    pub x: f64, pub y: f64,
    pub vx_min: f64, pub vx_max: f64, pub vy_min: f64, pub vy_max: f64,
    pub lifetime_min: i32, pub lifetime_max: i32,
    pub gravity_x: f64, pub gravity_y: f64,
    pub color: i64,
    pub size_min: i32, pub size_max: i32,
    pub fade: bool,
    pub mode: String,
    pub color_end: i64,
    pub has_color_end: bool,
    pub particles: Vec<Particle>,
}

impl ParticleSys {
    pub fn new(x: f64, y: f64) -> Self {
        ParticleSys {
            x, y,
            vx_min: -50.0, vx_max: 50.0, vy_min: -100.0, vy_max: -50.0,
            lifetime_min: 500, lifetime_max: 1000,
            gravity_x: 0.0, gravity_y: 200.0,
            color: 0xFFFFFF,
            size_min: 2, size_max: 4,
            fade: true,
            mode: "circle".to_string(),
            color_end: 0,
            has_color_end: false,
            particles: Vec::new(),
        }
    }

    pub fn update(&mut self, dt_ms: i32) {
        if self.particles.is_empty() { return; }
        let dt = dt_ms as f64 / 1000.0;
        for p in self.particles.iter_mut() { p.age += dt_ms; }
        self.particles.retain(|p| p.age < p.lifetime);
        let (gx, gy) = (self.gravity_x, self.gravity_y);
        for p in self.particles.iter_mut() {
            if gx != 0.0 { p.vx += gx * dt; }
            if gy != 0.0 { p.vy += gy * dt; }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
    }
}

pub struct SaveHandle {
    pub version: i64,
    pub data: Vec<(String, Value)>, // Einfuege-Reihenfolge (fuer JSON-Write)
}

impl SaveHandle {
    pub fn get(&self, k: &str) -> Option<&Value> { self.data.iter().find(|(key, _)| key == k).map(|(_, v)| v) }
    pub fn set(&mut self, k: String, v: Value) {
        if let Some(e) = self.data.iter_mut().find(|(key, _)| key == &k) { e.1 = v; } else { self.data.push((k, v)); }
    }
    pub fn remove(&mut self, k: &str) { self.data.retain(|(key, _)| key != k); }
}

/// FILE-Handle. `r`-Modus nutzt BufReader (READLINE/ENDOFFILE), `w`/`a` File.
pub enum FileH {
    Read(std::io::BufReader<std::fs::File>),
    Write(std::fs::File),
    Closed,
}

pub struct GbFile {
    pub path: String,
    pub mode: char,
    pub h: FileH,
}

pub struct TweenObj {
    pub start: f64,
    pub end: f64,
    pub duration: i64, // ms
    pub easing: String,
    pub start_ms: i64,
    pub paused_at: Option<i64>,
    pub mode: String, // "once" | "loop" | "pingpong"
}

/// Zustand eines `sprite`-Objekts (entspricht `_Sprite`).
pub struct SpriteObj {
    pub tex_idx: i64,
    pub frame_w: i32,
    pub frame_h: i32,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub anims: HashMap<String, (i32, i32, f64)>, // name -> (first, last, fps)
    pub current_anim: String,
    pub current_frame: i32,
    pub anim_time_ms: i64,
    pub looping: bool,
    pub finished: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub scale_x: f64,
    pub scale_y: f64,
    pub tint: Option<i64>,
}

impl SpriteObj {
    pub fn new(tex_idx: i64, frame_w: i32, frame_h: i32) -> Self {
        SpriteObj {
            tex_idx, frame_w, frame_h,
            x: 0.0, y: 0.0, vx: 0.0, vy: 0.0,
            anims: HashMap::new(),
            current_anim: String::new(),
            current_frame: 0,
            anim_time_ms: 0,
            looping: true,
            finished: false,
            flip_x: false, flip_y: false,
            scale_x: 1.0, scale_y: 1.0,
            tint: None,
        }
    }

    pub fn play(&mut self, name: &str, looping: bool) -> Result<(), String> {
        let &(first, _, _) = self.anims.get(name).ok_or_else(|| {
            let mut keys: Vec<&String> = self.anims.keys().collect();
            keys.sort();
            let avail: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
            format!("SPRITE: unbekannte Animation '{}' (definiert: {})", name,
                if avail.is_empty() { "keine".to_string() } else { avail.join(", ") })
        })?;
        if name == self.current_anim && self.looping == looping && !self.finished {
            return Ok(()); // idempotent
        }
        self.current_anim = name.to_string();
        self.current_frame = first;
        self.anim_time_ms = 0;
        self.looping = looping;
        self.finished = false;
        Ok(())
    }

    pub fn update(&mut self, dt_ms: i64) {
        let dt = dt_ms as f64 / 1000.0;
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        if self.current_anim.is_empty() || self.finished {
            return;
        }
        let (first, last, fps) = self.anims[&self.current_anim];
        if fps <= 0.0 {
            return;
        }
        self.anim_time_ms += dt_ms;
        let frame_duration = 1000.0 / fps;
        let elapsed = (self.anim_time_ms as f64 / frame_duration) as i64;
        let n = (last - first + 1) as i64;
        if self.looping {
            self.current_frame = first + (elapsed.rem_euclid(n)) as i32;
        } else if elapsed >= n {
            self.current_frame = last;
            self.finished = true;
        } else {
            self.current_frame = first + elapsed as i32;
        }
    }
}

pub struct Namespace {
    pub name: String,
    pub members: HashMap<String, Value>, // key = lower-case Member-Name
}

/// Mehrdimensionales, homogen getyptes Array (entspricht `_GBArray`).
pub struct GbArray {
    pub element_type: String,
    pub dims: Vec<i64>,
    pub strides: Vec<i64>,
    pub values: Vec<Value>,
}

impl GbArray {
    pub fn new(element_type: String, dims: Vec<i64>, default: impl Fn() -> Value) -> Self {
        let mut strides = vec![0i64; dims.len()];
        let mut acc = 1i64;
        for k in (0..dims.len()).rev() {
            strides[k] = acc;
            acc *= dims[k];
        }
        let total = if dims.is_empty() { 0 } else { acc };
        let values = (0..total).map(|_| default()).collect();
        GbArray { element_type, dims, strides, values }
    }

    /// Flacher Index mit Bounds-Check (entspricht `_GBArray.flat_index`).
    pub fn flat_index(&self, indices: &[i64]) -> Result<usize, String> {
        if indices.len() != self.dims.len() {
            return Err(format!(
                "Array hat {} Dimension(en), erhalten {} Index/-e",
                self.dims.len(), indices.len()
            ));
        }
        let mut flat = 0i64;
        for (k, &idx) in indices.iter().enumerate() {
            if idx < 0 || idx >= self.dims[k] {
                return Err(format!(
                    "Index {} ausserhalb [0..{}] in Dimension {}",
                    idx, self.dims[k] - 1, k
                ));
            }
            flat += idx * self.strides[k];
        }
        Ok(flat as usize)
    }
}

/// Hash-Map mit STRING-Keys, Insertion-Order erhalten (entspricht `_GBMap`,
/// das auf Python-`dict` aufsetzt). Lineare Liste -- GB-Maps sind klein.
pub struct GbMap {
    pub value_type: String,
    pub entries: Vec<(String, Value)>,
}

impl GbMap {
    pub fn new(value_type: String) -> Self {
        GbMap { value_type, entries: Vec::new() }
    }
    pub fn get(&self, k: &str) -> Option<&Value> {
        self.entries.iter().find(|(key, _)| key == k).map(|(_, v)| v)
    }
    pub fn put(&mut self, k: String, v: Value) {
        if let Some(e) = self.entries.iter_mut().find(|(key, _)| key == &k) {
            e.1 = v;
        } else {
            self.entries.push((k, v));
        }
    }
    pub fn remove(&mut self, k: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|(key, _)| key != k);
        before != self.entries.len()
    }
}

/// Instanz einer User-Klasse/Struct.
pub struct Instance {
    pub class_name: Rc<str>,
    pub fields: HashMap<String, FieldVal>,
}

pub struct FieldVal {
    pub ty: String,
    pub value: Value,
}

impl Value {
    pub fn fmt(&self) -> String {
        match self {
            Value::Nil => "NIL".to_string(),
            Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => fmt_float(*f),
            Value::Str(s) => s.to_string(),
            Value::Tuple(items) => {
                let inner: Vec<String> = items.iter().map(|x| x.fmt()).collect();
                format!("({})", inner.join(", "))
            }
            Value::FuncRef(name) => format!("<FUNCREF {}>", name),
            Value::CompMarker => "<COMP-MARKER>".to_string(),
            Value::Array(a) => {
                let a = a.borrow();
                let shape = a.dims.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(",");
                format!("<ARRAY[{}] OF {}>", shape, a.element_type.to_uppercase())
            }
            Value::Map(m) => {
                let m = m.borrow();
                format!("<MAP[{}] OF {}>", m.entries.len(), m.value_type.to_uppercase())
            }
            Value::Instance(i) => format!("<{}>", i.borrow().class_name),
            Value::Vec2(x, y) => format!("Vec2({}, {})", fmt_float(*x), fmt_float(*y)),
            Value::Namespace(ns) => format!("<NAMESPACE {}>", ns.name),
            Value::Sprite(s) => {
                let s = s.borrow();
                format!("<SPRITE @({:.0},{:.0}) frame={} anim='{}'>", s.x, s.y, s.current_frame, s.current_anim)
            }
            Value::File(f) => format!("<FILE {}>", f.borrow().path),
            Value::Tween(t) => { let t = t.borrow(); format!("<Tween {}->{} {}ms {} {}>", t.start, t.end, t.duration, t.easing, t.mode) }
            Value::Json(j) => {
                let s = serde_json::to_string(j.as_ref()).unwrap_or_default();
                let short = if s.len() <= 40 { s } else { format!("{}...", &s[..37]) };
                format!("<JSON {}>", short)
            }
            Value::Save(s) => { let s = s.borrow(); format!("<Save v{} keys={}>", s.version, s.data.len()) }
            Value::AStar(g) => { let g = g.borrow(); format!("<AStar {}x{}>", g.w, g.h) }
            Value::Particles(p) => format!("<ParticleSystem {} particles>", p.borrow().particles.len()),
            Value::Ecs(w) => format!("<ECS_WORLD entities={}>", w.borrow().count()),
            Value::Coroutine(c) => format!("<COROUTINE {}>", c.borrow().name),
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Nil => false,
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "NIL",
            Value::Bool(_) => "BOOLEAN",
            Value::Int(_) => "INTEGER",
            Value::Float(_) => "FLOAT",
            Value::Str(_) => "STRING",
            Value::Tuple(_) => "TUPLE",
            Value::FuncRef(_) => "FUNCREF",
            Value::CompMarker => "COMP_MARKER",
            Value::Array(_) => "ARRAY",
            Value::Map(_) => "MAP",
            Value::Instance(_) => "OBJECT",
            Value::Vec2(_, _) => "VEC2",
            Value::Namespace(_) => "NAMESPACE",
            Value::Sprite(_) => "SPRITE",
            Value::File(_) => "FILE",
            Value::Tween(_) => "TWEEN",
            Value::Json(_) => "JSON_HANDLE",
            Value::Save(_) => "SAVE_HANDLE",
            Value::AStar(_) => "ASTAR_GRID",
            Value::Particles(_) => "PARTICLE_SYSTEM",
            Value::Ecs(_) => "ECS_WORLD",
            Value::Coroutine(_) => "COROUTINE",
        }
    }

    pub fn str_rc(s: &str) -> Value {
        Value::Str(Rc::from(s))
    }
}

pub fn is_num(v: &Value) -> bool {
    matches!(v, Value::Int(_) | Value::Float(_))
}

pub fn as_f64(v: &Value) -> f64 {
    match v {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => f64::NAN,
    }
}

/// Python `==`: cross-numeric (1 == 1.0), sonst typgleich; Tupel elementweise.
pub fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Tuple(x), Value::Tuple(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| value_eq(p, q))
        }
        (Value::Vec2(ax, ay), Value::Vec2(bx, by)) => ax == bx && ay == by,
        _ if is_num(a) && is_num(b) => as_f64(a) == as_f64(b),
        _ => false,
    }
}

/// Float-Formatierung bit-identisch zu Python `repr(float)`.
///
/// Python (CPython `format_float_short`, Modus 'r') nutzt die kuerzeste
/// Round-Trip-Darstellung und wechselt zu E-Notation, wenn der Dezimalpunkt
/// `decpt <= -4` oder `decpt > 16` liegt (`decpt` = Position des Dezimalpunkts
/// relativ zur ersten signifikanten Ziffer, d.h. `exp + 1`). Sonst Fixpunkt;
/// ganzzahlige Werte bekommen ein `.0`.
///
/// Rusts `{}` liefert die kuerzesten Ziffern (immer Fixpunkt), `{:e}` die
/// kuerzeste Mantisse + Exponent -- daraus rekonstruieren wir Pythons Regel.
fn fmt_float(f: f64) -> String {
    if f.is_nan() {
        return "nan".to_string();
    }
    if f.is_infinite() {
        return if f < 0.0 { "-inf" } else { "inf" }.to_string();
    }
    // Ganzzahlige Floats (inkl. 0.0/-0.0): Python `_fmt` nutzt hier `f"{v:.1f}"`
    // -- IMMER Fixpunkt, auch fuer grosse Betraege (1e16 -> "10000000000000000.0").
    // Nur nicht-ganzzahlige Werte gehen durch `repr` (mit E-Notation-Schwelle).
    if f.fract() == 0.0 {
        return format!("{:.1}", f);
    }
    let neg = f < 0.0;
    let a = f.abs();
    // {:e} -> "m e EXP" mit kuerzester Mantisse m und Dezimalexponent EXP.
    let es = format!("{:e}", a);
    let (mant, exp_s) = es.split_once('e').unwrap();
    let exp: i32 = exp_s.parse().unwrap();
    let decpt = exp + 1;
    let body = if decpt <= -4 || decpt > 16 {
        // E-Notation, Python-Stil: Vorzeichen + min. 2-stelliger Exponent.
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{}e{}{:02}", mant, sign, exp.abs())
    } else {
        // Fixpunkt: kuerzeste Rust-Display-Form, ganzzahlig -> ".0".
        let mut s = format!("{}", a);
        if !s.contains('.') {
            s.push_str(".0");
        }
        s
    };
    if neg {
        format!("-{}", body)
    } else {
        body
    }
}
