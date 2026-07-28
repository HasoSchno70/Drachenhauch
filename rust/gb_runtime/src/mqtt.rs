//! MQTT-Client (MQTT_*) fuer ESP32/IoT-Steuerung -- das im Maker-/IoT-Bereich
//! dominante Pub/Sub-Protokoll, direkt gegen die OASIS-MQTT-3.1.1-Spezifikation
//! implementiert (nur `std::net`, Feature `net`, keine neue Abhaengigkeit).
//!
//! Bewusst NICHT abgedeckt (siehe docs/module-mqtt.md): nur **QoS 0**
//! (Publish/Subscribe ohne Ack-Handshake -- QoS 1/2 braeuchten Packet-ID-
//! Tracking + Retry-State-Machine), kein UNSUBSCRIBE, keine Will-Message,
//! kein TLS (mqtts://). Fuer die typische Bastler-Nutzung (Sensor-Werte
//! publizieren, auf Steuer-Topics subscriben gegen einen lokalen Broker wie
//! Mosquitto) reicht QoS 0 vollstaendig.
#![cfg(feature = "net")]

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

const PKT_CONNECT: u8 = 1;
const PKT_CONNACK: u8 = 2;
const PKT_PUBLISH: u8 = 3;
const PKT_SUBSCRIBE: u8 = 8;
const PKT_SUBACK: u8 = 9;
const PKT_PINGREQ: u8 = 12;
const PKT_PINGRESP: u8 = 13;
const PKT_DISCONNECT: u8 = 14;

pub struct Client {
    stream: TcpStream,
    connected: bool,
    keepalive: Duration,
    last_send: Instant,
    next_packet_id: u16,
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

fn encode_connect(client_id: &str, keepalive_s: u16, username: Option<&str>, password: Option<&str>) -> Vec<u8> {
    let mut body = encode_str("MQTT");
    body.push(0x04); // Protocol Level: MQTT 3.1.1
    let mut flags: u8 = 0x02; // Clean Session
    if username.is_some() { flags |= 0x80; }
    if password.is_some() { flags |= 0x40; }
    body.push(flags);
    body.extend_from_slice(&keepalive_s.to_be_bytes());
    body.extend_from_slice(&encode_str(client_id));
    if let Some(u) = username { body.extend_from_slice(&encode_str(u)); }
    if let Some(p) = password { body.extend_from_slice(&encode_str(p)); }
    wrap_packet(PKT_CONNECT << 4, &body)
}

fn encode_publish(topic: &str, payload: &[u8], retain: bool) -> Vec<u8> {
    let flags = (PKT_PUBLISH << 4) | if retain { 1 } else { 0 }; // QoS 0, DUP 0
    let mut body = encode_str(topic);
    body.extend_from_slice(payload);
    wrap_packet(flags, &body)
}

fn encode_subscribe(packet_id: u16, topic: &str) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&packet_id.to_be_bytes());
    body.extend_from_slice(&encode_str(topic));
    body.push(0x00); // angefragtes QoS 0
    wrap_packet((PKT_SUBSCRIBE << 4) | 0x02, &body) // Flags 0010 sind in der Spec Pflicht
}

#[derive(Debug, PartialEq)]
enum Event {
    ConnAck { return_code: u8 },
    Publish { topic: String, payload: String },
    PingResp,
    SubAck,
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
                    PKT_PUBLISH => decode_str(body).map(|(topic, rest)| {
                        let qos = (flags >> 1) & 0x03;
                        // QoS>0 haette hier eine 2-Byte-Packet-ID -- wird
                        // uebersprungen (nur QoS 0 wird unterstuetzt, siehe
                        // Modul-Kommentar oben).
                        let payload_bytes = if qos > 0 && rest.len() >= 2 { &rest[2..] } else { rest };
                        Event::Publish { topic, payload: String::from_utf8_lossy(payload_bytes).into_owned() }
                    }),
                    PKT_PINGRESP => Some(Event::PingResp),
                    PKT_SUBACK => Some(Event::SubAck),
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

#[allow(clippy::too_many_arguments)]
pub fn connect(
    host: &str,
    port: i64,
    client_id: &str,
    keepalive_s: i64,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<Client, String> {
    if !(0..=65535).contains(&port) {
        return Err(format!("MQTT_CONNECT: Port {} ausserhalb 0..65535", port));
    }
    let addr_str = format!("{}:{}", host, port);
    let sockaddr = addr_str
        .to_socket_addrs()
        .map_err(|e| format!("MQTT_CONNECT: {}", e))?
        .next()
        .ok_or_else(|| format!("MQTT_CONNECT: Adresse {} nicht aufloesbar", addr_str))?;
    let stream = TcpStream::connect_timeout(&sockaddr, Duration::from_secs(5))
        .map_err(|e| format!("MQTT_CONNECT: {}", e))?;
    // Kurzer Timeout: MQTT_UPDATE wird pro Frame gepollt (wie FIRMATA_UPDATE/
    // INPUT_UPDATE), darf also nicht spuerbar blockieren.
    stream.set_read_timeout(Some(Duration::from_millis(1))).ok();

    let ka = keepalive_s.clamp(1, 65535) as u16;
    let mut client = Client {
        stream,
        connected: false,
        keepalive: Duration::from_secs(ka as u64),
        last_send: Instant::now(),
        next_packet_id: 0,
        rx_pending: Vec::new(),
        incoming: VecDeque::new(),
        current: None,
    };

    let pkt = encode_connect(client_id, ka, username, password);
    client.stream.write_all(&pkt).map_err(|e| format!("MQTT_CONNECT: Schreiben fehlgeschlagen: {}", e))?;
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
                        return Ok(client);
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut || e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(format!("MQTT_CONNECT: Lesefehler: {}", e)),
        }
    }
}

