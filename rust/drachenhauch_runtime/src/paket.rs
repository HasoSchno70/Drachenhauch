//! `dhrt paket` -- Bibliotheken holen und teilen (docs/entwurf-pakete.md).
//!
//!   dhrt paket hole                       alles aus paket.json holen
//!   dhrt paket hole <quelle> [--als name] eines dazunehmen
//!   dhrt paket entferne <name>
//!   dhrt paket liste
//!   dhrt paket neu [name]
//!
//! **Kein eigener Server**: eine Quelle ist `github:nutzer/repo@stand`, eine
//! https-Adresse (ZIP oder einzelne `.dh`) oder ein lokaler Ordner/ZIP. Ein
//! Stand ist immer genau angegeben -- "der neueste" waere morgen ein anderer.
//!
//! **Pakete gehoeren zum Projekt**: sie landen in `pakete/<name>/` neben der
//! `paket.json`, und `IMPORT` sucht von der importierenden Datei aus nach
//! oben in jedem `pakete/` (preprocess::paketpfade). `paket.lock.json`
//! haelt fest, was tatsaechlich geholt wurde, samt SHA-256 -- kommt beim
//! naechsten Holen etwas anderes an, ist das ein Fehler statt eines stillen
//! Unterschieds.
//!
//! Beim Holen wird **nichts ausgefuehrt**; entpackt wird mit den Schutzregeln
//! des zip-Moduls (kein `..`, keine absoluten Pfade).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{json, Value};
use sha2::Digest;

const PAKET_JSON: &str = "paket.json";
const SPERRE_JSON: &str = "paket.lock.json";
const ORDNER: &str = "pakete";
/// Tiefe der Abhaengigkeiten -- ein Kreis ist schon vorher ein Fehler, das
/// hier faengt nur Unsinn ab.
const MAX_TIEFE: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub enum Art {
    GitHub { nutzer: String, repo: String, stand: String },
    Url(String),
    Lokal(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Quelle {
    pub roh: String,
    pub art: Art,
    pub name: String,
}

/// Eine Quelle lesen. `basis` ist der Ordner, zu dem ein lokaler Pfad
/// relativ ist (der Projektordner -- nicht der Arbeitsordner, sonst
/// hinge `dhrt paket hole` davon ab, wo man es aufruft).
pub fn quelle_lesen(roh: &str, basis: &Path) -> Result<Quelle, String> {
    let roh = roh.trim();
    if roh.is_empty() { return Err("leere Quelle".into()); }
    if let Some(rest) = roh.strip_prefix("github:") {
        let (pfad, stand) = rest.split_once('@').ok_or_else(|| format!(
            "'{}': der Stand fehlt -- github:nutzer/repo@stand (ein Tag, Zweig oder Commit). \
             Ohne Stand holte jeder Rechner etwas anderes.", roh))?;
        let (nutzer, repo) = pfad.split_once('/').ok_or_else(||
            format!("'{}': erwartet github:nutzer/repo@stand", roh))?;
        if nutzer.is_empty() || repo.is_empty() || stand.is_empty() || repo.contains('/') {
            return Err(format!("'{}': erwartet github:nutzer/repo@stand", roh));
        }
        return Ok(Quelle { roh: roh.into(), name: repo.to_string(),
            art: Art::GitHub { nutzer: nutzer.into(), repo: repo.into(), stand: stand.into() } });
    }
    if roh.starts_with("https://") || roh.starts_with("http://") {
        let ohne = roh.split(['?', '#']).next().unwrap_or(roh);
        let letzt = ohne.trim_end_matches('/').rsplit('/').next().unwrap_or("");
        let name = letzt.strip_suffix(".zip").or_else(|| letzt.strip_suffix(".dh")).unwrap_or(letzt);
        return Ok(Quelle { roh: roh.into(), name: name.to_string(), art: Art::Url(roh.into()) });
    }
    let pfad = basis.join(roh);
    let stamm = pfad.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let name = stamm.strip_suffix(".zip").or_else(|| stamm.strip_suffix(".dh")).unwrap_or(&stamm).to_string();
    Ok(Quelle { roh: roh.into(), name, art: Art::Lokal(pfad) })
}

/// Ein Paketname wird ein Ordnername und der erste Teil jedes IMPORTs.
pub fn name_pruefen(name: &str) -> Result<(), String> {
    if name.is_empty() || name.starts_with('.') || name == ORDNER
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')) {
        return Err(format!("'{}' taugt nicht als Paketname (Buchstaben, Ziffern, _ - .) -- mit --als name einen anderen waehlen", name));
    }
    Ok(())
}

fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{:02x}", x)).collect() }

/// Pruefsumme eines Ordners: die Dateien sortiert nach ihrem Weg, je Weg
/// und Inhalt -- so haengt sie nicht an der Reihenfolge, in der das
/// Dateisystem sie liefert.
fn ordner_summe(dir: &Path) -> Result<String, String> {
    let mut dateien = Vec::new();
    dateien_sammeln(dir, dir, &mut dateien)?;
    dateien.sort();
    let mut h = sha2::Sha256::new();
    for rel in dateien {
        h.update(rel.as_bytes());
        h.update([0u8]);
        h.update(std::fs::read(dir.join(&rel)).map_err(|e| format!("{}: {}", rel, e))?);
        h.update([0u8]);
    }
    Ok(hex(&h.finalize()))
}

/// Alle Dateien unter `dir` (Wege mit `/`, relativ zu `wurzel`) -- ohne
/// `.git` und ohne das eigene `pakete/` (die Abhaengigkeiten eines Pakets
/// holt `holen` selbst, sie gehoeren nicht zu seinem Inhalt).
fn dateien_sammeln(wurzel: &Path, dir: &Path, aus: &mut Vec<String>) -> Result<(), String> {
    let eintraege = std::fs::read_dir(dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
    for e in eintraege.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if name == ".git" || (dir == wurzel && name == ORDNER) { continue; }
            dateien_sammeln(wurzel, &p, aus)?;
        } else {
            let rel = p.strip_prefix(wurzel).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            aus.push(rel);
        }
    }
    Ok(())
}

