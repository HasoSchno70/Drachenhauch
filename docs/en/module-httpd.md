# Module `httpd`

A small web server running in step with the main loop — for the control panel
of a controller, a sheet of measurements on the home network, a remote control
for your own program.

```basic
IMPORT "httpd"
```

## Why it exists

The first general-purpose roadmap had explicitly struck it: *“anyone who
really needs a service puts a ready-made server in front of it"*. For a public
service the sentence still holds. Measured against the hobbyist vision, it
looks different: with `mqtt`, `firmata`, `serial` and `net` on board, exactly
one building block was missing for “my heating controller has a small web
interface" — and it was the smallest of all, because `NET_TCP_LISTEN` already
lies beneath it.

## Overview

| Function | Purpose |
|---|---|
| `HTTPD_START(port[, bind$])` → HTTPD | start the server; `0` = let it choose a free port |
| `HTTPD_PORT(s)` → INTEGER | the port actually taken |
| `HTTPD_ACCEPT(s)` → BOOLEAN | is a request waiting? (returns immediately) |
| `HTTPD_METHOD$(s)` → STRING | `GET`, `POST`, … |
| `HTTPD_PATH$(s)` → STRING | the requested path, without `?…` |
| `HTTPD_QUERY$(s, name$)` → STRING | a value from `?a=1&b=2` (decoded) |
| `HTTPD_HEADER$(s, name$)` → STRING | a header (case does not matter) |
| `HTTPD_BODY$(s)` → STRING | the body (with `POST`) |
| `HTTPD_SEND(s, code, typ$, inhalt$)` | reply and close |
| `HTTPD_SET_HEADER(s, name$, wert$)` | a header for **all** following replies; an empty value removes it |
| `HTTPD_SEND_FILE(s, code, pfad$)` | serve a specific file |
| `HTTPD_SEND_DIR(s, ordner$[, start$])` → BOOLEAN | resolve the requested path **safely** inside the folder |
| `HTTPD_STOP(s)` | stop the server |

## The pattern

One call per round — the same pattern as `INPUT_UPDATE`, `TIMER_UPDATE` and
`MQTT_UPDATE`:

```basic
IMPORT "httpd"

DIM s AS HTTPD
DIM grad AS FLOAT
s = HTTPD_START(8080)
grad = 21.5

WHILE NOT QUITREQUESTED()
    IF HTTPD_ACCEPT(s) THEN
        IF HTTPD_PATH$(s) = "/setzen" THEN
            grad = VAL(HTTPD_QUERY$(s, "grad"))
            HTTPD_SEND(s, 200, "text/plain; charset=utf-8", "ok")
        ELSE
            HTTPD_SEND(s, 200, "text/html; charset=utf-8", _
                       "<h1>" + STR$(grad) + " Grad</h1>")
        END IF
    END IF
    SLEEP(10)
WEND
HTTPD_STOP(s)
```

`HTTPD_ACCEPT` **returns immediately** when nobody knocks — so the main loop
keeps running, even in a game at 60 frames per second.

## Serving files

```basic
IF HTTPD_ACCEPT(s) THEN
    IF NOT HTTPD_SEND_DIR(s, "web") THEN
        PRINT "nicht gefunden: " + HTTPD_PATH$(s)
    END IF
END IF
```

`HTTPD_SEND_DIR` takes the requested path, resolves it **inside** the folder,
guesses the content type from the extension and replies. A path to a folder
gets `index.html` (or whatever is given as `start$`); a missing file becomes
`404` and returns `FALSE` — it is a server's everyday business and must not
stop the program.

**Use `HTTPD_SEND_DIR`, not handiwork.** The obvious line

```basic
HTTPD_SEND_FILE(s, 200, "web" + HTTPD_PATH$(s))     ' NOT like this
```

can be led out of the folder with `GET /../../geheim.txt` — and then serves
any file on the machine. `HTTPD_SEND_DIR` rejects every path component `..`
(plus colons and backslashes, which on Windows could mean a drive or an
alternate data stream) and replies with `403`. It is the same lesson as with
the zip-slip check in `ZIP_EXTRACT`.

`HTTPD_SEND_FILE` remains for the case where **the program** decides which
file goes out — no foreign path is involved there.

## Custom headers

`HTTPD_SET_HEADER` applies to every further reply of the server, not just the
next one — it is needed for headers that must be present everywhere, such as
the CORS permission without which a game in the browser cannot reach the
server:

```basic
IMPORT "httpd"
DIM s AS HTTPD
s = HTTPD_START(8787)
HTTPD_SET_HEADER(s, "Access-Control-Allow-Origin", "*")
```

Setting a header only on the next reply would mean that every place in the
program that replies has to remember it — and exactly one forgets. A value
with a line break is an error (it would otherwise write a second, freely
chosen header into the reply), and `Content-Type`, `Content-Length` and
`Connection` are set by `HTTPD_SEND` itself.

## Limits — and why

* **No HTTPS.** If you need encryption, put a reverse proxy in front. A TLS
  server here would mean certificate management inside the program.
* **No keep-alive**, one connection per request. For a control panel that
  costs nothing and saves half the state management of a real server.
* **One request per `HTTPD_ACCEPT`**, one after the other. Two browsers at the
  same time are served, just in turn.
* **The body is read entirely into memory**, upper limit 8 MiB. The server is
  not suited for uploading large files.
* **A started request may take 50 ms**, then the connection is dropped. Every
  millisecond here is one in which the program does not draw; a browser sends
  its request in one piece anyway.
* **No cookie, session or login vocabulary.** If you need a login, read
  `HTTPD_HEADER$(s, "Authorization")` yourself and compare with
  `SECURE_EQUALS`.

All together this means: **on the home network yes, on the open network no.**
That way round, the sentence from the old roadmap is right.

## While developing

`HTTPD_START(0)` lets the operating system choose a free port — handy when
several programs run at the same time. The chosen port is best reported with
`EPRINT`: it appears at once, even when the output goes to a file. `PRINT`
appears at once in a terminal too, but redirected to a file it is collected --
and a waiting server writes nothing more that would push the rest out.

```basic
s = HTTPD_START(0)
EPRINT("Läuft auf Port " + STR$(HTTPD_PORT(s)))
```

Example: [examples/173_webserver.dh](../../examples/173_webserver.dh).
