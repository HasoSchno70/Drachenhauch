//! Die Typen, die nicht zur Sprache selbst gehoeren: eingebaute Objekte
//! (IMAGE, FILE ...) und die Typen der Module. EINE Tafel, aus der lesen:
//!
//! - der Praeprozessor, welches `IMPORT` welchen Typ bringt
//!   (`preprocess::modul_typen`, `modules_for_type`, `compile_env`),
//! - `handle.rs`, welche Typen Handles mit Art sind (`Value::Handle`),
//! - `coerce`/`IS` (`value::wert_typ`), welche echte Werte sind.
//!
//! Bis 2026.53 waren das drei Listen (`MODULE_TYPES`, `handle::ARTEN`,
//! `value::WERT_TYPEN`), die nur ein Test zusammenhielt (Bericht D5). Ein
//! neuer Modultyp gehoert hierher und nirgends sonst.

/// Was ein Wert dieses Typs zur Laufzeit ist.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Art {
    /// Ein eigener Wert (`Value::Json`, `Value::Vec2` ...): eine Variable
    /// des Typs nimmt nur seinesgleichen (oder NIL).
    Wert,
    /// Eine Nummer mit Art (`Value::Handle`, handle.rs).
    Handle,
    /// Nur ein anderer Name fuer eine Zahl: AUDIO_CHANNEL ist die Nummer
    /// des Klangs, der spielt (`audio::ch_play`).
    Zahl,
}

/// Ein Typ: Name (wie `TYPEOF` ihn nennt), die Module, die ihn bringen
/// (leer = ohne `IMPORT`), und seine Art.
pub struct Typ {
    pub name: &'static str,
    pub module: &'static [&'static str],
    pub art: Art,
}

const fn t(name: &'static str, module: &'static [&'static str], art: Art) -> Typ {
    Typ { name, module, art }
}

use Art::{Handle as H, Wert as W, Zahl as Z};

pub const TYPEN: &[Typ] = &[
    // Ohne IMPORT
    t("IMAGE", &[], H), t("SOUND", &[], H), t("SPRITE_ATLAS", &[], H),
    t("FILE", &[], W), t("BUFFER", &[], W),
    // Module
    t("ANIM_FSM", &["animfsm"], W),
    t("ASTAR_GRID", &["astar"], W),
    t("CHART", &["chart"], W),
    t("AUDIO_CHANNEL", &["audio"], Z),
    t("SAMPLE", &["audio"], H), t("AUDIO_CLOCK", &["audio"], H), t("AUDIO_LISTENER", &["audio"], H),
    t("AUDIO_EMITTER", &["audio"], H), t("AUDIO_MOD", &["audio"], H),
    t("BT_HANDLE", &["bt"], H),
    t("CHAR_CONTROLLER", &["controller"], W),
    t("TILED_MAP", &["controller", "tile_collide", "tiled"], W),
    t("DB_CONN", &["db"], H), t("DB_RESULT", &["db"], H),
    t("ECS_WORLD", &["ecs"], W),
    t("FIRMATA_HANDLE", &["firmata"], H),
    t("GPIO_CHIP", &["gpio"], H), t("I2C_BUS", &["i2c"], H), t("SPI_DEVICE", &["spi"], H),
    t("WEBCAM", &["webcam"], H),
    t("WEBSOCKET", &["websocket"], H), t("WS_SERVER", &["websocket"], H),
    t("OSC", &["osc"], H),
    t("GUI_WIDGET", &["gui"], H), t("GUI_WINDOW", &["gui"], H),
    t("JSON_HANDLE", &["json"], W),
    t("VEC3", &["m3d"], W), t("VEC4", &["m3d"], W), t("QUAT", &["m3d"], W), t("MAT4", &["m3d"], W),
    t("MQTT_HANDLE", &["mqtt"], H),
    t("HTTPD", &["httpd"], H),
    t("XML_HANDLE", &["xml"], W),
    t("PDF", &["pdf"], H),
    t("GELD", &["geld"], W),
    t("SMTP", &["smtp"], H),
    t("XLSX", &["xlsx"], H),
    t("NET_LISTENER", &["net"], H), t("NET_SOCKET", &["net"], H), t("NET_UDP", &["net"], H),
    t("PARTICLE_SYSTEM", &["particles"], W),
    t("PHYSICS_BROAD", &["physics"], W),
    t("PHYS2D_WORLD", &["physics2d"], W),
    t("PHYS_WORLD", &["physics3d"], W),
    t("SAVE_HANDLE", &["save"], W),
    t("MIDI_IN", &["midi"], H), t("MIDI_OUT", &["midi"], H), t("MIDI_FILE", &["midi"], H),
    t("MODBUS", &["modbus"], H),
    t("SERIAL_HANDLE", &["serial"], H),
    t("SPRITE", &["sprite"], W),
    t("TWEEN", &["tween"], W),
    t("USB_HANDLE", &["usb"], H),
    t("VEC2", &["vec2"], W),
    t("VIDEO", &["video"], H),
];

/// Der Eintrag zu einem Typnamen (jede Schreibweise).
pub fn typ(name: &str) -> Option<&'static Typ> {
    TYPEN.iter().find(|t| t.name.eq_ignore_ascii_case(name))
}

/// Die Typen einer Art, in der Reihenfolge der Tafel.
pub fn der_art(art: Art) -> impl Iterator<Item = &'static Typ> {
    TYPEN.iter().filter(move |t| t.art == art)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jeder_name_einmal_und_gross() {
        for (i, a) in TYPEN.iter().enumerate() {
            assert_eq!(a.name, a.name.to_uppercase(), "{}", a.name);
            assert!(TYPEN[i + 1..].iter().all(|b| b.name != a.name), "{} doppelt", a.name);
            for m in a.module {
                assert_eq!(*m, m.to_lowercase(), "{}: Modul {}", a.name, m);
            }
        }
    }

    #[test]
    fn module_kennt_der_praeprozessor() {
        for a in TYPEN {
            for m in a.module {
                assert!(crate::preprocess::MODULES.contains(m), "{}: Modul {} gibt es nicht", a.name, m);
            }
        }
    }
}
