//! WebSocket (WS_*, RFC 6455): Client (`ws://`, `wss://`) und Server.
//!
//! Eine Verbindung ist ein TCP-Strom (bei `wss://` mit rustls darum), der
//! nach dem Handschlag ohne Warten gelesen wird: jeder WS_NEXT/WS_CONNECTED
//! holt ab, was da ist, setzt Rahmen zusammen (auch Fragmente), beantwortet
//! Ping mit Pong und ein Schliessen mit einem Schliessen. Nachrichten stehen
//! in einer Schlange und werden ueber das Cursor-Muster gelesen (wie MQTT
//! und DB): WS_NEXT, dann WS_MESSAGE$ bzw. WS_MESSAGE_BYTES.
//!
//! Server: WS_LISTEN + WS_ACCEPT (ohne Warten); der Handschlag einer
//! angenommenen Verbindung wartet hoechstens 3 s. Eine Anfrage, die kein
//! WebSocket will (ein Browser fragt nach /favicon.ico), bekommt 400 und
//! zaehlt nicht als Verbindung.

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
/// Groesste Nachricht (auch zusammengesetzt aus Fragmenten).
pub const MAX_NACHRICHT: usize = 16 * 1024 * 1024;

enum Strom {
    Klar(TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
}

impl Strom {
    fn tcp(&self) -> &TcpStream {
        match self { Strom::Klar(s) => s, Strom::Tls(s) => s.get_ref() }
    }
}

impl Read for Strom {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        match self { Strom::Klar(s) => s.read(b), Strom::Tls(s) => s.read(b) }
    }
}

