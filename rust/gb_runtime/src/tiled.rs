//! Tiled-Map-Loader (JSON) -- nativer Port von `gamebasic/modules/tiled.py`.
//!
//! Liest `.json`-Maps (Tiled "JSON Map"). Structs + Loader hier; die TILED_*-
//! Builtins liegen in `builtins.rs` (pur -- sie operieren auf dem
//! `Value::Tiled`-Handle + Dateisystem, brauchen keinen VM-State).

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::Value as J;

/// Typisierter Property-Wert (Tiled `{name,type,value}`).
#[derive(Clone)]
pub enum PropVal {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

impl PropVal {
    pub fn as_bool(&self) -> bool {
        match self {
            PropVal::Bool(b) => *b,
            PropVal::Int(i) => *i != 0,
            PropVal::Float(f) => *f != 0.0,
            PropVal::Str(s) => !s.is_empty(),
        }
    }
    pub fn as_int(&self) -> i64 {
        match self {
            PropVal::Bool(b) => if *b { 1 } else { 0 },
            PropVal::Int(i) => *i,
            PropVal::Float(f) => *f as i64,
            PropVal::Str(_) => 0,
        }
    }
    pub fn as_float(&self) -> f64 {
        match self {
            PropVal::Bool(b) => if *b { 1.0 } else { 0.0 },
            PropVal::Int(i) => *i as f64,
            PropVal::Float(f) => *f,
            PropVal::Str(_) => 0.0,
        }
    }
    pub fn as_string(&self) -> String {
        match self {
            PropVal::Bool(b) => if *b { "True".into() } else { "False".into() },
            PropVal::Int(i) => i.to_string(),
            PropVal::Float(f) => crate::value::Value::Float(*f).fmt(),
            PropVal::Str(s) => s.clone(),
        }
    }
}

pub struct TiledTileset {
    pub first_gid: i64,
    pub image: String,
    /// local_tile_id (gid - first_gid) -> {key: value}
    pub tile_properties: HashMap<i64, HashMap<String, PropVal>>,
}

pub struct TiledObject {
    pub name: String,
    pub type_: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub properties: HashMap<String, PropVal>,
}

pub struct TiledLayer {
    pub name: String,
    pub kind: String, // "tile" | "object" | "image" | <sonstiges>
    pub width: i64,
    pub height: i64,
    pub tiles: Vec<i64>,
    pub objects: Vec<TiledObject>,
    pub image: String,
    pub visible: bool,
    pub opacity: f64,
    pub obj_by_name: HashMap<String, Vec<usize>>,
}

pub struct TiledMap {
    pub width: i64,
    pub height: i64,
    pub tile_w: i64,
    pub tile_h: i64,
    pub tilesets: Vec<TiledTileset>, // nach first_gid sortiert
    pub layers: Vec<TiledLayer>,
    pub layer_by_name: HashMap<String, usize>,
}

impl TiledMap {
    /// (Tileset, local_id) fuer eine GID, oder (None,0).
    fn tileset_for_gid(&self, gid: i64) -> Option<(&TiledTileset, i64)> {
        if gid <= 0 {
            return None;
        }
        let mut best: Option<&TiledTileset> = None;
        for ts in &self.tilesets {
            if ts.first_gid <= gid {
                best = Some(ts);
            } else {
                break;
            }
        }
        best.map(|ts| (ts, gid - ts.first_gid))
    }

    pub fn tile_property(&self, gid: i64, key: &str) -> Option<&PropVal> {
        let (ts, local) = self.tileset_for_gid(gid)?;
        ts.tile_properties.get(&local)?.get(key)
    }

    // --- tile_collide-Helfer (Solid-Detection + Sweep) ---
    /// True, wenn irgendein Tileset irgendwo ein `solid`-Property gesetzt hat.
    pub fn solid_aware(&self) -> bool {
        self.tilesets.iter().any(|ts| {
            ts.tile_properties.values().any(|p| p.contains_key("solid"))
        })
    }

    pub fn is_solid_gid(&self, gid: i64) -> bool {
        if gid <= 0 {
            return false;
        }
        if self.solid_aware() {
            self.tile_property(gid, "solid").map(|p| p.as_bool()).unwrap_or(false)
        } else {
            gid > 0
        }
    }

    pub fn tile_is_solid_at(&self, layer: &TiledLayer, tx: i64, ty: i64) -> bool {
        if tx < 0 || ty < 0 || tx >= layer.width || ty >= layer.height {
            return true; // Welt-Rand blockiert
        }
        self.is_solid_gid(layer.tiles[(ty * layer.width + tx) as usize])
    }

