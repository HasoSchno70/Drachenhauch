"""HTTP- und HTML-Modul fuer GameBasic.

Komplett auf der Python-Standardbibliothek (urllib, html.parser, html) -
*kein* externer pip-Install noetig.

Built-ins:
    HTTP_GET(url$)                    -> STRING (Response-Body)
    HTTP_POST(url$, body$)            -> STRING (Response-Body)
    HTTP_DOWNLOAD(url$, pfad$)        -> INTEGER (geschriebene Bytes)
    HTTP_STATUS()                     -> INTEGER (letzter Status, z.B. 200)
    HTTP_HEADER(name$)                -> STRING (letzter Response-Header)
    URL_ENCODE(s$)                    -> STRING (%-encoding)
    URL_DECODE(s$)                    -> STRING
    HTML_TEXT(html$)                  -> STRING (Tags raus, Entities dekodiert)
    HTML_FIND_ALL(html$, tag$)        -> ARRAY OF STRING (innere Inhalte)
    HTML_GET_ATTR(tag_html$, attr$)   -> STRING (Attribut-Wert oder "")

Sicherheits-/UX-Notizen:
- Default-Timeout 10 Sekunden bei HTTP-Aufrufen, sonst friert die GB-VM ein
  bis der Server antwortet.
- HTTP-Fehler (Timeout, Verbindungsfehler, 4xx/5xx) werfen GBRuntimeError mit
  Status-Code in der Meldung. HTTP_STATUS() bleibt aber gesetzt.
- Bytes vs. Text: HTTP_GET/POST geben *STRING* zurueck (utf-8 mit replace).
  Fuer Binary lieber HTTP_DOWNLOAD nutzen.
"""
from __future__ import annotations

import html as _html_lib  # html.unescape - aufpassen: kein Konflikt mit unserem Modul
import re as _re
import urllib.parse as _urlparse
import urllib.request as _urlreq
import urllib.error as _urlerr
from html.parser import HTMLParser as _HTMLParser

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError


_DEFAULT_TIMEOUT = 10.0
# User-Agent setzen damit Server, die Python-urllib blocken, uns trotzdem
# antworten. Browser-aehnlicher Header.
_USER_AGENT = "GameBasic/0.1 (+https://github.com/anthropic-ai)"

# Stand der letzten Response - HTTP_STATUS / HTTP_HEADER greifen darauf zu
_last_status: int = 0
_last_headers: dict = {}


def _set_last(status: int, headers):
    global _last_status, _last_headers
    _last_status = int(status)
    _last_headers = {k.lower(): v for k, v in (headers.items() if hasattr(headers, "items") else [])}


def _do_request(req, timeout: float) -> bytes:
    """Fuehrt urllib-Request aus, setzt _last_*, gibt Body als Bytes zurueck."""
    try:
        with _urlreq.urlopen(req, timeout=timeout) as resp:
            _set_last(resp.status, resp.headers)
            return resp.read()
    except _urlerr.HTTPError as exc:
        # 4xx/5xx kommt als HTTPError - wir wollen Status weiterhin lesbar
        _set_last(exc.code, exc.headers)
        raise GBRuntimeError(
            f"HTTP {exc.code} {exc.reason} - {req.full_url}"
        )
    except _urlerr.URLError as exc:
        raise GBRuntimeError(f"HTTP-Fehler: {exc.reason} - {req.full_url}")
    except TimeoutError:
        raise GBRuntimeError(f"HTTP-Timeout ({timeout}s): {req.full_url}")


def _make_get(url: str):
    try:
        return _urlreq.Request(url, headers={"User-Agent": _USER_AGENT})
    except ValueError as exc:
        raise GBRuntimeError(f"HTTP: Ungueltige URL '{url}' ({exc})")


def _make_post(url: str, body: str):
    try:
        return _urlreq.Request(
            url,
            data=body.encode("utf-8"),
            method="POST",
            headers={
                "User-Agent": _USER_AGENT,
                "Content-Type": "application/x-www-form-urlencoded; charset=utf-8",
            },
        )
    except ValueError as exc:
        raise GBRuntimeError(f"HTTP: Ungueltige URL '{url}' ({exc})")


@builtin("HTTP_GET", arity=1, types=("str",))
def _http_get(url):
    body = _do_request(_make_get(url), _DEFAULT_TIMEOUT)
    return body.decode("utf-8", errors="replace")


@builtin("HTTP_POST", arity=2, types=("str", "str"))
def _http_post(url, body):
    response = _do_request(_make_post(url, body), _DEFAULT_TIMEOUT)
    return response.decode("utf-8", errors="replace")


@builtin("HTTP_DOWNLOAD", arity=2, types=("str", "str"))
def _http_download(url, path):
    body = _do_request(_make_get(url), _DEFAULT_TIMEOUT)
    try:
        with open(path, "wb") as fh:
            fh.write(body)
    except OSError as exc:
        raise GBRuntimeError(f"HTTP_DOWNLOAD: Datei nicht schreibbar: {exc}")
    return len(body)


@builtin("HTTP_STATUS", arity=0)
def _http_status():
    return _last_status


@builtin("HTTP_HEADER", arity=1, types=("str",))
def _http_header(name):
    return _last_headers.get(name.lower(), "")


# --- URL-Helpers ---------------------------------------------------

@builtin("URL_ENCODE", arity=1, types=("str",))
def _url_encode(s):
    return _urlparse.quote(s, safe="")


@builtin("URL_DECODE", arity=1, types=("str",))
def _url_decode(s):
    return _urlparse.unquote(s)


