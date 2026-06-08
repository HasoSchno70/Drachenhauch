//! Native Audio ueber raylib -- Core-Builtins LOADSOUND/PLAYSOUND/STOPSOUND
//! (SFX) sowie PLAYMUSIC/STOPMUSIC (Stream-Musik). Nur mit `--features
//! graphics` (raylib bundelt Audio mit).
//!
//! Audio ist -- wie RND/MILLIS/Tween -- **nicht bit-identisch** zur Python-
//! Version (anderer Mixer), nur funktional aequivalent.
//!
//! Lifetime-Trick: `Sound<'aud>`/`Music<'aud>` borgen das `RaylibAudio`-
//! Geraet. Da das Geraet den ganzen Prozess lebt, wird es per `Box::leak`
//! zu `&'static` gemacht -- so lassen sich Sounds/Musik in `Vec`/`Option`
//! halten, ohne ein self-referential struct zu bauen.
#![cfg(feature = "graphics")]

use std::os::raw::{c_uint, c_void};
use std::sync::Mutex;

use raylib::core::audio::{Music, RaylibAudio, Sound};

const TWO_PI: f32 = std::f32::consts::TAU;
const RING: usize = 4096;     // Ringpuffer fuer Mono-Samples (vom Audio-Thread)
const FFT_N: usize = 1024;    // FFT-Fenster

struct Ring { buf: [f32; RING], pos: usize }
// Global, weil der raylib-Mix-Callback eine freie `extern "C"`-Funktion sein
// muss (kein Closure-State). Der Audio-Thread schreibt, der Main-Thread liest.
static SAMPLES: Mutex<Ring> = Mutex::new(Ring { buf: [0.0; RING], pos: 0 });

/// raylib-Mix-Processor: bekommt das fertig gemischte Stereo-Float-Signal der
/// gesamten Pipeline. Wir mischen auf Mono und schieben es in den Ringpuffer.
/// `try_lock`, damit der Audio-Thread nie blockiert (verpasste Samples = egal).
unsafe extern "C" fn mixed_proc(buffer: *mut c_void, frames: c_uint) {
    let n = frames as usize;
    if buffer.is_null() || n == 0 { return; }
    let data = buffer as *const f32;
    if let Ok(mut ring) = SAMPLES.try_lock() {
        let mut p = ring.pos;
        for i in 0..n {
            let l = *data.add(i * 2);
            let r = *data.add(i * 2 + 1);
            ring.buf[p] = (l + r) * 0.5;
            p = (p + 1) % RING;
        }
        ring.pos = p;
    }
}

/// Iterative Radix-2-FFT (in-place), N muss Zweierpotenz sein.
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

pub struct Audio {
    dev: &'static RaylibAudio,
    sounds: Vec<Sound<'static>>,
    sound_vol: Vec<f32>,        // getrackte Volumes (raylib hat keinen Getter)
    music: Option<Music<'static>>,
    music_vol: f32,            // getracktes Music-Volume
    music_queue: Option<String>, // AUDIO_MUSIC_QUEUE -> bei Stream-Ende abspielen
    num_channels: i64,         // emuliert (raylib hat kein festes Channel-Limit)
    bands: Vec<f32>,   // geglaettete Band-Pegel (Peak-Hold)
    agc: f32,          // Auto-Gain-Referenz (adaptiv)
}

impl Audio {
    pub fn new() -> Result<Audio, String> {
        let dev = RaylibAudio::init_audio_device()
            .map_err(|_| "Audio-Geraet konnte nicht initialisiert werden".to_string())?;
        let dev: &'static RaylibAudio = Box::leak(Box::new(dev));
        // Mix-Tap fuer die FFT anhaengen (gesamte Pipeline -> Mono-Ring).
        unsafe { raylib::ffi::AttachAudioMixedProcessor(Some(mixed_proc)); }
        Ok(Audio {
            dev, sounds: Vec::new(), sound_vol: Vec::new(),
            music: None, music_vol: 1.0, music_queue: None, num_channels: 16,
            bands: Vec::new(), agc: 1e-4,
        })
    }

    /// Fuellt `out` mit B logarithmisch verteilten Band-Pegeln (0..1) aus dem
    /// zuletzt gehoerten Audio. Auto-Gain normalisiert lautstaerkeunabhaengig;
    /// Peak-Hold-Glaettung laesst die Balken springen + sanft fallen.
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

    pub fn load_sound(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let path = resolved.as_str();
        let s = self.dev.new_sound(path).map_err(|e| format!("LOADSOUND: {}", e))?;
        self.sounds.push(s);
        self.sound_vol.push(1.0);
        Ok((self.sounds.len() - 1) as i64)
    }

