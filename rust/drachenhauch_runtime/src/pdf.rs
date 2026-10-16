//! Modul `pdf` -- druckfertige Seiten schreiben (Punkt 7 des
//! Allzweck-Audits).
//!
//! Rechnung, Lieferschein, Bericht, Etikett: fuer kaufmaennische Software ist
//! das fast immer die erste Forderung nach „speichern". Bis hierher endete
//! der Weg beim Bildschirm oder bei einer Textdatei.
//!
//! **Das PDF schreibt krilla** (seit 2026-09-14). Bis dahin stand hier ein
//! eigener Schreiber, der nur die vierzehn Standardschriften kannte -- also
//! nichts einbetten musste, dafuer aber nur cp1252 konnte und ohne
//! Schriftmasse nicht messen. Mit krilla werden die Schriften EINGEBETTET und
//! auf die benutzten Zeichen beschnitten: Griechisch, Kyrillisch, Pfeile und
//! Haken gehen, und eine eigene TrueType-Datei laesst sich laden
//! (`PDF_FONT_LOAD`).
//!
//! **Die eingebauten Schriften sind DejaVu** (Sans, Serif, Sans Mono in je
//! vier Schnitten, aus dem Crate `dejavu`). Die alten Namen `helvetica`,
//! `times` und `courier` meinen seither diese drei -- ein Programm von vorher
//! laeuft weiter, sieht aber anders aus, und seine Texte sind etwas breiter.
//! `symbol` und `zapfdingbats` gibt es nicht mehr; ihre Zeichen hat DejaVu
//! Sans selbst.
//!
//! **Millimeter, von oben gezaehlt.** Wer eine Rechnung setzt, denkt in
//! „25 mm vom oberen Rand". krilla zaehlt wie ein Bildschirm ebenfalls von
//! oben; nur die SCHRIFTGROESSE bleibt in Punkten, weil sie jeder so kennt.
//!
//! **Aufgezeichnet, nicht sofort geschrieben.** Jede Seite haelt ihre
//! Befehle (`Op`) und geht damit auf drei Ziele: die Datei (`bauen`, hier),
//! den Drucker (`drucken.rs`) und ein Bild (`graphics.rs`).

use std::sync::Arc;

/// 1 mm in PDF-Punkten (1/72 Zoll).
const MM: f64 = 72.0 / 25.4;

/// Ein aufgezeichneter Zeichenbefehl. Masse in Millimetern ab Papierkante von
/// oben, Farbe als Anteile 0..1. `y` bei Text meint die OBERKANTE der Zeile,
/// wie im Programm angegeben.
#[derive(Clone, Debug)]
/// Jeder Befehl traegt seine Deckkraft (`deck`, 0..1, PDF_OPACITY) selbst --
/// ein Zustand im Strom muesste beim Seitenwechsel neu gesetzt werden, und
/// die Vorschau und der Drucker muessten ihn mitfuehren.
pub enum Op {
    /// `winkel` in Grad gegen den Uhrzeigersinn, gedreht um (x, y) -- die
    /// linke obere Ecke des Textes, wie ohne Winkel.
    Text { x: f64, y: f64, text: String, schrift: usize, groesse_pt: f64, farbe: (f64, f64, f64), winkel: f64, deck: f64 },
    Linie { x1: f64, y1: f64, x2: f64, y2: f64, breite_mm: f64, farbe: (f64, f64, f64), deck: f64 },
    Rechteck { x: f64, y: f64, b: f64, h: f64, fuellen: bool, breite_mm: f64, farbe: (f64, f64, f64), deck: f64 },
    /// PDF_CIRCLE / PDF_CIRCLE_FILL: Mittelpunkt und Radius in mm.
    Kreis { x: f64, y: f64, r: f64, fuellen: bool, breite_mm: f64, farbe: (f64, f64, f64), deck: f64 },
    /// PDF_POLYGON / PDF_POLYGON_FILL: geschlossener Zug, mindestens drei
    /// Punkte; gefuellt nach der Nicht-Null-Regel (wie PostScript `fill`).
    Vieleck { punkte: Vec<(f64, f64)>, fuellen: bool, breite_mm: f64, farbe: (f64, f64, f64), deck: f64 },
    /// PDF_IMAGE: ein Bild, RGBA Zeile fuer Zeile, `pw` x `ph` Punkte, in das
    /// Rechteck x/y/b/h (mm) gestreckt.
    Bild { x: f64, y: f64, b: f64, h: f64, rgba: Arc<Vec<u8>>, pw: u32, ph: u32, deck: f64 },
}

