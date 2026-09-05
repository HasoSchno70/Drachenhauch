//! Fenster als Prozess -- Weg B aus `docs/entwurf-native-fenster.md`.
//!
//! Drachenhauch kennt ein OS-Fenster je Prozess (raylib haelt seinen Zustand
//! global). Ein zweites Fenster ist deshalb ein zweiter `dhrt`, der sein
//! eigenes Programm mit eigenem `SCREEN` faehrt -- verbunden mit dem ersten
//! ueber einen **zeilenweisen Textkanal**. Kein geteilter Speicher: ein Bild,
//! ein Feld, ein Objekt gehoert einem Prozess; was hinueber soll, wird Text
//! (JSON, wenn es Struktur braucht). Dieselbe Grenze zieht `TASK_START`.
//!
//! **Der Kanal ist eine TCP-Verbindung auf 127.0.0.1**, nicht stdin/stdout des
//! Kindes: das Kind soll weiter PRINT benutzen duerfen, ohne dass seine
//! Ausgabe als Nachricht ankommt, und ein zusaetzliches geerbtes Handle waere
//! unter Windows und POSIX zweierlei. Die Eltern lauschen auf einem freien
//! Port, reichen ihn dem Kind in der Umgebung (`DHRT_ELTERN_PORT`), das Kind
//! verbindet sich beim ersten `PARENT_*`-Aufruf. Gemessen ist eine Runde
//! Eltern -> Kind -> Eltern auf dieser Strecke ein Bruchteil einer
//! Millisekunde (siehe tests/test_fenster_prozess.py).
//!
//! Zwei Regeln, die man nicht sieht:
//! - **Eine Nachricht ist eine Zeile.** Ein Zeilenumbruch darin ist ein Fehler,
//!   kein stilles Zerteilen.
//! - **Die Warteschlange ist begrenzt** (1024 Zeilen, die aelteste faellt weg)
//!   -- wer nicht liest, laesst die Gegenseite nicht haengen. Dasselbe Muster
//!   wie beim MIDI-Modul.
//! - **Faellt die Verbindung, beendet sich das Kind** (`std::process::exit`).
//!   Ein Fenster, dessen Hauptprogramm weg ist, waere sonst ein Zombie, den
//!   niemand mehr schliessen kann. `PARENT_ALIVE()` liefert vorher FALSE fuer
//!   ein Programm, das gar nicht als Kind gestartet wurde.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const ENV_PORT: &str = "DHRT_ELTERN_PORT";
const PUFFER: usize = 1024;
/// Wie lange die Eltern auf die Verbindung des Kindes warten -- ein Kind, das
/// nie `PARENT_*` ruft, verbindet sich nie; danach bleibt es ein Fenster ohne
/// Kanal, kein Fehler.
const VERBINDUNGSFRIST: Duration = Duration::from_secs(30);

/// Eine Verbindung, von beiden Seiten gleich benutzt: Leser-Thread fuellt
/// `eingang`, `senden` schreibt direkt oder merkt vor, bis verbunden ist.
struct Leitung {
    eingang: Mutex<VecDeque<String>>,
    schreiber: Mutex<Option<TcpStream>>,
    wartend: Mutex<Vec<String>>,
    offen: AtomicBool,
    /// Kindseite: beim Verbindungsende den Prozess beenden.
    beenden_bei_ende: bool,
}

impl Leitung {
    fn neu(beenden_bei_ende: bool) -> Arc<Leitung> {
        Arc::new(Leitung {
            eingang: Mutex::new(VecDeque::new()),
            schreiber: Mutex::new(None),
            wartend: Mutex::new(Vec::new()),
            offen: AtomicBool::new(false),
            beenden_bei_ende,
        })
    }

    fn senden(&self, text: &str, fn_: &str) -> Result<(), String> {
        if text.contains('\n') || text.contains('\r') {
            return Err(format!("{}: eine Nachricht ist EINE Zeile -- kein Zeilenumbruch darin (schick zwei Nachrichten oder JSON)", fn_));
        }
        let mut schreiber = self.schreiber.lock().unwrap();
        match schreiber.as_mut() {
            Some(s) => {
                if s.write_all(text.as_bytes()).and_then(|_| s.write_all(b"\n")).is_err() {
                    self.offen.store(false, Ordering::SeqCst);
                    return Err(format!("{}: die Gegenseite ist weg", fn_));
                }
                Ok(())
            }
            None => {
                // Noch nicht verbunden: vormerken, geht beim Verbinden raus.
                let mut w = self.wartend.lock().unwrap();
                if w.len() >= PUFFER { w.remove(0); }
                w.push(text.to_string());
                Ok(())
            }
        }
    }

