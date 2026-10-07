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
//! **Bitfelder** (`a AS LONG : 3`) liegen in einer Speichereinheit ihres
//! Typs. Wie sich mehrere eine Einheit teilen, legen die Compiler
//! verschieden fest (`Bitregel`): MSVC (Windows) beginnt eine neue Einheit,
//! sobald der Typ eine andere Groesse hat oder die Bits nicht mehr passen;
//! GCC und Clang (Linux, macOS) setzen ein Bitfeld an die naechste freie
//! Bitstelle, solange es keine Grenze seines Typs ueberschreitet -- auch
//! hinter ein Feld eines anderen Typs. Die Bits zaehlen von unten (alle
//! Ziele sind little-endian).
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
    /// Bei einem Bitfeld: seine Breite in Bits (0 = kein Bitfeld) und die
    /// Stelle des untersten Bits in der Einheit ab `offset`.
    pub bits: u32,
    pub bit: u32,
}

/// Nach welchen Regeln Bitfelder liegen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bitregel { Msvc, Gcc }

/// Die Regel des Systems, fuer das dhrt gebaut ist.
pub fn bitregel() -> Bitregel { if cfg!(windows) { Bitregel::Msvc } else { Bitregel::Gcc } }

/// Ob ein Typzeichen ein Bitfeld tragen kann (ganze Zahlen und BOOLEAN).
pub fn bitfaehig(art: char) -> bool { matches!(art, 'b' | 'B' | 's' | 'S' | 'l' | 'L' | 'q' | 'o') }

/// Ob die Zahl eines Bitfelds dieses Typs ein Vorzeichen hat.
fn mit_vorzeichen(art: char) -> bool { matches!(art, 'b' | 's' | 'l' | 'q') }

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
    lagen_rechnen_mit(roh, bitregel())
}

