//! Modul `pdf` -- druckfertige Seiten schreiben (Punkt 7 des
//! Allzweck-Audits).
//!
//! Rechnung, Lieferschein, Bericht, Etikett: fuer kaufmaennische Software ist
//! das fast immer die erste Forderung nach „speichern". Bis hierher endete
//! der Weg beim Bildschirm oder bei einer Textdatei.
//!
//! **Ohne eingebettete Schriften.** Jeder PDF-Leser bringt vierzehn
//! Standard-Schriften mit (Helvetica, Times, Courier in vier Schnitten, dazu
//! Symbol und ZapfDingbats); wer nur diese benutzt, braucht nichts
//! einzubetten. Genau daran haengt der Aufwand eines PDF-Erzeugers -- eine
//! TrueType-Datei einzubetten hiesse Tabellen parsen, Untermengen bilden und
//! einen CID-Font aufbauen. Der Preis dieser Entscheidung steht unten bei
//! `zeichenbreite`.
//!
//! **Millimeter, von oben gezaehlt.** PDF selbst rechnet in Punkten und von
//! UNTEN. Wer eine Rechnung setzt, denkt aber in „25 mm vom oberen Rand" --
//! also rechnet dieses Modul so und dreht die Y-Achse beim Schreiben um. Nur
//! die SCHRIFTGROESSE bleibt in Punkten, weil sie jeder so kennt (11 pt).

/// 1 mm in PDF-Punkten (1/72 Zoll).
const MM: f64 = 72.0 / 25.4;

/// Ein aufgezeichneter Zeichenbefehl -- damit dieselbe Seite auf mehr als ein
/// Ziel geht: PDF (hier), Drucker (`drucken.rs`), Vorschau (`graphics.rs`).
/// Masse in Millimetern ab Papierkante von oben, Farbe als Anteile 0..1.
/// `y` bei Text meint die OBERKANTE der Zeile, wie im Programm angegeben.
#[derive(Clone, Debug)]
pub enum Op {
    Text { x: f64, y: f64, text: String, schrift: usize, groesse_pt: f64, farbe: (f64, f64, f64) },
    Linie { x1: f64, y1: f64, x2: f64, y2: f64, breite_mm: f64, farbe: (f64, f64, f64) },
    Rechteck { x: f64, y: f64, b: f64, h: f64, fuellen: bool, breite_mm: f64, farbe: (f64, f64, f64) },
}

pub struct Seite {
    pub breite_mm: f64,
    pub hoehe_mm: f64,
    inhalt: String,
    /// Welche Schriften diese Seite benutzt (Index in `SCHRIFTEN`).
    benutzt: Vec<usize>,
    /// Dieselben Befehle noch einmal, fuer die anderen Ziele.
    pub ops: Vec<Op>,
}

pub struct Dokument {
    pub seiten: Vec<Seite>,
    pub breite_mm: f64,
    pub hoehe_mm: f64,
    /// Aktuelle Schrift + Groesse, gilt fuer den naechsten Text.
    schrift: usize,
    pub groesse_pt: f64,
    farbe: (f64, f64, f64),
    strich_mm: f64,
    pub titel: String,
}

/// Die vierzehn Standard-Schriften: (Name im Programm, Name im PDF, fest?).
///
/// `fest` heisst dicktengleich -- nur dort laesst sich die Breite eines
/// Textes ohne Schriftmasse ausrechnen.
const SCHRIFTEN: &[(&str, &str, bool)] = &[
    ("helvetica", "Helvetica", false),
    ("helvetica-fett", "Helvetica-Bold", false),
    ("helvetica-kursiv", "Helvetica-Oblique", false),
    ("helvetica-fett-kursiv", "Helvetica-BoldOblique", false),
    ("times", "Times-Roman", false),
    ("times-fett", "Times-Bold", false),
    ("times-kursiv", "Times-Italic", false),
    ("times-fett-kursiv", "Times-BoldItalic", false),
    ("courier", "Courier", true),
    ("courier-fett", "Courier-Bold", true),
    ("courier-kursiv", "Courier-Oblique", true),
    ("courier-fett-kursiv", "Courier-BoldOblique", true),
    ("symbol", "Symbol", false),
    ("zapfdingbats", "ZapfDingbats", false),
];

