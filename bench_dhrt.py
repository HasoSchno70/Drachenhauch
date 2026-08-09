"""Benchmark-Runner fuer die native Runtime dhrt.

Misst die Wandzeit von `dhrt run <datei.gb>` (Prozess inkl. Compile+Startup;
bei grossen Benches dominiert die VM-Schleife). Best-of-N gegen Rauschen.

Nutzung:
    python bench_dhrt.py [datei.gb ...]          # Default: examples/bench_*.gb
    python bench_dhrt.py --dhrt pfad\\zu\\dhrt.exe  # andere Exe (A/B-Vergleich)
    python bench_dhrt.py --runs 7                # Wiederholungen (Default 5)
"""
import argparse
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def find_dhrt() -> Path | None:
    base = ROOT / "rust" / "drachenhauch_runtime" / "target"
    for variant in ("release", "debug"):
        p = base / variant / "dhrt.exe"
        if p.exists():
            return p
    return None


def bench(dhrt: Path, gb_file: Path, runs: int) -> float:
    best = float("inf")
    for _ in range(runs):
        t0 = time.perf_counter()
        r = subprocess.run([str(dhrt), "run", str(gb_file)],
                           capture_output=True, text=True, timeout=600)
        dt = (time.perf_counter() - t0) * 1000.0
        if r.returncode != 0:
            print(f"  FEHLER ({gb_file.name}): {r.stderr.strip()[:200]}")
            return float("nan")
        best = min(best, dt)
    return best


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="*", help=".gb-Dateien (Default: examples/bench_*.gb)")
    ap.add_argument("--dhrt", default=None, help="Pfad zur dhrt-Exe")
    ap.add_argument("--runs", type=int, default=5)
    args = ap.parse_args()

    dhrt = Path(args.dhrt) if args.dhrt else find_dhrt()
    if not dhrt or not dhrt.exists():
        print("dhrt nicht gefunden -- erst bauen: python rust/build_runtime.py")
        return 1

    files = [Path(f) for f in args.files] if args.files else \
        sorted((ROOT / "examples").glob("bench_*.gb"))

    print(f"dhrt: {dhrt}")
    print(f"{'Bench':32s} {'best-of-' + str(args.runs):>12s}")
    print("-" * 46)
    for f in files:
        ms = bench(dhrt, f, args.runs)
        print(f"{f.name:32s} {ms:10.1f} ms")
    return 0


if __name__ == "__main__":
    sys.exit(main())
