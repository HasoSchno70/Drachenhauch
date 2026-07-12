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


def _project_root():
    """Basisordner des Editors (Beispiele/Showcase + Default-Arbeitsverzeichnis).

    - Eingefroren (per Installer): ein BESCHREIBBARER, fester Ort.
      Windows: ``%PUBLIC%\\Documents\\GameBasic`` -- dort legt der Inno-
      Installer ``examples/`` (inkl. ``screenshots/``) beim Installieren ab.
      macOS/Linux: ``~/Documents/GameBasic`` -- .dmg/AppImage/Tarball haben
      keinen Installations-Skript-Schritt wie Inno Setup, daher werden die
      Beispiele dort stattdessen beim allerersten Start aus dem Bundle
      kopiert (siehe `_seed_examples_if_missing`). Das PyInstaller-Bundle
      selbst ist schreibgeschuetzt (bzw. auf macOS ein signiertes .app-Paket)
      und enthaelt daher nie die tatsaechlich genutzte Beispiele-Kopie.
    - Entwicklung: das Repo-Verzeichnis (neben gbrun.py).
    """
    if getattr(sys, "frozen", False):
        if os.name == "nt":
            public = os.environ.get("PUBLIC") or r"C:\Users\Public"
            return Path(public) / "Documents" / "GameBasic"
        root = Path.home() / "Documents" / "GameBasic"
        _seed_examples_if_missing(root)
        return root
    return Path(__file__).resolve().parent


def _seed_examples_if_missing(root: Path) -> None:
    """macOS/Linux-Pendant zu Inno Setups Install-Zeit-Kopie: einmalig beim
    ersten Start examples/ aus dem eingefrorenen Bundle (_MEIPASS) in den
    beschreibbaren project_root kopieren, falls noch nicht vorhanden. Kein
    harter Fehler, wenn das (aus welchem Grund auch immer) scheitert --
    der Editor startet trotzdem, nur ohne Beispiele-Menue."""
    if (root / "examples").exists():
        return
    meipass = getattr(sys, "_MEIPASS", None)
    if not meipass:
        return
    bundled = Path(meipass) / "examples"
    if not bundled.is_dir():
        return
    try:
        import shutil
        root.mkdir(parents=True, exist_ok=True)
        shutil.copytree(bundled, root / "examples", dirs_exist_ok=True)
    except OSError:
        pass


def _print_help_and_examples():
    from gamebasic import __version__
    print(f"GameBasic v{__version__}")
    print()
    print("Verwendung:  python gbrun.py [--tokens|--ast] <datei.gb>")
    print("             gb.cmd                <datei.gb>     (Windows-Launcher mit .venv)")
    print()
    project_root = _project_root()
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


def _launch_audio_studio(project_root, initial_file=None, tab="tracker"):
    # Vereintes Audio Studio (Tracker + SFX in Tabs). gbsfx/gbtracker oeffnen
    # dasselbe Fenster auf dem passenden Tab.
    try:
        from gamebasic.audiostudio_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file, tab=tab)


def _launch_sfx_editor(project_root, initial_file=None):
    return _launch_audio_studio(project_root, initial_file, tab="sfx")


def _launch_tracker_editor(project_root, initial_file=None):
    return _launch_audio_studio(project_root, initial_file, tab="tracker")


def _launch_tilemap_editor(project_root, initial_file=None):
    try:
        from gamebasic.tilemapeditor_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def _launch_form_designer(project_root, initial_file=None):
    try:
        from gamebasic.formdesigner_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def _launch_anim_editor(project_root, initial_file=None):
    try:
        from gamebasic.animeditor_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def _launch_score_editor(project_root, initial_file=None):
    try:
        from gamebasic.scoreeditor_qt import launch
    except SystemExit:
        return None
    except ImportError:
        return None
    return launch(project_root, initial_file)


def _launch_chooser(project_root):
    """Start-Dialog ohne Argumente: Code-Editor oder WYSIWYG-Form-Designer.
    Liefert None, wenn PySide6 fehlt (Aufrufer zeigt dann Text-Hilfe)."""
    try:
        from PySide6.QtWidgets import (
            QApplication, QDialog, QVBoxLayout, QHBoxLayout, QPushButton, QLabel)
        from PySide6.QtCore import Qt
    except ImportError:
        return None
    app = QApplication.instance() or QApplication([])
    try:
        from gamebasic.editor_qt.theme import global_qss
        app.setStyleSheet(global_qss())
    except Exception:
        pass

    dlg = QDialog()
    dlg.setWindowTitle("GameBasic")
    lay = QVBoxLayout(dlg)
    title = QLabel("Was möchtest du öffnen?")
    title.setAlignment(Qt.AlignmentFlag.AlignCenter)
    f = title.font(); f.setPointSize(f.pointSize() + 3); f.setBold(True); title.setFont(f)
    lay.addWidget(title)

    choice = {"v": None}

    def pick(v):
        choice["v"] = v
        dlg.accept()

    row = QHBoxLayout()
    b_edit = QPushButton("📝  Code-Editor")
    b_form = QPushButton("🧩  Form-Designer (WYSIWYG)")
    b_anim = QPushButton("🎬  Animation-Editor (FSM)")
    for b in (b_edit, b_form, b_anim):
        b.setMinimumHeight(64)
        b.setMinimumWidth(220)
        row.addWidget(b)
    b_edit.clicked.connect(lambda: pick("editor"))
    b_form.clicked.connect(lambda: pick("form"))
    b_anim.clicked.connect(lambda: pick("anim"))
    lay.addLayout(row)

    dlg.exec()
    root = Path(project_root)
    if choice["v"] == "editor":
        return _launch_editor(root)
    if choice["v"] == "form":
        return _launch_form_designer(root)
    if choice["v"] == "anim":
        return _launch_anim_editor(root)
    return 0   # abgebrochen