pub fn schriftnamen() -> String {
    SCHRIFTEN.iter().map(|(n, _, _)| *n).collect::<Vec<_>>().join(", ")
}

/// Der Name, wie das Programm die Schrift nennt ("helvetica-fett") -- fuer
/// die Abbildung auf GDI-Schriften beim Drucken.
pub fn schrift_programmname(index: usize) -> &'static str {
    SCHRIFTEN.get(index).map(|(n, _, _)| *n).unwrap_or("helvetica")
}

pub fn schrift_index(name: &str) -> Option<usize> {
    let k = name.to_lowercase();
    SCHRIFTEN.iter().position(|(n, _, _)| *n == k)
}

/// Seitengroesse aus einem Namen -- in Millimetern.
pub fn seitenmass(name: &str, quer: bool) -> Option<(f64, f64)> {
    let (b, h) = match name.to_lowercase().as_str() {
        "a3" => (297.0, 420.0),
        "a4" | "" => (210.0, 297.0),
        "a5" => (148.0, 210.0),
        "a6" => (105.0, 148.0),
        "letter" => (215.9, 279.4),
        "legal" => (215.9, 355.6),
        _ => return None,
    };
    Some(if quer { (h, b) } else { (b, h) })
}

impl Dokument {
    pub fn neu(breite_mm: f64, hoehe_mm: f64) -> Dokument {
        let mut d = Dokument {
            seiten: Vec::new(), breite_mm, hoehe_mm,
            schrift: 0, groesse_pt: 11.0, farbe: (0.0, 0.0, 0.0), strich_mm: 0.2,
            titel: String::new(),
        };
        d.neue_seite();
        d
    }

    /// Eine neue Seite -- die vorige bleibt, wie sie ist.
    ///
    /// Schrift, Groesse, Farbe und Strichstaerke gelten weiter: sie sind
    /// Einstellungen des Dokuments, nicht der Seite. Alles andere hiesse, sie
    /// nach jedem Seitenwechsel neu zu setzen.
    pub fn neue_seite(&mut self) {
        self.seiten.push(Seite {
            breite_mm: self.breite_mm, hoehe_mm: self.hoehe_mm,
            inhalt: String::new(), benutzt: Vec::new(), ops: Vec::new(),
        });
        // Die Einstellungen gelten auch auf der neuen Seite -- der
        // Inhaltsstrom jeder Seite faengt aber bei null an, also noch einmal
        // hineinschreiben.
        let (r, g, b) = self.farbe;
        let s = self.strich_mm;
        self.setze_farbe_roh(r, g, b);
        self.setze_strich_roh(s);
    }

    fn hier(&mut self) -> &mut Seite {
        // `neu` legt immer eine Seite an, es gibt also nie keine.
        self.seiten.last_mut().unwrap()
    }

    pub fn schrift_index(&self) -> usize { self.schrift }

    pub fn setze_schrift(&mut self, index: usize, groesse_pt: f64) {
        self.schrift = index;
        self.groesse_pt = groesse_pt;
    }

    pub fn setze_farbe(&mut self, r: f64, g: f64, b: f64) {
        self.farbe = (r, g, b);
        self.setze_farbe_roh(r, g, b);
    }

    fn setze_farbe_roh(&mut self, r: f64, g: f64, b: f64) {
        // `rg` faerbt Fuellungen und Text, `RG` die Striche -- fuer ein
        // GB-Programm ist "die Farbe" eine Sache, also immer beides.
        let z = format!("{:.3} {:.3} {:.3} rg\n{:.3} {:.3} {:.3} RG\n", r, g, b, r, g, b);
        self.hier().inhalt.push_str(&z);
    }

