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

## WP C — HTTP für echte Dienste

Heute: nur `HTTP_GET(url$)`, `HTTP_POST(url$, body$)`, `HTTP_DOWNLOAD`. **Keine
eigenen Request-Header**, kein PUT/PATCH/DELETE, Timeout fest bei 10 Sekunden,
keine Sitzungen. Damit ist praktisch jede angemeldete REST-Schnittstelle außer
Reichweite.

`docs/module-html.md` rät im Abschnitt „Grenzen" selbst zu „Token-basierte Auth
via Header" — eine API, die es nicht gibt. Das ist der klarste Beleg, dass hier
etwas fehlt.

- [ ] `HTTP_REQUEST(methode$, url$, body$, header AS MAP OF STRING)` als *eine*
      allgemeine Funktion; `HTTP_GET`/`HTTP_POST` bleiben als Kurzform
- [ ] `HTTP_TIMEOUT(sekunden)`
- [ ] Rumpf als `BUFFER` senden und empfangen (nach WP B) — heute erzwingt
      UTF-8-mit-Ersetzung den Umweg über `HTTP_DOWNLOAD` in eine Datei
- [ ] Hintergrund-Variante (`HTTP_GET_START`-Muster) für alle Methoden, nicht
      nur GET
- [ ] Doku nachziehen: `docs/module-html.md` sagt „alles aus der Python-
      Standardbibliothek" — seit Stufe B ist es Rust; und der Header-Rat in
      „Grenzen" wird erst mit diesem WP wahr

## WP D — Identität und Prüfsummen

Vorhanden sind `CRC32`, `HASH` (FNV-1a) und Base64 — Prüfsummen fürs
Wiedererkennen, nichts für Vertrauen. Es fehlt alles, was eine Anmeldung, eine
Signatur oder eine stabile eindeutige Nummer braucht.

- [ ] `SHA256$`, `SHA1$`, `MD5$` — je über STRING und über BUFFER
- [ ] `HMAC_SHA256$(schluessel, daten)` — Web-APIs, Webhooks
- [ ] `UUID4$()`
- [ ] `RANDOM_BYTES(n)` → BUFFER, aus der Betriebssystem-Quelle (**nicht** aus
      dem Spiel-PRNG — der ist gesät und vorhersagbar, das ist hier ein Fehler
      und kein Detail)

## WP E — Prüfen und Melden

`buch-tippspiel/code/tippspiel_pruefung.dh` ist ein von Hand geschriebener
Ersatz für einen Testrahmen. Dass es ihn gibt, ist der beste Beleg, dass er
fehlt — und er zahlt sich bei jedem weiteren WP hier selbst zurück.

- [ ] `ASSERT(bedingung, meldung$)` und `ASSERT_EQ(ist, soll [, meldung$])` —
      bei Fehlschlag mit Datei:Zeile abbrechen
- [ ] Sammel-Modus: alle Prüfungen laufen lassen, am Ende Bilanz + Exit-Code
      ungleich 0 (braucht `EXIT` aus WP A)
- [ ] `LOG_INFO/WARN/ERROR(text$)` nach stderr mit Zeitstempel, Pegel über
      Umgebungsvariable einstellbar (braucht `GETENV$` aus WP A)

## WP F — Fehler, die man behandeln kann

`THROW` nimmt nur einen STRING, `CATCH` bekommt nur diesen String. Kein
`FINALLY` (verifiziert: im Lexer nicht vorhanden), kein Fehler-Code, keine
Fundstelle. Für Programme, die Aufräumen garantieren müssen — Datei schließen,
Transaktion zurückrollen, Gerät freigeben — ist das mühsam und fehleranfällig.

- [ ] `FINALLY`-Zweig in `TRY`
- [ ] Fundstelle im `CATCH`: `ERROR_LINE()`, `ERROR_FILE$()`, wenn machbar ein
      Aufruf-Pfad `ERROR_TRACE$()`
- [ ] Fehler-Code neben der Meldung: `THROW code$, meldung$` plus
      `ERROR_CODE$()` — damit ein `CATCH` entscheiden kann, statt Meldungstexte
      zu vergleichen

## WP G — Vererbung rund machen

`EXTENDS` gibt es, die Methodensuche läuft die Elternkette hoch
(`compiler.rs`). Aber es gibt **kein `SUPER`** — die überschriebene
Elternmethode ist nicht mehr erreichbar. In `docs/sprache.md` steht das
Ergebnis wörtlich im Beispiel: „Eigene Init, ruft super.Init nicht automatisch
auf" und dann drei Zeilen abgeschriebene Zuweisungen.

- [ ] `SUPER.Methode(...)` innerhalb einer Methode
- [ ] `ABSTRACT`-Methoden (deklariert, ohne Rumpf; `NEW` auf die Klasse ist ein
      Fehler) — der kleine Bruder von Interfaces, ohne neues Typ-Konzept

## WP H — Nebenläufigkeit

Coroutines sind kooperativ und frame-getrieben. Nur HTTP hat einen
Hintergrund-Pfad (`HTTP_GET_START`/`READY`/`RESULT`); `db`, `net` und `serial`
nicht. Ein langer SQL-Lauf friert das Fenster ein.

**Randbedingung, die den Entwurf bestimmt:** raylib will den Hauptthread, und
die VM ist nicht threadsicher. Ein Arbeitsthread darf darum weder zeichnen noch
VM-Zustand anfassen.

- [ ] Kurzfristig, billig: das `HTTP_GET_START`-Muster auf `db` (lange
      Abfragen) und `net` übertragen — dasselbe Polling wie `INPUT_UPDATE`
- [ ] Mittelfristig: `TASK_START(fnref, arg)` → ID, `TASK_READY(id)`,
      `TASK_RESULT(id)`. Aufgabe = **reine** Funktion, Argumente und Ergebnis
      werden **kopiert** übergeben. Keine geteilten Arrays, keine Objekte, kein
      Zeichnen im Task — das ist die Beschränkung, die den Rest sicher macht

## WP I — Namensräume und Module (der große Brocken)

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

- [ ] CSV lesen/schreiben, mit Anführungszeichen und Trennzeichen korrekt — der
      häufigste Datenaustausch überhaupt, heute Handarbeit mit `SPLIT$`
- [ ] ZIP lesen/schreiben (nach WP B) — Sicherungen, Belegsammlungen, Export
- [ ] Ein `SET`-Typ **oder** MAP mit INTEGER-Schlüssel. Heute sind Map-Schlüssel
      immer STRING; die Set-Comprehension liefert ersatzweise ein
      dedupliziertes TUPLE (steht so in CLAUDE.md) — brauchbar, aber O(n)
- [ ] `docs/editor.md` (Abschnitt „Debugger") behauptet, der Debugger laufe auf
      dem Python-Tree-Walker. Der ist seit Stufe B entfernt; der Debugger läuft
      längst über `dhrt debug` (`editor_qt/debugger.py`). Reiner Doku-Fehler,
      aber einer, der beim Lesen abschreckt

## Plattform

Die CI baut und testet nur unter Windows; für Linux und macOS läuft lediglich
`cargo check` (README.md). `wifi` ist ohnehin Windows-only. Solange
Drachenhauch Spiele exportiert, ist das vertretbar. Sobald es Werkzeuge bauen
soll, die anderswo laufen sollen, wird ein echter Build- und Testlauf auf
mindestens Linux fällig.

---

## Empfohlene Reihenfolge

**~~A~~ → ~~B~~ → C → D → E → F → G → H → I** (A und B sind erledigt, Nächstes ist C)

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
