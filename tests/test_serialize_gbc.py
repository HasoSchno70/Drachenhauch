"""Tests fuer die .gbc-Serialisierung (Schritt 1 der Rust-Runtime-Migration).

Prueft, dass `serialize_module` ein verlustfrei dekodierbares, eindeutig
getyptes JSON-Objekt erzeugt -- insbesondere die INT/FLOAT/BOOL-Disambiguierung,
von der die Bit-Identitaet der Rust-VM abhaengt.
"""
import json

import pytest

from gamebasic.bytecode import COMP_MARKER
from gamebasic.compiler import Compiler
from gamebasic.lexer import Lexer
from gamebasic.parser import Parser
from gamebasic.serialize import _enc, serialize_module


def _compile(src: str):
    return Compiler().compile(Parser(Lexer(src).tokenize()).parse())


def test_enc_disambiguates_int_float_bool():
    # bool VOR int -- isinstance(True, int) ist True.
    assert _enc(True) == {"b": True}
    assert _enc(False) == {"b": False}
    # int bleibt Plain-Zahl, float wird getaggt.
    assert _enc(5) == 5
    assert _enc(5.0) == {"f": 5.0}
    assert _enc(1.85) == {"f": 1.85}
    # 1 und 1.0 sind nach dem Encoding klar unterscheidbar.
    assert _enc(1) != _enc(1.0)


def test_enc_strings_none_tuples():
    assert _enc("hi") == "hi"
    assert _enc(None) is None
    assert _enc((1, 2.0, "x")) == [1, {"f": 2.0}, "x"]
    assert _enc(COMP_MARKER) == {"comp": True}


def test_enc_nested_tuple():
    assert _enc(((1, 2), (3.5,))) == [[1, 2], [{"f": 3.5}]]


def test_enc_funcref():
    from gamebasic.interpreter import _FuncRef
    assert _enc(_FuncRef("foo")) == {"funcref": "foo"}


def test_enc_rejects_runtime_handle():
    class _Bogus:
        pass
    with pytest.raises(TypeError):
        _enc(_Bogus())


def test_serialize_module_is_json_roundtrippable():
    module = _compile(
        "DIM a AS INTEGER\n"
        "a = 0\n"
        "DIM b AS FLOAT\n"
        "b = 1.5\n"
        "WHILE a < 3\n"
        "  PRINT a\n"
        "  a = a + 1\n"
        "WEND\n"
    )
    obj = serialize_module(module)
    # Muss valides JSON sein (kein NaN/Inf in diesem Programm).
    text = json.dumps(obj)
    back = json.loads(text)
    assert back["format"] == "gbc"
    assert back["version"] == 1
    assert back["n_globals"] == module.n_globals
    assert back["main"]["is_main"] is True
    # Code ist Liste von [op, arg]-Paaren mit int-Opcodes.
    for instr in back["main"]["code"]:
        assert isinstance(instr, list) and len(instr) == 2
        assert isinstance(instr[0], int)


def test_serialize_function_metadata():
    module = _compile(
        "FUNCTION sq(n AS INTEGER) AS INTEGER\n"
        "  RETURN n * n\n"
        "END FUNCTION\n"
        "PRINT sq(7)\n"
    )
    obj = serialize_module(module)
    assert "sq" in obj["functions"]
    fn = obj["functions"]["sq"]
    assert fn["n_params"] == 1
    assert fn["is_sub"] is False
    assert fn["return_type"] == "integer"
    assert fn["local_types"][0] == "integer"


def test_lines_parallel_to_code():
    """Jede Funktion traegt ein `lines`-Array parallel zu `code` -- die native
    Runtime nutzt es fuer Laufzeitfehler mit Zeilenangabe."""
    module = _compile(
        "DIM a AS INTEGER\n"       # Zeile 1
        "a = 1\n"                  # Zeile 2
        "PRINT a\n"                # Zeile 3
    )
    obj = serialize_module(module)
    main = obj["main"]
    assert len(main["lines"]) == len(main["code"])
    # Alle gestempelten Zeilen liegen im Quell-Bereich (1..3); >=1 echte Zeile.
    nonzero = [ln for ln in main["lines"] if ln]
    assert nonzero, "keine Zeilen gestempelt"
    assert max(main["lines"]) <= 3


def test_lines_point_at_failing_statement():
    """Die Instruktion(en) eines Statements tragen dessen Quell-Zeile."""
    from gamebasic.bytecode import Op
    module = _compile(
        "DIM a AS INTEGER\n"       # 1
        "PRINT 1\n"                # 2
        "a = 10 \\ 0\n"            # 3  (Integer-Division -> INT_DIV)
    )
    fn = module.main
    # Finde die INT_DIV-Instruktion und pruefe ihre gestempelte Zeile.
    idivs = [i for i, (op, _) in enumerate(fn.code) if op == Op.INT_DIV]
    assert idivs, "kein INT_DIV emittiert"
    assert fn.lines[idivs[0]] == 3


def test_builtin_call_compiles_to_call_builtin_in_fresh_process():
    """Regression: LEN/INT/... muessen zu CALL_BUILTIN (Op 51) kompilieren,
    nicht zu LOAD_NAME + CALL_VALUE. Das haengt daran, dass serialize.py die
    BUILTINS-Registry (via interpreter.py) befuellt, BEVOR der Compiler laeuft.
    Wir starten einen frischen Prozess, der NUR gamebasic.serialize importiert
    (nicht interpreter/gbrun), um die Import-Reihenfolge echt zu testen.
    """
    import json
    import subprocess
    import sys
    import tempfile
    from pathlib import Path

    with tempfile.TemporaryDirectory() as d:
        gb = Path(d) / "p.gb"
        gbc = Path(d) / "p.gbc"
        gb.write_text("DIM a[3] AS INTEGER\nPRINT LEN(a)\n", encoding="utf-8")
        # Frischer Prozess: importiert NUR serialize. compile_file_to_gbc muss
        # interpreter selbst importieren, damit LEN als Builtin erkannt wird.
        code = (
            "import gamebasic.serialize as s\n"
            f"s.compile_file_to_gbc(r'{gb}', r'{gbc}')\n"
        )
        out = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True)
        assert out.returncode == 0, out.stderr
        obj = json.loads(gbc.read_text(encoding="utf-8"))
        ops = [op for op, _ in obj["main"]["code"]]
        assert 51 in ops, f"CALL_BUILTIN (51) fehlt, LEN wurde nicht als Builtin erkannt. ops={ops}"
        assert 54 not in ops, f"CALL_VALUE (54) vorhanden -- LEN faelschlich als Funcref. ops={ops}"


def test_float_constants_preserved_exactly():
    module = _compile("DIM x AS FLOAT\nx = 0.1\nPRINT x\n")
    obj = serialize_module(module)
    # Irgendwo im const-Pool muss 0.1 als {"f": 0.1} stehen.
    floats = [c for c in obj["main"]["constants"] if isinstance(c, dict) and "f" in c]
    assert any(c["f"] == 0.1 for c in floats)
