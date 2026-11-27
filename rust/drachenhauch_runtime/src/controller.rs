//! Character-Controller (CHAR_*). Reine Platformer-Physik mit
//! Coyote-Time, Jump-Buffering und Variable-Jump-Height. Nutzt
//! `TiledMap::sweep_axis` (tile_collide-Port) fuer die Kollision.

use crate::tiled::{TiledLayer, TiledMap};

pub struct CharController {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub vx: f64,
    pub vy: f64,
    pub on_ground: bool,
    pub on_wall_left: bool,
    pub on_wall_right: bool,
    pub facing: i64,
    // Konfiguration
    pub move_speed: f64,
    pub jump_velocity: f64, // positiv gespeichert
    pub gravity: f64,
    pub max_fall: f64,
    /// Gnadenfristen in Bildern zu 1/60 s (f64, damit `CHAR_SET_COYOTE_MS`
    /// und ein dt in `CHAR_UPDATE` keine Bruchteile verlieren).
    pub coyote_max: f64,
    pub jump_buffer_max: f64,
    pub variable_jump: bool,
    pub variable_jump_cut: f64,
    // Interne Counter
    coyote_counter: f64,
    jump_buffer_counter: f64,
    prev_jump_held: bool,
    was_jumping: bool,
    // Frame-Input
    in_axis_x: i64,
    in_jump_pressed: bool,
    in_jump_held: bool,
}

impl CharController {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        CharController {
            x, y, w, h,
            vx: 0.0, vy: 0.0,
            on_ground: false, on_wall_left: false, on_wall_right: false,
            facing: 1,
            move_speed: 2.0,
            jump_velocity: 6.0,
            gravity: 0.25,
            max_fall: 7.0,
            coyote_max: 6.0,
            jump_buffer_max: 6.0,
            variable_jump: true,
            variable_jump_cut: 0.5,
            coyote_counter: 0.0,
            jump_buffer_counter: 0.0,
            prev_jump_held: false,
            was_jumping: false,
            in_axis_x: 0,
            in_jump_pressed: false,
            in_jump_held: false,
        }
    }

    pub fn set_input(&mut self, axis_x: i64, jump_pressed: bool, jump_held: bool) {
        self.in_axis_x = axis_x.clamp(-1, 1);
        self.in_jump_pressed = jump_pressed;
        self.in_jump_held = jump_held;
    }

    /// Ein Schritt (Velocity -> Kollision -> Status). `bilder` ist die Dauer
    /// des Schritts in Bildern zu 1/60 s: 1.0 ohne dt (wie bisher, je Bild),
    /// sonst `dt * 60`. Geschwindigkeiten bleiben "Pixel je 1/60 s", damit
    /// ein Spiel mit festem Takt sich nicht aendert und eines mit dt bei 30
    /// oder 144 Bildern gleich schnell laeuft.
    pub fn update(&mut self, map: &TiledMap, layer: &TiledLayer, bilder: f64) {
        // 1. Coyote-Counter
        if self.on_ground {
            self.coyote_counter = self.coyote_max;
        } else if self.coyote_counter > 0.0 {
            self.coyote_counter = (self.coyote_counter - bilder).max(0.0);
        }
        // 2. Jump-Buffer setzen/decrement -- bewusst dasselbe if/else-if-Muster
        // wie der Coyote-Counter oben (Schritt 1): GENAU eine der beiden
        // Operationen pro Frame. Review-Fund: vorher wurde hier nur GESETZT,
        // das Decrement lief separat am Frame-ENDE (Schritt 9) und wurde dort
        // per `!in_jump_pressed`-Guard genau auf dem Press-Frame uebersprungen
        // -- macht den Jump-Buffer bei identischem `jump_buffer_max` effektiv
        // 1 Frame LAENGER als das Coyote-Fenster (7 statt 6 nutzbare Frames
        // bei max=6), obwohl beide dieselbe Gnadenfrist-Semantik haben sollen.
        if self.in_jump_pressed {
            self.jump_buffer_counter = self.jump_buffer_max;
        } else if self.jump_buffer_counter > 0.0 {
            self.jump_buffer_counter = (self.jump_buffer_counter - bilder).max(0.0);
        }
        // 3. Variable-Jump-Cut
        if self.variable_jump && self.was_jumping
            && self.prev_jump_held && !self.in_jump_held && self.vy < 0.0
        {
            self.vy *= self.variable_jump_cut;
            self.was_jumping = false;
        }
        self.prev_jump_held = self.in_jump_held;
        // 4. Horizontal
        self.vx = self.in_axis_x as f64 * self.move_speed;
        if self.in_axis_x > 0 {
            self.facing = 1;
        } else if self.in_axis_x < 0 {
            self.facing = -1;
        }
        // 5. Gravitation + Terminal
        self.vy += self.gravity * bilder;
        if self.vy > self.max_fall {
            self.vy = self.max_fall;
        }
        // 6. Kollision: X dann Y
        let (nx, hit_x) = map.sweep_axis(layer, self.x, self.y, self.w, self.h, self.vx * bilder, 0);
        self.x = nx;
        if hit_x {
            self.vx = 0.0;
        }
        let (ny, hit_y) = map.sweep_axis(layer, self.x, self.y, self.w, self.h, self.vy * bilder, 1);
        self.y = ny;
        if hit_y {
            if self.vy > 0.0 {
                self.on_ground = true;
                self.was_jumping = false;
            }
            self.vy = 0.0;
        } else {
            self.on_ground = false;
        }
        // 7. Sprung-Try (nach Y-Sweep)
        let can_jump = self.on_ground || self.coyote_counter > 0.0;
        if self.jump_buffer_counter > 0.0 && can_jump {
            self.vy = -self.jump_velocity;
            self.on_ground = false;
            self.coyote_counter = 0.0;
            self.jump_buffer_counter = 0.0;
            self.was_jumping = true;
        }
        // 8. Wand-Detection
        self.on_wall_left = false;
        self.on_wall_right = false;
        if !self.on_ground {
            let (_, pl) = map.sweep_axis(layer, self.x, self.y, self.w, self.h, -1.0, 0);
            self.on_wall_left = pl;
            let (_, pr) = map.sweep_axis(layer, self.x, self.y, self.w, self.h, 1.0, 0);
            self.on_wall_right = pr;
        }
        // (Jump-Buffer-Decrement laeuft jetzt in Schritt 2, symmetrisch zum
        // Coyote-Counter in Schritt 1 -- kein separates Frame-Ende-Decrement
        // mehr noetig.)
    }
}
