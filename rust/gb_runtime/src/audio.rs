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
    music: Option<Music<'static>>,
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
        Ok(Audio { dev, sounds: Vec::new(), music: None, bands: Vec::new(), agc: 1e-4 })
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
        let s = self.dev.new_sound(path).map_err(|e| format!("LOADSOUND: {}", e))?;
        self.sounds.push(s);
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
        if let Some(m) = self.music.take() { m.stop_stream(); }
        let m = self.dev.new_music(path).map_err(|e| format!("PLAYMUSIC: {}", e))?;
        m.set_volume(volume.clamp(0.0, 1.0) as f32);
        m.play_stream();
        self.music = Some(m);
        Ok(())
    }

    pub fn stop_music(&mut self) {
        if let Some(m) = self.music.take() { m.stop_stream(); }
    }

    /// Pro Frame aufrufen (aus dem FLIP-Pfad), damit der Musik-Stream
    /// nachgefuettert wird -- sonst stockt die Wiedergabe.
    pub fn update(&self) {
        if let Some(m) = &self.music { m.update_stream(); }
    }
}
