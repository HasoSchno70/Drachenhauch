//! Module `gpio`, `i2c` und `spi`: die Pins eines Raspberry Pi -- und jedes
//! anderen Linux-Rechners mit Pins -- direkt, ohne Arduino dazwischen.
//!
//! Gesprochen wird mit den Schnittstellen des Kernels, nicht mit den
//! Registern des Pi-Chips:
//!   * GPIO ueber `/dev/gpiochipN` (Zeichengeraet, uAPI v2, Linux 5.10+),
//!   * I2C ueber `/dev/i2c-N` (i2c-dev; SMBus zuerst, rohes I2C nur fuer
//!     das, was SMBus nicht ausdruecken kann),
//!   * SPI ueber `/dev/spidevB.C` (spidev).
//!
//! Das hat zwei Gruende: es geht auf JEDEM Pi (3, 4, 5 haben drei
//! verschiedene GPIO-Chips -- der Pi 5 seinen RP1 hinter PCIe, an dem die
//! alten Register-Bibliotheken scheitern), und es laesst sich ohne Pi pruefen:
//! der Kernel hat Simulatoren fuer genau diese Schnittstellen (`gpio-sim`,
//! `i2c-stub`), die die CI auf einem gewoehnlichen Linux-Rechner laedt.
//!
//! Die Entscheidungen (Wortlisten, Flags, welcher I2C-Weg) sind reine
//! Funktionen und auf jedem System getestet; die ioctl-Aufrufe gibt es nur
//! unter Linux, anderswo melden die Befehle "nur unter Linux".

#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

// ---------------------------------------------------------------------------
// ioctl-Nummern (asm-generic, gilt fuer x86_64 und aarch64)
// ---------------------------------------------------------------------------

const IOC_W: u32 = 1;
const IOC_R: u32 = 2;

/// `_IOC(dir, typ, nr, groesse)` aus `<asm-generic/ioctl.h>`.
pub(crate) const fn ioc(dir: u32, typ: u32, nr: u32, groesse: u32) -> u32 {
    (dir << 30) | (groesse << 16) | (typ << 8) | nr
}

pub(crate) const GPIO_GET_CHIPINFO: u32 = ioc(IOC_R, 0xB4, 0x01, 68);
pub(crate) const GPIO_V2_GET_LINEINFO: u32 = ioc(IOC_R | IOC_W, 0xB4, 0x05, 256);
pub(crate) const GPIO_V2_GET_LINE: u32 = ioc(IOC_R | IOC_W, 0xB4, 0x07, 592);
pub(crate) const GPIO_V2_LINE_SET_CONFIG: u32 = ioc(IOC_R | IOC_W, 0xB4, 0x0D, 272);
pub(crate) const GPIO_V2_LINE_GET_VALUES: u32 = ioc(IOC_R | IOC_W, 0xB4, 0x0E, 16);
pub(crate) const GPIO_V2_LINE_SET_VALUES: u32 = ioc(IOC_R | IOC_W, 0xB4, 0x0F, 16);

pub(crate) const SPI_IOC_WR_MODE: u32 = ioc(IOC_W, b'k' as u32, 1, 1);
pub(crate) const SPI_IOC_WR_BITS_PER_WORD: u32 = ioc(IOC_W, b'k' as u32, 3, 1);
pub(crate) const SPI_IOC_WR_MAX_SPEED_HZ: u32 = ioc(IOC_W, b'k' as u32, 4, 4);
/// `SPI_IOC_MESSAGE(1)`: ein `spi_ioc_transfer` zu 32 Bytes.
pub(crate) const SPI_IOC_MESSAGE_1: u32 = ioc(IOC_W, b'k' as u32, 0, 32);

// i2c-dev hat feste Nummern ohne _IOC.
pub(crate) const I2C_SLAVE: u32 = 0x0703;
pub(crate) const I2C_TENBIT: u32 = 0x0704;
pub(crate) const I2C_FUNCS: u32 = 0x0705;
pub(crate) const I2C_RDWR: u32 = 0x0707;
pub(crate) const I2C_SMBUS: u32 = 0x0720;

// Faehigkeiten eines I2C-Busses (I2C_FUNCS).
pub(crate) const F_I2C: u64 = 0x0000_0001;
pub(crate) const F_10BIT: u64 = 0x0000_0002;
pub(crate) const F_QUICK: u64 = 0x0001_0000;
pub(crate) const F_READ_BYTE: u64 = 0x0002_0000;
pub(crate) const F_WRITE_BYTE: u64 = 0x0004_0000;
pub(crate) const F_READ_BYTE_DATA: u64 = 0x0008_0000;
pub(crate) const F_WRITE_BYTE_DATA: u64 = 0x0010_0000;
pub(crate) const F_READ_I2C_BLOCK: u64 = 0x0400_0000;
pub(crate) const F_WRITE_I2C_BLOCK: u64 = 0x0800_0000;

// Flags einer GPIO-Leitung (gpio_v2_line_flag).
pub(crate) const L_ACTIVE_LOW: u64 = 1 << 1;
pub(crate) const L_INPUT: u64 = 1 << 2;
pub(crate) const L_OUTPUT: u64 = 1 << 3;
pub(crate) const L_EDGE_RISING: u64 = 1 << 4;
pub(crate) const L_EDGE_FALLING: u64 = 1 << 5;
pub(crate) const L_OPEN_DRAIN: u64 = 1 << 6;
pub(crate) const L_BIAS_PULL_UP: u64 = 1 << 8;
pub(crate) const L_BIAS_PULL_DOWN: u64 = 1 << 9;
pub(crate) const L_BIAS_DISABLED: u64 = 1 << 10;

// ---------------------------------------------------------------------------
// Woerter und Flags -- rein, auf jedem System getestet
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Richtung { Ein, Aus, OffenerKollektor }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Widerstand { Unveraendert, Keiner, Hoch, Runter }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Flanke { Keine, Steigend, Fallend, Beide }

pub fn richtung_lesen(w: &str, f: &str) -> Result<Richtung, String> {
    match w.trim().to_lowercase().as_str() {
        "in" | "ein" | "eingang" | "input" => Ok(Richtung::Ein),
        "out" | "ausgang" | "output" => Ok(Richtung::Aus),
        "opendrain" | "open_drain" | "open-drain" | "offen" => Ok(Richtung::OffenerKollektor),
        _ => Err(format!("{f}: unbekannte Richtung \"{w}\" -- erlaubt sind \"in\", \"out\" und \"opendrain\"")),
    }
}

