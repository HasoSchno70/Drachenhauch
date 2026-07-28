"""LSP-Feature-Logik fuer GameBasic -- reine Funktionen, ohne Transport.

Jede Funktion bekommt den Dokument-Text (+ ggf. LSP-Position als 0-basierte
(line, character)) und liefert LSP-foermige Dicts/Listen zurueck. So ist die
gesamte Sprach-Intelligenz headless testbar; `server.py` macht nur JSON-RPC
+ Position-/URI-Verwaltung drumherum.

Wiederverwendung der vorhandenen Editor-Bausteine:
  - Diagnostics: `editor_qt.error_check._check_source`
  - Completions: `editor_qt.completer` + `symbols.all_user_identifier_positions`
  - Hover:       `editor_qt.builtin_docs.get_doc` + `symbols.extract_user_doc`
  - Def/Refs/Symbols: `editor_qt.symbols`
"""
from __future__ import annotations

from pathlib import Path

from ..editor_qt import symbols as _sym

# LSP CompletionItemKind / SymbolKind (Teilmenge)
CK_FUNCTION = 3
CK_VARIABLE = 6
CK_KEYWORD = 14
CK_SNIPPET = 15
CK_CONSTANT = 21

SK_CLASS = 5
SK_METHOD = 6
SK_PROPERTY = 7
SK_FIELD = 8
SK_ENUM = 10
SK_FUNCTION = 12
SK_VARIABLE = 13
SK_CONSTANT = 14
SK_STRUCT = 23

_IDENT_CHARS = set("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_")


def _lines(text: str) -> list[str]:
    return text.split("\n")


def _line_text(text: str, line0: int) -> str:
    lines = _lines(text)
    return lines[line0] if 0 <= line0 < len(lines) else ""


def word_at(text: str, line0: int, char0: int) -> tuple[str, int, int]:
    """Identifier um (line0, char0). Liefert (wort, start_char, end_char),
    Wort ohne trailing `$`. Leeres Wort -> ("", char0, char0)."""
    line = _line_text(text, line0)
    n = len(line)
    if char0 > n:
        char0 = n
    # Review-Fund: steht der Cursor direkt HINTER einem `$` (String-Variable
    # wie `x$`, Cursor am Wortende), stoppte die Rueckwaerts-Suche bisher
    # sofort (`$` ist kein _IDENT_CHARS-Zeichen) und lieferte ein leeres
    # Wort -- Hover/Goto-Definition/Find-References fanden `x$` dann nicht,
    # obwohl der Cursor direkt am Ende des Tokens steht (ein voelliger
    # Alltagsfall). Scan-Start in diesem Fall einen Schritt vor den `$`
    # zurueckversetzen, damit die Identifier-Zeichen davor erfasst werden.
    has_dollar = char0 > 0 and line[char0 - 1] == "$"
    scan_from = char0 - 1 if has_dollar else char0
    a = scan_from
    while a > 0 and line[a - 1] in _IDENT_CHARS:
        a -= 1
    b = scan_from
    while b < n and line[b] in _IDENT_CHARS:
        b += 1
    # `$`-Suffix (GB-String-Variablen) als Teil des Worts akzeptieren
    end = b
    if b < n and line[b] == "$":
        end = b + 1
    word = line[a:end].rstrip("$")
    return word, a, end


def _prefix_at(text: str, line0: int, char0: int) -> tuple[str, int]:
    """Wort-Praefix LINKS vom Cursor (fuer Completion). Liefert (prefix, start)."""
    line = _line_text(text, line0)
    a = min(char0, len(line))
    start = a
    while start > 0 and line[start - 1] in _IDENT_CHARS:
        start -= 1
    return line[start:a], start


# --------------------------------------------------------------- Diagnostics

def diagnostics(text: str, base_path, checker=None) -> list[dict]:
    """Alle Diagnosen der Pipeline als LSP-Diagnostics (Errors + Warnungen).

    `checker` (optional): wie bei `editor_qt.error_check.LiveErrorChecker` --
    ein Objekt mit `_set_active_proc(proc)`, damit der aufrufende Worker den
    laufenden `gbrt --check`-Subprozess kennt und bei einer neueren Anfrage
    abbrechen kann (siehe `lsp.server._DiagWorker`)."""
    from ..editor_qt.error_check import _check_source
    base = Path(base_path) if base_path else None
    out: list[dict] = []
    for problem in _check_source(text, base, checker):
        line0 = max(0, problem.line - 1)
        end_char = len(_line_text(text, line0)) or 1
        out.append({
            "range": {"start": {"line": line0, "character": 0},
                      "end": {"line": line0, "character": end_char}},
            # LSP-Severity: 1=Error, 2=Warning.
            "severity": 2 if problem.severity == "warning" else 1,
            "source": "gamebasic",
            "message": problem.message,
        })
    return out


# --------------------------------------------------------------- Completion

