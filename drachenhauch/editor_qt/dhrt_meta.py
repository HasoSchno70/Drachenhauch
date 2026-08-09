"""Builtin-Metadaten fuer Editor + LSP -- entkoppelt von der Laufzeit.

Quelle ist der eingefrorene `builtin_index.json` (Name/kind/Signatur/Modul),
generiert aus der Builtin-Registry-Union (Stufe B, Phase 2). Damit holen sich
Completer/Symbols/Builtins-Panel/LSP ihre Builtin-Namen + Signaturen, OHNE
`interpreter.py`/`builtins_registry` zur Laufzeit zu importieren -- Voraussetzung
fuer das Entfernen des Tree-Walkers (Stufe B). Faellt der Index, greift ein
minimaler Fallback (`builtin_docs.py`-Keys), damit der Editor nie ganz blind ist.

API (bewusst nah an dem, was die Konsumenten bisher aus der Registry zogen):
    builtin_index()        -> list[dict]  {name, kind, signature, module}
    builtin_names_upper()  -> list[str]    (fuer Completion)
    builtin_names_lower()  -> set[str]     (fuer Symbol-Filter)
    signature(name)        -> str          ("" wenn unbekannt)
    by_module()            -> dict[str, list[dict]]  (fuer das Builtins-Panel)
"""
from __future__ import annotations

import json
from functools import lru_cache
from pathlib import Path

_INDEX_PATH = Path(__file__).resolve().parent / "builtin_index.json"


@lru_cache(maxsize=1)
def builtin_index() -> list[dict]:
    """Liste aller Builtin-Eintraege (gecacht). Leere Liste, wenn der Index
    fehlt/kaputt ist -- die Aufrufer ergaenzen dann via builtin_docs-Fallback."""
    try:
        data = json.loads(_INDEX_PATH.read_text(encoding="utf-8"))
        entries = data.get("builtins", [])
        return [e for e in entries if isinstance(e, dict) and e.get("name")]
    except Exception:
        return []


def _fallback_names() -> list[str]:
    """builtin_docs.py-Keys (uppercase) -- Minimal-Set, wenn der Index fehlt."""
    try:
        from .builtin_docs import BUILTIN_DOCS
        return [n.upper() for n in BUILTIN_DOCS]
    except Exception:
        return []


@lru_cache(maxsize=1)
def builtin_names_upper() -> list[str]:
    names = [e["name"].upper() for e in builtin_index()]
    return names or _fallback_names()


@lru_cache(maxsize=1)
def builtin_names_lower() -> set[str]:
    return {n.lower() for n in builtin_names_upper()}


@lru_cache(maxsize=1)
def _sig_map() -> dict[str, str]:
    return {e["name"].lower(): e.get("signature", "") for e in builtin_index()}


def signature(name: str) -> str:
    return _sig_map().get(name.lower(), "")


def by_module() -> dict[str, list[dict]]:
    """Builtins nach Herkunfts-Modul gruppiert (fuer das Builtins-Panel)."""
    groups: dict[str, list[dict]] = {}
    for e in builtin_index():
        groups.setdefault(e.get("module") or "core", []).append(e)
    return groups
