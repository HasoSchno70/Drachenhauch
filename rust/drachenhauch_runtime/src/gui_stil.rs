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
use super::*;

/// Schichten der Aufloesung.
pub(super) const BASIS: usize = 0;
pub(super) const HOVER: usize = 1;
pub(super) const FOKUS: usize = 2;
pub(super) const AKTIV: usize = 3;
pub(super) const AUS: usize = 4;

const EIGENSCHAFTEN: &str = "bg, fg, border, accent, radius, groesse, stil";
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
}

#[derive(Clone, Debug)]
struct Regel { sel: Selektor, werte: StilWerte }

/// Eine gelesene Stildatei.
#[derive(Clone, Debug, Default)]
pub(super) struct Stildatei { regeln: Vec<Regel> }

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

fn selektor_lesen(s: &str, zeile: usize) -> Result<Selektor, String> {
    let s = s.trim();
    if s.is_empty() { return Err(format!("Zeile {}: leerer Selektor", zeile)); }
    if s.contains(char::is_whitespace) || s.contains('>') {
        return Err(format!("Zeile {}: '{}' -- Selektoren fuer Nachfahren gibt es (noch) nicht; erlaubt sind Art, .klasse, #name und Zustand", zeile, s));
    }
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
    let mut sel = Selektor { art: None, klassen: Vec::new(), name: None, an: false, schicht: BASIS, gewicht: 0 };
    let mut fluechtig = 0;
    for (art, wort) in teile {
        let w = wort.to_lowercase();
        match art {
            ' ' => {
                if w.is_empty() || w == "*" { continue; }
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
            match name.as_str() {
                "bg" | "hintergrund" | "background" => werte.farben.push(("bg".into(), farbe_lesen(wert, zeile, thema)?)),
                "fg" | "farbe" | "text" | "color" => werte.farben.push(("fg".into(), farbe_lesen(wert, zeile, thema)?)),
                "border" | "rand" => werte.farben.push(("border".into(), farbe_lesen(wert, zeile, thema)?)),
                "accent" | "akzent" => werte.farben.push(("accent".into(), farbe_lesen(wert, zeile, thema)?)),
                "radius" | "rundung" => werte.radius = Some(zahl_lesen_stil(wert, zeile, "radius", 0, 64)?),
                "groesse" | "size" | "schriftgroesse" => werte.groesse = Some(zahl_lesen_stil(wert, zeile, "groesse", 4, 200)?),
                "stil" | "style" | "schriftstil" => werte.stil = Some(crate::schnitt::stil_parsen(wert)
                    .map_err(|e| format!("Zeile {}: {}", zeile, e))?),
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
    Ok(Stildatei { regeln })
}

impl Stildatei {
    /// Aufloesen fuer ein Widget dieser Art mit diesen Klassen.
    pub(super) fn aufloesen(&self, art: &str, klassen: &[String], name: &str, an: bool) -> StilAufgeloest {
        let mut passend: Vec<(u32, usize, &Regel)> = self.regeln.iter().enumerate().filter(|(_, r)| {
            let s = &r.sel;
            s.art.as_ref().map(|a| a == art).unwrap_or(true)
                && s.klassen.iter().all(|k| klassen.iter().any(|x| x == k))
                && s.name.as_ref().map(|n| n == name).unwrap_or(true)
                && (!s.an || an)
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
        for win in self.windows.iter_mut() {
            for w in win.widgets.iter_mut() {
                if !w.alive { continue; }
                let key = (stand, w.checked, w.enabled, w.variante);
                if w.stil_key == Some(key) { continue; }
                let mut kl = w.klassen.clone();
                if w.kind == Kind::Button {
                    if let Some(a) = KNOPF_ARTEN.get(w.variante as usize) { kl.push(a.to_string()); }
                }
                let auf = datei.aufloesen(w.kind.as_str(), &kl, &w.name, w.checked);
                w.stil_auf = if auf.iter().all(|s| *s == StilWerte::default()) { None } else { Some(Box::new(auf)) };
                w.stil_key = Some(key);
            }
        }
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
        if let Some(h) = self.stil_farbe(w, rolle, HOVER) { c = mischen(c, h, weich(w.ueber_t)); }
        if let Some(f) = self.stil_farbe(w, rolle, FOKUS) { c = mischen(c, f, weich(w.fokus_t)); }
        if let Some(a) = self.stil_farbe(w, rolle, AKTIV) { c = mischen(c, a, weich(w.druck_t)); }
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
    /// Eckenrundung aus der Stildatei, None = die Metrik.
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
        let a = d.aufloesen("button", &["gefahr".into()], "ok", false);
        assert_eq!(a[BASIS].farbe("bg"), Some(&StilFarbe::Fest(0xC0392B)));
        assert_eq!(a[BASIS].farbe("fg"), Some(&StilFarbe::Thema("accent".into())));
        assert_eq!(a[BASIS].farbe("border"), Some(&StilFarbe::Fest(0x333333)));
        assert_eq!(a[BASIS].radius, Some(6));
        assert_eq!(a[BASIS].stil, Some(crate::schnitt::FETT));
        assert_eq!(a[HOVER].farbe("bg"), Some(&StilFarbe::Fest(0xE74C3C)));
        // Ohne die Klasse greift nur die Grundregel.
        let b = d.aufloesen("button", &[], "", false);
        assert_eq!(b[BASIS].farbe("bg"), Some(&StilFarbe::Fest(0x102030)));
        assert_eq!(b[HOVER], StilWerte::default());
        // Eine andere Art bekommt nur den Stern.
        let c = d.aufloesen("label", &[], "", false);
        assert_eq!(c[BASIS].farbe("bg"), None);
        assert_eq!(c[BASIS].farbe("border"), Some(&StilFarbe::Fest(0x333333)));
    }

    #[test]
    fn gleiches_gewicht_spaetere_gewinnt_und_checked() {
        let d = lesen("checkbox { fg: #111111 } checkbox { fg: #222222 } checkbox:checked { fg: #333333 }", &th).unwrap();
        assert_eq!(d.aufloesen("checkbox", &[], "", false)[BASIS].farbe("fg"), Some(&StilFarbe::Fest(0x222222)));
        assert_eq!(d.aufloesen("checkbox", &[], "", true)[BASIS].farbe("fg"), Some(&StilFarbe::Fest(0x333333)));
    }

    #[test]
    fn kommentare_und_mehrere_selektoren() {
        let d = lesen("/* oben\n */ label, button { // dahinter\n fg: #ABC }\n", &th).unwrap();
        assert_eq!(d.aufloesen("button", &[], "", false)[BASIS].farbe("fg"), Some(&StilFarbe::Fest(0xAABBCC)));
        assert_eq!(d.aufloesen("label", &[], "", false)[BASIS].farbe("fg"), Some(&StilFarbe::Fest(0xAABBCC)));
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
        assert!(f("panel button { fg: #fff }").contains("Nachfahren"));
        assert!(f("button:hover:focus { fg: #fff }").contains("hoechstens einer"));
        assert!(f("button { fg: #fff").contains("'{' ohne '}'"));
        assert!(f("button fg: #fff").contains("braucht einen Rumpf"));
        assert!(f("/* offen").contains("Kommentar ohne */"));
    }
}
