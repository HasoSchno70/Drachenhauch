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
    """Absichtlich `_check_via_dhrt` statt `_check_source`.

    `_check_source` sucht erst das Binary und faellt ohne gebautes `dhrt` auf
    den reinen Syntax-Pfad zurueck -- der startet gar keinen Prozess, und der
    Test schlug auf dem CI-Runner fehl ("es wurde gar kein Prozess
    gestartet"), obwohl der Fix da war. Der Aufruf hier braucht kein echtes
    Binary: der Start ist ohnehin abgefangen, gemessen werden nur die Flags.
    """
    from pathlib import Path

    from drachenhauch.editor_qt import error_check
    m = _Mitschnitt()
    m.haken(monkeypatch)
    error_check._check_via_dhrt("PRINT 1\n", tmp_path, Path("dhrt.exe"))
    m.pruefen()


def test_git_ohne_konsolenfenster(monkeypatch, tmp_path):
    from drachenhauch.editor_qt import gitinfo
    m = _Mitschnitt()
    m.haken(monkeypatch)
    gitinfo._run_git(["rev-parse", "HEAD"], tmp_path)
    m.pruefen()


def _dhrun():
    import importlib.util
    from pathlib import Path
    pfad = Path(__file__).resolve().parents[1] / "dhrun.py"
    spec = importlib.util.spec_from_file_location("_dhrun_test", pfad)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def test_direktstart_erbt_die_konsole_des_terminals(monkeypatch):
    """Im Terminal darf NICHTS unterdrueckt werden -- dort liest man PRINT."""
    m = _dhrun()
    monkeypatch.setattr(m, "_hat_konsole", lambda: True)
    assert m._ohne_konsolenfenster() == 0


def test_direktstart_ohne_konsole_oeffnet_keins(monkeypatch):
    """Aus der GUI heraus (keine Konsole zum Erben) bleibt das Fenster weg."""
    m = _dhrun()
    monkeypatch.setattr(m, "_hat_konsole", lambda: False)
    assert m._ohne_konsolenfenster() & NO_WINDOW


def test_konsolen_erkennung_sieht_auch_pseudokonsolen():
    """`GetConsoleWindow` waere hier die falsche Frage.

    Jedes moderne Terminal haengt an einer Pseudokonsole ohne Fenstergriff --
    `GetConsoleWindow()` liefert dort dieselbe 0 wie in einer GUI-Anwendung.
    Wer danach entscheidet, schneidet dem Terminal die Ausgabe ab. Dieser Test
    laeuft nur, wenn pytest selbst an einer Konsole haengt (sonst gibt es
    nichts zu zeigen).
    """
    import ctypes
    m = _dhrun()
    if not m._hat_konsole():
        pytest.skip("pytest laeuft ohne Konsole")
    assert m._ohne_konsolenfenster() == 0, "Terminal wuerde seine Ausgabe verlieren"


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