    /// Achsen-aligned Box-Sweep (axis 0=X, 1=Y). Liefert (neue_pos, hit).
    /// 1:1-Port von `tile_collide._sweep_axis` (bit-identisch).
    pub fn sweep_axis(&self, layer: &TiledLayer, x: f64, y: f64, w: f64, h: f64, delta: f64, axis: i32) -> (f64, bool) {
        let tw = self.tile_w;
        let th = self.tile_h;
        if tw <= 0 || th <= 0 {
            return (if axis == 0 { x + delta } else { y + delta }, false);
        }
        if delta == 0.0 {
            return (if axis == 0 { x } else { y }, false);
        }
        let twf = tw as f64;
        let thf = th as f64;
        if axis == 0 {
            let new_x = x + delta;
            if delta > 0.0 {
                let front_edge = new_x + w;
                let mut tx_far = (front_edge / twf).floor() as i64;
                if front_edge == (tx_far * tw) as f64 {
                    tx_far -= 1;
                }
                let tx_old = if x + w > 0.0 { ((x + w - 1.0) / twf).floor() as i64 } else { -1 };
                let ty_top = (y / thf).floor() as i64;
                let ty_bot = ((y + h - 1.0) / thf).floor() as i64;
                for tx in (tx_old + 1)..(tx_far + 1) {
                    for ty in ty_top..(ty_bot + 1) {
                        if self.tile_is_solid_at(layer, tx, ty) {
                            return ((tx * tw) as f64 - w, true);
                        }
                    }
                }
                (new_x, false)
            } else {
                let front_edge = new_x;
                let tx_far = (front_edge / twf).floor() as i64;
                let tx_old = (x / twf).floor() as i64;
                let ty_top = (y / thf).floor() as i64;
                let ty_bot = ((y + h - 1.0) / thf).floor() as i64;
                for tx in (tx_far..=(tx_old - 1)).rev() {
                    for ty in ty_top..(ty_bot + 1) {
                        if self.tile_is_solid_at(layer, tx, ty) {
                            return (((tx + 1) * tw) as f64, true);
                        }
                    }
                }
                (new_x, false)
            }
        } else {
            let new_y = y + delta;
            if delta > 0.0 {
                let front_edge = new_y + h;
                let mut ty_far = (front_edge / thf).floor() as i64;
                if front_edge == (ty_far * th) as f64 {
                    ty_far -= 1;
                }
                let ty_old = if y + h > 0.0 { ((y + h - 1.0) / thf).floor() as i64 } else { -1 };
                let tx_left = (x / twf).floor() as i64;
                let tx_right = ((x + w - 1.0) / twf).floor() as i64;
                for ty in (ty_old + 1)..(ty_far + 1) {
                    for tx in tx_left..(tx_right + 1) {
                        if self.tile_is_solid_at(layer, tx, ty) {
                            return ((ty * th) as f64 - h, true);
                        }
                    }
                }
                (new_y, false)
            } else {
                let front_edge = new_y;
                let ty_far = (front_edge / thf).floor() as i64;
                let ty_old = (y / thf).floor() as i64;
                let tx_left = (x / twf).floor() as i64;
                let tx_right = ((x + w - 1.0) / twf).floor() as i64;
                for ty in (ty_far..=(ty_old - 1)).rev() {
                    for tx in tx_left..(tx_right + 1) {
                        if self.tile_is_solid_at(layer, tx, ty) {
                            return (((ty + 1) * th) as f64, true);
                        }
                    }
                }
                (new_y, false)
            }
        }
    }
}

// --- JSON-Parse-Helfer -----------------------------------------------------

fn jget<'a>(o: &'a J, k: &str) -> Option<&'a J> {
    o.get(k)
}
fn jstr(o: &J, k: &str, def: &str) -> String {
    o.get(k).and_then(|v| v.as_str()).unwrap_or(def).to_string()
}
fn jint(o: &J, k: &str, def: i64) -> i64 {
    o.get(k).and_then(|v| v.as_i64()).unwrap_or(def)
}
fn jfloat(o: &J, k: &str, def: f64) -> f64 {
    o.get(k).and_then(|v| v.as_f64()).unwrap_or(def)
}
fn jbool(o: &J, k: &str, def: bool) -> bool {
    o.get(k).and_then(|v| v.as_bool()).unwrap_or(def)
}

