//! Eine TCP-Verbindung zu `host:port` -- EIN Weg fuer NET, WebSocket, MQTT,
//! SMTP und Modbus.
//!
//! Bis 2026.53 nahmen NET, WS, MQTT und SMTP nur die ERSTE Adresse eines
//! Namens. `localhost` liefert unter Windows zuerst `::1`; ein Dienst, der nur
//! auf IPv4 lauscht (NET_TCP_LISTEN, HTTPD_START, der eigene Mosquitto),
//! war dann "nicht erreichbar", obwohl er lief. Modbus, die Datenbank und
//! HTTP probierten schon alle -- jetzt tun es alle.

use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::time::Duration;

/// Wie lange das Aufloesen eines Namens hoechstens dauern darf.
const DNS_FRIST: Duration = Duration::from_secs(5);

/// Alle Adressen eines Namens. Eine IP braucht keinen Resolver; ein Name
/// wird in einem eigenen Faden mit Frist aufgeloest (der Resolver des
/// Systems laesst sich nicht abbrechen); ein Name auf `.local`, den das
/// System nicht kennt (Linux ohne nss-mdns), wird selbst ueber mDNS gefragt.
pub fn adressen(host: &str, port: u16, fn_: &str) -> Result<Vec<SocketAddr>, String> {
    if let Ok(ip) = host.trim_start_matches('[').trim_end_matches(']').parse::<std::net::IpAddr>() {
        return Ok(vec![SocketAddr::new(ip, port)]);
    }
    let name = host.to_string();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let r = (name.as_str(), port).to_socket_addrs().map(|it| it.collect::<Vec<_>>());
        let _ = tx.send(r);
    });
    let gefunden = match rx.recv_timeout(DNS_FRIST) {
        Ok(Ok(v)) => v,
        Ok(Err(_)) => Vec::new(),
        Err(_) => return Err(format!("{}: das Aufloesen von '{}' hat laenger als {} s gedauert",
                                     fn_, host, DNS_FRIST.as_secs())),
    };
    if !gefunden.is_empty() { return Ok(gefunden); }
    if host.to_ascii_lowercase().ends_with(".local") {
        if let Ok(ip) = crate::mdns::aufloesen(host, 1500) {
            if let Ok(ip) = ip.parse::<std::net::IpAddr>() { return Ok(vec![SocketAddr::new(ip, port)]); }
        }
        return Err(format!("{}: Host '{}' nicht aufloesbar (auch nicht ueber mDNS)", fn_, host));
    }
    Err(format!("{}: Host '{}' nicht aufloesbar", fn_, host))
}

/// Mit `host:port` verbinden: jede Adresse des Namens der Reihe nach, je mit
/// `frist`; die erste, die antwortet, gewinnt. Scheitern alle, nennt die
/// Meldung Ziel und den letzten Grund.
pub fn tcp(host: &str, port: u16, frist: Duration, fn_: &str) -> Result<TcpStream, String> {
    let mut letzter = String::new();
    for a in adressen(host, port, fn_)? {
        match TcpStream::connect_timeout(&a, frist) {
            Ok(s) => { let _ = s.set_nodelay(true); return Ok(s); }
            Err(e) => letzter = e.to_string(),
        }
    }
    Err(format!("{}: keine Verbindung zu {}:{} -- {}", fn_, host, port, letzter))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ein Dienst nur auf IPv4 ist ueber `localhost` erreichbar, auch wenn
    /// der Name zuerst `::1` liefert.
    #[test]
    fn localhost_findet_einen_dienst_nur_auf_ipv4() {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(tcp("localhost", port, Duration::from_secs(2), "PROBE").is_ok());
    }

    #[test]
    fn ein_name_den_es_nicht_gibt() {
        let e = tcp("gibt-es-nicht.invalid", 80, Duration::from_millis(200), "PROBE").unwrap_err();
        assert!(e.starts_with("PROBE: "), "{}", e);
    }
}
