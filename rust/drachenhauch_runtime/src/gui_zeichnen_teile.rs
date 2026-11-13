//! Zeichnen der grossen Widgets: Richtext, Baum, Regler, Karte, Bild, Popups, Tabelle und Zellen.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    /// Welcher Verweis liegt an dieser Stelle (Koordinaten relativ zum
    /// INHALT, also mit Scroll)? EINE Quelle fuer Klick und Mauszeiger.
    pub(super) fn rt_link_unter(r: &RichState, x: i32, y: i32) -> Option<String> {
        for z in r.zeilen.iter() {
            if y < z.y || y >= z.y + z.h { continue; }
            for l in z.laeufe.iter() {
                if l.link >= 0 && x >= l.x && x < l.x + l.breite {
                    return r.ziele.get(l.link as usize).cloned();
                }
            }
        }
        None
    }

    /// Gesetzten Text zeichnen. Nur die Zeilen im Bild -- ein Dokument mit
    /// zweitausend Zeilen darf nicht zweitausend Textaufrufe je Bild kosten.
    pub(super) fn draw_richtext(&self, g: &mut Graphics, wi: usize, idx: usize) {
        let wdg = &self.windows[wi].widgets[idx];
        let (ax, ay, w, h) = self.abs_rect(wi, wdg);
        let bg = self.wcol(wdg, "bg", "widget_bg");
        self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, bg, self.wcol(wdg, "border", "widget_border"));
        let r = wdg.rich.as_ref().unwrap();
        let fg = self.txt_col(wdg);
        let leise = self.th("muted_fg");
        let acc = self.acc_col(wdg);
        // Heller Grund oder dunkler? Danach richten sich die Farben der
        // Ueberschriften und des Codes -- auf Weiss braucht ein Gold mehr
        // Tiefe als auf Anthrazit.
        let hell = leuchtdichte(bg & 0xFF_FFFF) > 0.35;
        let code_grund = if hell { shade(bg & 0xFF_FFFF, -12) } else { shade(bg & 0xFF_FFFF, 14) };
        let (h2c, h3c, inline_c) = if hell { (0x9A5B00, 0x137A62, 0xB0381E) }
                                   else { (0xF0B45A, 0x7FD6BE, 0xF4927A) };
        let vorgabe: [i64; 6] = if hell {
            [0x6A737D, 0x22863A, 0x005CC5, 0xB3245A, -1, 0x6F42C1]
        } else {
            [0x7F8C98, 0xE6C07B, 0xD19A66, 0xC678DD, -1, 0x56B6C2]
        };
        let code_farbe = |art: u8| -> i64 {
            if art == 0 { return fg; }
            let k = (art - 1) as usize;
            match r.code_farben.get(k).copied().flatten() {
                Some(c) => c,
                None => if vorgabe[k] < 0 { fg } else { vorgabe[k] },
            }
        };
        let font = self.wfont(g, wdg);
        let cfont = if r.code_font >= 0 { r.code_font } else { font };
        g.push_clip(ax + 1, ay + 1, w - 2, h - 2);
        for (zi, z) in r.zeilen.iter().enumerate() {
            let zy = ay + z.y - r.scroll;
            if zy + z.h < ay { continue; }
            if zy > ay + h { break; }
            let (bx0, bx1) = if z.bx1 > 0 { (ax + z.bx0, ax + z.bx1) } else { (ax + self.sk(6), ax + w - self.sk(7)) };
            match z.band {
                1 => {
                    g.box_fill(bx0, zy - 1, bx1, zy + z.h - 2, code_grund);
                    // Ein schmaler Akzentstreifen links sagt "Code", auch
                    // wenn der Grund sich kaum vom Fenster abhebt.
                    g.box_fill(bx0, zy - 1, bx0 + self.sk(3) - 1, zy + z.h - 2, (0x60i64 << 24) | (acc & 0xFF_FFFF));
                }
                2 => g.box_fill(bx0, zy, bx1, zy + z.h - 1, (0x90i64 << 24) | (acc & 0xFF_FFFF)),
                3 => g.box_fill(bx0, zy, bx1, zy + z.h - 1, (0x3Ci64 << 24) | (acc & 0xFF_FFFF)),
                4 => g.box_fill(bx0, zy, bx1, zy + z.h - 1, if hell { shade(bg & 0xFF_FFFF, -7) } else { shade(bg & 0xFF_FFFF, 8) }),
                5 => {
                    g.box_fill(bx0, zy, bx1, zy + z.h - 1, (0x1Ei64 << 24) | (acc & 0xFF_FFFF));
                    g.box_fill(bx0, zy, bx0 + self.sk(3) - 1, zy + z.h - 1, acc);
                }
                _ => {}
            }
            if z.linie {
                let ly = zy + z.h / 2;
                g.line(ax + self.sk(12), ly, ax + w - self.sk(13), ly, shade(leise, -40));
            }
            if z.balken >= 0 {
                g.box_fill(ax + z.balken, zy, ax + z.balken + self.sk(2), zy + z.h - 1, acc);
            }
            // Code im Fliesstext bekommt einen eigenen Grund -- ueber die
            // Luecke hinweg, wenn zwei Code-Woerter nebeneinander stehen,
            // sonst saehe `DIM x` aus wie zwei Bausteine.
            for (k, l) in z.laeufe.iter().enumerate() {
                if l.rolle != 3 || l.link >= 0 { continue; }
                let weiter = z.laeufe.get(k + 1).map(|n| n.rolle == 3 && n.link < 0).unwrap_or(false);
                let x1 = if weiter { z.laeufe[k + 1].x } else { l.x + l.breite + self.sk(2) };
                let x0 = if k > 0 && z.laeufe[k - 1].rolle == 3 && z.laeufe[k - 1].link < 0 { l.x } else { l.x - self.sk(2) };
                g.box_fill(ax + x0, zy, ax + x1, zy + z.h - self.sk(3), code_grund);
            }
            // Fundstellen (schwach) und Auswahl (Akzent) UNTER den Text: je
            // Lauf der Teil, der in den Bereich faellt, an seinen Zeichen gemessen.
            let hat_nadel = !r.nadel.is_empty();
            if r.hat_auswahl || hat_nadel {
                let (text, starts) = Self::rt_zeilentext(z);
                let mut bereiche: Vec<(usize, usize, i64)> = Vec::new();
                if hat_nadel {
                    let klein: Vec<char> = text.to_lowercase().chars().collect();
                    let nadel: Vec<char> = r.nadel.chars().collect();
                    if klein.len() >= nadel.len() {
                        for c in 0..=(klein.len() - nadel.len()) {
                            if klein[c..c + nadel.len()] == nadel[..] { bereiche.push((c, c + nadel.len(), (0x48i64 << 24) | 0xE8B83A)); }
                        }
                    }
                }
                if r.hat_auswahl {
                    let ((az, ac), (ez, ec)) = Self::rt_bereich(r);
                    if zi >= az && zi <= ez {
                        let n = text.chars().count();
                        let von = if zi == az { ac } else { 0 };
                        let bis = if zi == ez { ec } else { n };
                        // Ueber eine Zeile hinaus markiert bis zu ihrem Ende.
                        if bis > von || (zi < ez && von == n) { bereiche.push((von, bis.max(von), (0x70i64 << 24) | (acc & 0xFF_FFFF))); }
                    }
                }
                for (von, bis, farbe) in bereiche {
                    for (k, l) in z.laeufe.iter().enumerate() {
                        let s = starts[k];
                        let len = l.text.chars().count();
                        let (a, b) = (von.max(s), bis.min(s + len));
                        if a >= b { continue; }
                        let f = if l.code { cfont } else { font };
                        let zeichen: Vec<char> = l.text.chars().collect();
                        let x0 = g.text_width_stil(&zeichen[..a - s].iter().collect::<String>(), l.groesse, f, l.stil);
                        let x1 = g.text_width_stil(&zeichen[..b - s].iter().collect::<String>(), l.groesse, f, l.stil);
                        g.box_fill(ax + l.x + x0, zy, ax + l.x + x1.max(x0 + 1), zy + z.h - 2, farbe);
                    }
                    // Das Leerzeichen zwischen zwei Laeufen gehoert mit dazu --
                    // sonst saehe eine Auswahl ueber drei Woerter aus wie drei.
                    for k in 1..z.laeufe.len() {
                        let s = starts[k];
                        if s == 0 || s - 1 < von || s - 1 >= bis { continue; }
                        let (vl, l) = (&z.laeufe[k - 1], &z.laeufe[k]);
                        let x0 = vl.x + vl.breite;
                        if l.x > x0 { g.box_fill(ax + x0, zy, ax + l.x, zy + z.h - 2, farbe); }
                    }
                }
            }
            for (k, l) in z.laeufe.iter().enumerate() {
                let farbe = match l.rolle {
                    1 => leise, 2 => acc, 3 => inline_c,
                    4 => acc, 5 => h2c, 6 => h3c,
                    8 => code_farbe(l.art),
                    _ => fg,
                };
                let farbe = if l.link >= 0 { acc } else { farbe };
                let f = if l.code { cfont } else { font };
                // Kommentare im Code kursiv -- so liest man sie weg.
                let stil = l.stil | if l.rolle == 8 && l.art == 1 { crate::schnitt::KURSIV } else { 0 };
                g.text_styled_stil(ax + l.x, zy + l.dy, l.text.clone(), farbe, f, l.groesse, stil);
                // Ein Verweis traegt seine Linie DURCHGEHEND: ueber mehrere
                // Woerter hinweg, nicht als Strich je Wort mit Luecken dazwischen.
                if l.link >= 0 {
                    let (off, dicke) = crate::schnitt::linie_unter(l.groesse as f32);
                    let ly = zy + l.dy + off.round() as i32;
                    let d = (dicke.round() as i32).max(1);
                    let x1 = match z.laeufe.get(k + 1) {
                        Some(n) if n.link == l.link => n.x,
                        _ => l.x + l.breite,
                    };
                    g.box_fill(ax + l.x, ly, ax + x1, ly + d - 1, (0x90i64 << 24) | (acc & 0xFF_FFFF));
                }
            }
        }
        // Knoepfe an den Codebloecken: kleine Flaechen im Kopfstreifen, mit
        // Rahmen und in gedaempfter Schrift -- sie sollen da sein, aber nicht
        // lauter als der Code.
        let kgr = ((if r.basis > 0 { r.basis } else { self.wsize(g, wdg) }) - 2).max(11);
        for (by, bh, _, rects) in &r.codebloecke {
            let zy = ay + by - r.scroll;
            if zy + bh < ay || zy > ay + h { continue; }
            for (k, (bx, bw)) in rects.iter().enumerate() {
                let x = ax + bx;
                let grund = if hell { shade(code_grund, -10) } else { shade(code_grund, 16) };
                let kante = if hell { shade(code_grund, -45) } else { shade(code_grund, 48) };
                let leise = self.leise(grund);
                g.box_fill(x, zy, x + bw - 1, zy + bh - 1, grund);
                g.rect(x, zy, x + bw - 1, zy + bh - 1, kante);
                let text = r.code_knoepfe.get(k).cloned().unwrap_or_default();
                let tw = g.text_width_stil(&text, kgr, font, 0);
                g.text_styled_stil(x + (bw - tw) / 2, zy + (bh - kgr) / 2, text, leise, font, kgr, 0);
            }
        }
        g.pop_clip();
        // Rollbalken, sobald es mehr gibt als hineinpasst -- ohne ihn sieht
        // ein langes Dokument aus wie ein kurzes.
        if r.inhalt_h > h {
            let sw = self.sk(4);
            let anteil = (h as f64 / r.inhalt_h as f64).min(1.0);
            let griff = ((h - 4) as f64 * anteil) as i32;
            let oben = ((h - 4 - griff) as f64
                * (r.scroll as f64 / (r.inhalt_h - h) as f64).clamp(0.0, 1.0)) as i32;
            g.box_fill(ax + w - sw - 2, ay + 2 + oben, ax + w - 3, ay + 2 + oben + griff, shade(leise, -20));
        }
    }

    pub(super) fn draw_tree(&self, g: &mut Graphics, wi: usize, idx: usize) {
        let wdg = &self.windows[wi].widgets[idx];
        let (ax, ay, w, h) = self.abs_rect(wi, wdg);
        self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
            self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
        let t = wdg.tree.as_ref().unwrap();
        let zeilen = Self::tree_zeilen_weich(t);
        let fg = self.txt_col(wdg);
        let acc = self.acc_col(wdg);
        let scroll = t.scroll;
        let hat_icon = t.nodes.iter().any(|n| n.icon >= 0 || !n.sym.is_empty());
        let isz = self.sk(TREE_ROW_H) - 4;
        // Mit Rollbalken enden die Zeilen vor ihm.
        let bar = self.list_bar_geom(wi, idx);
        let wz = if bar.is_some() { w - self.sk(8) - 2 } else { w };
        g.push_clip(ax + 1, ay + 1, w - 2, h - 2);
        if zeilen.is_empty() && !t.leer_text.is_empty() {
            let muted = self.leise(self.wcol(wdg, "bg", "widget_bg"));
            self.leer_hinweis(g, wdg, ax, ay + self.sk(10), w, &t.leer_text, muted);
        }
        let rh = self.sk(TREE_ROW_H);
        let mut summe = 0.0f64;
        for &(ni, anteil) in &zeilen {
            let ry = ay + 1 + summe.round() as i32 - scroll;
            summe += rh as f64 * anteil;
            let zh = ay + 1 + summe.round() as i32 - scroll - ry;
            if zh <= 0 || ry + rh < ay || ry > ay + h { continue; }
            // Eine Zeile, die gerade herauswaechst: auf ihren Anteil
            // beschnitten, der Inhalt steht oben -- wie unter einem Vorhang.
            let teil = anteil < 0.999;
            if teil { g.push_clip(ax + 1, ry, (w - 2).max(0), zh); }
            let node = &t.nodes[ni];
            let indent = node.level * self.sk(TREE_INDENT);
            let markiert = if t.multi { t.sel.get(ni).copied().unwrap_or(false) }
                           else { ni as i32 == t.selected };
            if markiert {
                g.box_fill(ax + 1, ry, ax + wz - 2, ry + self.sk(TREE_ROW_H) - 1, shade(acc, -110));
            } else if wdg.zeile_b.anteil(ni as i32) > 0.0 {
                let a = wdg.zeile_b.anteil(ni as i32);
                g.box_fill(ax + 1, ry, ax + wz - 2, ry + self.sk(TREE_ROW_H) - 1, deckkraft(shade(self.wcol(wdg, "bg", "widget_bg"), 18), a));
            }
            // Auf-/Zuklapp-Dreieck (nur bei Kindknoten), hinter der
            // Kaestchen-Spalte.
            let tx = ax + 4 + self.tree_kast_w(t) + indent;
            let cy = ry + self.sk(TREE_ROW_H) / 2;
            if node.has_children {
                // Der Winkel dreht sich mit: ">" zu, "v" offen, dazwischen
                // um 90 Grad gedreht (dieselben drei Punkte).
                let cx = tx + self.sk(TREE_TOGGLE_W) / 2;
                let a = weich(t.auf_t.get(ni).copied().unwrap_or(if node.expanded { 1.0 } else { 0.0 }));
                let (sn, cs) = (a * std::f64::consts::FRAC_PI_2).sin_cos();
                let dreh = |dx: f64, dy: f64| -> (i32, i32) {
                    (cx + (dx * cs - dy * sn).round() as i32, cy + (dx * sn + dy * cs).round() as i32)
                };
                let (p1, p2, p3) = (dreh(-2.0, -4.0), dreh(2.0, 0.0), dreh(-2.0, 4.0));
                g.line(p1.0, p1.1, p2.0, p2.1, fg);
                g.line(p2.0, p2.1, p3.0, p3.1, fg);
            }
            let mut lx = tx + self.sk(TREE_TOGGLE_W) + 2;
            if t.kaestchen {
                // Kaestchen wie bei der Liste: Rahmen, gefuellt mit Haken --
                // aber GANZ LINKS, vor dem Dreieck (siehe tree_kast_w).
                let (bx, cs) = self.tree_kast(ax, node.level);
                let by = ry + (self.sk(TREE_ROW_H) - cs) / 2;
                g.rect(bx, by, bx + cs - 1, by + cs - 1, self.wcol(wdg, "border", "widget_border"));
                if t.checks.get(ni).copied().unwrap_or(false) {
                    g.box_fill(bx + 2, by + 2, bx + cs - 3, by + cs - 3, acc);
                }
            }
            // Auf der Auswahl folgt die Schrift dem GRUND, wie in der Liste --
            // im hellen Thema stand sonst dunkle Schrift auf dunkelblauer Zeile
            // (gesehen im Navigator des Studios).
            let farbe = if markiert { lesbar_auf(shade(acc, -110)) } else if node.color >= 0 { node.color } else { fg };
            if hat_icon {
                // Platz fuer das Sinnbild bekommt JEDE Zeile, sobald eine
                // eines hat -- sonst staenden die Namen versetzt.
                if node.icon >= 0 {
                    g.draw_image_rect(node.icon, lx, ry + 2, isz, isz);
                } else if !node.sym.is_empty() {
                    // Ein eingebautes Sinnbild in der Farbe des Namens, etwas
                    // kleiner als die Zeile, damit es nicht an den Nachbarn klebt.
                    let s = (isz - 2).max(8);
                    sinnbild(g, &node.sym, lx + (isz - s) / 2, ry + (self.sk(TREE_ROW_H) - s) / 2, s, farbe, true);
                }
                lx += isz + 3;
            }
            self.wtext(g, wdg, lx, ry + (self.sk(TREE_ROW_H) - self.wsize(g, wdg)).max(0) / 2, node.label.clone(), farbe);
            if teil { g.pop_clip(); }
        }
        // Rollbalken wie bei der Liste.
        if let Some((tx, ty, bw, th, thy, thh)) = bar {
            let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
            g.box_fill(tx, ty, tx + bw - 1, ty + th - 1, shade(self.wcol(wdg, "bg", "widget_bg"), -10));
            let aktiv = self.listbar_zug.map(|(a, b, _)| (a, b) == (wi, idx)).unwrap_or(false)
                || (mx >= tx && mx < tx + bw && my >= thy && my < thy + thh);
            let rc = self.wcol(wdg, "border", "widget_border");
            g.round_rect(tx + 1, thy, tx + bw - 2, thy + thh - 1, 3, if aktiv { mischen(rc, acc, 0.5) } else { rc }, true);
        }
        // Umbenennen: Eingabefeld ueber dem Namen, wie bei der Liste.
        if let (Some(e), Some((ex, ey, ew, eh))) = (t.edit.as_ref(), self.baum_edit_rect(wi, idx)) {
            g.box_fill(ex, ey, ex + ew - 1, ey + eh - 1, shade(self.wcol(wdg, "bg", "widget_bg"), 12));
            g.rect(ex, ey, ex + ew - 1, ey + eh - 1, acc);
            let text: String = e.text.iter().collect();
            let tx = self.baum_text_x(t, ax, e.k);
            let ty = ey + (eh - self.wsize(g, wdg)).max(0) / 2;
            let x_bei = |g: &mut Graphics, n: i32| -> i32 {
                let vor: String = e.text.iter().take(n.max(0) as usize).collect();
                tx + self.wtext_width(g, wdg, &vor)
            };
            let (xa, xb) = (x_bei(g, e.anker.min(e.caret)), x_bei(g, e.anker.max(e.caret)));
            let cx = x_bei(g, e.caret);
            g.push_clip(ex + 1, ey + 1, (ew - 2).max(0), (eh - 2).max(0));
            if e.anker != e.caret { g.box_fill(xa, ey + 3, xb, ey + eh - 4, deckkraft(acc, 0.45)); }
            self.wtext(g, wdg, tx, ty, text, fg);
            g.line(cx, ey + 4, cx, ey + eh - 5, fg);
            g.pop_clip();
        }
        g.pop_clip();
    }

    /// Senkrechter Schieber (GUI_VSLIDER): dieselbe Rinne wie waagerecht, nur
    /// gedreht -- und gefuellt wird von UNTEN, weil der Wert nach oben waechst.
    pub(super) fn draw_vslider(&self, g: &mut Graphics, wdg: &Widget, ax: i32, ay: i32, w: i32, h: i32) {
        let handle_w = self.m("slider_handle_w");
        let span = wdg.max - wdg.min;
        let ratio = if span != 0.0 { ((wdg.value - wdg.min) / span).clamp(0.0, 1.0) } else { 0.0 };
        let acc = self.acc_col(wdg);
        if self.m_form("gradient") > 0 {
            let (tx, tr) = (ax + w / 2 - 3, ax + w / 2 + 3);
            self.fbox_tief(g, tx, ay, tr, ay + h - 1,
                           self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
            let von = ay + h - 1 - (ratio * (h - 1) as f64) as i32;
            if von < ay + h - 3 {
                g.round_gradient(tx + 1, von, tr - 1, ay + h - 2, 3, shade(acc, 24), shade(acc, -18));
            }
            let hy = ay + h - 1 - (ratio * (h - handle_w) as f64) as i32 - handle_w / 2;
            self.knauf(g, ax + w / 2, hy, (w / 2).max(5), self.griff_farbe(wdg));
        } else {
            g.box_fill(ax + w / 2 - 1, ay, ax + w / 2 + 1, ay + h - 1, self.wcol(wdg, "bg", "widget_bg"));
            g.rect(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "border", "widget_border"));
            let hy = ay + h - handle_w - (ratio * (h - handle_w) as f64) as i32;
            g.box_fill(ax, hy, ax + w - 1, hy + handle_w - 1, acc);
        }
    }

    /// Bild nach Modus: strecken (wie bisher), einpassen/fuellen mit
    /// Seitenverhaeltnis, mitte in Originalgroesse, kacheln.
