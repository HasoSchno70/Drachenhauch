"""Eine exportierte Spiel-.exe muss sich NACHTRAEGLICH signieren lassen.

`dhrt --export` haengt `<gbc><laenge><DHRTPAY1>` an eine Kopie der Runtime.
Signieren haengt den Zertifikatsblock ebenfalls ans Dateiende -- danach sind
die letzten 16 Bytes nicht mehr unsere. Frueher suchte `embedded_gbc()` genau
dort: das signierte Spiel fand sich selbst nicht mehr und verhielt sich wie
ein blankes `dhrt` ("Verwendung: ... <datei.gbc>").

Andersherum geht es nicht -- erst signieren, dann anhaengen zerstoert die
Signatur (gemessen: aus `Valid` wird `NotSigned`). Die Reihenfolge muss also
"exportieren, dann signieren" sein, und dieser Test haelt das fest.

Signiert wird hier nicht wirklich (das braeuchte ein Zertifikat); angehaengte
Bytes bilden den Zertifikatsblock strukturgleich nach.
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


def _exportiere(tmp_path, quelltext):
    gb = tmp_path / "spiel.gb"
    gb.write_text(quelltext, encoding="utf-8")
    res = subprocess.run([str(_DHRT), "--export", str(gb)], cwd=str(tmp_path),
                         capture_output=True, text=True, timeout=180)
    assert res.returncode == 0, res.stderr
    exe = tmp_path / "spiel_dist" / ("spiel.exe" if os.name == "nt" else "spiel")
    assert exe.exists(), f"kein Bundle erzeugt: {res.stdout}{res.stderr}"
    return exe


def _lauf(exe):
    return subprocess.run([str(exe)], capture_output=True, text=True, timeout=120)


def test_exportiertes_spiel_laeuft(tmp_path):
    exe = _exportiere(tmp_path, 'PRINT "hallo aus dem bundle"\n')
    res = _lauf(exe)
    assert res.returncode == 0, res.stderr
    assert "hallo aus dem bundle" in res.stdout


@pytest.mark.parametrize("zusatz", [
    4096,      # typische Groessenordnung eines Zertifikatsblocks
    16,        # exakt so viel, wie der Footer lang ist -- Grenzfall
    1,         # ein einziges Byte reicht schon, um den Footer zu verschieben
])
def test_bleibt_lauffaehig_wenn_hinten_etwas_angehaengt_wird(tmp_path, zusatz):
    exe = _exportiere(tmp_path, 'PRINT "immer noch da"\n')
    with exe.open("ab") as f:
        f.write(b"\x00" * zusatz)

    res = _lauf(exe)
    assert res.returncode == 0, res.stderr
    assert "immer noch da" in res.stdout, (
        f"Nutzlast nach {zusatz} angehaengten Bytes nicht mehr gefunden -- "
        f"ausgegeben wurde: {res.stdout!r}")


def test_blanke_runtime_bleibt_blank(tmp_path):
    """Die Gegenprobe: ohne Nutzlast darf dhrt sich NICHT als Spiel ausgeben.

    Die Rueckwaertssuche koennte sonst irgendwo im Binaercode etwas finden
    und eine leere .exe fuer ein Bundle halten."""
    kopie = tmp_path / _DHRT.name
    kopie.write_bytes(_DHRT.read_bytes())
    res = _lauf(kopie)
    assert "Verwendung" in (res.stdout + res.stderr), (
        "blanke Runtime haette die Verwendungs-Hilfe zeigen muessen, "
        f"kam aber mit: {res.stdout!r}")
