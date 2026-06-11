//! Entity-Component-System fuer das `ecs`-Modul. Sparse-Set pro Component,
//! Query = Intersection. Queries liefern aufsteigend sortierte Entity-IDs --
//! dokumentiertes Verhalten (stammt aus der frueheren Python-Implementierung,
//! GB-Programme duerfen sich darauf verlassen).

use std::collections::{HashMap, HashSet};

use crate::value::{as_f64, Value};

/// Sparse-Set: dense (Entity-IDs) + values (parallel) + sparse (ent->index).
#[derive(Default)]
pub struct Component {
    pub dense: Vec<i64>,
    pub values: Vec<Value>,
    pub sparse: HashMap<i64, usize>,
}

impl Component {
    fn has(&self, ent: i64) -> bool { self.sparse.contains_key(&ent) }
    fn get(&self, ent: i64) -> Option<Value> { self.sparse.get(&ent).map(|&i| self.values[i].clone()) }
    fn set(&mut self, ent: i64, value: Value) {
        if let Some(&i) = self.sparse.get(&ent) {
            self.values[i] = value;
        } else {
            self.sparse.insert(ent, self.dense.len());
            self.dense.push(ent);
            self.values.push(value);
        }
    }
    fn remove(&mut self, ent: i64) -> bool {
        let i = match self.sparse.remove(&ent) { Some(i) => i, None => return false };
        let last = self.dense.len() - 1;
        if i != last {
            let moved = self.dense[last];
            self.dense[i] = moved;
            self.values[i] = self.values[last].clone();
            self.sparse.insert(moved, i);
        }
        self.dense.pop();
        self.values.pop();
        true
    }
}

pub struct World {
    next_id: i64,
    entities: HashSet<i64>,
    components: HashMap<String, Component>,
}

type R<T> = Result<T, String>;

impl World {
    pub fn new() -> Self { World { next_id: 1, entities: HashSet::new(), components: HashMap::new() } }

    pub fn new_entity(&mut self) -> i64 {
        let eid = self.next_id;
        self.next_id += 1;
        self.entities.insert(eid);
        eid
    }
    pub fn destroy(&mut self, ent: i64) -> bool {
        if !self.entities.remove(&ent) { return false; }
        for c in self.components.values_mut() { c.remove(ent); }
        true
    }
    pub fn alive(&self, ent: i64) -> bool { self.entities.contains(&ent) }
    pub fn count(&self) -> usize { self.entities.len() }

    fn check_ent(&self, ent: i64, fn_: &str) -> R<()> {
        if !self.entities.contains(&ent) { return Err(format!("{}: Entity {} nicht in World", fn_, ent)); }
        Ok(())
    }

    pub fn set(&mut self, name: &str, ent: i64, value: Value, fn_: &str) -> R<()> {
        self.check_ent(ent, fn_)?;
        self.components.entry(name.to_string()).or_default().set(ent, value);
        Ok(())
    }

    pub fn get(&self, ent: i64, name: &str, fn_: &str) -> R<Value> {
        self.components.get(name).and_then(|c| c.get(ent))
            .ok_or_else(|| format!("{}: Component '{}' fehlt bei Entity {}", fn_, name, ent))
    }
    pub fn get_or(&self, ent: i64, name: &str) -> Option<Value> {
        self.components.get(name).and_then(|c| c.get(ent))
    }
    pub fn has_component(&self, ent: i64, name: &str) -> bool {
        self.entities.contains(&ent) && self.components.get(name).map(|c| c.has(ent)).unwrap_or(false)
    }
    pub fn remove_component(&mut self, ent: i64, name: &str) -> bool {
        self.components.get_mut(name).map(|c| c.remove(ent)).unwrap_or(false)
    }

    /// Intersection aller genannten Components, aufsteigend sortiert.
    pub fn query(&self, names: &[&str]) -> Vec<i64> {
        if names.is_empty() { return vec![]; }
        let mut comps = Vec::with_capacity(names.len());
        for name in names {
            match self.components.get(*name) {
                Some(c) if !c.dense.is_empty() => comps.push(c),
                _ => return vec![],
            }
        }
        // Kleinster Component als Basis (Performance; Resultat ist identisch).
        comps.sort_by_key(|c| c.dense.len());
        let base = comps[0];
        let others = &comps[1..];
        let mut out: Vec<i64> = base.dense.iter().copied()
            .filter(|&ent| others.iter().all(|c| c.has(ent)))
            .collect();
        out.sort();
        out
    }

