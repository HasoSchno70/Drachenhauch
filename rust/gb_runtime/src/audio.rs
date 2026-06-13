//! Native Audio ueber raylib -- Core-Builtins LOADSOUND/PLAYSOUND/STOPSOUND
//! (SFX) sowie PLAYMUSIC/STOPMUSIC (Stream-Musik). Nur mit `--features
//! graphics` (raylib bundelt Audio mit).
//!
//! Audio-Output ist -- wie RND/MILLIS/Tween -- naturgemaess nicht
//! deterministisch golden-testbar; getestet wird die Argument-Validierung.
//!
//! Lifetime-Trick: `Sound<'aud>`/`Music<'aud>` borgen das `RaylibAudio`-
//! Geraet. Da das Geraet den ganzen Prozess lebt, wird es per `Box::leak`
//! zu `&'static` gemacht -- so lassen sich Sounds/Musik in `Vec`/`Option`
//! halten, ohne ein self-referential struct zu bauen.
#![cfg(feature = "graphics")]

use std::collections::HashMap;
use std::os::raw::{c_uint, c_void};
use std::sync::Mutex;

use raylib::core::audio::{Music, RaylibAudio, Sound};

/// Ein geladenes PCM-Sample (mono, normalisiert [-1,1]) fuer den Amiga-Stil-
/// Sampler `SAMPLE_*`. Wiedergabe erfolgt resampled (Tonhoehe = Geschwindigkeit,
/// genau wie Paula): `SAMPLE_PLAY(sample, halbtoene, vol)`.
struct Sample {
    data: Vec<f32>,      // mono PCM, [-1,1]
    sr: u32,             // Quell-Sample-Rate
    loop_start: usize,   // Loop-Region (Frames); aktiv wenn loop_end > loop_start
    loop_end: usize,
}

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

/// Laufender Fade (AUDIO_MUSIC_PLAY/AUDIO_PLAY fade_in, AUDIO_MUSIC_STOP/
/// AUDIO_STOP fade_out). Zeitbasiert -- `update()` (pro Frame aus dem
/// FLIP-Pfad) schreibt das interpolierte Volume auf Stream bzw. Sound.
struct Fade {
    from: f32,
    to: f32,
    start: std::time::Instant,
    dur_ms: f64,
    stop_after: bool,   // Fade-out: am Ende Stream/Sound stoppen
}

/// Fade-Fortschritt 0..1 (pure, fuer #[test]).
fn fade_progress(elapsed_ms: f64, dur_ms: f64) -> f32 {
    if dur_ms <= 0.0 { return 1.0; }
    (elapsed_ms / dur_ms).clamp(0.0, 1.0) as f32
}

impl Fade {
    fn current_vol(&self) -> f32 {
        let t = fade_progress(self.start.elapsed().as_secs_f64() * 1000.0, self.dur_ms);
        self.from + (self.to - self.from) * t
    }
    fn done(&self) -> bool {
        self.start.elapsed().as_secs_f64() * 1000.0 >= self.dur_ms
    }
}

/// Laufende Pan-Animation (AUDIO_PAN_SLIDE / AUDIO_AUTOPAN). Bewegt NUR das
/// Stereo-Pan (Position 0=links .. 1=rechts), nie das Volume -- kollidiert
/// dadurch nicht mit Fades. `update()` schreibt die Position pro Frame.
enum PanAnim {
    /// Einmalige Fahrt von -> nach ueber dur_ms; bleibt am Ziel stehen.
    Slide { from: f64, to: f64, start: std::time::Instant, dur_ms: f64 },
    /// Endloses Pendeln links<->rechts; eine volle Runde = period_s.
    Pendulum { period_s: f64, depth: f64, start: std::time::Instant },
}

/// Pendel-Position 0..1 (pure, fuer #[test]). Kosinus-foermig: startet auf
/// der linken Auslenkung (bei depth=1.0 ganz links), wandert nach rechts
/// und zurueck; depth skaliert die Auslenkung um die Mitte 0.5.
fn pendulum_pos(elapsed_s: f64, period_s: f64, depth: f64) -> f64 {
    let d = depth.clamp(0.0, 1.0);
    0.5 - 0.5 * d * (std::f64::consts::TAU * elapsed_s / period_s).cos()
}

impl PanAnim {
    /// (aktuelle Position 0..1, Animation fertig?)
    fn position(&self) -> (f64, bool) {
        match self {
            PanAnim::Slide { from, to, start, dur_ms } => {
                let t = fade_progress(start.elapsed().as_secs_f64() * 1000.0, *dur_ms) as f64;
                (from + (to - from) * t, t >= 1.0)
            }
            PanAnim::Pendulum { period_s, depth, start } =>
                (pendulum_pos(start.elapsed().as_secs_f64(), *period_s, *depth), false),
        }
    }
}

