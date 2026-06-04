"""Einstiegspunkt: `py -m gamebasic.lsp` startet den LSP-Server (stdio)."""
import sys

from .server import serve

if __name__ == "__main__":
    sys.exit(serve())