    pub fn setze_strich(&mut self, mm: f64) {
        self.strich_mm = mm;
        self.setze_strich_roh(mm);
    }

    fn setze_strich_roh(&mut self, mm: f64) {
        let z = format!("{:.3} w\n", mm * MM);
        self.hier().inhalt.push_str(&z);
    }

    /// Y von oben in PDF-Y von unten.
    fn y(&self, mm: f64) -> f64 { (self.hoehe_mm - mm) * MM }

    pub fn text(&mut self, x_mm: f64, y_mm: f64, text: &str) -> Result<(), String> {
        let roh = pdf_text(text)?;
        let (schrift, groesse) = (self.schrift, self.groesse_pt);
        let (x, y) = (x_mm * MM, self.y(y_mm));
        let s = self.hier();
        if !s.benutzt.contains(&schrift) { s.benutzt.push(schrift); }
        // Die Y-Angabe meint die GRUNDLINIE. Wer "25 mm vom Rand" sagt, meint
        // aber die Oberkante der Zeile -- also um die Schriftgroesse nach
        // unten ruecken. (Genauer waere die Oberlaenge der Schrift; die steht
        // in den Schriftmassen, die hier bewusst fehlen.)
        let grundlinie = y - groesse;
        s.inhalt.push_str(&format!("BT /F{} {:.2} Tf {:.2} {:.2} Td {} Tj ET\n",
                                   schrift, groesse, x, grundlinie, roh));
        let farbe = self.farbe;
        self.hier().ops.push(Op::Text { x: x_mm, y: y_mm, text: text.to_string(), schrift, groesse_pt: groesse, farbe });
        Ok(())
    }

    pub fn linie(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let (a, b) = (self.y(y1), self.y(y2));
        let (px1, px2) = (x1 * MM, x2 * MM);
        self.hier().inhalt.push_str(&format!("{:.2} {:.2} m {:.2} {:.2} l S\n", px1, a, px2, b));
        let (farbe, breite_mm) = (self.farbe, self.strich_mm);
        self.hier().ops.push(Op::Linie { x1, y1, x2, y2, breite_mm, farbe });
    }

    /// Rechteck von (x,y) mit Breite/Hoehe -- `fuellen` entscheidet zwischen
    /// Flaeche und Umriss.
    pub fn rechteck(&mut self, x: f64, y: f64, b: f64, h: f64, fuellen: bool) {
        // PDF setzt das Rechteck von seiner UNTEREN Kante aus.
        let unten = self.y(y + h);
        let op = if fuellen { "f" } else { "S" };
        self.hier().inhalt.push_str(&format!("{:.2} {:.2} {:.2} {:.2} re {}\n",
                                             x * MM, unten, b * MM, h * MM, op));
        let (farbe, breite_mm) = (self.farbe, self.strich_mm);
        self.hier().ops.push(Op::Rechteck { x, y, b, h, fuellen, breite_mm, farbe });
    }

