"""Baut `gbrt` als WebAssembly (emscripten) fuer den Web-Playground.

**Status: experimentell.** Die native Runtime nutzt raylib (raylib-rs); ein
funktionierender Web-Build braucht die emscripten-Toolchain (`emcc`) UND den
Rust-Target `wasm32-unknown-emscripten`. Beides ist in dieser Umgebung nicht
installiert -> das Skript kompiliert dann nur die `.gbc` und gibt eine
Anleitung aus, statt zu scheitern.

Ablauf (mit vollstaendiger Toolchain):
  1. `<datei.gb>` -> Compiler -> `web/program.gbc`
  2. `cargo build --target wasm32-unknown-emscripten --features graphics --release`
     mit emscripten-Linker-Flags (ASYNCIFY fuer den blockierenden VM-Loop,
     GLFW3 fuer raylib, `--embed-file program.gbc@/program.gbc`).
  3. `gbrt.js` + `gbrt.wasm` nach `web/` kopieren.

Aufruf:
  .venv\\Scripts\\python.exe rust\\build_wasm.py examples\\01_hello.gb
  .venv\\Scripts\\python.exe rust\\build_wasm.py examples\\01_hello.gb web

Grenzen: siehe docs/web-playground.md (Render-Loop/ASYNCIFY, Audio, Threads).
"""
from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "rust" / "gb_runtime"
WEB = ROOT / "web"
TARGET = "wasm32-unknown-emscripten"

# emscripten-Linker-Flags (an raylibs Web-Beispiele angelehnt).
# ASYNCIFY laesst den blockierenden VM-/Render-Loop im Browser laufen, ohne die
# Engine auf emscripten_set_main_loop umzubauen (langsamer, aber funktioniert).
EMCC_FLAGS = [
    "-s", "USE_GLFW=3",
    "-s", "ASYNCIFY",
    "-s", "ALLOW_MEMORY_GROWTH=1",
    "-s", "ASSERTIONS=1",
    "-s", "EXPORTED_RUNTIME_METHODS=['callMain','FS','print']",
    "--embed-file", "program.gbc@/program.gbc",
]


def check_toolchain() -> dict:
    """Verfuegbarkeit der noetigen Tools (alles bool)."""
    info = {"emcc": shutil.which("emcc") is not None,
            "cargo": shutil.which("cargo") is not None,
            "wasm_target": False}
    if info["cargo"]:
        try:
            out = subprocess.run(["rustup", "target", "list", "--installed"],
                                 capture_output=True, text=True, timeout=30)
            info["wasm_target"] = TARGET in out.stdout
        except Exception:
            info["wasm_target"] = False
    return info


def compile_program(gb_path: str | Path, out_dir: str | Path = WEB) -> Path:
    """`.gb` -> `<out_dir>/program.gbc` (nutzt den normalen Python-Compiler)."""
    from gamebasic.serialize import compile_file_to_gbc
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    gbc = out_dir / "program.gbc"
    compile_file_to_gbc(gb_path, gbc)
    return gbc


def _print_manual(info: dict) -> None:
    print("\n--- WASM-Build uebersprungen (Toolchain unvollstaendig) ---")
    print(f"  emcc (emscripten):           {'OK' if info['emcc'] else 'FEHLT'}")
    print(f"  cargo:                       {'OK' if info['cargo'] else 'FEHLT'}")
    print(f"  rust-target {TARGET}: {'OK' if info['wasm_target'] else 'FEHLT'}")
    print("\nSo vervollstaendigen:")
    print("  1. emscripten installieren (https://emscripten.org), `emcc` in PATH.")
    print(f"  2. rustup target add {TARGET}")
    print("  3. Dieses Skript erneut ausfuehren.")
    print("\nDie program.gbc wurde bereits erzeugt. Manueller Build-Befehl:")
    print(f'  set EMCC_CFLAGS={" ".join(EMCC_FLAGS)}')
    print(f"  cargo build --manifest-path {CRATE / 'Cargo.toml'} \\")
    print(f"    --target {TARGET} --features graphics --release")
    print("  -> target/wasm32-unknown-emscripten/release/gbrt.{js,wasm} nach web/ kopieren")


def build(gb_path: str | Path, out_dir: str | Path = WEB) -> int:
    gbc = compile_program(gb_path, out_dir)
    print(f"kompiliert: {gbc}")
    info = check_toolchain()
    if not all(info.values()):
        _print_manual(info)
        return 0
    env = dict(os.environ)
    # emscripten-Flags + die einzubettende .gbc muss im CWD des Builds liegen.
    env["EMCC_CFLAGS"] = " ".join(EMCC_FLAGS)
    cmd = ["cargo", "build", "--manifest-path", str(CRATE / "Cargo.toml"),
           "--target", TARGET, "--features", "graphics", "--release"]
    print("Build:", " ".join(cmd))
    rc = subprocess.run(cmd, cwd=str(Path(out_dir)), env=env).returncode
    if rc != 0:
        print("cargo build fehlgeschlagen (siehe oben).")
        return rc
    rel = CRATE / "target" / TARGET / "release"
    for name in ("gbrt.js", "gbrt.wasm"):
        src = rel / name
        if src.exists():
            shutil.copy2(src, Path(out_dir) / name)
            print(f"kopiert: {Path(out_dir) / name}")
    print("\nFertig. Lokal testen:  py -m http.server -d web 8000  ->  http://localhost:8000")
    return 0


def main(argv=None) -> int:
    argv = argv if argv is not None else sys.argv
    args = argv[1:]
    if not args:
        print("Verwendung: build_wasm.py <datei.gb> [web-ordner]")
        return 1
    out = args[1] if len(args) > 1 else WEB
    return build(args[0], out)


if __name__ == "__main__":
    sys.path.insert(0, str(ROOT))
    sys.exit(main())
