"""Builtin-Metadaten-Shim (Stufe B, Phase 2): eingefrorener builtin_index.json
+ gbrt_meta-Loader entkoppeln Editor/LSP von der Laufzeit (interpreter.py)."""
from gamebasic.editor_qt import gbrt_meta


def test_index_loads_and_is_substantial():
    idx = gbrt_meta.builtin_index()
    assert len(idx) > 700                  # vollstaendige Registry-Union (~764)
    e = idx[0]
    assert {"name", "kind", "signature", "module"} <= set(e)


def test_core_and_graphics_and_gbrt_only_names_present():
    upper = set(gbrt_meta.builtin_names_upper())
    # Core, Grafik und gbrt-only (nur in builtin_docs) muessen alle dabei sein.
    for n in ("ABS", "SIN", "STR$", "LEN"):
        assert n in upper
    for n in ("LINE", "BOX", "DRAWIMAGE"):
        assert n in upper
    for n in ("ARRAY_PUSH", "STARTSWITH"):     # gbrt-only Erweiterungen
        assert n in upper


def test_names_lower_is_lowercased_set():
    low = gbrt_meta.builtin_names_lower()
    assert "abs" in low and "line" in low
    assert all(n == n.lower() for n in low)


def test_signature_lookup():
    assert gbrt_meta.signature("ABS")           # nicht leer
    assert gbrt_meta.signature("nicht_da_xyz") == ""


def test_by_module_grouping():
    groups = gbrt_meta.by_module()
    assert "core" in groups                     # Core-Builtins (kein IMPORT)
    # Modul-Builtins sind nach ihrem Modul gruppiert (z.B. g3d, json, vec2).
    assert any(m in groups for m in ("g3d", "json", "vec2"))
