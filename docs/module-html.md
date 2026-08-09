# Modul `html`

HTTP-Client + URL-Helpers + HTML-Parser, alles aus der Python-Standardbibliothek — **kein** externer pip-Install nötig.

```basic
IMPORT "html"
```

## Übersicht

| Bereich | Funktion | Rückgabe |
|---|---|---|
| HTTP | `HTTP_GET(url$)` | STRING (Response-Body) |
| HTTP | `HTTP_POST(url$, body$)` | STRING |
| HTTP | `HTTP_DOWNLOAD(url$, pfad$)` | INTEGER (Bytes) |
| HTTP | `HTTP_STATUS()` | INTEGER (z.B. 200, 404) |
| HTTP | `HTTP_HEADER(name$)` | STRING |
| URL | `URL_ENCODE(s$)` | STRING |
| URL | `URL_DECODE(s$)` | STRING |
| HTML | `HTML_TEXT(html$)` | STRING (Tags raus, Entities decodiert) |
| HTML | `HTML_FIND_ALL(html$, tag$)` | ARRAY OF STRING |
| HTML | `HTML_GET_ATTR(tag_html$, attr$)` | STRING |

## HTTP-Client

`HTTP_GET` / `HTTP_POST` machen einfache HTTP-Aufrufe und liefern den Response-Body als STRING (UTF-8 mit Replace-Strategie für Nicht-Decodierbares).

```basic
IMPORT "html"

DIM body AS STRING
body = HTTP_GET("https://api.example.com/scores")
PRINT "Status: ", HTTP_STATUS()
PRINT body
```

**Default-Timeout: 10 Sekunden.** Bei Timeout, Verbindungsfehler oder 4xx/5xx-Status wirft die Funktion `GBRuntimeError` mit Status-Code in der Meldung. Nach einem 4xx/5xx ist `HTTP_STATUS()` weiterhin lesbar — nützlich für `TRY/CATCH`-Pattern:

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

`HTTP_DOWNLOAD` schreibt direkt in eine Datei — sinnvoll für Binär-Daten (Bilder, Sounds), wo `HTTP_GET` mit UTF-8-Replace die Bytes verfälschen würde.

```basic
DIM bytes AS INTEGER
bytes = HTTP_DOWNLOAD("https://example.com/sprite.png", "assets/sprite.png")
PRINT "Heruntergeladen: ", bytes, " Bytes"
```

`HTTP_HEADER(name$)` liefert einen Response-Header der letzten Antwort. Header-Namen sind case-insensitive (`Content-Type` und `content-type` finden dasselbe).

## URL-Helpers

```basic
DIM q AS STRING
q = URL_ENCODE("hallo welt & co")    ' "hallo%20welt%20%26%20co"
DIM url AS STRING
url = "https://api.example.com/search?q=" + q
```

`URL_DECODE` macht das Gegenteil — `%XX`-Sequenzen werden zu Zeichen, `+` bleibt `+`. Beide Funktionen sind stdlib-`urllib.parse`-konform.

## HTML-Parser

`HTML_TEXT` strippt alle Tags und dekodiert HTML-Entities (`&amp;` → `&`, `&lt;` → `<`, …). Block-Level-Tags (`<p>`, `<div>`, `<li>`, …) erzeugen einen Newline. `<script>` und `<style>` werden komplett übersprungen.

```basic
DIM html AS STRING
html = HTTP_GET("https://example.com")
PRINT HTML_TEXT(html)
```

`HTML_FIND_ALL(html, tag)` liefert alle inneren HTML-Inhalte eines Tags als Array. Verschachtelte gleichnamige Tags werden korrekt gepaart (Stack-Tracking):

```basic
DIM links AS ARRAY OF STRING
links = HTML_FIND_ALL(html, "a")
DIM i AS INTEGER
FOR i = 0 TO LEN(links) - 1
    PRINT "Link-Inhalt: ", links[i]
NEXT
```

`HTML_GET_ATTR(tag_html, attr)` extrahiert ein Attribut aus dem ersten passenden Tag im übergebenen Snippet. Unterstützt doppelte/einfache Anführungszeichen und unquoted Werte (HTML5):

