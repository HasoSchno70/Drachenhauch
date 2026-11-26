//! Argumente der Befehle pruefen -- EINE Regel und EIN Wortlaut fuer alle
//! Familien.
//!
//! Bis 2026.53 hatte jede `try_*`-Familie ihre eigenen Helfer (`need_*` in
//! `builtins.rs`, `bi_*` in `vm.rs`, `gi`/`gs`/`gb` in `vm_grafik.rs`, zwei
//! `gi` in `vm_gui.rs`, `z_int` in `vm_system.rs` ...), und dieselbe Frage
//! hatte je Modul eine andere Antwort: `HTTP_TIMEOUT(10 / 2)` war ein Fehler,
//! `MID$(s, 4 / 2, 2)` nicht; `TEXT_BOLD(1)` galt still als FALSE,
//! `GPIO_WRITE(chip, pin, 1)` als TRUE; die Meldungen hatten fuenf Formen
//! (`docs/intern/inkonsistenzen-runtime.md`, A1/A2). Die Helfer der Familien
//! rufen jetzt diese Funktionen.
//!
//! Die Regeln:
//! - **INTEGER** (`int`): INTEGER oder eine Kommazahl ohne Nachkommastellen
//!   -- dieselbe Regel wie eine Zuweisung an eine INTEGER-Variable. Sonst ein
//!   Fehler, der den Weg nennt (`INT`, `ROUND`, `\`).
//! - **Koordinate** (`koord`): jede Zahl; Nachkommastellen fallen weg (zur
//!   Null hin). So zeichnet die Grafik seit jeher (`BOX(w / 2, ...)`), und ein
//!   Bild darf sich nicht um einen Punkt verschieben.
//! - **gerundet** (`int_gerundet`): jede endliche Zahl, gerundet -- fuer
//!   Werte, die typisch ausgerechnet werden (RGB-Anteile, Dauern).
//! - **Zahl** (`num`), **Text** (`text`).
//! - **Schalter** (`schalter`): an/aus-Optionen nehmen BOOLEAN oder eine Zahl
//!   (0 = aus, alles andere = an), wie man es von Arduino und BASIC kennt.
//!   Etwas anderes ist ein Fehler -- frueher galt es still als FALSE.
//!   Ein WahrheitsWERT, der gespeichert wird (`JSON_SET_BOOL`,
//!   `SAVE_SET_BOOL`, `ECS_ADD_BOOL`), bleibt streng BOOLEAN (`boolean`).
//!
//! Der Wortlaut ist immer `NAME: erwartet X (Arg n), erhalten Y` -- ohne
//! `(Arg n)`, wo die Stelle nicht bekannt ist; ein fehlendes Argument heisst
//! `NAME: fehlendes Argument n`. Mit dem Doppelpunkt findet
//! `vm_helfer::mit_signatur` den Befehl und haengt seine Form an.

use crate::value::Value;

type R<T> = Result<T, String>;

/// Was ein Wert ist, fuer "erhalten ...": bei einer Kommazahl samt Wert
/// (`FLOAT 2.5`), sonst der Typname.
fn erhalten(v: &Value) -> String {
    match v {
        Value::Float(f) => format!("FLOAT {}", crate::value::fmt_float(*f)),
        andere => andere.type_name().to_string(),
    }
}

/// `NAME: erwartet WAS (Arg n), erhalten TYP` -- die eine Form.
pub(crate) fn erwartet(fn_: &str, was: &str, nr: Option<usize>, v: &Value) -> String {
    match nr {
        Some(n) => format!("{}: erwartet {} (Arg {}), erhalten {}", fn_, was, n, erhalten(v)),
        None => format!("{}: erwartet {}, erhalten {}", fn_, was, erhalten(v)),
    }
}

/// `NAME: fehlendes Argument n`.
pub(crate) fn fehlt(fn_: &str, nr: usize) -> String {
    format!("{}: fehlendes Argument {}", fn_, nr)
}

/// Ein Text, wo eine Zahl stehen muss: der Hinweis aus `umstieg.rs` (Farbe
/// als Hex-Text, Tastenname, Themenname) haengt hinten an.
fn text_statt_zahl(fn_: &str, was: &str, nr: Option<usize>, v: &Value) -> String {
    let mut m = erwartet(fn_, was, nr, v);
    if let Value::Str(s) = v { m.push_str(&crate::umstieg::text_statt_zahl(fn_, s)); }
    m
}

/// INTEGER: ganze Zahl oder Kommazahl ohne Nachkommastellen.
pub(crate) fn int(v: &Value, fn_: &str, nr: Option<usize>) -> R<i64> {
    match v {
        Value::Int(i) => Ok(*i),
        Value::Float(f) => crate::builtins::glatt_int(*f).ok_or_else(||
            format!("{}{}", erwartet(fn_, "INTEGER", nr, v), crate::builtins::KEIN_GANZ_HINWEIS)),
        _ => Err(text_statt_zahl(fn_, "INTEGER", nr, v)),
    }
}

/// Eine Grafik-Zahl (Koordinate, Groesse, Handle): Nachkommastellen fallen
/// zur Null hin weg -- die Regel der Zeichenbefehle seit GameBasic.
#[cfg_attr(not(feature = "graphics"), allow(dead_code))] // nur die Grafik
pub(crate) fn koord(v: &Value, fn_: &str, nr: Option<usize>) -> R<i64> {
    match v {
        Value::Int(i) => Ok(*i),
        Value::Float(f) => Ok(*f as i64),
        _ => Err(text_statt_zahl(fn_, "Zahl", nr, v)),
    }
}

