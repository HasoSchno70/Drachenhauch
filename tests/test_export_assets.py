"""Export-Test: `dhrt --export` muss die vom Programm referenzierten Assets ins
Bundle kopieren -- auch ueber `../`-Pfade (z.B. ein Spiel in `code/` mit Assets
in `../assets/`). Frueher blieb das Bundle assetlos -> beim Doppelklick nur
schwarzer Bildschirm + sofortiges Beenden (LOADIMAGE-Fehler).
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for variant in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / variant / exe
        if p.exists():
            return p
    return None


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")


def test_export_bundles_parent_relative_assets(tmp_path):
    # Projekt: code/game.dh referenziert ../assets/... (eine Ebene hoeher).
    proj = tmp_path / "proj"
    (proj / "code").mkdir(parents=True)
    (proj / "assets" / "sprites").mkdir(parents=True)
    (proj / "assets" / "sprites" / "hero.png").write_bytes(b"\x89PNG\r\n\x1a\n fake")
    (proj / "assets" / "snd.wav").write_bytes(b"RIFF fake")
    gb = proj / "code" / "game.dh"
    gb.write_text(
        'DIM img AS IMAGE\n'
        'img = LOADIMAGE("../assets/sprites/hero.png")\n'
        'DIM ok AS BOOLEAN : ok = FILEEXISTS("../assets/snd.wav")\n'
        'PRINT "hi"\n',
        encoding="utf-8",
    )
    out = tmp_path / "dist"
    r = subprocess.run([str(_DHRT), "--export", str(gb), str(out)],
                       capture_output=True, text=True, timeout=60)
    assert r.returncode == 0, r.stderr

    # Exe da?
    exe = out / ("game.exe" if os.name == "nt" else "game")
    assert exe.exists(), f"Export-Exe fehlt ({list(out.iterdir())})"
    # Assets normalisiert (ohne ../) ins Bundle kopiert?
    assert (out / "assets" / "sprites" / "hero.png").is_file()
    assert (out / "assets" / "snd.wav").is_file()


def test_export_ignores_absolute_and_missing(tmp_path):
    # Absolute Pfade + nicht existierende Dateien duerfen den Export nicht
    # stoeren und nichts Falsches einsammeln.
    proj = tmp_path / "p2"
    proj.mkdir()
    gb = proj / "g.dh"
    gb.write_text(
        'PRINT "C:/windows/system32/x.dll"\n'      # absolut -> ignorieren
        'PRINT "../gibtsnicht/foo.png"\n'           # existiert nicht -> ignorieren
        'PRINT "hallo welt"\n',
        encoding="utf-8",
    )
    out = tmp_path / "d2"
    r = subprocess.run([str(_DHRT), "--export", str(gb), str(out)],
                       capture_output=True, text=True, timeout=60)
    assert r.returncode == 0, r.stderr
    assert (out / ("g.exe" if os.name == "nt" else "g")).exists()
    # kein assets-Ordner, keine system32-Kopie
    assert not (out / "assets").exists()
    assert not (out / "windows").exists()
