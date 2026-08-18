# Modul `html`

HTTP-Client + URL-Helpers + HTML-Parser. Alles nativ in `dhrt` (Rust, HTTP über
`ureq`) — **kein** pip-Install, kein Python zur Laufzeit.

```basic
IMPORT "html"
```

## Übersicht

| Bereich | Funktion | Rückgabe |
|---|---|---|
| HTTP | `HTTP_REQUEST(methode$, url$ [, rumpf [, kopfzeilen]])` | STRING (Antwort-Rumpf) |
| HTTP | `HTTP_GET(url$)` | STRING (Response-Body) |
| HTTP | `HTTP_POST(url$, body$)` | STRING |
| HTTP | `HTTP_DOWNLOAD(url$, pfad$)` | INTEGER (Bytes) |
| HTTP | `HTTP_STATUS()` | INTEGER (z.B. 200, 404) |
| HTTP | `HTTP_HEADER(name$)` | STRING (Kopfzeile der Antwort) |
| HTTP | `HTTP_BYTES()` | BUFFER (roher Rumpf der letzten Antwort) |
| HTTP | `HTTP_SET_HEADER(name$, wert$)` | — (gilt für alle folgenden Aufrufe) |
| HTTP | `HTTP_CLEAR_HEADERS()` | — |
| HTTP | `HTTP_TIMEOUT(sekunden)` | — |
| HTTP (Hintergrund) | `HTTP_REQUEST_START(methode$, url$ [, rumpf [, kopfzeilen]])` | INTEGER (Abruf-Nummer) |
| HTTP (Hintergrund) | `HTTP_GET_START(url$)` | INTEGER (Abruf-Nummer) |
| HTTP (Hintergrund) | `HTTP_READY(abruf)` | BOOLEAN |
| HTTP (Hintergrund) | `HTTP_RESULT(abruf)` | STRING (Response-Body) |
| HTTP (Hintergrund) | `HTTP_CANCEL(abruf)` | — |
| HTTP (Hintergrund) | `HTTP_PENDING()` | INTEGER (offene Abrufe) |
| HTTP (Hintergrund) | `HTTP_URL$(abruf)` | STRING |
| URL | `URL_ENCODE(s$)` | STRING |
| URL | `URL_DECODE(s$)` | STRING |
| HTML | `HTML_TEXT(html$)` | STRING (Tags raus, Entities decodiert) |
| HTML | `HTML_FIND_ALL(html$, tag$)` | ARRAY OF STRING |
| HTML | `HTML_GET_ATTR(tag_html$, attr$)` | STRING |

## HTTP_REQUEST — für alles, was über GET und POST hinausgeht

Echte Schnittstellen wollen mehr als GET und POST: einen Anmelde-Token in einer
Kopfzeile, ein `PUT` zum Ändern, ein `DELETE` zum Löschen, JSON als Inhaltstyp.
Dafür gibt es **einen** Befehl:

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

`methode$` ist eine von `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`,
`OPTIONS` (Groß-/Kleinschreibung egal). Etwas anderes ist ein **Fehler** —
ein `"GTE"` käme sonst als verwirrende Server-Antwort zurück statt als
Meldung an der Zeile, in der der Tippfehler steht.

