//! Mikrofon (MIC_*): Pegel, Spektrum, Tonhoehe, Wellenform und Aufnahme vom
//! Eingang des Rechners.
//!
//! Der Eingang laeuft ueber cpal (dieselbe Fassung, die Kira fuer die Ausgabe
//! benutzt) in einem eigenen Faden -- ein cpal-Strom ist nicht auf jedem
//! System `Send`, also lebt er dort, wo er angelegt wurde, bis MIC_STOP ihn
//! abholt. Der Rueckruf mischt die Kanaele zu einem und schiebt die Werte in
//! EINEN Puffer (`PUFFER`): Ring fuer Spektrum, Tonhoehe und Wellenform, die
//! Spitze seit der letzten Frage und, wenn eine laeuft, die Aufnahme.
//!
//! **Pruefen ohne Mikrofon:** `DHRT_MIKROFON=datei.wav` macht aus einer
//! WAV-Datei ein Mikrofon -- sie wird in Echtzeit eingespeist, in Schleife,
//! ueber denselben Puffer. Die Laeufer der CI haben keinen Eingang; so
//! laesst sich alles ausser dem cpal-Weg selbst pruefen.

use std::sync::mpsc;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Die juengsten Werte: genug fuer die FFT (1024) und die Tonhoehe bis
/// 50 Hz bei 96 kHz (2048 + 1920).
pub const RING: usize = 8192;
/// Die laengste Aufnahme in Sekunden (wie AUDIO_RECORD_START).
pub const MAX_SEKUNDEN: f64 = 3600.0;

struct Aufnahme { werte: Vec<f32>, max: usize }

struct Puffer {
    ring: Vec<f32>,
    pos: usize,
    /// Wie viele Werte insgesamt kamen -- ein leerer Ring ist keine Stille.
    gesamt: u64,
    spitze: f32,
    aufnahme: Option<Aufnahme>,
    fehler: Option<String>,
}

static PUFFER: Mutex<Puffer> = Mutex::new(Puffer {
    ring: Vec::new(), pos: 0, gesamt: 0, spitze: 0.0, aufnahme: None, fehler: None,
});

fn puffer() -> std::sync::MutexGuard<'static, Puffer> {
    PUFFER.lock().unwrap_or_else(|e| e.into_inner())
}

/// Neue Werte (ein Kanal, -1..1) annehmen -- vom cpal-Rueckruf oder vom
/// Faden, der eine Datei einspeist.
fn einspeisen(werte: &[f32]) {
    let mut p = puffer();
    if p.ring.len() != RING { p.ring = vec![0.0; RING]; p.pos = 0; }
    let mut pos = p.pos;
    let mut spitze = p.spitze;
    for &w in werte {
        p.ring[pos] = w;
        pos = (pos + 1) % RING;
        spitze = spitze.max(w.abs());
    }
    p.pos = pos;
    p.spitze = spitze;
    p.gesamt += werte.len() as u64;
    if let Some(a) = p.aufnahme.as_mut() {
        let rest = a.max.saturating_sub(a.werte.len());
        a.werte.extend_from_slice(&werte[..werte.len().min(rest)]);
    }
}

fn zuruecksetzen() {
    let mut p = puffer();
    p.ring = vec![0.0; RING];
    p.pos = 0;
    p.gesamt = 0;
    p.spitze = 0.0;
    p.aufnahme = None;
    p.fehler = None;
}

/// Die juengsten `n` Werte, aelteste zuerst; was noch nie kam, ist 0.
pub fn juengste(n: usize) -> Vec<f32> {
    let p = puffer();
    let n = n.min(RING);
    if p.ring.len() != RING { return vec![0.0; n]; }
    let start = (p.pos + RING - n) % RING;
    (0..n).map(|i| p.ring[(start + i) % RING]).collect()
}

/// Hoechster Ausschlag seit dem letzten Aufruf (0..1); setzt ihn zurueck.
pub fn spitze_nehmen() -> f64 {
    let mut p = puffer();
    let s = p.spitze;
    p.spitze = 0.0;
    s as f64
}

/// Wie viele Werte seit MIC_START kamen.
pub fn gesamt() -> u64 { puffer().gesamt }

