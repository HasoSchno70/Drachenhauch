"""Tests fuer das cloud-Modul (Cloud-Save + Leaderboard).

Golden-Tests gegen `dhrt` (Stufe B): laufen gegen einen lokalen Mock-Server
(ThreadingHTTPServer im pytest-Prozess, wie test_modules_html.py), der das
REST-Protokoll aus cloudserver/server.py nachbildet (nicht der echte Flask-
Server -- der wird separat in cloudserver/test_server.py getestet). Das
GB-Programm laeuft im dhrt-Subprozess und macht echte localhost-Requests.
"""
import contextlib
import json
import threading
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

from gamebasic.errors import GBRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


class _CloudMockHandler(BaseHTTPRequestHandler):
    """Bildet server.py's REST-Vertrag nach: /save/<id>, /leaderboard/<b>/submit,
    /leaderboard/<b>/top. Zustand (saves/scores) lebt pro Server-Instanz --
    _mock_cloud_server() erzeugt fuer jeden Test eine frische Handler-Klasse."""

    saves: dict = {}
    scores: dict = {}
    api_key: str | None = None

    def log_message(self, *a, **k):
        pass

    def _json(self, status, obj):
        body = json.dumps(obj).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _auth_ok(self):
        if self.api_key is None:
            return True
        if self.headers.get("X-Api-Key") != self.api_key:
            self._json(401, {"error": "unauthorized"})
            return False
        return True

    def do_GET(self):
        if not self._auth_ok():
            return
        parsed = urllib.parse.urlsplit(self.path)
        parts = [p for p in parsed.path.split("/") if p]
        if len(parts) == 2 and parts[0] == "save":
            data = self.saves.get(parts[1])
            if data is None:
                self._json(404, {"error": "not_found"})
            else:
                self._json(200, {"data": data, "updated_at": 0})
            return
        if len(parts) == 3 and parts[0] == "leaderboard" and parts[2] == "top":
            qs = urllib.parse.parse_qs(parsed.query)
            n = int(qs.get("n", ["10"])[0])
            order = qs.get("order", ["desc"])[0]
            items = sorted(self.scores.get(parts[1], {}).items(),
                            key=lambda kv: kv[1], reverse=(order == "desc"))[:n]
            self._json(200, {"entries": [{"name": k, "score": v} for k, v in items]})
            return
        self._json(404, {"error": "not_found"})

    def do_POST(self):
        if not self._auth_ok():
            return
        length = int(self.headers.get("Content-Length", 0))
        body = json.loads(self.rfile.read(length) or b"{}")
        parsed = urllib.parse.urlsplit(self.path)
        parts = [p for p in parsed.path.split("/") if p]
        if len(parts) == 2 and parts[0] == "save":
            self.saves[parts[1]] = body.get("data", "")
            self._json(200, {"ok": True})
            return
        if len(parts) == 3 and parts[0] == "leaderboard" and parts[2] == "submit":
            board = self.scores.setdefault(parts[1], {})
            name, score, best = body["name"], body["score"], body.get("best", "high")
            cur = board.get(name)
            better = cur is None or (score > cur if best == "high" else score < cur)
            if better:
                board[name] = score
            self._json(200, {"ok": True, "updated": better})
            return
        self._json(404, {"error": "not_found"})


@contextlib.contextmanager
def _mock_cloud_server(api_key: str | None = None):
    handler = type("Handler", (_CloudMockHandler,), {"saves": {}, "scores": {}, "api_key": api_key})
    server = ThreadingHTTPServer(("127.0.0.1", 0), handler)
    t = threading.Thread(target=server.serve_forever, daemon=True)
    t.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}"
    finally:
        server.shutdown()
        server.server_close()


# --- CLOUD_SAVE / CLOUD_LOAD -------------------------------------------------

def test_cloud_save_and_load_roundtrip(run_gb):
    with _mock_cloud_server() as base:
        out = _lines(run_gb(
            f'IMPORT "cloud"\nCLOUD_CONFIGURE("{base}", "")\n'
            'PRINT CLOUD_SAVE("spieler1", "{""gold"": 42}")\n'
            'PRINT CLOUD_LOAD("spieler1")\n'))
    assert out == ["TRUE", '{"gold": 42}']


def test_cloud_load_not_found_returns_empty_no_error(run_gb):
    with _mock_cloud_server() as base:
        out = _lines(run_gb(
            f'IMPORT "cloud"\nCLOUD_CONFIGURE("{base}", "")\n'
            'PRINT "["; CLOUD_LOAD("niemand"); "]"\n'
            'PRINT "["; CLOUD_LAST_ERROR$(); "]"\n'))
    assert out == ["[]", "[]"]


