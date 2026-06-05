//! Pure/deterministische Builtins fuer die Rust-VM (Schritt 3).
//!
//! Semantik 1:1 aus `gamebasic/interpreter.py` (BUILTINS). Nicht-determi-
//! nistische (RND/MILLIS/TIME$/…) und Grafik-Builtins sind NICHT hier --
//! sie liefern einen klaren Fehler (Grafik = Schritt 4).

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, Read, Write};
use std::rc::Rc;

use crate::tiled::{TiledLayer, TiledMap, TiledObject};
use crate::value::{as_f64, is_num, value_eq, FileH, GbArray, GbFile, GbMap, Particle, ParticleSys, SaveHandle, SpriteObj, TweenObj, Value};

type R = Result<Value, String>;

fn err(msg: impl Into<String>) -> R {
    Err(msg.into())
}

// --- PRNG fuer RND/RANDOMIZE (nicht-deterministisch wie Python; NICHT
// bit-identisch -- Programme mit RND ohne Seed sind erwartet unterschiedlich).
thread_local! {
    static RNG: std::cell::Cell<u64> = std::cell::Cell::new(seed_from_time());
}
fn seed_from_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x2545F4914F6CDD1D) | 1
}
fn next_rand() -> u64 {
    RNG.with(|s| {
        let mut x = s.get();
        if x == 0 { x = 0x9E3779B97F4A7C15; }
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        s.set(x);
        x
    })
}

/// Lokale Datum/Uhrzeit als (Jahr, Monat, Tag, Stunde, Minute, Sekunde).
/// Pendant zu Pythons `time.strftime` (LOKALzeit) fuer TIME$/DATE$. Auf Windows
/// liefert `GetLocalTime` die Felder direkt (inkl. Zeitzone); sonst Fallback auf
/// UTC, berechnet aus dem Unix-Timestamp (civil-from-days). Clock-basiert ->
/// Parity-Tests behandeln TIME$/DATE$ ohnehin als "erwartet unterschiedlich".
#[cfg(windows)]
fn local_datetime() -> (i64, i64, i64, i64, i64, i64) {
    #[repr(C)]
    struct SystemTimeW {
        w_year: u16, w_month: u16, w_day_of_week: u16, w_day: u16,
        w_hour: u16, w_minute: u16, w_second: u16, w_milliseconds: u16,
    }
    extern "system" {
        fn GetLocalTime(lp_system_time: *mut SystemTimeW);
    }
    unsafe {
        let mut st: SystemTimeW = std::mem::zeroed();
        GetLocalTime(&mut st);
        (st.w_year as i64, st.w_month as i64, st.w_day as i64,
         st.w_hour as i64, st.w_minute as i64, st.w_second as i64)
    }
}

#[cfg(not(windows))]
fn local_datetime() -> (i64, i64, i64, i64, i64, i64) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Howard Hinnant's civil_from_days (UTC).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    (y, mo, d, h, mi, s)
}

fn need_str<'a>(v: &'a Value, fn_: &str) -> Result<&'a str, String> {
    match v {
        Value::Str(s) => Ok(s),
        _ => Err(format!("{} erwartet STRING, erhalten {}", fn_, v.type_name())),
    }
}

fn need_int(v: &Value, fn_: &str) -> Result<i64, String> {
    match v {
        Value::Int(i) => Ok(*i),
        _ => Err(format!("{} erwartet INTEGER, erhalten {}", fn_, v.type_name())),
    }
}

fn need_num(v: &Value, fn_: &str) -> Result<f64, String> {
    match v {
        Value::Int(i) => Ok(*i as f64),
        Value::Float(f) => Ok(*f),
        _ => Err(format!("{} erwartet Zahl, erhalten {}", fn_, v.type_name())),
    }
}

fn type_default(t: &str) -> Value {
    match t {
        "integer" => Value::Int(0),
        "float" => Value::Float(0.0),
        "string" => Value::str_rc(""),
        "boolean" => Value::Bool(false),
        _ => Value::Nil,
    }
}

/// Parst einen String robust zu INTEGER (ganzzahlig, kein '.'/'e') oder FLOAT,
/// sonst None. Basis fuer ISNUMERIC (.is_some()) und TRYVAL (.unwrap_or(default)).
fn parse_number(s: &str) -> Option<Value> {
    let t = s.trim();
    if t.is_empty() { return None; }
    if !t.contains('.') && !t.contains('e') && !t.contains('E') {
        if let Ok(i) = t.parse::<i64>() { return Some(Value::Int(i)); }
    }
    t.parse::<f64>().ok().map(Value::Float)
}

/// Element-Coercion fuer Array-Schreibzugriffe (ARRAY_FILL/PUSH/INSERT/REDIM).
/// Identisch zur `coerce`-Logik der VM (vm.rs): INTEGER nimmt Int oder
/// ganzzahligen Float, FLOAT nimmt Float/Int, beides lehnt Bool ab; Referenz-/
/// sonstige Typen werden durchgereicht.
fn coerce_elem(value: Value, target: &str, ctx: &str) -> R {
    match target {
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
        _ => Ok(value),
    }
}

/// STR$-Semantik (`_b_str`).
fn str_of(v: &Value) -> String {
    match v {
        Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            if f.is_finite() && f.fract() == 0.0 {
                format!("{:.1}", f)
            } else {
                format!("{}", f)
            }
        }
        Value::Str(s) => s.to_string(),
        other => other.fmt(),
    }
}

// --- regex-Modul (REGEX_*) -------------------------------------------------
// Pattern-Cache (wie Python: 256 Eintraege, dann leeren). regex::Regex ist
// Clone (intern Arc) -> billig.
thread_local! {
    static RE_CACHE: RefCell<HashMap<String, regex::Regex>> = RefCell::new(HashMap::new());
}

fn regex_compile(pat: &str) -> Result<regex::Regex, String> {
    RE_CACHE.with(|c| {
        if let Some(r) = c.borrow().get(pat) {
            return Ok(r.clone());
        }
        match regex::Regex::new(pat) {
            Ok(r) => {
                let mut m = c.borrow_mut();
                if m.len() >= 256 { m.clear(); }
                m.insert(pat.to_string(), r.clone());
                Ok(r)
            }
            Err(e) => Err(format!("Ungueltige Regex '{}': {}", pat, e)),
        }
    })
}

/// Uebersetzt einen Python-`re.sub`-Replacement-String in die Rust-regex-
/// Syntax: `\1`->`${1}`, `\g<name>`->`${name}`, literales `$`->`$$`, `\\`->`\`.
fn translate_repl(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '$' => out.push_str("$$"),
            '\\' => match it.peek().copied() {
                Some(d) if d.is_ascii_digit() => {
                    let mut num = String::new();
                    while let Some(d) = it.peek().copied() {
                        if d.is_ascii_digit() { num.push(d); it.next(); } else { break; }
                    }
                    out.push_str(&format!("${{{}}}", num));
                }
                Some('g') => {
                    it.next();
                    if it.peek() == Some(&'<') {
                        it.next();
                        let mut name = String::new();
                        while let Some(ch) = it.peek().copied() {
                            it.next();
                            if ch == '>' { break; }
                            name.push(ch);
                        }
                        out.push_str(&format!("${{{}}}", name));
                    } else {
                        out.push('g');
                    }
                }
                Some('\\') => { it.next(); out.push('\\'); }
                Some(other) => { it.next(); out.push(other); }
                None => out.push('\\'),
            },
            other => out.push(other),
        }
    }
    out
}

fn new_str_array(items: Vec<String>) -> Value {
    let n = items.len() as i64;
    let mut arr = GbArray::new("string".to_string(), vec![n], || Value::str_rc(""));
    for (i, s) in items.into_iter().enumerate() {
        arr.values[i] = Value::str_rc(&s);
    }
    Value::Array(Rc::new(RefCell::new(arr)))
}

/// Banker's Rounding (Python `round`), Ergebnis als i64.
fn round_half_even(x: f64) -> i64 {
    let f = x.floor();
    let diff = x - f;
    let r = if diff < 0.5 {
        f
    } else if diff > 0.5 {
        f + 1.0
    } else {
        // exakt .5 -> zur geraden Zahl
        if (f as i64).rem_euclid(2) == 0 { f } else { f + 1.0 }
    };
    r as i64
}

/// Minimaler printf fuer FORMAT$ / f-String-Specs: %[-][0][width][.prec](d|i|f|s|x|X).
fn format_one(mask: &str, v: &Value) -> R {
    let bytes: Vec<char> = mask.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let mut used = false;
    while i < bytes.len() {
        if bytes[i] != '%' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i < bytes.len() && bytes[i] == '%' {
            out.push('%');
            i += 1;
            continue;
        }
        let mut left = false;
        let mut zero = false;
        while i < bytes.len() && (bytes[i] == '-' || bytes[i] == '0' || bytes[i] == '+' || bytes[i] == ' ') {
            if bytes[i] == '-' { left = true; }
            if bytes[i] == '0' { zero = true; }
            i += 1;
        }
        let mut width = 0usize;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            width = width * 10 + (bytes[i] as usize - '0' as usize);
            i += 1;
        }
        let mut prec: Option<usize> = None;
        if i < bytes.len() && bytes[i] == '.' {
            i += 1;
            let mut p = 0usize;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                p = p * 10 + (bytes[i] as usize - '0' as usize);
                i += 1;
            }
            prec = Some(p);
        }
        if i >= bytes.len() {
            return err(format!("FORMAT$: unvollstaendige Maske '{}'", mask));
        }
        let conv = bytes[i];
        i += 1;
        used = true;
        let s = match conv {
            'd' | 'i' => {
                let n = match v {
                    Value::Int(n) => *n,
                    Value::Bool(b) => *b as i64,
                    Value::Float(f) => *f as i64,
                    _ => return err(format!("FORMAT$: %{} erwartet Zahl", conv)),
                };
                n.to_string()
            }
            'f' => {
                let f = need_num(v, "FORMAT$")?;
                format!("{:.*}", prec.unwrap_or(6), f)
            }
            'x' => format!("{:x}", need_int(v, "FORMAT$")?),
            'X' => format!("{:X}", need_int(v, "FORMAT$")?),
            's' => match v {
                Value::Str(s) => s.to_string(),
                Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
                other => str_of(other),
            },
            c => return err(format!("FORMAT$: unbekannter Spezifizierer %{}", c)),
        };
        // Padding
        if s.len() < width {
            let pad = width - s.len();
            if left {
                out.push_str(&s);
                out.push_str(&" ".repeat(pad));
            } else if zero && conv != 's' {
                // Zero-Pad nach evtl. Vorzeichen
                if let Some(rest) = s.strip_prefix('-') {
                    out.push('-');
                    out.push_str(&"0".repeat(pad));
                    out.push_str(rest);
                } else {
                    out.push_str(&"0".repeat(pad));
                    out.push_str(&s);
                }
            } else {
                out.push_str(&" ".repeat(pad));
                out.push_str(&s);
            }
        } else {
            out.push_str(&s);
        }
    }
    let _ = used;
    Ok(Value::str_rc(&out))
}

/// Liefert `Some(Result)` wenn `name` ein bekanntes pures Builtin ist;
/// `None`, wenn unbekannt (dann pruefen die Aufrufer auf Grafik/etc.).
// --- tiled-Modul (TILED_*) -------------------------------------------------
fn tiled_h(v: &Value, fn_: &str) -> Result<Rc<RefCell<TiledMap>>, String> {
    match v {
        Value::Tiled(m) => Ok(m.clone()),
        _ => Err(format!("{} erwartet TILED_MAP", fn_)),
    }
}

fn layer_by_idx<'a>(m: &'a TiledMap, idx: i64, fn_: &str) -> Result<&'a TiledLayer, String> {
    if idx < 0 || idx as usize >= m.layers.len() {
        return Err(format!("{}: Layer-Index {} ausserhalb [0..{}]", fn_, idx, m.layers.len() as i64 - 1));
    }
    Ok(&m.layers[idx as usize])
}

fn tile_layer_idx(m: &TiledMap, idx: i64, fn_: &str) -> Result<usize, String> {
    let l = layer_by_idx(m, idx, fn_)?;
    if l.kind != "tile" {
        return Err(format!("{}: Layer {} ('{}') ist kein Tile-Layer", fn_, idx, l.name));
    }
    Ok(idx as usize)
}

fn layer_by_name<'a>(m: &'a TiledMap, name: &str, fn_: &str) -> Result<&'a TiledLayer, String> {
    match m.layer_by_name.get(name) {
        Some(&i) => Ok(&m.layers[i]),
        None => Err(format!("{}: Layer '{}' nicht gefunden", fn_, name)),
    }
}

fn get_obj<'a>(m: &'a TiledMap, name: &str, idx: i64, fn_: &str) -> Result<&'a TiledObject, String> {
    let layer = layer_by_name(m, name, fn_)?;
    if layer.kind != "object" {
        return Err(format!("{}: Layer '{}' ist kein Object-Layer", fn_, name));
    }
    if idx < 0 || idx as usize >= layer.objects.len() {
        return Err(format!("{}: Object-Index {} ausserhalb [0..{}]", fn_, idx, layer.objects.len() as i64 - 1));
    }
    Ok(&layer.objects[idx as usize])
}

fn need_bool(v: &Value, fn_: &str) -> Result<bool, String> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err(format!("{} erwartet BOOLEAN, erhalten {}", fn_, v.type_name())),
    }
}

fn char_h(v: &Value, fn_: &str) -> Result<Rc<RefCell<crate::controller::CharController>>, String> {
    match v {
        Value::CharController(c) => Ok(c.clone()),
        _ => Err(format!("{} erwartet CHAR_CONTROLLER", fn_)),
    }
}

pub fn call_builtin(name: &str, args: &[Value]) -> Option<R> {
    Some(call_inner(name, args))
}

