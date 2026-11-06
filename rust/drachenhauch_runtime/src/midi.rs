//! Modul `midi` -- Noten von einem angeschlossenen Instrument lesen und
//! welche hinausschicken. Feature `midi` (via `midir`: WinMM / ALSA / CoreMIDI).
//!
//! Warum es das gibt: Drachenhauch bringt Tracker, Sampler, sfxr-Synth, einen
//! Notenblatt-Editor und Kira-Busse mit -- und bis hierher konnte kein
//! angeschlossenes Keyboard etwas davon ansteuern. Genau diese eine Bruecke
//! fehlte.
//!
//! `MIDI_IN`/`MIDI_OUT` sind INTEGER-Handles (Index in einer VM-Vec), wie bei
//! `serial`. Eingehendes laeuft ueber das Cursor-Muster von `db`/`mqtt`:
//! `MIDI_NEXT(h)` holt die naechste Nachricht in einen Zwischenspeicher, die
//! `MIDI_*`-Abfragen lesen daraus.
//! **Nicht alles hier haengt am Feature.** `MIDI_NOTE_NAME$` und
//! `MIDI_NOTE_FREQ` rechnen nur um und stehen in JEDEM Bau -- wer eine
//! Notenanzeige oder einen Ton aus einer Notennummer baut, braucht dafuer
//! kein Geraet. Nur der Geraeteteil liegt hinter `feature = "midi"`.

use std::collections::VecDeque;
#[cfg(feature = "midi")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "midi")]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(feature = "midi")]
use std::time::{Duration, Instant};

#[cfg(feature = "midi")]
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

/// Wie viele unabgeholte Nachrichten hoechstens warten.
///
/// Ohne Deckel waechst der Puffer eines Programms, das `MIDI_NEXT` nicht (oder
/// zu selten) ruft, unbegrenzt -- ein haltendes Pedal schickt Dutzende
/// Nachrichten je Sekunde. Beim Ueberlauf faellt die AELTESTE weg: wer live
/// spielt, will den aktuellen Anschlag sehen, nicht den von vor zehn Sekunden.
#[cfg(feature = "midi")]
const MAX_WARTEND: usize = 1024;

#[cfg(feature = "midi")]
type Warteschlange = Arc<Mutex<VecDeque<Vec<u8>>>>;

#[cfg(feature = "midi")]
pub struct Eingang {
    /// Die Verbindung MUSS am Leben bleiben -- wird sie fallengelassen,
    /// schliesst midir den Port und der Rueckruf feuert nicht mehr.
    _conn: MidiInputConnection<()>,
    warteschlange: Warteschlange,
    /// Die zuletzt von `MIDI_NEXT` geholte Nachricht.
    aktuell: Option<Vec<u8>>,
    /// Was die MIDI-Uhr der Gegenseite sagt (Tempo, laeuft, Position).
    takt: Arc<Mutex<Takt>>,
    beginn: Instant,
}

/// Ein Ausgang. Die Verbindung liegt hinter einer Sperre, weil neben dem
/// Programm zwei Faeden senden koennen: die Uhr (MIDI_CLOCK_SEND) und das
/// Abspielen einer Datei (MIDI_FILE_PLAY).
#[cfg(feature = "midi")]
pub struct Ausgang {
    conn: Arc<Mutex<MidiOutputConnection>>,
    uhr: Option<Faden>,
    uhr_bpm: Arc<AtomicU64>,
    spieler: Option<Faden>,
    /// Sekunden seit Beginn der Datei (f64 als Bits), vom Spieler gesetzt.
    spiel_stand: Arc<AtomicU64>,
}

#[cfg(feature = "midi")]
struct Faden {
    halt: Arc<AtomicBool>,
    fertig: Arc<AtomicBool>,
    griff: Option<std::thread::JoinHandle<()>>,
}

#[cfg(feature = "midi")]
impl Faden {
    fn anhalten(&mut self) {
        self.halt.store(true, Ordering::SeqCst);
        if let Some(g) = self.griff.take() { let _ = g.join(); }
    }
}

