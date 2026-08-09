"""Builtin-Metadaten-Shim (Stufe B, Phase 2): eingefrorener builtin_index.json
+ dhrt_meta-Loader entkoppeln Editor/LSP von der Laufzeit (interpreter.py)."""
from drachenhauch.editor_qt import dhrt_meta


def test_index_loads_and_is_substantial():
    idx = dhrt_meta.builtin_index()
    assert len(idx) > 700                  # vollstaendige Registry-Union (~764)
    e = idx[0]
    assert {"name", "kind", "signature", "module"} <= set(e)


def test_core_and_graphics_and_dhrt_only_names_present():
    upper = set(dhrt_meta.builtin_names_upper())
    # Core, Grafik und dhrt-only (nur in builtin_docs) muessen alle dabei sein.
    for n in ("ABS", "SIN", "STR$", "LEN"):
        assert n in upper
    for n in ("LINE", "BOX", "DRAWIMAGE"):
        assert n in upper
    for n in ("ARRAY_PUSH", "STARTSWITH"):     # dhrt-only Erweiterungen
        assert n in upper


def test_names_lower_is_lowercased_set():
    low = dhrt_meta.builtin_names_lower()
    assert "abs" in low and "line" in low
    assert all(n == n.lower() for n in low)


def test_signature_lookup():
    assert dhrt_meta.signature("ABS")           # nicht leer
    assert dhrt_meta.signature("nicht_da_xyz") == ""


def test_by_module_grouping():
    groups = dhrt_meta.by_module()
    assert "core" in groups                     # Core-Builtins (kein IMPORT)
    # Modul-Builtins sind nach ihrem Modul gruppiert (z.B. g3d, json, vec2).
    assert any(m in groups for m in ("g3d", "json", "vec2"))
