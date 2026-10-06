//! Fremde Bibliotheken aufrufen: `DECLARE FUNCTION name LIB "bib" (...)`.
//!
//! Entwurf: docs/entwurf-ffi.md (Stufe 1). Der Compiler macht aus jedem
//! Aufruf einer deklarierten Funktion den internen Befehl `__ffi` und gibt
//! ihm die Deklaration als ersten Wert mit (`signatur_text`). Hier wird sie
//! einmal zerlegt, die Bibliothek beim ERSTEN Aufruf geladen (ein Programm,
//! das den Zweig nie nimmt, laeuft auch ohne sie) und je Signatur ein
//! Uebergang mit Cranelift gebaut -- dieselbe Bibliothek, die auch den
//! Maschinencode erzeugt; libffi braeuchte eine C-Bibliothek im Bau.
//!
//! Der Uebergang hat immer dieselbe Form, damit Rust ihn ohne Wissen ueber
//! die Signatur rufen kann:
//!
//! ```text
//! extern "C" fn(ziel: *const u8, args: *const u64, rueck: *mut u64)
//! ```
//!
//! Jedes Argument steht in einem 8-Byte-Platz (kleine Ganzzahlen und
//! SINGLE in den unteren Bytes -- alle Ziele sind little-endian), der
//! Uebergang laedt sie in der Breite der Signatur, ruft `ziel` nach der
//! C-Aufrufkonvention des Systems und legt das Ergebnis nach `rueck`.
//!
//! **Ein Absturz in fremdem Code ist nicht abzufangen** -- dafuer gibt es
//! keine Sicherung, nur den Hinweis in der Doku und TASK_START.
//!
//! **Rueckrufe (Stufe 3):** ein Parameter `AS FUNCTION(...) AS typ` bekommt
//! eine Drachenhauch-Funktion (FUNCREF, gebundene Methode, Lambda). Fuer
//! jede bekommt die Bibliothek einen eigenen Einstieg mit der C-Signatur
//! (`uebergang::Bauer::rueckruf`), der die Argumente in 8-Byte-Plaetze legt
//! und `eingang` ruft; der ruft die Funktion ueber die VM. Ein Fehler darin
//! darf nicht durch die C-Rahmen laufen -- er wird gemerkt, jeder weitere
//! Rueckruf liefert 0, und der Aufruf der Bibliothek meldet ihn, sobald sie
//! zurueckkehrt.

#[cfg(feature = "ffi")]
use std::cell::RefCell;
#[cfg(feature = "ffi")]
use std::collections::HashMap;
#[cfg(feature = "ffi")]
use std::rc::Rc;

use crate::value::Value;

/// Vorsatz fuer `LIB name`: der Name einer CONST statt eines Textes
/// (der Parser schreibt ihn, der Compiler setzt den Wert ein).
pub const LIB_CONST: char = '\u{1}';

/// Trenner zwischen den Teilen der Signatur (ASCII Unit Separator) -- kommt
/// in keinem Bibliotheks- oder Funktionsnamen vor.
pub const TRENNER: char = '\u{1f}';

/// Die Typwoerter einer `DECLARE`-Zeile, mit ihrem Kurzzeichen.
/// Gross/klein egal; die C-nahen Namen sind zweite Schreibweisen.
pub const TYPWOERTER: &[(&str, char)] = &[
    ("byte", 'b'), ("ubyte", 'B'), ("short", 's'), ("ushort", 'S'),
    ("long", 'l'), ("ulong", 'L'), ("integer", 'q'),
    ("zeiger", 'z'), ("ptr", 'z'),
    ("single", 'f'), ("float", 'd'), ("boolean", 'o'),
    ("text", 't'), ("cstr", 't'), ("wtext", 'w'), ("wstr", 'w'),
    ("buffer", 'p'),
];

/// Kurzzeichen eines Typworts (`None` = gibt es nicht).
pub fn typ_zeichen(wort: &str) -> Option<char> {
    let w = wort.to_lowercase();
    TYPWOERTER.iter().find(|(n, _)| *n == w).map(|(_, c)| *c)
}

/// Der Name, unter dem ein Kurzzeichen in Meldungen erscheint.
pub fn typ_name(c: char) -> &'static str {
    match c {
        'b' => "BYTE", 'B' => "UBYTE", 's' => "SHORT", 'S' => "USHORT",
        'l' => "LONG", 'L' => "ULONG", 'q' => "INTEGER", 'z' => "ZEIGER",
        'f' => "SINGLE", 'd' => "FLOAT", 'o' => "BOOLEAN",
        't' => "TEXT", 'w' => "WTEXT", 'p' => "BUFFER",
        'r' => "FUNCTION",
        'x' => "STRUCT",
        '*' => "...",
        _ => "?",
    }
}

/// Der Drachenhauch-Typ, den ein Kurzzeichen auf dieser Seite hat.
pub fn dh_typ(c: char) -> &'static str {
    match c {
        'f' | 'd' => "float",
        'o' => "boolean",
        't' | 'w' => "string",
        'p' | 'x' => "buffer",
        'r' => "funcref",
        _ => "integer",
    }
}

/// Das Typwort, das jemand mit einem C- oder BASIC-Namen meint (`int` ->
/// LONG); `None`, wenn das Wort keinem bekannten Namen entspricht.
pub fn typ_vorschlag(wort: &str) -> Option<&'static str> {
    let w = wort.to_lowercase();
    match w.as_str() {
        "string" => Some("TEXT (UTF-8, const char*) oder WTEXT (wchar_t*)"),
        "double" => Some("FLOAT"),
        "int" | "int32" | "dword" | "bool" => Some(if w == "dword" { "ULONG" } else { "LONG" }),
        "uint" | "uint32" => Some("ULONG"),
        "char" | "int8" => Some("BYTE"),
        "word" | "uint16" => Some("USHORT"),
        "int16" => Some("SHORT"),
        "int64" | "longlong" => Some("INTEGER"),
        "handle" | "hwnd" | "void" | "size_t" | "pointer" => Some("ZEIGER"),
        _ => None,
    }
}

/// Hinweis zu einem Wort, das kein Typwort einer `DECLARE`-Zeile ist.
pub fn typ_hinweis(wort: &str) -> String {
    let vorschlag = typ_vorschlag(wort);
    let liste = "BYTE, UBYTE, SHORT, USHORT, LONG, ULONG, INTEGER, ZEIGER, SINGLE, FLOAT, BOOLEAN, TEXT, WTEXT, BUFFER";
    match vorschlag {
        Some(v) => format!("'{}' ist kein Typ fuer fremde Bibliotheken -- hier heisst das {} (moeglich: {})", wort.to_uppercase(), v, liste),
        None => format!("'{}' ist kein Typ fuer fremde Bibliotheken (moeglich: {})", wort.to_uppercase(), liste),
    }
}

/// Die Signatur als Text, wie der Compiler sie dem Aufruf mitgibt:
/// Bibliothek, C-Name, Drachenhauch-Name, Rueckgabe (`v` = SUB) und die
/// Parameter als `[&]typ:name`, durch Komma getrennt. `typ` ist ein Zeichen,
/// bei einem Rueckruf `r`, dann seine Rueckgabe und seine Parameter
/// (`rlzz` = FUNCTION(ZEIGER, ZEIGER) AS LONG). Ein Struct als Wert ist
/// `x` und seine Lage (`struct_lesen`) -- als Parameter wie als Rueckgabe.
pub fn signatur_text(lib: &str, c_name: &str, name: &str, rueck: &str,
                     params: &[(String, bool, String)]) -> String {
    let ps: Vec<String> = params.iter()
        .map(|(c, br, n)| format!("{}{}:{}", if *br { "&" } else { "" }, c, n)).collect();
    format!("{lib}{t}{c_name}{t}{name}{t}{rueck}{t}{}", ps.join(","), t = TRENNER)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub art: char,
    pub byref: bool,
    pub name: String,
    /// Bei einem Rueckruf (`art` = 'r'): Rueckgabe, dann die Parameter.
    /// Bei einem Struct als Wert (`art` = 'x'): seine Lage.
    pub rr: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signatur {
    pub lib: String,
    pub c_name: String,
    pub name: String,
    pub rueck: char,
    /// Bei `rueck` = 'x': die Lage des Structs.
    pub rueck_rr: String,
    pub params: Vec<Param>,
}

pub fn signatur_lesen(s: &str) -> Result<Signatur, String> {
    let teile: Vec<&str> = s.split(TRENNER).collect();
    if teile.len() != 5 { return Err("fremde Funktion: kaputte Signatur".into()); }
    let rueck = teile[3].chars().next().unwrap_or('v');
    let rueck_rr = teile[3][rueck.len_utf8().min(teile[3].len())..].to_string();
    if rueck == 'x' && struct_lesen(&rueck_rr).is_none() { return Err("fremde Funktion: kaputte Signatur".into()); }
    let mut params = Vec::new();
    if !teile[4].is_empty() {
        for p in teile[4].split(',') {
            let (byref, rest) = match p.strip_prefix('&') { Some(r) => (true, r), None => (false, p) };
            let mut it = rest.splitn(2, ':');
            let typ = it.next().unwrap_or("");
            let art = typ.chars().next().ok_or("fremde Funktion: kaputte Signatur")?;
            let rr = typ[art.len_utf8()..].to_string();
            if art == 'r' && rr.is_empty() { return Err("fremde Funktion: kaputte Signatur".into()); }
            if art == 'x' && struct_lesen(&rr).is_none() { return Err("fremde Funktion: kaputte Signatur".into()); }
            let name = it.next().unwrap_or("").to_string();
            params.push(Param { art, byref, name, rr });
        }
    }
    Ok(Signatur { lib: teile[0].into(), c_name: teile[1].into(), name: teile[2].into(), rueck, rueck_rr, params })
}

// ------------------------------------------------------------------ Structs als Wert

/// Eine Zahl in einem Struct, wie die Aufrufkonvention sie sieht: Ganzzahl
/// (auch Zeiger und Zeichen) oder Kommazahl in einer der zwei Breiten.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glied { Ganz, F32, F64 }

/// Ein Struct, der als Wert uebergeben oder zurueckgegeben wird -- so, wie
/// der Compiler ihn in die Signatur schreibt (`cstruct::wert_text`):
/// `name;groesse;ausrichtung;glied;...` mit `glied` = `i4*2@0` (Art,
/// Breite eines Elements, Zahl, Stelle; `i` ganz, `f` Kommazahl).
/// Eingebettete Structs und Felder sind darin schon aufgeloest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructWert {
    pub name: String,
    pub groesse: u32,
    pub ausrichtung: u32,
    /// (Art, Breite eines Elements, Zahl der Elemente, Stelle)
    pub glieder: Vec<(Glied, u32, u32, u32)>,
}

pub fn struct_lesen(s: &str) -> Option<StructWert> {
    let mut it = s.split(';');
    let name = it.next()?.to_string();
    let groesse: u32 = it.next()?.parse().ok()?;
    let ausrichtung: u32 = it.next()?.parse().ok()?;
    let mut glieder = Vec::new();
    for g in it {
        let art = g.chars().next()?;
        let (breite, rest) = g[1..].split_once('*')?;
        let (zahl, stelle) = rest.split_once('@')?;
        let breite: u32 = breite.parse().ok()?;
        let glied = match (art, breite) {
            ('i', _) => Glied::Ganz,
            ('f', 4) => Glied::F32,
            ('f', 8) => Glied::F64,
            _ => return None,
        };
        glieder.push((glied, breite, zahl.parse().ok()?, stelle.parse().ok()?));
    }
    if groesse == 0 || name.is_empty() { return None; }
    Some(StructWert { name, groesse, ausrichtung, glieder })
}

/// Welche Regeln fuer Structs als Wert gelten.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Konvention {
    /// Windows x64: 1, 2, 4 oder 8 Bytes in einem Ganzzahl-Register, sonst
    /// ein Zeiger auf eine Kopie; Rueckgabe ebenso in RAX oder ueber eine
    /// versteckte Adresse als erstes Argument.
    Win64,
    /// System V x86-64 (Linux, macOS Intel): bis 16 Bytes in Achtergruppen,
    /// je nach Inhalt in Ganzzahl- oder SSE-Registern; groesser, schief
    /// gepackt oder ohne freie Register auf dem Stapel.
    SysV,
    /// AAPCS64 (ARM, auch Apple): bis vier gleiche Kommazahlen (HFA) in
    /// V-Registern, bis 16 Bytes in X-Registern, sonst ein Zeiger auf eine
    /// Kopie; die versteckte Rueckgabe-Adresse steht in X8.
    Aapcs,
}

pub fn konvention() -> Konvention {
    if cfg!(target_arch = "aarch64") { Konvention::Aapcs }
    else if cfg!(windows) { Konvention::Win64 }
    else { Konvention::SysV }
}

