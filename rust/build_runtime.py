"""Baut die native Rust-Runtime `gb_runtime` mit Grafik (raylib).

raylib kompiliert seine C-Quellen via cmake und braucht libclang fuer die
FFI-Bindings (bindgen). Dieses Skript stellt die noetige Umgebung her:
  - cmake aus den VS Build Tools auf den PATH
  - LIBCLANG_PATH auf die LLVM-Installation
  - cl.exe wird von der `cc`-Crate automatisch via vswhere/Registry gefunden

Aufruf:
    .venv\\Scripts\\python.exe rust\\build_runtime.py            # release, mit Grafik
    .venv\\Scripts\\python.exe rust\\build_runtime.py --no-graphics
    .venv\\Scripts\\python.exe rust\\build_runtime.py --debug

Ohne Grafik (`--no-graphics`) baut der pure VM-Kern ganz ohne C-Toolchain.
"""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CRATE = HERE / "gb_runtime"


def _find_first(globs: list[str]) -> str | None:
    for pattern in globs:
        base, _, rest = pattern.partition("*")
        # einfache Suche: feste bekannte Pfade zuerst
        p = Path(pattern)
        if "*" not in pattern and p.exists():
            return str(p)
    return None


def _find_cmake_bin() -> str | None:
    candidates = [
        r"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin",
        r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin",
        r"C:\Program Files\CMake\bin",
    ]
    for c in candidates:
        if (Path(c) / "cmake.exe").exists():
            return c
    # PATH?
    from shutil import which
    w = which("cmake")
    if w:
        return str(Path(w).parent)
    return None


def _find_libclang_dir() -> str | None:
    for c in [r"C:\Program Files\LLVM\bin", r"C:\Program Files (x86)\LLVM\bin"]:
        if (Path(c) / "libclang.dll").exists():
            return c
    return None


def main() -> int:
    args = sys.argv[1:]
    graphics = "--no-graphics" not in args
    release = "--debug" not in args

    env = dict(os.environ)

    if graphics:
        cmake = _find_cmake_bin()
        if not cmake:
            print("FEHLER: cmake nicht gefunden (VS Build Tools / CMake installieren)")
            return 1
        env["PATH"] = cmake + os.pathsep + env.get("PATH", "")
        libclang = _find_libclang_dir()
        if not libclang:
            print("FEHLER: libclang.dll nicht gefunden. LLVM installieren "
                  "(z.B. `winget install LLVM.LLVM`).")
            return 1
        env["LIBCLANG_PATH"] = libclang
        print(f"cmake:    {cmake}")
        print(f"libclang: {libclang}")

    # `--test` fuehrt die Rust-Unit-Tests aus (mit derselben Toolchain-Env),
    # sonst wird das Binary gebaut.
    if "--test" in args:
        cmd = ["cargo", "test"]
    else:
        cmd = ["cargo", "build"]
        if release:
            cmd.append("--release")

    # Feature-Auswahl: graphics (raylib) + Daten-Tier (db/net/http) sind der
    # Standard-Dev-Build. `--no-data` laesst db/net/http weg, `--hardware`
    # nimmt serial/usb/bt/wifi dazu (Phase 4).
    feats = []
    if graphics:
        feats.append("graphics")
    if "--no-data" not in args:
        feats += ["db", "net", "http"]
    if "--hardware" in args:
        feats += ["serial", "usb", "bt", "wifi"]
    if feats:
        cmd += ["--features", " ".join(feats)]

    print("->", " ".join(cmd))
    return subprocess.run(cmd, cwd=str(CRATE), env=env).returncode


if __name__ == "__main__":
    sys.exit(main())
