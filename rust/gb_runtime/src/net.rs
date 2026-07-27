//! Networking-Modul (NET_*) -- nativer Port von `gamebasic/modules/net.py`
//! via `std::net`. Feature `net` (nur stdlib, kein Crate).
//!
//! NET_LISTENER/NET_SOCKET/NET_UDP sind INTEGER-Handles (Index in VM-Vecs).
//! Default non-blocking (wie Python). Encoding UTF-8.
#![cfg(feature = "net")]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::mpsc;
use std::time::Duration;

/// Obergrenze fuer NET_RECV/NET_UDP_RECV(sock, n) -- ohne Cap loest ein
/// versehentlich riesiges `n` (Tippfehler, vertauschte Parameter) eine
/// Multi-GB-Allokation aus, die bei Fehlschlag den Prozess abbricht statt
/// einen fangbaren Fehler zu liefern.
const MAX_READ_BYTES: i64 = 64 * 1024 * 1024;

pub struct NetSock {
    pub stream: TcpStream,
    pub peer_host: String,
    pub peer_port: i64,
    /// Von der Gegenseite sauber geschlossen (read() == 0) oder ein Recv/Send
    /// ist mit einem echten Fehler (nicht WouldBlock) fehlgeschlagen.
    pub connected: bool,
    /// Unvollstaendiger Rest eines Mehrbyte-UTF-8-Zeichens vom letzten RECV,
    /// das quer ueber zwei TCP-Reads verteilt ankam.
    pending: Vec<u8>,
}

impl NetSock {
    fn decode_chunk(&mut self, bytes: &[u8]) -> String {
        let mut data = std::mem::take(&mut self.pending);
        data.extend_from_slice(bytes);
        let (text, leftover) = crate::text_stream::decode_utf8_streaming(data);
        self.pending = leftover;
        text
    }
}

pub struct UdpSock {
    pub sock: UdpSocket,
    pub bound_port: i64,
    pub last_from: (String, i64),
}

fn would_block(e: &std::io::Error) -> bool {
    matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)
}

pub fn set_timeout_tcp(s: &TcpStream, ms: i64) {
    if ms == 0 {
        let _ = s.set_nonblocking(true);
    } else if ms < 0 {
        let _ = s.set_nonblocking(false);
        let _ = s.set_read_timeout(None);
        let _ = s.set_write_timeout(None);
    } else {
        let _ = s.set_nonblocking(false);
        let d = Some(Duration::from_millis(ms as u64));
        let _ = s.set_read_timeout(d);
        let _ = s.set_write_timeout(d);
    }
}

pub fn set_timeout_udp(s: &UdpSocket, ms: i64) {
    if ms == 0 {
        let _ = s.set_nonblocking(true);
    } else if ms < 0 {
        let _ = s.set_nonblocking(false);
        let _ = s.set_read_timeout(None);
    } else {
        let _ = s.set_nonblocking(false);
        let _ = s.set_read_timeout(Some(Duration::from_millis(ms as u64)));
    }
}

/// Bind-Adresse aus dem optionalen `bind_addr`-Arg: leer -> alle IPv4-Interfaces
/// (bisheriges Default-Verhalten, unveraendert). `"::"` bindet einen reinen
/// IPv6-Listener/-Socket (z.B. fuer IPv6-Only-Netze) -- fuer "sowohl v4 als
/// auch v6" zwei Listener/Sockets auf demselben Port oeffnen (einen mit "",
/// einen mit "::"), echtes Dual-Stack-Binding braucht plattformspezifisches
/// IPV6_V6ONLY-Tuning und wird hier bewusst nicht per unsafe FFI nachgebaut.
fn resolve_bind_host(bind_addr: &str) -> &str {
    if bind_addr.is_empty() {
        "0.0.0.0"
    } else {
        bind_addr
    }
}

