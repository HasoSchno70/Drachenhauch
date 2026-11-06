//! Firmata-Protokoll (FIRMATA_*): direkte Steuerung eines mit Firmata
//! geflashten Arduino/ESP32 -- kein eigener Sketch, kein Textprotokoll, nur
//! einmalig Firmata auf den Mikrocontroller spielen (Arduino-IDE: Datei ->
//! Beispiele -> Firmata -> StandardFirmata -> Upload).
//!
//! **Zwei Leitungen:** eine serielle Schnittstelle (`COM3`, `/dev/ttyUSB0`;
//! Feature `serial`, dieselbe Crate wie serial.rs) oder `tcp:host:port` --
//! StandardFirmataWiFi/-Ethernet bieten genau das (Port 3030). Der Netzweg
//! braucht kein Feature: Firmata gibt es darum in JEDEM Bau, und die
//! Pruefsammlung spielt das Board ueber das net-Modul.
//!
//! Umfang: Pins (digital/analog, PWM), Servo (SERVO_CONFIG 0x70), I2C
//! (0x76-0x78, schreiben, einmal/fortlaufend lesen), Firmware-Abfrage (0x79)
//! und Schrittmotoren ueber AccelStepperFirmata (0x62) -- die stecken NICHT
//! in StandardFirmata, sondern in ConfigurableFirmata. OneWire, Encoder,
//! Scheduler bleiben aussen vor (docs/module-firmata.md).
//! Protokoll-Referenz: github.com/firmata/protocol (protocol.md, i2c.md,
//! servos.md, accelStepperFirmata.md).
//!
//! Zwei Numerierungs-Konventionen sind mit Absicht UNTERSCHIEDLICH (echte
//! Protokoll-Eigenheit, verifiziert gegen StandardFirmata.ino, nicht geraten):
//! FIRMATA_ANALOG_WRITE nimmt die rohe digitale Pin-Nummer (wie
//! FIRMATA_PIN_MODE), FIRMATA_ANALOG_READ nimmt den Analog-KANAL (A0=0,
//! A1=1, ...) -- Schreiben und Lesen sprechen NICHT dieselbe Nummer fuer
//! denselben physischen Pin.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

const CMD_SET_PIN_MODE: u8 = 0xF4;
const CMD_REPORT_ANALOG: u8 = 0xC0; // + Kanal (0..15)
const CMD_REPORT_DIGITAL: u8 = 0xD0; // + Port (0..15)
const CMD_DIGITAL_MESSAGE: u8 = 0x90; // + Port
const CMD_ANALOG_MESSAGE: u8 = 0xE0; // + Pin (Write) bzw. + Kanal (Read-Report)
const CMD_PROTOCOL_VERSION: u8 = 0xF9;
const SYSEX_START: u8 = 0xF0;
const SYSEX_END: u8 = 0xF7;

const SYSEX_ACCELSTEPPER: u8 = 0x62;
const SYSEX_EXTENDED_ANALOG: u8 = 0x6F;
const SYSEX_SERVO_CONFIG: u8 = 0x70;
const SYSEX_I2C_REQUEST: u8 = 0x76;
const SYSEX_I2C_REPLY: u8 = 0x77;
const SYSEX_I2C_CONFIG: u8 = 0x78;
const SYSEX_REPORT_FIRMWARE: u8 = 0x79;

// Zweites Byte einer I2C-Anfrage (i2c.md): Bits 3-4 die Art, Bit 5 = 10-Bit-
// Adresse, Bits 0-2 die oberen drei Bits einer solchen.
const I2C_WRITE: u8 = 0x00;
const I2C_READ: u8 = 0x08;
const I2C_READ_CONTINUOUSLY: u8 = 0x10;
const I2C_STOP_READING: u8 = 0x18;
const I2C_TEN_BIT: u8 = 0x20;

// Unterbefehle von AccelStepperFirmata (accelStepperFirmata.md).
const STEPPER_CONFIG: u8 = 0x00;
const STEPPER_ZERO: u8 = 0x01;
const STEPPER_STEP: u8 = 0x02;
const STEPPER_TO: u8 = 0x03;
const STEPPER_STOP: u8 = 0x05;
const STEPPER_REPORT: u8 = 0x06;
const STEPPER_ACCEL: u8 = 0x08;
const STEPPER_SPEED: u8 = 0x09;
const STEPPER_DONE: u8 = 0x0A;

/// Modus-Werte fuer FIRMATA_PIN_MODE (raw, keine Drachenhauch-Konstanten -- siehe
/// docs/module-firmata.md fuer die vollstaendige Tabelle): 0=INPUT, 1=OUTPUT,
/// 2=ANALOG, 3=PWM, 4=SERVO, 6=I2C, 11=PULLUP.
///
/// 0xE0-0xEF/0xC0-0xCF/0x90-0x9F/0xD0-0xDF haben nur ein 4-Bit-Nibble fuer
/// Port/Kanal/Pin im Kommando-Byte selbst -- daher 0..15, keine willkuerliche
/// Grenze. Hoehere Pins SCHREIBT EXTENDED_ANALOG (SysEx 0x6F).
const N_PORTS: usize = 16;
const N_CHANNELS: usize = 16;
/// AccelStepperFirmata kennt bis zu zehn Motoren.
const N_STEPPER: usize = 10;
/// So lange warten FIRMATA_I2C_READ und FIRMATA_FIRMWARE$ auf die Antwort.
const ANTWORT_FRIST: Duration = Duration::from_millis(1000);

