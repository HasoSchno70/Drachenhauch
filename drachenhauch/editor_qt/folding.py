"""Code-Folding-Region-Scanner fuer Drachenhauch.

Liefert Liste von Klapp-Regionen `(start_line, end_line, kind)`, jeweils
1-basierte Zeilennummern. Verwendet einen einfachen Stack-basierten
Pass ueber den Quelltext -- keine volle Lex-Pipeline noetig.

Erkannte Block-Konstrukte:
    SUB ... END SUB
    FUNCTION ... END FUNCTION
    CLASS ... END CLASS
    STRUCT ... END STRUCT
    IF ... END IF
    SELECT CASE ... END SELECT
    WHILE ... WEND
    FOR ... NEXT
    REPEAT ... UNTIL ...
    TRY ... END TRY
    ENUM ... END ENUM
    PROPERTY GET/SET ... END PROPERTY
    WITH ... END WITH
"""
from __future__ import annotations

from .symbols import _strip_inline_comment


# Mapping Opener-Kind -> Set akzeptierter Closer-Tokens.
# Multi-Token-Closer werden zur Erkennung als String mit Whitespace
# normalisiert ("end if", "end sub", etc.).
_CLOSERS: dict[str, frozenset[str]] = {
    "sub":      frozenset({"end sub"}),
    "function": frozenset({"end function"}),
    "class":    frozenset({"end class"}),
    "struct":   frozenset({"end struct"}),
    "if":       frozenset({"end if"}),
    "select":   frozenset({"end select"}),
    "while":    frozenset({"wend"}),
    "for":      frozenset({"next"}),     # NEXT oder NEXT i, beides erlaubt
    "repeat":   frozenset({"until"}),    # UNTIL <bedingung>
    "try":      frozenset({"end try"}),
    "enum":     frozenset({"end enum"}),
    "property": frozenset({"end property"}),
    "with":     frozenset({"end with"}),
}


def _opener_kind(upper: str) -> str | None:
    """Liefert den Opener-Kind fuer eine getrimmte upper-case Zeile, oder None.

    Wir akzeptieren nur Zeilen, die mit dem Opener-Keyword beginnen UND
    in denen das Konstrukt nicht direkt einzeilig wieder geschlossen wird
    (z.B. `IF x THEN PRINT 1` ist kein Faltbares).
    """
    if upper.startswith("SUB ") or upper == "SUB":
        return "sub"
    if upper.startswith("FUNCTION ") or upper == "FUNCTION":
        return "function"
    if upper.startswith("CLASS ") or upper == "CLASS":
        return "class"
    if upper.startswith("STRUCT ") or upper == "STRUCT":
        return "struct"
    if upper.startswith("SELECT CASE ") or upper == "SELECT CASE":
        return "select"
    if upper.startswith("WHILE ") or upper == "WHILE":
        return "while"
    if upper.startswith("FOR ") or upper == "FOR":
        return "for"
    if upper == "REPEAT":
        return "repeat"
    if upper == "TRY":
        return "try"
    if upper.startswith("ENUM ") or upper == "ENUM":
        return "enum"
    if upper.startswith("PROPERTY ") or upper == "PROPERTY":
        return "property"
    if upper.startswith("WITH ") or upper == "WITH":
        return "with"
    if upper.startswith("IF ") and upper.endswith(" THEN"):
        # `IF x THEN PRINT y` (single-line) erkennen wir, indem nach THEN
        # nichts mehr kommt -- das stripped Upper endet mit " THEN".
        return "if"
    return None


def _is_closer(upper: str, kind: str) -> bool:
    """Prueft ob die Zeile (upper-case, getrimmt) einen Block des
    angegebenen Kinds schliesst."""
    if kind == "for":
        # NEXT oder NEXT <var>
        return upper == "NEXT" or upper.startswith("NEXT ")
    if kind == "repeat":
        return upper == "UNTIL" or upper.startswith("UNTIL ")
    closers = _CLOSERS.get(kind, frozenset())
    return upper.lower() in closers


def scan(source: str) -> list[tuple[int, int, str]]:
    """Liefert die Fold-Regionen `(start_line, end_line, kind)` fuer den
    gesamten Quelltext. Zeilen sind 1-basiert; `end_line` ist die Zeile,
    auf der der Block-Closer steht.

    Verschachtelte Bloecke werden korrekt verarbeitet -- ein Stack haelt
    aktuell offene Opener. Wenn ein Closer nicht zum Top-of-Stack passt,
    wird der Stack defensiv aufgeschoben (kein Crash bei kaputtem Code).
    """
    out: list[tuple[int, int, str]] = []
    stack: list[tuple[int, str]] = []   # (line_number, kind)
    for ln, raw in enumerate(source.split("\n"), start=1):
        # Review-Fund: ein Trailing-Kommentar (z.B. "IF x > 0 THEN ' positive"
        # oder "END SUB ' cleanup done") wurde bisher NICHT abgeschnitten,
        # bevor die Zeile klassifiziert wurde -- weder Opener- noch Closer-
        # Erkennung tolerierten dadurch einen Kommentar am Zeilenende (die
        # Zeile endete dann z.B. auf "' POSITIVE" statt " THEN", oder
        # "END SUB ' CLEANUP DONE" traf den exakten Closer-String "end sub"
        # nicht mehr) -- der gesamte Block bekam still keinen Fold-Pfeil.
        # `_strip_inline_comment` (aus symbols.py) kennt bereits die
        # Quoted-String-Ausnahme (ein `'` INNERHALB eines "..."-Literals ist
        # kein Kommentar-Start).
        stripped = _strip_inline_comment(raw).strip()
        if not stripped:
            continue
        upper = stripped.upper()

        # Closer zuerst pruefen (z.B. "END IF" enthaelt nicht das Opener-IF).
        if stack:
            top_line, top_kind = stack[-1]
            if _is_closer(upper, top_kind):
                if ln > top_line:
                    out.append((top_line, ln, top_kind))
                stack.pop()
                continue
            # Closer-Pattern eines aelteren Stack-Eintrags? Zur Robustheit
            # erlauben wir, mit fehlenden Closern den Stack zu trimmen.
            for i in range(len(stack) - 1, -1, -1):
                _, k = stack[i]
                if _is_closer(upper, k):
                    # Verwerfe alles ueber Index i, schliesse i.
                    while len(stack) > i:
                        ln_old, kind_old = stack.pop()
                        if i == len(stack):
                            out.append((ln_old, ln, kind_old))
                    break
            else:
                # Kein passender Closer im Stack -- Zeile ist normaler Code.
                pass

        kind = _opener_kind(upper)
        if kind is not None:
            stack.append((ln, kind))
    return out