fn ordner_kopieren(von: &Path, nach: &Path) -> Result<(), String> {
    let mut dateien = Vec::new();
    dateien_sammeln(von, von, &mut dateien)?;
    for rel in dateien {
        let ziel = nach.join(&rel);
        if let Some(e) = ziel.parent() { std::fs::create_dir_all(e).map_err(|x| format!("{}: {}", e.display(), x))?; }
        std::fs::copy(von.join(&rel), &ziel).map_err(|x| format!("{}: {}", rel, x))?;
    }
    Ok(())
}

/// Liegt nach dem Entpacken alles in EINEM Oberordner (so liefert GitHub,
/// `spielkiste-1.2/`), wird dieser Ordner weggelassen.
fn oberordner_weg(dir: &Path) -> Result<(), String> {
    let eintraege: Vec<_> = std::fs::read_dir(dir).map_err(|e| e.to_string())?.flatten().collect();
    if eintraege.len() != 1 || !eintraege[0].path().is_dir() { return Ok(()); }
    let ober = eintraege[0].path();
    let zwischen = dir.with_extension("ober");
    let _ = std::fs::remove_dir_all(&zwischen);
    std::fs::rename(&ober, &zwischen).map_err(|e| e.to_string())?;
    std::fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
    std::fs::rename(&zwischen, dir).map_err(|e| e.to_string())?;
    Ok(())
}

/// Eine Adresse nach `datei` laden.
fn laden(url: &str, datei: &Path) -> Result<(), String> {
    #[cfg(feature = "http")]
    {
        crate::html::http_download(url, &datei.to_string_lossy())
            .map(|_| ()).map_err(|e| format!("{}: {}", url, e.msg))
    }
    #[cfg(not(feature = "http"))]
    {
        let _ = datei;
        Err(format!("{}: diese Fassung von dhrt ist ohne das Feature `http` gebaut", url))
    }
}

/// Woran ein Paket in der Kette erkannt wird: der aufgeloeste Ordner bzw.
/// die Adresse.
fn schluessel(q: &Quelle) -> String {
    match &q.art {
        Art::Lokal(p) => p.canonicalize().unwrap_or_else(|_| p.clone()).display().to_string(),
        Art::GitHub { nutzer, repo, stand } => format!("github:{}/{}@{}", nutzer, repo, stand),
        Art::Url(u) => u.clone(),
    }
}