#[cfg(feature = "midi")]
impl Drop for Ausgang {
    fn drop(&mut self) {
        if let Some(mut f) = self.uhr.take() { f.anhalten(); }
        if let Some(mut f) = self.spieler.take() { f.anhalten(); }
    }
}

#[cfg(feature = "midi")]
fn eingang_roh() -> Result<MidiInput, String> {
    MidiInput::new("Drachenhauch").map_err(|e| format!("MIDI: {}", e))
}
#[cfg(feature = "midi")]
fn ausgang_roh() -> Result<MidiOutput, String> {
    MidiOutput::new("Drachenhauch").map_err(|e| format!("MIDI: {}", e))
}

#[cfg(feature = "midi")]
pub fn in_count() -> Result<i64, String> {
    Ok(eingang_roh()?.ports().len() as i64)
}
#[cfg(feature = "midi")]
pub fn out_count() -> Result<i64, String> {
    Ok(ausgang_roh()?.ports().len() as i64)
}

#[cfg(feature = "midi")]
pub fn in_name(idx: i64) -> Result<String, String> {
    let m = eingang_roh()?;
    let ports = m.ports();
    let p = ports.get(idx.max(0) as usize)
        .ok_or_else(|| format!("MIDI_IN_NAME$: es gibt keinen Eingang {} (vorhanden: {})",
                               idx, ports.len()))?;
    m.port_name(p).map_err(|e| format!("MIDI_IN_NAME$: {}", e))
}

#[cfg(feature = "midi")]
pub fn out_name(idx: i64) -> Result<String, String> {
    let m = ausgang_roh()?;
    let ports = m.ports();
    let p = ports.get(idx.max(0) as usize)
        .ok_or_else(|| format!("MIDI_OUT_NAME$: es gibt keinen Ausgang {} (vorhanden: {})",
                               idx, ports.len()))?;
    m.port_name(p).map_err(|e| format!("MIDI_OUT_NAME$: {}", e))
}

#[cfg(feature = "midi")]
pub fn in_open(idx: i64) -> Result<Eingang, String> {
    let mut m = eingang_roh()?;
    // SysEx weglassen. Uhr (F8), Zeitcode (F1) und Active Sensing (FE)
    // kommen an, gehen aber nicht in die Warteschlange, sondern nur in den
    // Takt -- ein Keyboard schickt davon Dutzende je Sekunde, und die
    // Warteschlange waere voll, bevor der erste Anschlag ankommt. (Bis
    // 2026.42 liess midir sie ganz weg; Start, Stopp und Songposition kamen
    // damals schon an und kommen weiter an.)
    m.ignore(Ignore::Sysex);
    let ports = m.ports();
    let p = ports.get(idx.max(0) as usize)
        .ok_or_else(|| format!("MIDI_IN_OPEN: es gibt keinen Eingang {} (vorhanden: {})",
                               idx, ports.len()))?
        .clone();
    let warteschlange: Warteschlange = Arc::new(Mutex::new(VecDeque::new()));
    let ziel = warteschlange.clone();
    let takt = Arc::new(Mutex::new(Takt::default()));
    let takt2 = takt.clone();
    let beginn = Instant::now();
    let conn = m.connect(&p, "drachenhauch-in", move |_zeit, bytes, _| {
        // Die eigene Uhr statt midirs Zeitstempel: die zaehlt ab einem
        // Anfang, den man nicht kennt, und die Abfragen brauchen "jetzt".
        let jetzt = beginn.elapsed().as_micros() as u64;
        let nur_takt = takt2.lock().map(|mut t| t.nachricht(bytes, jetzt)).unwrap_or(false);
        if nur_takt { return; }
        if let Ok(mut q) = ziel.lock() {
            if q.len() >= MAX_WARTEND { q.pop_front(); }
            q.push_back(bytes.to_vec());
        }
    }, ()).map_err(|e| format!("MIDI_IN_OPEN: {}", e))?;
    Ok(Eingang { _conn: conn, warteschlange, aktuell: None, takt, beginn })
}

