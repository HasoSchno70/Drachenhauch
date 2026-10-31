//! Rechtschreibung (SPELL_*, docs/entwurf-gui-wie-gtk.md Paket 3).
//!
//! Zwei Quellen, ein Vertrag:
//! * unter Windows die Rechtschreibpruefung des Systems (`ISpellChecker`) --
//!   mit den Sprachen, die dort installiert sind, ohne eigene Dateien;
//! * sonst (und auf jedem System auf Wunsch) Hunspell-Woerterbuecher
//!   (`.dic` + `.aff`) ueber `spellbook`, unter Linux aus den ueblichen
//!   Ordnern.
//!
//! Ein Woerterbuch ist ein Handle (INTEGER), wie ein Bild. Die gui fragt
//! dieselben Funktionen (`fehler`, `vorschlaege`), wenn ein Feld per
//! GUI_SPELL prueft. Alles laeuft im Hauptfaden; die Ablage ist darum ein
//! `thread_local`.
use std::cell::RefCell;

enum Quelle {
    Hunspell(Box<spellbook::Dictionary>),
    #[cfg(windows)]
    Windows(windows_pruefer::Pruefer),
}

struct Woerterbuch {
    quelle: Quelle,
    sprache: String,
    /// Fuer diese Sitzung ignorierte Woerter (Hunspell; Windows merkt sie
    /// sich selbst).
    ignoriert: std::collections::HashSet<String>,
}

thread_local! {
    // Am Programmende NICHT abgeraeumt: die Pruefung von Windows haengt,
    // wenn ihr Objekt beim Abbau des Fadens freigegeben wird (gemessen --
    // das Programm endete nie). SPELL_FREE gibt waehrend des Laufs frei.
    static BUECHER: RefCell<std::mem::ManuallyDrop<Vec<Option<Woerterbuch>>>> =
        const { RefCell::new(std::mem::ManuallyDrop::new(Vec::new())) };
}

fn ablegen(w: Woerterbuch) -> i64 {
    BUECHER.with(|b| { let mut b = b.borrow_mut(); b.push(Some(w)); b.len() as i64 - 1 })
}

fn mit<T>(h: i64, f: &str, tun: impl FnOnce(&mut Woerterbuch) -> Result<T, String>) -> Result<T, String> {
    BUECHER.with(|b| {
        let mut b = b.borrow_mut();
        match b.get_mut(h.max(-1) as usize).and_then(|x| x.as_mut()) {
            Some(w) if h >= 0 => tun(w),
            _ => Err(format!("{}: {} ist kein Woerterbuch (SPELL_LOAD oder SPELL_SYSTEM)", f, h)),
        }
    })
}

/// Die Woerter eines Textes: (erstes Zeichen, Zeichenzahl, Wort). Ein Wort
/// sind Buchstaben, dazu ein Apostroph zwischen zwei Buchstaben ("don't");
/// einzelne Buchstaben zaehlen nicht.
pub fn woerter(text: &str) -> Vec<(usize, usize, String)> {
    let z: Vec<char> = text.chars().collect();
    let mut aus = Vec::new();
    let mut i = 0;
    while i < z.len() {
        if !z[i].is_alphabetic() { i += 1; continue; }
        let a = i;
        while i < z.len() && (z[i].is_alphabetic()
            || (matches!(z[i], '\'' | '\u{2019}') && i + 1 < z.len() && z[i + 1].is_alphabetic() && i > a)) {
            i += 1;
        }
        if i - a >= 2 { aus.push((a, i - a, z[a..i].iter().collect())); }
    }
    aus
}

/// Ordner, in denen Hunspell-Woerterbuecher liegen. `DHRT_WOERTERBUECHER`
/// geht vor (fuer Pruefungen und eigene Sammlungen) -- dann nimmt auch
/// Windows die Dateien statt der Systempruefung.
fn ordner() -> Vec<std::path::PathBuf> {
    if let Ok(o) = std::env::var("DHRT_WOERTERBUECHER") {
        return std::env::split_paths(&o).collect();
    }
    let mut v: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(h) = std::env::var("HOME") {
        v.push(std::path::Path::new(&h).join(".local/share/hunspell"));
        v.push(std::path::Path::new(&h).join("Library/Spelling"));
    }
    for o in ["/usr/share/hunspell", "/usr/share/myspell", "/usr/share/myspell/dicts", "/usr/local/share/hunspell",
              "/opt/homebrew/share/hunspell", "/Library/Spelling"] {
        v.push(o.into());
    }
    v
}