/// Jede endliche Zahl, gerundet.
pub(crate) fn int_gerundet(v: &Value, fn_: &str, nr: Option<usize>) -> R<i64> {
    match v {
        Value::Int(i) => Ok(*i),
        Value::Float(f) if f.is_finite() => Ok(f.round() as i64),
        _ => Err(text_statt_zahl(fn_, "eine endliche Zahl", nr, v)),
    }
}

/// Eine Zahl (INTEGER oder FLOAT) als Kommazahl.
pub(crate) fn num(v: &Value, fn_: &str, nr: Option<usize>) -> R<f64> {
    match v {
        Value::Int(i) => Ok(*i as f64),
        Value::Float(f) => Ok(*f),
        _ => Err(text_statt_zahl(fn_, "Zahl", nr, v)),
    }
}

/// Ein Text.
pub(crate) fn text<'a>(v: &'a Value, fn_: &str, nr: Option<usize>) -> R<&'a str> {
    match v {
        Value::Str(s) => Ok(s),
        _ => Err(erwartet(fn_, "STRING", nr, v)),
    }
}

/// Ein Schalter (an/aus-Option): BOOLEAN oder Zahl, 0 = aus.
pub(crate) fn schalter(v: &Value, fn_: &str, nr: Option<usize>) -> R<bool> {
    match v {
        Value::Bool(b) => Ok(*b),
        Value::Int(i) => Ok(*i != 0),
        Value::Float(f) => Ok(*f != 0.0),
        _ => Err(erwartet(fn_, "BOOLEAN oder Zahl", nr, v)),
    }
}

/// Ein Wahrheitswert, der als solcher gespeichert wird: nur BOOLEAN.
pub(crate) fn boolean(v: &Value, fn_: &str, nr: Option<usize>) -> R<bool> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err(erwartet(fn_, "BOOLEAN", nr, v)),
    }
}

// ---- dieselben Regeln fuer "Argument i aus der Liste a" (i ab 0) ----------

fn an<'a>(a: &'a [Value], i: usize, fn_: &str) -> R<&'a Value> {
    a.get(i).ok_or_else(|| fehlt(fn_, i + 1))
}

pub(crate) fn a_int(a: &[Value], i: usize, fn_: &str) -> R<i64> { int(an(a, i, fn_)?, fn_, Some(i + 1)) }
#[cfg_attr(not(feature = "graphics"), allow(dead_code))] // nur die Grafik
pub(crate) fn a_koord(a: &[Value], i: usize, fn_: &str) -> R<i64> { koord(an(a, i, fn_)?, fn_, Some(i + 1)) }
pub(crate) fn a_num(a: &[Value], i: usize, fn_: &str) -> R<f64> { num(an(a, i, fn_)?, fn_, Some(i + 1)) }
pub(crate) fn a_text<'a>(a: &'a [Value], i: usize, fn_: &str) -> R<&'a str> { text(an(a, i, fn_)?, fn_, Some(i + 1)) }
pub(crate) fn a_schalter(a: &[Value], i: usize, fn_: &str) -> R<bool> { schalter(an(a, i, fn_)?, fn_, Some(i + 1)) }

/// Ein optionaler Schalter: fehlt er, gilt `vorgabe`.
pub(crate) fn a_schalter_oder(a: &[Value], i: usize, vorgabe: bool, fn_: &str) -> R<bool> {
    match a.get(i) { None => Ok(vorgabe), Some(v) => schalter(v, fn_, Some(i + 1)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_nimmt_glatte_kommazahl() {
        assert_eq!(int(&Value::Float(5.0), "X", Some(1)), Ok(5));
        let e = int(&Value::Float(2.5), "X", Some(2)).unwrap_err();
        assert!(e.starts_with("X: erwartet INTEGER (Arg 2), erhalten FLOAT 2.5 -- keine ganze Zahl"), "{}", e);
        assert_eq!(int(&Value::Str(std::rc::Rc::new("a".to_string())), "X", None).unwrap_err(), "X: erwartet INTEGER, erhalten STRING");
    }

    #[test]
    fn koord_schneidet_ab() {
        assert_eq!(koord(&Value::Float(10.9), "X", Some(1)), Ok(10));
        assert_eq!(koord(&Value::Float(-0.9), "X", Some(1)), Ok(0));
    }

    #[test]
    fn schalter_nimmt_zahlen() {
        assert_eq!(schalter(&Value::Int(1), "X", None), Ok(true));
        assert_eq!(schalter(&Value::Int(0), "X", None), Ok(false));
        assert_eq!(schalter(&Value::Float(0.5), "X", None), Ok(true));
        assert_eq!(schalter(&Value::Bool(true), "X", None), Ok(true));
        assert_eq!(schalter(&Value::Str(std::rc::Rc::new("ja".to_string())), "X", Some(1)).unwrap_err(),
                   "X: erwartet BOOLEAN oder Zahl (Arg 1), erhalten STRING");
        assert_eq!(boolean(&Value::Int(1), "X", None).unwrap_err(), "X: erwartet BOOLEAN, erhalten INTEGER");
    }

    #[test]
    fn fehlendes_argument() {
        assert_eq!(a_int(&[], 0, "X").unwrap_err(), "X: fehlendes Argument 1");
        assert_eq!(a_schalter_oder(&[], 0, true, "X"), Ok(true));
    }
}
