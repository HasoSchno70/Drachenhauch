//! Befehle der Spiel-Module ohne Grafik: timer (AFTER/EVERY/COOLDOWN) und scene.
//!
//! Teil von `vm.rs`: dort stehen die Typen; hier nur Methoden. Bis 2026.53
//! standen sie in vm_system.rs (Bericht D2).

use super::*;

impl<'p> Vm<'p> {
    /// Modul `timer` (timer.rs): AFTER/EVERY/CANCEL/UPDATE + COOLDOWN.
    /// Kein Grafik-Bezug -- laeuft auch konsolen-only. TIMER_UPDATE feuert
    /// faellige FUNCREF-Callbacks nach dem gleichen Muster wie GUI_UPDATE.
    pub(super) fn try_timer(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("timer_") || name == "cooldown") { return Ok(None); }
        fn t_int(a: &[Value], i: usize, fn_: &str) -> R<i64> {
            crate::argument::a_int(a, i, fn_)
        }
        fn t_str<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            crate::argument::a_text(a, i, fn_)
        }
        fn t_func(a: &[Value], i: usize, fn_: &str) -> R<crate::value::Rueckruf> {
            match a.get(i).and_then(crate::value::Rueckruf::aus_wert) {
                Some(cb) => Ok(cb),
                None => Err(format!("{}: erwartet FUNCREF (Arg {})", fn_, i + 1)),
            }
        }
        let r = match name {
            "timer_after" => {
                let ms = t_int(a, 0, "TIMER_AFTER")?;
                if ms < 0 { return Err("TIMER_AFTER: ms muss >= 0 sein".into()); }
                Value::Int(self.timers.after(ms, t_func(a, 1, "TIMER_AFTER")?))
            }
            "timer_every" => {
                let ms = t_int(a, 0, "TIMER_EVERY")?;
                if ms <= 0 { return Err("TIMER_EVERY: ms muss > 0 sein".into()); }
                Value::Int(self.timers.every(ms, t_func(a, 1, "TIMER_EVERY")?))
            }
            "timer_cancel" => { self.timers.cancel(t_int(a, 0, "TIMER_CANCEL")?); Value::Nil }
            "timer_active" => Value::Bool(self.timers.active(t_int(a, 0, "TIMER_ACTIVE")?)),
            "timer_count" => Value::Int(self.timers.count()),
            "timer_clear" => { self.timers.clear(); Value::Nil }
            "timer_update" => {
                // Faellige Callbacks NACH dem Einsammeln feuern -- ein Callback
                // darf selbst Timer registrieren/canceln; neue Eintraege werden
                // erst beim naechsten Update faellig geprueft.
                for cb in self.timers.take_due() {
                    self.rueckruf_rufen(&cb, "TIMER-Callback")?;
                }
                Value::Nil
            }
            "cooldown" => {
                let id = t_str(a, 0, "COOLDOWN")?.to_string();
                let ms = t_int(a, 1, "COOLDOWN")?;
                if ms < 0 { return Err("COOLDOWN: ms muss >= 0 sein".into()); }
                Value::Bool(self.timers.cooldown(&id, ms))
            }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }

    /// Modul `scene` (globaler Stack-State, kein Grafik-Bezug).
    pub(super) fn try_scene(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !(name.starts_with("scene_")) { return Ok(None); }
        fn sa<'x>(a: &'x [Value], i: usize, fn_: &str) -> R<&'x str> {
            crate::argument::a_text(a, i, fn_)
        }
        let r = match name {
            "scene_push" => { self.scene_stack.push((sa(a, 0, "SCENE_PUSH")?.to_string(), HashMap::new())); Value::Nil }
            "scene_pop" => { if self.scene_stack.is_empty() { return Err("SCENE_POP: Stack ist bereits leer".into()); } self.scene_stack.pop(); Value::Nil }
            "scene_switch" => { let n = sa(a, 0, "SCENE_SWITCH")?.to_string(); self.scene_stack.clear(); self.scene_stack.push((n, HashMap::new())); Value::Nil }
            "scene_current" => Value::str_rc(self.scene_stack.last().map(|(n, _)| n.as_str()).unwrap_or("")),
            "scene_depth" => Value::Int(self.scene_stack.len() as i64),
            "scene_has" => { let n = sa(a, 0, "SCENE_HAS")?; Value::Bool(self.scene_stack.iter().any(|(x, _)| x == n)) }
            "scene_reset" => { self.scene_stack.clear(); Value::Nil }
            "scene_set_int" | "scene_set_float" | "scene_set_string" | "scene_set_bool" => {
                let key = sa(a, 0, "SCENE_SET")?.to_string();
                let v = match name {
                    "scene_set_int" => match a.get(1) { Some(Value::Int(i)) => Value::Int(*i), _ => return Err("SCENE_SET_INT: value muss INTEGER sein".into()) },
                    "scene_set_float" => match a.get(1) { Some(Value::Int(i)) => Value::Float(*i as f64), Some(Value::Float(f)) => Value::Float(*f), _ => return Err("SCENE_SET_FLOAT: value muss Zahl sein".into()) },
                    "scene_set_string" => match a.get(1) { Some(Value::Str(s)) => Value::Str(s.clone()), _ => return Err("SCENE_SET_STRING: value muss STRING sein".into()) },
                    _ => match a.get(1) { Some(Value::Bool(b)) => Value::Bool(*b), _ => return Err("SCENE_SET_BOOL: value muss BOOLEAN sein".into()) },
                };
                self.scene_top_mut()?.1.insert(key, v); Value::Nil
            }
            "scene_get_int" | "scene_get_float" | "scene_get_string" | "scene_get_bool" => {
                let key = sa(a, 0, "SCENE_GET")?;
                let data = &self.scene_top()?.1;
                let val = data.get(key).ok_or_else(|| format!("{}: Key '{}' nicht in Scene", name.to_uppercase(), key))?;
                let ok = scene_type_ok(val, name);
                if !ok { return Err(format!("{}: Key '{}' hat falschen Typ", name.to_uppercase(), key)); }
                val.clone()
            }
            "scene_get_int_or" | "scene_get_float_or" | "scene_get_string_or" | "scene_get_bool_or" => {
                let key = sa(a, 0, "SCENE_GET_OR")?;
                let def = a.get(1).cloned().unwrap_or(Value::Nil);
                let data = &self.scene_top()?.1;
                match data.get(key) { Some(v) if scene_type_ok(v, name) => v.clone(), _ => def }
            }
            "scene_has_key" => { let key = sa(a, 0, "SCENE_HAS_KEY")?; Value::Bool(self.scene_top()?.1.contains_key(key)) }
            "scene_delete" => { let key = sa(a, 0, "SCENE_DELETE")?.to_string(); self.scene_top_mut()?.1.remove(&key); Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }
}
