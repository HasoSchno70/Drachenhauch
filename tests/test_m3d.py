"""Golden-Tests fuer das m3d-Modul (VEC3/VEC4/QUAT/MAT4).

run_gb spawnt `dhrt run` -> skippt automatisch, wenn dhrt nicht gebaut ist.
f32-intern: bei nicht-exakten Werten via ROUND vergleichen.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/m3d.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import pytest

from drachenhauch.errors import DrachenhauchError


def _gb(body: str) -> str:
    return 'IMPORT "m3d"\n' + body


# --------------------------------------------------------------- VEC3


# --------------------------------------------------------------- VEC4


# --------------------------------------------------------------- QUAT


# --------------------------------------------------------------- MAT4


# --------------------------------------------------------------- Typ-Fehler


# --------------------------------------------------------- MODEL_INSTANCED
# Die Argument-Validierung von MODEL_INSTANCED laeuft in vm.rs, BEVOR der
# Grafik-Kontext angefasst wird -> diese Fehlerpfade brauchen kein Fenster.


def test_model_instanced_headless_render(tmp_path):
    """Echter Render-Pfad: die Instancing-Demo laeuft headless (DHRT_FRAMES)
    durch DrawMeshInstanced und schreibt einen Screenshot."""
    import os
    import subprocess
    from pathlib import Path

    root = Path(__file__).resolve().parent.parent
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    dhrt = next((root / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (root / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)
    if dhrt is None:
        pytest.skip("native Runtime 'dhrt' nicht gebaut")

    shot = tmp_path / "instanced.png"
    env = dict(os.environ, DHRT_FRAMES="2", DHRT_SCREENSHOT=str(shot))
    demo = root / "examples" / "104_instancing.dh"
    r = subprocess.run([str(dhrt), "run", str(demo)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60, env=env)
    assert r.returncode == 0, f"dhrt Exit {r.returncode}: {r.stderr}"
    assert shot.exists() and shot.stat().st_size > 0, "kein Screenshot erzeugt"


# --- Vorbelegung der Mathe-Typen -----------------------------------------
# Frueher waren MAT4/QUAT/VEC* nach dem DIM NIL: jede Rechnung darauf schlug
# fehl, und ein Array liess sich nicht schrittweise fuellen. Jetzt starten sie
# mit ihrem NEUTRALEN Element -- so wie INTEGER mit 0 und STRING mit "".


# --- MODEL_INSTANCED mit Farb-Array ---------------------------------------
# Der Instancing-Shader kennt nur EINE Farbe je Draw-Call (raylibs
# DrawMeshInstanced uebertraegt nur Matrizen). Ein Farb-Array wird deshalb
# nach Farben gruppiert: ein Draw-Call je VERSCHIEDENER Farbe.