/// Die Leitung zum Board. Beide Arten lesen, ohne zu warten: FIRMATA_UPDATE
/// laeuft je Bild, und ein Bild darf nicht auf ein Board warten.
enum Leitung {
    #[cfg(feature = "serial")]
    Seriell(Box<dyn serialport::SerialPort>),
    Tcp(TcpStream),
}

impl Leitung {
    fn schreiben(&mut self, bytes: &[u8]) -> Result<(), String> {
        match self {
            #[cfg(feature = "serial")]
            Leitung::Seriell(c) => c
                .write_all(bytes)
                .map_err(|e| format!("FIRMATA: Schreiben fehlgeschlagen: {}", e)),
            Leitung::Tcp(s) => {
                // Die Verbindung liest ohne Warten (set_nonblocking) -- dann
                // schreibt sie auch so, und ein voller Puffer ist ein WouldBlock,
                // kein Fehler. Kurz warten und weiterschreiben.
                let mut rest = bytes;
                let frist = Instant::now() + Duration::from_secs(5);
                while !rest.is_empty() {
                    match s.write(rest) {
                        Ok(0) => return Err("FIRMATA: die Verbindung zum Board ist beendet".to_string()),
                        Ok(n) => rest = &rest[n..],
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            if Instant::now() > frist {
                                return Err("FIRMATA: das Board nimmt nichts mehr an".to_string());
                            }
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                        Err(e) => return Err(format!("FIRMATA: Schreiben fehlgeschlagen: {}", e)),
                    }
                }
                Ok(())
            }
        }
    }

    /// Was gerade da ist; 0 = nichts.
    fn lesen(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        match self {
            #[cfg(feature = "serial")]
            Leitung::Seriell(c) => match c.read(buf) {
                Ok(n) => Ok(n),
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(0),
                Err(e) => Err(format!("FIRMATA_UPDATE: {}", e)),
            },
            Leitung::Tcp(s) => match s.read(buf) {
                // Bei TCP heisst 0 Bytes: die Gegenseite hat geschlossen.
                Ok(0) => Err("FIRMATA_UPDATE: die Verbindung zum Board ist beendet".to_string()),
                Ok(n) => Ok(n),
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::Interrupted =>
                {
                    Ok(0)
                }
                Err(e) => Err(format!("FIRMATA_UPDATE: {}", e)),
            },
        }
    }
}

pub struct Board {
    leitung: Leitung,
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
    /// I2C_CONFIG schon geschickt? StandardFirmata schaltet I2C erst damit
    /// ein -- die erste I2C-Anfrage schickt es darum selbst, wenn es fehlt.
    i2c_an: bool,
    /// Zaehlt jede I2C-Antwort; eine Antwort traegt die Nummer, mit der sie
    /// kam -- so erkennt FIRMATA_I2C_READ die NEUE.
    i2c_zaehler: u64,
    /// Letzte Antwort je (Adresse, Register).
    i2c_daten: HashMap<(u16, u16), (u64, Vec<u8>)>,
    /// Letzte Antwort je Adresse, gleich welches Register (fuer Lesen ohne
    /// Register: StandardFirmata meldet dann Register 0).
    i2c_letzte: HashMap<u16, (u64, Vec<u8>)>,
    firmware: Option<String>,
    stepper_da: [bool; N_STEPPER],
    stepper_pos: [i64; N_STEPPER],
    stepper_laeuft: [bool; N_STEPPER],
}

fn check_range(fn_: &str, label: &str, v: i64, lo: i64, hi: i64) -> Result<(), String> {
    if v < lo || v > hi {
        return Err(format!("{}: {} {} ausserhalb {}..{}", fn_, label, v, lo, hi));
    }
    Ok(())
}

/// Eine Zahl bis 14 Bit als zwei 7-Bit-Bytes, das niedrige zuerst -- so
/// stehen Zahlen in Firmata-SysEx-Bloecken (kein Byte darf 0x80 erreichen).
fn sieben(v: u32) -> [u8; 2] {
    [(v & 0x7F) as u8, ((v >> 7) & 0x7F) as u8]
}

/// Die Gegenrichtung: je zwei 7-Bit-Bytes ein Wert.
fn paare(d: &[u8]) -> Vec<u16> {
    d.chunks(2)
        .map(|p| (p[0] & 0x7F) as u16 | (((*p.get(1).unwrap_or(&0) & 0x7F) as u16) << 7))
        .collect()
}

