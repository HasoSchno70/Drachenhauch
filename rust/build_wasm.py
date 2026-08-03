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
    """emscripten-Linker-Flags. Die Datei-Quellpfade sind ABSOLUT
    (forward-slash), weil rustc den Linker nicht zwingend im `out_dir`-CWD
    aufruft -> relative Pfade wuerden vom file_packager nicht gefunden.
    (Der Repo-/web-Pfad enthaelt keine Leerzeichen -> EMCC_CFLAGS-safe.)

    Assets (`assets/` neben der .gb) kommen als PRELOAD dazu, nicht als
    embed: emscripten legt daraus eine eigene `gbrt.data` an, die der Browser
    nebenher laedt. Eingebettet wuerden die paar Megabyte in die `.wasm`
    wandern und jeden Start ausbremsen.
    """
    out = Path(out_dir).resolve()
    gb = (out / "program.gb").as_posix()
    flags = [
        "-s", "USE_GLFW=3",
        "-s", "ASYNCIFY",
        "-s", "ALLOW_MEMORY_GROWTH=1",
        "-s", "ASSERTIONS=1",
        # emscripten gibt einem Programm 64 KB Stapel -- auf dem Desktop sind
        # es 8 MB. Das Parsen eines Tracker-Moduls (xmrs) legt eine grosse
        # Struktur an, bevor sie auf den Haufen wandert, und sprengte die 64 KB:
        # AUDIO_MUSIC_PLAY brach mit einem nackten abort() ab.
        "-s", "STACK_SIZE=4MB",
        # WebGL 2 (= OpenGL ES 3.0). Nur damit versteht der Browser GLSL ES
        # 3.00, das bis auf die Versionszeile unserem Desktop-GLSL 330
        # entspricht -- Beleuchtung, PBR, Skybox und Instancing brauchen es.
        "-s", "MIN_WEBGL_VERSION=2",
        "-s", "MAX_WEBGL_VERSION=2",
        # ... und die leeren Ersatz-Bibliotheken auffindbar machen (siehe dort).
        f"-L{leere_gl_ersatzbibliotheken().as_posix()}",
        # requestFullscreen gehoert dazu: raylibs ToggleFullscreen ruft es auf
        # dem Web-Pfad auf, und ohne den Export bricht das Programm ab
        # (SET_FULLSCREEN(TRUE) ist in Spielen/Demos die Regel).
        "-s", "EXPORTED_RUNTIME_METHODS=['callMain','FS','print','requestFullscreen']",
        # Quelle einbetten -> gbrt kompiliert sie im Browser selbst (Front-End-
        # Port, kein Pyodide). Der frühere Python-.gbc-Fallback entfällt (Stufe B:
        # Python-Compiler/serialize entfernt).
    ]
    # emscripten laesst --embed und --preload NICHT gemeinsam zu. Ohne Assets
    # bleibt es beim Einbetten (eine Datei weniger); mit Assets wird alles
    # vorgeladen, damit die paar Megabyte nicht in die .wasm wandern.
    assets = out / "assets"
    if assets.is_dir():
        flags += ["--preload-file", f"{gb}@/program.gb",
                  "--preload-file", f"{assets.as_posix()}@/assets"]
    else:
        flags += ["--embed-file", f"{gb}@/program.gb"]
    return flags


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


def copy_assets(gb_path: str | Path, out_dir: str | Path = WEB) -> int:
    """Uebernimmt ein `assets/`-Verzeichnis NEBEN der `.gb` nach `<out_dir>`.

    Die Laufzeit chdirt im Browser nach `/`, und dort liegt auch `program.gb`
    -- relative Pfade wie `LOADIMAGE("assets/x.png")` treffen damit genau das
    eingebettete Verzeichnis, ohne dass ein Programm etwas anders schreiben
    muss als auf dem Desktop.

    Liefert die Groesse in Bytes (0 = kein assets/ vorhanden).
    """
    quelle = Path(gb_path).resolve().parent / "assets"
    ziel = Path(out_dir) / "assets"
    if ziel.exists():
        shutil.rmtree(ziel)          # alte Assets nicht mitschleppen
    if not quelle.is_dir():
        return 0
    shutil.copytree(quelle, ziel)
    return sum(f.stat().st_size for f in ziel.rglob("*") if f.is_file())


