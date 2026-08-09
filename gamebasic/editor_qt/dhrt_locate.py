"""Findet das `dhrt`-Binary -- EINE Quelle fuer alle Editor-Module
(output_console / debugger / error_check).

Reihenfolge:
1. Eingefrorene Installation (PyInstaller): neben `GameBasic.exe` (dorthin legt
   der Inno-Installer `dhrt.exe`) bzw. im Bundle (`_MEIPASS`).
2. Dev-Baum: `<project_root>/rust/drachenhauch_runtime/target/{release,debug}/dhrt[.exe]`
   bzw. relativ zu diesem Paket (Repo-Wurzel = parents[2]).
"""
from __future__ import annotations

import os
import sys
import threading
from pathlib import Path

# Geteiltes Semaphor ueber alle Aufrufer (error_check.py/profiler.py/
# debugger.py), die `dhrt` als Subprozess AUS EINEM HINTERGRUND-THREAD heraus
# starten. Grund: `subprocess.Popen.__init__` dupliziert/vererbt Handles auf
# Windows ueber einen Zwischen-Schritt (`_get_handles`/`_make_inheritable` ->
# `_execute_child`), der nicht dafuer ausgelegt ist, aus vielen Threads
# GLEICHZEITIG aufgerufen zu werden -- und `find_dhrt()` selbst (`Path.resolve()`
# -> `ntpath.realpath` -> `nt._getfinalpathname`) ist unter derselben
# Buendel-Last ebenfalls abgestuerzt. Reproduziert als sporadische Windows-
# Access-Violation, wenn viele Editor-Fenster/-Tabs (mit je eigenem Live-
# Error-Checker) ihren Debounce-Timer im selben Moment feuern und dadurch
# viele `dhrt --check`-Subprozesse gleichzeitig anstossen (per Thread-Dump
# verifiziert: zweistellige Anzahl Threads gleichzeitig in `find_dhrt`/
# `subprocess.py:_execute_child`/`_make_inheritable`). Ein reines Lock nur um
# `Popen()` selbst reichte NICHT (die Pfad-Aufloesung + Tempfile-I/O drumherum
# blieb unbegrenzt parallel, Crash blieb reproduzierbar) -- ein Semaphor
# umschliesst den GESAMTEN riskanten Abschnitt (Pfad-Aufloesung bis
# `communicate()`), egal wie viele Checks gleichzeitig angestossen werden.
# Empirisch verifiziert (10 aufeinanderfolgende gruene Volldurchlaeufe der
# Testsuite, vorher ca. 80% Absturzrate): Obergrenze 2 liess noch eine Rest-
# Absturzrate von ca. 20% -- erst vollstaendige Serialisierung (1) war
# zuverlaessig sauber. Normale Editor-Nutzung hat nie mehr als eine Handvoll
# gleichzeitiger Sitzungen und Checks sind Millisekunden-kurz -- die
# Serialisierung ist dort nicht spuerbar.
_MAX_CONCURRENT_DHRT_SPAWNS = 1
dhrt_spawn_semaphore = threading.BoundedSemaphore(_MAX_CONCURRENT_DHRT_SPAWNS)


def find_dhrt(project_root: Path | None = None) -> Path | None:
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
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
            cands.append(r / "rust" / "drachenhauch_runtime" / "target" / variant / exe)
    for p in cands:
        if p.exists():
            return p
    return None