    fn sound(&self, idx: i64, fn_: &str) -> Result<&Sound<'static>, String> {
        self.sounds.get(idx as usize)
            .ok_or_else(|| format!("{}: ungueltiges SOUND-Handle {}", fn_, idx))
    }

    pub fn play_sound(&self, idx: i64, volume: f64) -> Result<(), String> {
        let s = self.sound(idx, "PLAYSOUND")?;
        s.set_volume(volume.clamp(0.0, 1.0) as f32);
        s.play();
        Ok(())
    }

    pub fn stop_sound(&self, idx: i64) -> Result<(), String> {
        self.sound(idx, "STOPSOUND")?.stop();
        Ok(())
    }

    /// Laedt + startet einen Musik-Stream. Ersetzt eine evtl. laufende Musik
    /// (es gibt genau einen Stream gleichzeitig). Musik loopt (raylib-Default).
    pub fn play_music(&mut self, path: &str, volume: f64) -> Result<(), String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let path = resolved.as_str();
        if let Some(m) = self.music.take() { m.stop_stream(); }
        let m = self.dev.new_music(path).map_err(|e| format!("PLAYMUSIC: {}", e))?;
        let v = volume.clamp(0.0, 1.0) as f32;
        m.set_volume(v);
        m.play_stream();
        self.music = Some(m);
        self.music_vol = v;
        self.music_queue = None;
        Ok(())
    }

    pub fn stop_music(&mut self) {
        if let Some(m) = self.music.take() { m.stop_stream(); }
    }

    /// Pro Frame aufrufen (aus dem FLIP-Pfad), damit der Musik-Stream
    /// nachgefuettert wird -- sonst stockt die Wiedergabe. Spielt zudem einen
    /// per AUDIO_MUSIC_QUEUE vorgemerkten Track ab, sobald der aktuelle endet.
    pub fn update(&mut self) {
        if let Some(m) = &self.music {
            m.update_stream();
            if !m.is_stream_playing() && self.music_queue.is_some() {
                let path = self.music_queue.take().unwrap();
                if let Ok(nm) = self.dev.new_music(&path) {
                    nm.set_volume(self.music_vol);
                    nm.play_stream();
                    self.music = Some(nm);
                }
            }
        }
    }

    // ===================================================================
    // Erweitertes audio-Modul (AUDIO_*) -- funktional (nicht bit-identisch).
    // SOUND/AUDIO_CHANNEL sind beide INTEGER-Handles (Index in `sounds`);
    // ein "Channel" steuert die Wiedergabe genau dieses Sounds. raylib hat
    // keine pygame-Channels -> ein Sound = ein steuerbarer Slot.
    // ===================================================================

    fn snd(&self, idx: i64, fn_: &str) -> Result<&Sound<'static>, String> {
        self.sounds.get(idx as usize)
            .ok_or_else(|| format!("{}: ungueltiges Handle {}", fn_, idx))
    }

    // -- Tone-Generation (liefert ein SOUND-Handle) --
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
            let t = i as f64 / sr as f64;
            *b = waveform_value(&wf, freq, t);
        }
        self.push_wave_sound(&buf, volume, sr)
    }

    pub fn noise(&mut self, dur_ms: i64, volume: f64) -> Result<i64, String> {
        if dur_ms <= 0 { return Err("AUDIO_NOISE: dauer_ms muss > 0 sein".into()); }
        let sr: u32 = 44100;
        let n = (sr as f64 * dur_ms as f64 / 1000.0) as usize;
        if n == 0 { return Err("AUDIO_NOISE: dauer_ms zu klein fuer Sample-Rate".into()); }
        let mut buf = vec![0.0f64; n];
        for b in buf.iter_mut() { *b = rng_uniform(); }
        self.push_wave_sound(&buf, volume, sr)
    }

    /// Prozeduraler sfxr-Stil-Effekt (siehe gamebasic/synth.py -- gleiche
    /// Mathematik): Waveform mit Pitch-Slide (Phasen-Integration) + Vibrato +
    /// ADSR-Huellkurve. `stereo_width` in (0,1] -> breiter Stereo-Sound.
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
        let left = build_sfx_buffer(&wf, base_freq, slide, n, na, nd,
                                    vib_depth, vib_speed, sr);
        if stereo_width <= 0.0 {
            return self.push_wave_sound(&left, volume, sr);
        }
        let w = stereo_width.min(1.0);
        let right = if wf == "noise" {
            build_sfx_buffer(&wf, base_freq, slide, n, na, nd, vib_depth, vib_speed, sr)
        } else {
            let detune = 1.0 + 0.04 * w;          // bis 4% = Chorus-Breite
            build_sfx_buffer(&wf, base_freq * detune, slide * detune, n, na, nd,
                             vib_depth, vib_speed, sr)
        };
        self.push_wave_sound_stereo(&left, &right, volume, sr)
    }

    /// Wie push_wave_sound, aber interleaved Stereo (L/R) -> Stereo-WAV.
    fn push_wave_sound_stereo(&mut self, left: &[f64], right: &[f64],
                              volume: f64, sr: u32) -> Result<i64, String> {
        let n = left.len();
        let vol = volume.clamp(0.0, 1.0);
        let fade = ((sr as f64 * 0.005) as usize).min(n / 4);
        let mut samples = vec![0i16; n * 2];
        for i in 0..n {
            let mut e = 1.0;
            if fade > 0 {
                if i < fade { e = i as f64 / fade as f64; }
                else if i >= n - fade { e = (n - 1 - i) as f64 / fade as f64; }
            }
            let l = (left[i] * e * vol).clamp(-1.0, 1.0);
            let r = (right[i] * e * vol).clamp(-1.0, 1.0);
            samples[i * 2] = (l * 32767.0) as i16;
            samples[i * 2 + 1] = (r * 32767.0) as i16;
        }
        let wav = encode_wav16(&samples, 2, sr);
        let wave = self.dev.new_wave_from_memory(".wav", &wav)
            .map_err(|e| format!("AUDIO_SFX: {}", e))?;
        let s = self.dev.new_sound_from_wave(&wave)
            .map_err(|e| format!("AUDIO_SFX: {}", e))?;
        self.sounds.push(s);
        self.sound_vol.push(vol as f32);
        Ok((self.sounds.len() - 1) as i64)
    }

    /// Float-Buffer [-1,1] -> Anti-Click-Envelope -> Volume -> i16-PCM -> WAV
    /// im RAM -> raylib Sound. Liefert das SOUND-Handle.
    fn push_wave_sound(&mut self, buf: &[f64], volume: f64, sr: u32) -> Result<i64, String> {
        let n = buf.len();
        let vol = volume.clamp(0.0, 1.0);
        let fade = ((sr as f64 * 0.005) as usize).min(n / 4);
        let mut samples = vec![0i16; n];
        for i in 0..n {
            let mut v = buf[i];
            if fade > 0 {
                if i < fade { v *= i as f64 / fade as f64; }
                else if i >= n - fade { v *= (n - 1 - i) as f64 / fade as f64; }
            }
            v = (v * vol).clamp(-1.0, 1.0);
            samples[i] = (v * 32767.0) as i16;
        }
        let wav = encode_wav_mono16(&samples, sr);
        let wave = self.dev.new_wave_from_memory(".wav", &wav)
            .map_err(|e| format!("AUDIO_TONE: {}", e))?;
        let s = self.dev.new_sound_from_wave(&wave)
            .map_err(|e| format!("AUDIO_TONE: {}", e))?;
        self.sounds.push(s);
        self.sound_vol.push(vol as f32);
        Ok((self.sounds.len() - 1) as i64)
    }

    // -- Channel-Playback (Handle == Sound-Index) --
    pub fn ch_play(&mut self, idx: i64, volume: f64) -> Result<i64, String> {
        let v = volume.clamp(0.0, 1.0) as f32;
        self.snd(idx, "AUDIO_PLAY")?.set_volume(v);
        self.snd(idx, "AUDIO_PLAY")?.play();
        if let Some(slot) = self.sound_vol.get_mut(idx as usize) { *slot = v; }
        Ok(idx)
    }
    pub fn ch_pause(&self, idx: i64) -> Result<(), String> { self.snd(idx, "AUDIO_PAUSE")?.pause(); Ok(()) }
    pub fn ch_resume(&self, idx: i64) -> Result<(), String> { self.snd(idx, "AUDIO_RESUME")?.resume(); Ok(()) }
    pub fn ch_stop(&self, idx: i64) -> Result<(), String> { self.snd(idx, "AUDIO_STOP")?.stop(); Ok(()) }
    pub fn ch_is_playing(&self, idx: i64) -> Result<bool, String> { Ok(self.snd(idx, "AUDIO_IS_PLAYING")?.is_playing()) }
    pub fn ch_set_volume(&mut self, idx: i64, v: f64) -> Result<(), String> {
        let vol = v.clamp(0.0, 1.0) as f32;
        self.snd(idx, "AUDIO_VOLUME")?.set_volume(vol);
        if let Some(slot) = self.sound_vol.get_mut(idx as usize) { *slot = vol; }
        Ok(())
    }
    pub fn ch_get_volume(&self, idx: i64) -> Result<f64, String> {
        Ok(*self.sound_vol.get(idx as usize)
            .ok_or_else(|| format!("AUDIO_GET_VOLUME: ungueltiges Handle {}", idx))? as f64)
    }
    /// AUDIO_PAN(left,right) -> raylib hat nur (pan, volume). Naeherung:
    /// volume=max(l,r), pan=l-Anteil (raylib-Pan kann gespiegelt sein).
    pub fn ch_pan(&mut self, idx: i64, left: f64, right: f64) -> Result<(), String> {
        let l = left.clamp(0.0, 1.0);
        let r = right.clamp(0.0, 1.0);
        let vol = l.max(r);
        let pan = if l + r > 0.0 { l / (l + r) } else { 0.5 };
        let s = self.snd(idx, "AUDIO_PAN")?;
        s.set_volume(vol as f32);
        s.set_pan(pan as f32);
        if let Some(slot) = self.sound_vol.get_mut(idx as usize) { *slot = vol as f32; }
        Ok(())
    }

    // -- Mixer-weit --
    pub fn pause_all(&self) { for s in &self.sounds { s.pause(); } }
    pub fn resume_all(&self) { for s in &self.sounds { s.resume(); } }
    pub fn stop_all(&self) { for s in &self.sounds { s.stop(); } }
    pub fn set_num_channels(&mut self, n: i64) { self.num_channels = n.max(0); }
    pub fn get_num_channels(&self) -> i64 { self.num_channels }
    pub fn busy_channels(&self) -> i64 { self.sounds.iter().filter(|s| s.is_playing()).count() as i64 }

    // -- Music erweitert --
    pub fn music_load(&mut self, path: &str) -> Result<(), String> {
        if let Some(m) = self.music.take() { m.stop_stream(); }
        let m = self.dev.new_music(path).map_err(|e| format!("AUDIO_MUSIC_LOAD: {}", e))?;
        m.set_volume(self.music_vol);
        self.music = Some(m);
        self.music_queue = None;
        Ok(())
    }
    pub fn music_play(&self) { if let Some(m) = &self.music { m.play_stream(); } }
    pub fn music_stop(&mut self) { if let Some(m) = &self.music { m.stop_stream(); } }
    pub fn music_pause(&self) { if let Some(m) = &self.music { m.pause_stream(); } }
    pub fn music_resume(&self) { if let Some(m) = &self.music { m.resume_stream(); } }
    pub fn music_set_volume(&mut self, v: f64) {
        let vol = v.clamp(0.0, 1.0) as f32;
        self.music_vol = vol;
        if let Some(m) = &self.music { m.set_volume(vol); }
    }
    pub fn music_get_volume(&self) -> f64 { self.music_vol as f64 }
    pub fn music_position(&self) -> f64 {
        match &self.music { Some(m) => m.get_time_played().max(0.0) as f64, None => 0.0 }
    }
    pub fn music_busy(&self) -> bool {
        matches!(&self.music, Some(m) if m.is_stream_playing())
    }
    pub fn music_queue(&mut self, path: &str) { self.music_queue = Some(path.to_string()); }
}