pub fn widerstand_lesen(w: &str, f: &str) -> Result<Widerstand, String> {
    match w.trim().to_lowercase().as_str() {
        "" => Ok(Widerstand::Unveraendert),
        "none" | "keiner" | "kein" | "off" => Ok(Widerstand::Keiner),
        "up" | "pullup" | "pull_up" | "hoch" => Ok(Widerstand::Hoch),
        "down" | "pulldown" | "pull_down" | "runter" => Ok(Widerstand::Runter),
        _ => Err(format!("{f}: unbekannter Widerstand \"{w}\" -- erlaubt sind \"up\", \"down\" und \"none\"")),
    }
}

pub fn flanke_lesen(w: &str, f: &str) -> Result<Flanke, String> {
    match w.trim().to_lowercase().as_str() {
        "rising" | "steigend" | "hoch" => Ok(Flanke::Steigend),
        "falling" | "fallend" | "runter" => Ok(Flanke::Fallend),
        "both" | "beide" => Ok(Flanke::Beide),
        "none" | "keine" | "aus" => Ok(Flanke::Keine),
        _ => Err(format!("{f}: unbekannte Flanke \"{w}\" -- erlaubt sind \"rising\", \"falling\", \"both\" und \"none\"")),
    }
}

/// Die Flags einer Leitung. Ein Eingang mit Flanke meldet Ereignisse; ein
/// Ausgang kann keine Flanke haben (das waere ein Fehler des Kernels, darum
/// faellt sie hier weg).
pub fn linien_flags(r: Richtung, w: Widerstand, fl: Flanke, aktiv_tief: bool) -> u64 {
    let mut f = match r {
        Richtung::Ein => L_INPUT,
        Richtung::Aus => L_OUTPUT,
        Richtung::OffenerKollektor => L_OUTPUT | L_OPEN_DRAIN,
    };
    match w {
        Widerstand::Unveraendert => {}
        Widerstand::Keiner => f |= L_BIAS_DISABLED,
        Widerstand::Hoch => f |= L_BIAS_PULL_UP,
        Widerstand::Runter => f |= L_BIAS_PULL_DOWN,
    }
    if r == Richtung::Ein {
        match fl {
            Flanke::Keine => {}
            Flanke::Steigend => f |= L_EDGE_RISING,
            Flanke::Fallend => f |= L_EDGE_FALLING,
            Flanke::Beide => f |= L_EDGE_RISING | L_EDGE_FALLING,
        }
    }
    if aktiv_tief { f |= L_ACTIVE_LOW; }
    f
}

/// Welcher GPIO-Chip ist der mit den Stiftleisten-Pins? Pi 5: `pinctrl-rp1`
/// (je nach Kernel gpiochip0 oder gpiochip4), Pi 4: `pinctrl-bcm2711`,
/// Pi 3/Zero: `pinctrl-bcm2835`. Sonst der erste Chip.
pub fn standard_chip(chips: &[(String, String)]) -> Option<usize> {
    let pos = |pref: &str| chips.iter().position(|(_, l)| l.starts_with(pref));
    pos("pinctrl-rp1").or_else(|| pos("pinctrl-bcm")).or_else(|| pos("pinctrl-")).or(if chips.is_empty() { None } else { Some(0) })
}

/// Weg eines I2C-Auftrags. SMBus geht vor, weil der Kernel ihn auf jedem
/// echten I2C-Bus nachbildet (der Pi kann beides) und weil nur er sich ohne
/// Hardware pruefen laesst (`i2c-stub` kann kein rohes I2C).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum I2cWeg {
    Schnell,          // SMBus quick (0 Bytes schreiben)
    Byte,             // SMBus byte (1 Byte ohne Register)
    ByteDaten,        // SMBus byte data (Register + 1 Byte)
    Block,            // SMBus I2C block (Register + bis 32 Bytes)
    Roh,              // rohes I2C (write/read bzw. I2C_RDWR)
}

pub fn i2c_weg(funcs: u64, schreiben: bool, mit_register: bool, n: usize) -> Option<I2cWeg> {
    let hat = |b: u64| funcs & b != 0;
    let smbus = if schreiben {
        // Beim Schreiben ist das erste Byte das Register (wie bei FIRMATA_I2C_WRITE).
        match n {
            0 if hat(F_QUICK) => Some(I2cWeg::Schnell),
            1 if hat(F_WRITE_BYTE) => Some(I2cWeg::Byte),
            2 if hat(F_WRITE_BYTE_DATA) => Some(I2cWeg::ByteDaten),
            3..=33 if hat(F_WRITE_I2C_BLOCK) => Some(I2cWeg::Block),
            _ => None,
        }
    } else if mit_register {
        match n {
            1 if hat(F_READ_BYTE_DATA) => Some(I2cWeg::ByteDaten),
            1..=32 if hat(F_READ_I2C_BLOCK) => Some(I2cWeg::Block),
            _ => None,
        }
    } else {
        match n {
            1 if hat(F_READ_BYTE) => Some(I2cWeg::Byte),
            _ => None,
        }
    };
    smbus.or(if hat(F_I2C) && (n > 0 || !schreiben) { Some(I2cWeg::Roh) } else { None })
}

pub fn i2c_bus_pfad(ziel: &str) -> String {
    let z = ziel.trim();
    if z.starts_with('/') { z.to_string() }
    else if let Some(rest) = z.strip_prefix("i2c-") { format!("/dev/i2c-{rest}") }
    else { format!("/dev/i2c-{z}") }
}

pub fn gpio_chip_pfad(ziel: &str) -> String {
    let z = ziel.trim();
    if z.starts_with('/') { z.to_string() }
    else if z.chars().all(|c| c.is_ascii_digit()) { format!("/dev/gpiochip{z}") }
    else { format!("/dev/{z}") }
}

/// Ein Ereignis einer beobachteten Leitung.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ereignis {
    pub pin: u32,
    pub steigend: bool,
    pub zeit_ns: u64,
}

