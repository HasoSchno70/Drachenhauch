"""Auto-Completion-Quelle fuer den Code-Editor.

Liefert eine sortierte Vorschlagsliste aus:
- Sprach-Schluesselwoertern (DIM, FOR, IF, ...; aus `tokens.KEYWORDS`),
- registrierten Built-ins (BUILTINS / GRAPHICS_BUILTINS),
- Konstanten (Farben, KEY_*, PI),
- Snippet-Triggern (Anzeige + Bodyhinweis im Detail-Text).

Im CodeEditor: ein QCompleter mit dieser Liste, getriggert durch
Strg+Leertaste oder automatisch ab 2 Zeichen.
"""
from __future__ import annotations

from ..tokens import KEYWORDS as _KEYWORD_TABELLE
from .snippets import SNIPPETS


# Sprach-Schluesselwoerter -- ABGELEITET aus `tokens.KEYWORDS`, derselben
# Tabelle, die auch der Lexer benutzt (und die `lexer.rs` in dhrt spiegelt).
#
# Hier stand frueher eine handgepflegte Aufzaehlung. Sie war um drei
# Schluesselwoerter zurueckgeblieben: `IS` (der Laufzeit-Typtest `x IS Hund`
# / `x IS NOT NIL`), `FINALLY` und `PRIVATE` -- gerade das juengste
# Sprachfeature schlug der Editor also nicht vor. Abgeleitet kann das nicht
# mehr passieren.
_EXTRAS = frozenset({
    "REM",   # Kommentar-Marker, kein Keyword-Token (der Lexer verschluckt ihn)
    "PI",    # vorregistrierte Konstante, kein Keyword -- aber tippt man
})
KEYWORDS = sorted({k.upper() for k in _KEYWORD_TABELLE} | _EXTRAS)


def collect_builtins() -> list[str]:
    """Sammelt alle Built-in-Namen (uppercase) fuer Editor-Support.

    Quellen:
    - Python-Registry (`BUILTINS`/`GRAPHICS_BUILTINS`) -- die im Tree-Walker
      implementierten Builtins.
    - `BUILTIN_DOCS` -- enthaelt auch **dhrt-only**-Builtins (nur in der nativen
      Runtime implementiert, nicht mehr im Tree-Walker). So bekommen Editor-
      Highlighting/Completion sie trotzdem; die Ausfuehrung macht dhrt.
    """
    names: set[str] = set()
    try:
        # Builtin-Namen aus dem eingefrorenen dhrt-Metadaten-Index (Stufe B) --
        # keine Laufzeit-Abhaengigkeit vom (laengst entfernten) Tree-Walker.
        from .dhrt_meta import builtin_names_upper
        names.update(builtin_names_upper())
    except Exception:
        pass
    try:
        from .builtin_docs import BUILTIN_DOCS
        names.update(n.upper() for n in BUILTIN_DOCS)
    except Exception:
        pass
    return sorted(names)


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


def local_definition_names(source: str) -> list[str]:
    """User-eigene Symbole des aktuellen Buffers fuer die Completion:
    `SUB`/`FUNCTION`/`CLASS`/`STRUCT`/`ENUM`/`CONST`/`DIM` + Parameter.

    Casing wird wie geschrieben erhalten (der eigene Funktionsname soll
    so vorgeschlagen werden, wie man ihn definiert hat). Pro Name nur ein
    Eintrag (erstes Vorkommen gewinnt). Bei Scan-Fehlern leere Liste --
    Completion darf nie das Tippen stoeren."""
    try:
        from .symbols import scan_definitions
    except Exception:
        return []
    seen: dict[str, str] = {}
    try:
        for d in scan_definitions(source):
            key = d.name.lower()
            if key not in seen:
                seen[key] = d.name
    except Exception:
        return []
    return list(seen.values())