/// Hunspell-Woerterbuecher in den Ordnern: (Sprache wie "de-DE", .dic-Pfad).
fn hunspell_dateien() -> Vec<(String, std::path::PathBuf)> {
    let mut aus = Vec::new();
    for o in ordner() {
        let Ok(rd) = std::fs::read_dir(&o) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "dic").unwrap_or(false) && p.with_extension("aff").exists() {
                let stamm = p.file_stem().map(|s| s.to_string_lossy().replace('_', "-")).unwrap_or_default();
                if !aus.iter().any(|(s, _): &(String, std::path::PathBuf)| s.eq_ignore_ascii_case(&stamm)) {
                    aus.push((stamm, p));
                }
            }
        }
    }
    aus.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    aus
}

fn nimmt_dateien() -> bool {
    !cfg!(windows) || std::env::var("DHRT_WOERTERBUECHER").is_ok()
}

/// SPELL_LANGUAGES(): die Sprachen, fuer die SPELL_SYSTEM ein Woerterbuch hat.
pub fn sprachen() -> Vec<String> {
    if nimmt_dateien() { return hunspell_dateien().into_iter().map(|d| d.0).collect(); }
    #[cfg(windows)]
    { windows_pruefer::sprachen() }
    #[cfg(not(windows))]
    { Vec::new() }
}

/// SPELL_LOAD(dic$ [, aff$]): ein Hunspell-Woerterbuch aus Dateien.
pub fn laden(dic: &str, aff: &str) -> Result<i64, String> {
    let f = "SPELL_LOAD";
    let aff = if aff.is_empty() {
        std::path::Path::new(dic).with_extension("aff").to_string_lossy().into_owned()
    } else { aff.to_string() };
    let dic_t = std::fs::read_to_string(dic).map_err(|e| format!("{}: '{}' laesst sich nicht lesen: {}", f, dic, e))?;
    let aff_t = std::fs::read_to_string(&aff).map_err(|e| format!("{}: '{}' laesst sich nicht lesen: {}", f, aff, e))?;
    let d = spellbook::Dictionary::new(&aff_t, &dic_t).map_err(|e| format!("{}: '{}' ist kein Hunspell-Woerterbuch: {}", f, dic, e))?;
    let sprache = std::path::Path::new(dic).file_stem().map(|s| s.to_string_lossy().replace('_', "-")).unwrap_or_default();
    Ok(ablegen(Woerterbuch { quelle: Quelle::Hunspell(Box::new(d)), sprache, ignoriert: Default::default() }))
}

/// SPELL_SYSTEM(sprache$): das Woerterbuch des Systems fuer diese Sprache
/// ("de-DE", "en-US"; "de" nimmt das erste deutsche).
pub fn system(sprache: &str) -> Result<i64, String> {
    let f = "SPELL_SYSTEM";
    let s = sprache.trim().replace('_', "-");
    if nimmt_dateien() {
        let alle = hunspell_dateien();
        let treffer = alle.iter().find(|(n, _)| n.eq_ignore_ascii_case(&s))
            .or_else(|| alle.iter().find(|(n, _)| n.to_lowercase().starts_with(&format!("{}-", s.to_lowercase()))));
        return match treffer {
            Some((_, p)) => laden(&p.to_string_lossy(), ""),
            None => Err(format!("{}: kein Woerterbuch fuer '{}' -- {}", f, sprache, verfuegbar_hinweis(&alle.iter().map(|a| a.0.clone()).collect::<Vec<_>>()))),
        };
    }
    #[cfg(windows)]
    {
        let alle = windows_pruefer::sprachen();
        let tag = alle.iter().find(|n| n.eq_ignore_ascii_case(&s))
            .or_else(|| alle.iter().find(|n| n.to_lowercase().starts_with(&format!("{}-", s.to_lowercase()))))
            .cloned()
            .ok_or_else(|| format!("{}: keine Rechtschreibung fuer '{}' -- {}", f, sprache, verfuegbar_hinweis(&alle)))?;
        let p = windows_pruefer::Pruefer::neu(&tag).map_err(|e| format!("{}: {}", f, e))?;
        Ok(ablegen(Woerterbuch { quelle: Quelle::Windows(p), sprache: tag, ignoriert: Default::default() }))
    }
    #[cfg(not(windows))]
    { unreachable!() }
}

