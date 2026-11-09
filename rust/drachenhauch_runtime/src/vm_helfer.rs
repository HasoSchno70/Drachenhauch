//! Freie Helfer der VM: Meldungen, Index und Scheiben, Rechnen (`konstant_rechnen`, `cmp`, `div` ...), Wandeln (`coerce`), Felder anhaengen.
//!
//! Teil von `vm.rs`, herausgeloest ohne Aenderung am Inhalt.

use super::*;

// ===========================================================================
// Helfer
// ===========================================================================

/// Haengt an eine Laufzeitmeldung ueber ARGUMENTE die Signatur des Befehls.
///
/// "AUDIO_TONE: Argument 1 muss Zahl sein" sagt nicht, was an Platz 1 gehoert.
/// Der Name steht am Anfang jeder solchen Meldung, und das Befehlsverzeichnis
/// kennt die Form -- also wird sie angehaengt. Nur bei Meldungen, in denen es
/// erkennbar um Argumente geht: "Datei nicht gefunden" braucht sie nicht.
pub(crate) fn mit_signatur(msg: &str) -> String {
    let Some((kopf, rest)) = msg.split_once(':') else { return msg.to_string() };
    if kopf.is_empty() || kopf.len() > 40
        || !kopf.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '$') {
        return msg.to_string();
    }
    let ueber_argumente = ["Argument", "erwartet", "Erwartet", "muss", "erhalten"]
        .iter().any(|w| rest.contains(w));
    if !ueber_argumente { return msg.to_string(); }
    let low = kopf.to_lowercase();
    for (n, sig, _) in crate::compiler::builtin_eintraege() {
        if n.to_lowercase() == low && !sig.is_empty() && !sig.contains("Argumente") {
            return format!("{} -- Aufruf: {}", msg, sig);
        }
    }
    msg.to_string()
}

/// Zeichnet dieser Befehl auf den BILDSCHIRM? Dann braucht er ein Fenster
/// aus SCREEN -- anders als LOADIMAGE/IMAGE_*/imgfx, die auch ohne eins
/// arbeiten (sie brauchen nur den GL-Kontext).
///
/// Die Liste ist mit Absicht von Hand gefuehrt und kurz: sie deckt die
/// Befehle ab, mit denen ein erstes Grafikprogramm anfaengt. Ein Befehl, der
/// hier fehlt, verhaelt sich wie bisher -- ein Befehl, der zu Unrecht
/// darinstuende, braeche fremden Code.
pub(crate) fn ist_schirmbefehl(name: &str) -> bool {
    matches!(name,
        "flip" | "cls"
        | "plot" | "plots" | "line" | "lines" | "linew" | "box" | "boxes"
        | "rect" | "boxround" | "rectround" | "circle" | "circles"
        | "triangle" | "polygon" | "spline" | "gradientv" | "gradienth"
        | "pie" | "pieoutline" | "circle_gradient" | "boxrot" | "gradient4"
        | "ngon" | "ngonoutline" | "linedashed" | "drawimage9"
        | "bezier" | "bezier3" | "spline_basis" | "spline_bezier"
        | "text" | "drawfps"
        | "drawimage" | "drawimagerot" | "drawimageflipped"
        | "drawimagepart" | "drawimagepartex" | "drawtilemap"
        | "atlas_draw" | "atlas_draw_flipped" | "batch_draw")
}

/// `DIM p AS Spieler` legt die Variable an, aber kein Objekt -- sie ist NIL,
/// bis ein NEW kommt. In anderen BASICs (und bei STRUCT hier) entsteht das
/// Objekt mit der Deklaration, darum ist das die haeufigste NIL-Quelle.
pub(super) const NIL_NEW: &str = " -- die Variable ist noch NIL: eine Klasse muss mit NEW angelegt werden (p = NEW Klasse())";

/// Sagt diese Meldung "den Befehl hat dieser Bau nicht" (statt "falsche
/// Argumente")? Fuer `dhrt --fehlende`, das die kleinen Laufzeiten fragt.
/// Muss jede Form von `unknown_builtin_msg` und der Feature-Huellen treffen.
pub(crate) fn fehlt_im_bau(meldung: &str) -> bool {
    meldung.contains("noch nicht verfuegbar")
        || meldung.contains("ist ohne das Feature")
        || meldung.contains("das in diesem dhrt-Build fehlt")
        || meldung.starts_with("Unbekannter Befehl")
}

pub(super) fn unknown_builtin_msg(name: &str) -> String {
    // Hardware-/IoT-Module sind hinter Cargo-Features (serial/usb/bt/wifi) und im
    // Default-Build NICHT enthalten -- der Dispatch faellt dann hierher durch. Das
    // Builtin EXISTIERT (in dhrt implementiert), es fehlt nur im aktuellen Build.
    // Klare, handlungsleitende Meldung statt "noch nicht verfuegbar".
    if name.starts_with("phys2d_") || name.starts_with("phys3d_") {
        return format!("{}: diese Fassung von dhrt ist ohne das Feature `physik` gebaut",
                       name.to_uppercase());
    }
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
    // Ein Name, den das Befehlsverzeichnis gar nicht kennt, ist kein
    // "noch nicht verfuegbar" (so klingen nur die Bauten ohne Grafik/Ton, und
    // daran erkennen die Pruefsammlungen einen Bau ohne Fenster), sondern ein
    // unbekannter Befehl -- oft einer aus einem anderen BASIC.
    if !crate::compiler::is_known_builtin(name) {
        return format!("Unbekannter Befehl '{}' -- dhrt kennt ihn nicht (Tippfehler?){}{}",
                       name.to_uppercase(), crate::umstieg::befehl_hinweis(name),
                       crate::compiler::aehnliche_builtins(name));
    }
    if builtins::is_graphics_builtin(name) {
        format!("Grafik-Builtin '{}' im Rust-Kern noch nicht verfuegbar (Schritt 4)", name.to_uppercase())
    } else {
        format!("Builtin '{}' im Rust-Kern noch nicht verfuegbar", name.to_uppercase())
    }
}

pub(super) fn arg_truthy(a: &Arg) -> bool {
    match a {
        Arg::Int(i) => *i != 0,
        Arg::Val(Value::Bool(b)) => *b,
        Arg::None => false,
        _ => true,
    }
}

