//! Native Audio ueber **Kira** (cpal) -- alternatives Backend zu `audio.rs`
//! (raylib). Aktiv mit `--features kira_audio`. Vorteil: ein eigener Audio-
//! Thread, vollstaendig vom Game-Loop entkoppelt (kein per-Frame-Refill ->
//! kein Stottern bei schweren Frames), echte Mixer-Tracks/Effekte, tweenbare
//! Lautstaerke/Pitch/Pan. MOD/XM via reinem Rust-Player (xmrs/xmrsplayer).
//!
//! Bietet exakt dieselbe `Audio`-API wie `audio.rs`, damit vm.rs nichts
//! wissen muss (Modul-Alias in main.rs schaltet per cfg um).
//!
//! Audio-Output ist -- wie RND/MILLIS/Tween -- naturgemaess nicht
//! deterministisch golden-testbar; getestet wird die reine DSP-Mathematik
//! (resample/lofi/pendulum) + Argument-Validierung.
#![cfg(feature = "kira_audio")]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend, Decibels, Frame, Panning, Tween,
    effect::{Effect, EffectBuilder},
    info::Info,
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    sound::{PlaybackState, streaming::{StreamingSoundData, StreamingSoundHandle}},
    sound::FromFileError,
    track::MainTrackBuilder,
};
use xmrs::prelude::Module;
use xmrsplayer::prelude::XmrsPlayer;

/// Woher die aktuelle Musik kommt: Stream-Datei (ogg/mp3/wav/flac) oder ein
/// zu PCM gerendertes Tracker-Modul (.mod/.xm/.s3m/.it). Beim (Neu-)Starten
/// baut start_music daraus die passende Kira-Sound-Quelle.
enum MusicSource {
    Stream(String),          // Pfad -- bei jedem Play frisch geoeffnet (streamt von Platte)
    Module(StaticSoundData), // ein gerenderter Loop-Durchlauf (im RAM, loopt via loop_region)
}

/// Laufende Musik-Instanz. Beide Handle-Typen teilen dieselbe Methoden-API
/// (state/position/pause/resume/stop/set_volume/set_playback_rate) -> das
/// `on_music!`-Makro ruft sie einheitlich.
enum MusicHandle {
    Stream(StreamingSoundHandle<FromFileError>),
    Static(StaticSoundHandle),
}

/// Rendert ein Tracker-Modul zu einem Loop-Durchlauf (mono->stereo PCM bei
/// 44100; Kira resampelt beim Abspielen aufs Geraet). Modul + Player sind
/// lokal -> kein Lifetime-Leak. 5-Minuten-Sicherheitskappe.
fn render_module(bytes: &[u8], fn_: &str) -> Result<StaticSoundData, String> {
    let sr: u32 = 44100;
    let module = Module::load(bytes).map_err(|e| format!("{}: {:?}", fn_, e))?;
    let mut player = XmrsPlayer::new(&module, sr, 0);
    player.set_max_loop_count(1);          // genau ein Durchlauf -> danach loopt Kira
    let cap = (sr as usize) * 60 * 5;
    let mut frames: Vec<Frame> = Vec::new();
    {
        let mut it = player.by_ref();
        while frames.len() < cap {
            match (it.next(), it.next()) {
                (Some(l), Some(r)) => frames.push(Frame {
                    left: l as f32 / 32768.0, right: r as f32 / 32768.0,
                }),
                _ => break,
            }
        }
    }
    if frames.is_empty() { return Err(format!("{}: Modul ergab kein Audio", fn_)); }
    Ok(StaticSoundData {
        sample_rate: sr, frames: frames.into(),
        settings: StaticSoundSettings::new(), slice: None,
    })
}

fn is_module_path(p: &str) -> bool {
    let l = p.to_lowercase();
    l.ends_with(".mod") || l.ends_with(".xm") || l.ends_with(".s3m") || l.ends_with(".it")
}

// Einheitliche Steuerung beider MusicHandle-Varianten (identische Methoden-API).
fn mh_stop(h: &mut MusicHandle, t: Tween) {
    match h { MusicHandle::Stream(x) => x.stop(t), MusicHandle::Static(x) => x.stop(t) }
}
fn mh_pause(h: &mut MusicHandle, t: Tween) {
    match h { MusicHandle::Stream(x) => x.pause(t), MusicHandle::Static(x) => x.pause(t) }
}
fn mh_resume(h: &mut MusicHandle, t: Tween) {
    match h { MusicHandle::Stream(x) => x.resume(t), MusicHandle::Static(x) => x.resume(t) }
}
fn mh_set_volume(h: &mut MusicHandle, d: Decibels, t: Tween) {
    match h { MusicHandle::Stream(x) => x.set_volume(d, t), MusicHandle::Static(x) => x.set_volume(d, t) }
}
fn mh_set_rate(h: &mut MusicHandle, r: f64, t: Tween) {
    match h { MusicHandle::Stream(x) => x.set_playback_rate(r, t), MusicHandle::Static(x) => x.set_playback_rate(r, t) }
}
fn mh_state(h: &MusicHandle) -> PlaybackState {
    match h { MusicHandle::Stream(x) => x.state(), MusicHandle::Static(x) => x.state() }
}

const TWO_PI: f32 = std::f32::consts::TAU;
const PI64: f64 = std::f64::consts::PI;
const RING: usize = 4096;     // Ringpuffer fuer Mono-Samples (vom Audio-Thread)
const FFT_N: usize = 1024;    // FFT-Fenster

