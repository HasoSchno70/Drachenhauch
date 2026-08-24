//! Modul `xlsx` -- Auswertungen als Excel-Mappe (Punkt 7 des
//! Allzweck-Audits).
//!
//! CSV gab es schon, und zum Weitergeben von Daten reicht es. Was CSV nicht
//! kann: mehrere Blaetter, eine fette Kopfzeile, Spaltenbreiten, Zahlen- und
//! Datumsformate -- alles, was aus einer Datenliste eine ABLIEFERBARE
//! Auswertung macht. Und es kann keine Zahl von einem Text unterscheiden:
//! eine Postleitzahl `01067` wird beim Oeffnen in Excel zu `1067`.
//!
//! **Nur schreibend.** Zum LESEN einer Tabelle ist CSV der Weg (Excel
//! exportiert es, und seit Punkt 3 liest Drachenhauch auch cp1252). Ein
//! xlsx-Leser muesste geteilte Zeichenketten und Formatvorlagen aufloesen --
//! ein eigenes Modul fuer einen Fall, den CSV schon deckt.
//!
//! **Eine .xlsx-Datei ist ein ZIP mit XML-Teilen.** Beides liegt bereits im
//! Baum (`zipdatei.rs`, `xml`-Kenntnis), deshalb ist dieses Modul vor allem
//! Fleissarbeit an den Teilen, die Excel erwartet.

/// Wo eine Zahl in Excel steht: Tage seit dem 30.12.1899. Der Versatz zur
/// Unix-Zeit sind 25569 Tage.
///
/// Warum der 30. und nicht der 31.: Excel glaubt, 1900 sei ein Schaltjahr
/// gewesen (ein uebernommener Fehler aus Lotus 1-2-3). Der um einen Tag
/// verschobene Nullpunkt macht diesen Fehler fuer alle Daten ab dem 1.3.1900
/// wieder wett -- und das sind alle, die in einer Auswertung vorkommen.
const EXCEL_VERSATZ: f64 = 25569.0;

#[derive(Clone)]
pub enum Wert {
    Text(String),
    Zahl(f64),
}

#[derive(Clone)]
pub struct Zelle {
    pub wert: Wert,
    pub fett: bool,
    /// Zahlenformat wie in Excel (`0.00`, `#,##0.00`, `DD.MM.YYYY`);
    /// leer = Standard.
    pub format: String,
}

pub struct Blatt {
    pub name: String,
    /// Nur belegte Zellen: (zeile, spalte) -> Zelle. Eine Auswertung mit
    /// einer Spalte und tausend Zeilen soll keine Tabelle von tausend mal
    /// tausend anlegen.
    pub zellen: std::collections::BTreeMap<(u32, u32), Zelle>,
    pub breiten: std::collections::BTreeMap<u32, f64>,
}

pub struct Mappe {
    pub blaetter: Vec<Blatt>,
    /// Auf welches Blatt sich die naechsten Aufrufe beziehen.
    pub aktiv: usize,
}

/// Zeichen, die Excel in einem Blattnamen nicht annimmt.
const VERBOTEN: &[char] = &['[', ']', ':', '*', '?', '/', '\\'];

pub fn pruefe_blattname(name: &str) -> Result<(), String> {
    if name.is_empty() { return Err("Blattname ist leer".to_string()); }
    if name.chars().count() > 31 {
        return Err(format!("Blattname '{}' ist laenger als 31 Zeichen -- mehr nimmt Excel nicht", name));
    }
    if let Some(c) = name.chars().find(|c| VERBOTEN.contains(c)) {
        return Err(format!("Blattname '{}' enthaelt '{}' -- Excel erlaubt kein []:*?/\\", name, c));
    }
    if name.starts_with('\'') || name.ends_with('\'') {
        return Err(format!("Blattname '{}' darf nicht mit einem Apostroph anfangen oder enden", name));
    }
    Ok(())
}

impl Mappe {
    pub fn neu(erstes_blatt: &str) -> Result<Mappe, String> {
        pruefe_blattname(erstes_blatt)?;
        Ok(Mappe { blaetter: vec![Blatt::neu(erstes_blatt)], aktiv: 0 })
    }

    pub fn blatt_dazu(&mut self, name: &str) -> Result<(), String> {
        pruefe_blattname(name)?;
        if self.blaetter.iter().any(|b| b.name.eq_ignore_ascii_case(name)) {
            return Err(format!("Es gibt schon ein Blatt '{}' -- Excel unterscheidet die Namen nicht nach Gross- und Kleinschreibung", name));
        }
        self.blaetter.push(Blatt::neu(name));
        self.aktiv = self.blaetter.len() - 1;
        Ok(())
    }

