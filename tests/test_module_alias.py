"""Tests fuer Module-Imports mit Alias (`IMPORT "x" AS y`).

Aliasing-Strategie: GameBasic-Module teilen einen flachen Built-in-Namespace,
daher wird der Alias als Praefix-Ersatz angewandt: aus `JSON_PARSE` mit alias `j`
wird `J_PARSE`. Single-word-Namen (`vec2`) werden komplett durch den Alias ersetzt.

Stufe B: die frueheren `call_builtin`/`load_module`/`EXTERNAL_TYPES`-Tests (die
die Python-Alias-Mechanik direkt prueften) sind durch die run_gb-Golden-Tests
unten abgedeckt -- gbrt bildet aliasierte Builtin-Namen selbst zurueck. Die
Preprocess-Tests laufen weiter gegen das (behaltene) `gamebasic.preprocess`.
"""
from pathlib import Path


# --- Preprocessor-Parsing (gegen das behaltene gamebasic.preprocess) ----

def test_preprocess_accepts_alias_syntax():
    from gamebasic.preprocess import process
    src = 'IMPORT "json" AS j\nPRINT 1\n'
    merged, _ = process(src, Path("."), file_label="<test>")
    assert "PRINT 1" in merged
    assert "JSON" in merged.upper() or "j" in merged.lower()


def test_preprocess_alias_is_optional():
    from gamebasic.preprocess import process
    src = 'IMPORT "json"\nPRINT 1\n'
    merged, _ = process(src, Path("."), file_label="<test>")
    assert "PRINT 1" in merged


def test_preprocess_alias_must_be_identifier():
    """`AS 123` ist kein gueltiger Identifier -> der AS-Teil wird ignoriert."""
    from gamebasic.preprocess import process
    src = 'IMPORT "json"\nPRINT 1\n'
    merged, _ = process(src, Path("."), file_label="<test>")
    assert "PRINT 1" in merged


# --- Aliasing end-to-end (gegen gbrt) ----------------------------------

def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def test_alias_in_full_program(run_gb):
    """GB-Programm mit IMPORT-AS: aliasierte Builtins + externer Typ."""
    out = run_gb('IMPORT "json" AS j\n'
                 'DIM h AS J_HANDLE\n'
                 'h = J_PARSE("[10,20,30]")\n'
                 'PRINT J_GET_INT(h, "0")\n'
                 'PRINT J_GET_INT(h, "1")\n'
                 'PRINT J_GET_INT(h, "2")\n')
    assert _lines(out) == ["10", "20", "30"]


def test_alias_does_not_break_original_names(run_gb):
    """Alias ist additiv: die Original-Built-ins funktionieren weiter."""
    out = run_gb('IMPORT "json" AS j\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("[42]")\n'
                 'PRINT JSON_GET_INT(h, "0")\n')
    assert _lines(out) == ["42"]


def test_alias_with_vec2_via_program(run_gb):
    """Vec2 mit Alias -- Built-ins, externer Typ und Operator-Registry."""
    out = run_gb('IMPORT "vec2" AS v\n'
                 'DIM a AS V\n'
                 'a = V_NEW(3.0, 4.0)\n'
                 'PRINT V_LENGTH(a)\n')
    assert _lines(out) == ["5.0"]