    /// Das fertige PDF als Bytes.
    pub fn bauen(&self) -> Vec<u8> {
        // Objektnummern: 1 = Katalog, 2 = Seitenbaum, dann je Seite zwei
        // (Seite + Inhalt), danach die Schriften.
        let n_seiten = self.seiten.len();
        let erste_schrift = 3 + n_seiten * 2;
        let mut roh: Vec<u8> = Vec::new();
        let mut versatz: Vec<usize> = Vec::new();
        roh.extend_from_slice(b"%PDF-1.4\n");
        // Ein Kommentar mit hohen Bytes sagt jedem Werkzeug: das hier ist
        // keine Textdatei. So steht es in der Norm.
        roh.extend_from_slice(&[b'%', 0xE2, 0xE3, 0xCF, 0xD3, b'\n']);

        let mut obj = |roh: &mut Vec<u8>, versatz: &mut Vec<usize>, nr: usize, inhalt: &[u8]| {
            versatz.push(roh.len());
            roh.extend_from_slice(format!("{} 0 obj\n", nr).as_bytes());
            roh.extend_from_slice(inhalt);
            roh.extend_from_slice(b"\nendobj\n");
        };

        // 1: Katalog
        obj(&mut roh, &mut versatz, 1, b"<< /Type /Catalog /Pages 2 0 R >>");
        // 2: Seitenbaum
        let kinder: String = (0..n_seiten).map(|i| format!("{} 0 R ", 3 + i * 2)).collect();
        obj(&mut roh, &mut versatz, 2,
            format!("<< /Type /Pages /Kids [{}] /Count {} >>", kinder.trim_end(), n_seiten).as_bytes());
        // Je Seite: Seitenobjekt + Inhaltsstrom
        for (i, s) in self.seiten.iter().enumerate() {
            let schriften: String = s.benutzt.iter()
                .map(|f| format!("/F{} {} 0 R ", f, erste_schrift + f)).collect();
            let seite = format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] \
                 /Resources << /Font << {} >> >> /Contents {} 0 R >>",
                s.breite_mm * MM, s.hoehe_mm * MM, schriften.trim_end(), 4 + i * 2);
            obj(&mut roh, &mut versatz, 3 + i * 2, seite.as_bytes());
            // Inhalt gepackt -- ein Bericht mit vielen Zeilen wird sonst
            // unnoetig gross, und Deflate liegt ohnehin im Baum.
            let gepackt = miniz_oxide::deflate::compress_to_vec_zlib(s.inhalt.as_bytes(), 6);
            versatz.push(roh.len());
            roh.extend_from_slice(format!("{} 0 obj\n", 4 + i * 2).as_bytes());
            roh.extend_from_slice(
                format!("<< /Length {} /Filter /FlateDecode >>\nstream\n", gepackt.len()).as_bytes());
            roh.extend_from_slice(&gepackt);
            roh.extend_from_slice(b"\nendstream\nendobj\n");
        }
        // Schriften -- alle vierzehn, auch die ungenutzten: das kostet ein
        // paar Zeilen und erspart eine Umnummerierung.
        for (i, (_, pdf_name, _)) in SCHRIFTEN.iter().enumerate() {
            let f = format!("<< /Type /Font /Subtype /Type1 /BaseFont /{} /Encoding /WinAnsiEncoding >>",
                            pdf_name);
            obj(&mut roh, &mut versatz, erste_schrift + i, f.as_bytes());
        }
        // Angaben zum Dokument. Sie gehoeren in ein eigenes Objekt und nicht
        // in den Trailer -- dort las sie kein Leser. Gezeigt hat das erst der
        // Test mit einem FREMDEN Leser (PyMuPDF): die Datei war gueltig, der
        // Titel aber leer.
        //
        // KEIN Erstellungsdatum: dasselbe Programm soll zweimal dieselbe
        // Datei ergeben. Das macht Pruefungen vergleichbar und einen
        // Versionsverlauf lesbar -- und wer ein Datum braucht, schreibt es
        // sichtbar auf die Seite, wo es hingehoert.
        let info_nr = erste_schrift + SCHRIFTEN.len();
        let titel = if self.titel.is_empty() { String::new() }
                    else { format!("/Title ({}) ", pdf_roh(&self.titel)) };
        obj(&mut roh, &mut versatz, info_nr,
            format!("<< {}/Producer (Drachenhauch) >>", titel).as_bytes());

        // Querverweistabelle
        let xref_ab = roh.len();
        let anzahl = versatz.len() + 1;
        roh.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", anzahl).as_bytes());
        for v in &versatz {
            roh.extend_from_slice(format!("{:010} 00000 n \n", v).as_bytes());
        }
        roh.extend_from_slice(format!(
            "trailer\n<< /Size {} /Root 1 0 R /Info {} 0 R >>\nstartxref\n{}\n%%EOF\n",
            anzahl, info_nr, xref_ab).as_bytes());
        roh
    }
}

