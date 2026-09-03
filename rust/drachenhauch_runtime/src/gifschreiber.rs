//! Animierte GIFs schreiben (IMAGE_SAVE_GIF).
//!
//! raylib kann GIFs nur LESEN (`LoadImageAnim`, und auch das nur eingeschraenkt)
//! -- schreiben gar nicht. Fuer einen Sprite-Editor ist das die letzte
//! Einbahnstrasse: Einzelbilder erzeugen kann er, sie als Bewegung ausgeben
//! nicht.
//!
//! **Warum die `gif`-Crate und kein eigener Kodierer:** GIF komprimiert mit
//! LZW, und ein LZW-Schreiber ist genau die Art Code, die auf den ersten Blick
//! stimmt und in einem Randfall still etwas Falsches liefert (Code-Breite beim
//! Uebergang, Clear-Code zur rechten Zeit). Dieselbe Begruendung wie bei den
//! Pruefsummen in der Cargo.toml. Die Crate ist pure-Rust, ohne C-Werkzeuge.
//!
//! **Was GIF nicht kann, und was hier daraus folgt:**
//!
//! * Hoechstens 256 Farben je Bild. Pixelgrafik bleibt fast immer darunter --
//!   deshalb wird die Farbtafel EXAKT aus den vorhandenen Farben gebaut,
//!   solange es hoechstens 255 sind (plus einen Platz fuer "durchsichtig").
//!   Erst darueber wird zusammengefasst, und dann verschieben sich Farben.
//!   Ein Verfahren, das IMMER zusammenfasst, haette schon ein
//!   Vier-Farben-Sprite verfaelscht.
//! * Durchsichtigkeit nur GANZ oder GAR NICHT -- ein Bildpunkt ist
//!   durchsichtig oder deckend, nichts dazwischen. Halbdurchsichtige Punkte
//!   werden an der Schwelle 128 entschieden. Das ist keine Nachlaessigkeit,
//!   das Format kennt nichts anderes.

/// Ein Bild fuer den Schreiber: Masse und RGBA-Bytes (4 je Punkt).
pub struct Bild {
    pub breite: u16,
    pub hoehe: u16,
    pub rgba: Vec<u8>,
}

/// Ab dieser Deckkraft gilt ein Punkt als sichtbar.
const ALPHA_SCHWELLE: u8 = 128;

/// Die vorkommenden Farben, wenn es hoechstens `grenze` verschiedene sind.
///
/// Liefert `None`, sobald es mehr werden -- der Aufrufer fasst dann zusammen.
/// Durchsichtige Punkte zaehlen NICHT mit: sie bekommen ihren eigenen Platz.
pub fn farben_exakt(rgba: &[u8], grenze: usize) -> Option<Vec<[u8; 3]>> {
    let mut gesehen: Vec<[u8; 3]> = Vec::new();
    for p in rgba.chunks_exact(4) {
        if p[3] < ALPHA_SCHWELLE {
            continue;
        }
        let f = [p[0], p[1], p[2]];
        if !gesehen.contains(&f) {
            if gesehen.len() >= grenze {
                return None;
            }
            gesehen.push(f);
        }
    }
    Some(gesehen)
}

/// Bildpunkte auf Tafel-Nummern abbilden. Der letzte Platz ist "durchsichtig".
pub fn indizes(rgba: &[u8], tafel: &[[u8; 3]], durchsichtig: u8) -> Vec<u8> {
    let mut o = Vec::with_capacity(rgba.len() / 4);
    for p in rgba.chunks_exact(4) {
        if p[3] < ALPHA_SCHWELLE {
            o.push(durchsichtig);
        } else {
            let f = [p[0], p[1], p[2]];
            o.push(tafel.iter().position(|c| *c == f).unwrap_or(0) as u8);
        }
    }
    o
}

/// Die Farbtafel als flache RGB-Folge, aufgefuellt auf eine Zweierpotenz.
///
/// GIF speichert die Groesse der Tafel als Exponent -- eine Tafel mit 5
/// Eintraegen gibt es nicht, sie muss auf 8 aufgefuellt werden.
pub fn tafel_bytes(tafel: &[[u8; 3]], durchsichtig: u8) -> Vec<u8> {
    let noetig = (durchsichtig as usize + 1).max(2);
    let mut groesse = 2usize;
    while groesse < noetig {
        groesse *= 2;
    }
    let mut o = vec![0u8; groesse * 3];
    for (i, c) in tafel.iter().enumerate() {
        o[i * 3] = c[0];
        o[i * 3 + 1] = c[1];
        o[i * 3 + 2] = c[2];
    }
    o
}

