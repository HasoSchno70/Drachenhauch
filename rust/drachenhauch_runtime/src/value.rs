//! Drachenhauch-Laufzeitwerte fuer die Rust-VM.
//!
//! Skalare (Int/Float/Str/Bool/Tuple) sind Wert-Typen (immutable). Arrays,
//! Maps und Instanzen sind Referenz-Typen (`Rc<RefCell<…>>`) -- Zuweisung
//! aliased, genau wie in der Python-VM (dort dasselbe Objekt im Slot).
//!
//! `fmt`/`truthy` muessen bit-identisch zu `drachenhauch/vm.py` sein.

use std::cell::RefCell;
use std::collections::HashMap;
use rustc_hash::FxHashMap;
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
    /// Eine an eine Instanz GEBUNDENE Methode (`f = spieler.tick`).
    ///
    /// Nach aussen ist das ein FUNCREF: `type_name()` sagt "FUNCREF", damit
    /// `DIM f AS FUNCREF` sie aufnimmt und jede Stelle, die einen Rueckruf
    /// erwartet (GUI_ON_*, TIMER_*, SORT), sie ohne Sonderfall annimmt. Der
    /// Unterschied liegt nur darin, dass beim Aufruf ein Empfaenger mitkommt.
    ///
    /// Der Methodenname liegt kleingeschrieben vor -- so legt der Compiler
    /// die Methoden-Schluessel ab (`resolve_method` sucht damit).
    BoundMethod(Rc<(Value, Rc<str>)>),
    CompMarker,
    Array(Rc<RefCell<GbArray>>),
    Map(Rc<RefCell<GbMap>>),
    Instance(Rc<RefCell<Instance>>),
    /// Modul `vec2`: immutabler 2D-Vektor (Wert-Semantik wie ein Skalar).
    Vec2(f64, f64),
    /// Modul `geld`: ein Betrag in Hundertstel-Cent (vier Nachkommastellen).
    /// Warum ein eigener Wert und nicht einfach INTEGER: so kann die Sprache
    /// `betrag + 1.0` ablehnen, statt es still zu rechnen.
    Geld(i64),
    /// Modul `m3d`: immutable 3D-Mathe-Typen. f32 hält `Value` kompakt
    /// (Vec4/Quat = 16 B wie Vec2) und ist render-nativ; GB-FLOAT-Getter casten
    /// zu f64. MAT4 ist geboxt (16 floats wären zu groß inline), column-major
    /// (raylib/OpenGL) -> direkte Konversion zu raylib::ffi::Matrix.
    Vec3(f32, f32, f32),
    Vec4(f32, f32, f32, f32),
    Quat(f32, f32, f32, f32),
    Mat4(Rc<[f32; 16]>),
    /// ENUM- / STATIC-CONST-Namespace (`State.PLAYING`, `Player.MAX_HP`).
    /// Liegt als CONST-Wert im Programm; MemberAccess loest Member auf.
    Namespace(Rc<Namespace>),
    /// Modul `sprite`: animiertes Sheet-Sprite (Referenz-Typ).
    Sprite(Rc<RefCell<SpriteObj>>),
    /// FILE-Handle (core File-I/O).
    File(Rc<RefCell<GbFile>>),
    /// BUFFER: veraenderliche Bytefolge (WP B). Referenz-Typ wie ARRAY --
    /// uebergibt man ihn an eine SUB, teilen sich beide Seiten die Bytes.
    /// Bewusst KEIN STRING: der ist UTF-8 und kann gar nicht jede Bytefolge
    /// tragen; `LEN` zaehlte dort ausserdem Zeichen, nicht Bytes.
    Buffer(Rc<RefCell<Vec<u8>>>),
    /// Modul `tween`: zeitbasierte Interpolation (Referenz-Typ).
    Tween(Rc<RefCell<TweenObj>>),
    /// Modul `json`: geparstes JSON. Veraenderbar (`JSON_SET_*`/`JSON_APPEND_*`)
    /// und darum ein REFERENZ-Typ wie MAP, ARRAY und BUFFER: `b = a` legt
    /// keine Kopie an, beide Namen zeigen auf dasselbe Dokument. Bis 2026-08
    /// gab es nur Leser, deshalb stand hier "immutable, read-only" -- das war
    /// eine Folge des fehlenden Schreibwegs, keine Entscheidung ueber die
    /// Wertsemantik.
    Json(Rc<RefCell<serde_json::Value>>),
    /// Modul `xml`: ein Knoten des geparsten Baums. NUR LESEND, deshalb
    /// ohne RefCell -- ein `XML_FIND` gibt einen Teilbaum weiter, indem
    /// es den Rc klont; kopiert wird dabei nichts.
    Xml(Rc<crate::xml::Knoten>),
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
    /// Modul `tiled`: geladene Tiled-Map (Referenz-Typ, Bulk-Ops mutieren).
    Tiled(Rc<RefCell<crate::tiled::TiledMap>>),
    /// Modul `controller`: Character-Controller (Platformer-Physik).
    CharController(Rc<RefCell<crate::controller::CharController>>),
    /// Modul `physics`: Broadphase-Kollision (PHYSICS_BROAD_*, Referenz-Typ).
    PhysicsBroad(Rc<RefCell<crate::physics::BroadPhase>>),
    /// Modul `physics3d`: Rapier3D-Starrkoerper-Welt (PHYS3D_*, Referenz-Typ).
    Phys3d(Rc<RefCell<crate::physics3d::Phys3dWorld>>),
    /// Modul `physics2d`: Rapier2D-Starrkoerper-Welt (PHYS2D_*, Referenz-Typ).
    Phys2d(Rc<RefCell<crate::physics2d::Phys2dWorld>>),
    /// Modul `animfsm`: Animations-State-Machine (ANIM_FSM_*, Referenz-Typ).
    AnimFsm(Rc<RefCell<crate::animfsm::AnimFsmObj>>),
    /// Modul `chart`: Diagramm (CHART_*, Referenz-Typ -- Daten + Stil).
    Chart(Rc<RefCell<crate::chart::ChartObj>>),
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
    // Review-Fund: ohne dieses Flag konnte eine Coroutine sich selbst (direkt
    // oder ueber eine Kette anderer Coroutinen) re-entrant per CORO_RESUME
    // wieder aufrufen, waehrend ihr Frame bereits "unterwegs" war (locals/
    // stack per mem::take entleert) -- die verschachtelte Ausfuehrung sah dann
    // einen leeren `locals`-Vec und paniked mit einem Index-Out-of-Bounds statt
    // einen fangbaren Fehler zu werfen.
    pub running: bool,
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
    /// Ein Lesestrom, der KEINE Datei ist -- heute die Standardeingabe
    /// (`STDIN()`). Eigene Variante statt `Read`, weil hier `Seek` fehlt:
    /// eine Pipe laesst sich nicht zurueckspulen, und `SEEK`/`TELL` sollen
    /// das auch sagen statt es zu versuchen.
    Strom(Box<dyn std::io::BufRead>),
    Closed,
}