/// 32-Bit-Zahl fuer AccelStepperFirmata: Betrag in 31 Bit ueber fuenf
/// 7-Bit-Bytes, das Vorzeichen als Bit 3 des fuenften (wie firmata.js
/// `encode32BitSignedInteger` und ConfigurableFirmata `decode32BitSignedInteger`)
/// -- kein Zweierkomplement.
fn zahl32(v: i64) -> [u8; 5] {
    let m = v.unsigned_abs();
    let mut b = [
        (m & 0x7F) as u8,
        ((m >> 7) & 0x7F) as u8,
        ((m >> 14) & 0x7F) as u8,
        ((m >> 21) & 0x7F) as u8,
        ((m >> 28) & 0x07) as u8,
    ];
    if v < 0 {
        b[4] |= 0x08;
    }
    b
}

fn zahl32_lesen(d: &[u8]) -> i64 {
    if d.len() < 5 {
        return 0;
    }
    let m = (d[0] & 0x7F) as i64
        | ((d[1] & 0x7F) as i64) << 7
        | ((d[2] & 0x7F) as i64) << 14
        | ((d[3] & 0x7F) as i64) << 21
        | ((d[4] & 0x07) as i64) << 28;
    if d[4] & 0x08 != 0 { -m } else { m }
}

/// Kommazahl fuer AccelStepperFirmata (`decodeCustomFloat`): 23 Bit
/// Ganzzahl, 4 Bit Zehnerexponent mit Versatz 11, 1 Bit Vorzeichen --
/// Wert = Ganzzahl * 10^(Exponent - 11). Gewaehlt wird der kleinste
/// Exponent, bei dem die Ganzzahl noch passt und genau ist; geht es nirgends
/// genau, der kleinste, bei dem sie passt (abgeschnitten).
fn kommazahl(f: f64) -> Result<[u8; 4], String> {
    if f == 0.0 {
        return Ok([0, 0, 0, 0]);
    }
    if !f.is_finite() {
        return Err("keine endliche Zahl".to_string());
    }
    const GRENZE: f64 = 8_388_608.0; // 2^23
    let x = f.abs();
    let passt = |e: i32| -> Option<f64> {
        let s = x * 10f64.powi(-e);
        if s < GRENZE { Some(s) } else { None }
    };
    let mut wahl = None;
    for e in -11..=4 {
        if let Some(s) = passt(e) {
            let r = s.round();
            if r < GRENZE && (s - r).abs() <= 1e-9 * s.max(1.0) {
                wahl = Some((e, r as u32));
                break;
            }
        }
    }
    if wahl.is_none() {
        for e in -11..=4 {
            if let Some(s) = passt(e) {
                wahl = Some((e, s.trunc() as u32));
                break;
            }
        }
    }
    let (e, sig) = wahl.ok_or_else(|| format!("{} ist zu gross", f))?;
    let exp = (e + 11) as u32;
    Ok([
        (sig & 0x7F) as u8,
        ((sig >> 7) & 0x7F) as u8,
        ((sig >> 14) & 0x7F) as u8,
        (((sig >> 21) & 0x03) | ((exp & 0x0F) << 2) | (if f < 0.0 { 1 << 6 } else { 0 })) as u8,
    ])
}

#[cfg(test)]
fn kommazahl_lesen(d: &[u8; 4]) -> f64 {
    let sig = d[0] as u32 | (d[1] as u32) << 7 | (d[2] as u32) << 14 | ((d[3] & 0x03) as u32) << 21;
    let exp = ((d[3] >> 2) & 0x0F) as i32 - 11;
    let w = sig as f64 * 10f64.powi(exp);
    if d[3] & 0x40 != 0 { -w } else { w }
}

/// `tcp:host:port` -> (host, port); IPv6 in eckigen Klammern (`tcp:[::1]:3030`).
fn tcp_ziel(ziel: &str) -> Option<(String, u16)> {
    let rest = if ziel.len() >= 4 && ziel[..4].eq_ignore_ascii_case("tcp:") { &ziel[4..] } else { return None };
    let (host, port) = rest.rsplit_once(':')?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if host.is_empty() {
        return None;
    }
    Some((host.to_string(), port.trim().parse().ok()?))
}

/// Liste der seriellen Anschluesse (dieselbe wie SERIAL_PORTS).
pub fn ports() -> Result<String, String> {
    #[cfg(feature = "serial")]
    {
        Ok(crate::serial::ports())
    }
    #[cfg(not(feature = "serial"))]
    {
        Err(ohne_seriell("FIRMATA_PORTS"))
    }
}

#[cfg(not(feature = "serial"))]
fn ohne_seriell(fn_: &str) -> String {
    format!(
        "{}: serielle Anschluesse kennt diese Laufzeit nicht (gebaut ohne --hardware) \
         -- ueber das Netz geht \"tcp:host:port\" (StandardFirmataWiFi, Port 3030)",
        fn_
    )
}

