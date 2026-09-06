//! Sprachausgabe: `SPEAK` und Verwandte (Weg C aus `docs/entwurf-speak.md`).
//!
//! Der Grundgedanke: eine gesprochene Zeile ist **ein Klang unter Klaengen**.
//! Die Systemstimme liefert PCM (Windows: WinRT `Windows.Media.SpeechSynthesis`,
//! gemessen 12-35 ms je Satz; macOS: `say`; Linux: `espeak-ng`), daraus wird
//! ein `SOUND` wie aus `AUDIO_NOTE`, und Kira spielt ihn auf dem Bus `speech`
//! -- mit Lautstaerke, Pause, Ausblenden, raeumlich, speicherbar als WAV.
//!
//! **Die Warteschlange braucht kein Polling.** Wer anhaengt, bekommt seinen
//! Klang mit `StartTime::Delayed(rest)` an Kira uebergeben, und Kira zaehlt
//! die Zeit auf dem Audio-Faden herunter -- derselbe Gedanke wie bei
//! `AUDIO_PLAY_AT`. `SPEAKING()` fragt nur, ob das geplante Ende schon
//! vorbei ist. Ein Konsolenprogramm, das nach `SPEAK` in `INPUT` haengt,
//! hoert seine drei Saetze deshalb trotzdem nacheinander.
//!
//! Was hier NICHT steht: die Frage "Bildschirmleser zuerst?" -- die
//! entscheidet vm.rs, weil nur die VM Graphics UND gui kennt.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::audio::Audio;

/// (Text, Stimme, Tempo in Hundertsteln) -> Sound-Slot.
type Schluessel = (String, String, i64);

/// Mehr als so viele synthetisierte Saetze werden nicht aufgehoben: 200 KB
/// je Satz sind bei 64 Saetzen rund 13 MB. Verdraengt wird der aelteste,
/// der nicht gerade gesprochen wird.
const CACHE_MAX: usize = 64;

pub struct Sprecher {
    /// Anzeigename der Stimme; leer = Systemstimme.
    stimme: String,
    /// 0.5 .. 2.0, 1.0 = normal.
    tempo: f64,
    cache: HashMap<Schluessel, i64>,
    /// Einfuegereihenfolge fuer die Verdraengung.
    alter: Vec<Schluessel>,
    /// Slots, die noch gesprochen werden (Slot, geplantes Ende) -- damit die
    /// Verdraengung keinen Klang freigibt, der gerade laeuft.
    laufend: Vec<(i64, Instant)>,
    /// Wann die letzte geplante Ansage endet. `None` = still.
    ende: Option<Instant>,
    synth: Option<plattform::Synth>,
}

impl Default for Sprecher {
    fn default() -> Self { Self::new() }
}

impl Sprecher {
    pub fn new() -> Self {
        Sprecher { stimme: String::new(), tempo: 1.0, cache: HashMap::new(), alter: Vec::new(),
                   laufend: Vec::new(), ende: None, synth: None }
    }

    /// SPEAK_VOICE(name$): leer = Systemstimme; sonst muss der Name (ohne
    /// Ruecksicht auf Gross/Klein) in `SPEAK_VOICES()` stehen -- ein
    /// Tippfehler soll hier auffallen, nicht als falsche Stimme.
    pub fn stimme_setzen(&mut self, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() { self.stimme.clear(); return Ok(()); }
        let alle = self.stimmen()?;
        match alle.iter().find(|s| s.eq_ignore_ascii_case(name)) {
            Some(s) => { self.stimme = s.clone(); Ok(()) }
            None => Err(format!("SPEAK_VOICE: Stimme '{}' nicht gefunden -- vorhanden: {}",
                                name, alle.join(", "))),
        }
    }

    /// SPEAK_VOICES(): die Anzeigenamen der installierten Stimmen.
    pub fn stimmen(&self) -> Result<Vec<String>, String> { plattform::stimmen() }

    /// SPEAK_RATE(faktor): 0.5 (halb so schnell) .. 2.0 (doppelt).
    pub fn tempo_setzen(&mut self, faktor: f64) -> Result<(), String> {
        if !(0.5..=2.0).contains(&faktor) {
            return Err(format!("SPEAK_RATE: faktor muss 0.5 .. 2.0 sein (ist {})", faktor));
        }
        self.tempo = faktor;
        Ok(())
    }

