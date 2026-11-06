//! Modul `hotplug` -- melden, wenn ein Geraet angesteckt oder abgezogen wird:
//! serielle Anschluesse (ein Arduino am USB), MIDI-Ein- und -Ausgaenge und
//! USB-HID-Geraete -- je nachdem, was dieser Bau kennt (`HOTPLUG_KINDS`).
//!
//! **Abgefragt, nicht benachrichtigt.** Jedes System meldet das auf seine
//! Weise (WM_DEVICECHANGE, udev, IOKit), und keine davon kennen alle drei
//! Arten. Darum sieht ein Faden alle 500 ms (`HOTPLUG_INTERVAL`) in dieselben
//! Listen, die auch SERIAL_PORTS, MIDI_IN_NAME$ und USB_LIST liefern, und
//! vergleicht mit dem Stand davor. Was beim ersten Blick schon da ist, gilt
//! als Ausgangslage und wird nicht gemeldet -- dafuer ist `HOTPLUG_LIST`.
//!
//! **Pruefbar ohne Geraet:** `DHRT_GERAETE=datei` liest die Listen statt vom
//! System aus einer Textdatei (`art:name` je Zeile) -- eine Pruefung aendert
//! die Datei und sieht die Meldungen.

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

/// Wie viele ungelesene Meldungen hoechstens warten; dann faellt die aelteste.
const MAX_WARTEND: usize = 256;

pub struct Ereignis {
    pub art: String,
    pub name: String,
    pub dazu: bool,
}

struct Waechter {
    warteschlange: VecDeque<Ereignis>,
    stand: Vec<(&'static str, Vec<String>)>,
    intervall_ms: u64,
}

static W: OnceLock<Mutex<Waechter>> = OnceLock::new();

fn simuliert() -> Option<String> {
    std::env::var("DHRT_GERAETE").ok().filter(|s| !s.is_empty())
}

/// Die Arten, die dieser Bau beobachten kann.
pub fn arten() -> Vec<&'static str> {
    if simuliert().is_some() { return vec!["serial", "midi_in", "midi_out", "hid"]; }
    let mut v = Vec::new();
    if cfg!(feature = "serial") { v.push("serial"); }
    if cfg!(feature = "midi") { v.push("midi_in"); v.push("midi_out"); }
    if cfg!(feature = "usb") { v.push("hid"); }
    v
}

/// Was in `alt` fehlt und in `neu` dazukam -- beides in der Reihenfolge von
/// `neu` bzw. `alt`.
pub fn unterschied(alt: &[String], neu: &[String]) -> (Vec<String>, Vec<String>) {
    let weg = alt.iter().filter(|n| !neu.contains(n)).cloned().collect();
    let dazu = neu.iter().filter(|n| !alt.contains(n)).cloned().collect();
    (weg, dazu)
}

/// Eine Liste ordnen und Doppelte weglassen -- ein HID-Geraet meldet sich
/// mit jeder Schnittstelle einzeln, und eine Tastatur waere sonst dreimal
/// angesteckt worden.
pub fn ordnen(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v.dedup();
    v
}

fn liste(art: &str) -> Vec<String> {
    if let Some(datei) = simuliert() {
        let text = std::fs::read_to_string(&datei).unwrap_or_default();
        let v = text.lines()
            .filter_map(|z| z.trim().split_once(':'))
            .filter(|(a, _)| a.trim() == art)
            .map(|(_, n)| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .collect();
        return ordnen(v);
    }
    let v: Vec<String> = match art {
        #[cfg(feature = "serial")]
        "serial" => serialport::available_ports().map(|l| l.into_iter().map(|p| p.port_name).collect()).unwrap_or_default(),
        #[cfg(feature = "midi")]
        "midi_in" => midir::MidiInput::new("Drachenhauch-hotplug")
            .map(|m| m.ports().iter().filter_map(|p| m.port_name(p).ok()).collect()).unwrap_or_default(),
        #[cfg(feature = "midi")]
        "midi_out" => midir::MidiOutput::new("Drachenhauch-hotplug")
            .map(|m| m.ports().iter().filter_map(|p| m.port_name(p).ok()).collect()).unwrap_or_default(),
        #[cfg(feature = "usb")]
        "hid" => crate::usb::list().unwrap_or_default().lines().map(|z| {
            let t: Vec<&str> = z.split('|').collect();
            match t.get(1).map(|s| s.trim()).filter(|s| !s.is_empty()) {
                Some(p) => format!("{} {}", t[0], p),
                None => t[0].to_string(),
            }
        }).collect(),
        _ => Vec::new(),
    };
    ordnen(v)
}

fn waechter() -> &'static Mutex<Waechter> {
    W.get_or_init(|| {
        let stand = arten().into_iter().map(|a| (a, liste(a))).collect();
        let w = Mutex::new(Waechter { warteschlange: VecDeque::new(), stand, intervall_ms: 500 });
        std::thread::Builder::new()
            .name("drachenhauch-hotplug".into())
            .spawn(schauen)
            .ok();
        w
    })
}

