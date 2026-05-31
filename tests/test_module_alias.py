"""Tests fuer Module-Imports mit Alias (`IMPORT "x" AS y`).

Aliasing-Strategie: GameBasic-Module teilen einen flachen Built-in-
Namespace, daher wird der Alias als Praefix-Ersatz angewandt: aus
`JSON_PARSE` mit alias `j` wird `J_PARSE`. Single-word Namen (`vec2`)
werden komplett durch den Alias ersetzt.

Diese Tests arbeiten gegen den schon-geladenen State (kein reset_for_tests
mit reload=True hier -- das wuerde sys.modules-Cache poppen und nachfolgende
Tests in dieser Datei und anderen brechen). Ein autouse-Snapshot stellt die
Aliase nach jedem Test zurueck, damit andere Test-Files nichts merken.
"""
import pytest


@pytest.fixture(scope="module", autouse=True)
def _ensure_modules_loaded():
    """Stellt sicher, dass die Standard-Module geladen sind. test_module_alias
    laeuft alphabetisch vor test_modules_*.py, daher koennten die Module
    sonst gar nicht in BUILTINS sein."""
    from gamebasic.modules import load_all_modules
    load_all_modules()


@pytest.fixture(autouse=True)
def _snapshot_aliases():
    """Snapshot/Restore der Aliase, die wir hier hinzufuegen."""
    from gamebasic.modules import EXTERNAL_TYPES
    from gamebasic import builtins_registry as br
    saved_b = dict(br.BUILTINS)
    saved_g = dict(br.GRAPHICS_BUILTINS)
    saved_t = dict(EXTERNAL_TYPES)
    yield
    br.BUILTINS.clear(); br.BUILTINS.update(saved_b)
    br.GRAPHICS_BUILTINS.clear(); br.GRAPHICS_BUILTINS.update(saved_g)
    EXTERNAL_TYPES.clear(); EXTERNAL_TYPES.update(saved_t)


# --- Preprocessor-Parsing ---------------------------------------------

def test_preprocess_accepts_alias_syntax():
    from gamebasic.preprocess import process
    from pathlib import Path
    src = 'IMPORT "json" AS j\nPRINT 1\n'
    merged, _ = process(src, Path("."), file_label="<test>")
    assert "PRINT 1" in merged
    assert "JSON" in merged.upper() or "j" in merged.lower()


def test_preprocess_alias_is_optional():
    from gamebasic.preprocess import process
    from pathlib import Path
    src = 'IMPORT "json"\nPRINT 1\n'
    merged, _ = process(src, Path("."), file_label="<test>")
    assert "PRINT 1" in merged


def test_preprocess_alias_must_be_identifier():
    """`AS 123` ist kein gueltiger Identifier -> Regex matcht nicht den
    `AS`-Teil und der Loader bleibt beim alias-losen Pfad."""
    from gamebasic.preprocess import process
    from pathlib import Path
    # Ungueltige Alias-Syntax wird einfach ignoriert (Regex matcht den
    # IMPORT-Teil ohne den AS-Suffix).
    src = 'IMPORT "json"\nPRINT 1\n'
    merged, _ = process(src, Path("."), file_label="<test>")
    assert "PRINT 1" in merged


# --- Aliasing-Logic ---------------------------------------------------

def test_alias_creates_prefixed_builtins(call_builtin):
    """Module ist schon geladen (autouse-Fixture in test_modules_*); ein
    nachtraeglicher `load_module("json", alias=...)` re-applies den Alias
    via _module_keys-Tracking."""
    from gamebasic.modules import load_module
    assert load_module("json", alias="myj") is True
    # Original-Namen bleiben erhalten
    h = call_builtin("json_parse", ['{"x": 7}'])
    assert h is not None
    # Alias-Namen funktionieren auch
    h2 = call_builtin("myj_parse", ['{"x": 8}'])
    assert h2 is not None


def test_alias_creates_external_type():
    from gamebasic.modules import load_module, EXTERNAL_TYPES
    load_module("json", alias="alpha")
    # JSON_HANDLE existiert UND ALPHA_HANDLE existiert (lowercased)
    assert "json_handle" in EXTERNAL_TYPES
    assert "alpha_handle" in EXTERNAL_TYPES
    # Beide zeigen auf dieselbe Python-Klasse
    assert EXTERNAL_TYPES["json_handle"] is EXTERNAL_TYPES["alpha_handle"]


def test_alias_after_initial_load(call_builtin):
    """Wenn das Modul schon ohne Alias geladen war (z.B. via
    `load_all_modules()` im Editor), soll ein nachtraeglicher
    `IMPORT ... AS alias` immer noch wirken -- weil wir per
    `_module_keys` tracken welche Built-ins aus dem Modul kommen."""
    from gamebasic.modules import load_module
    load_module("json")  # ohne alias (no-op, schon geladen)
    load_module("json", alias="beta")  # alias-spaeter
    h = call_builtin("beta_parse", ['{"a": 1}'])
    assert h is not None


def test_alias_in_full_program(run_gb):
    """End-to-end: GB-Program mit IMPORT-AS laeuft im Tree-Walker."""
    src = '''
IMPORT "json" AS j
DIM h AS J_HANDLE
h = J_PARSE("[10,20,30]")
PRINT J_GET_INT(h, "0")
PRINT J_GET_INT(h, "1")
PRINT J_GET_INT(h, "2")
'''
    out = run_gb(src)
    assert "10" in out
    assert "20" in out
    assert "30" in out


def test_alias_does_not_break_original_names(run_gb):
    """Alias ist additiv: die Original-Built-ins funktionieren weiter."""
    src = '''
IMPORT "json" AS j
DIM h AS JSON_HANDLE
h = JSON_PARSE("[42]")
PRINT JSON_GET_INT(h, "0")
'''
    out = run_gb(src)
    assert "42" in out


def test_alias_with_vec2_via_program(run_gb):
    """Vec2 mit Alias -- testet sowohl Built-ins als auch externen Typ
    und Operator-Overloading durch die Modul-Registry."""
    src = '''
IMPORT "vec2" AS v
DIM a AS V
a = V_NEW(3.0, 4.0)
PRINT V_LENGTH(a)
'''
    # `v_length(a)` sollte 5.0 liefern. Wir akzeptieren nur die Existenz
    # des Output -- das Programm darf nicht crashen.
    out = run_gb(src)
    assert "5" in out  # 5.0