/// Breite eines Textes in Millimetern.
///
/// Bei Courier ist jedes Zeichen 600/1000 der Schriftgroesse breit -- das ist
/// die Bauart der Schrift. Fuer Helvetica und Times (je vier Schnitte) liegen
/// die Schriftmasse in `pdf_masse.rs`: **nicht aus dem Gedaechtnis**, sondern
/// von `tools/gen_pdf_masse.py` aus PyMuPDFs Base-14-Metriken erzeugt und in
/// `tests/test_pdf.py` gegen PyMuPDF nachgemessen. Lange stand hier, eine
/// geschaetzte Breite sei schlimmer als keine -- das galt, solange es nur
/// die Schaetzung gab. Ohne die Masse liess sich in einer Rechnung kein
/// Betrag rechtsbuendig setzen (sechster Pilot, 2026-09-05).
///
/// Symbol und ZapfDingbats bleiben ohne Mass, ebenso ein Zeichen, das die
/// Schrift nicht hat -- beides ein Fehler, keine Schaetzung.
pub fn zeichenbreite(schrift: usize, groesse_pt: f64, text: &str) -> Result<f64, String> {
    use crate::pdf_masse::*;
    let (name, _, fest) = SCHRIFTEN.get(schrift).ok_or("PDF_TEXT_WIDTH: unbekannte Schrift")?;
    if *fest { return Ok(text.chars().count() as f64 * groesse_pt * 0.6 / MM); }
    let tabelle: &[u16; 224] = match schrift {
        0 => &BREITEN_HELVETICA, 1 => &BREITEN_HELVETICA_FETT,
        2 => &BREITEN_HELVETICA_KURSIV, 3 => &BREITEN_HELVETICA_FETT_KURSIV,
        4 => &BREITEN_TIMES, 5 => &BREITEN_TIMES_FETT,
        6 => &BREITEN_TIMES_KURSIV, 7 => &BREITEN_TIMES_FETT_KURSIV,
        _ => return Err(format!("PDF_TEXT_WIDTH: fuer '{}' liegen keine Schriftmasse vor \
                                 (Helvetica, Times und Courier lassen sich messen)", name)),
    };
    let bytes = crate::kodierung::kodieren(text, crate::kodierung::Kodierung::Cp1252, "PDF_TEXT_WIDTH")?;
    let mut summe: u64 = 0;
    for b in bytes {
        if b < 32 { continue; }
        let w = tabelle[(b - 32) as usize];
        if w == 0 {
            return Err(format!("PDF_TEXT_WIDTH: '{}' hat kein Mass fuer das Zeichen '{}'", name, b as char));
        }
        summe += w as u64;
    }
    Ok(summe as f64 * groesse_pt / 1000.0 / MM)
}

/// Text als PDF-Zeichenkette `(...)` -- in WinAnsi (= cp1252) kodiert.
///
/// Ein Zeichen, das cp1252 nicht kennt, ist ein FEHLER und wird nicht durch
/// `?` ersetzt -- dieselbe Regel wie beim Schreiben einer cp1252-Textdatei
/// (Punkt 3). Auf einer Rechnung ist ein stumm verschwundenes Zeichen
/// schlimmer als eine Meldung.
fn pdf_text(s: &str) -> Result<String, String> {
    let bytes = crate::kodierung::kodieren(s, crate::kodierung::Kodierung::Cp1252, "PDF_TEXT")
        .map_err(|e| format!("{} (ein PDF mit den Standardschriften kann nur WinAnsi)", e))?;
    let mut raus = String::from("(");
    for b in bytes {
        match b {
            b'(' | b')' | b'\\' => { raus.push('\\'); raus.push(b as char); }
            // Nicht druckbare und hohe Bytes oktal -- so bleibt die Datei
            // ueberall lesbar, auch wenn ein Werkzeug sie als Text anfasst.
            0..=31 | 127..=255 => raus.push_str(&format!("\\{:03o}", b)),
            _ => raus.push(b as char),
        }
    }
    raus.push(')');
    Ok(raus)
}

