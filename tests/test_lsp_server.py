"""Tests fuer den LSP-Server-Kern (transport-unabhaengig via LspServer.handle)
plus ein End-to-End-Test ueber echtes stdio-Framing als Subprozess."""
import io
import json
import subprocess
import sys
import time

from gamebasic.lsp.server import LspServer, _read_message, _write_message, uri_to_path


def _server():
    sent = []
    srv = LspServer(sent.append)
    return srv, sent


def _wait_for_diagnostics(sent, prev_count, timeout=5.0):
    """Diagnostics laufen seit dem Fix in Task 28 in einem Hintergrund-Thread
    (_DiagWorker) -- didOpen/didChange kehren sofort zurueck, die
    publishDiagnostics-Notification landet asynchron in `sent`. Statt eines
    festen `sleep()` (langsam + flaky) pollt dies kurz, bis mindestens EINE
    neue Nachricht seit `prev_count` ankam."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        pub = [m for m in sent if m.get("method") == "textDocument/publishDiagnostics"]
        if len(pub) > prev_count:
            return pub
        time.sleep(0.01)
    raise AssertionError("Timeout: keine publishDiagnostics-Notification angekommen")


def _req(srv, sent, method, params, msg_id):
    srv.handle({"jsonrpc": "2.0", "id": msg_id, "method": method, "params": params})
    # letzte Response mit passender id
    for m in reversed(sent):
        if m.get("id") == msg_id:
            return m
    return None


def _notif(srv, method, params):
    srv.handle({"jsonrpc": "2.0", "method": method, "params": params})


SRC = "FUNCTION add(a AS INTEGER, b AS INTEGER) AS INTEGER\n    RETURN a + b\nEND FUNCTION\nDIM r AS INTEGER\nr = add(1, 2)\n"
URI = "file:///tmp/test.gb"


def _open(srv, text=SRC, uri=URI):
    _notif(srv, "textDocument/didOpen",
           {"textDocument": {"uri": uri, "languageId": "gamebasic",
                             "version": 1, "text": text}})


def test_initialize_capabilities():
    srv, sent = _server()
    resp = _req(srv, sent, "initialize", {}, 1)
    caps = resp["result"]["capabilities"]
    assert caps["hoverProvider"] is True
    assert caps["definitionProvider"] is True
    assert caps["documentSymbolProvider"] is True
    assert caps["completionProvider"]["triggerCharacters"] == ["."]


def test_didopen_publishes_diagnostics():
    srv, sent = _server()
    _open(srv, "DIM x AS\n")     # Fehler
    pub = _wait_for_diagnostics(sent, 0)
    assert len(pub[-1]["params"]["diagnostics"]) == 1


def test_didopen_clean_no_diagnostics():
    srv, sent = _server()
    _open(srv)
    pub = _wait_for_diagnostics(sent, 0)
    assert pub[-1]["params"]["diagnostics"] == []


def test_didchange_updates_and_rediagnoses():
    srv, sent = _server()
    _open(srv)
    pub = _wait_for_diagnostics(sent, 0)
    prev = len(pub)
    _notif(srv, "textDocument/didChange",
           {"textDocument": {"uri": URI, "version": 2},
            "contentChanges": [{"text": "DIM x AS\n"}]})
    pub = _wait_for_diagnostics(sent, prev)
    assert len(pub[-1]["params"]["diagnostics"]) == 1


def test_definition_returns_location():
    srv, sent = _server()
    _open(srv)
    # "add" im Aufruf (Zeile 5 -> idx 4, "r = add(1, 2)", char 4)
    resp = _req(srv, sent, "textDocument/definition",
                {"textDocument": {"uri": URI},
                 "position": {"line": 4, "character": 5}}, 2)
    loc = resp["result"]
    assert loc["uri"] == URI
    assert loc["range"]["start"]["line"] == 0     # FUNCTION add in Zeile 1


def test_hover_returns_markdown():
    srv, sent = _server()
    _open(srv)
    resp = _req(srv, sent, "textDocument/hover",
                {"textDocument": {"uri": URI},
                 "position": {"line": 4, "character": 5}}, 3)
    assert "add" in resp["result"]["contents"]["value"]


def test_completion_list():
    srv, sent = _server()
    _open(srv)
    resp = _req(srv, sent, "textDocument/completion",
                {"textDocument": {"uri": URI},
                 "position": {"line": 4, "character": 0}}, 4)
    assert isinstance(resp["result"], list) and resp["result"]


def test_references_locations():
    srv, sent = _server()
    _open(srv)
    resp = _req(srv, sent, "textDocument/references",
                {"textDocument": {"uri": URI},
                 "position": {"line": 0, "character": 9},
                 "context": {"includeDeclaration": True}}, 5)
    lines = sorted(l["range"]["start"]["line"] for l in resp["result"])
    assert 0 in lines and 4 in lines


def test_document_symbols():
    srv, sent = _server()
    _open(srv)
    resp = _req(srv, sent, "textDocument/documentSymbol", {"textDocument": {"uri": URI}}, 6)
    names = [s["name"] for s in resp["result"]]
    assert "add" in names


def test_unknown_method_responds_null():
    srv, sent = _server()
    resp = _req(srv, sent, "textDocument/foobar", {}, 7)
    assert resp["result"] is None


def test_shutdown_flag():
    srv, sent = _server()
    _req(srv, sent, "shutdown", {}, 8)
    assert srv.shutdown_requested is True


def test_framing_roundtrip():
    buf = io.BytesIO()
    _write_message(buf, {"jsonrpc": "2.0", "id": 1, "result": {"ok": True}})
    buf.seek(0)
    msg = _read_message(buf)
    assert msg["result"]["ok"] is True


def test_handle_ignores_non_dict_message():
    # Review-Fund: ein valider JSON-Top-Level-Wert, der kein Objekt ist (z.B.
    # ein JSON-RPC-Batch-Array), liess `msg.get(...)` mit einem ungefangenen
    # AttributeError abbrechen -- serve() ruft handle() ausserhalb jedes
    # try/except auf, das haette den kompletten Server-Prozess mitgerissen.
    srv, sent = _server()
    srv.handle([])
    srv.handle("not-a-dict")
    srv.handle(42)
    assert sent == []


def test_read_message_raises_instead_of_silent_eof_on_missing_content_length():
    # Review-Fund: ein fehlender Content-Length-Header lieferte bisher `None`
    # -- identisch zu echtem Stream-EOF. serve() behandelt jedes `None` als
    # "Verbindung zu Ende" (break) und beendet den ganzen Server. Jetzt wirft
    # das stattdessen, damit serve()s bestehendes `except Exception: continue`
    # nur DIESE eine kaputte Nachricht ueberspringt statt die Session zu
    # beenden.
    buf = io.BytesIO(b"X-Custom-Header: 5\r\n\r\nhello")
    try:
        _read_message(buf)
        assert False, "sollte ValueError werfen"
    except ValueError:
        pass


def test_read_message_returns_none_on_real_eof():
    # Echtes EOF (keine Bytes mehr) muss weiterhin None liefern, NICHT werfen
    # -- das ist der legitime "Verbindung geschlossen"-Fall.
    buf = io.BytesIO(b"")
    assert _read_message(buf) is None


def test_uri_to_path():
    p = uri_to_path("file:///tmp/foo%20bar.gb")
    assert p.endswith("foo bar.gb")


def test_end_to_end_subprocess():
    """Echter Server-Prozess: initialize -> didOpen -> definition ueber stdio."""
    proc = subprocess.Popen(
        [sys.executable, "-m", "gamebasic.lsp"],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def send(obj):
        data = json.dumps(obj).encode("utf-8")
        proc.stdin.write(f"Content-Length: {len(data)}\r\n\r\n".encode("ascii"))
        proc.stdin.write(data)
        proc.stdin.flush()

    def recv():
        return _read_message(proc.stdout)

    try:
        send({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}})
        init = recv()
        assert init["result"]["capabilities"]["hoverProvider"] is True
        send({"jsonrpc": "2.0", "method": "textDocument/didOpen",
              "params": {"textDocument": {"uri": URI, "languageId": "gamebasic",
                                          "version": 1, "text": SRC}}})
        # Diagnostics-Notification (clean -> leer)
        diag = recv()
        assert diag["method"] == "textDocument/publishDiagnostics"
        send({"jsonrpc": "2.0", "id": 2, "method": "textDocument/definition",
              "params": {"textDocument": {"uri": URI},
                         "position": {"line": 4, "character": 5}}})
        d = recv()
        assert d["result"]["range"]["start"]["line"] == 0
        send({"jsonrpc": "2.0", "id": 3, "method": "shutdown", "params": {}})
        recv()
        send({"jsonrpc": "2.0", "method": "exit", "params": {}})
    finally:
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
