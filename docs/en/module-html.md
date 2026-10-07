# Module `html`

HTTP client + URL helpers + HTML parser. All native in `dhrt` (Rust, HTTP via
`ureq`) — **no** pip install, no Python at run time.

```basic
IMPORT "html"
```

## Overview

| Area | Function | Returns | Meaning |
|---|---|---|---|
| HTTP | `HTTP_REQUEST(methode$, url$ [, rumpf [, kopfzeilen]])` | STRING (response body) | any HTTP method with its own body and headers |
| HTTP | `HTTP_GET(url$)` | STRING (response body) | fetch a page or file |
| HTTP | `HTTP_POST(url$, body$)` | STRING | send data and read the response |
| HTTP | `HTTP_DOWNLOAD(url$, pfad$)` | INTEGER (bytes) | download a file and save it |
| HTTP | `HTTP_STATUS()` | INTEGER (e.g. 200, 404) | status of the last request (200 = good, 404 = not there) |
| HTTP | `HTTP_HEADER(name$)` | STRING (header of the response) | read a header of the last response |
| HTTP | `HTTP_BYTES()` | BUFFER (raw body of the last response) |  |
| HTTP | `HTTP_SET_HEADER(name$, wert$)` | — (applies to all following calls) |  |
| HTTP | `HTTP_CLEAR_HEADERS()` | — |  |
| HTTP | `HTTP_TIMEOUT(sekunden)` | — |  |
| HTTP (background) | `HTTP_REQUEST_START(methode$, url$ [, rumpf [, kopfzeilen]])` | INTEGER (request number) |  |
| HTTP (background) | `HTTP_GET_START(url$)` | INTEGER (request number) |  |
| HTTP (background) | `HTTP_READY(abruf)` | BOOLEAN |  |
| HTTP (background) | `HTTP_RESULT(abruf)` | STRING (response body) |  |
| HTTP (background) | `HTTP_CANCEL(abruf)` | — |  |
| HTTP (background) | `HTTP_PENDING()` | INTEGER (open requests) |  |
| HTTP (background) | `HTTP_URL$(abruf)` | STRING |  |
| URL | `URL_ENCODE(s$)` | STRING | make text safe for a URL (spaces, umlauts, `&`) |
| URL | `URL_DECODE(s$)` | STRING | the counterpart — turn encoded text back |
| HTML | `HTML_TEXT(html$)` | STRING (tags removed, entities decoded) |  |
| HTML | `HTML_FIND_ALL(html$, tag$)` | ARRAY OF STRING |  |
| HTML | `HTML_GET_ATTR(tag_html$, attr$)` | STRING |  |

## HTTP_REQUEST — for everything beyond GET and POST

Real interfaces want more than GET and POST: a login token in a header, a
`PUT` to change, a `DELETE` to remove, JSON as the content type. There is
**one** command for that:

```basic
IMPORT "html"

DIM kopf AS MAP OF STRING
DIM antwort AS STRING

MAPPUT(kopf, "Authorization", "Bearer " + token)
MAPPUT(kopf, "Content-Type", "application/json")

antwort = HTTP_REQUEST("PUT", "https://api.example.com/dinge/7", _
                       "{""name"": ""Anna""}", kopf)
PRINT HTTP_STATUS()
```

`methode$` is one of `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`,
`OPTIONS` (case does not matter). Anything else is an **error** — a `"GTE"`
would otherwise come back as a confusing server response instead of a message
at the line where the typo is.

