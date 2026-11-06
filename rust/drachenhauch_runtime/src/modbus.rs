//! Modul `modbus` -- Geraete der Gebaeude- und Industrietechnik lesen und
//! schreiben: Stromzaehler, Wechselrichter, Heizungsregler, Frequenzumrichter,
//! SPS. Modbus ist das Protokoll, das sie fast alle sprechen.
//!
//! **Zwei Rahmen, drei Leitungen.** Modbus TCP (`MODBUS_TCP`, Port 502) legt
//! vor jede Anfrage einen Kopf mit Kennung und Laenge (MBAP). Modbus RTU
//! (`MODBUS_RTU`) schickt dieselbe Anfrage mit Geraeteadresse vorn und einer
//! Pruefsumme (CRC-16) hinten ueber RS-485 -- seriell mit dem Feature
//! `serial`, oder als `"tcp:host:port"` ueber einen RTU-Umsetzer im Netz
//! ("RTU over TCP"). Der Netzweg braucht kein Feature, darum steht das Modul
//! in JEDEM Bau, und so laesst sich auch RTU ohne Geraet pruefen.
//!
//! **Alles wartet auf die Antwort.** Modbus ist Frage und Antwort, ein Geraet
//! antwortet in Millisekunden; die Befehle kehren erst mit dem Ergebnis (oder
//! der Zeitgrenze, Vorgabe 1 s) zurueck. Die Rahmen bauen und lesen reine
//! Funktionen (`crc16`, `pdu_*`, `antwort_*`) -- mit Rust-Tests gegen die
//! Beispiele der Spezifikation.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

// ------------------------------------------------------------ reine Rechnerei

/// CRC-16 nach Modbus (Polynom 0xA001 gespiegelt, Anfang 0xFFFF). Im Rahmen
/// steht das niedrige Byte zuerst.
pub fn crc16(daten: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in daten {
        crc ^= b as u16;
        for _ in 0..8 {
            if crc & 1 != 0 { crc = (crc >> 1) ^ 0xA001; } else { crc >>= 1; }
        }
    }
    crc
}

/// Lesen: Funktion 1 (Spulen), 2 (Eingaenge), 3 (Halteregister), 4
/// (Eingangsregister) -- Adresse und Anzahl, beide 16 Bit, hohes Byte zuerst.
pub fn pdu_lesen(fc: u8, adr: u16, anzahl: u16) -> Vec<u8> {
    vec![fc, (adr >> 8) as u8, adr as u8, (anzahl >> 8) as u8, anzahl as u8]
}

/// Funktion 5: eine Spule -- an ist 0xFF00, aus 0x0000 (nichts anderes).
pub fn pdu_spule(adr: u16, an: bool) -> Vec<u8> {
    vec![5, (adr >> 8) as u8, adr as u8, if an { 0xFF } else { 0 }, 0]
}

/// Funktion 6: ein Halteregister.
pub fn pdu_register(adr: u16, wert: u16) -> Vec<u8> {
    vec![6, (adr >> 8) as u8, adr as u8, (wert >> 8) as u8, wert as u8]
}

/// Funktion 15: mehrere Spulen, acht je Byte, die erste im niedrigsten Bit.
pub fn pdu_spulen(adr: u16, werte: &[bool]) -> Vec<u8> {
    let n = werte.len();
    let bytes = n.div_ceil(8);
    let mut p = vec![15, (adr >> 8) as u8, adr as u8, (n >> 8) as u8, n as u8, bytes as u8];
    for k in 0..bytes {
        let mut b = 0u8;
        for bit in 0..8 {
            if werte.get(k * 8 + bit).copied().unwrap_or(false) { b |= 1 << bit; }
        }
        p.push(b);
    }
    p
}

/// Funktion 16: mehrere Halteregister.
pub fn pdu_registers(adr: u16, werte: &[u16]) -> Vec<u8> {
    let n = werte.len();
    let mut p = vec![16, (adr >> 8) as u8, adr as u8, (n >> 8) as u8, n as u8, (n * 2) as u8];
    for &w in werte { p.push((w >> 8) as u8); p.push(w as u8); }
    p
}

