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
import threading
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


class _DiagWorker:
    """Async-Diagnostik fuer EIN Dokument -- dasselbe Generation-Counter +
    Subprozess-Abbruch-Muster wie `editor_qt.error_check.LiveErrorChecker`,
    aber ohne Qt-Abhaengigkeit (der LSP-Server bleibt headless). Statt eines
    Qt-Signals ruft das Ergebnis einen einfachen Callback auf.

    Review-Fund: `LspServer._publish_diagnostics` rief `F.diagnostics(...)`
    bisher DIREKT aus `handle()` auf -- das laeuft (bei gebautem gbrt) durch
    `gbrt --check` als blockierenden Subprozess mit bis zu 15s Timeout.
    `serve()`s Lese-Loop ist strikt sequentiell: WAEHREND dieser Subprozess
    laeuft, kann der Server absolut nichts anderes verarbeiten -- kein Hover,
    keine Completion, keine neuere didChange. Bei normalem Tippen (jeder
    Tastendruck sendet ein volles didChange, Full-Sync) sammeln sich so
    Subprozess-Aufrufe seriell an; ein einzelner haengender/langsamer
    gbrt-Aufruf friert den GESAMTEN Server ein. Jetzt laeuft der eigentliche
    Check in einem Daemon-Thread; `check()` selbst kehrt sofort zurueck."""

    def __init__(self, on_result):
        self._gen = 0
        self._lock = threading.Lock()
        self._active_proc = None
        self._on_result = on_result

    def check(self, text: str, base_path) -> None:
        with self._lock:
            self._gen += 1
            gen = self._gen
            proc = self._active_proc
        # Einen noch laufenden Check-Subprozess abbrechen, nicht nur dessen
        # (dann veraltetes) Ergebnis verwerfen -- sonst koennten sich bei
        # schnellem Tippen mehrere `gbrt --check`-Prozesse gleichzeitig
        # ansammeln (identisches Muster zu LiveErrorChecker.check).
        if proc is not None:
            try:
                proc.terminate()
            except OSError:
                pass
        t = threading.Thread(target=self._run, args=(gen, text, base_path),
                             name="LspDiagnostics", daemon=True)
        t.start()

    def cancel(self) -> None:
        """Fuer didClose: laufenden Check abbrechen, OHNE einen neuen zu
        starten (das Dokument ist geschlossen, ein Ergebnis waere sinnlos)."""
        with self._lock:
            self._gen += 1
            proc = self._active_proc
        if proc is not None:
            try:
                proc.terminate()
            except OSError:
                pass

    def _set_active_proc(self, proc) -> None:
        with self._lock:
            self._active_proc = proc

    def _run(self, gen: int, text: str, base_path) -> None:
        diags = F.diagnostics(text, base_path, checker=self)
        with self._lock:
            if gen != self._gen:
                return                                 # ueberholt -- verwerfen
        self._on_result(diags)


