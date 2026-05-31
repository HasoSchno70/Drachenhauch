"""IMPORT-Preprocessor fuer GameBasic.

`process(source, base_path, file_label)` liefert ein Tupel
`(merged_source, origins)` zurueck. `origins[i]` ist `(file, original_line)`
fuer die 1-basierte Zeile `i` der gemergten Quelle - so koennen
Fehler-Reporter beim Anzeigen vom merged-line auf die Original-Datei
und -Zeile zurueckmappen.

Dies ist KEIN Modulsystem mit Namespaces - eingebundener Code teilt
sich den globalen Namensraum. Sinnvoll fuer Hilfsbibliotheken.
"""
from __future__ import annotations

import re
from pathlib import Path

from .errors import LexerError
from . import modules as _modules


# IMPORT-Syntax: `IMPORT "<name>"`  oder  `IMPORT "<name>" AS <alias>`.
# `<alias>` muss alphanumerisch + Underscore sein, mit Buchstabe/Underscore
# am Anfang -- wie ein Identifier. Der Alias ist optional und nur fuer
# Built-in-Module sinnvoll (Quellcode-IMPORT ignoriert ihn).
_IMPORT_RE = re.compile(
    r'^\s*IMPORT\s+"([^"]+)"\s*'
    r'(?:AS\s+([A-Za-z_][A-Za-z0-9_]*)\s*)?'
    r'(?:\'.*)?$',
    re.IGNORECASE,
)


def _looks_like_module_name(rel: str) -> bool:
    """Heuristik: Pfad ohne Slash/Backslash und ohne .gb-Endung -> Modul."""
    if "/" in rel or "\\" in rel:
        return False
    if rel.lower().endswith(".gb"):
        return False
    return _modules.is_valid_module_name(rel)


def process(source: str, base_path: Path | None = None,
            seen: set | None = None,
            file_label: str = "<main>") -> tuple[str, list]:
    """Expandiert IMPORTs rekursiv. Gibt (merged_source, origins) zurueck.

    origins[0] = None (1-basiert)
    origins[i] = (file_label, original_line) fuer 1<=i<=Zeilenzahl(merged_source)
    """
    if seen is None:
        seen = set()
    if base_path is None:
        base_path = Path.cwd()
    out_lines: list[str] = []
    origins: list = [None]   # 1-basiert
    for line_idx, raw in enumerate(source.split("\n"), start=1):
        m = _IMPORT_RE.match(raw)
        if not m:
            out_lines.append(raw)
            origins.append((file_label, line_idx))
            continue
        rel = m.group(1)
        alias = m.group(2)  # None wenn `AS <alias>` nicht angegeben
        target = (base_path / rel).resolve()
        if target in seen:
            out_lines.append(f"' [IMPORT bereits inkludiert: {rel}]")
            origins.append((file_label, line_idx))
            continue
        if not target.exists():
            # Fallback: Built-in-Modul wie "json", "db", "ui".
            # Nur wenn der Pfad nach einem Modul-Namen aussieht (kein Slash,
            # keine .gb-Endung) - sonst wuerde "missing.gb" auch als Modul
            # versucht und die Fehlermeldung wuerde irrefuehren.
            if _looks_like_module_name(rel) and _modules.load_module(rel, alias=alias):
                tag = f" AS {alias}" if alias else ""
                out_lines.append(f"' === IMPORT MODULE {rel}{tag} ===")
                origins.append((file_label, line_idx))
                continue
            raise LexerError(
                f"IMPORT: Datei nicht gefunden: {rel} (gesucht: {target})",
                line_idx, 1,
            )
        try:
            content = target.read_text(encoding="utf-8")
        except OSError as exc:
            raise LexerError(f"IMPORT: Lesefehler bei {rel}: {exc}", line_idx, 1)
        seen.add(target)
        inner_src, inner_origins = process(content, target.parent, seen,
                                           file_label=target.name)
        out_lines.append(f"' === IMPORT {rel} ===")
        origins.append((file_label, line_idx))
        # Inner-Quelle einfuegen
        for inner_line in inner_src.split("\n"):
            out_lines.append(inner_line)
        # Inner-Origins anhaengen (ohne den 0-Platzhalter)
        origins.extend(inner_origins[1:])
        out_lines.append(f"' === END IMPORT {rel} ===")
        origins.append((file_label, line_idx))
    return "\n".join(out_lines), origins

