"""GameBasic - Eintrittspunkt fuer den Interpreter.

Verwendung:
    python gbrun.py <datei.gb>
    python gbrun.py --tokens <datei.gb>   # nur Tokens ausgeben (Debug)
    python gbrun.py --ast <datei.gb>      # nur AST ausgeben (Debug)
    python gbrun.py --native <datei.gb>   # nativ via gbrt-Runtime ausfuehren
    python gbrun.py --export <datei.gb> [ordner]  # standalone .exe buendeln
"""
import os
import sys
from pathlib import Path

from gamebasic.lexer import Lexer
from gamebasic.parser import Parser
from gamebasic.errors import GameBasicError


def _print_help_and_examples():
    from gamebasic import __version__
    print(f"GameBasic v{__version__}")
    print()
    print("Verwendung:  python gbrun.py [--tokens|--ast] <datei.gb>")
    print("             gb.cmd                <datei.gb>     (Windows-Launcher mit .venv)")
    print()
    project_root = Path(__file__).resolve().parent
    examples_dir = project_root / "examples"
    if examples_dir.is_dir():
        files = sorted(p.name for p in examples_dir.glob("*.gb") if not p.name.startswith("_"))
        if files:
            print(f"Beispiele in {examples_dir.name}/:")
            for name in files:
                print(f"  examples/{name}")
            print()
    print("Tipp:  In PyCharm das Beispiel als 'Parameters' der Run-Konfiguration eintragen,")
    print("       z.B. 'examples/10_pong.gb'.")


