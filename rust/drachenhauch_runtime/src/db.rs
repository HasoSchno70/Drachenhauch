//! Datenbanken (DB_*): SQLite ueber `rusqlite` (bundled SQLite, Feature
//! `db`) und -- mit den Features `pg` bzw. `mysql` -- PostgreSQL und MySQL
//! ueber einen Server (docs/entwurf-datenbanktreiber.md).
//!
//! Dieselben Befehle fuer alle drei: `DB_OPEN` entscheidet am Ziel --
//! `postgres://...`/`postgresql://...` und `mysql://...` sind Server, alles
//! andere ist eine SQLite-Datei. So zieht ein Programm, das mit einer Datei
//! angefangen hat, mit EINER Zeile auf einen Server um.
//!
//! DB_CONN/DB_RESULT sind INTEGER-Handles (Index in VM-seitige Vecs). Ein
//! DB_QUERY laedt ALLE Zeilen eager in den Speicher (vermeidet self-
//! referenzielle Cursor-Lifetimes; fuer die Groessen hier unkritisch).
//!
//! **Platzhalter:** SQLite und MySQL nehmen `?`, PostgreSQL `$1`, `$2`. Damit
//! dasselbe SQL ueberall laeuft, wird fuer PostgreSQL `?` zu `$n` -- ausser in
//! Zeichenketten, Bezeichnern und Kommentaren, und gar nicht, sobald das SQL
//! selbst schon `$1` schreibt (`platzhalter_pg`).
//!
//! **Werte:** gebunden wird bei PostgreSQL im TEXTformat -- der Server wandelt
//! "42" selbst in int4, numeric oder text, je nachdem, was die Spalte will.
//! Gelesen wird binaer und hier entschluesselt (`pg::dekodieren`); `numeric`
//! und `DECIMAL` bleiben genau (`DbVal::Zahl`, als Text), damit Geld nicht
//! durch eine Kommazahl rundet.
#![cfg(feature = "db")]

use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::Connection;

use crate::value::Value;

/// Eine gespeicherte Zelle.
#[derive(Clone, Debug, PartialEq)]
pub enum DbVal {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
    /// Eine genaue Dezimalzahl (PostgreSQL `numeric`, MySQL `DECIMAL`) als
    /// Text -- DB_GET_STRING liefert sie unveraendert, DB_GET_FLOAT rechnet um.
    #[cfg_attr(not(any(feature = "pg", feature = "mysql")), allow(dead_code))]
    Zahl(String),
}

pub struct DbResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<DbVal>>,
    pub pos: i64, // -1 vor dem ersten DB_NEXT
    pub closed: bool,
}

impl DbResult {
    fn cur_value(&self, idx: i64, fn_: &str) -> Result<&DbVal, String> {
        if self.pos < 0 || self.pos as usize >= self.rows.len() {
            return Err(format!("{}: Keine aktuelle Zeile - DB_NEXT vor dem Lesen aufrufen", fn_));
        }
        let row = &self.rows[self.pos as usize];
        if idx < 0 || idx as usize >= row.len() {
            return Err(format!("{}: Spalten-Index {} ausserhalb [0..{}]", fn_, idx, row.len() as i64 - 1));
        }
        Ok(&row[idx as usize])
    }

    pub fn is_null(&self, idx: i64) -> Result<bool, String> {
        Ok(matches!(self.cur_value(idx, "DB_IS_NULL")?, DbVal::Null))
    }
    pub fn get_string(&self, idx: i64) -> Result<String, String> {
        Ok(match self.cur_value(idx, "DB_GET_STRING")? {
            DbVal::Null => String::new(),
            DbVal::Text(s) | DbVal::Zahl(s) => s.clone(),
            DbVal::Int(n) => n.to_string(),
            DbVal::Real(f) => Value::Float(*f).fmt(),
            DbVal::Blob(b) => String::from_utf8_lossy(b).into_owned(),
        })
    }
    pub fn get_int(&self, idx: i64) -> Result<i64, String> {
        match self.cur_value(idx, "DB_GET_INT")? {
            DbVal::Null => Ok(0),
            DbVal::Int(n) => Ok(*n),
            // `f as i64` saettigt bei Werten ausserhalb des i64-Bereichs still
            // auf i64::MAX/MIN statt zu scheitern -- fuer sowas riesige REALs
            // (z.B. 1e20) ist ein klarer Fehler richtiger als ein falscher Wert.
            DbVal::Real(f) if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 => Ok(*f as i64),
            // Eine Dezimalzahl ohne Nachkommastellen (`12.00` gilt als ganz).
            DbVal::Zahl(s) => {
                let (ganz, rest) = s.split_once('.').unwrap_or((s, ""));
                if rest.chars().all(|c| c == '0') {
                    if let Ok(n) = ganz.parse::<i64>() { return Ok(n); }
                }
                Err(format!("DB_GET_INT: Spalte {} ist die Dezimalzahl {}, nicht ganz", idx, s))
            }
            v => Err(format!("DB_GET_INT: Spalte {} ist {}, nicht INTEGER", idx, type_name(v))),
        }
    }
    pub fn get_float(&self, idx: i64) -> Result<f64, String> {
        match self.cur_value(idx, "DB_GET_FLOAT")? {
            DbVal::Null => Ok(0.0),
            DbVal::Int(n) => Ok(*n as f64),
            DbVal::Real(f) => Ok(*f),
            DbVal::Zahl(s) => s.parse::<f64>().map_err(|_| format!("DB_GET_FLOAT: Spalte {} ist {}, keine Zahl", idx, s)),
            v => Err(format!("DB_GET_FLOAT: Spalte {} ist {}, nicht Zahl", idx, type_name(v))),
        }
    }
    pub fn get_bool(&self, idx: i64) -> Result<bool, String> {
        match self.cur_value(idx, "DB_GET_BOOL")? {
            DbVal::Null => Ok(false),
            DbVal::Int(n) => Ok(*n != 0),
            DbVal::Real(f) => Ok(*f != 0.0),
            DbVal::Zahl(s) => Ok(s.parse::<f64>().map(|f| f != 0.0).unwrap_or(false)),
            v => Err(format!("DB_GET_BOOL: Spalte {} ist {}, nicht 0/1", idx, type_name(v))),
        }
    }
    pub fn col_name(&self, idx: i64) -> Result<String, String> {
        if idx < 0 || idx as usize >= self.columns.len() {
            return Err(format!("DB_COL_NAME: Index {} ausserhalb [0..{}]", idx, self.columns.len() as i64 - 1));
        }
        Ok(self.columns[idx as usize].clone())
    }
}

