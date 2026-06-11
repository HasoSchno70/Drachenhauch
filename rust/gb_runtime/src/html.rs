//! HTTP- + HTML-Modul (HTTP_*/URL_*/HTML_*) -- nativer Port von
//! `gamebasic/modules/html.py`. HTTP via `ureq`, HTML-Parsing als
//! handgeschriebener Scanner (funktional, nicht byte-identisch zu Pythons
//! html.parser bei kaputtem HTML). Feature `http`.
#![cfg(feature = "http")]

use std::io::Read;
use std::time::Duration;

const USER_AGENT: &str = "GameBasic/0.1 (+https://github.com/anthropic-ai)";
const TIMEOUT_SECS: u64 = 10;

pub struct HttpResult {
    pub status: i64,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub struct HttpErr {
    pub msg: String,
    pub status: i64, // 0 = Status nicht aktualisieren (Transport-Fehler)
    pub headers: Vec<(String, String)>,
}

fn collect_headers(resp: &ureq::Response) -> Vec<(String, String)> {
    resp.headers_names()
        .into_iter()
        .filter_map(|n| resp.header(&n).map(|v| (n.to_lowercase(), v.to_string())))
        .collect()
}

fn finish(resp: ureq::Response) -> HttpResult {
    let status = resp.status() as i64;
    let headers = collect_headers(&resp);
    let mut body = Vec::new();
    resp.into_reader().read_to_end(&mut body).ok();
    HttpResult { status, headers, body }
}

fn map_err(e: ureq::Error, url: &str) -> HttpErr {
    match e {
        ureq::Error::Status(code, resp) => {
            let headers = collect_headers(&resp);
            HttpErr {
                msg: format!("HTTP {} {} - {}", code, resp.status_text(), url),
                status: code as i64,
                headers,
            }
        }
        ureq::Error::Transport(t) => HttpErr {
            msg: format!("HTTP-Fehler: {} - {}", t, url),
            status: 0,
            headers: vec![],
        },
    }
}

pub fn http_get(url: &str) -> Result<HttpResult, HttpErr> {
    ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .call()
        .map(finish)
        .map_err(|e| map_err(e, url))
}

pub fn http_post(url: &str, body: &str) -> Result<HttpResult, HttpErr> {
    ureq::post(url)
        .set("User-Agent", USER_AGENT)
        .set("Content-Type", "application/x-www-form-urlencoded; charset=utf-8")
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .send_string(body)
        .map(finish)
        .map_err(|e| map_err(e, url))
}

// --- URL-Encoding -----------------------------------------------------------

pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push('%');
            out.push_str(&format!("{:02X}", b));
        }
    }
    out
}

pub fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h = hex_val(bytes[i + 1]);
            let l = hex_val(bytes[i + 2]);
            if let (Some(h), Some(l)) = (h, l) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// --- HTML-Entities ----------------------------------------------------------

fn named_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "amp" => "&", "lt" => "<", "gt" => ">", "quot" => "\"", "apos" => "'",
        "nbsp" => "\u{a0}", "copy" => "©", "reg" => "®", "trade" => "™",
        "mdash" => "—", "ndash" => "–", "hellip" => "…", "deg" => "°",
        "euro" => "€", "pound" => "£", "cent" => "¢", "sect" => "§",
        "laquo" => "«", "raquo" => "»", "middot" => "·", "bull" => "•",
        "ldquo" => "\u{201c}", "rdquo" => "\u{201d}", "lsquo" => "\u{2018}", "rsquo" => "\u{2019}",
        "auml" => "ä", "ouml" => "ö", "uuml" => "ü", "Auml" => "Ä", "Ouml" => "Ö", "Uuml" => "Ü",
        "szlig" => "ß", "eacute" => "é", "egrave" => "è", "agrave" => "à",
        _ => return None,
    })
}

