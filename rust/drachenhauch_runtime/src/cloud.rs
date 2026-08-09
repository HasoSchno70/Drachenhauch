//! Modul `cloud`: Cloud-Save + Leaderboard gegen den mitgelieferten
//! Referenz-Server (`cloudserver/server.py`, REST+JSON). HTTP via `ureq`
//! (wie html.rs), Feature `http`. Kein eigener State hier -- die VM haelt
//! Basis-URL/API-Key/letzte Fehlermeldung/Leaderboard-Ergebnisse
//! (`vm.rs`, analog zu db_conns/db_results).

#![cfg(feature = "http")]

use std::io::Read;
use std::time::Duration;

const USER_AGENT: &str = "GameBasic-cloud/0.1";
const TIMEOUT_SECS: u64 = 10;

pub struct CloudErr {
    pub msg: String,
    /// HTTP-Status, 0 = Transport-Fehler (kein Server erreicht).
    pub status: i64,
}

fn map_transport_err(e: ureq::Error, ctx: &str) -> CloudErr {
    match e {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            CloudErr { msg: format!("{}: HTTP {} - {}", ctx, code, body.trim()), status: code as i64 }
        }
        ureq::Error::Transport(t) => CloudErr { msg: format!("{}: {}", ctx, t), status: 0 },
    }
}

fn req_json(method: &str, url: &str, api_key: &str, body: Option<&serde_json::Value>, ctx: &str)
    -> Result<serde_json::Value, CloudErr>
{
    let mut req = match method {
        "GET" => ureq::get(url),
        "POST" => ureq::post(url),
        _ => unreachable!(),
    }
    .set("User-Agent", USER_AGENT)
    .timeout(Duration::from_secs(TIMEOUT_SECS));
    if !api_key.is_empty() {
        req = req.set("X-Api-Key", api_key);
    }
    let resp = match body {
        Some(b) => req.set("Content-Type", "application/json").send_string(&b.to_string()),
        None => req.call(),
    };
    let resp = resp.map_err(|e| map_transport_err(e, ctx))?;
    let mut text = String::new();
    resp.into_reader().read_to_string(&mut text).map_err(|e| CloudErr {
        msg: format!("{}: Antwort nicht lesbar: {}", ctx, e),
        status: 0,
    })?;
    serde_json::from_str(&text).map_err(|e| CloudErr {
        msg: format!("{}: Antwort ist kein gueltiges JSON: {}", ctx, e),
        status: 0,
    })
}

/// POST /save/<player_id> {"data": data}. Ok(()) bei Erfolg.
pub fn save_upload(base_url: &str, api_key: &str, player_id: &str, data: &str) -> Result<(), CloudErr> {
    let url = format!("{}/save/{}", base_url.trim_end_matches('/'), player_id);
    let body = serde_json::json!({ "data": data });
    req_json("POST", &url, api_key, Some(&body), "CLOUD_SAVE")?;
    Ok(())
}

/// GET /save/<player_id> -> Some(data) oder None (nicht gefunden, HTTP 404).
pub fn save_download(base_url: &str, api_key: &str, player_id: &str) -> Result<Option<String>, CloudErr> {
    let url = format!("{}/save/{}", base_url.trim_end_matches('/'), player_id);
    match req_json("GET", &url, api_key, None, "CLOUD_LOAD") {
        Ok(v) => Ok(Some(v.get("data").and_then(|d| d.as_str()).unwrap_or("").to_string())),
        Err(e) if e.status == 404 => Ok(None),
        Err(e) => Err(e),
    }
}

/// POST /leaderboard/<board>/submit {"name","score","best"}.
pub fn leaderboard_submit(base_url: &str, api_key: &str, board: &str, name: &str, score: f64, best_low: bool)
    -> Result<bool, CloudErr>
{
    let url = format!("{}/leaderboard/{}/submit", base_url.trim_end_matches('/'), board);
    let body = serde_json::json!({ "name": name, "score": score, "best": if best_low { "low" } else { "high" } });
    let v = req_json("POST", &url, api_key, Some(&body), "LEADERBOARD_SUBMIT")?;
    Ok(v.get("updated").and_then(|b| b.as_bool()).unwrap_or(false))
}

pub struct LeaderboardEntry {
    pub name: String,
    pub score: f64,
}

/// GET /leaderboard/<board>/top?n=N&order=... -> sortierte Eintraege.
pub fn leaderboard_fetch(base_url: &str, api_key: &str, board: &str, n: i64, ascending: bool)
    -> Result<Vec<LeaderboardEntry>, CloudErr>
{
    let order = if ascending { "asc" } else { "desc" };
    let url = format!("{}/leaderboard/{}/top?n={}&order={}", base_url.trim_end_matches('/'), board, n.max(1), order);
    let v = req_json("GET", &url, api_key, None, "LEADERBOARD_FETCH")?;
    let entries = v.get("entries").and_then(|e| e.as_array()).cloned().unwrap_or_default();
    Ok(entries.into_iter().map(|e| LeaderboardEntry {
        name: e.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
        score: e.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0),
    }).collect())
}
