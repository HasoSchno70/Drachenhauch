"""Tests fuer das html-Modul (HTTP-Client + URL-Helpers + HTML-Parser).

HTTP-Pfade werden mit einem lokalen Mock-Server (ThreadingHTTPServer) auf
einem freien Port getestet, damit wir keine Netzwerk-Abhaengigkeit haben.
URL- und HTML-Parser-Pfade werden direkt aufgerufen.
"""
import threading
import contextlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

from gamebasic.modules import load_module
from gamebasic.modules import html as html_mod
from gamebasic.errors import GBRuntimeError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("html")


# --- Lokaler Mock-HTTP-Server -----------------------------------------

class _MockHandler(BaseHTTPRequestHandler):
    routes: dict = {}  # set per-test in fixture

    def log_message(self, *args, **kwargs):
        pass  # stumm

    def do_GET(self):
        spec = self.routes.get(("GET", self.path))
        if spec is None:
            self.send_response(404)
            self.end_headers()
            return
        status, headers, body = spec
        self.send_response(status)
        for k, v in headers.items():
            self.send_header(k, v)
        self.end_headers()
        if isinstance(body, str):
            body = body.encode("utf-8")
        self.wfile.write(body)

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length).decode("utf-8")
        spec = self.routes.get(("POST", self.path))
        if spec is None:
            self.send_response(404)
            self.end_headers()
            return
        status, headers, _stub = spec
        self.send_response(status)
        for k, v in headers.items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(f"echo: {body}".encode("utf-8"))


@contextlib.contextmanager
def _mock_server(routes: dict):
    _MockHandler.routes = routes
    server = ThreadingHTTPServer(("127.0.0.1", 0), _MockHandler)
    port = server.server_port
    t = threading.Thread(target=server.serve_forever, daemon=True)
    t.start()
    try:
        yield f"http://127.0.0.1:{port}"
    finally:
        server.shutdown()
        server.server_close()


# --- Registrierung ---------------------------------------------------

def test_all_builtins_registered():
    from gamebasic.interpreter import BUILTINS
    expected = {
        "http_get", "http_post", "http_download", "http_status", "http_header",
        "url_encode", "url_decode",
        "html_text", "html_find_all", "html_get_attr",
    }
    assert expected <= set(BUILTINS.keys())


# --- HTTP_GET / HTTP_STATUS / HTTP_HEADER -----------------------------

def test_http_get_returns_body(call_builtin):
    routes = {("GET", "/hello"): (200, {"X-Demo": "yes"}, "Hello world")}
    with _mock_server(routes) as base:
        body = call_builtin("http_get", [base + "/hello"])
        assert body == "Hello world"
        assert call_builtin("http_status", []) == 200
        assert call_builtin("http_header", ["X-Demo"]) == "yes"


def test_http_get_404_raises_with_status_set(call_builtin):
    routes = {}  # alles 404
    with _mock_server(routes) as base:
        with pytest.raises(GBRuntimeError, match="HTTP 404"):
            call_builtin("http_get", [base + "/nope"])
        # Nach dem Fehler ist HTTP_STATUS trotzdem lesbar
        assert call_builtin("http_status", []) == 404


def test_http_post_sends_body(call_builtin):
    routes = {("POST", "/echo"): (200, {}, "")}
    with _mock_server(routes) as base:
        body = call_builtin("http_post", [base + "/echo", "name=Anna&age=30"])
        assert body == "echo: name=Anna&age=30"


def test_http_download_writes_file(tmp_path, call_builtin):
    routes = {("GET", "/file"): (200, {}, b"\x00\x01\x02test")}
    out = tmp_path / "downloaded.bin"
    with _mock_server(routes) as base:
        n = call_builtin("http_download", [base + "/file", str(out)])
        assert n == len(b"\x00\x01\x02test")
        assert out.read_bytes() == b"\x00\x01\x02test"


def test_http_get_invalid_url_raises(call_builtin):
    with pytest.raises(GBRuntimeError):
        # Ungueltiges Schema -> URLError
        call_builtin("http_get", ["not-a-url"])


