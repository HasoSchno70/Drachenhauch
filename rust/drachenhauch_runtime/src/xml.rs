//! Modul `xml` -- XML lesen (Punkt 7 des Allzweck-Audits).
//!
//! Der Fall dafuer ist fast immer derselbe: Daten kommen aus einem fremden
//! System. Rechnungen, Ausfuhrlisten, GPX-Spuren, SVG, die Antwort einer
//! aelteren Web-Schnittstelle -- alles XML, und ohne Leser war jede dieser
//! Quellen ausser Reichweite.
//!
//! **Nur lesend.** Anders als bei JSON (wo das Schreiben in Punkt 2 die
//! eigentliche Luecke war) ist ein XML-Baum, den ein GB-Programm selbst baut,
//! die Ausnahme; wer eine XML-Datei schreiben muss, klebt sie mit
//! `XML_ESCAPE$` zusammen, das genau die fuenf Zeichen ersetzt, an denen
//! Handarbeit sonst bricht. Ein Baum zum Bauen waere ein zweites Modul in
//! derselben Groesse -- das lohnt erst, wenn jemand es braucht.
//!
//! **Namensraeume bleiben stehen.** `<ns:titel>` heisst hier `ns:titel`, das
//! `xmlns` ist ein gewoehnliches Attribut. Echte Namensraum-Aufloesung
//! braucht einen Geltungsbereich je Element und beantwortet eine Frage, die
//! beim Auslesen einer bekannten Datei niemand stellt.

use std::rc::Rc;

/// Ein Stueck Inhalt: entweder Text oder ein Kind-Element.
///
/// Warum nicht `text: String` und `kinder: Vec<..>` NEBENEINANDER: dann geht
/// die Reihenfolge verloren. `<p>Hallo <b>schoene</b> Welt</p>` haette dann
/// den Text "Hallo  Welt" und irgendwo daneben ein `<b>` -- `text_tief()`
/// koennte daraus nie wieder "Hallo schoene Welt" machen. Der Fehler faellt
/// bei Daten-XML nicht auf (dort ist ein Element entweder Text ODER
/// Elemente) und bei Fliesstext sofort.
#[derive(Debug)]
pub enum Teil {
    Text(String),
    Kind(Rc<Knoten>),
}

#[derive(Debug)]
pub struct Knoten {
    pub name: String,
    pub attribute: Vec<(String, String)>,
    /// Inhalt in der Reihenfolge, in der er in der Datei steht.
    pub teile: Vec<Teil>,
}

impl Knoten {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attribute.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    /// Die Kind-Elemente, in ihrer Reihenfolge.
    pub fn kinder(&self) -> impl Iterator<Item = &Rc<Knoten>> {
        self.teile.iter().filter_map(|t| match t { Teil::Kind(k) => Some(k), _ => None })
    }

    pub fn anzahl_kinder(&self) -> usize { self.kinder().count() }

    /// Der unmittelbare Text dieses Elements (ohne den seiner Kinder).
    pub fn text(&self) -> String {
        self.teile.iter().filter_map(|t| match t { Teil::Text(s) => Some(s.as_str()), _ => None })
            .collect::<Vec<_>>().join("")
    }

    /// Der Text dieses Elements UND aller Nachfahren, in der richtigen
    /// Reihenfolge -- das, was ein Mensch als "Inhalt" sehen wuerde.
    pub fn text_tief(&self) -> String {
        let mut s = String::new();
        for t in &self.teile {
            match t {
                Teil::Text(x) => s.push_str(x),
                Teil::Kind(k) => s.push_str(&k.text_tief()),
            }
        }
        s
    }
}