impl Write for Strom {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        match self { Strom::Klar(s) => s.write(b), Strom::Tls(s) => s.write(b) }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self { Strom::Klar(s) => s.flush(), Strom::Tls(s) => s.flush() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Nachricht { pub binaer: bool, pub daten: Vec<u8> }

#[derive(Debug, PartialEq)]
pub struct Rahmen { pub fin: bool, pub op: u8, pub maskiert: bool, pub daten: Vec<u8> }

// --- Rahmen (rein) ----------------------------------------------------------

/// Einen Rahmen bauen; `maske` nur auf der Client-Seite (Pflicht dort, auf
/// der Server-Seite verboten).
pub fn rahmen_bauen(op: u8, daten: &[u8], maske: Option<[u8; 4]>) -> Vec<u8> {
    let mut out = Vec::with_capacity(daten.len() + 14);
    out.push(0x80 | (op & 0x0F));
    let m = if maske.is_some() { 0x80 } else { 0 };
    let n = daten.len();
    if n < 126 { out.push(m | n as u8); }
    else if n <= 0xFFFF { out.push(m | 126); out.extend_from_slice(&(n as u16).to_be_bytes()); }
    else { out.push(m | 127); out.extend_from_slice(&(n as u64).to_be_bytes()); }
    match maske {
        Some(k) => {
            out.extend_from_slice(&k);
            out.extend(daten.iter().enumerate().map(|(i, b)| b ^ k[i % 4]));
        }
        None => out.extend_from_slice(daten),
    }
    out
}

/// Den ersten Rahmen aus `buf` lesen: Ok(None) = noch nicht vollstaendig,
/// sonst (Rahmen, verbrauchte Bytes). Maskierte Daten kommen entmaskiert.
pub fn rahmen_lesen(buf: &[u8]) -> Result<Option<(Rahmen, usize)>, String> {
    if buf.len() < 2 { return Ok(None); }
    let fin = buf[0] & 0x80 != 0;
    if buf[0] & 0x70 != 0 { return Err("Rahmen mit gesetzten RSV-Bits (Erweiterung, die nicht ausgehandelt wurde)".into()); }
    let op = buf[0] & 0x0F;
    let maskiert = buf[1] & 0x80 != 0;
    let mut n = (buf[1] & 0x7F) as u64;
    let mut p = 2;
    if n == 126 {
        if buf.len() < 4 { return Ok(None); }
        n = u16::from_be_bytes([buf[2], buf[3]]) as u64;
        p = 4;
    } else if n == 127 {
        if buf.len() < 10 { return Ok(None); }
        n = u64::from_be_bytes(buf[2..10].try_into().unwrap());
        p = 10;
    }
    if n > MAX_NACHRICHT as u64 { return Err(format!("Rahmen mit {} Bytes -- hoechstens {}", n, MAX_NACHRICHT)); }
    if op >= 8 && (n > 125 || !fin) { return Err("Steuerrahmen zu lang oder zerteilt".into()); }
    let maske = if maskiert {
        if buf.len() < p + 4 { return Ok(None); }
        let k = [buf[p], buf[p + 1], buf[p + 2], buf[p + 3]];
        p += 4;
        Some(k)
    } else { None };
    let n = n as usize;
    if buf.len() < p + n { return Ok(None); }
    let mut daten = buf[p..p + n].to_vec();
    if let Some(k) = maske { for (i, b) in daten.iter_mut().enumerate() { *b ^= k[i % 4]; } }
    Ok(Some((Rahmen { fin, op, maskiert, daten }, p + n)))
}

/// Die Antwort des Servers auf einen Schluessel (Sec-WebSocket-Accept).
pub fn annahme_schluessel(schluessel: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut h = Sha1::new();
    h.update(schluessel.trim().as_bytes());
    h.update(GUID.as_bytes());
    crate::builtins::b64_encode(&h.finalize())
}

/// Eine Adresse zerlegen: (tls, host, port, pfad).
pub fn adresse_lesen(url: &str) -> Result<(bool, String, u16, String), String> {
    let u = url.trim();
    let klein = u.to_ascii_lowercase();
    let (tls, rest) = if klein.starts_with("wss://") { (true, &u[6..]) }
        else if klein.starts_with("ws://") { (false, &u[5..]) }
        else { return Err(format!("WS_CONNECT: {:?} ist keine WebSocket-Adresse (ws://... oder wss://...)", url)); };
    let (wirt, pfad) = match rest.find('/') { Some(i) => (&rest[..i], rest[i..].to_string()), None => (rest, "/".to_string()) };
    let (host, port) = if let Some(r) = wirt.strip_prefix('[') {
        let ende = r.find(']').ok_or_else(|| format!("WS_CONNECT: {:?} -- ']' fehlt", url))?;
        let host = r[..ende].to_string();
        let port = r[ende + 1..].strip_prefix(':').map(|p| p.parse::<u16>());
        (host, port)
    } else {
        match wirt.rfind(':') {
            Some(i) => (wirt[..i].to_string(), Some(wirt[i + 1..].parse::<u16>())),
            None => (wirt.to_string(), None),
        }
    };
    let port = match port {
        None => if tls { 443 } else { 80 },
        Some(Ok(p)) => p,
        Some(Err(_)) => return Err(format!("WS_CONNECT: {:?} -- der Port ist keine Zahl 0..65535", url)),
    };
    if host.is_empty() { return Err(format!("WS_CONNECT: {:?} -- der Rechner fehlt", url)); }
    Ok((tls, host, port, pfad))
}

/// Kopfzeilen einer HTTP-Nachricht (ohne die erste Zeile), Namen klein.
fn kopf_lesen(text: &str) -> Vec<(String, String)> {
    text.split("\r\n").skip(1).filter_map(|z| {
        let i = z.find(':')?;
        Some((z[..i].trim().to_ascii_lowercase(), z[i + 1..].trim().to_string()))
    }).collect()
}

fn kopf<'a>(k: &'a [(String, String)], name: &str) -> Option<&'a str> {
    k.iter().find(|(n, _)| n == name).map(|(_, w)| w.as_str())
}

fn zufall<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    if getrandom::getrandom(&mut b).is_err() {
        let t = Instant::now().elapsed().as_nanos() as u64 ^ std::process::id() as u64;
        for (i, x) in b.iter_mut().enumerate() { *x = (t >> ((i % 8) * 8)) as u8 ^ i as u8; }
    }
    b
}

