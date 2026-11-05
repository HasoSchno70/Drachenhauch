//! MQTT-Client (MQTT_*) fuer ESP32/IoT-Steuerung -- das im Maker-/IoT-Bereich
//! dominante Pub/Sub-Protokoll, direkt gegen die OASIS-MQTT-3.1.1-Spezifikation
//! implementiert (Feature `net`; TLS ueber dasselbe rustls+ring wie smtp).
//!
//! Umfang: QoS 0 und 1 beim Senden und Empfangen (QoS 1: der Broker
//! bestaetigt jede Nachricht mit PUBACK, eingehende bestaetigt der Client --
//! "mindestens einmal", eine Nachricht kann also doppelt ankommen),
//! SUBSCRIBE/UNSUBSCRIBE, letzter Wille, TLS (`mqtts://host`, eigene CA ueber
//! MQTT_TLS_CA). Bewusst NICHT: QoS 2 (vier Pakete je Nachricht, im
//! Bastlerbereich kaum benutzt), Sitzungen ueber eine Verbindung hinaus
//! (Clean Session ist immer an -- eine unbestaetigte Nachricht wird darum
//! nicht von selbst neu geschickt, MQTT_PENDING sagt, wie viele es sind),
//! MQTT 5.
#![cfg(feature = "net")]

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

const PKT_CONNECT: u8 = 1;
const PKT_CONNACK: u8 = 2;
const PKT_PUBLISH: u8 = 3;
const PKT_PUBACK: u8 = 4;
const PKT_SUBSCRIBE: u8 = 8;
const PKT_SUBACK: u8 = 9;
const PKT_UNSUBSCRIBE: u8 = 10;
const PKT_UNSUBACK: u8 = 11;
const PKT_PINGREQ: u8 = 12;
const PKT_PINGRESP: u8 = 13;
const PKT_DISCONNECT: u8 = 14;

/// Klartext oder TLS -- der Rest des Codes soll den Unterschied nicht sehen
/// (wie `Strom` in smtp.rs).
enum Strom {
    Klar(TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
}

impl Strom {
    fn tcp(&self) -> &TcpStream {
        match self {
            Strom::Klar(s) => s,
            Strom::Tls(s) => s.get_ref(),
        }
    }
}

impl Read for Strom {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Strom::Klar(s) => s.read(buf),
            Strom::Tls(s) => s.read(buf),
        }
    }
}

impl Write for Strom {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Strom::Klar(s) => s.write(buf),
            Strom::Tls(s) => s.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Strom::Klar(s) => s.flush(),
            Strom::Tls(s) => s.flush(),
        }
    }
}

pub struct Client {
    stream: Strom,
    connected: bool,
    keepalive: Duration,
    last_send: Instant,
    /// Wann das PINGREQ hinausging, auf dessen Antwort wir noch warten. Kommt
    /// nichts (Broker ohne Abmeldung weg: Strom, Netz -- dann gibt es auch
    /// kein RST), gilt die Verbindung nach einer halben Keepalive-Zeit als
    /// verloren. Ohne das merkte der Client es nie.
    ping_offen: Option<Instant>,
    next_packet_id: u16,
    /// QoS-1-Nachrichten, auf deren PUBACK wir noch warten (Paketnummer).
    unbestaetigt: HashMap<u16, Instant>,
    rx_pending: Vec<u8>,
    incoming: VecDeque<(String, String)>,
    /// Vom letzten `next_message()` aufgerufene (Topic, Payload) -- Getter-
    /// Paar-Muster wie `DB_NEXT` + `DB_GET_*` im db-Modul.
    current: Option<(String, String)>,
}

// ------------------------------------------------------------------
// Pure Encode/Decode-Funktionen (fuer #[test] ohne echten Broker)
// ------------------------------------------------------------------

/// "Remaining Length"-Variable-Encoding aus der MQTT-3.1.1-Spec (max. 4 Byte,
/// bis 268.435.455). Pure (fuer #[test]).
fn encode_remaining_length(mut n: usize) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (n % 128) as u8;
        n /= 128;
        if n > 0 { byte |= 0x80; }
        out.push(byte);
        if n == 0 { break; }
    }
    out
}

/// Liefert (Wert, verbrauchte Bytes) oder None, wenn der Puffer noch nicht
/// genug Bytes fuer eine vollstaendige Remaining-Length-Sequenz enthaelt.
fn decode_remaining_length(buf: &[u8]) -> Option<(usize, usize)> {
    let mut multiplier: usize = 1;
    let mut value: usize = 0;
    let mut i = 0;
    loop {
        let byte = *buf.get(i)?;
        value += (byte & 0x7F) as usize * multiplier;
        multiplier *= 128;
        i += 1;
        if byte & 0x80 == 0 { break; }
        if multiplier > 128 * 128 * 128 * 128 { return None; } // malformed
    }
    Some((value, i))
}

fn encode_str(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(2 + bytes.len());
    out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    out.extend_from_slice(bytes);
    out
}