fn call_inner(name: &str, a: &[Value]) -> R {
    macro_rules! arity {
        ($n:expr) => {
            if a.len() != $n {
                return err(format!("{}: erwartet {} Argument(e), erhalten {}", name.to_uppercase(), $n, a.len()));
            }
        };
    }
    match name {
        "str$" | "str" => { arity!(1); Ok(Value::str_rc(&str_of(&a[0]))) }
        "val" => {
            arity!(1);
            let s = need_str(&a[0], "VAL")?.trim().to_string();
            if s.contains('.') {
                Ok(Value::Float(s.parse::<f64>().unwrap_or(0.0)))
            } else {
                Ok(Value::Int(s.parse::<i64>().unwrap_or(0)))
            }
        }
        "int" => { arity!(1); Ok(Value::Int(need_num(&a[0], "INT")?.floor() as i64)) }
        "abs" => {
            arity!(1);
            match &a[0] {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                v => err(format!("ABS erwartet Zahl, erhalten {}", v.type_name())),
            }
        }
        "is_nil" => { arity!(1); Ok(Value::Bool(matches!(a[0], Value::Nil))) }
        "rnd" => {
            // RND() -> Float [0,1); RND(n) -> Int [0, n-1].
            if a.is_empty() {
                Ok(Value::Float((next_rand() >> 11) as f64 / (1u64 << 53) as f64))
            } else {
                let n = need_int(&a[0], "RND")?;
                if n <= 0 { return err("RND erwartet positives INTEGER"); }
                Ok(Value::Int((next_rand() % n as u64) as i64))
            }
        }
        "randomize" => {
            let seed = if a.is_empty() { seed_from_time() } else { need_int(&a[0], "RANDOMIZE")? as u64 | 1 };
            RNG.with(|s| s.set(if seed == 0 { 1 } else { seed }));
            Ok(Value::Nil)
        }
        "randint" => {
            // RANDINT(lo, hi) -> Int [lo, hi] (inklusiv). PRNG != Python ->
            // erwartet unterschiedlich in der Parity.
            arity!(2);
            let lo = need_int(&a[0], "RANDINT")?;
            let hi = need_int(&a[1], "RANDINT")?;
            if lo > hi { return err("RANDINT: lo darf nicht groesser als hi sein".to_string()); }
            let span = (hi - lo) as u64 + 1;
            Ok(Value::Int(lo + (next_rand() % span) as i64))
        }
        "randf" => {
            arity!(2);
            let lo = need_num(&a[0], "RANDF")?;
            let hi = need_num(&a[1], "RANDF")?;
            let r = (next_rand() >> 11) as f64 / (1u64 << 53) as f64;
            Ok(Value::Float(lo + r * (hi - lo)))
        }
        "choice" => {
            arity!(1);
            if let Value::Array(arr) = &a[0] {
                let arr = arr.borrow();
                if arr.dims.len() != 1 { return err("CHOICE: nur 1D-Arrays".to_string()); }
                let n = arr.values.len();
                if n == 0 { return err("CHOICE: Array ist leer".to_string()); }
                Ok(arr.values[(next_rand() % n as u64) as usize].clone())
            } else { err("CHOICE erwartet ARRAY".to_string()) }
        }
        "shuffle" => {
            arity!(1);
            if let Value::Array(arr) = &a[0] {
                let mut arr = arr.borrow_mut();
                if arr.dims.len() != 1 { return err("SHUFFLE: nur 1D-Arrays".to_string()); }
                let n = arr.values.len();
                // Fisher-Yates (selbe Schleifenrichtung wie der Tree-Walker).
                for i in (1..n).rev() {
                    let j = (next_rand() % (i as u64 + 1)) as usize;
                    arr.values.swap(i, j);
                }
                Ok(Value::Nil)
            } else { err("SHUFFLE erwartet ARRAY".to_string()) }
        }
        "millis" => {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
            Ok(Value::Int(ms))
        }
        "timer" => {
            // Sekunden seit erstem TIMER-Aufruf als FLOAT (wie der Tree-Walker --
            // MILLIS bleibt ms-INT). Wichtig u.a. fuer FPS: elapsed >= 0.5 s.
            use std::sync::OnceLock;
            use std::time::Instant;
            static TIMER_START: OnceLock<Instant> = OnceLock::new();
            Ok(Value::Float(TIMER_START.get_or_init(Instant::now).elapsed().as_secs_f64()))
        }
        "time$" | "time" => {
            arity!(0);
            let (_, _, _, h, mi, s) = local_datetime();
            Ok(Value::str_rc(&format!("{:02}:{:02}:{:02}", h, mi, s)))
        }
        "date$" | "date" => {
            arity!(0);
            let (y, mo, d, _, _, _) = local_datetime();
            Ok(Value::str_rc(&format!("{:04}-{:02}-{:02}", y, mo, d)))
        }
        "range" => {
            for v in a {
                if !matches!(v, Value::Int(_)) {
                    return err("RANGE: Argument muss INTEGER sein".to_string());
                }
            }
            let (start, stop, step) = match a.len() {
                1 => (0, need_int(&a[0], "RANGE")?, 1),
                2 => (need_int(&a[0], "RANGE")?, need_int(&a[1], "RANGE")?, 1),
                3 => {
                    let st = need_int(&a[2], "RANGE")?;
                    if st == 0 { return err("RANGE: step darf nicht 0 sein".to_string()); }
                    (need_int(&a[0], "RANGE")?, need_int(&a[1], "RANGE")?, st)
                }
                _ => return err("RANGE: 1..3 Argumente".to_string()),
            };
            let mut out = Vec::new();
            let mut k = start;
            if step > 0 { while k < stop { out.push(Value::Int(k)); k += step; } }
            else { while k > stop { out.push(Value::Int(k)); k += step; } }
            Ok(Value::Tuple(Rc::new(out)))
        }
        "__comp_iter" => {
            arity!(1);
            match &a[0] {
                Value::Str(s) => Ok(Value::Tuple(Rc::new(s.chars().map(|c| Value::str_rc(&c.to_string())).collect()))),
                Value::Tuple(t) => Ok(Value::Tuple(t.clone())),
                Value::Array(arr) => {
                    let arr = arr.borrow();
                    if arr.dims.len() != 1 { return err("Comprehension: nur 1D-Arrays".to_string()); }
                    Ok(Value::Tuple(Rc::new(arr.values.clone())))
                }
                Value::Map(m) => {
                    let m = m.borrow();
                    Ok(Value::Tuple(Rc::new(m.entries.iter().map(|(k, _)| Value::str_rc(k)).collect())))
                }
                v => err(format!("Comprehension: nicht iterierbar ({})", v.type_name())),
            }
        }
        "__set_dedup" => {
            arity!(1);
            if let Value::Tuple(t) = &a[0] {
                let mut out: Vec<Value> = Vec::new();
                for x in t.iter() {
                    if !out.iter().any(|y| value_eq(x, y)) { out.push(x.clone()); }
                }
                Ok(Value::Tuple(Rc::new(out)))
            } else { err("__SET_DEDUP erwartet TUPLE".to_string()) }
        }
        "__dict_from_pairs" => {
            arity!(1);
            if let Value::Tuple(t) = &a[0] {
                let mut vt = "any".to_string();
                if let Some(Value::Tuple(p0)) = t.first() {
                    if p0.len() == 2 {
                        vt = match &p0[1] {
                            Value::Bool(_) => "boolean", Value::Int(_) => "integer",
                            Value::Float(_) => "float", Value::Str(_) => "string", _ => "any",
                        }.to_string();
                    }
                }
                let mut m = GbMap::new(vt);
                for p in t.iter() {
                    if let Value::Tuple(kv) = p {
                        if kv.len() == 2 {
                            let k = need_str(&kv[0], "Dict-Comprehension")?.to_string();
                            m.put(k, kv[1].clone());
                            continue;
                        }
                    }
                    return err("__DICT_FROM_PAIRS: erwartet 2-Tupel".to_string());
                }
                Ok(Value::Map(Rc::new(RefCell::new(m))))
            } else { err("__DICT_FROM_PAIRS erwartet TUPLE".to_string()) }
        }
        "len" => {
            arity!(1);
            match &a[0] {
                Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
                Value::Tuple(t) => Ok(Value::Int(t.len() as i64)),
                Value::Array(arr) => {
                    let arr = arr.borrow();
                    Ok(Value::Int(if arr.dims.is_empty() { 0 } else { arr.dims[0] }))
                }
                _ => err("LEN erwartet STRING, TUPLE oder ARRAY".to_string()),
            }
        }
        "dimsize" => {
            arity!(2);
            if let Value::Array(arr) = &a[0] {
                let arr = arr.borrow();
                let n = need_int(&a[1], "DIMSIZE")?;
                if n < 0 || n as usize >= arr.dims.len() {
                    return err(format!("DIMSIZE: Dimension {} ausserhalb", n));
                }
                Ok(Value::Int(arr.dims[n as usize]))
            } else { err("DIMSIZE erwartet ARRAY".to_string()) }
        }
        "dimcount" => {
            arity!(1);
            if let Value::Array(arr) = &a[0] { Ok(Value::Int(arr.borrow().dims.len() as i64)) }
            else { err("DIMCOUNT erwartet ARRAY".to_string()) }
        }
        "chr$" | "chr" => {
            arity!(1);
            let n = need_int(&a[0], "CHR$")?;
            let c = char::from_u32(n as u32).ok_or_else(|| format!("CHR$: ungueltiger Codepoint {}", n))?;
            Ok(Value::str_rc(&c.to_string()))
        }
        "asc" => {
            arity!(1);
            let s = need_str(&a[0], "ASC")?;
            let c = s.chars().next().ok_or("ASC erwartet nicht-leeren STRING")?;
            Ok(Value::Int(c as i64))
        }
        "sqr" | "sqrt" => {   // SQRT = BASIC-Alias fuer SQR
            arity!(1);
            let x = need_num(&a[0], "SQR")?;
            if x < 0.0 { return err("SQR von negativer Zahl".to_string()); }
            Ok(Value::Float(x.sqrt()))
        }
        "upper$" | "upper" => { arity!(1); Ok(Value::str_rc(&need_str(&a[0], "UPPER$")?.to_uppercase())) }
        "lower$" | "lower" => { arity!(1); Ok(Value::str_rc(&need_str(&a[0], "LOWER$")?.to_lowercase())) }
        "rgb" => {
            arity!(3);
            let (r, g, b) = (need_int(&a[0], "RGB")?, need_int(&a[1], "RGB")?, need_int(&a[2], "RGB")?);
            for v in [r, g, b] { if v < 0 || v > 255 { return err("RGB-Werte muessen 0..255 sein".to_string()); } }
            Ok(Value::Int((r << 16) | (g << 8) | b))
        }
        "sin" => { arity!(1); Ok(Value::Float(need_num(&a[0], "SIN")?.sin())) }
        "cos" => { arity!(1); Ok(Value::Float(need_num(&a[0], "COS")?.cos())) }
        "tan" => { arity!(1); Ok(Value::Float(need_num(&a[0], "TAN")?.tan())) }
        "atan" => { arity!(1); Ok(Value::Float(need_num(&a[0], "ATAN")?.atan())) }
        "atan2" => { arity!(2); Ok(Value::Float(need_num(&a[0], "ATAN2")?.atan2(need_num(&a[1], "ATAN2")?))) }
        "floor" => { arity!(1); Ok(Value::Int(need_num(&a[0], "FLOOR")?.floor() as i64)) }
        "ceil" => { arity!(1); Ok(Value::Int(need_num(&a[0], "CEIL")?.ceil() as i64)) }
        "round" => {
            if a.len() == 1 {
                Ok(Value::Int(round_half_even(need_num(&a[0], "ROUND")?)))
            } else if a.len() == 2 {
                let x = need_num(&a[0], "ROUND")?;
                let n = need_int(&a[1], "ROUND")?;
                if n < 0 { return err("ROUND: decimals muss >= 0 sein".to_string()); }
                // Auf n Nachkommastellen runden via Decimal-Formatierung
                // (Half-to-even) -- exakt wie Python f"{x:.{n}f}".
                let s = format!("{:.*}", n as usize, x);
                Ok(Value::Float(s.parse::<f64>().unwrap_or(x)))
            } else {
                err(format!("ROUND: erwartet 1 oder 2 Argument(e), erhalten {}", a.len()))
            }
        }
        "log" => {
            let x = need_num(&a[0], "LOG")?;
            if x <= 0.0 { return err("LOG: Argument muss > 0 sein".to_string()); }
            if a.len() == 2 { Ok(Value::Float(x.log(need_num(&a[1], "LOG")?))) }
            else { Ok(Value::Float(x.ln())) }
        }
        "exp" => { arity!(1); Ok(Value::Float(need_num(&a[0], "EXP")?.exp())) }
        "pow" => {
            arity!(2);
            // base ** exponent (Python): int**int>=0 -> int, sonst float
            match (&a[0], &a[1]) {
                (Value::Int(b), Value::Int(e)) if *e >= 0 => Ok(Value::Int(b.pow(*e as u32))),
                _ => Ok(Value::Float(need_num(&a[0], "POW")?.powf(need_num(&a[1], "POW")?))),
            }
        }
        "min" | "max" => {
            if a.is_empty() { return err(format!("{}: mind. 1 Argument", name.to_uppercase())); }
            for v in a { if !is_num(v) { return err(format!("{} erwartet Zahl", name.to_uppercase())); } }
            let want_min = name == "min";
            let mut best = a[0].clone();
            for v in &a[1..] {
                let take = if want_min { as_f64(v) < as_f64(&best) } else { as_f64(v) > as_f64(&best) };
                if take { best = v.clone(); }
            }
            Ok(best)
        }
        "clamp" => {
            arity!(3);
            let (v, lo, hi) = (as_f64(&a[0]), as_f64(&a[1]), as_f64(&a[2]));
            for x in &a[..3] { if !is_num(x) { return err("CLAMP erwartet Zahlen".to_string()); } }
            if v < lo { Ok(a[1].clone()) } else if v > hi { Ok(a[2].clone()) } else { Ok(a[0].clone()) }
        }
        "sign" | "sgn" => {   // SGN = BASIC-Alias fuer SIGN
            arity!(1);
            let v = need_num(&a[0], "SIGN")?;
            Ok(Value::Int(if v > 0.0 { 1 } else if v < 0.0 { -1 } else { 0 }))
        }
        "asin" => {
            arity!(1);
            let x = need_num(&a[0], "ASIN")?;
            if x < -1.0 || x > 1.0 { return err("ASIN: Argument muss in [-1, 1] liegen".to_string()); }
            Ok(Value::Float(x.asin()))
        }
        "acos" => {
            arity!(1);
            let x = need_num(&a[0], "ACOS")?;
            if x < -1.0 || x > 1.0 { return err("ACOS: Argument muss in [-1, 1] liegen".to_string()); }
            Ok(Value::Float(x.acos()))
        }
        "hypot" => { arity!(2); Ok(Value::Float(need_num(&a[0], "HYPOT")?.hypot(need_num(&a[1], "HYPOT")?))) }
        "deg" => { arity!(1); Ok(Value::Float(need_num(&a[0], "DEG")? * 180.0 / std::f64::consts::PI)) }
        "rad" => { arity!(1); Ok(Value::Float(need_num(&a[0], "RAD")? * std::f64::consts::PI / 180.0)) }
        "lerp" => {
            arity!(3);
            let (av, bv, t) = (need_num(&a[0], "LERP")?, need_num(&a[1], "LERP")?, need_num(&a[2], "LERP")?);
            Ok(Value::Float(av + (bv - av) * t))
        }
        "remap" => {
            arity!(5);
            let v = need_num(&a[0], "REMAP")?;
            let (in_lo, in_hi) = (need_num(&a[1], "REMAP")?, need_num(&a[2], "REMAP")?);
            let (out_lo, out_hi) = (need_num(&a[3], "REMAP")?, need_num(&a[4], "REMAP")?);
            if in_hi == in_lo { return err("REMAP: in_lo und in_hi duerfen nicht gleich sein".to_string()); }
            Ok(Value::Float(out_lo + (v - in_lo) * (out_hi - out_lo) / (in_hi - in_lo)))
        }
        "frac" => {
            arity!(1);
            let x = need_num(&a[0], "FRAC")?;
            Ok(Value::Float(x - x.trunc()))
        }
        "red" => { arity!(1); Ok(Value::Int((need_int(&a[0], "RED")? >> 16) & 0xFF)) }
        "green" => { arity!(1); Ok(Value::Int((need_int(&a[0], "GREEN")? >> 8) & 0xFF)) }
        "blue" => { arity!(1); Ok(Value::Int(need_int(&a[0], "BLUE")? & 0xFF)) }
        "color_lerp" => {
            arity!(3);
            let c1 = need_int(&a[0], "COLOR_LERP")?;
            let c2 = need_int(&a[1], "COLOR_LERP")?;
            let mut t = need_num(&a[2], "COLOR_LERP")?;
            if t < 0.0 { t = 0.0; } else if t > 1.0 { t = 1.0; }
            let lerp_ch = |x1: i64, x2: i64| -> i64 {
                ((x1 as f64) + ((x2 - x1) as f64) * t + 0.5).floor() as i64
            };
            let r = lerp_ch((c1 >> 16) & 0xFF, (c2 >> 16) & 0xFF);
            let g = lerp_ch((c1 >> 8) & 0xFF, (c2 >> 8) & 0xFF);
            let b = lerp_ch(c1 & 0xFF, c2 & 0xFF);
            Ok(Value::Int((r << 16) | (g << 8) | b))
        }
        "hsv" => {
            arity!(3);
            let h = need_num(&a[0], "HSV")?.rem_euclid(360.0);
            let s = need_num(&a[1], "HSV")?.clamp(0.0, 1.0);
            let v = need_num(&a[2], "HSV")?.clamp(0.0, 1.0);
            let c = v * s;
            let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
            let m = v - c;
            let (r1, g1, b1) = if h < 60.0 { (c, x, 0.0) }
                else if h < 120.0 { (x, c, 0.0) }
                else if h < 180.0 { (0.0, c, x) }
                else if h < 240.0 { (0.0, x, c) }
                else if h < 300.0 { (x, 0.0, c) }
                else { (c, 0.0, x) };
            let u8c = |val: f64| -> i64 { ((val + m) * 255.0 + 0.5).floor() as i64 };
            Ok(Value::Int((u8c(r1) << 16) | (u8c(g1) << 8) | u8c(b1)))
        }
        "padl$" | "padl" | "padr$" | "padr" => {
            let s = need_str(&a[0], "PAD")?;
            let n = need_int(&a[1], "PAD")? as usize;
            let fill = if a.len() == 3 {
                let f = need_str(&a[2], "PAD")?;
                f.chars().next().unwrap_or(' ')
            } else { ' ' };
            let count = s.chars().count();
            if count >= n { return Ok(Value::str_rc(s)); }
            let pad: String = std::iter::repeat(fill).take(n - count).collect();
            let left = name.starts_with("padl");
            Ok(Value::str_rc(&if left { format!("{}{}", pad, s) } else { format!("{}{}", s, pad) }))
        }
        "format$" => { arity!(2); format_one(need_str(&a[1], "FORMAT$")?, &a[0]) }
        "repeat$" => {
            arity!(2);
            let s = need_str(&a[0], "REPEAT$")?;
            let n = need_int(&a[1], "REPEAT$")?.max(0) as usize;
            Ok(Value::str_rc(&s.repeat(n)))
        }
        "space$" | "space" => { arity!(1); Ok(Value::str_rc(&" ".repeat(need_int(&a[0], "SPACE$")?.max(0) as usize))) }
        "hex$" | "hex" => {
            arity!(1);
            let n = need_int(&a[0], "HEX$")?;
            Ok(Value::str_rc(&if n < 0 { format!("-{:X}", -n) } else { format!("{:X}", n) }))
        }
        // --- Map ---
        "mapput" => {
            arity!(3);
            if let Value::Map(m) = &a[0] {
                let k = need_str(&a[1], "MAPPUT")?.to_string();
                let v = coerce_map_value(&a[2], &m.borrow().value_type)?;
                m.borrow_mut().put(k, v);
                Ok(Value::Nil)
            } else { err("MAPPUT erwartet MAP".to_string()) }
        }
        "mapget" => {
            arity!(2);
            if let Value::Map(m) = &a[0] {
                let k = need_str(&a[1], "MAPGET")?;
                m.borrow().get(k).cloned().ok_or_else(|| format!("MAPGET: Schluessel '{}' nicht in Map", k))
            } else { err("MAPGET erwartet MAP".to_string()) }
        }
        "mapgetor" => {
            arity!(3);
            if let Value::Map(m) = &a[0] {
                let k = need_str(&a[1], "MAPGETOR")?;
                Ok(m.borrow().get(k).cloned().unwrap_or_else(|| a[2].clone()))
            } else { err("MAPGETOR erwartet MAP".to_string()) }
        }
        "maphas" => {
            arity!(2);
            if let Value::Map(m) = &a[0] {
                Ok(Value::Bool(m.borrow().get(need_str(&a[1], "MAPHAS")?).is_some()))
            } else { err("MAPHAS erwartet MAP".to_string()) }
        }
        "mapremove" => {
            arity!(2);
            if let Value::Map(m) = &a[0] {
                Ok(Value::Bool(m.borrow_mut().remove(need_str(&a[1], "MAPREMOVE")?)))
            } else { err("MAPREMOVE erwartet MAP".to_string()) }
        }
        "mapsize" => {
            arity!(1);
            if let Value::Map(m) = &a[0] { Ok(Value::Int(m.borrow().entries.len() as i64)) }
            else { err("MAPSIZE erwartet MAP".to_string()) }
        }
        "mapkeys" => {
            arity!(1);
            if let Value::Map(m) = &a[0] {
                Ok(new_str_array(m.borrow().entries.iter().map(|(k, _)| k.clone()).collect()))
            } else { err("MAPKEYS erwartet MAP".to_string()) }
        }
        "mapclear" => {
            arity!(1);
            if let Value::Map(m) = &a[0] { m.borrow_mut().entries.clear(); Ok(Value::Nil) }
            else { err("MAPCLEAR erwartet MAP".to_string()) }
        }
        "mapvalues" => {
            arity!(1);
            if let Value::Map(m) = &a[0] {
                let m = m.borrow();
                let vals: Vec<Value> = m.entries.iter().map(|(_, v)| v.clone()).collect();
                let n = vals.len() as i64;
                let mut arr = GbArray::new(m.value_type.clone(), vec![n], || type_default(&m.value_type));
                for (i, v) in vals.into_iter().enumerate() { arr.values[i] = v; }
                Ok(Value::Array(Rc::new(RefCell::new(arr))))
            } else { err("MAPVALUES erwartet MAP".to_string()) }
        }
        "mapitems" => {
            arity!(1);
            if let Value::Map(m) = &a[0] {
                let m = m.borrow();
                let items: Vec<Value> = m.entries.iter()
                    .map(|(k, v)| Value::Tuple(Rc::new(vec![Value::str_rc(k), v.clone()]))).collect();
                let n = items.len() as i64;
                let mut arr = GbArray::new("tuple".to_string(), vec![n], || Value::Tuple(Rc::new(vec![])));
                for (i, v) in items.into_iter().enumerate() { arr.values[i] = v; }
                Ok(Value::Array(Rc::new(RefCell::new(arr))))
            } else { err("MAPITEMS erwartet MAP".to_string()) }
        }
        // --- Array ---
        "sort" => {
            // SORT(arr) aufsteigend, SORT(arr, descending?) BOOL-Flag.
            // SORT(arr, comparator-FUNCREF) wird vorher in vm.rs abgefangen
            // (braucht die VM, um die User-Funktion zu rufen).
            if a.is_empty() || a.len() > 2 {
                return err(format!("SORT: erwartet 1 oder 2 Argument(e), erhalten {}", a.len()));
            }
            let desc = match a.get(1) {
                None => false,
                Some(Value::Bool(b)) => *b,
                Some(_) => return err("SORT: 2. Argument muss BOOLEAN (absteigend) oder FUNCREF (Comparator) sein".to_string()),
            };
            if let Value::Array(arr) = &a[0] {
                let mut arr = arr.borrow_mut();
                if !matches!(arr.element_type.as_str(), "integer" | "float" | "string") {
                    return err("SORT: nur ARRAY OF INTEGER/FLOAT/STRING".to_string());
                }
                if arr.dims.len() != 1 { return err("SORT: nur 1D-Arrays".to_string()); }
                arr.values.sort_by(|x, y| cmp_value(x, y));
                if desc { arr.values.reverse(); }
                Ok(Value::Nil)
            } else { err("SORT erwartet ARRAY".to_string()) }
        }
        "reverse" => {
            arity!(1);
            if let Value::Array(arr) = &a[0] {
                let mut arr = arr.borrow_mut();
                if arr.dims.len() != 1 { return err("REVERSE: nur 1D-Arrays".to_string()); }
                arr.values.reverse();
                Ok(Value::Nil)
            } else { err("REVERSE erwartet ARRAY".to_string()) }
        }
        "array_indexof" => {
            arity!(2);
            if let Value::Array(arr) = &a[0] {
                let arr = arr.borrow();
                if arr.dims.len() != 1 { return err("ARRAY_INDEXOF: nur 1D".to_string()); }
                for (i, v) in arr.values.iter().enumerate() {
                    if value_eq(v, &a[1]) { return Ok(Value::Int(i as i64)); }
                }
                Ok(Value::Int(-1))
            } else { err("ARRAY_INDEXOF erwartet ARRAY".to_string()) }
        }
        // --- Array-Aggregate (1D) ---
        "array_sum" => {
            arity!(1);
            if let Value::Array(arr) = &a[0] {
                let arr = arr.borrow();
                if arr.dims.len() != 1 { return err("ARRAY_SUM: nur 1D-Arrays".to_string()); }
                match arr.element_type.as_str() {
                    "integer" => {
                        let mut s: i64 = 0;
                        for v in &arr.values {
                            match v { Value::Int(i) => s = s.wrapping_add(*i), _ => return err("ARRAY_SUM: nicht-INTEGER-Element".to_string()) }
                        }
                        Ok(Value::Int(s))
                    }
                    "float" => {
                        let mut s = 0.0f64;
                        for v in &arr.values { s += as_f64(v); }
                        Ok(Value::Float(s))
                    }
                    _ => err("ARRAY_SUM: nur ARRAY OF INTEGER/FLOAT".to_string()),
                }
            } else { err("ARRAY_SUM erwartet ARRAY".to_string()) }
        }
        "array_avg" => {
            arity!(1);
            if let Value::Array(arr) = &a[0] {
                let arr = arr.borrow();
                if arr.dims.len() != 1 { return err("ARRAY_AVG: nur 1D-Arrays".to_string()); }
                if !matches!(arr.element_type.as_str(), "integer" | "float") { return err("ARRAY_AVG: nur ARRAY OF INTEGER/FLOAT".to_string()); }
                let n = arr.values.len();
                if n == 0 { return err("ARRAY_AVG: Array ist leer".to_string()); }
                let mut s = 0.0f64;
                for v in &arr.values { s += as_f64(v); }
                Ok(Value::Float(s / n as f64))
            } else { err("ARRAY_AVG erwartet ARRAY".to_string()) }
        }
        "array_min" | "array_max" => {
            arity!(1);
            if let Value::Array(arr) = &a[0] {
                let arr = arr.borrow();
                let fn_ = name.to_uppercase();
                if arr.dims.len() != 1 { return err(format!("{}: nur 1D-Arrays", fn_)); }
                if !matches!(arr.element_type.as_str(), "integer" | "float") { return err(format!("{}: nur ARRAY OF INTEGER/FLOAT", fn_)); }
                if arr.values.is_empty() { return err(format!("{}: Array ist leer", fn_)); }
                let want_min = name == "array_min";
                let mut best = arr.values[0].clone();
                for v in &arr.values[1..] {
                    let take = if want_min { as_f64(v) < as_f64(&best) } else { as_f64(v) > as_f64(&best) };
                    if take { best = v.clone(); }
                }
                Ok(best)
            } else { err(format!("{} erwartet ARRAY", name.to_uppercase())) }
        }
        "array_fill" => {
            arity!(2);
            if let Value::Array(arr) = &a[0] {
                let et = arr.borrow().element_type.clone();
                let v = coerce_elem(a[1].clone(), &et, "ARRAY_FILL")?;
                let mut arr = arr.borrow_mut();
                for slot in arr.values.iter_mut() { *slot = v.clone(); }
                Ok(Value::Nil)
            } else { err("ARRAY_FILL erwartet ARRAY".to_string()) }
        }
        "array_copy" => {
            arity!(1);
            if let Value::Array(arr) = &a[0] {
                let arr = arr.borrow();
                let new = GbArray {
                    element_type: arr.element_type.clone(),
                    dims: arr.dims.clone(),
                    strides: arr.strides.clone(),
                    values: arr.values.clone(),
                };
                Ok(Value::Array(Rc::new(RefCell::new(new))))
            } else { err("ARRAY_COPY erwartet ARRAY".to_string()) }
        }
        // --- Dynamische 1D-Arrays (wachsen/schrumpfen) ---
        // Mutieren das Array IN PLACE: values + dims=[len] + strides=[1].
        "array_push" => {
            arity!(2);
            if let Value::Array(arr) = &a[0] {
                let et = { let b = arr.borrow(); if b.dims.len() != 1 { return err("ARRAY_PUSH: nur 1D-Arrays".to_string()); } b.element_type.clone() };
                let v = coerce_elem(a[1].clone(), &et, "ARRAY_PUSH")?;
                let mut b = arr.borrow_mut();
                b.values.push(v);
                let n = b.values.len() as i64;
                b.dims = vec![n];
                b.strides = vec![1];
                Ok(Value::Int(n))
            } else { err("ARRAY_PUSH erwartet ARRAY".to_string()) }
        }
        "array_pop" => {
            arity!(1);
            if let Value::Array(arr) = &a[0] {
                let mut b = arr.borrow_mut();
                if b.dims.len() != 1 { return err("ARRAY_POP: nur 1D-Arrays".to_string()); }
                match b.values.pop() {
                    Some(v) => { let n = b.values.len() as i64; b.dims = vec![n]; b.strides = vec![1]; Ok(v) }
                    None => err("ARRAY_POP: Array ist leer".to_string()),
                }
            } else { err("ARRAY_POP erwartet ARRAY".to_string()) }
        }
        "array_insert" => {
            arity!(3);
            if let Value::Array(arr) = &a[0] {
                let et = { let b = arr.borrow(); if b.dims.len() != 1 { return err("ARRAY_INSERT: nur 1D-Arrays".to_string()); } b.element_type.clone() };
                let idx = need_int(&a[1], "ARRAY_INSERT")?;
                let v = coerce_elem(a[2].clone(), &et, "ARRAY_INSERT")?;
                let mut b = arr.borrow_mut();
                let n = b.values.len() as i64;
                if idx < 0 || idx > n { return err(format!("ARRAY_INSERT: Index {} ausserhalb [0..{}]", idx, n)); }
                b.values.insert(idx as usize, v);
                let nn = b.values.len() as i64;
                b.dims = vec![nn];
                b.strides = vec![1];
                Ok(Value::Int(nn))
            } else { err("ARRAY_INSERT erwartet ARRAY".to_string()) }
        }
        "array_remove_at" => {
            arity!(2);
            if let Value::Array(arr) = &a[0] {
                { let b = arr.borrow(); if b.dims.len() != 1 { return err("ARRAY_REMOVE_AT: nur 1D-Arrays".to_string()); } }
                let idx = need_int(&a[1], "ARRAY_REMOVE_AT")?;
                let mut b = arr.borrow_mut();
                let n = b.values.len() as i64;
                if idx < 0 || idx >= n { return err(format!("ARRAY_REMOVE_AT: Index {} ausserhalb [0..{}]", idx, n - 1)); }
                let v = b.values.remove(idx as usize);
                let nn = b.values.len() as i64;
                b.dims = vec![nn];
                b.strides = vec![1];
                Ok(v)
            } else { err("ARRAY_REMOVE_AT erwartet ARRAY".to_string()) }
        }
        "redim" => {
            // REDIM(arr, neue_laenge): 1D-Array vergroessern (mit Typ-Default
            // auffuellen) oder verkleinern (abschneiden), Bestand bleibt erhalten.
            arity!(2);
            if let Value::Array(arr) = &a[0] {
                let et = { let b = arr.borrow(); if b.dims.len() != 1 { return err("REDIM: nur 1D-Arrays".to_string()); } b.element_type.clone() };
                let new_len = need_int(&a[1], "REDIM")?;
                if new_len < 0 { return err("REDIM: Groesse darf nicht negativ sein".to_string()); }
                let mut b = arr.borrow_mut();
                let cur = b.values.len() as i64;
                if new_len < cur {
                    b.values.truncate(new_len as usize);
                } else {
                    for _ in cur..new_len { b.values.push(type_default(&et)); }
                }
                b.dims = vec![new_len];
                b.strides = vec![1];
                Ok(Value::Nil)
            } else { err("REDIM erwartet ARRAY".to_string()) }
        }
        "collides" => {
            arity!(8);
            let mut n = [0f64; 8];
            for i in 0..8 { n[i] = need_num(&a[i], "COLLIDES")?; }
            Ok(Value::Bool(n[0] < n[4] + n[6] && n[0] + n[2] > n[4] && n[1] < n[5] + n[7] && n[1] + n[3] > n[5]))
        }
        // --- Strings ---
        "left$" | "left" => {
            arity!(2);
            let s = need_str(&a[0], "LEFT$")?;
            let n = need_int(&a[1], "LEFT$")?.max(0) as usize;
            Ok(Value::str_rc(&s.chars().take(n).collect::<String>()))
        }
        "right$" | "right" => {
            arity!(2);
            let s = need_str(&a[0], "RIGHT$")?;
            let n = need_int(&a[1], "RIGHT$")?;
            if n <= 0 { return Ok(Value::str_rc("")); }
            let chars: Vec<char> = s.chars().collect();
            let n = (n as usize).min(chars.len());
            Ok(Value::str_rc(&chars[chars.len() - n..].iter().collect::<String>()))
        }
        "mid$" | "mid" => {
            let s = need_str(&a[0], "MID$")?;
            let chars: Vec<char> = s.chars().collect();
            let start = need_int(&a[1], "MID$")?.max(0) as usize;
            let start = start.min(chars.len());
            if a.len() == 3 {
                let cnt = need_int(&a[2], "MID$")?.max(0) as usize;
                let end = (start + cnt).min(chars.len());
                Ok(Value::str_rc(&chars[start..end].iter().collect::<String>()))
            } else {
                Ok(Value::str_rc(&chars[start..].iter().collect::<String>()))
            }
        }
        "instr" => {
            let hay = need_str(&a[0], "INSTR")?;
            let needle = need_str(&a[1], "INSTR")?;
            let start = if a.len() == 3 { need_int(&a[2], "INSTR")?.max(0) as usize } else { 0 };
            // 0-basiert in *Zeichen*. Python str.find ist byte/char-gleich fuer
            // ASCII; fuer Bit-Identitaet auf Zeichenebene rechnen.
            let hchars: Vec<char> = hay.chars().collect();
            let nchars: Vec<char> = needle.chars().collect();
            if nchars.is_empty() { return Ok(Value::Int(start.min(hchars.len()) as i64)); }
            if start > hchars.len() { return Ok(Value::Int(-1)); }
            let mut i = start;
            while i + nchars.len() <= hchars.len() {
                if hchars[i..i + nchars.len()] == nchars[..] { return Ok(Value::Int(i as i64)); }
                i += 1;
            }
            Ok(Value::Int(-1))
        }
        "replace$" | "replace" => {
            arity!(3);
            let s = need_str(&a[0], "REPLACE$")?;
            let old = need_str(&a[1], "REPLACE$")?;
            let new = need_str(&a[2], "REPLACE$")?;
            Ok(Value::str_rc(&if old.is_empty() { s.to_string() } else { s.replace(old, new) }))
        }
        "trim$" | "trim" => { arity!(1); Ok(Value::str_rc(need_str(&a[0], "TRIM$")?.trim())) }
        "split$" | "split" => {
            arity!(2);
            let s = need_str(&a[0], "SPLIT$")?;
            let delim = need_str(&a[1], "SPLIT$")?;
            let parts: Vec<String> = if delim.is_empty() {
                s.chars().map(|c| c.to_string()).collect()
            } else {
                s.split(delim).map(|x| x.to_string()).collect()
            };
            Ok(new_str_array(parts))
        }
        "join$" | "join" => {
            arity!(2);
            if let Value::Array(arr) = &a[0] {
                let arr = arr.borrow();
                if arr.element_type != "string" { return err("JOIN$: ARRAY OF STRING noetig".to_string()); }
                let delim = need_str(&a[1], "JOIN$")?;
                let parts: Vec<String> = arr.values.iter().map(|v| match v { Value::Str(s) => s.to_string(), o => o.fmt() }).collect();
                Ok(Value::str_rc(&parts.join(delim)))
            } else { err("JOIN$ erwartet ARRAY OF STRING".to_string()) }
        }
        // --- String-Erweiterungen (WP3) ---
        "ltrim$" | "ltrim" => { arity!(1); Ok(Value::str_rc(need_str(&a[0], "LTRIM$")?.trim_start())) }
        "rtrim$" | "rtrim" => { arity!(1); Ok(Value::str_rc(need_str(&a[0], "RTRIM$")?.trim_end())) }
        "reverse$" => { arity!(1); Ok(Value::str_rc(&need_str(&a[0], "REVERSE$")?.chars().rev().collect::<String>())) }
        "startswith" => { arity!(2); Ok(Value::Bool(need_str(&a[0], "STARTSWITH")?.starts_with(need_str(&a[1], "STARTSWITH")?))) }
        "endswith" => { arity!(2); Ok(Value::Bool(need_str(&a[0], "ENDSWITH")?.ends_with(need_str(&a[1], "ENDSWITH")?))) }
        "contains" => { arity!(2); Ok(Value::Bool(need_str(&a[0], "CONTAINS")?.contains(need_str(&a[1], "CONTAINS")?))) }
        "bin$" | "bin" => {
            arity!(1);
            let n = need_int(&a[0], "BIN$")?;
            Ok(Value::str_rc(&if n < 0 { format!("-{:b}", (n as i128).unsigned_abs()) } else { format!("{:b}", n) }))
        }
        "oct$" | "oct" => {
            arity!(1);
            let n = need_int(&a[0], "OCT$")?;
            Ok(Value::str_rc(&if n < 0 { format!("-{:o}", (n as i128).unsigned_abs()) } else { format!("{:o}", n) }))
        }
        "isnumeric" => { arity!(1); Ok(Value::Bool(parse_number(need_str(&a[0], "ISNUMERIC")?).is_some())) }
        "tryval" => {
            arity!(2);
            let s = need_str(&a[0], "TRYVAL")?;
            if !is_num(&a[1]) { return err("TRYVAL: default muss eine Zahl sein".to_string()); }
            Ok(parse_number(s).unwrap_or_else(|| a[1].clone()))
        }
        // ===== Modul: vec2 =====
        "vec2_new" => { arity!(2); Ok(Value::Vec2(need_num(&a[0], "VEC2_NEW")?, need_num(&a[1], "VEC2_NEW")?)) }
        "vec2_zero" => { arity!(0); Ok(Value::Vec2(0.0, 0.0)) }
        "vec2_x" => { arity!(1); Ok(Value::Float(vec2(&a[0], "VEC2_X")?.0)) }
        "vec2_y" => { arity!(1); Ok(Value::Float(vec2(&a[0], "VEC2_Y")?.1)) }
        "vec2_length" => { arity!(1); let (x, y) = vec2(&a[0], "VEC2_LENGTH")?; Ok(Value::Float(x.hypot(y))) }
        "vec2_length_sq" => { arity!(1); let (x, y) = vec2(&a[0], "VEC2_LENGTH_SQ")?; Ok(Value::Float(x * x + y * y)) }
        "vec2_normalize" => {
            arity!(1); let (x, y) = vec2(&a[0], "VEC2_NORMALIZE")?;
            let l = x.hypot(y);
            Ok(if l == 0.0 { Value::Vec2(0.0, 0.0) } else { Value::Vec2(x / l, y / l) })
        }
        "vec2_dot" => { arity!(2); let (ax, ay) = vec2(&a[0], "VEC2_DOT")?; let (bx, by) = vec2(&a[1], "VEC2_DOT")?; Ok(Value::Float(ax * bx + ay * by)) }
        "vec2_cross" => { arity!(2); let (ax, ay) = vec2(&a[0], "VEC2_CROSS")?; let (bx, by) = vec2(&a[1], "VEC2_CROSS")?; Ok(Value::Float(ax * by - ay * bx)) }
        "vec2_distance" => { arity!(2); let (ax, ay) = vec2(&a[0], "VEC2_DISTANCE")?; let (bx, by) = vec2(&a[1], "VEC2_DISTANCE")?; Ok(Value::Float((ax - bx).hypot(ay - by))) }
        "vec2_lerp" => {
            arity!(3); let (ax, ay) = vec2(&a[0], "VEC2_LERP")?; let (bx, by) = vec2(&a[1], "VEC2_LERP")?; let t = need_num(&a[2], "VEC2_LERP")?;
            Ok(Value::Vec2(ax + (bx - ax) * t, ay + (by - ay) * t))
        }
        "vec2_perp" => { arity!(1); let (x, y) = vec2(&a[0], "VEC2_PERP")?; Ok(Value::Vec2(-y, x)) }
        "vec2_reflect" => {
            arity!(2); let (vx, vy) = vec2(&a[0], "VEC2_REFLECT")?; let (nx, ny) = vec2(&a[1], "VEC2_REFLECT")?;
            let nlen2 = nx * nx + ny * ny;
            if nlen2 == 0.0 { return Ok(Value::Vec2(vx, vy)); }
            let dot = (vx * nx + vy * ny) / nlen2;
            Ok(Value::Vec2(vx - 2.0 * dot * nx, vy - 2.0 * dot * ny))
        }
        "vec2_angle" => { arity!(1); let (x, y) = vec2(&a[0], "VEC2_ANGLE")?; Ok(Value::Float(y.atan2(x))) }
        "vec2_from_angle" => { arity!(2); let an = need_num(&a[0], "VEC2_FROM_ANGLE")?; let len = need_num(&a[1], "VEC2_FROM_ANGLE")?; Ok(Value::Vec2(an.cos() * len, an.sin() * len)) }

        // ===== Modul: curves =====
        "curve_lerp" => { arity!(3); let (av, bv, t) = (need_num(&a[0], "CURVE_LERP")?, need_num(&a[1], "CURVE_LERP")?, need_num(&a[2], "CURVE_LERP")?); Ok(Value::Float(av + (bv - av) * t)) }
        "curve_smoothstep" => {
            arity!(3); let (e0, e1, x) = (need_num(&a[0], "CURVE_SMOOTHSTEP")?, need_num(&a[1], "CURVE_SMOOTHSTEP")?, need_num(&a[2], "CURVE_SMOOTHSTEP")?);
            if e1 == e0 { return Ok(Value::Float(0.0)); }
            let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
            Ok(Value::Float(t * t * (3.0 - 2.0 * t)))
        }
        "curve_smootherstep" => {
            arity!(3); let (e0, e1, x) = (need_num(&a[0], "CURVE_SMOOTHERSTEP")?, need_num(&a[1], "CURVE_SMOOTHERSTEP")?, need_num(&a[2], "CURVE_SMOOTHERSTEP")?);
            if e1 == e0 { return Ok(Value::Float(0.0)); }
            let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
            Ok(Value::Float(t * t * t * (t * (t * 6.0 - 15.0) + 10.0)))
        }
        "curve_bezier" => {
            arity!(5);
            Ok(Value::Float(bezier_1d(need_num(&a[0], "CURVE_BEZIER")?, need_num(&a[1], "CURVE_BEZIER")?, need_num(&a[2], "CURVE_BEZIER")?, need_num(&a[3], "CURVE_BEZIER")?, need_num(&a[4], "CURVE_BEZIER")?)))
        }
        "curve_bezier2" => {
            arity!(9); let t = need_num(&a[0], "CURVE_BEZIER2")?;
            let x = bezier_1d(t, need_num(&a[1], "B")?, need_num(&a[3], "B")?, need_num(&a[5], "B")?, need_num(&a[7], "B")?);
            let y = bezier_1d(t, need_num(&a[2], "B")?, need_num(&a[4], "B")?, need_num(&a[6], "B")?, need_num(&a[8], "B")?);
            Ok(Value::Tuple(Rc::new(vec![Value::Float(x), Value::Float(y)])))
        }
        "curve_catmull" => {
            arity!(5);
            Ok(Value::Float(catmull_1d(need_num(&a[0], "CURVE_CATMULL")?, need_num(&a[1], "CURVE_CATMULL")?, need_num(&a[2], "CURVE_CATMULL")?, need_num(&a[3], "CURVE_CATMULL")?, need_num(&a[4], "CURVE_CATMULL")?)))
        }
        "curve_catmull2" => {
            arity!(9); let t = need_num(&a[0], "CURVE_CATMULL2")?;
            let x = catmull_1d(t, need_num(&a[1], "C")?, need_num(&a[3], "C")?, need_num(&a[5], "C")?, need_num(&a[7], "C")?);
            let y = catmull_1d(t, need_num(&a[2], "C")?, need_num(&a[4], "C")?, need_num(&a[6], "C")?, need_num(&a[8], "C")?);
            Ok(Value::Tuple(Rc::new(vec![Value::Float(x), Value::Float(y)])))
        }
        "curve_hermite" => {
            arity!(5); let t = need_num(&a[0], "CURVE_HERMITE")?; let p0 = need_num(&a[1], "CURVE_HERMITE")?; let p1 = need_num(&a[2], "CURVE_HERMITE")?; let m0 = need_num(&a[3], "CURVE_HERMITE")?; let m1 = need_num(&a[4], "CURVE_HERMITE")?;
            let t2 = t * t; let t3 = t2 * t;
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0; let h10 = t3 - 2.0 * t2 + t; let h01 = -2.0 * t3 + 3.0 * t2; let h11 = t3 - t2;
            Ok(Value::Float(h00 * p0 + h10 * m0 + h01 * p1 + h11 * m1))
        }

        // ===== Modul: physics (pure) =====
        "physics_box_box" => { arity!(8); let v = nums(a, "PHYSICS_BOX_BOX")?; Ok(Value::Bool(v[0] < v[4] + v[6] && v[0] + v[2] > v[4] && v[1] < v[5] + v[7] && v[1] + v[3] > v[5])) }
        "physics_circle_circle" => { arity!(6); let v = nums(a, "PHYSICS_CIRCLE_CIRCLE")?; let dx = v[0] - v[3]; let dy = v[1] - v[4]; let rs = v[2] + v[5]; Ok(Value::Bool(dx * dx + dy * dy < rs * rs)) }
        "physics_box_circle" => { arity!(7); let v = nums(a, "PHYSICS_BOX_CIRCLE")?; let nx = v[0].max(v[4].min(v[0] + v[2])); let ny = v[1].max(v[5].min(v[1] + v[3])); let dx = v[4] - nx; let dy = v[5] - ny; Ok(Value::Bool(dx * dx + dy * dy < v[6] * v[6])) }
        "physics_point_box" => { arity!(6); let v = nums(a, "PHYSICS_POINT_BOX")?; Ok(Value::Bool(v[2] <= v[0] && v[0] < v[2] + v[4] && v[3] <= v[1] && v[1] < v[3] + v[5])) }
        "physics_point_circle" => { arity!(5); let v = nums(a, "PHYSICS_POINT_CIRCLE")?; let dx = v[0] - v[2]; let dy = v[1] - v[3]; Ok(Value::Bool(dx * dx + dy * dy < v[4] * v[4])) }
        "physics_distance" => { arity!(4); let v = nums(a, "PHYSICS_DISTANCE")?; Ok(Value::Float((v[2] - v[0]).hypot(v[3] - v[1]))) }
        "physics_distance2" => { arity!(4); let v = nums(a, "PHYSICS_DISTANCE2")?; let dx = v[2] - v[0]; let dy = v[3] - v[1]; Ok(Value::Float(dx * dx + dy * dy)) }
        "physics_length" => { arity!(2); let v = nums(a, "PHYSICS_LENGTH")?; Ok(Value::Float(v[0].hypot(v[1]))) }
        "physics_norm_x" => { arity!(2); let v = nums(a, "PHYSICS_NORM_X")?; let l = v[0].hypot(v[1]); Ok(Value::Float(if l > 0.0 { v[0] / l } else { 0.0 })) }
        "physics_norm_y" => { arity!(2); let v = nums(a, "PHYSICS_NORM_Y")?; let l = v[0].hypot(v[1]); Ok(Value::Float(if l > 0.0 { v[1] / l } else { 0.0 })) }
        "physics_reflect_x" => { arity!(4); let v = nums(a, "PHYSICS_REFLECT_X")?; Ok(Value::Float(reflect(v[0], v[1], v[2], v[3], true))) }
        "physics_reflect_y" => { arity!(4); let v = nums(a, "PHYSICS_REFLECT_Y")?; Ok(Value::Float(reflect(v[0], v[1], v[2], v[3], false))) }
        "physics_ray_box" => { arity!(8); let v = nums(a, "PHYSICS_RAY_BOX")?; Ok(Value::Float(ray_box(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]))) }
        "physics_ray_circle" => { arity!(7); let v = nums(a, "PHYSICS_RAY_CIRCLE")?; Ok(Value::Float(ray_circle(v[0], v[1], v[2], v[3], v[4], v[5], v[6]))) }

        // --- physics: Broadphase (PHYSICS_BROAD_*) ---
        "physics_broad_new" => {
            arity!(0);
            Ok(Value::PhysicsBroad(Rc::new(RefCell::new(crate::physics::BroadPhase::new()))))
        }
        "physics_broad_clear" => { arity!(1); broad_h(&a[0], "PHYSICS_BROAD_CLEAR")?.borrow_mut().clear(); Ok(Value::Nil) }
        "physics_broad_add" => {
            arity!(4);
            let b = broad_h(&a[0], "PHYSICS_BROAD_ADD")?;
            let (x, y, r) = (need_num(&a[1], "PHYSICS_BROAD_ADD")?, need_num(&a[2], "PHYSICS_BROAD_ADD")?, need_num(&a[3], "PHYSICS_BROAD_ADD")?);
            if r < 0.0 { return err("PHYSICS_BROAD_ADD: Radius muss >= 0 sein"); }
            Ok(Value::Int(b.borrow_mut().add(x, y, r)))
        }
        "physics_broad_count" => { arity!(1); Ok(Value::Int(broad_h(&a[0], "PHYSICS_BROAD_COUNT")?.borrow().count())) }
        "physics_broad_query" => { arity!(1); Ok(Value::Int(broad_h(&a[0], "PHYSICS_BROAD_QUERY")?.borrow_mut().query())) }
        "physics_broad_pair_count" => { arity!(1); Ok(Value::Int(broad_h(&a[0], "PHYSICS_BROAD_PAIR_COUNT")?.borrow().pair_count())) }
        "physics_broad_pair_a" | "physics_broad_pair_b" => {
            arity!(2);
            let fn_ = if name == "physics_broad_pair_a" { "PHYSICS_BROAD_PAIR_A" } else { "PHYSICS_BROAD_PAIR_B" };
            let b = broad_h(&a[0], fn_)?;
            let b = b.borrow();
            let i = need_int(&a[1], fn_)?;
            let pc = b.pair_count();
            if i < 0 || i >= pc {
                return err(format!("{}: Index {} ausserhalb [0..{}]", fn_, i, pc - 1));
            }
            Ok(Value::Int(if name == "physics_broad_pair_a" { b.pair_a(i as usize) } else { b.pair_b(i as usize) }))
        }

        // ===== Modul: sprite (ohne SPRITE_DRAW -> das ist Grafik, in vm.rs) =====
        "sprite_new" => {
            arity!(3);
            let img = need_int(&a[0], "SPRITE_NEW")?;
            let (fw, fh) = (need_int(&a[1], "SPRITE_NEW")? as i32, need_int(&a[2], "SPRITE_NEW")? as i32);
            if fw <= 0 || fh <= 0 { return err("SPRITE_NEW: frame_w und frame_h muessen > 0 sein"); }
            Ok(Value::Sprite(Rc::new(RefCell::new(SpriteObj::new(img, fw, fh)))))
        }
        "sprite_set_pos" => { arity!(3); let s = spr(&a[0], "SPRITE_SET_POS")?; let mut s = s.borrow_mut(); s.x = need_num(&a[1], "SPRITE_SET_POS")?; s.y = need_num(&a[2], "SPRITE_SET_POS")?; Ok(Value::Nil) }
        "sprite_set_velocity" => { arity!(3); let s = spr(&a[0], "SPRITE_SET_VELOCITY")?; let mut s = s.borrow_mut(); s.vx = need_num(&a[1], "SPRITE_SET_VELOCITY")?; s.vy = need_num(&a[2], "SPRITE_SET_VELOCITY")?; Ok(Value::Nil) }
        "sprite_get_x" => { arity!(1); Ok(Value::Float(spr(&a[0], "SPRITE_GET_X")?.borrow().x)) }
        "sprite_get_y" => { arity!(1); Ok(Value::Float(spr(&a[0], "SPRITE_GET_Y")?.borrow().y)) }
        "sprite_get_width" => { arity!(1); Ok(Value::Int(spr(&a[0], "SPRITE_GET_WIDTH")?.borrow().frame_w as i64)) }
        "sprite_get_height" => { arity!(1); Ok(Value::Int(spr(&a[0], "SPRITE_GET_HEIGHT")?.borrow().frame_h as i64)) }
        "sprite_set_flip" => {
            arity!(3); let s = spr(&a[0], "SPRITE_SET_FLIP")?; let mut s = s.borrow_mut();
            s.flip_x = matches!(a[1], Value::Bool(true)); s.flip_y = matches!(a[2], Value::Bool(true)); Ok(Value::Nil)
        }
        "sprite_set_scale" => {
            arity!(3); let sx = need_num(&a[1], "SPRITE_SET_SCALE")?; let sy = need_num(&a[2], "SPRITE_SET_SCALE")?;
            if sx <= 0.0 || sy <= 0.0 { return err("SPRITE_SET_SCALE: Skalierung muss > 0 sein"); }
            let s = spr(&a[0], "SPRITE_SET_SCALE")?; let mut s = s.borrow_mut(); s.scale_x = sx; s.scale_y = sy; Ok(Value::Nil)
        }
        "sprite_tint" => {
            arity!(2); let c = need_int(&a[1], "SPRITE_TINT")?;
            if c < 0 || c > 0xFFFFFF { return err("SPRITE_TINT: Farbe muss 0..0xFFFFFF sein"); }
            spr(&a[0], "SPRITE_TINT")?.borrow_mut().tint = Some(c); Ok(Value::Nil)
        }
        "sprite_tint_clear" => { arity!(1); spr(&a[0], "SPRITE_TINT_CLEAR")?.borrow_mut().tint = None; Ok(Value::Nil) }
        "sprite_add_anim" => {
            arity!(5);
            let name = need_str(&a[1], "SPRITE_ADD_ANIM")?.to_string();
            if name.is_empty() { return err("SPRITE_ADD_ANIM: name muss nicht-leerer STRING sein"); }
            let first = need_int(&a[2], "SPRITE_ADD_ANIM")? as i32;
            let last = need_int(&a[3], "SPRITE_ADD_ANIM")? as i32;
            let fps = need_num(&a[4], "SPRITE_ADD_ANIM")?;
            if first < 0 || last < first { return err("SPRITE_ADD_ANIM: first >= 0 und last >= first noetig"); }
            if fps < 0.0 { return err("SPRITE_ADD_ANIM: fps muss >= 0 sein"); }
            spr(&a[0], "SPRITE_ADD_ANIM")?.borrow_mut().anims.insert(name, (first, last, fps)); Ok(Value::Nil)
        }
        "sprite_play" => { arity!(2); let n = need_str(&a[1], "SPRITE_PLAY")?.to_string(); spr(&a[0], "SPRITE_PLAY")?.borrow_mut().play(&n, true)?; Ok(Value::Nil) }
        "sprite_play_once" => { arity!(2); let n = need_str(&a[1], "SPRITE_PLAY_ONCE")?.to_string(); spr(&a[0], "SPRITE_PLAY_ONCE")?.borrow_mut().play(&n, false)?; Ok(Value::Nil) }
        "sprite_current_anim" => { arity!(1); Ok(Value::str_rc(&spr(&a[0], "SPRITE_CURRENT_ANIM")?.borrow().current_anim)) }
        "sprite_is_finished" => { arity!(1); Ok(Value::Bool(spr(&a[0], "SPRITE_IS_FINISHED")?.borrow().finished)) }
        "sprite_set_frame" => {
            arity!(2); let idx = need_int(&a[1], "SPRITE_SET_FRAME")?;
            if idx < 0 { return err("SPRITE_SET_FRAME: idx muss >= 0 sein"); }
            spr(&a[0], "SPRITE_SET_FRAME")?.borrow_mut().current_frame = idx as i32; Ok(Value::Nil)
        }
        "sprite_get_frame" => { arity!(1); Ok(Value::Int(spr(&a[0], "SPRITE_GET_FRAME")?.borrow().current_frame as i64)) }
        "sprite_update" => {
            arity!(2); let dt = need_int(&a[1], "SPRITE_UPDATE")?;
            if dt < 0 { return err("SPRITE_UPDATE: dt_ms muss >= 0 sein"); }
            spr(&a[0], "SPRITE_UPDATE")?.borrow_mut().update(dt); Ok(Value::Nil)
        }
        "sprite_collides" | "sprite_collide" => {
            arity!(2); let a0 = spr(&a[0], "SPRITE_COLLIDES")?.borrow(); let b0 = spr(&a[1], "SPRITE_COLLIDES")?.borrow();
            Ok(Value::Bool(a0.x < b0.x + b0.frame_w as f64 && a0.x + a0.frame_w as f64 > b0.x
                && a0.y < b0.y + b0.frame_h as f64 && a0.y + a0.frame_h as f64 > b0.y))
        }
        "sprite_hit_box" => {
            arity!(5); let s = spr(&a[0], "SPRITE_HIT_BOX")?.borrow();
            let (x, y, w, h) = (need_num(&a[1], "S")?, need_num(&a[2], "S")?, need_num(&a[3], "S")?, need_num(&a[4], "S")?);
            Ok(Value::Bool(s.x < x + w && s.x + s.frame_w as f64 > x && s.y < y + h && s.y + s.frame_h as f64 > y))
        }
        "sprite_hit_point" => {
            arity!(3); let s = spr(&a[0], "SPRITE_HIT_POINT")?.borrow();
            let (x, y) = (need_num(&a[1], "S")?, need_num(&a[2], "S")?);
            Ok(Value::Bool(s.x <= x && x < s.x + s.frame_w as f64 && s.y <= y && y < s.y + s.frame_h as f64))
        }

        // ===== Core File-I/O =====
        "fileexists" => { arity!(1); Ok(Value::Bool(std::path::Path::new(need_str(&a[0], "FILEEXISTS")?).is_file())) }
        "openfile" => {
            arity!(2);
            let path = need_str(&a[0], "OPENFILE")?.to_string();
            let mode = need_str(&a[1], "OPENFILE")?;
            let h = match mode {
                "r" => FileH::Read(std::io::BufReader::new(std::fs::File::open(&path).map_err(|e| format!("OPENFILE: {}", e))?)),
                "w" => FileH::Write(std::fs::File::create(&path).map_err(|e| format!("OPENFILE: {}", e))?),
                "a" => FileH::Write(std::fs::OpenOptions::new().append(true).create(true).open(&path).map_err(|e| format!("OPENFILE: {}", e))?),
                _ => return err(format!("OPENFILE: ungueltiger Modus '{}' (erlaubt: r, w, a)", mode)),
            };
            let m = mode.chars().next().unwrap();
            Ok(Value::File(Rc::new(RefCell::new(GbFile { path, mode: m, h }))))
        }
        "closefile" => {
            arity!(1);
            let f = file_h(&a[0], "CLOSEFILE")?;
            f.borrow_mut().h = FileH::Closed;
            Ok(Value::Nil)
        }
        "readline" => {
            arity!(1);
            let f = file_h(&a[0], "READLINE")?;
            let mut f = f.borrow_mut();
            match &mut f.h {
                FileH::Read(r) => {
                    let mut line = String::new();
                    r.read_line(&mut line).map_err(|e| format!("READLINE: {}", e))?;
                    while line.ends_with('\n') || line.ends_with('\r') { line.pop(); }
                    Ok(Value::str_rc(&line))
                }
                _ => err("READLINE: Datei wurde nicht im Lese-Modus geoeffnet"),
            }
        }
        "readall$" | "readall" => {
            arity!(1);
            let f = file_h(&a[0], "READALL$")?;
            let mut f = f.borrow_mut();
            match &mut f.h {
                FileH::Read(r) => { let mut s = String::new(); r.read_to_string(&mut s).map_err(|e| format!("READALL$: {}", e))?; Ok(Value::str_rc(&s)) }
                _ => err("READALL$: Datei wurde nicht im Lese-Modus geoeffnet"),
            }
        }
        "endoffile" => {
            arity!(1);
            let f = file_h(&a[0], "ENDOFFILE")?;
            let mut f = f.borrow_mut();
            match &mut f.h {
                FileH::Read(r) => Ok(Value::Bool(r.fill_buf().map_err(|e| format!("ENDOFFILE: {}", e))?.is_empty())),
                _ => err("ENDOFFILE erwartet Lese-FILE"),
            }
        }
        "writeline" | "write" => {
            arity!(2);
            let text = need_str(&a[1], "WRITE")?.to_string();
            let f = file_h(&a[0], "WRITE")?;
            let mut f = f.borrow_mut();
            match &mut f.h {
                FileH::Write(w) => {
                    let payload = if name == "writeline" { format!("{}\n", text) } else { text };
                    w.write_all(payload.as_bytes()).map_err(|e| format!("WRITE: {}", e))?;
                    Ok(Value::Nil)
                }
                _ => err("WRITE: Datei wurde nicht im Schreib-Modus geoeffnet"),
            }
        }
        // --- Datei/Verzeichnis (WP3, pfadbasiert -- kein FILE-Handle) ---
        "direxists" => { arity!(1); Ok(Value::Bool(std::path::Path::new(need_str(&a[0], "DIREXISTS")?).is_dir())) }
        "dirlist" => {
            arity!(1);
            let path = need_str(&a[0], "DIRLIST")?;
            let rd = std::fs::read_dir(path).map_err(|e| format!("DIRLIST: {}", e))?;
            let mut names: Vec<String> = Vec::new();
            for entry in rd {
                let entry = entry.map_err(|e| format!("DIRLIST: {}", e))?;
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
            names.sort();   // OS-Reihenfolge ist undefiniert -> deterministisch sortieren
            Ok(new_str_array(names))
        }
        "mkdir" => {
            arity!(1);
            std::fs::create_dir_all(need_str(&a[0], "MKDIR")?).map_err(|e| format!("MKDIR: {}", e))?;
            Ok(Value::Nil)
        }
        "deletefile" => {
            arity!(1);
            std::fs::remove_file(need_str(&a[0], "DELETEFILE")?).map_err(|e| format!("DELETEFILE: {}", e))?;
            Ok(Value::Nil)
        }
        "rename" => {
            arity!(2);
            std::fs::rename(need_str(&a[0], "RENAME")?, need_str(&a[1], "RENAME")?).map_err(|e| format!("RENAME: {}", e))?;
            Ok(Value::Nil)
        }
        "writeall" => {
            arity!(2);
            std::fs::write(need_str(&a[0], "WRITEALL")?, need_str(&a[1], "WRITEALL")?).map_err(|e| format!("WRITEALL: {}", e))?;
            Ok(Value::Nil)
        }
        "readlines" => {
            arity!(1);
            let text = std::fs::read_to_string(need_str(&a[0], "READLINES")?).map_err(|e| format!("READLINES: {}", e))?;
            Ok(new_str_array(text.lines().map(|l| l.to_string()).collect()))
        }
        "filesize" => {
            arity!(1);
            let m = std::fs::metadata(need_str(&a[0], "FILESIZE")?).map_err(|e| format!("FILESIZE: {}", e))?;
            Ok(Value::Int(m.len() as i64))
        }
        "pathjoin" => {
            if a.len() < 2 { return err(format!("PATHJOIN: erwartet mind. 2 Argumente, erhalten {}", a.len())); }
            // Mit '/' verbinden (deterministisch, plattformuebergreifend; raylib/
            // std akzeptieren '/' auch auf Windows). Erstes Teil behaelt fuehrendes
            // '/', folgende werden beidseitig getrimmt.
            let mut cleaned: Vec<String> = Vec::new();
            for (i, v) in a.iter().enumerate() {
                let s = need_str(v, "PATHJOIN")?;
                let t = if i == 0 { s.trim_end_matches('/') } else { s.trim_matches('/') };
                if !t.is_empty() { cleaned.push(t.to_string()); }
            }
            Ok(Value::str_rc(&cleaned.join("/")))
        }

        // ===== Modul: tween (zeitbasiert, NICHT bit-identisch) =====
        "tween_new" | "tween_new_loop" | "tween_new_pingpong" => {
            if a.len() < 3 || a.len() > 4 { return err(format!("{}: 3..4 Argumente", name.to_uppercase())); }
            let start = need_num(&a[0], "TWEEN")?;
            let end = need_num(&a[1], "TWEEN")?;
            let duration = need_int(&a[2], "TWEEN")?;
            if duration < 0 { return err("TWEEN: dauer_ms muss >= 0 sein"); }
            let mode = match name { "tween_new_loop" => "loop", "tween_new_pingpong" => "pingpong", _ => "once" };
            if (mode == "loop" || mode == "pingpong") && duration == 0 { return err("TWEEN: dauer_ms muss > 0 sein"); }
            let easing = if a.len() == 4 { need_str(&a[3], "TWEEN")?.to_lowercase() } else { "linear".to_string() };
            if !is_easing(&easing) { return err(format!("TWEEN: unbekanntes easing '{}'", easing)); }
            Ok(Value::Tween(Rc::new(RefCell::new(TweenObj { start, end, duration, easing, start_ms: now_ms(), paused_at: None, mode: mode.to_string() }))))
        }
        "tween_value" => { arity!(1); let t = tween_h(&a[0], "TWEEN_VALUE")?.borrow(); let p = ease(&t.easing, progress(&t)); Ok(Value::Float(t.start + (t.end - t.start) * p)) }
        "tween_progress" => { arity!(1); Ok(Value::Float(progress(&tween_h(&a[0], "TWEEN_PROGRESS")?.borrow()))) }
        "tween_done" => { arity!(1); let t = tween_h(&a[0], "TWEEN_DONE")?.borrow(); Ok(Value::Bool(t.mode == "once" && progress(&t) >= 1.0)) }
        "tween_restart" => { arity!(1); let mut t = tween_h(&a[0], "TWEEN_RESTART")?.borrow_mut(); t.start_ms = now_ms(); t.paused_at = None; Ok(Value::Nil) }
        "tween_pause" => { arity!(1); let mut t = tween_h(&a[0], "TWEEN_PAUSE")?.borrow_mut(); if t.paused_at.is_none() { t.paused_at = Some(elapsed(&t)); } Ok(Value::Nil) }
        "tween_resume" => { arity!(1); let mut t = tween_h(&a[0], "TWEEN_RESUME")?.borrow_mut(); if let Some(p) = t.paused_at { t.start_ms = now_ms() - p; t.paused_at = None; } Ok(Value::Nil) }
        "tween_reverse" => { arity!(1); let mut t = tween_h(&a[0], "TWEEN_REVERSE")?.borrow_mut(); let s = t.start; t.start = t.end; t.end = s; t.start_ms = now_ms(); t.paused_at = None; Ok(Value::Nil) }
        "tween_easings" => { arity!(0); Ok(Value::str_rc("in_back, in_bounce, in_cubic, in_elastic, in_quad, in_sine, inout_back, inout_bounce, inout_cubic, inout_elastic, inout_quad, inout_sine, linear, out_back, out_bounce, out_cubic, out_elastic, out_quad, out_sine")) }

        // ===== Modul: json =====
        "json_parse" => {
            arity!(1);
            let s = need_str(&a[0], "JSON_PARSE")?;
            let v: serde_json::Value = serde_json::from_str(s).map_err(|e| format!("JSON_PARSE: {}", e))?;
            Ok(Value::Json(Rc::new(v)))
        }
        "json_load" => {
            arity!(1);
            let path = need_str(&a[0], "JSON_LOAD")?;
            let text = std::fs::read_to_string(path).map_err(|e| format!("JSON_LOAD: {}", e))?;
            let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("JSON_LOAD: {}: {}", path, e))?;
            Ok(Value::Json(Rc::new(v)))
        }
        "json_stringify" => { arity!(1); Ok(Value::str_rc(&serde_json::to_string(json_h(&a[0], "JSON_STRINGIFY")?.as_ref()).unwrap_or_default())) }
        "json_pretty" => { arity!(1); Ok(Value::str_rc(&serde_json::to_string_pretty(json_h(&a[0], "JSON_PRETTY")?.as_ref()).unwrap_or_default())) }
        "json_get_string" => {
            arity!(2); let h = json_h(&a[0], "JSON_GET_STRING")?; let v = json_resolve(h, need_str(&a[1], "JSON_GET_STRING")?, "JSON_GET_STRING")?;
            v.as_str().map(Value::str_rc).ok_or_else(|| format!("JSON_GET_STRING: Pfad '{}' ist kein String", need_str(&a[1], "x").unwrap_or("")))
        }
        "json_get_int" => {
            arity!(2); let h = json_h(&a[0], "JSON_GET_INT")?; let v = json_resolve(h, need_str(&a[1], "JSON_GET_INT")?, "JSON_GET_INT")?;
            if let Some(i) = v.as_i64() { Ok(Value::Int(i)) }
            else if let Some(f) = v.as_f64() { if f.fract() == 0.0 { Ok(Value::Int(f as i64)) } else { err("JSON_GET_INT: kein Integer") } }
            else { err("JSON_GET_INT: kein Integer") }
        }
        "json_get_float" => {
            arity!(2); let h = json_h(&a[0], "JSON_GET_FLOAT")?; let v = json_resolve(h, need_str(&a[1], "JSON_GET_FLOAT")?, "JSON_GET_FLOAT")?;
            v.as_f64().map(Value::Float).ok_or_else(|| "JSON_GET_FLOAT: keine Zahl".to_string())
        }
        "json_get_bool" => {
            arity!(2); let h = json_h(&a[0], "JSON_GET_BOOL")?; let v = json_resolve(h, need_str(&a[1], "JSON_GET_BOOL")?, "JSON_GET_BOOL")?;
            v.as_bool().map(Value::Bool).ok_or_else(|| "JSON_GET_BOOL: kein Boolean".to_string())
        }
        "json_has" => {
            arity!(2); let h = json_h(&a[0], "JSON_HAS")?;
            Ok(Value::Bool(json_try_resolve(h, need_str(&a[1], "JSON_HAS")?).is_some()))
        }
        "json_len" => {
            arity!(2); let h = json_h(&a[0], "JSON_LEN")?; let v = json_resolve(h, need_str(&a[1], "JSON_LEN")?, "JSON_LEN")?;
            match v {
                serde_json::Value::Array(a) => Ok(Value::Int(a.len() as i64)),
                serde_json::Value::Object(o) => Ok(Value::Int(o.len() as i64)),
                serde_json::Value::String(s) => Ok(Value::Int(s.chars().count() as i64)),
                _ => err("JSON_LEN: weder Array, Objekt noch String"),
            }
        }
        "json_type" => {
            arity!(2); let h = json_h(&a[0], "JSON_TYPE")?;
            let t = match json_try_resolve(h, need_str(&a[1], "JSON_TYPE")?) {
                None => "missing",
                Some(serde_json::Value::Null) => "null",
                Some(serde_json::Value::Bool(_)) => "boolean",
                Some(serde_json::Value::Number(_)) => "number",
                Some(serde_json::Value::String(_)) => "string",
                Some(serde_json::Value::Array(_)) => "array",
                Some(serde_json::Value::Object(_)) => "object",
            };
            Ok(Value::str_rc(t))
        }

        // ===== Modul: save =====
        "save_new" => { arity!(0); Ok(Value::Save(Rc::new(RefCell::new(SaveHandle { version: 1, data: Vec::new() })))) }
        "save_exists" => { arity!(1); Ok(Value::Bool(std::path::Path::new(need_str(&a[0], "SAVE_EXISTS")?).is_file())) }
        "save_load" => { arity!(1); save_load(need_str(&a[0], "SAVE_LOAD")?) }
        "save_load_or_new" => {
            arity!(1); let p = need_str(&a[0], "SAVE_LOAD_OR_NEW")?;
            if !std::path::Path::new(p).exists() { Ok(Value::Save(Rc::new(RefCell::new(SaveHandle { version: 1, data: Vec::new() })))) }
            else { save_load(p) }
        }
        "save_write" => {
            arity!(2); let s = save_h(&a[0], "SAVE_WRITE")?.borrow(); let path = need_str(&a[1], "SAVE_WRITE")?;
            let mut data_obj = serde_json::Map::new();
            for (k, v) in &s.data { data_obj.insert(k.clone(), val_to_json(v)); }
            let mut root = serde_json::Map::new();
            root.insert("_version".to_string(), serde_json::Value::from(s.version));
            root.insert("data".to_string(), serde_json::Value::Object(data_obj));
            let text = serde_json::to_string_pretty(&serde_json::Value::Object(root)).unwrap_or_default();
            std::fs::write(path, text).map_err(|e| format!("SAVE_WRITE: {}", e))?;
            Ok(Value::Nil)
        }
        "save_delete_file" => {
            arity!(1); let p = need_str(&a[0], "SAVE_DELETE_FILE")?;
            match std::fs::remove_file(p) { Ok(_) => {}, Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}, Err(e) => return err(format!("SAVE_DELETE_FILE: {}", e)) }
            Ok(Value::Nil)
        }
        "save_version" => { arity!(1); Ok(Value::Int(save_h(&a[0], "SAVE_VERSION")?.borrow().version)) }
        "save_set_version" => { arity!(2); save_h(&a[0], "SAVE_SET_VERSION")?.borrow_mut().version = need_int(&a[1], "SAVE_SET_VERSION")?; Ok(Value::Nil) }
        "save_set_int" => { arity!(3); let k = need_str(&a[1], "SAVE_SET_INT")?.to_string(); let v = Value::Int(need_int(&a[2], "SAVE_SET_INT")?); save_h(&a[0], "SAVE_SET_INT")?.borrow_mut().set(k, v); Ok(Value::Nil) }
        "save_set_float" => { arity!(3); let k = need_str(&a[1], "SAVE_SET_FLOAT")?.to_string(); let v = Value::Float(need_num(&a[2], "SAVE_SET_FLOAT")?); save_h(&a[0], "SAVE_SET_FLOAT")?.borrow_mut().set(k, v); Ok(Value::Nil) }
        "save_set_string" => { arity!(3); let k = need_str(&a[1], "SAVE_SET_STRING")?.to_string(); let v = Value::str_rc(need_str(&a[2], "SAVE_SET_STRING")?); save_h(&a[0], "SAVE_SET_STRING")?.borrow_mut().set(k, v); Ok(Value::Nil) }
        "save_set_bool" => { arity!(3); let k = need_str(&a[1], "SAVE_SET_BOOL")?.to_string(); let v = match &a[2] { Value::Bool(b) => Value::Bool(*b), _ => return err("SAVE_SET_BOOL: BOOLEAN noetig") }; save_h(&a[0], "SAVE_SET_BOOL")?.borrow_mut().set(k, v); Ok(Value::Nil) }
        "save_get_int" => { arity!(2); let s = save_h(&a[0], "SAVE_GET_INT")?.borrow(); let k = need_str(&a[1], "SAVE_GET_INT")?; match s.get(k) { Some(Value::Int(i)) => Ok(Value::Int(*i)), Some(Value::Float(f)) if f.fract() == 0.0 => Ok(Value::Int(*f as i64)), Some(_) => err("SAVE_GET_INT: kein INTEGER"), None => err(format!("SAVE_GET_INT: Key '{}' nicht im Save", k)) } }
        "save_get_float" => { arity!(2); let s = save_h(&a[0], "SAVE_GET_FLOAT")?.borrow(); let k = need_str(&a[1], "SAVE_GET_FLOAT")?; match s.get(k) { Some(v) if is_num(v) => Ok(Value::Float(as_f64(v))), Some(_) => err("SAVE_GET_FLOAT: keine Zahl"), None => err(format!("SAVE_GET_FLOAT: Key '{}' nicht im Save", k)) } }
        "save_get_string" => { arity!(2); let s = save_h(&a[0], "SAVE_GET_STRING")?.borrow(); let k = need_str(&a[1], "SAVE_GET_STRING")?; match s.get(k) { Some(Value::Str(st)) => Ok(Value::Str(st.clone())), Some(_) => err("SAVE_GET_STRING: kein STRING"), None => err(format!("SAVE_GET_STRING: Key '{}' nicht im Save", k)) } }
        "save_get_bool" => { arity!(2); let s = save_h(&a[0], "SAVE_GET_BOOL")?.borrow(); let k = need_str(&a[1], "SAVE_GET_BOOL")?; match s.get(k) { Some(Value::Bool(b)) => Ok(Value::Bool(*b)), Some(_) => err("SAVE_GET_BOOL: kein BOOLEAN"), None => err(format!("SAVE_GET_BOOL: Key '{}' nicht im Save", k)) } }
        "save_get_int_or" => { arity!(3); let s = save_h(&a[0], "SAVE_GET_INT_OR")?.borrow(); match s.get(need_str(&a[1], "S")?) { Some(Value::Int(i)) => Ok(Value::Int(*i)), Some(Value::Float(f)) if f.fract() == 0.0 => Ok(Value::Int(*f as i64)), _ => Ok(a[2].clone()) } }
        "save_get_float_or" => { arity!(3); let s = save_h(&a[0], "SAVE_GET_FLOAT_OR")?.borrow(); match s.get(need_str(&a[1], "S")?) { Some(v) if is_num(v) => Ok(Value::Float(as_f64(v))), _ => Ok(Value::Float(need_num(&a[2], "S")?)) } }
        "save_get_string_or" => { arity!(3); let s = save_h(&a[0], "SAVE_GET_STRING_OR")?.borrow(); match s.get(need_str(&a[1], "S")?) { Some(Value::Str(st)) => Ok(Value::Str(st.clone())), _ => Ok(a[2].clone()) } }
        "save_get_bool_or" => { arity!(3); let s = save_h(&a[0], "SAVE_GET_BOOL_OR")?.borrow(); match s.get(need_str(&a[1], "S")?) { Some(Value::Bool(b)) => Ok(Value::Bool(*b)), _ => Ok(a[2].clone()) } }
        "save_has" => { arity!(2); Ok(Value::Bool(save_h(&a[0], "SAVE_HAS")?.borrow().get(need_str(&a[1], "SAVE_HAS")?).is_some())) }
        "save_delete" => { arity!(2); let k = need_str(&a[1], "SAVE_DELETE")?.to_string(); save_h(&a[0], "SAVE_DELETE")?.borrow_mut().remove(&k); Ok(Value::Nil) }
        "save_clear" => { arity!(1); save_h(&a[0], "SAVE_CLEAR")?.borrow_mut().data.clear(); Ok(Value::Nil) }
        "save_keys" => { arity!(1); let s = save_h(&a[0], "SAVE_KEYS")?.borrow(); let mut keys: Vec<String> = s.data.iter().map(|(k, _)| k.clone()).collect(); keys.sort(); Ok(Value::str_rc(&keys.join(", "))) }

        // ===== Modul: astar =====
        "astar_new" => {
            arity!(2); let (w, h) = (need_int(&a[0], "ASTAR_NEW")?, need_int(&a[1], "ASTAR_NEW")?);
            if w <= 0 || h <= 0 { return err(format!("ASTAR_NEW: Breite und Hoehe muessen > 0 sein (erhalten {}x{})", w, h)); }
            Ok(Value::AStar(Rc::new(RefCell::new(crate::astar::AStarGrid::new(w as i32, h as i32)))))
        }
        "astar_clear" => { arity!(1); astar_h(&a[0], "ASTAR_CLEAR")?.borrow_mut().clear(); Ok(Value::Nil) }
        "astar_width" => { arity!(1); Ok(Value::Int(astar_h(&a[0], "ASTAR_WIDTH")?.borrow().w as i64)) }
        "astar_height" => { arity!(1); Ok(Value::Int(astar_h(&a[0], "ASTAR_HEIGHT")?.borrow().h as i64)) }
        "astar_set_wall" => { arity!(3); let g = astar_h(&a[0], "ASTAR_SET_WALL")?; astar_xy(&g.borrow(), &a[1], &a[2], "ASTAR_SET_WALL")?; let (x, y) = (need_int(&a[1], "S")? as i32, need_int(&a[2], "S")? as i32); g.borrow_mut().set_wall(x, y); Ok(Value::Nil) }
        "astar_set_passable" => { arity!(3); let g = astar_h(&a[0], "ASTAR_SET_PASSABLE")?; astar_xy(&g.borrow(), &a[1], &a[2], "ASTAR_SET_PASSABLE")?; let (x, y) = (need_int(&a[1], "S")? as i32, need_int(&a[2], "S")? as i32); g.borrow_mut().set_passable(x, y); Ok(Value::Nil) }
        "astar_is_wall" => { arity!(3); let g = astar_h(&a[0], "ASTAR_IS_WALL")?; astar_xy(&g.borrow(), &a[1], &a[2], "ASTAR_IS_WALL")?; Ok(Value::Bool(g.borrow().is_wall(need_int(&a[1], "S")? as i32, need_int(&a[2], "S")? as i32))) }
        "astar_set_diagonal" => { arity!(2); let allow = matches!(a[1], Value::Bool(true)); astar_h(&a[0], "ASTAR_SET_DIAGONAL")?.borrow_mut().set_diagonal(allow); Ok(Value::Nil) }
        "astar_set_heuristic" => {
            arity!(2); let key = need_str(&a[1], "ASTAR_SET_HEURISTIC")?.to_lowercase();
            if !matches!(key.as_str(), "manhattan" | "euclid" | "chebyshev") { return err(format!("ASTAR_SET_HEURISTIC: unbekannte Heuristik '{}' (erlaubt: chebyshev, euclid, manhattan)", key)); }
            astar_h(&a[0], "ASTAR_SET_HEURISTIC")?.borrow_mut().set_heuristic(&key); Ok(Value::Nil)
        }
        "astar_set_diagonal_cost" => { arity!(2); let c = need_num(&a[1], "ASTAR_SET_DIAGONAL_COST")?; if c <= 0.0 { return err(format!("ASTAR_SET_DIAGONAL_COST: cost muss > 0 sein (erhalten {})", c)); } astar_h(&a[0], "ASTAR_SET_DIAGONAL_COST")?.borrow_mut().set_diagonal_cost(c); Ok(Value::Nil) }
        "astar_find" => {
            arity!(5); let g = astar_h(&a[0], "ASTAR_FIND")?;
            { let gb = g.borrow(); astar_xy(&gb, &a[1], &a[2], "ASTAR_FIND")?; astar_xy(&gb, &a[3], &a[4], "ASTAR_FIND")?; }
            let (sx, sy, ex, ey) = (need_int(&a[1], "S")? as i32, need_int(&a[2], "S")? as i32, need_int(&a[3], "S")? as i32, need_int(&a[4], "S")? as i32);
            Ok(Value::Bool(g.borrow_mut().find(sx, sy, ex, ey)))
        }
        "astar_path_len" => { arity!(1); Ok(Value::Int(astar_h(&a[0], "ASTAR_PATH_LEN")?.borrow().path_len() as i64)) }
        "astar_path_x" | "astar_path_y" => {
            arity!(2); let g = astar_h(&a[0], "ASTAR_PATH")?.borrow(); let idx = need_int(&a[1], "ASTAR_PATH")?;
            if idx < 0 || idx as usize >= g.path_len() { return err(format!("{}: Index {} ausserhalb [0..{}]", name.to_uppercase(), idx, g.path_len() as i64 - 1)); }
            Ok(Value::Int(if name == "astar_path_x" { g.path_x(idx as usize) } else { g.path_y(idx as usize) } as i64))
        }
        "astar_path_cost" => { arity!(1); Ok(Value::Float(astar_h(&a[0], "ASTAR_PATH_COST")?.borrow().path_cost())) }
        "astar_clear_path" => { arity!(1); astar_h(&a[0], "ASTAR_CLEAR_PATH")?.borrow_mut().clear_path(); Ok(Value::Nil) }

        // ===== Modul: particles (RNG-Emit -> NICHT bit-identisch) =====
        "particle_system_new" => { arity!(2); Ok(Value::Particles(Rc::new(RefCell::new(ParticleSys::new(need_num(&a[0], "PARTICLE_SYSTEM_NEW")?, need_num(&a[1], "PARTICLE_SYSTEM_NEW")?))))) }
        "particle_set_pos" => { arity!(3); let s = psys(&a[0], "PARTICLE_SET_POS")?; let mut s = s.borrow_mut(); s.x = need_num(&a[1], "S")?; s.y = need_num(&a[2], "S")?; Ok(Value::Nil) }
        "particle_count" => { arity!(1); Ok(Value::Int(psys(&a[0], "PARTICLE_COUNT")?.borrow().particles.len() as i64)) }
        "particle_clear" => { arity!(1); psys(&a[0], "PARTICLE_CLEAR")?.borrow_mut().particles.clear(); Ok(Value::Nil) }
        "particle_set_velocity" => {
            arity!(5); let s = psys(&a[0], "PARTICLE_SET_VELOCITY")?; let mut s = s.borrow_mut();
            s.vx_min = need_num(&a[1], "S")?; s.vx_max = need_num(&a[2], "S")?; s.vy_min = need_num(&a[3], "S")?; s.vy_max = need_num(&a[4], "S")?;
            if s.vx_max < s.vx_min || s.vy_max < s.vy_min { return err("PARTICLE_SET_VELOCITY: max muss >= min sein"); }
            Ok(Value::Nil)
        }
        "particle_set_lifetime" => {
            arity!(3); let (mi, ma) = (need_int(&a[1], "S")? as i32, need_int(&a[2], "S")? as i32);
            if mi < 0 || ma < mi { return err("PARTICLE_SET_LIFETIME: ms_min >= 0 und ms_max >= ms_min noetig"); }
            let s = psys(&a[0], "PARTICLE_SET_LIFETIME")?; let mut s = s.borrow_mut(); s.lifetime_min = mi; s.lifetime_max = ma; Ok(Value::Nil)
        }
        "particle_set_gravity" => { arity!(3); let s = psys(&a[0], "PARTICLE_SET_GRAVITY")?; let mut s = s.borrow_mut(); s.gravity_x = need_num(&a[1], "S")?; s.gravity_y = need_num(&a[2], "S")?; Ok(Value::Nil) }
        "particle_set_color" => { arity!(2); let c = need_int(&a[1], "PARTICLE_SET_COLOR")?; if c < 0 || c > 0xFFFFFF { return err("PARTICLE_SET_COLOR: Farbe muss 0..0xFFFFFF sein"); } psys(&a[0], "PARTICLE_SET_COLOR")?.borrow_mut().color = c; Ok(Value::Nil) }
        "particle_set_size" => {
            arity!(3); let (mi, ma) = (need_int(&a[1], "S")? as i32, need_int(&a[2], "S")? as i32);
            if mi < 1 || ma < mi { return err("PARTICLE_SET_SIZE: min >= 1 und max >= min noetig"); }
            let s = psys(&a[0], "PARTICLE_SET_SIZE")?; let mut s = s.borrow_mut(); s.size_min = mi; s.size_max = ma; Ok(Value::Nil)
        }
        "particle_set_fade" => { arity!(2); psys(&a[0], "PARTICLE_SET_FADE")?.borrow_mut().fade = matches!(a[1], Value::Bool(true)); Ok(Value::Nil) }
        "particle_set_mode" => {
            arity!(2); let m = need_str(&a[1], "PARTICLE_SET_MODE")?.to_lowercase();
            if !matches!(m.as_str(), "circle" | "pixel" | "square" | "streak" | "glow") { return err(format!("PARTICLE_SET_MODE: unbekannter Modus '{}'", m)); }
            psys(&a[0], "PARTICLE_SET_MODE")?.borrow_mut().mode = m; Ok(Value::Nil)
        }
        "particle_set_color_end" => { arity!(2); let c = need_int(&a[1], "PARTICLE_SET_COLOR_END")?; if c < 0 || c > 0xFFFFFF { return err("PARTICLE_SET_COLOR_END: Farbe muss 0..0xFFFFFF sein"); } let s = psys(&a[0], "PARTICLE_SET_COLOR_END")?; let mut s = s.borrow_mut(); s.color_end = c; s.has_color_end = true; Ok(Value::Nil) }
        "particle_emit" => {
            arity!(2); let count = need_int(&a[1], "PARTICLE_EMIT")?;
            if count < 0 { return err("PARTICLE_EMIT: count muss >= 0 sein"); }
            let s = psys(&a[0], "PARTICLE_EMIT")?; let mut s = s.borrow_mut();
            for _ in 0..count {
                let p = Particle {
                    x: s.x, y: s.y,
                    vx: rng_uniform(s.vx_min, s.vx_max), vy: rng_uniform(s.vy_min, s.vy_max),
                    lifetime: rng_randint(s.lifetime_min, s.lifetime_max), age: 0,
                    size: rng_randint(s.size_min, s.size_max), color: s.color,
                };
                s.particles.push(p);
            }
            Ok(Value::Nil)
        }
        "particle_update" => { arity!(2); let dt = need_int(&a[1], "PARTICLE_UPDATE")?; if dt < 0 { return err("PARTICLE_UPDATE: dt_ms muss >= 0 sein"); } psys(&a[0], "PARTICLE_UPDATE")?.borrow_mut().update(dt as i32); Ok(Value::Nil) }

        // ===== Modul: ecs =====
        "ecs_new_world" => { arity!(0); Ok(Value::Ecs(Rc::new(RefCell::new(crate::ecs::World::new())))) }
        "ecs_new_entity" => { arity!(1); Ok(Value::Int(ecs_h(&a[0], "ECS_NEW_ENTITY")?.borrow_mut().new_entity())) }
        "ecs_destroy" => { arity!(2); Ok(Value::Bool(ecs_h(&a[0], "ECS_DESTROY")?.borrow_mut().destroy(need_int(&a[1], "ECS_DESTROY")?))) }
        "ecs_alive" => { arity!(2); Ok(Value::Bool(ecs_h(&a[0], "ECS_ALIVE")?.borrow().alive(need_int(&a[1], "ECS_ALIVE")?))) }
        "ecs_count" => { arity!(1); Ok(Value::Int(ecs_h(&a[0], "ECS_COUNT")?.borrow().count() as i64)) }
        "ecs_add_int" => { arity!(4); let v = match &a[3] { Value::Int(i) => Value::Int(*i), _ => return err("ECS_ADD_INT erwartet value (INTEGER)") }; ecs_set(&a[0], &a[1], &a[2], v, "ECS_ADD_INT") }
        "ecs_add_float" => { arity!(4); if !is_num(&a[3]) { return err("ECS_ADD_FLOAT erwartet value (Zahl)"); } let v = Value::Float(as_f64(&a[3])); ecs_set(&a[0], &a[1], &a[2], v, "ECS_ADD_FLOAT") }
        "ecs_add_string" => { arity!(4); let v = match &a[3] { Value::Str(s) => Value::Str(s.clone()), _ => return err("ECS_ADD_STRING erwartet value (STRING)") }; ecs_set(&a[0], &a[1], &a[2], v, "ECS_ADD_STRING") }
        "ecs_add_bool" => { arity!(4); let v = match &a[3] { Value::Bool(b) => Value::Bool(*b), _ => return err("ECS_ADD_BOOL erwartet value (BOOLEAN)") }; ecs_set(&a[0], &a[1], &a[2], v, "ECS_ADD_BOOL") }
        "ecs_add_obj" => { arity!(4); ecs_set(&a[0], &a[1], &a[2], a[3].clone(), "ECS_ADD_OBJ") }
        "ecs_has" => { arity!(3); Ok(Value::Bool(ecs_h(&a[0], "ECS_HAS")?.borrow().has_component(need_int(&a[1], "ECS_HAS")?, need_str(&a[2], "ECS_HAS")?))) }
        "ecs_remove" => { arity!(3); Ok(Value::Bool(ecs_h(&a[0], "ECS_REMOVE")?.borrow_mut().remove_component(need_int(&a[1], "ECS_REMOVE")?, need_str(&a[2], "ECS_REMOVE")?))) }
        "ecs_get" => { arity!(3); ecs_h(&a[0], "ECS_GET")?.borrow().get(need_int(&a[1], "ECS_GET")?, need_str(&a[2], "ECS_GET")?, "ECS_GET") }
        "ecs_get_int" => { arity!(3); match ecs_get_v(&a, "ECS_GET_INT")? { Value::Int(i) => Ok(Value::Int(i)), _ => err("ECS_GET_INT: Component ist nicht INTEGER") } }
        "ecs_get_float" => { arity!(3); let v = ecs_get_v(&a, "ECS_GET_FLOAT")?; if is_num(&v) { Ok(Value::Float(as_f64(&v))) } else { err("ECS_GET_FLOAT: Component ist nicht Zahl") } }
        "ecs_get_string" => { arity!(3); match ecs_get_v(&a, "ECS_GET_STRING")? { Value::Str(s) => Ok(Value::Str(s)), _ => err("ECS_GET_STRING: Component ist nicht STRING") } }
        "ecs_get_bool" => { arity!(3); match ecs_get_v(&a, "ECS_GET_BOOL")? { Value::Bool(b) => Ok(Value::Bool(b)), _ => err("ECS_GET_BOOL: Component ist nicht BOOLEAN") } }
        "ecs_get_or_int" => { arity!(4); match ecs_or(&a)? { Some(Value::Int(i)) => Ok(Value::Int(i)), _ => Ok(a[3].clone()) } }
        "ecs_get_or_float" => { arity!(4); match ecs_or(&a)? { Some(v) if is_num(&v) => Ok(Value::Float(as_f64(&v))), _ => Ok(a[3].clone()) } }
        "ecs_get_or_string" => { arity!(4); match ecs_or(&a)? { Some(Value::Str(s)) => Ok(Value::Str(s)), _ => Ok(a[3].clone()) } }
        "ecs_get_or_bool" => { arity!(4); match ecs_or(&a)? { Some(Value::Bool(b)) => Ok(Value::Bool(b)), _ => Ok(a[3].clone()) } }
        "ecs_query" => { arity!(2); Ok(int_array(ecs_h(&a[0], "ECS_QUERY")?.borrow().query(&[need_str(&a[1], "ECS_QUERY")?]))) }
        "ecs_query2" => { arity!(3); Ok(int_array(ecs_h(&a[0], "ECS_QUERY2")?.borrow().query(&[need_str(&a[1], "Q")?, need_str(&a[2], "Q")?]))) }
        "ecs_query3" => { arity!(4); Ok(int_array(ecs_h(&a[0], "ECS_QUERY3")?.borrow().query(&[need_str(&a[1], "Q")?, need_str(&a[2], "Q")?, need_str(&a[3], "Q")?]))) }
        "ecs_integrate_float" => { arity!(3); Ok(Value::Int(ecs_h(&a[0], "ECS_INTEGRATE_FLOAT")?.borrow_mut().integrate_float(need_str(&a[1], "I")?, need_str(&a[2], "I")?))) }
        "ecs_integrate_int" => { arity!(3); Ok(Value::Int(ecs_h(&a[0], "ECS_INTEGRATE_INT")?.borrow_mut().integrate_int(need_str(&a[1], "I")?, need_str(&a[2], "I")?))) }
        "ecs_scale_float" => { arity!(3); Ok(Value::Int(ecs_h(&a[0], "ECS_SCALE_FLOAT")?.borrow_mut().scale_float(need_str(&a[1], "S")?, need_num(&a[2], "ECS_SCALE_FLOAT")?))) }
        "ecs_fill_float" => { arity!(3); Ok(Value::Int(ecs_h(&a[0], "ECS_FILL_FLOAT")?.borrow_mut().fill_float(need_str(&a[1], "F")?, need_num(&a[2], "ECS_FILL_FLOAT")?))) }
        "ecs_fill_int" => { arity!(3); Ok(Value::Int(ecs_h(&a[0], "ECS_FILL_INT")?.borrow_mut().fill_int(need_str(&a[1], "F")?, need_int(&a[2], "ECS_FILL_INT")?))) }
        "ecs_clamp_float" => { arity!(4); let n = ecs_h(&a[0], "ECS_CLAMP_FLOAT")?.borrow_mut().clamp_float(need_str(&a[1], "C")?, need_num(&a[2], "C")?, need_num(&a[3], "C")?)?; Ok(Value::Int(n)) }
        "ecs_remove_dead" => { arity!(3); Ok(Value::Int(ecs_h(&a[0], "ECS_REMOVE_DEAD")?.borrow_mut().remove_dead(need_str(&a[1], "R")?, need_num(&a[2], "ECS_REMOVE_DEAD")?))) }
        "ecs_count_with" => { arity!(2); Ok(Value::Int(ecs_h(&a[0], "ECS_COUNT_WITH")?.borrow().count_with(need_str(&a[1], "ECS_COUNT_WITH")?))) }

        // --- tiled-Modul (TILED_*) ---
        "tiled_load" => { arity!(1); Ok(Value::Tiled(crate::tiled::load(need_str(&a[0], "TILED_LOAD")?)?)) }
        "tiled_width" => { arity!(1); Ok(Value::Int(tiled_h(&a[0], "TILED_WIDTH")?.borrow().width)) }
        "tiled_height" => { arity!(1); Ok(Value::Int(tiled_h(&a[0], "TILED_HEIGHT")?.borrow().height)) }
        "tiled_tile_width" => { arity!(1); Ok(Value::Int(tiled_h(&a[0], "TILED_TILE_WIDTH")?.borrow().tile_w)) }
        "tiled_tile_height" => { arity!(1); Ok(Value::Int(tiled_h(&a[0], "TILED_TILE_HEIGHT")?.borrow().tile_h)) }
        "tiled_layer_count" => { arity!(1); Ok(Value::Int(tiled_h(&a[0], "TILED_LAYER_COUNT")?.borrow().layers.len() as i64)) }
        "tiled_layer_name" => {
            arity!(2);
            let m = tiled_h(&a[0], "TILED_LAYER_NAME")?; let mb = m.borrow();
            Ok(Value::str_rc(&layer_by_idx(&mb, need_int(&a[1], "TILED_LAYER_NAME")?, "TILED_LAYER_NAME")?.name))
        }
        "tiled_layer_type" => {
            arity!(2);
            let m = tiled_h(&a[0], "TILED_LAYER_TYPE")?; let mb = m.borrow();
            Ok(Value::str_rc(&layer_by_idx(&mb, need_int(&a[1], "TILED_LAYER_TYPE")?, "TILED_LAYER_TYPE")?.kind))
        }
        "tiled_layer_index" => {
            arity!(2);
            let m = tiled_h(&a[0], "TILED_LAYER_INDEX")?; let mb = m.borrow();
            let name = need_str(&a[1], "TILED_LAYER_INDEX")?;
            Ok(Value::Int(mb.layer_by_name.get(name).map(|&i| i as i64).unwrap_or(-1)))
        }
        "tiled_layer_width" => {
            arity!(2);
            let m = tiled_h(&a[0], "TILED_LAYER_WIDTH")?; let mb = m.borrow();
            Ok(Value::Int(layer_by_idx(&mb, need_int(&a[1], "TILED_LAYER_WIDTH")?, "TILED_LAYER_WIDTH")?.width))
        }
        "tiled_layer_height" => {
            arity!(2);
            let m = tiled_h(&a[0], "TILED_LAYER_HEIGHT")?; let mb = m.borrow();
            Ok(Value::Int(layer_by_idx(&mb, need_int(&a[1], "TILED_LAYER_HEIGHT")?, "TILED_LAYER_HEIGHT")?.height))
        }
        "tiled_tile_at" => {
            arity!(4);
            let m = tiled_h(&a[0], "TILED_TILE_AT")?; let mb = m.borrow();
            let li = tile_layer_idx(&mb, need_int(&a[1], "TILED_TILE_AT")?, "TILED_TILE_AT")?;
            let (tx, ty) = (need_int(&a[2], "TILED_TILE_AT")?, need_int(&a[3], "TILED_TILE_AT")?);
            let layer = &mb.layers[li];
            if tx < 0 || ty < 0 || tx >= layer.width || ty >= layer.height {
                Ok(Value::Int(0))
            } else {
                Ok(Value::Int(layer.tiles[(ty * layer.width + tx) as usize]))
            }
        }
        "tiled_tile_set" => {
            arity!(5);
            let m = tiled_h(&a[0], "TILED_TILE_SET")?; let mut mb = m.borrow_mut();
            let li = tile_layer_idx(&mb, need_int(&a[1], "TILED_TILE_SET")?, "TILED_TILE_SET")?;
            let (tx, ty, gid) = (need_int(&a[2], "TILED_TILE_SET")?, need_int(&a[3], "TILED_TILE_SET")?, need_int(&a[4], "TILED_TILE_SET")?);
            let layer = &mut mb.layers[li];
            if tx < 0 || ty < 0 || tx >= layer.width || ty >= layer.height {
                Ok(Value::Int(0))
            } else {
                let i = (ty * layer.width + tx) as usize;
                let old = layer.tiles[i]; layer.tiles[i] = gid; Ok(Value::Int(old))
            }
        }
        "tiled_count_gid" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_COUNT_GID")?; let mb = m.borrow();
            let li = tile_layer_idx(&mb, need_int(&a[1], "TILED_COUNT_GID")?, "TILED_COUNT_GID")?;
            let gid = need_int(&a[2], "TILED_COUNT_GID")?;
            Ok(Value::Int(mb.layers[li].tiles.iter().filter(|&&g| g == gid).count() as i64))
        }
        "tiled_fill_rect" => {
            arity!(7);
            let m = tiled_h(&a[0], "TILED_FILL_RECT")?; let mut mb = m.borrow_mut();
            let li = tile_layer_idx(&mb, need_int(&a[1], "TILED_FILL_RECT")?, "TILED_FILL_RECT")?;
            let (tx, ty, w, h, gid) = (need_int(&a[2], "T")?, need_int(&a[3], "T")?, need_int(&a[4], "T")?, need_int(&a[5], "T")?, need_int(&a[6], "T")?);
            let layer = &mut mb.layers[li];
            let (lw, lh) = (layer.width, layer.height);
            let (x0, y0) = (tx.max(0), ty.max(0));
            let (x1, y1) = ((tx + w).min(lw), (ty + h).min(lh));
            let mut n = 0i64;
            let mut yy = y0;
            while yy < y1 {
                let base = yy * lw;
                let mut xx = x0;
                while xx < x1 {
                    let i = (base + xx) as usize;
                    if layer.tiles[i] != gid { layer.tiles[i] = gid; n += 1; }
                    xx += 1;
                }
                yy += 1;
            }
            Ok(Value::Int(n))
        }
        "tiled_replace" => {
            arity!(4);
            let m = tiled_h(&a[0], "TILED_REPLACE")?; let mut mb = m.borrow_mut();
            let li = tile_layer_idx(&mb, need_int(&a[1], "TILED_REPLACE")?, "TILED_REPLACE")?;
            let (from, to) = (need_int(&a[2], "TILED_REPLACE")?, need_int(&a[3], "TILED_REPLACE")?);
            let layer = &mut mb.layers[li];
            let mut n = 0i64;
            for g in layer.tiles.iter_mut() {
                if *g == from { *g = to; n += 1; }
            }
            Ok(Value::Int(n))
        }
        "tiled_flood_fill" => {
            arity!(5);
            let m = tiled_h(&a[0], "TILED_FLOOD_FILL")?; let mut mb = m.borrow_mut();
            let li = tile_layer_idx(&mb, need_int(&a[1], "TILED_FLOOD_FILL")?, "TILED_FLOOD_FILL")?;
            let (tx, ty, gid) = (need_int(&a[2], "T")?, need_int(&a[3], "T")?, need_int(&a[4], "T")?);
            let layer = &mut mb.layers[li];
            let (w, h) = (layer.width, layer.height);
            Ok(Value::Int(crate::tiled::flood_fill(&mut layer.tiles, w, h, tx, ty, gid)))
        }
        "tiled_tile_prop_bool" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_TILE_PROP_BOOL")?; let mb = m.borrow();
            Ok(Value::Bool(mb.tile_property(need_int(&a[1], "T")?, need_str(&a[2], "T")?).map(|p| p.as_bool()).unwrap_or(false)))
        }
        "tiled_tile_prop_int" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_TILE_PROP_INT")?; let mb = m.borrow();
            Ok(Value::Int(mb.tile_property(need_int(&a[1], "T")?, need_str(&a[2], "T")?).map(|p| p.as_int()).unwrap_or(0)))
        }
        "tiled_tile_prop_float" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_TILE_PROP_FLOAT")?; let mb = m.borrow();
            Ok(Value::Float(mb.tile_property(need_int(&a[1], "T")?, need_str(&a[2], "T")?).map(|p| p.as_float()).unwrap_or(0.0)))
        }
        "tiled_tile_prop_string" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_TILE_PROP_STRING")?; let mb = m.borrow();
            Ok(Value::str_rc(&mb.tile_property(need_int(&a[1], "T")?, need_str(&a[2], "T")?).map(|p| p.as_string()).unwrap_or_default()))
        }
        "tiled_tile_has_prop" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_TILE_HAS_PROP")?; let mb = m.borrow();
            Ok(Value::Bool(mb.tile_property(need_int(&a[1], "T")?, need_str(&a[2], "T")?).is_some()))
        }
        "tiled_object_count" => {
            arity!(2);
            let m = tiled_h(&a[0], "TILED_OBJECT_COUNT")?; let mb = m.borrow();
            let layer = layer_by_name(&mb, need_str(&a[1], "TILED_OBJECT_COUNT")?, "TILED_OBJECT_COUNT")?;
            if layer.kind != "object" {
                return err(format!("TILED_OBJECT_COUNT: Layer '{}' ist kein Object-Layer", layer.name));
            }
            Ok(Value::Int(layer.objects.len() as i64))
        }
        "tiled_object_name" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_OBJECT_NAME")?; let mb = m.borrow();
            Ok(Value::str_rc(&get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_NAME")?.name))
        }
        "tiled_object_type" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_OBJECT_TYPE")?; let mb = m.borrow();
            Ok(Value::str_rc(&get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_TYPE")?.type_))
        }
        "tiled_object_x" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_OBJECT_X")?; let mb = m.borrow();
            Ok(Value::Float(get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_X")?.x))
        }
        "tiled_object_y" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_OBJECT_Y")?; let mb = m.borrow();
            Ok(Value::Float(get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_Y")?.y))
        }
        "tiled_object_width" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_OBJECT_WIDTH")?; let mb = m.borrow();
            Ok(Value::Float(get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_WIDTH")?.width))
        }
        "tiled_object_height" => {
            arity!(3);
            let m = tiled_h(&a[0], "TILED_OBJECT_HEIGHT")?; let mb = m.borrow();
            Ok(Value::Float(get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_HEIGHT")?.height))
        }
        "tiled_object_prop_bool" => {
            arity!(4);
            let m = tiled_h(&a[0], "TILED_OBJECT_PROP_BOOL")?; let mb = m.borrow();
            let o = get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_PROP_BOOL")?;
            Ok(Value::Bool(o.properties.get(need_str(&a[3], "T")?).map(|p| p.as_bool()).unwrap_or(false)))
        }
        "tiled_object_prop_int" => {
            arity!(4);
            let m = tiled_h(&a[0], "TILED_OBJECT_PROP_INT")?; let mb = m.borrow();
            let o = get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_PROP_INT")?;
            Ok(Value::Int(o.properties.get(need_str(&a[3], "T")?).map(|p| p.as_int()).unwrap_or(0)))
        }
        "tiled_object_prop_float" => {
            arity!(4);
            let m = tiled_h(&a[0], "TILED_OBJECT_PROP_FLOAT")?; let mb = m.borrow();
            let o = get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_PROP_FLOAT")?;
            Ok(Value::Float(o.properties.get(need_str(&a[3], "T")?).map(|p| p.as_float()).unwrap_or(0.0)))
        }
        "tiled_object_prop_string" => {
            arity!(4);
            let m = tiled_h(&a[0], "TILED_OBJECT_PROP_STRING")?; let mb = m.borrow();
            let o = get_obj(&mb, need_str(&a[1], "T")?, need_int(&a[2], "T")?, "TILED_OBJECT_PROP_STRING")?;
            Ok(Value::str_rc(&o.properties.get(need_str(&a[3], "T")?).map(|p| p.as_string()).unwrap_or_default()))
        }
        "tiled_tileset_count" => { arity!(1); Ok(Value::Int(tiled_h(&a[0], "TILED_TILESET_COUNT")?.borrow().tilesets.len() as i64)) }
        "tiled_tileset_image" => {
            arity!(2);
            let m = tiled_h(&a[0], "TILED_TILESET_IMAGE")?; let mb = m.borrow();
            let idx = need_int(&a[1], "TILED_TILESET_IMAGE")?;
            if idx < 0 || idx as usize >= mb.tilesets.len() {
                return err(format!("TILED_TILESET_IMAGE: Index {} ausserhalb [0..{}]", idx, mb.tilesets.len() as i64 - 1));
            }
            Ok(Value::str_rc(&mb.tilesets[idx as usize].image))
        }
        "tiled_tileset_firstgid" => {
            arity!(2);
            let m = tiled_h(&a[0], "TILED_TILESET_FIRSTGID")?; let mb = m.borrow();
            let idx = need_int(&a[1], "TILED_TILESET_FIRSTGID")?;
            if idx < 0 || idx as usize >= mb.tilesets.len() {
                return err("TILED_TILESET_FIRSTGID: Index ausserhalb".to_string());
            }
            Ok(Value::Int(mb.tilesets[idx as usize].first_gid))
        }

        // --- controller-Modul (CHAR_*) ---
        "char_new" => {
            arity!(4);
            let x = need_num(&a[0], "CHAR_NEW")?;
            let y = need_num(&a[1], "CHAR_NEW")?;
            let w = need_num(&a[2], "CHAR_NEW")?;
            let h = need_num(&a[3], "CHAR_NEW")?;
            if w <= 0.0 || h <= 0.0 {
                return err("CHAR_NEW: w und h muessen > 0 sein".to_string());
            }
            Ok(Value::CharController(Rc::new(RefCell::new(crate::controller::CharController::new(x, y, w, h)))))
        }
        "char_set_pos" => {
            arity!(3);
            let c = char_h(&a[0], "CHAR_SET_POS")?; let mut cc = c.borrow_mut();
            cc.x = need_num(&a[1], "CHAR_SET_POS")?;
            cc.y = need_num(&a[2], "CHAR_SET_POS")?;
            Ok(Value::Nil)
        }
        "char_set_move_speed" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_MOVE_SPEED")?;
            c.borrow_mut().move_speed = need_num(&a[1], "CHAR_SET_MOVE_SPEED")?.max(0.0);
            Ok(Value::Nil)
        }
        "char_set_gravity" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_GRAVITY")?;
            c.borrow_mut().gravity = need_num(&a[1], "CHAR_SET_GRAVITY")?;
            Ok(Value::Nil)
        }
        "char_set_max_fall" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_MAX_FALL")?;
            c.borrow_mut().max_fall = need_num(&a[1], "CHAR_SET_MAX_FALL")?.max(0.0);
            Ok(Value::Nil)
        }
        "char_set_jump_velocity" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_JUMP_VELOCITY")?;
            c.borrow_mut().jump_velocity = need_num(&a[1], "CHAR_SET_JUMP_VELOCITY")?.abs();
            Ok(Value::Nil)
        }
        "char_set_coyote_time" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_COYOTE_TIME")?;
            c.borrow_mut().coyote_max = need_int(&a[1], "CHAR_SET_COYOTE_TIME")?.max(0);
            Ok(Value::Nil)
        }
        "char_set_jump_buffer" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_JUMP_BUFFER")?;
            c.borrow_mut().jump_buffer_max = need_int(&a[1], "CHAR_SET_JUMP_BUFFER")?.max(0);
            Ok(Value::Nil)
        }
        "char_set_variable_jump" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_VARIABLE_JUMP")?;
            c.borrow_mut().variable_jump = need_bool(&a[1], "CHAR_SET_VARIABLE_JUMP")?;
            Ok(Value::Nil)
        }
        "char_set_variable_jump_cut" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_VARIABLE_JUMP_CUT")?;
            c.borrow_mut().variable_jump_cut = need_num(&a[1], "CHAR_SET_VARIABLE_JUMP_CUT")?.clamp(0.0, 1.0);
            Ok(Value::Nil)
        }
        "char_set_input" => {
            arity!(4);
            let c = char_h(&a[0], "CHAR_SET_INPUT")?;
            let ax = need_int(&a[1], "CHAR_SET_INPUT")?;
            let jp = need_bool(&a[2], "CHAR_SET_INPUT")?;
            let jh = need_bool(&a[3], "CHAR_SET_INPUT")?;
            c.borrow_mut().set_input(ax, jp, jh);
            Ok(Value::Nil)
        }
        "char_update" => {
            arity!(3);
            let c = char_h(&a[0], "CHAR_UPDATE")?;
            let m = tiled_h(&a[1], "CHAR_UPDATE")?; let mb = m.borrow();
            let li = tile_layer_idx(&mb, need_int(&a[2], "CHAR_UPDATE")?, "CHAR_UPDATE")?;
            c.borrow_mut().update(&mb, &mb.layers[li]);
            Ok(Value::Nil)
        }
        "char_x" => { arity!(1); Ok(Value::Float(char_h(&a[0], "CHAR_X")?.borrow().x)) }
        "char_y" => { arity!(1); Ok(Value::Float(char_h(&a[0], "CHAR_Y")?.borrow().y)) }
        "char_w" => { arity!(1); Ok(Value::Float(char_h(&a[0], "CHAR_W")?.borrow().w)) }
        "char_h" => { arity!(1); Ok(Value::Float(char_h(&a[0], "CHAR_H")?.borrow().h)) }
        "char_vx" => { arity!(1); Ok(Value::Float(char_h(&a[0], "CHAR_VX")?.borrow().vx)) }
        "char_vy" => { arity!(1); Ok(Value::Float(char_h(&a[0], "CHAR_VY")?.borrow().vy)) }
        "char_on_ground" => { arity!(1); Ok(Value::Bool(char_h(&a[0], "CHAR_ON_GROUND")?.borrow().on_ground)) }
        "char_on_wall_left" => { arity!(1); Ok(Value::Bool(char_h(&a[0], "CHAR_ON_WALL_LEFT")?.borrow().on_wall_left)) }
        "char_on_wall_right" => { arity!(1); Ok(Value::Bool(char_h(&a[0], "CHAR_ON_WALL_RIGHT")?.borrow().on_wall_right)) }
        "char_facing" => { arity!(1); Ok(Value::Int(char_h(&a[0], "CHAR_FACING")?.borrow().facing)) }
        "char_set_vx" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_VX")?;
            c.borrow_mut().vx = need_num(&a[1], "CHAR_SET_VX")?;
            Ok(Value::Nil)
        }
        "char_set_vy" => {
            arity!(2);
            let c = char_h(&a[0], "CHAR_SET_VY")?;
            c.borrow_mut().vy = need_num(&a[1], "CHAR_SET_VY")?;
            Ok(Value::Nil)
        }

        // --- tile_collide-Modul (TILE_*) ---
        "tile_sweep_x" | "tile_sweep_y" => {
            arity!(7);
            let axis = if name == "tile_sweep_x" { 0 } else { 1 };
            let fname = if axis == 0 { "TILE_SWEEP_X" } else { "TILE_SWEEP_Y" };
            let m = tiled_h(&a[0], fname)?; let mb = m.borrow();
            let li = tile_layer_idx(&mb, need_int(&a[1], fname)?, fname)?;
            let x = need_num(&a[2], fname)?;
            let y = need_num(&a[3], fname)?;
            let w = need_num(&a[4], fname)?;
            let h = need_num(&a[5], fname)?;
            let d = need_num(&a[6], fname)?;
            if w <= 0.0 || h <= 0.0 {
                return err(format!("{}: w und h muessen > 0 sein", fname));
            }
            let (np, hit) = mb.sweep_axis(&mb.layers[li], x, y, w, h, d, axis);
            Ok(Value::Tuple(Rc::new(vec![Value::Float(np), Value::Bool(hit)])))
        }
        "tile_is_solid" => {
            arity!(4);
            let m = tiled_h(&a[0], "TILE_IS_SOLID")?; let mb = m.borrow();
            let li = tile_layer_idx(&mb, need_int(&a[1], "TILE_IS_SOLID")?, "TILE_IS_SOLID")?;
            let tx = need_int(&a[2], "TILE_IS_SOLID")?;
            let ty = need_int(&a[3], "TILE_IS_SOLID")?;
            Ok(Value::Bool(mb.tile_is_solid_at(&mb.layers[li], tx, ty)))
        }
        "tile_at_pixel" => {
            arity!(4);
            let m = tiled_h(&a[0], "TILE_AT_PIXEL")?; let mb = m.borrow();
            let li = tile_layer_idx(&mb, need_int(&a[1], "TILE_AT_PIXEL")?, "TILE_AT_PIXEL")?;
            let px = need_num(&a[2], "TILE_AT_PIXEL")?;
            let py = need_num(&a[3], "TILE_AT_PIXEL")?;
            let (tw, th) = (mb.tile_w, mb.tile_h);
            if tw <= 0 || th <= 0 {
                Ok(Value::Int(0))
            } else {
                let tx = (px / tw as f64).floor() as i64;
                let ty = (py / th as f64).floor() as i64;
                let layer = &mb.layers[li];
                let g = if tx < 0 || ty < 0 || tx >= layer.width || ty >= layer.height {
                    0
                } else {
                    layer.tiles[(ty * layer.width + tx) as usize]
                };
                Ok(Value::Int(g))
            }
        }

        // --- regex-Modul (REGEX_*) ---
        "regex_match" => {
            arity!(2);
            let pat = format!("^(?:{})$", need_str(&a[1], "REGEX_MATCH")?);
            Ok(Value::Bool(regex_compile(&pat)?.is_match(need_str(&a[0], "REGEX_MATCH")?)))
        }
        "regex_test" => {
            arity!(2);
            Ok(Value::Bool(regex_compile(need_str(&a[1], "REGEX_TEST")?)?
                .is_match(need_str(&a[0], "REGEX_TEST")?)))
        }
        "regex_find" => {
            arity!(2);
            let re = regex_compile(need_str(&a[1], "REGEX_FIND")?)?;
            let t = need_str(&a[0], "REGEX_FIND")?;
            Ok(Value::str_rc(re.find(t).map(|m| m.as_str()).unwrap_or("")))
        }
        "regex_find_all" => {
            arity!(2);
            let re = regex_compile(need_str(&a[1], "REGEX_FIND_ALL")?)?;
            let t = need_str(&a[0], "REGEX_FIND_ALL")?;
            let mut flat: Vec<String> = Vec::new();
            if re.captures_len() > 1 {
                // mind. eine Capture-Gruppe -> Gruppe 1 (wie Python findall)
                for cap in re.captures_iter(t) {
                    flat.push(cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string());
                }
            } else {
                for m in re.find_iter(t) { flat.push(m.as_str().to_string()); }
            }
            Ok(new_str_array(flat))
        }
        "regex_replace" => {
            arity!(3);
            let re = regex_compile(need_str(&a[1], "REGEX_REPLACE")?)?;
            let t = need_str(&a[0], "REGEX_REPLACE")?;
            let rep = translate_repl(need_str(&a[2], "REGEX_REPLACE")?);
            Ok(Value::str_rc(re.replace_all(t, rep.as_str()).as_ref()))
        }
        "regex_replace_once" => {
            arity!(3);
            let re = regex_compile(need_str(&a[1], "REGEX_REPLACE_ONCE")?)?;
            let t = need_str(&a[0], "REGEX_REPLACE_ONCE")?;
            let rep = translate_repl(need_str(&a[2], "REGEX_REPLACE_ONCE")?);
            Ok(Value::str_rc(re.replace(t, rep.as_str()).as_ref()))
        }
        "regex_split" => {
            arity!(2);
            let re = regex_compile(need_str(&a[1], "REGEX_SPLIT")?)?;
            let t = need_str(&a[0], "REGEX_SPLIT")?;
            Ok(new_str_array(re.split(t).map(|s| s.to_string()).collect()))
        }

        _ => err(format!("__UNKNOWN_BUILTIN__:{}", name)),
    }
}

