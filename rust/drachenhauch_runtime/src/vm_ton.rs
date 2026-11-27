//! Die Ton-Befehle (`try_ton`): Klaenge und Musik, das audio-Modul (Busse,
//! Effekte, Uhr, Sampler, Modulatoren, raeumlicher Ton) und die
//! Sprachausgabe. Dieselbe Familie wie die Grafik (`try_graphics` fragt hier
//! nach) -- bis 2026.53 standen die Arme mitten in vm_grafik.rs (Bericht D2).

use super::*;
use super::grafik::{easing_name, gb, gi, gs, leeres_feld, need_f};

impl<'p> Vm<'p> {
    pub(super) fn try_ton(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        let r = match name {
            // --- Audio (Core: SFX + Stream-Musik) ---
            "loadsound" => {
                let path = gs(a, 0, "LOADSOUND")?.to_string();
                Value::Int(self.audio_mut()?.load_sound(&path)?)
            }
            "sample_load" => {
                // SAMPLE_LOAD(path$) -> SAMPLE (Amiga-Stil-Sampler)
                let path = gs(a, 0, "SAMPLE_LOAD")?.to_string();
                Value::Int(self.audio_mut()?.sample_load(&path)?)
            }
            // Die Loop-Region in Sekunden wie SAMPLE_LEN (SAMPLE_SET_LOOP zaehlt
            // Frames -- die kennt ein Programm nur, wenn es die Abtastrate kennt).
            "sample_set_loop_time" => {
                let f = "SAMPLE_SET_LOOP_TIME";
                let idx = gi(a, 0, f)?;
                let (von, bis) = (need_f(a, 1, f)?, need_f(a, 2, f)?);
                let art = if a.len() >= 4 { Some(gs(a, 3, f)?.to_string()) } else { None };
                let au = self.audio_mut()?;
                let sr = au.sample_rate(idx, f)?;
                au.sample_set_loop(idx, (von * sr).round() as i64, (bis * sr).round() as i64, art.as_deref(), f)?;
                Value::Nil
            }
            "sample_set_loop" => {
                // SAMPLE_SET_LOOP(sample, start, end[, art$]) -- Loop-Region in
                // Frames; art$ = "vorwaerts" (Vorgabe) oder "pingpong".
                let idx = gi(a, 0, "SAMPLE_SET_LOOP")?;
                let start = gi(a, 1, "SAMPLE_SET_LOOP")?;
                let end = gi(a, 2, "SAMPLE_SET_LOOP")?;
                let art = if a.len() >= 4 { Some(gs(a, 3, "SAMPLE_SET_LOOP")?.to_string()) } else { None };
                self.audio_mut()?.sample_set_loop(idx, start, end, art.as_deref(), "SAMPLE_SET_LOOP")?;
                Value::Nil
            }
            "sample_from_buffer" => {
                // SAMPLE_FROM_BUFFER(puffer, abtastrate) -> SAMPLE (16-Bit-PCM,
                // little endian, mono -- so liegen die Samples im Tracker-Format)
                let bytes = match a.first() {
                    Some(Value::Buffer(b)) => b.borrow().clone(),
                    Some(v) => return Err(format!(
                        "SAMPLE_FROM_BUFFER: erwartet BUFFER, erhalten {}", v.type_name())),
                    None => return Err(crate::vm::falsche_argumente("SAMPLE_FROM_BUFFER", a.len())),
                };
                let sr = gi(a, 1, "SAMPLE_FROM_BUFFER")?;
                Value::Int(self.audio_mut()?.sample_from_pcm16(&bytes, sr)?)
            }
            "sample_from_sound" => {
                // SAMPLE_FROM_SOUND(sound) -> SAMPLE (Kanaele gemittelt)
                let idx = gi(a, 0, "SAMPLE_FROM_SOUND")?;
                Value::Int(self.audio_mut()?.sample_from_sound(idx)?)
            }
            "sample_note" => {
                // SAMPLE_NOTE(sample, halbtoene, dauer_ms, attack, decay, sustain,
                //   release, vol[, slide_halbtoene]) -> SOUND
                let f = "SAMPLE_NOTE";
                if a.len() < 8 || a.len() > 9 {
                    return Err(crate::vm::falsche_argumente("SAMPLE_NOTE", a.len()));
                }
                let idx = gi(a, 0, f)?;
                let semis = need_f(a, 1, f)?;
                let dauer = gi(a, 2, f)?;
                let atk = gi(a, 3, f)?;
                let dec = gi(a, 4, f)?;
                let sus = need_f(a, 5, f)?;
                let rel = gi(a, 6, f)?;
                let vol = need_f(a, 7, f)?;
                let sl = if a.len() >= 9 { need_f(a, 8, f)? } else { 0.0 };
                Value::Int(self.audio_mut()?.sample_note(idx, semis, dauer, atk, dec, sus, rel, vol, sl)?)
            }
            "sample_free" => { let i = gi(a, 0, "SAMPLE_FREE")?; self.audio_mut()?.sample_free(i)?; Value::Nil }
            "sample_len" => {
                // SAMPLE_LEN(sample) -> Sekunden bei Originaltonhoehe
                let idx = gi(a, 0, "SAMPLE_LEN")?;
                Value::Float(self.audio_mut()?.sample_len(idx)?)
            }
            "sample_play" => {
                // SAMPLE_PLAY(sample, halbtoene, vol[, dur_ms]) -> AUDIO_CHANNEL
                let idx = gi(a, 0, "SAMPLE_PLAY")?;
                let semis = need_f(a, 1, "SAMPLE_PLAY")?;
                // Review-Fund: `semis`/`dur_ms` waren voellig unbeschraenkt.
                // resample() leitet aus `semis` ein Pitch-Verhaeltnis
                // `2^(semis/12)` ab und daraus `out_len = n/ratio` -- ein
                // stark negativer Wert (z.B. -700) macht `ratio` astronomisch
                // klein, `out_len` ueberlief/saettigte auf usize::MAX, und
                // `Vec::with_capacity(out_len)` paniked mit einem
                // Kapazitaets-Ueberlauf (nicht fangbar per TRY/CATCH) --
                // direkt aus einem einzigen, gewoehnlich aussehenden
                // SAMPLE_PLAY-Aufruf heraus. +/-120 Halbtoene (10 Oktaven)
                // ist bereits weit jenseits jedes musikalischen Sinns.
                if !(-120.0..=120.0).contains(&semis) {
                    return Err("SAMPLE_PLAY: Halbtoene muss -120..120 sein".into());
                }
                let vol = if a.len() >= 3 { need_f(a, 2, "SAMPLE_PLAY")? } else { 1.0 };
                let dur = if a.len() >= 4 { gi(a, 3, "SAMPLE_PLAY")? } else { 0 };
                if !(0..=600_000).contains(&dur) {
                    return Err("SAMPLE_PLAY: dur_ms muss 0..600000 sein".into());
                }
                Value::Int(self.audio_mut()?.sample_play(idx, semis, vol, dur)?)
            }
            "audio_lofi" => {
                // AUDIO_LOFI(an[, bits[, cutoff_hz]]) -- Paula/Amiga-Lo-Fi.
                // Argument-Pruefung VOR der Audio-Initialisierung (golden-testbar).
                let on = match a.first() {
                    Some(_) => crate::argument::a_schalter(a, 0, "AUDIO_LOFI")?,
                    None => return Err(crate::vm::falsche_argumente("AUDIO_LOFI", a.len())),
                };
                let bits = if a.len() >= 2 {
                    let b = gi(a, 1, "AUDIO_LOFI")?;
                    if !(1..=16).contains(&b) {
                        return Err("AUDIO_LOFI: bits muss 1..16 sein".into());
                    }
                    b as u32
                } else { 8 };
                let cutoff = if a.len() >= 3 {
                    let c = need_f(a, 2, "AUDIO_LOFI")?;
                    if c < 0.0 { return Err("AUDIO_LOFI: cutoff_hz muss >= 0 sein".into()); }
                    c
                } else { 3300.0 };
                self.audio_mut()?.set_lofi(on, bits, cutoff);
                Value::Nil
            }
            "audio_bus_volume" => {
                // AUDIO_BUS_VOLUME(bus$, vol) -- Master pro Bus (sfx/music/master).
                // Bus-Name VOR der Audio-Initialisierung pruefen (golden-testbar).
                let bus = gs(a, 0, "AUDIO_BUS_VOLUME")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master" | "speech") {
                    return Err(format!("AUDIO_BUS_VOLUME: unbekannter Bus '{}' (sfx, music, master, speech)", bus));
                }
                let v = need_f(a, 1, "AUDIO_BUS_VOLUME")?;
                self.audio_mut()?.set_bus_volume(&bus, v)?;
                Value::Nil
            }
            "audio_bus_get_volume" => {
                let bus = gs(a, 0, "AUDIO_BUS_GET_VOLUME")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master" | "speech") {
                    return Err(format!("AUDIO_BUS_GET_VOLUME: unbekannter Bus '{}' (sfx, music, master, speech)", bus));
                }
                Value::Float(round_audio(self.audio_mut()?.get_bus_volume(&bus)?))
            }
            "audio_filter" => {
                // AUDIO_FILTER(bus$, cutoff_hz[, resonance]) -- Tiefpass.
                let bus = gs(a, 0, "AUDIO_FILTER")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_FILTER: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let cutoff = need_f(a, 1, "AUDIO_FILTER")?;
                let res = if a.len() >= 3 { need_f(a, 2, "AUDIO_FILTER")? } else { 0.0 };
                self.audio_mut()?.set_filter(&bus, cutoff, res)?;
                Value::Nil
            }
            "audio_reverb" => {
                // AUDIO_REVERB(bus$, mix[, feedback[, damping]]) -- Hall.
                let bus = gs(a, 0, "AUDIO_REVERB")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_REVERB: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let mix = need_f(a, 1, "AUDIO_REVERB")?;
                let fb = if a.len() >= 3 { need_f(a, 2, "AUDIO_REVERB")? } else { 0.9 };
                let damp = if a.len() >= 4 { need_f(a, 3, "AUDIO_REVERB")? } else { 0.1 };
                self.audio_mut()?.set_reverb(&bus, mix, fb, damp)?;
                Value::Nil
            }
            "audio_delay" => {
                // AUDIO_DELAY(bus$, mix[, feedback]) -- Echo (300 ms).
                let bus = gs(a, 0, "AUDIO_DELAY")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_DELAY: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let mix = need_f(a, 1, "AUDIO_DELAY")?;
                let fb = if a.len() >= 3 { need_f(a, 2, "AUDIO_DELAY")? } else { 0.5 };
                // Fehlend heisst die Vorgabe -- wie beim feedback (bis 2026.53
                // blieb die alte Zeit stehen).
                let time_ms = if a.len() >= 4 { gi(a, 3, "AUDIO_DELAY")? } else { 300 };
                if time_ms <= 0 { return Err(format!("AUDIO_DELAY: time_ms muss > 0 sein, erhalten {}", time_ms)); }
                self.audio_mut()?.set_delay(&bus, mix, fb, time_ms)?;
                Value::Nil
            }
            "audio_distortion" => {
                // AUDIO_DISTORTION(bus$, amount[, mix]) -- Overdrive/Fuzz.
                let bus = gs(a, 0, "AUDIO_DISTORTION")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_DISTORTION: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let amount = need_f(a, 1, "AUDIO_DISTORTION")?;
                let mix = if a.len() >= 3 { need_f(a, 2, "AUDIO_DISTORTION")? } else { 1.0 };
                self.audio_mut()?.set_distortion(&bus, amount, mix)?;
                Value::Nil
            }
            "audio_compressor" => {
                // AUDIO_COMPRESSOR(bus$, threshold_db, ratio[, makeup_db]).
                let bus = gs(a, 0, "AUDIO_COMPRESSOR")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_COMPRESSOR: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let thresh = need_f(a, 1, "AUDIO_COMPRESSOR")?;
                let ratio = need_f(a, 2, "AUDIO_COMPRESSOR")?;
                let makeup = if a.len() >= 4 { need_f(a, 3, "AUDIO_COMPRESSOR")? } else { 0.0 };
                self.audio_mut()?.set_compressor(&bus, thresh, ratio, makeup)?;
                Value::Nil
            }
            "audio_eq" => {
                // AUDIO_EQ(bus$, freq_hz, gain_db[, q]) -- parametrischer Bell-EQ.
                let bus = gs(a, 0, "AUDIO_EQ")?.to_lowercase();
                if !matches!(bus.as_str(), "sfx" | "music" | "master") {
                    return Err(format!("AUDIO_EQ: unbekannter Bus '{}' (sfx, music, master)", bus));
                }
                let freq = need_f(a, 1, "AUDIO_EQ")?;
                let gain = need_f(a, 2, "AUDIO_EQ")?;
                let q = if a.len() >= 4 { need_f(a, 3, "AUDIO_EQ")? } else { 1.0 };
                self.audio_mut()?.set_eq(&bus, freq, gain, q)?;
                Value::Nil
            }
            "playsound" => {
                // PLAYSOUND(sound[, loops, volume]): loops wie bei AUDIO_PLAY --
                // 0 einmal, -1 endlos, n Wiederholungen. Bis 2026.53 ging es
                // verloren ("raylib-Sounds loopen nicht" -- Kira kann es).
                let idx = gi(a, 0, "PLAYSOUND")?;
                let loops = if a.len() >= 2 { gi(a, 1, "PLAYSOUND")? } else { 0 };
                let vol = if a.len() >= 3 { need_f(a, 2, "PLAYSOUND")? } else { 1.0 };
                self.audio_mut()?.play_sound(idx, loops, vol)?;
                Value::Nil
            }
            "stopsound" => {
                let idx = gi(a, 0, "STOPSOUND")?;
                self.audio_mut()?.stop_sound(idx)?;
                Value::Nil
            }
            "unloadsound" => {
                // UNLOADSOUND(sound) -- stoppt + gibt den Frame-Puffer frei.
                // Index bleibt als Tombstone gueltig (kein Recycling).
                let idx = gi(a, 0, "UNLOADSOUND")?;
                self.audio_mut()?.unload_sound(idx)?;
                Value::Nil
            }
            "audio_sound_count" => {
                // AUDIO_SOUND_COUNT() -> Anzahl lebender Sound-Slots (Diagnose).
                Value::Int(self.audio_mut()?.sound_count())
            }
            "playmusic" => {
                // PLAYMUSIC(pfad$[, loops, volume]): n > 0 sind so viele
                // Wiederholungen wie bei AUDIO_MUSIC_PLAY; 0 und -1 endlos --
                // 0 wie bisher, weil Programme PLAYMUSIC(datei, 0, v) fuer
                // Hintergrundmusik schreiben (bis 2026.53 lief sie immer endlos).
                let path = gs(a, 0, "PLAYMUSIC")?.to_string();
                let loops = if a.len() >= 2 { gi(a, 1, "PLAYMUSIC")? } else { -1 };
                let vol = if a.len() >= 3 { need_f(a, 2, "PLAYMUSIC")? } else { 1.0 };
                self.audio_mut()?.play_music(&path, if loops > 0 { loops } else { -1 }, vol)?;
                Value::Nil
            }
            "stopmusic" => { self.audio_mut()?.stop_music(); Value::Nil }
            "audio_fft" => {
                // AUDIO_FFT(arr): fuellt ein 1D ARRAY OF FLOAT mit B Band-Pegeln
                // (0..1) aus dem aktuell hoerbaren Audio (echte FFT).
                let arr = match a.first() {
                    Some(Value::Array(x)) => x.clone(),
                    _ => return Err("AUDIO_FFT: erwartet ARRAY OF FLOAT".into()),
                };
                let n = {
                    let b = arr.borrow();
                    if b.element_type != "float" || b.dims.len() != 1 {
                        return Err("AUDIO_FFT: erwartet 1D ARRAY OF FLOAT".into());
                    }
                    if b.cells.len() == 0 { return Err(leeres_feld("AUDIO_FFT")); }
                    b.cells.len()
                };
                let mut tmp = vec![0.0f32; n];
                self.audio_mut()?.fft_bands(&mut tmp);
                let mut b = arr.borrow_mut();
                for i in 0..n { b.cells.set(i, Value::Float(tmp[i] as f64)); }
                Value::Nil
            }