#[cfg(feature = "midi")]
pub fn out_open(idx: i64) -> Result<Ausgang, String> {
    let m = ausgang_roh()?;
    let ports = m.ports();
    let p = ports.get(idx.max(0) as usize)
        .ok_or_else(|| format!("MIDI_OUT_OPEN: es gibt keinen Ausgang {} (vorhanden: {})",
                               idx, ports.len()))?
        .clone();
    let conn = m.connect(&p, "drachenhauch-out")
        .map_err(|e| format!("MIDI_OUT_OPEN: {}", e))?;
    Ok(Ausgang {
        conn: Arc::new(Mutex::new(conn)),
        uhr: None,
        uhr_bpm: Arc::new(AtomicU64::new(0)),
        spieler: None,
        spiel_stand: Arc::new(AtomicU64::new(0)),
    })
}

/// Naechste Nachricht in den Zwischenspeicher holen. `false` = nichts da.
#[cfg(feature = "midi")]
pub fn next(e: &mut Eingang) -> bool {
    let naechste = e.warteschlange.lock().ok().and_then(|mut q| q.pop_front());
    match naechste {
        Some(n) => { e.aktuell = Some(n); true }
        None => { e.aktuell = None; false }
    }
}

#[cfg(feature = "midi")]
pub fn wartend(e: &Eingang) -> i64 {
    e.warteschlange.lock().map(|q| q.len() as i64).unwrap_or(0)
}

// --------------------------------------------- Nachrichten entschluesseln
//
// BEWUSST ueber rohe Bytes statt ueber `Eingang`: an der Entwicklungsmaschine
// haengt kein MIDI-Geraet, und am Zielrechner auch keins. Haenge die
// Entschluesselung am Geraetetyp, ist sie NIRGENDS pruefbar -- so ist sie es
// ueberall, mit erfundenen Nachrichten (die Rust-`#[test]`s unten). Uebrig
// bleibt eine Transportschicht, die nur weiterreicht.
//
// Eine LEERE Nachricht (noch kein MIDI_NEXT, oder es kam nichts) liefert
// ueberall 0 bzw. FALSE -- auch beim Kanal, wo `0 + 1 = 1` sonst einen Kanal
// vortaeuschen wuerde, den es nicht gibt.

/// Statusbyte OHNE Kanal (0x90 = Note an, 0x80 = Note aus, 0xB0 = Regler ...).
pub fn status_von(m: &[u8]) -> i64 {
    match m.first() { Some(b) => (*b as i64) & 0xF0, None => 0 }
}

/// Kanal 1..16 -- im Protokoll stehen 0..15. Nach aussen zaehlen wir wie
/// jedes Geraetedisplay.
pub fn kanal_von(m: &[u8]) -> i64 {
    match m.first() { Some(b) => ((*b as i64) & 0x0F) + 1, None => 0 }
}

pub fn d1_von(m: &[u8]) -> i64 { m.get(1).map(|b| *b as i64).unwrap_or(0) }
pub fn d2_von(m: &[u8]) -> i64 { m.get(2).map(|b| *b as i64).unwrap_or(0) }

/// **Der Fallstrick des Protokolls:** die meisten Instrumente schicken kein
/// 0x80, sondern ein Note-AN mit Anschlagstaerke 0. Wer nur auf 0x80 prueft,
/// bekommt Toene, die nie aufhoeren.
pub fn ist_note_an(m: &[u8]) -> bool { status_von(m) == 0x90 && d2_von(m) > 0 }
pub fn ist_note_aus(m: &[u8]) -> bool {
    status_von(m) == 0x80 || (status_von(m) == 0x90 && d2_von(m) == 0)
}
pub fn ist_regler(m: &[u8]) -> bool { status_von(m) == 0xB0 }

// Die duenne Schicht darueber: aus dem Eingang die zuletzt geholte Nachricht
// nehmen und oben hineingeben. Mehr passiert hier nicht.
#[cfg(feature = "midi")]
fn nachricht(e: &Eingang) -> &[u8] {
    e.aktuell.as_deref().unwrap_or(&[])
}
#[cfg(feature = "midi")]
pub fn status(e: &Eingang) -> i64 { status_von(nachricht(e)) }
#[cfg(feature = "midi")]
pub fn channel(e: &Eingang) -> i64 { kanal_von(nachricht(e)) }
#[cfg(feature = "midi")]
pub fn data1(e: &Eingang) -> i64 { d1_von(nachricht(e)) }
#[cfg(feature = "midi")]
pub fn data2(e: &Eingang) -> i64 { d2_von(nachricht(e)) }
#[cfg(feature = "midi")]
pub fn is_note_on(e: &Eingang) -> bool { ist_note_an(nachricht(e)) }
#[cfg(feature = "midi")]
pub fn is_note_off(e: &Eingang) -> bool { ist_note_aus(nachricht(e)) }
#[cfg(feature = "midi")]
pub fn is_cc(e: &Eingang) -> bool { ist_regler(nachricht(e)) }

