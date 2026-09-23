//! Hinweise fuer Umsteiger aus QBasic, FreeBASIC, VB, Blitz und Python.
//!
//! Wer aus einem anderen BASIC kommt, schreibt `UCASE$`, `DIM d AS DOUBLE`
//! oder `r = RND` -- und bekam bisher Meldungen, die sagen, DASS etwas nicht
//! geht, aber nicht, wie es hier heisst. Diese Tabellen sagen es. Sie sind
//! rein (keine VM, kein Compiler), damit Compiler-Warnung und Laufzeitfehler
//! denselben Satz sagen und sich beides ohne Programm pruefen laesst.

/// Befehle anderer BASICs (und haeufiger Sprachen), die hier anders heissen.
/// `name` kleingeschrieben, wie der Compiler ihn sieht.
pub fn befehl(name: &str) -> Option<&'static str> {
    Some(match name {
        "ucase$" | "ucase" | "toupper" | "upper" => "heisst in Drachenhauch UPPER$(text)",
        "lcase$" | "lcase" | "tolower" | "lower" => "heisst in Drachenhauch LOWER$(text)",
        "string$" => "heisst in Drachenhauch REPEAT$(text, anzahl) -- oder \"*\" * 3",
        "lbound" => "Felder beginnen immer bei 0 -- LBOUND ist also 0",
        "ubound" => "der letzte Index ist LEN(feld) - 1 (Felder beginnen bei 0)",
        "fix" => "INT rundet ab (INT(-3.7) = -4); in Richtung 0: IIF(x < 0, -INT(-x), INT(x))",
        "cint" | "clng" | "cdbl" | "csng" | "cbool" =>
            "Umwandeln: INT(x) oder ROUND(x) fuer Ganzzahlen, FLT(x) fuer Kommazahlen, VAL(text) aus Text",
        "cstr" | "str" | "tostring" => "heisst in Drachenhauch STR$(wert)",
        "strlen" | "length" | "size" | "count" => "heisst in Drachenhauch LEN(wert)",
        "substr" | "substring" | "mid" => "heisst in Drachenhauch MID$(text, start, laenge) -- start zaehlt ab 0",
        "sqrt" => "heisst in Drachenhauch SQR(x)",
        "random" | "rand" => "heisst in Drachenhauch RND() (0 bis unter 1) -- ganze Zahl 1..6: INT(RND() * 6) + 1",
        "println" | "printf" | "echo" => "Ausgabe mit der Anweisung PRINT: PRINT \"Hallo\"; x",
        "wait" | "delay" => "heisst in Drachenhauch SLEEP(millisekunden)",
        "strcmp" => "Texte vergleicht man direkt: IF a = b THEN, a < b sortiert",
        "chr" => "heisst in Drachenhauch CHR$(code)",
        "hex" => "heisst in Drachenhauch HEX$(zahl)",
        "space" => "heisst in Drachenhauch SPACE$(anzahl)",
        "trim" => "heisst in Drachenhauch TRIM$(text)",
        "left" | "right" => "heisst in Drachenhauch LEFT$(text, n) bzw. RIGHT$(text, n)",
        "rtrim" | "ltrim" => "heisst in Drachenhauch RTRIM$(text) bzw. LTRIM$(text)",
        "append" | "push" => "ans Feld anhaengen: ARRAY_PUSH(feld, wert)",
        "bitand" => "ist in Drachenhauch der Operator BAND: a BAND b",
        "bitor" => "ist in Drachenhauch der Operator BOR: a BOR b",
        "bitxor" => "ist in Drachenhauch der Operator BXOR: a BXOR b",
        "bitnot" => "ist in Drachenhauch der Operator BNOT: BNOT a",
        "tab" => "gibt es als Befehl nicht -- Tabulator: CHR$(9), feste Breite: PADR$(text, n)",
        "spc" => "heisst in Drachenhauch SPACE$(anzahl)",
        "array_remove" | "array_delete" | "remove" => "heisst in Drachenhauch ARRAY_REMOVE_AT(feld, index)",
        "pset" => "heisst in Drachenhauch PLOT(x, y, farbe)",
        "color" => "eine Farbe ist ein Argument jedes Zeichenbefehls: BOX(x1, y1, x2, y2, RED), TEXT(x, y, text$, YELLOW)",
        "keydown" | "keyisdown" | "iskeydown" => "gehalten: KEYPRESSED(KEY_A), gerade gedrueckt: KEYHIT(KEY_A)",
        "locate" => "Text an eine Stelle: TEXT(x, y, text$) im Fenster",
        "trunc" | "cint2" => "INT(x) rundet ab; Richtung 0: IIF(x < 0, -INT(-x), INT(x))",
        "zeit_jahr" | "zeit_monat" | "zeit_tag" | "zeit_stunde" | "zeit_minute" | "zeit_sekunde" =>
            "ein Teil einer Zeit kommt aus ZEIT_TEIL(zeit, \"jahr\") -- oder ZEIT_FORMAT$(zeit, muster$)",
        "gui_window_new" | "gui_new_window" => "heisst GUI_WINDOW(titel$, x, y, breite, hoehe)",
        "stop" | "system" | "quit" => "das Programm beenden: END (allein auf der Zeile) oder EXIT(0)",
        _ => return None,
    })
}