    fn empfangen(&self) -> Option<String> {
        self.eingang.lock().unwrap().pop_front()
    }

    /// Verbindung uebernehmen: Vorgemerktes senden, Leser-Thread starten.
    fn verbinden(self: &Arc<Self>, stream: TcpStream) {
        let _ = stream.set_nodelay(true);
        let leser = match stream.try_clone() { Ok(s) => s, Err(_) => return };
        {
            let mut schreiber = self.schreiber.lock().unwrap();
            let mut s = stream;
            for z in self.wartend.lock().unwrap().drain(..) {
                let _ = s.write_all(z.as_bytes()).and_then(|_| s.write_all(b"\n"));
            }
            *schreiber = Some(s);
        }
        self.offen.store(true, Ordering::SeqCst);
        let me = Arc::clone(self);
        std::thread::spawn(move || {
            let mut r = BufReader::new(leser);
            let mut zeile = String::new();
            loop {
                zeile.clear();
                match r.read_line(&mut zeile) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let z = zeile.trim_end_matches(['\n', '\r']).to_string();
                        let mut e = me.eingang.lock().unwrap();
                        if e.len() >= PUFFER { e.pop_front(); }
                        e.push_back(z);
                    }
                }
            }
            me.offen.store(false, Ordering::SeqCst);
            if me.beenden_bei_ende {
                // Das Hauptprogramm ist weg: kein Fenster ohne Herrn.
                std::process::exit(0);
            }
        });
    }
}

// ---------------------------------------------------------------- Elternseite
pub struct Kind {
    prozess: std::process::Child,
    leitung: Arc<Leitung>,
}

#[derive(Default)]
pub struct Fenster {
    kinder: Vec<Option<Kind>>,
}