/// Ein Fehler des Geraets (abgezogen, verweigert) -- danach liefert es nichts mehr.
pub fn fehler() -> Option<String> { puffer().fehler.clone() }

pub fn aufnahme_starten(rate: u32, sekunden: f64) {
    let max = (rate as f64 * sekunden) as usize;
    puffer().aufnahme = Some(Aufnahme { werte: Vec::with_capacity(max.min(48000 * 60)), max });
}

pub fn aufnahme_laeuft() -> bool { puffer().aufnahme.is_some() }

pub fn aufnahme_nehmen() -> Option<Vec<f32>> { puffer().aufnahme.take().map(|a| a.werte) }

// --- Rechnen (rein, ohne Geraet) ----------------------------------------

/// Effektivwert (RMS) -- das, was man als Lautstaerke hoert, anders als die
/// Spitze, die ein einzelner Knacks bestimmt.
pub fn effektivwert(w: &[f32]) -> f64 {
    if w.is_empty() { return 0.0; }
    (w.iter().map(|&x| (x as f64) * (x as f64)).sum::<f64>() / w.len() as f64).sqrt()
}

/// Wie viele Werte die Tonhoehe braucht: ein Fenster von 2048 und Platz fuer
/// die laengste Periode (50 Hz).
pub fn tonhoehe_werte(rate: u32) -> usize {
    (2048 + (rate as usize / 50) + 2).min(RING)
}

/// Grundton in Hz nach dem YIN-Verfahren (de Cheveigne/Kawahara 2002):
/// Differenzfunktion, auf ihren laufenden Mittelwert bezogen, erster Wert
/// unter 0,15 und dort das oertliche Minimum, parabolisch verfeinert.
/// 50..2000 Hz; 0, wenn es leise ist (Effektivwert unter 0,01) oder kein
/// Ton heraussticht (Rauschen, Sprache zwischen zwei Silben).
pub fn tonhoehe(w: &[f32], rate: u32) -> f64 {
    if rate == 0 { return 0.0; }
    let rate_f = rate as f64;
    let tau_min = ((rate_f / 2000.0) as usize).max(2);
    let tau_max = (rate_f / 50.0) as usize;
    if w.len() < tau_max * 2 + 2 { return 0.0; }
    if effektivwert(w) < 0.01 { return 0.0; }
    let fenster = w.len() - tau_max - 1;
    let mut d = vec![0.0f64; tau_max + 2];
    for tau in 1..=tau_max + 1 {
        let mut s = 0.0f64;
        for i in 0..fenster {
            let x = (w[i] - w[i + tau]) as f64;
            s += x * x;
        }
        d[tau] = s;
    }
    let mut dn = vec![1.0f64; tau_max + 2];
    let mut summe = 0.0;
    for tau in 1..=tau_max + 1 {
        summe += d[tau];
        dn[tau] = if summe > 0.0 { d[tau] * tau as f64 / summe } else { 1.0 };
    }
    let mut tau = tau_min;
    while tau <= tau_max {
        if dn[tau] < 0.15 {
            while tau < tau_max && dn[tau + 1] < dn[tau] { tau += 1; }
            break;
        }
        tau += 1;
    }
    if tau > tau_max { return 0.0; }
    let (a, b, c) = (dn[tau - 1], dn[tau], dn[tau + 1]);
    let nenner = a - 2.0 * b + c;
    let feiner = if nenner.abs() > 1e-12 { tau as f64 + 0.5 * (a - c) / nenner } else { tau as f64 };
    if feiner <= 0.0 { 0.0 } else { rate_f / feiner }
}

/// Werte (ein Kanal) als WAV, 16 Bit -- fuer MIC_RECORD_SAVE, das ohne
/// Ausgabegeraet gehen soll (ein SOUND braucht die Audio-Ausgabe).
pub fn wav_bytes(werte: &[f32], rate: u32) -> Vec<u8> {
    let daten = werte.len() * 2;
    let mut b = Vec::with_capacity(44 + daten);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&((36 + daten) as u32).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(daten as u32).to_le_bytes());
    for &w in werte {
        let v = (w.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        b.extend_from_slice(&v.to_le_bytes());
    }
    b
}

