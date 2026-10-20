//! Zeichenindex fuer grosse Texte.
//!
//! MID$, INSTR, LEN & Co. zaehlen in ZEICHEN, Texte liegen als UTF-8 vor:
//! die Byte-Stelle des n-ten Zeichens kostet ohne Hilfe O(n). Bei kleinen
//! Texten ist das egal (und `is_ascii` macht es wortweise schnell), bei einem
//! eingelesenen Text von Megabytes wird jede Schleife `MID$(t, i, 1)` bzw.
//! `INSTR(t, s, k)` damit quadratisch -- gemessen (Funkenflug, 2026-10-20):
//! 30 Mikrosekunden je Aufruf bei Stelle 4 000 000.
//!
//! Darum bekommt ein grosser Text, der zum zweiten Mal hintereinander gefragt
//! wird, einen Index: ist er reines ASCII, nur diese Feststellung; sonst die
//! Byte-Stelle jedes `SCHRITT`-ten Zeichens. Danach kostet eine Stelle O(1)
//! bzw. hoechstens `SCHRITT` Zeichen.
//!
//! Der Text wird ueber ein `Weak` erkannt: das haelt den Speicher des Texts
//! nicht fest, und solange es lebt, kann keine andere Zeichenkette an dieselbe
//! Stelle ruecken. Gueltig ist ein Eintrag, solange der Text lebt und Anfang
//! und Laenge stimmen. Ein `Weak` verhindert `Rc::get_mut` -- das Anhaengen an
//! Ort und Stelle (`s = s + x`) ruft deshalb erst `loslassen`.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Kleinere Texte zaehlen weiter direkt.
const AB_BYTES: usize = 16 * 1024;
/// Jede wievielte Zeichenstelle gemerkt wird (Nicht-ASCII).
const SCHRITT: usize = 64;
/// So viele Texte zugleich -- wer abwechselnd in zweien sucht, soll nicht
/// jedes Mal neu bauen.
const PLAETZE: usize = 4;

struct Eintrag {
    text: Weak<String>,
    anfang: *const u8,
    bytes: usize,
    zeichen: usize,
    /// Byte-Stelle der Zeichen 0, SCHRITT, 2*SCHRITT, ...; leer = ASCII.
    marken: Vec<usize>,
}

impl Eintrag {
    fn passt(&self, s: &str) -> bool {
        self.anfang == s.as_ptr() && self.bytes == s.len() && self.text.strong_count() > 0
    }
}

/// Zuletzt gefragt, aber noch ohne Index (erst die zweite Frage baut).
struct Anwaerter {
    text: Weak<String>,
    anfang: *const u8,
    bytes: usize,
}

#[derive(Default)]
struct Speicher {
    eintraege: Vec<Eintrag>,          // vorne = zuletzt benutzt
    anwaerter: Option<Anwaerter>,
}

thread_local! {
    static SPEICHER: RefCell<Speicher> = RefCell::new(Speicher::default());
}

/// Ein Text wird gleich nach Zeichen gefragt: beim zweiten Mal hintereinander
/// bekommt er einen Index. Billig, wenn er schon einen hat oder klein ist.
pub(crate) fn anmelden(rc: &Rc<String>) {
    if rc.len() < AB_BYTES { return; }
    SPEICHER.with(|sp| {
        let mut sp = sp.borrow_mut();
        if let Some(i) = sp.eintraege.iter().position(|e| e.passt(rc)) {
            if i > 0 { let e = sp.eintraege.remove(i); sp.eintraege.insert(0, e); }
            return;
        }
        let zweites_mal = sp.anwaerter.as_ref().is_some_and(|a|
            a.anfang == rc.as_ptr() && a.bytes == rc.len() && a.text.strong_count() > 0);
        if !zweites_mal {
            sp.anwaerter = Some(Anwaerter { text: Rc::downgrade(rc), anfang: rc.as_ptr(), bytes: rc.len() });
            return;
        }
        sp.anwaerter = None;
        let e = bauen(rc);
        sp.eintraege.insert(0, e);
        sp.eintraege.truncate(PLAETZE);
        // tote Eintraege gleich mit weg (ihr `Weak` haelt sonst den Rc-Kopf)
        sp.eintraege.retain(|e| e.text.strong_count() > 0);
    });
}

fn bauen(rc: &Rc<String>) -> Eintrag {
    let s = rc.as_str();
    let mut marken = Vec::new();
    let zeichen = if s.is_ascii() {
        s.len()
    } else {
        marken.reserve(s.len() / SCHRITT + 1);
        let mut n = 0usize;
        for (b, &byte) in s.as_bytes().iter().enumerate() {
            // ein Zeichen beginnt an jedem Byte, das kein Folgebyte 10xxxxxx ist
            if byte & 0xC0 != 0x80 {
                if n % SCHRITT == 0 { marken.push(b); }
                n += 1;
            }
        }
        n
    };
    Eintrag { text: Rc::downgrade(rc), anfang: s.as_ptr(), bytes: s.len(), zeichen, marken }
}