/// Woher eine Stelle des Uebergangs ihren Wert nimmt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Laden {
    /// Der Wert im 8-Byte-Platz k.
    Platz(usize),
    /// Im Platz k steht ein Zeiger (auf die Kopie eines Structs); der Wert
    /// liegt dort an dieser Stelle.
    Ueber(usize, u32),
    /// Im Platz k steht ein Zeiger; so viele Bytes kommen als Ganzes auf den
    /// Stapel (System V, Klasse MEMORY).
    Kopie(usize, u32),
    /// Ein Fuellwert 0 -- er belegt ein Register, damit das naechste auf den
    /// Stapel kommt (AAPCS64: ein Struct, der nicht mehr ganz in die
    /// Register passt, geht ganz auf den Stapel).
    Null,
    /// Die Adresse, an die die Funktion ihren Struct schreibt.
    Versteckt,
}

/// Eine Stelle der C-Signatur.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Stelle { pub art: Art, pub laden: Laden }

/// Was nach dem Aufruf zurueckkommt.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rueckform {
    Nichts,
    Wert(Art),
    /// Ein Struct in Registern: jeder Wert an seine Stelle im Rueckgabe-Platz.
    Teile(Vec<(Art, u32)>),
}

/// Was ein Parameter (oder die Rueckgabe) ist.
#[derive(Clone, Copy, Debug)]
pub enum Form<'a> { Skalar(Art), Struct(&'a StructWert) }

fn rund8(n: u32) -> u32 { n.div_ceil(8) * 8 }

/// Alle Elemente eines Structs einzeln: (Art, Breite, Stelle).
fn elemente(s: &StructWert) -> Vec<(Glied, u32, u32)> {
    s.glieder.iter().flat_map(|&(g, b, n, o)| (0..n.max(1)).map(move |k| (g, b, o + k * b))).collect()
}

/// System V: die Klasse jeder Achtergruppe (`true` = SSE), oder `None`, wenn
/// der Struct im Speicher reist (groesser als 16 Bytes oder schief gepackt).
fn sysv_klassen(s: &StructWert) -> Option<Vec<bool>> {
    if s.groesse > 16 { return None; }
    let n = s.groesse.div_ceil(8) as usize;
    let mut ganz = vec![false; n];
    for (g, b, o) in elemente(s) {
        if b == 0 || o % b != 0 { return None; }
        if g == Glied::Ganz {
            // Ein Bereich von Zeichen kann ueber eine Grenze gehen.
            for e in (o / 8)..=((o + b - 1) / 8) { if (e as usize) < n { ganz[e as usize] = true; } }
        }
    }
    Some(ganz.into_iter().map(|g| !g).collect())
}

/// AAPCS64: ein Struct aus ein bis vier Kommazahlen derselben Breite (HFA).
fn hfa(s: &StructWert) -> Option<(Art, Vec<u32>)> {
    let el = elemente(s);
    if el.is_empty() || el.len() > 4 { return None; }
    let g = el[0].0;
    if g == Glied::Ganz || el.iter().any(|e| e.0 != g) { return None; }
    Some((if g == Glied::F32 { Art::F32 } else { Art::F64 }, el.iter().map(|e| e.2).collect()))
}

/// Ein Struct, der in Registern zurueckkommt: die Teile; `None` = ueber eine
/// versteckte Adresse.
fn rueck_teile(k: Konvention, s: &StructWert) -> Option<Vec<(Art, u32)>> {
    match k {
        Konvention::Win64 => matches!(s.groesse, 1 | 2 | 4 | 8).then(|| vec![(Art::I64, 0)]),
        Konvention::SysV => sysv_klassen(s).map(|kl| kl.iter().enumerate()
            .map(|(j, sse)| (if *sse { Art::F64 } else { Art::I64 }, j as u32 * 8)).collect()),
        Konvention::Aapcs => match hfa(s) {
            Some((a, stellen)) => Some(stellen.into_iter().map(|o| (a, o)).collect()),
            None if s.groesse <= 16 => Some((0..s.groesse.div_ceil(8)).map(|j| (Art::I64, j * 8)).collect()),
            None => None,
        },
    }
}

/// Der Plan eines Aufrufs: welche Werte in welcher Reihenfolge an die
/// C-Funktion gehen und wie das Ergebnis zurueckkommt. Ohne Structs ist es
/// Platz fuer Platz die Signatur; ein Struct als Wert wird nach den Regeln
/// des Systems zerlegt (`Konvention`). Platz k gehoert immer zu Parameter k
/// -- bei einem Struct steht dort der Zeiger auf seine Kopie.
pub fn plan(k: Konvention, params: &[Form], rueck: Form) -> Result<(Vec<Stelle>, Rueckform), String> {
    let mut st: Vec<Stelle> = Vec::new();
    // Freie Register je Klasse (Windows: jeder Parameter hat seinen Platz,
    // es muss nicht gezaehlt werden).
    let (mut ganz, mut komma) = match k { Konvention::Win64 => (usize::MAX, usize::MAX),
                                          Konvention::SysV => (6, 8), Konvention::Aapcs => (8, 8) };
    let rf = match rueck {
        Form::Skalar(Art::Nichts) => Rueckform::Nichts,
        Form::Skalar(a) => Rueckform::Wert(a),
        Form::Struct(s) => match rueck_teile(k, s) {
            Some(t) => Rueckform::Teile(t),
            None => {
                st.push(Stelle { art: Art::Zeiger, laden: Laden::Versteckt });
                // Unter AAPCS64 steht die Adresse in X8, nicht in X0..X7.
                if k != Konvention::Aapcs { ganz = ganz.saturating_sub(1); }
                Rueckform::Nichts
            }
        },
    };
    for (i, p) in params.iter().enumerate() {
        match *p {
            Form::Skalar(a) => {
                if matches!(a, Art::F32 | Art::F64) { komma = komma.saturating_sub(1); } else { ganz = ganz.saturating_sub(1); }
                st.push(Stelle { art: a, laden: Laden::Platz(i) });
            }
            Form::Struct(s) => match k {
                Konvention::Win64 => {
                    if matches!(s.groesse, 1 | 2 | 4 | 8) {
                        st.push(Stelle { art: Art::I64, laden: Laden::Ueber(i, 0) });
                    } else {
                        st.push(Stelle { art: Art::Zeiger, laden: Laden::Platz(i) });
                    }
                }
                Konvention::SysV => {
                    let kl = sysv_klassen(s).filter(|kl| {
                        let sse = kl.iter().filter(|x| **x).count();
                        sse <= komma && kl.len() - sse <= ganz
                    });
                    match kl {
                        Some(kl) => for (j, sse) in kl.iter().enumerate() {
                            if *sse { komma -= 1; } else { ganz -= 1; }
                            st.push(Stelle { art: if *sse { Art::F64 } else { Art::I64 }, laden: Laden::Ueber(i, j as u32 * 8) });
                        },
                        None => st.push(Stelle { art: Art::Zeiger, laden: Laden::Kopie(i, rund8(s.groesse)) }),
                    }
                }
                Konvention::Aapcs => {
                    if let Some((a, stellen)) = hfa(s) {
                        if stellen.len() > komma {
                            return Err(format!("der Struct {} (Argument {}) passt nicht mehr in die Register fuer Kommazahlen -- so viele Kommazahlen davor gehen hier (noch) nicht",
                                               s.name.to_uppercase(), i + 1));
                        }
                        komma -= stellen.len();
                        for o in stellen { st.push(Stelle { art: a, laden: Laden::Ueber(i, o) }); }
                    } else if s.groesse > 16 {
                        ganz = ganz.saturating_sub(1);
                        st.push(Stelle { art: Art::Zeiger, laden: Laden::Platz(i) });
                    } else {
                        let n = s.groesse.div_ceil(8) as usize;
                        if n > ganz {
                            // Passt er nicht mehr ganz hinein, geht er ganz auf den Stapel,
                            // und kein Ganzzahl-Register wird danach noch benutzt.
                            while ganz > 0 { st.push(Stelle { art: Art::I64, laden: Laden::Null }); ganz -= 1; }
                        } else {
                            ganz -= n;
                        }
                        for j in 0..n { st.push(Stelle { art: Art::I64, laden: Laden::Ueber(i, j as u32 * 8) }); }
                    }
                }
            },
        }
    }
    Ok((st, rf))
}

/// Die Dateinamen, unter denen eine Bibliothek gesucht wird -- in dieser
/// Reihenfolge, jeder erst neben dem Programm, dann neben dhrt, dann wo das
/// System sucht. Ein Name mit Pfad oder Endung gilt woertlich. Mehrere Namen
/// stehen durch `|` getrennt (`"libgtk-3-0.dll|libgtk-3.so.0"`) -- so traegt
/// EINE DECLARE-Zeile die Namen aller Systeme; der erste, der sich laden
/// laesst, gilt.
pub fn dateinamen(name: &str, system: &str) -> Vec<String> {
    if name.contains('|') {
        let mut alle: Vec<String> = Vec::new();
        for teil in name.split('|').map(str::trim).filter(|t| !t.is_empty()) {
            for n in dateinamen(teil, system) {
                if !alle.contains(&n) { alle.push(n); }
            }
        }
        return alle;
    }
    let low = name.to_lowercase();
    let woertlich = name.contains('/') || name.contains('\\')
        || low.ends_with(".dll") || low.ends_with(".dylib") || low.ends_with(".so")
        || low.contains(".so.") || low.contains(".framework");
    if woertlich { return vec![name.to_string()]; }
    match (low.as_str(), system) {
        ("c" | "m", "windows") => vec!["ucrtbase.dll".into(), "msvcrt.dll".into()],
        ("c", "macos") | ("m", "macos") => vec!["/usr/lib/libSystem.B.dylib".into()],
        ("c", _) => vec!["libc.so.6".into(), "libc.so".into()],
        ("m", _) => vec!["libm.so.6".into(), "libm.so".into()],
        (_, "windows") => vec![format!("{}.dll", name)],
        (_, "macos") => vec![format!("lib{}.dylib", name), format!("{}.dylib", name),
                             format!("{n}.framework/{n}", n = name)],
        _ => vec![format!("lib{}.so", name), format!("{}.so", name)],
    }
}

pub fn system() -> &'static str {
    if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "macos" } else { "linux" }
}

/// Wie ein Wert im Platz des Uebergangs liegt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Art { I8, U8, I16, U16, I32, U32, I64, Zeiger, F32, F64, Nichts }

pub fn art_von(c: char, byref: bool) -> Art {
    if byref { return Art::Zeiger; }
    match c {
        'b' => Art::I8, 'B' => Art::U8, 's' => Art::I16, 'S' => Art::U16,
        'l' | 'o' => Art::I32, 'L' => Art::U32, 'q' => Art::I64,
        'f' => Art::F32, 'd' => Art::F64,
        'v' => Art::Nichts,
        _ => Art::Zeiger,
    }
}

/// Grenzen einer Ganzzahl-Art.
fn grenzen(c: char) -> Option<(i64, i64)> {
    Some(match c {
        'b' => (i8::MIN as i64, i8::MAX as i64),
        'B' => (0, u8::MAX as i64),
        's' => (i16::MIN as i64, i16::MAX as i64),
        'S' => (0, u16::MAX as i64),
        'l' => (i32::MIN as i64, i32::MAX as i64),
        'L' => (0, u32::MAX as i64),
        _ => return None,
    })
}

/// Eine Zahl, Kommazahl oder ein Wahrheitswert in den 8-Byte-Platz, wie C
/// ihn erwartet. Ein Wert, der nicht passt, ist ein Fehler -- still
/// abgeschnitten saehe die Bibliothek eine andere Zahl.
pub fn zahl_platz(c: char, v: &Value) -> Result<u64, String> {
    match c {
        'f' | 'd' => {
            let x = match v {
                Value::Float(f) => *f,
                Value::Int(i) => *i as f64,
                _ => return Err(format!("erwartet eine Zahl ({}), erhalten {}", typ_name(c), v.type_name())),
            };
            Ok(if c == 'f' { (x as f32).to_bits() as u64 } else { x.to_bits() })
        }
        'o' => match v {
            Value::Bool(b) => Ok(*b as u64),
            _ => Err(format!("erwartet einen Wahrheitswert (BOOLEAN), erhalten {}", v.type_name())),
        },
        'z' => match v {
            Value::Int(i) => Ok(*i as u64),
            Value::Nil => Ok(0),
            _ => Err(format!("erwartet einen Zeiger (ZEIGER, eine ganze Zahl), erhalten {}", v.type_name())),
        },
        _ => {
            let i = match v {
                Value::Int(i) => *i,
                _ => return Err(format!("erwartet eine ganze Zahl ({}), erhalten {}", typ_name(c), v.type_name())),
            };
            if let Some((lo, hi)) = grenzen(c) {
                if i < lo || i > hi {
                    return Err(format!("{} passt nicht in {} ({} bis {})", i, typ_name(c), lo, hi));
                }
            }
            Ok(i as u64)
        }
    }
}

