"""Tests fuer die LSP-Feature-Logik (reine Funktionen, ohne Transport)."""
from gamebasic.lsp import features as F


SRC = (
    "' Spieler-Klasse\n"           # 1
    "CLASS Player\n"               # 2
    "    DIM hp AS INTEGER\n"      # 3
    "    SUB Init()\n"            # 4
    "        Self.hp = 100\n"     # 5
    "    END SUB\n"              # 6
    "END CLASS\n"               # 7
    "FUNCTION add(a AS INTEGER, b AS INTEGER) AS INTEGER\n"  # 8
    "    RETURN a + b\n"          # 9
    "END FUNCTION\n"            # 10
    "DIM result AS INTEGER\n"     # 11
    "result = add(1, 2)\n"       # 12
)


# --------------------------------------------------------------- word_at

def test_word_at_basic():
    # Zeile 12 (idx 11): "result = add(1, 2)"; Cursor auf "add"
    word, a, b = F.word_at(SRC, 11, 10)
    assert word == "add"


def test_word_at_empty_on_space():
    word, _, _ = F.word_at(SRC, 11, 7)   # auf dem "="
    assert word == ""


# --------------------------------------------------------------- diagnostics

def test_diagnostics_clean():
    assert F.diagnostics(SRC, None) == []


def test_diagnostics_syntax_error():
    bad = "DIM x AS\n"   # unvollstaendiges DIM
    diags = F.diagnostics(bad, None)
    assert len(diags) == 1
    assert diags[0]["severity"] == 1
    assert diags[0]["source"] == "gamebasic"
    assert "range" in diags[0]


# --------------------------------------------------------------- completion

def test_completion_prefix_filters():
    # Praefix "PRI" -> nur Labels, die damit beginnen; PRINT dabei.
    items = F.completions("PRI", 0, 3)
    labels = [it["label"] for it in items]
    assert labels                                  # nicht leer
    assert all(l.lower().startswith("pri") for l in labels)
    assert any(l.upper() == "PRINT" for l in labels)


def test_completion_includes_user_symbols():
    items = F.completions(SRC + "Pl", 12, 2)
    labels = [it["label"] for it in items]
    assert "Player" in labels


def test_completion_includes_builtins():
    items = F.completions("PRIN", 0, 4)
    labels = [it["label"].upper() for it in items]
    assert "PRINT" in labels


# --------------------------------------------------------------- hover

def test_hover_builtin():
    h = F.hover("DIM x AS INTEGER\nx = ABS(-5)\n", 1, 5)   # "ABS"
    assert h is not None
    assert "value" in h["contents"]
    assert "ABS" in h["contents"]["value"].upper()


def test_hover_user_function():
    h = F.hover(SRC, 11, 10)        # "add" in result = add(...)
    assert h is not None
    assert "add" in h["contents"]["value"]


def test_hover_none_on_blank():
    assert F.hover(SRC, 0, 0) is None or "Player" not in str(F.hover(SRC, 0, 0))


# --------------------------------------------------------------- definition

def test_definition_jumps_to_decl():
    d = F.definition(SRC, 11, 10)   # "add" -> FUNCTION add (Zeile 8 -> idx 7)
    assert d is not None
    assert d["line"] == 7


def test_definition_none_for_unknown():
    assert F.definition("PRINT 1\n", 0, 2) is None


# --------------------------------------------------------------- references

def test_references_finds_all():
    refs = F.references(SRC, 7, 9)   # "add" in der FUNCTION-Deklaration
    lines = sorted(r["line"] for r in refs)
    # Deklaration (idx7) + Aufruf (idx11)
    assert 7 in lines and 11 in lines


# --------------------------------------------------------------- document symbols

def test_document_symbols_hierarchy():
    syms = F.document_symbols(SRC)
    by_name = {s["name"]: s for s in syms}
    assert "Player" in by_name
    assert "add" in by_name
    # Init ist Kind von Player
    player = by_name["Player"]
    child_names = [c["name"] for c in player["children"]]
    assert "Init" in child_names
    assert player["kind"] == F.SK_CLASS


def test_document_symbols_enum_top_level():
    src = "ENUM State\n  A = 0\n  B = 1\nEND ENUM\n"
    syms = F.document_symbols(src)
    assert any(s["name"] == "State" and s["kind"] == F.SK_ENUM for s in syms)
