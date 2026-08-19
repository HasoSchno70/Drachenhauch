"""Erlaubt 'py -m drachenhauch <datei.dh>' von ueberall.

Stufe B: Der Tree-Walker ist entfernt -- Ausfuehrung laeuft ueber die native
Runtime `dhrt` (dhrt chdirt selbst ins Datei-Verzeichnis, daher funktionieren
relative Asset-Pfade jetzt auch hier). `--tokens`/`--ast` nutzen den behaltenen
die native Runtime `dhrt` -- Python parst hier nichts mehr.
"""
import sys
from pathlib import Path



def main(argv=None):
    argv = argv if argv is not None else sys.argv
    args = argv[1:]
    mode = "run"
    if args and args[0] in ("--tokens", "--ast"):
        mode = args.pop(0)[2:]

    if not args:
        print("Verwendung: py -m drachenhauch [--tokens|--ast] <datei.dh>")
        return 1

    path = Path(args[0])
    if not path.exists():
        print(f"Datei nicht gefunden: {path}")
        return 1

    # --- Ausfuehren: ueber die native Runtime (dhrt) ---
    if mode == "run":
        from dhrun import _run_native  # type: ignore
        return _run_native(path.resolve(), path)

    # --- Debug: --tokens / --ast reicht dhrt durch ---
    # Frueher lief das ueber den Python-Lexer/-Parser. `dhrt` kann beides
    # selbst (main.rs), und es ist ohnehin das Frontend, das wirklich zaehlt --
    # ein zweites, das leicht abweicht, half beim Suchen eher nicht.
    from dhrun import _find_dhrt  # type: ignore
    dhrt = _find_dhrt()
    if dhrt is None:
        print("Native Runtime 'dhrt' nicht gefunden. Einmalig bauen mit:")
        print("  .venv\\Scripts\\python.exe rust\\build_runtime.py")
        return 3
    import subprocess
    return subprocess.call([str(dhrt), "--" + mode, str(path)])


if __name__ == "__main__":
    sys.exit(main())
