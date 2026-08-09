"""Tests fuer die Link-Erkennung in der Output-Konsole.

Wir testen die Regex-Konstanten direkt -- der `_apply_links`-Pfad selbst
braucht Qt-Cursor-Operationen, die in einer Test-Umgebung Aufwand
fordern. Die Regexes sind die kritische Stelle: ein falscher Match
bedeutet im echten Editor einen toten Link oder gar einen falschen Sprung.
"""
from gamebasic.editor_qt.output_console import (
    _LINK_LINE,
    _LINK_FILE_HEADER,
    _LINK_FILE_LINE,
)


# --- _LINK_LINE ----------------------------------------------------

def test_link_line_matches_zeile_format():
    m = _LINK_LINE.search("[Zeile 42] ParseError: foo")
    assert m is not None
    assert m.group(1) == "42"


def test_link_line_doesnt_match_other_brackets():
    assert _LINK_LINE.search("[Test] foo") is None


# --- _LINK_FILE_HEADER --------------------------------------------

def test_file_header_simple():
    m = _LINK_FILE_HEADER.search("Fehler in foo.gb:\n  ...")
    assert m is not None
    assert m.group(1) == "foo.gb"


def test_file_header_with_spaces():
    """Hauptbug: Pfade mit Spaces wurden vorher nicht gematched."""
    m = _LINK_FILE_HEADER.search("Fehler in mein sprite.gb:\n  ...")
    assert m is not None
    assert m.group(1) == "mein sprite.gb"


def test_file_header_with_subdir():
    m = _LINK_FILE_HEADER.search("Fehler in subdir/foo.gb:\n  ...")
    assert m is not None
    assert m.group(1) == "subdir/foo.gb"


def test_file_header_lazy_stops_at_colon():
    """Bei mehreren `:` im Output darf der Match nicht ueberlaufen."""
    m = _LINK_FILE_HEADER.search("Fehler in foo.gb:42 weitere:Info")
    assert m is not None
    assert m.group(1) == "foo.gb"


def test_file_header_no_match_without_gb():
    assert _LINK_FILE_HEADER.search("Fehler in foo.txt:") is None


# --- _LINK_FILE_LINE ----------------------------------------------
# Strikt `\S+\.gb:\d+` -- Tracebacks ohne Spaces im Pfad. Ein zu
# permissiver Regex hier wuerde mehr falsch matchen als richtig.

def test_file_line_simple():
    m = _LINK_FILE_LINE.search("at foo.gb:42 in stack")
    assert m is not None
    assert m.group(1) == "foo.gb"
    assert m.group(2) == "42"


def test_file_line_at_line_start():
    m = _LINK_FILE_LINE.search("script.gb:5: error here")
    assert m is not None
    assert m.group(1) == "script.gb"
    assert m.group(2) == "5"


def test_file_line_with_drive_letter():
    """Windows-Absolutpfade: `C:\\foo\\bar.gb:42`. `\\S+` matched
    inklusive `:` und `\\`, der `\\.gb`-Anker stoppt am korrekten Ort."""
    m = _LINK_FILE_LINE.search(r"C:\foo\bar.gb:42 trace")
    assert m is not None
    assert m.group(1) == r"C:\foo\bar.gb"
    assert m.group(2) == "42"


def test_file_line_finds_all_matches():
    s = "a.gb:1 -> b.gb:2"
    matches = list(_LINK_FILE_LINE.finditer(s))
    assert len(matches) == 2
    assert matches[0].group(1) == "a.gb"
    assert matches[1].group(1) == "b.gb"


# --- start_run_auto: dhrt direkt + gbrun.py-Launcher-Fallback -----------
# Policy-Test ohne echten QProcess: wir mocken die dhrt-Suche und die
# beiden Start-Pfade, und pruefen nur, WELCHEN Modus start_run_auto waehlt.

import os  # noqa: E402
from pathlib import Path  # noqa: E402

import pytest  # noqa: E402

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module")
def _qapp():
    try:
        from PySide6.QtWidgets import QApplication
    except Exception:  # pragma: no cover - PySide6 fehlt
        pytest.skip("PySide6 nicht verfuegbar")
    app = QApplication.instance() or QApplication([])
    yield app


def _console(_qapp, tmp_path):
    from gamebasic.editor_qt.output_console import OutputConsole
    return OutputConsole(tmp_path)


def _pump_delete_later(app):
    """deleteLater() wirkt erst, wenn der Event-Loop DeferredDelete-Events
    zustellt -- ein einzelnes processEvents() reicht dafuer oft nicht."""
    from PySide6.QtCore import QEvent
    for _ in range(5):
        app.sendPostedEvents(None, QEvent.Type.DeferredDelete)
        app.processEvents()


