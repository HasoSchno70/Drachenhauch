//! mDNS (Multicast-DNS, RFC 6762, und DNS-SD, RFC 6763; MDNS_*): Namen wie
//! `esp32.local` aufloesen, Dienste im Netz finden ("_http._tcp") und eigene
//! ankuendigen -- ohne Avahi oder Bonjour.
//!
//! Gefragt wird von einem gewoehnlichen UDP-Anschluss aus ("legacy unicast",
//! RFC 6762 6.7): die Antwortenden schicken ihre Antwort dann direkt zurueck.
//! So muss niemand den Anschluss 5353 teilen, den unter Windows, macOS und
//! Linux schon das System benutzt. Nur wer selbst ankuendigt, haengt sich an
//! 5353 (SO_REUSEADDR/-PORT) und die Gruppe 224.0.0.251.
//!
//! **Pruefen ohne Netz:** `DHRT_MDNS_PORT=n` laesst Fragen und Ankuendigen
//! ueber 127.0.0.1:n laufen statt ueber die Gruppe -- zwei Programme auf
//! einem Rechner finden sich dann auch dort, wo kein Multicast geht.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const GRUPPE: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
pub const PORT: u16 = 5353;
const T_A: u16 = 1;
const T_PTR: u16 = 12;
const T_TXT: u16 = 16;
const T_SRV: u16 = 33;
const T_ANY: u16 = 255;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Dienst {
    pub name: String,
    pub host: String,
    pub adresse: String,
    pub port: u16,
    pub txt: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Daten {
    A(Ipv4Addr),
    Ptr(String),
    Srv { port: u16, ziel: String },
    Txt(Vec<String>),
    Anders,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Eintrag { pub name: String, pub typ: u16, pub daten: Daten }

// --- Pakete (rein) ----------------------------------------------------------

fn name_schreiben(b: &mut Vec<u8>, name: &str) {
    for teil in name.trim_end_matches('.').split('.') {
        let t = teil.as_bytes();
        let n = t.len().min(63);
        b.push(n as u8);
        b.extend_from_slice(&t[..n]);
    }
    b.push(0);
}

/// Eine Frage nach `name` vom Typ `typ`, mit Kennung (fuer legacy unicast).
pub fn frage(id: u16, name: &str, typ: u16) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&id.to_be_bytes());
    b.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
    name_schreiben(&mut b, name);
    b.extend_from_slice(&typ.to_be_bytes());
    b.extend_from_slice(&1u16.to_be_bytes());
    b
}

fn name_lesen(p: &[u8], mut i: usize) -> Result<(String, usize), String> {
    let mut teile = Vec::new();
    let mut ende = None;
    let mut spruenge = 0;
    loop {
        let n = *p.get(i).ok_or("Name ueber das Ende")? as usize;
        if n == 0 { if ende.is_none() { ende = Some(i + 1); } break; }
        if n & 0xC0 == 0xC0 {
            let ziel = ((n & 0x3F) << 8) | *p.get(i + 1).ok_or("Zeiger ueber das Ende")? as usize;
            if ende.is_none() { ende = Some(i + 2); }
            spruenge += 1;
            if spruenge > 32 || ziel >= p.len() { return Err("Zeigerkette im Namen".into()); }
            i = ziel;
            continue;
        }
        let t = p.get(i + 1..i + 1 + n).ok_or("Namensteil ueber das Ende")?;
        teile.push(String::from_utf8_lossy(t).into_owned());
        i += 1 + n;
    }
    Ok((teile.join("."), ende.unwrap()))
}

fn u16_bei(p: &[u8], i: usize) -> Result<u16, String> {
    Ok(u16::from_be_bytes([*p.get(i).ok_or("zu kurz")?, *p.get(i + 1).ok_or("zu kurz")?]))
}

