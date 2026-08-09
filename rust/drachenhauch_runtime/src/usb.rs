//! USB-HID (USB_*) -- nativer Port von `gamebasic/modules/usb.py` via
//! `hidapi`. Feature `usb`. USB_HANDLE = INTEGER-Index in einer VM-Vec.
//! Bytes <-> STRING via latin-1 (Codepoint 0..255 == ein Byte).
#![cfg(feature = "usb")]

use std::sync::{Mutex, OnceLock};

use hidapi::{HidApi, HidDevice};

/// `Mutex` statt `OnceLock<HidApi>` (frueher), weil `USB_LIST()` neu
/// angesteckte/entfernte Geraete nur sieht, wenn `refresh_devices(&mut self)`
/// aufgerufen werden kann -- ueber `OnceLock::get()` gibt es nie eine `&mut
/// HidApi`, `USB_LIST()` haette also strukturell fuer immer den Snapshot vom
/// allerersten Aufruf geliefert.
static API: OnceLock<Mutex<HidApi>> = OnceLock::new();

fn api() -> Result<&'static Mutex<HidApi>, String> {
    if let Some(a) = API.get() {
        return Ok(a);
    }
    let a = HidApi::new().map_err(|e| format!("USB: hidapi-Init fehlgeschlagen: {}", e))?;
    let _ = API.set(Mutex::new(a));
    API.get().ok_or_else(|| "USB: hidapi nicht verfuegbar".into())
}

pub fn latin1_encode(s: &str, fn_: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(s.len());
    for c in s.chars() {
        let cp = c as u32;
        if cp > 0xFF {
            return Err(format!("{}: STRING enthaelt Zeichen > 0xFF. Nur Bytes 0..255 sind erlaubt (latin-1).", fn_));
        }
        out.push(cp as u8);
    }
    Ok(out)
}

pub fn latin1_decode(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

pub fn list() -> Result<String, String> {
    let mutex = api()?;
    let mut a = mutex.lock().map_err(|_| "USB: hidapi-Lock vergiftet".to_string())?;
    // Ohne Refresh bliebe das immer der Geraete-Snapshot vom allerersten
    // USB_LIST()/USB_OPEN()-Aufruf -- neu angesteckte Geraete wuerden nie
    // auftauchen, entfernte nie verschwinden.
    a.refresh_devices().map_err(|e| format!("USB_LIST: {}", e))?;
    let mut lines = Vec::new();
    for d in a.device_list() {
        let prod = d.product_string().unwrap_or("");
        let manu = d.manufacturer_string().unwrap_or("");
        lines.push(format!("{:04X}:{:04X}|{}|{}", d.vendor_id(), d.product_id(), prod, manu));
    }
    Ok(lines.join("\n"))
}

pub fn open(vid: i64, pid: i64) -> Result<HidDevice, String> {
    // Review-Fund: `vid as u16`/`pid as u16` truncated einen ausserhalb
    // 0..=0xFFFF liegenden Wert lautlos -- USB_OPEN(&H10000, &H1234) haette
    // VID 0x0000 versucht (moeglicherweise ein VOELLIG ANDERES Geraet
    // getroffen), waehrend die Fehlermeldung im Misserfolgsfall trotzdem die
    // UNGETRUNCATEN Werte zeigte ("10000:1234 nicht gefunden") -- irrefuehrend
    // beim Debuggen, weil diese IDs nie tatsaechlich angefragt wurden.
    if !(0..=0xFFFF).contains(&vid) || !(0..=0xFFFF).contains(&pid) {
        return Err(format!("USB_OPEN: VID/PID muessen 0..0xFFFF sein (erhalten {:X}:{:X})", vid, pid));
    }
    let mutex = api()?;
    let a = mutex.lock().map_err(|_| "USB: hidapi-Lock vergiftet".to_string())?;
    a.open(vid as u16, pid as u16)
        .map_err(|e| format!("USB_OPEN: {:04X}:{:04X} nicht gefunden oder belegt ({})", vid, pid, e))
}

pub fn open_path(path: &str) -> Result<HidDevice, String> {
    let c = std::ffi::CString::new(path).map_err(|_| "USB_OPEN_PATH: ungueltiger Pfad".to_string())?;
    let mutex = api()?;
    let a = mutex.lock().map_err(|_| "USB: hidapi-Lock vergiftet".to_string())?;
    a.open_path(&c)
        .map_err(|e| format!("USB_OPEN_PATH: '{}' nicht erreichbar ({})", path, e))
}

pub fn write(d: &HidDevice, data: &str) -> Result<i64, String> {
    let bytes = latin1_encode(data, "USB_WRITE")?;
    d.write(&bytes).map(|n| n as i64).map_err(|e| format!("USB_WRITE: {}", e))
}

/// Obergrenze fuer USB_READ(dev, n, ...) -- ohne Cap loest ein versehentlich
/// riesiges `n` (Tippfehler, vertauschte Parameter mit timeout_ms) eine
/// Multi-GB-Allokation aus, die bei Fehlschlag den Prozess abbricht statt
/// einen fangbaren Fehler zu liefern.
const MAX_READ_BYTES: i64 = 64 * 1024 * 1024;

pub fn read(d: &HidDevice, n: i64, timeout_ms: i64) -> Result<String, String> {
    if n < 0 {
        return Err("USB_READ: Anzahl muss >= 0 sein".into());
    }
    if n > MAX_READ_BYTES {
        return Err(format!("USB_READ: Anzahl {} ueberschreitet das Limit von {} Byte", n, MAX_READ_BYTES));
    }
    let mut buf = vec![0u8; n as usize];
    // `timeout_ms as i32` waere ein truncating Cast -- ein Wert > i32::MAX
    // (~24,8 Tage in ms) wuerde unbemerkt ins Negative kippen und
    // ungewollt "blockiere fuer immer" ausloesen. Stattdessen clampen.
    let timeout_i32 = timeout_ms.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let got = d.read_timeout(&mut buf, timeout_i32).map_err(|e| format!("USB_READ: {}", e))?;
    Ok(latin1_decode(&buf[..got]))
}

pub fn product(d: &HidDevice) -> String {
    d.get_product_string().ok().flatten().unwrap_or_default()
}
pub fn manufacturer(d: &HidDevice) -> String {
    d.get_manufacturer_string().ok().flatten().unwrap_or_default()
}
pub fn serial(d: &HidDevice) -> String {
    d.get_serial_number_string().ok().flatten().unwrap_or_default()
}
