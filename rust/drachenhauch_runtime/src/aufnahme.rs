//! `dhrt aufnehmen programm.dh ziel.dhtest [name] [--bilder N]` -- die
//! Bedienung eines Programms aufnehmen und als Fall einer Pruefsammlung
//! anhaengen.
//!
//! dhrt startet sich selbst mit dem Programm; zwei Umgebungsvariablen sagen
//! der Laufzeit, was sie tun soll (vm_grafik.rs nach SCREEN, `Vm::run` am
//! Ende): `DHRT_RECORD=datei` nimmt Maus und Tasten auf
//! (AUTOMATION_RECORD), `DHRT_GUI_STATE_FILE=datei` schreibt am Ende den
//! Zustand der gui (GUI_STATE$). Daraus wird ein Fall:
//!
//! ```text
//! === name
//! --- programm ../pfad/zum/programm.dh
//! --- datei aufnahme.txt          (die Ereignisse)
//! --- umgebung
//! DHRT_FRAMES=...                 (letztes Ereignis + 30 Bilder)
//! DHRT_PLAYBACK=aufnahme.txt     (spielt nach SCREEN ab)
//! DHRT_GUI_STATE=1              (gibt am Ende GUI_STATE$ aus)
//! DHRT_OFFSCREEN=1                  (Fenster aus dem Blick)
//! --- enthaelt
//! gui-zustand
//!   textinput#name "Hallo"
//! ```
//!
//! Die Wiedergabe beginnt beim ersten Ereignis; damit die Bilder stimmen,
//! bekommt die Aufnahme eines in Bild 0 vorangestellt (CLAUDE.md,
//! Pruefsammlungen).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Die Ereigniszeilen einer Aufnahme (raylibs Format `e bild art p0 p1 p2`,
/// Kommentare weg) mit einem Ereignis in Bild 0 vorn; dazu das letzte Bild.
pub(crate) fn ereignisse_vorbereiten(text: &str) -> (Vec<String>, u64) {
    let mut e: Vec<String> = text.lines()
        .map(|l| l.split("//").next().unwrap_or("").trim())
        .filter(|l| l.starts_with("e "))
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();
    let bild = |l: &str| l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    let letztes = e.iter().map(|l| bild(l)).max().unwrap_or(0);
    if e.first().map(|l| bild(l) > 0).unwrap_or(true) {
        // Die Maus dorthin, wo sie zuerst gesehen wurde -- sonst (0, 0).
        let (x, y) = e.iter().find_map(|l| {
            let t: Vec<&str> = l.split_whitespace().collect();
            if t.get(2) == Some(&"7") { Some((t.get(3).copied().unwrap_or("0").to_string(), t.get(4).copied().unwrap_or("0").to_string())) } else { None }
        }).unwrap_or(("0".into(), "0".into()));
        e.insert(0, format!("e 0 7 {} {} 0 0", x, y));
    }
    (e, letztes)
}

/// Pfad von `ordner` zu `ziel`, mit `/` -- fuer `--- programm`.
pub(crate) fn relativ(ordner: &Path, ziel: &Path) -> String {
    let teile = |p: &Path| -> Vec<String> {
        p.components().filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().to_string()),
            std::path::Component::Prefix(p) => Some(p.as_os_str().to_string_lossy().replace("\\\\?\\", "").to_lowercase()),
            _ => None,
        }).collect()
    };
    let (a, b) = (teile(ordner), teile(ziel));
    let gleich = a.iter().zip(b.iter()).take_while(|(x, y)| x.eq_ignore_ascii_case(y)).count();
    let mut r: Vec<String> = vec!["..".to_string(); a.len() - gleich];
    r.extend(b[gleich..].iter().cloned());
    r.join("/")
}

/// Der Text des Falls.
pub(crate) fn fall_text(name: &str, programm: &str, ereignisse: &[String], bilder: u64, zustand: &[String], datum: &str) -> String {
    let mut t = String::new();
    t.push_str(&format!("=== {}\n", name));
    t.push_str(&format!("' Aufgenommen mit `dhrt aufnehmen` ({}): {} Ereignisse in {} Bildern.\n", datum, ereignisse.len(), bilder));
    t.push_str("' Erwartet wird der Zustand der gui am Ende (GUI_STATE$).\n");
    t.push_str(&format!("--- programm {}\n", programm));
    t.push_str("--- datei aufnahme.txt\n");
    t.push_str(&format!("c {}\n", ereignisse.len()));
    for e in ereignisse { t.push_str(e); t.push('\n'); }
    t.push('\n');
    t.push_str("--- umgebung\n");
    t.push_str(&format!("DHRT_FRAMES={}\nDHRT_PLAYBACK=aufnahme.txt\nDHRT_GUI_STATE=1\nDHRT_OFFSCREEN=1\n", bilder));
    t.push_str("--- enthaelt\ngui-zustand\n");
    for z in zustand { if !z.trim().is_empty() { t.push_str(z.trim_end()); t.push('\n'); } }
    t
}

fn heute() -> String {
    let (j, mo, d, h, mi, _) = crate::builtins::local_datetime();
    format!("{:04}-{:02}-{:02} {:02}:{:02}", j, mo, d, h, mi)
}

