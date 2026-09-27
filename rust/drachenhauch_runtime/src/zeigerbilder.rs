//! Eingebaute Zeigerbilder: Formen, die raylib nicht kennt (GLFW hat nur zehn
//! Standardzeiger). Jede ist ein 16x16-Raster aus `#` (weiss), `x` (schwarz)
//! und `.` (durchsichtig); um das Weisse legt `bild` von selbst einen
//! schwarzen Rand, damit die Form auf hellem UND dunklem Grund steht, und
//! vergroessert auf das Doppelte (32x32, die uebliche Zeigergroesse).
//!
//! Bewusst OHNE raylib -- die Rechnung ist so auch auf einem Rechner ohne
//! Fenster pruefbar; setzen tut sie `Graphics::zeiger_anwenden`.

/// Die gemeinsame Pfeilspitze (links oben, Spitze auf (1, 1)).
const PFEIL: [&str; 16] = [
    "................",
    ".#..............",
    ".##.............",
    ".###............",
    ".####...........",
    ".#####..........",
    ".######.........",
    ".#######........",
    ".########.......",
    ".#####..........",
    ".##.##..........",
    ".#...##.........",
    ".....##.........",
    "......##........",
    "......##........",
    "................",
];

/// Die eingebauten Formen: Name, Raster, Zusatz ueber dem Pfeil (oder kein
/// Pfeil), Brennpunkt im 16er-Raster.
fn raster(name: &str) -> Option<(Vec<&'static str>, bool, (i32, i32))> {
    Some(match name {
        "kopieren" => (vec![
            "................", "................", "................", "................",
            "................", "................", "................", "................",
            "................", "..........#####.", "..........##x##.", "..........#xxx#.",
            "..........##x##.", "..........#####.", "................", "................",
        ], true, (1, 1)),
        "hilfe" => (vec![
            "................", "..........###...", ".........#...#..", ".............#..",
            "............#...", "...........#....", "...........#....", "................",
            "...........#....", "................", "................", "................",
            "................", "................", "................", "................",
        ], true, (1, 1)),
        "arbeitet" => (vec![
            "................", "................", "................", "................",
            "................", "................", "................", "................",
            "................", "..........#####.", "...........###..", "............#...",
            "...........###..", "..........#####.", "................", "................",
        ], true, (1, 1)),
        "warten" => (vec![
            "................", "..############..", "...##########...", "....########....",
            ".....######.....", "......####......", ".......##.......", ".......##.......",
            "......####......", ".....######.....", "....########....", "...##########...",
            "..############..", "................", "................", "................",
        ], false, (7, 7)),
        "stift" => (vec![
            "................", "............##..", "...........####.", "..........####..",
            ".........###....", "........###.....", ".......###......", "......###.......",
            ".....###........", "....###.........", "...###..........", "..###...........",
            "..##............", ".x#.............", ".x..............", "................",
        ], false, (1, 14)),
        "pipette" => (vec![
            "................", "...........###..", "..........#####.", "..........#####.",
            "..........#####.", ".........###....", "........#.......", ".......#........",
            "......#.........", ".....#..........", "....#...........", "...#............",
            "..#.............", ".#..............", "................", "................",
        ], false, (1, 13)),
        _ => return None,
    })
}

/// Die Namen der eingebauten Formen (fuer die Tests).
#[cfg(test)]
pub const NAMEN: [&str; 6] = ["warten", "arbeitet", "hilfe", "kopieren", "stift", "pipette"];

/// RGBA-Punkte (32x32, Zeile fuer Zeile) und Brennpunkt einer eingebauten
/// Form; None = unbekannt.
pub fn bild(name: &str) -> Option<(Vec<u8>, i32, i32, (i32, i32))> {
    let (zusatz, mit_pfeil, brenn) = raster(name)?;
    // 0 durchsichtig, 1 weiss, 2 schwarz
    let mut z = [[0u8; 16]; 16];
    let mut setzen = |zeilen: &[&str]| {
        for (y, zeile) in zeilen.iter().enumerate() {
            for (x, c) in zeile.chars().enumerate().take(16) {
                match c { '#' => z[y][x] = 1, 'x' => z[y][x] = 2, _ => {} }
            }
        }
    };
    if mit_pfeil { setzen(&PFEIL); }
    setzen(&zusatz);
    // Rand: jeder durchsichtige Punkt neben einem weissen wird schwarz.
    let mut mit_rand = z;
    for y in 0..16i32 {
        for x in 0..16i32 {
            if z[y as usize][x as usize] != 0 { continue; }
            let nachbar = (-1..=1).any(|dy| (-1..=1).any(|dx| {
                let (nx, ny) = (x + dx, y + dy);
                (0..16).contains(&nx) && (0..16).contains(&ny) && z[ny as usize][nx as usize] == 1
            }));
            if nachbar { mit_rand[y as usize][x as usize] = 2; }
        }
    }
    let mut px = vec![0u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let farbe: [u8; 4] = match mit_rand[y / 2][x / 2] {
                1 => [255, 255, 255, 255],
                2 => [0, 0, 0, 255],
                _ => [0, 0, 0, 0],
            };
            px[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4].copy_from_slice(&farbe);
        }
    }
    Some((px, 32, 32, (brenn.0 * 2, brenn.1 * 2)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn punkt(px: &[u8], x: usize, y: usize) -> [u8; 4] {
        let i = (y * 32 + x) * 4;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    }

    #[test]
    fn alle_formen_haben_ein_bild_und_einen_brennpunkt_darin() {
        for n in NAMEN {
            let (px, w, h, (bx, by)) = bild(n).expect(n);
            assert_eq!((w, h, px.len()), (32, 32, 32 * 32 * 4), "{}", n);
            assert!((0..32).contains(&bx) && (0..32).contains(&by), "{}", n);
            // Etwas Weisses UND etwas Schwarzes -- sonst verschwaende die
            // Form auf einem der beiden Gruende.
            let weiss = px.chunks(4).any(|p| p == [255, 255, 255, 255]);
            let schwarz = px.chunks(4).any(|p| p == [0, 0, 0, 255]);
            assert!(weiss && schwarz, "{}", n);
        }
        assert!(bild("quatsch").is_none());
    }

    #[test]
    fn weiss_hat_immer_einen_schwarzen_rand() {
        for n in NAMEN {
            let (px, _, _, _) = bild(n).unwrap();
            for y in 0..32usize {
                for x in 0..32usize {
                    if punkt(&px, x, y) != [255, 255, 255, 255] { continue; }
                    // Kein weisser Punkt grenzt an einen durchsichtigen.
                    for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        if !(0..32).contains(&nx) || !(0..32).contains(&ny) { continue; }
                        assert_ne!(punkt(&px, nx as usize, ny as usize)[3], 0, "{} bei {},{}", n, x, y);
                    }
                }
            }
        }
    }

    #[test]
    fn die_pfeilformen_haben_die_spitze_links_oben() {
        for n in ["kopieren", "hilfe", "arbeitet"] {
            let (px, _, _, (bx, by)) = bild(n).unwrap();
            assert_eq!((bx, by), (2, 2), "{}", n);
            assert_eq!(punkt(&px, 2, 2), [255, 255, 255, 255], "{}", n);
        }
    }
}
