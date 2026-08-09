"""VM-Härtung: ein kaputter/abgeschnittener `.gbc` (Stack-Underflow) muss eine
saubere Laufzeit-Fehlermeldung liefern statt eines Rust-Panics.

Hintergrund: die VM-Dispatch-Schleife poppte den Operanden-Stack frueher mit
`stack.pop().unwrap()` -> bei kaputtem Bytecode = Prozess-Abbruch (Panic). Jetzt
`vm_pop(stack)?` -> normaler Err-Pfad ("VM: Stack underflow ...").
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_dhrt():
    base = _ROOT / "rust" / "drachenhauch_runtime" / "target"
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="dhrt nicht gebaut")


def _dump_gbc(tmp_path) -> dict:
    """Ein valides .gbc-JSON-Skelett (via `dhrt --dumpbc`) holen."""
    src = tmp_path / "tiny.gb"
    src.write_text("PRINT 1 + 2\n", encoding="utf-8")
    r = subprocess.run([str(_DHRT), "--dumpbc", str(src)],
                       capture_output=True, text=True, encoding="utf-8", timeout=30)
    assert r.returncode == 0, r.stderr
    return json.loads(r.stdout)


def _run_gbc(tmp_path, data: dict) -> subprocess.CompletedProcess:
    p = tmp_path / "corrupt.gbc"   # NICHT .gb -> dhrt nimmt den .gbc-Pfad
    p.write_text(json.dumps(data), encoding="utf-8")
    return subprocess.run([str(_DHRT), str(p)], capture_output=True,
                          text=True, encoding="utf-8", timeout=30)


def test_stack_underflow_is_clean_error_not_panic(tmp_path):
    data = _dump_gbc(tmp_path)
    # main: ADD (op 20) auf leerem Stack -> Underflow, danach HALT (99).
    data["main"]["constants"] = []
    data["main"]["code"] = [[20, None], [99, None]]
    data["main"]["lines"] = [1, 1]
    r = _run_gbc(tmp_path, data)

    out = (r.stdout or "") + (r.stderr or "")
    assert r.returncode != 0                       # Fehler, aber sauber beendet
    assert "Stack underflow" in out                # unsere Meldung
    assert "panicked" not in out.lower()           # KEIN Rust-Panic
    assert "RUST_BACKTRACE" not in out


def test_normal_program_still_runs(tmp_path):
    # Regressions-Sicherung: ein gueltiges .gbc laeuft unveraendert.
    data = _dump_gbc(tmp_path)
    r = _run_gbc(tmp_path, data)
    assert r.returncode == 0
    assert r.stdout.strip() == "3"
