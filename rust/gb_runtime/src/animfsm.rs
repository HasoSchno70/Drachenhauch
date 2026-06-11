//! Modul `animfsm`: datengetriebene Animations-State-Machine (Unity-Mecanim-Stil).
//!
//! Lädt eine `.gbanim`-JSON (vom `gbanim`-Editor erzeugt): **States** (je an eine
//! Sprite-Animation gebunden, optional mit Frame-Range), benannte **Parameter**
//! (bool/float/int/trigger) und **Transitions** mit Bedingungen. Pro Frame setzt
//! das Spiel die Parameter (`ANIM_FSM_SET_*`/`TRIGGER`) und ruft
//! `ANIM_FSM_UPDATE(fsm, sprite, dt)` -- die FSM schreibt die aktuelle Animation
//! voran, wertet die Transitions aus und spielt bei Zustandswechsel die passende
//! Animation auf dem SPRITE. Reiner Logik-Code (kein Grafik-State) -> in
//! builtins.rs dispatchbar, konsolen-Golden-testbar.

use crate::value::SpriteObj;
use std::collections::HashMap;

type J = serde_json::Value;

#[derive(Clone, Copy, PartialEq)]
pub enum ParamType {
    Bool,
    Float,
    Int,
    Trigger,
}

#[derive(Clone)]
pub struct Param {
    pub ptype: ParamType,
    pub b: bool,
    pub f: f64,
    pub i: i64,
    pub triggered: bool,
}