/// Hoechstens so viele Ereignisse warten; darueber faellt das aelteste weg
/// (wie bei MIDI -- wer liest, will den Stand von jetzt).
pub const SCHLANGE_MAX: usize = 1024;

fn bereich(f: &str, was: &str, v: i64, lo: i64, hi: i64) -> Result<(), String> {
    if v < lo || v > hi { Err(format!("{f}: {was} {v} ausserhalb {lo}..{hi}")) } else { Ok(()) }
}

pub fn bytes_pruefen(f: &str, d: &[i64]) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(d.len());
    for (k, &x) in d.iter().enumerate() {
        if !(0..=255).contains(&x) { return Err(format!("{f}: Byte {x} an Stelle {k} ausserhalb 0..255")); }
        out.push(x as u8);
    }
    Ok(out)
}

pub fn i2c_adresse_pruefen(f: &str, adr: i64) -> Result<(), String> { bereich(f, "Adresse", adr, 0, 1023) }

// ---------------------------------------------------------------------------
// Linux
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod os {
    use super::*;
    use std::collections::{HashMap, VecDeque};
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Write};
    use std::os::fd::{AsRawFd, FromRawFd};

    #[repr(C)]
    #[allow(dead_code)]
    struct ChipInfo { name: [u8; 32], label: [u8; 32], lines: u32 }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Attr { id: u32, padding: u32, wert: u64 }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CfgAttr { attr: Attr, mask: u64 }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct LineConfig { flags: u64, num_attrs: u32, padding: [u32; 5], attrs: [CfgAttr; 10] }

    #[repr(C)]
    struct LineRequest {
        offsets: [u32; 64], consumer: [u8; 32], config: LineConfig,
        num_lines: u32, event_buffer_size: u32, padding: [u32; 5], fd: i32,
    }

    #[repr(C)]
    struct LineValues { bits: u64, mask: u64 }

    #[repr(C)]
    struct LineEvent { timestamp_ns: u64, id: u32, offset: u32, seqno: u32, line_seqno: u32, padding: [u32; 6] }

    #[repr(C)]
    struct LineInfo { name: [u8; 32], consumer: [u8; 32], offset: u32, num_attrs: u32, flags: u64, attrs: [Attr; 10], padding: [u32; 4] }

    #[repr(C)]
    struct SmbusDaten { block: [u8; 34] }

    #[repr(C)]
    struct SmbusAuftrag { read_write: u8, command: u8, size: u32, data: *mut SmbusDaten }

    #[repr(C)]
    struct I2cMsg { addr: u16, flags: u16, len: u16, buf: *mut u8 }

    #[repr(C)]
    struct I2cRdwr { msgs: *mut I2cMsg, nmsgs: u32 }

    #[repr(C)]
    struct SpiTransfer {
        tx_buf: u64, rx_buf: u64, len: u32, speed_hz: u32, delay_usecs: u16,
        bits_per_word: u8, cs_change: u8, tx_nbits: u8, rx_nbits: u8, word_delay_usecs: u8, pad: u8,
    }

    // Die Groessen MUESSEN die des Kernels sein, sonst nimmt das ioctl die
    // falsche Nummer (die Groesse steht darin) -- beim Uebersetzen geprueft.
    const _: () = assert!(std::mem::size_of::<ChipInfo>() == 68);
    const _: () = assert!(std::mem::size_of::<LineConfig>() == 272);
    const _: () = assert!(std::mem::size_of::<LineRequest>() == 592);
    const _: () = assert!(std::mem::size_of::<LineValues>() == 16);
    const _: () = assert!(std::mem::size_of::<LineEvent>() == 48);
    const _: () = assert!(std::mem::size_of::<LineInfo>() == 256);
    const _: () = assert!(std::mem::size_of::<SpiTransfer>() == 32);

    fn ioctl<T>(fd: i32, nr: u32, arg: *mut T) -> std::io::Result<i32> {
        // `request` ist unter glibc ein unsigned long, unter musl ein int --
        // der Cast deckt beides.
        let r = unsafe { libc::ioctl(fd, nr as _, arg) };
        if r < 0 { Err(std::io::Error::last_os_error()) } else { Ok(r) }
    }

    fn text(b: &[u8]) -> String {
        let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
        String::from_utf8_lossy(&b[..n]).into_owned()
    }

    /// Geraetedatei oeffnen; die Meldung sagt, was zu tun ist (`gruppe` ist
    /// zugleich die Art: gpio, i2c oder spi).
    fn oeffnen_mit_hinweis(f: &str, pfad: &str, gruppe: &str) -> Result<File, String> {
        OpenOptions::new().read(true).write(true).open(pfad).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => match gruppe {
                "i2c" => format!("{f}: {pfad} gibt es nicht -- I2C einschalten (Raspberry Pi: sudo raspi-config, Interface Options, I2C) bzw. sudo modprobe i2c-dev"),
                "spi" => format!("{f}: {pfad} gibt es nicht -- SPI einschalten (Raspberry Pi: sudo raspi-config, Interface Options, SPI)"),
                _ => format!("{f}: {pfad} gibt es nicht"),
            },
            std::io::ErrorKind::PermissionDenied => format!(
                "{f}: keine Berechtigung fuer {pfad} -- den Benutzer in die Gruppe {gruppe} aufnehmen (sudo adduser $USER {gruppe}, dann neu anmelden)"),
            _ => format!("{f}: {pfad}: {e}"),
        })
    }

    // ---------------- GPIO ----------------

    struct Linie { datei: File, flags: u64, wert: bool, entprellen_us: u32 }

    pub struct Chip {
        datei: File,
        pub label: String,
        pub zahl: u32,
        linien: HashMap<u32, Linie>,
        schlange: VecDeque<Ereignis>,
        letztes: Option<Ereignis>,
    }

    pub fn chips() -> Result<Vec<(String, String, u32)>, String> {
        let mut namen: Vec<String> = match std::fs::read_dir("/dev") {
            Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.starts_with("gpiochip")).collect(),
            Err(e) => return Err(format!("GPIO_CHIPS: /dev: {e}")),
        };
        // gpiochip10 nach gpiochip9, nicht nach gpiochip1
        namen.sort_by_key(|n| n["gpiochip".len()..].parse::<u32>().unwrap_or(u32::MAX));
        let mut out = Vec::new();
        for n in namen {
            let pfad = format!("/dev/{n}");
            match OpenOptions::new().read(true).write(true).open(&pfad) {
                Ok(d) => {
                    let mut info: ChipInfo = unsafe { std::mem::zeroed() };
                    if ioctl(d.as_raw_fd(), GPIO_GET_CHIPINFO, &mut info).is_ok() {
                        out.push((n, text(&info.label), info.lines));
                    }
                }
                // Ohne Berechtigung trotzdem nennen -- sonst saehe es aus, als gaebe es keine Pins.
                Err(_) => out.push((n, String::new(), 0)),
            }
        }
        Ok(out)
    }

    impl Chip {
        pub fn oeffnen(ziel: &str) -> Result<Chip, String> {
            const F: &str = "GPIO_OPEN";
            let pfad = if ziel.trim().is_empty() {
                let liste = chips()?;
                let paare: Vec<(String, String)> = liste.iter().map(|(n, l, _)| (n.clone(), l.clone())).collect();
                match standard_chip(&paare) {
                    Some(i) => format!("/dev/{}", liste[i].0),
                    None => return Err(format!("{F}: kein GPIO-Chip gefunden (/dev/gpiochip*)")),
                }
            } else if ziel.contains('/') || ziel.starts_with("gpiochip") || ziel.chars().all(|c| c.is_ascii_digit()) {
                gpio_chip_pfad(ziel)
            } else {
                // eine Beschriftung wie "pinctrl-bcm2711" oder "gpio-sim"
                let liste = chips()?;
                match liste.iter().find(|(_, l, _)| l == ziel.trim()) {
                    Some((n, _, _)) => format!("/dev/{n}"),
                    None => return Err(format!("{F}: kein GPIO-Chip mit der Beschriftung \"{ziel}\" -- GPIO_CHIPS() nennt die vorhandenen")),
                }
            };
            let datei = oeffnen_mit_hinweis(F, &pfad, "gpio")?;
            let mut info: ChipInfo = unsafe { std::mem::zeroed() };
            ioctl(datei.as_raw_fd(), GPIO_GET_CHIPINFO, &mut info)
                .map_err(|e| format!("{F}: {pfad} ist kein GPIO-Chip ({e})"))?;
            Ok(Chip {
                datei, label: text(&info.label), zahl: info.lines,
                linien: HashMap::new(), schlange: VecDeque::new(), letztes: None,
            })
        }

        fn pin_pruefen(&self, f: &str, pin: i64) -> Result<u32, String> {
            if pin < 0 || pin >= self.zahl as i64 {
                return Err(format!("{f}: Pin {pin} ausserhalb 0..{} ({} hat {} Leitungen)", self.zahl as i64 - 1, self.label, self.zahl));
            }
            Ok(pin as u32)
        }

        fn info(&self, pin: u32) -> Option<LineInfo> {
            let mut li: LineInfo = unsafe { std::mem::zeroed() };
            li.offset = pin;
            ioctl(self.datei.as_raw_fd(), GPIO_V2_GET_LINEINFO, &mut li).ok().map(|_| li)
        }

        pub fn linien_name(&self, f: &str, pin: i64) -> Result<String, String> {
            let p = self.pin_pruefen(f, pin)?;
            Ok(self.info(p).map(|li| text(&li.name)).unwrap_or_default())
        }

        pub fn finden(&self, name: &str) -> i64 {
            let gesucht = name.trim().to_lowercase();
            (0..self.zahl).find(|&p| self.info(p).map(|li| text(&li.name).to_lowercase() == gesucht).unwrap_or(false))
                .map(|p| p as i64).unwrap_or(-1)
        }

        fn konfig(flags: u64, wert: Option<bool>, entprellen_us: u32) -> LineConfig {
            let mut c: LineConfig = unsafe { std::mem::zeroed() };
            c.flags = flags;
            let mut n = 0;
            if let Some(v) = wert {
                c.attrs[n] = CfgAttr { attr: Attr { id: 2, padding: 0, wert: v as u64 }, mask: 1 };
                n += 1;
            }
            if entprellen_us > 0 {
                c.attrs[n] = CfgAttr { attr: Attr { id: 3, padding: 0, wert: entprellen_us as u64 }, mask: 1 };
                n += 1;
            }
            c.num_attrs = n as u32;
            c
        }

        /// Leitung anfordern oder neu einstellen.
        fn stellen(&mut self, f: &str, pin: u32, flags: u64, wert: Option<bool>, entprellen_us: u32) -> Result<(), String> {
            let ausgang = flags & L_OUTPUT != 0;
            let wert = if ausgang { Some(wert.unwrap_or_else(|| self.linien.get(&pin).map(|l| l.wert).unwrap_or(false))) } else { None };
            let mut cfg = Self::konfig(flags, wert, entprellen_us);
            if let Some(l) = self.linien.get_mut(&pin) {
                ioctl(l.datei.as_raw_fd(), GPIO_V2_LINE_SET_CONFIG, &mut cfg)
                    .map_err(|e| format!("{f}: Pin {pin} laesst sich nicht so einstellen ({e})"))?;
                l.flags = flags; l.entprellen_us = entprellen_us;
                if let Some(v) = wert { l.wert = v; }
                return Ok(());
            }
            let mut req: LineRequest = unsafe { std::mem::zeroed() };
            req.offsets[0] = pin;
            req.num_lines = 1;
            let wer = b"drachenhauch";
            req.consumer[..wer.len()].copy_from_slice(wer);
            req.config = cfg;
            if let Err(e) = ioctl(self.datei.as_raw_fd(), GPIO_V2_GET_LINE, &mut req) {
                if e.raw_os_error() == Some(libc::EBUSY) {
                    let wer = self.info(pin).map(|li| text(&li.consumer)).unwrap_or_default();
                    return Err(if wer.is_empty() { format!("{f}: Pin {pin} ist schon belegt") }
                               else { format!("{f}: Pin {pin} ist schon belegt (von \"{wer}\")") });
                }
                return Err(format!("{f}: Pin {pin} laesst sich nicht anfordern ({e})"));
            }
            let datei = unsafe { File::from_raw_fd(req.fd) };
            // Ereignisse ohne Warten lesen
            unsafe {
                let fl = libc::fcntl(req.fd, libc::F_GETFL);
                libc::fcntl(req.fd, libc::F_SETFL, fl | libc::O_NONBLOCK);
            }
            self.linien.insert(pin, Linie { datei, flags, wert: wert.unwrap_or(false), entprellen_us });
            Ok(())
        }

        pub fn modus(&mut self, f: &str, pin: i64, r: Richtung, w: Widerstand) -> Result<(), String> {
            let p = self.pin_pruefen(f, pin)?;
            let alt = self.linien.get(&p).map(|l| (l.flags, l.entprellen_us));
            // Ein Widerstand "unveraendert" behaelt den bisherigen.
            let bias = alt.map(|(fl, _)| fl & (L_BIAS_PULL_UP | L_BIAS_PULL_DOWN | L_BIAS_DISABLED)).unwrap_or(0);
            let mut flags = linien_flags(r, w, Flanke::Keine, false);
            if w == Widerstand::Unveraendert { flags |= bias; }
            // Beobachtung bleibt, solange es ein Eingang bleibt.
            if r == Richtung::Ein { if let Some((fl, _)) = alt { flags |= fl & (L_EDGE_RISING | L_EDGE_FALLING); } }
            let ent = if r == Richtung::Ein { alt.map(|(_, e)| e).unwrap_or(0) } else { 0 };
            self.stellen(f, p, flags, None, ent)
        }

        pub fn schreiben(&mut self, f: &str, pin: i64, v: bool) -> Result<(), String> {
            let p = self.pin_pruefen(f, pin)?;
            match self.linien.get_mut(&p) {
                None => self.stellen(f, p, L_OUTPUT, Some(v), 0),
                Some(l) if l.flags & L_OUTPUT == 0 => Err(format!(
                    "{f}: Pin {p} ist ein Eingang -- erst GPIO_MODE(chip, {p}, \"out\")")),
                Some(l) => {
                    let mut wv = LineValues { bits: v as u64, mask: 1 };
                    ioctl(l.datei.as_raw_fd(), GPIO_V2_LINE_SET_VALUES, &mut wv)
                        .map_err(|e| format!("{f}: Pin {p}: {e}"))?;
                    l.wert = v;
                    Ok(())
                }
            }
        }

        pub fn lesen(&mut self, f: &str, pin: i64) -> Result<bool, String> {
            let p = self.pin_pruefen(f, pin)?;
            if !self.linien.contains_key(&p) { self.stellen(f, p, L_INPUT, None, 0)?; }
            let l = &self.linien[&p];
            let mut wv = LineValues { bits: 0, mask: 1 };
            ioctl(l.datei.as_raw_fd(), GPIO_V2_LINE_GET_VALUES, &mut wv)
                .map_err(|e| format!("{f}: Pin {p}: {e}"))?;
            Ok(wv.bits & 1 != 0)
        }

        pub fn beobachten(&mut self, f: &str, pin: i64, fl: Flanke, entprellen_ms: i64) -> Result<(), String> {
            let p = self.pin_pruefen(f, pin)?;
            bereich(f, "Entprellzeit (ms)", entprellen_ms, 0, 1000)?;
            let alt = self.linien.get(&p).map(|l| l.flags).unwrap_or(0);
            let bias = alt & (L_BIAS_PULL_UP | L_BIAS_PULL_DOWN | L_BIAS_DISABLED);
            let flags = linien_flags(Richtung::Ein, Widerstand::Unveraendert, fl, false) | bias;
            self.stellen(f, p, flags, None, (entprellen_ms * 1000) as u32)
        }

        pub fn freigeben(&mut self, pin: i64) {
            if pin >= 0 { self.linien.remove(&(pin as u32)); }
            self.schlange.retain(|e| e.pin as i64 != pin);
        }

        fn einsammeln(&mut self) {
            let mut puffer = [0u8; 48 * 16];
            let mut pins: Vec<u32> = self.linien.iter()
                .filter(|(_, l)| l.flags & (L_EDGE_RISING | L_EDGE_FALLING) != 0).map(|(p, _)| *p).collect();
            pins.sort_unstable();
            let mut neu: Vec<Ereignis> = Vec::new();
            for p in pins {
                let l = self.linien.get_mut(&p).unwrap();
                loop {
                    match l.datei.read(&mut puffer) {
                        Ok(n) if n >= 48 => {
                            for k in 0..n / 48 {
                                let ev: LineEvent = unsafe { std::ptr::read_unaligned(puffer[k * 48..].as_ptr() as *const LineEvent) };
                                neu.push(Ereignis { pin: ev.offset, steigend: ev.id == 1, zeit_ns: ev.timestamp_ns });
                            }
                        }
                        _ => break,
                    }
                }
            }
            // Mehrere Leitungen: in der Reihenfolge, in der sie geschahen.
            neu.sort_by_key(|e| e.zeit_ns);
            for e in neu {
                if self.schlange.len() >= SCHLANGE_MAX { self.schlange.pop_front(); }
                self.schlange.push_back(e);
            }
        }

        pub fn naechstes(&mut self) -> bool {
            self.einsammeln();
            self.letztes = self.schlange.pop_front();
            self.letztes.is_some()
        }

        pub fn letztes(&self) -> Option<Ereignis> { self.letztes }
    }

    // ---------------- I2C ----------------

    pub struct Bus { datei: File, funcs: u64, adr: i64 }

    impl Bus {
        pub fn oeffnen(ziel: &str) -> Result<Bus, String> {
            const F: &str = "I2C_OPEN";
            let pfad = i2c_bus_pfad(ziel);
            let datei = oeffnen_mit_hinweis(F, &pfad, "i2c")?;
            let mut funcs: libc::c_ulong = 0;
            ioctl(datei.as_raw_fd(), I2C_FUNCS, &mut funcs).map_err(|e| format!("{F}: {pfad} ist kein I2C-Bus ({e})"))?;
            Ok(Bus { datei, funcs: funcs as u64, adr: -1 })
        }

        fn adresse(&mut self, f: &str, adr: i64) -> Result<(), String> {
            i2c_adresse_pruefen(f, adr)?;
            if self.adr == adr { return Ok(()); }
            let fd = self.datei.as_raw_fd();
            let zehn = adr > 127;
            if zehn && self.funcs & F_10BIT == 0 { return Err(format!("{f}: dieser Bus kennt keine 10-Bit-Adressen")); }
            // Das ioctl nimmt den Wert selbst als Argument, keinen Zeiger.
            if unsafe { libc::ioctl(fd, I2C_TENBIT as _, zehn as libc::c_ulong) } < 0 && zehn {
                return Err(format!("{f}: 10-Bit-Adresse: {}", std::io::Error::last_os_error()));
            }
            if unsafe { libc::ioctl(fd, I2C_SLAVE as _, adr as libc::c_ulong) } < 0 {
                let e = std::io::Error::last_os_error();
                return Err(if e.raw_os_error() == Some(libc::EBUSY) {
                    format!("{f}: Adresse 0x{adr:02X} gehoert einem Treiber des Kernels")
                } else { format!("{f}: Adresse 0x{adr:02X}: {e}") });
            }
            self.adr = adr;
            Ok(())
        }

        fn smbus(&self, lesen: bool, cmd: u8, size: u32, daten: &mut SmbusDaten) -> std::io::Result<i32> {
            let mut a = SmbusAuftrag { read_write: lesen as u8, command: cmd, size, data: daten };
            ioctl(self.datei.as_raw_fd(), I2C_SMBUS, &mut a)
        }

        fn fehler(f: &str, adr: i64, e: std::io::Error) -> String {
            match e.raw_os_error() {
                Some(libc::ENXIO) | Some(libc::ENODEV) | Some(libc::EREMOTEIO) | Some(libc::EIO) =>
                    format!("{f}: an Adresse 0x{adr:02X} antwortet nichts (Verkabelung? I2C_SCAN zeigt, wer da ist)"),
                _ => format!("{f}: Adresse 0x{adr:02X}: {e}"),
            }
        }

        pub fn schreiben(&mut self, f: &str, adr: i64, d: &[u8]) -> Result<(), String> {
            self.adresse(f, adr)?;
            let weg = i2c_weg(self.funcs, true, true, d.len())
                .ok_or_else(|| format!("{f}: {} Bytes passen auf diesem Bus nicht in einen Auftrag", d.len()))?;
            let mut dat = SmbusDaten { block: [0; 34] };
            let r = match weg {
                I2cWeg::Schnell => self.smbus(false, 0, 0, &mut dat),
                I2cWeg::Byte => self.smbus(false, d[0], 1, &mut dat),
                I2cWeg::ByteDaten => { dat.block[0] = d[1]; self.smbus(false, d[0], 2, &mut dat) }
                I2cWeg::Block => {
                    dat.block[0] = (d.len() - 1) as u8;
                    dat.block[1..d.len()].copy_from_slice(&d[1..]);
                    self.smbus(false, d[0], 8, &mut dat)
                }
                I2cWeg::Roh => self.datei.write(d).map(|n| n as i32),
            };
            r.map(|_| ()).map_err(|e| Self::fehler(f, adr, e))
        }

        pub fn lesen(&mut self, f: &str, adr: i64, reg: i64, n: i64) -> Result<Vec<u8>, String> {
            self.adresse(f, adr)?;
            bereich(f, "Anzahl", n, 1, 4096)?;
            if reg >= 0 { bereich(f, "Register", reg, 0, 255)?; }
            let n = n as usize;
            let weg = i2c_weg(self.funcs, false, reg >= 0, n)
                .ok_or_else(|| format!("{f}: {n} Bytes lassen sich auf diesem Bus nicht in einem Auftrag lesen"))?;
            let mut dat = SmbusDaten { block: [0; 34] };
            let ergebnis = match weg {
                I2cWeg::Byte => self.smbus(true, 0, 1, &mut dat).map(|_| vec![dat.block[0]]),
                I2cWeg::ByteDaten => self.smbus(true, reg as u8, 2, &mut dat).map(|_| vec![dat.block[0]]),
                I2cWeg::Block => {
                    dat.block[0] = n as u8;
                    self.smbus(true, reg as u8, 8, &mut dat).map(|_| {
                        let k = (dat.block[0] as usize).min(n).min(32);
                        dat.block[1..=k].to_vec()
                    })
                }
                I2cWeg::Roh => {
                    let mut out = vec![0u8; n];
                    if reg >= 0 {
                        // Register schreiben und lesen in EINEM Auftrag (wiederholter Start)
                        let mut r = [reg as u8];
                        let flag10 = if adr > 127 { 0x0010 } else { 0 };
                        let mut msgs = [
                            I2cMsg { addr: adr as u16, flags: flag10, len: 1, buf: r.as_mut_ptr() },
                            I2cMsg { addr: adr as u16, flags: flag10 | 0x0001, len: n as u16, buf: out.as_mut_ptr() },
                        ];
                        let mut rw = I2cRdwr { msgs: msgs.as_mut_ptr(), nmsgs: 2 };
                        ioctl(self.datei.as_raw_fd(), I2C_RDWR, &mut rw).map(|_| out)
                    } else {
                        self.datei.read_exact(&mut out).map(|_| out)
                    }
                }
                I2cWeg::Schnell => unreachable!(),
            };
            ergebnis.map_err(|e| Self::fehler(f, adr, e))
        }

        /// Wer antwortet? Wie `i2cdetect`: die Bereiche, in denen EEPROMs
        /// liegen (die ein Schreiben falsch verstehen koennten), werden mit
        /// Lesen geprueft, der Rest mit einem leeren Schreiben.
        pub fn suchen(&mut self) -> Vec<i64> {
            let mut out = Vec::new();
            for adr in 0x03..=0x77i64 {
                if self.adresse("I2C_SCAN", adr).is_err() { continue; }
                let eeprom = (0x30..=0x37).contains(&adr) || (0x50..=0x5F).contains(&adr);
                let mut dat = SmbusDaten { block: [0; 34] };
                let ok = if eeprom || self.funcs & F_QUICK == 0 {
                    self.funcs & F_READ_BYTE != 0 && self.smbus(true, 0, 1, &mut dat).is_ok()
                } else {
                    self.smbus(false, 0, 0, &mut dat).is_ok()
                };
                if ok { out.push(adr); }
            }
            self.adr = -1;
            out
        }
    }

    // ---------------- SPI ----------------

    pub struct Spi { datei: File, hz: u32 }

    impl Spi {
        pub fn oeffnen(bus: i64, cs: i64, hz: i64, modus: i64) -> Result<Spi, String> {
            const F: &str = "SPI_OPEN";
            bereich(F, "Bus", bus, 0, 99)?;
            bereich(F, "Chip-Select", cs, 0, 99)?;
            bereich(F, "Takt (Hz)", hz, 1, 200_000_000)?;
            bereich(F, "Modus", modus, 0, 3)?;
            let pfad = format!("/dev/spidev{bus}.{cs}");
            let datei = oeffnen_mit_hinweis(F, &pfad, "spi")?;
            let mut s = Spi { datei, hz: hz as u32 };
            s.modus_setzen(F, modus)?;
            let mut bits: u8 = 8;
            ioctl(s.datei.as_raw_fd(), SPI_IOC_WR_BITS_PER_WORD, &mut bits).map_err(|e| format!("{F}: {e}"))?;
            s.takt_setzen(F, hz)?;
            Ok(s)
        }

        pub fn modus_setzen(&mut self, f: &str, modus: i64) -> Result<(), String> {
            bereich(f, "Modus", modus, 0, 3)?;
            let mut m = modus as u8;
            ioctl(self.datei.as_raw_fd(), SPI_IOC_WR_MODE, &mut m).map(|_| ()).map_err(|e| format!("{f}: {e}"))
        }

        pub fn takt_setzen(&mut self, f: &str, hz: i64) -> Result<(), String> {
            bereich(f, "Takt (Hz)", hz, 1, 200_000_000)?;
            let mut h = hz as u32;
            ioctl(self.datei.as_raw_fd(), SPI_IOC_WR_MAX_SPEED_HZ, &mut h).map_err(|e| format!("{f}: {e}"))?;
            self.hz = h;
            Ok(())
        }

        pub fn tauschen(&mut self, f: &str, d: &[u8]) -> Result<Vec<u8>, String> {
            if d.is_empty() { return Ok(Vec::new()); }
            if d.len() > 65536 { return Err(format!("{f}: hoechstens 65536 Bytes je Auftrag")); }
            let mut rx = vec![0u8; d.len()];
            let mut t = SpiTransfer {
                tx_buf: d.as_ptr() as u64, rx_buf: rx.as_mut_ptr() as u64, len: d.len() as u32,
                speed_hz: self.hz, delay_usecs: 0, bits_per_word: 8, cs_change: 0,
                tx_nbits: 0, rx_nbits: 0, word_delay_usecs: 0, pad: 0,
            };
            ioctl(self.datei.as_raw_fd(), SPI_IOC_MESSAGE_1, &mut t).map_err(|e| format!("{f}: {e}"))?;
            Ok(rx)
        }
    }
}