// --- FFT-Tap -----------------------------------------------------------
// Statt raylibs AttachAudioMixedProcessor haengen wir einen Effect an den
// Main-Track. Sein `process` laeuft auf dem Audio-Thread und schiebt das
// gemischte Signal (mono) in einen globalen Ringpuffer; fft_bands liest ihn.
struct Ring { buf: [f32; RING], pos: usize }
static SAMPLES: Mutex<Ring> = Mutex::new(Ring { buf: [0.0; RING], pos: 0 });

struct FftTapBuilder;
struct FftTapEffect;
impl EffectBuilder for FftTapBuilder {
    type Handle = ();
    fn build(self) -> (Box<dyn Effect>, ()) { (Box::new(FftTapEffect), ()) }
}
impl Effect for FftTapEffect {
    fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info) {
        if let Ok(mut ring) = SAMPLES.try_lock() {
            let mut p = ring.pos;
            for f in input.iter() {
                ring.buf[p] = (f.left + f.right) * 0.5;
                p = (p + 1) % RING;
            }
            ring.pos = p;
        }
    }
}

/// Iterative Radix-2-FFT (in-place), N muss Zweierpotenz sein. (Verbatim aus
/// audio.rs -- gleiche Mathematik.)
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 { j ^= bit; bit >>= 1; }
        j ^= bit;
        if i < j { re.swap(i, j); im.swap(i, j); }
    }
    let mut len = 2;
    while len <= n {
        let ang = -TWO_PI / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        let mut i = 0;
        while i < n {
            let (mut cr, mut ci) = (1.0f32, 0.0f32);
            for k in 0..len / 2 {
                let a = i + k;
                let b = i + k + len / 2;
                let tr = re[b] * cr - im[b] * ci;
                let ti = re[b] * ci + im[b] * cr;
                re[b] = re[a] - tr; im[b] = im[a] - ti;
                re[a] += tr; im[a] += ti;
                let ncr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr; cr = ncr;
            }
            i += len;
        }
        len <<= 1;
    }
}

// --- Hilfs-Konvertierungen ---------------------------------------------

/// Lineare Lautstaerke 0..1 -> Kira-Dezibel. 0 -> Stille.
fn db(v: f64) -> Decibels {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.0 { Decibels::SILENCE } else { Decibels((20.0 * v.log10()) as f32) }
}

/// Pan-Position 0=links .. 0.5=Mitte .. 1=rechts -> Kira-Panning(-1..1).
fn pan_of(pos: f64) -> Panning { Panning((2.0 * pos.clamp(0.0, 1.0) - 1.0) as f32) }

fn tween_ms(ms: i64) -> Tween {
    Tween { duration: Duration::from_millis(ms.max(0) as u64), ..Default::default() }
}
fn tween_now() -> Tween { Tween { duration: Duration::from_millis(4), ..Default::default() } }

// --- Per-Sound-Slot (SFX/Tone/Sample) ----------------------------------

/// Ein "Channel" == ein geladener/gebauter Sound. `data` ist die (billig
/// klonbare) Sample-Quelle, `handle` die zuletzt gestartete Instanz.
struct SoundSlot {
    data: StaticSoundData,
    handle: Option<StaticSoundHandle>,
    vol: f32,                 // getrackte Lautstaerke (linear 0..1)
    loops: i64,               // verbleibende endliche Wiederholungen (>0); -1 endlos via loop_region; 0 keine
    pan_anim: Option<Pendulum>, // AUTOPAN (per-Frame); SLIDE laeuft als Kira-Tween
}

/// Ein geladenes PCM-Sample fuer den Amiga-Stil-Sampler (SAMPLE_*).
struct Sample {
    data: Vec<f32>,
    sr: u32,
    loop_start: usize,
    loop_end: usize,
}

/// AUTOPAN-Pendel (per-Frame in update() geschrieben).
struct Pendulum { period_s: f64, depth: f64, start: std::time::Instant }

/// Pendel-Position 0..1 (pure, fuer #[test]). Kosinus-foermig, startet links.
fn pendulum_pos(elapsed_s: f64, period_s: f64, depth: f64) -> f64 {
    let d = depth.clamp(0.0, 1.0);
    0.5 - 0.5 * d * (std::f64::consts::TAU * elapsed_s / period_s).cos()
}

// --- Musik (Stream) ----------------------------------------------------

pub struct Audio {
    manager: AudioManager<DefaultBackend>,
    sounds: Vec<SoundSlot>,
    samples: Vec<Sample>,
    sample_cache: HashMap<(usize, i64, i64), i64>,
    num_channels: i64,
    bands: Vec<f32>,
    agc: f32,
    lofi: bool,
    lofi_bits: u32,
    lofi_cutoff: f64,
    // Musik (genau eine gleichzeitig)
    music_source: Option<MusicSource>,
    music_handle: Option<MusicHandle>,
    music_vol: f32,
    music_pitch: f32,
    music_loops: i64,         // verbleibende Wiederholungen; -1 endlos
    music_queue: Option<String>,
    music_paused: bool,
}

impl Audio {
    pub fn new() -> Result<Audio, String> {
        let settings = AudioManagerSettings {
            main_track_builder: MainTrackBuilder::new().with_effect(FftTapBuilder),
            ..Default::default()
        };
        let manager = AudioManager::<DefaultBackend>::new(settings)
            .map_err(|e| format!("Audio-Geraet konnte nicht initialisiert werden: {e:?}"))?;
        Ok(Audio {
            manager, sounds: Vec::new(), samples: Vec::new(),
            sample_cache: HashMap::new(), num_channels: 16,
            bands: Vec::new(), agc: 1e-4,
            lofi: false, lofi_bits: 8, lofi_cutoff: 3300.0,
            music_source: None, music_handle: None, music_vol: 1.0, music_pitch: 1.0,
            music_loops: -1, music_queue: None, music_paused: false,
        })
    }

