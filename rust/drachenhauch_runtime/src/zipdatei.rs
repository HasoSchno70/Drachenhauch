//! ZIP lesen und schreiben (WP J).
//!
//! Sicherungen, Belegsammlungen, Export -- und der uebliche Weg, mehrere
//! Dateien als eine weiterzugeben.
//!
//! # Zip-Slip
//!
//! Ein Archiv darf Eintraege wie `../../autoexec.bat` oder `C:\Windows\...`
//! enthalten. Wer beim Entpacken den Namen aus dem Archiv einfach an den
//! Zielordner haengt, schreibt damit ausserhalb davon -- der Angreifer waehlt
//! die Datei, das Opfer entpackt sie. Deshalb geht JEDER Name hier durch
//! `sicherer_name`, und ein Eintrag, der die Pruefung nicht besteht, wird
//! uebersprungen statt entpackt. `ZIP_EXTRACT` liefert die Zahl der wirklich
//! geschriebenen Dateien zurueck, damit ein Unterschied auffaellt.

use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

/// Einen Eintragsnamen aus dem Archiv in einen Pfad UNTERHALB von `ziel`
/// uebersetzen -- oder `None`, wenn er ausbrechen will.
///
/// Abgelehnt wird alles, was nicht ausschliesslich aus gewoehnlichen
/// Namensteilen besteht: absolute Pfade, Laufwerksbuchstaben, `..` und die
/// Windows-Sonderformen (`\\?\`, UNC).
fn sicherer_name(ziel: &Path, name: &str) -> Option<PathBuf> {
    // Rueckwaertsschraegstriche kommen in freier Wildbahn vor, obwohl die
    // ZIP-Spezifikation nur `/` erlaubt.
    let name = name.replace('\\', "/");
    if name.is_empty() {
        return None;
    }
    let mut pfad = ziel.to_path_buf();
    let roh = PathBuf::from(&name);
    for teil in roh.components() {
        match teil {
            Component::Normal(t) => {
                // `C:/Windows/...` ist NUR auf Windows ein absoluter Pfad.
                // Auf Linux ist `C:` ein ganz gewoehnlicher Ordnername, und
                // die Pruefung liess ihn durch -- entdeckt vom Linux-Job beim
                // allerersten Lauf. Ausgebrochen waere damit nichts (der Pfad
                // bleibt unter `ziel`), aber ein Archiv mit Laufwerksbuchstabe
                // ist auf jedem System verdaechtig, und ein Ordner namens
                // "C:" ist auf keinem gewollt. Also ueberall ablehnen.
                let n = t.to_string_lossy();
                let laufwerk = n.len() == 2 && n.ends_with(':')
                    && n.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false);
                if laufwerk { return None; }
                pfad.push(t);
            }
            // Alles andere ist ein Ausbruchsversuch oder sinnlos.
            _ => return None,
        }
    }
    // Doppelt gemoppelt, aber billig: das Ergebnis muss unter `ziel` liegen.
    if pfad.starts_with(ziel) && pfad != ziel { Some(pfad) } else { None }
}

/// Die Namen aller Eintraege.
pub fn liste(archiv: &Path) -> Result<Vec<String>, String> {
    let datei = std::fs::File::open(archiv)
        .map_err(|e| format!("{}: {}", archiv.display(), e))?;
    let mut zip = zip::ZipArchive::new(datei)
        .map_err(|e| format!("{}: kein lesbares ZIP: {}", archiv.display(), e))?;
    let mut raus = Vec::with_capacity(zip.len());
    for i in 0..zip.len() {
        let e = zip.by_index(i).map_err(|e| format!("Eintrag {}: {}", i, e))?;
        raus.push(e.name().to_string());
    }
    Ok(raus)
}

/// Einen einzelnen Eintrag als Bytes lesen.
pub fn lies(archiv: &Path, name: &str) -> Result<Vec<u8>, String> {
    let datei = std::fs::File::open(archiv)
        .map_err(|e| format!("{}: {}", archiv.display(), e))?;
    let mut zip = zip::ZipArchive::new(datei)
        .map_err(|e| format!("{}: kein lesbares ZIP: {}", archiv.display(), e))?;
    let mut e = zip.by_name(name)
        .map_err(|_| format!("{}: kein Eintrag namens {:?}", archiv.display(), name))?;
    let mut daten = Vec::new();
    e.read_to_end(&mut daten).map_err(|e| format!("{}: {}", name, e))?;
    Ok(daten)
}

