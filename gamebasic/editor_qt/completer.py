"""Auto-Completion-Quelle fuer den Code-Editor.

Liefert eine sortierte Vorschlagsliste aus:
- Sprach-Schluesselwoertern (DIM, FOR, IF, ...),
- registrierten Built-ins (BUILTINS / GRAPHICS_BUILTINS),
- Konstanten (Farben, KEY_*, PI),
- Snippet-Triggern (Anzeige + Bodyhinweis im Detail-Text).

Im CodeEditor: ein QCompleter mit dieser Liste, getriggert durch
Strg+Leertaste oder automatisch ab 2 Zeichen.
"""
from __future__ import annotations

from .snippets import SNIPPETS


KEYWORDS = sorted({
    "DIM", "AS", "IF", "THEN", "ELSE", "ELSEIF", "ELIF", "END", "WHILE", "WEND",
    "FOR", "TO", "STEP", "NEXT", "SUB", "FUNCTION", "RETURN", "CLASS",
    "NEW", "EXTENDS", "STRUCT", "PRINT", "INPUT", "AND", "OR", "NOT",
    "MOD", "TRUE", "FALSE", "CONST", "BREAK", "CONTINUE", "ARRAY", "OF",
    "INTEGER", "FLOAT", "STRING", "BOOLEAN", "IMAGE", "SOUND", "FILE",
    "MAP", "REM", "PI",
    "TRY", "CATCH", "THROW", "IMPORT", "BYREF",
    "BAND", "BOR", "BXOR", "BNOT", "SHL", "SHR",
    "TUPLE", "WITH", "STATIC", "FUNCREF", "IN", "WHERE", "PROPERTY",
    "ENUM", "SELECT", "CASE", "REPEAT", "UNTIL", "DATA", "READ", "RESTORE",
})


def collect_builtins() -> list[str]:
    """Sammelt alle Built-in-Namen (uppercase) fuer Editor-Support.

    Quellen:
    - Python-Registry (`BUILTINS`/`GRAPHICS_BUILTINS`) -- die im Tree-Walker
      implementierten Builtins.
    - `BUILTIN_DOCS` -- enthaelt auch **gbrt-only**-Builtins (nur in der nativen
      Runtime implementiert, nicht mehr im Tree-Walker). So bekommen Editor-
      Highlighting/Completion sie trotzdem; die Ausfuehrung macht gbrt.
    """
    names: set[str] = set()
    try:
        from ..interpreter import BUILTINS, GRAPHICS_BUILTINS
        names.update(BUILTINS.keys())
        names.update(GRAPHICS_BUILTINS.keys())
    except Exception:
        pass
    try:
        from .builtin_docs import BUILTIN_DOCS
        names.update(BUILTIN_DOCS.keys())
    except Exception:
        pass
    return sorted(n.upper() for n in names)


def collect_constants() -> list[str]:
    """Liefert KEY_*, Farb-Konstanten und PI als upper-case Liste."""
    out: list[str] = ["PI"]
    try:
        from ..graphics import COLORS as _C, KEYS as _K
        out.extend(sorted(n.upper() for n in _C))
        out.extend(sorted(n.upper() for n in _K))
    except Exception:
        pass
    return out


def all_completions() -> list[str]:
    """Vereinte, sortierte, dedup'te Liste fuer den Completer."""
    pool: set[str] = set(KEYWORDS)
    pool.update(collect_builtins())
    pool.update(collect_constants())
    # Snippet-Trigger sind lowercase, kollidieren nicht mit den
    # uppercase-Keywords -- der User merkt am Casing, was es ist.
    pool.update(SNIPPETS.keys())
    return sorted(pool, key=lambda s: (s.lower(), s))
