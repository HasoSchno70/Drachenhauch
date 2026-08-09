"""Tests fuer Autosave-Persistenz und Recovery.

Wir testen die `RecoveryEntry`-Roundtrip-Logik (write/read manifest) ohne
das Editor-UI hochzufahren. Der reale Tab-Close-Pfad braucht Qt + Editor
und wird durch den Smoke-Test in `test_spriteeditor_document.py` (analog)
abgedeckt; hier gehts nur um die Persistenz-Schicht.
"""
import json
from pathlib import Path
from unittest.mock import patch

import pytest

from drachenhauch.editor_qt.autosave import (
    RecoveryEntry,
    read_manifest,
    write_manifest,
    slug_for,
    autosave_dir,
)


@pytest.fixture
def tmp_autosave(tmp_path, monkeypatch):
    """Lenkt autosave_dir() / manifest_path() in tmp_path um."""
    folder = tmp_path / "autosave"
    folder.mkdir()
    monkeypatch.setattr(
        "drachenhauch.editor_qt.autosave.autosave_dir",
        lambda: folder,
    )
    monkeypatch.setattr(
        "drachenhauch.editor_qt.autosave.manifest_path",
        lambda: folder / "manifest.json",
    )
    return folder


def test_slug_for_named_file_is_stable():
    p = Path("/some/path/foo.gb")
    s1 = slug_for(p, 0)
    s2 = slug_for(p, 5)  # idx soll fuer benannte Files egal sein
    assert s1 == s2
    assert s1.startswith("saved_")
    assert s1.endswith(".gb")


def test_slug_for_unnamed_uses_idx():
    s0 = slug_for(None, 0)
    s1 = slug_for(None, 1)
    assert s0 != s1
    assert s0 == "unnamed_0.gb"
    assert s1 == "unnamed_1.gb"


def test_manifest_roundtrip_named(tmp_autosave):
    entries = [
        RecoveryEntry(
            autosave_file="saved_abc123.gb",
            original_path="/path/to/foo.gb",
            label="foo.gb",
        ),
    ]
    write_manifest(entries)
    loaded = read_manifest()
    assert len(loaded) == 1
    assert loaded[0].autosave_file == "saved_abc123.gb"
    assert loaded[0].original_path == "/path/to/foo.gb"
    assert loaded[0].label == "foo.gb"


def test_manifest_roundtrip_unnamed(tmp_autosave):
    entries = [
        RecoveryEntry(
            autosave_file="unnamed_0.gb",
            original_path=None,
            label="(neu)",
        ),
    ]
    write_manifest(entries)
    loaded = read_manifest()
    assert len(loaded) == 1
    assert loaded[0].original_path is None
    assert loaded[0].label == "(neu)"


def test_manifest_overwrite_clears_old(tmp_autosave):
    """Wichtig fuer den Tab-Close-Fall: nach Close-mit-Verwerfen wird
    write_manifest mit der reduzierten Liste aufgerufen, und die alte
    Manifest-Datei muss komplett ueberschrieben werden -- nicht nur
    angehaengt."""
    write_manifest([
        RecoveryEntry("saved_a.gb", "/a/foo.gb", "foo.gb"),
        RecoveryEntry("saved_b.gb", "/b/bar.gb", "bar.gb"),
    ])
    write_manifest([
        RecoveryEntry("saved_a.gb", "/a/foo.gb", "foo.gb"),
    ])
    loaded = read_manifest()
    assert len(loaded) == 1
    assert loaded[0].original_path == "/a/foo.gb"


def test_empty_manifest_roundtrip(tmp_autosave):
    """Letzter Tab geschlossen -> leeres Manifest schreiben, nicht
    Datei loeschen. read_manifest muss leere Liste liefern."""
    write_manifest([])
    loaded = read_manifest()
    assert loaded == []


def test_read_manifest_skips_invalid_entries(tmp_autosave):
    """Robustheit: defekte Eintraege im Manifest werden uebersprungen,
    nicht die ganze Liste verworfen."""
    raw = [
        {"autosave_file": "ok.gb", "original_path": None, "label": "ok"},
        {"autosave_file": 123, "label": "bad type"},  # autosave_file int
        "complete junk",
        {"label": "missing autosave_file"},
        {"autosave_file": "ok2.gb", "original_path": "/p", "label": "ok2"},
    ]
    (tmp_autosave / "manifest.json").write_text(
        json.dumps(raw), encoding="utf-8"
    )
    loaded = read_manifest()
    assert len(loaded) == 2
    assert loaded[0].label == "ok"
    assert loaded[1].label == "ok2"


def test_read_corrupted_json_returns_empty(tmp_autosave):
    (tmp_autosave / "manifest.json").write_text("{not json", encoding="utf-8")
    assert read_manifest() == []


def test_read_missing_manifest_returns_empty(tmp_autosave):
    # Keine manifest.json existiert
    assert read_manifest() == []


def test_write_manifest_reports_success(tmp_autosave):
    assert write_manifest([]) is True


def test_write_manifest_reports_failure_on_oserror(tmp_autosave, monkeypatch):
    # Review-Fund: write_manifest scheiterte frueher komplett stumm (Ordner
    # nicht beschreibbar/voll) -- der Aufrufer (main_window._do_autosave)
    # braucht den Rueckgabewert, um den User einmalig zu warnen.
    from pathlib import Path as _Path

    def _boom(self, *a, **kw):
        raise OSError("disk full")

    monkeypatch.setattr(_Path, "write_text", _boom)
    assert write_manifest([]) is False
