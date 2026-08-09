//! Serielle Schnittstelle (SERIAL_*) -- nativer Port von
//! `drachenhauch/modules/serial.py` via `serialport`. Feature `serial`.
//! SERIAL_HANDLE = INTEGER-Index in einer VM-Vec.
#![cfg(feature = "serial")]

use std::io::{Read, Write};
use std::time::Duration;

use serialport::SerialPort;

/// Obergrenze fuer SERIAL_READ(port, n) -- ohne Cap loest ein versehentlich
/// riesiges `n` (Tippfehler, vertauschte Parameter) eine Multi-GB-Allokation
/// aus, die bei Fehlschlag den Prozess abbricht statt einen fangbaren Fehler
/// zu liefern (dasselbe Muster wie bei net.rs/usb.rs).
const MAX_READ_BYTES: i64 = 64 * 1024 * 1024;

pub struct Port {
    conn: Box<dyn SerialPort>,
    /// Unvollstaendiger Rest eines Mehrbyte-UTF-8-Zeichens vom letzten READ,
    /// das quer ueber zwei Reads verteilt ankam (gleiches Muster wie
    /// `net::NetSock::pending` -- siehe `crate::net::decode_utf8_streaming`
    /// fuer die Begruendung).
    pending: Vec<u8>,
}

pub fn ports() -> String {
    match serialport::available_ports() {
        Ok(list) => list.iter().map(|p| p.port_name.clone()).collect::<Vec<_>>().join(", "),
        Err(_) => String::new(),
    }
}

pub fn open(port: &str, baud: i64) -> Result<Port, String> {
    let conn = serialport::new(port, baud as u32)
        .timeout(Duration::from_secs(1))
        .open()
        .map_err(|e| format!("SERIAL_OPEN: {}", e))?;
    Ok(Port { conn, pending: Vec::new() })
}

pub fn write(p: &mut Port, s: &str) -> Result<i64, String> {
    p.conn.write(s.as_bytes()).map(|n| n as i64).map_err(|e| format!("SERIAL_WRITE: {}", e))
}

pub fn read(p: &mut Port, n: i64) -> Result<String, String> {
    if n < 0 {
        return Err("SERIAL_READ: Anzahl muss >= 0 sein".into());
    }
    if n > MAX_READ_BYTES {
        return Err(format!("SERIAL_READ: Anzahl {} ueberschreitet das Limit von {} Byte", n, MAX_READ_BYTES));
    }
    let mut buf = vec![0u8; n as usize];
    match p.conn.read(&mut buf) {
        Ok(got) => {
            let mut data = std::mem::take(&mut p.pending);
            data.extend_from_slice(&buf[..got]);
            let (text, leftover) = crate::text_stream::decode_utf8_streaming(data);
            p.pending = leftover;
            Ok(text)
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(String::new()),
        Err(e) => Err(format!("SERIAL_READ: {}", e)),
    }
}

pub fn readline(p: &mut Port) -> Result<String, String> {
    let mut out: Vec<u8> = std::mem::take(&mut p.pending);
    let mut byte = [0u8; 1];
    loop {
        match p.conn.read(&mut byte) {
            Ok(0) => break,
            Ok(_) => {
                out.push(byte[0]);
                if byte[0] == b'\n' {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break,
            Err(e) => return Err(format!("SERIAL_READLINE: {}", e)),
        }
    }
    // Anders als SERIAL_READ liest READLINE bis zu einem Zeilenende (oder
    // Timeout) durch -- ein an einer Zwischenpause zerschnittenes Zeichen ist
    // hier kein Dauerzustand, daher genuegt eine einmalige lossy Dekodierung
    // OHNE Rest fuer den naechsten Aufruf aufzuheben (sonst wuerde ein echt
    // kaputtes letztes Byte vor einem Timeout den naechsten READLINE-Aufruf
    // unbemerkt verunreinigen).
    Ok(String::from_utf8_lossy(&out).into_owned())
}

pub fn available(p: &Port) -> Result<i64, String> {
    p.conn.bytes_to_read().map(|n| n as i64).map_err(|e| format!("SERIAL_AVAILABLE: {}", e))
}

pub fn flush(p: &Port) {
    let _ = p.conn.clear(serialport::ClearBuffer::All);
}

pub fn set_timeout(p: &mut Port, secs: f64) {
    // secs.max(0.0) faengt NaN ab (Rust: max() mit NaN liefert den anderen
    // Operanden), aber NICHT +Infinity -- Duration::from_secs_f64 dort wuerde
    // panicen (gleiche Bug-Klasse wie BT_SCAN mit NaN/Infinity-Timeout).
    // Review-Fund: das faengt auch NUR NaN/Infinity ab, nicht einen riesigen
    // aber ENDLICHEN Wert (z.B. 1e300) -- Duration::from_secs_f64 paniked
    // auch dafuer ("value is either too big or NaN"). 1h ist bereits weit
    // jenseits jedes sinnvollen Timeouts, deckelt aber diesen Fall mit ab.
    let clamped = if secs.is_finite() { secs.clamp(0.0, 3600.0) } else { 0.0 };
    let _ = p.conn.set_timeout(Duration::from_secs_f64(clamped));
}