/// Eine Farbe so weit gegen Weiss gemischt, wie die Deckkraft sagt -- fuer
/// Ziele, die keine Durchsicht kennen (GDI, die Vorschau). Auf weissem Papier
/// ist das dasselbe Bild, ueber anderem Inhalt eine Naeherung.
pub fn gegen_weiss(f: (f64, f64, f64), deck: f64) -> (f64, f64, f64) {
    let d = deck.clamp(0.0, 1.0);
    (f.0 * d + (1.0 - d), f.1 * d + (1.0 - d), f.2 * d + (1.0 - d))
}

pub struct Seite {
    pub breite_mm: f64,
    pub hoehe_mm: f64,
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
    /// PDF_OPACITY: gilt fuer alles, was danach gezeichnet wird.
    deck: f64,
    pub titel: String,
    /// Mit PDF_FONT_LOAD geladene Schriften: (Name, Dateiinhalt). Ihre
    /// Nummern beginnen hinter den eingebauten.
    geladen: Vec<(String, Arc<Vec<u8>>)>,
}

// Die eingebauten Schriften (DejaVu) stehen in schriften.rs -- LOADFONT
// benutzt sie auch ohne das pdf-Modul (Web-Bau).
use crate::schriften::{normalisieren, EINGEBAUT};

/// Seitengroesse aus einem Namen -- in Millimetern.
pub fn seitenmass(name: &str, quer: bool) -> Option<(f64, f64)> {
    let (b, h) = match name.to_lowercase().as_str() {
        "a0" => (841.0, 1189.0),
        "a1" => (594.0, 841.0),
        "a2" => (420.0, 594.0),
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

/// Breite eines Textes in Punkten, geformt wie beim Zeichnen.
///
/// krillas `draw_text` formt mit rustybuzz OHNE besondere Merkmale -- also
/// mit den Vorgaben der Schrift, Unterschneidung eingeschlossen. Genau so wird
/// hier gemessen; eine Summe der Zeichenbreiten laege bei "Te" oder "AV"
/// daneben, und ein rechtsbuendiger Betrag stuende nicht buendig.
fn breite_pt(daten: &[u8], groesse_pt: f64, text: &str) -> Result<f64, String> {
    let face = rustybuzz::Face::from_slice(daten, 0).ok_or("die Schrift laesst sich nicht lesen")?;
    let mut puffer = rustybuzz::UnicodeBuffer::new();
    puffer.push_str(text);
    puffer.guess_segment_properties();
    let geformt = rustybuzz::shape(&face, &[], puffer);
    let summe: i64 = geformt.glyph_positions().iter().map(|p| p.x_advance as i64).sum();
    Ok(summe as f64 / face.units_per_em() as f64 * groesse_pt)
}

/// Hat die Schrift jedes Zeichen des Textes? Ein fehlendes Zeichen ist ein
/// FEHLER und kein leeres Kaestchen -- auf einer Rechnung ist ein stumm
/// verschwundenes Zeichen schlimmer als eine Meldung.
fn zeichen_pruefen(daten: &[u8], name: &str, text: &str) -> Result<(), String> {
    let face = rustybuzz::Face::from_slice(daten, 0).ok_or("die Schrift laesst sich nicht lesen")?;
    for c in text.chars() {
        if c.is_control() { continue; }
        if face.glyph_index(c).is_none() {
            return Err(format!("die Schrift '{}' hat kein Zeichen '{}' (U+{:04X})", name, c, c as u32));
        }
    }
    Ok(())
}

impl Dokument {
    pub fn neu(breite_mm: f64, hoehe_mm: f64) -> Dokument {
        let mut d = Dokument {
            seiten: Vec::new(), breite_mm, hoehe_mm,
            schrift: 0, groesse_pt: 11.0, farbe: (0.0, 0.0, 0.0), strich_mm: 0.2, deck: 1.0,
            titel: String::new(), geladen: Vec::new(),
        };
        d.neue_seite();
        d
    }

    /// Eine neue Seite -- die vorige bleibt, wie sie ist. Schrift, Groesse,
    /// Farbe und Strichstaerke gelten weiter: sie sind Einstellungen des
    /// Dokuments, nicht der Seite.
    pub fn neue_seite(&mut self) {
        self.seiten.push(Seite { breite_mm: self.breite_mm, hoehe_mm: self.hoehe_mm, ops: Vec::new() });
    }

    /// PDF_PAGE: eine neue Seite -- ausser die erste ist noch leer. Sie legt
    /// `PDF_NEW` schon an, und wer wie bei anderen Bibliotheken erst
    /// PDF_PAGE ruft, bekam sonst eine leere erste Seite (Dogfooding
    /// 2026-10-09).
    pub fn seite_beginnen(&mut self) {
        if self.seiten.len() == 1 && self.seiten[0].ops.is_empty() { return; }
        self.neue_seite();
    }

    fn hier(&mut self) -> &mut Seite {
        // `neu` legt immer eine Seite an, es gibt also nie keine.
        self.seiten.last_mut().unwrap()
    }

    pub fn schrift_index(&self) -> usize { self.schrift }

    /// Alle Namen, unter denen PDF_FONT eine Schrift findet.
    fn schriftliste(&self) -> String {
        let mut n: Vec<String> = EINGEBAUT.iter().map(|(s, _)| s.to_string()).collect();
        n.extend(self.geladen.iter().map(|(s, _)| s.clone()));
        n.join(", ")
    }

    pub fn schrift_suchen(&self, name: &str) -> Result<usize, String> {
        let k = normalisieren(name);
        if let Some(i) = EINGEBAUT.iter().position(|(s, _)| *s == k) { return Ok(i); }
        if let Some(i) = self.geladen.iter().position(|(s, _)| *s == k) { return Ok(EINGEBAUT.len() + i); }
        if k == "symbol" || k == "zapfdingbats" {
            return Err(format!(
                "die Schrift '{}' gibt es nicht mehr -- seit die Schriften eingebettet werden, \
                 hat 'sans' griechische Buchstaben und Zeichen wie \u{2713} \u{2717} \u{2605} selbst", name));
        }
        Err(format!("Schrift '{}' gibt es nicht. Eingebaut sind {} (helvetica, times und courier \
                     meinen sans, serif und mono); eine eigene laedt PDF_FONT_LOAD", name, self.schriftliste()))
    }

    /// Eine eigene TrueType-/OpenType-Schrift unter einem Namen.
    pub fn schrift_laden(&mut self, name: &str, daten: Vec<u8>) -> Result<(), String> {
        let k = name.trim().to_lowercase();
        if k.is_empty() { return Err("der Name der Schrift darf nicht leer sein".into()); }
        let belegt = EINGEBAUT.iter().any(|(s, _)| *s == normalisieren(&k))
            || self.geladen.iter().any(|(s, _)| *s == k);
        if belegt { return Err(format!("eine Schrift '{}' gibt es schon", name)); }
        let daten = Arc::new(daten);
        let lesbar = rustybuzz::Face::from_slice(&daten, 0).is_some()
            && krilla::text::Font::new(krilla::Data::from(daten.clone()), 0).is_some();
        if !lesbar { return Err(format!("'{}' ist keine lesbare TrueType- oder OpenType-Schrift", name)); }
        self.geladen.push((k, daten));
        Ok(())
    }

    /// Der Name einer Schrift, wie das Programm sie nennt -- fuer die
    /// Abbildung auf GDI-Schriften beim Drucken.
    pub fn schrift_name(&self, index: usize) -> String {
        match EINGEBAUT.get(index) {
            Some((s, _)) => s.to_string(),
            None => self.geladen.get(index - EINGEBAUT.len()).map(|(s, _)| s.clone()).unwrap_or_default(),
        }
    }

    fn schrift_daten(&self, index: usize) -> &[u8] {
        match EINGEBAUT.get(index) {
            Some((_, f)) => f(),
            None => self.geladen[index - EINGEBAUT.len()].1.as_slice(),
        }
    }

    fn schrift_data(&self, index: usize) -> krilla::Data {
        match EINGEBAUT.get(index) {
            Some((_, f)) => f().into(),
            None => krilla::Data::from(self.geladen[index - EINGEBAUT.len()].1.clone()),
        }
    }

    pub fn setze_schrift(&mut self, index: usize, groesse_pt: f64) {
        self.schrift = index;
        self.groesse_pt = groesse_pt;
    }

    pub fn setze_farbe(&mut self, r: f64, g: f64, b: f64) { self.farbe = (r, g, b); }

    pub fn setze_strich(&mut self, mm: f64) { self.strich_mm = mm; }

    /// Deckkraft 0..1 fuer alles Folgende; ausserhalb ist ein Fehler, weil
    /// ein 35 statt 0.35 sonst still deckend waere.
    pub fn setze_deckkraft(&mut self, d: f64) -> Result<(), String> {
        if !(0.0..=1.0).contains(&d) {
            return Err(format!("die Deckkraft geht von 0 bis 1, nicht {}", d));
        }
        self.deck = d;
        Ok(())
    }

    pub fn deckkraft(&self) -> f64 { self.deck }

    pub fn text(&mut self, x: f64, y: f64, text: &str, winkel: f64) -> Result<(), String> {
        let (schrift, groesse_pt, farbe, deck) = (self.schrift, self.groesse_pt, self.farbe, self.deck);
        zeichen_pruefen(self.schrift_daten(schrift), &self.schrift_name(schrift), text)?;
        self.hier().ops.push(Op::Text { x, y, text: text.to_string(), schrift, groesse_pt, farbe, winkel, deck });
        Ok(())
    }

    /// Breite eines Textes in Millimetern, in der aktuellen Schrift und Groesse.
    pub fn textbreite(&self, text: &str) -> Result<f64, String> {
        let daten = self.schrift_daten(self.schrift);
        zeichen_pruefen(daten, &self.schrift_name(self.schrift), text)?;
        Ok(breite_pt(daten, self.groesse_pt, text)? / MM)
    }

    pub fn linie(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let (farbe, breite_mm, deck) = (self.farbe, self.strich_mm, self.deck);
        self.hier().ops.push(Op::Linie { x1, y1, x2, y2, breite_mm, farbe, deck });
    }

    /// Rechteck von (x,y) mit Breite/Hoehe -- `fuellen` entscheidet zwischen
    /// Flaeche und Umriss.
    pub fn rechteck(&mut self, x: f64, y: f64, b: f64, h: f64, fuellen: bool) {
        let (farbe, breite_mm, deck) = (self.farbe, self.strich_mm, self.deck);
        self.hier().ops.push(Op::Rechteck { x, y, b, h, fuellen, breite_mm, farbe, deck });
    }

    /// Kreis um (x, y) mit Radius r (mm); ein Radius <= 0 ist ein Fehler.
    pub fn kreis(&mut self, x: f64, y: f64, r: f64, fuellen: bool) -> Result<(), String> {
        if !(r > 0.0) { return Err(format!("der Radius muss groesser als 0 sein, nicht {}", r)); }
        let (farbe, breite_mm, deck) = (self.farbe, self.strich_mm, self.deck);
        self.hier().ops.push(Op::Kreis { x, y, r, fuellen, breite_mm, farbe, deck });
        Ok(())
    }

    /// Geschlossenes Vieleck aus den Punkten (mm).
    pub fn vieleck(&mut self, punkte: Vec<(f64, f64)>, fuellen: bool) -> Result<(), String> {
        if punkte.len() < 3 {
            return Err(format!("ein Vieleck braucht mindestens drei Punkte, nicht {}", punkte.len()));
        }
        let (farbe, breite_mm, deck) = (self.farbe, self.strich_mm, self.deck);
        self.hier().ops.push(Op::Vieleck { punkte, fuellen, breite_mm, farbe, deck });
        Ok(())
    }

    /// Ein Bild (RGBA) in das Rechteck x/y/b/h setzen; `h` <= 0 = nach dem
    /// Seitenverhaeltnis des Bildes.
    pub fn bild(&mut self, x: f64, y: f64, b: f64, h: f64, rgba: Vec<u8>, pw: u32, ph: u32) -> Result<(), String> {
        if pw == 0 || ph == 0 || rgba.len() != pw as usize * ph as usize * 4 {
            return Err("das Bild ist leer".into());
        }
        if !(b > 0.0) { return Err(format!("die Breite muss groesser als 0 sein, nicht {}", b)); }
        let h = if h > 0.0 { h } else { b * ph as f64 / pw as f64 };
        let deck = self.deck;
        self.hier().ops.push(Op::Bild { x, y, b, h, rgba: Arc::new(rgba), pw, ph, deck });
        Ok(())
    }

    /// Das fertige PDF als Bytes.
    ///
    /// KEIN Erstellungsdatum: dasselbe Programm soll zweimal dieselbe Datei
    /// ergeben. krilla schreibt ohne Datum und bildet die Dokumentkennung aus
    /// dem Inhalt -- gemessen: zwei Laeufe, Byte fuer Byte gleich.
    pub fn bauen(&self) -> Result<Vec<u8>, String> {
        use krilla::color::rgb;
        use krilla::geom::{PathBuilder, Point};
        use krilla::metadata::Metadata;
        use krilla::page::PageSettings;
        use krilla::paint::{Fill, Stroke};
        use krilla::text::{Font, TextDirection};

        fn farbe(f: (f64, f64, f64)) -> rgb::Color {
            let k = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            rgb::Color::new(k(f.0), k(f.1), k(f.2))
        }
        let fuellung = |f| Fill { paint: farbe(f).into(), ..Default::default() };
        let strich = |f, mm: f64| Stroke { paint: farbe(f).into(), width: (mm * MM) as f32, ..Default::default() };
        let p = |v: f64| (v * MM) as f32;
        // Flaeche oder Umriss -- dieselbe Entscheidung fuer Rechteck, Kreis und
        // Vieleck.
        let malen = |fl: &mut krilla::surface::Surface<'_>, pfad: &krilla::geom::Path, fuellen: bool, f: (f64, f64, f64), mm: f64| {
            if fuellen {
                fl.set_stroke(None);
                fl.set_fill(Some(fuellung(f)));
            } else {
                fl.set_fill(None);
                fl.set_stroke(Some(strich(f, mm)));
            }
            fl.draw_path(pfad);
        };

        let mut doc = krilla::Document::new();
        let mut angaben = Metadata::new().producer("Drachenhauch".to_string());
        if !self.titel.is_empty() { angaben = angaben.title(self.titel.clone()); }
        doc.set_metadata(angaben);

        let mut schriften: Vec<Option<Font>> = vec![None; EINGEBAUT.len() + self.geladen.len()];
        for s in &self.seiten {
            let einstellung = PageSettings::from_wh(p(s.breite_mm), p(s.hoehe_mm))
                .ok_or_else(|| format!("Seitengroesse {} x {} mm ist ungueltig", s.breite_mm, s.hoehe_mm))?;
            let mut seite = doc.start_page_with(einstellung);
            let mut flaeche = seite.surface();
            for op in &s.ops {
                let deck = match op {
                    Op::Text { deck, .. } | Op::Linie { deck, .. } | Op::Rechteck { deck, .. }
                    | Op::Kreis { deck, .. } | Op::Vieleck { deck, .. } | Op::Bild { deck, .. } => *deck,
                };
                // Jeder Befehl mit eigener Deckkraft ist eine eigene Gruppe:
                // was er ueberdeckt, scheint durch, sich selbst ueberdeckt er
                // nicht doppelt.
                let durchsicht = deck < 1.0;
                if durchsicht {
                    flaeche.push_opacity(krilla::num::NormalizedF32::new(deck.clamp(0.0, 1.0) as f32)
                        .unwrap_or(krilla::num::NormalizedF32::ONE));
                }
                match op {
                    Op::Text { x, y, text, schrift, groesse_pt, farbe: f, winkel, .. } => {
                        if schriften[*schrift].is_none() {
                            schriften[*schrift] = Some(Font::new(self.schrift_data(*schrift), 0)
                                .ok_or_else(|| format!("die Schrift '{}' laesst sich nicht einbetten", self.schrift_name(*schrift)))?);
                        }
                        let font = schriften[*schrift].clone().unwrap();
                        flaeche.set_stroke(None);
                        flaeche.set_fill(Some(fuellung(*f)));
                        // krilla zaehlt y nach unten: gegen den Uhrzeigersinn
                        // ist dort ein negativer Winkel.
                        let gedreht = *winkel != 0.0;
                        if gedreht {
                            flaeche.push_transform(&krilla::geom::Transform::from_rotate_at(-*winkel as f32, p(*x), p(*y)));
                        }
                        // Die Y-Angabe des Programms meint die OBERKANTE der
                        // Zeile, krilla setzt auf die Grundlinie -- also um die
                        // Schriftgroesse nach unten ruecken.
                        flaeche.draw_text(Point::from_xy(p(*x), p(*y) + *groesse_pt as f32),
                                          font, *groesse_pt as f32, text, false, TextDirection::Auto);
                        if gedreht { flaeche.pop(); }
                    }
                    Op::Linie { x1, y1, x2, y2, breite_mm, farbe: f, .. } => {
                        let mut pb = PathBuilder::new();
                        pb.move_to(p(*x1), p(*y1));
                        pb.line_to(p(*x2), p(*y2));
                        if let Some(pfad) = pb.finish() {
                            flaeche.set_fill(None);
                            flaeche.set_stroke(Some(strich(*f, *breite_mm)));
                            flaeche.draw_path(&pfad);
                        }
                    }
                    Op::Rechteck { x, y, b, h, fuellen, breite_mm, farbe: f, .. } => {
                        let mut pb = PathBuilder::new();
                        pb.move_to(p(*x), p(*y));
                        pb.line_to(p(*x + *b), p(*y));
                        pb.line_to(p(*x + *b), p(*y + *h));
                        pb.line_to(p(*x), p(*y + *h));
                        pb.close();
                        if let Some(pfad) = pb.finish() {
                            malen(&mut flaeche, &pfad, *fuellen, *f, *breite_mm);
                        }
                    }
                    Op::Kreis { x, y, r, fuellen, breite_mm, farbe: f, .. } => {
                        // vier Bezier-Viertel; k = 4/3 * (sqrt(2) - 1) haelt den
                        // Radius in der Mitte jedes Viertels auf 0,03 % genau
                        let (cx, cy, rr) = (p(*x), p(*y), p(*r));
                        let k = 0.552_284_8_f32 * rr;
                        let mut pb = PathBuilder::new();
                        pb.move_to(cx + rr, cy);
                        pb.cubic_to(cx + rr, cy + k, cx + k, cy + rr, cx, cy + rr);
                        pb.cubic_to(cx - k, cy + rr, cx - rr, cy + k, cx - rr, cy);
                        pb.cubic_to(cx - rr, cy - k, cx - k, cy - rr, cx, cy - rr);
                        pb.cubic_to(cx + k, cy - rr, cx + rr, cy - k, cx + rr, cy);
                        pb.close();
                        if let Some(pfad) = pb.finish() {
                            malen(&mut flaeche, &pfad, *fuellen, *f, *breite_mm);
                        }
                    }
                    Op::Vieleck { punkte, fuellen, breite_mm, farbe: f, .. } => {
                        let mut pb = PathBuilder::new();
                        pb.move_to(p(punkte[0].0), p(punkte[0].1));
                        for q in &punkte[1..] { pb.line_to(p(q.0), p(q.1)); }
                        pb.close();
                        if let Some(pfad) = pb.finish() {
                            malen(&mut flaeche, &pfad, *fuellen, *f, *breite_mm);
                        }
                    }
                    Op::Bild { x, y, b, h, rgba, pw, ph, .. } => {
                        let bild = krilla::image::Image::from_rgba8(rgba.as_ref().clone(), *pw, *ph);
                        if let Some(groesse) = krilla::geom::Size::from_wh(p(*b), p(*h)) {
                            flaeche.push_transform(&krilla::geom::Transform::from_translate(p(*x), p(*y)));
                            flaeche.draw_image(bild, groesse);
                            flaeche.pop();
                        }
                    }
                }
                if durchsicht { flaeche.pop(); }
            }
            flaeche.finish();
            seite.finish();
        }
        doc.finish().map_err(|e| format!("das PDF liess sich nicht schreiben: {:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zaehle(heu: &[u8], nadel: &[u8]) -> usize {
        heu.windows(nadel.len()).filter(|w| *w == nadel).count()
    }

    #[test]
    fn seitenmasse() {
        assert_eq!(seitenmass("a4", false), Some((210.0, 297.0)));
        assert_eq!(seitenmass("A4", true), Some((297.0, 210.0)));
        assert_eq!(seitenmass("gibtsnicht", false), None);
        assert_eq!(seitenmass("A0", false), Some((841.0, 1189.0)));
        assert_eq!(seitenmass("a2", true), Some((594.0, 420.0)));
    }

    #[test]
    fn alte_namen_meinen_dejavu() {
        let d = Dokument::neu(210.0, 297.0);
        assert_eq!(d.schrift_suchen("sans"), Ok(0));
        assert_eq!(d.schrift_suchen("HELVETICA"), Ok(0));
        assert_eq!(d.schrift_suchen("courier-fett"), Ok(9));
        assert_eq!(d.schrift_suchen("times-fett-kursiv"), Ok(7));
        assert!(d.schrift_suchen("comic sans").unwrap_err().contains("PDF_FONT_LOAD"));
        assert!(d.schrift_suchen("symbol").unwrap_err().contains("gibt es nicht mehr"));
    }

    #[test]
    fn mono_ist_dicktengleich_und_sans_nicht() {
        let mut d = Dokument::neu(210.0, 297.0);
        d.setze_schrift(8, 10.0);
        let (i, w) = (d.textbreite("iiii").unwrap(), d.textbreite("WWWW").unwrap());
        assert!((i - w).abs() < 1e-9 && i > 0.0);
        d.setze_schrift(0, 10.0);
        assert!(d.textbreite("iiii").unwrap() < d.textbreite("WWWW").unwrap());
        // Doppelte Groesse, doppelte Breite.
        let einfach = d.textbreite("Rechnung").unwrap();
        d.setze_schrift(0, 20.0);
        assert!((d.textbreite("Rechnung").unwrap() - 2.0 * einfach).abs() < 1e-9);
    }

    #[test]
    fn fehlendes_zeichen_ist_ein_fehler() {
        let mut d = Dokument::neu(210.0, 297.0);
        assert!(d.text(20.0, 25.0, "Grüße, Ωμέγα, Привет, 1 € \u{2713}", 0.0).is_ok());
        let e = d.text(20.0, 25.0, "中", 0.0).unwrap_err();
        assert!(e.contains("kein Zeichen") && e.contains("U+4E2D"), "{}", e);
        assert!(d.textbreite("中").is_err());
    }

    #[test]
    fn eigene_schrift_braucht_einen_freien_namen_und_lesbare_daten() {
        let mut d = Dokument::neu(210.0, 297.0);
        assert!(d.schrift_laden("sans", dejavu::sans::regular().to_vec()).is_err());
        assert!(d.schrift_laden("helvetica-fett", dejavu::sans::bold().to_vec()).is_err());
        assert!(d.schrift_laden("kaputt", b"keine schrift".to_vec()).is_err());
        d.schrift_laden("Hausschrift", dejavu::serif::italic().to_vec()).unwrap();
        assert_eq!(d.schrift_suchen("hausschrift"), Ok(12));
        assert_eq!(d.schrift_name(12), "hausschrift");
        assert!(d.schrift_laden("hausschrift", dejavu::serif::italic().to_vec()).is_err());
    }

    #[test]
    fn ein_leeres_dokument_ist_ein_gueltiges_pdf() {
        let b = Dokument::neu(210.0, 297.0).bauen().unwrap();
        assert!(b.starts_with(b"%PDF-"));
        assert!(b.ends_with(b"%%EOF") || b.ends_with(b"%%EOF\n"));
        assert_eq!(zaehle(&b, b"/Type/Page/"), 1);
    }

    #[test]
    fn seiten_zaehlen_und_titel_steht_drin() {
        let mut d = Dokument::neu(210.0, 297.0);
        d.titel = "Rechnung 4711".into();
        d.text(20.0, 25.0, "eins", 0.0).unwrap();
        d.neue_seite();
        d.neue_seite();
        d.linie(20.0, 45.0, 190.0, 45.0);
        d.rechteck(20.0, 50.0, 40.0, 10.0, true);
        let b = d.bauen().unwrap();
        assert_eq!(zaehle(&b, b"/Type/Page/"), 3);
        assert_eq!(zaehle(&b, b"(Rechnung 4711)"), 1);
    }

    #[test]
    fn zweimal_bauen_gibt_dieselben_bytes() {
        let mut d = Dokument::neu(210.0, 297.0);
        d.text(20.0, 25.0, "gleich", 0.0).unwrap();
        assert_eq!(d.bauen().unwrap(), d.bauen().unwrap());
    }
}
