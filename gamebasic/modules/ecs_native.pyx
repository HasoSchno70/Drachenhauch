# cython: language_level=3
# cython: boundscheck=False
# cython: wraparound=False
# cython: initializedcheck=False
# cython: cdivision=True
"""Native (Cython) Implementation der ECS-Kernklassen.

Identische Semantik zur urspruenglichen Python-Variante in modules/ecs.py;
schneller weil:

1. ``_World`` und ``_Component`` sind cdef-Klassen -- Attribut-Zugriff
   geht ueber C-Slots statt Dict-Lookup, und Methodenaufrufe sind
   direkte C-Calls statt Python-Frame-Setup.
2. Die Sparse-Set-Ops (has/get/set/remove) sind cpdef-Methoden mit
   ``except``-Annotation -- Cython generiert direkten Pfad-Code.
3. Wichtig: ``_World`` exponiert Fast-Path-Methoden ``get_float``,
   ``add_float`` usw., die die gesamte Pipeline eines @builtin-Calls
   (Type-Checks + dict-Lookups + Component-Zugriff + Wert-Konvertierung)
   in einer cdef-Function abwickeln. Die @builtin-Wrapper in modules/ecs.py
   schrumpfen damit zu One-Liner-Shims, was den Hot-Path bei 100k+
   Calls/Frame spuerbar entlastet.

Nur die hot-path-relevanten Typen (FLOAT, INT) bekommen Fast-Path-Methoden.
STRING / BOOL / OBJ bleiben in der Python-ecs.py-Schicht (selten genug).

Bei `register_type` und @builtin-Decorator bleiben die Klassen-Identitaet
und der Builtin-Name identisch. ``isinstance(x, _World)`` liefert weiterhin
korrekt True/False.
"""
from ..errors import GBRuntimeError, TypeMismatchError


cdef class _Component:
    """Sparse-Set fuer einen einzigen Component-Namen.

    sparse[ent_id] = i  =>  dense[i] == ent_id  und  values[i] = wert
    """
    cdef public list dense
    cdef public list values
    cdef public dict sparse

    def __init__(self):
        self.dense = []
        self.values = []
        self.sparse = {}

    def __len__(self):
        return len(self.dense)

    cpdef bint has(self, ent):
        return ent in self.sparse

    cpdef object get(self, ent):
        """Liefert den Wert oder wirft KeyError."""
        return self.values[<Py_ssize_t>self.sparse[ent]]

    cpdef object get_or(self, ent, default):
        """Liefert den Wert oder ``default`` wenn nicht vorhanden."""
        cdef object idx = self.sparse.get(ent)
        if idx is None:
            return default
        return self.values[<Py_ssize_t>idx]

    cpdef set(self, ent, value):
        """Setzt den Wert. Wenn ent schon Halter ist, ueberschreibe;
        sonst neu anhaengen."""
        cdef object existing = self.sparse.get(ent)
        cdef Py_ssize_t i
        if existing is not None:
            i = <Py_ssize_t>existing
            self.values[i] = value
        else:
            self.sparse[ent] = len(self.dense)
            self.dense.append(ent)
            self.values.append(value)

    cpdef bint remove(self, ent):
        """Entfernt ent aus dem Component. Liefert True wenn vorhanden.
        Swap-with-last fuer O(1)."""
        cdef object existing = self.sparse.pop(ent, None)
        if existing is None:
            return False
        cdef Py_ssize_t i = <Py_ssize_t>existing
        cdef Py_ssize_t last = len(self.dense) - 1
        cdef object moved_ent
        if i != last:
            moved_ent = self.dense[last]
            self.dense[i] = moved_ent
            self.values[i] = self.values[last]
            self.sparse[moved_ent] = i
        self.dense.pop()
        self.values.pop()
        return True


