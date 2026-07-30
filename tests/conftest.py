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


# Die fruehere `call_builtin`-Fixture (rief Python-Builtin-Impls direkt via
# `interpreter.BUILTINS`) ist entfernt -- alle Modul-Tests laufen jetzt als
# run_gb-Golden gegen gbrt (Stufe B, Phase 6/7-Teil2). Damit haengt kein Test
# mehr an interpreter.py/modules (Phase-8-Entblocker erledigt).


# --------------------------------------------------------------------------
# Qt-Altlasten nach JEDEM Test stilllegen
# --------------------------------------------------------------------------
# Die Qt-Testdateien bauen ihre Fenster als lokale Variable bzw. in einer
# function-scoped Fixture und schliessen sie nie -- das C++-Objekt lebt danach
# weiter (Qt haelt Top-Level-Widgets selbst am Leben, der Python-Refcount
# entscheidet nicht allein). In EINEM gemeinsamen pytest-Prozess summierte
# sich das auf ~16.000 lebende QObjects mit 244 faelligen QTimern, darunter
# 9 WIEDERHOLENDE 16-ms-Vorschau-Timer (Partikel-Editor, 60 FPS) und
# 8 wiederholende 30-s-Autosave-Timer aus geleakten `GameBasicEditor`n.
#
# Solange kein Test die Event-Loop pumpt, faellt das nicht auf. Der EINE Test,
# der es tut (`test_spriteeditor_qt_canvas.py`, "coalesced until event loop
# tick"), liess dann alle 244 ueberfaelligen Timer auf einmal feuern -- und
# weil die 16-ms-Timer sich schneller neu scharf machen, als die Queue
# abgearbeitet wird, kehrte `processEvents()` NIE zurueck (Haenger mit 100 %
# CPU, per py-spy exakt auf dieser Zeile stehend). Wo ein Slot dabei ein
# bereits geloeschtes C++-Objekt traf, gab es stattdessen die
# "Windows fatal exception: access violation".
#
# WARUM ENTSCHAERFEN STATT ZERSTOEREN: der naheliegende Weg waere
# `deleteLater()` auf jedes uebrige Top-Level-Widget. Genau das wurde
# ausprobiert -- und stuerzt beim tatsaechlichen Zerstoeren ab (Access
# Violation im `sendPostedEvents(DeferredDelete)`). Der Grund liegt NICHT im
# Test, sondern in echten Zerstoerungs-Reihenfolge-Fehlern der Editor-Widgets
# (dieselbe bekannte PySide6-Use-after-free-Serie; sichtbar auch als
# "RuntimeError: Internal C++ object (GBHighlighter) already deleted" aus
# `CodeEditor._on_theme_changed`). Diese Fixture repariert die NICHT und tut
# auch nicht so -- sie nimmt den Leichen nur die Zuendschnur:
#
#   * jeder aktive QTimer wird gestoppt  -> keine faelligen/wiederholenden
#     Timer mehr, die Queue laeuft leer, `processEvents()` kehrt zurueck
#   * jeder QFileSystemWatcher verliert seine Pfade -> kein Watcher-Thread,
#     der im Hintergrund weiter Aenderungs-Events nachschiebt
#   * die Fenster werden versteckt -> keine Repaints
#
# Die Objekte selbst bleiben am Leben (Speicher waechst weiter). Das ist
# unschoen, aber harmlos -- toedlich war ausschliesslich das Feuern.

def _disarm_leftover_qt_widgets() -> None:
    """Timer/Watcher aller uebrig gebliebenen Top-Level-Widgets stilllegen."""
    qtwidgets = sys.modules.get("PySide6.QtWidgets")
    if qtwidgets is None:
        return                      # Nicht-Qt-Test: nichts zu tun, nichts zu zahlen
    app = qtwidgets.QApplication.instance()
    if app is None:
        return

    import shiboken6
    from PySide6.QtCore import QFileSystemWatcher, QTimer

    for w in list(app.topLevelWidgets()):
        # Ein zerstoertes Elternteil nimmt seine Kinder mit -- die stehen
        # dann noch in der Liste, sind aber schon ungueltig.
        if not shiboken6.isValid(w):
            continue
        try:
            for t in w.findChildren(QTimer):
                if shiboken6.isValid(t) and t.isActive():
                    t.stop()
            for fsw in w.findChildren(QFileSystemWatcher):
                if not shiboken6.isValid(fsw):
                    continue
                paths = fsw.directories() + fsw.files()
                if paths:
                    fsw.removePaths(paths)
            if not w.isHidden():
                w.hide()
        except RuntimeError:
            # Waehrend des Durchlaufs weggeraeumtes C++-Objekt -- egal,
            # tot ist auch entschaerft.
            continue


@pytest.fixture(autouse=True)
def _qt_widget_cleanup():
    """Legt nach jedem Test die zurueckgelassenen Qt-Fenster still.

    Autouse + function-scoped: laeuft damit VOR den modul-eigenen Fixtures
    im Setup und folglich NACH deren Teardown -- eine `win`-Fixture darf ihr
    Fenster also noch ganz normal selbst abbauen.
    """
    yield
    _disarm_leftover_qt_widgets()


def quiesce_qt() -> int:
    """Den GANZEN Prozess ruhigstellen: jeden aktiven QTimer stoppen und alle
    schon zugestellten Events verwerfen. Gibt die Zahl gestoppter Timer zurueck.

    Warum zusaetzlich zu `_disarm_leftover_qt_widgets()`: das laeuft ueber
    `topLevelWidget.findChildren()` und erwischt damit nur Timer, die im
    Objektbaum eines Fensters haengen. `SnapshotUndo` (Undo-Debounce der
    Editoren) ist aber ein parentloses QObject -- nach einem gemeinsamen Lauf
    blieben so noch ~39 scharfe Timer uebrig, deren `changed`-Signal ueber
    `_mark_dirty()` wieder ein `mark()` ausloest. Diese Rueckkopplung reicht,
    damit die Event-Queue nie leer wird.

    TEUER (einmaliger Heap-Scan) -- nur direkt vor einer Stelle aufrufen, die
    wirklich die Event-Loop pumpt, nicht pro Test.
    """
    qtcore = sys.modules.get("PySide6.QtCore")
    if qtcore is None:
        return 0

    import gc
    import shiboken6

    stopped = 0
    for obj in gc.get_objects():
        if not isinstance(obj, qtcore.QTimer):
            continue
        try:
            if shiboken6.isValid(obj) and obj.isActive():
                obj.stop()
                stopped += 1
        except RuntimeError:
            pass                    # waehrenddessen weggeraeumt -- auch gut
    # Bereits gepostete Events (Repaints, queued Signals) der Altlasten
    # wegwerfen, damit der folgende Pump nur noch das sieht, was der Test
    # selbst erzeugt.
    qtcore.QCoreApplication.removePostedEvents(None)
    return stopped


@pytest.fixture
def quiet_qt_process():
    """Fixture-Form von `quiesce_qt()` -- fuer Tests, die selbst die
    Event-Loop pumpen muessen und deshalb einen ruhigen Prozess brauchen."""
    quiesce_qt()
    yield