    pub fn hier(&mut self) -> &mut Blatt { &mut self.blaetter[self.aktiv] }

    pub fn setze(&mut self, z: u32, s: u32, wert: Wert) {
        let b = self.hier();
        match b.zellen.get_mut(&(z, s)) {
            // Eine schon gesetzte Zelle behaelt ihre Gestaltung -- sonst
            // muesste man nach jeder Wertaenderung fett und Format neu
            // setzen.
            Some(alt) => alt.wert = wert,
            None => { b.zellen.insert((z, s), Zelle { wert, fett: false, format: String::new() }); },
        }
    }

    /// Gestaltung auch fuer eine Zelle, die noch keinen Wert hat -- damit
    /// laesst sich eine Kopfzeile vorbereiten.
    fn zelle_mut(&mut self, z: u32, s: u32) -> &mut Zelle {
        self.hier().zellen.entry((z, s))
            .or_insert(Zelle { wert: Wert::Text(String::new()), fett: false, format: String::new() })
    }

    pub fn setze_fett(&mut self, z: u32, s: u32, an: bool) { self.zelle_mut(z, s).fett = an; }

    pub fn setze_format(&mut self, z: u32, s: u32, f: &str) {
        self.zelle_mut(z, s).format = f.to_string();
    }

    /// Eine ganze Zeile fett -- die Kopfzeile ist der haeufigste Fall.
    ///
    /// Wirkt auf die Zellen, die es GIBT: eine Kopfzeile wird geschrieben und
    /// dann fett gesetzt, nicht umgekehrt.
    pub fn setze_zeile_fett(&mut self, z: u32, an: bool) {
        let b = self.hier();
        for ((zz, _), zelle) in b.zellen.iter_mut() {
            if *zz == z { zelle.fett = an; }
        }
    }

    pub fn setze_breite(&mut self, s: u32, breite: f64) {
        self.hier().breiten.insert(s, breite);
    }
}

impl Blatt {
    fn neu(name: &str) -> Blatt {
        Blatt { name: name.to_string(), zellen: Default::default(), breiten: Default::default() }
    }
}

/// Eine Zeit (Sekunden seit 1970, wie im Modul `zeit`) als Excel-Zahl.
pub fn zeit_zu_excel(sekunden: i64) -> f64 {
    sekunden as f64 / 86_400.0 + EXCEL_VERSATZ
}

/// Spaltennummer zu Excel-Buchstaben: 0 -> A, 25 -> Z, 26 -> AA.
pub fn spaltenname(mut s: u32) -> String {
    let mut raus = Vec::new();
    loop {
        raus.push((b'A' + (s % 26) as u8) as char);
        if s < 26 { break; }
        s = s / 26 - 1;
    }
    raus.iter().rev().collect()
}

fn xml_text(s: &str) -> String { crate::xml::entity_zu(s) }

