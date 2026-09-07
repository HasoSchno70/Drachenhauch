//! Pruefsammlungen fuer `dhrt test`: viele Faelle in EINER Datei (`*.dhtest`).
//!
//! Ein Pruefprogramm (`*_pruefung.dh`) ist ein Programm, das mit `ASSERT` selbst
//! prueft. Die meisten Tests der Sprache sind aber Ausgabevergleiche: ein
//! kurzes Programm, eine erwartete Ausgabe -- 1084 der pytest-Tests sahen 2026-09
//! genau so aus (`assert run_gb(src) == "1\n2\n"`). Fuer die braucht es kein
//! Programm je Datei, sondern eine Sammlung:
//!
//! ```text
//! ' Kopfkommentar bis zum ersten Fall
//! === zaehler liefert drei Werte
//! FUNCTION z() AS INTEGER
//!     YIELD 1
//!     YIELD 2
//! END FUNCTION
//! DIM c AS COROUTINE : c = z()
//! PRINT CORO_RESUME(c)
//! PRINT CORO_RESUME(c)
//! --- erwartet
//! 1
//! 2
//! === Division durch Null bricht ab
//! PRINT 1 \ 0
//! --- fehler
//! Division durch Null
//! ```
//!
//! Abschnitte je Fall: der Quelltext (alles bis zum ersten `---`), `--- erwartet`
//! (die Ausgabe, Zeile fuer Zeile), `--- enthaelt` (jede Zeile muss in der
//! Ausgabe vorkommen), `--- fehler` (Rueckgabewert ungleich 0, und jede Zeile
//! des Blocks steht in der Fehlermeldung), `--- datei <name>` (eine Datei, die
//! vor dem Lauf neben dem Programm liegt -- fuer JSON, Karten, Bilder als Text),
//! `--- umgebung` (`NAME=WERT` je Zeile, z. B. `DHRT_FRAMES=1`). Ohne Erwartung
//! zaehlt ein Fall als bestanden, wenn er mit 0 endet -- wie ein Pruefprogramm.
//!
//! Jeder Fall laeuft als eigener `dhrt run`-Prozess in einem eigenen
//! Verzeichnis (die Prozessgrenze ist die Zusage, wie bei `TASK_START`); die
//! Faelle einer Datei laufen parallel. Der Vergleich ist zeilenweise; EIN
//! Zeilenumbruch am Ende wird beiderseits nicht gezaehlt, weil `PRINT x;` und
//! `PRINT x` sich in einer Sammlung sonst nicht unterscheiden liessen.
//!
//! Reine Logik ohne Prozess und ohne Dateisystem -- damit sie sich hier pruefen
//! laesst; der Laeufer steht in `main.rs::test_main`.

/// Ein Fall einer Sammlung.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Fall {
    pub name: String,
    /// Zeile des `===` in der Datei (1-basiert), fuer die Meldung.
    pub zeile: usize,
    pub quelle: String,
    pub erwartet: Option<String>,
    pub enthaelt: Vec<String>,
    pub fehler: Option<Vec<String>>,
    pub dateien: Vec<(String, String)>,
    pub umgebung: Vec<(String, String)>,
}

/// Was ein Lauf ergeben hat.
#[derive(Debug, Clone, PartialEq)]
pub enum Ergebnis {
    Ok,
    Fehl(String),
    Uebersprungen(String),
}

/// Meldungen, an denen man eine Maschine ohne Bildschirm oder Soundkarte
/// erkennt -- dieselben wie in `tests/conftest.py`: ein Fall, der daran
/// scheitert, ist nicht falsch, er ist hier nicht pruefbar.
pub const KEIN_FENSTER: &[&str] = &[
    "Attempting to create window failed",
    "does not appear to support OpenGL",
    "Failed to initialize Window",
    "Failed to initialize platform",
    "NoDefaultOutputDevice",
];

