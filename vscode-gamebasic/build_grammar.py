"""Generiert syntaxes/gamebasic.tmLanguage.json aus den echten Projektlisten.

So bleibt das Syntax-Highlighting der VSCode-Extension deckungsgleich mit
Lexer-Keywords + registrierten Built-ins/Konstanten. Neu ausfuehren nach
Sprach-Aenderungen:

    .venv\\Scripts\\python.exe vscode-gamebasic\\build_grammar.py
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

# Projekt importierbar machen (Skript liegt in vscode-gamebasic/).
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from gamebasic.tokens import KEYWORDS                       # noqa: E402
from gamebasic.editor_qt.completer import (                 # noqa: E402
    collect_builtins, collect_constants)

TYPES = ["INTEGER", "FLOAT", "STRING", "BOOLEAN", "ARRAY", "MAP", "TUPLE",
         "FILE", "IMAGE", "SOUND", "FUNCREF", "COROUTINE", "OF"]
BOOLS = ["TRUE", "FALSE"]
# Steuerfluss vs. Deklaration grob trennen (rein kosmetisch fuers Theme).
DECL = {"DIM", "CONST", "SUB", "FUNCTION", "CLASS", "STRUCT", "ENUM",
        "PROPERTY", "STATIC", "OPERATOR", "EXTENDS", "BYREF", "AS", "IMPORT"}


def _alt(names) -> str:
    return "|".join(re.escape(n) for n in sorted(set(names), key=len, reverse=True))


def build() -> dict:
    kw_all = sorted(set(k.upper() for k in KEYWORDS))
    skip = set(TYPES) | set(BOOLS)
    control = [k for k in kw_all if k not in skip and k not in DECL]
    decl = [k for k in kw_all if k in DECL]

    builtins = collect_builtins()
    plain = [b for b in builtins if not b.endswith("$")]
    dollar = [b[:-1] for b in builtins if b.endswith("$")]
    consts = [c for c in collect_constants() if c not in BOOLS]

    patterns = [
        {"name": "comment.line.apostrophe.gamebasic",
         "match": "'.*$"},
        {"name": "comment.line.rem.gamebasic",
         "match": "(?i)\\bREM\\b.*$"},
        {"name": "string.quoted.double.gamebasic",
         "begin": "\"", "end": "\"",
         "patterns": [{"name": "constant.character.escape.gamebasic",
                       "match": "\"\""}]},
        {"name": "constant.numeric.gamebasic",
         "match": "\\b[0-9]+(\\.[0-9]+)?\\b"},
        {"name": "constant.language.boolean.gamebasic",
         "match": "(?i)\\b(" + _alt(BOOLS) + ")\\b"},
        {"name": "storage.type.gamebasic",
         "match": "(?i)\\b(" + _alt(TYPES) + ")\\b"},
        {"name": "keyword.declaration.gamebasic",
         "match": "(?i)\\b(" + _alt(decl) + ")\\b"},
        {"name": "keyword.control.gamebasic",
         "match": "(?i)\\b(" + _alt(control) + ")\\b"},
        {"name": "support.function.dollar.gamebasic",
         "match": "(?i)\\b(" + _alt(dollar) + ")\\$"},
        {"name": "support.function.gamebasic",
         "match": "(?i)\\b(" + _alt(plain) + ")\\b"},
        {"name": "constant.language.gamebasic",
         "match": "(?i)\\b(" + _alt(consts) + ")\\b"},
        {"name": "keyword.operator.gamebasic",
         "match": "<=|>=|<>|[-+*/\\\\=<>^]"},
    ]
    return {
        "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
        "name": "GameBasic",
        "scopeName": "source.gamebasic",
        "patterns": [{"include": "#root"}],
        "repository": {"root": {"patterns": patterns}},
    }


def main() -> None:
    out = Path(__file__).resolve().parent / "syntaxes" / "gamebasic.tmLanguage.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(build(), indent=2) + "\n", encoding="utf-8")
    print(f"geschrieben: {out}")


if __name__ == "__main__":
    main()