fn ecs_h<'a>(v: &'a Value, fn_: &str) -> Result<&'a Rc<RefCell<crate::ecs::World>>, String> {
    match v { Value::Ecs(w) => Ok(w), _ => Err(format!("{} erwartet ECS_WORLD", fn_)) }
}
fn ecs_set(w: &Value, ent: &Value, name: &Value, value: Value, fn_: &str) -> R {
    let ent = need_int(ent, fn_)?;
    let name = need_str(name, fn_)?.to_string();
    ecs_h(w, fn_)?.borrow_mut().set(&name, ent, value, fn_)?;
    Ok(Value::Nil)
}
fn ecs_get_v(a: &[Value], fn_: &str) -> R {
    ecs_h(&a[0], fn_)?.borrow().get(need_int(&a[1], fn_)?, need_str(&a[2], fn_)?, fn_)
}
fn ecs_or(a: &[Value]) -> Result<Option<Value>, String> {
    Ok(ecs_h(&a[0], "ECS_GET_OR")?.borrow().get_or(need_int(&a[1], "ECS_GET_OR")?, need_str(&a[2], "ECS_GET_OR")?))
}
fn int_array(values: Vec<i64>) -> Value {
    let n = values.len() as i64;
    let mut arr = GbArray::new("integer".to_string(), vec![n], || Value::Int(0));
    for (i, v) in values.into_iter().enumerate() { arr.values[i] = Value::Int(v); }
    Value::Array(Rc::new(RefCell::new(arr)))
}