pub struct Audio {
    dev: &'static RaylibAudio,
    sounds: Vec<Sound<'static>>,
    sound_vol: Vec<f32>,        // getrackte Volumes (raylib hat keinen Getter)
    music: Option<Music<'static>>,
    music_vol: f32,            // getracktes Music-Volume
    music_queue: Option<String>, // AUDIO_MUSIC_QUEUE -> bei Stream-Ende abspielen
    music_loops: i64,          // verbleibende Wiederholungen nach dem aktuellen Durchlauf; -1 = endlos (raylib-looping)
    music_pitch: f32,          // getrackter Music-Pitch (ueberlebt LOAD/QUEUE)
    music_fade: Option<Fade>,
    music_paused: bool,        // AUDIO_MUSIC_PAUSE -- damit update() eine Pause nicht als Stream-Ende deutet
    ch_fade: Vec<Option<Fade>>, // pro Sound-Handle: laufender Fade (AUDIO_PLAY fade_in / AUDIO_STOP fade_out)
    ch_loops: Vec<i64>,        // pro Sound-Handle: verbleibende Wiederholungen nach dem aktuellen Durchlauf; -1 = endlos, 0 = keine
    ch_paused: Vec<bool>,      // AUDIO_PAUSE -- damit update() eine Pause nicht als Sound-Ende deutet
    ch_pan_anim: Vec<Option<PanAnim>>, // pro Sound-Handle: laufende Pan-Animation (SLIDE/AUTOPAN)
    num_channels: i64,         // emuliert (raylib hat kein festes Channel-Limit)
    bands: Vec<f32>,   // geglaettete Band-Pegel (Peak-Hold)
    agc: f32,          // Auto-Gain-Referenz (adaptiv)
    samples: Vec<Sample>,                       // SAMPLE_LOAD-Pool
    sample_cache: HashMap<(usize, i64, i64), i64>, // (sample, centi-halbtoene, dur_ms) -> Sound-Handle
    lofi: bool,        // Paula-Lo-Fi-Modus (Bit-Crush + LED-Tiefpass)
    lofi_bits: u32,    // Bit-Tiefe (Default 8 = Amiga)
    lofi_cutoff: f64,  // LED-Filter-Cutoff in Hz (0 = aus; Default 3300)
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
            music: None, music_vol: 1.0, music_queue: None,
            music_loops: -1, music_pitch: 1.0, music_fade: None, music_paused: false,
            ch_fade: Vec::new(), ch_loops: Vec::new(), ch_paused: Vec::new(),
            ch_pan_anim: Vec::new(),
            num_channels: 16,
            bands: Vec::new(), agc: 1e-4,
            samples: Vec::new(), sample_cache: HashMap::new(),
            lofi: false, lofi_bits: 8, lofi_cutoff: 3300.0,
        })
    }

    /// AUDIO_LOFI(an[, bits[, cutoff_hz]]): Paula/Amiga-Lo-Fi fuer folgende
    /// synthetisierte Sounds (AUDIO_TONE/NOISE/SFX + SAMPLE_PLAY). `bits`
    /// (1..16, Default 8) = Bit-Crush-Aufloesung, `cutoff_hz` (>=0, Default
    /// 3300, 0 = aus) = LED-Tiefpass. Wirkt erst auf NEU gebaute Sounds ->
    /// der Sample-Cache wird invalidiert.
    pub fn set_lofi(&mut self, on: bool, bits: u32, cutoff: f64) {
        self.lofi = on;
        self.lofi_bits = bits.clamp(1, 16);
        self.lofi_cutoff = cutoff.max(0.0);
        self.sample_cache.clear();
    }

    /// Paula-Lo-Fi-Kette (pur): Bit-Crush dann One-Pole-Tiefpass. Reihenfolge
    /// wie auf echtem Amiga: 8-bit-DAC zuerst, dann der analoge LED-Filter.
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

    // -- Amiga-Stil-Sampler (SAMPLE_LOAD / SAMPLE_PLAY) --
    //
    // Ein geladenes PCM-Sample wird ueber die ganze Klaviatur gespielt,
    // indem es resampled wird (hoehere Note = schneller abgespielt = hoeher --
    // genau wie Paula auf dem Amiga). Resampelte Varianten werden gecacht.

    /// SAMPLE_LOAD(path$) -> SAMPLE: WAV/OGG/QOA laden, auf mono normalisieren.
    pub fn sample_load(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let mut wave = self.dev.new_wave(&resolved)
            .map_err(|e| format!("SAMPLE_LOAD: {}", e))?;
        let sr = wave.sample_rate();
        wave.format(sr as i32, 16, 1);          // mono, gleiche Rate behalten
        let data: Vec<f32> = wave.load_samples().as_ref().to_vec();
        if data.is_empty() {
            return Err("SAMPLE_LOAD: Sample ist leer".into());
        }
        self.samples.push(Sample { data, sr, loop_start: 0, loop_end: 0 });
        Ok((self.samples.len() - 1) as i64)
    }

    /// SAMPLE_SET_LOOP(sample, start, end): Loop-Region in Frames der Quelle.
    /// Wirkt nur bei `SAMPLE_PLAY` mit Dauer (sustained note).
    pub fn sample_set_loop(&mut self, idx: i64, start: i64, end: i64) -> Result<(), String> {
        let s = self.samples.get_mut(idx as usize)
            .ok_or_else(|| format!("SAMPLE_SET_LOOP: ungueltiges SAMPLE-Handle {}", idx))?;
        let n = s.data.len();
        let start = start.max(0) as usize;
        let end = (end.max(0) as usize).min(n);
        if end <= start {
            return Err("SAMPLE_SET_LOOP: end muss > start sein".into());
        }
        s.loop_start = start;
        s.loop_end = end;
        // Cache invalidieren -- Loop aendert gebaute Buffer.
        self.sample_cache.retain(|k, _| k.0 != idx as usize);
        Ok(())
    }

    /// SAMPLE_LEN(sample) -> Sekunden bei Originaltonhoehe.
    pub fn sample_len(&self, idx: i64) -> Result<f64, String> {
        let s = self.samples.get(idx as usize)
            .ok_or_else(|| format!("SAMPLE_LEN: ungueltiges SAMPLE-Handle {}", idx))?;
        Ok(s.data.len() as f64 / s.sr as f64)
    }

    /// SAMPLE_PLAY(sample, halbtoene, vol[, dur_ms]) -> AUDIO_CHANNEL.
    /// `halbtoene` verschiebt die Tonhoehe relativ zum Original (12 = Oktave
    /// hoeher). `dur_ms<=0` = ganzes Sample einmal (One-Shot, fuer Drums/Hits);
    /// `dur_ms>0` = auf diese Laenge gebaut (mit Loop-Region wird geloopt,
    /// sonst danach Stille) -- fuer gehaltene Noten.
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
        let v = volume.clamp(0.0, 1.0) as f32;
        let key = (si, (semitones * 100.0).round() as i64, dur_ms);
        if let Some(&snd) = self.sample_cache.get(&key) {
            let s = self.snd(snd, "SAMPLE_PLAY")?;
            s.set_volume(v);
            s.play();
            self.reset_channel_state(snd as usize, v);
            return Ok(snd);
        }
        let sr = self.samples[si].sr;
        let buf = self.build_resampled(si, ratio, dur_ms);
        if buf.is_empty() {
            return Err("SAMPLE_PLAY: leeres Sample".into());
        }
        // vol=1.0 in den Buffer backen, Laufstaerke per set_volume -> Cache
        // ist lautstaerke-unabhaengig.
        let snd = self.push_wave_sound(&buf, 1.0, sr)?;
        self.sample_cache.insert(key, snd);
        let s = self.snd(snd, "SAMPLE_PLAY")?;
        s.set_volume(v);
        s.play();
        self.reset_channel_state(snd as usize, v);
        Ok(snd)
    }

    /// Resampling mit linearer Interpolation. One-Shot (dur_ms<=0) spielt das
    /// ganze Sample bei `ratio`; mit Dauer wird ein Buffer fester Laenge
    /// gebaut, der die Loop-Region wiederholt (falls gesetzt).
    fn build_resampled(&self, si: usize, ratio: f64, dur_ms: i64) -> Vec<f64> {
        let s = &self.samples[si];
        resample(&s.data, s.sr, ratio, dur_ms, s.loop_start, s.loop_end)
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

    /// Sound + parallelen Channel-State (Volume/Fade/Loops/Paused) registrieren.
    fn push_sound(&mut self, s: Sound<'static>, vol: f32) -> i64 {
        self.sounds.push(s);
        self.sound_vol.push(vol);
        self.ch_fade.push(None);
        self.ch_loops.push(0);
        self.ch_paused.push(false);
        self.ch_pan_anim.push(None);
        (self.sounds.len() - 1) as i64
    }

    pub fn load_sound(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let path = resolved.as_str();
        let s = self.dev.new_sound(path).map_err(|e| format!("LOADSOUND: {}", e))?;
        Ok(self.push_sound(s, 1.0))
    }

    fn sound(&self, idx: i64, fn_: &str) -> Result<&Sound<'static>, String> {
        self.sounds.get(idx as usize)
            .ok_or_else(|| format!("{}: ungueltiges SOUND-Handle {}", fn_, idx))
    }

    pub fn play_sound(&mut self, idx: i64, volume: f64) -> Result<(), String> {
        let v = volume.clamp(0.0, 1.0) as f32;
        {
            let s = self.sound(idx, "PLAYSOUND")?;
            s.set_volume(v);
            s.play();
        }
        self.reset_channel_state(idx as usize, v);
        Ok(())
    }

    pub fn stop_sound(&mut self, idx: i64) -> Result<(), String> {
        self.sound(idx, "STOPSOUND")?.stop();
        let v = self.sound_vol[idx as usize];
        self.reset_channel_state(idx as usize, v);
        Ok(())
    }

    /// Loop-/Fade-/Pause-Tracking eines Handles zuruecksetzen -- noetig bei
    /// jedem Neu-Start/Stopp, damit update() einen gestoppten Sound nicht
    /// per Rest-Loop wiederbelebt.
    fn reset_channel_state(&mut self, i: usize, vol: f32) {
        self.sound_vol[i] = vol;
        self.ch_fade[i] = None;
        self.ch_loops[i] = 0;
        self.ch_paused[i] = false;
        self.ch_pan_anim[i] = None;
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
        m.set_pitch(self.music_pitch);
        m.play_stream();
        self.music = Some(m);
        self.music_vol = v;
        self.music_queue = None;
        self.music_loops = -1;
        self.music_fade = None;
        self.music_paused = false;
        Ok(())
    }

    pub fn stop_music(&mut self) {
        if let Some(m) = self.music.take() { m.stop_stream(); }
        self.music_fade = None;
        self.music_paused = false;
    }

    /// Pro Frame aufrufen (aus dem FLIP-Pfad), damit der Musik-Stream
    /// nachgefuettert wird -- sonst stockt die Wiedergabe. Treibt ausserdem
    /// laufende Fades (AUDIO_MUSIC_PLAY/STOP + AUDIO_PLAY/AUDIO_STOP),
    /// wiederholt endliche Loops (Musik UND Sound-Channels) und spielt einen
    /// per AUDIO_MUSIC_QUEUE vorgemerkten Track ab, sobald der aktuelle endet.
    pub fn update(&mut self) {
        // 0) Sound-Channels: Fades fortschreiben + Loops wiederholen.
        //    raylib-Sounds koennen nicht nativ loopen -> beendete Sounds mit
        //    Rest-Loops neu starten (pygame-Semantik: loops=N -> N+1
        //    Durchlaeufe, -1 = endlos). Pause friert wie bei der Musik ein.
        for i in 0..self.sounds.len() {
            if self.ch_paused[i] { continue; }
            if let Some(f) = &self.ch_fade[i] {
                self.sounds[i].set_volume(f.current_vol());
                if f.done() {
                    let stop = f.stop_after;
                    self.ch_fade[i] = None;
                    if stop {
                        self.sounds[i].stop();
                        self.sounds[i].set_volume(self.sound_vol[i]);   // fuers naechste Play
                        self.ch_loops[i] = 0;
                    }
                }
            }
            if self.ch_loops[i] != 0 && !self.sounds[i].is_playing() {
                if self.ch_loops[i] > 0 { self.ch_loops[i] -= 1; }
                self.sounds[i].play();
            }
            if let Some(anim) = &self.ch_pan_anim[i] {
                let (p, done) = anim.position();
                // Position 0=links..1=rechts -> raylib-Pan (gespiegelt, wie ch_pan).
                self.sounds[i].set_pan((1.0 - p.clamp(0.0, 1.0)) as f32);
                if done { self.ch_pan_anim[i] = None; }
            }
        }
        if self.music.is_none() { return; }
        // 1) Fade fortschreiben (waehrend Pause eingefroren -- zeitbasiert,
        //    aber eine pausierte Musik soll nicht "unterm Eis" leiser werden).
        if !self.music_paused {
            if let (Some(f), Some(m)) = (&self.music_fade, &self.music) {
                m.set_volume(f.current_vol());
                if f.done() {
                    let stop = f.stop_after;
                    self.music_fade = None;
                    if stop {
                        if let Some(m) = &self.music {
                            m.stop_stream();
                            m.set_volume(self.music_vol);   // fuers naechste Play
                        }
                        self.music_loops = 0;
                    }
                }
            }
        }
        // 2) Stream nachfuettern + endliche Loops + Queue. is_stream_playing()
        //    ist auch bei Pause false -- music_paused unterscheidet das.
        if let Some(m) = &self.music {
            m.update_stream();
            if !m.is_stream_playing() && !self.music_paused {
                if self.music_loops > 0 {
                    // Endlicher Loop: raylib hat den beendeten Stream auf
                    // Position 0 gestoppt -> einfach neu starten.
                    self.music_loops -= 1;
                    m.play_stream();
                } else if let Some(path) = self.music_queue.take() {
                    if let Ok(nm) = self.dev.new_music(&path) {
                        nm.set_volume(self.music_vol);
                        nm.set_pitch(self.music_pitch);
                        nm.play_stream();
                        self.music = Some(nm);
                        self.music_loops = -1;   // Queue-Track loopt (raylib-Default)
                        self.music_fade = None;
                    }
                }
            }
        }
    }

    // ===================================================================
    // Erweitertes audio-Modul (AUDIO_*).
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
        // Paula-Lo-Fi (falls aktiv) vor Fade/Volume -- pro Kanal.
        let mut lo_l; let mut lo_r;
        let (left, right): (&[f64], &[f64]) = if self.lofi {
            lo_l = left.to_vec(); lo_r = right.to_vec();
            Self::lofi_chain(&mut lo_l, sr, self.lofi_bits, self.lofi_cutoff);
            Self::lofi_chain(&mut lo_r, sr, self.lofi_bits, self.lofi_cutoff);
            (&lo_l, &lo_r)
        } else { (left, right) };
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
        Ok(self.push_sound(s, vol as f32))
    }

    /// Float-Buffer [-1,1] -> Anti-Click-Envelope -> Volume -> i16-PCM -> WAV
    /// im RAM -> raylib Sound. Liefert das SOUND-Handle.
    fn push_wave_sound(&mut self, buf: &[f64], volume: f64, sr: u32) -> Result<i64, String> {
        let n = buf.len();
        let vol = volume.clamp(0.0, 1.0);
        // Paula-Lo-Fi (falls aktiv) vor Fade/Volume.
        let mut lofi_buf;
        let buf: &[f64] = if self.lofi {
            lofi_buf = buf.to_vec();
            Self::lofi_chain(&mut lofi_buf, sr, self.lofi_bits, self.lofi_cutoff);
            &lofi_buf
        } else { buf };
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
        Ok(self.push_sound(s, vol as f32))
    }

    // -- Channel-Playback (Handle == Sound-Index) --
    /// AUDIO_PLAY(sound[, loops[, volume[, fade_in_ms]]]) -- pygame-Semantik:
    /// loops=0 einmal (Default), loops=N -> N+1 Durchlaeufe, loops=-1 endlos.
    /// raylib-Sounds loopen nicht nativ -> update() zaehlt und startet neu.
    pub fn ch_play(&mut self, idx: i64, loops: i64, volume: f64, fade_in_ms: i64) -> Result<i64, String> {
        let v = volume.clamp(0.0, 1.0) as f32;
        {
            let s = self.snd(idx, "AUDIO_PLAY")?;
            s.set_volume(if fade_in_ms > 0 { 0.0 } else { v });
            s.play();
        }
        let i = idx as usize;
        self.reset_channel_state(i, v);
        self.ch_loops[i] = if loops < 0 { -1 } else { loops };
        if fade_in_ms > 0 {
            self.ch_fade[i] = Some(Fade {
                from: 0.0, to: v,
                start: std::time::Instant::now(),
                dur_ms: fade_in_ms as f64, stop_after: false,
            });
        }
        Ok(idx)
    }
    pub fn ch_pause(&mut self, idx: i64) -> Result<(), String> {
        self.snd(idx, "AUDIO_PAUSE")?.pause();
        self.ch_paused[idx as usize] = true;
        Ok(())
    }
    pub fn ch_resume(&mut self, idx: i64) -> Result<(), String> {
        self.snd(idx, "AUDIO_RESUME")?.resume();
        self.ch_paused[idx as usize] = false;
        Ok(())
    }
    /// AUDIO_STOP(ch[, fade_out_ms]) -- ohne fade sofort, sonst ausfaden und
    /// am Fade-Ende stoppen (update() treibt den Fade).
    pub fn ch_stop(&mut self, idx: i64, fade_out_ms: i64) -> Result<(), String> {
        let playing = {
            let s = self.snd(idx, "AUDIO_STOP")?;
            if fade_out_ms <= 0 { s.stop(); }
            s.is_playing()
        };
        let i = idx as usize;
        if fade_out_ms <= 0 {
            let v = self.sound_vol[i];
            self.reset_channel_state(i, v);
            return Ok(());
        }
        if playing || self.ch_paused[i] {
            let from = self.ch_fade[i].as_ref()
                .map(|f| f.current_vol()).unwrap_or(self.sound_vol[i]);
            self.ch_fade[i] = Some(Fade {
                from, to: 0.0,
                start: std::time::Instant::now(),
                dur_ms: fade_out_ms as f64, stop_after: true,
            });
        }
        Ok(())
    }
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
    /// AUDIO_PITCH(ch, faktor) -- Abspielgeschwindigkeit/Tonhoehe (1.0 =
    /// normal, 2.0 = Oktave hoeher, 0.5 = Oktave tiefer). Wirkt sofort,
    /// auch auf einen bereits spielenden Sound. Klassiker: pro Schuss
    /// leicht variieren (0.9 + RANDF() * 0.2), damit nichts leiert.
    pub fn ch_pitch(&self, idx: i64, factor: f64) -> Result<(), String> {
        self.snd(idx, "AUDIO_PITCH")?.set_pitch(factor as f32);
        Ok(())
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
        self.ch_pan_anim[idx as usize] = None;   // manuelles Pan beendet die Animation
        Ok(())
    }

    /// AUDIO_PAN_POS(ch, p) -- Stereo-Position direkt: 0=links, 0.5=Mitte,
    /// 1=rechts. Fasst nur das Pan an (Volume bleibt unveraendert) und
    /// beendet eine laufende Pan-Animation (manuell gewinnt).
    pub fn ch_pan_pos(&mut self, idx: i64, p: f64) -> Result<(), String> {
        self.snd(idx, "AUDIO_PAN_POS")?.set_pan((1.0 - p.clamp(0.0, 1.0)) as f32);
        self.ch_pan_anim[idx as usize] = None;
        Ok(())
    }

    /// AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms) -- einmalige Stereo-
    /// Wanderung (Positionen 0=links..1=rechts), bleibt am Ziel stehen.
    /// Laeuft nicht-blockierend; update() treibt die Bewegung pro Frame.
    pub fn ch_pan_slide(&mut self, idx: i64, from: f64, to: f64, dur_ms: i64) -> Result<(), String> {
        let (from, to) = (from.clamp(0.0, 1.0), to.clamp(0.0, 1.0));
        self.snd(idx, "AUDIO_PAN_SLIDE")?.set_pan((1.0 - from) as f32);
        self.ch_pan_anim[idx as usize] = Some(PanAnim::Slide {
            from, to, start: std::time::Instant::now(), dur_ms: dur_ms as f64,
        });
        Ok(())
    }

    /// AUDIO_AUTOPAN(ch, periode_s[, tiefe]) -- endloses Pendeln
    /// links<->rechts (startet links). periode_s = Dauer einer vollen Runde;
    /// tiefe 0..1 = Auslenkung um die Mitte (1.0 = ganz links bis ganz
    /// rechts). periode_s <= 0 schaltet die Animation ab (Pan bleibt stehen).
    pub fn ch_autopan(&mut self, idx: i64, period_s: f64, depth: f64) -> Result<(), String> {
        self.snd(idx, "AUDIO_AUTOPAN")?;
        self.ch_pan_anim[idx as usize] = if period_s <= 0.0 { None } else {
            Some(PanAnim::Pendulum {
                period_s, depth: depth.clamp(0.0, 1.0),
                start: std::time::Instant::now(),
            })
        };
        Ok(())
    }

    // -- Mixer-weit --
    pub fn pause_all(&mut self) {
        for s in &self.sounds { s.pause(); }
        for p in &mut self.ch_paused { *p = true; }
    }
    pub fn resume_all(&mut self) {
        for s in &self.sounds { s.resume(); }
        for p in &mut self.ch_paused { *p = false; }
    }
    pub fn stop_all(&mut self) {
        for s in &self.sounds { s.stop(); }
        for f in &mut self.ch_fade { *f = None; }
        for l in &mut self.ch_loops { *l = 0; }
        for p in &mut self.ch_paused { *p = false; }
    }
    pub fn set_num_channels(&mut self, n: i64) { self.num_channels = n.max(0); }
    pub fn get_num_channels(&self) -> i64 { self.num_channels }
    pub fn busy_channels(&self) -> i64 { self.sounds.iter().filter(|s| s.is_playing()).count() as i64 }

    // -- Music erweitert --
    pub fn music_load(&mut self, path: &str) -> Result<(), String> {
        if let Some(m) = self.music.take() { m.stop_stream(); }
        let m = self.dev.new_music(path).map_err(|e| format!("AUDIO_MUSIC_LOAD: {}", e))?;
        m.set_volume(self.music_vol);
        m.set_pitch(self.music_pitch);
        self.music = Some(m);
        self.music_queue = None;
        self.music_loops = -1;
        self.music_fade = None;
        self.music_paused = false;
        Ok(())
    }
    /// AUDIO_MUSIC_PLAY([loops[, fade_in_ms]]) -- pygame-Semantik:
    /// loops=-1 endlos (Default), loops=N -> N+1 Durchlaeufe insgesamt.
    /// Endlos macht raylib selbst (looping-Flag); endliche Wiederholungen
    /// zaehlt update() und startet den beendeten Stream neu.
    pub fn music_play(&mut self, loops: i64, fade_in_ms: i64) {
        self.music_fade = None;
        self.music_paused = false;
        if let Some(m) = &mut self.music {
            m.as_mut().looping = loops < 0;
            self.music_loops = if loops < 0 { -1 } else { loops };
            // Wie pygame: Play startet immer am Anfang (StopMusicStream
            // setzt die Position zurueck, PlayMusicStream allein nicht).
            m.stop_stream();
            if fade_in_ms > 0 {
                m.set_volume(0.0);
                self.music_fade = Some(Fade {
                    from: 0.0, to: self.music_vol,
                    start: std::time::Instant::now(),
                    dur_ms: fade_in_ms as f64, stop_after: false,
                });
            } else {
                m.set_volume(self.music_vol);
            }
            m.play_stream();
        }
    }
    /// AUDIO_MUSIC_STOP([fade_out_ms]) -- ohne Argument sofort, sonst
    /// ausfaden und am Fade-Ende stoppen (update() treibt den Fade).
    pub fn music_stop(&mut self, fade_out_ms: i64) {
        if fade_out_ms <= 0 {
            self.music_fade = None;
            self.music_paused = false;
            if let Some(m) = &self.music { m.stop_stream(); }
            return;
        }
        if let Some(m) = &self.music {
            if m.is_stream_playing() {
                let from = self.music_fade.as_ref()
                    .map(|f| f.current_vol()).unwrap_or(self.music_vol);
                self.music_fade = Some(Fade {
                    from, to: 0.0,
                    start: std::time::Instant::now(),
                    dur_ms: fade_out_ms as f64, stop_after: true,
                });
            }
        }
    }
    pub fn music_pause(&mut self) {
        if let Some(m) = &self.music { m.pause_stream(); self.music_paused = true; }
    }
    pub fn music_resume(&mut self) {
        if let Some(m) = &self.music { m.resume_stream(); self.music_paused = false; }
    }
    pub fn music_set_volume(&mut self, v: f64) {
        let vol = v.clamp(0.0, 1.0) as f32;
        self.music_vol = vol;
        // Explizites Set-Volume gewinnt: laufenden Fade abbrechen.
        self.music_fade = None;
        if let Some(m) = &self.music { m.set_volume(vol); }
    }
    pub fn music_get_volume(&self) -> f64 { self.music_vol as f64 }
    /// AUDIO_MUSIC_PITCH(faktor) -- Pitch des Musik-Streams (1.0 = normal).
    /// Wird getrackt und ueberlebt AUDIO_MUSIC_LOAD/QUEUE (wie music_vol);
    /// Slow-Motion-Effekt: Pitch zusammen mit der Spiel-Zeit absenken.
    pub fn music_set_pitch(&mut self, factor: f64) {
        self.music_pitch = factor as f32;
        if let Some(m) = &self.music { m.set_pitch(self.music_pitch); }
    }
    pub fn music_get_pitch(&self) -> f64 { self.music_pitch as f64 }
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

