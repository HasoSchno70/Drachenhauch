//! Das Kira-Backend der Desktop-Laufzeit: das Geraet (cpal) -- oder stumm.
//!
//! `DHRT_OHNE_AUDIO=1` (alter Name `DH_OHNE_AUDIO`, aus der Python-Zeit)
//! schaltet den Ton stumm, ohne ihn abzuschalten: der Mix wird in Echtzeit
//! gerechnet und verworfen. Positionen, Uhren, Pegel, FFT und `AUDIO_SAVE_WAV`
//! laufen also wie mit Soundkarte -- nur hoert man nichts. Bis 2026.53 las
//! keine Zeile Rust die Variable (sie stammte aus conftest.py); die CI setzte
//! sie, und der Ton lief trotzdem an.
//!
//! Kiras eigenes `MockBackend` taugt dafuer nicht: es rechnet nur, wenn man es
//! von Hand antreibt -- Klaenge stuenden dann ewig am Anfang.

#![cfg(all(feature = "graphics", not(target_os = "emscripten")))]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use kira::backend::cpal::{CpalBackend, CpalBackendSettings};
use kira::backend::{Backend, Renderer};

/// Abtastrate des stummen Backends.
const STUMM_SR: u32 = 48_000;

/// Ist der Ton per Umgebung stumm geschaltet?
pub fn stumm_gewuenscht() -> bool {
    ["DHRT_OHNE_AUDIO", "DH_OHNE_AUDIO"].iter()
        .any(|n| std::env::var(n).map(|v| !v.trim().is_empty() && v.trim() != "0").unwrap_or(false))
}

pub enum WahlBackend {
    Geraet(CpalBackend),
    Stumm { puffer: usize, halt: Arc<AtomicBool> },
}

impl Backend for WahlBackend {
    /// TRUE: stumm.
    type Settings = bool;
    type Error = String;

    fn setup(stumm: bool, internal_buffer_size: usize) -> Result<(Self, u32), String> {
        if stumm {
            return Ok((WahlBackend::Stumm { puffer: internal_buffer_size.max(1), halt: Arc::new(AtomicBool::new(false)) }, STUMM_SR));
        }
        let (b, sr) = CpalBackend::setup(CpalBackendSettings::default(), internal_buffer_size)
            .map_err(|e| format!("{e:?}"))?;
        Ok((WahlBackend::Geraet(b), sr))
    }

    fn start(&mut self, mut renderer: Renderer) -> Result<(), String> {
        match self {
            WahlBackend::Geraet(b) => b.start(renderer).map_err(|e| format!("{e:?}")),
            WahlBackend::Stumm { puffer, halt } => {
                let (n, halt) = (*puffer, halt.clone());
                std::thread::Builder::new().name("dhrt-ton-stumm".into()).spawn(move || {
                    let mut rahmen = vec![0.0f32; n * 2];
                    let start = Instant::now();
                    let mut fertig: u64 = 0;
                    while !halt.load(Ordering::Relaxed) {
                        // So viele Bilder, wie seit dem Start faellig sind.
                        let soll = (start.elapsed().as_secs_f64() * STUMM_SR as f64) as u64;
                        while fertig + n as u64 <= soll {
                            renderer.on_start_processing();
                            renderer.process(&mut rahmen, 2);
                            fertig += n as u64;
                        }
                        std::thread::sleep(Duration::from_millis(5));
                    }
                }).map_err(|e| format!("stummer Ton: {e}"))?;
                Ok(())
            }
        }
    }
}

impl Drop for WahlBackend {
    fn drop(&mut self) {
        if let WahlBackend::Stumm { halt, .. } = self { halt.store(true, Ordering::Relaxed); }
    }
}
