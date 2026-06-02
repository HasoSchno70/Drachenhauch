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
from gamebasic.interpreter import Interpreter
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

    if args and args[0] in ("--tokens", "--ast", "--vm", "--bench", "--native", "--export"):
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

    # --- Native Runtime (gbrt): kompilieren -> .gbc -> ausfuehren ---
    if mode == "native":
        return _run_native(abs_path, path)

    # IMPORT-Preprocessor: textuelle Inklusion vor dem Lexen.
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
        if mode == "vm":
            from gamebasic.compiler import Compiler
            VMClass, kind = _resolve_vm()
            module = Compiler().compile(ast)
            VMClass().run(module)
            return 0
        if mode == "bench":
            return _bench(ast, path)
        Interpreter().run(ast)
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


def _resolve_vm():
    """Liefert (VMClass, kind). Bevorzugt nativ, faellt auf Python zurueck."""
    try:
        from gamebasic.vm_native import VM as NativeVM   # type: ignore
        return NativeVM, "native"
    except ImportError:
        from gamebasic.vm import VM as PyVM
        return PyVM, "python"


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
    """Kompiliert die `.gb`-Datei nach `.gbc` und fuehrt sie mit `gbrt` aus.

    Der Compiler laeuft weiter in Python (Lexer/Parser/Compiler bleiben dort);
    `gbrt` uebernimmt nur die Ausfuehrung. stdout/stderr und ein etwaiges
    Grafik-Fenster werden direkt durchgereicht. Rueckgabe = Exit-Code von gbrt
    (bzw. 2 bei Compile-Fehler, 3 wenn gbrt fehlt)."""
    import subprocess
    import tempfile
    from gamebasic.serialize import compile_file_to_gbc

    gbrt = _find_gbrt()
    if gbrt is None:
        print("Native Runtime 'gbrt' nicht gefunden. Einmalig bauen mit:")
        print("  .venv\\Scripts\\python.exe rust\\build_runtime.py")
        print("(ohne Grafik: rust\\build_runtime.py --no-graphics)")
        return 3

    # In eine temporaere .gbc kompilieren (neben der Quelle waere auch ok,
    # aber temp haelt das Projektverzeichnis sauber).
    fd, tmp = tempfile.mkstemp(suffix=".gbc")
    os.close(fd)
    try:
        try:
            compile_file_to_gbc(abs_path, tmp)
        except GameBasicError as e:
            print(f"Compile-Fehler in {path.name}:")
            print(f"  {e}")
            return 2
        # gbrt im Verzeichnis der Quelldatei ausfuehren (relative Assets),
        # stdout/stderr erben -> Live-Ausgabe + Fenster. Der Quell-Dateiname
        # wird als 2. Arg durchgereicht -> Laufzeitfehler zeigen datei.gb:Zeile.
        try:
            result = subprocess.run([str(gbrt), tmp, path.name],
                                    cwd=str(abs_path.parent))
        except OSError as e:
            print(f"Konnte gbrt nicht starten: {e}")
            return 3
        return result.returncode
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass


def _run_export(src, out_dir):
    """Buendelt `src` (.gb) zu einer eigenstaendigen Exe (gbrt + Bytecode + Assets).

    Rueckgabe: 0 ok, 1 Quelle fehlt, 2 Compile-Fehler, 3 Runtime fehlt."""
    src = Path(src)
    if not src.exists():
        print(f"Datei nicht gefunden: {src}")
        return 1
    gbrt = _find_gbrt()
    if gbrt is None:
        print("Native Runtime 'gbrt' nicht gefunden. Einmalig bauen mit:")
        print("  .venv\\Scripts\\python.exe rust\\build_runtime.py")
        return 3
    from gamebasic.export import export_standalone
    try:
        exe = export_standalone(src, gbrt, out_dir=out_dir)
    except GameBasicError as e:
        print(f"Compile-Fehler in {src.name}:")
        print(f"  {e}")
        return 2
    size_mb = exe.stat().st_size / (1024 * 1024)
    print(f"Standalone-Export erstellt: {exe}  ({size_mb:.1f} MB)")
    print(f"  -> laeuft ohne Python; Ordner '{exe.parent.name}' weitergeben.")
    print("  Assets-Konvention: Dateien unter 'assets/' werden mitkopiert.")
    return 0


def _bench(ast, path):
    """Vergleicht Tree-Walker, Python-VM und Native-VM auf demselben Programm."""
    import io
    import time
    import contextlib
    from gamebasic.compiler import Compiler
    from gamebasic.vm import VM as PyVM

    NativeVM = None
    try:
        from gamebasic.vm_native import VM as _NV  # type: ignore
        NativeVM = _NV
    except ImportError:
        pass

    print(f"=== Benchmark: {path.name} ===")

    # Tree-Walker
    buf_tw = io.StringIO()
    t0 = time.perf_counter()
    with contextlib.redirect_stdout(buf_tw):
        Interpreter().run(ast)
    tw_time = time.perf_counter() - t0
    tw_out = buf_tw.getvalue()

    # Compile
    t0 = time.perf_counter()
    module = Compiler().compile(ast)
    compile_time = time.perf_counter() - t0

    # Python-VM
    buf_py = io.StringIO()
    t0 = time.perf_counter()
    with contextlib.redirect_stdout(buf_py):
        PyVM().run(module)
    py_time = time.perf_counter() - t0
    py_out = buf_py.getvalue()

    # Native-VM
    nv_time = None
    nv_out = None
    if NativeVM is not None:
        buf_nv = io.StringIO()
        t0 = time.perf_counter()
        with contextlib.redirect_stdout(buf_nv):
            NativeVM().run(module)
        nv_time = time.perf_counter() - t0
        nv_out = buf_nv.getvalue()

    print(f"Tree-Walker:    {tw_time * 1000:9.2f} ms")
    print(f"Compile:        {compile_time * 1000:9.2f} ms")
    print(f"Python-VM:      {py_time * 1000:9.2f} ms        Speedup vs TW: {tw_time / py_time:5.2f}x")
    if nv_time is not None:
        print(f"Native-VM:      {nv_time * 1000:9.2f} ms        Speedup vs TW: {tw_time / nv_time:5.2f}x   (vs Py-VM: {py_time / nv_time:5.2f}x)")
    else:
        print("Native-VM:      nicht gebaut (run 'python setup.py build_ext --inplace')")

    outs = {"Tree-Walker": tw_out, "Python-VM": py_out}
    if nv_out is not None:
        outs["Native-VM"] = nv_out
    if len(set(outs.values())) == 1:
        print("Output:         ALLE IDENTISCH")
    else:
        print("Output:         UNTERSCHIEDLICH (!)")
        for label, out in outs.items():
            print(f"--- {label} ---")
            print(out)
    return 0


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
