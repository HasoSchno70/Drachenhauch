# Allzweck-Roadmap (Audit 2026-08-17)

Drachenhauch ist als **Game Basic** gestartet — ein Dialekt zum Schreiben von
Spielen. Diese Roadmap beantwortet eine andere Frage: **was fehlt, damit man
damit alles schreiben kann**, was man schreiben möchte — Werkzeuge, Skripte,
Datenbank-Anwendungen, Dienste, Auswertungen.

**Der Befund vorweg:** die Lücken sitzen *nicht* in der Syntax. Als Sprache ist
Drachenhauch längst ausdrucksstärker als das, wofür es gedacht war. Sie sitzen
an zwei Stellen:

1. **Das Programm kann nicht mit seinem Betriebssystem reden** — keine
   Argumente, keine Umgebung, kein Prozessaufruf, kein Rückgabewert. Damit
   fällt die gesamte Klasse „Werkzeug in einer Kette" weg.
2. **Großer Code lässt sich nicht ordnen** — ein flacher Namensraum, textuelles
   `IMPORT`, keine getrennte Übersetzung.

Alles Übrige in dieser Liste ist Bibliothek, und Bibliothek ist Fleißarbeit.

## Was schon trägt (damit die Lücken einordbar bleiben)

Typen mit Pascal-Strenge, Klassen mit Vererbung, `PROPERTY`, Operator-Overloading,
`STATIC CONST`, Comprehensions (Liste/Dict/Set), Tupel + Destructuring, f-Strings,
Coroutines, `BYREF`, Named Arguments, `FUNCREF`, Variadics, `WITH`, Slicing.
Dazu SQLite, JSON, Regex, HTTP-Client, Diagramme, eine 22-Widget-GUI mit
Tabellen und Formular-Designer, Sprachserver, Debugger, Profiler, Export als
eigenständige `.exe`.

Das Tippspiel (`buch-tippspiel/`) ist der Beleg: echte Datenbank-Anwendung mit
Netz, Fenstern, Reitern und Saison-Verwaltung — vollständig in Drachenhauch.

## Umsetzungs-Checkliste pro Befehl (Stand Stufe B — dhrt ist die einzige Runtime)

- [ ] `rust/drachenhauch_runtime/src/builtins.rs` (pur) bzw. `vm.rs` `try_*`
      (braucht VM-/Fenster-State). Arity + Typen im Wrapper prüfen,
      Fehlermeldung im gewohnten Wortlaut (`"NAME: erwartet …"`).
- [ ] `drachenhauch/editor_qt/builtin_index.json` — Name/kind/**Signatur**/Modul.
      Die Signatur muss stimmen, `dhrt --check` leitet daraus die erlaubte
      Argumentzahl ab; eine zu enge Signatur erzeugt Falsch-Alarme in fremdem Code.
- [ ] Golden-Test in `tests/` (`assert run_gb(src) == erwartet`), plus
      Rust-`#[test]` für reine Rechen-Anteile.
- [ ] Hover-Doku in `editor_qt/builtin_docs.py`.
- [ ] Bei neuem Keyword: `vscode-drachenhauch/build_grammar.py` neu generieren.
- [ ] Bei neuem Modul: Name in `preprocess.rs` `MODULES` **und**
      `drachenhauch/modules/__init__.py` `KNOWN_MODULES` (synchron halten),
      dazu `docs/module-<name>.md` + Zeile in `docs/README.md`.

---

## WP A — Betriebssystem-Anbindung (✅ ERLEDIGT 2026-08-17)

**Das war die größte Lücke.** Verifiziert gegen `builtins.rs`/`vm.rs`: die
einzigen `std::env::args()` im Quelltext waren die von `dhrt` selbst
(`main.rs`). Ein Drachenhauch-Programm konnte seine eigenen Argumente nicht
lesen.

Folge: eine exportierte `.exe` nahm keine Argumente entgegen, konnte sich in
keine Werkzeugkette einreihen, kein anderes Programm aufrufen und ihrem
Aufrufer nicht sagen, ob es geklappt hat. Genau das ist der Kern von
„Werkzeug, Skript, Automatisierung".

- [x] `ARGC()` / `ARG$(n)` — Kommandozeilenargumente des *Programms*, nicht die
      von `dhrt`. Trennung über die Konvention **`--`**: `dhrt run datei.dh --
      a b` gibt `a b` ans Programm, ohne `--` bekommt es nichts. Im
      **Bundle-Modus** (exportierte `.exe`) gibt es keine Runtime-Argumente,
      dort gehören alle dem Programm — ohne `--`. `ARG$` außerhalb liefert
      einen Leerstring statt eines Fehlers (Argumente sind Benutzereingabe).
- [x] `GETENV$(name$ [, vorgabe$])`, `SETENV(name$, wert$)` — `SETENV` wirkt auf
      diesen Prozess **und seine Kinder** (`SHELL`), nicht auf die Konsole des
      Aufrufers.
- [x] `EXIT([code])` — geordnetes Ende mit Rückgabewert. Läuft über denselben
      Sentinel-Kanal wie die beiden Stop-Signale (`vm.rs`), wird darum von
      `TRY`/`CATCH` **nicht** gefangen; entscheidend ist wie dort das Flag und
      nicht der Fehlertext, `THROW "__EXIT__"` bleibt ein normaler Fehler.
      Werte außerhalb 0..255 sind ein Fehler statt still gekappt zu werden.