/// Dekodiert &amp;/&#NN;/&#xHH;/benannte Entities. Unbekannte bleiben literal.
fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' {
            if let Some(semi) = chars[i + 1..].iter().position(|&c| c == ';').map(|p| i + 1 + p) {
                if semi - i <= 12 {
                    let ent: String = chars[i + 1..semi].iter().collect();
                    let decoded = if let Some(rest) = ent.strip_prefix('#') {
                        let code = if let Some(hex) = rest.strip_prefix(['x', 'X']) {
                            u32::from_str_radix(hex, 16).ok()
                        } else {
                            rest.parse::<u32>().ok()
                        };
                        code.and_then(char::from_u32).map(|c| c.to_string())
                    } else {
                        named_entity(&ent).map(|s| s.to_string())
                    };
                    if let Some(d) = decoded {
                        out.push_str(&d);
                        i = semi + 1;
                        continue;
                    }
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

// --- HTML-Scanner -----------------------------------------------------------

const BLOCK_TAGS: &[&str] = &["p", "br", "div", "li", "tr", "h1", "h2", "h3", "h4", "h5", "h6"];
const SKIP_TAGS: &[&str] = &["script", "style", "noscript"];

struct Tag {
    name: String,  // lowercase Tag-Name
    closing: bool,
}

fn parse_tag(raw: &str) -> Tag {
    let closing = raw.starts_with('/');
    let body = raw.trim_start_matches('/');
    let name: String = body.chars().take_while(|c| c.is_ascii_alphanumeric()).collect::<String>().to_lowercase();
    Tag { name, closing }
}

/// HTML_TEXT: Tags raus, Entities dekodiert, script/style geskippt,
/// Block-Tags -> Newline, Whitespace zusammengezogen.
pub fn html_text(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let mut skip = 0i32;
    while i < chars.len() {
        if chars[i] == '<' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '>' {
                j += 1;
            }
            let raw: String = chars[i + 1..j.min(chars.len())].iter().collect();
            i = if j < chars.len() { j + 1 } else { j };
            let tag = parse_tag(&raw);
            if SKIP_TAGS.contains(&tag.name.as_str()) {
                if tag.closing {
                    if skip > 0 { skip -= 1; }
                } else if !raw.ends_with('/') {
                    skip += 1;
                }
            }
            if !tag.closing && BLOCK_TAGS.contains(&tag.name.as_str()) {
                out.push('\n');
            }
        } else {
            if skip == 0 {
                out.push(chars[i]);
            }
            i += 1;
        }
    }
    let decoded = decode_entities(&out);
    // [ \t]+ -> " ", \n{3,} -> "\n\n", trim.
    let sp = regex::Regex::new(r"[ \t]+").unwrap().replace_all(&decoded, " ");
    let nl = regex::Regex::new(r"\n{3,}").unwrap().replace_all(&sp, "\n\n");
    nl.trim().to_string()
}

/// HTML_FIND_ALL: Inner-HTML aller <tag>...</tag> (mit Tags drin, Entities roh,
/// korrektes Nesting fuer gleiche Tags).
pub fn html_find_all(s: &str, target: &str) -> Vec<String> {
    let target = target.to_lowercase();
    let chars: Vec<char> = s.chars().collect();
    let mut found: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut depth = 0i32;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '>' {
                j += 1;
            }
            let raw: String = chars[i + 1..j.min(chars.len())].iter().collect();
            i = if j < chars.len() { j + 1 } else { j };
            let tag = parse_tag(&raw);
            let self_closing = raw.ends_with('/');
            if tag.name == target && !tag.closing && !self_closing {
                if depth > 0 {
                    buf.push('<');
                    buf.push_str(&raw);
                    buf.push('>');
                }
                depth += 1;
            } else if tag.name == target && tag.closing {
                if depth > 1 {
                    buf.push_str(&format!("</{}>", target));
                }
                depth -= 1;
                if depth == 0 {
                    found.push(std::mem::take(&mut buf));
                }
            } else if depth > 0 {
                buf.push('<');
                buf.push_str(&raw);
                buf.push('>');
            }
        } else {
            if depth > 0 {
                buf.push(chars[i]);
            }
            i += 1;
        }
    }
    found
}

/// HTML_GET_ATTR: Attribut-Wert aus dem ersten passenden Tag (" / ' / bare).
pub fn html_get_attr(s: &str, attr: &str) -> String {
    if attr.is_empty() {
        return String::new();
    }
    let esc = regex::escape(attr);
    for pat in [
        format!(r#"(?i)\s{}\s*=\s*"([^"]*)""#, esc),
        format!(r#"(?i)\s{}\s*=\s*'([^']*)'"#, esc),
        format!(r#"(?i)\s{}\s*=\s*([^\s>]+)"#, esc),
    ] {
        if let Ok(re) = regex::Regex::new(&pat) {
            if let Some(c) = re.captures(s) {
                if let Some(m) = c.get(1) {
                    return decode_entities(m.as_str());
                }
            }
        }
    }
    String::new()
}