fn psys<'a>(v: &'a Value, fn_: &str) -> Result<&'a Rc<RefCell<ParticleSys>>, String> {
    match v { Value::Particles(p) => Ok(p), _ => Err(format!("{} erwartet PARTICLE_SYSTEM", fn_)) }
}

fn rng_uniform(a: f64, b: f64) -> f64 {
    let r = (next_rand() >> 11) as f64 / (1u64 << 53) as f64;
    a + (b - a) * r
}
fn rng_randint(a: i32, b: i32) -> i32 {
    if b <= a { return a; }
    let span = (b - a + 1) as u64;
    a + (next_rand() % span) as i32
}

fn astar_h<'a>(v: &'a Value, fn_: &str) -> Result<&'a Rc<RefCell<crate::astar::AStarGrid>>, String> {
    match v { Value::AStar(g) => Ok(g), _ => Err(format!("{} erwartet ASTAR_GRID", fn_)) }
}

fn broad_h<'a>(v: &'a Value, fn_: &str) -> Result<&'a Rc<RefCell<crate::physics::BroadPhase>>, String> {
    match v { Value::PhysicsBroad(b) => Ok(b), _ => Err(format!("{} erwartet PHYSICS_BROAD", fn_)) }
}

fn astar_xy(g: &crate::astar::AStarGrid, x: &Value, y: &Value, fn_: &str) -> Result<(), String> {
    let (xi, yi) = (need_int(x, fn_)?, need_int(y, fn_)?);
    if !g.in_bounds(xi as i32, yi as i32) {
        return Err(format!("{}: ({}, {}) ausserhalb des Grids [{}x{}]", fn_, xi, yi, g.w, g.h));
    }
    Ok(())
}