- [x] `SHELL(programm$, ...)` → Rückgabewert; `SHELL_OUT$(...)` → stdout
      einsammeln. Argumente werden **einzeln** übergeben, nicht zu einer
      Kommandozeile zusammengeklebt — kein Quoting zu lernen, kein zerfallender
      Dateiname mit Leerzeichen. `SHELL_OUT$` liefert nur stdout; stderr des
      Kindes bleibt stderr, sonst mischten sich Fehlermeldungen in die
      Nutzdaten.
- [x] `EPRINT(text)` — Ausgabe nach stderr, damit `PRINT` als Nutzdaten
      durchgereicht werden kann. Builtin, also mit Klammern (anders als
      `PRINT`).
- [x] `CWD$()` / `CHDIR(pfad$)`. Der Stolperstein ist dokumentiert
      (`docs/builtins-core.md`): `dhrt` wechselt beim Start ins Verzeichnis der
      `.dh`-Datei, die `.exe` ins Exe-Verzeichnis — ein relativer Pfad vom
      Aufrufer ist also **nicht** relativ zu `CWD$()`.

**Umsetzung:** zustandsfrei in `builtins.rs`
(`ARGC`/`ARG$`/`GETENV$`/`SETENV`/`CWD$`/`CHDIR`), VM-behaftet in `vm.rs`
`try_os` (`EXIT`/`EPRINT`/`SHELL`/`SHELL_OUT$`). Der gemeinsame Grund für
`try_os`: **`PRINT` wird gepuffert** und erst am Programmende geschrieben — wer
daneben auf stderr schreibt oder ein Kindprogramm aufs selbe Terminal lässt,
sähe die Ausgaben sonst in falscher Reihenfolge. Diese vier flushen den Puffer
darum zuerst (geteilter Helfer `flush_out`, den sich `flush_and_prompt` für
`INPUT` jetzt mit ihnen teilt). Argument-Aufteilung in `main.rs`
(`setze_programm_args`, bewusst an den Stellen, die wirklich ein Programm
starten — der Bundle-Zweig setzt seine eigenen). Profiler und Debugger melden
`EXIT` nicht mehr als Fehler.

Tests: `tests/test_os_builtins.py` (28), neue Fixtures `run_gb_roh` (liefert
Rückgabewert + stderr, reicht Argumente durch) und `dhrt_pfad` in
`conftest.py`. Beispiel: `examples/161_werkzeug.dh` — ein `wc`-artiges Werkzeug
ohne Fenster, mit Argumenten, stderr-Meldungen und Rückgabewerten. Doku:
`docs/builtins-core.md`, Abschnitt „Betriebssystem".

## WP B — Bytes und Binärdateien (✅ ERLEDIGT 2026-08-17)

Es gab keinen Byte-Typ. Dateien waren Text (`READALL`/`WRITEALL`/`READLINES`),
`STRING` ist UTF-8 und taugt darum nicht als Byte-Behälter, `SEEK` fehlte.
Damit gingen nicht: eigene Dateiformate, Protokolle mitlesen, ZIP/PNG anfassen,
Serial-Geräte jenseits von Text, Bilder aus dem Netz weiterverarbeiten.

WP B ist Voraussetzung für WP C (Rumpf als Bytes), WP D (Prüfsummen über
Dateien) und ZIP in WP J.

- [x] **Kern-Typ `BUFFER`** (kein `IMPORT`, wie `FILE`): `BUFFER_NEW`,
      `BUFFER_LEN`, `BUFFER_GET/SET`, `BUFFER_FILL`, `BUFFER_RESIZE`,
      `BUFFER_SLICE`, `BUFFER_CONCAT`, `BUFFER_INDEXOF`. Referenz-Typ wie
      `ARRAY`. Index streng, Slice klemmt — dieselbe Regel wie bei Arrays.
- [x] Umwandlung: `BUFFER_FROM_STRING`/`BUFFER_TO_STRING$` (**streng**: kaputtes
      UTF-8 ist ein Fehler, kein stilles Ersatzzeichen), `BUFFER_TO_HEX$`/
      `BUFFER_FROM_HEX` (Leerzeichen erlaubt), `BUFFER_TO_BASE64$`/
      `BUFFER_FROM_BASE64` (rohe Bytes, anders als `BASE64_DECODE`).
- [x] Datei: `READ_BYTES(f, n)` → BUFFER, `WRITE_BYTES(f, buf)`, `SEEK(f, pos)`,
      `TELL(f)`, pfadbasiert `READALL_BYTES`/`WRITEALL_BYTES`.
- [x] Zahlen packen: `BUFFER_GET_/SET_` für `I16`/`U16`/`I32`/`U32`/`I64`/`F32`/
      `F64`, Byte-Reihenfolge als optionales `"le"`/`"be"`.

**Zwei bewusste Abweichungen von der ursprünglichen Planung oben:**

1. **Keine `"rb"`/`"wb"`-Modi.** Rust übersetzt im Textmodus nichts (kein
   CRLF-Gefummel wie in C) — Drachenhauch-Dateien sind ohnehin byte-genau.
   Getrennte Modi hätten also einen Unterschied vorgegaukelt, den es nicht
   gibt. `READ_BYTES`/`WRITE_BYTES`/`SEEK` arbeiten auf denselben Handles wie
   `READLINE`/`WRITELINE`. Ein Test belegt, dass alle 256 Bytewerte
   unverändert durch eine Datei gehen. **Was noch fehlt:** ein Modus, der
   gleichzeitig liest *und* schreibt (`"r+"`) — heute geht Ändern nur über
   `READALL_BYTES` → Puffer ändern → `WRITEALL_BYTES`.