def leere_gl_ersatzbibliotheken() -> Path:
    """Leere `libGLESv2.a`/`libGLdispatch.a` anlegen und ihren Ordner liefern.

    raylib-sys haengt fuer sein `opengl_es_30`-Feature bedingungslos
    `-lGLESv2 -lGLdispatch` an -- auch fuer emscripten, wo es diese
    Bibliotheken gar nicht gibt (der Browser liefert GL selbst, und emscripten
    linkt es ueber `-lGL-webgl2-getprocaddr`). Der Ziel-Filter fehlt dort
    schlicht; der Build bricht mit `unable to find library -lGLdispatch` ab.

    Statt auf einen Upstream-Fix zu warten, bekommt der Linker die Namen als
    LEERE Archive. Sie tragen nichts bei -- sie beenden nur die Suche. (Ein
    leeres ar-Archiv besteht aus genau seiner Kennung; kein Werkzeug noetig.)
    """
    ordner = CRATE / "target" / "wasm-gl-ersatz"
    ordner.mkdir(parents=True, exist_ok=True)
    for name in ("libGLESv2.a", "libGLdispatch.a"):
        ziel = ordner / name
        if not ziel.exists():
            ziel.write_bytes(b"!<arch>\n")
    return ordner


def erzwinge_relink(flags: str) -> bool:
    """Neu linken, wenn sich die emscripten-Flags geaendert haben.

    **Diese Falle kostet sonst Stunden:** cargo verfolgt `EMCC_CFLAGS` nicht.
    Aendert man nur ein Linker-Flag (etwa `STACK_SIZE`), sieht cargo unveraenderte
    Quellen, ueberspringt das Linken -- und die alte `.wasm` bleibt liegen. Der
    Build meldet Erfolg, das Flag ist aber nicht drin. Genau so sah es aus, als
    die Vergroesserung des Stapels "nicht half": sie war schlicht nie im Binary.

    Das Loeschen der Ausgabe ist der einfachste Zwang -- cargo linkt neu, sobald
    sie fehlt, kompiliert aber nichts unnoetig neu.
    """
    rel = CRATE / "target" / TARGET / "release"
    stempel = rel / "emcc_flags.txt"
    if stempel.exists() and stempel.read_text(encoding="utf-8") == flags:
        return False
    for muster in ("gbrt.js", "gbrt.wasm", "gbrt.data", "gbrt"):
        (rel / muster).unlink(missing_ok=True)
    rel.mkdir(parents=True, exist_ok=True)
    stempel.write_text(flags, encoding="utf-8")
    return True


def build(gb_path: str | Path, out_dir: str | Path = WEB) -> int:
    src = copy_source(gb_path, out_dir)
    print(f"Quelle eingebettet: {src}")
    groesse = copy_assets(gb_path, out_dir)
    if groesse:
        print(f"Assets uebernommen: {groesse / 1048576:.1f} MB "
              f"(werden als gbrt.data vorgeladen)")
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
    erzwinge_relink(env["EMCC_CFLAGS"])
    cmd = ["cargo", "build", "--manifest-path", str(CRATE / "Cargo.toml"),
           "--target", TARGET, "--features", "graphics", "--release"]
    print("Build:", " ".join(cmd))
    rc = subprocess.run(cmd, cwd=str(Path(out_dir)), env=env).returncode
    if rc != 0:
        print("cargo build fehlgeschlagen (siehe oben).")
        return rc
    rel = CRATE / "target" / TARGET / "release"
    # gbrt.data entsteht nur mit --preload-file (also wenn Assets dabei sind)
    # und liegt bei den deps, nicht neben der .wasm -- ohne sie fehlen dem
    # Programm im Browser alle Dateien.
    for name in ("gbrt.js", "gbrt.wasm", "gbrt.data"):
        src = rel / name
        if not src.exists():
            treffer = sorted(rel.glob(f"deps/{name}"))
            src = treffer[0] if treffer else src
        if src.exists():
            shutil.copy2(src, Path(out_dir) / name)
            print(f"kopiert: {Path(out_dir) / name} "
                  f"({src.stat().st_size / 1048576:.1f} MB)")
        elif name == "gbrt.data":
            # Kein Paket: das ist der Normalfall ohne assets/, kein Fehler.
            (Path(out_dir) / name).unlink(missing_ok=True)
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
