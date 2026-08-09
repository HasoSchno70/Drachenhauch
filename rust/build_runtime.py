"""Baut die native Rust-Runtime `drachenhauch_runtime` mit Grafik (raylib).

raylib kompiliert seine C-Quellen via cmake und braucht libclang fuer die
FFI-Bindings (bindgen). Dieses Skript stellt die noetige Umgebung her:
  - Windows: cmake aus den VS Build Tools auf den PATH, LIBCLANG_PATH auf
    die LLVM-Installation (Windows hat keine Paketmanager-Konvention dafuer,
    daher das manuelle Pfad-Raten). cl.exe wird von der `cc`-Crate
    automatisch via vswhere/Registry gefunden.
  - Linux/macOS: cmake/clang kommen ueblicherweise ueber den System-
    Paketmanager (apt/brew/...) und sind dann bereits im PATH bzw. von
    bindgen/clang-sys selbst per llvm-config/pkg-config auffindbar -- hier
    wird nur geprueft, ob sie da sind, nicht nach Pfaden gesucht.

Aufruf:
    .venv\\Scripts\\python.exe rust\\build_runtime.py            # release, mit Grafik
    .venv\\Scripts\\python.exe rust\\build_runtime.py --no-graphics
    .venv\\Scripts\\python.exe rust\\build_runtime.py --debug
    .venv\\Scripts\\python.exe rust\\build_runtime.py --hardware  # + serial/usb/bt/wifi

Ohne Grafik (`--no-graphics`) baut der pure VM-Kern ganz ohne C-Toolchain.

**Cross-Platform-Status:** Entwicklung/CI laufen bisher nur unter Windows.
Der Linux/macOS-Zweig unten ist nach bestem Wissen geschrieben, aber NICHT
auf echter Linux-/macOS-Hardware verifiziert -- bei Problemen bitte ein
Issue mit der genauen Fehlermeldung aufmachen. `wifi`-Feature ist ausserdem
weiterhin nur unter Windows funktionsfaehig (netsh-basiert).
"""
from __future__ import annotations

import os
import platform
import subprocess
import sys
from pathlib import Path
from shutil import which

HERE = Path(__file__).resolve().parent
CRATE = HERE / "drachenhauch_runtime"


def _find_cmake_bin_windows() -> str | None:
    candidates = [
        r"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin",
        r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin",
        r"C:\Program Files\CMake\bin",
    ]
    for c in candidates:
        if (Path(c) / "cmake.exe").exists():
            return c
    w = which("cmake")
    if w:
        return str(Path(w).parent)
    return None


def _find_libclang_dir_windows() -> str | None:
    for c in [r"C:\Program Files\LLVM\bin", r"C:\Program Files (x86)\LLVM\bin"]:
        if (Path(c) / "libclang.dll").exists():
            return c
    return None


def _setup_windows_toolchain(env: dict[str, str]) -> int:
    cmake = _find_cmake_bin_windows()
    if not cmake:
        print("FEHLER: cmake nicht gefunden (VS Build Tools / CMake installieren)")
        return 1
    env["PATH"] = cmake + os.pathsep + env.get("PATH", "")
    libclang = _find_libclang_dir_windows()
    if not libclang:
        print("FEHLER: libclang.dll nicht gefunden. LLVM installieren "
              "(z.B. `winget install LLVM.LLVM`).")
        return 1
    env["LIBCLANG_PATH"] = libclang
    print(f"cmake:    {cmake}")
    print(f"libclang: {libclang}")
    return 0


def _setup_unix_toolchain(env: dict[str, str]) -> int:
    """Linux/macOS: keine feste Verzeichnisstruktur wie unter Windows --
    cmake/clang werden ueber PATH erwartet (Standard nach Paketmanager-
    Installation). LIBCLANG_PATH wird bewusst NICHT gesetzt, wenn schon
    vorhanden (User-Override) oder clang gefunden wurde -- bindgen/clang-sys
    findet libclang in dem Fall meist selbst (llvm-config/pkg-config)."""
    is_macos = platform.system() == "Darwin"
    if which("cmake") is None:
        hint = "brew install cmake llvm" if is_macos else \
            "sudo apt install cmake clang libclang-dev   # Debian/Ubuntu; Paketname je Distro anders"
        print(f"FEHLER: cmake nicht gefunden. Installieren, z.B.:\n  {hint}")
        return 1
    if which("clang") is None and "LIBCLANG_PATH" not in env:
        hint = ("brew install llvm   # ggf. zusaetzlich LIBCLANG_PATH=\"$(brew --prefix llvm)/lib\" setzen"
                if is_macos else
                "sudo apt install libclang-dev   # Debian/Ubuntu; Paketname je Distro anders")
        print(f"WARNUNG: clang/libclang nicht gefunden -- bindgen (fuer raylib-FFI) "
              f"koennte scheitern. Installieren, z.B.:\n  {hint}")
    return 0


def main() -> int:
    args = sys.argv[1:]
    graphics = "--no-graphics" not in args
    release = "--debug" not in args
    system = platform.system()

    env = dict(os.environ)

    if graphics:
        setup_rc = _setup_windows_toolchain(env) if system == "Windows" else _setup_unix_toolchain(env)
        if setup_rc != 0:
            return setup_rc

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
        # `graphics` zieht raylib (Fenster/Input) + Kira (Audio, src/audio.rs).
        # `dialogs` (rfd, native OS-Dateidialoge) ist davon GETRENNT, weil es
        # den WASM-Build blockiert -- auf dem Desktop gehoert es aber dazu.
        feats += ["graphics", "dialogs"]
    if "--no-data" not in args:
        feats += ["db", "net", "http"]
    if "--hardware" in args:
        feats += ["serial", "usb", "bt", "wifi"]
        if system != "Windows" and "wifi" in feats:
            print("Hinweis: 'wifi' ist aktuell nur unter Windows funktionsfaehig "
                  "(netsh-basiert) -- WIFI_*-Builtins werfen auf diesem System "
                  "zur Laufzeit einen klaren Fehler, der Build selbst geht aber durch.")
    if feats:
        cmd += ["--features", " ".join(feats)]

    print("->", " ".join(cmd))
    return subprocess.run(cmd, cwd=str(CRATE), env=env).returncode


if __name__ == "__main__":
    sys.exit(main())
