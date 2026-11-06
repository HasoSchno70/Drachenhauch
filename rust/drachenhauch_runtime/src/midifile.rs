//! MIDI-Dateien (.mid, "Standard MIDI File") lesen und schreiben -- Teil des
//! Moduls `midi`, aber ohne Geraet: das hier rechnet nur, und steht darum in
//! JEDEM Bau (wie MIDI_NOTE_NAME$). Abspielen an einem Ausgang (MIDI_FILE_PLAY)
//! braucht das Feature `midi` und steht in midi.rs.
//!
//! Eine Datei besteht aus Spuren, eine Spur aus Ereignissen mit einem
//! Zeitstempel in TICKS. Wie lang ein Tick ist, sagt der Kopf (Ticks je
//! Viertel) zusammen mit den Tempowechseln -- die stehen irgendwo in einer
//! Spur (meist der ersten) und gelten fuer alle. Nach aussen spricht das
//! Modul darum in SEKUNDEN (lesen) und in SCHLAEGEN (schreiben): wer eine
//! Datei abspielt, will wissen wann, wer eine schreibt, denkt in Vierteln.

use std::collections::HashMap;

/// Hoechstens so viele Spuren (der Kopf erlaubte 65535; mehr als 128 ist
/// kein Musikstueck, sondern ein Tippfehler im Spur-Index).
pub const MAX_SPUREN: usize = 128;

#[derive(Clone)]
pub enum Art {
    /// Kanalnachricht: Status (mit Kanal) und ein oder zwei Datenbytes.
    Kanal(Vec<u8>),
    Meta(u8, Vec<u8>),
    Sysex(Vec<u8>),
}

#[derive(Clone)]
pub struct Ereignis {
    pub tick: u64,
    pub art: Art,
}