    // ---- intern: Sound aus Float-Buffer bauen + registrieren ----------

    fn make_data_mono(&self, buf: &[f64], vol: f64, sr: u32) -> StaticSoundData {
        let n = buf.len();
        let vol = vol.clamp(0.0, 1.0);
        let mut work;
        let src: &[f64] = if self.lofi {
            work = buf.to_vec();
            lofi_chain(&mut work, sr, self.lofi_bits, self.lofi_cutoff);
            &work
        } else { buf };
        let fade = ((sr as f64 * 0.005) as usize).min(n / 4);
        let frames: Arc<[Frame]> = (0..n).map(|i| {
            let mut e = 1.0;
            if fade > 0 {
                if i < fade { e = i as f64 / fade as f64; }
                else if i >= n - fade { e = (n - 1 - i) as f64 / fade as f64; }
            }
            let s = (src[i] * e * vol).clamp(-1.0, 1.0) as f32;
            Frame { left: s, right: s }
        }).collect();
        StaticSoundData { sample_rate: sr, frames, settings: StaticSoundSettings::new(), slice: None }
    }

    fn make_data_stereo(&self, left: &[f64], right: &[f64], vol: f64, sr: u32) -> StaticSoundData {
        let n = left.len();
        let vol = vol.clamp(0.0, 1.0);
        let (mut wl, mut wr);
        let (l, r): (&[f64], &[f64]) = if self.lofi {
            wl = left.to_vec(); wr = right.to_vec();
            lofi_chain(&mut wl, sr, self.lofi_bits, self.lofi_cutoff);
            lofi_chain(&mut wr, sr, self.lofi_bits, self.lofi_cutoff);
            (&wl, &wr)
        } else { (left, right) };
        let fade = ((sr as f64 * 0.005) as usize).min(n / 4);
        let frames: Arc<[Frame]> = (0..n).map(|i| {
            let mut e = 1.0;
            if fade > 0 {
                if i < fade { e = i as f64 / fade as f64; }
                else if i >= n - fade { e = (n - 1 - i) as f64 / fade as f64; }
            }
            Frame {
                left: (l[i] * e * vol).clamp(-1.0, 1.0) as f32,
                right: (r[i] * e * vol).clamp(-1.0, 1.0) as f32,
            }
        }).collect();
        StaticSoundData { sample_rate: sr, frames, settings: StaticSoundSettings::new(), slice: None }
    }

    fn push_slot(&mut self, data: StaticSoundData, vol: f32) -> i64 {
        self.sounds.push(SoundSlot { data, handle: None, vol, loops: 0, pan_anim: None });
        (self.sounds.len() - 1) as i64
    }

    fn slot(&self, idx: i64, fn_: &str) -> Result<&SoundSlot, String> {
        self.sounds.get(idx as usize)
            .ok_or_else(|| format!("{}: ungueltiges Handle {}", fn_, idx))
    }
    fn slot_mut(&mut self, idx: i64, fn_: &str) -> Result<&mut SoundSlot, String> {
        self.sounds.get_mut(idx as usize)
            .ok_or_else(|| format!("{}: ungueltiges Handle {}", fn_, idx))
    }

    /// Startet einen Slot neu (stoppt die alte Instanz) mit gegebener
    /// Lautstaerke/Loop/Fade. Gemeinsamer Kern fuer PLAYSOUND + AUDIO_PLAY.
    fn start_slot(&mut self, idx: i64, fn_: &str, vol: f64, loops: i64, fade_in_ms: i64) -> Result<(), String> {
        let vol = vol.clamp(0.0, 1.0);
        // alte Instanz stoppen
        if let Some(h) = self.slot_mut(idx, fn_)?.handle.as_mut() { h.stop(tween_now()); }
        let mut settings = StaticSoundSettings::new();
        if loops < 0 { settings = settings.loop_region(0.0..); }
        settings = settings.volume(if fade_in_ms > 0 { Decibels::SILENCE } else { db(vol) });
        let data = self.slot(idx, fn_)?.data.clone().with_settings(settings);
        let mut handle = self.manager.play(data)
            .map_err(|e| format!("{}: {:?}", fn_, e))?;
        if fade_in_ms > 0 { handle.set_volume(db(vol), tween_ms(fade_in_ms)); }
        let s = self.slot_mut(idx, fn_)?;
        s.handle = Some(handle);
        s.vol = vol as f32;
        s.loops = if loops < 0 { -1 } else { loops };
        s.pan_anim = None;
        Ok(())
    }

    // ================= FFT =================
    /// Fuellt `out` mit B logarithmisch verteilten Band-Pegeln (0..1).
    /// (Mathematik verbatim aus audio.rs.)
    pub fn fft_bands(&mut self, out: &mut [f32]) {
        let b = out.len();
        if b == 0 { return; }
        if self.bands.len() != b { self.bands = vec![0.0; b]; }
        let mut re = [0.0f32; FFT_N];
        let mut im = [0.0f32; FFT_N];
        {
            let ring = SAMPLES.lock().unwrap_or_else(|e| e.into_inner());
            let start = (ring.pos + RING - FFT_N) % RING;
            for i in 0..FFT_N {
                let s = ring.buf[(start + i) % RING];
                let w = 0.5 - 0.5 * (TWO_PI * i as f32 / (FFT_N as f32 - 1.0)).cos();
                re[i] = s * w;
            }
        }
        fft(&mut re, &mut im);
        let half = FFT_N / 2;
        let maxbin = half as f32;
        let mut framemax = 0.0f32;
        let mut raw = vec![0.0f32; b];
        for j in 0..b {
            let lo = (maxbin.powf(j as f32 / b as f32)) as usize;
            let hi = (maxbin.powf((j + 1) as f32 / b as f32)) as usize;
            let lo = lo.max(1);
            let hi = hi.max(lo + 1).min(half);
            let mut sum = 0.0f32;
            for k in lo..hi { sum += (re[k] * re[k] + im[k] * im[k]).sqrt(); }
            raw[j] = sum / (hi - lo) as f32;
            if raw[j] > framemax { framemax = raw[j]; }
        }
        self.agc = (self.agc * 0.995).max(framemax).max(1e-4);
        for j in 0..b {
            let norm = (raw[j] / self.agc).clamp(0.0, 1.0).sqrt();
            self.bands[j] = norm.max(self.bands[j] * 0.80);
            out[j] = self.bands[j];
        }
    }

