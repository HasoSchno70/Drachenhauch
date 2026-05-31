//! Bulk-Tilemap-Operation: Flood-Fill (4-verbunden).
//!
//! Einzige Tilemap-Op mit lohnender Rust-Beschleunigung: die Pro-Element-
//! Arbeit (Stack-basiertes BFS mit Nachbar-Checks) ist schwer genug, dass
//! Rust auch nach Abzug des O(n)-Boxings an der Python-Listen-Grenze klar
//! gewinnt -- wichtig fuer grosse prozedural generierte Karten.
//!
//! Andere Bulk-Ops (fill_rect, replace, count) bleiben in Python: sie sind
//! entweder In-place ohne Vollkopie (fill_rect) oder bereits C-schnell
//! (list.count), sodass die FFI-Kopie sie nicht schneller machen wuerde.
//!
//! Signatur: (tiles, width, height, tx, ty, new_gid) -> (neue_tiles, count).
//! `tiles` ist row-major. Ersetzt die zusammenhaengende Region der GID an
//! (tx, ty) durch new_gid; liefert die geaenderte Liste + Anzahl Tiles.

use pyo3::prelude::*;

#[pyfunction]
pub fn tilemap_flood_fill(
    mut tiles: Vec<i64>,
    width: i64,
    height: i64,
    tx: i64,
    ty: i64,
    new_gid: i64,
) -> (Vec<i64>, usize) {
    if tx < 0 || ty < 0 || tx >= width || ty >= height {
        return (tiles, 0);
    }
    let start = (ty * width + tx) as usize;
    let target = tiles[start];
    if target == new_gid {
        return (tiles, 0);
    }

    let mut stack: Vec<(i64, i64)> = vec![(tx, ty)];
    let mut count: usize = 0;
    while let Some((cx, cy)) = stack.pop() {
        if cx < 0 || cy < 0 || cx >= width || cy >= height {
            continue;
        }
        let idx = (cy * width + cx) as usize;
        if tiles[idx] != target {
            continue; // bereits gefuellt (== new_gid) oder andere GID
        }
        tiles[idx] = new_gid;
        count += 1;
        stack.push((cx + 1, cy));
        stack.push((cx - 1, cy));
        stack.push((cx, cy + 1));
        stack.push((cx, cy - 1));
    }
    (tiles, count)
}
