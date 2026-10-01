//! Kindprozesse mit LAUFENDER Ausgabe (`PROCESS_*`).
//!
//! `SHELL_START` liefert die Ausgabe erst am Ende (`SHELL_RESULT$`). Eine IDE
//! in Drachenhauch (Weg C aus `docs/entwurf-python-abbau.md`) muss zeigen, was
//! ein Programm druckt, WAEHREND es laeuft, und ihm Eingaben schicken
//! (`INPUT`, der Debugger). Darum hier das Muster von `WINDOW_RECV$`:
//! Faeden lesen stdout und stderr zeilenweise in Puffer, das Programm holt
//! je Bild ab, was seither kam.
//!
//!   PROCESS_START(programm$, ...)  -> INTEGER   ("dhrt" = diese Runtime selbst)
//!   PROCESS_READ$(p) / PROCESS_ERR$(p)          neue Ausgabe seit dem letzten Abruf
//!   PROCESS_WRITE(p, text$)                     an stdin (fuer INPUT: mit Zeilenende)
//!   PROCESS_RUNNING(p) -> BOOLEAN, PROCESS_CODE(p) -> INTEGER (-1 solange laeuft)
//!   PROCESS_KILL(p), PROCESS_CLOSE(p)
//!
//! Ein Kind bekommt `DHRT_FRAMES`/`DHRT_SCREENSHOT`/`DHRT_CONTACT*` NICHT
//! vererbt -- sonst stuerbe das aus der IDE gestartete Programm nach den N
//! Bildern des IDE-Tests (dieselbe Regel wie bei `WINDOW_OPEN`). Beim Drop
//! (Programmende der IDE) werden laufende Kinder beendet.
//!
//! `PROCESS_KILL` beendet den ganzen BAUM, nicht nur das Kind (seit
//! 2026-09-29, gefunden am Terminal der IDE): `cmd /C ping ...` abgebrochen
//! liess den `ping` weiterlaufen -- und der hielt unter Windows sogar die
//! geerbten Leitungen offen, der Aufrufer wartete auf ihn. Windows ueber
//! `taskkill /T` (VOR dem eigenen kill, der Baum haengt am Elternprozess),
//! Unix ueber eine eigene Prozessgruppe je Kind.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct Prozess {
    kind: Child,
    stdin: Option<ChildStdin>,
    stdout: Arc<Mutex<String>>,
    stderr: Arc<Mutex<String>>,
    code: Option<i64>,
    was: String,
}

fn sammler<R: Read + Send + 'static>(quelle: R, ziel: Arc<Mutex<String>>) {
    std::thread::spawn(move || {
        let mut leser = BufReader::new(quelle);
        let mut zeile = Vec::new();
        loop {
            zeile.clear();
            match leser.read_until(b'\n', &mut zeile) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let text = String::from_utf8_lossy(&zeile);
                    if let Ok(mut z) = ziel.lock() { z.push_str(&text); }
                }
            }
        }
    });
}

impl Prozess {
    pub fn starten(programm: &str, args: &[String]) -> Result<Prozess, String> {
        Self::starten_mit(programm, args, &[])
    }