def main(argv):
    args = argv[1:]
    mode = "run"

    # --- Editor explizit per Flag ---
    if args and args[0] in ("--editor", "-e"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_editor(_project_root(), initial)
        if rc is None:
            print("Editor benoetigt 'PySide6'. Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6")
            return 3
        return rc

    # --- Sprite-Editor explizit per Flag ---
    if args and args[0] in ("--sprites", "--sprite-editor", "-S"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_sprite_editor(_project_root(), initial)
        if rc is None:
            print("Sprite-Editor benoetigt 'PySide6' und 'Pillow'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6 Pillow")
            return 3
        return rc

    # --- Partikel-Editor explizit per Flag ---
    if args and args[0] in ("--particles", "--particle-editor", "-P"):
        args = args[1:]
        rc = _launch_particle_editor(_project_root(), None)
        if rc is None:
            print("Partikel-Editor benoetigt 'PySide6' und 'numpy'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6 numpy")
            return 3
        return rc

    # --- Audio Studio (vereint Tracker + SFX) explizit per Flag ---
    if args and args[0] in ("--audio", "--studio", "--audio-studio"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_audio_studio(_project_root(), initial, tab="tracker")
        if rc is None:
            print("Audio Studio benoetigt 'PySide6' und 'numpy'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6 numpy")
            return 3
        return rc

    # --- SFX-Generator per Flag (oeffnet das Studio auf dem SFX-Tab) ---
    if args and args[0] in ("--sfx", "--sound", "--sfx-editor"):
        args = args[1:]
        rc = _launch_sfx_editor(_project_root(), None)
        if rc is None:
            print("SFX-Generator benoetigt 'PySide6' und 'numpy'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6 numpy")
            return 3
        return rc

    # --- Tracker per Flag (oeffnet das Studio auf dem Tracker-Tab) ---
    if args and args[0] in ("--tracker", "--music"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_tracker_editor(_project_root(), initial)
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
        rc = _launch_tilemap_editor(_project_root(), initial)
        if rc is None:
            print("Tilemap-Editor benoetigt 'PySide6'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6")
            return 3
        return rc

    # --- Form-Designer (WYSIWYG, Xojo-Stil) explizit per Flag ---
    if args and args[0] in ("--form", "--form-designer", "--designer"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_form_designer(_project_root(), initial)
        if rc is None:
            print("Form-Designer benoetigt 'PySide6'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6")
            return 3
        return rc

    # --- Animations-FSM-Editor (gbanim, Unity-Mecanim-Stil) explizit per Flag ---
    if args and args[0] in ("--anim", "--anim-editor", "--fsm"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_anim_editor(_project_root(), initial)
        if rc is None:
            print("Anim-FSM-Editor benoetigt 'PySide6'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6")
            return 3
        return rc

    # --- Notenblatt-Editor (Notensatz-Stil, gbscore) explizit per Flag ---
    if args and args[0] in ("--score", "--notenblatt", "--score-editor"):
        args = args[1:]
        initial = Path(args[0]) if args else None
        rc = _launch_score_editor(_project_root(), initial)
        if rc is None:
            print("Notenblatt-Editor benoetigt 'PySide6', 'numpy' und "
                  "optional 'sounddevice'.")
            print("Im .venv installieren:")
            print("  .venv\\Scripts\\python.exe -m pip install PySide6 numpy")
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

    # --- Ohne Argumente ---
    if not args:
        # Installierte App: direkt den Code-Editor oeffnen (kein Auswahlfenster).
        if getattr(sys, "frozen", False):
            rc = _launch_editor(_project_root(), None)
            if rc is not None:
                return rc
        # Entwicklung: Auswahl Code-Editor / Form-Designer, sonst Hilfe.
        rc = _launch_chooser(_project_root())
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
    """Sucht das `gbrt`-Binary. Reihenfolge:
    1. Eingefrorene Installation (PyInstaller): neben der Exe bzw. im Bundle
       (_MEIPASS) -- so findet die installierte GameBasic-App ihre Runtime.
    2. Dev-Baum: rust/gb_runtime/target/{release,debug}/gbrt[.exe]."""
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    cands = []
    if getattr(sys, "frozen", False):
        cands.append(Path(sys.executable).resolve().parent / exe)
        meipass = getattr(sys, "_MEIPASS", None)
        if meipass:
            cands.append(Path(meipass) / exe)
    base = _project_root() / "rust" / "gb_runtime" / "target"
    cands += [base / "release" / exe, base / "debug" / exe]
    for p in cands:
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