fn save_h<'a>(v: &'a Value, fn_: &str) -> Result<&'a Rc<RefCell<SaveHandle>>, String> {
    match v { Value::Save(s) => Ok(s), _ => Err(format!("{} erwartet SAVE_HANDLE (aus SAVE_NEW/SAVE_LOAD)", fn_)) }
}

fn val_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Int(i) => serde_json::Value::from(*i),
        Value::Float(f) => serde_json::json!(f),
        Value::Str(s) => serde_json::Value::String(s.to_string()),
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Json(j) => j.as_ref().clone(),
        _ => serde_json::Value::Null,
    }
}

fn json_to_val(j: &serde_json::Value) -> Value {
    match j {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { Value::Int(i) } else { Value::Float(n.as_f64().unwrap_or(0.0)) }
        }
        serde_json::Value::String(s) => Value::str_rc(s),
        other => Value::Json(Rc::new(other.clone())),
    }
}

fn save_load(path: &str) -> R {
    let text = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound { format!("SAVE_LOAD: Datei '{}' nicht gefunden", path) } else { format!("SAVE_LOAD: {}", e) }
    })?;
    let raw: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("SAVE_LOAD: '{}' ist kein gueltiges JSON ({})", path, e))?;
    let obj = raw.as_object().ok_or("SAVE_LOAD: Save-Datei muss ein JSON-Objekt sein")?;
    let version = obj.get("_version").and_then(|v| v.as_i64()).unwrap_or(1);
    let mut data = Vec::new();
    if let Some(d) = obj.get("data").and_then(|v| v.as_object()) {
        for (k, v) in d { data.push((k.clone(), json_to_val(v))); }
    }
    Ok(Value::Save(Rc::new(RefCell::new(SaveHandle { version, data }))))
}