/// Der Platz nach dem Aufruf zurueck in einen Wert (Rueckgabe und BYREF).
pub fn platz_wert(c: char, p: u64) -> Value {
    match c {
        'b' => Value::Int(p as u8 as i8 as i64),
        'B' => Value::Int(p as u8 as i64),
        's' => Value::Int(p as u16 as i16 as i64),
        'S' => Value::Int(p as u16 as i64),
        'l' => Value::Int(p as u32 as i32 as i64),
        'L' => Value::Int(p as u32 as i64),
        'o' => Value::Bool(p as u32 != 0),
        'f' => Value::Float(f32::from_bits(p as u32) as f64),
        'd' => Value::Float(f64::from_bits(p)),
        'v' => Value::Nil,
        _ => Value::Int(p as i64),
    }
}

/// `wchar_t` ist unter Windows 16 Bit breit (UTF-16), sonst 32 Bit.
#[cfg(windows)]
type WZeichen = u16;
#[cfg(not(windows))]
type WZeichen = u32;

fn breit_kodieren(s: &str) -> Vec<WZeichen> {
    #[cfg(windows)]
    let mut v: Vec<WZeichen> = s.encode_utf16().collect();
    #[cfg(not(windows))]
    let mut v: Vec<WZeichen> = s.chars().map(|c| c as u32).collect();
    v.push(0);
    v
}

/// Text aus einem Zeiger der Bibliothek (NULL = leer). Kopiert, gibt nichts frei.
unsafe fn text_aus(p: u64, breit: bool) -> String {
    if p == 0 { return String::new(); }
    if breit {
        let z = p as *const WZeichen;
        let mut n = 0usize;
        while *z.add(n) != 0 { n += 1; }
        let s = std::slice::from_raw_parts(z, n);
        #[cfg(windows)]
        { String::from_utf16_lossy(s) }
        #[cfg(not(windows))]
        { s.iter().map(|&c| char::from_u32(c).unwrap_or('\u{fffd}')).collect() }
    } else {
        std::ffi::CStr::from_ptr(p as *const std::ffi::c_char).to_string_lossy().into_owned()
    }
}

// ------------------------------------------------------------------ Uebergang

#[cfg(feature = "ffi")]
mod uebergang {
    use super::{Art, Laden, Rueckform, Stelle};
    use cranelift_codegen::ir::{types, AbiParam, ArgumentPurpose, InstBuilder, MemFlagsData, StackSlotData, StackSlotKind, Type, UserFuncName};
    use cranelift_codegen::settings::Configurable;
    use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
    use cranelift_jit::{JITBuilder, JITModule};
    use cranelift_module::{default_libcall_names, Linkage, Module};
    use std::collections::HashMap;

    pub type Einstieg = unsafe extern "C" fn(*const u8, *const u64, *mut u64);

    pub struct Bauer {
        modul: JITModule,
        fctx: FunctionBuilderContext,
        fertig: HashMap<(Vec<Stelle>, Rueckform), Einstieg>,
        rueckrufe: usize,
    }

    fn typ(a: Art, zeiger: Type) -> Type {
        match a {
            Art::I8 | Art::U8 => types::I8,
            Art::I16 | Art::U16 => types::I16,
            Art::I32 | Art::U32 => types::I32,
            Art::I64 => types::I64,
            Art::Zeiger | Art::Nichts => zeiger,
            Art::F32 => types::F32,
            Art::F64 => types::F64,
        }
    }

    /// Kleine Ganzzahlen tragen ihre Erweiterung in der Signatur -- unter
    /// System V und auf Apple-ARM verlaesst sich der Gerufene darauf.
    fn abi(a: Art, zeiger: Type) -> AbiParam {
        let p = AbiParam::new(typ(a, zeiger));
        match a {
            Art::I8 | Art::I16 | Art::I32 => p.sext(),
            Art::U8 | Art::U16 | Art::U32 => p.uext(),
            _ => p,
        }
    }

    impl Bauer {
        pub fn neu() -> Result<Bauer, String> {
            let mut flags = cranelift_codegen::settings::builder();
            flags.set("opt_level", "speed").map_err(|e| format!("Cranelift: {:?}", e))?;
            let isa = cranelift_native::builder().map_err(|e| format!("Cranelift: {}", e))?
                .finish(cranelift_codegen::settings::Flags::new(flags))
                .map_err(|e| format!("Cranelift: {:?}", e))?;
            let modul = JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()));
            Ok(Bauer { modul, fctx: FunctionBuilderContext::new(), fertig: HashMap::new(), rueckrufe: 0 })
        }

        /// Der Uebergang fuer Werte, die schlicht Platz fuer Platz liegen.
        pub fn holen(&mut self, params: &[Art], rueck: Art) -> Result<Einstieg, String> {
            let stellen: Vec<Stelle> = params.iter().enumerate()
                .map(|(i, a)| Stelle { art: *a, laden: Laden::Platz(i) }).collect();
            let rf = if rueck == Art::Nichts { Rueckform::Nichts } else { Rueckform::Wert(rueck) };
            self.holen_plan(&stellen, &rf)
        }

        /// Der Uebergang nach einem Plan (`super::plan`) -- mit Structs als
        /// Wert. Eine Rueckgabe in Teilen kommt an ihre Stellen ab `rueck`.
        pub fn holen_plan(&mut self, stellen: &[Stelle], rueck: &Rueckform) -> Result<Einstieg, String> {
            let schluessel = (stellen.to_vec(), rueck.clone());
            if let Some(e) = self.fertig.get(&schluessel) { return Ok(*e); }
            let zt = self.modul.target_config().pointer_type();
            let mut sig = self.modul.make_signature();
            for _ in 0..3 { sig.params.push(AbiParam::new(zt)); }
            let mut ziel_sig = self.modul.make_signature();
            for s in stellen {
                ziel_sig.params.push(match s.laden {
                    Laden::Kopie(_, n) => AbiParam::special(zt, ArgumentPurpose::StructArgument(n)),
                    Laden::Versteckt => AbiParam::special(zt, ArgumentPurpose::StructReturn),
                    _ => abi(s.art, zt),
                });
            }
            match rueck {
                Rueckform::Nichts => {}
                Rueckform::Wert(a) => ziel_sig.returns.push(abi(*a, zt)),
                Rueckform::Teile(t) => for (a, _) in t { ziel_sig.returns.push(AbiParam::new(typ(*a, zt))); },
            }
            let nr = self.fertig.len();
            let id = self.modul.declare_function(&format!("ffi_{}", nr), Linkage::Local, &sig)
                .map_err(|e| format!("Cranelift: {:?}", e))?;
            let tc = self.modul.target_config();
            let mut ctx = self.modul.make_context();
            ctx.func.signature = sig;
            ctx.func.name = UserFuncName::user(2, id.as_u32());
            {
                let mut fb = FunctionBuilder::new(&mut ctx.func, &mut self.fctx);
                let b = fb.create_block();
                fb.append_block_params_for_function_params(b);
                fb.switch_to_block(b);
                let p = fb.block_params(b).to_vec();
                let mut args = Vec::with_capacity(stellen.len());
                for s in stellen {
                    let t = typ(s.art, zt);
                    args.push(match s.laden {
                        Laden::Platz(k) => fb.ins().load(t, MemFlagsData::trusted(), p[1], (k * 8) as i32),
                        Laden::Ueber(k, o) => {
                            let z = fb.ins().load(zt, MemFlagsData::trusted(), p[1], (k * 8) as i32);
                            fb.ins().load(t, MemFlagsData::trusted(), z, o as i32)
                        }
                        Laden::Kopie(k, _) => fb.ins().load(zt, MemFlagsData::trusted(), p[1], (k * 8) as i32),
                        Laden::Null => match s.art {
                            Art::F32 => fb.ins().f32const(0.0),
                            Art::F64 => fb.ins().f64const(0.0),
                            _ => fb.ins().iconst(t, 0),
                        },
                        Laden::Versteckt => p[2],
                    });
                }
                let sref = fb.import_signature(ziel_sig);
                let ruf = fb.ins().call_indirect(sref, p[0], &args);
                match rueck {
                    Rueckform::Nichts => {}
                    Rueckform::Wert(a) => {
                        let w = fb.inst_results(ruf)[0];
                        let w = match a {
                            Art::I8 | Art::I16 | Art::I32 => fb.ins().sextend(types::I64, w),
                            Art::U8 | Art::U16 | Art::U32 => fb.ins().uextend(types::I64, w),
                            _ => w,
                        };
                        fb.ins().store(MemFlagsData::trusted(), w, p[2], 0);
                    }
                    Rueckform::Teile(t) => {
                        let ws = fb.inst_results(ruf).to_vec();
                        for ((_, o), w) in t.iter().zip(ws) {
                            fb.ins().store(MemFlagsData::trusted(), w, p[2], *o as i32);
                        }
                    }
                }
                fb.ins().return_(&[]);
                fb.seal_all_blocks();
                fb.finalize(tc);
            }
            self.modul.define_function(id, &mut ctx).map_err(|e| format!("Cranelift: {:?}", e))?;
            self.modul.clear_context(&mut ctx);
            self.modul.finalize_definitions().map_err(|e| format!("Cranelift: {:?}", e))?;
            let z = self.modul.get_finalized_function(id);
            let e: Einstieg = unsafe { std::mem::transmute::<*const u8, Einstieg>(z) };
            self.fertig.insert(schluessel, e);
            Ok(e)
        }

        /// Ein Einstieg fuer die Bibliothek: eine Funktion mit der C-Signatur
        /// des Rueckrufs, die ihre Argumente in 8-Byte-Plaetze legt und
        /// `eingang(nummer, plaetze, rueck)` ruft. Jeder Rueckruf bekommt
        /// seinen eigenen (die Nummer steht als Konstante darin); er bleibt
        /// bis zum Ende des Programms -- die Bibliothek darf ihn behalten.
        pub fn rueckruf(&mut self, params: &[Art], rueck: Art, nummer: u64, eingang: usize)
            -> Result<*const u8, String> {
            let zt = self.modul.target_config().pointer_type();
            let mut sig = self.modul.make_signature();
            for a in params { sig.params.push(abi(*a, zt)); }
            if rueck != Art::Nichts { sig.returns.push(abi(rueck, zt)); }
            let mut ein_sig = self.modul.make_signature();
            for _ in 0..3 { ein_sig.params.push(AbiParam::new(zt)); }
            self.rueckrufe += 1;
            let id = self.modul.declare_function(&format!("rr_{}", self.rueckrufe), Linkage::Local, &sig)
                .map_err(|e| format!("Cranelift: {:?}", e))?;
            let tc = self.modul.target_config();
            let mut ctx = self.modul.make_context();
            ctx.func.signature = sig;
            ctx.func.name = UserFuncName::user(3, id.as_u32());
            {
                let mut fb = FunctionBuilder::new(&mut ctx.func, &mut self.fctx);
                let b = fb.create_block();
                fb.append_block_params_for_function_params(b);
                fb.switch_to_block(b);
                let p = fb.block_params(b).to_vec();
                // Platz n = Rueckgabe, davor die Argumente.
                let groesse = ((params.len() + 1) * 8) as u32;
                let slot = fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, groesse, 3));
                let null = fb.ins().iconst(types::I64, 0);
                fb.ins().stack_store(zt, null, slot, (params.len() * 8) as i32);
                for (i, a) in params.iter().enumerate() {
                    // Ganzzahlen auf 64 Bit erweitert -- `platz_wert` liest
                    // die unteren Bytes, die Erweiterung schadet nicht.
                    let w = match a {
                        Art::I8 | Art::I16 | Art::I32 => fb.ins().sextend(types::I64, p[i]),
                        Art::U8 | Art::U16 | Art::U32 => fb.ins().uextend(types::I64, p[i]),
                        _ => p[i],
                    };
                    fb.ins().stack_store(zt, w, slot, (i * 8) as i32);
                }
                let plaetze = fb.ins().stack_addr(zt, slot, 0);
                let rueckplatz = fb.ins().stack_addr(zt, slot, (params.len() * 8) as i32);
                let nr = fb.ins().iconst(zt, nummer as i64);
                let ziel = fb.ins().iconst(zt, eingang as i64);
                let sref = fb.import_signature(ein_sig);
                fb.ins().call_indirect(sref, ziel, &[nr, plaetze, rueckplatz]);
                if rueck != Art::Nichts {
                    let w = fb.ins().load(typ(rueck, zt), MemFlagsData::trusted(), rueckplatz, 0);
                    fb.ins().return_(&[w]);
                } else {
                    fb.ins().return_(&[]);
                }
                fb.seal_all_blocks();
                fb.finalize(tc);
            }
            self.modul.define_function(id, &mut ctx).map_err(|e| format!("Cranelift: {:?}", e))?;
            self.modul.clear_context(&mut ctx);
            self.modul.finalize_definitions().map_err(|e| format!("Cranelift: {:?}", e))?;
            Ok(self.modul.get_finalized_function(id))
        }
    }
}

// ------------------------------------------------------------------ Laden