pub(super) fn arg_value(a: &Arg) -> Value {
    match a {
        Arg::None => Value::Nil,
        Arg::Int(i) => Value::Int(*i),
        Arg::Str(s) => Value::Str(Rc::new(String::from(s.as_ref()))),
        Arg::Val(v) => v.clone(),
        Arg::List(_) | Arg::Ints(_) | Arg::Call(..) => Value::Nil,
    }
}

/// Gibt es eine Methode dieses Namens fuer Text, Feld, MAP oder Tupel?
pub(crate) fn ist_container_methode(name: &str) -> bool {
    let n = name.to_lowercase();
    ["string", "array", "map", "tuple"].iter().any(|k| container_method(k, &n).is_some())
}

pub(super) fn container_kind(v: &Value) -> Option<&'static str> {
    match v {
        Value::Str(_) => Some("string"),
        Value::Array(_) => Some("array"),
        Value::Map(_) => Some("map"),
        Value::Tuple(_) => Some("tuple"),
        _ => None,
    }
}

pub(super) fn container_method(kind: &str, method: &str) -> Option<&'static str> {
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

pub(crate) fn load_index(arr: &Value, idx_vals: &[Value]) -> R<Value> {
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
        // `m["name"]` -- die Schreibweise, die man aus jeder anderen Sprache
        // kennt. Gleichwertig zu MAPGET, samt derselben Meldung.
        Value::Map(m) => {
            if idx_vals.len() != 1 { return Err("MAP-Zugriff braucht genau einen Schluessel: m[\"name\"]".into()); }
            let k = match &idx_vals[0] {
                Value::Str(k) => k.to_string(),
                v => return Err(format!("MAP-Schluessel muss STRING sein, erhalten {}", v.type_name())),
            };
            let m = m.borrow();
            m.get(&k).cloned().ok_or_else(|| format!(
                "Schluessel '{}' nicht in der MAP -- vorher mit MAPHAS(m, \"{}\") fragen oder MAPGETOR(m, \"{}\", vorgabe) nehmen",
                k, k, k))
        }
        Value::Nil => Err("Index-Zugriff auf NIL".into()),
        v => Err(format!("Index-Zugriff auf Nicht-Array ({})", v.type_name())),
    }
}

pub(crate) fn store_index(arr: &Value, idx_vals: &[Value], v: Value) -> R<()> {
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
        // `m["name"] = wert` -- gleichwertig zu MAPPUT.
        Value::Map(m) => {
            if idx_vals.len() != 1 { return Err("MAP-Zuweisung braucht genau einen Schluessel: m[\"name\"] = wert".into()); }
            let k = match &idx_vals[0] {
                Value::Str(k) => k.to_string(),
                x => return Err(format!("MAP-Schluessel muss STRING sein, erhalten {}", x.type_name())),
            };
            let vt = m.borrow().value_type.clone();
            let cv = crate::builtins::coerce_map_value(&v, &vt)?;
            m.borrow_mut().put(k, cv);
            Ok(())
        }
        Value::Nil => Err("Index-Zuweisung an NIL".into()),
        Value::Str(_) => Err("Index-Zuweisung an einen Text: Texte lassen sich nicht stellenweise aendern -- \
                              neu zusammensetzen, z.B. s = LEFT$(s, i) + \"x\" + MID$(s, i + 1)".into()),
        v => Err(format!("Index-Zuweisung an Nicht-Array ({})", v.type_name())),
    }
}

pub(super) fn eval_in(needle: &Value, hay: &Value) -> R<bool> {
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

pub(super) fn apply_slice(target: &Value, lo: Option<&Value>, hi: Option<&Value>) -> R<Value> {
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
            // Mit eigener Untergrenze zaehlen `von` und `bis` in den Indizes
            // des Feldes (a[-5:0] bei DIM a[-10 TO 10]); das Ergebnis ist eine
            // neue Liste ab 0, wie bei ARRAY_FILTER.
            let u = arr.unten_von(0);
            let (a, b) = if u == 0 {
                (to_idx(lo, 0)?.min(n).max(0), to_idx(hi, n)?.min(n).max(0))
            } else {
                let feld = |o: Option<&Value>, def: i64| -> R<i64> {
                    match o {
                        None => Ok(def),
                        Some(Value::Int(i)) => Ok(i.saturating_sub(u)),
                        Some(v) => Err(format!("Slice-Index muss INTEGER sein, erhalten {}", v.type_name())),
                    }
                };
                (feld(lo, 0)?.min(n).max(0), feld(hi, n)?.min(n).max(0))
            };
            let slice = if a >= b { Cells::Val(vec![]) } else { arr.cells.slice(a as usize, b as usize) };
            let len = slice.len() as i64;
            let mut new = DhArray::new(arr.element_type.clone(), vec![len], || Value::Nil);
            new.cells = slice;
            Ok(Value::Array(Rc::new(RefCell::new(new))))
        }
        v => Err(format!("Slice-Zugriff: Erwartet STRING oder ARRAY, erhalten {}", v.type_name())),
    }
}

pub(super) fn scene_type_ok(v: &Value, name: &str) -> bool {
    if name.contains("int") { matches!(v, Value::Int(_)) }
    else if name.contains("float") { matches!(v, Value::Float(_)) }
    else if name.contains("string") { matches!(v, Value::Str(_)) }
    else if name.contains("bool") { matches!(v, Value::Bool(_)) }
    else { false }
}

pub(super) fn infer_type(v: &Value) -> &'static str {
    match v {
        Value::Bool(_) => "boolean",
        Value::Int(_) => "integer",
        Value::Float(_) => "float",
        Value::Str(_) => "string",
        _ => "any",
    }
}

/// `a + b` fuer zwei Zahlen; `None` fuer alles andere. Getrennt von
/// `Vm::addieren`, damit ADD_STORE_* den haeufigsten Fall (`z = z + i`) ohne
/// Umweg ueber den Platz der Zeichenkette erledigt.
#[inline(always)]
pub(super) fn zahlen_addieren(a: &Value, b: &Value) -> Option<R<Value>> {
    Some(match (a, b) {
        (Value::Int(x), Value::Int(y)) =>
            x.checked_add(*y).map(Value::Int).ok_or_else(|| int_overflow_msg("+")),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Float(*x as f64 + y)),
        (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x + *y as f64)),
        _ => return None,
    })
}

