//! Stildatei (GUI_STYLESHEET, docs/entwurf-gui-wie-gtk.md Paket 4, erste
//! Stufe): das Aussehen der Widgets per Regel statt per Befehl je Widget --
//! wie GTK-CSS, aber nur die Teilmenge, die zur gui passt.
//!
//! ```text
//! button              { radius: 6 }
//! button.gefahr:hover { bg: #E74C3C }
//! textinput:focus     { border: accent }
//! #speichern          { groesse: 16; stil: fett }
//! ```
//!
//! Ein Selektor ist eine Art (die Namen aus der `.dhform`, `*` = jede),
//! Klassen (`.name`, live ueber GUI_ADD_CLASS; die Knopfart zaehlt als
//! Klasse), ein Name (`#name`, GUI_SET_NAME) und Zustaende. Vorrang wie in
//! CSS: Name 100, Klasse und Zustand je 10, Art 1, bei Gleichstand gewinnt
//! die spaetere Regel; GUI_SET_COLOR am Widget schlaegt alles.
//!
//! Aufgeloest wird je Widget EINMAL, wenn sich Stildatei, Klassen, Name,
//! Knopfart, `:checked` oder `:disabled` aendern (`stil_pass` in
//! GUI_UPDATE) -- das Zeichnen vergleicht keine Selektoren. Die
//! fluechtigen Zustaende (`:hover`, `:focus`, `:active`) und `:disabled`
//! liegen als eigene Schichten daneben; gemischt wird beim Zeichnen ueber
//! die Uebergaenge des Widgets (`ueber_t`, `fokus_t`, `druck_t`), so blendet
//! eine Hover-Farbe ein, statt zu springen.
//!
//! Zweite Stufe: Form je Widget -- `abstand` (Innenabstand), `randbreite`,
//! `schatten`, `verlauf`, `glanz`, `fase`, `uebergang`. Sie gelten nur in
//! Regeln ohne fluechtigen Zustand (eine Form, die beim Ueberfahren springt,
//! waere ein Wackeln) und gehen beim Zeichnen ueber `form_jetzt` in
//! `flaeche`, so wie die Rundung ueber `rad_jetzt`.
//!
//! Dritte Stufe: Nachfahren (`panel.leiste button`, `layout > button`,
//! `window#name button`). Die Kette eines Widgets (`ketten`) kommt aus
//! Layout, Reiterwerk/Akkordeon/Assistent, rollendem Panel und zuletzt dem
//! Fenster; gebaut wird sie nur, wenn eine Regel nach Vorfahren fragt, und
//! ihr Hash steht im Schluessel -- eine Klasse weiter oben oder ein Umzug
//! loest neu auf, ohne dass jede Stelle daran denken muss. Dazu `uebergang`
//! je Farbrolle (`Widget::rollen_t`, nur wenn eine Regel es verlangt).
use super::*;

/// Schichten der Aufloesung.
pub(super) const BASIS: usize = 0;
pub(super) const HOVER: usize = 1;
pub(super) const FOKUS: usize = 2;
pub(super) const AKTIV: usize = 3;
pub(super) const AUS: usize = 4;

const EIGENSCHAFTEN: &str = "bg, fg, border, accent, radius, groesse, stil, abstand, randbreite, schatten, verlauf, glanz, fase, uebergang (auch je Farbe: uebergang: 120, bg 400)";
const ZUSTAENDE: &str = ":hover, :focus, :active, :disabled, :checked";

/// Eine Farbe aus der Stildatei: fest oder ein Name des Themas (dann folgt
/// sie einem Themenwechsel).
#[derive(Clone, Debug, PartialEq)]
pub(super) enum StilFarbe { Fest(i64), Thema(String) }

/// Was eine Regel (oder die Aufloesung einer Schicht) setzt.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct StilWerte {
    farben: Vec<(String, StilFarbe)>,
    radius: Option<i32>,
    groesse: Option<i32>,
    stil: Option<u8>,
    /// Innenabstand (senkrecht, waagerecht) in logischen Punkten.
    abstand: Option<(i32, i32)>,
    randbreite: Option<i32>,
    schatten: Option<Schatten>,
    verlauf: Option<i32>,
    glanz: Option<i32>,
    fase: Option<i32>,
    uebergang: Option<i32>,
    /// Dauer je Farbrolle (`uebergang: bg 400, border 0`) -- die Rollen ohne
    /// eigene Dauer nehmen `uebergang` bzw. die Metrik.
    uebergang_je: Vec<(usize, i32)>,
}

/// Die Farbrollen, die eine eigene Dauer haben koennen -- ihre Stelle ist
/// der Platz in `Widget::rollen_t`.
pub(super) const ROLLEN: [&str; 4] = ["bg", "fg", "border", "accent"];

fn rolle_von(s: &str) -> Option<usize> {
    match s.to_lowercase().as_str() {
        "bg" | "hintergrund" | "background" => Some(0),
        "fg" | "farbe" | "text" | "color" => Some(1),
        "border" | "rand" => Some(2),
        "accent" | "akzent" => Some(3),
        _ => None,
    }
}

/// `uebergang: 200` (alles) oder eine Liste `uebergang: 120, bg 400, border 0`:
/// eine nackte Zahl gilt fuer alles, eine Rolle mit Zahl nur fuer diese
/// Farbe. `ms` hinter der Zahl darf stehen.
fn uebergang_lesen(wert: &str, zeile: usize) -> Result<(Option<i32>, Vec<(usize, i32)>), String> {
    let mut alle = None;
    let mut je = Vec::new();
    for stueck in wert.split(',') {
        let teile: Vec<&str> = stueck.split_whitespace().collect();
        let zahl = |s: &str| zahl_lesen_stil(s.trim_end_matches("ms"), zeile, "uebergang", 0, 2000);
        match teile.as_slice() {
            [n] => alle = Some(zahl(n)?),
            [r, n] => match rolle_von(r) {
                Some(k) => { je.retain(|(x, _): &(usize, i32)| *x != k); je.push((k, zahl(n)?)); }
                None => return Err(format!("Zeile {}: uebergang fuer '{}' -- eine eigene Dauer haben bg, fg, border und accent", zeile, r)),
            },
            _ => return Err(format!("Zeile {}: uebergang erwartet eine Zahl oder eine Liste wie '120, bg 400, border 0', erhalten '{}'", zeile, wert.trim())),
        }
    }
    Ok((alle, je))
}