cdef class _World:
    """ECS-World mit Component-Storage als Sparse-Sets."""
    cdef public Py_ssize_t next_id
    cdef public set entities
    cdef public dict components

    def __init__(self):
        self.next_id = 1
        self.entities = set()
        self.components = {}

    def __repr__(self):
        return f"<ECS_WORLD entities={len(self.entities)}>"

    cpdef Py_ssize_t new_entity(self):
        cdef Py_ssize_t eid = self.next_id
        self.next_id = eid + 1
        self.entities.add(eid)
        return eid

    cpdef bint destroy(self, ent):
        if ent not in self.entities:
            return False
        self.entities.discard(ent)
        cdef _Component c
        for c in self.components.values():
            c.remove(ent)
        return True

    cpdef bint alive(self, ent):
        return ent in self.entities

    cpdef Py_ssize_t count(self):
        return len(self.entities)

    cdef inline _Component _get_or_create(self, str name):
        cdef _Component c = self.components.get(name)
        if c is None:
            c = _Component()
            self.components[name] = c
        return c

    # --- Fast-Path-Methoden ----------------------------------------
    # Diese subsumieren die ehemals separaten _check/_get_value/_b_-
    # Funktionen in einer cpdef-Method. Vom @builtin-Wrapper aus ist
    # das ein einziger C-Call statt einer Kette Python-Operationen.

    cpdef add_int(self, ent, str name, value):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_ADD_INT erwartet ent")
        if isinstance(value, bool) or not isinstance(value, int):
            raise TypeMismatchError("ECS_ADD_INT erwartet value")
        if ent not in self.entities:
            raise GBRuntimeError(f"ECS_ADD_INT: Entity {ent} nicht in World")
        self._get_or_create(name).set(ent, value)

    cpdef add_float(self, ent, str name, value):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_ADD_FLOAT erwartet ent")
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise TypeMismatchError("ECS_ADD_FLOAT erwartet value")
        if ent not in self.entities:
            raise GBRuntimeError(f"ECS_ADD_FLOAT: Entity {ent} nicht in World")
        self._get_or_create(name).set(ent, float(value))

    cpdef add_string(self, ent, str name, value):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_ADD_STRING erwartet ent")
        if not isinstance(value, str):
            raise TypeMismatchError("ECS_ADD_STRING erwartet value")
        if ent not in self.entities:
            raise GBRuntimeError(f"ECS_ADD_STRING: Entity {ent} nicht in World")
        self._get_or_create(name).set(ent, value)

    cpdef add_bool(self, ent, str name, value):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_ADD_BOOL erwartet ent")
        if not isinstance(value, bool):
            raise TypeMismatchError("ECS_ADD_BOOL erwartet value")
        if ent not in self.entities:
            raise GBRuntimeError(f"ECS_ADD_BOOL: Entity {ent} nicht in World")
        self._get_or_create(name).set(ent, value)

    cpdef add_obj(self, ent, str name, value):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_ADD_OBJ erwartet ent")
        if ent not in self.entities:
            raise GBRuntimeError(f"ECS_ADD_OBJ: Entity {ent} nicht in World")
        self._get_or_create(name).set(ent, value)

    cdef inline object _get_value(self, ent, str name, str fn):
        cdef _Component c = self.components.get(name)
        if c is None:
            raise GBRuntimeError(
                f"{fn}: Component '{name}' fehlt bei Entity {ent}"
            )
        cdef object idx = c.sparse.get(ent)
        if idx is None:
            raise GBRuntimeError(
                f"{fn}: Component '{name}' fehlt bei Entity {ent}"
            )
        return c.values[<Py_ssize_t>idx]

    cpdef object get(self, ent, str name):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET erwartet ent")
        return self._get_value(ent, name, "ECS_GET")

    cpdef object get_int(self, ent, str name):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET_INT erwartet ent")
        v = self._get_value(ent, name, "ECS_GET_INT")
        if isinstance(v, bool) or not isinstance(v, int):
            raise TypeMismatchError(
                f"ECS_GET_INT: Component '{name}' ist nicht INTEGER"
            )
        return v

    cpdef double get_float(self, ent, str name) except? -1.0:
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET_FLOAT erwartet ent")
        v = self._get_value(ent, name, "ECS_GET_FLOAT")
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise TypeMismatchError(
                f"ECS_GET_FLOAT: Component '{name}' ist nicht Zahl"
            )
        return <double>v

    cpdef str get_string(self, ent, str name):
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET_STRING erwartet ent")
        v = self._get_value(ent, name, "ECS_GET_STRING")
        if not isinstance(v, str):
            raise TypeMismatchError(
                f"ECS_GET_STRING: Component '{name}' ist nicht STRING"
            )
        return v

    cpdef bint get_bool(self, ent, str name) except? False:
        if isinstance(ent, bool) or not isinstance(ent, int):
            raise TypeMismatchError("ECS_GET_BOOL erwartet ent")
        v = self._get_value(ent, name, "ECS_GET_BOOL")
        if not isinstance(v, bool):
            raise TypeMismatchError(
                f"ECS_GET_BOOL: Component '{name}' ist nicht BOOLEAN"
            )
        return v

    cpdef bint has_component(self, ent, str name):
        if ent not in self.entities:
            return False
        cdef _Component c = self.components.get(name)
        return c is not None and c.has(ent)

    cpdef bint remove_component(self, ent, str name):
        cdef _Component c = self.components.get(name)
        if c is None:
            return False
        return c.remove(ent)

    # --- Bulk-System-Ops --------------------------------------------
    # Game-Engine-Pattern: ein Builtin verarbeitet eine ganze Component-
    # Schicht in einer C-Loop, statt dass das User-Programm pro Entity
    # 6 Python-Builtin-Calls absetzt. Das ist der Punkt, an dem ECS
    # seinen Performance-Vorteil ueberhaupt erst ausspielt -- vorher
    # dominiert die Builtin-Dispatch-Overhead alles.

    cpdef Py_ssize_t integrate_float(self, str target, str delta):
        """Fuer alle Entities, die ``target`` UND ``delta`` (FLOAT-Components)
        haben: ``target += delta``.

        Liefert die Anzahl bewegter Entities. Wenn einer der Components
        nicht existiert, ist das ein No-Op (Return 0). Komponenten-Werte
        werden zu float konvertiert; bool und non-numeric werfen.

        Ueber den kleineren der beiden Components iterieren -- wie der
        Query-Pfad. Fuer den typischen 1:1-Fall (px+vx auf der gleichen
        Entity-Population) macht das keinen Unterschied; bei sparsen
        Vel-Komponenten spart es Iterationen.
        """
        cdef _Component target_c = self.components.get(target)
        cdef _Component delta_c = self.components.get(delta)
        if target_c is None or delta_c is None:
            return 0
        cdef list t_values = target_c.values
        cdef list d_values = delta_c.values
        cdef list d_dense = delta_c.dense
        cdef list t_dense = target_c.dense
        cdef dict t_sparse = target_c.sparse
        cdef dict d_sparse = delta_c.sparse
        cdef Py_ssize_t count = 0
        cdef Py_ssize_t i, n, t_idx, d_idx
        cdef object idx_obj
        cdef object ent
        cdef double tv, dv

        if len(d_dense) <= len(t_dense):
            # Ueber delta iterieren, target_sparse fragen
            n = len(d_dense)
            for i in range(n):
                ent = d_dense[i]
                idx_obj = t_sparse.get(ent)
                if idx_obj is None:
                    continue
                t_idx = <Py_ssize_t>idx_obj
                tv = <double>t_values[t_idx]
                dv = <double>d_values[i]
                t_values[t_idx] = tv + dv
                count += 1
        else:
            # Ueber target iterieren, delta_sparse fragen
            n = len(t_dense)
            for i in range(n):
                ent = t_dense[i]
                idx_obj = d_sparse.get(ent)
                if idx_obj is None:
                    continue
                d_idx = <Py_ssize_t>idx_obj
                tv = <double>t_values[i]
                dv = <double>d_values[d_idx]
                t_values[i] = tv + dv
                count += 1
        return count

    cpdef Py_ssize_t integrate_int(self, str target, str delta):
        """INT-Variante von integrate_float. target += delta fuer alle
        Entities mit beiden Components. Liefert Anzahl bewegter Entities.
        """
        cdef _Component target_c = self.components.get(target)
        cdef _Component delta_c = self.components.get(delta)
        if target_c is None or delta_c is None:
            return 0
        cdef list t_values = target_c.values
        cdef list d_values = delta_c.values
        cdef list d_dense = delta_c.dense
        cdef list t_dense = target_c.dense
        cdef dict t_sparse = target_c.sparse
        cdef dict d_sparse = delta_c.sparse
        cdef Py_ssize_t count = 0
        cdef Py_ssize_t i, n, t_idx, d_idx
        cdef object idx_obj
        cdef object ent
        cdef long long tv, dv

        if len(d_dense) <= len(t_dense):
            n = len(d_dense)
            for i in range(n):
                ent = d_dense[i]
                idx_obj = t_sparse.get(ent)
                if idx_obj is None:
                    continue
                t_idx = <Py_ssize_t>idx_obj
                tv = <long long>t_values[t_idx]
                dv = <long long>d_values[i]
                t_values[t_idx] = tv + dv
                count += 1
        else:
            n = len(t_dense)
            for i in range(n):
                ent = t_dense[i]
                idx_obj = d_sparse.get(ent)
                if idx_obj is None:
                    continue
                d_idx = <Py_ssize_t>idx_obj
                tv = <long long>t_values[i]
                dv = <long long>d_values[d_idx]
                t_values[i] = tv + dv
                count += 1
        return count

    cpdef Py_ssize_t scale_float(self, str target, double factor):
        """Multipliziert alle Werte des FLOAT-Components mit ``factor``.
        Klassischer Use-Case: ``scale_float(world, "vel", 0.95)`` fuer
        Friction/Damping. Liefert Anzahl skalierter Entities.
        """
        cdef _Component c = self.components.get(target)
        if c is None:
            return 0
        cdef list values = c.values
        cdef Py_ssize_t i, n = len(values)
        for i in range(n):
            values[i] = <double>values[i] * factor
        return n

    cpdef Py_ssize_t fill_float(self, str target, double value):
        """Setzt alle Werte eines FLOAT-Components auf ``value``.
        Liefert Anzahl der gesetzten Entities (= Halter des Components).
        """
        cdef _Component c = self.components.get(target)
        if c is None:
            return 0
        cdef list values = c.values
        cdef Py_ssize_t i, n = len(values)
        for i in range(n):
            values[i] = value
        return n

    cpdef Py_ssize_t fill_int(self, str target, long long value):
        """INT-Variante von fill_float."""
        cdef _Component c = self.components.get(target)
        if c is None:
            return 0
        cdef list values = c.values
        cdef Py_ssize_t i, n = len(values)
        for i in range(n):
            values[i] = value
        return n

    cpdef Py_ssize_t clamp_float(self, str target, double lo, double hi):
        """Klemmt alle Werte eines FLOAT-Components auf [lo, hi]. Liefert
        Anzahl der bearbeiteten Entities. Wenn ``lo > hi``, wird gewerft.
        """
        if lo > hi:
            raise GBRuntimeError(
                f"ECS_CLAMP_FLOAT: lo ({lo}) > hi ({hi})"
            )
        cdef _Component c = self.components.get(target)
        if c is None:
            return 0
        cdef list values = c.values
        cdef Py_ssize_t i, n = len(values)
        cdef double v
        for i in range(n):
            v = <double>values[i]
            if v < lo:
                values[i] = lo
            elif v > hi:
                values[i] = hi
        return n

    cpdef Py_ssize_t remove_dead(self, str name, double threshold):
        """Zerstoert alle Entities, deren ``name``-Component-Wert <=
        ``threshold`` ist. Wird ueblicherweise mit ``hp`` und 0 benutzt:
        ``remove_dead(world, "hp", 0)``. Liefert Anzahl zerstoerter
        Entities.

        Wichtig: Sammelt zuerst alle IDs in einer Liste, weil destroy()
        den Sparse-Set mutiert -- waehrend des Iterierens zu loeschen
        wuerde Indizes durcheinanderbringen.
        """
        cdef _Component c = self.components.get(name)
        if c is None:
            return 0
        cdef list values = c.values
        cdef list dense = c.dense
        cdef list to_kill = []
        cdef Py_ssize_t i, n = len(dense)
        for i in range(n):
            if <double>values[i] <= threshold:
                to_kill.append(dense[i])
        for ent in to_kill:
            self.destroy(ent)
        return len(to_kill)

    cpdef Py_ssize_t count_with(self, str name):
        """Zaehlt Entities, die ``name`` als Component haben. O(1) --
        liest nur die Sparse-Set-Laenge, kein Scan.
        """
        cdef _Component c = self.components.get(name)
        if c is None:
            return 0
        return len(c.dense)