def _launch_editor(project_root, initial_file=None):
    try:
        from gamebasic.editor_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def _launch_sprite_editor(project_root, initial_file=None):
    try:
        from gamebasic.spriteeditor_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def _launch_particle_editor(project_root, initial_file=None):
    try:
        from gamebasic.particleeditor_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def _launch_sfx_editor(project_root, initial_file=None):
    try:
        from gamebasic.sfxeditor_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def _launch_tracker_editor(project_root, initial_file=None):
    try:
        from gamebasic.trackereditor_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def _launch_tilemap_editor(project_root, initial_file=None):
    try:
        from gamebasic.tilemapeditor_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def main(argv):
    args = argv[1:]
    mode = "run"

    # --- Editor explizit per Flag ---
    if args and args[0] in ("--editor", "-e"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_editor(Path(__file__).resolve().parent, initial)
        if rc is None:
            print("Editor benoetigt 'PySide6'. Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6")
            return 3
        return rc

    # --- Sprite-Editor explizit per Flag ---
    if args and args[0] in ("--sprites", "--sprite-editor", "-S"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_sprite_editor(Path(__file__).resolve().parent, initial)
        if rc is None:
            print("Sprite-Editor benoetigt 'PySide6' und 'Pillow'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6 Pillow")
            return 3
        return rc

    # --- Partikel-Editor explizit per Flag ---
    if args and args[0] in ("--particles", "--particle-editor", "-P"):
        args = args[1:]
        rc = _launch_particle_editor(Path(__file__).resolve().parent, None)
        if rc is None:
            print("Partikel-Editor benoetigt 'PySide6' und 'numpy'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6 numpy")
            return 3
        return rc

    # --- SFX-Generator explizit per Flag ---
    if args and args[0] in ("--sfx", "--sound", "--sfx-editor"):
        args = args[1:]
        rc = _launch_sfx_editor(Path(__file__).resolve().parent, None)
        if rc is None:
            print("SFX-Generator benoetigt 'PySide6' und 'numpy'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6 numpy")
            return 3
        return rc

    # --- Tracker (Musik-Editor) explizit per Flag ---
    if args and args[0] in ("--tracker", "--music"):
        args = args[1:]
        rc = _launch_tracker_editor(Path(__file__).resolve().parent, None)
        if rc is None:
            print("Tracker benoetigt 'PySide6' und 'numpy'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6 numpy")
            return 3
        return rc

    # --- Tilemap-/Level-Editor explizit per Flag ---
    if args and args[0] in ("--tilemap", "--level", "--map-editor"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_tilemap_editor(Path(__file__).resolve().parent, initial)
        if rc is None:
            print("Tilemap-Editor benoetigt 'PySide6'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6")
            return 3
        return rc

    if args and args[0] in ("--tokens", "--ast", "--native", "--export"):
        mode = args.pop(0)[2:]

    # --- Export: standalone .exe buendeln (Schritt 7) ---
    # Vor dem chdir/Editor-Pfad behandeln; nimmt optional ein Ausgabeverzeichnis.
    if mode == "export":
        if not args:
            print("Verwendung: python gbrun.py --export <datei.gb> [ausgabe-ordner]")
            return 1
        return _run_export(Path(args[0]), args[1] if len(args) > 1 else None)

    # --- Ohne Argumente: Editor starten, sonst Hilfe ---
    if not args:
        rc = _launch_editor(Path(__file__).resolve().parent)
        if rc is not None:
            return rc
        _print_help_and_examples()
        return 0

    path = Path(args[0])
    if not path.exists():
        print(f"Datei nicht gefunden: {path}")
        print("Tipp: 'python gbrun.py' ohne Argument zeigt verfuegbare Beispiele.")
        return 1

    # Resolve VOR chdir, damit relative Argumente weiterhin stimmen.
    abs_path = path.resolve()
    source = abs_path.read_text(encoding="utf-8")
    # Damit LoadImage("assets/...") aus dem Programm relativ zur .gb-Datei
    # funktioniert, ins Verzeichnis der Quelldatei wechseln.
    os.chdir(abs_path.parent)

    # --- Ausfuehren: immer ueber die native Runtime (gbrt) ---
    # Stufe B: der Tree-Walker ist entfernt; sowohl der Default-Run als auch
    # `--native` laufen ueber `gbrt run` (preprocess+lex+parse+compile+VM in Rust).
    if mode in ("run", "native"):
        return _run_native(abs_path, path)

    # --- Debug: --tokens / --ast (Python-Lexer/-Parser, fuer Dev/Parity) ---
    from gamebasic.preprocess import process as _preprocess
    source, origins = _preprocess(source, abs_path.parent, file_label=path.name)

    try:
        tokens = Lexer(source).tokenize()
        if mode == "tokens":
            for tok in tokens:
                print(tok)
            return 0
        ast = Parser(tokens).parse()
        if mode == "ast":
            _print_ast(ast)
            return 0
        return 0
    except GameBasicError as e:
        # Origin der Fehler-Zeile (falls in einer eingebundenen Datei)
        origin = None
        if 1 <= e.line < len(origins) and origins[e.line] is not None:
            origin = origins[e.line]
        print(f"Fehler in {path.name}:")
        if origin and origin[0] != path.name:
            print(f"  {type(e).__name__}: {e.message}")
            print(f"  -> {origin[0]}:{origin[1]} (via IMPORT in {path.name})")
        else:
            print(f"  {e}")
        return 2


def _find_gbrt():
    """Sucht das gebaute `gbrt`-Binary (Release bevorzugt, sonst Debug)."""
    base = Path(__file__).resolve().parent / "rust" / "gb_runtime" / "target"
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


def _run_native(abs_path, path):
    """Fuehrt die `.gb`-Datei mit `gbrt run` aus -- gbrts EIGENES Rust-Frontend
    (preprocess+lex+parse+compile+VM), KEIN Python-Compiler mehr.

    So laufen auch gbrt-only-Builtins (die der Python-Compiler nicht kennt). gbrt
    wechselt selbst ins Verzeichnis der Datei (relative Assets) und nutzt den
    Dateinamen fuer Fehler-Labels (`datei.gb:Zeile`). stdout/stderr + ein etwaiges
    Grafik-Fenster werden durchgereicht. Rueckgabe = Exit-Code von gbrt
    (bzw. 3 wenn gbrt fehlt)."""
    import subprocess

    gbrt = _find_gbrt()
    if gbrt is None:
        print("Native Runtime 'gbrt' nicht gefunden. Einmalig bauen mit:")
        print("  .venv\\Scripts\\python.exe rust\\build_runtime.py")
        print("(ohne Grafik: rust\\build_runtime.py --no-graphics)")
        return 3
    try:
        result = subprocess.run([str(gbrt), "run", str(abs_path)])
    except OSError as e:
        print(f"Konnte gbrt nicht starten: {e}")
        return 3
    return result.returncode


def _run_export(src, out_dir):
    """Buendelt `src` (.gb) zu einer eigenstaendigen Exe via `gbrt --export` --
    gbrts EIGENER Selbst-Export (kompiliert die Quelle mit dem Rust-Frontend,
    haengt den Payload an eine Kopie der Exe, kopiert `assets/`). KEIN Python-
    Compiler -> auch gbrt-only-Builtins exportieren.

    Rueckgabe: 0 ok, 1 Quelle fehlt, 3 Runtime fehlt, sonst gbrt-Exit-Code."""
    import subprocess

    src = Path(src)
    if not src.exists():
        print(f"Datei nicht gefunden: {src}")
        return 1
    gbrt = _find_gbrt()
    if gbrt is None:
        print("Native Runtime 'gbrt' nicht gefunden. Einmalig bauen mit:")
        print("  .venv\\Scripts\\python.exe rust\\build_runtime.py")
        return 3
    args = [str(gbrt), "--export", str(src.resolve())]
    if out_dir:
        args.append(str(out_dir))
    try:
        result = subprocess.run(args)   # gbrt druckt "Exportiert: <pfad>"
    except OSError as e:
        print(f"Konnte gbrt nicht starten: {e}")
        return 3
    if result.returncode == 0:
        print("  -> laeuft ohne Python. Assets-Konvention: 'assets/' wird mitkopiert.")
    return result.returncode


def _print_ast(node, indent=0):
    pad = "  " * indent
    if isinstance(node, list):
        for item in node:
            _print_ast(item, indent)
        return
    cls = type(node).__name__
    if hasattr(node, "__dataclass_fields__"):
        print(f"{pad}{cls}")
        for fname in node.__dataclass_fields__:
            val = getattr(node, fname)
            if isinstance(val, list) or hasattr(val, "__dataclass_fields__"):
                print(f"{pad}  .{fname}:")
                _print_ast(val, indent + 2)
            else:
                print(f"{pad}  .{fname} = {val!r}")
    else:
        print(f"{pad}{node!r}")


if __name__ == "__main__":
    sys.exit(main(sys.argv))