def test_run_auto_falls_back_when_dhrt_missing(_qapp, tmp_path, monkeypatch):
    import gamebasic.editor_qt.output_console as oc
    monkeypatch.setattr(oc, "_find_dhrt", lambda root: None)
    con = _console(_qapp, tmp_path)
    # start_run (via gbrun.py) starten wir nicht wirklich -> Stub.
    monkeypatch.setattr(con, "start_run", lambda fp, *a, **k: True)
    assert con.start_run_auto(tmp_path / "x.gb") == "py"
    # Hinweis, dass ueber den gbrun.py-Launcher gestartet wurde.
    assert "gbrun.py" in con.text.toPlainText()


def test_run_auto_uses_native_when_available(_qapp, tmp_path, monkeypatch):
    import gamebasic.editor_qt.output_console as oc
    monkeypatch.setattr(oc, "_find_dhrt", lambda root: Path("dhrt"))
    con = _console(_qapp, tmp_path)
    monkeypatch.setattr(con, "_start_native", lambda fp, dhrt: True)
    monkeypatch.setattr(con, "start_run", lambda *a, **k: pytest.fail("kein Fallback erwartet"))
    assert con.start_run_auto(tmp_path / "x.gb") == "native"


def test_run_auto_falls_back_when_native_fails(_qapp, tmp_path, monkeypatch):
    import gamebasic.editor_qt.output_console as oc
    monkeypatch.setattr(oc, "_find_dhrt", lambda root: Path("dhrt"))
    con = _console(_qapp, tmp_path)
    monkeypatch.setattr(con, "_start_native", lambda fp, dhrt: False)  # Direkt-Start-Fehler
    monkeypatch.setattr(con, "start_run", lambda fp, *a, **k: True)
    assert con.start_run_auto(tmp_path / "x.gb") == "py"
    assert "gbrun.py" in con.text.toPlainText()


# --- QProcess-Cleanup: kein Leak toter Prozess-Objekte ueber Run-Zyklen --
# Review-Fund: der alte self._proc haing (als Kind von `self`) nur an der
# Python-Referenz -- ohne deleteLater() lebte das beendete QProcess-Objekt
# bis zum Schliessen der Konsole weiter.

def test_on_finished_deletes_old_proc(_qapp, tmp_path):
    from PySide6.QtCore import QProcess
    con = _console(_qapp, tmp_path)
    proc = QProcess(con)
    con._proc = proc
    destroyed = []
    proc.destroyed.connect(lambda: destroyed.append(True))
    con._on_finished(0, None)
    _pump_delete_later(_qapp)
    assert destroyed == [True]


def test_on_error_failed_to_start_deletes_old_proc(_qapp, tmp_path):
    from PySide6.QtCore import QProcess
    con = _console(_qapp, tmp_path)
    proc = QProcess(con)
    con._proc = proc
    destroyed = []
    proc.destroyed.connect(lambda: destroyed.append(True))
    con._on_error(QProcess.ProcessError.FailedToStart)
    _pump_delete_later(_qapp)
    assert destroyed == [True]


# --- _on_error: FailedToStart-Race nach erfolgreichem waitForStarted -----
# Review-Fund: `errorOccurred` wurde komplett ignoriert -- bei einem
# asynchronen FailedToStart NACH dem waitForStarted()-Check blieb die
# Konsole im "laeuft"-Zustand haengen (kein finished, kein Fehlertext).

def test_on_error_failed_to_start_resets_state(_qapp, tmp_path):
    from PySide6.QtCore import QProcess
    con = _console(_qapp, tmp_path)
    con._proc = QProcess(con)      # simuliert einen (vermeintlich) laufenden Prozess
    con.input_entry.setEnabled(True)
    got_finished = []
    con.process_finished.connect(lambda code: got_finished.append(code))
    con._on_error(QProcess.ProcessError.FailedToStart)
    assert con._proc is None
    assert not con.input_entry.isEnabled()
    assert got_finished == [-1]
    assert "nicht gestartet" in con.text.toPlainText()


def test_on_error_other_kinds_are_noop(_qapp, tmp_path):
    # Andere Fehlerarten schliessen regulaer ueber `finished` ab -- hier
    # darf nichts passieren (kein doppeltes process_finished).
    from PySide6.QtCore import QProcess
    con = _console(_qapp, tmp_path)
    sentinel = object()
    con._proc = sentinel
    got_finished = []
    con.process_finished.connect(lambda code: got_finished.append(code))
    con._on_error(QProcess.ProcessError.Crashed)
    assert con._proc is sentinel
    assert got_finished == []


def test_console_caps_block_count_to_avoid_unbounded_growth(_qapp, tmp_path):
    """Review-Fund: kein Cap fuer die Konsolen-Groesse -- ein Programm mit
    schneller PRINT-Endlosschleife liess das Dokument unbegrenzt wachsen
    (mit steigenden Kosten pro append() durch den Link-Regex-Scan)."""
    con = _console(_qapp, tmp_path)
    assert con.text.maximumBlockCount() > 0
    for i in range(con.text.maximumBlockCount() + 500):
        con.append(f"line {i}\n")
    assert con.text.document().blockCount() <= con.text.maximumBlockCount()
