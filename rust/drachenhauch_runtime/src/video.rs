//! Video abspielen (Modul `video`, Feature `video`): MP4 mit H.264.
//!
//! Der Container wird mit der Crate `mp4` gelesen (reines Rust), die Bilder
//! dekodiert `openh264` -- Ciscos freier Decoder, der beim Bau aus dem
//! Quelltext mituebersetzt wird. Es muss also beim Nutzer nichts installiert
//! sein, und es geht auf allen drei Systemen gleich.
//!
//! Dieser Teil kennt kein Fenster: er liefert Bilder als RGBA-Punkte, das
//! Zeichnen macht die VM ueber ein gewoehnliches IMAGE. So laesst er sich ohne
//! Grafik pruefen.
//!
//! **Zwei Dinge, die nicht offensichtlich sind:**
//! 1. Der Decoder gibt Bilder VERZOEGERT heraus (das erste nach dem zweiten
//!    Paket), und am Ende stecken noch welche in ihm. Gezaehlt wird darum,
//!    was HERAUSKOMMT, nicht was hineingeht -- und am Ende wird geleert.
//! 2. `openh264` versucht per Vorgabe nach jedem Paket ein Bild
//!    herauszuziehen ("flush"). Bei einem High-Profile-Video mit acht Slices
//!    je Bild endete das nach neun Bildern mit "out of memory", danach waren
//!    die Parameter weg und kein Bild ging mehr. Ohne das Nachschieben laufen
//!    alle 240 durch.

use std::fs::File;
use std::io::BufReader;

use openh264::decoder::{Decoder, DecoderConfig, Flush};
use openh264::formats::YUVSource;
use openh264::OpenH264API;

pub struct Video {
    leser: mp4::Mp4Reader<BufReader<File>>,
    spur: u32,
    /// Zahl der Pakete (= Bilder) in der Spur.
    pub bilder: u32,
    pub breite: u32,
    pub hoehe: u32,
    /// Bilder je Sekunde (aus Zahl und Dauer der Spur).
    pub fps: f64,
    /// Dauer in Sekunden.
    pub dauer: f64,
    /// SPS und PPS im Annex-B-Format -- beim Zurueckspulen neu hinein.
    kopf: Vec<u8>,
    decoder: Decoder,
    /// Naechstes Paket (ab 1, wie die Crate zaehlt).
    naechstes: u32,
    /// Wie viele Bilder schon herauskamen.
    heraus: u32,
    /// Am Ende geleert: was dabei herauskam, wartet hier.
    rest: Vec<Vec<u8>>,
    geleert: bool,
    /// Das zuletzt hergestellte Bild und seine Nummer (ab 0, -1 = keins).
    pub rgba: Vec<u8>,
    pub bild_nr: i64,
}

fn decoder_neu() -> Result<Decoder, String> {
    // Ohne Nachschieben -- siehe Kopf der Datei.
    Decoder::with_api_config(OpenH264API::from_source(),
        DecoderConfig::new().flush_after_decode(Flush::NoFlush))
        .map_err(|e| format!("Video: Decoder laesst sich nicht anlegen ({})", e))
}

/// MP4 speichert die NAL-Einheiten mit vorangestellter Laenge, der Decoder
/// will sie mit Startcode (Annex B).
fn annex_b(daten: &[u8], laenge: usize, ziel: &mut Vec<u8>) {
    let mut o = 0;
    while o + laenge <= daten.len() {
        let mut l = 0usize;
        for k in 0..laenge { l = (l << 8) | daten[o + k] as usize; }
        o += laenge;
        if o + l > daten.len() { break; }
        ziel.extend_from_slice(&[0, 0, 0, 1]);
        ziel.extend_from_slice(&daten[o..o + l]);
        o += l;
    }
}

