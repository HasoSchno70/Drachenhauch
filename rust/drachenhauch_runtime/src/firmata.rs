//! Firmata-Protokoll (FIRMATA_*): direkte Pin-Steuerung eines mit
//! StandardFirmata geflashten Arduino/ESP32 ueber eine bestehende serielle
//! Verbindung -- kein eigener Sketch/Text-Protokoll noetig, nur einmalig
//! StandardFirmata auf den Mikrocontroller flashen (Arduino-IDE: Datei ->
//! Beispiele -> Firmata -> StandardFirmata -> Upload). Baut auf demselben
//! `serialport`-Crate wie serial.rs auf (Feature `serial`, keine neue
//! Abhaengigkeit).
//!
//! Nur Pin-I/O (Digital/Analog) -- I2C/Servo/OneWire/Stepper/Encoder aus dem
//! vollen Firmata-Sprachumfang bewusst NICHT abgedeckt (siehe
//! docs/module-firmata.md). Protokoll-Referenz: github.com/firmata/protocol.
//!
//! Zwei Numerierungs-Konventionen sind mit Absicht UNTERSCHIEDLICH (echte
//! Protokoll-Eigenheit, verifiziert gegen StandardFirmata.ino, nicht geraten):
//! FIRMATA_ANALOG_WRITE nimmt die rohe digitale Pin-Nummer (wie
//! FIRMATA_PIN_MODE), FIRMATA_ANALOG_READ nimmt den Analog-KANAL (A0=0,
//! A1=1, ...) -- Schreiben und Lesen sprechen NICHT dieselbe Nummer fuer
//! denselben physischen Pin.
#![cfg(feature = "serial")]

use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;

const CMD_SET_PIN_MODE: u8 = 0xF4;
const CMD_REPORT_ANALOG: u8 = 0xC0; // + Kanal (0..15)
const CMD_REPORT_DIGITAL: u8 = 0xD0; // + Port (0..15)
const CMD_DIGITAL_MESSAGE: u8 = 0x90; // + Port
const CMD_ANALOG_MESSAGE: u8 = 0xE0; // + Pin (Write) bzw. + Kanal (Read-Report)
const CMD_PROTOCOL_VERSION: u8 = 0xF9;
const SYSEX_START: u8 = 0xF0;
const SYSEX_END: u8 = 0xF7;

/// Modus-Werte fuer FIRMATA_PIN_MODE (raw, keine Drachenhauch-Konstanten -- siehe
/// docs/module-firmata.md fuer die vollstaendige Tabelle): 0=INPUT, 1=OUTPUT,
/// 2=ANALOG, 3=PWM, 11=PULLUP.
///
/// 0xE0-0xEF/0xC0-0xCF/0x90-0x9F/0xD0-0xDF haben nur ein 4-Bit-Nibble fuer
/// Port/Kanal/Pin im Kommando-Byte selbst -- daher 0..15, keine willkuerliche
/// Grenze. Fuer hoehere Analog-Pins gibt es EXTENDED_ANALOG (SysEx 0x6F),
/// hier bewusst nicht implementiert.
const N_PORTS: usize = 16;
const N_CHANNELS: usize = 16;

pub struct Board {
    conn: Box<dyn SerialPort>,
    /// Schattenregister des zuletzt GESENDETEN Digital-Outputs je Port (8 Pins/Port)
    /// -- noetig, weil DIGITAL_MESSAGE immer den ganzen Port ueberschreibt, ein
    /// einzelnes FIRMATA_DIGITAL_WRITE also die anderen 7 Pins unveraendert
    /// mitschicken muss.
    out_state: [u8; N_PORTS],
    /// Zuletzt vom Board GEMELDETER Digital-Zustand je Port.
    in_state: [u8; N_PORTS],
    reporting_digital: [bool; N_PORTS],
    analog_state: [i32; N_CHANNELS],
    reporting_analog: [bool; N_CHANNELS],
    /// Rest einer ueber zwei Reads verteilten, noch unvollstaendigen Nachricht.
    rx_pending: Vec<u8>,
}

fn check_range(fn_: &str, label: &str, v: i64, lo: i64, hi: i64) -> Result<(), String> {
    if v < lo || v > hi {
        return Err(format!("{}: {} {} ausserhalb {}..{}", fn_, label, v, lo, hi));
    }
    Ok(())
}