impl Fenster {
    /// WINDOW_OPEN: zweiten dhrt mit `datei` starten, Kanal vorbereiten.
    pub fn oeffnen(&mut self, exe: std::path::PathBuf, datei: &str, args: &[String]) -> Result<i64, String> {
        if !std::path::Path::new(datei).exists() {
            return Err(format!("WINDOW_OPEN: Datei '{}' nicht gefunden (Pfad relativ zum laufenden Programm)", datei));
        }
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("WINDOW_OPEN: kein freier Port auf 127.0.0.1: {}", e))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        let mut cmd = std::process::Command::new(&exe);
        cmd.arg("run").arg(datei);
        // Programmargumente stehen bei `dhrt run` hinter `--` (davor gehoeren
        // sie der Laufzeit) -- ohne den Trenner kaeme im Kind ARGC() = 0 an.
        if !args.is_empty() { cmd.arg("--"); }
        for a in args { cmd.arg(a); }
        cmd.env(ENV_PORT, port.to_string());
        // Die Kopfzeilen-Steuerung der Tests gilt fuer die Eltern, nicht fuer
        // das Kind -- ein Kind mit geerbtem DHRT_FRAMES stuerbe nach N Bildern.
        for v in ["DHRT_FRAMES", "DHRT_SCREENSHOT", "DHRT_CONTACT", "DHRT_CONTACT_MAX", "DHRT_CONTACT_COLS", "DHRT_CONTACT_EVERY"] {
            cmd.env_remove(v);
        }
        let prozess = cmd.spawn().map_err(|e| format!("WINDOW_OPEN: '{}' laesst sich nicht starten: {}", exe.display(), e))?;
        let leitung = Leitung::neu(false);
        // Auf die Verbindung warten -- in einem Thread, mit Frist, ohne den
        // Aufrufer aufzuhalten. Nicht blockierend, damit die Frist greift.
        let _ = listener.set_nonblocking(true);
        let l2 = Arc::clone(&leitung);
        std::thread::spawn(move || {
            let start = Instant::now();
            loop {
                match listener.accept() {
                    Ok((stream, _)) => { let _ = stream.set_nonblocking(false); l2.verbinden(stream); break; }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if start.elapsed() > VERBINDUNGSFRIST { break; }
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });
        self.kinder.push(Some(Kind { prozess, leitung }));
        Ok((self.kinder.len() - 1) as i64)
    }

    fn kind(&mut self, k: i64, fn_: &str) -> Result<&mut Kind, String> {
        if k < 0 { return Err(format!("{}: ungueltiges Fenster {}", fn_, k)); }
        self.kinder.get_mut(k as usize).and_then(|o| o.as_mut())
            .ok_or_else(|| format!("{}: Fenster {} gibt es nicht (mehr)", fn_, k))
    }

    pub fn senden(&mut self, k: i64, text: &str) -> Result<(), String> {
        self.kind(k, "WINDOW_SEND")?.leitung.senden(text, "WINDOW_SEND")
    }
    pub fn empfangen(&mut self, k: i64) -> Result<String, String> {
        Ok(self.kind(k, "WINDOW_RECV$")?.leitung.empfangen().unwrap_or_default())
    }
    /// Laeuft der Prozess noch? (Nicht: ist der Kanal verbunden.)
    pub fn lebt(&mut self, k: i64) -> Result<bool, String> {
        let kind = self.kind(k, "WINDOW_ALIVE")?;
        Ok(matches!(kind.prozess.try_wait(), Ok(None)))
    }
    pub fn schliessen(&mut self, k: i64) -> Result<(), String> {
        let kind = self.kind(k, "WINDOW_CLOSE")?;
        let _ = kind.prozess.kill();
        let _ = kind.prozess.wait();
        self.kinder[k as usize] = None;
        Ok(())
    }
}

impl Drop for Fenster {
    /// Die Eltern gehen: die Kinder auch. Die Verbindung schliesst sich
    /// ohnehin, das Kind beendet sich dann selbst -- kill ist der Gurt dazu.
    fn drop(&mut self) {
        for k in self.kinder.iter_mut().flatten() {
            let _ = k.prozess.kill();
            let _ = k.prozess.wait();
        }
    }
}

// ---------------------------------------------------------------- Kindseite
pub struct Eltern {
    leitung: Arc<Leitung>,
}

impl Eltern {
    /// Verbindet sich zu den Eltern, wenn dieses Programm als Kind gestartet
    /// wurde (Port in der Umgebung). Sonst None -- das Programm laeuft dann
    /// ganz normal allein.
    pub fn verbinden() -> Option<Eltern> {
        let port: u16 = std::env::var(ENV_PORT).ok()?.parse().ok()?;
        let stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
        let leitung = Leitung::neu(true);
        leitung.verbinden(stream);
        Some(Eltern { leitung })
    }
    pub fn senden(&self, text: &str) -> Result<(), String> { self.leitung.senden(text, "PARENT_SEND") }
    pub fn empfangen(&self) -> String { self.leitung.empfangen().unwrap_or_default() }
    pub fn lebt(&self) -> bool { self.leitung.offen.load(Ordering::SeqCst) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leitung_merkt_vor_und_liefert_in_reihenfolge() {
        let a = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = a.local_addr().unwrap().port();
        let eltern = Leitung::neu(false);
        eltern.senden("eins", "T").unwrap();          // vor der Verbindung
        let kind = Leitung::neu(false);
        kind.verbinden(TcpStream::connect(("127.0.0.1", port)).unwrap());
        let (s, _) = a.accept().unwrap();
        eltern.verbinden(s);
        eltern.senden("zwei", "T").unwrap();
        let mut got = Vec::new();
        let start = Instant::now();
        while got.len() < 2 && start.elapsed() < Duration::from_secs(2) {
            if let Some(z) = kind.empfangen() { got.push(z); } else { std::thread::sleep(Duration::from_millis(1)); }
        }
        assert_eq!(got, vec!["eins", "zwei"]);
        assert!(eltern.senden("a\nb", "T").is_err(), "eine Nachricht ist eine Zeile");
    }

    #[test]
    fn warteschlange_ist_begrenzt() {
        let a = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = a.local_addr().unwrap().port();
        let eltern = Leitung::neu(false);
        let kind = Leitung::neu(false);
        kind.verbinden(TcpStream::connect(("127.0.0.1", port)).unwrap());
        let (s, _) = a.accept().unwrap();
        eltern.verbinden(s);
        for i in 0..(PUFFER + 10) { eltern.senden(&i.to_string(), "T").unwrap(); }
        let start = Instant::now();
        while kind.eingang.lock().unwrap().len() < PUFFER && start.elapsed() < Duration::from_secs(3) {
            std::thread::sleep(Duration::from_millis(2));
        }
        std::thread::sleep(Duration::from_millis(50));
        let e = kind.eingang.lock().unwrap();
        assert_eq!(e.len(), PUFFER);
        assert_eq!(e.front().map(|s| s.as_str()), Some("10"), "die aeltesten fallen weg");
    }
}
