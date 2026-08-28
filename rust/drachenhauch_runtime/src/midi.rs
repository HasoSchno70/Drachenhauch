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