2. **Byte-Reihenfolge ist optional mit Vorgabe `"le"`**, nicht Pflicht wie oben
   geplant. Wer selbst schreibt und wieder liest, benutzt auf beiden Seiten
   dieselbe Vorgabe und ist immer richtig; die Angabe braucht nur, wer ein
   fremdes Format bedient (PNG/ZIP/Netz sind big-endian) — und wer das tut,
   denkt ohnehin darüber nach. Pflicht für alle hätte den häufigen Fall
   besteuert, um dem seltenen zu helfen.
   Dazu **`U16`/`U32` zusätzlich** zur geplanten Liste: ohne sie lassen sich
   PNG-Blocklängen und WAV-Größen nicht lesen, ein `u32` passt nicht in `I32`.

**Umsetzung:** `Value::Buffer(Rc<RefCell<Vec<u8>>>)` in `value.rs`, Builtins in
`builtins.rs` (alle zustandsfrei), `"buffer"` in `is_value_type` (`compiler.rs`)
— bewusst **kein Lexer-Keyword**, sonst wäre `DIM buffer AS INTEGER` in
bestehendem Code plötzlich ein Fehler. `PRINT puffer` zeigt nur die Länge, nicht
den Inhalt.

Tests: `tests/test_buffer.py` (48). Beispiel: `examples/162_binaerdatei.dh` —
liest Breite/Höhe und die Blockliste aus einer echten PNG-Datei ohne
Bildbibliothek und schreibt/liest ein eigenes Binärformat. Doku:
`docs/builtins-core.md`, Abschnitt „Bytes (BUFFER)".

## WP C — HTTP für echte Dienste (✅ ERLEDIGT 2026-08-17)

Vorher: nur `HTTP_GET(url$)`, `HTTP_POST(url$, body$)`, `HTTP_DOWNLOAD`. **Keine
eigenen Request-Header**, kein PUT/PATCH/DELETE, Timeout fest bei 10 Sekunden,
keine Sitzungen. Damit war praktisch jede angemeldete REST-Schnittstelle außer
Reichweite.

`docs/module-html.md` riet im Abschnitt „Grenzen" selbst zu „Token-basierte Auth
via Header" — eine API, die es nicht gab. Das war der klarste Beleg, dass hier
etwas fehlte.

- [x] `HTTP_REQUEST(methode$, url$ [, rumpf [, kopfzeilen]])` als *eine*
      allgemeine Funktion; `HTTP_GET`/`HTTP_POST` bleiben als Kurzform.
      Methode gegen eine feste Liste geprüft (GET/POST/PUT/PATCH/DELETE/HEAD/
      OPTIONS) — ein `"GTE"` soll als Fehler an seiner Zeile auffallen und
      nicht als merkwürdige Server-Antwort.
- [x] `HTTP_TIMEOUT(sekunden)` (1..600).
- [x] Rumpf als `BUFFER` senden **und** Antwort als `BUFFER` empfangen
      (`HTTP_BYTES()`, gilt für die letzte Antwort wie `HTTP_STATUS()` —
      also auch nach `HTTP_GET`, und nach einem Fehler leer).
- [x] Hintergrund-Variante für alle Methoden: `HTTP_REQUEST_START`.
      `HTTP_GET_START` bleibt die Kurzform.
- [x] Doku nachgezogen: „alles aus der Python-Standardbibliothek" korrigiert
      (es ist Rust/`ureq`), und der Header-Rat in „Grenzen" stimmt jetzt.

**Zwei Zugaben über die Planung hinaus:**

1. **`HTTP_SET_HEADER`/`HTTP_CLEAR_HEADERS`** — eine Kopfzeile, die *alle*
   folgenden Aufrufe mitschicken. Ohne das müsste die Anmeldung an jeden
   einzelnen Aufruf gehängt werden, und genau das ist der Fall, um den es bei
   „Token-Auth" geht. Pro Aufruf übergebene Kopfzeilen gewinnen; gleicher Name
   ersetzt statt sich zu häufen (sonst gingen beide raus und der Server
   entschiede).
2. **Kopfzeilen werden geprüft** (`pruefe_header`): ein CR/LF im Wert wird
   abgelehnt. Kommt der Wert aus einer Benutzereingabe — bei einem Token oder
   Dateinamen schnell der Fall — hängte er sonst beliebige weitere Kopfzeilen
   an die Anfrage (Header-Injection). Das ist eine Lücke, keine Kosmetik.

**Bewusst nicht getan:** `HTTP_REQUEST` rät **keinen** `Content-Type`. Welchen
Typ ein Rumpf hat, weiß nur der Aufrufer; ein falsch geratenes
`application/json` wäre schlimmer als gar keines. `HTTP_POST` behält dagegen
seine alte Formular-Vorgabe — daran hängen bestehende Programme (Test dafür).

**Umsetzung:** `html.rs` hat jetzt genau **einen** Weg, eine Anfrage zu bauen
(`Anfrage` + `http_request`); die früheren `http_get`/`http_post`-Kurzformen
dort sind ersatzlos weg, weil die VM ihre Anfrage selbst zusammenstellt (sie
muss die dauerhaften Kopfzeilen und die Zeitgrenze einsetzen) — zwei Bauwege
wären genau der Ort, an dem eines von beidem irgendwann fehlt. In `vm.rs`
ersetzt der geteilte Helfer `http_antwort` den vorher viermal wortgleichen
Block; mit `HTTP_BYTES` kam eine fünfte Sache dazu, die jeder Pfad tun muss.
`Abrufe::start` nimmt jetzt eine `Anfrage` statt nur einer URL.

