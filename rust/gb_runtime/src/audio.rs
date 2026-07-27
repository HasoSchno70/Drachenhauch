//! Native Audio ueber **Kira** (cpal) -- das Audio-Backend von gbrt (loeste
//! 2026-06-13 raylib-Audio ab). Vorteil gegenueber raylib: ein eigener Audio-
//! Thread, vollstaendig vom Game-Loop entkoppelt (kein per-Frame-Refill ->
//! kein Stottern bei schweren Frames), echte Mixer-Tracks/Effekte, tweenbare
//! Lautstaerke/Pitch/Pan. MOD/XM via reinem Rust-Player (xmrs/xmrsplayer).
//! Eingebunden mit `--features graphics` (raylib bleibt fuer Fenster/Input).
//!
//! Audio-Output ist -- wie RND/MILLIS/Tween -- naturgemaess nicht
//! deterministisch golden-testbar; getestet wird die reine DSP-Mathematik
//! (resample/lofi/pendulum) + Argument-Validierung.
#![cfg(feature = "graphics")]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering::Relaxed};
use std::time::Duration;

use kira::sound::{Sound, SoundData};

use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend, Decibels, Easing, Frame, Mix, Panning, Tween,
    clock::{ClockHandle, ClockSpeed, ClockTime},
    effect::{Effect, EffectBuilder},
    effect::filter::{FilterBuilder, FilterHandle},
    effect::reverb::{ReverbBuilder, ReverbHandle},
    effect::distortion::{DistortionBuilder, DistortionHandle},
    effect::compressor::{CompressorBuilder, CompressorHandle},
    effect::eq_filter::{EqFilterBuilder, EqFilterHandle, EqFilterKind},
    info::Info,
    listener::ListenerHandle,
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    sound::{PlaybackState, streaming::{StreamingSoundData, StreamingSoundHandle}},
    sound::FromFileError,
    track::{MainTrackBuilder, SpatialTrackBuilder, SpatialTrackHandle, TrackBuilder, TrackHandle},
};

/// Echtzeit-Effektkette eines Mixer-Busses (Filter/Reverb/Delay). Die Effekte
/// haengen beim Bau am Track; die Handles steuern sie zur Laufzeit. Default =
/// neutral (Filter offen, Reverb/Delay-Mix 0).
struct BusFx {
    filter: FilterHandle,
    reverb: ReverbHandle,
    delay: Arc<DelayShared>,   // eigener Delay (Zeit zur Laufzeit aenderbar)
    distortion: DistortionHandle,
    compressor: CompressorHandle,
    eq: EqFilterHandle,
}

/// Maximale Delay-Zeit des eigenen Delay-Effekts (Ringpuffer-Groesse).
const DELAY_MAX_S: f32 = 4.0;

/// Vom Main-Thread beschrieben, vom Delay-Effekt (Audio-Thread) gelesen.
struct DelayShared {
    delay_ms: AtomicU32,
    feedback: AtomicU32,   // f32-Bits, 0..0.95
    mix: AtomicU32,        // f32-Bits, 0..1 (0 = aus)
}

/// Eigener Stereo-Delay (Ringpuffer) -- im Gegensatz zu Kiras DelayHandle ist
/// die Delay-Zeit zur Laufzeit aenderbar (delay_ms-Atomic). Neutral = mix 0.
struct DelayBuilderRT;
struct DelayEffectRT {
    shared: Arc<DelayShared>,
    bl: Vec<f32>,
    br: Vec<f32>,
    pos: usize,
    sr: u32,
}
impl EffectBuilder for DelayBuilderRT {
    type Handle = Arc<DelayShared>;
    fn build(self) -> (Box<dyn Effect>, Arc<DelayShared>) {
        let shared = Arc::new(DelayShared {
            delay_ms: AtomicU32::new(300),
            feedback: AtomicU32::new(0.5f32.to_bits()),
            mix: AtomicU32::new(0.0f32.to_bits()),
        });
        (Box::new(DelayEffectRT { shared: shared.clone(), bl: Vec::new(), br: Vec::new(), pos: 0, sr: 44100 }), shared)
    }
}
impl DelayEffectRT {
    fn alloc(&mut self, sr: u32) {
        self.sr = sr.max(1);
        let n = (DELAY_MAX_S * self.sr as f32) as usize + 1;
        self.bl = vec![0.0; n];
        self.br = vec![0.0; n];
        self.pos = 0;
    }
}
impl Effect for DelayEffectRT {
    fn init(&mut self, sample_rate: u32, _buf: usize) { self.alloc(sample_rate); }
    fn on_change_sample_rate(&mut self, sr: u32) { self.alloc(sr); }
    fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info) {
        let n = self.bl.len();
        if n == 0 { return; }
        let mix = f32::from_bits(self.shared.mix.load(Relaxed)).clamp(0.0, 1.0);
        let fb = f32::from_bits(self.shared.feedback.load(Relaxed)).clamp(0.0, 0.95);
        let dms = self.shared.delay_ms.load(Relaxed) as u64;
        let delay = (((dms * self.sr as u64) / 1000) as usize).clamp(1, n - 1);
        for frame in input.iter_mut() {
            let read = (self.pos + n - delay) % n;
            let (wl, wr) = (self.bl[read], self.br[read]);
            self.bl[self.pos] = frame.left + wl * fb;
            self.br[self.pos] = frame.right + wr * fb;
            self.pos = (self.pos + 1) % n;
            frame.left = frame.left * (1.0 - mix) + wl * mix;
            frame.right = frame.right * (1.0 - mix) + wr * mix;
        }
    }
}

/// Haengt die Effektkette (neutral) an einen Track-Builder und liefert die
/// Steuer-Handles. Als Makro, weil Sub- und Main-TrackBuilder verschiedene
/// Typen sind (beide haben aber `add_effect`). Reihenfolge = Signalfluss:
/// EQ -> Filter -> Distortion -> Compressor -> Reverb -> Delay. Delay-Zeit fix
/// (300 ms). Default ueberall neutral (Mix 0 bzw. 0 dB / Cutoff offen).
macro_rules! attach_bus_fx {
    ($b:expr) => { BusFx {
        eq: $b.add_effect(EqFilterBuilder::new(EqFilterKind::Bell, 1000.0, Decibels(0.0), 1.0)),
        filter: $b.add_effect(FilterBuilder::new().cutoff(20000.0).resonance(0.0)),
        distortion: $b.add_effect(DistortionBuilder::new().drive(Decibels(0.0)).mix(Mix(0.0))),
        compressor: $b.add_effect(CompressorBuilder::new().mix(Mix(0.0))),
        reverb: $b.add_effect(ReverbBuilder::new().mix(Mix(0.0))),
        delay: $b.add_effect(DelayBuilderRT),
    }};
}
use xmrs::prelude::Module;
use xmrsplayer::prelude::XmrsPlayer;

/// Woher die aktuelle Musik kommt: Stream-Datei (ogg/mp3/wav/flac) oder die
/// rohen Bytes eines Tracker-Moduls (.mod/.xm/.s3m/.it). Module werden in
/// ECHTZEIT gestreamt (eigener Kira-Sound, kein Vorab-Render): instant geladen,
/// exaktes Endlos-Loopen, wenig RAM. Bytes werden pro Play frisch geparst (ms).
enum MusicSource {
    Stream(String),
    Module(Vec<u8>),
}

/// Laufende Musik-Instanz. Stream/Static teilen die Kira-Handle-API; Module
/// laufen ueber einen Custom-Sound, gesteuert per Atomics in `ModShared`.
enum MusicHandle {
    Stream(StreamingSoundHandle<FromFileError>),
    Static(StaticSoundHandle),
    Module(Arc<ModShared>),
}

fn is_module_path(p: &str) -> bool {
    let l = p.to_lowercase();
    l.ends_with(".mod") || l.ends_with(".xm") || l.ends_with(".s3m") || l.ends_with(".it")
}

// ===================================================================
// Echtzeit-Modul-Streaming: Custom-Kira-Sound, der den xmrs-Player auf dem
// Audio-Thread pollt. Steuerung (Volume/Fade/Pitch/Pause/Stop/Position) ueber
// `ModShared`-Atomics. Das Modul wird beim Laden geleakt (-> 'static Borrow
// fuer den Player) und vom Sound im Drop wieder freigegeben.
// ===================================================================

/// Vom Handle (Main-Thread) beschrieben, vom Sound (Audio-Thread) gelesen.
struct ModShared {
    vol_target: AtomicU32,        // f32-Bits, linear 0..1
    fade_ms: AtomicU32,           // Ramp-Dauer beim naechsten Volume-Wechsel
    fade_curve: AtomicU8,         // FadeCurve::as_u8() -- Kurve der naechsten Ramp
    pitch: AtomicU32,             // f32-Bits, Abspielrate
    paused: AtomicBool,
    stop_now: AtomicBool,         // sofort beenden
    finish_at_zero: AtomicBool,   // nach Fade-out auf 0 beenden
    pos_frames: AtomicU64,        // Sound schreibt -> Position
    finished: AtomicBool,
    sr: AtomicU32,                // Geraete-Sample-Rate (Sound schreibt)
}
impl ModShared {
    fn new(vol: f32, pitch: f32) -> Arc<Self> {
        Arc::new(ModShared {
            vol_target: AtomicU32::new(vol.to_bits()),
            fade_ms: AtomicU32::new(5),
            fade_curve: AtomicU8::new(FadeCurve::Linear.as_u8()),
            pitch: AtomicU32::new(pitch.to_bits()),
            paused: AtomicBool::new(false),
            stop_now: AtomicBool::new(false),
            finish_at_zero: AtomicBool::new(false),
            pos_frames: AtomicU64::new(0),
            finished: AtomicBool::new(false),
            sr: AtomicU32::new(0),
        })
    }
}

