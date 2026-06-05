"""Tests fuer die wiederverwendbare Preset-Bibliothek (Qt-frei)."""
import json

from gamebasic.editor_qt.preset_library import PresetLibrary


def test_add_get_names_roundtrip(tmp_path):
    lib = PresetLibrary(tmp_path / "p.json")
    assert lib.names() == []
    lib.add("Blau", {"color": 0x0000FF})
    lib.add("Rot", {"color": 0xFF0000})
    assert lib.names() == ["Blau", "Rot"]
    assert lib.get("Rot") == {"color": 0xFF0000}
    assert lib.get("fehlt") is None


def test_persistence(tmp_path):
    path = tmp_path / "p.json"
    PresetLibrary(path).add("X", {"a": 1})
    # Neue Instanz laedt die Datei
    lib2 = PresetLibrary(path)
    assert lib2.get("X") == {"a": 1}


def test_builtins_plus_user(tmp_path):
    lib = PresetLibrary(tmp_path / "p.json",
                        builtins={"Feuer": {"v": 1}})
    assert lib.names() == ["Feuer"]
    assert lib.is_builtin("Feuer") is True
    # User-Preset gleichen Namens ueberschreibt (in Anzeige) das Builtin
    lib.add("Feuer", {"v": 2})
    assert lib.get("Feuer") == {"v": 2}
    assert lib.is_builtin("Feuer") is False


def test_remove_only_user(tmp_path):
    lib = PresetLibrary(tmp_path / "p.json", builtins={"B": {"x": 0}})
    lib.add("U", {"y": 1})
    assert lib.remove("U") is True
    assert lib.remove("U") is False         # weg
    assert lib.remove("B") is False         # Builtin nicht loeschbar
    assert "B" in lib.names()


def test_corrupt_file_yields_empty(tmp_path):
    path = tmp_path / "p.json"
    path.write_text("{ kaputt", encoding="utf-8")
    lib = PresetLibrary(path)
    assert lib.names() == []                # faellt sauber auf leer zurueck


def test_only_user_saved_not_builtins(tmp_path):
    path = tmp_path / "p.json"
    lib = PresetLibrary(path, builtins={"B": {"x": 1}})
    lib.add("U", {"y": 2})
    data = json.loads(path.read_text(encoding="utf-8"))
    assert data == {"U": {"y": 2}}          # Builtins NICHT in der Datei


def test_add_empty_name_raises(tmp_path):
    import pytest
    lib = PresetLibrary(tmp_path / "p.json")
    with pytest.raises(ValueError):
        lib.add("   ", {"a": 1})
