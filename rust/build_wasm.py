"""Baut `gbrt` als WebAssembly (emscripten) fuer den Web-Playground.

**Status: experimentell.** Die native Runtime nutzt raylib (raylib-rs); ein
funktionierender Web-Build braucht die emscripten-Toolchain (`emcc`) UND den
Rust-Target `wasm32-unknown-emscripten`. Beides ist in dieser Umgebung nicht
installiert -> das Skript kompiliert dann nur die `.gbc` und gibt eine
Anleitung aus, statt zu scheitern.

Seit dem Front-End-Port (Lexer..Compiler in Rust, alle Stufen) kann gbrt die
`.gb`-QUELLE selbst kompilieren -> der Web-Build bettet jetzt die Quelle
(`program.gb`) ein und kompiliert im Browser (KEIN Pyodide noetig). Die
vorab-kompilierte `program.gbc` wird weiter erzeugt und als Fallback
eingebettet (`main.rs` liest `/program.gb` zuerst, dann `/program.gbc`).

Ablauf (mit vollstaendiger Toolchain):
  1. `<datei.gb>` -> `web/program.gb` (Quelle, kopiert) + `web/program.gbc`
     (Fallback, via Python-Compiler -- nur Build-Zeit, nicht im Browser noetig)
  2. `cargo build --target wasm32-unknown-emscripten --features graphics --release`
     mit emscripten-Linker-Flags (ASYNCIFY fuer den blockierenden VM-Loop,
     GLFW3 fuer raylib, `--embed-file program.gb@/program.gb` + `.gbc`).
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


def _first_glob(base: Path, pattern: str) -> Path | None:
    """Erstes Match von `base/pattern` (sortiert) oder None."""
    hits = sorted(base.glob(pattern))
    return hits[0] if hits else None


def setup_emscripten_env() -> bool:
    """Konfiguriert `os.environ` fuer den emscripten-Cross-Build (Windows),
    sodass `build_wasm.py` ohne manuelles Env-Setup laeuft. Idempotent und
    best-effort: setzt nur, was noch nicht gesetzt ist und was wirklich
    existiert. Gibt True zurueck, wenn ein emsdk gefunden+verdrahtet wurde.

    Hintergrund (Windows-Fallstricke, hart erkauft):
      - cc-rs/cmake-rs wrappen `.bat`-Compiler als `cmd /c emcc.bat` und
        schmuggeln das in CMAKE_C_FLAGS -> Build kaputt. Fix: CC/CXX/AR +
        Linker auf die `.exe`-Varianten (emcc.exe/em++.exe/emar.exe) zeigen.
      - bindgen nutzt die System-libclang, die emscriptens `stdarg.h` nicht
        findet -> BINDGEN_EXTRA_CLANG_ARGS mit clang-Builtin- + sysroot-Include.
      - emscripten-Cross-Compile braucht den Ninja-Generator + cmake/ninja auf
        PATH (hier aus den VS-BuildTools).
    """
    if sys.platform != "win32":
        return False  # Linux/macOS: `source emsdk_env.sh` genuegt i.d.R.
    emsdk = Path(os.environ.get("EMSDK", Path.home() / "emsdk"))
    em = emsdk / "upstream" / "emscripten"
    emcc = em / "emcc.exe"
    if not emcc.exists():
        return False

    def setdefault(key: str, val: str):
        if not os.environ.get(key) and val:
            os.environ[key] = val

    setdefault("EMSDK", str(emsdk))
    py = _first_glob(emsdk / "python", "*/python.exe")
    if py:
        setdefault("EMSDK_PYTHON", str(py))
    node = _first_glob(emsdk / "node", "*/bin/node.exe")
    if node:
        setdefault("EMSDK_NODE", str(node))
    setdefault("CC_wasm32_unknown_emscripten", str(emcc))
    setdefault("CXX_wasm32_unknown_emscripten", str(em / "em++.exe"))
    setdefault("AR_wasm32_unknown_emscripten", str(em / "emar.exe"))
    setdefault("CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_LINKER", str(emcc))
    setdefault("CMAKE_GENERATOR", "Ninja")
    # bindgen: clang-Builtin-Header (stdarg.h) + emscripten-Sysroot.
    clang_inc = _first_glob(emsdk / "upstream" / "lib" / "clang", "*/include")
    sysroot_inc = emsdk / "upstream" / "emscripten" / "cache" / "sysroot" / "include"
    if clang_inc and not os.environ.get("BINDGEN_EXTRA_CLANG_ARGS"):
        os.environ["BINDGEN_EXTRA_CLANG_ARGS"] = (
            f'-isystem "{clang_inc.as_posix()}" '
            f'-isystem "{sysroot_inc.as_posix()}"')
    # PATH: emsdk + emscripten + cmake/ninja vorne anstellen.
    extra = [str(emsdk), str(em)]
    cmake_dir = _find_cmake_dir()
    if cmake_dir:
        extra.append(str(cmake_dir))
    ninja_dir = _find_ninja_dir()
    if ninja_dir:
        extra.append(str(ninja_dir))
    os.environ["PATH"] = os.pathsep.join(extra + [os.environ.get("PATH", "")])
    return True


def _find_cmake_dir() -> Path | None:
    if shutil.which("cmake"):
        return None  # schon auf PATH
    for base in (r"C:\Program Files (x86)\Microsoft Visual Studio",
                 r"C:\Program Files\Microsoft Visual Studio"):
        hit = _first_glob(Path(base),
                          "*/BuildTools/Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin/cmake.exe")
        if hit:
            return hit.parent
    return None


def _find_ninja_dir() -> Path | None:
    if shutil.which("ninja"):
        return None
    for base in (r"C:\Program Files (x86)\Microsoft Visual Studio",
                 r"C:\Program Files\Microsoft Visual Studio"):
        hit = _first_glob(Path(base),
                          "*/BuildTools/Common7/IDE/CommonExtensions/Microsoft/CMake/Ninja/ninja.exe")
        if hit:
            return hit.parent
    return None

# emscripten-Linker-Flags (an raylibs Web-Beispiele angelehnt).
# ASYNCIFY laesst den blockierenden VM-/Render-Loop im Browser laufen, ohne die
# Engine auf emscripten_set_main_loop umzubauen (langsamer, aber funktioniert).
def emcc_flags(out_dir: str | Path) -> list:
    """emscripten-Linker-Flags. Die `--embed-file`-Quellpfade sind ABSOLUT
    (forward-slash), weil rustc den Linker nicht zwingend im `out_dir`-CWD
    aufruft -> relative Pfade wuerden vom file_packager nicht gefunden.
    (Der Repo-/web-Pfad enthaelt keine Leerzeichen -> EMCC_CFLAGS-safe.)"""
    out = Path(out_dir).resolve()
    gb = (out / "program.gb").as_posix()
    return [
        "-s", "USE_GLFW=3",
        "-s", "ASYNCIFY",
        "-s", "ALLOW_MEMORY_GROWTH=1",
        "-s", "ASSERTIONS=1",
        "-s", "EXPORTED_RUNTIME_METHODS=['callMain','FS','print']",
        # Quelle einbetten -> gbrt kompiliert sie im Browser selbst (Front-End-
        # Port, kein Pyodide). Der frühere Python-.gbc-Fallback entfällt (Stufe B:
        # Python-Compiler/serialize entfernt).
        "--embed-file", f"{gb}@/program.gb",
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


def _print_manual(info: dict, out_dir: str | Path = WEB) -> None:
    print("\n--- WASM-Build uebersprungen (Toolchain unvollstaendig) ---")
    print(f"  emcc (emscripten):           {'OK' if info['emcc'] else 'FEHLT'}")
    print(f"  cargo:                       {'OK' if info['cargo'] else 'FEHLT'}")
    print(f"  rust-target {TARGET}: {'OK' if info['wasm_target'] else 'FEHLT'}")
    print("\nSo vervollstaendigen:")
    print("  1. emscripten installieren (https://emscripten.org), `emcc` in PATH.")
    print(f"  2. rustup target add {TARGET}")
    print("  3. Dieses Skript erneut ausfuehren.")
    print("\nQuelle wurde bereits eingebettet. Manueller Build-Befehl:")
    print(f'  set EMCC_CFLAGS={" ".join(emcc_flags(out_dir))}')
    print(f"  cargo build --manifest-path {CRATE / 'Cargo.toml'} \\")
    print(f"    --target {TARGET} --features graphics --release")
    print("  -> target/wasm32-unknown-emscripten/release/gbrt.{js,wasm} nach web/ kopieren")


def copy_source(gb_path: str | Path, out_dir: str | Path = WEB) -> Path:
    """Kopiert die `.gb`-Quelle nach `<out_dir>/program.gb` -- gbrt kompiliert
    sie im Browser selbst (Front-End-Port). Kein Python im Browser noetig."""
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    dst = out_dir / "program.gb"
    dst.write_text(Path(gb_path).read_text(encoding="utf-8"), encoding="utf-8")
    return dst


def build(gb_path: str | Path, out_dir: str | Path = WEB) -> int:
    src = copy_source(gb_path, out_dir)
    print(f"Quelle eingebettet: {src}")
    # Windows: emscripten-Toolchain automatisch verdrahten (PATH + CC/CXX/AR/
    # Linker/bindgen-Includes), damit der Build ohne manuelles Env-Setup laeuft.
    if setup_emscripten_env():
        print("emscripten-Env aus emsdk verdrahtet.")
    info = check_toolchain()
    if not all(info.values()):
        _print_manual(info, out_dir)
        return 0
    env = dict(os.environ)
    # emscripten-Flags (absolute Embed-Pfade -> CWD-unabhaengig).
    env["EMCC_CFLAGS"] = " ".join(emcc_flags(out_dir))
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