/// Die Teile der Datei bauen -- fertig zum Packen.
pub fn teile(m: &Mappe) -> Vec<(String, Vec<u8>)> {
    // Alle vorkommenden Gestaltungen einsammeln: (fett, format).
    // Eintrag 0 ist immer der Standard, damit eine Zelle ohne Gestaltung
    // keine braucht.
    let mut stile: Vec<(bool, String)> = vec![(false, String::new())];
    for b in &m.blaetter {
        for z in b.zellen.values() {
            let s = (z.fett, z.format.clone());
            if s != (false, String::new()) && !stile.contains(&s) { stile.push(s); }
        }
    }
    let mut formate: Vec<String> = Vec::new();
    for (_, f) in &stile {
        if !f.is_empty() && !formate.contains(f) { formate.push(f.clone()); }
    }

    let mut raus: Vec<(String, Vec<u8>)> = Vec::new();
    let kopf = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";
    let ns = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
    let rel_ns = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

    // [Content_Types].xml
    let mut ct = String::from(kopf);
    ct.push_str("<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">");
    ct.push_str("<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>");
    ct.push_str("<Default Extension=\"xml\" ContentType=\"application/xml\"/>");
    ct.push_str("<Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>");
    for i in 0..m.blaetter.len() {
        ct.push_str(&format!("<Override PartName=\"/xl/worksheets/sheet{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>", i + 1));
    }
    ct.push_str("<Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>");
    ct.push_str("</Types>");
    raus.push(("[Content_Types].xml".into(), ct.into_bytes()));

    // _rels/.rels
    let rels = format!("{}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" Type=\"{}/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>",
        kopf, rel_ns);
    raus.push(("_rels/.rels".into(), rels.into_bytes()));

    // xl/workbook.xml
    let mut wb = format!("{}<workbook xmlns=\"{}\" xmlns:r=\"{}\"><sheets>", kopf, ns, rel_ns);
    for (i, b) in m.blaetter.iter().enumerate() {
        wb.push_str(&format!("<sheet name=\"{}\" sheetId=\"{}\" r:id=\"rId{}\"/>",
                             xml_text(&b.name), i + 1, i + 1));
    }
    wb.push_str("</sheets></workbook>");
    raus.push(("xl/workbook.xml".into(), wb.into_bytes()));

    // xl/_rels/workbook.xml.rels
    let mut wr = format!("{}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">", kopf);
    for i in 0..m.blaetter.len() {
        wr.push_str(&format!("<Relationship Id=\"rId{}\" Type=\"{}/worksheet\" Target=\"worksheets/sheet{}.xml\"/>",
                             i + 1, rel_ns, i + 1));
    }
    wr.push_str(&format!("<Relationship Id=\"rId{}\" Type=\"{}/styles\" Target=\"styles.xml\"/>",
                         m.blaetter.len() + 1, rel_ns));
    wr.push_str("</Relationships>");
    raus.push(("xl/_rels/workbook.xml.rels".into(), wr.into_bytes()));

    // xl/styles.xml
    let mut st = format!("{}<styleSheet xmlns=\"{}\">", kopf, ns);
    if !formate.is_empty() {
        st.push_str(&format!("<numFmts count=\"{}\">", formate.len()));
        for (i, f) in formate.iter().enumerate() {
            // Ab 164 sind die Nummern frei; darunter liegen Excels eigene.
            st.push_str(&format!("<numFmt numFmtId=\"{}\" formatCode=\"{}\"/>", 164 + i, xml_text(f)));
        }
        st.push_str("</numFmts>");
    }
    st.push_str("<fonts count=\"2\"><font><sz val=\"11\"/><name val=\"Calibri\"/></font>\
                 <font><b/><sz val=\"11\"/><name val=\"Calibri\"/></font></fonts>");
    // Excel besteht auf genau diesen beiden Fuellungen an Platz 0 und 1 --
    // eine Mappe ohne sie oeffnet es als beschaedigt.
    st.push_str("<fills count=\"2\"><fill><patternFill patternType=\"none\"/></fill>\
                 <fill><patternFill patternType=\"gray125\"/></fill></fills>");
    st.push_str("<borders count=\"1\"><border/></borders>");
    st.push_str("<cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>");
    st.push_str(&format!("<cellXfs count=\"{}\">", stile.len()));
    for (fett, format) in &stile {
        let nf = match formate.iter().position(|f| f == format) {
            Some(i) if !format.is_empty() => 164 + i,
            _ => 0,
        };
        st.push_str(&format!(
            "<xf numFmtId=\"{}\" fontId=\"{}\" fillId=\"0\" borderId=\"0\" xfId=\"0\"{}{}/>",
            nf, if *fett { 1 } else { 0 },
            if nf != 0 { " applyNumberFormat=\"1\"" } else { "" },
            if *fett { " applyFont=\"1\"" } else { "" }));
    }
    // Die benannte Vorlage "Standard" gehoert dazu, auch wenn niemand sie
    // benutzt: ohne sie meldet ein strenger Leser "Workbook contains no
    // default style" und setzt seine eigene ein. Aufgefallen beim
    // Gegenlesen mit openpyxl.
    st.push_str("</cellXfs><cellStyles count=\"1\">                 <cellStyle name=\"Standard\" xfId=\"0\" builtinId=\"0\"/></cellStyles>");
    st.push_str("</styleSheet>");
    raus.push(("xl/styles.xml".into(), st.into_bytes()));

    // Die Blaetter
    for (i, b) in m.blaetter.iter().enumerate() {
        let mut sh = format!("{}<worksheet xmlns=\"{}\">", kopf, ns);
        if !b.breiten.is_empty() {
            sh.push_str("<cols>");
            for (s, w) in &b.breiten {
                sh.push_str(&format!("<col min=\"{}\" max=\"{}\" width=\"{:.2}\" customWidth=\"1\"/>",
                                     s + 1, s + 1, w));
            }
            sh.push_str("</cols>");
        }
        sh.push_str("<sheetData>");
        let mut zeile_offen: Option<u32> = None;
        for ((z, s), zelle) in &b.zellen {
            if zeile_offen != Some(*z) {
                if zeile_offen.is_some() { sh.push_str("</row>"); }
                sh.push_str(&format!("<row r=\"{}\">", z + 1));
                zeile_offen = Some(*z);
            }
            let stil = stile.iter().position(|(f, fo)| *f == zelle.fett && *fo == zelle.format).unwrap_or(0);
            let s_attr = if stil == 0 { String::new() } else { format!(" s=\"{}\"", stil) };
            let bezug = format!("{}{}", spaltenname(*s), z + 1);
            match &zelle.wert {
                // `inlineStr` statt der geteilten Zeichenketten-Tabelle: die
                // spart bei vielen gleichen Texten Platz, kostet aber ein
                // weiteres Teil und eine zweite Buchfuehrung. Fuer eine
                // Auswertung faellt beides nicht ins Gewicht.
                Wert::Text(t) if !t.is_empty() => sh.push_str(&format!(
                    "<c r=\"{}\"{} t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
                    bezug, s_attr, xml_text(t))),
                Wert::Text(_) => sh.push_str(&format!("<c r=\"{}\"{}/>", bezug, s_attr)),
                Wert::Zahl(n) => sh.push_str(&format!("<c r=\"{}\"{}><v>{}</v></c>",
                                                      bezug, s_attr, zahl_text(*n))),
            }
        }
        if zeile_offen.is_some() { sh.push_str("</row>"); }
        sh.push_str("</sheetData></worksheet>");
        raus.push((format!("xl/worksheets/sheet{}.xml", i + 1), sh.into_bytes()));
    }
    raus
}

/// Eine Zahl so schreiben, wie XML sie erwartet: Punkt als Trenner, kein
/// `e`-Kurzschreiben fuer uebliche Groessen, keine unnoetigen Nullen.
fn zahl_text(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 { format!("{}", n as i64) } else { format!("{}", n) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spaltennamen() {
        assert_eq!(spaltenname(0), "A");
        assert_eq!(spaltenname(25), "Z");
        assert_eq!(spaltenname(26), "AA");
        assert_eq!(spaltenname(27), "AB");
        assert_eq!(spaltenname(51), "AZ");
        assert_eq!(spaltenname(52), "BA");
        assert_eq!(spaltenname(701), "ZZ");
        assert_eq!(spaltenname(702), "AAA");
    }

    #[test]
    fn blattnamen_werden_geprueft() {
        assert!(pruefe_blattname("Umsatz").is_ok());
        assert!(pruefe_blattname("").is_err());
        assert!(pruefe_blattname(&"x".repeat(32)).is_err());
        let e = pruefe_blattname("Q1/Q2").unwrap_err();
        assert!(e.contains("erlaubt kein"), "{}", e);
    }

    #[test]
    fn zwei_blaetter_gleichen_namens() {
        let mut m = Mappe::neu("Daten").unwrap();
        assert!(m.blatt_dazu("daten").is_err());
    }

    #[test]
    fn excel_datum() {
        // 1.1.1970 ist in Excel die 25569.
        assert!((zeit_zu_excel(0) - 25569.0).abs() < 1e-9);
        // Ein Tag weiter.
        assert!((zeit_zu_excel(86_400) - 25570.0).abs() < 1e-9);
    }

    #[test]
    fn zahlen_ohne_unnoetige_nullen() {
        assert_eq!(zahl_text(5.0), "5");
        assert_eq!(zahl_text(5.25), "5.25");
        assert_eq!(zahl_text(-3.0), "-3");
    }

    #[test]
    fn gestaltung_ueberlebt_eine_neue_zuweisung() {
        let mut m = Mappe::neu("A").unwrap();
        m.setze(0, 0, Wert::Text("alt".into()));
        m.setze_fett(0, 0, true);
        m.setze(0, 0, Wert::Text("neu".into()));
        let z = &m.blaetter[0].zellen[&(0, 0)];
        assert!(z.fett);
        assert!(matches!(&z.wert, Wert::Text(t) if t == "neu"));
    }

    #[test]
    fn die_teile_sind_vollstaendig() {
        let mut m = Mappe::neu("Eins").unwrap();
        m.blatt_dazu("Zwei").unwrap();
        let namen: Vec<String> = teile(&m).into_iter().map(|(n, _)| n).collect();
        for erwartet in ["[Content_Types].xml", "_rels/.rels", "xl/workbook.xml",
                         "xl/_rels/workbook.xml.rels", "xl/styles.xml",
                         "xl/worksheets/sheet1.xml", "xl/worksheets/sheet2.xml"] {
            assert!(namen.contains(&erwartet.to_string()), "{} fehlt in {:?}", erwartet, namen);
        }
    }

    #[test]
    fn sonderzeichen_im_text_werden_geschuetzt() {
        let mut m = Mappe::neu("A").unwrap();
        m.setze(0, 0, Wert::Text("Schrauben & <Muttern>".into()));
        let t = teile(&m);
        let blatt = t.iter().find(|(n, _)| n.contains("sheet1")).unwrap();
        let s = String::from_utf8_lossy(&blatt.1);
        assert!(s.contains("Schrauben &amp; &lt;Muttern&gt;"), "{}", s);
    }
}