/// Schlagschatten eines Widgets: Versatz, Weichheit, Farbe (ohne Farbe ein
/// halbdurchsichtiges Schwarz). `weich` 0 und Versatz 0 = kein Schatten.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Schatten { dx: i32, dy: i32, weich: i32, farbe: Option<StilFarbe> }

/// Was `flaeche` beim Zeichnen eines Widgets statt der Metriken nimmt.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct FormStil {
    pub verlauf: Option<i32>,
    pub glanz: Option<i32>,
    pub fase: Option<i32>,
    pub rand: Option<i32>,
}

impl StilWerte {
    fn farbe(&self, rolle: &str) -> Option<&StilFarbe> {
        self.farben.iter().find(|(r, _)| r == rolle).map(|(_, f)| f)
    }
    fn uebernehmen(&mut self, o: &StilWerte) {
        for (r, f) in &o.farben {
            match self.farben.iter_mut().find(|(x, _)| x == r) {
                Some(e) => e.1 = f.clone(),
                None => self.farben.push((r.clone(), f.clone())),
            }
        }
        if o.radius.is_some() { self.radius = o.radius; }
        if o.groesse.is_some() { self.groesse = o.groesse; }
        if o.stil.is_some() { self.stil = o.stil; }
        if o.abstand.is_some() { self.abstand = o.abstand; }
        if o.randbreite.is_some() { self.randbreite = o.randbreite; }
        if o.schatten.is_some() { self.schatten = o.schatten.clone(); }
        if o.verlauf.is_some() { self.verlauf = o.verlauf; }
        if o.glanz.is_some() { self.glanz = o.glanz; }
        if o.fase.is_some() { self.fase = o.fase; }
        if o.uebergang.is_some() { self.uebergang = o.uebergang; }
        for &(k, d) in &o.uebergang_je {
            self.uebergang_je.retain(|(x, _)| *x != k);
            self.uebergang_je.push((k, d));
        }
    }
}

#[derive(Clone, Debug)]
struct Selektor {
    art: Option<String>,
    klassen: Vec<String>,
    name: Option<String>,
    an: bool,
    schicht: usize,
    gewicht: u32,
    /// Die Teile davor (`panel.leiste button`), das naechste zuerst; `true`
    /// = direkt darueber (`>`), sonst irgendwo darueber.
    vorfahren: Vec<(Teil, bool)>,
}

/// Ein Teil eines Selektors ohne Zustand: so wird ein Vorfahre beschrieben.
#[derive(Clone, Debug, Default)]
struct Teil { art: Option<String>, klassen: Vec<String>, name: Option<String> }

/// Ein Glied der Kette ueber einem Widget: seine Behaelter (Layout,
/// rollendes Panel, Reiterwerk, Akkordeon, Assistent), zuletzt das Fenster.
#[derive(Clone, Debug, Default, Hash)]
pub(super) struct Glied { pub art: String, pub klassen: Vec<String>, pub name: String }

impl Teil {
    fn passt(&self, g: &Glied) -> bool {
        self.art.as_ref().map(|a| *a == g.art).unwrap_or(true)
            && self.klassen.iter().all(|k| g.klassen.iter().any(|x| x == k))
            && self.name.as_ref().map(|n| *n == g.name).unwrap_or(true)
    }
}

/// Passen die Vorfahren auf die Kette (das naechste Glied zuerst)? Ein Teil
/// ohne `>` darf ueber beliebig viele Glieder hinweg passen -- dann wird
/// jede Lage probiert, wie in CSS.
fn vorfahren_passen(vf: &[(Teil, bool)], kette: &[Glied]) -> bool {
    let Some(((t, direkt), rest)) = vf.split_first() else { return true };
    if *direkt {
        return kette.first().map(|g| t.passt(g)).unwrap_or(false) && vorfahren_passen(rest, &kette[1..]);
    }
    (0..kette.len()).any(|k| t.passt(&kette[k]) && vorfahren_passen(rest, &kette[k + 1..]))
}

#[derive(Clone, Debug)]
struct Regel { sel: Selektor, werte: StilWerte }

/// Eine gelesene Stildatei.
#[derive(Clone, Debug, Default)]
pub(super) struct Stildatei {
    regeln: Vec<Regel>,
    /// Hat irgendeine Regel Vorfahren? Nur dann muss `stil_pass` die Kette
    /// ueber jedem Widget bauen.
    pub(super) mit_vorfahren: bool,
}

/// Die Aufloesung eines Widgets: je Schicht, was gilt.
pub(super) type StilAufgeloest = [StilWerte; 5];

fn zeile_von(text: &str, pos: usize) -> usize { text[..pos.min(text.len())].matches('\n').count() + 1 }

/// Kommentare (`/* */` und `//` bis zum Zeilenende) durch Leerzeichen
/// ersetzen -- die Zeilen bleiben, damit eine Meldung die richtige nennt.
fn ohne_kommentare(text: &str) -> Result<String, String> {
    let z: Vec<char> = text.chars().collect();
    let mut o = String::with_capacity(text.len());
    let mut i = 0;
    let mut zeile = 1;
    while i < z.len() {
        if z[i] == '/' && z.get(i + 1) == Some(&'*') {
            let start = zeile;
            i += 2;
            o.push_str("  ");
            loop {
                if i >= z.len() { return Err(format!("Zeile {}: Kommentar ohne */", start)); }
                if z[i] == '*' && z.get(i + 1) == Some(&'/') { o.push_str("  "); i += 2; break; }
                if z[i] == '\n' { zeile += 1; o.push('\n'); } else { o.push(' '); }
                i += 1;
            }
        } else if z[i] == '/' && z.get(i + 1) == Some(&'/') {
            while i < z.len() && z[i] != '\n' { o.push(' '); i += 1; }
        } else {
            if z[i] == '\n' { zeile += 1; }
            o.push(z[i]);
            i += 1;
        }
    }
    Ok(o)
}

