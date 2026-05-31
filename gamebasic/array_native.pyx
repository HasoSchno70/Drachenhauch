# cython: language_level=3
# cython: boundscheck=False
# cython: wraparound=False
# cython: initializedcheck=False
# cython: cdivision=True
"""Native (Cython) Implementation von _GBArray.

Identische Semantik wie die ehemalige Python-Klasse in interpreter.py;
schneller, weil:

1. Cdef-class -- Attribut-Zugriff laeuft ueber C-Slots statt Dict-Lookup.
2. ``flat_index`` ist als ``cdef inline`` implementiert -- der Bounds-Check
   und die Stride-Multiplikation laufen in reinem C statt im Python-
   Interpreter.
3. Fuer ``ARRAY OF INTEGER/FLOAT`` haelt die Klasse zusaetzlich einen
   ``long[::1]`` bzw. ``double[::1]``-Memoryview auf den array.array-Buffer.
   Die neue ``get_at``/``set_at``-API bypasst die Python-getitem-Schicht
   und liest/schreibt direkt in den C-Buffer.
4. Aufrufer, die noch ``arr.values[arr.flat_index(idx)]`` schreiben,
   profitieren ebenfalls -- wegen (1) und (2). Die ``get_at``-API ist
   zusaetzlicher Speed nur fuer Hot-Pfade.

Public-API (kompatibel zur alten Python-Klasse):
    arr.element_type      str
    arr.dims              tuple
    arr.strides           tuple
    arr.values            list oder array.array
    arr.total_size()
    arr.flat_index(indices)
    len(arr)
    repr(arr)

Neu (fuer VM-Hot-Pfade):
    arr.get_at(indices)         schneller Read
    arr.set_at(indices, value)  schneller Write
"""
import array as _array_mod
from .errors import GBRuntimeError


cdef class _GBArray:
    cdef public str element_type
    cdef public tuple dims
    cdef public tuple strides
    cdef public object values        # list ODER array.array

    # Typed-Backing-Views. Nur belegt, wenn _kind != 0.
    cdef long long[::1] _int_view
    cdef double[::1] _float_view
    # 0=generic (Python-Liste), 1=integer (long-View), 2=float (double-View).
    cdef int _kind

    def __init__(self, element_type, dims, default_factory):
        self.element_type = element_type
        # Dims als Python-int-Tuple
        cdef list dim_list = [int(d) for d in dims]
        self.dims = tuple(dim_list)
        # Strides berechnen (Row-major).
        cdef list strides_rev = []
        cdef long acc = 1
        cdef long d
        for d in reversed(dim_list):
            strides_rev.append(acc)
            acc *= d
        strides_rev.reverse()
        self.strides = tuple(strides_rev)

        if element_type == "integer":
            backing = _array_mod.array('q', [0]) * acc
            self.values = backing
            self._int_view = backing
            self._kind = 1
        elif element_type == "float":
            backing = _array_mod.array('d', [0.0]) * acc
            self.values = backing
            self._float_view = backing
            self._kind = 2
        else:
            self.values = [default_factory() for _ in range(acc)]
            self._kind = 0

    cpdef Py_ssize_t total_size(self):
        return len(self.values)

    # Cdef-inline: Bounds-Check + Stride-Multiplikation in reinem C.
    # ``except -1`` gibt -1 als Exception-Sentinel zurueck. Da der gueltige
    # Bereich [0..N) ist und 0 eine gueltige Adresse ist, faengt Cython
    # echte Exceptions ueber den Negativ-Returnwert ab.
    cdef inline Py_ssize_t _flat_c(self, indices) except -1:
        cdef Py_ssize_t flat = 0
        cdef Py_ssize_t k, idx
        cdef Py_ssize_t n_dims = len(self.dims)
        cdef Py_ssize_t n_idx = len(indices)
        if n_idx != n_dims:
            raise GBRuntimeError(
                f"Array hat {n_dims} Dimension(en), erhalten {n_idx} Index/-e"
            )
        cdef tuple dims_t = self.dims
        cdef tuple strides_t = self.strides
        for k in range(n_dims):
            idx = indices[k]
            if idx < 0 or idx >= <Py_ssize_t>dims_t[k]:
                raise GBRuntimeError(
                    f"Index {idx} ausserhalb [0..{<Py_ssize_t>dims_t[k] - 1}] "
                    f"in Dimension {k}"
                )
            flat += idx * <Py_ssize_t>strides_t[k]
        return flat

    cpdef Py_ssize_t flat_index(self, indices):
        return self._flat_c(indices)

    # Schnelle typisierte Reads/Writes -- die VMs rufen diese statt
    # ``arr.values[arr.flat_index(idx)]``. Spart die Python-getitem-Schicht
    # auf array.array bzw. die Dict-Attribut-Lookups.
    cpdef object get_at(self, indices):
        cdef Py_ssize_t flat = self._flat_c(indices)
        if self._kind == 1:
            return self._int_view[flat]
        if self._kind == 2:
            return self._float_view[flat]
        return self.values[flat]

    cpdef set_at(self, indices, value):
        cdef Py_ssize_t flat = self._flat_c(indices)
        if self._kind == 1:
            # array.array('q') akzeptiert int. Wenn value out of range,
            # wirft Cython einen OverflowError -- konsistent mit dem
            # bestehenden 64-bit-Limit.
            self._int_view[flat] = value
        elif self._kind == 2:
            self._float_view[flat] = value
        else:
            self.values[flat] = value

    def __len__(self):
        return self.dims[0] if self.dims else 0

    def __repr__(self):
        shape = ",".join(str(d) for d in self.dims)
        return f"<ARRAY[{shape}] OF {self.element_type.upper()}>"