/// Ein rechter Summand, bei dem `Text + b` sicher nicht scheitert und keinen
/// eigenen Code (OPERATOR einer Klasse) laufen laesst. Nur dann darf
/// ADD_STORE_* den Platz vorher leeren -- ein Fehler danach liesse die
/// Variable sonst leer zurueck, und ein TRY/CATCH saehe NIL statt des Texts.
pub(super) fn ist_schlicht(b: &Value) -> bool {
    matches!(b, Value::Str(_) | Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Nil)
}

/// Nimmt ein Platz dieses Typs einen Text ohne Umwandlung auf? Haelt er schon
/// einen, ist das zwar immer so -- die Pruefung macht es nur sichtbar.
pub(super) fn text_platz_typ(t: &str) -> bool { matches!(t, "string" | "any" | "") }

/// `platz = a + b` an Ort und Stelle (ADD_STORE_FIELD/_MEMBER/_INDEX, auch im
/// Maschinencode): haelt der Platz noch genau den Text `a` (derselbe Verweis)
/// und ist `b` schlicht, geht `a` -- nur ein zweiter Verweis -- zuerst weg;
/// haelt dann sonst niemand den Text, haengt `push_str` an, statt ihn zu
/// kopieren. Das Ergebnis ist dasselbe wie ADD + STORE: `Text + schlicht`
/// ruft keinen Code (kein OPERATOR) und kann nicht scheitern, und ein Text in
/// einem Platz, der schon einen hielt, braucht keine Umwandlung. Weil der
/// Platz nie geleert wird, bleibt bei keinem Fehler ein Loch zurueck.
/// Passt etwas nicht, kommt `a` unveraendert zurueck.
pub(crate) fn an_ort_anhaengen(platz: &mut Value, a: Value, b: &Value) -> Result<(), Value> {
    match (&a, &*platz) {
        (Value::Str(ra), Value::Str(rp)) if Rc::ptr_eq(ra, rp) && ist_schlicht(b) => {}
        _ => return Err(a),
    }
    drop(a);
    let Value::Str(links) = platz else { unreachable!() };
    // ein Zeichenindex (textindex.rs) haelt ein Weak -- erst loslassen
    crate::textindex::loslassen(links);
    match Rc::get_mut(links) {
        Some(s) => match b { Value::Str(r) => s.push_str(r), _ => s.push_str(&b.fmt()) },
        None => {
            let rechts: std::borrow::Cow<str> = match b {
                Value::Str(r) => std::borrow::Cow::Borrowed(r.as_str()),
                _ => std::borrow::Cow::Owned(b.fmt()),
            };
            let mut s = String::with_capacity(links.len() + rechts.len());
            s.push_str(links);
            s.push_str(&rechts);
            *links = Rc::new(s);
        }
    }
    Ok(())
}

/// Der Platz `c[idx]` fuer `an_ort_anhaengen`: ein Element eines Feldes, das
/// Texte aufnimmt, bzw. ein vorhandener Eintrag einer solchen MAP. Alles
/// andere (falscher Index, neuer Schluessel, Zahlenfeld ...) gibt `a` zurueck
/// -- dann rechnet der Aufrufer ADD + STORE_INDEX mit dessen Meldungen.
pub(crate) fn index_anhaengen(c: &Value, idx: &[Value], a: Value, b: &Value) -> Result<(), Value> {
    if !matches!(a, Value::Str(_)) || !ist_schlicht(b) { return Err(a); }
    match c {
        Value::Array(arr) => {
            let mut ab = arr.borrow_mut();
            if !text_platz_typ(&ab.element_type) || idx.len() > 8 { return Err(a); }
            let mut ints = [0i64; 8];
            for (z, ix) in ints.iter_mut().zip(idx) {
                match ix { Value::Int(i) => *z = *i, _ => return Err(a) }
            }
            let Ok(flat) = ab.flat_index(&ints[..idx.len()]) else { return Err(a) };
            match &mut ab.cells {
                Cells::Val(v) if flat < v.len() => an_ort_anhaengen(&mut v[flat], a, b),
                _ => Err(a),
            }
        }
        Value::Map(m) => {
            let [Value::Str(k)] = idx else { return Err(a) };
            let mut mb = m.borrow_mut();
            if matches!(mb.value_type.as_str(), "integer" | "float" | "boolean") { return Err(a); }
            match mb.get_mut(k) {
                Some(platz) => an_ort_anhaengen(platz, a, b),
                None => Err(a),
            }
        }
        _ => Err(a),
    }
}

pub(super) fn require_number(a: &Value, b: &Value, op: &str) -> R<()> {
    // Eine FUNCTION ohne Klammern ist nur ihr Name (`zwei + 1` statt
    // `zwei() + 1`) -- in anderen BASICs ruft der Name allein sie auf.
    if let Some(Value::FuncRef(n)) = [a, b].into_iter().find(|v| matches!(v, Value::FuncRef(_))) {
        return Err(format!(
            "Operator '{}': '{}' ist eine FUNCTION ohne Klammern, also nur ihr Name -- aufrufen mit {}()",
            op, n, n));
    }
    if !is_num(a) || !is_num(b) {
        return Err(format!("Operator '{}' erwartet Zahlen, erhalten {} / {}", op, a.type_name(), b.type_name()));
    }
    Ok(())
}

pub(super) fn int_pair2(stack: &mut Vec<Value>, op: &str) -> R<(i64, i64)> {
    let b = vm_pop(stack)?;
    let a = vm_pop(stack)?;
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => Ok((*x, *y)),
        _ => Err(format!("{} erwartet INTEGER, erhalten {} / {}", op, a.type_name(), b.type_name())),
    }
}