Tests: `tests/test_http_request.py` (33, gegen einen lokalen Spiegel-Server,
der zurückmeldet was ankam — am Rückgabewert allein sähe man nicht, ob eine
Kopfzeile rausging). Beispiel: `examples/163_rest_api.dh` gegen httpbin.org.
Doku: `docs/module-html.md`.

## WP D — Identität und Prüfsummen (✅ ERLEDIGT 2026-08-17)

Vorhanden waren `CRC32`, `HASH` (FNV-1a) und Base64 — Prüfsummen fürs
Wiedererkennen, nichts für Vertrauen. Es fehlte alles, was eine Anmeldung, eine
Signatur oder eine stabile eindeutige Nummer braucht.

- [x] `SHA256$`, `SHA1$`, `MD5$` — je über STRING und über BUFFER
- [x] `HMAC_SHA256$(schluessel, daten)` — Web-APIs, Webhooks
- [x] `UUID4$()`
- [x] `RANDOM_BYTES(n)` → BUFFER, aus der Betriebssystem-Quelle (**nicht** aus
      dem Spiel-PRNG — der ist gesät und vorhersagbar, das ist hier ein Fehler
      und kein Detail). Ein Test hält genau das fest: nach `RANDOMIZE(1)`
      wiederholt sich `RND`, `RANDOM_BYTES` aber nicht.

**Zwei Zugaben über die Planung hinaus:**

1. **`SECURE_EQUALS(a, b)`** — Vergleich in konstanter Zeit. Ein gewöhnliches
   `=` bricht beim ersten ungleichen Zeichen ab; wer eine Signatur raten will,
   misst die Zeit und hat sie zeichenweise. Ohne diesen Vergleich wäre
   `HMAC_SHA256$` genau an der Stelle stumpf, für die es da ist — die
   naheliegende Benutzung wäre die falsche.
2. **`SHA256_FILE$`/`SHA1_FILE$`/`MD5_FILE$`** — blockweise über eine Datei.
   Die Alternative wäre `SHA256$(READALL_BYTES(...))`, und das scheitert genau
   dort, wo man eine Dateiprüfsumme braucht: bei großen Dateien.

**Umsetzung:** RustCrypto-Crates (`sha2`, `sha1`, `md-5`, `hmac`) plus `uuid`
und `getrandom` — bewusst **keine Handarbeit**: eine selbstgeschriebene
Hash-Funktion ist die Art Code, die auf den ersten Blick stimmt und in einem
Randfall still etwas anderes liefert, und hier hängt eine Signaturprüfung
daran. `sha1`, `digest`, `uuid` und `getrandom` lagen ohnehin schon transitiv
in der Lockfile (über `ureq`/`rustls`). Alles **ungated** — ein
Konsolenprogramm, das eine Prüfsumme bildet, soll nicht das `http`-Feature
brauchen.

Tests: `tests/test_krypto.py` (33), gegen die Vektoren aus den Normen
(FIPS 180-4, RFC 1321, RFC 4231) bzw. gegen Pythons `hashlib`/`hmac` — eine
Prüfsumme, die nur mit sich selbst übereinstimmt, wäre wertlos. Beispiel:
`examples/164_signatur.dh` (Webhook prüfen, Token würfeln, Datei-Prüfsumme).
Doku: `docs/builtins-core.md`, Abschnitt „Prüfsummen und Identität".

## WP E — Prüfen und Melden (✅ ERLEDIGT 2026-08-17)

`buch-tippspiel/code/tippspiel_pruefung.dh` war ein von Hand geschriebener
Ersatz für einen Testrahmen. Dass es ihn gab, war der beste Beleg, dass er
fehlte — und er zahlt sich bei jedem weiteren WP hier selbst zurück.

Der Entwurf ist an dieser Datei abgelesen: 119 Aufrufe der Form
`pruefe(was$, ist, soll)`, in **drei** fast wortgleichen Fassungen nur wegen
der Typen (`pruefe`/`pruefeJa`/`pruefeText`), am Ende eine von Hand gezählte
Bilanz — und **kein Rückgabewert**, ein Skript konnte „lief durch" also nicht
von „hat Fehler gefunden" unterscheiden.

- [x] `ASSERT(bedingung [, meldung$])` und `ASSERT_EQ(ist, soll [, was$])` —
      bei Fehlschlag mit Datei:Zeile abbrechen. Die Fundstelle kommt aus dem
      gewohnten Laufzeitfehler-Pfad, kostet also nichts extra.
- [x] Sammel-Modus (`ASSERT_COLLECT`): alle Prüfungen laufen lassen, am Ende
      `ASSERT_REPORT()` + Exit-Code über `EXIT` aus WP A. Dazu
      `ASSERT_COUNT`/`ASSERT_FAILED`.
- [x] `LOG_DEBUG/INFO/WARN/ERROR(text$)` nach stderr mit Zeitstempel, Pegel
      über `DH_LOG` (`debug`/`info`/`warn`/`error`/`aus`, Vorgabe `info`).

**Entscheidungen:**

- **`ASSERT` bricht per Vorgabe ab**, Sammeln ist die Ausnahme. Ein Assert, der
  nach einer verletzten Vorbedingung stillschweigend weiterläuft, ist
  gefährlicher als einer, der bei der ersten von 119 Prüfungen stehenbleibt —
  Letzteres ist ärgerlich, Ersteres unbemerkt falsch.
