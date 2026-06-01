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

use raylib::core::audio::{Music, RaylibAudio, Sound};

pub struct Audio {
    dev: &'static RaylibAudio,
    sounds: Vec<Sound<'static>>,
    music: Option<Music<'static>>,
}

impl Audio {
    pub fn new() -> Result<Audio, String> {
        let dev = RaylibAudio::init_audio_device()
            .map_err(|_| "Audio-Geraet konnte nicht initialisiert werden".to_string())?;
        let dev: &'static RaylibAudio = Box::leak(Box::new(dev));
        Ok(Audio { dev, sounds: Vec::new(), music: None })
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