// --- Geraete --------------------------------------------------------------

/// Ein laufender Eingang. Abraeumen (MIC_STOP, Programmende) haelt den Faden
/// an und wartet auf ihn -- danach kommt kein Wert mehr in den Puffer.
pub struct Lauf {
    stop: Option<mpsc::Sender<()>>,
    faden: Option<JoinHandle<()>>,
    pub name: String,
    pub rate: u32,
}

impl Drop for Lauf {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(f) = self.faden.take() { let _ = f.join(); }
    }
}

/// Die WAV-Datei, die ein Mikrofon spielt (Pruefungen), oder keine.
fn datei_statt_geraet() -> Option<String> {
    std::env::var("DHRT_MIKROFON").ok().filter(|s| !s.is_empty())
}

fn datei_name(pfad: &str) -> String {
    let name = std::path::Path::new(pfad).file_name().map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| pfad.to_string());
    format!("Datei: {}", name)
}

/// Die Namen aller Eingaenge, der Standard zuerst.
pub fn geraete() -> Result<Vec<String>, String> {
    if let Some(pfad) = datei_statt_geraet() { return Ok(vec![datei_name(&pfad)]); }
    geraete_cpal()
}

#[cfg(not(target_os = "emscripten"))]
fn geraete_cpal() -> Result<Vec<String>, String> {
    use cpal::traits::HostTrait;
    let host = cpal::default_host();
    let standard = host.default_input_device().map(|d| d.to_string());
    let mut namen: Vec<String> = host.input_devices()
        .map_err(|e| format!("MIC_DEVICES: die Eingaenge sind nicht lesbar -- {}", e))?
        .map(|d| d.to_string())
        .collect();
    if let Some(s) = standard {
        if let Some(i) = namen.iter().position(|n| *n == s) {
            let n = namen.remove(i);
            namen.insert(0, n);
        }
    }
    Ok(namen)
}

#[cfg(target_os = "emscripten")]
fn geraete_cpal() -> Result<Vec<String>, String> { Ok(Vec::new()) }

/// Den Eingang `name` oeffnen ("" = der Standard). Ein Name passt genau oder,
/// wenn es keinen genauen gibt, als Teiltext ohne Gross/klein -- die Namen
/// der Systeme sind lang ("Mikrofon (Realtek(R) Audio)").
pub fn starten(name: &str) -> Result<Lauf, String> {
    zuruecksetzen();
    if let Some(pfad) = datei_statt_geraet() { return datei_starten(&pfad, name); }
    starten_cpal(name)
}

fn datei_starten(pfad: &str, name: &str) -> Result<Lauf, String> {
    let anzeige = datei_name(pfad);
    if !name.is_empty() && !anzeige.to_lowercase().contains(&name.to_lowercase()) {
        return Err(format!("MIC_START: kein Eingang '{}' -- es gibt: {}", name, anzeige));
    }
    let bytes = std::fs::read(pfad).map_err(|e| format!("MIC_START: DHRT_MIKROFON '{}' -- {}", pfad, e))?;
    let (werte, rate) = crate::sprache::wav_lesen(&bytes)
        .map_err(|e| e.replace("SPEAK: die Sprachausgabe lieferte kein WAV", "keine WAV-Datei")
                       .replace("SPEAK: ", ""))
        .map_err(|e| format!("MIC_START: DHRT_MIKROFON '{}' -- {}", pfad, e))?;
    if werte.is_empty() { return Err(format!("MIC_START: DHRT_MIKROFON '{}' ist leer", pfad)); }
    let werte: Vec<f32> = werte.iter().map(|&w| w as f32).collect();
    let (tx, rx) = mpsc::channel::<()>();
    let block = (rate as usize / 100).max(1);
    let faden = std::thread::spawn(move || {
        let anfang = Instant::now();
        let mut stelle = 0usize;
        let mut geliefert: u64 = 0;
        loop {
            match rx.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {}
                _ => return,
            }
            // So viele Werte, wie die Uhr seit dem Anfang verlangt -- ein
            // verschlafener Faden holt auf, statt die Datei zu dehnen.
            let soll = (anfang.elapsed().as_secs_f64() * rate as f64) as u64 + block as u64;
            let mut stueck = Vec::with_capacity(block * 2);
            while geliefert < soll {
                stueck.push(werte[stelle]);
                stelle = (stelle + 1) % werte.len();
                geliefert += 1;
            }
            if !stueck.is_empty() { einspeisen(&stueck); }
            std::thread::sleep(Duration::from_millis(10));
        }
    });
    Ok(Lauf { stop: Some(tx), faden: Some(faden), name: anzeige, rate })
}

