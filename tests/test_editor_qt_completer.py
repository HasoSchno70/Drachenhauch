"""Tests fuer die Auto-Completion-Quelle.

Schwerpunkt: lokale Buffer-Symbole (SUB/FUNCTION/DIM/CLASS/Parameter)
landen in der Completion -- vorher kannte der Completer nur statische
Keywords/Builtins/Konstanten.
"""
from drachenhauch.editor_qt.completer import (
    all_completions,
    local_definition_names,
)


def test_local_names_include_defs_and_params():
    src = (
        "SUB greet(name$)\n"
        "  PRINT name$\n"
        "END SUB\n"
        "FUNCTION add(a, b)\n"
        "  RETURN a + b\n"
        "END FUNCTION\n"
        "DIM score AS INTEGER\n"
        "CLASS Player\n"
        "END CLASS\n"
    )
    names = set(local_definition_names(src))
    assert {"greet", "add", "score", "Player"} <= names      # Definitionen
    assert {"name", "a", "b"} <= names                       # Parameter


def test_local_names_preserve_casing_and_dedup():
    src = "FUNCTION myFunc()\nEND FUNCTION\nDIM myFunc\n"
    names = local_definition_names(src)
    assert names.count("myFunc") == 1          # erstes Vorkommen, keine Dublette
    assert "myFunc" in names                   # Casing wie geschrieben


def test_local_names_empty_and_robust():
    assert local_definition_names("") == []
    # Kaputter/teiltgetippter Code darf nie werfen.
    assert isinstance(local_definition_names("SUB \nDIM"), list)


def test_static_completions_present():
    pool = all_completions()
    assert "DIM" in pool and "FOR" in pool
    # Builtins/Konstanten sind dabei (Stichprobe).
    assert any(s == "PI" for s in pool)