/// Was nach dem Holen festgehalten wird.
pub struct Geholt { pub adresse: String, pub sha256: String }

/// Ein Paket nach `ziel` (= `pakete/<name>`) holen. Erst in einen
/// Nebenordner, dann die Pruefsumme, dann umbenennen -- ein abgebrochener
/// oder falscher Download laesst das alte Paket stehen. `erwartet` ist die
/// Pruefsumme aus der Sperrdatei; `kette` die Quellen auf dem Weg hierher
/// (ein Kreis ist ein Fehler).
pub fn holen(q: &Quelle, ziel: &Path, erwartet: Option<&str>, kette: &mut Vec<String>,
             melden: &mut dyn FnMut(&str)) -> Result<Geholt, String> {
    // Verglichen wird der aufgeloeste Ort, nicht der Text der Quelle --
    // `../libs/a` und `../a` koennen derselbe Ordner sein.
    let schluessel = schluessel(q);
    if kette.contains(&schluessel) {
        return Err(format!("Kreis in den Abhaengigkeiten: {} kommt auf dem Weg zu sich selbst noch einmal vor", q.roh));
    }
    if kette.len() >= MAX_TIEFE { return Err(format!("zu tief verschachtelt bei {}", q.roh)); }
    name_pruefen(&q.name)?;
    let eltern = ziel.parent().ok_or("kein Zielordner")?;
    std::fs::create_dir_all(eltern).map_err(|e| format!("{}: {}", eltern.display(), e))?;
    let neu = eltern.join(format!(".neu-{}", q.name));
    let _ = std::fs::remove_dir_all(&neu);
    std::fs::create_dir_all(&neu).map_err(|e| format!("{}: {}", neu.display(), e))?;
    let ergebnis = (|| -> Result<Geholt, String> {
        let adresse = match &q.art {
            Art::Lokal(p) if p.is_dir() => {
                ordner_kopieren(p, &neu)?;
                q.roh.clone()
            }
            Art::Lokal(p) => {
                let daten = std::fs::read(p).map_err(|e| format!("{}: {}", p.display(), e))?;
                einsetzen(&daten, p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), &neu)?;
                q.roh.clone()
            }
            Art::GitHub { nutzer, repo, stand } => {
                let url = format!("https://github.com/{}/{}/archive/{}.zip", nutzer, repo, stand);
                let daten = laden_daten(&url, &neu)?;
                einsetzen(&daten, format!("{}.zip", repo), &neu)?;
                url
            }
            Art::Url(url) => {
                let daten = laden_daten(url, &neu)?;
                let datei = url.split(['?', '#']).next().unwrap_or(url).rsplit('/').next().unwrap_or("paket").to_string();
                einsetzen(&daten, datei, &neu)?;
                url.clone()
            }
        };
        // Die Pruefsumme geht ueber den INHALT, nicht ueber die geladenen
        // Bytes: ein Server darf dasselbe ZIP neu packen (GitHub hat das 2023
        // getan), ohne dass sich etwas geaendert hat.
        let sha256 = ordner_summe(&neu)?;
        if let Some(e) = erwartet {
            if e != sha256 {
                return Err(format!(
                    "{}: angekommen ist etwas anderes als beim letzten Mal (SHA-256 {} statt {}). \
                     Wurde der Stand verschoben? Wer den neuen Inhalt will: dhrt paket hole --erneuern",
                    q.roh, &sha256[..12], &e[..e.len().min(12)]));
            }
        }
        Ok(Geholt { adresse, sha256 })
    })();
    let g = match ergebnis {
        Ok(g) => g,
        Err(e) => { let _ = std::fs::remove_dir_all(&neu); return Err(e); }
    };
    let _ = std::fs::remove_dir_all(ziel);
    std::fs::rename(&neu, ziel).map_err(|e| format!("{}: {}", ziel.display(), e))?;
    melden(&format!("{} {}  ({})", "geholt", q.name, q.roh));
    // Die Abhaengigkeiten des Pakets in SEIN pakete/. Ein lokaler Pfad in
    // seiner paket.json gilt ab dem Ordner, aus dem das Paket KAM -- nicht
    // ab der Kopie in pakete/.
    let herkunft: Option<PathBuf> = match &q.art {
        Art::Lokal(p) if p.is_dir() => Some(p.clone()),
        Art::Lokal(p) => p.parent().map(Path::to_path_buf),
        _ => None,
    };
    kette.push(schluessel);
    let r = abhaengigkeiten_holen(ziel, herkunft.as_deref(), kette, melden);
    kette.pop();
    r?;
    Ok(g)
}

