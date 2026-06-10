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


# --- start_run_auto: gbrt direkt + gbrun.py-Launcher-Fallback -----------
# Policy-Test ohne echten QProcess: wir mocken die gbrt-Suche und die
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


def test_run_auto_falls_back_when_gbrt_missing(_qapp, tmp_path, monkeypatch):
    import gamebasic.editor_qt.output_console as oc
    monkeypatch.setattr(oc, "_find_gbrt", lambda root: None)
    con = _console(_qapp, tmp_path)
    # start_run (via gbrun.py) starten wir nicht wirklich -> Stub.
    monkeypatch.setattr(con, "start_run", lambda fp, *a, **k: True)
    assert con.start_run_auto(tmp_path / "x.gb") == "py"
    # Hinweis, dass ueber den gbrun.py-Launcher gestartet wurde.
    assert "gbrun.py" in con.text.toPlainText()


def test_run_auto_uses_native_when_available(_qapp, tmp_path, monkeypatch):
    import gamebasic.editor_qt.output_console as oc
    monkeypatch.setattr(oc, "_find_gbrt", lambda root: Path("gbrt"))
    con = _console(_qapp, tmp_path)
    monkeypatch.setattr(con, "_start_native", lambda fp, gbrt: True)
    monkeypatch.setattr(con, "start_run", lambda *a, **k: pytest.fail("kein Fallback erwartet"))
    assert con.start_run_auto(tmp_path / "x.gb") == "native"


def test_run_auto_falls_back_when_native_fails(_qapp, tmp_path, monkeypatch):
    import gamebasic.editor_qt.output_console as oc
    monkeypatch.setattr(oc, "_find_gbrt", lambda root: Path("gbrt"))
    con = _console(_qapp, tmp_path)
    monkeypatch.setattr(con, "_start_native", lambda fp, gbrt: False)  # Direkt-Start-Fehler
    monkeypatch.setattr(con, "start_run", lambda fp, *a, **k: True)
    assert con.start_run_auto(tmp_path / "x.gb") == "py"
    assert "gbrun.py" in con.text.toPlainText()
