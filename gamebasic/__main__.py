"""Erlaubt 'py -m gamebasic <datei.gb>' von ueberall."""
import sys
from pathlib import Path

from .lexer import Lexer
from .parser import Parser
from .interpreter import Interpreter
from .errors import GameBasicError


def main(argv=None):
    argv = argv if argv is not None else sys.argv
    args = argv[1:]
    mode = "run"
    if args and args[0] in ("--tokens", "--ast"):
        mode = args.pop(0)[2:]

    if not args:
        print("Verwendung: py -m gamebasic [--tokens|--ast] <datei.gb>")
        return 1

    path = Path(args[0])
    if not path.exists():
        print(f"Datei nicht gefunden: {path}")
        return 1

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
            from gbrun import _print_ast  # type: ignore
            _print_ast(ast)
            return 0
        Interpreter().run(ast)
        return 0
    except GameBasicError as e:
        print(f"Fehler in {path.name}:")
        print(f"  {e}")
        return 2


if __name__ == "__main__":
    sys.exit(main())