fn laden_daten(url: &str, neu: &Path) -> Result<Vec<u8>, String> {
    let tmp = neu.join(".download");
    laden(url, &tmp)?;
    let daten = std::fs::read(&tmp).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&tmp);
    Ok(daten)
}

/// Geladene Bytes in `neu` ablegen: ein ZIP wird entpackt (und ein einzelner
/// Oberordner weggelassen), alles andere ist eine einzelne Datei.
fn einsetzen(daten: &[u8], dateiname: String, neu: &Path) -> Result<(), String> {
    if daten.starts_with(b"PK\x03\x04") || daten.starts_with(b"PK\x05\x06") {
        let zip = neu.with_extension("zip");
        std::fs::write(&zip, daten).map_err(|e| e.to_string())?;
        let r = crate::zipdatei::entpacke(&zip, neu);
        let _ = std::fs::remove_file(&zip);
        r?;
        oberordner_weg(neu)
    } else {
        if !dateiname.to_lowercase().ends_with(".dh") {
            return Err(format!("'{}' ist weder ein ZIP noch eine .dh-Datei", dateiname));
        }
        std::fs::write(neu.join(&dateiname), daten).map_err(|e| e.to_string())
    }
}

/// Hat ein Paket selbst eine paket.json, kommen deren Pakete in sein
/// eigenes `pakete/` -- mit seiner Sperrdatei, wenn es eine hat.
fn abhaengigkeiten_holen(dir: &Path, herkunft: Option<&Path>, kette: &mut Vec<String>,
                         melden: &mut dyn FnMut(&str)) -> Result<(), String> {
    let pj = dir.join(PAKET_JSON);
    if !pj.is_file() { return Ok(()); }
    let wunsch = json_lesen(&pj)?;
    let sperre = json_lesen(&dir.join(SPERRE_JSON)).unwrap_or(json!({}));
    for (name, q) in pakete_von(&wunsch) {
        let mut quelle = quelle_lesen(&q, herkunft.unwrap_or(dir))?;
        if matches!(quelle.art, Art::Lokal(_)) && herkunft.is_none() {
            return Err(format!(
                "ein heruntergeladenes Paket kann nicht auf einen lokalen Pfad verweisen ('{}' in {})",
                q, dir.join(PAKET_JSON).display()));
        }
        quelle.name = name.clone();
        let erwartet = sperre["pakete"][&name]["sha256"].as_str().map(str::to_string);
        holen(&quelle, &dir.join(ORDNER).join(&name), erwartet.as_deref(), kette, melden)?;
    }
    Ok(())
}

fn json_lesen(p: &Path) -> Result<Value, String> {
    let t = std::fs::read_to_string(p).map_err(|e| format!("{}: {}", p.display(), e))?;
    serde_json::from_str(&t).map_err(|e| format!("{}: kein gueltiges JSON ({})", p.display(), e))
}

fn json_schreiben(p: &Path, v: &Value) -> Result<(), String> {
    let mut t = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    t.push('\n');
    std::fs::write(p, t).map_err(|e| format!("{}: {}", p.display(), e))
}

/// (Name, Quelle) aus einer paket.json, in ihrer Reihenfolge.
fn pakete_von(wunsch: &Value) -> Vec<(String, String)> {
    wunsch.get("pakete").and_then(|p| p.as_object()).map(|m| m.iter()
        .filter_map(|(k, v)| v.as_str().map(|q| (k.clone(), q.to_string()))).collect()).unwrap_or_default()
}

