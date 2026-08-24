//! Modul `smtp` -- eine E-Mail bauen und verschicken.
//!
//! Das Gegenstueck zu `xlsx` und `pdf`: dort entsteht die Auswertung, hier
//! geht sie raus. Zusammen sind das die zwei Haelften der Kette, die in
//! Bueroprogrammen am haeufigsten verlangt wird -- "Bericht bauen und
//! rausschicken".
//!
//! **Aufbau wie bei `pdf`/`xlsx`:** ein Handle wird gefuellt und am Ende
//! einmal abgeschickt. Das Bauen der Nachricht (`nachricht`) ist von der
//! Uebertragung (`senden`) getrennt -- deshalb gibt es `SMTP_MESSAGE$`, das
//! genau die Zeichen liefert, die sonst ueber die Leitung gingen. Ohne diese
//! Trennung waere jeder Test auf einen laufenden Mailserver angewiesen.
//!
//! **Textteile werden base64-kodiert**, auch der reine Text. Der Grund sind
//! die zwei Fallen von SMTP: eine Zeile darf nicht laenger als 998 Zeichen
//! sein, und eine Zeile, die mit einem Punkt anfaengt, beendet sonst die
//! Nachricht. Base64 hat weder lange Zeilen noch fuehrende Punkte -- damit
//! sind beide Fallen zu, statt sie an drei Stellen einzeln zu umgehen.
//!
//! **Kopfzeilen-Einschleusung** ist der Angriff, den man hier bekommt, wenn
//! man nicht aufpasst: ein Zeilenumbruch in einem Betreff oder einer Adresse
//! haengt beliebige weitere Kopfzeilen an (`Bcc: ...`). Jeder Wert, der in
//! eine Kopfzeile geht, wird deshalb geprueft (`pruefe_kopfwert`) -- ein
//! Umbruch ist ein Fehler und wird nicht etwa still entfernt.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::builtins::b64_encode;

/// Wie die Verbindung gesichert wird.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sicherheit {
    /// Gar nicht -- nur fuer einen Relay auf demselben Rechner.
    Keine,
    /// Erst im Klartext verbinden, dann auf TLS hochschalten (Port 587).
    Starttls,
    /// Von der ersten Sekunde an TLS (Port 465).
    Tls,
}

impl Sicherheit {
    pub fn aus_name(s: &str) -> Result<Sicherheit, String> {
        match s.to_lowercase().as_str() {
            "keine" | "none" | "aus" => Ok(Sicherheit::Keine),
            "starttls" => Ok(Sicherheit::Starttls),
            "tls" | "ssl" => Ok(Sicherheit::Tls),
            _ => Err(format!(
                "unbekannte Sicherheit {:?} -- erlaubt sind \"starttls\", \"tls\", \"keine\"", s)),
        }
    }

    /// Die uebliche Zuordnung, wenn nichts gesagt wurde.
    ///
    /// 465 ist der Port fuer TLS von Anfang an, 25 der fuer die Zustellung
    /// zwischen Servern (dort meldet sich niemand an). Alles andere -- vor
    /// allem 587, der Einlieferungsport -- ist STARTTLS.
    pub fn aus_port(port: u16) -> Sicherheit {
        match port {
            465 => Sicherheit::Tls,
            25 => Sicherheit::Keine,
            _ => Sicherheit::Starttls,
        }
    }
}

pub struct Anhang {
    pub name: String,
    pub typ: String,
    pub daten: Vec<u8>,
}

pub struct Mail {
    pub host: String,
    pub port: u16,
    pub sicherheit: Sicherheit,
    pub benutzer: String,
    pub kennwort: String,
    /// (Adresse, Anzeigename)
    pub von: (String, String),
    pub an: Vec<(String, String)>,
    pub cc: Vec<(String, String)>,
    /// Blindkopie -- steht bewusst NICHT in den Kopfzeilen, nur im Umschlag.
    pub bcc: Vec<String>,
    pub betreff: String,
    pub text: String,
    pub html: String,
    pub anhaenge: Vec<Anhang>,
    pub frist_ms: u64,
}