impl GbFile {
    /// Der Lesestrom dieses Handles -- egal ob Datei oder Standardeingabe.
    ///
    /// Ohne diesen Helfer braeuchte jeder Leser (READLINE, READALL$,
    /// ENDOFFILE, READ_BYTES) zwei fast gleiche Zweige, und der naechste
    /// Stromtyp braechte einen dritten. `None` heisst: nicht zum Lesen offen.
    pub fn leser(&mut self) -> Option<&mut dyn std::io::BufRead> {
        match &mut self.h {
            FileH::Read(r) => Some(r),
            FileH::Strom(r) => Some(r.as_mut()),
            _ => None,
        }
    }
}

pub struct GbFile {
    pub path: String,
    pub h: FileH,
    /// Kodierung dieser Datei (aus `OPENFILE(pfad, modus, kodierung)`; ohne
    /// Angabe UTF-8). Gilt fuer READLINE/READALL$/WRITE/WRITELINE.
    pub kod: crate::kodierung::Kodierung,
    /// Wurde aus dieser Datei schon gelesen? Nur beim ALLERERSTEN Lesen darf
    /// ein BOM wegfallen -- taeten wir es bei jeder Zeile, verschwaende
    /// mitten im Text ein echtes (wenn auch seltenes) Zeichen.
    pub am_anfang: bool,
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
    pub members: FxHashMap<String, Value>, // key = lower-case Member-Name
}

