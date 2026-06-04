//! Bluetooth Low Energy (BT_*) -- nativer Port von `gamebasic/modules/bt.py`
//! via `btleplug` (async). Feature `bt`. BT_HANDLE = INTEGER-Index in einer
//! VM-Vec. Async wird ueber eine globale tokio-Runtime synchron getrieben
//! (jeder Aufruf blockiert bis zur Antwort). Bytes <-> STRING via latin-1.
#![cfg(feature = "bt")]

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use btleplug::api::{Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::runtime::Runtime;

struct BtGlobal {
    rt: Runtime,
    adapter: Adapter,
}

fn bt() -> Result<&'static BtGlobal, String> {
    static G: OnceLock<BtGlobal> = OnceLock::new();
    if let Some(g) = G.get() {
        return Ok(g);
    }
    let rt = Runtime::new().map_err(|e| format!("BT: Runtime: {}", e))?;
    let adapter = rt.block_on(async {
        let mgr = Manager::new().await.map_err(|e| format!("BT: Manager: {}", e))?;
        let mut adapters = mgr.adapters().await.map_err(|e| format!("BT: adapters: {}", e))?;
        if adapters.is_empty() {
            return Err("BT: kein Bluetooth-Adapter gefunden".to_string());
        }
        Ok::<Adapter, String>(adapters.remove(0))
    })?;
    let _ = G.set(BtGlobal { rt, adapter });
    G.get().ok_or_else(|| "BT: Init".into())
}

// Letzte Scan-Ergebnisse (Adresse -> Peripheral) fuer BT_CONNECT.
fn seen() -> &'static Mutex<HashMap<String, Peripheral>> {
    static S: OnceLock<Mutex<HashMap<String, Peripheral>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
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

fn char_props(f: CharPropFlags) -> String {
    let mut v = Vec::new();
    if f.contains(CharPropFlags::BROADCAST) { v.push("broadcast"); }
    if f.contains(CharPropFlags::READ) { v.push("read"); }
    if f.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) { v.push("write-without-response"); }
    if f.contains(CharPropFlags::WRITE) { v.push("write"); }
    if f.contains(CharPropFlags::NOTIFY) { v.push("notify"); }
    if f.contains(CharPropFlags::INDICATE) { v.push("indicate"); }
    if f.contains(CharPropFlags::AUTHENTICATED_SIGNED_WRITES) { v.push("authenticated-signed-writes"); }
    if f.contains(CharPropFlags::EXTENDED_PROPERTIES) { v.push("extended-properties"); }
    v.join(",")
}

async fn find_char(p: &Peripheral, uuid: &str) -> Option<Characteristic> {
    let _ = p.discover_services().await;
    p.characteristics().into_iter().find(|c| c.uuid.to_string().eq_ignore_ascii_case(uuid))
}

pub fn scan(timeout_s: f64) -> Result<String, String> {
    if timeout_s < 0.0 {
        return Err("BT_SCAN: Timeout muss >= 0 sein".into());
    }
    let g = bt()?;
    g.rt.block_on(async {
        g.adapter.start_scan(ScanFilter::default()).await.map_err(|e| format!("BT_SCAN: {}", e))?;
        tokio::time::sleep(Duration::from_secs_f64(timeout_s)).await;
        let peris = g.adapter.peripherals().await.map_err(|e| format!("BT_SCAN: {}", e))?;
        let mut rows = Vec::new();
        let mut map = seen().lock().unwrap();
        for p in peris {
            let addr = p.address().to_string();
            let props = p.properties().await.ok().flatten();
            let (name, rssi) = match props {
                Some(pr) => (pr.local_name.unwrap_or_default(), pr.rssi.unwrap_or(0)),
                None => (String::new(), 0),
            };
            map.insert(addr.clone(), p.clone());
            rows.push(format!("{}|{}|{}", addr, if name.is_empty() { "?".to_string() } else { name }, rssi));
        }
        Ok(rows.join("\n"))
    })
}

pub fn connect(addr: &str) -> Result<Peripheral, String> {
    let g = bt()?;
    let p = seen().lock().unwrap().get(addr).cloned()
        .ok_or_else(|| format!("BT_CONNECT: '{}' nicht gefunden - erst BT_SCAN aufrufen", addr))?;
    g.rt.block_on(async {
        p.connect().await.map_err(|e| format!("BT_CONNECT: {}", e))?;
        if !p.is_connected().await.unwrap_or(false) {
            return Err(format!("BT_CONNECT: Konnte {} nicht verbinden", addr));
        }
        Ok::<(), String>(())
    })?;
    Ok(p)
}

pub fn disconnect(p: &Peripheral) -> Result<(), String> {
    let g = bt()?;
    g.rt.block_on(async {
        if p.is_connected().await.unwrap_or(false) {
            let _ = p.disconnect().await;
        }
    });
    Ok(())
}

pub fn is_connected(p: &Peripheral) -> bool {
    match bt() {
        Ok(g) => g.rt.block_on(p.is_connected()).unwrap_or(false),
        Err(_) => false,
    }
}

pub fn services(p: &Peripheral) -> Result<String, String> {
    let g = bt()?;
    g.rt.block_on(async {
        p.discover_services().await.map_err(|e| format!("BT_SERVICES: {}", e))?;
        Ok(p.services().iter().map(|s| s.uuid.to_string()).collect::<Vec<_>>().join("\n"))
    })
}

pub fn characteristics(p: &Peripheral, svc: &str) -> Result<String, String> {
    let g = bt()?;
    g.rt.block_on(async {
        p.discover_services().await.map_err(|e| format!("BT_CHARACTERISTICS: {}", e))?;
        let mut out = Vec::new();
        for s in p.services() {
            if !s.uuid.to_string().eq_ignore_ascii_case(svc) {
                continue;
            }
            for c in s.characteristics {
                out.push(format!("{}|{}", c.uuid, char_props(c.properties)));
            }
        }
        Ok(out.join("\n"))
    })
}

pub fn read(p: &Peripheral, char_uuid: &str) -> Result<String, String> {
    let g = bt()?;
    g.rt.block_on(async {
        let ch = find_char(p, char_uuid).await
            .ok_or_else(|| format!("BT_READ: Characteristic '{}' nicht gefunden", char_uuid))?;
        let data = p.read(&ch).await.map_err(|e| format!("BT_READ: {}", e))?;
        Ok(latin1_decode(&data))
    })
}

pub fn write(p: &Peripheral, char_uuid: &str, payload: &str) -> Result<(), String> {
    let bytes = latin1_encode(payload, "BT_WRITE")?;
    let g = bt()?;
    g.rt.block_on(async {
        let ch = find_char(p, char_uuid).await
            .ok_or_else(|| format!("BT_WRITE: Characteristic '{}' nicht gefunden", char_uuid))?;
        let wt = if ch.properties.contains(CharPropFlags::WRITE) {
            WriteType::WithResponse
        } else {
            WriteType::WithoutResponse
        };
        p.write(&ch, &bytes, wt).await.map_err(|e| format!("BT_WRITE: {}", e))
    })
}
