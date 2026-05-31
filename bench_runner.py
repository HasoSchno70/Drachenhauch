"""Sammelt Benchmark-Ergebnisse fuer alle bench_*.gb-Files und liefert
eine Markdown-Tabelle mit Tree-Walker / Python-VM / Cython-VM-Zeiten.

Nutzung:
    python bench_runner.py
"""
import os
import re
import subprocess
import sys
from pathlib import Path


PYTHON = str(Path(sys.executable).resolve())
ROOT = Path(__file__).resolve().parent

BENCHES = [
    "bench_loop",
    "bench_fib",
    "bench_array_rw",
    "bench_method_dispatch",
    "bench_builtin_math",
    "bench_string_concat",
    "bench_ecs_movement",
    "bench_dict_comp",
]


def run(name: str) -> dict:
    p = subprocess.run(
        [PYTHON, str(ROOT / "gbrun.py"), "--bench", f"examples/{name}.gb"],
        capture_output=True, text=True, timeout=600,
        cwd=str(ROOT),
    )
    txt = p.stdout
    tw = re.search(r"Tree-Walker:\s+([\d.]+)\s+ms", txt)
    pv = re.search(r"Python-VM:\s+([\d.]+)\s+ms", txt)
    nv = re.search(r"Native-VM:\s+([\d.]+)\s+ms", txt)
    ident = "ALLE IDENTISCH" in txt
    return {
        "name": name,
        "tw": float(tw.group(1)) if tw else None,
        "pv": float(pv.group(1)) if pv else None,
        "nv": float(nv.group(1)) if nv else None,
        "ident": ident,
        "raw": txt,
    }


def main():
    results = [run(b) for b in BENCHES]

    print("| Benchmark              |  Tree-Walker |   Python-VM |   Native-VM | TW->Py | TW->Cy | Py->Cy | Ident |")
    print("|------------------------|--------------|-------------|-------------|--------|--------|--------|-------|")
    for r in results:
        if r["tw"] is None:
            print(f"| {r['name']:22s} | (Bench fehlgeschlagen)                                                              |")
            print(r["raw"][-400:])
            continue
        sp_pv = r["tw"] / r["pv"]
        sp_nv = r["tw"] / r["nv"]
        sp_pv_nv = r["pv"] / r["nv"]
        ok = "yes" if r["ident"] else "NO"
        print(
            f"| {r['name']:22s} "
            f"| {r['tw']:9.1f} ms "
            f"| {r['pv']:8.1f} ms "
            f"| {r['nv']:8.1f} ms "
            f"| {sp_pv:5.2f}x "
            f"| {sp_nv:5.2f}x "
            f"| {sp_pv_nv:5.2f}x "
            f"| {ok:5s} |"
        )


if __name__ == "__main__":
    main()