/// Alle Eintraege (Antworten, Zustaendige, Zusaetzliche) einer Antwort; eine
/// Frage liefert Ok(None).
pub fn antwort_lesen(p: &[u8]) -> Result<Option<(u16, Vec<Eintrag>)>, String> {
    if p.len() < 12 { return Err("Paket zu kurz".into()); }
    let id = u16_bei(p, 0)?;
    let flags = u16_bei(p, 2)?;
    if flags & 0x8000 == 0 { return Ok(None); }
    let fragen = u16_bei(p, 4)? as usize;
    let rest = u16_bei(p, 6)? as usize + u16_bei(p, 8)? as usize + u16_bei(p, 10)? as usize;
    let mut i = 12;
    for _ in 0..fragen { let (_, j) = name_lesen(p, i)?; i = j + 4; }
    let mut out = Vec::new();
    for _ in 0..rest {
        let (name, j) = name_lesen(p, i)?;
        let typ = u16_bei(p, j)?;
        let laenge = u16_bei(p, j + 8)? as usize;
        let d = j + 10;
        if d + laenge > p.len() { return Err("Eintrag ueber das Ende".into()); }
        let daten = match typ {
            T_A if laenge == 4 => Daten::A(Ipv4Addr::new(p[d], p[d + 1], p[d + 2], p[d + 3])),
            T_PTR => Daten::Ptr(name_lesen(p, d)?.0),
            T_SRV if laenge >= 7 => Daten::Srv { port: u16_bei(p, d + 4)?, ziel: name_lesen(p, d + 6)?.0 },
            T_TXT => {
                let mut texte = Vec::new();
                let mut k = d;
                while k < d + laenge {
                    let n = p[k] as usize;
                    if k + 1 + n > d + laenge { break; }
                    texte.push(String::from_utf8_lossy(&p[k + 1..k + 1 + n]).into_owned());
                    k += 1 + n;
                }
                Daten::Txt(texte)
            }
            _ => Daten::Anders,
        };
        out.push(Eintrag { name, typ, daten });
        i = d + laenge;
    }
    Ok(Some((id, out)))
}

fn eintrag_schreiben(b: &mut Vec<u8>, name: &str, typ: u16, cache_flush: bool, ttl: u32, daten: &[u8]) {
    name_schreiben(b, name);
    b.extend_from_slice(&typ.to_be_bytes());
    b.extend_from_slice(&(if cache_flush { 0x8001u16 } else { 1u16 }).to_be_bytes());
    b.extend_from_slice(&ttl.to_be_bytes());
    b.extend_from_slice(&(daten.len() as u16).to_be_bytes());
    b.extend_from_slice(daten);
}

fn name_bytes(name: &str) -> Vec<u8> { let mut b = Vec::new(); name_schreiben(&mut b, name); b }

/// Eine Antwort mit den Eintraegen eines angekuendigten Dienstes.
pub fn antwort_bauen(id: u16, a: &Ankuendigung, adresse: Ipv4Addr, ttl: u32) -> Vec<u8> {
    let instanz = a.instanz();
    let mut b = Vec::new();
    b.extend_from_slice(&id.to_be_bytes());
    b.extend_from_slice(&0x8400u16.to_be_bytes());
    b.extend_from_slice(&[0, 0, 0, 4, 0, 0, 0, 0]);
    eintrag_schreiben(&mut b, &a.dienst, T_PTR, false, ttl, &name_bytes(&instanz));
    let mut srv = vec![0, 0, 0, 0];
    srv.extend_from_slice(&a.port.to_be_bytes());
    srv.extend_from_slice(&name_bytes(&a.host));
    eintrag_schreiben(&mut b, &instanz, T_SRV, true, ttl, &srv);
    let mut txt = Vec::new();
    for (k, v) in &a.txt {
        let e = if v.is_empty() { k.clone() } else { format!("{}={}", k, v) };
        let e = e.as_bytes();
        let n = e.len().min(255);
        txt.push(n as u8);
        txt.extend_from_slice(&e[..n]);
    }
    if txt.is_empty() { txt.push(0); }
    eintrag_schreiben(&mut b, &instanz, T_TXT, true, ttl, &txt);
    eintrag_schreiben(&mut b, &a.host, T_A, true, ttl, &adresse.octets());
    b
}

/// "esp32" -> "esp32.local", "_http._tcp" -> "_http._tcp.local".
pub fn lokal(name: &str) -> String {
    let n = name.trim().trim_end_matches('.');
    if n.to_ascii_lowercase().ends_with(".local") { n.to_string() } else { format!("{}.local", n) }
}