pub fn main(args: &[String]) -> ExitCode {
    let mut frei: Vec<&String> = Vec::new();
    let mut bilder: Option<u64> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--bilder" {
            match args.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                Some(n) => bilder = Some(n),
                None => { eprintln!("aufnehmen --bilder: erwartet eine Zahl"); return ExitCode::from(2); }
            }
            i += 2;
        } else { frei.push(&args[i]); i += 1; }
    }
    if frei.len() < 2 {
        eprintln!("Aufruf: dhrt aufnehmen programm.dh ziel.dhtest [name] [--bilder N]");
        return ExitCode::from(2);
    }
    let programm = PathBuf::from(frei[0]);
    let ziel = PathBuf::from(frei[1]);
    if !programm.is_file() { eprintln!("aufnehmen: '{}' gibt es nicht", programm.display()); return ExitCode::from(2); }
    let name = frei.get(2).map(|s| s.to_string()).unwrap_or_else(|| {
        format!("aufnahme {} {}", programm.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(), heute())
    });
    let tmp = std::env::temp_dir().join(format!("dh_aufnahme_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp);
    let ev = tmp.join("ereignisse.txt");
    let zu = tmp.join("zustand.txt");
    let exe = match std::env::current_exe() { Ok(e) => e, Err(e) => { eprintln!("aufnehmen: {}", e); return ExitCode::from(1); } };
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("run");
    if let Some(n) = bilder { cmd.arg("--bilder").arg(n.to_string()); }
    cmd.arg(&programm).env("DHRT_RECORD", &ev).env("DHRT_GUI_STATE_FILE", &zu)
        .env_remove("DHRT_FRAMES").env_remove("DHRT_PLAYBACK").env_remove("DHRT_ABSPIELEN");
    eprintln!("aufnehmen: '{}' -- bedienen, dann das Fenster schliessen", programm.display());
    let status = match cmd.status() { Ok(s) => s, Err(e) => { eprintln!("aufnehmen: {}", e); return ExitCode::from(1); } };
    if !status.success() { eprintln!("aufnehmen: das Programm endete mit {}", status); }
    // Keine Datei heisst: es wurde nichts aufgenommen (kein SCREEN, oder die
    // Laufzeit hat die Aufnahme am Ende nicht geschrieben) -- kein Fall.
    let text = match std::fs::read_to_string(&ev) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("aufnehmen: keine Aufnahme -- oeffnet das Programm ein Fenster mit SCREEN?");
            let _ = std::fs::remove_dir_all(&tmp);
            return ExitCode::from(1);
        }
    };
    let (ereignisse, letztes) = ereignisse_vorbereiten(&text);
    let zustand: Vec<String> = std::fs::read_to_string(&zu).unwrap_or_default().lines().map(|s| s.to_string()).collect();
    if zustand.is_empty() { eprintln!("aufnehmen: kein Zustand der gui -- benutzt das Programm IMPORT \"gui\" und SCREEN?"); }
    let bilder_fall = (letztes + 30).max(bilder.unwrap_or(0)).max(30);
    let ordner = ziel.parent().map(|p| if p.as_os_str().is_empty() { Path::new(".") } else { p }).unwrap_or(Path::new("."));
    let abs = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let rel = relativ(&abs(ordner), &abs(&programm));
    let fall = fall_text(&name, &rel, &ereignisse, bilder_fall, &zustand, &heute());
    let neu = !ziel.exists();
    let mut inhalt = if neu {
        "' Aufgenommene Faelle (dhrt aufnehmen).\n' Format: docs/werkzeuge.md, dhrt test.\n\n".to_string()
    } else {
        let alt = std::fs::read_to_string(&ziel).unwrap_or_default();
        if alt.ends_with("\n\n") || alt.is_empty() { alt } else if alt.ends_with('\n') { alt + "\n" } else { alt + "\n\n" }
    };
    inhalt.push_str(&fall);
    if let Err(e) = std::fs::write(&ziel, inhalt) { eprintln!("aufnehmen: '{}' -- {}", ziel.display(), e); return ExitCode::from(1); }
    let _ = std::fs::remove_dir_all(&tmp);
    eprintln!("aufnehmen: Fall '{}' mit {} Ereignissen -> {}", name, ereignisse.len(), ziel.display());
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ereignisse_bekommen_ein_bild_null() {
        let (e, l) = ereignisse_vorbereiten("# raylib automation\nc 2\ne 37 7 120 80 0 0 // INPUT_MOUSE_POSITION\ne 40 6 0 0 0 0\n");
        assert_eq!(e, vec!["e 0 7 120 80 0 0", "e 37 7 120 80 0 0", "e 40 6 0 0 0 0"]);
        assert_eq!(l, 40);
        let (e, l) = ereignisse_vorbereiten("c 0\n");
        assert_eq!((e, l), (vec!["e 0 7 0 0 0 0".to_string()], 0));
        let (e, _) = ereignisse_vorbereiten("e 0 2 65 0 0 0\n");
        assert_eq!(e.len(), 1, "schon ein Ereignis in Bild 0");
    }

    #[test]
    fn relative_pfade() {
        assert_eq!(relativ(Path::new("/a/b/tests"), Path::new("/a/b/examples/x.dh")), "../examples/x.dh");
        assert_eq!(relativ(Path::new("/a"), Path::new("/a/x.dh")), "x.dh");
    }

    #[test]
    fn der_fall_ist_vollstaendig() {
        let t = fall_text("klick", "../x.dh", &["e 0 7 1 2 0 0".into()], 40, &["fenster \"T\"".into(), "  label(1) \"ja\"".into()], "2026-11-15 10:00");
        for s in ["=== klick\n", "--- programm ../x.dh\n", "c 1\ne 0 7 1 2 0 0\n\n", "DHRT_FRAMES=40\n", "DHRT_PLAYBACK=aufnahme.txt\n", "--- enthaelt\ngui-zustand\nfenster \"T\"\n  label(1) \"ja\"\n"] {
            assert!(t.contains(s), "fehlt: {:?}", s);
        }
    }
}
