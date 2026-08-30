//! Umrechnung zwischen RGB und HSV fuer den Farbwaehler (`GUI_COLORPICKER`).
//!
//! Freie Funktionen ohne Widget -- der Rundweg RGB -> HSV -> RGB laesst sich
//! so ueber alle Farben pruefen, ohne ein Fenster zu oeffnen.
//!
//! **Warum der Waehler HSV speichert und nicht RGB:** Bei Schwarz ist der
//! Farbton unbestimmt (jeder Ton ergibt Schwarz), bei Grau die Saettigung.
//! Wer nur die RGB-Farbe behaelt und den Ton bei jedem Zeichnen neu ausrechnet,
//! verliert ihn genau dann -- der Zeiger im Farbfeld springt beim Herunterziehen
//! auf Schwarz nach links, und beim Hochziehen kommt Rot statt der Farbe zurueck,
//! die man gewaehlt hatte.

/// `h` in Grad 0..360, `s` und `v` in 0..1 -> 0xRRGGBB.
pub fn hsv_zu_rgb(h: f32, s: f32, v: f32) -> i64 {
    let h = h.rem_euclid(360.0);
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let f = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as i64;
    (f(r) << 16) | (f(g) << 8) | f(b)
}

/// 0xRRGGBB -> (h in Grad, s, v).
///
/// Bei Grau ist der Farbton unbestimmt; hier kommt dann 0 heraus. Der Waehler
/// darf sich darauf NICHT verlassen -- er fuehrt seinen Ton selbst mit (siehe
/// Modul-Kommentar).
pub fn rgb_zu_hsv(c: i64) -> (f32, f32, f32) {
    let r = ((c >> 16) & 0xFF) as f32 / 255.0;
    let g = ((c >> 8) & 0xFF) as f32 / 255.0;
    let b = (c & 0xFF) as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d <= f32::EPSILON { 0.0 }
            else if max == r { 60.0 * (((g - b) / d) % 6.0) }
            else if max == g { 60.0 * ((b - r) / d + 2.0) }
            else { 60.0 * ((r - g) / d + 4.0) };
    let s = if max <= f32::EPSILON { 0.0 } else { d / max };
    (h.rem_euclid(360.0), s, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nah(a: f32, b: f32) -> bool { (a - b).abs() < 0.01 }

    #[test]
    fn die_reinen_farben() {
        assert_eq!(hsv_zu_rgb(0.0, 1.0, 1.0), 0xFF0000);
        assert_eq!(hsv_zu_rgb(120.0, 1.0, 1.0), 0x00FF00);
        assert_eq!(hsv_zu_rgb(240.0, 1.0, 1.0), 0x0000FF);
        assert_eq!(hsv_zu_rgb(60.0, 1.0, 1.0), 0xFFFF00);
        assert_eq!(hsv_zu_rgb(180.0, 1.0, 1.0), 0x00FFFF);
        assert_eq!(hsv_zu_rgb(300.0, 1.0, 1.0), 0xFF00FF);
    }

    #[test]
    fn schwarz_und_weiss() {
        assert_eq!(hsv_zu_rgb(0.0, 0.0, 0.0), 0x000000);
        assert_eq!(hsv_zu_rgb(0.0, 0.0, 1.0), 0xFFFFFF);
        assert_eq!(hsv_zu_rgb(200.0, 0.0, 1.0), 0xFFFFFF, "ohne Saettigung ist der Ton egal");
    }

    #[test]
    fn der_rundweg_haelt_ueber_viele_farben() {
        // Nicht drei Stichproben, sondern ein Gitter -- Umrechnungsfehler
        // sitzen gern in einem einzelnen der sechs Sektoren.
        let mut geprueft = 0;
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    let c = (r << 16) | (g << 8) | b;
                    let (h, s, v) = rgb_zu_hsv(c as i64);
                    assert_eq!(hsv_zu_rgb(h, s, v), c as i64,
                               "Rundweg verliert #{:06X}", c);
                    geprueft += 1;
                }
            }
        }
        assert!(geprueft > 4000, "nur {} Farben geprueft", geprueft);
    }

    #[test]
    fn hsv_werte_stimmen() {
        let (h, s, v) = rgb_zu_hsv(0xFF0000);
        assert!(nah(h, 0.0) && nah(s, 1.0) && nah(v, 1.0));
        let (h, s, v) = rgb_zu_hsv(0x808080);
        assert!(nah(s, 0.0), "Grau hat keine Saettigung");
        assert!(nah(v, 0.502), "v={}", v);
        assert!(nah(h, 0.0), "Grau: Ton unbestimmt -> 0");
    }

    #[test]
    fn farbton_laeuft_rundum() {
        // 360 Grad sind wieder 0 -- ohne rem_euclid faellt der Waehler beim
        // Ziehen ueber den Rand auf Schwarz.
        assert_eq!(hsv_zu_rgb(360.0, 1.0, 1.0), hsv_zu_rgb(0.0, 1.0, 1.0));
        assert_eq!(hsv_zu_rgb(-60.0, 1.0, 1.0), hsv_zu_rgb(300.0, 1.0, 1.0));
    }

    #[test]
    fn werte_ausserhalb_werden_geklemmt_statt_zu_kippen() {
        assert_eq!(hsv_zu_rgb(0.0, 5.0, 5.0), 0xFF0000);
        assert_eq!(hsv_zu_rgb(0.0, -1.0, -1.0), 0x000000);
    }
}
