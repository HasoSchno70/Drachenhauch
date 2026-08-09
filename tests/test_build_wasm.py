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


# (Der frühere Test compile_program (.gb -> program.gbc via Python-serialize) ist
#  entfernt: Stufe B bettet nur noch die Quelle ein, gbrt kompiliert im Browser.)


def test_copy_source_embeds_gb(tmp_path):
    bw = _load_build_wasm()
    gb = tmp_path / "hi.gb"
    gb.write_text('PRINT "x"\n', encoding="utf-8")
    out = bw.copy_source(gb, tmp_path)
    assert out.name == "program.gb" and out.exists()
    assert out.read_text(encoding="utf-8") == gb.read_text(encoding="utf-8")


def test_build_skips_gracefully_without_toolchain(tmp_path, monkeypatch):
    bw = _load_build_wasm()
    gb = tmp_path / "hi.gb"
    gb.write_text("PRINT 1\n", encoding="utf-8")
    # Toolchain als fehlend vortaeuschen -> build() darf nicht scheitern.
    monkeypatch.setattr(bw, "check_toolchain",
                        lambda: {"emcc": False, "cargo": False, "wasm_target": False})
    rc = bw.build(gb, tmp_path)
    assert rc == 0
    # Quelle wird eingebettet (gbrt kompiliert im Browser; kein Python-.gbc mehr).
    assert (tmp_path / "program.gb").exists()


def test_web_harness_files_present():
    web = ROOT / "web"
    html = (web / "index.html").read_text(encoding="utf-8")
    assert 'id="canvas"' in html          # raylib/emscripten Canvas
    assert 'id="src"' in html             # Quelltext-Editor (Live-Playground)
    assert "gbrt.js" in html              # laedt das wasm-Modul
    assert (web / "playground.js").exists()
    js = (web / "playground.js").read_text(encoding="utf-8")
    # Live-Editor: schreibt die User-Quelle ins FS, laedt zum Lauf frisch.
    assert "/program.gb" in js
    assert "location.reload" in js


def test_main_rs_has_wasm_entry():
    """main.rs kompiliert im emscripten-Build /program.gb (Quelle, Vorrang)
    und faellt auf /program.gbc zurueck."""
    src = (ROOT / "rust" / "drachenhauch_runtime" / "src" / "main.rs").read_text(
        encoding="utf-8")
    assert 'target_os = "emscripten"' in src
    assert "/program.gb" in src
    assert "/program.gbc" in src


def test_flag_aenderung_erzwingt_neues_linken(tmp_path, monkeypatch):
    """cargo verfolgt EMCC_CFLAGS nicht -- aendert man nur ein Linker-Flag,
    ueberspringt es das Linken und die ALTE .wasm bleibt liegen. Der Build
    meldet dann Erfolg, obwohl das Flag gar nicht drin ist (so sah es aus, als
    STACK_SIZE "nicht half"). Deshalb loescht build_wasm.py die Ausgabe selbst,
    sobald sich die Flags aendern."""
    bw = _load_build_wasm()
    rel = tmp_path / "target" / bw.TARGET / "release"
    rel.mkdir(parents=True)
    monkeypatch.setattr(bw, "CRATE", tmp_path)

    def lege_artefakte_an():
        for n in ("gbrt.js", "gbrt.wasm", "gbrt.data"):
            (rel / n).write_bytes(b"alt")

    lege_artefakte_an()
    assert bw.erzwinge_relink("-s A=1") is True, "erster Lauf muss stempeln"
    assert not (rel / "gbrt.wasm").exists(), "Ausgabe muss weg sein"

    # Gleiche Flags -> nichts anfassen (kein unnoetiges Neu-Linken).
    lege_artefakte_an()
    assert bw.erzwinge_relink("-s A=1") is False
    assert (rel / "gbrt.wasm").read_bytes() == b"alt"

    # Geaenderte Flags -> Ausgabe faellt weg, cargo linkt neu.
    assert bw.erzwinge_relink("-s A=2") is True
    assert not (rel / "gbrt.wasm").exists()
    assert not (rel / "gbrt.data").exists()
