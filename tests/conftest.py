"""Gemeinsame Pytest-Helpers fuer GameBasic-Tests."""
import io
import os
import sys
import contextlib
from pathlib import Path

import pytest


# Sicherstellen, dass das Projekt-Root im sys.path ist (fuer 'gamebasic' Import).
_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))


@pytest.fixture
def run_gb():
    """Fuehrt einen GB-Quelltext im Tree-Walker aus und gibt den stdout zurueck.

    Beispiel:
        def test_print(run_gb):
            assert run_gb('PRINT "hi"') == "hi\\n"
    """
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.interpreter import Interpreter
    from gamebasic.preprocess import process

    def _run(source: str, base: Path | None = None) -> str:
        if base is None:
            base = _ROOT
        prepped, _ = process(source, base, file_label="<test>")
        tokens = Lexer(prepped).tokenize()
        ast = Parser(tokens).parse()
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            Interpreter().run(ast)
        return buf.getvalue()

    return _run


@pytest.fixture
def run_vm():
    """Fuehrt einen GB-Quelltext im Python-VM aus und gibt stdout zurueck."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler
    from gamebasic.vm import VM
    from gamebasic.preprocess import process

    def _run(source: str, base: Path | None = None) -> str:
        if base is None:
            base = _ROOT
        prepped, _ = process(source, base, file_label="<test>")
        tokens = Lexer(prepped).tokenize()
        ast = Parser(tokens).parse()
        module = Compiler().compile(ast)
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            VM().run(module)
        return buf.getvalue()

    return _run


@pytest.fixture
def run_native():
    """Fuehrt einen GB-Quelltext in der nativen Cython-VM aus und gibt stdout
    zurueck. Skippt den Test, wenn die `.pyd` nicht gebaut ist."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler
    from gamebasic.preprocess import process
    try:
        from gamebasic.vm_native import VM as NativeVM  # type: ignore
    except ImportError:
        NativeVM = None

    def _run(source: str, base: Path | None = None) -> str:
        if NativeVM is None:
            pytest.skip("native VM (vm_native.pyd) nicht gebaut")
        if base is None:
            base = _ROOT
        prepped, _ = process(source, base, file_label="<test>")
        tokens = Lexer(prepped).tokenize()
        ast = Parser(tokens).parse()
        module = Compiler().compile(ast)
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            NativeVM().run(module)
        return buf.getvalue()

    return _run


@pytest.fixture
def run_all():
    """Fuehrt einen GB-Quelltext durch ALLE drei Pfade (Tree-Walker, Python-VM,
    native Cython-VM) und prueft, dass die Ausgaben bit-identisch sind --
    der Kern der GameBasic-Garantie. Liefert die gemeinsame Ausgabe zurueck.
    Ist die native VM nicht gebaut, werden nur Tree-Walker und Python-VM
    verglichen (kein Skip -- damit laeuft die Suite auch ohne `.pyd`).

        def test_x(run_all):
            assert run_all('PRINT 1 + 2') == "3\\n"
    """
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler
    from gamebasic.interpreter import Interpreter
    from gamebasic.vm import VM as PyVM
    from gamebasic.preprocess import process
    try:
        from gamebasic.vm_native import VM as NativeVM  # type: ignore
    except ImportError:
        NativeVM = None

    def _run_one(runner, ast_or_module):
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            runner(ast_or_module)
        return buf.getvalue()

    def _run(source: str, base: Path | None = None) -> str:
        if base is None:
            base = _ROOT
        prepped, _ = process(source, base, file_label="<test>")
        ast = Parser(Lexer(prepped).tokenize()).parse()
        module = Compiler().compile(ast)
        tw = _run_one(lambda a: Interpreter().run(a), ast)
        vm = _run_one(lambda m: PyVM().run(m), module)
        assert tw == vm, (
            f"Tree-Walker != Python-VM:\n  TW={tw!r}\n  VM={vm!r}")
        if NativeVM is not None:
            nv = _run_one(lambda m: NativeVM().run(m), module)
            assert tw == nv, (
                f"Tree-Walker != Native-VM:\n  TW={tw!r}\n  NV={nv!r}")
        return tw

    return _run


@pytest.fixture
def call_builtin():
    """Ruft eine Built-in-Funktion direkt auf, mit args als Liste."""
    from gamebasic.interpreter import BUILTINS

    def _call(name: str, args: list):
        fn = BUILTINS.get(name.lower())
        if fn is None:
            raise KeyError(f"Built-in '{name}' nicht registriert")
        return fn(args)

    return _call
