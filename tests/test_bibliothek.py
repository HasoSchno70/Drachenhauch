"""Bibliotheken teilen (Punkt 5 aus docs/allzweck-audit-2.md).

`IMPORT "x.dh"` loeste bis hierher **ausschliesslich relativ zur
importierenden Datei** auf. Eine geteilte Bibliothek musste also in jedes
Projekt kopiert werden -- damit gibt es keine Aktualisierung, kein "ich
benutze dieselbe Datumsbibliothek wie du" und keinen Ort, an dem eine
Gemeinschaft etwas ablegen koennte.

Gesucht wird jetzt in dieser Reihenfolge:

1. neben der importierenden Datei (die eigene Kopie gewinnt IMMER)
2. jeder Ordner aus `DH_PATH`
3. `<Benutzerordner>/.drachenhauch/bibliothek`

Die Aufloesung steht ZWEIMAL im Baum: in `preprocess.rs` (zum Ausfuehren) und
in `drachenhauch/preprocess.py` (der Editor braucht die Zeilen-Herkunft fuer
seine Fehlermeldungen). Beide muessen dasselbe tun, sonst zeigt der Editor
Fehler in Programmen, die laufen -- die letzten Tests hier halten das fest.
"""
import os
import subprocess
import sys
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

BIB = ("' Kleine Bibliothek.\n"
       "FUNCTION Tage(von AS INTEGER, bis AS INTEGER) AS INTEGER\n"
       "    RETURN (bis - von) \\ 86400\n"
       "END FUNCTION\n")


def _lauf(datei: Path, umgebung: dict | None = None):
    env = dict(os.environ)
    # Aus einer geerbten Einstellung darf kein Test-Ergebnis werden.
    env.pop("DH_PATH", None)
    if umgebung:
        env.update(umgebung)
    r = subprocess.run([str(_DHRT), "run", str(datei)], capture_output=True,
                       text=True, encoding="utf-8", errors="replace",
                       env=env, timeout=60)
    return r.returncode, (r.stdout or ""), (r.stderr or "")


@pytest.fixture
def projekt(tmp_path):
    """Ein Projekt und eine Bibliothek daneben -- getrennte Ordner."""
    (tmp_path / "lib").mkdir()
    (tmp_path / "proj").mkdir()
    (tmp_path / "lib" / "zeitraum.dh").write_text(BIB, encoding="utf-8")
    (tmp_path / "proj" / "haupt.dh").write_text(
        'IMPORT "zeitraum.dh"\nPRINT Tage(0, 172800)\n', encoding="utf-8")
    return tmp_path


def test_ohne_suchpfad_bleibt_es_ein_fehler(projekt):
    code, _, err = _lauf(projekt / "proj" / "haupt.dh")
    assert code != 0
    assert "nicht gefunden" in err


def test_die_meldung_nennt_alle_gesuchten_orte(projekt):
    """Sonst raet man, warum die Datei nicht gefunden wird."""
    _, _, err = _lauf(projekt / "proj" / "haupt.dh")
    assert "proj" in err                       # neben der Datei
    assert ".drachenhauch" in err              # der Benutzerordner


def test_dh_path_findet_die_bibliothek(projekt):
    code, out, err = _lauf(projekt / "proj" / "haupt.dh",
                           {"DH_PATH": str(projekt / "lib")})
    assert code == 0, err
    assert out.strip() == "2"


def test_die_eigene_kopie_gewinnt(projekt):
    """Wer eine Datei danebenlegt, will genau die -- und keine, die
    irgendwo auf dem Rechner liegt und sich unbemerkt aendert."""
    (projekt / "proj" / "zeitraum.dh").write_text(
        "FUNCTION Tage(von AS INTEGER, bis AS INTEGER) AS INTEGER\n"
        "    RETURN 999\n"
        "END FUNCTION\n", encoding="utf-8")
    _, out, _ = _lauf(projekt / "proj" / "haupt.dh",
                      {"DH_PATH": str(projekt / "lib")})
    assert out.strip() == "999"


def test_mehrere_ordner_in_der_reihenfolge(projekt):
    (projekt / "lib2").mkdir()
    (projekt / "lib2" / "extra.dh").write_text(
        "FUNCTION Drei() AS INTEGER\n    RETURN 3\nEND FUNCTION\n", encoding="utf-8")
    (projekt / "proj" / "zwei.dh").write_text(
        'IMPORT "zeitraum.dh"\nIMPORT "extra.dh"\nPRINT Tage(0, 86400) + Drei()\n',
        encoding="utf-8")
    pfad = os.pathsep.join([str(projekt / "lib"), str(projekt / "lib2")])
    code, out, err = _lauf(projekt / "proj" / "zwei.dh", {"DH_PATH": pfad})
    assert code == 0, err
    assert out.strip() == "4"


def test_der_erste_treffer_gewinnt(projekt):
    """Bei zwei gleichnamigen Dateien entscheidet die Reihenfolge in
    DH_PATH -- wie bei PATH und PYTHONPATH."""
    (projekt / "lib2").mkdir()
    (projekt / "lib2" / "zeitraum.dh").write_text(
        "FUNCTION Tage(von AS INTEGER, bis AS INTEGER) AS INTEGER\n"
        "    RETURN 42\nEND FUNCTION\n", encoding="utf-8")
    vorn = os.pathsep.join([str(projekt / "lib2"), str(projekt / "lib")])
    _, out, _ = _lauf(projekt / "proj" / "haupt.dh", {"DH_PATH": vorn})
    assert out.strip() == "42"