impl Mail {
    pub fn neu() -> Mail {
        Mail {
            host: String::new(),
            port: 587,
            sicherheit: Sicherheit::Starttls,
            benutzer: String::new(),
            kennwort: String::new(),
            von: (String::new(), String::new()),
            an: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            betreff: String::new(),
            text: String::new(),
            html: String::new(),
            anhaenge: Vec::new(),
            frist_ms: 30_000,
        }
    }

    /// Alle Empfaenger fuer den Umschlag (`RCPT TO`) -- inklusive Blindkopie.
    pub fn empfaenger(&self) -> Vec<String> {
        let mut v: Vec<String> = self.an.iter().map(|(a, _)| a.clone()).collect();
        v.extend(self.cc.iter().map(|(a, _)| a.clone()));
        v.extend(self.bcc.iter().cloned());
        v
    }
}

// ===================================================================
// Kopfzeilen und MIME (reine Funktionen -- ohne Netz testbar)
// ===================================================================

/// Ein Wert, der in eine Kopfzeile geht, darf keinen Umbruch enthalten.
///
/// Sonst haengt `betreff = "Rechnung\r\nBcc: fremder@example.com"` eine
/// zusaetzliche Kopfzeile an. Das still zu entfernen waere die schlechtere
/// Antwort: dann verschwindet ein Teil des Betreffs, ohne dass es jemand
/// merkt.
pub fn pruefe_kopfwert(s: &str, was: &str) -> Result<(), String> {
    if s.contains('\r') || s.contains('\n') {
        return Err(format!("{} enthaelt einen Zeilenumbruch -- das waere eine \
                            zusaetzliche Kopfzeile und ist deshalb nicht erlaubt", was));
    }
    Ok(())
}

/// Eine Adresse pruefen: genau ein `@`, keine spitzen Klammern, kein Umbruch.
pub fn pruefe_adresse(s: &str, was: &str) -> Result<(), String> {
    pruefe_kopfwert(s, was)?;
    let s = s.trim();
    if s.is_empty() {
        return Err(format!("{} ist leer", was));
    }
    if s.matches('@').count() != 1 || s.starts_with('@') || s.ends_with('@') {
        return Err(format!("{} {:?} sieht nicht wie eine Adresse aus \
                            (erwartet wird name@beispiel.de)", was, s));
    }
    if s.contains('<') || s.contains('>') || s.contains(' ') {
        return Err(format!("{} {:?} enthaelt ein Zeichen, das in einer Adresse \
                            nicht vorkommt (< > oder Leerzeichen). Der Anzeigename \
                            ist ein EIGENES Argument", was, s));
    }
    Ok(())
}

/// Nicht-ASCII in einer Kopfzeile: RFC-2047-Wort (`=?UTF-8?B?...?=`).
///
/// Eine Kopfzeile darf nur ASCII enthalten -- "Grüße" muss also kodiert
/// werden. Ein kodiertes Wort darf hoechstens 75 Zeichen lang sein, deshalb
/// wird in Stuecke geschnitten, und zwar entlang der ZEICHEN: mitten in
/// einem Umlaut zu trennen ergaebe zwei halbe Bytes und einen Leser, der
/// Fragezeichen anzeigt.
pub fn kodiere_wort(s: &str) -> String {
    if s.is_ascii() {
        return s.to_string();
    }
    // 75 = Grenze fuer das ganze Wort; "=?UTF-8?B?" + "?=" sind 12 Zeichen,
    // base64 macht aus 3 Bytes 4 -- also 45 Bytes je Stueck (60 Zeichen).
    let mut teile: Vec<String> = Vec::new();
    let mut puffer: Vec<u8> = Vec::new();
    for z in s.chars() {
        let mut b = [0u8; 4];
        let bytes = z.encode_utf8(&mut b).as_bytes();
        if puffer.len() + bytes.len() > 45 {
            teile.push(format!("=?UTF-8?B?{}?=", b64_encode(&puffer)));
            puffer.clear();
        }
        puffer.extend_from_slice(bytes);
    }
    if !puffer.is_empty() {
        teile.push(format!("=?UTF-8?B?{}?=", b64_encode(&puffer)));
    }
    teile.join("\r\n ")
}