pub fn txt_lesen(texte: &[String]) -> Vec<(String, String)> {
    texte.iter().filter(|t| !t.is_empty()).map(|t| match t.find('=') {
        Some(i) => (t[..i].to_string(), t[i + 1..].to_string()),
        None => (t.clone(), String::new()),
    }).collect()
}

/// "k=v;k2=v2" -> Liste.
pub fn txt_aus_text(s: &str) -> Vec<(String, String)> {
    s.split(';').filter(|t| !t.trim().is_empty()).map(|t| match t.find('=') {
        Some(i) => (t[..i].trim().to_string(), t[i + 1..].trim().to_string()),
        None => (t.trim().to_string(), String::new()),
    }).collect()
}

/// Aus allen gesammelten Eintraegen die Dienste vom Typ `dienst` bauen.
pub fn dienste_aus(eintraege: &[Eintrag], dienst: &str) -> Vec<Dienst> {
    let gleich = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
    let mut out: Vec<Dienst> = Vec::new();
    for e in eintraege {
        if let Daten::Ptr(instanz) = &e.daten {
            if gleich(&e.name, dienst) && !out.iter().any(|d| gleich(&d.name, instanz)) {
                out.push(Dienst { name: instanz.clone(), ..Default::default() });
            }
        }
    }
    for d in out.iter_mut() {
        for e in eintraege {
            if !gleich(&e.name, &d.name) { continue; }
            match &e.daten {
                Daten::Srv { port, ziel } => { d.port = *port; d.host = ziel.clone(); }
                Daten::Txt(t) => d.txt = txt_lesen(t),
                _ => {}
            }
        }
        for e in eintraege {
            if let Daten::A(ip) = &e.daten { if gleich(&e.name, &d.host) { d.adresse = ip.to_string(); break; } }
        }
    }
    out
}

/// Der Anzeigename einer Instanz: "Kueche._http._tcp.local" -> "Kueche".
pub fn kurzname(instanz: &str, dienst: &str) -> String {
    let i = instanz.len().saturating_sub(dienst.len() + 1);
    if instanz.len() > dienst.len() && instanz[i + 1..].eq_ignore_ascii_case(dienst) { instanz[..i].to_string() } else { instanz.to_string() }
}

// --- Fragen ---------------------------------------------------------------------

fn testport() -> Option<u16> {
    std::env::var("DHRT_MDNS_PORT").ok().and_then(|s| s.trim().parse().ok())
}

fn ziel() -> SocketAddr {
    match testport() {
        Some(p) => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, p)),
        None => SocketAddr::V4(SocketAddrV4::new(GRUPPE, PORT)),
    }
}

/// Fragen stellen und alle Eintraege sammeln, die bis `frist` ankommen; die
/// Fragen gehen nach 0, 300 und 900 ms noch einmal hinaus (Pakete gehen im
/// WLAN verloren). `fertig` darf frueher beenden.
fn sammeln(fragen: &[(String, u16)], ms: u64, fertig: impl Fn(&[Eintrag]) -> bool) -> Result<Vec<Eintrag>, String> {
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(|e| format!("MDNS: {}", e))?;
    sock.set_multicast_ttl_v4(255).ok();
    sock.set_read_timeout(Some(Duration::from_millis(20))).ok();
    let id: u16 = (std::process::id() as u16).wrapping_mul(31).wrapping_add(Instant::now().elapsed().subsec_nanos() as u16) | 1;
    let ziel = ziel();
    let anfang = Instant::now();
    let frist = anfang + Duration::from_millis(ms);
    let wiederholen = [0u64, 300, 900];
    let mut gesendet = 0;
    let mut eintraege = Vec::new();
    let mut buf = vec![0u8; 9000];
    while Instant::now() < frist {
        if gesendet < wiederholen.len() && anfang.elapsed() >= Duration::from_millis(wiederholen[gesendet]) {
            for (n, t) in fragen { let _ = sock.send_to(&frage(id, n, *t), ziel); }
            gesendet += 1;
        }
        match sock.recv_from(&mut buf) {
            Ok((n, _)) => if let Ok(Some((_, liste))) = antwort_lesen(&buf[..n]) {
                eintraege.extend(liste);
                if fertig(&eintraege) { break; }
            },
            Err(_) => {}
        }
    }
    Ok(eintraege)
}