    pub fn integrate_float(&mut self, target: &str, delta: &str) -> i64 {
        if !self.components.contains_key(target) || !self.components.contains_key(delta) { return 0; }
        // Ueber das kleinere Dense-Set iterieren (weniger Lookups).
        let (t_len, d_len) = (self.components[target].dense.len(), self.components[delta].dense.len());
        let mut count = 0i64;
        // Paare (ent, delta_value) sammeln, dann auf target anwenden -- vermeidet
        // gleichzeitige &mut/&-Borrows auf self.components.
        let pairs: Vec<(i64, f64)> = if d_len <= t_len {
            let dc = &self.components[delta];
            dc.dense.iter().zip(dc.values.iter()).map(|(&e, v)| (e, as_f64(v))).collect()
        } else {
            let tc = &self.components[target];
            tc.dense.iter().map(|&e| e).collect::<Vec<_>>().iter()
                .filter_map(|&e| self.components[delta].get(e).map(|v| (e, as_f64(&v)))).collect()
        };
        let tc = self.components.get_mut(target).unwrap();
        for (ent, dv) in pairs {
            if let Some(&i) = tc.sparse.get(&ent) {
                tc.values[i] = Value::Float(as_f64(&tc.values[i]) + dv);
                count += 1;
            }
        }
        count
    }
    pub fn integrate_int(&mut self, target: &str, delta: &str) -> i64 {
        if !self.components.contains_key(target) || !self.components.contains_key(delta) { return 0; }
        let pairs: Vec<(i64, i64)> = {
            let dc = &self.components[delta];
            dc.dense.iter().zip(dc.values.iter()).map(|(&e, v)| (e, as_f64(v) as i64)).collect()
        };
        let tc = self.components.get_mut(target).unwrap();
        let mut count = 0i64;
        for (ent, dv) in pairs {
            if let Some(&i) = tc.sparse.get(&ent) {
                tc.values[i] = Value::Int(as_f64(&tc.values[i]) as i64 + dv);
                count += 1;
            }
        }
        count
    }
    pub fn scale_float(&mut self, target: &str, factor: f64) -> i64 {
        match self.components.get_mut(target) {
            Some(c) => { for v in c.values.iter_mut() { *v = Value::Float(as_f64(v) * factor); } c.values.len() as i64 }
            None => 0,
        }
    }
    pub fn fill_float(&mut self, target: &str, value: f64) -> i64 {
        match self.components.get_mut(target) {
            Some(c) => { for v in c.values.iter_mut() { *v = Value::Float(value); } c.values.len() as i64 }
            None => 0,
        }
    }
    pub fn fill_int(&mut self, target: &str, value: i64) -> i64 {
        match self.components.get_mut(target) {
            Some(c) => { for v in c.values.iter_mut() { *v = Value::Int(value); } c.values.len() as i64 }
            None => 0,
        }
    }
    pub fn clamp_float(&mut self, target: &str, lo: f64, hi: f64) -> R<i64> {
        if lo > hi { return Err(format!("ECS_CLAMP_FLOAT: lo ({}) > hi ({})", lo, hi)); }
        match self.components.get_mut(target) {
            Some(c) => {
                for v in c.values.iter_mut() {
                    let f = as_f64(v);
                    if f < lo { *v = Value::Float(lo); } else if f > hi { *v = Value::Float(hi); }
                }
                Ok(c.values.len() as i64)
            }
            None => Ok(0),
        }
    }
    pub fn remove_dead(&mut self, name: &str, threshold: f64) -> i64 {
        let to_kill: Vec<i64> = match self.components.get(name) {
            Some(c) => c.dense.iter().zip(c.values.iter()).filter(|(_, v)| as_f64(v) <= threshold).map(|(&e, _)| e).collect(),
            None => return 0,
        };
        let n = to_kill.len() as i64;
        for ent in to_kill { self.destroy(ent); }
        n
    }
    pub fn count_with(&self, name: &str) -> i64 {
        self.components.get(name).map(|c| c.dense.len() as i64).unwrap_or(0)
    }
}