/// `Name <adresse>` -- oder nur die Adresse, wenn kein Name da ist.
pub fn adress_feld(a: &(String, String)) -> String {
    if a.1.trim().is_empty() {
        a.0.clone()
    } else {
        format!("{} <{}>", kodiere_wort(&a.1), a.0)
    }
}

/// Base64 in Zeilen zu 76 Zeichen -- so verlangt es MIME.
pub fn b64_zeilen(daten: &[u8]) -> String {
    let roh = b64_encode(daten);
    let mut out = String::with_capacity(roh.len() + roh.len() / 76 * 2);
    let bytes = roh.as_bytes();
    for (i, stueck) in bytes.chunks(76).enumerate() {
        if i > 0 {
            out.push_str("\r\n");
        }
        out.push_str(std::str::from_utf8(stueck).unwrap_or(""));
    }
    out
}

/// Datum im Format von RFC 5322: `Sun, 24 Aug 2026 18:35:19 +0000`.
///
/// Gerechnet wird in UTC (`+0000`) -- die Ortszeit-Verschiebung des Rechners
/// laesst sich ohne Zeitzonen-Datenbank nicht benennen, und eine falsche
/// Verschiebung waere schlimmer als eine ehrliche Angabe in UTC.
pub fn rfc_datum(utc_sekunden: i64) -> String {
    const TAGE: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    const MONATE: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                                "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let (j, mo, t, h, mi, s) = crate::zeit::in_teile(utc_sekunden);
    let wt = crate::zeit::wochentag(utc_sekunden); // 1 = Montag
    format!("{}, {:02} {} {} {:02}:{:02}:{:02} +0000",
            TAGE[(wt - 1) as usize], t, MONATE[(mo - 1) as usize], j, h, mi, s)
}

/// Die Trennzeichenkette zwischen zwei MIME-Teilen.
///
/// Sie muss im Inhalt garantiert nicht vorkommen. Weil alle Teile base64
/// sind (Alphabet A-Z a-z 0-9 + / =), reicht dafuer schon ein Unterstrich im
/// Namen; die Zufallszahl kommt dazu, damit zwei verschachtelte Ebenen
/// verschiedene Grenzen haben.
fn grenze(zufall: u64, tiefe: u8) -> String {
    format!("----=_Drachenhauch_{}_{:016x}", tiefe, zufall)
}

fn teil(typ: &str, inhalt: &[u8]) -> String {
    format!("Content-Type: {}\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n",
            typ, b64_zeilen(inhalt))
}