#[cfg(feature = "midi")]
pub fn send(a: &mut Ausgang, bytes: &[u8]) -> Result<(), String> {
    senden_ueber(&a.conn, bytes).map_err(|e| format!("MIDI_SEND: {}", e))
}

#[cfg(feature = "midi")]
fn senden_ueber(c: &Arc<Mutex<MidiOutputConnection>>, bytes: &[u8]) -> Result<(), String> {
    let mut g = c.lock().map_err(|_| "Sperre vergiftet".to_string())?;
    g.send(bytes).map_err(|e| e.to_string())
}

// --------------------------------------------------------------- die Uhr

#[cfg(feature = "midi")]
pub fn takt_bpm(e: &Eingang) -> f64 {
    let jetzt = e.beginn.elapsed().as_micros() as u64;
    e.takt.lock().map(|t| t.bpm(jetzt)).unwrap_or(0.0)
}
#[cfg(feature = "midi")]
pub fn takt_laeuft(e: &Eingang) -> bool { e.takt.lock().map(|t| t.laeuft).unwrap_or(false) }
#[cfg(feature = "midi")]
pub fn takt_ticks(e: &Eingang) -> i64 { e.takt.lock().map(|t| t.ticks).unwrap_or(0) }

/// Die Uhr hinausschicken: 24 Takte (F8) je Viertel, in einem eigenen Faden.
/// `bpm` <= 0 haelt sie an. Laeuft sie schon, aendert sich nur das Tempo --
/// ohne Neustart, sonst gaebe es an der Stelle einen Ruck.
#[cfg(feature = "midi")]
pub fn uhr_senden(a: &mut Ausgang, bpm: f64) -> Result<(), String> {
    if !bpm.is_finite() || bpm > 999.0 {
        return Err(format!("MIDI_CLOCK_SEND: Tempo {} ausserhalb 1..999 (0 haelt die Uhr an)", bpm));
    }
    if bpm <= 0.0 {
        if let Some(mut f) = a.uhr.take() { f.anhalten(); }
        return Ok(());
    }
    if bpm < 1.0 {
        return Err(format!("MIDI_CLOCK_SEND: Tempo {} ausserhalb 1..999 (0 haelt die Uhr an)", bpm));
    }
    a.uhr_bpm.store(bpm.to_bits(), Ordering::SeqCst);
    if a.uhr.is_some() { return Ok(()); }
    let halt = Arc::new(AtomicBool::new(false));
    let fertig = Arc::new(AtomicBool::new(false));
    let (h, c, t) = (halt.clone(), a.conn.clone(), a.uhr_bpm.clone());
    let griff = std::thread::Builder::new().name("drachenhauch-midi-uhr".into()).spawn(move || {
        takte_schlagen(|| { let _ = senden_ueber(&c, &[0xF8]); },
                       || f64::from_bits(t.load(Ordering::SeqCst)), &h);
    }).map_err(|e| format!("MIDI_CLOCK_SEND: {}", e))?;
    a.uhr = Some(Faden { halt, fertig, griff: Some(griff) });
    Ok(())
}

#[cfg(feature = "midi")]
pub fn uhr_laeuft(a: &Ausgang) -> bool { a.uhr.is_some() }

