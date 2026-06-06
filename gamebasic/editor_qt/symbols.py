"""Symbol-Index fuer Goto-Definition / Find-References / Rename.

Stack-basierter Scanner ueber den Quelltext. Liefert:
- Definitions: Liste `(name, kind, line, col_start, col_end)`. `kind`
  ist eine der Strings: "sub", "function", "class", "struct", "const",
  "dim", "param".
- References: Liste `(name, line, col_start, col_end)` mit ALLEN
  Identifier-Vorkommen (auch Definitionen selbst).

Das ist absichtlich keyword-/comment-aware aber NICHT Lex-genau --
fuer den Editor-Use-Case (Spruenge + Refs zu User-Symbolen) reicht
die Heuristik. Praezisere Pipeline ueber den GB-Lexer waere ein
Refactor wert, falls Phase 4 das braucht.
"""
from __future__ import annotations

import re
from dataclasses import dataclass


_IDENT_RE = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\$?")
_DEF_PATTERNS = [
    # (regex, kind, group_index_for_name)
    (re.compile(r"^\s*SUB\s+([A-Za-z_][A-Za-z0-9_]*)", re.IGNORECASE), "sub", 1),
    (re.compile(r"^\s*FUNCTION\s+([A-Za-z_][A-Za-z0-9_]*)", re.IGNORECASE), "function", 1),
    (re.compile(r"^\s*CLASS\s+([A-Za-z_][A-Za-z0-9_]*)", re.IGNORECASE), "class", 1),
    (re.compile(r"^\s*STRUCT\s+([A-Za-z_][A-Za-z0-9_]*)", re.IGNORECASE), "struct", 1),
    (re.compile(r"^\s*ENUM\s+([A-Za-z_][A-Za-z0-9_]*)", re.IGNORECASE), "enum", 1),
    (re.compile(r"^\s*CONST\s+([A-Za-z_][A-Za-z0-9_]*)", re.IGNORECASE), "const", 1),
    (re.compile(r"^\s*DIM\s+([A-Za-z_][A-Za-z0-9_]*)", re.IGNORECASE), "dim", 1),
    (re.compile(r"^\s*STATIC\s+CONST\s+([A-Za-z_][A-Za-z0-9_]*)", re.IGNORECASE), "const", 1),
]

# Tokens, die KEINE User-Symbole sind und die wir bei References ueberspringen.
# Lowercase Set fuer schnellen Lookup.
_KEYWORDS_AND_BUILTINS_LOWER: set[str] = set()
def _build_kw_set() -> None:
    """Lazy-baut die Filter-Liste -- braucht den Lexer-Tokens-Lookup."""
    global _KEYWORDS_AND_BUILTINS_LOWER
    if _KEYWORDS_AND_BUILTINS_LOWER:
        return
    out: set[str] = set()
    try:
        from ..tokens import KEYWORDS
        out.update(KEYWORDS.keys())
    except Exception:
        pass
    try:
        # Builtin-Namen aus dem eingefrorenen gbrt-Metadaten-Index (Stufe B).
        from .gbrt_meta import builtin_names_lower
        out.update(builtin_names_lower())
    except Exception:
        pass
    try:
        from ..graphics import COLORS as _C, KEYS as _K
        out.update(n.lower() for n in _C)
        out.update(n.lower() for n in _K)
    except Exception:
        pass
    # Kontextuelle Pseudo-Keywords.
    out.update({"true", "false", "self", "pi", "rem", "get", "set", "new"})
    _KEYWORDS_AND_BUILTINS_LOWER = out


@dataclass
class Definition:
    name: str
    kind: str
    line: int        # 1-basiert
    col: int         # 1-basiert, Start des Namens
    col_end: int     # 1-basiert (exclusive)


@dataclass
class Reference:
    name: str
    line: int
    col: int
    col_end: int


