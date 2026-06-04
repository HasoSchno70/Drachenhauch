//! Serielle Schnittstelle (SERIAL_*) -- nativer Port von
//! `gamebasic/modules/serial.py` via `serialport`. Feature `serial`.
//! SERIAL_HANDLE = INTEGER-Index in einer VM-Vec.
#![cfg(feature = "serial")]

use std::io::{Read, Write};
use std::time::Duration;

use serialport::SerialPort;

pub type Port = Box<dyn SerialPort>;

pub fn ports() -> String {
    match serialport::available_ports() {
        Ok(list) => list.iter().map(|p| p.port_name.clone()).collect::<Vec<_>>().join(", "),
        Err(_) => String::new(),
    }
}

pub fn open(port: &str, baud: i64) -> Result<Port, String> {
    serialport::new(port, baud as u32)
        .timeout(Duration::from_secs(1))
        .open()
        .map_err(|e| format!("SERIAL_OPEN: {}", e))
}

pub fn write(p: &mut Port, s: &str) -> Result<i64, String> {
    p.write(s.as_bytes()).map(|n| n as i64).map_err(|e| format!("SERIAL_WRITE: {}", e))
}

pub fn read(p: &mut Port, n: i64) -> Result<String, String> {
    if n < 0 {
        return Err("SERIAL_READ: Anzahl muss >= 0 sein".into());
    }
    let mut buf = vec![0u8; n as usize];
    match p.read(&mut buf) {
        Ok(got) => Ok(String::from_utf8_lossy(&buf[..got]).into_owned()),
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(String::new()),
        Err(e) => Err(format!("SERIAL_READ: {}", e)),
    }
}

pub fn readline(p: &mut Port) -> Result<String, String> {
    let mut out: Vec<u8> = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match p.read(&mut byte) {
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
    Ok(String::from_utf8_lossy(&out).into_owned())
}

pub fn available(p: &Port) -> Result<i64, String> {
    p.bytes_to_read().map(|n| n as i64).map_err(|e| format!("SERIAL_AVAILABLE: {}", e))
}

pub fn flush(p: &Port) {
    let _ = p.clear(serialport::ClearBuffer::All);
}

pub fn set_timeout(p: &mut Port, secs: f64) {
    let _ = p.set_timeout(Duration::from_secs_f64(secs.max(0.0)));
}
