"""Tests fuer Git-Blame (Porcelain-Parser + echtes Repo-Blame)."""
import shutil
import subprocess

import pytest

from gamebasic.editor_qt.gitinfo import (
    BlameLine, blame, is_git_repo, parse_porcelain,
)

# Beispiel-Porcelain (zwei Commits + eine uncommitted Zeile).
_PORCELAIN = (
    "a1b2c3d4e5f6071829304151617181920a1b2c3d 1 1 2\n"
    "author Alice\n"
    "author-mail <alice@example.com>\n"
    "author-time 1700000000\n"
    "author-tz +0000\n"
    "committer Alice\n"
    "committer-time 1700000000\n"
    "committer-tz +0000\n"
    "summary Erste Zeile\n"
    "filename foo.gb\n"
    "\tPRINT 1\n"
    "a1b2c3d4e5f6071829304151617181920a1b2c3d 2 2\n"
    "\tPRINT 2\n"
    "0000000000000000000000000000000000000000 3 3 1\n"
    "author Not Committed Yet\n"
    "author-time 1700000100\n"
    "author-tz +0000\n"
    "summary Version of foo.gb\n"
    "filename foo.gb\n"
    "\tPRINT 3\n"
)


def test_parse_porcelain_basic():
    lines = parse_porcelain(_PORCELAIN)
    assert len(lines) == 3
    assert [l.line for l in lines] == [1, 2, 3]


def test_parse_porcelain_metadata_cached_for_repeated_commit():
    lines = parse_porcelain(_PORCELAIN)
    # Zeile 2 wiederholt denselben Commit -> Metadaten aus Cache.
    assert lines[0].author == "Alice"
    assert lines[1].author == "Alice"
    assert lines[0].sha == "a1b2c3d4"
    assert lines[1].sha == "a1b2c3d4"
    assert lines[0].summary == "Erste Zeile"


def test_parse_porcelain_date_in_tz():
    lines = parse_porcelain(_PORCELAIN)
    # 1700000000 UTC = 2023-11-14.
    assert lines[0].date == "2023-11-14"


def test_parse_porcelain_uncommitted_line():
    lines = parse_porcelain(_PORCELAIN)
    assert lines[2].uncommitted is True
    assert lines[2].sha == ""


def test_parse_porcelain_empty():
    assert parse_porcelain("") == []


def test_blame_non_repo(tmp_path):
    f = tmp_path / "x.gb"
    f.write_text("PRINT 1\n", encoding="utf-8")
    res = blame(str(f))
    assert res.ok is False
    assert res.error


def test_blame_missing_file(tmp_path):
    res = blame(str(tmp_path / "nope.gb"))
    assert res.ok is False


@pytest.mark.skipif(shutil.which("git") is None, reason="git nicht installiert")
def test_blame_real_repo(tmp_path):
    """Echtes Mini-Repo: commit eine Datei, dann blame."""
    def run(*args):
        subprocess.run(["git", *args], cwd=tmp_path, check=True,
                       capture_output=True)
    run("init")
    run("config", "user.email", "t@t.de")
    run("config", "user.name", "Tester")
    f = tmp_path / "prog.gb"
    f.write_text("PRINT 1\nPRINT 2\n", encoding="utf-8")
    run("add", "prog.gb")
    run("-c", "commit.gpgsign=false", "commit", "-m", "init prog")

    assert is_git_repo(tmp_path) is True
    res = blame(str(f))
    assert res.ok is True
    assert len(res.lines) == 2
    assert all(isinstance(l, BlameLine) for l in res.lines)


@pytest.mark.skipif(shutil.which("git") is None, reason="git nicht installiert")
def test_blame_repo_dir_name_containing_dot(tmp_path):
    """Review-Fund: blame() ruft is_git_repo(cwd) mit einem BEREITS
    aufgeloesten Verzeichnis auf -- is_git_repo wendete intern zusaetzlich
    die Datei-vs-Verzeichnis-Heuristik (".suffix vorhanden -> eine Ebene
    hoch") ein ZWEITES Mal an. Heisst das Verzeichnis selbst z.B.
    "examples.bak" (Punkt im Namen), landete die Pruefung faelschlich im
    Elternverzeichnis (das kein Git-Repo ist) statt im echten Repo."""
    repo_dir = tmp_path / "examples.bak"
    repo_dir.mkdir()

    def run(*args):
        subprocess.run(["git", *args], cwd=repo_dir, check=True,
                       capture_output=True)
    run("init")
    run("config", "user.email", "t@t.de")
    run("config", "user.name", "Tester")
    f = repo_dir / "prog.gb"
    f.write_text("PRINT 1\n", encoding="utf-8")
    run("add", "prog.gb")
    run("-c", "commit.gpgsign=false", "commit", "-m", "init prog")

    assert is_git_repo(repo_dir) is True
    res = blame(str(f))
    assert res.ok is True
    assert res.lines[0].author == "Tester"
    assert res.lines[0].summary == "init prog"
    assert res.lines[0].sha and not res.lines[0].uncommitted

    # Uncommitted Zeile dazu -> als uncommitted erkannt.
    f.write_text("PRINT 1\nPRINT 2\nPRINT 3\n", encoding="utf-8")
    res2 = blame(str(f))
    assert res2.ok is True
    assert len(res2.lines) == 3
    assert res2.lines[2].uncommitted is True