/// Wie `pdf_text`, aber ohne Klammern und ohne Fehler (fuer den Titel).
fn pdf_roh(s: &str) -> String {
    s.chars().filter(|c| (*c as u32) < 128 && !matches!(c, '(' | ')' | '\\')).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seitenmasse() {
        assert_eq!(seitenmass("a4", false), Some((210.0, 297.0)));
        assert_eq!(seitenmass("A4", true), Some((297.0, 210.0)));
        assert_eq!(seitenmass("gibtsnicht", false), None);
    }

    #[test]
    fn schriften_werden_gefunden() {
        assert_eq!(schrift_index("helvetica"), Some(0));
        assert_eq!(schrift_index("COURIER-FETT"), Some(9));
        assert_eq!(schrift_index("comic sans"), None);
    }

    #[test]
    fn schriften_lassen_sich_messen() {
        // Courier: jedes Zeichen 600/1000 der Groesse -- das ist die Bauart
        // der Schrift, keine Schaetzung.
        let b = zeichenbreite(8, 12.0, "abcd").unwrap();
        assert!((b - 4.0 * 12.0 * 0.6 / MM).abs() < 1e-9);
        // Helvetica aus den erzeugten Massen: a 556, b 556, c 500, d 556.
        let h = zeichenbreite(0, 12.0, "abcd").unwrap();
        assert!((h - 2168.0 * 12.0 / 1000.0 / MM).abs() < 1e-9, "{}", h);
        // Fett ist breiter als normal, Times schmaler als Helvetica.
        assert!(zeichenbreite(1, 12.0, "Rechnung").unwrap() > h * 0.5);
        assert!(zeichenbreite(4, 12.0, "abcd").unwrap() < h);
        // Ein Euro-Zeichen hat ein Mass, Symbol hat keines.
        assert!(zeichenbreite(0, 12.0, "1,00 \u{20AC}").is_ok());
        assert!(zeichenbreite(12, 12.0, "abcd").is_err());
    }

    #[test]
    fn klammern_und_backslash_werden_geschuetzt() {
        assert_eq!(pdf_text("a(b)c\\d").unwrap(), "(a\\(b\\)c\\\\d)");
    }

    #[test]
    fn umlaute_gehen_oktal_hinein() {
        // 0xE4 = 344 oktal
        assert_eq!(pdf_text("ä").unwrap(), "(\\344)");
    }

    #[test]
    fn was_cp1252_nicht_kennt_ist_ein_fehler() {
        let e = pdf_text("Smiley 😀").unwrap_err();
        assert!(e.contains("WinAnsi"), "{}", e);
    }

    #[test]
    fn ein_leeres_dokument_ist_ein_gueltiges_pdf() {
        let d = Dokument::neu(210.0, 297.0);
        let b = d.bauen();
        assert!(b.starts_with(b"%PDF-1.4"));
        assert!(b.ends_with(b"%%EOF\n"));
        let text = String::from_utf8_lossy(&b);
        assert!(text.contains("/Type /Catalog"));
        assert!(text.contains("startxref"));
    }

    #[test]
    fn seiten_zaehlen() {
        let mut d = Dokument::neu(210.0, 297.0);
        d.neue_seite();
        d.neue_seite();
        assert_eq!(d.seiten.len(), 3);
        let text = String::from_utf8_lossy(&d.bauen()).into_owned();
        assert!(text.contains("/Count 3"), "{}", &text[..300]);
    }

    #[test]
    fn y_wird_von_oben_gezaehlt() {
        let d = Dokument::neu(210.0, 297.0);
        // 0 mm von oben = ganz oben = volle Hoehe in PDF-Punkten.
        assert!((d.y(0.0) - 297.0 * MM).abs() < 1e-9);
        assert!((d.y(297.0)).abs() < 1e-9);
    }
}