/// Die vollstaendige Nachricht bauen -- genau das, was `DATA` uebertraegt.
pub fn nachricht(m: &Mail, datum: &str, zufall: u64) -> Result<String, String> {
    if m.von.0.is_empty() {
        return Err("es fehlt der Absender (SMTP_FROM)".into());
    }
    if m.an.is_empty() && m.cc.is_empty() && m.bcc.is_empty() {
        return Err("es fehlt ein Empfaenger (SMTP_TO)".into());
    }
    if m.text.is_empty() && m.html.is_empty() && m.anhaenge.is_empty() {
        return Err("die Nachricht ist leer (SMTP_TEXT, SMTP_HTML oder SMTP_ATTACH)".into());
    }

    let mut k = String::new();
    k.push_str(&format!("From: {}\r\n", adress_feld(&m.von)));
    if !m.an.is_empty() {
        let liste: Vec<String> = m.an.iter().map(adress_feld).collect();
        k.push_str(&format!("To: {}\r\n", liste.join(", ")));
    }
    if !m.cc.is_empty() {
        let liste: Vec<String> = m.cc.iter().map(adress_feld).collect();
        k.push_str(&format!("Cc: {}\r\n", liste.join(", ")));
    }
    // Bcc steht bewusst NICHT hier -- sonst waere die Blindkopie keine.
    k.push_str(&format!("Subject: {}\r\n", kodiere_wort(&m.betreff)));
    k.push_str(&format!("Date: {}\r\n", datum));
    let domain = m.von.0.split('@').nth(1).unwrap_or("localhost");
    k.push_str(&format!("Message-ID: <{:016x}.drachenhauch@{}>\r\n", zufall, domain));
    k.push_str("MIME-Version: 1.0\r\n");

    // Der Rumpf: Text und/oder HTML als "alternative", Anhaenge aussen
    // herum als "mixed". Ohne Anhaenge und mit nur einem Rumpfteil bleibt
    // die Nachricht einteilig -- kein Grund, sie kuenstlich zu verschachteln.
    let rumpf = |grenz: &str| -> String {
        let mut s = String::new();
        let mut teile: Vec<String> = Vec::new();
        if !m.text.is_empty() {
            teile.push(teil("text/plain; charset=utf-8", m.text.as_bytes()));
        }
        if !m.html.is_empty() {
            teile.push(teil("text/html; charset=utf-8", m.html.as_bytes()));
        }
        if teile.len() == 1 {
            return teile.remove(0);
        }
        s.push_str(&format!("Content-Type: multipart/alternative; boundary=\"{}\"\r\n\r\n", grenz));
        for t in &teile {
            s.push_str(&format!("--{}\r\n{}", grenz, t));
        }
        s.push_str(&format!("--{}--\r\n", grenz));
        s
    };

    let hat_rumpf = !m.text.is_empty() || !m.html.is_empty();
    let body = if m.anhaenge.is_empty() {
        rumpf(&grenze(zufall, 2))
    } else {
        let g = grenze(zufall, 1);
        let mut s = format!("Content-Type: multipart/mixed; boundary=\"{}\"\r\n\r\n", g);
        if hat_rumpf {
            s.push_str(&format!("--{}\r\n{}", g, rumpf(&grenze(zufall, 2))));
        }
        for anh in &m.anhaenge {
            s.push_str(&format!("--{}\r\n", g));
            s.push_str(&format!("Content-Type: {}\r\n", anh.typ));
            s.push_str("Content-Transfer-Encoding: base64\r\n");
            s.push_str(&format!("Content-Disposition: attachment; filename=\"{}\"\r\n\r\n",
                                kodiere_wort(&anh.name)));
            s.push_str(&b64_zeilen(&anh.daten));
            s.push_str("\r\n");
        }
        s.push_str(&format!("--{}--\r\n", g));
        s
    };

    Ok(format!("{}{}", k, body))
}

/// Punkt-Verdopplung: eine Zeile, die mit `.` anfaengt, wuerde die
/// Uebertragung beenden.
///
/// Bei base64-Rumpf kann das gar nicht vorkommen -- aber die Regel gehoert
/// zum Protokoll, und der Tag, an dem hier jemand einen anderen Kodierweg
/// einbaut, kommt frueher als der Tag, an dem er daran denkt.
pub fn punkte_verdoppeln(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, zeile) in s.split("\r\n").enumerate() {
        if i > 0 {
            out.push_str("\r\n");
        }
        if zeile.starts_with('.') {
            out.push('.');
        }
        out.push_str(zeile);
    }
    out
}

// ===================================================================
// Die Verbindung
// ===================================================================