/// Partikel-Farbe pro Lebenszeit: optional Start->End-Verlauf + Fade.
#[cfg(feature = "graphics")]
pub(super) fn particle_color(start: i64, end: i64, has_end: bool, fade: bool, age: i32, lifetime: i32) -> i64 {
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
pub(super) fn module_op(op: char, a: &Value, b: &Value) -> Option<R<Value>> {
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
        Value::Vec2(..) | Value::Vec3(..) | Value::Vec4(..) | Value::Quat(..)
        | Value::Mat4(_) | Value::Geld(_));
    if !is_mod(a) && !is_mod(b) {
        return None;
    }
    let sf = |n: &Value| as_f64(n) as f32;     // Skalar als f32
    Some(match op {
        '+' => match (a, b) {
            (Value::Geld(x), Value::Geld(y)) => x.checked_add(*y).map(Value::Geld)
                .ok_or_else(|| int_overflow_msg("+")),
            (Value::Geld(_), n) | (n, Value::Geld(_)) if is_num(n) => Err(geld_mischt_nicht('+')),
            (Value::Vec2(ax, ay), Value::Vec2(bx, by)) => Ok(Value::Vec2(ax + bx, ay + by)),
            (Value::Vec3(ax, ay, az), Value::Vec3(bx, by, bz)) => Ok(Value::Vec3(ax + bx, ay + by, az + bz)),
            (Value::Vec4(ax, ay, az, aw), Value::Vec4(bx, by, bz, bw)) => Ok(Value::Vec4(ax + bx, ay + by, az + bz, aw + bw)),
            _ => Err("+ : inkompatible Vektor-Operanden (gleicher Vektor-Typ erwartet)".into()),
        },
        '-' => match (a, b) {
            (Value::Geld(x), Value::Geld(y)) => x.checked_sub(*y).map(Value::Geld)
                .ok_or_else(|| int_overflow_msg("-")),
            (Value::Geld(_), n) | (n, Value::Geld(_)) if is_num(n) => Err(geld_mischt_nicht('-')),
            (Value::Vec2(ax, ay), Value::Vec2(bx, by)) => Ok(Value::Vec2(ax - bx, ay - by)),
            (Value::Vec3(ax, ay, az), Value::Vec3(bx, by, bz)) => Ok(Value::Vec3(ax - bx, ay - by, az - bz)),
            (Value::Vec4(ax, ay, az, aw), Value::Vec4(bx, by, bz, bw)) => Ok(Value::Vec4(ax - bx, ay - by, az - bz, aw - bw)),
            _ => Err("- : inkompatible Vektor-Operanden (gleicher Vektor-Typ erwartet)".into()),
        },
        '*' => match (a, b) {
            // Menge mal Preis, Prozentsatz -- der Faktor darf eine Zahl
            // sein, gerechnet wird trotzdem ganzzahlig (geld.rs).
            (Value::Geld(w), n) | (n, Value::Geld(w)) if is_num(n) =>
                crate::geld::mal(*w, as_f64(n)).map(Value::Geld),
            (Value::Geld(_), Value::Geld(_)) =>
                Err("* : GELD mal GELD ergibt keinen Betrag (Euro mal Euro \
                     waeren Quadrat-Euro). Mit einer Zahl multiplizieren.".into()),
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
            // Betrag durch Betrag ist ein Verhaeltnis, also eine Zahl.
            (Value::Geld(x), Value::Geld(y)) => {
                if *y == 0 { Err("Division durch 0".into()) }
                else { Ok(Value::Float(*x as f64 / *y as f64)) }
            }
            (Value::Geld(w), n) if is_num(n) => crate::geld::durch(*w, as_f64(n)).map(Value::Geld),
            (n, Value::Geld(_)) if is_num(n) =>
                Err("/ : Zahl durch GELD ergibt keinen Betrag".into()),
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

/// Die Meldung, fuer die es den Typ ueberhaupt gibt.
pub(super) fn geld_mischt_nicht(op: char) -> String {
    format!("{} : GELD und eine gewoehnliche Zahl mischen sich nicht -- genau \
             dafuer gibt es den Typ. Entweder GELD_NEU(zahl) daraus machen \
             oder mit GELD_ZAHL(betrag) bewusst herausgehen.", op)
}

pub(super) fn int_overflow_msg(op: &str) -> String {
    format!("Ganzzahl-Ueberlauf bei '{}' (INTEGER ist 64-bit; Wertebereich ueberschritten)", op)
}

/// `a op b` fuer zwei SCHLICHTE Werte (Zahl, Text, Wahrheitswert) -- genau so,
/// wie die Befehle ADD/SUB/MUL/DIV/INT_DIV/MOD/POW es zur Laufzeit rechnen
/// (fuer solche Werte greift dort weder ein Modul- noch ein Klassen-Operator).
/// Der Compiler faltet damit Konstanten (`2 * 3`, `BREITE \ 2`); weil er
/// DIESELBEN Funktionen nimmt, kann das Ergebnis nicht von der Laufzeit
/// abweichen. `None` = nicht falten: ein anderer Operator, ein Wert, der
/// kein schlichter ist, ein Fehler (Ueberlauf, Division durch 0 -- der kommt
/// dann wie bisher zur Laufzeit) oder ein Ergebnis, das keine Konstante sein
/// kann (NaN/unendlich).
pub(crate) fn konstant_rechnen(op: &str, a: &Value, b: &Value) -> Option<Value> {
    match wert_rechnen(op, a, b)? {
        Value::Float(f) if !f.is_finite() => None,
        // `"x" * 100000000` gehoert nicht als 100-MB-Konstante in den Bytecode.
        Value::Str(s) if s.len() > 4096 => None,
        v => Some(v),
    }
}

/// `a op b` fuer zwei schlichte Werte mit den Regeln der Befehle -- ohne die
/// Grenzen des Faltens (fuer den Wertemodus des Maschinencodes, `jit::w_op`).
/// `None` bei einem Fehler: dann rechnet die VM und meldet ihn.
pub(crate) fn wert_rechnen(op: &str, a: &Value, b: &Value) -> Option<Value> {
    let schlicht = |v: &Value| matches!(v, Value::Int(_) | Value::Float(_) | Value::Str(_) | Value::Bool(_));
    if !schlicht(a) || !schlicht(b) { return None; }
    let (a, b) = (a.clone(), b.clone());
    let r = match op {
        "+" => match zahlen_addieren(&a, &b) {
            Some(r) => r,
            None if matches!(a, Value::Str(_)) || matches!(b, Value::Str(_)) => {
                let mut s = a.fmt();
                s.push_str(&b.fmt());
                Ok(Value::str_rc(s))
            }
            None => require_number(&a, &b, "+").and_then(|_| nn_add(a, b)),
        },
        "-" => match (&a, &b) {
            (Value::Int(x), Value::Int(y)) => x.checked_sub(*y).map(Value::Int).ok_or_else(|| int_overflow_msg("-")),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(*x as f64 - y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x - *y as f64)),
            _ => require_number(&a, &b, "-").and_then(|_| nn_arith(a, b, '-')),
        },
        "*" => match (&a, &b) {
            (Value::Int(x), Value::Int(y)) => x.checked_mul(*y).map(Value::Int).ok_or_else(|| int_overflow_msg("*")),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(*x as f64 * y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x * *y as f64)),
            _ => mul(a, b),
        },
        "/" => div(a, b),
        "\\" => int_div(a, b),
        "mod" => modulo(a, b),
        "^" => pow(a, b),
        _ => return None,
    };
    match r {
        Ok(v @ (Value::Int(_) | Value::Float(_) | Value::Str(_) | Value::Bool(_))) => Some(v),
        _ => None,
    }
}

/// `-a` fuer eine Zahl, wie NEG zur Laufzeit.
pub(crate) fn konstant_negieren(a: &Value) -> Option<Value> {
    if !matches!(a, Value::Int(_) | Value::Float(_)) { return None; }
    neg(a.clone()).ok()
}

pub(super) fn nn_add(a: Value, b: Value) -> R<Value> {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) =>
            x.checked_add(*y).map(Value::Int).ok_or_else(|| int_overflow_msg("+")),
        _ => Ok(Value::Float(as_f64(&a) + as_f64(&b))),
    }
}

