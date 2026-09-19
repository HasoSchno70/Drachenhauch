//! dhrt als Hauptprogramm eines `.app`-Bundles (2026-09-19).
//!
//! Bis hierher war das Hauptprogramm von `Drachenhauch.app` ein Shell-Skript
//! (installer/posix/drachenhauch), das die Beispiele kopierte und dann `dhrt`
//! startete. Fuer Apples Notarisierung muss jedes ausfuehrbare Programm mit
//! "Hardened Runtime" signiert sein -- das gibt es nur fuer echte Programme,
//! und ein Skript als Hauptprogramm ist dort ein bekannter Stolperstein.
//! Darum uebernimmt `dhrt` die Aufgabe selbst: gestartet OHNE Argumente aus
//! `<Name>.app/Contents/MacOS/`, mit der IDE unter `Contents/Resources/ide/`,
//! kopiert es die Beispiele in den Nutzerordner (nur, was dort fehlt) und
//! startet die IDE darin -- genau wie der Starter unter Linux.
//!
//! Der Pfad wird nur am Aufbau erkannt, nicht am System: so laesst sich der
//! Weg auch unter Windows pruefen (tests/pruef/werkzeug_paket.dhtest baut das
//! Bundle und startet dessen dhrt). Ein normaler `dhrt`-Aufruf ohne Argumente
//! liegt nie in so einem Ordner und meldet weiter "Verwendung: ...".
use std::path::{Path, PathBuf};

/// `Contents/Resources` des Bundles, wenn `exe` dessen Hauptprogramm ist und
/// dort eine IDE liegt.
pub fn bundle_resources(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    let contents = macos.parent()?;
    let app = contents.parent()?;
    let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if name(macos) != "MacOS" || name(contents) != "Contents" || !name(app).ends_with(".app") {
        return None;
    }
    let res = contents.join("Resources");
    if res.join("ide").join("ide.dh").is_file() { Some(res) } else { None }
}

/// Die Argumente zaehlen nicht, die macOS selbst anhaengt: aeltere Fassungen
/// geben einem vom Finder gestarteten Programm `-psn_0_12345` mit.
pub fn ohne_argumente(args: &[String]) -> bool {
    args.iter().skip(1).all(|a| a.starts_with("-psn_"))
}

/// Wohin die Beispiele kommen: `DH_IDE_BEISPIELE`, sonst
/// `~/Documents/Drachenhauch/examples`, ohne Dokumente-Ordner
/// `~/Drachenhauch/examples` -- dieselbe Regel wie der Linux-Starter.
pub fn beispiel_ziel(vorgabe: Option<&str>, home: &Path) -> PathBuf {
    if let Some(v) = vorgabe.filter(|v| !v.is_empty()) {
        return PathBuf::from(v);
    }
    let dok = home.join("Documents");
    let basis = if dok.is_dir() { dok } else { home.to_path_buf() };
    basis.join("Drachenhauch").join("examples")
}

/// Was im Ziel fehlt, aus `von` kopieren -- nichts ueberschreiben (bearbeitete
/// Beispiele bleiben, neue einer spaeteren Fassung kommen dazu). Liefert, wie
/// viele Dateien kopiert wurden.
pub fn fehlende_kopieren(von: &Path, nach: &Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(nach)?;
    let mut n = 0;
    for e in std::fs::read_dir(von)? {
        let e = e?;
        let ziel = nach.join(e.file_name());
        if e.file_type()?.is_dir() {
            n += fehlende_kopieren(&e.path(), &ziel)?;
        } else if !ziel.exists() {
            std::fs::copy(e.path(), &ziel)?;
            n += 1;
        }
    }
    Ok(n)
}

/// Alles vorbereiten und den Pfad der IDE liefern: Beispiele kopieren,
/// `DH_IDE_BEISPIELE`/`DH_IDE_WURZEL` setzen, in den Beispielordner wechseln
/// (dhrt merkt sich den Ort als DHRT_START_DIR, der Projektbaum zeigt beim
/// ersten Start etwas).
pub fn vorbereiten(res: &Path) -> PathBuf {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."));
    let ziel = beispiel_ziel(std::env::var("DH_IDE_BEISPIELE").ok().as_deref(), &home);
    if let Err(e) = fehlende_kopieren(&res.join("examples"), &ziel) {
        eprintln!("Beispiele nach {} kopieren: {}", ziel.display(), e);
    }
    std::env::set_var("DH_IDE_BEISPIELE", &ziel);
    std::env::set_var("DH_IDE_WURZEL", res);
    let _ = std::env::set_current_dir(&ziel);
    res.join("ide").join("ide.dh")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ordner(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("dhrt_appstart_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn erkennt_nur_ein_bundle_mit_ide() {
        let d = ordner("bundle");
        let exe = d.join("Drachenhauch.app/Contents/MacOS/dhrt");
        std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
        assert_eq!(bundle_resources(&exe), None, "ohne ide.dh kein Bundle-Start");
        let ide = d.join("Drachenhauch.app/Contents/Resources/ide/ide.dh");
        std::fs::create_dir_all(ide.parent().unwrap()).unwrap();
        std::fs::write(&ide, "").unwrap();
        assert_eq!(bundle_resources(&exe), Some(d.join("Drachenhauch.app/Contents/Resources")));
        // Derselbe Aufbau ohne .app ist kein Bundle.
        assert_eq!(bundle_resources(&d.join("x/Contents/MacOS/dhrt")), None);
    }

    #[test]
    fn psn_argumente_zaehlen_nicht() {
        assert!(ohne_argumente(&["dhrt".into()]));
        assert!(ohne_argumente(&["dhrt".into(), "-psn_0_1234".into()]));
        assert!(!ohne_argumente(&["dhrt".into(), "spiel.dh".into()]));
    }

    #[test]
    fn ziel_nach_dokumenten_und_vorgabe() {
        let h = ordner("home");
        assert_eq!(beispiel_ziel(None, &h), h.join("Drachenhauch/examples"));
        std::fs::create_dir_all(h.join("Documents")).unwrap();
        assert_eq!(beispiel_ziel(None, &h), h.join("Documents/Drachenhauch/examples"));
        assert_eq!(beispiel_ziel(Some("/x/y"), &h), PathBuf::from("/x/y"));
        assert_eq!(beispiel_ziel(Some(""), &h), h.join("Documents/Drachenhauch/examples"));
    }

    #[test]
    fn kopiert_nur_was_fehlt() {
        let d = ordner("kopie");
        let von = d.join("von");
        std::fs::create_dir_all(von.join("assets")).unwrap();
        std::fs::write(von.join("a.dh"), "neu").unwrap();
        std::fs::write(von.join("assets/b.png"), "bild").unwrap();
        let nach = d.join("nach");
        assert_eq!(fehlende_kopieren(&von, &nach).unwrap(), 2);
        std::fs::write(nach.join("a.dh"), "bearbeitet").unwrap();
        std::fs::remove_file(nach.join("assets/b.png")).unwrap();
        assert_eq!(fehlende_kopieren(&von, &nach).unwrap(), 1);
        assert_eq!(std::fs::read_to_string(nach.join("a.dh")).unwrap(), "bearbeitet");
        assert!(nach.join("assets/b.png").is_file());
    }
}