impl Video {
    pub fn oeffnen(pfad: &str) -> Result<Video, String> {
        let f = File::open(pfad).map_err(|e| format!("VIDEO_LOAD: '{}' laesst sich nicht oeffnen ({})", pfad, e))?;
        let groesse = f.metadata().map(|m| m.len()).unwrap_or(0);
        let leser = mp4::Mp4Reader::read_header(BufReader::new(f), groesse)
            .map_err(|e| format!("VIDEO_LOAD: '{}' ist kein lesbares MP4 ({})", pfad, e))?;
        // Die erste Bildspur.
        let (spur, t) = leser.tracks().iter()
            .find(|(_, t)| t.track_type().ok() == Some(mp4::TrackType::Video))
            .map(|(i, t)| (*i, t))
            .ok_or_else(|| format!("VIDEO_LOAD: '{}' hat keine Bildspur", pfad))?;
        match t.media_type() {
            Ok(mp4::MediaType::H264) => {}
            Ok(m) => return Err(format!("VIDEO_LOAD: '{}' ist {} -- abspielen kann Drachenhauch nur H.264 (AVC)", pfad, m)),
            Err(e) => return Err(format!("VIDEO_LOAD: '{}': unbekanntes Bildformat ({})", pfad, e)),
        }
        let avc = t.trak.mdia.minf.stbl.stsd.avc1.as_ref()
            .ok_or_else(|| format!("VIDEO_LOAD: '{}': H.264 ohne avcC-Kopf", pfad))?;
        let mut kopf = Vec::new();
        for p in avc.avcc.sequence_parameter_sets.iter().chain(avc.avcc.picture_parameter_sets.iter()) {
            kopf.extend_from_slice(&[0, 0, 0, 1]);
            kopf.extend_from_slice(&p.bytes);
        }
        let bilder = t.sample_count();
        let dauer = t.duration().as_secs_f64();
        if bilder == 0 { return Err(format!("VIDEO_LOAD: '{}' hat keine Bilder", pfad)); }
        let fps = if dauer > 0.0 { bilder as f64 / dauer } else { 25.0 };
        let (breite, hoehe) = (t.width() as u32, t.height() as u32);
        let mut v = Video {
            leser, spur, bilder, breite, hoehe, fps, dauer, kopf,
            decoder: decoder_neu()?, naechstes: 1, heraus: 0, rest: Vec::new(), geleert: false,
            rgba: vec![0; (breite as usize) * (hoehe as usize) * 4], bild_nr: -1,
        };
        v.anfang()?;
        // Das erste Bild gleich -- dann hat das IMAGE sofort einen Inhalt.
        v.bild(0)?;
        Ok(v)
    }

    /// Zurueck an den Anfang: ein frischer Decoder mit den Parametern.
    fn anfang(&mut self) -> Result<(), String> {
        self.decoder = decoder_neu()?;
        let kopf = self.kopf.clone();
        let _ = self.decoder.decode(&kopf);
        self.naechstes = 1;
        self.heraus = 0;
        self.rest.clear();
        self.geleert = false;
        Ok(())
    }

    /// Bild `k` (ab 0) herstellen. Liefert, ob es das gibt; das Bild steht
    /// danach in `rgba`. Vorwaerts wird weiterdekodiert, rueckwaerts vom
    /// Anfang an -- H.264 kennt nur Schluesselbilder am Anfang einer Gruppe,
    /// und dieses Modul spult selten zurueck (Schleife, Anhalten).
    pub fn bild(&mut self, k: u32) -> Result<bool, String> {
        if k >= self.bilder { return Ok(false); }
        if self.bild_nr == k as i64 { return Ok(true); }
        if (k as i64) < self.bild_nr || (self.bild_nr < 0 && self.heraus > 0) { self.anfang()?; }
        let (b, h) = (self.breite as usize, self.hoehe as usize);
        while self.heraus <= k {
            if !self.rest.is_empty() {
                let bild = self.rest.remove(0);
                if self.heraus == k { self.rgba = bild; self.bild_nr = k as i64; }
                self.heraus += 1;
                continue;
            }
            if self.naechstes <= self.bilder {
                let probe = self.leser.read_sample(self.spur, self.naechstes)
                    .map_err(|e| format!("Video: Bild {} laesst sich nicht lesen ({})", self.naechstes, e))?;
                self.naechstes += 1;
                let Some(probe) = probe else { continue };
                let mut paket = Vec::with_capacity(probe.bytes.len() + 16);
                annex_b(&probe.bytes, 4, &mut paket);
                match self.decoder.decode(&paket) {
                    Ok(Some(yuv)) => {
                        if self.heraus == k {
                            let (w, hh) = yuv.dimensions();
                            if w == b && hh == h { yuv.write_rgba8(&mut self.rgba); }
                            self.bild_nr = k as i64;
                        }
                        self.heraus += 1;
                    }
                    Ok(None) => {}
                    // Ein kaputtes Paket laesst ein Bild aus, statt das Video
                    // anzuhalten -- ein Abspieler zeigt dann das vorige weiter.
                    Err(_) => {}
                }
                continue;
            }
            if self.geleert { break; }
            // Alles hinein: was noch im Decoder steckt, herausholen.
            self.geleert = true;
            let bilder = self.decoder.flush_remaining()
                .map_err(|e| format!("Video: Ende laesst sich nicht lesen ({})", e))?;
            for yuv in bilder {
                let mut px = vec![0u8; b * h * 4];
                let (w, hh) = yuv.dimensions();
                if w == b && hh == h { yuv.write_rgba8(&mut px); }
                self.rest.push(px);
            }
            if self.rest.is_empty() { break; }
        }
        Ok(self.bild_nr == k as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::annex_b;

    #[test]
    fn laengenpraefix_wird_startcode() {
        let d = [0, 0, 0, 2, 0x65, 0x88, 0, 0, 0, 1, 0x41];
        let mut z = Vec::new();
        annex_b(&d, 4, &mut z);
        assert_eq!(z, vec![0, 0, 0, 1, 0x65, 0x88, 0, 0, 0, 1, 0x41]);
    }

    #[test]
    fn abgeschnittenes_paket_bricht_ab_statt_zu_lesen() {
        let d = [0, 0, 0, 9, 0x65];
        let mut z = Vec::new();
        annex_b(&d, 4, &mut z);
        assert!(z.is_empty());
    }
}
