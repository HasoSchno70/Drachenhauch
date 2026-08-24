//! Modul `httpd` -- ein kleiner Webserver (Punkt 7 des Allzweck-Audits).
//!
//! Die erste Allzweck-Roadmap hatte ihn gestrichen: *„wer wirklich einen
//! Dienst braucht, stellt einen fertigen Server davor"*. Vor dem
//! Bastler-Leitbild sieht das anders aus: mit `mqtt`, `firmata`, `serial` und
//! `net` an Bord fehlte fuer „meine Heizungssteuerung hat eine kleine
//! Weboberflaeche" genau ein Baustein -- und es ist der kleinste von allen,
//! weil `NET_TCP_LISTEN` schon darunter liegt.
//!
//! **Er ist bewusst klein.** HTTP/1.1 ohne Keep-Alive, ohne TLS, ohne
//! Nebenlaeufigkeit: eine Anfrage je `HTTPD_ACCEPT`, im selben Takt wie
//! `INPUT_UPDATE` und `TIMER_UPDATE`. Das reicht fuer eine Bedienoberflaeche
//! im Heimnetz und fuer eine Handvoll Messwerte. Wer damit ins offene Netz
//! geht, stellt einen richtigen Server davor -- so herum stimmt der Satz aus
//! der alten Roadmap.
//!
//! Was er NICHT ist: kein HTTPS (dafuer einen Reverse-Proxy davorstellen),
//! kein gleichzeitiges Bedienen mehrerer Anfragen, kein Hochladen grosser
//! Dateien (der Rumpf wird ganz in den Speicher gelesen, mit Obergrenze).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// Groesste Anfrage, die angenommen wird (Kopf + Rumpf).
///
/// Ohne Obergrenze koennte ein einziger Absender den Speicher fuellen -- und
/// zwar ohne boese Absicht, es genuegt ein hochgeladenes Video. 8 MiB ist
/// weit jenseits dessen, was eine Bedienoberflaeche braucht, und immer noch
/// eine klare Ansage.
const MAX_ANFRAGE: usize = 8 << 20;

/// Wie lange auf den Rest einer angefangenen Anfrage gewartet wird.
///
/// Der Server laeuft im Takt der Hauptschleife; jede Millisekunde hier ist
/// eine, die das Programm nicht zeichnet. Ein Browser schickt seine Anfrage
/// in einem Stueck, 50 ms sind also reichlich -- wer langsamer ist, wird
/// abgewiesen statt das Bild anzuhalten.
const LESE_FRIST_MS: u64 = 50;

pub struct Anfrage {
    pub methode: String,
    pub pfad: String,
    pub abfrage: Vec<(String, String)>,
    pub kopfzeilen: Vec<(String, String)>,
    pub rumpf: Vec<u8>,
}

pub struct Server {
    lauscher: TcpListener,
    pub port: i64,
    /// Die gerade angenommene Anfrage samt offener Verbindung. Beides gehoert
    /// zusammen: die Antwort geht genau an diesen Absender.
    pub offen: Option<(TcpStream, Anfrage)>,
}

pub fn starten(port: i64, bind: &str) -> Result<Server, String> {
    let addr = if bind.is_empty() { "0.0.0.0" } else { bind };
    let l = TcpListener::bind(format!("{}:{}", addr, port))
        .map_err(|e| format!("HTTPD_START: Port {} nicht belegbar: {}", port, e))?;
    let echter = l.local_addr().map(|a| a.port() as i64).unwrap_or(port);
    // Nicht blockierend: `HTTPD_ACCEPT` soll sofort zurueckkommen, wenn
    // niemand anklopft -- sonst stuende die Hauptschleife still.
    l.set_nonblocking(true)
        .map_err(|e| format!("HTTPD_START: {}", e))?;
    Ok(Server { lauscher: l, port: echter, offen: None })
}

/// Eine Anfrage annehmen, wenn eine anliegt. `false` = gerade nichts zu tun.
pub fn annehmen(s: &mut Server) -> Result<bool, String> {
    // Eine noch unbeantwortete Anfrage wird nicht verdraengt: sonst haette
    // ein Programm, das zwei Mal hintereinander ACCEPT ruft, seinen
    // Absender verloren.
    if s.offen.is_some() { return Ok(true); }
    let (strom, _) = match s.lauscher.accept() {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
        Err(e) => return Err(format!("HTTPD_ACCEPT: {}", e)),
    };
    strom.set_nonblocking(false).ok();
    strom.set_read_timeout(Some(std::time::Duration::from_millis(LESE_FRIST_MS))).ok();
    match lesen(&strom) {
        Ok(a) => { s.offen = Some((strom, a)); Ok(true) }
        Err(_) => {
            // Kaputte oder zu langsame Anfrage: Verbindung fallen lassen und
            // so tun, als haette niemand angeklopft. Ein Fehler waere hier
            // falsch -- das Programm kann nichts dafuer, und die Schleife
            // soll weiterlaufen.
            Ok(false)
        }
    }
}

