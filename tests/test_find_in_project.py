"""Tests fuer die projektweite Such-/Ersetzen-Logik (Qt-frei).

Die UI (`FindInProjectDialog`) bleibt ungetestet -- der Kern (Pattern-Bau,
Ersetzungs-String, Datei-Filter, Plan-Bildung inkl. offener-Buffer-Vorrang)
ist in reine Funktionen ausgelagert und hier abgedeckt.
"""
from pathlib import Path

from gamebasic.editor_qt.find_in_project import (
    build_pattern, replacement_string, iter_gb_files, plan_replacements,
)


# ----------------------------------------------------------- build_pattern

def test_build_pattern_case_insensitive_default():
    p = build_pattern("foo", case_sensitive=False, whole_word=False, regex=False)
    assert p.search("FOO bar")


def test_build_pattern_case_sensitive():
    p = build_pattern("foo", case_sensitive=True, whole_word=False, regex=False)
    assert not p.search("FOO")
    assert p.search("foo")


def test_build_pattern_whole_word():
    p = build_pattern("cat", case_sensitive=True, whole_word=True, regex=False)
    assert p.search("a cat here")
    assert not p.search("category")


def test_build_pattern_literal_when_not_regex():
    p = build_pattern("a.c", case_sensitive=True, whole_word=False, regex=False)
    assert p.search("a.c")
    assert not p.search("abc")          # '.' ist literal


def test_build_pattern_regex_on():
    p = build_pattern("a.c", case_sensitive=True, whole_word=False, regex=True)
    assert p.search("abc")              # '.' = beliebiges Zeichen


# ------------------------------------------------------- replacement_string

def test_replacement_string_escapes_backslashes_literal():
    assert replacement_string(r"a\1b", regex=False) == r"a\\1b"


def test_replacement_string_passthrough_in_regex():
    assert replacement_string(r"x\1y", regex=True) == r"x\1y"


# ------------------------------------------------------------ iter_gb_files

def test_iter_gb_files_filters(tmp_path):
    (tmp_path / "a.gb").write_text("x", encoding="utf-8")
    (tmp_path / "_skip.gb").write_text("x", encoding="utf-8")
    (tmp_path / "note.txt").write_text("x", encoding="utf-8")
    venv = tmp_path / ".venv"; venv.mkdir()
    (venv / "lib.gb").write_text("x", encoding="utf-8")
    names = {p.name for p in iter_gb_files(tmp_path)}
    assert names == {"a.gb"}


# --------------------------------------------------------- plan_replacements

def test_plan_replacements_from_disk(tmp_path):
    (tmp_path / "a.gb").write_text("PRINT foo\nPRINT foo\n", encoding="utf-8")
    (tmp_path / "b.gb").write_text("DIM x AS INTEGER\n", encoding="utf-8")
    pat = build_pattern("foo", case_sensitive=True, whole_word=False, regex=False)
    plan, total, skipped = plan_replacements(tmp_path, pat, replacement_string("bar", False))
    assert total == 2 and len(plan) == 1 and skipped == 0
    path, new_text, n = plan[0]
    assert path.name == "a.gb" and n == 2
    assert "bar" in new_text and "foo" not in new_text


def test_plan_replacements_regex_backref(tmp_path):
    (tmp_path / "a.gb").write_text("hp=10\nmp=5\n", encoding="utf-8")
    pat = build_pattern(r"(\w+)=(\d+)", case_sensitive=True,
                        whole_word=False, regex=True)
    plan, total, _skipped = plan_replacements(tmp_path, pat,
                                    replacement_string(r"\1 := \2", True))
    assert total == 2
    _p, new_text, _n = plan[0]
    assert "hp := 10" in new_text and "mp := 5" in new_text


def test_plan_replacements_prefers_open_buffer(tmp_path):
    f = tmp_path / "a.gb"
    f.write_text("PRINT foo\n", encoding="utf-8")        # Platte: 1x
    live = {f.resolve(): "foo foo foo\n"}                # offener Buffer: 3x

    def provider(p):
        return live.get(Path(p).resolve())

    pat = build_pattern("foo", case_sensitive=True, whole_word=False, regex=False)
    plan, total, _skipped = plan_replacements(tmp_path, pat, replacement_string("bar", False),
                                    provider)
    assert total == 3                  # Live-Text gewinnt ueber die Platte
    assert "bar bar bar" in plan[0][1]


