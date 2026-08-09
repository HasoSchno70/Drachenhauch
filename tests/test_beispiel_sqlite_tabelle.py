"""examples/158 -- die gui-Tabelle an einer echten SQLite-Datenbank.

Der Punkt, den dieser Test absichert, ist die ZUSAGE der Demo: sie arbeitet auf
einer Kopie und laesst die Originaldatei unangetastet. Eine Demo, die den
Spielstand des Nutzers aendert, waere eine schlechte Demo -- und ein Fehler
darin faellt sonst erst auf, wenn die Daten schon weg sind.
"""
import os
import shutil
import sqlite3
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_DEMO = _ROOT / "examples" / "158_gui_tabelle_sqlite.gb"
_ORIGINAL = _ROOT / "pyramid_pusher" / "pyramid_pusher.db"


def _gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


def _zustand(pfad):
    c = sqlite3.connect(pfad)
    try:
        return c.execute("SELECT id, title FROM solves ORDER BY id").fetchall()
    finally:
        c.close()


def test_demo_laesst_die_originaldatenbank_unberuehrt(tmp_path):
    gbrt = _gbrt()
    if gbrt is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    if not _ORIGINAL.exists():
        pytest.skip("pyramid_pusher.db nicht vorhanden")

    vorher = _zustand(_ORIGINAL)
    stempel = _ORIGINAL.stat().st_mtime_ns

    r = subprocess.run([str(gbrt), "run", str(_DEMO)], capture_output=True,
                       text=True, encoding="utf-8", timeout=120,
                       env=dict(os.environ, GBRT_FRAMES="20"))
    assert r.returncode == 0, f"gbrt Exit {r.returncode}: {r.stderr}"

    assert _zustand(_ORIGINAL) == vorher, "die Demo hat die Originaldatenbank geaendert"
    assert _ORIGINAL.stat().st_mtime_ns == stempel, "die Originaldatei wurde geschrieben"


def test_demo_raeumt_ihre_arbeitskopie_weg(tmp_path):
    """Sonst bliebe nach jedem Lauf eine .db im examples-Ordner liegen."""
    gbrt = _gbrt()
    if gbrt is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    if not _ORIGINAL.exists():
        pytest.skip("pyramid_pusher.db nicht vorhanden")

    kopie = _ROOT / "examples" / "tabelle_sqlite_arbeitskopie.db"
    kopie.unlink(missing_ok=True)
    r = subprocess.run([str(gbrt), "run", str(_DEMO)], capture_output=True,
                       text=True, encoding="utf-8", timeout=120,
                       env=dict(os.environ, GBRT_FRAMES="20"))
    assert r.returncode == 0, r.stderr
    assert not kopie.exists(), "Arbeitskopie blieb liegen"


def test_kopie_enthaelt_dieselben_daten(tmp_path):
    """VACUUM INTO ist der Trick, mit dem die Demo an eine beschreibbare
    Fassung kommt, ohne das Original anzufassen -- er muss wirklich ALLES
    kopieren, sonst zeigte die Tabelle weniger an als das Spiel gespeichert hat.
    """
    gbrt = _gbrt()
    if gbrt is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    if not _ORIGINAL.exists():
        pytest.skip("pyramid_pusher.db nicht vorhanden")

    quelle = tmp_path / "quelle.db"
    shutil.copy(_ORIGINAL, quelle)
    ziel = tmp_path / "kopie.db"
    prog = tmp_path / "vac.gb"
    prog.write_text(
        'IMPORT "db"\n'
        'DIM c AS DB_CONN\n'
        f'c = DB_OPEN("{quelle.name}")\n'
        f"DB_EXEC(c, \"VACUUM INTO '{ziel.name}'\")\n"
        'DB_CLOSE(c)\n', encoding="utf-8")
    r = subprocess.run([str(gbrt), "run", str(prog)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    assert r.returncode == 0, r.stderr
    assert _zustand(ziel) == _zustand(quelle)
