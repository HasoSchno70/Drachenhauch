//! Hintergrund-Auftraege (WP H).
//!
//! Ein Muster fuer alles, was laenger dauert als ein Bild: starten, pro Frame
//! nachsehen, abholen. Dasselbe, was `html.rs` fuer HTTP schon tut -- hier
//! einmal allgemein, damit die naechste blockierende Sache es nicht ein
//! drittes Mal nachbaut.
//!
//! **Was hier NICHT laeuft: GB-Code.** Ein Auftrag ist eine Rust-Funktion, die
//! ohne VM auskommt (eine SQL-Abfrage, ein Kindprozess). Der Grund steht in
//! `docs/allzweck-roadmap.md` unter WP H: `Value` haelt seine Zeichenketten,
//! Felder und Objekte in `Rc`, und ein `Rc` darf den Thread nicht wechseln.
//! Eine GB-Funktion im Hintergrund auszufuehren hiesse also, `Value` (und
//! damit auch `Func`/`Program`) auf `Arc` umzustellen -- was JEDEM
//! einthreadigen Programm Kosten aufbuerdet, um einem seltenen Fall zu
//! helfen. Diese Entscheidung gehoert nicht nebenbei getroffen.

use std::sync::mpsc::{channel, Receiver, TryRecvError};

struct Auftrag<T> {
    empfang: Receiver<T>,
    /// Fertiges Ergebnis, bis es abgeholt wird. Der Kanal liefert nur EINMAL --
    /// ohne diesen Zwischenspeicher waere `fertig()` ein Verbrauch, und ein
    /// zweiter Aufruf im selben Bild verloere das Ergebnis.
    ergebnis: Option<T>,
    /// Kurzbeschreibung fuer Fehlermeldungen ("SELECT ...", "git status").
    was: String,
}

/// Tombstone-Vec: eine einmal vergebene Nummer bleibt gueltig, auch wenn
/// davorstehende Auftraege schon abgeholt wurden.
pub struct Auftraege<T> {
    eintraege: Vec<Option<Auftrag<T>>>,
}

impl<T> Default for Auftraege<T> {
    fn default() -> Self { Auftraege { eintraege: Vec::new() } }
}

impl<T: Send + 'static> Auftraege<T> {
    /// Startet `arbeit` auf einem eigenen Thread und liefert die Auftragsnummer.
    pub fn start(&mut self, was: &str, arbeit: impl FnOnce() -> T + Send + 'static) -> i64 {
        let (sender, empfang) = channel();
        std::thread::spawn(move || {
            // Scheitert still, wenn der Empfaenger weg ist (Auftrag
            // abgebrochen) -- das Ergebnis interessiert dann niemanden mehr.
            let _ = sender.send(arbeit());
        });
        self.eintraege.push(Some(Auftrag { empfang, ergebnis: None, was: was.to_string() }));
        (self.eintraege.len() - 1) as i64
    }

    fn eintrag(&mut self, id: i64) -> Option<&mut Auftrag<T>> {
        if id < 0 { return None; }
        self.eintraege.get_mut(id as usize)?.as_mut()
    }

    /// Ist das Ergebnis da? Fragt nach, ohne zu warten.
    ///
    /// `Err` heisst: der Thread ist weg, ohne zu senden -- das kann nur ein
    /// Absturz im Auftrag sein. Als Fehler melden statt ewig "noch nicht".
    pub fn fertig(&mut self, id: i64) -> Result<bool, String> {
        let Some(e) = self.eintrag(id) else { return Ok(false) };
        if e.ergebnis.is_some() { return Ok(true); }
        match e.empfang.try_recv() {
            Ok(r) => { e.ergebnis = Some(r); Ok(true) }
            Err(TryRecvError::Empty) => Ok(false),
            Err(TryRecvError::Disconnected) =>
                Err(format!("Hintergrund-Auftrag abgebrochen: {}", e.was)),
        }
    }

    /// Holt das Ergebnis ab und gibt den Platz frei. `None` = noch nicht
    /// fertig (oder unbekannte Nummer); der Aufrufer soll dann weiter
    /// `fertig` fragen statt zu blockieren.
    pub fn abholen(&mut self, id: i64) -> Result<Option<T>, String> {
        if !self.fertig(id)? { return Ok(None); }
        let Some(platz) = self.eintraege.get_mut(id as usize) else { return Ok(None) };
        Ok(platz.take().and_then(|e| e.ergebnis))
    }

    /// Abbrechen: der Thread laeuft zu Ende, sein Ergebnis wird verworfen.
    /// Tolerant gegenueber schon abgeholten Nummern.
    pub fn abbrechen(&mut self, id: i64) {
        if id >= 0 {
            if let Some(platz) = self.eintraege.get_mut(id as usize) { *platz = None; }
        }
    }

    /// Wie viele Auftraege noch offen sind (fuer Anzeigen wie "lade ...").
    pub fn offen(&self) -> i64 {
        self.eintraege.iter().filter(|e| e.is_some()).count() as i64
    }
}

/// Was ein Kindprozess im Hintergrund zurueckbringt.
pub struct ShellErgebnis {
    pub code: i64,
    pub stdout: String,
    pub stderr: String,
}

/// Programm starten, auf sein Ende warten, Ausgabe einsammeln -- ohne VM,
/// laeuft also auf einem Auftrags-Thread.
pub fn shell_arbeit(prog: String, args: Vec<String>) -> Result<ShellErgebnis, String> {
    let out = std::process::Command::new(&prog).args(&args).output()
        .map_err(|e| format!("SHELL_START: '{}' laesst sich nicht starten: {}", prog, e))?;
    Ok(ShellErgebnis {
        // Kein Rueckgabewert (Unix: durch ein Signal beendet) -> -1, wie bei
        // SHELL. Eindeutig, weil ein echter Wert immer 0..255 ist.
        code: out.status.code().map(|c| c as i64).unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}