fn verfuegbar_hinweis(alle: &[String]) -> String {
    if alle.is_empty() {
        "es gibt keine. Unter Linux liefert das Paket hunspell-de-de (bzw. -en-us) die Woerterbuecher; sonst SPELL_LOAD mit einer .dic-Datei".into()
    } else {
        format!("verfuegbar sind: {}", alle.join(", "))
    }
}

pub fn freigeben(h: i64) {
    BUECHER.with(|b| { if let Some(x) = b.borrow_mut().get_mut(h.max(0) as usize) { if h >= 0 { *x = None; } } });
}

pub fn sprache(h: i64) -> Result<String, String> { mit(h, "SPELL_LANGUAGE$", |w| Ok(w.sprache.clone())) }

/// SPELL_CHECK(h, wort$): ist das Wort richtig geschrieben?
pub fn pruefen(h: i64, wort: &str) -> Result<bool, String> {
    mit(h, "SPELL_CHECK", |w| {
        if w.ignoriert.contains(wort) { return Ok(true); }
        match &w.quelle {
            Quelle::Hunspell(d) => Ok(d.check(wort)),
            #[cfg(windows)]
            Quelle::Windows(p) => p.fehler(wort).map(|f| f.is_empty()),
        }
    })
}

/// SPELL_SUGGEST: Vorschlaege fuer ein Wort, hoechstens `max`.
pub fn vorschlaege(h: i64, wort: &str, max: usize) -> Result<Vec<String>, String> {
    mit(h, "SPELL_SUGGEST", |w| {
        let mut v = match &w.quelle {
            Quelle::Hunspell(d) => { let mut o = Vec::new(); d.suggest(wort, &mut o); o }
            #[cfg(windows)]
            Quelle::Windows(p) => p.vorschlaege(wort)?,
        };
        v.retain(|s| s != wort);
        v.truncate(max);
        Ok(v)
    })
}

/// SPELL_IGNORE: ein Wort gilt fuer den Rest der Sitzung als richtig.
pub fn ignorieren(h: i64, wort: &str) -> Result<(), String> {
    mit(h, "SPELL_IGNORE", |w| {
        w.ignoriert.insert(wort.to_string());
        #[cfg(windows)]
        if let Quelle::Windows(p) = &w.quelle { p.ignorieren(wort)?; }
        Ok(())
    })
}

/// Die falsch geschriebenen Woerter eines Textes: (erstes Zeichen,
/// Zeichenzahl).
pub fn fehler(h: i64, text: &str) -> Result<Vec<(usize, usize)>, String> {
    mit(h, "SPELL_ERRORS", |w| {
        match &w.quelle {
            Quelle::Hunspell(d) => Ok(woerter(text).into_iter()
                .filter(|(_, _, t)| !w.ignoriert.contains(t) && !d.check(t))
                .map(|(a, n, _)| (a, n)).collect()),
            #[cfg(windows)]
            Quelle::Windows(p) => {
                // Windows zaehlt in UTF-16-Einheiten -- umrechnen in Zeichen.
                let fehl = p.fehler(text)?;
                let mut zu_zeichen = Vec::with_capacity(text.len() + 1);
                for (i, c) in text.chars().enumerate() { for _ in 0..c.len_utf16() { zu_zeichen.push(i); } }
                let n = text.chars().count();
                zu_zeichen.push(n);
                Ok(fehl.into_iter().filter_map(|(a16, l16)| {
                    let a = *zu_zeichen.get(a16 as usize)?;
                    let e = *zu_zeichen.get((a16 + l16) as usize).unwrap_or(&n);
                    let wort: String = text.chars().skip(a).take(e.saturating_sub(a)).collect();
                    if w.ignoriert.contains(&wort) || e <= a { None } else { Some((a, e - a)) }
                }).collect())
            }
        }
    })
}