def _strip_comment_and_strings(line: str) -> str:
    """Ersetzt Strings und Kommentare durch Leerzeichen gleicher Laenge --
    so verschiebt sich keine Spalten-Position, aber der Inhalt blockiert
    keine Identifier-Matches.

    Erkennt: doppelte Quotes (`"..."`), Kommentar mit `'` oder `REM `.
    Heuristisch -- z.B. Escape-Sequenzen werden als doppelte Quotes
    behandelt (GB nutzt `""` als Quote-Escape, ist also kompatibel).
    """
    out = []
    i = 0
    n = len(line)
    while i < n:
        ch = line[i]
        if ch == "'":
            out.append(" " * (n - i))
            break
        if ch == '"':
            out.append(" ")
            i += 1
            while i < n:
                if line[i] == '"':
                    out.append(" ")
                    i += 1
                    if i < n and line[i] == '"':  # `""`-Escape
                        out.append(" ")
                        i += 1
                        continue
                    break
                out.append(" ")
                i += 1
            continue
        if (ch in ("R", "r") and i + 3 <= n and line[i:i + 3].lower() == "rem"
                and (i + 3 == n or not (line[i + 3].isalnum() or line[i + 3] == "_"))):
            # Word-Anfang: davor entweder Zeilenanfang oder Nicht-Identifier.
            prev_ok = (i == 0) or not (line[i - 1].isalnum() or line[i - 1] == "_")
            if prev_ok:
                out.append(" " * (n - i))
                break
        out.append(ch)
        i += 1
    return "".join(out)


def scan_definitions(source: str) -> list[Definition]:
    """Liefert alle Top-Level-/Methoden-Definitionen in der Reihenfolge
    der Erscheinens.

    `DIM x AS T` und `CONST y = ...` werden mit erfasst -- so funktionieren
    Goto-Definition + Rename auch fuer Variablen.
    """
    out: list[Definition] = []
    for ln, raw in enumerate(source.split("\n"), start=1):
        cleaned = _strip_comment_and_strings(raw)
        for pat, kind, gi in _DEF_PATTERNS:
            m = pat.match(cleaned)
            if m:
                name = m.group(gi)
                out.append(Definition(
                    name=name, kind=kind, line=ln,
                    col=m.start(gi) + 1, col_end=m.end(gi) + 1,
                ))
                break
        # SUB/FUNCTION-Parameter erfassen (nach Open-Paren bis Close-Paren).
        if cleaned.lstrip().upper().startswith(("SUB ", "FUNCTION ", "PROPERTY ")):
            popen = cleaned.find("(")
            pclose = cleaned.find(")", popen + 1) if popen >= 0 else -1
            if popen >= 0 and pclose > popen:
                params_zone = cleaned[popen + 1:pclose]
                base = popen + 1
                # Parameter-Tokens: erstes Word je Komma-Segment.
                offset = base
                for seg in params_zone.split(","):
                    m = re.match(r"\s*(?:BYREF\s+|\.\.\.)?([A-Za-z_][A-Za-z0-9_]*)\$?", seg, re.IGNORECASE)
                    if m:
                        out.append(Definition(
                            name=m.group(1), kind="param", line=ln,
                            col=offset + m.start(1) + 1,
                            col_end=offset + m.end(1) + 1,
                        ))
                    offset += len(seg) + 1   # +1 fuer das Komma
    return out


def scan_references(source: str, name: str) -> list[Reference]:
    """Liefert alle Vorkommen des Identifiers `name` (case-insensitiv,
    matched ganze Worte). Strings + Kommentare werden ausgespart.

    Wortgrenze: vor und NACH dem Namen darf kein Identifier-Zeichen
    stehen. `x` matched also nicht in `xy`. Trailing-`$` (GB-Konvention
    fuer String-Variablen) wird optional akzeptiert, aber nur wenn
    der Original-Name ebenfalls auf `$` endet.
    """
    out: list[Reference] = []
    # `\bname\b` mit zusaetzlicher Lookahead-Pruefung um zu verhindern,
    # dass wir `x` in `xy` finden (weil `\b` zwischen `x` und `y` nicht
    # liegt -- aber bei MEHRZEILIGEN Patterns muessten wir defensiv
    # pruefen). Sicher ist sicher.
    name_re = re.compile(
        rf"\b{re.escape(name)}\b(?![A-Za-z0-9_$])", re.IGNORECASE,
    )
    for ln, raw in enumerate(source.split("\n"), start=1):
        cleaned = _strip_comment_and_strings(raw)
        for m in name_re.finditer(cleaned):
            out.append(Reference(
                name=m.group(0), line=ln,
                col=m.start() + 1, col_end=m.end() + 1,
            ))
    return out


