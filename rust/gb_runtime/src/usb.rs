//! USB-HID (USB_*) -- nativer Port von `gamebasic/modules/usb.py` via
//! `hidapi`. Feature `usb`. USB_HANDLE = INTEGER-Index in einer VM-Vec.
//! Bytes <-> STRING via latin-1 (Codepoint 0..255 == ein Byte).
#![cfg(feature = "usb")]

use std::sync::OnceLock;

use hidapi::{HidApi, HidDevice};

static API: OnceLock<HidApi> = OnceLock::new();

fn api() -> Result<&'static HidApi, String> {
    if let Some(a) = API.get() {
        return Ok(a);
    }
    let a = HidApi::new().map_err(|e| format!("USB: hidapi-Init fehlgeschlagen: {}", e))?;
    let _ = API.set(a);
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
    let a = api()?;
    let mut lines = Vec::new();
    for d in a.device_list() {
        let prod = d.product_string().unwrap_or("");
        let manu = d.manufacturer_string().unwrap_or("");
        lines.push(format!("{:04X}:{:04X}|{}|{}", d.vendor_id(), d.product_id(), prod, manu));
    }
    Ok(lines.join("\n"))
}

pub fn open(vid: i64, pid: i64) -> Result<HidDevice, String> {
    api()?.open(vid as u16, pid as u16)
        .map_err(|e| format!("USB_OPEN: {:04X}:{:04X} nicht gefunden oder belegt ({})", vid, pid, e))
}

pub fn open_path(path: &str) -> Result<HidDevice, String> {
    let c = std::ffi::CString::new(path).map_err(|_| "USB_OPEN_PATH: ungueltiger Pfad".to_string())?;
    api()?.open_path(&c)
        .map_err(|e| format!("USB_OPEN_PATH: '{}' nicht erreichbar ({})", path, e))
}

pub fn write(d: &HidDevice, data: &str) -> Result<i64, String> {
    let bytes = latin1_encode(data, "USB_WRITE")?;
    d.write(&bytes).map(|n| n as i64).map_err(|e| format!("USB_WRITE: {}", e))
}

pub fn read(d: &HidDevice, n: i64, timeout_ms: i64) -> Result<String, String> {
    if n < 0 {
        return Err("USB_READ: Anzahl muss >= 0 sein".into());
    }
    let mut buf = vec![0u8; n as usize];
    let got = d.read_timeout(&mut buf, timeout_ms as i32).map_err(|e| format!("USB_READ: {}", e))?;
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