`rumpf` darf ein `STRING` oder ein `BUFFER` sein (siehe
[Bytes](builtins-core.md#bytes-buffer)) — damit lässt sich auch ein Bild oder
eine Zip-Datei hochladen. Weglassen oder `""` heißt: kein Rumpf.

`kopfzeilen` ist eine `MAP OF STRING`. Sie überschreibt gleichnamige Vorgaben.

> **`HTTP_REQUEST` rät keinen `Content-Type`.** Welchen Typ ein Rumpf hat, weiß
> nur der Aufrufer; ein falsch geratenes `application/json` wäre schlimmer als
> gar keines. Wer JSON schickt, setzt die Kopfzeile also selbst. (`HTTP_POST`
> behält dagegen seine alte Vorgabe `application/x-www-form-urlencoded` — daran
> hängen bestehende Programme.)

### Ein Token einmal setzen

Wenn alle Aufrufe dieselbe Anmeldung brauchen, muss die Kopfzeile nicht an
jeden einzelnen:

```basic
HTTP_SET_HEADER("Authorization", "Bearer " + token)

PRINT HTTP_REQUEST("GET", basis + "/profil")      ' Token ist dabei
PRINT HTTP_GET(basis + "/nachrichten")            ' auch hier

HTTP_CLEAR_HEADERS()                              ' wieder abmelden
```

Das gilt für **alle** folgenden HTTP-Aufrufe des Moduls, auch für `HTTP_GET`
und die Hintergrund-Varianten. Eine Kopfzeile, die beim einzelnen Aufruf
mitgegeben wird, gewinnt gegen die dauerhafte. Zweimal derselbe Name ersetzt,
statt sich zu häufen — sonst gingen beide raus und der Server entschiede.

### Rohe Bytes als Antwort

Der Rückgabewert ist Text: nicht dekodierbare Bytes werden dabei ersetzt. Bei
einem Bild oder einer Zip-Datei bliebe davon nichts Brauchbares. `HTTP_BYTES()`
liefert deshalb den **rohen** Rumpf der letzten Antwort:

```basic
DIM egal AS STRING
DIM bild AS BUFFER
egal = HTTP_REQUEST("GET", "https://example.com/logo.png")
bild = HTTP_BYTES()
WRITEALL_BYTES("logo.png", bild)
```

`HTTP_BYTES()` gehört — wie `HTTP_STATUS()` und `HTTP_HEADER()` — zur *letzten*
Antwort und funktioniert nach jedem HTTP-Aufruf, auch nach `HTTP_GET`. Nach
einem fehlgeschlagenen Aufruf ist es leer (und nicht etwa noch die vorige
Antwort). Wer eine große Datei nur speichern will, ist mit `HTTP_DOWNLOAD`
weiterhin besser bedient — das streamt direkt auf die Platte.

### Zeitgrenze

```basic
HTTP_TIMEOUT(30)      ' Sekunden, 1..600; Vorgabe ist 10
```

Gilt für alle folgenden Aufrufe.

## HTTP-Client

`HTTP_GET` / `HTTP_POST` sind die Kurzformen für den häufigen Fall und liefern den Response-Body als STRING (UTF-8 mit Replace-Strategie für Nicht-Decodierbares).

```basic
IMPORT "html"

DIM body AS STRING
body = HTTP_GET("https://api.example.com/scores")
PRINT "Status: ", HTTP_STATUS()
PRINT body
```

**Default-Timeout: 10 Sekunden.** Bei Timeout, Verbindungsfehler oder 4xx/5xx-Status wirft die Funktion `DHRuntimeError` mit Status-Code in der Meldung. Nach einem 4xx/5xx ist `HTTP_STATUS()` weiterhin lesbar — nützlich für `TRY/CATCH`-Pattern:

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

## Abrufe im Hintergrund

`HTTP_GET` hält das ganze Programm an, bis die Antwort da ist. In einem Fenster heißt das: keine Maus, keine Taste, kein Neuzeichnen. Ein mittlerer JSON-Abruf dauert ~200 ms — rund zwölf ausgefallene Bilder, und bei schlechter Verbindung wartet das Fenster bis zum Timeout von 10 Sekunden.

Für alles, was in einer Schleife läuft, gibt es darum denselben Abruf zum Nachsehen statt zum Warten:

| Funktion | Wirkung |
|---|---|
| `HTTP_REQUEST_START(methode$, url$ [, rumpf [, kopfzeilen]])` → INTEGER | startet **jede** Anfrage im Hintergrund |
| `HTTP_GET_START(url$)` → INTEGER | Kurzform davon für GET |
| `HTTP_READY(abruf)` → BOOLEAN | ist die Antwort da? (fragt nach, wartet nicht) |
| `HTTP_RESULT(abruf)` → STRING | holt den Body ab und gibt den Platz frei |
| `HTTP_CANCEL(abruf)` | Abruf verwerfen (unbekannte Nummer = No-Op) |
| `HTTP_PENDING()` → INTEGER | wie viele Abrufe noch offen sind |
| `HTTP_URL$(abruf)` → STRING | die URL eines laufenden Abrufs |

Das Muster ist dasselbe wie bei `INPUT_UPDATE()`/`TIMER_UPDATE()`: einmal pro Frame nachsehen.

`HTTP_RESULT` setzt `HTTP_STATUS()`, `HTTP_HEADER()` und `HTTP_BYTES()` genauso
wie ein blockierender Aufruf — beim **Abholen**, nicht beim Starten. Bei
mehreren gleichzeitigen Abrufen gehören diese Werte also immer zu dem, den man
zuletzt abgeholt hat.

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

Nach `HTTP_RESULT` stehen `HTTP_STATUS()` und `HTTP_HEADER()` genau wie nach `HTTP_GET` bereit.

**Details:**

- **Fehler kommen beim Abholen.** Ein 404, ein Timeout oder ein Verbindungsfehler lässt `HTTP_GET_START` durchgehen und wirft erst bei `HTTP_RESULT` — dort, wo das Programm damit umgehen kann (`TRY`/`CATCH` um das Abholen).
- **`HTTP_READY` darf beliebig oft gefragt werden.** Nach dem ersten `TRUE` bleibt die Antwort liegen, bis sie abgeholt wird. Erst `HTTP_RESULT` verbraucht sie.
- **`HTTP_RESULT` vor `HTTP_READY`** bricht mit Klartext ab statt zu blockieren — sonst wäre der Vorteil wieder verspielt.
- **Mehrere Abrufe laufen wirklich gleichzeitig.** Zwei Anfragen zu je 300 ms sind nach ~300 ms fertig, nicht nach 600.
- **Nummern bleiben stabil** (Tombstones wie bei `timer`): eine abgeholte oder abgebrochene Nummer wird nicht neu vergeben. Ein Programm kann eine alte Nummer also nicht versehentlich auf einen fremden Abruf beziehen.
- **Abbrechen stoppt den Abruf nicht mitten im Netz** — er läuft zu Ende, sein Ergebnis wird verworfen. Das Programm wartet dabei auf nichts.
- **Nur GET.** Zum Daten-Holen reicht das, und jede weitere Form verdoppelt die Zustände, die ein Programm im Blick behalten muss. Für POST/Download in einer Schleife: seltener Aufruf an einer Stelle, an der ein kurzer Stillstand nicht stört.

## URL-Helpers

```basic
DIM q AS STRING
q = URL_ENCODE("hallo welt & co")    ' "hallo%20welt%20%26%20co"
DIM url AS STRING
url = "https://api.example.com/search?q=" + q
```

`URL_DECODE` macht das Gegenteil — `%XX`-Sequenzen werden zu Zeichen, `+` bleibt `+`. Beide sind in Rust geschrieben, liefern aber dasselbe wie Pythons `urllib.parse.quote`/`unquote` (nachgemessen, nicht bloss beabsichtigt).

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

- **User-Agent**: das Modul setzt einen eigenen User-Agent (`Drachenhauch/0.1 …`). Manche Server blocken Python's Default-`urllib`-UA — der eigene Header umgeht das.
- **HTTPS**: Zertifikats-Validierung gegen den System-CA-Store. Selbst-signierte Zertifikate werden abgelehnt — keine Bypass-Option im Modul (mit Absicht).
- **Cookies / Sessions** werden nicht gespeichert — jeder Aufruf steht für sich. Für angemeldete APIs ist Token-Auth der Weg: `HTTP_SET_HEADER("Authorization", "Bearer …")` einmal setzen, oder die Kopfzeile pro Aufruf an `HTTP_REQUEST` geben.
- **Kopfzeilen werden geprüft**, bevor sie rausgehen: ein Zeilenumbruch im Wert wird abgelehnt. Sonst ließen sich über einen Wert aus einer Benutzereingabe beliebige weitere Kopfzeilen einschmuggeln (Header-Injection).
- **Timeout**: 10 Sekunden, änderbar mit `HTTP_TIMEOUT(sekunden)` (1..600). Ein blockierender HTTP-Aufruf im Render-Tick friert die UI trotzdem ein — in einer Schleife gehören darum `HTTP_REQUEST_START`/`HTTP_READY`/`HTTP_RESULT` hin (siehe [Abrufe im Hintergrund](#abrufe-im-hintergrund)).
- **Kein SSRF-Schutz**: `HTTP_GET`/`HTTP_POST`/`HTTP_DOWNLOAD` akzeptieren jede erreichbare URL, inklusive `localhost`/privater IPs/interner Dienste — das Modul filtert das bewusst nicht (Drachenhauch-Programme laufen lokal vertrauenswürdig). Wer fremden/eingebetteten GB-Code ausführt (Multiplayer-Skripte, Mod-Support), sollte das selbst absichern (z.B. URL-Allowlist vor dem Aufruf prüfen) — die Runtime tut es nicht für dich.
- **`HTTP_DOWNLOAD`** streamt direkt in die Zieldatei (kein voller In-Memory-Puffer vorher) — bei sehr großen Downloads bleibt so nur der Festplattenplatz relevant, nicht der RAM-Verbrauch. Bricht der Transfer mitten im Body ab, wird die unvollständige Datei automatisch gelöscht statt einen abgeschnittenen Rest liegen zu lassen.

## Komplettes Beispiel

Siehe [examples/41_html.dh](../examples/41_html.dh).

## In der nativen Runtime (dhrt)

`html` laeuft nativ mit dem Cargo-Feature `http` (HTTP via `ureq` inkl. TLS/https). URL-Encode/Decode und der HTML-Parser (`HTML_TEXT`/`HTML_FIND_ALL`/`HTML_GET_ATTR`) sind als Rust-Scanner portiert (funktional; bei kaputtem HTML nicht zwingend byte-gleich zu Pythons `html.parser`). Der Standard-Dev-Build enthaelt `http`.