// --- Verbindung -------------------------------------------------------------

pub struct Verbindung {
    strom: Strom,
    /// Client-Seite: jeder Rahmen wird maskiert.
    client: bool,
    puffer: Vec<u8>,
    teil: Option<(bool, Vec<u8>)>,
    eingang: VecDeque<Nachricht>,
    aktuell: Option<Nachricht>,
    offen: bool,
    geschlossen_gesendet: bool,
    pub pfad: String,
    pub schliess_code: i64,
    pub schliess_grund: String,
    pub gegenstelle: String,
}

/// Bis zum Ende der Kopfzeilen lesen (hoechstens 16 KB); `rest` sind Bytes
/// dahinter, die schon zum WebSocket gehoeren.
fn http_kopf(strom: &mut Strom, frist: Instant) -> Result<(String, Vec<u8>), String> {
    let mut buf = Vec::new();
    let mut stueck = [0u8; 2048];
    loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let kopf = String::from_utf8_lossy(&buf[..i]).into_owned();
            return Ok((kopf, buf[i + 4..].to_vec()));
        }
        if buf.len() > 16384 { return Err("Kopfzeilen laenger als 16 KB".into()); }
        if Instant::now() > frist { return Err("Zeitueberschreitung beim Handschlag".into()); }
        match strom.read(&mut stueck) {
            Ok(0) => return Err("die Gegenstelle hat die Verbindung geschlossen".into()),
            Ok(n) => buf.extend_from_slice(&stueck[..n]),
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

fn tls_strom(tcp: TcpStream, name: &str) -> Result<Strom, String> {
    let mut wurzeln = rustls::RootCertStore::empty();
    wurzeln.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let anbieter = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    let konfig = rustls::ClientConfig::builder_with_provider(anbieter)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("WS_CONNECT: TLS liess sich nicht einrichten: {}", e))?
        .with_root_certificates(wurzeln)
        .with_no_client_auth();
    let sn = rustls::pki_types::ServerName::try_from(name.to_string())
        .map_err(|_| format!("WS_CONNECT: {:?} ist kein gueltiger Servername fuer TLS", name))?;
    let v = rustls::ClientConnection::new(std::sync::Arc::new(konfig), sn)
        .map_err(|e| format!("WS_CONNECT: TLS-Verbindung fehlgeschlagen: {}", e))?;
    Ok(Strom::Tls(Box::new(rustls::StreamOwned::new(v, tcp))))
}

