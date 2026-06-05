"""Reine Python-Implementation von ECS-`_World`/`_Component`.

Frueher gab es eine Cython-Variante (`ecs_native.pyx`, cdef-Klassen +
Sparse-Set in C). Die wurde entfernt -- der Tree-Walker ist jetzt reiner
Editor-/Referenzpfad, die Performance liegt in der nativen Runtime `gbrt`
(Rust). `ecs.py` importiert `_World`/`_Component` aus diesem Modul.

Sparse-Set: `_Component` haelt `dense` (Entity-IDs), `values` (parallel) und
`sparse` (Entity -> Index in dense). ADD/REMOVE laufen swap-mit-letztem in O(1),
Iteration kontinuierlich ueber `dense`/`values`.
"""
from __future__ import annotations

from ..errors import GBRuntimeError, TypeMismatchError


class _Component:
    __slots__ = ("dense", "values", "sparse")

    def __init__(self):
        self.dense = []
        self.values = []
        self.sparse = {}

    def __len__(self):
        return len(self.dense)

    def has(self, ent):
        return ent in self.sparse

    def get(self, ent):
        return self.values[self.sparse[ent]]

    def get_or(self, ent, default):
        i = self.sparse.get(ent, -1)
        if i < 0:
            return default
        return self.values[i]

    def set(self, ent, value):
        i = self.sparse.get(ent, -1)
        if i >= 0:
            self.values[i] = value
        else:
            self.sparse[ent] = len(self.dense)
            self.dense.append(ent)
            self.values.append(value)

    def remove(self, ent):
        i = self.sparse.pop(ent, -1)
        if i < 0:
            return False
        last = len(self.dense) - 1
        if i != last:
            moved_ent = self.dense[last]
            self.dense[i] = moved_ent
            self.values[i] = self.values[last]
            self.sparse[moved_ent] = i
        self.dense.pop()
        self.values.pop()
        return True


