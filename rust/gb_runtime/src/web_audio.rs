//! Hoerbarer Ton im Browser: eigenes Kira-Backend, Ausgabe ueber OpenAL.
//!
//! Warum nicht cpal wie auf dem Desktop: cpal hat keinen emscripten-Host mehr,
//! und sein WebAudio-Host haengt an wasm-bindgen-JS-Glue, das emscripten nicht
//! liefert. Emscripten bringt dafuer eine eigene OpenAL-Umsetzung mit, die
//! intern auf WebAudio zeichnet -- und die ist ohnehin schon gelinkt (`-lal`).
//!
//! Der Aufbau ist denkbar schlicht: Kiras `Backend`-Vertrag besteht aus zwei
//! Methoden, und die zweite reicht uns den `Renderer` durch. Damit haben wir
//! den fertigen Mix in der Hand und schieben ihn selbst in eine Warteschlange
//! von OpenAL-Puffern.
//!
//! **Die Warteschlange taktet sich selbst.** Wir fuellen sie bei jedem Bild
//! nach, aber nur so weit, wie WebAudio sie leergespielt hat -- im Beharrungs-
//! zustand entsteht damit exakt Echtzeit, ohne dass wir irgendwo die Uhr
//! ablesen muessten. Das ist genauer als das frueher benutzte Nachrechnen der
//! verstrichenen Zeit und macht `AUDIO_MUSIC_POSITION` im Browser belastbar.
//!
//! **Browser-Eigenheit:** eine `AudioContext` startet angehalten, bis der
//! Nutzer die Seite einmal angefasst hat (Autoplay-Sperre). Bis dahin verbraucht
//! WebAudio nichts, die Warteschlange bleibt voll, und wir rechnen nichts nach
//! -- die Wiedergabe steht also und laeuft beim ersten Klick los. Emscriptens
//! OpenAL haengt sich fuer das Fortsetzen selbst an die Ereignisse der Seite.

use kira::backend::{Backend, Renderer};
use std::ffi::c_void;

// --- OpenAL, so viel wie wir brauchen ---------------------------------------
// Eigene Deklarationen statt einer Crate: es sind neun Funktionen, und jede
// zusaetzliche Abhaengigkeit muesste fuer emscripten uebersetzen.
#[allow(non_camel_case_types)]
type ALCdevice = c_void;
#[allow(non_camel_case_types)]
type ALCcontext = c_void;

unsafe extern "C" {
    fn alcOpenDevice(name: *const i8) -> *mut ALCdevice;
    fn alcCreateContext(dev: *mut ALCdevice, attr: *const i32) -> *mut ALCcontext;
    fn alcMakeContextCurrent(ctx: *mut ALCcontext) -> i32;
    fn alcGetIntegerv(dev: *mut ALCdevice, param: i32, size: i32, values: *mut i32);
    fn alGenSources(n: i32, sources: *mut u32);
    fn alGenBuffers(n: i32, buffers: *mut u32);
    fn alBufferData(buffer: u32, format: i32, data: *const c_void, size: i32, freq: i32);
    fn alSourceQueueBuffers(source: u32, n: i32, buffers: *const u32);
    fn alSourceUnqueueBuffers(source: u32, n: i32, buffers: *mut u32);
    fn alGetSourcei(source: u32, param: i32, value: *mut i32);
    fn alSourcePlay(source: u32);
}

const AL_FORMAT_STEREO16: i32 = 0x1103;
const AL_BUFFERS_PROCESSED: i32 = 0x1016;
const AL_SOURCE_STATE: i32 = 0x1010;
const AL_PLAYING: i32 = 0x1012;
const ALC_FREQUENCY: i32 = 0x1007;

/// Bilder je Puffer (~46 ms bei 44,1 kHz).
///
/// Jeder Puffer wird als eigener WebAudio-Knoten eingeplant, jede Naht ist
/// also eine moegliche Stoerstelle -- weniger, groessere Puffer sind darum
/// ruhiger. Nach oben begrenzt die Verzoegerung: der Vorrat in der
/// Warteschlange IST die Zeit, die ein neu gestarteter Klang braucht, bis man
/// ihn hoert. 4 x 46 ms ~ 190 ms ist der Kompromiss.
const PUFFER_BILDER: usize = 2048;
/// Wie viele Puffer die Warteschlange fasst.
const PUFFER_ANZAHL: usize = 4;

pub struct WebBackend {
    sample_rate: u32,
    renderer: Option<Renderer>,
    quelle: u32,
    /// Puffer, die gerade NICHT in der Warteschlange stehen.
    frei: Vec<u32>,
    mix: Vec<f32>,
    pcm: Vec<i16>,
    laeuft: bool,
    leerlaeufe: u32,
}

impl Backend for WebBackend {
    /// Die Abtastrate -- alles andere kommt aus OpenAL.
    type Settings = u32;
    type Error = String;