/// Mit einem WebSocket-Server verbinden (wartet hoechstens 5 s).
pub fn verbinden(url: &str) -> Result<Verbindung, String> {
    let (tls, host, port, pfad) = adresse_lesen(url)?;
    let ziel = format!("{}:{}", host, port);
    let adr = match (host.as_str(), port).to_socket_addrs().ok().and_then(|mut a| a.next()) {
        Some(a) => a,
        // Ein Name auf .local, den das System nicht kennt (Linux ohne
        // nss-mdns): selbst ueber mDNS fragen.
        None if host.to_ascii_lowercase().ends_with(".local") => {
            let ip = crate::mdns::aufloesen(&host, 1500).unwrap_or_default();
            format!("{}:{}", ip, port).parse::<std::net::SocketAddr>()
                .map_err(|_| format!("WS_CONNECT: {} nicht aufloesbar (auch nicht ueber mDNS)", ziel))?
        }
        None => return Err(format!("WS_CONNECT: {} nicht aufloesbar", ziel)),
    };
    let tcp = TcpStream::connect_timeout(&adr, Duration::from_secs(5))
        .map_err(|e| format!("WS_CONNECT: {} -- {}", ziel, e))?;
    tcp.set_nodelay(true).ok();
    tcp.set_read_timeout(Some(Duration::from_millis(50))).ok();
    let mut strom = if tls { tls_strom(tcp, &host)? } else { Strom::Klar(tcp) };
    let schluessel = crate::builtins::b64_encode(&zufall::<16>());
    let wirt = if (tls && port == 443) || (!tls && port == 80) { host.clone() } else { ziel.clone() };
    let anfrage = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {}\r\nSec-WebSocket-Version: 13\r\nUser-Agent: Drachenhauch\r\n\r\n",
        pfad, wirt, schluessel);
    strom.write_all(anfrage.as_bytes()).and_then(|_| strom.flush()).map_err(|e| {
        if tls { format!("WS_CONNECT: TLS-Verbindung fehlgeschlagen: {}", e) } else { format!("WS_CONNECT: {} -- {}", ziel, e) }
    })?;
    let (antwort, rest) = http_kopf(&mut strom, Instant::now() + Duration::from_secs(5))
        .map_err(|e| format!("WS_CONNECT: {} -- {}", ziel, e))?;
    let erste = antwort.lines().next().unwrap_or("");
    if !erste.split_whitespace().nth(1).map_or(false, |c| c == "101") {
        return Err(format!("WS_CONNECT: {} will kein WebSocket -- Antwort {:?}", url, erste));
    }
    let k = kopf_lesen(&antwort);
    if kopf(&k, "sec-websocket-accept") != Some(annahme_schluessel(&schluessel).as_str()) {
        return Err(format!("WS_CONNECT: {} antwortet mit einem falschen Sec-WebSocket-Accept", url));
    }
    nicht_wartend(&strom);
    Ok(Verbindung {
        strom, client: true, puffer: rest, teil: None, eingang: VecDeque::new(), aktuell: None,
        offen: true, geschlossen_gesendet: false, pfad, schliess_code: 0, schliess_grund: String::new(),
        gegenstelle: ziel,
    })
}

fn nicht_wartend(strom: &Strom) {
    strom.tcp().set_read_timeout(None).ok();
    strom.tcp().set_nonblocking(true).ok();
}

