"""Tests fuer das WASM-Build-Geruest (rust/build_wasm.py) + Web-Harness.

Der eigentliche emscripten-Build ist hier nicht ausfuehrbar; getestet wird,
dass (a) die Toolchain-Pruefung graceful Bools liefert, (b) program.gbc korrekt
erzeugt wird und vom Loader gelesen werden kann, (c) das Web-Harness die
erwarteten Haken enthaelt.
"""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def _load_build_wasm():
    spec = importlib.util.spec_from_file_location(
        "build_wasm", ROOT / "rust" / "build_wasm.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def test_check_toolchain_returns_bools():
    bw = _load_build_wasm()
    info = bw.check_toolchain()
    assert set(info) == {"emcc", "cargo", "wasm_target"}
    assert all(isinstance(v, bool) for v in info.values())


def test_compile_program_produces_loadable_gbc(tmp_path):
    bw = _load_build_wasm()
    gb = tmp_path / "hi.gb"
    gb.write_text('PRINT "hallo"\n', encoding="utf-8")
    gbc = bw.compile_program(gb, tmp_path)
    assert gbc.name == "program.gbc" and gbc.exists()
    data = json.loads(gbc.read_text(encoding="utf-8"))
    assert isinstance(data, dict)        # serialisiertes Module
    # Vom selben Loader lesbar, den gbrt/Python nutzen.
    from gamebasic.serialize import compile_gb_to_module
    assert compile_gb_to_module(gb) is not None


def test_build_skips_gracefully_without_toolchain(tmp_path, monkeypatch):
    bw = _load_build_wasm()
    gb = tmp_path / "hi.gb"
    gb.write_text("PRINT 1\n", encoding="utf-8")
    # Toolchain als fehlend vortaeuschen -> build() darf nicht scheitern.
    monkeypatch.setattr(bw, "check_toolchain",
                        lambda: {"emcc": False, "cargo": False, "wasm_target": False})
    rc = bw.build(gb, tmp_path)
    assert rc == 0
    assert (tmp_path / "program.gbc").exists()


def test_web_harness_files_present():
    web = ROOT / "web"
    html = (web / "index.html").read_text(encoding="utf-8")
    assert 'id="canvas"' in html          # raylib/emscripten Canvas
    assert "gbrt.js" in html              # laedt das wasm-Modul
    assert (web / "playground.js").exists()
    js = (web / "playground.js").read_text(encoding="utf-8")
    assert "callMain" in js               # startet main()
    assert "program.gbc" in js


def test_main_rs_has_wasm_entry():
    """main.rs liest im emscripten-Build /program.gbc."""
    src = (ROOT / "rust" / "gb_runtime" / "src" / "main.rs").read_text(
        encoding="utf-8")
    assert 'target_os = "emscripten"' in src
    assert "/program.gbc" in src