- **`ASSERT` verlangt einen `BOOLEAN`.** `ASSERT(anzahl)` ist ein Fehler und
  nicht „wahr, weil nicht null". Eine Prüfung, die aus Versehen immer
  durchgeht, ist schlimmer als gar keine.
- **`ASSERT_EQ` vergleicht mit `value_eq`** — derselben Gleichheit wie der
  `=`-Operator. Eine zweite Vorstellung davon, wann zwei Werte gleich sind,
  wäre die sicherste Art, Vertrauen in die Prüfungen zu verspielen. Nebenbei
  ersetzt die eine Funktion die drei handgeschriebenen Typ-Varianten.
- **Fehlschläge nach stderr, Bilanz nach stdout.** Ein `pruefung > bericht.txt`
  liefert damit einen sauberen Bericht, ohne die Einzelheiten zu verlieren.

**Umsetzung:** `try_pruefen` in `vm.rs` (braucht Zähler, Modus, Quell-Zeile und
den Ausgabe-Puffer). Für die Zeilenangabe im Sammel-Modus gibt es einen
gezielten Nachschlag im `CALL_BUILTIN`-Arm: `cur_line` wird sonst **nur** beim
Profilieren/Debuggen mitgeführt, weil der Normalfall pro Instruktion nichts
zahlen soll — der Test auf das erste Byte des Namens hält die Kosten für alle
anderen Builtins bei einem Byte-Vergleich. `LOG_*` teilt sich `flush_out` mit
`EPRINT`/`SHELL` aus WP A.

Tests: `tests/test_pruefen.py` (27). Beispiel: `examples/165_pruefen.dh`.
Doku: `docs/builtins-core.md`, Abschnitt „Prüfen und Melden".

## WP F — Fehler, die man behandeln kann (✅ ERLEDIGT 2026-08-17)

`THROW` nahm nur einen STRING, `CATCH` bekam nur diesen String. Kein
`FINALLY`, kein Fehler-Code, keine Fundstelle. Für Programme, die Aufräumen
garantieren müssen — Datei schließen, Transaktion zurückrollen, Gerät
freigeben — war das mühsam und fehleranfällig.

- [x] `FINALLY`-Zweig in `TRY`. Läuft bei **allen** Auswegen: sauberer
      Durchlauf, gefangener Fehler, weitergereichter Fehler, Fehler im `CATCH`,
      `RETURN`, `BREAK`, `CONTINUE`. Verschachtelt von innen nach außen.
- [x] Fundstelle im `CATCH`: `ERROR_LINE()`.
- [x] Fehler-Code neben der Meldung: `THROW code$, meldung$` plus
      `ERROR_CODE$()` — damit ein `CATCH` entscheiden kann, statt Meldungstexte
      zu vergleichen. Eingebaute Laufzeitfehler haben `""`.

**Nicht umgesetzt** (aus der Planung gestrichen, mit Grund):

- **`ERROR_FILE$()`** — `IMPORT` fügt textuell zu *einer* Quelle zusammen, es
  gibt zur Laufzeit also nur eine Datei. Der Rückgabewert wäre immer derselbe.
- **`ERROR_TRACE$()`** (Aufruf-Pfad) — dafür müsste die VM bei jedem Aufruf
  eine Rahmen-Kette mitführen. Das kostet im Normalfall, in dem niemand einen
  Trace will. Stand oben schon als „wenn machbar".

**Der Entwurf in einem Satz:** der Fehler-Zweig eines `FINALLY` braucht
**keinen eigenen Handler-Typ** — der Abwickler in `vm.rs` tut für `CATCH` und
`FINALLY` genau dasselbe (Stack zurechtstutzen, Fehlerwert auflegen, springen),
nur das Sprungziel ist ein anderes. Damit kam WP F mit **einem** neuen Opcode
aus (`FIN_END` = Fehler weiterwerfen); der Rest ist erzeugter Code.

Der `FINALLY`-Block steht zweimal im Bytecode (normaler Weg / Fehler-Weg).
Das ist Absicht: die Alternative wäre ein Rücksprung-Mechanismus, und der
kostet zur *Laufzeit* etwas, während die Verdopplung nur Platz kostet.

**Der Teil, der wirklich Sorgfalt brauchte,** ist `RETURN`/`BREAK`/`CONTINUE`
aus dem `TRY` heraus. Ein `FINALLY`, das beim normalen Durchlauf läuft, bei
`RETURN` aber übersprungen wird, wäre schlimmer als keines — man verließe sich
darauf. Dafür führt der Compiler jetzt einen `try_stack` (`TryRahmen`) statt
eines bloßen Zählers und räumt vor jedem vorzeitigen Ausgang von innen nach
außen ab: erst die Handler entfernen, dann den `FINALLY`-Block einsetzen. Der
Rückgabewert wird **vor** dem Abräumen berechnet, damit `FINALLY` ihn nicht
mehr ändern kann.

**Ein Fehler, den erst der eigene Test fand:** `FIN_END` warf den Fehler als
*frischen* Fehler weiter — damit setzte die VM `ERROR_LINE` auf das Ende des
`FINALLY`-Blocks und leerte `ERROR_CODE$`. Ein `FINALLY` dazwischen löschte
also genau die Angaben, wegen derer man sie abfragt. Behoben über ein
`rethrow`-Flag, das die Fehlerbeobachtung überspringt.

Mitgezogen: der **Python-Parser** (`drachenhauch/parser.py`, `ast_nodes.py`,
`tokens.py`) — er dient dem Editor/LSP, und `test_rust_parser_parity.py`
vergleicht beide ASTs Feld für Feld.