/// Klartext oder TLS -- der Rest des Codes soll den Unterschied nicht sehen.
enum Strom {
    Klar(TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
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

impl Strom {
    fn tcp(&self) -> &TcpStream {
        match self {
            Strom::Klar(s) => s,
            Strom::Tls(s) => s.get_ref(),
        }
    }

    /// Auf TLS hochschalten.
    fn verschluesseln(self, host: &str) -> Result<Strom, String> {
        let tcp = match self {
            Strom::Klar(s) => s,
            Strom::Tls(_) => return Err("die Verbindung ist bereits verschluesselt".into()),
        };
        let mut wurzeln = rustls::RootCertStore::empty();
        wurzeln.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        // Der Anbieter wird ausdruecklich genannt statt ueber
        // `ClientConfig::builder()` geraten: waeren einmal zwei
        // Krypto-Anbieter im Baum, liefe der Ratepfad in einen Absturz.
        let anbieter = std::sync::Arc::new(rustls::crypto::ring::default_provider());
        let konfig = rustls::ClientConfig::builder_with_provider(anbieter)
            .with_safe_default_protocol_versions()
            .map_err(|e| format!("TLS liess sich nicht einrichten: {}", e))?
            .with_root_certificates(wurzeln)
            .with_no_client_auth();
        let name = rustls::pki_types::ServerName::try_from(host.to_string())
            .map_err(|_| format!("{:?} ist kein gueltiger Servername fuer TLS", host))?;
        let verbindung = rustls::ClientConnection::new(std::sync::Arc::new(konfig), name)
            .map_err(|e| format!("TLS-Verbindung fehlgeschlagen: {}", e))?;
        Ok(Strom::Tls(Box::new(rustls::StreamOwned::new(verbindung, tcp))))
    }
}

/// Eine Antwort lesen: `250 OK` oder mehrzeilig `250-...` bis `250 ...`.
///
/// Gelesen wird Zeichen fuer Zeichen und ohne Puffer. Das ist bei einer
/// Handvoll kurzer Zeilen schnell genug -- und es ist der Grund, warum
/// STARTTLS hier sicher ist: ein Puffer koennte Daten enthalten, die noch
/// aus der unverschluesselten Zeit stammen, und genau daraus besteht eine
/// bekannte Luecke.
fn lies_antwort(s: &mut Strom) -> Result<(u16, String), String> {
    let mut text = String::new();
    let mut code = 0u16;
    loop {
        let mut zeile = Vec::new();
        loop {
            let mut b = [0u8; 1];
            match s.read(&mut b) {
                Ok(0) => return Err("der Server hat die Verbindung geschlossen".into()),
                Ok(_) => {}
                Err(e) => return Err(format!("Lesen fehlgeschlagen: {}", e)),
            }
            if b[0] == b'\n' {
                break;
            }
            if b[0] != b'\r' {
                zeile.push(b[0]);
            }
        }
        let z = String::from_utf8_lossy(&zeile).to_string();
        if z.len() < 3 {
            return Err(format!("unverstaendliche Antwort: {:?}", z));
        }
        code = z[..3].parse::<u16>().map_err(|_| format!("unverstaendliche Antwort: {:?}", z))?;
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(z[3..].trim_start_matches(['-', ' ']));
        // Ein '-' an vierter Stelle heisst: es kommt noch eine Zeile.
        if z.as_bytes().get(3) != Some(&b'-') {
            break;
        }
    }
    Ok((code, text))
}

fn schreibe(s: &mut Strom, text: &str) -> Result<(), String> {
    s.write_all(text.as_bytes()).map_err(|e| format!("Schreiben fehlgeschlagen: {}", e))?;
    s.flush().map_err(|e| format!("Schreiben fehlgeschlagen: {}", e))
}

/// Einen Befehl schicken und die Antwort gegen die erwarteten Codes pruefen.
fn befehl(s: &mut Strom, cmd: &str, erwartet: &[u16], was: &str) -> Result<String, String> {
    schreibe(s, &format!("{}\r\n", cmd))?;
    let (code, text) = lies_antwort(s)?;
    if !erwartet.contains(&code) {
        return Err(format!("{}: der Server antwortet {} {}", was, code, text.replace('\n', " / ")));
    }
    Ok(text)
}

/// Die Nachricht abschicken.
pub fn senden(m: &Mail, nachricht: &str) -> Result<(), String> {
    if m.host.is_empty() {
        return Err("es fehlt der Server (SMTP_SERVER)".into());
    }
    // Ein Kennwort im Klartext ueber ein fremdes Netz waere still
    // preisgegeben. Auf demselben Rechner ist das etwas anderes -- dort ist
    // ein Relay ohne TLS der Normalfall.
    if m.sicherheit == Sicherheit::Keine && !m.kennwort.is_empty() && !ist_lokal(&m.host) {
        return Err(format!(
            "SMTP_SEND: Anmeldung ohne Verschluesselung an {} -- das Kennwort ginge \
             im Klartext ueber das Netz. Nimm \"starttls\" (Port 587) oder \"tls\" \
             (Port 465); ohne Verschluesselung geht eine Anmeldung nur an den \
             eigenen Rechner", m.host));
    }

    let frist = Duration::from_millis(m.frist_ms.max(1));
    let adressen: Vec<std::net::SocketAddr> = {
        use std::net::ToSocketAddrs;
        (m.host.as_str(), m.port).to_socket_addrs()
            .map_err(|e| format!("SMTP_SEND: {} ist nicht auffindbar ({})", m.host, e))?
            .collect()
    };
    let ziel = adressen.first()
        .ok_or_else(|| format!("SMTP_SEND: {} liefert keine Adresse", m.host))?;
    let tcp = TcpStream::connect_timeout(ziel, frist)
        .map_err(|e| format!("SMTP_SEND: keine Verbindung zu {}:{} ({})", m.host, m.port, e))?;
    tcp.set_read_timeout(Some(frist)).ok();
    tcp.set_write_timeout(Some(frist)).ok();

    let mut s = Strom::Klar(tcp);
    if m.sicherheit == Sicherheit::Tls {
        s = s.verschluesseln(&m.host).map_err(|e| format!("SMTP_SEND: {}", e))?;
    }

    let (code, text) = lies_antwort(&mut s)?;
    if code != 220 {
        return Err(format!("SMTP_SEND: der Server begruesst mit {} {}", code, text));
    }

    // Der Name im EHLO: die eigene Adresse in eckigen Klammern. Ein
    // erfundener Rechnername waere geraten; die Adresse stimmt immer.
    let ich = match s.tcp().local_addr() {
        Ok(a) => format!("[{}]", a.ip()),
        Err(_) => "[127.0.0.1]".to_string(),
    };
    let mut faehig = befehl(&mut s, &format!("EHLO {}", ich), &[250], "EHLO")?;

    if m.sicherheit == Sicherheit::Starttls {
        if !faehig.to_uppercase().contains("STARTTLS") {
            return Err(format!(
                "SMTP_SEND: {}:{} bietet kein STARTTLS an. Entweder ist der Port falsch \
                 (Einlieferung laeuft meist ueber 587, TLS-von-Anfang-an ueber 465) oder \
                 der Server kann keine Verschluesselung -- dann braucht es \"keine\", und \
                 zwar bewusst", m.host, m.port));
        }
        befehl(&mut s, "STARTTLS", &[220], "STARTTLS")?;
        s = s.verschluesseln(&m.host).map_err(|e| format!("SMTP_SEND: {}", e))?;
        // Nach dem Hochschalten gilt die alte Faehigkeitsliste nicht mehr:
        // viele Server bieten AUTH erst an, wenn verschluesselt wird.
        faehig = befehl(&mut s, &format!("EHLO {}", ich), &[250], "EHLO (nach STARTTLS)")?;
    }

    if !m.benutzer.is_empty() {
        anmelden(&mut s, &faehig, &m.benutzer, &m.kennwort)?;
    }

    befehl(&mut s, &format!("MAIL FROM:<{}>", m.von.0), &[250], "MAIL FROM")?;
    for e in m.empfaenger() {
        befehl(&mut s, &format!("RCPT TO:<{}>", e), &[250, 251],
               &format!("RCPT TO {}", e))?;
    }
    befehl(&mut s, "DATA", &[354], "DATA")?;
    schreibe(&mut s, &punkte_verdoppeln(nachricht))?;
    schreibe(&mut s, "\r\n.\r\n")?;
    let (code, text) = lies_antwort(&mut s)?;
    if code != 250 {
        return Err(format!("SMTP_SEND: der Server nimmt die Nachricht nicht an: {} {}",
                           code, text.replace('\n', " / ")));
    }
    // Ein Fehler beim QUIT waere folgenlos -- die Nachricht ist angenommen.
    let _ = befehl(&mut s, "QUIT", &[221], "QUIT");
    Ok(())
}

fn ist_lokal(host: &str) -> bool {
    let h = host.to_lowercase();
    h == "localhost" || h == "127.0.0.1" || h == "::1" || h == "[::1]"
}

/// AUTH PLAIN, sonst AUTH LOGIN -- die zwei, die jeder Server kann.
fn anmelden(s: &mut Strom, faehig: &str, benutzer: &str, kennwort: &str) -> Result<(), String> {
    let gross = faehig.to_uppercase();
    let auth: Vec<&str> = gross.lines()
        .filter(|z| z.trim_start().starts_with("AUTH"))
        .collect();
    let kann = |name: &str| auth.iter().any(|z| z.split_whitespace().any(|w| w == name));

    if kann("PLAIN") {
        let roh = format!("\0{}\0{}", benutzer, kennwort);
        befehl(s, &format!("AUTH PLAIN {}", b64_encode(roh.as_bytes())), &[235], "Anmeldung")?;
    } else if kann("LOGIN") {
        befehl(s, "AUTH LOGIN", &[334], "Anmeldung")?;
        befehl(s, &b64_encode(benutzer.as_bytes()), &[334], "Anmeldung (Benutzer)")?;
        befehl(s, &b64_encode(kennwort.as_bytes()), &[235], "Anmeldung (Kennwort)")?;
    } else if auth.is_empty() {
        return Err("SMTP_SEND: der Server bietet gar keine Anmeldung an. Entweder \
                    braucht er keine (dann SMTP_LOGIN weglassen), oder es fehlt die \
                    Verschluesselung -- viele Server nennen AUTH erst nach STARTTLS".into());
    } else {
        return Err(format!("SMTP_SEND: der Server bietet nur {} an; unterstuetzt \
                            werden PLAIN und LOGIN", auth.join(" / ")));
    }
    Ok(())
}

/// Zufall fuer Trennzeichen und Message-ID.
pub fn zufallszahl() -> u64 {
    let mut b = [0u8; 8];
    if getrandom::getrandom(&mut b).is_err() {
        // Notnagel: die Uhr. Beides dient nur der Eindeutigkeit, nicht der
        // Sicherheit -- eine Trennzeichenkette muss nicht unerratbar sein.
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        return n;
    }
    u64::from_le_bytes(b)
}

/// Sekunden seit 1970 in UTC -- fuer die `Date`-Kopfzeile.
pub fn jetzt_utc() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ===================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn mail() -> Mail {
        let mut m = Mail::neu();
        m.von = ("ich@beispiel.de".into(), String::new());
        m.an.push(("du@beispiel.de".into(), String::new()));
        m.betreff = "Test".into();
        m.text = "Hallo".into();
        m
    }

    #[test]
    fn umbruch_in_einer_kopfzeile_ist_ein_fehler() {
        assert!(pruefe_kopfwert("Rechnung\r\nBcc: fremd@x.de", "Betreff").is_err());
        assert!(pruefe_kopfwert("Rechnung", "Betreff").is_ok());
    }

    #[test]
    fn adressen_werden_geprueft() {
        assert!(pruefe_adresse("a@b.de", "An").is_ok());
        assert!(pruefe_adresse("a.b.de", "An").is_err());
        assert!(pruefe_adresse("a@b@c.de", "An").is_err());
        // Der Anzeigename gehoert in ein eigenes Argument.
        assert!(pruefe_adresse("Name <a@b.de>", "An").is_err());
    }

    #[test]
    fn ascii_bleibt_unveraendert() {
        assert_eq!(kodiere_wort("Rechnung 4711"), "Rechnung 4711");
    }

    #[test]
    fn umlaute_werden_kodiert() {
        // "Grüße" -> UTF-8-Bytes, base64.
        let w = kodiere_wort("Grüße");
        assert!(w.starts_with("=?UTF-8?B?") && w.ends_with("?="));
        let roh = &w[10..w.len() - 2];
        let zurueck = crate::builtins::b64_decode(roh).unwrap();
        assert_eq!(String::from_utf8(zurueck).unwrap(), "Grüße");
    }

    #[test]
    fn lange_umlautfolgen_werden_an_zeichengrenzen_geteilt() {
        let lang: String = std::iter::repeat('ä').take(60).collect();
        let w = kodiere_wort(&lang);
        assert!(w.contains("\r\n "), "muss gefaltet werden");
        // Jedes Stueck muss fuer sich dekodierbar sein -- genau das geht
        // schief, wenn mitten in einem Zeichen getrennt wird.
        let mut zusammen = String::new();
        for stueck in w.split("\r\n ") {
            let roh = &stueck[10..stueck.len() - 2];
            zusammen.push_str(&String::from_utf8(crate::builtins::b64_decode(roh).unwrap()).unwrap());
        }
        assert_eq!(zusammen, lang);
    }

    #[test]
    fn base64_zeilen_sind_hoechstens_76_zeichen() {
        let daten = vec![b'x'; 1000];
        for z in b64_zeilen(&daten).split("\r\n") {
            assert!(z.len() <= 76, "Zeile zu lang: {}", z.len());
        }
    }

    #[test]
    fn eine_zeile_mit_punkt_wird_verdoppelt() {
        assert_eq!(punkte_verdoppeln("a\r\n.b\r\nc"), "a\r\n..b\r\nc");
        assert_eq!(punkte_verdoppeln(".x"), "..x");
    }

    #[test]
    fn datum_im_format_der_norm() {
        // 2026-08-24 18:35:19 UTC war ein Montag.
        let t = crate::zeit::aus_teilen(2026, 8, 24, 18, 35, 19);
        assert_eq!(rfc_datum(t), "Mon, 24 Aug 2026 18:35:19 +0000");
    }

    #[test]
    fn einteilige_nachricht_bleibt_einteilig() {
        let n = nachricht(&mail(), "Mon, 24 Aug 2026 18:35:19 +0000", 1).unwrap();
        assert!(n.contains("Content-Type: text/plain; charset=utf-8"));
        assert!(!n.contains("multipart"), "kein Grund zu verschachteln");
    }

    #[test]
    fn blindkopie_steht_nicht_in_den_kopfzeilen() {
        let mut m = mail();
        m.bcc.push("heimlich@beispiel.de".into());
        let n = nachricht(&m, "x", 1).unwrap();
        assert!(!n.contains("heimlich@beispiel.de"), "sonst waere sie keine Blindkopie");
        assert!(m.empfaenger().contains(&"heimlich@beispiel.de".to_string()),
                "im Umschlag muss sie stehen");
    }

    #[test]
    fn text_und_html_ergeben_alternative() {
        let mut m = mail();
        m.html = "<b>Hallo</b>".into();
        let n = nachricht(&m, "x", 7).unwrap();
        assert!(n.contains("multipart/alternative"));
        assert!(n.contains("text/plain") && n.contains("text/html"));
    }

    #[test]
    fn anhang_ergibt_mixed_mit_eigener_grenze() {
        let mut m = mail();
        m.html = "<b>x</b>".into();
        m.anhaenge.push(Anhang { name: "a.txt".into(), typ: "text/plain".into(),
                                 daten: b"hallo".to_vec() });
        let n = nachricht(&m, "x", 9).unwrap();
        assert!(n.contains("multipart/mixed"));
        assert!(n.contains("Content-Disposition: attachment; filename=\"a.txt\""));
        // Die innere Grenze darf nicht dieselbe sein wie die aeussere.
        assert_ne!(grenze(9, 1), grenze(9, 2));
    }

    #[test]
    fn leere_nachricht_wird_abgelehnt() {
        let mut m = mail();
        m.text = String::new();
        assert!(nachricht(&m, "x", 1).unwrap_err().contains("leer"));
    }

    #[test]
    fn sicherheit_aus_dem_port() {
        assert_eq!(Sicherheit::aus_port(465), Sicherheit::Tls);
        assert_eq!(Sicherheit::aus_port(587), Sicherheit::Starttls);
        assert_eq!(Sicherheit::aus_port(25), Sicherheit::Keine);
    }
}