def completions(text: str, line0: int, char0: int) -> list[dict]:
    """Built-ins/Keywords/Konstanten/Snippets + im Dokument definierte Symbole,
    gefiltert nach dem Praefix links vom Cursor."""
    from ..editor_qt import completer
    from ..editor_qt.snippets import SNIPPETS
    from ..tokens import KEYWORDS

    prefix, _ = _prefix_at(text, line0, char0)
    pl = prefix.lower()

    kw = set(KEYWORDS.keys())
    builtins = set(completer.collect_builtins())
    consts = set(completer.collect_constants())
    snips = set(SNIPPETS.keys())

    items: list[dict] = []
    seen: set[str] = set()

    def add(label: str, kind: int, detail: str = "") -> None:
        key = label.lower()
        if key in seen:
            return
        if pl and not label.lower().startswith(pl):
            return
        seen.add(key)
        items.append({"label": label, "kind": kind, "detail": detail})

    # User-Symbole aus dem Dokument zuerst (kontextrelevant).
    for d in _sym.scan_definitions(text):
        kind = {"sub": CK_FUNCTION, "function": CK_FUNCTION,
                "class": CK_CONSTANT, "struct": CK_CONSTANT,
                "enum": CK_CONSTANT, "const": CK_CONSTANT}.get(d.kind, CK_VARIABLE)
        add(d.name, kind, d.kind)
    for name in builtins:
        add(name.upper(), CK_FUNCTION, "Built-in")
    for name in consts:
        add(name.upper(), CK_CONSTANT, "Konstante")
    for name in kw:
        add(name.upper(), CK_KEYWORD, "Keyword")
    for name in snips:
        add(name, CK_SNIPPET, "Snippet")
    return items


# --------------------------------------------------------------- Hover

def hover(text: str, line0: int, char0: int) -> dict | None:
    from ..editor_qt.builtin_docs import get_doc
    word, _, _ = word_at(text, line0, char0)
    if not word:
        return None
    sig = doc = None
    bd = get_doc(word)
    if bd is not None:
        sig, doc = bd
    else:
        ud = _sym.extract_user_doc(text, word)
        if ud is not None:
            sig, doc = ud
        else:
            # Review-Fund: BUILTIN_DOCS deckt nur einen Bruchteil der
            # tatsaechlichen Built-ins ab -- fuer den Rest lieferte Hover
            # bisher komplett None, ohne jeden Hinweis. Wenigstens die
            # Signatur (aus dem gefrorenen gbrt-Metadaten-Index, derselbe
            # den auch der Qt-Editor als Fallback nutzt) ist besser als gar
            # kein Hover.
            from ..editor_qt.gbrt_meta import signature as _gbrt_sig
            gsig = _gbrt_sig(word)
            if gsig:
                sig, doc = gsig, ""
    if sig is None:
        return None
    md = f"```gamebasic\n{sig}\n```"
    if doc:
        md += "\n\n" + doc
    return {"contents": {"kind": "markdown", "value": md}}


# --------------------------------------------------------------- Definition

def definition(text: str, line0: int, char0: int) -> dict | None:
    """Liefert die 0-basierte Position der Definition (oder None).
    `server.py` ergaenzt die URI."""
    word, _, _ = word_at(text, line0, char0)
    if not word:
        return None
    d = _sym.find_definition(text, word)
    if d is None:
        return None
    return {"line": d.line - 1, "character": max(0, d.col - 1),
            "end_character": max(0, d.col_end - 1)}


# --------------------------------------------------------------- References

def references(text: str, line0: int, char0: int) -> list[dict]:
    word, _, _ = word_at(text, line0, char0)
    if not word:
        return []
    out = []
    for r in _sym.scan_references(text, word):
        out.append({"line": r.line - 1, "character": max(0, r.col - 1),
                    "end_character": max(0, r.col_end - 1)})
    return out


# --------------------------------------------------------------- Document Symbols

def document_symbols(text: str) -> list[dict]:
    """Hierarchische DocumentSymbol-Liste: CLASS/STRUCT mit ihren Methoden/
    Properties verschachtelt, plus Top-Level SUB/FUNCTION und ENUMs."""
    scopes = _sym.scan_scopes(text)
    kind_map = {"class": SK_CLASS, "struct": SK_STRUCT, "sub": SK_FUNCTION,
                "function": SK_FUNCTION, "property": SK_PROPERTY}

    def mk(sc) -> dict:
        rng = {"start": {"line": sc.line - 1, "character": 0},
               "end": {"line": sc.end - 1, "character": 0}}
        return {
            "name": sc.name,
            "kind": kind_map.get(sc.kind, SK_FUNCTION),
            "range": rng,
            "selectionRange": rng,
            "children": [],
        }

    # Verschachtelung ueber Range-Containment (scopes sind in Quell-Reihenfolge).
    roots: list[dict] = []
    stack: list[tuple] = []   # (scope, node)
    for sc in scopes:
        node = mk(sc)
        while stack and not (stack[-1][0].line <= sc.line <= stack[-1][0].end):
            stack.pop()
        if stack:
            stack[-1][1]["children"].append(node)
        else:
            roots.append(node)
        stack.append((sc, node))

    # ENUMs als Top-Level dazu (nicht von scan_scopes erfasst).
    for d in _sym.scan_definitions(text):
        if d.kind == "enum":
            rng = {"start": {"line": d.line - 1, "character": 0},
                   "end": {"line": d.line - 1, "character": 0}}
            roots.append({"name": d.name, "kind": SK_ENUM,
                          "range": rng, "selectionRange": rng, "children": []})
    roots.sort(key=lambda n: n["range"]["start"]["line"])
    return roots