def all_user_identifier_positions(source: str) -> dict[str, list[Reference]]:
    """Fuer jeden Identifier (lowercase) im Quelltext: Liste aller
    Vorkommen. Keywords/Built-ins werden ausgefiltert.
    """
    _build_kw_set()
    by_name: dict[str, list[Reference]] = {}
    for ln, raw in enumerate(source.split("\n"), start=1):
        cleaned = _strip_comment_and_strings(raw)
        for m in _IDENT_RE.finditer(cleaned):
            ident = m.group(1)
            if ident.lower() in _KEYWORDS_AND_BUILTINS_LOWER:
                continue
            key = ident.lower()
            by_name.setdefault(key, []).append(Reference(
                name=ident, line=ln,
                col=m.start(1) + 1, col_end=m.end(1) + 1,
            ))
    return by_name


def find_definition(source: str, name: str) -> Definition | None:
    """Erste Definition (in Source-Reihenfolge) zu `name`. Fuer
    Goto-Definition reicht das -- DIM in einem Scope ist die naheliegende
    Quelle, Funktionen ueberschreiben nur explizit."""
    target = name.lower()
    for d in scan_definitions(source):
        if d.name.lower() == target:
            return d
    return None


@dataclass
class Scope:
    kind: str        # "class" | "struct" | "sub" | "function" | "property"
    name: str
    line: int        # 1-basiert, Zeile der Block-Eroeffnung
    end: int         # 1-basiert, Zeile des END (oder Dateiende bei offen)


# Block-Oeffner: Prefix -> kind. PROPERTY separat (Name nach GET/SET).
_SCOPE_OPENERS = (
    ("CLASS ", "class"),
    ("STRUCT ", "struct"),
    ("SUB ", "sub"),
    ("FUNCTION ", "function"),
)
_END_KIND = {
    "END CLASS": "class", "END STRUCT": "struct", "END SUB": "sub",
    "END FUNCTION": "function", "END PROPERTY": "property",
}
_PROP_RE = re.compile(
    r"^\s*PROPERTY\s+(?:GET|SET)\s+([A-Za-z_][A-Za-z0-9_]*)", re.IGNORECASE)


def scan_scopes(source: str) -> list[Scope]:
    """Liefert alle CLASS/STRUCT/SUB/FUNCTION/PROPERTY-Bloecke mit ihrem
    Zeilenbereich (`line`..`end`). Verschachtelung wird ueber einen Stack
    aufgeloest; nicht geschlossene Bloecke enden am Dateiende.

    Heuristik (string-/comment-aware via `_strip_comment_and_strings`),
    nicht Lex-genau -- fuer Breadcrumbs/Scope-Pfad ausreichend.
    """
    lines = source.split("\n")
    n = len(lines)
    stack: list[Scope] = []
    out: list[Scope] = []
    for ln, raw in enumerate(lines, start=1):
        s = _strip_comment_and_strings(raw).strip()
        if not s:
            continue
        upper = s.upper()
        # Block-Ende?
        ender = None
        if upper.startswith("END "):
            two = " ".join(upper.split()[:2])
            ender = _END_KIND.get(two)
        if ender is not None:
            # passenden Block vom Stack nehmen (top wenn Art passt)
            for i in range(len(stack) - 1, -1, -1):
                if stack[i].kind == ender:
                    sc = stack[i]
                    sc.end = ln
                    del stack[i:]   # i und alles darueber schliessen
                    break
            continue
        # Block-Anfang?
        mprop = _PROP_RE.match(s)
        if mprop:
            sc = Scope("property", mprop.group(1), ln, n)
            stack.append(sc); out.append(sc)
            continue
        for prefix, kind in _SCOPE_OPENERS:
            if upper.startswith(prefix):
                rest = s[len(prefix):].lstrip()
                name = re.match(r"([A-Za-z_][A-Za-z0-9_]*)", rest)
                sc = Scope(kind, name.group(1) if name else "?", ln, n)
                stack.append(sc); out.append(sc)
                break
    return out


def scope_path(source: str, line: int) -> list[Scope]:
    """Kette der Bloecke (aussen -> innen), die Zeile `line` umschliessen.
    Leere Liste auf Top-Level. Fuer die Breadcrumb-Leiste."""
    path = [sc for sc in scan_scopes(source) if sc.line <= line <= sc.end]
    path.sort(key=lambda sc: sc.line)
    return path