impl Verbindung {
    /// Schreiben, auch wenn der Strom nicht wartet (hoechstens 5 s).
    fn schreiben(&mut self, daten: &[u8]) -> Result<(), String> {
        let frist = Instant::now() + Duration::from_secs(5);
        let mut p = 0;
        while p < daten.len() {
            match self.strom.write(&daten[p..]) {
                Ok(0) => return Err("die Verbindung ist zu".into()),
                Ok(n) => p += n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() > frist { return Err("Zeitueberschreitung beim Senden".into()); }
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        loop {
            match self.strom.flush() {
                Ok(()) => return Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() > frist { return Err("Zeitueberschreitung beim Senden".into()); }
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    fn rahmen_senden(&mut self, op: u8, daten: &[u8]) -> Result<(), String> {
        let maske = if self.client { Some(zufall::<4>()) } else { None };
        let r = rahmen_bauen(op, daten, maske);
        self.schreiben(&r)
    }

    /// Alles abholen, was da ist, und die Rahmen verarbeiten.
    pub fn pumpen(&mut self) {
        if !self.offen { return; }
        let mut stueck = [0u8; 8192];
        loop {
            match self.strom.read(&mut stueck) {
                Ok(0) => { self.offen = false; if self.schliess_code == 0 { self.schliess_code = 1006; } break; }
                Ok(n) => self.puffer.extend_from_slice(&stueck[..n]),
                Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => break,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => { self.offen = false; if self.schliess_code == 0 { self.schliess_code = 1006; } break; }
            }
        }
        loop {
            let (r, n) = match rahmen_lesen(&self.puffer) {
                Ok(Some(x)) => x,
                Ok(None) => break,
                Err(e) => { self.abbrechen(1002, &e); return; }
            };
            self.puffer.drain(..n);
            // RFC 6455 5.1: ein Client maskiert jeden Rahmen, ein Server keinen.
            if r.maskiert == self.client {
                self.abbrechen(1002, if self.client { "maskierter Rahmen vom Server" } else { "unmaskierter Rahmen vom Client" });
                return;
            }
            match r.op {
                0 => {
                    let Some((binaer, mut gesammelt)) = self.teil.take() else { self.abbrechen(1002, "Fortsetzung ohne Anfang"); return; };
                    if gesammelt.len() + r.daten.len() > MAX_NACHRICHT { self.abbrechen(1009, "Nachricht zu gross"); return; }
                    gesammelt.extend_from_slice(&r.daten);
                    if r.fin { self.eingang.push_back(Nachricht { binaer, daten: gesammelt }); }
                    else { self.teil = Some((binaer, gesammelt)); }
                }
                1 | 2 => {
                    if self.teil.is_some() { self.abbrechen(1002, "neue Nachricht mitten in einer zerteilten"); return; }
                    let binaer = r.op == 2;
                    if r.fin { self.eingang.push_back(Nachricht { binaer, daten: r.daten }); }
                    else { self.teil = Some((binaer, r.daten)); }
                }
                8 => {
                    if r.daten.len() >= 2 {
                        self.schliess_code = u16::from_be_bytes([r.daten[0], r.daten[1]]) as i64;
                        self.schliess_grund = String::from_utf8_lossy(&r.daten[2..]).into_owned();
                    } else if self.schliess_code == 0 { self.schliess_code = 1005; }
                    if !self.geschlossen_gesendet {
                        let antwort = if r.daten.len() >= 2 { r.daten[..2].to_vec() } else { Vec::new() };
                        let _ = self.rahmen_senden(8, &antwort);
                        self.geschlossen_gesendet = true;
                    }
                    self.offen = false;
                    break;
                }
                9 => { let d = r.daten.clone(); let _ = self.rahmen_senden(10, &d); }
                10 => {}
                anders => { self.abbrechen(1002, &format!("unbekannter Rahmentyp {}", anders)); return; }
            }
        }
    }

    fn abbrechen(&mut self, code: u16, grund: &str) {
        let mut d = code.to_be_bytes().to_vec();
        d.extend_from_slice(grund.as_bytes().get(..120).unwrap_or(grund.as_bytes()));
        let _ = self.rahmen_senden(8, &d);
        self.geschlossen_gesendet = true;
        self.offen = false;
        self.schliess_code = code as i64;
        self.schliess_grund = grund.to_string();
    }

    pub fn offen(&mut self) -> bool { self.pumpen(); self.offen }

    pub fn senden(&mut self, daten: &[u8], binaer: bool) -> Result<bool, String> {
        self.pumpen();
        if !self.offen { return Ok(false); }
        match self.rahmen_senden(if binaer { 2 } else { 1 }, daten) {
            Ok(()) => Ok(true),
            Err(_) => { self.offen = false; if self.schliess_code == 0 { self.schliess_code = 1006; } Ok(false) }
        }
    }

    pub fn ping(&mut self) -> bool {
        self.pumpen();
        self.offen && self.rahmen_senden(9, b"dh").is_ok()
    }

    /// Die naechste Nachricht in den Cursor holen.
    pub fn naechste(&mut self) -> bool {
        self.pumpen();
        self.aktuell = self.eingang.pop_front();
        self.aktuell.is_some()
    }

    pub fn aktuell(&self) -> Option<&Nachricht> { self.aktuell.as_ref() }

    pub fn wartend(&mut self) -> usize { self.pumpen(); self.eingang.len() }

    /// Sauber schliessen: Schliess-Rahmen senden und kurz (hoechstens 1 s)
    /// auf die Antwort warten.
    pub fn schliessen(&mut self, code: u16, grund: &str) {
        if self.offen && !self.geschlossen_gesendet {
            let mut d = code.to_be_bytes().to_vec();
            d.extend_from_slice(grund.as_bytes().get(..120).unwrap_or(grund.as_bytes()));
            let _ = self.rahmen_senden(8, &d);
            self.geschlossen_gesendet = true;
            let frist = Instant::now() + Duration::from_secs(1);
            while self.offen && Instant::now() < frist {
                self.pumpen();
                std::thread::sleep(Duration::from_millis(5));
            }
            if self.schliess_code == 0 { self.schliess_code = code as i64; self.schliess_grund = grund.to_string(); }
        }
        self.offen = false;
        let _ = self.strom.tcp().shutdown(std::net::Shutdown::Both);
    }
}

impl Drop for Verbindung {
    fn drop(&mut self) {
        if self.offen { self.schliessen(1001, "weg"); }
    }
}

// --- Server -----------------------------------------------------------------

pub struct Server { lauscher: TcpListener, pub port: u16 }

pub fn lauschen(port: i64, bind: &str) -> Result<Server, String> {
    if !(0..=65535).contains(&port) { return Err(format!("WS_LISTEN: Port {} ausserhalb 0..65535", port)); }
    let l = TcpListener::bind((bind, port as u16)).map_err(|e| format!("WS_LISTEN: {}:{} -- {}", bind, port, e))?;
    l.set_nonblocking(true).map_err(|e| format!("WS_LISTEN: {}", e))?;
    let port = l.local_addr().map(|a| a.port()).unwrap_or(port as u16);
    Ok(Server { lauscher: l, port })
}

impl Server {
    /// Eine wartende Verbindung annehmen und den Handschlag fuehren; None,
    /// wenn keine wartet oder die Anfrage kein WebSocket wollte.
    pub fn annehmen(&mut self) -> Option<Verbindung> {
        loop {
            let (tcp, adr) = match self.lauscher.accept() { Ok(x) => x, Err(_) => return None };
            tcp.set_nonblocking(false).ok();
            tcp.set_nodelay(true).ok();
            tcp.set_read_timeout(Some(Duration::from_millis(50))).ok();
            let mut strom = Strom::Klar(tcp);
            let Ok((anfrage, rest)) = http_kopf(&mut strom, Instant::now() + Duration::from_secs(3)) else { continue };
            let erste = anfrage.lines().next().unwrap_or("").to_string();
            let mut teile = erste.split_whitespace();
            let methode = teile.next().unwrap_or("");
            let pfad = teile.next().unwrap_or("/").to_string();
            let k = kopf_lesen(&anfrage);
            let will = kopf(&k, "upgrade").map_or(false, |u| u.eq_ignore_ascii_case("websocket"));
            let schluessel = kopf(&k, "sec-websocket-key").map(|s| s.to_string());
            match (methode == "GET" && will, schluessel) {
                (true, Some(s)) => {
                    let antwort = format!(
                        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
                        annahme_schluessel(&s));
                    if strom.write_all(antwort.as_bytes()).and_then(|_| strom.flush()).is_err() { continue; }
                    nicht_wartend(&strom);
                    return Some(Verbindung {
                        strom, client: false, puffer: rest, teil: None, eingang: VecDeque::new(), aktuell: None,
                        offen: true, geschlossen_gesendet: false, pfad, schliess_code: 0,
                        schliess_grund: String::new(), gegenstelle: adr.to_string(),
                    });
                }
                _ => {
                    let text = "Hier spricht ein WebSocket-Server (Drachenhauch).\n";
                    let antwort = format!("HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", text.len(), text);
                    let _ = strom.write_all(antwort.as_bytes());
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annahme_wie_im_rfc() {
        // RFC 6455, Abschnitt 1.3
        assert_eq!(annahme_schluessel("dGhlIHNhbXBsZSBub25jZQ=="), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn rahmen_hin_und_zurueck() {
        // RFC 6455, 5.7: "Hello" unmaskiert und maskiert
        assert_eq!(rahmen_bauen(1, b"Hello", None), vec![0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f]);
        let m = rahmen_bauen(1, b"Hello", Some([0x37, 0xfa, 0x21, 0x3d]));
        assert_eq!(m, vec![0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58]);
        let (r, n) = rahmen_lesen(&m).unwrap().unwrap();
        assert_eq!((r.fin, r.op, r.maskiert, r.daten.as_slice(), n), (true, 1, true, &b"Hello"[..], 11));
        // 256 und 65536 Bytes: 16- und 64-Bit-Laenge
        for laenge in [126usize, 256, 65535, 65536] {
            let d = vec![7u8; laenge];
            let r = rahmen_bauen(2, &d, Some([1, 2, 3, 4]));
            let (x, n) = rahmen_lesen(&r).unwrap().unwrap();
            assert_eq!((x.daten.len(), n), (laenge, r.len()));
            assert!(x.daten.iter().all(|&b| b == 7));
            // unvollstaendig: ein Byte fehlt
            assert!(rahmen_lesen(&r[..r.len() - 1]).unwrap().is_none());
        }
    }

    #[test]
    fn falsche_rahmen() {
        assert!(rahmen_lesen(&[0xC1, 0x00]).is_err(), "RSV gesetzt");
        assert!(rahmen_lesen(&[0x09, 0x00]).is_err(), "Ping zerteilt");
        let mut lang = vec![0x82, 127];
        lang.extend_from_slice(&((MAX_NACHRICHT as u64) + 1).to_be_bytes());
        assert!(rahmen_lesen(&lang).is_err(), "zu gross");
    }

    #[test]
    fn adressen() {
        assert_eq!(adresse_lesen("ws://esp32.local/ws").unwrap(), (false, "esp32.local".into(), 80, "/ws".into()));
        assert_eq!(adresse_lesen("wss://echo.example.org").unwrap(), (true, "echo.example.org".into(), 443, "/".into()));
        assert_eq!(adresse_lesen("ws://127.0.0.1:8080/a?b=1").unwrap(), (false, "127.0.0.1".into(), 8080, "/a?b=1".into()));
        assert_eq!(adresse_lesen("ws://[::1]:9001/").unwrap(), (false, "::1".into(), 9001, "/".into()));
        assert!(adresse_lesen("http://x").is_err());
        assert!(adresse_lesen("ws://x:abc").is_err());
    }

    #[test]
    fn server_und_client_sprechen_miteinander() {
        let mut s = lauschen(0, "127.0.0.1").unwrap();
        let port = s.port;
        let faden = std::thread::spawn(move || {
            let mut c = verbinden(&format!("ws://127.0.0.1:{}/raum", port)).unwrap();
            assert!(c.senden(b"hallo", false).unwrap());
            assert!(c.senden(&vec![1u8; 70000], true).unwrap());
            let frist = Instant::now() + Duration::from_secs(5);
            while !c.naechste() { assert!(Instant::now() < frist); std::thread::sleep(Duration::from_millis(5)); }
            let n = c.aktuell().unwrap().clone();
            c.schliessen(1000, "fertig");
            n
        });
        let frist = Instant::now() + Duration::from_secs(5);
        let mut v = loop {
            if let Some(v) = s.annehmen() { break v; }
            assert!(Instant::now() < frist);
            std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(v.pfad, "/raum");
        let mut bekommen = Vec::new();
        while bekommen.len() < 2 {
            if v.naechste() { bekommen.push(v.aktuell().unwrap().clone()); }
            assert!(Instant::now() < frist);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(bekommen[0], Nachricht { binaer: false, daten: b"hallo".to_vec() });
        assert_eq!(bekommen[1].daten.len(), 70000);
        assert!(v.senden("zurück".as_bytes(), false).unwrap());
        let antwort = faden.join().unwrap();
        assert_eq!(String::from_utf8(antwort.daten).unwrap(), "zurück");
        let frist = Instant::now() + Duration::from_secs(3);
        while v.offen() { assert!(Instant::now() < frist); std::thread::sleep(Duration::from_millis(5)); }
        assert_eq!((v.schliess_code, v.schliess_grund.as_str()), (1000, "fertig"));
    }
}
