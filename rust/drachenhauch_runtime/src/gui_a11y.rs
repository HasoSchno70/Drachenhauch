//! Barrierefreiheit: der Baum fuer den Bildschirmleser und seine Aktionen (a11y.rs).
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Barrierefreiheit (a11y.rs, Weg C) -------------------------------------
    /// GUI_ANNOUNCE: ein Satz fuer den Bildschirmleser des Nutzers. Er wird
    /// als Live-Knoten in den Baum gestellt; das Hilfsprogramm spricht ihn
    /// mit seiner Stimme. Ohne Hilfsprogramm passiert nichts.
    pub fn announce(&mut self, text: String, dringend: bool) {
        self.ansage = text;
        self.ansage_dringend = dringend;
        self.ansage_nr = self.ansage_nr.wrapping_add(1);
    }

    pub(super) fn a11y_rect(x: i32, y: i32, w: i32, h: i32) -> accesskit::Rect {
        accesskit::Rect::new(x as f64, y as f64, (x + w) as f64, (y + h) as f64)
    }

    /// Beschriftung fuer ein Eingabe-Widget ohne eigenen Text: die
    /// Beschriftung LINKS daneben (gleiche Zeile) oder DARUEBER (gleiche
    /// linke Kante) -- so setzt der Form-Designer sie, und so erwartet es
    /// jeder Nutzer eines Bildschirmlesers ("Name, Eingabefeld").
    pub(super) fn a11y_beschriftung(&self, wi: usize, i: usize) -> Option<usize> {
        let win = &self.windows[wi];
        let w = &win.widgets[i];
        let mut beste: Option<(i32, usize)> = None;
        for (k, l) in win.widgets.iter().enumerate() {
            if k == i || l.kind != Kind::Label || !self.widget_shown(wi, l) || l.text.is_empty() { continue; }
            let links = l.y < w.y + w.h && l.y + l.h > w.y && l.x + l.w <= w.x + 8 && w.x - (l.x + l.w) < 60;
            let oben = (l.x - w.x).abs() <= 8 && l.y + l.h <= w.y + 4 && w.y - (l.y + l.h) < 16;
            if !(links || oben) { continue; }
            let abstand = if links { w.x - (l.x + l.w) } else { w.y - (l.y + l.h) + 100 };
            if beste.map_or(true, |(d, _)| abstand < d) { beste = Some((abstand, k)); }
        }
        beste.map(|(_, k)| k)
    }

    /// Der ganze Baum fuer das Hilfsprogramm: Wurzel = das OS-Fenster,
    /// darunter ein Ansage-Knoten und jedes sichtbare gui-Fenster mit
    /// Menueleiste, Reitern und Widgets. Vollstaendig, jedes Bild -- nur
    /// gebaut, wenn jemand zuhoert (siehe vm.rs).
    pub fn a11y_baum(&self, g: &Graphics, titel: &str) -> accesskit::TreeUpdate {
        use accesskit::{Live, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate};
        use crate::a11y::ids;
        let mut nodes: Vec<(NodeId, Node)> = Vec::new();
        let mut kinder: Vec<NodeId> = Vec::new();
        // Ansage: dieselbe Ansage zweimal waere fuer den Baum keine
        // Aenderung -- ein wechselndes Leerzeichen macht sie zu einer.
        let mut an = Node::new(Role::Label);
        an.set_live(if self.ansage_dringend { Live::Assertive } else { Live::Polite });
        let mut t = self.ansage.clone();
        if self.ansage_nr % 2 == 1 { t.push(' '); }
        an.set_value(t.clone());
        an.set_label(t);
        nodes.push((NodeId(ids::ANSAGE), an));
        kinder.push(NodeId(ids::ANSAGE));
        for &wi in &self.z_order {
            if let Some(win) = self.windows.get(wi) {
                if win.alive && win.visible { kinder.push(self.a11y_fenster(g, wi, &mut nodes)); }
            }
        }
        let mut root = Node::new(Role::Window);
        root.set_label(titel.to_string());
        root.set_children(kinder);
        nodes.push((NodeId(ids::ROOT), root));
        // Fokus: das Widget mit Fokus, sonst das Fenster mit Fokus, sonst die Wurzel.
        let mut focus = NodeId(ids::ROOT);
        if let Some(fw) = self.focus_window {
            if self.windows.get(fw).map_or(false, |w| w.alive && w.visible) {
                focus = NodeId(ids::fenster(fw));
                if let Some((wi, i)) = self.focus_widget {
                    if wi == fw {
                        if let Some(w) = self.windows[wi].widgets.get(i) {
                            if self.widget_shown(wi, w) { focus = NodeId(ids::widget(wi, i)); }
                        }
                    }
                }
            }
        }
        TreeUpdate { nodes, tree: Some(TreeInfo::new(NodeId(ids::ROOT))), tree_id: TreeId::ROOT, focus }
    }

    pub(super) fn a11y_fenster(&self, g: &Graphics, wi: usize, nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>) -> accesskit::NodeId {
        use accesskit::{Node, NodeId, Role};
        use crate::a11y::ids;
        let win = &self.windows[wi];
        let modal = self.modal == Some(wi);
        let mut n = Node::new(if win.dlg || modal { Role::Dialog } else { Role::Group });
        if modal { n.set_modal(); }
        n.set_label(win.title.clone());
        n.set_bounds(Self::a11y_rect(win.x, win.y, win.w, win.h));
        let mut kinder: Vec<NodeId> = Vec::new();
        if self.menubar_h(wi) > 0 { kinder.push(self.a11y_menueleiste(g, wi, nodes)); }
        if !win.tabs.is_empty() { kinder.push(self.a11y_reiter(g, wi, nodes)); }
        for i in 0..win.widgets.len() {
            let w = &win.widgets[i];
            if !self.widget_shown(wi, w) || matches!(w.kind, Kind::Layout | Kind::Separator) { continue; }
            kinder.push(self.a11y_widget(wi, i, nodes));
        }
        n.set_children(kinder);
        let id = NodeId(ids::fenster(wi));
        nodes.push((id, n));
        id
    }

    pub(super) fn a11y_menueleiste(&self, g: &Graphics, wi: usize, nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>) -> accesskit::NodeId {
        use accesskit::{Action, Node, NodeId, Role};
        use crate::a11y::ids;
        let win = &self.windows[wi];
        let by = win.y + if win.chrome { self.m("title_h") } else { 0 };
        let mh = self.sk(MENUBAR_H);
        let mut leiste = Node::new(Role::MenuBar);
        leiste.set_bounds(Self::a11y_rect(win.x, by, win.w, mh));
        let chain = self.popup_chain(g);
        let mut kinder = Vec::new();
        for (mi, x0, x1) in self.menubar_slots(g, wi) {
            let mut m = Node::new(Role::Menu);
            m.set_label(win.menus[mi].label.clone());
            m.set_bounds(Self::a11y_rect(x0, by, x1 - x0, mh));
            m.add_action(Action::Click);
            m.add_action(Action::Expand);
            m.add_action(Action::Collapse);
            m.set_expanded(self.open_menu == Some((wi, mi)));
            m.set_children(self.a11y_eintraege(wi, mi, &chain, g, nodes, 0));
            let id = NodeId(ids::menue(wi, mi));
            nodes.push((id, m));
            kinder.push(id);
        }
        leiste.set_children(kinder);
        let id = NodeId(ids::leiste(wi));
        nodes.push((id, leiste));
        id
    }

    /// Die Eintraege eines Menues (rekursiv fuer Untermenues, begrenzt).
    /// Lage nur, wenn das Popup gerade offen ist.
    pub(super) fn a11y_eintraege(&self, wi: usize, mi: usize, chain: &[(usize, usize, i32, i32)], g: &Graphics,
                      nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>, tiefe: u32) -> Vec<accesskit::NodeId> {
        use accesskit::{Action, Node, NodeId, Role, Toggled};
        use crate::a11y::ids;
        let mut kinder = Vec::new();
        let menu = match self.windows[wi].menus.get(mi) { Some(m) => m, None => return kinder };
        let offen = chain.iter().find(|c| c.0 == wi && c.1 == mi).map(|c| (c.2, c.3));
        let (wmax, ih) = (offen.map(|_| self.popup_size(g, wi, mi).0).unwrap_or(0), self.sk(MENU_ITEM_H));
        for (ii, it) in menu.items.iter().enumerate() {
            if it.separator { continue; }
            let mut n = Node::new(if it.haken { Role::MenuItemCheckBox } else { Role::MenuItem });
            n.set_label(it.label.clone());
            if !it.kuerzel.is_empty() { n.set_keyboard_shortcut(it.kuerzel.clone()); }
            if !it.enabled { n.set_disabled(); }
            if it.haken { n.set_toggled(if it.an { Toggled::True } else { Toggled::False }); }
            n.add_action(Action::Click);
            if let Some((px, py)) = offen { n.set_bounds(Self::a11y_rect(px, py + 2 + ii as i32 * ih, wmax, ih)); }
            if it.sub >= 0 && tiefe < 6 {
                n.set_expanded(self.sub_chain.iter().any(|s| s.0 == it.sub as usize));
                n.set_children(self.a11y_eintraege(wi, it.sub as usize, chain, g, nodes, tiefe + 1));
            }
            let id = NodeId(ids::eintrag(wi, mi, ii));
            nodes.push((id, n));
            kinder.push(id);
        }
        kinder
    }

    pub(super) fn a11y_reiter(&self, g: &Graphics, wi: usize, nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>) -> accesskit::NodeId {
        use accesskit::{Action, Node, NodeId, Role};
        use crate::a11y::ids;
        let win = &self.windows[wi];
        let by = win.y + (if win.chrome { self.m("title_h") } else { 0 }) + self.menubar_h(wi);
        let mut liste = Node::new(Role::TabList);
        liste.set_bounds(Self::a11y_rect(win.x, by, win.w, TABBAR_H));
        let mut kinder = Vec::new();
        for (ti, x0, x1) in self.tab_slots(g, wi) {
            let mut t = Node::new(Role::Tab);
            t.set_label(win.tabs[ti].clone());
            t.set_bounds(Self::a11y_rect(x0, by, x1 - x0, TABBAR_H));
            t.set_selected(win.active_tab == ti as i32);
            t.add_action(Action::Click);
            let id = NodeId(ids::reiter(wi, ti));
            nodes.push((id, t));
            kinder.push(id);
        }
        liste.set_children(kinder);
        let id = NodeId(ids::reiterleiste(wi));
        nodes.push((id, liste));
        id
    }

    /// Ein Widget als Knoten, samt Unterknoten (Listeneintraege, Zellen,
    /// Baumknoten). Rolle nach Art, Beschriftung aus Text oder Nachbar-
    /// Beschriftung, Wert, Rechteck, Zustaende, erlaubte Aktionen.
    pub(super) fn a11y_widget(&self, wi: usize, i: usize, nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>) -> accesskit::NodeId {
        use accesskit::{Action, Node, NodeId, Orientation, Role, Toggled};
        use crate::a11y::ids;
        let w = &self.windows[wi].widgets[i];
        let (x, y, bw, bh) = self.abs_rect(wi, w);
        let id = NodeId(ids::widget(wi, i));
        let mut kinder: Vec<NodeId> = Vec::new();
        let mut n = match w.kind {
            Kind::Button => {
                let mut n = Node::new(if self.windows[wi].default_btn == i as i32 { Role::DefaultButton } else { Role::Button });
                n.set_label(w.text.clone());
                n.add_action(Action::Click);
                n
            }
            Kind::Card => {
                // Ein Knopf, dessen Beschreibung der Bildschirmleser nach dem
                // Titel vorliest.
                let mut n = Node::new(Role::Button);
                n.set_label(w.text.clone());
                if !w.placeholder.is_empty() { n.set_description(w.placeholder.clone()); }
                n.add_action(Action::Click);
                n
            }
            Kind::Label => { let mut n = Node::new(Role::Label); n.set_label(w.text.clone()); n.set_value(w.text.clone()); n }
            Kind::Checkbox | Kind::Toggle | Kind::Radio => {
                let mut n = Node::new(match w.kind { Kind::Checkbox => Role::CheckBox, Kind::Toggle => Role::Switch, _ => Role::RadioButton });
                n.set_label(w.text.clone());
                n.set_toggled(if w.checked { Toggled::True } else { Toggled::False });
                n.add_action(Action::Click);
                n
            }
            Kind::Slider | Kind::Knob | Kind::Spinner | Kind::Progress => {
                let mut n = Node::new(match w.kind { Kind::Spinner => Role::SpinButton, Kind::Progress => Role::ProgressIndicator, _ => Role::Slider });
                n.set_numeric_value(w.value);
                n.set_min_numeric_value(w.min);
                n.set_max_numeric_value(w.max);
                n.set_numeric_value_step(if w.kind == Kind::Spinner { w.step.max(f64::EPSILON) } else { ((w.max - w.min) / 20.0).abs().max(f64::EPSILON) });
                n.set_value(zahl_kurz(w.value));
                if w.kind == Kind::Slider && w.vert { n.set_orientation(Orientation::Vertical); }
                if w.kind != Kind::Progress {
                    n.add_action(Action::Increment);
                    n.add_action(Action::Decrement);
                    n.add_action(Action::SetValue);
                }
                n
            }
            Kind::TextInput => {
                let mut n = Node::new(if w.passwort { Role::PasswordInput } else if w.zahlen != 0 { Role::NumberInput } else { Role::TextInput });
                if !w.passwort { n.set_value(w.text.clone()); }
                if !w.placeholder.is_empty() { n.set_placeholder(w.placeholder.clone()); }
                if w.nur_lesen { n.set_read_only(); } else { n.add_action(Action::SetValue); }
                n
            }
            Kind::TextArea => {
                let mut n = Node::new(Role::MultilineTextInput);
                n.set_value(w.text.clone());
                if w.nur_lesen { n.set_read_only(); } else { n.add_action(Action::SetValue); }
                n
            }
            Kind::Dropdown => {
                let mut n = Node::new(Role::ComboBox);
                if w.frei { n.set_value(w.text.clone()); }
                else if w.sel >= 0 { if let Some(t) = w.items.get(w.sel as usize) { n.set_value(t.clone()); } }
                n.set_expanded(self.open_dropdown == Some((wi, i)));
                n.add_action(Action::Expand);
                n.add_action(Action::Collapse);
                n.add_action(Action::SetValue);
                for (k, item) in w.items.iter().enumerate().take(4000) {
                    let mut o = Node::new(Role::ListBoxOption);
                    o.set_label(item.clone());
                    o.set_selected(w.sel == k as i32);
                    o.add_action(Action::Click);
                    let oid = NodeId(ids::teil(wi, i, k));
                    nodes.push((oid, o));
                    kinder.push(oid);
                }
                n
            }
            Kind::ListBox => {
                let mut n = Node::new(Role::ListBox);
                let l = w.list.as_ref();
                if l.map_or(false, |l| l.multi) { n.set_multiselectable(); }
                let zh = self.list_zeile_h(w);
                for (r, k) in liste_ansicht(&w.items, l.map(|b| &**b)).into_iter().enumerate().take(4000) {
                    let item = &w.items[k];
                    let mut o = Node::new(Role::ListBoxOption);
                    let detail = l.and_then(|l| l.details.get(k)).filter(|d| !d.is_empty());
                    o.set_label(match detail { Some(d) => format!("{}, {}", item, d), None => item.clone() });
                    if let Some(l) = l { if !l.waehlbar(k) { o.set_disabled(); } }
                    if let Some(t) = l.and_then(|l| l.tips.get(k)).filter(|t| !t.is_empty()) { o.set_description(t.clone()); }
                    let gewaehlt = match l { Some(l) if l.multi => l.sel.get(k).copied().unwrap_or(false), _ => w.sel == k as i32 };
                    o.set_selected(gewaehlt);
                    if let Some(l) = l { if l.kaestchen { o.set_toggled(if l.checks.get(k).copied().unwrap_or(false) { Toggled::True } else { Toggled::False }); } }
                    o.set_bounds(Self::a11y_rect(x, y + r as i32 * zh - w.value as i32, bw, zh));
                    o.add_action(Action::Click);
                    let oid = NodeId(ids::teil(wi, i, k));
                    nodes.push((oid, o));
                    kinder.push(oid);
                }
                n
            }
            Kind::Table => {
                let n = Node::new(Role::Table);
                if let Some(t) = w.tbl.as_ref() {
                    // Kopfzeile
                    let mut kopf = Node::new(Role::Row);
                    let mut zellen = Vec::new();
                    for (c, h) in t.headers.iter().enumerate().take(63) {
                        let mut z = Node::new(Role::ColumnHeader);
                        z.set_label(h.clone());
                        z.set_column_index(c);
                        let zid = NodeId(ids::teil(wi, i, 64000 + c));
                        nodes.push((zid, z));
                        zellen.push(zid);
                    }
                    kopf.set_children(zellen);
                    let kid = NodeId(ids::teil(wi, i, 65000));
                    nodes.push((kid, kopf));
                    kinder.push(kid);
                    // Datenzeilen in der sichtbaren Reihenfolge
                    for (r, &dz) in t.view.iter().enumerate().take(400) {
                        let mut zeile = Node::new(Role::Row);
                        zeile.set_row_index(r + 1);
                        zeile.set_selected(t.selected == dz as i32);
                        zeile.add_action(Action::Click);
                        let mut zellen = Vec::new();
                        if let Some(row) = t.rows.get(dz) {
                            for (c, cell) in row.iter().enumerate().take(63) {
                                let mut z = Node::new(Role::Cell);
                                let text = match cell.kind {
                                    CellKind::Check => (if cell.value > 0.5 { "ja" } else { "nein" }).to_string(),
                                    CellKind::Bar => zahl_kurz(cell.value),
                                    _ => cell.text.clone(),
                                };
                                z.set_label(text.clone());
                                z.set_value(text);
                                z.set_row_index(r + 1);
                                z.set_column_index(c);
                                let zid = NodeId(ids::teil(wi, i, r * 64 + 1 + c));
                                nodes.push((zid, z));
                                zellen.push(zid);
                            }
                        }
                        zeile.set_children(zellen);
                        let zid = NodeId(ids::teil(wi, i, r * 64));
                        nodes.push((zid, zeile));
                        kinder.push(zid);
                    }
                }
                n
            }
            Kind::Tree => {
                let n = Node::new(Role::Tree);
                if let Some(t) = w.tree.as_ref() {
                    let zh = self.sk(TREE_ROW_H);
                    for (r, k) in Self::tree_visible(t).into_iter().enumerate().take(4000) {
                        let kn = &t.nodes[k];
                        let mut o = Node::new(Role::TreeItem);
                        o.set_label(kn.label.clone());
                        o.set_level(kn.level as usize + 1);
                        o.set_selected(t.selected == k as i32);
                        if t.kaestchen {
                            o.set_toggled(if t.checks.get(k).copied().unwrap_or(false) { Toggled::True } else { Toggled::False });
                        }
                        if kn.has_children {
                            o.set_expanded(kn.expanded);
                            o.add_action(if kn.expanded { Action::Collapse } else { Action::Expand });
                        }
                        o.set_bounds(Self::a11y_rect(x, y + r as i32 * zh - t.scroll, bw, zh));
                        o.add_action(Action::Click);
                        let oid = NodeId(ids::teil(wi, i, k));
                        nodes.push((oid, o));
                        kinder.push(oid);
                    }
                }
                n
            }
            Kind::ColorPicker => {
                let mut n = Node::new(Role::ColorWell);
                n.set_value(format!("#{:06X}", crate::farbraum::hsv_zu_rgb(w.hsv[0], w.hsv[1], w.hsv[2])));
                n
            }
            Kind::DatePicker => {
                let mut n = Node::new(Role::DateInput);
                n.set_value(format!("{:04}-{:02}-{:02}", w.datum[0], w.datum[1], w.datum[2]));
                n
            }
            Kind::Accordion => {
                let n = Node::new(Role::Group);
                let kopf = self.sk(AKK_KOPF_H);
                if let Some(a) = w.akk.as_ref() {
                    for (k, &(ky, _)) in a.koepfe.iter().enumerate() {
                        let mut t = Node::new(Role::Button);
                        t.set_label(w.items[k].clone());
                        t.set_expanded(a.offen.get(k).copied().unwrap_or(false));
                        t.set_bounds(Self::a11y_rect(x, y + ky - a.scroll, w.w, kopf));
                        t.add_action(Action::Click);
                        let id2 = NodeId(ids::teil(wi, i, k));
                        nodes.push((id2, t));
                        kinder.push(id2);
                    }
                }
                n
            }
            Kind::Wizard => {
                let mut n = Node::new(Role::Group);
                if let Some(t) = w.items.get(w.sel.max(0) as usize) {
                    n.set_label(format!("Schritt {} von {}: {}", w.sel + 1, w.items.len(), t));
                }
                let gm = self.wz_geom(w, x, y);
                let texte = w.wz.as_ref().map(|z| z.texte.clone()).unwrap_or_else(|| WzState::default().texte);
                let letzter = w.sel + 1 >= w.items.len() as i32;
                for (k, (r, text)) in [(gm.zurueck, texte[0].clone()),
                                       (gm.weiter, if letzter { texte[2].clone() } else { texte[1].clone() }),
                                       (gm.abbrechen, texte[3].clone())].into_iter().enumerate() {
                    let mut t = Node::new(Role::Button);
                    t.set_label(text);
                    t.set_bounds(Self::a11y_rect(r.0, r.1, r.2, r.3));
                    t.add_action(Action::Click);
                    let id2 = NodeId(ids::teil(wi, i, k));
                    nodes.push((id2, t));
                    kinder.push(id2);
                }
                n
            }
            Kind::TabControl => {
                let n = Node::new(Role::TabList);
                let kopf = self.sk(TC_KOPF_H);
                for (k, x0, x1) in self.tc_koepfe(x, w) {
                    let mut t = Node::new(Role::Tab);
                    t.set_label(w.items[k].clone());
                    t.set_selected(w.sel == k as i32);
                    t.set_bounds(Self::a11y_rect(x0, y, x1 - x0, kopf));
                    t.add_action(Action::Click);
                    let id2 = NodeId(ids::teil(wi, i, k));
                    nodes.push((id2, t));
                    kinder.push(id2);
                }
                n
            }
            Kind::TimePicker => {
                let mut n = Node::new(Role::TimeInput);
                n.set_value(format!("{:02}:{:02}:{:02}", w.zeit[0], w.zeit[1], w.zeit[2]));
                n.add_action(Action::Increment);
                n.add_action(Action::Decrement);
                n
            }
            Kind::RichText => {
                let mut n = Node::new(Role::Document);
                if let Some(r) = w.rich.as_ref() { n.set_value(r.quelle.clone()); }
                n
            }
            Kind::GroupBox | Kind::Panel => { let mut n = Node::new(Role::Group); if !w.text.is_empty() { n.set_label(w.text.clone()); } n }
            Kind::Image => Node::new(Role::Image),
            Kind::Canvas => Node::new(Role::Canvas),
            Kind::Toolbar => {
                let n = Node::new(Role::Toolbar);
                if let Some(l) = w.leiste.as_ref() {
                    for (k, x0, x1) in self.tb_geom(x, y, bw, bh, w) {
                        let e = &l.eintraege[k];
                        if e.art != 0 { continue; }
                        let mut t = Node::new(Role::Button);
                        let name = if !e.text.is_empty() { e.text.clone() } else if !e.tip.is_empty() { e.tip.clone() } else { e.symbol.clone() };
                        t.set_label(name);
                        if e.aus { t.set_disabled(); } else { t.add_action(Action::Click); }
                        if e.kippbar { t.set_toggled(if e.an { accesskit::Toggled::True } else { accesskit::Toggled::False }); }
                        t.set_bounds(Self::a11y_rect(x0, y, x1 - x0, bh));
                        let id2 = NodeId(ids::teil(wi, i, k));
                        nodes.push((id2, t));
                        kinder.push(id2);
                    }
                }
                n
            }
            Kind::Splitter => { let mut n = Node::new(Role::Splitter); n.set_orientation(if w.group == "v" { Orientation::Vertical } else { Orientation::Horizontal }); n }
            Kind::StatusBar => {
                let mut n = Node::new(Role::Status);
                if let Some(s) = w.status.as_ref() {
                    n.set_value(s.felder.iter().map(|f| f.text.as_str()).filter(|t| !t.is_empty()).collect::<Vec<_>>().join(", "));
                    for (k, x0, x1) in self.sb_geom(x, bw, w) {
                        let f = &s.felder[k];
                        if !f.klickbar { continue; }
                        let mut t = Node::new(Role::Button);
                        t.set_label(if f.tip.is_empty() { f.text.clone() } else { format!("{} ({})", f.text, f.tip) });
                        t.add_action(Action::Click);
                        t.set_bounds(Self::a11y_rect(x0, y, x1 - x0, bh));
                        let id2 = NodeId(ids::teil(wi, i, k));
                        nodes.push((id2, t));
                        kinder.push(id2);
                    }
                }
                n
            }
            Kind::Breadcrumb => {
                let n = Node::new(Role::Navigation);
                if let Some(p) = w.pfad.as_ref() {
                    // Alle Teile, auch die unter dem "..." -- ein Bildschirmleser
                    // braucht den ganzen Pfad, nicht den, der gerade hineinpasst.
                    let (geo, punkte) = self.pf_geom(x, bw, w);
                    for (k, t) in p.texte.iter().enumerate() {
                        let mut o = Node::new(Role::Link);
                        o.set_label(t.clone());
                        o.add_action(Action::Click);
                        let r = geo.iter().find(|g| g.0 == k).map(|g| (g.1, g.2)).or(punkte).unwrap_or((x, x));
                        o.set_bounds(Self::a11y_rect(r.0, y, (r.1 - r.0).max(1), bh));
                        let id2 = NodeId(ids::teil(wi, i, k));
                        nodes.push((id2, o));
                        kinder.push(id2);
                    }
                }
                n
            }
            Kind::Separator | Kind::Layout => Node::new(Role::GenericContainer),
        };
        n.set_bounds(Self::a11y_rect(x, y, bw, bh));
        if !w.enabled { n.set_disabled(); }
        if w.kind.fokussierbar() { n.add_action(Action::Focus); }
        // Beschriftung: Tooltip als Beschreibung, Fehler vor allem anderen;
        // Eingabe-Widgets ohne eigenen Text bekommen die Nachbar-Beschriftung.
        if !w.fehler.is_empty() { n.set_description(w.fehler.clone()); }
        else if !w.tooltip.is_empty() { n.set_description(w.tooltip.clone()); }
        let ohne_text = matches!(w.kind, Kind::TextInput | Kind::TextArea | Kind::Dropdown | Kind::Spinner | Kind::Slider
            | Kind::Knob | Kind::ListBox | Kind::Table | Kind::Tree | Kind::ColorPicker | Kind::DatePicker | Kind::Progress
            | Kind::Image | Kind::Canvas);
        if ohne_text {
            if let Some(k) = self.a11y_beschriftung(wi, i) {
                n.set_labelled_by(vec![NodeId(ids::widget(wi, k))]);
            } else if !w.tooltip.is_empty() {
                n.set_label(w.tooltip.clone());
            }
        }
        if !kinder.is_empty() { n.set_children(kinder); }
        nodes.push((id, n));
        id
    }

    /// Eine Aktion des Hilfsprogramms ausfuehren -- auf DERSELBEN Mechanik
    /// wie Maus und Tastatur, damit `GUI_CLICKED`, `GUI_ON_CLICK` und
    /// `on_change` genauso feuern.
    pub fn a11y_aktion(&mut self, req: accesskit::ActionRequest, g: &Graphics) {
        use accesskit::{Action, ActionData};
        use crate::a11y::ids::{self, Ziel};
        let lebt = |gui: &Gui, wi: usize| gui.windows.get(wi).map_or(false, |w| w.alive && w.visible);
        match ids::zerlegen(req.target_node.0) {
            Ziel::Fenster(wi) => {
                if lebt(self, wi) && req.action == Action::Focus { self.focus_window = Some(wi); self.bring_to_front(wi); }
            }
            Ziel::Widget(wi, i) => {
                if !lebt(self, wi) || i >= self.windows[wi].widgets.len() { return; }
                let (kind, shown, enabled) = { let w = &self.windows[wi].widgets[i]; (w.kind, self.widget_shown(wi, w), w.enabled) };
                if !shown { return; }
                match req.action {
                    Action::Focus => {
                        if kind.fokussierbar() {
                            self.focus_widget = Some((wi, i)); self.focus_window = Some(wi); self.bring_to_front(wi);
                        }
                    }
                    Action::Click if enabled => self.a11y_klick(wi, i, kind),
                    Action::Increment | Action::Decrement if enabled => {
                        let d = if req.action == Action::Increment { 1.0 } else { -1.0 };
                        let (lo, hi, v, st) = { let w = &self.windows[wi].widgets[i]; (w.min, w.max, w.value, w.step) };
                        match kind {
                            Kind::Slider | Kind::Knob => self.set_value_fire(wi, i, v + d * (hi - lo) / 20.0),
                            Kind::Spinner => self.set_value_fire(wi, i, v + d * st),
                            _ => {}
                        }
                    }
                    Action::SetValue if enabled => match (kind, req.data) {
                        (Kind::TextInput | Kind::TextArea, Some(ActionData::Value(t))) => self.a11y_text_setzen(wi, i, t.to_string(), g),
                        (Kind::Slider | Kind::Knob | Kind::Spinner, Some(ActionData::NumericValue(v))) => self.set_value_fire(wi, i, v),
                        (Kind::Slider | Kind::Knob | Kind::Spinner, Some(ActionData::Value(t))) => {
                            if let Some(v) = zahl_lesen(&t) { self.set_value_fire(wi, i, v); }
                        }
                        (Kind::Dropdown, Some(ActionData::Value(t))) => {
                            if let Some(k) = self.windows[wi].widgets[i].items.iter().position(|x| x == &*t) { self.a11y_eintrag_waehlen(wi, i, k); }
                        }
                        _ => {}
                    },
                    Action::Expand if kind == Kind::Dropdown && enabled => { self.open_dropdown = Some((wi, i)); }
                    Action::Collapse if kind == Kind::Dropdown => { if self.open_dropdown == Some((wi, i)) { self.open_dropdown = None; } }
                    _ => {}
                }
            }
            Ziel::Teil(wi, i, k) => {
                if !lebt(self, wi) || i >= self.windows[wi].widgets.len() { return; }
                let (kind, shown, enabled) = { let w = &self.windows[wi].widgets[i]; (w.kind, self.widget_shown(wi, w), w.enabled) };
                if !shown || !enabled { return; }
                match (kind, req.action) {
                    (Kind::Toolbar, Action::Click) => self.tb_klick(wi, i, k as i32),
                    (Kind::Accordion, Action::Click) => {
                        if k < self.windows[wi].widgets[i].items.len() { self.akk_umschalten(wi, i, k); }
                    }
                    (Kind::Wizard, Action::Click) => self.wz_knopf(wi, i, k),
                    (Kind::Breadcrumb, Action::Click) => self.pf_klick(wi, i, k as i32),
                    (Kind::StatusBar, Action::Click) => {
                        let w = &mut self.windows[wi].widgets[i];
                        if w.status.as_ref().and_then(|s| s.felder.get(k)).map(|f| f.klickbar).unwrap_or(false) {
                            if let Some(s) = w.status.as_mut() { s.geklickt = k as i32; }
                            w.clicked = true;
                            let f = w.on_click.clone();
                            if let Some(f) = f { self.pending.push(f); }
                        }
                    }
                    (Kind::ListBox | Kind::Dropdown, Action::Click) => self.a11y_eintrag_waehlen(wi, i, k),
                    (Kind::Tree, Action::Click) => {
                        let ok = { let w = &mut self.windows[wi].widgets[i]; match w.tree.as_mut() { Some(t) if k < t.nodes.len() && t.selected != k as i32 => { t.selected = k as i32; true } _ => false } };
                        if ok { self.focus_widget = Some((wi, i)); self.focus_window = Some(wi); let f = self.windows[wi].widgets[i].on_change.clone(); if let Some(f) = f { self.pending.push(f); } }
                    }
                    (Kind::Tree, Action::Expand | Action::Collapse) => {
                        if let Some(t) = self.windows[wi].widgets[i].tree.as_mut() {
                            if let Some(kn) = t.nodes.get_mut(k) { if kn.has_children { kn.expanded = req.action == Action::Expand; } }
                        }
                    }
                    (Kind::Table, Action::Click) => {
                        // k = Zeile * 64 (+ Spalte + 1): die Zeile waehlen.
                        let r = k / 64;
                        let ok = { let w = &mut self.windows[wi].widgets[i]; match w.tbl.as_mut() { Some(t) => match t.view.get(r).copied() { Some(dz) => { t.selected = dz as i32; true } None => false }, None => false } };
                        if ok { self.focus_widget = Some((wi, i)); self.focus_window = Some(wi); let f = self.windows[wi].widgets[i].on_change.clone(); if let Some(f) = f { self.pending.push(f); } }
                    }
                    _ => {}
                }
            }
            Ziel::Menue(wi, mi) => {
                if !lebt(self, wi) || mi >= self.windows[wi].menus.len() { return; }
                match req.action {
                    Action::Click | Action::Expand => {
                        self.open_menu = Some((wi, mi)); self.context_open = None; self.sub_chain.clear();
                        self.menu_cursor = self.menue_schritt(wi, mi, None, 1);
                    }
                    Action::Collapse => { if self.open_menu.map(|m| m.0) == Some(wi) { self.open_menu = None; self.sub_chain.clear(); self.menu_cursor = None; } }
                    _ => {}
                }
            }
            Ziel::Eintrag(wi, mi, ii) => {
                if !lebt(self, wi) || req.action != Action::Click { return; }
                let (ok, sub) = match self.windows[wi].menus.get(mi).and_then(|m| m.items.get(ii)) {
                    Some(it) => (it.enabled && !it.separator, it.sub),
                    None => (false, -1),
                };
                if !ok { return; }
                if sub >= 0 {
                    // Untermenue-Kopf: Wurzel oeffnen, Kette bis hierher -- die
                    // Lage rechnet erst das naechste Zeichnen; ohne Geometrie
                    // steht das Popup an der Wurzel.
                    if self.open_menu.map(|m| m.0) != Some(wi) { self.open_menu = Some((wi, mi)); self.sub_chain.clear(); }
                    if !self.sub_chain.iter().any(|s| s.0 == sub as usize) { self.untermenue_per_taste(g, ii); }
                } else {
                    self.open_menu = None; self.context_open = None; self.sub_chain.clear(); self.menu_cursor = None;
                    self.fire_menu_item(wi, mi, ii);
                }
            }
            Ziel::Reiter(wi, ti) => {
                if lebt(self, wi) && req.action == Action::Click && ti < self.windows[wi].tabs.len() { self.windows[wi].active_tab = ti as i32; }
            }
            Ziel::Root | Ziel::Sonst => {}
        }
    }

    /// Klick des Hilfsprogramms auf ein Widget -- dieselben Wege wie die
    /// Leertaste (`widget_keys`), damit `clicked` und die Rueckrufe stimmen.
    pub(super) fn a11y_klick(&mut self, wi: usize, i: usize, kind: Kind) {
        match kind {
            Kind::Button | Kind::Card => {
                let w = &mut self.windows[wi].widgets[i];
                w.clicked = true;
                let f = w.on_click.clone();
                if let Some(f) = f { self.pending.push(f); }
            }
            Kind::Checkbox | Kind::Toggle => {
                let w = &mut self.windows[wi].widgets[i];
                w.checked = !w.checked;
                w.clicked = true;
                let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                if let Some(f) = oc { self.pending.push(f); }
                if let Some(f) = och { self.pending.push(f); }
            }
            Kind::Radio => {
                if !self.windows[wi].widgets[i].checked {
                    self.select_radio(wi, i);
                    self.windows[wi].widgets[i].clicked = true;
                    let w = &self.windows[wi].widgets[i];
                    let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                    if let Some(f) = oc { self.pending.push(f); }
                    if let Some(f) = och { self.pending.push(f); }
                }
            }
            Kind::Dropdown => {
                self.open_dropdown = if self.open_dropdown == Some((wi, i)) { None } else { Some((wi, i)) };
            }
            _ => {
                if kind.fokussierbar() { self.focus_widget = Some((wi, i)); self.focus_window = Some(wi); self.bring_to_front(wi); }
            }
        }
    }

    /// Eintrag `k` einer Liste/Klappliste waehlen (wie ein Klick darauf).
    pub(super) fn a11y_eintrag_waehlen(&mut self, wi: usize, i: usize, k: usize) {
        let n = self.windows[wi].widgets[i].items.len();
        if k >= n { return; }
        if self.windows[wi].widgets[i].list.as_ref().map(|l| !l.waehlbar(k)).unwrap_or(false) { return; }
        let w = &mut self.windows[wi].widgets[i];
        let vorher = w.sel;
        w.sel = k as i32;
        if let Some(l) = w.list.as_mut() {
            if l.multi { l.sync(n); for j in 0..n { l.sel[j] = j == k; } l.anker = k as i32; }
        }
        if w.kind == Kind::Dropdown { self.open_dropdown = None; }
        self.focus_widget = Some((wi, i)); self.focus_window = Some(wi);
        if vorher != k as i32 {
            let f = self.windows[wi].widgets[i].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Text aus dem Hilfsprogramm in ein Textfeld schreiben -- durch
    /// dieselben Grenzen wie beim Tippen (Hoechstlaenge, Zahlenfilter).
    pub(super) fn a11y_text_setzen(&mut self, wi: usize, i: usize, mut neu: String, g: &Graphics) {
        let jetzt = g.get_time();
        let w = &mut self.windows[wi].widgets[i];
        if w.nur_lesen { return; }
        if w.kind == Kind::TextInput {
            if w.maxlaenge > 0 { neu = neu.chars().take(w.maxlaenge as usize).collect(); }
            if !zahl_erlaubt(&neu, w.zahlen) { return; }
        }
        if neu == w.text { return; }
        let before = w.text.clone();
        let caret0 = w.caret;
        Self::undo_merken(w, &before, caret0, jetzt);
        let n = neu.chars().count() as i32;
        w.text = neu;
        w.spans.clear();
        w.wellen.clear();
        w.caret = n; w.sel_anchor = n; w.scroll = 0;
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }
}
