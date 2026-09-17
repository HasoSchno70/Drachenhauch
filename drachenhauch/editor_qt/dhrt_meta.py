"""Builtin-Metadaten fuer Editor + LSP -- entkoppelt von der Laufzeit.

Quelle ist der eingefrorene `daten/builtin_index.json` (Name/kind/Signatur/Modul),
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
import sys
from functools import lru_cache
from pathlib import Path


def daten_datei(name: str) -> Path:
    """Weg zu einer Datei in `daten/` -- dem Ordner, in dem das
    Befehlsverzeichnis liegt.

    Bis 2026-09-17 lagen die drei `builtin_*.json` NEBEN dieser Datei, also im
    Python-Paket; `dhrt` bettete sie von dort ein und erkannte die Repo-Wurzel
    an genau diesem Pfad. Damit liess sich `drachenhauch/` nicht loeschen,
    obwohl die Laufzeit nichts von Python braucht. Jetzt liegen sie im Repo
    neben `docs/`, und der Weg dorthin wird wie bei `dhrt_locate.find_dhrt`
    aus Kandidaten gesucht: im eingefrorenen Bundle zuerst (PyInstaller legt
    `daten/` per Spec dazu), sonst ueber die Repo-Wurzel. Der erste Treffer
    gewinnt; gibt es keinen, kommt der Repo-Weg zurueck, damit die Meldung des
    Aufrufers den erwarteten Ort nennt.
    """
    kandidaten: list[Path] = []
    meipass = getattr(sys, "_MEIPASS", None) if getattr(sys, "frozen", False) else None
    if meipass:
        kandidaten.append(Path(meipass) / "daten" / name)
    repo = Path(__file__).resolve().parents[2] / "daten" / name
    kandidaten.append(repo)
    for p in kandidaten:
        if p.is_file():
            return p
    return repo


@lru_cache(maxsize=1)
def builtin_index() -> list[dict]:
    """Liste aller Builtin-Eintraege (gecacht). Leere Liste, wenn der Index
    fehlt/kaputt ist -- die Aufrufer ergaenzen dann via builtin_docs-Fallback."""
    try:
        data = json.loads(daten_datei("builtin_index.json").read_text(encoding="utf-8"))
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