class LspServer:
    """Transport-unabhaengiger Kern. `send` schreibt eine ausgehende Message."""

    def __init__(self, send):
        self.send = send
        self.docs: dict[str, str] = {}
        self.shutdown_requested = False
        # stdout-Schreiben ist nicht threadsicher -- Diagnostics laufen in
        # eigenen Hintergrund-Threads (_DiagWorker) und wuerden sonst mit dem
        # Haupt-Thread um denselben Stream konkurrieren koennen (ineinander
        # verschraenkte Bytes wuerden die Content-Length-Rahmung brechen).
        self._send_lock = threading.Lock()
        self._diag_workers: dict[str, _DiagWorker] = {}

    # -------------------------------------------------- Eingang
    def handle(self, msg) -> None:
        # Review-Fund: ein valides JSON-Top-Level-Wert, der KEIN Objekt ist
        # (z.B. ein JSON-RPC-Batch-Array `[]`), liess `msg.get(...)` unten mit
        # einem ungefangenen AttributeError abbrechen -- serve() ruft handle()
        # ausserhalb jedes try/except auf, das riss den kompletten Server-
        # Prozess mit. So etwas (bewusst nicht unterstuetzte Batch-Requests
        # o.ae.) wird jetzt einfach ignoriert statt zu crashen.
        if not isinstance(msg, dict):
            return
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
            # Immer nach stderr loggen (nicht nur bei Requests mit msg_id) --
            # sonst verschwindet z.B. ein Fehler in _publish_diagnostics
            # (ueber didOpen/didChange, beides Notifications ohne msg_id)
            # spurlos: der Client sieht weiterhin die Diagnostics der
            # VORHERIGEN Dokument-Version, ohne jeden Hinweis warum.
            print(f"[gamebasic-lsp] {method}: {type(exc).__name__}: {exc}", file=sys.stderr)
            if msg_id is not None:
                self._error(msg_id, -32603, f"{type(exc).__name__}: {exc}")

    def _respond(self, msg_id, result) -> None:
        with self._send_lock:
            self.send({"jsonrpc": "2.0", "id": msg_id, "result": result})

    def _error(self, msg_id, code, message) -> None:
        with self._send_lock:
            self.send({"jsonrpc": "2.0", "id": msg_id,
                       "error": {"code": code, "message": message}})

    def _notify(self, method, params) -> None:
        with self._send_lock:
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
        worker = self._diag_workers.pop(uri, None)
        if worker is not None:
            worker.cancel()
        self._notify("textDocument/publishDiagnostics",
                     {"uri": uri, "diagnostics": []})

    def _publish_diagnostics(self, uri: str) -> None:
        text = self.docs.get(uri, "")
        worker = self._diag_workers.get(uri)
        if worker is None:
            worker = _DiagWorker(lambda diags, uri=uri: self._on_diagnostics(uri, diags))
            self._diag_workers[uri] = worker
        worker.check(text, _base_of(uri))

    def _on_diagnostics(self, uri: str, diags: list[dict]) -> None:
        self._notify("textDocument/publishDiagnostics",
                     {"uri": uri, "diagnostics": diags})

    # -------------------------------------------------- Sprach-Features
    def _pos(self, params):
        uri = params["textDocument"]["uri"]
        pos = params["position"]
        return uri, self.docs.get(uri, ""), pos["line"], pos["character"]

    def _m_textDocument_completion(self, params):
        _, text, line, char = self._pos(params)
        return F.completions(text, line, char)

    def _m_textDocument_hover(self, params):
        _, text, line, char = self._pos(params)
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
    """Liest eine Content-Length-gerahmte JSON-RPC-Message von `stream` (binary).

    Liefert `None` NUR bei echtem EOF (Stream/Verbindung geschlossen, kein
    einziges Header-Byte mehr verfuegbar). Ein fehlender/ungueltiger
    Content-Length-Header wirft stattdessen eine `ValueError` -- Review-Fund:
    vorher lieferte auch DAS `None`, und `serve()` behandelte jedes `None`
    identisch zu einem echten Verbindungsende (`break` -> Server-Prozess
    beendet sich). Eine einzelne kaputte/unvollstaendige Nachricht liess so
    die komplette LSP-Session lautlos sterben, ohne jeden Fehlerhinweis.
    """
    headers = {}
    while True:
        line = stream.readline()
        if not line:
            return None                              # echtes EOF
        line = line.decode("ascii", "replace").strip()
        if line == "":
            break
        if ":" in line:
            k, v = line.split(":", 1)
            headers[k.strip().lower()] = v.strip()
    raw_length = headers.get("content-length")
    if raw_length is None:
        raise ValueError("LSP-Nachricht ohne Content-Length-Header")
    try:
        length = int(raw_length)
    except ValueError:
        raise ValueError(f"LSP-Nachricht mit ungueltigem Content-Length: {raw_length!r}") from None
    if length <= 0:
        raise ValueError(f"LSP-Nachricht mit Content-Length <= 0: {length}")
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
