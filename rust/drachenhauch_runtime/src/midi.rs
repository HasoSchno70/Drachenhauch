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

#[cfg(feature = "midi")]
use std::collections::VecDeque;
#[cfg(feature = "midi")]
use std::sync::{Arc, Mutex};

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
}

#[cfg(feature = "midi")]
pub struct Ausgang {
    conn: MidiOutputConnection,
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
    // Uhr-, Active-Sensing- und SysEx-Nachrichten weglassen: ein Keyboard
    // schickt davon Dutzende je Sekunde, und keine davon ist eine Note. Sonst
    // waere die Warteschlange voll, bevor der erste Anschlag ankommt.
    m.ignore(Ignore::All);
    let ports = m.ports();
    let p = ports.get(idx.max(0) as usize)
        .ok_or_else(|| format!("MIDI_IN_OPEN: es gibt keinen Eingang {} (vorhanden: {})",
                               idx, ports.len()))?
        .clone();
    let warteschlange: Warteschlange = Arc::new(Mutex::new(VecDeque::new()));
    let ziel = warteschlange.clone();
    let conn = m.connect(&p, "drachenhauch-in", move |_zeit, bytes, _| {
        if let Ok(mut q) = ziel.lock() {
            if q.len() >= MAX_WARTEND { q.pop_front(); }
            q.push_back(bytes.to_vec());
        }
    }, ()).map_err(|e| format!("MIDI_IN_OPEN: {}", e))?;
    Ok(Eingang { _conn: conn, warteschlange, aktuell: None })
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
    Ok(Ausgang { conn })
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

#[cfg(feature = "midi")]
fn byte(e: &Eingang, i: usize) -> i64 {
    e.aktuell.as_ref().and_then(|m| m.get(i)).map(|b| *b as i64).unwrap_or(0)
}

/// Statusbyte OHNE Kanal (0x90 = Note an, 0x80 = Note aus, 0xB0 = Regler ...).
#[cfg(feature = "midi")]
pub fn status(e: &Eingang) -> i64 { byte(e, 0) & 0xF0 }
/// Kanal 1..16 (im Protokoll steckt 0..15).
#[cfg(feature = "midi")]
pub fn channel(e: &Eingang) -> i64 {
    if e.aktuell.is_none() { return 0; }
    (byte(e, 0) & 0x0F) + 1
}
#[cfg(feature = "midi")]
pub fn data1(e: &Eingang) -> i64 { byte(e, 1) }
#[cfg(feature = "midi")]
pub fn data2(e: &Eingang) -> i64 { byte(e, 2) }

/// **Der Fallstrick des Protokolls:** die meisten Instrumente schicken kein
/// 0x80, sondern ein Note-AN mit Anschlagstaerke 0. Wer nur auf 0x80 prueft,
/// bekommt Toene, die nie aufhoeren.
#[cfg(feature = "midi")]
pub fn is_note_on(e: &Eingang) -> bool { status(e) == 0x90 && data2(e) > 0 }
#[cfg(feature = "midi")]
pub fn is_note_off(e: &Eingang) -> bool {
    status(e) == 0x80 || (status(e) == 0x90 && data2(e) == 0)
}
#[cfg(feature = "midi")]
pub fn is_cc(e: &Eingang) -> bool { status(e) == 0xB0 }

#[cfg(feature = "midi")]
pub fn send(a: &mut Ausgang, bytes: &[u8]) -> Result<(), String> {
    a.conn.send(bytes).map_err(|e| format!("MIDI_SEND: {}", e))
}

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
