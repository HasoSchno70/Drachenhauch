"""Hilfsprozesse duerfen kein Konsolenfenster aufblitzen lassen.

`dhrt` und `git` sind Konsolen-Programme. Startet sie eine Anwendung ohne
eigene Konsole -- und genau das ist die installierte IDE (PyInstaller mit
`console=False`) --, legt Windows dem Kind ein EIGENES Konsolenfenster an, das
kurz aufblitzt. Im Entwicklungsbaum faellt das nie auf, weil der Editor dort
aus einem Terminal laeuft und das Kind dessen Konsole erbt -- deshalb dieser
Test: er prueft den Aufruf, nicht den Augenschein.

Gemessen wurde beides einmal von Hand aus einem `pythonw`-Elternprozess
(GetConsoleWindow() im Kind): ohne Flag entsteht ein Konsolenfenster, mit
`CREATE_NO_WINDOW` nicht. Der Run-Pfad ueber `QProcess` braucht nichts -- Qt
setzt das Flag selbst.
"""
from __future__ import annotations

import subprocess
import sys

import pytest

pytestmark = pytest.mark.skipif(sys.platform != "win32",
                                reason="CREATE_NO_WINDOW gibt es nur auf Windows")

NO_WINDOW = getattr(subprocess, "CREATE_NO_WINDOW", 0)


class _Mitschnitt:
    """Faengt Popen/run ab und merkt sich die creationflags."""

    def __init__(self):
        self.aufrufe: list[tuple[str, int]] = []

    def haken(self, monkeypatch):
        # Global am `subprocess`-Modul, nicht am Aufrufer: error_check
        # importiert subprocess ERST IN DER FUNKTION, ein Modul-Attribut gibt
        # es dort also gar nicht.
        def merken(*a, **kw):
            self.aufrufe.append((str(a[0][0]), kw.get("creationflags", 0)))
            raise OSError("Start unterbunden -- der Test will nur die Flags")
        monkeypatch.setattr(subprocess, "Popen", merken)
        monkeypatch.setattr(subprocess, "run", merken)

    def pruefen(self):
        assert self.aufrufe, "es wurde gar kein Prozess gestartet"
        for befehl, flags in self.aufrufe:
            assert flags & NO_WINDOW, f"{befehl} startet mit Konsolenfenster"


def test_live_diagnose_ohne_konsolenfenster(monkeypatch, tmp_path):
    from drachenhauch.editor_qt import error_check
    m = _Mitschnitt()
    m.haken(monkeypatch)
    error_check._check_source("PRINT 1\n", tmp_path)
    m.pruefen()


def test_git_ohne_konsolenfenster(monkeypatch, tmp_path):
    from drachenhauch.editor_qt import gitinfo
    m = _Mitschnitt()
    m.haken(monkeypatch)
    gitinfo._run_git(["rev-parse", "HEAD"], tmp_path)
    m.pruefen()


def test_alle_hilfsaufrufe_tragen_das_flag():
    """Grobnetz gegen eine neue Stelle, die es vergisst.

    Absichtlich textuell: ein Aufruf, der spaeter dazukommt, faellt hier auf,
    ohne dass jemand einen eigenen Test dafuer schreiben muss. Die
    PROGRAMM-Laeufe (`dhrt run`) sind bewusst ausgenommen -- dort ist die
    Konsole das einzige Fenster, in dem PRINT-Ausgaben landen.
    """
    import re
    from pathlib import Path

    wurzel = Path(__file__).resolve().parents[1] / "drachenhauch"
    muster = re.compile(r"subprocess\.(Popen|run)\(\s*\[str\(dhrt\),\s*\"(--check|--export|debug|profile)")
    fehlt = []
    for datei in sorted(wurzel.rglob("*.py")):
        text = datei.read_text(encoding="utf-8")
        for treffer in muster.finditer(text):
            # ab dem Treffer bis zur schliessenden Klammer des Aufrufs schauen
            ausschnitt = text[treffer.start():treffer.start() + 600]
            if "creationflags=" not in ausschnitt.split("\n\n")[0]:
                fehlt.append(f"{datei.name}: {treffer.group(2)}")
    assert not fehlt, "ohne creationflags (Konsolenfenster blitzt auf): " + ", ".join(fehlt)