fn type_name(v: &DbVal) -> &'static str {
    match v {
        DbVal::Null => "NULL",
        DbVal::Int(_) => "int",
        DbVal::Real(_) => "float",
        DbVal::Text(_) => "str",
        DbVal::Blob(_) => "bytes",
        DbVal::Zahl(_) => "decimal",
    }
}

/// Ein gebundener Wert, fuer alle drei Datenbanken gleich.
#[derive(Clone, Debug, PartialEq)]
pub enum Param {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

/// DH-Wert -> Bind-Wert (bool -> 0/1, Nil -> NULL, BUFFER -> Bytes).
pub fn dh_to_sql(v: &Value, fn_: &str) -> Result<Param, String> {
    Ok(match v {
        Value::Nil => Param::Null,
        Value::Bool(b) => Param::Int(if *b { 1 } else { 0 }),
        Value::Int(n) => Param::Int(*n),
        Value::Float(f) => Param::Real(*f),
        Value::Str(s) => Param::Text(s.to_string()),
        Value::Buffer(b) => Param::Blob(b.borrow().clone()),
        other => return Err(format!(
            "{}: Parameter-Typ {} nicht unterstuetzt (erlaubt: INTEGER, FLOAT, STRING, BOOLEAN, BUFFER, NIL)",
            fn_, other.type_name())),
    })
}

fn sqlite_param(p: &Param) -> SqlValue {
    match p {
        Param::Null => SqlValue::Null,
        Param::Int(n) => SqlValue::Integer(*n),
        Param::Real(f) => SqlValue::Real(*f),
        Param::Text(s) => SqlValue::Text(s.clone()),
        Param::Blob(b) => SqlValue::Blob(b.clone()),
    }
}

/// Eine offene Verbindung -- eine SQLite-Datei oder ein Server.
pub enum Verbindung {
    Sqlite(Connection),
    #[cfg(feature = "pg")]
    Pg(Box<postgres::Client>),
    #[cfg(feature = "mysql")]
    My(Box<mysql::Conn>),
}

/// Welche Datenbank ein Ziel meint.
fn art_von(ziel: &str) -> &'static str {
    let low = ziel.trim_start().to_ascii_lowercase();
    if low.starts_with("postgres://") || low.starts_with("postgresql://") { "postgres" }
    else if low.starts_with("mysql://") || low.starts_with("mariadb://") { "mysql" }
    else { "sqlite" }
}

/// Eine Datenbank oeffnen: eine SQLite-Datei oder einen Server.
pub fn open(ziel: &str) -> Result<Verbindung, String> {
    match art_von(ziel) {
        "postgres" => {
            #[cfg(feature = "pg")]
            { return pg::oeffnen(ziel.trim()).map(|c| Verbindung::Pg(Box::new(c))); }
            #[allow(unreachable_code)]
            Err("DB_OPEN: PostgreSQL ist in diesem Bau nicht dabei (Feature pg)".into())
        }
        "mysql" => {
            #[cfg(feature = "mysql")]
            { return my::oeffnen(ziel.trim()).map(|c| Verbindung::My(Box::new(c))); }
            #[allow(unreachable_code)]
            Err("DB_OPEN: MySQL ist in diesem Bau nicht dabei (Feature mysql)".into())
        }
        _ => Connection::open(ziel).map(Verbindung::Sqlite).map_err(|e| format!("DB_OPEN: {}", e)),
    }
}

impl Verbindung {
    /// `sqlite`, `postgres` oder `mysql` (DB_KIND$).
    pub fn art(&self) -> &'static str {
        match self {
            Verbindung::Sqlite(_) => "sqlite",
            #[cfg(feature = "pg")]
            Verbindung::Pg(_) => "postgres",
            #[cfg(feature = "mysql")]
            Verbindung::My(_) => "mysql",
        }
    }

    /// Eine Anweisung ohne Ergebnis; liefert die Zahl der betroffenen Zeilen.
    pub fn exec(&mut self, sql: &str, params: &[Param], fn_: &str) -> Result<i64, String> {
        match self {
            Verbindung::Sqlite(c) => c.execute(sql, rusqlite::params_from_iter(params.iter().map(sqlite_param)))
                .map(|n| n as i64).map_err(|e| format!("{}: {}", fn_, e)),
            #[cfg(feature = "pg")]
            Verbindung::Pg(c) => pg::exec(c, sql, params).map_err(|e| format!("{}: {}", fn_, e)),
            #[cfg(feature = "mysql")]
            Verbindung::My(c) => my::exec(c, sql, params).map_err(|e| format!("{}: {}", fn_, e)),
        }
    }

    /// Eine Abfrage; alle Zeilen auf einmal.
    pub fn query(&mut self, sql: &str, params: &[Param], fn_: &str) -> Result<DbResult, String> {
        match self {
            Verbindung::Sqlite(c) => sqlite_query(c, sql, params).map_err(|e| format!("{}: {}", fn_, e)),
            #[cfg(feature = "pg")]
            Verbindung::Pg(c) => pg::query(c, sql, params).map_err(|e| format!("{}: {}", fn_, e)),
            #[cfg(feature = "mysql")]
            Verbindung::My(c) => my::query(c, sql, params).map_err(|e| format!("{}: {}", fn_, e)),
        }
    }

    /// BEGIN, COMMIT, ROLLBACK -- ohne Platzhalter, ohne Ergebnis.
    pub fn befehl(&mut self, sql: &str, fn_: &str) -> Result<(), String> {
        match self {
            Verbindung::Sqlite(c) => c.execute_batch(sql).map_err(|e| format!("{}: {}", fn_, e)),
            #[cfg(feature = "pg")]
            Verbindung::Pg(c) => c.batch_execute(sql).map_err(|e| format!("{}: {}", fn_, pg::fehler(&e))),
            #[cfg(feature = "mysql")]
            Verbindung::My(c) => {
                use mysql::prelude::Queryable;
                c.query_drop(sql).map_err(|e| format!("{}: {}", fn_, my::fehler(&e)))
            }
        }
    }

    /// Die Kennung der zuletzt eingefuegten Zeile (DB_LAST_ROWID). Bei
    /// PostgreSQL der letzte Wert einer Sequenz in dieser Verbindung
    /// (`lastval()`) -- das ist, was eine `SERIAL`-/`IDENTITY`-Spalte vergibt.
    pub fn letzte_id(&mut self) -> Result<i64, String> {
        match self {
            Verbindung::Sqlite(c) => Ok(c.last_insert_rowid()),
            #[cfg(feature = "pg")]
            Verbindung::Pg(c) => {
                let r = pg::query(c, "SELECT lastval()", &[]).map_err(|e| format!(
                    "DB_LAST_ROWID: {} -- in dieser Verbindung wurde noch keine Kennung vergeben; mit RETURNING id am INSERT bekommt man sie sicher", e))?;
                match r.rows.first().and_then(|z| z.first()) {
                    Some(DbVal::Int(n)) => Ok(*n),
                    _ => Ok(0),
                }
            }
            #[cfg(feature = "mysql")]
            Verbindung::My(c) => Ok(c.last_insert_id() as i64),
        }
    }

    /// Ein INSERT ausfuehren und die neue Kennung (Spalte `id`) liefern --
    /// fuer GUI_FORM_SAVE. PostgreSQL bekommt dafuer `RETURNING id`.
    #[cfg_attr(not(feature = "graphics"), allow(dead_code))]
    pub fn einfuegen(&mut self, sql: &str, params: &[Param], fn_: &str) -> Result<i64, String> {
        #[cfg(feature = "pg")]
        if let Verbindung::Pg(_) = self {
            let r = self.query(&format!("{} RETURNING id", sql), params, fn_)?;
            return Ok(match r.rows.first().and_then(|z| z.first()) { Some(DbVal::Int(n)) => *n, _ => 0 });
        }
        self.exec(sql, params, fn_)?;
        self.letzte_id()
    }

    /// Steht die Verbindung noch (DB_PING)? Eine Datei bricht nicht weg.
    pub fn lebt(&mut self) -> bool {
        match self {
            Verbindung::Sqlite(_) => true,
            #[cfg(feature = "pg")]
            Verbindung::Pg(c) => c.is_valid(std::time::Duration::from_secs(5)).is_ok(),
            #[cfg(feature = "mysql")]
            Verbindung::My(c) => c.ping().is_ok(),
        }
    }
}