/// Die Bilder als animiertes GIF schreiben.
///
/// `verzoegerungen` in Hundertstelsekunden, EINE JE BILD -- das Format kann
/// das, und eine Bildfolge braucht es: eine Pose wird gehalten, eine
/// Bewegung laeuft schnell durch. Wer fuer alle dasselbe will, gibt
/// dieselbe Zahl mehrfach; das ist billiger als zwei Wege dafuer.
///
/// `wiederholen` schaltet die Endlosschleife. Alle Bilder muessen gleich
/// gross sein -- ein GIF hat EINE Leinwand, und ein abweichendes Bild waere
/// entweder beschnitten oder verschoben; beides waere stiller Verlust.
pub fn schreiben(pfad: &str, bilder: &[Bild], verzoegerungen: &[u16],
                 wiederholen: bool) -> Result<(), String> {
    if bilder.is_empty() {
        return Err("IMAGE_SAVE_GIF: keine Bilder".into());
    }
    if verzoegerungen.len() != bilder.len() {
        return Err(std::format!(
            "IMAGE_SAVE_GIF: {} Bilder, aber {} Zeiten -- je Bild genau eine",
            bilder.len(), verzoegerungen.len()));
    }
    let (b, h) = (bilder[0].breite, bilder[0].hoehe);
    if b == 0 || h == 0 {
        return Err("IMAGE_SAVE_GIF: Bildgroesse 0".into());
    }
    for (i, f) in bilder.iter().enumerate() {
        if f.breite != b || f.hoehe != h {
            return Err(std::format!(
                "IMAGE_SAVE_GIF: Bild {} ist {}x{}, das erste {}x{} -- ein GIF hat EINE Leinwand",
                i + 1, f.breite, f.hoehe, b, h));
        }
    }
    let datei = std::fs::File::create(pfad)
        .map_err(|e| std::format!("IMAGE_SAVE_GIF: '{}' -- {}", pfad, e))?;
    let mut schreiber = gif::Encoder::new(std::io::BufWriter::new(datei), b, h, &[])
        .map_err(|e| std::format!("IMAGE_SAVE_GIF: {}", e))?;
    if wiederholen {
        schreiber.set_repeat(gif::Repeat::Infinite)
            .map_err(|e| std::format!("IMAGE_SAVE_GIF: {}", e))?;
    }
    for (nr, bild) in bilder.iter().enumerate() {
        let mut rahmen = match farben_exakt(&bild.rgba, 255) {
            Some(tafel) => {
                // Der Platz HINTER den benutzten Farben ist der durchsichtige.
                let durchsichtig = tafel.len() as u8;
                let mut r = gif::Frame {
                    width: bild.breite,
                    height: bild.hoehe,
                    buffer: std::borrow::Cow::Owned(
                        indizes(&bild.rgba, &tafel, durchsichtig)),
                    palette: Some(tafel_bytes(&tafel, durchsichtig)),
                    transparent: Some(durchsichtig),
                    ..Default::default()
                };
                r.dispose = gif::DisposalMethod::Background;
                r
            }
            None => {
                // Mehr als 255 Farben: zusammenfassen. Die Deckkraft wird
                // vorher hart entschieden, sonst mischt das Verfahren
                // halbdurchsichtige Punkte in die Tafel.
                let mut hart: Vec<u8> = bild.rgba.clone();
                for p in hart.chunks_exact_mut(4) {
                    p[3] = if p[3] < ALPHA_SCHWELLE { 0 } else { 255 };
                }
                let mut r = gif::Frame::from_rgba_speed(bild.breite, bild.hoehe, &mut hart, 10);
                r.dispose = gif::DisposalMethod::Background;
                r
            }
        };
        rahmen.delay = verzoegerungen[nr];
        schreiber.write_frame(&rahmen)
            .map_err(|e| std::format!("IMAGE_SAVE_GIF: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(punkte: &[[u8; 4]]) -> Vec<u8> {
        punkte.iter().flat_map(|p| p.iter().copied()).collect()
    }

    #[test]
    fn wenige_farben_bleiben_exakt() {
        let b = rgba(&[[255, 0, 0, 255], [0, 255, 0, 255], [255, 0, 0, 255]]);
        let t = farben_exakt(&b, 255).unwrap();
        assert_eq!(t, vec![[255, 0, 0], [0, 255, 0]]);
    }

    #[test]
    fn durchsichtige_punkte_zaehlen_nicht_zur_tafel() {
        let b = rgba(&[[9, 9, 9, 0], [255, 0, 0, 255], [7, 7, 7, 10]]);
        assert_eq!(farben_exakt(&b, 255).unwrap(), vec![[255, 0, 0]]);
    }

    #[test]
    fn ueber_der_grenze_gibt_es_keine_exakte_tafel() {
        let viele: Vec<[u8; 4]> = (0..300u32).map(|i| [(i % 256) as u8, (i / 256) as u8, 0, 255]).collect();
        assert!(farben_exakt(&rgba(&viele), 255).is_none());
    }

    #[test]
    fn indizes_zeigen_auf_die_richtige_farbe() {
        let tafel = vec![[255, 0, 0], [0, 255, 0]];
        let b = rgba(&[[0, 255, 0, 255], [255, 0, 0, 255], [0, 0, 0, 0]]);
        assert_eq!(indizes(&b, &tafel, 2), vec![1, 0, 2]);
    }

    #[test]
    fn halbe_deckkraft_faellt_an_der_schwelle() {
        let tafel = vec![[1, 2, 3]];
        // 127 gilt als durchsichtig, 128 als sichtbar.
        let b = rgba(&[[1, 2, 3, 127], [1, 2, 3, 128]]);
        assert_eq!(indizes(&b, &tafel, 1), vec![1, 0]);
    }

    #[test]
    fn die_tafel_waechst_auf_eine_zweierpotenz() {
        // 5 Farben + ein durchsichtiger Platz = 6 -> aufgefuellt auf 8.
        let tafel: Vec<[u8; 3]> = (0..5).map(|i| [i, i, i]).collect();
        assert_eq!(tafel_bytes(&tafel, 5).len(), 8 * 3);
        // Eine einzige Farbe braucht trotzdem zwei Plaetze (GIF-Mindestmass).
        assert_eq!(tafel_bytes(&[[1, 2, 3]], 1).len(), 2 * 3);
    }

    #[test]
    fn verschieden_grosse_bilder_werden_abgelehnt() {
        let a = Bild { breite: 2, hoehe: 2, rgba: vec![0; 16] };
        let b = Bild { breite: 3, hoehe: 2, rgba: vec![0; 24] };
        let e = schreiben("egal.gif", &[a, b], 10, true).unwrap_err();
        assert!(e.contains("EINE Leinwand"), "{}", e);
    }

    #[test]
    fn ohne_bilder_ist_es_ein_fehler() {
        assert!(schreiben("egal.gif", &[], 10, true).is_err());
    }
}