impl Param {
    /// Numerischer Wert für Vergleichs-Operatoren (bool/trigger -> 0/1).
    fn num(&self) -> f64 {
        match self.ptype {
            ParamType::Bool => {
                if self.b {
                    1.0
                } else {
                    0.0
                }
            }
            ParamType::Float => self.f,
            ParamType::Int => self.i as f64,
            ParamType::Trigger => {
                if self.triggered {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct AnimState {
    pub name: String,
    pub anim: String,                  // Sprite-Animationsname
    pub looping: bool,                 // loop vs. einmal (one-shot)
    pub frames: Option<(i32, i32, f64)>, // optional: first,last,fps für ANIM_FSM_SETUP
}

#[derive(Clone, Copy)]
pub enum CondOp {
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
    IsTrue,
    IsFalse,
    Trigger,
}

#[derive(Clone)]
pub struct Condition {
    pub param: String, // lowercase
    pub op: CondOp,
    pub value: f64,
}

#[derive(Clone)]
pub struct Transition {
    pub from: String, // "*" = Any State
    pub to: String,
    pub wait_finished: bool, // erst wechseln, wenn die (one-shot) Animation fertig ist
    pub conditions: Vec<Condition>,
}

pub struct AnimFsmObj {
    pub params: HashMap<String, Param>, // key = lowercase
    pub states: Vec<AnimState>,
    pub transitions: Vec<Transition>,
    pub current: String,
}

impl AnimFsmObj {
    /// Aus geparstem `.gbanim`-JSON aufbauen (validiert States/Transitions/Params).
    pub fn from_json(root: &J) -> Result<Self, String> {
        let obj = root
            .as_object()
            .ok_or("ANIM_FSM_LOAD: JSON muss ein Object sein")?;

        // --- Parameter ---
        let mut params: HashMap<String, Param> = HashMap::new();
        if let Some(parr) = obj.get("params") {
            let arr = parr
                .as_array()
                .ok_or("ANIM_FSM_LOAD: 'params' muss ein Array sein")?;
            for p in arr {
                let name = p
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or("ANIM_FSM_LOAD: Parameter ohne 'name'")?;
                let tname = p.get("type").and_then(|v| v.as_str()).unwrap_or("float");
                let ptype = match tname {
                    "bool" | "boolean" => ParamType::Bool,
                    "float" | "num" => ParamType::Float,
                    "int" | "integer" => ParamType::Int,
                    "trigger" => ParamType::Trigger,
                    other => {
                        return Err(format!(
                            "ANIM_FSM_LOAD: unbekannter Parameter-Typ '{}'",
                            other
                        ))
                    }
                };
                let dv = p.get("default");
                params.insert(
                    name.to_lowercase(),
                    Param {
                        ptype,
                        b: dv.and_then(|v| v.as_bool()).unwrap_or(false),
                        f: dv.and_then(|v| v.as_f64()).unwrap_or(0.0),
                        i: dv.and_then(|v| v.as_i64()).unwrap_or(0),
                        triggered: false,
                    },
                );
            }
        }

        // --- States ---
        let sarr = obj
            .get("states")
            .and_then(|v| v.as_array())
            .ok_or("ANIM_FSM_LOAD: 'states' (Array) fehlt")?;
        if sarr.is_empty() {
            return Err("ANIM_FSM_LOAD: mindestens ein State noetig".into());
        }
        let mut states: Vec<AnimState> = Vec::new();
        for s in sarr {
            let name = s
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("ANIM_FSM_LOAD: State ohne 'name'")?
                .to_string();
            let anim = s
                .get("anim")
                .and_then(|v| v.as_str())
                .unwrap_or(&name)
                .to_string();
            let looping = s.get("loop").and_then(|v| v.as_bool()).unwrap_or(true);
            let frames = match (
                s.get("first").and_then(|v| v.as_i64()),
                s.get("last").and_then(|v| v.as_i64()),
            ) {
                (Some(f), Some(l)) => {
                    let fps = s.get("fps").and_then(|v| v.as_f64()).unwrap_or(8.0);
                    Some((f as i32, l as i32, fps))
                }
                _ => None,
            };
            if states.iter().any(|st| st.name == name) {
                return Err(format!("ANIM_FSM_LOAD: doppelter State '{}'", name));
            }
            states.push(AnimState {
                name,
                anim,
                looping,
                frames,
            });
        }

        // --- Default-State ---
        let default = obj
            .get("default")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| states[0].name.clone());
        if !states.iter().any(|s| s.name == default) {
            return Err(format!(
                "ANIM_FSM_LOAD: default-State '{}' existiert nicht",
                default
            ));
        }

        // --- Transitions ---
        let mut transitions: Vec<Transition> = Vec::new();
        if let Some(tarr) = obj.get("transitions") {
            let arr = tarr
                .as_array()
                .ok_or("ANIM_FSM_LOAD: 'transitions' muss ein Array sein")?;
            for t in arr {
                let from = t.get("from").and_then(|v| v.as_str()).unwrap_or("*").to_string();
                let to = t
                    .get("to")
                    .and_then(|v| v.as_str())
                    .ok_or("ANIM_FSM_LOAD: Transition ohne 'to'")?
                    .to_string();
                if !states.iter().any(|s| s.name == to) {
                    return Err(format!(
                        "ANIM_FSM_LOAD: Transition zu unbekanntem State '{}'",
                        to
                    ));
                }
                if from != "*" && !states.iter().any(|s| s.name == from) {
                    return Err(format!(
                        "ANIM_FSM_LOAD: Transition von unbekanntem State '{}'",
                        from
                    ));
                }
                let wait_finished = t
                    .get("wait_finished")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let mut conditions: Vec<Condition> = Vec::new();
                if let Some(carr) = t.get("conditions").and_then(|v| v.as_array()) {
                    for c in carr {
                        let param = c
                            .get("param")
                            .and_then(|v| v.as_str())
                            .ok_or("ANIM_FSM_LOAD: Condition ohne 'param'")?
                            .to_lowercase();
                        if !params.contains_key(&param) {
                            return Err(format!(
                                "ANIM_FSM_LOAD: Condition referenziert unbekannten Parameter '{}'",
                                param
                            ));
                        }
                        let opname = c.get("op").and_then(|v| v.as_str()).unwrap_or("trigger");
                        let op = match opname {
                            "gt" | ">" => CondOp::Gt,
                            "lt" | "<" => CondOp::Lt,
                            "ge" | ">=" => CondOp::Ge,
                            "le" | "<=" => CondOp::Le,
                            "eq" | "=" | "==" => CondOp::Eq,
                            "ne" | "<>" | "!=" => CondOp::Ne,
                            "true" | "is_true" => CondOp::IsTrue,
                            "false" | "is_false" => CondOp::IsFalse,
                            "trigger" => CondOp::Trigger,
                            other => {
                                return Err(format!(
                                    "ANIM_FSM_LOAD: unbekannter Operator '{}'",
                                    other
                                ))
                            }
                        };
                        let value = c.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        conditions.push(Condition { param, op, value });
                    }
                }
                transitions.push(Transition {
                    from,
                    to,
                    wait_finished,
                    conditions,
                });
            }
        }

        Ok(AnimFsmObj {
            params,
            states,
            transitions,
            current: default,
        })
    }

    fn state(&self, name: &str) -> Option<&AnimState> {
        self.states.iter().find(|s| s.name == name)
    }

    fn param_mut(&mut self, name: &str) -> Result<&mut Param, String> {
        self.params
            .get_mut(&name.to_lowercase())
            .ok_or_else(|| format!("ANIM_FSM: unbekannter Parameter '{}'", name))
    }

    pub fn set_bool(&mut self, name: &str, v: bool) -> Result<(), String> {
        self.param_mut(name)?.b = v;
        Ok(())
    }

    pub fn set_float(&mut self, name: &str, v: f64) -> Result<(), String> {
        self.param_mut(name)?.f = v;
        Ok(())
    }

    pub fn set_int(&mut self, name: &str, v: i64) -> Result<(), String> {
        self.param_mut(name)?.i = v;
        Ok(())
    }

    pub fn trigger(&mut self, name: &str) -> Result<(), String> {
        self.param_mut(name)?.triggered = true;
        Ok(())
    }

    pub fn get_num(&self, name: &str) -> Result<f64, String> {
        self.params
            .get(&name.to_lowercase())
            .map(|p| p.num())
            .ok_or_else(|| format!("ANIM_FSM: unbekannter Parameter '{}'", name))
    }

    /// Registriert die Frame-Ranges aller States als Animationen auf dem Sprite
    /// und stellt den Default-State ein. Bequeme Komplett-Einrichtung, damit der
    /// User nicht jede `SPRITE_ADD_ANIM` von Hand schreiben muss.
    pub fn setup(&self, sp: &mut SpriteObj) -> Result<(), String> {
        for s in &self.states {
            if let Some((first, last, fps)) = s.frames {
                sp.anims.insert(s.anim.clone(), (first, last, fps));
            }
        }
        if let Some(st) = self.state(&self.current) {
            if sp.anims.contains_key(&st.anim) {
                sp.play(&st.anim, st.looping)?;
            }
        }
        Ok(())
    }

    /// Pro Frame: Animation voranschreiben, Transitions auswerten, bei Wechsel die
    /// neue Animation spielen. Liefert TRUE, wenn der State gewechselt hat.
    pub fn update(&mut self, sp: &mut SpriteObj, dt_ms: i64) -> Result<bool, String> {
        // 1. aktuelle Animation voranschreiben (setzt sp.finished für wait_finished)
        sp.update(dt_ms);

        // 2. Transitions in Datei-Reihenfolge prüfen (from == current oder "*")
        let mut target: Option<String> = None;
        for t in &self.transitions {
            if t.from != "*" && t.from != self.current {
                continue;
            }
            if t.to == self.current {
                continue; // kein Selbst-Übergang
            }
            if t.wait_finished && !sp.finished {
                continue;
            }
            if self.conds_pass(&t.conditions) {
                target = Some(t.to.clone());
                break;
            }
        }

        let mut changed = false;
        if let Some(to) = target {
            if let Some(st) = self.state(&to) {
                let (anim, looping) = (st.anim.clone(), st.looping);
                self.current = to;
                if sp.anims.contains_key(&anim) {
                    sp.play(&anim, looping)?;
                }
                changed = true;
            }
        }

        // 3. Trigger zurücksetzen (einmal pro UPDATE konsumiert)
        for p in self.params.values_mut() {
            if p.ptype == ParamType::Trigger {
                p.triggered = false;
            }
        }
        Ok(changed)
    }

    fn conds_pass(&self, conds: &[Condition]) -> bool {
        conds.iter().all(|c| {
            let p = self.params.get(&c.param);
            let pv = p.map(|p| p.num()).unwrap_or(0.0);
            match c.op {
                CondOp::Gt => pv > c.value,
                CondOp::Lt => pv < c.value,
                CondOp::Ge => pv >= c.value,
                CondOp::Le => pv <= c.value,
                CondOp::Eq => (pv - c.value).abs() < 1e-9,
                CondOp::Ne => (pv - c.value).abs() >= 1e-9,
                CondOp::IsTrue => pv != 0.0,
                CondOp::IsFalse => pv == 0.0,
                CondOp::Trigger => p.map(|p| p.triggered).unwrap_or(false),
            }
        })
    }

    /// Erzwingt einen State (ohne Bedingungen) -- z.B. für Reset/Respawn.
    pub fn force(&mut self, sp: &mut SpriteObj, state: &str) -> Result<bool, String> {
        let st = self
            .state(state)
            .ok_or_else(|| format!("ANIM_FSM_FORCE: unbekannter State '{}'", state))?;
        let (anim, looping) = (st.anim.clone(), st.looping);
        let changed = self.current != state;
        self.current = state.to_string();
        if sp.anims.contains_key(&anim) {
            sp.play(&anim, looping)?;
        }
        Ok(changed)
    }
}