fn json_h<'a>(v: &'a Value, fn_: &str) -> Result<&'a Rc<serde_json::Value>, String> {
    match v { Value::Json(j) => Ok(j), _ => Err(format!("{} erwartet JSON-Handle (aus JSON_PARSE)", fn_)) }
}

/// Navigiert den Pfad ("user.name" / "items.0"); Fehler wenn nicht gefunden.
fn json_resolve<'a>(root: &'a serde_json::Value, path: &str, fn_: &str) -> Result<&'a serde_json::Value, String> {
    json_try_resolve(root, path).ok_or_else(|| format!("{}: Pfad '{}' nicht aufloesbar", fn_, path))
}

fn json_try_resolve<'a>(root: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() { return Some(root); }
    let mut cur = root;
    for part in path.split('.') {
        match cur {
            serde_json::Value::Array(arr) => {
                let idx: usize = part.parse().ok()?;
                cur = arr.get(idx)?;
            }
            serde_json::Value::Object(obj) => { cur = obj.get(part)?; }
            _ => return None,
        }
    }
    Some(cur)
}

fn file_h<'a>(v: &'a Value, fn_: &str) -> Result<&'a Rc<RefCell<GbFile>>, String> {
    match v { Value::File(f) => Ok(f), _ => Err(format!("{} erwartet FILE", fn_)) }
}

