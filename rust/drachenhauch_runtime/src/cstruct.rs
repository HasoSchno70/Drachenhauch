//! C-Structs ueber BUFFER: `STRUCT name LAYOUT C [PACK n] ... END STRUCT`.
//!
//! Entwurf: docs/entwurf-ffi.md (Struct-Lage). Eine Variable dieses Typs ist
//! zur Laufzeit ein gewoehnlicher BUFFER in der Groesse des Structs; der
//! Compiler kennt die Lage jedes Feldes und macht aus `st.monat` einen
//! Zugriff an fester Stelle (`__struct_get`/`__struct_set`). Darum geht ein
//! solcher Wert ohne Umweg an eine `DECLARE`-Funktion (`AS BUFFER` oder
//! `AS name`) -- und taugt ebenso fuer Dateiformate (`PACK 1`).
//!
//! Die Regeln sind die von C auf den Systemen, die dhrt kennt: jedes Feld
//! liegt auf einer Stelle, die durch seine Ausrichtung teilbar ist, der
//! Struct ist so ausgerichtet wie sein strengstes Feld und so lang, dass ein
//! zweiter direkt dahinter passte. `PACK n` begrenzt die Ausrichtung auf n
//! (wie `#pragma pack(n)`).
//!
//! Hier steht nichts, was fremden Speicher anfasst -- alle Zugriffe bleiben
//! im BUFFER und pruefen seine Laenge. Das Modul braucht das Feature `ffi`
//! darum nicht.

use std::collections::HashMap;

use crate::value::Value;

/// Ein Feld, wie der Parser es liefert: Name, Typ (Typwort klein, `text*n`,
/// `wtext*n` oder `#name` fuer einen anderen Struct) und die Zahl der
/// Elemente (0 = kein Feld von Elementen).
pub type RohFeld = (String, String, u32);

#[derive(Clone, Debug, PartialEq)]
pub struct Feld {
    pub name: String,
    /// Typzeichen wie in ffi.rs (`b`..`z`, `f`, `d`, `o`), `t`/`w` fuer Text,
    /// `#` fuer einen eingebetteten Struct.
    pub art: char,
    /// Bei `#`: der Name des Structs (klein).
    pub unter: String,
    /// Elemente eines Feldes (0 = einzeln).
    pub anzahl: u32,
    /// Bei `t`/`w`: Platz in Zeichen.
    pub zeichen: u32,
    pub offset: usize,
    /// Groesse EINES Elements in Bytes.
    pub groesse: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lage {
    pub name: String,
    pub groesse: usize,
    pub ausrichtung: usize,
    pub felder: Vec<Feld>,
}

impl Lage {
    pub fn feld(&self, name: &str) -> Option<&Feld> {
        self.felder.iter().find(|f| f.name.eq_ignore_ascii_case(name))
    }
    pub fn feldnamen(&self) -> String {
        self.felder.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", ")
    }
}

/// Breite eines `wchar_t` (wie in ffi.rs).
pub const WCHAR: usize = if cfg!(windows) { 2 } else { 4 };

/// Groesse und Ausrichtung eines einzelnen Wertes dieses Typzeichens.
pub fn breite(art: char) -> Option<usize> {
    Some(match art {
        'b' | 'B' | 't' => 1,
        's' | 'S' => 2,
        'l' | 'L' | 'o' | 'f' => 4,
        'q' | 'd' => 8,
        'z' => std::mem::size_of::<usize>(),
        'w' => WCHAR,
        _ => return None,
    })
}

