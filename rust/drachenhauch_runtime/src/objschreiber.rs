//! Modelle als Wavefront-OBJ schreiben (MODEL_SAVE).
//!
//! Damit wandert ein Modell, das ein Programm erzeugt hat (ein Labyrinth aus
//! MESH_CUBICMAP, ein Gelaende aus MESH_HEIGHTMAP), in Blender und andere
//! 3D-Programme -- und mit LOADMODEL wieder zurueck.
//!
//! **Warum nicht raylibs `ExportMesh`:** es schreibt nur EIN Netz eines
//! Modells, und es schreibt die Texturkoordinaten so, wie sie im Speicher
//! stehen -- raylibs OBJ-Lader kippt sie beim Lesen aber (`v = 1 - v`). Ein
//! damit gesichertes Modell kaeme mit auf dem Kopf stehender Textur zurueck.
//! Hier wird beim Schreiben ebenfalls gekippt, und alle Netze kommen in eine
//! Datei. Ohne raylib, damit es fuer sich pruefbar ist.

use std::fmt::Write;

/// Ein Netz: Eckpunkte (x, y, z hintereinander), wahlweise Texturkoordinaten
/// (u, v) und Normalen (x, y, z) je Eckpunkt, wahlweise Dreiecke als Indizes
/// (sonst bilden je drei Eckpunkte der Reihe nach ein Dreieck).
pub struct Netz {
    pub punkte: Vec<f32>,
    pub uv: Option<Vec<f32>>,
    pub normalen: Option<Vec<f32>>,
    pub indizes: Option<Vec<u16>>,
}

/// Der Text der OBJ-Datei. Zahlen in kuerzester Schreibweise, die genau den
/// Wert ergibt -- so ist die Datei bei gleichem Modell immer dieselbe.
pub fn obj_text(netze: &[Netz]) -> String {
    let mut s = String::from("# Drachenhauch MODEL_SAVE\n");
    let mut basis = 0usize;
    for (k, n) in netze.iter().enumerate() {
        let zahl = n.punkte.len() / 3;
        let uv = n.uv.as_ref().filter(|t| t.len() >= zahl * 2);
        let nor = n.normalen.as_ref().filter(|t| t.len() >= zahl * 3);
        let _ = writeln!(s, "o teil{}", k + 1);
        for p in n.punkte.chunks_exact(3) {
            let _ = writeln!(s, "v {} {} {}", p[0], p[1], p[2]);
        }
        if let Some(t) = uv {
            for e in t.chunks_exact(2).take(zahl) { let _ = writeln!(s, "vt {} {}", e[0], 1.0 - e[1]); }
        }
        if let Some(t) = nor {
            for e in t.chunks_exact(3).take(zahl) { let _ = writeln!(s, "vn {} {} {}", e[0], e[1], e[2]); }
        }
        let ecke = |i: usize| -> String {
            let j = basis + i + 1;
            match (uv.is_some(), nor.is_some()) {
                (true, true) => format!("{j}/{j}/{j}"),
                (true, false) => format!("{j}/{j}"),
                (false, true) => format!("{j}//{j}"),
                (false, false) => format!("{j}"),
            }
        };
        let dreiecke: Vec<usize> = match &n.indizes {
            Some(ix) => ix.iter().map(|&i| i as usize).collect(),
            None => (0..zahl - zahl % 3).collect(),
        };
        // Ein Dreieck mit einem Index ausserhalb faellt GANZ weg -- einzelne
        // Indizes zu streichen verschoebe jedes Dreieck dahinter.
        for d in dreiecke.chunks_exact(3).filter(|d| d.iter().all(|&i| i < zahl)) {
            let _ = writeln!(s, "f {} {} {}", ecke(d[0]), ecke(d[1]), ecke(d[2]));
        }
        basis += zahl;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dreieck(uv: bool, nor: bool) -> Netz {
        Netz {
            punkte: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5],
            uv: uv.then(|| vec![0.0, 0.0, 1.0, 0.25, 0.0, 1.0]),
            normalen: nor.then(|| vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]),
            indizes: None,
        }
    }

    #[test]
    fn eckpunkte_und_flaechen_mit_allem() {
        let t = obj_text(&[dreieck(true, true)]);
        assert!(t.contains("v 0 1 0.5\n"), "{t}");
        assert!(t.contains("vn 0 0 1\n"), "{t}");
        assert!(t.contains("f 1/1/1 2/2/2 3/3/3\n"), "{t}");
    }

    #[test]
    fn texturkoordinaten_werden_gekippt() {
        // raylibs Lader rechnet 1 - v; hin und zurueck muss es dasselbe sein.
        let t = obj_text(&[dreieck(true, false)]);
        assert!(t.contains("vt 1 0.75\n"), "{t}");
        assert!(t.contains("vt 0 0\n"), "{t}");
        assert!(t.contains("f 1/1 2/2 3/3\n"), "{t}");
    }

    #[test]
    fn ohne_texturkoordinaten_doppelter_strich() {
        let t = obj_text(&[dreieck(false, true)]);
        assert!(t.contains("f 1//1 2//2 3//3\n"), "{t}");
        assert!(!t.contains("vt "), "{t}");
    }

    #[test]
    fn mehrere_netze_zaehlen_weiter() {
        let t = obj_text(&[dreieck(false, false), dreieck(false, false)]);
        assert!(t.contains("o teil2\n"), "{t}");
        assert!(t.contains("f 4 5 6\n"), "{t}");
    }

    #[test]
    fn indizes_und_ein_falscher_index() {
        let mut n = dreieck(false, false);
        n.punkte.extend_from_slice(&[1.0, 1.0, 0.0]);
        n.indizes = Some(vec![0, 1, 2, 7, 0, 1, 1, 3, 2]);
        let t = obj_text(&[n]);
        assert!(t.contains("f 1 2 3\nf 2 4 3\n"), "{t}");
        // Index 7 gibt es nicht -- das Dreieck faellt weg statt ins Leere zu zeigen.
        assert_eq!(t.matches("\nf ").count(), 2, "{t}");
    }
}