    /// SPEAK_SOUND(text$): nur synthetisieren, als SOUND zurueckgeben.
    /// Derselbe Text mit derselben Stimme und demselben Tempo kommt aus dem
    /// Vorrat -- es sei denn, das Programm hat den Slot per UNLOADSOUND
    /// freigegeben; dann wird neu gerechnet statt einen toten Slot zu liefern.
    pub fn klang(&mut self, au: &mut Audio, text: &str) -> Result<i64, String> {
        let key: Schluessel = (text.to_string(), self.stimme.clone(), (self.tempo * 100.0).round() as i64);
        if let Some(&idx) = self.cache.get(&key) {
            if au.sound_lebt(idx) { return Ok(idx); }
            self.cache.remove(&key);
            self.alter.retain(|k| k != &key);
        }
        let (mut samples, rate) = plattform::synthese(&mut self.synth, text, &self.stimme, self.tempo)?;
        stille_kuerzen(&mut samples, rate);
        if samples.is_empty() { return Err("SPEAK: die Sprachausgabe lieferte keinen Klang".into()); }
        let idx = au.sound_aus_pcm(&samples, rate);
        self.cache.insert(key.clone(), idx);
        self.alter.push(key);
        self.verdraengen(au);
        Ok(idx)
    }

    /// SPEAK(text$[, unterbrechen]): anhaengen (Vorgabe) oder das Laufende
    /// abbrechen und sofort sprechen. Leerer Text tut nichts.
    pub fn sprechen(&mut self, au: &mut Audio, text: &str, unterbrechen: bool) -> Result<(), String> {
        if text.trim().is_empty() { return Ok(()); }
        let idx = self.klang(au, text)?;
        let dauer = au.sound_dauer(idx)?;
        if unterbrechen { self.stopp(au); }
        let jetzt = Instant::now();
        self.laufend.retain(|(_, e)| *e > jetzt);
        let start = match self.ende { Some(e) if e > jetzt => e, _ => jetzt };
        au.speech_play(idx, start.saturating_duration_since(jetzt))?;
        let ende = start + dauer;
        self.ende = Some(ende);
        self.laufend.push((idx, ende));
        Ok(())
    }

    /// SPEAK_STOP(): alles Laufende und Geplante verwerfen.
    pub fn stopp(&mut self, au: &mut Audio) {
        au.speech_stop();
        self.laufend.clear();
        self.ende = None;
    }

    /// SPEAKING(): laeuft oder wartet noch eine Ansage?
    pub fn spricht(&self) -> bool {
        self.ende.map_or(false, |e| Instant::now() < e)
    }

    /// SPEAK_WAIT(): blockiert, bis alles gesprochen ist. Der Audio-Faden
    /// spielt derweil weiter; `update` haelt nur die Buchfuehrung nach.
    pub fn warten(&mut self, au: &mut Audio) {
        while self.spricht() {
            std::thread::sleep(Duration::from_millis(5));
            au.update();
        }
        self.laufend.clear();
    }

    fn verdraengen(&mut self, au: &mut Audio) {
        while self.cache.len() > CACHE_MAX {
            let jetzt = Instant::now();
            let belegt: Vec<i64> = self.laufend.iter().filter(|(_, e)| *e > jetzt).map(|(i, _)| *i).collect();
            let Some(pos) = self.alter.iter().position(|k| {
                self.cache.get(k).map_or(true, |i| !belegt.contains(i))
            }) else { return };
            let key = self.alter.remove(pos);
            if let Some(idx) = self.cache.remove(&key) { let _ = au.unload_sound(idx); }
        }
    }
}

// ---------------------------------------------------------------------------
// WAV lesen -- alle drei Systeme liefern eines (WinRT-Strom, `say`-Datei,
// `espeak-ng --stdout`). Kein Crate: der Leser ist 40 Zeilen und pruefbar.
// ---------------------------------------------------------------------------

fn u16_at(b: &[u8], i: usize) -> u16 { u16::from_le_bytes([b[i], b[i + 1]]) }
fn u32_at(b: &[u8], i: usize) -> u32 { u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) }