/// `name.local` -> IPv4-Adresse als Text, "" wenn niemand antwortet.
pub fn aufloesen(name: &str, ms: u64) -> Result<String, String> {
    let n = lokal(name);
    let passt = |e: &Eintrag| e.name.eq_ignore_ascii_case(&n) && matches!(e.daten, Daten::A(_));
    let liste = sammeln(&[(n.clone(), T_A)], ms, |l| l.iter().any(passt))?;
    Ok(liste.iter().find(|e| passt(e)).map(|e| match e.daten { Daten::A(ip) => ip.to_string(), _ => String::new() }).unwrap_or_default())
}

/// Alle Dienste vom Typ `dienst` ("_http._tcp"), die bis `ms` antworten.
/// Fehlt einem Dienst SRV oder Adresse, wird nachgefragt.
pub fn suchen(dienst: &str, ms: u64) -> Result<Vec<Dienst>, String> {
    let d = lokal(dienst);
    let mut eintraege = sammeln(&[(d.clone(), T_PTR)], ms, |_| false)?;
    let mut liste = dienste_aus(&eintraege, &d);
    let offen: Vec<(String, u16)> = liste.iter().flat_map(|x| {
        let mut f = Vec::new();
        if x.host.is_empty() { f.push((x.name.clone(), T_SRV)); f.push((x.name.clone(), T_TXT)); }
        else if x.adresse.is_empty() { f.push((x.host.clone(), T_A)); }
        f
    }).collect();
    if !offen.is_empty() {
        eintraege.extend(sammeln(&offen, (ms / 2).max(300), |_| false)?);
        liste = dienste_aus(&eintraege, &d);
        let adressen: Vec<(String, u16)> = liste.iter().filter(|x| !x.host.is_empty() && x.adresse.is_empty()).map(|x| (x.host.clone(), T_A)).collect();
        if !adressen.is_empty() {
            eintraege.extend(sammeln(&adressen, (ms / 2).max(300), |_| false)?);
            liste = dienste_aus(&eintraege, &d);
        }
    }
    liste.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(liste)
}

// --- Ankuendigen ----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Ankuendigung { pub name: String, pub dienst: String, pub host: String, pub port: u16, pub txt: Vec<(String, String)> }

impl Ankuendigung {
    pub fn instanz(&self) -> String { format!("{}.{}", self.name, self.dienst) }
}

pub struct Antworter {
    stop: Arc<Mutex<bool>>,
    faden: Option<std::thread::JoinHandle<()>>,
    pub liste: Arc<Mutex<Vec<Ankuendigung>>>,
}

impl Drop for Antworter {
    fn drop(&mut self) {
        *self.stop.lock().unwrap_or_else(|e| e.into_inner()) = true;
        if let Some(f) = self.faden.take() { let _ = f.join(); }
    }
}

/// Die eigene Adresse, unter der andere diesen Rechner erreichen: die, ueber
/// die ein Paket an die Gruppe hinausginge.
pub fn eigene_adresse() -> Ipv4Addr {
    if testport().is_some() { return Ipv4Addr::LOCALHOST; }
    let s = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok();
    let a = s.and_then(|s| {
        s.connect((Ipv4Addr::new(192, 168, 0, 1), 9)).ok()?;
        match s.local_addr().ok()? { SocketAddr::V4(v) if !v.ip().is_unspecified() => Some(*v.ip()), _ => None }
    });
    a.unwrap_or(Ipv4Addr::LOCALHOST)
}