/// SoundData -> via `into_sound` in einen Sound + Handle (Arc<ModShared>).
struct ModuleSoundData {
    module_ptr: *mut Module,   // geleakt; ModuleSound gibt es im Drop frei
    loop_max: usize,           // 0 = endlos
    shared: Arc<ModShared>,
}
unsafe impl Send for ModuleSoundData {}

impl SoundData for ModuleSoundData {
    type Error = ();
    type Handle = Arc<ModShared>;
    fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), ()> {
        let handle = self.shared.clone();
        Ok((Box::new(ModuleSound {
            player: None, module_ptr: self.module_ptr, loop_max: self.loop_max,
            shared: self.shared, created: false,
            prev: (0.0, 0.0), next: (0.0, 0.0), frac: 0.0, primed: false, ended: false,
            vol_cur: 0.0, last_target: f32::NAN, ramp_total: 1, ramp_done: 1, ramp_start: 0.0,
            ramp_start_db: db(0.0).0, ramp_tgt_db: db(0.0).0,
            ramp_curve: FadeCurve::Linear,
            pos: 0,
        }), handle))
    }
}

struct ModuleSound {
    player: Option<XmrsPlayer<'static>>,
    module_ptr: *mut Module,
    loop_max: usize,
    shared: Arc<ModShared>,
    created: bool,
    // Pitch-Resampler (fraktionale Leseposition + lineare Interpolation)
    prev: (f32, f32),
    next: (f32, f32),
    frac: f64,
    primed: bool,
    ended: bool,
    // Volume-Ramp (Fades, klickfrei)
    vol_cur: f32,
    last_target: f32,
    ramp_total: u64,
    ramp_done: u64,
    ramp_start: f32,
    // Review-Fund: `ramp_start`/`tgt` (linear 0..1) wurden bisher DIREKT linear
    // interpoliert und mit der FadeCurve geformt -- Kiras eigener Tween-Pfad
    // (AUDIO_PLAY/STOP/AUDIO_MUSIC_PLAY/STOP fade_in/out_ms, siehe `mh_set_volume`
    // + die Kira-`Tweenable for Decibels`-Impl) interpoliert stattdessen den
    // DEZIBEL-Wert linear und konvertiert erst am Ende exponentiell zu
    // Amplitude. Dezibel ist eine logarithmische Skala -- "linear in Amplitude"
    // und "linear in dB" sind fuer denselben Start/Ziel-Wert UNTERSCHIEDLICHE
    // Kurven (v.a. nahe Stille), obwohl beide dieselbe FadeCurve-Formung
    // nutzen. Diese beiden Felder cachen die dB-Werte EINMAL pro Ramp-Start
    // (nicht pro Sample -- `db()` ruft `log10()`), damit der Modul-Fade-Pfad
    // dieselbe Kurve wie der Kira-Tween-Pfad trifft.
    ramp_start_db: f32,
    ramp_tgt_db: f32,
    ramp_curve: FadeCurve,
    pos: u64,
}
// module_ptr ist exklusiv im Besitz dieses Sounds; xmrs-Daten sind Send-fähig.
unsafe impl Send for ModuleSound {}

impl ModuleSound {
    fn pull(&mut self) -> Option<(f32, f32)> {
        let p = self.player.as_mut()?;
        let l = p.next()?;
        let r = p.next()?;
        Some((l as f32 / 32768.0, r as f32 / 32768.0))
    }
}

impl Sound for ModuleSound {
    fn process(&mut self, out: &mut [Frame], dt: f64, _info: &Info) {
        if !self.created {
            let sr = (1.0 / dt).round().max(1.0) as u32;
            self.shared.sr.store(sr, Relaxed);
            let m: &'static Module = unsafe { &*self.module_ptr };
            let mut p = XmrsPlayer::new(m, sr, 0);
            p.set_max_loop_count(self.loop_max);
            self.player = Some(p);
            self.created = true;
        }
        if self.shared.stop_now.load(Relaxed) { self.ended = true; }
        let sr = self.shared.sr.load(Relaxed).max(1) as f64;
        let paused = self.shared.paused.load(Relaxed);
        let pitch = f32::from_bits(self.shared.pitch.load(Relaxed)).max(0.0) as f64;
        // Volume-Ziel/Ramp aktualisieren, wenn sich das Ziel geaendert hat.
        let tgt = f32::from_bits(self.shared.vol_target.load(Relaxed));
        if tgt.to_bits() != self.last_target.to_bits() {
            self.ramp_start = self.vol_cur;
            // dB-Endpunkte EINMAL pro Ramp-Start cachen (nicht pro Sample).
            self.ramp_start_db = db(self.ramp_start as f64).0;
            self.ramp_tgt_db = db(tgt as f64).0;
            let fade_ms = self.shared.fade_ms.load(Relaxed) as f64;
            self.ramp_total = ((fade_ms * sr / 1000.0) as u64).max(1);
            self.ramp_done = 0;
            self.last_target = tgt;
            self.ramp_curve = FadeCurve::from_u8(self.shared.fade_curve.load(Relaxed));
        }
        let finish_at_zero = self.shared.finish_at_zero.load(Relaxed);

        for frame in out.iter_mut() {
            if self.ended || paused {
                *frame = Frame::ZERO;
                continue;
            }
            if !self.primed {
                self.prev = self.pull().unwrap_or((0.0, 0.0));
                self.next = self.pull().unwrap_or((0.0, 0.0));
                self.primed = true;
            }
            let f = self.frac as f32;
            let s = (self.prev.0 + (self.next.0 - self.prev.0) * f,
                     self.prev.1 + (self.next.1 - self.prev.1) * f);
            self.frac += pitch.max(1e-6);
            while self.frac >= 1.0 {
                self.prev = self.next;
                match self.pull() {
                    Some(v) => self.next = v,
                    None => { self.ended = true; break; }
                }
                self.frac -= 1.0;
            }
            // Volume-Ramp: linear-in-dB interpolieren (wie Kiras Tweenable
            // fuer Decibels), Kurve identisch zu Kira::Easing::apply()
            // (power=2), erst am Ende exponentiell zurueck zu Amplitude.
            if self.ramp_done < self.ramp_total {
                let t = self.ramp_curve.apply(self.ramp_done as f32 / self.ramp_total as f32);
                let db_cur = self.ramp_start_db + (self.ramp_tgt_db - self.ramp_start_db) * t;
                self.vol_cur = Decibels(db_cur).as_amplitude();
                self.ramp_done += 1;
            } else {
                self.vol_cur = tgt;
                if finish_at_zero && tgt <= 0.0001 { self.ended = true; }
            }
            *frame = Frame { left: s.0 * self.vol_cur, right: s.1 * self.vol_cur };
            self.pos += 1;
        }
        self.shared.pos_frames.store(self.pos, Relaxed);
        if self.ended { self.shared.finished.store(true, Relaxed); }
    }
    fn finished(&self) -> bool { self.ended }
}

impl Drop for ModuleSound {
    fn drop(&mut self) {
        // Erst den Player fallen lassen (gibt den Borrow aufs Modul frei),
        // dann das geleakte Modul freigeben.
        self.player = None;
        if !self.module_ptr.is_null() {
            unsafe { drop(Box::from_raw(self.module_ptr)); }
            self.module_ptr = std::ptr::null_mut();
        }
    }
}