/// Der Projektordner: der naechste Ordner mit paket.json vom Arbeitsordner
/// aus nach oben, sonst der Arbeitsordner selbst.
fn projekt() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.ancestors().find(|d| d.join(PAKET_JSON).is_file()).map(Path::to_path_buf).unwrap_or(cwd)
}

pub const HILFE: &str = "\
dhrt paket hole [--erneuern]           alles aus paket.json holen (--erneuern: neuen Inhalt annehmen)
dhrt paket hole <quelle> [--als name]  ein Paket dazunehmen
dhrt paket entferne <name>             ein Paket wieder herausnehmen
dhrt paket liste                       was das Projekt hat
dhrt paket neu [name]                  eine paket.json anlegen

Quellen: github:nutzer/repo@stand, https://.../paket.zip, https://.../datei.dh,
         ein lokaler Ordner oder ein lokales ZIP.
Die Pakete landen in pakete/<name>/ neben der paket.json; danach findet
IMPORT \"name/datei.dh\" sie. paket.lock.json haelt fest, was geholt wurde
(SHA-256). Beim Holen wird nichts ausgefuehrt -- aber ein Paket ist fremder
Code, der beim Laufen alles darf, was ein Programm darf.";

pub fn main(args: &[String]) -> ExitCode {
    let mut melden = |t: &str| println!("{}", t);
    let r = match args.first().map(String::as_str) {
        Some("hole") => hole(&args[1..], &mut melden),
        Some("entferne") => entferne(&args[1..], &mut melden),
        Some("liste") => liste(&mut melden),
        Some("neu") => neu(&args[1..], &mut melden),
        Some("--help") | Some("-h") | None => { println!("{}", HILFE); Ok(()) }
        Some(x) => Err(format!("unbekannt: {}\n{}", x, HILFE)),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => { eprintln!("dhrt paket: {}", e); ExitCode::from(1) }
    }
}

fn neu(args: &[String], melden: &mut dyn FnMut(&str)) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let pj = cwd.join(PAKET_JSON);
    if pj.exists() { return Err(format!("{} gibt es schon", pj.display())); }
    let name = args.first().cloned().unwrap_or_else(||
        cwd.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "projekt".into()));
    json_schreiben(&pj, &json!({"name": name, "pakete": {}}))?;
    melden(&format!("angelegt: {}", pj.display()));
    Ok(())
}

fn hole(args: &[String], melden: &mut dyn FnMut(&str)) -> Result<(), String> {
    let wurzel = projekt();
    let pj = wurzel.join(PAKET_JSON);
    let sj = wurzel.join(SPERRE_JSON);
    let mut wunsch = if pj.is_file() { json_lesen(&pj)? } else {
        json!({"name": wurzel.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), "pakete": {}})
    };
    let mut sperre = if sj.is_file() { json_lesen(&sj)? } else { json!({"pakete": {}}) };
    // Was zu holen ist: eine neue Quelle, oder alles aus der paket.json.
    let mut auftraege: Vec<(String, String)> = Vec::new();
    let mut als: Option<String> = None;
    // `--erneuern`: den Inhalt nehmen, der jetzt ankommt, und die Sperrdatei
    // nachziehen -- fuer den, der den neuen Stand bewusst will.
    let mut erneuern = false;
    let mut quelle: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--erneuern" {
            erneuern = true;
            i += 1;
        } else if args[i] == "--als" {
            als = Some(args.get(i + 1).cloned().ok_or("--als braucht einen Namen")?);
            i += 2;
        } else { quelle = Some(args[i].clone()); i += 1; }
    }
    let neu_dazu = quelle.is_some();
    if !neu_dazu && !pj.is_file() {
        return Err("hier gibt es keine paket.json -- dhrt paket hole <quelle> legt eine an, dhrt paket neu eine leere".into());
    }
    if let Some(q) = quelle {
        let mut qq = quelle_lesen(&q, &wurzel)?;
        if let Some(a) = als { qq.name = a; }
        name_pruefen(&qq.name)?;
        auftraege.push((qq.name.clone(), q));
    } else {
        auftraege = pakete_von(&wunsch);
        if auftraege.is_empty() { melden("paket.json nennt keine Pakete"); }
    }
    for (name, q) in auftraege {
        let mut qq = quelle_lesen(&q, &wurzel)?;
        qq.name = name.clone();
        // Die Pruefsumme gilt nur, solange die Quelle dieselbe ist.
        let erwartet = sperre["pakete"][&name].as_object()
            .filter(|e| e.get("quelle").and_then(|x| x.as_str()) == Some(q.as_str()))
            .and_then(|e| e.get("sha256")).and_then(|x| x.as_str()).map(str::to_string)
            .filter(|_| !erneuern);
        let g = holen(&qq, &wurzel.join(ORDNER).join(&name), erwartet.as_deref(), &mut Vec::new(), melden)?;
        if !wunsch["pakete"].is_object() { wunsch["pakete"] = json!({}); }
        wunsch["pakete"][&name] = json!(q);
        if !sperre["pakete"].is_object() { sperre["pakete"] = json!({}); }
        sperre["pakete"][&name] = json!({"quelle": q, "adresse": g.adresse, "sha256": g.sha256});
    }
    if neu_dazu || pj.is_file() { json_schreiben(&pj, &wunsch)?; }
    json_schreiben(&sj, &sperre)?;
    Ok(())
}

