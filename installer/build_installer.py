#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Baut den GameBasic-Windows-Installer in einem Rutsch.

Schritte:
  1. gbrt-Runtime sicherstellen (rust/build_runtime.py, falls gbrt.exe fehlt).
  2. App-Icon installer/GameBasic.ico aus gamebasic/assets/logo.png erzeugen.
  3. PyInstaller: gbrun.py + gamebasic-Paket -> dist/GameBasic/ (onedir, ohne Python).
  4. Inno Setup (ISCC): dist/GameBasic + gbrt.exe -> installer/output/GameBasic-Setup-<version>.exe.

Aufruf (im .venv):
  .venv\\Scripts\\python.exe installer\\build_installer.py
Optionen:
  --no-installer   nur PyInstaller (kein Inno-Schritt)
  --rebuild-gbrt   gbrt vorher neu bauen
"""
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INST = ROOT / "installer"
PY = Path(sys.executable)
GBRT = ROOT / "rust" / "gb_runtime" / "target" / "release" / "gbrt.exe"


def log(msg):
    print(f"\n=== {msg} ===", flush=True)


def version():
    sys.path.insert(0, str(ROOT))
    from gamebasic import __version__
    return __version__


def ensure_gbrt(rebuild):
    if rebuild or not GBRT.exists():
        log("gbrt-Runtime bauen")
        subprocess.run([str(PY), str(ROOT / "rust" / "build_runtime.py")],
                       cwd=ROOT, check=True)
    if not GBRT.exists():
        sys.exit("FEHLER: gbrt.exe wurde nicht gebaut.")
    print("gbrt:", GBRT)


def make_icon():
    log("App-Icon erzeugen")
    try:
        from PIL import Image
    except ImportError:
        print("Pillow fehlt -> ohne Icon."); return
    src = ROOT / "gamebasic" / "assets" / "logo.png"
    if not src.exists():
        print("logo.png fehlt -> ohne Icon."); return
    im = Image.open(src).convert("RGBA")
    # Auf Quadrat bringen (transparent gepolstert), dann Multi-Size-ICO.
    s = max(im.size)
    canvas = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    canvas.paste(im, ((s - im.width) // 2, (s - im.height) // 2))
    ico = INST / "GameBasic.ico"
    canvas.save(ico, sizes=[(16, 16), (32, 32), (48, 48), (64, 64),
                            (128, 128), (256, 256)])
    print("Icon:", ico)


def run_pyinstaller():
    log("PyInstaller (eingefrorene IDE)")
    dist = ROOT / "dist"
    if (dist / "GameBasic").exists():
        shutil.rmtree(dist / "GameBasic")
    subprocess.run(
        [str(PY), "-m", "PyInstaller", str(INST / "GameBasic.spec"),
         "--noconfirm", "--distpath", str(dist),
         "--workpath", str(ROOT / "build" / "pyi")],
        cwd=ROOT, check=True)
    exe = dist / "GameBasic" / "GameBasic.exe"
    if not exe.exists():
        sys.exit("FEHLER: PyInstaller-Ausgabe fehlt.")
    print("IDE:", exe)


def find_iscc():
    for p in (os.environ.get("ISCC"),
              r"C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
              r"C:\Program Files\Inno Setup 6\ISCC.exe"):
        if p and Path(p).exists():
            return p
    found = shutil.which("ISCC")
    return found


def run_inno(ver):
    iscc = find_iscc()
    if not iscc:
        print("\nInno Setup (ISCC.exe) nicht gefunden. dist/GameBasic ist fertig.")
        print("Installer manuell bauen: ISCC.exe /DAppVersion=%s installer\\GameBasic.iss" % ver)
        return
    log("Inno Setup (Installer verpacken)")
    subprocess.run([iscc, f"/DAppVersion={ver}", str(INST / "GameBasic.iss")],
                   cwd=INST, check=True)
    out = INST / "output" / f"GameBasic-Setup-{ver}.exe"
    print("\nFERTIG ->", out if out.exists() else (INST / "output"))


def main():
    rebuild = "--rebuild-gbrt" in sys.argv
    no_inst = "--no-installer" in sys.argv
    ver = version()
    print(f"GameBasic {ver} -- Installer-Build")
    ensure_gbrt(rebuild)
    make_icon()
    run_pyinstaller()
    if not no_inst:
        run_inno(ver)


if __name__ == "__main__":
    main()