pub fn is_connected(c: &Client) -> bool { c.connected }

pub fn publish(c: &mut Client, topic: &str, payload: &str, retain: bool) -> Result<(), String> {
    if !c.connected { return Err("MQTT_PUBLISH: nicht verbunden".into()); }
    let pkt = encode_publish(topic, payload.as_bytes(), retain);
    c.stream.write_all(&pkt).map_err(|e| format!("MQTT_PUBLISH: Schreiben fehlgeschlagen: {}", e))?;
    c.last_send = Instant::now();
    Ok(())
}

pub fn subscribe(c: &mut Client, topic: &str) -> Result<(), String> {
    if !c.connected { return Err("MQTT_SUBSCRIBE: nicht verbunden".into()); }
    c.next_packet_id = if c.next_packet_id == u16::MAX { 1 } else { c.next_packet_id + 1 };
    let pkt = encode_subscribe(c.next_packet_id, topic);
    c.stream.write_all(&pkt).map_err(|e| format!("MQTT_SUBSCRIBE: Schreiben fehlgeschlagen: {}", e))?;
    c.last_send = Instant::now();
    Ok(())
}

pub fn disconnect(c: &mut Client) {
    if c.connected {
        let _ = c.stream.write_all(&wrap_packet(PKT_DISCONNECT << 4, &[]));
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
        let _ = c.stream.write_all(&wrap_packet(PKT_PINGREQ << 4, &[]));
        c.last_send = Instant::now();
    }
    let mut chunk = [0u8; 4096];
    let n = match c.stream.read(&mut chunk) {
        Ok(0) => { c.connected = false; return Ok(()); } // Broker hat sauber geschlossen
        Ok(n) => n,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut || e.kind() == std::io::ErrorKind::WouldBlock => 0,
        Err(e) => { c.connected = false; return Err(format!("MQTT_UPDATE: {}", e)); }
    };
    if n == 0 { return Ok(()); }
    let mut pending = std::mem::take(&mut c.rx_pending);
    pending.extend_from_slice(&chunk[..n]);
    let (events, rest) = parse_packets(pending);
    c.rx_pending = rest;
    for ev in events {
        if let Event::Publish { topic, payload } = ev {
            c.incoming.push_back((topic, payload));
        }
    }
    Ok(())
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
        let pkt = encode_connect("test", 60, None, None);
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
        let pkt = encode_connect("c", 10, Some("u"), Some("p"));
        let flags_byte = pkt[9]; // Index: 0x10,len, "MQTT"(6), level(1) -> Flags bei Index 9
        assert_eq!(flags_byte, 0x02 | 0x80 | 0x40);
    }

    #[test]
    fn encode_publish_flags_carry_retain_bit() {
        let pkt = encode_publish("t", b"hi", false);
        assert_eq!(pkt[0], PKT_PUBLISH << 4);
        let pkt_retain = encode_publish("t", b"hi", true);
        assert_eq!(pkt_retain[0], (PKT_PUBLISH << 4) | 1);
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
        assert_eq!(events, vec![Event::Publish { topic: "sensors/temp".into(), payload: "21.5".into() }]);
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
        assert_eq!(events2, vec![Event::Publish { topic: "t".into(), payload: "payload".into() }]);
        assert!(rest2.is_empty());
    }

    #[test]
    fn subscribe_packet_id_increments_and_wraps() {
        let pkt = encode_subscribe(1, "a/b");
        assert_eq!(&pkt[2..4], &[0x00, 0x01]);
        let pkt2 = encode_subscribe(65535, "a/b");
        assert_eq!(&pkt2[2..4], &[0xFF, 0xFF]);
    }
}