/// Einen Pfad wie `"buch/titel"` von diesem Knoten aus verfolgen.
///
/// Bewusst dieselbe Schreibweise wie bei JSON, nur mit `/` statt `.` (so
/// steht es in jedem XML-Beispiel der Welt). Ein leerer Pfad meint den
/// Knoten selbst.
///
/// Gibt es einen Namen mehrfach, gilt der ERSTE -- fuer alle anderen gibt es
/// `XML_COUNT` und `XML_AT`. Ein Pfad, der raet, waere der sicherste Weg,
/// beim zweiten Datensatz etwas anderes zu bekommen.
pub fn folge<'a>(wurzel: &'a Rc<Knoten>, pfad: &str) -> Option<Rc<Knoten>> {
    let mut hier = wurzel.clone();
    for teil in pfad.split('/').filter(|t| !t.is_empty()) {
        let naechst = hier.kinder().find(|k| k.name == teil)?.clone();
        hier = naechst;
    }
    Some(hier)
}

/// Alle Kinder am Pfad -- fuer `XML_COUNT`/`XML_AT`.
///
/// `"posten"` liefert alle direkten Kinder dieses Namens, `"rechnung/posten"`
/// alle `posten` unter dem ersten `rechnung`.
pub fn alle(wurzel: &Rc<Knoten>, pfad: &str) -> Vec<Rc<Knoten>> {
    let teile: Vec<&str> = pfad.split('/').filter(|t| !t.is_empty()).collect();
    let Some((letztes, davor)) = teile.split_last() else { return vec![wurzel.clone()] };
    let mut hier = wurzel.clone();
    for t in davor {
        let naechst = hier.kinder().find(|k| k.name == *t).cloned();
        match naechst {
            Some(k) => hier = k,
            None => return Vec::new(),
        }
    }
    hier.kinder().filter(|k| k.name == *letztes).cloned().collect()
}

#[derive(Debug)]
pub struct Fehler {
    pub zeile: usize,
    pub was: String,
}

impl Fehler {
    fn neu(zeile: usize, was: &str) -> Fehler { Fehler { zeile, was: was.to_string() } }
}