def test_benutzerordner_wird_durchsucht(projekt, tmp_path):
    """`<heim>/.drachenhauch/bibliothek` -- der Ort, an dem eine
    Paketverwaltung spaeter ablegen wuerde."""
    heim = tmp_path / "heim"
    ziel = heim / ".drachenhauch" / "bibliothek"
    ziel.mkdir(parents=True)
    (ziel / "zeitraum.dh").write_text(BIB, encoding="utf-8")
    code, out, err = _lauf(projekt / "proj" / "haupt.dh",
                           {"USERPROFILE": str(heim), "HOME": str(heim)})
    assert code == 0, err
    assert out.strip() == "2"


def test_eine_bibliothek_darf_selbst_importieren(projekt):
    """Und zwar relativ zu SICH -- nicht relativ zum Hauptprogramm."""
    (projekt / "lib" / "basis.dh").write_text(
        "FUNCTION Verdopple(x AS INTEGER) AS INTEGER\n    RETURN x * 2\nEND FUNCTION\n",
        encoding="utf-8")
    (projekt / "lib" / "oben.dh").write_text(
        'IMPORT "basis.dh"\n'
        "FUNCTION Vierfach(x AS INTEGER) AS INTEGER\n"
        "    RETURN Verdopple(Verdopple(x))\nEND FUNCTION\n", encoding="utf-8")
    (projekt / "proj" / "tief.dh").write_text(
        'IMPORT "oben.dh"\nPRINT Vierfach(3)\n', encoding="utf-8")
    code, out, err = _lauf(projekt / "proj" / "tief.dh",
                           {"DH_PATH": str(projekt / "lib")})
    assert code == 0, err
    assert out.strip() == "12"


def test_namensraum_geht_auch_aus_der_bibliothek(projekt):
    """`IMPORT "x.dh" AS y` (WP I.1) darf am Fundort nicht haengen."""
    (projekt / "proj" / "ns.dh").write_text(
        'IMPORT "zeitraum.dh" AS zr\nPRINT zr.Tage(0, 259200)\n', encoding="utf-8")
    code, out, err = _lauf(projekt / "proj" / "ns.dh",
                           {"DH_PATH": str(projekt / "lib")})
    assert code == 0, err
    assert out.strip() == "3"


def test_eingebaute_module_gehen_vor_dem_suchpfad_nicht_verloren(projekt):
    """`IMPORT "json"` bleibt das eingebaute Modul -- der Suchpfad darf
    daran nichts aendern."""
    (projekt / "lib" / "json").write_text("' kein gueltiges Programm\n", encoding="utf-8")
    (projekt / "proj" / "j.dh").write_text(
        'IMPORT "json"\nDIM h AS JSON_HANDLE\nh = JSON_NEW_OBJECT()\n'
        'JSON_SET_INT(h, "a", 1)\nPRINT JSON_STRINGIFY(h)\n', encoding="utf-8")
    code, out, err = _lauf(projekt / "proj" / "j.dh", {"DH_PATH": str(projekt / "lib")})
    assert code == 0, err
    assert out.strip() == '{"a":1}'


# --------------------------------------------------- beide Implementierungen
def test_python_und_rust_suchen_dieselben_orte(monkeypatch, tmp_path):
    """Der Editor loest IMPORTs in Python noch einmal auf (Zeilen-Herkunft).
    Kennt eine der beiden Seiten einen Ordner mehr, zeigt der Editor Fehler
    in Programmen, die laufen -- oder schweigt zu welchen, die brechen."""
    from drachenhauch.preprocess import bibliothekspfade
    heim = tmp_path / "heim"
    monkeypatch.setenv("DH_PATH", os.pathsep.join(["/a", "/b"]))
    monkeypatch.setenv("USERPROFILE", str(heim))
    monkeypatch.setenv("HOME", str(heim))
    py = [str(p) for p in bibliothekspfade()]
    assert py[:2] == [str(Path("/a")), str(Path("/b"))]
    assert py[-1] == str(heim / ".drachenhauch" / "bibliothek")


def test_python_findet_die_bibliothek_auch(projekt, monkeypatch):
    """Dieselbe Datei, derselbe Fund -- sonst driftet der Editor weg."""
    from drachenhauch.preprocess import process
    monkeypatch.setenv("DH_PATH", str(projekt / "lib"))
    quelle = (projekt / "proj" / "haupt.dh").read_text(encoding="utf-8")
    merged, _ = process(quelle, projekt / "proj", file_label="haupt.dh")
    assert "FUNCTION Tage" in merged


def test_python_meldet_dieselben_orte(projekt, monkeypatch):
    from drachenhauch.errors import LexerError
    from drachenhauch.preprocess import process
    monkeypatch.delenv("DH_PATH", raising=False)
    quelle = (projekt / "proj" / "haupt.dh").read_text(encoding="utf-8")
    with pytest.raises(LexerError) as e:
        process(quelle, projekt / "proj", file_label="haupt.dh")
    assert "nicht gefunden" in str(e.value)
    assert ".drachenhauch" in str(e.value)