#[cfg(feature = "ffi")]
struct Aufruf {
    sig: Signatur,
    ziel: *const u8,
    einstieg: uebergang::Einstieg,
    /// Je Parameter seine Lage, wenn er ein Struct als Wert ist.
    structs: Vec<Option<StructWert>>,
    /// Die Lage des Structs, den die Funktion als Wert liefert.
    rueck_struct: Option<StructWert>,
}

#[cfg(feature = "ffi")]
#[derive(Default)]
struct Zustand {
    bauer: Option<uebergang::Bauer>,
    /// Geladene Bibliotheken, je Name in der DECLARE-Zeile. Sie bleiben bis
    /// zum Ende geladen -- ein Zeiger in sie hinein darf nie ins Leere zeigen.
    libs: HashMap<String, Rc<libloading::Library>>,
    aufrufe: HashMap<String, Rc<Aufruf>>,
    /// Die Rueckrufe, die je einer Bibliothek gegeben wurden; die Nummer im
    /// Einstieg ist der Platz hier. Nie entfernt -- die Bibliothek darf den
    /// Einstieg behalten, und der Wert haelt die Funktion (samt Objekt) am
    /// Leben.
    rueckrufe: Vec<Rc<Rueckruf>>,
}

#[cfg(feature = "ffi")]
struct Rueckruf {
    /// Rueckgabe, dann die Parameter (wie `Param::rr`).
    sig: String,
    wert: Value,
    /// Der Name fuer Meldungen.
    name: String,
    einstieg: usize,
}

#[cfg(feature = "ffi")]
thread_local! {
    static ZUSTAND: RefCell<Zustand> = RefCell::new(Zustand::default());
    /// Die VM, die die Funktionen der Rueckrufe ausfuehrt -- gesetzt vor
    /// jedem Aufruf einer Bibliothek (wie `Kontext::vm` im Maschinencode).
    static VM: std::cell::Cell<*mut crate::vm::Vm<'static>> = const { std::cell::Cell::new(std::ptr::null_mut()) };
    /// Der Fehler eines Rueckrufs, bis der Aufruf der Bibliothek ihn meldet.
    static RR_FEHLER: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Der Faden des Programms. Ein Rueckruf aus einem anderen wird nicht
/// ausgefuehrt (die VM ist nicht fuer mehrere Faeden gebaut) -- er liefert 0,
/// und der naechste Aufruf einer Bibliothek meldet es.
#[cfg(feature = "ffi")]
static HAUPTFADEN: std::sync::OnceLock<std::thread::ThreadId> = std::sync::OnceLock::new();
#[cfg(feature = "ffi")]
static FREMDER_FADEN: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Die VM fuer die Rueckrufe merken (vor jedem `__ffi`).
pub fn vm_setzen(vm: *mut crate::vm::Vm<'_>) {
    #[cfg(feature = "ffi")]
    {
        let _ = HAUPTFADEN.get_or_init(|| std::thread::current().id());
        VM.with(|v| v.set(vm as *mut crate::vm::Vm<'static>));
    }
    #[cfg(not(feature = "ffi"))]
    let _ = vm;
}

/// Dieselbe Funktion wie ein schon vergebener Rueckruf? Dann bekommt die
/// Bibliothek denselben Einstieg -- `qsort` in einer Schleife baut nicht je
/// Runde einen neuen. Ein Lambda, das jedes Mal neu entsteht, ist jedes Mal
/// ein anderes.
#[cfg(feature = "ffi")]
fn gleiche_funktion(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::FuncRef(x), Value::FuncRef(y)) => x == y,
        (Value::Closure(x), Value::Closure(y)) => Rc::ptr_eq(x, y),
        (Value::BoundMethod(x), Value::BoundMethod(y)) => x.1 == y.1 && match (&x.0, &y.0) {
            (Value::Instance(i), Value::Instance(j)) => Rc::ptr_eq(i, j),
            _ => Rc::ptr_eq(x, y),
        },
        _ => false,
    }
}

#[cfg(feature = "ffi")]
fn funktionsname(v: &Value) -> String {
    match v {
        Value::FuncRef(n) => n.to_string(),
        Value::BoundMethod(b) => b.1.to_string(),
        _ => "lambda".into(),
    }
}

/// Der Einstieg fuer einen Rueckruf-Parameter: einmal je (Signatur, Funktion).
#[cfg(feature = "ffi")]
fn rueckruf_einstieg(rr: &str, wert: &Value) -> Result<u64, String> {
    if let Some(e) = ZUSTAND.with(|z| z.borrow().rueckrufe.iter()
            .find(|r| r.sig == rr && gleiche_funktion(&r.wert, wert)).map(|r| r.einstieg)) {
        return Ok(e as u64);
    }
    let mut zeichen = rr.chars();
    let rueck = zeichen.next().unwrap_or('v');
    let params: Vec<char> = zeichen.collect();
    // Passt die Zahl der Parameter? Vorher fragen -- im Rueckruf waere es ein
    // Fehler, den die Bibliothek erst nach der ganzen Arbeit zurueckgibt.
    let vm = VM.with(|v| v.get());
    if !vm.is_null() {
        if let Some(n) = unsafe { (*vm).wert_stellen(wert, &funktionsname(wert))? } {
            if n != params.len() {
                return Err(format!("die Funktion {} nimmt {} Wert(e), der Rueckruf uebergibt {}",
                                   funktionsname(wert), n, params.len()));
            }
        }
    }
    let arten: Vec<Art> = params.iter().map(|c| art_von(*c, false)).collect();
    let nummer = ZUSTAND.with(|z| z.borrow().rueckrufe.len()) as u64;
    let einstieg = ZUSTAND.with(|z| -> Result<_, String> {
        let mut z = z.borrow_mut();
        if z.bauer.is_none() { z.bauer = Some(uebergang::Bauer::neu()?); }
        z.bauer.as_mut().unwrap().rueckruf(&arten, art_von(rueck, false), nummer, eingang as usize)
    })? as usize;
    ZUSTAND.with(|z| z.borrow_mut().rueckrufe.push(Rc::new(Rueckruf {
        sig: rr.to_string(), wert: wert.clone(), name: funktionsname(wert), einstieg })));
    Ok(einstieg as u64)
}

/// Hierher springt jeder Einstieg. Kein Fehler und kein Panic darf durch
/// die Rahmen der Bibliothek zurueck -- beides wird gemerkt.
#[cfg(feature = "ffi")]
extern "C" fn eingang(nummer: u64, plaetze: *const u64, rueck: *mut u64) {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { eingang_innen(nummer, plaetze, rueck) }));
    if r.is_err() {
        unsafe { *rueck = 0; }
        RR_FEHLER.with(|f| { f.borrow_mut().get_or_insert_with(|| "interner Fehler im Rueckruf".into()); });
    }
}

#[cfg(feature = "ffi")]
unsafe fn eingang_innen(nummer: u64, plaetze: *const u64, rueck: *mut u64) {
    *rueck = 0;
    if HAUPTFADEN.get() != Some(&std::thread::current().id()) {
        if let Ok(mut f) = FREMDER_FADEN.lock() {
            f.get_or_insert_with(|| "ein Rueckruf kam aus einem anderen Faden und wurde nicht ausgefuehrt -- Drachenhauch-Code laeuft nur auf dem Faden des Programms".into());
        }
        return;
    }
    if RR_FEHLER.with(|f| f.borrow().is_some()) { return; }
    let Some(rr) = ZUSTAND.with(|z| z.borrow().rueckrufe.get(nummer as usize).cloned()) else { return; };
    let setze = |m: String| RR_FEHLER.with(|f| *f.borrow_mut() = Some(m));
    let vm = VM.with(|v| v.get());
    if vm.is_null() { setze(format!("Rueckruf {}: keine laufende VM", rr.name)); return; }
    let mut zeichen = rr.sig.chars();
    let r = zeichen.next().unwrap_or('v');
    let werte: Vec<Value> = zeichen.enumerate().map(|(i, c)| {
        let p = *plaetze.add(i);
        match c {
            't' => Value::str_rc(text_aus(p, false)),
            'w' => Value::str_rc(text_aus(p, true)),
            c => platz_wert(c, p),
        }
    }).collect();
    match (*vm).wert_rufen(&rr.wert, werte, &rr.name) {
        Ok(v) => {
            if r != 'v' {
                match zahl_platz(r, &v) {
                    Ok(p) => *rueck = p,
                    Err(e) => setze(format!("der Rueckruf {} soll {} liefern: {}", rr.name, typ_name(r), e)),
                }
            }
        }
        Err(e) => setze(format!("Fehler im Rueckruf {}: {}", rr.name, e)),
    }
}

/// Nach dem Aufruf einer Bibliothek: hat ein Rueckruf einen Fehler gemerkt?
#[cfg(feature = "ffi")]
fn rueckruf_fehler() -> Option<String> {
    if let Some(e) = RR_FEHLER.with(|f| f.borrow_mut().take()) { return Some(e); }
    FREMDER_FADEN.lock().ok().and_then(|mut f| f.take())
}

#[cfg(feature = "ffi")]
fn bibliothek(name: &str) -> Result<Rc<libloading::Library>, String> {
    let namen = dateinamen(name, system());
    let mut orte: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(d) = std::env::current_dir() { orte.push(d); }
    if let Some(d) = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())) {
        if !orte.contains(&d) { orte.push(d); }
    }
    // Lag eine Datei da und liess sich nur nicht laden, ist das ein anderer
    // Fehler als "nicht gefunden" -- meist fehlt ihr selbst eine Bibliothek.
    let mut nicht_ladbar: Option<(String, String)> = None;
    for n in &namen {
        let pfad = std::path::Path::new(n);
        let mut kandidaten: Vec<std::path::PathBuf> = Vec::new();
        if pfad.is_relative() {
            for o in &orte {
                let k = o.join(pfad);
                if k.exists() { kandidaten.push(k); }
            }
        }
        kandidaten.push(pfad.to_path_buf());
        for k in kandidaten {
            match unsafe { laden(&k) } {
                Ok(l) => return Ok(Rc::new(l)),
                Err(e) => if k.is_file() && nicht_ladbar.is_none() {
                    nicht_ladbar = Some((k.display().to_string(), e.to_string()));
                },
            }
        }
    }
    if let Some((datei, grund)) = nicht_ladbar {
        return Err(format!("Bibliothek \"{}\" liegt als {} da, laesst sich aber nicht laden -- fehlt ihr selbst eine Bibliothek, oder passt sie nicht zu diesem System (64 Bit)? Das System sagt: {}",
                           name, datei, grund));
    }
    Err(format!("Bibliothek \"{}\" nicht gefunden (versucht: {} -- neben dem Programm, neben dhrt und wo das System sucht)",
                name, namen.join(", ")))
}

/// Eine Bibliothek laden. Unter Windows sucht ein voller Pfad ihre eigenen
/// Abhaengigkeiten zuerst in IHREM Ordner (`LOAD_WITH_ALTERED_SEARCH_PATH`)
/// -- sonst faende `C:/.../libgtk-3-0.dll` die `libglib` daneben nicht, und
/// man muesste den Ordner in den PATH legen.
#[cfg(feature = "ffi")]
unsafe fn laden(pfad: &std::path::Path) -> Result<libloading::Library, libloading::Error> {
    // Die Suche im eigenen Ordner geht nur mit Rueckstrichen im Pfad.
    #[cfg(windows)]
    if pfad.is_absolute() {
        let mit_rueckstrich = pfad.to_string_lossy().replace('/', "\\");
        return libloading::os::windows::Library::load_with_flags(
            &mit_rueckstrich, libloading::os::windows::LOAD_WITH_ALTERED_SEARCH_PATH).map(Into::into);
    }
    libloading::Library::new(pfad)
}

#[cfg(feature = "ffi")]
fn aufruf(text: &str) -> Result<Rc<Aufruf>, String> {
    if let Some(a) = ZUSTAND.with(|z| z.borrow().aufrufe.get(text).cloned()) { return Ok(a); }
    let sig = signatur_lesen(text)?;
    let lib = match ZUSTAND.with(|z| z.borrow().libs.get(&sig.lib.to_lowercase()).cloned()) {
        Some(l) => l,
        None => {
            let l = bibliothek(&sig.lib)?;
            ZUSTAND.with(|z| z.borrow_mut().libs.insert(sig.lib.to_lowercase(), l.clone()));
            l
        }
    };
    let ziel: *const u8 = unsafe {
        match lib.get::<unsafe extern "C" fn()>(sig.c_name.as_bytes()) {
            Ok(f) => *f as *const u8,
            Err(_) => return Err(format!("die Funktion '{}' gibt es in \"{}\" nicht (Gross/klein zaehlt{})",
                                         sig.c_name, sig.lib,
                                         if cfg!(windows) { "; Text-Funktionen der Windows-API heissen ...A oder ...W" } else { "" })),
        }
    };
    let a = Rc::new(aufruf_bauen(sig, ziel)?);
    ZUSTAND.with(|z| z.borrow_mut().aufrufe.insert(text.to_string(), a.clone()));
    Ok(a)
}

