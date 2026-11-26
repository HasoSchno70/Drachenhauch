//! Neue Builtin-Namen nach der Namens-Leitlinie (`docs/builtins-namen.md`).
//!
//! Die Befehle sind ueber Jahre gewachsen und heissen darum uneinheitlich:
//! `LOADIMAGE` neben `IMAGE_SAVE`, `KEYPRESSED` (gehalten) neben
//! `INPUT_PRESSED` (gerade gedrueckt), `ZEIT_JETZT` in einer sonst englischen
//! Befehlsliste. Statt umzubenennen -- jedes Programm, jedes Buch und jedes
//! Beispiel braeche --, bekommen sie hier einen zweiten Namen nach der
//! Leitlinie. Der alte bleibt gueltig.
//!
//! Der Compiler bildet einen neuen Namen auf den bisherigen ab
//! (`Compiler::resolve_builtin_alias`), bevor er irgendetwas nachschlaegt --
//! Laufzeit, Maschinencode, Argumentpruefung und Modul-Hinweis sehen also nur
//! den bisherigen, und die Laufzeit muss von den neuen Namen nichts wissen.
//! Der Index (`daten/builtin_index.json`) fuehrt beide, damit
//! Vervollstaendigung, Hover und Grammatik den neuen kennen; der Hover eines
//! bisherigen nennt den neuen (`lsp::builtin_doku`).
//!
//! Neue Eintraege: hier, im Index (Kopie des bisherigen mit neuem Namen) und
//! als Zeile in `docs/builtins-namen.md` und `docs/en/builtins-namen.md`. Die
//! Tests unten halten Tabelle und Index zusammen.