/// XML zu einem Baum.
///
/// Bewusst STRENG (anders als das INI-Modul nebenan): eine XML-Datei kommt
/// aus einem anderen Programm, nicht aus einem Texteditor. Ein nicht
/// geschlossenes Element ist dort kein Tippfehler eines Menschen, sondern ein
/// Zeichen, dass die Uebertragung abgebrochen ist -- und dann ist stilles
/// Weiterlesen die schlechteste Antwort.
pub fn lesen(quelle: &str) -> Result<Rc<Knoten>, Fehler> {
    let z: Vec<char> = quelle.chars().collect();
    let mut i = 0usize;
    let mut zeile = 1usize;
    // Stapel offener Elemente; das unterste ist eine Huelle, damit auch ein
    // Dokument mit fuehrendem Kommentar oder Deklaration einen Platz hat.
    let mut stapel: Vec<Knoten> = vec![Knoten {
        name: String::new(), attribute: vec![], teile: vec![] }];

    while i < z.len() {
        if z[i] == '<' {
            // <?xml ... ?>  /  <!-- ... -->  /  <![CDATA[ ... ]]>  /  <!DOCTYPE ...>
            if starts(&z, i, "<?") {
                i = bis(&z, i, "?>", &mut zeile).ok_or_else(|| Fehler::neu(zeile, "unbeendete <?...?>-Anweisung"))?;
                continue;
            }
            if starts(&z, i, "<!--") {
                i = bis(&z, i, "-->", &mut zeile).ok_or_else(|| Fehler::neu(zeile, "unbeendeter Kommentar"))?;
                continue;
            }
            if starts(&z, i, "<![CDATA[") {
                let start = i + 9;
                let ende = finde(&z, start, "]]>").ok_or_else(|| Fehler::neu(zeile, "unbeendeter CDATA-Abschnitt"))?;
                let roh: String = z[start..ende].iter().collect();
                zeile += roh.matches('\n').count();
                // CDATA ist woertlich -- KEINE Entity-Aufloesung.
                stapel.last_mut().unwrap().teile.push(Teil::Text(roh));
                i = ende + 3;
                continue;
            }
            if starts(&z, i, "<!") {
                i = bis(&z, i, ">", &mut zeile).ok_or_else(|| Fehler::neu(zeile, "unbeendete <!...>-Angabe"))?;
                continue;
            }
            if starts(&z, i, "</") {
                let ende = finde(&z, i, ">").ok_or_else(|| Fehler::neu(zeile, "unbeendetes Schluss-Element"))?;
                let name: String = z[i + 2..ende].iter().collect::<String>().trim().to_string();
                if stapel.len() <= 1 {
                    return Err(Fehler::neu(zeile, &format!("</{}> ohne passendes oeffnendes Element", name)));
                }
                let fertig = stapel.pop().unwrap();
                if fertig.name != name {
                    return Err(Fehler::neu(zeile, &format!(
                        "</{}> schliesst <{}> -- die Namen muessen zusammenpassen", name, fertig.name)));
                }
                stapel.last_mut().unwrap().teile.push(Teil::Kind(Rc::new(fertig)));
                i = ende + 1;
                continue;
            }
            // Ein oeffnendes Element.
            let ende = finde(&z, i, ">").ok_or_else(|| Fehler::neu(zeile, "unbeendetes Element"))?;
            let roh: String = z[i + 1..ende].iter().collect();
            zeile += roh.matches('\n').count();
            let selbst_schliessend = roh.trim_end().ends_with('/');
            let inhalt = roh.trim_end().trim_end_matches('/');
            let (name, attribute) = kopf_lesen(inhalt, zeile)?;
            if name.is_empty() { return Err(Fehler::neu(zeile, "Element ohne Namen")); }
            let k = Knoten { name, attribute, teile: vec![] };
            if selbst_schliessend {
                stapel.last_mut().unwrap().teile.push(Teil::Kind(Rc::new(k)));
            } else {
                stapel.push(k);
            }
            i = ende + 1;
            continue;
        }
        // Text bis zum naechsten `<`.
        let start = i;
        while i < z.len() && z[i] != '<' { if z[i] == '\n' { zeile += 1; } i += 1; }
        let roh: String = z[start..i].iter().collect();
        if !roh.trim().is_empty() || stapel.len() > 1 {
            stapel.last_mut().unwrap().teile.push(Teil::Text(entity_auf(&roh)));
        }
    }
    if stapel.len() > 1 {
        let offen = stapel.last().unwrap().name.clone();
        return Err(Fehler::neu(zeile, &format!("<{}> wurde nie geschlossen", offen)));
    }
    let huelle = stapel.pop().unwrap();
    // Ein wohlgeformtes Dokument hat genau ein Wurzelelement. Mehrere sind
    // ein Fehler, keines auch -- beides heisst, dass die Datei nicht das ist,
    // wofuer das Programm sie haelt.
    let wurzeln: Vec<Rc<Knoten>> = huelle.kinder().cloned().collect();
    match wurzeln.len() {
        1 => Ok(wurzeln[0].clone()),
        0 => Err(Fehler::neu(zeile, "kein Element gefunden -- ist das wirklich XML?")),
        n => Err(Fehler::neu(zeile, &format!("{} Wurzelelemente -- XML erlaubt genau eines", n))),
    }
}