#[cfg(not(target_os = "emscripten"))]
fn starten_cpal(name: &str) -> Result<Lauf, String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let host = cpal::default_host();
    let geraet = if name.is_empty() {
        host.default_input_device().ok_or_else(||
            "MIC_START: dieser Rechner hat kein Mikrofon (kein Eingang gefunden)".to_string())?
    } else {
        let alle: Vec<cpal::Device> = host.input_devices()
            .map_err(|e| format!("MIC_START: die Eingaenge sind nicht lesbar -- {}", e))?.collect();
        let klein = name.to_lowercase();
        let genau = alle.iter().position(|d| d.to_string() == name);
        let teil = alle.iter().position(|d| d.to_string().to_lowercase().contains(&klein));
        match genau.or(teil) {
            Some(i) => alle.into_iter().nth(i).unwrap(),
            None => {
                let namen: Vec<String> = alle.iter().map(|d| d.to_string()).collect();
                return Err(format!("MIC_START: kein Eingang '{}' -- es gibt: {}", name,
                    if namen.is_empty() { "keinen".to_string() } else { namen.join(", ") }));
            }
        }
    };
    let anzeige = geraet.to_string();
    let (tx_stop, rx_stop) = mpsc::channel::<()>();
    let (tx_bereit, rx_bereit) = mpsc::channel::<Result<u32, String>>();
    let faden = std::thread::spawn(move || {
        let ergebnis = (|| -> Result<(cpal::Stream, u32), String> {
            let cfg = geraet.default_input_config()
                .map_err(|e| format!("MIC_START: das Mikrofon nennt kein Format -- {}", e))?;
            let rate = cfg.sample_rate();
            let format = cfg.sample_format();
            let config = cfg.config();
            let strom = strom_bauen(&geraet, config, format)?;
            strom.play().map_err(|e| format!("MIC_START: das Mikrofon startet nicht -- {}", e))?;
            Ok((strom, rate))
        })();
        match ergebnis {
            Ok((strom, rate)) => {
                let _ = tx_bereit.send(Ok(rate));
                let _ = rx_stop.recv();
                drop(strom);
            }
            Err(e) => { let _ = tx_bereit.send(Err(e)); }
        }
    });
    let rate = match rx_bereit.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => { let _ = faden.join(); return Err(e); }
        Err(_) => return Err("MIC_START: das Mikrofon antwortet nicht (10 s)".into()),
    };
    Ok(Lauf { stop: Some(tx_stop), faden: Some(faden), name: anzeige, rate })
}

#[cfg(target_os = "emscripten")]
fn starten_cpal(_name: &str) -> Result<Lauf, String> {
    Err("MIC_START: im Browser gibt es noch kein Mikrofon".into())
}

#[cfg(not(target_os = "emscripten"))]
fn strom_bauen(geraet: &cpal::Device, config: cpal::StreamConfig, format: cpal::SampleFormat)
    -> Result<cpal::Stream, String>
{
    use cpal::SampleFormat as F;
    match format {
        F::F32 => strom_mit::<f32>(geraet, config),
        F::F64 => strom_mit::<f64>(geraet, config),
        F::I8 => strom_mit::<i8>(geraet, config),
        F::I16 => strom_mit::<i16>(geraet, config),
        F::I32 => strom_mit::<i32>(geraet, config),
        F::U8 => strom_mit::<u8>(geraet, config),
        F::U16 => strom_mit::<u16>(geraet, config),
        F::U32 => strom_mit::<u32>(geraet, config),
        anders => Err(format!("MIC_START: das Mikrofon liefert das Format {:?}, das dhrt nicht liest", anders)),
    }
}