/// Modbus TCP: Kopf (Kennung, Protokoll 0, Laenge = Einheit + PDU, Einheit).
pub fn tcp_rahmen(kennung: u16, einheit: u8, pdu: &[u8]) -> Vec<u8> {
    let l = pdu.len() + 1;
    let mut r = vec![(kennung >> 8) as u8, kennung as u8, 0, 0, (l >> 8) as u8, l as u8, einheit];
    r.extend_from_slice(pdu);
    r
}

/// Modbus RTU: Adresse, PDU, Pruefsumme (niedriges Byte zuerst).
pub fn rtu_rahmen(einheit: u8, pdu: &[u8]) -> Vec<u8> {
    let mut r = Vec::with_capacity(pdu.len() + 3);
    r.push(einheit);
    r.extend_from_slice(pdu);
    let c = crc16(&r);
    r.push(c as u8);
    r.push((c >> 8) as u8);
    r
}

/// Wie lang ein RTU-Antwortrahmen insgesamt ist, aus seinen ersten drei
/// Bytes (Adresse, Funktion, und bei Lese-Antworten die Bytezahl). RTU hat
/// keine Laenge im Rahmen -- das Ende erkennt man nur so.
pub fn rtu_laenge(anfang: &[u8]) -> Option<usize> {
    let fc = *anfang.get(1)?;
    if fc & 0x80 != 0 { return Some(5); }
    match fc {
        1..=4 => Some(3 + *anfang.get(2)? as usize + 2),
        5 | 6 | 15 | 16 => Some(8),
        _ => None,
    }
}

pub fn ausnahme_text(code: u8) -> &'static str {
    match code {
        1 => "unerlaubte Funktion",
        2 => "unerlaubte Datenadresse",
        3 => "unerlaubter Datenwert",
        4 => "Fehler im Geraet",
        5 => "angenommen, dauert noch",
        6 => "Geraet beschaeftigt",
        8 => "Paritaetsfehler im Speicher",
        10 => "Gateway: kein Weg zum Ziel",
        11 => "Gateway: Zielgeraet antwortet nicht",
        _ => "unbekannte Ausnahme",
    }
}

/// Die PDU einer Antwort pruefen: dieselbe Funktion wie gefragt, sonst eine
/// Ausnahme (Funktion + 0x80, dann ein Code).
pub fn antwort_pruefen(fc: u8, pdu: &[u8], fn_: &str) -> Result<(), String> {
    let f = *pdu.first().ok_or_else(|| format!("{}: leere Antwort", fn_))?;
    if f == fc | 0x80 {
        let code = pdu.get(1).copied().unwrap_or(0);
        return Err(format!("{}: das Geraet meldet Ausnahme {} ({})", fn_, code, ausnahme_text(code)));
    }
    if f != fc {
        return Err(format!("{}: Antwort auf Funktion {} statt {}", fn_, f, fc));
    }
    Ok(())
}

/// Bits aus einer Lese-Antwort (Funktion 1/2).
pub fn antwort_bits(pdu: &[u8], anzahl: usize, fn_: &str) -> Result<Vec<bool>, String> {
    let n = *pdu.get(1).ok_or_else(|| format!("{}: Antwort zu kurz", fn_))? as usize;
    if n != anzahl.div_ceil(8) || pdu.len() < 2 + n {
        return Err(format!("{}: Antwort mit {} Bytes, erwartet {}", fn_, n, anzahl.div_ceil(8)));
    }
    Ok((0..anzahl).map(|k| pdu[2 + k / 8] & (1 << (k % 8)) != 0).collect())
}

