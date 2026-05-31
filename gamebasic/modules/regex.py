"""Regex-Modul fuer GameBasic -- Pattern-Matching auf Strings.

Statt manueller String-Manipulation mit INSTR/MID$/SPLIT$ liefert das
regex-Modul Python-kompatible Regex-Operationen. Patterns sind die
gleiche Syntax wie Python's `re`-Modul (PCRE-aehnlich).

Built-ins:
  REGEX_MATCH(text, pattern)              AS BOOLEAN  -- vollstaendiger Match
  REGEX_TEST(text, pattern)               AS BOOLEAN  -- irgendwo im String
  REGEX_FIND(text, pattern)               AS STRING   -- erster Treffer (oder "")
  REGEX_FIND_ALL(text, pattern)           AS ARRAY OF STRING
  REGEX_REPLACE(text, pattern, repl)      AS STRING
  REGEX_REPLACE_ONCE(text, pattern, repl) AS STRING
  REGEX_SPLIT(text, pattern)              AS ARRAY OF STRING

Beispiele:

  IMPORT "regex"
  IF REGEX_TEST(line, "^[a-z]+$") THEN ...
  PRINT REGEX_REPLACE("Hallo Welt", "[aeiou]", "*")    ' "H*ll* W*lt"
  DIM parts AS ARRAY OF STRING
  parts = REGEX_SPLIT("a,b,,c", ",+")                  ' ["a", "b", "c"]

Patterns werden gecacht (kompilierte Regex pro Pattern), so dass
wiederholte Aufrufe mit gleichem Pattern keine Re-Compile-Kosten
haben.
"""
from __future__ import annotations

import re as _re

from ..builtins_registry import builtin
from ..errors import GBRuntimeError


# Pattern-Cache: gleicher Pattern-String → wiederverwendete kompilierte Regex.
# Begrenzt auf 256 Eintraege, damit ein Programm mit dynamischen Patterns
# nicht den Speicher fuellt. LRU waere ideal, aber das ist hier overkill --
# wir leeren komplett wenn voll.
_PATTERN_CACHE: dict = {}


def _compile(pattern: str):
    """Kompiliert das Pattern (mit Cache). Wirft GBRuntimeError bei
    ungueltiger Regex statt mit Python-Tracback zu sterben."""
    cached = _PATTERN_CACHE.get(pattern)
    if cached is not None:
        return cached
    if len(_PATTERN_CACHE) >= 256:
        _PATTERN_CACHE.clear()
    try:
        compiled = _re.compile(pattern)
    except _re.error as exc:
        raise GBRuntimeError(f"Ungueltige Regex '{pattern}': {exc}")
    _PATTERN_CACHE[pattern] = compiled
    return compiled


def _make_string_array(items):
    """Hilfsfunktion: liefert ein _GBArray mit STRING-Elementen aus einer
    Python-Liste."""
    from ..interpreter import _GBArray
    arr = _GBArray("string", [len(items)], lambda: "")
    for i, s in enumerate(items):
        arr.values[i] = s
    return arr


# --- Boolean-Tests ---------------------------------------------------

@builtin("REGEX_MATCH", arity=2, types=("str", "str"))
def _b_match(text, pattern):
    """Vollstaendiger Match -- der gesamte `text` muss von `pattern`
    abgedeckt werden (entspricht `re.fullmatch`)."""
    return _compile(pattern).fullmatch(text) is not None


@builtin("REGEX_TEST", arity=2, types=("str", "str"))
def _b_test(text, pattern):
    """Suche im String -- TRUE wenn das Pattern irgendwo im Text
    vorkommt (entspricht `re.search`)."""
    return _compile(pattern).search(text) is not None


# --- Treffer-Extraktion ---------------------------------------------

@builtin("REGEX_FIND", arity=2, types=("str", "str"))
def _b_find(text, pattern):
    """Liefert den ersten Treffer als STRING. Leerer String wenn nicht
    gefunden -- so kann man mit `IF REGEX_FIND(...) <> "" THEN` testen."""
    m = _compile(pattern).search(text)
    return m.group(0) if m is not None else ""


@builtin("REGEX_FIND_ALL", arity=2, types=("str", "str"))
def _b_find_all(text, pattern):
    """Alle nicht-ueberlappenden Treffer als ARRAY OF STRING."""
    matches = _compile(pattern).findall(text)
    # findall liefert bei Capture-Gruppen Tuples -- wir flatten zur ersten
    # Gruppe (oder dem ganzen Match wenn keine Gruppen).
    flat = []
    for m in matches:
        if isinstance(m, tuple):
            flat.append(m[0] if m else "")
        else:
            flat.append(m)
    return _make_string_array(flat)


# --- Ersetzung -------------------------------------------------------

@builtin("REGEX_REPLACE", arity=3, types=("str", "str", "str"))
def _b_replace(text, pattern, repl):
    """Ersetzt ALLE Treffer durch `repl`. Backslash-Sequenzen wie `\\1`
    referenzieren Capture-Gruppen (Python-Regex-Konvention)."""
    return _compile(pattern).sub(repl, text)


@builtin("REGEX_REPLACE_ONCE", arity=3, types=("str", "str", "str"))
def _b_replace_once(text, pattern, repl):
    """Ersetzt nur den ERSTEN Treffer."""
    return _compile(pattern).sub(repl, text, count=1)


# --- Split ----------------------------------------------------------

@builtin("REGEX_SPLIT", arity=2, types=("str", "str"))
def _b_split(text, pattern):
    """Splittet `text` an jedem Treffer von `pattern`. Im Gegensatz zu
    SPLIT$ kann das Trennmuster eine ganze Regex sein (z.B. mehrere
    Whitespaces auf einmal: `"\\s+"`)."""
    parts = _compile(pattern).split(text)
    return _make_string_array(parts)