fn schauen() {
    loop {
        let ms = W.get().and_then(|w| w.lock().ok().map(|w| w.intervall_ms)).unwrap_or(500);
        std::thread::sleep(Duration::from_millis(ms));
        let Some(w) = W.get() else { continue };
        // Erst ausserhalb der Sperre lesen (die Listen brauchen Millisekunden),
        // dann vergleichen.
        let arten: Vec<&'static str> = match w.lock() { Ok(g) => g.stand.iter().map(|(a, _)| *a).collect(), Err(_) => return };
        let neu: Vec<(&'static str, Vec<String>)> = arten.into_iter().map(|a| (a, liste(a))).collect();
        let Ok(mut g) = w.lock() else { return };
        for (art, liste) in neu {
            let Some(k) = g.stand.iter().position(|(a, _)| *a == art) else { continue };
            let (weg, dazu) = unterschied(&g.stand[k].1, &liste);
            for n in weg {
                if g.warteschlange.len() >= MAX_WARTEND { g.warteschlange.pop_front(); }
                g.warteschlange.push_back(Ereignis { art: art.to_string(), name: n, dazu: false });
            }
            for n in dazu {
                if g.warteschlange.len() >= MAX_WARTEND { g.warteschlange.pop_front(); }
                g.warteschlange.push_back(Ereignis { art: art.to_string(), name: n, dazu: true });
            }
            g.stand[k].1 = liste;
        }
    }
}

pub fn naechstes() -> Option<Ereignis> {
    waechter().lock().ok().and_then(|mut w| w.warteschlange.pop_front())
}

pub fn aktuelle(art: &str) -> Result<Vec<String>, String> {
    let w = waechter().lock().map_err(|_| "HOTPLUG_LIST: Sperre vergiftet".to_string())?;
    match w.stand.iter().find(|(a, _)| *a == art) {
        Some((_, l)) => Ok(l.clone()),
        None => Err(format!("HOTPLUG_LIST: die Art {:?} beobachtet diese Laufzeit nicht -- moeglich: {}",
                            art, if w.stand.is_empty() { "keine (gebaut ohne --hardware)".to_string() }
                                 else { w.stand.iter().map(|(a, _)| *a).collect::<Vec<_>>().join(", ") })),
    }
}

pub fn intervall(ms: i64) -> Result<(), String> {
    if !(100..=10000).contains(&ms) { return Err(format!("HOTPLUG_INTERVAL: {} ms ausserhalb 100..10000", ms)); }
    if let Ok(mut w) = waechter().lock() { w.intervall_ms = ms as u64; }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> { v.iter().map(|x| x.to_string()).collect() }

    #[test]
    fn unterschied_in_beide_richtungen() {
        let (weg, dazu) = unterschied(&s(&["COM3", "COM4"]), &s(&["COM4", "COM7"]));
        assert_eq!(weg, s(&["COM3"]));
        assert_eq!(dazu, s(&["COM7"]));
        let (weg, dazu) = unterschied(&s(&["a"]), &s(&["a"]));
        assert!(weg.is_empty() && dazu.is_empty());
    }

    #[test]
    fn doppelte_zaehlen_einmal() {
        assert_eq!(ordnen(s(&["046D:C52B Receiver", "046D:C52B Receiver", "0001:0002"])),
                   s(&["0001:0002", "046D:C52B Receiver"]));
    }
}