/// Register aus einer Lese-Antwort (Funktion 3/4).
pub fn antwort_register(pdu: &[u8], anzahl: usize, fn_: &str) -> Result<Vec<u16>, String> {
    let n = *pdu.get(1).ok_or_else(|| format!("{}: Antwort zu kurz", fn_))? as usize;
    if n != anzahl * 2 || pdu.len() < 2 + n {
        return Err(format!("{}: Antwort mit {} Bytes, erwartet {}", fn_, n, anzahl * 2));
    }
    Ok((0..anzahl).map(|k| ((pdu[2 + 2 * k] as u16) << 8) | pdu[3 + 2 * k] as u16).collect())
}

/// Wortfolge fuer 32-Bit-Werte in zwei Registern. Die Spezifikation legt sie
/// NICHT fest -- jeder Hersteller waehlt selbst, das Datenblatt sagt welche.
/// "ABCD" = hohes Wort zuerst, hohes Byte zuerst (die haeufigste).
pub fn folge_lesen(f: &str, fn_: &str) -> Result<[usize; 4], String> {
    match f.to_ascii_uppercase().as_str() {
        "ABCD" => Ok([0, 1, 2, 3]),
        "CDAB" => Ok([2, 3, 0, 1]),
        "BADC" => Ok([1, 0, 3, 2]),
        "DCBA" => Ok([3, 2, 1, 0]),
        _ => Err(format!("{}: Wortfolge {:?} -- erlaubt sind \"ABCD\", \"CDAB\", \"BADC\", \"DCBA\"", fn_, f)),
    }
}

/// Zwei Register -> vier Bytes in der Reihenfolge A B C D (A = hoechstes).
pub fn zwei_register(r0: u16, r1: u16, folge: [usize; 4]) -> [u8; 4] {
    let roh = [(r0 >> 8) as u8, r0 as u8, (r1 >> 8) as u8, r1 as u8];
    let mut abcd = [0u8; 4];
    // roh[k] traegt das Byte folge[k] (0 = A).
    for k in 0..4 { abcd[folge[k]] = roh[k]; }
    abcd
}

/// Vier Bytes A B C D -> zwei Register in der Wortfolge.
pub fn als_register(abcd: [u8; 4], folge: [usize; 4]) -> [u16; 2] {
    let mut roh = [0u8; 4];
    for k in 0..4 { roh[k] = abcd[folge[k]]; }
    [((roh[0] as u16) << 8) | roh[1] as u16, ((roh[2] as u16) << 8) | roh[3] as u16]
}

/// "8N1", "8E1", "8O2", "7E1" ...
pub fn format_lesen(f: &str) -> Option<(u8, char, u8)> {
    let c: Vec<char> = f.trim().to_ascii_uppercase().chars().collect();
    if c.len() != 3 { return None; }
    let daten = match c[0] { '7' => 7, '8' => 8, _ => return None };
    if !matches!(c[1], 'N' | 'E' | 'O') { return None; }
    let stopp = match c[2] { '1' => 1, '2' => 2, _ => return None };
    Some((daten, c[1], stopp))
}

// ------------------------------------------------------------------ Leitung

enum Leitung {
    Tcp(TcpStream),
    #[cfg(feature = "serial")]
    Seriell(Box<dyn serialport::SerialPort>),
}