fn lesen(mut strom: &TcpStream) -> Result<Anfrage, String> {
    let mut roh: Vec<u8> = Vec::new();
    let mut puffer = [0u8; 4096];
    // Erst den Kopf bis zur Leerzeile.
    let kopf_ende = loop {
        if let Some(p) = finde_kopfende(&roh) { break p; }
        if roh.len() > MAX_ANFRAGE { return Err("Anfrage zu gross".into()); }
        match strom.read(&mut puffer) {
            Ok(0) => return Err("Verbindung zu".into()),
            Ok(n) => roh.extend_from_slice(&puffer[..n]),
            Err(e) => return Err(e.to_string()),
        }
    };
    let kopf = String::from_utf8_lossy(&roh[..kopf_ende]).into_owned();
    let mut zeilen = kopf.lines();
    let erste = zeilen.next().ok_or("leere Anfrage")?;
    let mut teile = erste.split_whitespace();
    let methode = teile.next().unwrap_or("").to_uppercase();
    let ziel = teile.next().unwrap_or("/").to_string();
    let (pfad, abfrage) = pfad_und_abfrage(&ziel);

    let mut kopfzeilen: Vec<(String, String)> = Vec::new();
    for z in zeilen {
        if let Some((k, v)) = z.split_once(':') {
            kopfzeilen.push((k.trim().to_lowercase(), v.trim().to_string()));
        }
    }
    // Dann der Rumpf, soweit angekuendigt.
    let laenge: usize = kopfzeilen.iter()
        .find(|(k, _)| k == "content-length")
        .and_then(|(_, v)| v.trim().parse().ok())
        .unwrap_or(0);
    if laenge > MAX_ANFRAGE { return Err("Rumpf zu gross".into()); }
    let mut rumpf: Vec<u8> = roh[kopf_ende + 4..].to_vec();
    while rumpf.len() < laenge {
        match strom.read(&mut puffer) {
            Ok(0) => break,
            Ok(n) => rumpf.extend_from_slice(&puffer[..n]),
            Err(e) => return Err(e.to_string()),
        }
    }
    rumpf.truncate(laenge);
    Ok(Anfrage { methode, pfad, abfrage, kopfzeilen, rumpf })
}

fn finde_kopfende(roh: &[u8]) -> Option<usize> {
    roh.windows(4).position(|f| f == b"\r\n\r\n")
}

/// `/pfad?a=1&b=zwei` -> (`/pfad`, [(a,1),(b,zwei)]) -- mit Prozent-Auflösung.
fn pfad_und_abfrage(ziel: &str) -> (String, Vec<(String, String)>) {
    let (p, q) = match ziel.split_once('?') {
        Some((p, q)) => (p, q),
        None => (ziel, ""),
    };
    let mut raus: Vec<(String, String)> = Vec::new();
    for teil in q.split('&').filter(|t| !t.is_empty()) {
        let (k, v) = teil.split_once('=').unwrap_or((teil, ""));
        raus.push((prozent_auf(k), prozent_auf(v)));
    }
    (prozent_auf(p), raus)
}

/// `%20` und `+` zurueckuebersetzen.
pub fn prozent_auf(s: &str) -> String {
    let b = s.as_bytes();
    let mut raus: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                match u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    Ok(v) => { raus.push(v); i += 3; }
                    Err(_) => { raus.push(b[i]); i += 1; }
                }
            }
            b'+' => { raus.push(b' '); i += 1; }
            c => { raus.push(c); i += 1; }
        }
    }
    String::from_utf8_lossy(&raus).into_owned()
}

/// Antworten und die Verbindung schliessen.
///
/// Kein Keep-Alive: eine Verbindung je Anfrage. Das kostet bei einer
/// Bedienoberflaeche nichts und erspart die halbe Zustandsverwaltung eines
/// echten Servers.
pub fn antworten(s: &mut Server, code: i64, typ: &str, inhalt: &[u8]) -> Result<(), String> {
    let (mut strom, _) = s.offen.take()
        .ok_or("HTTPD_SEND: es liegt keine Anfrage an (erst HTTPD_ACCEPT)")?;
    let kopf = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        code, grund(code), typ, inhalt.len());
    strom.write_all(kopf.as_bytes()).map_err(|e| format!("HTTPD_SEND: {}", e))?;
    strom.write_all(inhalt).map_err(|e| format!("HTTPD_SEND: {}", e))?;
    strom.flush().ok();
    Ok(())
}