// Einheitliche Steuerung der MusicHandle-Varianten. Module-Arme schreiben die
// Atomics (Volume in Decibels -> linear via as_amplitude).
fn mh_stop(h: &mut MusicHandle, t: Tween) {
    match h {
        MusicHandle::Stream(x) => x.stop(t),
        MusicHandle::Static(x) => x.stop(t),
        MusicHandle::Module(a) => a.stop_now.store(true, Relaxed),
    }
}
fn mh_pause(h: &mut MusicHandle, t: Tween) {
    match h {
        MusicHandle::Stream(x) => x.pause(t),
        MusicHandle::Static(x) => x.pause(t),
        MusicHandle::Module(a) => a.paused.store(true, Relaxed),
    }
}
fn mh_resume(h: &mut MusicHandle, t: Tween) {
    match h {
        MusicHandle::Stream(x) => x.resume(t),
        MusicHandle::Static(x) => x.resume(t),
        MusicHandle::Module(a) => a.paused.store(false, Relaxed),
    }
}
fn mh_set_volume(h: &mut MusicHandle, d: Decibels, t: Tween) {
    match h {
        MusicHandle::Stream(x) => x.set_volume(d, t),
        MusicHandle::Static(x) => x.set_volume(d, t),
        MusicHandle::Module(a) => {
            a.fade_ms.store(5, Relaxed);
            a.fade_curve.store(FadeCurve::Linear.as_u8(), Relaxed);
            a.vol_target.store(d.as_amplitude().to_bits(), Relaxed);
        }
    }
}
fn mh_set_rate(h: &mut MusicHandle, r: f64, t: Tween) {
    match h {
        MusicHandle::Stream(x) => x.set_playback_rate(r, t),
        MusicHandle::Static(x) => x.set_playback_rate(r, t),
        MusicHandle::Module(a) => a.pitch.store((r as f32).to_bits(), Relaxed),
    }
}
fn mh_state(h: &MusicHandle) -> PlaybackState {
    match h {
        MusicHandle::Stream(x) => x.state(),
        MusicHandle::Static(x) => x.state(),
        MusicHandle::Module(a) => if a.finished.load(Relaxed) {
            PlaybackState::Stopped
        } else {
            PlaybackState::Playing
        },
    }
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

fn tween_now() -> Tween { Tween { duration: Duration::from_millis(4), ..Default::default() } }

// --- Listener/Emitter (raeumliches Audio) -------------------------------

/// GB-Weltkoordinaten (f64, beliebige Einheit) -> Kiras `mint::Vector3<f32>`.
fn vec3(x: f64, y: f64, z: f64) -> mint::Vector3<f32> {
    mint::Vector3 { x: x as f32, y: y as f32, z: z as f32 }
}

/// Blickrichtung als reine Y-Achsen-Drehung (Grad) -> Quaternion. Deckt die
/// typische Top-Down-/3rd-Person-Kamera ab (links/rechts drehen), ohne BASIC-
/// Nutzern volle Quaternionen zuzumuten (Pitch/Roll gibt's dafuer nicht).
/// Kira-Konvention: unrotiert blickt der Listener Richtung -Z, +X ist rechts.
fn yaw_quat(deg: f64) -> mint::Quaternion<f32> {
    let half = (deg.to_radians() * 0.5) as f32;
    mint::Quaternion { v: mint::Vector3 { x: 0.0, y: half.sin(), z: 0.0 }, s: half.cos() }
}

/// Zeitkurve fuer Fades/Slides -- macht sie natuerlicher als eine reine
/// lineare Rampe klingen (ein Fade-out z.B. wirkt mit "out" laenger hoerbar).
/// BASIC-seitig als Name (`"linear"`/`"in"`/`"out"`/`"inout"`), quadratische
/// Kurven (Kira: `Easing::{In,Out,InOut}Powi(2)`). Fuer den Kira-Tween-Pfad
/// direkt nach `Easing` konvertierbar; fuer den Modul-Fade (eigener Ramp,
/// kein Kira-Tween) liefert `apply()` dieselbe Mathematik als reine Funktion.
#[derive(Clone, Copy, PartialEq)]
enum FadeCurve { Linear, In, Out, InOut }

impl FadeCurve {
    fn parse(name: &str, fn_: &str) -> Result<Self, String> {
        match name.to_lowercase().as_str() {
            "" | "linear" => Ok(Self::Linear),
            "in" => Ok(Self::In),
            "out" => Ok(Self::Out),
            "inout" => Ok(Self::InOut),
            other => Err(format!(
                "{}: unbekanntes Easing '{}' (linear, in, out, inout)", fn_, other)),
        }
    }
    fn to_kira(self) -> Easing {
        match self {
            Self::Linear => Easing::Linear,
            Self::In => Easing::InPowi(2),
            Self::Out => Easing::OutPowi(2),
            Self::InOut => Easing::InOutPowi(2),
        }
    }
    /// Normalisierte Zeit `t` (0..1) -> geformte Zeit (0..1). Fuer den
    /// Modul-Fade-Ramp (audio-Thread-Hot-Path, kein Kira-Tween beteiligt).
    /// Identische Mathematik zu Kira's `Easing::apply()` mit power=2.
    fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::In => t * t,
            Self::Out => 1.0 - (1.0 - t) * (1.0 - t),
            Self::InOut => if t < 0.5 { 2.0 * t * t } else { 1.0 - 2.0 * (1.0 - t) * (1.0 - t) },
        }
    }
    fn as_u8(self) -> u8 {
        match self { Self::Linear => 0, Self::In => 1, Self::Out => 2, Self::InOut => 3 }
    }
    fn from_u8(v: u8) -> Self {
        match v { 1 => Self::In, 2 => Self::Out, 3 => Self::InOut, _ => Self::Linear }
    }
}

fn tween_ms_eased(ms: i64, curve: FadeCurve) -> Tween {
    Tween { duration: Duration::from_millis(ms.max(0) as u64), easing: curve.to_kira(), ..Default::default() }
}

// --- Per-Sound-Slot (SFX/Tone/Sample) ----------------------------------