class _World:
    __slots__ = ("next_id", "entities", "components")

    def __init__(self):
        self.next_id = 1
        self.entities = set()
        self.components = {}

    def __repr__(self):
        return f"<ECS_WORLD entities={len(self.entities)}>"

    def new_entity(self):
        eid = self.next_id
        self.next_id += 1
        self.entities.add(eid)
        return eid

    def destroy(self, ent):
        if ent not in self.entities:
            return False
        self.entities.discard(ent)
        for c in self.components.values():
            c.remove(ent)
        return True

    def alive(self, ent):
        return ent in self.entities

    def count(self):
        return len(self.entities)

    def _get_or_create(self, name):
        c = self.components.get(name)
        if c is None:
            c = _Component()
            self.components[name] = c
        return c

    def _check_ent(self, ent, fn):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError(f"{fn} erwartet ent")
        if ent not in self.entities:
            raise GBRuntimeError(f"{fn}: Entity {ent} nicht in World")

    def add_int(self, ent, name, value):
        if isinstance(value, bool) or not isinstance(value, int):
            raise TypeMismatchError("ECS_ADD_INT erwartet value")
        self._check_ent(ent, "ECS_ADD_INT")
        self._get_or_create(name).set(ent, value)

    def add_float(self, ent, name, value):
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise TypeMismatchError("ECS_ADD_FLOAT erwartet value")
        self._check_ent(ent, "ECS_ADD_FLOAT")
        self._get_or_create(name).set(ent, float(value))

    def add_string(self, ent, name, value):
        if not isinstance(value, str):
            raise TypeMismatchError("ECS_ADD_STRING erwartet value")
        self._check_ent(ent, "ECS_ADD_STRING")
        self._get_or_create(name).set(ent, value)

    def add_bool(self, ent, name, value):
        if not isinstance(value, bool):
            raise TypeMismatchError("ECS_ADD_BOOL erwartet value")
        self._check_ent(ent, "ECS_ADD_BOOL")
        self._get_or_create(name).set(ent, value)

    def add_obj(self, ent, name, value):
        self._check_ent(ent, "ECS_ADD_OBJ")
        self._get_or_create(name).set(ent, value)

    def _get_value(self, ent, name, fn):
        c = self.components.get(name)
        if c is None:
            raise GBRuntimeError(
                f"{fn}: Component '{name}' fehlt bei Entity {ent}"
            )
        i = c.sparse.get(ent, -1)
        if i < 0:
            raise GBRuntimeError(
                f"{fn}: Component '{name}' fehlt bei Entity {ent}"
            )
        return c.values[i]

    def get(self, ent, name):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET erwartet ent")
        return self._get_value(ent, name, "ECS_GET")

    def get_int(self, ent, name):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET_INT erwartet ent")
        v = self._get_value(ent, name, "ECS_GET_INT")
        if isinstance(v, bool) or not isinstance(v, int):
            raise TypeMismatchError(
                f"ECS_GET_INT: Component '{name}' ist nicht INTEGER"
            )
        return v

    def get_float(self, ent, name):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET_FLOAT erwartet ent")
        v = self._get_value(ent, name, "ECS_GET_FLOAT")
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise TypeMismatchError(
                f"ECS_GET_FLOAT: Component '{name}' ist nicht Zahl"
            )
        return float(v)

    def get_string(self, ent, name):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET_STRING erwartet ent")
        v = self._get_value(ent, name, "ECS_GET_STRING")
        if not isinstance(v, str):
            raise TypeMismatchError(
                f"ECS_GET_STRING: Component '{name}' ist nicht STRING"
            )
        return v

    def get_bool(self, ent, name):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET_BOOL erwartet ent")
        v = self._get_value(ent, name, "ECS_GET_BOOL")
        if not isinstance(v, bool):
            raise TypeMismatchError(
                f"ECS_GET_BOOL: Component '{name}' ist nicht BOOLEAN"
            )
        return v

    def has_component(self, ent, name):
        if ent not in self.entities:
            return False
        c = self.components.get(name)
        return c is not None and c.has(ent)

    def remove_component(self, ent, name):
        c = self.components.get(name)
        if c is None:
            return False
        return c.remove(ent)

    def integrate_float(self, target, delta):
        target_c = self.components.get(target)
        delta_c = self.components.get(delta)
        if target_c is None or delta_c is None:
            return 0
        t_values = target_c.values
        d_values = delta_c.values
        t_sparse = target_c.sparse
        d_sparse = delta_c.sparse
        count = 0
        if len(delta_c.dense) <= len(target_c.dense):
            for i, ent in enumerate(delta_c.dense):
                t_idx = t_sparse.get(ent, -1)
                if t_idx < 0:
                    continue
                t_values[t_idx] = float(t_values[t_idx]) + float(d_values[i])
                count += 1
        else:
            for i, ent in enumerate(target_c.dense):
                d_idx = d_sparse.get(ent, -1)
                if d_idx < 0:
                    continue
                t_values[i] = float(t_values[i]) + float(d_values[d_idx])
                count += 1
        return count

    def integrate_int(self, target, delta):
        target_c = self.components.get(target)
        delta_c = self.components.get(delta)
        if target_c is None or delta_c is None:
            return 0
        t_values = target_c.values
        d_values = delta_c.values
        t_sparse = target_c.sparse
        d_sparse = delta_c.sparse
        count = 0
        if len(delta_c.dense) <= len(target_c.dense):
            for i, ent in enumerate(delta_c.dense):
                t_idx = t_sparse.get(ent, -1)
                if t_idx < 0:
                    continue
                t_values[t_idx] = int(t_values[t_idx]) + int(d_values[i])
                count += 1
        else:
            for i, ent in enumerate(target_c.dense):
                d_idx = d_sparse.get(ent, -1)
                if d_idx < 0:
                    continue
                t_values[i] = int(t_values[i]) + int(d_values[d_idx])
                count += 1
        return count

    def scale_float(self, target, factor):
        c = self.components.get(target)
        if c is None:
            return 0
        for i in range(len(c.values)):
            c.values[i] = float(c.values[i]) * factor
        return len(c.values)

    def fill_float(self, target, value):
        c = self.components.get(target)
        if c is None:
            return 0
        n = len(c.values)
        for i in range(n):
            c.values[i] = value
        return n

    def fill_int(self, target, value):
        c = self.components.get(target)
        if c is None:
            return 0
        n = len(c.values)
        for i in range(n):
            c.values[i] = value
        return n

    def clamp_float(self, target, lo, hi):
        if lo > hi:
            raise GBRuntimeError(
                f"ECS_CLAMP_FLOAT: lo ({lo}) > hi ({hi})"
            )
        c = self.components.get(target)
        if c is None:
            return 0
        for i in range(len(c.values)):
            v = float(c.values[i])
            if v < lo:
                c.values[i] = lo
            elif v > hi:
                c.values[i] = hi
        return len(c.values)

    def remove_dead(self, name, threshold):
        c = self.components.get(name)
        if c is None:
            return 0
        to_kill = [
            ent for i, ent in enumerate(c.dense)
            if float(c.values[i]) <= threshold
        ]
        for ent in to_kill:
            self.destroy(ent)
        return len(to_kill)

    def count_with(self, name):
        c = self.components.get(name)
        if c is None:
            return 0
        return len(c.dense)