/// Alle Lagen eines Programms, in beliebiger Reihenfolge der Deklaration
/// (ein Struct darf einen spaeter deklarierten einbetten, aber nicht sich
/// selbst, auch nicht ueber Umwege).
pub fn lagen_rechnen(roh: &[(String, Option<u32>, Vec<RohFeld>)]) -> Result<HashMap<String, Lage>, String> {
    let mut fertig: HashMap<String, Lage> = HashMap::new();
    let nach_name: HashMap<String, usize> = roh.iter().enumerate()
        .map(|(i, r)| (r.0.to_lowercase(), i)).collect();
    fn rechne(i: usize, roh: &[(String, Option<u32>, Vec<RohFeld>)], nach_name: &HashMap<String, usize>,
              fertig: &mut HashMap<String, Lage>, unterwegs: &mut Vec<String>) -> Result<(), String> {
        let (name, pack, felder) = &roh[i];
        let schluessel = name.to_lowercase();
        if fertig.contains_key(&schluessel) { return Ok(()); }
        if unterwegs.contains(&schluessel) {
            return Err(format!("STRUCT {} LAYOUT C enthaelt sich selbst (ueber {}) -- ein Struct kann sich nicht einbetten, nur einen ZEIGER auf sich haben",
                               name, unterwegs.join(" -> ")));
        }
        unterwegs.push(schluessel.clone());
        let mut aus = Vec::new();
        let mut stelle = 0usize;
        let mut streng = 1usize;
        let grenze = pack.map(|p| p as usize).unwrap_or(usize::MAX);
        for (fname, typ, anzahl) in felder {
            if aus.iter().any(|f: &Feld| f.name.eq_ignore_ascii_case(fname)) {
                return Err(format!("STRUCT {}: das Feld '{}' steht zweimal da", name, fname));
            }
            let (art, unter, zeichen, groesse, ausr) = if let Some(n) = typ.strip_prefix('#') {
                let Some(&j) = nach_name.get(&n.to_lowercase()) else {
                    return Err(format!("STRUCT {}: '{}' ist weder ein Typwort noch ein STRUCT ... LAYOUT C", name, n));
                };
                rechne(j, roh, nach_name, fertig, unterwegs)?;
                let l = &fertig[&n.to_lowercase()];
                ('#', n.to_lowercase(), 0, l.groesse, l.ausrichtung)
            } else if let Some((w, n)) = typ.split_once('*') {
                let art = if w == "wtext" || w == "wstr" { 'w' } else { 't' };
                let n: u32 = n.parse().unwrap_or(0);
                let b = breite(art).unwrap();
                (art, String::new(), n, b * n as usize, b)
            } else {
                let Some(c) = crate::ffi::typ_zeichen(typ) else {
                    return Err(crate::ffi::typ_hinweis(typ));
                };
                let b = breite(c).ok_or_else(|| format!("STRUCT {}: {} geht in einem Struct nicht", name, typ.to_uppercase()))?;
                (c, String::new(), 0, b, b)
            };
            let ausr = ausr.min(grenze).max(1);
            stelle = stelle.div_ceil(ausr) * ausr;
            streng = streng.max(ausr);
            aus.push(Feld { name: fname.clone(), art, unter, anzahl: *anzahl, zeichen, offset: stelle, groesse });
            stelle += groesse * (*anzahl).max(1) as usize;
        }
        let groesse = stelle.div_ceil(streng) * streng;
        unterwegs.pop();
        fertig.insert(schluessel, Lage { name: name.clone(), groesse, ausrichtung: streng, felder: aus });
        Ok(())
    }
    for i in 0..roh.len() {
        rechne(i, roh, &nach_name, &mut fertig, &mut Vec::new())?;
    }
    Ok(fertig)
}

// ------------------------------------------------------------------ Laufzeit

fn buf<'a>(v: &'a Value, wo: &str) -> Result<&'a std::rc::Rc<std::cell::RefCell<Vec<u8>>>, String> {
    match v {
        Value::Buffer(b) => Ok(b),
        Value::Nil => Err(format!("{}: der Struct ist NIL -- er bekommt seinen Speicher mit DIM", wo)),
        _ => Err(format!("{}: erwartet einen BUFFER, erhalten {}", wo, v.type_name())),
    }
}

fn zahl(v: &Value) -> i64 { if let Value::Int(i) = v { *i } else { 0 } }
fn text(v: &Value) -> String { if let Value::Str(s) = v { s.to_string() } else { String::new() } }

fn bereich(len: usize, off: i64, n: usize, wo: &str) -> Result<usize, String> {
    if off < 0 || off as usize + n > len {
        return Err(format!("{}: der Puffer ist {} Bytes lang, das Feld liegt bei {}..{} -- ist es ein Puffer des richtigen Structs?",
                           wo, len, off, off + n as i64));
    }
    Ok(off as usize)
}

/// Die Befehle, die der Compiler fuer Struct-Felder einsetzt. `None` = kein
/// Befehl dieses Moduls.
pub fn befehl(name: &str, a: &[Value]) -> Option<Result<Value, String>> {
    Some(match name {
        // __struct_neu(groesse) -- der Speicher eines neuen Structs, voller Nullen.
        "__struct_neu" => Ok(crate::builtins::neuer_buffer(vec![0u8; zahl(&a[0]).max(0) as usize])),
        // __struct_index(i, anzahl, wo$) -- ein Index in ein Feld von Elementen.
        "__struct_index" => {
            let (i, n) = (&a[0], zahl(&a[1]));
            match i {
                Value::Int(k) if *k >= 0 && *k < n => Ok(Value::Int(*k)),
                Value::Int(k) => Err(format!("{}: Index {} liegt ausserhalb 0..{}", text(&a[2]), k, n - 1)),
                v => Err(format!("{}: der Index muss INTEGER sein, erhalten {}", text(&a[2]), v.type_name())),
            }
        }
        // __struct_get(puffer, wo$, offset, art$, zeichen)
        "__struct_get" => lesen(&a[0], &text(&a[1]), zahl(&a[2]), &text(&a[3]), zahl(&a[4]) as usize),
        // __struct_set(puffer, wo$, offset, art$, zeichen, wert)
        "__struct_set" => schreiben(&a[0], &text(&a[1]), zahl(&a[2]), &text(&a[3]), zahl(&a[4]) as usize, &a[5])
            .map(|_| Value::Nil),
        _ => return None,
    })
}