/// Liest ein laengenpraefixiertes UTF-8-String-Feld vom Puffer-Anfang;
/// liefert (String, Rest) oder None wenn unvollstaendig.
fn decode_str(buf: &[u8]) -> Option<(String, &[u8])> {
    if buf.len() < 2 { return None; }
    let len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
    if buf.len() < 2 + len { return None; }
    Some((String::from_utf8_lossy(&buf[2..2 + len]).into_owned(), &buf[2 + len..]))
}

fn wrap_packet(type_and_flags: u8, body: &[u8]) -> Vec<u8> {
    let mut packet = vec![type_and_flags];
    packet.extend_from_slice(&encode_remaining_length(body.len()));
    packet.extend_from_slice(body);
    packet
}

/// Der "letzte Wille" eines Clients: der Broker veroeffentlicht ihn, sobald
/// die Verbindung abreisst, ohne dass der Client sich abgemeldet hat (Strom
/// weg, Absturz, Netz). Ohne ihn bleibt ein `online` mit retain stehen, und
/// jeder, der spaeter dazukommt, haelt das tote Geraet fuer lebendig.
#[derive(Clone)]
pub struct Wille {
    pub topic: String,
    pub inhalt: Vec<u8>,
    pub retain: bool,
}

fn encode_connect(client_id: &str, keepalive_s: u16, username: Option<&str>, password: Option<&str>,
                  wille: Option<&Wille>) -> Vec<u8> {
    let mut body = encode_str("MQTT");
    body.push(0x04); // Protocol Level: MQTT 3.1.1
    let mut flags: u8 = 0x02; // Clean Session
    if username.is_some() { flags |= 0x80; }
    if password.is_some() { flags |= 0x40; }
    // Will-Flag 0x04, Will-QoS 0 (Bits 3-4), Will-Retain 0x20
    if let Some(w) = wille { flags |= 0x04; if w.retain { flags |= 0x20; } }
    body.push(flags);
    body.extend_from_slice(&keepalive_s.to_be_bytes());
    body.extend_from_slice(&encode_str(client_id));
    if let Some(w) = wille {
        body.extend_from_slice(&encode_str(&w.topic));
        body.extend_from_slice(&(w.inhalt.len() as u16).to_be_bytes());
        body.extend_from_slice(&w.inhalt);
    }
    if let Some(u) = username { body.extend_from_slice(&encode_str(u)); }
    if let Some(p) = password { body.extend_from_slice(&encode_str(p)); }
    wrap_packet(PKT_CONNECT << 4, &body)
}

/// PUBLISH: Bits 1-2 des Kopfbytes sind das QoS, Bit 0 retain. Ab QoS 1
/// steht hinter dem Topic die Paketnummer (3.3.2.2).
fn encode_publish(topic: &str, payload: &[u8], retain: bool, qos: u8, packet_id: u16) -> Vec<u8> {
    let flags = (PKT_PUBLISH << 4) | ((qos & 0x03) << 1) | if retain { 1 } else { 0 }; // DUP 0
    let mut body = encode_str(topic);
    if qos > 0 {
        body.extend_from_slice(&packet_id.to_be_bytes());
    }
    body.extend_from_slice(payload);
    wrap_packet(flags, &body)
}

fn encode_subscribe(packet_id: u16, topic: &str, qos: u8) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&packet_id.to_be_bytes());
    body.extend_from_slice(&encode_str(topic));
    body.push(qos & 0x03); // angefragtes QoS
    wrap_packet((PKT_SUBSCRIBE << 4) | 0x02, &body) // Flags 0010 sind in der Spec Pflicht
}

fn encode_unsubscribe(packet_id: u16, topic: &str) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&packet_id.to_be_bytes());
    body.extend_from_slice(&encode_str(topic));
    wrap_packet((PKT_UNSUBSCRIBE << 4) | 0x02, &body) // Flags 0010 Pflicht wie bei SUBSCRIBE
}

fn encode_puback(packet_id: u16) -> Vec<u8> {
    wrap_packet(PKT_PUBACK << 4, &packet_id.to_be_bytes())
}

#[derive(Debug, PartialEq)]
enum Event {
    ConnAck { return_code: u8 },
    /// `packet_id` ist 0 bei QoS 0.
    Publish { topic: String, payload: String, qos: u8, packet_id: u16 },
    PubAck(u16),
    PingResp,
    SubAck,
    UnsubAck,
    Other,
}