fn tween_h<'a>(v: &'a Value, fn_: &str) -> Result<&'a Rc<RefCell<TweenObj>>, String> {
    match v { Value::Tween(t) => Ok(t), _ => Err(format!("{} erwartet TWEEN", fn_)) }
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn elapsed(t: &TweenObj) -> i64 {
    t.paused_at.unwrap_or_else(|| now_ms() - t.start_ms)
}

fn progress(t: &TweenObj) -> f64 {
    if t.duration == 0 { return 1.0; }
    let e = elapsed(t);
    if e <= 0 { return 0.0; }
    let d = t.duration;
    match t.mode.as_str() {
        "loop" => (e % d) as f64 / d as f64,
        "pingpong" => {
            let cycle = e % (2 * d);
            if cycle <= d { cycle as f64 / d as f64 } else { 2.0 - cycle as f64 / d as f64 }
        }
        _ => if e >= d { 1.0 } else { e as f64 / d as f64 },
    }
}

fn is_easing(n: &str) -> bool {
    matches!(n, "linear" | "in_quad" | "out_quad" | "inout_quad" | "in_cubic" | "out_cubic" | "inout_cubic"
        | "in_sine" | "out_sine" | "inout_sine" | "in_bounce" | "out_bounce" | "inout_bounce"
        | "in_elastic" | "out_elastic" | "inout_elastic" | "in_back" | "out_back" | "inout_back")
}