pub fn listen(port: i64, bind_addr: &str) -> Result<(TcpListener, i64), String> {
    if !(0..=65535).contains(&port) {
        return Err(format!("NET_TCP_LISTEN: port out of range: {}", port));
    }
    let host = resolve_bind_host(bind_addr);
    let l = TcpListener::bind((host, port as u16))
        .map_err(|e| format!("NET_TCP_LISTEN: {}", e))?;
    l.set_nonblocking(true).ok();
    let actual = l.local_addr().map(|a| a.port() as i64).unwrap_or(port);
    Ok((l, actual))
}

pub fn accept(l: &TcpListener) -> Result<Option<NetSock>, String> {
    match l.accept() {
        Ok((stream, addr)) => {
            stream.set_nonblocking(true).ok();
            Ok(Some(NetSock {
                stream,
                peer_host: addr.ip().to_string(),
                peer_port: addr.port() as i64,
                connected: true,
                pending: Vec::new(),
            }))
        }
        Err(ref e) if would_block(e) => Ok(None),
        Err(e) => Err(format!("NET_TCP_ACCEPT: {}", e)),
    }
}

/// Loest `host` mit einem eigenen Timeout auf (statt der unbegrenzt blockierenden
/// stdlib-DNS-Aufloesung), damit ein haengender/langsamer DNS-Server
/// `NET_TCP_CONNECT` nicht den Game-Loop einfrieren lassen kann.
fn resolve_with_timeout(fn_: &str, host: &str, port: i64, timeout: Duration) -> Result<std::net::SocketAddr, String> {
    let host = host.to_string();
    let host_for_thread = host.clone();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = (host_for_thread.as_str(), port as u16)
            .to_socket_addrs()
            .map(|mut it| it.next())
            .map_err(|e| e.to_string());
        let _ = tx.send(result);
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(Some(addr))) => Ok(addr),
        Ok(Ok(None)) => Err(format!("{}: Host '{}' nicht aufloesbar", fn_, host)),
        Ok(Err(e)) => Err(format!("{}: {}", fn_, e)),
        Err(_) => Err(format!("{}: DNS-Aufloesung von '{}' hat laenger als {}ms gedauert", fn_, host, timeout.as_millis())),
    }
    // Hinweis: der Resolver-Thread laeuft im Timeout-Fall im Hintergrund weiter
    // zu Ende (blockierender OS-Resolver-Call laesst sich nicht abbrechen) und
    // stirbt beim Send in den dann verwaisten Channel -- kein Leak, nur ein
    // kurzlebiger Daemon-Thread.
}

pub fn connect(host: &str, port: i64) -> Result<NetSock, String> {
    if !(0..=65535).contains(&port) {
        return Err(format!("NET_TCP_CONNECT: port out of range: {}", port));
    }
    let addr = resolve_with_timeout("NET_TCP_CONNECT", host, port, Duration::from_secs(5))?;
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(|e| format!("NET_TCP_CONNECT: {}", e))?;
    stream.set_nonblocking(true).ok();
    Ok(NetSock {
        stream,
        peer_host: host.to_string(),
        peer_port: port,
        connected: true,
        pending: Vec::new(),
    })
}

pub fn send(s: &mut NetSock, text: &str) -> Result<i64, String> {
    match s.stream.write(text.as_bytes()) {
        Ok(n) => Ok(n as i64),
        Err(ref e) if would_block(e) => Ok(0),
        Err(e) => {
            s.connected = false;
            Err(format!("NET_SEND: {}", e))
        }
    }
}

pub fn recv(s: &mut NetSock, n: i64) -> Result<String, String> {
    if n <= 0 {
        return Err("NET_RECV: max_bytes muss > 0 sein".into());
    }
    if n > MAX_READ_BYTES {
        return Err(format!("NET_RECV: max_bytes {} ueberschreitet das Limit von {} Byte", n, MAX_READ_BYTES));
    }
    let mut buf = vec![0u8; n as usize];
    match s.stream.read(&mut buf) {
        Ok(0) => {
            // Gegenseite hat die Verbindung sauber geschlossen (TCP-FIN).
            s.connected = false;
            Ok(String::new())
        }
        Ok(got) => Ok(s.decode_chunk(&buf[..got])),
        Err(ref e) if would_block(e) => Ok(String::new()),
        Err(e) => {
            s.connected = false;
            Err(format!("NET_RECV: {}", e))
        }
    }
}