/// Parst so viele vollstaendige MQTT-Pakete wie moeglich vom Puffer-Anfang;
/// gibt die erkannten Events + den nicht verbrauchten Rest zurueck (fuer den
/// naechsten update()-Aufruf, falls ein Paket quer ueber zwei TCP-Reads
/// ankam -- gleiches Muster wie firmata.rs' parse_messages). Pure Funktion.
fn parse_packets(mut buf: Vec<u8>) -> (Vec<Event>, Vec<u8>) {
    let mut events = Vec::new();
    let mut i = 0;
    loop {
        if i >= buf.len() { break; }
        let type_flags = buf[i];
        let ptype = type_flags >> 4;
        let flags = type_flags & 0x0F;
        match decode_remaining_length(&buf[i + 1..]) {
            None => break, // Fixed-Header (Remaining-Length) noch nicht vollstaendig da
            Some((rem_len, rl_bytes)) => {
                let header_len = 1 + rl_bytes;
                let total_len = header_len + rem_len;
                if i + total_len > buf.len() { break; } // Body noch nicht vollstaendig angekommen
                let body = &buf[i + header_len..i + total_len];
                let ev = match ptype {
                    PKT_CONNACK if body.len() >= 2 => Some(Event::ConnAck { return_code: body[1] }),
                    PKT_PUBLISH => decode_str(body).and_then(|(topic, rest)| {
                        let qos = (flags >> 1) & 0x03;
                        // Ab QoS 1 steht hinter dem Topic die 2-Byte-Paketnummer.
                        let (packet_id, payload_bytes) = if qos > 0 {
                            if rest.len() < 2 { return None; }
                            (u16::from_be_bytes([rest[0], rest[1]]), &rest[2..])
                        } else {
                            (0, rest)
                        };
                        Some(Event::Publish { topic, payload: String::from_utf8_lossy(payload_bytes).into_owned(), qos, packet_id })
                    }),
                    PKT_PUBACK if body.len() >= 2 => Some(Event::PubAck(u16::from_be_bytes([body[0], body[1]]))),
                    PKT_PINGRESP => Some(Event::PingResp),
                    PKT_SUBACK => Some(Event::SubAck),
                    PKT_UNSUBACK => Some(Event::UnsubAck),
                    _ => Some(Event::Other),
                };
                if let Some(ev) = ev { events.push(ev); }
                i += total_len;
            }
        }
    }
    let rest = buf.split_off(i);
    (events, rest)
}

// ------------------------------------------------------------------
// I/O (Transport) -- duenne Huelle um TcpStream + die reinen Funktionen oben
// ------------------------------------------------------------------

/// `mqtts://host` (auch `ssl://`, `tls://`) = verschluesselt; `mqtt://host`,
/// `tcp://host` oder nur der Name = Klartext. Liefert (Name, TLS?).
fn ziel_lesen(host: &str) -> (String, bool) {
    let h = host.trim();
    let klein = h.to_ascii_lowercase();
    for (vorsatz, tls) in [("mqtts://", true), ("ssl://", true), ("tls://", true), ("mqtt://", false), ("tcp://", false)] {
        if klein.starts_with(vorsatz) {
            return (h[vorsatz.len()..].trim_end_matches('/').to_string(), tls);
        }
    }
    (h.to_string(), false)
}

/// Die Wurzeln, denen TLS glaubt: ohne eigene CA die von webpki-roots (wie
/// smtp), mit `ca` NUR die aus dieser PEM-Datei -- so verbindet man sich mit
/// einem Mosquitto, dessen Zertifikat man selbst ausgestellt hat.
fn tls_wurzeln(ca: Option<&str>) -> Result<rustls::RootCertStore, String> {
    let mut wurzeln = rustls::RootCertStore::empty();
    match ca {
        None => wurzeln.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned()),
        Some(pfad) => {
            use rustls::pki_types::pem::PemObject;
            let daten = std::fs::read(pfad).map_err(|e| format!("MQTT_CONNECT: CA-Datei {}: {}", pfad, e))?;
            let mut n = 0;
            for z in rustls::pki_types::CertificateDer::pem_slice_iter(&daten) {
                let z = z.map_err(|e| format!("MQTT_CONNECT: CA-Datei {}: {:?}", pfad, e))?;
                wurzeln.add(z).map_err(|e| format!("MQTT_CONNECT: CA-Datei {}: {}", pfad, e))?;
                n += 1;
            }
            if n == 0 {
                return Err(format!("MQTT_CONNECT: in der CA-Datei {} steht kein Zertifikat (erwartet PEM, -----BEGIN CERTIFICATE-----)", pfad));
            }
        }
    }
    Ok(wurzeln)
}

fn tls_strom(tcp: TcpStream, name: &str, ca: Option<&str>) -> Result<Strom, String> {
    let wurzeln = tls_wurzeln(ca)?;
    // Der Anbieter wird ausdruecklich genannt (wie smtp): waeren einmal zwei
    // Krypto-Anbieter im Baum, liefe `ClientConfig::builder()` in einen Absturz.
    let anbieter = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    let konfig = rustls::ClientConfig::builder_with_provider(anbieter)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("MQTT_CONNECT: TLS liess sich nicht einrichten: {}", e))?
        .with_root_certificates(wurzeln)
        .with_no_client_auth();
    let sn = rustls::pki_types::ServerName::try_from(name.to_string())
        .map_err(|_| format!("MQTT_CONNECT: {:?} ist kein gueltiger Servername fuer TLS", name))?;
    let verbindung = rustls::ClientConnection::new(std::sync::Arc::new(konfig), sn)
        .map_err(|e| format!("MQTT_CONNECT: TLS-Verbindung fehlgeschlagen: {}", e))?;
    Ok(Strom::Tls(Box::new(rustls::StreamOwned::new(verbindung, tcp))))
}