/// `name attr="wert" attr2='wert'` zerlegen.
fn kopf_lesen(s: &str, zeile: usize) -> Result<(String, Vec<(String, String)>), Fehler> {
    let z: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    while i < z.len() && !z[i].is_whitespace() { i += 1; }
    let name: String = z[..i].iter().collect();
    let mut attribute: Vec<(String, String)> = Vec::new();
    while i < z.len() {
        while i < z.len() && z[i].is_whitespace() { i += 1; }
        if i >= z.len() { break; }
        let start = i;
        while i < z.len() && z[i] != '=' && !z[i].is_whitespace() { i += 1; }
        let k: String = z[start..i].iter().collect();
        if k.is_empty() { break; }
        while i < z.len() && z[i].is_whitespace() { i += 1; }
        if i >= z.len() || z[i] != '=' {
            // Ein Attribut ohne Wert (HTML-Gewohnheit) -- in XML nicht
            // erlaubt, aber es kostet nichts, es als leer zu nehmen.
            attribute.push((k, String::new()));
            continue;
        }
        i += 1;
        while i < z.len() && z[i].is_whitespace() { i += 1; }
        let anfuehrung = if i < z.len() && (z[i] == '"' || z[i] == '\'') { let c = z[i]; i += 1; Some(c) } else { None };
        let wstart = i;
        match anfuehrung {
            Some(c) => { while i < z.len() && z[i] != c { i += 1; }
                         if i >= z.len() { return Err(Fehler::neu(zeile, &format!("Attribut '{}' ohne schliessendes Anfuehrungszeichen", k))); } }
            None => { while i < z.len() && !z[i].is_whitespace() { i += 1; } }
        }
        let w: String = z[wstart..i].iter().collect();
        if anfuehrung.is_some() { i += 1; }
        attribute.push((k, entity_auf(&w)));
    }
    Ok((name, attribute))
}

/// Die fuenf benannten Entities und die Zahl-Schreibweisen aufloesen.
pub fn entity_auf(s: &str) -> String {
    if !s.contains('&') { return s.to_string(); }
    let z: Vec<char> = s.chars().collect();
    let mut raus = String::with_capacity(s.len());
    let mut i = 0;
    while i < z.len() {
        if z[i] != '&' { raus.push(z[i]); i += 1; continue; }
        let Some(ende) = z[i..].iter().position(|c| *c == ';').map(|p| i + p) else {
            // Ein `&` ohne `;` ist in echten Dateien haeufig genug (und
            // gemeint), um es stehen zu lassen statt zu meckern.
            raus.push('&'); i += 1; continue;
        };
        let name: String = z[i + 1..ende].iter().collect();
        let ersatz = match name.as_str() {
            "lt" => Some('<'), "gt" => Some('>'), "amp" => Some('&'),
            "quot" => Some('"'), "apos" => Some('\''),
            _ => {
                if let Some(hex) = name.strip_prefix("#x").or_else(|| name.strip_prefix("#X")) {
                    u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                } else if let Some(dez) = name.strip_prefix('#') {
                    dez.parse::<u32>().ok().and_then(char::from_u32)
                } else { None }
            }
        };
        match ersatz {
            Some(c) => { raus.push(c); i = ende + 1; }
            None => { raus.push('&'); i += 1; }
        }
    }
    raus
}

/// Die fuenf Zeichen ersetzen, an denen von Hand gebautes XML bricht.
pub fn entity_zu(s: &str) -> String {
    let mut raus = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => raus.push_str("&amp;"),
            '<' => raus.push_str("&lt;"),
            '>' => raus.push_str("&gt;"),
            '"' => raus.push_str("&quot;"),
            '\'' => raus.push_str("&apos;"),
            _ => raus.push(c),
        }
    }
    raus
}

// --- kleine Helfer ---------------------------------------------------------

fn starts(z: &[char], i: usize, s: &str) -> bool {
    let m: Vec<char> = s.chars().collect();
    i + m.len() <= z.len() && z[i..i + m.len()] == m[..]
}

fn finde(z: &[char], ab: usize, s: &str) -> Option<usize> {
    let m: Vec<char> = s.chars().collect();
    (ab..z.len().saturating_sub(m.len() - 1)).find(|&i| z[i..i + m.len()] == m[..])
}

