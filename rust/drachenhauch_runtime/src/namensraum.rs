//! WP I.1 -- Namensraeume fuer importierte Quelldateien.
//!
//! `IMPORT "mathe.dh" AS mathe` macht aus den Top-Level-Namen der Datei einen
//! Namensraum: von aussen heissen sie `mathe.Distanz`, innerhalb der Datei
//! weiterhin schlicht `Distanz`.
//!
//! UMGESETZT WIRD DAS DURCH UMBENENNEN ZUR UEBERSETZUNGSZEIT, nicht durch
//! getrenntes Parsen. `IMPORT` bleibt ein Textmerge; dieser Durchgang laeuft
//! danach ueber das geparste AST und haengt den Namen ein Praefix an:
//!
//! ```text
//! mathe.dh:  FUNCTION Distanz(...)   ->  intern: mathe@distanz
//! Aufrufer:  mathe.Distanz(...)      ->  intern: mathe@distanz
//! mathe.dh:  quadrat(x)              ->  intern: mathe@quadrat
//! ```
//!
//! Der Preis dafuer ist ein Praefix in internen Namen; der Gewinn ist, dass
//! VM, Bytecode-Format, `.dhc`-Export, Zeilennummern, Debugger und Profiler
//! **unberuehrt** bleiben. Kein neuer Opcode.
//!
//! Welche gemergte Zeile zu welcher Datei gehoert, sagt die Herkunftstabelle
//! aus `preprocess` (kam mit WP I.4). Welche Datei einen Alias hat, sagt der
//! vierte Rueckgabewert von `preprocess::process`.

use crate::ast::{Node, Param};
use crate::preprocess::Herkunft;
use std::collections::{HashMap, HashSet};

/// Trennzeichen zwischen Alias und Name im internen Namen.
///
/// `@` kommt in keinem IDENT vor, den der Lexer liefert -- und das genuegt,
/// weil die Umbenennung NACH dem Parsen passiert: der Name geht nie wieder
/// durch den Lexer. Ein Nutzer kann den internen Namen also nicht von Hand
/// hinschreiben, auch nicht versehentlich.
pub const TRENNER: char = '@';

/// Jeden internen Namen in einem MELDUNGSTEXT lesbar machen.
///
/// Der Compiler baut seine Meldungen aus internen Namen (`mathe@quadrat: zu
/// viele Argumente`). Der Nutzer hat diesen Namen nie geschrieben und findet
/// ihn in seiner Datei auch nicht wieder -- also wird jedes `alias@name` beim
/// AUSGEBEN zu `alias.name`. Nur an dieser einen Stelle, damit die interne
/// Eindeutigkeit erhalten bleibt.
///
/// Ersetzt wird nur, was wie ein Name aussieht (Buchstabe/Unterstrich davor
/// und dahinter). Ein `@` in einer Zeichenkette des Nutzers bleibt damit in
/// aller Regel unberuehrt.
pub fn lesbar_text(text: &str) -> String {
    if !text.contains(TRENNER) {
        return text.to_string();
    }
    let b: Vec<char> = text.chars().collect();
    let namensteil = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut raus = String::with_capacity(text.len());
    for (i, c) in b.iter().enumerate() {
        let davor = i > 0 && namensteil(b[i - 1]);
        let danach = b.get(i + 1).map(|x| namensteil(*x)).unwrap_or(false);
        if *c == TRENNER && davor && danach {
            raus.push('.');
        } else {
            raus.push(*c);
        }
    }
    raus
}

fn intern(alias: &str, name: &str) -> String {
    format!("{}{}{}", alias, TRENNER, name.to_lowercase())
}

/// Welche Datei liefert die gemergte Zeile `zeile` (1-basiert)?
fn datei_von(herkunft: &[Herkunft], zeile: u32) -> &str {
    herkunft
        .get(zeile.saturating_sub(1) as usize)
        .map(|h| h.datei.as_str())
        .unwrap_or("")
}

