"""Gemeinsame Pytest-Helpers fuer Drachenhauch-Tests."""
import io
import os
import sys
import contextlib
from pathlib import Path

import pytest


# Sicherstellen, dass das Projekt-Root im sys.path ist (fuer 'drachenhauch' Import).
_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))


# --------------------------------------------------------------------------
# Marker `qt`: alles, was PySide6 anfasst
# --------------------------------------------------------------------------
# Damit laesst sich der Qt-freie Kern allein fahren: `pytest tests/ -m "not qt"`
# ist in ~2 Minuten durch (2243 Tests) statt in ~7 -- praktisch, solange man an
# Sprache/Runtime/Werkzeugen arbeitet und die Editoren nicht anfasst.
#
# Ausserdem der Notausgang, falls der Qt-Teardown wieder einen ganzen Lauf
# zerlegt: genau dafuer lief CI kurzzeitig auf Python 3.11 mit `-m "not qt"`
# (dort starb der gemeinsame Qt-Lauf reproduzierbar mit "Windows fatal
# exception: code 0xc0000374"). CI faehrt inzwischen nur noch 3.12, wo das
# Problem nicht auftritt.
#
# Der Marker geht ueber den QUELLTEXT der Testdatei, nicht ueber den Dateinamen:
# 13 der 42 Qt-Dateien heissen gar nicht `*qt*` (test_fader.py,
# test_sfxeditor.py, test_tracker_editor_*.py, ...) -- eine Namensregel wuerde
# sie durchrutschen lassen und den Qt-freien Lauf wieder verunreinigen.
_QT_SOURCE_CACHE: dict[str, bool] = {}


def _module_uses_qt(path: str) -> bool:
    hit = _QT_SOURCE_CACHE.get(path)
    if hit is None:
        try:
            hit = "PySide6" in Path(path).read_text(encoding="utf-8", errors="replace")
        except OSError:
            hit = False
        _QT_SOURCE_CACHE[path] = hit
    return hit


def pytest_configure(config):
    config.addinivalue_line("markers", "qt: braucht PySide6 (automatisch gesetzt)")


def pytest_collection_modifyitems(items):
    for item in items:
        if _module_uses_qt(str(item.path)):
            item.add_marker(pytest.mark.qt)


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for variant in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / variant / exe
        if p.exists():
            return p
    return None


_DHRT = _find_dhrt()


def _dhrt_err_message(stderr: str) -> str:
    """Bare Fehlermeldung aus dhrt-stderr extrahieren (ohne 'Laufzeitfehler in
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
    """Fuehrt einen GB-Quelltext ueber die native Runtime (`dhrt run`) aus und
    gibt stdout zurueck (LF). Bei einem Fehler (Exit != 0) wird ein
    DHRuntimeError mit der dhrt-Meldung geworfen -- so funktioniert
    `pytest.raises(DHRuntimeError, match=...)` weiter (Wortlaut = dhrt).

    Beispiel:
        def test_print(run_gb):
            assert run_gb('PRINT "hi"') == "hi\\n"
    """
    import subprocess
    import tempfile
    from drachenhauch.errors import DHRuntimeError, ParseError, LexerError

    def _run(source: str, base: Path | None = None) -> str:
        if _DHRT is None:
            pytest.skip("native Runtime 'dhrt' nicht gebaut")
        # Temp-Datei im System-Temp-Verzeichnis (NICHT in examples/_ROOT, sonst
        # fangen die rust-Parity-Sweeps `examples.glob("*.dh")` die Datei).
        # dhrt-Module (IMPORT "vec2") sind verzeichnis-unabhaengig; relative
        # .dh-Datei-Imports sind in run_gb-Tests nicht in Gebrauch.
        # `base`: wenn gesetzt, die .dh DORT ablegen -- dhrt chdirt ins Datei-
        # Verzeichnis, also finden relative Pfade (TILED_LOAD, LOADIMAGE, ...)
        # Fixture-Dateien, die der Test in `base` abgelegt hat.
        if base is not None:
            fd, tmp = tempfile.mkstemp(suffix=".dh", prefix="_gbtest_", dir=str(base))
        else:
            fd, tmp = tempfile.mkstemp(suffix=".dh", prefix="_gbtest_")
        os.close(fd)
        try:
            Path(tmp).write_text(source, encoding="utf-8")
            # dhrt gibt UTF-8 aus -> explizit so dekodieren (sonst mis-decodet
            # Windows mit dem Locale-Codec cp1252 bei Nicht-ASCII-Ausgabe).
            r = subprocess.run([str(_DHRT), "run", tmp], capture_output=True,
                               text=True, encoding="utf-8", timeout=60)
        finally:
            try:
                os.unlink(tmp)
            except OSError:
                pass
        if r.returncode != 0:
            stderr = r.stderr or ""
            msg = _dhrt_err_message(stderr)
            # Phasen-passenden Fehlertyp werfen, damit pytest.raises(ParseError/
            # LexerError/...) wie bisher trifft. dhrt unterscheidet keine
            # TYP-Fehler von anderen Laufzeitfehlern -> die kommen als
            # DHRuntimeError (Tests dafuer pruefen die Basis DrachenhauchError).
            if "Parse-Fehler" in stderr:
                raise ParseError(msg)
            if "Lexer-Fehler" in stderr:
                raise LexerError(msg)
            raise DHRuntimeError(msg)
        return (r.stdout or "").replace("\r\n", "\n")

    return _run


# Hinweis: `run_vm`, `run_native` und `run_all` sind seit dem Entfernen der
# Python-/Cython-Bytecode-VMs **Aliase auf den Tree-Walker**. Es gibt nur noch
# zwei Ausfuehrungspfade: Tree-Walker (Python, Referenz) und die native Runtime
# `dhrt` (Rust, Produktion). Die Compiler-/Bytecode-Abdeckung gegen `dhrt`
# liefert der dedizierte Paritaets-Sweep in `test_dhrt_parity.py`. Die Aliase
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
    Identitaet gegen die native Runtime prueft `test_dhrt_parity.py`.

        def test_x(run_all):
            assert run_all('PRINT 1 + 2') == "3\\n"
    """
    return run_gb