            // --- erweitertes audio-Modul (AUDIO_*) ---
            "audio_init" => { self.audio_mut()?; Value::Nil }
            "audio_set_num_channels" => { let n = gi(a, 0, "AUDIO_SET_NUM_CHANNELS")?; if n < 0 { return Err("AUDIO_SET_NUM_CHANNELS: n muss >= 0 sein".into()); } self.audio_mut()?.set_num_channels(n); Value::Nil }
            "audio_num_channels" => Value::Int(self.audio_mut()?.get_num_channels()),
            "audio_busy_channels" => Value::Int(self.audio_mut()?.busy_channels()),
            "audio_pause_all" => { self.audio_mut()?.pause_all(); Value::Nil }
            "audio_resume_all" => { self.audio_mut()?.resume_all(); Value::Nil }
            "audio_stop_all" => { self.audio_mut()?.stop_all(); Value::Nil }
            "audio_play" => {
                // AUDIO_PLAY(sound[, loops[, volume[, fade_in_ms[, easing$]]]]) -- loops=0 einmal (Default), -1 endlos
                let idx = gi(a, 0, "AUDIO_PLAY")?;
                let loops = if a.len() >= 2 { gi(a, 1, "AUDIO_PLAY")? } else { 0 };
                let vol = if a.len() >= 3 { need_f(a, 2, "AUDIO_PLAY")? } else { 1.0 };
                let fade = if a.len() >= 4 { gi(a, 3, "AUDIO_PLAY")? } else { 0 };
                if fade < 0 { return Err("AUDIO_PLAY: fade_in_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 4, "AUDIO_PLAY")?;
                Value::Int(self.audio_mut()?.ch_play(idx, loops, vol, fade, &easing)?)
            }
            "audio_pause" => { let i = gi(a, 0, "AUDIO_PAUSE")?; self.audio_mut()?.ch_pause(i)?; Value::Nil }
            "audio_resume" => { let i = gi(a, 0, "AUDIO_RESUME")?; self.audio_mut()?.ch_resume(i)?; Value::Nil }
            "audio_stop" => {
                // AUDIO_STOP(ch[, fade_out_ms[, easing$]])
                let i = gi(a, 0, "AUDIO_STOP")?;
                let fade = if a.len() >= 2 { gi(a, 1, "AUDIO_STOP")? } else { 0 };
                if fade < 0 { return Err("AUDIO_STOP: fade_out_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 2, "AUDIO_STOP")?;
                self.audio_mut()?.ch_stop(i, fade, &easing)?; Value::Nil
            }
            // AUDIO_IS_SILENT(): laeuft der Ton stumm (DHRT_OHNE_AUDIO)? Der
            // Mix wird dann gerechnet, aber nicht ausgegeben.
            "audio_is_silent" => Value::Bool(self.audio_mut()?.stumm),
            "audio_is_playing" => { let i = gi(a, 0, "AUDIO_IS_PLAYING")?; Value::Bool(self.audio_mut()?.ch_is_playing(i)?) }
            "audio_is_waiting" => { let i = gi(a, 0, "AUDIO_IS_WAITING")?; Value::Bool(self.audio_mut()?.ch_is_waiting(i)?) }
            "audio_record_start" => {
                if a.len() > 1 { return Err(format!("AUDIO_RECORD_START: erwartet 0 bis 1 Argumente, erhalten {}", a.len())); }
                let m = if a.is_empty() { 600.0 } else { need_f(a, 0, "AUDIO_RECORD_START")? };
                self.audio_mut()?.record_start(m)?; Value::Nil
            }
            "audio_record_stop" => { Value::Int(self.audio_mut()?.record_stop()?) }
            "audio_recording" => { Value::Bool(self.audio_mut()?.recording()) }
            "audio_peak" => { Value::Float(round_audio(self.audio_mut()?.peak())) }
            // AUDIO_SET_VOLUME ist der neue Name (namen.rs).
            "audio_volume" => { let i = gi(a, 0, "AUDIO_VOLUME")?; let v = need_f(a, 1, "AUDIO_VOLUME")?; self.audio_mut()?.ch_set_volume(i, v)?; Value::Nil }
            "audio_get_volume" => { let i = gi(a, 0, "AUDIO_GET_VOLUME")?; Value::Float(round_audio(self.audio_mut()?.ch_get_volume(i)?)) }
            "audio_pan" => { let i = gi(a, 0, "AUDIO_PAN")?; let l = need_f(a, 1, "AUDIO_PAN")?; let r = need_f(a, 2, "AUDIO_PAN")?; self.audio_mut()?.ch_pan(i, l, r)?; Value::Nil }
            "audio_pitch" => {
                let i = gi(a, 0, "AUDIO_PITCH")?;
                let f = need_f(a, 1, "AUDIO_PITCH")?;
                // Dieselbe Grenze wie AUDIO_MUSIC_PITCH: 0 < f <= 64 (bis 2026.53
                // hier ohne Obergrenze, dort still auf 64 geklemmt).
                if !(f > 0.0 && f <= 64.0) { return Err(format!("AUDIO_PITCH: faktor muss > 0 und <= 64 sein, erhalten {}", f)); }
                self.audio_mut()?.ch_pitch(i, f)?; Value::Nil
            }
            "audio_music_pitch" => {
                let f = need_f(a, 0, "AUDIO_MUSIC_PITCH")?;
                if !(f > 0.0 && f <= 64.0) { return Err(format!("AUDIO_MUSIC_PITCH: faktor muss > 0 und <= 64 sein, erhalten {}", f)); }
                self.audio_mut()?.music_set_pitch(f); Value::Nil
            }
            "audio_music_get_pitch" => Value::Float(self.audio_mut()?.music_get_pitch()),
            "audio_pan_pos" => { let i = gi(a, 0, "AUDIO_PAN_POS")?; let p = need_f(a, 1, "AUDIO_PAN_POS")?; self.audio_mut()?.ch_pan_pos(i, p, "AUDIO_PAN_POS")?; Value::Nil }
            // Pan in der Skala -1 (links) .. 0 (Mitte) .. 1 (rechts) wie
            // AUDIO_BUS_PAN und AUDIO_SOUND_MIX. AUDIO_PAN_POS/_SLIDE zaehlen
            // 0..1 und bleiben dabei (Bericht A8.1; eine Skala, neue Befehle).
            "audio_set_pan" => {
                let f = "AUDIO_SET_PAN";
                let i = gi(a, 0, f)?;
                let p = need_f(a, 1, f)?;
                if !(-1.0..=1.0).contains(&p) { return Err(format!("{}: pan ist -1..1, erhalten {}", f, p)); }
                self.audio_mut()?.ch_pan_pos(i, (p + 1.0) / 2.0, f)?; Value::Nil
            }
            "audio_set_pan_slide" => {
                let f = "AUDIO_SET_PAN_SLIDE";
                let i = gi(a, 0, f)?;
                let (von, nach) = (need_f(a, 1, f)?, need_f(a, 2, f)?);
                for p in [von, nach] {
                    if !(-1.0..=1.0).contains(&p) { return Err(format!("{}: pan ist -1..1, erhalten {}", f, p)); }
                }
                let dauer = gi(a, 3, f)?;
                if dauer <= 0 { return Err(format!("{}: dauer_ms muss > 0 sein", f)); }
                let easing = easing_name(a, 4, f)?;
                self.audio_mut()?.ch_pan_slide(i, (von + 1.0) / 2.0, (nach + 1.0) / 2.0, dauer, &easing, f)?; Value::Nil
            }
            "audio_pan_slide" => {
                // AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms[, easing$]) -- Positionen 0=links..1=rechts
                let i = gi(a, 0, "AUDIO_PAN_SLIDE")?;
                let von = need_f(a, 1, "AUDIO_PAN_SLIDE")?;
                let nach = need_f(a, 2, "AUDIO_PAN_SLIDE")?;
                let dauer = gi(a, 3, "AUDIO_PAN_SLIDE")?;
                if dauer <= 0 { return Err("AUDIO_PAN_SLIDE: dauer_ms muss > 0 sein".into()); }
                let easing = easing_name(a, 4, "AUDIO_PAN_SLIDE")?;
                self.audio_mut()?.ch_pan_slide(i, von, nach, dauer, &easing, "AUDIO_PAN_SLIDE")?; Value::Nil
            }
            // Dieselbe Pendelbewegung mit der Periode in ms (Regel 9: Zeitspannen
            // in ms; AUDIO_AUTOPAN bleibt bei Sekunden).
            "audio_autopan_ms" => {
                let i = gi(a, 0, "AUDIO_AUTOPAN_MS")?;
                let periode = need_f(a, 1, "AUDIO_AUTOPAN_MS")? / 1000.0;
                let tiefe = if a.len() >= 3 { need_f(a, 2, "AUDIO_AUTOPAN_MS")? } else { 1.0 };
                self.audio_mut()?.ch_autopan(i, periode, tiefe, "AUDIO_AUTOPAN_MS")?; Value::Nil
            }
            "audio_autopan" => {
                // AUDIO_AUTOPAN(ch, periode_s[, tiefe]) -- periode_s <= 0 = aus
                let i = gi(a, 0, "AUDIO_AUTOPAN")?;
                let periode = need_f(a, 1, "AUDIO_AUTOPAN")?;
                let tiefe = if a.len() >= 3 { need_f(a, 2, "AUDIO_AUTOPAN")? } else { 1.0 };
                self.audio_mut()?.ch_autopan(i, periode, tiefe, "AUDIO_AUTOPAN")?; Value::Nil
            }
            "audio_tone" => {
                let freq = need_f(a, 0, "AUDIO_TONE")?;
                let dur = gi(a, 1, "AUDIO_TONE")?;
                // Review-Fund: keine Obergrenze -- `dur` geht direkt in
                // `vec![0.0f64; n]` (n = Samples fuer die Dauer), ein absurd
                // grosser Wert (z.B. eine fehlerhafte Berechnung) fuehrte zu
                // einer Mehrere-GB-Allokation / Allocator-Abort statt eines
                // DH-Fehlers.
                if !(0..=600_000).contains(&dur) {
                    return Err("AUDIO_TONE: dur_ms muss 0..600000 sein".into());
                }
                let wf = if a.len() >= 3 { gs(a, 2, "AUDIO_TONE")?.to_string() } else { "sine".to_string() };
                let vol = if a.len() >= 4 { need_f(a, 3, "AUDIO_TONE")? } else { 1.0 };
                Value::Int(self.audio_mut()?.tone(freq, dur, &wf, vol)?)
            }
            "audio_noise" => {
                let dur = gi(a, 0, "AUDIO_NOISE")?;
                if !(0..=600_000).contains(&dur) {
                    return Err("AUDIO_NOISE: dur_ms muss 0..600000 sein".into());
                }
                let vol = if a.len() >= 2 { need_f(a, 1, "AUDIO_NOISE")? } else { 1.0 };
                Value::Int(self.audio_mut()?.noise(dur, vol)?)
            }
            "audio_sfx" => {
                // AUDIO_SFX(wf, freq, slide, atk, sus, dec, vib_d, vib_s, vol
                //   [, stereo_width, duty, pwm_depth, pwm_speed,
                //    flt_cutoff, flt_sweep, flt_res]) -- SID-Args optional.
                let wf = gs(a, 0, "AUDIO_SFX")?.to_string();
                let freq = need_f(a, 1, "AUDIO_SFX")?;
                let slide = need_f(a, 2, "AUDIO_SFX")?;
                let atk = gi(a, 3, "AUDIO_SFX")?;
                let sus = gi(a, 4, "AUDIO_SFX")?;
                let dec = gi(a, 5, "AUDIO_SFX")?;
                let vd = need_f(a, 6, "AUDIO_SFX")?;
                let vs = need_f(a, 7, "AUDIO_SFX")?;
                let vol = need_f(a, 8, "AUDIO_SFX")?;
                let optf = |i: usize, d: f64| -> R<f64> {
                    if a.len() > i { need_f(a, i, "AUDIO_SFX") } else { Ok(d) }
                };
                let width = optf(9, 0.0)?;
                let duty = optf(10, 0.5)?;
                let pwm_depth = optf(11, 0.0)?;
                let pwm_speed = optf(12, 0.0)?;
                let flt_cutoff = optf(13, 0.0)?;
                let flt_sweep = optf(14, 0.0)?;
                let flt_res = optf(15, 0.0)?;
                Value::Int(self.audio_mut()?.sfx(
                    &wf, freq, slide, atk, sus, dec, vd, vs, vol, width,
                    duty, pwm_depth, pwm_speed, flt_cutoff, flt_sweep, flt_res)?)
            }
            "audio_note" => {
                // AUDIO_NOTE(wf$, freq, dauer_ms, attack_ms, decay_ms, sustain,
                //   release_ms, vol [, vib_depth, vib_speed, detune_cents,
                //   slide_halbtoene]) -- eine gehaltene Note mit echter ADSR.
                let wf = gs(a, 0, "AUDIO_NOTE")?.to_string();
                let freq = need_f(a, 1, "AUDIO_NOTE")?;
                let dauer = gi(a, 2, "AUDIO_NOTE")?;
                let atk = gi(a, 3, "AUDIO_NOTE")?;
                let dec = gi(a, 4, "AUDIO_NOTE")?;
                let sus = need_f(a, 5, "AUDIO_NOTE")?;
                let rel = gi(a, 6, "AUDIO_NOTE")?;
                let vol = need_f(a, 7, "AUDIO_NOTE")?;
                let optf = |i: usize, d: f64| -> R<f64> {
                    if a.len() > i { need_f(a, i, "AUDIO_NOTE") } else { Ok(d) }
                };
                let vd = optf(8, 0.0)?;
                let vs = optf(9, 0.0)?;
                let det = optf(10, 0.0)?;
                let slide = optf(11, 0.0)?;
                Value::Int(self.audio_mut()?.note(&wf, freq, dauer, atk, dec, sus, rel, vol,
                                                   vd, vs, det, slide)?)
            }
            "audio_sound_new" => {
                let dauer = gi(a, 0, "AUDIO_SOUND_NEW")?;
                Value::Int(self.audio_mut()?.sound_new(dauer)?)
            }
            "audio_sound_mix" => {
                // AUDIO_SOUND_MIX(ziel, quelle, offset_ms[, vol[, pan]])
                let ziel = gi(a, 0, "AUDIO_SOUND_MIX")?;
                let quelle = gi(a, 1, "AUDIO_SOUND_MIX")?;
                let offset = need_f(a, 2, "AUDIO_SOUND_MIX")?;
                let vol = if a.len() > 3 { need_f(a, 3, "AUDIO_SOUND_MIX")? } else { 1.0 };
                let pan = if a.len() > 4 { Some(need_f(a, 4, "AUDIO_SOUND_MIX")?) } else { None };
                if !(0.0..=4.0).contains(&vol) {
                    return Err("AUDIO_SOUND_MIX: vol muss 0..4 sein".into());
                }
                self.audio_mut()?.sound_mix(ziel, quelle, offset, vol, pan)?;
                Value::Nil
            }
            "audio_sound_normalize" => {
                let idx = gi(a, 0, "AUDIO_SOUND_NORMALIZE")?;
                let spitze = if a.len() > 1 { need_f(a, 1, "AUDIO_SOUND_NORMALIZE")? } else { 1.0 };
                Value::Float(self.audio_mut()?.sound_normalize(idx, spitze)?)
            }
            // Am SOUND-Handle, nicht an AUDIO_SFX: so gelten beide fuer jede
            // Klangquelle (TONE, NOISE, SFX, geladene Datei).
            "audio_sound_wave" => {
                let w = self.audio_mut()?.sound_wave(
                    gi(a, 0, "AUDIO_SOUND_WAVE")?, gi(a, 1, "AUDIO_SOUND_WAVE")?)?;
                crate::builtins::new_float_array(w)
            }
            "audio_save_wav" => {
                let idx = gi(a, 0, "AUDIO_SAVE_WAV")?;
                let pfad = gs(a, 1, "AUDIO_SAVE_WAV")?.to_string();
                let bits = if a.len() > 2 { gi(a, 2, "AUDIO_SAVE_WAV")? } else { 16 };
                self.audio_mut()?.save_wav(idx, &pfad, bits)?;
                Value::Nil
            }
            // --- Sprachausgabe (sprache.rs, Weg C aus docs/intern/entwurf-speak.md) ---
            // SPEAK fragt ZUERST, ob ein Bildschirmleser zuhoert: dann geht der
            // Satz als Ansage in den Baum (seine Stimme, sein Tempo, seine
            // Braillezeile -- und nicht zwei Stimmen zugleich). Sonst die
            // Systemstimme durch Kira. SPEAK_SOUND ist immer Synthese.
            "speak" => {
                let text = gs(a, 0, "SPEAK")?.to_string();
                let unterbrechen = a.len() > 1 && gb(a, 1, name)?;
                if a.len() > 2 { return Err(crate::vm::falsche_argumente("SPEAK", a.len())); }
                if self.gfx.as_ref().map_or(false, |g| g.a11y_aktiv()) {
                    self.gui.announce(text.clone(), unterbrechen);
                    if let Some(g) = self.gfx.as_mut() { g.a11y_ansagen(text, unterbrechen); }
                } else {
                    self.audio_mut()?;
                    let au = self.audio.as_mut().unwrap();
                    self.sprecher.sprechen(au, &text, unterbrechen)?;
                }
                Value::Nil
            }
            "speak_stop" => {
                if !a.is_empty() { return Err("SPEAK_STOP: erwartet keine Argumente".into()); }
                if let Some(au) = self.audio.as_mut() { self.sprecher.stopp(au); }
                Value::Nil
            }
            "speaking" => {
                if !a.is_empty() { return Err("SPEAKING: erwartet keine Argumente".into()); }
                Value::Bool(self.sprecher.spricht())
            }
            "speak_wait" => {
                if !a.is_empty() { return Err("SPEAK_WAIT: erwartet keine Argumente".into()); }
                if let Some(au) = self.audio.as_mut() { self.sprecher.warten(au); }
                Value::Nil
            }
            "speak_voice" => { self.sprecher.stimme_setzen(&gs(a, 0, "SPEAK_VOICE")?)?; Value::Nil }
            "speak_voices" => {
                if !a.is_empty() { return Err("SPEAK_VOICES: erwartet keine Argumente".into()); }
                let namen = self.sprecher.stimmen()?;
                let n = namen.len() as i64;
                let mut arr = DhArray::new("string".to_string(), vec![n], || Value::str_rc(""));
                for (k, s) in namen.into_iter().enumerate() { arr.cells.set(k, Value::str_rc(&s)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            "speak_rate" => { self.sprecher.tempo_setzen(need_f(a, 0, "SPEAK_RATE")?)?; Value::Nil }
            "speak_sound" => {
                let text = gs(a, 0, "SPEAK_SOUND")?.to_string();
                if a.len() != 1 { return Err(crate::vm::falsche_argumente("SPEAK_SOUND", a.len())); }
                self.audio_mut()?;
                let au = self.audio.as_mut().unwrap();
                // Eine eigene Kopie: der Slot im Vorrat gehoert SPEAK und wird
                // nach 64 weiteren Saetzen verdraengt. Bis 2026.53 bekam das
                // Programm ihn selbst -- derselbe Text lieferte dasselbe
                // Handle, und das Handle starb spaeter still.
                let vorrat = self.sprecher.klang(au, &text)?;
                Value::Int(au.sound_kopie(vorrat, "SPEAK_SOUND")?)
            }
            "audio_music_load" => { let p = gs(a, 0, "AUDIO_MUSIC_LOAD")?.to_string(); self.audio_mut()?.music_load(&p)?; Value::Nil }
            "audio_music_play" => {
                // AUDIO_MUSIC_PLAY([loops[, fade_in_ms[, easing$]]]) -- loops=-1 endlos (Default)
                let loops = if !a.is_empty() { gi(a, 0, "AUDIO_MUSIC_PLAY")? } else { -1 };
                let fade = if a.len() >= 2 { gi(a, 1, "AUDIO_MUSIC_PLAY")? } else { 0 };
                if fade < 0 { return Err("AUDIO_MUSIC_PLAY: fade_in_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 2, "AUDIO_MUSIC_PLAY")?;
                self.audio_mut()?.music_play(loops, fade, &easing)?; Value::Nil
            }
            "audio_music_stop" => {
                // AUDIO_MUSIC_STOP([fade_out_ms[, easing$]])
                let fade = if !a.is_empty() { gi(a, 0, "AUDIO_MUSIC_STOP")? } else { 0 };
                if fade < 0 { return Err("AUDIO_MUSIC_STOP: fade_out_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 1, "AUDIO_MUSIC_STOP")?;
                self.audio_mut()?.music_stop(fade, &easing)?; Value::Nil
            }
            "audio_music_pause" => { self.audio_mut()?.music_pause(); Value::Nil }
            "audio_music_resume" => { self.audio_mut()?.music_resume(); Value::Nil }
            "audio_music_volume" => { let v = need_f(a, 0, "AUDIO_MUSIC_VOLUME")?; self.audio_mut()?.music_set_volume(v)?; Value::Nil }
            "audio_music_get_volume" => Value::Float(round_audio(self.audio_mut()?.music_get_volume())),
            "audio_music_position" => Value::Float(self.audio_mut()?.music_position()),
            "audio_music_seek" => {
                let s = need_f(a, 0, "AUDIO_MUSIC_SEEK")?;
                // Nicht `s < 0.0` schreiben -- so faellt NAN mit durch.
                if !(s >= 0.0) {
                    return Err(format!(
                        "AUDIO_MUSIC_SEEK: Position muss >= 0 sein (war {})", s));
                }
                self.audio_mut()?.music_seek(s)?;
                Value::Nil
            }
            "audio_music_busy" => Value::Bool(self.audio_mut()?.music_busy()),
            "audio_music_queue" => { let p = gs(a, 0, "AUDIO_MUSIC_QUEUE")?.to_string(); self.audio_mut()?.music_queue(&p)?; Value::Nil }

            // --- Clock (Kira-Uhr fuer sample-genaues Musik-/Rhythmus-Timing) ---
            // --- Modulatoren: LFO + Tweener ---------------------------------
            // Kira faehrt sie auf dem Audio-Thread: ein Tremolo/Filter-Sweep
            // laeuft sample-genau weiter, auch wenn ein Frame einbricht -- und
            // das DH-Programm ruft dafuer NICHTS pro Frame.
            "audio_lfo_new" => {
                let wave = gs(a, 0, "AUDIO_LFO_NEW")?.to_string();
                let hz = need_f(a, 1, "AUDIO_LFO_NEW")?;
                if hz < 0.0 { return Err("AUDIO_LFO_NEW: Frequenz darf nicht negativ sein".into()); }
                let amp = if a.len() > 2 { need_f(a, 2, "AUDIO_LFO_NEW")? } else { 1.0 };
                let off = if a.len() > 3 { need_f(a, 3, "AUDIO_LFO_NEW")? } else { 0.0 };
                Value::Int(self.audio_mut()?.lfo_new(&wave, hz, amp, off)?)
            }
            "audio_lfo_set" => {
                let m = gi(a, 0, "AUDIO_LFO_SET")?;
                let hz = if a.len() > 1 { Some(need_f(a, 1, "AUDIO_LFO_SET")?) } else { None };
                let amp = if a.len() > 2 { Some(need_f(a, 2, "AUDIO_LFO_SET")?) } else { None };
                let off = if a.len() > 3 { Some(need_f(a, 3, "AUDIO_LFO_SET")?) } else { None };
                self.audio_mut()?.lfo_set(m, hz, amp, off)?; Value::Nil
            }
            "audio_lfo_waveform" => {
                let m = gi(a, 0, "AUDIO_LFO_WAVEFORM")?;
                let w = gs(a, 1, "AUDIO_LFO_WAVEFORM")?.to_string();
                self.audio_mut()?.lfo_waveform(m, &w)?; Value::Nil
            }
            "audio_tweener_new" => {
                let v = if a.is_empty() { 0.0 } else { need_f(a, 0, "AUDIO_TWEENER_NEW")? };
                Value::Int(self.audio_mut()?.tweener_new(v)?)
            }
            "audio_tweener_to" => {
                let m = gi(a, 0, "AUDIO_TWEENER_TO")?;
                let target = need_f(a, 1, "AUDIO_TWEENER_TO")?;
                let ms = if a.len() > 2 { need_f(a, 2, "AUDIO_TWEENER_TO")? } else { 0.0 };
                let ez = if a.len() > 3 { gs(a, 3, "AUDIO_TWEENER_TO")?.to_string() } else { String::new() };
                self.audio_mut()?.tweener_to(m, target, ms, &ez)?; Value::Nil
            }
            "audio_mod_remove" => {
                let m = gi(a, 0, "AUDIO_MOD_REMOVE")?;
                self.audio_mut()?.mod_remove(m)?; Value::Nil
            }
            "audio_modulate" => {
                let bus = gs(a, 0, "AUDIO_MODULATE")?.to_string();
                let target = gs(a, 1, "AUDIO_MODULATE")?.to_string();
                let m = gi(a, 2, "AUDIO_MODULATE")?;
                let lo = need_f(a, 3, "AUDIO_MODULATE")?;
                let hi = need_f(a, 4, "AUDIO_MODULATE")?;
                self.audio_mut()?.modulate(&bus, &target, m, lo, hi)?; Value::Nil
            }
            "audio_bus_pan" => {
                let bus = gs(a, 0, "AUDIO_BUS_PAN")?.to_string();
                let pos = need_f(a, 1, "AUDIO_BUS_PAN")?;
                self.audio_mut()?.bus_pan(&bus, pos)?; Value::Nil
            }
            "audio_clock_new" => {
                // AUDIO_CLOCK_NEW(ticks_per_second) -- Wertpruefung VOR der
                // Audio-Initialisierung (golden-testbar).
                let tps = need_f(a, 0, "AUDIO_CLOCK_NEW")?;
                if tps <= 0.0 { return Err("AUDIO_CLOCK_NEW: ticks_per_second muss > 0 sein".into()); }
                Value::Int(self.audio_mut()?.clock_new(tps)?)
            }
            "audio_clock_start" => { let c = gi(a, 0, "AUDIO_CLOCK_START")?; self.audio_mut()?.clock_start(c)?; Value::Nil }
            "audio_clock_pause" => { let c = gi(a, 0, "AUDIO_CLOCK_PAUSE")?; self.audio_mut()?.clock_pause(c)?; Value::Nil }
            "audio_clock_stop" => { let c = gi(a, 0, "AUDIO_CLOCK_STOP")?; self.audio_mut()?.clock_stop(c)?; Value::Nil }
            "audio_clock_ticking" => { let c = gi(a, 0, "AUDIO_CLOCK_TICKING")?; Value::Bool(self.audio_mut()?.clock_ticking(c)?) }
            "audio_clock_ticks" => { let c = gi(a, 0, "AUDIO_CLOCK_TICKS")?; Value::Int(self.audio_mut()?.clock_ticks(c)?) }
            "audio_clock_set_speed" => {
                let c = gi(a, 0, "AUDIO_CLOCK_SET_SPEED")?;
                let tps = need_f(a, 1, "AUDIO_CLOCK_SET_SPEED")?;
                if tps <= 0.0 { return Err("AUDIO_CLOCK_SET_SPEED: ticks_per_second muss > 0 sein".into()); }
                self.audio_mut()?.clock_set_speed(c, tps)?; Value::Nil
            }
            "audio_clock_remove" => { let c = gi(a, 0, "AUDIO_CLOCK_REMOVE")?; self.audio_mut()?.clock_remove(c)?; Value::Nil }
            "audio_play_voice" => {
                let f = "AUDIO_PLAY_VOICE";
                let idx = gi(a, 0, f)?;
                let vol = if a.len() >= 2 { need_f(a, 1, f)? } else { 1.0 };
                let loops = if a.len() >= 3 { gi(a, 2, f)? } else { 0 };
                Value::Int(self.audio_mut()?.ch_play_voice(idx, vol, loops)?)
            }
            "audio_voice_count" => { let i = gi(a, 0, "AUDIO_VOICE_COUNT")?; Value::Int(self.audio_mut()?.voice_count(i)?) }
            "audio_play_at" => {
                // AUDIO_PLAY_AT(sound, clock, ticks[, volume[, loops]]) -- Start
                // exakt auf Tick `ticks` der Uhr `clock` statt sofort.
                let idx = gi(a, 0, "AUDIO_PLAY_AT")?;
                let clock = gi(a, 1, "AUDIO_PLAY_AT")?;
                let ticks = gi(a, 2, "AUDIO_PLAY_AT")?;
                let vol = if a.len() >= 4 { need_f(a, 3, "AUDIO_PLAY_AT")? } else { 1.0 };
                let loops = if a.len() >= 5 { gi(a, 4, "AUDIO_PLAY_AT")? } else { 0 };
                if ticks < 0 { return Err("AUDIO_PLAY_AT: ticks muss >= 0 sein".into()); }
                Value::Int(self.audio_mut()?.ch_play_at(idx, clock, ticks, vol, loops)?)
            }

            // --- Listener/Emitter (raeumliches Audio) ---
            "audio_listener_new" => {
                let x = need_f(a, 0, "AUDIO_LISTENER_NEW")?;
                let y = need_f(a, 1, "AUDIO_LISTENER_NEW")?;
                let z = need_f(a, 2, "AUDIO_LISTENER_NEW")?;
                Value::Int(self.audio_mut()?.listener_new(x, y, z)?)
            }
            "audio_listener_set_position" => {
                let l = gi(a, 0, "AUDIO_LISTENER_SET_POSITION")?;
                let x = need_f(a, 1, "AUDIO_LISTENER_SET_POSITION")?;
                let y = need_f(a, 2, "AUDIO_LISTENER_SET_POSITION")?;
                let z = need_f(a, 3, "AUDIO_LISTENER_SET_POSITION")?;
                self.audio_mut()?.listener_set_position(l, x, y, z)?; Value::Nil
            }
            "audio_listener_set_orientation" => {
                let l = gi(a, 0, "AUDIO_LISTENER_SET_ORIENTATION")?;
                let yaw = need_f(a, 1, "AUDIO_LISTENER_SET_ORIENTATION")?;
                self.audio_mut()?.listener_set_orientation(l, yaw)?; Value::Nil
            }
            "audio_listener_remove" => {
                let l = gi(a, 0, "AUDIO_LISTENER_REMOVE")?;
                self.audio_mut()?.listener_remove(l)?; Value::Nil
            }
            "audio_emitter_new" => {
                // AUDIO_EMITTER_NEW(listener, x, y, z[, min_dist[, max_dist]])
                let l = gi(a, 0, "AUDIO_EMITTER_NEW")?;
                let x = need_f(a, 1, "AUDIO_EMITTER_NEW")?;
                let y = need_f(a, 2, "AUDIO_EMITTER_NEW")?;
                let z = need_f(a, 3, "AUDIO_EMITTER_NEW")?;
                let min_d = if a.len() >= 5 { need_f(a, 4, "AUDIO_EMITTER_NEW")? } else { 1.0 };
                let max_d = if a.len() >= 6 { need_f(a, 5, "AUDIO_EMITTER_NEW")? } else { 100.0 };
                if min_d < 0.0 { return Err("AUDIO_EMITTER_NEW: min_dist muss >= 0 sein".into()); }
                if max_d <= min_d { return Err("AUDIO_EMITTER_NEW: max_dist muss > min_dist sein".into()); }
                Value::Int(self.audio_mut()?.emitter_new(l, x, y, z, min_d, max_d)?)
            }
            "audio_emitter_set_position" => {
                let e = gi(a, 0, "AUDIO_EMITTER_SET_POSITION")?;
                let x = need_f(a, 1, "AUDIO_EMITTER_SET_POSITION")?;
                let y = need_f(a, 2, "AUDIO_EMITTER_SET_POSITION")?;
                let z = need_f(a, 3, "AUDIO_EMITTER_SET_POSITION")?;
                self.audio_mut()?.emitter_set_position(e, x, y, z)?; Value::Nil
            }
            "audio_emitter_remove" => {
                let e = gi(a, 0, "AUDIO_EMITTER_REMOVE")?;
                self.audio_mut()?.emitter_remove(e)?; Value::Nil
            }
            "audio_play_on" => {
                // AUDIO_PLAY_ON(sound, emitter[, loops[, volume[, fade_in_ms[, easing$]]]])
                let idx = gi(a, 0, "AUDIO_PLAY_ON")?;
                let emitter = gi(a, 1, "AUDIO_PLAY_ON")?;
                let loops = if a.len() >= 3 { gi(a, 2, "AUDIO_PLAY_ON")? } else { 0 };
                let vol = if a.len() >= 4 { need_f(a, 3, "AUDIO_PLAY_ON")? } else { 1.0 };
                let fade = if a.len() >= 5 { gi(a, 4, "AUDIO_PLAY_ON")? } else { 0 };
                if fade < 0 { return Err("AUDIO_PLAY_ON: fade_in_ms muss >= 0 sein".into()); }
                let easing = easing_name(a, 5, "AUDIO_PLAY_ON")?;
                Value::Int(self.audio_mut()?.ch_play_on(idx, emitter, loops, vol, fade, &easing)?)
            }
            _ => return Ok(None), // kein Ton -> pure-Pfad
        };
        Ok(Some(r))
    }
}