fn parse_properties(j: Option<&J>) -> HashMap<String, PropVal> {
    let mut out = HashMap::new();
    let arr = match j.and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return out,
    };
    for p in arr {
        let name = match p.get("name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let ptype = p.get("type").and_then(|v| v.as_str()).unwrap_or("string");
        let val = p.get("value");
        let pv = match ptype {
            "bool" => PropVal::Bool(val.and_then(|v| v.as_bool()).unwrap_or(false)),
            "int" => PropVal::Int(val.and_then(|v| v.as_i64()).unwrap_or(0)),
            "float" => PropVal::Float(val.and_then(|v| v.as_f64()).unwrap_or(0.0)),
            _ => PropVal::Str(val.and_then(|v| v.as_str()).unwrap_or("").to_string()),
        };
        out.insert(name, pv);
    }
    out
}

/// Lexikalische Normalisierung (wie os.path.normpath): `.`/`..` kollabieren,
/// Backslashes auf Windows. Kein Symlink-Resolve (anders als Python resolve(),
/// aber identisch im typischen relativen-cwd-Fall).
fn normpath(p: &Path) -> String {
    let mut out: Vec<std::ffi::OsString> = Vec::new();
    for comp in p.components() {
        use std::path::Component::*;
        match comp {
            ParentDir => {
                if matches!(out.last().map(|s| s.to_str()), Some(Some(".."))) || out.is_empty() {
                    out.push("..".into());
                } else {
                    out.pop();
                }
            }
            CurDir => {}
            other => out.push(other.as_os_str().to_os_string()),
        }
    }
    let mut pb = PathBuf::new();
    for c in out {
        pb.push(c);
    }
    pb.to_string_lossy().to_string()
}

fn parse_tileset(t: &J, base_dir: &Path) -> TiledTileset {
    let mut ts = TiledTileset {
        first_gid: jint(t, "firstgid", 1),
        image: String::new(),
        tile_properties: HashMap::new(),
    };
    let img = jstr(t, "image", "");
    if !img.is_empty() {
        ts.image = normpath(&base_dir.join(&img));
    }
    if let Some(tiles) = jget(t, "tiles").and_then(|v| v.as_array()) {
        for tile in tiles {
            let tile_id = match tile.get("id").and_then(|v| v.as_i64()) {
                Some(i) => i,
                None => continue,
            };
            let props = parse_properties(tile.get("properties"));
            if !props.is_empty() {
                ts.tile_properties.insert(tile_id, props);
            }
        }
    }
    ts
}

fn parse_object(o: &J) -> TiledObject {
    let type_ = {
        let t = o.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if t.is_empty() {
            o.get("class").and_then(|v| v.as_str()).unwrap_or("").to_string()
        } else {
            t.to_string()
        }
    };
    TiledObject {
        name: jstr(o, "name", ""),
        type_,
        x: jfloat(o, "x", 0.0),
        y: jfloat(o, "y", 0.0),
        width: jfloat(o, "width", 0.0),
        height: jfloat(o, "height", 0.0),
        properties: parse_properties(o.get("properties")),
    }
}

fn parse_layer(layer: &J) -> Result<TiledLayer, String> {
    let mut l = TiledLayer {
        name: jstr(layer, "name", ""),
        kind: "tile".into(),
        width: 0,
        height: 0,
        tiles: Vec::new(),
        objects: Vec::new(),
        image: String::new(),
        visible: true,
        opacity: 1.0,
        obj_by_name: HashMap::new(),
    };
    let raw_type = jstr(layer, "type", "tilelayer");
    match raw_type.as_str() {
        "tilelayer" => {
            l.kind = "tile".into();
            l.width = jint(layer, "width", 0);
            l.height = jint(layer, "height", 0);
            match layer.get("data") {
                Some(J::Array(a)) => {
                    l.tiles = a.iter().map(|x| x.as_i64().unwrap_or(0)).collect();
                }
                Some(J::String(_)) => {
                    return Err("TILED_LOAD: base64-codierte Tile-Daten werden nicht unterstuetzt. \
In Tiled: Edit -> Preferences -> 'Store tile layer data as: CSV' und Map neu speichern.".into());
                }
                _ => {}
            }
        }
        "objectgroup" => {
            l.kind = "object".into();
            if let Some(objs) = layer.get("objects").and_then(|v| v.as_array()) {
                for o in objs {
                    let obj = parse_object(o);
                    let name = obj.name.clone();
                    l.objects.push(obj);
                    l.obj_by_name.entry(name).or_default().push(l.objects.len() - 1);
                }
            }
        }
        "imagelayer" => {
            l.kind = "image".into();
            l.image = jstr(layer, "image", "");
        }
        other => {
            l.kind = other.to_string();
        }
    }
    l.visible = jbool(layer, "visible", true);
    l.opacity = jfloat(layer, "opacity", 1.0);
    Ok(l)
}

