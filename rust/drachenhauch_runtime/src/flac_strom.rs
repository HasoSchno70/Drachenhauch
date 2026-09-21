//! FLAC als gestreamte Musik -- ein eigener Dekoder fuer Kira.
//!
//! Kiras eingebauter Weg (`StreamingSoundData::from_file`) geht fuer FLAC
//! durch Symphonia 0.6, und dessen FLAC-Leser springt nach dem ersten
//! Durchlauf nicht sauber an den Anfang zurueck: `resync` setzt den
//! Paketbauer nur zurueck, wenn sich der Leser dabei BEWEGT hat. Landet der
//! Sprung genau auf dem ersten Frame (bei Dateien unter 16 KB immer, bei
//! einer SEEKTABLE oft), bleibt der Zustand vom Dateiende stehen, und das
//! naechste Lesen endet mit `UnexpectedEof`. Kira stoppt den Klang dann
//! still -- eine FLAC-Musik mit Schleife (die Vorgabe von AUDIO_MUSIC_PLAY)
//! spielte keinen einzigen Ton.
//!
//! Der Ausweg hier: vor jedem Sprung wird die Datei NEU geoeffnet. Ein
//! frischer Leser hat einen leeren Paketbauer, und genau dann springt
//! Symphonia richtig. Das kostet einen Dateizugriff je Schleifendurchlauf
//! oder AUDIO_MUSIC_SEEK -- nichts, was man hoert.

use kira::Frame;
use kira::sound::FromFileError;
use kira::sound::streaming::Decoder;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::audio::AudioDecoder;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::units::Timestamp;

/// Traegt die Datei die FLAC-Kennung? Nach dem Inhalt, nicht nach der Endung --
/// so entscheidet auch Symphonia.
pub fn ist_flac(pfad: &str) -> bool {
    use std::io::Read;
    let mut kopf = [0u8; 4];
    std::fs::File::open(pfad)
        .and_then(|mut f| f.read_exact(&mut kopf))
        .map(|_| &kopf == b"fLaC")
        .unwrap_or(false)
}

pub struct FlacStrom {
    pfad: String,
    leser: Box<dyn FormatReader>,
    dekoder: Box<dyn AudioDecoder>,
    spur: u32,
    rate: u32,
    frames: usize,
    kanaele: usize,
}

fn oeffnen(pfad: &str) -> Result<(Box<dyn FormatReader>, Box<dyn AudioDecoder>, u32, u32, usize, usize), FromFileError> {
    let datei = std::fs::File::open(pfad)?;
    let mss = MediaSourceStream::new(Box::new(datei), Default::default());
    let leser = symphonia::default::get_probe().probe(
        &Hint::default(), mss, Default::default(), Default::default())?;
    let spur = leser.default_track(TrackType::Audio).ok_or(FromFileError::NoDefaultTrack)?;
    let p = match spur.codec_params.as_ref() {
        Some(CodecParameters::Audio(p)) => p,
        _ => return Err(FromFileError::NoDefaultTrack),
    };
    let rate = p.sample_rate.ok_or(FromFileError::UnknownSampleRate)?;
    let kanaele = p.channels.as_ref().map(|c| c.count()).unwrap_or(0);
    if kanaele == 0 || kanaele > 2 {
        return Err(FromFileError::UnsupportedChannelConfiguration);
    }
    let frames = spur.num_frames.ok_or(FromFileError::UnknownDuration)? as usize;
    let dekoder = symphonia::default::get_codecs().make_audio_decoder(p, &Default::default())?;
    let id = spur.id;
    Ok((leser, dekoder, id, rate, frames, kanaele))
}

impl FlacStrom {
    pub fn neu(pfad: &str) -> Result<Self, FromFileError> {
        let (leser, dekoder, spur, rate, frames, kanaele) = oeffnen(pfad)?;
        Ok(Self { pfad: pfad.to_string(), leser, dekoder, spur, rate, frames, kanaele })
    }
}

impl Decoder for FlacStrom {
    type Error = FromFileError;

    fn sample_rate(&self) -> u32 { self.rate }

    fn num_frames(&self) -> usize { self.frames }

    fn decode(&mut self) -> Result<Vec<Frame>, FromFileError> {
        let paket = loop {
            match self.leser.next_packet()? {
                Some(p) if p.track_id == self.spur => break p,
                Some(_) => continue,
                None => return Ok(Vec::new()),
            }
        };
        let puffer = self.dekoder.decode(&paket)?;
        let mut werte: Vec<f32> = Vec::new();
        puffer.copy_to_vec_interleaved(&mut werte);
        Ok(if self.kanaele == 1 {
            werte.into_iter().map(Frame::from_mono).collect()
        } else {
            werte.chunks_exact(2).map(|p| Frame::new(p[0], p[1])).collect()
        })
    }

    fn seek(&mut self, index: usize) -> Result<usize, FromFileError> {
        // Frisch oeffnen -- siehe Modulkommentar.
        let (leser, dekoder, spur, _, _, _) = oeffnen(&self.pfad)?;
        self.leser = leser;
        self.dekoder = dekoder;
        self.spur = spur;
        let erreicht = self.leser.seek(SeekMode::Accurate, SeekTo::Timestamp {
            ts: Timestamp::new(index as i64), track_id: self.spur,
        })?;
        Ok((erreicht.actual_ts.get().max(0) as usize).min(self.frames))
    }
}
