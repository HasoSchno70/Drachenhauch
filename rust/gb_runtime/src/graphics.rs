//! raylib-Grafik-Backend (Schritt 4). Nur mit `--features graphics`.
//!
//! Modell: Draw-Builtins zeichnen NICHT sofort, sondern haengen ein `Cmd`
//! an eine Liste. `CLS` leert die Liste + merkt die Clear-Farbe. `FLIP`
//! rendert alle Cmds in einem `begin_drawing`/`end_drawing`-Block und
//! praesentiert. Das vermeidet, raylibs Draw-Handle ueber Builtin-Aufrufe
//! hinweg zu halten (Borrow-Checker) und braucht keine Render-Texture.
//!
//! Bit-Identitaet gilt hier NICHT fuer Pixel (anderer Renderer als pygame) --
//! nur `PRINT`/stdout bleibt bit-identisch. Verifikation per Screenshot.

use std::collections::HashMap;

use raylib::prelude::*;

#[derive(Clone)]
enum Cmd {
    Clear(Color),
    Pixel(i32, i32, Color),
    Line(i32, i32, i32, i32, Color),
    BoxFill(i32, i32, i32, i32, Color),
    RectLines(i32, i32, i32, i32, Color),
    Circle(i32, i32, f32, Color),
    Triangle(i32, i32, i32, i32, i32, i32, Color),
    TriLines(i32, i32, i32, i32, i32, i32, Color),
    Ellipse(i32, i32, i32, i32, Color, bool), // cx, cy, rh, rv, color, filled
    Poly(Vec<(i32, i32)>, Color, bool),       // points, color, closed
    FillPoly(Vec<(i32, i32)>, Color),
    Text(i32, i32, String, i32, Color),
    Texture(usize, i32, i32),
    TexturePart(usize, i32, i32, i32, i32, i32, i32), // tex, sx,sy,sw,sh, dx,dy
    TextureFlipped(usize, i32, i32, bool, bool),       // tex, x, y, flip_h, flip_v
    AtlasDraw(usize, i32, i32, i32, i32, i32, i32, bool), // tex, sx,sy,sw,sh, dx,dy, flip_h
    // tex, src(sx,sy,sw,sh), dst(dx,dy,dw,dh), flip_x, flip_y, tint
    SpriteDraw(usize, i32, i32, i32, i32, i32, i32, i32, i32, bool, bool, Color),
}

struct Layer {
    z: i32,
    cmds: Vec<Cmd>,
}

struct Atlas {
    tex_idx: usize,
    frames: HashMap<String, (i32, i32, i32, i32)>,
}

/// GPU-Textur + CPU-Image (Image noetig fuer imgfx-Transformationen).
struct Tex {
    tex: Texture2D,
    img: Image,
}

pub struct Graphics {
    rl: RaylibHandle,
    thread: RaylibThread,
    width: i32,
    height: i32,
    scale: i32,
    // Z-Layer: Index 0 ist der Default-/Main-Layer (z=0). LAYER(name) schaltet
    // `active` um. FLIP komponiert alle Layer aufsteigend nach z und leert sie.
    layers: Vec<Layer>,
    layer_names: HashMap<String, usize>,
    active: usize,
    clear_color: Color,
    // Kamera (Modul `camera`): World->Screen-Transform fuer alle Draws.
    cam_x: f64,
    cam_y: f64,
    cam_zoom: f64,
    text_size: i32,
    textures: Vec<Tex>,
    image_cache: HashMap<String, i64>,
    atlases: Vec<Atlas>,
    pub frame_count: u64,
    max_frames: Option<u64>,
    screenshot: Option<String>,
    shot_taken: bool,
}

