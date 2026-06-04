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


# Hinweis: `run_vm`, `run_native` und `run_all` sind seit dem Entfernen der
# Python-/Cython-Bytecode-VMs **Aliase auf den Tree-Walker**. Es gibt nur noch
# zwei Ausfuehrungspfade: Tree-Walker (Python, Referenz) und die native Runtime
# `gbrt` (Rust, Produktion). Die Compiler-/Bytecode-Abdeckung gegen `gbrt`
# liefert der dedizierte Paritaets-Sweep in `test_gbrt_parity.py`. Die Aliase
# bleiben, damit die ~550 bestehenden Tests unveraendert weiterlaufen.

@pytest.fixture
def run_vm(run_gb):
    """Alias auf den Tree-Walker (frueher Python-VM -- entfernt)."""
    return run_gb


@pytest.fixture
def run_native(run_gb):
    """Alias auf den Tree-Walker (frueher Cython-VM -- entfernt)."""
    return run_gb


@pytest.fixture
def run_all(run_gb):
    """Alias auf den Tree-Walker (frueher 3-Pfad-Bit-Identitaet). Die
    Identitaet gegen die native Runtime prueft `test_gbrt_parity.py`.

        def test_x(run_all):
            assert run_all('PRINT 1 + 2') == "3\\n"
    """
    return run_gb


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