fn ist_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Ein Selektor: Teile, getrennt durch Leerraum (irgendwo darunter) oder
/// `>` (direkt darunter). Nur der letzte Teil -- das Widget selbst -- darf
/// Zustaende tragen; die Vorfahren beschreiben Behaelter und Fenster.
fn selektor_lesen(s: &str, zeile: usize) -> Result<Selektor, String> {
    let s = s.trim();
    if s.is_empty() { return Err(format!("Zeile {}: leerer Selektor", zeile)); }
    let gesp = s.replace('>', " > ");
    let mut teile: Vec<&str> = Vec::new();
    let mut direkt: Vec<bool> = Vec::new();   // direkt[i]: zwischen teile[i] und teile[i + 1]
    let mut offen = false;
    for t in gesp.split_whitespace() {
        if t == ">" {
            if teile.is_empty() || offen { return Err(format!("Zeile {}: '{}' -- vor '>' fehlt ein Teil", zeile, s)); }
            offen = true;
            continue;
        }
        if !teile.is_empty() { direkt.push(offen); }
        teile.push(t);
        offen = false;
    }
    if offen { return Err(format!("Zeile {}: '{}' -- hinter '>' fehlt ein Teil", zeile, s)); }
    let mut sel = teil_lesen(teile[teile.len() - 1], zeile, true)?;
    for i in (0..teile.len() - 1).rev() {
        let v = teil_lesen(teile[i], zeile, false)?;
        sel.gewicht += v.gewicht;
        sel.vorfahren.push((Teil { art: v.art, klassen: v.klassen, name: v.name }, direkt[i]));
    }
    Ok(sel)
}

/// Ein Teil ohne Leerraum: vorn die Art, dann `.klasse`, `#name`, `:zustand`.
/// `ziel` = der letzte Teil (das Widget); ein Vorfahre darf `window` sein,
/// aber keinen Zustand tragen.
fn teil_lesen(s: &str, zeile: usize, ziel: bool) -> Result<Selektor, String> {
    // In Teile zerlegen: vorn die Art, dann je ein Teil ab . # :
    let mut teile: Vec<(char, String)> = Vec::new();
    let mut akt = (' ', String::new());
    for c in s.chars() {
        if c == '.' || c == '#' || c == ':' {
            teile.push(std::mem::replace(&mut akt, (c, String::new())));
        } else {
            akt.1.push(c);
        }
    }
    teile.push(akt);
    let mut sel = Selektor { art: None, klassen: Vec::new(), name: None, an: false, schicht: BASIS, gewicht: 0,
                             vorfahren: Vec::new() };
    let mut fluechtig = 0;
    for (art, wort) in teile {
        let w = wort.to_lowercase();
        if art == ':' && !ziel {
            return Err(format!("Zeile {}: '{}' -- einen Zustand traegt nur der letzte Teil des Selektors, nicht ein Behaelter darueber", zeile, s));
        }
        match art {
            ' ' => {
                if w.is_empty() || w == "*" { continue; }
                if w == "window" || w == "fenster" {
                    if ziel {
                        return Err(format!("Zeile {}: 'window' gibt es nur als Vorfahre (window#name button) -- das Fenster selbst ist kein Widget", zeile));
                    }
                    sel.art = Some("window".into()); sel.gewicht += 1;
                    continue;
                }
                if Kind::from_str(&w).is_none() {
                    return Err(format!("Zeile {}: eine Widget-Art '{}' gibt es nicht -- die Namen sind die aus der .dhform (button, label, textinput, listbox, ...)", zeile, wort));
                }
                sel.art = Some(w); sel.gewicht += 1;
            }
            '.' => {
                if !ist_name(&wort) { return Err(format!("Zeile {}: Klasse '{}' -- erlaubt sind Buchstaben, Ziffern, _ und -", zeile, wort)); }
                sel.klassen.push(w); sel.gewicht += 10;
            }
            '#' => {
                if !ist_name(&wort) { return Err(format!("Zeile {}: Name '{}' -- erlaubt sind Buchstaben, Ziffern, _ und -", zeile, wort)); }
                if sel.name.is_some() { return Err(format!("Zeile {}: zwei Namen in einem Selektor", zeile)); }
                sel.name = Some(w); sel.gewicht += 100;
            }
            _ => {
                let s2 = match w.as_str() {
                    "hover" | "ueber" => HOVER,
                    "focus" | "fokus" => FOKUS,
                    "active" | "gedrueckt" => AKTIV,
                    "disabled" | "gesperrt" => AUS,
                    "checked" | "an" => { sel.an = true; sel.gewicht += 10; continue; }
                    _ => return Err(format!("Zeile {}: Zustand ':{}' gibt es nicht -- bekannt sind {}", zeile, wort, ZUSTAENDE)),
                };
                fluechtig += 1;
                if fluechtig > 1 {
                    return Err(format!("Zeile {}: hoechstens einer von :hover, :focus, :active, :disabled je Selektor", zeile));
                }
                sel.schicht = s2; sel.gewicht += 10;
            }
        }
    }
    Ok(sel)
}

fn farbe_lesen(wert: &str, zeile: usize, thema: &dyn Fn(&str) -> bool) -> Result<StilFarbe, String> {
    let w = wert.trim();
    if let Some(h) = w.strip_prefix('#') {
        if !h.chars().all(|c| c.is_ascii_hexdigit()) || ![3, 6, 8].contains(&h.len()) {
            return Err(format!("Zeile {}: '{}' ist keine Farbe -- erwartet #RGB, #RRGGBB, #AARRGGBB oder ein Name des Themas", zeile, w));
        }
        let voll: String = if h.len() == 3 { h.chars().flat_map(|c| [c, c]).collect() } else { h.to_string() };
        return Ok(StilFarbe::Fest(i64::from_str_radix(&voll, 16).unwrap_or(0)));
    }
    let name = match w.to_lowercase().as_str() { "akzent" => "accent".to_string(), x => x.to_string() };
    if thema(&name) { return Ok(StilFarbe::Thema(name)); }
    Err(format!("Zeile {}: '{}' ist keine Farbe -- erwartet #RGB, #RRGGBB, #AARRGGBB oder ein Name des Themas (accent, text_fg, widget_bg, ...)", zeile, w))
}

fn zahl_lesen_stil(wert: &str, zeile: usize, was: &str, von: i32, bis: i32) -> Result<i32, String> {
    match wert.trim().parse::<i32>() {
        Ok(n) if (von..=bis).contains(&n) => Ok(n),
        _ => Err(format!("Zeile {}: {} erwartet eine ganze Zahl {}..{}, erhalten '{}'", zeile, was, von, bis, wert.trim())),
    }
}

