//! TLS fuer die Netz-Module: EIN Strom (Klartext oder TLS) und EIN Aufbau.
//!
//! Bis 2026.53 stand derselbe Code viermal da -- websocket.rs, mqtt.rs,
//! smtp.rs (je ein eigenes `enum Strom` samt Read/Write) und db.rs (die
//! Konfiguration) --, mit kleinen Abweichungen bei Wurzeln und Meldungen
//! (Bericht A15). Der Anbieter wird ausdruecklich genannt statt ueber
//! `ClientConfig::builder()` geraten: waeren einmal zwei Krypto-Anbieter im
//! Baum, liefe der Ratepfad in einen Absturz.

#[cfg(any(feature = "net", feature = "smtp"))]
use std::io::{Read, Write};
#[cfg(any(feature = "net", feature = "smtp"))]
use std::net::TcpStream;
use std::sync::Arc;

#[cfg(any(feature = "net", feature = "smtp"))]
/// Klartext oder TLS -- der Rest des Codes soll den Unterschied nicht sehen.
pub enum Strom {
    Klar(TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
}

#[cfg(any(feature = "net", feature = "smtp"))]
impl Strom {
    /// Der TCP-Strom darunter (Fristen, Abschalten, nicht-blockierend).
    pub fn tcp(&self) -> &TcpStream {
        match self { Strom::Klar(s) => s, Strom::Tls(s) => s.get_ref() }
    }

    /// Auf TLS hochschalten (SMTP STARTTLS).
    pub fn verschluesseln(self, host: &str, fn_: &str) -> Result<Strom, String> {
        match self {
            Strom::Klar(tcp) => strom(tcp, host, None, fn_),
            Strom::Tls(_) => Err(format!("{}die Verbindung ist bereits verschluesselt", praefix(fn_))),
        }
    }
}

#[cfg(any(feature = "net", feature = "smtp"))]
impl Read for Strom {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        match self { Strom::Klar(s) => s.read(b), Strom::Tls(s) => s.read(b) }
    }
}

#[cfg(any(feature = "net", feature = "smtp"))]
impl Write for Strom {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        match self { Strom::Klar(s) => s.write(b), Strom::Tls(s) => s.write(b) }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self { Strom::Klar(s) => s.flush(), Strom::Tls(s) => s.flush() }
    }
}

fn praefix(fn_: &str) -> String {
    if fn_.is_empty() { String::new() } else { format!("{}: ", fn_) }
}

/// Die Wurzeln, denen TLS glaubt: ohne eigene CA die von webpki-roots, mit
/// `ca` NUR die aus dieser PEM-Datei (ein Broker mit selbst ausgestelltem
/// Zertifikat).
pub fn wurzeln(ca: Option<&str>, fn_: &str) -> Result<rustls::RootCertStore, String> {
    let mut w = rustls::RootCertStore::empty();
    match ca {
        None => w.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned()),
        Some(pfad) => {
            use rustls::pki_types::pem::PemObject;
            let p = praefix(fn_);
            let daten = std::fs::read(pfad).map_err(|e| format!("{}CA-Datei {}: {}", p, pfad, e))?;
            let mut n = 0;
            for z in rustls::pki_types::CertificateDer::pem_slice_iter(&daten) {
                let z = z.map_err(|e| format!("{}CA-Datei {}: {:?}", p, pfad, e))?;
                w.add(z).map_err(|e| format!("{}CA-Datei {}: {}", p, pfad, e))?;
                n += 1;
            }
            if n == 0 {
                return Err(format!("{}in der CA-Datei {} steht kein Zertifikat (erwartet PEM, -----BEGIN CERTIFICATE-----)", p, pfad));
            }
        }
    }
    Ok(w)
}

/// Die Konfiguration eines Clients mit diesen Wurzeln.
pub fn konfig(w: rustls::RootCertStore, fn_: &str) -> Result<rustls::ClientConfig, String> {
    Ok(rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("{}TLS liess sich nicht einrichten: {}", praefix(fn_), e))?
        .with_root_certificates(w)
        .with_no_client_auth())
}

#[cfg(any(feature = "net", feature = "smtp"))]
/// Einen TCP-Strom in TLS packen (Servername `name`, Wurzeln wie `wurzeln`).
pub fn strom(tcp: TcpStream, name: &str, ca: Option<&str>, fn_: &str) -> Result<Strom, String> {
    let k = konfig(wurzeln(ca, fn_)?, fn_)?;
    let p = praefix(fn_);
    let sn = rustls::pki_types::ServerName::try_from(name.to_string())
        .map_err(|_| format!("{}{:?} ist kein gueltiger Servername fuer TLS", p, name))?;
    let v = rustls::ClientConnection::new(Arc::new(k), sn)
        .map_err(|e| format!("{}TLS-Verbindung fehlgeschlagen: {}", p, e))?;
    Ok(Strom::Tls(Box::new(rustls::StreamOwned::new(v, tcp))))
}
