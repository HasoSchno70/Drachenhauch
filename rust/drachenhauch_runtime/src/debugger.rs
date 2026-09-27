//! Bausteine fuer `dhrt debug`, die nichts von der VM wissen muessen.
//!
//! **Der Kommando-Kanal.** Bis 2026-09-27 las der Debugger stdin NUR, solange
//! das Programm stand -- ein Haltepunkt, den man waehrend des Laufens setzte,
//! kam erst beim naechsten Halt an, also fuer ein Programm, das nie hielt,
//! gar nicht. Jetzt liest ein eigener Faden jede Zeile in eine Warteschlange.
//! Die VM fragt bei jedem Zeilenwechsel nach "lebenden" Kommandos
//! (`set-breakpoints`, `set-watches`, `pause`) und nimmt nur die, vor denen
//! kein Schritt-Kommando steht: ein `continue`, das schon vor dem naechsten
//! Halt geschickt wurde, gilt diesem Halt, und was dahinter steht, ebenfalls.
//! `input`-Kommandos warten, bis ein INPUT sie abholt.
//!
//! **Die Zeilenkarte.** Die VM zaehlt Zeilen der GEMERGTEN Quelle (nach dem
//! Einsetzen der IMPORTs). Ein Programm mit einem `IMPORT` am Anfang hielt
//! darum in "Zeile 7", obwohl es in Zeile 3 stand, und ein Haltepunkt in
//! Zeile 4 traf nie. Die Karte rechnet in beide Richtungen um -- nach aussen
//! spricht der Debugger nur noch (Datei, Zeile) -- und schiebt einen
//! Haltepunkt auf eine Zeile ohne Code (Kommentar, Leerzeile, `END IF`,
//! `SUB`-Kopf) zur naechsten Zeile DERSELBEN Datei, die Code hat.

use std::collections::{BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};

/// Kommandos, die auch waehrend des Laufens wirken: `pause` immer,
/// `set-breakpoints`/`set-watches` nur mit `"now": true`. Ohne den Schalter
/// gelten sie dem naechsten Halt wie bisher -- ein Skript, das alle Kommandos
/// vorab schickt, meint mit einem `set-breakpoints` hinter einem `continue`
/// den Halt DANACH, nicht "sofort".
fn ist_lebend(cmd: &serde_json::Value) -> bool {
    let jetzt = cmd.get("now").and_then(|v| v.as_bool()).unwrap_or(false);
    match art(cmd) {
        "pause" => true,
        "set-breakpoints" | "set-watches" => jetzt,
        _ => false,
    }
}

pub fn art(cmd: &serde_json::Value) -> &str {
    cmd.get("cmd").and_then(|v| v.as_str()).unwrap_or("")
}

pub struct Kanal {
    rx: Option<Receiver<Option<serde_json::Value>>>,
    warte: VecDeque<serde_json::Value>,
    zu: bool,
}

