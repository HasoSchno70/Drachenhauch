"""LSP-Server fuer GameBasic (stdio, JSON-RPC 2.0).

Die Sprach-Intelligenz liegt in `features.py`; hier nur Protokoll:
Dokument-Store, Methoden-Dispatch, Position-/URI-Verwaltung. `LspServer.handle`
ist transport-unabhaengig (testbar); `serve()` haengt es an stdin/stdout.

Unterstuetzte Methoden: initialize/initialized/shutdown/exit,
textDocument/didOpen|didChange|didClose, completion, hover, definition,
references, documentSymbol. Voll-Sync (TextDocumentSyncKind.Full).
"""
from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from urllib.parse import unquote, urlparse

from . import features as F

SERVER_NAME = "gamebasic-lsp"
SERVER_VERSION = "1.0"


def uri_to_path(uri: str) -> str | None:
    p = urlparse(uri)
    if p.scheme != "file":
        return None
    path = unquote(p.path)
    # Windows: "/C:/..." -> "C:/..."
    if os.name == "nt" and len(path) >= 3 and path[0] == "/" and path[2] == ":":
        path = path[1:]
    return path


def _base_of(uri: str):
    path = uri_to_path(uri)
    if not path:
        return None
    return Path(path).parent


def _loc(uri: str, item: dict) -> dict:
    """features-Position {line,character,end_character} -> LSP Location."""
    end_char = item.get("end_character", item["character"])
    return {
        "uri": uri,
        "range": {
            "start": {"line": item["line"], "character": item["character"]},
            "end": {"line": item["line"], "character": end_char},
        },
    }


class LspServer:
    """Transport-unabhaengiger Kern. `send` schreibt eine ausgehende Message."""

    def __init__(self, send):
        self.send = send
        self.docs: dict[str, str] = {}
        self.shutdown_requested = False

    # -------------------------------------------------- Eingang
    def handle(self, msg: dict) -> None:
        method = msg.get("method")
        msg_id = msg.get("id")
        if method is None:
            return                                  # Antwort auf Server-Request: ignorieren
        try:
            handler = getattr(self, "_m_" + method.replace("/", "_").replace("$", ""), None)
            if handler is None:
                if msg_id is not None:
                    self._respond(msg_id, None)
                return
            result = handler(msg.get("params") or {})
            if msg_id is not None:
                self._respond(msg_id, result)
        except Exception as exc:  # noqa: BLE001 -- Server soll nicht crashen
            if msg_id is not None:
                self._error(msg_id, -32603, f"{type(exc).__name__}: {exc}")

    def _respond(self, msg_id, result) -> None:
        self.send({"jsonrpc": "2.0", "id": msg_id, "result": result})

    def _error(self, msg_id, code, message) -> None:
        self.send({"jsonrpc": "2.0", "id": msg_id,
                   "error": {"code": code, "message": message}})

    def _notify(self, method, params) -> None:
        self.send({"jsonrpc": "2.0", "method": method, "params": params})

    # -------------------------------------------------- Lifecycle
    def _m_initialize(self, _params):
        return {
            "capabilities": {
                "textDocumentSync": 1,              # Full
                "completionProvider": {"triggerCharacters": ["."]},
                "hoverProvider": True,
                "definitionProvider": True,
                "referencesProvider": True,
                "documentSymbolProvider": True,
            },
            "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION},
        }

    def _m_initialized(self, _params):
        return None

    def _m_shutdown(self, _params):
        self.shutdown_requested = True
        return None

    def _m_exit(self, _params):
        self.shutdown_requested = True
        return None

    # -------------------------------------------------- Dokumente
    def _m_textDocument_didOpen(self, params):
        doc = params["textDocument"]
        uri = doc["uri"]
        self.docs[uri] = doc.get("text", "")
        self._publish_diagnostics(uri)

    def _m_textDocument_didChange(self, params):
        uri = params["textDocument"]["uri"]
        changes = params.get("contentChanges") or []
        if changes:                                 # Full-Sync: letzte gewinnt
            self.docs[uri] = changes[-1]["text"]
        self._publish_diagnostics(uri)

    def _m_textDocument_didClose(self, params):
        uri = params["textDocument"]["uri"]
        self.docs.pop(uri, None)
        self._notify("textDocument/publishDiagnostics",
                     {"uri": uri, "diagnostics": []})

    def _publish_diagnostics(self, uri: str) -> None:
        text = self.docs.get(uri, "")
        diags = F.diagnostics(text, _base_of(uri))
        self._notify("textDocument/publishDiagnostics",
                     {"uri": uri, "diagnostics": diags})

    # -------------------------------------------------- Sprach-Features
    def _pos(self, params):
        uri = params["textDocument"]["uri"]
        pos = params["position"]
        return uri, self.docs.get(uri, ""), pos["line"], pos["character"]

    def _m_textDocument_completion(self, params):
        uri, text, line, char = self._pos(params)
        return F.completions(text, line, char)

    def _m_textDocument_hover(self, params):
        uri, text, line, char = self._pos(params)
        return F.hover(text, line, char)

    def _m_textDocument_definition(self, params):
        uri, text, line, char = self._pos(params)
        d = F.definition(text, line, char)
        return _loc(uri, d) if d else None

    def _m_textDocument_references(self, params):
        uri, text, line, char = self._pos(params)
        return [_loc(uri, r) for r in F.references(text, line, char)]

    def _m_textDocument_documentSymbol(self, params):
        uri = params["textDocument"]["uri"]
        return F.document_symbols(self.docs.get(uri, ""))


# ------------------------------------------------------ stdio-Transport

def _read_message(stream) -> dict | None:
    """Liest eine Content-Length-gerahmte JSON-RPC-Message von `stream` (binary)."""
    headers = {}
    while True:
        line = stream.readline()
        if not line:
            return None
        line = line.decode("ascii", "replace").strip()
        if line == "":
            break
        if ":" in line:
            k, v = line.split(":", 1)
            headers[k.strip().lower()] = v.strip()
    length = int(headers.get("content-length", 0))
    if length <= 0:
        return None
    body = stream.read(length)
    return json.loads(body.decode("utf-8"))


def _write_message(stream, msg: dict) -> None:
    data = json.dumps(msg).encode("utf-8")
    stream.write(f"Content-Length: {len(data)}\r\n\r\n".encode("ascii"))
    stream.write(data)
    stream.flush()


def serve() -> int:
    stdin = sys.stdin.buffer
    stdout = sys.stdout.buffer
    server = LspServer(lambda m: _write_message(stdout, m))
    while True:
        try:
            msg = _read_message(stdin)
        except Exception:                            # noqa: BLE001
            continue
        if msg is None:
            break
        server.handle(msg)
        if server.shutdown_requested and msg.get("method") == "exit":
            break
    return 0