#[cfg(windows)]
mod windows_pruefer {
    use windows::core::{HSTRING, PWSTR};
    use windows::Win32::Globalization::{ISpellChecker, ISpellCheckerFactory, SpellCheckerFactory};
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, IEnumString, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};

    pub struct Pruefer { p: ISpellChecker }

    fn fabrik() -> Result<ISpellCheckerFactory, String> {
        unsafe {
            // Schon angemeldet (raylib/GLFW, Sprachausgabe) ist kein Fehler.
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| format!("die Rechtschreibpruefung von Windows ist nicht da ({})", e))
        }
    }
    fn texte(e: IEnumString) -> Vec<String> {
        let mut aus = Vec::new();
        unsafe {
            loop {
                let mut s = [PWSTR::null(); 1];
                let mut n = 0u32;
                let hr = e.Next(&mut s, Some(&mut n));
                if hr.is_err() || n == 0 { break; }
                if let Ok(t) = s[0].to_string() { aus.push(t); }
                CoTaskMemFree(Some(s[0].0 as *const _));
            }
        }
        aus
    }
    pub fn sprachen() -> Vec<String> {
        let Ok(f) = fabrik() else { return Vec::new() };
        let mut v = unsafe { f.SupportedLanguages() }.map(texte).unwrap_or_default();
        v.sort_by_key(|a| a.to_lowercase());
        v
    }
    impl Pruefer {
        pub fn neu(tag: &str) -> Result<Pruefer, String> {
            let f = fabrik()?;
            let p = unsafe { f.CreateSpellChecker(&HSTRING::from(tag)) }.map_err(|e| format!("'{}': {}", tag, e))?;
            Ok(Pruefer { p })
        }
        /// (Anfang, Laenge) in UTF-16-Einheiten.
        pub fn fehler(&self, text: &str) -> Result<Vec<(u32, u32)>, String> {
            let mut aus = Vec::new();
            unsafe {
                let e = self.p.Check(&HSTRING::from(text)).map_err(|e| e.to_string())?;
                loop {
                    let mut f = None;
                    let hr = e.Next(&mut f);
                    let Some(f) = f.filter(|_| hr.0 == 0) else { break };
                    if let (Ok(a), Ok(l)) = (f.StartIndex(), f.Length()) { aus.push((a, l)); }
                }
            }
            Ok(aus)
        }
        pub fn vorschlaege(&self, wort: &str) -> Result<Vec<String>, String> {
            unsafe { self.p.Suggest(&HSTRING::from(wort)) }.map(texte).map_err(|e| e.to_string())
        }
        pub fn ignorieren(&self, wort: &str) -> Result<(), String> {
            unsafe { self.p.Ignore(&HSTRING::from(wort)) }.map_err(|e| e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AFF: &str = "SET UTF-8\nTRY esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ\nSFX S Y 1\nSFX S 0 e .\n";
    const DIC: &str = "4\nHaus/S\nBaum\nGarten\nApfel\n";

    #[test]
    fn woerter_zerlegen() {
        let w: Vec<String> = woerter("Der Hund's Ball, 3 Äpfel - x y E-Mail").into_iter().map(|x| x.2).collect();
        assert_eq!(w, vec!["Der", "Hund's", "Ball", "Äpfel", "Mail"]);
        let w = woerter("ein Grüße");
        assert_eq!((w[1].0, w[1].1), (4, 5));
    }

    #[test]
    fn hunspell_pruefen_vorschlagen_ignorieren() {
        let d = spellbook::Dictionary::new(AFF, DIC).unwrap();
        let h = ablegen(Woerterbuch { quelle: Quelle::Hunspell(Box::new(d)), sprache: "xx".into(), ignoriert: Default::default() });
        assert!(pruefen(h, "Haus").unwrap());
        assert!(pruefen(h, "Hause").unwrap());
        assert!(!pruefen(h, "Hasu").unwrap());
        assert!(vorschlaege(h, "Hasu", 5).unwrap().contains(&"Haus".to_string()));
        assert_eq!(fehler(h, "Ein Hasu im Garten").unwrap(), vec![(0, 3), (4, 4), (9, 2)]);
        ignorieren(h, "Hasu").unwrap();
        assert!(pruefen(h, "Hasu").unwrap());
        freigeben(h);
        assert!(pruefen(h, "Haus").unwrap_err().contains("kein Woerterbuch"));
    }
}