/// RIFF/WAVE -> (Mono-Abtastwerte -1..1, Abtastrate). PCM 8/16/24/32 Bit
/// und IEEE-Float 32 Bit; mehrere Kanaele werden gemittelt. Eine Datenlaenge,
/// die ueber das Ende hinausreicht (Stroeme schreiben sie nicht nach), gilt
/// bis zum Ende.
pub fn wav_lesen(b: &[u8]) -> Result<(Vec<f64>, u32), String> {
    if b.len() < 12 || &b[0..4] != b"RIFF" || &b[8..12] != b"WAVE" {
        return Err("SPEAK: die Sprachausgabe lieferte kein WAV".into());
    }
    let (mut format, mut kanaele, mut rate, mut bits) = (0u16, 0u16, 0u32, 0u16);
    let mut daten: Option<&[u8]> = None;
    let mut i = 12;
    while i + 8 <= b.len() {
        let kennung = &b[i..i + 4];
        let laenge = u32_at(b, i + 4) as usize;
        let anfang = i + 8;
        let ende = anfang.saturating_add(laenge).min(b.len());
        match kennung {
            b"fmt " => {
                if ende - anfang < 16 { return Err("SPEAK: WAV-Kopf zu kurz".into()); }
                format = u16_at(b, anfang);
                kanaele = u16_at(b, anfang + 2);
                rate = u32_at(b, anfang + 4);
                bits = u16_at(b, anfang + 14);
            }
            b"data" => { daten = Some(&b[anfang..ende]); break; }
            _ => {}
        }
        i = anfang.saturating_add(laenge).saturating_add(laenge & 1);
    }
    let daten = daten.ok_or("SPEAK: WAV ohne Daten")?;
    if kanaele == 0 || rate == 0 { return Err("SPEAK: WAV ohne Format".into()); }
    let bytes_je = (bits as usize) / 8;
    if bytes_je == 0 { return Err("SPEAK: WAV mit 0 Bit".into()); }
    let lese: fn(&[u8]) -> f64 = match (format, bits) {
        (1, 8) => |s| (s[0] as f64 - 128.0) / 128.0,
        (1, 16) => |s| i16::from_le_bytes([s[0], s[1]]) as f64 / 32768.0,
        (1, 24) => |s| (i32::from_le_bytes([0, s[0], s[1], s[2]]) >> 8) as f64 / 8388608.0,
        (1, 32) => |s| i32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f64 / 2147483648.0,
        (3, 32) => |s| f32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f64,
        _ => return Err(format!("SPEAK: WAV-Format {} mit {} Bit nicht lesbar", format, bits)),
    };
    let schritt = bytes_je * kanaele as usize;
    let n = daten.len() / schritt;
    let mut out = Vec::with_capacity(n);
    for f in 0..n {
        let mut summe = 0.0;
        for k in 0..kanaele as usize {
            let p = f * schritt + k * bytes_je;
            summe += lese(&daten[p..p + bytes_je]);
        }
        out.push((summe / kanaele as f64).clamp(-1.0, 1.0));
    }
    Ok((out, rate))
}

/// Die Stille am Ende kuerzen. Gemessen an WinRT: "Treffer!" ist nach 0,6 s
/// gesprochen, der Strom ist 1,3 s lang -- 0,7 s Schweigen, die bei jeder
/// angehaengten Ansage als Pause hoerbar waeren. Es bleiben 120 ms Nachlauf
/// (und vorn 30 ms), damit nichts abgeschnitten klingt.
fn stille_kuerzen(s: &mut Vec<f64>, rate: u32) {
    const SCHWELLE: f64 = 0.004;
    let rate = rate.max(1) as usize;
    let Some(letzte) = s.iter().rposition(|v| v.abs() > SCHWELLE) else { s.clear(); return };
    let ende = letzte + 1;
    let behalten = (ende + rate * 120 / 1000).min(s.len());
    s.truncate(behalten);
    let anfang = s.iter().position(|v| v.abs() > SCHWELLE).unwrap_or(0);
    let ab = anfang.saturating_sub(rate * 30 / 1000);
    if ab > 0 { s.drain(..ab); }
}

/// Woerter je Minute fuer `say`/`espeak-ng`: beide sprechen bei etwa 175
/// normal; der Faktor skaliert das.
#[allow(dead_code)]
fn woerter_je_minute(tempo: f64) -> u32 { (175.0 * tempo).round().clamp(80.0, 450.0) as u32 }

// ---------------------------------------------------------------------------
// Windows: WinRT. Liefert einen WAV-Strom (mono, 16 kHz, 16 Bit) -- die
// Stimmen der Systemsprache (hier Stefan, Katja, Hedda), nicht die alten
// SAPI-Desktopstimmen.
// ---------------------------------------------------------------------------
#[cfg(windows)]
mod plattform {
    use windows::core::HSTRING;
    use windows::Media::SpeechSynthesis::{SpeechSynthesizer, VoiceInformation};
    use windows::Storage::Streams::DataReader;

    pub struct Synth(SpeechSynthesizer);

