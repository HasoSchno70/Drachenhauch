//! Modul `physics2d`: echte 2D-Starrkoerper-Physik via Rapier2D.
//!
//! Vollwertiger 2D-Solver (Schwerkraft, Integration, Kollisionsaufloesung,
//! Restitution/Reibung) -- das 2D-Pendant zu `physics3d`. Im Gegensatz zum
//! `physics`-Modul (reine Kollisions-Helfer) simuliert dies echte Dynamik.
//!
//! **Bildschirm-Konvention:** Y waechst nach UNTEN (wie bei allen GB-Draw-
//! Befehlen), Default-Schwerkraft zieht also nach unten (positives Y). Die
//! Solver-`length_unit` ist auf 100 gesetzt (1 "Meter" = 100 px), damit
//! pixel-grosse Welten stabil bleiben -- Default-Gravitation 980 px/s^2 ist
//! dann ~Erdbeschleunigung. Eigene Einheiten via `PHYS2D_SET_GRAVITY`.
//!
//! Koerper werden ueber einen stabilen Integer-Index angesprochen (auch nach
//! `remove` bleiben kleinere Indizes gueltig -- Tombstone-Slots).

use rapier2d::prelude::*;

pub struct Phys2dWorld {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    gravity: Vector<Real>,
    params: IntegrationParameters,
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad: DefaultBroadPhase,
    narrow: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd: CCDSolver,
    query: QueryPipeline,
    handles: Vec<Option<RigidBodyHandle>>,
}

impl Phys2dWorld {
    pub fn new() -> Self {
        let mut params = IntegrationParameters::default();
        // 1 "Meter" = 100 px -> stabiler Solver bei pixel-grossen Welten.
        params.length_unit = 100.0;
        Phys2dWorld {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            gravity: Vector::new(0.0, 980.0), // +Y = unten (Bildschirm)
            params,
            pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad: DefaultBroadPhase::new(),
            narrow: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd: CCDSolver::new(),
            query: QueryPipeline::new(),
            handles: Vec::new(),
        }
    }

    pub fn set_gravity(&mut self, x: f32, y: f32) {
        self.gravity = Vector::new(x, y);
    }

    fn insert_body(&mut self, rb: RigidBody, collider: Collider) -> i64 {
        let h = self.bodies.insert(rb);
        self.colliders.insert_with_parent(collider, h, &mut self.bodies);
        self.handles.push(Some(h));
        (self.handles.len() - 1) as i64
    }

    /// Rechteck. `hw`/`hh` = HALBE Breite/Hoehe (wie Rapier-Cuboid).
    pub fn add_box(&mut self, x: f32, y: f32, hw: f32, hh: f32,
                   dynamic: bool, bounce: f32) -> i64 {
        let rb = if dynamic { RigidBodyBuilder::dynamic() } else { RigidBodyBuilder::fixed() }
            .translation(Vector::new(x, y)).build();
        let collider = ColliderBuilder::cuboid(hw.max(0.001), hh.max(0.001))
            .restitution(bounce.clamp(0.0, 1.0))
            .friction(0.6)
            .build();
        self.insert_body(rb, collider)
    }

    pub fn add_circle(&mut self, x: f32, y: f32, r: f32,
                      dynamic: bool, bounce: f32) -> i64 {
        let rb = if dynamic { RigidBodyBuilder::dynamic() } else { RigidBodyBuilder::fixed() }
            .translation(Vector::new(x, y)).build();
        let collider = ColliderBuilder::ball(r.max(0.001))
            .restitution(bounce.clamp(0.0, 1.0))
            .friction(0.6)
            .build();
        self.insert_body(rb, collider)
    }

    pub fn step(&mut self, dt: f32) {
        self.params.dt = dt.clamp(0.0001, 0.05);
        self.pipeline.step(
            &self.gravity, &self.params, &mut self.islands,
            &mut self.broad, &mut self.narrow,
            &mut self.bodies, &mut self.colliders,
            &mut self.impulse_joints, &mut self.multibody_joints,
            &mut self.ccd, Some(&mut self.query), &(), &(),
        );
    }

    fn handle(&self, idx: i64) -> Option<RigidBodyHandle> {
        let i = idx as usize;
        if idx < 0 || i >= self.handles.len() { return None; }
        self.handles[i]
    }

    /// Position (x,y), oder (0,0) bei ungueltigem Index.
    pub fn pos(&self, idx: i64) -> (f32, f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get(h) {
                let t = rb.translation();
                return (t.x, t.y);
            }
        }
        (0.0, 0.0)
    }

    /// Rotationswinkel in Radiant (fuer Sprite-Rotation).
    pub fn angle(&self, idx: i64) -> f32 {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get(h) {
                return rb.rotation().angle();
            }
        }
        0.0
    }

    /// Lineare Geschwindigkeit (vx,vy).
    pub fn vel(&self, idx: i64) -> (f32, f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get(h) {
                let v = rb.linvel();
                return (v.x, v.y);
            }
        }
        (0.0, 0.0)
    }

    pub fn set_vel(&mut self, idx: i64, vx: f32, vy: f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.set_linvel(Vector::new(vx, vy), true);
            }
        }
    }

    pub fn apply_impulse(&mut self, idx: i64, ix: f32, iy: f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.apply_impulse(Vector::new(ix, iy), true);
            }
        }
    }

    pub fn set_pos(&mut self, idx: i64, x: f32, y: f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.set_translation(Vector::new(x, y), true);
            }
        }
    }

    /// Rotation sperren/freigeben -- z.B. damit eine Spielfigur nicht umkippt.
    pub fn lock_rotation(&mut self, idx: i64, locked: bool) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.lock_rotations(locked, true);
            }
        }
    }

    /// Zwischen statisch und dynamisch umschalten.
    ///
    /// Der klassische Fall ist ein Aufbau, der erst STEHEN und dann
    /// zusammenfallen soll: eine Mauer, ein Logo, ein Turm. Ohne diese
    /// Moeglichkeit muss man die Koerper entfernen und an derselben Stelle neu
    /// anlegen -- was Geschwindigkeit, Drehung und Kontakte verliert.
    ///
    /// `wake` weckt den Koerper mit auf: Rapier laesst ruhende Koerper
    /// schlafen, und ein frisch dynamisch gemachter Klotz wuerde sonst
    /// reglos in der Luft haengen, bis ihn etwas anstoesst.
    pub fn set_dynamic(&mut self, idx: i64, dynamic: bool) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.set_body_type(
                    if dynamic { RigidBodyType::Dynamic } else { RigidBodyType::Fixed },
                    true,
                );
            }
        }
    }

    /// Ist der Koerper dynamisch? (Statische zaehlen als FALSE.)
    pub fn is_dynamic(&self, idx: i64) -> bool {
        self.handle(idx)
            .and_then(|h| self.bodies.get(h))
            .map(|rb| rb.body_type() == RigidBodyType::Dynamic)
            .unwrap_or(false)
    }

    pub fn remove(&mut self, idx: i64) {
        let i = idx as usize;
        if idx < 0 || i >= self.handles.len() { return; }
        if let Some(h) = self.handles[i].take() {
            self.bodies.remove(h, &mut self.islands, &mut self.colliders,
                               &mut self.impulse_joints, &mut self.multibody_joints, true);
        }
    }

    pub fn count(&self) -> i64 {
        self.handles.iter().filter(|h| h.is_some()).count() as i64
    }
}