/// Resampling eines Mono-Samples mit linearer Interpolation (pur -- ohne
/// Audio-Geraet, daher unit-testbar). `ratio` = Schrittweite durch das
/// Quell-Sample pro Ausgabe-Sample: >1 = hoehere Tonhoehe (schneller,
/// kuerzer), <1 = tiefer. `dur_ms<=0` spielt das ganze Sample einmal;
/// `dur_ms>0` baut eine feste Laenge, die die Loop-Region wiederholt
/// (aktiv wenn loop_end>loop_start), sonst danach Stille.
fn resample(data: &[f32], sr: u32, ratio: f64, dur_ms: i64,
            loop_start: usize, loop_end: usize) -> Vec<f64> {
    let n = data.len();
    if n == 0 { return Vec::new(); }
    let lerp = |pos: f64| -> f64 {
        let i = pos.floor() as usize;
        if i + 1 >= n {
            return *data.get(i.min(n - 1)).unwrap_or(&0.0) as f64;
        }
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

#[cfg(test)]
mod tests {
    use super::{fade_progress, pendulum_pos, resample};

    #[test]
    fn resample_octave_up_halves_length() {
        // 100 Samples, +1 Oktave (ratio 2) -> halbe Laenge (One-Shot).
        let data: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
        let out = resample(&data, 44100, 2.0, 0, 0, 0);
        assert_eq!(out.len(), 50);
        // lineare Rampe bei Schrittweite 2 -> jedes 2. Element
        assert!((out[1] - 0.02).abs() < 1e-6);
        assert!((out[10] - 0.20).abs() < 1e-6);
    }

    #[test]
    fn resample_octave_down_doubles_length() {
        let data: Vec<f32> = (0..50).map(|i| i as f32).collect();
        let out = resample(&data, 44100, 0.5, 0, 0, 0);
        assert_eq!(out.len(), 100);
        // Schrittweite 0.5 -> Zwischenwerte interpoliert
        assert!((out[2] - 1.0).abs() < 1e-6);   // pos=1.0
        assert!((out[3] - 1.5).abs() < 1e-6);   // pos=1.5 interpoliert
    }

    #[test]
    fn resample_unity_is_identity() {
        let data: Vec<f32> = vec![0.1, -0.2, 0.3, -0.4];
        let out = resample(&data, 8000, 1.0, 0, 0, 0);
        assert_eq!(out.len(), 4);
        for (a, b) in out.iter().zip(data.iter()) {
            assert!((a - *b as f64).abs() < 1e-6);
        }
    }

    #[test]
    fn resample_duration_with_loop_repeats_region() {
        // 4 Frames, Loop ueber [1,3) -> bei Dauer wiederholt sich {1,2}.
        let data: Vec<f32> = vec![10.0, 20.0, 30.0, 40.0];
        // sr=1000, dur=10ms -> 10 Output-Frames; ratio=1
        let out = resample(&data, 1000, 1.0, 10, 1, 3);
        assert_eq!(out.len(), 10);
        // 0:10, 1:20, 2:30, dann Loop zurueck auf 1: 20,30,20,30,...
        assert!((out[0] - 10.0).abs() < 1e-6);
        assert!((out[1] - 20.0).abs() < 1e-6);
        assert!((out[2] - 30.0).abs() < 1e-6);
        assert!((out[3] - 20.0).abs() < 1e-6);   // gewrappt
        assert!((out[4] - 30.0).abs() < 1e-6);
        assert!((out[5] - 20.0).abs() < 1e-6);
    }

    #[test]
    fn resample_duration_without_loop_pads_silence() {
        let data: Vec<f32> = vec![0.5, 0.5];
        // 2 Frames, kein Loop, Dauer fuer 6 Frames -> Rest Stille
        let out = resample(&data, 1000, 1.0, 6, 0, 0);
        assert_eq!(out.len(), 6);
        assert!((out[0] - 0.5).abs() < 1e-6);
        assert!((out[1] - 0.5).abs() < 1e-6);
        assert_eq!(out[3], 0.0);
        assert_eq!(out[5], 0.0);
    }

    #[test]
    fn resample_empty_is_empty() {
        assert!(resample(&[], 44100, 1.0, 0, 0, 0).is_empty());
    }

    #[test]
    fn lofi_bitcrush_quantizes_to_levels() {
        use super::Audio;
        // 2 bit -> 4 Stufen (half=2): Werte rasten auf Vielfache von 0.5.
        let mut buf = vec![0.1, 0.3, 0.6, -0.9];
        Audio::lofi_chain(&mut buf, 44100, 2, 0.0);   // Filter aus -> nur Crush
        for v in &buf {
            let q = (v * 2.0).round() / 2.0;
            assert!((v - q).abs() < 1e-9, "nicht gerastert: {}", v);
        }
        assert!((buf[0] - 0.0).abs() < 1e-9);   // 0.1 -> 0.0
        assert!((buf[2] - 0.5).abs() < 1e-9);   // 0.6 -> 0.5
    }

    #[test]
    fn lofi_lowpass_attenuates_and_is_stable() {
        use super::Audio;
        // Wechselsignal (Nyquist-nah) -> Tiefpass daempft die Amplitude.
        let mut buf: Vec<f64> = (0..200).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        Audio::lofi_chain(&mut buf, 44100, 24, 2000.0);  // nur Filter (bits>=24 = kein Crush)
        let peak = buf.iter().cloned().fold(0.0f64, |m, v| m.max(v.abs()));
        assert!(peak < 0.5, "Tiefpass sollte die hohe Frequenz daempfen, peak={}", peak);
        assert!(buf.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn fade_progress_clamps_and_interpolates() {
        assert_eq!(fade_progress(0.0, 1000.0), 0.0);
        assert!((fade_progress(500.0, 1000.0) - 0.5).abs() < 1e-6);
        assert_eq!(fade_progress(1500.0, 1000.0), 1.0);   // ueber Ende -> geclampt
        assert_eq!(fade_progress(10.0, 0.0), 1.0);        // dur=0 -> sofort fertig
    }

    #[test]
    fn pendulum_starts_left_and_swings() {
        assert!(pendulum_pos(0.0, 4.0, 1.0).abs() < 1e-9);          // Start: ganz links
        assert!((pendulum_pos(2.0, 4.0, 1.0) - 1.0).abs() < 1e-9);  // halbe Runde: rechts
        assert!(pendulum_pos(4.0, 4.0, 1.0).abs() < 1e-9);          // volle Runde: links
        assert!((pendulum_pos(1.0, 4.0, 1.0) - 0.5).abs() < 1e-9);  // viertel: Mitte
        // tiefe=0.5 -> pendelt nur 0.25..0.75; tiefe wird geclampt
        assert!((pendulum_pos(0.0, 4.0, 0.5) - 0.25).abs() < 1e-9);
        assert!(pendulum_pos(0.0, 4.0, 7.0).abs() < 1e-9);
    }
}
