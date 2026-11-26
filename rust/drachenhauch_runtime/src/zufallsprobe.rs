//! Nur fuer Tests: Bytefolgen, die ein Decoder fuer Netz- oder Geraetedaten
//! ohne Panik ueberstehen muss.
//!
//! Was ueber eine Leitung kommt, schreibt die Gegenseite -- ein fremdes
//! Programm, ein kaputtes Geraet, ein Angreifer. Ein Paket darf darum nie ein
//! Programm beenden (Rangliste Punkt 5 in docs/intern/inkonsistenzen-runtime.md;
//! 2026.53 beendete `GET /%aÄ` den httpd). Die Proben: jedes gueltige Paket
//! abgeschnitten, jedes Byte gekippt, auf 0 und auf 0xFF gesetzt, dazu reiner
//! Zufall mit Vorliebe fuer Grenzwerte. Fest gesaet, damit ein Fund
//! wiederholbar ist.

/// xorshift64* -- klein, ohne Crate, reicht fuer Proben.
pub(crate) struct Zufall(u64);

impl Zufall {
    pub(crate) fn new(saat: u64) -> Self { Zufall(saat.max(1)) }
    pub(crate) fn zahl(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub(crate) fn byte(&mut self) -> u8 {
        // Grenzwerte haeufiger als sonst: dort liegen die Fehler.
        match self.zahl() % 8 {
            0 => 0,
            1 => 0xFF,
            2 => 0x7F,
            3 => 0x80,
            _ => self.zahl() as u8,
        }
    }
}

/// Die Proben zu einer Reihe gueltiger Pakete.
pub(crate) fn proben(gueltig: &[Vec<u8>], zufall: usize) -> Vec<Vec<u8>> {
    let mut aus: Vec<Vec<u8>> = Vec::new();
    for g in gueltig {
        aus.push(g.clone());
        for n in 0..g.len() { aus.push(g[..n].to_vec()); }
        for i in 0..g.len() {
            for wert in [0u8, 0xFF, g[i] ^ 0x01, g[i] ^ 0x80, g[i].wrapping_add(1)] {
                let mut k = g.clone();
                k[i] = wert;
                aus.push(k);
            }
        }
    }
    let mut z = Zufall::new(0x5EED_D4AC_4E4A_0C4A);
    for _ in 0..zufall {
        let laenge = (z.zahl() % 96) as usize;
        let mut b: Vec<u8> = (0..laenge).map(|_| z.byte()).collect();
        // Die Haelfte beginnt wie ein gueltiges Paket, sonst kommt der
        // Zufall selten ueber die erste Pruefung hinaus.
        if let Some(g) = gueltig.get((z.zahl() as usize) % gueltig.len().max(1)) {
            if z.zahl() % 2 == 0 {
                let n = (z.zahl() as usize) % (g.len() + 1);
                b.splice(0..0, g[..n].iter().copied());
            }
        }
        aus.push(b);
    }
    aus
}

/// `f` mit jeder Probe rufen; geraet es in Panik, faellt der Test mit den
/// ersten Funden (als Hex).
pub(crate) fn nie_panik(name: &str, gueltig: &[Vec<u8>], f: impl Fn(&[u8])) {
    let mut funde: Vec<String> = Vec::new();
    let alle = proben(gueltig, 20_000);
    for p in &alle {
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(p))).is_err() {
            funde.push(p.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" "));
        }
    }
    assert!(funde.is_empty(), "{}: {} von {} Proben enden in einer Panik, etwa:\n{}",
            name, funde.len(), alle.len(), funde.iter().take(5).cloned().collect::<Vec<_>>().join("\n"));
}