/// Element-Backing eines GbArray. ARRAY OF INTEGER/FLOAT speichern rohe
/// i64/f64 (dicht, cache-freundlich, kein Enum-Tag pro Element) -- alle
/// anderen Element-Typen generische Values. Entspricht dem
/// array.array('q'/'d')-Backing der frueheren Python-Referenz (daher gilt
/// weiter das dokumentierte 64-bit-Limit fuer INTEGER-Arrays).
///
/// `set`/`push`/`insert` erwarten bereits auf den Element-Typ gecoercte
/// Werte; bei einem (eigentlich unmoeglichen) Backing-Mismatch wird das
/// Backing defensiv zu `Val` promotet statt falsch zu speichern.
#[derive(Clone)]
pub enum Cells {
    Int(Vec<i64>),
    Float(Vec<f64>),
    Val(Vec<Value>),
}

impl Cells {
    pub fn len(&self) -> usize {
        match self { Cells::Int(v) => v.len(), Cells::Float(v) => v.len(), Cells::Val(v) => v.len() }
    }
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Element als Value (Index muss gueltig sein -- wie frueher `values[i]`).
    pub fn get(&self, i: usize) -> Value {
        match self {
            Cells::Int(v) => Value::Int(v[i]),
            Cells::Float(v) => Value::Float(v[i]),
            Cells::Val(v) => v[i].clone(),
        }
    }

    /// Backing zu Val(Vec<Value>) umwandeln (defensiver Mismatch-Ausweg).
    fn promote(&mut self) {
        let vals: Vec<Value> = match self {
            Cells::Int(v) => v.iter().map(|&x| Value::Int(x)).collect(),
            Cells::Float(v) => v.iter().map(|&x| Value::Float(x)).collect(),
            Cells::Val(_) => return,
        };
        *self = Cells::Val(vals);
    }

    pub fn set(&mut self, i: usize, v: Value) {
        match (&mut *self, v) {
            (Cells::Int(vec), Value::Int(x)) => vec[i] = x,
            (Cells::Int(vec), Value::Float(x)) if x.fract() == 0.0 => vec[i] = x as i64,
            (Cells::Float(vec), Value::Float(x)) => vec[i] = x,
            (Cells::Float(vec), Value::Int(x)) => vec[i] = x as f64,
            (Cells::Val(vec), v) => vec[i] = v,
            (_, v) => { self.promote(); self.set(i, v); }
        }
    }

    pub fn push(&mut self, v: Value) {
        match (&mut *self, v) {
            (Cells::Int(vec), Value::Int(x)) => vec.push(x),
            (Cells::Int(vec), Value::Float(x)) if x.fract() == 0.0 => vec.push(x as i64),
            (Cells::Float(vec), Value::Float(x)) => vec.push(x),
            (Cells::Float(vec), Value::Int(x)) => vec.push(x as f64),
            (Cells::Val(vec), v) => vec.push(v),
            (_, v) => { self.promote(); self.push(v); }
        }
    }

    pub fn pop(&mut self) -> Option<Value> {
        match self {
            Cells::Int(v) => v.pop().map(Value::Int),
            Cells::Float(v) => v.pop().map(Value::Float),
            Cells::Val(v) => v.pop(),
        }
    }

    pub fn insert(&mut self, i: usize, v: Value) {
        match (&mut *self, v) {
            (Cells::Int(vec), Value::Int(x)) => vec.insert(i, x),
            (Cells::Int(vec), Value::Float(x)) if x.fract() == 0.0 => vec.insert(i, x as i64),
            (Cells::Float(vec), Value::Float(x)) => vec.insert(i, x),
            (Cells::Float(vec), Value::Int(x)) => vec.insert(i, x as f64),
            (Cells::Val(vec), v) => vec.insert(i, v),
            (_, v) => { self.promote(); self.insert(i, v); }
        }
    }

    pub fn remove(&mut self, i: usize) -> Value {
        match self {
            Cells::Int(v) => Value::Int(v.remove(i)),
            Cells::Float(v) => Value::Float(v.remove(i)),
            Cells::Val(v) => v.remove(i),
        }
    }

    pub fn truncate(&mut self, n: usize) {
        match self { Cells::Int(v) => v.truncate(n), Cells::Float(v) => v.truncate(n), Cells::Val(v) => v.truncate(n) }
    }
    pub fn swap(&mut self, i: usize, j: usize) {
        match self { Cells::Int(v) => v.swap(i, j), Cells::Float(v) => v.swap(i, j), Cells::Val(v) => v.swap(i, j) }
    }
    pub fn reverse(&mut self) {
        match self { Cells::Int(v) => v.reverse(), Cells::Float(v) => v.reverse(), Cells::Val(v) => v.reverse() }
    }