pub fn open(port: &str, baud: i64) -> Result<Board, String> {
    let conn = serialport::new(port, baud as u32)
        // Kurzer Timeout: FIRMATA_UPDATE() wird pro Frame gepollt (wie
        // INPUT_UPDATE/TIMER_UPDATE), darf also nicht spuerbar blockieren --
        // anders als serial.rs (1s-Timeout, konsolenartige Nutzung).
        .timeout(Duration::from_millis(1))
        .open()
        .map_err(|e| format!("FIRMATA_OPEN: {}", e))?;
    // Arduino-Autoreset: viele Boards resetten beim Oeffnen des Ports (DTR-
    // Toggle) und brauchen eine kurze Bootzeit, bevor der Firmata-Sketch
    // Kommandos entgegennimmt -- bekannte, in jeder Firmata-Client-Bibliothek
    // (z.B. pyFirmata: BOARD_SETUP_WAIT_TIME) dokumentierte Eigenheit. ESP32-
    // Boards mit anderem Reset-Verhalten brauchen es meist nicht, die
    // Wartezeit schadet dort aber auch nicht.
    std::thread::sleep(Duration::from_millis(2000));
    Ok(Board {
        conn,
        out_state: [0; N_PORTS],
        in_state: [0; N_PORTS],
        reporting_digital: [false; N_PORTS],
        analog_state: [0; N_CHANNELS],
        reporting_analog: [false; N_CHANNELS],
        rx_pending: Vec::new(),
    })
}

fn write_bytes(b: &mut Board, bytes: &[u8]) -> Result<(), String> {
    b.conn
        .write_all(bytes)
        .map_err(|e| format!("FIRMATA: Schreiben fehlgeschlagen: {}", e))
}

pub fn pin_mode(b: &mut Board, pin: i64, mode: i64) -> Result<(), String> {
    check_range("FIRMATA_PIN_MODE", "Pin", pin, 0, 127)?;
    check_range("FIRMATA_PIN_MODE", "Modus", mode, 0, 127)?;
    write_bytes(b, &[CMD_SET_PIN_MODE, pin as u8, mode as u8])
}

/// (Port-Index, Bit-Position innerhalb des Ports) fuer einen rohen Pin (0..127).
/// Pure (fuer #[test]).
fn port_of(pin: i64) -> (usize, u32) {
    ((pin / 8) as usize, (pin % 8) as u32)
}

/// Pure Bit-Op auf dem Port-Schattenregister (fuer #[test] ohne Board).
fn set_port_bit(byte: u8, bit: u32, on: bool) -> u8 {
    if on { byte | (1 << bit) } else { byte & !(1 << bit) }
}

fn get_port_bit(byte: u8, bit: u32) -> bool {
    (byte >> bit) & 1 != 0
}

pub fn digital_write(b: &mut Board, pin: i64, value: bool) -> Result<(), String> {
    check_range("FIRMATA_DIGITAL_WRITE", "Pin", pin, 0, 127)?;
    let (port, bit) = port_of(pin);
    b.out_state[port] = set_port_bit(b.out_state[port], bit, value);
    let byte = b.out_state[port];
    let lo = byte & 0x7F;
    let hi = (byte >> 7) & 0x01;
    write_bytes(b, &[CMD_DIGITAL_MESSAGE | port as u8, lo, hi])
}

/// Liest den zuletzt VOM BOARD GEMELDETEN Zustand (siehe update()). Aktiviert
/// beim ersten Aufruf fuer diesen Pin automatisch das Reporting seines Ports
/// (REPORT_DIGITAL_PORT) -- kein separater Enable-Schritt noetig, damit
/// FIRMATA_DIGITAL_READ direkt nach FIRMATA_PIN_MODE(..., FIRMATA_INPUT)
/// funktioniert.
pub fn digital_read(b: &mut Board, pin: i64) -> Result<bool, String> {
    check_range("FIRMATA_DIGITAL_READ", "Pin", pin, 0, 127)?;
    let (port, bit) = port_of(pin);
    if !b.reporting_digital[port] {
        b.reporting_digital[port] = true;
        write_bytes(b, &[CMD_REPORT_DIGITAL | port as u8, 1])?;
    }
    Ok(get_port_bit(b.in_state[port], bit))
}

/// `pin` ist die ROHE digitale Pin-Nummer (wie bei FIRMATA_PIN_MODE) --
/// StandardFirmata liest den ANALOG_MESSAGE-Pin beim Schreiben direkt als
/// Pin-Index (analogWriteCallback), nicht als Analog-Kanal. Nur Pins 0..15
/// (Nibble im Kommando-Byte) -- EXTENDED_ANALOG fuer hoehere Pins nicht
/// implementiert.
pub fn analog_write(b: &mut Board, pin: i64, value: i64) -> Result<(), String> {
    check_range("FIRMATA_ANALOG_WRITE", "Pin", pin, 0, 15)?;
    check_range("FIRMATA_ANALOG_WRITE", "Wert", value, 0, 16383)?;
    let lo = (value & 0x7F) as u8;
    let hi = ((value >> 7) & 0x7F) as u8;
    write_bytes(b, &[CMD_ANALOG_MESSAGE | pin as u8, lo, hi])
}