`rumpf` may be a `STRING` or a `BUFFER` (see
[Bytes](builtins-core.md#bytes-buffer)) — so an image or a zip file can be
uploaded too. Omitted or `""` means: no body.

`kopfzeilen` is a `MAP OF STRING`. It overrides defaults of the same name.

> **`HTTP_REQUEST` does not guess a `Content-Type`.** Only the caller knows
> what type a body has; a wrongly guessed `application/json` would be worse
> than none at all. If you send JSON, you set the header yourself. (`HTTP_POST`
> on the other hand keeps its old default `application/x-www-form-urlencoded`
> — existing programs depend on it.)

### Setting a token once

If all calls need the same login, the header does not have to go on each one:

```basic
HTTP_SET_HEADER("Authorization", "Bearer " + token)

PRINT HTTP_REQUEST("GET", basis + "/profil")      ' token included
PRINT HTTP_GET(basis + "/nachrichten")            ' here too

HTTP_CLEAR_HEADERS()                              ' log out again
```

This applies to **all** following HTTP calls of the module, including
`HTTP_GET` and the background variants. A header passed with an individual
call wins over the permanent one. The same name twice replaces instead of
piling up — otherwise both would go out and the server would decide.

### Raw bytes as the response

The return value is text: bytes that cannot be decoded are replaced. With an
image or a zip file nothing usable would remain. `HTTP_BYTES()` therefore
returns the **raw** body of the last response:

```basic
DIM egal AS STRING
DIM bild AS BUFFER
egal = HTTP_REQUEST("GET", "https://example.com/logo.png")
bild = HTTP_BYTES()
WRITEALL_BYTES("logo.png", bild)
```

`HTTP_BYTES()` belongs — like `HTTP_STATUS()` and `HTTP_HEADER()` — to the
*last* response and works after every HTTP call, including `HTTP_GET`. After a
failed call it is empty (and not, say, still the previous response). If you
only want to save a large file, `HTTP_DOWNLOAD` still serves you better — it
streams straight to disk.

### Time limit

```basic
HTTP_TIMEOUT(30)      ' seconds, 1..600; default is 10
```

Applies to all following calls.

## HTTP client

`HTTP_GET` / `HTTP_POST` are the short forms for the common case and return the response body as a STRING (UTF-8 with a replace strategy for anything undecodable).

```basic
IMPORT "html"

DIM body AS STRING
body = HTTP_GET("https://api.example.com/scores")
PRINT "Status: ", HTTP_STATUS()
PRINT body
```

**Default timeout: 10 seconds.** On a timeout, a connection error or a 4xx/5xx status the function throws `DHRuntimeError` with the status code in the message. After a 4xx/5xx, `HTTP_STATUS()` remains readable — useful for the `TRY/CATCH` pattern:

```basic
TRY
    DIM resp AS STRING
    resp = HTTP_GET("https://api.example.com/missing")
CATCH e
    IF HTTP_STATUS() = 404 THEN
        PRINT "Endpoint existiert nicht."
    ELSE
        PRINT "Anderer Fehler: ", e
    END IF
END TRY
```

`HTTP_DOWNLOAD` writes straight into a file — sensible for binary data (images, sounds), where `HTTP_GET` with UTF-8 replacement would corrupt the bytes.

```basic
DIM bytes AS INTEGER
bytes = HTTP_DOWNLOAD("https://example.com/sprite.png", "assets/sprite.png")
PRINT "Heruntergeladen: ", bytes, " Bytes"
```

`HTTP_HEADER(name$)` returns a response header of the last response. Header names are case-insensitive (`Content-Type` and `content-type` find the same).

## Requests in the background

`HTTP_GET` halts the whole program until the response is there. In a window that means: no mouse, no key, no redrawing. A medium JSON request takes ~200 ms — about twelve dropped frames, and on a bad connection the window waits until the 10-second timeout.

For anything running in a loop there is therefore the same request for checking instead of waiting:

| Function | Effect |
|---|---|
| `HTTP_REQUEST_START(methode$, url$ [, rumpf [, kopfzeilen]])` → INTEGER | starts **any** request in the background |
| `HTTP_GET_START(url$)` → INTEGER | short form of it for GET |
| `HTTP_READY(abruf)` → BOOLEAN | is the response there? (checks, does not wait) |
| `HTTP_RESULT(abruf)` → STRING | collects the body and frees the slot |
| `HTTP_CANCEL(abruf)` | discard a request (unknown number = no-op) |
| `HTTP_PENDING()` → INTEGER | how many requests are still open |
| `HTTP_URL$(abruf)` → STRING | the URL of a running request |

The pattern is the same as with `INPUT_UPDATE()`/`TIMER_UPDATE()`: check once per frame.

`HTTP_RESULT` sets `HTTP_STATUS()`, `HTTP_HEADER()` and `HTTP_BYTES()` just
like a blocking call — on **collecting**, not on starting. With several
simultaneous requests these values therefore always belong to the one
collected last.

```basic
IMPORT "html"

DIM abruf AS INTEGER
DIM daten AS STRING
abruf = HTTP_GET_START("https://api.example.com/scores")

SCREEN(400, 120, "Lade")
WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 24, 32))
    IF abruf >= 0 AND HTTP_READY(abruf) THEN
        daten = HTTP_RESULT(abruf)
        abruf = -1
    END IF
    IF daten = "" THEN
        TEXT(20, 40, "Lade ... (" + STR$(HTTP_PENDING()) + " offen)", RGB(200, 200, 200))
    ELSE
        TEXT(20, 40, "Fertig: " + STR$(LEN(daten)) + " Zeichen", RGB(120, 220, 140))
    END IF
    FLIP()
WEND
```

After `HTTP_RESULT`, `HTTP_STATUS()` and `HTTP_HEADER()` are ready exactly as after `HTTP_GET`.

**Details:**

- **Errors come when collecting.** A 404, a timeout or a connection error lets `HTTP_GET_START` pass and only throws at `HTTP_RESULT` — where the program can deal with it (`TRY`/`CATCH` around the collecting).
- **`HTTP_READY` may be asked any number of times.** After the first `TRUE` the response stays there until it is collected. Only `HTTP_RESULT` consumes it.
- **`HTTP_RESULT` before `HTTP_READY`** aborts with a plain-language message instead of blocking — otherwise the advantage would be thrown away again.
- **Several requests really run at the same time.** Two requests of 300 ms each are done after ~300 ms, not after 600.
- **Numbers stay stable** (tombstones as with `timer`): a collected or cancelled number is not handed out again. So a program cannot accidentally refer an old number to someone else's request.
- **Cancelling does not stop the request in mid-network** — it runs to the end, and its result is discarded. The program does not wait for anything.
- **GET only.** That is enough for fetching data, and every further form doubles the states a program has to keep track of. For POST/download in a loop: a rare call at a place where a short standstill does not matter.

## URL helpers

```basic
DIM q AS STRING
q = URL_ENCODE("hallo welt & co")    ' "hallo%20welt%20%26%20co"
DIM url AS STRING
url = "https://api.example.com/search?q=" + q
```

`URL_DECODE` does the opposite — `%XX` sequences become characters, `+` stays `+`. Both are written in Rust but return the same as Python's `urllib.parse.quote`/`unquote` (measured, not merely intended).

## HTML parser

`HTML_TEXT` strips all tags and decodes HTML entities (`&amp;` → `&`, `&lt;` → `<`, …). Block-level tags (`<p>`, `<div>`, `<li>`, …) produce a newline. `<script>` and `<style>` are skipped entirely.

```basic
DIM html AS STRING
html = HTTP_GET("https://example.com")
PRINT HTML_TEXT(html)
```

`HTML_FIND_ALL(html, tag)` returns all inner HTML contents of a tag as an array. Nested tags of the same name are paired correctly (stack tracking):

```basic
DIM links AS ARRAY OF STRING
links = HTML_FIND_ALL(html, "a")
DIM i AS INTEGER
FOR i = 0 TO LEN(links) - 1
    PRINT "Link-Inhalt: ", links[i]
NEXT
```

`HTML_GET_ATTR(tag_html, attr)` extracts an attribute from the first matching tag in the given snippet. Supports double/single quotes and unquoted values (HTML5):

```basic
DIM all_a AS ARRAY OF STRING
all_a = HTML_FIND_ALL(html, "a")
' For each <a> tag we need its outer HTML - but HTML_FIND_ALL only returns
' the CONTENT between the tags. Whoever needs the href$ scrapes with
' regex or rebuilds the tag - or uses HTML_GET_ATTR on an already
' known tag snippet:
DIM href AS STRING
href = HTML_GET_ATTR("<a href='https://example.com' title='go'>...</a>", "href")
PRINT href                    ' "https://example.com"
```

> **Limit**: `HTML_FIND_ALL` returns the inner HTML, not the outer. If you need attributes AND content, you would have to parse the inner separately — or use `HTTP_GET` + regex/JSON directly if the API returns JSON.

## Complete example — pulling news titles from a web page

```basic
IMPORT "html"

DIM page AS STRING
page = HTTP_GET("https://example.com/news")

DIM titles AS ARRAY OF STRING
titles = HTML_FIND_ALL(page, "h2")

PRINT "Aktuelle Schlagzeilen:"
DIM i AS INTEGER
FOR i = 0 TO LEN(titles) - 1
    PRINT "  - ", HTML_TEXT(titles[i])    ' inner HTML to plain text
NEXT
```

## Combining with a JSON API

Most modern APIs return JSON, not HTML. Combine `html` with the `json` module:

```basic
IMPORT "html"
IMPORT "json"

DIM resp AS STRING
resp = HTTP_GET("https://api.github.com/repos/python/cpython")

DIM info AS JSON_HANDLE
info = JSON_PARSE(resp)
PRINT "Repo: ", JSON_GET_STRING(info, "full_name")
PRINT "Stars: ", JSON_GET_INT(info, "stargazers_count")
```

## Security / privacy notes

- **User agent**: the module sets its own user agent (`Drachenhauch/0.1 …`). Some servers block Python's default `urllib` UA — the custom header gets around that.
- **HTTPS**: certificate validation against the system CA store. Self-signed certificates are rejected — no bypass option in the module (on purpose).
- **Cookies / sessions** are not stored — every call stands on its own. For authenticated APIs, token auth is the way: set `HTTP_SET_HEADER("Authorization", "Bearer …")` once, or pass the header to `HTTP_REQUEST` per call.
- **Headers are checked** before they go out: a line break in the value is rejected. Otherwise arbitrary further headers could be smuggled in via a value from user input (header injection).
- **Timeout**: 10 seconds, changeable with `HTTP_TIMEOUT(sekunden)` (1..600). A blocking HTTP call in the render tick still freezes the UI — in a loop `HTTP_REQUEST_START`/`HTTP_READY`/`HTTP_RESULT` belong there instead (see [Requests in the background](#requests-in-the-background)).
- **No SSRF protection**: `HTTP_GET`/`HTTP_POST`/`HTTP_DOWNLOAD` accept any reachable URL, including `localhost`/private IPs/internal services — the module deliberately does not filter this (Drachenhauch programs run locally and are trusted). If you execute foreign/embedded DH code (multiplayer scripts, mod support), you should secure this yourself (e.g. check a URL allowlist before the call) — the runtime does not do it for you.
- **`HTTP_DOWNLOAD`** streams straight into the target file (no full in-memory buffer beforehand) — with very large downloads only the disk space matters, not the RAM usage. If the transfer breaks off in the middle of the body, the incomplete file is deleted automatically instead of leaving a truncated remainder behind.

## Complete example

See [examples/41_html.dh](../../examples/41_html.dh).

## In the native runtime (dhrt)

`html` runs natively with the Cargo feature `http` (HTTP via `ureq` incl. TLS/https). URL encode/decode and the HTML parser (`HTML_TEXT`/`HTML_FIND_ALL`/`HTML_GET_ATTR`) are ported as Rust scanners (functional; with broken HTML not necessarily byte-identical to Python's `html.parser`). The standard dev build includes `http`.