fn grund(code: i64) -> &'static str {
    match code {
        200 => "OK", 201 => "Created", 204 => "No Content",
        301 => "Moved Permanently", 302 => "Found", 304 => "Not Modified",
        400 => "Bad Request", 401 => "Unauthorized", 403 => "Forbidden",
        404 => "Not Found", 405 => "Method Not Allowed", 413 => "Payload Too Large",
        500 => "Internal Server Error", 503 => "Service Unavailable",
        _ => "OK",
    }
}

/// Vom Dateinamen auf den Inhaltstyp schliessen.
pub fn typ_aus_endung(pfad: &str) -> &'static str {
    let e = pfad.rsplit('.').next().unwrap_or("").to_lowercase();
    match e.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" | "csv" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "pdf" => "application/pdf",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

/// Einen angefragten Pfad SICHER in einem Ordner aufloesen.
///
/// Das ist der eigentliche Grund, warum es `HTTPD_SEND_DIR` gibt und nicht
/// nur `HTTPD_SEND_FILE`: die naheliegende Zeile
/// `HTTPD_SEND_FILE(s, 200, "web" + HTTPD_PATH$(s))` laesst sich mit
/// `/../../../etc/passwd` aus dem Ordner herausfuehren. Dieselbe Lehre wie
/// bei der Zip-Slip-Pruefung in `ZIP_EXTRACT`.
///
/// `None` = der Pfad fuehrt hinaus und wird nicht bedient.
pub fn sicher_verbinden(ordner: &std::path::Path, pfad: &str) -> Option<std::path::PathBuf> {
    let mut ziel = ordner.to_path_buf();
    for teil in pfad.split('/') {
        if teil.is_empty() || teil == "." { continue; }
        // `..` wird nicht verrechnet, sondern abgelehnt: ein aufsteigender
        // Pfad hat in einer Anfrage nichts zu suchen, und "erst verrechnen,
        // dann pruefen" ist genau die Stelle, an der solche Pruefungen
        // ueblicherweise Loecher haben.
        if teil == ".." { return None; }
        // Ein Doppelpunkt waere unter Windows ein Laufwerk oder ein
        // alternativer Datenstrom (`datei.txt:versteckt`).
        if teil.contains(':') || teil.contains('\\') { return None; }
        ziel.push(teil);
    }
    Some(ziel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abfrage_wird_zerlegt() {
        let (p, q) = pfad_und_abfrage("/mess?raum=Wohn%20zimmer&grad=21.5");
        assert_eq!(p, "/mess");
        assert_eq!(q, vec![("raum".to_string(), "Wohn zimmer".to_string()),
                           ("grad".to_string(), "21.5".to_string())]);
    }

    #[test]
    fn ohne_abfrage() {
        let (p, q) = pfad_und_abfrage("/");
        assert_eq!(p, "/");
        assert!(q.is_empty());
    }

    #[test]
    fn plus_ist_ein_leerzeichen() {
        assert_eq!(prozent_auf("a+b%2Bc"), "a b+c");
    }

    #[test]
    fn aufsteigende_pfade_werden_abgelehnt() {
        let w = std::path::Path::new("/web");
        assert!(sicher_verbinden(w, "/../etc/passwd").is_none());
        assert!(sicher_verbinden(w, "/unter/../../raus").is_none());
        assert!(sicher_verbinden(w, "/c:/windows").is_none());
        assert!(sicher_verbinden(w, "/a\\b").is_none());
        assert_eq!(sicher_verbinden(w, "/seite.html"), Some(w.join("seite.html")));
        assert_eq!(sicher_verbinden(w, "/unter/seite.html"),
                   Some(w.join("unter").join("seite.html")));
    }

    #[test]
    fn typen_aus_der_endung() {
        assert_eq!(typ_aus_endung("a.html"), "text/html; charset=utf-8");
        assert_eq!(typ_aus_endung("A.PNG"), "image/png");
        assert_eq!(typ_aus_endung("ohne"), "application/octet-stream");
    }

    #[test]
    fn kopfende_wird_gefunden() {
        assert_eq!(finde_kopfende(b"GET / HTTP/1.1\r\n\r\n"), Some(14));
        assert_eq!(finde_kopfende(b"GET / HTTP/1.1\r\n"), None);
    }
}
