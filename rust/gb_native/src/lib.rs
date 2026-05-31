//! Native Beschleuniger fuer GameBasic.
//!
//! Ein einziges Extension-Modul `gb_native`, das mehrere geschlossene,
//! datenintensive Algorithmen buendelt (so wird PyO3 nur einmal kompiliert):
//!
//!   - `AStarGrid`         -> Modul `astar`        (Pathfinding)
//!   - `BroadPhase`        -> Modul `physics`      (O(n)-Broadphase-Kollision)
//!   - `TileCollider`      -> Modul `tile_collide` (Box-vs-Tilemap-Sweep)
//!   - `tilemap_flood_fill`-> Modul `tiled`        (Bulk-Flood-Fill)
//!
//! Jede Klasse spiegelt exakt die Semantik ihres Python-Fallbacks; die
//! GameBasic-Validierung (Bounds, Typchecks, Fehlermeldungen) liegt in den
//! `@builtin`-Wrappern, nicht hier. Diese Klassen sind "dumme", schnelle
//! Container + Kernels und werfen keine GameBasic-Fehler.

use pyo3::prelude::*;

mod astar;
mod broadphase;
mod tilemap;
mod tilesweep;

#[pymodule]
fn gb_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<astar::AStarGrid>()?;
    m.add_class::<broadphase::BroadPhase>()?;
    m.add_class::<tilesweep::TileCollider>()?;
    m.add_function(wrap_pyfunction!(tilemap::tilemap_flood_fill, m)?)?;
    Ok(())
}
