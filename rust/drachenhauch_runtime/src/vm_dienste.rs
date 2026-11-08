//! Befehle der Daten- und Netzdienste: db, net, geld, smtp, xlsx, pdf, httpd, mqtt, html (HTTP) und cloud.
//!
//! Teil von `vm.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl<'p> Vm<'p> {
    /// Modul `scene` (globaler Stack-State, kein Grafik-Bezug).
    // ===================================================================
    // Modul db (SQLite, Feature `db`)
    // ===================================================================
    pub(super) fn try_db(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("db_")) { return Ok(None); }
        #[cfg(feature = "db")]
        { return self.try_db_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    #[cfg(feature = "db")]
    pub(super) fn db_conn(&mut self, idx: i64) -> R<&mut crate::db::Verbindung> {
        Self::handle_get_mut(&mut self.db_conns, idx, "DB", "ungueltiges/geschlossenes DB_CONN-Handle")
    }
    #[cfg(feature = "db")]
    pub(super) fn db_res(&self, idx: i64) -> R<&crate::db::DbResult> {
        let r = self.db_results.get(idx as usize)
            .ok_or_else(|| format!("DB: ungueltiges DB_RESULT-Handle {}", idx))?;
        if r.closed { return Err("DB: Result wurde bereits geschlossen".into()); }
        Ok(r)
    }
    #[cfg(feature = "db")]
    pub(super) fn db_res_mut(&mut self, idx: i64) -> R<&mut crate::db::DbResult> {
        let r = self.db_results.get_mut(idx as usize)
            .ok_or_else(|| format!("DB: ungueltiges DB_RESULT-Handle {}", idx))?;
        if r.closed { return Err("DB: Result wurde bereits geschlossen".into()); }
        Ok(r)
    }

    #[cfg(feature = "db")]
    pub(super) fn try_db_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::db;
        let v = match name {
            "db_open" => {
                let conn = db::open(bi_str(a, 0, "DB_OPEN")?)?;
                self.db_conns.push(Some(conn));
                Value::Int((self.db_conns.len() - 1) as i64)
            }
            "db_close" => {
                let i = bi_int(a, 0, "DB_CLOSE")? as usize;
                if let Some(s) = self.db_conns.get_mut(i) { *s = None; }
                Value::Nil
            }
            "db_last_rowid" => Value::Int(self.db_conn(bi_int(a, 0, "DB_LAST_ROWID")?)?.letzte_id()?),
            "db_kind$" => Value::str_rc(self.db_conn(bi_int(a, 0, "DB_KIND$")?)?.art()),
            "db_ping" => Value::Bool(self.db_conn(bi_int(a, 0, "DB_PING")?)?.lebt()),
            "db_exec" => {
                let params = db_params(a.get(2..).unwrap_or(&[]), "DB_EXEC")?;
                let sql = bi_str(a, 1, "DB_EXEC")?.to_string();
                Value::Int(self.db_conn(bi_int(a, 0, "DB_EXEC")?)?.exec(&sql, &params, "DB_EXEC")?)
            }
            "db_query" => {
                let params = db_params(a.get(2..).unwrap_or(&[]), "DB_QUERY")?;
                let sql = bi_str(a, 1, "DB_QUERY")?.to_string();
                let res = self.db_conn(bi_int(a, 0, "DB_QUERY")?)?.query(&sql, &params, "DB_QUERY")?;
                self.db_results.push(res);
                Value::Int((self.db_results.len() - 1) as i64)
            }
            "db_next" => {
                let r = self.db_res_mut(bi_int(a, 0, "DB_NEXT")?)?;
                r.pos += 1;
                Value::Bool((r.pos as usize) < r.rows.len())
            }
            "db_col_count" => Value::Int(self.db_res(bi_int(a, 0, "DB_COL_COUNT")?)?.columns.len() as i64),
            "db_col_name" => { let i1 = bi_int(a, 1, "DB_COL_NAME")?; Value::str_rc(&self.db_res(bi_int(a, 0, "DB_COL_NAME")?)?.col_name(i1)?) }
            "db_close_result" => {
                let r = self.db_res_mut(bi_int(a, 0, "DB_CLOSE_RESULT")?)?;
                r.closed = true;
                // rows leert sich NICHT von selbst durch das closed-Flag -- ohne
                // das hier bleiben eager geladene Zeilen bis Programmende im
                // Speicher, auch wenn artig geschlossen wird.
                r.rows = Vec::new();
                Value::Nil
            }
            "db_is_null" => { let i1 = bi_int(a, 1, "DB_IS_NULL")?; Value::Bool(self.db_res(bi_int(a, 0, "DB_IS_NULL")?)?.is_null(i1)?) }
            "db_get_buffer" => { let i1 = bi_int(a, 1, "DB_GET_BUFFER")?; Value::Buffer(Rc::new(RefCell::new(self.db_res(bi_int(a, 0, "DB_GET_BUFFER")?)?.get_buffer(i1)?))) }
            "db_get_string" => { let i1 = bi_int(a, 1, "DB_GET_STRING")?; Value::str_rc(&self.db_res(bi_int(a, 0, "DB_GET_STRING")?)?.get_string(i1)?) }
            "db_get_int" => { let i1 = bi_int(a, 1, "DB_GET_INT")?; Value::Int(self.db_res(bi_int(a, 0, "DB_GET_INT")?)?.get_int(i1)?) }
            "db_get_float" => { let i1 = bi_int(a, 1, "DB_GET_FLOAT")?; Value::Float(self.db_res(bi_int(a, 0, "DB_GET_FLOAT")?)?.get_float(i1)?) }
            "db_get_bool" => { let i1 = bi_int(a, 1, "DB_GET_BOOL")?; Value::Bool(self.db_res(bi_int(a, 0, "DB_GET_BOOL")?)?.get_bool(i1)?) }
            "db_begin" => { self.db_conn(bi_int(a, 0, "DB_BEGIN")?)?.befehl("BEGIN", "DB_BEGIN")?; Value::Nil }
            "db_commit" => { self.db_conn(bi_int(a, 0, "DB_COMMIT")?)?.befehl("COMMIT", "DB_COMMIT")?; Value::Nil }
            "db_rollback" => { self.db_conn(bi_int(a, 0, "DB_ROLLBACK")?)?.befehl("ROLLBACK", "DB_ROLLBACK")?; Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul net (std::net, Feature `net`)
    // ===================================================================
    pub(super) fn try_net(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("net_")) { return Ok(None); }
        #[cfg(feature = "net")]
        { return self.try_net_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    #[cfg(feature = "net")]
    pub(super) fn net_listener(&self, idx: i64) -> R<&(std::net::TcpListener, i64)> {
        Self::handle_get(&self.tcp_listeners, idx, "NET", "ungueltiges/geschlossenes NET_LISTENER-Handle")
    }
    #[cfg(feature = "net")]
    pub(super) fn net_sock(&self, idx: i64) -> R<&crate::net::NetSock> {
        Self::handle_get(&self.tcp_socks, idx, "NET", "ungueltiges/geschlossenes NET_SOCKET-Handle")
    }
    #[cfg(feature = "net")]
    pub(super) fn net_sock_mut(&mut self, idx: i64) -> R<&mut crate::net::NetSock> {
        Self::handle_get_mut(&mut self.tcp_socks, idx, "NET", "ungueltiges/geschlossenes NET_SOCKET-Handle")
    }
    #[cfg(feature = "net")]
    pub(super) fn net_udp(&self, idx: i64) -> R<&crate::net::UdpSock> {
        Self::handle_get(&self.udp_socks, idx, "NET", "ungueltiges/geschlossenes NET_UDP-Handle")
    }
    #[cfg(feature = "net")]
    pub(super) fn net_udp_mut(&mut self, idx: i64) -> R<&mut crate::net::UdpSock> {
        Self::handle_get_mut(&mut self.udp_socks, idx, "NET", "ungueltiges/geschlossenes NET_UDP-Handle")
    }

    #[cfg(feature = "net")]
    pub(super) fn try_net_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::net;
        let v = match name {
            "net_tcp_listen" => {
                let bind_addr = match a.get(1) { Some(Value::Str(s)) => s.as_ref(), _ => "" };
                let (l, port) = net::listen(bi_int(a, 0, "NET_TCP_LISTEN")?, bind_addr)?;
                self.tcp_listeners.push(Some((l, port)));
                Value::Int((self.tcp_listeners.len() - 1) as i64)
            }
            "net_listener_port" => Value::Int(self.net_listener(bi_int(a, 0, "NET_LISTENER_PORT")?)?.1),
            "net_tcp_accept" => {
                let i = bi_int(a, 0, "NET_TCP_ACCEPT")?;
                let res = { let l = &self.net_listener(i)?.0; net::accept(l)? };
                match res {
                    Some(sock) => { self.tcp_socks.push(Some(sock)); Value::Int((self.tcp_socks.len() - 1) as i64) }
                    None => Value::Nil,
                }
            }
            "net_tcp_connect" => {
                let host = bi_str(a, 0, "NET_TCP_CONNECT")?.to_string();
                let s = net::connect(&host, bi_int(a, 1, "NET_TCP_CONNECT")?)?;
                self.tcp_socks.push(Some(s));
                Value::Int((self.tcp_socks.len() - 1) as i64)
            }
            "net_send" => {
                let i = bi_int(a, 0, "NET_SEND")?;
                // Text oder ein BUFFER -- Bytes, die kein UTF-8 sind, liessen sich
                // sonst gar nicht verschicken.
                match a.get(1) {
                    Some(Value::Buffer(b)) => { let bytes = b.borrow().clone(); Value::Int(net::send_bytes(self.net_sock_mut(i)?, &bytes)?) }
                    _ => { let t = bi_str(a, 1, "NET_SEND")?.to_string(); Value::Int(net::send(self.net_sock_mut(i)?, &t)?) }
                }
            }
            "net_recv_bytes" => {
                let i = bi_int(a, 0, "NET_RECV_BYTES")?;
                let n = bi_int(a, 1, "NET_RECV_BYTES")?;
                let bytes = net::recv_bytes(self.net_sock_mut(i)?, n)?;
                Value::Buffer(std::rc::Rc::new(std::cell::RefCell::new(bytes)))
            }
            "net_recv" => { let i = bi_int(a, 0, "NET_RECV")?; let n = bi_int(a, 1, "NET_RECV")?; Value::str_rc(net::recv(self.net_sock_mut(i)?, n)?) }
            "net_peer_addr" => Value::str_rc(&self.net_sock(bi_int(a, 0, "NET_PEER_ADDR")?)?.peer_host),
            "net_peer_port" => Value::Int(self.net_sock(bi_int(a, 0, "NET_PEER_PORT")?)?.peer_port),
            "net_is_connected" => Value::Bool(net::is_connected(self.net_sock(bi_int(a, 0, "NET_IS_CONNECTED")?)?)),
            "net_close" => { let i = bi_int(a, 0, "NET_CLOSE")? as usize; if let Some(s) = self.tcp_socks.get_mut(i) { *s = None; } Value::Nil }
            "net_close_listener" => { let i = bi_int(a, 0, "NET_CLOSE_LISTENER")? as usize; if let Some(s) = self.tcp_listeners.get_mut(i) { *s = None; } Value::Nil }
            "net_set_timeout" => { let i = bi_int(a, 0, "NET_SET_TIMEOUT")?; let ms = bi_int(a, 1, "NET_SET_TIMEOUT")?; net::set_timeout_tcp(&self.net_sock(i)?.stream, ms); Value::Nil }
            "net_udp_bind" => {
                let bind_addr = match a.get(1) { Some(Value::Str(s)) => s.as_ref(), _ => "" };
                let s = net::udp_bind(bi_int(a, 0, "NET_UDP_BIND")?, bind_addr)?;
                self.udp_socks.push(Some(s));
                Value::Int((self.udp_socks.len() - 1) as i64)
            }
            "net_udp_open" => { let s = net::udp_open()?; self.udp_socks.push(Some(s)); Value::Int((self.udp_socks.len() - 1) as i64) }
            "net_udp_port" => Value::Int(self.net_udp(bi_int(a, 0, "NET_UDP_PORT")?)?.bound_port),
            "net_udp_send" => {
                let i = bi_int(a, 0, "NET_UDP_SEND")?;
                let h = bi_str(a, 1, "NET_UDP_SEND")?.to_string();
                let p = bi_int(a, 2, "NET_UDP_SEND")?;
                // Text oder ein BUFFER -- ein Spielstand mit Kommazahlen ist kein UTF-8.
                match a.get(3) {
                    Some(Value::Buffer(b)) => { let bytes = b.borrow().clone(); Value::Int(net::udp_send_bytes(self.net_udp(i)?, &h, p, &bytes)?) }
                    _ => { let t = bi_str(a, 3, "NET_UDP_SEND")?.to_string(); Value::Int(net::udp_send(self.net_udp(i)?, &h, p, &t)?) }
                }
            }
            "net_udp_recv_bytes" => {
                let i = bi_int(a, 0, "NET_UDP_RECV_BYTES")?;
                let n = bi_int(a, 1, "NET_UDP_RECV_BYTES")?;
                let bytes = net::udp_recv_bytes(self.net_udp_mut(i)?, n, "NET_UDP_RECV_BYTES")?;
                Value::Buffer(std::rc::Rc::new(std::cell::RefCell::new(bytes)))
            }
            // Absender getrennt: "::1:5000" laesst sich nur zerlegen, wenn man
            // weiss, dass am LETZTEN Doppelpunkt geschnitten werden muss.
            "net_udp_last_host$" | "net_udp_last_host" => Value::str_rc(&self.net_udp(bi_int(a, 0, "NET_UDP_LAST_HOST$")?)?.last_from.0),
            "net_udp_last_port" => Value::Int(self.net_udp(bi_int(a, 0, "NET_UDP_LAST_PORT")?)?.last_from.1),
            "net_udp_recv" => { let i = bi_int(a, 0, "NET_UDP_RECV")?; let n = bi_int(a, 1, "NET_UDP_RECV")?; Value::str_rc(net::udp_recv(self.net_udp_mut(i)?, n)?) }
            "net_udp_last_from" => {
                let s = self.net_udp(bi_int(a, 0, "NET_UDP_LAST_FROM")?)?;
                if s.last_from.0.is_empty() { Value::str_rc("") } else { Value::str_rc(format!("{}:{}", s.last_from.0, s.last_from.1)) }
            }
            "net_udp_set_timeout" => { let i = bi_int(a, 0, "NET_UDP_SET_TIMEOUT")?; let ms = bi_int(a, 1, "NET_UDP_SET_TIMEOUT")?; net::set_timeout_udp(&self.net_udp(i)?.sock, ms); Value::Nil }
            "net_udp_close" => { let i = bi_int(a, 0, "NET_UDP_CLOSE")? as usize; if let Some(s) = self.udp_socks.get_mut(i) { *s = None; } Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul mqtt (baut auf std::net auf, Feature `net`)
    // ===================================================================
    /// Modul `geld` -- ein Betrag als eigener Wert (Weg C des Entwurfs).
    pub(super) fn try_geld(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("geld_") { return Ok(None); }
        use crate::geld;
        let wert = |v: &Value, fn_: &str| -> R<i64> {
            match v {
                Value::Geld(w) => Ok(*w),
                other => Err(format!("{}: erwartet GELD, erhalten {} -- mit GELD_NEU() \
                                      einen Betrag daraus machen", fn_, other.type_name())),
            }
        };
        let v = match name {
            "geld_neu" => {
                if a.len() != 1 { return Err(format!("GELD_NEU: erwartet 1 Argument, erhalten {}", a.len())); }
                let w = match &a[0] {
                    Value::Str(s) => geld::aus_text(s).map_err(|e| format!("GELD_NEU: {}", e))?,
                    Value::Geld(w) => *w,
                    v if is_num(v) => geld::aus_zahl(as_f64(v)).map_err(|e| format!("GELD_NEU: {}", e))?,
                    v => return Err(format!("GELD_NEU: erwartet Text oder Zahl, erhalten {}", v.type_name())),
                };
                Value::Geld(w)
            }
            "geld_aus_cent" => { Value::Geld(bi_int(a, 0, "GELD_AUS_CENT")?
                .checked_mul(100).ok_or_else(|| "GELD_AUS_CENT: Betrag zu gross".to_string())?) }
            "geld_cent" => Value::Int(geld::runden(wert(&a[0], "GELD_CENT")?, 2) / 100),
            "geld_zahl" => Value::Float(wert(&a[0], "GELD_ZAHL")? as f64 / geld::SKALA as f64),
            "geld_runden" => {
                let w = wert(&a[0], "GELD_RUNDEN")?;
                let stellen = match a.get(1) {
                    None => 2,
                    Some(v) => {
                        let n = bi_int(a, 1, "GELD_RUNDEN")?;
                        let _ = v;
                        if !(0..=4).contains(&n) {
                            return Err("GELD_RUNDEN: 0 bis 4 Nachkommastellen \
                                        (mehr haelt ein Betrag nicht)".to_string());
                        }
                        n as u32
                    }
                };
                Value::Geld(geld::runden(w, stellen))
            }
            "geld_text$" => {
                let w = wert(&a[0], "GELD_TEXT$")?;
                let symbol = match a.get(1) {
                    Some(Value::Str(s)) => s.to_string(),
                    None => "\u{20ac}".to_string(),
                    Some(v) => return Err(format!("GELD_TEXT$: das Symbol muss ein STRING sein, \
                                                   erhalten {}", v.type_name())),
                };
                Value::str_rc(geld::text(w, &symbol, 2))
            }
            "geld_teilen" => {
                let w = wert(&a[0], "GELD_TEILEN")?;
                let n = bi_int(a, 1, "GELD_TEILEN")?;
                let teile = geld::teilen(w, n)?;
                crate::builtins::new_geld_array(teile)
            }
            "geld_abs" => Value::Geld(wert(&a[0], "GELD_ABS")?.abs()),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Modul `smtp` -- E-Mail verschicken.
    pub(super) fn try_smtp(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("smtp_") { return Ok(None); }
        #[cfg(feature = "smtp")]
        { return self.try_smtp_impl(name, a); }
        #[allow(unreachable_code)]
        {
            let _ = a;
            Err(format!("{}: diese Fassung von dhrt ist ohne das Feature `smtp` \
                         gebaut (verschluesselte Verbindungen fehlen dann)",
                        name.to_uppercase()))
        }
    }

    #[cfg(feature = "smtp")]
    pub(super) fn smtp_m(&mut self, idx: i64) -> R<&mut crate::smtp::Mail> {
        Self::handle_get_mut(&mut self.smtp_mail, idx, "SMTP",
                             "ungueltiges/geschlossenes SMTP-Handle")
    }

    /// Adresse plus optionaler Anzeigename -- dieselbe Form bei FROM/TO/CC.
    #[cfg(feature = "smtp")]
    pub(super) fn smtp_wer(a: &[Value], fn_: &str) -> R<(String, String)> {
        let adresse = bi_str(a, 1, fn_)?.trim().to_string();
        crate::smtp::pruefe_adresse(&adresse, fn_).map_err(|e| format!("{}: {}", fn_, e))?;
        let name = match a.get(2) {
            Some(Value::Str(s)) => s.to_string(),
            None => String::new(),
            Some(_) => return Err(format!("{}: der Anzeigename muss ein STRING sein", fn_)),
        };
        crate::smtp::pruefe_kopfwert(&name, &format!("{}: der Anzeigename", fn_))?;
        Ok((adresse, name))
    }

    #[cfg(feature = "smtp")]
    pub(super) fn try_smtp_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::smtp::{self, Sicherheit};
        let v = match name {
            "smtp_new" => {
                self.smtp_mail.push(Some(smtp::Mail::neu()));
                Value::Int((self.smtp_mail.len() - 1) as i64)
            }
            "smtp_server" => {
                let i = bi_int(a, 0, "SMTP_SERVER")?;
                let host = bi_str(a, 1, "SMTP_SERVER")?.trim().to_string();
                let port = bi_int(a, 2, "SMTP_SERVER")?;
                if !(1..=65535).contains(&port) {
                    return Err(format!("SMTP_SERVER: {} ist keine Portnummer", port));
                }
                let sicher = match a.get(3) {
                    Some(Value::Str(s)) => Sicherheit::aus_name(s)
                        .map_err(|e| format!("SMTP_SERVER: {}", e))?,
                    None => Sicherheit::aus_port(port as u16),
                    Some(_) => return Err("SMTP_SERVER: die Sicherheit muss ein STRING sein \
                                           (\"starttls\", \"tls\", \"keine\")".into()),
                };
                let m = self.smtp_m(i)?;
                m.host = host;
                m.port = port as u16;
                m.sicherheit = sicher;
                Value::Nil
            }
            "smtp_login" => {
                let i = bi_int(a, 0, "SMTP_LOGIN")?;
                let b = bi_str(a, 1, "SMTP_LOGIN")?.to_string();
                let k = bi_str(a, 2, "SMTP_LOGIN")?.to_string();
                let m = self.smtp_m(i)?;
                m.benutzer = b;
                m.kennwort = k;
                Value::Nil
            }
            "smtp_from" => {
                let i = bi_int(a, 0, "SMTP_FROM")?;
                let wer = Self::smtp_wer(a, "SMTP_FROM")?;
                self.smtp_m(i)?.von = wer;
                Value::Nil
            }
            "smtp_to" => {
                let i = bi_int(a, 0, "SMTP_TO")?;
                let wer = Self::smtp_wer(a, "SMTP_TO")?;
                self.smtp_m(i)?.an.push(wer);
                Value::Nil
            }
            "smtp_cc" => {
                let i = bi_int(a, 0, "SMTP_CC")?;
                let wer = Self::smtp_wer(a, "SMTP_CC")?;
                self.smtp_m(i)?.cc.push(wer);
                Value::Nil
            }
            "smtp_bcc" => {
                let i = bi_int(a, 0, "SMTP_BCC")?;
                let adresse = bi_str(a, 1, "SMTP_BCC")?.trim().to_string();
                smtp::pruefe_adresse(&adresse, "SMTP_BCC").map_err(|e| format!("SMTP_BCC: {}", e))?;
                self.smtp_m(i)?.bcc.push(adresse);
                Value::Nil
            }
            "smtp_subject" => {
                let i = bi_int(a, 0, "SMTP_SUBJECT")?;
                let s = bi_str(a, 1, "SMTP_SUBJECT")?.to_string();
                smtp::pruefe_kopfwert(&s, "SMTP_SUBJECT: der Betreff")?;
                self.smtp_m(i)?.betreff = s;
                Value::Nil
            }
            "smtp_text" => {
                let i = bi_int(a, 0, "SMTP_TEXT")?;
                let s = bi_str(a, 1, "SMTP_TEXT")?.to_string();
                self.smtp_m(i)?.text = s;
                Value::Nil
            }
            "smtp_html" => {
                let i = bi_int(a, 0, "SMTP_HTML")?;
                let s = bi_str(a, 1, "SMTP_HTML")?.to_string();
                self.smtp_m(i)?.html = s;
                Value::Nil
            }
            "smtp_attach" => {
                let i = bi_int(a, 0, "SMTP_ATTACH")?;
                let pfad = bi_str(a, 1, "SMTP_ATTACH")?.to_string();
                let daten = std::fs::read(&pfad)
                    .map_err(|e| format!("SMTP_ATTACH: {} ({})", e, pfad))?;
                let anzeige = match a.get(2) {
                    Some(Value::Str(s)) => s.to_string(),
                    None => std::path::Path::new(&pfad).file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| pfad.clone()),
                    Some(_) => return Err("SMTP_ATTACH: der Anzeigename muss ein STRING sein".into()),
                };
                smtp::pruefe_kopfwert(&anzeige, "SMTP_ATTACH: der Dateiname")?;
                if anzeige.contains('"') {
                    return Err("SMTP_ATTACH: ein Anfuehrungszeichen im Dateinamen wuerde \
                                die Kopfzeile zerreissen".into());
                }
                // Die Art wird an der Endung erkannt -- dieselbe Tabelle, die
                // auch der Webserver benutzt, damit es nur EINE gibt.
                let typ = crate::httpd::typ_aus_endung(&pfad).to_string();
                self.smtp_m(i)?.anhaenge.push(smtp::Anhang { name: anzeige, typ, daten });
                Value::Nil
            }
            "smtp_timeout" => {
                let i = bi_int(a, 0, "SMTP_TIMEOUT")?;
                let ms = bi_int(a, 1, "SMTP_TIMEOUT")?;
                if ms <= 0 { return Err("SMTP_TIMEOUT: die Frist muss groesser als 0 sein".into()); }
                self.smtp_m(i)?.frist_ms = ms as u64;
                Value::Nil
            }
            "smtp_message$" => {
                let i = bi_int(a, 0, "SMTP_MESSAGE$")?;
                let (datum, zufall) = (smtp::rfc_datum(smtp::jetzt_utc()), smtp::zufallszahl());
                let m = self.smtp_m(i)?;
                let n = smtp::nachricht(m, &datum, zufall)
                    .map_err(|e| format!("SMTP_MESSAGE$: {}", e))?;
                Value::str_rc(&n)
            }
            "smtp_send" => {
                let i = bi_int(a, 0, "SMTP_SEND")?;
                let (datum, zufall) = (smtp::rfc_datum(smtp::jetzt_utc()), smtp::zufallszahl());
                let m = self.smtp_m(i)?;
                let n = smtp::nachricht(m, &datum, zufall)
                    .map_err(|e| format!("SMTP_SEND: {}", e))?;
                smtp::senden(m, &n)?;
                Value::Nil
            }
            "smtp_close" => {
                let i = bi_int(a, 0, "SMTP_CLOSE")? as usize;
                if let Some(slot) = self.smtp_mail.get_mut(i) { *slot = None; }
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Modul `xlsx` -- Auswertungen als Excel-Mappe.
    pub(super) fn try_xlsx(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("xlsx_") { return Ok(None); }
        use crate::xlsx::{self, Wert};
        let v = match name {
            "xlsx_new" => {
                let blatt = match a.first() { Some(Value::Str(s)) => s.to_string(), _ => "Tabelle1".to_string() };
                let m = xlsx::Mappe::neu(&blatt).map_err(|e| format!("XLSX_NEW: {}", e))?;
                self.xlsx_mappe.push(Some(m));
                Value::Int((self.xlsx_mappe.len() - 1) as i64)
            }
            "xlsx_sheet" => {
                let i = bi_int(a, 0, "XLSX_SHEET")?;
                let n = bi_str(a, 1, "XLSX_SHEET")?.to_string();
                self.xlsx_m(i)?.blatt_dazu(&n).map_err(|e| format!("XLSX_SHEET: {}", e))?;
                Value::Nil
            }
            "xlsx_sheet_count" => { let i = bi_int(a, 0, "XLSX_SHEET_COUNT")?; Value::Int(self.xlsx_m(i)?.blaetter.len() as i64) }
            "xlsx_set" => {
                let (i, z, s) = Self::xlsx_zelle(a, "XLSX_SET")?;
                let t = bi_str(a, 3, "XLSX_SET")?.to_string();
                self.xlsx_m(i)?.setze(z, s, Wert::Text(t));
                Value::Nil
            }
            "xlsx_set_num" => {
                let (i, z, s) = Self::xlsx_zelle(a, "XLSX_SET_NUM")?;
                let n = bi_num(a, 3, "XLSX_SET_NUM")?;
                self.xlsx_m(i)?.setze(z, s, Wert::Zahl(n));
                Value::Nil
            }
            "xlsx_set_date" => {
                let (i, z, s) = Self::xlsx_zelle(a, "XLSX_SET_DATE")?;
                // Sekunden wie im Modul `zeit` -- so passt FILETIME und
                // ZEIT_JETZT ohne Umrechnung hinein.
                let sek = bi_int(a, 3, "XLSX_SET_DATE")?;
                let muster = match a.get(4) { Some(Value::Str(s)) => s.to_string(),
                                              _ => "DD.MM.YYYY".to_string() };
                let m = self.xlsx_m(i)?;
                m.setze(z, s, Wert::Zahl(xlsx::zeit_zu_excel(sek)));
                // Ohne Format staende in der Zelle die nackte Tageszahl --
                // ein Datum ist in Excel eine Zahl MIT Formatierung.
                m.setze_format(z, s, &muster);
                Value::Nil
            }
            "xlsx_bold" => {
                let (i, z, s) = Self::xlsx_zelle(a, "XLSX_BOLD")?;
                let an = match a.get(3) { Some(Value::Bool(b)) => *b, None => true,
                                          Some(_) => return Err("XLSX_BOLD: 4. Argument muss BOOLEAN sein".into()) };
                self.xlsx_m(i)?.setze_fett(z, s, an);
                Value::Nil
            }
            "xlsx_bold_row" => {
                let i = bi_int(a, 0, "XLSX_BOLD_ROW")?;
                let z = Self::xlsx_index(a, 1, "XLSX_BOLD_ROW", "Zeile")?;
                let an = match a.get(2) { Some(Value::Bool(b)) => *b, None => true,
                                          Some(_) => return Err("XLSX_BOLD_ROW: 3. Argument muss BOOLEAN sein".into()) };
                self.xlsx_m(i)?.setze_zeile_fett(z, an);
                Value::Nil
            }
            "xlsx_format" => {
                let (i, z, s) = Self::xlsx_zelle(a, "XLSX_FORMAT")?;
                let f = bi_str(a, 3, "XLSX_FORMAT")?.to_string();
                self.xlsx_m(i)?.setze_format(z, s, &f);
                Value::Nil
            }
            "xlsx_col_width" => {
                let i = bi_int(a, 0, "XLSX_COL_WIDTH")?;
                let s = Self::xlsx_index(a, 1, "XLSX_COL_WIDTH", "Spalte")?;
                let w = bi_num(a, 2, "XLSX_COL_WIDTH")?;
                self.xlsx_m(i)?.setze_breite(s, w.max(0.0));
                Value::Nil
            }
            "xlsx_save" => {
                let i = bi_int(a, 0, "XLSX_SAVE")?;
                let pfad = bi_str(a, 1, "XLSX_SAVE")?.to_string();
                let teile = crate::xlsx::teile(self.xlsx_m(i)?);
                crate::zipdatei::packe_daten(std::path::Path::new(&pfad), &teile)
                    .map_err(|e| format!("XLSX_SAVE: {}", e))?;
                Value::Nil
            }
            "xlsx_close" => {
                let i = bi_int(a, 0, "XLSX_CLOSE")? as usize;
                if let Some(slot) = self.xlsx_mappe.get_mut(i) { *slot = None; }
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub(super) fn xlsx_m(&mut self, idx: i64) -> R<&mut crate::xlsx::Mappe> {
        Self::handle_get_mut(&mut self.xlsx_mappe, idx, "XLSX", "ungueltiges/geschlossenes XLSX-Handle")
    }

    /// Zeile und Spalte einer Zelle -- beide 0-basiert wie ueberall in der
    /// Sprache, auch wenn Excel selbst bei 1 (und bei "A") anfaengt.
    pub(super) fn xlsx_zelle(a: &[Value], fn_: &str) -> R<(i64, u32, u32)> {
        Ok((bi_int(a, 0, fn_)?,
            Self::xlsx_index(a, 1, fn_, "Zeile")?,
            Self::xlsx_index(a, 2, fn_, "Spalte")?))
    }

    pub(super) fn xlsx_index(a: &[Value], i: usize, fn_: &str, was: &str) -> R<u32> {
        let n = bi_int(a, i, fn_)?;
        if n < 0 { return Err(format!("{}: {} {} ist negativ", fn_, was, n)); }
        // Excels eigene Grenzen: 1 048 576 Zeilen, 16 384 Spalten.
        let grenze = if was == "Zeile" { 1_048_575 } else { 16_383 };
        if n > grenze {
            return Err(format!("{}: {} {} liegt jenseits dessen, was Excel kann (bis {})",
                               fn_, was, n, grenze));
        }
        Ok(n as u32)
    }

    /// Modul `pdf` -- druckfertige Seiten schreiben.
    ///
    /// In vm.rs, weil ein offenes Dokument VM-Zustand ist (wie die
    /// Webserver daneben).
    pub(super) fn try_pdf(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("pdf_") { return Ok(None); }
        #[cfg(feature = "pdf")]
        { return self.try_pdf_impl(name, a); }
        #[allow(unreachable_code)]
        {
            let _ = a;
            Err(format!("{}: diese Fassung von dhrt ist ohne das Feature `pdf` gebaut",
                        name.to_uppercase()))
        }
    }

    #[cfg(feature = "pdf")]
    pub(super) fn try_pdf_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::pdf;
        let v = match name {
            "pdf_new" => {
                let art = match a.first() { Some(Value::Str(s)) => s.to_string(), _ => "a4".to_string() };
                let quer = match a.get(1) {
                    None => false,
                    Some(Value::Str(s)) => {
                        let l = s.to_lowercase();
                        match l.as_str() {
                            "quer" | "querformat" | "landscape" => true,
                            "hoch" | "hochformat" | "portrait" | "" => false,
                            _ => return Err(format!(
                                "PDF_NEW: '{}' kenne ich nicht -- \"hoch\" oder \"quer\"", s)),
                        }
                    }
                    Some(_) => return Err("PDF_NEW: 2. Argument ist \"hoch\" oder \"quer\"".to_string()),
                };
                let (b, h) = pdf::seitenmass(&art, quer).ok_or_else(|| format!(
                    "PDF_NEW: Seitengroesse '{}' kenne ich nicht -- a0 bis a6, letter, legal", art))?;
                self.pdf_dok.push(Some(pdf::Dokument::neu(b, h)));
                Value::Int((self.pdf_dok.len() - 1) as i64)
            }
            "pdf_page" => { let i = bi_int(a, 0, "PDF_PAGE")?; self.pdf_d(i)?.seite_beginnen(); Value::Nil }
            "pdf_page_count" => { let i = bi_int(a, 0, "PDF_PAGE_COUNT")?; Value::Int(self.pdf_d(i)?.seiten.len() as i64) }
            "pdf_page_width" => { let i = bi_int(a, 0, "PDF_PAGE_WIDTH")?; Value::Float(self.pdf_d(i)?.breite_mm) }
            "pdf_page_height" => { let i = bi_int(a, 0, "PDF_PAGE_HEIGHT")?; Value::Float(self.pdf_d(i)?.hoehe_mm) }
            "pdf_title" => {
                let i = bi_int(a, 0, "PDF_TITLE")?;
                let t = bi_str(a, 1, "PDF_TITLE")?.to_string();
                self.pdf_d(i)?.titel = t; Value::Nil
            }
            "pdf_font" => {
                let i = bi_int(a, 0, "PDF_FONT")?;
                let name = bi_str(a, 1, "PDF_FONT")?.to_string();
                let gr = bi_num(a, 2, "PDF_FONT")?;
                let d = self.pdf_d(i)?;
                let idx = d.schrift_suchen(&name).map_err(|e| format!("PDF_FONT: {}", e))?;
                if gr <= 0.0 { return Err(format!("PDF_FONT: Schriftgroesse {} ist nicht positiv", gr)); }
                d.setze_schrift(idx, gr);
                Value::Nil
            }
            // PDF_FONT_LOAD(p, pfad$, name$): eine eigene TrueType-/OpenType-
            // Schrift, danach unter `name` fuer PDF_FONT da.
            "pdf_font_load" => {
                let i = bi_int(a, 0, "PDF_FONT_LOAD")?;
                let pfad = bi_str(a, 1, "PDF_FONT_LOAD")?.to_string();
                let name = bi_str(a, 2, "PDF_FONT_LOAD")?.to_string();
                let daten = std::fs::read(&pfad)
                    .map_err(|e| format!("PDF_FONT_LOAD: {}: {}", pfad, e))?;
                self.pdf_d(i)?.schrift_laden(&name, daten)
                    .map_err(|e| format!("PDF_FONT_LOAD: {}", e))?;
                Value::Nil
            }
            "pdf_color" => {
                let i = bi_int(a, 0, "PDF_COLOR")?;
                let c = bi_int(a, 1, "PDF_COLOR")?;
                // Wie ueberall in Drachenhauch: 0xRRGGBB.
                let (r, g, b) = (((c >> 16) & 255) as f64 / 255.0,
                                 ((c >> 8) & 255) as f64 / 255.0,
                                 (c & 255) as f64 / 255.0);
                self.pdf_d(i)?.setze_farbe(r, g, b);
                Value::Nil
            }
            "pdf_line_width" => {
                let i = bi_int(a, 0, "PDF_LINE_WIDTH")?;
                let mm = bi_num(a, 1, "PDF_LINE_WIDTH")?;
                self.pdf_d(i)?.setze_strich(mm.max(0.0));
                Value::Nil
            }
            // PDF_TEXT(p, x, y, text$ [, winkel]) -- der Winkel in Grad gegen
            // den Uhrzeigersinn, gedreht um (x, y). Fuer Wasserzeichen und
            // senkrechte Achsenbeschriftungen (Glutfaden, 2026-10-16).
            "pdf_text" => {
                if a.len() != 4 && a.len() != 5 {
                    return Err(format!("PDF_TEXT: erwartet 4 oder 5 Argumente (pdf, x, y, text$ [, winkel]), nicht {}", a.len()));
                }
                let i = bi_int(a, 0, "PDF_TEXT")?;
                let (x, y) = (bi_num(a, 1, "PDF_TEXT")?, bi_num(a, 2, "PDF_TEXT")?);
                let t = bi_str(a, 3, "PDF_TEXT")?.to_string();
                let w = if a.len() > 4 { bi_num(a, 4, "PDF_TEXT")? } else { 0.0 };
                self.pdf_d(i)?.text(x, y, &t, w).map_err(|e| format!("PDF_TEXT: {}", e))?;
                Value::Nil
            }
            // PDF_OPACITY(p, deckkraft) -- 0..1 fuer alles, was danach kommt;
            // PDF_GET_OPACITY liest sie zurueck.
            "pdf_opacity" => {
                let i = bi_int(a, 0, "PDF_OPACITY")?;
                let d = bi_num(a, 1, "PDF_OPACITY")?;
                self.pdf_d(i)?.setze_deckkraft(d).map_err(|e| format!("PDF_OPACITY: {}", e))?;
                Value::Nil
            }
            "pdf_get_opacity" => { let i = bi_int(a, 0, "PDF_GET_OPACITY")?; Value::Float(self.pdf_d(i)?.deckkraft()) }
            // PDF_DASH(p, strich_mm [, luecke_mm]) -- gestrichelt fuer alle
            // folgenden Linien und Umrisse; ohne Luecke so lang wie der
            // Strich, PDF_DASH(p, 0) zieht wieder durch (Glutfaden Stufe 7:
            // die Spruenge im Stickpfad).
            "pdf_dash" => {
                if a.len() != 2 && a.len() != 3 {
                    return Err(format!("PDF_DASH: erwartet 2 oder 3 Argumente (pdf, strich_mm [, luecke_mm]), nicht {}", a.len()));
                }
                let i = bi_int(a, 0, "PDF_DASH")?;
                let an = bi_num(a, 1, "PDF_DASH")?;
                let aus = if a.len() > 2 { bi_num(a, 2, "PDF_DASH")? } else { an };
                self.pdf_d(i)?.setze_strichel(an, aus).map_err(|e| format!("PDF_DASH: {}", e))?;
                Value::Nil
            }
            "pdf_circle" | "pdf_circle_fill" => {
                let fn_ = if name == "pdf_circle" { "PDF_CIRCLE" } else { "PDF_CIRCLE_FILL" };
                let i = bi_int(a, 0, fn_)?;
                let (x, y, r) = (bi_num(a, 1, fn_)?, bi_num(a, 2, fn_)?, bi_num(a, 3, fn_)?);
                self.pdf_d(i)?.kreis(x, y, r, name == "pdf_circle_fill").map_err(|e| format!("{}: {}", fn_, e))?;
                Value::Nil
            }
            // PDF_POLYGON(p, xs, ys) -- ein geschlossener Zug; die Felder
            // muessen gleich lang sein.
            "pdf_polygon" | "pdf_polygon_fill" => {
                let fn_ = if name == "pdf_polygon" { "PDF_POLYGON" } else { "PDF_POLYGON_FILL" };
                let i = bi_int(a, 0, fn_)?;
                let xs = gfloats(a, 1, fn_)?;
                let ys = gfloats(a, 2, fn_)?;
                if xs.len() != ys.len() {
                    return Err(format!("{}: xs und ys muessen gleich lang sein ({} gegen {})", fn_, xs.len(), ys.len()));
                }
                let pkt: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();
                self.pdf_d(i)?.vieleck(pkt, name == "pdf_polygon_fill").map_err(|e| format!("{}: {}", fn_, e))?;
                Value::Nil
            }
            "pdf_text_width" => {
                let i = bi_int(a, 0, "PDF_TEXT_WIDTH")?;
                let t = bi_str(a, 1, "PDF_TEXT_WIDTH")?.to_string();
                Value::Float(self.pdf_d(i)?.textbreite(&t).map_err(|e| format!("PDF_TEXT_WIDTH: {}", e))?)
            }
            "pdf_line" => {
                let i = bi_int(a, 0, "PDF_LINE")?;
                let (x1, y1) = (bi_num(a, 1, "PDF_LINE")?, bi_num(a, 2, "PDF_LINE")?);
                let (x2, y2) = (bi_num(a, 3, "PDF_LINE")?, bi_num(a, 4, "PDF_LINE")?);
                self.pdf_d(i)?.linie(x1, y1, x2, y2);
                Value::Nil
            }
            "pdf_rect" | "pdf_rect_fill" => {
                let fn_ = if name == "pdf_rect" { "PDF_RECT" } else { "PDF_RECT_FILL" };
                let i = bi_int(a, 0, fn_)?;
                let (x, y) = (bi_num(a, 1, fn_)?, bi_num(a, 2, fn_)?);
                let (b, h) = (bi_num(a, 3, fn_)?, bi_num(a, 4, fn_)?);
                self.pdf_d(i)?.rechteck(x, y, b, h, name == "pdf_rect_fill");
                Value::Nil
            }
            // PDF_IMAGE(pdf, bild, x, y, b [, h]) -- ein IMAGE (LOADIMAGE,
            // RENDERTARGET_IMAGE, IMAGE_NEW ...) in die Seite setzen. Bis 2026.30
            // kannte das pdf-Modul nur Text, Linien und Rechtecke; ein Bericht
            // musste seine Diagramme Strich fuer Strich nachmalen
            // (Messdaten-Runde, 2026-10-13).
            "pdf_image" => {
                if a.len() != 5 && a.len() != 6 {
                    return Err(format!("PDF_IMAGE: erwartet 5 oder 6 Argumente (pdf, bild, x, y, b [, h]), nicht {}", a.len()));
                }
                let i = bi_int(a, 0, "PDF_IMAGE")?;
                let bild = bi_int(a, 1, "PDF_IMAGE")?;
                let (x, y, b) = (bi_num(a, 2, "PDF_IMAGE")?, bi_num(a, 3, "PDF_IMAGE")?, bi_num(a, 4, "PDF_IMAGE")?);
                let h = if a.len() > 5 { bi_num(a, 5, "PDF_IMAGE")? } else { 0.0 };
                #[cfg(feature = "graphics")]
                {
                    let g = self.gfx.as_ref().ok_or("PDF_IMAGE: kein Bild geladen -- ein Bild kommt aus LOADIMAGE, IMAGE_NEW oder RENDERTARGET_IMAGE")?;
                    let (px, pw, ph) = g.image_rgba(bild, "PDF_IMAGE")?;
                    self.pdf_d(i)?.bild(x, y, b, h, px, pw, ph).map_err(|e| format!("PDF_IMAGE: {}", e))?;
                    Value::Nil
                }
                #[cfg(not(feature = "graphics"))]
                { let _ = (i, bild, x, y, b, h); return Err("PDF_IMAGE: dieser Bau hat keine Grafik".into()); }
            }
            "pdf_save" => {
                let i = bi_int(a, 0, "PDF_SAVE")?;
                let pfad = bi_str(a, 1, "PDF_SAVE")?.to_string();
                let bytes = self.pdf_d(i)?.bauen().map_err(|e| format!("PDF_SAVE: {}", e))?;
                std::fs::write(&pfad, &bytes)
                    .map_err(|e| format!("PDF_SAVE: {}: {}", pfad, e))?;
                Value::Nil
            }
            // Drucken (docs/entwurf-drucken.md, Weg C): dieselbe Seite auf den
            // Drucker -- GDI unter Windows, lp unter Unix. `zieldatei` fuer
            // Drucker, die in eine Datei schreiben (Microsoft Print to PDF).
            "pdf_print" => {
                let i = bi_int(a, 0, "PDF_PRINT")?;
                let drucker = if a.len() > 1 { bi_str(a, 1, "PDF_PRINT")?.to_string() } else { String::new() };
                let kopien = if a.len() > 2 { bi_int(a, 2, "PDF_PRINT")? } else { 1 };
                let ziel = if a.len() > 3 { bi_str(a, 3, "PDF_PRINT")?.to_string() } else { String::new() };
                if a.len() > 4 { return Err("PDF_PRINT: erwartet 1..4 Argumente (pdf[, drucker$[, kopien[, zieldatei$]]])".into()); }
                let doc = self.pdf_d(i)?;
                crate::drucken::drucken(doc, &drucker, kopien, &ziel)?;
                Value::Nil
            }
            // PDF_PREVIEW(p, seite[, breite_px]) -> IMAGE: die Seite als Bild.
            "pdf_preview" => {
                let i = bi_int(a, 0, "PDF_PREVIEW")?;
                let seite = bi_int(a, 1, "PDF_PREVIEW")?;
                let breite = if a.len() > 2 { bi_int(a, 2, "PDF_PREVIEW")? as i32 } else { 600 };
                let (ops, b_mm, h_mm, schriften) = {
                    let doc = self.pdf_d(i)?;
                    let n = doc.seiten.len() as i64;
                    if seite < 1 || seite > n {
                        return Err(format!("PDF_PREVIEW: Seite {} gibt es nicht (1..{})", seite, n));
                    }
                    let sz = &doc.seiten[(seite - 1) as usize];
                    // Die Schriften, die auf der Seite vorkommen -- je Nummer.
                    let mut schriften: Vec<Option<(String, crate::pdf::SchriftDaten)>> = Vec::new();
                    for op in &sz.ops {
                        if let crate::pdf::Op::Text { schrift, .. } = op {
                            while schriften.len() <= *schrift { schriften.push(None); }
                            if schriften[*schrift].is_none() { schriften[*schrift] = doc.schrift_quelle(*schrift); }
                        }
                    }
                    (sz.ops.clone(), sz.breite_mm, sz.hoehe_mm, schriften)
                };
                #[cfg(feature = "graphics")]
                {
                    let g = self.gfx.as_mut().ok_or("PDF_PREVIEW: braucht ein Fenster (SCREEN) -- die Vorschau ist ein IMAGE")?;
                    Value::Int(g.pdf_vorschau(&ops, b_mm, h_mm, breite, &schriften)?)
                }
                #[cfg(not(feature = "graphics"))]
                { let _ = (ops, b_mm, h_mm, breite, schriften); return Err("PDF_PREVIEW: dieser Bau hat keine Grafik".into()); }
            }
            "pdf_close" => {
                let i = bi_int(a, 0, "PDF_CLOSE")? as usize;
                if let Some(slot) = self.pdf_dok.get_mut(i) { *slot = None; }
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    #[cfg(feature = "pdf")]
    pub(super) fn pdf_d(&mut self, idx: i64) -> R<&mut crate::pdf::Dokument> {
        Self::handle_get_mut(&mut self.pdf_dok, idx, "PDF", "ungueltiges/geschlossenes PDF-Handle")
    }

    /// Modul `httpd` -- ein kleiner Webserver im Takt der Hauptschleife.
    ///
    /// Steht in vm.rs und nicht in builtins.rs, weil die laufenden Server
    /// VM-Zustand sind (wie die MQTT-Verbindungen daneben).
    pub(super) fn try_httpd(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("httpd_") { return Ok(None); }
        #[cfg(feature = "net")]
        { return self.try_httpd_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a);
          Err("httpd: dieser dhrt-Build hat das Netz-Feature nicht (neu bauen mit python rust/build_runtime.py)".to_string()) }
    }

    #[cfg(feature = "net")]
    pub(super) fn httpd_srv(&mut self, idx: i64) -> R<&mut crate::httpd::Server> {
        Self::handle_get_mut(&mut self.httpd_server, idx, "HTTPD",
                             "ungueltiges/gestopptes HTTPD-Handle")
    }

    #[cfg(feature = "net")]
    pub(super) fn try_httpd_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::httpd;
        let v = match name {
            "httpd_start" => {
                let port = bi_int(a, 0, "HTTPD_START")?;
                let bind = match a.get(1) { Some(Value::Str(s)) => s.to_string(), _ => String::new() };
                let s = httpd::starten(port, &bind)?;
                self.httpd_server.push(Some(s));
                Value::Int((self.httpd_server.len() - 1) as i64)
            }
            "httpd_stop" => {
                let i = bi_int(a, 0, "HTTPD_STOP")? as usize;
                if let Some(slot) = self.httpd_server.get_mut(i) { *slot = None; }
                Value::Nil
            }
            "httpd_port" => { let i = bi_int(a, 0, "HTTPD_PORT")?; Value::Int(self.httpd_srv(i)?.port) }
            "httpd_set_header" => {
                let i = bi_int(a, 0, "HTTPD_SET_HEADER")?;
                let n = bi_str(a, 1, "HTTPD_SET_HEADER")?.to_string();
                let w = bi_str(a, 2, "HTTPD_SET_HEADER")?.to_string();
                httpd::kopfzeile_setzen(self.httpd_srv(i)?, &n, &w)?;
                Value::Nil
            }
            "httpd_accept" => {
                let i = bi_int(a, 0, "HTTPD_ACCEPT")?;
                Value::Bool(httpd::annehmen(self.httpd_srv(i)?)?)
            }
            "httpd_method$" | "httpd_method" => {
                let i = bi_int(a, 0, "HTTPD_METHOD$")?;
                Value::str_rc(&Self::httpd_anfrage(self.httpd_srv(i)?, "HTTPD_METHOD$")?.methode.clone())
            }
            "httpd_path$" | "httpd_path" => {
                let i = bi_int(a, 0, "HTTPD_PATH$")?;
                Value::str_rc(&Self::httpd_anfrage(self.httpd_srv(i)?, "HTTPD_PATH$")?.pfad.clone())
            }
            "httpd_body$" | "httpd_body" => {
                let i = bi_int(a, 0, "HTTPD_BODY$")?;
                let r = Self::httpd_anfrage(self.httpd_srv(i)?, "HTTPD_BODY$")?;
                Value::str_rc(String::from_utf8_lossy(&r.rumpf).into_owned())
            }
            "httpd_query$" | "httpd_query" => {
                let i = bi_int(a, 0, "HTTPD_QUERY$")?;
                let k = bi_str(a, 1, "HTTPD_QUERY$")?.to_lowercase();
                let r = Self::httpd_anfrage(self.httpd_srv(i)?, "HTTPD_QUERY$")?;
                // Nicht vorhanden -> Leerstring, kein Fehler: was in einer
                // Adresse steht, ist Benutzereingabe (wie ARG$).
                Value::str_rc(r.abfrage.iter().find(|(n, _)| n.to_lowercase() == k)
                               .map(|(_, v)| v.as_str()).unwrap_or(""))
            }
            "httpd_header$" | "httpd_header" => {
                let i = bi_int(a, 0, "HTTPD_HEADER$")?;
                let k = bi_str(a, 1, "HTTPD_HEADER$")?.to_lowercase();
                let r = Self::httpd_anfrage(self.httpd_srv(i)?, "HTTPD_HEADER$")?;
                Value::str_rc(r.kopfzeilen.iter().find(|(n, _)| *n == k)
                               .map(|(_, v)| v.as_str()).unwrap_or(""))
            }
            "httpd_send" => {
                let i = bi_int(a, 0, "HTTPD_SEND")?;
                let code = bi_int(a, 1, "HTTPD_SEND")?;
                let typ = bi_str(a, 2, "HTTPD_SEND")?.to_string();
                let inhalt = bi_str(a, 3, "HTTPD_SEND")?.to_string();
                httpd::antworten(self.httpd_srv(i)?, code, &typ, inhalt.as_bytes())?;
                Value::Nil
            }
            "httpd_send_file" => {
                let i = bi_int(a, 0, "HTTPD_SEND_FILE")?;
                let code = bi_int(a, 1, "HTTPD_SEND_FILE")?;
                let pfad = bi_str(a, 2, "HTTPD_SEND_FILE")?.to_string();
                let inhalt = std::fs::read(&pfad)
                    .map_err(|e| format!("HTTPD_SEND_FILE: {}: {}", pfad, e))?;
                let typ = httpd::typ_aus_endung(&pfad);
                httpd::antworten(self.httpd_srv(i)?, code, typ, &inhalt)?;
                Value::Nil
            }
            "httpd_send_dir" => {
                // Der SICHERE Weg, Dateien auszuliefern: der angefragte Pfad
                // wird selbst im Ordner aufgeloest, samt Pruefung gegen
                // `..`. Die naheliegende Handarbeit
                // (`HTTPD_SEND_FILE(s, 200, "web" + HTTPD_PATH$(s))`) laesst
                // sich mit `/../../geheim` aus dem Ordner herausfuehren --
                // dieselbe Lehre wie bei der Zip-Slip-Pruefung.
                let i = bi_int(a, 0, "HTTPD_SEND_DIR")?;
                let ordner = bi_str(a, 1, "HTTPD_SEND_DIR")?.to_string();
                let start = match a.get(2) { Some(Value::Str(s)) => s.to_string(), _ => "index.html".to_string() };
                let pfad = Self::httpd_anfrage(self.httpd_srv(i)?, "HTTPD_SEND_DIR")?.pfad.clone();
                let wurzel = std::path::Path::new(&ordner);
                let ziel = match httpd::sicher_verbinden(wurzel, &pfad) {
                    Some(z) => if z.is_dir() { z.join(&start) } else { z },
                    None => {
                        httpd::antworten(self.httpd_srv(i)?, 403, "text/plain; charset=utf-8",
                                         b"403 -- Pfad nicht erlaubt")?;
                        return Ok(Some(Value::Bool(false)));
                    }
                };
                match std::fs::read(&ziel) {
                    Ok(inhalt) => {
                        let typ = httpd::typ_aus_endung(&ziel.to_string_lossy());
                        httpd::antworten(self.httpd_srv(i)?, 200, typ, &inhalt)?;
                        Value::Bool(true)
                    }
                    Err(_) => {
                        // Kein Fehler, sondern ein 404 -- eine fehlende Datei
                        // ist der Alltag eines Servers und darf das Programm
                        // nicht anhalten. Der Rueckgabewert sagt es trotzdem.
                        httpd::antworten(self.httpd_srv(i)?, 404, "text/plain; charset=utf-8",
                                         b"404 -- nicht gefunden")?;
                        Value::Bool(false)
                    }
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// Die gerade offene Anfrage -- oder ein Hinweis, dass keine anliegt.
    #[cfg(feature = "net")]
    pub(super) fn httpd_anfrage<'x>(s: &'x crate::httpd::Server, fn_: &str) -> R<&'x crate::httpd::Anfrage> {
        s.offen.as_ref().map(|(_, a)| a)
            .ok_or_else(|| format!("{}: es liegt keine Anfrage an -- erst HTTPD_ACCEPT abfragen", fn_))
    }

    pub(super) fn try_mqtt(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("mqtt_")) { return Ok(None); }
        #[cfg(feature = "net")]
        { return self.try_mqtt_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }
    #[cfg(feature = "net")]
    pub(super) fn mqtt_client(&mut self, idx: i64) -> R<&mut crate::mqtt::Client> {
        Self::handle_get_mut(&mut self.mqtt_clients, idx, "MQTT", "ungueltiges/geschlossenes MQTT_HANDLE")
    }
    #[cfg(feature = "net")]
    pub(super) fn try_mqtt_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::mqtt;
        let v = match name {
            "mqtt_connect" => {
                let host = bi_str(a, 0, "MQTT_CONNECT")?.to_string();
                let port = bi_int(a, 1, "MQTT_CONNECT")?;
                let client_id = bi_str(a, 2, "MQTT_CONNECT")?.to_string();
                let keepalive = if a.len() >= 4 { bi_int(a, 3, "MQTT_CONNECT")? } else { 60 };
                let username = match a.get(4) { Some(Value::Str(s)) => Some(s.to_string()), _ => None };
                let password = match a.get(5) { Some(Value::Str(s)) => Some(s.to_string()), _ => None };
                // Der Wille gilt fuer genau diese Verbindung -- auch wenn sie
                // scheitert, wer neu verbindet, setzt ihn neu.
                let wille = self.mqtt_wille.take();
                let c = mqtt::connect(&host, port, &client_id, keepalive, username.as_deref(), password.as_deref(), wille.as_ref(), self.mqtt_ca.as_deref())?;
                self.mqtt_clients.push(Some(c));
                Value::Int((self.mqtt_clients.len() - 1) as i64)
            }
            // MQTT_WILL(topic$, inhalt$[, retain]) -- vor MQTT_CONNECT
            "mqtt_will" => {
                let topic = bi_str(a, 0, "MQTT_WILL")?.to_string();
                if topic.is_empty() || topic.contains('#') || topic.contains('+') {
                    return Err("MQTT_WILL: das Topic darf nicht leer sein und keine Platzhalter (# +) enthalten".into());
                }
                let inhalt = bi_str(a, 1, "MQTT_WILL")?.as_bytes().to_vec();
                let retain = match a.get(2) { Some(Value::Bool(b)) => *b, Some(Value::Int(n)) => *n != 0, None => false,
                    _ => return Err("MQTT_WILL: retain ist TRUE oder FALSE".into()) };
                self.mqtt_wille = Some(mqtt::Wille { topic, inhalt, retain });
                Value::Nil
            }
            // MQTT_TLS_CA(pfad$) -- "" = wieder die Wurzeln von webpki-roots
            "mqtt_tls_ca" => {
                let pfad = bi_str(a, 0, "MQTT_TLS_CA")?.to_string();
                if pfad.is_empty() {
                    self.mqtt_ca = None;
                } else {
                    if !std::path::Path::new(&pfad).is_file() {
                        return Err(format!("MQTT_TLS_CA: die Datei {} gibt es nicht", pfad));
                    }
                    self.mqtt_ca = Some(pfad);
                }
                Value::Nil
            }
            "mqtt_disconnect" => {
                let i = bi_int(a, 0, "MQTT_DISCONNECT")? as usize;
                if let Some(slot) = self.mqtt_clients.get_mut(i) {
                    if let Some(c) = slot.as_mut() { mqtt::disconnect(c); }
                    *slot = None;
                }
                Value::Nil
            }
            "mqtt_is_connected" => { let i = bi_int(a, 0, "MQTT_IS_CONNECTED")?; Value::Bool(self.mqtt_clients.get(i as usize).and_then(|o| o.as_ref()).map(mqtt::is_connected).unwrap_or(false)) }
            "mqtt_publish" => {
                let i = bi_int(a, 0, "MQTT_PUBLISH")?; let topic = bi_str(a, 1, "MQTT_PUBLISH")?.to_string(); let payload = bi_str(a, 2, "MQTT_PUBLISH")?.to_string();
                let retain = if a.len() >= 4 { bi_bool(a, 3, "MQTT_PUBLISH")? } else { false };
                let qos = mqtt::qos_pruefen("MQTT_PUBLISH", if a.len() >= 5 { bi_int(a, 4, "MQTT_PUBLISH")? } else { 0 })?;
                Value::Bool(mqtt::publish(self.mqtt_client(i)?, &topic, &payload, retain, qos))
            }
            "mqtt_subscribe" => {
                let i = bi_int(a, 0, "MQTT_SUBSCRIBE")?; let topic = bi_str(a, 1, "MQTT_SUBSCRIBE")?.to_string();
                let qos = mqtt::qos_pruefen("MQTT_SUBSCRIBE", if a.len() >= 3 { bi_int(a, 2, "MQTT_SUBSCRIBE")? } else { 0 })?;
                Value::Bool(mqtt::subscribe(self.mqtt_client(i)?, &topic, qos))
            }
            "mqtt_unsubscribe" => {
                let i = bi_int(a, 0, "MQTT_UNSUBSCRIBE")?; let topic = bi_str(a, 1, "MQTT_UNSUBSCRIBE")?.to_string();
                Value::Bool(mqtt::unsubscribe(self.mqtt_client(i)?, &topic))
            }
            "mqtt_pending" => { let i = bi_int(a, 0, "MQTT_PENDING")?; Value::Int(mqtt::pending(self.mqtt_client(i)?) as i64) }
            "mqtt_flush" => {
                let i = bi_int(a, 0, "MQTT_FLUSH")?;
                let ms = if a.len() >= 2 { bi_int(a, 1, "MQTT_FLUSH")? } else { 2000 };
                Value::Bool(mqtt::flush(self.mqtt_client(i)?, ms)?)
            }
            "mqtt_update" => { let i = bi_int(a, 0, "MQTT_UPDATE")?; mqtt::update(self.mqtt_client(i)?)?; Value::Nil }
            "mqtt_next_message" => { let i = bi_int(a, 0, "MQTT_NEXT_MESSAGE")?; Value::Bool(mqtt::next_message(self.mqtt_client(i)?)) }
            "mqtt_message_topic" => { let i = bi_int(a, 0, "MQTT_MESSAGE_TOPIC")?; Value::str_rc(mqtt::message_topic(self.mqtt_client(i)?)) }
            "mqtt_message_payload" => { let i = bi_int(a, 0, "MQTT_MESSAGE_PAYLOAD")?; Value::str_rc(mqtt::message_payload(self.mqtt_client(i)?)) }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul html (HTTP/HTML/URL, Feature `http`)
    // ===================================================================
    pub(super) fn try_html(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("http_") || name.starts_with("html_") || name.starts_with("url_")) { return Ok(None); }
        #[cfg(feature = "http")]
        { return self.try_html_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    /// Eine HTTP-Antwort in den VM-Zustand uebernehmen und als STRING liefern.
    ///
    /// Frueher stand dieser Block viermal wortgleich in `try_html_impl`. Mit
    /// `HTTP_BYTES` (WP C) kam eine fuenfte Sache dazu, die JEDER Pfad tun
    /// muss -- und genau so vergisst man sie in einem davon.
    #[cfg(feature = "http")]
    pub(super) fn http_antwort(&mut self, r: Result<crate::html::HttpResult, crate::html::HttpErr>) -> R<Value> {
        match r {
            Ok(r) => {
                self.http_status = r.status;
                self.http_headers = r.headers;
                let text = String::from_utf8_lossy(&r.body).into_owned();
                self.http_body = r.body;      // roh, fuer HTTP_BYTES()
                Ok(Value::str_rc(&text))
            }
            Err(e) => {
                if e.status != 0 { self.http_status = e.status; self.http_headers = e.headers; }
                // Sonst gehoerte der Rumpf der VORIGEN Antwort und HTTP_BYTES
                // lieferte nach einem Fehlschlag alte Daten als neue aus.
                self.http_body.clear();
                Err(e.msg)
            }
        }
    }

    /// Baut aus `(methode, url [, rumpf [, kopfzeilen]])` eine Anfrage.
    /// Gemeinsam fuer HTTP_REQUEST und HTTP_REQUEST_START.
    #[cfg(feature = "http")]
    pub(super) fn http_anfrage(&self, fn_: &str, a: &[Value]) -> R<crate::html::Anfrage> {
        if a.len() < 2 || a.len() > 4 {
            return Err(format!(
                "{}: erwartet 2..4 Argumente (methode, url [, rumpf [, kopfzeilen]]), erhalten {}",
                fn_, a.len()));
        }
        let mut anfrage = crate::html::Anfrage::neu(bi_str(a, 0, fn_)?, bi_str(a, 1, fn_)?);
        anfrage.timeout = self.http_timeout;
        // Rumpf: Text ODER Bytes. NIL/weggelassen = kein Rumpf.
        anfrage.body = match a.get(2) {
            None | Some(Value::Nil) => Vec::new(),
            Some(Value::Str(s)) => s.as_bytes().to_vec(),
            Some(Value::Buffer(b)) => b.borrow().clone(),
            Some(v) => return Err(format!(
                "{}: Rumpf erwartet STRING oder BUFFER, erhalten {}", fn_, v.type_name())),
        };
        // Erst die dauerhaften Kopfzeilen, dann die des Aufrufs -- gleiche
        // Namen ueberschreiben, der Aufruf gewinnt also gegen HTTP_SET_HEADER.
        anfrage.header = self.http_default_header.clone();
        match a.get(3) {
            None | Some(Value::Nil) => {}
            Some(Value::Map(m)) => {
                for (k, v) in m.borrow().entries().iter() {
                    match v {
                        Value::Str(s) => anfrage.header.push((k.clone(), s.to_string())),
                        andere => return Err(format!(
                            "{}: Kopfzeile '{}' erwartet STRING, erhalten {}",
                            fn_, k, andere.type_name())),
                    }
                }
            }
            Some(v) => return Err(format!(
                "{}: Kopfzeilen erwarten MAP OF STRING, erhalten {}", fn_, v.type_name())),
        }
        Ok(anfrage)
    }

    #[cfg(feature = "http")]
    pub(super) fn try_html_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::html;
        let v = match name {
            "http_get" => {
                let mut anfrage = html::Anfrage::neu("GET", bi_str(a, 0, "HTTP_GET")?);
                anfrage.timeout = self.http_timeout;
                anfrage.header = self.http_default_header.clone();
                let r = html::http_request(&anfrage);
                self.http_antwort(r)?
            }
            "http_post" => {
                let mut anfrage = html::Anfrage::neu("POST", bi_str(a, 0, "HTTP_POST")?);
                anfrage.timeout = self.http_timeout;
                anfrage.body = bi_str(a, 1, "HTTP_POST")?.as_bytes().to_vec();
                anfrage.header = self.http_default_header.clone();
                anfrage.header.push(("Content-Type".to_string(),
                                     "application/x-www-form-urlencoded; charset=utf-8".to_string()));
                let r = html::http_request(&anfrage);
                self.http_antwort(r)?
            }
            // --- WP C: eine Anfrage, wie sie eine echte API braucht ---
            "http_request" => {
                let anfrage = self.http_anfrage("HTTP_REQUEST", a)?;
                let r = html::http_request(&anfrage);
                self.http_antwort(r)?
            }
            "http_request_start" => {
                let anfrage = self.http_anfrage("HTTP_REQUEST_START", a)?;
                Value::Int(self.http_abrufe.start(anfrage))
            }
            "http_bytes" => {
                if !a.is_empty() { return Err(format!("HTTP_BYTES: erwartet 0 Argumente, erhalten {}", a.len())); }
                Value::Buffer(Rc::new(RefCell::new(self.http_body.clone())))
            }
            "http_timeout" => {
                let s = bi_int(a, 0, "HTTP_TIMEOUT")?;
                // Obergrenze, damit ein vertipptes HTTP_TIMEOUT(600000) das
                // Programm nicht faktisch fuer immer haengen laesst.
                if !(1..=600).contains(&s) {
                    return Err(format!("HTTP_TIMEOUT: {} Sekunden liegen ausserhalb 1..600", s));
                }
                self.http_timeout = s as u64;
                Value::Nil
            }
            "http_set_header" => {
                let k = bi_str(a, 0, "HTTP_SET_HEADER")?.to_string();
                let v = bi_str(a, 1, "HTTP_SET_HEADER")?.to_string();
                // Sofort pruefen statt erst beim naechsten Aufruf: der Fehler
                // gehoert an die Zeile, in der die Kopfzeile gesetzt wird.
                html::pruefe_header(&k, &v)?;
                // Gleicher Name ersetzt -- zweimal denselben Namen zu schicken
                // waere sonst das Ergebnis, und welcher gilt, entschiede der
                // Server.
                if let Some(e) = self.http_default_header.iter_mut().find(|(n, _)| n.eq_ignore_ascii_case(&k)) {
                    e.1 = v;
                } else {
                    self.http_default_header.push((k, v));
                }
                Value::Nil
            }
            "http_clear_headers" => { self.http_default_header.clear(); Value::Nil }
            "http_download" => {
                let url = bi_str(a, 0, "HTTP_DOWNLOAD")?.to_string();
                let path = bi_str(a, 1, "HTTP_DOWNLOAD")?.to_string();
                match html::http_download(&url, &path) {
                    Ok(r) => { self.http_status = r.status; self.http_headers = r.headers; Value::Int(r.bytes) }
                    Err(e) => { if e.status != 0 { self.http_status = e.status; self.http_headers = e.headers; } return Err(e.msg); }
                }
            }
            // --- Abrufe im Hintergrund: starten, nachsehen, abholen ---
            "http_get_start" => {
                let mut anfrage = html::Anfrage::neu("GET", bi_str(a, 0, "HTTP_GET_START")?);
                anfrage.timeout = self.http_timeout;
                anfrage.header = self.http_default_header.clone();
                Value::Int(self.http_abrufe.start(anfrage))
            }
            "http_ready" => Value::Bool(self.http_abrufe.fertig(bi_int(a, 0, "HTTP_READY")?)),
            "http_result" => {
                let id = bi_int(a, 0, "HTTP_RESULT")?;
                match self.http_abrufe.abholen(id) {
                    Some(r) => self.http_antwort(r)?,
                    // Abholen ohne vorheriges HTTP_READY ist der haeufigste
                    // Anfaengerfehler -- deshalb sagt die Meldung, was fehlt.
                    None => return Err(format!(
                        "HTTP_RESULT: Abruf {} ist nicht fertig (oder schon abgeholt) \
                         -- erst HTTP_READY({}) abfragen", id, id)),
                }
            }
            "http_cancel" => { self.http_abrufe.abbrechen(bi_int(a, 0, "HTTP_CANCEL")?); Value::Nil }
            "http_pending" => Value::Int(self.http_abrufe.offen()),
            "http_url$" | "http_url" => Value::str_rc(&self.http_abrufe.url(bi_int(a, 0, "HTTP_URL$")?)),

            "http_status" => Value::Int(self.http_status),
            "http_header" => {
                let n = bi_str(a, 0, "HTTP_HEADER")?.to_lowercase();
                Value::str_rc(self.http_headers.iter().find(|(k, _)| *k == n).map(|(_, v)| v.as_str()).unwrap_or(""))
            }
            "url_encode" => Value::str_rc(html::url_encode(bi_str(a, 0, "URL_ENCODE")?)),
            "url_decode" => Value::str_rc(html::url_decode(bi_str(a, 0, "URL_DECODE")?)),
            "html_text" => Value::str_rc(html::html_text(bi_str(a, 0, "HTML_TEXT")?)),
            "html_get_attr" => Value::str_rc(html::html_get_attr(bi_str(a, 0, "HTML_GET_ATTR")?, bi_str(a, 1, "HTML_GET_ATTR")?)),
            "html_find_all" => {
                let items = html::html_find_all(bi_str(a, 0, "HTML_FIND_ALL")?, bi_str(a, 1, "HTML_FIND_ALL")?);
                let n = items.len() as i64;
                let mut arr = DhArray::new("string".to_string(), vec![n], || Value::str_rc(""));
                for (i, s) in items.into_iter().enumerate() { arr.cells.set(i, Value::str_rc(&s)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul cloud (Cloud-Save + Leaderboard gegen cloudserver/server.dh, Feature `http`)
    // ===================================================================
    pub(super) fn try_cloud(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("cloud_") || name.starts_with("leaderboard_")) { return Ok(None); }
        #[cfg(feature = "http")]
        { return self.try_cloud_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    #[cfg(feature = "http")]
    pub(super) fn cloud_require_configured(&self, ctx: &str) -> R<()> {
        if self.cloud_base_url.is_empty() {
            return Err(format!("{}: CLOUD_CONFIGURE(base_url, api_key) muss zuerst aufgerufen werden", ctx));
        }
        Ok(())
    }

    #[cfg(feature = "http")]
    pub(super) fn try_cloud_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::cloud;
        fn bi_bool(a: &[Value], i: usize, fn_: &str) -> R<bool> {
            match a.get(i) {
                Some(Value::Bool(b)) => Ok(*b),
                Some(v) => Err(format!("{}: erwartet BOOLEAN, erhalten {}", fn_, v.type_name())),
                None => Err(format!("{}: fehlendes Argument {}", fn_, i + 1)),
            }
        }
        let v = match name {
            "cloud_configure" => {
                self.cloud_base_url = bi_str(a, 0, "CLOUD_CONFIGURE")?.to_string();
                self.cloud_api_key = bi_str(a, 1, "CLOUD_CONFIGURE")?.to_string();
                Value::Nil
            }
            "cloud_save" => {
                self.cloud_require_configured("CLOUD_SAVE")?;
                let player_id = bi_str(a, 0, "CLOUD_SAVE")?.to_string();
                let data = bi_str(a, 1, "CLOUD_SAVE")?.to_string();
                self.cloud_last_error.clear();
                match cloud::save_upload(&self.cloud_base_url, &self.cloud_api_key, &player_id, &data) {
                    Ok(()) => Value::Bool(true),
                    Err(e) => { self.cloud_last_error = e.msg; Value::Bool(false) }
                }
            }
            "cloud_load" | "cloud_load$" => {
                self.cloud_require_configured("CLOUD_LOAD")?;
                let player_id = bi_str(a, 0, "CLOUD_LOAD")?.to_string();
                self.cloud_last_error.clear();
                match cloud::save_download(&self.cloud_base_url, &self.cloud_api_key, &player_id) {
                    Ok(Some(data)) => Value::str_rc(&data),
                    Ok(None) => Value::str_rc(""),
                    Err(e) => { self.cloud_last_error = e.msg; Value::str_rc("") }
                }
            }
            "cloud_last_error$" | "cloud_last_error" => Value::str_rc(&self.cloud_last_error),
            "leaderboard_submit" => {
                self.cloud_require_configured("LEADERBOARD_SUBMIT")?;
                let board = bi_str(a, 0, "LEADERBOARD_SUBMIT")?.to_string();
                let pname = bi_str(a, 1, "LEADERBOARD_SUBMIT")?.to_string();
                let score = bi_num(a, 2, "LEADERBOARD_SUBMIT")?;
                let best_low = if a.len() >= 4 { bi_bool(a, 3, "LEADERBOARD_SUBMIT")? } else { false };
                self.cloud_last_error.clear();
                match cloud::leaderboard_submit(&self.cloud_base_url, &self.cloud_api_key, &board, &pname, score, best_low) {
                    Ok(updated) => Value::Bool(updated),
                    Err(e) => { self.cloud_last_error = e.msg; Value::Bool(false) }
                }
            }
            "leaderboard_fetch" => {
                self.cloud_require_configured("LEADERBOARD_FETCH")?;
                let board = bi_str(a, 0, "LEADERBOARD_FETCH")?.to_string();
                let n = bi_int(a, 1, "LEADERBOARD_FETCH")?;
                let ascending = if a.len() >= 3 { bi_bool(a, 2, "LEADERBOARD_FETCH")? } else { false };
                self.cloud_last_error.clear();
                let entries = match cloud::leaderboard_fetch(&self.cloud_base_url, &self.cloud_api_key, &board, n, ascending) {
                    Ok(e) => e,
                    Err(e) => { self.cloud_last_error = e.msg; Vec::new() }
                };
                let items: Vec<Value> = entries.into_iter()
                    .map(|e| Value::Tuple(Rc::new(vec![Value::str_rc(&e.name), Value::Float(e.score)])))
                    .collect();
                let cnt = items.len() as i64;
                let mut arr = DhArray::new("tuple".to_string(), vec![cnt], || Value::Tuple(Rc::new(vec![])));
                for (i, v) in items.into_iter().enumerate() { arr.cells.set(i, v); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }
}