#[allow(clippy::too_many_arguments)]
pub fn connect(
    host: &str,
    port: i64,
    client_id: &str,
    keepalive_s: i64,
    username: Option<&str>,
    password: Option<&str>,
    wille: Option<&Wille>,
    ca: Option<&str>,
) -> Result<Client, String> {
    if !(0..=65535).contains(&port) {
        return Err(format!("MQTT_CONNECT: Port {} ausserhalb 0..65535", port));
    }
    let (host, tls) = ziel_lesen(host);
    let host = host.as_str();
    let addr_str = format!("{}:{}", host, port);
    let sockaddr = addr_str
        .to_socket_addrs()
        .map_err(|e| format!("MQTT_CONNECT: {}", e))?
        .next()
        .ok_or_else(|| format!("MQTT_CONNECT: Adresse {} nicht aufloesbar", addr_str))?;
    let tcp = TcpStream::connect_timeout(&sockaddr, Duration::from_secs(5))
        .map_err(|e| format!("MQTT_CONNECT: {}", e))?;
    // Waehrend des Aufbaus (TLS-Handschlag, CONNACK) darf gewartet werden;
    // danach liest MQTT_UPDATE ohne Warten (siehe unten).
    tcp.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let stream = if tls { tls_strom(tcp, host, ca)? } else { Strom::Klar(tcp) };

    let ka = keepalive_s.clamp(1, 65535) as u16;
    let mut client = Client {
        stream,
        connected: false,
        keepalive: Duration::from_secs(ka as u64),
        last_send: Instant::now(),
        ping_offen: None,
        next_packet_id: 0,
        unbestaetigt: HashMap::new(),
        rx_pending: Vec::new(),
        incoming: VecDeque::new(),
        current: None,
    };

    let pkt = encode_connect(client_id, ka, username, password, wille);
    // Bei TLS laeuft hier der Handschlag -- ein falsches Zertifikat zeigt sich
    // also als Schreibfehler.
    client.stream.write_all(&pkt).and_then(|_| client.stream.flush()).map_err(|e| {
        if tls {
            format!("MQTT_CONNECT: TLS-Verbindung fehlgeschlagen: {}", e)
        } else {
            format!("MQTT_CONNECT: Schreiben fehlgeschlagen: {}", e)
        }
    })?;
    client.last_send = Instant::now();

    // Kurzes blockierendes Warten auf CONNACK -- einmaliger Setup-Kosten
    // (gleiches Muster wie FIRMATA_OPEN's Arduino-Reset-Wartezeit).
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut buf = Vec::new();
    loop {
        if Instant::now() > deadline {
            return Err("MQTT_CONNECT: Zeitueberschreitung beim Warten auf CONNACK".into());
        }
        let mut chunk = [0u8; 512];
        match client.stream.read(&mut chunk) {
            Ok(0) => return Err("MQTT_CONNECT: Verbindung vom Broker geschlossen".into()),
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                let (events, rest) = parse_packets(std::mem::take(&mut buf));
                buf = rest;
                for ev in events {
                    if let Event::ConnAck { return_code } = ev {
                        if return_code != 0 {
                            return Err(format!("MQTT_CONNECT: Broker lehnte ab (Return-Code {})", return_code));
                        }
                        client.connected = true;
                        client.rx_pending = buf;
                        // Kurzer Timeout: MQTT_UPDATE wird pro Frame gepollt
                        // (wie FIRMATA_UPDATE/INPUT_UPDATE), darf also nicht
                        // spuerbar blockieren.
                        client.stream.tcp().set_read_timeout(Some(Duration::from_millis(1))).ok();
                        return Ok(client);
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut || e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) if tls => return Err(format!("MQTT_CONNECT: TLS-Verbindung fehlgeschlagen: {}", e)),
            Err(e) => return Err(format!("MQTT_CONNECT: Lesefehler: {}", e)),
        }
    }
}

pub fn is_connected(c: &Client) -> bool { c.connected }

/// Wie lange auf die Antwort zu einem PINGREQ gewartet wird: die halbe
/// Keepalive-Zeit, mindestens 2 s (sonst gaelte bei Keepalive 1 schon eine
/// kleine Verzoegerung als Ausfall).
fn ping_frist(keepalive: Duration) -> Duration {
    (keepalive / 2).max(Duration::from_secs(2))
}

/// Ein Paket hinausschicken. Scheitert das Schreiben, ist die Verbindung weg
/// -- das ist ein ZUSTAND (MQTT_IS_CONNECTED wird FALSE), kein Laufzeitfehler:
/// ein Broker, der neu startet, darf ein Dashboard nicht beenden.
fn senden(c: &mut Client, pkt: &[u8]) -> bool {
    if !c.connected { return false; }
    match c.stream.write_all(pkt).and_then(|_| c.stream.flush()) {
        Ok(()) => { c.last_send = Instant::now(); true }
        Err(_) => { c.connected = false; false }
    }
}

/// Die naechste freie Paketnummer (1..65535; eine noch unbestaetigte wird
/// uebersprungen, sonst hielte ein spaetes PUBACK die falsche Nachricht fuer
/// zugestellt).
fn naechste_nummer(c: &mut Client) -> u16 {
    loop {
        c.next_packet_id = if c.next_packet_id == u16::MAX { 1 } else { c.next_packet_id + 1 };
        if !c.unbestaetigt.contains_key(&c.next_packet_id) {
            return c.next_packet_id;
        }
    }
}

pub fn qos_pruefen(fn_: &str, qos: i64) -> Result<u8, String> {
    match qos {
        0 | 1 => Ok(qos as u8),
        2 => Err(format!("{}: QoS 2 kann dieser Client nicht -- 0 oder 1", fn_)),
        _ => Err(format!("{}: QoS {} gibt es nicht -- 0 oder 1", fn_, qos)),
    }
}

/// TRUE, wenn die Nachricht hinausging; FALSE, wenn die Verbindung weg ist.
/// Mit QoS 1 zaehlt sie, bis der Broker sie bestaetigt (MQTT_PENDING).
pub fn publish(c: &mut Client, topic: &str, payload: &str, retain: bool, qos: u8) -> bool {
    let id = if qos > 0 { naechste_nummer(c) } else { 0 };
    let pkt = encode_publish(topic, payload.as_bytes(), retain, qos, id);
    let ok = senden(c, &pkt);
    if ok && qos > 0 {
        c.unbestaetigt.insert(id, Instant::now());
    }
    ok
}

pub fn subscribe(c: &mut Client, topic: &str, qos: u8) -> bool {
    let id = naechste_nummer(c);
    let pkt = encode_subscribe(id, topic, qos);
    senden(c, &pkt)
}

pub fn unsubscribe(c: &mut Client, topic: &str) -> bool {
    let id = naechste_nummer(c);
    let pkt = encode_unsubscribe(id, topic);
    senden(c, &pkt)
}

/// Wie viele QoS-1-Nachrichten der Broker noch nicht bestaetigt hat.
pub fn pending(c: &Client) -> usize {
    c.unbestaetigt.len()
}

/// Wartet, bis alle QoS-1-Nachrichten bestaetigt sind (hoechstens `ms`).
/// TRUE = alle angekommen; FALSE = Zeit um oder Verbindung weg.
pub fn flush(c: &mut Client, ms: i64) -> Result<bool, String> {
    let frist = Instant::now() + Duration::from_millis(ms.max(0) as u64);
    loop {
        update(c)?;
        if c.unbestaetigt.is_empty() { return Ok(true); }
        if !c.connected || Instant::now() > frist { return Ok(false); }
        std::thread::sleep(Duration::from_millis(1));
    }
}

pub fn disconnect(c: &mut Client) {
    if c.connected {
        let _ = c.stream.write_all(&wrap_packet(PKT_DISCONNECT << 4, &[]));
        let _ = c.stream.flush();
    }
    c.connected = false;
}

/// Liest alle aktuell verfuegbaren Bytes (nicht-blockierend) und aktualisiert
/// die eingehende Nachrichten-Queue. Sendet ein PINGREQ, wenn seit dem
/// letzten Senden mehr als die Haelfte des Keepalive-Intervalls vergangen
/// ist (Sicherheitsmarge gegen Broker-Timeout). Pro Frame aufrufen, wie
/// FIRMATA_UPDATE()/INPUT_UPDATE().
pub fn update(c: &mut Client) -> Result<(), String> {
    if !c.connected { return Ok(()); }
    if c.last_send.elapsed() > c.keepalive / 2 {
        if !senden(c, &wrap_packet(PKT_PINGREQ << 4, &[])) { return Ok(()); }
        if c.ping_offen.is_none() { c.ping_offen = Some(Instant::now()); }
    }
    if let Some(t) = c.ping_offen {
        if t.elapsed() > ping_frist(c.keepalive) { c.connected = false; return Ok(()); }
    }
    let mut chunk = [0u8; 4096];
    loop {
        let n = match c.stream.read(&mut chunk) {
            Ok(0) => { c.connected = false; return Ok(()); } // Broker hat geschlossen
            Ok(n) => n,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut || e.kind() == std::io::ErrorKind::WouldBlock => 0,
            // Jeder andere Lesefehler heisst: die Verbindung ist weg (unter
            // Windows 10054, wenn der Broker abbricht). Bis 2026.30 war das ein
            // Laufzeitfehler -- ein Dashboard ohne TRY endete, sobald der Broker
            // neu startete.
            Err(_) => { c.connected = false; return Ok(()); }
        };
        if n == 0 { return Ok(()); }
        c.ping_offen = None; // der Broker lebt
        let mut pending = std::mem::take(&mut c.rx_pending);
        pending.extend_from_slice(&chunk[..n]);
        let (events, rest) = parse_packets(pending);
        c.rx_pending = rest;
        for ev in events {
            match ev {
                Event::Publish { topic, payload, qos, packet_id } => {
                    // QoS 1: bestaetigen, sonst schickt der Broker sie wieder.
                    if qos == 1 && !senden(c, &encode_puback(packet_id)) {
                        return Ok(());
                    }
                    c.incoming.push_back((topic, payload));
                }
                Event::PubAck(id) => { c.unbestaetigt.remove(&id); }
                _ => {}
            }
        }
        if n < chunk.len() { return Ok(()); }
    }
}

pub fn next_message(c: &mut Client) -> bool {
    match c.incoming.pop_front() {
        Some(pair) => { c.current = Some(pair); true }
        None => false,
    }
}

pub fn message_topic(c: &Client) -> String {
    c.current.as_ref().map(|(t, _)| t.clone()).unwrap_or_default()
}

pub fn message_payload(c: &Client) -> String {
    c.current.as_ref().map(|(_, p)| p.clone()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_length_roundtrip_small_and_large() {
        for n in [0usize, 1, 127, 128, 16383, 16384, 2097151, 2097152] {
            let enc = encode_remaining_length(n);
            let (dec, consumed) = decode_remaining_length(&enc).unwrap();
            assert_eq!(dec, n, "n={n}");
            assert_eq!(consumed, enc.len(), "n={n}");
        }
    }

    #[test]
    fn remaining_length_incomplete_returns_none() {
        // 0x80 hat das Fortsetzungs-Bit gesetzt, aber kein Folgebyte da.
        assert_eq!(decode_remaining_length(&[0x80]), None);
        assert_eq!(decode_remaining_length(&[]), None);
    }

    #[test]
    fn encode_connect_matches_spec_byte_layout() {
        // Von Hand gegen die MQTT-3.1.1-Spec nachgerechnet: CONNECT("test", keepalive=60,
        // clean session, kein User/Passwort).
        let pkt = encode_connect("test", 60, None, None, None);
        let expected = vec![
            0x10, 0x10, // Fixed Header: Typ 1 (CONNECT), Remaining Length 16
            0x00, 0x04, b'M', b'Q', b'T', b'T', // Protokoll-Name "MQTT"
            0x04, // Protokoll-Level 3.1.1
            0x02, // Connect-Flags: nur Clean-Session
            0x00, 0x3C, // Keepalive 60 (0x003C)
            0x00, 0x04, b't', b'e', b's', b't', // Client-ID "test"
        ];
        assert_eq!(pkt, expected);
    }

    #[test]
    fn encode_connect_sets_username_password_flags() {
        let pkt = encode_connect("c", 10, Some("u"), Some("p"), None);
        let flags_byte = pkt[9]; // Index: 0x10,len, "MQTT"(6), level(1) -> Flags bei Index 9
        assert_eq!(flags_byte, 0x02 | 0x80 | 0x40);
    }

    #[test]
    fn ping_frist_halbe_keepalive_mindestens_zwei_sekunden() {
        assert_eq!(ping_frist(Duration::from_secs(60)), Duration::from_secs(30));
        assert_eq!(ping_frist(Duration::from_secs(1)), Duration::from_secs(2));
    }

    #[test]
    fn encode_connect_mit_letztem_willen() {
        // Von Hand nach MQTT 3.1.1, 3.1.2.5-3.1.2.7 und 3.1.3.2-3.1.3.3:
        // Will-Flag 0x04, Will-Retain 0x20, Topic und Nachricht hinter der
        // Client-Kennung, VOR Nutzername/Kennwort.
        let w = Wille { topic: "a/s".into(), inhalt: b"off".to_vec(), retain: true };
        let pkt = encode_connect("c", 10, Some("u"), None, Some(&w));
        assert_eq!(pkt[9], 0x02 | 0x80 | 0x04 | 0x20);
        let nach_id = 2 + 6 + 1 + 1 + 2 + 3; // Kopf, "MQTT", Level, Flags, Keepalive, "c"
        assert_eq!(&pkt[nach_id..], &[0, 3, b'a', b'/', b's', 0, 3, b'o', b'f', b'f', 0, 1, b'u']);
        let ohne_retain = Wille { retain: false, ..w };
        assert_eq!(encode_connect("c", 10, None, None, Some(&ohne_retain))[9], 0x02 | 0x04);
    }

    #[test]
    fn encode_publish_flags_carry_retain_bit() {
        let pkt = encode_publish("t", b"hi", false, 0, 0);
        assert_eq!(pkt[0], PKT_PUBLISH << 4);
        let pkt_retain = encode_publish("t", b"hi", true, 0, 0);
        assert_eq!(pkt_retain[0], (PKT_PUBLISH << 4) | 1);
    }

    #[test]
    fn encode_publish_qos1_mit_paketnummer() {
        // 3.3.1.2/3.3.2.2: QoS in Bits 1-2, Paketnummer hinter dem Topic.
        let pkt = encode_publish("t", b"hi", false, 1, 0x0102);
        assert_eq!(pkt, vec![0x32, 7, 0, 1, b't', 0x01, 0x02, b'h', b'i']);
    }

    #[test]
    fn unsubscribe_und_puback_nach_spec() {
        assert_eq!(encode_unsubscribe(5, "a/b"), vec![0xA2, 7, 0, 5, 0, 3, b'a', b'/', b'b']);
        assert_eq!(encode_puback(0x1234), vec![0x40, 2, 0x12, 0x34]);
        assert_eq!(encode_subscribe(1, "x", 1), vec![0x82, 6, 0, 1, 0, 1, b'x', 1]);
    }

    #[test]
    fn parse_publish_qos1_puback_unsuback() {
        let mut body = encode_str("t");
        body.extend_from_slice(&[0x00, 0x07]);
        body.extend_from_slice(b"x");
        let mut buf = vec![0x32];
        buf.extend_from_slice(&encode_remaining_length(body.len()));
        buf.extend_from_slice(&body);
        buf.extend_from_slice(&[0x40, 2, 0, 9, 0xB0, 2, 0, 3]);
        let (events, rest) = parse_packets(buf);
        assert_eq!(events, vec![
            Event::Publish { topic: "t".into(), payload: "x".into(), qos: 1, packet_id: 7 },
            Event::PubAck(9),
            Event::UnsubAck,
        ]);
        assert!(rest.is_empty());
    }

    #[test]
    fn ziel_mit_und_ohne_tls() {
        assert_eq!(ziel_lesen("mqtts://broker.example.com"), ("broker.example.com".to_string(), true));
        assert_eq!(ziel_lesen("SSL://b/"), ("b".to_string(), true));
        assert_eq!(ziel_lesen("mqtt://b"), ("b".to_string(), false));
        assert_eq!(ziel_lesen("192.168.1.5"), ("192.168.1.5".to_string(), false));
    }

    #[test]
    fn parse_connack_accepted() {
        let buf = vec![0x20, 0x02, 0x00, 0x00];
        let (events, rest) = parse_packets(buf);
        assert_eq!(events, vec![Event::ConnAck { return_code: 0 }]);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_connack_rejected() {
        let buf = vec![0x20, 0x02, 0x00, 0x05]; // "Not authorized"
        let (events, _) = parse_packets(buf);
        assert_eq!(events, vec![Event::ConnAck { return_code: 5 }]);
    }

    #[test]
    fn parse_publish_qos0() {
        let mut buf = vec![0x30]; // PUBLISH, QoS0, kein DUP/RETAIN
        let mut body = encode_str("sensors/temp");
        body.extend_from_slice(b"21.5");
        buf.extend_from_slice(&encode_remaining_length(body.len()));
        buf.extend_from_slice(&body);
        let (events, rest) = parse_packets(buf);
        assert_eq!(events, vec![Event::Publish { topic: "sensors/temp".into(), payload: "21.5".into(), qos: 0, packet_id: 0 }]);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_pingresp_and_suback() {
        let buf = vec![0xD0, 0x00, 0x90, 0x03, 0x00, 0x01, 0x00];
        let (events, rest) = parse_packets(buf);
        assert_eq!(events, vec![Event::PingResp, Event::SubAck]);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_leaves_incomplete_packet_for_next_call() {
        let buf = vec![0x30, 0x05, b'h', b'i']; // Remaining Length 5, aber nur 2 Body-Bytes da
        let (events, rest) = parse_packets(buf.clone());
        assert!(events.is_empty());
        assert_eq!(rest, buf);
    }

    #[test]
    fn parse_message_split_across_two_reads_reassembles() {
        let mut body = encode_str("t");
        body.extend_from_slice(b"payload");
        let mut full = vec![0x30];
        full.extend_from_slice(&encode_remaining_length(body.len()));
        full.extend_from_slice(&body);

        let split_at = full.len() - 3;
        let (events1, rest) = parse_packets(full[..split_at].to_vec());
        assert!(events1.is_empty());
        let mut buf2 = rest;
        buf2.extend_from_slice(&full[split_at..]);
        let (events2, rest2) = parse_packets(buf2);
        assert_eq!(events2, vec![Event::Publish { topic: "t".into(), payload: "payload".into(), qos: 0, packet_id: 0 }]);
        assert!(rest2.is_empty());
    }

    #[test]
    fn subscribe_packet_id_increments_and_wraps() {
        let pkt = encode_subscribe(1, "a/b", 0);
        assert_eq!(&pkt[2..4], &[0x00, 0x01]);
        let pkt2 = encode_subscribe(65535, "a/b", 0);
        assert_eq!(&pkt2[2..4], &[0xFF, 0xFF]);
    }

    /// Ein Broker mit TLS (rustls als Server, Zertifikat aus testdaten/mqtt_tls),
    /// gerade gross genug fuer einen Rundweg: CONNACK, SUBACK, ein PUBLISH mit
    /// QoS 1 bekommt sein PUBACK und geht als QoS 1 mit Nummer 77 zurueck,
    /// UNSUBSCRIBE bekommt UNSUBACK. Der Faden liefert die Paketarten in der
    /// Reihenfolge, in der sie ankamen.
    fn tls_broker() -> (u16, std::thread::JoinHandle<Vec<u8>>) {
        use rustls::pki_types::pem::PemObject;
        use rustls::pki_types::{CertificateDer, PrivateKeyDer};
        use std::sync::Arc;
        let kette: Vec<CertificateDer<'static>> =
            CertificateDer::pem_slice_iter(include_bytes!("../testdaten/mqtt_tls/server.pem"))
                .map(|z| z.unwrap())
                .collect();
        let schluessel = PrivateKeyDer::from_pem_slice(include_bytes!("../testdaten/mqtt_tls/server.key")).unwrap();
        let anbieter = Arc::new(rustls::crypto::ring::default_provider());
        let konfig = rustls::ServerConfig::builder_with_provider(anbieter)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(kette, schluessel)
            .unwrap();
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let faden = std::thread::spawn(move || {
            let mut arten = Vec::new();
            let (tcp, _) = match l.accept() {
                Ok(x) => x,
                Err(_) => return arten,
            };
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let conn = rustls::ServerConnection::new(Arc::new(konfig)).unwrap();
            let mut s = rustls::StreamOwned::new(conn, tcp);
            let mut puf: Vec<u8> = Vec::new();
            let mut chunk = [0u8; 1024];
            loop {
                // Ein ganzes Paket vorne im Puffer?
                if puf.len() >= 2 {
                    if let Some((rest_len, rl)) = decode_remaining_length(&puf[1..]) {
                        let ganz = 1 + rl + rest_len;
                        if puf.len() >= ganz {
                            let p: Vec<u8> = puf.drain(..ganz).collect();
                            let art = p[0] >> 4;
                            arten.push(art);
                            let body = &p[1 + rl..];
                            let antwort: Vec<u8> = match art {
                                PKT_CONNECT => vec![0x20, 2, 0, 0],
                                PKT_SUBSCRIBE => vec![0x90, 3, body[0], body[1], body[body.len() - 1]],
                                PKT_UNSUBSCRIBE => vec![0xB0, 2, body[0], body[1]],
                                PKT_PUBLISH => {
                                    let qos = (p[0] >> 1) & 3;
                                    let (topic, rest) = decode_str(body).unwrap();
                                    let (id, inhalt) = if qos > 0 { (&rest[..2], &rest[2..]) } else { (&rest[..0], rest) };
                                    let mut a = Vec::new();
                                    if qos > 0 {
                                        a.extend_from_slice(&[0x40, 2, id[0], id[1]]);
                                    }
                                    a.extend_from_slice(&encode_publish(&topic, inhalt, false, 1, 77));
                                    a
                                }
                                PKT_DISCONNECT => return arten,
                                _ => Vec::new(),
                            };
                            if !antwort.is_empty() {
                                s.write_all(&antwort).unwrap();
                                s.flush().unwrap();
                            }
                            continue;
                        }
                    }
                }
                match s.read(&mut chunk) {
                    Ok(0) | Err(_) => return arten,
                    Ok(n) => puf.extend_from_slice(&chunk[..n]),
                }
            }
        });
        (port, faden)
    }

    #[test]
    fn tls_rundweg_mit_eigener_ca() {
        let (port, faden) = tls_broker();
        let ca = std::env::temp_dir().join(format!("dh_mqtt_ca_{}.pem", std::process::id()));
        std::fs::write(&ca, include_bytes!("../testdaten/mqtt_tls/ca.pem")).unwrap();
        let mut c = connect("mqtts://127.0.0.1", port as i64, "t", 60, None, None, None, Some(ca.to_str().unwrap()))
            .expect("TLS-Verbindung mit eigener CA");
        let _ = std::fs::remove_file(&ca);
        assert!(subscribe(&mut c, "a/b", 1));
        assert!(publish(&mut c, "a/b", "hallo", false, 1));
        assert_eq!(pending(&c), 1);
        assert!(flush(&mut c, 3000).unwrap(), "PUBACK kam nicht");
        assert_eq!(pending(&c), 0);
        let t = Instant::now();
        while !next_message(&mut c) && t.elapsed() < Duration::from_secs(3) {
            update(&mut c).unwrap();
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(message_topic(&c), "a/b");
        assert_eq!(message_payload(&c), "hallo");
        assert!(unsubscribe(&mut c, "a/b"));
        disconnect(&mut c);
        // CONNECT, SUBSCRIBE, PUBLISH, unser PUBACK fuer Nummer 77, UNSUBSCRIBE, DISCONNECT.
        assert_eq!(faden.join().unwrap(), vec![1, 8, 3, 4, 10, 14]);
    }

    #[test]
    fn tls_ohne_passende_ca_scheitert_im_klartext() {
        let (port, faden) = tls_broker();
        // Ohne eigene CA gelten nur die oeffentlichen Wurzeln -- die Test-CA
        // ist keine davon.
        let fehler = connect("mqtts://127.0.0.1", port as i64, "t", 60, None, None, None, None)
            .err()
            .expect("ein fremdes Zertifikat muss abgelehnt werden");
        assert!(fehler.contains("TLS-Verbindung fehlgeschlagen"), "{}", fehler);
        let _ = faden.join();
    }
}