// ---------------------------------------------------------------------------
// Ausserhalb von Linux: dieselbe Schnittstelle, jeder Aufruf eine Meldung
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "linux"))]
mod os {
    use super::*;

    pub struct Chip { pub label: String, pub zahl: u32 }
    pub struct Bus;
    pub struct Spi;

    fn nein<T>(f: &str) -> Result<T, String> { Err(format!("{f}: gibt es nur unter Linux (etwa auf dem Raspberry Pi)")) }

    pub fn chips() -> Result<Vec<(String, String, u32)>, String> { nein("GPIO_CHIPS") }

    impl Chip {
        pub fn oeffnen(_ziel: &str) -> Result<Chip, String> { nein("GPIO_OPEN") }
        pub fn linien_name(&self, f: &str, _pin: i64) -> Result<String, String> { nein(f) }
        pub fn finden(&self, _name: &str) -> i64 { -1 }
        pub fn modus(&mut self, f: &str, _pin: i64, _r: Richtung, _w: Widerstand) -> Result<(), String> { nein(f) }
        pub fn schreiben(&mut self, f: &str, _pin: i64, _v: bool) -> Result<(), String> { nein(f) }
        pub fn lesen(&mut self, f: &str, _pin: i64) -> Result<bool, String> { nein(f) }
        pub fn beobachten(&mut self, f: &str, _pin: i64, _fl: Flanke, _ms: i64) -> Result<(), String> { nein(f) }
        pub fn freigeben(&mut self, _pin: i64) {}
        pub fn naechstes(&mut self) -> bool { false }
        pub fn letztes(&self) -> Option<Ereignis> { None }
    }
    impl Bus {
        pub fn oeffnen(_ziel: &str) -> Result<Bus, String> { nein("I2C_OPEN") }
        pub fn schreiben(&mut self, f: &str, _adr: i64, _d: &[u8]) -> Result<(), String> { nein(f) }
        pub fn lesen(&mut self, f: &str, _adr: i64, _reg: i64, _n: i64) -> Result<Vec<u8>, String> { nein(f) }
        pub fn suchen(&mut self) -> Vec<i64> { Vec::new() }
    }
    impl Spi {
        pub fn oeffnen(_bus: i64, _cs: i64, _hz: i64, _modus: i64) -> Result<Spi, String> { nein("SPI_OPEN") }
        pub fn modus_setzen(&mut self, f: &str, _m: i64) -> Result<(), String> { nein(f) }
        pub fn takt_setzen(&mut self, f: &str, _hz: i64) -> Result<(), String> { nein(f) }
        pub fn tauschen(&mut self, f: &str, _d: &[u8]) -> Result<Vec<u8>, String> { nein(f) }
    }
}

