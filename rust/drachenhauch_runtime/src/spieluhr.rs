//! Die Spieluhr: EINE Zeit fuer TWEEN, CAMERA_SHAKE und das timer-Modul.
//!
//! Im normalen Lauf ist sie die Wanduhr seit dem Programmstart (wie bisher).
//! Laeuft das Programm mit fester Bildzahl (`DHRT_FRAMES`, jeder Testlauf),
//! zaehlt DELTA fest 1/60 s je Bild -- die Wanduhr aber, was die Maschine
//! gerade schafft. Bis 2026.53 liefen beide auseinander: nach 30 Bildern war
//! die Summe von DELTA 0,5 s, ein TWEEN von 500 ms aber erst bei 0,164, und
//! ein laufendes Band stand in fuenf gleichen Laeufen an fuenf Stellen.
//! Jetzt geht die Spieluhr dann mit den Bildern weiter -- nahtlos ab dem
//! Stand der Wanduhr beim ersten Bild.

use std::cell::Cell;

thread_local! {
    /// (Wanduhr beim ersten festen Bild in ms, Sekunden der Bilduhr dann,
    /// Sekunden der Bilduhr jetzt). None: es gilt die Wanduhr.
    static FEST: Cell<Option<(f64, f64, f64)>> = const { Cell::new(None) };
}

fn wanduhr_ms() -> f64 {
    crate::builtins::seit_start_ms()
}

/// Graphics meldet je Bild den Stand seiner Bilduhr (`uhr_jetzt`), solange
/// die Bildzahl fest ist.
#[cfg_attr(not(feature = "graphics"), allow(dead_code))]
pub fn bild(sekunden: f64) {
    FEST.with(|f| {
        let neu = match f.get() {
            Some((basis, start, _)) => (basis, start, sekunden),
            None => (wanduhr_ms(), sekunden, sekunden),
        };
        f.set(Some(neu));
    });
}

/// Millisekunden der Spieluhr.
pub fn jetzt_ms() -> f64 {
    match FEST.with(|f| f.get()) {
        Some((basis, start, jetzt)) => basis + (jetzt - start) * 1000.0,
        None => wanduhr_ms(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mit_festen_bildern_zaehlt_sie_bilder() {
        bild(1.0 / 60.0);
        let a = jetzt_ms();
        bild(31.0 / 60.0);
        let b = jetzt_ms();
        assert!((b - a - 500.0).abs() < 1e-6, "{} {}", a, b);
        FEST.with(|f| f.set(None));
    }
}
