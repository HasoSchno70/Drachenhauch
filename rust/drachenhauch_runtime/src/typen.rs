//! Die Typen, die der Compiler einem Ausdruck zuschreibt (M1 aus
//! `docs/entwurf-maschinencode.md`).
//!
//! Anders als `Compiler::statischer_typ` (ein vorsichtiger Helfer fuer
//! WARNUNGEN, der auch mal "weiss nicht" sagen darf, wo er es wuesste) ist
//! das hier fuer den CODE gedacht: was hier steht, muss zur Laufzeit stimmen.
//! Geprueft wird das mit `DHRT_TYPEN_PRUEFEN=1` -- dann setzt der Compiler
//! hinter jeden getypten Ausdruck eine Probe (`TYP_PRUEFEN`), und die ganze
//! Pruefsammlung laeuft damit.
//!
//! Zwei Stellen, an denen Drachenhauch den Typ vom WERT abhaengig macht, und
//! die darum eine eigene Art brauchen:
//! - `a / b` und `a ^ b` mit zwei INTEGER liefern INTEGER, wenn es aufgeht
//!   (`480 / 2`, `2 ^ 3`), sonst FLOAT (`7 / 2`, `2 ^ -1`) -> [`Typ::Zahl`].
//! - `a AND b` / `a OR b` liefern einen der beiden WERTE (`6 AND 3` ist 3),
//!   also den Verbund beider Typen -> [`Typ::verbinden`].

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Typ {
    Int,
    Float,
    /// INTEGER oder FLOAT -- welches, entscheidet erst der Wert.
    Zahl,
    Bool,
    Str,
    Tupel,
    /// `ARRAY OF T` -- nie NIL (auch `DIM a AS ARRAY OF T` ist leer, nicht NIL).
    Feld(Box<Typ>),
    /// `MAP OF T`.
    Map(Box<Typ>),
    /// Eine Instanz dieser Klasse ODER einer Abkoemmlingin -- oder NIL: eine
    /// als Klasse angesagte Variable darf NIL halten.
    Klasse(String),
    /// Weiss der Compiler nicht (`any`, FUNCREF, Builtins, Modultypen, ...).
    Unbekannt,
}

impl Typ {
    /// Aus der Typangabe, wie sie im Compiler steht: `integer`, `float`,
    /// `string`, `boolean`, `tuple`, `array:T`, `map:T`, ein Klassenname.
    /// `klassen` sagt, welche Namen Klassen sind -- alles andere (Modultypen
    /// wie `vec2`, `any`, `funcref`) ist unbekannt.
    pub fn aus_angabe(t: &str, ist_klasse: &dyn Fn(&str) -> bool) -> Typ {
        let low = t.to_ascii_lowercase();
        match low.as_str() {
            "integer" => Typ::Int,
            "float" => Typ::Float,
            "string" => Typ::Str,
            "boolean" => Typ::Bool,
            "tuple" => Typ::Tupel,
            _ => {
                if low.starts_with("array:") {
                    Typ::Feld(Box::new(Typ::aus_angabe(&t[6..], ist_klasse)))
                } else if low.starts_with("map:") {
                    Typ::Map(Box::new(Typ::aus_angabe(&t[4..], ist_klasse)))
                } else if !t.is_empty() && ist_klasse(t) {
                    Typ::Klasse(t.to_string())
                } else {
                    Typ::Unbekannt
                }
            }
        }
    }

    pub fn ist_zahl(&self) -> bool { matches!(self, Typ::Int | Typ::Float | Typ::Zahl) }

    /// Der Typ eines Werts, der der eine ODER der andere sein kann
    /// (`IIF`, `AND`/`OR`).
    pub fn verbinden(&self, b: &Typ) -> Typ {
        if self == b { return self.clone(); }
        if self.ist_zahl() && b.ist_zahl() { return Typ::Zahl; }
        Typ::Unbekannt
    }

    /// Ergebnis von `+ - *` auf zwei Zahlen: FLOAT schlaegt alles, zwei
    /// INTEGER bleiben INTEGER, sonst haengt es am Wert.
    pub fn rechnen(&self, b: &Typ) -> Typ {
        match (self, b) {
            (Typ::Float, t) | (t, Typ::Float) if t.ist_zahl() => Typ::Float,
            (Typ::Int, Typ::Int) => Typ::Int,
            (a, b) if a.ist_zahl() && b.ist_zahl() => Typ::Zahl,
            _ => Typ::Unbekannt,
        }
    }

    /// Die Angabe fuer die Probe zur Laufzeit (`TYP_PRUEFEN`); `None`, wenn es
    /// nichts zu pruefen gibt.
    pub fn probe(&self) -> Option<String> {
        match self {
            Typ::Unbekannt => None,
            _ => Some(self.to_string()),
        }
    }
}

