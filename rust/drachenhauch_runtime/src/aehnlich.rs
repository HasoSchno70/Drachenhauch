//! "Meintest du ...?" -- naheliegende Namen zu einem unbekannten.
//!
//! Gebraucht an drei Stellen, die heute nur sagen, dass es den Namen nicht
//! gibt: der unbekannte Befehl (Compiler-Warnung und Laufzeitmeldung) und die
//! nirgends angelegte Variable. Wer `CIRLCE` schreibt, sieht den Dreher nicht
//! -- er sucht den Fehler woanders.
//!
//! Zwei Regeln, weil eine nicht reicht:
//!
//! * **Editierabstand** faengt Dreher und Vertipper (`CIRLCE` -> `CIRCLE`,
//!   `MAKEDIR` -> `MKDIR`). Die Schranke waechst mit der Laenge, sonst passt
//!   bei kurzen Namen alles auf alles (`RND` -> `AND`).
//! * **Wortteile** faengt die ausgelassene Mitte (`PARTICLE_NEW` ->
//!   `PARTICLE_SYSTEM_NEW`). Dort ist der Abstand 7, jede vernuenftige
//!   Schranke laesst ihn liegen -- aber die Teile des getippten Namens stehen
//!   der Reihe nach im echten.
//!
//! Rein (kennt weder VM noch Index) und damit fuer sich pruefbar.

/// Editierabstand (Levenshtein), zwei Zeilen statt voller Matrix.
fn abstand(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut vor: Vec<usize> = (0..=b.len()).collect();
    let mut akt = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        akt[0] = i;
        for j in 1..=b.len() {
            let kost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            akt[j] = (vor[j] + 1).min(akt[j - 1] + 1).min(vor[j - 1] + kost);
        }
        std::mem::swap(&mut vor, &mut akt);
    }
    vor[b.len()]
}

/// Wie weit darf ein Name daneben liegen? Bei drei Zeichen ist ein Tausch
/// schon ein anderer Name, bei zwoelf ist er ein Vertipper.
fn schranke(len: usize) -> usize {
    match len {
        0..=3 => 1,
        4..=6 => 2,
        _ => 3,
    }
}

/// Stehen die Wortteile von `getippt` der Reihe nach in `kandidat`?
/// `particle_new` steckt so in `particle_system_new`, `audio_play` nicht in
/// `audio_stop`. Hoechstens zwei Teile duerfen dazwischen fehlen, sonst
/// passt `image_new` auch auf `image_draw_image_new_irgendwas`.
fn teile_passen(getippt: &str, kandidat: &str) -> bool {
    let g: Vec<&str> = getippt.split('_').filter(|t| !t.is_empty()).collect();
    let k: Vec<&str> = kandidat.split('_').filter(|t| !t.is_empty()).collect();
    if g.len() < 2 || k.len() <= g.len() || k.len() > g.len() + 2 { return false; }
    let mut i = 0;
    for teil in &k {
        if i < g.len() && g[i] == *teil { i += 1; }
    }
    i == g.len()
}

/// Bis zu drei naheliegende Namen, bester zuerst. `kandidaten` und `name`
/// duerfen in beliebiger Schreibweise kommen; zurueck kommt die Schreibweise
/// der Kandidaten.
pub fn vorschlaege<'a>(name: &str, kandidaten: impl Iterator<Item = &'a str>) -> Vec<String> {
    let gesucht = name.to_lowercase();
    if gesucht.is_empty() { return Vec::new(); }
    let grenze = schranke(gesucht.chars().count());
    let mut treffer: Vec<(usize, String)> = Vec::new();
    for k in kandidaten {
        let kl = k.to_lowercase();
        if kl == gesucht { continue; }
        // Der Abstand kann die Schranke nie unterbieten, wenn schon die
        // Laengen weiter auseinander liegen -- das spart den Vergleich.
        let (lg, lk) = (gesucht.chars().count(), kl.chars().count());
        if lg.abs_diff(lk) <= grenze {
            let d = abstand(&gesucht, &kl);
            if d <= grenze { treffer.push((d, k.to_string())); continue; }
        }
        if teile_passen(&gesucht, &kl) {
            // Bewusst besser als die meisten Abstandstreffer: dass alle Teile
            // der Reihe nach passen, ist ein scharfes Zeichen. Nur ein Name,
            // der sich um EIN Zeichen unterscheidet, ist noch naeher --
            // sonst stuende `PARTICLE_DRAW` (Abstand 3) vor
            // `PARTICLE_SYSTEM_NEW`.
            treffer.push((2, k.to_string()));
        }
    }
    treffer.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    treffer.truncate(3);
    treffer.into_iter().map(|(_, n)| n).collect()
}