impl Leitung {
    fn schreiben(&mut self, b: &[u8]) -> std::io::Result<()> {
        match self {
            Leitung::Tcp(s) => s.write_all(b),
            #[cfg(feature = "serial")]
            Leitung::Seriell(s) => { s.write_all(b)?; s.flush() }
        }
    }
    /// Lesen, hoechstens bis `frist`; 0 Bytes = Zeit um.
    fn lesen(&mut self, buf: &mut [u8], frist: Instant) -> std::io::Result<usize> {
        let rest = frist.saturating_duration_since(Instant::now());
        if rest.is_zero() { return Ok(0); }
        let rest = rest.max(Duration::from_millis(1));
        let r = match self {
            Leitung::Tcp(s) => { s.set_read_timeout(Some(rest))?; s.read(buf) }
            #[cfg(feature = "serial")]
            Leitung::Seriell(s) => { let _ = s.set_timeout(rest); s.read(buf) }
        };
        match r {
            Ok(0) => Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "die Gegenseite hat die Verbindung geschlossen")),
            Ok(n) => Ok(n),
            Err(e) if matches!(e.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) => Ok(0),
            Err(e) => Err(e),
        }
    }
    /// Was noch von einer frueheren (zu spaeten) Antwort im Puffer liegt
    /// wegwerfen -- sonst haelt RTU sie fuer die Antwort auf die naechste
    /// Frage. TCP braucht es nicht (die Kennung ordnet zu).
    fn leeren(&mut self) {
        match self {
            Leitung::Tcp(s) => {
                if s.set_nonblocking(true).is_ok() {
                    let mut b = [0u8; 256];
                    while let Ok(n) = s.read(&mut b) { if n == 0 { break; } }
                    let _ = s.set_nonblocking(false);
                }
            }
            #[cfg(feature = "serial")]
            Leitung::Seriell(s) => { let _ = s.clear(serialport::ClearBuffer::Input); }
        }
    }
}

pub struct Geraet {
    leitung: Leitung,
    rtu: bool,
    pub einheit: u8,
    pub zeitgrenze: Duration,
    kennung: u16,
}

fn verbinden(host: &str, port: u16, fn_: &str) -> Result<TcpStream, String> {
    let adressen: Vec<_> = (host, port).to_socket_addrs()
        .map_err(|e| format!("{}: {} nicht gefunden: {}", fn_, host, e))?
        .collect();
    let mut letzter = format!("{}: {} hat keine Adresse", fn_, host);
    for a in adressen {
        match TcpStream::connect_timeout(&a, Duration::from_secs(3)) {
            Ok(s) => { let _ = s.set_nodelay(true); return Ok(s); }
            Err(e) => letzter = format!("{}: {}:{} -- {}", fn_, host, port, e),
        }
    }
    Err(letzter)
}

pub fn tcp(host: &str, port: i64, einheit: i64) -> Result<Geraet, String> {
    const F: &str = "MODBUS_TCP";
    if !(1..=65535).contains(&port) { return Err(format!("{}: Port {} ausserhalb 1..65535", F, port)); }
    let einheit = einheit_pruefen(einheit, F)?;
    let s = verbinden(host, port as u16, F)?;
    Ok(Geraet { leitung: Leitung::Tcp(s), rtu: false, einheit, zeitgrenze: Duration::from_millis(1000), kennung: 0 })
}

pub fn rtu(ziel: &str, baud: i64, format: &str, einheit: i64) -> Result<Geraet, String> {
    const F: &str = "MODBUS_RTU";
    let einheit = einheit_pruefen(einheit, F)?;
    let leitung = if ziel.len() >= 4 && ziel[..4].eq_ignore_ascii_case("tcp:") {
        let rest = &ziel[4..];
        let (host, port) = rest.rsplit_once(':')
            .and_then(|(h, p)| p.parse::<u16>().ok().filter(|p| *p > 0).map(|p| (h.trim_matches(|c| c == '[' || c == ']'), p)))
            .ok_or_else(|| format!("{}: \"{}\" ist kein Netzziel -- so: \"tcp:192.168.1.60:502\"", F, ziel))?;
        Leitung::Tcp(verbinden(host, port, F)?)
    } else {
        let (daten, paritaet, stopp) = format_lesen(format)
            .ok_or_else(|| format!("{}: Format {:?} -- so: \"8N1\", \"8E1\", \"8O1\", \"8N2\"", F, format))?;
        seriell(ziel, baud, daten, paritaet, stopp)?
    };
    Ok(Geraet { leitung, rtu: true, einheit, zeitgrenze: Duration::from_millis(1000), kennung: 0 })
}