fn sqlite_query(conn: &Connection, sql: &str, params: &[Param]) -> Result<DbResult, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let ncols = stmt.column_count();
    let columns: Vec<String> = (0..ncols)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();
    let mut out_rows: Vec<Vec<DbVal>> = Vec::new();
    let mut rows = stmt
        .query(rusqlite::params_from_iter(params.iter().map(sqlite_param)))
        .map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let mut r = Vec::with_capacity(ncols);
        for i in 0..ncols {
            let vr = row.get_ref(i).map_err(|e| e.to_string())?;
            r.push(match vr {
                ValueRef::Null => DbVal::Null,
                ValueRef::Integer(n) => DbVal::Int(n),
                ValueRef::Real(f) => DbVal::Real(f),
                ValueRef::Text(b) => DbVal::Text(String::from_utf8_lossy(b).into_owned()),
                ValueRef::Blob(b) => DbVal::Blob(b.to_vec()),
            });
        }
        out_rows.push(r);
    }
    Ok(DbResult { columns, rows: out_rows, pos: -1, closed: false })
}

// ------------------------------------------------------------------ Ziele und SQL

/// `sslmode=...` aus einem Ziel herausnehmen (beide Server verstehen es hier
/// gleich): das Ziel ohne den Schalter und der Wert (leer = nicht genannt).
#[cfg_attr(not(any(feature = "pg", feature = "mysql")), allow(dead_code))]
pub fn sslmode_herausnehmen(ziel: &str) -> (String, String) {
    let Some((vorn, abfrage)) = ziel.split_once('?') else { return (ziel.to_string(), String::new()) };
    let mut modus = String::new();
    let rest: Vec<&str> = abfrage.split('&').filter(|teil| {
        match teil.split_once('=') {
            Some((k, v)) if k.eq_ignore_ascii_case("sslmode") => { modus = v.to_ascii_lowercase(); false }
            _ => !teil.is_empty(),
        }
    }).collect();
    if rest.is_empty() { (vorn.to_string(), modus) } else { (format!("{}?{}", vorn, rest.join("&")), modus) }
}

#[cfg_attr(not(feature = "pg"), allow(dead_code))]
fn wortzeichen(c: char) -> bool { c.is_alphanumeric() || c == '_' }

/// Fuer PostgreSQL: `?` zu `$1`, `$2`, ... -- nur ausserhalb von
/// Zeichenketten ('..', E'..', $tag$..$tag$), Bezeichnern ("..") und
/// Kommentaren. Schreibt das SQL selbst schon `$1`, bleibt es, wie es ist.
#[cfg_attr(not(feature = "pg"), allow(dead_code))]
pub fn platzhalter_pg(sql: &str) -> String {
    let c: Vec<char> = sql.chars().collect();
    let mut aus = String::with_capacity(sql.len() + 8);
    let mut n = 0;
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        match ch {
            '\'' => {
                // E'...' kennt Rueckstrich-Folgen, '' ist immer ein Anfuehrungszeichen.
                let esc = i > 0 && (c[i - 1] == 'e' || c[i - 1] == 'E') && (i < 2 || !wortzeichen(c[i - 2]));
                let start = i;
                i += 1;
                while i < c.len() {
                    if esc && c[i] == '\\' { i += 2; continue; }
                    if c[i] == '\'' {
                        if i + 1 < c.len() && c[i + 1] == '\'' { i += 2; continue; }
                        break;
                    }
                    i += 1;
                }
                let ende = (i + 1).min(c.len());
                aus.extend(&c[start..ende]);
                i = ende;
                continue;
            }
            '"' => {
                let start = i;
                i += 1;
                while i < c.len() && c[i] != '"' { i += 1; }
                let ende = (i + 1).min(c.len());
                aus.extend(&c[start..ende]);
                i = ende;
                continue;
            }
            '-' if i + 1 < c.len() && c[i + 1] == '-' => {
                let start = i;
                while i < c.len() && c[i] != '\n' { i += 1; }
                aus.extend(&c[start..i]);
                continue;
            }
            '/' if i + 1 < c.len() && c[i + 1] == '*' => {
                // Block-Kommentare duerfen in PostgreSQL verschachtelt sein.
                let start = i;
                let mut tiefe = 0;
                while i < c.len() {
                    if c[i] == '/' && i + 1 < c.len() && c[i + 1] == '*' { tiefe += 1; i += 2; continue; }
                    if c[i] == '*' && i + 1 < c.len() && c[i + 1] == '/' { tiefe -= 1; i += 2; if tiefe == 0 { break; } continue; }
                    i += 1;
                }
                let ende = i.min(c.len());
                aus.extend(&c[start..ende]);
                i = ende;
                continue;
            }
            '$' if !(i > 0 && wortzeichen(c[i - 1])) => {
                // $1 im SQL: nichts uebersetzen.
                if i + 1 < c.len() && c[i + 1].is_ascii_digit() { return sql.to_string(); }
                // $tag$ ... $tag$ (auch $$ ... $$).
                let mut j = i + 1;
                while j < c.len() && wortzeichen(c[j]) { j += 1; }
                if j < c.len() && c[j] == '$' {
                    let tag: Vec<char> = c[i..=j].to_vec();
                    let mut k = j + 1;
                    let mut ende = c.len();
                    while k + tag.len() <= c.len() {
                        if c[k..k + tag.len()] == tag[..] { ende = k + tag.len(); break; }
                        k += 1;
                    }
                    aus.extend(&c[i..ende]);
                    i = ende;
                    continue;
                }
            }
            '?' => {
                n += 1;
                aus.push('$');
                aus.push_str(&n.to_string());
                i += 1;
                continue;
            }
            _ => {}
        }
        aus.push(ch);
        i += 1;
    }
    aus
}

