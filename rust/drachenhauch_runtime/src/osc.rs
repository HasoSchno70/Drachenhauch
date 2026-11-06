//! OSC (Open Sound Control 1.0, OSC_*): Nachrichten ueber UDP -- das, was
//! TouchOSC, Pure Data, Max, SuperCollider, Ableton (ueber Bruecken), VJ-
//! Programme und viele Bastelprojekte sprechen.
//!
//! Eine Nachricht ist eine Adresse ("/synth/1/freq") und eine Liste getypter
//! Werte. Gelesen werden i f s b h d t T F N I c r m S, auch in Buendeln
//! (#bundle, verschachtelt -- die Nachrichten darin kommen einzeln an);
//! geschrieben wird aus den Werten von Drachenhauch: INTEGER -> i (ausserhalb
//! von 32 Bit h), FLOAT -> f, STRING -> s, BOOLEAN -> T/F, BUFFER -> b, NIL -> N.

use std::collections::{HashMap, VecDeque};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

#[derive(Clone, Debug, PartialEq)]
pub enum Wert {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Text(String),
    Blob(Vec<u8>),
    Wahr,
    Falsch,
    Nichts,
    Unendlich,
    Zeit(u64),
    Zeichen(char),
    Farbe(u32),
    Midi([u8; 4]),
}