    fn f(was: &str, e: windows::core::Error) -> String { format!("SPEAK: {} ({})", was, e.message()) }

    /// WinRT braucht einen initialisierten Faden. raylibs GLFW hat den
    /// Hauptfaden schon als STA angemeldet (dann meldet RoInitialize
    /// RPC_E_CHANGED_MODE, und das ist in Ordnung); ein Konsolenprogramm
    /// ohne Fenster hat noch nichts -- dafuer dieser Aufruf.
    fn anmelden() {
        use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};
        unsafe { let _ = RoInitialize(RO_INIT_MULTITHREADED); }
    }

    pub fn stimmen() -> Result<Vec<String>, String> {
        anmelden();
        let alle = SpeechSynthesizer::AllVoices().map_err(|e| f("Stimmen nicht lesbar", e))?;
        let n = alle.Size().map_err(|e| f("Stimmen nicht lesbar", e))?;
        let mut namen = Vec::with_capacity(n as usize);
        for i in 0..n {
            let v = alle.GetAt(i).map_err(|e| f("Stimme nicht lesbar", e))?;
            namen.push(v.DisplayName().map_err(|e| f("Stimme ohne Namen", e))?.to_string());
        }
        Ok(namen)
    }

    fn stimme_finden(name: &str) -> Result<VoiceInformation, String> {
        if name.is_empty() {
            return SpeechSynthesizer::DefaultVoice().map_err(|e| f("keine Systemstimme", e));
        }
        let alle = SpeechSynthesizer::AllVoices().map_err(|e| f("Stimmen nicht lesbar", e))?;
        let n = alle.Size().map_err(|e| f("Stimmen nicht lesbar", e))?;
        for i in 0..n {
            let v = alle.GetAt(i).map_err(|e| f("Stimme nicht lesbar", e))?;
            if v.DisplayName().map_err(|e| f("Stimme ohne Namen", e))?.to_string().eq_ignore_ascii_case(name) {
                return Ok(v);
            }
        }
        Err(format!("SPEAK: Stimme '{}' nicht gefunden", name))
    }

    pub fn synthese(synth: &mut Option<Synth>, text: &str, stimme: &str, tempo: f64)
        -> Result<(Vec<f64>, u32), String>
    {
        anmelden();
        if synth.is_none() {
            *synth = Some(Synth(SpeechSynthesizer::new().map_err(|e| f("keine Sprachausgabe auf diesem System", e))?));
        }
        let s = &synth.as_ref().unwrap().0;
        s.SetVoice(&stimme_finden(stimme)?).map_err(|e| f("Stimme nicht setzbar", e))?;
        s.Options().and_then(|o| o.SetSpeakingRate(tempo)).map_err(|e| f("Tempo nicht setzbar", e))?;
        let strom = s.SynthesizeTextToStreamAsync(&HSTRING::from(text))
            .and_then(|op| op.join())
            .map_err(|e| f("Synthese fehlgeschlagen", e))?;
        let groesse = strom.Size().map_err(|e| f("Strom ohne Groesse", e))?;
        if groesse > u32::MAX as u64 { return Err("SPEAK: Text zu lang".into()); }
        let eingang = strom.GetInputStreamAt(0).map_err(|e| f("Strom nicht lesbar", e))?;
        let leser = DataReader::CreateDataReader(&eingang).map_err(|e| f("Strom nicht lesbar", e))?;
        leser.LoadAsync(groesse as u32).and_then(|op| op.join()).map_err(|e| f("Strom nicht lesbar", e))?;
        let mut bytes = vec![0u8; groesse as usize];
        leser.ReadBytes(&mut bytes).map_err(|e| f("Strom nicht lesbar", e))?;
        super::wav_lesen(&bytes)
    }
}

// ---------------------------------------------------------------------------
// macOS: `say` schreibt PCM in eine Datei. Ungeprueft -- kein Mac hier.
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
mod plattform {
    use std::io::Write;
    use std::process::{Command, Stdio};

    pub struct Synth;