pub(super) fn nn_arith(a: Value, b: Value, op: char) -> R<Value> {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            let r = match op { '-' => x.checked_sub(*y), '*' => x.checked_mul(*y), _ => unreachable!() };
            r.map(Value::Int).ok_or_else(|| int_overflow_msg(&op.to_string()))
        }
        _ => { let (x, y) = (as_f64(&a), as_f64(&b)); Ok(Value::Float(match op { '-' => x - y, '*' => x * y, _ => unreachable!() })) }
    }
}

pub(super) fn mul(a: Value, b: Value) -> R<Value> {
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
pub(super) fn round_audio(f: f64) -> f64 { (f * 1_000_000.0).round() / 1_000_000.0 }

/// Baut aus den Werten eines Array-Literals `[a, b, c]` ein 1D-DhArray.
/// Element-Typ wird aus den Werten hergeleitet (wie ein homogenes DH-Array):
/// nur Ganzzahlen -> integer; Zahlen mit mind. einem Float -> float (Ints
/// werden hochgezogen); nur Strings -> string; nur Wahrheitswerte -> boolean;
/// gemischt -> any (generisches Value-Backing).
/// Ein leeres, eindimensionales Feld zum Typ `array:T` -- der Startwert von
/// `DIM a AS ARRAY OF T` ohne Groesse.
/// Was NEW fuer ein Objekt einer Klasse braucht (`ClassInfo::vorlage`,
/// `Vm::vorlage_bauen`): der Klassenname EINMAL als `Rc<str>` (vorher je
/// Objekt neu), die Lage, je Platz der Anfangswert und die SUB Init.
pub(crate) struct ObjektVorlage {
    pub(super) klasse: Rc<str>,
    pub(super) layout: Rc<crate::value::Layout>,
    /// Anfangswerte; auf den Plaetzen in `frisch` nur Platzhalter.
    pub(super) werte: Vec<Value>,
    /// Plaetze, deren Wert jedes Objekt fuer sich braucht.
    pub(super) frisch: Vec<(usize, Frisch)>,
    /// `resolve_method(klasse, "init")` -- die Klasse oder eine Vorfahrin.
    /// Ein Zeiger wie in `Instr::merk`: das Programm aendert sich nach dem
    /// Laden nicht mehr.
    pub(super) init: Option<*const Func>,
}

/// Ein Anfangswert, der je Objekt neu entsteht.
pub(super) enum Frisch {
    /// STRUCT ... LAYOUT C: Puffer mit so vielen Bytes.
    Puffer(usize),
    /// Feld mit Groesse, dessen Elemente teilbar sind: die Vorlage kopieren.
    FeldKopie(DhArray),
    /// Feld mit Groesse, dessen Elemente selbst je Objekt neu entstehen.
    Feld(String, Vec<i64>, Vec<i64>),
    /// Ein STRUCT als Feld.
    Struktur(String),
    /// Ein Anfangswert mit eigenem Speicher (MAT4, Tupel, leeres Feld ...).
    Wert(String),
}

/// Darf ein Anfangswert zwischen Objekten geteilt werden? Werte ohne Speicher
/// und Text (der wird nur an Ort und Stelle veraendert, wenn niemand sonst
/// ihn haelt -- `Rc::get_mut`).
pub(super) fn teilbar(v: &Value) -> bool {
    matches!(v, Value::Nil | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::Str(_)
        | Value::Vec2(..) | Value::Geld(_) | Value::Vec3(..) | Value::Vec4(..) | Value::Quat(..))
}

pub(super) fn leeres_feld(ty: &str) -> Value {
    let elem = ty.strip_prefix("array:").unwrap_or("any").to_string();
    Value::Array(Rc::new(RefCell::new(DhArray::new(elem, vec![0], || Value::Nil))))
}

/// `a + b` fuer zwei Felder: ein NEUES Feld mit den Elementen beider
/// (`a = a + [x]` haengt an). Nur eindimensional; der Elementtyp bleibt,
/// wenn beide denselben haben, sonst bestimmen ihn die Werte.
pub(super) fn felder_verbinden(x: &Rc<RefCell<DhArray>>, y: &Rc<RefCell<DhArray>>) -> R<Value> {
    let (a, b) = (x.borrow(), y.borrow());
    if a.dims.len() != 1 || b.dims.len() != 1 {
        return Err("'+' verbindet nur eindimensionale Felder".into());
    }
    let (na, nb) = (a.dims[0] as usize, b.dims[0] as usize);
    let mut vals = Vec::with_capacity(na + nb);
    for i in 0..na { vals.push(a.cells.get(i)); }
    for i in 0..nb { vals.push(b.cells.get(i)); }
    let et = if a.element_type == b.element_type { a.element_type.clone() }
        else if na == 0 { b.element_type.clone() }
        else if nb == 0 { a.element_type.clone() }
        else {
            let zahl = |t: &str| t == "integer" || t == "float";
            if zahl(&a.element_type) && zahl(&b.element_type) { "float".to_string() }
            else if a.element_type == "any" || b.element_type == "any" { String::new() }
            else {
                return Err(format!("'+' verbindet nur Felder mit gleichem Elementtyp -- hier ARRAY OF {} und ARRAY OF {}",
                                   typ_lesbar(&a.element_type), typ_lesbar(&b.element_type)));
            }
        };
    if et.is_empty() || et == "any" { return Ok(array_literal(vals)); }
    let mut neu = DhArray::new(et.clone(), vec![vals.len() as i64], || Value::Nil);
    for (i, v) in vals.into_iter().enumerate() {
        neu.cells.set(i, coerce(v, &et, "Feld verbinden")?);
    }
    Ok(Value::Array(Rc::new(RefCell::new(neu))))
}

pub(super) fn array_literal(vals: Vec<Value>) -> Value {
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
    let arr = DhArray { element_type: etype.to_string(), dims: vec![n], strides: vec![1], cells, unten: Vec::new() };
    Value::Array(Rc::new(RefCell::new(arr)))
}

/// `/` liefert IMMER eine Kommazahl (seit 2026-09-24, Entscheidung des
/// Nutzers): vorher war `480 / 2` INTEGER und `7 / 2` FLOAT -- ein Typ, der
/// am Wert hing, und den weder ein Uebersetzer noch ein Leser vorhersehen
/// kann. Ganzzahlig teilt `\`.
pub(super) fn div(a: Value, b: Value) -> R<Value> {
    require_number(&a, &b, "/")?;
    let y = as_f64(&b);
    if y == 0.0 { return Err("Division durch 0".into()); }
    Ok(Value::Float(as_f64(&a) / y))
}

pub(super) fn int_div(a: Value, b: Value) -> R<Value> {
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => {
            if *y == 0 { return Err("Integer-Division durch 0".into()); }
            // Abgerundet wie FLOOR (seit 2026.38): -7 \ 2 = -4. So gilt mit
            // MOD (Vorzeichen des Teilers) immer (a \ b) * b + a MOD b = a.
            let q = x.checked_div(*y).ok_or_else(|| int_overflow_msg("\\"))?;
            Ok(Value::Int(if x % y != 0 && (*x < 0) != (*y < 0) { q - 1 } else { q }))
        }
        // Review-Fund: dieser Zweig faengt JEDEN nicht-Int/Int-Fall ab, nicht
        // nur FLOAT -- die Meldung nannte faelschlich immer "FLOAT" auch fuer
        // BOOLEAN/STRING-Operanden.
        _ => Err(format!("\\ erwartet INTEGER, erhalten {} / {}", a.type_name(), b.type_name())),
    }
}