    /// Teil-Kopie [a..b) als neues Backing (Slicing).
    pub fn slice(&self, a: usize, b: usize) -> Cells {
        match self {
            Cells::Int(v) => Cells::Int(v[a..b].to_vec()),
            Cells::Float(v) => Cells::Float(v[a..b].to_vec()),
            Cells::Val(v) => Cells::Val(v[a..b].to_vec()),
        }
    }

    /// Alle Elemente als Vec<Value> (fuer Tupel-Konversionen etc.).
    pub fn to_values(&self) -> Vec<Value> {
        match self {
            Cells::Int(v) => v.iter().map(|&x| Value::Int(x)).collect(),
            Cells::Float(v) => v.iter().map(|&x| Value::Float(x)).collect(),
            Cells::Val(v) => v.clone(),
        }
    }

    /// Iterator ueber Elemente als (konstruierte) Values.
    pub fn iter(&self) -> CellsIter<'_> { CellsIter { cells: self, i: 0 } }

    // Typisierte Direkt-Zugriffe fuer Bulk-Fast-Paths (weitere Varianten
    // bei Bedarf ergaenzen -- Konsumenten matchen sonst direkt auf Cells).
    pub fn as_ints(&self) -> Option<&[i64]> { if let Cells::Int(v) = self { Some(v) } else { None } }
    pub fn as_vals(&self) -> Option<&[Value]> { if let Cells::Val(v) = self { Some(v) } else { None } }
}

pub struct CellsIter<'a> { cells: &'a Cells, i: usize }
impl<'a> Iterator for CellsIter<'a> {
    type Item = Value;
    fn next(&mut self) -> Option<Value> {
        if self.i >= self.cells.len() { return None; }
        let v = self.cells.get(self.i);
        self.i += 1;
        Some(v)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rest = self.cells.len() - self.i;
        (rest, Some(rest))
    }
}

/// Mehrdimensionales, homogen getyptes Array (entspricht `_GBArray`).
pub struct GbArray {
    pub element_type: String,
    pub dims: Vec<i64>,
    pub strides: Vec<i64>,
    pub cells: Cells,
}