    /// Wie `starten`, mit zusaetzlicher Umgebung nur fuer dieses Kind.
    pub fn starten_mit(programm: &str, args: &[String], umgebung: &[(String, String)]) -> Result<Prozess, String> {
        // "dhrt" meint diese Runtime -- so startet die IDE ein Programm, ohne
        // dass die Exe im PATH liegen muss.
        let exe = if programm.eq_ignore_ascii_case("dhrt") {
            std::env::current_exe().map_err(|e| format!("PROCESS_START: eigener Pfad unbekannt: {}", e))?
                .to_string_lossy().into_owned()
        } else { programm.to_string() };
        let mut cmd = Command::new(&exe);
        cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        for v in ["DHRT_FRAMES", "DHRT_SCREENSHOT", "DHRT_CONTACT", "DHRT_CONTACT_MAX", "DHRT_CONTACT_COLS", "DHRT_CONTACT_EVERY"] {
            cmd.env_remove(v);
        }
        // Ein dhrt-Kind soll zeilenweise hinausschreiben, nicht blockweise:
        // an einer Leitung kaeme seine Ausgabe sonst erst am Ende (vm.rs,
        // `live_ausgabe`).
        cmd.env("DHRT_LIVE", "1");
        for (k, v) in umgebung { cmd.env(k, v); }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: kein Konsolenfenster hinter der IDE
        }
        #[cfg(all(unix, not(target_os = "emscripten")))]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0); // eigene Gruppe: PROCESS_KILL trifft auch die Enkel
        }
        let mut kind = cmd.spawn().map_err(|e| format!("PROCESS_START: '{}' laesst sich nicht starten: {}", programm, e))?;
        let stdout = Arc::new(Mutex::new(String::new()));
        let stderr = Arc::new(Mutex::new(String::new()));
        if let Some(o) = kind.stdout.take() { sammler(o, Arc::clone(&stdout)); }
        if let Some(e) = kind.stderr.take() { sammler(e, Arc::clone(&stderr)); }
        let stdin = kind.stdin.take();
        Ok(Prozess { kind, stdin, stdout, stderr, code: None, was: programm.to_string() })
    }

    /// Neue Standardausgabe seit dem letzten Abruf.
    pub fn lesen(&self) -> String {
        self.stdout.lock().map(|mut s| std::mem::take(&mut *s)).unwrap_or_default()
    }

    pub fn fehler_lesen(&self) -> String {
        self.stderr.lock().map(|mut s| std::mem::take(&mut *s)).unwrap_or_default()
    }

    pub fn schreiben(&mut self, text: &str) -> Result<(), String> {
        let Some(ein) = self.stdin.as_mut() else {
            return Err(format!("PROCESS_WRITE: die Eingabe von '{}' ist geschlossen", self.was));
        };
        ein.write_all(text.as_bytes()).and_then(|_| ein.flush())
            .map_err(|e| format!("PROCESS_WRITE: '{}' nimmt nichts mehr an: {}", self.was, e))
    }

    /// Eingabe schliessen -- ein Programm, das bis zum Ende liest, sieht so
    /// sein Dateiende.
    pub fn eingabe_schliessen(&mut self) { self.stdin = None; }

    /// Laeuft es noch? Merkt sich den Rueckgabewert, sobald es endet.
    pub fn laeuft(&mut self) -> bool {
        if self.code.is_some() { return false; }
        match self.kind.try_wait() {
            Ok(Some(status)) => { self.code = Some(status.code().map(|c| c as i64).unwrap_or(-1)); false }
            Ok(None) => true,
            Err(_) => { self.code = Some(-1); false }
        }
    }

    /// Rueckgabewert; -1, solange das Programm laeuft (oder ohne Wert endete).
    pub fn code(&mut self) -> i64 {
        self.laeuft();
        self.code.unwrap_or(-1)
    }

    pub fn beenden(&mut self) {
        if self.code.is_none() {
            if self.laeuft() { baum_beenden(self.kind.id()); }
            let _ = self.kind.kill();
            let _ = self.kind.wait();
            self.code = Some(-1);
        }
    }
}

impl Prozess {
    /// PROCESS_FRONT: das sichtbare Fenster des Kindes nach vorn holen. Ein
    /// Programm im Hintergrund darf sich unter Windows nicht selbst nach
    /// vorn stellen (die Taskleiste blinkt nur) -- das darf nur, wer gerade
    /// vorn ist. Darum holt der Aufrufer es: die IDE nach "Weiter" im
    /// Debugger, sonst laege das Programm hinter ihr. TRUE, wenn das Kind
    /// ein sichtbares Fenster hat (ob Windows es nach vorn laesst, sagt es
    /// nicht), FALSE ohne Fenster oder wenn es nicht mehr laeuft.
    pub fn nach_vorn(&mut self) -> bool {
        if !self.laeuft() { return false; }
        fenster_nach_vorn(self.kind.id())
    }
}

#[cfg(windows)]
fn fenster_nach_vorn(pid: u32) -> bool {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        AllowSetForegroundWindow, EnumWindows, GetWindowThreadProcessId, IsIconic,
        IsWindowVisible, SetForegroundWindow, ShowWindow, BringWindowToTop, SW_RESTORE,
        GetWindow, GW_OWNER,
    };
    unsafe extern "system" fn suchen(h: HWND, l: LPARAM) -> windows::core::BOOL {
        unsafe {
            let ziel = &mut *(l.0 as *mut (u32, HWND));
            let mut pid = 0u32;
            GetWindowThreadProcessId(h, Some(&mut pid));
            let ohne_besitzer = GetWindow(h, GW_OWNER).map(|o| o.0.is_null()).unwrap_or(true);
            if pid == ziel.0 && IsWindowVisible(h).as_bool() && ohne_besitzer {
                ziel.1 = h;
                return false.into();
            }
            true.into()
        }
    }
    let mut ziel: (u32, HWND) = (pid, HWND(std::ptr::null_mut()));
    unsafe {
        // Die Erlaubnis zuerst: hat das Kind noch kein Fenster (es oeffnet
        // es gleich), darf sein erstes dann vorn aufgehen.
        let _ = AllowSetForegroundWindow(pid);
        let _ = EnumWindows(Some(suchen), LPARAM(&mut ziel as *mut _ as isize));
        if ziel.1.0.is_null() { return false; }
        if IsIconic(ziel.1).as_bool() { let _ = ShowWindow(ziel.1, SW_RESTORE); }
        let _ = BringWindowToTop(ziel.1);
        // Ob Windows es zulaesst, entscheidet es selbst (der Aufrufer muss
        // vorn sein); die Antwort sagt nur, ob es ein Fenster gab.
        let _ = SetForegroundWindow(ziel.1);
        true
    }
}