/// `schatten: weich` oder `schatten: dx dy weich [farbe]`.
fn schatten_lesen(wert: &str, zeile: usize, thema: &dyn Fn(&str) -> bool) -> Result<Schatten, String> {
    let teile: Vec<&str> = wert.split_whitespace().collect();
    let zahl = |s: &str, was: &str, von: i32, bis: i32| zahl_lesen_stil(s, zeile, was, von, bis);
    match teile.len() {
        1 => {
            let w = zahl(teile[0], "schatten", 0, 32)?;
            Ok(Schatten { dx: 0, dy: if w > 0 { (w / 3).max(1) } else { 0 }, weich: w, farbe: None })
        }
        3 | 4 => Ok(Schatten {
            dx: zahl(teile[0], "schatten (dx)", -32, 32)?,
            dy: zahl(teile[1], "schatten (dy)", -32, 32)?,
            weich: zahl(teile[2], "schatten (weich)", 0, 32)?,
            farbe: match teile.get(3) { Some(f) => Some(farbe_lesen(f, zeile, thema)?), None => None },
        }),
        _ => Err(format!("Zeile {}: schatten erwartet 'weich' oder 'dx dy weich [farbe]', erhalten '{}'", zeile, wert)),
    }
}

/// Eine Stildatei lesen. `thema` sagt, ob ein Name eine Farbe des Themas ist.
pub(super) fn lesen(text: &str, thema: &dyn Fn(&str) -> bool) -> Result<Stildatei, String> {
    let t = ohne_kommentare(text)?;
    let mut regeln = Vec::new();
    let mut pos = 0;
    while let Some(rel) = t[pos..].find('{') {
        let auf = pos + rel;
        let kopf = &t[pos..auf];
        if kopf.contains('}') { return Err(format!("Zeile {}: '}}' ohne '{{'", zeile_von(&t, pos + kopf.find('}').unwrap()))); }
        let zu = match t[auf + 1..].find('}') {
            Some(r) => auf + 1 + r,
            None => return Err(format!("Zeile {}: '{{' ohne '}}'", zeile_von(&t, auf))),
        };
        let rumpf = &t[auf + 1..zu];
        if rumpf.contains('{') { return Err(format!("Zeile {}: '{{' im Rumpf -- Regeln lassen sich nicht schachteln", zeile_von(&t, auf + 1 + rumpf.find('{').unwrap()))); }
        // Kopfzeile: die erste nicht leere Zeile vor der Klammer.
        let kopf_zeile = zeile_von(&t, pos + kopf.len() - kopf.trim_start().len());
        let selektoren: Vec<Selektor> = kopf.split(',')
            .map(|s| selektor_lesen(s, kopf_zeile)).collect::<Result<_, _>>()?;
        let mit_zustand = selektoren.iter().any(|s| s.schicht != BASIS);
        let mut werte = StilWerte::default();
        let mut off = auf + 1;
        for decl in rumpf.split(|c| c == ';' || c == '\n') {
            let zeile = zeile_von(&t, off);
            off += decl.len() + 1;
            let d = decl.trim();
            if d.is_empty() { continue; }
            let (name, wert) = d.split_once(':').ok_or_else(|| format!("Zeile {}: '{}' -- erwartet eigenschaft: wert", zeile, d))?;
            let name = name.trim().to_lowercase();
            let wert = wert.trim();
            let form = matches!(name.as_str(), "abstand" | "padding" | "innenabstand" | "randbreite" | "border-width"
                | "border_width" | "schatten" | "shadow" | "verlauf" | "gradient" | "glanz" | "gloss" | "fase" | "bevel"
                | "uebergang" | "transition");
            if form && mit_zustand {
                return Err(format!("Zeile {}: '{}' gilt nur in Regeln ohne :hover, :focus, :active, :disabled -- eine Form, die mit dem Zustand springt, waere ein Wackeln", zeile, name));
            }
            match name.as_str() {
                "bg" | "hintergrund" | "background" => werte.farben.push(("bg".into(), farbe_lesen(wert, zeile, thema)?)),
                "fg" | "farbe" | "text" | "color" => werte.farben.push(("fg".into(), farbe_lesen(wert, zeile, thema)?)),
                "border" | "rand" => werte.farben.push(("border".into(), farbe_lesen(wert, zeile, thema)?)),
                "accent" | "akzent" => werte.farben.push(("accent".into(), farbe_lesen(wert, zeile, thema)?)),
                "radius" | "rundung" => werte.radius = Some(zahl_lesen_stil(wert, zeile, "radius", 0, 64)?),
                "groesse" | "size" | "schriftgroesse" => werte.groesse = Some(zahl_lesen_stil(wert, zeile, "groesse", 4, 200)?),
                "stil" | "style" | "schriftstil" => werte.stil = Some(crate::schnitt::stil_parsen(wert)
                    .map_err(|e| format!("Zeile {}: {}", zeile, e))?),
                "abstand" | "padding" | "innenabstand" => {
                    let teile: Vec<&str> = wert.split_whitespace().collect();
                    let v = match teile.as_slice() {
                        [a] => { let n = zahl_lesen_stil(a, zeile, "abstand", 0, 40)?; (n, n) }
                        [a, b] => (zahl_lesen_stil(a, zeile, "abstand", 0, 40)?, zahl_lesen_stil(b, zeile, "abstand", 0, 40)?),
                        _ => return Err(format!("Zeile {}: abstand erwartet eine oder zwei Zahlen (senkrecht waagerecht), erhalten '{}'", zeile, wert)),
                    };
                    werte.abstand = Some(v);
                }
                "randbreite" | "border-width" | "border_width" => werte.randbreite = Some(zahl_lesen_stil(wert, zeile, "randbreite", 0, 8)?),
                "schatten" | "shadow" => werte.schatten = Some(schatten_lesen(wert, zeile, thema)?),
                "verlauf" | "gradient" => werte.verlauf = Some(zahl_lesen_stil(wert, zeile, "verlauf", 0, 64)?),
                "glanz" | "gloss" => werte.glanz = Some(zahl_lesen_stil(wert, zeile, "glanz", 0, 100)?),
                "fase" | "bevel" => werte.fase = Some(zahl_lesen_stil(wert, zeile, "fase", 0, 1)?),
                "uebergang" | "transition" => {
                    let (alle, je) = uebergang_lesen(wert, zeile)?;
                    if alle.is_some() { werte.uebergang = alle; }
                    for (k, d) in je { werte.uebergang_je.retain(|(x, _)| *x != k); werte.uebergang_je.push((k, d)); }
                }
                _ => return Err(format!("Zeile {}: Eigenschaft '{}' gibt es nicht -- bekannt sind {}", zeile, name, EIGENSCHAFTEN)),
            }
        }
        for sel in selektoren { regeln.push(Regel { sel, werte: werte.clone() }); }
        pos = zu + 1;
    }
    if !t[pos..].trim().is_empty() {
        let rest = t[pos..].trim_start();
        return Err(format!("Zeile {}: '{}' -- eine Regel braucht einen Rumpf in {{ }}", zeile_von(&t, t.len() - rest.len()), rest.lines().next().unwrap_or("").trim()));
    }
    let mit_vorfahren = regeln.iter().any(|r: &Regel| !r.sel.vorfahren.is_empty());
    Ok(Stildatei { regeln, mit_vorfahren })
}

