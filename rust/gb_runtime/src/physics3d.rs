//! Modul `physics3d`: echte 3D-Starrkoerper-Physik via Rapier3D.
//!
//! Im Gegensatz zum `physics`-Modul (reine Kollisions-Helfer + Broadphase) ist
//! dies ein vollwertiger Solver: Schwerkraft, Integration, Kollisionsaufloesung,
//! Restitution/Reibung. Eine `Phys3dWorld` haelt die komplette Rapier-Pipeline;
//! Koerper werden ueber einen stabilen Integer-Index angesprochen (auch nach
//! `remove` bleiben kleinere Indizes gueltig -- Tombstone-Slots).

use rapier3d::prelude::*;

pub struct Phys3dWorld {
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

impl Phys3dWorld {
    pub fn new() -> Self {
        Phys3dWorld {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            gravity: Vector::new(0.0, -9.81, 0.0),
            params: IntegrationParameters::default(),
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

    pub fn set_gravity(&mut self, x: f32, y: f32, z: f32) {
        self.gravity = Vector::new(x, y, z);
    }

    fn insert_body(&mut self, rb: RigidBody, collider: Collider) -> i64 {
        let h = self.bodies.insert(rb);
        self.colliders.insert_with_parent(collider, h, &mut self.bodies);
        self.handles.push(Some(h));
        (self.handles.len() - 1) as i64
    }

    pub fn add_box(&mut self, x: f32, y: f32, z: f32, hx: f32, hy: f32, hz: f32,
                   dynamic: bool, bounce: f32) -> i64 {
        let rb = if dynamic {
            RigidBodyBuilder::dynamic()
        } else {
            RigidBodyBuilder::fixed()
        }.translation(Vector::new(x, y, z)).build();
        let collider = ColliderBuilder::cuboid(hx.max(0.001), hy.max(0.001), hz.max(0.001))
            .restitution(bounce.clamp(0.0, 1.0))
            .friction(0.6)
            .build();
        self.insert_body(rb, collider)
    }

    pub fn add_sphere(&mut self, x: f32, y: f32, z: f32, r: f32,
                      dynamic: bool, bounce: f32) -> i64 {
        let rb = if dynamic {
            RigidBodyBuilder::dynamic()
        } else {
            RigidBodyBuilder::fixed()
        }.translation(Vector::new(x, y, z)).build();
        let collider = ColliderBuilder::ball(r.max(0.001))
            .restitution(bounce.clamp(0.0, 1.0))
            .friction(0.6)
            .build();
        self.insert_body(rb, collider)
    }

    pub fn step(&mut self, dt: f32) {
        // dt in sinnvollen Grenzen halten (Tunneln/Explosion vermeiden).
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

    /// Position (x,y,z) eines Koerpers, oder (0,0,0) bei ungueltigem Index.
    pub fn pos(&self, idx: i64) -> (f32, f32, f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get(h) {
                let t = rb.translation();
                return (t.x, t.y, t.z);
            }
        }
        (0.0, 0.0, 0.0)
    }

    /// Rotations-Quaternion (i,j,k,w).
    pub fn rot(&self, idx: i64) -> (f32, f32, f32, f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get(h) {
                let q = rb.rotation();
                return (q.i, q.j, q.k, q.w);
            }
        }
        (0.0, 0.0, 0.0, 1.0)
    }

    pub fn set_vel(&mut self, idx: i64, vx: f32, vy: f32, vz: f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.set_linvel(Vector::new(vx, vy, vz), true);
            }
        }
    }

    pub fn apply_impulse(&mut self, idx: i64, ix: f32, iy: f32, iz: f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.apply_impulse(Vector::new(ix, iy, iz), true);
            }
        }
    }

    pub fn set_pos(&mut self, idx: i64, x: f32, y: f32, z: f32) {
        if let Some(h) = self.handle(idx) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.set_translation(Vector::new(x, y, z), true);
            }
        }
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
