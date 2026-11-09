//! Die Grafik-Befehle (`try_graphics`): Fenster, Zeichnen, Texte, Bilder, Eingabe, Audio, 3D und alle Module, die ein Fenster brauchen.
//!
//! Teil von `vm.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl<'p> Vm<'p> {
    #[cfg(feature = "graphics")]
    pub(super) fn try_graphics(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        // Grafik-Koordinaten sind "intish": INTEGER direkt, FLOAT wird zu int
        // trunkiert (Richtung 0, wie Pythons int()). Der Tree-Walker macht das
        // gleiche -- seine Zeichenprimitive (PLOT/LINE/BOX/RECT/CIRCLE/GRADIENT*)
        // pruefen via `_check_int`, das in interpreter.py ein Alias auf
        // `_check_intish` ist (interpreter.py:3541). So sind F5 (Tree-Walker) und
        // F6 (dhrt) bei Float-Koordinaten konsistent (z.B. LINE(10.5, ...)).
        fn gi(a: &[Value], i: usize, fn_: &str) -> R<i64> {
            match a.get(i) {
                Some(Value::Int(n)) => Ok(*n),
                Some(Value::Float(f)) => Ok(*f as i64),
                // NIL ist hier fast immer ein Bild/Klang/Handle, das nie
                // geladen wurde -- "erwartet Zahl" fuehrte in die Irre.
                Some(Value::Nil) => Err(format!(
                    "{}: Argument {} ist NIL -- die Variable hat noch keinen Wert (Bild/Klang vorher laden, z.B. b = LOADIMAGE(\"bild.png\"))",
                    fn_, i + 1)),
                Some(Value::Str(s)) => Err(format!(
                    "{}: erwartet Zahl, erhalten STRING (Argument {}){}",
                    fn_, i + 1, crate::umstieg::text_statt_zahl(fn_, s))),
                Some(v) => Err(format!("{}: erwartet Zahl, erhalten {} (Argument {})", fn_, v.type_name(), i + 1)),
                None => Err(format!("{}: fehlendes Argument {}", fn_, i + 1)),
            }
        }
        // Tastenargument von KEYHIT & Co.: eine Zahl ist die LAGE der Taste,
        // ein einzelner Buchstabe (oder eine Ziffer) die Taste, auf der er steht.
        fn taste(g: &crate::graphics::Graphics, a: &[Value], fn_: &str) -> R<i64> {
            if let Some(Value::Str(s)) = a.first() {
                let mut z = s.chars();
                if let (Some(c), None) = (z.next(), z.next()) {
                    if let Some(k) = g.taste_fuer_zeichen(c) { return Ok(k); }
                }
                return Err(format!(
                    "{}: ein Text meint den Buchstaben oder die Ziffer auf der Taste -- \"{}\" ist keins (Sondertasten als Konstante: KEY_ENTER, KEY_LEFT, ...)",
                    fn_, s));
            }
            gi(a, 0, fn_)
        }
        fn gs<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            match a.get(i) {
                Some(Value::Str(s)) => Ok(s),
                _ => Err(format!("{}: erwartet STRING", fn_)),
            }
        }
        fn gb(a: &[Value], i: usize) -> bool {
            matches!(a.get(i), Some(Value::Bool(true)))
        }
        // Flag-Argument: akzeptiert TRUE/FALSE UND 1/0 (wie need_flag); fehlend = false.
        fn gflag(a: &[Value], i: usize) -> bool {
            match a.get(i) {
                Some(Value::Bool(b)) => *b,
                Some(Value::Int(n)) => *n != 0,
                Some(Value::Float(f)) => *f != 0.0,
                _ => false,
            }
        }
        fn need_f(a: &[Value], i: usize, fn_: &str) -> R<f64> {
            match a.get(i) {
                Some(Value::Int(n)) => Ok(*n as f64),
                Some(Value::Float(f)) => Ok(*f),
                _ => Err(format!("{}: Argument {} muss Zahl sein", fn_, i + 1)),
            }
        }
        // Farbe fuer die Leinwand-Befehle (IMAGE_FILL_* & Co.). Ohne Deckkraft
        // gilt die Farbe wie ueberall (oberstes Byte 0 = deckend); mit ihr
        // bestimmt allein die Deckkraft -- auch 0, was sich als Farbe nicht
        // schreiben laesst.
        fn lw_farbe(a: &[Value], i: usize, deck: Option<usize>, fn_: &str) -> R<[u8; 4]> {
            let v = match a.get(i) {
                Some(Value::Int(n)) => *n,
                _ => return Err(format!("{}: Argument {} muss eine Farbe sein", fn_, i + 1)),
            };
            let u = v as u32;
            let al = ((u >> 24) & 0xFF) as u8;
            let mut c = [((u >> 16) & 0xFF) as u8, ((u >> 8) & 0xFF) as u8, (u & 0xFF) as u8,
                         if al == 0 { 255 } else { al }];
            if let Some(d) = deck.and_then(|k| a.get(k)) {
                let d = match d {
                    Value::Int(n) => *n,
                    Value::Float(f) => *f as i64,
                    _ => return Err(format!("{}: die Deckkraft muss eine Zahl sein", fn_)),
                };
                if !(0..=255).contains(&d) {
                    return Err(format!("{}: die Deckkraft muss 0..255 sein, war {}", fn_, d));
                }
                c[3] = d as u8;
            }
            Ok(c)
        }
        // Genau `n` Zahlen-Argumente als f32 (fuer die koordinatenreichen
        // Geometrie-Builtins -- 15 einzelne need_f-Zeilen liest niemand mehr).
        fn fv(a: &[Value], n: usize, fn_: &str) -> R<Vec<f32>> {
            if a.len() != n {
                return Err(format!("{}: erwartet {} Zahlen, erhalten {}", fn_, n, a.len()));
            }
            (0..n).map(|i| Ok(need_f(a, i, fn_)? as f32)).collect()
        }
        // Optionaler Easing-Name fuer Fades/Slides (linear/in/out/inout,
        // Default "linear" bei fehlendem Argument). Pruefung VOR der Audio-
        // Initialisierung -> golden-testbar (wie die Bus-Namen-Checks).
        fn easing_name(a: &[Value], i: usize, fn_: &str) -> R<String> {
            let name = if a.len() > i { gs(a, i, fn_)?.to_lowercase() } else { "linear".to_string() };
            if !matches!(name.as_str(), "linear" | "in" | "out" | "inout") {
                return Err(format!("{}: unbekanntes Easing '{}' (linear, in, out, inout)", fn_, name));
            }
            Ok(name)
        }
        // 1D-INTEGER-Array -> Vec<i32> (fuer Bulk-Draws).
        fn arr_i32(v: &Value, fn_: &str) -> R<Vec<i32>> {
            match v {
                Value::Array(a) => {
                    let a = a.borrow();
                    if let Some(ints) = a.cells.as_ints() {
                        return Ok(ints.iter().map(|&i| i as i32).collect());
                    }
                    let mut o = Vec::with_capacity(a.cells.len());
                    for x in a.cells.iter() {
                        match x { Value::Int(i) => o.push(i as i32), _ => return Err(format!("{}: ARRAY OF INTEGER noetig", fn_)) }
                    }
                    Ok(o)
                }
                _ => Err(format!("{}: ARRAY noetig", fn_)),
            }
        }
        // color-Arg fuer Bulk: INTEGER (alle gleich) oder ARRAY OF INTEGER.
        /// Optionale Stueckzahl der Bulk-Zeichenbefehle lesen.
        ///
        /// Ohne sie zeichnen sie das GANZE Array -- wer einen Puffer fest
        /// dimensioniert und pro Bild nur teilweise fuellt, bekam die alten
        /// Werte der restlichen Plaetze mitgezeichnet. Zusaetzlich wird hier
        /// ein Argument zu viel abgelehnt: vorher wurde es still ignoriert,
        /// eine mitgegebene Stueckzahl also wirkungslos verschluckt.
        fn bulk_count(a: &[Value], idx: usize, n: usize, fn_: &str) -> R<usize> {
            if a.len() > idx + 1 {
                return Err(format!("{}: zu viele Argumente ({} statt hoechstens {})",
                                   fn_, a.len(), idx + 1));
            }
            match a.get(idx) {
                None => Ok(n),
                Some(v) => {
                    let c = match v {
                        Value::Int(i) => *i,
                        Value::Float(f) if f.fract() == 0.0 => *f as i64,
                        _ => return Err(format!("{}: anzahl muss eine ganze Zahl sein", fn_)),
                    };
                    if c < 0 { return Err(format!("{}: anzahl darf nicht negativ sein", fn_)); }
                    Ok((c as usize).min(n))
                }
            }
        }

        fn bulk_color(v: &Value, n: usize, fn_: &str) -> R<Vec<i64>> {
            match v {
                Value::Int(c) => Ok(vec![*c; n]),
                Value::Array(a) => {
                    let a = a.borrow();
                    // >= statt ==: mit einer Stueckzahl darf das Farb-Array so
                    // lang bleiben wie der Puffer, auch wenn weniger gezeichnet wird.
                    if a.cells.len() < n { return Err(format!("{}: colors-Array muss mindestens so lang wie die Stueckzahl sein", fn_)); }
                    let mut o = Vec::with_capacity(n);
                    if let Some(ints) = a.cells.as_ints() { return Ok(ints[..n].to_vec()); }
                    for x in a.cells.iter().take(n) { match x { Value::Int(i) => o.push(i), _ => return Err(format!("{}: color-ARRAY OF INTEGER noetig", fn_)) } }
                    Ok(o)
                }
                _ => Err(format!("{}: color muss INTEGER oder ARRAY sein", fn_)),
            }
        }
        // Grafik-Builtins lazy-initialisieren ein verstecktes Fenster, wenn noch
        // kein SCREEN aufgerufen wurde -- so funktionieren headless Bild-/Kamera-/
        // Sprite-Ops (LOADIMAGE, imgfx, CAMERA_*, SPRITE_*) ohne sichtbares Fenster,
        // wie das pygame-Lazy-Init im Tree-Walker. Ein spaeteres SCREEN macht das
        // Fenster sichtbar (reconfigure), statt ein zweites zu erzeugen.
        macro_rules! g {
            () => {{
                if self.gfx.is_none() {
                    // Scheitert das Fenster, ist es ein Fehler DIESES Befehls.
                    self.gfx = Some(crate::graphics::Graphics::new_headless()
                        .map_err(|e| format!("{}: {}", name.to_uppercase(), e))?);
                }
                self.gfx.as_mut().unwrap()
            }};
        }
        // Zeichnen, ohne dass es je ein SCREEN gab: das haette ein
        // VERSTECKTES Fenster angelegt (g!(), fuer LOADIMAGE/imgfx) und still
        // hineingemalt -- kein Fehler, kein Bild, kein Wort. Fuer den ersten
        // Versuch mit Grafik ist das die teuerste Falle ueberhaupt: das
        // Programm laeuft, und man sucht den Fehler im eigenen Code.
        // Innerhalb eines Render-Ziels ist es erlaubt -- dort zeichnet man
        // absichtlich neben den Schirm.
        if ist_schirmbefehl(name)
            && self.gfx.as_ref().is_none_or(|g| !g.schirm_bereit()) {
            return Err(format!(
                "{}: es gibt noch kein Fenster -- vor dem ersten Zeichenbefehl SCREEN(breite, hoehe, \"Titel\") aufrufen",
                name.to_uppercase()));
        }
        let r = match name {
            "screen" => {
                let w = gi(a, 0, "SCREEN")? as i32;
                let h = gi(a, 1, "SCREEN")? as i32;
                let title = if a.len() >= 3 { gs(a, 2, "SCREEN")?.to_string() } else { "Drachenhauch".to_string() };
                let scale = if a.len() >= 4 { gi(a, 3, "SCREEN")? as i32 } else { 1 };
                if scale < 1 { return Err("SCREEN: skala muss >= 1 sein".into()); }
                match self.gfx.as_mut() {
                    Some(gfx) => gfx.reconfigure(w, h, &title, scale),
                    None => self.gfx = Some(crate::graphics::Graphics::new(w, h, &title, scale)
                        .map_err(|e| format!("SCREEN: {}", e))?),
                }
                Value::Nil
            }
            "screen_native" => {
                let title = if !a.is_empty() { gs(a, 0, "SCREEN_NATIVE")?.to_string() } else { "Drachenhauch".to_string() };
                g!().screen_native(&title);
                Value::Nil
            }
            "screen_transparent" => {
                let w = gi(a, 0, "SCREEN_TRANSPARENT")? as i32;
                let h = gi(a, 1, "SCREEN_TRANSPARENT")? as i32;
                let title = if a.len() >= 3 { gs(a, 2, "SCREEN_TRANSPARENT")?.to_string() } else { "Drachenhauch".to_string() };
                let scale = if a.len() >= 4 { gi(a, 3, "SCREEN_TRANSPARENT")? as i32 } else { 1 };
                if scale < 1 { return Err("SCREEN_TRANSPARENT: skala muss >= 1 sein".into()); }
                // Transparenz ist ein Fenster-Erzeugungs-Flag (GLFW) -- nicht nachruestbar.
                if self.gfx.is_some() {
                    return Err("SCREEN_TRANSPARENT muss die ERSTE Grafik-Anweisung sein (vor LOADIMAGE/SCREEN/... -- Transparenz kann nicht nachtraeglich gesetzt werden)".into());
                }
                // w/h <= 0 -> ganzer aktueller Monitor (Vollbild-Overlay). Der Monitor
                // laesst sich erst nach der Fenster-Erzeugung abfragen, darum zuerst
                // transparent erzeugen, dann auf Monitorgroesse abdecken.
                let native = w <= 0 || h <= 0;
                let mut g = crate::graphics::Graphics::new_transparent(if native { 100 } else { w }, if native { 100 } else { h }, &title, scale)
                    .map_err(|e| format!("SCREEN_TRANSPARENT: {}", e))?;
                if native { g.cover_current_monitor(&title); }
                self.gfx = Some(g);
                Value::Nil
            }
            "cls" => { let c = if a.is_empty() { 0 } else { gi(a, 0, "CLS")? }; g!().cls(c); Value::Nil }
            "plot" => {
                let c = if a.len() == 3 { gi(a, 2, "PLOT")? } else { 0xFFFFFF };
                g!().plot(gi(a, 0, "PLOT")? as i32, gi(a, 1, "PLOT")? as i32, c); Value::Nil
            }
            "line" => {
                let c = if a.len() == 5 { gi(a, 4, "LINE")? } else { 0xFFFFFF };
                g!().line(gi(a,0,"LINE")? as i32, gi(a,1,"LINE")? as i32, gi(a,2,"LINE")? as i32, gi(a,3,"LINE")? as i32, c); Value::Nil
            }
            "box" => {
                let c = if a.len() == 5 { gi(a, 4, "BOX")? } else { 0xFFFFFF };
                g!().box_fill(gi(a,0,"BOX")? as i32, gi(a,1,"BOX")? as i32, gi(a,2,"BOX")? as i32, gi(a,3,"BOX")? as i32, c); Value::Nil
            }
            "rect" => {
                let c = if a.len() == 5 { gi(a, 4, "RECT")? } else { 0xFFFFFF };
                g!().rect(gi(a,0,"RECT")? as i32, gi(a,1,"RECT")? as i32, gi(a,2,"RECT")? as i32, gi(a,3,"RECT")? as i32, c); Value::Nil
            }
            "circle" => {
                let c = if a.len() == 4 { gi(a, 3, "CIRCLE")? } else { 0xFFFFFF };
                g!().circle(gi(a,0,"CIRCLE")? as i32, gi(a,1,"CIRCLE")? as i32, gi(a,2,"CIRCLE")? as i32, c); Value::Nil
            }
            "circleoutline" => {
                let c = if a.len() >= 4 { gi(a, 3, "CIRCLEOUTLINE")? } else { 0xFFFFFF };
                g!().circle_outline(gi(a,0,"CIRCLEOUTLINE")? as i32, gi(a,1,"CIRCLEOUTLINE")? as i32, gi(a,2,"CIRCLEOUTLINE")? as i32, c); Value::Nil
            }
            "linew" => {
                let c = if a.len() == 6 { gi(a, 5, "LINEW")? } else { 0xFFFFFF };
                g!().line_thick(gi(a,0,"LINEW")? as i32, gi(a,1,"LINEW")? as i32, gi(a,2,"LINEW")? as i32,
                    gi(a,3,"LINEW")? as i32, need_f(a,4,"LINEW")?, c); Value::Nil
            }
            "boxround" => {
                let c = if a.len() == 6 { gi(a, 5, "BOXROUND")? } else { 0xFFFFFF };
                g!().round_rect(gi(a,0,"BOXROUND")? as i32, gi(a,1,"BOXROUND")? as i32, gi(a,2,"BOXROUND")? as i32,
                    gi(a,3,"BOXROUND")? as i32, gi(a,4,"BOXROUND")? as i32, c, true); Value::Nil
            }
            "rectround" => {
                let c = if a.len() == 6 { gi(a, 5, "RECTROUND")? } else { 0xFFFFFF };
                g!().round_rect(gi(a,0,"RECTROUND")? as i32, gi(a,1,"RECTROUND")? as i32, gi(a,2,"RECTROUND")? as i32,
                    gi(a,3,"RECTROUND")? as i32, gi(a,4,"RECTROUND")? as i32, c, false); Value::Nil
            }
            "gradientv" => {
                g!().gradient_rect(gi(a,0,"GRADIENTV")? as i32, gi(a,1,"GRADIENTV")? as i32, gi(a,2,"GRADIENTV")? as i32,
                    gi(a,3,"GRADIENTV")? as i32, gi(a,4,"GRADIENTV")?, gi(a,5,"GRADIENTV")?, true); Value::Nil
            }
            "gradienth" => {
                g!().gradient_rect(gi(a,0,"GRADIENTH")? as i32, gi(a,1,"GRADIENTH")? as i32, gi(a,2,"GRADIENTH")? as i32,
                    gi(a,3,"GRADIENTH")? as i32, gi(a,4,"GRADIENTH")?, gi(a,5,"GRADIENTH")?, false); Value::Nil
            }
            "spline" => {
                let xs = arr_i32(&a[0], "SPLINE")?;
                let ys = arr_i32(&a[1], "SPLINE")?;
                if xs.len() != ys.len() { return Err("SPLINE: xs und ys muessen gleich lang sein".into()); }
                let w = if a.len() >= 4 { need_f(a, 3, "SPLINE")? } else { 1.0 };
                let c = if a.len() >= 3 { gi(a, 2, "SPLINE")? } else { 0xFFFFFF };
                g!().spline(&xs, &ys, w, c); Value::Nil
            }
            // Einzelne Bezier-Segmente: quadratisch (ein Kontrollpunkt) und
            // kubisch (zwei).
            "bezier" | "bezier3" => {
                let (n, punkte) = if name == "bezier" { ("BEZIER", 3) } else { ("BEZIER3", 4) };
                let k = punkte * 2;
                if a.len() < k || a.len() > k + 2 {
                    return Err(if name == "bezier" {
                        format!("BEZIER: erwartet (x1, y1, kx, ky, x2, y2[, farbe[, breite]]), erhalten {} Argumente", a.len())
                    } else {
                        format!("BEZIER3: erwartet (x1, y1, k1x, k1y, k2x, k2y, x2, y2[, farbe[, breite]]), erhalten {} Argumente", a.len())
                    });
                }
                let mut xs = Vec::with_capacity(punkte);
                let mut ys = Vec::with_capacity(punkte);
                for i in 0..punkte {
                    xs.push(gi(a, 2 * i, n)? as i32);
                    ys.push(gi(a, 2 * i + 1, n)? as i32);
                }
                let c = if a.len() > k { gi(a, k, n)? } else { 0xFFFFFF };
                let w = if a.len() > k + 1 { need_f(a, k + 1, n)? } else { 1.0 };
                g!().kurve(if name == "bezier" { 1 } else { 2 }, &xs, &ys, w, c);
                Value::Nil
            }
            "spline_basis" | "spline_bezier" => {
                let n = if name == "spline_basis" { "SPLINE_BASIS" } else { "SPLINE_BEZIER" };
                if a.len() < 2 || a.len() > 4 {
                    return Err(format!("{}: erwartet (xs, ys[, farbe[, breite]]), erhalten {} Argumente", n, a.len()));
                }
                let xs = arr_i32(&a[0], n)?;
                let ys = arr_i32(&a[1], n)?;
                if xs.len() != ys.len() { return Err(format!("{}: xs und ys muessen gleich lang sein", n)); }
                if name == "spline_basis" && xs.len() < 4 {
                    return Err(format!("SPLINE_BASIS: braucht mindestens 4 Punkte, nicht {}", xs.len()));
                }
                if name == "spline_bezier" && (xs.len() < 4 || (xs.len() - 1) % 3 != 0) {
                    return Err(format!(
                        "SPLINE_BEZIER: braucht 4, 7, 10 ... Punkte (Anfang, dann je zwei Kontrollpunkte und ein Ende), nicht {}",
                        xs.len()));
                }
                let c = if a.len() >= 3 { gi(a, 2, n)? } else { 0xFFFFFF };
                let w = if a.len() >= 4 { need_f(a, 3, n)? } else { 1.0 };
                g!().kurve(if name == "spline_basis" { 0 } else { 2 }, &xs, &ys, w, c);
                Value::Nil
            }
            // --- Blend-Modes (Batch 2) ---
            "blend_mode" => {
                let s = gs(a, 0, "BLEND_MODE")?.to_lowercase();
                let m = match s.as_str() {
                    "alpha" | "none" | "normal" => 0,
                    "add" | "additive" => 1,
                    "mult" | "multiply" | "multiplied" => 2,
                    "subtract" | "sub" => 4,
                    _ => return Err(format!("BLEND_MODE: unbekannter Modus '{}' (alpha/add/mult/subtract)", s)),
                };
                g!().blend_mode(m); Value::Nil
            }
            // --- Prozedurale Texturen (Batch 3) -> IMAGE-Handle ---
            "gentex_perlin" => Value::Int(g!().gen_tex_perlin(
                gi(a,0,"GENTEX_PERLIN")? as i32, gi(a,1,"GENTEX_PERLIN")? as i32, need_f(a,2,"GENTEX_PERLIN")?)?),
            "gentex_gradient" => Value::Int(g!().gen_tex_gradient(
                gi(a,0,"GENTEX_GRADIENT")? as i32, gi(a,1,"GENTEX_GRADIENT")? as i32,
                gi(a,2,"GENTEX_GRADIENT")?, gi(a,3,"GENTEX_GRADIENT")?, gb(a,4))?),
            "gentex_checked" => Value::Int(g!().gen_tex_checked(
                gi(a,0,"GENTEX_CHECKED")? as i32, gi(a,1,"GENTEX_CHECKED")? as i32,
                gi(a,2,"GENTEX_CHECKED")? as i32, gi(a,3,"GENTEX_CHECKED")? as i32,
                gi(a,4,"GENTEX_CHECKED")?, gi(a,5,"GENTEX_CHECKED")?)?),
            "gentex_color" => Value::Int(g!().gen_tex_color(
                gi(a,0,"GENTEX_COLOR")? as i32, gi(a,1,"GENTEX_COLOR")? as i32, gi(a,2,"GENTEX_COLOR")?)?),
            "gentex_radial" => {
                // dichte = Anteil des Radius in voller Innenfarbe. Ueber 1 gab
                // raylib still eine harte Scheibe statt eines Lichts
                // (Web-Runde 2026-10-13: 1.6 sah nach "dichter" aus).
                let dichte = if a.len() >= 5 { need_f(a,4,"GENTEX_RADIAL")? } else { 0.0 };
                if !(0.0..=1.0).contains(&dichte) {
                    return Err(format!("GENTEX_RADIAL: dichte {} liegt nicht in 0..1 -- sie ist der Anteil des Radius in voller Innenfarbe (0 = weicher Verlauf ab der Mitte, 1 = Scheibe)", dichte));
                }
                Value::Int(g!().gen_tex_radial(
                    gi(a,0,"GENTEX_RADIAL")? as i32, gi(a,1,"GENTEX_RADIAL")? as i32,
                    gi(a,2,"GENTEX_RADIAL")?, gi(a,3,"GENTEX_RADIAL")?, dichte)?)
            }
            "gentex_cellular" => Value::Int(g!().gen_tex_cellular(
                gi(a,0,"GENTEX_CELLULAR")? as i32, gi(a,1,"GENTEX_CELLULAR")? as i32,
                gi(a,2,"GENTEX_CELLULAR")?)?),
            "gentex_noise" => Value::Int(g!().gen_tex_noise(
                gi(a,0,"GENTEX_NOISE")? as i32, gi(a,1,"GENTEX_NOISE")? as i32,
                need_f(a,2,"GENTEX_NOISE")?)?),
            "gentex_gradient_box" => Value::Int(g!().gen_tex_gradient_square(
                gi(a,0,"GENTEX_GRADIENT_BOX")? as i32, gi(a,1,"GENTEX_GRADIENT_BOX")? as i32,
                need_f(a,2,"GENTEX_GRADIENT_BOX")?, gi(a,3,"GENTEX_GRADIENT_BOX")?,
                gi(a,4,"GENTEX_GRADIENT_BOX")?)?),
            // --- Bitmap-Font ---
            "loadfont_image" => Value::Int(g!().load_font_image(
                gi(a,0,"LOADFONT_IMAGE")?, gi(a,1,"LOADFONT_IMAGE")?, gi(a,2,"LOADFONT_IMAGE")?)?),
            "text_line_spacing" => { let n = gi(a,0,"TEXT_LINE_SPACING")?;
                                     g!().text_line_spacing(n); Value::Nil }
            // --- Clipboard + Drag&Drop (Batch 5) ---
            "clipboard_get" => Value::Str(g!().clipboard_get().into()),
            // Scheitert das Schreiben, ist das ein FEHLER und kein Schweigen:
            // die Zwischenablage gehoert jeweils EINEM Prozess, und wer seinen
            // Text nicht hineinbekommt, faendet beim naechsten Einfuegen den
            // alten wieder -- der Schaden zeigt sich dann an ganz anderer
            // Stelle. `clipboard_set` hat es da schon zehnmal versucht.
            "clipboard_set" => {
                let s = gs(a, 0, "CLIPBOARD_SET")?.to_string();
                if !g!().clipboard_set(&s) {
                    return Err("CLIPBOARD_SET: Text konnte nicht in die Zwischenablage gelegt werden \
                                -- ein anderes Programm haelt sie offen, oder der Text enthaelt ein Nullbyte".into());
                }
                Value::Nil
            }
            "files_dropped" => Value::Int(g!().dropped_files().len() as i64),
            "file_dropped" => {
                let i = gi(a, 0, "FILE_DROPPED")? as usize;
                Value::Str(g!().dropped_files().get(i).cloned().unwrap_or_default().into())
            }
            // --- Render-Targets (Batch 4) ---
            // RENDERTARGET_NEW(w, h [, behalten]) -- `behalten` = TRUE laesst den
            // Inhalt ueber das Bild hinaus stehen (Rueckkopplung/Nachzieher).
            "rendertarget_new" => Value::Int(g!().rendertarget_new(
                gi(a,0,"RENDERTARGET_NEW")? as i32, gi(a,1,"RENDERTARGET_NEW")? as i32,
                if a.len() >= 3 { gb(a, 2) } else { false })?),
            "rendertarget_clear" => {
                let farbe = if a.len() >= 2 { Some(gi(a,1,"RENDERTARGET_CLEAR")?) } else { None };
                g!().rendertarget_clear(gi(a,0,"RENDERTARGET_CLEAR")?, farbe)?;
                Value::Nil
            }
            "rendertarget_begin" => { g!().rendertarget_begin(gi(a,0,"RENDERTARGET_BEGIN")?)?; Value::Nil }
            "rendertarget_end" => { g!().rendertarget_end(); Value::Nil }
            "rendertarget_image" => Value::Int(g!().rendertarget_image(gi(a,0,"RENDERTARGET_IMAGE")?)?),
            "rendertarget_draw" => {
                let scale = if a.len() >= 4 { need_f(a,3,"RENDERTARGET_DRAW")? } else { 1.0 };
                let tint = if a.len() >= 5 { Some(gi(a,4,"RENDERTARGET_DRAW")?) } else { None };
                // Optionales 6. Argument: flip_v (vertikal spiegeln, fuer Boden-
                // Reflexionen). RENDERTARGET_DRAW(rt, x, y, scale, tint, TRUE).
                let flip_v = if a.len() >= 6 { gb(a, 5) } else { false };
                g!().rendertarget_draw(gi(a,0,"RENDERTARGET_DRAW")?,
                    gi(a,1,"RENDERTARGET_DRAW")? as i32, gi(a,2,"RENDERTARGET_DRAW")? as i32,
                    scale, tint, flip_v)?; Value::Nil
            }
            "triangle" => {
                let c = if a.len() == 7 { gi(a, 6, "TRIANGLE")? } else { 0xFFFFFF };
                g!().triangle(gi(a,0,"TRIANGLE")? as i32, gi(a,1,"TRIANGLE")? as i32, gi(a,2,"TRIANGLE")? as i32,
                    gi(a,3,"TRIANGLE")? as i32, gi(a,4,"TRIANGLE")? as i32, gi(a,5,"TRIANGLE")? as i32, c); Value::Nil
            }
            "triangleoutline" => {
                let c = if a.len() >= 7 { gi(a, 6, "TRIANGLEOUTLINE")? } else { 0xFFFFFF };
                g!().triangle_outline(gi(a,0,"TRIANGLEOUTLINE")? as i32, gi(a,1,"TRIANGLEOUTLINE")? as i32,
                    gi(a,2,"TRIANGLEOUTLINE")? as i32, gi(a,3,"TRIANGLEOUTLINE")? as i32,
                    gi(a,4,"TRIANGLEOUTLINE")? as i32, gi(a,5,"TRIANGLEOUTLINE")? as i32, c); Value::Nil
            }
            "ellipse" => {
                let c = if a.len() == 5 { gi(a, 4, "ELLIPSE")? } else { 0xFFFFFF };
                g!().ellipse(gi(a,0,"ELLIPSE")? as i32, gi(a,1,"ELLIPSE")? as i32, gi(a,2,"ELLIPSE")? as i32, gi(a,3,"ELLIPSE")? as i32, c); Value::Nil
            }
            "ellipseoutline" => {
                let c = if a.len() >= 5 { gi(a, 4, "ELLIPSEOUTLINE")? } else { 0xFFFFFF };
                g!().ellipse_outline(gi(a,0,"ELLIPSEOUTLINE")? as i32, gi(a,1,"ELLIPSEOUTLINE")? as i32, gi(a,2,"ELLIPSEOUTLINE")? as i32, gi(a,3,"ELLIPSEOUTLINE")? as i32, c); Value::Nil
            }
            "arc" => {
                let start = match a.get(4) { Some(Value::Int(n)) => *n as f64, Some(Value::Float(f)) => *f, _ => return Err("ARC: start muss Zahl sein".into()) };
                let end = match a.get(5) { Some(Value::Int(n)) => *n as f64, Some(Value::Float(f)) => *f, _ => return Err("ARC: end muss Zahl sein".into()) };
                let c = if a.len() >= 7 { gi(a, 6, "ARC")? } else { 0xFFFFFF };
                let width = if a.len() >= 8 { Some(need_f(a, 7, "ARC")?) } else { None };
                g!().arc(gi(a,0,"ARC")? as i32, gi(a,1,"ARC")? as i32, gi(a,2,"ARC")? as i32, gi(a,3,"ARC")? as i32, start, end, width, c); Value::Nil
            }
            // Kuchenstueck: derselbe Kreisring wie das chart-Modul, mit Innenradius 0.
            "pie" | "pieoutline" => {
                let n = if name == "pie" { "PIE" } else { "PIEOUTLINE" };
                if a.len() < 5 || a.len() > 6 {
                    return Err(format!("{}: erwartet (x, y, r, von_grad, bis_grad[, farbe]), erhalten {} Argumente", n, a.len()));
                }
                let c = if a.len() == 6 { gi(a, 5, n)? } else { 0xFFFFFF };
                g!().ring(gi(a,0,n)? as i32, gi(a,1,n)? as i32, 0, gi(a,2,n)? as i32,
                          need_f(a,3,n)?, need_f(a,4,n)?, c, name == "pie");
                Value::Nil
            }
            "circle_gradient" => {
                if a.len() != 5 {
                    return Err(format!("CIRCLE_GRADIENT: erwartet (x, y, r, innen, aussen), erhalten {} Argumente", a.len()));
                }
                g!().circle_gradient(gi(a,0,"CIRCLE_GRADIENT")? as i32, gi(a,1,"CIRCLE_GRADIENT")? as i32,
                    gi(a,2,"CIRCLE_GRADIENT")? as i32, gi(a,3,"CIRCLE_GRADIENT")?, gi(a,4,"CIRCLE_GRADIENT")?);
                Value::Nil
            }
            "boxrot" => {
                if a.len() < 5 || a.len() > 6 {
                    return Err(format!("BOXROT: erwartet (x, y, breite, hoehe, winkel[, farbe]), erhalten {} Argumente", a.len()));
                }
                let c = if a.len() == 6 { gi(a, 5, "BOXROT")? } else { 0xFFFFFF };
                g!().box_rot(gi(a,0,"BOXROT")? as i32, gi(a,1,"BOXROT")? as i32, gi(a,2,"BOXROT")? as i32,
                    gi(a,3,"BOXROT")? as i32, need_f(a,4,"BOXROT")?, c);
                Value::Nil
            }
            "gradient4" => {
                if a.len() != 8 {
                    return Err(format!("GRADIENT4: erwartet (x1, y1, x2, y2, oben_links, oben_rechts, unten_rechts, unten_links), erhalten {} Argumente", a.len()));
                }
                g!().gradient4(gi(a,0,"GRADIENT4")? as i32, gi(a,1,"GRADIENT4")? as i32, gi(a,2,"GRADIENT4")? as i32,
                    gi(a,3,"GRADIENT4")? as i32,
                    [gi(a,4,"GRADIENT4")?, gi(a,5,"GRADIENT4")?, gi(a,6,"GRADIENT4")?, gi(a,7,"GRADIENT4")?]);
                Value::Nil
            }
            "ngon" | "ngonoutline" => {
                let n = if name == "ngon" { "NGON" } else { "NGONOUTLINE" };
                let max = if name == "ngon" { 6 } else { 7 };
                if a.len() < 5 || a.len() > max {
                    return Err(if name == "ngon" {
                        format!("NGON: erwartet (x, y, ecken, r, winkel[, farbe]), erhalten {} Argumente", a.len())
                    } else {
                        format!("NGONOUTLINE: erwartet (x, y, ecken, r, winkel[, farbe[, breite]]), erhalten {} Argumente", a.len())
                    });
                }
                let ecken = gi(a, 2, n)?;
                if ecken < 3 { return Err(format!("{}: ein Vieleck hat mindestens 3 Ecken, nicht {}", n, ecken)); }
                if ecken > 1000 { return Err(format!("{}: hoechstens 1000 Ecken, nicht {}", n, ecken)); }
                let c = if a.len() >= 6 { gi(a, 5, n)? } else { 0xFFFFFF };
                let breite = if name == "ngon" { None }
                             else if a.len() == 7 { Some(need_f(a, 6, n)?) } else { Some(1.0) };
                g!().ngon(gi(a,0,n)? as i32, gi(a,1,n)? as i32, ecken as i32, gi(a,3,n)? as i32,
                          need_f(a,4,n)?, c, breite);
                Value::Nil
            }
            "linedashed" => {
                if a.len() < 6 || a.len() > 7 {
                    return Err(format!("LINEDASHED: erwartet (x1, y1, x2, y2, strich, luecke[, farbe]), erhalten {} Argumente", a.len()));
                }
                let (st, lu) = (gi(a,4,"LINEDASHED")?, gi(a,5,"LINEDASHED")?);
                if st < 1 || lu < 1 {
                    return Err(format!("LINEDASHED: Strich und Luecke muessen mindestens 1 sein, nicht {} und {}", st, lu));
                }
                let c = if a.len() == 7 { gi(a, 6, "LINEDASHED")? } else { 0xFFFFFF };
                g!().line_dashed(gi(a,0,"LINEDASHED")? as i32, gi(a,1,"LINEDASHED")? as i32, gi(a,2,"LINEDASHED")? as i32,
                    gi(a,3,"LINEDASHED")? as i32, st as i32, lu as i32, c);
                Value::Nil
            }
            "drawimage9" => {
                if a.len() < 9 || a.len() > 10 {
                    return Err(format!("DRAWIMAGE9: erwartet (bild, x, y, breite, hoehe, links, oben, rechts, unten[, farbe]), erhalten {} Argumente", a.len()));
                }
                let tint = if a.len() == 10 { Some(gi(a, 9, "DRAWIMAGE9")?) } else { None };
                g!().draw_image_9(gi(a,0,"DRAWIMAGE9")?, gi(a,1,"DRAWIMAGE9")? as i32, gi(a,2,"DRAWIMAGE9")? as i32,
                    gi(a,3,"DRAWIMAGE9")? as i32, gi(a,4,"DRAWIMAGE9")? as i32,
                    [gi(a,5,"DRAWIMAGE9")? as i32, gi(a,6,"DRAWIMAGE9")? as i32,
                     gi(a,7,"DRAWIMAGE9")? as i32, gi(a,8,"DRAWIMAGE9")? as i32], tint)?;
                Value::Nil
            }
            "polygon" | "polygonoutline" => {
                let filled = name == "polygon";
                let pts: Vec<i32> = match a.get(0) {
                    Some(Value::Array(arr)) => {
                        let arr = arr.borrow();
                        let mut v = Vec::with_capacity(arr.cells.len());
                        for x in arr.cells.iter() { match x { Value::Int(i) => v.push(i as i32), _ => return Err("POLYGON: Punkte muessen INTEGER sein".into()) } }
                        v
                    }
                    _ => return Err("POLYGON: Punkte muessen ein ARRAY OF INTEGER sein".into()),
                };
                let c = if a.len() >= 2 { gi(a, 1, "POLYGON")? } else { 0xFFFFFF };
                g!().polygon(&pts, c, filled)?; Value::Nil
            }
            "text" => {
                let c = if a.len() == 4 { gi(a, 3, "TEXT")? } else { 0xFFFFFF };
                let s = match a.get(2) { Some(Value::Str(s)) => s.to_string(), Some(v) => v.fmt(), None => String::new() };
                g!().text(gi(a,0,"TEXT")? as i32, gi(a,1,"TEXT")? as i32, s, c); Value::Nil
            }
            "textrot" => {
                // TEXTROT(x, y, s$, winkel[, skala[, farbe]]) -- zentriert auf (x,y)
                let x = gi(a, 0, "TEXTROT")? as i32;
                let y = gi(a, 1, "TEXTROT")? as i32;
                let s = match a.get(2) { Some(Value::Str(s)) => s.to_string(), Some(v) => v.fmt(), None => String::new() };
                let ang = need_f(a, 3, "TEXTROT")? as f32;
                let scl = if a.len() >= 5 { need_f(a, 4, "TEXTROT")? } else { 1.0 };
                if scl <= 0.0 { return Err("TEXTROT: skala muss > 0 sein".into()); }
                let c = if a.len() >= 6 { gi(a, 5, "TEXTROT")? } else { 0xFFFFFF };
                g!().text_rot(x, y, s, ang, scl as f32, c); Value::Nil
            }
            "text_size" => { g!().set_text_size(gi(a,0,"TEXT_SIZE")? as i32); Value::Nil }
            "text_width" => Value::Int(g!().text_width(gs(a,0,"TEXT_WIDTH")?) as i64),
            "text_height" => Value::Int(g!().text_height() as i64),
            "text_style" => {
                let bits = crate::schnitt::stil_parsen(&gs(a,0,"TEXT_STYLE")?).map_err(|e| format!("TEXT_STYLE: {}", e))?;
                g!().set_text_stil(bits); Value::Nil
            }
            "text_get_style$" | "text_get_style" => Value::Str(crate::schnitt::stil_text(g!().text_stil()).into()),
            "font_style" | "font_has_style" => {
                let f = if name == "font_style" { "FONT_STYLE" } else { "FONT_HAS_STYLE" };
                if a.len() != 2 { return Err(format!("{}: erwartet (font, stil$)", f)); }
                let bits = crate::schnitt::stil_parsen(&gs(a,1,f)?).map_err(|e| format!("{}: {}", f, e))?;
                let (h, echt) = g!().font_schnitt(gi(a,0,f)?, bits)?;
                if name == "font_style" { Value::Int(h) } else { Value::Bool(echt) }
            }
            // Masse aus der Schriftdatei: Grundlinie und das Rechteck, das eine
            // Glyphe bedeckt -- relativ zu dem Punkt, an den TEXT den Text
            // setzt. Damit laesst sich ein Zeichen genau platzieren (ein
            // Notenschluessel auf seine Linie, ein Symbol mittig in einen
            // Knopf), ohne die Datei selbst auszulesen.
            "font_baseline" => {
                if a.len() != 2 { return Err("FONT_BASELINE: erwartet (font, groesse)".into()); }
                Value::Float(g!().font_baseline(gi(a,0,"FONT_BASELINE")?, zahl_arg(a,1,"FONT_BASELINE")?)?)
            }
            "font_glyph_box" => {
                if a.len() != 3 { return Err("FONT_GLYPH_BOX: erwartet (font, zeichen$, groesse)".into()); }
                let t = gs(a,1,"FONT_GLYPH_BOX")?;
                let mut it = t.chars();
                let c = match (it.next(), it.next()) {
                    (Some(c), None) => c,
                    _ => return Err("FONT_GLYPH_BOX: erwartet GENAU ein Zeichen (Arg 2)".into()),
                };
                let (x, y, b, h) = g!().font_glyph_box(gi(a,0,"FONT_GLYPH_BOX")?, c, zahl_arg(a,2,"FONT_GLYPH_BOX")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Float(x), Value::Float(y), Value::Float(b), Value::Float(h)]))
            }
            "loadfont" => {
                if a.len() > 3 { return Err("LOADFONT: erwartet (pfad$, groesse[, zeichen$])".into()); }
                if a.len() == 3 {
                    Value::Int(g!().load_font_mit(gs(a,0,"LOADFONT")?, gi(a,1,"LOADFONT")? as i32, &gs(a,2,"LOADFONT")?)?)
                } else {
                    Value::Int(g!().load_font(gs(a,0,"LOADFONT")?, gi(a,1,"LOADFONT")? as i32)?)
                }
            }
            "setfont" => {
                // Zweites Argument (2026-10-14, Texteditor-Runde): die Groesse,
                // in der ab jetzt gezeichnet wird. Ohne sie gilt weiter die
                // Lade-Groesse -- wer scharf bei 32 laedt und bei 16 zeichnen
                // will, schrieb vorher SETFONT + TEXT_SIZE und vergass das zweite.
                if a.is_empty() || a.len() > 2 {
                    return Err(format!("SETFONT: erwartet (handle[, groesse]), erhalten {} Argument(e)", a.len()));
                }
                g!().set_font(gi(a,0,"SETFONT")?)?;
                if a.len() == 2 {
                    let gr = gi(a,1,"SETFONT")?;
                    if gr < 1 { return Err(format!("SETFONT: groesse muss mindestens 1 sein, erhalten {}", gr)); }
                    g!().set_text_size(gr as i32);
                }
                Value::Nil
            }
            "loadfont_sdf" => {
                if a.len() < 2 || a.len() > 3 { return Err("LOADFONT_SDF: erwartet (pfad$, groesse[, zeichen$])".into()); }
                let z = if a.len() == 3 { Some(gs(a,2,"LOADFONT_SDF")?) } else { None };
                Value::Int(g!().load_font_sdf(gs(a,0,"LOADFONT_SDF")?, gi(a,1,"LOADFONT_SDF")? as i32, z.as_deref())?)
            }
            "font_has_glyph" | "font_glyph_width" => {
                let f = if name == "font_has_glyph" { "FONT_HAS_GLYPH" } else { "FONT_GLYPH_WIDTH" };
                let z = gs(a, 1, f)?;
                let mut zeichen = z.chars();
                let c = match (zeichen.next(), zeichen.next()) {
                    (Some(c), None) => c,
                    _ => return Err(format!("{}: erwartet genau ein Zeichen, nicht \"{}\"", f, z)),
                };
                let h = gi(a, 0, f)?;
                if name == "font_has_glyph" { Value::Bool(g!().font_has_glyph(h, c)?) }
                else {
                    let size = need_f(a, 2, f)?;
                    if !size.is_finite() || size <= 0.0 {
                        return Err(format!("FONT_GLYPH_WIDTH: die Groesse muss groesser als 0 sein, nicht {}", size));
                    }
                    Value::Float(g!().font_glyph_width(h, c, size as f32)?)
                }
            }
            "font_to_image" => {
                if a.is_empty() || a.len() > 3 { return Err("FONT_TO_IMAGE: erwartet (font[, erstes[, letztes]])".into()); }
                let erstes = if a.len() > 1 { gi(a,1,"FONT_TO_IMAGE")? } else { 32 };
                let letztes = if a.len() > 2 { gi(a,2,"FONT_TO_IMAGE")? } else if a.len() > 1 { erstes.max(126) } else { 126 };
                if erstes < 0 || letztes < 0 || erstes > 0x10FFFF || letztes > 0x10FFFF {
                    return Err(format!("FONT_TO_IMAGE: Zeichennummern gehen von 0 bis &H10FFFF, nicht {} und {}", erstes, letztes));
                }
                Value::Int(g!().font_to_image(gi(a,0,"FONT_TO_IMAGE")?, erstes as u32, letztes as u32)?)
            }
            "drawfps" => {
                if a.len() != 2 && a.len() != 3 { return Err("DRAWFPS: erwartet (x, y[, farbe])".into()); }
                let farbe = if a.len() == 3 { Some(gi(a,2,"DRAWFPS")?) } else { None };
                g!().draw_fps(gi(a,0,"DRAWFPS")? as i32, gi(a,1,"DRAWFPS")? as i32, farbe);
                Value::Nil
            }
            "text_spacing" => { g!().set_text_spacing(gi(a,0,"TEXT_SPACING")? as i32); Value::Nil }
            "loadimage" => Value::Int(g!().load_texture(gs(a,0,"LOADIMAGE")?)?),
            "drawimage" => {
                let idx = gi(a, 0, "DRAWIMAGE")?;
                g!().draw_image(idx, gi(a,1,"DRAWIMAGE")? as i32, gi(a,2,"DRAWIMAGE")? as i32)?; Value::Nil
            }
            "drawimagerot" => {
                // DRAWIMAGEROT(img, x, y, winkel_grad [, skala [, tint]]) -- zentriert + gedreht.
                let idx = gi(a, 0, "DRAWIMAGEROT")?;
                let x = gi(a, 1, "DRAWIMAGEROT")? as i32;
                let y = gi(a, 2, "DRAWIMAGEROT")? as i32;
                let ang = need_f(a, 3, "DRAWIMAGEROT")? as f32;
                let scale = if a.len() >= 5 { need_f(a, 4, "DRAWIMAGEROT")? as f32 } else { 1.0 };
                let tint = if a.len() >= 6 { Some(gi(a, 5, "DRAWIMAGEROT")?) } else { None };
                g!().draw_image_rot(idx, x, y, ang, scale, tint)?; Value::Nil
            }
            // IMAGE_WIDTH/IMAGE_HEIGHT: derselbe Befehl im Namensschema der
            // uebrigen IMAGE_*-Befehle (IMAGE_NEW, IMAGE_SAVE ...).
            "imagewidth" | "image_width" => Value::Int(g!().image_width(gi(a,0,"IMAGE_WIDTH")?)?),
            "imageheight" | "image_height" => Value::Int(g!().image_height(gi(a,0,"IMAGE_HEIGHT")?)?),
            "flip" => {
                g!().flip();
                // Gesammelte PRINT-Zeilen eines Spiels gehen spaetestens nach
                // 50 ms hinaus, nicht erst beim Beenden.
                if !self.out.is_empty() && self.out_zeit.elapsed() >= std::time::Duration::from_millis(50) { self.flush_out(); }
                // Musik-Stream nachfuettern (sonst stockt die Wiedergabe).
                if let Some(au) = self.audio.as_mut() { au.update(); }
                Value::Nil
            }
            "keypressed" => { let k = taste(g!(), a, "KEYPRESSED")?; Value::Bool(g!().key_down(k)) }
            "inkey$" | "inkey" => Value::Str(g!().inkey().into()),
            "waitkey" => Value::Int(g!().waitkey()),
            // SCROLL verschiebt persistente Framebuffer-Pixel -- die native
            // Runtime zeichnet jeden Frame neu aus dem Command-Buffer (Layer
            // werden pro FLIP geleert), es gibt also keine persistenten Pixel
            // zum Verschieben. Graceful No-Op (Programm laeuft); SCROLL ist
            // Tree-Walker-only (siehe docs/befehlssatz-roadmap.md).
            "scroll" => { let _ = (gi(a,0,"SCROLL")?, gi(a,1,"SCROLL")?); Value::Nil }
            "joystick_count" => Value::Int(g!().joystick_count()),
            "joystick_name" => Value::Str(g!().joystick_name(gi(a,0,"JOYSTICK_NAME")?)?.into()),
            "joystick_axis" => Value::Float(g!().joystick_axis(gi(a,0,"JOYSTICK_AXIS")?, gi(a,1,"JOYSTICK_AXIS")?)?),
            "joystick_button" => Value::Bool(g!().joystick_button(gi(a,0,"JOYSTICK_BUTTON")?, gi(a,1,"JOYSTICK_BUTTON")?)?),
            "joystick_hat_x" => Value::Int(g!().joystick_hat_x(gi(a,0,"JOYSTICK_HAT_X")?, gi(a,1,"JOYSTICK_HAT_X")?)?),
            "joystick_hat_y" => Value::Int(g!().joystick_hat_y(gi(a,0,"JOYSTICK_HAT_Y")?, gi(a,1,"JOYSTICK_HAT_Y")?)?),
            "joystick_rumble" => {
                g!().joystick_rumble(gi(a,0,"JOYSTICK_RUMBLE")?, need_f(a,1,"JOYSTICK_RUMBLE")?, need_f(a,2,"JOYSTICK_RUMBLE")?, need_f(a,3,"JOYSTICK_RUMBLE")?)?;
                Value::Nil
            }
            "quitrequested" => Value::Bool(g!().quit_requested()),
            "window_close_requested" => Value::Bool(g!().close_requested()),
            "mousex" => Value::Int(g!().mouse_x()),
            "mousey" => Value::Int(g!().mouse_y()),
            "mousebutton" => Value::Bool(g!().mouse_button(gi(a,0,"MOUSEBUTTON")?)),
            // --- Eingabe-Flanken: "genau in DIESEM Frame" ---------------------
            // MOUSEBUTTON/KEYPRESSED bleiben "gehalten" (Namen sind historisch,
            // sie umzudeuten wuerde bestehende Programme still kaputtmachen).
            "mouse_hit" => Value::Bool(g!().mouse_hit(gi(a,0,"MOUSE_HIT")?)),
            "mouse_released" => Value::Bool(g!().mouse_released(gi(a,0,"MOUSE_RELEASED")?)),
            "keyhit" => { let k = taste(g!(), a, "KEYHIT")?; Value::Bool(g!().key_hit(k)) }
            "keyreleased" => { let k = taste(g!(), a, "KEYRELEASED")?; Value::Bool(g!().key_released_edge(k)) }
            "keyrepeat" => { let k = taste(g!(), a, "KEYREPEAT")?; Value::Bool(g!().key_repeat(k)) }
            "mouse_delta_x" => Value::Float(g!().mouse_delta_x()),
            "mouse_delta_y" => Value::Float(g!().mouse_delta_y()),
            "mouse_set_pos" => { let (x, y) = (gi(a,0,"MOUSE_SET_POS")?, gi(a,1,"MOUSE_SET_POS")?);
                                 g!().mouse_set_pos(x, y); Value::Nil }
            "mouse_on_screen" => Value::Bool(g!().mouse_on_screen()),
            "mouse_cursor" => { let s = gs(a,0,"MOUSE_CURSOR")?.to_string();
                                g!().mouse_cursor(&s)?; Value::Nil }
            "mouse_cursor_get$" | "mouse_cursor_get" => Value::str_rc(g!().zeiger_jetzt()),
            "mouse_cursor_new" => { let (b, x, y) = (gi(a,0,"MOUSE_CURSOR_NEW")?, gi(a,1,"MOUSE_CURSOR_NEW")?, gi(a,2,"MOUSE_CURSOR_NEW")?);
                                    Value::str_rc(g!().zeiger_neu(b, x, y)?) }
            "joystick_hit" => Value::Bool(g!().joystick_hit(gi(a,0,"JOYSTICK_HIT")?,
                                                            gi(a,1,"JOYSTICK_HIT")?)?),
            "joystick_released" => Value::Bool(g!().joystick_released(
                gi(a,0,"JOYSTICK_RELEASED")?, gi(a,1,"JOYSTICK_RELEASED")?)?),
            "joystick_any_button" => Value::Int(g!().joystick_any_button()),
            "joystick_axis_count" => Value::Int(g!().joystick_axis_count(
                gi(a,0,"JOYSTICK_AXIS_COUNT")?)?),
            // --- Touch + Gesten (raylib-Subsystem, war komplett ungenutzt) ----
            "touch_count" => Value::Int(g!().touch_count()),
            "touch_x" => Value::Float(g!().touch_x(gi(a,0,"TOUCH_X")?)),
            "touch_y" => Value::Float(g!().touch_y(gi(a,0,"TOUCH_Y")?)),
            "touch_id" => Value::Int(g!().touch_id(gi(a,0,"TOUCH_ID")?)),
            "gesture$" => Value::Str(g!().gesture().into()),
            "gesture_drag_x" => Value::Float(g!().gesture_drag_x()),
            "gesture_drag_y" => Value::Float(g!().gesture_drag_y()),
            "gesture_drag_angle" => Value::Float(g!().gesture_drag_angle()),
            "gesture_pinch_x" => Value::Float(g!().gesture_pinch_x()),
            "gesture_pinch_y" => Value::Float(g!().gesture_pinch_y()),
            "gesture_pinch_angle" => Value::Float(g!().gesture_pinch_angle()),
            "gesture_hold_time" => Value::Float(g!().gesture_hold_time()),
            // --- Fenster-Zustand + Politur ------------------------------------
            "window_focused" => Value::Bool(g!().window_focused()),
            "window_minimized" => Value::Bool(g!().window_minimized()),
            "window_maximized" => Value::Bool(g!().window_maximized()),
            "window_hidden" => Value::Bool(g!().window_hidden()),
            "window_is_fullscreen" => Value::Bool(g!().window_fullscreen_state()),
            "window_focus" => { g!().window_focus(); Value::Nil }
            "window_opacity" => { let v = need_f(a,0,"WINDOW_OPACITY")?;
                                  g!().window_opacity(v); Value::Nil }
            "window_icon" => { let i = gi(a,0,"WINDOW_ICON")?; g!().window_icon(i)?; Value::Nil }
            "window_hide" => { g!().window_hide(true); Value::Nil }
            "window_show" => { g!().window_hide(false); Value::Nil }
            "tray_show" => {
                if a.len() > 2 { return Err("TRAY_SHOW: erwartet ([bild[, hinweis$]])".into()); }
                // Ein Bild ist ein Handle (Zahl); ein Text allein ist der Hinweis.
                let (bild, tipp) = match (a.first(), a.get(1)) {
                    (Some(Value::Str(s)), None) => (None, Some(s.to_string())),
                    (Some(_), t) => (Some(gi(a,0,"TRAY_SHOW")?), match t { Some(_) => Some(gs(a,1,"TRAY_SHOW")?.to_string()), None => None }),
                    (None, _) => (None, None),
                };
                g!().tray_show(bild, tipp.as_deref())?;
                Value::Nil
            }
            "tray_hide" => { let _ = g!(); crate::tray::weg(); Value::Nil }
            "tray_tooltip" => {
                let t = gs(a,0,"TRAY_TOOLTIP")?.to_string();
                let _ = g!();
                crate::tray::tipp(&t).map_err(|e| format!("TRAY_TOOLTIP: {}", e))?;
                Value::Nil
            }
            "tray_menu" => {
                let t = gs(a,0,"TRAY_MENU")?;
                let eintraege: Vec<String> = t.split('|').map(|e| e.trim().to_string()).filter(|e| !e.is_empty()).collect();
                if eintraege.len() > 64 { return Err(format!("TRAY_MENU: hoechstens 64 Eintraege, nicht {}", eintraege.len())); }
                let _ = g!();
                crate::tray::menue(eintraege);
                Value::Nil
            }
            "tray_shown" => { let _ = g!(); Value::Bool(crate::tray::sichtbar()) }
            "tray_clicked" => { let _ = g!(); Value::Bool(crate::tray::bild().klick) }
            "tray_double_clicked" => { let _ = g!(); Value::Bool(crate::tray::bild().doppel) }
            "tray_menu_clicked$" | "tray_menu_clicked" => { let _ = g!(); Value::str_rc(crate::tray::bild().menue.unwrap_or_default()) }
            "notify" => {
                let (titel, text) = (gs(a,0,"NOTIFY")?.to_string(), if a.len() > 1 { gs(a,1,"NOTIFY")?.to_string() } else { String::new() });
                if a.len() > 2 { return Err("NOTIFY: erwartet (titel$[, text$])".into()); }
                g!().notify(&titel, &text)?;
                Value::Nil
            }
            "notify_clicked" => { let _ = g!(); Value::Bool(crate::tray::bild().mitteilung) }
            "get_time" => Value::Float(g!().get_time()),
            "openurl" => { let u = gs(a,0,"OPENURL")?.to_string(); g!().open_url(&u)?; Value::Nil }
            // Graceful ohne SCREEN (0 / 0) -- wie der Tree-Walker (_buf_size=(0,0),
            // pop_mouse_wheel ohne Fenster = 0).
            "mousewheel" => Value::Int(self.gfx.as_ref().map(|g| g.pop_mouse_wheel()).unwrap_or(0)),
            "mousewheel_x" => Value::Float(self.gfx.as_ref().map(|g| g.mouse_wheel_x()).unwrap_or(0.0)),
            "mousewheel_y" => Value::Float(self.gfx.as_ref().map(|g| g.mouse_wheel_y()).unwrap_or(0.0)),
            "key_name$" => Value::Str(g!().key_name(gi(a,0,"KEY_NAME$")?).into()),
            "key_any_hit" => Value::Int(g!().key_any_hit()),
            // Eingabe aufzeichnen/abspielen: Demo-Modus, Fehlerberichte zum
            // Nachspielen, automatische Spieltests.
            "automation_record" => { let p = gs(a,0,"AUTOMATION_RECORD")?.to_string(); g!().automation_record(&p)?; Value::Nil }
            "automation_stop" => Value::Int(g!().automation_stop()?),
            "automation_play" => { let p = gs(a,0,"AUTOMATION_PLAY")?.to_string(); Value::Int(g!().automation_play(&p)?) }
            "automation_recording" => Value::Bool(g!().automation_recording()),
            "automation_playing" => Value::Bool(g!().automation_playing()),
            "automation_frame" => Value::Int(g!().automation_frame()),
            "automation_count" => Value::Int(g!().automation_count()),
            "joystick_mappings" => Value::Int(g!().joystick_mappings(gs(a,0,"JOYSTICK_MAPPINGS")?)),
            "window_dpi_x" => Value::Float(g!().window_dpi_x()),
            "window_render_width" => Value::Int(g!().render_size().0),
            "window_render_height" => Value::Int(g!().render_size().1),
            "window_wait_events" => {
                let s = need_f(a, 0, "WINDOW_WAIT_EVENTS")?;
                if !s.is_finite() || s < 0.0 {
                    return Err(format!("WINDOW_WAIT_EVENTS: die Dauer in Sekunden muss 0 oder groesser sein, nicht {}", s));
                }
                g!().wait_events(s);
                Value::Nil
            }
            "mouse_offset" => {
                g!().mouse_offset(gi(a, 0, "MOUSE_OFFSET")? as i32, gi(a, 1, "MOUSE_OFFSET")? as i32);
                Value::Nil
            }
            "mouse_scale" => {
                let (sx, sy) = (need_f(a, 0, "MOUSE_SCALE")?, need_f(a, 1, "MOUSE_SCALE")?);
                if !(sx.is_finite() && sy.is_finite()) || sx <= 0.0 || sy <= 0.0 {
                    return Err(format!("MOUSE_SCALE: der Massstab muss groesser als 0 sein, nicht {} und {}", sx, sy));
                }
                g!().mouse_scale(sx as f32, sy as f32);
                Value::Nil
            }
            "window_dpi_y" => Value::Float(g!().window_dpi_y()),
            "screenwidth" => Value::Int(self.gfx.as_ref().map(|g| g.screen_width()).unwrap_or(0)),
            "screenheight" => Value::Int(self.gfx.as_ref().map(|g| g.screen_height()).unwrap_or(0)),
            // Ein Bit von TEXT_STYLE an- oder ausschalten -- die aelteren
            // Namen, bis 2026-09-21 wirkungslos (raylib hat keine Schnitte;
            // jetzt sucht die Laufzeit sie als Datei oder bildet sie nach).
            "text_bold" | "text_italic" => {
                let bit = if name == "text_bold" { crate::schnitt::FETT } else { crate::schnitt::KURSIV };
                let an = gb(a, 0);
                let st = g!().text_stil();
                g!().set_text_stil(if an { st | bit } else { st & !bit });
                Value::Nil
            }
            "set_fullscreen" => { g!().set_fullscreen(gb(a, 0)); Value::Nil }
            "mouse_visible" => { g!().mouse_visible(gb(a, 0)); Value::Nil }
            "mouse_lock" => { g!().mouse_lock(gb(a, 0)); Value::Nil }
            "mouse_hidden" => Value::Bool(g!().mouse_hidden()),
            "delta" => Value::Float(g!().delta()),
            "fps" => Value::Int(g!().fps()),
            "setfps" => { g!().set_target_fps(gi(a, 0, "SETFPS")?); Value::Nil }
            "setwindowtitle" => { g!().set_window_title(gs(a, 0, "SETWINDOWTITLE")?); Value::Nil }
            "savescreenshot" => { g!().save_screenshot(gs(a, 0, "SAVESCREENSHOT")?); Value::Nil }
            // GFX_PUSH/GFX_POP: Zeichenzustand sichern und zurueckholen. Licht,
            // Nebel, Himmel, Schatten, Kamera, Schrift und Post-Effekt sind
            // global -- ohne das muss jede Szene beim Verlassen von Hand
            // aufraeumen, und eine vergessene Zeile faellt erst spaeter auf.
            "gfx_push" => { g!().gfx_push(); Value::Nil }
            "gfx_pop" => {
                if !g!().gfx_pop() {
                    return Err("GFX_POP: der Stapel ist leer -- zu jedem GFX_POP gehoert ein GFX_PUSH".into());
                }
                Value::Nil
            }
            "gfx_depth" => Value::Int(g!().gfx_depth()),
            // --- Clip-Rechteck (Scissor) ---
            // Die Mechanik lag laengst da (das gui-Modul clippt seine Fenster
            // damit); herausgefuehrt war sie nie. Es ist ein STAPEL: ein
            // inneres SCISSOR schneidet mit dem aeusseren, SCISSOR_END nimmt
            // nur die oberste Ebene zurueck.
            "scissor" => {
                let (x, y) = (gi(a, 0, "SCISSOR")?, gi(a, 1, "SCISSOR")?);
                let (w, h) = (gi(a, 2, "SCISSOR")?, gi(a, 3, "SCISSOR")?);
                if w < 0 || h < 0 {
                    return Err("SCISSOR: Breite und Hoehe duerfen nicht negativ sein".into());
                }
                g!().push_clip(x as i32, y as i32, w as i32, h as i32);
                Value::Nil
            }
            "scissor_end" => {
                if !g!().pop_clip() {
                    return Err("SCISSOR_END: es ist kein Clip offen -- zu jedem SCISSOR_END gehoert ein SCISSOR".into());
                }
                Value::Nil
            }
            "scissor_depth" => Value::Int(g!().clip_depth()),
            // Dasselbe fuer die Audio-Busse: Effekte, die eine Szene anhaengt,
            // sollen den Rest des Programms nicht mitnehmen.
            "audio_push" => { self.audio_mut()?.audio_push(); Value::Nil }
            "audio_pop" => {
                if !self.audio_mut()?.audio_pop()? {
                    return Err("AUDIO_POP: der Stapel ist leer -- zu jedem AUDIO_POP gehoert ein AUDIO_PUSH".into());
                }
                Value::Nil
            }
            "audio_depth" => Value::Int(self.audio_mut()?.audio_depth()),
            // Natives OS-Fenster (das SCREEN-Fenster) steuern.
            "window_resizable" => { g!().window_resizable(gb(a, 0)); Value::Nil }
            "window_min_size" => { g!().window_min_size(gi(a,0,"WINDOW_MIN_SIZE")? as i32, gi(a,1,"WINDOW_MIN_SIZE")? as i32); Value::Nil }
            "window_max_size" => { g!().window_max_size(gi(a,0,"WINDOW_MAX_SIZE")? as i32, gi(a,1,"WINDOW_MAX_SIZE")? as i32); Value::Nil }
            "window_maximize" => { g!().window_maximize(); Value::Nil }
            "window_minimize" => { g!().window_minimize(); Value::Nil }
            "window_restore" => { g!().window_restore(); Value::Nil }
            "window_resized" => Value::Bool(g!().window_resized()),
            "window_undecorated" => { g!().window_undecorated(gb(a, 0)); Value::Nil }
            "window_topmost" => { g!().window_topmost(gb(a, 0)); Value::Nil }
            "window_esc_quit" => { g!().set_esc_quit(gb(a, 0)); Value::Nil }
            "window_passthrough" => { g!().window_passthrough(gb(a, 0)); Value::Nil }

            // --- Native OS-Datei-/Ordner-Dialoge (rfd; liefern Pfad oder "") ---
            // Eigenes Feature `dialogs`: im Web-Build gibt es keine
            // OS-Dialoge, und rfd wuerde den Build sogar verhindern (es zieht
            // js-sys/wasm-bindgen in einer Version nach, mit der cpals
            // WebAudio-Host nicht mehr uebersetzt). Ohne das Feature melden
            // die Aufrufe sich klar, statt still etwas Falsches zu liefern.
            #[cfg(feature = "dialogs")]
            "file_open_dialog" => {
                let title = if !a.is_empty() { gs(a, 0, "FILE_OPEN_DIALOG")?.to_string() } else { String::new() };
                let exts = if a.len() >= 2 { crate::filedialog::parse_exts(gs(a, 1, "FILE_OPEN_DIALOG")?) } else { vec![] };
                Value::str_rc(crate::filedialog::open(&title, &exts))
            }
            #[cfg(feature = "dialogs")]
            "file_save_dialog" => {
                let title = if !a.is_empty() { gs(a, 0, "FILE_SAVE_DIALOG")?.to_string() } else { String::new() };
                let default_name = if a.len() >= 2 { gs(a, 1, "FILE_SAVE_DIALOG")?.to_string() } else { String::new() };
                let exts = if a.len() >= 3 { crate::filedialog::parse_exts(gs(a, 2, "FILE_SAVE_DIALOG")?) } else { vec![] };
                Value::str_rc(crate::filedialog::save(&title, &default_name, &exts))
            }
            #[cfg(feature = "dialogs")]
            "folder_dialog" => {
                let title = if !a.is_empty() { gs(a, 0, "FOLDER_DIALOG")?.to_string() } else { String::new() };
                Value::str_rc(crate::filedialog::folder(&title))
            }
            #[cfg(feature = "dialogs")]
            "gui_message" => {
                crate::filedialog::message(gs(a, 0, "GUI_MESSAGE")?, gs(a, 1, "GUI_MESSAGE")?);
                Value::Nil
            }
            #[cfg(feature = "dialogs")]
            "gui_confirm" => {
                // Drittes Argument: "janein" beschriftet die Knoepfe als Frage
                // statt als Anweisung. Vorgabe bleibt OK/Abbrechen.
                let stil = if a.len() > 2 { gs(a, 2, "GUI_CONFIRM")?.to_lowercase() } else { String::new() };
                let ja_nein = match stil.as_str() {
                    "" | "ok" | "okabbrechen" => false,
                    "janein" | "ja/nein" | "janein?" | "frage" => true,
                    other => return Err(format!(
                        "GUI_CONFIRM: '{}' ist kein Stil -- moeglich sind \"ok\" (Vorgabe) und \"janein\"", other)),
                };
                Value::Bool(crate::filedialog::confirm(
                    gs(a, 0, "GUI_CONFIRM")?, gs(a, 1, "GUI_CONFIRM")?, ja_nein))
            }
            #[cfg(not(feature = "dialogs"))]
            "file_open_dialog" | "file_save_dialog" | "folder_dialog"
            | "gui_message" | "gui_confirm" => {
                return Err(format!(
                    "{}: in diesem Build nicht verfuegbar -- native OS-Dialoge gibt es nur auf dem Desktop (Feature `dialogs`), nicht im Browser.",
                    name.to_uppercase()));
            }

            // --- Monitore / Display-Infos ---
            "monitor_count" => Value::Int(g!().monitor_count()),
            "current_monitor" => Value::Int(g!().current_monitor()),
            "monitor_width" => Value::Int(g!().monitor_width(gi(a, 0, "MONITOR_WIDTH")?)),
            "monitor_height" => Value::Int(g!().monitor_height(gi(a, 0, "MONITOR_HEIGHT")?)),
            "monitor_physical_width" => Value::Int(g!().monitor_mm(gi(a, 0, "MONITOR_PHYSICAL_WIDTH")?).0),
            "monitor_physical_height" => Value::Int(g!().monitor_mm(gi(a, 0, "MONITOR_PHYSICAL_HEIGHT")?).1),
            "monitor_refresh" => Value::Int(g!().monitor_refresh(gi(a, 0, "MONITOR_REFRESH")?)),
            "monitor_name" => { let s = g!().monitor_name(gi(a, 0, "MONITOR_NAME")?); Value::str_rc(&s) }
            "monitor_x" => Value::Int(g!().monitor_x(gi(a, 0, "MONITOR_X")?)),
            "monitor_y" => Value::Int(g!().monitor_y(gi(a, 0, "MONITOR_Y")?)),
            "set_window_monitor" => { g!().set_window_monitor(gi(a, 0, "SET_WINDOW_MONITOR")?); Value::Nil }
            "window_x" => Value::Int(g!().window_x()),
            "window_y" => Value::Int(g!().window_y()),
            "set_window_pos" => { g!().set_window_pos(gi(a, 0, "SET_WINDOW_POS")?, gi(a, 1, "SET_WINDOW_POS")?); Value::Nil }

            // --- Shader / Post-Processing ---
            "shader_load" => {
                // SHADER_LOAD(quelle$): Datei-Pfad ODER GLSL-Quelltext.
                let arg = gs(a, 0, "SHADER_LOAD")?;
                let resolved = crate::builtins::resolve_asset_path(arg);
                let code = match std::fs::read_to_string(&resolved) {
                    Ok(text) => text,
                    Err(_) => arg.to_string(), // kein Pfad -> als Code behandeln
                };
                Value::Int(g!().load_shader(&code))
            }
            "shader_set" => {
                g!().shader_set_float(gi(a,0,"SHADER_SET")?, gs(a,1,"SHADER_SET")?, need_f(a,2,"SHADER_SET")?);
                Value::Nil
            }
            "shader_set2" => {
                g!().shader_set_vec2(gi(a,0,"SHADER_SET2")?, gs(a,1,"SHADER_SET2")?,
                    need_f(a,2,"SHADER_SET2")?, need_f(a,3,"SHADER_SET2")?);
                Value::Nil
            }
            "shader_set3" => {
                g!().shader_set_vec3(gi(a,0,"SHADER_SET3")?, gs(a,1,"SHADER_SET3")?,
                    need_f(a,2,"SHADER_SET3")?, need_f(a,3,"SHADER_SET3")?, need_f(a,4,"SHADER_SET3")?);
                Value::Nil
            }
            "shader_set_array" => {
                let (h, n) = (gi(a,0,"SHADER_SET_ARRAY")?, gs(a,1,"SHADER_SET_ARRAY")?.to_string());
                let v = gfloats(a, 2, "SHADER_SET_ARRAY")?;
                g!().shader_set_array(h, &n, &v)?;
                Value::Nil
            }
            "shader_set_texture" => {
                let (h, n) = (gi(a,0,"SHADER_SET_TEXTURE")?, gs(a,1,"SHADER_SET_TEXTURE")?.to_string());
                let img = gi(a,2,"SHADER_SET_TEXTURE")?;
                g!().shader_set_texture(h, &n, img)?;
                Value::Nil
            }
            "shader_set_matrix" => {
                let (h, n) = (gi(a,0,"SHADER_SET_MATRIX")?, gs(a,1,"SHADER_SET_MATRIX")?.to_string());
                let m = match a.get(2) {
                    Some(Value::Mat4(m)) => **m,
                    _ => return Err("SHADER_SET_MATRIX: erwartet MAT4 (Modul m3d)".into()),
                };
                g!().shader_set_matrix(h, &n, &m)?;
                Value::Nil
            }
            "postfx" => {
                if a.is_empty() || a.len() > 2 {
                    return Err("POSTFX: erwartet (shader [, massstab])".into());
                }
                let m = if a.len() == 2 { Some(need_f(a, 1, "POSTFX")? as f64) } else { None };
                g!().set_postfx(gi(a, 0, "POSTFX")?, m)?;
                Value::Nil
            }

            // --- 3D (Modul g3d) ---
            "camera3d" => {
                g!().set_camera3d(
                    need_f(a,0,"CAMERA3D")? as f32, need_f(a,1,"CAMERA3D")? as f32, need_f(a,2,"CAMERA3D")? as f32,
                    need_f(a,3,"CAMERA3D")? as f32, need_f(a,4,"CAMERA3D")? as f32, need_f(a,5,"CAMERA3D")? as f32,
                    need_f(a,6,"CAMERA3D")? as f32);
                Value::Nil
            }
            "camera_orbit" => {
                let fovy = if a.len() >= 7 { need_f(a, 6, "CAMERA_ORBIT")? as f32 } else { 0.0 };
                g!().camera_orbit(
                    need_f(a,0,"CAMERA_ORBIT")? as f32, need_f(a,1,"CAMERA_ORBIT")? as f32, need_f(a,2,"CAMERA_ORBIT")? as f32,
                    need_f(a,3,"CAMERA_ORBIT")? as f32, need_f(a,4,"CAMERA_ORBIT")? as f32, need_f(a,5,"CAMERA_ORBIT")? as f32,
                    fovy);
                Value::Nil
            }
            "cube" | "cube_wires" => {
                let wires = name == "cube_wires";
                let f = if wires { "CUBE_WIRES" } else { "CUBE" };
                g!().cube(need_f(a,0,f)? as f32, need_f(a,1,f)? as f32, need_f(a,2,f)? as f32,
                          need_f(a,3,f)? as f32, need_f(a,4,f)? as f32, need_f(a,5,f)? as f32,
                          gi(a,6,f)?, wires);
                Value::Nil
            }
            "sphere" | "sphere_wires" => {
                let wires = name == "sphere_wires";
                let f = if wires { "SPHERE_WIRES" } else { "SPHERE" };
                g!().sphere(need_f(a,0,f)? as f32, need_f(a,1,f)? as f32, need_f(a,2,f)? as f32,
                            need_f(a,3,f)? as f32, gi(a,4,f)?, wires);
                Value::Nil
            }
            "cylinder" => {
                g!().cylinder(need_f(a,0,"CYLINDER")? as f32, need_f(a,1,"CYLINDER")? as f32, need_f(a,2,"CYLINDER")? as f32,
                              need_f(a,3,"CYLINDER")? as f32, need_f(a,4,"CYLINDER")? as f32, need_f(a,5,"CYLINDER")? as f32,
                              gi(a,6,"CYLINDER")?);
                Value::Nil
            }
            "plane" => {
                g!().plane(need_f(a,0,"PLANE")? as f32, need_f(a,1,"PLANE")? as f32, need_f(a,2,"PLANE")? as f32,
                           need_f(a,3,"PLANE")? as f32, need_f(a,4,"PLANE")? as f32, gi(a,5,"PLANE")?);
                Value::Nil
            }
            "line3d" => {
                g!().line3d(need_f(a,0,"LINE3D")? as f32, need_f(a,1,"LINE3D")? as f32, need_f(a,2,"LINE3D")? as f32,
                            need_f(a,3,"LINE3D")? as f32, need_f(a,4,"LINE3D")? as f32, need_f(a,5,"LINE3D")? as f32,
                            gi(a,6,"LINE3D")?);
                Value::Nil
            }
            "point3d" => {
                g!().point3d(need_f(a,0,"POINT3D")? as f32, need_f(a,1,"POINT3D")? as f32, need_f(a,2,"POINT3D")? as f32,
                             gi(a,3,"POINT3D")?);
                Value::Nil
            }
            "grid3d" => {
                g!().grid3d(gi(a,0,"GRID3D")? as i32, need_f(a,1,"GRID3D")? as f32);
                Value::Nil
            }
            "loadmodel" => Value::Int(g!().load_model(gs(a,0,"LOADMODEL")?)?),
            "model_load_anims" => Value::Int(g!().load_model_anims(gs(a,0,"MODEL_LOAD_ANIMS")?)?),
            "model_anim_count" => Value::Int(g!().anim_count(gi(a,0,"MODEL_ANIM_COUNT")?)?),
            "model_anim_frames" => Value::Int(g!().anim_frames(gi(a,0,"MODEL_ANIM_FRAMES")?, gi(a,1,"MODEL_ANIM_FRAMES")?)?),
            "model_anim_name" => Value::str_rc(&g!().anim_name(gi(a,0,"MODEL_ANIM_NAME")?, gi(a,1,"MODEL_ANIM_NAME")?)?),
            "model_animate" => { g!().model_animate(gi(a,0,"MODEL_ANIMATE")?, gi(a,1,"MODEL_ANIMATE")?, gi(a,2,"MODEL_ANIMATE")?, gi(a,3,"MODEL_ANIMATE")? as i32)?; Value::Nil }
            "model_animate_blend" => {
                g!().model_animate_blend(
                    gi(a,0,"MODEL_ANIMATE_BLEND")?, gi(a,1,"MODEL_ANIMATE_BLEND")?,
                    gi(a,2,"MODEL_ANIMATE_BLEND")?, gi(a,3,"MODEL_ANIMATE_BLEND")? as i32,
                    gi(a,4,"MODEL_ANIMATE_BLEND")?, gi(a,5,"MODEL_ANIMATE_BLEND")? as i32,
                    need_f(a,6,"MODEL_ANIMATE_BLEND")? as f32,
                )?;
                Value::Nil
            }
            "mesh_cube" => Value::Int(g!().mesh_cube(
                need_f(a,0,"MESH_CUBE")? as f32, need_f(a,1,"MESH_CUBE")? as f32, need_f(a,2,"MESH_CUBE")? as f32)?),
            "mesh_sphere" => Value::Int(g!().mesh_sphere(
                need_f(a,0,"MESH_SPHERE")? as f32, gi(a,1,"MESH_SPHERE")? as i32, gi(a,2,"MESH_SPHERE")? as i32)?),
            "mesh_cylinder" => Value::Int(g!().mesh_cylinder(
                need_f(a,0,"MESH_CYLINDER")? as f32, need_f(a,1,"MESH_CYLINDER")? as f32, gi(a,2,"MESH_CYLINDER")? as i32)?),
            "mesh_torus" => Value::Int(g!().mesh_torus(
                need_f(a,0,"MESH_TORUS")? as f32, need_f(a,1,"MESH_TORUS")? as f32, gi(a,2,"MESH_TORUS")? as i32, gi(a,3,"MESH_TORUS")? as i32)?),
            "mesh_knot" => Value::Int(g!().mesh_knot(
                need_f(a,0,"MESH_KNOT")? as f32, need_f(a,1,"MESH_KNOT")? as f32, gi(a,2,"MESH_KNOT")? as i32, gi(a,3,"MESH_KNOT")? as i32)?),
            "mesh_plane" => Value::Int(g!().mesh_plane(
                need_f(a,0,"MESH_PLANE")? as f32, need_f(a,1,"MESH_PLANE")? as f32, gi(a,2,"MESH_PLANE")? as i32, gi(a,3,"MESH_PLANE")? as i32)?),
            "mesh_cone" => Value::Int(g!().mesh_cone(need_f(a,0,"MESH_CONE")? as f32,
                need_f(a,1,"MESH_CONE")? as f32, gi(a,2,"MESH_CONE")? as i32)?),
            "mesh_hemisphere" => Value::Int(g!().mesh_hemisphere(need_f(a,0,"MESH_HEMISPHERE")? as f32,
                gi(a,1,"MESH_HEMISPHERE")? as i32, gi(a,2,"MESH_HEMISPHERE")? as i32)?),
            "mesh_poly" => Value::Int(g!().mesh_poly(gi(a,0,"MESH_POLY")? as i32, need_f(a,1,"MESH_POLY")? as f32)?),
            "mesh_cubicmap" => Value::Int(g!().mesh_cubicmap(gi(a,0,"MESH_CUBICMAP")?,
                need_f(a,1,"MESH_CUBICMAP")? as f32, need_f(a,2,"MESH_CUBICMAP")? as f32,
                need_f(a,3,"MESH_CUBICMAP")? as f32)?),
            "mesh_heightmap" => Value::Int(g!().mesh_heightmap(
                gi(a,0,"MESH_HEIGHTMAP")?, need_f(a,1,"MESH_HEIGHTMAP")? as f32, need_f(a,2,"MESH_HEIGHTMAP")? as f32, need_f(a,3,"MESH_HEIGHTMAP")? as f32)?),
            "model" => {
                g!().draw_model(gi(a,0,"MODEL")?, need_f(a,1,"MODEL")? as f32, need_f(a,2,"MODEL")? as f32,
                                need_f(a,3,"MODEL")? as f32, need_f(a,4,"MODEL")? as f32, gi(a,5,"MODEL")?)?;
                Value::Nil
            }
            "model_ex" => {
                g!().draw_model_ex(gi(a,0,"MODEL_EX")?, need_f(a,1,"MODEL_EX")? as f32, need_f(a,2,"MODEL_EX")? as f32, need_f(a,3,"MODEL_EX")? as f32,
                                   need_f(a,4,"MODEL_EX")? as f32, need_f(a,5,"MODEL_EX")? as f32, need_f(a,6,"MODEL_EX")? as f32,
                                   need_f(a,7,"MODEL_EX")? as f32, need_f(a,8,"MODEL_EX")? as f32, gi(a,9,"MODEL_EX")?)?;
                Value::Nil
            }
            "model_wires" => {
                g!().draw_model_wires(gi(a,0,"MODEL_WIRES")?, need_f(a,1,"MODEL_WIRES")? as f32, need_f(a,2,"MODEL_WIRES")? as f32,
                                      need_f(a,3,"MODEL_WIRES")? as f32, need_f(a,4,"MODEL_WIRES")? as f32, gi(a,5,"MODEL_WIRES")?)?;
                Value::Nil
            }
            "model_matrix" => {
                // MODEL_MATRIX(handle, mat [, tint]) -- Welt-Transform aus m3d MAT4.
                let mat = match a.get(1) {
                    Some(Value::Mat4(m)) => m.clone(),
                    _ => return Err("MODEL_MATRIX: Arg 2 muss MAT4 sein".into()),
                };
                let tint = if a.len() >= 3 { gi(a, 2, "MODEL_MATRIX")? } else { 0xFF_FFFF };
                g!().draw_model_matrix(gi(a, 0, "MODEL_MATRIX")?, mat, tint)?;
                Value::Nil
            }
            "cube_matrix_wires" => {
                // CUBE_MATRIX_WIRES(mat [, farbe]) -- Kanten eines Einheitswuerfels.
                let mat = match a.first() {
                    Some(Value::Mat4(m)) => m.clone(),
                    _ => return Err("CUBE_MATRIX_WIRES: Arg 1 muss MAT4 sein".into()),
                };
                let farbe = if a.len() >= 2 { gi(a, 1, "CUBE_MATRIX_WIRES")? } else { 0xFF_FFFF };
                g!().draw_cube_matrix_wires(mat, farbe);
                Value::Nil
            }
            "model_matrix_wires" => {
                // MODEL_MATRIX_WIRES(handle, mat [, farbe]) -- Umriss zu MODEL_MATRIX.
                let mat = match a.get(1) {
                    Some(Value::Mat4(m)) => m.clone(),
                    _ => return Err("MODEL_MATRIX_WIRES: Arg 2 muss MAT4 sein".into()),
                };
                let farbe = if a.len() >= 3 { gi(a, 2, "MODEL_MATRIX_WIRES")? } else { 0xFF_FFFF };
                g!().draw_model_matrix_wires(gi(a, 0, "MODEL_MATRIX_WIRES")?, mat, farbe)?;
                Value::Nil
            }
            "model_instanced" => {
                // MODEL_INSTANCED(handle, matrizen [, tint]) -- GPU-Instancing:
                // dasselbe Modell mit N Welt-Matrizen (ARRAY OF MAT4 / TUPLE von
                // MAT4) in EINEM Draw-Call. mat -> [f32;16] (column-major) sammeln.
                let collect_mats = |els: &mut dyn Iterator<Item = &Value>| -> R<Vec<[f32; 16]>> {
                    let mut v = Vec::new();
                    for (k, el) in els.enumerate() {
                        match el {
                            Value::Mat4(m) => v.push(**m),
                            other => return Err(format!(
                                "MODEL_INSTANCED: Element {} ist kein MAT4 (sondern {})",
                                k, other.type_name())),
                        }
                    }
                    Ok(v)
                };
                // Optionale Stueckzahl wie bei PLOTS/LINES/... -- ohne sie wird das
                // ganze Array gezeichnet, und ein fest dimensionierter Puffer
                // schleppt seine ungenutzten Plaetze mit ins Bild. Sie muss VOR
                // dem Einsammeln greifen: sonst stolpert das Einsammeln ueber
                // die noch nicht belegten (NIL-)Plaetze, die gar nicht gezeichnet
                // werden sollen.
                let mats = match a.get(1) {
                    Some(Value::Array(arr)) => {
                        let b = arr.borrow();
                        if b.dims.len() != 1 {
                            return Err("MODEL_INSTANCED: Arg 2 muss ein 1D-ARRAY OF MAT4 sein".into());
                        }
                        match b.cells.as_vals() {
                            Some(vals) => {
                                let n = bulk_count(a, 3, vals.len(), "MODEL_INSTANCED")?;
                                collect_mats(&mut vals.iter().take(n))?
                            }
                            None => return Err("MODEL_INSTANCED: Arg 2 muss ein ARRAY OF MAT4 sein".into()),
                        }
                    }
                    Some(Value::Tuple(t)) => {
                        let n = bulk_count(a, 3, t.len(), "MODEL_INSTANCED")?;
                        collect_mats(&mut t.iter().take(n))?
                    }
                    _ => return Err("MODEL_INSTANCED: Arg 2 muss ARRAY OF MAT4 oder TUPLE von MAT4 sein".into()),
                };
                let handle = gi(a, 0, "MODEL_INSTANCED")?;
                // tint darf eine Farbe ODER ein ARRAY OF INTEGER sein (eine Farbe
                // je Matrix). Die Farbe je Instanz reist in der untersten Zeile
                // der Matrix mit (graphics.rs, INST_VS) -- EIN Zeichenaufruf,
                // egal wie viele Farben.
                match a.get(2) {
                    Some(Value::Array(arr)) => {
                        let farben = {
                            let b = arr.borrow();
                            if b.dims.len() != 1 {
                                return Err("MODEL_INSTANCED: tint-Array muss 1D sein".into());
                            }
                            if b.cells.len() < mats.len() {
                                return Err(format!(
                                    "MODEL_INSTANCED: tint-Array ist kuerzer als die Matrizen ({} < {})",
                                    b.cells.len(), mats.len()));
                            }
                            let mut v = Vec::with_capacity(mats.len());
                            for x in b.cells.iter().take(mats.len()) {
                                match x {
                                    Value::Int(i) => v.push(i),
                                    other => return Err(format!(
                                        "MODEL_INSTANCED: tint-Array braucht INTEGER-Farben (erhalten {})",
                                        other.type_name())),
                                }
                            }
                            v
                        };
                        g!().draw_model_instanced_farben(handle, mats, &farben)?;
                    }
                    _ => {
                        let tint = if a.len() >= 3 { gi(a, 2, "MODEL_INSTANCED")? } else { 0xFF_FFFF };
                        g!().draw_model_instanced(handle, mats, tint)?;
                    }
                }
                Value::Nil
            }
            "model_texture" => {
                g!().model_set_texture(gi(a,0,"MODEL_TEXTURE")?, gi(a,1,"MODEL_TEXTURE")?)?;
                Value::Nil
            }
            "model_texture_normal" => {
                g!().model_set_normal(gi(a,0,"MODEL_TEXTURE_NORMAL")?, gi(a,1,"MODEL_TEXTURE_NORMAL")?)?;
                Value::Nil
            }
            "model_pbr" => {
                g!().model_pbr(gi(a,0,"MODEL_PBR")?, need_f(a,1,"MODEL_PBR")?, need_f(a,2,"MODEL_PBR")?)?;
                Value::Nil
            }
            "model_emissive" => {
                g!().model_emissive(gi(a,0,"MODEL_EMISSIVE")?, gi(a,1,"MODEL_EMISSIVE")?, need_f(a,2,"MODEL_EMISSIVE")?)?;
                Value::Nil
            }
            "billboard" => {
                g!().billboard(gi(a,0,"BILLBOARD")?, need_f(a,1,"BILLBOARD")? as f32, need_f(a,2,"BILLBOARD")? as f32,
                               need_f(a,3,"BILLBOARD")? as f32, need_f(a,4,"BILLBOARD")? as f32, gi(a,5,"BILLBOARD")?)?;
                Value::Nil
            }
            "billboard_part" => {
                if a.len() != 10 {
                    return Err(format!("BILLBOARD_PART: erwartet (bild, sx, sy, sb, sh, x, y, z, groesse, farbe), erhalten {} Argumente", a.len()));
                }
                let f = "BILLBOARD_PART";
                g!().billboard_part(gi(a,0,f)?,
                    [need_f(a,1,f)? as f32, need_f(a,2,f)? as f32, need_f(a,3,f)? as f32, need_f(a,4,f)? as f32],
                    need_f(a,5,f)? as f32, need_f(a,6,f)? as f32, need_f(a,7,f)? as f32,
                    need_f(a,8,f)? as f32, gi(a,9,f)?)?;
                Value::Nil
            }
            "billboard_ex" => {
                if a.len() != 8 {
                    return Err(format!("BILLBOARD_EX: erwartet (bild, x, y, z, breite, hoehe, winkel, farbe), erhalten {} Argumente", a.len()));
                }
                let f = "BILLBOARD_EX";
                g!().billboard_ex(gi(a,0,f)?, need_f(a,1,f)? as f32, need_f(a,2,f)? as f32, need_f(a,3,f)? as f32,
                    need_f(a,4,f)? as f32, need_f(a,5,f)? as f32, need_f(a,6,f)? as f32, gi(a,7,f)?)?;
                Value::Nil
            }
            "capsule" | "capsule_wires" => {
                let f = if name == "capsule" { "CAPSULE" } else { "CAPSULE_WIRES" };
                if a.len() != 8 {
                    return Err(format!("{}: erwartet (x1, y1, z1, x2, y2, z2, r, farbe), erhalten {} Argumente", f, a.len()));
                }
                let mut p = [0f32; 6];
                for (i, v) in p.iter_mut().enumerate() { *v = need_f(a, i, f)? as f32; }
                g!().capsule(p, need_f(a,6,f)? as f32, gi(a,7,f)?, name == "capsule_wires");
                Value::Nil
            }
            "cylinder_ex" => {
                let f = "CYLINDER_EX";
                if a.len() != 9 {
                    return Err(format!("CYLINDER_EX: erwartet (x1, y1, z1, x2, y2, z2, r_anfang, r_ende, farbe), erhalten {} Argumente", a.len()));
                }
                let mut p = [0f32; 6];
                for (i, v) in p.iter_mut().enumerate() { *v = need_f(a, i, f)? as f32; }
                g!().cylinder_ex(p, need_f(a,6,f)? as f32, need_f(a,7,f)? as f32, gi(a,8,f)?);
                Value::Nil
            }
            "cylinder_wires" => {
                let f = "CYLINDER_WIRES";
                if a.len() != 7 {
                    return Err(format!("CYLINDER_WIRES: erwartet (x, y, z, r_oben, r_unten, hoehe, farbe), erhalten {} Argumente", a.len()));
                }
                g!().cylinder_wires(need_f(a,0,f)? as f32, need_f(a,1,f)? as f32, need_f(a,2,f)? as f32,
                    need_f(a,3,f)? as f32, need_f(a,4,f)? as f32, need_f(a,5,f)? as f32, gi(a,6,f)?);
                Value::Nil
            }
            "triangle3d" => {
                let f = "TRIANGLE3D";
                if a.len() != 10 {
                    return Err(format!("TRIANGLE3D: erwartet (x1, y1, z1, x2, y2, z2, x3, y3, z3, farbe), erhalten {} Argumente", a.len()));
                }
                let mut p = [0f32; 9];
                for (i, v) in p.iter_mut().enumerate() { *v = need_f(a, i, f)? as f32; }
                g!().triangle3d(p, gi(a,9,f)?);
                Value::Nil
            }
            "circle3d" => {
                let f = "CIRCLE3D";
                if a.len() != 5 {
                    return Err(format!("CIRCLE3D: erwartet (x, y, z, r, farbe), erhalten {} Argumente", a.len()));
                }
                g!().circle3d(need_f(a,0,f)? as f32, need_f(a,1,f)? as f32, need_f(a,2,f)? as f32,
                    need_f(a,3,f)? as f32, gi(a,4,f)?);
                Value::Nil
            }
            "bbox_wires" => {
                let f = "BBOX_WIRES";
                if a.len() != 7 {
                    return Err(format!("BBOX_WIRES: erwartet (x1, y1, z1, x2, y2, z2, farbe), erhalten {} Argumente", a.len()));
                }
                let mut p = [0f32; 6];
                for (i, v) in p.iter_mut().enumerate() { *v = need_f(a, i, f)? as f32; }
                g!().bbox_wires(p, gi(a,6,f)?);
                Value::Nil
            }
            "model_bbox" => {
                let b = g!().model_bbox(gi(a,0,"MODEL_BBOX")?)?;
                // raylib rechnet in f32: 1.2 kaeme als 1.2000000476837158 an.
                // Ueber die kuerzeste Darstellung der f32 gewandelt bleibt es 1.2.
                Value::Tuple(Rc::new(b.iter()
                    .map(|&v| Value::Float(v.to_string().parse::<f64>().unwrap_or(v as f64)))
                    .collect()))
            }
            "ray_hit_box" => Value::Float(g!().ray_hit_box(
                need_f(a,0,"RAY_HIT_BOX")? as f32, need_f(a,1,"RAY_HIT_BOX")? as f32, need_f(a,2,"RAY_HIT_BOX")? as f32,
                need_f(a,3,"RAY_HIT_BOX")? as f32, need_f(a,4,"RAY_HIT_BOX")? as f32, need_f(a,5,"RAY_HIT_BOX")? as f32,
                need_f(a,6,"RAY_HIT_BOX")? as f32, need_f(a,7,"RAY_HIT_BOX")? as f32, need_f(a,8,"RAY_HIT_BOX")? as f32,
                need_f(a,9,"RAY_HIT_BOX")? as f32, need_f(a,10,"RAY_HIT_BOX")? as f32, need_f(a,11,"RAY_HIT_BOX")? as f32)),
            "ray_hit_sphere" => Value::Float(g!().ray_hit_sphere(
                need_f(a,0,"RAY_HIT_SPHERE")? as f32, need_f(a,1,"RAY_HIT_SPHERE")? as f32, need_f(a,2,"RAY_HIT_SPHERE")? as f32,
                need_f(a,3,"RAY_HIT_SPHERE")? as f32, need_f(a,4,"RAY_HIT_SPHERE")? as f32, need_f(a,5,"RAY_HIT_SPHERE")? as f32,
                need_f(a,6,"RAY_HIT_SPHERE")? as f32, need_f(a,7,"RAY_HIT_SPHERE")? as f32, need_f(a,8,"RAY_HIT_SPHERE")? as f32,
                need_f(a,9,"RAY_HIT_SPHERE")? as f32)),
            // Picking auf ECHTER Geometrie statt nur Huellkoerper: einzelne
            // Dreiecke/Vierecke (Boden-Kacheln, Wandflaechen, In-Welt-Panels).
            "ray_hit_tri" => {
                let v = fv(a, 15, "RAY_HIT_TRI")?;
                Value::Float(g!().ray_hit_tri([v[0],v[1],v[2]], [v[3],v[4],v[5]],
                    [[v[6],v[7],v[8]], [v[9],v[10],v[11]], [v[12],v[13],v[14]]]))
            }
            "ray_hit_quad" => {
                let v = fv(a, 18, "RAY_HIT_QUAD")?;
                Value::Float(g!().ray_hit_quad([v[0],v[1],v[2]], [v[3],v[4],v[5]],
                    [[v[6],v[7],v[8]], [v[9],v[10],v[11]], [v[12],v[13],v[14]], [v[15],v[16],v[17]]]))
            }
            "pick_tri" => {
                let v = fv(a, 9, "PICK_TRI")?;
                Value::Float(g!().pick_tri([[v[0],v[1],v[2]], [v[3],v[4],v[5]], [v[6],v[7],v[8]]]))
            }
            "pick_quad" => {
                let v = fv(a, 12, "PICK_QUAD")?;
                Value::Float(g!().pick_quad([[v[0],v[1],v[2]], [v[3],v[4],v[5]],
                                             [v[6],v[7],v[8]], [v[9],v[10],v[11]]]))
            }
            "pick_box" => Value::Float(g!().pick_box(
                need_f(a,0,"PICK_BOX")? as f32, need_f(a,1,"PICK_BOX")? as f32, need_f(a,2,"PICK_BOX")? as f32,
                need_f(a,3,"PICK_BOX")? as f32, need_f(a,4,"PICK_BOX")? as f32, need_f(a,5,"PICK_BOX")? as f32)),
            "pick_sphere" => Value::Float(g!().pick_sphere(
                need_f(a,0,"PICK_SPHERE")? as f32, need_f(a,1,"PICK_SPHERE")? as f32, need_f(a,2,"PICK_SPHERE")? as f32,
                need_f(a,3,"PICK_SPHERE")? as f32)),
            "ray_hit_model" => {
                let scale = if a.len() >= 11 { need_f(a,10,"RAY_HIT_MODEL")? as f32 } else { 1.0 };
                Value::Float(g!().ray_hit_model(gi(a,0,"RAY_HIT_MODEL")?,
                    need_f(a,1,"RAY_HIT_MODEL")? as f32, need_f(a,2,"RAY_HIT_MODEL")? as f32, need_f(a,3,"RAY_HIT_MODEL")? as f32,
                    need_f(a,4,"RAY_HIT_MODEL")? as f32, need_f(a,5,"RAY_HIT_MODEL")? as f32, need_f(a,6,"RAY_HIT_MODEL")? as f32,
                    need_f(a,7,"RAY_HIT_MODEL")? as f32, need_f(a,8,"RAY_HIT_MODEL")? as f32, need_f(a,9,"RAY_HIT_MODEL")? as f32, scale))
            }
            "pick_model" => {
                let scale = if a.len() >= 5 { need_f(a,4,"PICK_MODEL")? as f32 } else { 1.0 };
                Value::Float(g!().pick_model(gi(a,0,"PICK_MODEL")?,
                    need_f(a,1,"PICK_MODEL")? as f32, need_f(a,2,"PICK_MODEL")? as f32, need_f(a,3,"PICK_MODEL")? as f32, scale))
            }
            "world_to_screen_x" => Value::Float(g!().world_to_screen(
                need_f(a,0,"WORLD_TO_SCREEN_X")? as f32, need_f(a,1,"WORLD_TO_SCREEN_X")? as f32, need_f(a,2,"WORLD_TO_SCREEN_X")? as f32).0 as f64),
            "world_to_screen_y" => Value::Float(g!().world_to_screen(
                need_f(a,0,"WORLD_TO_SCREEN_Y")? as f32, need_f(a,1,"WORLD_TO_SCREEN_Y")? as f32, need_f(a,2,"WORLD_TO_SCREEN_Y")? as f32).1 as f64),
            "screen_to_world_dir_x" => Value::Float(g!().screen_ray_dir(
                need_f(a,0,"SCREEN_TO_WORLD_DIR_X")? as f32, need_f(a,1,"SCREEN_TO_WORLD_DIR_X")? as f32).0 as f64),
            "screen_to_world_dir_y" => Value::Float(g!().screen_ray_dir(
                need_f(a,0,"SCREEN_TO_WORLD_DIR_Y")? as f32, need_f(a,1,"SCREEN_TO_WORLD_DIR_Y")? as f32).1 as f64),
            "screen_to_world_dir_z" => Value::Float(g!().screen_ray_dir(
                need_f(a,0,"SCREEN_TO_WORLD_DIR_Z")? as f32, need_f(a,1,"SCREEN_TO_WORLD_DIR_Z")? as f32).2 as f64),
            "getpixel" => Value::Int(g!().get_pixel(gi(a,0,"GETPIXEL")?, gi(a,1,"GETPIXEL")? as i32, gi(a,2,"GETPIXEL")? as i32)),
            "mouse_ground_x" => Value::Float(g!().mouse_ground(need_f(a,0,"MOUSE_GROUND_X")? as f32).0 as f64),
            "mouse_ground_z" => Value::Float(g!().mouse_ground(need_f(a,0,"MOUSE_GROUND_Z")? as f32).1 as f64),
            "mouse_ground_hit" => Value::Bool(g!().mouse_ground(need_f(a,0,"MOUSE_GROUND_HIT")? as f32).2),

            // --- Kamera-Modi (raylib UpdateCamera) ---
            "camera3d_update" => { g!().camera3d_update(gi(a,0,"CAMERA3D_UPDATE")?); Value::Nil }
            "camera3d_move" => {
                if a.len() != 5 && a.len() != 6 { return Err("CAMERA3D_MOVE: erwartet (vor, rechts, hoch, gieren, neigen[, zoom])".into()); }
                let mut v = [0f32; 6];
                for (i, w) in v.iter_mut().enumerate().take(a.len()) {
                    let x = need_f(a, i, "CAMERA3D_MOVE")?;
                    if !x.is_finite() { return Err(format!("CAMERA3D_MOVE: Argument {} muss eine endliche Zahl sein, nicht {}", i + 1, x)); }
                    *w = x as f32;
                }
                g!().camera3d_move(v[0], v[1], v[2], v[3], v[4], v[5]);
                Value::Nil
            }
            "model_save" => { g!().model_save(gi(a,0,"MODEL_SAVE")?, gs(a,1,"MODEL_SAVE")?)?; Value::Nil }
            // m3d: View-/Projektions-Matrix-Override (Ortho, Custom-Frustum, Gizmos).
            // CAMERA3D(...) setzt beide zurueck auf Standard-Perspektive.
            "camera3d_view" => {
                let m = match a.first() { Some(Value::Mat4(m)) => **m, _ => return Err("CAMERA3D_VIEW: Arg 1 muss MAT4 sein".into()) };
                g!().set_camera3d_view(m); Value::Nil
            }
            "camera3d_projection" => {
                let m = match a.first() { Some(Value::Mat4(m)) => **m, _ => return Err("CAMERA3D_PROJECTION: Arg 1 muss MAT4 sein".into()) };
                g!().set_camera3d_projection(m); Value::Nil
            }
            "camera3d_x" => Value::Float(g!().cam3d_pos().0),
            "camera3d_y" => Value::Float(g!().cam3d_pos().1),
            "camera3d_z" => Value::Float(g!().cam3d_pos().2),
            "camera3d_target_x" => Value::Float(g!().cam3d_target().0),
            "camera3d_target_y" => Value::Float(g!().cam3d_target().1),
            "camera3d_target_z" => Value::Float(g!().cam3d_target().2),

            // --- Beleuchtung (Blinn-Phong) ---
            "light_enable" => { g!().light_enable(); Value::Nil }
            "light_ambient" => { g!().light_ambient(gi(a,0,"LIGHT_AMBIENT")?, need_f(a,1,"LIGHT_AMBIENT")?); Value::Nil }
            "light_fog" => { g!().light_fog(gi(a,0,"LIGHT_FOG")?, need_f(a,1,"LIGHT_FOG")?); Value::Nil }
            "light_env" => { g!().light_env(gi(a,0,"LIGHT_ENV")?, gi(a,1,"LIGHT_ENV")?, need_f(a,2,"LIGHT_ENV")?); Value::Nil }
            "light_env_hdr" => {
                // LIGHT_ENV_HDR(pfad$[, intensitaet]) -- echtes HDR-Cubemap-IBL.
                let path = gs(a, 0, "LIGHT_ENV_HDR")?.to_string();
                let intensity = if a.len() >= 2 { need_f(a, 1, "LIGHT_ENV_HDR")? } else { 1.0 };
                g!().light_env_hdr(&path, intensity)?; Value::Nil
            }
            "skybox" => { g!().skybox(gb(a, 0)); Value::Nil }
            "skybox_image" => { g!().skybox_image(gi(a, 0, "SKYBOX_IMAGE")?)?; Value::Nil }
            "image_mipmaps" => Value::Int(g!().image_mipmaps(gi(a, 0, "IMAGE_MIPMAPS")?)?),
            "light_directional" => Value::Int(g!().light_add(
                0, need_f(a,0,"LIGHT_DIRECTIONAL")? as f32, need_f(a,1,"LIGHT_DIRECTIONAL")? as f32,
                need_f(a,2,"LIGHT_DIRECTIONAL")? as f32, gi(a,3,"LIGHT_DIRECTIONAL")?)),
            "light_point" => Value::Int(g!().light_add(
                1, need_f(a,0,"LIGHT_POINT")? as f32, need_f(a,1,"LIGHT_POINT")? as f32,
                need_f(a,2,"LIGHT_POINT")? as f32, gi(a,3,"LIGHT_POINT")?)),
            "light_set_pos" => { g!().light_set_pos(gi(a,0,"LIGHT_SET_POS")?,
                need_f(a,1,"LIGHT_SET_POS")? as f32, need_f(a,2,"LIGHT_SET_POS")? as f32, need_f(a,3,"LIGHT_SET_POS")? as f32)?; Value::Nil }
            "light_set_color" => { g!().light_set_color(gi(a,0,"LIGHT_SET_COLOR")?, gi(a,1,"LIGHT_SET_COLOR")?)?; Value::Nil }
            "light_set_enabled" => { g!().light_set_enabled(gi(a,0,"LIGHT_SET_ENABLED")?, gb(a,1))?; Value::Nil }
            "model_lit" => { g!().model_lit(gi(a,0,"MODEL_LIT")?)?; Value::Nil }
            "shadow_enable" => {
                let res = if !a.is_empty() { gi(a,0,"SHADOW_ENABLE")? as i32 } else { 1024 };
                g!().shadow_enable(res)?; Value::Nil
            }
            "shadow_area" => { g!().shadow_area(need_f(a,0,"SHADOW_AREA")?, need_f(a,1,"SHADOW_AREA")?); Value::Nil }
            "shadow_target" => { g!().shadow_target(
                need_f(a,0,"SHADOW_TARGET")? as f32, need_f(a,1,"SHADOW_TARGET")? as f32, need_f(a,2,"SHADOW_TARGET")? as f32); Value::Nil }

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
            "sample_set_loop" => {
                // SAMPLE_SET_LOOP(sample, start, end[, art$]) -- Loop-Region in
                // Frames; art$ = "vorwaerts" (Vorgabe) oder "pingpong".
                let idx = gi(a, 0, "SAMPLE_SET_LOOP")?;
                let start = gi(a, 1, "SAMPLE_SET_LOOP")?;
                let end = gi(a, 2, "SAMPLE_SET_LOOP")?;
                let art = if a.len() >= 4 { Some(gs(a, 3, "SAMPLE_SET_LOOP")?.to_string()) } else { None };
                self.audio_mut()?.sample_set_loop(idx, start, end, art.as_deref())?;
                Value::Nil
            }
            "sample_from_buffer" => {
                // SAMPLE_FROM_BUFFER(puffer, abtastrate) -> SAMPLE (16-Bit-PCM,
                // little endian, mono -- so liegen die Samples im Tracker-Format)
                let bytes = match a.first() {
                    Some(Value::Buffer(b)) => b.borrow().clone(),
                    Some(v) => return Err(format!(
                        "SAMPLE_FROM_BUFFER: erwartet BUFFER, erhalten {}", v.type_name())),
                    None => return Err("SAMPLE_FROM_BUFFER: erwartet (puffer, abtastrate)".into()),
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
                    return Err("SAMPLE_NOTE: erwartet (sample, halbtoene, dauer_ms, attack_ms, decay_ms, sustain, release_ms, vol[, slide_halbtoene])".into());
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
                    Some(v) => v.truthy(),
                    None => return Err("AUDIO_LOFI: erwartet (an[, bits[, cutoff_hz]])".into()),
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
                let time_ms = if a.len() >= 4 { gi(a, 3, "AUDIO_DELAY")? } else { 0 };
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
                // PLAYSOUND(sound[, loops, volume]). `loops` wird nativ ignoriert
                // (raylib-Sounds loopen nicht) -- SFX spielen einmal.
                let idx = gi(a, 0, "PLAYSOUND")?;
                let vol = if a.len() >= 3 { need_f(a, 2, "PLAYSOUND")? } else { 1.0 };
                self.audio_mut()?.play_sound(idx, vol)?;
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
                // PLAYMUSIC(pfad$[, loops, volume]). Musik loopt (raylib-Default);
                // `loops` wird nativ nicht ausgewertet.
                let path = gs(a, 0, "PLAYMUSIC")?.to_string();
                let vol = if a.len() >= 3 { need_f(a, 2, "PLAYMUSIC")? } else { 1.0 };
                self.audio_mut()?.play_music(&path, vol)?;
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
            "audio_is_playing" => { let i = gi(a, 0, "AUDIO_IS_PLAYING")?; Value::Bool(self.audio_mut()?.ch_is_playing(i)?) }
            "audio_is_waiting" => { let i = gi(a, 0, "AUDIO_IS_WAITING")?; Value::Bool(self.audio_mut()?.ch_is_waiting(i)?) }
            "audio_record_start" => {
                if a.len() > 1 { return Err(format!("AUDIO_RECORD_START: erwartet 0..1 Argumente, erhalten {}", a.len())); }
                let m = if a.is_empty() { 600.0 } else { need_f(a, 0, "AUDIO_RECORD_START")? };
                self.audio_mut()?.record_start(m)?; Value::Nil
            }
            "audio_record_stop" => { Value::Int(self.audio_mut()?.record_stop()?) }
            "audio_recording" => { Value::Bool(self.audio_mut()?.recording()) }
            "audio_peak" => { Value::Float(round_audio(self.audio_mut()?.peak())) }
            "audio_volume" | "audio_set_volume" => { let i = gi(a, 0, "AUDIO_VOLUME")?; let v = need_f(a, 1, "AUDIO_VOLUME")?; self.audio_mut()?.ch_set_volume(i, v)?; Value::Nil }
            "audio_get_volume" => { let i = gi(a, 0, "AUDIO_GET_VOLUME")?; Value::Float(round_audio(self.audio_mut()?.ch_get_volume(i)?)) }
            "audio_pan" => { let i = gi(a, 0, "AUDIO_PAN")?; let l = need_f(a, 1, "AUDIO_PAN")?; let r = need_f(a, 2, "AUDIO_PAN")?; self.audio_mut()?.ch_pan(i, l, r)?; Value::Nil }
            "audio_pitch" => {
                let i = gi(a, 0, "AUDIO_PITCH")?;
                let f = need_f(a, 1, "AUDIO_PITCH")?;
                if f <= 0.0 { return Err("AUDIO_PITCH: faktor muss > 0 sein".into()); }
                self.audio_mut()?.ch_pitch(i, f)?; Value::Nil
            }
            "audio_music_pitch" => {
                let f = need_f(a, 0, "AUDIO_MUSIC_PITCH")?;
                if f <= 0.0 { return Err("AUDIO_MUSIC_PITCH: faktor muss > 0 sein".into()); }
                self.audio_mut()?.music_set_pitch(f); Value::Nil
            }
            "audio_music_get_pitch" => Value::Float(self.audio_mut()?.music_get_pitch()),
            "audio_pan_pos" => { let i = gi(a, 0, "AUDIO_PAN_POS")?; let p = need_f(a, 1, "AUDIO_PAN_POS")?; self.audio_mut()?.ch_pan_pos(i, p)?; Value::Nil }
            "audio_pan_slide" => {
                // AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms[, easing$]) -- Positionen 0=links..1=rechts
                let i = gi(a, 0, "AUDIO_PAN_SLIDE")?;
                let von = need_f(a, 1, "AUDIO_PAN_SLIDE")?;
                let nach = need_f(a, 2, "AUDIO_PAN_SLIDE")?;
                let dauer = gi(a, 3, "AUDIO_PAN_SLIDE")?;
                if dauer <= 0 { return Err("AUDIO_PAN_SLIDE: dauer_ms muss > 0 sein".into()); }
                let easing = easing_name(a, 4, "AUDIO_PAN_SLIDE")?;
                self.audio_mut()?.ch_pan_slide(i, von, nach, dauer, &easing)?; Value::Nil
            }
            "audio_autopan" => {
                // AUDIO_AUTOPAN(ch, periode_s[, tiefe]) -- periode_s <= 0 = aus
                let i = gi(a, 0, "AUDIO_AUTOPAN")?;
                let periode = need_f(a, 1, "AUDIO_AUTOPAN")?;
                let tiefe = if a.len() >= 3 { need_f(a, 2, "AUDIO_AUTOPAN")? } else { 1.0 };
                self.audio_mut()?.ch_autopan(i, periode, tiefe)?; Value::Nil
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
            // --- Sprachausgabe (sprache.rs, Weg C aus docs/entwurf-speak.md) ---
            // SPEAK fragt ZUERST, ob ein Bildschirmleser zuhoert: dann geht der
            // Satz als Ansage in den Baum (seine Stimme, sein Tempo, seine
            // Braillezeile -- und nicht zwei Stimmen zugleich). Sonst die
            // Systemstimme durch Kira. SPEAK_SOUND ist immer Synthese.
            "speak" => {
                let text = gs(a, 0, "SPEAK")?.to_string();
                let unterbrechen = a.len() > 1 && gflag(a, 1);
                if a.len() > 2 { return Err("SPEAK: erwartet (text$[, unterbrechen])".into()); }
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
                if a.len() != 1 { return Err("SPEAK_SOUND: erwartet (text$)".into()); }
                self.audio_mut()?;
                let au = self.audio.as_mut().unwrap();
                Value::Int(self.sprecher.klang(au, &text)?)
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
            "audio_music_volume" | "audio_music_set_volume" => { let v = need_f(a, 0, "AUDIO_MUSIC_VOLUME")?; self.audio_mut()?.music_set_volume(v); Value::Nil }
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
            "audio_music_queue" => { let p = gs(a, 0, "AUDIO_MUSIC_QUEUE")?.to_string(); self.audio_mut()?.music_queue(&p); Value::Nil }

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

            // --- Bulk-Draws ---
            "plots" => {
                let xs = arr_i32(&a[0], "PLOTS")?; let ys = arr_i32(&a[1], "PLOTS")?;
                let n = bulk_count(a, 3, xs.len().min(ys.len()), "PLOTS")?;
                let cols = bulk_color(&a[2], n, "PLOTS")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.plot(xs[i], ys[i], cols[i]); }
                Value::Nil
            }
            "boxes" => {
                let x1=arr_i32(&a[0],"BOXES")?; let y1=arr_i32(&a[1],"BOXES")?; let x2=arr_i32(&a[2],"BOXES")?; let y2=arr_i32(&a[3],"BOXES")?;
                let n = bulk_count(a, 5, x1.len().min(y1.len()).min(x2.len()).min(y2.len()), "BOXES")?;
                let cols = bulk_color(&a[4], n, "BOXES")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.box_fill(x1[i], y1[i], x2[i], y2[i], cols[i]); }
                Value::Nil
            }
            "circles" => {
                let xs=arr_i32(&a[0],"CIRCLES")?; let ys=arr_i32(&a[1],"CIRCLES")?; let rs=arr_i32(&a[2],"CIRCLES")?;
                let n = bulk_count(a, 4, xs.len().min(ys.len()).min(rs.len()), "CIRCLES")?;
                let cols = bulk_color(&a[3], n, "CIRCLES")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.circle(xs[i], ys[i], rs[i], cols[i]); }
                Value::Nil
            }
            "lines" => {
                let x1=arr_i32(&a[0],"LINES")?; let y1=arr_i32(&a[1],"LINES")?; let x2=arr_i32(&a[2],"LINES")?; let y2=arr_i32(&a[3],"LINES")?;
                let n = bulk_count(a, 5, x1.len().min(y1.len()).min(x2.len()).min(y2.len()), "LINES")?;
                let cols = bulk_color(&a[4], n, "LINES")?;
                let g = self.gfx.as_mut().ok_or("Grafik-Builtin vor SCREEN aufgerufen")?;
                for i in 0..n { g.line(x1[i], y1[i], x2[i], y2[i], cols[i]); }
                Value::Nil
            }

            // --- Bilder erweitert ---
            "drawimagepart" => {
                let idx = gi(a,0,"DRAWIMAGEPART")?;
                g!().draw_image_part(idx, gi(a,1,"DRAWIMAGEPART")? as i32, gi(a,2,"DRAWIMAGEPART")? as i32,
                    gi(a,3,"DRAWIMAGEPART")? as i32, gi(a,4,"DRAWIMAGEPART")? as i32,
                    gi(a,5,"DRAWIMAGEPART")? as i32, gi(a,6,"DRAWIMAGEPART")? as i32)?; Value::Nil
            }
            "drawimagepartex" => {
                let idx = gi(a,0,"DRAWIMAGEPARTEX")?;
                if a.len() < 9 || a.len() > 10 { return Err("DRAWIMAGEPARTEX: erwartet (img, sx, sy, sw, sh, x, y, w, h [, farbe])".into()); }
                // Die Farbe toent das Bild; ihr Alpha-Byte ist die Deckkraft
                // (RGBA(255, 255, 255, 128) = halb durchsichtig, Farben wie ueberall).
                let tint = if a.len() == 10 { Some(gi(a,9,"DRAWIMAGEPARTEX")?) } else { None };
                g!().draw_image_part_ex(idx, gi(a,1,"DRAWIMAGEPARTEX")? as i32, gi(a,2,"DRAWIMAGEPARTEX")? as i32,
                    gi(a,3,"DRAWIMAGEPARTEX")? as i32, gi(a,4,"DRAWIMAGEPARTEX")? as i32,
                    gi(a,5,"DRAWIMAGEPARTEX")? as i32, gi(a,6,"DRAWIMAGEPARTEX")? as i32,
                    gi(a,7,"DRAWIMAGEPARTEX")? as i32, gi(a,8,"DRAWIMAGEPARTEX")? as i32, tint)?; Value::Nil
            }
            "drawimageflipped" => {
                let idx = gi(a,0,"DRAWIMAGEFLIPPED")?;
                let fh = gb(a, 3); let fv = gb(a, 4);
                g!().draw_image_flipped(idx, gi(a,1,"DRAWIMAGEFLIPPED")? as i32, gi(a,2,"DRAWIMAGEFLIPPED")? as i32, fh, fv)?; Value::Nil
            }
            "drawtilemap" => {
                if a.len() != 6 { return Err(format!("DRAWTILEMAP: erwartet 6 Argument(e), erhalten {}", a.len())); }
                let idx = gi(a, 0, "DRAWTILEMAP")?;
                let (vals, rows, cols) = match &a[1] {
                    Value::Array(arr) => {
                        let arr = arr.borrow();
                        if arr.element_type != "integer" {
                            return Err("DRAWTILEMAP: map muss ARRAY OF INTEGER sein".into());
                        }
                        if arr.dims.len() != 2 {
                            return Err("DRAWTILEMAP: map muss 2D sein (zeilen x spalten)".into());
                        }
                        let mut v = Vec::with_capacity(arr.cells.len());
                        if let Some(ints) = arr.cells.as_ints() {
                            v.extend_from_slice(ints);
                        } else {
                            for x in arr.cells.iter() {
                                match x {
                                    Value::Int(n) => v.push(n),
                                    _ => return Err("DRAWTILEMAP: map muss ARRAY OF INTEGER sein".into()),
                                }
                            }
                        }
                        (v, arr.dims[0] as i32, arr.dims[1] as i32)
                    }
                    _ => return Err("DRAWTILEMAP: map muss ARRAY OF INTEGER sein".into()),
                };
                let tw = gi(a, 2, "DRAWTILEMAP")? as i32;
                let th = gi(a, 3, "DRAWTILEMAP")? as i32;
                let sx = gi(a, 4, "DRAWTILEMAP")? as i32;
                let sy = gi(a, 5, "DRAWTILEMAP")? as i32;
                g!().draw_tilemap(idx, &vals, rows, cols, tw, th, sx, sy)?;
                Value::Nil
            }
            "load_assets" => Value::Int(g!().load_assets(gs(a,0,"LOAD_ASSETS")?)?),

            // --- Z-Layer ---
            "layer_define" => { g!().layer_define(gs(a,0,"LAYER_DEFINE")?, gi(a,1,"LAYER_DEFINE")? as i32); Value::Nil }
            "layer" => { let n = gs(a,0,"LAYER")?.to_string(); g!().layer(&n); Value::Nil }
            "layer_end" => { g!().layer_end(); Value::Nil }
            "layer_clear" => { let n = gs(a,0,"LAYER_CLEAR")?.to_string(); g!().layer_clear(&n); Value::Nil }

            // --- Sprite-Atlas ---
            "atlas_load" => Value::Int(g!().atlas_load(gs(a,0,"ATLAS_LOAD")?)?),
            "atlas_draw" | "batch_draw" => {
                let atlas = gi(a,0,"ATLAS_DRAW")?;
                let name = gs(a,1,"ATLAS_DRAW")?.to_string();
                let tint = if a.len() > 4 { Some(gi(a,4,"ATLAS_DRAW")?) } else { None };
                g!().atlas_draw(atlas, &name, gi(a,2,"ATLAS_DRAW")? as i32, gi(a,3,"ATLAS_DRAW")? as i32, false, false, tint)?; Value::Nil
            }
            "atlas_draw_flipped" => {
                // ATLAS_DRAW_FLIPPED(atlas, name, x, y[, flip_x[, flip_y[, tint]]])
                // flip_x/flip_y akzeptieren TRUE/FALSE und 1/0; tint optional (7. Arg).
                let atlas = gi(a,0,"ATLAS_DRAW_FLIPPED")?;
                let name = gs(a,1,"ATLAS_DRAW_FLIPPED")?.to_string();
                let fx = gflag(a, 4);
                let fy = gflag(a, 5);
                let tint = if a.len() > 6 { Some(gi(a,6,"ATLAS_DRAW_FLIPPED")?) } else { None };
                g!().atlas_draw(atlas, &name, gi(a,2,"ATLAS_DRAW_FLIPPED")? as i32, gi(a,3,"ATLAS_DRAW_FLIPPED")? as i32, fx, fy, tint)?; Value::Nil
            }
            "batch_flush" => Value::Nil, // Recording-Modell: alles flusht beim FLIP

            // --- Modul: input ---
            "input_bind" => {
                if a.len() < 2 { return Err("INPUT_BIND: erwartet Action + mind. 1 Key".into()); }
                let action = gs(a, 0, "INPUT_BIND")?.to_lowercase();
                let mut keys = Vec::with_capacity(a.len() - 1);
                for i in 1..a.len() { keys.push(gi(a, i, "INPUT_BIND")?); }
                self.input_state.actions.insert(action, keys);
                Value::Nil
            }
            "input_unbind" => { let action = gs(a, 0, "INPUT_UNBIND")?.to_lowercase(); self.input_state.actions.remove(&action); Value::Nil }
            "input_reset" => { self.input_state.actions.clear(); self.input_state.prev.clear(); self.input_state.cur.clear(); Value::Nil }
            "input_update" => {
                // Ohne Fenster (kein SCREEN) sind keine Tasten gedrueckt --
                // wie in der Python-VM (pygame ohne Display). Konsolen-Demos
                // laufen so bit-identisch.
                let keys: Vec<i64> = self.input_state.actions.values().flatten().copied().collect();
                let mut cur = HashSet::new();
                if let Some(g) = self.gfx.as_ref() {
                    for k in keys { if g.key_down(k) { cur.insert(k); } }
                }
                self.input_state.prev = std::mem::replace(&mut self.input_state.cur, cur);
                Value::Nil
            }
            "input_held" | "input_pressed" | "input_released" => {
                let action = gs(a, 0, "INPUT")?.to_lowercase();
                let keys = self.input_state.actions.get(&action)
                    .ok_or_else(|| format!("{}: Action '{}' nicht gebunden", name.to_uppercase(), action))?;
                let (cur, prev) = (&self.input_state.cur, &self.input_state.prev);
                let r = match name {
                    "input_held" => keys.iter().any(|k| cur.contains(k)),
                    "input_pressed" => keys.iter().any(|k| cur.contains(k) && !prev.contains(k)),
                    _ => keys.iter().any(|k| prev.contains(k) && !cur.contains(k)),
                };
                Value::Bool(r)
            }
            "input_axis" => {
                let neg = gs(a, 0, "INPUT_AXIS")?.to_lowercase();
                let pos = gs(a, 1, "INPUT_AXIS")?.to_lowercase();
                let held = |act: &str| -> R<bool> {
                    let keys = self.input_state.actions.get(act)
                        .ok_or_else(|| format!("Action '{}' nicht gebunden", act))?;
                    Ok(keys.iter().any(|k| self.input_state.cur.contains(k)))
                };
                let (n, p) = (held(&neg)?, held(&pos)?);
                Value::Int(if n && !p { -1 } else if p && !n { 1 } else { 0 })
            }
            "input_bound" => {
                let action = gs(a, 0, "INPUT_BOUND")?.to_lowercase();
                Value::Bool(self.input_state.actions.get(&action).map(|k| !k.is_empty()).unwrap_or(false))
            }
            "input_joy_count" => Value::Int(self.gfx.as_ref().map(|g| g.joy_count()).unwrap_or(0)),
            "input_joy_name" => {
                let idx = gi(a, 0, "INPUT_JOY_NAME")?;
                Value::Str(self.gfx.as_ref().map(|g| g.joy_name(idx)).unwrap_or_default().into())
            }
            "input_joy_axis" => {
                let pad = gi(a, 0, "INPUT_JOY_AXIS")?;
                let name = gs(a, 1, "INPUT_JOY_AXIS")?.to_lowercase();
                let axis_idx = match name.as_str() {
                    "left_x" => 0, "left_y" => 1, "right_x" => 2, "right_y" => 3,
                    "lt" => 4, "rt" => 5,
                    _ => return Err(format!(
                        "INPUT_JOY_AXIS: unbekannte Achse '{}' (erlaubt: left_x, left_y, right_x, right_y, lt, rt)", name)),
                };
                let mut v = self.gfx.as_ref().map(|g| g.joy_axis(pad, axis_idx)).unwrap_or(0.0);
                // Deadzone fuer Sticks (Trigger unangetastet) -- wie Python.
                if (name.starts_with("left_") || name.starts_with("right_")) && v.abs() < 0.15 {
                    v = 0.0;
                }
                Value::Float(v)
            }

            // --- Modul: camera ---
            "camera_set" => {
                let x = need_f(a, 0, "CAMERA_SET")?; let y = need_f(a, 1, "CAMERA_SET")?;
                let zoom = if a.len() >= 3 { let z = need_f(a, 2, "CAMERA_SET")?; if z <= 0.0 { return Err("CAMERA_SET: zoom muss > 0 sein".into()); } z } else { 1.0 };
                g!().set_camera(x, y, zoom);
                if a.len() >= 4 { g!().set_camera_rotation(need_f(a, 3, "CAMERA_SET")?); }
                Value::Nil
            }
            "camera_reset" => { g!().reset_camera(); Value::Nil }
            "camera_set_rotation" => { g!().set_camera_rotation(need_f(a, 0, "CAMERA_SET_ROTATION")?); Value::Nil }
            "camera_rotation" => Value::Float(g!().camera_rotation()),
            "camera_shake" => {
                // CAMERA_SHAKE(staerke[, dauer_ms]) -- staerke=0 stoppt sofort
                let amp = need_f(a, 0, "CAMERA_SHAKE")?;
                if amp < 0.0 { return Err("CAMERA_SHAKE: staerke muss >= 0 sein".into()); }
                let dur = if a.len() >= 2 { gi(a, 1, "CAMERA_SHAKE")? } else { 300 };
                if dur <= 0 { return Err("CAMERA_SHAKE: dauer_ms muss > 0 sein".into()); }
                g!().camera_shake(amp, dur as f64); Value::Nil
            }
            "camera_x" => Value::Float(g!().camera().0),
            "camera_y" => Value::Float(g!().camera().1),
            "camera_zoom" => Value::Float(g!().camera().2),
            "camera_follow" => {
                let (tx, ty, sw, sh) = (need_f(a,0,"CAMERA_FOLLOW")?, need_f(a,1,"CAMERA_FOLLOW")?, need_f(a,2,"CAMERA_FOLLOW")?, need_f(a,3,"CAMERA_FOLLOW")?);
                let z = g!().camera().2;
                g!().set_camera(tx - (sw / z) / 2.0, ty - (sh / z) / 2.0, z); Value::Nil
            }
            // Zweites Argument (die jeweils andere Screen-Achse) ist optional --
            // ohne Rotation identisch zur alten Ein-Argument-Form; MIT aktiver
            // CAMERA_SET_ROTATION braucht die korrekte Umkehrung beide Werte
            // (Rotation mischt x/y), siehe s2w_x_rot/s2w_y_rot.
            "camera_s2w_x" => {
                let sx = need_f(a, 0, "CAMERA_S2W_X")?;
                Value::Float(if a.len() >= 2 { g!().s2w_x_rot(sx, need_f(a, 1, "CAMERA_S2W_X")?) } else { g!().s2w_x(sx) })
            }
            "camera_s2w_y" => {
                let sy = need_f(a, 0, "CAMERA_S2W_Y")?;
                Value::Float(if a.len() >= 2 { g!().s2w_y_rot(need_f(a, 1, "CAMERA_S2W_Y")?, sy) } else { g!().s2w_y(sy) })
            }

            // --- Modul: sprite (nur SPRITE_DRAW braucht Grafik) ---
            "sprite_draw" => {
                let sp = match a.first() {
                    Some(Value::Sprite(s)) => s.clone(),
                    _ => return Err("SPRITE_DRAW erwartet SPRITE".into()),
                };
                let s = sp.borrow();
                let g = self.gfx.as_mut().ok_or("SPRITE_DRAW vor SCREEN aufgerufen")?;
                g.draw_sprite(s.tex_idx, s.current_frame, s.frame_w, s.frame_h,
                    s.x as i32, s.y as i32, s.flip_x, s.flip_y, s.scale_x, s.scale_y, s.tint)?;
                Value::Nil
            }
            // --- Modul: imgfx (immutable, liefern neues IMAGE-Handle) ---
            "image_scale" => {
                let (b, h) = (gi(a,1,"IMAGE_SCALE")? as i32, gi(a,2,"IMAGE_SCALE")? as i32);
                if a.len() > 4 { return Err("IMAGE_SCALE: erwartet (bild, breite, hoehe [, filter$])".into()); }
                if a.len() == 4 {
                    let name = gs(a, 3, "IMAGE_SCALE")?;
                    let f = crate::resample::Filter::aus_namen(&name).ok_or_else(|| format!(
                        "IMAGE_SCALE: unbekannter Filter '{}' -- moeglich sind lanczos, bikubisch, box, hamming, linear", name))?;
                    Value::Int(g!().image_scale_filter(gi(a,0,"IMAGE_SCALE")?, b, h, f)?)
                } else {
                    Value::Int(g!().image_scale(gi(a,0,"IMAGE_SCALE")?, b, h)?)
                }
            }
            "image_enhance" => {
                if a.len() != 3 { return Err("IMAGE_ENHANCE: erwartet (bild, art$, faktor)".into()); }
                let name = gs(a, 1, "IMAGE_ENHANCE")?;
                let art = crate::resample::Anpassung::aus_namen(&name).ok_or_else(|| format!(
                    "IMAGE_ENHANCE: unbekannte Art '{}' -- moeglich sind helligkeit, kontrast, saettigung", name))?;
                let f = need_f(a, 2, "IMAGE_ENHANCE")?;
                if !f.is_finite() || f < 0.0 { return Err("IMAGE_ENHANCE: der Faktor muss eine Zahl ab 0 sein".into()); }
                Value::Int(g!().image_enhance(gi(a,0,"IMAGE_ENHANCE")?, art, f)?)
            }
            "image_scale_nn" => Value::Int(g!().image_scale_nn(gi(a,0,"IMAGE_SCALE_NN")?, gi(a,1,"IMAGE_SCALE_NN")? as i32, gi(a,2,"IMAGE_SCALE_NN")? as i32)?),
            "image_rotate" => Value::Int(g!().image_rotate(gi(a,0,"IMAGE_ROTATE")?, need_f(a,1,"IMAGE_ROTATE")? as f32)?),
            "image_rotate_cw" => Value::Int(g!().image_rotate_cw(gi(a,0,"IMAGE_ROTATE_CW")?)?),
            "image_rotate_ccw" => Value::Int(g!().image_rotate_ccw(gi(a,0,"IMAGE_ROTATE_CCW")?)?),
            "image_flip" => Value::Int(g!().image_flip(gi(a,0,"IMAGE_FLIP")?, gb(a,1), gb(a,2))?),
            "image_tint" => { let c = gi(a,1,"IMAGE_TINT")?; if c < 0 || c > 0xFFFFFF { return Err("IMAGE_TINT: Farbe muss 0..0xFFFFFF sein".into()); } Value::Int(g!().image_tint(gi(a,0,"IMAGE_TINT")?, c)?) }
            "image_copy" => Value::Int(g!().image_copy(gi(a,0,"IMAGE_COPY")?)?),
            "image_new" => Value::Int(g!().image_new(
                gi(a,0,"IMAGE_NEW")? as i32, gi(a,1,"IMAGE_NEW")? as i32,
                if a.len() >= 3 { Some(gi(a,2,"IMAGE_NEW")?) } else { None })?),
            "image_clear" => {
                let r = if a.len() >= 5 {
                    Some((gi(a,1,"IMAGE_CLEAR")? as i32, gi(a,2,"IMAGE_CLEAR")? as i32,
                          gi(a,3,"IMAGE_CLEAR")? as i32, gi(a,4,"IMAGE_CLEAR")? as i32))
                } else if a.len() == 1 { None } else {
                    return Err("IMAGE_CLEAR: entweder nur das Bild oder Bild, x, y, b, h".into());
                };
                g!().image_clear(gi(a,0,"IMAGE_CLEAR")?, r)?;
                Value::Nil
            }
            "image_draw_image" => {
                let quelle = if a.len() >= 8 {
                    Some((gi(a,4,"IMAGE_DRAW_IMAGE")? as i32, gi(a,5,"IMAGE_DRAW_IMAGE")? as i32,
                          gi(a,6,"IMAGE_DRAW_IMAGE")? as i32, gi(a,7,"IMAGE_DRAW_IMAGE")? as i32))
                } else if a.len() == 4 || a.len() == 5 { None } else {
                    return Err("IMAGE_DRAW_IMAGE: erwartet (ziel, quelle, x, y), \
optional dahinter (qx, qy, qb, qh) und eine Faerbung".into());
                };
                // Die Faerbung steht hinter der Quelle, bei kurzer Form direkt
                // hinter y. WEISS ist "unveraendert".
                let tint = match a.len() { 5 => gi(a,4,"IMAGE_DRAW_IMAGE")?,
                                           9 => gi(a,8,"IMAGE_DRAW_IMAGE")?,
                                           _ => 0xFFFFFF };
                g!().image_draw_image(gi(a,0,"IMAGE_DRAW_IMAGE")?, gi(a,1,"IMAGE_DRAW_IMAGE")?,
                                      gi(a,2,"IMAGE_DRAW_IMAGE")? as i32,
                                      gi(a,3,"IMAGE_DRAW_IMAGE")? as i32, quelle, tint)?;
                Value::Nil
            }
            "getalpha" => Value::Int(g!().get_alpha(gi(a,0,"GETALPHA")?,
                gi(a,1,"GETALPHA")? as i32, gi(a,2,"GETALPHA")? as i32)),
            "image_save" => {
                let ohne_alpha = match a.get(2) {
                    None => false,
                    Some(Value::Bool(b)) => !*b,
                    Some(_) => return Err("IMAGE_SAVE: drittes Argument ist ein Wahrheitswert (mit Deckkraft-Kanal?)".into()),
                };
                g!().image_save(gi(a,0,"IMAGE_SAVE")?, gs(a,1,"IMAGE_SAVE")?, ohne_alpha)?; Value::Nil
            }
            "image_free" => { g!().image_free(gi(a,0,"IMAGE_FREE")?)?; Value::Nil }
            "image_from_buffer" => {
                if a.len() != 1 {
                    return Err(format!("IMAGE_FROM_BUFFER: erwartet (puffer), erhalten {} Argumente", a.len()));
                }
                let bytes = crate::builtins::buf_h(&a[0], "IMAGE_FROM_BUFFER")?.borrow().clone();
                Value::Int(g!().image_from_buffer(&bytes)?)
            }
            "image_to_buffer" => {
                if a.is_empty() || a.len() > 2 {
                    return Err(format!("IMAGE_TO_BUFFER: erwartet (bild[, mit_alpha]), erhalten {} Argumente", a.len()));
                }
                let ohne_alpha = match a.get(1) {
                    None => false,
                    Some(Value::Bool(b)) => !*b,
                    Some(_) => return Err("IMAGE_TO_BUFFER: zweites Argument ist ein Wahrheitswert (mit Deckkraft-Kanal?)".into()),
                };
                crate::builtins::neuer_buffer(g!().image_to_buffer(gi(a,0,"IMAGE_TO_BUFFER")?, ohne_alpha)?)
            }
            "image_save_gif" => {
                // Die Bildnummern kommen als ARRAY OF IMAGE (so haelt ein
                // Programm seine Einzelbilder ohnehin) oder als TUPLE.
                //
                // `anzahl` wird VOR dem Umwandeln gebraucht: ein fest
                // dimensioniertes Feld hat hinter den benutzten Plaetzen NIL
                // stehen, und die sind keine Bilder. Ohne diese Reihenfolge
                // scheitert schon `DIM b[16] AS IMAGE` mit drei belegten
                // Plaetzen -- der Normalfall.
                let roh: Vec<Value> = match &a[0] {
                    Value::Array(arr) => arr.borrow().cells.iter().collect(),
                    Value::Tuple(t) => t.iter().cloned().collect(),
                    _ => return Err("IMAGE_SAVE_GIF: erwartet ARRAY OF IMAGE".into()),
                };
                let anzahl = if a.len() >= 5 { gi(a,4,"IMAGE_SAVE_GIF")? } else { roh.len() as i64 };
                if anzahl < 1 || anzahl as usize > roh.len() {
                    return Err(std::format!(
                        "IMAGE_SAVE_GIF: {} Bilder verlangt, {} vorhanden", anzahl, roh.len()));
                }
                let mut bilder = Vec::with_capacity(anzahl as usize);
                for (i, x) in roh.iter().take(anzahl as usize).enumerate() {
                    match x {
                        Value::Int(n) => bilder.push(*n),
                        Value::Nil => return Err(std::format!(
                            "IMAGE_SAVE_GIF: Platz {} ist leer -- mit `anzahl` sagen, \
wie viele Plaetze gelten", i + 1)),
                        _ => return Err("IMAGE_SAVE_GIF: ARRAY OF IMAGE noetig".into()),
                    }
                }
                // GIF rechnet in Hundertstelsekunden. Unter 2 legen die
                // meisten Betrachter still ihre eigene Dauer fest (meist 10) --
                // dann liefe die Ausgabe LANGSAMER als verlangt, ohne Hinweis.
                let hundertstel = |ms: f64| ((ms / 10.0).round() as i64).clamp(2, 65535) as u16;
                // Der dritte Parameter ist ZWEIERLEI: eine Zahl sind Bilder je
                // Sekunde fuer alle, ein FELD ist die Dauer JE BILD in
                // Millisekunden. GIF kann das je Bild, und eine Bildfolge
                // braucht es -- eine Pose wird gehalten, eine Bewegung laeuft
                // schnell durch.
                //
                // Warum die Einheit wechselt: ein einzelnes Bild hat keine
                // Bildrate, es hat eine Dauer. `[4, 12]` als "250 ms, 83 ms"
                // zu lesen waere die schlechtere Zumutung als zwei Einheiten,
                // die je Form eindeutig sind.
                let zeiten: Vec<u16> = match a.get(2) {
                    None => vec![hundertstel(100.0); bilder.len()],
                    Some(Value::Array(_)) | Some(Value::Tuple(_)) => {
                        let roh: Vec<Value> = match &a[2] {
                            Value::Array(arr) => arr.borrow().cells.iter().collect(),
                            Value::Tuple(t) => t.iter().cloned().collect(),
                            _ => unreachable!(),
                        };
                        if roh.len() < bilder.len() {
                            return Err(std::format!(
                                "IMAGE_SAVE_GIF: {} Bilder, aber nur {} Zeiten -- je Bild eine Dauer in Millisekunden", bilder.len(), roh.len()));
                        }
                        let mut v = Vec::with_capacity(bilder.len());
                        for (i, x) in roh.iter().take(bilder.len()).enumerate() {
                            let ms = match x {
                                Value::Int(n) => *n as f64,
                                Value::Float(f) => *f,
                                _ => return Err(std::format!(
                                    "IMAGE_SAVE_GIF: Zeit {} ist keine Zahl", i + 1)),
                            };
                            if ms <= 0.0 {
                                return Err(std::format!(
                                    "IMAGE_SAVE_GIF: Zeit {} ist {} ms -- muss > 0 sein",
                                    i + 1, ms));
                            }
                            v.push(hundertstel(ms));
                        }
                        v
                    }
                    Some(_) => {
                        let fps = need_f(a,2,"IMAGE_SAVE_GIF")?;
                        if fps <= 0.0 {
                            return Err("IMAGE_SAVE_GIF: Bilder je Sekunde muessen > 0 sein".into());
                        }
                        vec![hundertstel(1000.0 / fps); bilder.len()]
                    }
                };
                // `gflag` fehlend = false -- hier ist die Vorgabe aber TRUE
                // (eine Bewegung, die einmal laeuft und dann steht, ist
                // selten gemeint), also selbst pruefen.
                let wdh = if a.len() >= 4 { gflag(a, 3) } else { true };
                g!().image_save_gif(&bilder, gs(a,1,"IMAGE_SAVE_GIF")?, &zeiten, wdh)?;
                Value::Nil
            }
            "image_crop" => Value::Int(g!().image_crop(gi(a,0,"IMAGE_CROP")?, gi(a,1,"IMAGE_CROP")? as i32, gi(a,2,"IMAGE_CROP")? as i32, gi(a,3,"IMAGE_CROP")? as i32, gi(a,4,"IMAGE_CROP")? as i32)?),
            "image_resize_canvas" => {
                let fill = if a.len() >= 6 { gi(a,5,"IMAGE_RESIZE_CANVAS")? } else { 0 };
                Value::Int(g!().image_resize_canvas(gi(a,0,"IMAGE_RESIZE_CANVAS")?, gi(a,1,"IMAGE_RESIZE_CANVAS")? as i32, gi(a,2,"IMAGE_RESIZE_CANVAS")? as i32, gi(a,3,"IMAGE_RESIZE_CANVAS")? as i32, gi(a,4,"IMAGE_RESIZE_CANVAS")? as i32, fill)?)
            }
            "image_convolve" => {
                let k = gfloats(a, 1, "IMAGE_CONVOLVE")?;
                Value::Int(g!().image_convolve(gi(a,0,"IMAGE_CONVOLVE")?, &k)?)
            }
            "image_alpha_mask" => Value::Int(g!().image_alpha_mask(
                gi(a,0,"IMAGE_ALPHA_MASK")?, gi(a,1,"IMAGE_ALPHA_MASK")?)?),
            "image_alpha_crop" => Value::Int(g!().image_alpha_crop(
                gi(a,0,"IMAGE_ALPHA_CROP")?, need_f(a,1,"IMAGE_ALPHA_CROP")?)?),
            "image_alpha_premultiply" => Value::Int(g!().image_alpha_premultiply(
                gi(a,0,"IMAGE_ALPHA_PREMULTIPLY")?)?),
            "image_dither" => Value::Int(g!().image_dither(
                gi(a,0,"IMAGE_DITHER")?, gi(a,1,"IMAGE_DITHER")?, gi(a,2,"IMAGE_DITHER")?,
                gi(a,3,"IMAGE_DITHER")?, gi(a,4,"IMAGE_DITHER")?)?),
            "image_palette" => {
                let cols = g!().image_palette(gi(a,0,"IMAGE_PALETTE")?, gi(a,1,"IMAGE_PALETTE")?)?;
                crate::builtins::new_int_array(cols)
            }
            "image_blur" => Value::Int(g!().image_blur(gi(a,0,"IMAGE_BLUR")?, gi(a,1,"IMAGE_BLUR")? as i32)?),
            "image_brightness" => Value::Int(g!().image_brightness(gi(a,0,"IMAGE_BRIGHTNESS")?, gi(a,1,"IMAGE_BRIGHTNESS")? as i32)?),
            "image_contrast" => Value::Int(g!().image_contrast(gi(a,0,"IMAGE_CONTRAST")?, need_f(a,1,"IMAGE_CONTRAST")? as f32)?),
            "image_grayscale" => Value::Int(g!().image_grayscale(gi(a,0,"IMAGE_GRAYSCALE")?)?),
            "image_invert" => Value::Int(g!().image_invert(gi(a,0,"IMAGE_INVERT")?)?),
            "image_color_to_alpha" => {
                const F: &str = "IMAGE_COLOR_TO_ALPHA";
                if a.len() != 2 && a.len() != 3 { return Err(format!("{}: erwartet (bild, farbe [, schwelle])", F)); }
                let s = if a.len() == 3 { need_f(a, 2, F)? as f32 } else { 0.0 };
                if !(0.0..1.0).contains(&s) { return Err(format!("{}: die Schwelle liegt zwischen 0 und 1 (nicht {})", F, s)); }
                Value::Int(g!().image_color_to_alpha(gi(a, 0, F)?, gi(a, 1, F)?, s)?)
            }
            "image_replace_color" => Value::Int(g!().image_replace_color(gi(a,0,"IMAGE_REPLACE_COLOR")?, gi(a,1,"IMAGE_REPLACE_COLOR")?, gi(a,2,"IMAGE_REPLACE_COLOR")?)?),
            "image_draw_line" => { g!().image_draw_line(gi(a,0,"IMAGE_DRAW_LINE")?, gi(a,1,"IMAGE_DRAW_LINE")? as i32, gi(a,2,"IMAGE_DRAW_LINE")? as i32, gi(a,3,"IMAGE_DRAW_LINE")? as i32, gi(a,4,"IMAGE_DRAW_LINE")? as i32, gi(a,5,"IMAGE_DRAW_LINE")?)?; Value::Nil }
            "image_draw_circle" => { g!().image_draw_circle(gi(a,0,"IMAGE_DRAW_CIRCLE")?, gi(a,1,"IMAGE_DRAW_CIRCLE")? as i32, gi(a,2,"IMAGE_DRAW_CIRCLE")? as i32, gi(a,3,"IMAGE_DRAW_CIRCLE")? as i32, gi(a,4,"IMAGE_DRAW_CIRCLE")?)?; Value::Nil }
            "image_draw_rect" => { g!().image_draw_rect(gi(a,0,"IMAGE_DRAW_RECT")?, gi(a,1,"IMAGE_DRAW_RECT")? as i32, gi(a,2,"IMAGE_DRAW_RECT")? as i32, gi(a,3,"IMAGE_DRAW_RECT")? as i32, gi(a,4,"IMAGE_DRAW_RECT")? as i32, gi(a,5,"IMAGE_DRAW_RECT")?)?; Value::Nil }
            // --- Formen mit Kommazahlen und Deckkraft (leinwand.rs) ---
            "image_fill_rect" => {
                const F: &str = "IMAGE_FILL_RECT";
                if a.len() != 6 && a.len() != 7 { return Err(format!("{}: erwartet (bild, x0, y0, x1, y1, farbe [, deckkraft])", F)); }
                let (x0, y0, x1, y1) = (need_f(a,1,F)? as f32, need_f(a,2,F)? as f32, need_f(a,3,F)? as f32, need_f(a,4,F)? as f32);
                let c = lw_farbe(a, 5, Some(6), F)?;
                g!().image_leinwand(gi(a,0,F)?, F, |l| l.rechteck(x0, y0, x1, y1, c))?; Value::Nil
            }
            "image_frame" => {
                const F: &str = "IMAGE_FRAME";
                if a.len() != 7 && a.len() != 8 { return Err(format!("{}: erwartet (bild, x0, y0, x1, y1, breite, farbe [, deckkraft])", F)); }
                let (x0, y0, x1, y1) = (need_f(a,1,F)? as f32, need_f(a,2,F)? as f32, need_f(a,3,F)? as f32, need_f(a,4,F)? as f32);
                let b = need_f(a,5,F)? as f32;
                let c = lw_farbe(a, 6, Some(7), F)?;
                g!().image_leinwand(gi(a,0,F)?, F, |l| l.rahmen(x0, y0, x1, y1, b, c))?; Value::Nil
            }
            "image_fill_roundrect" => {
                const F: &str = "IMAGE_FILL_ROUNDRECT";
                if a.len() != 7 && a.len() != 8 { return Err(format!("{}: erwartet (bild, x0, y0, x1, y1, radius, farbe [, deckkraft])", F)); }
                let (x0, y0, x1, y1) = (need_f(a,1,F)? as f32, need_f(a,2,F)? as f32, need_f(a,3,F)? as f32, need_f(a,4,F)? as f32);
                let r = need_f(a,5,F)? as f32;
                let c = lw_farbe(a, 6, Some(7), F)?;
                g!().image_leinwand(gi(a,0,F)?, F, |l| l.rund_rechteck(x0, y0, x1, y1, r, c))?; Value::Nil
            }
            "image_fill_ellipse" => {
                const F: &str = "IMAGE_FILL_ELLIPSE";
                if a.len() != 6 && a.len() != 7 { return Err(format!("{}: erwartet (bild, cx, cy, rx, ry, farbe [, deckkraft])", F)); }
                let (cx, cy, rx, ry) = (need_f(a,1,F)? as f32, need_f(a,2,F)? as f32, need_f(a,3,F)? as f32, need_f(a,4,F)? as f32);
                let c = lw_farbe(a, 5, Some(6), F)?;
                g!().image_leinwand(gi(a,0,F)?, F, |l| l.ellipse(cx, cy, rx, ry, c))?; Value::Nil
            }
            "image_ring" => {
                const F: &str = "IMAGE_RING";
                if a.len() != 6 && a.len() != 7 { return Err(format!("{}: erwartet (bild, cx, cy, r, breite, farbe [, deckkraft])", F)); }
                let (cx, cy, r, b) = (need_f(a,1,F)? as f32, need_f(a,2,F)? as f32, need_f(a,3,F)? as f32, need_f(a,4,F)? as f32);
                let c = lw_farbe(a, 5, Some(6), F)?;
                g!().image_leinwand(gi(a,0,F)?, F, |l| l.ring(cx, cy, r, b, c))?; Value::Nil
            }
            "image_fill_poly" | "image_polyline" => {
                let linie = name == "image_polyline";
                let f: &str = if linie { "IMAGE_POLYLINE" } else { "IMAGE_FILL_POLY" };
                let (soll, extra) = if linie { (5, "breite, ") } else { (4, "") };
                if a.len() != soll && a.len() != soll + 1 {
                    return Err(format!("{}: erwartet (bild, xs, ys, {}farbe [, deckkraft])", f, extra));
                }
                let xs = gfloats(a, 1, f)?;
                let ys = gfloats(a, 2, f)?;
                if xs.len() != ys.len() {
                    return Err(format!("{}: xs und ys muessen gleich lang sein ({} gegen {})", f, xs.len(), ys.len()));
                }
                let pkt: Vec<(f32, f32)> = xs.iter().zip(ys.iter()).map(|(x, y)| (*x as f32, *y as f32)).collect();
                if linie {
                    let b = need_f(a,3,f)? as f32;
                    let c = lw_farbe(a, 4, Some(5), f)?;
                    g!().image_leinwand(gi(a,0,f)?, f, |l| l.linie(&pkt, b, c))?;
                } else {
                    let c = lw_farbe(a, 3, Some(4), f)?;
                    g!().image_leinwand(gi(a,0,f)?, f, |l| l.vieleck(&pkt, c))?;
                }
                Value::Nil
            }
            "image_gradient" => {
                const F: &str = "IMAGE_GRADIENT";
                if a.len() != 7 && a.len() != 8 && a.len() != 10 {
                    return Err(format!("{}: erwartet (bild, x0, y0, x1, y1, farbe1, farbe2 [, senkrecht [, deckkraft1, deckkraft2]])", F));
                }
                let (x0, y0, x1, y1) = (need_f(a,1,F)? as f32, need_f(a,2,F)? as f32, need_f(a,3,F)? as f32, need_f(a,4,F)? as f32);
                let (d1, d2) = if a.len() == 10 { (Some(8), Some(9)) } else { (None, None) };
                let c1 = lw_farbe(a, 5, d1, F)?;
                let c2 = lw_farbe(a, 6, d2, F)?;
                let senkrecht = if a.len() >= 8 { gb(a, 7) } else { true };
                g!().image_leinwand(gi(a,0,F)?, F, |l| l.verlauf(x0, y0, x1, y1, c1, c2, senkrecht))?; Value::Nil
            }
            "image_draw_text" => {
                if a.len() != 6 && a.len() != 7 { return Err("IMAGE_DRAW_TEXT: erwartet (bild, x, y, text$, groesse, farbe[, font])".into()); }
                let txt = gs(a,3,"IMAGE_DRAW_TEXT")?.to_string();
                let (b, x, y) = (gi(a,0,"IMAGE_DRAW_TEXT")?, gi(a,1,"IMAGE_DRAW_TEXT")? as i32, gi(a,2,"IMAGE_DRAW_TEXT")? as i32);
                let (sz, c) = (gi(a,4,"IMAGE_DRAW_TEXT")? as i32, gi(a,5,"IMAGE_DRAW_TEXT")?);
                // Ohne Schrift der alte Weg, damit bestehende Bilder gleich bleiben.
                if a.len() == 7 { g!().image_draw_text_font(b, x, y, &txt, sz, c, gi(a,6,"IMAGE_DRAW_TEXT")?)?; }
                else { g!().image_draw_text(b, x, y, &txt, sz, c)?; }
                Value::Nil
            }
            "image_text" => {
                if a.len() != 3 && a.len() != 4 { return Err("IMAGE_TEXT: erwartet (text$, groesse, farbe[, font])".into()); }
                let txt = gs(a,0,"IMAGE_TEXT")?.to_string();
                let font = if a.len() == 4 { gi(a,3,"IMAGE_TEXT")? } else { -1 };
                Value::Int(g!().image_text(&txt, gi(a,1,"IMAGE_TEXT")? as i32, gi(a,2,"IMAGE_TEXT")?, font)?)
            }
            "image_load_gif" => {
                let handles = g!().image_load_gif(gs(a,0,"IMAGE_LOAD_GIF")?)?;
                let n = handles.len() as i64;
                let mut arr = DhArray::new("image".to_string(), vec![n], || Value::Int(0));
                for (i, h) in handles.into_iter().enumerate() { arr.cells.set(i, Value::Int(h)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            "image_channel" => Value::Int(g!().image_channel(gi(a,0,"IMAGE_CHANNEL")?, gi(a,1,"IMAGE_CHANNEL")?)?),

            // --- Modul: ui (Immediate-Mode) ---
            "ui_label" => {
                let x = gi(a,0,"UI_LABEL")? as i32 + self.ui_state.offset_x; let y = gi(a,1,"UI_LABEL")? as i32 + self.ui_state.offset_y;
                let text = gs(a,2,"UI_LABEL")?.to_string();
                let c = if a.len() == 4 { gi(a,3,"UI_LABEL")? } else { self.ui_state.th("text_fg") };
                g!().text(x, y, text, c); Value::Nil
            }
            "ui_button" => {
                let id = gs(a,0,"UI_BUTTON")?.to_string();
                let (x,y,w,h) = (gi(a,1,"UI_BUTTON")? as i32 + self.ui_state.offset_x, gi(a,2,"UI_BUTTON")? as i32 + self.ui_state.offset_y, gi(a,3,"UI_BUTTON")? as i32, gi(a,4,"UI_BUTTON")? as i32);
                let text = gs(a,5,"UI_BUTTON")?.to_string();
                let bg_color = if a.len() >= 7 { gi(a,6,"UI_BUTTON")? } else { self.ui_state.th("button_bg") };
                let fg_color = if a.len() >= 8 { gi(a,7,"UI_BUTTON")? } else { self.ui_state.th("text_fg") };
                let (mx,my,down) = self.ui_mouse_gated()?;
                let hovered = ui_in_box(mx,my,x,y,w,h);
                if hovered && down && !self.ui_state.was_mouse_down { self.ui_state.click_origin = Some(id.clone()); }
                let clicked = hovered && !down && self.ui_state.was_mouse_down && self.ui_state.click_origin.as_deref() == Some(id.as_str());
                let bg = if hovered && down { ui_darken(bg_color, 40) } else if hovered { ui_lighten(bg_color, 0.25) } else { bg_color };
                let pl = self.ui_state.plastik();
                let g = self.gfx.as_mut().unwrap();
                ui_flaeche(g, pl, x, y, w, h, bg, fg_color, false);
                // Beschriftung mittig und im Knopf beschnitten -- wie im
                // Modul `gui`; links anzukleben und ueberzulaufen sieht bei
                // Knoepfen falsch aus.
                let sz = g.text_height();
                let tw = g.text_width(&text);
                let frei = (w - 12).max(0);
                let tx = x + 6 + ((frei - tw) / 2).max(0);
                let beschnitten = tw > frei;
                if beschnitten { g.push_clip(x + 6, y + 1, frei, (h - 2).max(0)); }
                g.text(tx, y + (h - sz) / 2, text, fg_color);
                if beschnitten { g.pop_clip(); }
                Value::Bool(clicked)
            }
            "ui_checkbox" => {
                let id = gs(a,0,"UI_CHECKBOX")?.to_string();
                let (x,y) = (gi(a,1,"UI_CHECKBOX")? as i32 + self.ui_state.offset_x, gi(a,2,"UI_CHECKBOX")? as i32 + self.ui_state.offset_y);
                let label = gs(a,3,"UI_CHECKBOX")?.to_string();
                if !self.ui_state.checkbox.contains_key(&id) {
                    let init = if a.len() == 5 { matches!(a[4], Value::Bool(true)) } else { false };
                    self.ui_state.checkbox.insert(id.clone(), init);
                }
                let bs = self.ui_state.metric("checkbox_size") as i32;
                let (mx,my,down) = self.ui_mouse_gated()?;
                let hovered = ui_in_box(mx,my,x,y,bs,bs);
                if hovered && down && !self.ui_state.was_mouse_down {
                    let v = !self.ui_state.checkbox[&id]; self.ui_state.checkbox.insert(id.clone(), v);
                }
                let val = self.ui_state.checkbox[&id];
                let (border, fill, fg) = (self.ui_state.th("field_border"), self.ui_state.th("accent"), self.ui_state.th("text_fg"));
                let feld = self.ui_state.th("field_bg");
                let pl = self.ui_state.plastik();
                let g = self.gfx.as_mut().unwrap();
                ui_flaeche(g, pl, x, y, bs, bs, feld, border, true);
                if hovered { g.rect(x-1,y-1,x+bs,y+bs,fill); }
                if val { g.box_fill(x+3,y+3,x+bs-4,y+bs-4,fill); }
                g.text(x+bs+5, y, label, fg);
                Value::Bool(val)
            }
            "ui_slider" => {
                let id = gs(a,0,"UI_SLIDER")?.to_string();
                let (x,y,w) = (gi(a,1,"UI_SLIDER")? as i32 + self.ui_state.offset_x, gi(a,2,"UI_SLIDER")? as i32 + self.ui_state.offset_y, gi(a,3,"UI_SLIDER")? as i32);
                let (mn, mx_) = (need_f(a,4,"UI_SLIDER")?, need_f(a,5,"UI_SLIDER")?);
                if mx_ <= mn { return Err("UI_SLIDER: max muss > min sein".into()); }
                if !self.ui_state.slider.contains_key(&id) {
                    let init = if a.len() == 7 { need_f(a,6,"UI_SLIDER")?.max(mn).min(mx_) } else { mn };
                    self.ui_state.slider.insert(id.clone(), init);
                }
                let h = self.ui_state.metric("slider_h") as i32;
                let (mox,moy,down) = self.ui_mouse_gated()?;
                if ui_in_box(mox,moy,x,y,w,h) && down {
                    let rel = ((mox - x) as f64 / (w - 1).max(1) as f64).max(0.0).min(1.0);
                    self.ui_state.slider.insert(id.clone(), mn + rel * (mx_ - mn));
                }
                let val = self.ui_state.slider[&id];
                let (track, border, handle) = (self.ui_state.th("slider_track"), self.ui_state.th("field_border"), self.ui_state.th("accent"));
                let handle_w = 10;
                let hx = x + ((val - mn) / (mx_ - mn) * (w - handle_w) as f64) as i32;
                let pl = self.ui_state.plastik();
                let g = self.gfx.as_mut().unwrap();
                if pl.grad > 0 {
                    ui_flaeche(g, pl, x, y + h / 2 - 3, w, 6, track, border, true);
                    ui_flaeche(g, pl, hx, y, handle_w, h, handle, border, false);
                } else {
                    g.box_fill(x, y + h/2 - 1, x+w-1, y+h/2+1, track);
                    g.rect(x,y,x+w-1,y+h-1,border);
                    g.box_fill(hx, y, hx+handle_w-1, y+h-1, handle);
                }
                Value::Float(val)
            }
            "ui_progress" => {
                let (x,y,w,h) = (gi(a,0,"UI_PROGRESS")? as i32 + self.ui_state.offset_x, gi(a,1,"UI_PROGRESS")? as i32 + self.ui_state.offset_y, gi(a,2,"UI_PROGRESS")? as i32, gi(a,3,"UI_PROGRESS")? as i32);
                let (value, maxv) = (need_f(a,4,"UI_PROGRESS")?, need_f(a,5,"UI_PROGRESS")?);
                if maxv <= 0.0 { return Err("UI_PROGRESS: max muss > 0 sein".into()); }
                let fg = if a.len() >= 7 { gi(a,6,"UI_PROGRESS")? } else { self.ui_state.th("progress_fg") };
                let bg = if a.len() >= 8 { gi(a,7,"UI_PROGRESS")? } else { self.ui_state.th("progress_bg") };
                if w >= 2 && h >= 2 {
                    let ratio = (value / maxv).max(0.0).min(1.0);
                    let fill_w = ((w - 2) as f64 * ratio) as i32;
                    let tfg = self.ui_state.th("field_border");
                    let pl = self.ui_state.plastik();
                    let g = self.gfx.as_mut().unwrap();
                    ui_flaeche(g, pl, x, y, w, h, bg, tfg, true);
                    if fill_w > 0 {
                        ui_flaeche(g, pl, x + 1, y + 1, fill_w, (h - 2).max(1), fg, fg, false);
                    }
                }
                Value::Nil
            }
            "ui_panel" => {
                let (x,y,w,h) = (gi(a,0,"UI_PANEL")? as i32 + self.ui_state.offset_x, gi(a,1,"UI_PANEL")? as i32 + self.ui_state.offset_y, gi(a,2,"UI_PANEL")? as i32, gi(a,3,"UI_PANEL")? as i32);
                let title = if a.len() >= 5 { gs(a,4,"UI_PANEL")?.to_string() } else { String::new() };
                let bg = if a.len() >= 6 { gi(a,5,"UI_PANEL")? } else { self.ui_state.th("panel_bg") };
                let (border, tbg, fg) = (self.ui_state.th("panel_border"), self.ui_state.th("panel_title_bg"), self.ui_state.th("text_fg"));
                let pl = self.ui_state.plastik();
                let g = self.gfx.as_mut().unwrap();
                ui_flaeche(g, pl, x, y, w, h, bg, border, false);
                if !title.is_empty() {
                    g.box_fill(x,y,x+w-1,y+17,tbg); g.rect(x,y,x+w-1,y+17,border); g.text(x+6,y+2,title,fg);
                }
                Value::Nil
            }
            "ui_radio" => {
                let id = gs(a,0,"UI_RADIO")?.to_string();
                let (x,y) = (gi(a,1,"UI_RADIO")? as i32 + self.ui_state.offset_x, gi(a,2,"UI_RADIO")? as i32 + self.ui_state.offset_y);
                let opts: Vec<String> = match a.get(3) {
                    Some(Value::Array(arr)) => { let arr = arr.borrow();
                        if arr.element_type != "string" && arr.cells.len() > 0 { return Err("UI_RADIO: options muss ARRAY OF STRING sein".into()); }
                        arr.cells.iter().map(|v| match v { Value::Str(s) => s.to_string(), o => o.fmt() }).collect() }
                    _ => return Err("UI_RADIO: options muss ARRAY OF STRING sein".into()),
                };
                let n = opts.len() as i64;
                if n == 0 { return Ok(Some(Value::Int(-1))); }
                if !self.ui_state.radio.contains_key(&id) {
                    let mut def = if a.len() >= 5 { gi(a,4,"UI_RADIO")? } else { 0 };
                    if def < 0 || def >= n { def = 0; }
                    self.ui_state.radio.insert(id.clone(), def);
                }
                let (mx,my,down) = self.ui_mouse_gated()?;
                let (row_h, radius) = (18i32, 5i32);
                for i in 0..n as i32 {
                    if ui_in_box(mx,my,x,y+i*row_h,200,row_h) && down && !self.ui_state.was_mouse_down {
                        self.ui_state.radio.insert(id.clone(), i as i64);
                    }
                }
                let sel = self.ui_state.radio[&id];
                let (fieldbg, accent, fg, border) = (self.ui_state.th("field_bg"), self.ui_state.th("accent"), self.ui_state.th("text_fg"), self.ui_state.th("field_border"));
                let g = self.gfx.as_mut().unwrap();
                for i in 0..n as i32 {
                    let cy = y + i*row_h + row_h/2; let cx = x + radius + 2;
                    g.circle(cx, cy, radius, fieldbg);
                    if i as i64 == sel { g.circle(cx,cy,radius,accent); g.circle(cx,cy,radius-2,fg); }
                    else { g.rect(cx-radius,cy-radius,cx+radius,cy+radius,border); }
                    g.text(x + 2*radius + 8, y + i*row_h + 2, opts[i as usize].clone(), fg);
                }
                Value::Int(sel)
            }
            "ui_textfield" => {
                let id = gs(a,0,"UI_TEXTFIELD")?.to_string();
                let x = gi(a,1,"UI_TEXTFIELD")? as i32 + self.ui_state.offset_x;
                let y = gi(a,2,"UI_TEXTFIELD")? as i32 + self.ui_state.offset_y;
                let w = gi(a,3,"UI_TEXTFIELD")? as i32;
                let h = gi(a,4,"UI_TEXTFIELD")? as i32;
                let placeholder = if a.len() >= 6 { gs(a,5,"UI_TEXTFIELD")?.to_string() } else { String::new() };
                self.ui_state.text.entry(id.clone()).or_insert_with(String::new);
                let blocked = self.ui_state.input_blocked;
                let (mx, my, down) = {
                    let g = self.gfx.as_ref().ok_or("UI_TEXTFIELD vor SCREEN")?;
                    let down = g.mouse_button(0);
                    if blocked { (-9999i32, -9999i32, down) } else { (g.mouse_x() as i32, g.mouse_y() as i32, down) }
                };
                let hovered = ui_in_box(mx, my, x, y, w, h);
                // Klick fokussiert / blurred
                if down && !self.ui_state.was_mouse_down {
                    if hovered { self.ui_state.focused = Some(id.clone()); }
                    else if self.ui_state.focused.as_deref() == Some(id.as_str()) { self.ui_state.focused = None; }
                }
                let is_focused = self.ui_state.focused.as_deref() == Some(id.as_str());
                if is_focused {
                    let typed = { let g = self.gfx.as_mut().ok_or("UI_TEXTFIELD vor SCREEN")?; g.pop_text_input() };
                    if !typed.is_empty() {
                        let clean: String = typed.chars().filter(|&c| c != '\t').collect();
                        self.ui_state.text.get_mut(&id).unwrap().push_str(&clean);
                    }
                    let bs = { let g = self.gfx.as_ref().ok_or("UI_TEXTFIELD vor SCREEN")?; g.key_down(8) };
                    if bs && !self.ui_state.prev_backspace {
                        self.ui_state.text.get_mut(&id).unwrap().pop();
                    }
                }
                // Zeichnen
                let border = if is_focused { self.ui_state.th("accent") } else { self.ui_state.th("field_border") };
                let (bg, fg, muted) = (self.ui_state.th("field_bg"), self.ui_state.th("text_fg"), self.ui_state.th("muted_fg"));
                let txt = self.ui_state.text[&id].clone();
                let caret_on = is_focused && (self.ui_state.frame_count / 16) % 2 == 0;
                {
                    let pl = self.ui_state.plastik();
                    let g = self.gfx.as_mut().ok_or("UI_TEXTFIELD vor SCREEN")?;
                    ui_flaeche(g, pl, x, y, w, h, bg, border, true);
                    if !txt.is_empty() {
                        g.text(x + 5, y + (h - 14) / 2, txt.clone(), fg);
                    } else if !placeholder.is_empty() && !is_focused {
                        g.text(x + 5, y + (h - 14) / 2, placeholder, muted);
                    }
                    if caret_on {
                        let mut cx = x + 5 + (txt.chars().count() as i32) * 8;
                        if cx > x + w - 3 { cx = x + w - 3; }
                        g.line(cx, y + 3, cx, y + h - 4, fg);
                    }
                }
                Value::str_rc(&txt)
            }
            "ui_textfield_set" => {
                let id = gs(a,0,"UI_TEXTFIELD_SET")?.to_string();
                let v = gs(a,1,"UI_TEXTFIELD_SET")?.to_string();
                self.ui_state.text.insert(id, v);
                Value::Nil
            }
            "ui_window_begin" => {
                let id = gs(a,0,"UI_WINDOW_BEGIN")?.to_string();
                let title = gs(a,1,"UI_WINDOW_BEGIN")?.to_string();
                let x = gi(a,2,"UI_WINDOW_BEGIN")? as i32;
                let y = gi(a,3,"UI_WINDOW_BEGIN")? as i32;
                let w = gi(a,4,"UI_WINDOW_BEGIN")? as i32;
                let h = gi(a,5,"UI_WINDOW_BEGIN")? as i32;
                let (mut wx, mut wy, mut collapsed) =
                    *self.ui_state.windows.entry(id.clone()).or_insert((x, y, false));
                // Fenster-Management nutzt IMMER die echte Maus (nie gated).
                let (mx, my, is_down) = {
                    let g = self.gfx.as_ref().ok_or("UI_WINDOW_BEGIN vor SCREEN")?;
                    (g.mouse_x() as i32, g.mouse_y() as i32, g.mouse_button(0))
                };
                let just_pressed = is_down && !self.ui_state.was_mouse_down;
                let owns = self.ui_state.active_win.is_none()
                    || self.ui_state.active_win.as_deref() == Some(id.as_str());
                let title_h = self.ui_state.metric("win_title_h") as i32;
                let full_h = if collapsed { title_h } else { h };
                if ui_in_box(mx, my, wx, wy, w, full_h) {
                    self.ui_state.hover_win = Some(id.clone());   // letzter Schreiber = oberstes
                }
                const COLLAPSE: i32 = 12;
                let cb_x = wx + 4;
                let cb_y = wy + (title_h - COLLAPSE) / 2;
                if owns && just_pressed {
                    if ui_in_box(mx, my, cb_x, cb_y, COLLAPSE, COLLAPSE) {
                        collapsed = !collapsed;
                    } else if ui_in_box(mx, my, wx, wy, w, title_h) {
                        self.ui_state.drag_win = Some(id.clone());
                        self.ui_state.drag_off = (mx - wx, my - wy);
                    }
                }
                if self.ui_state.drag_win.as_deref() == Some(id.as_str()) && is_down {
                    wx = mx - self.ui_state.drag_off.0;
                    wy = my - self.ui_state.drag_off.1;
                }
                self.ui_state.windows.insert(id.clone(), (wx, wy, collapsed));
                // Zeichnen
                let draw_h = if collapsed { title_h } else { h };
                let (win_bg, win_border) = (self.ui_state.th("win_bg"), self.ui_state.th("win_border"));
                let title_bg = if owns { self.ui_state.th("win_title_bg_focus") } else { self.ui_state.th("win_title_bg") };
                let fg = self.ui_state.th("text_fg");
                let pl = self.ui_state.plastik();
                {
                    let g = self.gfx.as_mut().ok_or("UI_WINDOW_BEGIN vor SCREEN")?;
                    ui_flaeche(g, pl, wx, wy, w, draw_h, win_bg, win_border, false);
                    // Titelleiste: oben rund wie das Fenster, unten buendig --
                    // deshalb hier ohne eigene Rundung.
                    ui_flaeche(g, UiPlastik { rad: 0, ..pl }, wx, wy, w, title_h, title_bg, win_border, false);
                    g.text(cb_x, cb_y - 2, (if collapsed { "+" } else { "-" }).to_string(), fg);
                    g.text(wx + COLLAPSE + 8, wy + 3, title, fg);
                }
                self.ui_state.win_stack.push((self.ui_state.offset_x, self.ui_state.offset_y, self.ui_state.input_blocked));
                self.ui_state.offset_x = wx;
                self.ui_state.offset_y = wy + title_h;
                self.ui_state.input_blocked = !owns;
                Value::Bool(!collapsed)
            }
            "ui_window_end" => {
                if let Some((ox, oy, blk)) = self.ui_state.win_stack.pop() {
                    self.ui_state.offset_x = ox;
                    self.ui_state.offset_y = oy;
                    self.ui_state.input_blocked = blk;
                }
                Value::Nil
            }
            "ui_table" => self.ui_table(a)?,
            "ui_table_selected" => {
                let id = gs(a,0,"UI_TABLE_SELECTED")?;
                Value::Int(self.ui_state.tables.get(id).map(|t| t.selected).unwrap_or(-1) as i64)
            }
            "ui_table_set_selected" => {
                let id = gs(a,0,"UI_TABLE_SET_SELECTED")?.to_string();
                let row = gi(a,1,"UI_TABLE_SET_SELECTED")?;
                if let Some(t) = self.ui_state.tables.get_mut(&id) { t.selected = if row >= 0 { row as i32 } else { -1 }; }
                Value::Nil
            }
            "ui_table_header_click" => {
                let id = gs(a,0,"UI_TABLE_HEADER_CLICK")?;
                Value::Int(self.ui_state.tables.get(id).map(|t| t.header_col).unwrap_or(-1) as i64)
            }
            "ui_end_frame" => {
                let (down, bs) = {
                    let g = self.gfx.as_ref().ok_or("UI_END_FRAME vor SCREEN")?;
                    (g.mouse_button(0), g.key_down(8))
                };
                if !down && self.ui_state.was_mouse_down {
                    self.ui_state.click_origin = None;
                    self.ui_state.drag_win = None;
                }
                self.ui_state.was_mouse_down = down;
                self.ui_state.prev_backspace = bs;
                self.ui_state.frame_count += 1;
                // Z-Order: oberstes gehovertes Fenster wird im NAECHSTEN Frame input-aktiv.
                self.ui_state.active_win = self.ui_state.hover_win.take();
                self.ui_state.offset_x = 0;
                self.ui_state.offset_y = 0;
                self.ui_state.input_blocked = false;
                self.ui_state.win_stack.clear();
                Value::Nil
            }
            "ui_reset" => {
                self.ui_state = UiState::new(); Value::Nil
            }
            "ui_theme_set" => {
                let k = gs(a,0,"UI_THEME_SET")?.to_string();
                if !self.ui_state.theme.contains_key(&k) { return Err(format!("UI_THEME_SET: unbekannter Schluessel '{}'", k)); }
                let c = gi(a,1,"UI_THEME_SET")?; if c < 0 || c > 0xFFFFFF { return Err("UI_THEME_SET: Farbe muss 0..0xFFFFFF sein".into()); }
                self.ui_state.theme.insert(k, c); Value::Nil
            }
            "ui_theme_get" => { let k = gs(a,0,"UI_THEME_GET")?; self.ui_state.theme.get(k).copied().map(Value::Int).ok_or_else(|| format!("UI_THEME_GET: unbekannter Schluessel '{}'", k))? }
            "ui_metric_set" => {
                let k = gs(a,0,"UI_METRIC_SET")?.to_string();
                if !self.ui_state.metrics.contains_key(&k) { return Err(format!("UI_METRIC_SET: unbekannter Schluessel '{}'", k)); }
                let v = gi(a,1,"UI_METRIC_SET")?; if v < 1 { return Err("UI_METRIC_SET: Wert muss >= 1 sein".into()); }
                self.ui_state.metrics.insert(k, v); Value::Nil
            }
            "ui_metric_get" => { let k = gs(a,0,"UI_METRIC_GET")?; self.ui_state.metrics.get(k).copied().map(Value::Int).ok_or_else(|| format!("UI_METRIC_GET: unbekannter Schluessel '{}'", k))? }
            "ui_theme_preset" => {
                let n = gs(a,0,"UI_THEME_PRESET")?.to_lowercase();
                let p = ui_preset(&n).ok_or_else(|| format!("UI_THEME_PRESET: unbekanntes Preset '{}'", n))?;
                for (k, v) in p { self.ui_state.theme.insert(k.to_string(), v); }
                // Ein Preset ist ein KOMPLETTER Look: Farben UND Plastik.
                for (k, v) in ui_preset_metrics(&n) { self.ui_state.metrics.insert(k.to_string(), v); }
                Value::Nil
            }

            "chart_draw" => {
                if a.len() != 1 { return Err("CHART_DRAW: erwartet 1 Argument -- Aufruf: CHART_DRAW(diagramm)".into()); }
                let c = crate::builtins::chart_h(&a[0], "CHART_DRAW")?.clone();
                // CHART_DRAW wertet auch die Maus aus und schreibt Hover/Klick
                // ins Handle zurueck -> veraenderlicher Zugriff.
                let mut c = c.borrow_mut();
                // Die Schrift: die eigene des Diagramms (`schrift`), sonst die
                // der Oberflaeche -- SETFONT des Programms oder die eingebaute
                // Sans (GUI_FONT(-1) = Pixelschrift). Gemessen wird mit der
                // aktiven Schrift, gezeichnet mit `schrift` -- darum gelten
                // beide fuer die Dauer des Zeichnens als dieselbe.
                let eigen = c.style.schrift;
                let tausch = if eigen >= 0 {
                    self.gfx.as_mut().map(|g| { let n = g.text_height(); g.schrift_tauschen(eigen, n) })
                } else { self.sans_eintauschen() };
                let g = self.gfx.as_mut().ok_or("CHART_DRAW vor SCREEN aufgerufen")?;
                c.style.schrift = g.active_font();
                c.draw(g);
                c.style.schrift = eigen;
                if let Some((f, n)) = tausch { g.schrift_tauschen(f, n); }
                Value::Nil
            }

            "particle_draw" => {
                let sys = match a.first() { Some(Value::Particles(p)) => p.clone(), _ => return Err("PARTICLE_DRAW erwartet PARTICLE_SYSTEM".into()) };
                let s = sys.borrow();
                let g = self.gfx.as_mut().ok_or("PARTICLE_DRAW vor SCREEN aufgerufen")?;
                let mode = s.mode.as_str();
                for p in &s.particles {
                    let col = particle_color(p.color, s.color_end, s.has_color_end, s.fade, p.age, p.lifetime);
                    let (x, y, sz) = (p.x as i32, p.y as i32, p.size);
                    match mode {
                        "pixel" => g.plot(x, y, col),
                        "square" => g.box_fill(x - sz, y - sz, x + sz, y + sz, col),
                        "streak" => g.line(x, y, (p.x - p.vx * 0.04) as i32, (p.y - p.vy * 0.04) as i32, col),
                        // glow additiv ist im Recording-Modell nicht direkt machbar -> Kreis-Approx.
                        _ => g.circle(x, y, sz.max(1), col),
                    }
                }
                Value::Nil
            }

            _ => return Ok(None), // kein Grafik-Builtin -> pure-Pfad
        };
        Ok(Some(r))
    }
}
