"""Baut die nativen Rust-Module fuer GameBasic und legt die .pyd-Dateien
neben den Cython-Modulen in gamebasic/ ab.

Verwendung (aus dem Projekt-Root oder beliebig):
    .venv\\Scripts\\python.exe rust\\build.py [--debug]

Voraussetzung: Rust-Toolchain (cargo) im PATH. PyO3 wird von cargo
automatisch von crates.io geholt.

Analog zu `setup.py build_ext --inplace` fuer die Cython-Module. Wenn die
.pyd fehlt, faellt das jeweilige Modul auf seine reine Python-Variante
zurueck (z.B. astar -> _AStarGrid).
"""
import os
import shutil
import subprocess
import sys
from pathlib import Path

# crate-name -> Ziel-.pyd relativ zu gamebasic/
CRATES = {
    "gb_native": "gb_native.pyd",
}

ROOT = Path(__file__).resolve().parent.parent      # Projekt-Root
RUST_DIR = ROOT / "rust"
PKG_DIR = ROOT / "gamebasic"


def main() -> int:
    debug = "--debug" in sys.argv
    profile_dir = "debug" if debug else "release"
    env = dict(os.environ)
    # PyO3 soll gegen genau diesen Interpreter konfigurieren.
    env["PYO3_PYTHON"] = sys.executable

    for crate, pyd_name in CRATES.items():
        crate_dir = RUST_DIR / crate
        print(f"== Baue {crate} ({profile_dir}) ==")
        cmd = ["cargo", "build", "--manifest-path", str(crate_dir / "Cargo.toml")]
        if not debug:
            cmd.append("--release")
        rc = subprocess.call(cmd, env=env)
        if rc != 0:
            print(f"FEHLER: cargo build fuer {crate} fehlgeschlagen (rc={rc})")
            return rc

        # cargo legt auf Windows eine .dll, auf Linux .so, auf macOS .dylib ab.
        target = crate_dir / "target" / profile_dir
        src = None
        for ext in (".dll", ".so", ".dylib"):
            cand = target / f"{crate}{ext}"
            if cand.exists():
                src = cand
                break
            # Linux/macOS praefixen mit 'lib'
            cand = target / f"lib{crate}{ext}"
            if cand.exists():
                src = cand
                break
        if src is None:
            print(f"FEHLER: kein Build-Artefakt fuer {crate} in {target} gefunden")
            return 1

        dst = PKG_DIR / pyd_name
        shutil.copyfile(src, dst)
        print(f"   {src.name} -> {dst}")

    print("Fertig.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