# --- HTML-Parser ---------------------------------------------------

class _TextStripper(_HTMLParser):
    """Strippt alle Tags + dekodiert Entities."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.parts: list = []
        self._skip_depth = 0

    SKIP_TAGS = {"script", "style", "noscript"}

    def handle_starttag(self, tag, attrs):
        if tag in self.SKIP_TAGS:
            self._skip_depth += 1
        # Block-Level-Tags ergeben einen Whitespace-Trenner damit zwei
        # aufeinanderfolgende <p>...<p> nicht zu einem langen Wort werden.
        if tag in ("p", "br", "div", "li", "tr", "h1", "h2", "h3", "h4", "h5", "h6"):
            self.parts.append("\n")

    def handle_endtag(self, tag):
        if tag in self.SKIP_TAGS and self._skip_depth > 0:
            self._skip_depth -= 1

    def handle_data(self, data):
        if self._skip_depth == 0:
            self.parts.append(data)


@builtin("HTML_TEXT", arity=1, types=("str",))
def _html_text(s):
    p = _TextStripper()
    try:
        p.feed(s)
    except Exception as exc:
        raise GBRuntimeError(f"HTML_TEXT: Parser-Fehler: {exc}")
    out = "".join(p.parts)
    # Mehrfach-Whitespace zusammenziehen, aber Newlines respektieren
    out = _re.sub(r"[ \t]+", " ", out)
    out = _re.sub(r"\n{3,}", "\n\n", out)
    return out.strip()


class _TagFinder(_HTMLParser):
    """Sammelt fuer einen Tag-Namen alle Inner-HTML-Inhalte (mit Tags drin).

    Beispiel: tag='a' im HTML '<a href="x">click</a> text <a>two</a>' liefert
    ['click', 'two'].

    Verschachtelte gleiche Tags werden korrekt gepaart (Stack-Tracking).
    """

    def __init__(self, target: str):
        super().__init__(convert_charrefs=False)
        self.target = target.lower()
        self.found: list = []
        self._stack_depth = 0
        self._buf: list = []

    def handle_starttag(self, tag, attrs):
        if tag == self.target:
            if self._stack_depth == 0:
                self._buf = []
            else:
                # bereits in einem Match - wir wollen den Tag *im Inhalt*
                self._buf.append(self.get_starttag_text() or f"<{tag}>")
            self._stack_depth += 1
            return
        if self._stack_depth > 0:
            self._buf.append(self.get_starttag_text() or f"<{tag}>")

    def handle_endtag(self, tag):
        if tag == self.target:
            if self._stack_depth > 1:
                self._buf.append(f"</{tag}>")
            self._stack_depth -= 1
            if self._stack_depth == 0:
                self.found.append("".join(self._buf))
                self._buf = []
            return
        if self._stack_depth > 0:
            self._buf.append(f"</{tag}>")

    def handle_startendtag(self, tag, attrs):
        if self._stack_depth > 0:
            self._buf.append(self.get_starttag_text() or f"<{tag}/>")

    def handle_data(self, data):
        if self._stack_depth > 0:
            self._buf.append(data)

    def handle_entityref(self, name):
        if self._stack_depth > 0:
            self._buf.append(f"&{name};")

    def handle_charref(self, name):
        if self._stack_depth > 0:
            self._buf.append(f"&#{name};")


def _to_gb_string_array(items: list):
    """Wickelt eine Python-Liste in das GameBasic _GBArray ein, das die VM
    als ARRAY OF STRING akzeptiert. Lazy-Import, um Zirkular-Imports beim
    Modul-Load zu vermeiden."""
    from ..interpreter import _GBArray
    arr = _GBArray("string", [len(items)], lambda: "")
    for i, p in enumerate(items):
        arr.values[i] = p
    return arr


@builtin("HTML_FIND_ALL", arity=2, types=("str", "str"))
def _html_find_all(s, tag):
    p = _TagFinder(tag)
    try:
        p.feed(s)
    except Exception as exc:
        raise GBRuntimeError(f"HTML_FIND_ALL: Parser-Fehler: {exc}")
    return _to_gb_string_array(list(p.found))


# Regex-basiert: simpler als Parser, fuer den Single-Tag-Fall (User uebergibt
# den Outer-HTML eines Tags, wir extrahieren ein Attribut). Anfuehrungszeichen
# einfach oder doppelt, kein Quote (HTML5 erlaubt unquoted-Werte).
_RE_ATTR_DBL = _re.compile(r'\s+%s\s*=\s*"([^"]*)"', _re.IGNORECASE)
_RE_ATTR_SGL = _re.compile(r"\s+%s\s*=\s*'([^']*)'", _re.IGNORECASE)
_RE_ATTR_BARE = _re.compile(r"\s+%s\s*=\s*([^\s>]+)", _re.IGNORECASE)


@builtin("HTML_GET_ATTR", arity=2, types=("str", "str"))
def _html_get_attr(s, attr):
    """Holt Attribut-Wert aus dem ersten passenden Tag in `s`.

    Fuer 'href' im '<a href="x" class="big">..</a>' liefert 'x'. Wenn das
    Attribut nicht existiert, kommt "" zurueck.
    """
    if not attr:
        return ""
    name = _re.escape(attr)
    for rx in (_RE_ATTR_DBL, _RE_ATTR_SGL, _RE_ATTR_BARE):
        m = _re.search(rx.pattern % name, s, _re.IGNORECASE)
        if m:
            return _html_lib.unescape(m.group(1))
    return ""