/// Typnamen anderer Sprachen.
pub fn typ(name: &str) -> Option<&'static str> {
    Some(match name {
        "double" | "single" | "real" | "decimal" | "number" | "f32" | "f64" =>
            "Kommazahlen heissen in Drachenhauch FLOAT (immer 64 Bit)",
        "long" | "short" | "byte" | "int" | "longint" | "uinteger" | "ulong" | "i32" | "i64" =>
            "ganze Zahlen heissen in Drachenhauch INTEGER (immer 64 Bit)",
        "bool" => "heisst in Drachenhauch BOOLEAN",
        "str" | "text" | "char" | "zstring" => "Text heisst in Drachenhauch STRING",
        "variant" | "object" =>
            "einen Typ fuer alles gibt es nicht -- einen festen Typ waehlen (INTEGER, FLOAT, STRING, BOOLEAN, eine Klasse)",
        "list" | "vector" => "Listen sind Felder: DIM a AS ARRAY OF INTEGER (waechst mit ARRAY_PUSH)",
        "dict" | "dictionary" | "hashmap" => "Woerterbuecher sind MAPs: DIM m AS MAP OF INTEGER",
        _ => {
            if name.starts_with("array:") {
                return Some("Felder von Feldern gibt es nicht -- zweidimensional: DIM g[3, 4] AS INTEGER");
            }
            return None;
        }
    })
}

/// Befehle ohne Argumente, die man aus anderen BASICs OHNE Klammern kennt
/// (`r = RND`, `t = TIMER`). Hier ist jeder Aufruf mit Klammern.
pub fn ohne_klammern(name: &str) -> Option<String> {
    let n = match name {
        "rnd" | "timer" | "inkey$" | "millis" | "date$" | "time$" | "cls" | "flip"
        | "screenwidth" | "screenheight" | "mousex" | "mousey" | "fps" | "delta" => name,
        _ => return None,
    };
    Some(format!("{} ist ein Befehl -- aufgerufen wird er mit Klammern: {}()", n.to_uppercase(), n.to_uppercase()))
}

/// Anhang fuer "Variable 'x' nicht deklariert": was der Name vermutlich
/// meinte. Leer, wenn nichts Besonderes dahintersteckt.
pub fn name_hinweis(name: &str) -> String {
    let n = name.to_lowercase();
    if let Some(h) = ohne_klammern(&n) { return format!(" -- {}", h); }
    if let Some(h) = befehl(&n) { return format!(" -- {}", h); }
    match n.as_str() {
        "true" | "false" | "nil" => String::new(),
        "none" | "null" | "nothing" => " -- 'kein Wert' heisst in Drachenhauch NIL".into(),
        "me" | "this" => " -- die eigene Instanz heisst in Drachenhauch Self (Self.x = 3)".into(),
        _ => String::new(),
    }
}

/// Sieht dieser Text nach einer Datei aus, die man erst LADEN muss?
///
/// Gebraucht dort, wo ein Bild-/Klang-/Schrift-HANDLE erwartet wird und ein
/// Text ankommt: `PLAYSOUND("sprung.wav")` ist die Zeile, die jeder aus einem
/// anderen BASIC zuerst schreibt, und "erwartet Zahl, erhalten STRING" sagt
/// ihm nichts. Liefert (Art, Ladebefehl).
pub fn lader_fuer_datei(text: &str) -> Option<(&'static str, &'static str)> {
    let t = text.to_lowercase();
    let endung = t.rsplit_once('.').map(|(_, e)| e)?;
    Some(match endung {
        "png" | "jpg" | "jpeg" | "bmp" | "gif" | "tga" | "qoi" => ("Bild", "LOADIMAGE"),
        "wav" | "ogg" | "mp3" | "flac" | "qoa" => ("Klang", "LOADSOUND"),
        "ttf" | "otf" => ("Schrift", "LOADFONT"),
        _ => return None,
    })
}

/// Fertiger Anhang fuer genau diesen Fall, oder leer.
pub fn datei_statt_handle(befehl: &str, text: &str) -> String {
    match lader_fuer_datei(text) {
        Some((art, lader)) => format!(
            " -- ein Dateiname ist noch kein {art}: erst `x = {lader}(\"{text}\")`, dann `{}(x, ...)`",
            befehl.to_uppercase()),
        None => String::new(),
    }
}

