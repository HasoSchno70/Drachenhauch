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
//! vor dem Lauf neben dem Programm liegt -- fuer JSON, Karten, Bilder als Text;
//! `--- datei <name> base64` fuer Bytes, die kein Text sind: cp1252-Dateien,
//! ZIP-Archive, Bilder), `--- umgebung` (`NAME=WERT` je Zeile, z. B. `DHRT_FRAMES=1`). Ohne Erwartung
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
    /// `--- erwartet ungefaehr`: Zahlen in der Ausgabe duerfen um 1e-6 (relativ
    /// oder absolut) abweichen -- fuer SIN, SQR und alles, was auf drei
    /// Betriebssystemen in der letzten Stelle anders rundet.
    pub ungefaehr: bool,
    pub enthaelt: Vec<String>,
    pub fehler: Option<Vec<String>>,
    /// Beilagen als Bytes -- Text landet unveraendert (UTF-8), ein
    /// `base64`-Block dekodiert. Bytes, nicht String, weil eine cp1252-Datei
    /// oder ein ZIP-Archiv sich als String gar nicht halten liesse.
    pub dateien: Vec<(String, Vec<u8>)>,
    /// `--- verzeichnis name`: ein (leeres) Verzeichnis neben dem Programm --
    /// fuer DIRLIST, RMDIR und alles, was Ordner sehen will.
    pub verzeichnisse: Vec<String>,
    pub umgebung: Vec<(String, String)>,
    /// `--- bild [datei]`: Punktproben an einem Bild -- ohne Namen am
    /// Bildschirmfoto nach dem Lauf (der Laeufer setzt DHRT_SCREENSHOT und,
    /// falls die Umgebung keins nennt, DHRT_FRAMES=2), mit Namen an einer
    /// Datei, die das Programm selbst geschrieben hat.
    pub bild: Option<BildPruefung>,
}

/// Was `--- bild` an einem Bild pruefen soll.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BildPruefung {
    pub datei: Option<String>,
    pub proben: Vec<Probe>,
}

/// Eine Zeile im `--- bild`-Block.
#[derive(Debug, Clone, PartialEq)]
pub enum Probe {
    /// `groesse B H`
    Groesse(u32, u32),
    /// `X Y #RRGGBB [+-N]` bzw. `X Y nicht #RRGGBB [+-N]` -- jeder Kanal darf
    /// um N abweichen (Kantenglaettung und Treiber runden verschieden).
    Farbe { x: u32, y: u32, rgb: [u8; 3], toleranz: u8, nicht: bool },
    /// `X Y = X2 Y2` bzw. `X Y <> X2 Y2` -- zwei Punkte gegeneinander, wenn
    /// die absolute Farbe egal ist (eine Kante ist da oder nicht).
    Vergleich { x: u32, y: u32, x2: u32, y2: u32, gleich: bool },
}

/// Ein dekodiertes Bild, zeilenweise RGB -- wie es der Laeufer aus der
/// PNG liest. Ohne raylib gibt es hier nichts zu dekodieren, deshalb ist die
/// Struktur schlicht und der Leser steht in main.rs hinter dem Grafik-Feature.
#[derive(Debug, Clone, PartialEq)]
pub struct Bild {
    pub breite: u32,
    pub hoehe: u32,
    pub pixel: Vec<[u8; 3]>,
}

impl Bild {
    fn punkt(&self, x: u32, y: u32) -> Result<[u8; 3], String> {
        if x >= self.breite || y >= self.hoehe {
            return Err(format!("Punkt ({}, {}) liegt ausserhalb des Bildes ({}x{})", x, y, self.breite, self.hoehe));
        }
        Ok(self.pixel[(y * self.breite + x) as usize])
    }
}

