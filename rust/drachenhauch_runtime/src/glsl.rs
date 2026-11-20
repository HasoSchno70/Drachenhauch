//! Kleine Lesehilfen fuer GLSL-Quelltext, ohne Fenster pruefbar.
//!
//! `SHADER_LOAD` nimmt eine Datei ODER Quelltext, und `SHADER_SET` & Co.
//! schweigen, wenn es den Namen im Shader nicht gibt (raylib liefert dann nur
//! den Ort -1). Ein vertippter Dateiname kam so als "Quelltext" beim Treiber an
//! ("syntax error, unexpected '.'"), ein vertipptes Uniform tat einfach nichts.
//! Beides bemerkt man jetzt am Quelltext selbst.

/// Ist das Argument von `SHADER_LOAD` Quelltext oder ein Dateiname?
/// Ein Shader hat immer mindestens eine Anweisung (`;`) und einen Block
/// (`{`); ein Pfad hat beides nicht.
pub fn ist_quelltext(arg: &str) -> bool {
    arg.contains('\n') || arg.contains(';') || arg.contains('{')
}

/// Die Namen aller `uniform`s, die der Quelltext deklariert -- auch mehrere
/// in einer Zeile (`uniform float a, b;`) und Felder (`uniform float w[4];`).
/// Kommentare zaehlen nicht.
pub fn uniforme(quelle: &str) -> Vec<String> {
    let text = ohne_kommentare(quelle);
    let mut namen = Vec::new();
    for anweisung in text.split(';') {
        let woerter: Vec<&str> = anweisung.split_whitespace().collect();
        let Some(pos) = woerter.iter().position(|w| *w == "uniform") else { continue };
        // uniform [Genauigkeit] Typ Name[, Name ...]
        let mut rest = &woerter[pos + 1..];
        if let Some(w) = rest.first() {
            if matches!(*w, "lowp" | "mediump" | "highp") { rest = &rest[1..]; }
        }
        if rest.len() < 2 { continue; }
        // Ein Uniform-Block (`uniform Licht { ... }`) hat keine einfachen Namen.
        if rest.iter().any(|w| w.contains('{')) { continue; }
        let liste = rest[1..].join(" ");
        for teil in liste.split(',') {
            let name: String = teil.trim().chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
            if !name.is_empty() && !namen.contains(&name) { namen.push(name); }
        }
    }
    namen
}

fn ohne_kommentare(s: &str) -> String {
    let mut aus = String::with_capacity(s.len());
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' { i += 1; }
        } else if b[i] == '/' && b.get(i + 1) == Some(&'*') {
            i += 2;
            while i < b.len() && !(b[i] == '*' && b.get(i + 1) == Some(&'/')) { i += 1; }
            i += 2;
            aus.push(' ');
        } else {
            aus.push(b[i]);
            i += 1;
        }
    }
    aus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namen_aus_dem_quelltext() {
        let q = "#version 330\n\
                 in vec2 fragTexCoord;\n\
                 uniform sampler2D texture0;\n\
                 uniform  float zeit; // uniform float kommentar;\n\
                 /* uniform vec2 auchnicht; */\n\
                 uniform highp vec2 groesse, mitte;\n\
                 uniform float stufen[8];\n\
                 void main() { float uniformx = 1.0; }\n";
        assert_eq!(uniforme(q), ["texture0", "zeit", "groesse", "mitte", "stufen"]);
    }

    #[test]
    fn block_und_leeres() {
        assert!(uniforme("").is_empty());
        assert!(uniforme("uniform Licht { vec3 farbe; };").is_empty());
    }

    #[test]
    fn datei_oder_quelltext() {
        assert!(!ist_quelltext("plasma.fs"));
        assert!(!ist_quelltext("assets/shader/kranz.fs"));
        assert!(ist_quelltext("void main() { }"));
        assert!(ist_quelltext("#version 330\nvoid main() {}"));
    }
}