/// Ein "Channel" == ein geladener/gebauter Sound. `data` ist die (billig
/// klonbare) Sample-Quelle, `handle` die zuletzt gestartete Instanz.
/// `data == None` markiert einen via UNLOADSOUND freigegebenen Slot
/// (Frame-Puffer freigegeben, Index bleibt als Tombstone stabil).
struct SoundSlot {
    data: Option<StaticSoundData>,
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
#[derive(Clone, Copy)]
struct Pendulum { period_s: f64, depth: f64, start: std::time::Instant }

/// Eine Kira-Uhr + der zuletzt KOMMANDIERTE Ticking-Zustand. `ClockHandle::
/// ticking()` spiegelt Kiras Audio-Thread nur asynchron (per Kommando-Queue,
/// erst beim naechsten Audio-Callback aktualisiert) -- eine Abfrage direkt
/// nach START/PAUSE/STOP koennte sonst kurzzeitig den alten Wert zeigen.
/// Da GameBasic der einzige Aufrufer ist, ist der selbst mitgefuehrte Wert
/// immer sofort korrekt (kein Audio-Thread-Race).
struct ClockSlot { handle: ClockHandle, ticking: bool }

/// Pendel-Position 0..1 (pure, fuer #[test]). Kosinus-foermig, startet links.
fn pendulum_pos(elapsed_s: f64, period_s: f64, depth: f64) -> f64 {
    let d = depth.clamp(0.0, 1.0);
    0.5 - 0.5 * d * (std::f64::consts::TAU * elapsed_s / period_s).cos()
}

// --- Musik (Stream) ----------------------------------------------------

pub struct Audio {
    manager: AudioManager<DefaultBackend>,
    // Mixer-Busse: SFX/Sampler/Tone laufen auf sfx_track, Musik auf
    // music_track (beide routen in den Main-Track, wo der FFT-Tap haengt).
    // Bus-Volume × Sound-Volume multiplizieren sich im Mixer -> echter Master.
    sfx_track: TrackHandle,
    music_track: TrackHandle,
    sfx_bus: f32,
    music_bus: f32,
    master_bus: f32,
    // Echtzeit-Effektketten je Bus (Filter/Reverb/Delay).
    sfx_fx: BusFx,
    music_fx: BusFx,
    master_fx: BusFx,
    sounds: Vec<SoundSlot>,
    samples: Vec<Sample>,
    sample_cache: HashMap<(usize, i64, i64), i64>,
    // Kira-Uhren fuer sample-genaues Musik-/Rhythmus-Timing (AUDIO_CLOCK_*).
    // `None` = per AUDIO_CLOCK_REMOVE entfernt (Tombstone, Index bleibt stabil;
    // Kira kennt kein remove_clock() -- nur Handle-Drop gibt die Uhr frei).
    clocks: Vec<Option<ClockSlot>>,
    // Raeumliches Audio: Listener (Ohr/Kamera-Position) + Emitter (spatiale
    // Sub-Tracks, an je einen Listener gebunden -- Kira berechnet Panning/
    // Lautstaerke-Abnahme daraus selbst). Gleiches Tombstone-Vec-Pattern wie
    // Clocks (kein remove_listener()/remove_spatial_sub_track() -- nur Drop).
    listeners: Vec<Option<ListenerHandle>>,
    emitters: Vec<Option<SpatialTrackHandle>>,
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
        // Master-Track: erst die Effektkette, dann der FFT-Tap zuletzt ->
        // die FFT erfasst den fertig prozessierten Mix.
        let mut main = MainTrackBuilder::new();
        let master_fx = attach_bus_fx!(main);
        main.add_effect(FftTapBuilder);
        let settings = AudioManagerSettings { main_track_builder: main, ..Default::default() };
        let mut manager = AudioManager::<DefaultBackend>::new(settings)
            .map_err(|e| format!("Audio-Geraet konnte nicht initialisiert werden: {e:?}"))?;
        let mut sfx_b = TrackBuilder::new();
        let sfx_fx = attach_bus_fx!(sfx_b);
        let sfx_track = manager.add_sub_track(sfx_b)
            .map_err(|e| format!("SFX-Bus konnte nicht angelegt werden: {e:?}"))?;
        let mut music_b = TrackBuilder::new();
        let music_fx = attach_bus_fx!(music_b);
        let music_track = manager.add_sub_track(music_b)
            .map_err(|e| format!("Musik-Bus konnte nicht angelegt werden: {e:?}"))?;
        Ok(Audio {
            manager, sfx_track, music_track,
            sfx_bus: 1.0, music_bus: 1.0, master_bus: 1.0,
            sfx_fx, music_fx, master_fx,
            sounds: Vec::new(), samples: Vec::new(),
            sample_cache: HashMap::new(), clocks: Vec::new(),
            listeners: Vec::new(), emitters: Vec::new(), num_channels: 16,
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
        self.sounds.push(SoundSlot { data: Some(data), handle: None, vol, loops: 0, pan_anim: None });
        (self.sounds.len() - 1) as i64
    }

    /// UNLOADSOUND(sound) -- stoppt die laufende Instanz und gibt den
    /// Frame-Puffer frei. Der Slot-Index bleibt als Tombstone gueltig
    /// (kein Recycling -> alte Handles aliasen nie einen neuen Sound),
    /// erneutes Abspielen wirft eine klare Meldung. SAMPLE_PLAY-Cache-
    /// Eintraege, die auf diesen Slot zeigen, werden invalidiert.
    pub fn unload_sound(&mut self, idx: i64) -> Result<(), String> {
        let s = self.slot_mut(idx, "UNLOADSOUND")?;
        if let Some(h) = s.handle.as_mut() { h.stop(tween_now()); }
        s.handle = None;
        s.data = None;
        s.pan_anim = None;
        self.sample_cache.retain(|_, &mut v| v != idx);
        Ok(())
    }

    /// AUDIO_SOUND_COUNT() -- Anzahl lebender (nicht freigegebener) Sound-Slots.
    /// Diagnose gegen Puffer-Akkumulation in langen Songs.
    pub fn sound_count(&self) -> i64 {
        self.sounds.iter().filter(|s| s.data.is_some()).count() as i64
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
    fn start_slot(&mut self, idx: i64, fn_: &str, vol: f64, loops: i64, fade_in_ms: i64,
                  curve: FadeCurve) -> Result<(), String> {
        let vol = vol.clamp(0.0, 1.0);
        // alte Instanz stoppen
        if let Some(h) = self.slot_mut(idx, fn_)?.handle.as_mut() { h.stop(tween_now()); }
        let mut settings = StaticSoundSettings::new();
        if loops < 0 { settings = settings.loop_region(0.0..); }
        settings = settings.volume(if fade_in_ms > 0 { Decibels::SILENCE } else { db(vol) });
        let data = self.slot(idx, fn_)?.data.as_ref()
            .ok_or_else(|| format!("{}: Sound {} wurde freigegeben (UNLOADSOUND)", fn_, idx))?
            .clone().with_settings(settings);
        let mut handle = self.sfx_track.play(data)   // SFX-Bus
            .map_err(|e| format!("{}: {:?}", fn_, e))?;
        if fade_in_ms > 0 { handle.set_volume(db(vol), tween_ms_eased(fade_in_ms, curve)); }
        let s = self.slot_mut(idx, fn_)?;
        s.handle = Some(handle);
        s.vol = vol as f32;
        s.loops = if loops < 0 { -1 } else { loops };
        s.pan_anim = None;
        Ok(())
    }

    /// AUDIO_PLAY_AT(sound, clock, ticks[, volume[, loops]]) -- wie AUDIO_PLAY,
    /// aber der Start wird an einen Clock-Tick gebunden statt sofort zu
    /// erfolgen: sample-genau, weil Kira selbst auf dem Audio-Thread
    /// vergleicht, wann die Uhr `ticks` erreicht (kein Polling/Update-Call
    /// noetig -- anders als das frame-getriebene `timer`-Modul).
    pub fn ch_play_at(&mut self, idx: i64, clock_idx: i64, ticks: i64,
                       volume: f64, loops: i64) -> Result<i64, String> {
        let clock_id = self.clock_slot(clock_idx, "AUDIO_PLAY_AT")?.handle.id();
        let vol = volume.clamp(0.0, 1.0);
        if let Some(h) = self.slot_mut(idx, "AUDIO_PLAY_AT")?.handle.as_mut() { h.stop(tween_now()); }
        let mut settings = StaticSoundSettings::new()
            .start_time(ClockTime { clock: clock_id, ticks: ticks as u64, fraction: 0.0 })
            .volume(db(vol));
        if loops < 0 { settings = settings.loop_region(0.0..); }
        let data = self.slot(idx, "AUDIO_PLAY_AT")?.data.as_ref()
            .ok_or_else(|| format!("AUDIO_PLAY_AT: Sound {} wurde freigegeben (UNLOADSOUND)", idx))?
            .clone().with_settings(settings);
        let handle = self.sfx_track.play(data)
            .map_err(|e| format!("AUDIO_PLAY_AT: {:?}", e))?;
        let s = self.slot_mut(idx, "AUDIO_PLAY_AT")?;
        s.handle = Some(handle);
        s.vol = vol as f32;
        s.loops = if loops < 0 { -1 } else { loops };
        s.pan_anim = None;
        Ok(idx)
    }

    // ================= Clock (sample-genaues Musik-/Rhythmus-Timing) ======

    fn clock_slot(&self, idx: i64, fn_: &str) -> Result<&ClockSlot, String> {
        self.clocks.get(idx as usize).and_then(|c| c.as_ref())
            .ok_or_else(|| format!("{}: ungueltiges oder entferntes Uhr-Handle {}", fn_, idx))
    }
    fn clock_slot_mut(&mut self, idx: i64, fn_: &str) -> Result<&mut ClockSlot, String> {
        self.clocks.get_mut(idx as usize).and_then(|c| c.as_mut())
            .ok_or_else(|| format!("{}: ungueltiges oder entferntes Uhr-Handle {}", fn_, idx))
    }

    /// AUDIO_CLOCK_NEW(ticks_per_second) -- neue Uhr, startet PAUSIERT (Kira-
    /// Konvention -- explizit AUDIO_CLOCK_START noetig). `ticks_per_second`
    /// ist die einzige Geschwindigkeits-Einheit; BPM rechnet der Aufrufer
    /// selbst um (z.B. `bpm / 60.0 * subdivisions`).
    pub fn clock_new(&mut self, ticks_per_second: f64) -> Result<i64, String> {
        let handle = self.manager.add_clock(ClockSpeed::TicksPerSecond(ticks_per_second))
            .map_err(|e| format!(
                "AUDIO_CLOCK_NEW: Uhr konnte nicht angelegt werden (Limit erreicht?): {:?}", e))?;
        self.clocks.push(Some(ClockSlot { handle, ticking: false }));
        Ok((self.clocks.len() - 1) as i64)
    }

    pub fn clock_start(&mut self, idx: i64) -> Result<(), String> {
        let s = self.clock_slot_mut(idx, "AUDIO_CLOCK_START")?;
        s.handle.start();
        s.ticking = true;
        Ok(())
    }
    pub fn clock_pause(&mut self, idx: i64) -> Result<(), String> {
        let s = self.clock_slot_mut(idx, "AUDIO_CLOCK_PAUSE")?;
        s.handle.pause();
        s.ticking = false;
        Ok(())
    }
    /// Stoppt UND setzt auf Tick 0 zurueck (Kira-Semantik von `stop()` --
    /// anders als AUDIO_CLOCK_PAUSE, das die Position haelt).
    pub fn clock_stop(&mut self, idx: i64) -> Result<(), String> {
        let s = self.clock_slot_mut(idx, "AUDIO_CLOCK_STOP")?;
        s.handle.stop();
        s.ticking = false;
        Ok(())
    }
    pub fn clock_ticking(&self, idx: i64) -> Result<bool, String> {
        Ok(self.clock_slot(idx, "AUDIO_CLOCK_TICKING")?.ticking)
    }
    pub fn clock_ticks(&self, idx: i64) -> Result<i64, String> {
        Ok(self.clock_slot(idx, "AUDIO_CLOCK_TICKS")?.handle.time().ticks as i64)
    }
    pub fn clock_set_speed(&mut self, idx: i64, ticks_per_second: f64) -> Result<(), String> {
        self.clock_slot_mut(idx, "AUDIO_CLOCK_SET_SPEED")?
            .handle.set_speed(ClockSpeed::TicksPerSecond(ticks_per_second), tween_now());
        Ok(())
    }
    /// AUDIO_CLOCK_REMOVE(clock) -- gibt die Uhr frei. Kira kennt keine
    /// remove_clock()-Methode -- Entfernen laeuft ausschliesslich ueber
    /// Handle-Drop, darum ersetzt das den Vec-Slot durch `None` (Tombstone,
    /// wie UNLOADSOUND). Alles, was noch auf einen Tick DIESER Uhr wartet
    /// (AUDIO_PLAY_AT), startet danach nie mehr (Kira: WhenToStart::Never) --
    /// kein Crash, aber auch kein Nachtraeglich-Start.
    pub fn clock_remove(&mut self, idx: i64) -> Result<(), String> {
        let slot = self.clocks.get_mut(idx as usize)
            .ok_or_else(|| format!("AUDIO_CLOCK_REMOVE: ungueltiges Uhr-Handle {}", idx))?;
        *slot = None;
        Ok(())
    }

    // ================= Listener/Emitter (raeumliches Audio) ===============

    fn listener_slot(&self, idx: i64, fn_: &str) -> Result<&ListenerHandle, String> {
        self.listeners.get(idx as usize).and_then(|l| l.as_ref())
            .ok_or_else(|| format!("{}: ungueltiges oder entferntes Listener-Handle {}", fn_, idx))
    }
    fn listener_slot_mut(&mut self, idx: i64, fn_: &str) -> Result<&mut ListenerHandle, String> {
        self.listeners.get_mut(idx as usize).and_then(|l| l.as_mut())
            .ok_or_else(|| format!("{}: ungueltiges oder entferntes Listener-Handle {}", fn_, idx))
    }
    fn emitter_slot_mut(&mut self, idx: i64, fn_: &str) -> Result<&mut SpatialTrackHandle, String> {
        self.emitters.get_mut(idx as usize).and_then(|e| e.as_mut())
            .ok_or_else(|| format!("{}: ungueltiges oder entferntes Emitter-Handle {}", fn_, idx))
    }

    /// AUDIO_LISTENER_NEW(x, y, z) -> AUDIO_LISTENER -- das "Ohr" der Szene
    /// (typischerweise die Kamera-/Spielerposition). Orientierung startet
    /// unrotiert (blickt -Z); AUDIO_LISTENER_SET_ORIENTATION dreht um Y (Yaw).
    pub fn listener_new(&mut self, x: f64, y: f64, z: f64) -> Result<i64, String> {
        let handle = self.manager.add_listener(vec3(x, y, z), yaw_quat(0.0))
            .map_err(|e| format!(
                "AUDIO_LISTENER_NEW: Listener konnte nicht angelegt werden (Limit erreicht?): {:?}", e))?;
        self.listeners.push(Some(handle));
        Ok((self.listeners.len() - 1) as i64)
    }
    pub fn listener_set_position(&mut self, idx: i64, x: f64, y: f64, z: f64) -> Result<(), String> {
        self.listener_slot_mut(idx, "AUDIO_LISTENER_SET_POSITION")?
            .set_position(vec3(x, y, z), tween_now());
        Ok(())
    }
    /// `yaw_deg`: Drehung um die Y-Achse in Grad (0 = blickt -Z, Kira-Default).
    pub fn listener_set_orientation(&mut self, idx: i64, yaw_deg: f64) -> Result<(), String> {
        self.listener_slot_mut(idx, "AUDIO_LISTENER_SET_ORIENTATION")?
            .set_orientation(yaw_quat(yaw_deg), tween_now());
        Ok(())
    }
    /// AUDIO_LISTENER_REMOVE -- Kira hat kein remove_listener(), nur Handle-
    /// Drop. Emitter, die noch auf diesen Listener zeigen, liefern danach
    /// keine sinnvolle Panning-Berechnung mehr (kein Crash, aber undefiniert
    /// -- vor dem Entfernen erst alle abhaengigen Emitter entfernen).
    pub fn listener_remove(&mut self, idx: i64) -> Result<(), String> {
        let slot = self.listeners.get_mut(idx as usize)
            .ok_or_else(|| format!("AUDIO_LISTENER_REMOVE: ungueltiges Listener-Handle {}", idx))?;
        *slot = None;
        Ok(())
    }

    /// AUDIO_EMITTER_NEW(listener, x, y, z[, min_dist[, max_dist]]) ->
    /// AUDIO_EMITTER -- ein raeumlicher Sub-Track (eigene Mini-Bus), an einen
    /// Listener + Position gebunden. Kira berechnet Lautstaerke-Abnahme
    /// (linear zwischen min_dist=laut/max_dist=lautlos) und Stereo-Panning aus
    /// der Position relativ zum Listener selbst -- keine eigene DSP noetig.
    /// Laeuft auf dem SFX-Bus (Effekte/Busse gelten weiterhin).
    pub fn emitter_new(&mut self, listener_idx: i64, x: f64, y: f64, z: f64,
                       min_dist: f64, max_dist: f64) -> Result<i64, String> {
        let listener_id = self.listener_slot(listener_idx, "AUDIO_EMITTER_NEW")?.id();
        let builder = SpatialTrackBuilder::new()
            .distances((min_dist as f32, max_dist.max(min_dist + 0.01) as f32));
        let handle = self.sfx_track.add_spatial_sub_track(listener_id, vec3(x, y, z), builder)
            .map_err(|e| format!(
                "AUDIO_EMITTER_NEW: Emitter konnte nicht angelegt werden (Limit erreicht?): {:?}", e))?;
        self.emitters.push(Some(handle));
        Ok((self.emitters.len() - 1) as i64)
    }
    pub fn emitter_set_position(&mut self, idx: i64, x: f64, y: f64, z: f64) -> Result<(), String> {
        self.emitter_slot_mut(idx, "AUDIO_EMITTER_SET_POSITION")?
            .set_position(vec3(x, y, z), tween_now());
        Ok(())
    }
    /// AUDIO_EMITTER_REMOVE -- wie bei Clock/Listener: Kira entfernt spatiale
    /// Sub-Tracks nur per Handle-Drop.
    pub fn emitter_remove(&mut self, idx: i64) -> Result<(), String> {
        let slot = self.emitters.get_mut(idx as usize)
            .ok_or_else(|| format!("AUDIO_EMITTER_REMOVE: ungueltiges Emitter-Handle {}", idx))?;
        *slot = None;
        Ok(())
    }

    /// AUDIO_PLAY_ON(sound, emitter[, loops[, volume[, fade_in_ms[, easing$]]]])
    /// -- wie AUDIO_PLAY, aber der Sound startet auf dem raeumlichen Emitter-
    /// Track statt dem flachen SFX-Bus (Panning/Lautstaerke folgen der
    /// Emitter-Position relativ zum gebundenen Listener). Der SoundSlot-
    /// Handle ist danach identisch nutzbar (AUDIO_STOP/PAUSE/VOLUME/...) --
    /// `StaticSoundHandle` unterscheidet nicht, von welchem Track-Typ es kommt.
    pub fn ch_play_on(&mut self, idx: i64, emitter_idx: i64, loops: i64, volume: f64,
                      fade_in_ms: i64, easing: &str) -> Result<i64, String> {
        let curve = FadeCurve::parse(easing, "AUDIO_PLAY_ON")?;
        let vol = volume.clamp(0.0, 1.0);
        if let Some(h) = self.slot_mut(idx, "AUDIO_PLAY_ON")?.handle.as_mut() { h.stop(tween_now()); }
        let mut settings = StaticSoundSettings::new();
        if loops < 0 { settings = settings.loop_region(0.0..); }
        settings = settings.volume(if fade_in_ms > 0 { Decibels::SILENCE } else { db(vol) });
        let data = self.slot(idx, "AUDIO_PLAY_ON")?.data.as_ref()
            .ok_or_else(|| format!("AUDIO_PLAY_ON: Sound {} wurde freigegeben (UNLOADSOUND)", idx))?
            .clone().with_settings(settings);
        let emitter = self.emitter_slot_mut(emitter_idx, "AUDIO_PLAY_ON")?;
        let mut handle = emitter.play(data).map_err(|e| format!("AUDIO_PLAY_ON: {:?}", e))?;
        if fade_in_ms > 0 { handle.set_volume(db(vol), tween_ms_eased(fade_in_ms, curve)); }
        let s = self.slot_mut(idx, "AUDIO_PLAY_ON")?;
        s.handle = Some(handle);
        s.vol = vol as f32;
        s.loops = if loops < 0 { -1 } else { loops };
        s.pan_anim = None;
        Ok(idx)
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

    // ================= Mixer-Busse =================
    /// AUDIO_BUS_VOLUME(bus$, vol) -- Master-Lautstaerke eines Busses
    /// ("sfx", "music", "master"), 0..1. Multipliziert sich mit den
    /// einzelnen Sound-/Musik-Volumes (Kira-Mixer).
    pub fn set_bus_volume(&mut self, bus: &str, v: f64) -> Result<(), String> {
        let vol = v.clamp(0.0, 1.0);
        match bus.to_lowercase().as_str() {
            "sfx" => { self.sfx_bus = vol as f32; self.sfx_track.set_volume(db(vol), tween_now()); }
            "music" => { self.music_bus = vol as f32; self.music_track.set_volume(db(vol), tween_now()); }
            "master" => { self.master_bus = vol as f32; self.manager.main_track().set_volume(db(vol), tween_now()); }
            other => return Err(format!(
                "AUDIO_BUS_VOLUME: unbekannter Bus '{}' (sfx, music, master)", other)),
        }
        Ok(())
    }
    pub fn get_bus_volume(&self, bus: &str) -> Result<f64, String> {
        Ok(match bus.to_lowercase().as_str() {
            "sfx" => self.sfx_bus,
            "music" => self.music_bus,
            "master" => self.master_bus,
            other => return Err(format!(
                "AUDIO_BUS_GET_VOLUME: unbekannter Bus '{}' (sfx, music, master)", other)),
        } as f64)
    }

    fn bus_fx(&mut self, bus: &str, fn_: &str) -> Result<&mut BusFx, String> {
        match bus.to_lowercase().as_str() {
            "sfx" => Ok(&mut self.sfx_fx),
            "music" => Ok(&mut self.music_fx),
            "master" => Ok(&mut self.master_fx),
            other => Err(format!("{}: unbekannter Bus '{}' (sfx, music, master)", fn_, other)),
        }
    }

    /// AUDIO_FILTER(bus$, cutoff_hz[, resonance]) -- Tiefpass auf einem Bus.
    /// cutoff_hz <= 0 oder >= 20000 = offen (Effekt aus). resonance 0..1
    /// (Betonung um den Cutoff -- der "weeoow"-SID/Acid-Charakter).
    pub fn set_filter(&mut self, bus: &str, cutoff_hz: f64, resonance: f64) -> Result<(), String> {
        let c = if cutoff_hz <= 0.0 { 20000.0 } else { cutoff_hz.clamp(20.0, 20000.0) };
        let r = resonance.clamp(0.0, 1.0);
        let fx = self.bus_fx(bus, "AUDIO_FILTER")?;
        fx.filter.set_cutoff(c, tween_now());
        fx.filter.set_resonance(r, tween_now());
        Ok(())
    }

    /// AUDIO_REVERB(bus$, mix[, feedback[, damping]]) -- Hall auf einem Bus.
    /// mix 0..1 (0 = aus), feedback 0..1 (Nachhall-Laenge), damping 0..1
    /// (Hoehen-Daempfung). Fuer Hoehlen/Hallen.
    pub fn set_reverb(&mut self, bus: &str, mix: f64, feedback: f64, damping: f64) -> Result<(), String> {
        let fx = self.bus_fx(bus, "AUDIO_REVERB")?;
        fx.reverb.set_mix(Mix(mix.clamp(0.0, 1.0) as f32), tween_now());
        fx.reverb.set_feedback(feedback.clamp(0.0, 1.0), tween_now());
        fx.reverb.set_damping(damping.clamp(0.0, 1.0), tween_now());
        Ok(())
    }

    /// AUDIO_DELAY(bus$, mix[, feedback[, time_ms]]) -- Echo auf einem Bus.
    /// mix 0..1 (0 = aus), feedback 0..0.95 (Abfall pro Wiederholung), time_ms
    /// = Echo-Zeit (zur Laufzeit aenderbar, 1..4000; <=0 = unveraendert lassen).
    pub fn set_delay(&mut self, bus: &str, mix: f64, feedback: f64, time_ms: i64) -> Result<(), String> {
        let d = &self.bus_fx(bus, "AUDIO_DELAY")?.delay;
        d.mix.store((mix.clamp(0.0, 1.0) as f32).to_bits(), Relaxed);
        d.feedback.store((feedback.clamp(0.0, 0.95) as f32).to_bits(), Relaxed);
        if time_ms > 0 {
            d.delay_ms.store((time_ms.clamp(1, (DELAY_MAX_S * 1000.0) as i64)) as u32, Relaxed);
        }
        Ok(())
    }

    /// AUDIO_DISTORTION(bus$, amount[, mix]) -- Overdrive/Fuzz auf einem Bus.
    /// amount 0..1 -> 0..36 dB Drive in den Clipper; amount<=0 = aus. mix
    /// 0..1 (Default 1.0 = voll verzerrt).
    pub fn set_distortion(&mut self, bus: &str, amount: f64, mix: f64) -> Result<(), String> {
        let drive_db = (amount.clamp(0.0, 1.0) * 36.0) as f32;
        let m = if amount <= 0.0 { 0.0 } else { mix.clamp(0.0, 1.0) as f32 };
        let fx = self.bus_fx(bus, "AUDIO_DISTORTION")?;
        fx.distortion.set_drive(Decibels(drive_db), tween_now());
        fx.distortion.set_mix(Mix(m), tween_now());
        Ok(())
    }

    /// AUDIO_COMPRESSOR(bus$, threshold_db, ratio[, makeup_db]) -- Dynamik-
    /// Kompressor (Glue/Pump). threshold_db typ. -24..0, ratio >= 1 (1 = aus),
    /// makeup_db hebt den Pegel danach an. ratio<=1 schaltet den Effekt aus.
    pub fn set_compressor(&mut self, bus: &str, threshold_db: f64, ratio: f64, makeup_db: f64) -> Result<(), String> {
        let active = ratio > 1.0;
        let fx = self.bus_fx(bus, "AUDIO_COMPRESSOR")?;
        fx.compressor.set_threshold(threshold_db, tween_now());
        fx.compressor.set_ratio(ratio.max(1.0), tween_now());
        fx.compressor.set_makeup_gain(Decibels(makeup_db as f32), tween_now());
        fx.compressor.set_mix(Mix(if active { 1.0 } else { 0.0 }), tween_now());
        Ok(())
    }

    /// AUDIO_EQ(bus$, freq_hz, gain_db[, q]) -- parametrischer Glocken-EQ
    /// (eine Band) auf einem Bus. gain_db 0 = transparent (kein Effekt),
    /// >0 anheben / <0 absenken; q = Bandbreite (hoeher = schmaler).
    pub fn set_eq(&mut self, bus: &str, freq_hz: f64, gain_db: f64, q: f64) -> Result<(), String> {
        let f = freq_hz.clamp(20.0, 20000.0);
        let fx = self.bus_fx(bus, "AUDIO_EQ")?;
        fx.eq.set_frequency(f, tween_now());
        fx.eq.set_gain(Decibels(gain_db as f32), tween_now());
        fx.eq.set_q(q.max(0.05), tween_now());
        Ok(())
    }

    // ================= Core SFX =================
    pub fn load_sound(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let data = StaticSoundData::from_file(&resolved)
            .map_err(|e| format!("LOADSOUND: {:?}", e))?;
        Ok(self.push_slot(data, 1.0))
    }

    pub fn play_sound(&mut self, idx: i64, volume: f64) -> Result<(), String> {
        self.start_slot(idx, "PLAYSOUND", volume, 0, 0, FadeCurve::Linear)
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
               stereo_width: f64,
               duty: f64, pwm_depth: f64, pwm_speed: f64,
               flt_cutoff: f64, flt_sweep: f64, flt_res: f64) -> Result<i64, String> {
        let fx = SidFx { duty, pwm_depth, pwm_speed, flt_cutoff, flt_sweep, flt_res };
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
        let left = build_sfx_buffer(&wf, base_freq, slide, n, na, nd, vib_depth, vib_speed, sr, fx);
        let data = if stereo_width <= 0.0 {
            self.make_data_mono(&left, volume, sr)
        } else {
            let w = stereo_width.min(1.0);
            let right = if wf == "noise" {
                build_sfx_buffer(&wf, base_freq, slide, n, na, nd, vib_depth, vib_speed, sr, fx)
            } else {
                let detune = 1.0 + 0.04 * w;
                build_sfx_buffer(&wf, base_freq * detune, slide * detune, n, na, nd, vib_depth, vib_speed, sr, fx)
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
            return self.start_slot(snd, "SAMPLE_PLAY", v, 0, 0, FadeCurve::Linear).map(|_| snd);
        }
        let sr = self.samples[si].sr;
        let s = &self.samples[si];
        let buf = resample(&s.data, s.sr, ratio, dur_ms, s.loop_start, s.loop_end);
        if buf.is_empty() { return Err("SAMPLE_PLAY: leeres Sample".into()); }
        // Volume nicht in den Cache backen -> bei vol=1.0 bauen, per Slot setzen.
        let data = self.make_data_mono(&buf, 1.0, sr);
        let snd = self.push_slot(data, 1.0);
        self.sample_cache.insert(key, snd);
        self.start_slot(snd, "SAMPLE_PLAY", v, 0, 0, FadeCurve::Linear)?;
        Ok(snd)
    }

    // ================= Channel-Steuerung (AUDIO_*) =================
    pub fn ch_play(&mut self, idx: i64, loops: i64, volume: f64, fade_in_ms: i64,
                   easing: &str) -> Result<i64, String> {
        let curve = FadeCurve::parse(easing, "AUDIO_PLAY")?;
        self.start_slot(idx, "AUDIO_PLAY", volume, loops, fade_in_ms, curve)?;
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
    pub fn ch_stop(&mut self, idx: i64, fade_out_ms: i64, easing: &str) -> Result<(), String> {
        let curve = FadeCurve::parse(easing, "AUDIO_STOP")?;
        let s = self.slot_mut(idx, "AUDIO_STOP")?;
        s.loops = 0;
        if let Some(h) = s.handle.as_mut() {
            h.stop(if fade_out_ms > 0 { tween_ms_eased(fade_out_ms, curve) } else { tween_now() });
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
    pub fn ch_pan_slide(&mut self, idx: i64, from: f64, to: f64, dur_ms: i64,
                        easing: &str) -> Result<(), String> {
        let curve = FadeCurve::parse(easing, "AUDIO_PAN_SLIDE")?;
        let (from, to) = (from.clamp(0.0, 1.0), to.clamp(0.0, 1.0));
        let s = self.slot_mut(idx, "AUDIO_PAN_SLIDE")?;
        s.pan_anim = None;
        if let Some(h) = s.handle.as_mut() {
            h.set_panning(pan_of(from), tween_now());                    // Startpunkt
            h.set_panning(pan_of(to), tween_ms_eased(dur_ms, curve));    // Kira tweent nativ
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
            // Einmal parsen, um Fehler frueh zu melden; gespielt wird aus den Bytes.
            Module::load(&bytes).map_err(|e| format!("AUDIO_MUSIC_LOAD: {:?}", e))?;
            MusicSource::Module(bytes)
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

    fn start_music(&mut self, vol: f32, loops: i64, fade_in_ms: i64, curve: FadeCurve) -> Result<(), String> {
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
                MusicHandle::Stream(self.music_track.play(data)   // Musik-Bus
                    .map_err(|e| format!("AUDIO_MUSIC_PLAY: {:?}", e))?)
            }
            Some(MusicSource::Module(bytes)) => {
                // Modul frisch parsen + leaken -> 'static-Borrow fuer den
                // Player; der ModuleSound gibt das Modul im Drop frei.
                let module = Module::load(bytes)
                    .map_err(|e| format!("AUDIO_MUSIC_PLAY: {:?}", e))?;
                let module_ptr = Box::into_raw(Box::new(module));
                let loop_max = if endless { 0 } else { (loops as usize) + 1 };
                // Fade-in: bei 0 starten und im Sound hochrampen.
                let shared = ModShared::new(if fade_in_ms > 0 { 0.0 } else { vol }, self.music_pitch);
                if fade_in_ms > 0 {
                    shared.fade_ms.store(fade_in_ms as u32, Relaxed);
                    shared.fade_curve.store(curve.as_u8(), Relaxed);
                    shared.vol_target.store(vol.to_bits(), Relaxed);
                }
                let data = ModuleSoundData { module_ptr, loop_max, shared };
                MusicHandle::Module(self.music_track.play(data)   // Musik-Bus
                    .map_err(|_| "AUDIO_MUSIC_PLAY: Modul konnte nicht gestartet werden".to_string())?)
            }
        };
        // Fade-in fuer Stream/Static (Module macht es selbst, s.o.).
        if fade_in_ms > 0 && !matches!(handle, MusicHandle::Module(_)) {
            mh_set_volume(&mut handle, db(vol as f64), tween_ms_eased(fade_in_ms, curve));
        }
        self.music_handle = Some(handle);
        self.music_loops = if endless { -1 } else { loops };
        self.music_paused = false;
        Ok(())
    }

    pub fn play_music(&mut self, path: &str, volume: f64) -> Result<(), String> {
        self.music_vol = volume.clamp(0.0, 1.0) as f32;
        self.music_load(path)?;
        self.start_music(self.music_vol, -1, 0, FadeCurve::Linear)
    }
    pub fn stop_music(&mut self) {
        if let Some(h) = self.music_handle.as_mut() { mh_stop(h, tween_now()); }
        self.music_handle = None;
    }
    pub fn music_play(&mut self, loops: i64, fade_in_ms: i64, easing: &str) -> Result<(), String> {
        let curve = FadeCurve::parse(easing, "AUDIO_MUSIC_PLAY")?;
        let v = self.music_vol;
        self.start_music(v, loops, fade_in_ms, curve)
    }
    pub fn music_stop(&mut self, fade_out_ms: i64, easing: &str) -> Result<(), String> {
        let curve = FadeCurve::parse(easing, "AUDIO_MUSIC_STOP")?;
        // Review-Fund: `ch_stop` (normale Kanaele) setzt `loops = 0` VOR dem
        // eigentlichen Stop -- music_stop tat das nie. `update()` sieht einen
        // Fade-Stop als normales "Ende erreicht" (music_ended), und mit noch
        // positivem `music_loops` (endliche Wiederholungen) startete der Song
        // einen Frame nach dem beabsichtigten Fade-Out wieder von vorne, auf
        // voller Lautstaerke. Endlos-Loops (-1) und Tracker-Module waren
        // unbetroffen (music_loops bleibt -1 bzw. `is_module`-Gate greift).
        self.music_loops = 0;
        match self.music_handle.as_mut() {
            Some(MusicHandle::Module(a)) if fade_out_ms > 0 => {
                // Ausfaden auf 0 und am Ende beenden (Sound raeumt sich selbst).
                a.fade_ms.store(fade_out_ms as u32, Relaxed);
                a.fade_curve.store(curve.as_u8(), Relaxed);
                a.finish_at_zero.store(true, Relaxed);
                a.vol_target.store(0.0f32.to_bits(), Relaxed);
            }
            Some(h) => mh_stop(h, if fade_out_ms > 0 { tween_ms_eased(fade_out_ms, curve) } else { tween_now() }),
            None => {}
        }
        if fade_out_ms <= 0 { self.music_handle = None; }
        Ok(())
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
        // Review-Fund: nur nach unten geklemmt -- ein absurd hoher Faktor
        // (z.B. AUDIO_MUSIC_PITCH 1000000) laesst ModuleSound::process
        // (`self.frac += pitch; while self.frac >= 1.0 { pull(); }`) pro
        // Ausgabe-Sample ~pitch-mal den Tracker-Player aufrufen -- auf dem
        // ECHTZEIT-Audio-Thread, also Dropouts/Freeze statt nur einer
        // Verzerrung. 64x Geschwindigkeit ist bereits weit jenseits jedes
        // musikalischen Sinns, deckelt aber die Worst-Case-Arbeit pro Sample.
        self.music_pitch = factor.clamp(0.0, 64.0) as f32;
        let r = self.music_pitch as f64;
        if let Some(h) = self.music_handle.as_mut() { mh_set_rate(h, r, tween_now()); }
    }
    pub fn music_get_pitch(&self) -> f64 { self.music_pitch as f64 }
    pub fn music_position(&self) -> f64 {
        match self.music_handle.as_ref() {
            Some(MusicHandle::Stream(h)) => h.position(),
            Some(MusicHandle::Static(h)) => h.position(),
            Some(MusicHandle::Module(a)) => {
                let sr = a.sr.load(Relaxed).max(1) as f64;
                a.pos_frames.load(Relaxed) as f64 / sr
            }
            None => 0.0,
        }
    }
    pub fn music_busy(&self) -> bool {
        matches!(self.music_handle.as_ref().map(mh_state), Some(PlaybackState::Playing))
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
                // Review-Fund: `start_slot` setzt `pan_anim = None` (richtig
                // fuer einen ECHTEN neuen AUDIO_PLAY-Aufruf) -- dieser interne
                // Neustart fuer eine endliche Wiederholung ist aber dieselbe
                // logische Wiedergabe, kein neuer AUDIO_PLAY. Ohne diese
                // Rettung stoppte AUTOPAN nach der ersten Wiederholung eines
                // Sounds mit `AUDIO_PLAY(ch, 3)` + `AUDIO_AUTOPAN(ch, ...)`.
                let (vol, rem, pan) = { let s = &self.sounds[i]; (s.vol as f64, s.loops - 1, s.pan_anim) };
                let _ = self.start_slot(i as i64, "AUDIO_PLAY", vol, 0, 0, FadeCurve::Linear);
                self.sounds[i].loops = rem;
                self.sounds[i].pan_anim = pan;
            }
        }
        // Musik: endliche Loops + Queue.
        let music_ended = matches!(self.music_handle.as_ref().map(mh_state), Some(PlaybackState::Stopped))
            && !self.music_paused;
        if music_ended {
            // Module zaehlen ihre Loops selbst (loop_max im Player) -> nicht
            // erneut neu starten; nur Stream/Static brauchen die Restart-Zaehlung.
            let is_module = matches!(self.music_handle.as_ref(), Some(MusicHandle::Module(_)));
            if self.music_loops > 0 && !is_module {
                self.music_loops -= 1;
                let v = self.music_vol;
                let _ = self.start_music(v, 0, 0, FadeCurve::Linear);
            } else if let Some(path) = self.music_queue.take() {
                let v = self.music_vol;
                // Queue-Track laden (Stream oder Modul) und endlos starten.
                if self.music_load(&path).is_ok() { let _ = self.start_music(v, -1, 0, FadeCurve::Linear); }
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

/// Parameter der SID-Erweiterungen von AUDIO_SFX (Pulsbreite/PWM + resonanter
/// Tiefpass-Sweep). Defaults reproduzieren exakt den bisherigen Klang.
#[derive(Clone, Copy)]
struct SidFx {
    duty: f64, pwm_depth: f64, pwm_speed: f64,
    flt_cutoff: f64, flt_sweep: f64, flt_res: f64,
}
impl Default for SidFx {
    fn default() -> Self {
        SidFx { duty: 0.5, pwm_depth: 0.0, pwm_speed: 0.0,
                flt_cutoff: 0.0, flt_sweep: 0.0, flt_res: 0.0 }
    }
}

/// Resonanter Tiefpass (Chamberlin SVF) mit ueber die Zeit gleitender
/// Grenzfrequenz -- der SID/Acid-Sweep. Identisch zu `synth.svf_lowpass`
/// (Editor-Vorschau), damit beide gleich klingen. Pure -> Rust-`#[test]`.
fn svf_lowpass(wave: &mut [f64], cutoff: f64, sweep: f64, res: f64, sr: u32) {
    if cutoff <= 0.0 || wave.is_empty() { return; }
    let q = 1.0 - res.clamp(0.0, 0.95);
    let nyq = sr as f64 * 0.5;
    let (mut low, mut band) = (0.0f64, 0.0f64);
    for i in 0..wave.len() {
        let fc = (cutoff + sweep * (i as f64 / sr as f64)).clamp(10.0, nyq * 0.99);
        let f = 2.0 * (PI64 * fc / sr as f64).sin();
        low += f * band;
        let high = wave[i] - low - q * band;
        band += f * high;
        wave[i] = low.clamp(-1.0, 1.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn build_sfx_buffer(wf: &str, base_freq: f64, slide: f64, n: usize,
                    na: usize, nd: usize, vib_depth: f64, vib_speed: f64,
                    sr: u32, fx: SidFx) -> Vec<f64> {
    let mut buf = vec![0.0f64; n];
    let mut phase = 0.0f64;
    for i in 0..n {
        let t = i as f64 / sr as f64;
        let mut freq = base_freq + slide * t;
        if vib_depth > 0.0 && vib_speed > 0.0 {
            freq *= 1.0 + vib_depth * (2.0 * PI64 * vib_speed * t).sin();
        }
        freq = freq.clamp(20.0, sr as f64 / 2.0);
        buf[i] = if wf == "noise" {
            rng_uniform()
        } else {
            phase += 2.0 * PI64 * freq / sr as f64;
            if wf == "square" {
                // Pulsbreite (ph mod 1 < duty); duty=0.5 == sign(sin) (alt).
                let ph = phase / (2.0 * PI64);
                let frac = ph - ph.floor();
                let duty_t = if fx.pwm_depth > 0.0 && fx.pwm_speed > 0.0 {
                    fx.duty + fx.pwm_depth * (2.0 * PI64 * fx.pwm_speed * t).sin()
                } else { fx.duty }.clamp(0.05, 0.95);
                if frac < duty_t { 1.0 } else { -1.0 }
            } else {
                phase_value(wf, phase)
            }
        };
    }
    if fx.flt_cutoff > 0.0 {
        svf_lowpass(&mut buf, fx.flt_cutoff, fx.flt_sweep, fx.flt_res, sr);
    }
    for i in 0..n {
        let env = if na > 0 && i < na {
            i as f64 / na as f64
        } else if nd > 0 && i >= n - nd {
            (n - 1 - i) as f64 / nd as f64
        } else { 1.0 };
        buf[i] = (buf[i] * env).clamp(-1.0, 1.0);
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
    use super::{build_sfx_buffer, db, lofi_chain, pendulum_pos, resample,
                svf_lowpass, Decibels, FadeCurve, SidFx, yaw_quat};
    use kira::Tweenable;

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

    #[test]
    fn square_duty_default_matches_sign_of_sin() {
        // duty=0.5 (Default) muss exakt die alte sign(sin)-Rechteckwelle sein.
        let n = 441;
        let buf = build_sfx_buffer("square", 440.0, 0.0, n, 0, 0, 0.0, 0.0,
                                   44100, SidFx::default());
        let mut phase = 0.0f64;
        for v in buf.iter().take(n) {
            phase += 2.0 * super::PI64 * 440.0 / 44100.0;
            let expect = if phase.sin() >= 0.0 { 1.0 } else { -1.0 };
            assert!((v - expect).abs() < 1e-9);
        }
    }

    #[test]
    fn square_duty_narrow_spends_less_time_high() {
        // Schmale Pulsbreite (10%) -> Welle ist ueberwiegend bei -1.
        let n = 4410;
        let fx = SidFx { duty: 0.1, ..SidFx::default() };
        let buf = build_sfx_buffer("square", 220.0, 0.0, n, 0, 0, 0.0, 0.0, 44100, fx);
        let high = buf.iter().filter(|&&v| v > 0.0).count();
        assert!(high < n / 3, "schmaler Puls sollte selten high sein, war {high}/{n}");
    }

    #[test]
    fn svf_lowpass_attenuates_high_freq() {
        // Hochfrequentes Signal durch einen tiefen Cutoff -> deutlich leiser.
        let n = 4410;
        let mut sig: Vec<f64> = (0..n)
            .map(|i| (2.0 * super::PI64 * 8000.0 * i as f64 / 44100.0).sin())
            .collect();
        let before = sig.iter().map(|v| v.abs()).fold(0.0, f64::max);
        svf_lowpass(&mut sig, 300.0, 0.0, 0.0, 44100);
        let after = sig.iter().skip(n / 2).map(|v| v.abs()).fold(0.0, f64::max);
        assert!(after < before * 0.5, "Tiefpass sollte 8kHz daempfen: {after} vs {before}");
    }

    #[test]
    fn svf_cutoff_zero_is_bypass() {
        let mut sig = vec![0.3, -0.7, 0.5, -0.2];
        let copy = sig.clone();
        svf_lowpass(&mut sig, 0.0, 0.0, 0.0, 44100);
        assert_eq!(sig, copy);
    }

    #[test]
    fn fade_curve_parse_accepts_known_names_case_insensitive() {
        assert!(matches!(FadeCurve::parse("", "X").unwrap(), FadeCurve::Linear));
        assert!(matches!(FadeCurve::parse("LINEAR", "X").unwrap(), FadeCurve::Linear));
        assert!(matches!(FadeCurve::parse("In", "X").unwrap(), FadeCurve::In));
        assert!(matches!(FadeCurve::parse("OUT", "X").unwrap(), FadeCurve::Out));
        assert!(matches!(FadeCurve::parse("inout", "X").unwrap(), FadeCurve::InOut));
        assert!(FadeCurve::parse("bounce", "X").is_err());
    }

    #[test]
    fn fade_curve_endpoints_are_fixed() {
        // Jede Kurve muss bei t=0 -> 0 und t=1 -> 1 beginnen/enden (sonst
        // springt die Lautstaerke am Fade-Rand).
        for c in [FadeCurve::Linear, FadeCurve::In, FadeCurve::Out, FadeCurve::InOut] {
            assert!((c.apply(0.0) - 0.0).abs() < 1e-6);
            assert!((c.apply(1.0) - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn fade_curve_matches_kira_easing_powi2() {
        // FadeCurve::apply() dupliziert Kiras Easing::{In,Out,InOut}Powi(2)-
        // Mathematik fuer den Modul-Fade-Pfad (kein Kira-Tween beteiligt) --
        // muss exakt uebereinstimmen, sonst klingen MOD-Fades anders als
        // Stream/Static-Fades mit derselben Kurve.
        let samples = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
        for &t in &samples {
            let in_expect = t * t;
            let out_expect = 1.0 - (1.0 - t) * (1.0 - t);
            let inout_expect = if t < 0.5 { 2.0 * t * t } else { 1.0 - 2.0 * (1.0 - t) * (1.0 - t) };
            assert!((FadeCurve::In.apply(t) - in_expect).abs() < 1e-6);
            assert!((FadeCurve::Out.apply(t) - out_expect).abs() < 1e-6);
            assert!((FadeCurve::InOut.apply(t) - inout_expect).abs() < 1e-6);
        }
    }

    #[test]
    fn module_fade_ramp_matches_kira_decibels_tween() {
        // Review-Fund: der MOD/XM-Modul-Fade-Pfad (ModuleSound::process)
        // interpolierte bisher DIREKT die lineare Amplitude (`ramp_start +
        // (tgt - ramp_start) * t`), waehrend der Kira-Tween-Pfad (AUDIO_PLAY/
        // STOP/AUDIO_MUSIC_PLAY/STOP fade_in/out_ms) ueber Kiras `Tweenable
        // for Decibels` linear-in-DEZIBEL interpoliert und erst am Ende
        // exponentiell zu Amplitude konvertiert -- zwei unterschiedliche
        // Kurven fuer denselben Start/Ziel-Wert (dB ist eine log-Skala).
        // Dieser Test verifiziert, dass die (jetzt korrigierte) Formel im
        // Modul-Pfad exakt Kiras eigene `Decibels::interpolate` +
        // `as_amplitude` nachbildet, statt nur die Formung (FadeCurve::apply)
        // gegen sich selbst zu pruefen wie `fade_curve_matches_kira_easing_powi2`.
        let vol_pairs = [(1.0, 0.0), (0.0, 1.0), (0.3, 0.8), (0.6, 0.1)];
        let ts = [0.0f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
        for &(v_start, v_target) in &vol_pairs {
            for &t in &ts {
                // Modul-Pfad-Formel (audio.rs ModuleSound::process).
                let db_start = db(v_start).0;
                let db_tgt = db(v_target).0;
                let got = Decibels(db_start + (db_tgt - db_start) * t).as_amplitude();
                // Kiras eigene Tweenable-Interpolation ueber Decibels.
                let expect = Decibels::interpolate(db(v_start), db(v_target), t as f64).as_amplitude();
                assert!((got - expect).abs() < 1e-5,
                    "v_start={v_start} v_target={v_target} t={t}: got {got}, expected {expect} (Kira-Tween)");
            }
        }
    }

    #[test]
    fn fade_curve_in_starts_slower_than_out() {
        // "in" beschleunigt erst spaet (leiser Start), "out" ist frueh schon
        // lauter -- am selben t muss out >= in gelten (ausser an den Enden).
        for t in [0.1, 0.3, 0.5, 0.7, 0.9] {
            assert!(FadeCurve::Out.apply(t) >= FadeCurve::In.apply(t));
        }
    }

    #[test]
    fn yaw_quat_zero_is_identity() {
        // 0 Grad = unrotiert -> Kiras Default-Blickrichtung (-Z) bleibt.
        let q = yaw_quat(0.0);
        assert!((q.s - 1.0).abs() < 1e-6);
        assert!(q.v.x.abs() < 1e-6);
        assert!(q.v.y.abs() < 1e-6);
        assert!(q.v.z.abs() < 1e-6);
    }

    #[test]
    fn yaw_quat_is_unit_length() {
        // Jede gueltige Rotation muss ein Einheits-Quaternion sein, sonst
        // verzerrt Kira die Panning-Berechnung.
        for deg in [0.0, 45.0, 90.0, 180.0, -90.0, 270.0, 720.0] {
            let q = yaw_quat(deg);
            let len_sq = q.s * q.s + q.v.x * q.v.x + q.v.y * q.v.y + q.v.z * q.v.z;
            assert!((len_sq - 1.0).abs() < 1e-5, "deg={deg} len_sq={len_sq}");
        }
    }

    #[test]
    fn yaw_quat_90_degrees_matches_expected_components() {
        // 90 Grad um Y: s=cos(45deg), v.y=sin(45deg), x/z bleiben 0.
        let q = yaw_quat(90.0);
        let expect = (std::f64::consts::FRAC_PI_4).cos() as f32;
        assert!((q.s - expect).abs() < 1e-6);
        assert!((q.v.y - expect).abs() < 1e-6);
        assert!(q.v.x.abs() < 1e-6);
        assert!(q.v.z.abs() < 1e-6);
    }
}