/// GB-Farbe (0xRRGGBB INTEGER) -> raylib Color.
fn col(c: i64) -> Color {
    let v = c as u32;
    Color::new(((v >> 16) & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, (v & 0xFF) as u8, 255)
}

impl Graphics {
    pub fn new(width: i32, height: i32, title: &str, scale: i32) -> Graphics {
        let win_w = width * scale;
        let win_h = height * scale;
        let (mut rl, thread) = raylib::init().size(win_w, win_h).title(title).build();
        rl.set_target_fps(60);
        // Headless-Verifizierung: GBRT_FRAMES begrenzt die Frames, GBRT_SCREENSHOT
        // legt den PNG-Pfad fest (Screenshot beim letzten Frame).
        let max_frames = std::env::var("GBRT_FRAMES").ok().and_then(|s| s.parse().ok());
        let screenshot = std::env::var("GBRT_SCREENSHOT").ok();
        let mut layer_names = HashMap::new();
        layer_names.insert(String::new(), 0usize); // Main-Layer
        Graphics {
            rl, thread, width, height, scale,
            layers: vec![Layer { z: 0, cmds: Vec::new() }],
            layer_names,
            active: 0,
            clear_color: Color::BLACK,
            cam_x: 0.0, cam_y: 0.0, cam_zoom: 1.0,
            text_size: 20,
            textures: Vec::new(),
            image_cache: HashMap::new(),
            atlases: Vec::new(),
            frame_count: 0,
            max_frames,
            screenshot,
            shot_taken: false,
        }
    }

    fn emit(&mut self, c: Cmd) {
        let a = self.active;
        self.layers[a].cmds.push(c);
    }

    // --- Kamera (Modul `camera`) ---
    // World->Screen: sx = int((x - cam_x) * zoom). Bei Identitaet (0,0,1) No-Op.
    fn w2s(&self, x: i32, y: i32) -> (i32, i32) {
        (((x as f64 - self.cam_x) * self.cam_zoom) as i32,
         ((y as f64 - self.cam_y) * self.cam_zoom) as i32)
    }
    fn ssize(&self, s: i32) -> i32 { ((s as f64 * self.cam_zoom) as i32).max(0) }
    pub fn set_camera(&mut self, x: f64, y: f64, zoom: f64) { self.cam_x = x; self.cam_y = y; self.cam_zoom = zoom; }
    pub fn reset_camera(&mut self) { self.cam_x = 0.0; self.cam_y = 0.0; self.cam_zoom = 1.0; }
    pub fn camera(&self) -> (f64, f64, f64) { (self.cam_x, self.cam_y, self.cam_zoom) }
    pub fn s2w_x(&self, sx: f64) -> f64 { if self.cam_zoom == 0.0 { sx } else { sx / self.cam_zoom + self.cam_x } }
    pub fn s2w_y(&self, sy: f64) -> f64 { if self.cam_zoom == 0.0 { sy } else { sy / self.cam_zoom + self.cam_y } }

    pub fn cls(&mut self, color: i64) {
        // CLS setzt die Hintergrundfarbe (beim FLIP gecleart) und leert den
        // aktiven Layer (Wipe). Die Layer werden ohnehin pro FLIP geleert.
        self.clear_color = col(color);
        let a = self.active;
        self.layers[a].cmds.clear();
    }

    // --- Z-Layer ---
    pub fn layer_define(&mut self, name: &str, z: i32) {
        if let Some(&i) = self.layer_names.get(name) {
            self.layers[i].z = z;
        } else {
            self.layers.push(Layer { z, cmds: Vec::new() });
            self.layer_names.insert(name.to_string(), self.layers.len() - 1);
        }
    }
    pub fn layer(&mut self, name: &str) {
        if let Some(&i) = self.layer_names.get(name) {
            self.active = i;
        } else {
            // Auto-Define mit Auto-Z (hoechstes + 10).
            let z = self.layers.iter().map(|l| l.z).max().unwrap_or(0) + 10;
            self.layers.push(Layer { z, cmds: Vec::new() });
            let idx = self.layers.len() - 1;
            self.layer_names.insert(name.to_string(), idx);
            self.active = idx;
        }
    }
    pub fn layer_end(&mut self) { self.active = 0; }
    pub fn layer_clear(&mut self, name: &str) {
        if let Some(&i) = self.layer_names.get(name) { self.layers[i].cmds.clear(); }
    }

    pub fn plot(&mut self, x: i32, y: i32, c: i64) { let (x, y) = self.w2s(x, y); self.emit(Cmd::Pixel(x, y, col(c))); }
    pub fn line(&mut self, a: i32, b: i32, cc: i32, d: i32, c: i64) { let (a, b) = self.w2s(a, b); let (cc, d) = self.w2s(cc, d); self.emit(Cmd::Line(a, b, cc, d, col(c))); }
    pub fn box_fill(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i64) { let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2); self.emit(Cmd::BoxFill(x1, y1, x2, y2, col(c))); }
    pub fn rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i64) { let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2); self.emit(Cmd::RectLines(x1, y1, x2, y2, col(c))); }
    pub fn circle(&mut self, x: i32, y: i32, r: i32, c: i64) { let (x, y) = self.w2s(x, y); let r = self.ssize(r); self.emit(Cmd::Circle(x, y, r as f32, col(c))); }
    pub fn triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2); let (x3, y3) = self.w2s(x3, y3);
        self.emit(Cmd::Triangle(x1, y1, x2, y2, x3, y3, col(c)));
    }
    pub fn triangle_outline(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2); let (x3, y3) = self.w2s(x3, y3);
        self.emit(Cmd::TriLines(x1, y1, x2, y2, x3, y3, col(c)));
    }
    fn ellipse_box(x1: i32, y1: i32, x2: i32, y2: i32) -> (i32, i32, i32, i32) {
        let x = x1.min(x2); let y = y1.min(y2);
        let w = (x2 - x1).abs() + 1; let h = (y2 - y1).abs() + 1;
        (x + w / 2, y + h / 2, w / 2, h / 2) // cx, cy, rh, rv
    }
    pub fn ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2);
        let (cx, cy, rh, rv) = Self::ellipse_box(x1, y1, x2, y2);
        self.emit(Cmd::Ellipse(cx, cy, rh, rv, col(c), true));
    }
    pub fn ellipse_outline(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2);
        let (cx, cy, rh, rv) = Self::ellipse_box(x1, y1, x2, y2);
        self.emit(Cmd::Ellipse(cx, cy, rh, rv, col(c), false));
    }
    pub fn arc(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, start: f64, end: f64, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2);
        let (cx, cy, rh, rv) = Self::ellipse_box(x1, y1, x2, y2);
        let n = 48;
        let mut pts = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let t = start + (end - start) * (i as f64) / (n as f64);
            let px = cx as f64 + rh as f64 * t.cos();
            let py = cy as f64 - rv as f64 * t.sin(); // y nach unten -> CCW = minus sin
            pts.push((px.round() as i32, py.round() as i32));
        }
        self.emit(Cmd::Poly(pts, col(c), false));
    }
    pub fn polygon(&mut self, flat: &[i32], c: i64, filled: bool) -> Result<(), String> {
        if flat.len() < 6 || flat.len() % 2 != 0 {
            return Err("POLYGON: braucht mindestens 3 Punkte (6 Werte)".into());
        }
        let pts: Vec<(i32, i32)> = flat.chunks(2).map(|p| { let (x, y) = self.w2s(p[0], p[1]); (x, y) }).collect();
        if filled { self.emit(Cmd::FillPoly(pts, col(c))); }
        else { self.emit(Cmd::Poly(pts, col(c), true)); }
        Ok(())
    }
    pub fn text(&mut self, x: i32, y: i32, s: String, c: i64) {
        // Position via w2s (inkl. Zoom), aber Font-Groesse bleibt -- wie Python.
        let (x, y) = self.w2s(x, y);
        let sz = self.text_size;
        self.emit(Cmd::Text(x, y, s, sz, col(c)));
    }
    pub fn set_text_size(&mut self, sz: i32) { self.text_size = sz.max(1); }
    pub fn text_width(&self, s: &str) -> i32 {
        let c = std::ffi::CString::new(s).unwrap_or_default();
        unsafe { raylib::ffi::MeasureText(c.as_ptr(), self.text_size) }
    }
    pub fn text_height(&self) -> i32 { self.text_size }

    pub fn load_texture(&mut self, path: &str) -> Result<i64, String> {
        if let Some(&h) = self.image_cache.get(path) { return Ok(h); }
        // CPU-Image laden (fuer imgfx) + GPU-Textur daraus.
        let img = Image::load_image(path).map_err(|e| format!("LOADIMAGE: {}", e))?;
        let tex = self.rl.load_texture_from_image(&self.thread, &img).map_err(|e| format!("LOADIMAGE: {}", e))?;
        self.textures.push(Tex { tex, img });
        let h = (self.textures.len() - 1) as i64;
        self.image_cache.insert(path.to_string(), h);
        if let Ok(abs) = std::fs::canonicalize(path) {
            self.image_cache.insert(abs.to_string_lossy().to_string(), h);
        }
        Ok(h)
    }

    fn push_tex_from_image(&mut self, img: Image) -> Result<i64, String> {
        let tex = self.rl.load_texture_from_image(&self.thread, &img).map_err(|e| format!("IMAGE: {}", e))?;
        self.textures.push(Tex { tex, img });
        Ok((self.textures.len() - 1) as i64)
    }

    fn src_image(&self, idx: i64, fn_: &str) -> Result<Image, String> {
        self.textures.get(idx as usize).map(|t| t.img.clone()).ok_or_else(|| format!("{}: ungueltiges IMAGE-Handle", fn_))
    }

    // --- imgfx (immutable: liefern neues IMAGE-Handle) ---
    pub fn image_scale(&mut self, idx: i64, w: i32, h: i32) -> Result<i64, String> {
        if w <= 0 || h <= 0 { return Err("IMAGE_SCALE: w und h muessen > 0 sein".into()); }
        let mut img = self.src_image(idx, "IMAGE_SCALE")?;
        img.resize(w, h);
        self.push_tex_from_image(img)
    }
    pub fn image_rotate(&mut self, idx: i64, degrees: f32) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_ROTATE")?;
        img.rotate(degrees as i32);
        self.push_tex_from_image(img)
    }
    pub fn image_flip(&mut self, idx: i64, fx: bool, fy: bool) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_FLIP")?;
        if fx { img.flip_horizontal(); }
        if fy { img.flip_vertical(); }
        self.push_tex_from_image(img)
    }
    pub fn image_tint(&mut self, idx: i64, color: i64) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_TINT")?;
        img.color_tint(col(color));
        self.push_tex_from_image(img)
    }
    pub fn image_copy(&mut self, idx: i64) -> Result<i64, String> {
        let img = self.src_image(idx, "IMAGE_COPY")?;
        self.push_tex_from_image(img)
    }
    fn cache_image_alias(&mut self, alias: &str, handle: i64) {
        self.image_cache.insert(alias.to_string(), handle);
    }
    pub fn draw_image(&mut self, idx: i64, x: i32, y: i32) -> Result<(), String> {
        let i = idx as usize;
        if i >= self.textures.len() { return Err("DRAWIMAGE: ungueltiges IMAGE-Handle".into()); }
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::Texture(i, x, y));
        Ok(())
    }
    pub fn draw_image_part(&mut self, idx: i64, sx: i32, sy: i32, sw: i32, sh: i32, dx: i32, dy: i32) -> Result<(), String> {
        let i = idx as usize;
        if i >= self.textures.len() { return Err("DRAWIMAGEPART: ungueltiges IMAGE-Handle".into()); }
        let (dx, dy) = self.w2s(dx, dy);
        self.emit(Cmd::TexturePart(i, sx, sy, sw, sh, dx, dy));
        Ok(())
    }
    pub fn draw_image_flipped(&mut self, idx: i64, x: i32, y: i32, fh: bool, fv: bool) -> Result<(), String> {
        let i = idx as usize;
        if i >= self.textures.len() { return Err("DRAWIMAGEFLIPPED: ungueltiges IMAGE-Handle".into()); }
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::TextureFlipped(i, x, y, fh, fv));
        Ok(())
    }
    pub fn image_width(&self, idx: i64) -> Result<i64, String> {
        self.textures.get(idx as usize).map(|t| t.tex.width as i64).ok_or_else(|| "IMAGEWIDTH: ungueltiges Handle".into())
    }
    pub fn image_height(&self, idx: i64) -> Result<i64, String> {
        self.textures.get(idx as usize).map(|t| t.tex.height as i64).ok_or_else(|| "IMAGEHEIGHT: ungueltiges Handle".into())
    }

    // --- LOAD_ASSETS: Bilder aus JSON-Manifest vorladen (Alias/Pfad-Cache) ---
    pub fn load_assets(&mut self, manifest_path: &str) -> Result<i64, String> {
        let text = std::fs::read_to_string(manifest_path)
            .map_err(|_| format!("LOAD_ASSETS: Manifest nicht gefunden: {}", manifest_path))?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("LOAD_ASSETS: Manifest-Lesefehler '{}': {}", manifest_path, e))?;
        let dir = std::path::Path::new(manifest_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let mut count = 0i64;
        if let Some(images) = json.get("images") {
            if let Some(obj) = images.as_object() {
                for (alias, rel) in obj {
                    let rel = rel.as_str().ok_or("LOAD_ASSETS: Pfad muss STRING sein")?;
                    let full = dir.join(rel);
                    let h = self.load_texture(&full.to_string_lossy())?;
                    self.cache_image_alias(alias, h);
                    count += 1;
                }
            } else if let Some(arr) = images.as_array() {
                for rel in arr {
                    let rel = rel.as_str().ok_or("LOAD_ASSETS: Pfad muss STRING sein")?;
                    let full = dir.join(rel);
                    self.load_texture(&full.to_string_lossy())?;
                    count += 1;
                }
            }
        }
        // "sounds" wird ignoriert (Audio = spaeterer Schritt).
        Ok(count)
    }

    // --- Sprite-Atlas ---
    pub fn atlas_load(&mut self, manifest_path: &str) -> Result<i64, String> {
        let text = std::fs::read_to_string(manifest_path)
            .map_err(|_| format!("ATLAS_LOAD: Manifest nicht gefunden: {}", manifest_path))?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("ATLAS_LOAD: {}", e))?;
        let dir = std::path::Path::new(manifest_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let image = json.get("image").and_then(|v| v.as_str())
            .ok_or("ATLAS_LOAD: 'image' fehlt im Manifest")?;
        let full = dir.join(image);
        let tex_idx = self.load_texture(&full.to_string_lossy())? as usize;
        let mut frames = HashMap::new();
        if let Some(sprites) = json.get("sprites").and_then(|v| v.as_object()) {
            for (name, rect) in sprites {
                let arr = rect.as_array().ok_or("ATLAS_LOAD: rect muss [x,y,w,h] sein")?;
                if arr.len() != 4 { return Err(format!("ATLAS_LOAD: rect '{}' braucht 4 Werte", name)); }
                let g = |i: usize| arr[i].as_i64().unwrap_or(0) as i32;
                frames.insert(name.clone(), (g(0), g(1), g(2), g(3)));
            }
        }
        self.atlases.push(Atlas { tex_idx, frames });
        Ok((self.atlases.len() - 1) as i64)
    }
    pub fn atlas_draw(&mut self, atlas: i64, name: &str, x: i32, y: i32, flip_h: bool) -> Result<(), String> {
        let (tex, sx, sy, sw, sh) = {
            let a = self.atlases.get(atlas as usize).ok_or("ATLAS_DRAW: ungueltiges Atlas-Handle")?;
            let &(sx, sy, sw, sh) = a.frames.get(name)
                .ok_or_else(|| format!("ATLAS_DRAW: Sprite '{}' nicht im Atlas", name))?;
            (a.tex_idx, sx, sy, sw, sh)
        };
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::AtlasDraw(tex, sx, sy, sw, sh, x, y, flip_h));
        Ok(())
    }

    /// SPRITE_DRAW: aktuelles Frame als Sheet-Sub-Rect, Camera-aware.
    pub fn draw_sprite(&mut self, tex_idx: i64, frame: i32, fw: i32, fh: i32,
                       x: i32, y: i32, flip_x: bool, flip_y: bool,
                       scale_x: f64, scale_y: f64, tint: Option<i64>) -> Result<(), String> {
        let i = tex_idx as usize;
        let tex_w = self.textures.get(i).map(|t| t.tex.width).ok_or("SPRITE_DRAW: ungueltiges IMAGE-Handle")?;
        let cols = (tex_w / fw).max(1);
        let (gcol, grow) = (frame % cols, frame / cols);
        let (sx, sy) = (gcol * fw, grow * fh);
        let (dx, dy) = self.w2s(x, y);
        // Zielgroesse: Frame * Sprite-Scale * Camera-Zoom (Screen-Scale `s` im Replay).
        let dw = (fw as f64 * scale_x * self.cam_zoom).max(1.0) as i32;
        let dh = (fh as f64 * scale_y * self.cam_zoom).max(1.0) as i32;
        let tint_col = match tint {
            Some(c) => col(c),
            None => Color::WHITE,
        };
        self.emit(Cmd::SpriteDraw(i, sx, sy, fw, fh, dx, dy, dw, dh, flip_x, flip_y, tint_col));
        Ok(())
    }

    pub fn key_down(&self, code: i64) -> bool {
        match map_key(code) { Some(k) => self.rl.is_key_down(k), None => false }
    }
    /// Leert raylibs Tipp-Zeichen-Queue dieses Frames und liefert die getippten
    /// Zeichen als String (für UI_TEXTFIELD). Wie pygames Text-Input-Puffer.
    pub fn pop_text_input(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.rl.get_char_pressed() {
            s.push(c);
        }
        s
    }
    pub fn mouse_x(&self) -> i64 { (self.rl.get_mouse_x() / self.scale) as i64 }
    pub fn mouse_y(&self) -> i64 { (self.rl.get_mouse_y() / self.scale) as i64 }
    pub fn mouse_button(&self, b: i64) -> bool {
        let btn = match b { 0 => MouseButton::MOUSE_BUTTON_LEFT, 1 => MouseButton::MOUSE_BUTTON_RIGHT, 2 => MouseButton::MOUSE_BUTTON_MIDDLE, _ => return false };
        self.rl.is_mouse_button_down(btn)
    }
    pub fn millis(&self) -> i64 { (self.rl.get_time() * 1000.0) as i64 }

    pub fn quit_requested(&self) -> bool {
        if let Some(mx) = self.max_frames {
            if self.frame_count >= mx { return true; }
        }
        self.rl.window_should_close()
    }

    pub fn flip(&mut self) {
        let s = self.scale;
        let clear_color = self.clear_color;
        // Layer aufsteigend nach z komponieren (niedrigstes z = hinten).
        let mut order: Vec<usize> = (0..self.layers.len()).collect();
        order.sort_by_key(|&i| self.layers[i].z);
        let Graphics { rl, thread, layers, textures, .. } = self;
        {
            let mut d = rl.begin_drawing(thread);
            d.clear_background(clear_color);
            for &li in &order {
              for c in layers[li].cmds.iter() {
                match c {
                    Cmd::Clear(col) => d.clear_background(*col),
                    Cmd::Pixel(x, y, col) => {
                        if s == 1 { d.draw_pixel(*x, *y, *col); }
                        else { d.draw_rectangle(x * s, y * s, s, s, *col); }
                    }
                    Cmd::Line(a, b, cc, dd, col) => d.draw_line(a * s, b * s, cc * s, dd * s, *col),
                    Cmd::BoxFill(x1, y1, x2, y2, col) => {
                        let x = (*x1).min(*x2) * s; let y = (*y1).min(*y2) * s;
                        let w = ((x2 - x1).abs() + 1) * s; let h = ((y2 - y1).abs() + 1) * s;
                        d.draw_rectangle(x, y, w, h, *col);
                    }
                    Cmd::RectLines(x1, y1, x2, y2, col) => {
                        let x = (*x1).min(*x2) * s; let y = (*y1).min(*y2) * s;
                        let w = ((x2 - x1).abs() + 1) * s; let h = ((y2 - y1).abs() + 1) * s;
                        d.draw_rectangle_lines(x, y, w, h, *col);
                    }
                    Cmd::Circle(x, y, r, col) => d.draw_circle(x * s, y * s, r * s as f32, *col),
                    Cmd::Triangle(x1, y1, x2, y2, x3, y3, col) => {
                        // raylib erwartet CCW; wir uebergeben wie angegeben.
                        d.draw_triangle(
                            Vector2::new((x1 * s) as f32, (y1 * s) as f32),
                            Vector2::new((x2 * s) as f32, (y2 * s) as f32),
                            Vector2::new((x3 * s) as f32, (y3 * s) as f32),
                            *col,
                        );
                    }
                    Cmd::TriLines(x1, y1, x2, y2, x3, y3, col) => {
                        d.draw_triangle_lines(
                            Vector2::new((x1 * s) as f32, (y1 * s) as f32),
                            Vector2::new((x2 * s) as f32, (y2 * s) as f32),
                            Vector2::new((x3 * s) as f32, (y3 * s) as f32),
                            *col,
                        );
                    }
                    Cmd::Ellipse(cx, cy, rh, rv, col, filled) => {
                        if *filled { d.draw_ellipse(cx * s, cy * s, (*rh * s) as f32, (*rv * s) as f32, *col); }
                        else { d.draw_ellipse_lines(cx * s, cy * s, (*rh * s) as f32, (*rv * s) as f32, *col); }
                    }
                    Cmd::Poly(pts, col, closed) => {
                        let n = pts.len();
                        if n >= 2 {
                            for i in 0..n - 1 {
                                d.draw_line(pts[i].0 * s, pts[i].1 * s, pts[i + 1].0 * s, pts[i + 1].1 * s, *col);
                            }
                            if *closed {
                                d.draw_line(pts[n - 1].0 * s, pts[n - 1].1 * s, pts[0].0 * s, pts[0].1 * s, *col);
                            }
                        }
                    }
                    Cmd::FillPoly(pts, col) => {
                        // Triangle-Fan (korrekt fuer konvexe Polygone).
                        if pts.len() >= 3 {
                            let v: Vec<Vector2> = pts.iter().map(|p| Vector2::new((p.0 * s) as f32, (p.1 * s) as f32)).collect();
                            d.draw_triangle_fan(&v, *col);
                        }
                    }
                    Cmd::Text(x, y, txt, sz, col) => d.draw_text(txt, x * s, y * s, sz * s, *col),
                    Cmd::Texture(i, x, y) => {
                        if s == 1 { d.draw_texture(&textures[*i].tex, *x, *y, Color::WHITE); }
                        else { d.draw_texture_ex(&textures[*i].tex, Vector2::new((x * s) as f32, (y * s) as f32), 0.0, s as f32, Color::WHITE); }
                    }
                    Cmd::TexturePart(i, sx, sy, sw, sh, dx, dy) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32, *sw as f32, *sh as f32);
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (sw * s) as f32, (sh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                    }
                    Cmd::TextureFlipped(i, x, y, fh, fv) => {
                        let t = &textures[*i].tex;
                        let sw = if *fh { -(t.width as f32) } else { t.width as f32 };
                        let sh = if *fv { -(t.height as f32) } else { t.height as f32 };
                        let src = Rectangle::new(0.0, 0.0, sw, sh);
                        let dst = Rectangle::new((x * s) as f32, (y * s) as f32, (t.width * s) as f32, (t.height * s) as f32);
                        d.draw_texture_pro(t, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                    }
                    Cmd::AtlasDraw(i, sx, sy, sw, sh, dx, dy, fh) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32, if *fh { -(*sw as f32) } else { *sw as f32 }, *sh as f32);
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (sw * s) as f32, (sh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0,Color::WHITE);
                    }
                    Cmd::SpriteDraw(i, sx, sy, sw, sh, dx, dy, dw, dh, fx, fy, tint) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32,
                            if *fx { -(*sw as f32) } else { *sw as f32 },
                            if *fy { -(*sh as f32) } else { *sh as f32 });
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (dw * s) as f32, (dh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0,*tint);
                    }
                }
              }
            }
        }
        // Layer fuer den naechsten Frame leeren (Immediate-Mode pro Frame).
        for l in self.layers.iter_mut() { l.cmds.clear(); }
        self.frame_count += 1;
        // Headless-Screenshot beim Erreichen der Frame-Grenze.
        if let (Some(mx), Some(path), false) = (self.max_frames, self.screenshot.clone(), self.shot_taken) {
            if self.frame_count >= mx {
                self.rl.take_screenshot(&self.thread, &path);
                self.shot_taken = true;
            }
        }
    }
}