pub(super) fn modulo(a: Value, b: Value) -> R<Value> {
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

pub(super) fn pow(a: Value, b: Value) -> R<Value> {
    require_number(&a, &b, "^")?;
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) if *y >= 0 => {
            if *y > u32::MAX as i64 { return Err(int_overflow_msg("^")); }
            x.checked_pow(*y as u32).map(Value::Int).ok_or_else(|| int_overflow_msg("^"))
        }
        _ => Ok(Value::Float(as_f64(&a).powf(as_f64(&b)))),
    }
}

pub(super) fn neg(v: Value) -> R<Value> {
    match v {
        Value::Int(i) => i.checked_neg().map(Value::Int).ok_or_else(|| int_overflow_msg("-")),
        Value::Float(f) => Ok(Value::Float(-f)),
        Value::Geld(w) => w.checked_neg().map(Value::Geld)
            .ok_or_else(|| int_overflow_msg("-")),
        _ => Err("Unaeres '-' erwartet Zahl".into()),
    }
}

pub(crate) fn cmp(a: &Value, b: &Value, op: char) -> R<bool> {
    let ord = if let (Value::Geld(x), Value::Geld(y)) = (a, b) {
        // Betraege vergleichen sich EXAKT -- das ist einer der Gruende,
        // warum es den Typ gibt.
        Some(x.cmp(y))
    } else if is_num(a) && is_num(b) {
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
pub(super) fn input_prompt(p: &str) -> String {
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
pub(super) fn read_input_line() -> String {
    use std::io::BufRead;
    let mut line = String::new();
    let _ = std::io::stdin().lock().read_line(&mut line);
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }
    line
}

/// INPUT-Rohwert auf den Ziel-Typ coercen (Semantik 1:1 aus interpreter).
pub(super) fn coerce_input(raw: &str, ty: &str) -> R<Value> {
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

#[inline(never)]
pub(crate) fn coerce(value: Value, target: &str, ctx: &str) -> R<Value> {
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
            Value::FuncRef(_) | Value::BoundMethod(_) | Value::Closure(_) | Value::Nil => Ok(value),
            _ => Err(format!("{}: Erwartet FUNCREF, erhalten {}", ctx, value.type_name())),
        },
        // GELD ist streng, anders als die uebrigen Modul-Typen: eine Zahl
        // stillschweigend in einer GELD-Variablen abzulegen wuerde genau
        // die Trennung aufweichen, fuer die es den Typ gibt.
        "geld" => match value {
            Value::Geld(_) | Value::Nil => Ok(value),
            v if is_num(&v) => Err(format!(
                "{}: Erwartet GELD, erhalten {} -- mit GELD_NEU({}) einen Betrag daraus machen",
                ctx, v.type_name(), v.fmt())),
            v => Err(format!("{}: Erwartet GELD, erhalten {}", ctx, v.type_name())),
        },
        // Modul-Typen -> Durchreichen (Referenz-Typen); eine MAP prueft
        // coerce_map wie ein Feld coerce_array; eine Klasse
        // nimmt nur NIL und Instanzen ihrer selbst oder einer Unterklasse.
        _ => match target.strip_prefix("array:") {
            Some(elem) => coerce_array(value, elem, ctx),
            None if target.starts_with("map:") => coerce_map(value, &target[4..], ctx),
            None => {
                crate::value::klasse_pruefen(&value, target, ctx)?;
                Ok(value)
            }
        },
    }
}

/// Einen internen Typnamen so schreiben, wie er im Quelltext steht
/// (`array:integer` -> `ARRAY OF INTEGER`). Gegenstueck zu
/// `Compiler::typ_klartext` -- die interne Schreibweise sagt einem Nutzer nichts.
pub(super) fn typ_lesbar(t: &str) -> String {
    if let Some(e) = t.strip_prefix("array:") { return format!("ARRAY OF {}", typ_lesbar(e)); }
    if let Some(e) = t.strip_prefix("map:") { return format!("MAP OF {}", typ_lesbar(e)); }
    t.to_uppercase()
}

/// Zuweisung an ein MAP-Ziel (`DIM m AS MAP OF Hund : m = ...`), nach
/// denselben Regeln wie `coerce_array`. Bis 2026.34 reichte `coerce` jede
/// MAP durch: eine MAP OF STRING ging an eine MAP OF INTEGER, und ein
/// Literal mit einer Katze an eine MAP OF Hund.
///
/// * Gleicher Werttyp oder der einer Unterklasse: passt (die MAP behaelt
///   ihren Werttyp, jeder Schreibzugriff prueft ihn).
/// * Eine MAP ohne Werttyp (ein Literal mit Objekten oder gemischten
///   Werten, `{}`): jeder Wert wird geprueft. Gehoert sie sonst niemandem,
///   nimmt sie den Werttyp an. Eine frische MAP von Ganzzahlen wird so auch
///   zu einer von Kommazahlen.
/// * Sonst: Fehler.
#[inline(never)]
pub(super) fn coerce_map(value: Value, vt: &str, ctx: &str) -> R<Value> {
    if matches!(value, Value::Nil) { return Ok(value); }
    let ziel = format!("MAP OF {}", typ_lesbar(vt));
    let m = match &value {
        Value::Map(m) => m,
        _ => return Err(format!("{}: Erwartet {}, erhalten {}", ctx, ziel, value.type_name())),
    };
    if vt.is_empty() || vt == "any" { return Ok(value); }
    let ist = m.borrow().value_type.clone();
    if ist == vt || crate::value::klasse_erbt(&ist, vt) { return Ok(value); }
    let frisch = Rc::strong_count(m) == 1;
    if ist == "any" || (frisch && ist == "integer" && vt == "float") {
        let schluessel: Vec<String> = m.borrow().entries().keys().cloned().collect();
        let mut gepasst = Vec::with_capacity(schluessel.len());
        for k in &schluessel {
            let v = m.borrow().get(k).cloned().unwrap_or(Value::Nil);
            gepasst.push(coerce(v, vt, &format!("{}: Wert '{}'", ctx, k))?);
        }
        if frisch {
            let mut mb = m.borrow_mut();
            mb.value_type = vt.to_string();
            for (k, v) in schluessel.iter().zip(gepasst) {
                if let Some(platz) = mb.get_mut(k) { *platz = v; }
            }
        }
        return Ok(value);
    }
    Err(format!("{}: Erwartet {}, erhalten MAP OF {}", ctx, ziel, typ_lesbar(&ist)))
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
#[inline(never)]
pub(super) fn coerce_array(value: Value, elem: &str, ctx: &str) -> R<Value> {
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
            // Auch der Speicher wie bei `DIM a[n] AS INTEGER`: sonst landet
            // jedes ARRAY_PUSH in Werten (Cells::Val), und der Maschinencode
            // darf so ein Feld nur lesen -- jede Schleife, die hineinschreibt,
            // lief im Wertemodus (gemessen 39 statt 1 ns je `a[i] = x`).
            match elem {
                "integer" => a.cells = Cells::Int(Vec::new()),
                "float" => a.cells = Cells::Float(Vec::new()),
                _ => {}
            }
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
    if ist == elem { return Ok(value); }
    // Ein Feld einer Unterklasse passt an ein Feld der Vorfahrin: es behaelt
    // seinen eigenen Elementtyp, und den prueft jeder Schreibzugriff -- ueber
    // den Namen der Vorfahrin kommt keine Katze in ein Feld von Hunden.
    if crate::value::klasse_erbt(&ist, elem) { return Ok(value); }
    if ist == "any" { return any_feld_annehmen(value, elem, ctx); }
    Err(format!("{}: Erwartet {}, erhalten ARRAY OF {}", ctx, ziel, typ_lesbar(&ist)))
}

/// Ein Feld ohne Elementtyp (ARRAY OF ANY, etwa ein Literal mit Objekten
/// `[NEW Hund()]`) an ein `ARRAY OF elem`: jedes Element muss passen. Bis
/// 2026-10-23 ging es ungeprueft durch -- `DIM h AS ARRAY OF Hund : h =
/// [NEW Katze()]` lief, und die Katze fiel erst viel spaeter auf.
///
/// Gehoert das Feld sonst niemandem (ein Literal von eben), bekommt es den
/// Elementtyp des Ziels; dann prueft jeder spaetere Schreibzugriff. Ein
/// geteiltes Feld (hinter einem anderen Namen ARRAY OF ANY) bleibt, wie es
/// ist -- der andere Name darf weiter alles hineinlegen.
#[cold]
pub(super) fn any_feld_annehmen(value: Value, elem: &str, ctx: &str) -> R<Value> {
    let Value::Array(arr) = &value else { return Ok(value) };
    let frisch = Rc::strong_count(arr) == 1;
    let werte = arr.borrow().cells.to_values();
    let mut gepasst = Vec::with_capacity(werte.len());
    for (i, v) in werte.into_iter().enumerate() {
        gepasst.push(coerce(v, elem, &format!("{}: Element {}", ctx, i))?);
    }
    if frisch {
        let mut a = arr.borrow_mut();
        let mut neu = DhArray::new(elem.to_string(), a.dims.clone(), || Value::Nil);
        for (i, v) in gepasst.into_iter().enumerate() { neu.cells.set(i, v); }
        *a = neu;
    }
    Ok(value)
}

/// Platz des Feldes `name` im Objekt, ueber den Merkplatz des Befehls: hat
/// das Objekt die Lage des letzten Treffers, ohne Suche; sonst gesucht und
/// gemerkt. Nur fuer FELDER -- wer eine PROPERTY meint, fragt vorher.
#[inline]
pub(super) fn feld_platz(instr: &crate::model::Instr, obj: &Instance, name: &str) -> Option<usize> {
    let (lage, platz) = instr.merk.get();
    if std::ptr::eq(Rc::as_ptr(&obj.layout), lage) { return Some(platz); }
    let i = obj.platz(name)?;
    instr.merk.set((Rc::as_ptr(&obj.layout), i));
    Some(i)
}

thread_local! {
    pub(super) static LEERE_LAGE: Rc<crate::value::Layout> = Rc::new(crate::value::Layout::default());
}

/// CALL_METHOD auf eine Methode mit BYREF-Parametern: der Compiler kannte die
/// Klasse nicht (`DIM x` ohne Typ, Feldelement ohne Typ), konnte die Argumente
/// also nicht erfassen. Still als BYVAL zu laufen hiesse, die Aenderungen zu
/// verlieren -- darum ein Fehler mit dem Weg hinaus.
pub(super) fn byref_ohne_typ(m: &Func, method: &str, cn: &str, argc: usize) -> Result<(), String> {
    if m.param_byref.iter().take(argc).any(|&b| b) {
        let k = crate::namensraum::lesbar_text(cn).to_uppercase();
        return Err(format!(
            "{}.{} hat BYREF-Parameter, aber beim Uebersetzen war die Klasse des Objekts \
             nicht bekannt -- die Werte kaemen nicht zurueck. Das Objekt in einer \
             Variablen mit Typ halten (DIM x AS {}) und darueber aufrufen.", k, method, k));
    }
    Ok(())
}

#[cfg(test)]
mod anhaengen_tests {
    use super::*;

    fn text(s: &str) -> Value { Value::Str(Rc::new(s.to_string())) }
    fn str_von(v: &Value) -> &Rc<String> { match v { Value::Str(r) => r, _ => panic!("kein Text") } }

    #[test]
    fn haengt_an_ort_und_stelle_an() {
        let mut platz = Value::Str(Rc::new(String::with_capacity(64)));
        let vorher = Rc::as_ptr(str_von(&platz));
        for b in [text("ab"), Value::Int(7), Value::Float(0.5), Value::Bool(true), Value::Nil] {
            let a = platz.clone();
            assert!(an_ort_anhaengen(&mut platz, a, &b).is_ok());
        }
        assert_eq!(str_von(&platz).as_str(), format!("ab70.5TRUE{}", Value::Nil.fmt()));
        // Kein neuer Text: derselbe Speicher, nur laenger.
        assert_eq!(Rc::as_ptr(str_von(&platz)), vorher);
    }

    #[test]
    fn ein_zweiter_halter_behaelt_seinen_text() {
        let mut platz = text("abc");
        let kopie = platz.clone();
        let a = platz.clone();
        assert!(an_ort_anhaengen(&mut platz, a, &text("d")).is_ok());
        assert_eq!(str_von(&platz).as_str(), "abcd");
        assert_eq!(str_von(&kopie).as_str(), "abc");
    }

    #[test]
    fn rechts_derselbe_text() {
        let mut platz = text("ab");
        let (a, b) = (platz.clone(), platz.clone());
        assert!(an_ort_anhaengen(&mut platz, a, &b).is_ok());
        assert_eq!(str_von(&platz).as_str(), "abab");
    }

    #[test]
    fn passt_nicht_gibt_links_zurueck_und_laesst_den_platz() {
        // Der Platz haelt einen anderen Text (der Ausdruck hat ihn ersetzt).
        let mut platz = text("neu");
        let a = text("alt");
        let zurueck = an_ort_anhaengen(&mut platz, a, &text("!")).unwrap_err();
        assert_eq!(str_von(&zurueck).as_str(), "alt");
        assert_eq!(str_von(&platz).as_str(), "neu");
        // Rechts ein Wert, dessen `+` Code laufen lassen koennte.
        let mut platz = text("x");
        let a = platz.clone();
        let feld = Value::Array(Rc::new(RefCell::new(DhArray::new("any".into(), vec![1], || Value::Nil))));
        assert!(an_ort_anhaengen(&mut platz, a, &feld).is_err());
        assert_eq!(str_von(&platz).as_str(), "x");
        // Zahlen sind kein Fall fuer diesen Weg.
        let mut platz = Value::Int(1);
        assert!(an_ort_anhaengen(&mut platz, Value::Int(1), &Value::Int(2)).is_err());
    }

    #[test]
    fn zeichenindex_haelt_nicht_auf() {
        // Ein grosser Text mit Zeichenindex (Weak darauf) haengt trotzdem an.
        let mut platz = Value::Str(Rc::new("ä".repeat(200_000)));
        crate::textindex::anmelden(str_von(&platz));
        crate::textindex::anmelden(str_von(&platz));
        let vorher = Rc::as_ptr(str_von(&platz));
        let a = platz.clone();
        assert!(an_ort_anhaengen(&mut platz, a, &text("z")).is_ok());
        assert_eq!(Rc::as_ptr(str_von(&platz)), vorher, "hat kopiert statt angehaengt");
        assert!(str_von(&platz).ends_with("äz"));
    }

    #[test]
    fn element_und_map_eintrag() {
        let arr = DhArray::new("string".into(), vec![2, 2], || Value::Str(Rc::new(String::new())));
        let c = Value::Array(Rc::new(RefCell::new(arr)));
        store_index(&c, &[Value::Int(1), Value::Int(0)], text("a")).unwrap();
        let a = load_index(&c, &[Value::Int(1), Value::Int(0)]).unwrap();
        assert!(index_anhaengen(&c, &[Value::Int(1), Value::Int(0)], a, &text("b")).is_ok());
        assert_eq!(str_von(&load_index(&c, &[Value::Int(1), Value::Int(0)]).unwrap()).as_str(), "ab");
        // Falscher Index: zurueck, ADD + STORE_INDEX meldet es.
        assert!(index_anhaengen(&c, &[Value::Int(5), Value::Int(0)], text("ab"), &text("b")).is_err());
        assert!(index_anhaengen(&c, &[Value::Int(1)], text("ab"), &text("b")).is_err());

        let m = Value::Map(Rc::new(RefCell::new(DhMap::new("string".into()))));
        store_index(&m, &[text("k")], text("m")).unwrap();
        let a = load_index(&m, &[text("k")]).unwrap();
        assert!(index_anhaengen(&m, &[text("k")], a, &text("n")).is_ok());
        assert_eq!(str_von(&load_index(&m, &[text("k")]).unwrap()).as_str(), "mn");
        // Ein neuer Schluessel wird hier nicht angelegt.
        assert!(index_anhaengen(&m, &[text("neu")], text("x"), &text("y")).is_err());
        assert!(load_index(&m, &[text("neu")]).is_err());
        // Ein Zahlenfeld ist kein Platz fuer Text.
        let z = Value::Array(Rc::new(RefCell::new(DhArray::new("integer".into(), vec![1], || Value::Int(0)))));
        assert!(index_anhaengen(&z, &[Value::Int(0)], text("1"), &text("x")).is_err());
    }
}

/// Ein Zahl-Argument (INTEGER oder FLOAT) als Kommazahl -- fuer die
/// Font-Masse (FONT_BASELINE, FONT_GLYPH_BOX).
#[cfg(feature = "graphics")]
pub(super) fn zahl_arg(a: &[Value], i: usize, f: &str) -> R<f64> {
    match a.get(i) {
        Some(Value::Int(n)) => Ok(*n as f64),
        Some(Value::Float(x)) => Ok(*x),
        _ => Err(format!("{}: erwartet eine Zahl (Arg {})", f, i + 1)),
    }
}