/// `channel` ist der Analog-KANAL (A0=0, A1=1, ...), NICHT die digitale
/// Pin-Nummer -- StandardFirmata sendet beim Reporting den Kanal
/// (PIN_TO_ANALOG), nicht den Pin-Index. Aktiviert Reporting beim ersten
/// Aufruf automatisch (wie digital_read).
pub fn analog_read(b: &mut Board, channel: i64) -> Result<i64, String> {
    check_range("FIRMATA_ANALOG_READ", "Kanal", channel, 0, 15)?;
    let ch = channel as usize;
    if !b.reporting_analog[ch] {
        b.reporting_analog[ch] = true;
        write_bytes(b, &[CMD_REPORT_ANALOG | channel as u8, 1])?;
    }
    Ok(b.analog_state[ch] as i64)
}

#[derive(Debug, PartialEq, Eq)]
enum Msg {
    Digital { port: usize, byte: u8 },
    Analog { channel: usize, value: i32 },
}

/// Parst so viele vollstaendige Firmata-Nachrichten wie moeglich vom
/// Puffer-Anfang; gibt die erkannten Nachrichten + den nicht verbrauchten
/// Rest zurueck (fuer den naechsten update()-Aufruf, falls eine Nachricht
/// quer ueber zwei Reads ankam -- gleiches Muster wie serial.rs' `pending`).
/// Unbekannte/nicht abgeschlossene SysEx-Bloecke und der Protokoll-Version-
/// Report werden erkannt und uebersprungen, nicht als Daten fehlinterpretiert.
/// Pure Funktion (fuer #[test] ohne echten Port).
fn parse_messages(mut buf: Vec<u8>) -> (Vec<Msg>, Vec<u8>) {
    let mut out = Vec::new();
    let mut i = 0;
    while i < buf.len() {
        let b0 = buf[i];
        if b0 & 0xF0 == CMD_DIGITAL_MESSAGE {
            if i + 2 >= buf.len() { break; }
            let port = (b0 & 0x0F) as usize;
            let byte = (buf[i + 1] & 0x7F) | ((buf[i + 2] & 0x01) << 7);
            out.push(Msg::Digital { port, byte });
            i += 3;
        } else if b0 & 0xF0 == CMD_ANALOG_MESSAGE {
            if i + 2 >= buf.len() { break; }
            let channel = (b0 & 0x0F) as usize;
            let value = (buf[i + 1] & 0x7F) as i32 | (((buf[i + 2] & 0x7F) as i32) << 7);
            out.push(Msg::Analog { channel, value });
            i += 3;
        } else if b0 == CMD_PROTOCOL_VERSION {
            if i + 2 >= buf.len() { break; }
            i += 3;
        } else if b0 == SYSEX_START {
            match buf[i..].iter().position(|&x| x == SYSEX_END) {
                Some(rel) => i += rel + 1,
                None => break, // SysEx noch nicht vollstaendig angekommen
            }
        } else {
            // Stray-Byte (z.B. Boot-Rauschen) -- einzeln ueberspringen statt haengenzubleiben.
            i += 1;
        }
    }
    let rest = buf.split_off(i);
    (out, rest)
}

