//! Befehle fuer System und Ablauf: zeit, Betriebssystem, Prozesse und Hintergrundaufgaben, Pruefungen (ASSERT, LOG), timer, scene.
//!
//! Teil von `vm.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl<'p> Vm<'p> {
    /// Modul `zeit` (zeit.rs): mit Datum und Uhrzeit rechnen.
    ///
    /// Der Zeitwert ist eine ganze Zahl: Sekunden seit 1970 in ORTSZEIT --
    /// so passen `ZEIT_JETZT()` und ein aus der Datenbank gelesener Anstoss
    /// ohne Zeitzonen-Umrechnung zusammen. Alles hier ist reine Mathematik
    /// (die Tests dazu stehen in zeit.rs); nur JETZT fragt die Uhr.
    pub(super) fn try_zeit(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("zeit_") { return Ok(None); }
        use crate::zeit;

        fn z_int(a: &[Value], i: usize, fn_: &str) -> R<i64> {
            match a.get(i) {
                Some(Value::Int(n)) => Ok(*n),
                Some(Value::Float(f)) => Ok(*f as i64),
                _ => Err(format!("{}: erwartet einen Zeitwert (INTEGER, Arg {})", fn_, i + 1)),
            }
        }
        fn z_str<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) {
                Some(Value::Str(s)) => Ok(s),
                _ => Err(format!("{}: erwartet STRING (Arg {})", fn_, i + 1)),
            }
        }

        let v = match name {
            "zeit_jetzt" => {
                let (j, mo, d, h, mi, s) = crate::builtins::local_datetime();
                Value::Int(zeit::aus_teilen(j, mo, d, h, mi, s))
            }
            // Unix-Zeit (UTC, mit Bruchteil) -- die Zahl, die andere Programme,
            // Web-APIs und Dateiformate meinen (Pythons time.time())
            "zeit_unix" => {
                if !a.is_empty() { return Err("ZEIT_UNIX: erwartet keine Argumente".into()); }
                let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                Value::Float(d.as_secs_f64())
            }
            "zeit_aus_unix" => {
                let u = match a.first() {
                    Some(Value::Int(n)) if a.len() == 1 => *n,
                    Some(Value::Float(f)) if a.len() == 1 && f.is_finite() => f.floor() as i64,
                    _ => return Err("ZEIT_AUS_UNIX: erwartet Unix-Sekunden (eine Zahl)".into()),
                };
                Value::Int(zeit::aus_unix_mit(u, crate::builtins::lokaler_versatz))
            }
            "zeit_nach_unix" => {
                if a.len() != 1 { return Err("ZEIT_NACH_UNIX: erwartet einen Zeitpunkt".into()); }
                let t = z_int(a, 0, "ZEIT_NACH_UNIX")?;
                Value::Int(zeit::nach_unix_mit(t, crate::builtins::lokaler_versatz))
            }
            "zeit_parse" => {
                let text = z_str(a, 0, "ZEIT_PARSE")?;
                match zeit::parse(text) {
                    Some(t) => Value::Int(t),
                    // Klartext statt stiller -1: ein unlesbares Datum ist fast
                    // immer ein Fehler in den Daten, und der faellt sonst erst
                    // viel spaeter als unsinnige Rechnung auf.
                    None => return Err(format!(
                        "ZEIT_PARSE: '{}' ist kein Zeitpunkt -- erwartet \
                         'JJJJ-MM-TT hh:mm:ss' (auch mit T, ohne Sekunden, \
                         nur Datum oder mit Zeitzone) oder \
                         'Sat, 28 Aug 2026 20:30:00 +0200'", text)),
                }
            }
            // ZEIT_TEXT$ gibt Text, ZEIT_LESBAR fragt nach. Getrennte Namen,
            // weil "lesbar" sich als beides lesen laesst -- ich habe die
            // zwei beim Schreiben der ersten Pruefung selbst verwechselt.
            "zeit_text$" | "zeit_text" =>
                Value::str_rc(zeit::format(z_int(a, 0, "ZEIT_TEXT$")?, "")),
            "zeit_lesbar" => Value::Bool(zeit::parse(z_str(a, 0, "ZEIT_LESBAR")?).is_some()),
            "zeit_format$" | "zeit_format" => {
                let t = z_int(a, 0, "ZEIT_FORMAT$")?;
                let muster = if a.len() > 1 { z_str(a, 1, "ZEIT_FORMAT$")? } else { "" };
                Value::str_rc(zeit::format_pruefen(t, muster)?)
            }
            "zeit_teil" => {
                let t = z_int(a, 0, "ZEIT_TEIL")?;
                let was = z_str(a, 1, "ZEIT_TEIL")?;
                match zeit::teil(t, was) {
                    Some(n) => Value::Int(n),
                    None => return Err(format!(
                        "ZEIT_TEIL: '{}' ist kein Feld -- moeglich sind {}", was, zeit::TEILE)),
                }
            }
            "zeit_aus_teilen" => {
                Value::Int(zeit::aus_teilen(
                    z_int(a, 0, "ZEIT_AUS_TEILEN")?, z_int(a, 1, "ZEIT_AUS_TEILEN")?,
                    z_int(a, 2, "ZEIT_AUS_TEILEN")?,
                    if a.len() > 3 { z_int(a, 3, "ZEIT_AUS_TEILEN")? } else { 0 },
                    if a.len() > 4 { z_int(a, 4, "ZEIT_AUS_TEILEN")? } else { 0 },
                    if a.len() > 5 { z_int(a, 5, "ZEIT_AUS_TEILEN")? } else { 0 }))
            }
            "zeit_plus" => Value::Int(
                z_int(a, 0, "ZEIT_PLUS")?.saturating_add(z_int(a, 1, "ZEIT_PLUS")?)),
            "zeit_diff" => Value::Int(
                z_int(a, 0, "ZEIT_DIFF")?.saturating_sub(z_int(a, 1, "ZEIT_DIFF")?)),
            "zeit_dauer$" | "zeit_dauer" => Value::str_rc(zeit::dauer(z_int(a, 0, "ZEIT_DAUER$")?)),
            "zeit_wochentag" => Value::Int(zeit::wochentag(z_int(a, 0, "ZEIT_WOCHENTAG")?)),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Betriebssystem-Builtins, die VM-Zustand brauchen (WP A). Die
    /// zustandsfreien (`ARGC`/`ARG$`/`GETENV$`/`SETENV`/`CWD$`/`CHDIR`) stehen
    /// in `builtins.rs`.
    ///
    /// Gemeinsamer Grund, warum diese vier hier stehen: **`PRINT` wird
    /// gepuffert** (`self.out`, geschrieben erst am Programmende). Wer daneben
    /// auf stderr schreibt oder ein Kindprogramm auf dasselbe Terminal laufen
    /// laesst, saehe die Ausgaben sonst in falscher Reihenfolge -- die eigenen
    /// PRINTs kaemen ganz zum Schluss. Darum flusht jeder dieser Befehle den
    /// Puffer zuerst.
    pub(super) fn try_os(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if let Some(v) = self.try_pruefen(name, a)? { return Ok(Some(v)); }
        // SLEEP hat nichts mit Grafik zu tun -- `std::thread::sleep`, sonst
        // nichts. Es sass trotzdem in `try_graphics` und war damit in einem
        // Build ohne raylib nicht verfuegbar. Aufgefallen beim Linux-CI-Job:
        // ein Konsolenprogramm, das auf einen Hintergrund-Auftrag wartet,
        // braucht genau das, um sich zu bremsen -- ohne SLEEP dreht die
        // Warteschleife auf 100% eines Kerns.
        if name == "sleep" {
            let ms = bi_int(a, 0, "SLEEP")?.max(0) as u64;
            // Was vor der Pause ausgegeben wurde, steht waehrend der Pause da.
            self.flush_out();
            std::thread::sleep(std::time::Duration::from_millis(ms));
            return Ok(Some(Value::Nil));
        }
        // STDIN() -- die Standardeingabe als FILE-Handle. Steht hier und nicht
        // in builtins.rs, weil es dieselbe Sperre braucht wie INPUT: unter
        // `dhrt profile --stoppable` und unter `dhrt debug` gehoert stdin dem
        // Stop-Kanal bzw. dem Kommando-Protokoll. Ein Programm, das dort liest,
        // wuerde dem Debugger eine Kommandozeile stehlen -- es bekommt darum
        // einen Strom, der sofort zu Ende ist (wie INPUT dort "" liefert).
        if name == "stdin" {
            if a.len() > 1 {
                return Err(format!("STDIN: erwartet 0..1 Argument(e), erhalten {}", a.len()));
            }
            let kod = match a.first() {
                None => crate::kodierung::Kodierung::Utf8,
                Some(Value::Str(s)) => crate::kodierung::aus_name(s, "STDIN")?,
                Some(_) => return Err("STDIN: erwartet STRING (Kodierung)".to_string()),
            };
            return Ok(Some(self.stdin_handle(kod)?));
        }
        if !matches!(name, "exit" | "eprint" | "shell" | "shell_out$" | "shell_out") { return Ok(None); }
        fn o_str<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) { Some(Value::Str(s)) => Ok(s), _ => Err(format!("{}: erwartet STRING (Arg {})", fn_, i + 1)) }
        }
        let v = match name {
            "exit" => {
                if a.len() > 1 { return Err(format!("EXIT: erwartet 0..1 Argument(e), erhalten {}", a.len())); }
                let code = match a.first() {
                    None => 0,
                    Some(Value::Int(n)) => *n,
                    _ => return Err("EXIT: erwartet INTEGER (Rueckgabewert)".into()),
                };
                // Betriebssysteme uebertragen nur das untere Byte eines
                // Rueckgabewerts. Statt `EXIT(256)` still zu 0 werden zu lassen
                // -- also aus "Fehler" ein "alles gut" zu machen -- ist das ein
                // Fehler mit Ansage.
                if !(0..=255).contains(&code) {
                    return Err(format!("EXIT: Rueckgabewert {} liegt ausserhalb 0..255", code));
                }
                self.exit_code = Some(code as i32);
                return Err(EXIT_REQUEST.into());
            }
            "eprint" => {
                if a.len() != 1 { return Err(format!("EPRINT: erwartet 1 Argument, erhalten {}", a.len())); }
                let text = crate::builtins::str_of(&a[0]);
                self.flush_out();
                use std::io::Write;
                let se = std::io::stderr();
                let mut h = se.lock();
                let _ = writeln!(h, "{}", text);
                let _ = h.flush();
                Value::Nil
            }
            "shell" | "shell_out$" | "shell_out" => {
                let anzeige = if name == "shell" { "SHELL" } else { "SHELL_OUT$" };
                if a.is_empty() { return Err(format!("{}: erwartet mind. 1 Argument (Programm)", anzeige)); }
                let prog = o_str(a, 0, anzeige)?.to_string();
                // Argumente EINZELN statt als eine Kommandozeile -- damit gibt
                // es keine Quoting-Regeln zu lernen und keine Shell, die aus
                // einem Dateinamen mit Leerzeichen zwei Argumente macht. Wer
                // wirklich eine Shell will, ruft sie ausdruecklich auf
                // (SHELL("cmd", "/c", "dir | more")).
                let mut rest: Vec<String> = Vec::new();
                for i in 1..a.len() { rest.push(o_str(a, i, anzeige)?.to_string()); }
                let mut cmd = std::process::Command::new(&prog);
                cmd.args(&rest);
                self.flush_out();
                if name == "shell" {
                    // Erbt stdout/stderr -- das Kind schreibt direkt aufs
                    // Terminal, ohne Umweg ueber unseren Puffer.
                    let st = cmd.status().map_err(|e| format!("SHELL: '{}' laesst sich nicht starten: {}", prog, e))?;
                    // Kein Rueckgabewert (Unix: durch ein Signal beendet) ->
                    // -1. Das ist eindeutig, weil ein echter Rueckgabewert
                    // immer 0..255 ist.
                    Value::Int(st.code().map(|c| c as i64).unwrap_or(-1))
                } else {
                    let out = cmd.output().map_err(|e| format!("SHELL_OUT$: '{}' laesst sich nicht starten: {}", prog, e))?;
                    // stderr des Kindes bleibt stderr (durchgereicht), nur
                    // stdout ist das Ergebnis -- sonst mischten sich
                    // Fehlermeldungen unbemerkt in die Nutzdaten.
                    use std::io::Write;
                    let se = std::io::stderr();
                    let _ = se.lock().write_all(&out.stderr);
                    Value::str_rc(String::from_utf8_lossy(&out.stdout).into_owned())
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Hintergrund-Auftraege (WP H): SHELL_START, DB_QUERY_START und TASK_START.
    ///
    /// Dasselbe Muster ueberall -- starten, pro Bild nachsehen, abholen.
    /// `shell_`/`db_query_` laufen als reine Rust-Arbeit ohne VM. `task_`
    /// fuehrt dagegen DH-Code aus, und zwar in einem EIGENEN dhrt-Prozess:
    /// im Thread ginge es nicht, weil `Value` ueberall `Rc` haelt und
    /// `Program` damit weder Send noch Sync ist.
    /// Der Kindprozess zu Argument 0 (PROCESS_*), mit Meldung fuer eine
    /// unbekannte oder schon geschlossene Nummer.
    pub(super) fn prozess(&mut self, a: &[Value], fn_: &str) -> R<&mut crate::prozess::Prozess> {
        let id = bi_int(a, 0, fn_)?;
        if id >= 0 {
            if let Some(Some(p)) = self.prozesse.get_mut(id as usize) { return Ok(p); }
        }
        Err(format!("{}: Prozess {} gibt es nicht (nie gestartet oder schon PROCESS_CLOSE)", fn_, id))
    }

    pub(super) fn try_hintergrund(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("shell_") || name.starts_with("db_query_")
             || name.starts_with("task_") || name.starts_with("window_") || name.starts_with("parent_")
             || name.starts_with("process_") || name.starts_with("code_") || name == "syntax_spans"
             || name == "opendoc" || name.starts_with("printer")) { return Ok(None); }
        let v = match name {
            // ===== Kindprozesse mit laufender Ausgabe (prozess.rs, fuer die IDE) =====
            "process_start" | "process_start_detached" => {
                // Losgeloest (PROCESS_START_DETACHED): ohne Leitungen und ohne
                // Griff, das Kind ueberlebt das Ende dieses Programms; die
                // Argumente nehmen beide gleich.
                let fn_ = if name == "process_start" { "PROCESS_START" } else { "PROCESS_START_DETACHED" };
                if a.is_empty() { return Err(format!("{}: erwartet mind. 1 Argument (Programm)", fn_)); }
                let prog = bi_str(a, 0, fn_)?.to_string();
                // Ein FELD von Texten wird ausgebreitet (eine variable Zahl von
                // Argumenten liess sich vorher gar nicht uebergeben), eine MAP
                // von Texten ist zusaetzliche Umgebung NUR fuer das Kind --
                // SETENV in den Eltern haette sie allen spaeteren Kindern
                // mitgegeben, und entfernen laesst sich eine Variable nicht.
                let mut rest: Vec<String> = Vec::new();
                let mut umgebung: Vec<(String, String)> = Vec::new();
                for i in 1..a.len() {
                    match &a[i] {
                        Value::Array(arr) => {
                            for x in arr.borrow().cells.iter() {
                                match x {
                                    Value::Str(s) => rest.push(s.to_string()),
                                    _ => return Err(format!("{}: ein Feld von Argumenten muss aus Texten bestehen", fn_)),
                                }
                            }
                        }
                        Value::Map(m) => {
                            for (k, v) in m.borrow().entries().iter() {
                                match v {
                                    Value::Str(s) => umgebung.push((k.clone(), s.to_string())),
                                    _ => return Err(format!("{}: die Umgebung muss aus Texten bestehen ('{}')", fn_, k)),
                                }
                            }
                        }
                        _ => rest.push(bi_str(a, i, fn_)?.to_string()),
                    }
                }
                if name == "process_start_detached" {
                    return Ok(Some(Value::Int(crate::prozess::Prozess::losgeloest(&prog, &rest, &umgebung)? as i64)));
                }
                let p = crate::prozess::Prozess::starten_mit(&prog, &rest, &umgebung)?;
                self.prozesse.push(Some(p));
                Value::Int((self.prozesse.len() - 1) as i64)
            }
            "process_read$" | "process_read" => Value::str_rc(&self.prozess(a, "PROCESS_READ$")?.lesen()),
            "process_err$" | "process_err" => Value::str_rc(&self.prozess(a, "PROCESS_ERR$")?.fehler_lesen()),
            "process_write" => {
                let text = bi_str(a, 1, "PROCESS_WRITE")?.to_string();
                self.prozess(a, "PROCESS_WRITE")?.schreiben(&text)?;
                Value::Nil
            }
            "process_close_input" => { self.prozess(a, "PROCESS_CLOSE_INPUT")?.eingabe_schliessen(); Value::Nil }
            "process_running" => Value::Bool(self.prozess(a, "PROCESS_RUNNING")?.laeuft()),
            "process_code" => Value::Int(self.prozess(a, "PROCESS_CODE")?.code()),
            "process_kill" => { self.prozess(a, "PROCESS_KILL")?.beenden(); Value::Nil }
            "process_front" => Value::Bool(self.prozess(a, "PROCESS_FRONT")?.nach_vorn()),
            "process_close" => {
                let id = bi_int(a, 0, "PROCESS_CLOSE")?;
                if id >= 0 { if let Some(platz) = self.prozesse.get_mut(id as usize) { *platz = None; } }
                Value::Nil
            }
            // ===== Sprachdienste fuer Editoren in Drachenhauch (lsp.rs, in-Prozess) =====
            // Dieselben Funktionen, die `dhrt lsp` VS Code anbietet -- eine IDE
            // in Drachenhauch braucht dafuer keinen zweiten Prozess und keine
            // JSON-RPC-Rahmung in BASIC. Zeilen und Spalten zaehlen ab 1.
            "code_check$" | "code_check" => {
                let text = bi_str(a, 0, "CODE_CHECK$")?.to_string();
                let basis = if a.len() > 1 { std::path::PathBuf::from(bi_str(a, 1, "CODE_CHECK$")?) }
                            else { std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")) };
                let liste: Vec<serde_json::Value> = crate::lsp::diagnose(&text, &basis).into_iter().map(|d| {
                    let zeile = d["range"]["start"]["line"].as_u64().unwrap_or(0) + 1;
                    let schwere = match d["severity"].as_u64() { Some(2) => "warnung", Some(3) | Some(4) => "hinweis", _ => "fehler" };
                    // Spalte ab 1 und Laenge in Zeichen -- wo im Text die
                    // Meldung hingehoert (lsp::fehler_bereich).
                    let von = d["range"]["start"]["character"].as_u64().unwrap_or(0);
                    let bis = d["range"]["end"]["character"].as_u64().unwrap_or(von + 1);
                    let mut e = serde_json::json!({"zeile": zeile, "schwere": schwere,
                                       "meldung": d["message"].as_str().unwrap_or(""),
                                       "spalte": von + 1, "laenge": bis.saturating_sub(von).max(1)});
                    // Korrekturen ab 1 gezaehlt wie alles hier: je Aenderung
                    // Zeile, Spalte, Laenge (0 = einfuegen) und der neue Text.
                    if let Some(k) = d["data"]["korrekturen"].as_array() {
                        e["korrekturen"] = serde_json::Value::Array(k.iter().map(|k| serde_json::json!({
                            "titel": k["titel"],
                            "aenderungen": k["aenderungen"].as_array().cloned().unwrap_or_default().iter().map(|a| {
                                let (v, b) = (a["von"].as_u64().unwrap_or(0), a["bis"].as_u64().unwrap_or(0));
                                let (z, bz) = (a["zeile"].as_u64().unwrap_or(0), a["bis_zeile"].as_u64().unwrap_or(0));
                                // laenge nur innerhalb einer Zeile; bis_zeile/bis_spalte immer.
                                let laenge = if bz == z { b.saturating_sub(v) as i64 } else { -1 };
                                serde_json::json!({"zeile": z + 1, "spalte": v + 1, "laenge": laenge,
                                                   "bis_zeile": bz + 1, "bis_spalte": b + 1, "text": a["text"]})
                            }).collect::<Vec<_>>(),
                        })).collect());
                    }
                    e
                }).collect();
                Value::str_rc(&serde_json::Value::Array(liste).to_string())
            }
            "code_hover$" | "code_hover" => {
                let text = bi_str(a, 0, "CODE_HOVER$")?;
                let z = bi_int(a, 1, "CODE_HOVER$")?.max(1) as usize - 1;
                let s = bi_int(a, 2, "CODE_HOVER$")?.max(1) as usize - 1;
                let basis = code_basis(a, 3, "CODE_HOVER$")?;
                let md = crate::lsp::hover(text, basis.as_deref(), z, s)
                    .and_then(|h| h["contents"]["value"].as_str().map(str::to_string)).unwrap_or_default();
                Value::str_rc(&md)
            }
            "code_complete" => {
                let text = bi_str(a, 0, "CODE_COMPLETE")?;
                let z = bi_int(a, 1, "CODE_COMPLETE")?.max(1) as usize - 1;
                let s = bi_int(a, 2, "CODE_COMPLETE")?.max(1) as usize - 1;
                let basis = code_basis(a, 3, "CODE_COMPLETE")?;
                let namen: Vec<String> = crate::lsp::vervollstaendigung(text, basis.as_deref(), z, s).into_iter()
                    .filter_map(|e| e["label"].as_str().map(str::to_string)).collect();
                let n = namen.len() as i64;
                let mut arr = DhArray::new("string".to_string(), vec![n], || Value::str_rc(""));
                for (k, s) in namen.into_iter().enumerate() { arr.cells.set(k, Value::str_rc(&s)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            "code_definition" => {
                let text = bi_str(a, 0, "CODE_DEFINITION")?;
                let z = bi_int(a, 1, "CODE_DEFINITION")?.max(1) as usize - 1;
                let s = bi_int(a, 2, "CODE_DEFINITION")?.max(1) as usize - 1;
                let d = crate::lsp::definition(text, "", z, s);
                let (zeile, spalte) = match d["range"]["start"]["line"].as_u64() {
                    Some(l) => (l as i64 + 1, d["range"]["start"]["character"].as_u64().unwrap_or(0) as i64 + 1),
                    None => (-1, -1),
                };
                Value::Tuple(Rc::new(vec![Value::Int(zeile), Value::Int(spalte)]))
            }
            "code_references" => {
                let text = bi_str(a, 0, "CODE_REFERENCES")?;
                let z = bi_int(a, 1, "CODE_REFERENCES")?.max(1) as usize - 1;
                let s = bi_int(a, 2, "CODE_REFERENCES")?.max(1) as usize - 1;
                let zeilen: Vec<i64> = crate::lsp::fundstellen(text, "", z, s).as_array().map(|v| v.iter()
                    .filter_map(|o| o["range"]["start"]["line"].as_u64().map(|l| l as i64 + 1)).collect()).unwrap_or_default();
                let n = zeilen.len() as i64;
                let mut arr = DhArray::new("integer".to_string(), vec![n], || Value::Int(0));
                for (k, l) in zeilen.into_iter().enumerate() { arr.cells.set(k, Value::Int(l)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            // CODE_FORMAT$(quelltext$[, einruecken]): derselbe Formatierer wie
            // `dhrt fmt` -- Schluesselwoerter gross, Leerraum am Zeilenende weg,
            // mit `einruecken` (Vorgabe TRUE) die Bloecke neu eingerueckt. Leer,
            // wenn sich die Quelle nicht lexen laesst: an kaputtem Code
            // herumzuruecken hilft niemandem, und die IDE sagt es dann.
            "code_format$" | "code_format" => {
                let text = bi_str(a, 0, "CODE_FORMAT$")?;
                let einruecken = if a.len() > 1 { a[1].truthy() } else { true };
                Value::str_rc(crate::formatiere(text, einruecken, "    ").unwrap_or_default())
            }
            // CODE_RENAME$: ein Symbol im ganzen Text umbenennen. Liefert den
            // neuen Quelltext, oder LEER, wenn an der Stelle kein Name steht
            // oder der neue keiner ist -- ein Aufrufer, der das nicht prueft,
            // schreibt dann nichts, statt die Datei zu leeren.
            "code_rename$" | "code_rename" => {
                let text = bi_str(a, 0, "CODE_RENAME$")?;
                let z = bi_int(a, 1, "CODE_RENAME$")?.max(1) as usize - 1;
                let s = bi_int(a, 2, "CODE_RENAME$")?.max(1) as usize - 1;
                let neu = bi_str(a, 3, "CODE_RENAME$")?;
                Value::str_rc(crate::lsp::umbenennen(text, z, s, neu).unwrap_or_default())
            }
            "code_method_calls$" | "code_method_calls" => {
                let text = bi_str(a, 0, "CODE_METHOD_CALLS$")?.to_string();
                let basis = code_basis(a, 1, "CODE_METHOD_CALLS$")?
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
                let liste: Vec<serde_json::Value> = crate::lsp::methoden_aufrufe(&text, &basis).into_iter()
                    .map(|(z, n, k, sig)| serde_json::json!({"zeile": z, "name": n, "klasse": k, "signatur": sig})).collect();
                Value::str_rc(serde_json::Value::Array(liste).to_string())
            }
            // CODE_CLASSES$: Klassen und Structs des Textes mit ihren
            // Mitgliedern als JSON, ohne zu uebersetzen (lsp::klassen_struktur).
            "code_classes$" | "code_classes" => {
                let text = bi_str(a, 0, "CODE_CLASSES$")?;
                Value::str_rc(crate::lsp::klassen_struktur(text).to_string())
            }
            "code_types$" | "code_types" => {
                let text = bi_str(a, 0, "CODE_TYPES$")?.to_string();
                let basis = if a.len() > 1 { std::path::PathBuf::from(bi_str(a, 1, "CODE_TYPES$")?) }
                            else { std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")) };
                let liste: Vec<serde_json::Value> = crate::lsp::typ_hinweise(&text, &basis).into_iter()
                    .map(|(z, n, t)| serde_json::json!({"zeile": z, "name": n, "typ": t})).collect();
                Value::str_rc(serde_json::Value::Array(liste).to_string())
            }
            // Quelltext einfaerben: (starts, laengen, arten) als TUPLE. Reine
            // Textzerlegung, darum hier und nicht bei der Grafik -- auch ein Bau
            // ohne Fenster (die Website, tools/website.dh) faerbt damit Code.
            // Drei Rueckgaben in einem Aufruf, weil sie IMMER zusammengehoeren
            // -- drei getrennte Aufrufe muessten denselben Text dreimal
            // abtasten und koennten auseinander laufen.
            "syntax_spans" => {
                let src = bi_str(a, 0, "SYNTAX_SPANS")?.to_string();
                let sp = crate::syntax::spans(&src);
                let starts: Vec<i64> = sp.iter().map(|&(s, _, _)| s as i64).collect();
                let laengen: Vec<i64> = sp.iter().map(|&(_, l, _)| l as i64).collect();
                let arten: Vec<String> = sp.iter().map(|&(_, _, a)| a.name().to_string()).collect();
                Value::Tuple(std::rc::Rc::new(vec![
                    crate::builtins::new_int_array(starts),
                    crate::builtins::new_int_array(laengen),
                    crate::builtins::new_string_array(arten),
                ]))
            }
            "code_names" => {
                let text = bi_str(a, 0, "CODE_NAMES")?;
                let mut m = crate::value::DhMap::new("integer".to_string());
                for (k, n) in crate::syntax::namen_zaehlen(text) { m.put(k, Value::Int(n)); }
                Value::Map(std::rc::Rc::new(std::cell::RefCell::new(m)))
            }
            "code_symbols$" | "code_symbols" => {
                fn um(v: &serde_json::Value) -> serde_json::Value {
                    serde_json::json!({
                        "name": v["name"], "art": match (v["kind"].as_i64(), v["detail"].as_str()) {
                            (Some(5), _) => "class", (Some(23), _) => "struct", (Some(7), _) => "property", (Some(10), _) => "enum",
                            (_, Some("sub")) => "sub", _ => "function" },
                        "von": v["range"]["start"]["line"].as_u64().unwrap_or(0) + 1,
                        "bis": v["range"]["end"]["line"].as_u64().unwrap_or(0) + 1,
                        "kinder": v["children"].as_array().map(|k| k.iter().map(um).collect::<Vec<_>>()).unwrap_or_default(),
                    })
                }
                let text = bi_str(a, 0, "CODE_SYMBOLS$")?;
                let g = crate::lsp::gliederung(text);
                let liste: Vec<serde_json::Value> = g.as_array().map(|v| v.iter().map(um).collect()).unwrap_or_default();
                Value::str_rc(&serde_json::Value::Array(liste).to_string())
            }
            // ===== Drucken + Oeffnen (docs/intern/entwurf-drucken.md) =====
            "opendoc" => { crate::drucken::oeffnen(bi_str(a, 0, "OPENDOC")?)?; Value::Nil }
            "printers" => {
                let namen = crate::drucken::drucker_liste()?;
                let n = namen.len() as i64;
                let mut arr = DhArray::new("string".to_string(), vec![n], || Value::str_rc(""));
                for (k, s) in namen.into_iter().enumerate() { arr.cells.set(k, Value::str_rc(&s)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            "printer_default$" | "printer_default" => Value::str_rc(crate::drucken::standard_drucker()?),
            // ===== Fenster als Prozess (docs/intern/entwurf-native-fenster.md, Weg B) =====
            // Ein zweites OS-Fenster ist ein zweiter dhrt mit eigenem SCREEN;
            // die Seiten reden ueber Textzeilen. Kein geteilter Zustand --
            // dieselbe Grenze wie bei TASK_START, aus demselben Grund.
            "window_open" => {
                if a.is_empty() { return Err("WINDOW_OPEN: erwartet die Programmdatei, z.B. WINDOW_OPEN(\"palette.dh\")".into()); }
                let datei = bi_str(a, 0, "WINDOW_OPEN")?.to_string();
                let exe = std::env::current_exe().map_err(|e| format!(
                    "WINDOW_OPEN: eigenen Programmpfad nicht gefunden: {}", e))?;
                let mut args: Vec<String> = Vec::new();
                for (i, wert) in a.iter().enumerate().skip(1) {
                    args.push(match wert {
                        Value::Int(n) => n.to_string(),
                        Value::Str(s) => s.to_string(),
                        Value::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
                        Value::Float(f) => f.to_string(),
                        andere => return Err(format!(
                            "WINDOW_OPEN: Argument {} geht ueber eine Prozessgrenze und muss eine Zahl, \
                             ein Text oder BOOLEAN sein, nicht {}. Fuer alles andere reich JSON durch.",
                            i, andere.type_name())),
                    });
                }
                Value::Int(self.fenster.oeffnen(exe, &datei, &args)?)
            }
            "window_send" => { self.fenster.senden(bi_int(a, 0, "WINDOW_SEND")?, bi_str(a, 1, "WINDOW_SEND")?)?; Value::Nil }
            "window_recv$" | "window_recv" => Value::str_rc(&self.fenster.empfangen(bi_int(a, 0, "WINDOW_RECV$")?)?),
            "window_alive" => Value::Bool(self.fenster.lebt(bi_int(a, 0, "WINDOW_ALIVE")?)?),
            "window_close" => { self.fenster.schliessen(bi_int(a, 0, "WINDOW_CLOSE")?)?; Value::Nil }
            "parent_send" | "parent_recv$" | "parent_recv" | "parent_alive" => {
                if !self.eltern_geprueft {
                    self.eltern = crate::fenster::Eltern::verbinden();
                    self.eltern_geprueft = true;
                }
                match (name, self.eltern.as_ref()) {
                    ("parent_alive", None) => Value::Bool(false),
                    ("parent_alive", Some(e)) => Value::Bool(e.lebt()),
                    (_, None) => return Err(format!(
                        "{}: dieses Programm wurde nicht mit WINDOW_OPEN gestartet -- es hat keine Eltern \
                         (PARENT_ALIVE() sagt das ohne Fehler)", name.to_uppercase())),
                    ("parent_send", Some(e)) => { e.senden(bi_str(a, 0, "PARENT_SEND")?)?; Value::Nil }
                    (_, Some(e)) => Value::str_rc(&e.empfangen()),
                }
            }
            "shell_start" => {
                if a.is_empty() { return Err("SHELL_START: erwartet mind. 1 Argument (Programm)".into()); }
                let prog = bi_str(a, 0, "SHELL_START")?.to_string();
                let mut rest: Vec<String> = Vec::new();
                for i in 1..a.len() { rest.push(bi_str(a, i, "SHELL_START")?.to_string()); }
                Value::Int(self.shell_auftraege.start(&prog.clone(),
                    move || crate::hintergrund::shell_arbeit(prog, rest)))
            }
            // ===== Auftraege mit eigener DH-Funktion (WP H, Weg C) =====
            // Der Auftrag laeuft als eigener dhrt-Prozess, nicht als Thread:
            // `Value` haelt ueberall `Rc`, `Program` ist weder Send noch Sync.
            // Die Grenze ist zugleich die Zusage -- ein Auftrag sieht KEINE
            // Globals des Hauptprogramms, auch keine CONST. Er bekommt mit,
            // was er braucht. Siehe docs/intern/entwurf-task-start.md.
            "task_start" => {
                if a.is_empty() {
                    return Err("TASK_START: erwartet eine Funktion, z.B. TASK_START(Rechne, 42)".into());
                }
                let funktion = match &a[0] {
                    // Eine gebundene Methode kann hier NICHT durch: der Auftrag
                    // laeuft als eigener Prozess, die Instanz existiert dort nicht.
                    Value::BoundMethod(b) => return Err(format!(
                        "TASK_START: '{}' ist an ein Objekt gebunden. Ein Auftrag laeuft in einem eigenen Prozess und sieht dieses Objekt nicht -- nimm eine freie FUNCTION und gib ihr die Werte als Argumente mit.",
                        b.1)),
                    // Ein Lambda mit Kopien ebenso: die Werte gehoeren zu
                    // diesem Prozess. Ohne Kopien ist es eine gewoehnliche
                    // Funktion -- der Auftrag uebersetzt dieselbe Datei und
                    // findet sie unter demselben Namen.
                    Value::Closure(_) => return Err(
                        "TASK_START: dieses Lambda traegt Werte aus seiner Umgebung mit, und ein Auftrag laeuft in einem eigenen Prozess -- gib sie ihm als Argumente: TASK_START(FUNCTION(a, b) ..., a, b)".into()),
                    Value::FuncRef(n) => n.to_string(),
                    Value::Str(s) => s.to_string(),
                    andere => return Err(format!(
                        "TASK_START: erwartet eine Funktion als erstes Argument, \
                         bekommen {}. Schreib den Namen ohne Klammern: \
                         TASK_START(Rechne, 42)", andere.type_name())),
                };
                let datei = match crate::builtins::quelldatei() {
                    Some(d) => d.clone(),
                    None => return Err(
                        "TASK_START: die laufende Datei ist unbekannt -- \
                         Auftraege gibt es nur bei `dhrt run <datei.dh>`".into()),
                };
                let exe = std::env::current_exe().map_err(|e| format!(
                    "TASK_START: eigenen Programmpfad nicht gefunden: {}", e))?;
                // Beliebig viele Argumente -- sie gehen als eigene
                // Kommandozeilen-Worte an `dhrt call`, werden also weder
                // getrennt noch zusammengeklebt. Ein Text mit Leerzeichen
                // bleibt damit EIN Argument.
                let mut args: Vec<String> = Vec::new();
                for (i, wert) in a.iter().enumerate().skip(1) {
                    args.push(match wert {
                        Value::Int(n) => n.to_string(),
                        Value::Str(s) => s.to_string(),
                        Value::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
                        Value::Float(f) => f.to_string(),
                        andere => return Err(format!(
                            "TASK_START: Argument {} geht ueber eine Prozessgrenze \
                             und muss eine Zahl, ein Text oder BOOLEAN sein, nicht \
                             {}. Fuer alles andere reich JSON durch.",
                            i, andere.type_name())),
                    });
                }
                let was = funktion.clone();
                Value::Int(self.task_auftraege.start(&was,
                    move || crate::hintergrund::task_arbeit(exe, datei, funktion, args)))
            }
            "task_ready" => Value::Bool(self.task_auftraege.fertig(bi_int(a, 0, "TASK_READY")?)?),
            // Einmal abholbar, wie bei SHELL_RESULT$: `abholen` nimmt das
            // Ergebnis aus der Verwaltung. Darum gibt es KEIN zweites
            // TASK_OUTPUT$ daneben -- zwei Abholer wuerden sich gegenseitig
            // das Ergebnis wegnehmen, je nachdem wer zuerst fragt. Was der
            // Auftrag gedruckt hat, geht damit verloren: ein Auftrag rechnet,
            // er redet nicht.
            "task_result$" | "task_result" => {
                let id = bi_int(a, 0, "TASK_RESULT$")?;
                match self.task_auftraege.abholen(id)? {
                    Some(Ok(e)) => Value::str_rc(&e.ergebnis),
                    Some(Err(e)) => return Err(e),
                    None => return Err(format!(
                        "TASK_RESULT$: Auftrag {} ist noch nicht fertig -- erst \
                         TASK_READY({}) fragen", id, id)),
                }
            }
            "task_cancel" => { self.task_auftraege.abbrechen(bi_int(a, 0, "TASK_CANCEL")?); Value::Nil }
            "task_pending" => Value::Int(self.task_auftraege.offen()),

            "shell_ready" => Value::Bool(self.shell_auftraege.fertig(bi_int(a, 0, "SHELL_READY")?)?),
            "shell_result$" | "shell_result" => {
                let id = bi_int(a, 0, "SHELL_RESULT$")?;
                match self.shell_auftraege.abholen(id)? {
                    Some(Ok(e)) => {
                        self.shell_letzter_code = e.code;
                        self.shell_letzter_fehler = e.stderr;
                        Value::str_rc(&e.stdout)
                    }
                    Some(Err(msg)) => {
                        self.shell_letzter_code = -1;
                        self.shell_letzter_fehler = msg.clone();
                        return Err(msg);
                    }
                    // Abholen ohne vorheriges SHELL_READY ist der haeufigste
                    // Anfaengerfehler -- die Meldung sagt, was fehlt.
                    None => return Err(format!(
                        "SHELL_RESULT$: Auftrag {} ist nicht fertig (oder schon abgeholt) -- erst SHELL_READY({}) abfragen", id, id)),
                }
            }
            "shell_code" => {
                if !a.is_empty() { return Err("SHELL_CODE: erwartet 0 Argumente (gilt fuer den zuletzt abgeholten Auftrag)".into()); }
                Value::Int(self.shell_letzter_code)
            }
            "shell_err$" | "shell_err" => {
                if !a.is_empty() { return Err("SHELL_ERR$: erwartet 0 Argumente (gilt fuer den zuletzt abgeholten Auftrag)".into()); }
                Value::str_rc(&self.shell_letzter_fehler)
            }
            "shell_cancel" => { self.shell_auftraege.abbrechen(bi_int(a, 0, "SHELL_CANCEL")?); Value::Nil }
            "shell_pending" => Value::Int(self.shell_auftraege.offen()),
            _ => return self.try_hintergrund_db(name, a),
        };
        Ok(Some(v))
    }

    #[cfg(not(feature = "db"))]
    pub(super) fn try_hintergrund_db(&mut self, _name: &str, _a: &[Value]) -> R<Option<Value>> { Ok(None) }

    /// Abfragen im Hintergrund (WP H).
    ///
    /// Der Auftrag oeffnet eine EIGENE Verbindung zur Datei, statt die des
    /// Programms mitzunehmen: eine `rusqlite::Connection` ist `Send`, aber
    /// nicht `Sync` -- sie mitzugeben hiesse, sie dem Hauptthread wegzunehmen,
    /// und der soll ja weiterarbeiten koennen. Preis dafuer: der Auftrag sieht
    /// nur, was schon festgeschrieben ist, nicht die offene Transaktion des
    /// Programms. Steht so in der Doku.
    #[cfg(feature = "db")]
    pub(super) fn try_hintergrund_db(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        let v = match name {
            "db_query_start" => {
                let datei = bi_str(a, 0, "DB_QUERY_START")?.to_string();
                let sql = bi_str(a, 1, "DB_QUERY_START")?.to_string();
                let params = db_params(a.get(2..).unwrap_or(&[]), "DB_QUERY_START")?;
                let kurz = sql.chars().take(40).collect::<String>();
                Value::Int(self.db_auftraege.start(&kurz, move || {
                    // Eine eigene Verbindung -- zur Datei oder zum Server.
                    let mut conn = crate::db::open(&datei)
                        .map_err(|e| e.replacen("DB_OPEN", "DB_QUERY_START", 1))?;
                    conn.query(&sql, &params, "DB_QUERY_START")
                }))
            }
            "db_query_ready" => Value::Bool(self.db_auftraege.fertig(bi_int(a, 0, "DB_QUERY_READY")?)?),
            "db_query_result" => {
                let id = bi_int(a, 0, "DB_QUERY_RESULT")?;
                match self.db_auftraege.abholen(id)? {
                    Some(Ok(res)) => {
                        self.db_results.push(res);
                        Value::Int((self.db_results.len() - 1) as i64)
                    }
                    Some(Err(msg)) => return Err(msg),
                    None => return Err(format!(
                        "DB_QUERY_RESULT: Auftrag {} ist nicht fertig (oder schon abgeholt) -- erst DB_QUERY_READY({}) abfragen", id, id)),
                }
            }
            "db_query_cancel" => { self.db_auftraege.abbrechen(bi_int(a, 0, "DB_QUERY_CANCEL")?); Value::Nil }
            "db_query_pending" => Value::Int(self.db_auftraege.offen()),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Pruefen und Melden (WP E): ASSERT-Familie und LOG_*.
    ///
    /// Braucht VM-Zustand (Zaehler, Sammel-Modus, Quell-Zeile, Ausgabe-Puffer)
    /// und steht darum hier statt in `builtins.rs`.
    pub(super) fn try_pruefen(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("assert") || name.starts_with("log_") || name.starts_with("error_")) {
            return Ok(None);
        }
        let v = match name {
            // WP F -- Angaben zum zuletzt aufgetretenen Fehler. Im CATCH-Zweig
            // gedacht; ausserhalb liefern sie den letzten Stand (0 bzw. "").
            "error_line" => {
                if !a.is_empty() { return Err("ERROR_LINE: erwartet 0 Argumente".into()); }
                Value::Int(self.error_line as i64)
            }
            "error_code$" | "error_code" => {
                if !a.is_empty() { return Err("ERROR_CODE$: erwartet 0 Argumente".into()); }
                Value::str_rc(&self.error_code)
            }
            "assert" => {
                if a.is_empty() || a.len() > 2 {
                    return Err(format!("ASSERT: erwartet 1..2 Argumente (bedingung [, meldung]), erhalten {}", a.len()));
                }
                // Absichtlich streng auf BOOLEAN: `ASSERT(anzahl)` waere sonst
                // still "wahr, weil nicht null" -- und eine Pruefung, die aus
                // Versehen immer durchgeht, ist schlimmer als gar keine.
                let ok = match &a[0] {
                    Value::Bool(b) => *b,
                    andere => return Err(format!(
                        "ASSERT: erwartet BOOLEAN, erhalten {} -- einen Vergleich schreiben, \
                         z.B. ASSERT(anzahl > 0)", andere.type_name())),
                };
                let text = match a.get(1) {
                    Some(v) => crate::builtins::str_of(v),
                    None => String::new(),
                };
                self.assert_werten(ok, if text.is_empty() { "Bedingung nicht erfuellt".to_string() } else { text })?
            }
            "assert_eq" => {
                if a.len() < 2 || a.len() > 3 {
                    return Err(format!("ASSERT_EQ: erwartet 2..3 Argumente (ist, soll [, was]), erhalten {}", a.len()));
                }
                // Dieselbe Gleichheit wie der `=`-Operator der Sprache --
                // eine zweite Vorstellung davon, wann zwei Werte gleich sind,
                // waere die sicherste Art, Vertrauen zu verspielen.
                let ok = crate::value::value_eq(&a[0], &a[1]);
                let was = match a.get(2) {
                    Some(v) => crate::builtins::str_of(v),
                    None => String::new(),
                };
                let meldung = if ok {
                    was
                } else {
                    let kern = format!("erhalten {}, erwartet {}",
                                       crate::builtins::str_of(&a[0]),
                                       crate::builtins::str_of(&a[1]));
                    if was.is_empty() { kern } else { format!("{}: {}", was, kern) }
                };
                self.assert_werten(ok, meldung)?
            }
            "assert_collect" => {
                if a.len() != 1 { return Err(format!("ASSERT_COLLECT: erwartet 1 Argument, erhalten {}", a.len())); }
                self.assert_sammeln = match &a[0] {
                    Value::Bool(b) => *b,
                    andere => return Err(format!("ASSERT_COLLECT: erwartet BOOLEAN, erhalten {}", andere.type_name())),
                };
                Value::Nil
            }
            "assert_count" => { if !a.is_empty() { return Err("ASSERT_COUNT: erwartet 0 Argumente".into()); } Value::Int(self.assert_geprueft) }
            "assert_failed" => { if !a.is_empty() { return Err("ASSERT_FAILED: erwartet 0 Argumente".into()); } Value::Int(self.assert_fehler) }
            "assert_report" => {
                if !a.is_empty() { return Err("ASSERT_REPORT: erwartet 0 Argumente".into()); }
                // Die Bilanz gehoert zu den Nutzdaten (stdout), nicht zu den
                // Meldungen: sie ist das Ergebnis des Pruefprogramms.
                let zeile = if self.assert_fehler == 0 {
                    format!("ALLES GRUEN -- {} Pruefungen", self.assert_geprueft)
                } else {
                    format!("FEHLER: {} von {} Pruefungen", self.assert_fehler, self.assert_geprueft)
                };
                self.out.push_str(&zeile);
                self.out.push('\n');
                Value::Int(self.assert_fehler)
            }
            "log_debug" | "log_info" | "log_warn" | "log_error" => {
                if a.len() != 1 { return Err(format!("{}: erwartet 1 Argument, erhalten {}", name.to_uppercase(), a.len())); }
                let stufe = match name { "log_debug" => 0u8, "log_info" => 1, "log_warn" => 2, _ => 3 };
                if stufe >= self.log_schwelle() {
                    let (_, _, _, h, mi, s) = crate::builtins::local_datetime();
                    let marke = match stufe { 0 => "DEBUG", 1 => "INFO", 2 => "WARN", _ => "ERROR" };
                    let text = crate::builtins::str_of(&a[0]);
                    self.flush_out();
                    use std::io::Write;
                    let se = std::io::stderr();
                    let mut hd = se.lock();
                    let _ = writeln!(hd, "{:02}:{:02}:{:02} {:<5} {}", h, mi, s, marke, text);
                    let _ = hd.flush();
                }
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Eine Pruefung verbuchen. Im Sammel-Modus wird ein Fehlschlag gemeldet
    /// und weitergemacht, sonst bricht er ab (mit Datei:Zeile aus dem
    /// gewohnten Laufzeitfehler-Pfad).
    pub(super) fn assert_werten(&mut self, ok: bool, meldung: String) -> R<Value> {
        self.assert_geprueft += 1;
        if ok { return Ok(Value::Nil); }
        self.assert_fehler += 1;
        if !self.assert_sammeln {
            return Err(format!("ASSERT fehlgeschlagen: {}", meldung));
        }
        // Fehlschlaege gehen nach stderr, damit die Nutzdaten auf stdout
        // sauber bleiben -- ein Pruefprogramm laesst sich so umleiten.
        self.flush_out();
        use std::io::Write;
        let se = std::io::stderr();
        let mut h = se.lock();
        let _ = if self.cur_line != 0 {
            writeln!(h, "FEHL  Zeile {}: {}", self.cur_line, meldung)
        } else {
            writeln!(h, "FEHL  {}", meldung)
        };
        let _ = h.flush();
        Ok(Value::Nil)
    }

    /// Ab welcher Stufe LOG_* etwas ausgibt. Aus `DH_LOG` (debug/info/warn/
    /// error/aus), einmal nachgesehen und dann gemerkt. Vorgabe: `info` --
    /// LOG_DEBUG schweigt also, bis jemand es einschaltet.
    pub(super) fn log_schwelle(&mut self) -> u8 {
        if let Some(p) = self.log_pegel { return p; }
        let p = match std::env::var("DH_LOG").unwrap_or_default().to_ascii_lowercase().as_str() {
            "debug" => 0,
            "warn" => 2,
            "error" => 3,
            "aus" | "off" | "none" => 9,
            _ => 1,          // info (Vorgabe, auch bei unbekanntem Wert)
        };
        self.log_pegel = Some(p);
        p
    }

    /// Gepufferte `PRINT`-Ausgabe sofort auf echtes stdout schreiben und den
    /// Puffer leeren, damit `take_output()` am Ende nichts doppelt schreibt.
    /// Unter dem Profiler NICHT -- dort gehoert stdout dem JSON-Blob (gleiche
    /// Ueberlegung wie in `flush_and_prompt`).
    pub(super) fn flush_out(&mut self) {
        // Im Debugger ist stdout der EREIGNISKANAL (JSON-Zeilen): Programm-
        // Ausgabe geht dort als `output`-Ereignis hinaus (out_sent), nie roh --
        // sonst schriebe DHRT_LIVE eine PRINT-Zeile mitten in den Strom.
        if self.sammeln || self.prof.is_some() || self.dbg.is_some() || self.out.is_empty() { return; }
        use std::io::Write;
        let so = std::io::stdout();
        let mut h = so.lock();
        let _ = h.write_all(self.out.as_bytes());
        let _ = h.flush();
        self.out.clear();
        self.out_zeit = std::time::Instant::now();
    }

    /// Modul `timer` (timer.rs): AFTER/EVERY/CANCEL/UPDATE + COOLDOWN.
    /// Kein Grafik-Bezug -- laeuft auch konsolen-only. TIMER_UPDATE feuert
    /// faellige FUNCREF-Callbacks nach dem gleichen Muster wie GUI_UPDATE.
    pub(super) fn try_timer(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("timer_") || name == "cooldown") { return Ok(None); }
        fn t_int(a: &[Value], i: usize, fn_: &str) -> R<i64> {
            match a.get(i) { Some(Value::Int(n)) => Ok(*n), _ => Err(format!("{}: erwartet INTEGER (Arg {})", fn_, i + 1)) }
        }
        fn t_str<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) { Some(Value::Str(s)) => Ok(s), _ => Err(format!("{}: erwartet STRING (Arg {})", fn_, i + 1)) }
        }
        fn t_func(a: &[Value], i: usize, fn_: &str) -> R<crate::value::Rueckruf> {
            match a.get(i).and_then(crate::value::Rueckruf::aus_wert) {
                Some(cb) => Ok(cb),
                None => Err(format!("{}: erwartet FUNCREF (Arg {})", fn_, i + 1)),
            }
        }
        let r = match name {
            "timer_after" => {
                let ms = t_int(a, 0, "TIMER_AFTER")?;
                if ms < 0 { return Err("TIMER_AFTER: ms muss >= 0 sein".into()); }
                Value::Int(self.timers.after(ms, t_func(a, 1, "TIMER_AFTER")?))
            }
            "timer_every" => {
                let ms = t_int(a, 0, "TIMER_EVERY")?;
                if ms <= 0 { return Err("TIMER_EVERY: ms muss > 0 sein".into()); }
                Value::Int(self.timers.every(ms, t_func(a, 1, "TIMER_EVERY")?))
            }
            "timer_cancel" => { self.timers.cancel(t_int(a, 0, "TIMER_CANCEL")?); Value::Nil }
            "timer_active" => Value::Bool(self.timers.active(t_int(a, 0, "TIMER_ACTIVE")?)),
            "timer_count" => Value::Int(self.timers.count()),
            "timer_clear" => { self.timers.clear(); Value::Nil }
            "timer_update" => {
                // Faellige Callbacks NACH dem Einsammeln feuern -- ein Callback
                // darf selbst Timer registrieren/canceln; neue Eintraege werden
                // erst beim naechsten Update faellig geprueft.
                for cb in self.timers.take_due() {
                    self.rueckruf_rufen(&cb, "TIMER-Callback")?;
                }
                Value::Nil
            }
            "cooldown" => {
                let id = t_str(a, 0, "COOLDOWN")?.to_string();
                let ms = t_int(a, 1, "COOLDOWN")?;
                if ms < 0 { return Err("COOLDOWN: ms muss >= 0 sein".into()); }
                Value::Bool(self.timers.cooldown(&id, ms))
            }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }

    pub(super) fn try_scene(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("scene_")) { return Ok(None); }
        fn sa<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) { Some(Value::Str(s)) => Ok(s), _ => Err(format!("{}: erwartet STRING", fn_)) }
        }
        let r = match name {
            "scene_push" => { self.scene_stack.push((sa(a, 0, "SCENE_PUSH")?.to_string(), HashMap::new())); Value::Nil }
            "scene_pop" => { if self.scene_stack.is_empty() { return Err("SCENE_POP: Stack ist bereits leer".into()); } self.scene_stack.pop(); Value::Nil }
            "scene_switch" => { let n = sa(a, 0, "SCENE_SWITCH")?.to_string(); self.scene_stack.clear(); self.scene_stack.push((n, HashMap::new())); Value::Nil }
            "scene_current" => Value::str_rc(self.scene_stack.last().map(|(n, _)| n.as_str()).unwrap_or("")),
            "scene_depth" => Value::Int(self.scene_stack.len() as i64),
            "scene_has" => { let n = sa(a, 0, "SCENE_HAS")?; Value::Bool(self.scene_stack.iter().any(|(x, _)| x == n)) }
            "scene_reset" => { self.scene_stack.clear(); Value::Nil }
            "scene_set_int" | "scene_set_float" | "scene_set_string" | "scene_set_bool" => {
                let key = sa(a, 0, "SCENE_SET")?.to_string();
                let v = match name {
                    "scene_set_int" => match a.get(1) { Some(Value::Int(i)) => Value::Int(*i), _ => return Err("SCENE_SET_INT: value muss INTEGER sein".into()) },
                    "scene_set_float" => match a.get(1) { Some(Value::Int(i)) => Value::Float(*i as f64), Some(Value::Float(f)) => Value::Float(*f), _ => return Err("SCENE_SET_FLOAT: value muss Zahl sein".into()) },
                    "scene_set_string" => match a.get(1) { Some(Value::Str(s)) => Value::Str(s.clone()), _ => return Err("SCENE_SET_STRING: value muss STRING sein".into()) },
                    _ => match a.get(1) { Some(Value::Bool(b)) => Value::Bool(*b), _ => return Err("SCENE_SET_BOOL: value muss BOOLEAN sein".into()) },
                };
                self.scene_top_mut()?.1.insert(key, v); Value::Nil
            }
            "scene_get_int" | "scene_get_float" | "scene_get_string" | "scene_get_bool" => {
                let key = sa(a, 0, "SCENE_GET")?;
                let data = &self.scene_top()?.1;
                let val = data.get(key).ok_or_else(|| format!("{}: Key '{}' nicht in Scene", name.to_uppercase(), key))?;
                let ok = scene_type_ok(val, name);
                if !ok { return Err(format!("{}: Key '{}' hat falschen Typ", name.to_uppercase(), key)); }
                val.clone()
            }
            "scene_get_int_or" | "scene_get_float_or" | "scene_get_string_or" | "scene_get_bool_or" => {
                let key = sa(a, 0, "SCENE_GET_OR")?;
                let def = a.get(1).cloned().unwrap_or(Value::Nil);
                let data = &self.scene_top()?.1;
                match data.get(key) { Some(v) if scene_type_ok(v, name) => v.clone(), _ => def }
            }
            "scene_has_key" => { let key = sa(a, 0, "SCENE_HAS_KEY")?; Value::Bool(self.scene_top()?.1.contains_key(key)) }
            "scene_delete" => { let key = sa(a, 0, "SCENE_DELETE")?.to_string(); self.scene_top_mut()?.1.remove(&key); Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }
}