/// Bis hinter `s` springen und dabei die Zeilen mitzaehlen.
fn bis(z: &[char], ab: usize, s: &str, zeile: &mut usize) -> Option<usize> {
    let e = finde(z, ab, s)?;
    *zeile += z[ab..e].iter().filter(|c| **c == '\n').count();
    Some(e + s.chars().count())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> Rc<Knoten> { lesen(s).unwrap() }

    #[test]
    fn elemente_attribute_text() {
        let d = w(r#"<buch jahr="1979"><titel>Per Anhalter</titel></buch>"#);
        assert_eq!(d.name, "buch");
        assert_eq!(d.attr("jahr"), Some("1979"));
        assert_eq!(folge(&d, "titel").unwrap().text(), "Per Anhalter");
    }

    #[test]
    fn selbstschliessend_und_verschachtelt() {
        let d = w("<a><b/><c><d>x</d></c></a>");
        assert_eq!(d.anzahl_kinder(), 2);
        assert_eq!(folge(&d, "c/d").unwrap().text(), "x");
        assert_eq!(folge(&d, "b").unwrap().anzahl_kinder(), 0);
    }

    #[test]
    fn deklaration_kommentar_doctype_stoeren_nicht() {
        let d = w("<?xml version=\"1.0\"?>\n<!-- Hinweis -->\n<!DOCTYPE a>\n<a>x</a>");
        assert_eq!(d.name, "a");
        assert_eq!(d.text(), "x");
    }

    #[test]
    fn entities_und_cdata() {
        let d = w("<a>5 &lt; 6 &amp; 7 &#65;</a>");
        assert_eq!(d.text(), "5 < 6 & 7 A");
        let c = w("<a><![CDATA[roh & <ungeschuetzt>]]></a>");
        assert_eq!(c.text(), "roh & <ungeschuetzt>");
    }

    #[test]
    fn attribute_in_beiden_anfuehrungszeichen() {
        let d = w("<a x=\"1\" y='zwei drei' z=\"&amp;\"/>");
        assert_eq!(d.anzahl_kinder(), 0);
        assert_eq!(d.attr("x"), Some("1"));
        assert_eq!(d.attr("y"), Some("zwei drei"));
        assert_eq!(d.attr("z"), Some("&"));
    }

    #[test]
    fn mehrere_gleiche_kinder() {
        let d = w("<liste><p>a</p><p>b</p><p>c</p></liste>");
        assert_eq!(alle(&d, "p").len(), 3);
        assert_eq!(alle(&d, "p")[1].text(), "b");
        // `folge` nimmt den ersten -- Raten waere der sicherste Weg, beim
        // zweiten Datensatz etwas anderes zu bekommen.
        assert_eq!(folge(&d, "p").unwrap().text(), "a");
    }

    #[test]
    fn text_tief_sammelt_alles_ein() {
        let d = w("<p>Hallo <b>schoene</b> Welt</p>");
        assert_eq!(d.text(), "Hallo  Welt");
        // Und HIER liegt der Unterschied: die Reihenfolge stimmt.
        assert_eq!(d.text_tief(), "Hallo schoene Welt");
    }

    #[test]
    fn namensraeume_bleiben_im_namen() {
        let d = w(r#"<ns:a xmlns:ns="http://x"><ns:b>1</ns:b></ns:a>"#);
        assert_eq!(d.name, "ns:a");
        assert_eq!(d.attr("xmlns:ns"), Some("http://x"));
        assert_eq!(folge(&d, "ns:b").unwrap().text(), "1");
    }

    #[test]
    fn fehler_nennen_die_zeile() {
        let e = lesen("<a>\n<b>\n</a>").unwrap_err();
        assert!(e.was.contains("</a> schliesst <b>"), "{}", e.was);
        assert_eq!(e.zeile, 3);
        assert!(lesen("<a><b></b>").unwrap_err().was.contains("nie geschlossen"));
        assert!(lesen("kein xml").unwrap_err().was.contains("wirklich XML"));
        assert!(lesen("<a/><b/>").unwrap_err().was.contains("Wurzelelemente"));
    }

    #[test]
    fn escape_und_zurueck() {
        let roh = "5 < 6 & \"sieben\" 'acht'";
        assert_eq!(entity_auf(&entity_zu(roh)), roh);
        assert_eq!(entity_zu("<a>"), "&lt;a&gt;");
    }
}