/// Tage seit 1970-01-01 -> (Jahr, Monat, Tag) (Howard Hinnant, civil_from_days).
fn datum_aus_tagen(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg_attr(not(feature = "pg"), allow(dead_code))]
fn datum_text(tage_seit_1970: i64) -> String {
    let (j, m, t) = datum_aus_tagen(tage_seit_1970);
    if j <= 0 { format!("{:04}-{:02}-{:02} BC", 1 - j, m, t) } else { format!("{:04}-{:02}-{:02}", j, m, t) }
}

/// Mikrosekunden seit Mitternacht -> `HH:MM:SS[.ffffff]` (Stunden duerfen ueber 23 gehen).
#[cfg_attr(not(feature = "pg"), allow(dead_code))]
fn zeit_text(mikro: i64) -> String {
    let neg = mikro < 0;
    let m = mikro.unsigned_abs();
    let (s, us) = (m / 1_000_000, m % 1_000_000);
    let mut t = format!("{}{:02}:{:02}:{:02}", if neg { "-" } else { "" }, s / 3600, (s / 60) % 60, s % 60);
    if us != 0 { t.push_str(format!(".{:06}", us).trim_end_matches('0')); }
    t
}

// ------------------------------------------------------------------ PostgreSQL

#[cfg(feature = "pg")]
mod pg {
    use super::*;
    use postgres::types::{FromSql, Format, IsNull, Kind, ToSql, Type};

    pub fn fehler(e: &postgres::Error) -> String {
        if let Some(db) = e.as_db_error() {
            let mut t = format!("{} (PostgreSQL {})", db.message(), db.code().code());
            if let Some(d) = db.detail() { t.push_str(&format!(" -- {}", d)); }
            if let Some(h) = db.hint() { t.push_str(&format!(" -- {}", h)); }
            return t;
        }
        let t = e.to_string();
        // Die Ursache steht oft erst im Fehler darunter (Verbindung, TLS, Spalte).
        match std::error::Error::source(e) {
            Some(q) if !t.contains(&q.to_string()) => format!("{}: {}", t, q),
            _ => t,
        }
    }

    pub fn oeffnen(ziel: &str) -> Result<postgres::Client, String> {
        let (url, modus) = sslmode_herausnehmen(ziel);
        let mut cfg: postgres::Config = url.parse().map_err(|e| format!(
            "DB_OPEN: {} -- erwartet postgres://nutzer:kennwort@host:5432/datenbank", e))?;
        cfg.connect_timeout(std::time::Duration::from_secs(10));
        let r = match modus.as_str() {
            "disable" => cfg.connect(postgres::NoTls),
            "" | "prefer" | "require" | "verify-ca" | "verify-full" => {
                let pruefen = modus.starts_with("verify");
                cfg.ssl_mode(if modus.is_empty() || modus == "prefer" { postgres::config::SslMode::Prefer }
                             else { postgres::config::SslMode::Require });
                let tls = tokio_postgres_rustls::MakeRustlsConnect::new(super::tls_konfig(pruefen)?);
                cfg.connect(tls)
            }
            m => return Err(format!("DB_OPEN: sslmode={} gibt es nicht (disable, prefer, require, verify-full)", m)),
        };
        r.map_err(|e| format!("DB_OPEN: {}", fehler(&e)))
    }