/// Plan und Uebergang fuer eine Signatur, deren Ziel schon feststeht.
#[cfg(feature = "ffi")]
fn aufruf_bauen(sig: Signatur, ziel: *const u8) -> Result<Aufruf, String> {
    // Bei `...` steht der Uebergang fuer die festen Parameter; jeder Aufruf
    // mit weiteren Werten holt sich seinen eigenen (`variadisch_rufen`).
    let structs: Vec<Option<StructWert>> = sig.params.iter()
        .map(|p| if p.art == 'x' { struct_lesen(&p.rr) } else { None }).collect();
    let rueck_struct = if sig.rueck == 'x' { struct_lesen(&sig.rueck_rr) } else { None };
    if (structs.iter().any(Option::is_some) || rueck_struct.is_some()) && sig.params.iter().any(|p| p.art == '*') {
        return Err("ein Struct als Wert geht (noch) nicht in einer Funktion mit ... -- uebergib ihn dort als Zeiger (ohne BYVAL)".into());
    }
    let formen: Vec<Form> = sig.params.iter().zip(&structs).filter(|(p, _)| p.art != '*')
        .map(|(p, s)| match s { Some(s) => Form::Struct(s), None => Form::Skalar(art_von(p.art, p.byref)) }).collect();
    let rueck = match &rueck_struct { Some(s) => Form::Struct(s), None => Form::Skalar(art_von(sig.rueck, false)) };
    let (stellen, rf) = plan(konvention(), &formen, rueck)?;
    let einstieg = ZUSTAND.with(|z| -> Result<_, String> {
        let mut z = z.borrow_mut();
        if z.bauer.is_none() { z.bauer = Some(uebergang::Bauer::neu()?); }
        z.bauer.as_mut().unwrap().holen_plan(&stellen, &rf)
    })?;
    Ok(Aufruf { sig, ziel, einstieg, structs, rueck_struct })
}

/// Der Befehl `__ffi(signatur, argumente...)`. Ohne BYREF das Ergebnis,
/// mit BYREF ein Tupel `(ergebnis, letzter_byref, ..., erster_byref)` --
/// so legt `UNPACK_TUPLE` die Werte in der Reihenfolge auf den Stapel, die
/// das Zurueckschreiben des Compilers erwartet.
pub fn rufen(a: &[Value]) -> Result<Value, String> {
    // Ohne das Feature zuerst DAS sagen, vor jeder anderen Pruefung: die
    // Probe von `--export --schlank` (`--fehlende`, 40 x NIL) erkennt einen
    // fehlenden Befehl an "ist ohne das Feature" (vm::fehlt_im_bau).
    #[cfg(not(feature = "ffi"))]
    {
        let name = match a.first() {
            Some(Value::Str(t)) => signatur_lesen(t).map(|s| s.name).unwrap_or_default(),
            _ => String::new(),
        };
        if cfg!(target_os = "emscripten") {
            return Err(format!("{}: fremde Bibliotheken (DECLARE ... LIB) gibt es im Browser nicht", name));
        }
        return Err(format!("{}: fremde Bibliotheken (DECLARE ... LIB) gehen in diesem Bau nicht -- er ist ohne das Feature ffi gebaut", name));
    }
    #[cfg(feature = "ffi")]
    {
        let Some(Value::Str(text)) = a.first() else { return Err("fremde Funktion: Signatur fehlt".into()); };
        let auf = match aufruf(text) {
            Ok(x) => x,
            Err(e) => {
                let name = signatur_lesen(text).map(|s| s.name).unwrap_or_default();
                return Err(format!("{}: {}", name, e));
            }
        };
        rufen_mit(&auf, &a[1..])
    }
}

/// Die Befehle fuer Zeiger einer Bibliothek (Stufe 2): `TEXT_AUS_ZEIGER$`,
/// `BUFFER_AUS_ZEIGER`, `BUFFER_ZEIGER`. `None` = kein Befehl dieser Familie.
/// Alle drei vertrauen dem Zeiger -- ein falscher beendet das Programm wie
/// jeder Fehler in fremdem Code; geprueft wird, was sich pruefen laesst
/// (Nullzeiger, Laenge, Obergrenze).
pub fn zeiger_befehl(name: &str, a: &[Value]) -> Option<Result<Value, String>> {
    let gross = match name {
        "text_aus_zeiger$" => "TEXT_AUS_ZEIGER$",
        "buffer_aus_zeiger" => "BUFFER_AUS_ZEIGER",
        "buffer_zeiger" => "BUFFER_ZEIGER",
        _ => return None,
    };
    #[cfg(not(feature = "ffi"))]
    {
        let _ = a;
        return Some(Err(format!("{}: Zeiger gibt es nur mit fremden Bibliotheken -- dieser Bau ist ohne das Feature ffi gebaut", gross)));
    }
    #[cfg(feature = "ffi")]
    Some(zeiger_rufen(gross, a))
}

#[cfg(feature = "ffi")]
fn zeiger_rufen(gross: &str, a: &[Value]) -> Result<Value, String> {
    let argzahl = |von: usize, bis: usize| -> Result<(), String> {
        if a.len() < von || a.len() > bis {
            let soll = if von == bis { von.to_string() } else { format!("{} bis {}", von, bis) };
            return Err(format!("{}: erwartet {} Argument(e), erhalten {}", gross, soll, a.len()));
        }
        Ok(())
    };
    let zeiger = |v: &Value| -> Result<u64, String> {
        match v {
            Value::Int(i) => Ok(*i as u64),
            Value::Nil => Ok(0),
            _ => Err(format!("{}: erwartet einen Zeiger (INTEGER), erhalten {}", gross, v.type_name())),
        }
    };
    match gross {
        "TEXT_AUS_ZEIGER$" => {
            argzahl(1, 2)?;
            let p = zeiger(&a[0])?;
            let breit = match a.get(1) {
                None => false,
                Some(Value::Bool(b)) => *b,
                Some(Value::Int(i)) => *i != 0,
                Some(v) => return Err(format!("{}: breit erwartet TRUE/FALSE, erhalten {}", gross, v.type_name())),
            };
            Ok(Value::str_rc(unsafe { text_aus(p, breit) }))
        }
        "BUFFER_AUS_ZEIGER" => {
            argzahl(2, 2)?;
            let p = zeiger(&a[0])?;
            let n = match &a[1] {
                Value::Int(i) => *i,
                v => return Err(format!("{}: die Laenge erwartet INTEGER, erhalten {}", gross, v.type_name())),
            };
            if n < 0 { return Err(format!("{}: Laenge {} ist negativ", gross, n)); }
            if n > 1 << 30 {
                return Err(format!("{}: Laenge {} ueberschreitet die Obergrenze von {} Bytes", gross, n, 1i64 << 30));
            }
            if n == 0 { return Ok(crate::builtins::neuer_buffer(Vec::new())); }
            if p == 0 { return Err(format!("{}: der Zeiger ist 0 (NULL) -- dort liegen keine {} Bytes", gross, n)); }
            let bytes = unsafe { std::slice::from_raw_parts(p as *const u8, n as usize) }.to_vec();
            Ok(crate::builtins::neuer_buffer(bytes))
        }
        _ => {
            argzahl(1, 1)?;
            match &a[0] {
                // Ein leerer Puffer hat keine Bytes, auf die man zeigen koennte.
                Value::Buffer(b) => {
                    let b = b.borrow();
                    Ok(Value::Int(if b.is_empty() { 0 } else { b.as_ptr() as usize as i64 }))
                }
                v => Err(format!("{}: erwartet einen BUFFER, erhalten {}", gross, v.type_name())),
            }
        }
    }
}

/// Ob eine Kommazahl hinter `...` in einem Gleitkomma-Register reist. Unter
/// Windows x64 liest die gerufene Funktion sie aus dem Speicher, in den sie
/// die Ganzzahl-Register sichert, und auf Apple-ARM liegt alles hinter `...`
/// auf dem Stapel -- dort genuegt ihr Bitmuster als Ganzzahl.
#[cfg(feature = "ffi")]
const KOMMA_IN_FP_REGISTER: bool = !cfg!(any(windows, all(target_os = "macos", target_arch = "aarch64")));

/// Ein Aufruf mit Werten hinter `...`. Cranelift kennt keine variadischen
/// Aufrufe; jedes System bekommt darum eine gewoehnliche Signatur, die
/// genauso aufgerufen wird:
///
/// * Windows x64, Linux ARM: die weiteren Werte als gewoehnliche Parameter.
/// * Linux/macOS x86-64: dasselbe, aber ueber das Sprungbrett, das `al`
///   setzt (System V verlangt dort die Zahl der Vektor-Register).
/// * macOS ARM: die festen in die Register, die restlichen der acht
///   Ganzzahl-Register mit Nullen auffuellen -- dann landen die weiteren
///   Werte auf dem Stapel, wo Apple sie hinter `...` erwartet.
#[cfg(feature = "ffi")]
fn variadisch_rufen(auf: &Aufruf, fest: usize, plaetze: &[u64], weitere: &[Art], rueck: &mut u64) -> Result<(), String> {
    let sig = &auf.sig;
    let mut arten: Vec<Art> = sig.params[..fest].iter().map(|p| art_von(p.art, p.byref)).collect();
    let mut werte: Vec<u64> = plaetze[..fest].to_vec();
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        let ganz = arten.iter().filter(|a| !matches!(a, Art::F32 | Art::F64)).count();
        for _ in ganz..8 { arten.push(Art::I64); werte.push(0); }
    }
    arten.extend_from_slice(weitere);
    werte.extend_from_slice(&plaetze[fest..]);
    let einstieg = ZUSTAND.with(|z| -> Result<_, String> {
        let mut z = z.borrow_mut();
        if z.bauer.is_none() { z.bauer = Some(uebergang::Bauer::neu()?); }
        z.bauer.as_mut().unwrap().holen(&arten, art_von(sig.rueck, false))
    })?;
    let ziel = sprungbrett::ziel(auf.ziel);
    unsafe { einstieg(ziel, werte.as_ptr(), rueck) };
    Ok(())
}

/// System V x86-64: vor einem variadischen Aufruf steht in `al` eine obere
/// Grenze fuer die benutzten Vektor-Register (0..8) -- die gerufene Funktion
/// sichert danach ihre XMM-Register. Cranelift setzt `al` nicht; das
/// Sprungbrett setzt 8 (immer erlaubt) und springt zum eigentlichen Ziel,
/// das kurz vorher in `ZIEL` steht (ein Faden, darum genuegt eine Stelle).
#[cfg(all(feature = "ffi", target_arch = "x86_64", not(windows)))]
mod sprungbrett {
    pub static mut ZIEL: usize = 0;
    #[cfg(target_os = "macos")]
    std::arch::global_asm!(".text", ".globl _dh_vararg_sprung", "_dh_vararg_sprung:",
                           "mov al, 8", "jmp qword ptr [rip + {z}]", z = sym ZIEL);
    #[cfg(not(target_os = "macos"))]
    std::arch::global_asm!(".text", ".globl dh_vararg_sprung", "dh_vararg_sprung:",
                           "mov al, 8", "jmp qword ptr [rip + {z}]", z = sym ZIEL);
    extern "C" { fn dh_vararg_sprung(); }

    pub fn ziel(echt: *const u8) -> *const u8 {
        unsafe { *std::ptr::addr_of_mut!(ZIEL) = echt as usize; }
        dh_vararg_sprung as *const u8
    }
}

#[cfg(all(feature = "ffi", not(all(target_arch = "x86_64", not(windows)))))]
mod sprungbrett {
    pub fn ziel(echt: *const u8) -> *const u8 { echt }
}