Tests: `tests/test_finally.py` (25). Beispiel: `examples/166_aufraeumen.dh`.
Doku: `docs/sprache.md`, Abschnitt „Try / Catch / Throw".

## WP G — Vererbung rund machen (✅ ERLEDIGT 2026-08-17)

`EXTENDS` gab es, die Methodensuche lief die Elternkette hoch. Aber es gab
**kein `SUPER`** — die überschriebene Elternmethode war nicht mehr erreichbar.
In `docs/sprache.md` stand das Ergebnis wörtlich im Beispiel: „Eigene Init,
ruft super.Init nicht automatisch auf" und dann drei Zeilen abgeschriebene
Zuweisungen. Genau dieses Beispiel ist jetzt ersetzt.

- [x] `SUPER.Methode(...)` innerhalb einer Methode
- [x] `ABSTRACT`-Methoden (angekündigt, ohne Rumpf; `NEW` auf eine Klasse mit
      offener Ankündigung ist ein Fehler) — der kleine Bruder von Interfaces,
      ohne neues Typ-Konzept

**Beide sind bewusst KEINE neuen Schlüsselwörter.** `SUPER` ist ein Identifier
mit Sonderbehandlung (genau wie `Self` es schon war), `ABSTRACT` wird über eine
Vorausschau im `CLASS`-Rumpf erkannt (`abstract` + `SUB`/`FUNCTION`). Ein neues
reserviertes Wort hätte `DIM abstract AS …` in bestehendem Code zum Fehler
gemacht — die Liste der reservierten Wörter ist in `docs/sprache.md` bereits
als Stolperstein dokumentiert, sie soll nicht ohne Not wachsen. Zwei Tests
halten fest, dass beide Namen weiter als Variablen taugen.

**Der Kern von `SUPER`:** ein eigener Opcode `CALL_SUPER`, der sich von
`CALL_METHOD` nur darin unterscheidet, **wo die Suche beginnt** — bei der
Elternklasse *der Stelle im Quelltext* (fest im Bytecode) statt bei der Klasse
des Objekts. Über `CALL_METHOD` fände sie wieder die überschreibende Methode,
also sich selbst, bis der Stapel voll ist. Weil der Startpunkt statisch ist,
funktioniert es auch über drei Ebenen und über Zwischenklassen hinweg, die
nichts überschreiben (Tests für beides).

**`ABSTRACT` prüft beim Übersetzen, nicht zur Laufzeit.** Der Compiler kennt
alle Klassen — es gibt keinen Grund, damit bis zum Programmstart zu warten.
`offene_abstracts()` läuft die Vererbung von der Klasse nach oben: was eine
abgeleitete Klasse ausfüllt, gilt weiter oben als erledigt. Die Meldung nennt
alle offenen Namen auf einmal. Eine angekündigte Methode steht trotzdem als
leere Methode in der Klasse, damit ein Aufruf auflöst — so kann die Basisklasse
mit Methoden arbeiten, die es bei ihr noch gar nicht gibt (`Zeige()` ruft
`Flaeche()`).

Mitgezogen: der **Python-Parser** (`parser.py`, `ast_nodes.py`) —
`test_rust_parser_parity.py` vergleicht beide ASTs Feld für Feld, `ClassDecl`
hat jetzt in beiden ein `abstracts`.

Tests: `tests/test_vererbung.py` (20). Beispiel: `examples/167_vererbung.dh`.
Doku: `docs/sprache.md`, Abschnitt „Klassen und Strukturen".

## WP H — Nebenläufigkeit (teilweise erledigt 2026-08-17)

Coroutines sind kooperativ und frame-getrieben. Nur HTTP hatte einen
Hintergrund-Pfad (`HTTP_GET_START`/`READY`/`RESULT`); ein langer SQL-Lauf fror
das Fenster ein.

**Randbedingung, die den Entwurf bestimmt:** raylib will den Hauptthread, und
die VM ist nicht threadsicher. Ein Arbeitsthread darf darum weder zeichnen noch
VM-Zustand anfassen.

- [x] Das `HTTP_GET_START`-Muster verallgemeinert: `hintergrund.rs` hält den
      Auftrags-Speicher **einmal** (Tombstone-Vec, `start`/`fertig`/`abholen`/
      `abbrechen`/`offen`), statt ihn ein drittes Mal nachzubauen.
- [x] **`DB_QUERY_START`/`READY`/`RESULT`/`CANCEL`/`PENDING`** — Abfragen im
      Hintergrund.
- [x] **`SHELL_START`/`READY`/`RESULT$`/`CANCEL`/`PENDING`** plus
      `SHELL_CODE()`/`SHELL_ERR$()` (ohne Argument, für den zuletzt abgeholten
      Auftrag — wie `HTTP_STATUS()`). War nicht geplant, gehört aber genau
      hierher: `SHELL` aus WP A blockiert bis zum Ende des Kindprogramms.
- [n] **`net` brauchte nichts.** Gemessen statt angenommen: die Sockets sind
      laut `docs/module-net.md` **non-blocking by default**, `NET_RECV`/`ACCEPT`
      kehren sofort zurück. Nur `NET_TCP_CONNECT` blockiert einmalig, und das
      mit einem festen 5-Sekunden-Deckel. Die Zeile oben war falsch.
- [ ] **`TASK_START(fnref, arg)` — NICHT umgesetzt, und zwar mit Grund.**

