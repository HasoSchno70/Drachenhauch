//! Box-vs-Tilemap-Sweep. Spiegelt `tile_collide.py::_sweep_axis` exakt
//! (inkl. der floor/Kanten-Logik), arbeitet aber auf einer einmal nach Rust
//! gespiegelten Solid-Maske (Map ist nach dem Load immutable). Liefert wie
//! das Python-Original (neue_pos, hit_flag).

use pyo3::prelude::*;

#[pyclass]
pub struct TileCollider {
    width: i32,
    height: i32,
    tile_w: f64,
    tile_h: f64,
    solid: Vec<bool>,
}

impl TileCollider {
    #[inline]
    fn solid_at(&self, tx: i64, ty: i64) -> bool {
        // Out-of-Bounds = solid (Welt-Rand blockiert) -- gleiche Konvention
        // wie tile_collide._tile_is_solid_at.
        if tx < 0 || ty < 0 || tx >= self.width as i64 || ty >= self.height as i64 {
            return true;
        }
        self.solid[(ty * self.width as i64 + tx) as usize]
    }

    fn sweep_impl(&self, x: f64, y: f64, w: f64, h: f64, delta: f64, axis_y: bool) -> (f64, bool) {
        let tw = self.tile_w;
        let th = self.tile_h;
        if tw <= 0.0 || th <= 0.0 {
            return (if axis_y { y + delta } else { x + delta }, false);
        }
        if delta == 0.0 {
            return (if axis_y { y } else { x }, false);
        }

        if !axis_y {
            let new_x = x + delta;
            if delta > 0.0 {
                let front_edge = new_x + w;
                let mut tx_far = (front_edge / tw).floor() as i64;
                if front_edge == (tx_far as f64) * tw {
                    tx_far -= 1;
                }
                let tx_old = if x + w > 0.0 {
                    ((x + w - 1.0) / tw).floor() as i64
                } else {
                    -1
                };
                let ty_top = (y / th).floor() as i64;
                let ty_bot = ((y + h - 1.0) / th).floor() as i64;
                let mut tx = tx_old + 1;
                while tx <= tx_far {
                    let mut ty = ty_top;
                    while ty <= ty_bot {
                        if self.solid_at(tx, ty) {
                            return ((tx as f64) * tw - w, true);
                        }
                        ty += 1;
                    }
                    tx += 1;
                }
                (new_x, false)
            } else {
                let front_edge = new_x;
                let tx_far = (front_edge / tw).floor() as i64;
                let tx_old = (x / tw).floor() as i64;
                let ty_top = (y / th).floor() as i64;
                let ty_bot = ((y + h - 1.0) / th).floor() as i64;
                let mut tx = tx_old - 1;
                while tx >= tx_far {
                    let mut ty = ty_top;
                    while ty <= ty_bot {
                        if self.solid_at(tx, ty) {
                            return (((tx + 1) as f64) * tw, true);
                        }
                        ty += 1;
                    }
                    tx -= 1;
                }
                (new_x, false)
            }
        } else {
            let new_y = y + delta;
            if delta > 0.0 {
                let front_edge = new_y + h;
                let mut ty_far = (front_edge / th).floor() as i64;
                if front_edge == (ty_far as f64) * th {
                    ty_far -= 1;
                }
                let ty_old = if y + h > 0.0 {
                    ((y + h - 1.0) / th).floor() as i64
                } else {
                    -1
                };
                let tx_left = (x / tw).floor() as i64;
                let tx_right = ((x + w - 1.0) / tw).floor() as i64;
                let mut ty = ty_old + 1;
                while ty <= ty_far {
                    let mut tx = tx_left;
                    while tx <= tx_right {
                        if self.solid_at(tx, ty) {
                            return ((ty as f64) * th - h, true);
                        }
                        tx += 1;
                    }
                    ty += 1;
                }
                (new_y, false)
            } else {
                let front_edge = new_y;
                let ty_far = (front_edge / th).floor() as i64;
                let ty_old = (y / th).floor() as i64;
                let tx_left = (x / tw).floor() as i64;
                let tx_right = ((x + w - 1.0) / tw).floor() as i64;
                let mut ty = ty_old - 1;
                while ty >= ty_far {
                    let mut tx = tx_left;
                    while tx <= tx_right {
                        if self.solid_at(tx, ty) {
                            return (((ty + 1) as f64) * th, true);
                        }
                        tx += 1;
                    }
                    ty -= 1;
                }
                (new_y, false)
            }
        }
    }
}

#[pymethods]
impl TileCollider {
    // width/height in Tiles, tile_w/tile_h in Pixel, solid: row-major Maske
    // (width*height). Alles vom Wrapper aus der immutablen Tiled-Map gebaut.
    #[new]
    fn new(width: i32, height: i32, tile_w: f64, tile_h: f64, solid: Vec<bool>) -> Self {
        TileCollider { width, height, tile_w, tile_h, solid }
    }

    fn is_solid_at(&self, tx: i64, ty: i64) -> bool {
        self.solid_at(tx, ty)
    }

    fn sweep_x(&self, x: f64, y: f64, w: f64, h: f64, dx: f64) -> (f64, bool) {
        self.sweep_impl(x, y, w, h, dx, false)
    }

    fn sweep_y(&self, x: f64, y: f64, w: f64, h: f64, dy: f64) -> (f64, bool) {
        self.sweep_impl(x, y, w, h, dy, true)
    }

    fn __repr__(&self) -> String {
        let n_solid = self.solid.iter().filter(|&&s| s).count();
        format!(
            "<TileCollider {}x{} tiles, {} solid>",
            self.width, self.height, n_solid
        )
    }
}