pub fn open(ziel: &str, baud: i64) -> Result<Board, String> {
    let leitung = if ziel.len() >= 4 && ziel[..4].eq_ignore_ascii_case("tcp:") {
        let (host, port) = tcp_ziel(ziel).ok_or_else(|| {
            format!("FIRMATA_OPEN: \"{}\" ist kein Netzziel -- so: \"tcp:192.168.1.50:3030\"", ziel)
        })?;
        let adressen: Vec<_> = (host.as_str(), port)
            .to_socket_addrs()
            .map_err(|e| format!("FIRMATA_OPEN: {} nicht gefunden: {}", host, e))?
            .collect();
        let mut letzter = format!("FIRMATA_OPEN: {} hat keine Adresse", host);
        let mut verbunden = None;
        for a in adressen {
            match TcpStream::connect_timeout(&a, Duration::from_secs(3)) {
                Ok(s) => {
                    verbunden = Some(s);
                    break;
                }
                Err(e) => letzter = format!("FIRMATA_OPEN: {}:{} -- {}", host, port, e),
            }
        }
        let s = verbunden.ok_or(letzter)?;
        let _ = s.set_nodelay(true);
        s.set_nonblocking(true).map_err(|e| format!("FIRMATA_OPEN: {}", e))?;
        Leitung::Tcp(s)
    } else {
        seriell_oeffnen(ziel, baud)?
    };
    Ok(Board {
        leitung,
        out_state: [0; N_PORTS],
        in_state: [0; N_PORTS],
        reporting_digital: [false; N_PORTS],
        analog_state: [0; N_CHANNELS],
        reporting_analog: [false; N_CHANNELS],
        rx_pending: Vec::new(),
        i2c_an: false,
        i2c_zaehler: 0,
        i2c_daten: HashMap::new(),
        i2c_letzte: HashMap::new(),
        firmware: None,
        stepper_da: [false; N_STEPPER],
        stepper_pos: [0; N_STEPPER],
        stepper_laeuft: [false; N_STEPPER],
    })
}

#[cfg(feature = "serial")]
fn seriell_oeffnen(port: &str, baud: i64) -> Result<Leitung, String> {
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
    // Wartezeit schadet dort aber auch nicht. Uebers Netz gibt es keinen
    // Reset, dort wird nicht gewartet.
    std::thread::sleep(Duration::from_millis(2000));
    Ok(Leitung::Seriell(conn))
}

#[cfg(not(feature = "serial"))]
fn seriell_oeffnen(_port: &str, _baud: i64) -> Result<Leitung, String> {
    Err(ohne_seriell("FIRMATA_OPEN"))
}

fn write_bytes(b: &mut Board, bytes: &[u8]) -> Result<(), String> {
    b.leitung.schreiben(bytes)
}