/// Der Kern der Uhr: in gleichen Abstaenden `schlagen` rufen, bis `halt`.
/// Gerechnet wird von einem festen Anfang aus (Schlag n faellt auf
/// n * Abstand), nicht von Schlag zu Schlag -- sonst summierte sich jede
/// Verspaetung auf. Faellt der Faden mehr als einen Abstand zurueck (der
/// Rechner stand), beginnt die Zaehlung neu, statt die Luecke im Eiltempo
/// nachzuholen.
#[cfg(feature = "midi")]
pub fn takte_schlagen(mut schlagen: impl FnMut(), bpm: impl Fn() -> f64, halt: &AtomicBool) {
    let mut anfang = Instant::now();
    let mut zeit = 0.0f64; // Sekunden ab `anfang` fuer den naechsten Schlag
    while !halt.load(Ordering::SeqCst) {
        let b = bpm();
        if b <= 0.0 { std::thread::sleep(Duration::from_millis(5)); continue; }
        let abstand = 60.0 / (b * 24.0);
        let jetzt = anfang.elapsed().as_secs_f64();
        if jetzt + 0.0005 >= zeit {
            schlagen();
            zeit += abstand;
            if jetzt > zeit + abstand {
                anfang = Instant::now();
                zeit = abstand;
            }
            continue;
        }
        let rest = zeit - jetzt;
        // Grob schlafen, die letzte Millisekunde nachsehen.
        if rest > 0.002 { std::thread::sleep(Duration::from_secs_f64(rest - 0.0015)); }
        else { std::thread::yield_now(); }
    }
}

// ---------------------------------------------------- Datei abspielen

/// Die Nachrichten einer Datei in ihrer Zeit hinausschicken. Haelt man an
/// (oder schliesst den Ausgang), gehen die noch klingenden Noten aus --
/// sonst hinge am Synthesizer ein Ton, bis jemand den Stecker zieht.
#[cfg(feature = "midi")]
pub fn datei_spielen(a: &mut Ausgang, ablauf: Vec<(f64, Vec<u8>)>, ab: f64) -> Result<(), String> {
    if let Some(mut f) = a.spieler.take() { f.anhalten(); }
    let halt = Arc::new(AtomicBool::new(false));
    let fertig = Arc::new(AtomicBool::new(false));
    let (h, fe, c, stand) = (halt.clone(), fertig.clone(), a.conn.clone(), a.spiel_stand.clone());
    stand.store(ab.to_bits(), Ordering::SeqCst);
    let griff = std::thread::Builder::new().name("drachenhauch-midi-spieler".into()).spawn(move || {
        let anfang = Instant::now();
        let mut klingt: Vec<(u8, u8)> = Vec::new();
        for (t, m) in ablauf.into_iter().filter(|(t, _)| *t >= ab) {
            loop {
                if h.load(Ordering::SeqCst) { break; }
                let jetzt = ab + anfang.elapsed().as_secs_f64();
                stand.store(jetzt.to_bits(), Ordering::SeqCst);
                let rest = t - jetzt;
                if rest <= 0.0005 { break; }
                std::thread::sleep(Duration::from_secs_f64((rest - 0.0005).min(0.01)));
            }
            if h.load(Ordering::SeqCst) { break; }
            let art = m[0] & 0xF0;
            let kanal = m[0] & 0x0F;
            if art == 0x90 && m.get(2).copied().unwrap_or(0) > 0 {
                klingt.push((kanal, m[1]));
            } else if art == 0x80 || art == 0x90 {
                if let Some(k) = klingt.iter().position(|x| *x == (kanal, m[1])) { klingt.remove(k); }
            }
            let _ = senden_ueber(&c, &m);
        }
        for (k, n) in klingt { let _ = senden_ueber(&c, &[0x80 | k, n, 0]); }
        fe.store(true, Ordering::SeqCst);
    }).map_err(|e| format!("MIDI_FILE_PLAY: {}", e))?;
    a.spieler = Some(Faden { halt, fertig, griff: Some(griff) });
    Ok(())
}

#[cfg(feature = "midi")]
pub fn datei_halten(a: &mut Ausgang) {
    if let Some(mut f) = a.spieler.take() { f.anhalten(); }
}

#[cfg(feature = "midi")]
pub fn datei_spielt(a: &Ausgang) -> bool {
    a.spieler.as_ref().map(|f| !f.fertig.load(Ordering::SeqCst)).unwrap_or(false)
}