/// SDL/pygame-Keycode (Wert der GB-KEY_*-Konstanten) -> raylib KeyboardKey.
fn map_key(code: i64) -> Option<KeyboardKey> {
    use KeyboardKey::*;
    Some(match code {
        27 => KEY_ESCAPE,
        13 => KEY_ENTER,
        32 => KEY_SPACE,
        9 => KEY_TAB,
        8 => KEY_BACKSPACE,
        1073741904 => KEY_LEFT,
        1073741903 => KEY_RIGHT,
        1073741906 => KEY_UP,
        1073741905 => KEY_DOWN,
        // Buchstaben: pygame 97..122 (lowercase ascii) -> raylib 65..90.
        97..=122 => return key_from_i32((code - 32) as i32),
        // Ziffern: pygame 48..57 == raylib KEY_ZERO..KEY_NINE.
        48..=57 => return key_from_i32(code as i32),
        // F1..F12: pygame 1073741882.. -> raylib KEY_F1=290..
        1073741882..=1073741893 => return key_from_i32((290 + (code - 1073741882)) as i32),
        _ => return None,
    })
}

fn key_from_i32(v: i32) -> Option<KeyboardKey> {
    // raylib-rs bietet kein from_i32 stabil ueber alle Versionen; wir mappen
    // die hier benoetigten Bereiche (Buchstaben/Ziffern/F-Keys) explizit.
    use KeyboardKey::*;
    Some(match v {
        65 => KEY_A, 66 => KEY_B, 67 => KEY_C, 68 => KEY_D, 69 => KEY_E, 70 => KEY_F,
        71 => KEY_G, 72 => KEY_H, 73 => KEY_I, 74 => KEY_J, 75 => KEY_K, 76 => KEY_L,
        77 => KEY_M, 78 => KEY_N, 79 => KEY_O, 80 => KEY_P, 81 => KEY_Q, 82 => KEY_R,
        83 => KEY_S, 84 => KEY_T, 85 => KEY_U, 86 => KEY_V, 87 => KEY_W, 88 => KEY_X,
        89 => KEY_Y, 90 => KEY_Z,
        48 => KEY_ZERO, 49 => KEY_ONE, 50 => KEY_TWO, 51 => KEY_THREE, 52 => KEY_FOUR,
        53 => KEY_FIVE, 54 => KEY_SIX, 55 => KEY_SEVEN, 56 => KEY_EIGHT, 57 => KEY_NINE,
        290 => KEY_F1, 291 => KEY_F2, 292 => KEY_F3, 293 => KEY_F4, 294 => KEY_F5,
        295 => KEY_F6, 296 => KEY_F7, 297 => KEY_F8, 298 => KEY_F9, 299 => KEY_F10,
        300 => KEY_F11, 301 => KEY_F12,
        _ => return None,
    })
}