fn entferne(args: &[String], melden: &mut dyn FnMut(&str)) -> Result<(), String> {
    let name = args.first().ok_or("welches Paket? dhrt paket entferne <name>")?;
    name_pruefen(name)?;
    let wurzel = projekt();
    let pj = wurzel.join(PAKET_JSON);
    let sj = wurzel.join(SPERRE_JSON);
    let mut gefunden = false;
    for p in [&pj, &sj] {
        if !p.is_file() { continue; }
        let mut v = json_lesen(p)?;
        if let Some(m) = v.get_mut("pakete").and_then(|m| m.as_object_mut()) {
            if m.shift_remove(name.as_str()).is_some() { gefunden = true; }
        }
        json_schreiben(p, &v)?;
    }
    let dir = wurzel.join(ORDNER).join(name);
    if dir.is_dir() { std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?; gefunden = true; }
    if !gefunden { return Err(format!("'{}' gibt es in diesem Projekt nicht", name)); }
    melden(&format!("entfernt {}", name));
    Ok(())
}

fn liste(melden: &mut dyn FnMut(&str)) -> Result<(), String> {
    let wurzel = projekt();
    let pj = wurzel.join(PAKET_JSON);
    if !pj.is_file() { melden("hier gibt es keine paket.json"); return Ok(()); }
    let wunsch = json_lesen(&pj)?;
    let sperre = json_lesen(&wurzel.join(SPERRE_JSON)).unwrap_or(json!({}));
    for (name, q) in pakete_von(&wunsch) {
        let sha = sperre["pakete"][&name]["sha256"].as_str().map(|s| s[..s.len().min(12)].to_string());
        let da = wurzel.join(ORDNER).join(&name).is_dir();
        melden(&format!("{}  {}  {}{}", name, q, sha.unwrap_or_else(|| "(nicht gesperrt)".into()),
                        if da { "" } else { "  -- fehlt, dhrt paket hole" }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quellen_lesen() {
        let b = Path::new("/projekt");
        let q = quelle_lesen("github:hans/spielkiste@1.2", b).unwrap();
        assert_eq!(q.name, "spielkiste");
        assert_eq!(q.art, Art::GitHub { nutzer: "hans".into(), repo: "spielkiste".into(), stand: "1.2".into() });
        assert!(quelle_lesen("github:hans/spielkiste", b).unwrap_err().contains("Stand fehlt"));
        assert!(quelle_lesen("github:hans@1", b).is_err());
        assert_eq!(quelle_lesen("https://x.de/a/kiste.zip?x=1", b).unwrap().name, "kiste");
        assert_eq!(quelle_lesen("https://x.de/hilfe.dh", b).unwrap().name, "hilfe");
        assert_eq!(quelle_lesen("libs/werkzeug", b).unwrap().art, Art::Lokal(b.join("libs/werkzeug")));
    }

    #[test]
    fn namen() {
        assert!(name_pruefen("spiel-kiste_2.0").is_ok());
        for n in ["", ".x", "pakete", "a/b", "a b", ".."] { assert!(name_pruefen(n).is_err(), "{}", n); }
    }
}