impl fmt::Display for Typ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Typ::Int => write!(f, "INTEGER"),
            Typ::Float => write!(f, "FLOAT"),
            Typ::Zahl => write!(f, "ZAHL"),
            Typ::Bool => write!(f, "BOOLEAN"),
            Typ::Str => write!(f, "STRING"),
            Typ::Tupel => write!(f, "TUPLE"),
            Typ::Feld(t) => write!(f, "ARRAY OF {}", t),
            Typ::Map(t) => write!(f, "MAP OF {}", t),
            Typ::Klasse(k) => write!(f, "{}", k),
            Typ::Unbekannt => write!(f, "?"),
        }
    }
}

/// Rueckgabetypen eingebauter Befehle -- nur die hier, jeder nachgemessen
/// (2026-09-24, mit TYPEOF und dann ueber die Pruefsammlung mit Proben).
/// Die Regel "endet auf `$`, also Text" stimmt NICHT: `SPLIT$` liefert ein
/// Feld. Und manche haengen am Wert: `VAL("3")` ist INTEGER, `VAL("3.5")`
/// FLOAT; `MIN(1, 2.0)` ist INTEGER. `ABS` liefert den Typ seines
/// Arguments (hier als `Zahl`, der Aufrufer setzt ihn ein). `RND()` ist
/// FLOAT, `RND(6)` aber INTEGER -- das entscheidet der Aufrufer an der
/// Argumentzahl (hier nicht in der Tabelle).
pub fn builtin_typ(name: &str) -> Option<Typ> {
    Some(match name {
        "str$" | "str" | "chr$" | "chr" | "left$" | "left" | "right$" | "right"
        | "mid$" | "mid" | "upper$" | "upper" | "lower$" | "lower" | "trim$" | "trim" | "ltrim$" | "rtrim$"
        | "replace$" | "replace" | "format$" | "hex$" | "bin$" | "oct$" | "space$"
        | "repeat$" | "join$" | "padl$" | "padr$" => Typ::Str,
        "len" | "int" | "millis" => Typ::Int,
        "sqr" | "sin" | "cos" | "timer" => Typ::Float,
        "abs" => Typ::Zahl,
        _ => return None,
    })
}

/// Passt ein Laufzeitwert zu der Angabe aus [`Typ::probe`]? Klassen fragt
/// der Aufrufer selbst (er kennt die Vererbung); hier nur, ob die Angabe eine
/// Klasse meint (`Err(name)`).
pub fn passt(angabe: &str, v: &crate::value::Value) -> Result<bool, String> {
    use crate::value::Value;
    Ok(match angabe {
        "INTEGER" => matches!(v, Value::Int(_)),
        "FLOAT" => matches!(v, Value::Float(_)),
        "ZAHL" => matches!(v, Value::Int(_) | Value::Float(_)),
        "BOOLEAN" => matches!(v, Value::Bool(_)),
        "STRING" => matches!(v, Value::Str(_)),
        "TUPLE" => matches!(v, Value::Tuple(_)),
        _ if angabe.starts_with("ARRAY OF ") => matches!(v, Value::Array(_)),
        _ if angabe.starts_with("MAP OF ") => matches!(v, Value::Map(_)),
        _ => return Err(angabe.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keine(_: &str) -> bool { false }

    #[test]
    fn angaben() {
        assert_eq!(Typ::aus_angabe("INTEGER", &keine), Typ::Int);
        assert_eq!(Typ::aus_angabe("array:float", &keine), Typ::Feld(Box::new(Typ::Float)));
        assert_eq!(Typ::aus_angabe("map:string", &keine), Typ::Map(Box::new(Typ::Str)));
        assert_eq!(Typ::aus_angabe("vec2", &keine), Typ::Unbekannt);
        assert_eq!(Typ::aus_angabe("any", &keine), Typ::Unbekannt);
        assert_eq!(Typ::aus_angabe("Tier", &|k| k == "Tier"), Typ::Klasse("Tier".into()));
    }

    #[test]
    fn verbinden_und_rechnen() {
        assert_eq!(Typ::Int.verbinden(&Typ::Int), Typ::Int);
        assert_eq!(Typ::Int.verbinden(&Typ::Float), Typ::Zahl);
        assert_eq!(Typ::Bool.verbinden(&Typ::Int), Typ::Unbekannt);
        assert_eq!(Typ::Int.rechnen(&Typ::Int), Typ::Int);
        assert_eq!(Typ::Zahl.rechnen(&Typ::Float), Typ::Float);
        assert_eq!(Typ::Zahl.rechnen(&Typ::Int), Typ::Zahl);
        assert_eq!(Typ::Str.rechnen(&Typ::Int), Typ::Unbekannt);
    }

    #[test]
    fn probe_und_passt() {
        use crate::value::Value;
        assert_eq!(Typ::Unbekannt.probe(), None);
        assert_eq!(passt("ZAHL", &Value::Float(1.0)), Ok(true));
        assert_eq!(passt("INTEGER", &Value::Float(1.0)), Ok(false));
        assert_eq!(passt("Tier", &Value::Nil), Err("Tier".into()));
    }
}