    /// Ein Wert im TEXTformat: der Server wandelt ihn in den Typ der Stelle.
    struct Wert<'a>(&'a Param);

    impl std::fmt::Debug for Wert<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self.0) }
    }

    impl ToSql for Wert<'_> {
        fn to_sql(&self, ty: &Type, out: &mut bytes::BytesMut) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
            let text = match self.0 {
                Param::Null => return Ok(IsNull::Yes),
                Param::Int(n) => n.to_string(),
                Param::Real(f) if f.is_nan() => "NaN".into(),
                Param::Real(f) if f.is_infinite() => if *f > 0.0 { "Infinity".into() } else { "-Infinity".into() },
                Param::Real(f) => format!("{:?}", f),
                Param::Text(s) => {
                    if s.contains('\0') { return Err("ein Nullzeichen im Text -- PostgreSQL nimmt es nicht".into()); }
                    s.clone()
                }
                Param::Blob(b) => {
                    if *ty == Type::BYTEA {
                        let mut h = String::with_capacity(2 + b.len() * 2);
                        h.push_str("\\x");
                        for x in b { h.push_str(&format!("{:02x}", x)); }
                        h
                    } else {
                        String::from_utf8_lossy(b).into_owned()
                    }
                }
            };
            out.extend_from_slice(text.as_bytes());
            Ok(IsNull::No)
        }
        fn accepts(_: &Type) -> bool { true }
        fn encode_format(&self, _ty: &Type) -> Format { Format::Text }
        postgres::types::to_sql_checked!();
    }

    /// Eine Zelle, wie sie binaer ankommt -- entschluesselt nach ihrem Typ.
    struct Roh(DbVal);

    impl<'a> FromSql<'a> for Roh {
        fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            Ok(Roh(dekodieren(ty, raw)?))
        }
        fn accepts(_: &Type) -> bool { true }
    }

    fn zahl<const N: usize>(raw: &[u8]) -> Result<[u8; N], String> {
        raw.try_into().map_err(|_| format!("erwartet {} Bytes, erhalten {}", N, raw.len()))
    }

    /// Mikrosekunden seit 2000-01-01 -> Datum und Uhrzeit.
    fn zeitpunkt(us: i64) -> String {
        if us == i64::MAX { return "infinity".into(); }
        if us == i64::MIN { return "-infinity".into(); }
        let tage = us.div_euclid(86_400_000_000);
        let rest = us.rem_euclid(86_400_000_000);
        format!("{} {}", datum_text(tage + 10_957), zeit_text(rest))
    }

    pub fn dekodieren(ty: &Type, raw: &[u8]) -> Result<DbVal, String> {
        let text = || String::from_utf8_lossy(raw).into_owned();
        Ok(match *ty {
            Type::BOOL => DbVal::Int((raw.first().copied().unwrap_or(0) != 0) as i64),
            Type::INT2 => DbVal::Int(i16::from_be_bytes(zahl(raw)?) as i64),
            Type::INT4 => DbVal::Int(i32::from_be_bytes(zahl(raw)?) as i64),
            Type::INT8 => DbVal::Int(i64::from_be_bytes(zahl(raw)?)),
            Type::OID => DbVal::Int(u32::from_be_bytes(zahl(raw)?) as i64),
            Type::FLOAT4 => DbVal::Real(f32::from_be_bytes(zahl(raw)?) as f64),
            Type::FLOAT8 => DbVal::Real(f64::from_be_bytes(zahl(raw)?)),
            Type::NUMERIC => DbVal::Zahl(numeric_text(raw)?),
            // money: ganze Cent als int8 (zwei Nachkommastellen wie lc_monetary=C).
            Type::MONEY => {
                let n = i64::from_be_bytes(zahl(raw)?);
                DbVal::Zahl(format!("{}{}.{:02}", if n < 0 { "-" } else { "" }, n.unsigned_abs() / 100, n.unsigned_abs() % 100))
            }
            Type::BYTEA => DbVal::Blob(raw.to_vec()),
            Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME | Type::UNKNOWN | Type::CHAR
            | Type::JSON | Type::XML => DbVal::Text(text()),
            Type::JSONB => match raw.split_first() {
                Some((1, rest)) => DbVal::Text(String::from_utf8_lossy(rest).into_owned()),
                _ => return Err("jsonb in unbekannter Fassung".into()),
            },
            Type::UUID => {
                let b: [u8; 16] = zahl(raw)?;
                let h: String = b.iter().map(|x| format!("{:02x}", x)).collect();
                DbVal::Text(format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32]))
            }
            Type::DATE => {
                let t = i32::from_be_bytes(zahl(raw)?);
                DbVal::Text(match t { i32::MAX => "infinity".into(), i32::MIN => "-infinity".into(),
                                      t => datum_text(t as i64 + 10_957) })
            }
            Type::TIME => DbVal::Text(zeit_text(i64::from_be_bytes(zahl(raw)?))),
            Type::TIMESTAMP => DbVal::Text(zeitpunkt(i64::from_be_bytes(zahl(raw)?))),
            // timestamptz kommt in UTC -- so steht es auch da.
            Type::TIMESTAMPTZ => {
                let t = zeitpunkt(i64::from_be_bytes(zahl(raw)?));
                DbVal::Text(if t.ends_with("infinity") { t } else { format!("{}+00", t) })
            }
            Type::INTERVAL => {
                let b: [u8; 16] = zahl(raw)?;
                let us = i64::from_be_bytes(b[0..8].try_into().unwrap());
                let tage = i32::from_be_bytes(b[8..12].try_into().unwrap());
                let monate = i32::from_be_bytes(b[12..16].try_into().unwrap());
                let mut teile: Vec<String> = Vec::new();
                let (j, m) = (monate / 12, monate % 12);
                if j != 0 { teile.push(format!("{} year{}", j, if j.abs() == 1 { "" } else { "s" })); }
                if m != 0 { teile.push(format!("{} mon{}", m, if m.abs() == 1 { "" } else { "s" })); }
                if tage != 0 { teile.push(format!("{} day{}", tage, if tage.abs() == 1 { "" } else { "s" })); }
                if us != 0 || teile.is_empty() { teile.push(zeit_text(us)); }
                DbVal::Text(teile.join(" "))
            }
            _ => match ty.kind() {
                Kind::Enum(_) => DbVal::Text(text()),
                Kind::Domain(inner) => dekodieren(inner, raw)?,
                _ => return Err(format!("den Typ {} liest Drachenhauch (noch) nicht -- in der Abfrage mit ::text umwandeln", ty.name())),
            },
        })
    }

    /// numeric binaer -> Dezimaltext: Ziffern zur Basis 10000, `weight` ist
    /// der Exponent der ersten, `dscale` die Zahl der Nachkommastellen.
    pub fn numeric_text(raw: &[u8]) -> Result<String, String> {
        if raw.len() < 8 { return Err("numeric zu kurz".into()); }
        let n = u16::from_be_bytes([raw[0], raw[1]]) as usize;
        let weight = i16::from_be_bytes([raw[2], raw[3]]) as i32;
        let sign = u16::from_be_bytes([raw[4], raw[5]]);
        let dscale = u16::from_be_bytes([raw[6], raw[7]]) as usize;
        match sign {
            0xC000 => return Ok("NaN".into()),
            0xD000 => return Ok("Infinity".into()),
            0xF000 => return Ok("-Infinity".into()),
            _ => {}
        }
        if raw.len() < 8 + 2 * n { return Err("numeric zu kurz".into()); }
        let ziffer = |i: i32| -> u16 {
            if i < 0 || i as usize >= n { 0 } else { u16::from_be_bytes([raw[8 + 2 * i as usize], raw[9 + 2 * i as usize]]) }
        };
        let mut t = String::new();
        if sign == 0x4000 { t.push('-'); }
        if weight < 0 {
            t.push('0');
        } else {
            for i in 0..=weight {
                let d = ziffer(i);
                if i == 0 { t.push_str(&d.to_string()); } else { t.push_str(&format!("{:04}", d)); }
            }
        }
        if dscale > 0 {
            let mut nach = String::new();
            let mut i = weight + 1;
            while nach.len() < dscale {
                nach.push_str(&format!("{:04}", ziffer(i)));
                i += 1;
            }
            nach.truncate(dscale);
            t.push('.');
            t.push_str(&nach);
        }
        if t.starts_with('-') && t[1..].chars().all(|c| c == '0' || c == '.') { t.remove(0); }
        Ok(t)
    }

    fn vorbereiten(c: &mut postgres::Client, sql: &str, params: &[Param]) -> Result<postgres::Statement, String> {
        let stmt = c.prepare(&platzhalter_pg(sql)).map_err(|e| fehler(&e))?;
        if stmt.params().len() != params.len() {
            return Err(format!("die Abfrage hat {} Platzhalter, uebergeben sind {} Werte", stmt.params().len(), params.len()));
        }
        Ok(stmt)
    }

    pub fn exec(c: &mut postgres::Client, sql: &str, params: &[Param]) -> Result<i64, String> {
        let stmt = vorbereiten(c, sql, params)?;
        let werte: Vec<Wert> = params.iter().map(Wert).collect();
        let refs: Vec<&(dyn ToSql + Sync)> = werte.iter().map(|w| w as &(dyn ToSql + Sync)).collect();
        c.execute(&stmt, &refs).map(|n| n as i64).map_err(|e| fehler(&e))
    }

    pub fn query(c: &mut postgres::Client, sql: &str, params: &[Param]) -> Result<DbResult, String> {
        let stmt = vorbereiten(c, sql, params)?;
        let columns: Vec<String> = stmt.columns().iter().map(|k| k.name().to_string()).collect();
        let werte: Vec<Wert> = params.iter().map(Wert).collect();
        let refs: Vec<&(dyn ToSql + Sync)> = werte.iter().map(|w| w as &(dyn ToSql + Sync)).collect();
        let zeilen = c.query(&stmt, &refs).map_err(|e| fehler(&e))?;
        let mut rows = Vec::with_capacity(zeilen.len());
        for z in &zeilen {
            let mut r = Vec::with_capacity(columns.len());
            for (i, name) in columns.iter().enumerate() {
                // Die eigene Meldung steht als Ursache unter "error deserializing column".
                let v: Option<Roh> = z.try_get(i).map_err(|e| format!("Spalte {} ({}): {}", i, name,
                    std::error::Error::source(&e).map(|q| q.to_string()).unwrap_or_else(|| fehler(&e))))?;
                r.push(v.map(|x| x.0).unwrap_or(DbVal::Null));
            }
            rows.push(r);
        }
        Ok(DbResult { columns, rows, pos: -1, closed: false })
    }
}