def test_plan_replacements_no_match_empty_plan(tmp_path):
    (tmp_path / "a.gb").write_text("nothing here\n", encoding="utf-8")
    pat = build_pattern("zzz", case_sensitive=True, whole_word=False, regex=False)
    plan, total, skipped = plan_replacements(tmp_path, pat, "x")
    assert plan == [] and total == 0 and skipped == 0


def test_plan_replacements_counts_unreadable_files(tmp_path, monkeypatch):
    """Review-Fund: eine nicht lesbare Datei wurde bisher komplett
    stillschweigend ausgelassen -- kein Hinweis, dass das Ergebnis
    unvollstaendig sein koennte."""
    (tmp_path / "a.gb").write_text("PRINT foo\n", encoding="utf-8")
    (tmp_path / "b.gb").write_text("PRINT foo\n", encoding="utf-8")
    pat = build_pattern("foo", case_sensitive=True, whole_word=False, regex=False)

    orig_read_text = Path.read_text

    def flaky_read_text(self, *a, **kw):
        if self.name == "b.gb":
            raise OSError("simulierter Lesefehler")
        return orig_read_text(self, *a, **kw)

    monkeypatch.setattr(Path, "read_text", flaky_read_text)
    plan, total, skipped = plan_replacements(tmp_path, pat, replacement_string("bar", False))
    assert skipped == 1
    assert total == 1 and len(plan) == 1   # a.gb weiterhin gefunden


def test_plan_replacements_skips_non_utf8_file_instead_of_corrupting_it(tmp_path):
    """Review-Fund: eine Datei mit nicht-UTF-8-Bytes (z.B. cp1252-Reste)
    wurde bisher lossy gelesen (errors="replace") -- bei einem Treffer
    IRGENDWO im Text wurde die KOMPLETTE (mit U+FFFD durchsetzte) Fassung
    zurueckgeschrieben, auch Stellen, die mit dem Treffer nichts zu tun
    hatten. Die Datei muss jetzt wie ein Lesefehler behandelt (uebersprungen)
    werden, statt lossy in den Plan aufgenommen zu werden."""
    good = tmp_path / "a.gb"
    good.write_text("PRINT foo\n", encoding="utf-8")
    bad = tmp_path / "b.gb"
    # "foo" als Treffer-Text PLUS ein paar echte cp1252-Bytes (0x84 = "„" in
    # cp1252, aber ungueltiges UTF-8-Fortsetzungsbyte) anderswo in der Datei.
    bad.write_bytes("PRINT foo\n' Kommentar: ".encode("utf-8") + b"\x84\n")
    pat = build_pattern("foo", case_sensitive=True, whole_word=False, regex=False)

    plan, total, skipped = plan_replacements(tmp_path, pat, replacement_string("bar", False))

    assert skipped == 1
    assert total == 1 and len(plan) == 1
    assert plan[0][0] == good
    # Die kaputte Datei bleibt auf der Platte unveraendert (nie geschrieben).
    assert bad.read_bytes() == "PRINT foo\n' Kommentar: ".encode("utf-8") + b"\x84\n"


# ------------------------------------------------------------- Dialog (UI)

def test_dialog_replace_all_writes_disk(tmp_path, monkeypatch):
    """End-to-End: _replace_all baut Plan, fragt (gemockt: Ja) und wendet
    die apply_replacement-Callback an."""
    import os
    os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
    try:
        from PySide6.QtWidgets import QApplication, QMessageBox
        from gamebasic.editor_qt.find_in_project import FindInProjectDialog
    except Exception:
        import pytest
        pytest.skip("PySide6 nicht verfuegbar")
    QApplication.instance() or QApplication([])
    (tmp_path / "a.gb").write_text("PRINT foo\nPRINT foo\n", encoding="utf-8")
    (tmp_path / "b.gb").write_text("DIM x AS INTEGER\n", encoding="utf-8")

    dlg = FindInProjectDialog(None, tmp_path)
    dlg.entry.setText("foo")
    dlg.replace_entry.setText("bar")
    applied: list = []

    def _apply(p, text):
        applied.append(Path(p).name)
        Path(p).write_text(text, encoding="utf-8")

    dlg.apply_replacement = _apply
    monkeypatch.setattr(QMessageBox, "question",
                        staticmethod(lambda *a, **k: QMessageBox.StandardButton.Yes))
    monkeypatch.setattr(dlg, "_start", lambda: None)   # kein Such-Thread im Test
    dlg._replace_all()

    assert applied == ["a.gb"]
    assert (tmp_path / "a.gb").read_text(encoding="utf-8") == "PRINT bar\nPRINT bar\n"
    assert (tmp_path / "b.gb").read_text(encoding="utf-8") == "DIM x AS INTEGER\n"