    pub fn stimmen() -> Result<Vec<String>, String> {
        let out = Command::new("say").arg("-v").arg("?").output()
            .map_err(|e| format!("SPEAK: 'say' nicht gefunden ({})", e))?;
        // "Anna                de_DE    # Hallo! ..." -- der Name endet vor
        // dem ersten Doppel-Leerzeichen (er darf selbst eines enthalten).
        Ok(String::from_utf8_lossy(&out.stdout).lines()
            .filter_map(|l| l.split("  ").next().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect())
    }

    pub fn synthese(_s: &mut Option<Synth>, text: &str, stimme: &str, tempo: f64)
        -> Result<(Vec<f64>, u32), String>
    {
        let datei = std::env::temp_dir().join(format!("dhrt_speak_{}.wav", std::process::id()));
        let mut c = Command::new("say");
        c.arg("-o").arg(&datei).arg("--file-format=WAVE").arg("--data-format=LEI16@22050")
         .arg("-r").arg(super::woerter_je_minute(tempo).to_string());
        if !stimme.is_empty() { c.arg("-v").arg(stimme); }
        // Der Text geht ueber stdin -- so kann er mit "-" beginnen.
        let mut kind = c.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn()
            .map_err(|e| format!("SPEAK: 'say' nicht gefunden ({})", e))?;
        if let Some(mut ein) = kind.stdin.take() { let _ = ein.write_all(text.as_bytes()); }
        let out = kind.wait_with_output().map_err(|e| format!("SPEAK: say: {}", e))?;
        if !out.status.success() {
            return Err(format!("SPEAK: say meldet: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        let bytes = std::fs::read(&datei).map_err(|e| format!("SPEAK: say schrieb keine Datei ({})", e))?;
        let _ = std::fs::remove_file(&datei);
        super::wav_lesen(&bytes)
    }
}

// ---------------------------------------------------------------------------
// Linux: `espeak-ng --stdout`. Ungeprueft -- hier nicht installiert.
// ---------------------------------------------------------------------------
#[cfg(all(unix, not(target_os = "macos"), not(target_os = "emscripten")))]
mod plattform {
    use std::io::Write;
    use std::process::{Command, Stdio};

    pub struct Synth;

    const FEHLT: &str = "SPEAK: 'espeak-ng' nicht gefunden -- installieren mit: sudo apt install espeak-ng";

    pub fn stimmen() -> Result<Vec<String>, String> {
        let out = Command::new("espeak-ng").arg("--voices").output().map_err(|_| FEHLT.to_string())?;
        // "Pty Language       Age/Gender VoiceName          File          Other Languages"
        Ok(String::from_utf8_lossy(&out.stdout).lines().skip(1)
            .filter_map(|l| l.split_whitespace().nth(3).map(String::from))
            .collect())
    }

    /// Ohne gewaehlte Stimme die Sprache der Umgebung (`LANG=de_DE.UTF-8`
    /// -> `de`); espeak-ng spraeche sonst Englisch.
    fn sprache_der_umgebung() -> String {
        for k in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(v) = std::env::var(k) {
                let s: String = v.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
                if s.len() >= 2 { return s.to_lowercase(); }
            }
        }
        "en".to_string()
    }

    pub fn synthese(_s: &mut Option<Synth>, text: &str, stimme: &str, tempo: f64)
        -> Result<(Vec<f64>, u32), String>
    {
        let mut c = Command::new("espeak-ng");
        c.arg("--stdout").arg("--stdin").arg("-s").arg(super::woerter_je_minute(tempo).to_string());
        c.arg("-v").arg(if stimme.is_empty() { sprache_der_umgebung() } else { stimme.to_string() });
        let mut kind = c.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
            .map_err(|_| FEHLT.to_string())?;
        if let Some(mut ein) = kind.stdin.take() { let _ = ein.write_all(text.as_bytes()); }
        let out = kind.wait_with_output().map_err(|e| format!("SPEAK: espeak-ng: {}", e))?;
        if !out.status.success() {
            return Err(format!("SPEAK: espeak-ng meldet: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        super::wav_lesen(&out.stdout)
    }
}

#[cfg(target_os = "emscripten")]
mod plattform {
    pub struct Synth;
    pub fn stimmen() -> Result<Vec<String>, String> { Err("SPEAK: im Web-Bau keine Sprachausgabe".into()) }
    pub fn synthese(_s: &mut Option<Synth>, _t: &str, _v: &str, _r: f64) -> Result<(Vec<f64>, u32), String> {
        Err("SPEAK: im Web-Bau keine Sprachausgabe".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(kanaele: u16, bits: u16, rate: u32, format: u16, daten: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFF");
        b.extend_from_slice(&((36 + daten.len()) as u32).to_le_bytes());
        b.extend_from_slice(b"WAVE");
        b.extend_from_slice(b"fmt ");
        b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&format.to_le_bytes());
        b.extend_from_slice(&kanaele.to_le_bytes());
        b.extend_from_slice(&rate.to_le_bytes());
        b.extend_from_slice(&(rate * kanaele as u32 * bits as u32 / 8).to_le_bytes());
        b.extend_from_slice(&(kanaele * bits / 8).to_le_bytes());
        b.extend_from_slice(&bits.to_le_bytes());
        b.extend_from_slice(b"data");
        b.extend_from_slice(&(daten.len() as u32).to_le_bytes());
        b.extend_from_slice(daten);
        b
    }

    #[test]
    fn sechzehn_bit_stereo_wird_gemittelt() {
        let mut d = Vec::new();
        for (l, r) in [(16384i16, -16384i16), (32767, 32767), (0, -32768)] {
            d.extend_from_slice(&l.to_le_bytes()); d.extend_from_slice(&r.to_le_bytes());
        }
        let (s, rate) = wav_lesen(&wav(2, 16, 16000, 1, &d)).unwrap();
        assert_eq!(rate, 16000);
        assert_eq!(s.len(), 3);
        assert!(s[0].abs() < 1e-9, "{}", s[0]);
        assert!((s[1] - 0.99997).abs() < 1e-3);
        assert!((s[2] + 0.5).abs() < 1e-9);
    }

    #[test]
    fn acht_bit_ist_vorzeichenlos() {
        let (s, _) = wav_lesen(&wav(1, 8, 8000, 1, &[128, 255, 0])).unwrap();
        assert!(s[0].abs() < 1e-9);
        assert!(s[1] > 0.99 && s[2] <= -1.0 + 1e-9);
    }

    #[test]
    fn float_und_unbekannter_block() {
        let mut d = Vec::new();
        for v in [0.25f32, -0.5] { d.extend_from_slice(&v.to_le_bytes()); }
        let mut b = wav(1, 32, 22050, 3, &d);
        // Einen fremden Block VOR "fmt " einschieben -- espeak und say
        // schreiben LIST/FLLR-Bloecke, die ein Leser ueberspringen muss.
        let mut fremd = Vec::new();
        fremd.extend_from_slice(b"LIST");
        fremd.extend_from_slice(&3u32.to_le_bytes());
        fremd.extend_from_slice(&[b'a', b'b', b'c', 0]);
        b.splice(12..12, fremd);
        let (s, rate) = wav_lesen(&b).unwrap();
        assert_eq!(rate, 22050);
        assert_eq!(s, vec![0.25, -0.5]);
    }

    #[test]
    fn datenlaenge_ueber_das_ende_hinaus_gilt_bis_zum_ende() {
        // `espeak-ng --stdout` kennt die Laenge beim Schreiben des Kopfs nicht.
        let mut b = wav(1, 16, 16000, 1, &[0, 0, 0, 64, 0, 0x40]);
        let p = b.len() - 6 - 4;
        b[p..p + 4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        let (s, _) = wav_lesen(&b).unwrap();
        assert_eq!(s.len(), 3);
        assert!((s[2] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn kein_wav_ist_ein_fehler_im_klartext() {
        assert!(wav_lesen(b"hallo welt, das ist kein wav").unwrap_err().contains("kein WAV"));
        assert!(wav_lesen(&wav(1, 12, 8000, 1, &[0, 0])).unwrap_err().contains("12 Bit"));
    }

    #[test]
    fn stille_am_ende_faellt_weg_nachlauf_bleibt() {
        // 0,5 s Ton, dann 1,0 s Stille bei 1000 Hz Abtastrate.
        let mut s: Vec<f64> = (0..500).map(|i| if i % 2 == 0 { 0.5 } else { -0.5 }).collect();
        s.extend(std::iter::repeat(0.0).take(1000));
        stille_kuerzen(&mut s, 1000);
        assert_eq!(s.len(), 500 + 120);
        // Vorn: 200 Proben Stille -> 30 bleiben.
        let mut v: Vec<f64> = vec![0.0; 200];
        v.extend((0..100).map(|_| 0.3));
        stille_kuerzen(&mut v, 1000);
        assert_eq!(v.len(), 30 + 100 + 0);
        // Nur Stille: bleibt leer (der Aufrufer meldet das).
        let mut leer = vec![0.0; 300];
        stille_kuerzen(&mut leer, 1000);
        assert!(leer.is_empty());
    }

    #[test]
    fn tempo_grenzen() {
        assert_eq!(woerter_je_minute(1.0), 175);
        assert_eq!(woerter_je_minute(2.0), 350);
        assert_eq!(woerter_je_minute(0.5), 88);
    }
}