impl Kanal {
    /// Faden starten, der stdin zeilenweise liest. Eine Zeile, die kein JSON
    /// ist, wird uebergangen (wie frueher).
    pub fn neu() -> Kanal {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            use std::io::BufRead;
            let stdin = std::io::stdin();
            let mut h = stdin.lock();
            loop {
                let mut zeile = String::new();
                match h.read_line(&mut zeile) {
                    Ok(0) | Err(_) => { let _ = tx.send(None); return; }
                    Ok(_) => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(zeile.trim()) {
                            if tx.send(Some(v)).is_err() { return; }
                        }
                    }
                }
            }
        });
        Kanal { rx: Some(rx), warte: VecDeque::new(), zu: false }
    }

    /// Fuer Tests: ein Kanal aus einer festen Liste (danach geschlossen).
    #[cfg(test)]
    pub fn aus(cmds: Vec<serde_json::Value>) -> Kanal {
        Kanal { rx: None, warte: cmds.into(), zu: true }
    }

    fn einsammeln(&mut self) {
        let rx = match &self.rx { Some(r) => r, None => return };
        loop {
            match rx.try_recv() {
                Ok(Some(v)) => self.warte.push_back(v),
                Ok(None) | Err(TryRecvError::Disconnected) => { self.zu = true; self.rx = None; return; }
                Err(TryRecvError::Empty) => return,
            }
        }
    }

    /// Das naechste Kommando, das kein `input` ist -- blockierend. None =
    /// stdin ist zu und nichts mehr da.
    pub fn naechstes(&mut self) -> Option<serde_json::Value> {
        loop {
            self.einsammeln();
            if let Some(i) = self.warte.iter().position(|c| art(c) != "input") {
                return self.warte.remove(i);
            }
            if self.zu { return None; }
            self.warten();
        }
    }

    /// Einen Moment auf ein neues Kommando warten (blockierend).
    fn warten(&mut self) {
        let rx = match &self.rx { Some(r) => r, None => { self.zu = true; return; } };
        match rx.recv() {
            Ok(Some(v)) => self.warte.push_back(v),
            _ => { self.zu = true; self.rx = None; }
        }
    }

    /// Die lebenden Kommandos, vor denen kein Schritt-Kommando steht.
    /// `input` ist keine Schranke -- es wartet auf seinen INPUT.
    pub fn lebende(&mut self) -> Vec<serde_json::Value> {
        self.einsammeln();
        let mut raus = Vec::new();
        let mut i = 0;
        while i < self.warte.len() {
            let a = art(&self.warte[i]).to_string();
            if ist_lebend(&self.warte[i]) {
                raus.push(self.warte.remove(i).unwrap());
            } else if a == "input" {
                i += 1;
            } else {
                break;
            }
        }
        raus
    }

    /// Eine Eingabezeile fuer INPUT: das erste `input`-Kommando. Steht
    /// davor ein `stop`, gilt das (Err). None = noch keine da.
    pub fn eingabe(&mut self) -> Result<Option<String>, ()> {
        self.einsammeln();
        for i in 0..self.warte.len() {
            match art(&self.warte[i]) {
                "input" => {
                    let c = self.warte.remove(i).unwrap();
                    return Ok(Some(c.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string()));
                }
                "stop" => return Err(()),
                _ => {}
            }
        }
        Ok(None)
    }

    /// Blockierend auf irgendein neues Kommando warten; FALSE = stdin zu.
    pub fn auf_neues_warten(&mut self) -> bool {
        self.einsammeln();
        if self.zu { return false; }
        self.warten();
        !self.zu || !self.warte.is_empty()
    }
}

/// Umrechnung gemergte Zeile <-> (Datei, Zeile).
pub struct Karte {
    /// Je gemergter Zeile (Index = Zeile - 1): Dateinummer und Zeile darin.
    herkunft: Vec<(usize, u32)>,
    /// Anzeigename je Datei (Hauptdatei: wie beim Aufruf angegeben).
    anzeige: Vec<String>,
    /// Kanonischer Pfad je Datei (zum Vergleichen).
    kanon: Vec<PathBuf>,
    /// Gemergte Zeilen, an denen Code steht.
    ausfuehrbar: BTreeSet<u32>,
}

/// Pfad vergleichbar machen: kanonisch, ohne `\\?\`, unter Windows klein.
pub fn vergleichbar(p: &str) -> PathBuf {
    let pb = PathBuf::from(p);
    let c = std::fs::canonicalize(&pb).unwrap_or(pb);
    let s = c.to_string_lossy().to_string();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
    #[cfg(windows)]
    let s = s.replace('/', "\\").to_lowercase();
    PathBuf::from(s)
}

impl Karte {
    /// `herkunft`: je gemergter Zeile (Pfad der Datei, "" = Hauptdatei; Zeile).
    pub fn neu(haupt_anzeige: &str, haupt_pfad: &str, herkunft: &[(String, u32)],
               ausfuehrbar: BTreeSet<u32>) -> Karte {
        let mut anzeige = vec![haupt_anzeige.to_string()];
        let mut kanon = vec![vergleichbar(haupt_pfad)];
        let mut nummern: Vec<(String, usize)> = Vec::new();
        let mut h = Vec::with_capacity(herkunft.len());
        for (pfad, zeile) in herkunft {
            let nr = if pfad.is_empty() { 0 } else {
                match nummern.iter().find(|(p, _)| p == pfad) {
                    Some((_, n)) => *n,
                    None => {
                        anzeige.push(pfad.clone());
                        kanon.push(vergleichbar(pfad));
                        nummern.push((pfad.clone(), anzeige.len() - 1));
                        anzeige.len() - 1
                    }
                }
            };
            h.push((nr, *zeile));
        }
        Karte { herkunft: h, anzeige, kanon, ausfuehrbar }
    }

