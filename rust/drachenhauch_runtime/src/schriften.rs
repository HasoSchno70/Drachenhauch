//! Die eingebauten Schriften: DejaVu Sans, Serif und Sans Mono in je vier
//! Schnitten (Crate `dejavu`, rund 5,3 MB). Das pdf-Modul bettet sie ein,
//! LOADFONT laedt sie unter denselben Namen -- auch im Web-Bau, der das
//! pdf-Modul nicht hat (Feature `schriften`, Messdaten-/Web-Runde 2026-10-13).

/// Noto Music (SIL Open Font License, daten/schriften/OFL-NotoMusic.txt):
/// die Notenzeichen des Unicode-Blocks "Musical Symbols" -- Schluessel,
/// Notenkoepfe, Pausen, Vorzeichen. 124 KB. Fuer `LOADFONT("noten", n,
/// zeichen$)`; ohne Zeichenliste backt LOADFONT nur den Grundvorrat, und den
/// hat die Notenschrift kaum (die fehlenden Zeichen laedt FLIP nach).
fn noten() -> &'static [u8] { include_bytes!("../../../daten/schriften/NotoMusic-Regular.ttf") }

/// Die eingebauten Schriften: Name im Programm und Dateiinhalt.
pub const EINGEBAUT: [(&str, fn() -> &'static [u8]); 13] = [
    ("sans", dejavu::sans::regular),
    ("sans-fett", dejavu::sans::bold),
    ("sans-kursiv", dejavu::sans::oblique),
    ("sans-fett-kursiv", dejavu::sans::bold_oblique),
    ("serif", dejavu::serif::regular),
    ("serif-fett", dejavu::serif::bold),
    ("serif-kursiv", dejavu::serif::italic),
    ("serif-fett-kursiv", dejavu::serif::bold_italic),
    ("mono", dejavu::sans_mono::regular),
    ("mono-fett", dejavu::sans_mono::bold),
    ("mono-kursiv", dejavu::sans_mono::oblique),
    ("mono-fett-kursiv", dejavu::sans_mono::bold_oblique),
    ("noten", noten),
];

/// Die Bytes einer eingebauten Schrift ("sans", "serif-fett", "mono", auch
/// die alten Namen helvetica/times/courier) -- auch fuer LOADFONT: so hat ein
/// Programm auf jedem System eine richtige Schrift, ohne einen Pfad zu raten.
pub fn eingebaute_schrift(name: &str) -> Option<&'static [u8]> {
    let n = normalisieren(name);
    EINGEBAUT.iter().find(|(k, _)| *k == n).map(|(_, f)| f())
}

/// Die Namen von vorher meinen die passende DejaVu-Familie.
pub const ALTE_NAMEN: [(&str, &str); 3] = [("helvetica", "sans"), ("times", "serif"), ("courier", "mono")];

pub fn normalisieren(name: &str) -> String {
    let k = name.trim().to_lowercase();
    for (alt, neu) in ALTE_NAMEN {
        if k == alt { return neu.to_string(); }
        if let Some(rest) = k.strip_prefix(alt).and_then(|r| r.strip_prefix('-')) {
            return format!("{}-{}", neu, rest);
        }
    }
    k
}