#[cfg(feature = "serial")]
fn seriell(port: &str, baud: i64, daten: u8, paritaet: char, stopp: u8) -> Result<Leitung, String> {
    use serialport::{DataBits, Parity, StopBits};
    if !(300..=4_000_000).contains(&baud) { return Err(format!("MODBUS_RTU: Baudrate {} ausserhalb 300..4000000", baud)); }
    let conn = serialport::new(port, baud as u32)
        .data_bits(if daten == 7 { DataBits::Seven } else { DataBits::Eight })
        .parity(match paritaet { 'E' => Parity::Even, 'O' => Parity::Odd, _ => Parity::None })
        .stop_bits(if stopp == 2 { StopBits::Two } else { StopBits::One })
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|e| format!("MODBUS_RTU: {}: {}", port, e))?;
    Ok(Leitung::Seriell(conn))
}

#[cfg(not(feature = "serial"))]
fn seriell(_port: &str, _baud: i64, _d: u8, _p: char, _s: u8) -> Result<Leitung, String> {
    Err("MODBUS_RTU: serielle Anschluesse kennt diese Laufzeit nicht (gebaut ohne --hardware) \
         -- ueber einen RTU-Umsetzer im Netz geht \"tcp:host:port\"".to_string())
}

pub fn einheit_pruefen(e: i64, fn_: &str) -> Result<u8, String> {
    if !(0..=255).contains(&e) { return Err(format!("{}: Geraeteadresse {} ausserhalb 0..255", fn_, e)); }
    Ok(e as u8)
}

impl Geraet {
    /// Eine Anfrage schicken und die PDU der Antwort holen. `None` = keine
    /// Antwort erwartet (RTU an Adresse 0: an alle, niemand antwortet).
    fn fragen(&mut self, pdu: &[u8], fn_: &str) -> Result<Option<Vec<u8>>, String> {
        let frist = Instant::now() + self.zeitgrenze;
        let ms = self.zeitgrenze.as_millis();
        let io = |e: std::io::Error| format!("{}: {}", fn_, e);
        if self.rtu {
            self.leitung.leeren();
            self.leitung.schreiben(&rtu_rahmen(self.einheit, pdu)).map_err(io)?;
            if self.einheit == 0 { return Ok(None); }
            let mut r: Vec<u8> = Vec::new();
            let mut buf = [0u8; 260];
            loop {
                let ziel = if r.len() >= 3 { rtu_laenge(&r) } else { None };
                if let Some(z) = ziel { if r.len() >= z { r.truncate(z); break; } }
                if r.len() >= 3 && ziel.is_none() {
                    return Err(format!("{}: unverstaendliche Antwort (Funktion {})", fn_, r[1]));
                }
                let n = self.leitung.lesen(&mut buf, frist).map_err(io)?;
                if n == 0 {
                    return Err(if r.is_empty() {
                        format!("{}: keine Antwort von Geraet {} nach {} ms", fn_, self.einheit, ms)
                    } else {
                        format!("{}: Antwort nach {} von {} Bytes abgebrochen", fn_, r.len(), ziel.unwrap_or(r.len() + 1))
                    });
                }
                r.extend_from_slice(&buf[..n]);
            }
            let (rumpf, crc) = r.split_at(r.len() - 2);
            if crc16(rumpf) != (crc[0] as u16 | (crc[1] as u16) << 8) {
                return Err(format!("{}: Pruefsumme der Antwort stimmt nicht (Stoerung auf der Leitung?)", fn_));
            }
            if rumpf[0] != self.einheit {
                return Err(format!("{}: Antwort von Geraet {} statt {}", fn_, rumpf[0], self.einheit));
            }
            return Ok(Some(rumpf[1..].to_vec()));
        }
        self.kennung = self.kennung.wrapping_add(1);
        let k = self.kennung;
        self.leitung.schreiben(&tcp_rahmen(k, self.einheit, pdu)).map_err(io)?;
        // Eine Antwort mit fremder Kennung gehoert zu einer frueheren Frage,
        // deren Zeitgrenze schon abgelaufen war -- uebergehen.
        loop {
            let mut kopf = [0u8; 7];
            self.genau(&mut kopf, frist, fn_, ms)?;
            if kopf[2] != 0 || kopf[3] != 0 {
                return Err(format!("{}: kein Modbus-TCP (Protokoll {})", fn_, (kopf[2] as u16) << 8 | kopf[3] as u16));
            }
            let l = ((kopf[4] as usize) << 8) | kopf[5] as usize;
            if !(2..=254).contains(&l) { return Err(format!("{}: Laenge {} im Kopf ist kein Modbus-TCP", fn_, l)); }
            let mut rumpf = vec![0u8; l - 1];
            self.genau(&mut rumpf, frist, fn_, ms)?;
            if ((kopf[0] as u16) << 8 | kopf[1] as u16) == k { return Ok(Some(rumpf)); }
        }
    }