/// Wie `lagen_rechnen`, mit den Bitfeld-Regeln eines bestimmten Systems --
/// die Rust-Tests rechnen beide auf jedem System.
pub fn lagen_rechnen_mit(roh: &[(String, Option<u32>, Vec<RohFeld>)], regel: Bitregel) -> Result<HashMap<String, Lage>, String> {
    let mut fertig: HashMap<String, Lage> = HashMap::new();
    let nach_name: HashMap<String, usize> = roh.iter().enumerate()
        .map(|(i, r)| (r.0.to_lowercase(), i)).collect();
    fn rechne(i: usize, roh: &[(String, Option<u32>, Vec<RohFeld>)], nach_name: &HashMap<String, usize>,
              fertig: &mut HashMap<String, Lage>, unterwegs: &mut Vec<String>, regel: Bitregel) -> Result<(), String> {
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
        // Bitfelder: die Stelle in BITS (GCC) bzw. die offene Einheit
        // (Stelle, Groesse, belegte Bits) unter MSVC.
        let mut bitstelle: Option<usize> = None;
        let mut einheit: Option<(usize, usize, u32)> = None;
        for (fname, typ_roh, anzahl) in felder {
            if aus.iter().any(|f: &Feld| f.name.eq_ignore_ascii_case(fname)) {
                return Err(format!("STRUCT {}: das Feld '{}' steht zweimal da", name, fname));
            }
            // `long:3` = ein Bitfeld mit drei Bits.
            let (typ, bits) = match typ_roh.split_once(':') {
                Some((t, b)) => (t.to_string(), b.parse::<u32>().unwrap_or(0)),
                None => (typ_roh.clone(), 0),
            };
            let typ = &typ;
            if bits > 0 {
                let c = crate::ffi::typ_zeichen(typ).unwrap_or('?');
                if !bitfaehig(c) {
                    return Err(format!("STRUCT {}: das Bitfeld '{}' braucht eine ganze Zahl (BYTE, SHORT, LONG, INTEGER, auch ohne Vorzeichen, oder BOOLEAN), nicht {}",
                                       name, fname, typ.to_uppercase()));
                }
                if *anzahl > 0 {
                    return Err(format!("STRUCT {}: '{}' ist ein Bitfeld -- ein Feld von Bitfeldern gibt es in C nicht", name, fname));
                }
                if pack.is_some() {
                    return Err(format!("STRUCT {}: Bitfelder gehen (noch) nicht zusammen mit PACK -- dort legen die Compiler sie verschieden", name));
                }
                let b = breite(c).unwrap();
                if bits as usize > b * 8 {
                    return Err(format!("STRUCT {}: das Bitfeld '{}' hat {} Bits, {} hat nur {}", name, fname, bits, typ.to_uppercase(), b * 8));
                }
                streng = streng.max(b);
                let (offset, bit) = match regel {
                    Bitregel::Gcc => {
                        // An die naechste freie Bitstelle, solange es keine
                        // Grenze seines Typs ueberschreitet.
                        let mut pos = bitstelle.unwrap_or(stelle * 8);
                        let einh = b * 8;
                        if pos / einh != (pos + bits as usize - 1) / einh { pos = pos.div_ceil(einh) * einh; }
                        bitstelle = Some(pos + bits as usize);
                        let o = pos / einh * b;
                        (o, (pos - o * 8) as u32)
                    }
                    Bitregel::Msvc => match einheit {
                        Some((o, g, belegt)) if g == b && belegt + bits <= (g * 8) as u32 => {
                            einheit = Some((o, g, belegt + bits));
                            (o, belegt)
                        }
                        _ => {
                            let o = stelle.div_ceil(b) * b;
                            stelle = o + b;
                            einheit = Some((o, b, bits));
                            (o, 0)
                        }
                    },
                };
                aus.push(Feld { name: fname.clone(), art: c, unter: String::new(), anzahl: 0, zeichen: 0,
                                offset, groesse: b, bits, bit });
                continue;
            }
            // Ein gewoehnliches Feld beginnt hinter allen Bits davor.
            if let Some(pos) = bitstelle.take() { stelle = stelle.max(pos.div_ceil(8)); }
            einheit = None;
            let (art, unter, zeichen, groesse, ausr) = if let Some(n) = typ.strip_prefix('#') {
                let Some(&j) = nach_name.get(&n.to_lowercase()) else {
                    return Err(format!("STRUCT {}: '{}' ist weder ein Typwort noch ein STRUCT ... LAYOUT C", name, n));
                };
                rechne(j, roh, nach_name, fertig, unterwegs, regel)?;
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
            aus.push(Feld { name: fname.clone(), art, unter, anzahl: *anzahl, zeichen, offset: stelle, groesse, bits: 0, bit: 0 });
            stelle += groesse * (*anzahl).max(1) as usize;
        }
        if let Some(pos) = bitstelle { stelle = stelle.max(pos.div_ceil(8)); }
        let groesse = stelle.div_ceil(streng) * streng;
        unterwegs.pop();
        fertig.insert(schluessel, Lage { name: name.clone(), groesse, ausrichtung: streng, felder: aus });
        Ok(())
    }
    for i in 0..roh.len() {
        rechne(i, roh, &nach_name, &mut fertig, &mut Vec::new(), regel)?;
    }
    Ok(fertig)
}

/// Die Lage eines Structs fuer die Signatur einer fremden Funktion, die ihn
/// als Wert nimmt oder liefert (`ffi::struct_lesen`): Name, Groesse,
/// Ausrichtung und jede Zahl darin mit ihrer Stelle -- eingebettete Structs
/// und Felder aufgeloest, Text fester Breite als Bereich von Ganzzahlen.
pub fn wert_text(name: &str, lagen: &HashMap<String, Lage>) -> String {
    fn glieder(l: &Lage, lagen: &HashMap<String, Lage>, basis: usize, aus: &mut Vec<String>) {
        let mut einheiten: Vec<usize> = Vec::new();
        for f in &l.felder {
            let n = f.anzahl.max(1) as usize;
            let stelle = basis + f.offset;
            // Bitfelder: ihre Einheit ist eine ganze Zahl, je Einheit einmal.
            if f.bits > 0 {
                if !einheiten.contains(&stelle) {
                    einheiten.push(stelle);
                    aus.push(format!("i{}*1@{}", f.groesse, stelle));
                }
                continue;
            }
            match f.art {
                '#' => if let Some(u) = lagen.get(&f.unter) {
                    for k in 0..n { glieder(u, lagen, stelle + k * f.groesse, aus); }
                },
                't' | 'w' => {
                    let b = breite(f.art).unwrap_or(1);
                    aus.push(format!("i{}*{}@{}", b, f.zeichen as usize * n, stelle));
                }
                'f' => aus.push(format!("f4*{}@{}", n, stelle)),
                'd' => aus.push(format!("f8*{}@{}", n, stelle)),
                c => aus.push(format!("i{}*{}@{}", breite(c).unwrap_or(8), n, stelle)),
            }
        }
    }
    let Some(l) = lagen.get(&name.to_lowercase()) else { return String::new() };
    let mut aus = Vec::new();
    glieder(l, lagen, 0, &mut aus);
    let mut text = format!("{};{};{}", name.to_lowercase(), l.groesse, l.ausrichtung);
    for g in aus { text.push(';'); text.push_str(&g); }
    text
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
        // __struct_feld_neu(n, wo$, groesse) -- ein Feld von n Structs.
        "__struct_feld_neu" => match &a[0] {
            Value::Int(n) if *n >= 0 && (*n as u128) * (zahl(&a[2]) as u128) <= 1 << 30 =>
                Ok(crate::builtins::neuer_buffer(vec![0u8; (*n * zahl(&a[2])) as usize])),
            Value::Int(n) if *n < 0 => Err(format!("DIM {}: die Groesse {} ist negativ", text(&a[1]), n)),
            Value::Int(n) => Err(format!("DIM {}: {} Structs zu {} Bytes ueberschreiten die Obergrenze von {} Bytes",
                                         text(&a[1]), n, zahl(&a[2]), 1i64 << 30)),
            v => Err(format!("DIM {}: die Groesse muss INTEGER sein, erhalten {}", text(&a[1]), v.type_name())),
        },
        // __struct_index_puffer(i, puffer, groesse, wo$) -- ein Index in ein
        // Feld von Structs; wie viele es sind, sagt der Puffer.
        "__struct_index_puffer" => {
            let wo = text(&a[3]);
            let n = match &a[1] {
                Value::Buffer(b) => b.borrow().len() as i64 / zahl(&a[2]).max(1),
                v => return Some(buf(v, &wo).map(|_| Value::Nil)),
            };
            match &a[0] {
                Value::Int(k) if *k >= 0 && *k < n => Ok(Value::Int(*k)),
                Value::Int(k) if n == 0 => Err(format!("{}: Index {} -- das Feld ist leer", wo, k)),
                Value::Int(k) => Err(format!("{}: Index {} liegt ausserhalb 0..{}", wo, k, n - 1)),
                v => Err(format!("{}: der Index muss INTEGER sein, erhalten {}", wo, v.type_name())),
            }
        }
        // __struct_anzahl(puffer, groesse) -- LEN eines Feldes von Structs.
        "__struct_anzahl" => match &a[0] {
            Value::Buffer(b) => Ok(Value::Int(b.borrow().len() as i64 / zahl(&a[1]).max(1))),
            v => buf(v, "LEN").map(|_| Value::Nil),
        },
        // __struct_get(puffer, wo$, offset, art$, zeichen)
        "__struct_get" => lesen(&a[0], &text(&a[1]), zahl(&a[2]), &text(&a[3]), zahl(&a[4]) as usize),
        // __struct_set(puffer, wo$, offset, art$, zeichen, wert)
        "__struct_set" => schreiben(&a[0], &text(&a[1]), zahl(&a[2]), &text(&a[3]), zahl(&a[4]) as usize, &a[5])
            .map(|_| Value::Nil),
        _ => return None,
    })
}