    // ================= Lo-Fi =================
    pub fn set_lofi(&mut self, on: bool, bits: u32, cutoff: f64) {
        self.lofi = on;
        self.lofi_bits = bits.clamp(1, 16);
        self.lofi_cutoff = cutoff.max(0.0);
        self.sample_cache.clear();
    }

    // ================= Core SFX =================
    pub fn load_sound(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let data = StaticSoundData::from_file(&resolved)
            .map_err(|e| format!("LOADSOUND: {:?}", e))?;
        Ok(self.push_slot(data, 1.0))
    }

    pub fn play_sound(&mut self, idx: i64, volume: f64) -> Result<(), String> {
        self.start_slot(idx, "PLAYSOUND", volume, 0, 0)
    }

    pub fn stop_sound(&mut self, idx: i64) -> Result<(), String> {
        if let Some(h) = self.slot_mut(idx, "STOPSOUND")?.handle.as_mut() { h.stop(tween_now()); }
        Ok(())
    }

    // ================= Tone-Generation =================
    pub fn tone(&mut self, freq: f64, dur_ms: i64, waveform: &str, volume: f64) -> Result<i64, String> {
        if freq <= 0.0 { return Err("AUDIO_TONE: freq_hz muss > 0 sein".into()); }
        if dur_ms <= 0 { return Err("AUDIO_TONE: dauer_ms muss > 0 sein".into()); }
        let wf = waveform.to_lowercase();
        if !matches!(wf.as_str(), "sine" | "square" | "saw" | "triangle" | "noise") {
            return Err(format!(
                "AUDIO_TONE: unbekannte Waveform '{}' (erlaubt: sine, square, saw, triangle, noise)", waveform));
        }
        let sr: u32 = 44100;
        let n = (sr as f64 * dur_ms as f64 / 1000.0) as usize;
        if n == 0 { return Err("AUDIO_TONE: dauer_ms zu klein fuer Sample-Rate".into()); }
        let mut buf = vec![0.0f64; n];
        for (i, b) in buf.iter_mut().enumerate() {
            *b = waveform_value(&wf, freq, i as f64 / sr as f64);
        }
        let data = self.make_data_mono(&buf, volume, sr);
        Ok(self.push_slot(data, volume.clamp(0.0, 1.0) as f32))
    }