    fn setup(_wunsch: u32, _internal_buffer_size: usize) -> Result<(Self, u32), String> {
        let (quelle, frei, sample_rate) = unsafe {
            let dev = alcOpenDevice(std::ptr::null());
            if dev.is_null() { return Err("OpenAL: kein Ausgabegeraet".into()); }
            let ctx = alcCreateContext(dev, std::ptr::null());
            if ctx.is_null() { return Err("OpenAL: kein Kontext".into()); }
            alcMakeContextCurrent(ctx);
            // Mit DER Abtastrate rechnen, die der Browser tatsaechlich fahrt --
            // nicht mit einer gewuenschten. Weichen die beiden ab, rechnet
            // WebAudio JEDEN Puffer einzeln um, und weil dabei die Nachbarn
            // fehlen, entsteht an jeder Naht ein Sprung: es stottert im
            // Puffertakt. (Uebliche Faelle: 48000 statt 44100.)
            let mut rate: i32 = 0;
            alcGetIntegerv(dev, ALC_FREQUENCY, 1, &mut rate);
            let sample_rate = if rate > 0 { rate as u32 } else { 44100 };
            let mut quelle: u32 = 0;
            alGenSources(1, &mut quelle);
            let mut puffer = [0u32; PUFFER_ANZAHL];
            alGenBuffers(PUFFER_ANZAHL as i32, puffer.as_mut_ptr());
            (quelle, puffer.to_vec(), sample_rate)
        };
        Ok((WebBackend {
            sample_rate,
            renderer: None,
            quelle,
            frei,
            mix: vec![0.0; PUFFER_BILDER * 2],
            pcm: vec![0; PUFFER_BILDER * 2],
            laeuft: false,
            leerlaeufe: 0,
        }, sample_rate))
    }

    fn start(&mut self, renderer: Renderer) -> Result<(), String> {
        self.renderer = Some(renderer);
        Ok(())
    }
}

impl WebBackend {
    /// Einmal pro Bild aufrufen: verbrauchte Puffer einsammeln und die
    /// Warteschlange wieder auffuellen.
    pub fn nachfuellen(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else { return };

        // Was WebAudio abgespielt hat, kommt zurueck in den freien Vorrat.
        unsafe {
            let mut fertig: i32 = 0;
            alGetSourcei(self.quelle, AL_BUFFERS_PROCESSED, &mut fertig);
            for _ in 0..fertig {
                let mut b: u32 = 0;
                alSourceUnqueueBuffers(self.quelle, 1, &mut b);
                if b != 0 { self.frei.push(b); }
            }
        }
        if self.frei.is_empty() { return; }

        // Befehle aus dem Hauptthread uebernehmen (neue Sounds, Tweens) und
        // so viel Ton rechnen, wie Platz ist.
        renderer.on_start_processing();
        while let Some(b) = self.frei.pop() {
            for v in self.mix.iter_mut() { *v = 0.0; }
            renderer.process(&mut self.mix, 2);
            for (ziel, &quelle) in self.pcm.iter_mut().zip(self.mix.iter()) {
                *ziel = (quelle.clamp(-1.0, 1.0) * 32767.0) as i16;
            }
            unsafe {
                alBufferData(b, AL_FORMAT_STEREO16,
                             self.pcm.as_ptr() as *const c_void,
                             (self.pcm.len() * 2) as i32,
                             self.sample_rate as i32);
                alSourceQueueBuffers(self.quelle, 1, &b);
            }
        }

        // Die Quelle laeuft nach einem Leerlauf nicht von selbst weiter --
        // nach jedem Nachfuellen pruefen und notfalls neu anstossen.
        unsafe {
            let mut zustand: i32 = 0;
            alGetSourcei(self.quelle, AL_SOURCE_STATE, &mut zustand);
            if zustand != AL_PLAYING {
                // Beim ERSTEN Mal ist das der normale Start. Jedes weitere Mal
                // heisst: die Warteschlange war leer, es hat gestockt. Das
                // sagen wir laut -- ohne diese Zeile ist Stottern nur
                // hoerbar, aber nicht messbar, und man raet an den Puffer-
                // groessen herum.
                if self.laeuft {
                    self.leerlaeufe += 1;
                    if self.leerlaeufe <= 5 {
                        eprintln!("Ton-Warteschlange lief leer ({}. Mal) -- \
                                   das Bild braucht laenger als der Vorrat reicht",
                                  self.leerlaeufe);
                    }
                }
                alSourcePlay(self.quelle);
                self.laeuft = true;
            }
        }
    }

    /// Laeuft die Wiedergabe? (Fuer Fehlersuche und Tests.)
    pub fn laeuft(&self) -> bool { self.laeuft }
    /// Wie oft die Warteschlange leerlief (0 = sauber durchgelaufen).
    pub fn leerlaeufe(&self) -> u32 { self.leerlaeufe }
}
