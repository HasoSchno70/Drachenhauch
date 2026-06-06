"""Gemeinsame Pytest-Helpers fuer GameBasic-Tests."""
import io
import os
import sys
import contextlib
from pathlib import Path

import pytest


# Sicherstellen, dass das Projekt-Root im sys.path ist (fuer 'gamebasic' Import).
_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))


def _find_gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = _ROOT / "rust" / "gb_runtime" / "target" / variant / exe
        if p.exists():
            return p
    return None


_GBRT = _find_gbrt()


def _gbrt_err_message(stderr: str) -> str:
    """Bare Fehlermeldung aus gbrt-stderr extrahieren (ohne 'Laufzeitfehler in
    label:line:'-Praefix), damit `pytest.raises(match=...)` die Meldung trifft."""
    s = (stderr or "").strip()
    import re
    m = re.search(r"(?:Laufzeitfehler in|Fehler in)\s+[^:]+:\d+:\s*(.*)", s, re.S)
    if m:
        return m.group(1).strip()
    # Compile/Parse: "label:line: <phase>-Fehler: MSG"
    m = re.search(r":\d+:\s*(?:[A-Za-z]+-Fehler[^:]*:\s*)?(.*)", s, re.S)
    return m.group(1).strip() if m else s


@pytest.fixture
def run_gb():
    """Fuehrt einen GB-Quelltext ueber die native Runtime (`gbrt run`) aus und
    gibt stdout zurueck (LF). Bei einem Fehler (Exit != 0) wird ein
    GBRuntimeError mit der gbrt-Meldung geworfen -- so funktioniert
    `pytest.raises(GBRuntimeError, match=...)` weiter (Wortlaut = gbrt).

    Beispiel:
        def test_print(run_gb):
            assert run_gb('PRINT "hi"') == "hi\\n"
    """
    import subprocess
    import tempfile
    from gamebasic.errors import GBRuntimeError, ParseError, LexerError

    def _run(source: str, base: Path | None = None) -> str:
        if _GBRT is None:
            pytest.skip("native Runtime 'gbrt' nicht gebaut")
        # Temp-Datei im System-Temp-Verzeichnis (NICHT in examples/_ROOT, sonst
        # fangen die rust-Parity-Sweeps `examples.glob("*.gb")` die Datei).
        # gbrt-Module (IMPORT "vec2") sind verzeichnis-unabhaengig; relative
        # .gb-Datei-Imports sind in run_gb-Tests nicht in Gebrauch.
        # `base`: wenn gesetzt, die .gb DORT ablegen -- gbrt chdirt ins Datei-
        # Verzeichnis, also finden relative Pfade (TILED_LOAD, LOADIMAGE, ...)
        # Fixture-Dateien, die der Test in `base` abgelegt hat.
        if base is not None:
            fd, tmp = tempfile.mkstemp(suffix=".gb", prefix="_gbtest_", dir=str(base))
        else:
            fd, tmp = tempfile.mkstemp(suffix=".gb", prefix="_gbtest_")
        os.close(fd)
        try:
            Path(tmp).write_text(source, encoding="utf-8")
            # gbrt gibt UTF-8 aus -> explizit so dekodieren (sonst mis-decodet
            # Windows mit dem Locale-Codec cp1252 bei Nicht-ASCII-Ausgabe).
            r = subprocess.run([str(_GBRT), "run", tmp], capture_output=True,
                               text=True, encoding="utf-8", timeout=60)
        finally:
            try:
                os.unlink(tmp)
            except OSError:
                pass
        if r.returncode != 0:
            stderr = r.stderr or ""
            msg = _gbrt_err_message(stderr)
            # Phasen-passenden Fehlertyp werfen, damit pytest.raises(ParseError/
            # LexerError/...) wie bisher trifft. gbrt unterscheidet keine
            # TYP-Fehler von anderen Laufzeitfehlern -> die kommen als
            # GBRuntimeError (Tests dafuer pruefen die Basis GameBasicError).
            if "Parse-Fehler" in stderr:
                raise ParseError(msg)
            if "Lexer-Fehler" in stderr:
                raise LexerError(msg)
            raise GBRuntimeError(msg)
        return (r.stdout or "").replace("\r\n", "\n")

    return _run


# Hinweis: `run_vm`, `run_native` und `run_all` sind seit dem Entfernen der
# Python-/Cython-Bytecode-VMs **Aliase auf den Tree-Walker**. Es gibt nur noch
# zwei Ausfuehrungspfade: Tree-Walker (Python, Referenz) und die native Runtime
# `gbrt` (Rust, Produktion). Die Compiler-/Bytecode-Abdeckung gegen `gbrt`
# liefert der dedizierte Paritaets-Sweep in `test_gbrt_parity.py`. Die Aliase
# bleiben, damit die ~550 bestehenden Tests unveraendert weiterlaufen.

@pytest.fixture
def run_vm(run_gb):
    """Alias auf den Tree-Walker (frueher Python-VM -- entfernt)."""
    return run_gb


@pytest.fixture
def run_native(run_gb):
    """Alias auf den Tree-Walker (frueher Cython-VM -- entfernt)."""
    return run_gb


@pytest.fixture
def run_all(run_gb):
    """Alias auf den Tree-Walker (frueher 3-Pfad-Bit-Identitaet). Die
    Identitaet gegen die native Runtime prueft `test_gbrt_parity.py`.

        def test_x(run_all):
            assert run_all('PRINT 1 + 2') == "3\\n"
    """
    return run_gb


@pytest.fixture
def call_builtin():
    """Ruft eine Built-in-Funktion direkt auf, mit args als Liste."""
    from gamebasic.interpreter import BUILTINS

    def _call(name: str, args: list):
        fn = BUILTINS.get(name.lower())
        if fn is None:
            raise KeyError(f"Built-in '{name}' nicht registriert")
        return fn(args)

    return _call
