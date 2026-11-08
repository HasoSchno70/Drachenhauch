//! Befehle fuer Geraete: midi, video, serial, modbus, hotplug, websocket, osc, mdns, webcam, Mikrofon, Pins (gpio/i2c/spi), firmata, usb, wifi, bt.
//!
//! Teil von `vm.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl<'p> Vm<'p> {
    // ===================================================================
    // Modul serial (Feature `serial`)
    // ===================================================================
    /// Modul `midi` (midi.rs) -- Noten lesen und schicken.
    ///
    /// Die beiden Umrechner (`MIDI_NOTE_NAME$`, `MIDI_NOTE_FREQ`) stehen VOR
    /// dem Feature-Gate: sie brauchen kein Geraet, und wer eine Notenanzeige
    /// oder einen Ton aus einer Notennummer baut, soll sie in jedem Bau haben.
    pub(super) fn try_midi(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("midi_") { return Ok(None); }
        match name {
            "midi_note_name$" | "midi_note_name" => {
                let n = bi_int(a, 0, "MIDI_NOTE_NAME$")?;
                return Ok(Some(Value::str_rc(crate::midi::note_name(n))));
            }
            "midi_note_freq" => {
                let n = bi_int(a, 0, "MIDI_NOTE_FREQ")?;
                return Ok(Some(Value::Float(crate::midi::note_freq(n))));
            }
            _ => {}
        }
        if name.starts_with("midi_file_") && !matches!(name, "midi_file_play" | "midi_file_stop" | "midi_file_playing" | "midi_file_position") {
            return self.try_midi_datei(name, a);
        }
        #[cfg(feature = "midi")]
        { return self.try_midi_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    // ===================================================================
    // Modul video (Feature `video` + Grafik): MP4/H.264 in ein IMAGE
    // ===================================================================
    pub(super) fn try_video(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("video_") { return Ok(None); }
        #[cfg(all(feature = "video", feature = "graphics"))]
        { return self.try_video_impl(name, a); }
        // Ohne das Feature: unbekannter Befehl -- "im Rust-Kern noch nicht
        // verfuegbar", wie jeder andere, den dieser Bau nicht hat.
        #[allow(unreachable_code)]
        { let _ = a; Ok(None) }
    }

    #[cfg(all(feature = "video", feature = "graphics"))]
    pub(super) fn video_z(&mut self, a: &[Value], fn_: &str) -> R<usize> {
        let i = bi_int(a, 0, fn_)?;
        match self.videos.get(i as usize) {
            Some(Some(_)) if i >= 0 => Ok(i as usize),
            _ => Err(format!("{}: ungueltiges oder freigegebenes VIDEO {}", fn_, i)),
        }
    }

    /// Das Bild zur Uhrzeit des Videos herstellen und ins IMAGE schreiben,
    /// wenn es ein neues ist.
    #[cfg(all(feature = "video", feature = "graphics"))]
    pub(super) fn video_bild_zeigen(&mut self, i: usize) -> R<()> {
        let z = self.videos[i].as_mut().unwrap();
        // Ein Hauch Zugabe: 120 mal 1/60 s aufsummiert ist knapp UNTER 2,0,
        // und ohne sie stuende nach genau zwei Sekunden noch das Bild davor.
        let k = ((z.zeit * z.video.fps + 1e-6) as i64).clamp(0, z.video.bilder as i64 - 1) as u32;
        z.video.bild(k)?;
        if z.video.bild_nr != z.geschrieben {
            z.geschrieben = z.video.bild_nr;
            let bild = z.bild;
            let px = std::mem::take(&mut z.video.rgba);
            let r = match self.gfx.as_mut() { Some(g) => g.image_punkte_setzen(bild, &px), None => Ok(()) };
            self.videos[i].as_mut().unwrap().video.rgba = px;
            r?;
        }
        Ok(())
    }

    /// Die Uhr weiterstellen -- hoechstens einmal je gezeigtem Bild, auch wenn
    /// das Programm VIDEO_UPDATE und VIDEO_DRAW beide ruft. Die Zeit kommt
    /// aus DELTA(): ohne Fenster (Tests) ist das fest 1/60 s, das Video
    /// laeuft dort also Bild fuer Bild gleich ab.
    #[cfg(all(feature = "video", feature = "graphics"))]
    pub(super) fn video_weiter(&mut self, i: usize) -> R<()> {
        let (zaehler, dt) = match self.gfx.as_ref() { Some(g) => (g.bild_zaehler(), g.delta()), None => (0, 0.0) };
        let z = self.videos[i].as_mut().unwrap();
        if z.stand == Some(zaehler) { return Ok(()); }
        z.stand = Some(zaehler);
        if z.spielt {
            z.zeit += dt;
            if z.zeit >= z.video.dauer {
                if z.schleife && z.video.dauer > 0.0 {
                    z.zeit %= z.video.dauer;
                } else {
                    // Am Ende steht das letzte Bild -- ein Vorspann, der
                    // danach schwarz wuerde, blitzte beim Uebergang.
                    z.zeit = z.video.dauer;
                    z.spielt = false;
                    z.fertig = true;
                }
            }
        }
        self.video_bild_zeigen(i)
    }

    #[cfg(all(feature = "video", feature = "graphics"))]
    pub(super) fn try_video_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        let v = match name {
            "video_load" => {
                let pfad = bi_str(a, 0, "VIDEO_LOAD")?.to_string();
                if !std::path::Path::new(&pfad).exists() {
                    return Err(format!("VIDEO_LOAD: Datei '{}' nicht gefunden", pfad));
                }
                let video = crate::video::Video::oeffnen(&pfad)?;
                if self.gfx.is_none() {
                    self.gfx = Some(crate::graphics::Graphics::new_headless()
                        .map_err(|e| format!("VIDEO_LOAD: {}", e))?);
                }
                let g = self.gfx.as_mut().unwrap();
                let bild = g.image_new(video.breite as i32, video.hoehe as i32, Some(0))?;
                g.image_punkte_setzen(bild, &video.rgba)?;
                let geschrieben = video.bild_nr;
                self.videos.push(Some(VideoZustand {
                    video, bild, zeit: 0.0, spielt: false, schleife: false, fertig: false,
                    stand: None, geschrieben,
                }));
                Value::Int((self.videos.len() - 1) as i64)
            }
            "video_play" => {
                let i = self.video_z(a, "VIDEO_PLAY")?;
                let z = self.videos[i].as_mut().unwrap();
                // Ein zu Ende gelaufenes Video beginnt von vorn.
                if z.fertig { z.zeit = 0.0; z.fertig = false; }
                z.spielt = true;
                // Die Uhr laeuft ab dem NAECHSTEN Bild -- sonst zaehlte die
                // Zeit vor dem Start mit.
                z.stand = self.gfx.as_ref().map(|g| g.bild_zaehler());
                Value::Nil
            }
            "video_pause" => {
                let i = self.video_z(a, "VIDEO_PAUSE")?;
                self.videos[i].as_mut().unwrap().spielt = false;
                Value::Nil
            }
            "video_stop" => {
                let i = self.video_z(a, "VIDEO_STOP")?;
                let z = self.videos[i].as_mut().unwrap();
                z.spielt = false; z.fertig = false; z.zeit = 0.0;
                self.video_bild_zeigen(i)?;
                Value::Nil
            }
            "video_seek" => {
                let i = self.video_z(a, "VIDEO_SEEK")?;
                let t = bi_num(a, 1, "VIDEO_SEEK")?;
                let z = self.videos[i].as_mut().unwrap();
                z.zeit = t.clamp(0.0, z.video.dauer);
                z.fertig = false;
                self.video_bild_zeigen(i)?;
                Value::Nil
            }
            "video_loop" => {
                let i = self.video_z(a, "VIDEO_LOOP")?;
                let an = match a.get(1) { Some(Value::Bool(b)) => *b, Some(Value::Int(n)) => *n != 0,
                    _ => return Err("VIDEO_LOOP: erwartet (video, an)".into()) };
                self.videos[i].as_mut().unwrap().schleife = an;
                Value::Nil
            }
            "video_update" => {
                let i = self.video_z(a, "VIDEO_UPDATE")?;
                self.video_weiter(i)?;
                Value::Nil
            }
            "video_draw" => {
                let i = self.video_z(a, "VIDEO_DRAW")?;
                if !(a.len() == 3 || a.len() == 5) { return Err("VIDEO_DRAW: erwartet (video, x, y [, breite, hoehe])".into()); }
                let x = bi_int(a, 1, "VIDEO_DRAW")? as i32;
                let y = bi_int(a, 2, "VIDEO_DRAW")? as i32;
                self.video_weiter(i)?;
                let z = self.videos[i].as_ref().unwrap();
                let (w, h) = if a.len() == 5 { (bi_int(a, 3, "VIDEO_DRAW")? as i32, bi_int(a, 4, "VIDEO_DRAW")? as i32) }
                             else { (z.video.breite as i32, z.video.hoehe as i32) };
                let bild = z.bild;
                match self.gfx.as_mut() {
                    Some(g) if g.schirm_bereit() => g.draw_image_rect(bild, x, y, w, h),
                    _ => return Err("VIDEO_DRAW: es gibt noch kein Fenster -- vor dem ersten Zeichenbefehl SCREEN(breite, hoehe, \"Titel\") aufrufen".into()),
                }
                Value::Nil
            }
            "video_playing" => { let i = self.video_z(a, "VIDEO_PLAYING")?; Value::Bool(self.videos[i].as_ref().unwrap().spielt) }
            "video_done" => { let i = self.video_z(a, "VIDEO_DONE")?; Value::Bool(self.videos[i].as_ref().unwrap().fertig) }
            "video_position" => { let i = self.video_z(a, "VIDEO_POSITION")?; Value::Float(self.videos[i].as_ref().unwrap().zeit) }
            "video_duration" => { let i = self.video_z(a, "VIDEO_DURATION")?; Value::Float(self.videos[i].as_ref().unwrap().video.dauer) }
            "video_width" => { let i = self.video_z(a, "VIDEO_WIDTH")?; Value::Int(self.videos[i].as_ref().unwrap().video.breite as i64) }
            "video_height" => { let i = self.video_z(a, "VIDEO_HEIGHT")?; Value::Int(self.videos[i].as_ref().unwrap().video.hoehe as i64) }
            "video_frames" => { let i = self.video_z(a, "VIDEO_FRAMES")?; Value::Int(self.videos[i].as_ref().unwrap().video.bilder as i64) }
            "video_frame" => { let i = self.video_z(a, "VIDEO_FRAME")?; Value::Int(self.videos[i].as_ref().unwrap().geschrieben) }
            "video_fps" => { let i = self.video_z(a, "VIDEO_FPS")?; Value::Float(self.videos[i].as_ref().unwrap().video.fps) }
            "video_image" => { let i = self.video_z(a, "VIDEO_IMAGE")?; Value::Int(self.videos[i].as_ref().unwrap().bild) }
            "video_free" => {
                let i = self.video_z(a, "VIDEO_FREE")?;
                let z = self.videos[i].take().unwrap();
                if let Some(g) = self.gfx.as_mut() { let _ = g.image_free(z.bild); }
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    #[cfg(feature = "midi")]
    pub(super) fn midi_in(&mut self, idx: i64) -> R<&mut crate::midi::Eingang> {
        Self::handle_get_mut(&mut self.midi_ins, idx, "MIDI", "ungueltiges/geschlossenes MIDI_IN")
    }
    #[cfg(feature = "midi")]
    pub(super) fn midi_out(&mut self, idx: i64) -> R<&mut crate::midi::Ausgang> {
        Self::handle_get_mut(&mut self.midi_outs, idx, "MIDI", "ungueltiges/geschlossenes MIDI_OUT")
    }

    /// MIDI-Dateien (in jedem Bau -- nur das Abspielen braucht ein Geraet).
    pub(super) fn try_midi_datei(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::midifile::{Art, Datei};
        macro_rules! datei { ($f:expr) => {
            Self::handle_get_mut(&mut self.midi_dateien, bi_int(a, 0, $f)?, $f, "ungueltige/freigegebene MIDI_FILE")?
        }; }
        fn spur_nr(a: &[Value], i: usize, fn_: &str) -> R<usize> {
            let k = bi_int(a, i, fn_)?;
            if !(0..crate::midifile::MAX_SPUREN as i64).contains(&k) {
                return Err(format!("{}: Spur {} ausserhalb 0..{}", fn_, k, crate::midifile::MAX_SPUREN - 1));
            }
            Ok(k as usize)
        }
        // Schlaege (Viertel, Kommazahl) -> Ticks der Datei.
        fn tick(d: &Datei, schlag: f64, was: &str, fn_: &str) -> R<u64> {
            let ppq = d.ppq().ok_or_else(|| format!("{}: die Datei zaehlt in SMPTE-Bildern, nicht in Schlaegen", fn_))?;
            if !schlag.is_finite() || schlag < 0.0 || schlag > 1e7 {
                return Err(format!("{}: {} {} -- erwartet eine Zahl ab 0 (in Vierteln)", fn_, was, schlag));
            }
            Ok((schlag * ppq as f64).round() as u64)
        }
        let v = match name {
            "midi_file_new" => {
                const F: &str = "MIDI_FILE_NEW";
                let ppq = if a.is_empty() { 480 } else { bi_int(a, 0, F)? };
                if !(24..=32767).contains(&ppq) { return Err(format!("{}: {} Ticks je Viertel ausserhalb 24..32767", F, ppq)); }
                self.midi_dateien.push(Some(Datei::neu(ppq as u16)));
                Value::Int((self.midi_dateien.len() - 1) as i64)
            }
            "midi_file_load" => {
                const F: &str = "MIDI_FILE_LOAD";
                let pfad = bi_str(a, 0, F)?;
                let b = std::fs::read(pfad).map_err(|e| format!("{}: {}: {}", F, pfad, e))?;
                let d = Datei::lesen(&b).map_err(|e| format!("{}: {}: {}", F, pfad, e))?;
                self.midi_dateien.push(Some(d));
                Value::Int((self.midi_dateien.len() - 1) as i64)
            }
            "midi_file_save" => {
                const F: &str = "MIDI_FILE_SAVE";
                let pfad = bi_str(a, 1, F)?.to_string();
                let b = datei!(F).schreiben();
                std::fs::write(&pfad, b).map_err(|e| format!("{}: {}: {}", F, pfad, e))?;
                Value::Nil
            }
            "midi_file_free" => {
                let i = bi_int(a, 0, "MIDI_FILE_FREE")? as usize;
                if let Some(x) = self.midi_dateien.get_mut(i) { *x = None; }
                Value::Nil
            }
            "midi_file_tracks" => Value::Int(datei!("MIDI_FILE_TRACKS").spuren.len() as i64),
            "midi_file_ppq" => Value::Int(datei!("MIDI_FILE_PPQ").ppq().map(|p| p as i64).unwrap_or(0)),
            "midi_file_duration" => Value::Float(datei!("MIDI_FILE_DURATION").dauer()),
            "midi_file_tempo" => Value::Float(datei!("MIDI_FILE_TEMPO").anfangstempo()),
            "midi_file_track_name$" | "midi_file_track_name" => {
                const F: &str = "MIDI_FILE_TRACK_NAME$";
                let k = spur_nr(a, 1, F)?;
                Value::str_rc(datei!(F).spuren.get(k).map(|s| s.name.clone()).unwrap_or_default())
            }
            "midi_file_set_track_name" => {
                const F: &str = "MIDI_FILE_SET_TRACK_NAME";
                let k = spur_nr(a, 1, F)?;
                let n = bi_str(a, 2, F)?.to_string();
                datei!(F).spur(k)?.name = n;
                Value::Nil
            }
            "midi_file_events" => Value::Int(datei!("MIDI_FILE_EVENTS").ereignis_zahl() as i64),
            "midi_file_event" => {
                const F: &str = "MIDI_FILE_EVENT";
                let i = bi_int(a, 1, F)?;
                let d = datei!(F);
                let n = d.ereignis_zahl();
                let (t, spur, m) = (if i >= 0 { d.ereignis(i as usize) } else { None }).ok_or_else(||
                    format!("{}: Ereignis {} gibt es nicht -- die Datei hat {} (gezaehlt ab 0)", F, i, n))?;
                Value::Tuple(Rc::new(vec![
                    Value::Float(t), Value::Int(spur as i64), Value::Int((m[0] & 0xF0) as i64),
                    Value::Int((m[0] & 0x0F) as i64 + 1), Value::Int(m.get(1).copied().unwrap_or(0) as i64),
                    Value::Int(m.get(2).copied().unwrap_or(0) as i64),
                ]))
            }
            "midi_file_notes" => Value::Int(datei!("MIDI_FILE_NOTES").noten_zahl() as i64),
            "midi_file_note" => {
                const F: &str = "MIDI_FILE_NOTE";
                let i = bi_int(a, 1, F)?;
                let d = datei!(F);
                let n = d.noten_zahl();
                let x = (if i >= 0 { d.note(i as usize) } else { None }).ok_or_else(||
                    format!("{}: Note {} gibt es nicht -- die Datei hat {} (gezaehlt ab 0)", F, i, n))?;
                Value::Tuple(Rc::new(vec![
                    Value::Float(x.start), Value::Float(x.dauer), Value::Int(x.kanal as i64),
                    Value::Int(x.note as i64), Value::Int(x.anschlag as i64), Value::Int(x.spur as i64),
                ]))
            }
            "midi_file_add_note" => {
                const F: &str = "MIDI_FILE_ADD_NOTE";
                let k = spur_nr(a, 1, F)?;
                let kanal = crate::midi::kanal_byte(bi_int(a, 2, F)?, F)?;
                let note = crate::midi::sieben_bit(bi_int(a, 3, F)?, "Note", F)?;
                let ans = crate::midi::sieben_bit(bi_int(a, 4, F)?, "Anschlag", F)?;
                if ans == 0 { return Err(format!("{}: Anschlag 0 ist ein Note-aus -- erwartet 1..127", F)); }
                let (beat, dauer) = (bi_num(a, 5, F)?, bi_num(a, 6, F)?);
                let d = datei!(F);
                let t0 = tick(d, beat, "Beginn", F)?;
                let t1 = tick(d, beat + dauer, "Ende", F)?;
                if dauer <= 0.0 { return Err(format!("{}: Dauer {} -- erwartet mehr als 0 (in Vierteln)", F, dauer)); }
                d.hinzu(k, t0, Art::Kanal(vec![0x90 | kanal, note, ans]))?;
                d.hinzu(k, t1.max(t0 + 1), Art::Kanal(vec![0x80 | kanal, note, 64]))?;
                Value::Nil
            }
            "midi_file_add_event" => {
                const F: &str = "MIDI_FILE_ADD_EVENT";
                let k = spur_nr(a, 1, F)?;
                let beat = bi_num(a, 2, F)?;
                let st = bi_int(a, 3, F)?;
                if !(0x80..=0xEF).contains(&st) {
                    return Err(format!("{}: das Statusbyte muss zwischen 128 und 239 liegen (Kanalnachricht), war {}", F, st));
                }
                let d1 = crate::midi::sieben_bit(bi_int(a, 4, F)?, "Datenbyte 1", F)?;
                let mut m = vec![st as u8, d1];
                if !matches!(st & 0xF0, 0xC0 | 0xD0) {
                    let d2 = if a.len() > 5 { bi_int(a, 5, F)? } else { 0 };
                    m.push(crate::midi::sieben_bit(d2, "Datenbyte 2", F)?);
                }
                let d = datei!(F);
                let t = tick(d, beat, "Zeitpunkt", F)?;
                d.hinzu(k, t, Art::Kanal(m))?;
                Value::Nil
            }
            "midi_file_set_tempo" => {
                const F: &str = "MIDI_FILE_SET_TEMPO";
                let bpm = bi_num(a, 1, F)?;
                if !(4.0..=999.0).contains(&bpm) { return Err(format!("{}: Tempo {} ausserhalb 4..999", F, bpm)); }
                let beat = if a.len() > 2 { bi_num(a, 2, F)? } else { 0.0 };
                let d = datei!(F);
                let t = tick(d, beat, "Zeitpunkt", F)?;
                d.tempo_setzen(t, bpm)?;
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    #[cfg(feature = "midi")]
    pub(super) fn try_midi_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::midi;
        let v = match name {
            // --- Die Uhr ---
            "midi_clock_bpm" => { let i = bi_int(a, 0, "MIDI_CLOCK_BPM")?; Value::Float(midi::takt_bpm(self.midi_in(i)?)) }
            "midi_clock_running" => { let i = bi_int(a, 0, "MIDI_CLOCK_RUNNING")?; Value::Bool(midi::takt_laeuft(self.midi_in(i)?)) }
            "midi_clock_ticks" => { let i = bi_int(a, 0, "MIDI_CLOCK_TICKS")?; Value::Int(midi::takt_ticks(self.midi_in(i)?)) }
            "midi_clock_send" => {
                let i = bi_int(a, 0, "MIDI_CLOCK_SEND")?;
                let bpm = bi_num(a, 1, "MIDI_CLOCK_SEND")?;
                midi::uhr_senden(self.midi_out(i)?, bpm)?;
                Value::Nil
            }
            "midi_start" | "midi_stop" | "midi_continue" => {
                let (b, f) = match name { "midi_start" => (0xFA, "MIDI_START"), "midi_stop" => (0xFC, "MIDI_STOP"), _ => (0xFB, "MIDI_CONTINUE") };
                let i = bi_int(a, 0, f)?;
                midi::send(self.midi_out(i)?, &[b])?;
                Value::Nil
            }
            "midi_song_position" => {
                const F: &str = "MIDI_SONG_POSITION";
                let i = bi_int(a, 0, F)?;
                let p = bi_int(a, 1, F)?;
                if !(0..=16383).contains(&p) { return Err(format!("{}: {} Sechzehntel ausserhalb 0..16383", F, p)); }
                midi::send(self.midi_out(i)?, &[0xF2, (p & 0x7F) as u8, (p >> 7) as u8])?;
                Value::Nil
            }
            // --- Eine Datei abspielen ---
            "midi_file_play" => {
                const F: &str = "MIDI_FILE_PLAY";
                let i = bi_int(a, 0, F)?;
                let k = bi_int(a, 1, F)?;
                let ab = if a.len() > 2 { bi_num(a, 2, F)? } else { 0.0 };
                if !ab.is_finite() || ab < 0.0 { return Err(format!("{}: Beginn {} -- erwartet Sekunden ab 0", F, ab)); }
                let ablauf = Self::handle_get_mut(&mut self.midi_dateien, k, F, "ungueltige/freigegebene MIDI_FILE")?.ablauf();
                midi::datei_spielen(self.midi_out(i)?, ablauf, ab)?;
                Value::Nil
            }
            "midi_file_stop" => { let i = bi_int(a, 0, "MIDI_FILE_STOP")?; midi::datei_halten(self.midi_out(i)?); Value::Nil }
            "midi_file_playing" => { let i = bi_int(a, 0, "MIDI_FILE_PLAYING")?; Value::Bool(midi::datei_spielt(self.midi_out(i)?)) }
            "midi_file_position" => { let i = bi_int(a, 0, "MIDI_FILE_POSITION")?; Value::Float(midi::datei_stand(self.midi_out(i)?)) }
            // --- Anschluesse finden ---
            "midi_in_count" => Value::Int(midi::in_count()?),
            "midi_out_count" => Value::Int(midi::out_count()?),
            "midi_in_name$" | "midi_in_name" =>
                Value::str_rc(midi::in_name(bi_int(a, 0, "MIDI_IN_NAME$")?)?),
            "midi_out_name$" | "midi_out_name" =>
                Value::str_rc(midi::out_name(bi_int(a, 0, "MIDI_OUT_NAME$")?)?),
            // --- Oeffnen und schliessen ---
            "midi_in_open" => {
                let e = midi::in_open(bi_int(a, 0, "MIDI_IN_OPEN")?)?;
                self.midi_ins.push(Some(e));
                Value::Int((self.midi_ins.len() - 1) as i64)
            }
            "midi_out_open" => {
                let o = midi::out_open(bi_int(a, 0, "MIDI_OUT_OPEN")?)?;
                self.midi_outs.push(Some(o));
                Value::Int((self.midi_outs.len() - 1) as i64)
            }
            "midi_in_close" => {
                let i = bi_int(a, 0, "MIDI_IN_CLOSE")? as usize;
                if let Some(s) = self.midi_ins.get_mut(i) { *s = None; }
                Value::Nil
            }
            "midi_out_close" => {
                let i = bi_int(a, 0, "MIDI_OUT_CLOSE")? as usize;
                if let Some(s) = self.midi_outs.get_mut(i) { *s = None; }
                Value::Nil
            }
            // --- Empfangen (Cursor-Muster wie db/mqtt) ---
            "midi_next" => { let i = bi_int(a, 0, "MIDI_NEXT")?; Value::Bool(midi::next(self.midi_in(i)?)) }
            "midi_pending" => { let i = bi_int(a, 0, "MIDI_PENDING")?; Value::Int(midi::wartend(self.midi_in(i)?)) }
            "midi_status" => { let i = bi_int(a, 0, "MIDI_STATUS")?; Value::Int(midi::status(self.midi_in(i)?)) }
            "midi_channel" => { let i = bi_int(a, 0, "MIDI_CHANNEL")?; Value::Int(midi::channel(self.midi_in(i)?)) }
            "midi_data1" => { let i = bi_int(a, 0, "MIDI_DATA1")?; Value::Int(midi::data1(self.midi_in(i)?)) }
            "midi_data2" => { let i = bi_int(a, 0, "MIDI_DATA2")?; Value::Int(midi::data2(self.midi_in(i)?)) }
            "midi_is_note_on" => { let i = bi_int(a, 0, "MIDI_IS_NOTE_ON")?; Value::Bool(midi::is_note_on(self.midi_in(i)?)) }
            "midi_is_note_off" => { let i = bi_int(a, 0, "MIDI_IS_NOTE_OFF")?; Value::Bool(midi::is_note_off(self.midi_in(i)?)) }
            "midi_is_cc" => { let i = bi_int(a, 0, "MIDI_IS_CC")?; Value::Bool(midi::is_cc(self.midi_in(i)?)) }
            // Note/Anschlag und Regler sind nur sprechende Namen fuer data1/data2.
            "midi_note" => { let i = bi_int(a, 0, "MIDI_NOTE")?; Value::Int(midi::data1(self.midi_in(i)?)) }
            "midi_velocity" => { let i = bi_int(a, 0, "MIDI_VELOCITY")?; Value::Int(midi::data2(self.midi_in(i)?)) }
            "midi_cc_number" => { let i = bi_int(a, 0, "MIDI_CC_NUMBER")?; Value::Int(midi::data1(self.midi_in(i)?)) }
            "midi_cc_value" => { let i = bi_int(a, 0, "MIDI_CC_VALUE")?; Value::Int(midi::data2(self.midi_in(i)?)) }
            // --- Senden ---
            "midi_note_on" => {
                let i = bi_int(a, 0, "MIDI_NOTE_ON")?;
                let k = midi::kanal_byte(bi_int(a, 1, "MIDI_NOTE_ON")?, "MIDI_NOTE_ON")?;
                let n = midi::sieben_bit(bi_int(a, 2, "MIDI_NOTE_ON")?, "Note", "MIDI_NOTE_ON")?;
                let v = midi::sieben_bit(bi_int(a, 3, "MIDI_NOTE_ON")?, "Anschlag", "MIDI_NOTE_ON")?;
                midi::send(self.midi_out(i)?, &[0x90 | k, n, v])?;
                Value::Nil
            }
            "midi_note_off" => {
                let i = bi_int(a, 0, "MIDI_NOTE_OFF")?;
                let k = midi::kanal_byte(bi_int(a, 1, "MIDI_NOTE_OFF")?, "MIDI_NOTE_OFF")?;
                let n = midi::sieben_bit(bi_int(a, 2, "MIDI_NOTE_OFF")?, "Note", "MIDI_NOTE_OFF")?;
                midi::send(self.midi_out(i)?, &[0x80 | k, n, 0])?;
                Value::Nil
            }
            "midi_cc" => {
                let i = bi_int(a, 0, "MIDI_CC")?;
                let k = midi::kanal_byte(bi_int(a, 1, "MIDI_CC")?, "MIDI_CC")?;
                let nr = midi::sieben_bit(bi_int(a, 2, "MIDI_CC")?, "Reglernummer", "MIDI_CC")?;
                let w = midi::sieben_bit(bi_int(a, 3, "MIDI_CC")?, "Reglerwert", "MIDI_CC")?;
                midi::send(self.midi_out(i)?, &[0xB0 | k, nr, w])?;
                Value::Nil
            }
            // Roher Ausweg fuer alles, wofuer es oben keinen Namen gibt.
            "midi_send" => {
                let i = bi_int(a, 0, "MIDI_SEND")?;
                let st = bi_int(a, 1, "MIDI_SEND")?;
                if !(0x80..=0xFF).contains(&st) {
                    return Err(format!("MIDI_SEND: das Statusbyte muss zwischen 128 und 255 liegen, war {}", st));
                }
                let d1 = midi::sieben_bit(bi_int(a, 2, "MIDI_SEND")?, "Datenbyte 1", "MIDI_SEND")?;
                let d2 = midi::sieben_bit(bi_int(a, 3, "MIDI_SEND")?, "Datenbyte 2", "MIDI_SEND")?;
                midi::send(self.midi_out(i)?, &[st as u8, d1, d2])?;
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub(super) fn try_serial(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("serial_")) { return Ok(None); }
        #[cfg(feature = "serial")]
        { return self.try_serial_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }
    #[cfg(feature = "serial")]
    pub(super) fn ser_port(&mut self, idx: i64) -> R<&mut crate::serial::Port> {
        Self::handle_get_mut(&mut self.serial_ports, idx, "SERIAL", "ungueltiges/geschlossenes SERIAL_HANDLE")
    }
    #[cfg(feature = "serial")]
    pub(super) fn try_serial_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::serial;
        let v = match name {
            "serial_ports" => Value::str_rc(serial::ports()),
            "serial_open" => {
                let p = serial::open(bi_str(a, 0, "SERIAL_OPEN")?, bi_int(a, 1, "SERIAL_OPEN")?)?;
                self.serial_ports.push(Some(p));
                Value::Int((self.serial_ports.len() - 1) as i64)
            }
            "serial_close" => { let i = bi_int(a, 0, "SERIAL_CLOSE")? as usize; if let Some(s) = self.serial_ports.get_mut(i) { *s = None; } Value::Nil }
            "serial_is_open" => { let i = bi_int(a, 0, "SERIAL_IS_OPEN")?; Value::Bool(self.serial_ports.get(i as usize).map(|o| o.is_some()).unwrap_or(false)) }
            "serial_write" => { let i = bi_int(a, 0, "SERIAL_WRITE")?; let s = bi_str(a, 1, "SERIAL_WRITE")?.to_string(); Value::Int(serial::write(self.ser_port(i)?, &s)?) }
            "serial_read" => { let i = bi_int(a, 0, "SERIAL_READ")?; let n = bi_int(a, 1, "SERIAL_READ")?; Value::str_rc(serial::read(self.ser_port(i)?, n)?) }
            "serial_readline" => { let i = bi_int(a, 0, "SERIAL_READLINE")?; Value::str_rc(serial::readline(self.ser_port(i)?)?) }
            "serial_available" => { let i = bi_int(a, 0, "SERIAL_AVAILABLE")?; Value::Int(serial::available(self.ser_port(i)?)?) }
            "serial_flush" => { let i = bi_int(a, 0, "SERIAL_FLUSH")?; serial::flush(self.ser_port(i)?); Value::Nil }
            "serial_timeout" => { let i = bi_int(a, 0, "SERIAL_TIMEOUT")?; let s = bi_num(a, 1, "SERIAL_TIMEOUT")?; serial::set_timeout(self.ser_port(i)?, s); Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Module gpio/i2c/spi: die Pins des Kernels (pins.rs). In jedem Bau;
    // ausserhalb von Linux meldet jeder Aufruf "nur unter Linux".
    // ===================================================================
    // ===================================================================
    // Mikrofon (MIC_*, src/mikrofon.rs): ein Eingang zur Zeit
    // ===================================================================
    // ===================================================================
    // Modul webcam (src/webcam.rs): Bilder einer Kamera in ein IMAGE
    // ===================================================================
    // ===================================================================
    // WebSocket (WS_*), OSC (OSC_*), mDNS (MDNS_*): src/websocket.rs,
    // src/osc.rs, src/mdns.rs
    // ===================================================================
    pub(super) fn try_netzdienste(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if name.starts_with("ws_") {
            #[cfg(feature = "net")]
            { return self.try_websocket(name, a); }
            #[allow(unreachable_code)]
            { return Ok(None); }
        }
        if name.starts_with("osc_") { return self.try_osc(name, a); }
        if name.starts_with("mdns_") { return self.try_mdns(name, a); }
        Ok(None)
    }

    // ===================================================================
    // Modbus (MODBUS_*, in jedem Bau) und Ein-/Ausstecken (HOTPLUG_*)
    // ===================================================================
    pub(super) fn try_geraete(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if name.starts_with("modbus_") { return self.try_modbus(name, a); }
        if name.starts_with("hotplug_") { return self.try_hotplug(name, a); }
        Ok(None)
    }

    pub(super) fn try_modbus(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::modbus;
        macro_rules! geraet { ($f:expr) => {
            Self::handle_get_mut(&mut self.modbus_geraete, bi_int(a, 0, $f)?, $f, "ungueltiges/geschlossenes MODBUS")?
        }; }
        fn int_opt(a: &[Value], i: usize, vorgabe: i64, fn_: &str) -> R<i64> {
            if a.len() > i { bi_int(a, i, fn_) } else { Ok(vorgabe) }
        }
        fn folge(a: &[Value], i: usize, fn_: &str) -> R<[usize; 4]> {
            let f = if a.len() > i { bi_str(a, i, fn_)?.to_string() } else { "ABCD".to_string() };
            modbus::folge_lesen(&f, fn_)
        }
        fn bool_feld(v: Vec<bool>) -> Value {
            let n = v.len() as i64;
            let mut arr = crate::value::DhArray::new("boolean".to_string(), vec![n], || Value::Bool(false));
            for (k, x) in v.into_iter().enumerate() { arr.cells.set(k, Value::Bool(x)); }
            Value::Array(Rc::new(RefCell::new(arr)))
        }
        fn int_feld(v: Vec<i64>) -> Value {
            let n = v.len() as i64;
            let mut arr = crate::value::DhArray::new("integer".to_string(), vec![n], || Value::Int(0));
            for (k, x) in v.into_iter().enumerate() { arr.cells.set(k, Value::Int(x)); }
            Value::Array(Rc::new(RefCell::new(arr)))
        }
        // Zwei Register aus einem Feld ab Stelle i (zaehlt wie das Feld).
        fn zwei(a: &[Value], fn_: &str) -> R<(u16, u16)> {
            let i = bi_int(a, 1, fn_)?;
            match a.first() {
                Some(Value::Array(arr)) => {
                    let arr = arr.borrow();
                    let u = arr.unten_von(0);
                    let k = i - u;
                    let n = arr.cells.len() as i64;
                    if k < 0 || k + 1 >= n {
                        return Err(format!("{}: Stelle {} und {} -- das Feld hat {} Register", fn_, i, i + 1, n));
                    }
                    let lies = |k: i64| -> R<u16> {
                        match arr.cells.get(k as usize) {
                            Value::Int(x) if (-32768..=65535).contains(&x) => Ok(x as i32 as u16),
                            v => Err(format!("{}: an Stelle {} steht kein Registerwert ({})", fn_, k + u, v.type_name())),
                        }
                    };
                    Ok((lies(k)?, lies(k + 1)?))
                }
                Some(v) => Err(format!("{}: erwartet das Feld aus MODBUS_READ_HOLDING/_INPUT, nicht {}", fn_, v.type_name())),
                None => Err(format!("{}: das Feld fehlt", fn_)),
            }
        }
        let v = match name {
            "modbus_tcp" => {
                const F: &str = "MODBUS_TCP";
                let host = bi_str(a, 0, F)?.to_string();
                let g = modbus::tcp(&host, int_opt(a, 1, 502, F)?, int_opt(a, 2, 1, F)?)?;
                self.modbus_geraete.push(Some(g));
                Value::Int((self.modbus_geraete.len() - 1) as i64)
            }
            "modbus_rtu" => {
                const F: &str = "MODBUS_RTU";
                let ziel = bi_str(a, 0, F)?.to_string();
                let baud = int_opt(a, 1, 9600, F)?;
                let format = if a.len() > 2 { bi_str(a, 2, F)?.to_string() } else { "8N1".to_string() };
                let g = modbus::rtu(&ziel, baud, &format, int_opt(a, 3, 1, F)?)?;
                self.modbus_geraete.push(Some(g));
                Value::Int((self.modbus_geraete.len() - 1) as i64)
            }
            "modbus_close" => {
                let i = bi_int(a, 0, "MODBUS_CLOSE")? as usize;
                if let Some(x) = self.modbus_geraete.get_mut(i) { *x = None; }
                Value::Nil
            }
            "modbus_unit" => {
                let e = modbus::einheit_pruefen(bi_int(a, 1, "MODBUS_UNIT")?, "MODBUS_UNIT")?;
                geraet!("MODBUS_UNIT").einheit = e;
                Value::Nil
            }
            "modbus_timeout" => {
                let ms = bi_int(a, 1, "MODBUS_TIMEOUT")?;
                if !(10..=60000).contains(&ms) { return Err(format!("MODBUS_TIMEOUT: {} ms ausserhalb 10..60000", ms)); }
                geraet!("MODBUS_TIMEOUT").zeitgrenze = std::time::Duration::from_millis(ms as u64);
                Value::Nil
            }
            "modbus_read_coils" | "modbus_read_discrete" => {
                let (fc, f) = if name == "modbus_read_coils" { (1, "MODBUS_READ_COILS") } else { (2, "MODBUS_READ_DISCRETE") };
                let (adr, n) = (bi_int(a, 1, f)?, bi_int(a, 2, f)?);
                bool_feld(geraet!(f).bits(fc, adr, n, f)?)
            }
            "modbus_read_holding" | "modbus_read_input" => {
                let (fc, f) = if name == "modbus_read_holding" { (3, "MODBUS_READ_HOLDING") } else { (4, "MODBUS_READ_INPUT") };
                let (adr, n) = (bi_int(a, 1, f)?, bi_int(a, 2, f)?);
                int_feld(geraet!(f).register(fc, adr, n, f)?.into_iter().map(|x| x as i64).collect())
            }
            "modbus_write_coil" => {
                const F: &str = "MODBUS_WRITE_COIL";
                let (adr, _) = modbus::bereich(bi_int(a, 1, F)?, 1, 1, F)?;
                let an = bi_bool(a, 2, F)?;
                geraet!(F).schreiben(modbus::pdu_spule(adr, an), F)?;
                Value::Nil
            }
            "modbus_write_register" => {
                const F: &str = "MODBUS_WRITE_REGISTER";
                let (adr, _) = modbus::bereich(bi_int(a, 1, F)?, 1, 1, F)?;
                let w = modbus::registerwert(bi_int(a, 2, F)?, F)?;
                geraet!(F).schreiben(modbus::pdu_register(adr, w), F)?;
                Value::Nil
            }
            "modbus_write_coils" | "modbus_write_registers" => {
                let spulen = name == "modbus_write_coils";
                let f = if spulen { "MODBUS_WRITE_COILS" } else { "MODBUS_WRITE_REGISTERS" };
                let werte: Vec<Value> = match a.get(2) {
                    Some(Value::Array(arr)) => { let arr = arr.borrow(); (0..arr.cells.len()).map(|k| arr.cells.get(k)).collect() }
                    Some(v) => return Err(format!("{}: erwartet ein Feld, nicht {}", f, v.type_name())),
                    None => return Err(format!("{}: das Feld fehlt", f)),
                };
                let (adr, _) = modbus::bereich(bi_int(a, 1, f)?, werte.len() as i64, if spulen { 1968 } else { 123 }, f)?;
                let pdu = if spulen {
                    let mut b = Vec::with_capacity(werte.len());
                    for (k, w) in werte.iter().enumerate() {
                        b.push(match w {
                            Value::Bool(x) => *x,
                            Value::Int(0) => false,
                            Value::Int(1) => true,
                            v => return Err(format!("{}: an Stelle {} steht kein Wahrheitswert ({})", f, k, v.type_name())),
                        });
                    }
                    modbus::pdu_spulen(adr, &b)
                } else {
                    let mut r = Vec::with_capacity(werte.len());
                    for (k, w) in werte.iter().enumerate() {
                        match w {
                            Value::Int(x) => r.push(modbus::registerwert(*x, f)?),
                            v => return Err(format!("{}: an Stelle {} steht keine ganze Zahl ({})", f, k, v.type_name())),
                        }
                    }
                    modbus::pdu_registers(adr, &r)
                };
                geraet!(f).schreiben(pdu, f)?;
                Value::Nil
            }
            // --- Werte umrechnen ---
            "modbus_int16" => {
                let w = bi_int(a, 0, "MODBUS_INT16")?;
                Value::Int(modbus::registerwert(w, "MODBUS_INT16")? as i16 as i64)
            }
            "modbus_int32" | "modbus_uint32" | "modbus_float" => {
                let f = match name { "modbus_int32" => "MODBUS_INT32", "modbus_uint32" => "MODBUS_UINT32", _ => "MODBUS_FLOAT" };
                let (r0, r1) = zwei(a, f)?;
                let b = modbus::zwei_register(r0, r1, folge(a, 2, f)?);
                match name {
                    "modbus_int32" => Value::Int(i32::from_be_bytes(b) as i64),
                    "modbus_uint32" => Value::Int(u32::from_be_bytes(b) as i64),
                    _ => Value::Float(f32::from_be_bytes(b) as f64),
                }
            }
            "modbus_int32_regs" | "modbus_float_regs" => {
                let f = if name == "modbus_float_regs" { "MODBUS_FLOAT_REGS" } else { "MODBUS_INT32_REGS" };
                let b = if name == "modbus_float_regs" {
                    (bi_num(a, 0, f)? as f32).to_be_bytes()
                } else {
                    let n = bi_int(a, 0, f)?;
                    if !(-2147483648..=4294967295).contains(&n) {
                        return Err(format!("{}: {} passt nicht in 32 Bit", f, n));
                    }
                    (n as u32).to_be_bytes()
                };
                let r = modbus::als_register(b, folge(a, 1, f)?);
                int_feld(vec![r[0] as i64, r[1] as i64])
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub(super) fn try_hotplug(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::hotplug;
        fn text_feld(v: Vec<String>) -> Value {
            let n = v.len() as i64;
            let mut arr = crate::value::DhArray::new("string".to_string(), vec![n], || Value::str_rc(String::new()));
            for (k, x) in v.into_iter().enumerate() { arr.cells.set(k, Value::str_rc(x)); }
            Value::Array(Rc::new(RefCell::new(arr)))
        }
        let v = match name {
            "hotplug_kinds" => text_feld(hotplug::arten().into_iter().map(String::from).collect()),
            "hotplug_list" => text_feld(hotplug::aktuelle(bi_str(a, 0, "HOTPLUG_LIST")?)?),
            "hotplug_interval" => { hotplug::intervall(bi_int(a, 0, "HOTPLUG_INTERVAL")?)?; Value::Nil }
            "hotplug_next" => {
                self.hotplug_aktuell = hotplug::naechstes();
                Value::Bool(self.hotplug_aktuell.is_some())
            }
            "hotplug_kind$" | "hotplug_kind" => Value::str_rc(self.hotplug_aktuell.as_ref().map(|e| e.art.clone()).unwrap_or_default()),
            "hotplug_name$" | "hotplug_name" => Value::str_rc(self.hotplug_aktuell.as_ref().map(|e| e.name.clone()).unwrap_or_default()),
            "hotplug_added" => Value::Bool(self.hotplug_aktuell.as_ref().map(|e| e.dazu).unwrap_or(false)),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    #[cfg(feature = "net")]
    pub(super) fn try_websocket(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::websocket as ws;
        fn puffer(v: Vec<u8>) -> Value { Value::Buffer(Rc::new(RefCell::new(v))) }
        macro_rules! vb { ($f:expr) => {
            Self::handle_get_mut(&mut self.ws_verbindungen, bi_int(a, 0, $f)?, $f, "ungueltiges/geschlossenes WEBSOCKET")?
        }; }
        macro_rules! srv { ($f:expr) => {
            Self::handle_get_mut(&mut self.ws_server, bi_int(a, 0, $f)?, $f, "ungueltiger/geschlossener WS_SERVER")?
        }; }
        let v = match name {
            "ws_connect" => {
                let url = bi_str(a, 0, "WS_CONNECT")?.to_string();
                self.ws_verbindungen.push(Some(ws::verbinden(&url)?));
                Value::Int((self.ws_verbindungen.len() - 1) as i64)
            }
            "ws_listen" => {
                let port = bi_int(a, 0, "WS_LISTEN")?;
                let bind = if a.len() >= 2 { bi_str(a, 1, "WS_LISTEN")?.to_string() } else { "0.0.0.0".to_string() };
                self.ws_server.push(Some(ws::lauschen(port, &bind)?));
                Value::Int((self.ws_server.len() - 1) as i64)
            }
            "ws_port" => Value::Int(srv!("WS_PORT").port as i64),
            "ws_accept" => {
                match srv!("WS_ACCEPT").annehmen() {
                    Some(v) => { self.ws_verbindungen.push(Some(v)); Value::Int((self.ws_verbindungen.len() - 1) as i64) }
                    None => Value::Int(-1),
                }
            }
            "ws_stop" => { let i = bi_int(a, 0, "WS_STOP")? as usize; if let Some(x) = self.ws_server.get_mut(i) { *x = None; } Value::Nil }
            "ws_send" => {
                const F: &str = "WS_SEND";
                let (daten, binaer) = match a.get(1) {
                    Some(Value::Str(t)) => (t.as_bytes().to_vec(), false),
                    Some(Value::Buffer(b)) => (b.borrow().clone(), true),
                    Some(v) => return Err(format!("{}: erwartet einen Text oder einen BUFFER, nicht {}", F, v.type_name())),
                    None => return Err(format!("{}: fehlendes Argument 2", F)),
                };
                if daten.len() > ws::MAX_NACHRICHT { return Err(format!("{}: hoechstens {} Bytes je Nachricht", F, ws::MAX_NACHRICHT)); }
                Value::Bool(vb!(F).senden(&daten, binaer)?)
            }
            "ws_next" => Value::Bool(vb!("WS_NEXT").naechste()),
            "ws_message$" | "ws_message" => {
                let n = vb!("WS_MESSAGE$").aktuell().map(|n| String::from_utf8_lossy(&n.daten).into_owned()).unwrap_or_default();
                Value::str_rc(n)
            }
            "ws_message_bytes" => puffer(vb!("WS_MESSAGE_BYTES").aktuell().map(|n| n.daten.clone()).unwrap_or_default()),
            "ws_is_binary" => Value::Bool(vb!("WS_IS_BINARY").aktuell().map_or(false, |n| n.binaer)),
            "ws_pending" => Value::Int(vb!("WS_PENDING").wartend() as i64),
            "ws_connected" => Value::Bool(vb!("WS_CONNECTED").offen()),
            "ws_ping" => Value::Bool(vb!("WS_PING").ping()),
            "ws_path$" | "ws_path" => Value::str_rc(vb!("WS_PATH$").pfad.clone()),
            "ws_peer$" | "ws_peer" => Value::str_rc(vb!("WS_PEER$").gegenstelle.clone()),
            "ws_close_code" => Value::Int(vb!("WS_CLOSE_CODE").schliess_code),
            "ws_close_reason$" | "ws_close_reason" => Value::str_rc(vb!("WS_CLOSE_REASON$").schliess_grund.clone()),
            "ws_close" => {
                const F: &str = "WS_CLOSE";
                let code = if a.len() >= 2 { bi_int(a, 1, F)? } else { 1000 };
                if !(1000..=4999).contains(&code) || matches!(code, 1004..=1006 | 1015) {
                    return Err(format!("{}: Code {} ist nicht erlaubt (1000 normal, 1001 weg, 4000..4999 eigene)", F, code));
                }
                let grund = if a.len() >= 3 { bi_str(a, 2, F)?.to_string() } else { String::new() };
                let i = bi_int(a, 0, F)?;
                vb!(F).schliessen(code as u16, &grund);
                let _ = i;
                Value::Nil
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub(super) fn try_osc(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::osc;
        macro_rules! anschluss { ($f:expr) => {
            Self::handle_get_mut(&mut self.osc_anschluesse, bi_int(a, 0, $f)?, $f, "ungueltiger/geschlossener OSC")?
        }; }
        let v = match name {
            "osc_open" => {
                let port = if a.is_empty() { 0 } else { bi_int(a, 0, "OSC_OPEN")? };
                let bind = if a.len() >= 2 { bi_str(a, 1, "OSC_OPEN")?.to_string() } else { "0.0.0.0".to_string() };
                self.osc_anschluesse.push(Some(osc::oeffnen(port, &bind)?));
                Value::Int((self.osc_anschluesse.len() - 1) as i64)
            }
            "osc_close" => { let i = bi_int(a, 0, "OSC_CLOSE")? as usize; if let Some(x) = self.osc_anschluesse.get_mut(i) { *x = None; } Value::Nil }
            "osc_port" => Value::Int(anschluss!("OSC_PORT").port as i64),
            "osc_send" => {
                const F: &str = "OSC_SEND";
                if a.len() < 4 { return Err(format!("{}: erwartet (osc, host$, port, adresse$, werte...)", F)); }
                let host = bi_str(a, 1, F)?.to_string();
                let port = bi_int(a, 2, F)?;
                if !(1..=65535).contains(&port) { return Err(format!("{}: Port {} ausserhalb 1..65535", F, port)); }
                let adresse = bi_str(a, 3, F)?.to_string();
                if !adresse.starts_with('/') { return Err(format!("{}: eine OSC-Adresse beginnt mit /, nicht {:?}", F, adresse)); }
                let mut werte = Vec::new();
                for (k, w) in a[4..].iter().enumerate() {
                    werte.push(match w {
                        Value::Int(n) => if let Ok(x) = i32::try_from(*n) { osc::Wert::Int(x) } else { osc::Wert::Long(*n) },
                        Value::Float(f) => osc::Wert::Float(*f as f32),
                        Value::Str(t) => osc::Wert::Text(t.to_string()),
                        Value::Bool(b) => if *b { osc::Wert::Wahr } else { osc::Wert::Falsch },
                        Value::Buffer(b) => osc::Wert::Blob(b.borrow().clone()),
                        Value::Nil => osc::Wert::Nichts,
                        v => return Err(format!("{}: Wert {} ist {} -- OSC kennt Zahlen, Texte, Wahrheitswerte und BUFFER", F, k + 1, v.type_name())),
                    });
                }
                let n = osc::Nachricht { adresse, werte };
                anschluss!(F).senden(&host, port as u16, &n)?;
                Value::Nil
            }
            "osc_next" => Value::Bool(anschluss!("OSC_NEXT").naechste()),
            "osc_address$" | "osc_address" => Value::str_rc(anschluss!("OSC_ADDRESS$").aktuell().map(|(n, _)| n.adresse.clone()).unwrap_or_default()),
            "osc_types$" | "osc_types" => Value::str_rc(anschluss!("OSC_TYPES$").aktuell().map(|(n, _)| n.werte.iter().map(|w| w.kennung()).collect::<String>()).unwrap_or_default()),
            "osc_count" => Value::Int(anschluss!("OSC_COUNT").aktuell().map_or(0, |(n, _)| n.werte.len() as i64)),
            "osc_arg" => {
                const F: &str = "OSC_ARG";
                let i = bi_int(a, 1, F)?;
                let werte = anschluss!(F).aktuell().map(|(n, _)| n.werte.clone()).unwrap_or_default();
                let w = usize::try_from(i).ok().and_then(|i| werte.get(i))
                    .ok_or_else(|| format!("{}: Wert {} gibt es nicht -- die Nachricht hat {} (gezaehlt ab 0)", F, i, werte.len()))?;
                match w {
                    osc::Wert::Int(x) => Value::Int(*x as i64),
                    osc::Wert::Long(x) => Value::Int(*x),
                    osc::Wert::Float(x) => Value::Float(*x as f64),
                    osc::Wert::Double(x) => Value::Float(*x),
                    osc::Wert::Text(t) => Value::str_rc(t.clone()),
                    osc::Wert::Blob(d) => Value::Buffer(Rc::new(RefCell::new(d.clone()))),
                    osc::Wert::Wahr => Value::Bool(true),
                    osc::Wert::Falsch => Value::Bool(false),
                    osc::Wert::Nichts => Value::Nil,
                    osc::Wert::Unendlich => Value::Float(f64::INFINITY),
                    osc::Wert::Zeit(t) => Value::Int(*t as i64),
                    osc::Wert::Zeichen(c) => Value::str_rc(c.to_string()),
                    osc::Wert::Farbe(c) => Value::Int(*c as i64),
                    osc::Wert::Midi(m) => Value::Int(u32::from_be_bytes(*m) as i64),
                }
            }
            "osc_sender$" | "osc_sender" => Value::str_rc(anschluss!("OSC_SENDER$").aktuell().map(|(_, s)| s.ip().to_string()).unwrap_or_default()),
            "osc_sender_port" => Value::Int(anschluss!("OSC_SENDER_PORT").aktuell().map_or(0, |(_, s)| s.port() as i64)),
            "osc_match" => {
                let m = bi_str(a, 0, "OSC_MATCH")?.to_string();
                let ad = bi_str(a, 1, "OSC_MATCH")?.to_string();
                Value::Bool(osc::passt(&m, &ad))
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub(super) fn try_mdns(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::mdns;
        fn frist(a: &[Value], i: usize, f: &str, vorgabe: i64) -> R<u64> {
            let ms = if a.len() > i { bi_int(a, i, f)? } else { vorgabe };
            if !(50..=30000).contains(&ms) { return Err(format!("{}: Wartezeit {} ms ausserhalb 50..30000", f, ms)); }
            Ok(ms as u64)
        }
        macro_rules! dienst { ($f:expr) => {{
            let i = bi_int(a, 0, $f)?;
            let n = self.mdns_dienste.len();
            usize::try_from(i).ok().and_then(|i| self.mdns_dienste.get(i))
                .ok_or_else(|| format!("{}: Dienst {} gibt es nicht -- die letzte Suche fand {} (gezaehlt ab 0)", $f, i, n))?
        }}; }
        let v = match name {
            "mdns_resolve$" | "mdns_resolve" => {
                let n = bi_str(a, 0, "MDNS_RESOLVE$")?.to_string();
                let ms = frist(a, 1, "MDNS_RESOLVE$", 1500)?;
                Value::str_rc(mdns::aufloesen(&n, ms)?)
            }
            "mdns_browse" => {
                let d = bi_str(a, 0, "MDNS_BROWSE")?.to_string();
                if !d.starts_with('_') { return Err(format!("MDNS_BROWSE: ein Diensttyp sieht aus wie \"_http._tcp\", nicht {:?}", d)); }
                let ms = frist(a, 1, "MDNS_BROWSE", 2000)?;
                self.mdns_typ = mdns::lokal(&d);
                self.mdns_dienste = mdns::suchen(&d, ms)?;
                Value::Int(self.mdns_dienste.len() as i64)
            }
            "mdns_service_name$" | "mdns_service_name" => { let t = self.mdns_typ.clone(); let d = dienst!("MDNS_SERVICE_NAME$"); Value::str_rc(mdns::kurzname(&d.name, &t)) }
            "mdns_service_host$" | "mdns_service_host" => Value::str_rc(dienst!("MDNS_SERVICE_HOST$").host.clone()),
            "mdns_service_address$" | "mdns_service_address" => Value::str_rc(dienst!("MDNS_SERVICE_ADDRESS$").adresse.clone()),
            "mdns_service_port" => Value::Int(dienst!("MDNS_SERVICE_PORT").port as i64),
            "mdns_service_txt$" | "mdns_service_txt" => {
                let k = bi_str(a, 1, "MDNS_SERVICE_TXT$")?.to_string();
                let d = dienst!("MDNS_SERVICE_TXT$");
                Value::str_rc(d.txt.iter().find(|(n, _)| n.eq_ignore_ascii_case(&k)).map(|(_, w)| w.clone()).unwrap_or_default())
            }
            "mdns_announce" => {
                const F: &str = "MDNS_ANNOUNCE";
                let n = bi_str(a, 0, F)?.to_string();
                let d = bi_str(a, 1, F)?.to_string();
                let port = bi_int(a, 2, F)?;
                if !(1..=65535).contains(&port) { return Err(format!("{}: Port {} ausserhalb 1..65535", F, port)); }
                if n.is_empty() || n.contains('.') { return Err(format!("{}: der Name darf nicht leer sein und keinen Punkt enthalten, nicht {:?}", F, n)); }
                if !d.starts_with('_') { return Err(format!("{}: ein Diensttyp sieht aus wie \"_http._tcp\", nicht {:?}", F, d)); }
                let txt = if a.len() >= 4 { mdns::txt_aus_text(bi_str(a, 3, F)?) } else { Vec::new() };
                let host: String = n.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c.to_ascii_lowercase() } else { '-' }).collect();
                let ank = mdns::Ankuendigung { name: n, dienst: mdns::lokal(&d), host: mdns::lokal(&host), port: port as u16, txt };
                if self.mdns_antworter.is_none() { self.mdns_antworter = Some(mdns::antworter_starten()?); }
                let liste = self.mdns_antworter.as_ref().unwrap().liste.clone();
                {
                    let mut l = liste.lock().unwrap_or_else(|e| e.into_inner());
                    l.retain(|x| x.instanz() != ank.instanz());
                    l.push(ank.clone());
                }
                mdns::melden(&ank);
                Value::str_rc(ank.host)
            }
            "mdns_stop" => { self.mdns_antworter = None; Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub(super) fn try_webcam(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("webcam_") { return Ok(None); }
        #[cfg(feature = "graphics")]
        { return self.try_webcam_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = a; Ok(None) }
    }

    #[cfg(feature = "graphics")]
    pub(super) fn webcam_z(&mut self, a: &[Value], f: &str) -> R<usize> {
        let i = bi_int(a, 0, f)?;
        match self.webcams.get(i.max(0) as usize) {
            Some(Some(_)) if i >= 0 => Ok(i as usize),
            _ => Err(format!("{}: ungueltige oder geschlossene WEBCAM {}", f, i)),
        }
    }

    /// Ein neues Bild der Kamera ins IMAGE schreiben; TRUE, wenn eins kam.
    /// Hat sich die Groesse geaendert, gibt es ein neues IMAGE (WEBCAM_IMAGE
    /// liefert es).
    #[cfg(feature = "graphics")]
    pub(super) fn webcam_holen(&mut self, i: usize) -> R<bool> {
        let z = self.webcams[i].as_mut().unwrap();
        let mut px = std::mem::take(&mut z.px);
        let Some((b, h)) = z.kamera.neues(&mut px) else { z.px = px; return Ok(false) };
        let (alt_bild, alt_mass) = (z.bild, z.mass);
        let g = self.gfx.as_mut().ok_or("WEBCAM: kein Bild ohne Grafik")?;
        let bild = if alt_bild >= 0 && alt_mass == (b, h) { alt_bild } else {
            if alt_bild >= 0 { let _ = g.image_free(alt_bild); }
            g.image_new(b as i32, h as i32, Some(0))?
        };
        let r = g.image_punkte_setzen(bild, &px);
        let z = self.webcams[i].as_mut().unwrap();
        z.bild = bild;
        z.mass = (b, h);
        z.px = px;
        r.map(|_| true)
    }

    #[cfg(feature = "graphics")]
    pub(super) fn try_webcam_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::webcam;
        let v = match name {
            "webcam_devices" => {
                if !a.is_empty() { return Err(format!("WEBCAM_DEVICES: erwartet keine Argumente, erhalten {}", a.len())); }
                let namen = webcam::geraete()?;
                let n = namen.len() as i64;
                let mut arr = crate::value::DhArray::new("string".to_string(), vec![n], || Value::str_rc(""));
                for (k, s) in namen.into_iter().enumerate() { arr.cells.set(k, Value::str_rc(s)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            "webcam_open" => {
                const F: &str = "WEBCAM_OPEN";
                if a.len() > 3 || a.len() == 2 {
                    return Err(format!("{}: erwartet 0, 1 oder 3 Argumente (geraet$, breite, hoehe), erhalten {}", F, a.len()));
                }
                let wunsch = if a.is_empty() { String::new() } else { bi_str(a, 0, F)?.to_string() };
                let (b, h) = if a.len() == 3 { (bi_int(a, 1, F)?, bi_int(a, 2, F)?) } else { (0, 0) };
                if !(0..=8192).contains(&b) || !(0..=8192).contains(&h) {
                    return Err(format!("{}: breite und hoehe muessen 0..8192 sein (0 = was die Kamera vorschlaegt)", F));
                }
                // Das Bild braucht die Grafik (ein IMAGE lebt auf der GPU):
                // ohne SCREEN wird ein verstecktes Fenster angelegt, wie bei LOADIMAGE.
                if self.gfx.is_none() {
                    self.gfx = Some(crate::graphics::Graphics::new_headless()
                        .map_err(|e| format!("{}: {}", F, e))?);
                }
                let kamera = webcam::oeffnen(&wunsch, b as u32, h as u32)?;
                let mass = kamera.mass();
                let g = self.gfx.as_mut().ok_or("WEBCAM_OPEN: kein Bild ohne Grafik")?;
                let (mb, mh) = (mass.0.max(1), mass.1.max(1));
                let bild = g.image_new(mb as i32, mh as i32, Some(0))?;
                self.webcams.push(Some(WebcamZustand { kamera, bild, mass: (mb, mh), px: Vec::new() }));
                Value::Int((self.webcams.len() - 1) as i64)
            }
            "webcam_close" => {
                let i = self.webcam_z(a, "WEBCAM_CLOSE")?;
                if let Some(z) = self.webcams[i].take() {
                    if let Some(g) = self.gfx.as_mut() { let _ = g.image_free(z.bild); }
                }
                Value::Nil
            }
            "webcam_update" => { let i = self.webcam_z(a, "WEBCAM_UPDATE")?; Value::Bool(self.webcam_holen(i)?) }
            "webcam_image" => {
                let i = self.webcam_z(a, "WEBCAM_IMAGE")?;
                self.webcam_holen(i)?;
                Value::Int(self.webcams[i].as_ref().unwrap().bild)
            }
            "webcam_width" => { let i = self.webcam_z(a, "WEBCAM_WIDTH")?; Value::Int(self.webcams[i].as_ref().unwrap().mass.0 as i64) }
            "webcam_height" => { let i = self.webcam_z(a, "WEBCAM_HEIGHT")?; Value::Int(self.webcams[i].as_ref().unwrap().mass.1 as i64) }
            "webcam_frames" => { let i = self.webcam_z(a, "WEBCAM_FRAMES")?; Value::Int(self.webcams[i].as_ref().unwrap().kamera.bilder() as i64) }
            "webcam_name$" => { let i = self.webcam_z(a, "WEBCAM_NAME$")?; Value::str_rc(self.webcams[i].as_ref().unwrap().kamera.name.clone()) }
            "webcam_error$" => { let i = self.webcam_z(a, "WEBCAM_ERROR$")?; Value::str_rc(self.webcams[i].as_ref().unwrap().kamera.fehler().unwrap_or_default()) }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub(super) fn try_mikrofon(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("mic_") { return Ok(None); }
        #[cfg(feature = "graphics")]
        { return self.try_mikrofon_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = a; Ok(None) }
    }

    #[cfg(feature = "graphics")]
    pub(super) fn try_mikrofon_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::mikrofon as mic;
        fn float_feld(a: &[Value], f: &str) -> R<Rc<RefCell<crate::value::DhArray>>> {
            let arr = match a.first() {
                Some(Value::Array(x)) => x.clone(),
                _ => return Err(format!("{}: erwartet ARRAY OF FLOAT", f)),
            };
            {
                let b = arr.borrow();
                if b.element_type != "float" || b.dims.len() != 1 {
                    return Err(format!("{}: erwartet 1D ARRAY OF FLOAT", f));
                }
            }
            Ok(arr)
        }
        fn keine_args(a: &[Value], f: &str) -> R<()> {
            if a.is_empty() { Ok(()) } else { Err(format!("{}: erwartet keine Argumente, erhalten {}", f, a.len())) }
        }
        // Was nur mit laufendem Eingang einen Sinn hat: ohne ihn waere eine
        // stille Null von "es ist leise" nicht zu unterscheiden.
        macro_rules! lauf { ($f:expr) => {
            match self.mikrofon.as_ref() {
                Some(l) => l,
                None => return Err(format!("{}: es laeuft kein Mikrofon -- erst MIC_START", $f)),
            }
        }; }
        let v = match name {
            "mic_devices" => {
                keine_args(a, "MIC_DEVICES")?;
                let namen = mic::geraete()?;
                let n = namen.len() as i64;
                let mut arr = crate::value::DhArray::new("string".to_string(), vec![n], || Value::str_rc(""));
                for (k, s) in namen.into_iter().enumerate() { arr.cells.set(k, Value::str_rc(s)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            "mic_start" => {
                if a.len() > 1 { return Err(format!("MIC_START: erwartet 0..1 Argumente, erhalten {}", a.len())); }
                let wunsch = if a.is_empty() { String::new() } else { bi_str(a, 0, "MIC_START")?.to_string() };
                self.mikrofon = None;
                self.mic_baender.clear();
                self.mic_agc = 1e-4;
                self.mikrofon = Some(mic::starten(&wunsch)?);
                Value::Nil
            }
            "mic_stop" => { keine_args(a, "MIC_STOP")?; self.mikrofon = None; Value::Nil }
            "mic_active" => { keine_args(a, "MIC_ACTIVE")?; Value::Bool(self.mikrofon.is_some() && mic::fehler().is_none()) }
            "mic_error$" => { keine_args(a, "MIC_ERROR$")?; Value::str_rc(mic::fehler().unwrap_or_default()) }
            "mic_device$" => { keine_args(a, "MIC_DEVICE$")?; Value::str_rc(self.mikrofon.as_ref().map(|l| l.name.clone()).unwrap_or_default()) }
            "mic_rate" => { keine_args(a, "MIC_RATE")?; Value::Int(self.mikrofon.as_ref().map_or(0, |l| l.rate as i64)) }
            "mic_level" => { keine_args(a, "MIC_LEVEL")?; lauf!("MIC_LEVEL"); Value::Float(mic::spitze_nehmen()) }
            "mic_rms" => {
                keine_args(a, "MIC_RMS")?;
                // Die letzten 50 ms -- so lang wie ein Pegelmesser mittelt.
                let n = (lauf!("MIC_RMS").rate as usize / 20).max(1);
                Value::Float(mic::effektivwert(&mic::juengste(n)))
            }
            "mic_pitch" => {
                keine_args(a, "MIC_PITCH")?;
                let rate = lauf!("MIC_PITCH").rate;
                Value::Float(mic::tonhoehe(&mic::juengste(mic::tonhoehe_werte(rate)), rate))
            }
            "mic_fft" => {
                let arr = float_feld(a, "MIC_FFT")?;
                lauf!("MIC_FFT");
                let n = arr.borrow().cells.len();
                let mut tmp = vec![0.0f32; n];
                let proben = mic::juengste(crate::audio::FFT_N);
                crate::audio::baender_rechnen(&proben, &mut self.mic_baender, &mut self.mic_agc, &mut tmp);
                let mut b = arr.borrow_mut();
                for i in 0..n { b.cells.set(i, Value::Float(tmp[i] as f64)); }
                Value::Nil
            }
            "mic_wave" => {
                let arr = float_feld(a, "MIC_WAVE")?;
                lauf!("MIC_WAVE");
                let n = arr.borrow().cells.len();
                if n > mic::RING { return Err(format!("MIC_WAVE: hoechstens {} Werte, das Feld hat {}", mic::RING, n)); }
                let werte = mic::juengste(n);
                let mut b = arr.borrow_mut();
                for i in 0..n { b.cells.set(i, Value::Float(werte[i] as f64)); }
                Value::Nil
            }
            "mic_samples" => { keine_args(a, "MIC_SAMPLES")?; lauf!("MIC_SAMPLES"); Value::Int(mic::gesamt() as i64) }
            "mic_record_start" => {
                if a.len() > 1 { return Err(format!("MIC_RECORD_START: erwartet 0..1 Argumente, erhalten {}", a.len())); }
                let sek = if a.is_empty() { 600.0 } else { bi_num(a, 0, "MIC_RECORD_START")? };
                if !(sek > 0.0 && sek <= mic::MAX_SEKUNDEN) {
                    return Err("MIC_RECORD_START: max_sekunden muss 0 < x <= 3600 sein".into());
                }
                let rate = lauf!("MIC_RECORD_START").rate;
                mic::aufnahme_starten(rate, sek);
                Value::Nil
            }
            "mic_recording" => { keine_args(a, "MIC_RECORDING")?; Value::Bool(self.mikrofon.is_some() && mic::aufnahme_laeuft()) }
            "mic_record_stop" => {
                keine_args(a, "MIC_RECORD_STOP")?;
                let rate = lauf!("MIC_RECORD_STOP").rate;
                let werte = mic::aufnahme_nehmen()
                    .ok_or("MIC_RECORD_STOP: es laeuft keine Aufnahme -- erst MIC_RECORD_START")?;
                let pcm: Vec<f64> = if werte.is_empty() { vec![0.0] } else { werte.iter().map(|&w| w as f64).collect() };
                Value::Int(self.audio_mut()?.sound_aus_pcm(&pcm, rate))
            }
            "mic_record_save" => {
                let pfad = bi_str(a, 0, "MIC_RECORD_SAVE")?.to_string();
                if a.len() > 1 { return Err(format!("MIC_RECORD_SAVE: erwartet 1 Argument, erhalten {}", a.len())); }
                let rate = lauf!("MIC_RECORD_SAVE").rate;
                let werte = mic::aufnahme_nehmen()
                    .ok_or("MIC_RECORD_SAVE: es laeuft keine Aufnahme -- erst MIC_RECORD_START")?;
                std::fs::write(&pfad, mic::wav_bytes(&werte, rate))
                    .map_err(|e| format!("MIC_RECORD_SAVE: '{}' -- {}", pfad, e))?;
                Value::Int(werte.len() as i64)
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub(super) fn try_pins(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("gpio_") || name.starts_with("i2c_") || name.starts_with("spi_")) { return Ok(None); }
        use crate::pins;
        fn bytes(a: &[Value], i: usize, f: &str) -> R<Vec<u8>> {
            let roh: Vec<i64> = match a.get(i) {
                Some(Value::Buffer(b)) => return Ok(b.borrow().clone()),
                Some(Value::Array(arr)) => {
                    let arr = arr.borrow();
                    let mut out = Vec::with_capacity(arr.cells.len());
                    for k in 0..arr.cells.len() {
                        match arr.cells.get(k) {
                            Value::Int(x) => out.push(x),
                            Value::Float(v) if v.fract() == 0.0 => out.push(v as i64),
                            v => return Err(format!("{}: im Feld steht an Stelle {} keine ganze Zahl ({})", f, k, v.type_name())),
                        }
                    }
                    out
                }
                Some(v) => return Err(format!("{}: erwartet ein Feld aus Zahlen oder einen BUFFER, nicht {}", f, v.type_name())),
                None => return Err(format!("{}: die Bytes fehlen", f)),
            };
            pins::bytes_pruefen(f, &roh)
        }
        fn feld(v: Vec<u8>) -> Value {
            let n = v.len() as i64;
            let mut arr = crate::value::DhArray::new("integer".to_string(), vec![n], || Value::Int(0));
            for (k, x) in v.into_iter().enumerate() { arr.cells.set(k, Value::Int(x as i64)); }
            Value::Array(Rc::new(RefCell::new(arr)))
        }
        fn zahlen(v: Vec<i64>) -> Value {
            let n = v.len() as i64;
            let mut arr = crate::value::DhArray::new("integer".to_string(), vec![n], || Value::Int(0));
            for (k, x) in v.into_iter().enumerate() { arr.cells.set(k, Value::Int(x)); }
            Value::Array(Rc::new(RefCell::new(arr)))
        }
        // Wahrheitswert oder 0/1 -- ein Pin ist beides gewohnt.
        fn an_aus(a: &[Value], i: usize, f: &str) -> R<bool> {
            match a.get(i) {
                Some(Value::Bool(b)) => Ok(*b),
                Some(Value::Int(0)) => Ok(false),
                Some(Value::Int(1)) => Ok(true),
                Some(v) => Err(format!("{}: erwartet TRUE/FALSE oder 0/1, erhalten {}", f, v.type_name())),
                None => Err(format!("{}: fehlendes Argument {}", f, i + 1)),
            }
        }
        fn ziel(a: &[Value], i: usize, vorgabe: &str, f: &str) -> R<String> {
            match a.get(i) {
                None => Ok(vorgabe.to_string()),
                Some(Value::Int(n)) => Ok(n.to_string()),
                Some(Value::Str(s)) => Ok(s.to_string()),
                Some(v) => Err(format!("{}: erwartet eine Nummer oder einen Namen, nicht {}", f, v.type_name())),
            }
        }
        macro_rules! chip { ($f:expr) => {
            Self::handle_get_mut(&mut self.gpio_chips, bi_int(a, 0, $f)?, $f, "ungueltiger/geschlossener GPIO_CHIP")?
        }; }
        macro_rules! bus { ($f:expr) => {
            Self::handle_get_mut(&mut self.i2c_busse, bi_int(a, 0, $f)?, $f, "ungueltiger/geschlossener I2C_BUS")?
        }; }
        macro_rules! spi { ($f:expr) => {
            Self::handle_get_mut(&mut self.spi_geraete, bi_int(a, 0, $f)?, $f, "ungueltiges/geschlossenes SPI_DEVICE")?
        }; }
        let v = match name {
            "gpio_chips" => {
                let liste = pins::chips()?;
                let n = liste.len() as i64;
                let mut arr = crate::value::DhArray::new("string".to_string(), vec![n], || Value::str_rc(""));
                for (k, (name, _, _)) in liste.into_iter().enumerate() { arr.cells.set(k, Value::str_rc(name)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            "gpio_open" => {
                let z = ziel(a, 0, "", "GPIO_OPEN")?;
                self.gpio_chips.push(Some(pins::Chip::oeffnen(&z)?));
                Value::Int((self.gpio_chips.len() - 1) as i64)
            }
            "gpio_close" => { let i = bi_int(a, 0, "GPIO_CLOSE")? as usize; if let Some(s) = self.gpio_chips.get_mut(i) { *s = None; } Value::Nil }
            "gpio_label$" => Value::str_rc(chip!("GPIO_LABEL$").label.clone()),
            "gpio_count" => Value::Int(chip!("GPIO_COUNT").zahl as i64),
            "gpio_line_name$" => {
                const F: &str = "GPIO_LINE_NAME$"; let pin = bi_int(a, 1, F)?;
                Value::str_rc(chip!(F).linien_name(F, pin)?)
            }
            "gpio_find" => { const F: &str = "GPIO_FIND"; let n = bi_str(a, 1, F)?.to_string(); Value::Int(chip!(F).finden(&n)) }
            "gpio_mode" => {
                const F: &str = "GPIO_MODE";
                let pin = bi_int(a, 1, F)?;
                let r = pins::richtung_lesen(bi_str(a, 2, F)?, F)?;
                let w = if a.len() > 3 { pins::widerstand_lesen(bi_str(a, 3, F)?, F)? } else { pins::Widerstand::Unveraendert };
                chip!(F).modus(F, pin, r, w)?; Value::Nil
            }
            "gpio_write" => {
                const F: &str = "GPIO_WRITE";
                let pin = bi_int(a, 1, F)?; let wert = an_aus(a, 2, F)?;
                chip!(F).schreiben(F, pin, wert)?; Value::Nil
            }
            "gpio_read" => { const F: &str = "GPIO_READ"; let pin = bi_int(a, 1, F)?; Value::Bool(chip!(F).lesen(F, pin)?) }
            "gpio_watch" => {
                const F: &str = "GPIO_WATCH";
                let pin = bi_int(a, 1, F)?;
                let fl = pins::flanke_lesen(bi_str(a, 2, F)?, F)?;
                let ms = if a.len() > 3 { bi_int(a, 3, F)? } else { 0 };
                chip!(F).beobachten(F, pin, fl, ms)?; Value::Nil
            }
            "gpio_release" => { const F: &str = "GPIO_RELEASE"; let pin = bi_int(a, 1, F)?; chip!(F).freigeben(pin); Value::Nil }
            "gpio_next" => Value::Bool(chip!("GPIO_NEXT").naechstes()),
            "gpio_event_pin" => Value::Int(chip!("GPIO_EVENT_PIN").letztes().map(|e| e.pin as i64).unwrap_or(-1)),
            "gpio_event_rising" => Value::Bool(chip!("GPIO_EVENT_RISING").letztes().map(|e| e.steigend).unwrap_or(false)),
            "gpio_event_time" => Value::Float(chip!("GPIO_EVENT_TIME").letztes().map(|e| e.zeit_ns as f64 / 1e9).unwrap_or(0.0)),

            "i2c_open" => {
                let z = ziel(a, 0, "1", "I2C_OPEN")?;
                self.i2c_busse.push(Some(pins::Bus::oeffnen(&z)?));
                Value::Int((self.i2c_busse.len() - 1) as i64)
            }
            "i2c_close" => { let i = bi_int(a, 0, "I2C_CLOSE")? as usize; if let Some(s) = self.i2c_busse.get_mut(i) { *s = None; } Value::Nil }
            "i2c_write" => {
                const F: &str = "I2C_WRITE";
                let adr = bi_int(a, 1, F)?; let d = bytes(a, 2, F)?;
                bus!(F).schreiben(F, adr, &d)?; Value::Nil
            }
            "i2c_read" => {
                const F: &str = "I2C_READ";
                let adr = bi_int(a, 1, F)?; let reg = bi_int(a, 2, F)?; let n = bi_int(a, 3, F)?;
                feld(bus!(F).lesen(F, adr, reg, n)?)
            }
            "i2c_scan" => zahlen(bus!("I2C_SCAN").suchen()),

            "spi_open" => {
                const F: &str = "SPI_OPEN";
                let b = bi_int(a, 0, F)?; let cs = bi_int(a, 1, F)?;
                let hz = if a.len() > 2 { bi_int(a, 2, F)? } else { 1_000_000 };
                let m = if a.len() > 3 { bi_int(a, 3, F)? } else { 0 };
                self.spi_geraete.push(Some(pins::Spi::oeffnen(b, cs, hz, m)?));
                Value::Int((self.spi_geraete.len() - 1) as i64)
            }
            "spi_close" => { let i = bi_int(a, 0, "SPI_CLOSE")? as usize; if let Some(s) = self.spi_geraete.get_mut(i) { *s = None; } Value::Nil }
            "spi_transfer" => { const F: &str = "SPI_TRANSFER"; let d = bytes(a, 1, F)?; feld(spi!(F).tauschen(F, &d)?) }
            "spi_write" => { const F: &str = "SPI_WRITE"; let d = bytes(a, 1, F)?; spi!(F).tauschen(F, &d)?; Value::Nil }
            "spi_read" => {
                const F: &str = "SPI_READ"; let n = bi_int(a, 1, F)?;
                if !(1..=65536).contains(&n) { return Err(format!("{F}: Anzahl {n} ausserhalb 1..65536")); }
                feld(spi!(F).tauschen(F, &vec![0u8; n as usize])?)
            }
            "spi_speed" => { const F: &str = "SPI_SPEED"; let hz = bi_int(a, 1, F)?; spi!(F).takt_setzen(F, hz)?; Value::Nil }
            "spi_mode" => { const F: &str = "SPI_MODE"; let m = bi_int(a, 1, F)?; spi!(F).modus_setzen(F, m)?; Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul firmata: in jedem Bau (tcp:host:port), seriell mit Feature `serial`
    // ===================================================================
    pub(super) fn try_firmata(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("firmata_")) { return Ok(None); }
        self.try_firmata_impl(name, a)
    }
    pub(super) fn firmata_board(&mut self, idx: i64) -> R<&mut crate::firmata::Board> {
        Self::handle_get_mut(&mut self.firmata_boards, idx, "FIRMATA", "ungueltiges/geschlossenes FIRMATA_HANDLE")
    }
    pub(super) fn try_firmata_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::firmata;
        // Bytes fuer I2C: ein Feld aus Zahlen oder ein BUFFER.
        fn bytes_arg(a: &[Value], i: usize, fn_: &str) -> R<Vec<i64>> {
            match a.get(i) {
                Some(Value::Buffer(b)) => Ok(b.borrow().iter().map(|&x| x as i64).collect()),
                Some(Value::Array(arr)) => {
                    let arr = arr.borrow();
                    let mut out = Vec::with_capacity(arr.cells.len());
                    for k in 0..arr.cells.len() {
                        match arr.cells.get(k) {
                            Value::Int(x) => out.push(x),
                            Value::Float(f) if f.fract() == 0.0 => out.push(f as i64),
                            v => return Err(format!("{}: im Feld steht an Stelle {} keine ganze Zahl ({})", fn_, k, v.type_name())),
                        }
                    }
                    Ok(out)
                }
                Some(v) => Err(format!("{}: erwartet ein Feld aus Zahlen oder einen BUFFER, nicht {}", fn_, v.type_name())),
                None => Err(format!("{}: die Bytes fehlen", fn_)),
            }
        }
        fn int_opt(a: &[Value], i: usize, vorgabe: i64, fn_: &str) -> R<i64> {
            if a.len() > i { bi_int(a, i, fn_) } else { Ok(vorgabe) }
        }
        fn feld(v: Vec<u8>) -> Value {
            let n = v.len() as i64;
            let mut arr = crate::value::DhArray::new("integer".to_string(), vec![n], || Value::Int(0));
            for (k, x) in v.into_iter().enumerate() { arr.cells.set(k, Value::Int(x as i64)); }
            Value::Array(Rc::new(RefCell::new(arr)))
        }
        let v = match name {
            // Dieselbe Port-Liste wie SERIAL_PORTS() (serialport::available_ports) --
            // eigener Name, damit ein firmata-only-Script nicht extra IMPORT "serial" braucht.
            "firmata_ports" => Value::str_rc(firmata::ports()?),
            "firmata_open" => {
                let ziel = bi_str(a, 0, "FIRMATA_OPEN")?;
                // Ueber das Netz zaehlt keine Baudrate -- dort darf sie fehlen.
                let baud = if a.len() > 1 { bi_int(a, 1, "FIRMATA_OPEN")? } else { 57600 };
                let b = firmata::open(ziel, baud)?;
                self.firmata_boards.push(Some(b));
                Value::Int((self.firmata_boards.len() - 1) as i64)
            }
            "firmata_close" => { let i = bi_int(a, 0, "FIRMATA_CLOSE")? as usize; if let Some(s) = self.firmata_boards.get_mut(i) { *s = None; } Value::Nil }
            "firmata_is_open" => { let i = bi_int(a, 0, "FIRMATA_IS_OPEN")?; Value::Bool(self.firmata_boards.get(i as usize).map(|o| o.is_some()).unwrap_or(false)) }
            "firmata_pin_mode" => {
                let i = bi_int(a, 0, "FIRMATA_PIN_MODE")?; let pin = bi_int(a, 1, "FIRMATA_PIN_MODE")?; let mode = bi_int(a, 2, "FIRMATA_PIN_MODE")?;
                firmata::pin_mode(self.firmata_board(i)?, pin, mode)?; Value::Nil
            }
            "firmata_digital_write" => {
                let i = bi_int(a, 0, "FIRMATA_DIGITAL_WRITE")?; let pin = bi_int(a, 1, "FIRMATA_DIGITAL_WRITE")?; let val = bi_bool(a, 2, "FIRMATA_DIGITAL_WRITE")?;
                firmata::digital_write(self.firmata_board(i)?, pin, val)?; Value::Nil
            }
            "firmata_digital_read" => {
                let i = bi_int(a, 0, "FIRMATA_DIGITAL_READ")?; let pin = bi_int(a, 1, "FIRMATA_DIGITAL_READ")?;
                Value::Bool(firmata::digital_read(self.firmata_board(i)?, pin)?)
            }
            "firmata_analog_write" => {
                let i = bi_int(a, 0, "FIRMATA_ANALOG_WRITE")?; let pin = bi_int(a, 1, "FIRMATA_ANALOG_WRITE")?; let val = bi_int(a, 2, "FIRMATA_ANALOG_WRITE")?;
                firmata::analog_write(self.firmata_board(i)?, pin, val)?; Value::Nil
            }
            "firmata_analog_read" => {
                let i = bi_int(a, 0, "FIRMATA_ANALOG_READ")?; let ch = bi_int(a, 1, "FIRMATA_ANALOG_READ")?;
                Value::Int(firmata::analog_read(self.firmata_board(i)?, ch)?)
            }
            "firmata_update" => { let i = bi_int(a, 0, "FIRMATA_UPDATE")?; firmata::update(self.firmata_board(i)?)?; Value::Nil }
            "firmata_firmware$" => { let i = bi_int(a, 0, "FIRMATA_FIRMWARE$")?; Value::str_rc(firmata::firmware(self.firmata_board(i)?)?) }
            "firmata_servo_attach" => {
                const F: &str = "FIRMATA_SERVO_ATTACH";
                let i = bi_int(a, 0, F)?; let pin = bi_int(a, 1, F)?;
                let lo = int_opt(a, 2, 544, F)?; let hi = int_opt(a, 3, 2400, F)?;
                firmata::servo_attach(self.firmata_board(i)?, pin, lo, hi)?; Value::Nil
            }
            "firmata_servo_write" => {
                const F: &str = "FIRMATA_SERVO_WRITE";
                let i = bi_int(a, 0, F)?; let pin = bi_int(a, 1, F)?; let w = bi_int(a, 2, F)?;
                firmata::servo_write(self.firmata_board(i)?, pin, w)?; Value::Nil
            }
            "firmata_i2c_config" => {
                let i = bi_int(a, 0, "FIRMATA_I2C_CONFIG")?; let d = int_opt(a, 1, 0, "FIRMATA_I2C_CONFIG")?;
                firmata::i2c_config(self.firmata_board(i)?, d)?; Value::Nil
            }
            "firmata_i2c_write" => {
                const F: &str = "FIRMATA_I2C_WRITE";
                let i = bi_int(a, 0, F)?; let adr = bi_int(a, 1, F)?; let d = bytes_arg(a, 2, F)?;
                firmata::i2c_write(self.firmata_board(i)?, adr, &d)?; Value::Nil
            }
            "firmata_i2c_read" => {
                const F: &str = "FIRMATA_I2C_READ";
                let i = bi_int(a, 0, F)?; let adr = bi_int(a, 1, F)?; let reg = bi_int(a, 2, F)?; let n = bi_int(a, 3, F)?;
                feld(firmata::i2c_read(self.firmata_board(i)?, adr, reg, n)?)
            }
            "firmata_i2c_request" => {
                const F: &str = "FIRMATA_I2C_REQUEST";
                let i = bi_int(a, 0, F)?; let adr = bi_int(a, 1, F)?; let reg = bi_int(a, 2, F)?; let n = bi_int(a, 3, F)?;
                let dauernd = if a.len() > 4 { bi_bool(a, 4, F)? } else { false };
                firmata::i2c_request(self.firmata_board(i)?, F, adr, reg, n, dauernd)?; Value::Nil
            }
            "firmata_i2c_data" => {
                const F: &str = "FIRMATA_I2C_DATA";
                let i = bi_int(a, 0, F)?; let adr = bi_int(a, 1, F)?; let reg = int_opt(a, 2, -1, F)?;
                feld(firmata::i2c_data(self.firmata_board(i)?, adr, reg)?)
            }
            "firmata_i2c_stop" => {
                let i = bi_int(a, 0, "FIRMATA_I2C_STOP")?; let adr = bi_int(a, 1, "FIRMATA_I2C_STOP")?;
                firmata::i2c_stop(self.firmata_board(i)?, adr)?; Value::Nil
            }
            "firmata_stepper_config" => {
                const F: &str = "FIRMATA_STEPPER_CONFIG";
                let i = bi_int(a, 0, F)?; let nr = bi_int(a, 1, F)?; let art = bi_str(a, 2, F)?.to_string();
                let mut pins = Vec::new();
                for k in 3..a.len() { pins.push(bi_int(a, k, F)?); }
                firmata::stepper_config(self.firmata_board(i)?, nr, &art, &pins)?; Value::Nil
            }
            "firmata_stepper_move" | "firmata_stepper_to" => {
                let absolut = name == "firmata_stepper_to";
                let f = if absolut { "FIRMATA_STEPPER_TO" } else { "FIRMATA_STEPPER_MOVE" };
                let i = bi_int(a, 0, f)?; let nr = bi_int(a, 1, f)?; let s = bi_int(a, 2, f)?;
                firmata::stepper_move(self.firmata_board(i)?, nr, s, absolut)?; Value::Nil
            }
            "firmata_stepper_zero" => {
                let i = bi_int(a, 0, "FIRMATA_STEPPER_ZERO")?; let nr = bi_int(a, 1, "FIRMATA_STEPPER_ZERO")?;
                firmata::stepper_zero(self.firmata_board(i)?, nr)?; Value::Nil
            }
            "firmata_stepper_stop" => {
                let i = bi_int(a, 0, "FIRMATA_STEPPER_STOP")?; let nr = bi_int(a, 1, "FIRMATA_STEPPER_STOP")?;
                firmata::stepper_stop(self.firmata_board(i)?, nr)?; Value::Nil
            }
            "firmata_stepper_speed" | "firmata_stepper_accel" => {
                let besch = name == "firmata_stepper_accel";
                let f = if besch { "FIRMATA_STEPPER_ACCEL" } else { "FIRMATA_STEPPER_SPEED" };
                let i = bi_int(a, 0, f)?; let nr = bi_int(a, 1, f)?; let w = bi_num(a, 2, f)?;
                firmata::stepper_tempo(self.firmata_board(i)?, nr, w, besch)?; Value::Nil
            }
            "firmata_stepper_position" => {
                let i = bi_int(a, 0, "FIRMATA_STEPPER_POSITION")?; let nr = bi_int(a, 1, "FIRMATA_STEPPER_POSITION")?;
                Value::Int(firmata::stepper_position(self.firmata_board(i)?, nr)?)
            }
            "firmata_stepper_running" => {
                let i = bi_int(a, 0, "FIRMATA_STEPPER_RUNNING")?; let nr = bi_int(a, 1, "FIRMATA_STEPPER_RUNNING")?;
                Value::Bool(firmata::stepper_running(self.firmata_board(i)?, nr)?)
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul usb (Feature `usb`)
    // ===================================================================
    pub(super) fn try_usb(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("usb_")) { return Ok(None); }
        #[cfg(feature = "usb")]
        { return self.try_usb_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }
    #[cfg(feature = "usb")]
    pub(super) fn usb_dev(&self, idx: i64) -> R<&hidapi::HidDevice> {
        Self::handle_get(&self.usb_devs, idx, "USB", "ungueltiges/geschlossenes USB_HANDLE")
    }
    #[cfg(feature = "usb")]
    pub(super) fn try_usb_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::usb;
        let v = match name {
            "usb_list" => Value::str_rc(usb::list()?),
            "usb_open" => { let d = usb::open(bi_int(a, 0, "USB_OPEN")?, bi_int(a, 1, "USB_OPEN")?)?; self.usb_devs.push(Some(d)); Value::Int((self.usb_devs.len() - 1) as i64) }
            "usb_open_path" => { let d = usb::open_path(bi_str(a, 0, "USB_OPEN_PATH")?)?; self.usb_devs.push(Some(d)); Value::Int((self.usb_devs.len() - 1) as i64) }
            "usb_close" => { let i = bi_int(a, 0, "USB_CLOSE")? as usize; if let Some(s) = self.usb_devs.get_mut(i) { *s = None; } Value::Nil }
            "usb_write" => { let i = bi_int(a, 0, "USB_WRITE")?; let s = bi_str(a, 1, "USB_WRITE")?.to_string(); Value::Int(usb::write(self.usb_dev(i)?, &s)?) }
            "usb_read" => { let i = bi_int(a, 0, "USB_READ")?; let n = bi_int(a, 1, "USB_READ")?; let t = bi_int(a, 2, "USB_READ")?; Value::str_rc(usb::read(self.usb_dev(i)?, n, t)?) }
            "usb_product" => Value::str_rc(usb::product(self.usb_dev(bi_int(a, 0, "USB_PRODUCT")?)?)),
            "usb_manufacturer" => Value::str_rc(usb::manufacturer(self.usb_dev(bi_int(a, 0, "USB_MANUFACTURER")?)?)),
            "usb_serial" => Value::str_rc(usb::serial(self.usb_dev(bi_int(a, 0, "USB_SERIAL")?)?)),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul wifi (Feature `wifi`, Windows-only)
    // ===================================================================
    pub(super) fn try_wifi(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("wifi_")) { return Ok(None); }
        #[cfg(feature = "wifi")]
        { return self.try_wifi_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }

    #[cfg(feature = "wifi")]
    pub(super) fn try_wifi_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::wifi;
        let v = match name {
            "wifi_available" => Value::Bool(wifi::available()),
            "wifi_current" => Value::str_rc(wifi::current()?),
            "wifi_signal" => Value::Int(wifi::signal()?),
            "wifi_scan" => Value::str_rc(wifi::scan()?),
            "wifi_connect" => Value::Bool(wifi::connect(bi_str(a, 0, "WIFI_CONNECT")?, bi_str(a, 1, "WIFI_CONNECT")?)?),
            "wifi_disconnect" => Value::Bool(wifi::disconnect()?),
            "wifi_profiles" => Value::str_rc(wifi::profiles()?),
            "wifi_delete_profile" => Value::Bool(wifi::delete_profile(bi_str(a, 0, "WIFI_DELETE_PROFILE")?)?),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    // ===================================================================
    // Modul bt (Feature `bt`, BLE async)
    // ===================================================================
    pub(super) fn try_bt(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("bt_")) { return Ok(None); }
        #[cfg(feature = "bt")]
        { return self.try_bt_impl(name, a); }
        #[allow(unreachable_code)]
        { let _ = (name, a); Ok(None) }
    }
    #[cfg(feature = "bt")]
    pub(super) fn bt_periph(&self, idx: i64) -> R<&btleplug::platform::Peripheral> {
        Self::handle_get(&self.bt_periphs, idx, "BT", "ungueltiges/getrenntes BT_HANDLE")
    }
    #[cfg(feature = "bt")]
    pub(super) fn try_bt_impl(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        use crate::bt;
        let v = match name {
            "bt_scan" => Value::str_rc(bt::scan(bi_num(a, 0, "BT_SCAN")?)?),
            "bt_connect" => { let p = bt::connect(bi_str(a, 0, "BT_CONNECT")?)?; self.bt_periphs.push(Some(p)); Value::Int((self.bt_periphs.len() - 1) as i64) }
            "bt_disconnect" => { let i = bi_int(a, 0, "BT_DISCONNECT")?; bt::disconnect(self.bt_periph(i)?)?; if let Some(s) = self.bt_periphs.get_mut(i as usize) { *s = None; } Value::Nil }
            "bt_is_connected" => { let i = bi_int(a, 0, "BT_IS_CONNECTED")?; Value::Bool(match self.bt_periphs.get(i as usize).and_then(|o| o.as_ref()) { Some(p) => bt::is_connected(p), None => false }) }
            "bt_services" => { let i = bi_int(a, 0, "BT_SERVICES")?; Value::str_rc(bt::services(self.bt_periph(i)?)?) }
            "bt_characteristics" => { let i = bi_int(a, 0, "BT_CHARACTERISTICS")?; let s = bi_str(a, 1, "BT_CHARACTERISTICS")?.to_string(); Value::str_rc(bt::characteristics(self.bt_periph(i)?, &s)?) }
            "bt_read" => { let i = bi_int(a, 0, "BT_READ")?; let c = bi_str(a, 1, "BT_READ")?.to_string(); Value::str_rc(bt::read(self.bt_periph(i)?, &c)?) }
            "bt_write" => { let i = bi_int(a, 0, "BT_WRITE")?; let c = bi_str(a, 1, "BT_WRITE")?.to_string(); let d = bi_str(a, 2, "BT_WRITE")?.to_string(); bt::write(self.bt_periph(i)?, &c, &d)?; Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }
}