#[cfg(feature = "midi")]
pub fn datei_stand(a: &Ausgang) -> f64 { f64::from_bits(a.spiel_stand.load(Ordering::SeqCst)) }

/// 0..127 pruefen -- ein groesserer Wert setzte im Protokoll das Statusbit und
/// wuerde als voellig andere Nachricht gelesen.
pub fn sieben_bit(wert: i64, was: &str, fn_: &str) -> Result<u8, String> {
    if !(0..=127).contains(&wert) {
        return Err(format!("{}: {} muss zwischen 0 und 127 liegen, war {}", fn_, was, wert));
    }
    Ok(wert as u8)
}

pub fn kanal_byte(kanal: i64, fn_: &str) -> Result<u8, String> {
    if !(1..=16).contains(&kanal) {
        return Err(format!("{}: Kanal muss zwischen 1 und 16 liegen, war {}", fn_, kanal));
    }
    Ok((kanal - 1) as u8)
}

// ------------------------------------------------------------ reine Rechnerei

/// Was die MIDI-Uhr einer Gegenseite verraet: das Tempo (aus den Abstaenden
/// der Uhrtakte F8, 24 je Viertel), ob sie laeuft (Start FA/Weiter FB bis
/// Stopp FC) und wo sie steht (Takte seit Start; Songposition F2 setzt sie).
#[derive(Default)]
pub struct Takt {
    pub laeuft: bool,
    pub ticks: i64,
    abstaende: VecDeque<u64>,
    letzter: Option<u64>,
}

impl Takt {
    /// Eine Nachricht verarbeiten. `true` = sie gehoert NUR der Uhr und geht
    /// nicht in die Warteschlange (Uhrtakt, Zeitcode, Active Sensing).
    pub fn nachricht(&mut self, b: &[u8], jetzt_us: u64) -> bool {
        match b.first().copied() {
            Some(0xF8) => {
                if let Some(l) = self.letzter {
                    let d = jetzt_us.saturating_sub(l);
                    // Eine Pause ueber einer Sekunde: die Uhr stand -- das
                    // Mittel neu beginnen, sonst zoege die Luecke es herunter.
                    if d > 1_000_000 { self.abstaende.clear(); }
                    else if d > 0 {
                        self.abstaende.push_back(d);
                        if self.abstaende.len() > 24 { self.abstaende.pop_front(); }
                    }
                }
                self.letzter = Some(jetzt_us);
                if self.laeuft { self.ticks += 1; }
                true
            }
            Some(0xFA) => { self.laeuft = true; self.ticks = 0; false }
            Some(0xFB) => { self.laeuft = true; false }
            Some(0xFC) => { self.laeuft = false; false }
            // Songposition in Sechzehnteln, 14 Bit; je Sechzehntel 6 Takte.
            Some(0xF2) => {
                let pos = (b.get(1).copied().unwrap_or(0) as i64) | ((b.get(2).copied().unwrap_or(0) as i64) << 7);
                self.ticks = pos * 6;
                false
            }
            Some(0xF1) | Some(0xFE) => true,
            _ => false,
        }
    }

    /// Schlaege je Minute; 0, wenn seit einer halben Sekunde kein Takt kam
    /// (die Uhr der Gegenseite steht oder es gibt keine).
    pub fn bpm(&self, jetzt_us: u64) -> f64 {
        match self.letzter {
            Some(l) if jetzt_us.saturating_sub(l) <= 500_000 && self.abstaende.len() >= 2 => {
                let mittel = self.abstaende.iter().sum::<u64>() as f64 / self.abstaende.len() as f64;
                60_000_000.0 / (mittel * 24.0)
            }
            _ => 0.0,
        }
    }
}
//
// Die beiden folgenden brauchen KEIN Geraet -- sie rechnen nur um. Damit ist
// der nuetzlichste Teil des Moduls auch auf einer Maschine ohne MIDI-Anschluss
// pruefbar (und testbar).