// ------------------------------------------------------------------ MySQL

#[cfg(feature = "mysql")]
mod my {
    use super::*;
    use mysql::consts::ColumnType as T;
    use mysql::prelude::Queryable;

    pub fn fehler(e: &mysql::Error) -> String {
        match e {
            mysql::Error::MySqlError(m) => format!("{} (MySQL {})", m.message, m.code),
            mysql::Error::DriverError(d) => d.to_string(),
            mysql::Error::IoError(i) => i.to_string(),
            e => e.to_string(),
        }
    }

    pub fn oeffnen(ziel: &str) -> Result<mysql::Conn, String> {
        let (mut url, modus) = sslmode_herausnehmen(ziel);
        if url.to_ascii_lowercase().starts_with("mariadb://") { url = format!("mysql://{}", &url[10..]); }
        let opts = mysql::Opts::from_url(&url).map_err(|e| format!(
            "DB_OPEN: {} -- erwartet mysql://nutzer:kennwort@host:3306/datenbank", e))?;
        // prefer_socket aus: sonst wechselt der Treiber bei einem Server auf
        // demselben Rechner nach dem Verbinden auf dessen Unix-Socket -- und
        // ueber die Socket gibt es kein TLS; `sslmode=require` verbaende dann
        // stillschweigend unverschluesselt (gefunden in der CI unter Linux,
        // unter Windows gibt es die Socket nicht). Verbunden wird an die
        // Adresse, die im Ziel steht.
        let basis = || mysql::OptsBuilder::from_opts(opts.clone())
            .prefer_socket(false)
            .tcp_connect_timeout(Some(std::time::Duration::from_secs(10)));
        let mit_tls = |pruefen: bool, b: mysql::OptsBuilder| {
            let ssl = if pruefen { mysql::SslOpts::default() } else {
                mysql::SslOpts::default().with_danger_accept_invalid_certs(true).with_danger_skip_domain_validation(true)
            };
            b.ssl_opts(ssl)
        };
        let verbinden = |b: mysql::OptsBuilder| -> Result<mysql::Conn, mysql::Error> {
            match modus.as_str() {
                "disable" => mysql::Conn::new(b),
                "require" | "verify-ca" => mysql::Conn::new(mit_tls(false, b)),
                "verify-full" => mysql::Conn::new(mit_tls(true, b)),
                // prefer: verschluesselt, wenn der Server es kann, sonst ohne. Ein
                // Fehler des Servers (Kennwort, Datenbank) gilt dagegen sofort.
                _ => match mysql::Conn::new(mit_tls(false, b.clone())) {
                    Ok(c) => Ok(c),
                    Err(e @ mysql::Error::MySqlError(_)) => Err(e),
                    Err(e) => mysql::Conn::new(b).map_err(|_| e),
                },
            }
        };
        if !matches!(modus.as_str(), "" | "prefer" | "disable" | "require" | "verify-ca" | "verify-full") {
            return Err(format!("DB_OPEN: sslmode={} gibt es nicht (disable, prefer, require, verify-full)", modus));
        }
        let mut bester = match verbinden(basis()) {
            Ok(c) => return Ok(c),
            Err(e) => e,
        };
        // Der Treiber versucht nur die ERSTE Adresse eines Namens -- `localhost`
        // ist unter Windows zuerst ::1, und ein Server, der nur auf IPv4
        // lauscht, waere so nie erreichbar. Darum die weiteren Adressen selbst
        // (nicht bei verify-full: dort zaehlt der Name fuer das Zertifikat, und
        // der Treiber nimmt ihn nicht getrennt von der Adresse). Antwortet
        // unter einer davon der Server selbst (Kennwort, Datenbank), zaehlt
        // seine Meldung -- nicht die verweigerte Verbindung davor.
        if !matches!(bester, mysql::Error::MySqlError(_)) && modus != "verify-full" {
            use std::net::ToSocketAddrs;
            let host = opts.get_ip_or_hostname().to_string();
            if let Ok(adressen) = (host.as_str(), opts.get_tcp_port()).to_socket_addrs() {
                for a in adressen.skip(1) {
                    match verbinden(basis().ip_or_hostname(Some(a.ip().to_string()))) {
                        Ok(c) => return Ok(c),
                        Err(e @ mysql::Error::MySqlError(_)) => { bester = e; break; }
                        Err(_) => {}
                    }
                }
            }
        }
        Err(format!("DB_OPEN: {}", fehler(&bester)))
    }