**Warum GB-Code nicht im Hintergrund laufen kann.** `Value` hält Zeichenketten,
Arrays, Maps und Objekte durchgehend in `Rc` (28 Stellen in `value.rs`), und
`Func` hält `Vec<Value>` als Parameter-Vorgaben — `Program` ist damit weder
`Send` noch `Sync`. Eine GB-Funktion in einem Thread auszuführen hieße also,
`Value` (und mit ihm `Func`/`Program`) auf `Arc` umzustellen. Das verteuert
**jede** Zeichenketten- und Array-Operation in **jedem** einthreadigen
Programm, um einem seltenen Fall zu helfen.

Es gäbe einen Umweg — dem Arbeitsthread das Programm-JSON mitgeben, damit er
sich seine eigene VM baut. Der hat aber eine unangenehme Kante: eine frische VM
hat **keine initialisierten Globals**, und eine `CONST` auf oberster Ebene ist
ein Global. Eine „reine" Funktion, die eine Konstante benutzt, sähe dort einen
Vorgabewert. Das ist keine Beschränkung, die man nebenbei einführt.

Beides ist eine eigene Entscheidung, keine Fleißarbeit — darum bleibt der Punkt
offen und ehrlich als offen markiert.

Tests: `tests/test_hintergrund.py` (17), darunter der Beleg, dass ein Auftrag
die offene Transaktion des Programms **nicht** sieht (eigene Verbindung).
Beispiel: `examples/168_hintergrund.dh` — Abfrage und Prozess laufen, die
Hauptschleife dreht sich 692 mal weiter. Doku: `docs/module-db.md`,
`docs/builtins-core.md`.

## WP I — Namensräume und Module (✅ VOLLSTÄNDIG ERLEDIGT 2026-08-19)

> **Entwurf liegt vor: [entwurf-namensraeume.md](entwurf-namensraeume.md).**
> Dort steht der Befund mit Messungen, ein Vorschlag für Syntax und
> Semantik, ein Bauweg ohne VM-Änderung, ein Stufenplan und die vier
> Fragen, die zu entscheiden waren — **alle vier beantwortet**, siehe
> Abschnitt 6 dort. Gebaut sind **I.1** (`IMPORT "x.dh" AS x`, `PRIVATE`,
> Abschottung gegen die Globals des Hauptprogramms) und **I.4**
> (Meldungen mit Datei und Zeile). Seit I.2 auch Klassen und Structs als Typ und hinter `NEW`.
> Offen: I.3 (ENUMs).
>
> **Stufe I.4 ist gebaut (2026-08-17):** Namenskollisionen nennen jetzt
> **beide** Dateien und Zeilen, und alle Meldungen der Übersetzungs-Phasen
> zeigen auf die Datei, die der Nutzer vor sich hat — statt in die gemergte
> Quelle. Dafür liefert `preprocess` eine Zeilen-Herkunftstabelle mit, die
> zugleich Schritt 1 des Bauwegs für I.1 ist. Tests:
> `tests/test_import_meldungen.py` (9).

`IMPORT "datei.dh"` ist textuelles Inkludieren in einen flachen globalen
Namensraum, in dem bereits **1316 Builtins** liegen. `IMPORT ... AS alias` ist
kein Namensraum, sondern Präfix-Kopieren, und funktioniert nur, wenn der
Builtin-Präfix zufällig dem Modulnamen entspricht (bei `imgfx`, das `IMAGE_*`
registriert, schon nicht — steht so in CLAUDE.md).

Folgen: keine getrennte Übersetzung, keine fremde Bibliothek ohne
Kollisionsrisiko, kein `PRIVATE`, keine Paketverwaltung. Unterhalb von ein paar
tausend Zeilen fällt das nicht auf; darüber ist es die Hauptbremse.

**Dieser Punkt hat als einziger Rückwirkung auf allen bestehenden Code** — er
gehört nicht nebenbei erledigt, sondern braucht einen eigenen Entwurf. Skizze
als Ausgangspunkt:

- Datei = Modul, Name aus dem Dateinamen; `IMPORT "mathe.dh" AS mathe` macht
  Namen als `mathe.Distanz(...)` erreichbar statt sie einzumischen
- `PRIVATE` vor `SUB`/`FUNCTION`/`DIM` hält Namen im Modul
- Ohne `AS` bleibt das heutige Verhalten (Einmischen) — sonst bricht jedes
  bestehende Programm
- Built-in-Module bleiben flach; sie umzustellen wäre eine zweite, größere
  Entscheidung

## WP J — Kleinkram mit Wirkung

- [x] **CSV lesen/schreiben** (2026-08-19): `CSV_PARSE`/`CSV_LOAD`/
      `CSV_FORMAT$`/`CSV_SAVE`/`CSV_ROW$`. RFC 4180 mit frei wählbarem
      Trennzeichen, `\r\n` und `\n`, BOM-Abschnitt und einer Toleranz für
      kaputte Dateien (fehlende Schluss-Anführung liest bis zum Ende, statt
      den Import abzubrechen). Zeilen ungleicher Länge werden auf die breiteste
      aufgefüllt. Neun Rust-`#[test]`s für die Zerlegung, zehn Golden-Tests für
      den Weg durch die Sprache, [examples/169_csv.dh](../examples/169_csv.dh)