fn ease_out_bounce(t: f64) -> f64 {
    let (n1, d1) = (7.5625f64, 2.75f64);
    if t < 1.0 / d1 { n1 * t * t }
    else if t < 2.0 / d1 { let t = t - 1.5 / d1; n1 * t * t + 0.75 }
    else if t < 2.5 / d1 { let t = t - 2.25 / d1; n1 * t * t + 0.9375 }
    else { let t = t - 2.625 / d1; n1 * t * t + 0.984375 }
}

fn ease(name: &str, t: f64) -> f64 {
    use std::f64::consts::PI;
    match name {
        "linear" => t,
        "in_quad" => t * t,
        "out_quad" => 1.0 - (1.0 - t).powi(2),
        "inout_quad" => if t < 0.5 { 2.0 * t * t } else { 1.0 - 2.0 * (1.0 - t).powi(2) },
        "in_cubic" => t.powi(3),
        "out_cubic" => 1.0 - (1.0 - t).powi(3),
        "inout_cubic" => if t < 0.5 { 4.0 * t.powi(3) } else { 1.0 - 4.0 * (1.0 - t).powi(3) },
        "in_sine" => 1.0 - (t * PI / 2.0).cos(),
        "out_sine" => (t * PI / 2.0).sin(),
        "inout_sine" => -((PI * t).cos() - 1.0) / 2.0,
        "out_bounce" => ease_out_bounce(t),
        "in_bounce" => 1.0 - ease_out_bounce(1.0 - t),
        "inout_bounce" => if t < 0.5 { (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0 } else { (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0 },
        "out_elastic" => { if t == 0.0 || t == 1.0 { t } else { let c4 = (2.0 * PI) / 3.0; 2f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0 } },
        "in_elastic" => { if t == 0.0 || t == 1.0 { t } else { let c4 = (2.0 * PI) / 3.0; -(2f64.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin() } },
        "inout_elastic" => {
            if t == 0.0 || t == 1.0 { t } else {
                let c5 = (2.0 * PI) / 4.5;
                if t < 0.5 { -(2f64.powf(20.0 * t - 10.0)) * ((20.0 * t - 11.125) * c5).sin() / 2.0 }
                else { 2f64.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin() / 2.0 + 1.0 }
            }
        }
        "in_back" => { let c1 = 1.70158; let c3 = c1 + 1.0; c3 * t.powi(3) - c1 * t.powi(2) },
        "out_back" => { let c1 = 1.70158; let c3 = c1 + 1.0; 1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2) },
        "inout_back" => {
            let c1 = 1.70158; let c2 = c1 * 1.525;
            if t < 0.5 { ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0 }
            else { ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0) / 2.0 }
        }
        _ => t,
    }
}

fn spr<'a>(v: &'a Value, fn_: &str) -> Result<&'a std::rc::Rc<std::cell::RefCell<SpriteObj>>, String> {
    match v {
        Value::Sprite(s) => Ok(s),
        _ => Err(format!("{} erwartet SPRITE, erhalten {}", fn_, v.type_name())),
    }
}

fn vec2(v: &Value, fn_: &str) -> Result<(f64, f64), String> {
    match v {
        Value::Vec2(x, y) => Ok((*x, *y)),
        _ => Err(format!("{}: Erwartet VEC2, erhalten {}", fn_, v.type_name())),
    }
}

fn nums(a: &[Value], fn_: &str) -> Result<Vec<f64>, String> {
    let mut v = Vec::with_capacity(a.len());
    for (i, x) in a.iter().enumerate() { v.push(need_num(x, fn_).map_err(|_| format!("{}: Arg {} muss Zahl sein", fn_, i + 1))?); }
    Ok(v)
}

fn bezier_1d(t: f64, p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

fn catmull_1d(t: f64, p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
    let t2 = t * t; let t3 = t2 * t;
    0.5 * (2.0 * p1 + (-p0 + p2) * t + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2 + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

fn reflect(vx: f64, vy: f64, nx: f64, ny: f64, want_x: bool) -> f64 {
    let nl2 = nx * nx + ny * ny;
    if nl2 == 0.0 { return if want_x { vx } else { vy }; }
    let nl = nl2.sqrt();
    let (nxh, nyh) = (nx / nl, ny / nl);
    let dot = vx * nxh + vy * nyh;
    if want_x { vx - 2.0 * dot * nxh } else { vy - 2.0 * dot * nyh }
}

fn ray_box(rx: f64, ry: f64, dx: f64, dy: f64, bx: f64, by: f64, bw: f64, bh: f64) -> f64 {
    let (tx_min, tx_max);
    if dx == 0.0 {
        if rx < bx || rx >= bx + bw { return -1.0; }
        tx_min = f64::NEG_INFINITY; tx_max = f64::INFINITY;
    } else {
        let t1 = (bx - rx) / dx; let t2 = (bx + bw - rx) / dx;
        tx_min = t1.min(t2); tx_max = t1.max(t2);
    }
    let (ty_min, ty_max);
    if dy == 0.0 {
        if ry < by || ry >= by + bh { return -1.0; }
        ty_min = f64::NEG_INFINITY; ty_max = f64::INFINITY;
    } else {
        let t1 = (by - ry) / dy; let t2 = (by + bh - ry) / dy;
        ty_min = t1.min(t2); ty_max = t1.max(t2);
    }
    let t_enter = tx_min.max(ty_min);
    let t_exit = tx_max.min(ty_max);
    if t_enter > t_exit || t_exit < 0.0 || t_enter > 1.0 { return -1.0; }
    t_enter.max(0.0)
}

fn ray_circle(rx: f64, ry: f64, dx: f64, dy: f64, cx: f64, cy: f64, cr: f64) -> f64 {
    let fx = rx - cx; let fy = ry - cy;
    let a = dx * dx + dy * dy;
    if a == 0.0 {
        return if fx * fx + fy * fy <= cr * cr { 0.0 } else { -1.0 };
    }
    let b = 2.0 * (fx * dx + fy * dy);
    let c = fx * fx + fy * fy - cr * cr;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 { return -1.0; }
    let sq = disc.sqrt();
    let t1 = (-b - sq) / (2.0 * a);
    let t2 = (-b + sq) / (2.0 * a);
    for t in [t1, t2] { if (0.0..=1.0).contains(&t) { return t; } }
    -1.0
}

/// Map-Wert-Coercion (`_coerce_map_value`).
fn coerce_map_value(v: &Value, vt: &str) -> R {
    match vt {
        "integer" => match v { Value::Int(_) => Ok(v.clone()), _ => err(format!("Map-Wert: Erwartet INTEGER, erhalten {}", v.type_name())) },
        "float" => match v { Value::Float(_) => Ok(v.clone()), Value::Int(i) => Ok(Value::Float(*i as f64)), _ => err("Map-Wert: Erwartet FLOAT".to_string()) },
        "string" => match v { Value::Str(_) => Ok(v.clone()), _ => err("Map-Wert: Erwartet STRING".to_string()) },
        "boolean" => match v { Value::Bool(_) => Ok(v.clone()), _ => err("Map-Wert: Erwartet BOOLEAN".to_string()) },
        _ => Ok(v.clone()),
    }
}

/// Ordering fuer SORT (homogene Arrays).
fn cmp_value(x: &Value, y: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering::Equal;
    if is_num(x) && is_num(y) {
        as_f64(x).partial_cmp(&as_f64(y)).unwrap_or(Equal)
    } else if let (Value::Str(a), Value::Str(b)) = (x, y) {
        a.as_ref().cmp(b.as_ref())
    } else {
        Equal
    }
}

/// Liste der Grafik-Builtin-Namen (nur fuer bessere Fehlermeldungen).
pub fn is_graphics_builtin(name: &str) -> bool {
    matches!(name,
        "screen" | "cls" | "flip" | "text" | "text_bold" | "text_size" | "circle"
        | "box" | "line" | "plot" | "rect" | "drawimage" | "loadimage" | "loadsound"
        | "playsound" | "keypressed" | "keydown" | "mousex" | "mousey" | "mousebutton"
        | "quitrequested" | "sleep" | "imagewidth" | "imageheight" | "color" | "fill"
    )
}
