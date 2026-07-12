"""Tests fuer das html-Modul (HTTP-Client + URL-Helpers + HTML-Parser).

Golden-Tests gegen `gbrt` (Stufe B): HTTP-Pfade laufen gegen einen lokalen
Mock-Server (ThreadingHTTPServer im pytest-Prozess); das GB-Programm laeuft im
gbrt-Subprozess und macht echte localhost-Requests dorthin. URL-/HTML-Parser sind
pure. Frueher via `call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""
import contextlib
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

from gamebasic.errors import GBRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


# --- Lokaler Mock-HTTP-Server -----------------------------------------

class _MockHandler(BaseHTTPRequestHandler):
    routes: dict = {}

    def log_message(self, *args, **kwargs):
        pass

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
    t = threading.Thread(target=server.serve_forever, daemon=True)
    t.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}"
    finally:
        server.shutdown()
        server.server_close()


# --- HTTP_GET / HTTP_STATUS / HTTP_HEADER -----------------------------

def test_http_get_returns_body(run_gb):
    routes = {("GET", "/hello"): (200, {"X-Demo": "yes"}, "Hello world")}
    with _mock_server(routes) as base:
        out = _lines(run_gb(
            f'IMPORT "html"\nPRINT HTTP_GET("{base}/hello")\n'
            'PRINT HTTP_STATUS()\nPRINT HTTP_HEADER("X-Demo")\n'))
    assert out == ["Hello world", "200", "yes"]


def test_http_get_404_raises_with_status_set(run_gb):
    with _mock_server({}) as base:
        with pytest.raises(GBRuntimeError, match="404"):
            run_gb(f'IMPORT "html"\nPRINT HTTP_GET("{base}/nope")\n')


def test_http_post_sends_body(run_gb):
    routes = {("POST", "/echo"): (200, {}, "")}
    with _mock_server(routes) as base:
        out = _lines(run_gb(
            f'IMPORT "html"\nPRINT HTTP_POST("{base}/echo", "name=Anna&age=30")\n'))
    assert out == ["echo: name=Anna&age=30"]


def test_http_download_writes_file(run_gb, tmp_path):
    routes = {("GET", "/file"): (200, {}, b"\x00\x01\x02test")}
    with _mock_server(routes) as base:
        out = _lines(run_gb(
            f'IMPORT "html"\nPRINT HTTP_DOWNLOAD("{base}/file", "dl.bin")\n',
            base=tmp_path))
    assert out == [str(len(b"\x00\x01\x02test"))]
    assert (tmp_path / "dl.bin").read_bytes() == b"\x00\x01\x02test"


def test_http_get_invalid_url_raises(run_gb):
    with pytest.raises(GBRuntimeError):
        run_gb('IMPORT "html"\nPRINT HTTP_GET("not-a-url")\n')


class _TruncatedHandler(BaseHTTPRequestHandler):
    """Verspricht per Content-Length mehr Bytes als gesendet werden und kappt
    dann die Verbindung -- simuliert einen mitten im Transfer abgebrochenen
    Download. Frueher wurde das still verschluckt (Body als Erfolg
    zurueckgegeben, abgeschnitten)."""

    def log_message(self, *args, **kwargs):
        pass

    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Length", "1000000")
        self.end_headers()
        self.wfile.write(b"nur ein paar bytes")
        self.wfile.flush()
        self.close_connection = True


@contextlib.contextmanager
def _truncated_server():
    server = ThreadingHTTPServer(("127.0.0.1", 0), _TruncatedHandler)
    t = threading.Thread(target=server.serve_forever, daemon=True)
    t.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}"
    finally:
        server.shutdown()
        server.server_close()


def test_http_get_truncated_body_raises_instead_of_returning_partial(run_gb):
    with _truncated_server() as base:
        with pytest.raises(GBRuntimeError):
            run_gb(f'IMPORT "html"\nPRINT HTTP_GET("{base}/x")\n')


def test_http_download_truncated_body_raises_and_removes_partial_file(run_gb, tmp_path):
    with _truncated_server() as base:
        with pytest.raises(GBRuntimeError):
            run_gb(f'IMPORT "html"\nHTTP_DOWNLOAD("{base}/x", "dl.bin")\n', base=tmp_path)
    # Keine abgeschnittene Datei zurueckgelassen.
    assert not (tmp_path / "dl.bin").exists()


# --- URL-Helpers ----------------------------------------------------

def test_url_encode_special_chars(run_gb):
    assert _lines(run_gb('IMPORT "html"\nPRINT URL_ENCODE("Hallo Welt!")\n')) == \
        ["Hallo%20Welt%21"]


def test_url_encode_decode_roundtrip(run_gb):
    raw = "ä?&=#/+ space"   # ae-Umlaut + URL-Sonderzeichen
    out = run_gb(f'IMPORT "html"\nPRINT URL_DECODE(URL_ENCODE("{raw}"))\n')
    assert _lines(out) == [raw]


# --- HTML_TEXT ------------------------------------------------------

def test_html_text_strips_tags(run_gb):
    assert _lines(run_gb('IMPORT "html"\nPRINT HTML_TEXT("<p>Hallo <b>Welt</b></p>")\n')) == \
        ["Hallo Welt"]


def test_html_text_decodes_entities(run_gb):
    assert _lines(run_gb('IMPORT "html"\nPRINT HTML_TEXT("<p>5 &lt; 10 &amp; ok</p>")\n')) == \
        ["5 < 10 & ok"]


def test_html_text_comment_with_gt_does_not_leak_into_text(run_gb):
    # Ein '>' INNERHALB eines Kommentars ist gueltiges HTML -- der Tag-Scanner
    # darf dort nicht vorzeitig abbrechen (frueher landete der Kommentar-Rest
    # als sichtbarer Text in der Ausgabe).
    src = "<p>vorher</p><!-- if (x>1) then y --><p>nachher</p>"
    out = run_gb(f'IMPORT "html"\nPRINT HTML_TEXT("{src}")\n')
    assert "if (x" not in out
    assert "vorher" in out
    assert "nachher" in out


def test_html_text_skips_script_and_style(run_gb):
    src = ("<html><head><style>body { color: red; }</style>"
           "<script>alert('hi')</script></head><body>Sichtbarer Text</body></html>")
    out = run_gb(f'IMPORT "html"\nPRINT HTML_TEXT("{src}")\n')
    assert "color" not in out
    assert "alert" not in out
    assert "Sichtbarer Text" in out


# --- HTML_FIND_ALL --------------------------------------------------

def test_find_all_simple_tags(run_gb):
    out = _lines(run_gb(
        'IMPORT "html"\nDIM a AS ARRAY OF STRING\n'
        'a = HTML_FIND_ALL("<a href=""x"">eins</a> Filler <a>zwei</a>", "a")\n'
        "PRINT LEN(a)\nPRINT a[0]\nPRINT a[1]\n"))
    assert out == ["2", "eins", "zwei"]


def test_find_all_no_matches_returns_empty(run_gb):
    out = _lines(run_gb(
        'IMPORT "html"\nDIM a AS ARRAY OF STRING\n'
        'a = HTML_FIND_ALL("<p>x</p>", "a")\nPRINT LEN(a)\n'))
    assert out == ["0"]


def test_find_all_preserves_inner_tags(run_gb):
    out = _lines(run_gb(
        'IMPORT "html"\nDIM a AS ARRAY OF STRING\n'
        'a = HTML_FIND_ALL("<li><b>bold</b> text</li>", "li")\n'
        "PRINT LEN(a)\nPRINT a[0]\n"))
    assert out[0] == "1"
    assert "<b>bold</b>" in out[1]


def test_find_all_orphaned_closing_tag_does_not_break_later_matches(run_gb):
    # Ein verwaistes </div> VOR jedem passenden <div> darf den Tiefenzaehler
    # nicht unter 0 druecken -- sonst faellt der danach folgende, korrekt
    # verschachtelte Treffer stillschweigend aus dem Ergebnis.
    out = _lines(run_gb(
        'IMPORT "html"\nDIM a AS ARRAY OF STRING\n'
        'a = HTML_FIND_ALL("</div><div>echt</div>", "div")\n'
        "PRINT LEN(a)\nPRINT a[0]\n"))
    assert out == ["1", "echt"]


def test_find_all_nested_same_tag(run_gb):
    out = _lines(run_gb(
        'IMPORT "html"\nDIM a AS ARRAY OF STRING\n'
        'a = HTML_FIND_ALL("<div><div>inner</div></div>", "div")\n'
        "PRINT LEN(a)\nPRINT a[0]\n"))
    assert out[0] == "1"
    assert "<div>inner</div>" in out[1]


# --- HTML_GET_ATTR --------------------------------------------------

def test_get_attr_double_quoted(run_gb):
    assert _lines(run_gb(
        'IMPORT "html"\nPRINT HTML_GET_ATTR('
        '"<a href=""https://x.com"" class=""big"">link</a>", "href")\n')) == \
        ["https://x.com"]


def test_get_attr_single_quoted(run_gb):
    assert _lines(run_gb(
        'IMPORT "html"\nPRINT HTML_GET_ATTR("<a href=\'https://x.com\'>link</a>", "href")\n')) == \
        ["https://x.com"]


def test_get_attr_unquoted(run_gb):
    assert _lines(run_gb(
        'IMPORT "html"\nPRINT HTML_GET_ATTR("<input value=42 type=hidden>", "value")\n')) == \
        ["42"]


def test_get_attr_missing_returns_empty(run_gb):
    assert _lines(run_gb(
        'IMPORT "html"\nPRINT "[" + HTML_GET_ATTR("<a href=\'x\'>link</a>", "title") + "]"\n')) == \
        ["[]"]


def test_get_attr_decodes_entities(run_gb):
    assert _lines(run_gb(
        'IMPORT "html"\nPRINT HTML_GET_ATTR("<a title=""A &amp; B"">link</a>", "title")\n')) == \
        ["A & B"]