#[derive(PartialEq, Clone, Copy)]
enum Abschnitt { Quelle, Erwartet, Enthaelt, Fehler, Datei, Umgebung }

/// Eine Sammlung aus ihrem Text lesen.
pub fn parsen(text: &str) -> Result<Vec<Fall>, String> {
    let mut faelle: Vec<Fall> = Vec::new();
    let mut abschnitt = Abschnitt::Quelle;
    let mut puffer: Vec<String> = Vec::new();
    let mut datei_name = String::new();

    fn abschliessen(f: &mut Fall, a: Abschnitt, puffer: &mut Vec<String>, datei_name: &str) {
        let text = puffer.join("\n");
        match a {
            Abschnitt::Quelle => f.quelle = text,
            // Leerzeilen am Ende des Blocks sind der Abstand zum naechsten Fall,
            // keine erwartete Ausgabe.
            Abschnitt::Erwartet => {
                let mut z: Vec<&String> = puffer.iter().collect();
                while z.last().is_some_and(|s| s.trim().is_empty()) { z.pop(); }
                f.erwartet = Some(z.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"));
            }
            Abschnitt::Enthaelt => f.enthaelt.extend(puffer.iter().filter(|z| !z.trim().is_empty()).cloned()),
            Abschnitt::Fehler => {
                let zeilen: Vec<String> = puffer.iter().filter(|z| !z.trim().is_empty()).cloned().collect();
                f.fehler = Some(zeilen);
            }
            Abschnitt::Datei => f.dateien.push((datei_name.to_string(), text)),
            Abschnitt::Umgebung => {
                for z in puffer.iter().filter(|z| !z.trim().is_empty()) {
                    if let Some((k, v)) = z.split_once('=') {
                        f.umgebung.push((k.trim().to_string(), v.trim().to_string()));
                    }
                }
            }
        }
        puffer.clear();
    }

    for (nr, roh) in text.lines().enumerate() {
        let zeile = roh.trim_end_matches('\r');
        if let Some(rest) = zeile.strip_prefix("=== ") {
            if let Some(f) = faelle.last_mut() { abschliessen(f, abschnitt, &mut puffer, &datei_name); }
            let name = rest.trim().to_string();
            if name.is_empty() { return Err(format!("Zeile {}: ein Fall braucht einen Namen hinter '==='", nr + 1)); }
            if faelle.iter().any(|f| f.name == name) {
                return Err(format!("Zeile {}: den Fall '{}' gibt es schon", nr + 1, name));
            }
            faelle.push(Fall { name, zeile: nr + 1, ..Default::default() });
            abschnitt = Abschnitt::Quelle;
            continue;
        }
        if faelle.is_empty() { continue; }           // Kopfkommentar vor dem ersten Fall
        if let Some(rest) = zeile.strip_prefix("--- ") {
            let f = faelle.last_mut().unwrap();
            abschliessen(f, abschnitt, &mut puffer, &datei_name);
            let (wort, arg) = match rest.trim().split_once(char::is_whitespace) {
                Some((w, a)) => (w, a.trim()),
                None => (rest.trim(), ""),
            };
            abschnitt = match wort {
                "erwartet" => Abschnitt::Erwartet,
                "enthaelt" => Abschnitt::Enthaelt,
                "fehler" => Abschnitt::Fehler,
                "umgebung" => Abschnitt::Umgebung,
                "datei" => {
                    if arg.is_empty() { return Err(format!("Zeile {}: '--- datei' braucht einen Namen", nr + 1)); }
                    datei_name = arg.to_string();
                    Abschnitt::Datei
                }
                other => return Err(format!("Zeile {}: unbekannter Abschnitt '--- {}' (erwartet, enthaelt, fehler, datei, umgebung)", nr + 1, other)),
            };
            continue;
        }
        puffer.push(zeile.to_string());
    }
    if let Some(f) = faelle.last_mut() { abschliessen(f, abschnitt, &mut puffer, &datei_name); }
    for f in &faelle {
        if f.quelle.trim().is_empty() {
            return Err(format!("Zeile {}: der Fall '{}' hat keinen Quelltext", f.zeile, f.name));
        }
    }
    Ok(faelle)
}

/// Ein Zeilenumbruch am Ende zaehlt nicht, Windows-Umbrueche auch nicht.
fn glatt(s: &str) -> String {
    let s = s.replace("\r\n", "\n");
    let s = s.strip_suffix('\n').unwrap_or(&s).to_string();
    s
}

/// Erste abweichende Zeile als lesbare Meldung.
fn unterschied(erwartet: &str, ist: &str) -> String {
    let e: Vec<&str> = erwartet.lines().collect();
    let i: Vec<&str> = ist.lines().collect();
    for n in 0..e.len().max(i.len()) {
        let a = e.get(n).copied();
        let b = i.get(n).copied();
        if a != b {
            return format!("Ausgabezeile {}: erwartet {}, erhalten {}", n + 1,
                           a.map(|s| format!("'{}'", s)).unwrap_or_else(|| "(nichts mehr)".into()),
                           b.map(|s| format!("'{}'", s)).unwrap_or_else(|| "(nichts mehr)".into()));
        }
    }
    "Ausgabe weicht ab".into()
}

/// Einen abgeschlossenen Lauf bewerten. `ohne_grafik`: ist `DHRT_OHNE_GRAFIK`
/// gesetzt, ist ein fehlender Grafik-Builtin kein Fehlschlag (Build ohne raylib).
pub fn bewerten(fall: &Fall, code: i32, stdout: &str, stderr: &str, ohne_grafik: bool) -> Ergebnis {
    if code != 0 {
        if let Some(m) = KEIN_FENSTER.iter().find(|m| stderr.contains(*m) || stdout.contains(*m)) {
            return Ergebnis::Uebersprungen(format!("kein Fenster moeglich ({})", m));
        }
        if ohne_grafik && stderr.contains("im Rust-Kern noch nicht verfuegbar") {
            return Ergebnis::Uebersprungen("Build ohne Grafik".into());
        }
    }
    let out = glatt(stdout);
    if let Some(muster) = &fall.fehler {
        if code == 0 {
            return Ergebnis::Fehl("ein Fehler war erwartet, das Programm lief durch".into());
        }
        let meldung = glatt(stderr);
        for m in muster {
            if !meldung.contains(m.as_str()) {
                return Ergebnis::Fehl(format!("Fehlermeldung sollte '{}' enthalten, war: {}", m, meldung.lines().last().unwrap_or("")));
            }
        }
    } else if code != 0 {
        let letzte = glatt(stderr);
        return Ergebnis::Fehl(format!("Rueckgabewert {}: {}", code, letzte.lines().last().unwrap_or("(keine Meldung)")));
    }
    if let Some(e) = &fall.erwartet {
        let e = glatt(e);
        if e != out { return Ergebnis::Fehl(unterschied(&e, &out)); }
    }
    for z in &fall.enthaelt {
        if !out.contains(z.as_str()) {
            return Ergebnis::Fehl(format!("Ausgabe sollte '{}' enthalten", z));
        }
    }
    Ergebnis::Ok
}

#[cfg(test)]
mod tests {
    use super::*;

    const BEISPIEL: &str = "' Kopf\n=== eins\nPRINT 1\n--- erwartet\n1\n=== zwei\nPRINT 1 \\ 0\n--- fehler\nDivision\n=== drei\nPRINT \"a\"\nPRINT \"b\"\n--- enthaelt\nb\n--- datei karte.json\n{\"x\": 1}\n--- umgebung\nDHRT_FRAMES=1\n";

    #[test]
    fn liest_faelle_und_abschnitte() {
        let f = parsen(BEISPIEL).unwrap();
        assert_eq!(f.len(), 3);
        assert_eq!(f[0].name, "eins");
        assert_eq!(f[0].zeile, 2);
        assert_eq!(f[0].quelle, "PRINT 1");
        assert_eq!(f[0].erwartet.as_deref(), Some("1"));
        assert_eq!(parsen("=== a\nPRINT 1\n--- erwartet\n1\n\n\n").unwrap()[0].erwartet.as_deref(), Some("1"));
        assert_eq!(f[1].fehler.as_ref().unwrap(), &vec!["Division".to_string()]);
        assert_eq!(f[2].enthaelt, vec!["b".to_string()]);
        assert_eq!(f[2].dateien, vec![("karte.json".to_string(), "{\"x\": 1}".to_string())]);
        assert_eq!(f[2].umgebung, vec![("DHRT_FRAMES".to_string(), "1".to_string())]);
    }

    #[test]
    fn doppelter_name_und_fehlender_quelltext_sind_fehler() {
        assert!(parsen("=== a\nPRINT 1\n=== a\nPRINT 2\n").unwrap_err().contains("gibt es schon"));
        assert!(parsen("=== a\n--- erwartet\n1\n").unwrap_err().contains("keinen Quelltext"));
        assert!(parsen("=== a\nPRINT 1\n--- sonstwas\n").unwrap_err().contains("unbekannter Abschnitt"));
        assert!(parsen("=== \nPRINT 1\n").unwrap_err().contains("Namen"));
    }

    #[test]
    fn bewerten_vergleicht_zeilenweise_und_ohne_letzten_umbruch() {
        let f = &parsen(BEISPIEL).unwrap()[0];
        assert_eq!(bewerten(f, 0, "1\n", "", false), Ergebnis::Ok);
        assert_eq!(bewerten(f, 0, "1\r\n", "", false), Ergebnis::Ok);
        assert_eq!(bewerten(f, 0, "1", "", false), Ergebnis::Ok);
        match bewerten(f, 0, "2\n", "", false) {
            Ergebnis::Fehl(m) => assert!(m.contains("Ausgabezeile 1") && m.contains("'1'") && m.contains("'2'"), "{}", m),
            r => panic!("{:?}", r),
        }
        match bewerten(f, 0, "1\n1\n", "", false) {
            Ergebnis::Fehl(m) => assert!(m.contains("(nichts mehr)"), "{}", m),
            r => panic!("{:?}", r),
        }
    }

    #[test]
    fn fehlerfall_braucht_rueckgabewert_und_meldung() {
        let f = &parsen(BEISPIEL).unwrap()[1];
        assert_eq!(bewerten(f, 1, "", "Laufzeitfehler in x.dh:1: Division durch Null\n", false), Ergebnis::Ok);
        assert!(matches!(bewerten(f, 0, "", "", false), Ergebnis::Fehl(_)));
        assert!(matches!(bewerten(f, 1, "", "etwas anderes", false), Ergebnis::Fehl(_)));
    }

    #[test]
    fn ohne_erwartung_zaehlt_der_rueckgabewert() {
        let f = &parsen("=== a\nPRINT 1\n").unwrap()[0];
        assert_eq!(bewerten(f, 0, "irgendwas\n", "", false), Ergebnis::Ok);
        assert!(matches!(bewerten(f, 3, "", "Fehler", false), Ergebnis::Fehl(_)));
    }

    #[test]
    fn kein_fenster_und_ohne_grafik_werden_uebersprungen() {
        let f = &parsen("=== a\nSCREEN(1,1,\"t\",1)\n").unwrap()[0];
        assert!(matches!(bewerten(f, 1, "", "Attempting to create window failed", false), Ergebnis::Uebersprungen(_)));
        assert!(matches!(bewerten(f, 1, "", "SCREEN im Rust-Kern noch nicht verfuegbar", true), Ergebnis::Uebersprungen(_)));
        assert!(matches!(bewerten(f, 1, "", "SCREEN im Rust-Kern noch nicht verfuegbar", false), Ergebnis::Fehl(_)));
    }
}