fn hex_farbe(wort: &str) -> Result<[u8; 3], String> {
    let h = wort.strip_prefix('#').ok_or_else(|| format!("Farbe '{}' muss mit # beginnen (#RRGGBB oder #RGB)", wort))?;
    let voll: String = match h.len() {
        6 => h.to_string(),
        3 => h.chars().flat_map(|c| [c, c]).collect(),
        _ => return Err(format!("Farbe '{}' muss #RRGGBB oder #RGB sein", wort)),
    };
    let n = u32::from_str_radix(&voll, 16).map_err(|_| format!("Farbe '{}' ist kein Hexwert", wort))?;
    Ok([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

fn zahl(wort: &str, was: &str) -> Result<u32, String> {
    wort.parse::<u32>().map_err(|_| format!("{} '{}' ist keine ganze Zahl", was, wort))
}

/// Eine Zeile des `--- bild`-Blocks lesen.
pub fn probe_parsen(zeile: &str) -> Result<Probe, String> {
    let w: Vec<&str> = zeile.split_whitespace().collect();
    match w.as_slice() {
        ["groesse", b, h] => Ok(Probe::Groesse(zahl(b, "Breite")?, zahl(h, "Hoehe")?)),
        [x, y, "=", x2, y2] | [x, y, "<>", x2, y2] => Ok(Probe::Vergleich {
            x: zahl(x, "x")?, y: zahl(y, "y")?, x2: zahl(x2, "x2")?, y2: zahl(y2, "y2")?, gleich: w[2] == "=",
        }),
        [x, y, rest @ ..] if !rest.is_empty() => {
            let (nicht, rest) = match rest {
                ["nicht", r @ ..] => (true, r),
                r => (false, r),
            };
            let (farbe, toleranz) = match rest {
                [f] => (hex_farbe(f)?, 0),
                [f, t] => {
                    let t = t.strip_prefix("+-").ok_or_else(|| format!("Toleranz '{}' schreibt man +-N", t))?;
                    (hex_farbe(f)?, zahl(t, "Toleranz")?.min(255) as u8)
                }
                _ => return Err(format!("Probe '{}' nicht verstanden", zeile.trim())),
            };
            Ok(Probe::Farbe { x: zahl(x, "x")?, y: zahl(y, "y")?, rgb: farbe, toleranz, nicht })
        }
        _ => Err(format!("Probe '{}' nicht verstanden (groesse B H | X Y [nicht] #RRGGBB [+-N] | X Y = X2 Y2 | X Y <> X2 Y2)", zeile.trim())),
    }
}

fn farbe_text(c: [u8; 3]) -> String { format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]) }

fn nah(a: [u8; 3], b: [u8; 3], toleranz: u8) -> bool {
    (0..3).all(|i| (a[i] as i32 - b[i] as i32).abs() <= toleranz as i32)
}

/// raylib schreibt seine eigenen Meldungen auf stdout -- etwa `WARNING:
/// AUTOMATION: ...` beim Abspielen einer Aufnahmedatei, und der Wortlaut
/// haengt an der Datei und an der raylib-Fassung. Sie sind nicht die Ausgabe
/// des Programms und zaehlen fuer `--- erwartet`/`--- enthaelt` nicht (die
/// pytest-Tests filterten dieselben Praefixe).
pub fn ohne_logzeilen(stdout: &str) -> String {
    const PRAEFIXE: [&str; 4] = ["INFO: ", "WARNING: ", "TRACE: ", "DEBUG: "];
    stdout.lines()
        .filter(|z| !PRAEFIXE.iter().any(|p| z.starts_with(p)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Alle Proben gegen das Bild; die erste, die nicht passt, ist die Meldung.
pub fn bild_pruefen(p: &BildPruefung, b: &Bild) -> Result<(), String> {
    for probe in &p.proben {
        match probe {
            Probe::Groesse(bw, bh) => {
                if (b.breite, b.hoehe) != (*bw, *bh) {
                    return Err(format!("Bild ist {}x{}, erwartet {}x{}", b.breite, b.hoehe, bw, bh));
                }
            }
            Probe::Farbe { x, y, rgb, toleranz, nicht } => {
                let ist = b.punkt(*x, *y)?;
                let trifft = nah(ist, *rgb, *toleranz);
                if trifft == *nicht {
                    let tol = if *toleranz > 0 { format!(" +-{}", toleranz) } else { String::new() };
                    return Err(if *nicht {
                        format!("Punkt ({}, {}) ist {} -- sollte nicht {}{} sein", x, y, farbe_text(ist), farbe_text(*rgb), tol)
                    } else {
                        format!("Punkt ({}, {}) ist {}, erwartet {}{}", x, y, farbe_text(ist), farbe_text(*rgb), tol)
                    });
                }
            }
            Probe::Vergleich { x, y, x2, y2, gleich } => {
                let (a, c) = (b.punkt(*x, *y)?, b.punkt(*x2, *y2)?);
                if (a == c) != *gleich {
                    return Err(format!("Punkt ({}, {}) ist {}, Punkt ({}, {}) ist {} -- erwartet {}", x, y, farbe_text(a), x2, y2, farbe_text(c), if *gleich { "gleich" } else { "verschieden" }));
                }
            }
        }
    }
    Ok(())
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
enum Abschnitt { Quelle, Erwartet, Enthaelt, Fehler, Datei, Verzeichnis, Umgebung, Bild }

/// Eine Sammlung aus ihrem Text lesen.
pub fn parsen(text: &str) -> Result<Vec<Fall>, String> {
    let mut faelle: Vec<Fall> = Vec::new();
    let mut abschnitt = Abschnitt::Quelle;
    let mut puffer: Vec<String> = Vec::new();
    let mut datei_name = String::new();
    let mut datei_b64 = false;

    fn abschliessen(f: &mut Fall, a: Abschnitt, puffer: &mut Vec<String>, datei_name: &str, datei_b64: bool) -> Result<(), String> {
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
            Abschnitt::Datei => {
                // Base64 darf umbrochen sein (76 Zeichen je Zeile ist ueblich):
                // aller Leerraum faellt vor dem Dekodieren weg.
                let bytes = if datei_b64 {
                    let dicht: String = text.split_whitespace().collect();
                    crate::builtins::b64_decode(&dicht)
                        .map_err(|e| format!("Fall '{}', Beilage '{}': kein gueltiges Base64 ({})", f.name, datei_name, e))?
                } else {
                    text.into_bytes()
                };
                f.dateien.push((datei_name.to_string(), bytes));
            }
            Abschnitt::Verzeichnis => {}      // der Name stand in der Kopfzeile, Inhalt gibt es keinen
            Abschnitt::Bild => {
                let bp = f.bild.get_or_insert_with(BildPruefung::default);
                for z in puffer.iter().map(|z| z.trim()).filter(|z| !z.is_empty() && !z.starts_with('\'')) {
                    bp.proben.push(probe_parsen(z).map_err(|e| format!("Fall '{}', --- bild: {}", f.name, e))?);
                }
            }
            Abschnitt::Umgebung => {
                for z in puffer.iter().filter(|z| !z.trim().is_empty()) {
                    if let Some((k, v)) = z.split_once('=') {
                        f.umgebung.push((k.trim().to_string(), v.trim().to_string()));
                    }
                }
            }
        }
        puffer.clear();
        Ok(())
    }

    for (nr, roh) in text.lines().enumerate() {
        let zeile = roh.trim_end_matches('\r');
        if let Some(rest) = zeile.strip_prefix("=== ") {
            if let Some(f) = faelle.last_mut() { abschliessen(f, abschnitt, &mut puffer, &datei_name, datei_b64)?; }
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
            abschliessen(f, abschnitt, &mut puffer, &datei_name, datei_b64)?;
            let (wort, arg) = match rest.trim().split_once(char::is_whitespace) {
                Some((w, a)) => (w, a.trim()),
                None => (rest.trim(), ""),
            };
            abschnitt = match wort {
                "erwartet" => {
                    if arg == "ungefaehr" { f.ungefaehr = true; }
                    else if !arg.is_empty() { return Err(format!("Zeile {}: '--- erwartet {}' kenne ich nicht (nur 'ungefaehr')", nr + 1, arg)); }
                    Abschnitt::Erwartet
                }
                "enthaelt" => Abschnitt::Enthaelt,
                "fehler" => Abschnitt::Fehler,
                "umgebung" => Abschnitt::Umgebung,
                "datei" => {
                    if arg.is_empty() { return Err(format!("Zeile {}: '--- datei' braucht einen Namen", nr + 1)); }
                    let (name, art) = match arg.split_once(char::is_whitespace) {
                        Some((n, a)) => (n, a.trim()),
                        None => (arg, ""),
                    };
                    datei_b64 = match art {
                        "" => false,
                        "base64" => true,
                        other => return Err(format!("Zeile {}: '--- datei {} {}' kenne ich nicht (nur 'base64')", nr + 1, name, other)),
                    };
                    datei_name = name.to_string();
                    Abschnitt::Datei
                }
                "verzeichnis" => {
                    if arg.is_empty() { return Err(format!("Zeile {}: '--- verzeichnis' braucht einen Namen", nr + 1)); }
                    f.verzeichnisse.push(arg.to_string());
                    Abschnitt::Verzeichnis
                }
                "bild" => {
                    if f.bild.is_some() { return Err(format!("Zeile {}: '--- bild' gibt es in diesem Fall schon", nr + 1)); }
                    f.bild = Some(BildPruefung { datei: if arg.is_empty() { None } else { Some(arg.to_string()) }, proben: Vec::new() });
                    Abschnitt::Bild
                }
                other => return Err(format!("Zeile {}: unbekannter Abschnitt '--- {}' (erwartet, enthaelt, fehler, datei, verzeichnis, umgebung, bild)", nr + 1, other)),
            };
            continue;
        }
        puffer.push(zeile.to_string());
    }
    if let Some(f) = faelle.last_mut() { abschliessen(f, abschnitt, &mut puffer, &datei_name, datei_b64)?; }
    for f in &faelle {
        if f.quelle.trim().is_empty() {
            return Err(format!("Zeile {}: der Fall '{}' hat keinen Quelltext", f.zeile, f.name));
        }
    }
    Ok(faelle)
}

/// Leerzeilen am Ende zaehlen nicht, Windows-Umbrueche auch nicht -- in einer
/// Sammlung ist die Leerzeile am Blockende der Abstand zum naechsten Fall, und
/// dieselbe Regel muss fuer die Ausgabe gelten, sonst waere ein `PRINT ""` als
/// letzte Zeile nie zu treffen.
fn glatt(s: &str) -> String {
    let s = s.replace("\r\n", "\n");
    let mut zeilen: Vec<&str> = s.split('\n').collect();
    while zeilen.last().is_some_and(|z| z.trim().is_empty()) { zeilen.pop(); }
    zeilen.join("\n")
}

/// Zeilenweise gleich, wobei Zahlen um 1e-6 abweichen duerfen. Verglichen
/// wird je Zeile Wort fuer Wort (getrennt an Leerraum und Komma); ein Wort,
/// das auf beiden Seiten als Zahl lesbar ist, zaehlt numerisch, alles andere
/// wortgleich.
fn gleich_ungefaehr(erwartet: &str, ist: &str) -> bool {
    let e: Vec<&str> = erwartet.lines().collect();
    let i: Vec<&str> = ist.lines().collect();
    if e.len() != i.len() { return false; }
    let teile = |z: &str| -> Vec<String> {
        z.split(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')' || c == '[' || c == ']')
            .filter(|s| !s.is_empty()).map(|s| s.to_string()).collect()
    };
    for (a, b) in e.iter().zip(i.iter()) {
        let (ta, tb) = (teile(a), teile(b));
        if ta.len() != tb.len() { return false; }
        for (x, y) in ta.iter().zip(tb.iter()) {
            if x == y { continue; }
            match (x.parse::<f64>(), y.parse::<f64>()) {
                (Ok(p), Ok(q)) => {
                    let tol = 1e-6_f64.max(p.abs().max(q.abs()) * 1e-6);
                    if (p - q).abs() > tol { return false; }
                }
                _ => return false,
            }
        }
    }
    true
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
    let out = glatt(&ohne_logzeilen(stdout));
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
        let gleich = if fall.ungefaehr { gleich_ungefaehr(&e, &out) } else { e == out };
        if !gleich { return Ergebnis::Fehl(unterschied(&e, &out)); }
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
        assert_eq!(f[2].dateien, vec![("karte.json".to_string(), "{\"x\": 1}".to_string().into_bytes())]);
        assert_eq!(f[2].umgebung, vec![("DHRT_FRAMES".to_string(), "1".to_string())]);
        let v = parsen("=== a\nPRINT 1\n--- verzeichnis leer/tief\n--- erwartet\n1\n").unwrap();
        assert_eq!(v[0].verzeichnisse, vec!["leer/tief".to_string()]);
        assert_eq!(v[0].erwartet.as_deref(), Some("1"));
    }

    #[test]
    fn raylib_meldungen_zaehlen_nicht_als_ausgabe() {
        let f = parsen("=== a\nPRINT 1\n--- erwartet\n1\n2\n").unwrap();
        let out = "WARNING: AUTOMATION: [ev.txt] Issue reading line to buffer\n1\nINFO: IMAGE: Data loaded successfully\n2\n";
        assert_eq!(bewerten(&f[0], 0, out, "", false), Ergebnis::Ok);
        assert_eq!(ohne_logzeilen("a\nTRACE: x\nb"), "a\nb");
        // Eine Leerzeile am Ende eines datei-Blocks ist der Endumbruch der Datei.
        let f = parsen("=== a\nPRINT 1\n--- datei ev.txt\nc 1\n\n--- erwartet\n1\n").unwrap();
        assert_eq!(f[0].dateien[0].1, b"c 1\n".to_vec());
        let f = parsen("=== a\nPRINT 1\n--- datei ev.txt\nc 1\n--- erwartet\n1\n").unwrap();
        assert_eq!(f[0].dateien[0].1, b"c 1".to_vec());
    }

    #[test]
    fn bildproben_lesen_und_pruefen() {
        let f = parsen("=== a\nSCREEN(4, 4)\n--- bild\ngroesse 4 4\n1 1 #FF0000\n2 2 nicht #F00\n0 0 #102030 +-16\n1 1 <> 0 0\n' Kommentar\n1 1 = 1 1\n").unwrap();
        let bp = f[0].bild.as_ref().unwrap();
        assert_eq!(bp.datei, None);
        assert_eq!(bp.proben.len(), 6);
        assert_eq!(bp.proben[1], Probe::Farbe { x: 1, y: 1, rgb: [255, 0, 0], toleranz: 0, nicht: false });
        assert_eq!(bp.proben[2], Probe::Farbe { x: 2, y: 2, rgb: [255, 0, 0], toleranz: 0, nicht: true });
        assert_eq!(bp.proben[3], Probe::Farbe { x: 0, y: 0, rgb: [16, 32, 48], toleranz: 16, nicht: false });
        // 4x4: (1,1) rot, sonst (20,40,60)
        let mut px = vec![[20u8, 40, 60]; 16];
        px[5] = [255, 0, 0];
        let b = Bild { breite: 4, hoehe: 4, pixel: px };
        assert_eq!(bild_pruefen(bp, &b), Ok(()));
        // Toleranz ueberschritten: (0,0) ist (20,40,60), erwartet (16,32,48) +-8 -> Blau weicht um 12 ab
        let eng = BildPruefung { datei: None, proben: vec![probe_parsen("0 0 #102030 +-8").unwrap()] };
        assert!(bild_pruefen(&eng, &b).unwrap_err().contains("erwartet #102030 +-8"));
        let nicht = BildPruefung { datei: None, proben: vec![probe_parsen("1 1 nicht #FF0000").unwrap()] };
        assert!(bild_pruefen(&nicht, &b).unwrap_err().contains("sollte nicht"));
        let raus = BildPruefung { datei: None, proben: vec![probe_parsen("9 0 #000000").unwrap()] };
        assert!(bild_pruefen(&raus, &b).unwrap_err().contains("ausserhalb"));
        let gr = BildPruefung { datei: None, proben: vec![Probe::Groesse(5, 4)] };
        assert!(bild_pruefen(&gr, &b).unwrap_err().contains("erwartet 5x4"));
        // Datei statt Bildschirmfoto, und Fehler im Block sind Dateifehler
        assert_eq!(parsen("=== a\nPRINT 1\n--- bild out.png\n0 0 #000\n").unwrap()[0].bild.as_ref().unwrap().datei.as_deref(), Some("out.png"));
        assert!(parsen("=== a\nPRINT 1\n--- bild\n0 0 rot\n").unwrap_err().contains("muss mit #"));
        assert!(parsen("=== a\nPRINT 1\n--- bild\nirgendwas\n").unwrap_err().contains("nicht verstanden"));
    }

    #[test]
    fn beilage_als_base64_wird_zu_bytes() {
        // "Köln" in cp1252: 4B F6 6C 6E -- als UTF-8-Text nicht darstellbar,
        // genau der Fall, fuer den es den Block gibt. Umbruch im Block ist erlaubt.
        let f = parsen("=== a\nPRINT 1\n--- datei alt.txt base64\nS/Zs\nbg==\n--- erwartet\n1\n").unwrap();
        assert_eq!(f[0].dateien, vec![("alt.txt".to_string(), vec![0x4B, 0xF6, 0x6C, 0x6E])]);
        assert_eq!(f[0].erwartet.as_deref(), Some("1"));
        assert!(parsen("=== a\nPRINT 1\n--- datei x.bin base64\n!!!\n").unwrap_err().contains("kein gueltiges Base64"));
        assert!(parsen("=== a\nPRINT 1\n--- datei x.bin hex\nff\n").unwrap_err().contains("nur 'base64'"));
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
        assert_eq!(bewerten(f, 0, "1\n\n\n", "", false), Ergebnis::Ok);   // PRINT "" am Ende
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
    fn ungefaehr_laesst_zahlen_in_der_letzten_stelle_durch() {
        let f = &parsen("=== a\nPRINT SIN(1.0)\n--- erwartet ungefaehr\n0.8414709848078965 x (1, 2.5)\n").unwrap()[0];
        assert!(f.ungefaehr);
        assert_eq!(bewerten(f, 0, "0.8414709848078966 x (1, 2.5000000001)\n", "", false), Ergebnis::Ok);
        assert!(matches!(bewerten(f, 0, "0.8414 x (1, 2.5)\n", "", false), Ergebnis::Fehl(_)));
        assert!(matches!(bewerten(f, 0, "0.8414709848078965 y (1, 2.5)\n", "", false), Ergebnis::Fehl(_)));
        assert!(matches!(bewerten(f, 0, "0.8414709848078965 x (1, 2.5)\nmehr\n", "", false), Ergebnis::Fehl(_)));
        assert!(parsen("=== a\nPRINT 1\n--- erwartet genau\n1\n").unwrap_err().contains("kenne ich nicht"));
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