fn anschluss() -> Result<(UdpSocket, bool), String> {
    use socket2::{Domain, Protocol, Socket, Type};
    if let Some(p) = testport() {
        let s = UdpSocket::bind((Ipv4Addr::LOCALHOST, p)).map_err(|e| format!("MDNS_ANNOUNCE: 127.0.0.1:{} -- {}", p, e))?;
        return Ok((s, false));
    }
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).map_err(|e| format!("MDNS_ANNOUNCE: {}", e))?;
    s.set_reuse_address(true).ok();
    #[cfg(all(unix, not(target_os = "emscripten")))]
    s.set_reuse_port(true).ok();
    s.bind(&SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT)).into())
        .map_err(|e| format!("MDNS_ANNOUNCE: Anschluss 5353 ist nicht zu bekommen -- {}", e))?;
    s.join_multicast_v4(&GRUPPE, &Ipv4Addr::UNSPECIFIED).map_err(|e| format!("MDNS_ANNOUNCE: Gruppe 224.0.0.251 -- {}", e))?;
    s.set_multicast_loop_v4(true).ok();
    s.set_multicast_ttl_v4(255).ok();
    Ok((s.into(), true))
}

/// Den Antworter starten (einmal je Programm); er beantwortet Fragen nach
/// den Diensten in `liste` und nach ihren Rechnernamen.
pub fn antworter_starten() -> Result<Antworter, String> {
    let (sock, gruppe) = anschluss()?;
    sock.set_read_timeout(Some(Duration::from_millis(50))).ok();
    let stop = Arc::new(Mutex::new(false));
    let liste: Arc<Mutex<Vec<Ankuendigung>>> = Arc::new(Mutex::new(Vec::new()));
    let (s2, l2) = (stop.clone(), liste.clone());
    let faden = std::thread::spawn(move || {
        let adresse = eigene_adresse();
        let mut buf = vec![0u8; 9000];
        while !*s2.lock().unwrap_or_else(|e| e.into_inner()) {
            let Ok((n, von)) = sock.recv_from(&mut buf) else { continue };
            let p = &buf[..n];
            if n < 12 || p[2] & 0x80 != 0 { continue; }
            let id = u16::from_be_bytes([p[0], p[1]]);
            let fragen = u16::from_be_bytes([p[4], p[5]]) as usize;
            let mut i = 12;
            let mut gefragt = Vec::new();
            for _ in 0..fragen {
                let Ok((name, j)) = name_lesen(p, i) else { break };
                let typ = u16::from_be_bytes([p.get(j).copied().unwrap_or(0), p.get(j + 1).copied().unwrap_or(0)]);
                let qu = p.get(j + 2).map_or(false, |b| b & 0x80 != 0);
                gefragt.push((name, typ, qu));
                i = j + 4;
            }
            let liste = l2.lock().unwrap_or_else(|e| e.into_inner()).clone();
            for a in &liste {
                let trifft = gefragt.iter().any(|(name, typ, _)| {
                    let g = |x: &str| name.eq_ignore_ascii_case(x);
                    (g(&a.dienst) && matches!(*typ, T_PTR | T_ANY))
                        || (g(&a.instanz()) && matches!(*typ, T_SRV | T_TXT | T_ANY))
                        || (g(&a.host) && matches!(*typ, T_A | T_ANY))
                        || (g("_services._dns-sd._udp.local") && *typ == T_PTR)
                });
                if !trifft { continue; }
                // Eine Frage von einem anderen Anschluss als 5353 (oder ohne
                // Gruppe) bekommt die Antwort direkt und mit ihrer Kennung,
                // sonst geht sie an die Gruppe.
                let direkt = !gruppe || von.port() != PORT || gefragt.iter().any(|(_, _, qu)| *qu);
                let antwort = antwort_bauen(if direkt { id } else { 0 }, a, adresse, 120);
                let wohin = if direkt { von } else { SocketAddr::V4(SocketAddrV4::new(GRUPPE, PORT)) };
                let _ = sock.send_to(&antwort, wohin);
                if direkt && gruppe && von.port() == PORT {
                    let _ = sock.send_to(&antwort_bauen(0, a, adresse, 120), SocketAddr::V4(SocketAddrV4::new(GRUPPE, PORT)));
                }
            }
        }
        // Abmelden: alle Eintraege mit Lebensdauer 0.
        let liste = l2.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if gruppe { for a in &liste { let _ = sock.send_to(&antwort_bauen(0, a, adresse, 0), SocketAddr::V4(SocketAddrV4::new(GRUPPE, PORT))); } }
    });
    Ok(Antworter { stop, faden: Some(faden), liste })
}