#[cfg(feature = "ffi")]
fn rufen_mit(auf: &Aufruf, args: &[Value]) -> Result<Value, String> {
    let sig = &auf.sig;
    let variadisch = sig.params.last().map(|p| p.art == '*').unwrap_or(false);
    let fest = if variadisch { sig.params.len() - 1 } else { sig.params.len() };
    if variadisch && args.len() < fest {
        return Err(format!("{}: erwartet mindestens {} Argument(e), erhalten {}", sig.name, fest, args.len()));
    }
    if !variadisch && args.len() != fest {
        return Err(format!("{}: erwartet {} Argument(e), erhalten {}", sig.name, fest, args.len()));
    }
    let n = args.len();
    let mut plaetze = vec![0u64; n.max(1)];
    // Was fuer die Dauer des Aufrufs leben muss: kopierte Texte und die
    // Plaetze der BYREF-Werte (feste Groesse, damit ihre Adressen halten).
    let mut texte: Vec<std::ffi::CString> = Vec::new();
    let mut breite: Vec<Vec<WZeichen>> = Vec::new();
    let mut ref_plaetze = vec![0u64; n.max(1)];
    let mut kopien: Vec<Vec<u64>> = Vec::new();
    for (i, (p, v)) in sig.params[..fest].iter().zip(args).enumerate() {
        let typname = match &auf.structs[i] { Some(s) => s.name.to_uppercase(), None => typ_name(p.art).to_string() };
        let fehler = |e: String| format!("{}: Argument {} ({} AS {}): {}", sig.name, i + 1, p.name, typname, e);
        if let Some(s) = &auf.structs[i] {
            // Als Wert: die Funktion bekommt eine Kopie -- was sie daran
            // aendert, sieht das Programm nicht. In 8-Byte-Worten, damit jedes
            // Laden (auch ueber das Ende eines Structs mit 12 Bytes hinaus)
            // im Speicher bleibt und die Kopie ausgerichtet ist.
            let b = match v {
                Value::Buffer(b) => b.borrow(),
                Value::Nil => return Err(fehler("der Struct ist NIL -- er bekommt seinen Speicher mit DIM".into())),
                _ => return Err(fehler(format!("erwartet einen Struct {} (einen BUFFER), erhalten {}", s.name.to_uppercase(), v.type_name()))),
            };
            if b.len() < s.groesse as usize {
                return Err(fehler(format!("der Puffer ist {} Bytes lang, der Struct braucht {}", b.len(), s.groesse)));
            }
            let mut k = vec![0u64; (s.groesse as usize).div_ceil(8).max(2)];
            unsafe { std::ptr::copy_nonoverlapping(b.as_ptr(), k.as_mut_ptr() as *mut u8, s.groesse as usize); }
            plaetze[i] = k.as_ptr() as u64;
            kopien.push(k);
            continue;
        }
        if p.byref {
            ref_plaetze[i] = zahl_platz(p.art, v).map_err(fehler)?;
            plaetze[i] = (&mut ref_plaetze[i] as *mut u64) as u64;
            continue;
        }
        plaetze[i] = match p.art {
            't' => match v {
                Value::Str(s) => {
                    let c = std::ffi::CString::new(s.as_bytes())
                        .map_err(|_| fehler("ein Nullzeichen mitten im Text -- C saehe nur den Anfang".into()))?;
                    let z = c.as_ptr() as u64;
                    texte.push(c);
                    z
                }
                Value::Nil => 0,
                _ => return Err(fehler(format!("erwartet einen Text (STRING), erhalten {}", v.type_name()))),
            },
            'w' => match v {
                Value::Str(s) => {
                    if s.contains('\0') {
                        return Err(fehler("ein Nullzeichen mitten im Text -- C saehe nur den Anfang".into()));
                    }
                    let w = breit_kodieren(s);
                    let z = w.as_ptr() as u64;
                    breite.push(w);
                    z
                }
                Value::Nil => 0,
                _ => return Err(fehler(format!("erwartet einen Text (STRING), erhalten {}", v.type_name()))),
            },
            'p' => match v {
                Value::Buffer(b) => unsafe { (*b.as_ptr()).as_mut_ptr() as u64 },
                Value::Nil => 0,
                _ => return Err(fehler(format!("erwartet einen BUFFER, erhalten {}", v.type_name()))),
            },
            'r' => match v {
                Value::FuncRef(_) | Value::BoundMethod(_) | Value::Closure(_) =>
                    rueckruf_einstieg(&p.rr, v).map_err(fehler)?,
                Value::Nil => 0,
                _ => return Err(fehler(format!("erwartet eine Funktion (FUNCREF, z.B. den Namen einer FUNCTION ohne Klammern), erhalten {}",
                                               v.type_name()))),
            },
            c => zahl_platz(c, v).map_err(fehler)?,
        };
    }
    // Die weiteren Werte hinter `...`: ihr Typ kommt aus dem Wert -- C kennt
    // dort ohnehin nur int/long, double und Zeiger.
    let mut weitere: Vec<Art> = Vec::new();
    for (i, v) in args[fest..].iter().enumerate() {
        let fehler = |e: String| format!("{}: Argument {} (hinter ...): {}", sig.name, fest + i + 1, e);
        let (platz, komma) = match v {
            Value::Int(x) => (*x as u64, false),
            Value::Float(f) => (f.to_bits(), true),
            Value::Bool(b) => (*b as u64, false),
            Value::Nil => (0, false),
            Value::Str(s) => {
                let c = std::ffi::CString::new(s.as_bytes())
                    .map_err(|_| fehler("ein Nullzeichen mitten im Text -- C saehe nur den Anfang".into()))?;
                let z = c.as_ptr() as u64;
                texte.push(c);
                (z, false)
            }
            Value::Buffer(b) => (unsafe { (*b.as_ptr()).as_mut_ptr() as u64 }, false),
            _ => return Err(fehler(format!("erwartet eine Zahl, einen Text, einen BUFFER oder NIL, erhalten {}", v.type_name()))),
        };
        plaetze[fest + i] = platz;
        weitere.push(if komma && KOMMA_IN_FP_REGISTER { Art::F64 } else { Art::I64 });
    }
    // Der Platz fuer die Rueckgabe: ein Wert, die Teile eines Structs aus
    // den Registern oder der ganze Struct, den die Funktion selbst schreibt.
    let worte = auf.rueck_struct.as_ref().map(|s| (s.groesse as usize).div_ceil(8)).unwrap_or(0).max(4);
    let mut rueck_platz = vec![0u64; worte];
    if variadisch {
        let mut r = 0u64;
        variadisch_rufen(auf, fest, &plaetze[..n], &weitere, &mut r)?;
        rueck_platz[0] = r;
    } else {
        unsafe { (auf.einstieg)(auf.ziel, plaetze.as_ptr(), rueck_platz.as_mut_ptr()) };
    }
    drop(texte);
    drop(breite);
    drop(kopien);
    if let Some(e) = rueckruf_fehler() { return Err(format!("{}: {}", sig.name, e)); }
    let rueck = rueck_platz[0];
    let ergebnis = match sig.rueck {
        'x' => {
            let g = auf.rueck_struct.as_ref().map(|s| s.groesse as usize).unwrap_or(0);
            let bytes = unsafe { std::slice::from_raw_parts(rueck_platz.as_ptr() as *const u8, g) }.to_vec();
            crate::builtins::neuer_buffer(bytes)
        }
        't' => Value::str_rc(unsafe { text_aus(rueck, false) }),
        'w' => Value::str_rc(unsafe { text_aus(rueck, true) }),
        c => platz_wert(c, rueck),
    };
    if !sig.params.iter().any(|p| p.byref) { return Ok(ergebnis); }
    let mut tupel = vec![ergebnis];
    for (i, p) in sig.params.iter().enumerate().rev() {
        if p.byref { tupel.push(platz_wert(p.art, ref_plaetze[i])); }
    }
    Ok(Value::Tuple(Rc::new(tupel)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatur_hin_und_zurueck() {
        let t = signatur_text("user32", "MessageBoxW", "box", "l",
                              &[("z".into(), false, "f".into()), ("w".into(), false, "t".into()), ("l".into(), true, "n".into()),
                                ("rlzz".into(), false, "cmp".into())]);
        let s = signatur_lesen(&t).unwrap();
        assert_eq!(s.lib, "user32");
        assert_eq!(s.c_name, "MessageBoxW");
        assert_eq!(s.rueck, 'l');
        assert_eq!(s.params.len(), 4);
        assert!(s.params[2].byref);
        assert_eq!(s.params[1], Param { art: 'w', byref: false, name: "t".into(), rr: String::new() });
        assert_eq!(s.params[3], Param { art: 'r', byref: false, name: "cmp".into(), rr: "lzz".into() });
        let leer = signatur_lesen(&signatur_text("c", "getpid", "getpid", "l", &[])).unwrap();
        assert!(leer.params.is_empty());
    }

    #[test]
    fn mehrere_namen_in_lib() {
        assert_eq!(dateinamen("libgtk-3-0.dll|libgtk-3.so.0", "linux"), ["libgtk-3-0.dll", "libgtk-3.so.0"]);
        assert_eq!(dateinamen("user32 | c", "windows"), ["user32.dll", "ucrtbase.dll", "msvcrt.dll"]);
        assert_eq!(dateinamen("sqlite3|sqlite3", "macos"), dateinamen("sqlite3", "macos"));
    }

    #[test]
    fn dateinamen_je_system() {
        assert_eq!(dateinamen("user32", "windows"), vec!["user32.dll"]);
        assert_eq!(dateinamen("c", "windows"), vec!["ucrtbase.dll", "msvcrt.dll"]);
        assert_eq!(dateinamen("c", "linux")[0], "libc.so.6");
        assert_eq!(dateinamen("m", "linux")[0], "libm.so.6");
        assert_eq!(dateinamen("sqlite3", "linux"), vec!["libsqlite3.so", "sqlite3.so"]);
        assert_eq!(dateinamen("sqlite3", "macos")[0], "libsqlite3.dylib");
        assert_eq!(dateinamen("c", "macos"), vec!["/usr/lib/libSystem.B.dylib"]);
        // Mit Endung oder Pfad woertlich.
        assert_eq!(dateinamen("libfoo.so.6", "linux"), vec!["libfoo.so.6"]);
        assert_eq!(dateinamen("lib/x.dll", "windows"), vec!["lib/x.dll"]);
        assert_eq!(dateinamen("Mein.DLL", "windows"), vec!["Mein.DLL"]);
    }

    #[test]
    fn zahlen_muessen_passen() {
        assert_eq!(zahl_platz('l', &Value::Int(-1)).unwrap() as i64, -1);
        assert!(zahl_platz('l', &Value::Int(1 << 40)).err().unwrap().contains("passt nicht in LONG"));
        assert!(zahl_platz('L', &Value::Int(-1)).is_err());
        assert_eq!(zahl_platz('L', &Value::Int(4294967295)).unwrap(), 4294967295);
        assert!(zahl_platz('B', &Value::Int(256)).is_err());
        assert!(zahl_platz('l', &Value::Float(1.0)).is_err());
        assert!(zahl_platz('o', &Value::Int(1)).is_err());
        assert_eq!(zahl_platz('d', &Value::Int(2)).unwrap(), 2.0f64.to_bits());
        assert_eq!(zahl_platz('f', &Value::Float(1.5)).unwrap(), 1.5f32.to_bits() as u64);
        assert_eq!(zahl_platz('z', &Value::Nil).unwrap(), 0);
    }

    #[test]
    fn plaetze_zurueck() {
        assert!(matches!(platz_wert('b', 0xff), Value::Int(-1)));
        assert!(matches!(platz_wert('B', 0x1ff), Value::Int(255)));
        assert!(matches!(platz_wert('l', 0xffff_ffff), Value::Int(-1)));
        assert!(matches!(platz_wert('L', 0xffff_ffff), Value::Int(4294967295)));
        assert!(matches!(platz_wert('o', 2), Value::Bool(true)));
        assert!(matches!(platz_wert('f', 2.5f32.to_bits() as u64), Value::Float(x) if x == 2.5));
    }

    #[test]
    fn typwoerter_und_hinweise() {
        assert_eq!(typ_zeichen("ZEIGER"), Some('z'));
        assert_eq!(typ_zeichen("ptr"), Some('z'));
        assert_eq!(typ_zeichen("CStr"), Some('t'));
        assert_eq!(typ_zeichen("wstr"), Some('w'));
        assert_eq!(typ_zeichen("double"), None);
        assert!(typ_hinweis("double").contains("FLOAT"));
        assert!(typ_hinweis("string").contains("TEXT"));
        assert!(typ_hinweis("dword").contains("ULONG"));
    }

    #[test]
    fn lage_in_der_signatur() {
        let s = struct_lesen("vec3;12;4;f4*3@0").unwrap();
        assert_eq!((s.name.as_str(), s.groesse, s.ausrichtung, s.glieder.len()), ("vec3", 12, 4, 1));
        assert_eq!(s.glieder[0], (Glied::F32, 4, 3, 0));
        assert!(struct_lesen("x;12;4;f2*1@0").is_none());
        assert!(struct_lesen("x;0;4;i4*1@0").is_none());
        let sig = signatur_lesen(&signatur_text("m", "csqrt", "csqrt", "xkomplex;16;8;f8*2@0",
            &[("xkomplex;16;8;f8*2@0".into(), false, "z".into())])).unwrap();
        assert_eq!((sig.rueck, sig.rueck_rr.as_str(), sig.params[0].art), ('x', "komplex;16;8;f8*2@0", 'x'));
        assert!(signatur_lesen(&signatur_text("m", "f", "f", "xkaputt", &[])).is_err());
    }

    /// Wie jedes System einen Struct zerlegt -- unabhaengig davon, auf
    /// welchem die Pruefung laeuft.
    #[test]
    fn plaene_je_konvention() {
        use Konvention::*;
        let l = |s: &str| struct_lesen(s).unwrap();
        let (p2f, f3, d4, i3, mix, b3) = (l("p;8;4;f4*2@0"), l("f;12;4;f4*3@0"), l("d;32;8;f8*4@0"),
                                          l("i;24;8;i8*3@0"), l("m;16;8;f8*1@0;i8*1@8"), l("b;3;1;i1*3@0"));
        let st = |a, laden| Stelle { art: a, laden };
        // Windows: 8 Bytes im Register, sonst ein Zeiger; Rueckgabe ueber RAX
        // oder die versteckte Adresse als erstes Argument.
        assert_eq!(plan(Win64, &[Form::Struct(&p2f)], Form::Struct(&p2f)).unwrap(),
                   (vec![st(Art::I64, Laden::Ueber(0, 0))], Rueckform::Teile(vec![(Art::I64, 0)])));
        assert_eq!(plan(Win64, &[Form::Struct(&f3)], Form::Struct(&b3)).unwrap(),
                   (vec![st(Art::Zeiger, Laden::Versteckt), st(Art::Zeiger, Laden::Platz(0))], Rueckform::Nichts));
        // System V: zwei Kommazahlen teilen sich ein SSE-Register; ganz und
        // Komma in einer Achtergruppe ist ganz; ueber 16 Bytes auf den Stapel.
        assert_eq!(plan(SysV, &[Form::Struct(&p2f), Form::Struct(&mix)], Form::Struct(&f3)).unwrap(),
                   (vec![st(Art::F64, Laden::Ueber(0, 0)), st(Art::F64, Laden::Ueber(1, 0)), st(Art::I64, Laden::Ueber(1, 8))],
                    Rueckform::Teile(vec![(Art::F64, 0), (Art::F64, 8)])));
        assert_eq!(plan(SysV, &[Form::Struct(&i3)], Form::Struct(&i3)).unwrap(),
                   (vec![st(Art::Zeiger, Laden::Versteckt), st(Art::Zeiger, Laden::Kopie(0, 24))], Rueckform::Nichts));
        // Sind die Register voll, geht der Struct auf den Stapel -- und ein
        // spaeteres Ganzzahl-Argument nimmt trotzdem das letzte freie.
        let zwei = l("z;16;8;i8*2@0");
        let mut f: Vec<Form> = vec![Form::Skalar(Art::I64); 5];
        f.push(Form::Struct(&zwei));
        f.push(Form::Skalar(Art::I64));
        let (p, _) = plan(SysV, &f, Form::Skalar(Art::Nichts)).unwrap();
        assert_eq!(&p[5..], &[st(Art::Zeiger, Laden::Kopie(5, 16)), st(Art::I64, Laden::Platz(6))]);
        // ... waehrend ein gemischter noch passt (ein Ganzzahl-, ein SSE-Register).
        f[5] = Form::Struct(&mix);
        let (p, _) = plan(SysV, &f, Form::Skalar(Art::Nichts)).unwrap();
        assert_eq!(&p[5..7], &[st(Art::F64, Laden::Ueber(5, 0)), st(Art::I64, Laden::Ueber(5, 8))]);
        // AAPCS64: HFA in V-Registern, die versteckte Adresse in X8 zaehlt
        // nicht mit, ein halb passender Struct geht ganz auf den Stapel.
        assert_eq!(plan(Aapcs, &[Form::Struct(&f3)], Form::Struct(&d4)).unwrap(),
                   (vec![st(Art::F32, Laden::Ueber(0, 0)), st(Art::F32, Laden::Ueber(0, 4)), st(Art::F32, Laden::Ueber(0, 8))],
                    Rueckform::Teile(vec![(Art::F64, 0), (Art::F64, 8), (Art::F64, 16), (Art::F64, 24)])));
        assert_eq!(plan(Aapcs, &[Form::Struct(&i3)], Form::Struct(&i3)).unwrap(),
                   (vec![st(Art::Zeiger, Laden::Versteckt), st(Art::Zeiger, Laden::Platz(0))], Rueckform::Nichts));
        let mut f: Vec<Form> = vec![Form::Skalar(Art::I64); 7];
        f.push(Form::Struct(&mix));
        let (p, _) = plan(Aapcs, &f, Form::Skalar(Art::Nichts)).unwrap();
        assert_eq!(&p[7..], &[st(Art::I64, Laden::Null), st(Art::I64, Laden::Ueber(7, 0)), st(Art::I64, Laden::Ueber(7, 8))]);
        let f: Vec<Form> = [vec![Form::Skalar(Art::F64); 7], vec![Form::Struct(&d4)]].concat();
        assert!(plan(Aapcs, &f, Form::Skalar(Art::Nichts)).unwrap_err().contains("Kommazahlen"));
        // Schief gepackt (PACK 1): unter System V immer im Speicher.
        let schief = l("s;12;1;i1*1@0;i8*1@1;i1*3@9");
        assert_eq!(plan(SysV, &[Form::Struct(&schief)], Form::Skalar(Art::Nichts)).unwrap().0,
                   vec![st(Art::Zeiger, Laden::Kopie(0, 16))]);
    }

    #[cfg(feature = "ffi")]
    mod aufrufe {
        use super::super::*;

        extern "C" fn mischen(a: i8, b: u16, c: i32, d: f32, e: f64, f: i64) -> f64 {
            a as f64 + b as f64 + c as f64 + d as f64 + e + f as f64
        }
        extern "C" fn neg8(x: i8) -> i8 { x.wrapping_neg() }
        extern "C" fn verdoppeln(x: *mut i32) { unsafe { *x *= 2 } }
        extern "C" fn laenge(s: *const std::ffi::c_char) -> usize {
            unsafe { std::ffi::CStr::from_ptr(s).to_bytes().len() }
        }
        extern "C" fn hallo() -> *const std::ffi::c_char { b"hallo\0".as_ptr() as *const _ }
        extern "C" fn breit(s: *const WZeichen) -> i32 {
            let mut n = 0; unsafe { while *s.add(n) != 0 { n += 1; } } n as i32
        }
        extern "C" fn fuellen(p: *mut u8, n: i32) { for i in 0..n as usize { unsafe { *p.add(i) = i as u8 + 1; } } }
        extern "C" fn ist_wahr(b: i32) -> i32 { (b != 0) as i32 * 7 }

        fn fehler(r: Result<Value, String>) -> String { match r { Err(e) => e, Ok(_) => panic!("Fehler erwartet") } }

        fn auf(ziel: *const u8, rueck: char, ps: &[(char, bool)]) -> Aufruf {
            let params: Vec<Param> = ps.iter().map(|(a, b)| Param { art: *a, byref: *b, name: "x".into(), rr: String::new() }).collect();
            let arten: Vec<Art> = params.iter().map(|p| art_von(p.art, p.byref)).collect();
            let mut bauer = uebergang::Bauer::neu().unwrap();
            let einstieg = bauer.holen(&arten, art_von(rueck, false)).unwrap();
            std::mem::forget(bauer); // der Code muss leben bleiben
            let structs = vec![None; params.len()];
            Aufruf { sig: Signatur { lib: "test".into(), c_name: "t".into(), name: "t".into(), rueck, rueck_rr: String::new(), params },
                     ziel, einstieg, structs, rueck_struct: None }
        }

        #[test]
        fn zahlen_aller_breiten() {
            let a = auf(mischen as *const u8, 'd', &[('b', false), ('S', false), ('l', false), ('f', false), ('d', false), ('q', false)]);
            let r = rufen_mit(&a, &[Value::Int(-3), Value::Int(60000), Value::Int(-100000), Value::Float(0.5),
                                    Value::Float(0.25), Value::Int(1 << 40)]).unwrap();
            let soll = -3.0 + 60000.0 - 100000.0 + 0.5 + 0.25 + (1u64 << 40) as f64;
            assert!(matches!(r, Value::Float(x) if x == soll));
            let n = auf(neg8 as *const u8, 'b', &[('b', false)]);
            assert!(matches!(rufen_mit(&n, &[Value::Int(5)]).unwrap(), Value::Int(-5)));
            assert!(matches!(rufen_mit(&n, &[Value::Int(-128)]).unwrap(), Value::Int(-128)));
            let w = auf(ist_wahr as *const u8, 'l', &[('o', false)]);
            assert!(matches!(rufen_mit(&w, &[Value::Bool(true)]).unwrap(), Value::Int(7)));
        }

        #[test]
        fn byref_text_und_buffer() {
            let a = auf(verdoppeln as *const u8, 'v', &[('l', true)]);
            let r = rufen_mit(&a, &[Value::Int(21)]).unwrap();
            match r { Value::Tuple(t) => { assert!(matches!(t[0], Value::Nil)); assert!(matches!(t[1], Value::Int(42))); }
                      _ => panic!("Tupel erwartet") }
            let l = auf(laenge as *const u8, 'z', &[('t', false)]);
            assert!(matches!(rufen_mit(&l, &[Value::str_rc("Drachenhauch")]).unwrap(), Value::Int(12)));
            assert!(fehler(rufen_mit(&l, &[Value::str_rc("a\0b")])).contains("Nullzeichen"));
            let h = auf(hallo as *const u8, 't', &[]);
            assert!(matches!(rufen_mit(&h, &[]).unwrap(), Value::Str(s) if s.as_str() == "hallo"));
            let b = auf(breit as *const u8, 'l', &[('w', false)]);
            assert!(matches!(rufen_mit(&b, &[Value::str_rc("Grüße")]).unwrap(), Value::Int(5)));
            let f = auf(fuellen as *const u8, 'v', &[('p', false), ('l', false)]);
            let puf = Rc::new(RefCell::new(vec![0u8; 4]));
            rufen_mit(&f, &[Value::Buffer(puf.clone()), Value::Int(4)]).unwrap();
            assert_eq!(*puf.borrow(), vec![1, 2, 3, 4]);
            assert!(fehler(rufen_mit(&f, &[Value::Buffer(puf), Value::Int(1 << 40)])).contains("Argument 2"));
        }

        /// Der Einstieg eines Rueckrufs, ohne VM: an seiner Stelle ein
        /// Eingang, der die Plaetze mitschreibt und eine Antwort hinterlegt.
        #[test]
        fn rueckruf_einstieg_legt_plaetze_ab() {
            use std::sync::Mutex;
            static GESEHEN: Mutex<Vec<u64>> = Mutex::new(Vec::new());
            extern "C" fn test_eingang(nummer: u64, plaetze: *const u64, rueck: *mut u64) {
                let mut g = GESEHEN.lock().unwrap();
                g.clear();
                g.push(nummer);
                for i in 0..5 { g.push(unsafe { *plaetze.add(i) }); }
                unsafe { *rueck = (-12345i32) as u32 as u64; }
            }
            let mut bauer = uebergang::Bauer::neu().unwrap();
            let z = bauer.rueckruf(&[Art::I8, Art::U16, Art::F32, Art::F64, Art::I64], Art::I32, 77,
                                   test_eingang as usize).unwrap();
            std::mem::forget(bauer);
            let f: extern "C" fn(i8, u16, f32, f64, i64) -> i32 = unsafe { std::mem::transmute(z) };
            assert_eq!(f(-5, 60000, 1.5, -2.25, 1 << 40), -12345);
            let g = GESEHEN.lock().unwrap().clone();
            assert_eq!(g[0], 77);
            assert!(matches!(platz_wert('b', g[1]), Value::Int(-5)));
            assert!(matches!(platz_wert('S', g[2]), Value::Int(60000)));
            assert!(matches!(platz_wert('f', g[3]), Value::Float(x) if x == 1.5));
            assert!(matches!(platz_wert('d', g[4]), Value::Float(x) if x == -2.25));
            assert!(matches!(platz_wert('q', g[5]), Value::Int(x) if x == 1 << 40));
        }

        #[test]
        fn zeiger_lesen_und_puffer_zeigen() {
            fn z(name: &str, a: &[Value]) -> Result<Value, String> { zeiger_befehl(name, a).expect("Befehl der Familie") }
            assert!(zeiger_befehl("buffer_new", &[]).is_none());
            // Ein Text der Rust-Seite als C-Zeiger, schmal und breit.
            let schmal = std::ffi::CString::new("Grüße").unwrap();
            let p = Value::Int(schmal.as_ptr() as i64);
            assert!(matches!(z("text_aus_zeiger$", &[p.clone()]).unwrap(), Value::Str(s) if s.as_str() == "Grüße"));
            let breit = breit_kodieren("Drache 🐉");
            let pb = Value::Int(breit.as_ptr() as i64);
            assert!(matches!(z("text_aus_zeiger$", &[pb, Value::Bool(true)]).unwrap(), Value::Str(s) if s.as_str() == "Drache 🐉"));
            assert!(matches!(z("text_aus_zeiger$", &[Value::Int(0)]).unwrap(), Value::Str(s) if s.is_empty()));
            // Bytes: genau die verlangte Zahl, samt der Null am Ende.
            match z("buffer_aus_zeiger", &[p, Value::Int(8)]).unwrap() {
                Value::Buffer(b) => assert_eq!(*b.borrow(), "Grüße\0".as_bytes()),
                _ => panic!("BUFFER erwartet"),
            }
            assert!(fehler(z("buffer_aus_zeiger", &[Value::Int(0), Value::Int(1)])).contains("NULL"));
            assert!(fehler(z("buffer_aus_zeiger", &[Value::Int(1), Value::Int(-1)])).contains("negativ"));
            // BUFFER_ZEIGER zeigt auf die Bytes selbst: hin und zurueck.
            let puf = Rc::new(RefCell::new(b"abc\0".to_vec()));
            let adr = z("buffer_zeiger", &[Value::Buffer(puf.clone())]).unwrap();
            assert!(matches!(&adr, Value::Int(i) if *i as usize == puf.borrow().as_ptr() as usize));
            assert!(matches!(z("text_aus_zeiger$", &[adr]).unwrap(), Value::Str(s) if s.as_str() == "abc"));
            let leer = Rc::new(RefCell::new(Vec::new()));
            assert!(matches!(z("buffer_zeiger", &[Value::Buffer(leer)]).unwrap(), Value::Int(0)));
            assert!(fehler(z("buffer_zeiger", &[Value::Int(3)])).contains("BUFFER"));
        }

        // ---------------------------------------------- Structs als Wert
        // Rust haelt sich bei `extern "C"` an die Aufrufkonvention des Systems;
        // jede dieser Funktionen ist damit ein Vergleich fuer den Uebergang --
        // die CI faehrt sie unter Windows x64, System V und Apple-ARM.

        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct P2f { x: f32, y: f32 }
        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct P2d { x: f64, y: f64 }
        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct P2i { x: i32, y: i32 }
        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct Mix { d: f64, i: i64 }
        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct Ganz8 { a: i32, f: f32 }
        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct B3 { a: i8, b: i8, c: i8 }
        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct F3 { v: [f32; 3] }
        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct I3 { v: [i64; 3] }
        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct D4 { v: [f64; 4] }

        extern "C" fn p2f_tauschen(p: P2f) -> P2f { P2f { x: p.y, y: p.x } }
        extern "C" fn p2d_summe(a: P2d, b: P2d) -> P2d { P2d { x: a.x + b.x, y: a.y + b.y } }
        extern "C" fn p2i_mal(p: P2i, k: i32) -> P2i { P2i { x: p.x * k, y: p.y * k } }
        extern "C" fn mix_rechnen(a: i32, m: Mix, b: f64) -> Mix { Mix { d: m.d * 2.0 + b, i: m.i + a as i64 } }
        extern "C" fn ganz8(g: Ganz8) -> Ganz8 { Ganz8 { a: g.a + 1, f: g.f * 3.0 } }
        extern "C" fn b3_drehen(b: B3) -> B3 { B3 { a: b.c, b: b.a, c: b.b } }
        extern "C" fn f3_skalieren(f: F3, k: f32) -> F3 { F3 { v: [f.v[0] * k, f.v[1] * k, f.v[2] * k] } }
        extern "C" fn i3_summe(x: I3, y: I3) -> I3 { I3 { v: [x.v[0] + y.v[0], x.v[1] + y.v[1], x.v[2] + y.v[2]] } }
        extern "C" fn d4_mal(x: D4, k: f64) -> D4 { D4 { v: x.v.map(|e| e * k) } }
        extern "C" fn d4_spur(x: D4) -> f64 { x.v.iter().sum() }
        #[allow(clippy::too_many_arguments)]
        extern "C" fn viele_ganz(a: i64, b: i64, c: i64, d: i64, e: i64, f: i64, p: P2i, m: Mix, g: i64) -> i64 {
            a + b + c + d + e + f + p.x as i64 * 100 + p.y as i64 * 1000 + m.i * 10000 + m.d as i64 * 100000 + g * 1000000
        }
        #[repr(C)] #[derive(Clone, Copy, Debug, PartialEq)] struct Z2 { a: i64, b: i64 }
        #[allow(clippy::too_many_arguments)]
        extern "C" fn fuenf_und_zwei(a: i64, b: i64, c: i64, d: i64, e: i64, z: Z2, g: i64) -> i64 {
            a + b + c + d + e + z.a * 100 + z.b * 1000 + g * 10000
        }
        #[allow(clippy::too_many_arguments)]
        extern "C" fn viele_komma(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: f64, q: Mix, z: i64) -> f64 {
            a + b + c + d + e + f + g + h + q.d * 10.0 + q.i as f64 * 100.0 + z as f64 * 1000.0
        }

        /// Die Lage in der Form, die der Compiler schreibt.
        fn lage(name: &str, groesse: usize, ausr: usize, glieder: &str) -> String {
            format!("x{};{};{};{}", name, groesse, ausr, glieder)
        }
        fn als_puffer<T>(x: &T) -> Value {
            let b = unsafe { std::slice::from_raw_parts(x as *const T as *const u8, std::mem::size_of::<T>()) };
            crate::builtins::neuer_buffer(b.to_vec())
        }
        fn aus_puffer<T: Copy>(v: Value) -> T {
            match v {
                Value::Buffer(b) => {
                    let b = b.borrow();
                    assert_eq!(b.len(), std::mem::size_of::<T>());
                    unsafe { std::ptr::read_unaligned(b.as_ptr() as *const T) }
                }
                v => panic!("BUFFER erwartet, erhalten {}", v.type_name()),
            }
        }
        fn rufe(ziel: *const u8, rueck: &str, params: &[String], args: &[Value]) -> Result<Value, String> {
            let ps: Vec<(String, bool, String)> = params.iter().enumerate()
                .map(|(i, p)| (p.clone(), false, format!("a{}", i))).collect();
            let sig = signatur_lesen(&signatur_text("test", "t", "t", rueck, &ps))?;
            let a = aufruf_bauen(sig, ziel)?;
            rufen_mit(&a, args)
        }

        #[test]
        fn structs_als_wert() {
            let p2f = lage("p2f", 8, 4, "f4*2@0");
            let p2d = lage("p2d", 16, 8, "f8*2@0");
            let p2i = lage("p2i", 8, 4, "i4*2@0");
            let mix = lage("mix", 16, 8, "f8*1@0;i8*1@8");
            let g8 = lage("g8", 8, 4, "i4*1@0;f4*1@4");
            let b3 = lage("b3", 3, 1, "i1*3@0");
            let f3 = lage("f3", 12, 4, "f4*3@0");
            let i3 = lage("i3", 24, 8, "i8*3@0");
            let d4 = lage("d4", 32, 8, "f8*4@0");

            let r: P2f = aus_puffer(rufe(p2f_tauschen as *const u8, &p2f, std::slice::from_ref(&p2f),
                                         &[als_puffer(&P2f { x: 1.5, y: -2.25 })]).unwrap());
            assert_eq!(r, P2f { x: -2.25, y: 1.5 });
            let r: P2d = aus_puffer(rufe(p2d_summe as *const u8, &p2d, &[p2d.clone(), p2d.clone()],
                                         &[als_puffer(&P2d { x: 1.0, y: 2.0 }), als_puffer(&P2d { x: 0.5, y: 10.0 })]).unwrap());
            assert_eq!(r, P2d { x: 1.5, y: 12.0 });
            let r: P2i = aus_puffer(rufe(p2i_mal as *const u8, &p2i, &[p2i.clone(), "l".into()],
                                         &[als_puffer(&P2i { x: 3, y: -4 }), Value::Int(5)]).unwrap());
            assert_eq!(r, P2i { x: 15, y: -20 });
            let r: Mix = aus_puffer(rufe(mix_rechnen as *const u8, &mix, &["l".into(), mix.clone(), "d".into()],
                                         &[Value::Int(7), als_puffer(&Mix { d: 1.25, i: 1 << 40 }), Value::Float(0.5)]).unwrap());
            assert_eq!(r, Mix { d: 3.0, i: (1 << 40) + 7 });
            let r: Ganz8 = aus_puffer(rufe(ganz8 as *const u8, &g8, std::slice::from_ref(&g8),
                                           &[als_puffer(&Ganz8 { a: 41, f: 1.5 })]).unwrap());
            assert_eq!(r, Ganz8 { a: 42, f: 4.5 });
            let r: B3 = aus_puffer(rufe(b3_drehen as *const u8, &b3, std::slice::from_ref(&b3),
                                        &[als_puffer(&B3 { a: 1, b: -2, c: 3 })]).unwrap());
            assert_eq!(r, B3 { a: 3, b: 1, c: -2 });
            let r: F3 = aus_puffer(rufe(f3_skalieren as *const u8, &f3, &[f3.clone(), "f".into()],
                                        &[als_puffer(&F3 { v: [1.0, 2.0, -3.0] }), Value::Float(2.5)]).unwrap());
            assert_eq!(r, F3 { v: [2.5, 5.0, -7.5] });
            let r: I3 = aus_puffer(rufe(i3_summe as *const u8, &i3, &[i3.clone(), i3.clone()],
                                        &[als_puffer(&I3 { v: [1, 2, 3] }), als_puffer(&I3 { v: [10, 20, -30] })]).unwrap());
            assert_eq!(r, I3 { v: [11, 22, -27] });
            let r: D4 = aus_puffer(rufe(d4_mal as *const u8, &d4, &[d4.clone(), "d".into()],
                                        &[als_puffer(&D4 { v: [1.0, 2.0, 3.0, 4.0] }), Value::Float(-0.5)]).unwrap());
            assert_eq!(r, D4 { v: [-0.5, -1.0, -1.5, -2.0] });
            let r = rufe(d4_spur as *const u8, "d", std::slice::from_ref(&d4),
                         &[als_puffer(&D4 { v: [1.0, 2.0, 3.0, 4.5] })]).unwrap();
            assert!(matches!(r, Value::Float(f) if f == 10.5));
            // Die Kopie gehoert der Funktion: der Puffer des Programms bleibt.
            let orig = als_puffer(&P2f { x: 1.0, y: 2.0 });
            rufe(p2f_tauschen as *const u8, &p2f, std::slice::from_ref(&p2f), std::slice::from_ref(&orig)).unwrap();
            assert_eq!(aus_puffer::<P2f>(orig), P2f { x: 1.0, y: 2.0 });
        }

        #[test]
        fn structs_hinter_vollen_registern() {
            // Sechs Ganzzahlen davor: unter System V sind die Register voll,
            // beide Structs gehen auf den Stapel; unter AAPCS64 passt der
            // erste noch, der zweite nicht mehr ganz.
            let p2i = lage("p2i", 8, 4, "i4*2@0");
            let mix = lage("mix", 16, 8, "f8*1@0;i8*1@8");
            let mut ps: Vec<String> = vec!["q".into(); 6];
            ps.push(p2i.clone());
            ps.push(mix.clone());
            ps.push("q".into());
            let mut args: Vec<Value> = (1..=6).map(Value::Int).collect();
            args.push(als_puffer(&P2i { x: 2, y: 3 }));
            args.push(als_puffer(&Mix { d: 4.0, i: 5 }));
            args.push(Value::Int(6));
            let r = rufe(viele_ganz as *const u8, "q", &ps, &args).unwrap();
            assert!(matches!(r, Value::Int(i) if i == 21 + 200 + 3000 + 50000 + 400000 + 6000000));
            // Fuenf davor: unter System V geht der Struct auf den Stapel, die
            // Ganzzahl dahinter nimmt das letzte freie Register.
            let mut ps: Vec<String> = vec!["q".into(); 5];
            ps.push(lage("z2", 16, 8, "i8*2@0"));
            ps.push("q".into());
            let mut args: Vec<Value> = (1..=5).map(Value::Int).collect();
            args.push(als_puffer(&Z2 { a: 2, b: 3 }));
            args.push(Value::Int(4));
            let r = rufe(fuenf_und_zwei as *const u8, "q", &ps, &args).unwrap();
            assert!(matches!(r, Value::Int(i) if i == 15 + 200 + 3000 + 40000));
            // Acht Kommazahlen davor.
            let mut ps: Vec<String> = vec!["d".into(); 8];
            ps.push(mix.clone());
            ps.push("q".into());
            let mut args: Vec<Value> = (1..=8).map(|i| Value::Float(i as f64)).collect();
            args.push(als_puffer(&Mix { d: 2.0, i: 3 }));
            args.push(Value::Int(4));
            let r = rufe(viele_komma as *const u8, "d", &ps, &args).unwrap();
            assert!(matches!(r, Value::Float(f) if f == 36.0 + 20.0 + 300.0 + 4000.0));
        }

        #[test]
        fn falscher_struct_wert() {
            let p2f = lage("p2f", 8, 4, "f4*2@0");
            let kurz = crate::builtins::neuer_buffer(vec![0u8; 4]);
            assert!(fehler(rufe(p2f_tauschen as *const u8, &p2f, std::slice::from_ref(&p2f), &[kurz])).contains("braucht 8"));
            assert!(fehler(rufe(p2f_tauschen as *const u8, &p2f, std::slice::from_ref(&p2f), &[Value::Int(3)])).contains("P2F"));
            assert!(fehler(rufe(p2f_tauschen as *const u8, &p2f, std::slice::from_ref(&p2f), &[Value::Nil])).contains("NIL"));
        }
    }
}
