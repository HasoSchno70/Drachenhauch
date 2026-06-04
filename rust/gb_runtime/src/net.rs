//! Networking-Modul (NET_*) -- nativer Port von `gamebasic/modules/net.py`
//! via `std::net`. Feature `net` (nur stdlib, kein Crate).
//!
//! NET_LISTENER/NET_SOCKET/NET_UDP sind INTEGER-Handles (Index in VM-Vecs).
//! Default non-blocking (wie Python). Encoding UTF-8.
#![cfg(feature = "net")]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Duration;

pub struct NetSock {
    pub stream: TcpStream,
    pub peer_host: String,
    pub peer_port: i64,
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

pub fn listen(port: i64) -> Result<(TcpListener, i64), String> {
    if !(0..=65535).contains(&port) {
        return Err(format!("NET_TCP_LISTEN: port out of range: {}", port));
    }
    let l = TcpListener::bind(("0.0.0.0", port as u16))
        .map_err(|e| format!("NET_TCP_LISTEN: {}", e))?;
    l.set_nonblocking(true).ok();
    let actual = l.local_addr().map(|a| a.port() as i64).unwrap_or(port);
    Ok((l, actual))
}

pub fn accept(l: &TcpListener) -> Result<Option<NetSock>, String> {
    match l.accept() {
        Ok((stream, addr)) => {
            stream.set_nonblocking(true).ok();
            Ok(Some(NetSock { stream, peer_host: addr.ip().to_string(), peer_port: addr.port() as i64 }))
        }
        Err(ref e) if would_block(e) => Ok(None),
        Err(e) => Err(format!("NET_TCP_ACCEPT: {}", e)),
    }
}

pub fn connect(host: &str, port: i64) -> Result<NetSock, String> {
    if !(0..=65535).contains(&port) {
        return Err(format!("NET_TCP_CONNECT: port out of range: {}", port));
    }
    let addr = (host, port as u16)
        .to_socket_addrs()
        .map_err(|e| format!("NET_TCP_CONNECT: {}", e))?
        .next()
        .ok_or_else(|| format!("NET_TCP_CONNECT: Host '{}' nicht aufloesbar", host))?;
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(|e| format!("NET_TCP_CONNECT: {}", e))?;
    stream.set_nonblocking(true).ok();
    Ok(NetSock { stream, peer_host: host.to_string(), peer_port: port })
}

pub fn send(s: &mut NetSock, text: &str) -> Result<i64, String> {
    s.stream.write(text.as_bytes()).map(|n| n as i64).map_err(|e| format!("NET_SEND: {}", e))
}

pub fn recv(s: &mut NetSock, n: i64) -> Result<String, String> {
    if n <= 0 {
        return Err("NET_RECV: max_bytes muss > 0 sein".into());
    }
    let mut buf = vec![0u8; n as usize];
    match s.stream.read(&mut buf) {
        Ok(got) => Ok(String::from_utf8_lossy(&buf[..got]).into_owned()),
        Err(ref e) if would_block(e) => Ok(String::new()),
        Err(e) => Err(format!("NET_RECV: {}", e)),
    }
}

pub fn udp_bind(port: i64) -> Result<UdpSock, String> {
    if !(0..=65535).contains(&port) {
        return Err(format!("NET_UDP_BIND: port out of range: {}", port));
    }
    let s = UdpSocket::bind(("0.0.0.0", port as u16)).map_err(|e| format!("NET_UDP_BIND: {}", e))?;
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
    s.sock.send_to(text.as_bytes(), (host, port as u16))
        .map(|n| n as i64)
        .map_err(|e| format!("NET_UDP_SEND: {}", e))
}

pub fn udp_recv(s: &mut UdpSock, n: i64) -> Result<String, String> {
    if n <= 0 {
        return Err("NET_UDP_RECV: max_bytes muss > 0 sein".into());
    }
    let mut buf = vec![0u8; n as usize];
    match s.sock.recv_from(&mut buf) {
        Ok((got, addr)) => {
            s.last_from = (addr.ip().to_string(), addr.port() as i64);
            Ok(String::from_utf8_lossy(&buf[..got]).into_owned())
        }
        Err(ref e) if would_block(e) => Ok(String::new()),
        Err(e) => Err(format!("NET_UDP_RECV: {}", e)),
    }
}