    pub fn noise(&mut self, dur_ms: i64, volume: f64) -> Result<i64, String> {
        if dur_ms <= 0 { return Err("AUDIO_NOISE: dauer_ms muss > 0 sein".into()); }
        let sr: u32 = 44100;
        let n = (sr as f64 * dur_ms as f64 / 1000.0) as usize;
        if n == 0 { return Err("AUDIO_NOISE: dauer_ms zu klein fuer Sample-Rate".into()); }
        let mut buf = vec![0.0f64; n];
        for b in buf.iter_mut() { *b = rng_uniform(); }
        let data = self.make_data_mono(&buf, volume, sr);
        Ok(self.push_slot(data, volume.clamp(0.0, 1.0) as f32))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sfx(&mut self, waveform: &str, base_freq: f64, slide: f64,
               attack_ms: i64, sustain_ms: i64, decay_ms: i64,
               vib_depth: f64, vib_speed: f64, volume: f64,
               stereo_width: f64) -> Result<i64, String> {
        let wf = waveform.to_lowercase();
        if !matches!(wf.as_str(), "sine" | "square" | "saw" | "triangle" | "noise") {
            return Err(format!(
                "AUDIO_SFX: unbekannte Waveform '{}' (erlaubt: sine, square, saw, triangle, noise)", waveform));
        }
        if attack_ms < 0 || sustain_ms < 0 || decay_ms < 0 {
            return Err("AUDIO_SFX: Attack/Sustain/Decay muessen >= 0 sein".into());
        }
        let total_ms = (attack_ms + sustain_ms + decay_ms).max(1);
        let sr: u32 = 44100;
        let n = (sr as f64 * total_ms as f64 / 1000.0) as usize;
        if n == 0 { return Err("AUDIO_SFX: Gesamtdauer zu klein".into()); }
        let na = (n as f64 * attack_ms as f64 / total_ms as f64) as usize;
        let nd = (n as f64 * decay_ms as f64 / total_ms as f64) as usize;
        let left = build_sfx_buffer(&wf, base_freq, slide, n, na, nd, vib_depth, vib_speed, sr);
        let data = if stereo_width <= 0.0 {
            self.make_data_mono(&left, volume, sr)
        } else {
            let w = stereo_width.min(1.0);
            let right = if wf == "noise" {
                build_sfx_buffer(&wf, base_freq, slide, n, na, nd, vib_depth, vib_speed, sr)
            } else {
                let detune = 1.0 + 0.04 * w;
                build_sfx_buffer(&wf, base_freq * detune, slide * detune, n, na, nd, vib_depth, vib_speed, sr)
            };
            self.make_data_stereo(&left, &right, volume, sr)
        };
        Ok(self.push_slot(data, volume.clamp(0.0, 1.0) as f32))
    }

    // ================= Sampler (SAMPLE_*) =================
    pub fn sample_load(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        // Kira dekodiert die Datei; wir lesen die Frames als mono f32.
        let data = StaticSoundData::from_file(&resolved)
            .map_err(|e| format!("SAMPLE_LOAD: {:?}", e))?;
        let sr = data.sample_rate;
        let mono: Vec<f32> = data.frames.iter().map(|f| (f.left + f.right) * 0.5).collect();
        if mono.is_empty() { return Err("SAMPLE_LOAD: Sample ist leer".into()); }
        self.samples.push(Sample { data: mono, sr, loop_start: 0, loop_end: 0 });
        Ok((self.samples.len() - 1) as i64)
    }

    pub fn sample_set_loop(&mut self, idx: i64, start: i64, end: i64) -> Result<(), String> {
        let s = self.samples.get_mut(idx as usize)
            .ok_or_else(|| format!("SAMPLE_SET_LOOP: ungueltiges SAMPLE-Handle {}", idx))?;
        let n = s.data.len();
        let start = start.max(0) as usize;
        let end = (end.max(0) as usize).min(n);
        if end <= start { return Err("SAMPLE_SET_LOOP: end muss > start sein".into()); }
        s.loop_start = start; s.loop_end = end;
        self.sample_cache.retain(|k, _| k.0 != idx as usize);
        Ok(())
    }

    pub fn sample_len(&self, idx: i64) -> Result<f64, String> {
        let s = self.samples.get(idx as usize)
            .ok_or_else(|| format!("SAMPLE_LEN: ungueltiges SAMPLE-Handle {}", idx))?;
        Ok(s.data.len() as f64 / s.sr as f64)
    }

    pub fn sample_play(&mut self, sidx: i64, semitones: f64, volume: f64,
                       dur_ms: i64) -> Result<i64, String> {
        let si = sidx as usize;
        if si >= self.samples.len() {
            return Err(format!("SAMPLE_PLAY: ungueltiges SAMPLE-Handle {}", sidx));
        }
        let ratio = 2f64.powf(semitones / 12.0);
        if !ratio.is_finite() || ratio <= 0.0 {
            return Err("SAMPLE_PLAY: ungueltige Tonhoehe".into());
        }
        let v = volume.clamp(0.0, 1.0);
        let key = (si, (semitones * 100.0).round() as i64, dur_ms);
        if let Some(&snd) = self.sample_cache.get(&key) {
            return self.start_slot(snd, "SAMPLE_PLAY", v, 0, 0).map(|_| snd);
        }
        let sr = self.samples[si].sr;
        let s = &self.samples[si];
        let buf = resample(&s.data, s.sr, ratio, dur_ms, s.loop_start, s.loop_end);
        if buf.is_empty() { return Err("SAMPLE_PLAY: leeres Sample".into()); }
        // Volume nicht in den Cache backen -> bei vol=1.0 bauen, per Slot setzen.
        let data = self.make_data_mono(&buf, 1.0, sr);
        let snd = self.push_slot(data, 1.0);
        self.sample_cache.insert(key, snd);
        self.start_slot(snd, "SAMPLE_PLAY", v, 0, 0)?;
        Ok(snd)
    }

    // ================= Channel-Steuerung (AUDIO_*) =================
    pub fn ch_play(&mut self, idx: i64, loops: i64, volume: f64, fade_in_ms: i64) -> Result<i64, String> {
        self.start_slot(idx, "AUDIO_PLAY", volume, loops, fade_in_ms)?;
        Ok(idx)
    }
    pub fn ch_pause(&mut self, idx: i64) -> Result<(), String> {
        if let Some(h) = self.slot_mut(idx, "AUDIO_PAUSE")?.handle.as_mut() { h.pause(tween_now()); }
        Ok(())
    }
    pub fn ch_resume(&mut self, idx: i64) -> Result<(), String> {
        if let Some(h) = self.slot_mut(idx, "AUDIO_RESUME")?.handle.as_mut() { h.resume(tween_now()); }
        Ok(())
    }
    pub fn ch_stop(&mut self, idx: i64, fade_out_ms: i64) -> Result<(), String> {
        let s = self.slot_mut(idx, "AUDIO_STOP")?;
        s.loops = 0;
        if let Some(h) = s.handle.as_mut() {
            h.stop(if fade_out_ms > 0 { tween_ms(fade_out_ms) } else { tween_now() });
        }
        Ok(())
    }
    pub fn ch_is_playing(&self, idx: i64) -> Result<bool, String> {
        Ok(matches!(self.slot(idx, "AUDIO_IS_PLAYING")?.handle.as_ref().map(|h| h.state()),
                    Some(PlaybackState::Playing)))
    }
    pub fn ch_set_volume(&mut self, idx: i64, v: f64) -> Result<(), String> {
        let vol = v.clamp(0.0, 1.0);
        let s = self.slot_mut(idx, "AUDIO_VOLUME")?;
        s.vol = vol as f32;
        if let Some(h) = s.handle.as_mut() { h.set_volume(db(vol), tween_now()); }
        Ok(())
    }
    pub fn ch_get_volume(&self, idx: i64) -> Result<f64, String> {
        Ok(self.slot(idx, "AUDIO_GET_VOLUME")?.vol as f64)
    }
    /// AUDIO_PITCH(ch, faktor) -- Abspielgeschwindigkeit/Tonhoehe (1.0 normal,
    /// 2.0 Oktave hoeher, 0.5 tiefer). Kira braucht &mut auf dem Handle; die VM
    /// ruft ueber audio_mut() (=&mut Audio), daher ist die Signatur &mut self.
    pub fn ch_pitch(&mut self, idx: i64, factor: f64) -> Result<(), String> {
        if let Some(h) = self.slot_mut(idx, "AUDIO_PITCH")?.handle.as_mut() {
            h.set_playback_rate(factor, tween_now());
        }
        Ok(())
    }
    pub fn ch_pan(&mut self, idx: i64, left: f64, right: f64) -> Result<(), String> {
        let l = left.clamp(0.0, 1.0);
        let r = right.clamp(0.0, 1.0);
        let vol = l.max(r);
        let pos = if l + r > 0.0 { r / (l + r) } else { 0.5 };  // 0=links,1=rechts
        let s = self.slot_mut(idx, "AUDIO_PAN")?;
        s.vol = vol as f32;
        s.pan_anim = None;
        if let Some(h) = s.handle.as_mut() {
            h.set_volume(db(vol), tween_now());
            h.set_panning(pan_of(pos), tween_now());
        }
        Ok(())
    }
    pub fn ch_pan_pos(&mut self, idx: i64, p: f64) -> Result<(), String> {
        let s = self.slot_mut(idx, "AUDIO_PAN_POS")?;
        s.pan_anim = None;
        if let Some(h) = s.handle.as_mut() { h.set_panning(pan_of(p), tween_now()); }
        Ok(())
    }
    pub fn ch_pan_slide(&mut self, idx: i64, from: f64, to: f64, dur_ms: i64) -> Result<(), String> {
        let (from, to) = (from.clamp(0.0, 1.0), to.clamp(0.0, 1.0));
        let s = self.slot_mut(idx, "AUDIO_PAN_SLIDE")?;
        s.pan_anim = None;
        if let Some(h) = s.handle.as_mut() {
            h.set_panning(pan_of(from), tween_now());           // Startpunkt
            h.set_panning(pan_of(to), tween_ms(dur_ms));        // Kira tweent nativ
        }
        Ok(())
    }
    pub fn ch_autopan(&mut self, idx: i64, period_s: f64, depth: f64) -> Result<(), String> {
        let s = self.slot_mut(idx, "AUDIO_AUTOPAN")?;
        s.pan_anim = if period_s <= 0.0 { None } else {
            Some(Pendulum { period_s, depth: depth.clamp(0.0, 1.0), start: std::time::Instant::now() })
        };
        Ok(())
    }

    pub fn pause_all(&mut self) {
        for s in &mut self.sounds { if let Some(h) = s.handle.as_mut() { h.pause(tween_now()); } }
    }
    pub fn resume_all(&mut self) {
        for s in &mut self.sounds { if let Some(h) = s.handle.as_mut() { h.resume(tween_now()); } }
    }
    pub fn stop_all(&mut self) {
        for s in &mut self.sounds {
            s.loops = 0; s.pan_anim = None;
            if let Some(h) = s.handle.as_mut() { h.stop(tween_now()); }
        }
    }
    pub fn set_num_channels(&mut self, n: i64) { self.num_channels = n.max(0); }
    pub fn get_num_channels(&self) -> i64 { self.num_channels }
    pub fn busy_channels(&self) -> i64 {
        self.sounds.iter().filter(|s| matches!(s.handle.as_ref().map(|h| h.state()), Some(PlaybackState::Playing))).count() as i64
    }

    // ================= Musik =================
    // Stream-Formate (ogg/mp3/wav/flac) streamen von Platte; Tracker-Module
    // (.mod/.xm/.s3m/.it) werden beim Laden EINMAL zu PCM gerendert (ein
    // Loop-Durchlauf) und im RAM via loop_region nahtlos geloopt -> volle
    // Steuerung (Volume/Pitch/Pause/Position) gratis ueber den Sound-Handle.

    pub fn music_load(&mut self, path: &str) -> Result<(), String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let source = if is_module_path(&resolved) {
            let bytes = std::fs::read(&resolved)
                .map_err(|e| format!("AUDIO_MUSIC_LOAD: {}", e))?;
            MusicSource::Module(render_module(&bytes, "AUDIO_MUSIC_LOAD")?)
        } else {
            // Stream testweise oeffnen (Fehler frueh melden).
            StreamingSoundData::from_file(&resolved)
                .map_err(|e| format!("AUDIO_MUSIC_LOAD: {:?}", e))?;
            MusicSource::Stream(resolved)
        };
        if let Some(h) = self.music_handle.as_mut() { mh_stop(h, tween_now()); }
        self.music_handle = None;
        self.music_source = Some(source);
        self.music_queue = None;
        self.music_loops = -1;
        self.music_paused = false;
        Ok(())
    }

