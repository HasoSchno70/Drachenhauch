"""Erlaubt 'py -m drachenhauch <datei.gb>' von ueberall.

Stufe B: Der Tree-Walker ist entfernt -- Ausfuehrung laeuft ueber die native
Runtime `dhrt` (dhrt chdirt selbst ins Datei-Verzeichnis, daher funktionieren
relative Asset-Pfade jetzt auch hier). `--tokens`/`--ast` nutzen den behaltenen
Python-Lexer/-Parser (Dev/Parity).
"""
import sys
from pathlib import Path

from .lexer import Lexer
from .parser import Parser
from .errors import DrachenhauchError


def main(argv=None):
    argv = argv if argv is not None else sys.argv
    args = argv[1:]
    mode = "run"
    if args and args[0] in ("--tokens", "--ast"):
        mode = args.pop(0)[2:]

    if not args:
        print("Verwendung: py -m drachenhauch [--tokens|--ast] <datei.gb>")
        return 1

    path = Path(args[0])
    if not path.exists():
        print(f"Datei nicht gefunden: {path}")
        return 1

    # --- Ausfuehren: ueber die native Runtime (dhrt) ---
    if mode == "run":
        from dhrun import _run_native  # type: ignore
        return _run_native(path.resolve(), path)

    # --- Debug: --tokens / --ast ueber den Python-Lexer/-Parser ---
    source = path.read_text(encoding="utf-8")
    from .preprocess import process as _preprocess
    source, _origins = _preprocess(source, path.parent, file_label=path.name)
    try:
        tokens = Lexer(source).tokenize()
        if mode == "tokens":
            for tok in tokens:
                print(tok)
            return 0
        ast = Parser(tokens).parse()
        if mode == "ast":
            from dhrun import _print_ast  # type: ignore
            _print_ast(ast)
            return 0
        return 0
    except DrachenhauchError as e:
        print(f"Fehler in {path.name}:")
        print(f"  {e}")
        return 2


if __name__ == "__main__":
    sys.exit(main())