impl Wert {
    pub fn kennung(&self) -> char {
        match self {
            Wert::Int(_) => 'i', Wert::Long(_) => 'h', Wert::Float(_) => 'f', Wert::Double(_) => 'd',
            Wert::Text(_) => 's', Wert::Blob(_) => 'b', Wert::Wahr => 'T', Wert::Falsch => 'F',
            Wert::Nichts => 'N', Wert::Unendlich => 'I', Wert::Zeit(_) => 't', Wert::Zeichen(_) => 'c',
            Wert::Farbe(_) => 'r', Wert::Midi(_) => 'm',
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Nachricht { pub adresse: String, pub werte: Vec<Wert> }

// --- Kodieren (rein) --------------------------------------------------------

fn auf_vier(b: &mut Vec<u8>) { while b.len() % 4 != 0 { b.push(0); } }

fn text_schreiben(b: &mut Vec<u8>, s: &str) {
    b.extend_from_slice(s.as_bytes());
    b.push(0);
    auf_vier(b);
}

pub fn kodieren(n: &Nachricht) -> Vec<u8> {
    let mut b = Vec::new();
    text_schreiben(&mut b, &n.adresse);
    let tags: String = std::iter::once(',').chain(n.werte.iter().map(|w| w.kennung())).collect();
    text_schreiben(&mut b, &tags);
    for w in &n.werte {
        match w {
            Wert::Int(x) => b.extend_from_slice(&x.to_be_bytes()),
            Wert::Long(x) => b.extend_from_slice(&x.to_be_bytes()),
            Wert::Float(x) => b.extend_from_slice(&x.to_be_bytes()),
            Wert::Double(x) => b.extend_from_slice(&x.to_be_bytes()),
            Wert::Text(s) => text_schreiben(&mut b, s),
            Wert::Blob(d) => { b.extend_from_slice(&(d.len() as i32).to_be_bytes()); b.extend_from_slice(d); auf_vier(&mut b); }
            Wert::Zeit(t) => b.extend_from_slice(&t.to_be_bytes()),
            Wert::Zeichen(c) => b.extend_from_slice(&(*c as u32).to_be_bytes()),
            Wert::Farbe(c) => b.extend_from_slice(&c.to_be_bytes()),
            Wert::Midi(m) => b.extend_from_slice(m),
            Wert::Wahr | Wert::Falsch | Wert::Nichts | Wert::Unendlich => {}
        }
    }
    b
}

// --- Lesen (rein) -----------------------------------------------------------

struct Leser<'a> { b: &'a [u8], p: usize }

impl<'a> Leser<'a> {
    fn text(&mut self) -> Result<String, String> {
        let rest = &self.b[self.p..];
        let ende = rest.iter().position(|&c| c == 0).ok_or("Zeichenkette ohne Ende")?;
        let s = String::from_utf8_lossy(&rest[..ende]).into_owned();
        self.p += (ende + 4) & !3;
        if self.p > self.b.len() { return Err("Zeichenkette ueber das Ende".into()); }
        Ok(s)
    }
    fn bytes<const N: usize>(&mut self) -> Result<[u8; N], String> {
        if self.p + N > self.b.len() { return Err("Nachricht zu kurz".into()); }
        let mut a = [0u8; N];
        a.copy_from_slice(&self.b[self.p..self.p + N]);
        self.p += N;
        Ok(a)
    }
}

/// Ein Paket lesen; Buendel kommen als ihre einzelnen Nachrichten heraus.
pub fn lesen(b: &[u8]) -> Result<Vec<Nachricht>, String> {
    let mut out = Vec::new();
    lesen_in(b, &mut out, 0)?;
    Ok(out)
}

fn lesen_in(b: &[u8], out: &mut Vec<Nachricht>, tiefe: usize) -> Result<(), String> {
    if tiefe > 8 { return Err("Buendel zu tief verschachtelt".into()); }
    if b.starts_with(b"#bundle\0") {
        let mut p = 16;
        while p + 4 <= b.len() {
            let n = i32::from_be_bytes(b[p..p + 4].try_into().unwrap());
            if n < 0 || p + 4 + n as usize > b.len() { return Err("Buendel-Element ueber das Ende".into()); }
            lesen_in(&b[p + 4..p + 4 + n as usize], out, tiefe + 1)?;
            p += 4 + n as usize;
        }
        return Ok(());
    }
    if !b.starts_with(b"/") { return Err("keine OSC-Nachricht (beginnt nicht mit /)".into()); }
    let mut l = Leser { b, p: 0 };
    let adresse = l.text()?;
    let mut werte = Vec::new();
    if l.p >= b.len() { out.push(Nachricht { adresse, werte }); return Ok(()); }
    let tags = l.text()?;
    let tags = tags.strip_prefix(',').ok_or("Typangabe ohne Komma")?;
    for t in tags.chars() {
        werte.push(match t {
            'i' => Wert::Int(i32::from_be_bytes(l.bytes::<4>()?)),
            'h' => Wert::Long(i64::from_be_bytes(l.bytes::<8>()?)),
            'f' => Wert::Float(f32::from_be_bytes(l.bytes::<4>()?)),
            'd' => Wert::Double(f64::from_be_bytes(l.bytes::<8>()?)),
            's' | 'S' => Wert::Text(l.text()?),
            'b' => {
                let n = i32::from_be_bytes(l.bytes::<4>()?);
                if n < 0 || l.p + n as usize > b.len() { return Err("Blob ueber das Ende".into()); }
                let d = b[l.p..l.p + n as usize].to_vec();
                l.p += (n as usize + 3) & !3;
                Wert::Blob(d)
            }
            't' => Wert::Zeit(u64::from_be_bytes(l.bytes::<8>()?)),
            'c' => Wert::Zeichen(char::from_u32(u32::from_be_bytes(l.bytes::<4>()?)).unwrap_or('?')),
            'r' => Wert::Farbe(u32::from_be_bytes(l.bytes::<4>()?)),
            'm' => Wert::Midi(l.bytes::<4>()?),
            'T' => Wert::Wahr,
            'F' => Wert::Falsch,
            'N' => Wert::Nichts,
            'I' => Wert::Unendlich,
            '[' | ']' => continue,
            anders => return Err(format!("unbekannter Typ '{}'", anders)),
        });
    }
    out.push(Nachricht { adresse, werte });
    Ok(())
}

// --- Adressmuster (rein) ------------------------------------------------------

/// Passt die Adresse auf das Muster? Je Teil zwischen den Schraegstrichen:
/// `?` ein Zeichen, `*` beliebig viele, `[abc]`/`[a-z]`/`[!a-z]` eines aus
/// einer Menge, `{rot,gruen}` eine der Moeglichkeiten.
pub fn passt(muster: &str, adresse: &str) -> bool {
    let m: Vec<&str> = muster.split('/').collect();
    let a: Vec<&str> = adresse.split('/').collect();
    m.len() == a.len() && m.iter().zip(&a).all(|(x, y)| teil_passt(&x.chars().collect::<Vec<_>>(), &y.chars().collect::<Vec<_>>()))
}

fn teil_passt(m: &[char], a: &[char]) -> bool {
    match m.first() {
        None => a.is_empty(),
        Some('*') => (0..=a.len()).any(|i| teil_passt(&m[1..], &a[i..])),
        Some('?') => !a.is_empty() && teil_passt(&m[1..], &a[1..]),
        Some('[') => {
            let Some(ende) = m.iter().position(|&c| c == ']') else { return false };
            let Some(&c) = a.first() else { return false };
            let menge = &m[1..ende];
            let (nicht, menge) = if menge.first() == Some(&'!') { (true, &menge[1..]) } else { (false, menge) };
            let mut drin = false;
            let mut i = 0;
            while i < menge.len() {
                if i + 2 < menge.len() && menge[i + 1] == '-' {
                    if c >= menge[i] && c <= menge[i + 2] { drin = true; }
                    i += 3;
                } else {
                    if c == menge[i] { drin = true; }
                    i += 1;
                }
            }
            drin != nicht && teil_passt(&m[ende + 1..], &a[1..])
        }
        Some('{') => {
            let Some(ende) = m.iter().position(|&c| c == '}') else { return false };
            let inhalt: String = m[1..ende].iter().collect();
            inhalt.split(',').any(|w| {
                let w: Vec<char> = w.chars().collect();
                a.len() >= w.len() && a[..w.len()] == w[..] && teil_passt(&m[ende + 1..], &a[w.len()..])
            })
        }
        Some(&c) => a.first() == Some(&c) && teil_passt(&m[1..], &a[1..]),
    }
}

// --- Anschluss ----------------------------------------------------------------

pub struct Osc {
    sock: UdpSocket,
    pub port: u16,
    eingang: VecDeque<(Nachricht, SocketAddr)>,
    aktuell: Option<(Nachricht, SocketAddr)>,
    ziele: HashMap<(String, u16), SocketAddr>,
    pub fehlerhaft: u64,
}

pub fn oeffnen(port: i64, bind: &str) -> Result<Osc, String> {
    if !(0..=65535).contains(&port) { return Err(format!("OSC_OPEN: Port {} ausserhalb 0..65535", port)); }
    let sock = UdpSocket::bind((bind, port as u16)).map_err(|e| format!("OSC_OPEN: {}:{} -- {}", bind, port, e))?;
    sock.set_nonblocking(true).map_err(|e| format!("OSC_OPEN: {}", e))?;
    sock.set_broadcast(true).ok();
    let port = sock.local_addr().map(|a| a.port()).unwrap_or(port as u16);
    Ok(Osc { sock, port, eingang: VecDeque::new(), aktuell: None, ziele: HashMap::new(), fehlerhaft: 0 })
}

impl Osc {
    fn ziel(&mut self, host: &str, port: u16) -> Result<SocketAddr, String> {
        if let Some(a) = self.ziele.get(&(host.to_string(), port)) { return Ok(*a); }
        let a = (host, port).to_socket_addrs().map_err(|e| format!("OSC_SEND: {}:{} -- {}", host, port, e))?
            .find(|a| a.is_ipv4() == self.sock.local_addr().map(|l| l.is_ipv4()).unwrap_or(true))
            .ok_or_else(|| format!("OSC_SEND: {} nicht aufloesbar", host))?;
        self.ziele.insert((host.to_string(), port), a);
        Ok(a)
    }

    pub fn senden(&mut self, host: &str, port: u16, n: &Nachricht) -> Result<(), String> {
        let ziel = self.ziel(host, port)?;
        let paket = kodieren(n);
        match self.sock.send_to(&paket, ziel) {
            Ok(_) => Ok(()),
            // Windows meldet ein Paket an einen geschlossenen Port beim
            // naechsten Senden/Empfangen (10054) -- fuer UDP kein Fehler.
            Err(e) if e.raw_os_error() == Some(10054) => Ok(()),
            Err(e) => Err(format!("OSC_SEND: {}:{} -- {}", host, port, e)),
        }
    }

    fn pumpen(&mut self) {
        let mut buf = vec![0u8; 65536];
        loop {
            match self.sock.recv_from(&mut buf) {
                Ok((n, von)) => match lesen(&buf[..n]) {
                    Ok(liste) => for m in liste { if self.eingang.len() >= 4096 { self.eingang.pop_front(); } self.eingang.push_back((m, von)); },
                    Err(_) => self.fehlerhaft += 1,
                },
                Err(e) if e.raw_os_error() == Some(10054) => continue,
                Err(_) => break,
            }
        }
    }

    pub fn naechste(&mut self) -> bool {
        self.pumpen();
        self.aktuell = self.eingang.pop_front();
        self.aktuell.is_some()
    }

    pub fn aktuell(&self) -> Option<&(Nachricht, SocketAddr)> { self.aktuell.as_ref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beispiel_aus_der_spezifikation() {
        // OSC 1.0: "/oscillator/4/frequency" ,f 440.0
        let n = Nachricht { adresse: "/oscillator/4/frequency".into(), werte: vec![Wert::Float(440.0)] };
        let b = kodieren(&n);
        let erwartet: Vec<u8> = [
            &b"/oscillator/4/frequency\0"[..], b",f\0\0", &[0x43, 0xdc, 0x00, 0x00],
        ].concat();
        assert_eq!(b, erwartet);
        // "/foo" 1000, -1, "hello", 1.234, 5.678
        let n = Nachricht { adresse: "/foo".into(), werte: vec![Wert::Int(1000), Wert::Int(-1), Wert::Text("hello".into()), Wert::Float(1.234), Wert::Float(5.678)] };
        let b = kodieren(&n);
        assert_eq!(&b[..8], b"/foo\0\0\0\0");
        assert_eq!(&b[8..16], b",iisff\0\0");
        assert_eq!(&b[16..20], &[0, 0, 3, 0xe8]);
        assert_eq!(&b[20..24], &[0xff; 4]);
        assert_eq!(&b[24..32], b"hello\0\0\0");
        assert_eq!(&b[32..36], &[0x3f, 0x9d, 0xf3, 0xb6]);
        assert_eq!(&b[36..40], &[0x40, 0xb5, 0xb2, 0x2d]);
        assert_eq!(lesen(&b).unwrap(), vec![n]);
    }

    #[test]
    fn alle_typen_hin_und_zurueck() {
        let n = Nachricht { adresse: "/a/b".into(), werte: vec![
            Wert::Int(-7), Wert::Long(1 << 40), Wert::Float(0.5), Wert::Double(1.0 / 3.0), Wert::Text("äöü".into()),
            Wert::Blob(vec![1, 2, 3]), Wert::Wahr, Wert::Falsch, Wert::Nichts, Wert::Unendlich, Wert::Zeit(1),
            Wert::Zeichen('x'), Wert::Farbe(0xFF8800FF), Wert::Midi([0, 0x90, 60, 100]),
        ] };
        let b = kodieren(&n);
        assert_eq!(b.len() % 4, 0);
        assert_eq!(lesen(&b).unwrap(), vec![n]);
    }

    #[test]
    fn buendel_werden_einzeln() {
        let a = kodieren(&Nachricht { adresse: "/eins".into(), werte: vec![Wert::Int(1)] });
        let c = kodieren(&Nachricht { adresse: "/zwei".into(), werte: vec![] });
        let mut innen = b"#bundle\0".to_vec();
        innen.extend_from_slice(&1u64.to_be_bytes());
        innen.extend_from_slice(&(c.len() as i32).to_be_bytes());
        innen.extend_from_slice(&c);
        let mut b = b"#bundle\0".to_vec();
        b.extend_from_slice(&1u64.to_be_bytes());
        for e in [&a, &innen] { b.extend_from_slice(&(e.len() as i32).to_be_bytes()); b.extend_from_slice(e); }
        let liste = lesen(&b).unwrap();
        assert_eq!(liste.iter().map(|n| n.adresse.as_str()).collect::<Vec<_>>(), vec!["/eins", "/zwei"]);
        assert!(lesen(b"keine").is_err());
        assert!(lesen(b"/x\0\0,q\0\0").is_err());
        // Ohne Typangabe (alte Sender): eine Nachricht ohne Werte
        assert_eq!(lesen(b"/alt\0\0\0\0").unwrap()[0].werte, vec![]);
    }

    #[test]
    fn muster() {
        assert!(passt("/synth/*/freq", "/synth/1/freq"));
        assert!(!passt("/synth/*/freq", "/synth/1/2/freq"));
        assert!(passt("/fader?", "/fader3"));
        assert!(passt("/fader[1-4]", "/fader3"));
        assert!(!passt("/fader[!1-4]", "/fader3"));
        assert!(passt("/farbe/{rot,gruen}", "/farbe/gruen"));
        assert!(!passt("/farbe/{rot,gruen}", "/farbe/blau"));
        assert!(passt("/a/b", "/a/b") && !passt("/a/b", "/a/c"));
        assert!(passt("/*", "/x") && passt("/x*y", "/xy") && passt("/x*y", "/xabcy"));
    }
}