    /// (Datei, Zeile) einer gemergten Zeile. Ausserhalb der Tabelle: die
    /// Hauptdatei mit der rohen Zahl (lieber etwas als nichts).
    pub fn stelle(&self, gemergt: u32) -> (String, u32) {
        match self.herkunft.get(gemergt.saturating_sub(1) as usize) {
            Some(&(nr, z)) => (self.anzeige[nr].clone(), z),
            None => (self.anzeige[0].clone(), gemergt),
        }
    }

    fn datei_nr(&self, datei: &str) -> Option<usize> {
        if datei.is_empty() { return Some(0); }
        let v = vergleichbar(datei);
        self.kanon.iter().position(|k| *k == v)
    }

    /// Wo ein Haltepunkt in (datei, zeile) wirklich haelt: die gemergte
    /// Zeile und die Zeile in der Datei -- auf einer Zeile ohne Code die
    /// naechste darunter, die Code hat (in derselben Datei). None = die
    /// Datei gehoert nicht zum Programm oder darunter steht kein Code mehr.
    pub fn haltestelle(&self, datei: &str, zeile: u32) -> Option<(u32, u32)> {
        let nr = self.datei_nr(datei)?;
        let start = self.herkunft.iter().position(|&(n, z)| n == nr && z == zeile)?;
        for (i, &(n, z)) in self.herkunft.iter().enumerate().skip(start) {
            if n != nr { continue; }
            let g = (i + 1) as u32;
            if self.ausfuehrbar.contains(&g) { return Some((g, z)); }
        }
        None
    }

    pub fn haupt(&self) -> &str { &self.anzeige[0] }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn karte() -> Karte {
        // Hauptdatei: 1 IMPORT (Marker 1..3 mit Zeile 1), dann Zeilen 2..4.
        let h = vec![
            (String::new(), 1), ("helfer.dh".to_string(), 1), ("helfer.dh".to_string(), 2),
            (String::new(), 1), (String::new(), 2), (String::new(), 3), (String::new(), 4),
        ];
        let aus: BTreeSet<u32> = [3, 6, 7].into_iter().collect();
        Karte::neu("haupt.dh", "haupt.dh", &h, aus)
    }

    #[test]
    fn stelle_rechnet_zurueck() {
        let k = karte();
        assert_eq!(k.stelle(6), ("haupt.dh".to_string(), 3));
        assert_eq!(k.stelle(3), ("helfer.dh".to_string(), 2));
    }

    #[test]
    fn haltepunkt_rutscht_zur_naechsten_zeile_mit_code() {
        let k = karte();
        assert_eq!(k.haltestelle("", 3), Some((6, 3)));
        // Zeile 2 (DIM ohne Code) -> Zeile 3; Zeile 1 (IMPORT) ueberspringt
        // die Zeilen der importierten Datei.
        assert_eq!(k.haltestelle("", 2), Some((6, 3)));
        assert_eq!(k.haltestelle("", 1), Some((6, 3)));
        assert_eq!(k.haltestelle("helfer.dh", 1), Some((3, 2)));
        assert_eq!(k.haltestelle("", 99), None);
        assert_eq!(k.haltestelle("fremd.dh", 1), None);
    }

    #[test]
    fn lebende_kommandos_nur_vor_einer_schranke() {
        let mut k = Kanal::aus(vec![
            json!({"cmd":"set-breakpoints","lines":[3],"now":true}),
            json!({"cmd":"input","text":"5"}),
            json!({"cmd":"pause"}),
            json!({"cmd":"continue"}),
            json!({"cmd":"set-watches","exprs":[]}),
        ]);
        let l: Vec<String> = k.lebende().iter().map(|c| art(c).to_string()).collect();
        assert_eq!(l, vec!["set-breakpoints", "pause"]);
        // Ohne "now" wartet es auf den naechsten Halt.
        let mut k2 = Kanal::aus(vec![json!({"cmd":"set-breakpoints","lines":[3]})]);
        assert!(k2.lebende().is_empty());
        assert_eq!(k.eingabe(), Ok(Some("5".to_string())));
        assert_eq!(art(&k.naechstes().unwrap()), "continue");
        assert_eq!(art(&k.naechstes().unwrap()), "set-watches");
        assert!(k.naechstes().is_none());
    }

    #[test]
    fn stop_vor_der_eingabe_gewinnt() {
        let mut k = Kanal::aus(vec![json!({"cmd":"stop"}), json!({"cmd":"input","text":"x"})]);
        assert_eq!(k.eingabe(), Err(()));
    }
}