/// Alles entpacken. Liefert die Zahl der geschriebenen Dateien.
pub fn entpacke(archiv: &Path, ziel: &Path) -> Result<i64, String> {
    let datei = std::fs::File::open(archiv)
        .map_err(|e| format!("{}: {}", archiv.display(), e))?;
    let mut zip = zip::ZipArchive::new(datei)
        .map_err(|e| format!("{}: kein lesbares ZIP: {}", archiv.display(), e))?;
    std::fs::create_dir_all(ziel)
        .map_err(|e| format!("{}: {}", ziel.display(), e))?;
    let ziel = ziel.canonicalize().unwrap_or_else(|_| ziel.to_path_buf());
    let mut anzahl = 0i64;
    for i in 0..zip.len() {
        let mut e = zip.by_index(i).map_err(|e| format!("Eintrag {}: {}", i, e))?;
        let name = e.name().to_string();
        let pfad = match sicherer_name(&ziel, &name) {
            Some(p) => p,
            None => continue,        // Zip-Slip: ueberspringen, nicht schreiben
        };
        if e.is_dir() || name.ends_with('/') {
            std::fs::create_dir_all(&pfad)
                .map_err(|x| format!("{}: {}", pfad.display(), x))?;
            continue;
        }
        if let Some(eltern) = pfad.parent() {
            std::fs::create_dir_all(eltern)
                .map_err(|x| format!("{}: {}", eltern.display(), x))?;
        }
        let mut aus = std::fs::File::create(&pfad)
            .map_err(|x| format!("{}: {}", pfad.display(), x))?;
        std::io::copy(&mut e, &mut aus)
            .map_err(|x| format!("{}: {}", pfad.display(), x))?;
        anzahl += 1;
    }
    Ok(anzahl)
}

/// Ein Archiv aus einer Liste von Dateien bauen.
///
/// Im Archiv steht jeweils nur der DATEINAME, nicht der Pfad, unter dem die
/// Datei lag -- sonst traegt ein Archiv die Verzeichnisstruktur des Rechners
/// nach aussen, auf dem es entstanden ist.
pub fn packe(archiv: &Path, dateien: &[String]) -> Result<i64, String> {
    let aus = std::fs::File::create(archiv)
        .map_err(|e| format!("{}: {}", archiv.display(), e))?;
    let mut zip = zip::ZipWriter::new(aus);
    let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut anzahl = 0i64;
    for d in dateien {
        let pfad = Path::new(d);
        let name = pfad.file_name()
            .ok_or_else(|| format!("ZIP_CREATE: {:?} hat keinen Dateinamen", d))?
            .to_string_lossy()
            .to_string();
        let inhalt = std::fs::read(pfad)
            .map_err(|e| format!("ZIP_CREATE: {}: {}", pfad.display(), e))?;
        zip.start_file(name, opts)
            .map_err(|e| format!("ZIP_CREATE: {}", e))?;
        zip.write_all(&inhalt)
            .map_err(|e| format!("ZIP_CREATE: {}: {}", pfad.display(), e))?;
        anzahl += 1;
    }
    zip.finish().map_err(|e| format!("ZIP_CREATE: {}", e))?;
    Ok(anzahl)
}

/// Ein Archiv aus Name/Inhalt-Paaren bauen, ohne den Umweg ueber Dateien.
pub fn packe_daten(archiv: &Path, eintraege: &[(String, Vec<u8>)]) -> Result<i64, String> {
    let aus = std::fs::File::create(archiv)
        .map_err(|e| format!("{}: {}", archiv.display(), e))?;
    let mut zip = zip::ZipWriter::new(aus);
    let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, daten) in eintraege {
        zip.start_file(name.clone(), opts)
            .map_err(|e| format!("ZIP_WRITE: {}", e))?;
        zip.write_all(daten).map_err(|e| format!("ZIP_WRITE: {}: {}", name, e))?;
    }
    zip.finish().map_err(|e| format!("ZIP_WRITE: {}", e))?;
    Ok(eintraege.len() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ausbruchsversuche_werden_abgelehnt() {
        // `C:/...` ist bewusst dabei: auf Windows ein absoluter Pfad, auf
        // Linux ein Ordner namens "C:" -- abgelehnt wird es auf beiden.
        let ziel = Path::new("/tmp/ziel");
        for boese in ["../raus.txt", "../../raus.txt", "/etc/passwd",
                      "C:/Windows/system32/x.dll", "a/../../raus.txt", "",
                      "D:/daten.txt", "unter/C:/x.txt"] {
            assert!(sicherer_name(ziel, boese).is_none(), "durchgelassen: {}", boese);
        }
    }

    #[test]
    fn gewoehnliche_namen_gehen_durch() {
        let ziel = Path::new("/tmp/ziel");
        assert_eq!(sicherer_name(ziel, "a.txt"), Some(ziel.join("a.txt")));
        assert_eq!(sicherer_name(ziel, "unter/b.txt"),
                   Some(ziel.join("unter").join("b.txt")));
    }

    #[test]
    fn rueckwaertsschraegstrich_gilt_als_trenner() {
        // Kommt vor, obwohl die Spezifikation nur `/` erlaubt -- und `..\x`
        // muss genauso abgelehnt werden wie `../x`.
        let ziel = Path::new("/tmp/ziel");
        assert_eq!(sicherer_name(ziel, "unter\\b.txt"),
                   Some(ziel.join("unter").join("b.txt")));
        assert!(sicherer_name(ziel, "..\\raus.txt").is_none());
    }
}