/// Notennummer -> Name, z.B. 60 -> "C4".
///
/// Die Oktavzaehlung folgt der verbreiteten Konvention (Yamaha/MIDI-Standard):
/// Note 60 ist C4. Manche Hersteller nennen dieselbe Note C3 -- das ist eine
/// Zaehlweise, kein Fehler.
pub fn note_name(note: i64) -> String {
    const NAMEN: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "H"];
    if !(0..=127).contains(&note) { return String::new(); }
    let oktave = note / 12 - 1;
    format!("{}{}", NAMEN[(note % 12) as usize], oktave)
}

/// Notennummer -> Frequenz in Hertz (A4 = Note 69 = 440 Hz).
/// Gedacht als Bruecke zu `AUDIO_TONE`.
pub fn note_freq(note: i64) -> f64 {
    440.0 * 2f64.powf((note as f64 - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notennamen_folgen_der_konvention() {
        assert_eq!(note_name(60), "C4");
        assert_eq!(note_name(69), "A4");
        assert_eq!(note_name(0), "C-1");
        assert_eq!(note_name(127), "G9");
        // Im deutschen Sprachraum heisst H, was anderswo B heisst.
        assert_eq!(note_name(71), "H4");
        // Ausserhalb des Protokolls: leer statt raten.
        assert_eq!(note_name(-1), "");
        assert_eq!(note_name(128), "");
    }

    #[test]
    fn frequenzen_treffen_den_kammerton() {
        assert!((note_freq(69) - 440.0).abs() < 1e-9);
        // Eine Oktave hoeher ist genau das Doppelte.
        assert!((note_freq(81) - 880.0).abs() < 1e-9);
        assert!((note_freq(57) - 220.0).abs() < 1e-9);
    }

    // --- Nachrichten entschluesseln (erfundene Bytes, kein Geraet noetig) ---

    #[test]
    fn note_an_wird_gelesen() {
        let m = [0x90u8, 60, 100];          // Kanal 1, C4, Anschlag 100
        assert!(ist_note_an(&m));
        assert!(!ist_note_aus(&m));
        assert!(!ist_regler(&m));
        assert_eq!(status_von(&m), 0x90);
        assert_eq!(kanal_von(&m), 1);
        assert_eq!(d1_von(&m), 60);
        assert_eq!(d2_von(&m), 100);
    }

    #[test]
    fn note_aus_kommt_in_zwei_formen() {
        // Die saubere Form, die kaum ein Instrument benutzt.
        let echt = [0x80u8, 60, 0];
        assert!(ist_note_aus(&echt));
        assert!(!ist_note_an(&echt));

        // Die uebliche: Note AN mit Anschlag 0. Wer nur auf 0x80 prueft,
        // bekommt Toene, die nie aufhoeren -- genau dafuer ist dieser Test da.
        let ueblich = [0x90u8, 60, 0];
        assert!(ist_note_aus(&ueblich));
        assert!(!ist_note_an(&ueblich));
    }

    #[test]
    fn kanaele_zaehlen_ab_eins() {
        assert_eq!(kanal_von(&[0x90, 60, 100]), 1);    // Protokoll 0
        assert_eq!(kanal_von(&[0x9F, 60, 100]), 16);   // Protokoll 15
        assert_eq!(kanal_von(&[0x85, 60, 0]), 6);      // auch bei Note-aus
        // Der Kanal darf den Status nicht verwaschen.
        assert_eq!(status_von(&[0x9F, 60, 100]), 0x90);
    }

    #[test]
    fn regler_wird_gelesen() {
        let m = [0xB0u8, 7, 90];           // Regler 7 (Lautstaerke) auf 90
        assert!(ist_regler(&m));
        assert!(!ist_note_an(&m));
        assert!(!ist_note_aus(&m));
        assert_eq!(d1_von(&m), 7);
        assert_eq!(d2_von(&m), 90);
    }

    #[test]
    fn leere_nachricht_taeuscht_nichts_vor() {
        // Vor dem ersten MIDI_NEXT, oder wenn nichts angekommen ist.
        let leer: [u8; 0] = [];
        assert_eq!(status_von(&leer), 0);
        assert_eq!(d1_von(&leer), 0);
        assert_eq!(d2_von(&leer), 0);
        assert!(!ist_note_an(&leer));
        assert!(!ist_note_aus(&leer));
        assert!(!ist_regler(&leer));
        // Der wichtigste der sechs: `0 & 0x0F + 1` waere 1 und haette einen
        // Kanal vorgetaeuscht, den es gar nicht gibt.
        assert_eq!(kanal_von(&leer), 0);
    }

    #[test]
    fn zu_kurze_nachricht_gilt_als_note_aus() {
        // Fehlt das dritte Byte, ist der Anschlag 0 -- und ein Note-AN mit
        // Anschlag 0 IST ein Note-aus. Lieber ein Ton, der endet, als einer,
        // der haengenbleibt.
        let kurz = [0x90u8, 60];
        assert_eq!(d2_von(&kurz), 0);
        assert!(ist_note_aus(&kurz));
        assert!(!ist_note_an(&kurz));
    }

    #[test]
    fn der_takt_liest_tempo_start_und_stopp() {
        let mut t = Takt::default();
        // 120 Schlaege je Minute = 24 Takte in 500 ms, alle 20833 us einer.
        assert!(!t.nachricht(&[0xFA], 0));
        for k in 0..48u64 { assert!(t.nachricht(&[0xF8], k * 20833)); }
        assert!((t.bpm(48 * 20833) - 120.0).abs() < 0.01, "{}", t.bpm(48 * 20833));
        assert!(t.laeuft);
        assert_eq!(t.ticks, 48);
        assert!(!t.nachricht(&[0xFC], 1_000_000));
        assert!(!t.laeuft);
        t.nachricht(&[0xF8], 1_010_000);
        assert_eq!(t.ticks, 48, "gestoppt zaehlt nicht weiter");
        // Eine halbe Sekunde still: kein Tempo mehr.
        assert_eq!(t.bpm(1_600_000), 0.0);
        // Songposition: 8 Sechzehntel = zwei Viertel = 48 Takte.
        t.nachricht(&[0xF2, 8, 0], 2_000_000);
        assert_eq!(t.ticks, 48);
        t.nachricht(&[0xF2, 0, 1], 2_000_000);
        assert_eq!(t.ticks, 128 * 6);
        assert!(t.nachricht(&[0xFE], 0) && t.nachricht(&[0xF1, 3], 0));
        assert!(!t.nachricht(&[0x90, 60, 100], 0));
    }

    #[test]
    fn eine_pause_beginnt_das_mittel_neu() {
        let mut t = Takt::default();
        for k in 0..30u64 { t.nachricht(&[0xF8], k * 10_000); }
        // Zwei Sekunden Pause, dann 60 Schlaege je Minute.
        let ab = 30 * 10_000 + 2_000_000;
        for k in 0..30u64 { t.nachricht(&[0xF8], ab + k * 41_667); }
        assert!((t.bpm(ab + 29 * 41_667) - 60.0).abs() < 0.01);
    }

    #[cfg(feature = "midi")]
    #[test]
    fn die_uhr_schlaegt_gleichmaessig() {
        use std::sync::atomic::AtomicBool;
        let halt = AtomicBool::new(false);
        let mut zeiten: Vec<Instant> = Vec::new();
        let anfang = Instant::now();
        takte_schlagen(|| {
            zeiten.push(Instant::now());
            if zeiten.len() >= 49 { halt.store(true, Ordering::SeqCst); }
        }, || 240.0, &halt);
        // 240 Schlaege je Minute: 48 Takte = zwei Viertel = 0,5 s.
        let d = zeiten[48].duration_since(zeiten[0]).as_secs_f64();
        assert!((d - 0.5).abs() < 0.03, "{}", d);
        assert!(anfang.elapsed().as_secs_f64() < 1.0);
    }

    #[test]
    fn wertebereiche_werden_geprueft() {
        assert!(sieben_bit(0, "Note", "X").is_ok());
        assert!(sieben_bit(127, "Note", "X").is_ok());
        assert!(sieben_bit(128, "Note", "X").is_err());
        assert!(sieben_bit(-1, "Note", "X").is_err());
        assert_eq!(kanal_byte(1, "X").unwrap(), 0);
        assert_eq!(kanal_byte(16, "X").unwrap(), 15);
        assert!(kanal_byte(0, "X").is_err());
        assert!(kanal_byte(17, "X").is_err());
    }
}