/// Bei einem Bitfeld steht hinter dem Typzeichen `breite@bit` (`l3@5`).
fn bitfeld(art: &str) -> Option<(u32, u32)> {
    let rest = art.get(1..)?;
    let (b, s) = rest.split_once('@')?;
    Some((b.parse().ok()?, s.parse().ok()?))
}

fn lesen(b: &Value, wo: &str, off: i64, art: &str, zeichen: usize) -> Result<Value, String> {
    let b = buf(b, wo)?.borrow();
    let c = art.chars().next().unwrap_or('q');
    let w = breite(c).unwrap_or(8);
    if let Some((bits, bit)) = bitfeld(art) {
        let o = bereich(b.len(), off, w, wo)?;
        let mut platz = [0u8; 8];
        platz[..w].copy_from_slice(&b[o..o + w]);
        let roh = u64::from_le_bytes(platz) >> bit;
        let maske = if bits >= 64 { u64::MAX } else { (1u64 << bits) - 1 };
        let x = roh & maske;
        return Ok(if c == 'o' {
            Value::Bool(x != 0)
        } else if mit_vorzeichen(c) && bits < 64 && x >> (bits - 1) & 1 == 1 {
            Value::Int((x | !maske) as i64)
        } else {
            Value::Int(x as i64)
        });
    }
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
    if let Some((bits, bit)) = bitfeld(art) {
        let x: i64 = match (c, v) {
            ('o', Value::Bool(t)) => *t as i64,
            ('o', _) => return Err(format!("{}: erwartet einen Wahrheitswert (BOOLEAN), erhalten {}", wo, v.type_name())),
            (_, Value::Int(i)) => *i,
            _ => return Err(format!("{}: erwartet eine ganze Zahl ({}), erhalten {}", wo, crate::ffi::typ_name(c), v.type_name())),
        };
        let (lo, hi) = if bits >= 64 { (i64::MIN, i64::MAX) }
            else if mit_vorzeichen(c) { (-(1i64 << (bits - 1)), (1i64 << (bits - 1)) - 1) }
            else { (0, ((1u64 << bits) - 1) as i64) };
        if x < lo || x > hi {
            return Err(format!("{}: {} passt nicht in ein Bitfeld mit {} Bits ({} bis {})", wo, x, bits, lo, hi));
        }
        let o = bereich(b.len(), off, w, wo)?;
        let mut platz = [0u8; 8];
        platz[..w].copy_from_slice(&b[o..o + w]);
        let maske = (if bits >= 64 { u64::MAX } else { (1u64 << bits) - 1 }) << bit;
        let neu = (u64::from_le_bytes(platz) & !maske) | (((x as u64) << bit) & maske);
        b[o..o + w].copy_from_slice(&neu.to_le_bytes()[..w]);
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
    fn lage_als_wert_fuer_die_signatur() {
        let l = lagen_rechnen(&[
            roh("linie", None, &[("n", "ubyte", 0), ("p", "#punkt", 2), ("t", "text*3", 0)]),
            roh("punkt", None, &[("x", "single", 0), ("y", "float", 0)]),
        ]).unwrap();
        // punkt: x@0 (4), y@8 (8) -- 16 Bytes; linie: n@0, p@8 und @24, t@40.
        assert_eq!(wert_text("Punkt", &l), "punkt;16;8;f4*1@0;f8*1@8");
        assert_eq!(wert_text("linie", &l), "linie;48;8;i1*1@0;f4*1@8;f8*1@16;f4*1@24;f8*1@32;i1*3@40");
        let s = crate::ffi::struct_lesen(&wert_text("linie", &l)).unwrap();
        assert_eq!((s.groesse, s.glieder.len()), (48, 6));
    }

    /// Bitfelder nach beiden Regeln -- die Zahlen sind die von MSVC bzw. GCC
    /// und Clang fuer dieselbe C-Deklaration, nachgesehen mit
    /// `clang --target=x86_64-pc-windows-msvc|x86_64-linux-gnu|aarch64-apple-darwin
    /// -Xclang -fdump-record-layouts-simple`.
    #[test]
    fn bitfelder_nach_msvc_und_gcc() {
        use Bitregel::*;
        let lage = |regel, felder: &[(&str, &str, u32)]| {
            let l = lagen_rechnen_mit(&[roh("s", None, felder)], regel).unwrap();
            let s = &l["s"];
            let f: Vec<(usize, u32, u32)> = s.felder.iter().map(|f| (f.offset, f.bit, f.bits)).collect();
            (s.groesse, f)
        };
        // unsigned a:3, b:5, c:24 -- dicht in einer Einheit, bei beiden.
        let f = [("a", "ulong:3", 0), ("b", "ulong:5", 0), ("c", "ulong:24", 0)];
        for r in [Msvc, Gcc] { assert_eq!(lage(r, &f), (4, vec![(0, 0, 3), (0, 3, 5), (0, 8, 24)])); }
        // char c; int a:4 -- GCC teilt die Einheit mit dem char, MSVC nicht.
        let f = [("c", "byte", 0), ("a", "long:4", 0)];
        assert_eq!(lage(Gcc, &f), (4, vec![(0, 0, 0), (0, 8, 4)]));
        assert_eq!(lage(Msvc, &f), (8, vec![(0, 0, 0), (4, 0, 4)]));
        // int a:3; char c -- danach beginnt c im naechsten Byte bzw. hinter der Einheit.
        let f = [("a", "long:3", 0), ("c", "byte", 0)];
        assert_eq!(lage(Gcc, &f), (4, vec![(0, 0, 3), (1, 0, 0)]));
        assert_eq!(lage(Msvc, &f), (8, vec![(0, 0, 3), (4, 0, 0)]));
        // short a:4, b:4; int c:8 -- GCC: alles in der ersten int-Einheit;
        // MSVC: eine neue Einheit, weil der Typ eine andere Groesse hat.
        let f = [("a", "short:4", 0), ("b", "short:4", 0), ("c", "long:8", 0)];
        assert_eq!(lage(Gcc, &f), (4, vec![(0, 0, 4), (0, 4, 4), (0, 8, 8)]));
        assert_eq!(lage(Msvc, &f), (8, vec![(0, 0, 4), (0, 4, 4), (4, 0, 8)]));
        // int a:30; int b:4 -- passt nicht mehr, bei beiden die naechste Einheit.
        let f = [("a", "long:30", 0), ("b", "long:4", 0)];
        for r in [Msvc, Gcc] { assert_eq!(lage(r, &f), (8, vec![(0, 0, 30), (4, 0, 4)])); }
        // char a:3; int b:30 -- GCC: b ueberschritte die Grenze ab Bit 3.
        let f = [("a", "byte:3", 0), ("b", "long:30", 0)];
        assert_eq!(lage(Gcc, &f), (8, vec![(0, 0, 3), (4, 0, 30)]));
        assert_eq!(lage(Msvc, &f), (8, vec![(0, 0, 3), (4, 0, 30)]));
        // long long x:40; int y:10 -- GCC: y in der zweiten int-Einheit ab Bit 8.
        let f = [("x", "integer:40", 0), ("y", "long:10", 0)];
        assert_eq!(lage(Gcc, &f), (8, vec![(0, 0, 40), (4, 8, 10)]));
        assert_eq!(lage(Msvc, &f), (16, vec![(0, 0, 40), (8, 0, 10)]));
        // int a:3; char c; int b:4 -- GCC setzt b hinter c in dieselbe Einheit.
        let f = [("a", "long:3", 0), ("c", "byte", 0), ("b", "long:4", 0)];
        assert_eq!(lage(Gcc, &f), (4, vec![(0, 0, 3), (1, 0, 0), (0, 16, 4)]));
        assert_eq!(lage(Msvc, &f), (12, vec![(0, 0, 3), (4, 0, 0), (8, 0, 4)]));
        // char c; long long x:4 -- GCC im selben 64-Bit-Wort, MSVC dahinter.
        let f = [("c", "byte", 0), ("x", "integer:4", 0)];
        assert_eq!(lage(Gcc, &f), (8, vec![(0, 0, 0), (0, 8, 4)]));
        assert_eq!(lage(Msvc, &f), (16, vec![(0, 0, 0), (8, 0, 4)]));
        // int a:4; double d; char e:2 -- ein gewoehnliches Feld dazwischen.
        let f = [("a", "long:4", 0), ("d", "float", 0), ("e", "byte:2", 0)];
        for r in [Msvc, Gcc] { assert_eq!(lage(r, &f), (24, vec![(0, 0, 4), (8, 0, 0), (16, 0, 2)])); }
        // unsigned char a:7, b:2; unsigned short c:9 -- bei beiden je eine
        // neue Einheit, aus verschiedenen Gruenden.
        let f = [("a", "ubyte:7", 0), ("b", "ubyte:2", 0), ("c", "ushort:9", 0)];
        for r in [Msvc, Gcc] { assert_eq!(lage(r, &f), (4, vec![(0, 0, 7), (1, 0, 2), (2, 0, 9)])); }
        // Die Lage als Wert: eine Einheit nur einmal.
        let l = lagen_rechnen_mit(&[roh("s", None, &[("a", "ulong:3", 0), ("b", "ulong:5", 0), ("d", "float", 0)])], Gcc).unwrap();
        assert_eq!(wert_text("s", &l), "s;16;8;i4*1@0;f8*1@8");
        // Was nicht geht.
        let fehler = |felder: &[(&str, &str, u32)], pack| lagen_rechnen_mit(&[roh("s", pack, felder)], Gcc).unwrap_err();
        assert!(fehler(&[("a", "float:3", 0)], None).contains("ganze Zahl"));
        assert!(fehler(&[("a", "byte:9", 0)], None).contains("nur 8"));
        assert!(fehler(&[("a", "long:3", 2)], None).contains("Feld von Bitfeldern"));
        assert!(fehler(&[("a", "long:3", 0)], Some(1)).contains("PACK"));
    }

    #[test]
    fn bitfelder_lesen_und_schreiben() {
        let b = crate::builtins::neuer_buffer(vec![0u8; 8]);
        let r = |n: &str, a: &[Value]| befehl(n, a).unwrap();
        let s = |x: &str| Value::str_rc(x);
        let setze = |art: &str, x: Value| r("__struct_set", &[b.clone(), s("s.f"), Value::Int(0), s(art), Value::Int(0), x]);
        let lies = |art: &str| r("__struct_get", &[b.clone(), s("s.f"), Value::Int(0), s(art), Value::Int(0)]).unwrap();
        // Drei Felder in einer Einheit, ohne einander zu stoeren.
        setze("L3@0", Value::Int(5)).unwrap();
        setze("l5@3", Value::Int(-7)).unwrap();
        setze("L24@8", Value::Int(0xABCDEF)).unwrap();
        assert!(matches!(lies("L3@0"), Value::Int(5)));
        assert!(matches!(lies("l5@3"), Value::Int(-7)));
        assert!(matches!(lies("L5@3"), Value::Int(25)));
        assert!(matches!(lies("L24@8"), Value::Int(0xABCDEF)));
        let Value::Buffer(roh) = &b else { panic!() };
        assert_eq!(u32::from_le_bytes(roh.borrow()[..4].try_into().unwrap()), 5 | (25 << 3) | (0xABCDEF << 8));
        // Was nicht hineinpasst, ist ein Fehler -- auch die Grenzen sind genau.
        assert!(setze("l5@3", Value::Int(16)).err().unwrap().contains("-16 bis 15"));
        assert!(setze("L3@0", Value::Int(8)).err().unwrap().contains("0 bis 7"));
        assert!(setze("L3@0", Value::Int(-1)).is_err());
        setze("l5@3", Value::Int(-16)).unwrap();
        assert!(matches!(lies("l5@3"), Value::Int(-16)));
        // BOOLEAN mit einem Bit, und eine 64-Bit-Einheit.
        setze("o1@31", Value::Bool(true)).unwrap();
        assert!(matches!(lies("o1@31"), Value::Bool(true)));
        assert!(matches!(lies("L3@0"), Value::Int(5)));
        setze("q40@0", Value::Int(-2)).unwrap();
        assert!(matches!(lies("q40@0"), Value::Int(-2)));
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