# Die fruehere `call_builtin`-Fixture (rief Python-Builtin-Impls direkt via
# `interpreter.BUILTINS`) ist entfernt -- alle Modul-Tests laufen jetzt als
# run_gb-Golden gegen dhrt (Stufe B, Phase 6/7-Teil2). Damit haengt kein Test
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
# 8 wiederholende 30-s-Autosave-Timer aus geleakten `DrachenhauchEditor`n.
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
# ausprobiert -- und stuerzt beim tatsaechlichen Zerstoeren ab (Abbruch in
# `CodeEditor.event()`, waehrend `sendPostedEvents(DeferredDelete)` das
# Objekt unter dem laufenden Python-Frame zerlegt). Der Grund liegt NICHT im
# Test, sondern in echten Zerstoerungs-Reihenfolge-Fehlern der Editor-Widgets
# (dieselbe bekannte PySide6-Use-after-free-Serie).
#
# NACHGEMESSEN, damit das nicht wieder jemand "aufraeumt": einzeln laesst sich
# JEDES der sieben Editor-Fenster sauber abbauen (erzeugen, schliessen,
# deleteLater, DeferredDelete zustellen -- kein Absturz, kein Rest). Erst im
# echten Testlauf, mit den Altlasten vieler vorheriger Tests, kippt es: die
# Abbau-Variante dieser Fixture stuerzte in 1 von 5 Laeufen ab (immer derselbe
# Stack). Wer es erneut versuchen will, misst also bitte im vollen Lauf und
# mehrfach -- ein einzelner gruener Durchgang beweist hier gar nichts.
#
# Die Fixture nimmt den Leichen daher nur die Zuendschnur:
#
#   * jeder aktive QTimer wird gestoppt  -> keine faelligen/wiederholenden
#     Timer mehr, die Queue laeuft leer, `processEvents()` kehrt zurueck
#   * jeder QFileSystemWatcher verliert seine Pfade -> kein Watcher-Thread,
#     der im Hintergrund weiter Aenderungs-Events nachschiebt
#   * die Fenster werden versteckt -> keine Repaints
#   * die schon GEPOSTETEN Repaint- und Queued-Signal-Ereignisse werden
#     verworfen (siehe unten) -> ein `processEvents()` im naechsten Test
#     stellt nichts mehr an die Halbtoten zu
#
# Zum letzten Punkt, damit die Begruendung ehrlich bleibt: er war als Fix fuer
# den CI-Absturz auf Python 3.11 gedacht ("Windows fatal exception: code
# 0xc0000374", HEAP CORRUPTION, gemeldet beim `topLevelWidgets()`-Aufruf hier)
# -- die Vermutung war, dass ein `app.processEvents()` in einem spaeteren Test
# gepostete Ereignisse an halb abgeraeumte Objekte zustellt. **Das war falsch:**
# mit dieser Aenderung stuerzte CI an exakt derselben Stelle weiter ab. Der
# Verdacht liegt seitdem eher auf dieser Fixture selbst (sie laeuft nach JEDEM
# Test ueber alle uebrigen Fenster). CI faehrt auf 3.11 deshalb nur noch den
# Qt-freien Kern (`-m "not qt"`), siehe .github/workflows/ci.yml.
#
# Behalten wird der Schritt trotzdem, aber aus dem gemessenen Grund: der volle
# lokale Lauf wurde damit 22 % schneller (449 s statt 573 s) -- der Repaint-
# Nachschub der Altlasten war auch Rechenzeit.
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


def _drop_pending_events() -> None:
    """Gepostete Repaint- und Queued-Signal-Ereignisse der Altlasten wegwerfen.

    Nur diese beiden Arten, NICHT alles: ein pauschales
    `removePostedEvents(None)` wuerde auch `DeferredDelete` mitnehmen und damit
    Objekte, die ein Test ordentlich per `deleteLater()` abgemeldet hat, fuer
    immer am Leben lassen (davor warnt die Qt-Doku ausdruecklich).

    * `UpdateRequest` -- der Repaint-Nachschub versteckter Fenster
    * `MetaCall`      -- Slot-Aufrufe aus queued Signal-Verbindungen; genau die
                         treffen sonst Objekte, deren C++-Seite schon weg ist
    """
    qtcore = sys.modules.get("PySide6.QtCore")
    if qtcore is None:
        return
    ev = qtcore.QEvent.Type
    for kind in (ev.UpdateRequest, ev.MetaCall):
        qtcore.QCoreApplication.removePostedEvents(None, kind)


@pytest.fixture(autouse=True)
def _qt_widget_cleanup():
    """Legt nach jedem Test die zurueckgelassenen Qt-Fenster still.

    Autouse + function-scoped: laeuft damit VOR den modul-eigenen Fixtures
    im Setup und folglich NACH deren Teardown -- eine `win`-Fixture darf ihr
    Fenster also noch ganz normal selbst abbauen.
    """
    yield
    _disarm_leftover_qt_widgets()
    _drop_pending_events()


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