    fn start_music(&mut self, vol: f32, loops: i64, fade_in_ms: i64) -> Result<(), String> {
        if let Some(h) = self.music_handle.as_mut() { mh_stop(h, tween_now()); }
        let vol_db = if fade_in_ms > 0 { Decibels::SILENCE } else { db(vol as f64) };
        let pitch = self.music_pitch as f64;
        let endless = loops < 0;
        let mut handle = match self.music_source.as_ref() {
            None => return Ok(()),
            Some(MusicSource::Stream(path)) => {
                let mut data = StreamingSoundData::from_file(path)
                    .map_err(|e| format!("AUDIO_MUSIC_PLAY: {:?}", e))?;
                if endless { data = data.loop_region(0.0..); }
                data = data.volume(vol_db).playback_rate(pitch);
                MusicHandle::Stream(self.manager.play(data)
                    .map_err(|e| format!("AUDIO_MUSIC_PLAY: {:?}", e))?)
            }
            Some(MusicSource::Module(rendered)) => {
                let mut settings = StaticSoundSettings::new().volume(vol_db).playback_rate(pitch);
                if endless { settings = settings.loop_region(0.0..); }
                let data = rendered.clone().with_settings(settings);
                MusicHandle::Static(self.manager.play(data)
                    .map_err(|e| format!("AUDIO_MUSIC_PLAY: {:?}", e))?)
            }
        };
        if fade_in_ms > 0 { mh_set_volume(&mut handle, db(vol as f64), tween_ms(fade_in_ms)); }
        self.music_handle = Some(handle);
        self.music_loops = if endless { -1 } else { loops };
        self.music_paused = false;
        Ok(())
    }