/// Ein TEXT, wo eine Zahl hingehoert: der Anhang, der sagt WARUM.
///
/// Drei Faelle, alle gemessen an dem, was man aus anderen BASICs mitbringt:
/// eine Taste als Buchstabe (`KEYHIT("a")` -- dort vergleicht man INKEY$),
/// eine Farbe als Hex-Text (`RGB("FF0000")`) und ein Dateiname, wo ein
/// geladenes Bild/ein geladener Klang hingehoert.
pub fn text_statt_zahl(befehl: &str, text: &str) -> String {
    let b = befehl.to_lowercase();
    if matches!(b.as_str(), "keyhit" | "keypressed" | "keyreleased" | "keyrepeat")
        && text.chars().count() == 1 {
        let c = text.chars().next().unwrap_or('a');
        return format!(
            " -- eine Taste ist eine ZAHL: {}(ASC(\"{}\")) oder die Konstante KEY_{}",
            b.to_uppercase(), c, c.to_uppercase());
    }
    if (b == "rgb" || b == "rgba")
        && text.trim_start_matches('#').chars().all(|c| c.is_ascii_hexdigit())
        && !text.is_empty() {
        return format!(
            " -- eine Farbe aus Hex-Text kommt aus COLOR_FROM_HEX(\"{}\"); im Quelltext geht auch &H{}", text, text.trim_start_matches('#').to_uppercase());
    }
    datei_statt_handle(befehl, text)
}

/// Anhang fuer einen unbekannten Befehl.
pub fn befehl_hinweis(name: &str) -> String {
    let n = name.to_lowercase();
    match befehl(&n) {
        Some(h) => format!(" -- {}", h),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bekannte_umsteiger_namen() {
        assert!(befehl("ucase$").unwrap().contains("UPPER$"));
        assert!(befehl("ubound").unwrap().contains("LEN"));
        assert!(befehl("printf").unwrap().contains("PRINT"));
        assert!(befehl("gibtsnicht").is_none());
        assert!(typ("double").unwrap().contains("FLOAT"));
        assert!(typ("long").unwrap().contains("INTEGER"));
        assert!(typ("array:integer").unwrap().contains("g[3, 4]"));
        assert!(typ("integer").is_none());
    }

    #[test]
    fn dateiname_statt_handle() {
        assert_eq!(lader_fuer_datei("held.PNG"), Some(("Bild", "LOADIMAGE")));
        assert_eq!(lader_fuer_datei("sprung.wav"), Some(("Klang", "LOADSOUND")));
        assert_eq!(lader_fuer_datei("schrift.ttf"), Some(("Schrift", "LOADFONT")));
        // Kein Dateiname, und darum auch kein Hinweis: sonst stuende er unter
        // jedem Text, der zufaellig an der Stelle einer Zahl steht.
        assert_eq!(lader_fuer_datei("links"), None);
        assert_eq!(lader_fuer_datei("3.5"), None);
        assert!(datei_statt_handle("playsound", "sprung.wav").contains("LOADSOUND"));
        assert!(datei_statt_handle("playsound", "sprung.wav").contains("PLAYSOUND(x, ...)"));
        assert!(datei_statt_handle("box", "links").is_empty());
    }

    #[test]
    fn text_wo_eine_zahl_hingehoert() {
        assert!(text_statt_zahl("keyhit", "a").contains("ASC(\"a\")"));
        assert!(text_statt_zahl("keyhit", "a").contains("KEY_A"));
        // Mehr als ein Zeichen ist keine Taste -- dann kein Tasten-Satz.
        assert!(text_statt_zahl("keyhit", "leertaste").is_empty());
        assert!(text_statt_zahl("rgb", "FF0000").contains("COLOR_FROM_HEX"));
        assert!(text_statt_zahl("rgb", "#f80").contains("&HF80"));
        assert!(text_statt_zahl("rgb", "rot").is_empty());
        // Der Datei-Fall laeuft weiter durch.
        assert!(text_statt_zahl("playsound", "a.wav").contains("LOADSOUND"));
    }

    #[test]
    fn hinweise_fuer_namen() {
        assert!(name_hinweis("TIMER").contains("TIMER()"));
        assert!(name_hinweis("rnd").contains("RND()"));
        assert!(name_hinweis("stop").contains("END"));
        assert!(name_hinweis("zaehler").is_empty());
        assert!(befehl_hinweis("LCASE$").contains("LOWER$"));
        assert!(befehl_hinweis("meinding").is_empty());
    }
}
