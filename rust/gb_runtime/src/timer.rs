//! Modul `timer` -- geplante Aktionen ohne MILLIS-Buchfuehrung.
//!
//! `TIMER_AFTER(ms, funcref)` (einmalig) und `TIMER_EVERY(ms, funcref)`
//! (wiederkehrend) registrieren FUNCREF-Callbacks; `TIMER_UPDATE()` (pro
//! Frame, wie INPUT_UPDATE/GUI_UPDATE) feuert die faelligen. Dazu
//! `COOLDOWN("id", ms)` -- ein String-ID-basierter Ratenbegrenzer im Stil
//! des Immediate-Mode-ui-Moduls: TRUE wenn frei, startet dann die Sperre.
//!
//! Die Struktur haelt nur State + Zeitmathematik; das Aufrufen der
//! GB-Funktionen macht die VM (`try_timer` in vm.rs, gleiches Muster wie
//! die GUI-Callbacks). Kein Grafik-Bezug -- laeuft auch konsolen-only.
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct Entry {
    due: Instant,
    interval_ms: Option<i64>,   // Some(_) = TIMER_EVERY, None = TIMER_AFTER
    func: String,               // FUNCREF-Name (parameterlos aufgerufen)
}

#[derive(Default)]
pub struct Timers {
    entries: Vec<Option<Entry>>,          // Tombstones -> IDs bleiben stabil
    cooldowns: HashMap<String, Instant>,  // Cooldown-ID -> wieder frei ab
}

impl Timers {
    pub fn after(&mut self, ms: i64, func: String) -> i64 {
        self.add(ms, None, func)
    }

    pub fn every(&mut self, ms: i64, func: String) -> i64 {
        self.add(ms, Some(ms), func)
    }

    fn add(&mut self, ms: i64, interval_ms: Option<i64>, func: String) -> i64 {
        self.entries.push(Some(Entry {
            due: Instant::now() + Duration::from_millis(clamp_ms(ms)),
            interval_ms, func,
        }));
        (self.entries.len() - 1) as i64
    }

    /// Abbrechen ist tolerant: eine bereits gefeuerte/abgebrochene ID ist
    /// ein No-Op (AFTER-Timer raeumen sich selbst weg).
    pub fn cancel(&mut self, id: i64) {
        if id >= 0 {
            if let Some(slot) = self.entries.get_mut(id as usize) { *slot = None; }
        }
    }

    pub fn active(&self, id: i64) -> bool {
        id >= 0 && matches!(self.entries.get(id as usize), Some(Some(_)))
    }

    pub fn count(&self) -> i64 {
        self.entries.iter().flatten().count() as i64
    }

    /// Alle Timer + Cooldowns verwerfen (z.B. beim Scene-Wechsel).
    pub fn clear(&mut self) {
        // Review-Fund: `entries.clear()` leert den Vec komplett -- die
        // naechste TIMER_AFTER/EVERY-ID beginnt danach wieder bei 0 und
        // kollidiert mit IDs aus VOR dem Clear (z.B. eine alte, laengst
        // vergessene ID aus der letzten Szene trifft per TIMER_CANCEL/ACTIVE
        // einen VOELLIG ANDEREN, neu registrierten Timer). Das widerspricht
        // dem dokumentierten Tombstone-Vertrag ("Timer-IDs bleiben stabil").
        // Stattdessen jeden Slot einzeln auf None setzen (wie cancel()) --
        // die Vec-Laenge (und damit die naechste vergebene ID) bleibt erhalten.
        for slot in &mut self.entries { *slot = None; }
        self.cooldowns.clear();
    }

    /// Faellige Timer einsammeln (Funktionsnamen in Registrierungs-
    /// Reihenfolge). AFTER-Eintraege raeumen sich weg, EVERY-Eintraege
    /// werden auf jetzt+Intervall gestellt -- ein EVERY feuert pro Update
    /// hoechstens EINMAL (kein Aufhol-Burst nach einem Lag/SLEEP).
    pub fn take_due(&mut self) -> Vec<String> {
        let now = Instant::now();
        let mut due = Vec::new();
        for slot in &mut self.entries {
            if let Some(e) = slot {
                if e.due <= now {
                    due.push(e.func.clone());
                    match e.interval_ms {
                        Some(iv) => e.due = now + Duration::from_millis(clamp_ms(iv)),
                        None => *slot = None,
                    }
                }
            }
        }
        due
    }

    /// COOLDOWN(id, ms): TRUE wenn die ID frei ist -- und startet dann
    /// sofort die Sperre. FALSE solange die Sperre laeuft.
    pub fn cooldown(&mut self, id: &str, ms: i64) -> bool {
        let now = Instant::now();
        if matches!(self.cooldowns.get(id), Some(frei_ab) if *frei_ab > now) {
            return false;
        }
        self.cooldowns.insert(id.to_string(), now + Duration::from_millis(clamp_ms(ms)));
        true
    }
}

/// Review-Fund: `Instant + Duration` PANIKED, wenn die Summe ueberlaeuft --
/// `TIMER_AFTER(9223372036854775807, cb)` bzw. `COOLDOWN("x", i64::MAX)`
/// abortierten so den gesamten Prozess (nicht per TRY/CATCH fangbar), obwohl
/// nur `ms >= 0` geprueft wurde. Ein Deckel weit ueber jeder realistischen
/// Spiellaufzeit (24h) verhindert das, ohne echte Anwendungsfaelle einzuschraenken.
fn clamp_ms(ms: i64) -> u64 {
    const MAX_TIMER_MS: i64 = 24 * 60 * 60 * 1000;
    ms.clamp(0, MAX_TIMER_MS) as u64
}

#[cfg(test)]
mod tests {
    use super::Timers;

    #[test]
    fn after_fires_once_and_clears() {
        let mut t = Timers::default();
        let id = t.after(0, "f".into());          // ms=0 -> sofort faellig
        assert!(t.active(id));
        assert_eq!(t.take_due(), vec!["f".to_string()]);
        assert!(!t.active(id));                   // AFTER raeumt sich weg
        assert!(t.take_due().is_empty());
        assert_eq!(t.count(), 0);
    }

    #[test]
    fn every_refires_until_cancel_and_ids_stay_stable() {
        let mut t = Timers::default();
        let a = t.after(60_000, "a".into());      // weit in der Zukunft
        let e = t.every(0, "e".into());
        assert_eq!(t.take_due(), vec!["e".to_string()]);
        assert_eq!(t.take_due(), vec!["e".to_string()]);   // EVERY bleibt
        t.cancel(e);
        assert!(t.take_due().is_empty());
        assert!(t.active(a));                     // Tombstone: a-ID unberuehrt
        t.cancel(e);                              // doppelt canceln = No-Op
        t.cancel(999);                            // unbekannte ID = No-Op
    }

    #[test]
    fn cooldown_locks_then_frees() {
        let mut t = Timers::default();
        assert!(t.cooldown("schuss", 60_000));    // frei -> TRUE + Sperre
        assert!(!t.cooldown("schuss", 60_000));   // gesperrt
        assert!(t.cooldown("anderes", 60_000));   // andere ID unabhaengig
        assert!(t.cooldown("sofort", 0));         // ms=0 -> nie gesperrt
        assert!(t.cooldown("sofort", 0));
        t.clear();
        assert!(t.cooldown("schuss", 60_000));    // clear gibt frei
    }
}