    pub fn play_music(&mut self, path: &str, volume: f64) -> Result<(), String> {
        self.music_vol = volume.clamp(0.0, 1.0) as f32;
        self.music_load(path)?;
        self.start_music(self.music_vol, -1, 0)
    }
    pub fn stop_music(&mut self) {
        if let Some(h) = self.music_handle.as_mut() { mh_stop(h, tween_now()); }
        self.music_handle = None;
    }
    pub fn music_play(&mut self, loops: i64, fade_in_ms: i64) {
        let v = self.music_vol;
        let _ = self.start_music(v, loops, fade_in_ms);
    }
    pub fn music_stop(&mut self, fade_out_ms: i64) {
        if let Some(h) = self.music_handle.as_mut() {
            mh_stop(h, if fade_out_ms > 0 { tween_ms(fade_out_ms) } else { tween_now() });
        }
        if fade_out_ms <= 0 { self.music_handle = None; }
    }
    pub fn music_pause(&mut self) {
        if let Some(h) = self.music_handle.as_mut() { mh_pause(h, tween_now()); self.music_paused = true; }
    }
    pub fn music_resume(&mut self) {
        if let Some(h) = self.music_handle.as_mut() { mh_resume(h, tween_now()); self.music_paused = false; }
    }
    pub fn music_set_volume(&mut self, v: f64) {
        self.music_vol = v.clamp(0.0, 1.0) as f32;
        let d = db(self.music_vol as f64);
        if let Some(h) = self.music_handle.as_mut() { mh_set_volume(h, d, tween_now()); }
    }
    pub fn music_get_volume(&self) -> f64 { self.music_vol as f64 }
    pub fn music_set_pitch(&mut self, factor: f64) {
        self.music_pitch = factor.max(0.0) as f32;
        let r = self.music_pitch as f64;
        if let Some(h) = self.music_handle.as_mut() { mh_set_rate(h, r, tween_now()); }
    }
    pub fn music_get_pitch(&self) -> f64 { self.music_pitch as f64 }
    pub fn music_position(&self) -> f64 {
        match self.music_handle.as_ref() {
            Some(MusicHandle::Stream(h)) => h.position(),
            Some(MusicHandle::Static(h)) => h.position(),
            None => 0.0,
        }
    }
    pub fn music_busy(&self) -> bool {
        let st = match self.music_handle.as_ref() {
            Some(MusicHandle::Stream(h)) => Some(h.state()),
            Some(MusicHandle::Static(h)) => Some(h.state()),
            None => None,
        };
        matches!(st, Some(PlaybackState::Playing))
    }
    pub fn music_queue(&mut self, path: &str) { self.music_queue = Some(path.to_string()); }

    // ================= per-Frame-Update (aus FLIP) =================
    pub fn update(&mut self) {
        // AUTOPAN: Pendel-Position pro Frame schreiben.
        for s in &mut self.sounds {
            if let Some(p) = &s.pan_anim {
                let pos = pendulum_pos(p.start.elapsed().as_secs_f64(), p.period_s, p.depth);
                if let Some(h) = s.handle.as_mut() { h.set_panning(pan_of(pos), tween_now()); }
            }
        }
        // Endliche SFX-Loops: gestoppte Slots mit Restwiederholungen neu starten.
        let n = self.sounds.len();
        for i in 0..n {
            let restart = {
                let s = &self.sounds[i];
                s.loops > 0 && matches!(s.handle.as_ref().map(|h| h.state()), Some(PlaybackState::Stopped))
            };
            if restart {
                let (vol, rem) = { let s = &self.sounds[i]; (s.vol as f64, s.loops - 1) };
                let _ = self.start_slot(i as i64, "AUDIO_PLAY", vol, 0, 0);
                self.sounds[i].loops = rem;
            }
        }
        // Musik: endliche Loops + Queue.
        let music_ended = matches!(self.music_handle.as_ref().map(mh_state), Some(PlaybackState::Stopped))
            && !self.music_paused;
        if music_ended {
            if self.music_loops > 0 {
                self.music_loops -= 1;
                let v = self.music_vol;
                let _ = self.start_music(v, 0, 0);
            } else if let Some(path) = self.music_queue.take() {
                let v = self.music_vol;
                // Queue-Track laden (Stream oder Modul) und endlos starten.
                if self.music_load(&path).is_ok() { let _ = self.start_music(v, -1, 0); }
            }
        }
    }
}

// ======================================================================
// Pure DSP-Helfer (verbatim aus audio.rs -- gleiche Mathematik)
// ======================================================================