fn lesen(b: &Value, wo: &str, off: i64, art: &str, zeichen: usize) -> Result<Value, String> {
    let b = buf(b, wo)?.borrow();
    let c = art.chars().next().unwrap_or('q');
    let w = breite(c).unwrap_or(8);
    if c == 't' || c == 'w' {
        let o = bereich(b.len(), off, w * zeichen, wo)?;
        let teil = &b[o..o + w * zeichen];
        return Ok(Value::str_rc(if c == 't' {
            let ende = teil.iter().position(|&x| x == 0).unwrap_or(teil.len());
            String::from_utf8_lossy(&teil[..ende]).into_owned()
        } else if WCHAR == 2 {
            let einh: Vec<u16> = teil.chunks(2).map(|p| u16::from_le_bytes([p[0], p[1]])).take_while(|&u| u != 0).collect();
            String::from_utf16_lossy(&einh)
        } else {
            teil.chunks(4).map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]))
                .take_while(|&u| u != 0).map(|u| char::from_u32(u).unwrap_or('\u{fffd}')).collect()
        }));
    }
    let o = bereich(b.len(), off, w, wo)?;
    let mut platz = [0u8; 8];
    platz[..w].copy_from_slice(&b[o..o + w]);
    Ok(crate::ffi::platz_wert(c, u64::from_le_bytes(platz)))
}