/// Eine Kachel: Flaeche, Bild oben (mit Innenabstand), Titel kraeftig,
    /// Beschreibung umbrochen in der Schrift des Widgets -- nicht kleiner,
    /// sonst ist sie unter einem Bild in 190 Punkten Breite nicht zu lesen.
    /// Passt sie nicht, endet die letzte Zeile mit "...", und der Tooltip
    /// (falls gesetzt) sagt den Rest. Unter der Maus hebt sich die ganze
    /// Kachel und bekommt einen Akzentrahmen; gedrueckt sinkt sie.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_karte(&self, g: &mut Graphics, wi: usize, idx: usize, ax: i32, ay: i32, w: i32, h: i32) {
        let wdg = &self.windows[wi].widgets[idx];
        let (eh, ed) = if wdg.enabled { (weich(wdg.ueber_t), weich(wdg.druck_t)) } else { (0.0, 0.0) };
        let grund = self.wcol(wdg, "bg", "widget_bg");
        let bg = mischen(mischen(grund, shade(grund, 18), eh), shade(grund, -14), ed);
        let rahmen = mischen(self.wcol(wdg, "border", "widget_border"), self.acc_col(wdg), eh.max(ed));
        self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, bg, rahmen);
        if eh > 0.0 {
            let rad = self.m("corner_radius");
            g.round_rect(ax + 1, ay + 1, ax + w - 2, ay + h - 2, (rad - 1).max(0), deckkraft(self.acc_col(wdg), eh), false);
        }
        // Innenabstand: aus der Stildatei (senkrecht, waagerecht), sonst 6.
        let (pv, ph) = self.w_abstand(wdg).unwrap_or((self.sk(6), self.sk(6)));
        let sz = self.wsize(g, wdg);
        let font = self.wfont(g, wdg);
        let titel_sz = sz + self.sk(1);
        let zeile_h = sz + self.sk(3);
        let innen_w = (w - 2 * ph).max(1);
        // Bild: 16:10 der Innenbreite, aber nie so hoch, dass fuer Titel und
        // eine Zeile Beschreibung kein Platz bleibt.
        let rest_mind = titel_sz + self.sk(8) + if wdg.placeholder.is_empty() { 0 } else { zeile_h };
        let bild_h = if wdg.sel >= 0 { (innen_w * 10 / 16).min(h - 2 * pv - rest_mind).max(0) } else { 0 };
        if bild_h > 0 {
            self.draw_bild(g, wdg, ax + ph, ay + pv, innen_w, bild_h);
            g.rect(ax + ph, ay + pv, ax + ph + innen_w - 1, ay + pv + bild_h - 1, shade(rahmen, -20));
        }
        let mut y = ay + pv + bild_h + if bild_h > 0 { self.sk(6) } else { 0 };
        let tfarbe = mischen(self.txt_col(wdg), self.acc_col(wdg), eh);
        g.push_clip(ax + ph, ay, innen_w, h);
        if !wdg.text.is_empty() {
            // Kraeftig: zweimal, um einen Punkt versetzt -- aus einer Schrift
            // laesst sich keine Strichstaerke rechnen (wie beim gesetzten Text).
            let mut titel = wdg.text.clone();
            while titel.chars().count() > 1 && g.text_width_in(&titel, titel_sz, font) > innen_w {
                titel.pop();
            }
            if titel != wdg.text { titel = format!("{}...", titel.trim_end().trim_end_matches('.')); }
            g.text_styled(ax + ph, y, titel.clone(), tfarbe, font, titel_sz);
            g.text_styled(ax + ph + 1, y, titel, tfarbe, font, titel_sz);
            y += titel_sz + self.sk(4);
        }
        if !wdg.placeholder.is_empty() {
            let unten = ay + h - pv;
            let platz = ((unten - y) / zeile_h).max(0) as usize;
            let mut zeilen = self.umbrechen(g, wdg, &wdg.placeholder, innen_w);
            if zeilen.len() > platz && platz > 0 {
                // Der Rest rueckt in die letzte sichtbare Zeile nach und wird
                // erst dort abgeschnitten -- sonst endete ein Text, dessen
                // zweites Wort nicht mehr passte, schon nach dem ersten.
                let rest = zeilen[platz - 1..].join(" ");
                zeilen.truncate(platz);
                zeilen[platz - 1] = rest;
                if let Some(l) = zeilen.last_mut() {
                    while !l.is_empty() && self.wtext_width(g, wdg, &format!("{}...", l)) > innen_w { l.pop(); }
                    *l = format!("{}...", l.trim_end());
                }
            } else if platz == 0 {
                zeilen.clear();
            }
            let farbe = self.leise(bg);
            for z in zeilen {
                self.wtext(g, wdg, ax + ph, y, z, farbe);
                y += zeile_h;
            }
        }
        g.pop_clip();
    }

    pub(super) fn draw_bild(&self, g: &mut Graphics, wdg: &Widget, ax: i32, ay: i32, w: i32, h: i32) {
        let tex = wdg.sel as i64;
        if wdg.bildmodus == 0 { g.draw_image_rect(tex, ax, ay, w, h); return; }
        let (iw, ih) = (g.image_width(tex).unwrap_or(0) as i32, g.image_height(tex).unwrap_or(0) as i32);
        if iw <= 0 || ih <= 0 || w <= 0 || h <= 0 { return; }
        g.push_clip(ax, ay, w, h);
        match wdg.bildmodus {
            1 | 2 => {
                let (sx, sy) = (w as f64 / iw as f64, h as f64 / ih as f64);
                let f = if wdg.bildmodus == 1 { sx.min(sy) } else { sx.max(sy) };
                let (dw, dh) = (((iw as f64 * f) as i32).max(1), ((ih as f64 * f) as i32).max(1));
                g.draw_image_rect(tex, ax + (w - dw) / 2, ay + (h - dh) / 2, dw, dh);
            }
            3 => g.draw_image_rect(tex, ax + (w - iw) / 2, ay + (h - ih) / 2, iw, ih),
            _ => {
                let mut y = ay;
                while y < ay + h {
                    let mut x = ax;
                    while x < ax + w { g.draw_image_rect(tex, x, y, iw, ih); x += iw; }
                    y += ih;
                }
            }
        }
        g.pop_clip();
    }

    pub(super) fn draw_dropdown_popup(&self, g: &mut Graphics, wi: usize, idx: usize) {
        let wdg = &self.windows[wi].widgets[idx];
        if wdg.items.is_empty() { return; }
        let (px, py, pw, ph_voll) = self.dropdown_popup_rect(wi, idx);
        // Aufklappen: die Liste rollt von oben herab (die Eintraege stehen
        // schon an ihrer Stelle, sichtbar ist der obere Teil) und blendet
        // dabei ein. Der Treffertest nimmt die volle Liste -- so schnell
        // klickt niemand, und ein Klick soll nie ins Leere gehen.
        // Im Bild des Klicks hat der Uebergang noch nicht begonnen
        // (`uebergaenge` lief vor dem Druck) -- das heisst "noch zu",
        // nicht "ganz offen", sonst blitzte die volle Liste einmal auf.
        let t = if self.dd_auf_von == Some((wi, idx)) { weich(self.dd_auf_t) }
                else if self.m("uebergang") > 0 { 0.0 } else { 1.0 };
        let ph = ((ph_voll as f64 * t).round() as i32).max(1);
        // Klappt die Liste nach OBEN auf, rollt sie von unten herauf.
        let (_, box_y, _, _) = self.abs_rect(wi, wdg);
        let nach_oben = py < box_y;
        let clip_y = if nach_oben { py + ph_voll - ph } else { py };
        let bg = deckkraft(self.wcol(wdg, "bg", "widget_bg"), 0.35 + 0.65 * t);
        let border = self.wcol(wdg, "border", "widget_border");
        let fg = self.wcol(wdg, "fg", "text_fg");
        let acc = self.wcol(wdg, "accent", "accent");
        let pad = self.m("pad");
        let ih = self.sk(DROPDOWN_ITEM_H);
        let n = wdg.items.len() as i32;
        let rows = (ph_voll / ih).max(1);
        let scroll = if self.dd_auf_von == Some((wi, idx)) { self.dd_scroll } else { 0 };
        let bar = n > rows;
        let bw = self.sk(8);
        let pz = if bar { pw - bw - 2 } else { pw };
        g.push_clip(px, clip_y, pw, ph);
        g.box_fill(px, py, px + pw - 1, py + ph_voll - 1, bg);
        g.rect(px, py, px + pw - 1, py + ph_voll - 1, border);
        let mark = if self.dd_auf_von == Some((wi, idx)) { self.dd_mark } else { -1 };
        for k in scroll..(scroll + rows).min(n) {
            let it = &wdg.items[k as usize];
            let iy = py + (k - scroll) * ih;
            let a = wdg.zeile_b.anteil(k);
            if k == wdg.sel {
                g.box_fill(px + 1, iy, px + pz - 2, iy + ih - 1, shade(acc, -110));
            } else if k == mark {
                g.box_fill(px + 1, iy, px + pz - 2, iy + ih - 1, shade(bg, 22));
            } else if a > 0.0 {
                g.box_fill(px + 1, iy, px + pz - 2, iy + ih - 1, deckkraft(shade(bg, 22), a));
            }
            // Die Tastatur-Markierung auf der gewaehlten Zeile: ein Rahmen,
            // sonst saehe man sie dort nicht.
            if k == mark { g.rect(px + 1, iy, px + pz - 2, iy + ih - 1, acc); }
            let th14 = self.ctext_height(g);
            self.ctext(g, px + pad, iy + (ih - th14) / 2, it.clone(), fg);
        }
        // Rollbalken, wenn nicht alles hineinpasst.
        if bar {
            let (tx, ty, th) = (px + pw - 1 - bw, py + 1, (ph_voll - 2).max(1));
            let thh = ((th as i64 * rows as i64 / n.max(1) as i64) as i32).clamp(self.sk(12).min(th), th);
            let thy = ty + ((th - thh) as f64 * (scroll as f64 / (n - rows).max(1) as f64)) as i32;
            g.box_fill(tx, ty, tx + bw - 1, ty + th - 1, shade(bg, -10));
            g.round_rect(tx + 1, thy, tx + bw - 2, thy + thh - 1, 3, border, true);
        }
        g.pop_clip();
    }

    /// Flaechenfarbe einer Zelle: eigene Farbe > Zeilenfarbe > Zebra > nichts.
    /// -1 heisst jeweils "nicht gesetzt" und reicht an die naechste Stufe
    /// weiter -- so faerbt eine Zeilenfarbe alle Zellen, eine einzelne Zelle
    /// kann aber ausscheren.
    pub(super) fn cell_bg(t: &TableState, r: usize, c: usize, zebra_bg: i64) -> i64 {
        if let Some(cell) = t.cell(r, c) {
            if cell.bg() >= 0 { return cell.bg(); }
        }
        if let Some(&rb) = t.row_bg.get(r) {
            if rb >= 0 { return rb; }
        }
        zebra_bg
    }
    pub(super) fn cell_fg(t: &TableState, r: usize, c: usize, thema: i64) -> i64 {
        if let Some(cell) = t.cell(r, c) {
            if cell.fg() >= 0 { return cell.fg(); }
        }
        if let Some(&rf) = t.row_fg.get(r) {
            if rf >= 0 { return rf; }
        }
        thema
    }

    /// x-Lage von Text in einer Zelle nach Ausrichtung. Getrennt, weil Kopf-
    /// und Datenzellen dieselbe Regel brauchen.
    pub(super) fn align_x(links: i32, breite: i32, textbreite: i32, align: i8, pad: i32) -> i32 {
        match align {
            1 => links + (breite - textbreite) / 2,
            2 => links + breite - textbreite - pad,
            _ => links + pad,
        }
        .max(links + 2)
    }

    /// Kleines Dreieck als Sortier-Anzeige. Ohne sie sieht man der Tabelle
    /// nicht an, WONACH sie gerade sortiert ist.
    pub(super) fn sort_arrow(g: &mut Graphics, cx: i32, cy: i32, desc: bool, c: i64) {
        // Aufsteigend zeigt der Pfeil NACH OBEN, absteigend nach unten -- so
        // herum kennt man es von jeder Tabelle. (Erst herum: die Spitze zeigte
        // genau falsch, was im Bild sofort auffiel.)
        let (a, b) = if desc { (3, -2) } else { (-3, 2) };
        g.line(cx - 4, cy + b, cx, cy + a + b, c);
        g.line(cx, cy + a + b, cx + 4, cy + b, c);
    }

    pub(super) fn draw_table(&self, g: &mut Graphics, wi: usize, idx: usize) {
        let gm = self.table_geom(wi, idx);
        let wdg = &self.windows[wi].widgets[idx];
        let t = wdg.tbl.as_ref().unwrap();
        let (ax, ay, w, h) = (gm.ax, gm.ay, gm.w, gm.h);
        let (body_x, body_y, body_w, body_h) = (gm.body_x, gm.body_y, gm.body_w, gm.body_h);

        let bg = self.wcol(wdg, "bg", "widget_bg");
        let border = self.wcol(wdg, "border", "widget_border");
        let fg = self.wcol(wdg, "fg", "text_fg");
        let title_fg = self.wcol(wdg, "fg", "title_fg");
        let accent = self.wcol(wdg, "accent", "accent");
        let sel_bg = shade(accent, -110);
        let hover_bg = shade(bg, 22);
        let zebra_bg = shade(bg, 10);
        let grid_c = shade(bg, 14);
        let schrift = self.wsize(g, wdg);

        // Aussenrahmen + Kopfzeile.
        g.box_fill(ax, ay, ax + w - 1, ay + h - 1, bg);
        g.rect(ax, ay, ax + w - 1, ay + h - 1, border);
        if gm.header_h > 0 {
            let titel_h = t.header_h;
            g.box_fill(ax + 1, ay + 1, ax + w - 2, ay + gm.header_h - 1, self.th("title_bg"));
            g.line(ax, ay + gm.header_h - 1, ax + w - 1, ay + gm.header_h - 1, border);

            // Kopf-Zellen. Feste Spalten stehen still, der Rest scrollt --
            // beides ueber `col_x`, damit Kopf und Daten nie versetzt sind.
            g.push_clip(ax + 1, ay + 1, gm.body_w_raw, gm.header_h - 2);
            for p in 0..gm.n_cols {
                let c = gm.order[p];                 // Anzeige-Position -> Datenspalte
                let cx = Self::col_x(&gm, p);
                let cw = gm.col_widths[p];
                let (kx, kw) = Self::col_clip(&gm, p);
                if cx + cw > kx && cx < kx + kw {
                    g.push_clip(kx, ay + 1, kw, gm.header_h - 2);
                    // Die Spalte, die gerade gezogen wird, hebt sich ab --
                    // sonst sieht man beim Verschieben nicht, was man haelt.
                    if let Some((dp, _, true)) = t.head_drag {
                        if dp == p {
                            g.box_fill(cx, ay + 1, cx + cw - 1, ay + titel_h - 2,
                                       (90i64 << 24) | (accent & 0xFFFFFF));
                        }
                    }
                    let al = *t.col_align.get(c).unwrap_or(&0);
                    // Platz fuer den Sortierpfeil freihalten, sonst schreibt
                    // ein langer Titel darueber.
                    let pfeil_platz = if t.sort_col == c as i32 { 14 } else { 0 };
                    // Ein Titel, breiter als seine Spalte, endet mit "..." --
                    // vorher schrieb er in den Kopf daneben (der Zuschnitt gilt
                    // nur der ganzen Kopfzeile; Kuehlschrank-App englisch:
                    // "Best-before dateStatus").
                    let s = self.text_kuerzen(g, wdg, t.header(c), cw - pfeil_platz - 2 * self.sk(TBL_PADDING));
                    let tw = self.wtext_width(g, wdg, &s);
                    let tx = Self::align_x(cx, cw - pfeil_platz, tw, al, self.sk(TBL_PADDING));
                    let ty = ay + (titel_h - schrift).max(0) / 2;
                    self.wtext(g, wdg, tx, ty, s, title_fg);
                    if t.sort_col == c as i32 {
                        Self::sort_arrow(g, cx + cw - 11, ay + titel_h / 2, t.sort_desc, title_fg);
                    }
                    if p < gm.n_cols - 1 {
                        g.line(cx + cw, ay + 1, cx + cw, ay + gm.header_h - 2, border);
                    }
                    g.pop_clip();
                }
            }
            // Filterzeile: je Spalte ein kleines Eingabefeld.
            if t.filter_row {
                let fy = ay + titel_h;
                for p in 0..gm.n_cols {
                    let c = gm.order[p];
                    let cx = Self::col_x(&gm, p);
                    let cw = gm.col_widths[p];
                    let (kx, kw) = Self::col_clip(&gm, p);
                    if cx + cw > kx && cx < kx + kw {
                        g.push_clip(kx, fy, kw, self.sk(TBL_FILTER_H));
                        let fokus = t.filter_focus == c as i32;
                        self.fbox_tief(g, cx + 2, fy + 2, cx + cw - 3, fy + self.sk(TBL_FILTER_H) - 3,
                                       self.th("win_bg"),
                                       if fokus { accent } else { self.th("widget_border") });
                        let leer = t.filters.get(c).map(|s| s.is_empty()).unwrap_or(true);
                        let s = if leer { "Filter ...".to_string() }
                                else { t.filters[c].clone() };
                        // Gedaempft wie der Platzhalter eines Textfelds, gegen DESSEN Grund
                        // gemessen (vorher aus der Kopfschrift abgeleitet: auf hellen
                        // Themen fast schwarz, auf dem schwarzen Feld unsichtbar).
                        let farbe = if leer { self.leise(self.th("win_bg")) } else { fg };
                        g.push_clip(cx + 4, fy + 2, (cw - 8).max(1), self.sk(TBL_FILTER_H) - 4);
                        self.wtext(g, wdg, cx + 6, fy + (self.sk(TBL_FILTER_H) - schrift).max(0) / 2,
                                   s, farbe);
                        g.pop_clip();
                        g.pop_clip();
                    }
                }
            }
            g.pop_clip();
            // Kante des festen Blocks. NUR wenn wirklich seitwaerts gescrollt
            // ist -- bei stehender Tabelle erklaert sie nichts und waere eine
            // willkuerliche Markierung. Und im Rahmenton, nicht im Akzent: der
            // bedeutet ueberall sonst "ausgewaehlt/aktiv", hier hiesse er
            // "ab hier scrollt es" -- zwei Dinge in einer Farbe.
            if gm.frozen > 0 && t.scroll_x > 0 {
                let fx = body_x + gm.frozen_w;
                g.line(fx, ay + 1, fx, ay + gm.header_h - 2, shade(border, 40));
            }
        }

        // Body.
        // Zellmodus: Bereich und aktuelle Zelle statt der ganzen Zeile.
        let ber = if t.zellmodus { t.bereich() } else { None };
        let ber_gross = ber.map(|(v1, v2, p1, p2)| v1 != v2 || p1 != p2).unwrap_or(false);
        let cur_p = if t.zellmodus { gm.order.iter().position(|&c| c as i32 == t.cur_c) } else { None };
        let gefokust = self.focus_widget == Some((wi, idx));
        g.push_clip(body_x, body_y, body_w, body_h);
        let first = (t.scroll_y / gm.row_h).max(0);
        let last = ((t.scroll_y + body_h) / gm.row_h + 1).min(gm.n_rows as i32);
        for r in first..last {
            // r = Ansichtszeile (Position auf dem Schirm), ri = Datenzeile.
            // Farben, Inhalte und die Auswahl haengen an den DATEN, die Lage
            // an der Ansicht -- werden die verwechselt, faerbt eine sortierte
            // Tabelle die falschen Zeilen.
            let ri = match t.view.get(r as usize) { Some(&x) => x, None => continue };
            let row_y = body_y + r * gm.row_h - t.scroll_y;
            // Zebra nach der ANSICHTSzeile, nicht nach der Datenzeile --
            // sonst waeren die Streifen nach dem Filtern unregelmaessig.
            let zebra = if t.zebra && r % 2 == 1 { zebra_bg } else { -1 };

            // Zellflaechen zuerst, danach Auswahl/Hover DARUEBER -- sonst
            // deckte eine eigene Zellfarbe die Auswahl zu und man saehe nicht
            // mehr, welche Zeile gewaehlt ist.
            for p in 0..gm.n_cols {
                let c = gm.order[p];
                let cx = Self::col_x(&gm, p);
                let cw = gm.col_widths[p];
                let (kx, kw) = Self::col_clip(&gm, p);
                if cx + cw > kx && cx < kx + kw {
                    let cb = Self::cell_bg(t, ri, c, zebra);
                    if cb >= 0 {
                        g.push_clip(kx, row_y, kw, gm.row_h);
                        g.box_fill(cx, row_y, cx + cw - 1, row_y + gm.row_h - 1, cb);
                        g.pop_clip();
                    }
                }
            }
            // Halbdurchsichtig ueber die Zellflaechen: eine eigene Zellfarbe
            // bleibt sichtbar, die Auswahl aber auch. Alpha steckt im
            // hoechsten Byte (0xAARRGGBB).
            if let (Some((v1, v2, p1, p2)), true) = (ber, ber_gross) {
                if (r as usize) >= v1 && (r as usize) <= v2 {
                    for p in p1..=p2.min(gm.n_cols.saturating_sub(1)) {
                        let (cx, cw) = (Self::col_x(&gm, p), gm.col_widths[p]);
                        let (kx, kw) = Self::col_clip(&gm, p);
                        g.push_clip(kx, row_y, kw, gm.row_h);
                        g.box_fill(cx, row_y, cx + cw - 1, row_y + gm.row_h - 1,
                                   (110i64 << 24) | (sel_bg & 0xFFFFFF));
                        g.pop_clip();
                    }
                }
            }
            if !t.zellmodus && t.ist_gewaehlt(ri as i32) {
                g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + gm.row_h - 1,
                           (150i64 << 24) | (sel_bg & 0xFFFFFF));
            } else if !t.zellmodus && wdg.zeile_b.anteil(ri as i32) > 0.0 {
                let a = (110.0 * wdg.zeile_b.anteil(ri as i32)).round().max(1.0) as i64;
                g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + gm.row_h - 1,
                           (a << 24) | (hover_bg & 0xFFFFFF));
            }

            // Inhalte.
            for p in 0..gm.n_cols {
                let c = gm.order[p];
                let cx = Self::col_x(&gm, p);
                let cw = gm.col_widths[p];
                let (kx, kw) = Self::col_clip(&gm, p);
                let clip_x = (cx + 1).max(kx);
                let clip_w = (cw - 2).min((kx + kw) - clip_x);
                if clip_w > 0 {
                    g.push_clip(clip_x, row_y, clip_w, gm.row_h);
                    // Baum: die erste sichtbare Spalte rueckt nach der Tiefe
                    // ein und traegt das Dreieck der Zeile.
                    let (dx, dw) = if t.baum && p == 0 {
                        let (e, o) = self.tt_einzug(t, ri);
                        if t.hat_kinder.get(ri).copied().unwrap_or(false) {
                            let (tx, ty, d) = (cx + e + self.sk(7), row_y + gm.row_h / 2, self.sk(4));
                            if t.offen.get(ri).copied().unwrap_or(false) {
                                g.triangle(tx - d, ty - d / 2, tx + d, ty - d / 2, tx, ty + d, fg);
                            } else {
                                g.triangle(tx - d / 2, ty - d, tx - d / 2, ty + d, tx + d, ty, fg);
                            }
                        }
                        (cx + o, (cw - o).max(1))
                    } else { (cx, cw) };
                    self.draw_cell(g, wdg, t, ri, c, dx, row_y, dw, gm.row_h, fg, accent, border);
                    g.pop_clip();
                }
                if t.grid && p < gm.n_cols - 1 && cx + cw > kx && cx + cw < kx + kw {
                    g.line(cx + cw, row_y, cx + cw, row_y + gm.row_h - 1, grid_c);
                }
            }
            if t.grid {
                g.line(body_x, row_y + gm.row_h - 1, body_x + body_w - 1, row_y + gm.row_h - 1, grid_c);
            }
            // Rahmen um die aktuelle Zelle, NACH dem Gitter -- sonst schnitte
            // eine Gitterlinie ihn durch. Ohne Fokus gedaempft: man soll sehen,
            // wo man war, aber nicht glauben, die Tasten kaemen hier an.
            if let Some(p) = cur_p {
                if ri as i32 == t.cur_r && t.edit_row < 0 {
                    let (cx, cw) = (Self::col_x(&gm, p), gm.col_widths[p]);
                    let (kx, kw) = Self::col_clip(&gm, p);
                    let farbe = if gefokust { accent } else { shade(border, 30) };
                    g.push_clip(kx, row_y, kw, gm.row_h);
                    g.rect(cx, row_y, cx + cw - 1, row_y + gm.row_h - 1, farbe);
                    g.rect(cx + 1, row_y + 1, cx + cw - 2, row_y + gm.row_h - 2, farbe);
                    g.pop_clip();
                }
            }
        }
        if gm.frozen > 0 && t.scroll_x > 0 {
            let fx = body_x + gm.frozen_w;
            g.line(fx, body_y, fx, body_y + body_h - 1, shade(border, 40));
        }
        // Leer ist eine Aussage: "Keine Treffer" statt einer Flaeche, die
        // aussieht, als sei die Tabelle kaputt (wie bei der Liste).
        if t.view.is_empty() && !t.leer_text.is_empty() {
            let muted = self.leise(bg);
            self.leer_hinweis(g, wdg, body_x, body_y + self.sk(10), body_w, &t.leer_text, muted);
        }
        g.pop_clip();

        // Senkrechte Bildlaufleiste.
        if gm.need_v {
            let sb_x = ax + gm.body_w_raw - self.sk(TBL_SCROLL_W) + 1;
            let track = body_h;
            let handle = Self::handle_height(track, body_h, gm.total_h);
            let ratio = if gm.max_scroll_y > 0 { t.scroll_y as f64 / gm.max_scroll_y as f64 } else { 0.0 };
            let hy = body_y + ((track - handle) as f64 * ratio) as i32;
            g.box_fill(sb_x, body_y, sb_x + self.sk(TBL_SCROLL_W) - 1, body_y + track - 1, shade(bg, -8));
            g.box_fill(sb_x + 2, hy, sb_x + self.sk(TBL_SCROLL_W) - 3, hy + handle - 1,
                       if t.drag_v { accent } else { border });
        }
        // Waagerechte Bildlaufleiste.
        if gm.need_h {
            let hs_y = ay + h - self.sk(TBL_SCROLL_W) - 1;
            let track = body_w;
            let handle = Self::handle_height(track, body_w, gm.total_w);
            let ratio = if gm.max_scroll_x > 0 { t.scroll_x as f64 / gm.max_scroll_x as f64 } else { 0.0 };
            let hx = body_x + ((track - handle) as f64 * ratio) as i32;
            g.box_fill(body_x, hs_y, body_x + track - 1, hs_y + self.sk(TBL_SCROLL_W) - 1, shade(bg, -8));
            g.box_fill(hx, hs_y + 2, hx + handle - 1, hs_y + self.sk(TBL_SCROLL_W) - 3,
                       if t.drag_h { accent } else { border });
        }
    }

    /// Eine einzelne Zelle zeichnen. Getrennt, weil sonst fuenf Zellarten in
    /// der ohnehin langen Zeichenschleife stehen wuerden.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_cell(&self, g: &mut Graphics, wdg: &Widget, t: &TableState,
                 r: usize, c: usize, cx: i32, cy: i32, cw: i32, ch: i32,
                 fg_thema: i64, accent: i64, border: i64) {
        let leer = Cell::text(String::new());
        let cell = t.cell(r, c).unwrap_or(&leer);
        let fgc = Self::cell_fg(t, r, c, fg_thema);
        let al = t.align_of(r, c);
        let schrift = self.wsize(g, wdg);

        // Wird DIESE Zelle gerade bearbeitet? Dann ein Eingabefeld statt Text.
        if t.edit_row == r as i32 && t.edit_col == c as i32 && t.col_typ_of(c) == 3 {
            // Auswahlspalte: der gewaehlte Eintrag und ein Pfeil -- die Liste
            // selbst liegt in der oberen Schicht (draw_wahl_popup).
            self.fbox_tief(g, cx + 1, cy + 2, cx + cw - 2, cy + ch - 3, self.th("win_bg"), accent);
            self.wtext(g, wdg, cx + 5, cy + (ch - schrift).max(0) / 2, t.edit_text.clone(), fg_thema);
            let (px, py) = (cx + cw - 12, cy + ch / 2);
            g.line(px - 4, py - 2, px, py + 2, accent);
            g.line(px, py + 2, px + 4, py - 2, accent);
            return;
        }
        if t.edit_row == r as i32 && t.edit_col == c as i32 {
            self.fbox_tief(g, cx + 1, cy + 2, cx + cw - 2, cy + ch - 3,
                           self.th("win_bg"), accent);
            let alle: Vec<char> = t.edit_text.chars().collect();
            let tx = cx + 5 - t.edit_scroll;
            let ty = cy + (ch - schrift).max(0) / 2;
            g.push_clip(cx + 3, cy + 2, (cw - 6).max(1), ch - 4);

            // Markierter Bereich als Flaeche HINTER dem Text -- ohne sie sieht
            // man nicht, was das naechste Zeichen ersetzen wuerde.
            let (lo, hi) = (t.edit_caret.min(t.edit_anchor).clamp(0, alle.len() as i32),
                            t.edit_caret.max(t.edit_anchor).clamp(0, alle.len() as i32));
            if lo != hi {
                let vor: String = alle[..lo as usize].iter().collect();
                let mitte: String = alle[lo as usize..hi as usize].iter().collect();
                let x1 = tx + self.wtext_width(g, wdg, &vor);
                let x2 = x1 + self.wtext_width(g, wdg, &mitte);
                g.box_fill(x1, cy + 4, x2 - 1, cy + ch - 5,
                           (140i64 << 24) | (accent & 0xFFFFFF));
            }
            self.wtext(g, wdg, tx, ty, t.edit_text.clone(), fg_thema);
            // Schreibmarke -- ohne sie sieht man nicht, wo man tippt.
            let bis: String = alle[..t.edit_caret.clamp(0, alle.len() as i32) as usize].iter().collect();
            let vx = tx + self.wtext_width(g, wdg, &bis);
            g.line(vx, cy + 4, vx, cy + ch - 5, accent);
            g.pop_clip();
            return;
        }

        match cell.kind() {
            CellKind::Text => {
                let tw = self.wtext_width(g, wdg, &cell.text);
                let tx = Self::align_x(cx, cw, tw, al, self.sk(TBL_PADDING));
                self.wtext(g, wdg, tx, cy + (ch - schrift).max(0) / 2, cell.text.clone(), fgc);
            }
            CellKind::Image => {
                if cell.image() >= 0 {
                    // In die Zelle einpassen, Seitenverhaeltnis behalten --
                    // gestreckte Bilder in einer Tabelle sehen kaputt aus.
                    let iw = g.image_width(cell.image()).unwrap_or(0);
                    let ih = g.image_height(cell.image()).unwrap_or(0);
                    if iw > 0 && ih > 0 {
                        let rand = 2;
                        let max_w = (cw - 2 * rand).max(1);
                        let max_h = (ch - 2 * rand).max(1);
                        let s = (max_w as f64 / iw as f64).min(max_h as f64 / ih as f64).min(4.0);
                        let dw = ((iw as f64 * s) as i32).max(1);
                        let dh = ((ih as f64 * s) as i32).max(1);
                        let dx = match al { 1 => cx + (cw - dw) / 2, 2 => cx + cw - dw - rand, _ => cx + rand };
                        g.draw_image_rect(cell.image(), dx, cy + (ch - dh) / 2, dw, dh);
                    }
                }
            }
            CellKind::Check => {
                let b = (ch - 8).clamp(10, 18);
                let bx = match al { 1 => cx + (cw - b) / 2, 2 => cx + cw - b - self.sk(TBL_PADDING), _ => cx + self.sk(TBL_PADDING) };
                let by = cy + (ch - b) / 2;
                g.box_fill(bx, by, bx + b - 1, by + b - 1, self.th("win_bg"));
                g.rect(bx, by, bx + b - 1, by + b - 1, border);
                if cell.value() >= 0.5 {
                    // Haken als zwei Striche -- ein gefuelltes Kaestchen
                    // liesse sich von einer eingefaerbten Zelle nicht trennen.
                    let m = b / 2;
                    g.line(bx + 3, by + m, bx + m - 1, by + b - 4, accent);
                    g.line(bx + m - 1, by + b - 4, bx + b - 3, by + 3, accent);
                }
            }
            CellKind::Bar => {
                let rand = 3;
                let bw = (cw - 2 * rand).max(1);
                let bh = (ch - 2 * rand).clamp(6, 22);
                let bx = cx + rand;
                let by = cy + (ch - bh) / 2;
                let v = cell.value().clamp(0.0, 1.0);
                g.box_fill(bx, by, bx + bw - 1, by + bh - 1, self.th("win_bg"));
                if v > 0.0 {
                    let fw = ((bw - 2) as f64 * v) as i32;
                    if fw > 0 {
                        let farbe = if cell.fg() >= 0 { cell.fg() } else { accent };
                        g.box_fill(bx + 1, by + 1, bx + fw, by + bh - 2, farbe);
                    }
                }
                g.rect(bx, by, bx + bw - 1, by + bh - 1, border);
                if !cell.text.is_empty() {
                    let tw = self.wtext_width(g, wdg, &cell.text);
                    self.wtext(g, wdg, bx + (bw - tw) / 2, by + (bh - schrift).max(0) / 2,
                               cell.text.clone(), fg_thema);
                }
            }
            CellKind::Button => {
                let rand = 3;
                let bx = cx + rand;
                let by = cy + rand;
                let bw = (cw - 2 * rand).max(1);
                let bh = (ch - 2 * rand).max(1);
                self.fbox(g, bx, by, bx + bw - 1, by + bh - 1,
                          self.th("widget_bg"), border);
                let tw = self.wtext_width(g, wdg, &cell.text);
                self.wtext(g, wdg, bx + (bw - tw) / 2, by + (bh - schrift).max(0) / 2,
                           cell.text.clone(), fgc);
            }
        }
    }
}