/// Fertiger Meldungsanhang, oder leer. Gross geschrieben -- so stehen Befehle
/// in jeder Meldung.
pub fn hinweis<'a>(name: &str, kandidaten: impl Iterator<Item = &'a str>) -> String {
    let v = vorschlaege(name, kandidaten);
    match v.len() {
        0 => String::new(),
        1 => format!(" Meintest du {}?", v[0].to_uppercase()),
        _ => format!(" Meintest du {}?",
                     v.iter().map(|s| s.to_uppercase()).collect::<Vec<_>>().join(", ")),
    }
}

/// Wie `hinweis`, aber fuer Variablennamen: dort ist die Schreibweise die des
/// Programms, nicht die eines Verzeichnisses.
pub fn hinweis_klein<'a>(name: &str, kandidaten: impl Iterator<Item = &'a str>) -> String {
    let v = vorschlaege(name, kandidaten);
    if v.is_empty() { return String::new(); }
    format!(" Meintest du {}?", v.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAMEN: &[&str] = &[
        "CIRCLE", "CIRCLEF", "CLS", "BOX", "RND", "AND", "MKDIR", "RMDIR",
        "PARTICLE_SYSTEM_NEW", "PARTICLE_EMIT", "PARTICLE_DRAW", "SPRITE_ADD_ANIM", "SPRITE_NEW",
        "IMAGE_NEW", "IMAGE_DRAW_IMAGE", "AUDIO_MUSIC_PLAY", "AUDIO_PLAY",
    ];

    fn v(name: &str) -> Vec<String> { vorschlaege(name, NAMEN.iter().copied()) }

    #[test]
    fn dreher_und_vertipper() {
        assert_eq!(v("CIRLCE"), vec!["CIRCLE"]);
        assert_eq!(v("MAKEDIR"), vec!["MKDIR"]);
        assert_eq!(v("SPRITE_SET_ANIM"), vec!["SPRITE_ADD_ANIM"]);
        // Schreibweise ist egal, zurueck kommt die des Verzeichnisses.
        assert_eq!(v("cirlce"), vec!["CIRCLE"]);
    }

    #[test]
    fn ausgelassenes_wort() {
        // Der Teiletreffer steht VOR dem Abstandstreffer PARTICLE_DRAW (3).
        assert_eq!(v("PARTICLE_NEW"), vec!["PARTICLE_SYSTEM_NEW", "PARTICLE_DRAW"]);
        // ... aber nicht in die andere Richtung: AUDIO_MUSIC_PLAY hat ein
        // Wort MEHR als AUDIO_PLAY, das ist ein anderer Befehl.
        assert!(!v("AUDIO_PLAY_MUSIC").iter().any(|s| s == "AUDIO_PLAY"));
    }

    #[test]
    fn kurze_namen_bleiben_streng() {
        // Bei drei Zeichen zaehlt nur EIN Unterschied. `CLR` -> `CLS` ist
        // damit ein Vorschlag, `XYZ` -> nichts. Zwei Unterschiede waeren bei
        // so kurzen Namen schon ein anderer Befehl.
        assert_eq!(v("CLR"), vec!["CLS"]);
        assert!(v("XYZ").is_empty());
    }

    #[test]
    fn nichts_erfinden() {
        assert!(v("VOELLIG_ANDERES_DING").is_empty());
        assert!(v("").is_empty());
        assert!(hinweis("gibtsnicht_und_nie", NAMEN.iter().copied()).is_empty());
        assert!(hinweis("CIRLCE", NAMEN.iter().copied()).contains("CIRCLE"));
    }

    #[test]
    fn hoechstens_drei() {
        let viele = ["ab1", "ab2", "ab3", "ab4", "ab5"];
        assert_eq!(vorschlaege("ab", viele.iter().copied()).len(), 3);
    }
}