fn schreiben(b: &Value, wo: &str, off: i64, art: &str, zeichen: usize, v: &Value) -> Result<(), String> {
    let mut b = buf(b, wo)?.borrow_mut();
    let c = art.chars().next().unwrap_or('q');
    let w = breite(c).unwrap_or(8);
    if c == 't' || c == 'w' {
        let s = match v {
            Value::Str(s) => s.to_string(),
            _ => return Err(format!("{}: erwartet einen Text (STRING), erhalten {}", wo, v.type_name())),
        };
        let o = bereich(b.len(), off, w * zeichen, wo)?;
        let bytes: Vec<u8> = if c == 't' {
            s.as_bytes().to_vec()
        } else if WCHAR == 2 {
            s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect()
        } else {
            s.chars().flat_map(|ch| (ch as u32).to_le_bytes()).collect()
        };
        if bytes.len() > w * zeichen {
            return Err(if c == 't' {
                format!("{}: der Text braucht {} Bytes in UTF-8, im Feld ist Platz fuer {}", wo, bytes.len(), zeichen)
            } else {
                format!("{}: der Text braucht {} Zeichen, im Feld ist Platz fuer {}", wo, bytes.len() / w, zeichen)
            });
        }
        let teil = &mut b[o..o + w * zeichen];
        teil.fill(0);
        teil[..bytes.len()].copy_from_slice(&bytes);
        return Ok(());
    }
    let p = crate::ffi::zahl_platz(c, v).map_err(|e| format!("{}: {}", wo, e))?;
    let o = bereich(b.len(), off, w, wo)?;
    b[o..o + w].copy_from_slice(&p.to_le_bytes()[..w]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roh(name: &str, pack: Option<u32>, felder: &[(&str, &str, u32)]) -> (String, Option<u32>, Vec<RohFeld>) {
        (name.into(), pack, felder.iter().map(|(n, t, a)| (n.to_string(), t.to_string(), *a)).collect())
    }

    #[test]
    fn lage_nach_den_regeln_von_c() {
        // SYSTEMTIME: acht WORD -- 16 Bytes, dicht.
        let l = lagen_rechnen(&[roh("SYSTEMTIME", None, &[("jahr", "ushort", 0), ("monat", "ushort", 0),
            ("wt", "ushort", 0), ("tag", "ushort", 0), ("h", "ushort", 0), ("m", "ushort", 0),
            ("s", "ushort", 0), ("ms", "ushort", 0)])]).unwrap();
        let st = &l["systemtime"];
        assert_eq!(st.groesse, 16);
        assert_eq!(st.feld("monat").unwrap().offset, 2);
        // { BYTE a; DOUBLE d; SHORT s; } -> a@0, d@8, s@16, Groesse 24 (Ausrichtung 8)
        let l = lagen_rechnen(&[roh("m", None, &[("a", "byte", 0), ("d", "float", 0), ("s", "short", 0)])]).unwrap();
        let m = &l["m"];
        assert_eq!((m.feld("d").unwrap().offset, m.feld("s").unwrap().offset, m.groesse, m.ausrichtung), (8, 16, 24, 8));
        // PACK 1 schiebt nichts auf.
        let l = lagen_rechnen(&[roh("p", Some(1), &[("a", "byte", 0), ("d", "float", 0), ("s", "short", 0)])]).unwrap();
        assert_eq!((l["p"].feld("d").unwrap().offset, l["p"].groesse), (1, 11));
        // PACK 2: Ausrichtung hoechstens 2.
        let l = lagen_rechnen(&[roh("p", Some(2), &[("a", "byte", 0), ("l", "long", 0)])]).unwrap();
        assert_eq!((l["p"].feld("l").unwrap().offset, l["p"].groesse), (2, 6));
    }

    #[test]
    fn felder_text_und_eingebettete_structs() {
        // utsname unter Linux: sechs char[65].
        let l = lagen_rechnen(&[roh("uts", None, &[("sys", "text*65", 0), ("node", "text*65", 0)])]).unwrap();
        assert_eq!((l["uts"].feld("node").unwrap().offset, l["uts"].groesse), (65, 130));
        // Ein spaeter deklarierter Struct darf eingebettet werden, samt Feld von Elementen.
        let l = lagen_rechnen(&[
            roh("linie", None, &[("n", "byte", 0), ("p", "#punkt", 2)]),
            roh("punkt", None, &[("x", "long", 0), ("y", "long", 0)]),
        ]).unwrap();
        let li = &l["linie"];
        assert_eq!((li.feld("p").unwrap().offset, li.feld("p").unwrap().groesse, li.groesse), (4, 8, 20));
        // Ein Feld von Zahlen.
        let l = lagen_rechnen(&[roh("w", None, &[("n", "long", 0), ("w", "float", 3)])]).unwrap();
        assert_eq!((l["w"].feld("w").unwrap().offset, l["w"].groesse), (8, 32));
    }

    #[test]
    fn fehler_in_der_lage() {
        assert!(lagen_rechnen(&[roh("a", None, &[("b", "#a", 0)])]).unwrap_err().contains("sich selbst"));
        assert!(lagen_rechnen(&[roh("a", None, &[("x", "long", 0), ("X", "long", 0)])]).unwrap_err().contains("zweimal"));
        assert!(lagen_rechnen(&[roh("a", None, &[("x", "#gibtsnicht", 0)])]).unwrap_err().contains("weder"));
    }

    #[test]
    fn lesen_und_schreiben_im_puffer() {
        let b = crate::builtins::neuer_buffer(vec![0u8; 16]);
        let r = |n: &str, a: &[Value]| befehl(n, a).unwrap();
        let s = |x: &str| Value::str_rc(x);
        r("__struct_set", &[b.clone(), s("st.x"), Value::Int(2), s("s"), Value::Int(0), Value::Int(-2)]).unwrap();
        assert!(matches!(r("__struct_get", &[b.clone(), s("st.x"), Value::Int(2), s("s"), Value::Int(0)]).unwrap(), Value::Int(-2)));
        assert!(matches!(r("__struct_get", &[b.clone(), s("st.x"), Value::Int(2), s("S"), Value::Int(0)]).unwrap(), Value::Int(65534)));
        // Ein Wert, der nicht passt, ist ein Fehler wie bei DECLARE.
        assert!(r("__struct_set", &[b.clone(), s("st.x"), Value::Int(0), s("B"), Value::Int(0), Value::Int(300)])
            .err().unwrap().contains("passt nicht"));
        // Text: bis zur Null, zu lang ist ein Fehler.
        r("__struct_set", &[b.clone(), s("st.t"), Value::Int(8), s("t"), Value::Int(6), s("Grüß")]).unwrap();
        assert!(matches!(r("__struct_get", &[b.clone(), s("st.t"), Value::Int(8), s("t"), Value::Int(6)]).unwrap(),
                         Value::Str(x) if x.as_str() == "Grüß"));
        assert!(r("__struct_set", &[b.clone(), s("st.t"), Value::Int(8), s("t"), Value::Int(6), s("Drachenhauch")])
            .err().unwrap().contains("Platz"));
        // Ausserhalb des Puffers.
        assert!(r("__struct_get", &[b.clone(), s("st.y"), Value::Int(14), s("l"), Value::Int(0)]).err().unwrap().contains("16 Bytes"));
        assert!(r("__struct_index", &[Value::Int(3), Value::Int(3), s("w.a")]).err().unwrap().contains("0..2"));
    }
}
