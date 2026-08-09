//! A*-Pathfinding fuer das `astar`-Modul. Portiert aus
//! `rust/gb_native/src/astar.rs` (PyO3-Helper) -- selber Algorithmus, selber
//! counter-FIFO-Tie-Break, damit bei mehreren optimalen Pfaden derselbe wie
//! in der Python-Referenz (`astar.py::_AStarGrid`) herauskommt.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, Copy)]
enum Heuristic {
    Manhattan,
    Euclid,
    Chebyshev,
}

#[inline]
fn heuristic_value(h: Heuristic, x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    match h {
        Heuristic::Manhattan => ((x1 - x2).abs() + (y1 - y2).abs()) as f64,
        Heuristic::Euclid => {
            let dx = (x1 - x2) as f64;
            let dy = (y1 - y2) as f64;
            (dx * dx + dy * dy).sqrt()
        }
        Heuristic::Chebyshev => (x1 - x2).abs().max((y1 - y2).abs()) as f64,
    }
}

// Min-Heap-Eintrag, sortiert nach (f, counter). counter ist eindeutig, also
// werden x/y nie verglichen (wie Pythons Tupel (f, counter, x, y)).
struct Entry { f: f64, counter: u64, x: i32, y: i32 }
impl PartialEq for Entry { fn eq(&self, o: &Self) -> bool { self.f == o.f && self.counter == o.counter } }
impl Eq for Entry {}
impl Ord for Entry {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering { self.f.total_cmp(&o.f).then(self.counter.cmp(&o.counter)) }
}
impl PartialOrd for Entry { fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(o)) } }

const ORTHO: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const DIAG: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

pub struct AStarGrid {
    pub w: i32,
    pub h: i32,
    walls: Vec<bool>,
    diagonal: bool,
    heuristic: Heuristic,
    diagonal_cost: f64,
    path: Vec<(i32, i32)>,
    path_cost: f64,
}

impl AStarGrid {
    #[inline]
    fn idx(&self, x: i32, y: i32) -> usize { (y * self.w + x) as usize }
    pub fn in_bounds(&self, x: i32, y: i32) -> bool { x >= 0 && x < self.w && y >= 0 && y < self.h }

    pub fn new(w: i32, h: i32) -> Self {
        AStarGrid {
            w, h,
            walls: vec![false; (w * h) as usize],
            diagonal: false,
            heuristic: Heuristic::Manhattan,
            diagonal_cost: std::f64::consts::SQRT_2,
            path: Vec::new(),
            path_cost: 0.0,
        }
    }

    pub fn set_wall(&mut self, x: i32, y: i32) { let i = self.idx(x, y); self.walls[i] = true; }
    pub fn set_passable(&mut self, x: i32, y: i32) { let i = self.idx(x, y); self.walls[i] = false; }
    pub fn is_wall(&self, x: i32, y: i32) -> bool { self.walls[self.idx(x, y)] }
    pub fn clear(&mut self) { for w in self.walls.iter_mut() { *w = false; } self.path.clear(); self.path_cost = 0.0; }
    pub fn set_diagonal(&mut self, allow: bool) { self.diagonal = allow; }
    pub fn set_heuristic(&mut self, key: &str) {
        self.heuristic = match key { "euclid" => Heuristic::Euclid, "chebyshev" => Heuristic::Chebyshev, _ => Heuristic::Manhattan };
    }
    pub fn set_diagonal_cost(&mut self, cost: f64) { self.diagonal_cost = cost; }

    pub fn find(&mut self, sx: i32, sy: i32, ex: i32, ey: i32) -> bool {
        let w = self.w;
        let n = (self.w * self.h) as usize;
        let sidx = self.idx(sx, sy);
        let eidx = self.idx(ex, ey);
        if self.walls[sidx] || self.walls[eidx] {
            self.path = Vec::new();
            self.path_cost = 0.0;
            return false;
        }
        let mut g_score = vec![f64::INFINITY; n];
        let mut came_from = vec![-1i32; n];
        let mut closed = vec![false; n];
        let mut heap: BinaryHeap<Reverse<Entry>> = BinaryHeap::new();
        let mut counter: u64 = 0;
        let h = self.heuristic;
        let diag_cost = self.diagonal_cost;

        g_score[sidx] = 0.0;
        heap.push(Reverse(Entry { f: 0.0, counter, x: sx, y: sy }));

        while let Some(Reverse(cur)) = heap.pop() {
            let (cx, cy) = (cur.x, cur.y);
            if cx == ex && cy == ey {
                let mut path = vec![(cx, cy)];
                let mut ci = self.idx(cx, cy);
                while came_from[ci] != -1 {
                    ci = came_from[ci] as usize;
                    path.push(((ci as i32) % w, (ci as i32) / w));
                }
                path.reverse();
                self.path_cost = g_score[eidx];
                self.path = path;
                return true;
            }
            let cidx = self.idx(cx, cy);
            if closed[cidx] { continue; }
            closed[cidx] = true;
            let gc = g_score[cidx];

            for (dx, dy) in ORTHO {
                let (nx, ny) = (cx + dx, cy + dy);
                if !self.in_bounds(nx, ny) { continue; }
                let nidx = self.idx(nx, ny);
                if self.walls[nidx] { continue; }
                let tentative = gc + 1.0;
                if tentative < g_score[nidx] {
                    came_from[nidx] = cidx as i32;
                    g_score[nidx] = tentative;
                    let f = tentative + heuristic_value(h, nx, ny, ex, ey);
                    counter += 1;
                    heap.push(Reverse(Entry { f, counter, x: nx, y: ny }));
                }
            }
            if self.diagonal {
                for (dx, dy) in DIAG {
                    let (nx, ny) = (cx + dx, cy + dy);
                    if !self.in_bounds(nx, ny) { continue; }
                    let nidx = self.idx(nx, ny);
                    if self.walls[nidx] { continue; }
                    if self.walls[self.idx(cx + dx, cy)] || self.walls[self.idx(cx, cy + dy)] { continue; }
                    let tentative = gc + diag_cost;
                    if tentative < g_score[nidx] {
                        came_from[nidx] = cidx as i32;
                        g_score[nidx] = tentative;
                        let f = tentative + heuristic_value(h, nx, ny, ex, ey);
                        counter += 1;
                        heap.push(Reverse(Entry { f, counter, x: nx, y: ny }));
                    }
                }
            }
        }
        self.path = Vec::new();
        self.path_cost = 0.0;
        false
    }

    pub fn path_len(&self) -> usize { self.path.len() }
    pub fn path_x(&self, idx: usize) -> i32 { self.path[idx].0 }
    pub fn path_y(&self, idx: usize) -> i32 { self.path[idx].1 }
    pub fn path_cost(&self) -> f64 { self.path_cost }
    pub fn clear_path(&mut self) { self.path.clear(); self.path_cost = 0.0; }
}