    fn genau(&mut self, ziel: &mut [u8], frist: Instant, fn_: &str, ms: u128) -> Result<(), String> {
        let mut n = 0;
        while n < ziel.len() {
            let k = self.leitung.lesen(&mut ziel[n..], frist).map_err(|e| format!("{}: {}", fn_, e))?;
            if k == 0 { return Err(format!("{}: keine Antwort von Geraet {} nach {} ms", fn_, self.einheit, ms)); }
            n += k;
        }
        Ok(())
    }

    fn lesend(&mut self, fn_: &str) -> Result<(), String> {
        if self.rtu && self.einheit == 0 {
            return Err(format!("{}: an Adresse 0 (an alle) antwortet niemand -- lesen geht nur von einem Geraet", fn_));
        }
        Ok(())
    }

    pub fn bits(&mut self, fc: u8, adr: i64, anzahl: i64, fn_: &str) -> Result<Vec<bool>, String> {
        self.lesend(fn_)?;
        let (adr, n) = bereich(adr, anzahl, 2000, fn_)?;
        let a = self.fragen(&pdu_lesen(fc, adr, n), fn_)?.unwrap_or_default();
        antwort_pruefen(fc, &a, fn_)?;
        antwort_bits(&a, n as usize, fn_)
    }

    pub fn register(&mut self, fc: u8, adr: i64, anzahl: i64, fn_: &str) -> Result<Vec<u16>, String> {
        self.lesend(fn_)?;
        let (adr, n) = bereich(adr, anzahl, 125, fn_)?;
        let a = self.fragen(&pdu_lesen(fc, adr, n), fn_)?.unwrap_or_default();
        antwort_pruefen(fc, &a, fn_)?;
        antwort_register(&a, n as usize, fn_)
    }

    /// Schreiben; die Antwort wiederholt Adresse und Wert bzw. Anzahl.
    pub fn schreiben(&mut self, pdu: Vec<u8>, fn_: &str) -> Result<(), String> {
        if let Some(a) = self.fragen(&pdu, fn_)? {
            antwort_pruefen(pdu[0], &a, fn_)?;
            if a.len() < 5 || a[1..5] != pdu[1..5] {
                return Err(format!("{}: das Geraet bestaetigt etwas anderes als geschrieben", fn_));
            }
        }
        Ok(())
    }
}

/// Adresse 0..65535 und Anzahl 1..max, zusammen nicht ueber 65536.
pub fn bereich(adr: i64, anzahl: i64, max: i64, fn_: &str) -> Result<(u16, u16), String> {
    if !(0..=65535).contains(&adr) { return Err(format!("{}: Adresse {} ausserhalb 0..65535", fn_, adr)); }
    if !(1..=max).contains(&anzahl) { return Err(format!("{}: Anzahl {} ausserhalb 1..{}", fn_, anzahl, max)); }
    if adr + anzahl > 65536 { return Err(format!("{}: {} ab Adresse {} reicht ueber 65535 hinaus", fn_, anzahl, adr)); }
    Ok((adr as u16, anzahl as u16))
}

