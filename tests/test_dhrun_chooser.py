"""dhrun: ohne Argumente erscheint der Start-Dialog (Code-Editor / Form-Designer).
Explizite Flags (--editor/--form/Datei) umgehen ihn weiterhin."""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import dhrun


def test_no_args_routes_to_chooser(monkeypatch):
    called = {}
    monkeypatch.setattr(dhrun, "_launch_chooser",
                        lambda root: called.setdefault("x", True) and 0 or 0)
    rc = dhrun.main(["dhrun.py"])
    assert rc == 0 and called.get("x")


def test_file_arg_does_not_open_chooser(monkeypatch, tmp_path):
    # Eine Datei als Argument fuehrt NICHT zum Dialog (sondern zum Run-Pfad).
    monkeypatch.setattr(dhrun, "_launch_chooser",
                        lambda root: (_ for _ in ()).throw(AssertionError("Chooser darf nicht aufgehen")))
    monkeypatch.setattr(dhrun, "_run_native", lambda abs_path, path: 0)
    f = tmp_path / "x.dh"
    f.write_text('PRINT 1\n', encoding="utf-8")
    # main() macht os.chdir ins Datei-Verzeichnis -> danach wiederherstellen,
    # sonst laufen spaetere subprocess-Tests (z.B. LSP) im falschen CWD.
    cwd = os.getcwd()
    try:
        assert dhrun.main(["dhrun.py", str(f)]) == 0
    finally:
        os.chdir(cwd)


# Hinweis: der Dialog-Bau (_launch_chooser mit modalem exec) wird NICHT als Test
# gefahren -- ein modaler Qt-exec im geteilten pytest-Prozess ist instabil
# (Segfault). Offscreen-Bau wurde manuell verifiziert; die Routing-Tests oben
# decken die dhrun-Verdrahtung ab.