#[derive(Clone, Default)]
pub struct Spur {
    pub name: String,
    pub ereignisse: Vec<Ereignis>,
    /// Wo "Ende der Spur" stand -- kann hinter dem letzten Ereignis liegen
    /// (ein Takt Stille am Schluss).
    pub ende: u64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Teilung {
    Ppq(u16),
    Smpte { fps: u8, tpf: u8 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub start: f64,
    pub dauer: f64,
    pub kanal: u8,
    pub note: u8,
    pub anschlag: u8,
    pub spur: usize,
}

/// Zeitlich geordnet ueber alle Spuren -- gerechnet, sobald jemand fragt,
/// und verworfen, sobald sich etwas aendert.
struct Ansicht {
    /// (Sekunde, Spur, Bytes) der Kanalnachrichten.
    ereignisse: Vec<(f64, usize, Vec<u8>)>,
    noten: Vec<Note>,
    dauer: f64,
}

pub struct Datei {
    pub teilung: Teilung,
    pub spuren: Vec<Spur>,
    ansicht: Option<Ansicht>,
}

fn vlq_lesen(b: &[u8], pos: &mut usize) -> Result<u64, String> {
    let mut v: u64 = 0;
    for k in 0..4 {
        let x = *b.get(*pos).ok_or("Datei endet mitten in einer Zahl")?;
        *pos += 1;
        v = (v << 7) | (x & 0x7F) as u64;
        if x & 0x80 == 0 { return Ok(v); }
        if k == 3 { break; }
    }
    Err("eine Zahl ist laenger als vier Bytes".into())
}

pub fn vlq_schreiben(mut v: u64, out: &mut Vec<u8>) {
    let mut teile = vec![(v & 0x7F) as u8];
    v >>= 7;
    while v > 0 {
        teile.push(((v & 0x7F) as u8) | 0x80);
        v >>= 7;
    }
    teile.reverse();
    out.extend_from_slice(&teile);
}

fn u16_be(b: &[u8], p: usize) -> u16 { ((b[p] as u16) << 8) | b[p + 1] as u16 }
fn u32_be(b: &[u8], p: usize) -> u32 { u32::from_be_bytes([b[p], b[p + 1], b[p + 2], b[p + 3]]) }

fn spur_lesen(b: &[u8]) -> Result<Spur, String> {
    let mut s = Spur::default();
    let mut pos = 0;
    let mut tick: u64 = 0;
    let mut laufend: Option<u8> = None;
    while pos < b.len() {
        tick += vlq_lesen(b, &mut pos)?;
        let mut st = *b.get(pos).ok_or("Spur endet nach einer Zeitangabe")?;
        if st >= 0x80 {
            pos += 1;
        } else {
            st = laufend.ok_or_else(|| format!("Datenbyte {} ohne Statusbyte davor", st))?;
        }
        match st {
            0xFF => {
                laufend = None;
                let typ = *b.get(pos).ok_or("Spur endet in einem Meta-Ereignis")?;
                pos += 1;
                let l = vlq_lesen(b, &mut pos)? as usize;
                let d = b.get(pos..pos + l).ok_or("Spur endet in einem Meta-Ereignis")?.to_vec();
                pos += l;
                match typ {
                    0x2F => { s.ende = tick; return Ok(s); }
                    0x03 if s.name.is_empty() => s.name = String::from_utf8_lossy(&d).into_owned(),
                    _ => s.ereignisse.push(Ereignis { tick, art: Art::Meta(typ, d) }),
                }
            }
            0xF0 | 0xF7 => {
                laufend = None;
                let l = vlq_lesen(b, &mut pos)? as usize;
                let d = b.get(pos..pos + l).ok_or("Spur endet in einer SysEx-Nachricht")?.to_vec();
                pos += l;
                s.ereignisse.push(Ereignis { tick, art: Art::Sysex(d) });
            }
            0xF1..=0xFE => return Err(format!("Statusbyte {} gehoert nicht in eine Datei", st)),
            _ => {
                laufend = Some(st);
                let n = if matches!(st & 0xF0, 0xC0 | 0xD0) { 1 } else { 2 };
                let d = b.get(pos..pos + n).ok_or("Spur endet in einer Nachricht")?;
                if d.iter().any(|x| *x >= 0x80) {
                    return Err(format!("Nachricht {} mit Datenbyte ueber 127", st));
                }
                let mut m = vec![st];
                m.extend_from_slice(d);
                pos += n;
                s.ereignisse.push(Ereignis { tick, art: Art::Kanal(m) });
            }
        }
    }
    // Ohne "Ende der Spur" -- viele Programme vergessen es. Kein Fehler.
    s.ende = tick;
    Ok(s)
}

impl Datei {
    pub fn neu(ppq: u16) -> Datei {
        Datei { teilung: Teilung::Ppq(ppq), spuren: Vec::new(), ansicht: None }
    }

    pub fn lesen(b: &[u8]) -> Result<Datei, String> {
        if b.len() < 14 || &b[0..4] != b"MThd" {
            return Err("keine MIDI-Datei (der Kopf \"MThd\" fehlt)".into());
        }
        let kl = u32_be(b, 4) as usize;
        if kl < 6 || b.len() < 8 + kl { return Err("der Kopf der Datei ist zu kurz".into()); }
        let format = u16_be(b, 8);
        if format > 2 { return Err(format!("Format {} gibt es nicht (0, 1 oder 2)", format)); }
        let div = u16_be(b, 12);
        let teilung = if div & 0x8000 != 0 {
            let fps = (-(((div >> 8) as u8 as i8) as i16)) as u8;
            let tpf = div as u8;
            if fps == 0 || tpf == 0 { return Err("Zeitteilung im Kopf ist ungueltig".into()); }
            Teilung::Smpte { fps, tpf }
        } else {
            if div == 0 { return Err("0 Ticks je Viertel im Kopf".into()); }
            Teilung::Ppq(div)
        };
        let mut pos = 8 + kl;
        let mut spuren = Vec::new();
        while pos + 8 <= b.len() {
            let l = u32_be(b, pos + 4) as usize;
            let ende = (pos + 8).saturating_add(l).min(b.len());
            if &b[pos..pos + 4] == b"MTrk" {
                if pos + 8 + l > b.len() {
                    return Err(format!("Spur {}: die Datei ist abgeschnitten ({} von {} Bytes)",
                                       spuren.len(), b.len() - pos - 8, l));
                }
                if spuren.len() >= MAX_SPUREN { return Err(format!("mehr als {} Spuren", MAX_SPUREN)); }
                let s = spur_lesen(&b[pos + 8..ende])
                    .map_err(|e| format!("Spur {}: {}", spuren.len(), e))?;
                spuren.push(s);
            }
            pos = ende;
        }
        Ok(Datei { teilung, spuren, ansicht: None })
    }

    pub fn schreiben(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"MThd");
        o.extend_from_slice(&6u32.to_be_bytes());
        o.extend_from_slice(&1u16.to_be_bytes());
        o.extend_from_slice(&(self.spuren.len() as u16).to_be_bytes());
        let div: u16 = match self.teilung {
            Teilung::Ppq(p) => p,
            Teilung::Smpte { fps, tpf } => (((-(fps as i8)) as u8 as u16) << 8) | tpf as u16,
        };
        o.extend_from_slice(&div.to_be_bytes());
        for s in &self.spuren {
            let mut t = Vec::new();
            if !s.name.is_empty() {
                t.push(0);
                t.extend_from_slice(&[0xFF, 0x03]);
                vlq_schreiben(s.name.len() as u64, &mut t);
                t.extend_from_slice(s.name.as_bytes());
            }
            // Zur selben Zeit: erst Meta (Tempo vor der Note, die es betrifft),
            // dann Note-aus (sonst schnitte eine wiederholte Note sich selbst
            // ab), dann Programm/Regler, zuletzt Note-an.
            let mut ord: Vec<&Ereignis> = s.ereignisse.iter().collect();
            ord.sort_by_key(|e| (e.tick, rang(e)));
            let mut zuletzt = 0u64;
            for e in ord {
                vlq_schreiben(e.tick - zuletzt, &mut t);
                zuletzt = e.tick;
                match &e.art {
                    Art::Kanal(m) => t.extend_from_slice(m),
                    Art::Meta(typ, d) => {
                        t.extend_from_slice(&[0xFF, *typ]);
                        vlq_schreiben(d.len() as u64, &mut t);
                        t.extend_from_slice(d);
                    }
                    Art::Sysex(d) => {
                        t.push(0xF0);
                        vlq_schreiben(d.len() as u64, &mut t);
                        t.extend_from_slice(d);
                    }
                }
            }
            vlq_schreiben(s.ende.max(zuletzt) - zuletzt, &mut t);
            t.extend_from_slice(&[0xFF, 0x2F, 0x00]);
            o.extend_from_slice(b"MTrk");
            o.extend_from_slice(&(t.len() as u32).to_be_bytes());
            o.extend_from_slice(&t);
        }
        o
    }

    pub fn ppq(&self) -> Option<u16> {
        match self.teilung { Teilung::Ppq(p) => Some(p), _ => None }
    }

    /// Die Spur `k`, angelegt wenn noetig (samt allen davor).
    pub fn spur(&mut self, k: usize) -> Result<&mut Spur, String> {
        if k >= MAX_SPUREN { return Err(format!("Spur {} ausserhalb 0..{}", k, MAX_SPUREN - 1)); }
        while self.spuren.len() <= k { self.spuren.push(Spur::default()); }
        self.ansicht = None;
        Ok(&mut self.spuren[k])
    }

    pub fn hinzu(&mut self, spur: usize, tick: u64, art: Art) -> Result<(), String> {
        self.spur(spur)?.ereignisse.push(Ereignis { tick, art });
        Ok(())
    }

    /// Tempo in Spur 0 setzen; ein Wechsel an genau dieser Stelle wird
    /// ersetzt statt verdoppelt.
    pub fn tempo_setzen(&mut self, tick: u64, bpm: f64) -> Result<(), String> {
        let us = (60_000_000.0 / bpm).round() as u32;
        let d = vec![(us >> 16) as u8, (us >> 8) as u8, us as u8];
        let s = self.spur(0)?;
        s.ereignisse.retain(|e| !(e.tick == tick && matches!(e.art, Art::Meta(0x51, _))));
        s.ereignisse.push(Ereignis { tick, art: Art::Meta(0x51, d) });
        Ok(())
    }

    /// Tempowechsel (Tick, Mikrosekunden je Viertel) ueber alle Spuren.
    fn tempi(&self) -> Vec<(u64, u32)> {
        let mut v: Vec<(u64, u32)> = Vec::new();
        for s in &self.spuren {
            for e in &s.ereignisse {
                if let Art::Meta(0x51, d) = &e.art {
                    if d.len() == 3 {
                        v.push((e.tick, ((d[0] as u32) << 16) | ((d[1] as u32) << 8) | d[2] as u32));
                    }
                }
            }
        }
        v.sort_by_key(|t| t.0);
        v
    }

    pub fn anfangstempo(&self) -> f64 {
        match self.tempi().first() {
            Some((0, us)) if *us > 0 => 60_000_000.0 / *us as f64,
            _ => 120.0,
        }
    }

    pub fn sekunde(&self, tick: u64, tempi: &[(u64, u32)]) -> f64 {
        match self.teilung {
            Teilung::Smpte { fps, tpf } => {
                // 29 steht fuer 29,97 (Drop-Frame).
                let f = if fps == 29 { 29.97 } else { fps as f64 };
                tick as f64 / (f * tpf as f64)
            }
            Teilung::Ppq(ppq) => {
                let mut t = 0.0;
                let mut ab = 0u64;
                let mut us = 500_000u32;
                for &(tt, neu) in tempi {
                    if tt >= tick { break; }
                    t += (tt - ab) as f64 * us as f64 / ppq as f64;
                    ab = tt;
                    us = neu;
                }
                (t + (tick - ab) as f64 * us as f64 / ppq as f64) / 1e6
            }
        }
    }

    fn ansicht(&mut self) -> &Ansicht {
        if self.ansicht.is_none() {
            let tempi = self.tempi();
            let mut alle: Vec<(u64, u8, usize, usize)> = Vec::new();
            let mut ende = 0u64;
            for (k, s) in self.spuren.iter().enumerate() {
                ende = ende.max(s.ende);
                for (i, e) in s.ereignisse.iter().enumerate() {
                    ende = ende.max(e.tick);
                    if matches!(e.art, Art::Kanal(_)) { alle.push((e.tick, rang(e), k, i)); }
                }
            }
            // Zur selben Zeit in derselben Folge wie beim Schreiben -- auch
            // fuer eine Datei, die noch nie geschrieben wurde.
            alle.sort();
            let mut ereignisse = Vec::with_capacity(alle.len());
            let mut noten: Vec<Note> = Vec::new();
            // Offene Noten je (Spur, Kanal, Note): ein Note-aus beendet die
            // AELTESTE offene (wie jedes Programm, das zweimal dieselbe Taste
            // anschlaegt, sie auch wieder loslaesst).
            let mut offen: HashMap<(usize, u8, u8), Vec<usize>> = HashMap::new();
            for (tick, _, k, i) in alle {
                let Art::Kanal(m) = &self.spuren[k].ereignisse[i].art else { continue };
                let t = self.sekunde(tick, &tempi);
                let art = m[0] & 0xF0;
                let kanal = m[0] & 0x0F;
                if art == 0x90 && m.get(2).copied().unwrap_or(0) > 0 {
                    offen.entry((k, kanal, m[1])).or_default().push(noten.len());
                    noten.push(Note { start: t, dauer: -1.0, kanal: kanal + 1, note: m[1], anschlag: m[2], spur: k });
                } else if art == 0x80 || art == 0x90 {
                    if let Some(l) = offen.get_mut(&(k, kanal, m[1])) {
                        if !l.is_empty() {
                            let n = l.remove(0);
                            noten[n].dauer = t - noten[n].start;
                        }
                    }
                }
                ereignisse.push((t, k, m.clone()));
            }
            let dauer = self.sekunde(ende, &tempi);
            for n in noten.iter_mut() { if n.dauer < 0.0 { n.dauer = dauer - n.start; } }
            // Nach Beginn geordnet (stabil: bei gleichem Beginn bleibt die
            // Reihenfolge der Datei).
            noten.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
            self.ansicht = Some(Ansicht { ereignisse, noten, dauer });
        }
        self.ansicht.as_ref().unwrap()
    }

    pub fn dauer(&mut self) -> f64 { self.ansicht().dauer }
    pub fn ereignis_zahl(&mut self) -> usize { self.ansicht().ereignisse.len() }
    pub fn ereignis(&mut self, i: usize) -> Option<(f64, usize, Vec<u8>)> { self.ansicht().ereignisse.get(i).cloned() }
    pub fn noten_zahl(&mut self) -> usize { self.ansicht().noten.len() }
    pub fn note(&mut self, i: usize) -> Option<Note> { self.ansicht().noten.get(i).cloned() }
    /// Alle Kanalnachrichten mit Zeit -- fuer das Abspielen.
    pub fn ablauf(&mut self) -> Vec<(f64, Vec<u8>)> {
        self.ansicht().ereignisse.iter().map(|(t, _, m)| (*t, m.clone())).collect()
    }
}

fn rang(e: &Ereignis) -> u8 {
    match &e.art {
        Art::Meta(..) => 0,
        Art::Kanal(m) if m[0] & 0xF0 == 0x80 || (m[0] & 0xF0 == 0x90 && m.get(2) == Some(&0)) => 1,
        // Note-an zuletzt: Programm und Regler muessen VOR der Note stehen,
        // die sie betreffen, sonst klingt sie noch mit dem alten Klang.
        Art::Kanal(m) if m[0] & 0xF0 == 0x90 => 3,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zahlen_wie_in_der_spezifikation() {
        // Die Tabelle der Spezifikation (SMF 1.0, "Variable-length quantity").
        for (v, b) in [(0u64, vec![0x00]), (0x40, vec![0x40]), (0x7F, vec![0x7F]), (0x80, vec![0x81, 0x00]),
                       (0x2000, vec![0xC0, 0x00]), (0x3FFF, vec![0xFF, 0x7F]), (0x4000, vec![0x81, 0x80, 0x00]),
                       (0x0FFFFFFF, vec![0xFF, 0xFF, 0xFF, 0x7F])] {
            let mut o = Vec::new();
            vlq_schreiben(v, &mut o);
            assert_eq!(o, b, "{:#x}", v);
            let mut p = 0;
            assert_eq!(vlq_lesen(&b, &mut p).unwrap(), v);
        }
    }

    /// Die Beispieldatei aus der Spezifikation (Format 0, ein Akkord).
    fn beispiel() -> Vec<u8> {
        let mut b = vec![0x4D, 0x54, 0x68, 0x64, 0, 0, 0, 6, 0, 0, 0, 1, 0, 0x60,
                         0x4D, 0x54, 0x72, 0x6B, 0, 0, 0, 0x3B];
        b.extend_from_slice(&[
            0x00, 0xFF, 0x58, 0x04, 0x04, 0x02, 0x18, 0x08,
            0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20,
            0x00, 0xC0, 0x05, 0x00, 0xC1, 0x2E, 0x00, 0xC2, 0x46,
            0x00, 0x92, 0x30, 0x60, 0x00, 0x3C, 0x60,
            0x60, 0x91, 0x43, 0x40,
            0x60, 0x90, 0x4C, 0x20,
            0x81, 0x40, 0x82, 0x30, 0x40, 0x00, 0x3C, 0x40, 0x00, 0x81, 0x43, 0x40, 0x00, 0x80, 0x4C, 0x40,
            0x00, 0xFF, 0x2F, 0x00]);
        b
    }

    #[test]
    fn die_beispieldatei_lesen() {
        let mut d = Datei::lesen(&beispiel()).unwrap();
        assert_eq!(d.teilung, Teilung::Ppq(96));
        assert_eq!(d.spuren.len(), 1);
        assert!((d.anfangstempo() - 120.0).abs() < 1e-9);
        assert_eq!(d.noten_zahl(), 4);
        let n: Vec<(f64, f64, u8, u8)> = (0..4).map(|i| { let n = d.note(i).unwrap(); (n.start, n.dauer, n.kanal, n.note) }).collect();
        assert_eq!(n, vec![(0.0, 2.0, 3, 0x30), (0.0, 2.0, 3, 0x3C), (0.5, 1.5, 2, 0x43), (1.0, 1.0, 1, 0x4C)]);
        assert!((d.dauer() - 2.0).abs() < 1e-9);
        // Laufender Status: das zweite Note-an (00 3C 60) hat kein eigenes Statusbyte.
        assert_eq!(d.ereignis(4).unwrap().2, vec![0x92, 0x3C, 0x60]);
    }

    #[test]
    fn tempowechsel_und_rundweg() {
        let mut d = Datei::neu(480);
        d.tempo_setzen(0, 120.0).unwrap();
        d.tempo_setzen(960, 60.0).unwrap();
        d.spur(1).unwrap().name = "Bass".into();
        for (k, t) in [0u64, 960, 1920].iter().enumerate() {
            d.hinzu(1, *t, Art::Kanal(vec![0x90, 40 + k as u8, 100])).unwrap();
            d.hinzu(1, *t + 480, Art::Kanal(vec![0x80, 40 + k as u8, 64])).unwrap();
        }
        // Zwei Schlaege zu 0,5 s, dann je Schlag 1 s.
        let starts: Vec<f64> = (0..3).map(|i| d.note(i).unwrap().start).collect();
        assert_eq!(starts, vec![0.0, 1.0, 3.0]);
        assert_eq!(d.note(1).unwrap().dauer, 1.0);
        let bytes = d.schreiben();
        let mut e = Datei::lesen(&bytes).unwrap();
        assert_eq!(e.spuren[1].name, "Bass");
        assert_eq!(e.noten_zahl(), 3);
        assert_eq!(e.note(2).unwrap(), d.note(2).unwrap());
        assert_eq!(e.schreiben(), bytes);
    }

    #[test]
    fn note_aus_vor_note_an_zur_selben_zeit() {
        let mut d = Datei::neu(96);
        d.hinzu(0, 0, Art::Kanal(vec![0x90, 60, 100])).unwrap();
        d.hinzu(0, 96, Art::Kanal(vec![0x90, 60, 100])).unwrap();
        d.hinzu(0, 96, Art::Kanal(vec![0x80, 60, 64])).unwrap();
        d.hinzu(0, 192, Art::Kanal(vec![0x80, 60, 64])).unwrap();
        let b = d.schreiben();
        // In der Datei steht bei Tick 96 erst das Note-aus, dann das Note-an.
        let aus = b.windows(3).position(|w| w == [0x80, 60, 64]).unwrap();
        let an2 = b.windows(3).rposition(|w| w == [0x90, 60, 100]).unwrap();
        assert!(aus < an2);
        let mut e = Datei::lesen(&b).unwrap();
        assert_eq!(e.noten_zahl(), 2);
        assert_eq!(e.note(0).unwrap().dauer, 0.5);
        assert_eq!(e.note(1).unwrap().dauer, 0.5);
    }

    #[test]
    fn kaputtes_wird_gemeldet() {
        assert!(Datei::lesen(b"RIFF....").err().unwrap().contains("MThd"));
        let mut b = beispiel();
        b.truncate(30);
        assert!(Datei::lesen(&b).err().unwrap().starts_with("Spur 0:"));
    }
}