/// Was ein Namensraum nach aussen anbietet, plus was er nur intern hat.
#[derive(Default)]
struct Modul {
    /// Top-Level-Namen (lowercase), die umbenannt werden.
    namen: HashSet<String>,
    /// Untermenge davon: mit `PRIVATE` markiert, also von aussen nicht
    /// erreichbar. Umbenannt wird trotzdem -- privat heisst unsichtbar, nicht
    /// unbenannt.
    privat: HashSet<String>,
    /// Klassen und Structs. Seit I.2 namensraumfaehig: `DIM p AS mathe.Punkt`
    /// und `NEW mathe.Punkt()`.
    klassen: HashSet<String>,
    /// ENUMs. Noch nicht namensraumfaehig (I.3) -- sie stehen hier, damit
    /// `mathe.Farbe` eine praezise Meldung bekommt statt "kennt keinen Namen".
    enums: HashSet<String>,
}

/// Alle Top-Level-Deklarationen je Alias einsammeln.
fn sammeln(
    prog: &Node,
    herkunft: &[Herkunft],
    datei_alias: &HashMap<String, String>,
    privat: &[(u32, String)],
) -> (HashMap<String, Modul>, HashSet<String>) {
    let mut module: HashMap<String, Modul> = HashMap::new();
    // Top-Level-Namen der HAUPTDATEI (datei == ""). Ein Namensraum darf sie
    // nicht sehen -- das ist der eigentliche Gewinn von `AS` und die einzige
    // Stelle, an der sich durch das Anfuegen von `AS` Verhalten aendert.
    let mut haupt: HashSet<String> = HashSet::new();
    let stmts = match prog {
        Node::Program { statements } => statements,
        _ => return (module, haupt),
    };
    for s in stmts {
        let (zeile, body) = match s {
            Node::Stmt { line, body } => (*line, body.as_ref()),
            other => (0, other),
        };
        let datei = datei_von(herkunft, zeile);
        let alias = match datei_alias.get(datei) {
            Some(a) => a.clone(),
            None => {
                if datei.is_empty() {
                    match body {
                        Node::FunctionDecl { name, .. } | Node::SubDecl { name, .. }
                        | Node::Dim { name, .. } | Node::Const { name, .. } => {
                            haupt.insert(name.to_lowercase());
                        }
                        Node::MultiDim { dims } => {
                            for d in dims {
                                if let Node::Dim { name, .. } = d {
                                    haupt.insert(name.to_lowercase());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                continue;
            }
        };
        let eintrag = module.entry(alias).or_default();
        match body {
            Node::FunctionDecl { name, .. } | Node::SubDecl { name, .. } => {
                eintrag.namen.insert(name.to_lowercase());
            }
            Node::Dim { name, .. } | Node::Const { name, .. } => {
                eintrag.namen.insert(name.to_lowercase());
            }
            Node::MultiDim { dims } => {
                for d in dims {
                    if let Node::Dim { name, .. } = d {
                        eintrag.namen.insert(name.to_lowercase());
                    }
                }
            }
            Node::ClassDecl { name, .. } => { eintrag.klassen.insert(name.to_lowercase()); }
            Node::EnumDecl { name, .. } => { eintrag.enums.insert(name.to_lowercase()); }
            _ => {}
        }
    }
    // PRIVATE-Namen nachtragen. Sie werden trotzdem umbenannt -- privat heisst
    // unsichtbar von aussen, nicht unbenannt.
    for (zeile, name) in privat {
        if let Some(alias) = datei_alias.get(datei_von(herkunft, *zeile)) {
            if let Some(m) = module.get_mut(alias) {
                m.privat.insert(name.clone());
            }
        }
    }
    (module, haupt)
}

/// Lokale Namen einer Funktion: Parameter und alles, was ihr Rumpf deklariert.
///
/// Ohne das wuerde ein lokales `DIM zaehler` in einer Modul-Funktion mit einem
/// gleichnamigen Top-Level-`DIM` verschmelzen -- zwei Variablen, ein Name, und
/// das Programm rechnet still falsch.
fn lokale(params: &[Param], body: &[Node], raus: &mut HashSet<String>) {
    for p in params {
        raus.insert(p.name.to_lowercase());
    }
    fn geh(ns: &[Node], raus: &mut HashSet<String>) {
        for n in ns {
            let b = match n {
                Node::Stmt { body, .. } => body.as_ref(),
                other => other,
            };
            match b {
                Node::Dim { name, .. } | Node::Const { name, .. } => {
                    raus.insert(name.to_lowercase());
                }
                Node::MultiDim { dims } => geh(dims, raus),
                Node::For { var, body, .. } | Node::ForEach { var, body, .. } => {
                    raus.insert(var.to_lowercase());
                    geh(body, raus);
                }
                Node::If { then_block, elseif_branches, else_block, .. } => {
                    geh(then_block, raus);
                    for (_, blk) in elseif_branches { geh(blk, raus); }
                    geh(else_block, raus);
                }
                Node::While { body, .. } | Node::Repeat { body, .. }
                | Node::With { body, .. } => geh(body, raus),
                Node::Try { body, catch_block, finally_block, catch_var, .. } => {
                    if !catch_var.is_empty() { raus.insert(catch_var.to_lowercase()); }
                    geh(body, raus);
                    geh(catch_block, raus);
                    geh(finally_block, raus);
                }
                Node::Select { cases, else_block, .. } => {
                    for (_, _, blk) in cases { geh(blk, raus); }
                    geh(else_block, raus);
                }
                _ => {}
            }
        }
    }
    geh(body, raus);
}

/// Zustand waehrend des Umbenennens.
struct Lauf<'a> {
    herkunft: &'a [Herkunft],
    datei_alias: &'a HashMap<String, String>,
    module: &'a HashMap<String, Modul>,
    haupt: &'a HashSet<String>,
    /// Namensraum der gerade bearbeiteten Zeile (None = Hauptprogramm oder
    /// eine importierte Datei ohne `AS`).
    hier: Option<String>,
    /// Lokale Namen, die in diesem Rumpf NICHT umbenannt werden duerfen.
    lokal: HashSet<String>,
    fehler: Option<(u32, String)>,
    /// Zeile fuer die naechste Fehlermeldung.
    zeile: u32,
}

impl<'a> Lauf<'a> {
    /// Soll dieser Name im aktuellen Modul umbenannt werden?
    fn eigen(&self, name: &str) -> Option<String> {
        let alias = self.hier.as_ref()?;
        let low = name.to_lowercase();
        if self.lokal.contains(&low) {
            return None;
        }
        let m = self.module.get(alias)?;
        if m.namen.contains(&low) { Some(intern(alias, &low)) } else { None }
    }

    /// Einen TYPNAMEN aufloesen (WP I.2).
    ///
    /// Drei Faelle, und `array:`/`map:` koennen sie beliebig tief schachteln
    /// (`ARRAY OF mathe.Punkt` kommt als `array:mathe.Punkt` herein):
    ///   - `alias.Klasse` von aussen  -> interner Name,
    ///   - eigene Klasse im Modul     -> interner Name,
    ///   - alles andere               -> unveraendert (INTEGER, STRING, ...).
    fn typ(&mut self, name: &mut String) {
        for praefix in ["array:", "map:"] {
            if let Some(rest) = name.strip_prefix(praefix) {
                let mut inner = rest.to_string();
                self.typ(&mut inner);
                *name = format!("{}{}", praefix, inner);
                return;
            }
        }
        if let Some((alias_teil, typ_teil)) = name.split_once('.') {
            let alias = alias_teil.to_lowercase();
            let low = typ_teil.to_lowercase();
            if !self.datei_alias.values().any(|a| *a == alias) {
                return;             // kein Namensraum -- der Compiler meldet es
            }
            match self.module.get(&alias) {
                Some(m) if m.klassen.contains(&low) => *name = intern(&alias, &low),
                Some(m) if m.enums.contains(&low) => {
                    self.fehler.get_or_insert((self.zeile, format!(
                        "{}.{} ist ein ENUM -- ENUMs lassen sich noch nicht als Typ \
                         ueber den Namensraum benennen (WP I.3).", alias_teil, typ_teil)));
                }
                _ => {
                    self.fehler.get_or_insert((self.zeile, format!(
                        "{} kennt keine Klasse {}.", alias_teil, typ_teil)));
                }
            }
            return;
        }
        // Unqualifiziert INNERHALB des Moduls: die eigene Klasse mitziehen.
        if let Some(alias) = self.hier.clone() {
            let low = name.to_lowercase();
            if self.module.get(&alias).map(|m| m.klassen.contains(&low)).unwrap_or(false) {
                *name = intern(&alias, &low);
            }
        }
    }

    fn namen(&mut self, name: &mut String) {
        if let Some(neu) = self.eigen(name) {
            *name = neu;
            return;
        }
        // Entscheidung 3 aus dem Entwurf: ein Namensraum sieht die Globals des
        // Hauptprogramms NICHT. Ohne diese Pruefung wuerde der Name still auf
        // das Global des Aufrufers zeigen -- die Datei liefe je nach
        // Hauptprogramm anders, und niemand saehe warum.
        if self.hier.is_some() {
            let low = name.to_lowercase();
            if !self.lokal.contains(&low) && self.haupt.contains(&low) {
                self.fehler.get_or_insert((self.zeile, format!(
                    concat!("{} kommt aus dem Hauptprogramm. Eine mit AS importierte Datei ",
                            "sieht dessen Globals nicht -- reiche den Wert als Parameter ",
                            "herein oder deklariere ihn in der Datei selbst."),
                    name)));
            }
        }
    }

    /// `alias.name` von aussen aufloesen. Liefert den internen Namen oder
    /// setzt einen Fehler.
    fn qualifiziert(&mut self, alias_kandidat: &str, name: &str) -> Option<String> {
        let alias = alias_kandidat.to_lowercase();
        if !self.datei_alias.values().any(|a| *a == alias) {
            return None;
        }
        let low = name.to_lowercase();
        let m = self.module.get(&alias)?;
        if m.klassen.contains(&low) {
            // In AUSDRUCKS-Position ist ein blosser Klassenname trotzdem nichts
            // Sinnvolles -- gemeint ist fast immer `NEW alias.Klasse()`.
            self.fehler.get_or_insert((self.zeile, format!(
                "{}.{} ist eine Klasse. Zum Erzeugen `NEW {}.{}(...)`, als Typ \
                 `DIM x AS {}.{}`.", alias, name, alias, name, alias, name)));
            return None;
        }
        if m.enums.contains(&low) {
            self.fehler.get_or_insert((self.zeile, format!(
                concat!("{}.{} ist eine Klasse oder ein ENUM -- die lassen sich noch ",
                            "nicht ueber den Namensraum BENENNEN (WP I.2). Eine Funktion ",
                            "der Datei, die so einen Wert LIEFERT, geht aber schon: ",
                            "`{}.ErzeugePunkt(...)` und danach `.feld` darauf. Wer den ",
                            "Typ wirklich selbst hinschreiben muss, importiert die Datei ",
                            "zusaetzlich ohne AS -- dann steht er flach zur Verfuegung."),
                    alias, name, alias)));
            return None;
        }
        if !m.namen.contains(&low) {
            self.fehler.get_or_insert((self.zeile, format!(
                "{} kennt keinen Namen {}.", alias, name)));
            return None;
        }
        if m.privat.contains(&low) {
            self.fehler.get_or_insert((self.zeile, format!(
                "{}.{} ist PRIVATE und nur innerhalb der eigenen Datei verwendbar.",
                alias, name)));
            return None;
        }
        Some(intern(&alias, &low))
    }
}

/// Rumpf einer Modul-Funktion bearbeiten: erst die lokalen Namen ermitteln,
/// dann umbenennen, danach den alten Stand zurueckgeben.
fn mit_lokalen(l: &mut Lauf, params: &[Param], body: &mut Vec<Node>) {
    let vorher = std::mem::take(&mut l.lokal);
    let mut lok = vorher.clone();
    lokale(params, body, &mut lok);
    l.lokal = lok;
    for n in body.iter_mut() {
        knoten(l, n);
    }
    l.lokal = vorher;
}

fn knoten(l: &mut Lauf, n: &mut Node) {
    use Node::*;
    match n {
        Stmt { line, body } => {
            let vorher = l.hier.clone();
            let vorher_zeile = l.zeile;
            l.zeile = *line;
            l.hier = l.datei_alias.get(datei_von(l.herkunft, *line)).cloned();
            knoten(l, body);
            l.hier = vorher;
            l.zeile = vorher_zeile;
        }

        // --- die Stelle, an der `alias.name` verschwindet -----------------
        MemberAccess { target, name } => {
            let treffer = match target.as_ref() {
                Identifier(t) => l.qualifiziert(&t.clone(), name),
                _ => None,
            };
            match treffer {
                Some(neu) => *n = Identifier(neu),
                None => knoten(l, target),
            }
        }
        MemberAssign { target, name, value } => {
            knoten(l, value);
            let treffer = match target.as_ref() {
                Identifier(t) => l.qualifiziert(&t.clone(), name),
                _ => None,
            };
            match treffer {
                Some(neu) => {
                    let wert = std::mem::replace(value.as_mut(), NilLit);
                    *n = Assign { name: neu, value: Box::new(wert) };
                }
                None => knoten(l, target),
            }
        }

        // --- Namen, die zum Modul gehoeren koennen ------------------------
        Identifier(name) => l.namen(name),
        Assign { name, value } => { l.namen(name); knoten(l, value); }
        Dim { name, type_name, array_dims } => {
            l.namen(name);
            l.typ(type_name);
            if let Some(ds) = array_dims { for d in ds { knoten(l, d); } }
        }
        Const { name, type_name, value } => {
            l.namen(name);
            if let Some(t) = type_name { l.typ(t); }
            knoten(l, value);
        }
        MultiDim { dims } => { for d in dims { knoten(l, d); } }

        FunctionDecl { name, params, return_type, body } => {
            l.namen(name);
            l.typ(return_type);
            for p in params.iter_mut() { l.typ(&mut p.type_name); }
            let ps = params.clone();
            mit_lokalen(l, &ps, body);
        }
        SubDecl { name, params, body } => {
            l.namen(name);
            for p in params.iter_mut() { l.typ(&mut p.type_name); }
            let ps = params.clone();
            mit_lokalen(l, &ps, body);
        }

        // --- alles Uebrige: nur durchreichen ------------------------------
        Program { statements } => { for s in statements { knoten(l, s); } }
        BinaryOp { left, right, .. } => { knoten(l, left); knoten(l, right); }
        UnaryOp { operand, .. } => knoten(l, operand),
        TernaryExpr { cond, then_expr, else_expr } => {
            knoten(l, cond); knoten(l, then_expr); knoten(l, else_expr);
        }
        Yield(Some(v)) => knoten(l, v),
        Call { callee, args } => { knoten(l, callee); for a in args { knoten(l, a); } }
        ListComp { iterable, filter, transform, .. } => {
            knoten(l, iterable);
            if let Some(f) = filter { knoten(l, f); }
            knoten(l, transform);
        }
        DictComp { iterable, filter, key, value, .. } => {
            knoten(l, iterable);
            if let Some(f) = filter { knoten(l, f); }
            knoten(l, key); knoten(l, value);
        }
        SetComp { iterable, filter, transform, .. } => {
            knoten(l, iterable);
            if let Some(f) = filter { knoten(l, f); }
            knoten(l, transform);
        }
        ArrayLit(items) => { for i in items { knoten(l, i); } }
        TupleLit { elements } => { for e in elements { knoten(l, e); } }
        NamedArg { value, .. } => knoten(l, value),
        New { class_name, args } => {
            l.typ(class_name);
            if let Some(args) = args { for a in args { knoten(l, a); } }
        }
        IndexAccess { target, indices } => {
            knoten(l, target); for i in indices { knoten(l, i); }
        }
        SliceAccess { target, lo, hi } => {
            knoten(l, target);
            if let Some(x) = lo { knoten(l, x); }
            if let Some(x) = hi { knoten(l, x); }
        }
        IndexAssign { target, indices, value } => {
            knoten(l, target);
            for i in indices { knoten(l, i); }
            knoten(l, value);
        }
        TupleAssign { targets, value } => {
            for t in targets { knoten(l, t); }
            knoten(l, value);
        }
        Try { body, catch_block, finally_block, .. } => {
            for x in body.iter_mut() { knoten(l, x); }
            for x in catch_block.iter_mut() { knoten(l, x); }
            for x in finally_block.iter_mut() { knoten(l, x); }
        }
        Throw { value, code } => {
            knoten(l, value);
            if let Some(c) = code { knoten(l, c); }
        }
        Print { items, .. } => { for i in items { knoten(l, i); } }
        Input { prompt, target } => {
            if let Some(p) = prompt { knoten(l, p); }
            l.namen(target);
        }
        If { condition, then_block, elseif_branches, else_block } => {
            knoten(l, condition);
            for x in then_block.iter_mut() { knoten(l, x); }
            for (c, blk) in elseif_branches.iter_mut() {
                knoten(l, c);
                for x in blk.iter_mut() { knoten(l, x); }
            }
            for x in else_block.iter_mut() { knoten(l, x); }
        }
        Select { subject, cases, else_block } => {
            knoten(l, subject);
            for (_, guard, blk) in cases.iter_mut() {
                if let Some(g) = guard { knoten(l, g); }
                for x in blk.iter_mut() { knoten(l, x); }
            }
            for x in else_block.iter_mut() { knoten(l, x); }
        }
        While { condition, body } => {
            knoten(l, condition);
            for x in body.iter_mut() { knoten(l, x); }
        }
        For { var, start, end, step, body } => {
            l.namen(var);
            knoten(l, start); knoten(l, end);
            if let Some(s) = step { knoten(l, s); }
            for x in body.iter_mut() { knoten(l, x); }
        }
        ForEach { var, iterable, body } => {
            l.namen(var);
            knoten(l, iterable);
            for x in body.iter_mut() { knoten(l, x); }
        }
        Repeat { body, condition } => {
            for x in body.iter_mut() { knoten(l, x); }
            knoten(l, condition);
        }
        Data { values } => { for v in values { knoten(l, v); } }
        Read { targets } => { for t in targets { knoten(l, t); } }
        ExprStmt { expr } => knoten(l, expr),
        Return(Some(v)) => knoten(l, v),
        PropertyDecl { func, .. } => knoten(l, func),
        With { target, body, .. } => {
            knoten(l, target);
            for x in body.iter_mut() { knoten(l, x); }
        }
        // Klassen und ENUMs bleiben in I.1 flach: ihre Ruempfe werden nicht
        // angefasst, damit Felder und Methoden sich nicht mit Modulnamen
        // vermischen. Ein Zugriff `alias.Klasse` bekommt in `qualifiziert`
        // eine eigene Meldung.
        ClassDecl { name, parent, fields, methods, properties, .. } => {
            if let Some(alias) = l.hier.clone() {
                let low = name.to_lowercase();
                if l.module.get(&alias).map(|m| m.klassen.contains(&low)).unwrap_or(false) {
                    *name = intern(&alias, &low);
                }
            }
            if let Some(pa) = parent { l.typ(pa); }
            // Felder: nur die Typen, nicht die Feldnamen -- die gehoeren der
            // Klasse und haben mit dem Modul-Namensraum nichts zu tun.
            for f in fields.iter_mut() {
                if let Dim { type_name, .. } = f { l.typ(type_name); }
                if let Stmt { body, .. } = f {
                    if let Dim { type_name, .. } = body.as_mut() { l.typ(type_name); }
                }
            }
            for m in methods.iter_mut() { knoten(l, m); }
            for pr in properties.iter_mut() { knoten(l, pr); }
        }
        EnumDecl { .. } => {}
        _ => {}
    }
}

/// Namensraeume auf das geparste Programm anwenden.
///
/// `namensraeume` sind die `(datei, alias)`-Paare aus `preprocess::process`.
/// Ohne ein einziges `IMPORT ... AS` an einer Quelldatei passiert hier nichts
/// (fruehes `return`) -- bestehende Programme nehmen also denselben Weg wie
/// bisher, ohne einen einzigen zusaetzlichen Knotenbesuch.
pub fn anwenden(
    prog: &mut Node,
    herkunft: &[Herkunft],
    namensraeume: &[(String, String)],
    privat: &[(u32, String)],
) -> Result<(), (u32, String)> {
    if namensraeume.is_empty() {
        return Ok(());
    }
    let datei_alias: HashMap<String, String> = namensraeume.iter().cloned().collect();
    let (module, haupt) = sammeln(prog, herkunft, &datei_alias, privat);
    let mut l = Lauf {
        herkunft,
        datei_alias: &datei_alias,
        module: &module,
        haupt: &haupt,
        hier: None,
        lokal: HashSet::new(),
        fehler: None,
        zeile: 0,
    };
    knoten(&mut l, prog);
    match l.fehler {
        Some(f) => Err(f),
        None => Ok(()),
    }
}