```basic
DIM all_a AS ARRAY OF STRING
all_a = HTML_FIND_ALL(html, "a")
' Fuer jeden <a>-Tag: dessen Outer-HTML brauchen wir - HTML_FIND_ALL liefert
' aber nur den INHALT zwischen den Tags. Wer das href$ braucht, scrapt mit
' Regex oder rebuilds den Tag - oder nutzt HTML_GET_ATTR auf einem schon
' bekannten Tag-Snippet:
DIM href AS STRING
href = HTML_GET_ATTR("<a href='https://example.com' title='go'>...</a>", "href")
PRINT href                    ' "https://example.com"
```

> **Limit**: `HTML_FIND_ALL` gibt den Inner-HTML zurück, nicht den Outer. Wenn du Attribute UND Inhalt brauchst, müsstest du den Inner separat parsen — oder direkt `HTTP_GET` + Regex/JSON nutzen wenn die API JSON liefert.

## Komplettbeispiel — News-Titel von einer Webseite ziehen

```basic
IMPORT "html"

DIM page AS STRING
page = HTTP_GET("https://example.com/news")

DIM titles AS ARRAY OF STRING
titles = HTML_FIND_ALL(page, "h2")

PRINT "Aktuelle Schlagzeilen:"
DIM i AS INTEGER
FOR i = 0 TO LEN(titles) - 1
    PRINT "  - ", HTML_TEXT(titles[i])    ' Inner-HTML zu reinem Text
NEXT
```

## JSON-API kombinieren

Die meisten modernen APIs liefern JSON, nicht HTML. Kombiniere `html` mit dem `json`-Modul:

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

## Sicherheits-/Privacy-Hinweise

- **User-Agent**: das Modul setzt einen eigenen User-Agent (`GameBasic/0.1 …`). Manche Server blocken Python's Default-`urllib`-UA — der eigene Header umgeht das.
- **HTTPS**: Zertifikats-Validierung läuft via Python-Default (System-CA-Store). Selbst-signierte Zertifikate werden abgelehnt — keine Bypass-Option im Modul (mit Absicht).
- **Cookies / Sessions** werden nicht persistent gespeichert — jeder Aufruf ist stateless. Für Login-geschützte APIs lieber Token-basierte Auth via Header einsetzen.
- **Timeout**: 10 Sekunden hart codiert. Game-Loop sollte HTTP-Calls nicht im Render-Tick machen — friert sonst die UI ein.
- **Kein SSRF-Schutz**: `HTTP_GET`/`HTTP_POST`/`HTTP_DOWNLOAD` akzeptieren jede erreichbare URL, inklusive `localhost`/privater IPs/interner Dienste — das Modul filtert das bewusst nicht (GameBasic-Programme laufen lokal vertrauenswürdig). Wer fremden/eingebetteten GB-Code ausführt (Multiplayer-Skripte, Mod-Support), sollte das selbst absichern (z.B. URL-Allowlist vor dem Aufruf prüfen) — die Runtime tut es nicht für dich.
- **`HTTP_DOWNLOAD`** streamt direkt in die Zieldatei (kein voller In-Memory-Puffer vorher) — bei sehr großen Downloads bleibt so nur der Festplattenplatz relevant, nicht der RAM-Verbrauch. Bricht der Transfer mitten im Body ab, wird die unvollständige Datei automatisch gelöscht statt einen abgeschnittenen Rest liegen zu lassen.

## Komplettes Beispiel

Siehe [examples/41_html.gb](../examples/41_html.gb).

## In der nativen Runtime (dhrt)

`html` laeuft nativ mit dem Cargo-Feature `http` (HTTP via `ureq` inkl. TLS/https). URL-Encode/Decode und der HTML-Parser (`HTML_TEXT`/`HTML_FIND_ALL`/`HTML_GET_ATTR`) sind als Rust-Scanner portiert (funktional; bei kaputtem HTML nicht zwingend byte-gleich zu Pythons `html.parser`). Der Standard-Dev-Build enthaelt `http`.
