//! Der Debugger der VM (`dhrt debug`): Haltepunkte, Schritte, Variablen als JSON, Ausdruecke auswerten.
//!
//! Teil von `vm.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl<'p> Vm<'p> {
    /// Debugging fuer den naechsten `run()` aktivieren (`dhrt debug`).
    /// `karte`: gemergte Zeilen <-> (Datei, Zeile), siehe debugger.rs.
    pub fn enable_debug(&mut self, karte: Option<crate::debugger::Karte>, klassen: HashMap<String, String>,
                        namen: HashMap<String, String>) {
        let mut d = DebugState::new();
        d.karte = karte;
        self.dbg = Some(d);
        self.dbg_klassen = klassen;
        self.dbg_namen = namen;
    }

    /// Ein Name, wie er im Quelltext steht (die VM kennt ihn nur klein).
    pub(super) fn dbg_name(&self, n: &str) -> String {
        self.dbg_namen.get(&n.to_lowercase()).cloned().unwrap_or_else(|| n.to_string())
    }

    /// Wie ein Wert im Debugger dasteht: (Typ, Wert). Eine Instanz traegt
    /// den Klassennamen, wie er geschrieben wurde, statt `OBJECT` / `<held>`.
    pub(super) fn dbg_anzeige(&self, v: &Value) -> (String, String) {
        if let Value::Instance(rc) = v {
            let cn = rc.borrow().class_name.to_string();
            let name = self.dbg_klassen.get(&cn.to_lowercase()).cloned().unwrap_or(cn);
            return (name.clone(), format!("<{}>", name));
        }
        (v.type_name().to_string(), dbg_short(v))
    }

    /// Die Kinder eines zusammengesetzten Werts fuer die Variablenliste:
    /// Felder einer Instanz, Elemente eines Feldes oder Tupels, Eintraege
    /// einer MAP. Begrenzt (100 je Ebene, drei Ebenen tief, `budget` Knoten
    /// je Variable), damit ein Halt mit einem Feld von 100 000 Zahlen nicht
    /// Megabytes JSON schickt. None = keine Kinder.
    pub(super) fn dbg_kinder(&self, v: &Value, tiefe: u32, budget: &mut usize) -> Option<serde_json::Value> {
        const JE_EBENE: usize = 100;
        if tiefe >= 3 || *budget == 0 { return None; }
        let mut paare: Vec<(String, Value)> = Vec::new();
        let mut gesamt = 0usize;
        match v {
            Value::Instance(rc) => {
                let inst = rc.borrow();
                let mut namen: Vec<(&String, &u32)> = inst.layout.index.iter().collect();
                namen.sort_by_key(|(_, &k)| k);
                gesamt = namen.len();
                for (n, &k) in namen.into_iter().take(JE_EBENE) {
                    if let Some(w) = inst.fields.get(k as usize) { paare.push((n.clone(), w.clone())); }
                }
            }
            Value::Array(rc) => {
                let a = rc.borrow();
                gesamt = a.cells.len();
                for i in 0..gesamt.min(JE_EBENE) {
                    // Die Indizes so, wie das Programm sie schreibt -- bei
                    // `DIM tage[1 TO 31]` also ab [1].
                    let name = if a.dims.len() <= 1 { format!("[{}]", i as i64 + a.unten_von(0)) } else {
                        let mut rest = i as i64;
                        let teile: Vec<String> = a.strides.iter().enumerate().map(|(k, &s)| {
                            let q = rest / s.max(1); rest %= s.max(1); (q + a.unten_von(k)).to_string()
                        }).collect();
                        format!("[{}]", teile.join(", "))
                    };
                    paare.push((name, a.cells.get(i)));
                }
            }
            Value::Tuple(t) => {
                gesamt = t.len();
                for (i, w) in t.iter().take(JE_EBENE).enumerate() { paare.push((format!("[{}]", i), w.clone())); }
            }
            Value::Map(rc) => {
                let m = rc.borrow();
                gesamt = m.len();
                for (k, w) in m.entries().iter().take(JE_EBENE) { paare.push((format!("\"{}\"", k), w.clone())); }
            }
            _ => return None,
        }
        let mut out = Vec::new();
        for (name, w) in paare {
            if *budget == 0 { break; }
            *budget -= 1;
            let (typ, wert) = self.dbg_anzeige(&w);
            // Feldnamen einer Instanz wie geschrieben; `[3]` und MAP-Schluessel
            // bleiben, wie sie sind (keiner davon steht klein im Programm).
            let name = if matches!(v, Value::Instance(_)) { self.dbg_name(&name) } else { name };
            let mut e = serde_json::json!({"name": name, "type": typ, "value": wert});
            if let Some(k) = self.dbg_kinder(&w, tiefe + 1, budget) { e["children"] = k; }
            out.push(e);
        }
        if gesamt > out.len() {
            out.push(serde_json::json!({"name": "...", "type": "", "value": format!("{} weitere", gesamt - out.len())}));
        }
        if out.is_empty() { None } else { Some(serde_json::Value::Array(out)) }
    }

    /// Ein Eintrag der Variablenliste samt Kindern.
    /// `decl` = der angesagte Typ. Ein Handle (GUI_WIDGET, DB_CONN, IMAGE ...)
    /// ist zur Laufzeit eine Ganzzahl -- angezeigt wird, was im DIM steht,
    /// statt eines INTEGER, das niemand geschrieben hat (bei NIL ebenso).
    pub(super) fn dbg_var_json(&self, name: &str, v: &Value, decl: Option<&str>) -> serde_json::Value {
        let (mut typ, wert) = self.dbg_anzeige(v);
        // Auch bei NIL: ein noch nicht belegtes `DIM w AS GUI_WIDGET` oder
        // `DIM p AS Held` sagt mehr mit dem Typ, der dasteht.
        if let (Value::Int(_) | Value::Nil, Some(d)) = (v, decl) {
            let d = d.trim().to_lowercase();
            if !d.is_empty() && !matches!(d.as_str(), "any" | "int" | "integer" | "num" | "float")
                && d.chars().all(|c| c.is_alphanumeric() || c == '_') {
                typ = self.dbg_klassen.get(&d).cloned().unwrap_or_else(|| d.to_uppercase());
            }
        }
        let mut e = serde_json::json!({"name": self.dbg_name(name), "type": typ, "value": wert});
        let mut budget = 400usize;
        if let Some(k) = self.dbg_kinder(v, 0, &mut budget) { e["children"] = k; }
        e
    }

    /// Kommt ein Fehler oben an, haelt der Debugger an seiner Stelle an:
    /// Variablen, Stapel und Ueberwachen so, wie sie beim Fehler waren, und
    /// `eval` geht noch -- `continue`/Schritt/`stop` beenden. Ohne Stand
    /// (Fehler vor der ersten Zeile, oder kein Debugger) nichts.
    pub fn debug_fehler_halt(&mut self, meldung: &str) {
        let mut dbg = match self.dbg.take() { Some(d) => d, None => return };
        let stand = match dbg.fehler_stand.take() {
            Some(s) => s,
            None => { self.dbg = Some(dbg); return; }
        };
        if self.out.len() > dbg.out_sent {
            let chunk = self.out[dbg.out_sent..].to_string();
            dbg.out_sent = self.out.len();
            dbg_emit(&serde_json::json!({"event":"output","text":chunk}));
        }
        // SAFETY: zeigt in `prog`, das der Aufrufer (debug_main) haelt.
        let fn_: &Func = unsafe { &*stand.fn_ };
        let mut locals = stand.locals;
        dbg.stapel = stand.stapel;
        let (datei, zeile) = dbg.stelle(stand.zeile);
        crate::prozess::vordergrund_freigeben();
        dbg_emit(&serde_json::json!({
            "event": "paused", "reason": "error", "message": crate::meldung::t(&meldung),
            "line": zeile, "file": datei, "depth": dbg.stapel.len(),
            "locals": self.dbg_locals_json(fn_, &locals),
            "globals": self.dbg_globals_json(),
            "stack": Self::dbg_stack_json(&dbg, stand.zeile),
            "watches": self.dbg_watches_json(&dbg, fn_, &locals),
        }));
        while let Some(cmd) = dbg.kanal.naechstes(&mut Self::fenster_pumpe(self.fenster_da())) {
            match crate::debugger::art(&cmd) {
                "eval" => self.dbg_eval_antwort(&cmd, fn_, &mut locals),
                "set-watches" => {
                    Self::dbg_watches_setzen(&mut dbg, &cmd);
                    dbg_emit(&serde_json::json!({"event":"watches",
                        "watches": self.dbg_watches_json(&dbg, fn_, &locals)}));
                }
                "set-breakpoints" => self.dbg_set_breakpoints(&mut dbg, &cmd),
                "set" => dbg_emit(&serde_json::json!({"event":"set-error",
                    "name": cmd.get("name").cloned().unwrap_or_default(),
                    "message": "set: nach einem Fehler laeuft das Programm nicht weiter"})),
                _ => break,
            }
        }
        self.dbg = Some(dbg);
    }

    /// Den Text eines Protokollpunkts bauen: `{ausdruck}` ausgewertet wie
    /// ein `eval`, ein Fehler steht als `{? meldung}` im Text statt ihn zu
    /// verschlucken.
    pub(super) fn dbg_log_text(&self, vorlage: &str, fn_: &Func, locals: &[Value]) -> String {
        let mut out = String::new();
        for (stueck, ausdruck) in crate::debugger::log_stuecke(vorlage) {
            if !ausdruck { out.push_str(&stueck); continue; }
            match crate::parser::parse_expression(&stueck).and_then(|n| self.eval_node(&n, fn_, locals)) {
                Ok(v) => out.push_str(&self.dbg_anzeige(&v).1),
                Err(e) => { out.push_str("{? "); out.push_str(&e); out.push('}'); }
            }
        }
        out
    }

    /// `eval` beantworten (im Halt und im Fehler-Halt).
    pub(super) fn dbg_eval_antwort(&self, cmd: &serde_json::Value, fn_: &Func, locals: &mut [Value]) {
        let src = cmd.get("expr").and_then(|v| v.as_str()).unwrap_or("");
        // `id` kommt unveraendert zurueck -- so weiss ein Editor,
        // welche Frage beantwortet ist (Tooltip oder Eingabezeile).
        let id = cmd.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let mut ev = match crate::parser::parse_expression(src)
            .and_then(|n| self.eval_node(&n, fn_, locals)) {
            Ok(v) => {
                let (typ, wert) = self.dbg_anzeige(&v);
                serde_json::json!({"event":"eval-result","value":wert,"type":typ})
            }
            Err(e) => serde_json::json!({"event":"eval-error","message":e}),
        };
        if !id.is_null() { ev["id"] = id; ev["expr"] = serde_json::json!(src); }
        dbg_emit(&ev);
    }

    /// Gibt es ein Fenster, dessen Nachrichten waehrend eines Halts abgeholt
    /// werden muessen?
    pub(super) fn fenster_da(&self) -> bool {
        #[cfg(feature = "graphics")]
        { return self.gfx.is_some(); }
        #[allow(unreachable_code)]
        false
    }

    /// Die Pumpe fuer das Warten im Halt: holt die Fensternachrichten ab
    /// (raylibs PollInputEvents), ohne zu zeichnen -- das Fenster bleibt
    /// stehen, aber Windows haelt es nicht fuer abgestuerzt.
    pub(super) fn fenster_pumpe(da: bool) -> impl FnMut() {
        move || {
            #[cfg(feature = "graphics")]
            if da { unsafe { raylib::ffi::PollInputEvents(); } }
            #[cfg(not(feature = "graphics"))]
            let _ = da;
        }
    }

    /// (Datei, Zeile) einer gemergten Zeile fuer die Meldungen des Debuggers.
    pub fn debug_stelle(&self, gemergt: u32) -> (String, u32) {
        match &self.dbg {
            Some(d) => d.stelle(gemergt),
            None => (String::new(), gemergt),
        }
    }

    /// Restlichen (noch nicht gesendeten) Programm-Output als output-Event
    /// schicken (live-Ausgabe waehrend des Debuggens). Auch am Ende aufrufen.
    pub fn debug_flush_output(&mut self) {
        if let Some(mut dbg) = self.dbg.take() {
            if self.out.len() > dbg.out_sent {
                let chunk = self.out[dbg.out_sent..].to_string();
                dbg.out_sent = self.out.len();
                dbg_emit(&serde_json::json!({"event":"output","text":chunk}));
            }
            self.dbg = Some(dbg);
        }
    }

    /// Per-Zeile-Hook fuer den Debugger. take/restore von self.dbg vermeidet
    /// Borrow-Konflikte mit self.globals/out beim Snapshot/eval.
    #[inline(never)]
    pub(super) fn debug_on_line(&mut self, fn_: &Func, locals: &mut [Value]) -> R<()> {
        let mut dbg = match self.dbg.take() { Some(d) => d, None => return Ok(()) };
        let r = self.debug_cycle(&mut dbg, fn_, locals);
        self.dbg = Some(dbg);
        r
    }

    pub(super) fn debug_cycle(&mut self, dbg: &mut DebugState, fn_: &Func, locals: &mut [Value]) -> R<()> {
        let line = self.cur_line;
        let depth = self.depth;
        // Was waehrend des Laufens kam: Haltepunkte, Ueberwachen, Anhalten.
        for cmd in dbg.kanal.lebende() {
            match crate::debugger::art(&cmd) {
                "set-breakpoints" => self.dbg_set_breakpoints(dbg, &cmd),
                "set-watches" => Self::dbg_watches_setzen(dbg, &cmd),
                "pause" => dbg.step = StepMode::Into,
                _ => {}
            }
        }
        // Neue Ausgabe live nachschieben.
        if self.out.len() > dbg.out_sent {
            let chunk = self.out[dbg.out_sent..].to_string();
            dbg.out_sent = self.out.len();
            dbg_emit(&serde_json::json!({"event":"output","text":chunk}));
        }
        // Pause noetig? (Step-Regel ODER Breakpoint mit erfuellter Bedingung)
        let mut pause = match dbg.step {
            StepMode::Into => true,
            StepMode::Over => depth <= dbg.step_depth,
            StepMode::Out  => depth <  dbg.step_depth,
            StepMode::Run  => false,
        };
        if dbg.run_to == Some(line) { pause = true; }
        let mut protokoll: Option<String> = None;
        if !pause {
            if let Some(hp) = dbg.breakpoints.get_mut(&line) {
                // Bedingung: fail-open (bei Eval-Fehler trotzdem anhalten).
                let bed_ok = match &hp.bed {
                    None => true,
                    Some(expr) => self.eval_node(expr, fn_, locals)
                        .map(|v| dbg_truthy(&v)).unwrap_or(true),
                };
                if bed_ok {
                    hp.zaehler += 1;
                    let treffer_ok = match hp.treffer {
                        None => true,
                        Some((art, n)) => crate::debugger::treffer_passt(art, n, hp.zaehler),
                    };
                    if treffer_ok {
                        // Ein Protokollpunkt haelt nicht, er schreibt.
                        match hp.log.clone() {
                            Some(vorlage) => protokoll = Some(self.dbg_log_text(&vorlage, fn_, locals)),
                            None => pause = true,
                        }
                    }
                }
            }
        }
        if let Some(text) = protokoll {
            let (datei, zeile) = dbg.stelle(line);
            dbg_emit(&serde_json::json!({"event": "log", "text": text, "line": zeile, "file": datei}));
        }
        if !pause { return Ok(()); }
        // Jeder Halt verbraucht "bis hier" -- auch einer an einem Haltepunkt
        // davor, sonst hielte das Programm spaeter unerwartet noch einmal.
        dbg.run_to = None;
        let (datei, zeile) = dbg.stelle(line);
        // Das Programm ist womoeglich gerade vorn -- die IDE darf sich nur
        // nach vorn holen, wenn es den Vordergrund freigibt.
        crate::prozess::vordergrund_freigeben();
        dbg_emit(&serde_json::json!({
            "event": "paused", "line": zeile, "file": datei, "depth": depth,
            "locals": self.dbg_locals_json(fn_, locals),
            "globals": self.dbg_globals_json(),
            "stack": Self::dbg_stack_json(dbg, line),
            "watches": self.dbg_watches_json(dbg, fn_, locals),
        }));
        // Kommandos lesen bis continue/step/stop/EOF.
        loop {
            let cmd = match dbg.kanal.naechstes(&mut Self::fenster_pumpe(self.fenster_da())) {
                Some(c) => c,
                None => { self.debug_stop_flag = true; return Err("__DEBUG_STOP__".into()); }
            };
            match cmd.get("cmd").and_then(|v| v.as_str()).unwrap_or("") {
                "continue"  => { dbg.step = StepMode::Run; return Ok(()); }
                "step-over" => { dbg.step = StepMode::Over; dbg.step_depth = depth; return Ok(()); }
                "step-into" => { dbg.step = StepMode::Into; return Ok(()); }
                "step-out"  => { dbg.step = StepMode::Out; dbg.step_depth = depth; return Ok(()); }
                "stop"      => { self.debug_stop_flag = true; return Err("__DEBUG_STOP__".into()); }
                "set-breakpoints" => self.dbg_set_breakpoints(dbg, &cmd),
                "pause" => {}
                "run-to" => {
                    if let Some(ln) = cmd.get("line").and_then(|v| v.as_u64()) {
                        let datei = cmd.get("file").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        if let Some((g, _)) = dbg.haltestelle(&datei, ln as u32) {
                            dbg.run_to = Some(g);
                            dbg.step = StepMode::Run;
                            return Ok(());
                        }
                        // Dort haelt nichts (fremde Datei, kein Code darunter):
                        // sagen statt stumm stehen zu bleiben -- der Editor
                        // wartete sonst auf ein Laufen, das nie kommt.
                        dbg_emit(&serde_json::json!({"event": "run-to-error",
                            "line": ln, "file": datei,
                            "message": "dort und darunter steht kein Code dieses Programms"}));
                    }
                }
                "set-watches" => {
                    Self::dbg_watches_setzen(dbg, &cmd);
                    // Gleich beantworten -- wer im Halt einen Ausdruck
                    // dazunimmt, will seinen Wert jetzt sehen, nicht erst beim
                    // naechsten Halt.
                    dbg_emit(&serde_json::json!({"event":"watches",
                        "watches": self.dbg_watches_json(dbg, fn_, locals)}));
                }
                "set" => {
                    let name = cmd.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let src = cmd.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    match self.dbg_setzen(fn_, locals, &name, &src) {
                        Ok(v) => dbg_emit(&serde_json::json!({"event":"set-result","name":name,
                            "value":dbg_short(&v),"type":v.type_name(),
                            "locals": self.dbg_locals_json(fn_, locals),
                            "globals": self.dbg_globals_json(),
                            "watches": self.dbg_watches_json(dbg, fn_, locals)})),
                        Err(e) => dbg_emit(&serde_json::json!({"event":"set-error","name":name,"message":e})),
                    }
                }
                "eval" => self.dbg_eval_antwort(&cmd, fn_, locals),
                _ => {}
            }
        }
    }

    /// Der Aufrufstapel fuer das paused-Ereignis: INNEN zuerst, je Eintrag
    /// Name und Zeile -- die innerste steht in `line`, jede aeussere in der
    /// Zeile, in der sie die naechst innere rief.
    pub(super) fn dbg_stack_json(dbg: &DebugState, line: u32) -> serde_json::Value {
        let n = dbg.stapel.len();
        let mut out = Vec::new();
        for k in (0..n).rev() {
            let gemergt = if k + 1 == n { line } else { dbg.stapel[k + 1].1 };
            let (datei, zeile) = dbg.stelle(gemergt);
            out.push(serde_json::json!({"name": dbg.stapel[k].0, "line": zeile, "file": datei}));
        }
        serde_json::Value::Array(out)
    }

    pub(super) fn dbg_watches_setzen(dbg: &mut DebugState, cmd: &serde_json::Value) {
        dbg.watches = cmd.get("exprs").and_then(|v| v.as_array()).map(|a| a.iter()
            .filter_map(|e| e.as_str())
            .map(|src| (src.to_string(), crate::parser::parse_expression(src).ok()))
            .collect()).unwrap_or_default();
    }

    /// Die ueberwachten Ausdruecke, jeder fuer sich ausgewertet: ein Fehler
    /// in einem nimmt den anderen nichts.
    pub(super) fn dbg_watches_json(&self, dbg: &DebugState, fn_: &Func, locals: &[Value]) -> serde_json::Value {
        let mut out = Vec::new();
        for (src, node) in &dbg.watches {
            let r = match node {
                Some(n) => self.eval_node(n, fn_, locals),
                None => Err("laesst sich nicht lesen".into()),
            };
            out.push(match r {
                Ok(v) => { let (typ, wert) = self.dbg_anzeige(&v); serde_json::json!({"expr": src, "value": wert, "type": typ}) }
                Err(e) => serde_json::json!({"expr": src, "error": e}),
            });
        }
        serde_json::Value::Array(out)
    }

    /// Eine Variable im Halt setzen: erst die lokale des aktuellen Rahmens,
    /// sonst die globale. Der neue Wert geht durch dieselbe Umwandlung wie
    /// eine Zuweisung -- aus "5" wird in einer INTEGER-Variable kein Text.
    pub(super) fn dbg_setzen(&self, fn_: &Func, locals: &mut [Value], name: &str, src: &str) -> R<Value> {
        let key = name.trim().to_lowercase();
        if key.is_empty() { return Err("set: kein Name".into()); }
        let node = crate::parser::parse_expression(src)?;
        let v = self.eval_node(&node, fn_, locals)?;
        if let Some(i) = fn_.local_names.iter().position(|n| *n == key) {
            if i < locals.len() {
                let ty = fn_.local_types.get(i).cloned().unwrap_or_default();
                let neu = if ty.is_empty() { v } else { coerce(v, &ty, name)? };
                locals[i] = neu.clone();
                return Ok(neu);
            }
        }
        if let Some(slot) = self.globals.get(&key) {
            let mut s = slot.borrow_mut();
            if s.is_const { return Err(format!("set: '{}' ist eine Konstante", name)); }
            let neu = if s.ty.is_empty() { v } else { coerce(v, &s.ty, name)? };
            s.value = neu.clone();
            return Ok(neu);
        }
        Err(format!("set: '{}' nicht gefunden", name))
    }

    /// Haltepunkte ersetzen. Zwei Formen, auch gemischt: `lines` +
    /// `conditions` meinen die HAUPTdatei (die alte Form), `breakpoints` ist
    /// eine Liste von {file, line[, condition]} fuer jede Datei des
    /// Programms. Jeder wird auf seine Haltestelle gelegt (Zeile ohne Code ->
    /// die naechste mit), und die Antwort `breakpoints` sagt, wo er wirklich
    /// haelt -- damit ein Editor den roten Punkt dorthin setzen kann.
    pub(super) fn dbg_set_breakpoints(&self, dbg: &mut DebugState, cmd: &serde_json::Value) {
        dbg.breakpoints.clear();
        // (Datei, Zeile, Bedingung, Trefferzahl, Protokolltext)
        let mut wuensche: Vec<(String, u32, Option<String>, Option<String>, Option<String>)> = Vec::new();
        let bedingung_haupt = |ln: u64| cmd.get("conditions").and_then(|c| c.get(ln.to_string()))
            .and_then(|v| v.as_str()).map(|s| s.to_string());
        if let Some(lines) = cmd.get("lines").and_then(|v| v.as_array()) {
            for l in lines {
                if let Some(ln) = l.as_u64() { wuensche.push((String::new(), ln as u32, bedingung_haupt(ln), None, None)); }
            }
        }
        // Eine Bedingung ohne Eintrag in `lines` (alte Form) gilt trotzdem.
        if let Some(conds) = cmd.get("conditions").and_then(|v| v.as_object()) {
            for k in conds.keys() {
                if let Ok(ln) = k.parse::<u32>() {
                    if !wuensche.iter().any(|w| w.0.is_empty() && w.1 == ln) {
                        wuensche.push((String::new(), ln, bedingung_haupt(ln as u64), None, None));
                    }
                }
            }
        }
        if let Some(bps) = cmd.get("breakpoints").and_then(|v| v.as_array()) {
            for b in bps {
                let datei = b.get("file").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ln = match b.get("line").and_then(|v| v.as_u64()) { Some(l) => l as u32, None => continue };
                let text = |k: &str| b.get(k).and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty()).map(|s| s.to_string());
                // `hits` darf auch eine Zahl sein.
                let hits = text("hits").or_else(|| b.get("hits").and_then(|v| v.as_u64()).map(|n| n.to_string()));
                wuensche.push((datei, ln, text("condition"), hits, text("log")));
            }
        }
        let mut antwort = Vec::new();
        for (datei, ln, bed, hits, log) in wuensche {
            let anzeige = if datei.is_empty() { dbg.haupt() } else { datei.clone() };
            match dbg.haltestelle(&datei, ln) {
                Some((g, echt)) => {
                    // Bedingung vorab parsen; bei Parse-Fehler unbedingter BP.
                    let node = bed.as_deref().and_then(|src| crate::parser::parse_expression(src).ok());
                    let hp = Haltepunkt { bed: node,
                        treffer: hits.as_deref().and_then(crate::debugger::treffer_lesen),
                        log, zaehler: 0 };
                    // Zwei Wuensche auf derselben Haltestelle: einer ohne
                    // Bedingung haelt immer, ein bedingter nimmt ihm das nicht.
                    let alt_ohne = dbg.breakpoints.get(&g).map(|h| h.schlicht()).unwrap_or(false);
                    if !alt_ohne { dbg.breakpoints.insert(g, hp); }
                    antwort.push(serde_json::json!({"file": anzeige, "line": ln, "actual": echt, "verified": true}));
                }
                // `in_program`: haelt er nie, weil darunter kein Code steht
                // (ein Befund) -- oder weil die Datei gar nicht dazugehoert?
                None => antwort.push(serde_json::json!({"file": anzeige, "line": ln, "actual": 0, "verified": false,
                                                        "in_program": dbg.gehoert_dazu(&datei)})),
            }
        }
        dbg_emit(&serde_json::json!({"event": "breakpoints", "breakpoints": antwort}));
    }

    pub(super) fn dbg_locals_json(&self, fn_: &Func, locals: &[Value]) -> serde_json::Value {
        let mut out = Vec::new();
        for (i, nm) in fn_.local_names.iter().enumerate() {
            // Compiler-Zwischenwerte ueberspringen (namenlos oder __-Praefix).
            if nm.is_empty() || nm.starts_with("__") { continue; }
            if let Some(v) = locals.get(i) {
                let decl = fn_.local_types.get(i).map(|s| s.as_str());
                out.push(self.dbg_var_json(nm, v, decl));
            }
        }
        serde_json::Value::Array(out)
    }

    pub(super) fn dbg_globals_json(&self) -> serde_json::Value {
        let mut out = Vec::new();
        for (name, slot) in &self.globals {
            if name.starts_with("__") { continue; }
            let s = slot.borrow();
            if s.is_const { continue; }      // Baseline-Konstanten (Farben/Keys/PI) ausblenden
            out.push(self.dbg_var_json(name, &s.value, Some(&s.ty)));
        }
        // Ohne Ruecksicht auf Gross/klein -- die Namen stehen in ihrer
        // Schreibweise da, und `MAX_HP` gehoert nicht vor `alpha`.
        out.sort_by_key(|e| e["name"].as_str().unwrap_or("").to_lowercase());
        serde_json::Value::Array(out)
    }

    /// Mini-Evaluator fuer Debugger-Bedingungen + `eval`: Identifier (Locals
    /// per Name / Globals), Literale, unaere/binaere Operatoren. Bewusst ein
    /// Subset (kein Member/Index/Call) -- deckt typische Breakpoint-Bedingungen.
    pub(super) fn eval_node(&self, n: &crate::ast::Node, fn_: &Func, locals: &[Value]) -> R<Value> {
        use crate::ast::{Node, NumV};
        match n {
            Node::NumberLit(NumV::Int(i)) => Ok(Value::Int(*i)),
            Node::NumberLit(NumV::Float(f)) => Ok(Value::Float(*f)),
            Node::StringLit(s) => Ok(Value::str_rc(s)),
            Node::BoolLit(b) => Ok(Value::Bool(*b)),
            Node::Identifier(name) => {
                let key = name.to_lowercase();
                if let Some(i) = fn_.local_names.iter().position(|n| *n == key) {
                    if let Some(v) = locals.get(i) { return Ok(v.clone()); }
                }
                if let Some(slot) = self.globals.get(&key) {
                    return Ok(slot.borrow().value.clone());
                }
                Err(format!("eval: '{}' nicht gefunden", name))
            }
            Node::UnaryOp { op, operand } => {
                let v = self.eval_node(operand, fn_, locals)?;
                match op.as_str() {
                    "-" => match v { Value::Int(i) => Ok(Value::Int(-i)),
                                     _ => Ok(Value::Float(-as_f64(&v))) },
                    "not" => Ok(Value::Bool(!dbg_truthy(&v))),
                    _ => Err(format!("eval: unaerer Operator '{}' nicht unterstuetzt", op)),
                }
            }
            Node::BinaryOp { op, left, right } => {
                let a = self.eval_node(left, fn_, locals)?;
                let b = self.eval_node(right, fn_, locals)?;
                dbg_binop(op, &a, &b)
            }
            // Mitglieder und Indizes (2026-09-27): `held.hp`, `feld[3]`,
            // `karte["a"]` -- fuer Ueberwachen, die Eingabezeile und die
            // Werte beim Ueberfahren. Nur FELDER: eine PROPERTY fuehrte Code
            // aus, und Hinsehen soll nichts veraendern.
            Node::MemberAccess { target, name } => {
                let t = self.eval_node(target, fn_, locals)?;
                match &t {
                    Value::Instance(rc) => {
                        let inst = rc.borrow();
                        match inst.platz(&name.to_lowercase()) {
                            Some(k) => Ok(inst.fields[k].clone()),
                            None => Err(format!("eval: {} hat kein Feld '{}' (eine PROPERTY wertet der Debugger nicht aus)",
                                self.dbg_klassen.get(&inst.class_name.to_lowercase()).cloned().unwrap_or_else(|| inst.class_name.to_string()), name)),
                        }
                    }
                    Value::Nil => Err(format!("eval: '{}' auf NIL", name)),
                    _ => Err(format!("eval: {} hat keine Mitglieder", t.type_name())),
                }
            }
            Node::IndexAccess { target, indices } => {
                let t = self.eval_node(target, fn_, locals)?;
                let idx: Vec<Value> = indices.iter().map(|i| self.eval_node(i, fn_, locals)).collect::<R<_>>()?;
                match (&t, idx.as_slice()) {
                    (Value::Array(rc), _) => {
                        let mut ganz = Vec::with_capacity(idx.len());
                        for v in &idx {
                            match v { Value::Int(i) => ganz.push(*i), _ => return Err("eval: ein Feld-Index muss INTEGER sein".into()) }
                        }
                        let a = rc.borrow();
                        let k = a.flat_index(&ganz)?;
                        Ok(a.cells.get(k))
                    }
                    (Value::Map(rc), [Value::Str(k)]) => rc.borrow().get(k.as_str()).cloned()
                        .ok_or_else(|| format!("eval: Schluessel '{}' fehlt", k)),
                    (Value::Tuple(tp), [Value::Int(i)]) => tp.get(*i as usize).cloned()
                        .ok_or_else(|| format!("eval: Index {} ausserhalb", i)),
                    (Value::Str(s), [Value::Int(i)]) => s.chars().nth((*i).max(0) as usize)
                        .map(|c| Value::str_rc(c.to_string()))
                        .ok_or_else(|| format!("eval: Index {} ausserhalb", i)),
                    _ => Err(format!("eval: {} laesst sich so nicht indizieren", t.type_name())),
                }
            }
            _ => Err("eval: Ausdruck nicht unterstuetzt (nur Namen, Literale, Operatoren, Felder und Indizes -- keine Aufrufe)".into()),
        }
    }
}
