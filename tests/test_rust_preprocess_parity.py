"""Parity-Test: Rust-Preprocessor (`gbrt --preprocess`) == preprocess.process().

Stufe 4 der Front-End-Portierung. Der Rust-Preprocessor (src/preprocess.rs)
expandiert IMPORTs (Quellcode-Inlining + Built-in-Modul-Erkennung) VOR dem
Lexen. Gate = **Merge-Ergebnis-Gleichheit**: die gemergte Quelle muss exakt der
von `gamebasic.preprocess.process()` entsprechen (Zeilen-Enden normalisiert).

Zusaetzlich ein End-to-End-Check: `gbrt --runsrc` (jetzt mit Preprocess) auf
einem Programm mit Quellcode- UND Modul-IMPORT == Python-Tree-Walker (stdout).
"""
import contextlib
import io
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    base = _ROOT / "rust" / "gb_runtime" / "target"
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


_GBRT = _find_gbrt()
pytestmark = pytest.mark.skipif(
    _GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")


def _py_merge(main: Path) -> str:
    from gamebasic.preprocess import process
    src = main.read_text(encoding="utf-8")
    merged, _ = process(src, main.parent, file_label=main.name)
    return merged.replace("\r\n", "\n")


def _rs_merge(main: Path):
    out = subprocess.run([str(_GBRT), "--preprocess", str(main)],
                         capture_output=True, text=True, encoding="utf-8")
    return out.returncode, out.stdout.replace("\r\n", "\n")


def _tw_run(main: Path) -> str:
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.interpreter import Interpreter
    from gamebasic.preprocess import process
    src = main.read_text(encoding="utf-8")
    prepped, _ = process(src, main.parent, file_label=main.name)
    ast = Parser(Lexer(prepped).tokenize()).parse()
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        Interpreter().run(ast)
    return buf.getvalue().replace("\r\n", "\n")


def _runsrc(main: Path):
    out = subprocess.run([str(_GBRT), "--runsrc", str(main)],
                         capture_output=True, text=True, encoding="utf-8")
    return out.returncode, out.stdout.replace("\r\n", "\n")


def _write(d: Path, name: str, content: str) -> Path:
    p = d / name
    p.write_text(content, encoding="utf-8")
    return p


def test_merge_source_import(tmp_path):
    _write(tmp_path, "helper.gb",
           "FUNCTION dbl(n AS INTEGER) AS INTEGER\n  RETURN n * 2\nEND FUNCTION\n")
    main = _write(tmp_path, "main.gb", 'IMPORT "helper.gb"\nPRINT dbl(21)\n')
    rc, rs = _rs_merge(main)
    assert rc == 0
    assert rs == _py_merge(main)


def test_merge_module_import(tmp_path):
    main = _write(tmp_path, "main.gb",
                  'IMPORT "vec2"\nDIM v AS VEC2\nv = VEC2_NEW(3.0, 4.0)\nPRINT VEC2_LENGTH(v)\n')
    rc, rs = _rs_merge(main)
    assert rc == 0
    assert rs == _py_merge(main)


def test_merge_module_import_alias(tmp_path):
    main = _write(tmp_path, "main.gb", 'IMPORT "json" AS j\nPRINT 1\n')
    rc, rs = _rs_merge(main)
    assert rc == 0
    assert rs == _py_merge(main)


def test_merge_nested_and_duplicate(tmp_path):
    _write(tmp_path, "util.gb", "CONST K AS INTEGER = 7\n")
    _write(tmp_path, "helper.gb", 'IMPORT "util.gb"\nFUNCTION f() AS INTEGER\n  RETURN K\nEND FUNCTION\n')
    # main importiert helper UND util -- util ist via helper schon gesehen.
    main = _write(tmp_path, "main.gb",
                  'IMPORT "helper.gb"\nIMPORT "util.gb"\nPRINT f()\n')
    rc, rs = _rs_merge(main)
    assert rc == 0
    assert rs == _py_merge(main)


def test_merge_trailing_comment_on_import(tmp_path):
    _write(tmp_path, "helper.gb", "CONST Z AS INTEGER = 1\n")
    main = _write(tmp_path, "main.gb", "IMPORT \"helper.gb\"   ' lade helfer\nPRINT Z\n")
    rc, rs = _rs_merge(main)
    assert rc == 0
    assert rs == _py_merge(main)


def test_missing_import_errors_both(tmp_path):
    from gamebasic.preprocess import process
    from gamebasic.errors import LexerError
    main = _write(tmp_path, "main.gb", 'IMPORT "nichtda.gb"\nPRINT 1\n')
    rc, _ = _rs_merge(main)
    assert rc != 0
    with pytest.raises(LexerError):
        process(main.read_text(encoding="utf-8"), main.parent, file_label="main.gb")


def test_e2e_runsrc_with_imports(tmp_path):
    _write(tmp_path, "mathlib.gb",
           "FUNCTION sq(n AS INTEGER) AS INTEGER\n  RETURN n * n\nEND FUNCTION\n")
    main = _write(tmp_path, "main.gb",
                  'IMPORT "mathlib.gb"\nIMPORT "vec2"\n'
                  'DIM v AS VEC2\nv = VEC2_NEW(6.0, 8.0)\n'
                  'PRINT sq(9)\nPRINT VEC2_LENGTH(v)\n')
    rc, rs = _runsrc(main)
    assert rc == 0, f"runsrc Exit {rc}"
    assert rs == _tw_run(main)