/// Laedt eine Tiled-JSON-Map (Pfad relativ zum cwd).
pub fn load(path: &str) -> Result<Rc<RefCell<TiledMap>>, String> {
    let resolved = crate::builtins::resolve_asset_path(path);
    let path = resolved.as_str();
    let text = std::fs::read_to_string(path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!("TILED_LOAD: Datei nicht gefunden: {}", path)
            } else {
                format!("TILED_LOAD: Lesefehler '{}': {}", path, e)
            }
        })?;
    let data: J = serde_json::from_str(&text)
        .map_err(|e| format!("TILED_LOAD: Lesefehler '{}': {}", path, e))?;
    if !data.is_object() {
        return Err("TILED_LOAD: Manifest muss JSON-Object sein".into());
    }
    if data.get("type").and_then(|v| v.as_str()) != Some("map") {
        return Err(format!(
            "TILED_LOAD: erwartet eine Tiled-Map (type='map'), erhalten type='{}'",
            data.get("type").and_then(|v| v.as_str()).unwrap_or("None")
        ));
    }
    // base_dir = absoluter Eltern-Ordner der Map.
    let abs = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    let abs = PathBuf::from(normpath(&abs));
    let base_dir = abs.parent().map(|p| p.to_path_buf()).unwrap_or_default();

    let mut m = TiledMap {
        width: jint(&data, "width", 0),
        height: jint(&data, "height", 0),
        tile_w: jint(&data, "tilewidth", 0),
        tile_h: jint(&data, "tileheight", 0),
        tilesets: Vec::new(),
        layers: Vec::new(),
        layer_by_name: HashMap::new(),
    };

    if let Some(tsets) = data.get("tilesets").and_then(|v| v.as_array()) {
        for t in tsets {
            if !t.is_object() {
                continue;
            }
            if let Some(src) = t.get("source").and_then(|v| v.as_str()) {
                // External tileset: laden + firstgid aus der Map uebernehmen.
                let ext_path = PathBuf::from(normpath(&base_dir.join(src)));
                let ext_text = std::fs::read_to_string(&ext_path).map_err(|e| {
                    format!("TILED_LOAD: External-Tileset '{}' nicht ladbar: {}", src, e)
                })?;
                let mut ext: J = serde_json::from_str(&ext_text).map_err(|e| {
                    format!("TILED_LOAD: External-Tileset '{}' nicht ladbar: {}", src, e)
                })?;
                if let Some(obj) = ext.as_object_mut() {
                    obj.insert("firstgid".into(), J::from(jint(t, "firstgid", 1)));
                }
                let ts_dir = ext_path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
                m.tilesets.push(parse_tileset(&ext, &ts_dir));
            } else {
                m.tilesets.push(parse_tileset(t, &base_dir));
            }
        }
    }
    m.tilesets.sort_by_key(|ts| ts.first_gid);

    if let Some(layers) = data.get("layers").and_then(|v| v.as_array()) {
        for layer in layers {
            if !layer.is_object() {
                continue;
            }
            let l = parse_layer(layer)?;
            let idx = m.layers.len();
            m.layer_by_name.insert(l.name.clone(), idx);
            m.layers.push(l);
        }
    }
    Ok(Rc::new(RefCell::new(m)))
}

/// 4-verbundener Flood-Fill (deterministisch; identisch zum Python-Fallback
/// und zum gb_native-BFS). Liefert Anzahl gefuellter Tiles.
pub fn flood_fill(tiles: &mut [i64], width: i64, height: i64, tx: i64, ty: i64, gid: i64) -> i64 {
    if tx < 0 || ty < 0 || tx >= width || ty >= height {
        return 0;
    }
    let idx = (ty * width + tx) as usize;
    let target = tiles[idx];
    if target == gid {
        return 0;
    }
    let mut stack = vec![(tx, ty)];
    let mut n = 0i64;
    while let Some((cx, cy)) = stack.pop() {
        if cx < 0 || cy < 0 || cx >= width || cy >= height {
            continue;
        }
        let i = (cy * width + cx) as usize;
        if tiles[i] != target {
            continue;
        }
        tiles[i] = gid;
        n += 1;
        stack.push((cx + 1, cy));
        stack.push((cx - 1, cy));
        stack.push((cx, cy + 1));
        stack.push((cx, cy - 1));
    }
    n
}