pub fn is_connected(s: &NetSock) -> bool {
    s.connected
}

pub fn udp_bind(port: i64, bind_addr: &str) -> Result<UdpSock, String> {
    if !(0..=65535).contains(&port) {
        return Err(format!("NET_UDP_BIND: port out of range: {}", port));
    }
    let host = resolve_bind_host(bind_addr);
    let s = UdpSocket::bind((host, port as u16)).map_err(|e| format!("NET_UDP_BIND: {}", e))?;
    s.set_nonblocking(true).ok();
    let actual = s.local_addr().map(|a| a.port() as i64).unwrap_or(port);
    Ok(UdpSock { sock: s, bound_port: actual, last_from: (String::new(), 0) })
}

pub fn udp_open() -> Result<UdpSock, String> {
    let s = UdpSocket::bind(("0.0.0.0", 0)).map_err(|e| format!("NET_UDP_OPEN: {}", e))?;
    s.set_nonblocking(true).ok();
    Ok(UdpSock { sock: s, bound_port: 0, last_from: (String::new(), 0) })
}

pub fn udp_send(s: &UdpSock, host: &str, port: i64, text: &str) -> Result<i64, String> {
    // Review-Fund: `(host, port).to_socket_addrs()` (versteckt hinter dem
    // Tupel-ToSocketAddrs-Impl, den send_to hier nutzt) macht einen
    // SYNCHRONEN, nicht abbrechbaren OS-Resolver-Aufruf VOR jedem einzelnen
    // Datagramm -- genau das, wofuer NET_TCP_CONNECT extra
    // resolve_with_timeout() bekommen hat ("ein haengender/langsamer DNS-
    // Server soll NET_TCP_CONNECT nicht den Game-Loop einfrieren lassen").
    // Bei einer hostname-basierten UDP-Position-Sync (dokumentiertes 30-Hz-
    // Beispiel) haette ein antwortloser DNS-Server das Spiel bei JEDEM
    // Frame kurz eingefroren. Port wurde ausserdem gar nicht validiert
    // (NET_UDP_SEND(s,"host",70000,"x") sendete lautlos an Port 4464).
    if !(0..=65535).contains(&port) {
        return Err(format!("NET_UDP_SEND: port out of range: {}", port));
    }
    let addr = resolve_with_timeout("NET_UDP_SEND", host, port, Duration::from_secs(2))?;
    s.sock.send_to(text.as_bytes(), addr)
        .map(|n| n as i64)
        .map_err(|e| format!("NET_UDP_SEND: {}", e))
}

pub fn udp_recv(s: &mut UdpSock, n: i64) -> Result<String, String> {
    if n <= 0 {
        return Err("NET_UDP_RECV: max_bytes muss > 0 sein".into());
    }
    if n > MAX_READ_BYTES {
        return Err(format!("NET_UDP_RECV: max_bytes {} ueberschreitet das Limit von {} Byte", n, MAX_READ_BYTES));
    }
    let mut buf = vec![0u8; n as usize];
    match s.sock.recv_from(&mut buf) {
        Ok((got, addr)) => {
            s.last_from = (addr.ip().to_string(), addr.port() as i64);
            // Ein UDP-Datagramm kommt immer komplett-oder-gar-nicht an -- der
            // einzige Fall fuer einen unvollstaendigen Zeichen-Rest ist ein zu
            // kleiner `max_bytes`-Puffer, der das Datagramm abschneidet. Dieser
            // Rest wird verworfen statt in ein naechstes (unabhaengiges!)
            // Datagramm hineingemischt zu werden.
            let (text, _leftover) = crate::text_stream::decode_utf8_streaming(buf[..got].to_vec());
            Ok(text)
        }
        Err(ref e) if would_block(e) => Ok(String::new()),
        Err(e) => Err(format!("NET_UDP_RECV: {}", e)),
    }
}