/// Liest alle aktuell verfuegbaren Bytes (nicht-blockierend, 1ms-Timeout aus
/// open()) und aktualisiert die Digital-/Analog-Caches. Pro Frame aufrufen,
/// wie INPUT_UPDATE()/TIMER_UPDATE() -- ohne Aufruf bleiben
/// FIRMATA_DIGITAL_READ/FIRMATA_ANALOG_READ auf dem letzten Stand stehen.
pub fn update(b: &mut Board) -> Result<(), String> {
    let mut chunk = [0u8; 4096];
    let n = match b.conn.read(&mut chunk) {
        Ok(n) => n,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => 0,
        Err(e) => return Err(format!("FIRMATA_UPDATE: {}", e)),
    };
    if n == 0 {
        return Ok(());
    }
    let mut pending = std::mem::take(&mut b.rx_pending);
    pending.extend_from_slice(&chunk[..n]);
    let (msgs, rest) = parse_messages(pending);
    b.rx_pending = rest;
    for m in msgs {
        match m {
            Msg::Digital { port, byte } => {
                if port < N_PORTS { b.in_state[port] = byte; }
            }
            Msg::Analog { channel, value } => {
                if channel < N_CHANNELS { b.analog_state[channel] = value; }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_of_splits_pin_into_port_and_bit() {
        assert_eq!(port_of(0), (0, 0));
        assert_eq!(port_of(7), (0, 7));
        assert_eq!(port_of(8), (1, 0));
        assert_eq!(port_of(13), (1, 5));
    }

    #[test]
    fn port_bit_set_and_get_roundtrip() {
        let mut byte = 0u8;
        byte = set_port_bit(byte, 3, true);
        assert!(get_port_bit(byte, 3));
        assert!(!get_port_bit(byte, 2));
        byte = set_port_bit(byte, 3, false);
        assert!(!get_port_bit(byte, 3));
    }

    #[test]
    fn digital_write_preserves_other_pins_in_same_port() {
        // Pin 2 und Pin 5 liegen im selben Port (0..7) -- Setzen von Pin 2
        // darf Pin 5 nicht antasten (das ist der ganze Grund fuer out_state).
        let mut byte = 0u8;
        byte = set_port_bit(byte, 5, true);
        byte = set_port_bit(byte, 2, true);
        assert!(get_port_bit(byte, 5));
        assert!(get_port_bit(byte, 2));
        byte = set_port_bit(byte, 2, false);
        assert!(get_port_bit(byte, 5));
        assert!(!get_port_bit(byte, 2));
    }

    #[test]
    fn parse_single_digital_message() {
        // Port 1, Pins 0-6 = 0b0010101 (bit0,2,4 gesetzt), Pin7 = 1
        let buf = vec![CMD_DIGITAL_MESSAGE | 1, 0b0010101, 0b1];
        let (msgs, rest) = parse_messages(buf);
        assert_eq!(msgs, vec![Msg::Digital { port: 1, byte: 0b10010101 }]);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_single_analog_message_14bit_value() {
        // Kanal 3, Wert 1023 (0x3FF) = lo 0x7F, hi 0x07
        let buf = vec![CMD_ANALOG_MESSAGE | 3, 0x7F, 0x07];
        let (msgs, rest) = parse_messages(buf);
        assert_eq!(msgs, vec![Msg::Analog { channel: 3, value: 1023 }]);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_leaves_incomplete_trailing_message_for_next_call() {
        let buf = vec![CMD_DIGITAL_MESSAGE | 0, 0x7F]; // 3. Byte fehlt noch
        let (msgs, rest) = parse_messages(buf);
        assert!(msgs.is_empty());
        assert_eq!(rest, vec![CMD_DIGITAL_MESSAGE | 0, 0x7F]);
    }

    #[test]
    fn parse_message_split_across_two_reads_reassembles() {
        let (msgs1, rest) = parse_messages(vec![CMD_ANALOG_MESSAGE | 0, 0x11]);
        assert!(msgs1.is_empty());
        let mut buf2 = rest;
        buf2.push(0x02);
        let (msgs2, rest2) = parse_messages(buf2);
        assert_eq!(msgs2, vec![Msg::Analog { channel: 0, value: 0x11 | (0x02 << 7) }]);
        assert!(rest2.is_empty());
    }

    #[test]
    fn parse_skips_protocol_version_report() {
        let buf = vec![CMD_PROTOCOL_VERSION, 2, 5, CMD_DIGITAL_MESSAGE | 0, 0x01, 0x00];
        let (msgs, rest) = parse_messages(buf);
        assert_eq!(msgs, vec![Msg::Digital { port: 0, byte: 0x01 }]);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_skips_complete_sysex_block() {
        let buf = vec![SYSEX_START, 0x79, 2, 3, SYSEX_END, CMD_ANALOG_MESSAGE | 5, 0x00, 0x01];
        let (msgs, rest) = parse_messages(buf);
        assert_eq!(msgs, vec![Msg::Analog { channel: 5, value: 128 }]);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_leaves_incomplete_sysex_block_for_next_call() {
        let buf = vec![SYSEX_START, 0x79, 2, 3]; // kein 0xF7 noch
        let (msgs, rest) = parse_messages(buf);
        assert!(msgs.is_empty());
        assert_eq!(rest, vec![SYSEX_START, 0x79, 2, 3]);
    }

    #[test]
    fn parse_skips_stray_bytes_without_getting_stuck() {
        let buf = vec![0x00, 0x01, CMD_DIGITAL_MESSAGE | 2, 0x40, 0x00];
        let (msgs, rest) = parse_messages(buf);
        assert_eq!(msgs, vec![Msg::Digital { port: 2, byte: 0x40 }]);
        assert!(rest.is_empty());
    }
}