impl GbArray {
    pub fn new(element_type: String, dims: Vec<i64>, default: impl Fn() -> Value) -> Self {
        let mut strides = vec![0i64; dims.len()];
        let mut acc = 1i64;
        for k in (0..dims.len()).rev() {
            strides[k] = acc;
            acc *= dims[k];
        }
        let total = if dims.is_empty() { 0 } else { acc as usize };
        let cells = match element_type.as_str() {
            "integer" => Cells::Int(vec![0i64; total]),
            "float" => Cells::Float(vec![0.0f64; total]),
            _ => Cells::Val((0..total).map(|_| default()).collect()),
        };
        GbArray { element_type, dims, strides, cells }
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

/// Map mit STRING-Keys und erhaltener Einfuege-Reihenfolge.
///
/// Zwei Datenstrukturen, die zusammengehalten werden muessen:
///   - `eintraege` haelt die Reihenfolge (MAPKEYS/MAPVALUES/MAPITEMS und die
///     JSON-Ausgabe verlassen sich darauf),
///   - `index` bildet Key -> Position ab, damit Nachschlagen nicht linear ist.
///
/// GEMESSEN, warum das noetig war: mit blosser linearer Suche kostete eine
/// Map mit 5 000 Eintraegen 16 ms zum Fuellen, mit 10 000 schon 75 ms und mit
/// 20 000 dann 224 ms -- sauber quadratisch. Der Kommentar davor sagte "GB-Maps
/// sind klein"; das stimmt fuer Spielstaende, aber nicht fuer eine Sprache, mit
/// der man auch Daten verarbeiten koennen soll.
///
/// `eintraege` ist ABSICHTLICH privat. Vorher war es `pub`, und `MAPCLEAR`
/// griff direkt darauf zu -- mit einem Index daneben waere genau das die
/// Stelle, an der beide still auseinanderlaufen.
pub struct GbMap {
    pub value_type: String,
    eintraege: Vec<(String, Value)>,
    index: HashMap<String, usize>,
    /// Elementart, wenn die Map als MENGE benutzt wird (`SET_*`): `'i'` oder
    /// `'s'`, gesetzt von der ersten Aufnahme.
    ///
    /// Schluessel sind immer Zeichenketten -- ohne diese Merkung fielen
    /// `SET_ADD(m, 5)` und `SET_ADD(m, "5")` auf denselben Eintrag, und die
    /// Menge haette danach ein Element statt zwei. Statt das hinzunehmen legt
    /// die erste Aufnahme die Art fest; jede spaetere Abweichung meldet einen
    /// Fehler. Nebenbei weiss `SET_ITEMS` dadurch, ob es Zahlen oder Texte
    /// zurueckgeben muss.
    set_art: Option<char>,
}

impl GbMap {
    pub fn new(value_type: String) -> Self {
        GbMap { value_type, eintraege: Vec::new(), index: HashMap::new(),
                set_art: None }
    }

    /// Elementart der Menge (None = noch leer bzw. nie als Menge benutzt).
    pub fn set_art(&self) -> Option<char> { self.set_art }

    /// Elementart festlegen oder pruefen. Fehler, wenn sie schon anders ist.
    pub fn set_art_pruefen(&mut self, art: char, fn_: &str) -> Result<(), String> {
        match self.set_art {
            None => { self.set_art = Some(art); Ok(()) }
            Some(a) if a == art => Ok(()),
            Some(a) => Err(format!(
                "{}: diese Menge enthaelt {}, hier kam {}. Eine Menge fuehrt \
                 eine Elementart -- die erste Aufnahme legt sie fest.",
                fn_, if a == 'i' { "Zahlen" } else { "Texte" },
                if art == 'i' { "eine Zahl" } else { "ein Text" })),
        }
    }

    /// Eintraege in Einfuege-Reihenfolge, nur lesend.
    pub fn entries(&self) -> &[(String, Value)] { &self.eintraege }

    pub fn len(&self) -> usize { self.eintraege.len() }
    pub fn is_empty(&self) -> bool { self.eintraege.is_empty() }

    pub fn get(&self, k: &str) -> Option<&Value> {
        self.index.get(k).map(|i| &self.eintraege[*i].1)
    }

    pub fn put(&mut self, k: String, v: Value) {
        match self.index.get(&k) {
            Some(i) => self.eintraege[*i].1 = v,
            None => {
                self.index.insert(k.clone(), self.eintraege.len());
                self.eintraege.push((k, v));
            }
        }
    }

    /// Loeschen ist O(n): die Positionen aller nachfolgenden Eintraege
    /// verschieben sich, der Index wird also neu aufgebaut. Bewusst so --
    /// Nachschlagen und Einfuegen sind der haeufige Fall, Loeschen nicht, und
    /// Grabsteine wuerden die Reihenfolge-Zusage verkomplizieren.
    pub fn remove(&mut self, k: &str) -> bool {
        match self.index.remove(k) {
            None => false,
            Some(pos) => {
                self.eintraege.remove(pos);
                for (_, i) in self.index.iter_mut() {
                    if *i > pos { *i -= 1; }
                }
                true
            }
        }
    }

    pub fn clear(&mut self) {
        self.eintraege.clear();
        self.index.clear();
        // Mit dem letzten Element geht auch die Elementart -- sonst koennte
        // eine geleerte Menge nie die Art wechseln.
        self.set_art = None;
    }
}

/// Instanz einer User-Klasse/Struct.
pub struct Instance {
    pub class_name: Rc<str>,
    pub fields: FxHashMap<String, FieldVal>,
}

pub struct FieldVal {
    pub ty: String,
    pub value: Value,
}

/// Ein gespeicherter Rueckruf, wie ihn `gui` und `timer` fuehren.
///
/// Warum nicht einfach ein `Value`: `gui.rs` schreibt den Handler-NAMEN in
/// die `.dhform`-Datei (der Form-Designer liest ihn dort wieder). Der Name
/// muss also erhalten bleiben, auch wenn zusaetzlich ein Empfaenger
/// mitgefuehrt wird. Darum beides nebeneinander statt einem Wert, aus dem
/// man den Namen erst wieder herauspulen muesste.
///
/// `empfaenger = None` ist eine freie Funktion, `Some(instanz)` eine
/// gebundene Methode.
#[derive(Clone)]
pub struct Rueckruf {
    pub name: Rc<str>,
    pub empfaenger: Option<Value>,
}

impl Rueckruf {
    /// Aus dem Wert bauen, den ein Builtin als Rueckruf-Argument bekommen hat.
    /// `None`, wenn es weder FUNCREF noch gebundene Methode ist.
    pub fn aus_wert(v: &Value) -> Option<Rueckruf> {
        match v {
            Value::FuncRef(n) => Some(Rueckruf { name: n.clone(), empfaenger: None }),
            Value::BoundMethod(b) =>
                Some(Rueckruf { name: b.1.clone(), empfaenger: Some(b.0.clone()) }),
            _ => None,
        }
    }

    /// Nur fuer das Wiederherstellen aus einer Datei (dort steht bloss ein Name).
    pub fn benannt(name: &str) -> Rueckruf {
        Rueckruf { name: Rc::from(name), empfaenger: None }
    }

    /// Ist der Rueckruf an eine Instanz gebunden? Solche lassen sich nicht in
    /// eine `.dhform` schreiben -- die Instanz gibt es beim Laden nicht.
    pub fn ist_gebunden(&self) -> bool {
        self.empfaenger.is_some()
    }
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
            Value::BoundMethod(b) => match &b.0 {
                Value::Instance(i) => format!("<FUNCREF {}.{}>", i.borrow().class_name, b.1),
                other => format!("<FUNCREF {}.{}>", other.type_name(), b.1),
            },
            Value::CompMarker => "<COMP-MARKER>".to_string(),
            Value::Array(a) => {
                let a = a.borrow();
                let shape = a.dims.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(",");
                format!("<ARRAY[{}] OF {}>", shape, a.element_type.to_uppercase())
            }
            Value::Map(m) => {
                let m = m.borrow();
                format!("<MAP[{}] OF {}>", m.len(), m.value_type.to_uppercase())
            }
            Value::Instance(i) => format!("<{}>", i.borrow().class_name),
            Value::Vec2(x, y) => format!("Vec2({}, {})", fmt_float(*x), fmt_float(*y)),
            Value::Geld(w) => crate::geld::anzeige(*w),
            Value::Vec3(x, y, z) =>
                format!("Vec3({}, {}, {})", fmt_float(*x as f64), fmt_float(*y as f64), fmt_float(*z as f64)),
            Value::Vec4(x, y, z, w) =>
                format!("Vec4({}, {}, {}, {})", fmt_float(*x as f64), fmt_float(*y as f64), fmt_float(*z as f64), fmt_float(*w as f64)),
            Value::Quat(x, y, z, w) =>
                format!("Quat({}, {}, {}, {})", fmt_float(*x as f64), fmt_float(*y as f64), fmt_float(*z as f64), fmt_float(*w as f64)),
            Value::Mat4(m) => {
                let parts: Vec<String> = m.iter().map(|v| fmt_float(*v as f64)).collect();
                format!("Mat4[{}]", parts.join(", "))
            }
            Value::Namespace(ns) => format!("<NAMESPACE {}>", ns.name),
            Value::Sprite(s) => {
                let s = s.borrow();
                format!("<SPRITE @({:.0},{:.0}) frame={} anim='{}'>", s.x, s.y, s.current_frame, s.current_anim)
            }
            Value::File(f) => format!("<FILE {}>", f.borrow().path),
            // Bewusst nur die Laenge, nicht der Inhalt: ein `PRINT puffer`
            // duerfte sonst megabyteweise Bytes in die Konsole kippen. Wer die
            // Bytes sehen will, nimmt BUFFER_TO_HEX$.
            Value::Buffer(b) => format!("<BUFFER {} Bytes>", b.borrow().len()),
            Value::Tween(t) => { let t = t.borrow(); format!("<Tween {}->{} {}ms {} {}>", t.start, t.end, t.duration, t.easing, t.mode) }
            // Wie bei BUFFER und JSON: eine kurze Kennzeichnung, nicht
            // der ganze Baum -- `PRINT knoten` soll eine Zeile bleiben.
            Value::Xml(k) => format!("<{} mit {} Kind(ern)>", k.name, k.anzahl_kinder()),
            Value::Json(j) => {
                let s = serde_json::to_string(&*j.borrow()).unwrap_or_default();
                // Review-Fund: `&s[..37]` ist ein BYTE-Slice -- bei einem
                // Mehrbyte-Zeichen (Umlaute etc.) genau an Position 37 paniked
                // das mit "byte index 37 is not a char boundary" (reproduzierbar
                // allein durch `PRINT JSON_PARSE(...)` mit passendem Inhalt).
                // char-basiertes take() ist immer eine gueltige Grenze.
                let short = if s.chars().count() <= 40 { s }
                    else { format!("{}...", s.chars().take(37).collect::<String>()) };
                format!("<JSON {}>", short)
            }
            Value::Save(s) => { let s = s.borrow(); format!("<Save v{} keys={}>", s.version, s.data.len()) }
            Value::AStar(g) => { let g = g.borrow(); format!("<AStar {}x{}>", g.w, g.h) }
            Value::Particles(p) => format!("<ParticleSystem {} particles>", p.borrow().particles.len()),
            Value::Chart(c) => { let c = c.borrow(); format!("<CHART {} {} Reihen, {} Punkte>", c.kind.name(), c.series.len(), c.labels.len()) }
            Value::Ecs(w) => format!("<ECS_WORLD entities={}>", w.borrow().count()),
            Value::Coroutine(c) => format!("<COROUTINE {}>", c.borrow().name),
            Value::Tiled(m) => {
                let m = m.borrow();
                format!("<TILED_MAP {}x{} tiles, {} layers>", m.width, m.height, m.layers.len())
            }
            Value::CharController(c) => {
                let c = c.borrow();
                format!(
                    "<CHAR_CONTROLLER pos=({:.1}, {:.1}) vel=({:.1}, {:.1}) ground={}>",
                    c.x, c.y, c.vx, c.vy, if c.on_ground { "True" } else { "False" }
                )
            }
            Value::PhysicsBroad(b) => {
                let b = b.borrow();
                format!("<BroadPhase {} entities, {} pairs>", b.count(), b.pair_count())
            }
            Value::Phys3d(w) => format!("<PHYS_WORLD {} bodies>", w.borrow().count()),
            Value::Phys2d(w) => format!("<PHYS2D_WORLD {} bodies>", w.borrow().count()),
            Value::AnimFsm(f) => {
                let f = f.borrow();
                format!("<ANIM_FSM state='{}' ({} states)>", f.current, f.states.len())
            }
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
            // Bewusst ebenfalls "FUNCREF": eine gebundene Methode ist fuer den
            // Aufrufer dasselbe Ding wie eine freie Funktion.
            Value::BoundMethod(_) => "FUNCREF",
            Value::CompMarker => "COMP_MARKER",
            Value::Array(_) => "ARRAY",
            Value::Map(_) => "MAP",
            Value::Instance(_) => "OBJECT",
            Value::Vec2(_, _) => "VEC2",
            Value::Geld(_) => "GELD",
            Value::Vec3(..) => "VEC3",
            Value::Vec4(..) => "VEC4",
            Value::Quat(..) => "QUAT",
            Value::Mat4(_) => "MAT4",
            Value::Namespace(_) => "NAMESPACE",
            Value::Sprite(_) => "SPRITE",
            Value::File(_) => "FILE",
            Value::Buffer(_) => "BUFFER",
            Value::Tween(_) => "TWEEN",
            Value::Json(_) => "JSON_HANDLE",
            Value::Xml(_) => "XML_HANDLE",
            Value::Save(_) => "SAVE_HANDLE",
            Value::AStar(_) => "ASTAR_GRID",
            Value::Particles(_) => "PARTICLE_SYSTEM",
            Value::Ecs(_) => "ECS_WORLD",
            Value::Coroutine(_) => "COROUTINE",
            Value::Tiled(_) => "TILED_MAP",
            Value::CharController(_) => "CHAR_CONTROLLER",
            Value::PhysicsBroad(_) => "PHYSICS_BROAD",
            Value::Phys3d(_) => "PHYS_WORLD",
            Value::Phys2d(_) => "PHYS2D_WORLD",
            Value::AnimFsm(_) => "ANIM_FSM",
            Value::Chart(_) => "CHART",
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
        (Value::Geld(x), Value::Geld(y)) => x == y,
        (Value::Vec3(ax, ay, az), Value::Vec3(bx, by, bz)) => ax == bx && ay == by && az == bz,
        (Value::Vec4(ax, ay, az, aw), Value::Vec4(bx, by, bz, bw)) =>
            ax == bx && ay == by && az == bz && aw == bw,
        (Value::Quat(ax, ay, az, aw), Value::Quat(bx, by, bz, bw)) =>
            ax == bx && ay == by && az == bz && aw == bw,
        (Value::Mat4(x), Value::Mat4(y)) => x.iter().zip(y.iter()).all(|(p, q)| p == q),
        // Referenz-Typen (ARRAY/MAP/Instanz/Tiled/...) sind ueber Rc aliasbar
        // (`b = a` teilt dasselbe Objekt) -> Gleichheit = Identitaet. Damit ist
        // u.a. `a = a` TRUE (vorher fielen sie auf `=> false`).
        // Review-Fund: nur 4 von ~18 Referenz-Typen hatten einen ptr_eq-Arm --
        // Sprite/Tween/File/Save/AStar/Particles/Ecs/Coroutine/
        // CharController/PhysicsBroad/Phys3d/Phys2d/AnimFsm/Namespace/Json
        // fielen alle auf `_ => false` durch, `IF spr = spr THEN` war also
        // FALSE und ARRAY_INDEXOF(handles, h) konnte ein Handle nie finden.
        (Value::Array(x), Value::Array(y)) => Rc::ptr_eq(x, y),
        (Value::Map(x), Value::Map(y)) => Rc::ptr_eq(x, y),
        (Value::Instance(x), Value::Instance(y)) => Rc::ptr_eq(x, y),
        (Value::Tiled(x), Value::Tiled(y)) => Rc::ptr_eq(x, y),
        (Value::Sprite(x), Value::Sprite(y)) => Rc::ptr_eq(x, y),
        (Value::File(x), Value::File(y)) => Rc::ptr_eq(x, y),
        (Value::Tween(x), Value::Tween(y)) => Rc::ptr_eq(x, y),
        (Value::Json(x), Value::Json(y)) => Rc::ptr_eq(x, y),
        (Value::Xml(x), Value::Xml(y)) => Rc::ptr_eq(x, y),
        (Value::Save(x), Value::Save(y)) => Rc::ptr_eq(x, y),
        (Value::AStar(x), Value::AStar(y)) => Rc::ptr_eq(x, y),
        (Value::Particles(x), Value::Particles(y)) => Rc::ptr_eq(x, y),
        (Value::Ecs(x), Value::Ecs(y)) => Rc::ptr_eq(x, y),
        (Value::Coroutine(x), Value::Coroutine(y)) => Rc::ptr_eq(x, y),
        (Value::CharController(x), Value::CharController(y)) => Rc::ptr_eq(x, y),
        (Value::PhysicsBroad(x), Value::PhysicsBroad(y)) => Rc::ptr_eq(x, y),
        (Value::Phys3d(x), Value::Phys3d(y)) => Rc::ptr_eq(x, y),
        (Value::Phys2d(x), Value::Phys2d(y)) => Rc::ptr_eq(x, y),
        (Value::AnimFsm(x), Value::AnimFsm(y)) => Rc::ptr_eq(x, y),
        (Value::Namespace(x), Value::Namespace(y)) => Rc::ptr_eq(x, y),
        // FUNCREF ist ein WERT-Typ (Funktionsname), keine mutierbare Referenz
        // -- `f = @foo` muss unabhaengig davon TRUE sein, ob beide FuncRefs
        // aus demselben LOAD_FUNCREF stammen. `Rc<str> == Rc<str>` vergleicht
        // ueber Deref bereits den STRING-Inhalt (nicht die Pointer-Adresse).
        (Value::FuncRef(x), Value::FuncRef(y)) => x == y,
        // Gebundene Methoden sind gleich, wenn Empfaenger UND Methode gleich
        // sind. Der Empfaenger ist ein Referenz-Typ -- verglichen wird also
        // dieselbe Instanz, nicht eine mit gleichem Inhalt.
        (Value::BoundMethod(x), Value::BoundMethod(y)) =>
            x.1 == y.1 && value_eq(&x.0, &y.0),
        // Eine gebundene Methode ist NIE gleich einer freien Funktion, auch
        // wenn die Namen zufaellig uebereinstimmen.
        (Value::BoundMethod(_), Value::FuncRef(_)) | (Value::FuncRef(_), Value::BoundMethod(_)) => false,
        (Value::CompMarker, Value::CompMarker) => true,
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
