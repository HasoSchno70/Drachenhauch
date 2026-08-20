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
    ///
    /// ACHTUNG, das ist NICHT "wie viele rechnen noch": gezaehlt wird,
    /// was noch nicht ABGEHOLT ist. Ein fertiger, aber nicht abgeholter
    /// Auftrag zaehlt mit. Wer `WHILE PENDING() > 0` schreibt und erst
    /// danach abholen will, wartet ewig -- gefragt wird pro Auftrag mit
    /// `fertig`.
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

/// Ergebnis eines Auftrags (`TASK_*`).
pub struct TaskErgebnis {
    /// Der Rueckgabewert der Funktion, als Text.
    pub ergebnis: String,
    /// Was die Funktion selbst gedruckt hat -- getrennt, damit der Aufrufer
    /// nicht Ergebnis und Ausgabe auseinanderfieseln muss.
    pub ausgabe: String,
}

/// Eine GB-Funktion in einem EIGENEN dhrt-Prozess ausfuehren.
///
/// Warum ein Prozess und kein Thread: `Value` haelt ueberall `Rc`, `Program`
/// ist damit weder `Send` noch `Sync` und laesst sich nicht ueber eine
/// Thread-Grenze reichen. Ein Prozess teilt keinen Speicher -- damit
/// verschwindet das Problem, ohne eine Zeile an `Value` zu aendern. Der Preis
/// sind gemessene ~12 ms Prozessstart. Siehe docs/entwurf-task-start.md.
///
/// Laeuft ohne VM, also auf einem Auftrags-Thread wie `shell_arbeit`.
pub fn task_arbeit(exe: std::path::PathBuf, datei: String, funktion: String,
                   arg: Option<String>) -> Result<TaskErgebnis, String> {
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("call").arg(&datei).arg(&funktion);
    if let Some(a) = arg { cmd.arg(a); }
    let out = cmd.output().map_err(|e| format!(
        "TASK_START: '{}' laesst sich nicht starten: {}", exe.display(), e))?;

    let roh = String::from_utf8_lossy(&out.stdout);
    // `dhrt call` antwortet mit genau einer JSON-Zeile -- auch im Fehlerfall.
    // Die letzte nehmen, falls doch etwas davor landet.
    let zeile = roh.lines().rev().find(|z| !z.trim().is_empty()).unwrap_or("");
    let v: serde_json::Value = serde_json::from_str(zeile).map_err(|_| {
        let err = String::from_utf8_lossy(&out.stderr);
        format!("TASK_START: unverstaendliche Antwort des Auftrags: {}",
                if err.trim().is_empty() { zeile.to_string() } else { err.into_owned() })
    })?;
    if v.get("ok").and_then(|b| b.as_bool()) != Some(true) {
        return Err(format!("TASK: {}", v.get("fehler")
            .and_then(|f| f.as_str()).unwrap_or("unbekannter Fehler")));
    }
    let ergebnis = match v.get("ergebnis") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Null) | None => String::new(),
        Some(andere) => andere.to_string(),
    };
    Ok(TaskErgebnis {
        ergebnis,
        ausgabe: v.get("ausgabe").and_then(|s| s.as_str()).unwrap_or("").to_string(),
    })
}