/// Einmal von selbst an die Gruppe melden (RFC 6762 8.3) -- sonst erfaehrt
/// nur, wer fragt.
pub fn melden(a: &Ankuendigung) {
    if testport().is_some() { return; }
    if let Ok(s) = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)) {
        s.set_multicast_ttl_v4(255).ok();
        let _ = s.send_to(&antwort_bauen(0, a, eigene_adresse(), 120), SocketAddr::V4(SocketAddrV4::new(GRUPPE, PORT)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ank() -> Ankuendigung {
        Ankuendigung { name: "Kueche".into(), dienst: "_http._tcp.local".into(), host: "kueche.local".into(), port: 8080,
            txt: vec![("pfad".into(), "/".into()), ("an".into(), String::new())] }
    }

    #[test]
    fn frage_wie_dig() {
        // dig @224.0.0.251 -p 5353 esp32.local A: Kennung, eine Frage, Name, Typ 1, Klasse 1
        let f = frage(0x1234, "esp32.local", T_A);
        assert_eq!(&f[..12], &[0x12, 0x34, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&f[12..], &[5, b'e', b's', b'p', b'3', b'2', 5, b'l', b'o', b'c', b'a', b'l', 0, 0, 1, 0, 1]);
    }

    #[test]
    fn antwort_hin_und_zurueck() {
        let p = antwort_bauen(7, &ank(), Ipv4Addr::new(192, 168, 1, 50), 120);
        let (id, liste) = antwort_lesen(&p).unwrap().unwrap();
        assert_eq!(id, 7);
        let d = dienste_aus(&liste, "_http._tcp.local");
        assert_eq!(d, vec![Dienst {
            name: "Kueche._http._tcp.local".into(), host: "kueche.local".into(), adresse: "192.168.1.50".into(),
            port: 8080, txt: vec![("pfad".into(), "/".into()), ("an".into(), String::new())],
        }]);
        assert_eq!(kurzname(&d[0].name, "_http._tcp.local"), "Kueche");
        // Eine Frage ist keine Antwort.
        assert!(antwort_lesen(&frage(1, "x.local", T_A)).unwrap().is_none());
    }

    #[test]
    fn verweise_im_namen() {
        // Antwort mit Namenskompression: "a.local" A, Name als Zeiger auf die Frage.
        let mut p = vec![0, 0, 0x84, 0, 0, 1, 0, 1, 0, 0, 0, 0];
        p.extend_from_slice(&[1, b'a', 5, b'l', b'o', b'c', b'a', b'l', 0, 0, 1, 0, 1]);
        p.extend_from_slice(&[0xC0, 12, 0, 1, 0x80, 1, 0, 0, 0, 120, 0, 4, 10, 0, 0, 9]);
        let (_, l) = antwort_lesen(&p).unwrap().unwrap();
        assert_eq!(l, vec![Eintrag { name: "a.local".into(), typ: T_A, daten: Daten::A(Ipv4Addr::new(10, 0, 0, 9)) }]);
        // Zeiger auf sich selbst
        let mut kaputt = vec![0, 0, 0x84, 0, 0, 0, 0, 1, 0, 0, 0, 0];
        kaputt.extend_from_slice(&[0xC0, 12, 0, 1, 0, 1, 0, 0, 0, 1, 0, 4, 1, 2, 3, 4]);
        assert!(antwort_lesen(&kaputt).is_err());
    }

    #[test]
    fn namen_und_txt() {
        assert_eq!(lokal("esp32"), "esp32.local");
        assert_eq!(lokal("esp32.LOCAL."), "esp32.LOCAL");
        assert_eq!(lokal("_http._tcp"), "_http._tcp.local");
        assert_eq!(txt_aus_text("a=1; b = zwei ;c"), vec![("a".into(), "1".into()), ("b".into(), "zwei".into()), ("c".into(), String::new())]);
    }
}