#[cfg(not(windows))]
fn fenster_nach_vorn(_pid: u32) -> bool { false }

/// Beim Halt im Debugger: wer jetzt vorn ist (das Programm), gibt den
/// Vordergrund frei, damit die IDE sich nach vorn holen darf.
#[cfg(windows)]
pub fn vordergrund_freigeben() {
    use windows::Win32::UI::WindowsAndMessaging::{AllowSetForegroundWindow, ASFW_ANY};
    unsafe { let _ = AllowSetForegroundWindow(ASFW_ANY); }
}

#[cfg(not(windows))]
pub fn vordergrund_freigeben() {}

/// Die Nachkommen eines Kindes beenden (das Kind selbst gleich mit).
#[cfg(windows)]
fn baum_beenden(pid: u32) {
    use std::os::windows::process::CommandExt;
    let _ = Command::new("taskkill")
        .args(["/T", "/F", "/PID", &pid.to_string()])
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .creation_flags(0x0800_0000)
        .status();
}

#[cfg(all(unix, not(target_os = "emscripten")))]
fn baum_beenden(pid: u32) {
    // Die Gruppe traegt die Nummer des Kindes (process_group(0) beim Start).
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{}", pid)])
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .status();
}

#[cfg(not(any(windows, all(unix, not(target_os = "emscripten")))))]
fn baum_beenden(_pid: u32) {}

impl Drop for Prozess {
    fn drop(&mut self) { self.beenden(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_laeuft_und_liefert_ausgabe() {
        // Ein Programm, das es auf jedem System gibt (im Testlauf ist "dhrt"
        // die Test-Exe, nicht die Runtime -- die prueft tests/pruef/ide_bausteine.dhtest).
        let (prog, args): (&str, Vec<String>) = if cfg!(windows) {
            ("cmd", vec!["/c".into(), "echo hallo".into()])
        } else {
            ("sh", vec!["-c".into(), "echo hallo".into()])
        };
        let mut p = Prozess::starten(prog, &args).unwrap();
        let start = std::time::Instant::now();
        while p.laeuft() && start.elapsed().as_secs() < 20 { std::thread::sleep(std::time::Duration::from_millis(10)); }
        std::thread::sleep(std::time::Duration::from_millis(50));
        let aus = p.lesen();
        assert!(aus.contains("hallo"), "{:?}", aus);
        assert_eq!(p.code(), 0);
        assert!(p.lesen().is_empty(), "zweiter Abruf liefert nichts Neues");
    }

    #[test]
    #[cfg(windows)]
    fn kill_beendet_auch_die_enkel() {
        // cmd startet ping als Enkel; ohne den Baum liefe er 20 s weiter und
        // hielte die Leitung offen -- das Lesen bis zum Ende dauerte so lange.
        let mut p = Prozess::starten("cmd", &["/C".into(), "ping -n 20 127.0.0.1".into()]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        let t = std::time::Instant::now();
        p.beenden();
        // Enkel weg: nach kurzer Zeit gibt es keinen ping mit dieser Leitung
        // mehr -- geprueft an der Ausgabe, die sonst weiterliefe.
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let vorher = p.lesen();
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let nachher = p.lesen();
        assert!(t.elapsed().as_secs() < 10);
        assert!(nachher.is_empty(), "ping lief weiter: vorher {:?}, nachher {:?}", vorher, nachher);
    }

    #[test]
    fn unbekanntes_programm_meldet_sich_beim_start() {
        let e = Prozess::starten("gibt_es_nicht_xyz", &[]).err().unwrap();
        assert!(e.contains("laesst sich nicht starten"));
    }
}