# --- URL-Helpers ----------------------------------------------------

def test_url_encode_special_chars(call_builtin):
    assert call_builtin("url_encode", ["Hallo Welt!"]) == "Hallo%20Welt%21"


def test_url_encode_decode_roundtrip(call_builtin):
    raw = "ä?&=#/+ space"
    enc = call_builtin("url_encode", [raw])
    assert call_builtin("url_decode", [enc]) == raw


# --- HTML_TEXT ------------------------------------------------------

def test_html_text_strips_tags(call_builtin):
    out = call_builtin("html_text", ["<p>Hallo <b>Welt</b></p>"])
    assert out == "Hallo Welt"


def test_html_text_decodes_entities(call_builtin):
    out = call_builtin("html_text", ["<p>5 &lt; 10 &amp; ok</p>"])
    assert out == "5 < 10 & ok"


def test_html_text_skips_script_and_style(call_builtin):
    src = """
    <html><head>
    <style>body { color: red; }</style>
    <script>alert('hi')</script>
    </head><body>Sichtbarer Text</body></html>
    """
    out = call_builtin("html_text", [src])
    assert "color" not in out
    assert "alert" not in out
    assert "Sichtbarer Text" in out


def test_html_text_block_separators(call_builtin):
    out = call_builtin("html_text", ["<p>Eins</p><p>Zwei</p>"])
    # Zwischen den Absaetzen sollte ein Newline sein
    assert "Eins" in out and "Zwei" in out
    assert "\n" in out or " " in out  # Trennung vorhanden


# --- HTML_FIND_ALL --------------------------------------------------

def _arr_values(gb_arr):
    """Helper: holt die Python-Liste aus einem _GBArray-Result."""
    return list(gb_arr.values)


def test_find_all_simple_tags(call_builtin):
    out = call_builtin(
        "html_find_all",
        ['<a href="x">eins</a> Filler <a>zwei</a>', "a"],
    )
    assert _arr_values(out) == ["eins", "zwei"]


def test_find_all_no_matches_returns_empty(call_builtin):
    out = call_builtin("html_find_all", ["<p>x</p>", "a"])
    assert _arr_values(out) == []


def test_find_all_preserves_inner_tags(call_builtin):
    out = call_builtin(
        "html_find_all",
        ["<li><b>bold</b> text</li>", "li"],
    )
    vals = _arr_values(out)
    assert len(vals) == 1
    assert "<b>bold</b>" in vals[0]


def test_find_all_nested_same_tag(call_builtin):
    """Verschachtelte gleiche Tags: aeusserer Match enthaelt inneren."""
    src = "<div><div>inner</div></div>"
    out = call_builtin("html_find_all", [src, "div"])
    vals = _arr_values(out)
    assert len(vals) == 1
    assert "<div>inner</div>" in vals[0]


# --- HTML_GET_ATTR --------------------------------------------------

def test_get_attr_double_quoted(call_builtin):
    assert call_builtin(
        "html_get_attr",
        ['<a href="https://x.com" class="big">link</a>', "href"],
    ) == "https://x.com"


def test_get_attr_single_quoted(call_builtin):
    assert call_builtin(
        "html_get_attr",
        ["<a href='https://x.com'>link</a>", "href"],
    ) == "https://x.com"


def test_get_attr_unquoted(call_builtin):
    assert call_builtin(
        "html_get_attr",
        ["<input value=42 type=hidden>", "value"],
    ) == "42"


def test_get_attr_missing_returns_empty(call_builtin):
    assert call_builtin(
        "html_get_attr",
        ["<a href='x'>link</a>", "title"],
    ) == ""


def test_get_attr_decodes_entities(call_builtin):
    """Der Wert wird als HTML-Entity-dekodiert ausgegeben."""
    assert call_builtin(
        "html_get_attr",
        ['<a title="A &amp; B">link</a>', "title"],
    ) == "A & B"
