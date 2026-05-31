"""Verpacke ein GameBasic-Spiel zu einer eigenstaendigen .exe.

Benutzung:
    .venv\\Scripts\\python.exe gb-package.py <pfad/zum/spiel.gb> [--onefile] [--windowed]

Alternativ ueber den Wrapper:
    gb-package examples\\10_pong.gb --windowed --onefile

Was passiert:
1) Es wird ein Launcher-Skript fuer dieses Spiel generiert.
2) PyInstaller bundelt: Python-Runtime + pygame + gamebasic-Paket
   (inkl. nativer VM .pyd) + die .gb-Datei + ggf. assets/-Ordner.
3) Output landet unter dist/<name>/<name>.exe (oder dist/<name>.exe bei --onefile).

Voraussetzung: ein gebautes vm_native.cp312-win_amd64.pyd (gb-build.cmd)
und alle Laufzeitabhaengigkeiten im .venv (pygame, customtkinter optional).
"""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import textwrap
from pathlib import Path


_LAUNCHER_TEMPLATE = '''\
"""Auto-generierter GameBasic-Launcher fuer "{game_name}"."""
import os
import sys
from pathlib import Path

if getattr(sys, "frozen", False):
    BASE = Path(sys._MEIPASS)
else:
    BASE = Path(__file__).resolve().parent

# Damit relative LoadImage("assets/...")-Pfade funktionieren.
os.chdir(BASE)

GAME_FILE = BASE / "{game_filename}"


def _show_error(msg: str):
    """Fehler dem User zeigen - in Konsole und (falls moeglich) als Messagebox."""
    try:
        sys.stderr.write(f"GameBasic-Fehler: {{msg}}\\n")
    except Exception:
        pass
    try:
        import tkinter as tk
        from tkinter import messagebox
        root = tk.Tk()
        root.withdraw()
        messagebox.showerror("GameBasic - Fehler", msg)
        root.destroy()
    except Exception:
        # Konsolen-Fallback
        try:
            input("Druecke Enter zum Beenden ...")
        except Exception:
            pass


def main() -> int:
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler
    from gamebasic.errors import GameBasicError

    # Native VM bevorzugt; faellt auf Python-VM zurueck wenn .pyd fehlt.
    try:
        from gamebasic.vm_native import VM
    except ImportError:
        from gamebasic.vm import VM

    try:
        source = GAME_FILE.read_text(encoding="utf-8")
        from gamebasic.preprocess import process as _preprocess
        source, _origins = _preprocess(source, GAME_FILE.parent,
                                       file_label=GAME_FILE.name)
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        module = Compiler().compile(ast)
        VM().run(module)
    except GameBasicError as e:
        _show_error(str(e))
        return 2
    except Exception as e:  # pragma: no cover
        _show_error(f"Unerwarteter Fehler: {{type(e).__name__}}: {{e}}")
        return 3
    return 0


if __name__ == "__main__":
    sys.exit(main())
'''


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Packe ein GameBasic-Spiel als standalone .exe (PyInstaller).",
    )
    parser.add_argument("game", help="Pfad zur .gb-Datei")
    parser.add_argument(
        "--name",
        help="Name der ausgegebenen .exe (Default: aus Dateiname)",
    )
    parser.add_argument(
        "--onefile", action="store_true",
        help="Eine einzige .exe (langsamer Start, leichter zu verteilen)",
    )
    parser.add_argument(
        "--windowed", action="store_true",
        help="Kein Konsolenfenster (fuer Release; sonst Konsole zum Debuggen)",
    )
    parser.add_argument(
        "--icon",
        help="Pfad zu einer .ico-Datei",
    )
    parser.add_argument(
        "--clean", action="store_true",
        help="Loescht alte Build-Artefakte vorher",
    )
    args = parser.parse_args()

    project_root = Path(__file__).resolve().parent
    gb_path = Path(args.game).resolve()
    if not gb_path.exists():
        print(f"FEHLER: Datei nicht gefunden: {gb_path}")
        return 1
    if gb_path.suffix.lower() != ".gb":
        print(f"WARNUNG: Datei hat keine .gb-Endung: {gb_path}")

    name = args.name or gb_path.stem

    # Vorbereitung
    if args.clean:
        for sub in ("build", "dist"):
            p = project_root / sub
            if p.exists():
                shutil.rmtree(p)
        spec = project_root / f"{name}.spec"
        if spec.exists():
            spec.unlink()

    # Launcher generieren
    launcher_dir = project_root / "build" / "_launchers"
    launcher_dir.mkdir(parents=True, exist_ok=True)
    launcher_path = launcher_dir / f"_launch_{name}.py"
    launcher_path.write_text(
        _LAUNCHER_TEMPLATE.format(
            game_name=name,
            game_filename=gb_path.name,
        ),
        encoding="utf-8",
    )

    # PyInstaller-Argumente
    sep = ";" if os.name == "nt" else ":"
    py = sys.executable

    py_args: list[str] = [
        py, "-m", "PyInstaller",
        "--name", name,
        "--noconfirm",
        "--paths", str(project_root),
        # Spiel-Quelle ins Bundle-Wurzelverzeichnis
        "--add-data", f"{gb_path}{sep}.",
        # Komplettes gamebasic-Paket einbinden (inkl. Submodule + .pyd)
        "--collect-submodules", "gamebasic",
        "--collect-data", "gamebasic",
        "--collect-binaries", "gamebasic",
    ]
    if args.onefile:
        py_args.append("--onefile")
    if args.windowed:
        py_args.append("--windowed")
    if args.icon:
        py_args += ["--icon", str(Path(args.icon).resolve())]

    # Optional: Assets-Ordner mitnehmen, wenn neben der .gb-Datei
    assets = gb_path.parent / "assets"
    if assets.exists() and assets.is_dir():
        py_args += ["--add-data", f"{assets}{sep}assets"]

    py_args.append(str(launcher_path))

    print()
    print(f"Build-Ziel: {name}{'.exe' if args.onefile else ' (Ordner)'}")
    print(f"Spiel:      {gb_path}")
    if assets.exists():
        print(f"Assets:     {assets}")
    print()

    try:
        subprocess.run(py_args, check=True, cwd=str(project_root))
    except subprocess.CalledProcessError as exc:
        print(f"PyInstaller schlug fehl (Exit {exc.returncode}).")
        return exc.returncode

    if args.onefile:
        out = project_root / "dist" / f"{name}.exe"
    else:
        out = project_root / "dist" / name / f"{name}.exe"

    print()
    print("=" * 60)
    if out.exists():
        size_mb = out.stat().st_size / (1024 * 1024)
        print(f"FERTIG: {out}")
        print(f"Groesse: {size_mb:.1f} MB")
        if not args.onefile:
            folder_size = sum(f.stat().st_size for f in out.parent.rglob("*") if f.is_file())
            print(f"Gesamt-Ordner: {folder_size / (1024 * 1024):.1f} MB ({out.parent})")
    else:
        print(f"WARNUNG: Erwartete Datei nicht gefunden: {out}")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    sys.exit(main())