/// Vor dem Aendern an Ort und Stelle: Index und Anwaerter dieses Texts
/// vergessen (sonst scheitert `Rc::get_mut` am `Weak`).
#[inline(always)]
pub(crate) fn loslassen(rc: &Rc<String>) {
    // der Normalfall (kein Weak) ohne Aufruf -- das liegt im heissen Pfad
    // jedes Anhaengens
    if Rc::weak_count(rc) != 0 { loslassen_langsam(rc); }
}

#[cold]
#[inline(never)]
fn loslassen_langsam(rc: &Rc<String>) {
    let p = Rc::as_ptr(rc);
    SPEICHER.with(|sp| {
        let mut sp = sp.borrow_mut();
        sp.eintraege.retain(|e| e.text.as_ptr() != p);
        if sp.anwaerter.as_ref().is_some_and(|a| a.text.as_ptr() == p) { sp.anwaerter = None; }
    });
}

fn mit<T>(s: &str, f: impl FnOnce(&Eintrag) -> T) -> Option<T> {
    if s.len() < AB_BYTES { return None; }
    SPEICHER.with(|sp| {
        let sp = sp.borrow();
        sp.eintraege.iter().find(|e| e.passt(s)).map(f)
    })
}

/// Zeichenzahl, wenn der Text einen Index hat.
pub(crate) fn zahl(s: &str) -> Option<usize> {
    mit(s, |e| e.zeichen)
}

/// Byte-Stelle des n-ten Zeichens (genau am Ende die Laenge, dahinter None);
/// None auch, wenn der Text keinen Index hat -- das sagt das aeussere Option.
pub(crate) fn stelle(s: &str, n: usize) -> Option<Option<usize>> {
    mit(s, |e| {
        if n > e.zeichen { return None; }
        if n == e.zeichen { return Some(s.len()); }
        if e.marken.is_empty() { return Some(n); }
        let b = e.marken[n / SCHRITT];
        let rest = n % SCHRITT;
        Some(if rest == 0 { b } else {
            s[b..].char_indices().nth(rest).map_or(s.len(), |(o, _)| b + o)
        })
    })
}

/// Zeichen vor der Byte-Stelle `byte` (muss auf einer Zeichengrenze liegen).
pub(crate) fn zeichen_vor(s: &str, byte: usize) -> Option<usize> {
    mit(s, |e| {
        if e.marken.is_empty() { return byte; }
        // letzte Marke <= byte
        let m = e.marken.partition_point(|&b| b <= byte) - 1;
        m * SCHRITT + s[e.marken[m]..byte].chars().count()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gross(muster: &str) -> Rc<String> {
        Rc::new(muster.repeat(AB_BYTES / muster.len() + 50))
    }

    fn naiv_stelle(s: &str, n: usize) -> Option<usize> {
        let z = s.chars().count();
        if n > z { None } else if n == z { Some(s.len()) } else { s.char_indices().nth(n).map(|(b, _)| b) }
    }

    #[test]
    fn erst_die_zweite_frage_baut() {
        let t = gross("abc");
        anmelden(&t);
        assert_eq!(zahl(&t), None);
        anmelden(&t);
        assert_eq!(zahl(&t), Some(t.len()));
    }

    #[test]
    fn utf8_stellen_stimmen() {
        let t = gross("aä€🐉b");
        anmelden(&t);
        anmelden(&t);
        let z = t.chars().count();
        assert_eq!(zahl(&t), Some(z));
        for n in (0..z + 3).step_by(7).chain([0, 1, 63, 64, 65, z - 1, z, z + 1]) {
            assert_eq!(stelle(&t, n), Some(naiv_stelle(&t, n)), "n = {}", n);
        }
        for (n, (b, _)) in t.char_indices().enumerate().step_by(5) {
            assert_eq!(zeichen_vor(&t, b), Some(n));
        }
        assert_eq!(zeichen_vor(&t, t.len()), Some(z));
    }

    #[test]
    fn kleine_texte_ohne_index() {
        let t = Rc::new("kurz".to_string());
        anmelden(&t);
        anmelden(&t);
        assert_eq!(zahl(&t), None);
        assert_eq!(Rc::weak_count(&t), 0);
    }

    #[test]
    fn toter_text_gilt_nicht_mehr() {
        let t = gross("xyz");
        anmelden(&t);
        anmelden(&t);
        let laenge = t.len();
        drop(t);
        // ein neuer Text gleicher Laenge -- egal wo er liegt, der alte Index
        // darf nicht greifen (der Text ist tot)
        let u = Rc::new("ä".repeat(laenge / 2));
        assert_eq!(zahl(&u), None);
    }

    #[test]
    fn loslassen_gibt_get_mut_frei() {
        let mut t = gross("abc");
        anmelden(&t);
        anmelden(&t);
        assert!(Rc::get_mut(&mut t).is_none());
        loslassen(&t);
        let s = Rc::get_mut(&mut t).expect("nach loslassen exklusiv");
        s.push('d');
        // geaenderte Laenge: kein (alter) Index
        assert_eq!(zahl(&t), None);
    }

    #[test]
    fn anwaerter_haelt_auch_fest() {
        let mut t = gross("abc");
        anmelden(&t);
        assert!(Rc::get_mut(&mut t).is_none());
        loslassen(&t);
        assert!(Rc::get_mut(&mut t).is_some());
    }
}