    fn wert(p: &Param) -> mysql::Value {
        match p {
            Param::Null => mysql::Value::NULL,
            Param::Int(n) => mysql::Value::Int(*n),
            Param::Real(f) => mysql::Value::Double(*f),
            Param::Text(s) => mysql::Value::Bytes(s.as_bytes().to_vec()),
            Param::Blob(b) => mysql::Value::Bytes(b.clone()),
        }
    }

    fn params(ps: &[Param]) -> mysql::Params {
        if ps.is_empty() { mysql::Params::Empty } else { mysql::Params::Positional(ps.iter().map(wert).collect()) }
    }

    fn vorbereiten(c: &mut mysql::Conn, sql: &str, ps: &[Param]) -> Result<mysql::Statement, String> {
        let stmt = c.prep(sql).map_err(|e| fehler(&e))?;
        if stmt.num_params() as usize != ps.len() {
            return Err(format!("die Abfrage hat {} Platzhalter, uebergeben sind {} Werte", stmt.num_params(), ps.len()));
        }
        Ok(stmt)
    }

    pub fn exec(c: &mut mysql::Conn, sql: &str, ps: &[Param]) -> Result<i64, String> {
        let stmt = vorbereiten(c, sql, ps)?;
        c.exec_drop(&stmt, params(ps)).map_err(|e| fehler(&e))?;
        Ok(c.affected_rows() as i64)
    }

    /// Ein Wert, wie MySQL ihn schickt, samt der Spalte (fuer Bytes: Text,
    /// Bytes oder Dezimalzahl entscheidet erst ihr Typ).
    pub fn zelle(v: mysql::Value, typ: T, binaer: bool) -> DbVal {
        let mikro = |us: u32| if us == 0 { String::new() } else { format!(".{:06}", us).trim_end_matches('0').to_string() };
        match v {
            mysql::Value::NULL => DbVal::Null,
            mysql::Value::Int(n) => DbVal::Int(n),
            mysql::Value::UInt(n) => i64::try_from(n).map(DbVal::Int).unwrap_or_else(|_| DbVal::Zahl(n.to_string())),
            mysql::Value::Float(f) => DbVal::Real(f as f64),
            mysql::Value::Double(f) => DbVal::Real(f),
            mysql::Value::Date(j, m, t, h, mi, s, us) => DbVal::Text(if typ == T::MYSQL_TYPE_DATE {
                format!("{:04}-{:02}-{:02}", j, m, t)
            } else {
                format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}{}", j, m, t, h, mi, s, mikro(us))
            }),
            mysql::Value::Time(neg, tage, h, m, s, us) => DbVal::Text(format!("{}{:02}:{:02}:{:02}{}",
                if neg { "-" } else { "" }, tage * 24 + h as u32, m, s, mikro(us))),
            mysql::Value::Bytes(b) => match typ {
                T::MYSQL_TYPE_NEWDECIMAL | T::MYSQL_TYPE_DECIMAL => DbVal::Zahl(String::from_utf8_lossy(&b).into_owned()),
                T::MYSQL_TYPE_BIT => DbVal::Int(b.iter().fold(0i64, |a, x| (a << 8) | *x as i64)),
                T::MYSQL_TYPE_TINY | T::MYSQL_TYPE_SHORT | T::MYSQL_TYPE_LONG | T::MYSQL_TYPE_LONGLONG
                | T::MYSQL_TYPE_INT24 | T::MYSQL_TYPE_YEAR => {
                    let s = String::from_utf8_lossy(&b).into_owned();
                    s.parse::<i64>().map(DbVal::Int).unwrap_or(DbVal::Zahl(s))
                }
                T::MYSQL_TYPE_FLOAT | T::MYSQL_TYPE_DOUBLE => {
                    let s = String::from_utf8_lossy(&b).into_owned();
                    s.parse::<f64>().map(DbVal::Real).unwrap_or(DbVal::Text(s))
                }
                T::MYSQL_TYPE_TINY_BLOB | T::MYSQL_TYPE_MEDIUM_BLOB | T::MYSQL_TYPE_LONG_BLOB | T::MYSQL_TYPE_BLOB
                | T::MYSQL_TYPE_VAR_STRING | T::MYSQL_TYPE_STRING | T::MYSQL_TYPE_VARCHAR | T::MYSQL_TYPE_GEOMETRY
                    if binaer => DbVal::Blob(b),
                _ => DbVal::Text(String::from_utf8_lossy(&b).into_owned()),
            },
        }
    }

    pub fn query(c: &mut mysql::Conn, sql: &str, ps: &[Param]) -> Result<DbResult, String> {
        let stmt = vorbereiten(c, sql, ps)?;
        let spalten: Vec<(String, T, bool)> = stmt.columns().iter()
            .map(|k| (k.name_str().into_owned(), k.column_type(), k.character_set() == 63)).collect();
        let ergebnis = c.exec_iter(&stmt, params(ps)).map_err(|e| fehler(&e))?;
        let mut rows = Vec::new();
        for z in ergebnis {
            let z = z.map_err(|e| fehler(&e))?;
            rows.push(z.unwrap().into_iter().enumerate().map(|(i, v)| {
                let (typ, binaer) = spalten.get(i).map(|s| (s.1, s.2)).unwrap_or((T::MYSQL_TYPE_VAR_STRING, false));
                zelle(v, typ, binaer)
            }).collect());
        }
        Ok(DbResult { columns: spalten.into_iter().map(|s| s.0).collect(), rows, pos: -1, closed: false })
    }
}

/// TLS fuer PostgreSQL: rustls mit ring wie smtp/ureq. Ohne Pruefung des
/// Zertifikats (`sslmode=prefer/require`, wie libpq: verschluesselt, aber
/// jedem Server geglaubt -- die meisten eigenen Server haben ein selbst
/// ausgestelltes Zertifikat) oder mit (`verify-full`, gegen die Wurzeln von
/// webpki-roots).
#[cfg(feature = "pg")]
fn tls_konfig(pruefen: bool) -> Result<rustls::ClientConfig, String> {
    use std::sync::Arc;
    let anbieter = Arc::new(rustls::crypto::ring::default_provider());
    let stufe = rustls::ClientConfig::builder_with_provider(anbieter.clone())
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("DB_OPEN: TLS liess sich nicht einrichten: {}", e))?;
    Ok(if pruefen {
        let mut wurzeln = rustls::RootCertStore::empty();
        wurzeln.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        stufe.with_root_certificates(wurzeln).with_no_client_auth()
    } else {
        stufe.dangerous().with_custom_certificate_verifier(Arc::new(OhnePruefung(anbieter))).with_no_client_auth()
    })
}