def test_cloud_save_overwrites(run_gb):
    with _mock_cloud_server() as base:
        out = _lines(run_gb(
            f'IMPORT "cloud"\nCLOUD_CONFIGURE("{base}", "")\n'
            'CLOUD_SAVE("p", "v1")\nCLOUD_SAVE("p", "v2")\n'
            'PRINT CLOUD_LOAD("p")\n'))
    assert out == ["v2"]


def test_cloud_wrong_api_key_fails_gracefully(run_gb):
    with _mock_cloud_server(api_key="geheim") as base:
        out = _lines(run_gb(
            f'IMPORT "cloud"\nCLOUD_CONFIGURE("{base}", "falsch")\n'
            'PRINT "["; CLOUD_LOAD("p"); "]"\n'
            'PRINT CLOUD_LAST_ERROR$() <> ""\n'))
    assert out == ["[]", "TRUE"]


def test_cloud_correct_api_key_works(run_gb):
    with _mock_cloud_server(api_key="geheim") as base:
        out = _lines(run_gb(
            f'IMPORT "cloud"\nCLOUD_CONFIGURE("{base}", "geheim")\n'
            'CLOUD_SAVE("p", "ok")\n'
            'PRINT CLOUD_LOAD("p")\n'))
    assert out == ["ok"]


def test_cloud_call_without_configure_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="CLOUD_CONFIGURE"):
        run_gb('IMPORT "cloud"\nPRINT CLOUD_LOAD("p")\n')


# --- LEADERBOARD_SUBMIT / LEADERBOARD_FETCH ---------------------------------

def test_leaderboard_submit_and_fetch_sorted_desc(run_gb):
    with _mock_cloud_server() as base:
        out = _lines(run_gb(
            f'IMPORT "cloud"\nCLOUD_CONFIGURE("{base}", "")\n'
            'LEADERBOARD_SUBMIT("hs", "anna", 100)\n'
            'LEADERBOARD_SUBMIT("hs", "bert", 250)\n'
            'LEADERBOARD_SUBMIT("hs", "carla", 50)\n'
            'DIM top AS ARRAY OF TUPLE\n'
            'top = LEADERBOARD_FETCH("hs", 10)\n'
            'PRINT LEN(top)\n'
            'DIM i AS INTEGER\n'
            'FOR i = 0 TO LEN(top) - 1\n'
            '    PRINT top[i][0]; " "; top[i][1]\n'
            'NEXT\n'))
    assert out == ["3", "bert 250.0", "anna 100.0", "carla 50.0"]


def test_leaderboard_keeps_only_best_score(run_gb):
    with _mock_cloud_server() as base:
        out = _lines(run_gb(
            f'IMPORT "cloud"\nCLOUD_CONFIGURE("{base}", "")\n'
            'PRINT LEADERBOARD_SUBMIT("hs", "anna", 100)\n'
            'PRINT LEADERBOARD_SUBMIT("hs", "anna", 50)\n'   # schlechter -> verworfen
            'PRINT LEADERBOARD_SUBMIT("hs", "anna", 300)\n'  # besser -> uebernommen
            'DIM top AS ARRAY OF TUPLE\n'
            'top = LEADERBOARD_FETCH("hs", 5)\n'
            'PRINT top[0][1]\n'))
    assert out == ["TRUE", "FALSE", "TRUE", "300.0"]


def test_leaderboard_best_low_mode(run_gb):
    with _mock_cloud_server() as base:
        out = _lines(run_gb(
            f'IMPORT "cloud"\nCLOUD_CONFIGURE("{base}", "")\n'
            'LEADERBOARD_SUBMIT("speedrun", "anna", 61.2, TRUE)\n'
            'LEADERBOARD_SUBMIT("speedrun", "anna", 58.9, TRUE)\n'  # kleiner ist besser
            'DIM top AS ARRAY OF TUPLE\n'
            'top = LEADERBOARD_FETCH("speedrun", 5, TRUE)\n'  # aufsteigend
            'PRINT top[0][1]\n'))
    assert out == ["58.9"]


def test_leaderboard_fetch_empty_board(run_gb):
    with _mock_cloud_server() as base:
        out = _lines(run_gb(
            f'IMPORT "cloud"\nCLOUD_CONFIGURE("{base}", "")\n'
            'DIM top AS ARRAY OF TUPLE\n'
            'top = LEADERBOARD_FETCH("leer", 10)\n'
            'PRINT LEN(top)\n'))
    assert out == ["0"]