/// `(neu, bisher)`, klein geschrieben wie der Lexer Namen liefert.
pub const NEU: &[(&str, &str)] = &[
    // ---- gehalten / gerade gedrueckt / losgelassen: _DOWN, _HIT, _RELEASED
    ("key_down", "keypressed"),
    ("key_hit", "keyhit"),
    ("key_released", "keyreleased"),
    ("key_repeat", "keyrepeat"),
    ("key_wait", "waitkey"),
    ("mouse_down", "mousebutton"),
    ("mouse_x", "mousex"),
    ("mouse_y", "mousey"),
    ("mouse_wheel", "mousewheel"),
    ("mouse_wheel_x", "mousewheel_x"),
    ("mouse_wheel_y", "mousewheel_y"),
    ("mouse_set_visible", "mouse_visible"),
    ("joystick_down", "joystick_button"),
    ("input_down", "input_held"),
    ("input_hit", "input_pressed"),
    // ---- gleicher Name, andere Bedeutung
    ("gui_listbox_items", "gui_set_listbox"),
    ("gui_dropdown_items", "gui_set_dropdown"),
    ("gui_dock_set_size", "gui_dock_size"),
    ("audio_music_set_pitch", "audio_music_pitch"),
    ("set_count", "set_size"),
    // ---- gui: GUI_SET_X setzt, GUI_GET_X liest
    ("gui_get_text$", "gui_text"),
    ("gui_get_value", "gui_value"),
    ("gui_get_checked", "gui_checked"),
    ("gui_get_enabled", "gui_enabled"),
    ("gui_get_visible", "gui_visible"),
    ("gui_get_active_tab", "gui_active_tab"),
    ("gui_get_date$", "gui_date"),
    ("gui_get_time$", "gui_time$"),
    ("gui_get_error$", "gui_error"),
    ("gui_get_picked_color", "gui_picked_color"),
    ("gui_get_motion", "gui_motion"),
    ("gui_get_sounds", "gui_sounds"),
    ("gui_set_tooltip", "gui_tooltip"),
    ("gui_get_tooltip$", "gui_tooltip$"),
    ("gui_dock_set_title", "gui_dock_title"),
    ("gui_dock_get_title$", "gui_dock_title$"),
    ("gui_statusbar_set_tip", "gui_statusbar_tip"),
    ("gui_statusbar_get_tip$", "gui_statusbar_tip$"),
    // ---- Fenster und Schirm
    ("window_set_fullscreen", "set_fullscreen"),
    ("window_set_pos", "set_window_pos"),
    ("window_set_monitor", "set_window_monitor"),
    ("window_set_title", "setwindowtitle"),
    ("window_set_fps", "setfps"),
    ("screen_width", "screenwidth"),
    ("screen_height", "screenheight"),
    ("screenshot_save", "savescreenshot"),
    ("quit_requested", "quitrequested"),
    // ---- Zeichnen
    ("draw_image", "drawimage"),
    ("draw_image_flipped", "drawimageflipped"),
    ("draw_image_part", "drawimagepart"),
    ("draw_image_part_ex", "drawimagepartex"),
    ("draw_image_rot", "drawimagerot"),
    ("draw_image_9", "drawimage9"),
    ("draw_tilemap", "drawtilemap"),
    ("draw_fps", "drawfps"),
    ("box_round", "boxround"),
    ("rect_round", "rectround"),
    ("circle_outline", "circleoutline"),
    ("ellipse_outline", "ellipseoutline"),
    ("triangle_outline", "triangleoutline"),
    ("polygon_outline", "polygonoutline"),
    ("ngon_outline", "ngonoutline"),
    ("pie_outline", "pieoutline"),
    // ---- Bilder, Schrift, Ton, Modelle: BEREICH_VERB
    ("image_load", "loadimage"),
    ("image_get_pixel", "getpixel"),
    ("image_get_alpha", "getalpha"),
    ("font_load", "loadfont"),
    ("font_load_image", "loadfont_image"),
    ("font_set", "setfont"),
    ("sound_load", "loadsound"),
    ("sound_play", "playsound"),
    ("sound_stop", "stopsound"),
    ("sound_free", "unloadsound"),
    ("music_play", "playmusic"),
    ("music_stop", "stopmusic"),
    ("model_load", "loadmodel"),
    // Bis 2026.53 als zweiter Name im Match-Arm (vm_grafik.rs) -- ohne
    // Hover-Hinweis, und Meldungen nannten je nach Arm den einen oder anderen.
    ("image_width", "imagewidth"),
    ("image_height", "imageheight"),
    ("audio_set_volume", "audio_volume"),
    ("audio_music_set_volume", "audio_music_volume"),
    // Fristen heissen _SET_TIMEOUT und zaehlen ms (beide taten das schon).
    ("smtp_set_timeout", "smtp_timeout"),
    ("modbus_set_timeout", "modbus_timeout"),
    // Freigeben heisst _FREE, eine Verbindung schliessen _CLOSE (Regel 4,
    // die Handle-Regel in CLAUDE.md) -- bis 2026.53 _REMOVE, _DESTROY und
    // _DISCONNECT nebeneinander.
    ("audio_clock_free", "audio_clock_remove"),
    ("audio_emitter_free", "audio_emitter_remove"),
    ("audio_listener_free", "audio_listener_remove"),
    ("audio_mod_free", "audio_mod_remove"),
    ("gui_widget_free", "gui_destroy"),
    ("gui_window_free", "gui_window_destroy"),
    ("phys2d_body_free", "phys2d_remove"),
    ("phys3d_body_free", "phys3d_remove"),
    ("bt_close", "bt_disconnect"),
    ("mqtt_close", "mqtt_disconnect"),
    // ---- Dateien (mit Pfad), Dateien (mit Handle), Ordner, Pfade
    ("file_exists", "fileexists"),
    ("file_size", "filesize"),
    ("file_time", "filetime"),
    ("file_copy", "copyfile"),
    ("file_delete", "deletefile"),
    ("file_same", "samefile"),
    ("file_append", "appendfile"),
    ("file_read$", "readall$"),
    ("file_read_lines", "readlines"),
    ("file_read_bytes", "readall_bytes"),
    ("file_write", "writeall"),
    ("file_write_bytes", "writeall_bytes"),
    ("file_open", "openfile"),
    ("file_close", "closefile"),
    ("file_read_line$", "readline"),
    ("file_write_line", "writeline"),
    ("file_eof", "endoffile"),
    ("dir_exists", "direxists"),
    ("dir_list", "dirlist"),
    ("dir_list_rec", "dirlist_rec"),
    ("path_join", "pathjoin"),
    ("path_dir", "dirname"),
    ("path_base", "basename"),
    ("path_real$", "realpath$"),
    ("start_dir$", "startdir$"),
    ("arg_path$", "argpath$"),
    ("env_get$", "getenv$"),
    ("env_set", "setenv"),
    ("url_open", "openurl"),
    ("doc_open", "opendoc"),
    // ---- MAP wie ARRAY_ und SET_: mit Unterstrich, Anzahl heisst COUNT
    ("map_get", "mapget"),
    ("map_get_or", "mapgetor"),
    ("map_put", "mapput"),
    ("map_has", "maphas"),
    ("map_remove", "mapremove"),
    ("map_clear", "mapclear"),
    ("map_keys", "mapkeys"),
    ("map_values", "mapvalues"),
    ("map_items", "mapitems"),
    ("map_count", "mapsize"),
    ("array_dim_count", "dimcount"),
    ("array_dim_size", "dimsize"),
    ("mqtt_unacked_count", "mqtt_pending"),
    ("move_toward", "movetoward"),
    // ---- Englisch statt Deutsch (Module zeit und geld, FFI-Zeiger)
    ("time_now", "zeit_jetzt"),
    ("time_unix", "zeit_unix"),
    ("time_from_unix", "zeit_aus_unix"),
    ("time_to_unix", "zeit_nach_unix"),
    ("time_from_parts", "zeit_aus_teilen"),
    ("time_part", "zeit_teil"),
    ("time_format$", "zeit_format$"),
    ("time_text$", "zeit_text$"),
    ("time_parse", "zeit_parse"),
    ("time_can_parse", "zeit_lesbar"),
    ("time_add", "zeit_plus"),
    ("time_diff", "zeit_diff"),
    ("time_duration$", "zeit_dauer$"),
    ("time_weekday", "zeit_wochentag"),
    ("money_new", "geld_neu"),
    ("money_abs", "geld_abs"),
    ("money_cents", "geld_cent"),
    ("money_from_cents", "geld_aus_cent"),
    ("money_round", "geld_runden"),
    ("money_split", "geld_teilen"),
    ("money_text$", "geld_text$"),
    ("money_value", "geld_zahl"),
    ("buffer_from_pointer", "buffer_aus_zeiger"),
    ("text_from_pointer$", "text_aus_zeiger$"),
];