#[cfg(feature = "pg")]
#[derive(Debug)]
struct OhnePruefung(std::sync::Arc<rustls::crypto::CryptoProvider>);

#[cfg(feature = "pg")]
impl rustls::client::danger::ServerCertVerifier for OhnePruefung {
    fn verify_server_cert(&self, _e: &rustls::pki_types::CertificateDer<'_>, _i: &[rustls::pki_types::CertificateDer<'_>],
                          _s: &rustls::pki_types::ServerName<'_>, _o: &[u8], _n: rustls::pki_types::UnixTime)
                          -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(&self, m: &[u8], c: &rustls::pki_types::CertificateDer<'_>, d: &rustls::DigitallySignedStruct)
                              -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(m, c, d, &self.0.signature_verification_algorithms)
    }
    fn verify_tls13_signature(&self, m: &[u8], c: &rustls::pki_types::CertificateDer<'_>, d: &rustls::DigitallySignedStruct)
                              -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(m, c, d, &self.0.signature_verification_algorithms)
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ziel_entscheidet_die_datenbank() {
        assert_eq!(art_von("spiel.db"), "sqlite");
        assert_eq!(art_von(":memory:"), "sqlite");
        assert_eq!(art_von("postgres://a@b/c"), "postgres");
        assert_eq!(art_von("PostgreSQL://a@b/c"), "postgres");
        assert_eq!(art_von("mysql://a@b/c"), "mysql");
        assert_eq!(art_von("mariadb://a@b/c"), "mysql");
    }

    #[test]
    fn sslmode_wird_herausgenommen() {
        assert_eq!(sslmode_herausnehmen("postgres://h/d"), ("postgres://h/d".into(), String::new()));
        assert_eq!(sslmode_herausnehmen("postgres://h/d?sslmode=Require"), ("postgres://h/d".into(), "require".into()));
        assert_eq!(sslmode_herausnehmen("mysql://h/d?a=1&sslmode=disable&b=2"), ("mysql://h/d?a=1&b=2".into(), "disable".into()));
    }

    #[test]
    fn fragezeichen_werden_zu_dollar() {
        assert_eq!(platzhalter_pg("SELECT * FROM t WHERE a = ? AND b = ?"), "SELECT * FROM t WHERE a = $1 AND b = $2");
        // In Zeichenketten, Bezeichnern und Kommentaren bleibt es.
        assert_eq!(platzhalter_pg("SELECT '?', \"wer?\", ? -- noch ?\n, ?"), "SELECT '?', \"wer?\", $1 -- noch ?\n, $2");
        assert_eq!(platzhalter_pg("SELECT 'it''s ?', ? /* a /* ? */ ? */ ?"), "SELECT 'it''s ?', $1 /* a /* ? */ ? */ $2");
        assert_eq!(platzhalter_pg("SELECT E'\\'?', ?"), "SELECT E'\\'?', $1");
        assert_eq!(platzhalter_pg("SELECT $$ein ? hier$$, ?, $tag$?$tag$"), "SELECT $$ein ? hier$$, $1, $tag$?$tag$");
        // Wer selbst $1 schreibt, bekommt sein SQL unveraendert.
        assert_eq!(platzhalter_pg("SELECT $1, '?'"), "SELECT $1, '?'");
        assert_eq!(platzhalter_pg("SELECT preis$1 FROM t WHERE x = ?"), "SELECT preis$1 FROM t WHERE x = $1");
    }

    #[test]
    fn datum_und_zeit() {
        assert_eq!(datum_text(0), "1970-01-01");
        assert_eq!(datum_text(10_957), "2000-01-01");
        assert_eq!(datum_text(19_723), "2024-01-01");
        assert_eq!(datum_text(-1), "1969-12-31");
        assert_eq!(datum_text(11_016), "2000-02-29");
        assert_eq!(zeit_text(0), "00:00:00");
        assert_eq!(zeit_text(((13 * 60 + 5) * 60 + 9) * 1_000_000 + 250_000), "13:05:09.25");
        assert_eq!(zeit_text(-90 * 1_000_000), "-00:01:30");
    }

    #[test]
    fn dezimalzahlen_bleiben_genau() {
        let r = DbResult { columns: vec!["a".into()], rows: vec![vec![DbVal::Zahl("19.99".into())], vec![DbVal::Zahl("12.000".into())]], pos: 0, closed: false };
        assert_eq!(r.get_string(0).unwrap(), "19.99");
        assert_eq!(r.get_float(0).unwrap(), 19.99);
        assert!(r.get_int(0).unwrap_err().contains("Dezimalzahl"));
        let r2 = DbResult { pos: 1, ..r };
        assert_eq!(r2.get_int(0).unwrap(), 12);
    }

    #[cfg(feature = "pg")]
    #[test]
    fn numeric_aus_dem_binaerformat() {
        // (Ziffern zur Basis 10000, weight, sign, dscale)
        fn num(ziffern: &[u16], weight: i16, sign: u16, dscale: u16) -> Vec<u8> {
            let mut b = Vec::new();
            b.extend((ziffern.len() as u16).to_be_bytes());
            b.extend(weight.to_be_bytes());
            b.extend(sign.to_be_bytes());
            b.extend(dscale.to_be_bytes());
            for z in ziffern { b.extend(z.to_be_bytes()); }
            b
        }
        assert_eq!(pg::numeric_text(&num(&[19, 9900], 0, 0, 2)).unwrap(), "19.99");
        assert_eq!(pg::numeric_text(&num(&[1, 2345, 6789], 1, 0x4000, 4)).unwrap(), "-12345.6789");
        assert_eq!(pg::numeric_text(&num(&[], 0, 0, 0)).unwrap(), "0");
        assert_eq!(pg::numeric_text(&num(&[], 0, 0, 3)).unwrap(), "0.000");
        assert_eq!(pg::numeric_text(&num(&[50], -1, 0, 3)).unwrap(), "0.005");
        assert_eq!(pg::numeric_text(&num(&[5], -2, 0, 8)).unwrap(), "0.00000005");
        assert_eq!(pg::numeric_text(&num(&[1], 2, 0, 0)).unwrap(), "100000000");
        assert_eq!(pg::numeric_text(&num(&[], 0, 0xC000, 0)).unwrap(), "NaN");
    }
}