const PI64: f64 = std::f64::consts::PI;

/// Eine Periode bei (t*freq): Werte in [-1,1]. (noise wird separat erzeugt.)
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

/// Waveform-Wert aus der integrierten Phase (in Radiant) -- fuer Pitch-Slides,
/// wo die Frequenz pro Sample variiert. `noise` wird separat erzeugt.
fn phase_value(kind: &str, phase: f64) -> f64 {
    let ph = phase / (2.0 * PI64);          // in Zyklen
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
/// Uniform [-1,1) (xorshift) -- nicht-deterministisch (wie np.random).
fn rng_uniform() -> f64 {
    ARNG.with(|s| {
        let mut x = s.get();
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        s.set(x);
        ((x >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    })
}

/// Minimaler PCM16-Mono-WAV-Encoder (RIFF/WAVE/fmt/data).
fn encode_wav_mono16(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    encode_wav16(samples, 1, sample_rate)
}

/// PCM16-WAV-Encoder fuer `channels` Kanaele (Samples interleaved).
fn encode_wav16(samples: &[i16], channels: u16, sample_rate: u32) -> Vec<u8> {
    let block_align = 2u16 * channels;
    let data_len = (samples.len() * 2) as u32;
    let mut v = Vec::with_capacity(44 + data_len as usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVE");
    v.extend_from_slice(b"fmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());   // PCM
    v.extend_from_slice(&channels.to_le_bytes());
    v.extend_from_slice(&sample_rate.to_le_bytes());
    v.extend_from_slice(&(sample_rate * block_align as u32).to_le_bytes()); // byte rate
    v.extend_from_slice(&block_align.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());  // bits/sample
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for s in samples { v.extend_from_slice(&s.to_le_bytes()); }
    v
}

/// Baut einen Mono-SFX-Float-Buffer (Pitch-Slide + Vibrato + ADSR), wie
/// `gamebasic.synth._mono`. Ohne Volume.
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
        } else {
            1.0
        };
        buf[i] = (v * env).clamp(-1.0, 1.0);
    }
    buf
}