/// Englische Modulnamen: `IMPORT "time"` ist `IMPORT "zeit"`.
const MODULE: &[(&str, &str)] = &[("time", "zeit"), ("money", "geld")];

/// Der bisherige Name zu einem neuen (klein). `None`: kein neuer Name.
pub fn bisher(name: &str) -> Option<&'static str> {
    NEU.iter().find(|(n, _)| *n == name).map(|(_, b)| *b)
}

/// Der neue Name zu einem bisherigen (klein) -- fuer den Hover.
pub fn neu(name: &str) -> Option<&'static str> {
    NEU.iter().find(|(_, b)| *b == name).map(|(n, _)| *n)
}

/// Kanonischer Modulname (`time` -> `zeit`); sonst unveraendert.
pub fn modul(name: &str) -> &str {
    MODULE.iter().find(|(e, _)| *e == name).map(|(_, d)| *d).unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn index() -> (HashSet<String>, std::collections::HashMap<String, String>) {
        let raw = include_str!("../../../daten/builtin_index.json");
        let v: serde_json::Value = serde_json::from_str(raw).unwrap();
        let mut namen = HashSet::new();
        let mut sig = std::collections::HashMap::new();
        for e in v["builtins"].as_array().unwrap() {
            let n = e["name"].as_str().unwrap().to_lowercase();
            sig.insert(n.clone(), e["signature"].as_str().unwrap_or("").to_string());
            namen.insert(n);
        }
        (namen, sig)
    }

    #[test]
    fn jeder_bisherige_name_ist_ein_builtin() {
        let (namen, _) = index();
        for (n, b) in NEU {
            assert!(namen.contains(*b), "{} zeigt auf {}, das es nicht gibt", n, b);
        }
    }

    #[test]
    fn jeder_neue_name_steht_im_index_mit_eigener_signatur() {
        let (namen, sig) = index();
        for (n, _) in NEU {
            assert!(namen.contains(*n), "{} fehlt in daten/builtin_index.json", n);
            let s = sig[*n].to_lowercase();
            assert!(s.starts_with(&format!("{}(", n)), "Signatur von {} beginnt nicht mit dem Namen: {}", n, s);
        }
    }

    #[test]
    fn kein_name_doppelt_und_keiner_zeigt_auf_einen_neuen() {
        let mut gesehen = HashSet::new();
        for (n, b) in NEU {
            assert!(gesehen.insert(*n), "{} doppelt", n);
            assert!(bisher(b).is_none(), "{} zeigt auf {}, das selbst ein neuer Name ist", n, b);
            assert_ne!(n, b);
        }
    }

    #[test]
    fn neue_namen_folgen_der_leitlinie() {
        for (n, _) in NEU {
            // Mit Unterstrich gegliedert (ausser einwortigen gibt es keine),
            // nur Kleinbuchstaben, Ziffern, `_` und am Ende hoechstens `$`.
            assert!(n.contains('_'), "{}: ohne Unterstrich", n);
            let k = n.trim_end_matches('$');
            assert!(k.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'), "{}", n);
        }
    }

    #[test]
    fn modulnamen() {
        assert_eq!(modul("time"), "zeit");
        assert_eq!(modul("money"), "geld");
        assert_eq!(modul("json"), "json");
    }
}