impl Stildatei {
    /// Aufloesen fuer ein Widget dieser Art mit diesen Klassen; `kette` sind
    /// seine Behaelter und das Fenster, das naechste zuerst.
    pub(super) fn aufloesen(&self, art: &str, klassen: &[String], name: &str, an: bool, kette: &[Glied]) -> StilAufgeloest {
        let mut passend: Vec<(u32, usize, &Regel)> = self.regeln.iter().enumerate().filter(|(_, r)| {
            let s = &r.sel;
            s.art.as_ref().map(|a| a == art).unwrap_or(true)
                && s.klassen.iter().all(|k| klassen.iter().any(|x| x == k))
                && s.name.as_ref().map(|n| n == name).unwrap_or(true)
                && (!s.an || an)
                && vorfahren_passen(&s.vorfahren, kette)
        }).map(|(i, r)| (r.sel.gewicht, i, r)).collect();
        passend.sort_by_key(|(g, i, _)| (*g, *i));
        let mut o: StilAufgeloest = Default::default();
        for (_, _, r) in passend { o[r.sel.schicht].uebernehmen(&r.werte); }
        o
    }
}

impl Gui {
    // ================================================================ Befehle
    /// GUI_STYLESHEET(text$): eine Stildatei setzen ("" = keine).
    pub fn stylesheet(&mut self, text: &str) -> Result<(), String> {
        self.stylesheet_aus(text, "GUI_STYLESHEET")
    }
    pub fn stylesheet_load(&mut self, pfad: &str) -> Result<(), String> {
        let text = std::fs::read_to_string(pfad)
            .map_err(|e| format!("GUI_STYLESHEET_LOAD: '{}' laesst sich nicht lesen: {}", pfad, e))?;
        self.stylesheet_aus(&text, &format!("GUI_STYLESHEET_LOAD ({})", pfad))
    }
    fn stylesheet_aus(&mut self, text: &str, f: &str) -> Result<(), String> {
        let thema = |n: &str| self.theme.contains_key(n);
        let datei = lesen(text, &thema).map_err(|e| format!("{}: {}", f, e))?;
        self.stil = if datei.regeln.is_empty() { None } else { Some(Box::new(datei)) };
        self.stil_stand += 1;
        Ok(())
    }
    pub fn set_name(&mut self, h: i64, name: &str) -> Result<(), String> {
        if !name.is_empty() && !ist_name(name) {
            return Err(format!("GUI_SET_NAME: Name '{}' -- erlaubt sind Buchstaben, Ziffern, _ und -", name));
        }
        let w = self.wdg_mut(h, "GUI_SET_NAME")?;
        w.name = name.to_lowercase();
        w.stil_key = None;
        Ok(())
    }
    pub fn get_name(&self, h: i64) -> Result<String, String> {
        Ok(self.wdg(h, "GUI_GET_NAME$")?.name.clone())
    }
    pub fn add_class(&mut self, h: i64, k: &str) -> Result<(), String> {
        if !ist_name(k) { return Err(format!("GUI_ADD_CLASS: Klasse '{}' -- erlaubt sind Buchstaben, Ziffern, _ und -", k)); }
        let w = self.wdg_mut(h, "GUI_ADD_CLASS")?;
        let k = k.to_lowercase();
        if !w.klassen.contains(&k) { w.klassen.push(k); w.stil_key = None; }
        Ok(())
    }
    pub fn remove_class(&mut self, h: i64, k: &str) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_REMOVE_CLASS")?;
        let k = k.to_lowercase();
        let n = w.klassen.len();
        w.klassen.retain(|x| *x != k);
        if w.klassen.len() != n { w.stil_key = None; }
        Ok(())
    }
    pub fn has_class(&self, h: i64, k: &str) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_HAS_CLASS")?;
        let k = k.to_lowercase();
        Ok(w.klassen.contains(&k) || (w.kind == Kind::Button && KNOPF_ARTEN.get(w.variante as usize) == Some(&k.as_str())))
    }

    // ================================================================ je Bild
    /// Am Ende von GUI_UPDATE: jedes Widget, bei dem sich etwas geaendert
    /// hat, neu aufloesen.
    pub(super) fn stil_pass(&mut self) {
        let Some(datei) = self.stil.as_ref() else {
            for win in self.windows.iter_mut() { for w in win.widgets.iter_mut() { w.stil_auf = None; w.stil_key = None; } }
            return;
        };
        let stand = self.stil_stand;
        let mit_vorfahren = datei.mit_vorfahren;
        for win in self.windows.iter_mut() {
            // Die Ketten ueber den Widgets -- nur, wenn eine Regel danach
            // fragt. Ein Hash der Kette steht im Schluessel: aendert sich
            // eine Klasse weiter oben oder zieht ein Widget in einen anderen
            // Behaelter, wird neu aufgeloest, ohne dass jede Stelle, die das
            // tut, daran denken muss.
            let ketten: Vec<Vec<Glied>> = if mit_vorfahren { Self::ketten(win) } else { Vec::new() };
            for (i, w) in win.widgets.iter_mut().enumerate() {
                if !w.alive { continue; }
                let kette: &[Glied] = ketten.get(i).map(|k| k.as_slice()).unwrap_or(&[]);
                let khash = if kette.is_empty() { 0 } else {
                    use std::hash::{Hash, Hasher};
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    kette.hash(&mut h);
                    h.finish()
                };
                let key = (stand, w.checked, w.enabled, w.variante, khash);
                if w.stil_key == Some(key) { continue; }
                let mut kl = w.klassen.clone();
                if w.kind == Kind::Button {
                    if let Some(a) = KNOPF_ARTEN.get(w.variante as usize) { kl.push(a.to_string()); }
                }
                let auf = datei.aufloesen(w.kind.as_str(), &kl, &w.name, w.checked, kette);
                w.stil_auf = if auf.iter().all(|s| *s == StilWerte::default()) { None } else { Some(Box::new(auf)) };
                w.stil_key = Some(key);
            }
        }
    }
    /// Je Widget die Kette seiner Behaelter bis zum Fenster, das naechste
    /// zuerst. Behaelter ist, wer Kinder fuehrt: ein Layout (seine Kinder-
    /// liste), sonst Reiterwerk/Akkordeon/Assistent (`tc_von`) und das
    /// rollende Panel (`panel_von`) -- beide je Bild nachgefuehrt.
    fn ketten(win: &Window) -> Vec<Vec<Glied>> {
        let n = win.widgets.len();
        let mut eltern = vec![-1i32; n];
        for (j, w) in win.widgets.iter().enumerate() {
            if !w.alive { continue; }
            if let Some(l) = w.layout.as_ref() {
                for &(k, _) in &l.kinder { if k >= 0 && (k as usize) < n && k as usize != j { eltern[k as usize] = j as i32; } }
            }
        }
        for (i, w) in win.widgets.iter().enumerate() {
            if eltern[i] >= 0 { continue; }
            if w.tc_von >= 0 && w.tc_von as usize != i { eltern[i] = w.tc_von; }
            else if w.panel_von >= 0 && w.panel_von as usize != i { eltern[i] = w.panel_von; }
        }
        let glied = |w: &Widget| Glied { art: w.kind.as_str().to_string(), klassen: w.klassen.clone(), name: w.name.clone() };
        let fenster = Glied { art: "window".into(), klassen: Vec::new(), name: win.name.clone() };
        (0..n).map(|i| {
            let mut k = Vec::new();
            let mut p = eltern[i];
            // Hoechstens 32 Stufen -- ein Kreis waere ein Fehler anderswo,
            // darf hier aber nicht haengen.
            while p >= 0 && k.len() < 32 {
                k.push(glied(&win.widgets[p as usize]));
                p = eltern[p as usize];
            }
            k.push(fenster.clone());
            k
        }).collect()
    }
    /// GUI_WINDOW_SET_NAME(win, name$): ein Name fuer `window#name` in der
    /// Stildatei.
    pub fn window_set_name(&mut self, h: i64, name: &str) -> Result<(), String> {
        if !name.is_empty() && !ist_name(name) {
            return Err(format!("GUI_WINDOW_SET_NAME: Name '{}' -- erlaubt sind Buchstaben, Ziffern, _ und -", name));
        }
        self.win_mut(h, "GUI_WINDOW_SET_NAME")?.name = name.to_lowercase();
        self.stil_stand += 1;
        Ok(())
    }
    pub fn window_get_name(&self, h: i64) -> Result<String, String> {
        self.windows.get(h as usize).map(|w| w.name.clone())
            .ok_or_else(|| "GUI_WINDOW_GET_NAME$: ungueltiges GUI_WINDOW-Handle".to_string())
    }

    // ================================================================ Zeichnen
    fn stil_wert(&self, f: &StilFarbe) -> i64 {
        match f { StilFarbe::Fest(c) => *c, StilFarbe::Thema(n) => self.th(n) }
    }
    /// Die Farbe einer Rolle aus der Schicht `s` der Stildatei.
    pub(super) fn stil_farbe(&self, w: &Widget, rolle: &str, s: usize) -> Option<i64> {
        w.stil_auf.as_ref()?.get(s)?.farbe(rolle).map(|f| self.stil_wert(f))
    }
    /// Die eigene Farbe einer Rolle OHNE Zustaende: GUI_SET_COLOR, sonst die
    /// Grundschicht der Stildatei.
    pub(super) fn eigen(&self, w: &Widget, rolle: &str) -> Option<i64> {
        w.ov.get(rolle).copied().or_else(|| self.stil_farbe(w, rolle, BASIS))
    }
    /// Eine Farbe samt Zustaenden: `basis` ist, was ohne die Stildatei gaelte;
    /// Hover, Fokus und Druck blenden ueber die Uebergaenge des Widgets ein,
    /// ein gesperrtes Widget nimmt seine eigene Farbe.
    pub(super) fn stil_misch(&self, w: &Widget, rolle: &str, basis: i64) -> i64 {
        if w.ov.contains_key(rolle) || w.stil_auf.is_none() { return basis; }
        if !w.enabled {
            if let Some(c) = self.stil_farbe(w, rolle, AUS) { return c; }
            return basis;
        }
        let mut c = basis;
        if let Some(h) = self.stil_farbe(w, rolle, HOVER) { c = mischen(c, h, Self::rolle_t(w, rolle, 0)); }
        if let Some(f) = self.stil_farbe(w, rolle, FOKUS) { c = mischen(c, f, Self::rolle_t(w, rolle, 1)); }
        if let Some(a) = self.stil_farbe(w, rolle, AKTIV) { c = mischen(c, a, Self::rolle_t(w, rolle, 2)); }
        c
    }
    /// GUI_GET_COLOR(wdg, rolle$): die Farbe, die GUI_SET_COLOR oder die
    /// Stildatei im JETZIGEN Zustand vorgeben (ohne Ueberblenden), -1 = das
    /// Thema entscheidet.
    pub fn get_color(&self, h: i64, rolle: &str) -> Result<i64, String> {
        let rolle = rolle.to_lowercase();
        if !matches!(rolle.as_str(), "bg" | "fg" | "border" | "accent") {
            return Err(format!("GUI_GET_COLOR: Rolle '{}' gibt es nicht -- bg, fg, border oder accent", rolle));
        }
        let (wi, i) = Self::dec_widget(h);
        let w = self.wdg(h, "GUI_GET_COLOR")?;
        if let Some(c) = w.ov.get(&rolle) { return Ok(*c); }
        let mut folge = Vec::new();
        if !w.enabled { folge.push(AUS); }
        else {
            if self.press_origin == Some((wi, i)) { folge.push(AKTIV); }
            if self.focus_widget == Some((wi, i)) { folge.push(FOKUS); }
            if w.hovered { folge.push(HOVER); }
        }
        folge.push(BASIS);
        Ok(folge.into_iter().find_map(|s| self.stil_farbe(w, &rolle, s)).unwrap_or(-1))
    }
    /// Setzt die Stildatei fuer diese Rolle in dieser Schicht etwas?
    pub(super) fn stil_hat(&self, w: &Widget, rolle: &str, s: usize) -> bool {
        self.stil_farbe(w, rolle, s).is_some()
    }
    /// Schriftgroesse: die eigene (GUI_SET_FONT_SIZE), sonst die der
    /// Stildatei, 0 = keine.
    pub(super) fn w_groesse(&self, w: &Widget) -> i32 {
        if w.font_size > 0 { return w.font_size; }
        w.stil_auf.as_ref().and_then(|s| s[BASIS].groesse).unwrap_or(0)
    }
    /// Schriftstil: der eigene (GUI_SET_FONT_STYLE), sonst der der Stildatei.
    pub(super) fn w_stil(&self, w: &Widget) -> u8 {
        if w.stil != 0 { return w.stil; }
        w.stil_auf.as_ref().and_then(|s| s[BASIS].stil).unwrap_or(0)
    }
    /// Verlauf, Glanz, Fase und Randbreite aus der Stildatei (nichts gesetzt
    /// = die Metriken).
    pub(super) fn w_form(&self, w: &Widget) -> FormStil {
        match w.stil_auf.as_ref() {
            Some(s) => FormStil { verlauf: s[BASIS].verlauf, glanz: s[BASIS].glanz, fase: s[BASIS].fase,
                                  rand: s[BASIS].randbreite.map(|r| if r > 0 { self.sk(r).max(1) } else { 0 }) },
            None => FormStil::default(),
        }
    }
    /// Innenabstand (senkrecht, waagerecht) in Bildpunkten, None = wie bisher.
    pub(super) fn w_abstand(&self, w: &Widget) -> Option<(i32, i32)> {
        w.stil_auf.as_ref().and_then(|s| s[BASIS].abstand).map(|(v, h)| (self.sk(v), self.sk(h)))
    }
    /// Schlagschatten (dx, dy, weich, farbe) in Bildpunkten.
    pub(super) fn w_schatten(&self, w: &Widget) -> Option<(i32, i32, i32, i64)> {
        let s = w.stil_auf.as_ref()?[BASIS].schatten.as_ref()?;
        if s.weich == 0 && s.dx == 0 && s.dy == 0 { return None; }
        let farbe = s.farbe.as_ref().map(|f| self.stil_wert(f)).unwrap_or(0x50000000);
        Some((self.sk(s.dx), self.sk(s.dy), self.sk(s.weich), farbe))
    }
    /// Dauer der Uebergaenge dieses Widgets in ms, None = die Metrik.
    pub(super) fn w_uebergang(w: &Widget) -> Option<i32> {
        w.stil_auf.as_ref().and_then(|s| s[BASIS].uebergang)
    }
    /// Eigene Dauern je Farbrolle, None = keine (dann gilt `w_uebergang`
    /// fuer alle Farben, und `rollen_t` wird nicht gefuehrt).
    pub(super) fn w_uebergang_je(w: &Widget) -> Option<[Option<i32>; 4]> {
        let je = &w.stil_auf.as_ref()?[BASIS].uebergang_je;
        if je.is_empty() { return None; }
        let mut d = [None; 4];
        for &(k, v) in je { d[k] = Some(v); }
        Some(d)
    }
    /// Wie weit eine Farbrolle eines Zustands eingeblendet ist (0..1, schon
    /// weich): `zustand` 0 Ueberfahren, 1 Fokus, 2 Druck. Mit eigenen
    /// Dauern je Rolle kommt der Wert aus `rollen_t`, sonst aus den drei
    /// Uebergaengen des Widgets.
    pub(super) fn rolle_t(w: &Widget, rolle: &str, zustand: usize) -> f64 {
        let k = ROLLEN.iter().position(|r| *r == rolle);
        if let (Some(k), Some(rt)) = (k, w.rollen_t.as_ref()) { return weich(rt[k][zustand]); }
        weich(match zustand { 0 => w.ueber_t, 1 => w.fokus_t, _ => w.druck_t })
    }
    /// Der Innenabstand waagerecht in Bildpunkten: aus der Stildatei, sonst
    /// die Metrik `pad` -- eine Stelle fuer Zeichnen und Treffer.
    pub(super) fn w_pad(&self, w: &Widget) -> i32 {
        self.w_abstand(w).map(|a| a.1).unwrap_or_else(|| self.m("pad"))
    }
    /// Der Schlagschatten unter einem Widget: gestaffelte, nach aussen
    /// blasser werdende Lagen -- raylib hat keinen Weichzeichner fuer Formen.
    pub(super) fn schatten_zeichnen(&self, g: &mut Graphics, w: &Widget, x: i32, y: i32, b: i32, h: i32) {
        let Some((dx, dy, weich, farbe)) = self.w_schatten(w) else { return };
        let rad = self.w_radius(w).unwrap_or_else(|| self.m("corner_radius"));
        // Deckkraft 0 heisst in dhrt DECKEND.
        let a = match (farbe >> 24) & 0xFF { 0 => 255, a => a };
        let rgb = farbe & 0xFF_FFFF;
        let n = weich.clamp(1, 8);
        for k in (0..n).rev() {
            let e = if n == 1 { 0 } else { weich * k / (n - 1) };
            let la = ((a as f64) * 1.6 / (n as f64 + 0.6)).round().clamp(1.0, 255.0) as i64;
            g.round_rect(x + dx - e, y + dy - e, x + b - 1 + dx + e, y + h - 1 + dy + e, (rad + e).max(0),
                         (la << 24) | rgb, true);
        }
    }
    pub(super) fn w_radius(&self, w: &Widget) -> Option<i32> {
        w.stil_auf.as_ref().and_then(|s| s[BASIS].radius).map(|r| self.sk(r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn th(n: &str) -> bool { matches!(n, "accent" | "text_fg" | "widget_bg") }

    #[test]
    fn regeln_mit_vorrang_und_schichten() {
        let d = lesen("button { bg: #102030; radius: 6 }\n\
                       button.gefahr { bg: #C0392B }\n\
                       button.gefahr:hover { bg: #E74C3C }\n\
                       #ok { fg: akzent; stil: fett }\n\
                       * { border: #333 }", &th).unwrap();
        let a = d.aufloesen("button", &["gefahr".into()], "ok", false, &[]);
        assert_eq!(a[BASIS].farbe("bg"), Some(&StilFarbe::Fest(0xC0392B)));
        assert_eq!(a[BASIS].farbe("fg"), Some(&StilFarbe::Thema("accent".into())));
        assert_eq!(a[BASIS].farbe("border"), Some(&StilFarbe::Fest(0x333333)));
        assert_eq!(a[BASIS].radius, Some(6));
        assert_eq!(a[BASIS].stil, Some(crate::schnitt::FETT));
        assert_eq!(a[HOVER].farbe("bg"), Some(&StilFarbe::Fest(0xE74C3C)));
        // Ohne die Klasse greift nur die Grundregel.
        let b = d.aufloesen("button", &[], "", false, &[]);
        assert_eq!(b[BASIS].farbe("bg"), Some(&StilFarbe::Fest(0x102030)));
        assert_eq!(b[HOVER], StilWerte::default());
        // Eine andere Art bekommt nur den Stern.
        let c = d.aufloesen("label", &[], "", false, &[]);
        assert_eq!(c[BASIS].farbe("bg"), None);
        assert_eq!(c[BASIS].farbe("border"), Some(&StilFarbe::Fest(0x333333)));
    }

    #[test]
    fn gleiches_gewicht_spaetere_gewinnt_und_checked() {
        let d = lesen("checkbox { fg: #111111 } checkbox { fg: #222222 } checkbox:checked { fg: #333333 }", &th).unwrap();
        assert_eq!(d.aufloesen("checkbox", &[], "", false, &[])[BASIS].farbe("fg"), Some(&StilFarbe::Fest(0x222222)));
        assert_eq!(d.aufloesen("checkbox", &[], "", true, &[])[BASIS].farbe("fg"), Some(&StilFarbe::Fest(0x333333)));
    }

    #[test]
    fn kommentare_und_mehrere_selektoren() {
        let d = lesen("/* oben\n */ label, button { // dahinter\n fg: #ABC }\n", &th).unwrap();
        assert_eq!(d.aufloesen("button", &[], "", false, &[])[BASIS].farbe("fg"), Some(&StilFarbe::Fest(0xAABBCC)));
        assert_eq!(d.aufloesen("label", &[], "", false, &[])[BASIS].farbe("fg"), Some(&StilFarbe::Fest(0xAABBCC)));
    }

    #[test]
    fn fehler_nennen_die_zeile() {
        let f = |t: &str| lesen(t, &th).unwrap_err();
        assert!(f("button {\n  farb: #fff\n}").starts_with("Zeile 2: Eigenschaft 'farb' gibt es nicht"));
        assert!(f("\nknopf { fg: #fff }").starts_with("Zeile 2: eine Widget-Art 'knopf'"));
        assert!(f("button:wackeln { fg: #fff }").contains("Zustand ':wackeln'"));
        assert!(f("button { fg: #12 }").contains("ist keine Farbe"));
        assert!(f("button { fg: rot }").contains("Name des Themas"));
        assert!(f("button { radius: 99 }").contains("0..64"));
        assert!(f("panel:hover button { fg: #fff }").contains("nur der letzte Teil"));
        assert!(f("> button { fg: #fff }").contains("vor '>' fehlt"));
        assert!(f("panel > { fg: #fff }").contains("hinter '>' fehlt"));
        assert!(f("window { fg: #fff }").contains("nur als Vorfahre"));
        assert!(f("button { uebergang: knopf 30 }").contains("eigene Dauer haben bg"));
        assert!(f("button:hover:focus { fg: #fff }").contains("hoechstens einer"));
        assert!(f("button { fg: #fff").contains("'{' ohne '}'"));
        assert!(f("button fg: #fff").contains("braucht einen Rumpf"));
        assert!(f("/* offen").contains("Kommentar ohne */"));
    }

    fn glied(art: &str, kl: &[&str], name: &str) -> Glied {
        Glied { art: art.into(), klassen: kl.iter().map(|s| s.to_string()).collect(), name: name.into() }
    }

    #[test]
    fn nachfahren_irgendwo_und_direkt() {
        let d = lesen("panel.leiste button { bg: #111111 }
                       layout > button { fg: #222222 }
                       window#einst button:hover { bg: #333333 }", &th).unwrap();
        assert!(d.mit_vorfahren);
        // Knopf in einem Layout, das im Panel .leiste liegt, im Fenster "einst".
        let k = [glied("layout", &[], ""), glied("panel", &["leiste"], ""), glied("window", &[], "einst")];
        let a = d.aufloesen("button", &[], "", false, &k);
        assert_eq!(a[BASIS].farbe("bg"), Some(&StilFarbe::Fest(0x111111)));   // irgendwo darueber
        assert_eq!(a[BASIS].farbe("fg"), Some(&StilFarbe::Fest(0x222222)));   // direkt darueber
        assert_eq!(a[HOVER].farbe("bg"), Some(&StilFarbe::Fest(0x333333)));
        // Direkt im Panel: kein Layout direkt darueber.
        let k2 = [glied("panel", &["leiste"], ""), glied("window", &[], "einst")];
        let b = d.aufloesen("button", &[], "", false, &k2);
        assert_eq!(b[BASIS].farbe("fg"), None);
        assert_eq!(b[BASIS].farbe("bg"), Some(&StilFarbe::Fest(0x111111)));
        // Ein Layout zwischen Knopf und Layout: '>' traegt nicht weiter.
        let k3 = [glied("panel", &[], ""), glied("layout", &[], ""), glied("window", &[], "")];
        let c = d.aufloesen("button", &[], "", false, &k3);
        assert_eq!(c[BASIS].farbe("fg"), None);
        assert_eq!(c[BASIS].farbe("bg"), None);   // das Panel traegt die Klasse nicht
        // Ein anderes Fenster: die Hover-Regel greift nicht.
        assert_eq!(d.aufloesen("button", &[], "", false, &[glied("window", &[], "anders")])[HOVER], StilWerte::default());
        // Vorrang: Vorfahren zaehlen mit (panel.leiste button = 1 + 10 + 1).
        let e = lesen("button.x { bg: #AAAAAA }
 panel.leiste button { bg: #BBBBBB }", &th).unwrap();
        assert_eq!(e.aufloesen("button", &["x".into()], "", false, &k2)[BASIS].farbe("bg"), Some(&StilFarbe::Fest(0xBBBBBB)));
    }

    #[test]
    fn uebergang_je_rolle() {
        let d = lesen("button { uebergang: 120, bg 400ms, border 0 }", &th).unwrap();
        let a = d.aufloesen("button", &[], "", false, &[]);
        assert_eq!(a[BASIS].uebergang, Some(120));
        let mut je = a[BASIS].uebergang_je.clone();
        je.sort();
        assert_eq!(je, vec![(0, 400), (2, 0)]);
        // Eine spaetere Regel ersetzt nur die Rolle, die sie nennt.
        let d2 = lesen("button { uebergang: bg 400, fg 50 } #k { uebergang: bg 10 }", &th).unwrap();
        let mut je2 = d2.aufloesen("button", &[], "k", false, &[])[BASIS].uebergang_je.clone();
        je2.sort();
        assert_eq!(je2, vec![(0, 10), (1, 50)]);
    }
}