/// Ein SysEx-Block: F0, Befehl, Daten (alle unter 0x80), F7.
fn sysex(b: &mut Board, befehl: u8, daten: &[u8]) -> Result<(), String> {
    let mut out = Vec::with_capacity(daten.len() + 3);
    out.push(SYSEX_START);
    out.push(befehl);
    out.extend(daten.iter().map(|x| x & 0x7F));
    out.push(SYSEX_END);
    write_bytes(b, &out)
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

/// Die Bytes, die einen Wert an einen Pin schicken: ANALOG_MESSAGE bis Pin
/// 15 (das Nibble im Befehl), darueber EXTENDED_ANALOG. Pure (fuer #[test]).
fn analog_bytes(pin: u8, value: u32) -> Vec<u8> {
    if pin < 16 {
        let [lo, hi] = sieben(value);
        vec![CMD_ANALOG_MESSAGE | pin, lo, hi]
    } else {
        let [lo, hi] = sieben(value);
        vec![SYSEX_START, SYSEX_EXTENDED_ANALOG, pin, lo, hi, SYSEX_END]
    }
}

/// `pin` ist die ROHE digitale Pin-Nummer (wie bei FIRMATA_PIN_MODE) --
/// StandardFirmata liest den ANALOG_MESSAGE-Pin beim Schreiben direkt als
/// Pin-Index (analogWriteCallback), nicht als Analog-Kanal.
pub fn analog_write(b: &mut Board, pin: i64, value: i64) -> Result<(), String> {
    check_range("FIRMATA_ANALOG_WRITE", "Pin", pin, 0, 127)?;
    check_range("FIRMATA_ANALOG_WRITE", "Wert", value, 0, 16383)?;
    write_bytes(b, &analog_bytes(pin as u8, value as u32))
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

// ------------------------------------------------------------ Servo

/// SERVO_CONFIG: haengt den Servo an den Pin (StandardFirmata schaltet den
/// Pin dabei selbst auf SERVO) und legt die Pulsbreiten fuer 0 und 180 Grad
/// fest. Die Vorgabe 544/2400 ist die der Arduino-Servo-Bibliothek.
pub fn servo_attach(b: &mut Board, pin: i64, min_us: i64, max_us: i64) -> Result<(), String> {
    check_range("FIRMATA_SERVO_ATTACH", "Pin", pin, 0, 127)?;
    check_range("FIRMATA_SERVO_ATTACH", "kleinster Puls", min_us, 1, 16382)?;
    check_range("FIRMATA_SERVO_ATTACH", "groesster Puls", max_us, min_us + 1, 16383)?;
    let [a, b1] = sieben(min_us as u32);
    let [c, d] = sieben(max_us as u32);
    sysex(b, SYSEX_SERVO_CONFIG, &[pin as u8, a, b1, c, d])
}

/// Winkel 0..180. Geht als Analog-Wert an den Pin -- so stellt StandardFirmata
/// einen Servo (analogWriteCallback -> servo.write).
pub fn servo_write(b: &mut Board, pin: i64, winkel: i64) -> Result<(), String> {
    check_range("FIRMATA_SERVO_WRITE", "Pin", pin, 0, 127)?;
    check_range("FIRMATA_SERVO_WRITE", "Winkel", winkel, 0, 180)?;
    write_bytes(b, &analog_bytes(pin as u8, winkel as u32))
}

// ------------------------------------------------------------ I2C

pub fn i2c_config(b: &mut Board, verzoegerung_us: i64) -> Result<(), String> {
    check_range("FIRMATA_I2C_CONFIG", "Verzoegerung", verzoegerung_us, 0, 16383)?;
    let d = sieben(verzoegerung_us as u32);
    sysex(b, SYSEX_I2C_CONFIG, &d)?;
    b.i2c_an = true;
    Ok(())
}

fn i2c_bereit(b: &mut Board) -> Result<(), String> {
    if !b.i2c_an { i2c_config(b, 0) } else { Ok(()) }
}

/// Die ersten beiden Bytes einer I2C-Anfrage. Adressen bis 127 sind die
/// gewoehnlichen 7-Bit-Adressen, bis 1023 gehen als 10-Bit-Adresse.
fn i2c_kopf(fn_: &str, adresse: i64, art: u8) -> Result<[u8; 2], String> {
    check_range(fn_, "Adresse", adresse, 0, 1023)?;
    let a = adresse as u32;
    Ok(if a > 127 {
        [(a & 0x7F) as u8, art | I2C_TEN_BIT | ((a >> 7) & 0x07) as u8]
    } else {
        [a as u8, art]
    })
}

pub fn i2c_write(b: &mut Board, adresse: i64, daten: &[i64]) -> Result<(), String> {
    let kopf = i2c_kopf("FIRMATA_I2C_WRITE", adresse, I2C_WRITE)?;
    let mut d = kopf.to_vec();
    for (k, &x) in daten.iter().enumerate() {
        if !(0..=255).contains(&x) {
            return Err(format!("FIRMATA_I2C_WRITE: Byte {} ist {} -- erlaubt sind 0..255", k, x));
        }
        d.extend_from_slice(&sieben(x as u32));
    }
    i2c_bereit(b)?;
    sysex(b, SYSEX_I2C_REQUEST, &d)
}

/// Anfrage: `anzahl` Bytes lesen, ab `register` (-1 = ohne Register, das Geraet
/// liefert, was es von sich aus liefert). `dauernd` = das Board liest weiter
/// und meldet jedes Mal (bis FIRMATA_I2C_STOP).
pub fn i2c_request(b: &mut Board, fn_: &str, adresse: i64, register: i64, anzahl: i64, dauernd: bool) -> Result<(), String> {
    let kopf = i2c_kopf(fn_, adresse, if dauernd { I2C_READ_CONTINUOUSLY } else { I2C_READ })?;
    check_range(fn_, "Register", register, -1, 16383)?;
    // Der Puffer der Wire-Bibliothek fasst 32 Bytes; ESP32 mehr. Firmata selbst
    // meldet bis 64 -- darueber kaeme die Antwort nicht in einem Stueck.
    check_range(fn_, "Anzahl", anzahl, 1, 64)?;
    let mut d = kopf.to_vec();
    if register >= 0 {
        d.extend_from_slice(&sieben(register as u32));
    }
    d.extend_from_slice(&sieben(anzahl as u32));
    i2c_bereit(b)?;
    sysex(b, SYSEX_I2C_REQUEST, &d)
}

pub fn i2c_stop(b: &mut Board, adresse: i64) -> Result<(), String> {
    let kopf = i2c_kopf("FIRMATA_I2C_STOP", adresse, I2C_STOP_READING)?;
    i2c_bereit(b)?;
    sysex(b, SYSEX_I2C_REQUEST, &kopf)
}

/// Die neueste Antwort fuer (Adresse, Register) bzw. ohne Register die
/// neueste der Adresse: (Nummer, Bytes).
fn i2c_antwort(b: &Board, adresse: i64, register: i64) -> Option<&(u64, Vec<u8>)> {
    if register < 0 {
        b.i2c_letzte.get(&(adresse as u16))
    } else {
        b.i2c_daten.get(&(adresse as u16, register as u16))
    }
}

/// Was zuletzt von dort kam, leer wenn noch nichts. Fuer FIRMATA_I2C_REQUEST
/// in der Bildschleife -- die Antwort kommt mit einem spaeteren FIRMATA_UPDATE.
pub fn i2c_data(b: &Board, adresse: i64, register: i64) -> Result<Vec<u8>, String> {
    check_range("FIRMATA_I2C_DATA", "Adresse", adresse, 0, 1023)?;
    check_range("FIRMATA_I2C_DATA", "Register", register, -1, 16383)?;
    Ok(i2c_antwort(b, adresse, register).map(|(_, d)| d.clone()).unwrap_or_default())
}

/// Lesen und auf die Antwort warten (hoechstens eine Sekunde). Bequem fuer
/// Programme ohne Bildschleife; ein Spiel nimmt FIRMATA_I2C_REQUEST.
pub fn i2c_read(b: &mut Board, adresse: i64, register: i64, anzahl: i64) -> Result<Vec<u8>, String> {
    let vorher = b.i2c_zaehler;
    i2c_request(b, "FIRMATA_I2C_READ", adresse, register, anzahl, false)?;
    let frist = Instant::now() + ANTWORT_FRIST;
    loop {
        update(b)?;
        if let Some((nr, d)) = i2c_antwort(b, adresse, register) {
            if *nr > vorher {
                return Ok(d.clone());
            }
        }
        if Instant::now() > frist {
            return Err(format!(
                "FIRMATA_I2C_READ: keine Antwort von Adresse {} (0x{:02X}) -- haengt dort ein Geraet, und laeuft StandardFirmata?",
                adresse, adresse
            ));
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

// ------------------------------------------------------------ Firmware

/// Name und Fassung der Firmware ("StandardFirmata.ino 2.5"). Viele Boards
/// melden sie beim Start von selbst; sonst wird gefragt und gewartet.
pub fn firmware(b: &mut Board) -> Result<String, String> {
    if let Some(f) = &b.firmware {
        return Ok(f.clone());
    }
    update(b)?;
    if b.firmware.is_none() {
        sysex(b, SYSEX_REPORT_FIRMWARE, &[])?;
        let frist = Instant::now() + ANTWORT_FRIST;
        while b.firmware.is_none() {
            if Instant::now() > frist {
                return Err("FIRMATA_FIRMWARE$: keine Antwort -- laeuft Firmata auf dem Board?".to_string());
            }
            std::thread::sleep(Duration::from_millis(1));
            update(b)?;
        }
    }
    Ok(b.firmware.clone().unwrap_or_default())
}

// ------------------------------------------------------------ Schrittmotor

fn stepper_art(art: &str) -> Option<(u8, usize)> {
    match art.trim().to_ascii_lowercase().as_str() {
        "treiber" | "driver" => Some((1, 2)),
        "zweidraht" | "two_wire" | "2" => Some((2, 2)),
        "dreidraht" | "three_wire" | "3" => Some((3, 3)),
        "vierdraht" | "four_wire" | "4" => Some((4, 4)),
        _ => None,
    }
}

pub fn stepper_config(b: &mut Board, nr: i64, art: &str, pins: &[i64]) -> Result<(), String> {
    const F: &str = "FIRMATA_STEPPER_CONFIG";
    check_range(F, "Motor", nr, 0, N_STEPPER as i64 - 1)?;
    let (typ, n) = stepper_art(art).ok_or_else(|| {
        format!("{}: unbekannte Art \"{}\" -- treiber (Schritt + Richtung), zweidraht, dreidraht, vierdraht", F, art)
    })?;
    if pins.len() != n {
        return Err(format!("{}: \"{}\" braucht {} Pins, angegeben sind {}", F, art.trim(), n, pins.len()));
    }
    for &p in pins {
        check_range(F, "Pin", p, 0, 127)?;
    }
    // Schnittstelle: Bits 4-6 die Art, Bits 1-3 die Schrittweite (0 = Vollschritt),
    // Bit 0 = Freigabe-Pin (hier keiner).
    let mut d = vec![STEPPER_CONFIG, nr as u8, typ << 4];
    d.extend(pins.iter().map(|&p| p as u8));
    sysex(b, SYSEX_ACCELSTEPPER, &d)?;
    let i = nr as usize;
    b.stepper_da[i] = true;
    b.stepper_pos[i] = 0;
    b.stepper_laeuft[i] = false;
    Ok(())
}

fn stepper_nr(b: &Board, fn_: &str, nr: i64) -> Result<usize, String> {
    check_range(fn_, "Motor", nr, 0, N_STEPPER as i64 - 1)?;
    if !b.stepper_da[nr as usize] {
        return Err(format!("{}: Motor {} ist nicht eingerichtet -- erst FIRMATA_STEPPER_CONFIG", fn_, nr));
    }
    Ok(nr as usize)
}

const SCHRITT_GRENZE: i64 = (1 << 31) - 1;

/// Relativ (`schritte`, Vorzeichen = Richtung) oder absolut (`ziel`).
pub fn stepper_move(b: &mut Board, nr: i64, schritte: i64, absolut: bool) -> Result<(), String> {
    let f = if absolut { "FIRMATA_STEPPER_TO" } else { "FIRMATA_STEPPER_MOVE" };
    let i = stepper_nr(b, f, nr)?;
    check_range(f, if absolut { "Ziel" } else { "Schritte" }, schritte, -SCHRITT_GRENZE, SCHRITT_GRENZE)?;
    let mut d = vec![if absolut { STEPPER_TO } else { STEPPER_STEP }, i as u8];
    d.extend_from_slice(&zahl32(schritte));
    sysex(b, SYSEX_ACCELSTEPPER, &d)?;
    b.stepper_laeuft[i] = true;
    Ok(())
}

pub fn stepper_zero(b: &mut Board, nr: i64) -> Result<(), String> {
    let i = stepper_nr(b, "FIRMATA_STEPPER_ZERO", nr)?;
    sysex(b, SYSEX_ACCELSTEPPER, &[STEPPER_ZERO, i as u8])?;
    b.stepper_pos[i] = 0;
    Ok(())
}

pub fn stepper_stop(b: &mut Board, nr: i64) -> Result<(), String> {
    let i = stepper_nr(b, "FIRMATA_STEPPER_STOP", nr)?;
    sysex(b, SYSEX_ACCELSTEPPER, &[STEPPER_STOP, i as u8])?;
    b.stepper_laeuft[i] = false;
    Ok(())
}

/// Hoechstgeschwindigkeit (Schritte je Sekunde) bzw. Beschleunigung
/// (Schritte je Sekunde zum Quadrat, 0 = ohne).
pub fn stepper_tempo(b: &mut Board, nr: i64, wert: f64, beschleunigung: bool) -> Result<(), String> {
    let f = if beschleunigung { "FIRMATA_STEPPER_ACCEL" } else { "FIRMATA_STEPPER_SPEED" };
    let i = stepper_nr(b, f, nr)?;
    if !(wert >= 0.0) || (!beschleunigung && wert == 0.0) {
        return Err(format!("{}: {} muss {} sein", f, wert, if beschleunigung { ">= 0" } else { "> 0" }));
    }
    let k = kommazahl(wert).map_err(|e| format!("{}: {}", f, e))?;
    let mut d = vec![if beschleunigung { STEPPER_ACCEL } else { STEPPER_SPEED }, i as u8];
    d.extend_from_slice(&k);
    sysex(b, SYSEX_ACCELSTEPPER, &d)
}

/// Die zuletzt gemeldete Lage; fragt zugleich nach der naechsten, damit ein
/// Programm, das je Bild fragt, mitlaufen sieht.
pub fn stepper_position(b: &mut Board, nr: i64) -> Result<i64, String> {
    let i = stepper_nr(b, "FIRMATA_STEPPER_POSITION", nr)?;
    sysex(b, SYSEX_ACCELSTEPPER, &[STEPPER_REPORT, i as u8])?;
    Ok(b.stepper_pos[i])
}

/// TRUE von FIRMATA_STEPPER_MOVE/TO bis das Board "fertig" meldet.
pub fn stepper_running(b: &Board, nr: i64) -> Result<bool, String> {
    let i = stepper_nr(b, "FIRMATA_STEPPER_RUNNING", nr)?;
    Ok(b.stepper_laeuft[i])
}

// ------------------------------------------------------------ Empfangen

#[derive(Debug, PartialEq, Eq)]
enum Msg {
    Digital { port: usize, byte: u8 },
    Analog { channel: usize, value: i32 },
    /// Befehlsbyte + Daten eines SysEx-Blocks (ohne F0/F7).
    Sysex(Vec<u8>),
}

/// Parst so viele vollstaendige Firmata-Nachrichten wie moeglich vom
/// Puffer-Anfang; gibt die erkannten Nachrichten + den nicht verbrauchten
/// Rest zurueck (fuer den naechsten update()-Aufruf, falls eine Nachricht
/// quer ueber zwei Reads ankam -- gleiches Muster wie serial.rs' `pending`).
/// Nicht abgeschlossene SysEx-Bloecke bleiben fuer das naechste Mal liegen,
/// der Protokoll-Version-Report wird uebersprungen.
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
                Some(rel) => {
                    if rel > 1 {
                        out.push(Msg::Sysex(buf[i + 1..i + rel].to_vec()));
                    }
                    i += rel + 1;
                }
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

/// Ein SysEx-Block vom Board: I2C-Antwort, Firmware, Schrittmotor. Alles
/// andere (Capability-Antworten, Strings) wird uebergangen.
fn sysex_auswerten(b: &mut Board, d: &[u8]) {
    let (befehl, rest) = match d.split_first() {
        Some(x) => x,
        None => return,
    };
    match *befehl {
        SYSEX_I2C_REPLY => {
            let w = paare(rest);
            if w.len() < 2 {
                return;
            }
            let (adresse, register) = (w[0], w[1]);
            let daten: Vec<u8> = w[2..].iter().map(|&x| (x & 0xFF) as u8).collect();
            b.i2c_zaehler += 1;
            let nr = b.i2c_zaehler;
            b.i2c_daten.insert((adresse, register), (nr, daten.clone()));
            b.i2c_letzte.insert(adresse, (nr, daten));
        }
        SYSEX_REPORT_FIRMWARE => {
            if rest.len() < 2 {
                return;
            }
            let name: String = paare(&rest[2..])
                .into_iter()
                .filter_map(|c| char::from_u32(c as u32))
                .filter(|c| *c != '\0')
                .collect();
            let fassung = format!("{}.{}", rest[0], rest[1]);
            b.firmware = Some(if name.is_empty() { fassung } else { format!("{} {}", name, fassung) });
        }
        SYSEX_ACCELSTEPPER => {
            if rest.len() < 7 {
                return;
            }
            let nr = rest[1] as usize;
            if nr >= N_STEPPER {
                return;
            }
            let pos = zahl32_lesen(&rest[2..7]);
            match rest[0] {
                STEPPER_REPORT => b.stepper_pos[nr] = pos,
                STEPPER_DONE => {
                    b.stepper_pos[nr] = pos;
                    b.stepper_laeuft[nr] = false;
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// Liest alle aktuell verfuegbaren Bytes (ohne zu warten) und aktualisiert
/// Pins, I2C-Antworten, Firmware und Schrittmotoren. Pro Frame aufrufen,
/// wie INPUT_UPDATE()/TIMER_UPDATE() -- ohne Aufruf bleiben die Lese-Befehle
/// auf dem letzten Stand stehen.
pub fn update(b: &mut Board) -> Result<(), String> {
    let mut chunk = [0u8; 4096];
    loop {
        let n = b.leitung.lesen(&mut chunk)?;
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
                Msg::Sysex(d) => sysex_auswerten(b, &d),
            }
        }
        if n < chunk.len() {
            return Ok(());
        }
    }
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
    fn parse_returns_complete_sysex_block() {
        let buf = vec![SYSEX_START, 0x79, 2, 3, SYSEX_END, CMD_ANALOG_MESSAGE | 5, 0x00, 0x01];
        let (msgs, rest) = parse_messages(buf);
        assert_eq!(msgs, vec![Msg::Sysex(vec![0x79, 2, 3]), Msg::Analog { channel: 5, value: 128 }]);
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

    #[test]
    fn analog_bis_15_kurz_darueber_extended() {
        assert_eq!(analog_bytes(9, 90), vec![0xE9, 90, 0]);
        assert_eq!(analog_bytes(3, 1023), vec![0xE3, 0x7F, 0x07]);
        assert_eq!(analog_bytes(20, 180), vec![0xF0, 0x6F, 20, 0x34, 0x01, 0xF7]);
    }

    #[test]
    fn zahl32_wie_firmata_js() {
        // firmata.js encode32BitSignedInteger: Betrag + Vorzeichenbit 0x08 im fuenften Byte.
        assert_eq!(zahl32(0), [0, 0, 0, 0, 0]);
        assert_eq!(zahl32(200), [0x48, 0x01, 0, 0, 0]);
        assert_eq!(zahl32(-200), [0x48, 0x01, 0, 0, 0x08]);
        assert_eq!(zahl32(SCHRITT_GRENZE), [0x7F, 0x7F, 0x7F, 0x7F, 0x07]);
        for v in [0, 1, -1, 127, 128, -5000, 123_456_789, -SCHRITT_GRENZE] {
            assert_eq!(zahl32_lesen(&zahl32(v)), v);
        }
    }

    #[test]
    fn kommazahl_wie_decode_custom_float() {
        // 0 sind vier Nullbytes (firmata.js encodeCustomFloat).
        assert_eq!(kommazahl(0.0).unwrap(), [0, 0, 0, 0]);
        for v in [100.0, 0.5, 1234.5, 2000.0, -300.25, 0.001, 1.0 / 3.0, 50_000_000.0] {
            let w = kommazahl_lesen(&kommazahl(v).unwrap());
            assert!((w - v).abs() <= v.abs() * 1e-6, "{} -> {}", v, w);
        }
        // Das Vorzeichen ist Bit 6 des vierten Bytes, der Exponent Bits 2-5.
        let k = kommazahl(-0.5).unwrap();
        assert_eq!(k[3] & 0x40, 0x40);
        assert!(kommazahl(f64::NAN).is_err());
        assert!(kommazahl(1e12).is_err());
    }

    #[test]
    fn i2c_kopf_sieben_und_zehn_bit() {
        assert_eq!(i2c_kopf("T", 0x3C, I2C_WRITE).unwrap(), [0x3C, 0x00]);
        assert_eq!(i2c_kopf("T", 0x68, I2C_READ).unwrap(), [0x68, 0x08]);
        // 10 Bit: 0x2A5 -> unten 0x25, oben Bit 5 + 0b101
        assert_eq!(i2c_kopf("T", 0x2A5, I2C_READ_CONTINUOUSLY).unwrap(), [0x25, 0x10 | 0x20 | 0x05]);
        assert!(i2c_kopf("T", 1024, I2C_WRITE).is_err());
    }

    #[test]
    fn netzziel_zerlegen() {
        assert_eq!(tcp_ziel("tcp:192.168.1.50:3030"), Some(("192.168.1.50".to_string(), 3030)));
        assert_eq!(tcp_ziel("TCP:esp32.local:3030"), Some(("esp32.local".to_string(), 3030)));
        assert_eq!(tcp_ziel("tcp:[::1]:3030"), Some(("::1".to_string(), 3030)));
        assert_eq!(tcp_ziel("tcp:host"), None);
        assert_eq!(tcp_ziel("tcp::3030"), None);
        assert_eq!(tcp_ziel("COM3"), None);
    }
}
