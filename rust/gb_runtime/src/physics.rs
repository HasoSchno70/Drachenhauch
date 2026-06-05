//! Broadphase-Kollision (PHYSICS_BROAD_*) fuer viele Kreis-Entities.
//!
//! Portiert aus `rust/gb_native/src/broadphase.rs` (PyO3-Helfer, den der
//! Tree-Walker nutzt) -- SELBER Algorithmus, SELBE Paar-Reihenfolge, damit
//! gbrt bit-identische Paare/Reihenfolge liefert wie der Tree-Walker (egal ob
//! der dort das native `gb_native` oder den O(n^2)-Python-Fallback verwendet).
//!
//! Uniform-Grid mit Zellgroesse = 2 * max(radius): jede Entity liegt in genau
//! EINER Zelle, ueberlappende Kreise sind in derselben oder einer Nachbarzelle
//! -> 3x3-Nachbarschaft genuegt, O(n) statt O(n^2). Exakter Kreis-Test
//! (Abstand^2 < (r_i + r_j)^2, strikt `<`). Paar-Reihenfolge: pro i aufsteigende
//! j (j > i).

use std::collections::HashMap;

pub struct BroadPhase {
    xs: Vec<f64>,
    ys: Vec<f64>,
    rs: Vec<f64>,
    pairs: Vec<(i64, i64)>,
}

impl BroadPhase {
    pub fn new() -> Self {
        BroadPhase { xs: Vec::new(), ys: Vec::new(), rs: Vec::new(), pairs: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.xs.clear();
        self.ys.clear();
        self.rs.clear();
        self.pairs.clear();
    }

    /// Fuegt einen Kreis hinzu, liefert seinen 0-basierten Index.
    pub fn add(&mut self, x: f64, y: f64, r: f64) -> i64 {
        self.xs.push(x);
        self.ys.push(y);
        self.rs.push(r);
        (self.xs.len() - 1) as i64
    }

    pub fn count(&self) -> i64 {
        self.xs.len() as i64
    }

    /// Berechnet alle ueberlappenden Paare, speichert sie und liefert die Anzahl.
    pub fn query(&mut self) -> i64 {
        self.pairs.clear();
        let n = self.xs.len();
        if n < 2 {
            return 0;
        }
        let mut max_r = 0.0f64;
        for &r in &self.rs {
            if r > max_r {
                max_r = r;
            }
        }
        if max_r <= 0.0 {
            // Ohne positiven Radius keine Ueberlappung (strikt `<`).
            return 0;
        }
        let cs = 2.0 * max_r;
        let cell = |x: f64, y: f64| -> (i64, i64) {
            ((x / cs).floor() as i64, (y / cs).floor() as i64)
        };

        let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for i in 0..n {
            grid.entry(cell(self.xs[i], self.ys[i])).or_default().push(i);
        }

        let mut cands: Vec<usize> = Vec::new();
        for i in 0..n {
            let (cx, cy) = cell(self.xs[i], self.ys[i]);
            cands.clear();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if let Some(v) = grid.get(&(cx + dx, cy + dy)) {
                        for &j in v {
                            if j > i {
                                cands.push(j);
                            }
                        }
                    }
                }
            }
            cands.sort_unstable();
            for &j in &cands {
                let ddx = self.xs[i] - self.xs[j];
                let ddy = self.ys[i] - self.ys[j];
                let rsum = self.rs[i] + self.rs[j];
                if ddx * ddx + ddy * ddy < rsum * rsum {
                    self.pairs.push((i as i64, j as i64));
                }
            }
        }
        self.pairs.len() as i64
    }

    pub fn pair_count(&self) -> i64 {
        self.pairs.len() as i64
    }
    pub fn pair_a(&self, i: usize) -> i64 {
        self.pairs[i].0
    }
    pub fn pair_b(&self, i: usize) -> i64 {
        self.pairs[i].1
    }
}