- [x] **ZIP lesen/schreiben** (2026-08-19): `ZIP_LIST`/`ZIP_READ`/`ZIP_READ$`/
      `ZIP_EXTRACT`/`ZIP_CREATE`/`ZIP_WRITE`. Beim Entpacken geht jeder
      Eintragsname durch eine Zip-Slip-Prüfung — ein Archiv darf
      `../../autoexec.bat` heißen, und solche Einträge werden übersprungen
      statt geschrieben. Drei Rust-`#[test]`s für die Namensprüfung, neun
      Golden-Tests, davon zwei mit einem echten bösartigen Archiv
- [x] **MAP beschleunigt** (2026-08-19) — der Punkt war falsch zugeschnitten.
      Nachgemessen war nicht nur der TUPLE-Behelf O(n), sondern `GbMap` selbst:
      ein `Vec` mit linearer Suche. 20 000 Einträge kosteten 224 ms zum Füllen
      und 187 ms zum Lesen; mit einem Hash-Index daneben sind es 8 bzw. 7 ms,
      und das Wachstum ist linear statt quadratisch. Das half jedem
      bestehenden Programm, während ein `SET` auf der alten Grundlage genauso
      langsam gewesen wäre
- [ ] Ein eigener `SET`-Typ **oder** MAP mit INTEGER-Schlüssel — die Frage
      stellt sich nach der Beschleunigung neu. Der Geschwindigkeitsgrund ist
      weg; es bleibt die Frage, ob `SET` als eigener Typ die Absicht besser
      ausdrückt, und ob INTEGER-Schlüssel das Sparen der `STR$()`-Umwege wert
      sind
- [x] `docs/editor.md` (Abschnitt „Debugger") behauptete, der Debugger laufe auf
      dem Python-Tree-Walker. Der ist seit Stufe B entfernt; er läuft längst über
      `dhrt debug`. **Korrigiert 2026-08-17** — und beim Nachmessen der einen
      Aussage („`INPUT` liefert im Debugger EOF") kam ein echter Fehler heraus:
      `INPUT` schrieb seinen Prompt **roh auf stdout**, also mitten in den
      JSON-Protokollstrom, und las dann von **stdin**, also aus dem
      Kommando-Kanal des Debuggers. Eine Debug-Sitzung mit `INPUT` lief danach
      aus dem Tritt. Beide Wächter deckten nur den Profiler ab, nicht den
      Debugger (`flush_and_prompt`, `read_input_line` in `vm.rs`). Behoben,
      zwei Tests in `tests/test_dhrt_debug.py` halten es fest

## Python-Parser entfernen (Entwurf liegt vor)

> **[entwurf-python-parser-entfernen.md](entwurf-python-parser-entfernen.md).**
> Beim Ausführen ist Python längst nicht beteiligt — ein zweiter Parser lebt
> aber weiter und kostet bei jeder Sprachänderung Doppelarbeit (zuletzt bei
> `PRIVATE` und den punktierten Typnamen). Gemessen: 3196 Zeilen, zwei echte
> Nutzer, und der Editor-Rückfall greift nur, wenn `dhrt` gar nicht gebaut ist
> — also wenn ohnehin nichts läuft. Die Arbeit steckt nicht im Löschen,
> sondern darin, 11 Testdateien vorher zu triagieren. **Triage erledigt**
> (Abschnitt 3): es sind ~20 Einzelstellen, nicht 2639 Zeilen — meine erste
> Schätzung nach Dateigröße lag um eine Größenordnung daneben.

## Plattform

Die CI baut und testet nur unter Windows; für Linux und macOS läuft lediglich
`cargo check` (README.md). `wifi` ist ohnehin Windows-only. Solange
Drachenhauch Spiele exportiert, ist das vertretbar. Sobald es Werkzeuge bauen
soll, die anderswo laufen sollen, wird ein echter Build- und Testlauf auf
mindestens Linux fällig.

---

## Empfohlene Reihenfolge

**~~A~~ → ~~B~~ → ~~C~~ → ~~D~~ → ~~E~~ → ~~F~~ → ~~G~~ → H → I** (A bis G erledigt, H teilweise — der `TASK_*`-Teil braucht eine eigene Entscheidung, siehe dort)

Die Begründung ist durchgehend „wie viele neue Programme wird das möglich
machen, pro Aufwand":

- **A** ist klein und macht aus Drachenhauch schlagartig eine Skriptsprache.
  Ohne A bleibt jedes Programm eine Insel — das ist der eine Punkt, an dem
  „Tausendsassa" heute konkret scheitert.
- **B** ist die Voraussetzung für C, D und ZIP.
- **C** und **D** sind je ein überschaubarer Brocken und öffnen zusammen die
  gesamte Welt der angemeldeten Web-Dienste.
- **E** zahlt sich ab dann bei jedem weiteren Schritt selbst zurück.
- **F** und **G** sind Bequemlichkeit für großen Code, kein Türöffner.
- **H** wird erst dringend, wenn Programme länger laufen als ein Frame.
- **I** zuletzt, weil es als einziges den Bestand anfasst.

## Bewusst nicht auf der Liste

- **Generics.** Passt nicht zu BASIC-Lesbarkeit; `ARRAY OF ANY` plus
  Laufzeit-Typprüfung (`TYPEOF`) deckt die realen Fälle.
- **`async`/`await`.** Zwei Nebenläufigkeits-Modelle nebeneinander (Coroutinen
  *und* async) wären ein eigener Stolperstein; WP H reicht.
- **Eine Paketverwaltung.** Ohne WP I hätte sie nichts, was sie verwalten
  könnte.
- **Ein eigener HTTP-Server.** `net` kann TCP annehmen (`NET_TCP_LISTEN`);
  wer wirklich einen Dienst braucht, stellt einen fertigen Server davor.