/// Ein Registerwert: 0..65535, oder -32768..-1 als Zweierkomplement.
pub fn registerwert(w: i64, fn_: &str) -> Result<u16, String> {
    if !(-32768..=65535).contains(&w) {
        return Err(format!("{}: Registerwert {} ausserhalb -32768..65535", fn_, w));
    }
    Ok(w as i32 as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pruefsumme_wie_im_beispiel() {
        // Das Lehrbuchbeispiel: Geraet 1, lies 10 Halteregister ab 0.
        assert_eq!(rtu_rahmen(1, &pdu_lesen(3, 0, 10)), vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A, 0xC5, 0xCD]);
        // Modbus-Spezifikation, Anhang (Funktion 5): Spule 172 an, Geraet 0x11.
        assert_eq!(rtu_rahmen(0x11, &pdu_spule(0xAC, true)), vec![0x11, 0x05, 0x00, 0xAC, 0xFF, 0x00, 0x4E, 0x8B]);
        assert_eq!(crc16(b"123456789"), 0x4B37);
    }

    #[test]
    fn rahmen_aus_der_spezifikation() {
        // "Modbus Application Protocol V1.1b3", 6.11/6.12.
        assert_eq!(pdu_spulen(19, &[true, false, true, true, false, false, true, true, true, false]),
                   vec![0x0F, 0x00, 0x13, 0x00, 0x0A, 0x02, 0xCD, 0x01]);
        assert_eq!(pdu_registers(1, &[0x000A, 0x0102]), vec![0x10, 0x00, 0x01, 0x00, 0x02, 0x04, 0x00, 0x0A, 0x01, 0x02]);
        assert_eq!(tcp_rahmen(0x1234, 1, &pdu_lesen(4, 8, 1)), vec![0x12, 0x34, 0, 0, 0, 6, 1, 4, 0, 8, 0, 1]);
        // 6.1: Spulen 20..38 lesen, Antwort CD 6B 05.
        let b = antwort_bits(&[1, 3, 0xCD, 0x6B, 0x05], 19, "T").unwrap();
        let s: String = b.iter().map(|x| if *x { '1' } else { '0' }).collect();
        assert_eq!(s, "1011001111010110101");
        assert_eq!(antwort_register(&[3, 6, 0x02, 0x2B, 0, 0, 0, 0x64], 3, "T").unwrap(), vec![555, 0, 100]);
    }

    #[test]
    fn laenge_und_ausnahme() {
        assert_eq!(rtu_laenge(&[1, 3, 4]), Some(9));
        assert_eq!(rtu_laenge(&[1, 0x83, 2]), Some(5));
        assert_eq!(rtu_laenge(&[1, 16, 0]), Some(8));
        assert_eq!(rtu_laenge(&[1, 43, 0]), None);
        let e = antwort_pruefen(3, &[0x83, 2], "MODBUS_READ_HOLDING").unwrap_err();
        assert!(e.contains("Ausnahme 2 (unerlaubte Datenadresse)"), "{}", e);
    }

    #[test]
    fn wortfolgen() {
        // 123.456 als float32 = 42 F6 E9 79
        let abcd = 123.456f32.to_be_bytes();
        for f in ["ABCD", "CDAB", "BADC", "DCBA"] {
            let o = folge_lesen(f, "T").unwrap();
            let r = als_register(abcd, o);
            assert_eq!(zwei_register(r[0], r[1], o), abcd, "{}", f);
        }
        assert_eq!(als_register(abcd, folge_lesen("ABCD", "T").unwrap()), [0x42F6, 0xE979]);
        assert_eq!(als_register(abcd, folge_lesen("CDAB", "T").unwrap()), [0xE979, 0x42F6]);
        assert_eq!(als_register(abcd, folge_lesen("BADC", "T").unwrap()), [0xF642, 0x79E9]);
        assert_eq!(als_register(abcd, folge_lesen("dcba", "T").unwrap()), [0x79E9, 0xF642]);
        assert_eq!(format_lesen("8e1"), Some((8, 'E', 1)));
        assert_eq!(format_lesen("9N1"), None);
    }
}