pub use os::{chips, Bus, Chip, Spi};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ioctl_nummern_wie_der_kernel() {
        // Werte aus <linux/gpio.h>, <linux/spi/spidev.h> (wie libgpiod und
        // py-spidev sie benutzen) -- die Groesse steckt in der Nummer.
        assert_eq!(GPIO_GET_CHIPINFO, 0x8044_B401);
        assert_eq!(GPIO_V2_GET_LINEINFO, 0xC100_B405);
        assert_eq!(GPIO_V2_GET_LINE, 0xC250_B407);
        assert_eq!(GPIO_V2_LINE_SET_CONFIG, 0xC110_B40D);
        assert_eq!(GPIO_V2_LINE_GET_VALUES, 0xC010_B40E);
        assert_eq!(GPIO_V2_LINE_SET_VALUES, 0xC010_B40F);
        assert_eq!(SPI_IOC_MESSAGE_1, 0x4020_6B00);
        assert_eq!(SPI_IOC_WR_MODE, 0x4001_6B01);
        assert_eq!(SPI_IOC_WR_BITS_PER_WORD, 0x4001_6B03);
        assert_eq!(SPI_IOC_WR_MAX_SPEED_HZ, 0x4004_6B04);
    }

    #[test]
    fn woerter_deutsch_und_englisch() {
        assert_eq!(richtung_lesen("OUT", "X").unwrap(), Richtung::Aus);
        assert_eq!(richtung_lesen("eingang", "X").unwrap(), Richtung::Ein);
        assert_eq!(richtung_lesen("opendrain", "X").unwrap(), Richtung::OffenerKollektor);
        assert!(richtung_lesen("rein", "GPIO_MODE").unwrap_err().contains("\"in\""));
        assert_eq!(widerstand_lesen("", "X").unwrap(), Widerstand::Unveraendert);
        assert_eq!(widerstand_lesen("hoch", "X").unwrap(), Widerstand::Hoch);
        assert_eq!(widerstand_lesen("pulldown", "X").unwrap(), Widerstand::Runter);
        assert_eq!(flanke_lesen("beide", "X").unwrap(), Flanke::Beide);
        assert_eq!(flanke_lesen("none", "X").unwrap(), Flanke::Keine);
        assert!(flanke_lesen("schnell", "X").is_err());
    }

    #[test]
    fn flags_einer_leitung() {
        assert_eq!(linien_flags(Richtung::Aus, Widerstand::Unveraendert, Flanke::Keine, false), L_OUTPUT);
        assert_eq!(linien_flags(Richtung::Ein, Widerstand::Hoch, Flanke::Fallend, false), L_INPUT | L_BIAS_PULL_UP | L_EDGE_FALLING);
        assert_eq!(linien_flags(Richtung::Ein, Widerstand::Keiner, Flanke::Beide, false),
            L_INPUT | L_BIAS_DISABLED | L_EDGE_RISING | L_EDGE_FALLING);
        // Ein Ausgang hat keine Flanke -- der Kernel lehnte sie ab.
        assert_eq!(linien_flags(Richtung::OffenerKollektor, Widerstand::Unveraendert, Flanke::Beide, false), L_OUTPUT | L_OPEN_DRAIN);
        assert_eq!(linien_flags(Richtung::Ein, Widerstand::Unveraendert, Flanke::Keine, true), L_INPUT | L_ACTIVE_LOW);
    }

    #[test]
    fn standard_chip_ist_der_der_stiftleiste() {
        let c = |v: &[&str]| v.iter().enumerate().map(|(i, l)| (format!("gpiochip{i}"), l.to_string())).collect::<Vec<_>>();
        // Pi 5 mit altem Kernel: die Stiftleiste ist gpiochip4
        assert_eq!(standard_chip(&c(&["gpio-brcmstb@107d508500", "gpio-brcmstb@107d508520", "gpio-brcmstb@107d517c00", "gpio-brcmstb@107d517c20", "pinctrl-rp1"])), Some(4));
        assert_eq!(standard_chip(&c(&["pinctrl-bcm2711", "raspberrypi-exp-gpio"])), Some(0));
        assert_eq!(standard_chip(&c(&["raspberrypi-exp-gpio", "pinctrl-bcm2835"])), Some(1));
        assert_eq!(standard_chip(&c(&["gpio-sim"])), Some(0));
        assert_eq!(standard_chip(&[]), None);
    }

    #[test]
    fn i2c_waehlt_smbus_zuerst() {
        let alles = F_I2C | F_QUICK | F_READ_BYTE | F_WRITE_BYTE | F_READ_BYTE_DATA | F_WRITE_BYTE_DATA | F_READ_I2C_BLOCK | F_WRITE_I2C_BLOCK;
        let stub = alles & !F_I2C; // i2c-stub: nur SMBus
        assert_eq!(i2c_weg(alles, true, true, 2), Some(I2cWeg::ByteDaten));
        assert_eq!(i2c_weg(alles, true, true, 1), Some(I2cWeg::Byte));
        assert_eq!(i2c_weg(alles, true, true, 0), Some(I2cWeg::Schnell));
        assert_eq!(i2c_weg(alles, true, true, 33), Some(I2cWeg::Block));
        assert_eq!(i2c_weg(alles, true, true, 34), Some(I2cWeg::Roh));
        assert_eq!(i2c_weg(stub, true, true, 34), None);
        assert_eq!(i2c_weg(alles, false, true, 1), Some(I2cWeg::ByteDaten));
        assert_eq!(i2c_weg(alles, false, true, 6), Some(I2cWeg::Block));
        assert_eq!(i2c_weg(alles, false, true, 40), Some(I2cWeg::Roh));
        assert_eq!(i2c_weg(alles, false, false, 1), Some(I2cWeg::Byte));
        assert_eq!(i2c_weg(alles, false, false, 2), Some(I2cWeg::Roh));
        assert_eq!(i2c_weg(stub, false, false, 2), None);
        // nur rohes I2C (manche USB-Adapter): alles roh
        assert_eq!(i2c_weg(F_I2C, false, true, 1), Some(I2cWeg::Roh));
        assert_eq!(i2c_weg(F_I2C, true, true, 0), None);
    }

    #[test]
    fn pfade() {
        assert_eq!(i2c_bus_pfad("1"), "/dev/i2c-1");
        assert_eq!(i2c_bus_pfad("i2c-20"), "/dev/i2c-20");
        assert_eq!(i2c_bus_pfad("/dev/i2c-3"), "/dev/i2c-3");
        assert_eq!(gpio_chip_pfad("4"), "/dev/gpiochip4");
        assert_eq!(gpio_chip_pfad("gpiochip0"), "/dev/gpiochip0");
        assert!(bytes_pruefen("I2C_WRITE", &[1, 256]).unwrap_err().contains("Stelle 1"));
    }
}
