//! Der EINE WAV-Schreiber (PCM, 8 oder 16 Bit). Bis 2026.53 standen zwei
//! Kopien da -- `Audio::save_wav` und `mikrofon::wav_bytes` (Bericht A19).

/// `werte`: Abtastwerte -1..1, bei Stereo abwechselnd links/rechts.
pub fn bytes(kanaele: u16, rate: u32, bits: u16, werte: &[f32]) -> Vec<u8> {
    let bytes_pro_wert: u16 = if bits == 8 { 1 } else { 2 };
    let daten = werte.len() * bytes_pro_wert as usize;
    let block_align = kanaele * bytes_pro_wert;
    let mut b = Vec::with_capacity(44 + daten);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&((36 + daten) as u32).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());          // fmt-Laenge
    b.extend_from_slice(&1u16.to_le_bytes());           // PCM
    b.extend_from_slice(&kanaele.to_le_bytes());
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * block_align as u32).to_le_bytes());
    b.extend_from_slice(&block_align.to_le_bytes());
    b.extend_from_slice(&bits.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(daten as u32).to_le_bytes());
    for &w in werte {
        let v = w.clamp(-1.0, 1.0);
        if bits == 8 {
            b.push((v * 127.0 + 128.0).round().clamp(0.0, 255.0) as u8);
        } else {
            b.extend_from_slice(&((v * 32767.0).round() as i16).to_le_bytes());
        }
    }
    b
}

#[cfg(test)]
mod tests {
    #[test]
    fn kopf_und_werte() {
        let b = super::bytes(1, 22050, 16, &[0.0, 1.0, -1.0, 2.0]);
        assert_eq!(&b[0..4], b"RIFF");
        assert_eq!(b.len(), 44 + 8);
        assert_eq!(u32::from_le_bytes([b[28], b[29], b[30], b[31]]), 44100);
        assert_eq!(i16::from_le_bytes([b[46], b[47]]), 32767);
        let acht = super::bytes(2, 8000, 8, &[0.0, -1.0]);
        assert_eq!(&acht[44..], &[128, 1]);
        assert_eq!(u16::from_le_bytes([acht[32], acht[33]]), 2);
    }
}
