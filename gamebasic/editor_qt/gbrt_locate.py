"""Findet das `gbrt`-Binary -- EINE Quelle fuer alle Editor-Module
(output_console / debugger / error_check).

Reihenfolge:
1. Eingefrorene Installation (PyInstaller): neben `GameBasic.exe` (dorthin legt
   der Inno-Installer `gbrt.exe`) bzw. im Bundle (`_MEIPASS`).
2. Dev-Baum: `<project_root>/rust/gb_runtime/target/{release,debug}/gbrt[.exe]`
   bzw. relativ zu diesem Paket (Repo-Wurzel = parents[2]).
"""
from __future__ import annotations

import os
import sys
from pathlib import Path


def find_gbrt(project_root: Path | None = None) -> Path | None:
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    cands: list[Path] = []
    if getattr(sys, "frozen", False):
        cands.append(Path(sys.executable).resolve().parent / exe)
        meipass = getattr(sys, "_MEIPASS", None)
        if meipass:
            cands.append(Path(meipass) / exe)
    roots: list[Path] = []
    if project_root is not None:
        roots.append(Path(project_root))
    roots.append(Path(__file__).resolve().parents[2])   # Repo-Wurzel (Dev)
    for r in roots:
        for variant in ("release", "debug"):
            cands.append(r / "rust" / "gb_runtime" / "target" / variant / exe)
    for p in cands:
        if p.exists():
            return p
    return None
