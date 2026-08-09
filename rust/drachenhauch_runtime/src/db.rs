//! SQLite-Modul (DB_*) -- nativer Port von `drachenhauch/modules/db.py` via
//! `rusqlite` (bundled SQLite). Feature `db`.
//!
//! DB_CONN/DB_RESULT sind INTEGER-Handles (Index in VM-seitige Vecs). Ein
//! DB_QUERY laedt ALLE Zeilen eager in den Speicher (vermeidet self-
//! referenzielle Cursor-Lifetimes; fuer Game-DBs unkritisch).
#![cfg(feature = "db")]

use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::Connection;

use crate::value::Value;

/// Eine gespeicherte Zelle (SQLite-Spaltenwert).
pub enum DbVal {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
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
            DbVal::Text(s) => s.clone(),
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
            v => Err(format!("DB_GET_INT: Spalte {} ist {}, nicht INTEGER", idx, type_name(v))),
        }
    }
    pub fn get_float(&self, idx: i64) -> Result<f64, String> {
        match self.cur_value(idx, "DB_GET_FLOAT")? {
            DbVal::Null => Ok(0.0),
            DbVal::Int(n) => Ok(*n as f64),
            DbVal::Real(f) => Ok(*f),
            v => Err(format!("DB_GET_FLOAT: Spalte {} ist {}, nicht Zahl", idx, type_name(v))),
        }
    }
    pub fn get_bool(&self, idx: i64) -> Result<bool, String> {
        match self.cur_value(idx, "DB_GET_BOOL")? {
            DbVal::Null => Ok(false),
            DbVal::Int(n) => Ok(*n != 0),
            DbVal::Real(f) => Ok(*f != 0.0),
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
    }
}

/// GB-Wert -> SQLite-Bind-Wert (bool -> 0/1, Nil -> NULL).
pub fn gb_to_sql(v: &Value, fn_: &str) -> Result<SqlValue, String> {
    Ok(match v {
        Value::Nil => SqlValue::Null,
        Value::Bool(b) => SqlValue::Integer(if *b { 1 } else { 0 }),
        Value::Int(n) => SqlValue::Integer(*n),
        Value::Float(f) => SqlValue::Real(*f),
        Value::Str(s) => SqlValue::Text(s.to_string()),
        other => return Err(format!(
            "{}: Parameter-Typ {} nicht unterstuetzt (erlaubt: INTEGER, FLOAT, STRING, BOOLEAN, NIL)",
            fn_, other.type_name())),
    })
}

pub fn open(path: &str) -> Result<Connection, String> {
    Connection::open(path).map_err(|e| format!("DB_OPEN: {}", e))
}

pub fn exec(conn: &Connection, sql: &str, params: &[SqlValue]) -> Result<i64, String> {
    conn.execute(sql, rusqlite::params_from_iter(params.iter()))
        .map(|n| n as i64)
        .map_err(|e| format!("DB_EXEC: {}", e))
}

pub fn query(conn: &Connection, sql: &str, params: &[SqlValue]) -> Result<DbResult, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| format!("DB_QUERY: {}", e))?;
    let ncols = stmt.column_count();
    let columns: Vec<String> = (0..ncols)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();
    let mut out_rows: Vec<Vec<DbVal>> = Vec::new();
    let mut rows = stmt
        .query(rusqlite::params_from_iter(params.iter()))
        .map_err(|e| format!("DB_QUERY: {}", e))?;
    while let Some(row) = rows.next().map_err(|e| format!("DB_QUERY: {}", e))? {
        let mut r = Vec::with_capacity(ncols);
        for i in 0..ncols {
            let vr = row.get_ref(i).map_err(|e| format!("DB_QUERY: {}", e))?;
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
