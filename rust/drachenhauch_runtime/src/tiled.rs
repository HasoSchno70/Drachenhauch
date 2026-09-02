//! Tiled-Map-Loader (JSON) -- nativer Port von `drachenhauch/modules/tiled.py`.
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
    /// Wie viele Kacheln dieses Tileset hat. Beim Laden aus `tilecount`;
    /// gebraucht wird sie, um beim ANHAENGEN eines weiteren Tilesets die
    /// naechste freie GID zu finden -- ueberlappende GID-Bereiche zerstoeren
    /// stillschweigend die Zuordnung aller Kacheln.
    pub tile_count: i64,
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
        // Review-Fund: Tiled kodiert Flip-Zustand (horizontal/vertikal/
        // diagonal) in den oberen 3 Bits einer GID (0x80000000/0x40000000/
        // 0x20000000); die eigentliche Tile-ID ist `gid & 0x1FFFFFFF`. Ohne
        // diese Maskierung landete eine gespiegelte Kachel (vom Tiled-Editor
        // ganz normal per Klick erzeugt) bei einer riesigen GID, die
        // `tileset_for_gid` auf das FALSCHE (meist letzte) Tileset abbildete
        // -- `tile_property`/`is_solid_gid` fanden dann kein `solid`-Property
        // mehr, gespiegelte Waende liessen den Spieler stillschweigend
        // hindurchlaufen.
        let gid = gid & 0x1FFF_FFFF;
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
    /// Alle Eigenschaften EINER Kachel (fuer TILED_TILE_PROP_KEYS).
    /// `tile_property` beantwortet nur die Frage nach einem bekannten
    /// Schluessel -- anzeigen laesst sich damit nichts.
    pub fn tile_properties_of(&self, gid: i64) -> Option<&HashMap<String, PropVal>> {
        let (ts, local) = self.tileset_for_gid(gid)?;
        ts.tile_properties.get(&local)
    }

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
        tile_count: jint(t, "tilecount", 0),
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
            // Review-Fund: width/height wurden nie auf Plausibilitaet
            // geprueft -- ein negativer Wert oder ein absurd grosser Wert
            // (i64::MAX) liess `width * height` weiter unten ueberlaufen.
            if l.width < 0 || l.height < 0 {
                return Err(format!(
                    "TILED_LOAD: Layer '{}' hat negative width/height ({}/{})",
                    l.name, l.width, l.height));
            }
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
            // Review-Fund: `tiles.len()` wurde nie gegen `width*height`
            // geprueft -- jeder Aufrufer indiziert stattdessen ueber
            // `ty * layer.width + tx` (gegen den HEADER, nicht den
            // tatsaechlichen Vektor) -- eine gekuerzte/manipulierte `data`
            // liess `tile_is_solid_at`/`TILED_TILE_AT`/`TILE_COLLIDE` mit
            // einem rohen Index-Out-Of-Bounds-Panic abstuerzen statt eines
            // sauberen TILED_LOAD-Fehlers.
            let expected = (l.width as i64).checked_mul(l.height as i64);
            match expected {
                Some(n) if n as usize == l.tiles.len() => {}
                _ => return Err(format!(
                    "TILED_LOAD: Layer '{}' hat {} Tile(s), erwartet {}x{}={}",
                    l.name, l.tiles.len(), l.width, l.height,
                    expected.map(|n| n.to_string()).unwrap_or_else(|| "??".into()))),
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

/// Eine leere Karte anlegen (TILED_NEW).
pub fn neu(w: i64, h: i64, tw: i64, th: i64) -> Result<Rc<RefCell<TiledMap>>, String> {
    if w < 1 || h < 1 || tw < 1 || th < 1 {
        return Err("TILED_NEW: Breite, Hoehe und Kachelmasse muessen >= 1 sein".into());
    }
    if w * h > 4_000_000 {
        return Err(std::format!(
            "TILED_NEW: {}x{} Kacheln sind zu viel (Obergrenze 4 Millionen)", w, h));
    }
    Ok(Rc::new(RefCell::new(TiledMap {
        width: w, height: h, tile_w: tw, tile_h: th,
        tilesets: Vec::new(), layers: Vec::new(), layer_by_name: HashMap::new(),
    })))
}

/// Eine leere Kachel-Ebene anhaengen (TILED_ADD_LAYER) -> Index.
pub fn ebene_anhaengen(m: &Rc<RefCell<TiledMap>>, name: &str) -> Result<i64, String> {
    let mut map = m.borrow_mut();
    if map.layer_by_name.contains_key(name) {
        return Err(std::format!("TILED_ADD_LAYER: '{}' gibt es schon", name));
    }
    let (w, h) = (map.width, map.height);
    let idx = map.layers.len();
    map.layers.push(TiledLayer {
        name: name.to_string(), kind: "tile".into(), width: w, height: h,
        tiles: vec![0; (w * h) as usize], objects: Vec::new(),
        image: String::new(), visible: true, opacity: 1.0,
        obj_by_name: HashMap::new(),
    });
    map.layer_by_name.insert(name.to_string(), idx);
    Ok(idx as i64)
}

/// Den Namensindex nach einer Umbenennung/Entfernung neu aufbauen.
///
/// Er bildet Name -> Position ab; jede Aenderung an der Reihenfolge macht
/// ihn falsch, und ein falscher Index zeigt stillschweigend auf die
/// NACHBAR-Ebene statt einen Fehler zu melden.
fn namen_neu(map: &mut TiledMap) {
    map.layer_by_name.clear();
    for (i, l) in map.layers.iter().enumerate() {
        map.layer_by_name.insert(l.name.clone(), i);
    }
}

/// Eine Ebene umbenennen (TILED_LAYER_RENAME).
pub fn ebene_umbenennen(m: &Rc<RefCell<TiledMap>>, idx: i64, name: &str) -> Result<(), String> {
    let mut map = m.borrow_mut();
    if idx < 0 || idx as usize >= map.layers.len() {
        return Err(std::format!("TILED_LAYER_RENAME: Ebene {} gibt es nicht", idx));
    }
    if name.is_empty() {
        return Err("TILED_LAYER_RENAME: der Name darf nicht leer sein".into());
    }
    // Der eigene alte Name ist kein Konflikt -- sonst waere ein Umbenennen
    // auf denselben Namen (aus einem Eingabefeld heraus der Normalfall) ein
    // Fehler.
    if let Some(&vorhanden) = map.layer_by_name.get(name) {
        if vorhanden != idx as usize {
            return Err(std::format!("TILED_LAYER_RENAME: '{}' gibt es schon", name));
        }
    }
    map.layers[idx as usize].name = name.to_string();
    namen_neu(&mut map);
    Ok(())
}

/// Eine Ebene ein- oder ausblenden (TILED_LAYER_SET_VISIBLE).
pub fn ebene_sichtbar_setzen(m: &Rc<RefCell<TiledMap>>, idx: i64, an: bool) -> Result<(), String> {
    let mut map = m.borrow_mut();
    if idx < 0 || idx as usize >= map.layers.len() {
        return Err(std::format!("TILED_LAYER_SET_VISIBLE: Ebene {} gibt es nicht", idx));
    }
    map.layers[idx as usize].visible = an;
    Ok(())
}

/// Eine Ebene entfernen (TILED_REMOVE_LAYER).
pub fn ebene_entfernen(m: &Rc<RefCell<TiledMap>>, idx: i64) -> Result<(), String> {
    let mut map = m.borrow_mut();
    if idx < 0 || idx as usize >= map.layers.len() {
        return Err(std::format!("TILED_REMOVE_LAYER: Ebene {} gibt es nicht", idx));
    }
    map.layers.remove(idx as usize);
    namen_neu(&mut map);
    Ok(())
}

/// Eine OBJEKT-Ebene anhaengen (TILED_ADD_OBJECT_LAYER) -> Index.
///
/// Eigene Funktion statt eines Schalters an `ebene_anhaengen`: die beiden
/// Arten teilen fast nichts. Eine Kachelebene ist ein Feld fester Groesse
/// (`tiles`), eine Objektebene eine Liste -- und `speichern` schreibt sie in
/// verschiedene Formen ("tilelayer" gegen "objectgroup").
pub fn objekt_ebene_anhaengen(m: &Rc<RefCell<TiledMap>>, name: &str) -> Result<i64, String> {
    let mut map = m.borrow_mut();
    if map.layer_by_name.contains_key(name) {
        return Err(std::format!("TILED_ADD_OBJECT_LAYER: '{}' gibt es schon", name));
    }
    if name.is_empty() {
        return Err("TILED_ADD_OBJECT_LAYER: der Name darf nicht leer sein".into());
    }
    let idx = map.layers.len();
    map.layers.push(TiledLayer {
        name: name.to_string(), kind: "object".into(),
        // Breite/Hoehe bleiben 0 und `tiles` leer: eine Objektebene hat kein
        // Raster. Tiled schreibt bei einer objectgroup ebenfalls keins.
        width: 0, height: 0, tiles: Vec::new(), objects: Vec::new(),
        image: String::new(), visible: true, opacity: 1.0,
        obj_by_name: HashMap::new(),
    });
    map.layer_by_name.insert(name.to_string(), idx);
    Ok(idx as i64)
}

/// Die Position einer Objektebene ueber ihren Namen (mit Pruefung der Art).
fn objekt_ebene(map: &TiledMap, ebene: &str, fn_: &str) -> Result<usize, String> {
    match map.layer_by_name.get(ebene) {
        Some(&i) if map.layers[i].kind == "object" => Ok(i),
        Some(_) => Err(std::format!("{}: '{}' ist keine Objekt-Ebene", fn_, ebene)),
        None => Err(std::format!("{}: Ebene '{}' gibt es nicht", fn_, ebene)),
    }
}

/// Ein Objekt anhaengen (TILED_ADD_OBJECT) -> Index INNERHALB seiner Ebene.
///
/// Angesprochen wird die Ebene ueber ihren NAMEN, wie bei allen
/// TILED_OBJECT_*-Abfragen -- eine Objektebene heisst "spawns" oder
/// "trigger", und danach sucht ein Spiel sie auch.
#[allow(clippy::too_many_arguments)]
pub fn objekt_anhaengen(m: &Rc<RefCell<TiledMap>>, ebene: &str, name: &str, typ: &str,
                        x: f64, y: f64, w: f64, h: f64) -> Result<i64, String> {
    let mut map = m.borrow_mut();
    let li = objekt_ebene(&map, ebene, "TILED_ADD_OBJECT")?;
    let l = &mut map.layers[li];
    let idx = l.objects.len();
    l.objects.push(TiledObject {
        name: name.to_string(), type_: typ.to_string(),
        x, y, width: w, height: h, properties: HashMap::new(),
    });
    // Der Namensindex wird MITgefuehrt, nicht spaeter neu gebaut: die
    // TILED_OBJECT_*-Abfragen suchen darueber, und ein fehlender Eintrag
    // liesse ein gerade angelegtes Objekt unauffindbar erscheinen.
    l.obj_by_name.entry(name.to_string()).or_default().push(idx);
    Ok(idx as i64)
}

/// Eigenschaft eines Objekts setzen (`Some`) oder entfernen (`None`).
pub fn objekt_eigenschaft(m: &Rc<RefCell<TiledMap>>, ebene: &str, idx: i64, key: &str,
                          wert: Option<PropVal>) -> Result<(), String> {
    let fn_ = if wert.is_some() { "TILED_OBJECT_SET_PROP" } else { "TILED_OBJECT_REMOVE_PROP" };
    let mut map = m.borrow_mut();
    let li = objekt_ebene(&map, ebene, fn_)?;
    let l = &mut map.layers[li];
    if idx < 0 || idx as usize >= l.objects.len() {
        return Err(std::format!("{}: Objekt {} gibt es in '{}' nicht", fn_, idx, ebene));
    }
    if key.is_empty() {
        return Err(std::format!("{}: der Schluessel darf nicht leer sein", fn_));
    }
    match wert {
        Some(v) => { l.objects[idx as usize].properties.insert(key.to_string(), v); }
        None => { l.objects[idx as usize].properties.remove(key); }
    }
    Ok(())
}

/// Eigenschaft einer KACHEL setzen (`Some`) oder entfernen (`None`).
///
/// Angesprochen ueber die GID, wie beim Lesen (`TILED_TILE_PROP_*`) --
/// gespeichert wird sie beim Tileset unter der lokalen Nummer, aber das ist
/// Buchhaltung, die ein Programm nicht kennen muss. Die Flip-Bits werden
/// dabei genauso maskiert wie beim Lesen: sonst legte eine gespiegelte
/// Kachel ihre Eigenschaft woanders ab als eine ungespiegelte.
pub fn kachel_eigenschaft(m: &Rc<RefCell<TiledMap>>, gid: i64, key: &str,
                          wert: Option<PropVal>) -> Result<(), String> {
    let fn_ = if wert.is_some() { "TILED_TILE_SET_PROP" } else { "TILED_TILE_REMOVE_PROP" };
    if key.is_empty() {
        return Err(std::format!("{}: der Schluessel darf nicht leer sein", fn_));
    }
    let gid = gid & 0x1FFF_FFFF;
    let mut map = m.borrow_mut();
    // Dasselbe "letztes passendes Tileset"-Verfahren wie `tileset_for_gid`,
    // nur mit dem INDEX -- fuer das Aendern braucht es eine veraenderliche
    // Ausleihe, und die gibt eine Referenz nicht her.
    let mut treffer: Option<usize> = None;
    for (i, ts) in map.tilesets.iter().enumerate() {
        if ts.first_gid <= gid { treffer = Some(i); } else { break; }
    }
    let ti = treffer.ok_or_else(|| std::format!(
        "{}: zu GID {} gibt es kein Tileset", fn_, gid))?;
    let lokal = gid - map.tilesets[ti].first_gid;
    let ts = &mut map.tilesets[ti];
    if lokal < 0 || lokal >= ts.tile_count {
        return Err(std::format!(
            "{}: GID {} liegt ausserhalb des Tilesets (0..{})", fn_, gid, ts.tile_count - 1));
    }
    match wert {
        Some(v) => { ts.tile_properties.entry(lokal).or_default().insert(key.to_string(), v); }
        None => {
            if let Some(props) = ts.tile_properties.get_mut(&lokal) {
                props.remove(key);
                // Leere Eintraege wieder loswerden -- `speichern` listet
                // jede Kachel auf, die einen Eintrag hat, und eine mit einer
                // leeren Eigenschaftsliste waere Rauschen in der Datei.
                if props.is_empty() { ts.tile_properties.remove(&lokal); }
            }
        }
    }
    Ok(())
}

/// Ein Tileset anhaengen (TILED_ADD_TILESET) -> Index.
///
/// Die `firstgid` wird selbst vergeben: erste GID = 1, danach jeweils hinter
/// dem vorigen Tileset. Sie von Hand setzen zu lassen waere die haeufigste
/// Fehlerquelle -- ueberlappende Bereiche zerstoeren stillschweigend die
/// Zuordnung aller Kacheln.
pub fn tileset_anhaengen(m: &Rc<RefCell<TiledMap>>, bild: &str, kacheln: i64)
                         -> Result<i64, String> {
    if kacheln < 1 {
        return Err("TILED_ADD_TILESET: die Kachelzahl muss >= 1 sein".into());
    }
    let mut map = m.borrow_mut();
    let first = map.tilesets.last().map(|t| t.first_gid + t.tile_count).unwrap_or(1);
    let idx = map.tilesets.len();
    map.tilesets.push(TiledTileset {
        first_gid: first, image: bild.to_string(), tile_count: kacheln,
        tile_properties: HashMap::new(),
    });
    Ok(idx as i64)
}

/// Die Karte als Tiled-JSON schreiben (TILED_SAVE).
///
/// Erzeugt genau die Form, die `load` wieder liest -- eingebettete Tilesets,
/// CSV-Kacheldaten. Der Rundweg ist getestet; ein Schreiber, den nur der
/// eigene Leser versteht, waere kein Dateiformat, sondern ein Notizzettel.
pub fn speichern(m: &Rc<RefCell<TiledMap>>, pfad: &str) -> Result<(), String> {
    let map = m.borrow();
    let tilesets: Vec<J> = map.tilesets.iter().map(|t| serde_json::json!({
        "firstgid": t.first_gid,
        "image": t.image,
        "tilecount": t.tile_count,
        "name": std::path::Path::new(&t.image).file_stem()
                    .and_then(|s| s.to_str()).unwrap_or("tileset"),
        "tilewidth": map.tile_w,
        "tileheight": map.tile_h,
        // Nur Kacheln MIT Eigenschaften auflisten -- Tiled macht es genauso,
        // und bei 2000 Kacheln waere alles andere eine unlesbare Datei.
        "tiles": t.tile_properties.iter().map(|(id, props)| serde_json::json!({
            "id": id,
            "properties": props.iter().map(|(k, v)| serde_json::json!({
                "name": k,
                "type": match v { PropVal::Bool(_) => "bool", PropVal::Int(_) => "int",
                                  PropVal::Float(_) => "float", PropVal::Str(_) => "string" },
                "value": match v {
                    PropVal::Bool(b) => J::from(*b),
                    PropVal::Int(i) => J::from(*i),
                    PropVal::Float(f) => J::from(*f),
                    PropVal::Str(s) => J::from(s.clone()),
                },
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })).collect();

    let layers: Vec<J> = map.layers.iter().enumerate().map(|(i, l)| {
        let mut o = serde_json::json!({
            "id": i + 1,
            "name": l.name,
            "opacity": l.opacity,
            "visible": l.visible,
            "x": 0, "y": 0,
        });
        if l.kind == "object" {
            o["type"] = J::from("objectgroup");
            o["objects"] = J::from(l.objects.iter().enumerate().map(|(oi, ob)| serde_json::json!({
                "id": oi + 1, "name": ob.name, "type": ob.type_,
                "x": ob.x, "y": ob.y, "width": ob.width, "height": ob.height,
                "visible": true, "rotation": 0,
                "properties": ob.properties.iter().map(|(k, v)| serde_json::json!({
                    "name": k,
                    "type": match v { PropVal::Bool(_) => "bool", PropVal::Int(_) => "int",
                                      PropVal::Float(_) => "float", PropVal::Str(_) => "string" },
                    "value": match v {
                        PropVal::Bool(b) => J::from(*b),
                        PropVal::Int(i) => J::from(*i),
                        PropVal::Float(f) => J::from(*f),
                        PropVal::Str(s) => J::from(s.clone()),
                    },
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>());
        } else {
            o["type"] = J::from("tilelayer");
            o["width"] = J::from(l.width);
            o["height"] = J::from(l.height);
            o["data"] = J::from(l.tiles.clone());
        }
        o
    }).collect();

    let doc = serde_json::json!({
        "type": "map",
        "version": "1.10",
        "tiledversion": "1.10.2",
        "orientation": "orthogonal",
        "renderorder": "right-down",
        "infinite": false,
        "width": map.width, "height": map.height,
        "tilewidth": map.tile_w, "tileheight": map.tile_h,
        "nextlayerid": map.layers.len() + 1,
        "nextobjectid": 1,
        "tilesets": tilesets,
        "layers": layers,
    });
    let text = serde_json::to_string_pretty(&doc)
        .map_err(|e| std::format!("TILED_SAVE: {}", e))?;
    std::fs::write(pfad, text)
        .map_err(|e| std::format!("TILED_SAVE: '{}' liess sich nicht schreiben -- {}", pfad, e))
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