def extract_user_doc(source: str, name: str) -> tuple[str, str] | None:
    """Liefert `(signatur, doc)` zu einem User-Symbol -- analog zu
    `editor_qt.builtin_docs.get_doc`, aber fuer im Buffer definierte
    SUBs/FUNCTIONs/CLASSes/STRUCTs/ENUMs/CONSTs.

    Signatur:
      - SUB/FUNCTION: `SUB foo(a AS INTEGER, b AS STRING)` -- die Decl-
        Zeile bis zum Ende der Param-Liste, plus `AS <typ>` bei FUNCTION.
      - CLASS/STRUCT/ENUM/CONST: die Decl-Zeile als-ist (bis zum ersten
        Comment oder Zeilenende).
      - DIM/param: die Decl-Zeile als-ist.

    Doc:
      Block aus aufeinanderfolgenden Comment-Zeilen DIREKT ueber der
      Definition. `'`-Kommentare und `REM`-Kommentare werden gleich
      behandelt; ein leerer/Code-Zwischenraum bricht den Block ab.
      Leerer String, wenn keine Comment-Zeilen vorhanden sind.
    """
    d = find_definition(source, name)
    if d is None:
        return None
    lines = source.split("\n")
    if not (1 <= d.line <= len(lines)):
        return None
    decl_line = lines[d.line - 1]
    sig = _extract_signature(decl_line, d.kind, lines, d.line - 1)
    doc = _extract_leading_doc(lines, d.line - 1)
    return (sig, doc)


def _extract_signature(decl_line: str, kind: str, lines: list[str],
                       decl_idx: int) -> str:
    """Holt die Signatur-Zeile (potenziell ueber mehrere Source-Zeilen,
    falls die Param-Liste umgebrochen wurde).

    `decl_idx` ist 0-basiert. Bei `SUB`/`FUNCTION` lesen wir solange
    weiter, bis die schliessende Klammer + ggf. `AS <typ>` gefunden wurde.
    """
    sig = decl_line.rstrip()
    # Inline-Kommentar abschneiden.
    sig = _strip_inline_comment(sig)
    if kind in ("sub", "function"):
        # Multi-Line-Param-Listen sind selten, aber moeglich. Wenn die Decl-
        # Zeile keinen `)` hat, lesen wir weiter.
        if "(" in sig and ")" not in sig:
            buf = sig
            i = decl_idx + 1
            while i < len(lines) and ")" not in buf:
                next_line = _strip_inline_comment(lines[i].rstrip())
                buf = buf + " " + next_line.lstrip()
                i += 1
            sig = buf
    return sig.strip()


def _extract_leading_doc(lines: list[str], decl_idx: int) -> str:
    """Liest Comment-Zeilen rueckwaerts ab `decl_idx - 1`, solange
    aufeinanderfolgende Zeilen Comments sind. Liefert die Comments
    ohne ihr `'` / `REM`-Prefix in Source-Reihenfolge.
    """
    out: list[str] = []
    i = decl_idx - 1
    while i >= 0:
        ln = lines[i].strip()
        if not ln:
            break  # leere Zeile bricht den Doc-Block
        text = _strip_comment_marker(ln)
        if text is None:
            break
        out.append(text)
        i -= 1
    out.reverse()
    return "\n".join(out).strip()


def _strip_comment_marker(line: str) -> str | None:
    """Wenn `line` ein Comment ist, gib den Text dahinter zurueck.
    Sonst None. Akzeptiert `'comment` und `REM comment` (case-insensitiv,
    `REM` muss am Wortanfang stehen)."""
    if line.startswith("'"):
        return line[1:].strip()
    if len(line) >= 3 and line[:3].lower() == "rem":
        rest = line[3:]
        if not rest or rest[0] in (" ", "\t"):
            return rest.strip()
    return None


def _strip_inline_comment(line: str) -> str:
    """Schneidet einen Trailing-`'`-Kommentar von der Code-Zeile ab.
    Strings (mit `""`-Escape) werden korrekt uebersprungen.
    `REM` wird hier NICHT als inline-Kommentar behandelt -- das ist nur
    am Zeilenanfang ueblich.
    """
    in_str = False
    i = 0
    while i < len(line):
        ch = line[i]
        if in_str:
            if ch == '"':
                if i + 1 < len(line) and line[i + 1] == '"':
                    i += 2
                    continue
                in_str = False
            i += 1
            continue
        if ch == '"':
            in_str = True
            i += 1
            continue
        if ch == "'":
            return line[:i].rstrip()
        i += 1
    return line
