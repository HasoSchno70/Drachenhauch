"""JSON-Extraktion des Profilers (Qt-/dhrt-frei -- laeuft immer).

`dhrt profile` liefert das Ergebnis als eine JSON-Zeile, aber raylib schreibt
seine TraceLogs (WARNING/INFO) ebenfalls auf stdout. `_extract_profile_json`
muss den JSON-Blob trotzdem zuverlaessig herausziehen -- sonst bleibt die
Auswertung leer (Regression 99_ibl_hdr.gb: `MESH_SPHERE` loest eine
raylib-MESH-WARNING aus, die dem JSON vorangestellt wird)."""
from gamebasic.editor_qt.profiler import _extract_profile_json


def test_clean():
    out = '{"total_time":0.5,"output":"hi","lines":[],"stopped":false}'
    assert _extract_profile_json(out)["output"] == "hi"


def test_raylib_warning_prefix():
    out = (
        "WARNING: MESH: vertexCount expected to be a multiple of 3.\n"
        '{"total_time":0.5,"output":"","lines":[{"line":1,"count":1,"time":0.1}],"stopped":false}\n'
    )
    data = _extract_profile_json(out)
    assert data["total_time"] == 0.5
    assert len(data["lines"]) == 1


def test_shutdown_log_suffix():
    # Fenster-Shutdown-Logs koennen NACH dem JSON-println kommen.
    out = (
        '{"total_time":0.5,"output":"","lines":[],"stopped":false}\n'
        "INFO: Window closed successfully\n"
    )
    assert _extract_profile_json(out)["total_time"] == 0.5


def test_empty_or_garbage():
    assert _extract_profile_json("") == {}
    assert _extract_profile_json("WARNING: only logs here\n") == {}