#[cfg(not(target_os = "emscripten"))]
fn strom_mit<T>(geraet: &cpal::Device, config: cpal::StreamConfig) -> Result<cpal::Stream, String>
where T: cpal::SizedSample, f32: cpal::FromSample<T>
{
    use cpal::traits::DeviceTrait;
    use cpal::Sample;
    let kanaele = (config.channels as usize).max(1);
    let mut mono: Vec<f32> = Vec::new();
    geraet.build_input_stream::<T, _, _>(
        config,
        move |daten: &[T], _info: &cpal::InputCallbackInfo| {
            mono.clear();
            for rahmen in daten.chunks(kanaele) {
                let s: f32 = rahmen.iter().map(|&x| f32::from_sample(x)).sum();
                mono.push(s / kanaele as f32);
            }
            einspeisen(&mono);
        },
        |e| { puffer().fehler = Some(e.to_string()); },
        None,
    ).map_err(|e| format!("MIC_START: das Mikrofon laesst sich nicht oeffnen -- {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sinus(hz: f64, rate: u32, n: usize, amp: f32) -> Vec<f32> {
        (0..n).map(|i| (amp as f64 * (std::f64::consts::TAU * hz * i as f64 / rate as f64).sin()) as f32).collect()
    }

    #[test]
    fn tonhoehe_trifft_sinus_und_saege() {
        for &(hz, rate) in &[(440.0, 48000u32), (110.0, 44100), (82.41, 48000), (1000.0, 44100), (55.0, 48000)] {
            let w = sinus(hz, rate, tonhoehe_werte(rate), 0.5);
            let f = tonhoehe(&w, rate);
            assert!((f - hz).abs() / hz < 0.005, "{} Hz bei {}: {}", hz, rate, f);
        }
        // Eine Saege hat starke Obertoene -- gemeint ist trotzdem der Grundton.
        let rate = 44100;
        let saege: Vec<f32> = (0..tonhoehe_werte(rate))
            .map(|i| ((i as f64 * 196.0 / rate as f64).fract() * 2.0 - 1.0) as f32 * 0.4).collect();
        let f = tonhoehe(&saege, rate);
        assert!((f - 196.0).abs() < 1.5, "Saege: {}", f);
    }

    #[test]
    fn tonhoehe_schweigt_bei_stille_und_rauschen() {
        let rate = 48000;
        assert_eq!(tonhoehe(&vec![0.0; tonhoehe_werte(rate)], rate), 0.0);
        assert_eq!(tonhoehe(&sinus(440.0, rate, tonhoehe_werte(rate), 0.005), rate), 0.0);
        let mut x: u32 = 12345;
        let rauschen: Vec<f32> = (0..tonhoehe_werte(rate)).map(|_| {
            x ^= x << 13; x ^= x >> 17; x ^= x << 5;
            (x as f32 / u32::MAX as f32) - 0.5
        }).collect();
        assert_eq!(tonhoehe(&rauschen, rate), 0.0);
        assert_eq!(tonhoehe(&[0.5; 10], rate), 0.0);
    }

    #[test]
    fn effektivwert_eines_sinus() {
        let w = sinus(100.0, 48000, 48000, 1.0);
        assert!((effektivwert(&w) - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-3);
        assert_eq!(effektivwert(&[]), 0.0);
    }

    #[test]
    fn wav_kopf_und_werte() {
        let b = wav_bytes(&[0.0, 1.0, -1.0, 2.0], 22050);
        assert_eq!(&b[0..4], b"RIFF");
        assert_eq!(b.len(), 44 + 8);
        assert_eq!(u32::from_le_bytes([b[24], b[25], b[26], b[27]]), 22050);
        assert_eq!(i16::from_le_bytes([b[46], b[47]]), 32767);
        assert_eq!(i16::from_le_bytes([b[48], b[49]]), -32767);
        assert_eq!(i16::from_le_bytes([b[50], b[51]]), 32767);
        let (werte, rate) = crate::sprache::wav_lesen(&b).unwrap();
        assert_eq!((werte.len(), rate), (4, 22050));
    }
}