fn waveform_value(kind: &str, freq: f64, t: f64) -> f64 {
    match kind {
        "sine" => (2.0 * PI64 * freq * t).sin(),
        "square" => if (2.0 * PI64 * freq * t).sin() >= 0.0 { 1.0 } else { -1.0 },
        "saw" => 2.0 * (t * freq - (0.5 + t * freq).floor()),
        "triangle" => 2.0 * (2.0 * (t * freq - (0.5 + t * freq).floor())).abs() - 1.0,
        "noise" => rng_uniform(),
        _ => 0.0,
    }
}

fn phase_value(kind: &str, phase: f64) -> f64 {
    let ph = phase / (2.0 * PI64);
    match kind {
        "sine" => phase.sin(),
        "square" => if phase.sin() >= 0.0 { 1.0 } else { -1.0 },
        "saw" => 2.0 * (ph - (0.5 + ph).floor()),
        "triangle" => 2.0 * (2.0 * (ph - (0.5 + ph).floor())).abs() - 1.0,
        _ => 0.0,
    }
}

thread_local! {
    static ARNG: std::cell::Cell<u64> = std::cell::Cell::new({
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x9E3779B9) | 1
    });
}
fn rng_uniform() -> f64 {
    ARNG.with(|s| {
        let mut x = s.get();
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        s.set(x);
        ((x >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    })
}

#[allow(clippy::too_many_arguments)]
fn build_sfx_buffer(wf: &str, base_freq: f64, slide: f64, n: usize,
                    na: usize, nd: usize, vib_depth: f64, vib_speed: f64,
                    sr: u32) -> Vec<f64> {
    let mut buf = vec![0.0f64; n];
    let mut phase = 0.0f64;
    for i in 0..n {
        let t = i as f64 / sr as f64;
        let mut freq = base_freq + slide * t;
        if vib_depth > 0.0 && vib_speed > 0.0 {
            freq *= 1.0 + vib_depth * (2.0 * PI64 * vib_speed * t).sin();
        }
        freq = freq.clamp(20.0, sr as f64 / 2.0);
        let v = if wf == "noise" {
            rng_uniform()
        } else {
            phase += 2.0 * PI64 * freq / sr as f64;
            phase_value(wf, phase)
        };
        let env = if na > 0 && i < na {
            i as f64 / na as f64
        } else if nd > 0 && i >= n - nd {
            (n - 1 - i) as f64 / nd as f64
        } else { 1.0 };
        buf[i] = (v * env).clamp(-1.0, 1.0);
    }
    buf
}

fn resample(data: &[f32], sr: u32, ratio: f64, dur_ms: i64,
            loop_start: usize, loop_end: usize) -> Vec<f64> {
    let n = data.len();
    if n == 0 { return Vec::new(); }
    let lerp = |pos: f64| -> f64 {
        let i = pos.floor() as usize;
        if i + 1 >= n { return *data.get(i.min(n - 1)).unwrap_or(&0.0) as f64; }
        let frac = pos - i as f64;
        (data[i] as f64) * (1.0 - frac) + (data[i + 1] as f64) * frac
    };
    if dur_ms <= 0 {
        let out_len = ((n as f64) / ratio).floor() as usize;
        let mut out = Vec::with_capacity(out_len);
        let mut pos = 0.0;
        for _ in 0..out_len { out.push(lerp(pos)); pos += ratio; }
        out
    } else {
        let out_len = (sr as f64 * dur_ms as f64 / 1000.0) as usize;
        let loop_on = loop_end > loop_start;
        let (ls, le) = (loop_start as f64, loop_end as f64);
        let mut out = Vec::with_capacity(out_len);
        let mut pos = 0.0;
        for _ in 0..out_len {
            if loop_on && pos >= le {
                let span = le - ls;
                pos = ls + ((pos - ls) % span);
            } else if !loop_on && pos >= n as f64 {
                out.push(0.0);
                continue;
            }
            out.push(lerp(pos));
            pos += ratio;
        }
        out
    }
}

fn lofi_chain(buf: &mut [f64], sr: u32, bits: u32, cutoff: f64) {
    if bits >= 1 && bits < 24 {
        let levels = (1u64 << bits) as f64;
        let half = levels / 2.0;
        for s in buf.iter_mut() {
            *s = ((*s * half).round() / half).clamp(-1.0, 1.0);
        }
    }
    if cutoff > 0.0 && sr > 0 {
        let dt = 1.0 / sr as f64;
        let rc = 1.0 / (2.0 * PI64 * cutoff);
        let alpha = dt / (rc + dt);
        let mut y = 0.0;
        for s in buf.iter_mut() {
            y += alpha * (*s - y);
            *s = y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{lofi_chain, pendulum_pos, resample};

    #[test]
    fn resample_octave_up_halves_length() {
        let data: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
        let out = resample(&data, 44100, 2.0, 0, 0, 0);
        assert_eq!(out.len(), 50);
        assert!((out[10] - 0.20).abs() < 1e-6);
    }
    #[test]
    fn resample_duration_with_loop_repeats_region() {
        let data: Vec<f32> = vec![10.0, 20.0, 30.0, 40.0];
        let out = resample(&data, 1000, 1.0, 10, 1, 3);
        assert_eq!(out.len(), 10);
        assert!((out[3] - 20.0).abs() < 1e-6);
        assert!((out[4] - 30.0).abs() < 1e-6);
    }
    #[test]
    fn lofi_bitcrush_quantizes() {
        let mut buf = vec![0.1, 0.6, -0.9];
        lofi_chain(&mut buf, 44100, 2, 0.0);
        assert!((buf[1] - 0.5).abs() < 1e-9);
    }
    #[test]
    fn pendulum_starts_left() {
        assert!(pendulum_pos(0.0, 4.0, 1.0).abs() < 1e-9);
        assert!((pendulum_pos(2.0, 4.0, 1.0) - 1.0).abs() < 1e-9);
    }
}
